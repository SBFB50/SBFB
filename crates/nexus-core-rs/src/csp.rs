// SPDX-License-Identifier: AGPL-3.0-or-later
//! Blob-serve Content-Security-Policy contract — the single source of truth.
//!
//! The CSP / COOP / COEP headers the daemon injects on every blob-serve
//! response live here so that **two** consumers share one definition with
//! zero drift:
//!
//! 1. `nexus-shell-daemon` injects [`BLOB_SERVE_CSP`] at runtime
//!    (`http.rs` → `blob_serve::BLOB_SERVE_CSP`, re-exported from this
//!    module).
//! 2. The Factory authoring CSP gate (`sbfb-factory::gates::run_gate_csp_authoring`,
//!    Sprint 79 Phase E) imports the same const and asserts — via a
//!    cross-crate test — that its static asset scanner enforces every
//!    directive this policy sets to `'none'`. See [`none_directives`].
//!
//! This crate is the natural host: it already owns the cross-crate contract
//! constants (`canonical::DOMAIN_*_V1`) and is a common dependency of both
//! `nexus-shell-daemon-core` and `sbfb-factory`, so neither side re-hardcodes
//! the policy. The machine-readable mirror `csp-contract.json` (verified
//! against these consts by [`tests`]) lets the JS lint
//! (`examples/daisyui-animejs-showcase/scripts/check-csp.mjs`) consume the
//! same contract without re-deriving it.

/// The `Content-Security-Policy` header value injected on every blob-serve
/// response. Defense-in-depth for both iframe and direct URL navigation:
/// `sandbox allow-scripts` gives an opaque origin even in a top-level tab
/// (blocks localStorage, cookies, SW scope on the daemon origin);
/// `worker-src 'none'` prevents Service Worker registration; `frame-src
/// 'none'` blocks nested iframes (the `/auth/token` exfiltration vector);
/// `form-action 'none'` blocks form-based exfiltration that `connect-src
/// 'none'` does not cover; `base-uri 'none'` blocks `<base href>` hijacking
/// of relative URLs.
pub const BLOB_SERVE_CSP: &str = "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors *; sandbox allow-scripts";

/// The `Cross-Origin-Opener-Policy` header injected alongside the CSP.
pub const BLOB_SERVE_COOP: &str = "same-origin";

/// The `Cross-Origin-Embedder-Policy` header injected alongside the CSP.
pub const BLOB_SERVE_COEP: &str = "require-corp";

/// Absolute URLs allowed to appear in scanned runtime assets even though the
/// CSP forbids remote network access. These are **never fetched** by the
/// browser — they are XML namespace identifiers that live inside `data:`
/// URIs / inline SVG, and the MIT license banner that compiled Tailwind/daisyUI
/// CSS must preserve. The Factory CSP gate and the JS lint share this list so
/// a legitimate inline `<svg xmlns="http://www.w3.org/2000/svg">` or license
/// banner does not trip the gate.
pub const CSS_URL_ALLOW: &[&str] = &[
    "http://www.w3.org/2000/svg",   // SVG xmlns (inline SVG / data: URIs)
    "http://www.w3.org/1999/xlink", // xlink xmlns
    "https://tailwindcss.com",      // MIT license banner (must be preserved)
];

/// Parse a CSP string and return the directive names whose value is exactly
/// `'none'`. Deterministic, allocation-light: splits on `;`, trims, and keeps
/// the directive name when its single value token is `'none'`.
///
/// For [`BLOB_SERVE_CSP`] this returns
/// `["connect-src", "worker-src", "frame-src", "object-src", "base-uri", "form-action"]`.
/// The Factory authoring gate uses it as the anti-drift contract: if a future
/// edit adds a new `'none'` directive to the policy, the gate's coverage test
/// fails until a matching detection rule is added.
pub fn none_directives(csp: &str) -> Vec<&str> {
    csp.split(';')
        .filter_map(|segment| {
            let segment = segment.trim();
            let (name, value) = segment.split_once(char::is_whitespace)?;
            if value.trim() == "'none'" {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_directives_extracts_the_six_blocked_directives() {
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
            "the set of 'none' directives drives the authoring gate's required coverage"
        );
    }

    #[test]
    fn none_directives_ignores_non_none_values() {
        // default-src has an allowlist value, frame-ancestors is `*`, sandbox
        // is a token list — none of these are `'none'`.
        let got = none_directives(BLOB_SERVE_CSP);
        assert!(!got.contains(&"default-src"));
        assert!(!got.contains(&"frame-ancestors"));
        assert!(!got.contains(&"sandbox"));
    }

    #[test]
    fn none_directives_handles_empty_and_trailing_separators() {
        assert!(none_directives("").is_empty());
        assert_eq!(none_directives("connect-src 'none';"), vec!["connect-src"]);
        assert_eq!(
            none_directives("  connect-src   'none'  ;  worker-src 'none'"),
            vec!["connect-src", "worker-src"]
        );
    }

    /// The committed `csp-contract.json` is a machine-readable mirror of the
    /// Rust consts, consumed by the JS lint. This test makes it a
    /// verified-by-recompute derivative (anti-drift), mirroring the
    /// `animejs_manifest.rs` hash pattern: edit the const and the JSON must be
    /// regenerated or this test fails.
    #[test]
    fn csp_contract_json_mirrors_the_rust_consts() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/csp-contract.json");
        let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(
            json["csp"].as_str().unwrap(),
            BLOB_SERVE_CSP,
            "csp-contract.json `csp` drifted from BLOB_SERVE_CSP"
        );

        let json_dirs: Vec<&str> = json["none_directives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            json_dirs,
            none_directives(BLOB_SERVE_CSP),
            "csp-contract.json `none_directives` drifted from the policy"
        );

        let json_allow: Vec<&str> = json["css_url_allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            json_allow,
            CSS_URL_ALLOW.to_vec(),
            "csp-contract.json `css_url_allow` drifted from CSS_URL_ALLOW"
        );
    }
}
