// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-peer rate limiter for `browse_request` gossip messages.
//!
//! GCRA (Generic Cell Rate Algorithm) via the `governor` crate,
//! keyed by NodeId hex string. Quota: 10 browse requests per minute
//! per peer. Defends against browse_request spam that would force
//! the daemon to replay its entire outbox on every incoming request.

use std::num::NonZeroU32;

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

const BROWSE_REQUESTS_PER_MINUTE: u32 = 10;

pub struct BrowseRequestLimiter {
    limiter: DefaultKeyedRateLimiter<String>,
}

impl BrowseRequestLimiter {
    pub fn new() -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(BROWSE_REQUESTS_PER_MINUTE).expect("constant is non-zero"),
        );
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    /// Returns `true` if the peer is within its browse_request quota.
    pub fn check_peer(&self, peer_id: &str) -> bool {
        self.limiter.check_key(&peer_id.to_string()).is_ok()
    }

    pub fn retain_recent(&self) {
        self.limiter.retain_recent();
    }
}

impl Default for BrowseRequestLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_quota() {
        let limiter = BrowseRequestLimiter::new();
        let peer = "peer_aabbccdd";
        for i in 0..5 {
            assert!(
                limiter.check_peer(peer),
                "request {i} should be allowed under quota"
            );
        }
    }

    #[test]
    fn rejects_over_quota() {
        let limiter = BrowseRequestLimiter::new();
        let peer = "peer_spammer";
        let mut rejected = 0;
        for _ in 0..15 {
            if !limiter.check_peer(peer) {
                rejected += 1;
            }
        }
        assert!(
            rejected > 0,
            "at least some requests must be rejected over quota"
        );
    }

    #[test]
    fn independent_peers() {
        let limiter = BrowseRequestLimiter::new();
        let peer_a = "peer_aaaa";
        let peer_b = "peer_bbbb";

        for _ in 0..BROWSE_REQUESTS_PER_MINUTE {
            assert!(limiter.check_peer(peer_a));
        }
        assert!(!limiter.check_peer(peer_a), "peer_a exhausted, must reject");

        for _ in 0..5 {
            assert!(
                limiter.check_peer(peer_b),
                "peer_b must be independent from peer_a"
            );
        }
    }

    #[tokio::test]
    async fn quota_recovers() {
        let limiter = BrowseRequestLimiter::new();
        let peer = "peer_recovery";

        for _ in 0..BROWSE_REQUESTS_PER_MINUTE {
            limiter.check_peer(peer);
        }
        assert!(!limiter.check_peer(peer), "must be exhausted");

        tokio::time::sleep(std::time::Duration::from_secs(7)).await;

        assert!(limiter.check_peer(peer), "quota must recover after waiting");
    }
}
