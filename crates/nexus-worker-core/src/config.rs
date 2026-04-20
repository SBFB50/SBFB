// SPDX-License-Identifier: AGPL-3.0-or-later
//! Worker configuration: layered loading and TOML persistence.
//!
//! The worker keeps its full state on disk in three files:
//!
//! 1. **`worker.toml`** — human-readable config (this module's
//!    primary responsibility). Loaded via the `config` crate with
//!    defaults ← file ← environment variable precedence.
//! 2. **`worker.key`** — 32-byte Ed25519 secret key. Managed by
//!    [`nexus_core_rs::KeyPair::load_or_generate`].
//! 3. **`allowlist.sqlite3`** — enrolled projects + budgets,
//!    maintained by W7 (see `crate::allowlist`).
//!
//! All three live under a single, platform-specific directory
//! resolved by the [`directories`] crate. On Linux this is
//! `~/.config/nexus-grid/`, on Windows
//! `%APPDATA%\FlowUP\nexus-grid\config\`, and on macOS
//! `~/Library/Application Support/dev.FlowUP.nexus-grid/`.
//!
//! ## Loading order (highest wins)
//!
//! ```text
//! 1. WorkerConfig::default()                     ← hard-coded
//! 2. $CONFIG_DIR/worker.toml  (or --config <P>)  ← from disk
//! 3. NEXUS_WORKER__SECTION__KEY=value            ← env vars
//! ```
//!
//! Env vars use double underscores as section separators so that
//! e.g. `NEXUS_WORKER__OLLAMA__ENDPOINT=http://10.0.0.5:11434`
//! overrides `[ollama] endpoint`. This matches the canonical
//! `config` crate 12-factor pattern.

use std::path::{Path, PathBuf};

use config::{Config as ConfigBuilder, ConfigError, Environment, File as ConfigFile, FileFormat};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{PROJECT_APPLICATION, PROJECT_ORGANIZATION, PROJECT_QUALIFIER};

// =================================================================
// Errors
// =================================================================

/// Errors that can arise when loading, saving, or resolving the
/// worker configuration.
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// The `directories` crate could not determine a standard
    /// application directory for this platform. This is extremely
    /// rare — it happens on exotic OS / sandbox setups where
    /// neither `HOME` nor `%APPDATA%` is set.
    #[error(
        "could not resolve a standard application directory on this platform; set {0} to an explicit config file path"
    )]
    NoProjectDirs(&'static str),

    /// The requested config file does not exist. Returned from
    /// [`WorkerConfig::load_required`] but NOT from
    /// [`WorkerConfig::load`] which treats a missing file as
    /// "use defaults".
    #[error("config file not found at {0}")]
    NotFound(PathBuf),

    /// Underlying error from the `config` crate (parse failure,
    /// type mismatch, malformed TOML, ...).
    #[error("config parse error: {0}")]
    Parse(#[from] ConfigError),

    /// TOML serialization error on save.
    #[error("toml serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Filesystem error (permission denied, disk full, ...).
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// =================================================================
// Paths
// =================================================================

/// Resolved filesystem locations the worker uses for persistent
/// state.
///
/// Produced by [`WorkerPaths::resolve`] which honours both the
/// platform's standard application directories (via
/// [`ProjectDirs`]) and an optional per-invocation override
/// (e.g. `nexus-worker --config /tmp/fixture.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPaths {
    /// Path to the `worker.toml` config file. This is the single
    /// override point — everything else is derived from its
    /// parent directory.
    pub config_file: PathBuf,
    /// Directory for persistent data (secret key, allowlist db).
    pub data_dir: PathBuf,
    /// Directory for rotating log files.
    pub log_dir: PathBuf,
    /// Directory for cached artifacts (model blobs, etc.).
    pub cache_dir: PathBuf,
}

impl WorkerPaths {
    /// Compute the canonical paths for this platform.
    ///
    /// If `custom_config` is `Some`, the `config_file` field is
    /// set to that path and every other directory is derived
    /// from the config file's parent. This lets e2e tests point
    /// the whole stack at a temporary directory by passing a
    /// single `--config /tmp/test-worker.toml` flag.
    ///
    /// If `custom_config` is `None`, falls back to the
    /// `directories` crate's `ProjectDirs` for the current OS:
    ///
    /// - Linux:   `~/.config/nexus-grid/worker.toml`
    /// - macOS:   `~/Library/Application Support/dev.FlowUP.nexus-grid/worker.toml`
    /// - Windows: `%APPDATA%\FlowUP\nexus-grid\config\worker.toml`
    pub fn resolve(custom_config: Option<PathBuf>) -> Result<Self, ConfigLoadError> {
        if let Some(config_file) = custom_config {
            let parent = config_file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            return Ok(Self {
                config_file,
                data_dir: parent.join("data"),
                log_dir: parent.join("logs"),
                cache_dir: parent.join("cache"),
            });
        }

        let proj = ProjectDirs::from(PROJECT_QUALIFIER, PROJECT_ORGANIZATION, PROJECT_APPLICATION)
            .ok_or(ConfigLoadError::NoProjectDirs("--config <PATH>"))?;

        Ok(Self {
            config_file: proj.config_dir().join("worker.toml"),
            data_dir: proj.data_dir().to_path_buf(),
            // state_dir is Linux-only; fall back to data_dir on
            // macOS / Windows where state and data coincide.
            log_dir: proj
                .state_dir()
                .map(|p| p.join("logs"))
                .unwrap_or_else(|| proj.data_dir().join("logs")),
            cache_dir: proj.cache_dir().to_path_buf(),
        })
    }

    /// The canonical path for the Ed25519 secret key file if the
    /// config does not override it.
    pub fn default_secret_key_file(&self) -> PathBuf {
        self.data_dir.join("worker.key")
    }

    /// The canonical path for the allowlist SQLite database (W7).
    pub fn default_allowlist_db(&self) -> PathBuf {
        self.data_dir.join("allowlist.sqlite3")
    }

    /// The canonical path for the NVML utilization+duration profile
    /// SQLite database (Sprint 22 Phase D — log-only baseline
    /// foundation for the Sprint 24 random re-run sampling
    /// detection of `C-ComputeTheft`).
    pub fn default_nvml_profile_db(&self) -> PathBuf {
        self.data_dir.join("nvml_profile.sqlite3")
    }

    /// Create every parent directory referenced by these paths.
    ///
    /// Called by [`WorkerConfig::save_new`] before writing the
    /// initial `worker.toml`, and can be called again by the
    /// engine on startup to tolerate user-initiated cleanups.
    pub fn ensure_dirs(&self) -> Result<(), ConfigLoadError> {
        for dir in [
            self.config_file.parent().unwrap_or(Path::new(".")),
            &self.data_dir,
            &self.log_dir,
            &self.cache_dir,
        ] {
            std::fs::create_dir_all(dir).map_err(|e| ConfigLoadError::Io {
                path: dir.to_path_buf(),
                source: e,
            })?;
        }
        Ok(())
    }
}

// =================================================================
// Config sections
// =================================================================

/// Top-level worker configuration.
///
/// Every field has a `#[serde(default)]` so a partially populated
/// `worker.toml` still loads — only the overrides need to be
/// specified. The full set of defaults is produced by
/// `WorkerConfig::default()` via the derived `Default` impl,
/// which in turn uses each section's own `Default` impl.
///
/// ## Sprint 20 Phase D breaking change
///
/// The pre-S20 `[ollama]` section has been folded into
/// `[llm.ollama]` alongside the new `[llm.llama_cpp]` section.
/// Worker config files must regenerate — the project is pre-launch
/// (no tolerant decoder per `CLAUDE.md §Pre-launch protocol policy`)
/// so there is no backward-compatible alias for the old path.
///
/// Env-var overrides follow the same rename :
/// `NEXUS_WORKER__OLLAMA__ENDPOINT` → `NEXUS_WORKER__LLM__OLLAMA__ENDPOINT`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkerConfig {
    #[serde(default)]
    pub identity: Identity,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub gpu: Gpu,
    #[serde(default)]
    pub engine: Engine,
    #[serde(default)]
    pub logging: Logging,
    #[serde(default)]
    pub ephemeral: crate::ephemeral::EphemeralConfig,
}

/// `[identity]` section: who this worker is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// Human-readable name, shown in local CLI output and logs.
    /// Never shared over the P2P network (the Ed25519 public key
    /// is the only network-visible identifier).
    pub name: String,

    /// Optional override for the secret key file path.
    ///
    /// `None` ⇒ use [`WorkerPaths::default_secret_key_file`]
    /// (platform standard). Set to a custom path to point the
    /// worker at a different key file, e.g. for e2e tests that
    /// share a fixture directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_path: Option<PathBuf>,
}

impl Default for Identity {
    fn default() -> Self {
        Self {
            name: "unnamed-worker".to_string(),
            secret_key_path: None,
        }
    }
}

/// `[llm]` section: which backend the worker uses + per-backend
/// subsections.
///
/// Sprint 20 Phase D replaces the old `[ollama]` top-level section
/// with a backend-agnostic `[llm]` container. Two backends ship :
///
/// - `ollama` : HTTP to a local Ollama daemon. Always compiled.
/// - `llama_cpp` : in-process `llama-cpp-2` + `llguidance`. Gated
///   behind the `llm_llama_cpp` Cargo feature (cmake required).
///
/// The `backend` field selects which one is used at runtime.
/// Defaults to `Ollama` so a vanilla build without the
/// `llm_llama_cpp` feature works out of the box ; production
/// deployments are expected to set `backend = "llama_cpp"` and
/// build with the feature enabled (see the design doc
/// `.planning/research/S20_phase_D_structured_output_design.md`
/// §2.4 for the rationale).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    #[serde(default)]
    pub backend: BackendKind,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub llama_cpp: LlamaCppConfig,
}

/// Which [`LlmBackend`][crate::llm::LlmBackend] the worker runs.
///
/// Default is [`BackendKind::Ollama`] — always compiled and
/// cmake-free. A future HARDENING sprint may flip the default to
/// `LlamaCpp` once the build chain is documented across all dev
/// platforms (cf. design doc §2.4 "Tension résolue").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// Ollama HTTP daemon. Default.
    #[default]
    Ollama,
    /// In-process llama.cpp with llguidance-constrained sampling.
    /// Requires the `llm_llama_cpp` Cargo feature.
    LlamaCpp,
}

/// `[llm.ollama]` section: how to reach the local Ollama daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// HTTP endpoint to the Ollama server (default:
    /// `http://localhost:11434`, matching the Ollama installer).
    pub endpoint: String,
    /// Per-request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            timeout_secs: 300,
        }
    }
}

/// `[llm.llama_cpp]` section: how the in-process llama.cpp runtime
/// loads its model.
///
/// Populated only when `[llm] backend = "llama_cpp"`. The factory
/// validates paths eagerly so a missing or unreadable GGUF surfaces
/// at startup (not on the first generate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    /// Absolute or tilde-expandable path to the GGUF model file.
    /// Defaults to an empty string ; the factory rejects an empty
    /// path when the backend is actually selected.
    pub model_path: String,
    /// Context window size in tokens (`n_ctx`). Defaults to 4096 ;
    /// a caller that wants the model's training default can set
    /// this to `0` and the backend interprets it as "use model
    /// default" (mirrors `llama-cpp-2` `Option<NonZeroU32>`).
    pub n_ctx: u32,
    /// Number of layers to offload to the GPU. `-1` offloads all
    /// available layers ; `0` stays on CPU. Matches
    /// `LlamaModelParams::with_n_gpu_layers` semantics.
    pub n_gpu_layers: i32,
    /// CPU threads used during generation. `0` ⇒ let llama.cpp
    /// pick based on the number of cores.
    pub n_threads: u32,
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            n_ctx: 4096,
            n_gpu_layers: -1,
            n_threads: 0,
        }
    }
}

/// `[gpu]` section: how aggressively the worker may use the GPU.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gpu {
    /// Fraction of total VRAM the worker is allowed to consume.
    /// Values are clamped to `[0.1, 1.0]` at load time.
    pub max_vram_fraction: f32,
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            max_vram_fraction: 0.9,
        }
    }
}

/// `[engine]` section: state-machine and scheduling knobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Engine {
    /// How often to scan project docs for new tasks, in ms.
    pub task_poll_interval_ms: u64,
    /// Maximum number of tasks running concurrently. Bounded in
    /// practice by VRAM and Ollama's own concurrency limits.
    pub max_concurrent_tasks: usize,
    /// Sprint 5 Phase A: how often to flush the shell-facing
    /// `state.json` snapshot, in seconds. The shell polls the
    /// coordinator proxy every 2s by default so 5s here gives
    /// a responsive display without thrashing the SSD. Clamped
    /// to a minimum of 1s at load time.
    #[serde(default = "default_state_flush_secs")]
    pub state_flush_secs: u64,
}

fn default_state_flush_secs() -> u64 {
    5
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            task_poll_interval_ms: 2000,
            max_concurrent_tasks: 1,
            state_flush_secs: default_state_flush_secs(),
        }
    }
}

/// `[logging]` section: tracing-subscriber filter directive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Logging {
    /// `tracing-subscriber` EnvFilter directive. Defaults to
    /// `info` which is the right level for daemon deployments.
    pub level: String,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

// =================================================================
// WorkerConfig impl — load, save, helpers
// =================================================================

impl WorkerConfig {
    /// Environment variable prefix for overrides.
    ///
    /// Nested keys use double underscores, e.g.
    /// `NEXUS_WORKER__OLLAMA__ENDPOINT=http://10.0.0.5:11434`.
    const ENV_PREFIX: &'static str = "NEXUS_WORKER";
    const ENV_SEPARATOR: &'static str = "__";

    /// Build a [`WorkerConfig`] by layering defaults, the TOML
    /// file at `config_file` (if it exists), and environment
    /// variables with the `NEXUS_WORKER__` prefix.
    ///
    /// A missing `config_file` is NOT an error — it just means
    /// the defaults + env vars will be used. If you want a hard
    /// failure on a missing file, use [`WorkerConfig::load_required`].
    pub fn load(config_file: &Path) -> Result<Self, ConfigLoadError> {
        let mut builder = ConfigBuilder::builder();

        // Layer 1: hard-coded defaults as typed overrides so the
        // `config` crate applies correct types even when every
        // other layer is empty.
        let defaults = Self::default();
        let default_toml = toml::to_string(&defaults)?;
        builder = builder.add_source(ConfigFile::from_str(&default_toml, FileFormat::Toml));

        // Layer 2: the config file on disk, if present.
        if config_file.exists() {
            builder = builder.add_source(
                ConfigFile::from(config_file)
                    .format(FileFormat::Toml)
                    .required(true),
            );
        }

        // Layer 3: environment variables NEXUS_WORKER__*.
        builder = builder.add_source(
            Environment::with_prefix(Self::ENV_PREFIX)
                .separator(Self::ENV_SEPARATOR)
                .try_parsing(true),
        );

        let cfg: WorkerConfig = builder.build()?.try_deserialize()?;
        Ok(cfg.clamped())
    }

    /// Same as [`WorkerConfig::load`] but returns
    /// [`ConfigLoadError::NotFound`] if `config_file` does not
    /// exist on disk.
    ///
    /// Use this from `start` / `stats` / `join` commands where a
    /// missing config means the worker was never registered.
    pub fn load_required(config_file: &Path) -> Result<Self, ConfigLoadError> {
        if !config_file.exists() {
            return Err(ConfigLoadError::NotFound(config_file.to_path_buf()));
        }
        Self::load(config_file)
    }

    /// Serialize the config to pretty TOML and write it to
    /// `config_file`, creating every missing parent directory
    /// along the way.
    ///
    /// This is a plain overwrite — callers that want
    /// "register only if not already registered" semantics
    /// should check `config_file.exists()` before calling.
    pub fn save(&self, config_file: &Path) -> Result<(), ConfigLoadError> {
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigLoadError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let body = toml::to_string_pretty(self)?;
        std::fs::write(config_file, body).map_err(|e| ConfigLoadError::Io {
            path: config_file.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Apply range clamps to fields that accept arbitrary numeric
    /// input. Called at the end of [`WorkerConfig::load`] so
    /// downstream code never sees out-of-range values.
    fn clamped(mut self) -> Self {
        self.gpu.max_vram_fraction = self.gpu.max_vram_fraction.clamp(0.1, 1.0);
        if self.engine.max_concurrent_tasks == 0 {
            self.engine.max_concurrent_tasks = 1;
        }
        if self.engine.task_poll_interval_ms < 100 {
            self.engine.task_poll_interval_ms = 100;
        }
        if self.engine.state_flush_secs == 0 {
            self.engine.state_flush_secs = 1;
        }
        self
    }

    /// Return the concrete secret key path for this config, given
    /// the resolved [`WorkerPaths`]. Honours
    /// `identity.secret_key_path` if set, otherwise falls back to
    /// [`WorkerPaths::default_secret_key_file`].
    pub fn resolve_secret_key_path(&self, paths: &WorkerPaths) -> PathBuf {
        self.identity
            .secret_key_path
            .clone()
            .unwrap_or_else(|| paths.default_secret_key_file())
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_round_trips_through_toml() {
        let cfg = WorkerConfig::default();
        let body = toml::to_string_pretty(&cfg).unwrap();
        let back: WorkerConfig = toml::from_str(&body).unwrap();
        assert_eq!(cfg, back, "WorkerConfig serde round-trip must be stable");
    }

    #[test]
    fn default_has_sensible_values() {
        let cfg = WorkerConfig::default();
        assert_eq!(cfg.llm.backend, BackendKind::Ollama);
        assert_eq!(cfg.llm.ollama.endpoint, "http://localhost:11434");
        assert_eq!(cfg.llm.ollama.timeout_secs, 300);
        assert_eq!(cfg.llm.llama_cpp.model_path, "");
        assert_eq!(cfg.llm.llama_cpp.n_ctx, 4096);
        assert_eq!(cfg.llm.llama_cpp.n_gpu_layers, -1);
        assert_eq!(cfg.gpu.max_vram_fraction, 0.9);
        assert_eq!(cfg.engine.task_poll_interval_ms, 2000);
        assert_eq!(cfg.engine.max_concurrent_tasks, 1);
        assert_eq!(cfg.engine.state_flush_secs, 5);
        assert_eq!(cfg.logging.level, "info");
        assert_eq!(cfg.identity.name, "unnamed-worker");
        assert_eq!(cfg.identity.secret_key_path, None);
    }

    #[test]
    fn save_then_load_preserves_values() {
        // This test reads `llm.ollama.endpoint` through
        // WorkerConfig::load, which honours
        // NEXUS_WORKER__LLM__OLLAMA__ENDPOINT. The
        // env_var_overrides_file_value test below briefly sets that
        // variable; if cargo schedules the two in parallel this test
        // can observe the leaked value. Hold the shared lock and
        // defensively clear the var before loading.
        let _guard = env_var_test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("NEXUS_WORKER__LLM__OLLAMA__ENDPOINT");

        let dir = tempdir().unwrap();
        let path = dir.path().join("worker.toml");

        let mut cfg = WorkerConfig::default();
        cfg.identity.name = "rtx5080-home".to_string();
        cfg.llm.ollama.endpoint = "http://10.0.0.5:11434".to_string();
        cfg.engine.max_concurrent_tasks = 4;

        cfg.save(&path).unwrap();
        assert!(path.exists(), "save must create the config file");

        let loaded = WorkerConfig::load(&path).unwrap();
        assert_eq!(cfg, loaded);
    }

    /// Serialize every test that touches
    /// `NEXUS_WORKER__LLM__OLLAMA__ENDPOINT` so cargo's default
    /// parallel test runner doesn't race them against each other.
    /// Added in Sprint 4 Phase C after the env-var test caused
    /// intermittent failures under parallel execution (renamed
    /// in Sprint 20 Phase D with the `[llm]` migration).
    fn env_var_test_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    #[test]
    fn load_missing_file_returns_defaults_plus_env() {
        let _guard = env_var_test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Paranoid cleanup so a leaked env var from a hypothetical
        // third test cannot contaminate this one either.
        std::env::remove_var("NEXUS_WORKER__LLM__OLLAMA__ENDPOINT");

        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        assert!(!path.exists());

        let loaded = WorkerConfig::load(&path).unwrap();
        assert_eq!(loaded, WorkerConfig::default());
    }

    #[test]
    fn load_required_missing_file_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        let err = WorkerConfig::load_required(&path).unwrap_err();
        match err {
            ConfigLoadError::NotFound(p) => assert_eq!(p, path),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn partial_toml_falls_back_to_defaults_per_section() {
        // Same rationale as save_then_load_preserves_values:
        // reads through WorkerConfig::load which is sensitive to
        // a leaked NEXUS_WORKER__LLM__OLLAMA__ENDPOINT from a
        // parallel test. Hold the shared lock and clear the var.
        let _guard = env_var_test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("NEXUS_WORKER__LLM__OLLAMA__ENDPOINT");

        let dir = tempdir().unwrap();
        let path = dir.path().join("partial.toml");

        // Only override llm.ollama.endpoint — every other section
        // must stay at the default.
        std::fs::write(
            &path,
            r#"
[llm.ollama]
endpoint = "http://elsewhere:11434"
"#,
        )
        .unwrap();

        let loaded = WorkerConfig::load(&path).unwrap();
        assert_eq!(loaded.llm.backend, BackendKind::Ollama); // default
        assert_eq!(loaded.llm.ollama.endpoint, "http://elsewhere:11434");
        assert_eq!(loaded.llm.ollama.timeout_secs, 300); // default
        assert_eq!(loaded.gpu.max_vram_fraction, 0.9); // default
        assert_eq!(loaded.identity.name, "unnamed-worker"); // default
    }

    #[test]
    fn clamp_applies_to_out_of_range_values() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("out-of-range.toml");

        std::fs::write(
            &path,
            r#"
[gpu]
max_vram_fraction = 5.0

[engine]
task_poll_interval_ms = 10
max_concurrent_tasks = 0
state_flush_secs = 0
"#,
        )
        .unwrap();

        let loaded = WorkerConfig::load(&path).unwrap();
        assert_eq!(loaded.gpu.max_vram_fraction, 1.0, "clamped to upper bound");
        assert_eq!(
            loaded.engine.task_poll_interval_ms, 100,
            "clamped to minimum interval"
        );
        assert_eq!(loaded.engine.max_concurrent_tasks, 1, "zero clamped to one");
        assert_eq!(
            loaded.engine.state_flush_secs, 1,
            "zero flush interval clamped to 1s minimum"
        );
    }

    #[test]
    fn secret_key_path_is_omitted_from_toml_when_none() {
        let cfg = WorkerConfig::default();
        let body = toml::to_string_pretty(&cfg).unwrap();
        assert!(
            !body.contains("secret_key_path"),
            "default config must not write the secret_key_path key; got: {body}"
        );
    }

    #[test]
    fn secret_key_path_is_written_when_some() {
        let mut cfg = WorkerConfig::default();
        cfg.identity.secret_key_path = Some(PathBuf::from("/custom/key.bin"));
        let body = toml::to_string_pretty(&cfg).unwrap();
        assert!(body.contains("secret_key_path"));
        assert!(body.contains("/custom/key.bin"));
    }

    #[test]
    fn resolve_secret_key_path_falls_back_to_default() {
        let paths = WorkerPaths {
            config_file: PathBuf::from("/tmp/cfg/worker.toml"),
            data_dir: PathBuf::from("/tmp/data"),
            log_dir: PathBuf::from("/tmp/logs"),
            cache_dir: PathBuf::from("/tmp/cache"),
        };

        let cfg = WorkerConfig::default();
        assert_eq!(
            cfg.resolve_secret_key_path(&paths),
            PathBuf::from("/tmp/data/worker.key")
        );

        let mut cfg_override = WorkerConfig::default();
        cfg_override.identity.secret_key_path = Some(PathBuf::from("/special/key"));
        assert_eq!(
            cfg_override.resolve_secret_key_path(&paths),
            PathBuf::from("/special/key")
        );
    }

    #[test]
    fn paths_with_custom_config_derives_from_parent() {
        let dir = tempdir().unwrap();
        let custom = dir.path().join("fixture").join("worker.toml");
        let paths = WorkerPaths::resolve(Some(custom.clone())).unwrap();

        assert_eq!(paths.config_file, custom);
        let parent = custom.parent().unwrap();
        assert_eq!(paths.data_dir, parent.join("data"));
        assert_eq!(paths.log_dir, parent.join("logs"));
        assert_eq!(paths.cache_dir, parent.join("cache"));
    }

    #[test]
    fn paths_ensure_dirs_creates_everything() {
        let dir = tempdir().unwrap();
        let custom = dir.path().join("fixture").join("worker.toml");
        let paths = WorkerPaths::resolve(Some(custom)).unwrap();

        paths.ensure_dirs().unwrap();

        assert!(paths.data_dir.is_dir());
        assert!(paths.log_dir.is_dir());
        assert!(paths.cache_dir.is_dir());
        assert!(paths.config_file.parent().unwrap().is_dir());
    }

    #[test]
    fn env_var_overrides_file_value() {
        let _guard = env_var_test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let dir = tempdir().unwrap();
        let path = dir.path().join("env.toml");

        std::fs::write(
            &path,
            r#"
[llm.ollama]
endpoint = "http://from-file:11434"
"#,
        )
        .unwrap();

        // SAFETY: guarded by env_var_test_lock above so no
        // parallel test can observe the transient mutation.
        std::env::set_var(
            "NEXUS_WORKER__LLM__OLLAMA__ENDPOINT",
            "http://from-env:11434",
        );
        let loaded = WorkerConfig::load(&path).unwrap();
        std::env::remove_var("NEXUS_WORKER__LLM__OLLAMA__ENDPOINT");

        assert_eq!(
            loaded.llm.ollama.endpoint, "http://from-env:11434",
            "env var must override file value"
        );
    }

    #[test]
    fn llm_backend_parses_llama_cpp_value() {
        let toml_body = r#"
[llm]
backend = "llama_cpp"

[llm.llama_cpp]
model_path = "/models/qwen.gguf"
n_ctx = 8192
n_gpu_layers = 0
n_threads = 4
"#;
        let cfg: WorkerConfig = toml::from_str(toml_body).unwrap();
        assert_eq!(cfg.llm.backend, BackendKind::LlamaCpp);
        assert_eq!(cfg.llm.llama_cpp.model_path, "/models/qwen.gguf");
        assert_eq!(cfg.llm.llama_cpp.n_ctx, 8192);
        assert_eq!(cfg.llm.llama_cpp.n_gpu_layers, 0);
        assert_eq!(cfg.llm.llama_cpp.n_threads, 4);
    }

    #[test]
    fn llm_section_omitted_falls_back_to_ollama_default() {
        let cfg: WorkerConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.llm.backend, BackendKind::Ollama);
        assert_eq!(cfg.llm.ollama.endpoint, "http://localhost:11434");
    }

    #[test]
    fn backend_kind_serializes_as_snake_case() {
        let via_json = serde_json::to_string(&BackendKind::LlamaCpp).unwrap();
        assert_eq!(via_json, "\"llama_cpp\"");
        let via_json = serde_json::to_string(&BackendKind::Ollama).unwrap();
        assert_eq!(via_json, "\"ollama\"");
    }
}
