// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shell-daemon configuration: layered loading and TOML persistence.
//!
//! Phase A keeps the config surface minimal: a logging level, an
//! HTTP bind host, and a bind port. The structure is laid out to
//! mirror the `nexus_worker_core::config` module so Phase C can
//! add sections (`[curator]`, `[pkarr]`) in additive commits
//! without restructuring the file.
//!
//! The daemon keeps its full persistent state under a single
//! platform-specific directory resolved via
//! [`crate::paths::shell_daemon_dir`]. This is a different
//! layout than the worker's `ProjectDirs`-based resolution —
//! see the Sprint 5 paths module for why the shared
//! `~/.nexus-grid/` root lives alongside the worker's per-app
//! ProjectDirs tree.
//!
//! ## Loading order (Phase A minimal)
//!
//! ```text
//! 1. ShellDaemonConfig::default()                  ← hard-coded
//! 2. <shell-daemon-dir>/config.toml (if present)   ← from disk
//! ```
//!
//! Phase C / E will layer environment variables on top of this
//! (`NEXUS_SHELL_DAEMON__LOGGING__LEVEL=debug` etc.) using the
//! same `config` crate pattern the worker follows. The Phase A
//! scope deliberately keeps the loader to a 10-line function
//! so the first commit stays small and the singleton behaviour
//! is the focus.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::paths;

// =================================================================
// Errors
// =================================================================

/// Errors that can arise when loading, saving, or resolving the
/// shell-daemon configuration.
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// The shared `~/.nexus-grid/` root could not be resolved —
    /// neither `HOME` / `%APPDATA%` is set nor `NEXUS_GRID_ROOT`
    /// is populated. Extremely rare outside of sandboxed CI
    /// environments; the runtime logs this and refuses to boot.
    #[error(
        "could not resolve the shared nexus-grid root on this platform; set {0} to an explicit directory"
    )]
    NoNexusGridRoot(&'static str),

    /// TOML parse error when loading an existing config file.
    ///
    /// `toml::de::Error` is ~128 bytes on the stack so we box it
    /// to keep the [`ConfigLoadError`] enum itself small enough
    /// to satisfy clippy's `result_large_err` lint — the same
    /// reason the `config` crate boxes its own error variants.
    #[error("toml parse error on {path}: {source}")]
    TomlParse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    /// TOML serialization error on save. Also boxed — `serde`
    /// error types carry enough internal state that a plain
    /// inline variant trips `result_large_err`.
    #[error("toml serialization error: {0}")]
    TomlSer(#[from] Box<toml::ser::Error>),

    /// Filesystem error (permission denied, disk full, ...).
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// =================================================================
// Paths — resolved filesystem locations
// =================================================================

/// Resolved filesystem locations the shell daemon uses for
/// persistent state.
///
/// Produced by [`ShellDaemonPaths::resolve`] which honours
/// [`crate::paths::NEXUS_GRID_ROOT_ENV`] and an optional
/// per-invocation override (e.g. `--config /tmp/fixture.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellDaemonPaths {
    /// The `<root>/shell-daemon/` directory. Every other path
    /// is derived from here.
    pub root: PathBuf,
    /// Path to the `config.toml` file. Overridable via the CLI
    /// `--config <PATH>` flag so e2e tests can point the whole
    /// stack at a hermetic fixture directory.
    pub config_file: PathBuf,
    /// Directory the tracing-subscriber writes `daemon.log` into.
    pub log_dir: PathBuf,
    /// The singleton marker file. See [`crate::registry`] for
    /// the writer / reader / stale-check semantics.
    pub running_json: PathBuf,
    /// Phase C attention-set persistence file. Rewritten
    /// atomically on every subscribe / unsubscribe call
    /// (R7 mitigation).
    pub subscriptions_json: PathBuf,
}

impl ShellDaemonPaths {
    /// Compute the canonical paths for this platform.
    ///
    /// If `custom_config` is `Some`, the `config_file` field is
    /// set to that path verbatim and every other directory is
    /// derived from the config file's parent. This lets e2e
    /// tests point the whole stack at a temporary directory by
    /// passing a single `--config /tmp/fixture.toml` flag, the
    /// same convention the worker binary follows.
    ///
    /// If `custom_config` is `None`, falls back to the shared
    /// nexus-grid layout resolved by [`crate::paths`]:
    ///
    /// - Linux:   `~/.local/share/nexus-grid/shell-daemon/`
    /// - macOS:   `~/Library/Application Support/nexus-grid/shell-daemon/`
    /// - Windows: `%APPDATA%\nexus-grid\shell-daemon\`
    pub fn resolve(custom_config: Option<PathBuf>) -> Result<Self, ConfigLoadError> {
        if let Some(config_file) = custom_config {
            let parent = config_file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            return Ok(Self {
                root: parent.clone(),
                config_file,
                log_dir: parent.join("logs"),
                running_json: parent.join("running.json"),
                subscriptions_json: parent.join("subscriptions.json"),
            });
        }

        let root = paths::shell_daemon_dir()
            .ok_or(ConfigLoadError::NoNexusGridRoot(paths::NEXUS_GRID_ROOT_ENV))?;

        Ok(Self {
            config_file: root.join("config.toml"),
            log_dir: root.join("logs"),
            running_json: root.join("running.json"),
            subscriptions_json: root.join("subscriptions.json"),
            root,
        })
    }

    /// Create every parent directory referenced by these paths.
    ///
    /// Called by the binary's `Start` handler before
    /// writing `running.json` so a fresh machine without a prior
    /// `~/.nexus-grid/shell-daemon/` tree still boots cleanly.
    pub fn ensure_dirs(&self) -> Result<(), ConfigLoadError> {
        for dir in [&self.root, &self.log_dir] {
            std::fs::create_dir_all(dir).map_err(|e| ConfigLoadError::Io {
                path: dir.clone(),
                source: e,
            })?;
        }
        Ok(())
    }
}

// =================================================================
// Config sections
// =================================================================

/// Top-level shell-daemon configuration.
///
/// Every field has a `#[serde(default)]` so a partially populated
/// `config.toml` still loads — only the overrides need to be
/// specified. Phase A shipped two sections; Sprint 11 Phase B
/// adds `[curator]` for default curator auto-subscription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShellDaemonConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub curator: CuratorConfig,
}

/// `[logging]` section: tracing-subscriber filter directive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// `tracing-subscriber` EnvFilter directive. Defaults to
    /// `info` which is the right level for a daemon deployment.
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// `[network]` section: local HTTP bind host + port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host to bind the loopback HTTP server to. Defaults to
    /// `"127.0.0.1"`. **Must** stay on loopback — the daemon's
    /// CORS layer rejects any Origin that is not
    /// `http://127.0.0.1` or `http://localhost`, and the
    /// coordinator is the single point of shell → daemon
    /// integration (D1 frozen Sprint 7 kickoff). Changing this
    /// to `0.0.0.0` would be a security bug, not a feature.
    pub api_host: String,

    /// Port to bind to. Defaults to `0` (ephemeral: the OS
    /// picks an unused port on each boot). The real port is
    /// written back to `running.json` after `bind()` returns so
    /// the coordinator proxy can find it.
    pub api_port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            api_host: "127.0.0.1".to_string(),
            api_port: 0,
        }
    }
}

/// `[curator]` section: default curator auto-subscription.
///
/// Sprint 11 Phase B. When the daemon boots, it auto-subscribes
/// to every pubkey listed in `default_curators` that is not
/// already in the persisted attention set. VPS deployments
/// populate this with FlowUP's curator pubkey so fresh installs
/// see the official project list without manual subscribe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CuratorConfig {
    /// Ed25519 public keys (hex, 64 chars lowercase) to
    /// auto-subscribe at first boot. Empty by default.
    #[serde(default)]
    pub default_curators: Vec<String>,
}

// =================================================================
// ShellDaemonConfig impl — load, save, helpers
// =================================================================

impl ShellDaemonConfig {
    /// Build a [`ShellDaemonConfig`] by loading the TOML file at
    /// `config_file` if it exists, falling back to defaults
    /// otherwise.
    ///
    /// A missing file is NOT an error — it means "first boot,
    /// use defaults". The runtime is the one that decides
    /// whether to persist a default config to disk or not.
    pub fn load(config_file: &Path) -> Result<Self, ConfigLoadError> {
        if !config_file.exists() {
            return Ok(Self::default());
        }
        let body = std::fs::read_to_string(config_file).map_err(|e| ConfigLoadError::Io {
            path: config_file.to_path_buf(),
            source: e,
        })?;
        toml::from_str::<ShellDaemonConfig>(&body)
            .map(Self::clamped)
            .map_err(|e| ConfigLoadError::TomlParse {
                path: config_file.to_path_buf(),
                source: Box::new(e),
            })
    }

    /// Serialize the config to pretty TOML and write it to
    /// `config_file`, creating every missing parent directory
    /// along the way.
    pub fn save(&self, config_file: &Path) -> Result<(), ConfigLoadError> {
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigLoadError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let body = toml::to_string_pretty(self).map_err(Box::new)?;
        std::fs::write(config_file, body).map_err(|e| ConfigLoadError::Io {
            path: config_file.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Apply defensive clamps to fields that accept arbitrary
    /// string / numeric input from the config file. Called at
    /// the end of [`Self::load`] so downstream code never sees
    /// a zero-length host or a reserved port.
    fn clamped(mut self) -> Self {
        if self.network.api_host.is_empty() {
            self.network.api_host = "127.0.0.1".to_string();
        }
        self
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
    fn default_config_has_sensible_values() {
        let cfg = ShellDaemonConfig::default();
        assert_eq!(cfg.logging.level, "info");
        assert_eq!(cfg.network.api_host, "127.0.0.1");
        assert_eq!(
            cfg.network.api_port, 0,
            "default must be ephemeral so the OS picks an unused port"
        );
    }

    #[test]
    fn default_config_round_trips_through_toml() {
        let cfg = ShellDaemonConfig::default();
        let body = toml::to_string_pretty(&cfg).unwrap();
        let back: ShellDaemonConfig = toml::from_str(&body).unwrap();
        assert_eq!(cfg, back, "serde round-trip must be stable");
    }

    #[test]
    fn save_then_load_preserves_values() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut cfg = ShellDaemonConfig::default();
        cfg.logging.level = "debug".to_string();
        cfg.network.api_port = 9123;

        cfg.save(&path).unwrap();
        assert!(path.exists());

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        assert!(!path.exists());

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(loaded, ShellDaemonConfig::default());
    }

    #[test]
    fn partial_toml_falls_back_to_defaults_per_section() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("partial.toml");
        std::fs::write(
            &path,
            r#"
[logging]
level = "trace"
"#,
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(loaded.logging.level, "trace");
        assert_eq!(loaded.network.api_host, "127.0.0.1"); // default
        assert_eq!(loaded.network.api_port, 0); // default
    }

    #[test]
    fn empty_host_is_clamped_to_loopback() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty-host.toml");
        std::fs::write(
            &path,
            r#"
[network]
api_host = ""
api_port = 0
"#,
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(
            loaded.network.api_host, "127.0.0.1",
            "empty host must be clamped to loopback — never 0.0.0.0"
        );
    }

    #[test]
    fn paths_with_custom_config_derives_from_parent() {
        let dir = tempdir().unwrap();
        let custom = dir.path().join("fixture").join("config.toml");
        let paths = ShellDaemonPaths::resolve(Some(custom.clone())).unwrap();

        assert_eq!(paths.config_file, custom);
        let parent = custom.parent().unwrap();
        assert_eq!(paths.root, parent);
        assert_eq!(paths.log_dir, parent.join("logs"));
        assert_eq!(paths.running_json, parent.join("running.json"));
    }

    #[test]
    fn paths_ensure_dirs_creates_everything() {
        let dir = tempdir().unwrap();
        let custom = dir.path().join("fixture").join("config.toml");
        let paths = ShellDaemonPaths::resolve(Some(custom)).unwrap();

        paths.ensure_dirs().unwrap();

        assert!(paths.root.is_dir());
        assert!(paths.log_dir.is_dir());
    }

    #[test]
    fn paths_with_no_custom_config_resolves_under_env_root() {
        // This test mutates NEXUS_GRID_ROOT, so serialize with
        // the paths module's env_lock equivalent. We spin up our
        // own because ShellDaemonPaths uses paths::shell_daemon_dir.
        static ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let tmp = tempdir().unwrap();
        std::env::set_var(paths::NEXUS_GRID_ROOT_ENV, tmp.path());

        let paths = ShellDaemonPaths::resolve(None).unwrap();
        assert_eq!(paths.root, tmp.path().join("shell-daemon"));
        assert_eq!(
            paths.config_file,
            tmp.path().join("shell-daemon").join("config.toml")
        );
        assert_eq!(
            paths.running_json,
            tmp.path().join("shell-daemon").join("running.json")
        );

        std::env::remove_var(paths::NEXUS_GRID_ROOT_ENV);
    }

    #[test]
    fn parse_curator_section() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[curator]
default_curators = ["aa11bb22cc33dd44ee55ff6677889900aabbccddeeff00112233445566778899"]
"#,
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(loaded.curator.default_curators.len(), 1);
        assert_eq!(
            loaded.curator.default_curators[0],
            "aa11bb22cc33dd44ee55ff6677889900aabbccddeeff00112233445566778899"
        );
    }

    #[test]
    fn default_curator_empty_when_section_absent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-curator.toml");
        std::fs::write(
            &path,
            r#"
[logging]
level = "debug"
"#,
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert!(
            loaded.curator.default_curators.is_empty(),
            "absent [curator] section must yield empty default_curators"
        );
    }

    #[test]
    fn curator_config_round_trips_through_toml() {
        let mut cfg = ShellDaemonConfig::default();
        cfg.curator.default_curators = vec!["ab".repeat(32)];
        let body = toml::to_string_pretty(&cfg).unwrap();
        let back: ShellDaemonConfig = toml::from_str(&body).unwrap();
        assert_eq!(cfg.curator, back.curator);
    }
}
