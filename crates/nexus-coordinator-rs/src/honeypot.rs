// SPDX-License-Identifier: AGPL-3.0-or-later
//! Honeypot canary peers — eclipse detection via ephemeral dummy
//! identities (Sprint 40 Phase C, port of honeypot.py S23).

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_CANARY_COUNT: usize = 5;
pub const DEFAULT_ROTATION_INTERVAL_S: f64 = 6.0 * 3600.0;
pub const ECLIPSE_CO_LOCATION_THRESHOLD: f64 = 0.80;
pub const ECLIPSE_CONSECUTIVE_ROTATIONS: usize = 3;

#[derive(Debug, Clone)]
pub struct CanaryPeer {
    pub public_key_hex: String,
    pub created_at: f64,
}

#[derive(Debug, Clone)]
pub struct EclipseAlert {
    pub worker_id: String,
    pub co_location_pct: f64,
    pub consecutive_rotations: usize,
    pub detected_at: f64,
}

pub struct CanaryPeerFactory;

impl CanaryPeerFactory {
    pub fn generate(count: usize) -> Vec<CanaryPeer> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        (0..count)
            .map(|_| {
                let kp = nexus_core_rs::crypto::KeyPair::generate();
                CanaryPeer {
                    public_key_hex: hex::encode(kp.public_bytes()),
                    created_at: now,
                }
            })
            .collect()
    }
}

pub struct EclipseDetector {
    threshold: f64,
    required_rotations: usize,
    streak: HashMap<String, usize>,
    current_sightings: HashMap<String, HashSet<String>>,
}

impl EclipseDetector {
    pub fn new(threshold: f64, required_rotations: usize) -> Self {
        Self {
            threshold,
            required_rotations,
            streak: HashMap::new(),
            current_sightings: HashMap::new(),
        }
    }

    pub fn report_neighborhood(
        &mut self,
        worker_id: &str,
        canary_pubkeys: &HashSet<String>,
    ) {
        let existing = self
            .current_sightings
            .entry(worker_id.to_string())
            .or_default();
        existing.extend(canary_pubkeys.iter().cloned());
    }

    pub fn evaluate(&mut self, canary_set: &[CanaryPeer]) -> Vec<EclipseAlert> {
        if canary_set.is_empty() {
            return Vec::new();
        }

        let canary_keys: HashSet<&str> = canary_set
            .iter()
            .map(|p| p.public_key_hex.as_str())
            .collect();
        let total = canary_keys.len();
        let mut alerts = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let mut workers_above = HashSet::new();
        for (worker_id, seen) in &self.current_sightings {
            let overlap = seen.iter().filter(|k| canary_keys.contains(k.as_str())).count();
            let pct = overlap as f64 / total as f64;
            if pct >= self.threshold {
                workers_above.insert(worker_id.clone());
                let streak = self.streak.entry(worker_id.clone()).or_insert(0);
                *streak += 1;
                if *streak >= self.required_rotations {
                    alerts.push(EclipseAlert {
                        worker_id: worker_id.clone(),
                        co_location_pct: pct,
                        consecutive_rotations: *streak,
                        detected_at: now,
                    });
                }
            }
        }

        self.streak
            .retain(|worker_id, _| workers_above.contains(worker_id));

        alerts
    }

    pub fn advance_rotation(&mut self) {
        self.current_sightings.clear();
    }

    pub fn streak(&self) -> &HashMap<String, usize> {
        &self.streak
    }
}

impl Default for EclipseDetector {
    fn default() -> Self {
        Self::new(ECLIPSE_CO_LOCATION_THRESHOLD, ECLIPSE_CONSECUTIVE_ROTATIONS)
    }
}

pub struct CanaryRotationScheduler {
    interval_s: f64,
    canary_count: usize,
    last_rotation: f64,
    current_canaries: Vec<CanaryPeer>,
    detector: EclipseDetector,
}

impl CanaryRotationScheduler {
    pub fn new(interval_s: f64, canary_count: usize) -> Self {
        Self {
            interval_s,
            canary_count,
            last_rotation: 0.0,
            current_canaries: Vec::new(),
            detector: EclipseDetector::default(),
        }
    }

    pub fn current_canaries(&self) -> &[CanaryPeer] {
        &self.current_canaries
    }

    pub fn detector(&self) -> &EclipseDetector {
        &self.detector
    }

    pub fn detector_mut(&mut self) -> &mut EclipseDetector {
        &mut self.detector
    }

    pub fn should_rotate(&self, now: f64) -> bool {
        (now - self.last_rotation) >= self.interval_s
    }

    pub fn rotate(&mut self, now: f64) -> Vec<EclipseAlert> {
        let alerts = self.detector.evaluate(&self.current_canaries);
        self.detector.advance_rotation();
        self.current_canaries = CanaryPeerFactory::generate(self.canary_count);
        self.last_rotation = now;
        alerts
    }
}

impl Default for CanaryRotationScheduler {
    fn default() -> Self {
        Self::new(DEFAULT_ROTATION_INTERVAL_S, DEFAULT_CANARY_COUNT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_generate_unique_keys() {
        let peers = CanaryPeerFactory::generate(5);
        assert_eq!(peers.len(), 5);
        let keys: HashSet<&str> = peers.iter().map(|p| p.public_key_hex.as_str()).collect();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn eclipse_alert_threshold() {
        let mut det = EclipseDetector::new(0.8, 3);
        let canaries = CanaryPeerFactory::generate(5);
        let all_keys: HashSet<String> = canaries
            .iter()
            .map(|c| c.public_key_hex.clone())
            .collect();

        for _ in 0..3 {
            det.report_neighborhood("w1", &all_keys);
            det.evaluate(&canaries);
            det.advance_rotation();
        }
        det.report_neighborhood("w1", &all_keys);
        let alerts = det.evaluate(&canaries);
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].worker_id, "w1");
    }

    #[test]
    fn eclipse_no_alert_below_threshold() {
        let mut det = EclipseDetector::new(0.8, 3);
        let canaries = CanaryPeerFactory::generate(10);
        let partial: HashSet<String> = canaries
            .iter()
            .take(3)
            .map(|c| c.public_key_hex.clone())
            .collect();

        det.report_neighborhood("w1", &partial);
        let alerts = det.evaluate(&canaries);
        assert!(alerts.is_empty());
    }

    #[test]
    fn rotation_generates_new_peers() {
        let mut sched = CanaryRotationScheduler::new(100.0, 3);
        sched.rotate(0.0);
        let first_keys: Vec<String> = sched
            .current_canaries()
            .iter()
            .map(|c| c.public_key_hex.clone())
            .collect();
        sched.rotate(100.0);
        let second_keys: Vec<String> = sched
            .current_canaries()
            .iter()
            .map(|c| c.public_key_hex.clone())
            .collect();
        assert_ne!(first_keys, second_keys);
    }
}
