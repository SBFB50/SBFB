// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canary input injection — known-answer probe system for compute
//! theft detection (Sprint 40 Phase B, port of canary_input.py S22).

use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use serde::{Deserialize, Serialize};

pub const CANARY_INPUT_SET_VERSION: u32 = 1;
pub const DEFAULT_INJECT_RATE: usize = 100;
pub const DEFAULT_TOLERANCE: f64 = 0.85;
pub const DEFAULT_ROTATION_FREQUENCY_DAYS: u32 = 30;
const RING_CAPACITY: usize = 100;
const MTIME_DEBOUNCE_SECS: f64 = 0.05;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanaryPrompt {
    pub prompt_id: String,
    pub prompt: String,
    pub expected_answer: String,
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_tolerance() -> f64 {
    DEFAULT_TOLERANCE
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryInputSet {
    #[serde(default = "default_version")]
    pub version: u32,
    pub created_at_unix: i64,
    pub prompts: Vec<CanaryPrompt>,
    pub coord_pubkey_hex: String,
    pub signature_hex: String,
}

fn default_version() -> u32 {
    CANARY_INPUT_SET_VERSION
}

impl CanaryInputSet {
    pub fn signable_json(&self) -> String {
        let payload = serde_json::json!({
            "created_at_unix": self.created_at_unix,
            "prompts": self.prompts.iter().map(|p| {
                serde_json::json!({
                    "expected_answer": p.expected_answer,
                    "prompt": p.prompt,
                    "prompt_id": p.prompt_id,
                    "tolerance": p.tolerance,
                })
            }).collect::<Vec<_>>(),
            "version": self.version,
        });
        serde_json::to_string(&payload).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DivergenceRecord {
    pub prompt_id: String,
    pub observed_at_unix: i64,
    pub similarity: f64,
    pub expected_answer: String,
    pub observed_answer: String,
    pub worker_pubkey_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryInputPolicy {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_inject_rate")]
    pub inject_rate: usize,
    #[serde(default = "default_tolerance")]
    pub default_tolerance: f64,
    #[serde(default = "default_rotation_days")]
    pub rotation_frequency_days: u32,
    pub set_path: Option<String>,
}

fn default_true() -> bool {
    true
}
fn default_inject_rate() -> usize {
    DEFAULT_INJECT_RATE
}
fn default_rotation_days() -> u32 {
    DEFAULT_ROTATION_FREQUENCY_DAYS
}

impl Default for CanaryInputPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            inject_rate: DEFAULT_INJECT_RATE,
            default_tolerance: DEFAULT_TOLERANCE,
            rotation_frequency_days: DEFAULT_ROTATION_FREQUENCY_DAYS,
            set_path: None,
        }
    }
}

impl CanaryInputPolicy {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(default)]
            default: Option<CanaryInputPolicy>,
        }
        let wrapper: Wrapper =
            toml::from_str(text).map_err(|e| format!("TOML parse error: {e}"))?;
        Ok(wrapper.default.unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// Sign + verify helpers
// ---------------------------------------------------------------------------

pub fn build_canary_input_set(
    prompts: Vec<CanaryPrompt>,
    keypair: &nexus_core_rs::crypto::KeyPair,
) -> CanaryInputSet {
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let mut set = CanaryInputSet {
        version: CANARY_INPUT_SET_VERSION,
        created_at_unix: created,
        prompts,
        coord_pubkey_hex: hex::encode(keypair.public_bytes()),
        signature_hex: String::new(),
    };
    let sig = keypair.sign(set.signable_json().as_bytes());
    set.signature_hex = hex::encode(sig);
    set
}

pub fn verify_canary_input_set(
    set: &CanaryInputSet,
    expected_pubkey: Option<&[u8; 32]>,
) -> Result<(), String> {
    if set.version != CANARY_INPUT_SET_VERSION {
        return Err(format!(
            "unsupported canary_input_set version {}, expected {}",
            set.version, CANARY_INPUT_SET_VERSION
        ));
    }
    let sig_bytes: [u8; 64] = hex::decode(&set.signature_hex)
        .map_err(|e| format!("bad signature hex: {e}"))?
        .try_into()
        .map_err(|_| "signature must be 64 bytes".to_string())?;
    let pubkey_bytes: [u8; 32] = hex::decode(&set.coord_pubkey_hex)
        .map_err(|e| format!("bad pubkey hex: {e}"))?
        .try_into()
        .map_err(|_| "pubkey must be 32 bytes".to_string())?;
    if let Some(expected) = expected_pubkey {
        if &pubkey_bytes != expected {
            return Err("canary_input_set signed by unexpected pubkey".into());
        }
    }
    nexus_core_rs::crypto::verify(&pubkey_bytes, set.signable_json().as_bytes(), &sig_bytes)
        .map_err(|e| format!("signature verification failed: {e}"))
}

pub fn save_canary_input_set(set: &CanaryInputSet, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(set).map_err(io::Error::other)?;
    std::fs::write(path, json)
}

pub fn load_canary_input_set(
    path: &Path,
    expected_pubkey: Option<&[u8; 32]>,
) -> Result<CanaryInputSet, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;
    let set: CanaryInputSet =
        serde_json::from_str(&text).map_err(|e| format!("JSON parse failed: {e}"))?;
    verify_canary_input_set(&set, expected_pubkey)?;
    Ok(set)
}

// ---------------------------------------------------------------------------
// Injector
// ---------------------------------------------------------------------------

pub struct CanaryInputInjector {
    canary_set: Mutex<Option<CanaryInputSet>>,
    inject_rate: AtomicUsize,
    rr_index: AtomicUsize,
    seen_count: AtomicUsize,
    injected_count: AtomicUsize,
}

impl CanaryInputInjector {
    pub fn new(canary_set: Option<CanaryInputSet>, inject_rate: usize) -> Self {
        Self {
            canary_set: Mutex::new(canary_set),
            inject_rate: AtomicUsize::new(inject_rate.max(1)),
            rr_index: AtomicUsize::new(0),
            seen_count: AtomicUsize::new(0),
            injected_count: AtomicUsize::new(0),
        }
    }

    pub fn should_inject(&self) -> bool {
        self.seen_count.fetch_add(1, Ordering::Relaxed);
        let guard = self.canary_set.lock().unwrap_or_else(|p| p.into_inner());
        if guard.as_ref().map_or(true, |s| s.prompts.is_empty()) {
            return false;
        }
        drop(guard);
        let rate = self.inject_rate.load(Ordering::Relaxed);
        if rate <= 1 {
            self.injected_count.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        let draw = rand::thread_rng().gen_range(1..=rate);
        if draw == 1 {
            self.injected_count.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    pub fn next_prompt(&self) -> Option<CanaryPrompt> {
        let guard = self.canary_set.lock().unwrap_or_else(|p| p.into_inner());
        let set = guard.as_ref()?;
        if set.prompts.is_empty() {
            return None;
        }
        let idx = self.rr_index.fetch_add(1, Ordering::Relaxed) % set.prompts.len();
        Some(set.prompts[idx].clone())
    }

    pub fn set_canary_set(&self, new_set: Option<CanaryInputSet>) {
        let mut guard = self.canary_set.lock().unwrap_or_else(|p| p.into_inner());
        *guard = new_set;
        self.rr_index.store(0, Ordering::Relaxed);
    }

    pub fn set_inject_rate(&self, new_rate: usize) {
        self.inject_rate.store(new_rate.max(1), Ordering::Relaxed);
    }

    pub fn stats(&self) -> (usize, usize) {
        (
            self.seen_count.load(Ordering::Relaxed),
            self.injected_count.load(Ordering::Relaxed),
        )
    }
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

pub struct CanaryInputObserver {
    canary_set: Mutex<Option<CanaryInputSet>>,
    default_tolerance: f64,
    ring_capacity: usize,
    ring: Mutex<VecDeque<DivergenceRecord>>,
    observed_count: AtomicUsize,
    alerts_count: AtomicUsize,
}

impl CanaryInputObserver {
    pub fn new(
        canary_set: Option<CanaryInputSet>,
        default_tolerance: f64,
        ring_capacity: usize,
    ) -> Self {
        Self {
            canary_set: Mutex::new(canary_set),
            default_tolerance,
            ring_capacity: ring_capacity.max(1),
            ring: Mutex::new(VecDeque::new()),
            observed_count: AtomicUsize::new(0),
            alerts_count: AtomicUsize::new(0),
        }
    }

    pub fn observe(
        &self,
        prompt_id: &str,
        observed_answer: &str,
        worker_pubkey_hex: Option<&str>,
    ) -> bool {
        self.observed_count.fetch_add(1, Ordering::Relaxed);
        let guard = self.canary_set.lock().unwrap_or_else(|p| p.into_inner());
        let set = match guard.as_ref() {
            Some(s) => s,
            None => return false,
        };
        let prompt = match set.prompts.iter().find(|p| p.prompt_id == prompt_id) {
            Some(p) => p,
            None => return false,
        };
        let tolerance = if prompt.tolerance > 0.0 {
            prompt.tolerance
        } else {
            self.default_tolerance
        };
        let similarity = strsim::normalized_levenshtein(&prompt.expected_answer, observed_answer);
        if similarity >= tolerance {
            return false;
        }
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let record = DivergenceRecord {
            prompt_id: prompt_id.to_string(),
            observed_at_unix: ts,
            similarity,
            expected_answer: prompt.expected_answer.clone(),
            observed_answer: observed_answer.to_string(),
            worker_pubkey_hex: worker_pubkey_hex.map(String::from),
        };
        drop(guard);
        let mut ring = self.ring.lock().unwrap_or_else(|p| p.into_inner());
        ring.push_back(record);
        while ring.len() > self.ring_capacity {
            ring.pop_front();
        }
        self.alerts_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn recent_divergences(&self, limit: usize) -> Vec<DivergenceRecord> {
        let ring = self.ring.lock().unwrap_or_else(|p| p.into_inner());
        ring.iter().rev().take(limit).cloned().collect()
    }

    pub fn set_canary_set(&self, new_set: Option<CanaryInputSet>) {
        let mut guard = self.canary_set.lock().unwrap_or_else(|p| p.into_inner());
        *guard = new_set;
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let ring = self.ring.lock().unwrap_or_else(|p| p.into_inner());
        (
            self.observed_count.load(Ordering::Relaxed),
            self.alerts_count.load(Ordering::Relaxed),
            ring.len(),
        )
    }
}

// ---------------------------------------------------------------------------
// Manager (policy hot-reload + signed set reload)
// ---------------------------------------------------------------------------

pub struct CanaryInputManager {
    policy_path: Option<PathBuf>,
    canary_set_path: Option<PathBuf>,
    coord_pubkey: Option<[u8; 32]>,
    policy: Mutex<CanaryInputPolicy>,
    injector: CanaryInputInjector,
    observer: CanaryInputObserver,
    reload_policy_mtime: Mutex<Option<f64>>,
    reload_set_mtime: Mutex<Option<f64>>,
    last_reload_check: Mutex<f64>,
}

impl CanaryInputManager {
    pub fn new(
        policy_path: Option<PathBuf>,
        canary_set_path: Option<PathBuf>,
        coord_pubkey: Option<[u8; 32]>,
    ) -> Self {
        let mut policy = CanaryInputPolicy::default();
        let mut policy_mtime: Option<f64> = None;

        if let Some(ref pp) = policy_path {
            if pp.exists() {
                if let Ok(text) = std::fs::read_to_string(pp) {
                    if let Ok(p) = CanaryInputPolicy::from_toml(&text) {
                        policy = p;
                    }
                }
                policy_mtime = pp
                    .metadata()
                    .ok()
                    .and_then(|m| {
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    })
                    .map(|d| d.as_secs_f64());
            }
        }

        let initial_set = Self::try_load_set(&canary_set_path, &policy, coord_pubkey.as_ref());

        let injector = CanaryInputInjector::new(initial_set.clone(), policy.inject_rate);
        let observer =
            CanaryInputObserver::new(initial_set, policy.default_tolerance, RING_CAPACITY);

        Self {
            policy_path,
            canary_set_path,
            coord_pubkey,
            policy: Mutex::new(policy),
            injector,
            observer,
            reload_policy_mtime: Mutex::new(policy_mtime),
            reload_set_mtime: Mutex::new(None),
            last_reload_check: Mutex::new(0.0),
        }
    }

    pub fn policy(&self) -> CanaryInputPolicy {
        self.policy
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub fn maybe_inject(&self) -> Option<CanaryPrompt> {
        self.maybe_reload();
        let pol = self.policy.lock().unwrap_or_else(|p| p.into_inner());
        if !pol.enabled {
            return None;
        }
        drop(pol);
        if !self.injector.should_inject() {
            return None;
        }
        self.injector.next_prompt()
    }

    pub fn observe_result(
        &self,
        prompt_id: &str,
        observed_answer: &str,
        worker_pubkey_hex: Option<&str>,
    ) -> bool {
        self.maybe_reload();
        self.observer
            .observe(prompt_id, observed_answer, worker_pubkey_hex)
    }

    fn effective_set_path(&self) -> Option<PathBuf> {
        if self.canary_set_path.is_some() {
            return self.canary_set_path.clone();
        }
        let pol = self.policy.lock().unwrap_or_else(|p| p.into_inner());
        pol.set_path.as_ref().map(PathBuf::from)
    }

    fn maybe_reload(&self) {
        let now = monotonic_secs();
        {
            let mut last = self
                .last_reload_check
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if now - *last < MTIME_DEBOUNCE_SECS {
                return;
            }
            *last = now;
        }
        self.reload_policy();
        self.reload_set();
    }

    fn reload_policy(&self) {
        let pp = match &self.policy_path {
            Some(p) => p,
            None => return,
        };
        let mtime = match file_mtime(pp) {
            Some(m) => m,
            None => return,
        };
        {
            let cached = self
                .reload_policy_mtime
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if cached.is_some_and(|c| mtime <= c) {
                return;
            }
        }
        let text = match std::fs::read_to_string(pp) {
            Ok(t) => t,
            Err(_) => return,
        };
        let new_policy = match CanaryInputPolicy::from_toml(&text) {
            Ok(p) => p,
            Err(_) => return,
        };
        self.injector.set_inject_rate(new_policy.inject_rate);
        *self.policy.lock().unwrap_or_else(|p| p.into_inner()) = new_policy;
        *self
            .reload_policy_mtime
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(mtime);
    }

    fn reload_set(&self) {
        let target = match self.effective_set_path() {
            Some(p) => p,
            None => return,
        };
        let mtime = match file_mtime(&target) {
            Some(m) => m,
            None => return,
        };
        {
            let cached = self
                .reload_set_mtime
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if cached.is_some_and(|c| mtime <= c) {
                return;
            }
        }
        let new_set = match load_canary_input_set(&target, self.coord_pubkey.as_ref()) {
            Ok(s) => s,
            Err(_) => return,
        };
        self.injector.set_canary_set(Some(new_set.clone()));
        self.observer.set_canary_set(Some(new_set));
        *self
            .reload_set_mtime
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(mtime);
    }

    fn try_load_set(
        path: &Option<PathBuf>,
        policy: &CanaryInputPolicy,
        pubkey: Option<&[u8; 32]>,
    ) -> Option<CanaryInputSet> {
        let target = if let Some(p) = path.as_ref() {
            p.clone()
        } else {
            PathBuf::from(policy.set_path.as_ref()?)
        };
        if !target.exists() {
            return None;
        }
        load_canary_input_set(&target, pubkey).ok()
    }
}

fn file_mtime(path: &Path) -> Option<f64> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs_f64())
}

fn monotonic_secs() -> f64 {
    use std::time::Instant;
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_secs_f64()
}

// ---------------------------------------------------------------------------
// Guardrail adapter
// ---------------------------------------------------------------------------

pub struct CanaryInputGuardrail {
    injector: std::sync::Arc<CanaryInputInjector>,
}

impl CanaryInputGuardrail {
    pub fn new(injector: std::sync::Arc<CanaryInputInjector>) -> Self {
        Self { injector }
    }
}

impl crate::guardrails::Guardrail for CanaryInputGuardrail {
    fn name(&self) -> &str {
        "canary_input"
    }

    fn direction(&self) -> crate::guardrails::GuardrailDirection {
        crate::guardrails::GuardrailDirection::Input
    }

    fn check(
        &self,
        _ctx: &crate::guardrails::GuardrailContext<'_>,
    ) -> crate::guardrails::GuardrailOutcome {
        if self.injector.should_inject() && self.injector.next_prompt().is_some() {
            return crate::guardrails::GuardrailOutcome::Tripwire {
                reason: "canary_input_injected".into(),
            };
        }
        crate::guardrails::GuardrailOutcome::Pass
    }
}

// ---------------------------------------------------------------------------
// Seed probes
// ---------------------------------------------------------------------------

pub const DEFAULT_SEED_PROMPTS: &[(&str, &str, &str)] = &[
    (
        "canary.arith.01",
        "What is 17 plus 42? Answer with a number only.",
        "59",
    ),
    (
        "canary.arith.02",
        "What is 8 times 9? Answer with a number only.",
        "72",
    ),
    (
        "canary.fact.01",
        "What is the chemical symbol for gold? Answer with the symbol only.",
        "Au",
    ),
    (
        "canary.fact.02",
        "How many continents are there? Answer with a number only.",
        "7",
    ),
    (
        "canary.geo.01",
        "What is the capital of France? Answer with the city name only.",
        "Paris",
    ),
];

pub fn seed_prompts() -> Vec<CanaryPrompt> {
    DEFAULT_SEED_PROMPTS
        .iter()
        .map(|(id, prompt, answer)| CanaryPrompt {
            prompt_id: id.to_string(),
            prompt: prompt.to_string(),
            expected_answer: answer.to_string(),
            tolerance: DEFAULT_TOLERANCE,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keypair() -> nexus_core_rs::crypto::KeyPair {
        nexus_core_rs::crypto::KeyPair::generate()
    }

    fn make_prompts() -> Vec<CanaryPrompt> {
        vec![
            CanaryPrompt {
                prompt_id: "t1".into(),
                prompt: "What is 2+2?".into(),
                expected_answer: "4".into(),
                tolerance: DEFAULT_TOLERANCE,
            },
            CanaryPrompt {
                prompt_id: "t2".into(),
                prompt: "Capital of France?".into(),
                expected_answer: "Paris".into(),
                tolerance: DEFAULT_TOLERANCE,
            },
        ]
    }

    #[test]
    fn canary_prompt_serde_roundtrip() {
        let p = CanaryPrompt {
            prompt_id: "test".into(),
            prompt: "question".into(),
            expected_answer: "answer".into(),
            tolerance: 0.9,
        };
        let json = serde_json::to_string(&p).unwrap();
        let p2: CanaryPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn canary_input_set_sign_verify() {
        let kp = make_keypair();
        let set = build_canary_input_set(make_prompts(), &kp);
        assert_eq!(set.version, CANARY_INPUT_SET_VERSION);
        assert_eq!(set.prompts.len(), 2);
        assert!(verify_canary_input_set(&set, None).is_ok());
        assert!(verify_canary_input_set(&set, Some(&kp.public_bytes())).is_ok());
    }

    #[test]
    fn canary_input_set_tampered_fails() {
        let kp = make_keypair();
        let mut set = build_canary_input_set(make_prompts(), &kp);
        set.prompts[0].expected_answer = "tampered".into();
        assert!(verify_canary_input_set(&set, None).is_err());
    }

    #[test]
    fn canary_input_set_wrong_pubkey_fails() {
        let kp = make_keypair();
        let set = build_canary_input_set(make_prompts(), &kp);
        let other = make_keypair();
        assert!(verify_canary_input_set(&set, Some(&other.public_bytes())).is_err());
    }

    #[test]
    fn canary_input_set_save_load() {
        let kp = make_keypair();
        let set = build_canary_input_set(make_prompts(), &kp);
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("canary_set.json");
        save_canary_input_set(&set, &path).unwrap();
        let loaded = load_canary_input_set(&path, Some(&kp.public_bytes())).unwrap();
        assert_eq!(loaded.prompts.len(), 2);
        assert_eq!(loaded.coord_pubkey_hex, set.coord_pubkey_hex);
    }

    #[test]
    fn injector_rate_always() {
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let inj = CanaryInputInjector::new(Some(set), 1);
        assert!(inj.should_inject());
        assert!(inj.should_inject());
    }

    #[test]
    fn injector_round_robin() {
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let inj = CanaryInputInjector::new(Some(set), 1);
        let p1 = inj.next_prompt().unwrap();
        let p2 = inj.next_prompt().unwrap();
        let p3 = inj.next_prompt().unwrap();
        assert_eq!(p1.prompt_id, "t1");
        assert_eq!(p2.prompt_id, "t2");
        assert_eq!(p3.prompt_id, "t1");
    }

    #[test]
    fn observer_divergence_below_tolerance() {
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let obs = CanaryInputObserver::new(Some(set), DEFAULT_TOLERANCE, RING_CAPACITY);
        let diverged = obs.observe("t1", "completely wrong answer", None);
        assert!(diverged);
        assert_eq!(obs.recent_divergences(10).len(), 1);
    }

    #[test]
    fn observer_no_divergence_above_tolerance() {
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let obs = CanaryInputObserver::new(Some(set), DEFAULT_TOLERANCE, RING_CAPACITY);
        let diverged = obs.observe("t1", "4", None);
        assert!(!diverged);
        assert_eq!(obs.recent_divergences(10).len(), 0);
    }

    #[test]
    fn observer_ring_buffer_bounded() {
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let obs = CanaryInputObserver::new(Some(set), DEFAULT_TOLERANCE, 3);
        for _ in 0..10 {
            obs.observe("t1", "wrong", None);
        }
        let (_, alerts, ring_size) = obs.stats();
        assert_eq!(alerts, 10);
        assert_eq!(ring_size, 3);
    }

    #[test]
    fn policy_from_toml() {
        let toml = r#"
[default]
enabled = true
inject_rate = 50
default_tolerance = 0.9
"#;
        let policy = CanaryInputPolicy::from_toml(toml).unwrap();
        assert_eq!(policy.inject_rate, 50);
        assert!((policy.default_tolerance - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn default_seed_prompts_count() {
        assert_eq!(DEFAULT_SEED_PROMPTS.len(), 5);
        let prompts = seed_prompts();
        assert_eq!(prompts.len(), 5);
        assert_eq!(prompts[0].prompt_id, "canary.arith.01");
    }

    #[test]
    fn guardrail_tripwire_on_inject() {
        use crate::guardrails::Guardrail;
        let set = CanaryInputSet {
            version: 1,
            created_at_unix: 0,
            prompts: make_prompts(),
            coord_pubkey_hex: String::new(),
            signature_hex: String::new(),
        };
        let inj = std::sync::Arc::new(CanaryInputInjector::new(Some(set), 1));
        let guard = CanaryInputGuardrail::new(inj);
        let ctx = crate::guardrails::GuardrailContext {
            system_prompt: "",
            user_prompt: "test",
            model_output: "",
        };
        let outcome = guard.check(&ctx);
        assert!(matches!(
            outcome,
            crate::guardrails::GuardrailOutcome::Tripwire { .. }
        ));
    }
}
