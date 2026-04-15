// SPDX-License-Identifier: AGPL-3.0-or-later
//! Loopback authentication primitives — bearer token, Host +
//! Origin header allowlist, and the axum middleware layer that
//! applies all three checks on every request except `/health`.
//!
//! Sprint 16 Phase A (D1 — defense en profondeur loopback).
//!
//! ## Three checks
//!
//! 1. `X-SBFB-Token: <hex>` must match the daemon's 256-bit
//!    token, loaded from `~/.sbfb/auth_token` at boot.
//! 2. `Host:` must resolve to a loopback name — `localhost`,
//!    `127.0.0.1`, or `[::1]` — optionally with a port. Blocks
//!    DNS rebinding (CVE-2025-49596 Anthropic MCP Inspector,
//!    CVSS 9.4).
//! 3. `Origin:` is either absent (CLI / curl) or a loopback
//!    HTTP URL (the React shell served from `http://localhost:*`).
//!    Blocks cross-origin fetches from malicious pages or
//!    extensions with `host_permissions: "http://localhost/*"`.
//!
//! `/health` is exempted so a launcher probe or a monitoring
//! loop does not need to know the token.
//!
//! The token is produced on first boot by `nexus-launcher`
//! (see `crates/nexus-launcher/src/auth.rs`). The daemon and
//! coordinator both read it from the same on-disk path on
//! startup — rotation is a "delete the file, restart" flow,
//! deliberately identical to BOINC, Jupyter, and Syncthing.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

/// Name of the HTTP header carrying the bearer token. Lowercase
/// so axum `HeaderMap::get` matches regardless of the client's
/// case (HTTP/2 lowercases everything, HTTP/1.1 is case-insensitive
/// but axum stores names lowercased in `HeaderMap`).
pub const AUTH_HEADER: &str = "x-sbfb-token";

/// Environment variable the daemon / coordinator read at boot to
/// discover a token written by the launcher. If unset, the
/// daemon falls back to the `auth_token_path()` on disk.
pub const AUTH_TOKEN_ENV: &str = "SBFB_AUTH_TOKEN";

/// Length of the hex-encoded token (256 bits / 4 bits per hex
/// char). Used by [`generate_token`] and by the validator to
/// reject tokens of a wrong shape early.
pub const TOKEN_HEX_LEN: usize = 64;

// =================================================================
// Paths
// =================================================================

/// Return the path to the `.sbfb` security root for the current
/// user. Honours the `SBFB_HOME` env override so integration
/// tests can redirect the token + consent + usage files at a
/// throwaway directory without touching the developer's real
/// `~/.sbfb/`.
///
/// Falls back to `$HOME/.sbfb` (Unix) or `%USERPROFILE%\.sbfb`
/// (Windows). Returns `None` only on the rare platform where
/// neither the override nor the home dir resolves.
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

/// Return `<sbfb_home>/auth_token` — the plaintext hex token
/// the launcher writes at first boot.
pub fn auth_token_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("auth_token"))
}

/// Return `<sbfb_home>/canary-key.key` — the maintainer's
/// persistent Ed25519 signing key for the Sprint 18 Phase E2
/// warrant canary flow.
///
/// This key is deliberately distinct from the daemon's iroh
/// node identity (which is minted fresh on every boot via
/// [`nexus_core_rs::create_node`]). A warrant canary needs a
/// **stable** maintainer pubkey that outlives any single daemon
/// process so verifiers can trust one long-lived pubkey across
/// months of canary publications.
///
/// The file is 32 raw bytes (Ed25519 secret key), created with
/// mode `0600` on Unix via
/// [`nexus_core_rs::KeyPair::load_or_generate`].
pub fn canary_key_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("canary-key.key"))
}

/// Return `<sbfb_home>/run` — the directory that holds Unix
/// Domain Sockets on Linux/macOS. Created with mode `0700` by the
/// launcher at boot (Sprint 16 Phase B). Windows uses kernel
/// Named Pipes instead so the directory is never read on that
/// platform, but the helper still resolves to a stable path so
/// Windows-only tests can opt into a tempdir-backed path under
/// `SBFB_HOME`.
pub fn sbfb_run_dir() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("run"))
}

/// Return the Unix Domain Socket path the daemon binds when
/// running on Unix. `None` only on platforms where neither
/// `SBFB_HOME` nor a home dir resolves.
pub fn daemon_socket_path() -> Option<PathBuf> {
    sbfb_run_dir().map(|d| d.join("daemon.sock"))
}

/// Return the Unix Domain Socket path the coordinator binds when
/// running on Unix.
pub fn coordinator_socket_path() -> Option<PathBuf> {
    sbfb_run_dir().map(|d| d.join("coordinator.sock"))
}

/// Windows Named Pipe name the daemon binds when running on
/// Windows. The `\\.\pipe\` prefix is the kernel Named Pipe
/// namespace; `sbfb-daemon` is the per-application leaf. Tests
/// override the leaf via `SBFB_PIPE_SUFFIX` so two cargo test
/// runs do not collide on the same pipe name.
pub fn daemon_pipe_name() -> String {
    let suffix = std::env::var("SBFB_PIPE_SUFFIX").unwrap_or_default();
    format!(r"\\.\pipe\sbfb-daemon{suffix}")
}

/// Windows Named Pipe name the coordinator binds when running on
/// Windows.
pub fn coordinator_pipe_name() -> String {
    let suffix = std::env::var("SBFB_PIPE_SUFFIX").unwrap_or_default();
    format!(r"\\.\pipe\sbfb-coordinator{suffix}")
}

// =================================================================
// Token
// =================================================================

/// Generate a 256-bit token and return it hex-encoded (64 chars,
/// lowercase). Uses `getrandom` through `rand::rngs::OsRng` for
/// a CSPRNG.
pub fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Load the token from `path`, or generate + persist one if the
/// file is absent. Idempotent: a second call returns the same
/// token. Writes are atomic (tempfile + rename).
///
/// ## Permissions
///
/// - Parent dir `<sbfb_home>` is created with mode `0700` on Unix.
/// - File is written with mode `0600` on Unix.
/// - On Windows the mode bits are ignored; the dir lives under
///   `%USERPROFILE%` which has a default ACL restricting access
///   to the logged-in user + admins. Full DACL hardening is
///   Sprint 17+ (see `RUNTIME_ISOLATION.md`).
pub fn load_or_generate_token(path: &Path) -> std::io::Result<String> {
    if let Some(existing) = read_token_file(path)? {
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
    write_token_file(path, &token)?;
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
    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, perms)
}

// =================================================================
// Header predicates
// =================================================================

/// Return `true` iff the `Host:` header value is a loopback
/// host with an optional port: `localhost`, `127.0.0.1`, or
/// `[::1]`, followed by an optional `:PORT` (u16).
pub fn is_loopback_host(host: &str) -> bool {
    let (host_only, port_opt) = if let Some(rest) = host.strip_prefix('[') {
        // IPv6 literal: `[::1]` or `[::1]:PORT`
        let close = rest.find(']').map(|i| i + 1);
        match close {
            Some(end) => {
                let inside = &rest[..end - 1];
                let tail = &host[end + 1..]; // skip leading '[' and trailing ']'
                let port = tail.strip_prefix(':');
                (inside, port)
            }
            None => return false,
        }
    } else {
        match host.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (host, None),
        }
    };

    let host_ok = matches!(host_only, "localhost" | "127.0.0.1" | "::1");
    if !host_ok {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

/// Return `true` iff the `Origin:` header value is an HTTP
/// loopback URL, optionally with a port, and no path. Reuses
/// the same loopback name allowlist as [`is_loopback_host`].
pub fn is_loopback_origin(origin: &str) -> bool {
    let Some(rest) = origin.strip_prefix("http://") else {
        return false;
    };
    // Accept `http://host` and `http://host:port` — nothing
    // after the authority component.
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority != rest {
        return false;
    }
    is_loopback_host(authority)
}

// =================================================================
// Middleware
// =================================================================

/// State injected into the axum middleware. Validates a request
/// token against either a fixed string captured at boot
/// ([`AuthState::Static`]) or a [`TokenRotator`] kept fresh by a
/// background watcher ([`AuthState::Rotated`]).
///
/// The rotated variant is what makes Sprint 18 Phase D's
/// `tokens.json` rotation actually take effect on the daemon HTTP
/// surface — the launcher writes a new pair every 24 h, the
/// [`TokenRotatorWatcher`] thread re-reads the file via `notify`,
/// and the next request handed to [`auth_required`] validates
/// against the fresh pair (current + previous-during-overlap)
/// without restarting the daemon.
///
/// The static variant is preserved for the pre-rotation boot path
/// (`tokens.json` absent) and for every UDS / Named Pipe accept
/// loop that hands the middleware a placeholder token before
/// applying the [`PeerCredsVerified`] bypass.
#[derive(Debug, Clone)]
pub enum AuthState {
    /// Single token captured at boot. Used by the legacy
    /// (`SBFB_AUTH_TOKEN` env / `auth_token` file) path that
    /// preceded Sprint 18 Phase D rotation, and by the UDS /
    /// Named Pipe accept loops where the bearer check is
    /// always bypassed by [`PeerCredsVerified`].
    Static(String),

    /// Reference-counted handle to the daemon's
    /// [`TokenRotator`]. The middleware reads the inner state on
    /// every request, so a rotation written by the launcher to
    /// `tokens.json` and replayed by [`TokenRotatorWatcher`] is
    /// visible to the next request without a daemon restart.
    Rotated(Arc<RwLock<TokenRotator>>),
}

impl AuthState {
    /// Build a static [`AuthState`]. Backwards-compatible with the
    /// pre-Sprint-18-D-1 callers that built `AuthState::new(token)`
    /// — every existing test continues to compile unchanged.
    pub fn new(token: String) -> Self {
        Self::Static(token)
    }

    /// Build an [`AuthState`] that consults a [`TokenRotator`] on
    /// every request. The `Arc` is cloned cheap into the axum
    /// middleware state ; the underlying `RwLock` is the same
    /// instance the watcher writes to on a `tokens.json` change.
    pub fn rotated(rotator: Arc<RwLock<TokenRotator>>) -> Self {
        Self::Rotated(rotator)
    }

    /// Constant-time predicate the middleware calls on every
    /// request. Returns `true` when `request_token` matches the
    /// static token, the rotator's current, or the rotator's
    /// previous token within the overlap window. A poisoned
    /// `RwLock` (writer thread panicked) collapses to `false`
    /// rather than spreading the panic to the request handler —
    /// the watcher logs warn on poison, the next valid rotation
    /// recovers.
    pub fn validate(&self, request_token: &str) -> bool {
        match self {
            Self::Static(expected) => {
                constant_time_eq(request_token.as_bytes(), expected.as_bytes())
            }
            Self::Rotated(rotator) => match rotator.read() {
                Ok(guard) => validate_token_with_rotator(request_token, &guard),
                Err(_) => false,
            },
        }
    }
}

/// Marker injected by the UDS / Named Pipe accept loop into the
/// request extensions when the OS-level peer credentials match
/// the current user (SO_PEERCRED on Unix, DACL-restricted Named
/// Pipe ACL on Windows). When present, [`auth_required`]
/// bypasses the bearer + Host + Origin checks because the
/// kernel has already authenticated the peer.
///
/// Sprint 16 Phase B (D2 — defense en profondeur).
#[derive(Debug, Clone, Copy)]
pub struct PeerCredsVerified;

/// Axum middleware that enforces the triple check on every
/// request except `/health`. Returns 401 on missing/wrong
/// token, 403 on bad Host or bad Origin.
///
/// ## Bypass for trusted peers
///
/// If the request carries a [`PeerCredsVerified`] extension —
/// injected by the UDS / Named Pipe accept loop after the
/// kernel has authenticated the peer — the middleware skips
/// every header-based check. This lets the CLI and other local
/// processes connect via UDS/NP without reading the bearer
/// token file, while the TCP path stays guarded for the browser.
pub async fn auth_required(
    State(auth): State<AuthState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Exemption: liveness probe bypasses auth. Kept explicit so
    // the grep line in the threat model (Phase E) points at one
    // obvious allowlist entry.
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    // Bypass for UDS / Named Pipe connections that the accept
    // loop has already authenticated via peer credentials. The
    // marker is a private type — a malicious caller cannot inject
    // it from over the wire because axum strips request
    // extensions on the public Request type.
    if req.extensions().get::<PeerCredsVerified>().is_some() {
        return next.run(req).await;
    }

    // 1. Bearer token. `auth.validate` dispatches to the
    //    static-string check or to the rotator (current + previous
    //    during overlap). Sprint 18 audit fix D-1 — same predicate
    //    a launcher-rotated token reaches without a daemon restart.
    let token_ok = req
        .headers()
        .get(AUTH_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|t| auth.validate(t))
        .unwrap_or(false);
    if !token_ok {
        return (StatusCode::UNAUTHORIZED, "missing or invalid token").into_response();
    }

    // 2. Host header allowlist (block DNS rebinding)
    let host_ok = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(is_loopback_host)
        .unwrap_or(false);
    if !host_ok {
        return (StatusCode::FORBIDDEN, "host not allowed").into_response();
    }

    // 3. Origin check: absent is OK (CLI / curl), otherwise
    //    must be a loopback http origin.
    if let Some(origin) = req.headers().get(axum::http::header::ORIGIN) {
        let ok = origin
            .to_str()
            .ok()
            .map(is_loopback_origin)
            .unwrap_or(false);
        if !ok {
            return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
        }
    }

    next.run(req).await
}

/// Constant-time slice compare. The `subtle` crate would be
/// preferable but avoiding a new dep for a 256-bit hex string
/// is cheap enough here — we iterate the fixed length and or
/// the differences.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Ergonomic helper: build a `HeaderValue` for the token header.
/// Used by the daemon / coordinator tests and by the launcher's
/// `/auth/token` handler response.
pub fn header_value_for(token: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(token).ok()
}

// =================================================================
// Token rotation — Sprint 18 Phase D
// =================================================================

/// How long the previous token stays valid after a rotation, so
/// an in-flight request signed with the old token completes
/// instead of failing at 401. 10 minutes is well above the
/// longest P99 request the daemon serves (`/blob-serve/*` on a
/// cold blob), and short enough that a stolen old token cannot
/// be replayed the next day.
pub const TOKEN_OVERLAP_DURATION: Duration = Duration::from_secs(600);

/// Basename of the rotation state file persisted by the launcher
/// at `<sbfb_home>/tokens.json`.
pub const TOKENS_FILE_NAME: &str = "tokens.json";

/// On-disk representation of the rotation state. The launcher
/// owns write access; the daemon reads it to populate its in-
/// memory [`TokenRotator`]. The format is stable across
/// launcher / daemon versions within v1 (pre-launch policy).
///
/// `rotated_at` is the Unix epoch (seconds) of the most recent
/// rotation — persisted across process restarts so an
/// unfortunately-timed launcher crash right after a rotation
/// does not silently shrink the overlap window to zero for a
/// still-running daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TokensFile {
    pub current: String,
    pub previous: Option<String>,
    pub rotated_at: u64,
}

/// Pair of currently-valid tokens plus the instant of the last
/// rotation, used by [`validate_token_with_rotator`] to accept
/// either token during the overlap window.
///
/// The struct holds both a [`std::time::Instant`] (for overlap
/// arithmetic — monotonic, unaffected by wall-clock skew) and a
/// Unix timestamp (for persistence — the launcher can reload
/// state across restarts). Only the launcher mutates this state;
/// the daemon holds a read-only clone.
#[derive(Debug, Clone)]
pub struct TokenRotator {
    current: String,
    previous: Option<String>,
    rotated_at: Instant,
    rotated_at_unix: u64,
}

impl TokenRotator {
    /// Build a rotator with a single active token and no
    /// predecessor. Used at first launcher boot when no
    /// `tokens.json` exists on disk yet.
    pub fn new(current: String) -> Self {
        Self {
            current,
            previous: None,
            rotated_at: Instant::now(),
            rotated_at_unix: unix_now(),
        }
    }

    /// Promote the current token to `previous`, install
    /// `new_current` as the fresh current, and stamp the
    /// rotation instant. Idempotent on the previous slot — a
    /// rotation always overwrites the prior predecessor so the
    /// overlap window never chains past one generation.
    pub fn rotate(&mut self, new_current: String) {
        let prev = std::mem::replace(&mut self.current, new_current);
        self.previous = Some(prev);
        self.rotated_at = Instant::now();
        self.rotated_at_unix = unix_now();
    }

    /// Currently-active token. Always present.
    pub fn current(&self) -> &str {
        &self.current
    }

    /// Predecessor token, or `None` if no rotation has happened
    /// yet *or* the overlap window has elapsed. Callers that
    /// want the raw stored predecessor regardless of window
    /// should go through [`Self::previous_raw`].
    pub fn previous(&self) -> Option<&str> {
        if self.is_in_overlap_window() {
            self.previous.as_deref()
        } else {
            None
        }
    }

    /// Raw predecessor token ignoring the overlap window. Only
    /// used by persistence — [`validate_token_with_rotator`]
    /// goes through [`Self::previous`] which already gates on
    /// the window.
    pub fn previous_raw(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    /// True iff a predecessor is stored *and* the overlap
    /// window has not yet elapsed since the last rotation.
    pub fn is_in_overlap_window(&self) -> bool {
        self.previous.is_some() && self.rotated_at.elapsed() < TOKEN_OVERLAP_DURATION
    }

    /// Unix timestamp (seconds) of the most recent rotation.
    pub fn rotated_at_unix(&self) -> u64 {
        self.rotated_at_unix
    }

    /// Persist the pair atomically (tempfile + rename) so a
    /// crash mid-write never leaves the daemon with a truncated
    /// file. Mode is `0600` on Unix; the parent dir is created
    /// with mode `0700` if absent (same contract as the
    /// long-lived `auth_token` file).
    pub fn write_atomic(&self, path: &Path) -> std::io::Result<()> {
        let payload = TokensFile {
            current: self.current.clone(),
            previous: self.previous.clone(),
            rotated_at: self.rotated_at_unix,
        };
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "tokens.json path has no parent",
            )
        })?;
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        set_mode(parent, 0o700)?;

        let tmp = path.with_extension("tmp");
        let body =
            serde_json::to_string(&payload).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(&tmp, body)?;
        #[cfg(unix)]
        set_mode(&tmp, 0o600)?;
        std::fs::rename(&tmp, path)
    }

    /// Load a rotator from `path`. Returns `Ok(None)` if the
    /// file is absent; callers use [`Self::new`] to seed a
    /// fresh rotator on first boot.
    pub fn load(path: &Path) -> std::io::Result<Option<Self>> {
        let body = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let payload: TokensFile =
            serde_json::from_str(&body).map_err(|e| std::io::Error::other(e.to_string()))?;
        // Reconstruct the monotonic instant from (now - age):
        // age = max(0, now - rotated_at_unix). If the wall clock
        // went backwards for any reason, treat the rotation as
        // "just now" — defensive, and the overlap window simply
        // restarts rather than being perpetually expired.
        let age = unix_now().saturating_sub(payload.rotated_at);
        let rotated_at = Instant::now()
            .checked_sub(Duration::from_secs(age))
            .unwrap_or_else(Instant::now);
        Ok(Some(Self {
            current: payload.current,
            previous: payload.previous,
            rotated_at,
            rotated_at_unix: payload.rotated_at,
        }))
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Path to `<sbfb_home>/tokens.json`. Returns `None` on
/// platforms where neither `SBFB_HOME` nor the user home dir
/// resolves.
pub fn tokens_file_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join(TOKENS_FILE_NAME))
}

/// Constant-time check of `request_token` against the
/// rotator's current token plus, when the overlap window is
/// still open, the immediately-previous token. Returns `true`
/// on match, `false` otherwise.
pub fn validate_token_with_rotator(request_token: &str, rotator: &TokenRotator) -> bool {
    if constant_time_eq(request_token.as_bytes(), rotator.current.as_bytes()) {
        return true;
    }
    if let Some(prev) = rotator.previous() {
        if constant_time_eq(request_token.as_bytes(), prev.as_bytes()) {
            return true;
        }
    }
    false
}

// =================================================================
// File watcher — daemon-side bridge for launcher rotation
// =================================================================
//
// Sprint 18 audit fix D-1.
//
// The launcher's `spawn_rotation_loop` writes a fresh `tokens.json`
// every 24 h. Until D-1, the daemon HTTP layer captured a single
// token at boot and never re-read the file, so the rotation existed
// in the file system but never reached `auth_required`. The watcher
// below closes that gap : a `notify::recommended_watcher` thread on
// the parent dir reloads the file on every Modify/Create event,
// updates the shared `Arc<RwLock<TokenRotator>>`, and the next
// request handed to `auth_required` validates against the fresh
// pair (current + previous-during-overlap). Pattern mirrors
// `nexus_worker_core::consent::ConsentWatcher` — same crate, same
// 50 ms debounce, same parent-dir watch + path-filter to ignore
// sibling files.

/// Live, reload-on-change handle to a [`TokenRotator`]. Pulled out
/// of the watcher via [`TokenRotatorWatcher::shared`] and handed to
/// [`AuthState::rotated`] — the middleware reads the inner state on
/// every request, so a launcher rotation persisted to `tokens.json`
/// is visible to the next HTTP request without restarting the
/// daemon.
///
/// The watcher owns a background thread joined to a private
/// `notify::RecommendedWatcher`. Both are dropped together when the
/// `TokenRotatorWatcher` value goes out of scope ; in production
/// the daemon keeps the value alive on its [`crate::DaemonRuntime`]
/// for the whole process lifetime, so the watcher loop runs until
/// shutdown.
pub struct TokenRotatorWatcher {
    inner: Arc<RwLock<TokenRotator>>,
    /// Underscore : we never read the watcher after construction.
    /// `Drop` on it shuts the inotify / ReadDirectoryChangesW
    /// observer down which in turn closes the channel and stops
    /// the background thread.
    _watcher: notify::RecommendedWatcher,
    /// Joined on `Drop` for clean teardown ; tests rely on the
    /// watcher thread exiting before the tempdir disappears.
    _join: Option<std::thread::JoinHandle<()>>,
}

impl TokenRotatorWatcher {
    /// Spawn a watcher for `path`. The file **must** already exist
    /// — the daemon-side caller ([`crate::DaemonRuntime::start`])
    /// only invokes this constructor after a successful
    /// [`TokenRotator::load`], so a missing file is never a runtime
    /// surprise. Tests construct from an existing tempfile.
    ///
    /// The first read happens synchronously in the caller via
    /// [`TokenRotator::load`] ; the watcher only handles
    /// **subsequent** rewrites. This split keeps the constructor
    /// total : if the file disappears later the in-memory state
    /// stays as the last successfully-loaded snapshot (mirroring
    /// the `consent.json` C-4 fix from Sprint 16 — a deletion does
    /// not silently revert the daemon to "no tokens").
    pub fn spawn(path: PathBuf, initial: TokenRotator) -> notify::Result<Self> {
        use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

        let inner = Arc::new(RwLock::new(initial));

        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;
        // Watch the parent dir, not the file directly : write+rename
        // changes the inode and detaches a file-level watch.
        watcher.watch(&parent, RecursiveMode::NonRecursive)?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let join = std::thread::Builder::new()
            .name("sbfb-tokens-watch".into())
            .spawn(move || {
                while let Ok(evt) = rx.recv() {
                    match evt {
                        Ok(event) => {
                            // notify reports every entry under the
                            // watched dir ; filter to events that
                            // touch our specific path so a sibling
                            // (`auth_token`, `consent.json`, etc.)
                            // does not trigger a reload.
                            if !event.paths.iter().any(|p| p == &path_thread) {
                                continue;
                            }
                            // C-4 pattern : a Remove (user deletes
                            // tokens.json by hand) keeps the last
                            // known rotator state in memory rather
                            // than collapsing to an "no tokens"
                            // default that would lock everyone out.
                            if matches!(event.kind, EventKind::Remove(_)) {
                                tracing::warn!(
                                    path = %path_thread.display(),
                                    "tokens.json removed — keeping in-memory rotator until recreated"
                                );
                                continue;
                            }
                            if !matches!(
                                event.kind,
                                EventKind::Modify(_) | EventKind::Create(_)
                            ) {
                                continue;
                            }
                            // Debounce write+rename : editors and
                            // the launcher's `write_atomic` emit
                            // Create+Modify in quick succession.
                            std::thread::sleep(Duration::from_millis(50));
                            match TokenRotator::load(&path_thread) {
                                Ok(Some(fresh)) => {
                                    if let Ok(mut guard) = inner_thread.write() {
                                        *guard = fresh;
                                        tracing::debug!(
                                            path = %path_thread.display(),
                                            "tokens.json reloaded"
                                        );
                                    } else {
                                        tracing::warn!(
                                            path = %path_thread.display(),
                                            "tokens.json reload skipped — rotator lock poisoned"
                                        );
                                    }
                                }
                                Ok(None) => {
                                    tracing::warn!(
                                        path = %path_thread.display(),
                                        "tokens.json absent during reload — keeping in-memory state"
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        error = %e,
                                        path = %path_thread.display(),
                                        "tokens.json reload failed — keeping in-memory state"
                                    );
                                }
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "tokens watcher event error"),
                    }
                }
            })
            .map_err(|e| notify::Error::generic(&format!("watcher thread spawn failed: {e}")))?;

        Ok(Self {
            inner,
            _watcher: watcher,
            _join: Some(join),
        })
    }

    /// Cheap clone of the shared `Arc<RwLock<TokenRotator>>` — the
    /// same handle [`AuthState::rotated`] consumes. Multiple
    /// callers can hold their own clone ; the watcher continues
    /// updating the inner state until the watcher itself is
    /// dropped.
    pub fn shared(&self) -> Arc<RwLock<TokenRotator>> {
        Arc::clone(&self.inner)
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::to_bytes,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    fn build_router(token: &str) -> Router {
        let auth = AuthState::new(token.to_string());
        Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/protected", get(|| async { "secret" }))
            .layer(middleware::from_fn_with_state(auth, auth_required))
    }

    async fn send(router: Router, req: Request<Body>) -> (StatusCode, String) {
        let resp = router.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn health_is_public() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_rejects_missing_token() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header("host", "127.0.0.1:7777")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn protected_rejects_wrong_token() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "feedface")
            .header("host", "127.0.0.1:7777")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn protected_accepts_token_host_and_no_origin() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "127.0.0.1:7777")
            .body(Body::empty())
            .unwrap();
        let (status, body) = send(router, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "secret");
    }

    #[tokio::test]
    async fn protected_accepts_localhost_host_and_loopback_origin() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "localhost")
            .header("origin", "http://localhost:5173")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_rejects_rebound_host() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "attacker.com")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn protected_rejects_cross_origin() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "127.0.0.1:7777")
            .header("origin", "https://attacker.com")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn protected_rejects_https_loopback_origin() {
        // https://localhost is NOT acceptable — daemon is http-only
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "127.0.0.1:7777")
            .header("origin", "https://localhost:5173")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ipv6_loopback_host_accepted() {
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header(AUTH_HEADER, "deadbeef")
            .header("host", "[::1]:7777")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[test]
    fn token_generation_is_32_bytes_hex() {
        let t = generate_token();
        assert_eq!(t.len(), TOKEN_HEX_LEN);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn load_or_generate_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".sbfb").join("auth_token");
        let a = load_or_generate_token(&path).unwrap();
        let b = load_or_generate_token(&path).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), TOKEN_HEX_LEN);
    }

    #[test]
    fn load_or_generate_rejects_malformed_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth_token");
        std::fs::write(&path, "not-hex!!").unwrap();
        let err = load_or_generate_token(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[cfg(unix)]
    #[test]
    fn load_or_generate_sets_unix_perms() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".sbfb").join("auth_token");
        let _ = load_or_generate_token(&path).unwrap();
        let file_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600, "auth_token must be 0600");
        let dir_mode = std::fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dir_mode, 0o700, "parent dir must be 0700");
    }

    #[test]
    fn is_loopback_host_matches_expected() {
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("localhost:7777"));
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.0.0.1:7777"));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("[::1]:7777"));
        assert!(!is_loopback_host("attacker.com"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("example.com:7777"));
        assert!(!is_loopback_host("localhost:not-a-port"));
    }

    #[test]
    fn is_loopback_origin_matches_expected() {
        assert!(is_loopback_origin("http://localhost"));
        assert!(is_loopback_origin("http://localhost:5173"));
        assert!(is_loopback_origin("http://127.0.0.1:8080"));
        assert!(is_loopback_origin("http://[::1]:7777"));
        assert!(!is_loopback_origin("https://localhost"));
        assert!(!is_loopback_origin("http://attacker.com"));
        assert!(!is_loopback_origin("http://localhost/path"));
    }

    #[test]
    fn constant_time_eq_matches_slice_eq() {
        assert!(constant_time_eq(b"", b""));
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn sbfb_home_honours_override() {
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::var("SBFB_HOME").ok();
        std::env::set_var("SBFB_HOME", dir.path());
        let home = sbfb_home().unwrap();
        assert_eq!(home, dir.path());
        match prev {
            Some(v) => std::env::set_var("SBFB_HOME", v),
            None => std::env::remove_var("SBFB_HOME"),
        }
    }

    #[test]
    fn run_dir_paths_resolve_under_sbfb_home() {
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::var("SBFB_HOME").ok();
        std::env::set_var("SBFB_HOME", dir.path());

        let run = sbfb_run_dir().unwrap();
        assert_eq!(run, dir.path().join("run"));
        let dsock = daemon_socket_path().unwrap();
        assert_eq!(dsock, dir.path().join("run").join("daemon.sock"));
        let csock = coordinator_socket_path().unwrap();
        assert_eq!(csock, dir.path().join("run").join("coordinator.sock"));

        match prev {
            Some(v) => std::env::set_var("SBFB_HOME", v),
            None => std::env::remove_var("SBFB_HOME"),
        }
    }

    #[test]
    fn windows_pipe_names_have_kernel_prefix() {
        // Suffix override is used by the daemon's named pipe tests
        // to avoid collisions with a real running daemon. The
        // production path keeps the leaf stable.
        let prev = std::env::var("SBFB_PIPE_SUFFIX").ok();
        std::env::remove_var("SBFB_PIPE_SUFFIX");

        let d = daemon_pipe_name();
        assert_eq!(d, r"\\.\pipe\sbfb-daemon");
        let c = coordinator_pipe_name();
        assert_eq!(c, r"\\.\pipe\sbfb-coordinator");

        std::env::set_var("SBFB_PIPE_SUFFIX", "-test123");
        assert_eq!(daemon_pipe_name(), r"\\.\pipe\sbfb-daemon-test123");

        match prev {
            Some(v) => std::env::set_var("SBFB_PIPE_SUFFIX", v),
            None => std::env::remove_var("SBFB_PIPE_SUFFIX"),
        }
    }

    #[tokio::test]
    async fn peer_creds_marker_bypasses_bearer() {
        // A request with no token but a PeerCredsVerified extension
        // must reach the handler — it represents a UDS / Named Pipe
        // connection that the kernel has already authenticated.
        async fn inject_marker(mut req: Request<Body>, next: Next) -> Response {
            req.extensions_mut().insert(PeerCredsVerified);
            next.run(req).await
        }
        let auth = AuthState::new("deadbeef".to_string());
        let router: Router = Router::new()
            .route("/protected", get(|| async { "secret" }))
            .layer(middleware::from_fn_with_state(auth.clone(), auth_required))
            .layer(middleware::from_fn(inject_marker));
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        let (status, body) = send(router, req).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "secret");
    }

    #[tokio::test]
    async fn peer_creds_marker_does_not_leak_via_http() {
        // A client cannot inject the marker by sending a header —
        // the bypass relies on a private type added to the request
        // extensions by the accept loop, which the wire format
        // cannot carry.
        let router = build_router("deadbeef");
        let req = Request::builder()
            .uri("/protected")
            .header("x-peer-creds-verified", "1") // attempted spoof
            .header("host", "127.0.0.1:7777")
            .body(Body::empty())
            .unwrap();
        let (status, _) = send(router, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
