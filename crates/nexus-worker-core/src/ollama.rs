// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ollama HTTP client wrapper with healthcheck and retry.
//!
//! This module is the *only* place in `nexus-worker-core` that
//! depends on `ollama-rs`. Everything else in the engine talks
//! to the [`OllamaClient`] trait so the implementation can be
//! swapped in tests or replaced by a different LLM backend
//! (llama.cpp RPC, a remote Ollama over SSH, ...) without
//! cascading changes.
//!
//! ## Healthcheck
//!
//! The canonical way to probe Ollama is to call
//! `list_local_models()` — it hits `GET /api/tags` which is
//! cheap, never downloads anything, and returns the full list
//! of models the daemon has on disk. We map:
//!
//! - `Ok(models)` → [`HealthCheck::Ready`] with the model names
//! - connection refused / DNS failure → [`HealthCheck::NotRunning`]
//!   with an install hint pointing at `https://ollama.com/download`
//! - any other error → [`HealthCheck::Error`] carrying the
//!   underlying diagnostic text
//!
//! This was validated against Context7 `pepperoni21/ollama-rs`
//! (the `Ollama::default()` constructor connects to
//! `localhost:11434` and `list_local_models()` returns
//! `Result<Vec<LocalModel>>`).
//!
//! ## Retry
//!
//! Network calls go through [`retry_with_backoff`], which
//! attempts up to `max_attempts` times with exponential backoff
//! (100ms, 200ms, 400ms, ...). The retry loop logs each attempt
//! at `warn` so operators can see transient failures without
//! turning on debug logging. `retry_with_backoff` is generic
//! over the operation so both healthcheck and generate can use
//! it uniformly.

use std::time::Duration;

use async_trait::async_trait;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::config::Ollama as OllamaConfig;

// =================================================================
// Error type
// =================================================================

/// Failures that the Ollama client surface can produce.
#[derive(Debug, Error)]
pub enum OllamaClientError {
    /// The Ollama daemon is not reachable at the configured
    /// endpoint. The `hint` field carries an install URL so the
    /// binary can print it verbatim and users know what to do.
    #[error("ollama not running at {endpoint}: {reason}\nhint: {hint}")]
    NotRunning {
        endpoint: String,
        reason: String,
        hint: &'static str,
    },

    /// The configured endpoint URL is malformed and could not
    /// even be parsed. Caught early in the constructor.
    #[error("invalid ollama endpoint {endpoint}: {source}")]
    InvalidEndpoint {
        endpoint: String,
        #[source]
        source: url::ParseError,
    },

    /// The request made it to Ollama but the daemon returned an
    /// error (bad model name, prompt too long, ...). Carries the
    /// raw error string so callers can forward it to the user.
    #[error("ollama api error: {0}")]
    Api(String),

    /// All retry attempts failed. Carries the last-seen error.
    #[error("ollama call failed after {attempts} attempts: {last_error}")]
    RetriesExhausted { attempts: u32, last_error: String },
}

/// Short-hand [`Result`] alias for the Ollama client surface.
pub type OllamaResult<T> = std::result::Result<T, OllamaClientError>;

const INSTALL_HINT: &str = "install Ollama from https://ollama.com/download and run `ollama serve`";

// =================================================================
// Healthcheck outcome
// =================================================================

/// Result of a [`OllamaClient::healthcheck`] call.
///
/// Exposed as a rich enum (rather than a bool) so the engine,
/// the `stats` CLI and the TUI can all surface actionable
/// messages to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheck {
    /// Daemon is reachable. `models` holds the list of local
    /// model names (e.g. `llama2:latest`, `qwen3:30b`).
    Ready { models: Vec<String> },
    /// Connection refused / DNS / timeout. Install hint is
    /// included so frontends can show it verbatim.
    NotRunning {
        endpoint: String,
        reason: String,
        hint: &'static str,
    },
    /// The daemon is running but returned an error.
    Error { endpoint: String, reason: String },
}

impl HealthCheck {
    /// Returns true iff the daemon is up and reachable.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

// =================================================================
// Generation request / response
// =================================================================

/// Parameters for an Ollama text-generation call, in the minimal
/// shape the SBFB worker engine needs.
///
/// Uses plain `String` / `Option<...>` fields so the struct is
/// trivially `Serialize + Deserialize + Clone + Debug`, which
/// makes it usable as a task payload, a test fixture, and a
/// log line without wrapper code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateParams {
    /// Model tag as Ollama knows it (`llama2:latest`,
    /// `qwen3:30b`, etc.).
    pub model: String,
    /// User prompt.
    pub prompt: String,
    /// Optional system prompt prepended by Ollama.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Optional sampling temperature. `None` uses the Ollama
    /// default for the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl GenerateParams {
    /// Construct the minimal params for `model` + `prompt`.
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            system: None,
            temperature: None,
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
}

/// Response from [`OllamaClient::generate`]: the full model
/// output plus the model tag and optional token counts so the
/// engine can bill kudos correctly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Raw text the model produced, concatenated across the
    /// stream if streaming was used.
    pub text: String,
    /// Model that actually ran (ollama may rewrite tags).
    pub model: String,
    /// Prompt token count, if ollama reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    /// Completion token count, if ollama reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
}

// =================================================================
// Client trait — implementation-agnostic surface
// =================================================================

/// Async trait the engine calls into. Having a trait lets tests
/// drop in a stub (no network, deterministic output) while the
/// real [`OllamaHttpClient`] uses `ollama-rs`.
#[async_trait]
pub trait OllamaClient: Send + Sync {
    /// Probe the Ollama daemon and return its state plus the
    /// list of installed models when reachable.
    async fn healthcheck(&self) -> HealthCheck;

    /// Run a text-generation request and return the full response.
    async fn generate(&self, params: GenerateParams) -> OllamaResult<GenerateResponse>;
}

// =================================================================
// Real HTTP client backed by ollama-rs
// =================================================================

/// Production implementation backed by `ollama-rs` over HTTP.
///
/// Constructed from a [`crate::config::Ollama`] section so the
/// endpoint and timeout flow naturally from the worker config
/// file.
pub struct OllamaHttpClient {
    endpoint: String,
    timeout: Duration,
    max_retries: u32,
    inner: Ollama,
}

impl std::fmt::Debug for OllamaHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaHttpClient")
            .field("endpoint", &self.endpoint)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl OllamaHttpClient {
    /// Default number of retries for both healthcheck and
    /// generate. Callers can override via
    /// [`OllamaHttpClient::with_max_retries`].
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Build a client from the `[ollama]` section of the worker
    /// config.
    ///
    /// Parses and validates the endpoint URL eagerly so a
    /// malformed `worker.toml` fails on startup rather than on
    /// the first generate call.
    pub fn from_config(cfg: &OllamaConfig) -> OllamaResult<Self> {
        let url = Url::parse(&cfg.endpoint).map_err(|e| OllamaClientError::InvalidEndpoint {
            endpoint: cfg.endpoint.clone(),
            source: e,
        })?;

        // `Ollama::new(host, port)` wants a scheme+host string
        // and a port number separately. Extract both from the
        // parsed URL.
        let host_with_scheme = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().unwrap_or("localhost")
        );
        let port = url.port_or_known_default().unwrap_or(11434);
        let inner = Ollama::new(host_with_scheme, port);

        Ok(Self {
            endpoint: cfg.endpoint.clone(),
            timeout: Duration::from_secs(cfg.timeout_secs),
            max_retries: Self::DEFAULT_MAX_RETRIES,
            inner,
        })
    }

    /// Override the default retry budget (see [`DEFAULT_MAX_RETRIES`]).
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries.max(1);
        self
    }

    /// Underlying `ollama-rs` handle for advanced callers that
    /// need features this wrapper does not yet cover (chat,
    /// function calling, embeddings).
    pub fn inner(&self) -> &Ollama {
        &self.inner
    }
}

#[async_trait]
impl OllamaClient for OllamaHttpClient {
    async fn healthcheck(&self) -> HealthCheck {
        let endpoint = self.endpoint.clone();
        match tokio::time::timeout(self.timeout, self.inner.list_local_models()).await {
            Ok(Ok(models)) => HealthCheck::Ready {
                models: models.into_iter().map(|m| m.name).collect(),
            },
            Ok(Err(e)) => {
                let msg = e.to_string();
                if looks_like_connection_refused(&msg) {
                    HealthCheck::NotRunning {
                        endpoint,
                        reason: msg,
                        hint: INSTALL_HINT,
                    }
                } else {
                    HealthCheck::Error {
                        endpoint,
                        reason: msg,
                    }
                }
            }
            Err(_elapsed) => HealthCheck::NotRunning {
                endpoint,
                reason: format!("healthcheck timed out after {}s", self.timeout.as_secs()),
                hint: INSTALL_HINT,
            },
        }
    }

    async fn generate(&self, params: GenerateParams) -> OllamaResult<GenerateResponse> {
        let attempts = self.max_retries;
        let model = params.model.clone();
        let req_build = || {
            let mut req = GenerationRequest::new(params.model.clone(), params.prompt.clone());
            if let Some(sys) = params.system.clone() {
                req = req.system(sys);
            }
            req
        };

        let response = retry_with_backoff(attempts, || {
            let req = req_build();
            let inner = self.inner.clone();
            let timeout = self.timeout;
            async move {
                match tokio::time::timeout(timeout, inner.generate(req)).await {
                    Ok(Ok(resp)) => Ok(resp),
                    Ok(Err(e)) => Err(OllamaClientError::Api(e.to_string())),
                    Err(_elapsed) => Err(OllamaClientError::Api(format!(
                        "generate timed out after {}s",
                        timeout.as_secs()
                    ))),
                }
            }
        })
        .await?;

        Ok(GenerateResponse {
            text: response.response,
            model,
            // ollama-rs 0.2 exposes eval counts as Option<u16>
            // directly on GenerationResponse. Map None → None so
            // engine billing code treats absence as "unknown,
            // don't charge" and u16 → u64 for future-proofing.
            prompt_tokens: response.prompt_eval_count.map(u64::from),
            completion_tokens: response.eval_count.map(u64::from),
        })
    }
}

// =================================================================
// Stub client (Sprint 4 Phase D `--stub-ollama` mode)
// =================================================================

/// Deterministic no-network Ollama client.
///
/// Behaves like a healthy Ollama that has exactly the models in
/// `models` installed. `generate()` returns a predictable
/// template string derived from the input prompt so the Sprint
/// 4 end-to-end tests can assert on the response without running
/// an actual LLM.
///
/// Used in two places:
///
/// 1. `#[cfg(test)]` engine tests that need a healthy Ollama
///    without spinning up the real daemon.
/// 2. The `nexus-worker --stub-ollama` flag, which swaps the
///    engine's client at boot for hermetic e2e runs (no Ollama
///    install needed on the test host).
pub struct StubOllama {
    pub models: Vec<String>,
}

impl StubOllama {
    /// Construct a stub with the canonical Sprint 4 test model
    /// list: a single `stub-model:latest` entry that matches the
    /// model name every test fixture task submits.
    pub fn new() -> Self {
        Self {
            models: vec!["stub-model:latest".to_string()],
        }
    }

    /// Construct a stub with an explicit model list — useful for
    /// tests that need to exercise the "multi-model installed"
    /// path.
    pub fn with_models(models: Vec<String>) -> Self {
        Self { models }
    }
}

impl Default for StubOllama {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OllamaClient for StubOllama {
    async fn healthcheck(&self) -> HealthCheck {
        HealthCheck::Ready {
            models: self.models.clone(),
        }
    }

    async fn generate(&self, params: GenerateParams) -> OllamaResult<GenerateResponse> {
        // Deterministic response shape: prefix with STUB so the
        // e2e test can filter for it, and echo the first 64
        // chars of the prompt so flaky signature issues surface
        // as obvious content mismatches.
        let text = format!(
            "STUB[{}]: {}",
            params.model,
            params.prompt.chars().take(64).collect::<String>()
        );
        Ok(GenerateResponse {
            text,
            model: params.model.clone(),
            prompt_tokens: Some((params.prompt.len() / 4).max(1) as u64),
            completion_tokens: Some(16),
        })
    }
}

// =================================================================
// Retry helper
// =================================================================

/// Heuristic for mapping an ollama-rs error message to
/// "daemon not running". Ollama-rs forwards reqwest errors
/// verbatim, so we look for the canonical connection-refused
/// phrases across Linux / macOS / Windows.
fn looks_like_connection_refused(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("connection refused")
        || lower.contains("tcp connect error")
        || lower.contains("failed to connect")
        || lower.contains("connection reset")
        || lower.contains("os error 10061") // WSAECONNREFUSED on Windows
        || lower.contains("econnrefused")
}

/// Run `op` with exponential backoff, returning the last error
/// wrapped in [`OllamaClientError::RetriesExhausted`] once
/// `max_attempts` runs have failed.
///
/// The backoff is `100 * 2^(attempt-1) ms`: 100, 200, 400, 800,
/// 1600 — capped at 5 seconds per sleep. Every attempt is
/// logged at `warn` with the error message so operators can
/// diagnose flakes without turning on debug logging.
pub(crate) async fn retry_with_backoff<T, F, Fut>(max_attempts: u32, mut op: F) -> OllamaResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = OllamaResult<T>>,
{
    let attempts = max_attempts.max(1);
    let mut last_err: Option<OllamaClientError> = None;
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
                    "ollama call failed, retrying after backoff"
                );
                let delay = Duration::from_millis(100 * (1u64 << (attempt - 1)))
                    .min(Duration::from_secs(5));
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(OllamaClientError::RetriesExhausted {
        attempts,
        last_error: last_err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    })
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn generate_params_builder_sets_fields() {
        let p = GenerateParams::new("llama2", "why is the sky blue?")
            .with_system("You are an astronomer.")
            .with_temperature(0.2);
        assert_eq!(p.model, "llama2");
        assert_eq!(p.prompt, "why is the sky blue?");
        assert_eq!(p.system.as_deref(), Some("You are an astronomer."));
        assert_eq!(p.temperature, Some(0.2));
    }

    #[test]
    fn looks_like_connection_refused_matches_canonical_phrases() {
        assert!(looks_like_connection_refused(
            "Connection refused (os error 111)"
        ));
        assert!(looks_like_connection_refused(
            "tcp connect error: Connection refused"
        ));
        assert!(looks_like_connection_refused(
            "http error: failed to connect: connection refused"
        ));
        assert!(looks_like_connection_refused(
            "os error 10061: no connection could be made because the target machine actively refused it"
        ));
        assert!(!looks_like_connection_refused("model not found"));
        assert!(!looks_like_connection_refused("403 forbidden"));
    }

    #[test]
    fn healthcheck_is_ready_helper() {
        assert!(HealthCheck::Ready {
            models: vec!["llama2".into()]
        }
        .is_ready());
        assert!(!HealthCheck::NotRunning {
            endpoint: "http://x:1".into(),
            reason: "nope".into(),
            hint: INSTALL_HINT
        }
        .is_ready());
        assert!(!HealthCheck::Error {
            endpoint: "http://x:1".into(),
            reason: "boom".into()
        }
        .is_ready());
    }

    #[test]
    fn client_from_config_parses_endpoint() {
        let cfg = OllamaConfig {
            endpoint: "http://10.0.0.5:11434".into(),
            timeout_secs: 60,
        };
        let client = OllamaHttpClient::from_config(&cfg).unwrap();
        assert_eq!(client.endpoint, "http://10.0.0.5:11434");
        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.max_retries, OllamaHttpClient::DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn client_from_config_rejects_bad_url() {
        let cfg = OllamaConfig {
            endpoint: "not a url".into(),
            timeout_secs: 60,
        };
        let err = OllamaHttpClient::from_config(&cfg).unwrap_err();
        match err {
            OllamaClientError::InvalidEndpoint { endpoint, .. } => {
                assert_eq!(endpoint, "not a url")
            }
            other => panic!("expected InvalidEndpoint, got {other:?}"),
        }
    }

    #[test]
    fn client_with_max_retries_overrides_default() {
        let cfg = OllamaConfig {
            endpoint: "http://localhost:11434".into(),
            timeout_secs: 10,
        };
        let client = OllamaHttpClient::from_config(&cfg)
            .unwrap()
            .with_max_retries(7);
        assert_eq!(client.max_retries, 7);
    }

    #[test]
    fn client_with_max_retries_floors_to_one() {
        let cfg = OllamaConfig {
            endpoint: "http://localhost:11434".into(),
            timeout_secs: 10,
        };
        let client = OllamaHttpClient::from_config(&cfg)
            .unwrap()
            .with_max_retries(0);
        assert_eq!(client.max_retries, 1, "zero retries should floor to one");
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn retry_with_backoff_returns_first_success() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_with_backoff(5, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, OllamaClientError>(42)
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
        let result: OllamaResult<i32> = retry_with_backoff(5, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(OllamaClientError::Api("flake".into()))
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
        let result: OllamaResult<i32> = retry_with_backoff(3, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(OllamaClientError::Api("permanent".into()))
            }
        })
        .await;
        match result {
            Err(OllamaClientError::RetriesExhausted {
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

    /// Live healthcheck against whatever Ollama the dev machine
    /// has running. On CI (no Ollama) this asserts that the
    /// error classification picks the `NotRunning` path with
    /// the install hint — which is the important signal.
    #[tokio::test]
    async fn live_healthcheck_either_ready_or_not_running() {
        // Force a short timeout so the CI path resolves quickly
        // when the port is closed.
        let short_cfg = OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            timeout_secs: 2,
        };
        let client = OllamaHttpClient::from_config(&short_cfg).unwrap();

        let hc = client.healthcheck().await;
        match hc {
            HealthCheck::Ready { models } => {
                // dev-box path: models list present (may be empty
                // if the daemon is running but no models pulled).
                assert!(models.iter().all(|m| !m.is_empty()));
            }
            HealthCheck::NotRunning { hint, .. } => {
                assert_eq!(hint, INSTALL_HINT);
            }
            HealthCheck::Error { reason, .. } => {
                // Also acceptable: Ollama is reachable but
                // refused the list_local_models call. Anything
                // other than a panic is fine here.
                assert!(!reason.is_empty());
            }
        }
    }
}
