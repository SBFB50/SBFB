// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 79 Phase A — hermetic provenance check for the promoted anime.js
//! knowledge pack. The corpus under `docs/factory/knowledge/animejs/` is a
//! repo-visible PROCESS asset, content-addressed by the git commit. Its
//! `MANIFEST.json` self-records a per-layer blake3 16-hex digest; this test
//! recomputes those digests and asserts equality, so a silent byte drift in
//! any hashed layer fails the build.
//!
//! NB (Phase A preflight, PLAN-ADAPT): this deliberately does NOT call
//! `provenance::compute_output_hash` / `Provenance::generate` / FG8 — those
//! hash an APP WORKSPACE at publish, never `docs/`. The kickoff's
//! "hashed for free by provenance + FG8" claim was false; the manifest is
//! verified by this standalone per-layer recompute instead.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// `docs/factory/knowledge/animejs/` resolved from the crate dir
/// (`<workspace>/crates/sbfb-factory` -> `../../docs/...`).
fn animejs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/factory/knowledge/animejs")
}

#[test]
fn animejs_manifest_hashes_match_promoted_layers() {
    let dir = animejs_dir();

    let manifest_bytes =
        std::fs::read(dir.join("MANIFEST.json")).expect("read animejs MANIFEST.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).expect("parse MANIFEST.json");

    // Expected per-layer digests recorded in the manifest.
    let expected: BTreeMap<String, String> = manifest["hashes"]
        .as_object()
        .expect("MANIFEST.hashes object")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().expect("hash is a string").to_string()))
        .collect();

    // Recompute blake3 16-hex over every promoted layer file actually present
    // (MANIFEST.json + dotfiles like .gitattributes excluded).
    let mut computed: BTreeMap<String, String> = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).expect("read animejs dir") {
        let entry = entry.expect("dir entry");
        if !entry.file_type().expect("file type").is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "MANIFEST.json" || name.starts_with('.') {
            continue;
        }
        let bytes = std::fs::read(entry.path()).expect("read layer file");
        // The MANIFEST digests are computed over LF bytes; `.gitattributes` pins
        // `eol=lf` on this corpus so the working tree equals the committed blob on
        // every platform. Guard that invariant explicitly: a stray CR means a CRLF
        // drift that would make the recompute non-portable (green here, red on a
        // clean checkout / CI). Fail with a clear diagnostic, not a hash mismatch.
        assert!(
            !bytes.contains(&b'\r'),
            "{name} contains CR bytes (CRLF): the hashed corpus must be LF-only \
             (.gitattributes pins eol=lf) so blake3 stays portable across checkouts"
        );
        let hex = blake3::hash(&bytes).to_hex();
        computed.insert(name, hex[..16].to_string());
    }

    // Coverage: the manifest must hash exactly the promoted layer files
    // present (no missing layer, no stale extra) — keeps the check non-vacuous.
    let computed_keys: Vec<&String> = computed.keys().collect();
    let expected_keys: Vec<&String> = expected.keys().collect();
    assert_eq!(
        computed_keys,
        expected_keys,
        "MANIFEST.hashes must cover exactly the promoted layer files present in {}",
        dir.display()
    );

    // Each recorded digest must match the recomputed bytes.
    assert_eq!(
        computed, expected,
        "per-layer blake3[..16] recompute must equal MANIFEST.hashes (left = computed actual, right = MANIFEST)"
    );

    // Manifest sanity: pinned version + a freshness field present.
    assert_eq!(
        manifest["versions"]["animejs"].as_str(),
        Some("4.5.0"),
        "MANIFEST must pin animejs 4.5.0"
    );
    assert!(
        manifest.get("freshness").is_some(),
        "MANIFEST must record a freshness field (snapshot date + manual refresh policy)"
    );
}
