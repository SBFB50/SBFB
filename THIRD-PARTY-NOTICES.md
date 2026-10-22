# Third-party notices

SBFB / nexus-grid is licensed under AGPL-3.0-or-later. It vendors and forks the
third-party components below. Their original licenses are preserved and apply to
the vendored code; the SBFB modifications are released under AGPL-3.0-or-later as
part of this source-verifiable repository.

## llama.cpp (vendored + forked) — Sprint 77 Phase F1

- **What**: the `ggml` / `llama.cpp` inference runtime, bundled inside the
  `llama-cpp-sys-2` crate, vendored at `vendor/llama-cpp-sys-2/llama.cpp/`.
- **Upstream**: <https://github.com/ggml-org/llama.cpp>, as snapshotted by
  `utilityai/llama-cpp-rs` commit `4afdaf0782ef7f3254a186a7ff67a1c7491c6dce`
  (see `vendor/llama-cpp-sys-2/.cargo_vcs_info.json`).
- **License**: MIT (`vendor/llama-cpp-sys-2/llama.cpp/LICENSE`,
  Copyright (c) 2023-2026 The ggml authors). MIT is permissive and compatible
  with this repository's AGPL-3.0-or-later licensing.
- **SBFB modifications**: a minimal, backend-agnostic pipeline layer-split patch
  (execute a contiguous window of transformer layers, inject/extract the boundary
  residual hidden state, and partial-load only that window's weights via
  `TENSOR_SKIP`). The authoritative record of the delta is
  `patches/llama-cpp-shard.patch`; it also lives directly in the vendored tree,
  with `SBFB S77 fork` comments marking the changed regions.

## llama-cpp-2 / llama-cpp-sys-2 (vendored + forked) — Sprint 77 Phase F1

- **What**: the safe Rust wrapper (`llama-cpp-2`) and `-sys` bindings
  (`llama-cpp-sys-2`) for llama.cpp, pinned to `0.1.146`, vendored at
  `vendor/llama-cpp-2/` and `vendor/llama-cpp-sys-2/`.
- **Upstream**: <https://github.com/utilityai/llama-cpp-rs>.
- **License**: MIT OR Apache-2.0 (see each crate's `Cargo.toml`). Compatible
  with AGPL-3.0-or-later.
- **SBFB modifications**: `with_shard_range` setters on `LlamaModelParams` and
  `LlamaContextParams`, and a raw-embeddings batch path (`new_embeddings` /
  `add_embedding`) on `LlamaBatch`, to drive the forked partial-decode API.

These crates are consumed only when an operator builds nexus-worker with one of
the `llm_llama_cpp*` Cargo features; the default build and CI never compile them.
The override is wired via `[patch.crates-io]` in the root `Cargo.toml`.

## iroh-gossip (vendored + patched) — chat plan Phase I, 2026-10-22

- **What**: the `iroh-gossip` crate, pinned `0.101.0`, vendored at
  `vendor/iroh-gossip/` (the published crate, unchanged except below) and
  substituted through `[patch.crates-io]` in the workspace `Cargo.toml`.
- **Upstream**: <https://github.com/n0-computer/iroh-gossip>.
- **License**: MIT OR Apache-2.0 (`vendor/iroh-gossip/LICENSE-MIT`,
  `vendor/iroh-gossip/LICENSE-APACHE`). Compatible with this repository's
  AGPL-3.0-or-later licensing.
- **SBFB modifications**: `net::Builder::addr_filter(AddrFilter)`, applied to
  the node's own address before it is encoded as gossip peer data, so a node
  configured as discreet does not hand its public IP to topic members it never
  connected to. Re-appliable as `patches/iroh-gossip-addr-filter.patch`;
  changed regions are marked `SBFB patch (addr filter)`. Default behaviour
  (no filter set) is byte-identical to upstream.
