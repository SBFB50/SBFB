// SPDX-License-Identifier: AGPL-3.0-or-later
//! Re-run sampling — spot-check re-dispatch for compute theft
//! detection (Sprint 40 Phase C, port of rerun.py S24).

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RerunConfig {
    pub sample_rate: f64,
}

impl RerunConfig {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate: sample_rate.clamp(0.0, 1.0),
        }
    }
}

impl Default for RerunConfig {
    fn default() -> Self {
        Self { sample_rate: 0.01 }
    }
}

pub struct RerunSampler {
    sample_rate: f64,
    rerun_map: HashMap<String, String>,
}

impl RerunSampler {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate: sample_rate.clamp(0.0, 1.0),
            rerun_map: HashMap::new(),
        }
    }

    pub fn should_rerun(&self, task_id: &str) -> bool {
        if self.is_rerun(task_id) {
            return false;
        }
        let hash = simple_hash(task_id);
        (hash as f64 / u64::MAX as f64) < self.sample_rate
    }

    pub fn register_rerun(&mut self, original_task_id: &str, rerun_task_id: &str) {
        self.rerun_map
            .insert(rerun_task_id.to_string(), original_task_id.to_string());
    }

    pub fn is_rerun(&self, task_id: &str) -> bool {
        self.rerun_map.contains_key(task_id)
    }

    pub fn get_original(&self, rerun_task_id: &str) -> Option<&str> {
        self.rerun_map.get(rerun_task_id).map(|s| s.as_str())
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

pub struct DivergenceScorer;

impl DivergenceScorer {
    pub fn score(original_hash: &[u8], rerun_hash: &[u8]) -> f64 {
        if original_hash == rerun_hash {
            0.0
        } else {
            1.0
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    let hash = blake3::hash(s.as_bytes());
    let bytes = hash.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_anti_loop_rerun_of_rerun() {
        let mut sampler = RerunSampler::new(1.0);
        sampler.register_rerun("original-1", "rerun-1");
        assert!(!sampler.should_rerun("rerun-1"));
    }

    #[test]
    fn sampler_rate_zero_never_reruns() {
        let sampler = RerunSampler::new(0.0);
        for i in 0..100 {
            assert!(!sampler.should_rerun(&format!("task-{i}")));
        }
    }

    #[test]
    fn sampler_get_original() {
        let mut sampler = RerunSampler::new(0.5);
        sampler.register_rerun("orig", "rerun-abc");
        assert_eq!(sampler.get_original("rerun-abc"), Some("orig"));
        assert_eq!(sampler.get_original("unknown"), None);
    }

    #[test]
    fn divergence_scorer_match() {
        let hash = b"same-hash-value";
        assert!((DivergenceScorer::score(hash, hash) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn divergence_scorer_mismatch() {
        assert!((DivergenceScorer::score(b"hash-a", b"hash-b") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simple_hash_deterministic() {
        let a = simple_hash("task-abc-123");
        let b = simple_hash("task-abc-123");
        assert_eq!(a, b);
        let c = simple_hash("task-abc-124");
        assert_ne!(a, c);
    }
}
