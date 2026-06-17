// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 76 Phase F (D5) — quantization documentation guards.
//!
//! Phase F is **doc-only**: the 4-bit quantization runtime already
//! exists (`LlamaCppBackend::ensure_model` loads any pre-quantized
//! GGUF via `load_from_file` + `with_n_gpu_layers`). These tests
//! lock the operator documentation in place and the anti-scope-creep
//! invariant that the backend stays single-GPU (no mono-machine
//! tensor-split — that is Sprint 77 cross-machine sharding).
//!
//! They read repository files as plain text, so they are independent
//! of the `llm_llama_cpp` Cargo feature and run in the default suite.

use std::fs;
use std::path::PathBuf;

/// `docs/operators/QUANTIZATION.md` at the workspace root, reached from
/// this crate's manifest dir (`crates/nexus-worker-core` →
/// `crates` → workspace root).
fn quantization_doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root is two levels above the crate manifest")
        .join("docs/operators/QUANTIZATION.md")
}

fn read_quantization_doc() -> String {
    let path = quantization_doc_path();
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The `llama_cpp.rs` backend source, read as text (no feature needed).
fn read_llama_cpp_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/llm/llama_cpp.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Test 1 — the operator doc exists and is non-trivial.
#[test]
fn quantization_doc_present() {
    let path = quantization_doc_path();
    assert!(path.is_file(), "missing operator doc: {}", path.display());
    let doc = read_quantization_doc();
    assert!(
        doc.len() > 1024,
        "QUANTIZATION.md is suspiciously short ({} bytes)",
        doc.len()
    );
}

/// Test 2 — the doc carries the VRAM footprint table (the three GGUF
/// formats) and names the honest single-GPU ≤14B target.
#[test]
fn quantization_doc_has_footprint_table() {
    let doc = read_quantization_doc();
    for token in ["Q4_K_M", "IQ4_XS", "Q2_K", "14B"] {
        assert!(
            doc.contains(token),
            "QUANTIZATION.md footprint table missing `{token}`"
        );
    }
}

/// Test 3 — the doc states large models (70B) belong to cross-machine
/// sharding in S77, not mono-machine multi-GPU.
#[test]
fn quantization_doc_states_70b_is_s77() {
    let doc = read_quantization_doc();
    for token in ["70B", "S77", "sharding"] {
        assert!(
            doc.contains(token),
            "QUANTIZATION.md must tie 70B to S77 cross-machine sharding (missing `{token}`)"
        );
    }
}

/// Test 4 — the doc spells out the redundancy>1 quorum pre-condition:
/// the same GGUF is required for the exact-match quorum to form.
#[test]
fn quantization_doc_states_quorum_precondition() {
    let doc = read_quantization_doc();
    for token in ["quorum", "GGUF", "exact-match"] {
        assert!(
            doc.contains(token),
            "QUANTIZATION.md must state the same-GGUF quorum pre-condition (missing `{token}`)"
        );
    }
}

/// Test 5 — anti-scope-creep invariant: the in-process backend wires
/// ONLY `with_n_gpu_layers` (single-GPU offload). The mono-machine
/// tensor-split API (`with_split_mode` / `with_devices`) is rejected
/// for S76 and reserved for S77 cross-machine sharding — its presence
/// here would mean Phase F leaked runtime code.
#[test]
fn llama_cpp_unchanged_doc_only() {
    let src = read_llama_cpp_source();
    assert!(
        src.contains("with_n_gpu_layers"),
        "llama_cpp.rs should still wire with_n_gpu_layers (single-GPU offload)"
    );
    for forbidden in ["with_split_mode", "with_devices"] {
        assert!(
            !src.contains(forbidden),
            "llama_cpp.rs must NOT wire `{forbidden}` (mono-machine tensor-split is S77, not S76 doc-only)"
        );
    }
}
