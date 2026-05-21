// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 68 Phase B — ephemeral preview store.
//!
//! Holds zip archives uploaded by `sbfb-factory preview` for a
//! limited time so the developer can test their app in the
//! browser via blob-serve before publishing.
//!
//! Entries are keyed by BLAKE3 hash of the raw bytes and evicted
//! after [`DEFAULT_TTL`]. The store is deliberately separate from
//! [`crate::blob_serve::BlobServeCache`] — previews are ephemeral
//! local uploads, not P2P blobs.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);
pub const MAX_PREVIEW_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug)]
struct PreviewEntry {
    data: Vec<u8>,
    created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct PreviewStore {
    inner: Arc<RwLock<HashMap<String, PreviewEntry>>>,
    ttl: Duration,
}

impl PreviewStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn load(&self, data: Vec<u8>) -> Result<String, PreviewError> {
        if data.len() > MAX_PREVIEW_BYTES {
            return Err(PreviewError::TooLarge {
                actual: data.len(),
                limit: MAX_PREVIEW_BYTES,
            });
        }
        let hash = blake3::hash(&data);
        let hash_hex = hash.to_hex().to_string();
        let entry = PreviewEntry {
            data,
            created_at: Instant::now(),
        };
        let mut guard = self.inner.write().map_err(|_| PreviewError::LockPoisoned)?;
        guard.insert(hash_hex.clone(), entry);
        Ok(hash_hex)
    }

    pub fn get(&self, hash: &str) -> Option<Vec<u8>> {
        let guard = self.inner.read().ok()?;
        let entry = guard.get(hash)?;
        if entry.created_at.elapsed() > self.ttl {
            return None;
        }
        Some(entry.data.clone())
    }

    pub fn has(&self, hash: &str) -> bool {
        let guard = match self.inner.read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        guard
            .get(hash)
            .map(|e| e.created_at.elapsed() <= self.ttl)
            .unwrap_or(false)
    }

    pub fn evict_expired(&self) {
        if let Ok(mut guard) = self.inner.write() {
            let ttl = self.ttl;
            guard.retain(|_, entry| entry.created_at.elapsed() <= ttl);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    #[error("preview size {actual} exceeds limit {limit}")]
    TooLarge { actual: usize, limit: usize },
    #[error("internal lock poisoned")]
    LockPoisoned,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_blake3_hash() {
        let store = PreviewStore::new(DEFAULT_TTL);
        let data = b"<html><body>hello</body></html>".to_vec();
        let expected = blake3::hash(&data).to_hex().to_string();
        let hash = store.load(data).unwrap();
        assert_eq!(hash, expected);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn get_returns_data_before_ttl() {
        let store = PreviewStore::new(DEFAULT_TTL);
        let data = b"preview content".to_vec();
        let hash = store.load(data.clone()).unwrap();
        let got = store.get(&hash).unwrap();
        assert_eq!(got, data);
    }

    #[test]
    fn get_returns_none_after_ttl() {
        let store = PreviewStore::new(Duration::from_millis(1));
        let data = b"ephemeral".to_vec();
        let hash = store.load(data).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(store.get(&hash).is_none());
    }

    #[test]
    fn rejects_oversized_upload() {
        let store = PreviewStore::new(DEFAULT_TTL);
        let data = vec![0u8; MAX_PREVIEW_BYTES + 1];
        let err = store.load(data).unwrap_err();
        assert!(matches!(err, PreviewError::TooLarge { .. }));
    }

    #[test]
    fn evict_expired_removes_stale_entries() {
        let store = PreviewStore::new(Duration::from_millis(1));
        let hash = store.load(b"stale".to_vec()).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        store.evict_expired();
        assert!(!store.has(&hash));
    }

    #[test]
    fn has_returns_false_for_unknown_hash() {
        let store = PreviewStore::new(DEFAULT_TTL);
        assert!(!store.has("0000000000000000000000000000000000000000000000000000000000000000"));
    }
}
