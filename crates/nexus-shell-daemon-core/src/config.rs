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
//! Environment-variable layering on top of this
//! (`NEXUS_SHELL_DAEMON__LOGGING__LEVEL=debug` etc.), using the
//! same `config` crate pattern the worker follows, is a possible
//! extension; the loader is deliberately kept to a small function
//! so the singleton behaviour stays the focus.

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

        let grid_root = paths::nexus_grid_root()
            .ok_or(ConfigLoadError::NoNexusGridRoot(paths::NEXUS_GRID_ROOT_ENV))?;
        let root = grid_root.join("shell-daemon");

        Ok(Self {
            config_file: root.join("config.toml"),
            log_dir: grid_root.join("logs"),
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
    #[serde(default)]
    pub seed: SeedConfig,
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

/// `[seed]` section: headless boot seed driver (Sprint 75 Phase E, D3).
///
/// The operational model of an always-on anchor (a VPS) without a UI
/// session: at boot the daemon acquires and pins every project listed
/// here — apps it may have NEVER deployed locally — resolving each one
/// through the subscribed node directories + the best-effort seeder
/// registry, then records a `keep_online` row and re-announces the seed
/// to the feed.
///
/// Anti-recentralization guards (kickoff §4):
/// - **Verrou 3** — the compiled default is EMPTY. The anchor's seed
///   list lives in the OPERATOR's own `config.toml`, never in a
///   non-empty default shipped to everyone (a non-empty compiled
///   default would make one node a de-facto central server).
/// - **Verrou 5 (nuance)** — the boot fetch this section drives is a
///   network call at boot, but it is config-driven EXPLICIT: the
///   operator wrote these project ids. An empty list means ZERO boot
///   network calls from this driver.
/// - **Bounded seed (Q4)** — the per-project accept-list IS the bound
///   (Radicle `seedingPolicy: default block + allow` shape). There is
///   deliberately no numeric disk/app quota knob; the GC reaper /
///   enforced disk budget is deferred post-launch (scope cut #3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SeedConfig {
    /// Project ids (64-char hex, the `blake3(project_name)` id every
    /// deploy/publish path derives) this node acquires + pins at boot.
    /// Empty by default (verrou 3).
    #[serde(default)]
    pub keep_online_projects: Vec<String>,
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
        // T31: validate default_curators are 64-char hex strings.
        self.curator.default_curators.retain(|hex| {
            let valid = hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit());
            if !valid {
                tracing::warn!(
                    curator = %hex,
                    "dropping invalid default_curators entry (expected 64 hex chars)"
                );
            }
            valid
        });
        // Sprint 75 Phase E: same shape rule for the boot seed list — a
        // project id is `hex::encode(blake3(project_name))`, exactly 64 hex
        // chars. A malformed entry can never resolve to pullable content,
        // so drop it loudly at load (mirrors T31). Case is NORMALIZED to
        // lowercase first (the Phase D SeedRegistry lesson): every
        // downstream lookup is an exact lowercase string match, so an
        // uppercase paste would otherwise survive validation but silently
        // never resolve at boot.
        for pid in &mut self.seed.keep_online_projects {
            pid.make_ascii_lowercase();
        }
        self.seed.keep_online_projects.retain(|hex| {
            let valid = hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit());
            if !valid {
                tracing::warn!(
                    project = %hex,
                    "dropping invalid [seed] keep_online_projects entry (expected 64 hex chars)"
                );
            }
            valid
        });
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
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(paths::NEXUS_GRID_ROOT_ENV, tmp.path()) };

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
        assert_eq!(paths.log_dir, tmp.path().join("logs"));

        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(paths::NEXUS_GRID_ROOT_ENV) };
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
    fn t31_invalid_hex_curator_dropped_at_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad-curator.toml");
        std::fs::write(
            &path,
            r#"
[curator]
default_curators = ["not-hex", "ab112233445566778899aabbccddeeff00112233445566778899aabbccddeeff", "short"]
"#,
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(
            loaded.curator.default_curators.len(),
            1,
            "only the valid 64-char hex entry should survive"
        );
        assert_eq!(
            loaded.curator.default_curators[0],
            "ab112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
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

    /// Sprint 75 Phase E (plan §E.3 #5): the on-disk `[seed]` section and
    /// the parser are a producer/consumer pair — this pins the exact TOML
    /// shape `deploy/config.toml.example` documents (section name, key
    /// name, list of 64-hex strings) plus the serde round-trip.
    #[test]
    fn config_seed_section_parsed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let pid_a = "1a".repeat(32);
        let pid_b = "2b".repeat(32);
        std::fs::write(
            &path,
            format!(
                r#"
[seed]
keep_online_projects = ["{pid_a}", "{pid_b}"]
"#
            ),
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(
            loaded.seed.keep_online_projects,
            vec![pid_a, pid_b],
            "the [seed] keep_online_projects list must parse verbatim"
        );

        let body = toml::to_string_pretty(&loaded).unwrap();
        let back: ShellDaemonConfig = toml::from_str(&body).unwrap();
        assert_eq!(loaded.seed, back.seed, "serde round-trip must be stable");
    }

    /// Verrou 3 tripwire (kickoff §4, fail-fast row 13): the COMPILED
    /// default seed list is empty — an anchor's seed set only ever comes
    /// from the operator's own config.toml, never from a shipped default.
    #[test]
    fn seed_section_empty_by_default() {
        assert!(
            ShellDaemonConfig::default()
                .seed
                .keep_online_projects
                .is_empty(),
            "compiled default [seed] list MUST be empty (verrou 3)"
        );

        let dir = tempdir().unwrap();
        let path = dir.path().join("no-seed.toml");
        std::fs::write(&path, "[logging]\nlevel = \"debug\"\n").unwrap();
        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert!(
            loaded.seed.keep_online_projects.is_empty(),
            "absent [seed] section must yield an empty list (zero boot fetch)"
        );
    }

    /// Mirrors T31: a malformed project id (not 64 hex chars) can never
    /// resolve to pullable content — dropped loudly at load. Case
    /// variants are NORMALIZED to lowercase (not dropped): every
    /// downstream lookup is an exact lowercase match, so an uppercase
    /// paste must resolve instead of silently never matching.
    #[test]
    fn invalid_seed_project_ids_dropped_at_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad-seed.toml");
        let valid = "ab".repeat(32);
        let uppercase = "CD".repeat(32);
        std::fs::write(
            &path,
            format!(
                r#"
[seed]
keep_online_projects = ["not-a-project-id", "{valid}", "deadbeef", "{uppercase}"]
"#
            ),
        )
        .unwrap();

        let loaded = ShellDaemonConfig::load(&path).unwrap();
        assert_eq!(
            loaded.seed.keep_online_projects,
            vec![valid, "cd".repeat(32)],
            "valid ids survive; the uppercase paste survives NORMALIZED to lowercase"
        );
    }
}
