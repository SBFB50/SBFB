// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tor transport wrapper (arti-client, feature-gated).
//!
//! Scope today: coordinator outbound TCP via Tor. Config opt-in
//! (disabled by default). Fallback to direct if Tor is unavailable.
//! Holding the `TorClient` handle for actual connection routing is
//! not yet wired (the bootstrap handle is dropped after probing).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;

/// Configuration for the `[tor]` section in `tor.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TorConfig {
    /// Master switch — `false` means all connections go direct.
    #[serde(default)]
    pub enabled: bool,
    /// Seconds to wait for the Tor network bootstrap before falling
    /// back to direct connections.
    #[serde(default = "default_bootstrap_timeout")]
    pub bootstrap_timeout_s: u64,
}

fn default_bootstrap_timeout() -> u64 {
    30
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bootstrap_timeout_s: 30,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TorConfigFile {
    tor: TorConfig,
}

impl TorConfig {
    /// Load a `TorConfig` from a TOML file with a top-level `[tor]`
    /// section. Returns the default (disabled) config on any error.
    pub fn from_toml_file(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("tor config not found at {}: {e}", path.display());
                return Self::default();
            }
        };
        match toml::from_str::<TorConfigFile>(&content) {
            Ok(file) => file.tor,
            Err(e) => {
                tracing::warn!("failed to parse tor config at {}: {e}", path.display());
                Self::default()
            }
        }
    }
}

/// Whether the `tor` Cargo feature is compiled in.
pub fn tor_feature_compiled() -> bool {
    cfg!(feature = "tor")
}

/// Tor transport handle. Feature-gated internals: without the `tor`
/// Cargo feature, bootstrap is a no-op and `is_available` always
/// returns `false`.
pub struct TorTransport {
    config: TorConfig,
    available: AtomicBool,
}

impl TorTransport {
    /// Create a new transport from the given config. Does NOT
    /// bootstrap — call [`bootstrap`](Self::bootstrap) to connect.
    pub fn new(config: TorConfig) -> Self {
        Self {
            config,
            available: AtomicBool::new(false),
        }
    }

    /// The active configuration.
    pub fn config(&self) -> &TorConfig {
        &self.config
    }

    /// Whether a Tor circuit is bootstrapped and ready.
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    /// Bootstrap the Tor client. No-op if `enabled = false` or if
    /// the `tor` feature is not compiled in. On bootstrap failure,
    /// logs a warning and falls back to direct (no error returned).
    pub async fn bootstrap(&self) -> crate::Result<()> {
        if !self.config.enabled {
            tracing::info!("tor transport disabled by configuration");
            return Ok(());
        }
        self.bootstrap_inner().await
    }

    #[cfg(feature = "tor")]
    async fn bootstrap_inner(&self) -> crate::Result<()> {
        use arti_client::{TorClient, TorClientConfig};

        tracing::info!(
            timeout_s = self.config.bootstrap_timeout_s,
            "bootstrapping Tor client"
        );

        let arti_config = TorClientConfig::default();
        let timeout = std::time::Duration::from_secs(self.config.bootstrap_timeout_s);

        match tokio::time::timeout(timeout, TorClient::create_bootstrapped(arti_config)).await {
            Ok(Ok(_client)) => {
                tracing::info!("Tor client bootstrapped");
                self.available.store(true, Ordering::Relaxed);
                // The bootstrapped client handle is dropped here;
                // storing it for actual connection routing is not
                // yet wired.
            }
            Ok(Err(e)) => {
                tracing::warn!("Tor bootstrap failed, falling back to direct: {e}");
            }
            Err(_) => {
                tracing::warn!(
                    timeout_s = self.config.bootstrap_timeout_s,
                    "Tor bootstrap timed out, falling back to direct"
                );
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "tor"))]
    async fn bootstrap_inner(&self) -> crate::Result<()> {
        tracing::warn!("Tor transport requested but binary compiled without 'tor' feature");
        Ok(())
    }

    /// Lightweight health probe. Returns `false` if Tor is not
    /// bootstrapped.
    pub fn health_check(&self) -> bool {
        self.is_available()
    }
}

// ======================================================================
// Tests
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tor_config_default() {
        let cfg = TorConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.bootstrap_timeout_s, 30);
    }

    #[test]
    fn test_tor_config_parse_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tor.toml");
        std::fs::write(&path, "[tor]\nenabled = true\nbootstrap_timeout_s = 45\n").unwrap();
        let cfg = TorConfig::from_toml_file(&path);
        assert!(cfg.enabled);
        assert_eq!(cfg.bootstrap_timeout_s, 45);
    }

    #[test]
    fn test_tor_config_missing_file_returns_default() {
        let cfg = TorConfig::from_toml_file(Path::new("/nonexistent/tor.toml"));
        assert!(!cfg.enabled);
        assert_eq!(cfg.bootstrap_timeout_s, 30);
    }

    #[tokio::test]
    async fn test_tor_config_disabled_noop() {
        let transport = TorTransport::new(TorConfig::default());
        assert!(!transport.is_available());
        transport.bootstrap().await.unwrap();
        assert!(!transport.is_available());
    }

    #[tokio::test]
    async fn test_tor_transport_fallback_on_failure() {
        let cfg = TorConfig {
            enabled: true,
            bootstrap_timeout_s: 1,
        };
        let transport = TorTransport::new(cfg);
        transport.bootstrap().await.unwrap();
        // Without the `tor` feature, bootstrap is a no-op warning.
        // With the feature but no network, Tor times out and falls
        // back gracefully. Either way: no panic, no error.
        #[cfg(not(feature = "tor"))]
        assert!(!transport.is_available());
    }
}
