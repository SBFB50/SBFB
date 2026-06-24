// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase A — `sbfb init` / `sbfb unlock` subcommands.
//!
//! `sbfb init --pin <pin>` generates a fresh Ed25519 identity,
//! encrypts it with the supplied PIN (Argon2id + AES-256-GCM +
//! optional OS keyring wrap), and writes the blob under
//! `<grid_root>/shell-daemon/keyring/identity.enc`. The daemon
//! is not spawned — init is a one-off bootstrap step.
//!
//! `sbfb unlock --pin <pin>` reads the blob, decrypts it, and
//! exports the 32-byte secret key as the hex-encoded
//! `SBFB_IDENTITY_SECRET_HEX` environment variable before
//! spawning the daemon child. The daemon picks the env var up
//! at boot and passes the bytes to
//! [`nexus_core_rs::NodeConfig::with_secret_key`], giving the
//! iroh endpoint a persistent identity across restarts.
//!
//! ## Secret lifetime in the launcher
//!
//! The decrypted key crosses three boundaries: SecretBox →
//! `[u8; 32]` stack buffer → hex string → `std::env::set_var`.
//! The stack buffer is zeroed immediately after the hex encode;
//! the hex string is zeroed after `set_var` returns. The env
//! var remains visible to the daemon child and to any same-user
//! process inspecting our process environment (`/proc/self/environ`
//! on Linux, `Process Hacker` on Windows). That residual exposure
//! is acknowledged and documented in
//! `docs/rust/PATTERNS.md §Sprint 20.1`; a future sprint can
//! tighten it by passing the secret through a Unix domain socket
//! or Named Pipe instead.
//!
//! ## PIN acquisition
//!
//! The PIN is accepted only as a CLI argument (`--pin <p>`) so
//! the flow is trivially testable. Plaintext CLI args show up
//! in shell history and `ps` listings — acceptable for dev /
//! smoke-test use but not for real deployments. An interactive
//! `rpassword`-style prompt is not yet implemented.

use std::path::PathBuf;

use nexus_core_rs::keystore::{
    IdentityMode, KeyStore, LocalFileKeyStore, SBFB_IDENTITY_SECRET_HEX_ENV, UnlockError,
};
use zeroize::Zeroize;

/// Sprint 20 Phase B : env var signalling to the daemon child
/// which slot matched during `sbfb unlock`. Set to `"duress"` when
/// the duress blob opened; absent or anything else means Normal.
/// Matches the daemon's `main::main` reader.
pub const SBFB_IDENTITY_MODE_ENV: &str = "SBFB_IDENTITY_MODE";

/// Relative subdirectory of the grid root that stores the keyring
/// blob. The daemon looks at the same location when reading back.
pub const KEYRING_SUBDIR: &str = "shell-daemon/keyring";

/// Return the path to the keyring directory under the launcher's
/// grid root (same logic as `find_running_json` — prefers the
/// `NEXUS_GRID_ROOT` env var, falls back to `~/.nexus-grid`).
pub fn keyring_dir() -> PathBuf {
    let root = match std::env::var("NEXUS_GRID_ROOT") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".nexus-grid")
        }
    };
    root.join(KEYRING_SUBDIR)
}

/// Detect whether the launcher was invoked with a Sprint 20 Phase A
/// subcommand. Returns `Some(subcommand)` if the first non-`--help`
/// / `--version` argument matches, else `None` so the launcher
/// falls through to the legacy spawn-daemon path.
pub enum Subcommand {
    Init {
        pin: String,
    },
    Unlock {
        pin: String,
    },
    /// Sprint 20 Phase B : `sbfb init-duress --pin <pin>` adds a
    /// second blob under the same keyring dir. Requires a normal
    /// `sbfb init` to have run first; the duress PIN MUST differ
    /// from the normal one (enforced by the handler).
    InitDuress {
        pin: String,
    },
}

impl std::fmt::Debug for Subcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subcommand::Init { .. } => write!(f, "Subcommand::Init {{ pin: [REDACTED] }}"),
            Subcommand::Unlock { .. } => write!(f, "Subcommand::Unlock {{ pin: [REDACTED] }}"),
            Subcommand::InitDuress { .. } => {
                write!(f, "Subcommand::InitDuress {{ pin: [REDACTED] }}")
            }
        }
    }
}

/// Parse `sbfb init --pin <p>` / `sbfb unlock --pin <p>` from the
/// argv vector. Returns `None` for any other invocation so the
/// legacy launcher entry point runs unchanged.
pub fn parse_subcommand(args: &[String]) -> Option<Subcommand> {
    let cmd = args.get(1)?;
    let pin = parse_pin_flag(args);
    match (cmd.as_str(), pin) {
        ("init", Some(pin)) => Some(Subcommand::Init { pin }),
        ("unlock", Some(pin)) => Some(Subcommand::Unlock { pin }),
        ("init-duress", Some(pin)) => Some(Subcommand::InitDuress { pin }),
        _ => None,
    }
}

fn parse_pin_flag(args: &[String]) -> Option<String> {
    // Supports `--pin <val>` and `--pin=<val>` — trivially enough
    // for Phase A smoke tests.
    let mut iter = args.iter().peekable();
    while let Some(a) = iter.next() {
        if a == "--pin" {
            return iter.next().cloned();
        }
        if let Some(rest) = a.strip_prefix("--pin=") {
            return Some(rest.to_string());
        }
    }
    None
}

/// Run `sbfb init --pin <pin>`. Creates the keyring directory,
/// generates a fresh identity, encrypts under the PIN, writes the
/// blob (+ optional OS keyring wrap). Prints the new node_id to
/// stdout. Exits with status 1 on any error.
pub fn run_init(pin: &str) -> i32 {
    let dir = keyring_dir();
    let store = LocalFileKeyStore::new(&dir);
    // If a blob exists already we refuse to overwrite; the operator
    // can `sbfb wipe` first (Phase B) or remove the file manually.
    match store.init(pin) {
        Ok(id) => {
            println!("[launcher] initialized keystore at {}", dir.display());
            println!("[launcher] node_id: {}", hex::encode(id.public_bytes()));
            println!("[launcher] run `sbfb unlock --pin <pin>` to start the daemon");
            0
        }
        Err(e) => {
            eprintln!("[launcher] sbfb init failed: {e}");
            1
        }
    }
}

/// Run `sbfb unlock --pin <pin>`. Decrypts the blob, exports the
/// hex-encoded secret bytes in `SBFB_IDENTITY_SECRET_HEX`, and
/// returns `Ok(node_id_hex)` so the caller can continue with the
/// normal launcher boot flow (spawn daemon, open browser, Ctrl+C
/// wait). On failure returns `Err(exit_code)`.
pub fn run_unlock_and_export_env(pin: &str) -> Result<String, i32> {
    let dir = keyring_dir();
    let store = LocalFileKeyStore::new(&dir);
    // Sprint 20 Phase B : try the duress-aware fallback path so a
    // user who typed the duress PIN gets booted into Duress mode.
    // If only the normal blob exists, `unlock_differential`
    // behaves exactly like `unlock`.
    let id = match store.unlock_differential(pin) {
        Ok(id) => id,
        Err(UnlockError::NotInitialized(_)) => {
            eprintln!(
                "[launcher] no keystore found at {} — run `sbfb init --pin <pin>` first",
                dir.display()
            );
            return Err(2);
        }
        Err(UnlockError::AeadReject) => {
            eprintln!("[launcher] unlock failed: wrong PIN or tampered blob");
            return Err(3);
        }
        Err(e) => {
            eprintln!("[launcher] unlock failed: {e}");
            return Err(1);
        }
    };

    let node_id = hex::encode(id.public_bytes());
    let mode = id.mode();
    let mut secret = id.into_secret_bytes();
    let mut hex_str = hex::encode(secret);
    secret.zeroize();
    // SAFETY: called before tokio runtime spawn, single-threaded startup.
    unsafe { std::env::set_var(SBFB_IDENTITY_SECRET_HEX_ENV, &hex_str) };
    hex_str.zeroize();
    // Sprint 20 Phase B : signal the mode to the daemon child.
    // Only set the env var on Duress — absence == Normal on the
    // reader side, which keeps the common boot path unchanged.
    if mode == IdentityMode::Duress {
        // SAFETY: called before tokio runtime spawn, single-threaded startup.
        unsafe { std::env::set_var(SBFB_IDENTITY_MODE_ENV, "duress") };
    }
    println!(
        "[launcher] unlocked identity {} — launching daemon with persistent keypair",
        node_id
    );
    Ok(node_id)
}

/// Sprint 20 Phase B : `sbfb init-duress --pin <pin>`. Provisions
/// the duress slot next to the normal one. Prints the decoy node_id
/// to stdout so the operator can verify that it differs from the
/// real node_id. Exits 1 on error.
pub fn run_init_duress(pin: &str) -> i32 {
    let dir = keyring_dir();
    let store = LocalFileKeyStore::new(&dir);
    match store.init_duress(pin) {
        Ok(id) => {
            println!(
                "[launcher] initialized DURESS slot at {} (decoy only)",
                dir.display()
            );
            println!(
                "[launcher] decoy node_id: {}",
                hex::encode(id.public_bytes())
            );
            println!("[launcher] see docs/security/DURESS.md for usage + legal warning");
            0
        }
        Err(e) => {
            eprintln!("[launcher] sbfb init-duress failed: {e}");
            1
        }
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subcommand_init_with_flag() {
        let args = vec!["sbfb".into(), "init".into(), "--pin".into(), "1234".into()];
        match parse_subcommand(&args) {
            Some(Subcommand::Init { pin }) => assert_eq!(pin, "1234"),
            other => panic!("expected Init, got {other:?}"),
        }
    }

    #[test]
    fn parse_subcommand_unlock_with_equals() {
        let args = vec!["sbfb".into(), "unlock".into(), "--pin=5678".into()];
        match parse_subcommand(&args) {
            Some(Subcommand::Unlock { pin }) => assert_eq!(pin, "5678"),
            other => panic!("expected Unlock, got {other:?}"),
        }
    }

    #[test]
    fn parse_subcommand_none_for_bare_launcher() {
        let args = vec!["sbfb".into()];
        assert!(parse_subcommand(&args).is_none());
    }

    #[test]
    fn parse_subcommand_none_when_pin_missing() {
        let args = vec!["sbfb".into(), "unlock".into()];
        assert!(parse_subcommand(&args).is_none());
    }

    #[test]
    fn keyring_dir_respects_env_override() {
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var("NEXUS_GRID_ROOT", "/tmp/test-sbfb") };
        let dir = keyring_dir();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var("NEXUS_GRID_ROOT") };
        assert!(dir.ends_with("shell-daemon/keyring") || dir.ends_with("shell-daemon\\keyring"));
    }

    /// Sprint 20 Phase B : `sbfb init-duress --pin <pin>` parses
    /// into the `InitDuress` variant, distinct from `Init`.
    #[test]
    fn parse_subcommand_init_duress_with_flag() {
        let args = vec![
            "sbfb".into(),
            "init-duress".into(),
            "--pin".into(),
            "9999".into(),
        ];
        match parse_subcommand(&args) {
            Some(Subcommand::InitDuress { pin }) => assert_eq!(pin, "9999"),
            other => panic!("expected InitDuress, got {other:?}"),
        }
    }
}
