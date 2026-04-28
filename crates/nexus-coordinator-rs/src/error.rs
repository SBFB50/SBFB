// SPDX-License-Identifier: AGPL-3.0-or-later
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("migration error: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("task not found: {0}")]
    TaskNotFound(String),
}
