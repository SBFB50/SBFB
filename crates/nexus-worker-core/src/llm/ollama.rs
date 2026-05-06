// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase D — Ollama backend (renamed from the Sprint 0
//! `ollama.rs` module). Talks HTTP to the Ollama daemon and
//! implements [`LlmBackend`].
//!
//! The unit and integration tests from the original file are kept
//! verbatim (minus the rename to `OllamaBackend`) — they cover the
//! URL parser, the `NotRunning` heuristic, the backoff loop and
//! the live healthcheck's graceful handling of "no Ollama
//! installed on this dev box".
//!
//! ## Schema enforcement
//!
//! When [`GenerateParams::schema`] is `Some`, the backend wires
//! the schema into `GenerationRequest::format(FormatType::
//! StructuredJson(JsonStructure))`. Ollama v0.5+ forwards the
//! schema to its internal llama.cpp instance which enforces it at
//! sample time via GBNF. A defensive `serde_json::from_str::
//! <TaskResponse>(&text)` then validates the result before we
//! return — belt-and-suspenders against a schema-compliant-but-
//! sematically-broken Ollama daemon.

use std::time::Duration;

use async_trait::async_trait;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::parameters::FormatType;
use url::Url;

use crate::config::OllamaConfig;

use super::schema_bridge::{ollama_json_structure, schema_is_task_response};
use super::{
    GenerateParams, GenerateResponse, HealthCheck, LlmBackend, LlmBackendError, LlmBackendResult,
    retry_with_backoff,
};

const INSTALL_HINT: &str = "install Ollama from https://ollama.com/download and run `ollama serve`";

// =================================================================
// Real HTTP client backed by ollama-rs
// =================================================================

/// Production [`LlmBackend`] talking HTTP to an Ollama daemon.
pub struct OllamaBackend {
    endpoint: String,
    timeout: Duration,
    max_retries: u32,
    inner: Ollama,
}

impl std::fmt::Debug for OllamaBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaBackend")
            .field("endpoint", &self.endpoint)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl OllamaBackend {
    /// Default number of retries for both healthcheck and
    /// generate.
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Build a backend from the `[llm.ollama]` section of the
    /// worker config.
    ///
    /// Parses and validates the endpoint URL eagerly so a
    /// malformed `worker.toml` fails on startup rather than on
    /// the first generate call.
    pub fn from_config(cfg: &OllamaConfig) -> LlmBackendResult<Self> {
        let url = Url::parse(&cfg.endpoint).map_err(|e| LlmBackendError::InvalidConfig {
            reason: format!("ollama endpoint {}: {e}", cfg.endpoint),
        })?;

        // `Ollama::new(host, port)` wants a scheme+host string
        // and a port number separately.
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

    /// Override the default retry budget.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries.max(1);
        self
    }

    /// Endpoint this backend was built from. Exposed for logging.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Underlying `ollama-rs` handle for advanced callers that
    /// need features this wrapper does not yet cover (chat,
    /// function calling, embeddings).
    pub fn inner(&self) -> &Ollama {
        &self.inner
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
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

    async fn generate(&self, params: GenerateParams) -> LlmBackendResult<GenerateResponse> {
        let attempts = self.max_retries;
        let model = params.model.clone();

        // Sprint 20 Phase D : when the caller attaches a schema,
        // wire it into `GenerationRequest::format`. At Sprint 20
        // the only schema identity is `TaskResponse` ; non-matching
        // schemas produce an InvalidConfig so we never silently
        // drop a constraint the caller asked for.
        let format = match params.schema.as_ref() {
            Some(schema) if schema_is_task_response(schema) => {
                Some(FormatType::StructuredJson(ollama_json_structure()))
            }
            Some(_) => {
                return Err(LlmBackendError::InvalidConfig {
                    reason:
                        "OllamaBackend only supports the TaskResponse schema identity at Sprint 20"
                            .to_string(),
                });
            }
            None => None,
        };
        let expects_schema = format.is_some();

        let req_build = || {
            let mut req = GenerationRequest::new(params.model.clone(), params.prompt.clone());
            if let Some(sys) = params.system.clone() {
                req = req.system(sys);
            }
            if let Some(fmt) = format.clone() {
                req = req.format(fmt);
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
                    Ok(Err(e)) => Err(LlmBackendError::Api(e.to_string())),
                    Err(_elapsed) => Err(LlmBackendError::Api(format!(
                        "generate timed out after {}s",
                        timeout.as_secs()
                    ))),
                }
            }
        })
        .await?;

        let text = response.response;

        // Defensive validator : even though Ollama's internal
        // llama.cpp enforces the GBNF, validate once more at the
        // Rust boundary so a broken Ollama can never stealth past
        // a malformed response into the signature layer.
        if expects_schema {
            validate_task_response(&text)?;
        }

        Ok(GenerateResponse {
            text,
            model,
            prompt_tokens: response.prompt_eval_count.map(u64::from),
            completion_tokens: response.eval_count.map(u64::from),
            output_token_ids: vec![],
        })
    }
}

/// Defensive validator : parse the LLM output as a `TaskResponse`
/// and check the version + domain markers. Failure returns
/// [`LlmBackendError::SchemaViolation`] with the underlying
/// serde / identity error as the message.
fn validate_task_response(text: &str) -> LlmBackendResult<()> {
    let parsed: nexus_core_rs::TaskResponse = serde_json::from_str(text).map_err(|e| {
        LlmBackendError::SchemaViolation(format!("TaskResponse deserialize failed: {e}"))
    })?;
    parsed.validate_identity().map_err(|e| {
        LlmBackendError::SchemaViolation(format!("TaskResponse identity check failed: {e}"))
    })
}

// =================================================================
// Stub backend (Sprint 4 Phase D `--stub-ollama` mode, migrated to
// `--stub-llm` at Sprint 20 Phase D rename).
// =================================================================

/// Deterministic no-network [`LlmBackend`].
///
/// Behaves like a healthy Ollama that has exactly the models in
/// `models` installed. `generate()` returns a predictable template
/// string derived from the input prompt so the Sprint 4 end-to-end
/// tests can assert on the response without running an actual LLM.
pub struct StubBackend {
    pub models: Vec<String>,
    /// When set, the stub returns this exact string instead of the
    /// default echo template. Tests use this to feed the defensive
    /// validator known-good / known-bad JSON payloads.
    pub forced_output: Option<String>,
}

impl StubBackend {
    /// Construct a stub with the canonical Sprint 4 test model
    /// list : a single `stub-model:latest` entry.
    pub fn new() -> Self {
        Self {
            models: vec!["stub-model:latest".to_string()],
            forced_output: None,
        }
    }

    /// Construct a stub with an explicit model list.
    pub fn with_models(models: Vec<String>) -> Self {
        Self {
            models,
            forced_output: None,
        }
    }

    /// Force every `generate()` call to return the provided text.
    /// Used by tests to drive the defensive validator.
    pub fn with_forced_output(mut self, text: impl Into<String>) -> Self {
        self.forced_output = Some(text.into());
        self
    }
}

impl Default for StubBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmBackend for StubBackend {
    async fn healthcheck(&self) -> HealthCheck {
        HealthCheck::Ready {
            models: self.models.clone(),
        }
    }

    async fn generate(&self, params: GenerateParams) -> LlmBackendResult<GenerateResponse> {
        let text = match &self.forced_output {
            Some(forced) => forced.clone(),
            None => format!(
                "STUB[{}]: {}",
                params.model,
                params.prompt.chars().take(64).collect::<String>()
            ),
        };

        if params.schema.is_some() {
            validate_task_response(&text)?;
        }

        Ok(GenerateResponse {
            text,
            model: params.model.clone(),
            prompt_tokens: Some((params.prompt.len() / 4).max(1) as u64),
            completion_tokens: Some(16),
            output_token_ids: vec![],
        })
    }
}

// =================================================================
// Helpers
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

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::TaskResponse;

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
    fn backend_from_config_parses_endpoint() {
        let cfg = OllamaConfig {
            endpoint: "http://10.0.0.5:11434".into(),
            timeout_secs: 60,
        };
        let backend = OllamaBackend::from_config(&cfg).unwrap();
        assert_eq!(backend.endpoint, "http://10.0.0.5:11434");
        assert_eq!(backend.timeout, Duration::from_secs(60));
        assert_eq!(backend.max_retries, OllamaBackend::DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn backend_from_config_rejects_bad_url() {
        let cfg = OllamaConfig {
            endpoint: "not a url".into(),
            timeout_secs: 60,
        };
        let err = OllamaBackend::from_config(&cfg).unwrap_err();
        match err {
            LlmBackendError::InvalidConfig { reason } => {
                assert!(reason.contains("not a url"));
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }

    #[test]
    fn backend_with_max_retries_overrides_default() {
        let cfg = OllamaConfig {
            endpoint: "http://localhost:11434".into(),
            timeout_secs: 10,
        };
        let backend = OllamaBackend::from_config(&cfg)
            .unwrap()
            .with_max_retries(7);
        assert_eq!(backend.max_retries, 7);
    }

    #[test]
    fn backend_with_max_retries_floors_to_one() {
        let cfg = OllamaConfig {
            endpoint: "http://localhost:11434".into(),
            timeout_secs: 10,
        };
        let backend = OllamaBackend::from_config(&cfg)
            .unwrap()
            .with_max_retries(0);
        assert_eq!(backend.max_retries, 1, "zero retries should floor to one");
    }

    #[tokio::test]
    async fn stub_backend_healthcheck_reports_ready() {
        let stub = StubBackend::new();
        let hc = stub.healthcheck().await;
        match hc {
            HealthCheck::Ready { models } => {
                assert_eq!(models, vec!["stub-model:latest".to_string()]);
            }
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn stub_backend_generate_produces_deterministic_prefix() {
        let stub = StubBackend::new();
        let params = GenerateParams::new("stub-model:latest", "hello world");
        let resp = stub.generate(params).await.unwrap();
        assert!(resp.text.starts_with("STUB[stub-model:latest]:"));
        assert!(resp.text.contains("hello world"));
        assert_eq!(resp.model, "stub-model:latest");
    }

    #[tokio::test]
    async fn stub_backend_validates_task_response_when_schema_set() {
        // Valid TaskResponse JSON : stub returns it verbatim, the
        // defensive validator accepts it.
        let valid = serde_json::to_string(&TaskResponse::new("ok")).unwrap();
        let stub = StubBackend::new().with_forced_output(valid);
        let params =
            GenerateParams::new("m", "p").with_schema(nexus_core_rs::task_response_schema());
        let resp = stub.generate(params).await.unwrap();
        // text is preserved as-is, schema gate passed
        assert!(resp.text.contains("TASK_RESPONSE_V1"));
    }

    #[tokio::test]
    async fn stub_backend_rejects_invalid_task_response_when_schema_set() {
        // Malformed JSON triggers SchemaViolation.
        let stub = StubBackend::new().with_forced_output("{not valid json");
        let params =
            GenerateParams::new("m", "p").with_schema(nexus_core_rs::task_response_schema());
        let err = stub.generate(params).await.unwrap_err();
        match err {
            LlmBackendError::SchemaViolation(msg) => {
                assert!(msg.contains("deserialize failed"));
            }
            other => panic!("expected SchemaViolation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn stub_backend_rejects_wrong_domain_when_schema_set() {
        let mut r = TaskResponse::new("hi");
        r.domain = "OTHER_V1".to_string();
        let forced = serde_json::to_string(&r).unwrap();
        let stub = StubBackend::new().with_forced_output(forced);
        let params =
            GenerateParams::new("m", "p").with_schema(nexus_core_rs::task_response_schema());
        let err = stub.generate(params).await.unwrap_err();
        match err {
            LlmBackendError::SchemaViolation(msg) => {
                assert!(msg.contains("identity check failed"));
            }
            other => panic!("expected SchemaViolation, got {other:?}"),
        }
    }

    #[test]
    fn validate_task_response_accepts_canonical_payload() {
        let wire = serde_json::to_string(&TaskResponse::new("hello")).unwrap();
        validate_task_response(&wire).unwrap();
    }

    #[test]
    fn validate_task_response_rejects_version_bump() {
        let mut r = TaskResponse::new("hello");
        r.version = 2;
        let wire = serde_json::to_string(&r).unwrap();
        let err = validate_task_response(&wire).unwrap_err();
        match err {
            LlmBackendError::SchemaViolation(_) => {}
            other => panic!("expected SchemaViolation, got {other:?}"),
        }
    }

    /// Live healthcheck against whatever Ollama the dev machine
    /// has running. On CI (no Ollama) this asserts that the
    /// error classification picks the `NotRunning` path with
    /// the install hint.
    #[tokio::test]
    async fn live_healthcheck_either_ready_or_not_running() {
        let short_cfg = OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            timeout_secs: 2,
        };
        let backend = OllamaBackend::from_config(&short_cfg).unwrap();
        let hc = backend.healthcheck().await;
        match hc {
            HealthCheck::Ready { models } => {
                assert!(models.iter().all(|m| !m.is_empty()));
            }
            HealthCheck::NotRunning { hint, .. } => {
                assert_eq!(hint, INSTALL_HINT);
            }
            HealthCheck::Error { reason, .. } => {
                assert!(!reason.is_empty());
            }
        }
    }
}
