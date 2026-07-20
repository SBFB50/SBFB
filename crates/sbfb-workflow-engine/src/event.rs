// SPDX-License-Identifier: AGPL-3.0-or-later
//! `RunEvent` v0 — one append-only entry of the run journal.
//!
//! The DURABLE STORE is the source of truth for replay — and that store is
//! the journal PLUS the outbox, not the journal alone. Being precise costs
//! nothing and avoids a false claim: the receipt, the pending request hash,
//! the target's idempotency claim and its dedup scope all live in the outbox
//! row, deliberately, so that no material payload ever reaches the journal.
//! What the journal alone guarantees is the ORDER and the OCCURRENCE of what
//! happened. Nothing is carried in memory across a step.
//!
//! Pre-launch protocol: [`RUN_EVENT_FORMAT_VERSION`] is a new local version,
//! not a bump. The envelope is deliberately kept in this crate rather than
//! filling the reserved `RunEvent.v1` identity in `sbfb-workflow-core`: the
//! spike must be free to change its journal shape while the GO/NO-GO
//! question is still open, and the reserved identity stays an untouched stub
//! so the `len == 7` registry and its schema golden are unaffected.
//!
//! Data posture: the enum below can hold an effect id and a receipt DIGEST.
//! It has NO FIELD DEDICATED to receipt content, prompt text or a secret —
//! absent by TYPE, not skipped at serialization time, so no `Debug` render,
//! log line or journal row carries a payload.
//!
//! What that does NOT claim: `run_id` and `node_instance_id` are caller-chosen
//! `String`s, validated only for length and control characters. A caller who
//! puts a token or an email in an identifier persists it here. Identifiers are
//! the caller's to choose; the guarantee is about payload FIELDS, not about
//! what someone names their run.

use serde::{Deserialize, Serialize};

/// Version of the local run-journal envelope.
pub const RUN_EVENT_FORMAT_VERSION: u16 = 1;

/// Why a run reached a terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalReason {
    /// A replayed step re-computed a different request hash than the one
    /// memoised. The run stops instead of answering a new question with an
    /// old answer.
    StepHashMismatch,
    /// An effect was intended against a target that does not declare
    /// idempotency, and the process died before the result was journaled.
    /// The engine cannot know whether the side output happened, so it
    /// reports the honest outcome instead of retrying (which could double
    /// the effect) or inventing a success.
    OutcomeUnknown,
}

/// What happened at one point of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunEventKind {
    /// The run was created.
    RunStarted,
    /// An effect is about to be attempted. Written in the SAME transaction
    /// as the outbox row it refers to.
    EffectIntended {
        /// Engine-derived effect id.
        effect_id: String,
    },
    /// The effect returned a receipt. Written in the SAME transaction as the
    /// outbox row transition to `applied`.
    EffectCompleted {
        /// Engine-derived effect id.
        effect_id: String,
        /// Digest of the receipt — never the receipt itself.
        receipt_digest: String,
    },
    /// Every step of the run is memoised and the run is finished.
    RunCompleted,
    /// The run stopped and will not advance.
    RunTerminal {
        /// Why it stopped.
        reason: TerminalReason,
        /// Digest journaled when the step was intended, when the reason is a
        /// hash divergence. Both hashes are recorded so a post-mortem can be
        /// made from the JOURNAL ALONE: without them only the first caller
        /// ever sees WHAT diverged, and a later `execute` just reports "this
        /// run is terminal". Hex digests carry no payload content.
        expected_hash: Option<String>,
        /// Digest re-computed on replay, when the reason is a divergence.
        got_hash: Option<String>,
    },
}

impl TerminalReason {
    /// Stable string stored in `runs.terminal_reason`. Exhaustive `match`
    /// rather than a serialization round-trip with a fallback: a silent
    /// `"unknown"` in the field that carries the fail-closed diagnosis would
    /// be the worst possible place for a lossy default.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StepHashMismatch => "step_hash_mismatch",
            Self::OutcomeUnknown => "outcome_unknown",
        }
    }
}

impl RunEventKind {
    /// Stable discriminant stored in the `kind` column so SQL can filter
    /// without parsing the payload.
    pub fn discriminant(&self) -> &'static str {
        match self {
            Self::RunStarted => "run_started",
            Self::EffectIntended { .. } => "effect_intended",
            Self::EffectCompleted { .. } => "effect_completed",
            Self::RunCompleted => "run_completed",
            Self::RunTerminal { .. } => "run_terminal",
        }
    }
}

/// One journal entry. Primary key `(run_id, sequence)`.
///
/// There is no timestamp: the spike keeps the journal free of any
/// environment-derived value so an event stream is byte-reproducible across
/// machines and can be pinned by a golden fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvent {
    /// Envelope version, written to its own column at append time and READ
    /// BACK from the row. Re-injecting the current constant at read time
    /// would make the field agree with itself forever — a version nobody can
    /// disagree with is not a version.
    pub format_version: u16,
    /// Owning run.
    pub run_id: String,
    /// Monotonic per-run sequence, starting at 0.
    pub sequence: u64,
    /// Node instance the entry is about, when it is about one.
    pub node_instance_id: Option<String>,
    /// Step request hash, present on the entries that carry a step identity.
    pub request_hash: Option<String>,
    /// What happened.
    pub kind: RunEventKind,
}
