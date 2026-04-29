// SPDX-License-Identifier: AGPL-3.0-or-later
//! Redundancy voting — majority hash comparison for multi-worker
//! task verification (Sprint 40 Phase C, port of redundancy.py S23).

use std::collections::HashMap;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteVerdict {
    Majority,
    Mismatch,
}

#[derive(Debug, Clone)]
pub struct VoteOutcome {
    pub verdict: VoteVerdict,
    pub canonical_hash: Option<String>,
    pub all_hashes: HashMap<String, String>,
    pub outlier_worker_ids: Vec<String>,
}

pub fn hash_result_bytes(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex::encode(digest)
}

pub struct RedundancyDispatcher {
    factors: HashMap<String, usize>,
    results: HashMap<String, Vec<(String, String)>>,
    quarantined: HashMap<String, Vec<String>>,
}

impl RedundancyDispatcher {
    pub fn new() -> Self {
        Self {
            factors: HashMap::new(),
            results: HashMap::new(),
            quarantined: HashMap::new(),
        }
    }

    pub fn register_task(&mut self, task_id: &str, factor: usize) {
        self.factors.insert(task_id.to_string(), factor);
        self.results.entry(task_id.to_string()).or_default();
    }

    pub fn is_redundant(&self, task_id: &str) -> bool {
        self.factors.get(task_id).copied().unwrap_or(1) > 1
    }

    pub fn collect_result(
        &mut self,
        task_id: &str,
        worker_id: &str,
        result_bytes: &[u8],
    ) -> Option<VoteOutcome> {
        let h = hash_result_bytes(result_bytes);
        let results = self.results.entry(task_id.to_string()).or_default();
        results.push((worker_id.to_string(), h));
        let factor = self.factors.get(task_id).copied().unwrap_or(1);
        if results.len() >= factor {
            return Some(self.vote(task_id));
        }
        None
    }

    pub fn vote(&self, task_id: &str) -> VoteOutcome {
        let results = match self.results.get(task_id) {
            Some(r) if !r.is_empty() => r,
            _ => {
                return VoteOutcome {
                    verdict: VoteVerdict::Mismatch,
                    canonical_hash: None,
                    all_hashes: HashMap::new(),
                    outlier_worker_ids: Vec::new(),
                }
            }
        };

        let mut hash_counts: HashMap<&str, usize> = HashMap::new();
        for (_, h) in results {
            *hash_counts.entry(h.as_str()).or_insert(0) += 1;
        }

        let all_hashes: HashMap<String, String> = results
            .iter()
            .map(|(wid, h)| (wid.clone(), h.clone()))
            .collect();

        let total = results.len();
        let majority_threshold = total / 2 + 1;

        let (best_hash, best_count) = hash_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&h, &c)| (h, c))
            .unwrap();

        if best_count >= majority_threshold {
            let outliers: Vec<String> = results
                .iter()
                .filter(|(_, h)| h != best_hash)
                .map(|(wid, _)| wid.clone())
                .collect();
            VoteOutcome {
                verdict: VoteVerdict::Majority,
                canonical_hash: Some(best_hash.to_string()),
                all_hashes,
                outlier_worker_ids: outliers,
            }
        } else {
            let all_workers: Vec<String> = results.iter().map(|(wid, _)| wid.clone()).collect();
            VoteOutcome {
                verdict: VoteVerdict::Mismatch,
                canonical_hash: None,
                all_hashes,
                outlier_worker_ids: all_workers,
            }
        }
    }

    pub fn quarantine_outliers(&mut self, task_id: &str, worker_ids: &[String]) {
        let q = self.quarantined.entry(task_id.to_string()).or_default();
        for wid in worker_ids {
            q.push(wid.clone());
        }
    }
}

impl Default for RedundancyDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_majority_3_workers() {
        let mut rd = RedundancyDispatcher::new();
        rd.register_task("t1", 3);
        rd.collect_result("t1", "w1", b"same result");
        rd.collect_result("t1", "w2", b"same result");
        let outcome = rd.collect_result("t1", "w3", b"different").unwrap();
        assert_eq!(outcome.verdict, VoteVerdict::Majority);
        assert!(outcome.canonical_hash.is_some());
        assert_eq!(outcome.outlier_worker_ids, vec!["w3"]);
    }

    #[test]
    fn vote_mismatch_all_different() {
        let mut rd = RedundancyDispatcher::new();
        rd.register_task("t1", 3);
        rd.collect_result("t1", "w1", b"result A");
        rd.collect_result("t1", "w2", b"result B");
        let outcome = rd.collect_result("t1", "w3", b"result C").unwrap();
        assert_eq!(outcome.verdict, VoteVerdict::Mismatch);
        assert_eq!(outcome.outlier_worker_ids.len(), 3);
    }

    #[test]
    fn vote_pending_not_enough_results() {
        let mut rd = RedundancyDispatcher::new();
        rd.register_task("t1", 3);
        assert!(rd.collect_result("t1", "w1", b"data").is_none());
        assert!(rd.collect_result("t1", "w2", b"data").is_none());
    }
}
