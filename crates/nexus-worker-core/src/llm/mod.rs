// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase D — dual-backend LLM abstraction.
//!
//! The worker speaks to an LLM via the [`LlmBackend`] trait rather
//! than a concrete client. Two implementations ship today :
//!
//! - [`OllamaBackend`][ollama::OllamaBackend] — the Sprint 0
//!   baseline. Talks HTTP to the Ollama daemon (`localhost:11434`
//!   by default). Zero build dependency on cmake, NASM, or a GPU
//!   toolchain — any developer can `cargo build --no-default-features
//!   --features llm_ollama` and get a working worker against their
//!   local Ollama install.
//! - [`LlamaCppBackend`][llama_cpp::LlamaCppBackend] — the Sprint 20
//!   production-default. Embeds the `llama-cpp-2` Rust binding +
//!   `llguidance` constrained-decoding engine directly in the worker
//!   process. Gated behind the `llm_llama_cpp` Cargo feature so
//!   vanilla `cargo build` stays cmake-free.
//!
//! Worker config (`worker.toml`) selects the backend via the
//! `[llm] backend = "ollama" | "llama_cpp"` key — see
//! [`crate::config::LlmConfig`].
//!
//! ## Why two backends
//!
//! Ollama alone gives us JSON Schema enforcement via the v0.5+
//! `format` parameter (Ollama's internal llama.cpp runs the GBNF
//! grammar), which is enough to secure the signature chain
//! (invalid JSON → signature refuse). But everything upstream of
//! sample time (tool-call interception for S22 sandbox,
//! process-boundary VRAM wipe for S23 ephemeral workers, PQC
//! task-response signing inline with sampling for S26) requires
//! direct control over the LLM process — impossible across the
//! Ollama HTTP boundary.
//!
//! The `llama_cpp` backend is therefore an architectural unlock,
//! not just a performance win. See the design doc
//! `.planning/research/S20_phase_D_structured_output_design.md`
//! §7 for the full threat-model alignment argument.
//!
//! ## Grammar ≠ prompt-injection defense
//!
//! **Reminder**: the JSON Schema enforcement both backends perform
//! constrains the *format* of the output, not its *content*. A
//! successful prompt-injection against the user query can still
//! produce schema-valid output with a malicious payload. Defense
//! against prompt injection is a separate layer : Sprint 21
//! client-side redaction + Sprint 22 tool-calling sandbox. See
//! `docs/rust/PATTERNS.md §P30` for the longer form warning.

pub mod factory;
#[cfg(feature = "llm_llama_cpp")]
pub mod llama_cpp;
pub mod ollama;
pub mod schema_bridge;
pub mod watermark;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-exports so downstream code can write `use
// nexus_worker_core::llm::{LlmBackend, GenerateParams, ...}` without
// walking the submodule tree.
pub use factory::{FactoryError, build_backend};
pub use ollama::{OllamaBackend, StubBackend};

// =================================================================
// Unified error surface
// =================================================================

/// Every failure an [`LlmBackend`] can surface to the engine.
///
/// Backends wrap their native error types into this enum so the
/// engine loop has a single `match` to maintain. The variants match
/// the operational categories the worker cares about :
///
/// - `NotRunning` → the engine signals the TUI / shell to show an
///   install or start hint.
/// - `Api` → the backend accepted the request but the model /
///   runtime returned an error. Task is marked failed, retry logic
///   upstream.
/// - `SchemaViolation` → the model produced a byte sequence that
///   looked like JSON but failed the defensive validator. The
///   worker refuses to sign and returns the response rejected.
/// - `UnsupportedBackend` → `worker.toml` requested a backend that
///   wasn't compiled into this binary. Loud, fail-fast.
/// - `InvalidConfig` → the backend's config section fails its own
///   validation (bad URL, missing file, ...).
/// - `RetriesExhausted` → all backoff attempts failed.
#[derive(Debug, Error)]
pub enum LlmBackendError {
    /// The backend runtime is not reachable.
    #[error("llm backend not running at {endpoint}: {reason}\nhint: {hint}")]
    NotRunning {
        endpoint: String,
        reason: String,
        hint: &'static str,
    },

    /// Config validation failed (bad URL, missing file, etc.).
    #[error("invalid llm config: {reason}")]
    InvalidConfig { reason: String },

    /// Backend accepted the request but returned an error.
    #[error("llm api error: {0}")]
    Api(String),

    /// The LLM output parsed as JSON but failed the defensive
    /// validator against the expected schema identity (wrong
    /// version / domain) or failed to deserialize as the
    /// `TaskResponse` type.
    #[error("llm output violates schema: {0}")]
    SchemaViolation(String),

    /// All retry attempts failed. Carries the last-seen error.
    #[error("llm call failed after {attempts} attempts: {last_error}")]
    RetriesExhausted { attempts: u32, last_error: String },

    /// `worker.toml` requested a backend that was not compiled
    /// into this binary. Operator must rebuild with the matching
    /// feature flag.
    #[error(
        "unsupported llm backend {requested:?}: feature {feature:?} is not compiled in this binary\nhint: rebuild with `cargo build --features {feature}` or set `[llm] backend = \"ollama\"` in worker.toml"
    )]
    UnsupportedBackend {
        requested: &'static str,
        feature: &'static str,
    },
}

pub type LlmBackendResult<T> = std::result::Result<T, LlmBackendError>;

// =================================================================
// Healthcheck outcome
// =================================================================

/// Result of a [`LlmBackend::healthcheck`] call.
///
/// Structurally identical to the pre-Sprint 20 `ollama::HealthCheck`
/// enum — renamed and moved here so every backend shares one view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheck {
    /// Backend is reachable. `models` holds the list of available
    /// model tags the backend can serve. Empty for
    /// [`LlamaCppBackend`] if the configured GGUF is missing.
    Ready { models: Vec<String> },
    /// Backend unreachable (daemon down, socket refused, GGUF
    /// file missing). Carries an install / bootstrap hint so
    /// the TUI can echo it verbatim.
    NotRunning {
        endpoint: String,
        reason: String,
        hint: &'static str,
    },
    /// Backend is reachable but returned an error.
    Error { endpoint: String, reason: String },
}

impl HealthCheck {
    /// True iff the backend is up and reachable.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

// =================================================================
// Request / response
// =================================================================

/// Parameters for a single generate call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateParams {
    /// Model tag (e.g. `llama3:8b`, `qwen2.5-7b-instruct-q4_k_m`).
    /// For [`LlamaCppBackend`] this is the logical tag used to
    /// pick the GGUF path from the config, not the file path
    /// itself.
    pub model: String,
    /// User prompt.
    pub prompt: String,
    /// Optional system prompt prepended before the user prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Optional sampling temperature. `None` uses the backend's
    /// default (usually the model's training default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional JSON Schema the response must satisfy. Sprint 20
    /// only ships one schema identity (`TaskResponse`) — the
    /// backends resolve the value through
    /// [`schema_bridge::task_response_schema_value`] so a future
    /// sprint that introduces per-task schemas can switch on a
    /// marker inside the value without touching the trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub watermark_seed: Vec<u8>,
    #[serde(default)]
    pub watermark_enabled: bool,
    #[serde(default)]
    pub watermark_delta: f32,
    #[serde(default)]
    pub watermark_window_size: usize,
}

impl GenerateParams {
    /// Minimal params for `model` + `prompt`.
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            system: None,
            temperature: None,
            schema: None,
            watermark_seed: Vec::new(),
            watermark_enabled: false,
            watermark_delta: 0.0,
            watermark_window_size: 0,
        }
    }

    /// Attach a system prompt.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Attach a sampling temperature.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Attach a JSON Schema. Both backends will constrain their
    /// sampler to emit schema-matching output.
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Attach watermark injection parameters.
    pub fn with_watermark(
        mut self,
        enabled: bool,
        seed: Vec<u8>,
        delta: f32,
        window_size: usize,
    ) -> Self {
        self.watermark_enabled = enabled;
        self.watermark_seed = seed;
        self.watermark_delta = delta;
        self.watermark_window_size = window_size;
        self
    }
}

/// Response from [`LlmBackend::generate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Raw text the model produced. When `schema` was set on the
    /// request, this is guaranteed to parse as the configured
    /// schema identity (defensive validator runs before return).
    pub text: String,
    /// Model tag the backend actually ran (ollama may rewrite
    /// tags ; llama-cpp echoes the configured path's stem).
    pub model: String,
    /// Prompt token count if the backend reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    /// Completion token count if the backend reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
    /// Output token IDs for watermark z-test detection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_token_ids: Vec<u32>,
}

// =================================================================
// Trait
// =================================================================

/// Implementation-agnostic surface the engine calls into.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Probe the backend runtime and return its state plus the
    /// list of available models.
    async fn healthcheck(&self) -> HealthCheck;

    /// Run a text-generation request and return the full response.
    /// When `params.schema` is set, the returned `text` is
    /// guaranteed to deserialize against the schema identity
    /// (Sprint 20 ships only [`nexus_core_rs::TaskResponse`]).
    async fn generate(&self, params: GenerateParams) -> LlmBackendResult<GenerateResponse>;
}

// =================================================================
// Shared retry helper
// =================================================================

/// Run `op` with exponential backoff, returning the last error
/// wrapped in [`LlmBackendError::RetriesExhausted`] once
/// `max_attempts` runs have failed. Exposed to submodules so both
/// the Ollama and the llama.cpp backends share one retry policy.
pub(crate) async fn retry_with_backoff<T, F, Fut>(
    max_attempts: u32,
    mut op: F,
) -> LlmBackendResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = LlmBackendResult<T>>,
{
    let attempts = max_attempts.max(1);
    let mut last_err: Option<LlmBackendError> = None;
    for attempt in 1..=attempts {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt >= attempts {
                    last_err = Some(e);
                    break;
                }
                tracing::warn!(
                    attempt,
                    total = attempts,
                    error = %e,
                    "llm call failed, retrying after backoff"
                );
                let delay = std::time::Duration::from_millis(100 * (1u64 << (attempt - 1)))
                    .min(std::time::Duration::from_secs(5));
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(LlmBackendError::RetriesExhausted {
        attempts,
        last_error: last_err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    })
}

// =================================================================
// Tests — trait-level behavioural checks via StubBackend
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn generate_params_builder_sets_fields() {
        let p = GenerateParams::new("llama3:8b", "why is the sky blue?")
            .with_system("You are an astronomer.")
            .with_temperature(0.2)
            .with_schema(serde_json::json!({"type": "object"}));
        assert_eq!(p.model, "llama3:8b");
        assert_eq!(p.prompt, "why is the sky blue?");
        assert_eq!(p.system.as_deref(), Some("You are an astronomer."));
        assert_eq!(p.temperature, Some(0.2));
        assert!(p.schema.is_some());
    }

    #[test]
    fn generate_params_watermark_builder_sets_fields() {
        let p = GenerateParams::new("m", "p").with_watermark(
            true,
            b"seed-32-bytes-exactly-here!12345".to_vec(),
            2.5,
            4,
        );
        assert!(p.watermark_enabled);
        assert_eq!(p.watermark_seed.len(), 32);
        assert!((p.watermark_delta - 2.5).abs() < f32::EPSILON);
        assert_eq!(p.watermark_window_size, 4);
    }

    #[test]
    fn generate_params_schema_roundtrip_through_serde() {
        let p = GenerateParams::new("m", "p").with_schema(serde_json::json!({"foo": 42}));
        let wire = serde_json::to_string(&p).unwrap();
        let back: GenerateParams = serde_json::from_str(&wire).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn generate_response_output_token_ids_serde() {
        let r = GenerateResponse {
            text: "hello".into(),
            model: "m".into(),
            prompt_tokens: Some(5),
            completion_tokens: Some(3),
            output_token_ids: vec![10, 20, 30],
        };
        let wire = serde_json::to_string(&r).unwrap();
        assert!(wire.contains("output_token_ids"));
        let back: GenerateResponse = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.output_token_ids, vec![10, 20, 30]);

        let r_empty = GenerateResponse {
            output_token_ids: vec![],
            ..r.clone()
        };
        let wire_empty = serde_json::to_string(&r_empty).unwrap();
        assert!(
            !wire_empty.contains("output_token_ids"),
            "empty vec should be skipped in serialization"
        );
    }

    #[test]
    fn healthcheck_is_ready_helper() {
        assert!(
            HealthCheck::Ready {
                models: vec!["m".into()],
            }
            .is_ready()
        );
        assert!(
            !HealthCheck::NotRunning {
                endpoint: "e".into(),
                reason: "r".into(),
                hint: "h",
            }
            .is_ready()
        );
        assert!(
            !HealthCheck::Error {
                endpoint: "e".into(),
                reason: "r".into(),
            }
            .is_ready()
        );
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn retry_with_backoff_returns_first_success() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_with_backoff(5, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, LlmBackendError>(42)
            }
        })
        .await
        .unwrap();
        assert_eq!(result, 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn retry_with_backoff_recovers_on_second_attempt() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: LlmBackendResult<i32> = retry_with_backoff(5, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(LlmBackendError::Api("flake".into()))
                } else {
                    Ok(7)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 7);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn retry_with_backoff_returns_retries_exhausted_after_all_attempts() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: LlmBackendResult<i32> = retry_with_backoff(3, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(LlmBackendError::Api("permanent".into()))
            }
        })
        .await;
        match result {
            Err(LlmBackendError::RetriesExhausted {
                attempts,
                last_error,
            }) => {
                assert_eq!(attempts, 3);
                assert!(last_error.contains("permanent"));
            }
            other => panic!("expected RetriesExhausted, got {other:?}"),
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
}
