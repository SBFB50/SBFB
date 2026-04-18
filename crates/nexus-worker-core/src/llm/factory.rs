// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolve a worker `[llm]` config section into a boxed
//! [`LlmBackend`]. The factory honors Cargo feature flags so the
//! binary fails loud at startup when the config requests a backend
//! that was not compiled in.

use thiserror::Error;

use crate::config::{BackendKind, LlmConfig};

use super::{LlmBackend, LlmBackendError, OllamaBackend};

/// Failures that can arise when building the backend from config.
#[derive(Debug, Error)]
pub enum FactoryError {
    /// Wrapping of [`LlmBackendError`] when the backend's own
    /// constructor rejects the config (bad URL, missing file, ...).
    #[error(transparent)]
    Backend(#[from] LlmBackendError),

    /// The config requested a backend whose feature is not
    /// compiled in this binary. Human-readable hint embeds the
    /// feature name and the fallback option.
    #[error(
        "unsupported llm backend {requested:?}: feature {feature:?} is not compiled in this binary\nhint: rebuild with `cargo build --features {feature}` or set `[llm] backend = \"ollama\"` in worker.toml"
    )]
    UnsupportedBackend {
        requested: &'static str,
        feature: &'static str,
    },
}

/// Build the concrete [`LlmBackend`] the engine should use, given
/// a parsed `[llm]` config section.
///
/// Wrapping the return type as `Box<dyn LlmBackend>` keeps the
/// engine generic over the concrete backend — it consumes the
/// trait surface and never needs to pattern-match on the variant
/// of [`BackendKind`].
pub fn build_backend(cfg: &LlmConfig) -> Result<Box<dyn LlmBackend>, FactoryError> {
    match cfg.backend {
        BackendKind::Ollama => {
            let backend = OllamaBackend::from_config(&cfg.ollama)?;
            Ok(Box::new(backend))
        }
        BackendKind::LlamaCpp => build_llama_cpp(cfg),
    }
}

#[cfg(feature = "llm_llama_cpp")]
fn build_llama_cpp(cfg: &LlmConfig) -> Result<Box<dyn LlmBackend>, FactoryError> {
    let backend = super::llama_cpp::LlamaCppBackend::from_config(&cfg.llama_cpp)?;
    Ok(Box::new(backend))
}

#[cfg(not(feature = "llm_llama_cpp"))]
fn build_llama_cpp(_cfg: &LlmConfig) -> Result<Box<dyn LlmBackend>, FactoryError> {
    Err(FactoryError::UnsupportedBackend {
        requested: "llama_cpp",
        feature: "llm_llama_cpp",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LlamaCppConfig, OllamaConfig};

    fn ollama_cfg() -> LlmConfig {
        LlmConfig {
            backend: BackendKind::Ollama,
            ollama: OllamaConfig {
                endpoint: "http://localhost:11434".into(),
                timeout_secs: 60,
            },
            llama_cpp: LlamaCppConfig::default(),
        }
    }

    #[test]
    fn factory_builds_ollama_backend() {
        let cfg = ollama_cfg();
        let backend = build_backend(&cfg).expect("ollama backend should build");
        // Sanity : the returned trait object is usable as a
        // `dyn LlmBackend`.
        let _: &dyn LlmBackend = backend.as_ref();
    }

    #[test]
    fn factory_rejects_invalid_ollama_endpoint() {
        let mut cfg = ollama_cfg();
        cfg.ollama.endpoint = "not a url".into();
        match build_backend(&cfg) {
            Err(FactoryError::Backend(LlmBackendError::InvalidConfig { reason })) => {
                assert!(reason.contains("not a url"));
            }
            Err(other) => panic!("expected InvalidConfig, got {other:?}"),
            Ok(_) => panic!("expected build_backend to fail on invalid URL"),
        }
    }

    /// When the `llm_llama_cpp` feature is NOT compiled in, the
    /// factory must fail loud at build-backend time rather than
    /// silently falling back to Ollama.
    #[cfg(not(feature = "llm_llama_cpp"))]
    #[test]
    fn factory_rejects_llama_cpp_when_feature_off() {
        let mut cfg = ollama_cfg();
        cfg.backend = BackendKind::LlamaCpp;
        match build_backend(&cfg) {
            Err(FactoryError::UnsupportedBackend {
                requested, feature, ..
            }) => {
                assert_eq!(requested, "llama_cpp");
                assert_eq!(feature, "llm_llama_cpp");
            }
            Err(other) => panic!("expected UnsupportedBackend, got {other:?}"),
            Ok(_) => panic!("feature off must refuse llama_cpp backend"),
        }
    }

    /// When the feature IS compiled in, the factory must attempt
    /// to build the backend. We don't need a real GGUF file on disk
    /// — the expected path is to bubble up an InvalidConfig /
    /// NotRunning error when the GGUF is missing, not a panic.
    #[cfg(feature = "llm_llama_cpp")]
    #[test]
    fn factory_attempts_llama_cpp_when_feature_on() {
        let mut cfg = ollama_cfg();
        cfg.backend = BackendKind::LlamaCpp;
        // Leave the default placeholder model_path — the backend
        // surfaces an InvalidConfig when the file is missing.
        let result = build_backend(&cfg);
        assert!(result.is_err(), "missing GGUF must surface an error");
    }
}
