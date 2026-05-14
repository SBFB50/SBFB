// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-author rate limiter for feed sync (remote entry ingestion).
//!
//! GCRA (Generic Cell Rate Algorithm) via the `governor` crate,
//! keyed by `author_pubkey` hex string.
//! Quota: 5 feed operations per minute per author.

use std::num::NonZeroU32;

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

pub const FEED_OPS_PER_MINUTE: u32 = 5;

pub struct FeedRateLimiter {
    limiter: DefaultKeyedRateLimiter<String>,
}

impl std::fmt::Debug for FeedRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeedRateLimiter")
            .field("quota", &format_args!("{}/min", FEED_OPS_PER_MINUTE))
            .finish()
    }
}

impl FeedRateLimiter {
    pub fn new() -> Self {
        let quota =
            Quota::per_minute(NonZeroU32::new(FEED_OPS_PER_MINUTE).expect("constant is non-zero"));
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    pub fn check_author(&self, author_pubkey: &str) -> bool {
        self.limiter.check_key(&author_pubkey.to_string()).is_ok()
    }

    pub fn retain_recent(&self) {
        self.limiter.retain_recent();
    }
}

impl Default for FeedRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_quota() {
        let limiter = FeedRateLimiter::new();
        for i in 0..FEED_OPS_PER_MINUTE {
            assert!(
                limiter.check_author("aaaa1111"),
                "op {i} should be allowed under quota"
            );
        }
    }

    #[test]
    fn test_feed_rate_limiter_rejects_excess() {
        let limiter = FeedRateLimiter::new();
        let mut rejected = 0;
        for _ in 0..10 {
            if !limiter.check_author("aaaa1111") {
                rejected += 1;
            }
        }
        assert!(
            rejected > 0,
            "at least some ops must be rejected over 5/min quota"
        );
    }

    #[test]
    fn independent_authors() {
        let limiter = FeedRateLimiter::new();
        for _ in 0..FEED_OPS_PER_MINUTE {
            assert!(limiter.check_author("author_a"));
        }
        assert!(
            !limiter.check_author("author_a"),
            "author_a exhausted, must reject"
        );
        for _ in 0..FEED_OPS_PER_MINUTE {
            assert!(
                limiter.check_author("author_b"),
                "author_b must be independent from author_a"
            );
        }
    }
}
