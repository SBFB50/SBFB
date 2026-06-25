// Runnable example — the blob-serve CSP contract every authored SBFB app must
// satisfy. This is the canonical, source-anchored example for the Factory
// agent-consumable authoring layer (Sprint 79 Phase I).
//
// It is lifted from the single source of truth `nexus_core_rs::csp`
// (`BLOB_SERVE_CSP`, `none_directives`, `CSS_URL_ALLOW`) — the exact constant the
// daemon injects on every blob-serve response AND the Factory authoring CSP gate
// (`sbfb_factory::gates::run_gate_csp_authoring`) imports. The example does NOT
// re-implement the gate (the gate is a binary-private static scanner inside the
// `sbfb-factory` binary crate); it proves the CONTRACT the gate is built on: the
// set of exfiltration-critical `'none'` directives an authored app must never
// violate, and the only absolute-URL exceptions a scanned asset may carry.
//
// Regular `//` comments (not `//!`) on purpose: this file is `include!`d into
// `crates/nexus-core-rs/tests/factory_csp_contract.rs`, so an inner doc comment
// would land mid-file and fail to compile.
//
// It is COMPILED AND EXECUTED by that integration test (which `include!`s this
// file), so the workspace runner `cargo nextest` runs it. A value drift in the
// CSP contract therefore fails this test, an API drift fails the build — either
// way the documented authoring rules can never silently rot.
//
// Authority: this file is rank-1 (a repo file). It proves the CSP-CONTRACT
// primitive that authoring rests on. It does NOT prove a generated app is safe at
// runtime — static conformance of *delivered* assets is FG-CSP-authoring
// (publish-time), and the runtime net is the self-check viewer (Sprint 79 Phase
// H). Caveat cardinal: lint statique != garantie runtime ; the knowledge is
// consumed, never authoritative (0 verdict PASS). See `docs/factory/WIRING_SPEC.md`.

use nexus_core_rs::{BLOB_SERVE_CSP, CSS_URL_ALLOW, none_directives};

// ── The exfiltration-critical directives an authored app must never violate ──
// Each `'none'` directive in BLOB_SERVE_CSP maps to one hard authoring rule. This
// map is the human-readable twin of the policy; keeping it in lock-step with
// `none_directives(BLOB_SERVE_CSP)` is the anti-drift contract (third test).
const AUTHORING_RULES: &[(&str, &str)] = &[
    (
        "connect-src",
        "no fetch / XHR / WebSocket / EventSource / sendBeacon — seed your RNG, ship every asset locally",
    ),
    (
        "worker-src",
        "no Worker / SharedWorker / importScripts / ServiceWorker registration",
    ),
    (
        "frame-src",
        "no nested iframe (the /auth/token exfiltration vector)",
    ),
    ("object-src", "no <object> / <embed>"),
    ("base-uri", "no <base href> hijack of relative URLs"),
    (
        "form-action",
        "no <form action> exfiltration (blocked even where sandbox allow-forms is set)",
    ),
];

// What it demonstrates, for an authoring agent (`use nexus_core_rs::…`):
//   1. The six `'none'` directives that ARE the authoring contract the Factory
//      CSP gate enforces.
//   2. The only absolute URLs a scanned asset may carry (never fetched).
//   3. That the human-readable rule map stays in lock-step with the policy.

#[test]
fn blob_serve_csp_blocks_the_six_exfil_directives() {
    let got = none_directives(BLOB_SERVE_CSP);
    assert_eq!(
        got,
        vec![
            "connect-src",
            "worker-src",
            "frame-src",
            "object-src",
            "base-uri",
            "form-action",
        ],
        "the set of 'none' directives is the authoring contract the Factory CSP gate enforces"
    );
}

#[test]
fn css_url_allow_is_the_only_absolute_url_exception() {
    // A scanned/compiled asset may carry these absolute URLs even though
    // `default-src 'self'` forbids remote network: they are never fetched (XML
    // namespace identifiers inside inline SVG / data: URIs, and the MIT license
    // banner that compiled Tailwind/daisyUI CSS must preserve). Any OTHER absolute
    // http(s) URL in a scanned asset trips FG-CSP-authoring.
    assert_eq!(
        CSS_URL_ALLOW.to_vec(),
        vec![
            "http://www.w3.org/2000/svg",
            "http://www.w3.org/1999/xlink",
            "https://tailwindcss.com",
        ],
    );
}

#[test]
fn authoring_rules_stay_in_lockstep_with_the_policy() {
    // Anti-drift: if BLOB_SERVE_CSP gains or loses a `'none'` directive, this
    // example's human-readable rule map must be updated in the same edit, or the
    // build goes red. The documented authoring guidance can never silently rot.
    let policy: Vec<&str> = none_directives(BLOB_SERVE_CSP);
    let documented: Vec<&str> = AUTHORING_RULES.iter().map(|(directive, _)| *directive).collect();
    assert_eq!(
        documented, policy,
        "AUTHORING_RULES must document exactly the policy's 'none' directives (left = doc, right = policy)"
    );
}
