// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-author per-app rate limiter for storage write endpoints.
//!
//! GCRA (Generic Cell Rate Algorithm) via the `governor` crate,
//! keyed by composite `{author}:{app_name}` string.
//! Quota: 10 writes per minute per author per app.

use std::num::NonZeroU32;

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

pub const STORAGE_WRITES_PER_MINUTE: u32 = 10;

pub struct StorageWriteLimiter {
    limiter: DefaultKeyedRateLimiter<String>,
}

impl std::fmt::Debug for StorageWriteLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageWriteLimiter")
            .field("quota", &format_args!("{}/min", STORAGE_WRITES_PER_MINUTE))
            .finish()
    }
}

impl StorageWriteLimiter {
    pub fn new() -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(STORAGE_WRITES_PER_MINUTE).expect("constant is non-zero"),
        );
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    /// Returns `true` if the author is within their write quota for the given app.
    pub fn check_write(&self, author: &str, app: &str) -> bool {
        let key = format!("{author}:{app}");
        self.limiter.check_key(&key).is_ok()
    }

    pub fn retain_recent(&self) {
        self.limiter.retain_recent();
    }
}

impl Default for StorageWriteLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_quota() {
        let limiter = StorageWriteLimiter::new();
        for i in 0..5 {
            assert!(
                limiter.check_write("node_abc", "sbfb-ideas"),
                "write {i} should be allowed under quota"
            );
        }
    }

    #[test]
    fn rejects_over_quota() {
        let limiter = StorageWriteLimiter::new();
        let mut rejected = 0;
        for _ in 0..15 {
            if !limiter.check_write("node_abc", "sbfb-ideas") {
                rejected += 1;
            }
        }
        assert!(
            rejected > 0,
            "at least some writes must be rejected over quota"
        );
    }

    #[test]
    fn independent_apps() {
        let limiter = StorageWriteLimiter::new();
        let author = "node_abc";

        for _ in 0..STORAGE_WRITES_PER_MINUTE {
            assert!(limiter.check_write(author, "app-a"));
        }
        assert!(
            !limiter.check_write(author, "app-a"),
            "app-a exhausted, must reject"
        );

        for _ in 0..5 {
            assert!(
                limiter.check_write(author, "app-b"),
                "app-b must be independent from app-a"
            );
        }
    }

    #[test]
    fn independent_authors() {
        let limiter = StorageWriteLimiter::new();
        let app = "sbfb-ideas";

        for _ in 0..STORAGE_WRITES_PER_MINUTE {
            assert!(limiter.check_write("author_a", app));
        }
        assert!(
            !limiter.check_write("author_a", app),
            "author_a exhausted, must reject"
        );

        for _ in 0..5 {
            assert!(
                limiter.check_write("author_b", app),
                "author_b must be independent from author_a"
            );
        }
    }
}
