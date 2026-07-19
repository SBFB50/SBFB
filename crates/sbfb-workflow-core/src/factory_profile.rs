// SPDX-License-Identifier: AGPL-3.0-or-later
//! `FactoryProcessProfile.v1` — the machine-readable Factory/process contract
//! (Sprint 83 Phase D).
//!
//! ## What this contract is
//!
//! The real sprint system of this repository is prose: it lives in
//! `docs/agent/PROCESS.md`, `docs/agent/AGENT_SYSTEM.md`,
//! `docs/agent/RRV_FACTORY_CONTRACT.md`, `docs/factory/FACTORY_GATES.md`, and
//! the two finality hooks under `.claude/hooks/`. A workflow runtime that wants
//! to *reason about* that process (which roles exist, which gates block, which
//! commit-body sections are mandatory) cannot parse the prose reliably. This
//! module extracts the process into a single machine-readable profile —
//! `FactoryProcessProfile.v1` — and **binds it to the BLAKE3 digests of its
//! sources**. If a source doc or hook drifts, the embedded digest no longer
//! matches and the profile is [`ProfileStale`]: the extracted enumeration must
//! be re-verified against the changed source before it is trusted again. This
//! mirrors the Sprint 81 K binding "loaded-stage ↔ signed manifest, fail-closed".
//!
//! ## The `documented | enforced` axis
//!
//! Every process element carries an [`EnforcementKind`]:
//!
//! - [`EnforcementKind::Enforced`] — a mechanical gate on the AUTHORITATIVE
//!   commit/publish path blocks the process on violation (as opposed to pure
//!   prompt discipline or a merely advisory/warning check). Such an element
//!   carries a [`MechanismAnchor`] naming the file and a token that resolves
//!   inside it (a hook path + grep token, or a `gates.rs` symbol). The
//!   [`tests::documented_vs_enforced_marked`] test resolves every anchor against
//!   the real file, so an enforcement CLAIM cannot silently lose its token:
//!   delete `run_gate_csp_authoring` and the test goes red.
//! - [`EnforcementKind::Documented`] — carried by prompt/process discipline, or
//!   backed only by a warning / a bypassable-per-path check. No mechanism anchor.
//!
//! `Enforced` here means "the authoritative path mechanically blocks it under
//! normal operation", NOT "impossible to bypass under every condition". Every
//! gate has documented exemptions or scope limits, disclosed but NOT exhaustively
//! modelled (scope cut, see below). Roles and lifecycle modes are descriptive:
//! `Documented`, never `Enforced`.
//!
//! Two hook layers exist and `TOOLING.md` names which is authoritative: the
//! provider-neutral Git hooks (`.githooks/` → `scripts/agent/agentctl.py`) are
//! "the source of truth"; the Claude `PreToolUse` hooks (`.claude/hooks/`) are
//! "a faster feedback layer, not the source of truth". `agentctl
//! precommit-lightcheck` BLOCKS `G1` (design review — with an auditable,
//! declared `G1 skipped` exemption) and `G5` (staged/module coherence) on every
//! provider → `Enforced`. `G8` (preflight presence) is blocked ONLY by the
//! Claude lightcheck; the authoritative `agentctl` gate merely WARNs it → a
//! non-Claude commit bypasses it, so `G8` is `Documented`. The two finality
//! hooks (`Enforced`) block PHASE commits (`feat`/`fix`/`docs`/`test`/`refactor`)
//! — a missing Codex artifact / a review not reading `## Verdict: PASS`; `chore`
//! commits are out of that scope by design. `G2`-`G4`/`G6`-`G10` are prompt-carried.
//!
//! Among the publication gates, only `FG-CSP-authoring` is truly non-delegable:
//! it runs outside `--skip-gates` on BOTH the `publish` and `redeploy` paths
//! (`sbfb-factory` `pipeline.rs`, `atelier.rs`). `FG8` provenance runs outside
//! `--skip-gates` on `publish` but is ABSENT on the `redeploy` path and has a
//! documented N0-upload skip (`FACTORY_GATES.md`), so it is delegable →
//! `Documented`. `FG0`-`FG10` (spec/bypassable) are `Documented`.
//!
//! One asymmetry is deliberate and disclosed: a [`MechanismAnchor`] that points
//! into a file which is ALSO one of the six digested [`SourceId`] sources (`G1`,
//! `G5` and the two finality hooks anchor into the digested Claude hooks, which
//! mirror the authoritative `agentctl` blocks) gains `PROFILE_STALE` runtime
//! drift protection for free. The `FG-CSP-authoring` anchor points into
//! `crates/sbfb-factory/src/gates.rs`, which is NOT a digested source (churny
//! implementation code; digesting it would cry-wolf on every unrelated gate
//! edit), so its drift protection is test-time only (the
//! [`tests::documented_vs_enforced_marked`] negative control turns the suite red
//! if the symbol is gutted or renamed — a CI gate, not a runtime `PROFILE_STALE`).
//!
//! SCOPE CUT (disclosed residual). The enforcement markings are DERIVED from the
//! authoritative mechanism files — `scripts/agent/agentctl.py`, `.githooks/`,
//! `crates/sbfb-factory/src/pipeline.rs`, `gates.rs`, `docs/agent/TOOLING.md` —
//! which are deliberately NOT in the six-source digest set (they are churny code;
//! digesting them would cry-wolf). Consequently a SEMANTIC change to the
//! authoritative enforcement (e.g. `agentctl` ceasing to block `G1`) is caught by
//! the syntactic anchor test only for an anchored token, NOT by `PROFILE_STALE`.
//! Full drift protection of the enforcement decisions against those files, and an
//! exhaustive model of every per-gate exemption / skip-flag / path variation, are
//! future work. What Phase D binds fail-closed is the process-CONTRACT content
//! (roles, modes, gate identities, body, invariants) against its doc/hook sources.
//!
//! ## `gate-check` is a diagnostic, never an authority
//!
//! [`ProcessInvariants::gate_check_is_authority`] is `false`
//! (`AGENT_SYSTEM.md §2` gate-check row; `RRV_FACTORY_CONTRACT.md §2`: no
//! downstream component holds final verification authority). [`detect_stale`]
//! returns a diagnostic to its caller; nothing here wires a runtime process
//! verdict. The profile describes the process — it does not adjudicate it.
//!
//! ## Pre-launch protocol / wire
//!
//! `FactoryProcessProfile.v1` is a LOCAL, non-signed, non-gossiped entry golden.
//! It declares NO `DOMAIN_*_V1` and NO `*_FORMAT_VERSION` — inventing one to
//! "tag" the frontier would trip the frozen `DOMAIN_*_V1` census
//! (`scripts/check-frontier-contracts.sh`) and IS a wire bump by definition. Its
//! machine contract is (a) the generated schema snapshot pinned by
//! [`crate::contracts::all_contract_schemas`] against `contracts.schema.json`
//! (this identity is one of the seven, `len == 7`), and (b) the extracted
//! instance golden `factory_profile.golden.json` next to this module, pinned by
//! [`tests::extract_matches_golden`]. The BLAKE3 digest binding uses the
//! `blake3` workspace dependency (already resolved for the crypto crates, so no
//! new lock node).

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

/// The authoritative prose sources the profile is extracted from and bound to.
/// Each variant's [`SourceId::path`] is repo-relative from the workspace root.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum SourceId {
    /// `docs/agent/PROCESS.md` — the vendor-neutral sprint workflow: the
    /// Guarantee Matrix (`G1`-`G10`), the 9-section commit body, `PASS-PENDING`,
    /// the three verification blocks.
    Process,
    /// `docs/agent/AGENT_SYSTEM.md` — the role registry (9), the lifecycle modes
    /// (10), the preflight scans, the audit tracks, the gate verdicts.
    AgentSystem,
    /// `docs/agent/RRV_FACTORY_CONTRACT.md` — the authority order
    /// `process > RRV > Factory` and the "no downstream authority" principle.
    RrvFactoryContract,
    /// `docs/factory/FACTORY_GATES.md` — the publication pipeline `FG0`-`FG10`
    /// plus the non-delegable `FG-CSP-authoring`.
    FactoryGates,
    /// `.claude/hooks/phase-precommit-lightcheck.sh` — the mechanical precommit
    /// gate (Phase-A design review, staging coherence, preflight presence, Codex
    /// artifact).
    HookLightcheck,
    /// `.claude/hooks/phase-auditor-gate.sh` — the mechanical gate that blocks a
    /// phase commit unless the final review reads `## Verdict: PASS`.
    HookAuditorGate,
}

impl SourceId {
    /// Every source in the digest set, in a stable order.
    pub const ALL: [SourceId; 6] = [
        SourceId::Process,
        SourceId::AgentSystem,
        SourceId::RrvFactoryContract,
        SourceId::FactoryGates,
        SourceId::HookLightcheck,
        SourceId::HookAuditorGate,
    ];

    /// The repo-relative path of this source, from the workspace root.
    pub fn path(self) -> &'static str {
        match self {
            SourceId::Process => "docs/agent/PROCESS.md",
            SourceId::AgentSystem => "docs/agent/AGENT_SYSTEM.md",
            SourceId::RrvFactoryContract => "docs/agent/RRV_FACTORY_CONTRACT.md",
            SourceId::FactoryGates => "docs/factory/FACTORY_GATES.md",
            SourceId::HookLightcheck => ".claude/hooks/phase-precommit-lightcheck.sh",
            SourceId::HookAuditorGate => ".claude/hooks/phase-auditor-gate.sh",
        }
    }
}

/// `Documented` vs `Enforced` — the AM-2 enforcement axis. See the module docs
/// for the exact meaning: `Enforced` requires a resolvable [`MechanismAnchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementKind {
    /// Carried by prompt/process discipline or a bypassable/spec-level check.
    Documented,
    /// A mechanical gate on the authoritative path blocks the process on
    /// violation under normal operation (see the module docs — NOT a claim of
    /// impossible-to-bypass; documented exemptions/scope exist).
    Enforced,
}

/// Where an `Enforced` element's mechanism lives: a repo-relative `file` and a
/// `needle` token that must appear inside it. The test layer resolves every
/// anchor, so an `Enforced` marking whose mechanism vanished fails loudly
/// instead of lying (P72 anti-score / gate-discriminant discipline).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MechanismAnchor {
    /// Repo-relative path from the workspace root (e.g. a hook or `gates.rs`).
    pub file: String,
    /// A stable token that must be present in `file` for the marking to hold.
    pub needle: String,
}

/// One named element of the process (a role, a mode, a gate, or a hook) with
/// its enforcement marking, optional mechanism, and the source that declares it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessElement {
    /// Stable identifier (`driver`, `G1`, `FG-CSP-authoring`, `kickoff`, …).
    pub id: String,
    /// Short human title from the source.
    pub title: String,
    /// `Documented` or `Enforced`.
    pub enforcement: EnforcementKind,
    /// Present iff `enforcement == Enforced`; names a resolvable mechanism.
    pub mechanism: Option<MechanismAnchor>,
    /// Which digested source declares this element.
    pub source: SourceId,
}

impl ProcessElement {
    fn documented(id: &str, title: &str, source: SourceId) -> Self {
        ProcessElement {
            id: id.to_string(),
            title: title.to_string(),
            enforcement: EnforcementKind::Documented,
            mechanism: None,
            source,
        }
    }

    fn enforced(id: &str, title: &str, source: SourceId, file: &str, needle: &str) -> Self {
        ProcessElement {
            id: id.to_string(),
            title: title.to_string(),
            enforcement: EnforcementKind::Enforced,
            mechanism: Some(MechanismAnchor {
                file: file.to_string(),
                needle: needle.to_string(),
            }),
            source,
        }
    }
}

/// The BLAKE3 digest of one source, as embedded in the profile. `blake3` is the
/// lowercase hex digest of the LF/BOM-canonicalised bytes (see [`canonicalise`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceDigest {
    /// Which source this digest binds.
    pub id: SourceId,
    /// The source's repo-relative path (mirror of [`SourceId::path`]).
    pub path: String,
    /// Lowercase hex BLAKE3 of the canonicalised source bytes.
    pub blake3: String,
}

/// Boolean process invariants extracted from the sources. Each is a fact the
/// runtime can assert against, not a knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessInvariants {
    /// `PASS-PENDING` is never a committable verdict (`PROCESS.md` Phase Commit
    /// Finality; `AGENT_SYSTEM.md §5.2`).
    pub pass_pending_committable: bool,
    /// Codex output is raw, never rewritten by the driver
    /// (`PROCESS.md`; `AGENT_SYSTEM.md §5.3`).
    pub codex_output_rewritable: bool,
    /// The final `## Verdict: PASS` is only written after Codex reconciliation
    /// (`AGENT_SYSTEM.md §5.2`).
    pub reconciliation_required_before_pass: bool,
    /// `gate-check` is a diagnostic, never an authority (`AGENT_SYSTEM.md §2`;
    /// `RRV_FACTORY_CONTRACT.md §2`).
    pub gate_check_is_authority: bool,
}

// Factory/process-contract frontier (Sprint 83 Phase D, doctrine §7): the
// machine-readable identity of the repo's own sprint process, consumed by a
// distinct runtime — the Workflow Agents engine that reasons about which gates
// block and which roles exist. It is a LOCAL entry golden, not a signed wire
// type, so it carries no `// FRONTIER:` domain/version tag (that opt-in registry
// is for `DOMAIN_*_V1` families and inventing one here would trip the frozen
// census). Its machine contract is the generated schema snapshot pinned by
// `contracts::all_contract_schemas` (this identity is one of the seven, len==7)
// plus the extracted instance golden `factory_profile.golden.json` below; the
// GUIDE + llms.txt closure is deferred to the S83 wrap-up phase per AM-3.
/// `FactoryProcessProfile.v1` — the extracted, digest-bound Factory/process
/// contract. Assembled by [`extract`] from the six [`SourceId`] sources;
/// [`detect_stale`] re-checks it against current sources, fail-closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "FactoryProcessProfile.v1")]
pub struct FactoryProcessProfile {
    /// The 9 process roles (`AGENT_SYSTEM.md §2`), all `Documented`.
    pub roles: Vec<ProcessElement>,
    /// The 10 lifecycle modes (`AGENT_SYSTEM.md §4`), all `Documented`.
    pub modes: Vec<ProcessElement>,
    /// The process gates `G1`-`G10` (`PROCESS.md` Guarantee Matrix). `G1` and
    /// `G5` are `Enforced` (blocked by the authoritative `agentctl` gate,
    /// mirrored by the Claude lightcheck); the rest — including `G8`, which the
    /// authoritative gate only WARNs — are `Documented`.
    pub process_gates: Vec<ProcessElement>,
    /// The publication gates `FG0`-`FG10` plus `FG-CSP-authoring`
    /// (`FACTORY_GATES.md`). Only `FG-CSP-authoring` is `Enforced` (truly
    /// non-delegable: publish + redeploy); the rest — including `FG8`, which is
    /// skipped on the redeploy path and by the N0 exception — are `Documented`.
    pub publication_gates: Vec<ProcessElement>,
    /// The two finality hooks, both `Enforced`.
    pub finality_hooks: Vec<ProcessElement>,
    /// The five G8 preflight scans (`AGENT_SYSTEM.md §5.1`).
    pub preflight_scans: Vec<String>,
    /// The nine mandatory commit-body sections (`PROCESS.md` Phase Commit Finality).
    pub commit_body_sections: Vec<String>,
    /// The three full verification blocks (`PROCESS.md` Quality Bar).
    pub verification_blocks: Vec<String>,
    /// The nine audit-gate tracks (`AGENT_SYSTEM.md §5.4`). Note: `README §7.1`
    /// Cas A lists an extended 11-track canon (adds `J`/`K` standing); the bound
    /// authority here is `AGENT_SYSTEM.md §5.4`, so the count is 9.
    pub audit_tracks: Vec<String>,
    /// Authority order `process > RRV > Factory` (`RRV_FACTORY_CONTRACT.md §2`).
    pub authority_order: Vec<String>,
    /// Boolean process invariants.
    pub invariants: ProcessInvariants,
    /// The BLAKE3 digests of the six sources, in [`SourceId::ALL`] order.
    pub sources: Vec<SourceDigest>,
}

/// Failure to extract a profile from the supplied sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    /// A [`SourceId`] from [`SourceId::ALL`] was missing from the input.
    MissingSource(SourceId),
    /// A [`SourceId`] was supplied more than once — ambiguous input, rejected
    /// rather than silently taking the first (fail-closed construction).
    DuplicateSource(SourceId),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::MissingSource(id) => {
                write!(f, "missing required source: {}", id.path())
            }
            ExtractError::DuplicateSource(id) => {
                write!(f, "duplicate source supplied: {}", id.path())
            }
        }
    }
}

impl std::error::Error for ExtractError {}

/// Why a profile is stale relative to the current sources or its own canonical
/// content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleReason {
    /// A source (embedded in the profile, or in the current input) was absent.
    Missing,
    /// The current source digest differs from the embedded one.
    DigestMismatch,
    /// The source bindings are not well-formed: the embedded set is not exactly
    /// [`SourceId::ALL`] in order (empty, partial, duplicated, extra, or
    /// reordered → `source_id` is `None`), or a specific binding is bad — a
    /// duplicated current entry, or an embedded `path` that does not mirror its id
    /// (`source_id` names it). Treated as stale, not fresh — an ambiguous or
    /// tampered binding must never pass fail-closed.
    Malformed,
    /// The profile's CONTENT (roles, modes, gates, hooks, invariants …) diverges
    /// from the canonical enumeration, even though its source digests are fresh.
    /// A deserialized or tampered payload — e.g. an `Enforced` gate flipped to
    /// `Documented`, or `process_gates` cleared — is stale, never fresh. Not
    /// attributable to one source, so [`ProfileStale::source_id`] is `None`.
    ContentMismatch,
}

/// A profile that no longer matches its sources or its canonical content —
/// fail-closed. `source_id` names WHICH source drifted (or `None` for a whole-
/// profile [`StaleReason::ContentMismatch`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileStale {
    /// The source whose binding is stale, or `None` for a content mismatch.
    pub source_id: Option<SourceId>,
    /// Missing, digest mismatch, malformed binding, or content mismatch.
    pub reason: StaleReason,
}

impl std::fmt::Display for ProfileStale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match self.reason {
            StaleReason::Missing => "source missing",
            StaleReason::DigestMismatch => "source digest mismatch",
            StaleReason::Malformed => {
                "malformed source binding (wrong shape/order, duplicate, or path mismatch)"
            }
            StaleReason::ContentMismatch => {
                "profile content diverges from the canonical enumeration"
            }
        };
        match self.source_id {
            Some(id) => write!(f, "PROFILE_STALE: {} for {}", what, id.path()),
            None => write!(f, "PROFILE_STALE: {what}"),
        }
    }
}

impl std::error::Error for ProfileStale {}

/// Canonicalise source bytes before hashing: strip a leading UTF-8 BOM and fold
/// paired `\r\n` into `\n` (so a CRLF checkout hashes identically to LF), while
/// PRESERVING a lone `\r`. Defence-in-depth on top of `.gitattributes eol=lf`:
/// a stray BOM or a CRLF checkout never triggers a false [`ProfileStale`] between
/// Windows and Linux, yet a genuinely different byte sequence — including a bare
/// carriage return, which is significant in shell source — still yields a
/// different digest, so "any real source drift ⇒ stale" holds fail-closed.
fn canonicalise(bytes: &[u8]) -> Vec<u8> {
    let src = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if src[i] == b'\r' && src.get(i + 1) == Some(&b'\n') {
            out.push(b'\n');
            i += 2;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

/// Lowercase hex BLAKE3 of the canonicalised bytes. Each source is hashed
/// independently — never a length-ambiguous concatenation of name+content.
fn digest_hex(bytes: &[u8]) -> String {
    blake3::hash(&canonicalise(bytes)).to_hex().to_string()
}

fn roles() -> Vec<ProcessElement> {
    use SourceId::AgentSystem as A;
    vec![
        ProcessElement::documented("driver", "primary implementation agent", A),
        ProcessElement::documented("reviewer", "independent quality pass", A),
        ProcessElement::documented("auditor", "independent 7-dimension audit", A),
        ProcessElement::documented(
            "codex-verifier",
            "external cross-verification, raw output",
            A,
        ),
        ProcessElement::documented("researcher", "fact-finding, no code, no verdict", A),
        ProcessElement::documented(
            "preflight-checker",
            "5 scans S1-S4, no DESIGN-CONFLICT ruling",
            A,
        ),
        ProcessElement::documented("kickoff-author", "kickoff + plan + design review", A),
        ProcessElement::documented("audit-gate-runner", "9-track sprint audit", A),
        ProcessElement::documented("gate-check", "diagnostic gate check, never an authority", A),
    ]
}

fn modes() -> Vec<ProcessElement> {
    use SourceId::AgentSystem as A;
    vec![
        ProcessElement::documented("kickoff", "open sprint: kickoff/plan/design-review", A),
        ProcessElement::documented("plan", "update the phase plan", A),
        ProcessElement::documented("preflight", "G8 pre-code scans", A),
        ProcessElement::documented("implement", "drive one phase", A),
        ProcessElement::documented("review", "post-code 11-dimension review", A),
        ProcessElement::documented("codex-verify", "Codex cross-verification", A),
        ProcessElement::documented("commit", "9-section body phase commit", A),
        ProcessElement::documented("audit-gate", "Phase 0 sprint audit", A),
        ProcessElement::documented("verification", "sprint verification", A),
        ProcessElement::documented("handoff", "point-in-time inter-provider transfer", A),
    ]
}

fn process_gates() -> Vec<ProcessElement> {
    use SourceId::Process as P;
    // Enforced = blocked by the AUTHORITATIVE provider-neutral gate (agentctl,
    // TOOLING.md's source of truth), mirrored by the Claude lightcheck: G1
    // (design review) and G5 (staged/module coherence). G8 is blocked ONLY by
    // the non-authoritative Claude lightcheck (Check 8) while agentctl merely
    // WARNs a missing preflight — so it is bypassable and stays Documented (cf.
    // module docs). G2-G4/G6-G10 are prompt-carried.
    vec![
        ProcessElement::enforced(
            "G1",
            "Design Review Board",
            P,
            SourceId::HookLightcheck.path(),
            "design_review",
        ),
        ProcessElement::documented("G2", "triggers_revalidate", P),
        ProcessElement::documented("G3", "SMART verification", P),
        ProcessElement::documented("G4", "rigor signal", P),
        ProcessElement::enforced(
            "G5",
            "retired working-tree audit (staged/module/commit checks)",
            P,
            SourceId::HookLightcheck.path(),
            "coherence staging",
        ),
        ProcessElement::documented("G6", "memory carry-over", P),
        ProcessElement::documented("G7", "carry escalation", P),
        // Blocked by the Claude lightcheck (Check 8) but only WARNed by the
        // authoritative agentctl gate → bypassable, so Documented (module docs).
        ProcessElement::documented("G8", "phase preflight presence", P),
        ProcessElement::documented("G9", "factual research gate", P),
        ProcessElement::documented("G10", "OSS prior art", P),
    ]
}

fn publication_gates() -> Vec<ProcessElement> {
    use SourceId::FactoryGates as F;
    // Only FG-CSP-authoring is truly non-delegable: it runs outside --skip-gates
    // on BOTH the publish and redeploy paths. FG8 provenance runs outside
    // --skip-gates on publish but is ABSENT on redeploy and has a documented N0
    // skip → delegable → Documented. FG5/FG6 are relaxed by --skip-gates, FG4 is
    // recorded but non-blocking; all the rest are spec-level → Documented.
    vec![
        ProcessElement::documented("FG0", "Classification", F),
        ProcessElement::documented("FG1", "Scope", F),
        ProcessElement::documented("FG2", "Template", F),
        ProcessElement::documented("FG3", "Manifest", F),
        ProcessElement::documented("FG4", "Diff (recorded, non-blocking)", F),
        ProcessElement::documented("FG5", "Sandbox (relaxable via --skip-gates)", F),
        ProcessElement::documented("FG6", "Secrets/deps (relaxable via --skip-gates)", F),
        ProcessElement::enforced(
            "FG-CSP-authoring",
            "CSP conformance at publish (non-delegable: publish + redeploy)",
            F,
            "crates/sbfb-factory/src/gates.rs",
            "run_gate_csp_authoring",
        ),
        ProcessElement::documented("FG7", "Preview", F),
        ProcessElement::documented("FG8", "Provenance (delegable: skipped on redeploy / N0)", F),
        ProcessElement::documented("FG9", "Publish", F),
        ProcessElement::documented("FG10", "Review", F),
    ]
}

fn finality_hooks() -> Vec<ProcessElement> {
    vec![
        ProcessElement::enforced(
            "phase-precommit-lightcheck",
            "precommit gate for phase commits: design review, staging, Codex artifact",
            SourceId::HookLightcheck,
            SourceId::HookLightcheck.path(),
            "Codex review",
        ),
        ProcessElement::enforced(
            "phase-auditor-gate",
            "blocks a phase commit whose review verdict is not PASS (agentctl tolerates a spaced colon)",
            SourceId::HookAuditorGate,
            SourceId::HookAuditorGate.path(),
            "## Verdict: PASS",
        ),
    ]
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// How many entries in a `(SourceId, bytes)` slice carry a given id — the
/// non-`ExactlyOne` cases are what fail-closed callers must reject.
enum Cardinality {
    Absent,
    Duplicate,
}

/// The UNIQUE byte slice for `id`, or which ambiguity applies. Rejecting the
/// duplicate case (rather than taking the first match) is what keeps [`extract`]
/// and [`detect_stale`] fail-closed against ambiguous input.
fn unique_bytes<'a>(
    sources: &'a [(SourceId, &'a [u8])],
    id: SourceId,
) -> Result<&'a [u8], Cardinality> {
    let mut it = sources.iter().filter(|(s, _)| *s == id);
    match (it.next(), it.next()) {
        (Some((_, b)), None) => Ok(*b),
        (None, _) => Err(Cardinality::Absent),
        (Some(_), Some(_)) => Err(Cardinality::Duplicate),
    }
}

/// Extract a [`FactoryProcessProfile`] from the raw bytes of the six sources.
///
/// PURE over injected bytes: this crate links no filesystem (`lib.rs` charter),
/// so the caller supplies the bytes (the test reads them from disk). Every
/// [`SourceId::ALL`] entry must be present EXACTLY once, or
/// [`ExtractError::MissingSource`] / [`ExtractError::DuplicateSource`] is
/// returned (fail-closed construction: no silent first-match on duplicates). The
/// digests bind the extracted enumeration to the exact source versions it was
/// authored against.
pub fn extract(sources: &[(SourceId, &[u8])]) -> Result<FactoryProcessProfile, ExtractError> {
    let mut digests = Vec::with_capacity(SourceId::ALL.len());
    for id in SourceId::ALL {
        let bytes = unique_bytes(sources, id).map_err(|c| match c {
            Cardinality::Absent => ExtractError::MissingSource(id),
            Cardinality::Duplicate => ExtractError::DuplicateSource(id),
        })?;
        digests.push(SourceDigest {
            id,
            path: id.path().to_string(),
            blake3: digest_hex(bytes),
        });
    }

    Ok(canonical_profile(digests))
}

/// Build the profile with its CANONICAL content (the constant enumeration —
/// roles, modes, gates, hooks, scans, sections, blocks, tracks, authority,
/// invariants) and the supplied source digests. The content does NOT depend on
/// the source bytes, so this is the single source of truth for what a
/// well-formed profile's content must be — used both by [`extract`] and by
/// [`detect_stale`]'s content-integrity check.
fn canonical_profile(sources: Vec<SourceDigest>) -> FactoryProcessProfile {
    FactoryProcessProfile {
        roles: roles(),
        modes: modes(),
        process_gates: process_gates(),
        publication_gates: publication_gates(),
        finality_hooks: finality_hooks(),
        preflight_scans: strings(&["S1a", "S1b", "S2", "S3", "S4"]),
        commit_body_sections: strings(&[
            "Contexte",
            "Fichiers",
            "Delta tests",
            "Verification",
            "Scope cuts",
            "G8 traceability",
            "Pre-launch protocol",
            "Codex verification",
            "Carry closure",
        ]),
        verification_blocks: strings(&["Rust", "Python", "frontend"]),
        audit_tracks: strings(&[
            "A suites",
            "B security",
            "C patterns",
            "D scope",
            "E tests delta",
            "F review files",
            "G carry-overs",
            "H HARDENING",
            "I meta-process",
        ]),
        authority_order: strings(&["process", "RRV", "Factory"]),
        invariants: ProcessInvariants {
            pass_pending_committable: false,
            codex_output_rewritable: false,
            reconciliation_required_before_pass: true,
            gate_check_is_authority: false,
        },
        sources,
    }
}

/// Fail-closed integrity + staleness check. Fully validates a
/// [`FactoryProcessProfile`] that may have been deserialized or tampered:
///
/// 1. CONTENT integrity — the profile's non-source content must equal the
///    canonical enumeration ([`canonical_profile`] over the profile's own
///    sources). A payload with fresh digests but altered content — an `Enforced`
///    gate flipped to `Documented`, `process_gates` cleared, an invariant negated — is
///    [`StaleReason::ContentMismatch`] (`source_id == None`).
/// 2. SOURCE freshness — for EVERY canonical source ([`SourceId::ALL`]), require
///    exactly one embedded digest (0 = [`StaleReason::Missing`], >1 =
///    [`StaleReason::Malformed`]) whose `path` mirrors the id, and exactly one
///    current byte slice, then compare digests ([`StaleReason::DigestMismatch`]).
///
/// Iterating [`SourceId::ALL`] (not `profile.sources`) and rejecting duplicates
/// on both sides is what keeps step 2 fail-closed against a stripped/duplicated
/// binding. This is the runtime-callable mirror of the Sprint 81 K "loaded-stage
/// ↔ signed manifest" binding — a diagnostic returned to the caller, never a
/// process-verdict authority.
pub fn detect_stale(
    profile: &FactoryProcessProfile,
    current: &[(SourceId, &[u8])],
) -> Result<(), ProfileStale> {
    // (1) Content integrity: the canonical content is deterministic and
    // independent of the source bytes, so rebuild it over the profile's OWN
    // sources and compare — any divergence is a tampered/deserialized payload.
    if *profile != canonical_profile(profile.sources.clone()) {
        return Err(ProfileStale {
            source_id: None,
            reason: StaleReason::ContentMismatch,
        });
    }

    // (2) Source binding shape: the sources must be EXACTLY `SourceId::ALL`, in
    // canonical order (`extract`'s documented contract). This catches a
    // stripped, partial, duplicated, extra, OR REORDERED vector up front — the
    // per-id checks below filter by id and would otherwise ignore vector order.
    if !profile.sources.iter().map(|d| d.id).eq(SourceId::ALL) {
        return Err(ProfileStale {
            source_id: None,
            reason: StaleReason::Malformed,
        });
    }

    // (3) Source freshness. Step 2 proved `profile.sources` is exactly
    // `SourceId::ALL` in order (`Iterator::eq` also compares length), so index
    // positionally: `profile.sources[i].id == id` holds by construction.
    for (i, id) in SourceId::ALL.into_iter().enumerate() {
        let digest = &profile.sources[i];
        // The embedded path must mirror the id — a tampered path with an
        // otherwise valid id/digest is a malformed binding, not fresh (the shape
        // check validated ids, not paths).
        if digest.path != id.path() {
            return Err(ProfileStale {
                source_id: Some(id),
                reason: StaleReason::Malformed,
            });
        }
        // Exactly one CURRENT byte slice for this id (a duplicated current input
        // with a drifted second entry must not be masked by the first).
        let bytes = unique_bytes(current, id).map_err(|c| ProfileStale {
            source_id: Some(id),
            reason: match c {
                Cardinality::Absent => StaleReason::Missing,
                Cardinality::Duplicate => StaleReason::Malformed,
            },
        })?;
        if digest_hex(bytes) != digest.blake3 {
            return Err(ProfileStale {
                source_id: Some(id),
                reason: StaleReason::DigestMismatch,
            });
        }
    }
    Ok(())
}

/// The generated JSON Schema (draft 2020-12) for [`FactoryProcessProfile`].
pub fn factory_profile_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(FactoryProcessProfile))
        .expect("FactoryProcessProfile schema serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Workspace root, two levels up from this crate's manifest dir.
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    /// Read a repo-relative source path (segment-joined for Windows/Linux parity).
    fn read_source(rel: &str) -> Vec<u8> {
        let mut p = repo_root();
        for seg in rel.split('/') {
            p = p.join(seg);
        }
        std::fs::read(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    }

    /// The six sources as (id, bytes), read live from disk.
    fn live_sources() -> Vec<(SourceId, Vec<u8>)> {
        SourceId::ALL
            .iter()
            .map(|id| (*id, read_source(id.path())))
            .collect()
    }

    fn as_refs(owned: &[(SourceId, Vec<u8>)]) -> Vec<(SourceId, &[u8])> {
        owned.iter().map(|(id, b)| (*id, b.as_slice())).collect()
    }

    /// The extracted profile equals the committed golden. STRICT (NOT fail-open,
    /// mirrors `accessibility_profile::tests::schema_snapshot`): a missing or
    /// empty golden FAILS asking for regeneration, so deleting the golden cannot
    /// self-heal and defeat the drift gate. Because the golden embeds the source
    /// digests, this test is ALSO the committed-profile staleness gate: edit any
    /// bound source and the freshly extracted digest diverges from the golden,
    /// turning this red (regenerate + re-verify the enumeration, then commit).
    ///
    /// Regenerate an intentional change with
    /// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core extract_matches_golden`.
    #[test]
    fn extract_matches_golden() {
        let owned = live_sources();
        let profile = extract(&as_refs(&owned)).expect("extract from live sources");
        let live_pretty = serde_json::to_string_pretty(&profile).unwrap();

        let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("factory_profile.golden.json");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&golden_path, format!("{live_pretty}\n"))
                .expect("write factory_profile.golden.json");
            return;
        }

        let golden_str = std::fs::read_to_string(&golden_path).expect(
            "factory_profile.golden.json missing — regenerate with \
             UPDATE_SNAPSHOTS=1 and commit it",
        );
        assert!(
            !golden_str.trim().is_empty(),
            "factory_profile.golden.json is empty — regenerate with \
             UPDATE_SNAPSHOTS=1 and commit it",
        );
        let golden: FactoryProcessProfile =
            serde_json::from_str(&golden_str).expect("golden must deserialize into the profile");

        assert_eq!(
            profile, golden,
            "factory profile drift — a bound source changed or the enumeration \
             was edited; re-verify against the source, then UPDATE_SNAPSHOTS=1 to refresh"
        );
    }

    /// PROFILE_STALE fail-closed, proven red-before / block-after. A profile
    /// extracted from the live sources is fresh against those same sources (Ok);
    /// mutating a single source byte makes `detect_stale` return the RIGHT
    /// `source_id` with a digest mismatch; dropping a source yields `Missing`.
    /// The Ok-on-unchanged arm is the red control: it proves the mismatch arm is
    /// not vacuously failing.
    #[test]
    fn profile_stale_on_source_digest_mismatch() {
        let owned = live_sources();
        let profile = extract(&as_refs(&owned)).expect("extract");

        // Control: unchanged sources are NOT stale.
        assert_eq!(
            detect_stale(&profile, &as_refs(&owned)),
            Ok(()),
            "fresh sources must not be reported stale"
        );

        // Mutate exactly one source (FACTORY_GATES.md) by appending a byte.
        let mut mutated = owned.clone();
        let idx = mutated
            .iter()
            .position(|(id, _)| *id == SourceId::FactoryGates)
            .expect("FactoryGates present");
        mutated[idx].1.push(b'X');
        let err = detect_stale(&profile, &as_refs(&mutated)).expect_err("mutation must be stale");
        assert_eq!(err.source_id, Some(SourceId::FactoryGates));
        assert_eq!(err.reason, StaleReason::DigestMismatch);

        // A whitespace-only mutation is ALSO caught (the appended char is not \r/BOM).
        let mut appended_ws = owned.clone();
        appended_ws[idx].1.push(b' ');
        let ws_err =
            detect_stale(&profile, &as_refs(&appended_ws)).expect_err("trailing space is stale");
        assert_eq!(ws_err.source_id, Some(SourceId::FactoryGates));

        // Dropping a CURRENT source is reported as Missing, not silently Ok.
        let dropped: Vec<(SourceId, Vec<u8>)> = owned
            .iter()
            .filter(|(id, _)| *id != SourceId::HookAuditorGate)
            .cloned()
            .collect();
        let miss = detect_stale(&profile, &as_refs(&dropped)).expect_err("missing source is stale");
        assert_eq!(miss.source_id, Some(SourceId::HookAuditorGate));
        assert_eq!(miss.reason, StaleReason::Missing);
    }

    /// Fail-closed against a tampered/deserialized profile: an empty, partial, or
    /// duplicated embedded `sources` set must NOT pass as fresh (Codex S83-D). The
    /// check iterates [`SourceId::ALL`], so a stripped digest set fails at the
    /// first id, and a duplicate entry for one id is ambiguous and also stale.
    #[test]
    fn detect_stale_fails_closed_on_malformed_sources() {
        let owned = live_sources();
        let mut profile = extract(&as_refs(&owned)).expect("extract");

        // Empty / partial / duplicated / REORDERED embedded sources all fail the
        // "exactly SourceId::ALL, in order" shape check (source_id None).
        let mut emptied = profile.clone();
        emptied.sources.clear();
        let err = detect_stale(&emptied, &as_refs(&owned)).expect_err("empty sources is stale");
        assert_eq!(err.source_id, None);
        assert_eq!(err.reason, StaleReason::Malformed);

        let mut partial = profile.clone();
        partial.sources.retain(|d| d.id != SourceId::Process);
        let perr = detect_stale(&partial, &as_refs(&owned)).expect_err("partial sources is stale");
        assert_eq!(perr.source_id, None);
        assert_eq!(perr.reason, StaleReason::Malformed);

        let dup = profile.sources[0].clone();
        profile.sources.push(dup.clone());
        let derr = detect_stale(&profile, &as_refs(&owned)).expect_err("duplicate source is stale");
        assert_eq!(derr.source_id, None);
        assert_eq!(derr.reason, StaleReason::Malformed);

        // A REORDERED sources vector (all digests/paths valid, wrong order)
        // violates the documented order contract and must not pass as fresh.
        let mut reordered = extract(&as_refs(&owned)).expect("extract");
        reordered.sources.reverse();
        let rerr =
            detect_stale(&reordered, &as_refs(&owned)).expect_err("reordered sources is stale");
        assert_eq!(rerr.source_id, None);
        assert_eq!(rerr.reason, StaleReason::Malformed);

        // An embedded `path` that does not mirror its id is a Malformed binding,
        // even with a valid id and digest.
        let mut tampered = extract(&as_refs(&owned)).expect("extract");
        tampered.sources[0].path = "docs/agent/EVIL.md".to_string();
        let terr = detect_stale(&tampered, &as_refs(&owned)).expect_err("path tamper is stale");
        assert_eq!(terr.source_id, Some(tampered.sources[0].id));
        assert_eq!(terr.reason, StaleReason::Malformed);

        // A DUPLICATED CURRENT input (drifted second entry) must NOT be masked by a
        // fresh first entry — the current side is checked for uniqueness too.
        let fresh = extract(&as_refs(&owned)).expect("extract");
        let mut dup_current = owned.clone();
        let mut drifted = owned[0].clone();
        drifted.1.push(b'Z');
        dup_current.push(drifted); // second entry for owned[0].0, divergent
        let cerr =
            detect_stale(&fresh, &as_refs(&dup_current)).expect_err("duplicate current is stale");
        assert_eq!(cerr.source_id, Some(owned[0].0));
        assert_eq!(cerr.reason, StaleReason::Malformed);
    }

    /// Fail-closed against a CONTENT-tampered profile: a payload with fresh
    /// source digests but altered content (Codex S83-D) — an `Enforced` gate
    /// (`G1`) flipped to `Documented`, or `process_gates` cleared — is
    /// `ContentMismatch`, never fresh. The strict golden protects the committed
    /// snapshot; this protects the public runtime API against a forged profile.
    #[test]
    fn detect_stale_fails_closed_on_content_tamper() {
        let owned = live_sources();

        // Flip an Enforced gate (G1) to Documented, keeping the six source
        // digests untouched — a dishonest marking must not pass as fresh.
        let mut flipped = extract(&as_refs(&owned)).expect("extract");
        let g1 = flipped
            .process_gates
            .iter_mut()
            .find(|g| g.id == "G1")
            .expect("G1 present");
        g1.enforcement = EnforcementKind::Documented;
        g1.mechanism = None;
        let ferr = detect_stale(&flipped, &as_refs(&owned)).expect_err("content tamper is stale");
        assert_eq!(ferr.source_id, None);
        assert_eq!(ferr.reason, StaleReason::ContentMismatch);

        // Clearing a whole content bucket is also caught, digests intact.
        let mut cleared = extract(&as_refs(&owned)).expect("extract");
        cleared.process_gates.clear();
        let cerr = detect_stale(&cleared, &as_refs(&owned)).expect_err("cleared gates is stale");
        assert_eq!(cerr.source_id, None);
        assert_eq!(cerr.reason, StaleReason::ContentMismatch);

        // Negating an invariant is caught too.
        let mut lied = extract(&as_refs(&owned)).expect("extract");
        lied.invariants.gate_check_is_authority = true;
        let lerr = detect_stale(&lied, &as_refs(&owned)).expect_err("invariant lie is stale");
        assert_eq!(lerr.reason, StaleReason::ContentMismatch);
    }

    /// `extract` is fail-closed against a duplicated input source (no silent
    /// first-match).
    #[test]
    fn extract_rejects_duplicate_source() {
        let owned = live_sources();
        let mut dup = owned.clone();
        dup.push(owned[0].clone());
        assert_eq!(
            extract(&as_refs(&dup)),
            Err(ExtractError::DuplicateSource(owned[0].0))
        );
    }

    /// CRLF/BOM determinism: the same logical content hashes identically whether
    /// it arrives as LF, CRLF, or BOM+CRLF. Guards against a false cross-OS
    /// PROFILE_STALE independent of `.gitattributes`. A lone `\r` (NOT part of a
    /// CRLF pair) is preserved, so a genuinely different byte sequence still
    /// yields a different digest (fail-closed, Codex S83-D).
    #[test]
    fn canonicalisation_is_crlf_bom_stable() {
        let lf = b"line one\nline two\n".to_vec();
        let crlf = b"line one\r\nline two\r\n".to_vec();
        let bom_crlf = {
            let mut v = vec![0xEF, 0xBB, 0xBF];
            v.extend_from_slice(&crlf);
            v
        };
        assert_eq!(digest_hex(&lf), digest_hex(&crlf));
        assert_eq!(digest_hex(&lf), digest_hex(&bom_crlf));
        // A real content change still differs.
        assert_ne!(digest_hex(&lf), digest_hex(b"line one\nline three\n"));
        // A lone CR is a real difference and must NOT collide with its stripped
        // form (only paired \r\n folds to \n).
        assert_ne!(digest_hex(b"exit 0"), digest_hex(b"exit 0\r"));
        assert_ne!(digest_hex(b"a\rb"), digest_hex(b"ab"));
    }

    /// Resolve a mechanism anchor against the real repo file.
    fn anchor_resolves(anchor: &MechanismAnchor) -> bool {
        let mut p = repo_root();
        for seg in anchor.file.split('/') {
            p = p.join(seg);
        }
        match std::fs::read_to_string(&p) {
            Ok(content) => content.contains(&anchor.needle),
            Err(_) => false,
        }
    }

    /// Every element carries an honest marking, and every `Enforced` marking is
    /// backed by a mechanism that RESOLVES against the real file. Non-vacuous:
    /// the structural `Enforced ⟺ mechanism.is_some()` law is asserted, each real
    /// anchor is resolved, and a synthetic bad anchor is proven to FAIL
    /// resolution (so the resolver is not trivially true).
    #[test]
    fn documented_vs_enforced_marked() {
        let owned = live_sources();
        let profile = extract(&as_refs(&owned)).expect("extract");

        let buckets = [
            &profile.roles,
            &profile.modes,
            &profile.process_gates,
            &profile.publication_gates,
            &profile.finality_hooks,
        ];

        let mut enforced_seen = 0usize;
        for bucket in buckets {
            for el in bucket {
                // Structural honesty: Enforced iff a mechanism is attached.
                assert_eq!(
                    el.enforcement == EnforcementKind::Enforced,
                    el.mechanism.is_some(),
                    "element {} must have a mechanism iff Enforced",
                    el.id
                );
                if let Some(anchor) = &el.mechanism {
                    assert!(
                        anchor_resolves(anchor),
                        "Enforced element {} claims mechanism {}:{} but it does not resolve",
                        el.id,
                        anchor.file,
                        anchor.needle
                    );
                    enforced_seen += 1;
                }
            }
        }

        // The enforced set is exactly G1, G5, FG-CSP-authoring, and the 2
        // finality hooks — the five authoritatively-blocked mechanical checks.
        // G8 (Claude-layer only) and FG8 (delegable) are Documented.
        assert_eq!(
            enforced_seen, 5,
            "expected exactly five Enforced mechanisms"
        );

        // Negative control: a bogus anchor must NOT resolve (resolver is real).
        let bogus = MechanismAnchor {
            file: "crates/sbfb-factory/src/gates.rs".to_string(),
            needle: "this_symbol_does_not_exist_9f3c1".to_string(),
        };
        assert!(
            !anchor_resolves(&bogus),
            "a nonexistent needle must fail resolution"
        );
        // And a nonexistent file must not resolve either.
        let no_file = MechanismAnchor {
            file: "crates/does/not/exist.rs".to_string(),
            needle: "anything".to_string(),
        };
        assert!(
            !anchor_resolves(&no_file),
            "a missing file must fail resolution"
        );
    }

    /// The extracted enumeration has the required cardinalities (D.4 acceptance):
    /// 9 roles, 10 modes, G1-G10, FG0-FG10 (+FG-CSP-authoring), 2 hooks, and the
    /// descriptive process facts.
    #[test]
    fn enumeration_cardinalities() {
        let owned = live_sources();
        let profile = extract(&as_refs(&owned)).expect("extract");
        assert_eq!(profile.roles.len(), 9, "9 roles");
        assert_eq!(profile.modes.len(), 10, "10 modes");
        assert_eq!(profile.process_gates.len(), 10, "G1-G10");
        // FG0-FG10 is 11 gates, plus the non-delegable FG-CSP-authoring.
        assert_eq!(
            profile.publication_gates.len(),
            12,
            "FG0-FG10 + FG-CSP-authoring"
        );
        assert!(
            profile
                .publication_gates
                .iter()
                .any(|g| g.id == "FG-CSP-authoring"),
            "FG-CSP-authoring must be present so at least one publication gate is Enforced"
        );
        assert_eq!(profile.finality_hooks.len(), 2, "2 finality hooks");
        assert_eq!(profile.preflight_scans.len(), 5, "S1a,S1b,S2,S3,S4");
        assert_eq!(profile.commit_body_sections.len(), 9, "9-section body");
        assert_eq!(profile.verification_blocks.len(), 3, "Rust/Python/frontend");
        assert_eq!(
            profile.audit_tracks.len(),
            9,
            "9 audit tracks (AGENT_SYSTEM §5.4)"
        );
        assert_eq!(profile.sources.len(), 6, "6 bound sources");
        // Every G1-G10 id present, in order.
        let g_ids: Vec<&str> = profile
            .process_gates
            .iter()
            .map(|g| g.id.as_str())
            .collect();
        assert_eq!(
            g_ids,
            ["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9", "G10"]
        );
    }

    /// A missing source aborts extraction rather than producing a partial profile.
    #[test]
    fn extract_requires_all_sources() {
        let owned = live_sources();
        let partial: Vec<(SourceId, Vec<u8>)> = owned
            .iter()
            .filter(|(id, _)| *id != SourceId::Process)
            .cloned()
            .collect();
        assert_eq!(
            extract(&as_refs(&partial)),
            Err(ExtractError::MissingSource(SourceId::Process))
        );
    }

    /// Bound-source invariants are the extracted values, not knobs.
    #[test]
    fn invariants_reflect_the_process() {
        let owned = live_sources();
        let profile = extract(&as_refs(&owned)).expect("extract");
        assert!(!profile.invariants.pass_pending_committable);
        assert!(!profile.invariants.codex_output_rewritable);
        assert!(profile.invariants.reconciliation_required_before_pass);
        assert!(!profile.invariants.gate_check_is_authority);
        assert_eq!(profile.authority_order, ["process", "RRV", "Factory"]);
    }

    /// The schema carries the contract identity as its title.
    #[test]
    fn schema_title_is_the_identity() {
        let schema = factory_profile_schema();
        assert_eq!(
            schema.get("title").and_then(|v| v.as_str()),
            Some("FactoryProcessProfile.v1")
        );
    }

    /// The error `Display` impls name the offending source and reason across
    /// EVERY [`StaleReason`] and [`ExtractError`] arm, so a diagnostic surfaced to
    /// a caller is actionable.
    #[test]
    fn error_display_names_source_and_reason() {
        // ExtractError, both arms.
        let missing = ExtractError::MissingSource(SourceId::Process).to_string();
        assert!(missing.contains("missing"), "{missing}");
        assert!(missing.contains("docs/agent/PROCESS.md"), "{missing}");
        let dup = ExtractError::DuplicateSource(SourceId::AgentSystem).to_string();
        assert!(dup.contains("duplicate"), "{dup}");
        assert!(dup.contains("docs/agent/AGENT_SYSTEM.md"), "{dup}");

        // ProfileStale, every StaleReason arm. Source-attributed arms name the path.
        let cases = [
            (
                Some(SourceId::FactoryGates),
                StaleReason::DigestMismatch,
                "digest mismatch",
                Some("docs/factory/FACTORY_GATES.md"),
            ),
            (
                Some(SourceId::HookAuditorGate),
                StaleReason::Missing,
                "source missing",
                Some(".claude/hooks/phase-auditor-gate.sh"),
            ),
            (
                Some(SourceId::HookLightcheck),
                StaleReason::Malformed,
                "malformed source binding",
                Some(".claude/hooks/phase-precommit-lightcheck.sh"),
            ),
            // Content mismatch is not attributable to one source (source_id None).
            (None, StaleReason::ContentMismatch, "content diverges", None),
        ];
        for (source_id, reason, needle, path) in cases {
            let s = ProfileStale { source_id, reason }.to_string();
            assert!(s.starts_with("PROFILE_STALE:"), "{s}");
            assert!(s.contains(needle), "{s}");
            if let Some(p) = path {
                assert!(s.contains(p), "{s}");
            }
        }
    }
}
