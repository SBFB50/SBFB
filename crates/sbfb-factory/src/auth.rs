// SPDX-License-Identifier: AGPL-3.0-or-later
//! Loopback authentication for the Factory Operator HTTP server.
//!
//! Sprint 71 Phase C (D5 — align the Operator on the daemon's
//! Sprint 16 loopback hardening). The Operator `:3001` server writes
//! files (`/api/artifacts/draft`) and spawns `bypassPermissions`
//! agents (`/api/chat/{id}/stream`, `/api/terminal/ws`), so every
//! request must prove it comes from a same-host, same-origin caller
//! holding the shared bearer token.
//!
//! ## Three checks (mirrors `nexus-shell-daemon-core::auth`)
//!
//! 1. `X-SBFB-Token: <hex>` matches the 256-bit token loaded from
//!    `SBFB_AUTH_TOKEN` or `<sbfb_home>/auth_token`.
//! 2. `Host:` resolves to a loopback name (blocks DNS rebinding —
//!    CVE-2025-49596 / CVE-2025-66414 class).
//! 3. `Origin:` is absent (CLI / Vite proxy) or a loopback URL
//!    (blocks cross-site fetches and cross-site WebSocket hijacking
//!    on `/api/terminal/ws`).
//!
//! ## Deliberate duplication (tech debt)
//!
//! The canonical implementation lives in
//! `nexus-shell-daemon-core::auth` (the daemon's loopback layer,
//! Sprint 16). It is re-implemented here — rather than depending on
//! the daemon core — to keep this scaffolding tool free of the
//! daemon's iroh / gossip dependency tree. Both copies are
//! unit-tested. Unifying them into a shared loopback-auth module is
//! tracked tech debt (folded into `docs/shell/PATTERNS.md` at the
//! Sprint 71 wrap-up).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// HTTP header carrying the bearer token. Lowercase to match axum's
/// normalized `HeaderMap` and the daemon's `AUTH_HEADER`.
pub const AUTH_HEADER: &str = "x-sbfb-token";

/// Env var a launcher / test sets to inject the token without a
/// file. Mirrors the daemon's `SBFB_AUTH_TOKEN`.
pub const AUTH_TOKEN_ENV: &str = "SBFB_AUTH_TOKEN";

/// Hex length of a 256-bit token (32 bytes, 2 hex chars per byte).
const TOKEN_HEX_LEN: usize = 64;

/// Auth state cloned into the axum middleware: the expected token.
#[derive(Clone)]
pub struct AuthState {
    token: Arc<String>,
}

impl AuthState {
    pub fn new(token: String) -> Self {
        Self {
            token: Arc::new(token),
        }
    }
}

/// `<sbfb_home>` honouring `SBFB_HOME`, else `$HOME` /
/// `%USERPROFILE%` + `.sbfb`. Same resolution as the daemon and as
/// [`crate::daemon_client`].
pub fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SBFB_HOME") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())?;
    if home.is_empty() {
        return None;
    }
    Some(PathBuf::from(home).join(".sbfb"))
}

/// `<sbfb_home>/auth_token` — the plaintext hex token the launcher
/// writes at first boot, shared with the daemon and coordinator.
pub fn auth_token_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("auth_token"))
}

/// Generate a 256-bit token, hex-encoded (64 lowercase chars), from
/// the OS CSPRNG via `rand::rngs::OsRng`.
fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Resolve the Operator's bearer token: `SBFB_AUTH_TOKEN` env first,
/// then `<sbfb_home>/auth_token` (the same token the daemon uses),
/// then generate + persist one so a standalone Operator still works.
/// Writes are atomic (tempfile + rename), `0600` on Unix.
pub fn load_or_generate_token() -> std::io::Result<String> {
    if let Ok(env_tok) = std::env::var(AUTH_TOKEN_ENV) {
        let trimmed = env_tok.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    let path = auth_token_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot resolve sbfb_home for auth_token",
        )
    })?;

    if let Some(existing) = read_token_file(&path)? {
        return Ok(existing);
    }

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "auth_token path has no parent",
        )
    })?;
    std::fs::create_dir_all(parent)?;
    #[cfg(unix)]
    set_mode(parent, 0o700)?;

    let token = generate_token();
    write_token_file(&path, &token)?;
    Ok(token)
}

fn read_token_file(path: &Path) -> std::io::Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.len() == TOKEN_HEX_LEN && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(Some(trimmed))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "auth_token file exists but is malformed",
                ))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn write_token_file(path: &Path, token: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, token)?;
    #[cfg(unix)]
    set_mode(&tmp, 0o600)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

/// Return `true` iff the `Host:` value is a loopback host with an
/// optional `:PORT`: `localhost`, `127.0.0.1`, or `[::1]`.
pub fn is_loopback_host(host: &str) -> bool {
    let (host_only, port_opt) = if let Some(rest) = host.strip_prefix('[') {
        // IPv6 literal: `[::1]` or `[::1]:PORT`.
        match rest.find(']').map(|i| i + 1) {
            Some(end) => {
                let inside = &rest[..end - 1];
                let tail = &host[end + 1..];
                (inside, tail.strip_prefix(':'))
            }
            None => return false,
        }
    } else {
        match host.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (host, None),
        }
    };

    if !matches!(host_only, "localhost" | "127.0.0.1" | "::1") {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

/// Return `true` iff the `Origin:` value is an HTTP loopback URL with
/// an optional port and no path. Reuses [`is_loopback_host`].
pub fn is_loopback_origin(origin: &str) -> bool {
    let Some(rest) = origin.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority != rest {
        return false;
    }
    is_loopback_host(authority)
}

/// Constant-time byte comparison. The length is not secret (tokens
/// are a fixed 64 hex chars), so an early length check is fine.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Axum middleware enforcing Host + Origin + bearer token on every
/// Operator route. 403 on a non-loopback Host/Origin, 401 on a
/// missing/wrong token.
pub async fn auth_required(State(auth): State<AuthState>, req: Request, next: Next) -> Response {
    let headers = req.headers();

    let host_ok = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(is_loopback_host)
        .unwrap_or(false);
    if !host_ok {
        return (StatusCode::FORBIDDEN, "non-loopback Host rejected").into_response();
    }

    if let Some(origin) = headers.get(header::ORIGIN) {
        let origin_ok = origin
            .to_str()
            .ok()
            .map(is_loopback_origin)
            .unwrap_or(false);
        if !origin_ok {
            return (StatusCode::FORBIDDEN, "non-loopback Origin rejected").into_response();
        }
    }

    let token_ok = headers
        .get(AUTH_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|t| constant_time_eq(t.as_bytes(), auth.token.as_bytes()))
        .unwrap_or(false);
    if !token_ok {
        return (StatusCode::UNAUTHORIZED, "missing or invalid X-SBFB-Token").into_response();
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn loopback_host_accepts_localhost_variants() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.0.0.1:3001"));
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("localhost:5174"));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("[::1]:3001"));
    }

    #[test]
    fn loopback_host_rejects_foreign_and_bad_port() {
        assert!(!is_loopback_host("evil.com"));
        assert!(!is_loopback_host("evil.com:3001"));
        assert!(!is_loopback_host("127.0.0.1:notaport"));
        assert!(!is_loopback_host("169.254.169.254"));
    }

    #[test]
    fn loopback_origin_accepts_loopback_rejects_rest() {
        assert!(is_loopback_origin("http://127.0.0.1:3001"));
        assert!(is_loopback_origin("http://localhost:5174"));
        assert!(!is_loopback_origin("http://evil.com"));
        assert!(!is_loopback_origin("https://localhost")); // https, not http
        assert!(!is_loopback_origin("http://localhost/evil")); // path present
        assert!(!is_loopback_origin("http://127.0.0.1.evil.com")); // suffix trick
    }

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }

    #[test]
    #[serial(sbfb_env)]
    fn env_token_takes_precedence_over_file() {
        // `#[serial(sbfb_env)]` (P2-A-1 review P1): nextest isolates per
        // process, and this serializes the env mutation under plain
        // `cargo test` so it never races the other env-mutating tests.
        let expected = "a".repeat(TOKEN_HEX_LEN);
        unsafe { std::env::set_var(AUTH_TOKEN_ENV, &expected) };
        let token = load_or_generate_token().expect("env token");
        assert_eq!(token, expected);
        unsafe { std::env::remove_var(AUTH_TOKEN_ENV) };
    }
}
