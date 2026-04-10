//! Error type for nexus-core-rs.
//!
//! A single `NexusError` enum covers every failure mode in this
//! crate. It is `Sync + Send + 'static` so it can cross async task
//! boundaries and be wrapped by Python exceptions in nexus-core-py.

use thiserror::Error;

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, NexusError>;

/// Errors that can arise from nexus-core-rs operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NexusError {
    /// The underlying iroh endpoint failed to start or bind.
    #[error("iroh endpoint error: {0}")]
    Endpoint(String),

    /// A peer discovery operation (pkarr publish/resolve) failed.
    #[error("discovery error: {0}")]
    Discovery(String),

    /// A document (iroh-docs) replica operation failed.
    #[error("docs error: {0}")]
    Docs(String),

    /// A gossip (iroh-gossip) topic operation failed.
    #[error("gossip error: {0}")]
    Gossip(String),

    /// A blob (iroh-blobs) transfer or pin operation failed.
    #[error("blobs error: {0}")]
    Blobs(String),

    /// Cryptographic signing/verification failed (Ed25519, BLAKE3).
    #[error("crypto error: {0}")]
    Crypto(String),

    /// An I/O error from tokio or the filesystem.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Any other error, used as an escape hatch for third-party error
    /// types that do not warrant their own variant yet.
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for NexusError {
    fn from(err: anyhow::Error) -> Self {
        NexusError::Other(err.to_string())
    }
}
