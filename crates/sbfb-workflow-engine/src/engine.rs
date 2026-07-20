// SPDX-License-Identifier: AGPL-3.0-or-later
//! The reducer: a linear run of steps, memoised against the journal.
//!
//! Replay is NOT re-execution. Before acting, a step asks the journal
//! whether its effect already completed; if it did, the recorded receipt is
//! returned and nothing reaches the outside. That is the memoisation-of-steps
//! family (kickoff D3) rather than deterministic total replay: the engine
//! never re-runs a step to rebuild state, so an effect cannot be duplicated
//! by the act of rebuilding.
//!
//! Memoisation is indexed on `node_instance_id` — the full instance path,
//! never an ordinal counter. An ordinal index silently re-points memoised
//! answers as soon as a step is inserted, which is the classic way a durable
//! engine returns the wrong answer with total confidence.
//!
//! There is NO in-memory frontier cache: the store is re-read at every
//! step. That is slower and it is the point — a run that stops leaves no
//! half-trusted snapshot behind, because there was never a snapshot to
//! half-trust.

use std::collections::{BTreeMap, BTreeSet};

use crate::echo::{EffectTarget, TargetError};
use crate::event::TerminalReason;
use crate::fault::{FaultAction, FaultHook, FaultPoint, SpikeKnobs};
use crate::hash::{
    HashError, IdempotencyKey, StepRequest, canonical_digest, derive_effect_id, step_request_hash,
    validate_identity_component,
};
use crate::store::{RunStatus, RunStore, StoreError};

/// Context string of the spec digest. Local, never signed or gossiped.
pub const RUN_SPEC_DIGEST_CONTEXT: &str = "sbfb.workflow.engine.run-spec.v0";

/// One step of the spike run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepSpec {
    /// Full instance path, e.g. `root/echo#1`.
    pub node_instance_id: String,
    /// Deterministic input.
    pub input: BTreeMap<String, String>,
}

/// A linear run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSpec {
    /// Run identity — the same id re-executed is a RESUME, not a new run.
    pub run_id: String,
    /// Steps, in order.
    pub steps: Vec<StepSpec>,
}

impl RunSpec {
    /// Digest binding this run to its step LIST (ordered node paths).
    ///
    /// Per-step hashes only cover steps present on both sides of a replay, so
    /// they structurally cannot see a step that was added or removed. This
    /// digest is what closes that hole.
    pub fn digest(&self) -> Result<String, HashError> {
        validate_identity_component("run_id", &self.run_id)?;
        let nodes: Vec<&str> = self
            .steps
            .iter()
            .map(|s| s.node_instance_id.as_str())
            .collect();
        for node in &nodes {
            validate_identity_component("node_instance_id", node)?;
        }
        canonical_digest(RUN_SPEC_DIGEST_CONTEXT, &(self.run_id.as_str(), &nodes))
    }

    /// Refuse a spec that lists the same node instance twice.
    pub fn assert_distinct_nodes(&self) -> Result<(), EngineError> {
        let mut seen = BTreeSet::new();
        for step in &self.steps {
            if !seen.insert(step.node_instance_id.as_str()) {
                return Err(EngineError::DuplicateNodeInstance {
                    node_instance_id: step.node_instance_id.clone(),
                });
            }
        }
        Ok(())
    }
}

/// What one `execute` call did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    /// Run identity.
    pub run_id: String,
    /// Steps whose effect was applied during this call.
    pub steps_executed: usize,
    /// Steps served from the journal without touching the target.
    pub steps_memoised: usize,
    /// Receipt per node instance, memoised or fresh.
    pub receipts: BTreeMap<String, String>,
}

/// The engine refused to advance.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Store failure. A `HaltedInTransaction` is surfaced as
    /// [`EngineError::Halted`] instead, so the harness sees ONE halt shape
    /// wherever the fault point sits.
    #[error(transparent)]
    Store(StoreError),
    /// Canonicalisation failure.
    #[error(transparent)]
    Hash(#[from] HashError),
    /// Effect target failure.
    #[error(transparent)]
    Target(#[from] TargetError),
    /// FAIL-CLOSED: a replayed step asks a different question than the one
    /// whose answer is memoised. The run is terminal; it is never re-driven
    /// automatically and the stale answer is never returned.
    #[error(
        "step request hash mismatch at {node_instance_id}: journaled {expected}, replayed {got}"
    )]
    StepHashMismatch {
        /// Node instance whose request diverged.
        node_instance_id: String,
        /// Hash recorded when the effect was intended.
        expected: String,
        /// Hash re-computed on replay.
        got: String,
    },
    /// FAIL-CLOSED: an unconfirmed effect against a target that does not
    /// declare idempotency. Retrying could double the side output, so the
    /// run stops and says so instead of inventing a success.
    #[error(
        "outcome unknown for {node_instance_id} (effect {effect_id}): target is not idempotent"
    )]
    OutcomeUnknown {
        /// Node instance left unresolved.
        node_instance_id: String,
        /// Effect id left unconfirmed.
        effect_id: String,
    },
    /// A fault hook stopped the run at a crash frontier (harness only).
    #[error("halted at fault point {at:?}")]
    Halted {
        /// Where the run was interrupted.
        at: FaultPoint,
    },
    /// The run already reached a terminal state and is not resumable.
    #[error("run {run_id} is terminal: {reason}")]
    RunAlreadyTerminal {
        /// Run identity.
        run_id: String,
        /// Recorded reason.
        reason: String,
    },
    /// FAIL-CLOSED: the run is closed and cannot take another step. Without
    /// this guard, re-executing a completed run with an extra step appended
    /// entries AFTER the closing entry and never re-closed the run: a reader
    /// stopping at the terminal marker would miss an effect that really
    /// happened, and "the journal is the source of truth" would be false.
    #[error("run {run_id} is already completed and cannot be extended")]
    RunAlreadyCompleted {
        /// Run identity.
        run_id: String,
    },
    /// FAIL-CLOSED: the spec does not match the one this run was created
    /// with. Step-level hashes only cover steps present on BOTH sides, so a
    /// step ADDED or REMOVED between two runs would otherwise be invisible
    /// and `execute` would return a success describing a run that never
    /// happened.
    #[error("spec digest mismatch for run {run_id}: recorded {expected}, replayed {got}")]
    SpecDigestMismatch {
        /// Run identity.
        run_id: String,
        /// Digest recorded when the run was created.
        expected: String,
        /// Digest of the spec passed to this call.
        got: String,
    },
    /// A spec lists the same node instance twice. Memoisation is keyed on
    /// that path, so a duplicate would make the second occurrence serve the
    /// first one's answer (or kill the run on a hash mismatch) depending only
    /// on the data.
    #[error("duplicate node instance in spec: {node_instance_id}")]
    DuplicateNodeInstance {
        /// The repeated path.
        node_instance_id: String,
    },
    /// FAIL-CLOSED: at resume time the target deduplicates in a DIFFERENT
    /// durable store than the one journaled at intent time. Same name, same
    /// promise, different backend — the receipt of the first attempt is not
    /// where the new target would look for it.
    #[error(
        "target dedup scope changed for {node_instance_id}: journaled {journaled}, live {live}"
    )]
    TargetScopeChanged {
        /// Node instance concerned.
        node_instance_id: String,
        /// Scope recorded at intent time.
        journaled: String,
        /// Scope the live target reports now.
        live: String,
    },
    /// FAIL-CLOSED: at resume time the target's idempotency claim differs
    /// from the one journaled when the effect was intended.
    #[error(
        "target idempotency changed for {node_instance_id}: journaled {journaled}, live {live}"
    )]
    TargetIdempotencyChanged {
        /// Node instance concerned.
        node_instance_id: String,
        /// What was recorded at intent time.
        journaled: bool,
        /// What the live target claims now.
        live: bool,
    },
}

/// The durable-execution spike engine.
#[derive(Debug)]
pub struct SpikeEngine {
    store: RunStore,
    knobs: SpikeKnobs,
}

impl SpikeEngine {
    /// Build an engine over an already-open store.
    pub fn new(store: RunStore, knobs: SpikeKnobs) -> Self {
        Self { store, knobs }
    }

    /// Borrow the store (assertions, inspection).
    pub fn store(&self) -> &RunStore {
        &self.store
    }

    /// Run — or resume — `spec` against `target`.
    ///
    /// Calling this twice with the same `run_id` is the RESUME path: every
    /// completed step is served from the journal and the target is not
    /// touched again.
    pub fn execute(
        &mut self,
        spec: &RunSpec,
        target: &mut dyn EffectTarget,
        faults: &dyn FaultHook,
    ) -> Result<RunOutcome, EngineError> {
        spec.assert_distinct_nodes()?;
        let spec_digest = spec.digest()?;

        match self.store.run_status(&spec.run_id)? {
            Some(RunStatus::Terminal) => {
                return Err(EngineError::RunAlreadyTerminal {
                    run_id: spec.run_id.clone(),
                    reason: self
                        .store
                        .terminal_reason(&spec.run_id)?
                        .unwrap_or_else(|| "unknown".to_string()),
                });
            }
            Some(RunStatus::Completed) if !spec.steps.is_empty() => {
                // A completed run is CLOSED. Serving its memoised steps again
                // is harmless, but only after the spec is proven identical —
                // which the digest check below does. An extension is refused.
                let recorded = self.store.spec_digest(&spec.run_id)?;
                if recorded.as_deref() != Some(spec_digest.as_str()) {
                    return Err(EngineError::RunAlreadyCompleted {
                        run_id: spec.run_id.clone(),
                    });
                }
            }
            _ => {}
        }

        self.store.ensure_run(&spec.run_id, &spec_digest)?;

        // The spec is bound to the run by a digest over the ordered node
        // paths: a step added or removed is caught HERE, where per-step
        // hashes structurally cannot see it.
        if let Some(recorded) = self.store.spec_digest(&spec.run_id)?
            && recorded != spec_digest
        {
            return Err(EngineError::SpecDigestMismatch {
                run_id: spec.run_id.clone(),
                expected: recorded,
                got: spec_digest,
            });
        }

        let mut outcome = RunOutcome {
            run_id: spec.run_id.clone(),
            steps_executed: 0,
            steps_memoised: 0,
            receipts: BTreeMap::new(),
        };

        for step in &spec.steps {
            let request = StepRequest::new(
                step.node_instance_id.clone(),
                target.name(),
                step.input.clone(),
            )?;
            let request_hash = step_request_hash(&request)?;

            // Memoised? Serve the recorded receipt, touch nothing outside.
            if let Some(done) = self
                .store
                .completed_effect(&spec.run_id, &step.node_instance_id)?
            {
                if self.knobs.verify_step_hash() && done.request_hash != request_hash {
                    self.store.record_terminal(
                        &spec.run_id,
                        Some(&step.node_instance_id),
                        TerminalReason::StepHashMismatch,
                        Some(&done.request_hash),
                        Some(&request_hash),
                    )?;
                    return Err(EngineError::StepHashMismatch {
                        node_instance_id: step.node_instance_id.clone(),
                        expected: done.request_hash,
                        got: request_hash,
                    });
                }
                outcome.steps_memoised += 1;
                outcome
                    .receipts
                    .insert(step.node_instance_id.clone(), done.receipt);
                continue;
            }

            let effect_id = match self
                .store
                .pending_effect(&spec.run_id, &step.node_instance_id)?
            {
                // Resume over an UNCONFIRMED effect: the process died
                // between the intent and the result, so whether the side
                // output happened is unknown to the engine.
                Some(pending) => {
                    // Hash FIRST: a divergent request is a spec bug, and
                    // reporting it as `outcome_unknown` would blame the
                    // infrastructure for an authoring mistake. Both branches
                    // are terminal and apply nothing, so the order changes
                    // only the diagnosis — which is the whole value of a
                    // fail-closed stop.
                    if self.knobs.verify_step_hash() && pending.request_hash != request_hash {
                        self.store.record_terminal(
                            &spec.run_id,
                            Some(&step.node_instance_id),
                            TerminalReason::StepHashMismatch,
                            Some(&pending.request_hash),
                            Some(&request_hash),
                        )?;
                        return Err(EngineError::StepHashMismatch {
                            node_instance_id: step.node_instance_id.clone(),
                            expected: pending.request_hash,
                            got: request_hash,
                        });
                    }
                    // The idempotency claim is read from the JOURNAL, not
                    // from the live object: a target with the same name but a
                    // different promise must not be able to authorise a
                    // re-application after the fact.
                    if pending.target_idempotent != target.is_idempotent() {
                        return Err(EngineError::TargetIdempotencyChanged {
                            node_instance_id: step.node_instance_id.clone(),
                            journaled: pending.target_idempotent,
                            live: target.is_idempotent(),
                        });
                    }
                    // The dedup BACKEND must be the same one, not merely a
                    // target with the same name and the same promise: the
                    // receipt of the first attempt lives in the journaled
                    // scope, and a different backend would happily apply the
                    // key a second time.
                    if pending.dedup_scope != target.dedup_scope() {
                        return Err(EngineError::TargetScopeChanged {
                            node_instance_id: step.node_instance_id.clone(),
                            journaled: pending.dedup_scope,
                            live: target.dedup_scope(),
                        });
                    }
                    if !pending.target_idempotent {
                        self.store.record_terminal(
                            &spec.run_id,
                            Some(&step.node_instance_id),
                            TerminalReason::OutcomeUnknown,
                            None,
                            None,
                        )?;
                        return Err(EngineError::OutcomeUnknown {
                            node_instance_id: step.node_instance_id.clone(),
                            effect_id: pending.effect_id,
                        });
                    }
                    pending.effect_id
                }
                None => {
                    let effect_id =
                        derive_effect_id(&spec.run_id, &step.node_instance_id, &request_hash);
                    // RED CONTROL — the split path exists only behind
                    // `spike-faults`; without the feature the degraded knob
                    // cannot even be built, so the guaranteed call is the
                    // only code compiled here.
                    #[cfg(feature = "spike-faults")]
                    if self.knobs.atomic_outbox() {
                        self.store.record_intent_atomic(
                            &spec.run_id,
                            &step.node_instance_id,
                            &request_hash,
                            &effect_id,
                            target.is_idempotent(),
                            &target.dedup_scope(),
                            faults,
                        )?;
                    } else {
                        self.store.record_intent_outbox_only(
                            &spec.run_id,
                            &step.node_instance_id,
                            &request_hash,
                            &effect_id,
                            target.is_idempotent(),
                            &target.dedup_scope(),
                        )?;
                        halt_if_requested(faults, FaultPoint::BetweenOutboxAndIntentEvent)?;
                        self.store.record_intent_event_only(
                            &spec.run_id,
                            &step.node_instance_id,
                            &request_hash,
                            &effect_id,
                        )?;
                    }
                    #[cfg(not(feature = "spike-faults"))]
                    self.store.record_intent_atomic(
                        &spec.run_id,
                        &step.node_instance_id,
                        &request_hash,
                        &effect_id,
                        target.is_idempotent(),
                        &target.dedup_scope(),
                        faults,
                    )?;
                    effect_id
                }
            };

            halt_if_requested(faults, FaultPoint::AfterIntentBeforeEffect)?;

            let key = IdempotencyKey::new(
                spec.run_id.clone(),
                step.node_instance_id.clone(),
                effect_id.clone(),
            )?;
            let receipt = target.apply(&key, &step.input)?;

            halt_if_requested(faults, FaultPoint::AfterEffectBeforeResult)?;

            // RED CONTROL — the split completion path exists only behind
            // `spike-faults`; without the feature the degraded knob cannot be
            // built, so only the atomic call is compiled.
            #[cfg(feature = "spike-faults")]
            if self.knobs.atomic_completion() {
                self.store.record_completion(
                    &spec.run_id,
                    &step.node_instance_id,
                    &effect_id,
                    &request_hash,
                    &receipt,
                    faults,
                )?;
            } else {
                self.store.record_completion_event_only(
                    &spec.run_id,
                    &step.node_instance_id,
                    &effect_id,
                    &request_hash,
                    &receipt,
                )?;
            }
            #[cfg(not(feature = "spike-faults"))]
            self.store.record_completion(
                &spec.run_id,
                &step.node_instance_id,
                &effect_id,
                &request_hash,
                &receipt,
                faults,
            )?;

            halt_if_requested(faults, FaultPoint::AfterResult)?;

            outcome.steps_executed += 1;
            outcome
                .receipts
                .insert(step.node_instance_id.clone(), receipt);
        }

        // Closing a run is itself idempotent: re-running a completed run
        // appends nothing, so the journal does not grow with every replay.
        if self.store.run_status(&spec.run_id)? != Some(RunStatus::Completed) {
            self.store.record_run_completed(&spec.run_id)?;
        }

        Ok(outcome)
    }
}

fn halt_if_requested(faults: &dyn FaultHook, point: FaultPoint) -> Result<(), EngineError> {
    match faults.hit(point) {
        FaultAction::Continue => Ok(()),
        FaultAction::Halt => Err(EngineError::Halted { at: point }),
    }
}

impl From<StoreError> for EngineError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::HaltedInTransaction { at } => Self::Halted { at },
            other => Self::Store(other),
        }
    }
}
