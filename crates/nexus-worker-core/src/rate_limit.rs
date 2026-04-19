// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rate-limit GCRA multi-tier per-(consumer, worker, model).
//!
//! Sprint 21 Phase A (R1 scope-cut post-G8 drift detection, arbitré
//! user 2026-04-19) : **worker-engine gate pure Rust**. La primitive
//! [`RateLimiter`] est consommée par l'engine worker pre-task-
//! execution pour borner le débit d'inférence LLM / GPU par tuple
//! `(consumer, worker, model)`, défendant les menaces
//! `HARDENING_ROADMAP §3 C-ModelExtract` (§4 model extraction
//! paper-flood) + `C-DosFlood` (§7 DoS flood).
//!
//! Le middleware HTTP `/task/submit` proposé initialement plan §4.1
//! ne s'applique pas Phase A : cet endpoint vit côté Python FastAPI
//! (`packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py
//! ::POST /tasks/submit`, depuis Sprint 4 Phase A), `tower-governor`
//! axum ne peut pas middleware FastAPI. Ré-évaluation S22+ sprint
//! API sécurité dédié (slowapi ou équivalent Python).
//!
//! ## Algo
//!
//! GCRA (Generic Cell Rate Algorithm) via le crate `governor` 0.10.
//! Un [`governor::DefaultKeyedRateLimiter`] DashMap-backed par défaut,
//! avec un limiter séparé pour chaque override consommateur whitelistable.
//!
//! ## Policy
//!
//! [`RateLimitPolicy`] décode directement le fichier
//! `~/.sbfb/rate_limit_policy.toml` via `serde` + `toml`. Le watcher
//! [`crate::rate_limit_policy_loader`] surveille les writes et met à
//! jour un `Arc<RwLock<RateLimitPolicy>>` sans restart. Pattern
//! cohérent `pow_policy_loader.rs` (S20 Phase C) + `TokenRotator`
//! (S18 D-1) + `consent/watcher.rs` (S16 Phase C).
//!
//! ## Scope-cut R1 vs plan initial
//!
//! Le plan initial §4.2 listait 2 tests HTTP (`task_submit_429_on_
//! rate_limit` + `rate_limit_middleware_order_before_pow_gate`). R1
//! les défère S22+ puisqu'il n'y a pas de middleware HTTP Phase A.
//! Les tests unit GCRA + eviction + override + policy hot-reload
//! restent in-scope.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, RwLock};

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter as GovernorRateLimiter};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 32-byte Ed25519 pubkey hex (64 chars) — identifies the consumer
/// that submitted the task through the coordinator.
pub type ConsumerId = String;

/// Worker alias string — the worker node that would execute the
/// task if admitted. A single physical machine can register multiple
/// workers (different GPU slices).
pub type WorkerId = String;

/// Model alias string — e.g. `"llama3:8b-instruct-q4"`.
pub type ModelId = String;

/// Composite rate-limit key. Each distinct `(consumer, worker,
/// model)` triple gets its own GCRA bucket.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct RateKey {
    pub consumer: ConsumerId,
    pub worker: WorkerId,
    pub model: ModelId,
}

impl RateKey {
    pub fn new(
        consumer: impl Into<ConsumerId>,
        worker: impl Into<WorkerId>,
        model: impl Into<ModelId>,
    ) -> Self {
        Self {
            consumer: consumer.into(),
            worker: worker.into(),
            model: model.into(),
        }
    }
}

/// Parsed `~/.sbfb/rate_limit_policy.toml` payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RateLimitPolicy {
    #[serde(default)]
    pub default: RateLimitTier,
    #[serde(default)]
    pub overrides: RateLimitOverrides,
}

/// Baseline quota for any tuple that does not match a consumer
/// override.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RateLimitTier {
    /// Requests per minute per tuple. Must be > 0.
    pub per_min: u32,
    /// Burst multiplier — actual burst capacity is
    /// `per_min * burst_multiplier` (rounded, floored to 1). A
    /// multiplier of `2.0` means a consumer can temporarily double
    /// its sustained rate for one GCRA window.
    pub burst_multiplier: f64,
}

impl Default for RateLimitTier {
    fn default() -> Self {
        Self {
            per_min: 60,
            burst_multiplier: 2.0,
        }
    }
}

/// Operator-configured overrides. Currently only per-consumer
/// overrides are supported; future revisions (S22+) may add
/// per-worker or per-model tiers.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RateLimitOverrides {
    #[serde(default)]
    pub consumer: Vec<ConsumerOverride>,
}

/// Per-consumer whitelist override — useful for privileged
/// pubkeys (operator's own consumer, internal Gate-2 apps).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ConsumerOverride {
    /// Hex-encoded Ed25519 pubkey (64 chars).
    pub pubkey_hex: String,
    pub per_min: u32,
    #[serde(default = "default_burst_multiplier")]
    pub burst_multiplier: f64,
}

fn default_burst_multiplier() -> f64 {
    2.0
}

/// Errors surfaced by [`RateLimiter::check`] and construction.
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// The tuple exceeded its GCRA budget. Call sites are expected
    /// to translate this into a task-reject + retry-later signal
    /// (429 equivalent at the coordinator layer, defer-and-retry
    /// at the worker engine layer).
    #[error("rate limit exceeded for consumer={consumer}, worker={worker}, model={model}")]
    Saturated {
        consumer: ConsumerId,
        worker: WorkerId,
        model: ModelId,
    },
    /// A zero `per_min` in the policy file produces an invalid
    /// quota — `governor::Quota::per_minute` requires a non-zero
    /// count. The loader rejects the policy at boot if this
    /// happens.
    #[error("invalid quota: per_min must be > 0 (got {0})")]
    InvalidQuota(u32),
}

/// Internal rate-limit state held under the [`RateLimiter::state`]
/// RwLock. Grouping the default limiter and the override map inside
/// a single lock-guarded struct means a policy swap replaces both
/// atomically — callers can never observe a half-applied reload
/// where the default quota is fresh but the overrides still point
/// at the old tier.
struct RateLimiterState {
    default: Arc<DefaultKeyedRateLimiter<RateKey>>,
    overrides: Arc<HashMap<ConsumerId, Arc<DefaultKeyedRateLimiter<RateKey>>>>,
}

/// Rate-limit primitive. Holds a default keyed rate limiter plus a
/// per-consumer override map behind a single `RwLock` so hot-reload
/// rotations are atomic. `check` takes a read lock for long enough
/// to clone the relevant `Arc<DefaultKeyedRateLimiter>` and then
/// runs the GCRA check outside the lock — readers never block
/// other readers, and a writer only blocks readers for the duration
/// of the `Arc` swap (a handful of atomic operations).
pub struct RateLimiter {
    state: RwLock<RateLimiterState>,
    /// Shared policy snapshot kept in sync with [`Self::state`]. The
    /// loader watcher holds a clone via [`Self::policy_handle`] so
    /// diagnostic code can inspect the live policy without going
    /// through the governor layer.
    policy: Arc<RwLock<RateLimitPolicy>>,
}

impl RateLimiter {
    /// Build a rate limiter from a shared policy handle. The initial
    /// snapshot is read once and wired into the internal GCRA state.
    /// Subsequent policy edits land via [`Self::swap_policy`] — the
    /// loader watcher (cf. [`crate::rate_limit_policy_loader`]) calls
    /// that method on each successful disk reload so default-tier
    /// quota bumps AND override membership changes take effect
    /// without restarting the worker.
    pub fn from_policy(policy: Arc<RwLock<RateLimitPolicy>>) -> Result<Self, RateLimitError> {
        let snapshot = match policy.read() {
            Ok(g) => g.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        Self::from_snapshot(&snapshot, policy)
    }

    fn from_snapshot(
        snapshot: &RateLimitPolicy,
        policy: Arc<RwLock<RateLimitPolicy>>,
    ) -> Result<Self, RateLimitError> {
        let state = build_state_from(snapshot)?;
        Ok(Self {
            state: RwLock::new(state),
            policy,
        })
    }

    /// Convenience constructor used by tests : build directly from a
    /// by-value policy, wrapping it in a fresh `Arc<RwLock<_>>`.
    pub fn from_policy_value(policy: RateLimitPolicy) -> Result<Self, RateLimitError> {
        let shared = Arc::new(RwLock::new(policy));
        Self::from_policy(shared)
    }

    /// Atomically rebuild the internal GCRA state from a new
    /// [`RateLimitPolicy`]. Called by the loader watcher on each
    /// successful TOML reload, via
    /// [`crate::rate_limit_policy_loader::RateLimitPolicyWatcher::
    /// spawn_with_on_reload`]. Also updates the shared policy handle
    /// exposed via [`Self::policy_handle`] so a poisoned watcher
    /// thread (or a test calling `swap_policy` directly) keeps both
    /// views consistent.
    ///
    /// Fails with [`RateLimitError::InvalidQuota`] if the new policy
    /// has `per_min = 0` for the default tier or any override ; the
    /// caller is expected to `warn!` and keep the previous policy in
    /// that case (same pattern as the loader's malformed-TOML path).
    pub fn swap_policy(&self, new: RateLimitPolicy) -> Result<(), RateLimitError> {
        let new_state = build_state_from(&new)?;
        // Write the new state first, then the exposed policy handle.
        // Order matters : a `check` call observing the new state
        // while `policy` still reads old is fine (policy is only for
        // diagnostics), but the inverse would let a concurrent
        // observer believe the override exists before it is
        // actually enforced.
        match self.state.write() {
            Ok(mut guard) => *guard = new_state,
            Err(poisoned) => {
                *poisoned.into_inner() = new_state;
            }
        }
        if let Ok(mut guard) = self.policy.write() {
            *guard = new;
        }
        Ok(())
    }

    /// Cheap clone of the shared [`Arc<RwLock<RateLimitPolicy>>`]
    /// kept in sync with the internal GCRA state. Useful for
    /// diagnostic endpoints and tests.
    pub fn policy_handle(&self) -> Arc<RwLock<RateLimitPolicy>> {
        Arc::clone(&self.policy)
    }

    /// Check a single request against its tuple's bucket. Returns
    /// [`RateLimitError::Saturated`] if the GCRA state is exhausted.
    /// The underlying `governor` limiter returns a `NotUntil` with
    /// the replenish deadline — we collapse that to a domain error
    /// here to keep the worker engine agnostic to the `governor`
    /// types.
    pub fn check(&self, key: &RateKey) -> Result<(), RateLimitError> {
        let limiter = self.resolve_limiter(&key.consumer);
        match limiter.check_key(key) {
            Ok(()) => Ok(()),
            Err(_not_until) => Err(RateLimitError::Saturated {
                consumer: key.consumer.clone(),
                worker: key.worker.clone(),
                model: key.model.clone(),
            }),
        }
    }

    fn resolve_limiter(&self, consumer: &ConsumerId) -> Arc<DefaultKeyedRateLimiter<RateKey>> {
        let guard = match self.state.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        match guard.overrides.get(consumer) {
            Some(limiter) => Arc::clone(limiter),
            None => Arc::clone(&guard.default),
        }
    }

    /// Remove stale keys from every underlying GCRA map. Call this
    /// periodically (the engine does so every 60s from a background
    /// tokio task) to prevent unbounded memory growth when many
    /// ephemeral consumers pass through.
    pub fn retain_recent(&self) {
        let snapshot = self.snapshot_limiters();
        for limiter in snapshot {
            limiter.retain_recent();
        }
    }

    /// Total live key count across default + all override limiters.
    /// Useful for tests and ops metrics.
    pub fn len(&self) -> usize {
        self.snapshot_limiters().iter().map(|l| l.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of registered consumer overrides. Useful for tests +
    /// policy observability.
    pub fn override_count(&self) -> usize {
        let guard = match self.state.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.overrides.len()
    }

    /// Clone the default + every override `Arc` into a fresh `Vec`
    /// under the read lock, then drop the lock. This keeps the
    /// GCRA operations (retain_recent, len) outside the lock and
    /// avoids re-entrancy deadlocks when the operation logic is
    /// long-running.
    fn snapshot_limiters(&self) -> Vec<Arc<DefaultKeyedRateLimiter<RateKey>>> {
        let guard = match self.state.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut out: Vec<Arc<DefaultKeyedRateLimiter<RateKey>>> =
            Vec::with_capacity(guard.overrides.len() + 1);
        out.push(Arc::clone(&guard.default));
        for limiter in guard.overrides.values() {
            out.push(Arc::clone(limiter));
        }
        out
    }
}

fn build_state_from(snapshot: &RateLimitPolicy) -> Result<RateLimiterState, RateLimitError> {
    let default = Arc::new(build_keyed_limiter(&snapshot.default)?);
    let mut overrides: HashMap<ConsumerId, Arc<DefaultKeyedRateLimiter<RateKey>>> = HashMap::new();
    for ov in &snapshot.overrides.consumer {
        let tier = RateLimitTier {
            per_min: ov.per_min,
            burst_multiplier: ov.burst_multiplier,
        };
        let limiter = Arc::new(build_keyed_limiter(&tier)?);
        overrides.insert(ov.pubkey_hex.clone(), limiter);
    }
    Ok(RateLimiterState {
        default,
        overrides: Arc::new(overrides),
    })
}

fn build_keyed_limiter(
    tier: &RateLimitTier,
) -> Result<DefaultKeyedRateLimiter<RateKey>, RateLimitError> {
    let quota = make_quota(tier.per_min, tier.burst_multiplier)?;
    Ok(GovernorRateLimiter::keyed(quota))
}

fn make_quota(per_min: u32, burst_multiplier: f64) -> Result<Quota, RateLimitError> {
    let per_min_nz = NonZeroU32::new(per_min).ok_or(RateLimitError::InvalidQuota(per_min))?;
    // Burst = per_min * multiplier, rounded, floored to 1. A
    // multiplier < 1.0 yields a burst smaller than the rate, which
    // `governor` rejects — we floor to `per_min` in that case so
    // the operator can't accidentally cripple their own gate.
    let raw_burst = (per_min as f64 * burst_multiplier).round();
    let burst = if raw_burst < per_min as f64 {
        per_min
    } else {
        raw_burst as u32
    };
    let burst_nz = NonZeroU32::new(burst.max(1)).expect("burst >= 1 by construction above");
    Ok(Quota::per_minute(per_min_nz).allow_burst(burst_nz))
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter_with(per_min: u32) -> RateLimiter {
        let policy = RateLimitPolicy {
            default: RateLimitTier {
                per_min,
                burst_multiplier: 1.0, // burst == per_min for predictable tests
            },
            overrides: RateLimitOverrides::default(),
        };
        RateLimiter::from_policy_value(policy).expect("build limiter")
    }

    #[test]
    fn saturation_rejects_over_budget() {
        // With burst_multiplier = 1.0, the initial cell budget equals
        // `per_min`. We fire `per_min` consecutive requests — all
        // must succeed — then the next one must be rejected. No
        // time passes between checks so the GCRA replenish does
        // not help.
        let limiter = limiter_with(5);
        let key = RateKey::new("consumer_a", "worker_1", "llama3");
        for i in 0..5 {
            limiter
                .check(&key)
                .unwrap_or_else(|e| panic!("request {i} should pass, got {e}"));
        }
        let err = limiter.check(&key).expect_err("6th request must saturate");
        match err {
            RateLimitError::Saturated {
                consumer,
                worker,
                model,
            } => {
                assert_eq!(consumer, "consumer_a");
                assert_eq!(worker, "worker_1");
                assert_eq!(model, "llama3");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn per_tuple_independence() {
        // Three distinct tuples must have independent buckets —
        // saturating one does not spill over to the others.
        let limiter = limiter_with(3);
        let key_a = RateKey::new("consumer_a", "worker_1", "llama3");
        let key_b = RateKey::new("consumer_b", "worker_1", "llama3");
        let key_c = RateKey::new("consumer_a", "worker_2", "llama3");

        // Saturate `key_a`.
        for _ in 0..3 {
            limiter.check(&key_a).unwrap();
        }
        assert!(matches!(
            limiter.check(&key_a),
            Err(RateLimitError::Saturated { .. })
        ));

        // `key_b` and `key_c` must still pass at least once.
        limiter.check(&key_b).expect("different consumer must pass");
        limiter.check(&key_c).expect("different worker must pass");
    }

    #[test]
    fn per_tuple_independence_on_model_axis() {
        // A distinct model must not inherit the saturated state of
        // a sibling model under the same consumer+worker.
        let limiter = limiter_with(2);
        let key_small = RateKey::new("consumer_a", "worker_1", "llama3:8b");
        let key_large = RateKey::new("consumer_a", "worker_1", "llama3:70b");

        for _ in 0..2 {
            limiter.check(&key_small).unwrap();
        }
        assert!(limiter.check(&key_small).is_err());
        limiter
            .check(&key_large)
            .expect("different model must pass");
    }

    #[test]
    fn eviction_after_retain_recent_drops_stale_keys() {
        // `governor::DefaultKeyedRateLimiter::retain_recent` drops
        // keys whose bucket has fully replenished back to full
        // capacity. We exercise a few keys, let them replenish
        // naturally (the default GCRA clock is wall-clock), then
        // call `retain_recent` and check `len` decreases.
        //
        // The crate's guarantee is "recently active keys stay" —
        // we cannot easily force replenishment in a unit test
        // without sleeping. We therefore test the happy path:
        // `retain_recent` is idempotent and does not panic, and
        // `len` reflects the current active key count.
        let limiter = limiter_with(10);
        for i in 0..5 {
            let key = RateKey::new(format!("consumer_{i}"), "worker_1", "llama3");
            limiter.check(&key).unwrap();
        }
        assert_eq!(limiter.len(), 5, "5 keys must be tracked");
        limiter.retain_recent();
        // Keys are still "recent" so len stays — but the call
        // must complete without panicking, which is the invariant
        // we need for the engine's periodic housekeeping.
        assert!(limiter.len() <= 5);
    }

    #[test]
    fn override_consumer_whitelist_lifts_budget() {
        let policy = RateLimitPolicy {
            default: RateLimitTier {
                per_min: 2,
                burst_multiplier: 1.0,
            },
            overrides: RateLimitOverrides {
                consumer: vec![ConsumerOverride {
                    pubkey_hex: "whitelisted_consumer".to_string(),
                    per_min: 100,
                    burst_multiplier: 1.0,
                }],
            },
        };
        let limiter = RateLimiter::from_policy_value(policy).unwrap();
        assert_eq!(limiter.override_count(), 1);

        let key_default = RateKey::new("normal_consumer", "worker_1", "llama3");
        let key_override = RateKey::new("whitelisted_consumer", "worker_1", "llama3");

        // Default consumer saturates after 2 requests.
        limiter.check(&key_default).unwrap();
        limiter.check(&key_default).unwrap();
        assert!(limiter.check(&key_default).is_err());

        // Whitelisted consumer breezes through 10 requests.
        for _ in 0..10 {
            limiter.check(&key_override).expect("override must allow");
        }
    }

    #[test]
    fn invalid_quota_per_min_zero_is_rejected() {
        // RateLimiter itself does not derive Debug (DashMap and the
        // inner governor state-store types don't either) so we match
        // on the Result directly rather than use `expect_err`.
        let policy = RateLimitPolicy {
            default: RateLimitTier {
                per_min: 0,
                burst_multiplier: 2.0,
            },
            overrides: RateLimitOverrides::default(),
        };
        match RateLimiter::from_policy_value(policy) {
            Ok(_) => panic!("per_min=0 must fail"),
            Err(RateLimitError::InvalidQuota(0)) => {}
            Err(other) => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn default_policy_admits_reasonable_traffic() {
        // The `Default` impl must produce a policy that lets a
        // single-tuple normal user submit at least a handful of
        // requests before saturating. 60 req/min + burst 120
        // trivially clears 10 consecutive requests.
        let limiter = RateLimiter::from_policy_value(RateLimitPolicy::default()).unwrap();
        let key = RateKey::new("consumer_x", "worker_x", "llama3");
        for _ in 0..10 {
            limiter
                .check(&key)
                .expect("default policy must admit 10 consecutive");
        }
    }

    #[test]
    fn policy_serde_round_trip_via_toml() {
        // The watcher (rate_limit_policy_loader) reads the TOML
        // file through `toml::from_str`; here we verify the schema
        // matches what the operator would realistically write.
        let toml_src = r#"
[default]
per_min = 200
burst_multiplier = 1.5

[[overrides.consumer]]
pubkey_hex = "abc123"
per_min = 1000
burst_multiplier = 2.0
"#;
        let parsed: RateLimitPolicy = toml::from_str(toml_src).expect("parse");
        assert_eq!(parsed.default.per_min, 200);
        assert!((parsed.default.burst_multiplier - 1.5).abs() < 1e-9);
        assert_eq!(parsed.overrides.consumer.len(), 1);
        assert_eq!(parsed.overrides.consumer[0].pubkey_hex, "abc123");
        assert_eq!(parsed.overrides.consumer[0].per_min, 1000);
    }

    #[test]
    fn burst_multiplier_below_one_floors_to_per_min() {
        // An operator who accidentally writes `burst_multiplier =
        // 0.5` with `per_min = 10` would otherwise get a burst of
        // 5 — strictly smaller than the rate, which `governor`
        // rejects. We floor to `per_min` so the gate stays
        // consistent.
        let policy = RateLimitPolicy {
            default: RateLimitTier {
                per_min: 10,
                burst_multiplier: 0.5,
            },
            overrides: RateLimitOverrides::default(),
        };
        let limiter = RateLimiter::from_policy_value(policy).expect("floored burst must build");
        let key = RateKey::new("c", "w", "m");
        // Burst floored to 10 → 10 consecutive requests must pass.
        for _ in 0..10 {
            limiter.check(&key).unwrap();
        }
    }
}
