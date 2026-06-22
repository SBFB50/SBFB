// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase F1 — pipeline layer-split shard primitives + forked backend.
//!
//! This module turns the SBFB fork of llama.cpp (vendored under `vendor/`, carrying the
//! pipeline layer-split patch + `TENSOR_SKIP` partial load) into a worker-side backend that
//! executes a CONTIGUOUS window of transformer layers `[layer_start, layer_end)` and either
//!
//! - **first shard** : embeds input tokens, runs its layers, emits the RAW residual stream
//!   at the boundary (no final norm, no lm_head), or
//! - **intermediate shard** : injects the upstream boundary residual as the batch `embd`,
//!   runs its layers, emits the next boundary residual, or
//! - **last shard** : injects the upstream residual, runs its layers, applies the final norm
//!   (the post-norm hidden state is what TOPLOC N0 fingerprints in Phase G).
//!
//! The boundary hand-off is a plain `[n_tokens, n_embd]` row-major fp32 tensor — see
//! [`crate::llm::shard`] doc and `nexus-core-rs` `shard.rs` for the on-wire convention.
//!
//! ## Two compilation tiers
//!
//! [`ShardWindow`] and [`top_k_by_magnitude`] are pure logic, compiled and unit-tested in CI
//! WITHOUT the `llm_llama_cpp` feature (no GGUF, no native build). [`ShardBackend`] is gated
//! behind `llm_llama_cpp` (it needs the forked native runtime) and exercised by `#[ignore]`-d
//! integration tests that require a GGUF on disk — CI never builds the feature, so the fork
//! is covered by the double test discipline (hermetic primitive in CI + GGUF integration
//! runnable locally), per Sprint 77 risk R2.

use std::fmt;

/// Error building a [`ShardWindow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardWindowError {
    /// `start >= end` after resolving `end == 0` to `n_layer` — an empty or inverted window.
    EmptyOrInverted {
        /// Requested inclusive start layer.
        start: u32,
        /// Requested exclusive end layer (already resolved; `0` means `n_layer`).
        end: u32,
    },
    /// The resolved exclusive end is past the model's layer count.
    EndBeyondModel {
        /// Resolved exclusive end layer.
        end: u32,
        /// The model's total transformer layer count.
        n_layer: u32,
    },
}

impl fmt::Display for ShardWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShardWindowError::EmptyOrInverted { start, end } => write!(
                f,
                "empty or inverted shard window [{start}, {end}) (start must be < end)"
            ),
            ShardWindowError::EndBeyondModel { end, n_layer } => write!(
                f,
                "shard window end {end} exceeds model layer count {n_layer}"
            ),
        }
    }
}

impl std::error::Error for ShardWindowError {}

/// A validated contiguous pipeline-split layer window for one shard.
///
/// Layers `[start, end)` are executed by this shard. The convenience value `end == 0`
/// resolves to `n_layer` ("run to the end"). For a contiguous pipeline split, the boundary
/// roles are derived from the window: [`is_first`](Self::is_first) ⟺ the shard owns layer 0
/// (so it embeds input tokens) and [`is_last`](Self::is_last) ⟺ the shard owns the final
/// layer (so it applies the output norm + lm_head). These mirror the
/// `shard_layer_start/end/is_first/is_last` fields the fork reads on BOTH
/// `llama_model_params` (partial load) and `llama_context_params` (partial execution).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShardWindow {
    start: u32,
    end: u32,
    n_layer: u32,
}

impl ShardWindow {
    /// Validate a window against a model of `n_layer` transformer layers. `end == 0` resolves
    /// to `n_layer`.
    ///
    /// # Errors
    ///
    /// Returns [`ShardWindowError`] if the resolved window is empty/inverted or runs past the
    /// model.
    pub fn new(start: u32, end: u32, n_layer: u32) -> Result<Self, ShardWindowError> {
        let end = if end == 0 { n_layer } else { end };
        if end > n_layer {
            return Err(ShardWindowError::EndBeyondModel { end, n_layer });
        }
        if start >= end {
            return Err(ShardWindowError::EmptyOrInverted { start, end });
        }
        Ok(Self {
            start,
            end,
            n_layer,
        })
    }

    /// Inclusive start layer.
    #[must_use]
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Exclusive end layer (resolved; never `0`).
    #[must_use]
    pub fn end(&self) -> u32 {
        self.end
    }

    /// Number of transformer layers this shard executes.
    #[must_use]
    pub fn n_layers(&self) -> u32 {
        self.end - self.start
    }

    /// Whether this shard owns layer 0 (embeds input tokens).
    #[must_use]
    pub fn is_first(&self) -> bool {
        self.start == 0
    }

    /// Whether this shard owns the final layer (applies output norm + lm_head).
    #[must_use]
    pub fn is_last(&self) -> bool {
        self.end == self.n_layer
    }
}

/// Extract the `k` entries of `values` with the largest absolute value, as `(index, value)`
/// pairs sorted by descending magnitude (ties broken by ascending index, fully deterministic).
///
/// This is the lossless top-k material the N0 TOPLOC fingerprint (Phase G) hashes; Phase F1
/// only proves it is extractable from a boundary hidden state without information loss. `NaN`
/// magnitudes sort last (treated as the smallest). `k` is clamped to `values.len()`.
#[must_use]
pub fn top_k_by_magnitude(values: &[f32], k: usize) -> Vec<(u32, f32)> {
    let mut idx: Vec<u32> = (0..u32::try_from(values.len()).unwrap_or(u32::MAX)).collect();
    idx.sort_by(|&a, &b| {
        let va = values[a as usize].abs();
        let vb = values[b as usize].abs();
        // Descending by magnitude; NaN is treated as smallest so it sinks to the end.
        match (va.is_nan(), vb.is_nan()) {
            (true, true) => a.cmp(&b),
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => vb
                .partial_cmp(&va)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b)),
        }
    });
    idx.into_iter()
        .take(k.min(values.len()))
        .map(|i| (i, values[i as usize]))
        .collect()
}

/// The number of tokens in a flat `[n_tokens, n_embd]` row-major hidden-state buffer, or `None`
/// if `hidden_len` is not a positive multiple of `n_embd`. Used to validate an injected boundary
/// residual before the unsafe batch write in [`ShardBackend::forward_hidden`]; pure so it is
/// unit-tested in CI without a GGUF.
#[must_use]
pub fn hidden_token_count(hidden_len: usize, n_embd: usize) -> Option<usize> {
    if n_embd == 0 || hidden_len == 0 || hidden_len % n_embd != 0 {
        return None;
    }
    Some(hidden_len / n_embd)
}

/// Sprint 77 Phase G — compute the **N0 TOPLOC commitment** of a post-norm
/// hidden state `hidden` (one token's `[n_embd]` activation vector, typically
/// the last token of the last shard): extract the top-`TOPLOC_TOP_K`
/// activations by magnitude (lossless, [`top_k_by_magnitude`]) and commit to
/// the canonical all-integer [`nexus_core_rs::ToplocFingerprint`]. Returns the
/// 32-byte BLAKE3 commitment to write into `RunProof::activation_fingerprint`
/// (shard path) or `ResultPayload::logprobs_hash` (whole-model path).
///
/// Pure (no GGUF, no native build) so it is unit-tested in CI; the gated
/// backend wires it through [`ShardBackend::toploc_commitment_last_token`].
#[must_use]
pub fn toploc_commitment(hidden: &[f32]) -> [u8; 32] {
    let topk = top_k_by_magnitude(hidden, nexus_core_rs::TOPLOC_TOP_K);
    nexus_core_rs::ToplocFingerprint::from_topk(&topk).commitment()
}

#[cfg(feature = "llm_llama_cpp")]
pub use backend::{ShardBackend, ShardBackendForwarder};

#[cfg(feature = "llm_llama_cpp")]
mod backend {
    use std::num::NonZeroU32;
    use std::path::PathBuf;
    use std::sync::Arc;

    use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
    use llama_cpp_2::llama_batch::LlamaBatch;
    use llama_cpp_2::model::params::LlamaModelParams;
    use llama_cpp_2::model::{AddBos, LlamaModel};
    use llama_cpp_2::token::LlamaToken;

    use super::ShardWindow;
    use crate::llm::llama_cpp::shared_backend;
    use crate::llm::{LlmBackendError, LlmBackendResult};

    /// A worker-side pipeline-split shard backend over the SBFB fork of llama.cpp.
    ///
    /// On [`load`](Self::load) it allocates VRAM for ONLY its layer window (partial load via
    /// the fork's `TENSOR_SKIP`), then [`forward_tokens`](Self::forward_tokens) (first shard)
    /// or [`forward_hidden`](Self::forward_hidden) (downstream shard) runs the window and
    /// returns the boundary hidden state as a `[n_tokens, n_embd]` row-major fp32 buffer.
    ///
    /// A fresh context is created per forward for simplicity (Phase F1 proves the mechanism);
    /// a long-lived shard session reuses one context across decode steps (Phase F2 wiring).
    pub struct ShardBackend {
        model: Arc<LlamaModel>,
        window: ShardWindow,
        n_embd: usize,
        n_ctx: u32,
    }

    impl std::fmt::Debug for ShardBackend {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ShardBackend")
                .field("window", &self.window)
                .field("n_embd", &self.n_embd)
                .field("n_ctx", &self.n_ctx)
                .finish()
        }
    }

    impl ShardBackend {
        /// Load a model with ONLY the layers `[layer_start, layer_end)` resident.
        ///
        /// `is_first` / `is_last` are the shard's boundary roles, sourced from the
        /// `ShardAssignment` (the scheduler knows the total layer count). They drive the
        /// fork's partial load (`is_first` keeps the token embedding; `is_last` keeps the
        /// output norm + lm_head). The window is re-validated against the loaded model and
        /// the flags must agree with a contiguous split, else an error is returned.
        ///
        /// # Errors
        ///
        /// [`LlmBackendError`] if the native backend cannot init, the GGUF cannot load, or the
        /// window/flags are inconsistent with the loaded model.
        ///
        /// # Aborts
        ///
        /// An out-of-range window (`layer_start >= layer_end`, or `layer_end > n_layer`) trips a
        /// native `GGML_ASSERT` inside `load_from_file` that aborts the process — it fires BEFORE
        /// the recoverable [`ShardWindow`] check below. The scheduler (placement) and the F2 claim
        /// gate MUST pre-validate the window against the model's known layer count; this post-load
        /// check is a defensive backstop for already-bounded inputs, not a guard against arbitrary
        /// windows. Moving the validation ahead of the native load is tracked as a P2 for F2.
        pub fn load(
            model_path: impl Into<PathBuf>,
            layer_start: u32,
            layer_end: u32,
            is_first: bool,
            is_last: bool,
            n_gpu_layers: u32,
            n_ctx: u32,
        ) -> LlmBackendResult<Self> {
            let backend = shared_backend()?;
            let model_path = model_path.into();

            let model_params = LlamaModelParams::default()
                .with_n_gpu_layers(n_gpu_layers)
                .with_shard_range(
                    i32::try_from(layer_start).unwrap_or(i32::MAX),
                    i32::try_from(layer_end).unwrap_or(i32::MAX),
                    is_first,
                    is_last,
                );

            let model = LlamaModel::load_from_file(backend, model_path.clone(), &model_params)
                .map_err(|e| {
                    LlmBackendError::Api(format!(
                        "shard load GGUF {} [{layer_start},{layer_end}): {e}",
                        model_path.display()
                    ))
                })?;

            // Fail closed on architecture: only the LLAMA-family graph builder is patched for the
            // layer-split, and the partial-load TENSOR_SKIP is applied to every arch. A sharded
            // non-llama GGUF would skip layer tensors its unpatched builder still references. The
            // scheduler/F2 only ever assigns shards of supported models; this is the defensive
            // backstop at the API boundary. Llama / Mistral / Mixtral all report "llama".
            let arch = model
                .meta_val_str("general.architecture")
                .unwrap_or_default();
            if arch != "llama" {
                return Err(LlmBackendError::InvalidConfig {
                    reason: format!(
                        "shard backend supports llama-arch GGUFs only, got architecture {arch:?}"
                    ),
                });
            }

            let n_layer = model.n_layer();
            let window = ShardWindow::new(layer_start, layer_end, n_layer).map_err(|e| {
                LlmBackendError::InvalidConfig {
                    reason: e.to_string(),
                }
            })?;

            // A contiguous pipeline split fully determines the boundary roles; reject a
            // caller whose flags disagree (they fed the fork's partial load wrong).
            if window.is_first() != is_first || window.is_last() != is_last {
                return Err(LlmBackendError::InvalidConfig {
                    reason: format!(
                        "shard flags (is_first={is_first}, is_last={is_last}) disagree with window \
                         [{layer_start},{layer_end}) over {n_layer} layers (derived \
                         is_first={}, is_last={})",
                        window.is_first(),
                        window.is_last()
                    ),
                });
            }

            let n_embd = usize::try_from(model.n_embd()).unwrap_or(0);
            if n_embd == 0 {
                return Err(LlmBackendError::Api(
                    "model reports zero embedding width".to_string(),
                ));
            }

            Ok(Self {
                model: Arc::new(model),
                window,
                n_embd,
                n_ctx,
            })
        }

        /// The shard's validated layer window.
        #[must_use]
        pub fn window(&self) -> ShardWindow {
            self.window
        }

        /// The model's embedding width (the boundary hidden state row stride).
        #[must_use]
        pub fn n_embd(&self) -> usize {
            self.n_embd
        }

        /// Sprint 77 Phase G — the **N0 TOPLOC commitment** of this shard's
        /// boundary output for the LAST token. Only meaningful on the last
        /// shard, where the boundary is the post-norm hidden state TOPLOC
        /// fingerprints. `boundary` is the `[n_tokens, n_embd]` row-major fp32
        /// buffer from [`Self::forward_tokens`] / [`Self::forward_hidden`].
        ///
        /// # Errors
        ///
        /// [`LlmBackendError`] if `boundary` is not a positive multiple of
        /// `n_embd`.
        pub fn toploc_commitment_last_token(&self, boundary: &[f32]) -> LlmBackendResult<[u8; 32]> {
            let n_tokens =
                super::hidden_token_count(boundary.len(), self.n_embd).ok_or_else(|| {
                    LlmBackendError::InvalidConfig {
                        reason: format!(
                            "toploc boundary length {} is not a positive multiple of n_embd {}",
                            boundary.len(),
                            self.n_embd
                        ),
                    }
                })?;
            let last = &boundary[(n_tokens - 1) * self.n_embd..];
            Ok(super::toploc_commitment(last))
        }

        fn context_params(&self) -> LlamaContextParams {
            let mut p = LlamaContextParams::default()
                // Per-token hidden states (no pooling) are required to read the boundary
                // residual of every token via `embeddings_ith`.
                .with_embeddings(true)
                .with_pooling_type(LlamaPoolingType::None)
                .with_shard_range(
                    i32::try_from(self.window.start()).unwrap_or(i32::MAX),
                    i32::try_from(self.window.end()).unwrap_or(i32::MAX),
                    self.window.is_first(),
                    self.window.is_last(),
                );
            if let Some(nz) = NonZeroU32::new(self.n_ctx) {
                p = p.with_n_ctx(Some(nz));
            }
            p
        }

        /// First-shard forward: embed `token_ids`, run the window, return the boundary hidden
        /// state as `[n_tokens, n_embd]` row-major fp32.
        ///
        /// # Errors
        ///
        /// [`LlmBackendError`] on context/decode/extraction failure, or if called on a
        /// non-first shard.
        pub fn forward_tokens(&self, token_ids: &[i32]) -> LlmBackendResult<Vec<f32>> {
            if !self.window.is_first() {
                return Err(LlmBackendError::InvalidConfig {
                    reason: "forward_tokens called on a non-first shard (use forward_hidden)"
                        .to_string(),
                });
            }
            if token_ids.is_empty() {
                return Err(LlmBackendError::InvalidConfig {
                    reason: "forward_tokens called with no tokens".to_string(),
                });
            }
            let backend = shared_backend()?;
            let mut ctx = self
                .model
                .new_context(backend, self.context_params())
                .map_err(|e| LlmBackendError::Api(format!("shard new_context: {e}")))?;

            let mut batch = LlamaBatch::new(token_ids.len(), 1);
            for (i, &tok) in token_ids.iter().enumerate() {
                batch
                    .add(LlamaToken::new(tok), i as i32, &[0], true)
                    .map_err(|e| LlmBackendError::Api(format!("shard batch.add: {e}")))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| LlmBackendError::Api(format!("shard decode tokens: {e}")))?;

            self.collect_boundary(&ctx, token_ids.len())
        }

        /// Downstream-shard forward: inject the upstream boundary hidden state `hidden`
        /// (`[n_tokens, n_embd]` row-major fp32), run the window, return this shard's boundary
        /// hidden state in the same layout.
        ///
        /// # Errors
        ///
        /// [`LlmBackendError`] on a shape mismatch, on context/decode/extraction failure, or if
        /// called on a first shard.
        pub fn forward_hidden(&self, hidden: &[f32]) -> LlmBackendResult<Vec<f32>> {
            if self.window.is_first() {
                return Err(LlmBackendError::InvalidConfig {
                    reason: "forward_hidden called on the first shard (use forward_tokens)"
                        .to_string(),
                });
            }
            let n_tokens = super::hidden_token_count(hidden.len(), self.n_embd).ok_or_else(
                || LlmBackendError::InvalidConfig {
                    reason: format!(
                        "injected hidden state length {} is not a positive multiple of n_embd {}",
                        hidden.len(),
                        self.n_embd
                    ),
                },
            )?;
            let backend = shared_backend()?;
            let mut ctx = self
                .model
                .new_context(backend, self.context_params())
                .map_err(|e| LlmBackendError::Api(format!("shard new_context: {e}")))?;

            let mut batch = LlamaBatch::new_embeddings(n_tokens, self.n_embd, 1);
            for i in 0..n_tokens {
                let row = &hidden[i * self.n_embd..(i + 1) * self.n_embd];
                batch
                    .add_embedding(row, i as i32, &[0], true)
                    .map_err(|e| LlmBackendError::Api(format!("shard add_embedding: {e}")))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| LlmBackendError::Api(format!("shard decode hidden: {e}")))?;

            self.collect_boundary(&ctx, n_tokens)
        }

        /// Tokenize `text` with this model's vocab (BOS-prefixed), returning raw token ids —
        /// the first shard's input. Convenience for callers/tests that hold text.
        ///
        /// # Errors
        ///
        /// [`LlmBackendError`] if tokenization fails.
        pub fn tokenize(&self, text: &str) -> LlmBackendResult<Vec<i32>> {
            let toks = self
                .model
                .str_to_token(text, AddBos::Always)
                .map_err(|e| LlmBackendError::Api(format!("shard tokenize: {e}")))?;
            Ok(toks.into_iter().map(|t| t.0).collect())
        }

        fn collect_boundary(
            &self,
            ctx: &llama_cpp_2::context::LlamaContext<'_>,
            n_tokens: usize,
        ) -> LlmBackendResult<Vec<f32>> {
            let mut out = Vec::with_capacity(n_tokens * self.n_embd);
            for i in 0..n_tokens {
                let emb = ctx
                    .embeddings_ith(i as i32)
                    .map_err(|e| LlmBackendError::Api(format!("shard embeddings_ith({i}): {e}")))?;
                if emb.len() != self.n_embd {
                    return Err(LlmBackendError::Api(format!(
                        "shard boundary row {i} width {} != n_embd {}",
                        emb.len(),
                        self.n_embd
                    )));
                }
                out.extend_from_slice(emb);
            }
            Ok(out)
        }
    }

    /// A [`nexus_core_rs::ShardForwarder`] over a loaded [`ShardBackend`].
    ///
    /// The Sprint 77 Phase F2 dependency-inversion bridge: it lets the core-rs
    /// `sbfb/shard/1` handler ([`nexus_core_rs::ShardProtocol`]) run this
    /// worker's layer block without `nexus-core-rs` depending on
    /// `nexus-worker-core`. The `accept()` server is always a downstream shard,
    /// so each inbound frame is an upstream boundary hidden state: it is decoded
    /// from row-major **little-endian fp32** bytes (the documented wire shape),
    /// run through [`ShardBackend::forward_hidden`], and the resulting boundary
    /// state re-encoded the same way.
    #[derive(Debug)]
    pub struct ShardBackendForwarder {
        backend: Arc<ShardBackend>,
    }

    impl ShardBackendForwarder {
        /// Wrap a loaded backend as a data-plane forwarder.
        #[must_use]
        pub fn new(backend: Arc<ShardBackend>) -> Self {
            Self { backend }
        }
    }

    impl nexus_core_rs::ShardForwarder for ShardBackendForwarder {
        fn forward(&self, upstream_frame: &[u8]) -> nexus_core_rs::Result<Vec<u8>> {
            if upstream_frame.len() % 4 != 0 {
                return Err(nexus_core_rs::NexusError::Other(format!(
                    "shard frame {} bytes is not a multiple of 4 (fp32)",
                    upstream_frame.len()
                )));
            }
            let hidden: Vec<f32> = upstream_frame
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let out = self.backend.forward_hidden(&hidden).map_err(|e| {
                nexus_core_rs::NexusError::Other(format!("shard forward_hidden: {e}"))
            })?;
            let mut bytes = Vec::with_capacity(out.len() * 4);
            for v in out {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Ok(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_window_validates_contiguous_range() {
        let w = ShardWindow::new(0, 16, 32).expect("valid window");
        assert_eq!(w.start(), 0);
        assert_eq!(w.end(), 16);
        assert_eq!(w.n_layers(), 16);
        assert!(w.is_first(), "owns layer 0");
        assert!(!w.is_last(), "does not own the final layer");

        let mid = ShardWindow::new(16, 24, 32).expect("valid mid window");
        assert!(!mid.is_first());
        assert!(!mid.is_last());
        assert_eq!(mid.n_layers(), 8);

        let tail = ShardWindow::new(24, 32, 32).expect("valid tail window");
        assert!(!tail.is_first());
        assert!(tail.is_last(), "owns the final layer");
    }

    #[test]
    fn shard_window_end_zero_means_n_layer() {
        let w = ShardWindow::new(24, 0, 32).expect("end==0 resolves to n_layer");
        assert_eq!(w.end(), 32);
        assert!(w.is_last());

        let whole = ShardWindow::new(0, 0, 32).expect("whole model");
        assert!(
            whole.is_first() && whole.is_last(),
            "single shard = whole model"
        );
        assert_eq!(whole.n_layers(), 32);
    }

    #[test]
    fn shard_window_rejects_invalid() {
        assert_eq!(
            ShardWindow::new(16, 8, 32),
            Err(ShardWindowError::EmptyOrInverted { start: 16, end: 8 }),
            "inverted window rejected"
        );
        assert_eq!(
            ShardWindow::new(8, 8, 32),
            Err(ShardWindowError::EmptyOrInverted { start: 8, end: 8 }),
            "empty window rejected"
        );
        assert_eq!(
            ShardWindow::new(0, 40, 32),
            Err(ShardWindowError::EndBeyondModel {
                end: 40,
                n_layer: 32
            }),
            "window past the model rejected"
        );
    }

    #[test]
    fn top_k_extracts_largest_by_magnitude_deterministically() {
        // Negative magnitudes count; ties break by ascending index.
        let v = [0.1_f32, -5.0, 3.0, -5.0, 0.0, 2.0];
        let top = top_k_by_magnitude(&v, 3);
        assert_eq!(
            top,
            vec![(1, -5.0), (3, -5.0), (2, 3.0)],
            "top-3 by |value|, ties by index"
        );
    }

    #[test]
    fn top_k_clamps_k_and_handles_nan() {
        let v = [1.0_f32, f32::NAN, 2.0];
        // k larger than len is clamped; NaN sinks to the end.
        let top = top_k_by_magnitude(&v, 10);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0], (2, 2.0));
        assert_eq!(top[1], (0, 1.0));
        assert_eq!(top[2].0, 1, "NaN entry sorts last");
        assert!(top[2].1.is_nan());

        assert!(top_k_by_magnitude(&v, 0).is_empty(), "k=0 yields nothing");
    }

    #[test]
    fn hidden_token_count_validates_shape() {
        assert_eq!(hidden_token_count(12, 4), Some(3));
        assert_eq!(hidden_token_count(4, 4), Some(1));
        assert_eq!(hidden_token_count(0, 4), None, "empty buffer rejected");
        assert_eq!(
            hidden_token_count(5, 4),
            None,
            "non-multiple of n_embd rejected"
        );
        assert_eq!(hidden_token_count(4, 0), None, "zero n_embd rejected");
    }

    #[test]
    fn toploc_commitment_is_deterministic_and_swap_sensitive() {
        // Phase G: the worker-side N0 commitment over a hidden state vector is
        // deterministic, and a different hidden state (a swapped model) yields a
        // different commitment.
        let a = [0.1_f32, 5.0, -3.0, 2.0, 0.5, -8.0];
        let b = [0.1_f32, 5.0, -3.0, 2.0, 0.5, -8.0];
        assert_eq!(
            toploc_commitment(&a),
            toploc_commitment(&b),
            "same hidden state → same commitment"
        );
        let c = [9.0_f32, -1.0, 0.2, 7.0, -4.0, 0.3];
        assert_ne!(
            toploc_commitment(&a),
            toploc_commitment(&c),
            "different hidden state → different commitment"
        );
    }
}

// Integration tests against a real GGUF model. Gated behind the feature AND `#[ignore]`d:
// CI never builds `llm_llama_cpp`, so these are run locally with the model on disk via
//   SBFB_SHARD_TEST_GGUF=/path/to/llama-arch-q4.gguf \
//     cargo test -p nexus-worker-core --features llm_llama_cpp -- --ignored shard_backend
#[cfg(all(test, feature = "llm_llama_cpp"))]
mod gguf_tests {
    use super::*;

    fn gguf_path() -> Option<std::path::PathBuf> {
        std::env::var_os("SBFB_SHARD_TEST_GGUF").map(std::path::PathBuf::from)
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        dot / (na * nb)
    }

    /// P-D: a windowed load succeeds and reports the right geometry. The "VRAM reduced" claim
    /// is proven structurally (only `[start,end)` layers are created; the fork's TENSOR_SKIP
    /// subtracts the rest from `size_data`) and observable in the loader's "unused tensor"
    /// logs; here we assert the load is accepted and the boundary width matches n_embd.
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn shard_backend_loads_layer_subset() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        // First shard over the first half of the layers (end resolved at load).
        let shard = ShardBackend::load(&path, 0, 1, true, false, 0, 512)
            .expect("load first 1 layer as a first shard");
        assert!(shard.n_embd() > 0);
        assert_eq!(shard.window().start(), 0);
        assert!(shard.window().is_first());
        assert!(!shard.window().is_last());
    }

    /// Spike property ported to a test: a full forward equals a split forward across an
    /// internal boundary (same backend → bit-exact; we assert cosine ~ 1.0 to tolerate any
    /// fp reduction-order drift). Loads the WHOLE model twice with different windows.
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn shard_backend_partial_equals_full() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        let tokens: Vec<i32> = {
            let probe =
                ShardBackend::load(&path, 0, 0, true, true, 0, 512).expect("load whole model");
            probe.tokenize("The quick brown fox").expect("tokenize")
        };

        // Whole model in one shot.
        let whole = ShardBackend::load(&path, 0, 0, true, true, 0, 512).expect("whole");
        let n_layer = whole.window().end();
        let k = n_layer / 2;
        let full = whole.forward_tokens(&tokens).expect("full forward");

        // Same computation split at layer k: [0,k) then [k,n_layer).
        let head = ShardBackend::load(&path, 0, k, true, false, 0, 512).expect("head shard");
        let tail = ShardBackend::load(&path, k, n_layer, false, true, 0, 512).expect("tail shard");
        let boundary = head.forward_tokens(&tokens).expect("head forward");
        let split = tail.forward_hidden(&boundary).expect("tail forward");

        assert_eq!(full.len(), split.len(), "same [n_tokens, n_embd]");
        let cos = cosine(&full, &split);
        assert!(
            cos > 0.999,
            "split forward must match full forward (cosine {cos} <= 0.999)"
        );
    }

    /// Middle-shard proof: a 3-way split head [0,k) + MIDDLE [k,m) + tail [m,n_layer) equals the
    /// full forward. Exercises the intermediate shard (is_first == false && is_last == false) —
    /// the path that NULL-deref'd before the fork kept tok_embd resident on every shard. The
    /// boundary residual is handed off twice (head→middle→tail) via embd injection.
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn shard_backend_three_way_equals_full() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        let tokens: Vec<i32> = {
            let probe =
                ShardBackend::load(&path, 0, 0, true, true, 0, 512).expect("load whole model");
            probe
                .tokenize("The quick brown fox jumps")
                .expect("tokenize")
        };

        let whole = ShardBackend::load(&path, 0, 0, true, true, 0, 512).expect("whole");
        let n_layer = whole.window().end();
        assert!(n_layer >= 3, "need >= 3 layers to form a middle shard");
        let k = n_layer / 3;
        let m = (2 * n_layer) / 3;
        let full = whole.forward_tokens(&tokens).expect("full forward");

        let head = ShardBackend::load(&path, 0, k, true, false, 0, 512).expect("head shard");
        let middle = ShardBackend::load(&path, k, m, false, false, 0, 512).expect("middle shard");
        let tail = ShardBackend::load(&path, m, n_layer, false, true, 0, 512).expect("tail shard");
        assert!(
            !middle.window().is_first() && !middle.window().is_last(),
            "middle is a true intermediate shard"
        );

        let b1 = head.forward_tokens(&tokens).expect("head forward");
        let b2 = middle.forward_hidden(&b1).expect("middle forward");
        let split = tail.forward_hidden(&b2).expect("tail forward");

        assert_eq!(full.len(), split.len(), "same [n_tokens, n_embd]");
        let cos = cosine(&full, &split);
        assert!(
            cos > 0.999,
            "3-way split must match full forward (cosine {cos} <= 0.999)"
        );
    }

    /// The boundary hidden state is extractable and yields lossless top-k (the N0 prerequisite).
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn shard_backend_hidden_state_extractable() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        let shard = ShardBackend::load(&path, 0, 0, true, true, 0, 512).expect("whole model");
        let tokens = shard.tokenize("Hello").expect("tokenize");
        let hidden = shard.forward_tokens(&tokens).expect("forward");
        assert_eq!(hidden.len(), tokens.len() * shard.n_embd());
        // Last token's hidden state → top-128 is extractable and ordered.
        let last = &hidden[(tokens.len() - 1) * shard.n_embd()..];
        let top = top_k_by_magnitude(last, 128.min(shard.n_embd()));
        assert_eq!(top.len(), 128.min(shard.n_embd()));
        for w in top.windows(2) {
            assert!(
                w[0].1.abs() >= w[1].1.abs(),
                "top-k ordered by descending magnitude"
            );
        }
    }
}
