// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host-neutral signal trait (Sprint 83 Phase J) — how external signals
//! and human decisions reach a paused run.
//!
//! ## Division of authority (research §12.6, THREAT §19)
//!
//! When the scheduler parks a node (`wait.signal` → `waiting_external`,
//! `human.review` → `waiting_human`) it OPENS A WAIT in the durable store:
//! the node, the awaited signal name, and an engine-DERIVED mono-usage
//! nonce. The host (daemon route, Hub bridge, test harness) learns the open
//! waits by reading them back, obtains the signal or the human decision,
//! and DEPOSITS it durably. The scheduler then CONSUMES the deposit — nonce
//! spent, `signal_consumed`/`human_decision_recorded` journaled, wait
//! closed — in ONE store transaction, so a replayed delivery of the same
//! nonce can never re-fire the resume.
//!
//! What the ENGINE validates: the nonce (mono-usage), the expiry (against
//! the injected clock), the decision label (closed set of the plan). What
//! it does NOT validate: WHO the human principal is and whether the
//! approval is signed — that binding is the Phase L1 host chrome, and this
//! trait deliberately has no field for it.
//!
//! ## Why a trait and not just store calls
//!
//! The orchestrator drains a [`SignalSource`] at every scheduling pass —
//! that is the host-neutral seam: a scripted source drives the golden
//! fixtures deterministically (preflight DÉC-1), a daemon-backed source
//! would poll its HTTP intake, and neither changes one line of scheduler
//! code.

use crate::store::PendingWait;

/// One deposit a host hands to the engine in response to an open wait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDeposit {
    /// The mono-usage nonce of the wait this deposit answers (learned from
    /// [`PendingWait::nonce`]). A deposit whose nonce matches no open wait
    /// is refused loudly at consumption.
    pub nonce: String,
    /// Signal payload — MATERIAL data: it lands in the durable signal
    /// table, and only its digest ever reaches the journal.
    pub payload: serde_json::Value,
    /// The decision label, for a `human.review` wait. `None` for a plain
    /// `wait.signal`. Validated against the node's closed label set at
    /// consumption, never trusted.
    pub decision_label: Option<String>,
    /// Optional expiry the DEPOSITOR stamps on its deposit (host policy —
    /// e.g. "this approval is valid 15 minutes"). A deposit consumed after
    /// this instant is refused loudly. `None` = no expiry.
    pub expires_at_ms: Option<i64>,
}

/// A host-side source of signal deposits, drained by the orchestrator
/// before each scheduling pass.
pub trait SignalSource {
    /// Offer deposits for the given OPEN waits. Called with the current
    /// open-wait list so the source can correlate nonces; anything it
    /// returns is recorded durably and then consumed by the scheduler.
    fn poll(&mut self, open_waits: &[PendingWait]) -> Vec<SignalDeposit>;
}

/// A source with nothing to deliver — the default for runs that never park.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSignals;

impl SignalSource for NoSignals {
    fn poll(&mut self, _open_waits: &[PendingWait]) -> Vec<SignalDeposit> {
        Vec::new()
    }
}

/// A deterministic scripted source for tests and golden fixtures
/// (preflight DÉC-1): each entry answers the FIRST open wait matching its
/// node's structural path, in script order, exactly once.
#[derive(Debug, Default)]
pub struct ScriptedSignals {
    entries: Vec<ScriptEntry>,
}

#[derive(Debug)]
struct ScriptEntry {
    /// Structural node path this entry answers (runtime iteration suffixes
    /// are projected away before matching, so one entry can answer the
    /// same wait node across repeat iterations only if listed once per
    /// iteration — a script is a finite, ordered budget of answers).
    structural_node_id: String,
    payload: serde_json::Value,
    decision_label: Option<String>,
    expires_at_ms: Option<i64>,
    used: bool,
}

impl ScriptedSignals {
    /// An empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a `wait.signal` answer for the node at `structural_node_id`.
    pub fn signal(mut self, structural_node_id: &str, payload: serde_json::Value) -> Self {
        self.entries.push(ScriptEntry {
            structural_node_id: structural_node_id.to_string(),
            payload,
            decision_label: None,
            expires_at_ms: None,
            used: false,
        });
        self
    }

    /// Append a `human.review` decision for the node at
    /// `structural_node_id`.
    pub fn decision(
        mut self,
        structural_node_id: &str,
        decision_label: &str,
        payload: serde_json::Value,
    ) -> Self {
        self.entries.push(ScriptEntry {
            structural_node_id: structural_node_id.to_string(),
            payload,
            decision_label: Some(decision_label.to_string()),
            expires_at_ms: None,
            used: false,
        });
        self
    }

    /// Append an already-expired deposit (adversarial tests).
    pub fn expired_signal(
        mut self,
        structural_node_id: &str,
        payload: serde_json::Value,
        expires_at_ms: i64,
    ) -> Self {
        self.entries.push(ScriptEntry {
            structural_node_id: structural_node_id.to_string(),
            payload,
            decision_label: None,
            expires_at_ms: Some(expires_at_ms),
            used: false,
        });
        self
    }
}

impl SignalSource for ScriptedSignals {
    fn poll(&mut self, open_waits: &[PendingWait]) -> Vec<SignalDeposit> {
        let mut out = Vec::new();
        for wait in open_waits {
            let structural = crate::runtime_id::structural_id(&wait.node_instance_id);
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|e| !e.used && e.structural_node_id == structural)
            {
                entry.used = true;
                out.push(SignalDeposit {
                    nonce: wait.nonce.clone(),
                    payload: entry.payload.clone(),
                    decision_label: entry.decision_label.clone(),
                    expires_at_ms: entry.expires_at_ms,
                });
            }
        }
        out
    }
}
