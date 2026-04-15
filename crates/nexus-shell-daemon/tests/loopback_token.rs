// SPDX-License-Identifier: AGPL-3.0-or-later
//! Daemon-side token-overlap validation tests — Sprint 18 Phase D
//! + Sprint 18 audit fix D-1 (`AuthState::Rotated` end-to-end).
//!
//! These tests live in the daemon crate rather than in
//! `nexus-shell-daemon-core` so a regression in the wiring
//! between the rotator primitive and the loopback validator
//! fails on its way into the HTTP router, not one layer
//! deeper.
//!
//! Phase D shipped only the primitive ([`validate_token_with_rotator`]).
//! Audit fix D-1 closes the gap by adding the [`AuthState::Rotated`]
//! variant + the [`TokenRotatorWatcher`] file-watcher ; the four
//! `auth_state_rotated_*` and `tokens_watcher_*` tests below
//! exercise the new public surface end-to-end without booting the
//! axum router (the routing layer is covered by `auth::tests` in
//! the core crate ; here we focus on the rotation behaviour the
//! middleware now sees).

use nexus_shell_daemon_core::auth::{
    validate_token_with_rotator, AuthState, TokenRotator, TokenRotatorWatcher, TokensFile,
    TOKEN_OVERLAP_DURATION,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[test]
fn accepts_current_token() {
    let r = TokenRotator::new("current-only".to_string());
    assert!(validate_token_with_rotator("current-only", &r));
    assert!(!validate_token_with_rotator("something-else", &r));
}

#[test]
fn accepts_previous_token_during_overlap() {
    let mut r = TokenRotator::new("old".to_string());
    r.rotate("new".to_string());
    // Immediately after rotation, both tokens must validate.
    assert!(validate_token_with_rotator("new", &r));
    assert!(validate_token_with_rotator("old", &r));
    // A third token is still rejected.
    assert!(!validate_token_with_rotator("stranger", &r));
}

#[test]
fn rejects_previous_token_after_overlap() {
    // Mint a rotator whose last rotation is well outside the
    // 10 min overlap window by round-tripping through the file
    // persistence layer with a stale `rotated_at` Unix time.
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("tokens.json");
    let stale_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (TOKEN_OVERLAP_DURATION.as_secs() + 120);
    let body = serde_json::to_string(&TokensFile {
        current: "new".to_string(),
        previous: Some("old".to_string()),
        rotated_at: stale_unix,
    })
    .unwrap();
    std::fs::write(&path, body).unwrap();

    let r = TokenRotator::load(&path).unwrap().unwrap();
    assert!(!r.is_in_overlap_window());
    assert!(
        validate_token_with_rotator("new", &r),
        "current token must remain valid after overlap elapses"
    );
    assert!(
        !validate_token_with_rotator("old", &r),
        "previous token must be rejected once the overlap window has elapsed"
    );
}

// =================================================================
// Sprint 18 audit fix D-1 — `AuthState::Rotated` end-to-end
// =================================================================

#[test]
fn auth_state_rotated_validates_current_and_previous_during_overlap() {
    // Same scenario as `accepts_previous_token_during_overlap` but
    // exercised through the `AuthState::Rotated` dispatch path the
    // axum middleware actually consults — proves the variant
    // forwards to `validate_token_with_rotator` correctly.
    let mut rotator = TokenRotator::new("old".to_string());
    rotator.rotate("new".to_string());
    let auth = AuthState::rotated(Arc::new(RwLock::new(rotator)));

    assert!(auth.validate("new"), "current must validate");
    assert!(
        auth.validate("old"),
        "previous within overlap must validate"
    );
    assert!(!auth.validate("foreign"), "unknown token must reject");
}

#[test]
fn auth_state_rotated_rejects_previous_after_overlap() {
    // Round-trip through `TokensFile` to mint a rotator whose
    // overlap window has already elapsed (same trick as
    // `rejects_previous_token_after_overlap` above), then dispatch
    // through `AuthState::Rotated` to confirm the middleware sees
    // the elapsed window.
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("tokens.json");
    let stale_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (TOKEN_OVERLAP_DURATION.as_secs() + 120);
    let body = serde_json::to_string(&TokensFile {
        current: "new".to_string(),
        previous: Some("old".to_string()),
        rotated_at: stale_unix,
    })
    .unwrap();
    std::fs::write(&path, body).unwrap();

    let rotator = TokenRotator::load(&path).unwrap().unwrap();
    let auth = AuthState::rotated(Arc::new(RwLock::new(rotator)));

    assert!(auth.validate("new"), "current still valid post-overlap");
    assert!(
        !auth.validate("old"),
        "previous token rejected once overlap elapsed"
    );
}

#[test]
fn auth_state_static_validates_only_exact_match() {
    // The legacy `Static` variant must keep its strict single-
    // token semantics — anything else is a regression on the
    // pre-D-1 boot path the launcher still uses when no
    // `tokens.json` exists.
    let auth = AuthState::new("deadbeef".to_string());
    assert!(auth.validate("deadbeef"));
    assert!(!auth.validate("DEADBEEF"));
    assert!(!auth.validate("deadbeef0"));
    assert!(!auth.validate(""));
}

#[test]
fn tokens_watcher_picks_up_external_rewrite() {
    // The whole point of D-1 : the launcher writes a fresh
    // `tokens.json`, the daemon's watcher thread reloads it, and
    // the new pair is visible to `AuthState::Rotated::validate`
    // without rebooting the daemon. We:
    //
    //   1. seed the file with `{current: "first"}`,
    //   2. spawn the watcher,
    //   3. rewrite the file with `{current: "second", previous: "first"}`,
    //   4. poll up to 5 s for the watcher to pick it up.
    //
    // The 5 s budget accounts for `notify`'s 50 ms debounce plus
    // CI scheduler jitter. Real rotations happen on a 24 h cadence,
    // so the latency budget is utterly non-critical — we just need
    // it to eventually reflect the new state.
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("tokens.json");

    // Step 1 — initial state: only "first" is valid.
    let initial = TokenRotator::new("first".to_string());
    initial.write_atomic(&path).unwrap();

    let initial_loaded = TokenRotator::load(&path).unwrap().unwrap();
    let watcher = TokenRotatorWatcher::spawn(path.clone(), initial_loaded)
        .expect("watcher spawns when tokens.json exists");
    let auth = AuthState::rotated(watcher.shared());

    assert!(auth.validate("first"));
    assert!(!auth.validate("second"));

    // Step 2 — rotate via a separate rotator and persist atomically,
    // mirroring exactly what the launcher's `spawn_rotation_loop`
    // does at h+24 h.
    let mut next = TokenRotator::new("first".to_string());
    next.rotate("second".to_string());
    next.write_atomic(&path).unwrap();

    // Step 3 — poll for the watcher to reload. The deadline must
    // be generous enough for `notify` on Windows (ReadDirectoryChangesW
    // can take a few hundred ms on cold inodes).
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut last_seen = String::new();
    while std::time::Instant::now() < deadline {
        if auth.validate("second") {
            last_seen = "second".into();
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        last_seen, "second",
        "watcher must propagate the rewritten current token within 5s"
    );

    // The previous slot must also have been picked up — this is
    // what protects in-flight requests across a rotation.
    assert!(
        auth.validate("first"),
        "previous slot must be readable through the watcher's snapshot"
    );

    // Sanity : a third token still rejects, ruling out an
    // accidentally-permissive watcher reload (e.g. clobbering the
    // rotator with default-true state).
    assert!(!auth.validate("third"));

    drop(watcher);
}
