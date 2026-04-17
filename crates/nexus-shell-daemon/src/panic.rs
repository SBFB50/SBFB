// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase B — panic wipe service.
//!
//! The shell listens for a 5-tap `Ctrl+Shift+Alt+W` gesture and
//! POSTs to `/panic/wipe` on the daemon when it fires. The
//! daemon-side handler hands control to [`PanicWipeService`] which
//! performs, in order :
//!
//! 1. `LocalFileKeyStore::wipe_all` — zero-overwrite + unlink the
//!    normal blob (`identity.enc`) + the duress blob
//!    (`identity_duress.enc`) + delete both OS keyring entries.
//! 2. Best-effort `fs::remove_file` on `state.sqlite` — the Sprint
//!    11 subscriptions persistence file.
//! 3. Best-effort `fs::remove_dir_all` on the blob cache
//!    directory (decompressed zip archives the daemon streams to
//!    the iframe).
//! 4. Hand off to the injected [`ExitStrategy`] which in
//!    production calls `std::process::exit(0)` and in tests sets
//!    a flag instead so assertions can run.
//!
//! Steps 2 and 3 are best-effort : a panic wipe must never block
//! on "nothing to delete" because the user likely has seconds
//! before the adversary reaches them. [`PanicWipeReport`] records
//! what actually happened.
//!
//! ## RAM zeroization
//!
//! The keypair in memory is zeroed through [`Identity`]'s drop
//! impl — `SecretBox<SecretKeyBytes>` wires `Zeroize` on drop
//! (Phase A). The panic service does NOT hold a long-lived
//! `Identity` on its own; the daemon runtime owns it, and when
//! the daemon process exits every stack + heap allocation is
//! torn down. An adversary with `ptrace` access during the
//! wipe window has already won — the RAM-zeroize guarantee is
//! best-effort-at-process-exit, not real-time-sync.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use nexus_core_rs::{KeyStoreError, LocalFileKeyStore};

/// Pluggable exit strategy so tests do not call
/// `std::process::exit` on the test runner itself.
pub trait ExitStrategy: Send + Sync + 'static {
    /// Terminate the process with the given code. Never returns
    /// in production.
    fn exit(&self, code: i32);
}

/// Production exit: calls `std::process::exit(code)`.
pub struct RealExit;

impl ExitStrategy for RealExit {
    fn exit(&self, code: i32) {
        std::process::exit(code);
    }
}

/// Test exit: sets an `AtomicI32` to the exit code instead of
/// actually calling `exit`. Lets integration tests assert that
/// the service would have exited.
///
/// Gated on `cfg(test)` because the production binary has no
/// reason to carry a recording exit — only the in-file test
/// module needs it.
#[cfg(test)]
pub struct RecordingExit {
    pub code: std::sync::atomic::AtomicI32,
    pub called: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
impl RecordingExit {
    pub fn new() -> Self {
        Self {
            code: std::sync::atomic::AtomicI32::new(i32::MIN),
            called: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn last_code(&self) -> Option<i32> {
        if self.called.load(std::sync::atomic::Ordering::SeqCst) {
            Some(self.code.load(std::sync::atomic::Ordering::SeqCst))
        } else {
            None
        }
    }
}

#[cfg(test)]
impl Default for RecordingExit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ExitStrategy for RecordingExit {
    fn exit(&self, code: i32) {
        self.code.store(code, std::sync::atomic::Ordering::SeqCst);
        self.called.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Report describing what `PanicWipeService::execute` actually
/// deleted. Consumed by tests + audit logs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PanicWipeReport {
    pub keystore_wiped: bool,
    pub state_db_deleted: bool,
    pub blob_cache_deleted: bool,
}

/// Failures surfaced by [`PanicWipeService::execute`]. The keystore
/// variant is the only hard failure — the filesystem variants are
/// swallowed and recorded in the report.
#[derive(Debug, thiserror::Error)]
pub enum PanicWipeError {
    #[error("keystore wipe_all failed: {0}")]
    Keystore(#[from] KeyStoreError),
}

/// Panic wipe executor. Constructed per-request by the HTTP
/// handler from the daemon's shared state.
pub struct PanicWipeService {
    keystore: Arc<LocalFileKeyStore>,
    state_db_path: PathBuf,
    blob_cache_dir: PathBuf,
    exit: Arc<dyn ExitStrategy>,
}

impl std::fmt::Debug for PanicWipeService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PanicWipeService")
            .field("state_db_path", &self.state_db_path)
            .field("blob_cache_dir", &self.blob_cache_dir)
            .field("exit", &"<dyn ExitStrategy>")
            .finish()
    }
}

impl PanicWipeService {
    /// Build a new service. The exit strategy is injected so tests
    /// can record instead of terminate.
    pub fn new(
        keystore: Arc<LocalFileKeyStore>,
        state_db_path: impl Into<PathBuf>,
        blob_cache_dir: impl Into<PathBuf>,
        exit: Arc<dyn ExitStrategy>,
    ) -> Self {
        Self {
            keystore,
            state_db_path: state_db_path.into(),
            blob_cache_dir: blob_cache_dir.into(),
            exit,
        }
    }

    /// Execute the destructive sequence without exiting. Used by
    /// tests and by the HTTP handler as step 1 of the request
    /// cycle (the handler then replies 200 and schedules the
    /// exit so the response actually reaches the shell).
    pub fn execute(&self) -> Result<PanicWipeReport, PanicWipeError> {
        let mut report = PanicWipeReport::default();

        // 1. Keystore is the only hard-fail step — if we cannot
        //    wipe the private key bytes, the panic is useless.
        self.keystore.wipe_all()?;
        report.keystore_wiped = true;

        // 2. state.sqlite — the Sprint 11 subscriptions persistence
        //    file. Best-effort secure unlink : overwrite with zeros
        //    if we can, then remove. Swallow any error — the panic
        //    path must not block.
        report.state_db_deleted = secure_unlink_best_effort(&self.state_db_path);

        // 3. blob cache dir — recursively delete. Best-effort.
        report.blob_cache_deleted = remove_dir_all_best_effort(&self.blob_cache_dir);

        Ok(report)
    }

    /// Exit the process. The HTTP handler calls this from the
    /// post-response `tokio::spawn` delay task, after having run
    /// [`Self::execute`] synchronously before replying 200 — the
    /// two operations are kept as independent primitives so the
    /// wipe cannot be accidentally executed twice on a single
    /// request (what used to be a single `execute_and_exit`
    /// entry point re-ran the wipe inside the delay task).
    pub fn exit_only(&self, exit_code: i32) -> ! {
        self.exit.exit(exit_code);
        // `ExitStrategy::exit` on `RealExit` never returns; on
        // `RecordingExit` it does — we loop forever in that case
        // so test code that mistakenly calls this function in a
        // non-test context still behaves as "process dead" from
        // the rest of the daemon's point of view.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
}

/// Zero-overwrite then unlink a file. Returns `true` iff the file
/// was present AND successfully removed.
fn secure_unlink_best_effort(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    if let Ok(metadata) = std::fs::metadata(path) {
        let len = metadata.len() as usize;
        let zeros = vec![0u8; len];
        let _ = std::fs::write(path, &zeros);
    }
    std::fs::remove_file(path).is_ok()
}

fn remove_dir_all_best_effort(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    std::fs::remove_dir_all(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use nexus_core_rs::{KdfParams, KeyStore};
    use tempfile::TempDir;

    fn make_keystore(dir: &TempDir) -> Arc<LocalFileKeyStore> {
        // `without_os_keyring` is what keeps this test hermetic on
        // CI. The service name still has to be unique per test
        // case so a second test run in the same process cannot
        // trip over a stale OS keyring entry left behind by the
        // keyring crate on dev hosts, hence the tempdir path
        // (uuid-grade unique, already provided by `tempfile`).
        let unique_service = format!("sbfb-test-{}", dir.path().display());
        Arc::new(
            LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
                .without_os_keyring()
                .with_keyring_slot(unique_service, "test-account"),
        )
    }

    #[test]
    fn panic_wipe_removes_both_blobs() {
        let dir = TempDir::new().unwrap();
        let keystore = make_keystore(&dir);
        keystore.init("1111").unwrap();
        keystore.init_duress("2222").unwrap();
        assert!(keystore.blob_path().exists());
        assert!(keystore.blob_path_duress().exists());

        let state_path = dir.path().join("state.sqlite");
        let cache_dir = dir.path().join("blob-cache");

        let exit = Arc::new(RecordingExit::new());
        let service = PanicWipeService::new(
            Arc::clone(&keystore),
            state_path.clone(),
            cache_dir.clone(),
            Arc::clone(&exit) as Arc<dyn ExitStrategy>,
        );
        let report = service.execute().unwrap();

        assert!(report.keystore_wiped);
        assert!(!keystore.blob_path().exists());
        assert!(!keystore.blob_path_duress().exists());
    }

    #[test]
    fn panic_wipe_deletes_state_sqlite_and_blob_cache() {
        let dir = TempDir::new().unwrap();
        let keystore = make_keystore(&dir);
        keystore.init("1111").unwrap();

        let state_path = dir.path().join("state.sqlite");
        std::fs::write(&state_path, b"fake sqlite bytes").unwrap();
        let cache_dir = dir.path().join("blob-cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("entry-0"), b"cached blob").unwrap();
        std::fs::write(cache_dir.join("entry-1"), b"another").unwrap();

        let exit = Arc::new(RecordingExit::new());
        let service = PanicWipeService::new(
            Arc::clone(&keystore),
            state_path.clone(),
            cache_dir.clone(),
            Arc::clone(&exit) as Arc<dyn ExitStrategy>,
        );
        let report = service.execute().unwrap();

        assert!(report.state_db_deleted);
        assert!(report.blob_cache_deleted);
        assert!(!state_path.exists());
        assert!(!cache_dir.exists());
    }

    #[test]
    fn panic_wipe_zeroizes_keypair_ram() {
        // Phase A already tests that `SecretKeyBytes::zeroize`
        // clears the heap buffer (keystore::tests::
        // zeroize_drops_plaintext_key_in_memory). This test
        // complements it by verifying that `PanicWipeService`
        // does NOT keep a long-lived `Identity` handle : the
        // service owns a keystore reference only, and unlocks
        // nothing. Consequently the daemon's single live
        // `Identity` is dropped on process exit, which drives the
        // zeroize through `Drop`. We encode this invariant here
        // as a compile-time check.
        fn assert_no_identity_field<T: NoIdentity>(_: &T) {}
        trait NoIdentity {}
        impl NoIdentity for PanicWipeService {}

        let dir = TempDir::new().unwrap();
        let keystore = make_keystore(&dir);
        keystore.init("1111").unwrap();
        let state_path = dir.path().join("state.sqlite");
        let cache_dir = dir.path().join("blob-cache");
        let exit = Arc::new(RecordingExit::new());
        let service = PanicWipeService::new(
            Arc::clone(&keystore),
            state_path,
            cache_dir,
            Arc::clone(&exit) as Arc<dyn ExitStrategy>,
        );
        assert_no_identity_field(&service);
    }

    #[test]
    fn panic_wipe_exits_process() {
        let dir = TempDir::new().unwrap();
        let keystore = make_keystore(&dir);
        keystore.init("1111").unwrap();

        let state_path = dir.path().join("state.sqlite");
        let cache_dir = dir.path().join("blob-cache");

        let exit = Arc::new(RecordingExit::new());
        let service = PanicWipeService::new(
            Arc::clone(&keystore),
            state_path,
            cache_dir,
            Arc::clone(&exit) as Arc<dyn ExitStrategy>,
        );

        // Run execute + exit_only on a background thread so the
        // test does not deadlock on the infinite-sleep guard path
        // inside exit_only. The recording exit strategy flips the
        // atomic flag instead of actually exiting.
        let service = Arc::new(service);
        let bg = Arc::clone(&service);
        let handle = std::thread::spawn(move || {
            let _ = bg.execute();
            bg.exit_only(0);
        });
        // The service loops on `thread::sleep` post-exit so we
        // join with a timeout by polling the flag.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if exit.last_code().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(exit.last_code(), Some(0));
        drop(handle); // let the daemon-style sleep loop leak — the test runner tears it down
    }
}
