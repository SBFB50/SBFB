// SPDX-License-Identifier: AGPL-3.0-or-later
//! The activity layer (Sprint 83 Phase K): the NON-BLOCKING effect adapter
//! the orchestrator drives — start, poll, cancel — next to the FROZEN
//! blocking [`crate::echo::EffectTarget`] of the Phase E spike.
//!
//! ## Why a second trait instead of extending `EffectTarget`
//!
//! `SpikeEngine::execute` calls `EffectTarget::apply` and is the frozen
//! proof of Phase E — mutating that signature would disturb the proof.
//! And a BLOCKING `apply` makes three Phase K deliverables unreachable:
//! a "late result after cancel" has no window to exist in (the dispatch
//! blocks until completion and cancel is checked before dispatch), a
//! descendant process tree cannot be killed DURING a run, and the fencing
//! token cannot be verified target-side. The [`Activity`] trait carries
//! all three; only the orchestrator consumes it.
//!
//! ## Fencing, target-side (closing the Phase E clause-3 window)
//!
//! The store's epoch CAS fences STORE writes; the window between a
//! committed intent and the target-side application belonged to the
//! target's deduplication alone (`lib.rs`, concurrency clause). An
//! [`EffectContext`] therefore carries the writer's lease epoch, and a
//! fencing-aware target REFUSES an application whose epoch is lower than
//! the highest it has durably seen ([`FencedFileActivity`] is the
//! reference implementation). Honest bound: the refusal is as atomic as
//! the TARGET's own check-and-apply — a target that cannot check and
//! apply atomically in its own backend narrows the window, it does not
//! close it for concurrent writers outside the single-writer model.
//!
//! ## Receipts (§17.1 role 6 — the LOCAL activity receipt)
//!
//! [`ActivityReceipt`] is the structured receipt of one local activity:
//! it binds the FULL idempotency tuple (run, node, effect id, attempt,
//! request hash) plus the activity's name, dedup scope, fencing epoch and
//! outcome facts, canonicalised through the crate's single JCS door. It is
//! NOT Ed25519-signed: the census of `DOMAIN_*_V1` signing families stays
//! frozen at 29, the crate stays keyless (host-neutral charter), and the
//! rich provider attestation (model/CLI/version) is the Phase O adapter
//! layer (`G-CLAUDE-CODEX-RECEIPT`). The receipt's material lives in the
//! outbox row; only its `receipt_digest` reaches the journal — the journal
//! is LINKED to the material by that digest (anti-leak: no prompt/output in
//! the journal), but the material is NOT re-verified against the digest AT
//! READ TIME (A11: an adversary with SQLite file write access rewrites
//! both; read-time-verified + signed receipts are Phase O).

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::hash::{HashError, IdempotencyKey, canonical_string};

/// Self-describing kind tag of an [`ActivityReceipt`] rendering. A LOCAL
/// marker (module doc: not a signing domain) — it lets the
/// value-resolution layer distinguish a structured receipt from the
/// spike's raw echo map without guessing.
pub const ACTIVITY_RECEIPT_KIND: &str = "sbfb.workflow.engine.activity-receipt.v0";

/// Epoch recorded when the writer holds no lease (the spike's unfenced
/// single-process mode). Any fenced application outranks it.
pub const UNFENCED_EPOCH: i64 = 0;

/// Everything one attempt of an effect is allowed to know. Built by the
/// orchestrator; the SAME context (same key, same attempt) must be
/// presented to `poll`/`cancel` that was presented to `start` — an
/// activity keys its in-flight work by [`IdempotencyKey::as_line`].
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// The idempotency key `(run_id, node_instance_id, effect_id)` —
    /// stable across attempts.
    pub key: IdempotencyKey,
    /// 1-based attempt number — changes at each retry (§12.5).
    pub attempt: u32,
    /// The writer's lease epoch, [`UNFENCED_EPOCH`] when no lease is
    /// held. A fencing-aware target refuses a lower epoch than the
    /// highest it has durably seen.
    pub fencing_epoch: i64,
    /// ABSOLUTE deadline of this attempt (the orchestrator judges it
    /// against the watermarked clock; the activity may use it to bound
    /// its own work).
    pub deadline_ms: i64,
    /// The effect input: `{inbound port → canonical JCS of the port's
    /// value}` — the same map the request hash covers.
    pub input: BTreeMap<String, String>,
}

/// What one poll observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollOutcome {
    /// Still in flight.
    Pending,
    /// The effect confirmed: material output for downstream resolution,
    /// plus closed outcome FACTS for the receipt (exit code, byte counts —
    /// strings under closed keys, never prompt text).
    Ready {
        /// Output value delivered downstream (must canonicalise: no
        /// `null` anywhere — the crate's JCS door refuses it).
        output: serde_json::Value,
        /// Outcome facts recorded in the receipt.
        facts: BTreeMap<String, String>,
    },
    /// The attempt failed. `retryable: false` settles the node terminally
    /// at the first such failure.
    Failed {
        /// Whether the engine may try again under the same key.
        retryable: bool,
        /// Human-readable reason — surfaces in the ERROR path only, never
        /// in the journal (payload-free by type).
        reason: String,
    },
}

/// An activity refused or failed at the adapter boundary.
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    /// The context's fencing epoch is lower than the highest this
    /// target's backend has durably seen: the writer was deposed. The
    /// orchestrator surfaces this loudly — a deposed writer must stop,
    /// not journal.
    #[error("fenced out: target saw epoch {seen}, presented {presented}")]
    Fenced {
        /// Highest epoch durably recorded target-side.
        seen: i64,
        /// Epoch the deposed writer presented.
        presented: i64,
    },
    /// The activity holds no in-flight work under this key — the resume
    /// path after a process restart (in-memory state died with the
    /// previous incarnation). The orchestrator reaps the orphan and
    /// decides by the JOURNALED idempotency claim.
    #[error("no in-flight attempt under this key")]
    UnknownAttempt,
    /// A declared precondition does not hold (worktree state divergence).
    /// Fail-closed and NON-retryable: the world the plan was made against
    /// is gone; retrying cannot bring it back.
    #[error("precondition mismatch: {0}")]
    PreconditionMismatch(String),
    /// The activity refuses the request (malformed input, out-of-contract
    /// path…). Non-retryable.
    #[error("refused: {0}")]
    Refused(String),
    /// I/O failure at the adapter — retryable (transient by default).
    #[error("activity io: {0}")]
    Io(#[from] std::io::Error),
    /// Canonicalisation failure.
    #[error(transparent)]
    Hash(#[from] HashError),
}

/// A non-blocking effect adapter. One in-flight attempt per key at most —
/// the orchestrator's attempts table enforces it store-side, and the
/// activity may assume it.
pub trait Activity {
    /// Stable name, part of the hashed step request.
    fn name(&self) -> &str;

    /// Whether the target collapses two applications of the SAME key into
    /// one side output. Same trust contract as
    /// [`crate::echo::EffectTarget::is_idempotent`]: a claim the engine
    /// journals at intent time and re-checks on resume.
    fn is_idempotent(&self) -> bool;

    /// Stable identity of the DURABLE backend this target deduplicates
    /// in. Same contract as [`crate::echo::EffectTarget::dedup_scope`].
    fn dedup_scope(&self) -> String;

    /// Begin one attempt. Must NOT block on the effect's completion; a
    /// synchronous activity may do its work here and answer the first
    /// `poll` with `Ready`.
    fn start(&mut self, ctx: &EffectContext) -> Result<(), ActivityError>;

    /// Observe the attempt.
    fn poll(&mut self, ctx: &EffectContext) -> Result<PollOutcome, ActivityError>;

    /// Stop the attempt and release its resources (kill the process
    /// tree, drop buffers). Idempotent; unknown keys are a no-op.
    fn cancel(&mut self, ctx: &EffectContext) -> Result<(), ActivityError>;

    /// OS-level handle of the attempt's runtime work (a process-group
    /// id), for the store's bookkeeping — what an orphan-reaping resume
    /// passes back to [`Activity::reap_orphan`]. `None` for in-process
    /// activities.
    fn runtime_handle(&self, ctx: &EffectContext) -> Option<String> {
        let _ = ctx;
        None
    }

    /// Best-effort cleanup of an orphan left by a PREVIOUS incarnation
    /// (the in-memory state died; `runtime_handle` is the durable trace).
    /// Default: nothing to reap.
    fn reap_orphan(
        &mut self,
        ctx: &EffectContext,
        runtime_handle: Option<&str>,
    ) -> Result<(), ActivityError> {
        let _ = (ctx, runtime_handle);
        Ok(())
    }
}

/// The structured receipt of one LOCAL activity (§17.1 role 6) — see the
/// module doc for the journal-digest link and why it is not Ed25519-signed
/// here.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivityReceipt {
    /// Self-describing kind tag, always [`ACTIVITY_RECEIPT_KIND`].
    pub kind: String,
    /// Owning run.
    pub run_id: String,
    /// Node instance.
    pub node_instance_id: String,
    /// Engine-derived effect id.
    pub effect_id: String,
    /// Attempt that confirmed.
    pub attempt: u32,
    /// Step request hash the receipt answers.
    pub request_hash: String,
    /// Activity name that applied it.
    pub activity: String,
    /// Dedup backend token journaled at intent time.
    pub dedup_scope: String,
    /// Fencing epoch under which it applied ([`UNFENCED_EPOCH`] when
    /// unfenced).
    pub fencing_epoch: i64,
    /// Closed outcome facts (exit code, byte counts…).
    pub facts: BTreeMap<String, String>,
    /// Material output value for downstream resolution.
    pub output: serde_json::Value,
}

impl ActivityReceipt {
    /// Render the canonical (JCS) text stored in the outbox row — the
    /// exact bytes [`crate::hash::receipt_digest`] digests for the
    /// journal.
    pub fn render(&self) -> Result<String, HashError> {
        canonical_string(self)
    }

    /// Parse a receipt text back, `None` when it is not an activity
    /// receipt (the spike's raw echo map, or foreign bytes).
    pub fn parse(receipt: &str) -> Option<Self> {
        let parsed: Self = serde_json::from_str(receipt).ok()?;
        (parsed.kind == ACTIVITY_RECEIPT_KIND).then_some(parsed)
    }
}

/// Scripted test double: a deterministic sequence of poll outcomes per
/// node instance. Honest about what it is — no side output, no backend;
/// the discriminating side-output tests use [`FencedFileActivity`].
#[derive(Debug, Default)]
pub struct ScriptedActivity {
    name: String,
    idempotent: bool,
    scope: String,
    /// A durable OS handle this activity reports (so a test can prove the
    /// orphan-reap path receives the JOURNALED handle). `None` = the
    /// in-process default.
    handle: Option<String>,
    /// Remaining poll outcomes per `node_instance_id` (front = next).
    scripts: BTreeMap<String, Vec<PollOutcome>>,
    /// Every lifecycle call, for assertions: `start:<node>#<attempt>`,
    /// `poll:<node>`, `cancel:<node>`, `reap:<node>:<handle>`.
    pub calls: Vec<String>,
}

impl ScriptedActivity {
    /// A scripted activity declaring `idempotent`.
    pub fn new(name: impl Into<String>, idempotent: bool) -> Self {
        Self {
            name: name.into(),
            idempotent,
            scope: "scripted".to_string(),
            handle: None,
            scripts: BTreeMap::new(),
            calls: Vec::new(),
        }
    }

    /// Report a durable runtime handle — the orchestrator persists it, and
    /// a later orphan-reap must receive it back.
    pub fn with_handle(mut self, handle: impl Into<String>) -> Self {
        self.handle = Some(handle.into());
        self
    }

    /// Program the poll outcomes of one node instance, in order. The last
    /// outcome repeats once the script is exhausted.
    pub fn script(&mut self, node_instance_id: impl Into<String>, outcomes: Vec<PollOutcome>) {
        self.scripts.insert(node_instance_id.into(), outcomes);
    }
}

impl Activity for ScriptedActivity {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_idempotent(&self) -> bool {
        self.idempotent
    }

    fn dedup_scope(&self) -> String {
        self.scope.clone()
    }

    fn start(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        self.calls.push(format!(
            "start:{}#{}",
            ctx.key.node_instance_id(),
            ctx.attempt
        ));
        Ok(())
    }

    fn poll(&mut self, ctx: &EffectContext) -> Result<PollOutcome, ActivityError> {
        let node = ctx.key.node_instance_id().to_string();
        self.calls.push(format!("poll:{node}"));
        let Some(script) = self.scripts.get_mut(&node) else {
            return Err(ActivityError::UnknownAttempt);
        };
        if script.len() > 1 {
            Ok(script.remove(0))
        } else {
            script.first().cloned().ok_or(ActivityError::UnknownAttempt)
        }
    }

    fn cancel(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        self.calls
            .push(format!("cancel:{}", ctx.key.node_instance_id()));
        Ok(())
    }

    fn runtime_handle(&self, _ctx: &EffectContext) -> Option<String> {
        self.handle.clone()
    }

    fn reap_orphan(
        &mut self,
        ctx: &EffectContext,
        runtime_handle: Option<&str>,
    ) -> Result<(), ActivityError> {
        // Record the handle RECEIVED — a test asserts it is the journaled
        // one, so replacing the caller's `open.runtime_handle` with `None`
        // is caught.
        self.calls.push(format!(
            "reap:{}:{}",
            ctx.key.node_instance_id(),
            runtime_handle.unwrap_or("<none>")
        ));
        Ok(())
    }
}

/// Field separator of the fenced side-output log — same domain as the
/// echo target's: components are control-character-free by validation.
const FIELD_SEPARATOR: char = '\t';

/// The fencing-aware reference target: an fsync'd append-only log OUTSIDE
/// the store, deduplicating on the idempotency key AND refusing any
/// application whose epoch is below the highest the log has seen. The
/// Phase K counterpart of [`crate::echo::FileEchoTarget`] — that one
/// stays byte-frozen with the spike.
#[derive(Debug)]
pub struct FencedFileActivity {
    log_path: PathBuf,
    /// Ready results stashed by `start`, keyed by
    /// [`IdempotencyKey::as_line`].
    ready: BTreeMap<String, serde_json::Value>,
    /// Fence check enabled — the red-control constructor turns it off.
    enforce_fence: bool,
}

/// One line of the fenced side-output log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FencedSideOutput {
    /// Rendered idempotency key.
    pub key_line: String,
    /// Epoch under which it was applied.
    pub epoch: i64,
    /// Canonical JCS of the output value.
    pub output_jcs: String,
}

/// Freeze a side-output path ONCE at construction (ex-Codex r5
/// K-FENCE-SCOPE-REL-05): canonical when the file resolves; for the
/// nominal not-yet-created log, the canonical PARENT joined with the leaf
/// name; raw as last resort. A relative spelling kept raw would give two
/// incarnations under different ambient cwd ONE durable scope over TWO
/// different logs — the fenced backend's dedup and fencing would silently
/// bind to nothing. (The spike's `FileEchoTarget` keeps its frozen
/// Phase-E clause-2 posture and is deliberately NOT retrofitted — it is
/// the sealed proof, not a production adapter.)
fn freeze_side_output_path(path: PathBuf) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
        // A bare leaf ("effects.log") has `parent() == Some("")` — treat
        // it as the CONSTRUCTION cwd (`.`) so the exact r5 scenario (one
        // scope string, two logs across two ambient cwd) cannot ride the
        // empty-parent branch (ex-Codex r6 K-FENCE-BARE-LEAF-06).
        let parent = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        if let Ok(canonical_parent) = parent.canonicalize() {
            return canonical_parent.join(name);
        }
    }
    path
}

impl FencedFileActivity {
    /// The honest fenced idempotent target. The log path is FROZEN here
    /// (see [`freeze_side_output_path`]) so, for the SUPPORTED inputs
    /// (absolute, or under an existing parent), the durable scope names
    /// ONE backend, not a spelling the ambient cwd re-interprets — the
    /// raw last-resort fallback stays an ambient dependency, same
    /// documented bound as the process cwd and the worktree root.
    pub fn new(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: freeze_side_output_path(log_path.into()),
            ready: BTreeMap::new(),
            enforce_fence: true,
        }
    }

    /// RED CONTROL — same target with the fence check REMOVED: proves the
    /// stale-writer refusal comes from the check, not from wishful
    /// thinking. Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn ignoring_fence(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: freeze_side_output_path(log_path.into()),
            ready: BTreeMap::new(),
            enforce_fence: false,
        }
    }

    /// Path of the side-output log.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Echo semantics shared with the spike target: the output value
    /// inverts the input map (single-port maps collapse to the port's
    /// parsed value) so a value passes through a leaf chain unchanged.
    fn echo_output(input: &BTreeMap<String, String>) -> Result<serde_json::Value, ActivityError> {
        if input.len() == 1 {
            let jcs = input.values().next().expect("len checked");
            return serde_json::from_str(jcs)
                .map_err(|e| ActivityError::Refused(format!("input port is not JSON: {e}")));
        }
        let mut obj = serde_json::Map::new();
        for (port, jcs) in input {
            let value: serde_json::Value = serde_json::from_str(jcs)
                .map_err(|e| ActivityError::Refused(format!("input port {port}: {e}")))?;
            obj.insert(port.clone(), value);
        }
        Ok(serde_json::Value::Object(obj))
    }
}

/// Read every fenced side output recorded so far — usable from a process
/// that did not produce them (the out-of-process counting the harness
/// relies on). A missing log is an empty list.
pub fn read_fenced_side_outputs(path: &Path) -> Result<Vec<FencedSideOutput>, ActivityError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(ActivityError::Io(e)),
    };
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    let mut out = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, FIELD_SEPARATOR);
        let (key_line, epoch_raw, output_jcs) = match (parts.next(), parts.next(), parts.next()) {
            (Some(k), Some(e), Some(o)) => (k, e, o),
            _ => {
                return Err(ActivityError::Refused(format!(
                    "side-output log corrupt at line {}",
                    index + 1
                )));
            }
        };
        let epoch: i64 = epoch_raw.parse().map_err(|_| {
            ActivityError::Refused(format!("side-output epoch corrupt at line {}", index + 1))
        })?;
        out.push(FencedSideOutput {
            key_line: key_line.to_string(),
            epoch,
            output_jcs: output_jcs.to_string(),
        });
    }
    Ok(out)
}

/// How many side outputs one key produced — the falsifiable quantity.
pub fn fenced_side_output_count(path: &Path, key: &IdempotencyKey) -> Result<usize, ActivityError> {
    let key_line = key.as_line();
    Ok(read_fenced_side_outputs(path)?
        .into_iter()
        .filter(|s| s.key_line == key_line)
        .count())
}

impl Activity for FencedFileActivity {
    fn name(&self) -> &str {
        "fenced-file"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn dedup_scope(&self) -> String {
        self.log_path.display().to_string()
    }

    fn start(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        let key_line = ctx.key.as_line();
        let existing = read_fenced_side_outputs(&self.log_path)?;

        // Fence FIRST: a deposed writer must not even collapse into the
        // dedup path (its whole resume is illegitimate).
        if self.enforce_fence {
            let seen = existing.iter().map(|s| s.epoch).max().unwrap_or(i64::MIN);
            if seen > ctx.fencing_epoch {
                return Err(ActivityError::Fenced {
                    seen,
                    presented: ctx.fencing_epoch,
                });
            }
        }

        // Dedup on the key: an already-applied key returns the recorded
        // output and appends nothing.
        if let Some(done) = existing.iter().find(|s| s.key_line == key_line) {
            let output: serde_json::Value = serde_json::from_str(&done.output_jcs)
                .map_err(|e| ActivityError::Refused(format!("recorded output corrupt: {e}")))?;
            self.ready.insert(key_line, output);
            return Ok(());
        }

        let output = Self::echo_output(&ctx.input)?;
        let output_jcs = canonical_string(&output)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(
            file,
            "{key_line}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{output_jcs}",
            ctx.fencing_epoch
        )?;
        // The side output must outlive an abrupt process exit (the spike
        // target's rationale, unchanged).
        file.sync_all()?;
        self.ready.insert(key_line, output);
        Ok(())
    }

    fn poll(&mut self, ctx: &EffectContext) -> Result<PollOutcome, ActivityError> {
        match self.ready.get(&ctx.key.as_line()) {
            Some(output) => Ok(PollOutcome::Ready {
                output: output.clone(),
                facts: BTreeMap::new(),
            }),
            None => Err(ActivityError::UnknownAttempt),
        }
    }

    fn cancel(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        self.ready.remove(&ctx.key.as_line());
        Ok(())
    }
}
