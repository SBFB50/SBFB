// SPDX-License-Identifier: AGPL-3.0-or-later
//! `RunEvent.v1` — one append-only entry of the run journal (D3), and the
//! pure upcasting door that interprets older journal payloads.
//!
//! Authored in Sprint 83 Phase I (the run-store phase, as `contracts.rs`
//! assigned it). The envelope lived in `sbfb-workflow-engine::event` during
//! the Phase E spike, deliberately NOT filling this identity "while the
//! GO/NO-GO question is still open" — the spike returned GO, that rationale
//! expired, and the types moved here following the cadence every other
//! filled identity set (D → F → G → H: fields authored in a dedicated
//! module, re-exported from [`crate::contracts`]). The engine re-exports
//! them, so its callers see the same paths as before.
//!
//! ## What the journal alone guarantees
//!
//! The DURABLE STORE is the source of truth for replay — and that store is
//! the journal PLUS the outbox, not the journal alone. The receipt, the
//! pending request hash, the target's idempotency claim and its dedup scope
//! all live in the outbox row, deliberately, so that no material payload
//! ever reaches the journal. What the journal alone guarantees is the ORDER
//! and the OCCURRENCE of what happened.
//!
//! ## Pre-launch protocol
//!
//! [`RUN_EVENT_FORMAT_VERSION`] is a version of the LOCAL run-journal
//! envelope, not a wire format: journal rows never leave the node and are
//! never signed (`DOMAIN_RUN_EVENT_V1` stays a RESERVED tag with zero
//! producers, for a FUTURE exported form that is a different artefact).
//! The envelope is versioned ONCE; evolution goes through [`upcast_payload`]
//! at interpretation time (the feed raw-op policy applied to a local store),
//! never through a destructive rewrite of journal rows.
//!
//! ## Data posture
//!
//! [`RunEventKind`] can hold an effect id and a receipt DIGEST. It has NO
//! FIELD DEDICATED to receipt content, prompt text or a secret — absent by
//! TYPE, not skipped at serialization time, so no `Debug` render, log line
//! or journal row carries a payload.
//!
//! What that does NOT claim: `run_id` and `node_instance_id` are
//! caller-chosen `String`s, validated (engine-side) only for length and
//! control characters. A caller who puts a token or an email in an
//! identifier persists it in the journal. Identifiers are the caller's to
//! choose; the guarantee is about payload FIELDS, not about what someone
//! names their run.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Version of the local run-journal envelope. Bumped when a NEW payload
/// shape starts being WRITTEN; every older version stays readable through
/// the upcaster chain, interpreted — never migrated — at read time.
///
/// Pre-launch nuance (CLAUDE.md protocol policy): while no released journal
/// exists, ADDING variants to [`RunEventKind`] REDEFINES version 1 rather
/// than bumping — Phase J added its scheduler-decision entries
/// (`branch_taken`, `map_fanout`, `signal_consumed`,
/// `human_decision_recorded`, `cancel_requested`) exactly this way. The
/// constant moves the day a released journal must stay readable across the
/// change, and its first bump must bring the missing upcaster hop with it
/// (the chain-length invariant fails closed otherwise).
pub const RUN_EVENT_FORMAT_VERSION: u16 = 1;

/// Why a run reached a terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    /// The run was cancelled (research §12.7 honest cancel, Sprint 83
    /// Phase J): emission stopped, in-flight results were KEPT in the
    /// journal but not propagated, and the run closed as cancelled — never
    /// re-labelled completed. The `cancel_requested` journal entry carries
    /// WHO asked ([`CancelOrigin`]); this reason records only the outcome.
    Cancelled,
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
            Self::Cancelled => "cancelled",
        }
    }
}

/// Who asked for a cancellation. A CLOSED set: cancellation authority is
/// host/timeout only (`CancellationPolicy` doc, graph.rs) — no variant
/// exists that a model output could route to, and none carries free-form
/// text.
///
/// `BudgetExhausted` is deliberately ABSENT: real budget consumption is
/// metered by the real-adapter layer (Phase K) — inventing the variant
/// before anything can trigger it would be a legacy decoder for a format
/// that never shipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CancelOrigin {
    /// The host operator asked. Always honoured: a pack cannot opt out of
    /// operator authority (`allow_host_operator_cancel` is a claim the
    /// engine refuses to let opt out — Phase J enforcement).
    HostOperator,
    /// The compiled plan's `workflow_timeout_seconds` deadline expired and
    /// its `cancellation_policy.cancel_on_timeout` is set. The deadline is
    /// an ABSOLUTE wall-clock value stored in the durable timer table at
    /// run start — never re-derived on replay.
    WorkflowTimeout,
}

/// What happened at one point of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    /// A ROUTING decision was taken at the node this entry is about
    /// (Sprint 83 Phase J). The journal records every scheduler decision
    /// that is NOT derivable from plan topology alone, so the reducer
    /// rebuilds control flow WITHOUT parsing any material payload — the
    /// replay-divergence mitigation (THREAT §19): journal the decision
    /// BEFORE acting on it.
    ///
    /// `port` is the branch label actually taken: a `switch`/`classifier`
    /// output label (or the reserved `default`), or the engine-internal
    /// `continue`/`stop` verdict of a bounded `repeat` iteration.
    BranchTaken {
        /// Output port (or repeat verdict) the flow continues on.
        port: String,
    },
    /// A `map` node decided its fan-out (Phase J). The cardinality is the
    /// RUNTIME collection size, already proven ≤ the structural
    /// `max_cardinality` of the signed plan — an oversized collection is
    /// refused BEFORE this entry is written, so a journaled fan-out is
    /// always a bounded one.
    MapFanout {
        /// Number of child iterations the map dispatched.
        cardinality: u32,
    },
    /// A `wait.signal` interruption consumed its mono-usage signal
    /// (research §12.6, Phase J). Written in the SAME transaction that
    /// marks the deposited signal consumed — a replayed delivery of the
    /// same nonce can never re-fire the resume.
    SignalConsumed {
        /// Static signal name of the awaited node.
        signal_name: String,
        /// The engine-derived mono-usage nonce this consumption spent.
        nonce: String,
        /// Digest of the signal payload — never the payload itself.
        payload_digest: String,
    },
    /// A `human.review` interruption consumed its decision (Phase J).
    /// The decision label is validated against the node's closed
    /// `decision_labels` BEFORE this entry is written. The signed HITL
    /// principal/approval binding is Phase L1 — this entry records the
    /// consumption, not an authenticated approval.
    HumanDecisionRecorded {
        /// The engine-derived mono-usage nonce this consumption spent.
        nonce: String,
        /// The decision label taken, one of the node's closed set.
        decision_label: String,
        /// Digest of the decision payload — never the payload itself.
        payload_digest: String,
    },
    /// A cancellation was requested (research §12.7, Phase J). From this
    /// entry on the scheduler emits NO new effect; results of effects
    /// already in flight are kept in the journal but not propagated, and
    /// the run closes as `cancelled`.
    CancelRequested {
        /// Who asked — a closed set, never free-form.
        origin: CancelOrigin,
    },
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
            Self::BranchTaken { .. } => "branch_taken",
            Self::MapFanout { .. } => "map_fanout",
            Self::SignalConsumed { .. } => "signal_consumed",
            Self::HumanDecisionRecorded { .. } => "human_decision_recorded",
            Self::CancelRequested { .. } => "cancel_requested",
        }
    }
}

/// One journal entry. Primary key `(run_id, sequence)`.
///
/// There is no timestamp: the journal is kept free of any
/// environment-derived value so an event stream is byte-reproducible across
/// machines and can be pinned by a golden fixture. (Wall-clock values the
/// store DOES hold — lease expiry, retention ages — live in state tables,
/// never in journal rows.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "RunEvent.v1")]
pub struct RunEvent {
    /// Envelope version, written to its own column at append time and READ
    /// BACK from the row. Re-injecting the current constant at read time
    /// would make the field agree with itself forever — a version nobody can
    /// disagree with is not a version. After upcasting, this field still
    /// carries the version the ROW was written under (provenance); only
    /// `kind` is brought to the current shape.
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

/// One upcasting hop: transforms a payload `Value` written under version `n`
/// into the shape of version `n + 1`. Pure data-to-data — an upcaster is
/// NEVER code carried by the store, never a script, never a migration that
/// rewrites rows (the checkpoint security doctrine, D3.6: state stays pure
/// JSON, interpreted through functions compiled into THIS binary).
pub type RunEventUpcaster = fn(serde_json::Value) -> Result<serde_json::Value, String>;

/// The production upcaster chain: `RUN_EVENT_UPCASTERS[j]` upcasts version
/// `j + 1` to version `j + 2`, so the chain covers `1 .. RUN_EVENT_FORMAT_VERSION`
/// and its length is ALWAYS `RUN_EVENT_FORMAT_VERSION - 1`.
///
/// Empty today: version 1 is the FIRST version ever written (pre-launch
/// protocol — no released journal exists), so there is nothing to upcast
/// FROM. The chain-length invariant is enforced fail-closed by
/// [`upcast_payload`], so a future bump that forgets its hop is a loud
/// error, never a silent skip. No hop for a version that never shipped is
/// deliberately authored (a legacy decoder for a format that never existed
/// is the zombie class the pre-launch policy deletes on sight).
pub const RUN_EVENT_UPCASTERS: &[RunEventUpcaster] = &[];

/// Upcasting refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpcastError {
    /// Version 0 never existed; a row claiming it is corrupt, not old.
    VersionZero,
    /// The row was written by a NEWER engine than this one. Interpreting it
    /// anyway would be the "downgrade dangereux" the plan refuses: an old
    /// binary silently mis-reading shapes it does not know. Fail closed.
    NewerThanSupported {
        /// Version found on the row.
        found: u16,
        /// Highest version this binary can interpret.
        supported: u16,
    },
    /// The chain does not cover `1 .. current`. A gap means some version has
    /// no path to the present — refusing loudly here is what makes "every
    /// old run replays after an upgrade" enforceable instead of aspirational.
    ChainGap {
        /// Hops in the chain.
        chain_len: usize,
        /// Current envelope version the chain must reach.
        current: u16,
    },
    /// One hop refused its payload.
    Step {
        /// Version the failing hop upcasts FROM.
        from_version: u16,
        /// Why the hop refused.
        reason: String,
    },
}

impl std::fmt::Display for UpcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionZero => write!(f, "format version 0 never existed"),
            Self::NewerThanSupported { found, supported } => write!(
                f,
                "row written by a newer engine (format version {found}, this binary supports up to {supported})"
            ),
            Self::ChainGap { chain_len, current } => write!(
                f,
                "upcaster chain has {chain_len} hop(s) but must cover versions 1..{current}"
            ),
            Self::Step {
                from_version,
                reason,
            } => write!(f, "upcaster from version {from_version} refused: {reason}"),
        }
    }
}

impl std::error::Error for UpcastError {}

/// Bring a payload written under `found` to the shape of `current`, through
/// `chain` — AT INTERPRETATION TIME. The journal row itself is never
/// rewritten.
///
/// Contract (all fail-closed, none silent):
/// - `chain.len()` MUST equal `current - 1` ([`UpcastError::ChainGap`]) —
///   checked FIRST, even for a row already at `current`, so a mis-assembled
///   chain cannot hide behind fresh rows;
/// - `found == 0` is refused ([`UpcastError::VersionZero`]);
/// - `found > current` is refused ([`UpcastError::NewerThanSupported`]) —
///   the downgrade guard;
/// - otherwise hops `chain[found - 1 ..]` run in order, each moving the
///   payload up exactly one version.
pub fn upcast_payload(
    chain: &[RunEventUpcaster],
    current: u16,
    found: u16,
    payload: serde_json::Value,
) -> Result<serde_json::Value, UpcastError> {
    if chain.len() != usize::from(current).saturating_sub(1) {
        return Err(UpcastError::ChainGap {
            chain_len: chain.len(),
            current,
        });
    }
    if found == 0 {
        return Err(UpcastError::VersionZero);
    }
    if found > current {
        return Err(UpcastError::NewerThanSupported {
            found,
            supported: current,
        });
    }
    let mut value = payload;
    for (index, hop) in chain.iter().enumerate().skip(usize::from(found) - 1) {
        let from_version = u16::try_from(index + 1).expect("chain length bounded by u16 current");
        value = hop(value).map_err(|reason| UpcastError::Step {
            from_version,
            reason,
        })?;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Synthetic hop v1→v2: renames `"a"` to `"b"`. Refuses a payload
    /// without `"a"` so a skipped or mis-ordered hop is observable.
    fn hop_v1_to_v2(value: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut obj = match value {
            serde_json::Value::Object(o) => o,
            other => return Err(format!("expected object, got {other}")),
        };
        let a = obj.remove("a").ok_or("missing field 'a'")?;
        obj.insert("b".to_string(), a);
        Ok(serde_json::Value::Object(obj))
    }

    /// Synthetic hop v2→v3: renames `"b"` to `"c"`. Refuses without `"b"`.
    fn hop_v2_to_v3(value: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut obj = match value {
            serde_json::Value::Object(o) => o,
            other => return Err(format!("expected object, got {other}")),
        };
        let b = obj.remove("b").ok_or("missing field 'b'")?;
        obj.insert("c".to_string(), b);
        Ok(serde_json::Value::Object(obj))
    }

    const SYNTHETIC_CHAIN: &[RunEventUpcaster] = &[hop_v1_to_v2, hop_v2_to_v3];

    /// PER-HOP coverage (S1a Axon precedent): each starting version crosses
    /// exactly the hops between it and current — v1 crosses both, v2 crosses
    /// the second only, v3 (current) crosses none.
    #[test]
    fn upcast_runs_exactly_the_hops_between_found_and_current() {
        let v1 = json!({"a": 1});
        assert_eq!(
            upcast_payload(SYNTHETIC_CHAIN, 3, 1, v1).expect("v1 upcasts"),
            json!({"c": 1})
        );
        let v2 = json!({"b": 2});
        assert_eq!(
            upcast_payload(SYNTHETIC_CHAIN, 3, 2, v2).expect("v2 upcasts"),
            json!({"c": 2})
        );
        let v3 = json!({"c": 3});
        assert_eq!(
            upcast_payload(SYNTHETIC_CHAIN, 3, 3, v3).expect("v3 passes through"),
            json!({"c": 3})
        );
    }

    /// RED CONTROL — the hops are LOAD-BEARING, not decorative: a v1 payload
    /// entering the wrong hop (chain missing its first hop ⇒ ChainGap; hop
    /// applied to the wrong shape ⇒ Step error), never a silent pass-through.
    #[test]
    fn upcast_wrong_shape_at_a_hop_fails_loudly() {
        // A v2-shaped payload declared as v1 hits hop_v1_to_v2, which
        // refuses it — proving each hop actually transforms.
        let mislabelled = json!({"b": 2});
        let err = upcast_payload(SYNTHETIC_CHAIN, 3, 1, mislabelled).unwrap_err();
        assert!(
            matches!(
                err,
                UpcastError::Step {
                    from_version: 1,
                    ..
                }
            ),
            "got {err:?}"
        );
    }

    /// RED CONTROL — a chain that does not cover 1..current is refused even
    /// for a row ALREADY at current: a mis-assembled chain must not hide
    /// behind fresh rows.
    #[test]
    fn upcast_chain_gap_is_refused_even_at_current_version() {
        let short: &[RunEventUpcaster] = &[hop_v1_to_v2];
        let err = upcast_payload(short, 3, 3, json!({"c": 1})).unwrap_err();
        assert_eq!(
            err,
            UpcastError::ChainGap {
                chain_len: 1,
                current: 3
            }
        );
    }

    /// Downgrade guard: a row from a NEWER engine is refused, version 0 is
    /// refused as corrupt.
    #[test]
    fn upcast_refuses_newer_and_version_zero() {
        let err = upcast_payload(SYNTHETIC_CHAIN, 3, 4, json!({})).unwrap_err();
        assert_eq!(
            err,
            UpcastError::NewerThanSupported {
                found: 4,
                supported: 3
            }
        );
        let err = upcast_payload(SYNTHETIC_CHAIN, 3, 0, json!({})).unwrap_err();
        assert_eq!(err, UpcastError::VersionZero);
    }

    /// The PRODUCTION chain honours the length invariant for the current
    /// version — the compile-time pair (`RUN_EVENT_UPCASTERS`,
    /// `RUN_EVENT_FORMAT_VERSION`) can never drift apart without this test
    /// going red.
    #[test]
    fn production_chain_covers_current_version() {
        assert_eq!(
            RUN_EVENT_UPCASTERS.len(),
            usize::from(RUN_EVENT_FORMAT_VERSION) - 1,
            "RUN_EVENT_UPCASTERS must hold exactly one hop per version below current"
        );
        // And a current-version payload passes through unchanged.
        let payload = json!({"kind": "run_started"});
        assert_eq!(
            upcast_payload(
                RUN_EVENT_UPCASTERS,
                RUN_EVENT_FORMAT_VERSION,
                RUN_EVENT_FORMAT_VERSION,
                payload.clone()
            )
            .expect("current version passes"),
            payload
        );
    }
}
