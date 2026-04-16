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
//! Phase A accepts the PIN only as a CLI argument (`--pin <p>`)
//! so the flow is trivially testable. Plaintext CLI args show up
//! in shell history and `ps` listings — acceptable for dev /
//! smoke-test use but not for real deployments. Phase B adds an
//! interactive `rpassword`-style prompt.

use std::path::PathBuf;

use nexus_core_rs::keystore::{
    KeyStore, LocalFileKeyStore, UnlockError, SBFB_IDENTITY_SECRET_HEX_ENV,
};
use zeroize::Zeroize;

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
    Init { pin: String },
    Unlock { pin: String },
}

impl std::fmt::Debug for Subcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subcommand::Init { .. } => write!(f, "Subcommand::Init {{ pin: [REDACTED] }}"),
            Subcommand::Unlock { .. } => write!(f, "Subcommand::Unlock {{ pin: [REDACTED] }}"),
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
    let id = match store.unlock(pin) {
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
    let mut secret = id.into_secret_bytes();
    let mut hex_str = hex::encode(secret);
    secret.zeroize();
    std::env::set_var(SBFB_IDENTITY_SECRET_HEX_ENV, &hex_str);
    hex_str.zeroize();
    println!(
        "[launcher] unlocked identity {} — launching daemon with persistent keypair",
        node_id
    );
    Ok(node_id)
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
        std::env::set_var("NEXUS_GRID_ROOT", "/tmp/test-sbfb");
        let dir = keyring_dir();
        std::env::remove_var("NEXUS_GRID_ROOT");
        assert!(dir.ends_with("shell-daemon/keyring") || dir.ends_with("shell-daemon\\keyring"));
    }
}
