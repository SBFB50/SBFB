// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase F2 — worker-side shard claim gate.
//!
//! Before a worker joins a sharded-inference pipeline it must DECIDE whether to
//! claim the [`ShardAssignment`](nexus_core_rs::ShardAssignment) an initiator
//! assigned it. The decision is split into two **pure** phases so the costly
//! I/O only ever runs for an authorised manifest (crypto-before-I/O, mirroring
//! the data-plane admission that checks membership before `accept_bi`):
//!
//! 1. [`authorize_claim`] — signature of the
//!    [`ShardedSessionManifestEntry`](nexus_core_rs::ShardedSessionManifestEntry)
//!    FIRST, then compute-group membership of self, then this worker's
//!    presence in the signed plan. No file read, no GPU snapshot. A
//!    forged / non-member / not-in-plan manifest is refused here, before any
//!    GGUF read or GPU query (DoS pre-auth).
//! 2. [`assess_capacity`] — given the model facts read from the GGUF header
//!    (see [`read_gguf_model_facts`], feature-gated) and a measured VRAM
//!    snapshot, validate the layer window against the model's layer count
//!    (closes the F1 validation-ordering P2: a window past the model would
//!    otherwise trip a native `GGML_ASSERT` that aborts the process at load),
//!    estimate the shard's resident VRAM, and refuse if it exceeds free VRAM.
//!
//! The capacity check is **fail-closed**: [`estimate_shard_resident_bytes`]
//! over-estimates (a generous fixed backend headroom on top of the exact
//! per-tensor sizes ggml reports, which are themselves a lower bound — they
//! omit the CUDA context, compute buffers and allocator fragmentation), an
//! unknown / unreadable GGUF header is REFUSED (never "estimate 0", which would
//! pass any free-VRAM check), and an unsupported architecture is refused.
//!
//! ## VRAM snapshot, not a live pump (scope cut #7)
//!
//! The capacity check reads a SINGLE point-in-time
//! [`GpuStats::vram_free_bytes`](crate::gpu::GpuStats) at claim time (the
//! `gpu_snapshot` model), exactly like the placement scheduler reads it once.
//! It never arms a continuous VRAM-admission pump — that stays post-S77.

use nexus_core_rs::{ComputeGroupEntry, ShardAssignment, ShardedSessionManifestEntry};

use crate::llm::shard::ShardWindow;

/// Bytes per element of the KV cache (fp16 K/V — the llama.cpp default).
///
/// The KV cache is an estimate folded into a fail-closed headroom, so a
/// backend that runs an fp32 KV cache is still covered by
/// [`VRAM_BACKEND_OVERHEAD_BYTES`]; fp16 is the realistic default.
pub const KV_CACHE_DTYPE_BYTES: u64 = 2;

/// Fixed VRAM headroom added on top of the summed tensor + KV-cache bytes.
///
/// `gguf_get_tensor_size` reports the exact on-disk tensor size but NOT the
/// real device footprint: a load also allocates the backend context (a CUDA
/// context alone is ~300-600 MiB), compute / graph buffers, and pays allocator
/// fragmentation. Summing tensor sizes is therefore a *lower* bound; this
/// generous fixed headroom pushes the claim decision fail-closed (refuse a
/// borderline claim rather than accept one that OOMs at load). 768 MiB.
pub const VRAM_BACKEND_OVERHEAD_BYTES: u64 = 768 * 1024 * 1024;

/// GGUF tensor name of the token embedding matrix. Resident on EVERY shard
/// (the F1 partial load keeps `tok_embd` on each node, mid-shard included).
pub const GGUF_TENSOR_TOKEN_EMBD: &str = "token_embd.weight";

/// GGUF tensor name of the final output norm. Resident on the LAST shard only.
pub const GGUF_TENSOR_OUTPUT_NORM: &str = "output_norm.weight";

/// GGUF tensor name of the output projection (lm_head). Resident on the LAST
/// shard only (and absent / tied to `tok_embd` in some models).
pub const GGUF_TENSOR_OUTPUT: &str = "output.weight";

/// Prefix of a per-layer block tensor name (`blk.{N}.…`).
pub const GGUF_BLOCK_PREFIX: &str = "blk.";

/// Facts a header-only GGUF read yields for the claim gate.
///
/// Plain data so the pure [`assess_capacity`] / [`estimate_shard_resident_bytes`]
/// are unit-tested in CI without a real GGUF (the feature-gated
/// [`read_gguf_model_facts`] populates it from disk on a real run).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GgufModelFacts {
    /// `general.architecture` (the F1 backend supports `"llama"` only).
    pub architecture: String,
    /// `<arch>.block_count` — total transformer layers in the model.
    pub n_layer: u32,
    /// `<arch>.embedding_length` — hidden dimension.
    pub n_embd: u32,
    /// `<arch>.attention.head_count` — number of attention heads.
    pub n_head: u32,
    /// `<arch>.attention.head_count_kv` — number of KV heads (== `n_head` for
    /// multi-head attention; smaller for grouped-query attention).
    pub n_head_kv: u32,
    /// `(name, on-disk byte size)` for every tensor, from
    /// `GgufContext::tensor_size` (block-quantization aware).
    pub tensor_sizes: Vec<(String, u64)>,
}

/// Parameters of the per-shard KV-cache VRAM estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShardVramParams {
    /// Context length the shard will run with (already bounded to
    /// [`nexus_core_rs::MAX_SHARD_N_CTX`] by the caller).
    pub n_ctx: u32,
    /// Number of transformer layers resident on THIS shard.
    pub n_layers_in_shard: u32,
    /// Number of KV heads (`GgufModelFacts::n_head_kv`).
    pub n_kv_heads: u32,
    /// Per-head dimension (`n_embd / n_head`).
    pub head_dim: u32,
}

/// The accepted-claim summary: the validated window + the estimated footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimAccept {
    /// Inclusive first layer this shard runs.
    pub layer_start: u32,
    /// Exclusive last layer this shard runs.
    pub layer_end: u32,
    /// Whether this shard owns layer 0 (embeds input tokens).
    pub is_first: bool,
    /// Whether this shard owns the final layer (applies output norm + lm_head).
    pub is_last: bool,
    /// The fail-closed VRAM estimate that was checked against free VRAM.
    pub required_vram_bytes: u64,
}

/// Why a worker refused to claim an assignment. A refusal is a normal outcome
/// (the worker DEFERS), never a crash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimRejection {
    /// The session manifest signature did not verify.
    BadManifestSignature(String),
    /// This worker is not on the compute group's admission allowlist.
    NotAMember,
    /// This worker has no assignment in the signed plan.
    NotInPlan,
    /// The assignment's layer window is empty / inverted / past the model.
    /// Refusing here is what keeps a signed-but-out-of-range assignment from
    /// reaching the native load (which would abort the process).
    WindowOutOfRange {
        /// Requested inclusive start layer.
        layer_start: u32,
        /// Requested exclusive end layer.
        layer_end: u32,
        /// The model's actual layer count.
        n_layer: u32,
    },
    /// The estimated shard VRAM exceeds the measured free VRAM.
    InsufficientVram {
        /// Fail-closed estimate of the shard's VRAM need.
        required_bytes: u64,
        /// Measured free VRAM at claim time.
        free_bytes: u64,
    },
    /// The GGUF header could not be read (missing / corrupt / unsupported
    /// architecture). Refused fail-closed — never treated as "needs 0 VRAM".
    ModelUnreadable(String),
    /// No GPU snapshot was available to measure free VRAM.
    GpuUnavailable(String),
}

impl std::fmt::Display for ClaimRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimRejection::BadManifestSignature(e) => {
                write!(f, "shard manifest signature invalid: {e}")
            }
            ClaimRejection::NotAMember => write!(f, "worker not on the compute-group allowlist"),
            ClaimRejection::NotInPlan => write!(f, "worker has no assignment in the signed plan"),
            ClaimRejection::WindowOutOfRange {
                layer_start,
                layer_end,
                n_layer,
            } => write!(
                f,
                "assignment window [{layer_start},{layer_end}) out of range for {n_layer}-layer model"
            ),
            ClaimRejection::InsufficientVram {
                required_bytes,
                free_bytes,
            } => write!(
                f,
                "shard needs ~{required_bytes} VRAM bytes, only {free_bytes} free"
            ),
            ClaimRejection::ModelUnreadable(e) => write!(f, "model header unreadable: {e}"),
            ClaimRejection::GpuUnavailable(e) => write!(f, "gpu snapshot unavailable: {e}"),
        }
    }
}

impl std::error::Error for ClaimRejection {}

/// Parse the layer index of a `blk.{N}.…` tensor name (e.g.
/// `"blk.12.attn_q.weight" -> Some(12)`). Returns `None` for non-block tensors
/// (`token_embd`, `output_norm`, `output`, …). The trailing `.` in the split
/// is what makes `blk.1.` distinct from `blk.10.`.
fn blk_layer_index(name: &str) -> Option<u32> {
    let rest = name.strip_prefix(GGUF_BLOCK_PREFIX)?;
    let (num, _) = rest.split_once('.')?;
    num.parse::<u32>().ok()
}

/// Whether a model geometry is degenerate / hostile and must be REFUSED rather
/// than estimated. A 0 layer/embd/head count — or an `n_embd < n_head` that
/// makes integer `head_dim = n_embd / n_head` collapse to 0 — would zero out a
/// VRAM term (head_dim 0 → KV estimate 0 → a multi-GB KV cache silently
/// under-counted past the fixed headroom → fail-OPEN). A valid attention model
/// always has `n_embd == n_head * head_dim >= n_head` (head_dim ≥ 1). Pure so
/// the fail-closed guard is unit-tested in CI without a GGUF (the feature-gated
/// [`read_gguf_model_facts`] applies it on a real header).
#[must_use]
pub fn is_degenerate_geometry(n_layer: u32, n_embd: u32, n_head: u32) -> bool {
    n_layer == 0 || n_embd == 0 || n_head == 0 || n_embd < n_head
}

/// Estimate the VRAM a shard's resident weights + KV cache + backend overhead
/// require, in bytes. **Fail-closed**: an over-estimate refuses a borderline
/// claim; it never under-counts on purpose.
///
/// `tensor_sizes` are `(name, on-disk bytes)` for every tensor in the GGUF.
/// Resident on this shard:
/// - every `blk.{i}.*` tensor with `i ∈ [window.start(), window.end())`,
/// - `token_embd.weight` always (the F1 partial load keeps it on every shard),
/// - `output_norm.weight` and `output.weight` only when `window.is_last()`.
///
/// Then a KV-cache estimate (`2 · n_layers · n_ctx · n_kv_heads · head_dim ·
/// [`KV_CACHE_DTYPE_BYTES`]`) and a fixed [`VRAM_BACKEND_OVERHEAD_BYTES`]
/// headroom. All arithmetic saturates (a pathological GGUF can never wrap the
/// estimate to a small number that would pass the check).
#[must_use]
pub fn estimate_shard_resident_bytes(
    tensor_sizes: &[(String, u64)],
    window: ShardWindow,
    params: ShardVramParams,
) -> u64 {
    let mut weight_bytes: u64 = 0;
    for (name, size) in tensor_sizes {
        let resident = match blk_layer_index(name) {
            Some(layer) => layer >= window.start() && layer < window.end(),
            None => {
                name == GGUF_TENSOR_TOKEN_EMBD
                    || (window.is_last()
                        && (name == GGUF_TENSOR_OUTPUT_NORM || name == GGUF_TENSOR_OUTPUT))
            }
        };
        if resident {
            weight_bytes = weight_bytes.saturating_add(*size);
        }
    }

    let kv_bytes = 2u64
        .saturating_mul(u64::from(params.n_layers_in_shard))
        .saturating_mul(u64::from(params.n_ctx))
        .saturating_mul(u64::from(params.n_kv_heads))
        .saturating_mul(u64::from(params.head_dim))
        .saturating_mul(KV_CACHE_DTYPE_BYTES);

    weight_bytes
        .saturating_add(kv_bytes)
        .saturating_add(VRAM_BACKEND_OVERHEAD_BYTES)
}

/// Phase 1 — crypto + membership gate (PURE, no I/O). MUST run before any GGUF
/// read or GPU snapshot.
///
/// Order is load-bearing (DoS pre-auth): signature FIRST, then membership, then
/// in-plan. Returns this worker's [`ShardAssignment`] (borrowed from the
/// manifest) on success.
///
/// # Errors
///
/// [`ClaimRejection::BadManifestSignature`] / [`ClaimRejection::NotAMember`] /
/// [`ClaimRejection::NotInPlan`].
pub fn authorize_claim<'a>(
    manifest_entry: &'a ShardedSessionManifestEntry,
    group: &ComputeGroupEntry,
    self_pubkey: &[u8; 32],
) -> Result<&'a ShardAssignment, ClaimRejection> {
    // 1. Signature FIRST — a forged manifest never reaches membership / I/O.
    manifest_entry
        .verify_signature()
        .map_err(|e| ClaimRejection::BadManifestSignature(e.to_string()))?;

    // 2. This worker must be on the compute-group allowlist.
    if !group.is_member(self_pubkey) {
        return Err(ClaimRejection::NotAMember);
    }

    // 3. This worker must have an assignment in the signed plan.
    manifest_entry
        .manifest
        .plan
        .assignments
        .iter()
        .find(|a| &a.worker_pubkey == self_pubkey)
        .ok_or(ClaimRejection::NotInPlan)
}

/// Phase 2 — capacity gate (PURE, no I/O). Runs AFTER the GGUF header read +
/// GPU snapshot that the authorised phase-1 pass justified.
///
/// Validates the layer window against the model's layer count (pre-load —
/// closes the F1 validation-ordering P2) and refuses fail-closed if the
/// estimated shard VRAM exceeds `vram_free_bytes`.
///
/// `effective_n_ctx` must already be bounded by the caller (see
/// [`nexus_core_rs::MAX_SHARD_N_CTX`]) so the KV estimate matches the context
/// the shard will actually run with.
///
/// # Errors
///
/// [`ClaimRejection::WindowOutOfRange`] / [`ClaimRejection::InsufficientVram`].
pub fn assess_capacity(
    assignment: &ShardAssignment,
    facts: &GgufModelFacts,
    effective_n_ctx: u32,
    vram_free_bytes: u64,
) -> Result<ClaimAccept, ClaimRejection> {
    let window = ShardWindow::new(assignment.layer_start, assignment.layer_end, facts.n_layer)
        .map_err(|_| ClaimRejection::WindowOutOfRange {
            layer_start: assignment.layer_start,
            layer_end: assignment.layer_end,
            n_layer: facts.n_layer,
        })?;

    // `checked_div` yields None on a zero head count (malformed metadata),
    // collapsing head_dim to 0; the KV term then degenerates to 0 and the fixed
    // backend headroom still keeps the decision fail-closed.
    let head_dim = facts.n_embd.checked_div(facts.n_head).unwrap_or(0);
    let params = ShardVramParams {
        n_ctx: effective_n_ctx,
        n_layers_in_shard: window.n_layers(),
        n_kv_heads: facts.n_head_kv,
        head_dim,
    };
    let required = estimate_shard_resident_bytes(&facts.tensor_sizes, window, params);

    if required > vram_free_bytes {
        return Err(ClaimRejection::InsufficientVram {
            required_bytes: required,
            free_bytes: vram_free_bytes,
        });
    }

    Ok(ClaimAccept {
        layer_start: window.start(),
        layer_end: window.end(),
        is_first: window.is_first(),
        is_last: window.is_last(),
        required_vram_bytes: required,
    })
}

/// Read the model facts the claim gate needs from a GGUF header — no tensor
/// weights are loaded (`no_alloc`), so this is cheap and runs before the real
/// native load. Feature-gated on the forked llama.cpp runtime (the same
/// feature that makes a worker able to run a shard at all).
///
/// Fail-closed: any missing / malformed metadata yields
/// [`ClaimRejection::ModelUnreadable`] so the caller refuses the claim rather
/// than proceeding with a bogus VRAM estimate.
///
/// # Errors
///
/// [`ClaimRejection::ModelUnreadable`] if the file is not a readable GGUF or a
/// required metadata key is missing / mistyped.
#[cfg(feature = "llm_llama_cpp")]
pub fn read_gguf_model_facts(path: &std::path::Path) -> Result<GgufModelFacts, ClaimRejection> {
    use llama_cpp_2::gguf::GgufContext;

    let ctx = GgufContext::from_file(path).ok_or_else(|| {
        ClaimRejection::ModelUnreadable(format!("cannot open GGUF header {}", path.display()))
    })?;

    // Typed metadata getters live on GgufContext (the vendored wrapper that owns
    // the `llama_cpp_sys_2` FFI); this crate reads facts without depending on
    // `llama-cpp-sys-2` directly.
    let architecture = ctx.meta_str("general.architecture").ok_or_else(|| {
        ClaimRejection::ModelUnreadable("missing general.architecture".to_string())
    })?;

    let n_layer = ctx
        .meta_u32(&format!("{architecture}.block_count"))
        .ok_or_else(|| {
            ClaimRejection::ModelUnreadable(format!("missing {architecture}.block_count"))
        })?;
    let n_embd = ctx
        .meta_u32(&format!("{architecture}.embedding_length"))
        .ok_or_else(|| {
            ClaimRejection::ModelUnreadable(format!("missing {architecture}.embedding_length"))
        })?;
    let n_head = ctx
        .meta_u32(&format!("{architecture}.attention.head_count"))
        .ok_or_else(|| {
            ClaimRejection::ModelUnreadable(format!("missing {architecture}.attention.head_count"))
        })?;
    // KV head count defaults to head_count for multi-head attention (older
    // GGUFs omit it). A present-but-zero value (malformed) is also treated as
    // absent → falls back to head_count, never a 0 that would zero the KV term.
    let n_head_kv = ctx
        .meta_u32(&format!("{architecture}.attention.head_count_kv"))
        .filter(|&v| v != 0)
        .unwrap_or(n_head);

    // Fail-closed on a degenerate / hostile geometry that would zero a VRAM
    // term and under-count past the headroom (Codex F2 R1 PARTIEL). Refuse
    // rather than degrade the estimate. See [`is_degenerate_geometry`].
    if is_degenerate_geometry(n_layer, n_embd, n_head) {
        return Err(ClaimRejection::ModelUnreadable(format!(
            "degenerate {architecture} geometry (n_layer={n_layer}, n_embd={n_embd}, n_head={n_head})"
        )));
    }

    let n_tensors = ctx.n_tensors();
    let mut tensor_sizes = Vec::with_capacity(usize::try_from(n_tensors.max(0)).unwrap_or(0));
    for i in 0..n_tensors {
        // Fail-closed: a tensor whose name cannot be read (null / invalid UTF-8)
        // would be SKIPPED and its bytes dropped from the resident sum → an
        // under-estimate that could accept a claim that then OOMs. Refuse the
        // whole header instead of silently undercounting (Codex F2 R1 PARTIEL).
        let name = ctx.tensor_name(i).ok_or_else(|| {
            ClaimRejection::ModelUnreadable(format!("tensor {i} has an unreadable name"))
        })?;
        tensor_sizes.push((name.to_string(), ctx.tensor_size(i)));
    }

    Ok(GgufModelFacts {
        architecture,
        n_layer,
        n_embd,
        n_head,
        n_head_kv,
        tensor_sizes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::{
        ComputeGroup, KeyPair, KvCachePolicy, ShardPlan, ShardRole, ShardedSessionManifest,
        ShardedSessionManifestEntry,
    };

    fn assignment_for(worker: &[u8; 32], start: u32, end: u32) -> ShardAssignment {
        ShardAssignment {
            worker_pubkey: *worker,
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: vec![[7u8; 32]],
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [9u8; 32],
        }
    }

    /// A signed manifest placing `worker` on `[start,end)` of a model.
    fn signed_manifest(
        initiator: &KeyPair,
        worker: &[u8; 32],
        start: u32,
        end: u32,
    ) -> ShardedSessionManifestEntry {
        let manifest = ShardedSessionManifest::new(
            initiator.public_bytes(),
            "session-shard-1",
            "pilot-shard",
            1,
            ShardPlan::new(vec![assignment_for(worker, start, end)]),
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        );
        ShardedSessionManifestEntry::sign(manifest, initiator).unwrap()
    }

    fn group_with(initiator: &KeyPair, members: &[&[u8; 32]]) -> ComputeGroupEntry {
        let mut g = ComputeGroup::new(initiator.public_bytes(), "pilot-shard", 1);
        for m in members {
            g = g.with_member(**m);
        }
        ComputeGroupEntry::sign(g, initiator).unwrap()
    }

    fn facts(n_layer: u32) -> GgufModelFacts {
        // 3 layers × (one weight per layer) + tok_embd + output, sizes chosen
        // so a single layer is well under a GiB but several layers + headroom
        // can exceed a tight free budget.
        let mut tensor_sizes = vec![
            (GGUF_TENSOR_TOKEN_EMBD.to_string(), 50_000_000),
            (GGUF_TENSOR_OUTPUT_NORM.to_string(), 1_000),
            (GGUF_TENSOR_OUTPUT.to_string(), 50_000_000),
        ];
        for i in 0..n_layer {
            tensor_sizes.push((format!("blk.{i}.attn_q.weight"), 100_000_000));
        }
        GgufModelFacts {
            architecture: "llama".to_string(),
            n_layer,
            n_embd: 4096,
            n_head: 32,
            n_head_kv: 8,
            tensor_sizes,
        }
    }

    #[test]
    fn blk_layer_index_parses_and_disambiguates() {
        assert_eq!(blk_layer_index("blk.0.attn_q.weight"), Some(0));
        assert_eq!(blk_layer_index("blk.12.ffn_down.weight"), Some(12));
        assert_eq!(blk_layer_index("token_embd.weight"), None);
        assert_eq!(blk_layer_index("output.weight"), None);
        // The trailing dot disambiguates 1 from 10/12.
        assert_eq!(blk_layer_index("blk.1.x"), Some(1));
        assert_ne!(blk_layer_index("blk.10.x"), Some(1));
    }

    #[test]
    fn degenerate_geometry_is_rejected_fail_closed() {
        // A healthy llama geometry is accepted.
        assert!(!is_degenerate_geometry(32, 4096, 32));
        assert!(
            !is_degenerate_geometry(1, 32, 32),
            "n_embd == n_head ok (head_dim 1)"
        );
        // Zero counts (malformed / hostile) are degenerate.
        assert!(is_degenerate_geometry(0, 4096, 32), "0 layers");
        assert!(is_degenerate_geometry(32, 0, 32), "0 embd");
        assert!(is_degenerate_geometry(32, 4096, 0), "0 heads");
        // n_embd < n_head would integer-divide head_dim to 0 (KV term → 0,
        // fail-OPEN) — must be rejected.
        assert!(
            is_degenerate_geometry(32, 10, 32),
            "n_embd < n_head collapses head_dim to 0"
        );
    }

    #[test]
    fn estimate_residents_depend_on_is_last() {
        let model = facts(6);
        // Tail shard [4,6): 2 layers + tok_embd + output_norm + output.
        let tail = ShardWindow::new(4, 6, 6).unwrap();
        let p = ShardVramParams {
            n_ctx: 0, // isolate the weight sum from the KV term
            n_layers_in_shard: tail.n_layers(),
            n_kv_heads: 8,
            head_dim: 128,
        };
        let tail_bytes = estimate_shard_resident_bytes(&model.tensor_sizes, tail, p);
        // 2×100M + tok_embd 50M + output_norm 1k + output 50M + headroom.
        assert_eq!(
            tail_bytes,
            200_000_000 + 50_000_000 + 1_000 + 50_000_000 + VRAM_BACKEND_OVERHEAD_BYTES
        );

        // Middle shard [2,4): 2 layers + tok_embd ONLY (no output/norm).
        let mid = ShardWindow::new(2, 4, 6).unwrap();
        let mid_bytes = estimate_shard_resident_bytes(&model.tensor_sizes, mid, p);
        assert_eq!(
            mid_bytes,
            200_000_000 + 50_000_000 + VRAM_BACKEND_OVERHEAD_BYTES,
            "a mid shard keeps tok_embd but NOT output_norm/lm_head"
        );
        assert!(
            tail_bytes > mid_bytes,
            "the last shard must estimate more than a same-width mid shard"
        );
    }

    #[test]
    fn estimate_includes_kv_cache_term() {
        let model = facts(4);
        let w = ShardWindow::new(0, 1, 4).unwrap();
        let no_kv = estimate_shard_resident_bytes(
            &model.tensor_sizes,
            w,
            ShardVramParams {
                n_ctx: 0,
                n_layers_in_shard: 1,
                n_kv_heads: 8,
                head_dim: 128,
            },
        );
        let with_kv = estimate_shard_resident_bytes(
            &model.tensor_sizes,
            w,
            ShardVramParams {
                n_ctx: 4096,
                n_layers_in_shard: 1,
                n_kv_heads: 8,
                head_dim: 128,
            },
        );
        // KV = 2 (K+V) · n_layers(1) · n_ctx(4096) · n_kv_heads(8) · head_dim(128) · dtype(2).
        assert_eq!(with_kv - no_kv, 2 * 4096 * 8 * 128 * 2);
    }

    #[test]
    fn authorize_rejects_unsigned_before_anything() {
        let initiator = KeyPair::generate();
        let worker = KeyPair::generate();
        let group = group_with(&initiator, &[&worker.public_bytes()]);
        let mut entry = signed_manifest(&initiator, &worker.public_bytes(), 0, 4);
        // Tamper after signing → signature no longer verifies.
        entry.manifest.revision += 1;
        let got = authorize_claim(&entry, &group, &worker.public_bytes());
        assert!(matches!(got, Err(ClaimRejection::BadManifestSignature(_))));
    }

    #[test]
    fn authorize_rejects_non_member() {
        let initiator = KeyPair::generate();
        let worker = KeyPair::generate();
        // Group does NOT include the worker.
        let group = group_with(&initiator, &[]);
        let entry = signed_manifest(&initiator, &worker.public_bytes(), 0, 4);
        let got = authorize_claim(&entry, &group, &worker.public_bytes());
        assert_eq!(got.unwrap_err(), ClaimRejection::NotAMember);
    }

    #[test]
    fn authorize_rejects_worker_absent_from_plan() {
        let initiator = KeyPair::generate();
        let planned = KeyPair::generate();
        let me = KeyPair::generate();
        // The plan assigns `planned`, but `me` (a member) is not in it.
        let group = group_with(&initiator, &[&me.public_bytes()]);
        let entry = signed_manifest(&initiator, &planned.public_bytes(), 0, 4);
        let got = authorize_claim(&entry, &group, &me.public_bytes());
        assert_eq!(got.unwrap_err(), ClaimRejection::NotInPlan);
    }

    #[test]
    fn assess_rejects_window_past_model() {
        let initiator = KeyPair::generate();
        let worker = KeyPair::generate();
        // Assignment claims layers [0,99) but the model has only 6.
        let a = assignment_for(&worker.public_bytes(), 0, 99);
        let _ = &initiator;
        let got = assess_capacity(&a, &facts(6), 512, u64::MAX);
        assert!(matches!(
            got,
            Err(ClaimRejection::WindowOutOfRange { n_layer: 6, .. })
        ));
    }

    #[test]
    fn shard_assignment_claim_respects_group() {
        // The budgeted F2 hermetic test: a member with an in-plan assignment is
        // accepted iff its estimated VRAM fits free VRAM; a non-member is
        // refused; an over-budget claim is refused. Mirrors the runtime order:
        // authorize_claim (crypto) THEN assess_capacity (capacity).
        let initiator = KeyPair::generate();
        let worker = KeyPair::generate();
        let wpk = worker.public_bytes();
        let model = facts(6);

        // Member, in plan, signed.
        let group = group_with(&initiator, &[&wpk]);
        let entry = signed_manifest(&initiator, &wpk, 0, 2);
        let assignment = authorize_claim(&entry, &group, &wpk).expect("member in plan authorised");

        // Plenty of free VRAM → accept.
        let accept = assess_capacity(assignment, &model, 512, 64 * 1024 * 1024 * 1024)
            .expect("fits a 64 GiB budget");
        assert_eq!((accept.layer_start, accept.layer_end), (0, 2));
        assert!(accept.is_first && !accept.is_last);
        assert!(accept.required_vram_bytes >= VRAM_BACKEND_OVERHEAD_BYTES);

        // Tiny free VRAM → refuse (fail-closed), no crash.
        let refused = assess_capacity(assignment, &model, 512, 1_000);
        assert!(matches!(
            refused,
            Err(ClaimRejection::InsufficientVram {
                free_bytes: 1_000,
                ..
            })
        ));

        // A non-member with the same (signed) manifest is refused at phase 1,
        // before any capacity / GGUF work.
        let outsider = KeyPair::generate();
        let outsider_group = group_with(&initiator, &[&wpk]); // outsider absent
        assert_eq!(
            authorize_claim(&entry, &outsider_group, &outsider.public_bytes()).unwrap_err(),
            ClaimRejection::NotAMember
        );
    }
}

// Integration tests of the feature-gated GGUF header read against a real model.
// Gated behind the feature AND `#[ignore]`d: CI never builds `llm_llama_cpp`,
// so these run locally with the model on disk via
//   SBFB_SHARD_TEST_GGUF=/path/to/llama-arch-q4.gguf \
//     cargo test -p nexus-worker-core --features llm_llama_cpp -- --ignored read_gguf
#[cfg(all(test, feature = "llm_llama_cpp"))]
mod gguf_tests {
    use super::*;
    use crate::llm::shard::ShardWindow;

    fn gguf_path() -> Option<std::path::PathBuf> {
        std::env::var_os("SBFB_SHARD_TEST_GGUF").map(std::path::PathBuf::from)
    }

    /// The extended `GgufContext` FFI accessors + `read_gguf_model_facts` pull a
    /// coherent llama geometry (arch, layer/embd/head counts, a non-empty tensor
    /// table with `token_embd` resident) from a real GGUF header.
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn read_gguf_model_facts_extracts_llama_geometry() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        let facts = read_gguf_model_facts(&path).expect("read llama GGUF facts");
        assert_eq!(facts.architecture, "llama");
        assert!(facts.n_layer > 0, "n_layer must be positive");
        assert!(facts.n_embd > 0);
        assert!(facts.n_head > 0);
        assert!(facts.n_head_kv > 0);
        assert!(!facts.tensor_sizes.is_empty(), "tensor table non-empty");
        assert!(
            facts
                .tensor_sizes
                .iter()
                .any(|(n, s)| n == GGUF_TENSOR_TOKEN_EMBD && *s > 0),
            "token_embd.weight present with a non-zero size"
        );
    }

    /// A real-GGUF VRAM estimate: a half-model shard sums real per-tensor sizes
    /// (via the ggml-computed `gguf_get_tensor_size`), exceeds the fixed backend
    /// headroom, and is strictly below the whole-model estimate.
    #[test]
    #[ignore = "requires SBFB_SHARD_TEST_GGUF (llama-arch GGUF on disk)"]
    fn estimate_shard_subset_is_bounded_by_whole_model() {
        let Some(path) = gguf_path() else {
            eprintln!("SBFB_SHARD_TEST_GGUF unset — skipping");
            return;
        };
        let facts = read_gguf_model_facts(&path).expect("facts");
        let n_layer = facts.n_layer;
        assert!(n_layer >= 2, "need >= 2 layers to split");
        let head_dim = facts.n_embd.checked_div(facts.n_head).unwrap_or(0);
        let estimate = |w: ShardWindow| {
            estimate_shard_resident_bytes(
                &facts.tensor_sizes,
                w,
                ShardVramParams {
                    n_ctx: 512,
                    n_layers_in_shard: w.n_layers(),
                    n_kv_heads: facts.n_head_kv,
                    head_dim,
                },
            )
        };
        let half = n_layer / 2;
        let head = estimate(ShardWindow::new(0, half, n_layer).expect("head window"));
        let whole = estimate(ShardWindow::new(0, n_layer, n_layer).expect("whole window"));
        assert!(
            head > VRAM_BACKEND_OVERHEAD_BYTES,
            "a real shard adds layer weights over the fixed headroom"
        );
        assert!(
            head < whole,
            "a half-model shard must estimate strictly less than the whole model"
        );
    }
}
