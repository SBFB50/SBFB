// SPDX-License-Identifier: AGPL-3.0-or-later
//! Daemon-side token-overlap validation tests — Sprint 18 Phase D.
//!
//! These tests live in the daemon crate rather than in
//! `nexus-shell-daemon-core` so a regression in the wiring
//! between the rotator primitive and the loopback validator
//! fails on its way into the HTTP router, not one layer
//! deeper. They do not boot the real daemon binary — that would
//! require the launcher-side rotation loop which is covered by
//! `nexus-launcher::token_rotation::tests`. Instead they exercise
//! [`validate_token_with_rotator`] directly, which is the exact
//! predicate the daemon middleware calls on every request once
//! the rotator is wired in (carry-over Phase F / Sprint 19).

use nexus_shell_daemon_core::auth::{validate_token_with_rotator, TokenRotator};

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
    use nexus_shell_daemon_core::auth::{TokensFile, TOKEN_OVERLAP_DURATION};
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
