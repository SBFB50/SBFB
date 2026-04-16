// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase A — integration tests for the encryption-at-rest
//! keystore. Covers cross-boundary flows (fs + OS keyring + notify
//! watcher + concurrent unlocks) that do not belong in the
//! in-module primitive suite.
//!
//! Tests that require a live OS credential store (macOS Keychain,
//! Windows Credential Manager, Linux Secret Service) are gated on
//! `keyring::Entry::new` returning `Ok` at the start of each test.
//! Headless CI runners without a Secret Service daemon simply skip
//! the affected cases with a tracing warn — the primitive suite
//! already covers the Argon2id + AEAD path without the keyring
//! layer.

use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use nexus_core_rs::keystore::{
    KdfParams, KeyStore, LocalFileKeyStore, UnlockError, BLOB_FILE_NAME,
};
use tempfile::TempDir;

/// Return a unique keyring service string so two parallel tests do
/// not clobber the same slot in the OS store. The tempdir absolute
/// path is already unique per test.
fn unique_slot(dir: &TempDir, tag: &str) -> (String, String) {
    let hash = blake3::hash(dir.path().to_string_lossy().as_bytes());
    let short = &hex::encode(hash.as_bytes())[..16];
    (
        format!("sbfb-daemon-test-{tag}-{short}"),
        "integration".to_string(),
    )
}

/// Check whether the OS keyring is usable on this host AND that a
/// cross-handle round-trip (set via one `Entry`, read via a freshly
/// constructed `Entry`) works reliably. Returns `false` on:
/// - Headless Linux CI runners without a Secret Service daemon.
/// - Windows Credential Manager instances where the backend does
///   not persist credentials across `Entry` handles under the
///   current process user context (observed with
///   `keyring = 3.6.3` under certain Windows user profile
///   configurations). The `#[test] init_creates_blob_and_keyring_
///   entry` case skips gracefully rather than spuriously failing.
fn os_keyring_available() -> bool {
    let Ok(entry1) = keyring::Entry::new("sbfb-daemon-probe", "availability") else {
        return false;
    };
    let _ = entry1.delete_credential();
    if entry1.set_secret(b"probe").is_err() {
        return false;
    }
    // Cross-handle read: construct a brand new Entry and attempt a
    // get_secret. If the backend binds credentials to the handle
    // that created them (rare but observed on some Windows configs
    // with keyring-rs 3.6), the round-trip fails and we skip the
    // keyring-bound tests.
    let cross_ok = match keyring::Entry::new("sbfb-daemon-probe", "availability") {
        Ok(entry2) => entry2.get_secret().is_ok(),
        Err(_) => false,
    };
    let _ = entry1.delete_credential();
    cross_ok
}

/// #15 post-init the blob file exists AND the keyring entry can be
/// used to unlock (verifying the keyring layer round-trip end-to-
/// end instead of reading the entry directly — direct reads via a
/// fresh `keyring::Entry` handle are flaky on some Windows
/// Credential Manager backends that normalise the target-name
/// mapping between `new()` and `get_secret()`).
#[test]
fn init_creates_blob_and_keyring_entry() {
    if !os_keyring_available() {
        eprintln!("[skip] OS keyring unavailable on this host");
        return;
    }
    let dir = TempDir::new().unwrap();
    let (service, account) = unique_slot(&dir, "init-blob-kr");
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .with_keyring_slot(service.clone(), account.clone());

    let id = store.init("1234").expect("init must succeed");
    assert_eq!(id.mode(), nexus_core_rs::keystore::IdentityMode::Normal);
    let public = *id.public_bytes();

    // Blob file exists.
    let blob = dir.path().join(BLOB_FILE_NAME);
    assert!(blob.exists(), "blob file must exist after init");

    // The keyring entry is proven to exist because a fresh store
    // instance can unlock via it (the blob flags byte is set to
    // `uses_keyring=true`, so the unlock path reads kek2 from the
    // custom slot and derives the same final_kek).
    let store2 = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .with_keyring_slot(service, account);
    let id2 = store2
        .unlock("1234")
        .expect("unlock must read kek2 from keyring");
    assert_eq!(id2.public_bytes(), &public);

    // Clean up via the store's own wipe (removes the keyring entry
    // with the same service/account mapping init used, avoiding
    // the direct-Entry flakiness).
    store2.wipe().expect("wipe must succeed");
    assert!(!blob.exists(), "wipe removes the blob");
}

/// #16 init on a dir that already holds a blob refuses to clobber.
#[test]
fn init_idempotent_rejects_reinit() {
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    store.init("1234").expect("first init");
    let err = store.init("1234").expect_err("second init must reject");
    let msg = format!("{err}");
    assert!(
        msg.contains("refusing to overwrite"),
        "expected AlreadyInitialized, got: {msg}"
    );
}

/// #17 disk persistence: a new `LocalFileKeyStore` instance
/// pointing at the same dir can unlock a blob written by an earlier
/// process — simulated here by dropping the first store before
/// constructing the second.
#[test]
fn unlock_after_restart_works() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    let public_bytes = {
        let store =
            LocalFileKeyStore::with_params(&path, KdfParams::fast_for_tests()).without_os_keyring();
        let id = store.init("1234").unwrap();
        *id.public_bytes()
    };

    // New store, same path, same params.
    let store2 =
        LocalFileKeyStore::with_params(&path, KdfParams::fast_for_tests()).without_os_keyring();
    let id2 = store2
        .unlock("1234")
        .expect("unlock after 'restart' must work");
    assert_eq!(id2.public_bytes(), &public_bytes);
}

/// #18 `notify` file-watcher observes a `rotate_pin` on disk.
/// Verifies the hot-reload contract: an external rotation causes a
/// Create/Modify event on the blob path within a bounded window.
#[test]
fn hot_reload_blob_rotation_watcher() {
    use notify::{EventKind, RecursiveMode, Watcher};
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    store.init("1234").unwrap();

    let blob_path = dir.path().join(BLOB_FILE_NAME);
    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |res| tx.send(res).unwrap_or(())).unwrap();
    watcher
        .watch(dir.path(), RecursiveMode::NonRecursive)
        .unwrap();

    // Rotate the PIN — triggers write_atomic which on Windows does
    // fs::write(tmp) + fs::rename(tmp, blob_path). On Unix the
    // rename is atomic. Either way the watcher sees at least one
    // event on the blob file.
    store
        .rotate_pin("1234", "5678")
        .expect("rotate must succeed");

    let mut saw_event = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Ok(Ok(ev)) = rx.recv_timeout(Duration::from_millis(500)) {
            if ev.paths.iter().any(|p| p == &blob_path)
                && matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any
                )
            {
                saw_event = true;
                break;
            }
        }
    }
    assert!(
        saw_event,
        "notify watcher did not observe a blob event within 5 s"
    );
}

/// #19 concurrent unlocks with the same correct PIN all succeed.
#[test]
fn concurrent_unlock_same_pin_safe() {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(
        LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
            .without_os_keyring(),
    );
    store.init("1234").unwrap();

    let mut handles = Vec::new();
    for _ in 0..8 {
        let s = Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            let id = s.unlock("1234").expect("concurrent unlock");
            *id.public_bytes()
        }));
    }
    let mut public = None;
    for h in handles {
        let pb = h.join().unwrap();
        match public {
            None => public = Some(pb),
            Some(prev) => assert_eq!(
                pb, prev,
                "all concurrent unlocks must return the same identity"
            ),
        }
    }
}

/// #20 concurrent unlocks with a mix of correct and wrong PINs
/// never surface one thread's decrypted bytes to another, and the
/// wrong-PIN threads never succeed.
#[test]
fn concurrent_unlock_different_pins_safe() {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(
        LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
            .without_os_keyring(),
    );
    let id = store.init("correct").unwrap();
    let expected_public = *id.public_bytes();

    let mut handles = Vec::new();
    for i in 0..8 {
        let s = Arc::clone(&store);
        let pin = if i % 2 == 0 { "correct" } else { "wrong" };
        handles.push(std::thread::spawn(move || match s.unlock(pin) {
            Ok(id) => Ok(*id.public_bytes()),
            Err(e) => Err(format!("{e}")),
        }));
    }

    let mut correct_hits = 0;
    let mut wrong_hits = 0;
    for h in handles {
        match h.join().unwrap() {
            Ok(pb) => {
                assert_eq!(pb, expected_public, "unlock leaked a different keypair");
                correct_hits += 1;
            }
            Err(_) => wrong_hits += 1,
        }
    }
    assert_eq!(correct_hits, 4);
    assert_eq!(wrong_hits, 4);
}

/// #21 a single ciphertext bit flip is rejected with `AeadReject`.
#[test]
fn blob_corruption_fails_loud() {
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    store.init("1234").unwrap();

    // Flip a random byte in the ciphertext region.
    let blob_path = dir.path().join(BLOB_FILE_NAME);
    let mut bytes = std::fs::read(&blob_path).unwrap();
    let idx = nexus_core_rs::keystore::BLOB_HEADER_LEN + 3;
    bytes[idx] ^= 0x01;
    std::fs::write(&blob_path, &bytes).unwrap();

    let err = store.unlock("1234").unwrap_err();
    assert!(matches!(err, UnlockError::AeadReject));
}

/// #22 `without_os_keyring` mode works end-to-end with no keyring
/// touch — documented degraded mode for CI / minimal-Linux hosts
/// that do not run a Secret Service daemon.
#[test]
fn keyring_entry_missing_falls_back_to_blob_only() {
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    let id = store.init("1234").unwrap();
    let public = *id.public_bytes();

    // Re-open with a fresh store instance — still no keyring.
    let store2 = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    let id2 = store2
        .unlock("1234")
        .expect("unlock without keyring must succeed");
    assert_eq!(id2.public_bytes(), &public);

    // A store that expects the keyring layer on a blob that was
    // written without it must still unlock, because the blob
    // flags byte records the choice. The use_keyring flag on the
    // consumer store is authoritative only for init-time kek2
    // generation; unlock reads what the blob tells it.
    let store3 = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests());
    let id3 = store3
        .unlock("1234")
        .expect("unlock infers layer from blob flags");
    assert_eq!(id3.public_bytes(), &public);
}
