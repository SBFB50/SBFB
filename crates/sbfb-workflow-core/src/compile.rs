// SPDX-License-Identifier: AGPL-3.0-or-later
//! `CompiledPlan.v1` — the authoritative Rust compiler (Sprint 83 Phase H).
//!
//! ONE semantic authority (research §12.1): the sandboxed Hub iframe may lint
//! for immediate feedback, but the authoritative result of "is this plan
//! valid, and what exactly will run" comes from THIS module, on the host.
//! `compile` turns a RESOLVED pack (the typed pack root plus the raw
//! graph/agent documents its lock pins) into an immutable [`CompiledPlan`]:
//! every reference bound, every occurrence named, every capability closed,
//! every budget compared dimension by dimension, every refusal enumerated.
//!
//! ## The twenty static refusal classes
//!
//! Thirteen from research §10.5, six consigned for Phase H by the
//! node-profile freeze §8 (`sprint83_node_profile_freeze.md`), plus the
//! nominal-by-digest port compatibility the freeze §7 assigns to this phase.
//! Each class has its own [`CompileError`] variant and its own discriminating
//! test. The DAG property is proven BY COMPILATION, never "by construction":
//! the parse (Phase G) accepts an `edges` list spelling `A→B, B→A`; this
//! compiler refuses it.
//!
//! ## What Phase H does NOT close (honest bounds)
//!
//! The compiler closes the STATIC half of Plan-Then-Execute: the SET of
//! capabilities, the topology, the pinned artefacts, the bounds. It does not
//! bind capabilities per effect (`effect_id` binding = runtime, Phases J/K),
//! does not intersect the seven authority layers (that computation is
//! delivered in `sbfb-workflow-engine::capability`, Phase L; its per-effect
//! binding into dispatch is the run-surface phase), does not bind
//! secrets (`secret_ref` keystore binding = Phase L1), does not touch
//! SSRF/DNS egress (Phase P), and ships no TS mirror or canvas authoring
//! surface (Phase T — the Rust schema snapshot is this phase's sole
//! anti-drift). The secret/endpoint scan below REFUSES what it
//! recognises in the graph/agent documents themselves; prompt and corpus
//! FILE bytes are pinned by digest and scanned at pack-assembly (graph module
//! doc), and a regex-shaped scan is best-effort by nature — the enforced
//! guarantees are the format's required fields and the closed capability set.
//!
//! ## Determinism
//!
//! `compile` is a pure function of its RESOLVED input: same input, same
//! plan, byte-identical under both build graphs (workspace `preserve_order`
//! and `-p` alike) because every collection is a `BTreeMap`, an explicitly
//! sorted `Vec` (edges, human gates, capability instances) or a `Vec` in
//! pinned-document order (entrypoints), every digest goes through the
//! Phase F canonicalisation door ([`crate::pack::digest_for`], JCS + domain
//! prefix), and no clock, counter, UUID or randomness exists anywhere in
//! the path. Every provided document is bound by HASH: the compiler derives
//! `blake3(bytes)` itself and refuses bytes the signed lock does not pin
//! (Codex r2 — a caller-supplied pin proved nothing). `node_instance_id` is a
//! pure function of the structural path (research §12.2), encoded RFC 6901
//! (`/` joins segments, `~1`/`~0` escape them), so two occurrences can never
//! collide and the same pack always yields the same identities.
//!
//! ## Signature posture (Phase H)
//!
//! A `CompiledPlan` is a LOCAL, DERIVED artefact: it is not signed and not
//! gossiped, so it declares no `DOMAIN_*_V1` and no format version —
//! `DOMAIN_COMPILED_PLAN_V1` stays a RESERVED tag with zero producers
//! (`canonical.rs`), and the frozen census of 29 is untouched. The digest
//! that carries AUTHORITY over a pack remains the `content` digest recomputed
//! from `PACK.lock` under the pack's Ed25519 signature (verified BEFORE this
//! compiler ever runs); [`compiled_plan_digest`] and the embedded
//! `graph_semantic_digest` are derived, LINKED digests — never an autonomous
//! integrity surface. The signed facet `pack.digests.compiled_plan` remains a
//! SHAPE-validated forward claim in this phase (see `pack.rs`): this module
//! provides the digest function that makes the claim checkable, and the
//! materialising verification that reconciles claim against artefact is a
//! later phase's pass.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent_definition::{BudgetPolicy, ProviderClass, SbfbAgentDefinition, SchemaRef};
use crate::graph::{
    CancellationPolicy, ErrorPolicy, NodeFailureAction, SbfbWorkflowGraph, TimeoutPolicy,
    WorkflowEdge, WorkflowNode,
};
use crate::pack::{
    PackDigestKind, PackError, SbfbPromptAgentPack, content_digest_of_lock, digest_for,
    graph_semantic_view, is_lower_hex_digest, looks_like_materialised_secret,
};

/// Version of the compiler that produced a plan — the crate version, baked at
/// build time. Part of `CompiledPlan.v1` family 9 (research §12.1).
///
/// Coupling stated plainly: this value enters the plan (and therefore
/// [`compiled_plan_digest`]), so `compile` is pure of `(pack, compiler
/// version)` — a workspace version bump legitimately moves every plan digest
/// and requires refreshing the `*.compiled.golden.json` fixtures
/// (`UPDATE_SNAPSHOTS=1`, workspace build). The AUTHORITY digest of a pack
/// (`content`, Ed25519-signed) never includes it.
pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The compilation profile. v1 has exactly one: everything fail-closed.
pub const COMPILER_PROFILE_STRICT_V1: &str = "strict-v1";

/// Hard ceiling on subworkflow nesting depth. Content-addressed pinning makes
/// a true reference cycle cryptographically inexpressible (a child's digest
/// cannot contain itself), so this bound is defense-in-depth against
/// pathological nesting, in the spirit of the pack's `MAX_*` bounds — not a
/// §10.5 refusal class.
pub const MAX_PLAN_DEPTH: u32 = 16;

/// The reserved routing port every `switch`/`classifier` must wire.
pub const DEFAULT_PORT: &str = "default";
/// The ordinary data output port.
pub const OUT_PORT: &str = "out";
/// The single data input port of v1 (destinations declare nothing else yet).
pub const IN_PORT: &str = "in";
/// The control-transfer port of a `handoff` node.
pub const HANDOFF_PORT: &str = "to";
/// The failure route port, wired only under `route-error-port` policy.
pub const ERROR_PORT: &str = "error";

/// Ceiling on ONE routing/decision label, in bytes (Sprint 83 Phase K,
/// closing the J carry "borne label vs String compilateur"): the engine's
/// deposit door bounds a stored `decision_label` at
/// `MAX_SIGNAL_PAYLOAD_BYTES` (256 KiB), so a longer label would compile
/// into a plan whose `human.review` decision can never be DEPOSITED — a
/// valid-but-unresumable plan. The compiler refuses it instead. The engine
/// asserts the two constants stay equal (it depends on this crate; the
/// reverse import would invert the dependency).
pub const MAX_DECISION_LABEL_BYTES: usize = 256 * 1024;

/// Field names the freeze §8 rule 6 treats as CLASSIFICATION discriminants:
/// a `switch` may not discriminate on one of these when the field's
/// provenance traces to a model output and an effect depends on the route.
/// Scope is deliberately narrow — classification fields ONLY; a
/// model-produced QUALITY gate is a different class and stays legal.
///
/// NOMINAL v1 bound (honesty): recognition is exact-lowercase membership in
/// this fixed catalogue. A semantically-classifying field named OUTSIDE it
/// (`sensitivity`, `clearance`, …) escapes this refusal — the same
/// best-effort residual class as the secret/endpoint scan (THREAT_MODEL §18
/// T-PLAN-CLASSIFICATION-LAUNDER caveat); the frozen rule 6 defines the
/// catalogued scope. The typed capability classes now exist engine-side
/// (`sbfb-workflow-engine::capability`, §14.2), but this compiler
/// recognition stays NOMINAL — once that computation is bound into dispatch
/// (the run-surface phase) the runtime intersection WILL judge the effect
/// class by TYPE in defense-in-depth; it does not tighten this static
/// membership. Publication stays backstopped by the HITL domination rule.
pub const CLASSIFICATION_DISCRIMINANTS: [&str; 6] = [
    "visibility",
    "classification",
    "confidentiality",
    "confidentiality_level",
    "privacy",
    "privacy_level",
];

/// Label terms that make a `classifier`'s proposed route a CLASSIFICATION
/// decision (freeze §8 rule 6): a model proposing one of these labels with an
/// effect downstream is refused — publication/visibility comes from a
/// deterministic source or a `human.review`, never an LLM (§19.3).
/// Same NOMINAL v1 bound as [`CLASSIFICATION_DISCRIMINANTS`]: an unlisted
/// synonym escapes.
pub const CLASSIFICATION_LABEL_TERMS: [&str; 6] = [
    "public",
    "private",
    "internal",
    "secret",
    "confidential",
    "restricted",
];

/// Dot-segments of a `capability_ref` that mark it as PUBLICATION/EGRESS
/// class for the §10.5 refusal "publication without provenance, HITL or
/// required evaluation". Provenance and the evaluation digest are REQUIRED
/// fields of the signed pack root (Phase F — a parse cannot lack them), so
/// the live half of that refusal here is the HITL one: a marked capability
/// call must be dominated by a `human.review` gate.
///
/// NOMINAL v1 bound (honesty): a publishing capability named without any of
/// these segments (`content.share`, `feed.upload`, …) is NOT captured by
/// this STATIC marker set. The typed capability classes now exist engine-side
/// (§14.2): once bound into dispatch (run-surface phase) the runtime WILL
/// judge the publication class by TYPE, not by name (defense-in-depth), but
/// this compiler marker set stays a best-effort under-inclusive STATIC
/// recogniser, stated as such in THREAT_MODEL §18/§19.3.
pub const PUBLICATION_CAPABILITY_MARKERS: [&str; 4] =
    ["publish", "egress", "announce", "broadcast"];

/// Model-id prefix the `claude-code` provider class is bound to (nominal v1;
/// richer catalogs belong to the adapter phases N/O).
pub const CLAUDE_CODE_MODEL_PREFIX: &str = "claude";
/// Model-id prefix the `codex` provider class is bound to (nominal v1).
pub const CODEX_MODEL_PREFIX: &str = "gpt";

/// Everything the compiler refuses. One variant per refusal class — the
/// twenty classes of the preflight §9 table plus the parse/binding/bound
/// refusals the classes rest on. Hand-rolled like [`PackError`]: no
/// `thiserror`, every variant terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// A provided document failed the strict typed parse (covers the
    /// parse-carried classes: unbounded `map`, missing budget, unknown
    /// kind/field, `null`, zero bounds).
    InvalidDocument { path: String, detail: String },
    /// §10.5 "référence non pinnée": a digest is malformed, absent from
    /// `PACK.lock`, unresolved, or does not match the provided bytes' pin.
    UnpinnedReference { path: String, detail: String },
    /// §10.5 "node ou sortie inaccessible".
    UnreachableNode {
        workflow_id: String,
        node_id: String,
    },
    /// A workflow with no entrypoint, or an entrypoint naming no node.
    NoEntrypoint { workflow_id: String, detail: String },
    /// Freeze §8 rule 3: ANY cycle over arbitrary edges (subsumes §10.5
    /// "cycle non porté par un node de boucle borné" and self-loops).
    CycleDetected { workflow_id: String, detail: String },
    /// Freeze §8 rule 1: an edge endpoint naming no node.
    DanglingEdge { workflow_id: String, detail: String },
    /// S1a-07: a fully identical `(from, from_port, to, to_port)` edge twice
    /// — silent fan-in doubling that can fake a join quorum.
    DuplicateEdge { workflow_id: String, detail: String },
    /// An edge uses a port its endpoint does not expose (a surnumerary
    /// routing edge, a `to_port` other than `in`, an out-edge on `output`).
    UnknownPort { workflow_id: String, detail: String },
    /// Freeze §8 rule 2: a declared port with no edge (a routing label or
    /// human decision that dead-ends, a missing `default`, an unwired
    /// `error` port under `route-error-port` policy).
    UnwiredPort { workflow_id: String, detail: String },
    /// §10.5 "join dont les branches et la politique sont incohérentes" +
    /// freeze §8 rule 2 (sources must exist and actually feed the join).
    UnsatisfiableJoin {
        workflow_id: String,
        node_id: String,
        detail: String,
    },
    /// Freeze §8 rule 5: `default` among `branch_labels`, empty or duplicate
    /// label sets.
    InvalidRoutingLabels {
        workflow_id: String,
        node_id: String,
        detail: String,
    },
    /// Freeze §8 rule 4: a data-driven key (node id, `field_map` key) using
    /// a reserved presentation name — the `graph_semantic` malleability
    /// closure.
    PresentationKeyAsData { workflow_id: String, detail: String },
    /// An invoked/handed-off agent id absent from the graph's `agent_refs`.
    UndeclaredAgent { workflow_id: String, detail: String },
    /// A `capability.call` referencing a capability its own workflow does
    /// not request.
    CapabilityNotRequested {
        workflow_id: String,
        node_id: String,
        capability: String,
    },
    /// §10.5 "capacité demandée absente du manifeste du pack": a workflow
    /// requests a capability the pack root does not.
    CapabilityNotInManifest {
        workflow_id: String,
        capability: String,
    },
    /// §10.5 "provider incompatible avec le modèle ou la sortie structurée"
    /// (nominal v1: CLI provider classes are vendor-bound by model prefix).
    ProviderIncompatible { path: String, detail: String },
    /// §10.5 "handoff non autorisé": an upstream giver agent whose
    /// definition does not allowlist the handoff target.
    UnauthorizedHandoff {
        workflow_id: String,
        node_id: String,
        detail: String,
    },
    /// §10.5 "secret literal" in a graph/agent document string.
    SecretLiteral { path: String },
    /// §10.5 "endpoint ou code exécutable interdit" in a graph/agent
    /// document string.
    ForbiddenEndpoint { path: String, detail: String },
    /// §10.5 "budget enfant supérieur au budget parent", one dimension at a
    /// time — never an aggregate comparison.
    BudgetOverflow {
        path: String,
        dimension: String,
        child: u64,
        parent: u64,
    },
    /// §10.5 "publication sans provenance / HITL / évaluation requise" —
    /// the live HITL half (provenance and evaluation are pack-required by
    /// parse): a publication-class capability call not dominated by a
    /// non-model gate (`human.review`, or the host-delivered external
    /// `wait.signal` — the babel community-review pattern ratified by the
    /// freeze §8.6 scoping).
    PublicationWithoutHumanGate {
        workflow_id: String,
        node_id: String,
    },
    /// Freeze §8 rule 6: a classification discriminant whose provenance
    /// traces to a model output, with an effect depending on the route.
    ClassificationFromModel {
        workflow_id: String,
        node_id: String,
        detail: String,
    },
    /// Freeze §7: nominal-by-digest port compatibility — the same schema id
    /// pinned under two different digests across one edge.
    PortTypeMismatch { workflow_id: String, detail: String },
    /// §10.5 "output non typé" landed as artefact binding: an `output` node
    /// and the `artifact_outputs` declaration must agree both ways.
    OutputNotDeclared { workflow_id: String, detail: String },
    /// Nesting beyond [`MAX_PLAN_DEPTH`] (bound, not a §10.5 class).
    PlanTooDeep { detail: String },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDocument { path, detail } => {
                write!(f, "invalid document at {path}: {detail}")
            }
            Self::UnpinnedReference { path, detail } => {
                write!(f, "unpinned reference at {path}: {detail}")
            }
            Self::UnreachableNode {
                workflow_id,
                node_id,
            } => write!(f, "unreachable node {node_id} in workflow {workflow_id}"),
            Self::NoEntrypoint {
                workflow_id,
                detail,
            } => write!(f, "no entrypoint in workflow {workflow_id}: {detail}"),
            Self::CycleDetected {
                workflow_id,
                detail,
            } => write!(f, "cycle in workflow {workflow_id}: {detail}"),
            Self::DanglingEdge {
                workflow_id,
                detail,
            } => write!(f, "dangling edge in workflow {workflow_id}: {detail}"),
            Self::DuplicateEdge {
                workflow_id,
                detail,
            } => write!(f, "duplicate edge in workflow {workflow_id}: {detail}"),
            Self::UnknownPort {
                workflow_id,
                detail,
            } => write!(f, "unknown port in workflow {workflow_id}: {detail}"),
            Self::UnwiredPort {
                workflow_id,
                detail,
            } => write!(f, "unwired port in workflow {workflow_id}: {detail}"),
            Self::UnsatisfiableJoin {
                workflow_id,
                node_id,
                detail,
            } => write!(
                f,
                "unsatisfiable join {node_id} in workflow {workflow_id}: {detail}"
            ),
            Self::InvalidRoutingLabels {
                workflow_id,
                node_id,
                detail,
            } => write!(
                f,
                "invalid routing labels on {node_id} in workflow {workflow_id}: {detail}"
            ),
            Self::PresentationKeyAsData {
                workflow_id,
                detail,
            } => write!(
                f,
                "reserved presentation name used as data in workflow {workflow_id}: {detail}"
            ),
            Self::UndeclaredAgent {
                workflow_id,
                detail,
            } => write!(f, "undeclared agent in workflow {workflow_id}: {detail}"),
            Self::CapabilityNotRequested {
                workflow_id,
                node_id,
                capability,
            } => write!(
                f,
                "capability {capability} called by {node_id} is not requested by workflow {workflow_id}"
            ),
            Self::CapabilityNotInManifest {
                workflow_id,
                capability,
            } => write!(
                f,
                "capability {capability} requested by workflow {workflow_id} is absent from the pack manifest"
            ),
            Self::ProviderIncompatible { path, detail } => {
                write!(f, "provider incompatible at {path}: {detail}")
            }
            Self::UnauthorizedHandoff {
                workflow_id,
                node_id,
                detail,
            } => write!(
                f,
                "unauthorized handoff at {node_id} in workflow {workflow_id}: {detail}"
            ),
            Self::SecretLiteral { path } => write!(f, "secret-shaped literal at {path}"),
            Self::ForbiddenEndpoint { path, detail } => {
                write!(
                    f,
                    "forbidden endpoint or executable marker at {path}: {detail}"
                )
            }
            Self::BudgetOverflow {
                path,
                dimension,
                child,
                parent,
            } => write!(
                f,
                "budget overflow at {path}: {dimension} child {child} > parent {parent}"
            ),
            Self::PublicationWithoutHumanGate {
                workflow_id,
                node_id,
            } => write!(
                f,
                "publication capability call {node_id} in workflow {workflow_id} is not dominated by a human.review gate"
            ),
            Self::ClassificationFromModel {
                workflow_id,
                node_id,
                detail,
            } => write!(
                f,
                "model-produced classification discriminant at {node_id} in workflow {workflow_id}: {detail}"
            ),
            Self::PortTypeMismatch {
                workflow_id,
                detail,
            } => write!(f, "port type mismatch in workflow {workflow_id}: {detail}"),
            Self::OutputNotDeclared {
                workflow_id,
                detail,
            } => write!(
                f,
                "output/artefact binding broken in workflow {workflow_id}: {detail}"
            ),
            Self::PlanTooDeep { detail } => write!(f, "plan nesting too deep: {detail}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// One raw document handed to the compiler as its pinned BYTES (Codex r2:
/// the compiler derives the pin ITSELF — `blake3(bytes)` — instead of
/// trusting a caller-supplied pin, so a materialiser handing bytes that do
/// not match the lock or a reference is refused, never silently compiled).
/// The bytes are the exact bytes the pin covers (for git-committed test
/// fixtures, the checkout-newline-canonicalised bytes, exactly like the
/// `PACK.lock` pins of Phase G).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidedDocument {
    /// The pinned raw bytes of the document (parsed strictly, and hashed,
    /// by the compiler itself).
    pub bytes: Vec<u8>,
}

impl ProvidedDocument {
    /// The derived `PACK.lock` pin of these bytes.
    pub fn pin(&self) -> String {
        crate::pack::blake3_hex(&self.bytes)
    }
}

/// The resolved input of one compilation: the verified, typed pack root and
/// the raw documents its lock pins. Ed25519 verification happens BEFORE this
/// point (`verify_signed_pack`) — the compiler trusts its caller to hand it
/// the payload of a verified pack, and re-derives the authoritative
/// `content` digest itself from the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileInput {
    /// The verified pack root (manifest, lock, provenance, requested
    /// capabilities). The caller MUST have verified the signed envelope
    /// (`verify_signed_pack`, Ed25519 before parse) — the plan binds
    /// `pack_digest`/name/revision, never the signer key itself.
    pub pack: SbfbPromptAgentPack,
    /// The root workflow document. WHICH lock-pinned workflow is the root
    /// is the LAUNCHING HOST's authority (a pack may legitimately carry
    /// several launchable workflows; the format pins no designated root):
    /// the compiled plan makes the choice auditable — `root_workflow_id`
    /// plus its exact digest under `artefact_digests.workflows` — rather
    /// than restricting it (Codex r3, statuted).
    pub root_workflow: ProvidedDocument,
    /// Child workflow documents, keyed by `workflow_id`.
    pub subworkflows: BTreeMap<String, ProvidedDocument>,
    /// Agent definition documents, keyed by `agent_id`.
    pub agents: BTreeMap<String, ProvidedDocument>,
}

/// Family 9 of `CompiledPlan.v1`: which compiler, under which profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompilerProfile.v1")]
pub struct CompilerProfile {
    /// The producing crate version ([`COMPILER_VERSION`]).
    pub version: String,
    /// The compilation profile ([`COMPILER_PROFILE_STRICT_V1`]).
    pub profile: String,
}

/// The §13.2 placement vocabulary, one canonical kebab spelling per class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementClass {
    /// Execute on the initiating node only.
    LocalOnly,
    /// Prefer local, may fall back to an approved provider.
    LocalPreferred,
    /// An approved provider may serve the call.
    ApprovedProvider,
    /// The SBFB network may serve the call.
    SbfbNetworkAllowed,
    /// The SBFB network must serve the call.
    SbfbNetworkRequired,
}

/// Family 7 of `CompiledPlan.v1`. The v1 graph format declares no placement
/// field (frozen form), so the compiler assigns the most restrictive class to
/// the whole plan: everything executes on the initiating node (WA-3 — the
/// run's authority is the initiating node). Per-node placement arrives with
/// the network verticals as a conscious v1 redefinition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompiledPlacement.v1")]
pub struct CompiledPlacement {
    /// Placement class applied to every node of this plan.
    pub default_class: PlacementClass,
}

/// One node occurrence of the expanded plan (family 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompiledNodeInstance.v1")]
pub struct CompiledNodeInstance {
    /// The workflow identity this occurrence belongs to.
    pub workflow_id: String,
    /// The template node id inside that workflow.
    pub node_id: String,
    /// The frozen kind tag (serde-authoritative spelling).
    pub kind: String,
    /// Expansion depth: 0 for the root workflow's nodes.
    pub depth: u32,
    /// The full normalized node body. Self-contained UP TO pinned
    /// references: node bodies are embedded, while agent definitions,
    /// prompts, schemas and corpora stay pinned BY DIGEST in
    /// `artefact_digests` — the engine executes from the plan and resolves
    /// those digests against the verified pack store, never against loose
    /// source documents.
    pub node: WorkflowNode,
}

/// One edge of the expanded plan, endpoints named by `node_instance_id`
/// (family 5). Sorted canonically `(from, from_port, to, to_port)` — edge
/// ORDER carries no meaning in a compiled plan, so the compiler erases it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompiledEdge.v1")]
pub struct CompiledEdge {
    /// Source `node_instance_id`.
    pub from: String,
    /// Source port.
    pub from_port: String,
    /// Destination `node_instance_id`.
    pub to: String,
    /// Destination port.
    pub to_port: String,
}

// `CompiledPlan.v1` frontier posture (Sprint 83 Phase H, doctrine §7): a
// LOCAL, derived artefact read by the host engine (Phases I/J) and inspected
// by the Hub — produced by this compiler, never signed or gossiped on its
// own, so it carries no `// FRONTIER:` domain/version tag (the reserved
// `DOMAIN_COMPILED_PLAN_V1` keeps zero producers and the frozen census of 29
// is untouched). Its machine contract is the generated schema snapshot pinned
// by `contracts::all_contract_schemas` (identity 6/7, len==7) plus the four
// committed `*.compiled.golden.json` fixtures.
/// `CompiledPlan.v1` — the immutable plan produced by the authoritative Rust
/// compiler (D3/D4). The nine field families of research §12.1, every
/// collection deterministic. Immutable during a run: editing a draft creates
/// a NEW plan; it never changes a running one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompiledPlan.v1")]
pub struct CompiledPlan {
    /// Family 9: producing compiler version + profile.
    pub compiler: CompilerProfile,
    /// Family 1: the pack source digest — the authoritative `content`
    /// digest RECOMPUTED from `PACK.lock` (never copied from the claim).
    pub pack_digest: String,
    /// Publisher-identity binding (§19.3: a model output cannot change a
    /// publisher): the pack's stable name.
    pub pack_name: String,
    /// Publisher-identity binding: the pack revision this plan derives from.
    pub pack_revision: u64,
    /// The root workflow this plan expands.
    pub root_workflow_id: String,
    /// Family 2: exact digests of every pinned artefact the plan touches,
    /// `category → id → blake3` (categories: `workflows`, `agents`,
    /// `prompts`, `schemas`, `tools`, `corpora`, `guardrails`,
    /// `subworkflows`).
    pub artefact_digests: BTreeMap<String, BTreeMap<String, String>>,
    /// Family 3: the CLOSED capability set — `capability_ref → sorted
    /// node_instance_ids` allowed to call it. Nothing outside this map can
    /// ever be granted; membership is DATA for the runtime (per-effect
    /// enforcement binding is Phases J/K/L).
    pub capability_closure: BTreeMap<String, Vec<String>>,
    /// Family 4: every node occurrence, keyed by `node_instance_id`
    /// (RFC 6901 path of node ids, pure function of the structural path).
    pub node_instances: BTreeMap<String, CompiledNodeInstance>,
    /// Family 5: the normalized wiring, canonically sorted.
    pub edges: Vec<CompiledEdge>,
    /// Family 5: entry occurrences per scope (`""` = the root scope, else
    /// the expanding occurrence's `node_instance_id`), in pinned-document
    /// order (deterministic; order is authored data, not presentation).
    pub entrypoints: BTreeMap<String, Vec<String>>,
    /// §19.3 HITL binding: the `human.review` occurrences of the plan,
    /// sorted — a model output cannot add, remove or reroute one.
    pub human_gates: Vec<String>,
    /// Family 5 digest: the semantic digest of the normalized graphs
    /// (edges canonically sorted BEFORE digesting — canvas order and
    /// outline order collapse to one digest, WA-T1.2 foundation). Derived
    /// and LINKED to `pack_digest`; never an autonomous integrity surface.
    pub graph_semantic_digest: String,
    /// Family 6: the budget ceiling of each workflow scope (`""` = root).
    pub scope_budgets: BTreeMap<String, BudgetPolicy>,
    /// Family 6: the declared budget of each budget-carrying occurrence,
    /// already proven ≤ its scope (and ≤ its agent definition) dimension by
    /// dimension.
    pub node_budgets: BTreeMap<String, BudgetPolicy>,
    /// Family 7: placement.
    pub placement: CompiledPlacement,
    /// Family 8: error policy per scope.
    pub error_policies: BTreeMap<String, ErrorPolicy>,
    /// Family 8: timeout policy per scope.
    pub timeout_policies: BTreeMap<String, TimeoutPolicy>,
    /// Family 8: cancellation policy per scope.
    pub cancellation_policies: BTreeMap<String, CancellationPolicy>,
}

/// The derived digest of a compiled plan (`plan_digest` of research §12.2),
/// under the Phase F digest door with the `compiled_plan` domain. The plan
/// cannot carry its own digest (self-reference), so it travels alongside.
///
/// # Errors
///
/// Propagates the canonicalisation errors of [`digest_for`].
pub fn compiled_plan_digest(plan: &CompiledPlan) -> Result<String, PackError> {
    let value = serde_json::to_value(plan)
        .map_err(|e| PackError::Canonicalisation(format!("plan does not serialise: {e}")))?;
    digest_for(PackDigestKind::CompiledPlan, &value)
}

/// RFC 6901 escaping of one path segment (`~` → `~0`, `/` → `~1`): what
/// makes `node_instance_id` collision-free even against node ids containing
/// the separator.
fn escape_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// The `node_instance_id` of a node under a scope: `scope + "/" + escaped
/// node id`. The root scope is `""`, so root occurrences read `/node-id` —
/// exactly a JSON Pointer.
fn instance_id(scope: &str, node_id: &str) -> String {
    format!("{scope}/{}", escape_segment(node_id))
}

/// The serde-authoritative kind tag of a node body.
fn kind_of(node: &WorkflowNode) -> String {
    serde_json::to_value(node).expect("node serialises")["kind"]
        .as_str()
        .expect("kind is a string")
        .to_string()
}

/// The four budget dimensions, compared one by one — never an aggregate.
fn budget_dimensions(b: &BudgetPolicy) -> [(&'static str, u64); 4] {
    [
        ("max_total_tokens", b.max_total_tokens),
        ("max_cost_micro_usd", b.max_cost_micro_usd),
        ("max_wall_clock_seconds", b.max_wall_clock_seconds),
        ("max_turns", u64::from(b.max_turns)),
    ]
}

fn check_budget_le(
    child: &BudgetPolicy,
    parent: &BudgetPolicy,
    path: &str,
) -> Result<(), CompileError> {
    for ((dimension, child_v), (_, parent_v)) in budget_dimensions(child)
        .into_iter()
        .zip(budget_dimensions(parent))
    {
        if child_v > parent_v {
            return Err(CompileError::BudgetOverflow {
                path: path.to_string(),
                dimension: dimension.to_string(),
                child: child_v,
                parent: parent_v,
            });
        }
    }
    Ok(())
}

/// Nominal v1 provider/model compatibility (§10.5): the CLI provider classes
/// are vendor-bound — `claude-code` serves `claude*` models, `codex` serves
/// `gpt*` models. `local`/`cloud` accept any model id. Structured output is
/// declared by every provider class in v1; honouring it is the adapters'
/// contract (Phases N/O).
fn check_model_policy(
    policy: &crate::agent_definition::ModelPolicy,
    path: &str,
) -> Result<(), CompileError> {
    let bound_prefix = match policy.provider_class {
        ProviderClass::ClaudeCode => Some(CLAUDE_CODE_MODEL_PREFIX),
        ProviderClass::Codex => Some(CODEX_MODEL_PREFIX),
        ProviderClass::Local | ProviderClass::Cloud => None,
    };
    if let Some(prefix) = bound_prefix
        && !policy.model_id.to_ascii_lowercase().starts_with(prefix)
    {
        return Err(CompileError::ProviderIncompatible {
            path: path.to_string(),
            detail: format!(
                "provider class is bound to `{prefix}*` models, got `{}`",
                policy.model_id
            ),
        });
    }
    Ok(())
}

/// Best-effort fail-closed scan of every string value of a graph/agent
/// document (§10.5 "secret literal" + "endpoint ou code exécutable
/// interdit"). Reuses the pack scanner's secret shape; unlike the pack's
/// report-only scan, the compiler REFUSES. Prompt/corpus FILE bytes are not
/// seen here (pinned by digest; scanned at pack-assembly).
///
/// Stated bounds: endpoint detection is the `://` substring ONLY —
/// slash-less URI schemes (`data:`, `javascript:`) are not recognised, and
/// a split or encoded literal walks past any regex-shaped scan. Both fall
/// in the same accepted residual (THREAT_MODEL §18 T-PLAN-SCAN-EVASION):
/// the enforced guarantee is the FORMAT (no endpoint/code field exists,
/// secrets are opaque refs), never this scan.
fn scan_document_strings(value: &serde_json::Value, path: &str) -> Result<(), CompileError> {
    match value {
        serde_json::Value::String(s) => {
            if looks_like_materialised_secret(s) {
                return Err(CompileError::SecretLiteral {
                    path: path.to_string(),
                });
            }
            if s.contains("://") {
                return Err(CompileError::ForbiddenEndpoint {
                    path: path.to_string(),
                    detail: "string carries a URL-scheme endpoint".to_string(),
                });
            }
            for marker in ["#!", "$("] {
                if s.contains(marker) {
                    return Err(CompileError::ForbiddenEndpoint {
                        path: path.to_string(),
                        detail: format!("string carries executable marker `{marker}`"),
                    });
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                scan_document_strings(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                scan_document_strings(member, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Does this capability ref carry a publication/egress marker dot-segment?
fn is_publication_capability(capability_ref: &str) -> bool {
    capability_ref
        .split('.')
        .any(|segment| PUBLICATION_CAPABILITY_MARKERS.contains(&segment))
}

/// Compile a resolved pack into its immutable plan. Pure and deterministic:
/// same input, same plan, byte for byte.
///
/// # Errors
///
/// The first refusal encountered, as a [`CompileError`] — every variant is
/// terminal; there is no degraded plan.
pub fn compile(input: &CompileInput) -> Result<CompiledPlan, CompileError> {
    // The authoritative pack digest is RECOMPUTED from the lock (S3-07),
    // never read from the signed claim.
    let pack_digest =
        content_digest_of_lock(&input.pack.lock).map_err(|e| CompileError::InvalidDocument {
            path: "$.pack.lock".to_string(),
            detail: format!("content digest: {e}"),
        })?;

    // The lock's digest set: every reference the plan touches must be pinned
    // by the pack ("référence non pinnée" otherwise).
    let mut lock_digests: BTreeSet<&str> = BTreeSet::new();
    for entry in &input.pack.lock.entries {
        if !is_lower_hex_digest(&entry.blake3) {
            return Err(CompileError::UnpinnedReference {
                path: format!("$.pack.lock[{}]", entry.path),
                detail: "lock digest is not lowercase BLAKE3 hex".to_string(),
            });
        }
        lock_digests.insert(entry.blake3.as_str());
    }

    let manifest_capabilities: BTreeSet<&str> = input
        .pack
        .requested_capabilities
        .iter()
        .map(String::as_str)
        .collect();

    let mut ctx = PlanBuilder {
        input,
        pack_digest,
        lock_digests,
        manifest_capabilities,
        artefact_digests: BTreeMap::new(),
        capability_closure: BTreeMap::new(),
        node_instances: BTreeMap::new(),
        edges: Vec::new(),
        entrypoints: BTreeMap::new(),
        human_gates: Vec::new(),
        scope_budgets: BTreeMap::new(),
        node_budgets: BTreeMap::new(),
        error_policies: BTreeMap::new(),
        timeout_policies: BTreeMap::new(),
        cancellation_policies: BTreeMap::new(),
        semantic_views: BTreeMap::new(),
        agents_parsed: BTreeMap::new(),
    };

    // Root workflow: binding, strict parse, then the recursive expansion.
    let root = ctx.load_workflow_document(&input.root_workflow, "$.root_workflow", None)?;
    let root_id = root.workflow_id.clone();
    ctx.expand_workflow("", &root, None, 0)?;

    let graph_semantic_digest = digest_for(
        PackDigestKind::GraphSemantic,
        &serde_json::Value::Object(
            ctx.semantic_views
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ),
    )
    .map_err(|e| CompileError::InvalidDocument {
        path: "$.graph_semantic".to_string(),
        detail: format!("semantic digest: {e}"),
    })?;

    let PlanBuilder {
        pack_digest,
        artefact_digests,
        capability_closure,
        node_instances,
        mut edges,
        entrypoints,
        mut human_gates,
        scope_budgets,
        node_budgets,
        error_policies,
        timeout_policies,
        cancellation_policies,
        ..
    } = ctx;
    edges.sort();
    human_gates.sort();
    let mut capability_closure = capability_closure;
    for instances in capability_closure.values_mut() {
        instances.sort();
    }

    Ok(CompiledPlan {
        compiler: CompilerProfile {
            version: COMPILER_VERSION.to_string(),
            profile: COMPILER_PROFILE_STRICT_V1.to_string(),
        },
        pack_digest,
        pack_name: input.pack.name.clone(),
        pack_revision: input.pack.pack_revision,
        root_workflow_id: root_id,
        artefact_digests,
        capability_closure,
        node_instances,
        edges,
        entrypoints,
        human_gates,
        graph_semantic_digest,
        scope_budgets,
        node_budgets,
        placement: CompiledPlacement {
            default_class: PlacementClass::LocalOnly,
        },
        error_policies,
        timeout_policies,
        cancellation_policies,
    })
}

/// The working state of one compilation.
struct PlanBuilder<'a> {
    input: &'a CompileInput,
    pack_digest: String,
    lock_digests: BTreeSet<&'a str>,
    manifest_capabilities: BTreeSet<&'a str>,
    artefact_digests: BTreeMap<String, BTreeMap<String, String>>,
    capability_closure: BTreeMap<String, Vec<String>>,
    node_instances: BTreeMap<String, CompiledNodeInstance>,
    edges: Vec<CompiledEdge>,
    entrypoints: BTreeMap<String, Vec<String>>,
    human_gates: Vec<String>,
    scope_budgets: BTreeMap<String, BudgetPolicy>,
    node_budgets: BTreeMap<String, BudgetPolicy>,
    error_policies: BTreeMap<String, ErrorPolicy>,
    timeout_policies: BTreeMap<String, TimeoutPolicy>,
    cancellation_policies: BTreeMap<String, CancellationPolicy>,
    /// `workflow_id →` normalized semantic view (edges sorted), the source
    /// of the plan-level `graph_semantic_digest`.
    semantic_views: BTreeMap<String, serde_json::Value>,
    /// Agent definitions already strictly parsed, by agent id, together
    /// with the DIGEST they were bound under (Codex r2: a cache hit must
    /// still confront the new reference's pin — two `agent_refs` naming one
    /// id under two digests is a refusal, never a silent first-wins).
    agents_parsed: BTreeMap<String, (String, SbfbAgentDefinition)>,
}

impl<'a> PlanBuilder<'a> {
    /// Derive the pin of provided bytes and bind it: the compiler hashes
    /// the bytes ITSELF (Codex r2 — a caller-supplied pin proved nothing
    /// about the document actually handed over), then requires lock
    /// membership and, when a reference named this document, pin equality.
    fn bind_provided(
        &self,
        provided: &ProvidedDocument,
        path: &str,
        expected_pin: Option<&str>,
    ) -> Result<String, CompileError> {
        let pin = provided.pin();
        if !self.lock_digests.contains(pin.as_str()) {
            return Err(CompileError::UnpinnedReference {
                path: path.to_string(),
                detail: format!("provided bytes hash to {pin}, which PACK.lock does not pin"),
            });
        }
        if let Some(expected) = expected_pin
            && expected != pin
        {
            return Err(CompileError::UnpinnedReference {
                path: path.to_string(),
                detail: format!("reference pins {expected}, provided bytes hash to {pin}"),
            });
        }
        Ok(pin)
    }

    /// Bind + strictly parse one workflow document: bytes hashed and bound
    /// to the lock and the referencing pin, secret/endpoint scan,
    /// presentation-key rule over its data-driven keys.
    fn load_workflow_document(
        &mut self,
        provided: &ProvidedDocument,
        path: &str,
        expected_pin: Option<&str>,
    ) -> Result<SbfbWorkflowGraph, CompileError> {
        let pin = self.bind_provided(provided, path, expected_pin)?;
        let document: serde_json::Value =
            serde_json::from_slice(&provided.bytes).map_err(|e| CompileError::InvalidDocument {
                path: path.to_string(),
                detail: format!("not valid JSON: {e}"),
            })?;
        scan_document_strings(&document, path)?;
        let graph: SbfbWorkflowGraph =
            serde_json::from_value(document).map_err(|e| CompileError::InvalidDocument {
                path: path.to_string(),
                detail: e.to_string(),
            })?;

        // Freeze §8 rule 4: data-driven keys stay disjoint from the reserved
        // presentation names, or two different plans share one semantic
        // digest.
        for node_id in graph.nodes.keys() {
            if crate::pack::PRESENTATION_KEYS.contains(&node_id.as_str()) {
                return Err(CompileError::PresentationKeyAsData {
                    workflow_id: graph.workflow_id.clone(),
                    detail: format!("node id `{node_id}` is a reserved presentation name"),
                });
            }
        }
        for (node_id, node) in &graph.nodes {
            if let WorkflowNode::Transform(t) = node {
                for key in t.field_map.keys() {
                    if crate::pack::PRESENTATION_KEYS.contains(&key.as_str()) {
                        return Err(CompileError::PresentationKeyAsData {
                            workflow_id: graph.workflow_id.clone(),
                            detail: format!(
                                "field_map key `{key}` of node `{node_id}` is a reserved presentation name"
                            ),
                        });
                    }
                }
            }
        }

        // Normalized semantic view: the typed graph with edges canonically
        // sorted, presentation-stripped (identity on semantic documents).
        // The one-digest-per-workflow_id rule below makes an overwrite here
        // same-bytes-only, hence benign.
        let mut normalized = graph.clone();
        normalized.edges.sort_by_key(edge_sort_key);
        let view = graph_semantic_view(
            &serde_json::to_value(&normalized).expect("typed graph serialises"),
        );
        self.semantic_views.insert(graph.workflow_id.clone(), view);

        self.record_digest("workflows", &graph.workflow_id, &pin, path)?;
        Ok(graph)
    }

    /// Bind + strictly parse one agent definition (memoised by id, the
    /// cached DIGEST re-confronted on every reference).
    fn load_agent(
        &mut self,
        workflow_id: &str,
        agent_ref: &crate::graph::AgentRef,
    ) -> Result<SbfbAgentDefinition, CompileError> {
        let path = format!("$.agents.{}", agent_ref.agent_id);
        if let Some((cached_digest, parsed)) = self.agents_parsed.get(&agent_ref.agent_id) {
            if cached_digest != &agent_ref.blake3 {
                return Err(CompileError::UnpinnedReference {
                    path,
                    detail: format!(
                        "agent id `{}` referenced under two digests ({cached_digest} vs {}) — one id, one artefact",
                        agent_ref.agent_id, agent_ref.blake3
                    ),
                });
            }
            return Ok(parsed.clone());
        }
        let provided = self.input.agents.get(&agent_ref.agent_id).ok_or_else(|| {
            CompileError::UnpinnedReference {
                path: path.clone(),
                detail: format!(
                    "agent `{}` referenced by workflow `{workflow_id}` was not provided",
                    agent_ref.agent_id
                ),
            }
        })?;
        self.check_pin(&path, &agent_ref.blake3)?;
        let pin = self.bind_provided(provided, &path, Some(&agent_ref.blake3))?;
        let document: serde_json::Value =
            serde_json::from_slice(&provided.bytes).map_err(|e| CompileError::InvalidDocument {
                path: path.clone(),
                detail: format!("not valid JSON: {e}"),
            })?;
        scan_document_strings(&document, &path)?;
        let agent: SbfbAgentDefinition =
            serde_json::from_value(document).map_err(|e| CompileError::InvalidDocument {
                path: path.clone(),
                detail: e.to_string(),
            })?;
        if agent.agent_id != agent_ref.agent_id {
            return Err(CompileError::UnpinnedReference {
                path,
                detail: format!(
                    "provided agent declares id `{}`, reference names `{}`",
                    agent.agent_id, agent_ref.agent_id
                ),
            });
        }
        check_model_policy(
            &agent.model_policy,
            &format!("$.agents.{}.model_policy", agent.agent_id),
        )?;
        self.record_digest("agents", &agent.agent_id, &pin, &path)?;
        self.record_digest(
            "prompts",
            &agent.instructions_ref.prompt_id,
            &agent.instructions_ref.blake3,
            &path,
        )?;
        self.check_pin(
            &format!("$.agents.{}.instructions_ref", agent.agent_id),
            &agent.instructions_ref.blake3,
        )?;
        for (schema, field) in [
            (&agent.input_schema_ref, "input_schema_ref"),
            (&agent.output_schema_ref, "output_schema_ref"),
        ] {
            self.check_schema_ref(schema, &format!("$.agents.{}.{field}", agent.agent_id))?;
        }
        for tool in &agent.tool_refs {
            self.check_pin(
                &format!("$.agents.{}.tool_refs.{}", agent.agent_id, tool.tool_id),
                &tool.blake3,
            )?;
            self.record_digest(
                "tools",
                &tool.tool_id,
                &tool.blake3,
                &format!("$.agents.{}.tool_refs.{}", agent.agent_id, tool.tool_id),
            )?;
        }
        for guardrail in &agent.guardrail_refs {
            self.check_pin(
                &format!(
                    "$.agents.{}.guardrail_refs.{}",
                    agent.agent_id, guardrail.guardrail_id
                ),
                &guardrail.blake3,
            )?;
            self.record_digest(
                "guardrails",
                &guardrail.guardrail_id,
                &guardrail.blake3,
                &format!(
                    "$.agents.{}.guardrail_refs.{}",
                    agent.agent_id, guardrail.guardrail_id
                ),
            )?;
        }
        self.agents_parsed
            .insert(agent.agent_id.clone(), (pin, agent.clone()));
        Ok(agent)
    }

    fn check_pin(&self, path: &str, digest: &str) -> Result<(), CompileError> {
        if !is_lower_hex_digest(digest) {
            return Err(CompileError::UnpinnedReference {
                path: path.to_string(),
                detail: "digest is not lowercase BLAKE3 hex".to_string(),
            });
        }
        if !self.lock_digests.contains(digest) {
            return Err(CompileError::UnpinnedReference {
                path: path.to_string(),
                detail: "digest is not pinned by PACK.lock".to_string(),
            });
        }
        Ok(())
    }

    fn check_schema_ref(&mut self, schema: &SchemaRef, path: &str) -> Result<(), CompileError> {
        self.check_pin(path, &schema.blake3)?;
        // Nominal-by-digest coherence (freeze §7): one schema id, ONE digest
        // across the whole plan. Two ports naming the same id under two
        // different byte pins is the drift/poisoning case this rule closes.
        // (`record_digest` enforces the same rule for every category; the
        // dedicated variant here names the SCHEMA/port diagnostic.)
        if let Some(previous) = self
            .artefact_digests
            .get("schemas")
            .and_then(|m| m.get(&schema.schema_id))
            && previous != &schema.blake3
        {
            return Err(CompileError::PortTypeMismatch {
                workflow_id: String::new(),
                detail: format!(
                    "schema id `{}` pinned under two digests ({previous} vs {}) at {path}",
                    schema.schema_id, schema.blake3
                ),
            });
        }
        self.record_digest("schemas", &schema.schema_id, &schema.blake3, path)
    }

    /// One id, ONE digest — for EVERY artefact category (Codex r2: a silent
    /// overwrite in `artefact_digests` would let two homonymous prompts,
    /// agents, workflows, tools, corpora or guardrails coexist while the
    /// plan pins only one of them).
    fn record_digest(
        &mut self,
        category: &str,
        id: &str,
        digest: &str,
        path: &str,
    ) -> Result<(), CompileError> {
        let entry = self
            .artefact_digests
            .entry(category.to_string())
            .or_default();
        if let Some(previous) = entry.get(id) {
            if previous != digest {
                return Err(CompileError::UnpinnedReference {
                    path: path.to_string(),
                    detail: format!(
                        "{category} id `{id}` bound to two digests ({previous} vs {digest}) — one id, one artefact"
                    ),
                });
            }
            return Ok(());
        }
        entry.insert(id.to_string(), digest.to_string());
        Ok(())
    }

    /// Validate one workflow and expand its occurrences under `scope`
    /// (`""` = root), recursing into pinned children.
    fn expand_workflow(
        &mut self,
        scope: &str,
        graph: &SbfbWorkflowGraph,
        parent_budget: Option<&BudgetPolicy>,
        depth: u32,
    ) -> Result<(), CompileError> {
        if depth > MAX_PLAN_DEPTH {
            return Err(CompileError::PlanTooDeep {
                detail: format!(
                    "workflow `{}` nests deeper than MAX_PLAN_DEPTH={MAX_PLAN_DEPTH}",
                    graph.workflow_id
                ),
            });
        }
        let wf = graph.workflow_id.as_str();

        // Child scope budget ≤ parent scope budget, dimension by dimension.
        if let Some(parent) = parent_budget {
            check_budget_le(
                &graph.budget_policy,
                parent,
                &format!("workflow `{wf}` budget_policy"),
            )?;
        }
        self.scope_budgets
            .insert(scope.to_string(), graph.budget_policy.clone());
        self.error_policies
            .insert(scope.to_string(), graph.error_policy.clone());
        self.timeout_policies
            .insert(scope.to_string(), graph.timeout_policy.clone());
        self.cancellation_policies
            .insert(scope.to_string(), graph.cancellation_policy.clone());

        // §10.5: every capability this workflow requests must be in the pack
        // manifest.
        for capability in &graph.capability_requests {
            if !self.manifest_capabilities.contains(capability.as_str()) {
                return Err(CompileError::CapabilityNotInManifest {
                    workflow_id: wf.to_string(),
                    capability: capability.clone(),
                });
            }
        }

        self.validate_structure(graph)?;
        self.validate_ports_and_joins(graph)?;
        self.validate_reachability(graph)?;
        self.validate_dag(graph)?;
        self.validate_artifact_outputs(graph)?;

        // Agent bindings BEFORE the edge-level port-type pass (Codex r2:
        // `declared_output_schema`/`declared_input_schema` read
        // `agents_parsed` — loading agents after the pass silently exempted
        // every agent-adjacent edge from the nominal type check).
        let mut declared_agents: BTreeMap<&str, &crate::graph::AgentRef> = BTreeMap::new();
        for agent_ref in &graph.agent_refs {
            declared_agents.insert(agent_ref.agent_id.as_str(), agent_ref);
            self.load_agent(wf, agent_ref)?;
        }
        let declared_subworkflows: BTreeMap<&str, &crate::graph::SubworkflowRef> = graph
            .subworkflow_refs
            .iter()
            .map(|r| (r.workflow_id.as_str(), r))
            .collect();
        for sub_ref in &graph.subworkflow_refs {
            // A DECLARED child is bound to the lock even before (and
            // whether or not) any node expands it (Codex r2).
            self.check_pin(
                &format!("$.workflows.{wf}.subworkflow_refs.{}", sub_ref.workflow_id),
                &sub_ref.blake3,
            )?;
            self.record_digest(
                "subworkflows",
                &sub_ref.workflow_id,
                &sub_ref.blake3,
                &format!("$.workflows.{wf}.subworkflow_refs.{}", sub_ref.workflow_id),
            )?;
        }

        self.validate_port_types(graph)?;
        self.validate_hitl_domination(graph)?;
        self.validate_classification_provenance(graph)?;

        // Workflow-level schema pins.
        self.check_schema_ref(
            &graph.input_schema_ref,
            &format!("$.workflows.{wf}.input_schema_ref"),
        )?;
        self.check_schema_ref(
            &graph.output_schema_ref,
            &format!("$.workflows.{wf}.output_schema_ref"),
        )?;
        for artefact in &graph.artifact_outputs {
            self.check_schema_ref(
                &artefact.schema_ref,
                &format!("$.workflows.{wf}.artifact_outputs.{}", artefact.name),
            )?;
        }

        // Entry occurrences of this scope.
        let entries: Vec<String> = graph
            .entrypoints
            .iter()
            .map(|e| instance_id(scope, e))
            .collect();
        self.entrypoints.insert(scope.to_string(), entries);

        // Occurrences, per-node checks, recursion into children.
        for (node_id, node) in &graph.nodes {
            let id = instance_id(scope, node_id);
            let kind = kind_of(node);
            match node {
                WorkflowNode::ModelInvoke(n) => {
                    check_model_policy(
                        &n.model_policy,
                        &format!("$.workflows.{wf}.nodes.{node_id}.model_policy"),
                    )?;
                    check_budget_le(
                        &n.budget,
                        &graph.budget_policy,
                        &format!("node `{node_id}` of workflow `{wf}`"),
                    )?;
                    self.node_budgets.insert(id.clone(), n.budget.clone());
                    self.check_pin(
                        &format!("$.workflows.{wf}.nodes.{node_id}.prompt_ref"),
                        &n.prompt_ref.blake3,
                    )?;
                    self.record_digest(
                        "prompts",
                        &n.prompt_ref.prompt_id,
                        &n.prompt_ref.blake3,
                        &format!("$.workflows.{wf}.nodes.{node_id}.prompt_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.input_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.input_schema_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.output_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.output_schema_ref"),
                    )?;
                }
                WorkflowNode::Classifier(n) => {
                    check_model_policy(
                        &n.model_policy,
                        &format!("$.workflows.{wf}.nodes.{node_id}.model_policy"),
                    )?;
                    check_budget_le(
                        &n.budget,
                        &graph.budget_policy,
                        &format!("node `{node_id}` of workflow `{wf}`"),
                    )?;
                    self.node_budgets.insert(id.clone(), n.budget.clone());
                    self.check_pin(
                        &format!("$.workflows.{wf}.nodes.{node_id}.prompt_ref"),
                        &n.prompt_ref.blake3,
                    )?;
                    self.record_digest(
                        "prompts",
                        &n.prompt_ref.prompt_id,
                        &n.prompt_ref.blake3,
                        &format!("$.workflows.{wf}.nodes.{node_id}.prompt_ref"),
                    )?;
                }
                WorkflowNode::AgentInvoke(n) => {
                    let agent_ref = declared_agents.get(n.agent_id.as_str()).ok_or_else(|| {
                        CompileError::UndeclaredAgent {
                            workflow_id: wf.to_string(),
                            detail: format!(
                                "node `{node_id}` invokes `{}`, absent from agent_refs",
                                n.agent_id
                            ),
                        }
                    })?;
                    let agent = self.load_agent(wf, agent_ref)?;
                    check_budget_le(
                        &n.budget,
                        &agent.budget,
                        &format!("node `{node_id}` vs agent `{}` definition", n.agent_id),
                    )?;
                    check_budget_le(
                        &n.budget,
                        &graph.budget_policy,
                        &format!("node `{node_id}` of workflow `{wf}`"),
                    )?;
                    self.node_budgets.insert(id.clone(), n.budget.clone());
                }
                WorkflowNode::AgentAsTool(n) => {
                    let agent_ref = declared_agents.get(n.agent_id.as_str()).ok_or_else(|| {
                        CompileError::UndeclaredAgent {
                            workflow_id: wf.to_string(),
                            detail: format!(
                                "node `{node_id}` calls `{}` as tool, absent from agent_refs",
                                n.agent_id
                            ),
                        }
                    })?;
                    let agent = self.load_agent(wf, agent_ref)?;
                    check_budget_le(
                        &n.budget,
                        &agent.budget,
                        &format!("node `{node_id}` vs agent `{}` definition", n.agent_id),
                    )?;
                    check_budget_le(
                        &n.budget,
                        &graph.budget_policy,
                        &format!("node `{node_id}` of workflow `{wf}`"),
                    )?;
                    self.node_budgets.insert(id.clone(), n.budget.clone());
                    self.check_schema_ref(
                        &n.input_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.input_schema_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.output_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.output_schema_ref"),
                    )?;
                }
                WorkflowNode::Handoff(n) => {
                    if !declared_agents.contains_key(n.target_agent_id.as_str()) {
                        return Err(CompileError::UndeclaredAgent {
                            workflow_id: wf.to_string(),
                            detail: format!(
                                "handoff `{node_id}` targets `{}`, absent from agent_refs",
                                n.target_agent_id
                            ),
                        });
                    }
                    // §10.5: every upstream agent invocation whose flow
                    // reaches this handoff must allowlist the target —
                    // TRANSITIVELY through non-agent nodes (Codex r2: a
                    // transform between the agent and the handoff must not
                    // launder the giver check).
                    for giver_id in upstream_agent_invokers(graph, node_id) {
                        let Some(WorkflowNode::AgentInvoke(giver)) = graph.nodes.get(&giver_id)
                        else {
                            continue;
                        };
                        let giver_def = self
                            .agents_parsed
                            .get(&giver.agent_id)
                            .map(|(_, a)| a)
                            .expect("giver agent parsed with agent_refs");
                        if !giver_def.allowed_handoffs.contains(&n.target_agent_id) {
                            return Err(CompileError::UnauthorizedHandoff {
                                workflow_id: wf.to_string(),
                                node_id: node_id.clone(),
                                detail: format!(
                                    "giver `{}` does not allowlist target `{}`",
                                    giver.agent_id, n.target_agent_id
                                ),
                            });
                        }
                    }
                    self.check_schema_ref(
                        &n.payload_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.payload_schema_ref"),
                    )?;
                }
                WorkflowNode::CapabilityCall(n) => {
                    if !graph.capability_requests.contains(&n.capability_ref) {
                        return Err(CompileError::CapabilityNotRequested {
                            workflow_id: wf.to_string(),
                            node_id: node_id.clone(),
                            capability: n.capability_ref.clone(),
                        });
                    }
                    self.capability_closure
                        .entry(n.capability_ref.clone())
                        .or_default()
                        .push(id.clone());
                    self.check_schema_ref(
                        &n.args_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.args_schema_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.output_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.output_schema_ref"),
                    )?;
                }
                WorkflowNode::KnowledgeQuery(n) => {
                    self.check_pin(
                        &format!("$.workflows.{wf}.nodes.{node_id}.corpus_ref"),
                        &n.corpus_ref.blake3,
                    )?;
                    self.record_digest(
                        "corpora",
                        &n.corpus_ref.corpus_id,
                        &n.corpus_ref.blake3,
                        &format!("$.workflows.{wf}.nodes.{node_id}.corpus_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.query_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.query_schema_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.output_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.output_schema_ref"),
                    )?;
                }
                WorkflowNode::HumanReview(n) => {
                    self.human_gates.push(id.clone());
                    self.check_schema_ref(
                        &n.payload_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.payload_schema_ref"),
                    )?;
                }
                WorkflowNode::Input(n) => {
                    self.check_schema_ref(
                        &n.schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.schema_ref"),
                    )?;
                }
                WorkflowNode::Output(n) => {
                    self.check_schema_ref(
                        &n.schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.schema_ref"),
                    )?;
                }
                WorkflowNode::Transform(n) => {
                    self.check_schema_ref(
                        &n.input_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.input_schema_ref"),
                    )?;
                    self.check_schema_ref(
                        &n.output_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.output_schema_ref"),
                    )?;
                }
                WorkflowNode::WaitSignal(n) => {
                    self.check_schema_ref(
                        &n.payload_schema_ref,
                        &format!("$.workflows.{wf}.nodes.{node_id}.payload_schema_ref"),
                    )?;
                }
                WorkflowNode::Map(n) => {
                    self.expand_child(
                        scope,
                        node_id,
                        &n.body_subworkflow_ref,
                        &declared_subworkflows,
                        graph,
                        depth,
                    )?;
                }
                WorkflowNode::Repeat(n) => {
                    self.expand_child(
                        scope,
                        node_id,
                        &n.body_subworkflow_ref,
                        &declared_subworkflows,
                        graph,
                        depth,
                    )?;
                }
                WorkflowNode::Subworkflow(n) => {
                    self.expand_child(
                        scope,
                        node_id,
                        &n.subworkflow_ref,
                        &declared_subworkflows,
                        graph,
                        depth,
                    )?;
                }
                WorkflowNode::Switch(_) | WorkflowNode::Parallel(_) | WorkflowNode::Join(_) => {}
            }
            self.node_instances.insert(
                id,
                CompiledNodeInstance {
                    workflow_id: wf.to_string(),
                    node_id: node_id.clone(),
                    kind,
                    depth,
                    node: node.clone(),
                },
            );
        }

        // Scope edges, endpoints as occurrence ids.
        for edge in &graph.edges {
            self.edges.push(CompiledEdge {
                from: instance_id(scope, &edge.from),
                from_port: edge.from_port.clone(),
                to: instance_id(scope, &edge.to),
                to_port: edge.to_port.clone(),
            });
        }
        Ok(())
    }

    /// Recurse into one pinned child workflow under the expanding node.
    fn expand_child(
        &mut self,
        scope: &str,
        node_id: &str,
        sub_ref: &crate::graph::SubworkflowRef,
        declared: &BTreeMap<&str, &crate::graph::SubworkflowRef>,
        parent: &SbfbWorkflowGraph,
        depth: u32,
    ) -> Result<(), CompileError> {
        let wf = parent.workflow_id.as_str();
        let path = format!("$.workflows.{wf}.nodes.{node_id}");
        match declared.get(sub_ref.workflow_id.as_str()) {
            Some(declared_ref) if *declared_ref == sub_ref => {}
            Some(_) => {
                return Err(CompileError::UnpinnedReference {
                    path,
                    detail: format!(
                        "node pins subworkflow `{}` under a digest different from subworkflow_refs",
                        sub_ref.workflow_id
                    ),
                });
            }
            None => {
                return Err(CompileError::UnpinnedReference {
                    path,
                    detail: format!(
                        "subworkflow `{}` is absent from subworkflow_refs",
                        sub_ref.workflow_id
                    ),
                });
            }
        }
        let provided = self
            .input
            .subworkflows
            .get(&sub_ref.workflow_id)
            .ok_or_else(|| CompileError::UnpinnedReference {
                path: path.clone(),
                detail: format!(
                    "subworkflow `{}` was not provided to the compiler",
                    sub_ref.workflow_id
                ),
            })?
            .clone();
        let child = self.load_workflow_document(&provided, &path, Some(sub_ref.blake3.as_str()))?;
        if child.workflow_id != sub_ref.workflow_id {
            return Err(CompileError::UnpinnedReference {
                path,
                detail: format!(
                    "provided subworkflow declares id `{}`, reference names `{}`",
                    child.workflow_id, sub_ref.workflow_id
                ),
            });
        }
        let child_scope = instance_id(scope, node_id);
        self.expand_workflow(&child_scope, &child, Some(&parent.budget_policy), depth + 1)
    }

    /// Freeze §8 rules 1 (edge integrity + duplicates) and 5 (routing
    /// labels sane), plus entrypoint existence.
    fn validate_structure(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        if graph.entrypoints.is_empty() {
            return Err(CompileError::NoEntrypoint {
                workflow_id: wf.to_string(),
                detail: "entrypoints is empty".to_string(),
            });
        }
        for entry in &graph.entrypoints {
            if !graph.nodes.contains_key(entry) {
                return Err(CompileError::NoEntrypoint {
                    workflow_id: wf.to_string(),
                    detail: format!("entrypoint `{entry}` names no node"),
                });
            }
        }
        let mut seen: BTreeSet<(&str, &str, &str, &str)> = BTreeSet::new();
        for edge in &graph.edges {
            for endpoint in [&edge.from, &edge.to] {
                if !graph.nodes.contains_key(endpoint) {
                    return Err(CompileError::DanglingEdge {
                        workflow_id: wf.to_string(),
                        detail: format!(
                            "edge {}:{} → {}:{} names unknown node `{endpoint}`",
                            edge.from, edge.from_port, edge.to, edge.to_port
                        ),
                    });
                }
            }
            if !seen.insert((&edge.from, &edge.from_port, &edge.to, &edge.to_port)) {
                return Err(CompileError::DuplicateEdge {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "edge {}:{} → {}:{} appears twice",
                        edge.from, edge.from_port, edge.to, edge.to_port
                    ),
                });
            }
        }
        for (node_id, node) in &graph.nodes {
            let labels: Option<&Vec<String>> = match node {
                WorkflowNode::Switch(n) => Some(&n.branch_labels),
                WorkflowNode::Classifier(n) => Some(&n.branch_labels),
                WorkflowNode::Parallel(n) => Some(&n.branch_labels),
                WorkflowNode::HumanReview(n) => Some(&n.decision_labels),
                _ => None,
            };
            if let Some(labels) = labels {
                if labels.is_empty() {
                    return Err(CompileError::InvalidRoutingLabels {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: "label set is empty".to_string(),
                    });
                }
                let mut set = BTreeSet::new();
                for label in labels {
                    if !set.insert(label.as_str()) {
                        return Err(CompileError::InvalidRoutingLabels {
                            workflow_id: wf.to_string(),
                            node_id: node_id.clone(),
                            detail: format!("label `{label}` is duplicated"),
                        });
                    }
                    // Storage-aligned bound (Phase K): a label the engine's
                    // deposit door would refuse to STORE must not compile.
                    if label.len() > MAX_DECISION_LABEL_BYTES {
                        return Err(CompileError::InvalidRoutingLabels {
                            workflow_id: wf.to_string(),
                            node_id: node_id.clone(),
                            detail: format!(
                                "label of {} bytes exceeds the {MAX_DECISION_LABEL_BYTES}-byte storage bound",
                                label.len()
                            ),
                        });
                    }
                }
                let default_reserved =
                    matches!(node, WorkflowNode::Switch(_) | WorkflowNode::Classifier(_));
                if default_reserved && set.contains(DEFAULT_PORT) {
                    return Err(CompileError::InvalidRoutingLabels {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: format!("`{DEFAULT_PORT}` collides with the reserved port"),
                    });
                }
            }
        }
        Ok(())
    }

    /// Port validity (freeze §8 rule 2): every edge leaves a port its node
    /// exposes, every declared routing port is wired exactly once, `error`
    /// ports exist exactly when the policy routes them, destinations receive
    /// on `in`.
    fn validate_ports_and_joins(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        let route_error = graph.error_policy.on_node_failure == NodeFailureAction::RouteErrorPort;

        // `to_port` is NOT constrained in v1: destination ports are named by
        // the consumer's input schema FIELDS (the conception golden fans two
        // ports, `variants` and `checks`, into its judge), and judging them
        // would require opening schema bytes this compiler never receives —
        // the shape-level fan-in typing is the goldens' own contract
        // (`conception_fanin_types_align`).

        for (node_id, node) in &graph.nodes {
            let out_edges: Vec<&WorkflowEdge> =
                graph.edges.iter().filter(|e| &e.from == node_id).collect();
            // The exclusive-routing ports must be wired EXACTLY once each;
            // `out`-class ports may fan out.
            let exact_ports: Option<Vec<String>> = match node {
                WorkflowNode::Switch(n) => Some(
                    n.branch_labels
                        .iter()
                        .cloned()
                        .chain([DEFAULT_PORT.to_string()])
                        .collect(),
                ),
                WorkflowNode::Classifier(n) => Some(
                    n.branch_labels
                        .iter()
                        .cloned()
                        .chain([DEFAULT_PORT.to_string()])
                        .collect(),
                ),
                WorkflowNode::Parallel(n) => Some(n.branch_labels.clone()),
                WorkflowNode::HumanReview(n) => Some(n.decision_labels.clone()),
                _ => None,
            };
            match exact_ports {
                Some(ports) => {
                    let port_set: BTreeSet<&str> = ports.iter().map(String::as_str).collect();
                    for edge in &out_edges {
                        if !port_set.contains(edge.from_port.as_str()) {
                            return Err(CompileError::UnknownPort {
                                workflow_id: wf.to_string(),
                                detail: format!(
                                    "node `{node_id}` exposes no port `{}`",
                                    edge.from_port
                                ),
                            });
                        }
                    }
                    for port in &ports {
                        let wired = out_edges.iter().filter(|e| &e.from_port == port).count();
                        if wired == 0 {
                            return Err(CompileError::UnwiredPort {
                                workflow_id: wf.to_string(),
                                detail: format!("port `{port}` of node `{node_id}` has no edge"),
                            });
                        }
                        if wired > 1 {
                            return Err(CompileError::UnknownPort {
                                workflow_id: wf.to_string(),
                                detail: format!(
                                    "exclusive port `{port}` of node `{node_id}` is wired {wired} times"
                                ),
                            });
                        }
                    }
                }
                None => {
                    let data_port = match node {
                        WorkflowNode::Handoff(_) => Some(HANDOFF_PORT),
                        WorkflowNode::Output(_) => None,
                        _ => Some(OUT_PORT),
                    };
                    for edge in &out_edges {
                        let valid = data_port == Some(edge.from_port.as_str())
                            || (route_error
                                && edge.from_port == ERROR_PORT
                                && !matches!(node, WorkflowNode::Output(_)));
                        if !valid {
                            return Err(CompileError::UnknownPort {
                                workflow_id: wf.to_string(),
                                detail: format!(
                                    "node `{node_id}` exposes no port `{}`",
                                    edge.from_port
                                ),
                            });
                        }
                    }
                    if route_error
                        && !matches!(node, WorkflowNode::Output(_))
                        && !out_edges.iter().any(|e| e.from_port == ERROR_PORT)
                    {
                        return Err(CompileError::UnwiredPort {
                            workflow_id: wf.to_string(),
                            detail: format!(
                                "error policy routes failures but node `{node_id}` wires no `{ERROR_PORT}` port"
                            ),
                        });
                    }
                    // A handoff transfers control exactly once.
                    if matches!(node, WorkflowNode::Handoff(_)) {
                        let to_edges = out_edges
                            .iter()
                            .filter(|e| e.from_port == HANDOFF_PORT)
                            .count();
                        if to_edges != 1 {
                            return Err(CompileError::UnwiredPort {
                                workflow_id: wf.to_string(),
                                detail: format!(
                                    "handoff `{node_id}` must transfer control exactly once (found {to_edges})"
                                ),
                            });
                        }
                    }
                }
            }
        }

        // Joins: declared sources exist, are unique, actually feed the join,
        // no undeclared feeder, and the policy is satisfiable.
        for (node_id, node) in &graph.nodes {
            let WorkflowNode::Join(join) = node else {
                continue;
            };
            let mut sources = BTreeSet::new();
            for source in &join.source_node_ids {
                if !graph.nodes.contains_key(source) {
                    return Err(CompileError::UnsatisfiableJoin {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: format!("declared source `{source}` names no node"),
                    });
                }
                if !sources.insert(source.as_str()) {
                    return Err(CompileError::UnsatisfiableJoin {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: format!("declared source `{source}` is duplicated"),
                    });
                }
                if !graph
                    .edges
                    .iter()
                    .any(|e| &e.from == source && &e.to == node_id)
                {
                    return Err(CompileError::UnsatisfiableJoin {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: format!("declared source `{source}` never feeds the join"),
                    });
                }
            }
            for edge in graph.edges.iter().filter(|e| &e.to == node_id) {
                if !sources.contains(edge.from.as_str()) {
                    return Err(CompileError::UnsatisfiableJoin {
                        workflow_id: wf.to_string(),
                        node_id: node_id.clone(),
                        detail: format!("feeder `{}` is not among the declared sources", edge.from),
                    });
                }
            }
            self.check_join_policy(graph, node_id, join, &sources)?;
        }
        Ok(())
    }

    /// §10.5 "join dont les branches et la politique sont incohérentes":
    /// pairwise exclusivity of sources through per-label domination of the
    /// exclusive-choice nodes (`switch`, `classifier`, `human.review`). A
    /// source strictly dominated by label A and another strictly dominated
    /// by label B of the SAME chooser can never both arrive.
    ///
    /// Stated bound (Codex r2): the analysis is SINGLE-LEVEL — it models
    /// each chooser independently and does not propagate satisfiability
    /// through compositions of intermediate joins (an `all`-join network
    /// whose combinations starve a downstream quorum can slip through).
    /// This is a LIVENESS approximation, not an authority hole: an
    /// unsatisfiable join blocks the run at runtime (Phase J re-observes
    /// arrivals); it never grants anything.
    fn check_join_policy(
        &self,
        graph: &SbfbWorkflowGraph,
        join_id: &str,
        join: &crate::graph::JoinNode,
        sources: &BTreeSet<&str>,
    ) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        let quorum: Option<NonZeroU32> = match &join.policy {
            crate::graph::JoinPolicy::All(_) => None,
            // `any` is satisfied by a single arrival: exclusive branches are
            // exactly its intended use.
            crate::graph::JoinPolicy::Any(_) => return Ok(()),
            crate::graph::JoinPolicy::Quorum(q) => Some(q.quorum),
        };
        if let Some(q) = quorum
            && q.get() as usize > sources.len()
        {
            return Err(CompileError::UnsatisfiableJoin {
                workflow_id: wf.to_string(),
                node_id: join_id.to_string(),
                detail: format!(
                    "quorum {} exceeds the {} declared sources",
                    q.get(),
                    sources.len()
                ),
            });
        }

        // Per chooser: group the sources by the label that strictly
        // dominates them. At most one label of a chooser fires per run.
        let full_reach = reachable_from_entrypoints(graph, None);
        let mut lost_per_chooser: usize = 0;
        for (chooser_id, node) in &graph.nodes {
            let is_chooser = matches!(
                node,
                WorkflowNode::Switch(_)
                    | WorkflowNode::Classifier(_)
                    | WorkflowNode::HumanReview(_)
            );
            if !is_chooser || !full_reach.contains(chooser_id.as_str()) {
                continue;
            }
            let ports: BTreeSet<&str> = graph
                .edges
                .iter()
                .filter(|e| &e.from == chooser_id)
                .map(|e| e.from_port.as_str())
                .collect();
            let mut groups: BTreeMap<&str, usize> = BTreeMap::new();
            for port in ports {
                let reach_without =
                    reachable_from_entrypoints(graph, Some((chooser_id.as_str(), port)));
                for source in sources {
                    if full_reach.contains(*source) && !reach_without.contains(*source) {
                        *groups.entry(port).or_default() += 1;
                    }
                }
            }
            let dominated: usize = groups.values().sum();
            let largest = groups.values().copied().max().unwrap_or(0);
            if dominated > largest {
                match quorum {
                    // An `all` join with two sources behind different labels
                    // of one chooser can never open.
                    None => {
                        return Err(CompileError::UnsatisfiableJoin {
                            workflow_id: wf.to_string(),
                            node_id: join_id.to_string(),
                            detail: format!(
                                "policy `all` but sources are split across exclusive labels of `{chooser_id}`"
                            ),
                        });
                    }
                    Some(_) => lost_per_chooser += dominated - largest,
                }
            }
        }
        if let Some(q) = quorum {
            let satisfiable = sources.len().saturating_sub(lost_per_chooser);
            if q.get() as usize > satisfiable {
                return Err(CompileError::UnsatisfiableJoin {
                    workflow_id: wf.to_string(),
                    node_id: join_id.to_string(),
                    detail: format!(
                        "quorum {} exceeds the {} sources satisfiable across exclusive branches",
                        q.get(),
                        satisfiable
                    ),
                });
            }
        }
        Ok(())
    }

    /// §10.5 "node ou sortie inaccessible".
    fn validate_reachability(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let reach = reachable_from_entrypoints(graph, None);
        for node_id in graph.nodes.keys() {
            if !reach.contains(node_id.as_str()) {
                return Err(CompileError::UnreachableNode {
                    workflow_id: graph.workflow_id.clone(),
                    node_id: node_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Freeze §8 rule 3: Kahn's algorithm over the (deduplicated) edges —
    /// ANY cycle is refused; the only legal iteration is the pinned body of
    /// `repeat`/`map`. DAG proven BY COMPILATION.
    fn validate_dag(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let mut indegree: BTreeMap<&str, usize> =
            graph.nodes.keys().map(|k| (k.as_str(), 0)).collect();
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &graph.edges {
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
            *indegree.entry(edge.to.as_str()).or_default() += 1;
        }
        let mut queue: Vec<&str> = indegree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(k, _)| *k)
            .collect();
        let mut visited = 0usize;
        while let Some(current) = queue.pop() {
            visited += 1;
            for next in adjacency.get(current).into_iter().flatten() {
                let d = indegree.get_mut(next).expect("edge endpoints validated");
                *d -= 1;
                if *d == 0 {
                    queue.push(next);
                }
            }
        }
        if visited != graph.nodes.len() {
            let stuck: Vec<&str> = indegree
                .iter()
                .filter(|(_, d)| **d > 0)
                .map(|(k, _)| *k)
                .collect();
            return Err(CompileError::CycleDetected {
                workflow_id: graph.workflow_id.clone(),
                detail: format!("cycle through {}", stuck.join(", ")),
            });
        }
        Ok(())
    }

    /// §10.5 "output non typé" landed as the two-way artefact binding.
    fn validate_artifact_outputs(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        let mut declared: BTreeMap<&str, &crate::graph::ArtifactOutput> = BTreeMap::new();
        for artefact in &graph.artifact_outputs {
            if declared.insert(artefact.name.as_str(), artefact).is_some() {
                return Err(CompileError::OutputNotDeclared {
                    workflow_id: wf.to_string(),
                    detail: format!("artefact `{}` is declared twice", artefact.name),
                });
            }
        }
        let mut published: BTreeSet<&str> = BTreeSet::new();
        for (node_id, node) in &graph.nodes {
            let WorkflowNode::Output(output) = node else {
                continue;
            };
            published.insert(output.artifact_name.as_str());
            let Some(artefact) = declared.get(output.artifact_name.as_str()) else {
                return Err(CompileError::OutputNotDeclared {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "output node `{node_id}` publishes `{}`, absent from artifact_outputs",
                        output.artifact_name
                    ),
                });
            };
            if artefact.from_node != *node_id {
                return Err(CompileError::OutputNotDeclared {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "artefact `{}` declares from_node `{}`, published by `{node_id}`",
                        output.artifact_name, artefact.from_node
                    ),
                });
            }
            if artefact.schema_ref != output.schema_ref {
                return Err(CompileError::OutputNotDeclared {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "artefact `{}` and its output node pin different schemas",
                        output.artifact_name
                    ),
                });
            }
        }
        for (name, artefact) in &declared {
            if !published.contains(name) {
                return Err(CompileError::OutputNotDeclared {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "artefact `{name}` names from_node `{}` but no output node publishes it",
                        artefact.from_node
                    ),
                });
            }
        }
        Ok(())
    }

    /// Freeze §7 port compatibility, nominal by digest: across one edge, a
    /// schema id shared by the producer's declared output and the consumer's
    /// declared input must pin the SAME digest. Heterogeneous ids are not
    /// judged in v1 (no structural subtyping — assumed scope cut, tested);
    /// the same-id-different-digest case is the drift/poisoning one.
    /// (The plan-wide one-digest-per-schema-id rule of [`Self::check_schema_ref`]
    /// subsumes most violations; this pass reports the EDGE, which is the
    /// actionable diagnostic.)
    fn validate_port_types(&mut self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        for edge in &graph.edges {
            let producer = graph.nodes.get(&edge.from).expect("edges validated");
            let consumer = graph.nodes.get(&edge.to).expect("edges validated");
            let out_schema = declared_output_schema(producer, &self.agents_parsed);
            let in_schema = declared_input_schema(consumer, &self.agents_parsed);
            if let (Some(out_ref), Some(in_ref)) = (out_schema, in_schema)
                && out_ref.schema_id == in_ref.schema_id
                && out_ref.blake3 != in_ref.blake3
            {
                return Err(CompileError::PortTypeMismatch {
                    workflow_id: wf.to_string(),
                    detail: format!(
                        "edge {}:{} → {}:{} pins schema `{}` under two digests",
                        edge.from, edge.from_port, edge.to, edge.to_port, out_ref.schema_id
                    ),
                });
            }
        }
        Ok(())
    }

    /// §10.5 "publication sans provenance / HITL / évaluation requise" —
    /// the HITL half: every publication-class capability call is dominated
    /// by the set of NON-MODEL gates — `human.review`, or the
    /// host-delivered external `wait.signal` (babel publishes behind the
    /// community-review signal, the pattern the freeze §8.6 scoping
    /// ratifies) — i.e. unreachable once those gates are removed.
    /// Provenance and the evaluation digest are REQUIRED pack fields
    /// (Phase F parse).
    fn validate_hitl_domination(&self, graph: &SbfbWorkflowGraph) -> Result<(), CompileError> {
        let human_gates: BTreeSet<&str> = graph
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n,
                    WorkflowNode::HumanReview(_) | WorkflowNode::WaitSignal(_)
                )
            })
            .map(|(id, _)| id.as_str())
            .collect();
        for (node_id, node) in &graph.nodes {
            let WorkflowNode::CapabilityCall(call) = node else {
                continue;
            };
            if !is_publication_capability(&call.capability_ref) {
                continue;
            }
            let reach_without_gates = reachable_avoiding(graph, &human_gates);
            if reach_without_gates.contains(node_id.as_str()) {
                return Err(CompileError::PublicationWithoutHumanGate {
                    workflow_id: graph.workflow_id.clone(),
                    node_id: node_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Freeze §8 rule 6, by data-flow ANALYSIS over the existing typed ports
    /// (never a new schema field): a `switch` discriminating on a
    /// classification field, or a `classifier` proposing classification
    /// labels, with an effect (`capability.call`) depending on the route,
    /// is refused unless the discriminant's provenance is deterministic or
    /// human. Model outputs taint; `human.review` launders; `map`/`repeat`/
    /// `subworkflow` outputs are OPAQUE and taint fail-closed (documented
    /// v1 conservatism).
    fn validate_classification_provenance(
        &self,
        graph: &SbfbWorkflowGraph,
    ) -> Result<(), CompileError> {
        let wf = graph.workflow_id.as_str();
        for (node_id, node) in &graph.nodes {
            match node {
                WorkflowNode::Switch(switch) => {
                    let field = switch.discriminant_field.to_ascii_lowercase();
                    if !CLASSIFICATION_DISCRIMINANTS.contains(&field.as_str()) {
                        continue;
                    }
                    if !effect_depends_on(graph, node_id) {
                        continue;
                    }
                    if field_taints_from_model(
                        graph,
                        node_id,
                        &switch.discriminant_field,
                        &mut BTreeSet::new(),
                    ) {
                        return Err(CompileError::ClassificationFromModel {
                            workflow_id: wf.to_string(),
                            node_id: node_id.clone(),
                            detail: format!(
                                "discriminant `{}` traces to a model output",
                                switch.discriminant_field
                            ),
                        });
                    }
                }
                WorkflowNode::Classifier(classifier) => {
                    let has_classification_label = classifier.branch_labels.iter().any(|l| {
                        CLASSIFICATION_LABEL_TERMS.contains(&l.to_ascii_lowercase().as_str())
                    });
                    if has_classification_label && effect_depends_on(graph, node_id) {
                        // The classifier's proposal IS a model output.
                        return Err(CompileError::ClassificationFromModel {
                            workflow_id: wf.to_string(),
                            node_id: node_id.clone(),
                            detail: "classifier proposes classification labels with an effect downstream"
                                .to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// The declared output schema of a node, when the format declares one
/// (routing/join/loop outputs are polymorphic pass-throughs in v1).
fn declared_output_schema<'a>(
    node: &'a WorkflowNode,
    agents: &'a BTreeMap<String, (String, SbfbAgentDefinition)>,
) -> Option<&'a SchemaRef> {
    match node {
        WorkflowNode::Input(n) => Some(&n.schema_ref),
        WorkflowNode::Transform(n) => Some(&n.output_schema_ref),
        WorkflowNode::ModelInvoke(n) => Some(&n.output_schema_ref),
        WorkflowNode::AgentInvoke(n) => agents.get(&n.agent_id).map(|(_, a)| &a.output_schema_ref),
        WorkflowNode::AgentAsTool(n) => Some(&n.output_schema_ref),
        WorkflowNode::Handoff(n) => Some(&n.payload_schema_ref),
        WorkflowNode::CapabilityCall(n) => Some(&n.output_schema_ref),
        WorkflowNode::KnowledgeQuery(n) => Some(&n.output_schema_ref),
        WorkflowNode::HumanReview(n) => Some(&n.payload_schema_ref),
        WorkflowNode::WaitSignal(n) => Some(&n.payload_schema_ref),
        _ => None,
    }
}

/// The declared input schema of a node, when the format declares one.
fn declared_input_schema<'a>(
    node: &'a WorkflowNode,
    agents: &'a BTreeMap<String, (String, SbfbAgentDefinition)>,
) -> Option<&'a SchemaRef> {
    match node {
        WorkflowNode::Transform(n) => Some(&n.input_schema_ref),
        WorkflowNode::ModelInvoke(n) => Some(&n.input_schema_ref),
        WorkflowNode::AgentInvoke(n) => agents.get(&n.agent_id).map(|(_, a)| &a.input_schema_ref),
        WorkflowNode::AgentAsTool(n) => Some(&n.input_schema_ref),
        WorkflowNode::Handoff(n) => Some(&n.payload_schema_ref),
        WorkflowNode::CapabilityCall(n) => Some(&n.args_schema_ref),
        WorkflowNode::KnowledgeQuery(n) => Some(&n.query_schema_ref),
        WorkflowNode::HumanReview(n) => Some(&n.payload_schema_ref),
        WorkflowNode::Output(n) => Some(&n.schema_ref),
        _ => None,
    }
}

/// Nodes reachable from the entrypoints, optionally with ONE chooser port's
/// edges removed (the per-label domination probe of the join checks).
fn reachable_from_entrypoints<'a>(
    graph: &'a SbfbWorkflowGraph,
    without_port: Option<(&str, &str)>,
) -> BTreeSet<&'a str> {
    let mut queue: Vec<&str> = graph
        .entrypoints
        .iter()
        .map(String::as_str)
        .filter(|e| graph.nodes.contains_key(*e))
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        for edge in graph.edges.iter().filter(|e| e.from == current) {
            if let Some((node, port)) = without_port
                && edge.from == node
                && edge.from_port == port
            {
                continue;
            }
            queue.push(edge.to.as_str());
        }
    }
    seen
}

/// Nodes reachable from the entrypoints without ever ENTERING one of the
/// `avoided` nodes (the human-gate domination probe).
fn reachable_avoiding<'a>(
    graph: &'a SbfbWorkflowGraph,
    avoided: &BTreeSet<&str>,
) -> BTreeSet<&'a str> {
    let mut queue: Vec<&str> = graph
        .entrypoints
        .iter()
        .map(String::as_str)
        .filter(|e| graph.nodes.contains_key(*e) && !avoided.contains(*e))
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        for edge in graph.edges.iter().filter(|e| e.from == current) {
            if !avoided.contains(edge.to.as_str()) {
                queue.push(edge.to.as_str());
            }
        }
    }
    seen
}

/// The `agent.invoke` nodes whose flow reaches `node_id` TRANSITIVELY
/// through non-agent nodes (backward BFS; the walk does not continue PAST an
/// `agent.invoke` — that agent is the responsible giver on its path).
///
/// CONTRACT (Codex r3, statuted): `agent.as_tool` is NOT a giver — the
/// specialist RETURNS to the parent flow (D4: the parent keeps synthesis
/// and guardrails), so responsibility for a downstream handoff lies with
/// the parent's own orchestrating `agent.invoke` (if any), never with the
/// tool-called specialist; the walk passes THROUGH `agent.as_tool` nodes
/// looking for that orchestrator. Pinned by
/// `handoff_giver_contract_stops_at_agent_invoke`.
fn upstream_agent_invokers(graph: &SbfbWorkflowGraph, node_id: &str) -> Vec<String> {
    let mut queue: Vec<&str> = graph
        .edges
        .iter()
        .filter(|e| e.to == node_id)
        .map(|e| e.from.as_str())
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut givers: BTreeSet<String> = BTreeSet::new();
    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        if matches!(graph.nodes.get(current), Some(WorkflowNode::AgentInvoke(_))) {
            givers.insert(current.to_string());
            continue;
        }
        for edge in graph.edges.iter().filter(|e| e.to == current) {
            queue.push(edge.from.as_str());
        }
    }
    givers.into_iter().collect()
}

/// Does any effect depend on this node's route downstream?
///
/// Deliberately BROADER than the freeze §8.6 letter ("un capability.call
/// d'egress ou de publication") on two axes (fail-closed; the typed
/// capability classes now exist engine-side [§19.3] but this static
/// recognition stays broad-by-design — the runtime WILL refine by type once
/// bound into dispatch): v1 treats
/// EVERY `capability.call` as an effect, and treats a downstream
/// `map`/`repeat`/`subworkflow` as one too (Codex r2 — the child scope is
/// expanded and validated on its own, but a PARENT-scope classification
/// switch routing into an effectful child would otherwise escape this
/// scope-local walk). May over-refuse a model→classification→non-egress
/// chain; that conservatism is the intended v1 posture.
fn effect_depends_on(graph: &SbfbWorkflowGraph, node_id: &str) -> bool {
    let mut queue: Vec<&str> = vec![node_id];
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current != node_id
            && matches!(
                graph.nodes.get(current),
                Some(
                    WorkflowNode::CapabilityCall(_)
                        | WorkflowNode::Map(_)
                        | WorkflowNode::Repeat(_)
                        | WorkflowNode::Subworkflow(_)
                )
            )
        {
            return true;
        }
        for edge in graph.edges.iter().filter(|e| e.from == current) {
            queue.push(edge.to.as_str());
        }
    }
    false
}

/// Is the provenance of `field` at `node_id` (through its in-edges) possibly
/// a model output? Fail-closed: what cannot be PROVEN deterministic/human
/// taints. `visiting` guards re-entry (the DAG is already proven, but the
/// guard keeps this function total on any input).
fn field_taints_from_model(
    graph: &SbfbWorkflowGraph,
    node_id: &str,
    field: &str,
    visiting: &mut BTreeSet<(String, String)>,
) -> bool {
    if !visiting.insert((node_id.to_string(), field.to_string())) {
        return false;
    }
    let mut has_producer = false;
    for edge in graph.edges.iter().filter(|e| e.to == node_id) {
        has_producer = true;
        let Some(producer) = graph.nodes.get(&edge.from) else {
            return true;
        };
        let tainted = match producer {
            // Model outputs taint.
            WorkflowNode::ModelInvoke(_)
            | WorkflowNode::Classifier(_)
            | WorkflowNode::AgentInvoke(_)
            | WorkflowNode::AgentAsTool(_) => true,
            // Deterministic or human sources launder.
            WorkflowNode::Input(_) | WorkflowNode::HumanReview(_) | WorkflowNode::WaitSignal(_) => {
                false
            }
            // A capability result CAN be a function of its model-fed
            // arguments (an echo-shaped activity would relay the model's
            // classification verbatim), and a knowledge-query RESULT is a
            // model-steerable SELECTION even over a pinned corpus (Codex
            // r3) — neither launders: recurse.
            WorkflowNode::CapabilityCall(_) | WorkflowNode::KnowledgeQuery(_) => {
                field_taints_from_model(graph, &edge.from, field, visiting)
            }
            // Declarative mapping: follow the field upstream; a field the
            // map does not produce has UNKNOWN provenance — taint.
            WorkflowNode::Transform(t) => match t.field_map.get(field) {
                Some(source_field) => {
                    field_taints_from_model(graph, &edge.from, source_field, visiting)
                }
                None => true,
            },
            // Pass-throughs: recurse on the same field.
            WorkflowNode::Switch(_)
            | WorkflowNode::Parallel(_)
            | WorkflowNode::Join(_)
            | WorkflowNode::Handoff(_) => {
                field_taints_from_model(graph, &edge.from, field, visiting)
            }
            // Opaque child outputs: fail-closed.
            WorkflowNode::Map(_) | WorkflowNode::Repeat(_) | WorkflowNode::Subworkflow(_) => true,
            WorkflowNode::Output(_) => true,
        };
        if tainted {
            return true;
        }
    }
    // No producer at all: the payload predates the graph (entry data) —
    // deterministic.
    let _ = has_producer;
    false
}

/// Canonical edge order: `(from, from_port, to, to_port)`.
fn edge_sort_key(edge: &WorkflowEdge) -> (String, String, String, String) {
    (
        edge.from.clone(),
        edge.from_port.clone(),
        edge.to.clone(),
        edge.to_port.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_store::{
        ArtifactStore, GOLDEN_NAMES, golden_path, parse_json, src_dir, write_pretty,
    };
    use crate::pack::{
        PACK_FORMAT_VERSION, PackDigestSet, PackDigests, PackLock, PackLockEntry, PackProvenance,
        PackSubject, blake3_hex,
    };

    /// The golden store categories and their file extensions, mirrored from
    /// the Phase G pin helpers.
    const STORE_CATEGORIES: [(&str, &str, &str); 7] = [
        ("agents", ".agent.json", "agent.json"),
        ("subworkflows", ".workflow.json", "workflow.json"),
        ("schemas", ".schema.json", "schema.json"),
        ("prompts", ".md", "md"),
        ("tools", ".md", "md"),
        ("corpora", ".md", "md"),
        ("guardrails", ".md", "md"),
    ];

    fn canonical_pin(path: &std::path::Path) -> (String, u64) {
        let bytes =
            std::fs::read(path).unwrap_or_else(|e| panic!("{} must exist ({e})", path.display()));
        let canonical = crate::factory_profile::canonicalise(&bytes);
        (
            blake3::hash(&canonical).to_hex().to_string(),
            canonical.len() as u64,
        )
    }

    /// Wrap one committed golden workflow in a deterministic synthetic pack:
    /// the lock pins every store file plus the root document, the manifest
    /// requests the union of every workflow's capability requests, and the
    /// `content` digest is computed from the lock exactly like a real pack's.
    fn golden_input(name: &str) -> CompileInput {
        let store = ArtifactStore::open(name);
        let mut entries: Vec<PackLockEntry> = Vec::new();

        let root_path = golden_path(name);
        let (root_pin, root_size) = canonical_pin(&root_path);
        entries.push(PackLockEntry {
            path: format!("workflows/{name}.workflow.json"),
            blake3: root_pin.clone(),
            size_bytes: root_size,
        });
        for (category, suffix, ext) in STORE_CATEGORIES {
            for id in store.ids(category, suffix) {
                let (pin, size) = canonical_pin(&store.file(category, &id, ext));
                entries.push(PackLockEntry {
                    path: format!("{category}/{id}.{ext}"),
                    blake3: pin,
                    size_bytes: size,
                });
            }
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        let lock = PackLock { entries };
        let content = crate::pack::content_digest_of_lock(&lock).expect("lock digests");

        let canonical_bytes_of = |path: &std::path::Path| -> Vec<u8> {
            let bytes = std::fs::read(path)
                .unwrap_or_else(|e| panic!("{} must exist ({e})", path.display()));
            crate::factory_profile::canonicalise(&bytes)
        };
        let root_bytes = canonical_bytes_of(&root_path);
        let root_value = parse_json(&root_path);
        let mut capabilities: BTreeSet<String> = BTreeSet::new();
        let mut collect_caps = |doc: &serde_json::Value| {
            for cap in doc["capability_requests"].as_array().into_iter().flatten() {
                capabilities.insert(cap.as_str().expect("capability is a string").to_string());
            }
        };
        collect_caps(&root_value);
        let mut subworkflows = BTreeMap::new();
        for id in store.ids("subworkflows", ".workflow.json") {
            let path = store.file("subworkflows", &id, "workflow.json");
            collect_caps(&parse_json(&path));
            subworkflows.insert(
                id.clone(),
                ProvidedDocument {
                    bytes: canonical_bytes_of(&path),
                },
            );
        }
        let mut agents = BTreeMap::new();
        for id in store.ids("agents", ".agent.json") {
            agents.insert(
                id.clone(),
                ProvidedDocument {
                    bytes: canonical_bytes_of(&store.file("agents", &id, "agent.json")),
                },
            );
        }

        CompileInput {
            pack: SbfbPromptAgentPack {
                format_version: PACK_FORMAT_VERSION,
                pack_revision: 1,
                name: format!("golden-{name}"),
                description: "Test pack wrapping one committed golden workflow.".to_string(),
                digests: PackDigests {
                    content: content.clone(),
                    graph_semantic: blake3_hex(b"graph_semantic placeholder"),
                    presentation: blake3_hex(b"presentation placeholder"),
                    compiled_plan: blake3_hex(b"compiled_plan placeholder"),
                    evaluation: blake3_hex(b"evaluation placeholder"),
                    publication: blake3_hex(b"publication placeholder"),
                },
                lock,
                provenance: PackProvenance {
                    subject: vec![PackSubject {
                        name: format!("golden-{name}"),
                        digest: PackDigestSet { blake3: content },
                    }],
                    builder_id: "sbfb-workflow-core/compile-tests".to_string(),
                },
                requested_capabilities: capabilities.into_iter().collect(),
                secret_refs: Vec::new(),
            },
            root_workflow: ProvidedDocument { bytes: root_bytes },
            subworkflows,
            agents,
        }
    }

    /// Parse the root workflow document of an input back to a Value.
    fn root_value(input: &CompileInput) -> serde_json::Value {
        serde_json::from_slice(&input.root_workflow.bytes).expect("root document is JSON")
    }

    /// Replace a lock entry `old_pin` with the pin of `new_bytes` and
    /// recompute the content claim.
    fn reseal_lock(input: &mut CompileInput, old_pin: &str, new_bytes: &[u8]) {
        let new_pin = blake3_hex(new_bytes);
        for entry in &mut input.pack.lock.entries {
            if entry.blake3 == old_pin {
                entry.blake3 = new_pin.clone();
                entry.size_bytes = new_bytes.len() as u64;
            }
        }
        input.pack.digests.content =
            crate::pack::content_digest_of_lock(&input.pack.lock).expect("lock digests");
    }

    /// Mutate the ROOT document of a built input and re-seal the binding
    /// chain (bytes → pin → lock → content): the mutation under test stays
    /// the ONLY broken invariant.
    fn remint_root(input: &mut CompileInput, mutate: impl FnOnce(&mut serde_json::Value)) {
        let old_pin = input.root_workflow.pin();
        let mut value = root_value(input);
        mutate(&mut value);
        let bytes = serde_json::to_vec_pretty(&value).expect("root serialises");
        reseal_lock(input, &old_pin, &bytes);
        input.root_workflow.bytes = bytes;
    }

    /// Mutate one AGENT document and re-seal the chain, including the
    /// root's `agent_refs` pin for that agent.
    fn remint_agent(
        input: &mut CompileInput,
        agent_id: &str,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) {
        let provided = input.agents.get(agent_id).expect("agent provided");
        let old_pin = provided.pin();
        let mut value: serde_json::Value =
            serde_json::from_slice(&provided.bytes).expect("agent document is JSON");
        mutate(&mut value);
        let bytes = serde_json::to_vec_pretty(&value).expect("agent serialises");
        let new_pin = blake3_hex(&bytes);
        reseal_lock(input, &old_pin, &bytes);
        input
            .agents
            .get_mut(agent_id)
            .expect("agent provided")
            .bytes = bytes;
        remint_root(input, |root| {
            for r in root["agent_refs"]
                .as_array_mut()
                .expect("agent_refs")
                .iter_mut()
            {
                if r["agent_id"] == agent_id {
                    r["blake3"] = serde_json::json!(new_pin.clone());
                }
            }
        });
    }

    /// Mutate one SUBWORKFLOW document and re-seal the chain, including the
    /// root's `subworkflow_refs` and expanding-node pins for it.
    fn remint_subworkflow(
        input: &mut CompileInput,
        workflow_id: &str,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) {
        let provided = input.subworkflows.get(workflow_id).expect("child provided");
        let old_pin = provided.pin();
        let mut value: serde_json::Value =
            serde_json::from_slice(&provided.bytes).expect("child document is JSON");
        mutate(&mut value);
        let bytes = serde_json::to_vec_pretty(&value).expect("child serialises");
        let new_pin = blake3_hex(&bytes);
        reseal_lock(input, &old_pin, &bytes);
        input
            .subworkflows
            .get_mut(workflow_id)
            .expect("child provided")
            .bytes = bytes;
        remint_root(input, |root| {
            for r in root["subworkflow_refs"]
                .as_array_mut()
                .expect("subworkflow_refs")
                .iter_mut()
            {
                if r["workflow_id"] == workflow_id {
                    r["blake3"] = serde_json::json!(new_pin.clone());
                }
            }
            for node in root["nodes"].as_object_mut().expect("nodes").values_mut() {
                for key in ["body_subworkflow_ref", "subworkflow_ref"] {
                    if node[key]["workflow_id"] == workflow_id {
                        node[key]["blake3"] = serde_json::json!(new_pin.clone());
                    }
                }
            }
        });
    }

    /// A deterministic fake pin for synthetic graphs.
    fn pin_for(tag: &str) -> String {
        blake3_hex(format!("synthetic:{tag}").as_bytes())
    }

    fn sref(id: &str) -> serde_json::Value {
        serde_json::json!({ "schema_id": id, "blake3": pin_for(&format!("schema:{id}")) })
    }

    fn big_budget() -> serde_json::Value {
        serde_json::json!({
            "max_total_tokens": 1000000u64,
            "max_cost_micro_usd": 1000000u64,
            "max_wall_clock_seconds": 3600u64,
            "max_turns": 100u32
        })
    }

    /// A minimal valid workflow document: `start` (input) → `do` (capability
    /// call) → `done` (output). Tests mutate it into each refusal.
    fn mini_graph() -> serde_json::Value {
        serde_json::json!({
            "workflow_id": "mini",
            "workflow_version": 1,
            "description": "Minimal synthetic workflow.",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "entrypoints": ["start"],
            "nodes": {
                "start": { "kind": "input", "schema_ref": sref("payload") },
                "do": {
                    "kind": "capability.call",
                    "capability_ref": "cap.run",
                    "args_schema_ref": sref("payload"),
                    "output_schema_ref": sref("payload")
                },
                "done": {
                    "kind": "output",
                    "schema_ref": sref("payload"),
                    "artifact_name": "result"
                }
            },
            "edges": [
                { "from": "start", "from_port": "out", "to": "do", "to_port": "in" },
                { "from": "do", "from_port": "out", "to": "done", "to_port": "in" }
            ],
            "error_policy": { "on_node_failure": "fail-workflow", "max_retries_per_node": 0 },
            "timeout_policy": { "workflow_timeout_seconds": 60, "default_node_timeout_seconds": 30 },
            "budget_policy": big_budget(),
            "cancellation_policy": {
                "cancel_on_budget_exhausted": true,
                "cancel_on_timeout": true,
                "allow_host_operator_cancel": true
            },
            "capability_requests": ["cap.run"],
            "agent_refs": [],
            "subworkflow_refs": [],
            "artifact_outputs": [{
                "name": "result",
                "schema_ref": sref("payload"),
                "from_node": "done"
            }]
        })
    }

    /// Collect every `"blake3"` string of a document tree (the pins its
    /// refs claim), so the synthetic lock can pin them all.
    fn collect_pins(value: &serde_json::Value, out: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::Object(members) => {
                for (key, member) in members {
                    if key == "blake3"
                        && let Some(pin) = member.as_str()
                    {
                        out.insert(pin.to_string());
                    }
                    collect_pins(member, out);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    collect_pins(item, out);
                }
            }
            _ => {}
        }
    }

    /// Rewrite every `{agent_id|workflow_id, blake3}` reference pair whose
    /// id is in `pins` to the REAL derived pin of that document (the
    /// bytes-binding compiler refuses fantasy pins).
    fn patch_pins(value: &mut serde_json::Value, pins: &BTreeMap<String, String>) {
        match value {
            serde_json::Value::Object(members) => {
                let target = ["agent_id", "workflow_id"].iter().find_map(|k| {
                    members
                        .get(*k)
                        .and_then(|v| v.as_str())
                        .filter(|id| pins.contains_key(*id))
                        .map(str::to_string)
                });
                if let (Some(id), true) = (target, members.contains_key("blake3")) {
                    members.insert("blake3".into(), serde_json::json!(pins[&id].clone()));
                }
                for member in members.values_mut() {
                    patch_pins(member, pins);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    patch_pins(item, pins);
                }
            }
            _ => {}
        }
    }

    /// Wrap synthetic documents in a self-consistent pack: children are
    /// serialised leaf-first (callers pass dependencies before dependents),
    /// references are patched to the REAL derived pins, and the lock pins
    /// every document plus every leaf pin the documents claim.
    fn synth_input(
        root: serde_json::Value,
        subworkflows: Vec<(&str, serde_json::Value)>,
        agents: Vec<(&str, serde_json::Value)>,
        capabilities: &[&str],
    ) -> CompileInput {
        let mut real_pins: BTreeMap<String, String> = BTreeMap::new();
        let mut agent_docs = BTreeMap::new();
        for (id, doc) in agents {
            let bytes = serde_json::to_vec_pretty(&doc).expect("agent serialises");
            real_pins.insert(id.to_string(), blake3_hex(&bytes));
            agent_docs.insert(id.to_string(), ProvidedDocument { bytes });
        }
        let mut sub_docs = BTreeMap::new();
        let mut patched_subs: Vec<serde_json::Value> = Vec::new();
        for (id, mut doc) in subworkflows {
            patch_pins(&mut doc, &real_pins);
            let bytes = serde_json::to_vec_pretty(&doc).expect("child serialises");
            real_pins.insert(id.to_string(), blake3_hex(&bytes));
            patched_subs.push(doc);
            sub_docs.insert(id.to_string(), ProvidedDocument { bytes });
        }
        let mut root = root;
        patch_pins(&mut root, &real_pins);
        let root_bytes = serde_json::to_vec_pretty(&root).expect("root serialises");

        let mut pins: BTreeSet<String> = BTreeSet::new();
        collect_pins(&root, &mut pins);
        for doc in &patched_subs {
            collect_pins(doc, &mut pins);
        }
        for provided in agent_docs.values() {
            let doc: serde_json::Value =
                serde_json::from_slice(&provided.bytes).expect("agent is JSON");
            collect_pins(&doc, &mut pins);
        }
        pins.insert(blake3_hex(&root_bytes));
        for provided in sub_docs.values().chain(agent_docs.values()) {
            pins.insert(provided.pin());
        }
        let entries: Vec<PackLockEntry> = pins
            .iter()
            .map(|pin| PackLockEntry {
                path: format!("blob/{pin}"),
                blake3: pin.clone(),
                size_bytes: 1,
            })
            .collect();
        let lock = PackLock { entries };
        let content = crate::pack::content_digest_of_lock(&lock).expect("lock digests");
        CompileInput {
            pack: SbfbPromptAgentPack {
                format_version: PACK_FORMAT_VERSION,
                pack_revision: 1,
                name: "synthetic".to_string(),
                description: "Synthetic test pack.".to_string(),
                digests: PackDigests {
                    content: content.clone(),
                    graph_semantic: blake3_hex(b"gs"),
                    presentation: blake3_hex(b"pr"),
                    compiled_plan: blake3_hex(b"cp"),
                    evaluation: blake3_hex(b"ev"),
                    publication: blake3_hex(b"pu"),
                },
                lock,
                provenance: PackProvenance {
                    subject: vec![PackSubject {
                        name: "synthetic".to_string(),
                        digest: PackDigestSet { blake3: content },
                    }],
                    builder_id: "sbfb-workflow-core/compile-tests".to_string(),
                },
                requested_capabilities: capabilities.iter().map(|c| c.to_string()).collect(),
                secret_refs: Vec::new(),
            },
            root_workflow: ProvidedDocument { bytes: root_bytes },
            subworkflows: sub_docs,
            agents: agent_docs,
        }
    }

    fn mini_input(root: serde_json::Value) -> CompileInput {
        synth_input(root, Vec::new(), Vec::new(), &["cap.run"])
    }

    /// H.4 exit condition: the four goldens compile completely, Factory
    /// FIRST — plus the red arm that keeps the test non-vacuous (a mutated
    /// pack/agent binding must be REFUSED, patron `golden_negative_controls`).
    #[test]
    fn four_goldens_compile() {
        for name in GOLDEN_NAMES {
            let input = golden_input(name);
            let plan =
                compile(&input).unwrap_or_else(|e| panic!("{name} golden must compile: {e}"));
            assert_eq!(plan.root_workflow_id, {
                let (_, g) = crate::golden_store::parse_graph(&golden_path(name));
                g.workflow_id
            });
            assert!(
                plan.node_instances.len()
                    >= root_value(&input)["nodes"]
                        .as_object()
                        .expect("nodes")
                        .len(),
                "{name}: the plan must expand at least the root nodes"
            );
            assert!(
                !plan.capability_closure.is_empty(),
                "{name}: golden richness guarantees >= 1 capability.call"
            );
            assert_eq!(
                plan.pack_digest,
                crate::pack::content_digest_of_lock(&input.pack.lock).unwrap(),
                "{name}: the plan digest binding is the RECOMPUTED content digest"
            );
            assert_eq!(plan.compiler.version, COMPILER_VERSION);
            assert_eq!(plan.compiler.profile, COMPILER_PROFILE_STRICT_V1);
            assert!(
                plan.entrypoints.contains_key(""),
                "{name}: the root scope must declare its entrypoints"
            );
        }

        // Red arm 1: a budget mutated ABOVE its workflow ceiling is refused.
        let mut hostile = golden_input("factory");
        remint_root(&mut hostile, |v| {
            v["nodes"]["drive-implementation"]["budget"]["max_total_tokens"] =
                serde_json::json!(100_000_000_000u64);
        });
        match compile(&hostile) {
            Err(CompileError::BudgetOverflow { dimension, .. }) => {
                assert_eq!(dimension, "max_total_tokens");
            }
            other => panic!("mutated budget must be refused, got {other:?}"),
        }

        // Red arm 2: an agent whose provided pin no longer matches the
        // graph's agent_ref is refused.
        let mut hostile = golden_input("factory");
        hostile
            .agents
            .get_mut("factory-driver")
            .expect("factory-driver provided")
            .bytes = b"{\"substituted\": true}".to_vec();
        assert!(
            matches!(
                compile(&hostile),
                Err(CompileError::UnpinnedReference { .. })
            ),
            "a substituted agent document must be refused"
        );
    }

    /// S1a-05: `compile` is a pure deterministic function — two runs over
    /// one input yield byte-identical plans and one digest. Honest scope
    /// (review B-5): THIS test proves intra-run/intra-profile stability
    /// only; the cross-profile guarantee (`-p` vs `--workspace`, the
    /// `preserve_order` hazard) is carried by the deterministic mechanics
    /// (BTreeMap/sorted Vec/JCS, module doc) plus
    /// `golden_compiled_plans_stable` comparing committed fixtures as
    /// `serde_json::Value` under both build graphs in CI.
    #[test]
    fn compile_is_deterministic() {
        let input = golden_input("factory");
        let a = compile(&input).expect("compiles");
        let b = compile(&input).expect("compiles");
        assert_eq!(a, b, "two compilations must be identical");
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap(),
            "and their serialisations byte-identical"
        );
        assert_eq!(
            compiled_plan_digest(&a).unwrap(),
            compiled_plan_digest(&b).unwrap()
        );
    }

    /// S1a-02 (scoped) / NEUF-01: edge ORDER is presentation of the source
    /// document, not meaning — the compiler normalises it, so two authoring
    /// orders collapse to ONE semantic digest and ONE plan (the WA-T1.2
    /// foundation at the compile level).
    #[test]
    fn graph_semantic_digest_order_independent() {
        let input = golden_input("factory");
        let plan = compile(&input).expect("compiles");

        let mut reordered = input.clone();
        remint_root(&mut reordered, |v| {
            v["edges"].as_array_mut().expect("edges array").reverse();
        });
        let plan_reordered = compile(&reordered).expect("still compiles");

        assert_eq!(
            plan.graph_semantic_digest, plan_reordered.graph_semantic_digest,
            "edge order must not move the semantic digest"
        );
        // The reordered BYTES are honestly a different pack (different lock,
        // different content digest, different ROOT document pin) — the plans
        // must be identical UP TO those byte-derived bindings, which is
        // exactly the WA-T1.2 claim: same MEANING, one semantic digest.
        let mut a = serde_json::to_value(&plan).unwrap();
        let mut b = serde_json::to_value(&plan_reordered).unwrap();
        assert_ne!(a["pack_digest"], b["pack_digest"]);
        for v in [&mut a, &mut b] {
            v["pack_digest"] = serde_json::json!("");
            v["artefact_digests"]["workflows"][&plan.root_workflow_id] = serde_json::json!("");
        }
        assert_eq!(a, b, "identical plans up to the byte-derived bindings");
    }

    /// The Plan-Then-Execute invariant, made non-vacuous (S3-05, patron
    /// `golden_negative_controls_reject`):
    ///
    /// (a) `compile` derives from the PACK ONLY — its input type has no
    ///     model-output channel, so a hostile model output cannot move the
    ///     plan: recompiling the same input is byte-stable (arm i), while
    ///     ANY pack change moves the digest (arm ii);
    /// (b) each of the NINE §19.3 families a prompt injection may never
    ///     touch is PINNED BY THE PLAN DIGEST: mutating that family in the
    ///     plan changes the digest, so a runtime holding the digest detects
    ///     it. A family absent from the plan would fail the existence
    ///     assertion — that is the discriminant.
    #[test]
    fn plan_then_execute_invariant() {
        let input = golden_input("factory");
        let plan = compile(&input).expect("compiles");
        let digest = compiled_plan_digest(&plan).unwrap();

        // Arm (i): no model-output channel — the same pack recompiles to
        // the same bytes no matter what any model said meanwhile.
        assert_eq!(
            compiled_plan_digest(&compile(&input).unwrap()).unwrap(),
            digest
        );

        // Arm (ii): the plan is SENSITIVE to the pack fields it DERIVES
        // FROM (revision, name, lock, pinned documents) — a revision bump
        // alone moves the digest. Fields the plan does not project
        // (description, provenance prose, unconsumed digest claims) do NOT
        // move it: their integrity is carried by the pack signature, never
        // by the plan (Codex r3 — the earlier "any pack change" wording
        // overclaimed).
        let mut bumped = input.clone();
        bumped.pack.pack_revision = 2;
        assert_ne!(
            compiled_plan_digest(&compile(&bumped).unwrap()).unwrap(),
            digest,
            "a plan-projected pack change must change the plan"
        );

        // The nine §19.3 families, each mutated at a path that MUST exist.
        // What discriminates here (review B-6): the digest flip itself is
        // trivial (blake3 covers the whole Value) — the load-bearing checks
        // are the `.expect(…)` EXISTENCE guards on each mutation path (a
        // family absent from the plan would panic there) plus the
        // `assert_ne!(mutated, value)` no-op guard.
        type PlanMutation<'a> = (&'a str, Box<dyn Fn(&mut serde_json::Value)>);
        let value = serde_json::to_value(&plan).unwrap();
        let mutations: Vec<PlanMutation> = vec![
            (
                "topology (edges)",
                Box::new(|v| {
                    let e = v["edges"][0].as_object_mut().expect("edge 0 exists");
                    e.insert("to".into(), serde_json::json!("/somewhere-else"));
                }),
            ),
            (
                "tool/artefact digests",
                Box::new(|v| {
                    let prompts = v["artefact_digests"]["prompts"]
                        .as_object_mut()
                        .expect("prompt digests exist");
                    let key = prompts.keys().next().expect("one prompt").clone();
                    prompts.insert(key, serde_json::json!(blake3_hex(b"poisoned prompt")));
                }),
            ),
            (
                "destinations (capability instances)",
                Box::new(|v| {
                    let closure = v["capability_closure"]["process.git_commit"]
                        .as_array_mut()
                        .expect("commit capability closed");
                    closure[0] = serde_json::json!("/somewhere-else");
                }),
            ),
            (
                "grants (capability set)",
                Box::new(|v| {
                    v["capability_closure"]
                        .as_object_mut()
                        .expect("closure")
                        .insert("sneaky.grant".into(), serde_json::json!(["/do"]));
                }),
            ),
            (
                "classification (human decision labels)",
                Box::new(|v| {
                    let labels = v["node_instances"]["/po-ratification"]["node"]["decision_labels"]
                        .as_array_mut()
                        .expect("human gate labels exist");
                    labels.push(serde_json::json!("leak"));
                }),
            ),
            (
                "output schemas",
                Box::new(|v| {
                    let node = v["node_instances"]["/phase-record"]["node"]
                        .as_object_mut()
                        .expect("output node body");
                    node["schema_ref"]["blake3"] = serde_json::json!(blake3_hex(b"swapped schema"));
                }),
            ),
            (
                "HITL gates",
                Box::new(|v| {
                    let gates = v["human_gates"].as_array_mut().expect("human gates");
                    assert!(!gates.is_empty(), "factory pins its human gate");
                    gates.clear();
                }),
            ),
            (
                "budgets and bounds",
                Box::new(|v| {
                    let budget = v["scope_budgets"][""]
                        .as_object_mut()
                        .expect("root scope budget");
                    budget["max_total_tokens"] = serde_json::json!(999_999_999_999u64);
                }),
            ),
            (
                "publisher identity",
                Box::new(|v| {
                    v["pack_name"] = serde_json::json!("someone-else");
                }),
            ),
        ];
        for (family, mutate) in mutations {
            let mut mutated = value.clone();
            mutate(&mut mutated);
            assert_ne!(mutated, value, "{family}: the mutation must land");
            let mutated_digest =
                digest_for(PackDigestKind::CompiledPlan, &mutated).expect("digestable");
            assert_ne!(
                mutated_digest, digest,
                "{family}: the family must be covered by the plan digest"
            );
        }
    }

    /// The committed `*.compiled.golden.json` fixtures (H.2): the exact
    /// plans the four goldens compile to, compared as `serde_json::Value`
    /// (never strings — `preserve_order`). Refresh after an intentional
    /// change with `UPDATE_SNAPSHOTS=1 cargo test --workspace
    /// golden_compiled_plans_stable` (WORKSPACE build, like the schema
    /// snapshot).
    #[test]
    fn golden_compiled_plans_stable() {
        for name in GOLDEN_NAMES {
            let plan = compile(&golden_input(name))
                .unwrap_or_else(|e| panic!("{name} golden must compile: {e}"));
            let value = serde_json::to_value(&plan).expect("plan serialises");
            let path = src_dir().join(format!("{name}.compiled.golden.json"));
            if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
                write_pretty(&path, &value);
                continue;
            }
            let committed = parse_json(&path);
            assert_eq!(
                value, committed,
                "{name}: compiled plan drift — refresh with UPDATE_SNAPSHOTS=1 under the workspace build"
            );
        }
    }

    /// Occurrence identities are a pure function of the structural path
    /// (research §12.2), RFC 6901-encoded: root nodes at `/id`, child
    /// occurrences under their expanding node's path.
    #[test]
    fn node_instance_ids_encode_the_structural_path() {
        assert_eq!(instance_id("", "phase-brief"), "/phase-brief");
        assert_eq!(
            instance_id("/codex-rounds", "codex-verify"),
            "/codex-rounds/codex-verify"
        );
        assert_eq!(
            instance_id("", "a/b~c"),
            "/a~1b~0c",
            "separator and tilde must be RFC 6901-escaped"
        );

        let plan = compile(&golden_input("factory")).expect("compiles");
        assert!(plan.node_instances.contains_key("/phase-brief"));
        assert_eq!(
            plan.node_instances["/codex-rounds"].kind, "repeat",
            "the expanding occurrence itself is an instance"
        );
        assert!(
            plan.node_instances
                .keys()
                .any(|k| k.starts_with("/codex-rounds/")),
            "the repeat body must expand under the expanding node's path"
        );
        assert_eq!(plan.entrypoints[""], vec!["/phase-brief".to_string()]);
        assert!(
            plan.entrypoints.contains_key("/codex-rounds"),
            "the child scope declares its own entry occurrences"
        );
        assert_eq!(
            plan.capability_closure["process.git_commit"],
            vec!["/commit-phase".to_string()]
        );
        assert_eq!(
            plan.capability_closure["process.run_tests"],
            vec!["/run-suites".to_string()]
        );
        assert!(plan.human_gates.contains(&"/po-ratification".to_string()));

        let ideas = compile(&golden_input("ideas")).expect("compiles");
        assert!(ideas.human_gates.contains(&"/publish-decision".to_string()));
    }

    /// Every negative below first proves the unmutated base compiles, so an
    /// error assertion can never pass for the wrong reason (patron
    /// `control_base_parses`, review r1 D2-01 discipline).
    fn control_mini_compiles() {
        compile(&mini_input(mini_graph())).expect("control: the mini workflow compiles");
    }

    fn expect_error(input: &CompileInput, check: impl Fn(&CompileError) -> bool, what: &str) {
        match compile(input) {
            Err(e) if check(&e) => {}
            Err(e) => panic!("{what}: wrong refusal {e:?}"),
            Ok(_) => panic!("{what}: must be refused"),
        }
    }

    /// §10.5 class 1 — "node ou sortie inaccessible".
    #[test]
    fn static_validation_rejects_unreachable_node() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["nodes"]["island"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {}
        });
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::UnreachableNode { node_id, .. } if node_id == "island"),
            "unwired node",
        );
    }

    /// Freeze §8 rule 3 — ANY cycle over arbitrary edges (subsuming the
    /// §10.5 "cycle non porté par une boucle bornée" and the self-loop).
    #[test]
    fn static_validation_rejects_cycle() {
        control_mini_compiles();
        let mut root = mini_graph();
        for (id,) in [("a",), ("b",)] {
            root["nodes"][id] = serde_json::json!({
                "kind": "transform",
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "field_map": {}
            });
        }
        let edges = root["edges"].as_array_mut().unwrap();
        edges.push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "a", "to_port": "in"}),
        );
        edges
            .push(serde_json::json!({"from": "a", "from_port": "out", "to": "b", "to_port": "in"}));
        edges
            .push(serde_json::json!({"from": "b", "from_port": "out", "to": "a", "to_port": "in"}));
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::CycleDetected { .. }),
            "hand-spelled cycle",
        );

        // The self-loop is the same refusal (S1a-07 adjustment).
        let mut root = mini_graph();
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "do", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::CycleDetected { .. }),
            "self-loop",
        );
    }

    /// §10.5 class 3 — "`map` sans limite de cardinalité" is carried by the
    /// strict parse (the bound is a REQUIRED NonZero of the frozen form):
    /// the compiler turns it into a refusal, never a panic or a default.
    #[test]
    fn static_validation_rejects_unbounded_map() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["nodes"]["fan"] = serde_json::json!({
            "kind": "map",
            "collection_field": "items",
            "body_subworkflow_ref": {"workflow_id": "child", "blake3": pin_for("doc:sub:child")}
            // no max_cardinality
        });
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "fan", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::InvalidDocument { detail, .. } if detail.contains("max_cardinality")),
            "map without bound",
        );

        // Zero is refused by the NonZero type for the same reason.
        let mut root = mini_graph();
        root["nodes"]["fan"] = serde_json::json!({
            "kind": "map",
            "collection_field": "items",
            "max_cardinality": 0,
            "body_subworkflow_ref": {"workflow_id": "child", "blake3": pin_for("doc:sub:child")}
        });
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::InvalidDocument { .. }),
            "zero map bound",
        );
    }

    /// §10.5 class 4 — an `all` join fed by mutually-exclusive routing
    /// branches deadlocks a signed plan; `any` over the same topology is
    /// exactly what `any` is for.
    #[test]
    fn static_validation_rejects_incoherent_join() {
        let route = |mode: serde_json::Value| {
            let mut root = mini_graph();
            root["nodes"]["sw"] = serde_json::json!({
                "kind": "switch", "discriminant_field": "route",
                "branch_labels": ["left", "right"]
            });
            for id in ["na", "nb", "nd"] {
                root["nodes"][id] = serde_json::json!({
                    "kind": "transform",
                    "input_schema_ref": sref("payload"),
                    "output_schema_ref": sref("payload"),
                    "field_map": {}
                });
            }
            root["nodes"]["j"] = serde_json::json!({
                "kind": "join", "policy": mode, "source_node_ids": ["na", "nb"]
            });
            root["edges"] = serde_json::json!([
                {"from": "start", "from_port": "out", "to": "sw", "to_port": "in"},
                {"from": "sw", "from_port": "left", "to": "na", "to_port": "in"},
                {"from": "sw", "from_port": "right", "to": "nb", "to_port": "in"},
                {"from": "sw", "from_port": "default", "to": "nd", "to_port": "in"},
                {"from": "na", "from_port": "out", "to": "j", "to_port": "in"},
                {"from": "nb", "from_port": "out", "to": "j", "to_port": "in"},
                {"from": "nd", "from_port": "out", "to": "do", "to_port": "in"},
                {"from": "j", "from_port": "out", "to": "do", "to_port": "in"},
                {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
            ]);
            mini_input(root)
        };
        compile(&route(serde_json::json!({"mode": "any"})))
            .expect("control: `any` over exclusive branches is legal");
        expect_error(
            &route(serde_json::json!({"mode": "all"})),
            |e| matches!(e, CompileError::UnsatisfiableJoin { node_id, .. } if node_id == "j"),
            "`all` over exclusive branches",
        );
        expect_error(
            &route(serde_json::json!({"mode": "quorum", "quorum": 2})),
            |e| matches!(e, CompileError::UnsatisfiableJoin { .. }),
            "quorum 2 with at most 1 satisfiable source",
        );
    }

    /// Freeze §8 rule 2 (join half) — sources must exist, be unique, and
    /// actually feed the join; feeders must be declared; quorum bounded.
    #[test]
    fn static_validation_rejects_unsatisfiable_join_wiring() {
        control_mini_compiles();
        let base = |sources: serde_json::Value, extra_edges: Vec<serde_json::Value>| {
            let mut root = mini_graph();
            root["nodes"]["j"] = serde_json::json!({
                "kind": "join", "policy": {"mode": "quorum", "quorum": 3},
                "source_node_ids": sources
            });
            let edges = root["edges"].as_array_mut().unwrap();
            edges.push(
                serde_json::json!({"from": "j", "from_port": "out", "to": "do", "to_port": "in"}),
            );
            for e in extra_edges {
                edges.push(e);
            }
            mini_input(root)
        };
        expect_error(
            &base(serde_json::json!(["ghost"]), vec![]),
            |e| matches!(e, CompileError::UnsatisfiableJoin { detail, .. } if detail.contains("ghost")),
            "source naming no node",
        );
        expect_error(
            &base(
                serde_json::json!(["start", "start"]),
                vec![
                    serde_json::json!({"from": "start", "from_port": "out", "to": "j", "to_port": "in"}),
                ],
            ),
            |e| matches!(e, CompileError::UnsatisfiableJoin { detail, .. } if detail.contains("duplicated")),
            "duplicated source",
        );
        expect_error(
            &base(serde_json::json!(["start"]), vec![]),
            |e| matches!(e, CompileError::UnsatisfiableJoin { detail, .. } if detail.contains("never feeds")),
            "declared source without an edge",
        );
        expect_error(
            &base(
                serde_json::json!(["start"]),
                vec![
                    serde_json::json!({"from": "start", "from_port": "out", "to": "j", "to_port": "in"}),
                ],
            ),
            |e| matches!(e, CompileError::UnsatisfiableJoin { detail, .. } if detail.contains("quorum 3")),
            "quorum above the source count",
        );
    }

    /// §10.5 class 5 — "référence non pinnée": a ref whose digest is not in
    /// `PACK.lock`, malformed, or bound to different provided bytes.
    #[test]
    fn static_validation_rejects_unpinned_ref() {
        control_mini_compiles();
        // A schema pin the lock does not carry.
        let mut input = mini_input(mini_graph());
        input
            .pack
            .lock
            .entries
            .retain(|e| e.blake3 != pin_for("schema:payload"));
        input.pack.digests.content = crate::pack::content_digest_of_lock(&input.pack.lock).unwrap();
        expect_error(
            &input,
            |e| matches!(e, CompileError::UnpinnedReference { .. }),
            "schema digest absent from the lock",
        );

        // A malformed (non-hex) REFERENCE pin, mutated AFTER the input was
        // built: the synthetic lock mirrors every in-document pin, so an
        // in-document mutation BEFORE input-building would be refused at
        // LOCK level ("lock digest is not hex") and the ref-level hex guard
        // of `check_pin` would never run (review B-3). Mutating after keeps
        // the lock clean; the mutated ref is the WORKFLOW-level input pin,
        // which sits on no edge, so the edge-level port-type pass cannot
        // intercept before the ref-level guard fires.
        let mut input = mini_input(mini_graph());
        remint_root(&mut input, |v| {
            v["input_schema_ref"]["blake3"] = serde_json::json!("not-a-digest");
        });
        for entry in &input.pack.lock.entries {
            assert!(
                crate::pack::is_lower_hex_digest(&entry.blake3),
                "control: the lock must stay clean so the REF guard fires"
            );
        }
        expect_error(
            &input,
            |e| matches!(e, CompileError::UnpinnedReference { detail, .. } if detail.contains("hex")),
            "malformed reference pin",
        );
    }

    /// §10.5 class 6 — "output non typé", landed as the two-way
    /// output-node ↔ `artifact_outputs` binding.
    #[test]
    fn static_validation_rejects_undeclared_output() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["artifact_outputs"] = serde_json::json!([]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::OutputNotDeclared { detail, .. } if detail.contains("result")),
            "output node without declaration",
        );

        let mut root = mini_graph();
        root["artifact_outputs"][0]["schema_ref"] = sref("other");
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::OutputNotDeclared { detail, .. } if detail.contains("different schemas")),
            "declaration pinning another schema",
        );

        let mut root = mini_graph();
        root["artifact_outputs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "phantom", "schema_ref": sref("payload"), "from_node": "done"
            }));
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::OutputNotDeclared { detail, .. } if detail.contains("phantom")),
            "declared artefact no node publishes",
        );
    }

    /// §10.5 class 7 — "capacité demandée absente du manifeste du pack" +
    /// the workflow-level closure ("capability.call must reference a
    /// requested capability").
    #[test]
    fn static_validation_rejects_capability_not_granted() {
        control_mini_compiles();
        // Workflow requests something the pack manifest does not carry.
        let input = synth_input(mini_graph(), Vec::new(), Vec::new(), &[]);
        expect_error(
            &input,
            |e| matches!(e, CompileError::CapabilityNotInManifest { capability, .. } if capability == "cap.run"),
            "workflow request outside the manifest",
        );

        // Node calls something its own workflow does not request.
        let mut root = mini_graph();
        root["nodes"]["do"]["capability_ref"] = serde_json::json!("cap.other");
        let input = synth_input(root, Vec::new(), Vec::new(), &["cap.run", "cap.other"]);
        expect_error(
            &input,
            |e| {
                matches!(e, CompileError::CapabilityNotRequested { capability, node_id, .. }
                    if capability == "cap.other" && node_id == "do")
            },
            "call outside the workflow's own requests",
        );
    }

    /// §10.5 class 8 — "provider incompatible" (nominal v1: the CLI
    /// provider classes are vendor-bound by model prefix).
    #[test]
    fn static_validation_rejects_provider_incompatible() {
        let base = golden_input("factory");
        compile(&base).expect("control: the factory golden compiles");
        let mut hostile = base.clone();
        remint_agent(&mut hostile, "factory-driver", |v| {
            v["model_policy"]["model_id"] = serde_json::json!("gpt-5.6-sol");
        });
        expect_error(
            &hostile,
            |e| matches!(e, CompileError::ProviderIncompatible { detail, .. } if detail.contains("claude")),
            "claude-code provider bound to a gpt model",
        );
    }

    /// §10.5 class 9 — "handoff non autorisé par l'agent cible" (giver
    /// allowlist) + the undeclared-target refusal.
    #[test]
    fn static_validation_rejects_unauthorized_handoff() {
        let base = golden_input("factory");
        compile(&base).expect("control: the factory golden compiles");
        let mut hostile = base.clone();
        remint_agent(&mut hostile, "factory-driver", |v| {
            v["allowed_handoffs"] = serde_json::json!([]);
        });
        expect_error(
            &hostile,
            |e| {
                matches!(e, CompileError::UnauthorizedHandoff { node_id, .. }
                    if node_id == "handoff-to-review")
            },
            "giver without the target in its allowlist",
        );

        let mut hostile = golden_input("factory");
        remint_root(&mut hostile, |v| {
            v["nodes"]["handoff-to-review"]["target_agent_id"] = serde_json::json!("ghost-agent");
        });
        expect_error(
            &hostile,
            |e| matches!(e, CompileError::UndeclaredAgent { detail, .. } if detail.contains("ghost-agent")),
            "handoff to an undeclared agent",
        );
    }

    /// §10.5 class 10 — "secret literal".
    #[test]
    fn static_validation_rejects_secret_literal() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["description"] = serde_json::json!("sk-test1234567890 pasted by mistake");
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::SecretLiteral { path } if path.contains("description")),
            "credential-shaped literal",
        );
    }

    /// §10.5 class 11 — "endpoint ou code exécutable interdit".
    #[test]
    fn static_validation_rejects_forbidden_endpoint() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["description"] = serde_json::json!("see https://exfil.example/setup");
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ForbiddenEndpoint { .. }),
            "URL endpoint",
        );

        let mut root = mini_graph();
        root["nodes"]["do"]["capability_ref"] = serde_json::json!("cap.run");
        root["description"] = serde_json::json!("run $(curl evil)");
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ForbiddenEndpoint { detail, .. } if detail.contains("$(")),
            "executable marker",
        );
    }

    /// §10.5 class 12 — "budget enfant > parent", per DIMENSION: node vs
    /// workflow, invocation vs agent definition, child workflow vs parent
    /// workflow.
    #[test]
    fn static_validation_rejects_budget_overflow() {
        let base = golden_input("factory");
        compile(&base).expect("control: the factory golden compiles");

        // Node above its workflow ceiling (also above the agent definition;
        // the definition check fires first and is the same refusal class).
        let mut hostile = base.clone();
        remint_root(&mut hostile, |v| {
            v["nodes"]["drive-implementation"]["budget"]["max_wall_clock_seconds"] =
                serde_json::json!(999_999_999u64);
        });
        expect_error(
            &hostile,
            |e| matches!(e, CompileError::BudgetOverflow { dimension, .. } if dimension == "max_wall_clock_seconds"),
            "node budget above its ceilings",
        );

        // Child workflow above the parent scope, dimension by dimension.
        let mut hostile = golden_input("factory");
        remint_subworkflow(&mut hostile, "factory-codex-round", |v| {
            v["budget_policy"]["max_turns"] = serde_json::json!(1_000_000u32);
        });
        expect_error(
            &hostile,
            |e| matches!(e, CompileError::BudgetOverflow { dimension, .. } if dimension == "max_turns"),
            "child workflow budget above the parent",
        );
    }

    /// §10.5 class 13 — publication behind a non-model gate: the HITL half
    /// is live (provenance/evaluation are pack-required by parse). The
    /// control arm shows both accepted gates: `human.review` (ideas
    /// pattern) and `wait.signal` (babel community pattern).
    #[test]
    fn static_validation_rejects_publication_without_gate() {
        let publishing =
            |gated: bool| {
                let mut root = mini_graph();
                root["capability_requests"] = serde_json::json!(["content.publish"]);
                root["nodes"]["do"]["capability_ref"] = serde_json::json!("content.publish");
                if gated {
                    root["nodes"]["gate"] = serde_json::json!({
                        "kind": "human.review",
                        "payload_schema_ref": sref("payload"),
                        "decision_labels": ["approve", "reject"]
                    });
                    root["nodes"]["rejected"] = serde_json::json!({
                        "kind": "output", "schema_ref": sref("payload"), "artifact_name": "rejected"
                    });
                    root["artifact_outputs"].as_array_mut().unwrap().push(serde_json::json!({
                    "name": "rejected", "schema_ref": sref("payload"), "from_node": "rejected"
                }));
                    root["edges"] = serde_json::json!([
                        {"from": "start", "from_port": "out", "to": "gate", "to_port": "in"},
                        {"from": "gate", "from_port": "approve", "to": "do", "to_port": "in"},
                        {"from": "gate", "from_port": "reject", "to": "rejected", "to_port": "in"},
                        {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
                    ]);
                }
                synth_input(root, Vec::new(), Vec::new(), &["content.publish"])
            };
        compile(&publishing(true)).expect("control: a gated publication compiles");
        expect_error(
            &publishing(false),
            |e| matches!(e, CompileError::PublicationWithoutHumanGate { node_id, .. } if node_id == "do"),
            "ungated publication capability",
        );
    }

    /// Freeze §8 rule 1 — edge referential integrity + the duplicate-edge
    /// refusal (S1a-07).
    #[test]
    fn static_validation_rejects_dangling_and_duplicate_edges() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "ghost", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::DanglingEdge { detail, .. } if detail.contains("ghost")),
            "edge to a ghost node",
        );

        let mut root = mini_graph();
        let dup = root["edges"][0].clone();
        root["edges"].as_array_mut().unwrap().push(dup);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::DuplicateEdge { .. }),
            "identical edge twice",
        );
    }

    /// Freeze §8 rule 2 — port validity: unknown from_port, unwired routing
    /// label, unwired `error` port under route-error-port policy.
    #[test]
    fn static_validation_rejects_port_violations() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["edges"][0]["from_port"] = serde_json::json!("sideways");
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::UnknownPort { detail, .. } if detail.contains("sideways")),
            "undeclared from_port",
        );

        let mut root = mini_graph();
        root["nodes"]["sw"] = serde_json::json!({
            "kind": "switch", "discriminant_field": "route", "branch_labels": ["a", "b"]
        });
        let edges = root["edges"].as_array_mut().unwrap();
        edges.push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "sw", "to_port": "in"}),
        );
        edges.push(
            serde_json::json!({"from": "sw", "from_port": "a", "to": "done", "to_port": "in"}),
        );
        edges.push(serde_json::json!({"from": "sw", "from_port": "default", "to": "done", "to_port": "in"}));
        // label `b` left unwired
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::UnwiredPort { detail, .. } if detail.contains("`b`")),
            "routing label without an edge",
        );

        let mut root = mini_graph();
        root["error_policy"] = serde_json::json!({
            "on_node_failure": "route-error-port", "max_retries_per_node": 1
        });
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::UnwiredPort { detail, .. } if detail.contains("error")),
            "route-error-port policy without error wiring",
        );
    }

    /// Freeze §8 rule 4 — data-driven keys disjoint from
    /// `PRESENTATION_KEYS` (the `graph_semantic` malleability closure).
    #[test]
    fn static_validation_rejects_presentation_key_as_data() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["nodes"]["layout"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {}
        });
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "layout", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::PresentationKeyAsData { detail, .. } if detail.contains("layout")),
            "node id using a reserved presentation name",
        );

        let mut root = mini_graph();
        root["nodes"]["remap"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {"zoom": "source"}
        });
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "remap", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::PresentationKeyAsData { detail, .. } if detail.contains("zoom")),
            "field_map key using a reserved presentation name",
        );
    }

    /// Freeze §8 rule 5 — routing labels sane.
    #[test]
    fn static_validation_rejects_default_in_branch_labels() {
        control_mini_compiles();
        let with_switch = |labels: serde_json::Value| {
            let mut root = mini_graph();
            root["nodes"]["sw"] = serde_json::json!({
                "kind": "switch", "discriminant_field": "route", "branch_labels": labels
            });
            root["edges"].as_array_mut().unwrap().push(
                serde_json::json!({"from": "do", "from_port": "out", "to": "sw", "to_port": "in"}),
            );
            mini_input(root)
        };
        expect_error(
            &with_switch(serde_json::json!(["default"])),
            |e| matches!(e, CompileError::InvalidRoutingLabels { detail, .. } if detail.contains("default")),
            "`default` among branch labels",
        );
        expect_error(
            &with_switch(serde_json::json!([])),
            |e| matches!(e, CompileError::InvalidRoutingLabels { detail, .. } if detail.contains("empty")),
            "empty label set",
        );
        expect_error(
            &with_switch(serde_json::json!(["a", "a"])),
            |e| matches!(e, CompileError::InvalidRoutingLabels { detail, .. } if detail.contains("duplicated")),
            "duplicated label",
        );
        // Phase K storage-aligned bound: a label the engine's deposit
        // door would refuse to STORE (256 KiB) must not compile — a
        // valid-but-unresumable plan is a refusal class, not a surprise.
        let oversized = "x".repeat(MAX_DECISION_LABEL_BYTES + 1);
        expect_error(
            &with_switch(serde_json::json!([oversized])),
            |e| matches!(e, CompileError::InvalidRoutingLabels { detail, .. } if detail.contains("storage bound")),
            "label beyond the storage bound",
        );
    }

    /// Freeze §8 rule 6 — provenance of classification discriminants, by
    /// data-flow analysis over the EXISTING typed ports (never a new schema
    /// field): a model-produced `visibility` routing an effect is refused;
    /// laundering it through a `human.review` legalises it; a QUALITY
    /// discriminant stays out of scope (the babel pattern).
    #[test]
    fn static_validation_rejects_classification_from_model() {
        let routed = |discriminant: &str, gate_between: bool| {
            let mut root = mini_graph();
            root["nodes"]["think"] = serde_json::json!({
                "kind": "model.invoke",
                "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                                  "require_structured_output": true},
                "budget": big_budget()
            });
            root["nodes"]["sw"] = serde_json::json!({
                "kind": "switch", "discriminant_field": discriminant,
                "branch_labels": ["go"]
            });
            root["nodes"]["sink"] = serde_json::json!({
                "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
            });
            root["artifact_outputs"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
                }));
            let mut edges = vec![
                serde_json::json!({"from": "sw", "from_port": "go", "to": "do", "to_port": "in"}),
                serde_json::json!({"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"}),
                serde_json::json!({"from": "do", "from_port": "out", "to": "done", "to_port": "in"}),
            ];
            if gate_between {
                root["nodes"]["gate"] = serde_json::json!({
                    "kind": "human.review",
                    "payload_schema_ref": sref("payload"),
                    "decision_labels": ["confirm"]
                });
                edges.push(serde_json::json!({"from": "start", "from_port": "out", "to": "think", "to_port": "in"}));
                edges.push(serde_json::json!({"from": "think", "from_port": "out", "to": "gate", "to_port": "in"}));
                edges.push(serde_json::json!({"from": "gate", "from_port": "confirm", "to": "sw", "to_port": "in"}));
            } else {
                edges.push(serde_json::json!({"from": "start", "from_port": "out", "to": "think", "to_port": "in"}));
                edges.push(serde_json::json!({"from": "think", "from_port": "out", "to": "sw", "to_port": "in"}));
            }
            root["edges"] = serde_json::Value::Array(edges);
            mini_input(root)
        };
        compile(&routed("quality", false))
            .expect("control: a model-fed QUALITY gate is out of the rule's scope");
        compile(&routed("visibility", true))
            .expect("control: a human.review launders the provenance");
        expect_error(
            &routed("visibility", false),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "model-produced classification discriminant routing an effect",
        );

        // The classifier arm: proposing classification labels with an
        // effect downstream IS the model deciding the classification.
        let mut root = mini_graph();
        root["nodes"]["cls"] = serde_json::json!({
            "kind": "classifier",
            "prompt_ref": {"prompt_id": "c", "blake3": pin_for("prompt:c")},
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            "branch_labels": ["public", "internal"],
            "budget": big_budget()
        });
        root["nodes"]["sink"] = serde_json::json!({
            "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
        });
        root["artifact_outputs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
            }));
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "cls", "to_port": "in"},
            {"from": "cls", "from_port": "public", "to": "do", "to_port": "in"},
            {"from": "cls", "from_port": "internal", "to": "sink", "to_port": "in"},
            {"from": "cls", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "cls"),
            "classifier proposing classification labels above an effect",
        );
    }

    /// The taint TRAVERSAL branches of `field_taints_from_model`, each with
    /// its own red arm (review C-2 — the laundering-prone paths must have
    /// discriminating controls, not just the direct model→switch case):
    /// transform-rename follows `field_map`, an unmapped field is unknown
    /// provenance (fail-closed), map bodies are opaque, and a join ORs its
    /// sources' taint.
    #[test]
    fn static_validation_taint_traversal_branches() {
        // Shared scaffold: sw(visibility) routing `go` into the effect and
        // `default` into a sink output.
        let scaffold = |root: &mut serde_json::Value| {
            root["nodes"]["sw"] = serde_json::json!({
                "kind": "switch", "discriminant_field": "visibility",
                "branch_labels": ["go"]
            });
            root["nodes"]["sink"] = serde_json::json!({
                "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
            });
            root["artifact_outputs"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
                }));
        };
        let model_node = || {
            serde_json::json!({
                "kind": "model.invoke",
                "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                                  "require_structured_output": true},
                "budget": big_budget()
            })
        };
        let remap = |map: serde_json::Value| {
            serde_json::json!({
                "kind": "transform",
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "field_map": map
            })
        };

        // (a) transform-RENAME does not launder: model → transform
        // {visibility ← verdict} → switch. Control: same rename chain fed by
        // the INPUT is clean and compiles.
        let rename_route = |with_model: bool| {
            let mut root = mini_graph();
            scaffold(&mut root);
            root["nodes"]["remap"] = remap(serde_json::json!({"visibility": "verdict"}));
            let mut edges = vec![
                serde_json::json!({"from": "remap", "from_port": "out", "to": "sw", "to_port": "in"}),
                serde_json::json!({"from": "sw", "from_port": "go", "to": "do", "to_port": "in"}),
                serde_json::json!({"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"}),
                serde_json::json!({"from": "do", "from_port": "out", "to": "done", "to_port": "in"}),
            ];
            if with_model {
                root["nodes"]["think"] = model_node();
                edges.push(serde_json::json!({"from": "start", "from_port": "out", "to": "think", "to_port": "in"}));
                edges.push(serde_json::json!({"from": "think", "from_port": "out", "to": "remap", "to_port": "in"}));
            } else {
                edges.push(serde_json::json!({"from": "start", "from_port": "out", "to": "remap", "to_port": "in"}));
            }
            root["edges"] = serde_json::Value::Array(edges);
            mini_input(root)
        };
        compile(&rename_route(false)).expect("control: input-fed rename chain is clean");
        expect_error(
            &rename_route(true),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "transform rename must not launder model provenance",
        );

        // (b) a field the transform does NOT map has unknown provenance —
        // fail-closed even with a clean upstream.
        let mut root = mini_graph();
        scaffold(&mut root);
        root["nodes"]["remap"] = remap(serde_json::json!({"other": "x"}));
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "remap", "to_port": "in"},
            {"from": "remap", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "do", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "an unmapped discriminant is unknown provenance — fail-closed",
        );

        // (c) a map body is OPAQUE: its output taints fail-closed.
        let mut child = mini_graph();
        child["workflow_id"] = serde_json::json!("child");
        let mut root = mini_graph();
        scaffold(&mut root);
        root["nodes"]["fan"] = serde_json::json!({
            "kind": "map",
            "collection_field": "items",
            "max_cardinality": 4,
            "body_subworkflow_ref": {"workflow_id": "child", "blake3": pin_for("doc:sub:child")}
        });
        root["subworkflow_refs"] = serde_json::json!([
            {"workflow_id": "child", "blake3": pin_for("doc:sub:child")}
        ]);
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "fan", "to_port": "in"},
            {"from": "fan", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "do", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &synth_input(root, vec![("child", child)], Vec::new(), &["cap.run"]),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "an opaque map-body output must taint fail-closed",
        );

        // (d) a join ORs its sources: one clean transform + one model in →
        // tainted.
        let mut root = mini_graph();
        scaffold(&mut root);
        root["nodes"]["prep"] = remap(serde_json::json!({"visibility": "visibility"}));
        root["nodes"]["think"] = model_node();
        root["nodes"]["j"] = serde_json::json!({
            "kind": "join", "policy": {"mode": "all"},
            "source_node_ids": ["prep", "think"]
        });
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "prep", "to_port": "in"},
            {"from": "start", "from_port": "out", "to": "think", "to_port": "in"},
            {"from": "prep", "from_port": "out", "to": "j", "to_port": "in"},
            {"from": "think", "from_port": "out", "to": "j", "to_port": "in"},
            {"from": "j", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "do", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "a join with one model source must taint",
        );
    }

    /// Freeze §7 — port compatibility, nominal by digest: one schema id
    /// pinned under two digests across an edge is the drift/poisoning case.
    #[test]
    fn static_validation_rejects_port_type_mismatch() {
        control_mini_compiles();
        let mut root = mini_graph();
        // The producer pins `payload` under a DIFFERENT digest than the
        // consumer's `payload`.
        root["nodes"]["start"]["schema_ref"]["blake3"] =
            serde_json::json!(pin_for("schema:payload-drifted"));
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::PortTypeMismatch { detail, .. } if detail.contains("payload")),
            "same schema id under two digests",
        );

        // The PLAN-WIDE one-digest-per-schema-id rule (`check_schema_ref`)
        // fires even OFF-edge (review B-4 — the edge-level pass runs first
        // and would otherwise mask it): the workflow-level output pin
        // diverging from the input pin of the same id sits on no edge.
        let mut root = mini_graph();
        root["output_schema_ref"]["blake3"] =
            serde_json::json!(pin_for("schema:payload-divergent"));
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::PortTypeMismatch { detail, .. } if detail.contains("two digests")),
            "off-edge same-id divergence",
        );
    }

    /// Structure refusals around entrypoints and undeclared referents.
    #[test]
    fn static_validation_rejects_missing_entrypoints_and_referents() {
        control_mini_compiles();
        let mut root = mini_graph();
        root["entrypoints"] = serde_json::json!([]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::NoEntrypoint { .. }),
            "empty entrypoints",
        );

        let mut root = mini_graph();
        root["entrypoints"] = serde_json::json!(["ghost"]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::NoEntrypoint { detail, .. } if detail.contains("ghost")),
            "entrypoint naming no node",
        );

        let mut root = mini_graph();
        root["nodes"]["call"] = serde_json::json!({
            "kind": "agent.invoke", "agent_id": "ghost", "budget": big_budget()
        });
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "call", "to_port": "in"}),
        );
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::UndeclaredAgent { detail, .. } if detail.contains("ghost")),
            "agent.invoke outside agent_refs",
        );
    }

    /// A chain of `levels` nested subworkflows under the root, each pinned
    /// through a `subworkflow` node.
    fn deep_input(levels: u32) -> CompileInput {
        let leaf_id = format!("sub-{levels}");
        let mut subworkflows: Vec<(String, serde_json::Value)> = Vec::new();
        let mut leaf = mini_graph();
        leaf["workflow_id"] = serde_json::json!(leaf_id);
        subworkflows.push((leaf_id, leaf));
        for depth in (1..levels).rev() {
            let id = format!("sub-{depth}");
            let child_id = format!("sub-{}", depth + 1);
            let mut wf = mini_graph();
            wf["workflow_id"] = serde_json::json!(id.clone());
            wf["nodes"]["deeper"] = serde_json::json!({
                "kind": "subworkflow",
                "subworkflow_ref": {"workflow_id": child_id, "blake3": pin_for(&format!("doc:sub:{child_id}"))}
            });
            wf["subworkflow_refs"] = serde_json::json!([
                {"workflow_id": child_id, "blake3": pin_for(&format!("doc:sub:{child_id}"))}
            ]);
            wf["edges"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({"from": "do", "from_port": "out", "to": "deeper", "to_port": "in"}));
            subworkflows.push((id, wf));
        }
        let mut root = mini_graph();
        root["nodes"]["deeper"] = serde_json::json!({
            "kind": "subworkflow",
            "subworkflow_ref": {"workflow_id": "sub-1", "blake3": pin_for("doc:sub:sub-1")}
        });
        root["subworkflow_refs"] = serde_json::json!([
            {"workflow_id": "sub-1", "blake3": pin_for("doc:sub:sub-1")}
        ]);
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "deeper", "to_port": "in"}),
        );
        synth_input(
            root,
            subworkflows
                .iter()
                .map(|(id, doc)| (id.as_str(), doc.clone()))
                .collect(),
            Vec::new(),
            &["cap.run"],
        )
    }

    /// The nesting bound is INCLUSIVE (review B-2 — a `>` vs `>=` mutation
    /// must be killed from both sides): exactly [`MAX_PLAN_DEPTH`] levels
    /// compile, one more is refused.
    #[test]
    fn static_validation_rejects_plan_too_deep() {
        compile(&deep_input(MAX_PLAN_DEPTH))
            .expect("control: exactly MAX_PLAN_DEPTH levels must compile");
        expect_error(
            &deep_input(MAX_PLAN_DEPTH + 1),
            |e| matches!(e, CompileError::PlanTooDeep { .. }),
            "one level beyond the bound",
        );
    }

    /// Budget comparison is `child <= parent`, and EQUALITY passes (review
    /// B-2: every golden is strictly below its ceilings, so a `>` → `>=`
    /// mutation — refusing equality, a real bug — would otherwise survive
    /// the whole suite).
    #[test]
    fn budget_equality_is_within_bounds() {
        let mut root = mini_graph();
        root["nodes"]["think"] = serde_json::json!({
            "kind": "model.invoke",
            "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            // EXACTLY the workflow ceiling, every dimension.
            "budget": big_budget()
        });
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "think", "to_port": "in"}),
        );
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "think", "from_port": "out", "to": "done", "to_port": "in"}),
        );
        // Rewire done's feed: think now terminates the chain.
        root["edges"]
            .as_array_mut()
            .unwrap()
            .retain(|e| !(e["from"] == "do" && e["to"] == "done"));
        compile(&mini_input(root)).expect("a node budget EQUAL to its scope must compile");
    }

    /// A minimal valid agent definition document.
    fn mini_agent(id: &str, out_schema: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "agent_id": id,
            "agent_version": 1,
            "display_name": id,
            "description": "Synthetic bounded agent.",
            "instructions_ref": {"prompt_id": format!("{id}-instructions"),
                                  "blake3": pin_for(&format!("prompt:{id}"))},
            "input_schema_ref": sref("payload"),
            "output_schema_ref": out_schema,
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            "tool_refs": [],
            "allowed_handoffs": [],
            "context_policy": {"allowed_context_fields": []},
            "memory_policy": "none",
            "guardrail_refs": [],
            "approval_policy": "never",
            "budget": big_budget()
        })
    }

    /// A root invoking one declared agent, its handoff chain configurable.
    fn agent_root(with_transform_between: bool) -> serde_json::Value {
        let mut root = mini_graph();
        root["agent_refs"] = serde_json::json!([
            {"agent_id": "giver", "blake3": "patched"},
            {"agent_id": "receiver", "blake3": "patched"}
        ]);
        root["nodes"]["invoke"] = serde_json::json!({
            "kind": "agent.invoke", "agent_id": "giver", "budget": big_budget()
        });
        root["nodes"]["hand"] = serde_json::json!({
            "kind": "handoff", "target_agent_id": "receiver",
            "payload_schema_ref": sref("payload"),
            "context_policy": {"allowed_context_fields": []}
        });
        let mut extra = vec![
            serde_json::json!({"from": "do", "from_port": "out", "to": "invoke", "to_port": "in"}),
            serde_json::json!({"from": "hand", "from_port": "to", "to": "done", "to_port": "in"}),
        ];
        if with_transform_between {
            root["nodes"]["mid"] = serde_json::json!({
                "kind": "transform",
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "field_map": {}
            });
            extra.push(serde_json::json!({"from": "invoke", "from_port": "out", "to": "mid", "to_port": "in"}));
            extra.push(serde_json::json!({"from": "mid", "from_port": "out", "to": "hand", "to_port": "in"}));
        } else {
            extra.push(serde_json::json!({"from": "invoke", "from_port": "out", "to": "hand", "to_port": "in"}));
        }
        root["edges"].as_array_mut().unwrap().extend(extra);
        // `done` is now fed by the handoff, not by `do`.
        root["edges"]
            .as_array_mut()
            .unwrap()
            .retain(|e| !(e["from"] == "do" && e["to"] == "done"));
        root
    }

    /// Codex r2: a transform between the giver agent and the handoff must
    /// not launder the allowlist check (transitive upstream walk).
    #[test]
    fn static_validation_rejects_transitive_unauthorized_handoff() {
        let allowing = |allowed: serde_json::Value| {
            let mut giver = mini_agent("giver", sref("payload"));
            giver["allowed_handoffs"] = allowed;
            synth_input(
                agent_root(true),
                Vec::new(),
                vec![
                    ("giver", giver),
                    ("receiver", mini_agent("receiver", sref("payload"))),
                ],
                &["cap.run"],
            )
        };
        compile(&allowing(serde_json::json!(["receiver"])))
            .expect("control: an allowlisting giver compiles through a transform");
        expect_error(
            &allowing(serde_json::json!([])),
            |e| matches!(e, CompileError::UnauthorizedHandoff { node_id, .. } if node_id == "hand"),
            "a transform must not launder the giver allowlist",
        );
    }

    /// Codex r2: one agent id, ONE digest — a second reference under a
    /// different digest must be refused even on a warm cache.
    #[test]
    fn static_validation_rejects_agent_under_two_digests() {
        let child = {
            let mut c = mini_graph();
            c["workflow_id"] = serde_json::json!("child");
            c["agent_refs"] = serde_json::json!([
                {"agent_id": "giver", "blake3": "patched"}
            ]);
            c
        };
        let mut root = mini_graph();
        root["agent_refs"] = serde_json::json!([{"agent_id": "giver", "blake3": "patched"}]);
        root["nodes"]["deeper"] = serde_json::json!({
            "kind": "subworkflow",
            "subworkflow_ref": {"workflow_id": "child", "blake3": "patched"}
        });
        root["subworkflow_refs"] = serde_json::json!([
            {"workflow_id": "child", "blake3": "patched"}
        ]);
        root["edges"].as_array_mut().unwrap().push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "deeper", "to_port": "in"}),
        );
        let mut input = synth_input(
            root,
            vec![("child", child)],
            vec![("giver", mini_agent("giver", sref("payload")))],
            &["cap.run"],
        );
        compile(&input).expect("control: one digest everywhere compiles");
        remint_subworkflow(&mut input, "child", |v| {
            v["agent_refs"][0]["blake3"] = serde_json::json!(pin_for("agent:giver-fork"));
        });
        expect_error(
            &input,
            |e| matches!(e, CompileError::UnpinnedReference { detail, .. } if detail.contains("two digests")),
            "a second digest for a cached agent id",
        );
    }

    /// Codex r2: a capability result CAN relay its model-fed arguments —
    /// `capability.call` must not launder classification provenance.
    #[test]
    fn static_validation_rejects_capability_laundered_classification() {
        let mut root = mini_graph();
        root["nodes"]["think"] = serde_json::json!({
            "kind": "model.invoke",
            "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            "budget": big_budget()
        });
        root["nodes"]["relay"] = serde_json::json!({
            "kind": "capability.call", "capability_ref": "cap.run",
            "args_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload")
        });
        root["nodes"]["sw"] = serde_json::json!({
            "kind": "switch", "discriminant_field": "visibility", "branch_labels": ["go"]
        });
        root["nodes"]["sink"] = serde_json::json!({
            "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
        });
        root["artifact_outputs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
            }));
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "think", "to_port": "in"},
            {"from": "think", "from_port": "out", "to": "relay", "to_port": "in"},
            {"from": "relay", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "do", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "a capability call must not launder model provenance",
        );
    }

    /// Codex r3: a knowledge-query RESULT is a model-steerable SELECTION
    /// even over a pinned corpus - it must not launder either.
    #[test]
    fn static_validation_rejects_knowledge_laundered_classification() {
        let mut root = mini_graph();
        root["nodes"]["think"] = serde_json::json!({
            "kind": "model.invoke",
            "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            "budget": big_budget()
        });
        root["nodes"]["lookup"] = serde_json::json!({
            "kind": "knowledge.query",
            "corpus_ref": {"corpus_id": "kb", "blake3": pin_for("corpus:kb")},
            "query_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload")
        });
        root["nodes"]["sw"] = serde_json::json!({
            "kind": "switch", "discriminant_field": "visibility", "branch_labels": ["go"]
        });
        root["nodes"]["sink"] = serde_json::json!({
            "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
        });
        root["artifact_outputs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
            }));
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "think", "to_port": "in"},
            {"from": "think", "from_port": "out", "to": "lookup", "to_port": "in"},
            {"from": "lookup", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "do", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &mini_input(root),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "a knowledge query must not launder model provenance",
        );
    }

    /// Codex r3 (statuted contract): `agent.as_tool` is NOT a handoff
    /// giver - the specialist RETURNS to the parent (D4); the walk passes
    /// through it looking for an orchestrating `agent.invoke`.
    #[test]
    fn handoff_giver_contract_stops_at_agent_invoke() {
        let mut root = mini_graph();
        root["agent_refs"] = serde_json::json!([
            {"agent_id": "worker", "blake3": "patched"},
            {"agent_id": "receiver", "blake3": "patched"}
        ]);
        root["nodes"]["tool"] = serde_json::json!({
            "kind": "agent.as_tool", "agent_id": "worker",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "budget": big_budget()
        });
        root["nodes"]["mid"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {}
        });
        root["nodes"]["hand"] = serde_json::json!({
            "kind": "handoff", "target_agent_id": "receiver",
            "payload_schema_ref": sref("payload"),
            "context_policy": {"allowed_context_fields": []}
        });
        {
            let edges = root["edges"].as_array_mut().unwrap();
            edges.push(serde_json::json!({"from": "do", "from_port": "out", "to": "tool", "to_port": "in"}));
            edges.push(serde_json::json!({"from": "tool", "from_port": "out", "to": "mid", "to_port": "in"}));
            edges.push(serde_json::json!({"from": "mid", "from_port": "out", "to": "hand", "to_port": "in"}));
            edges.push(serde_json::json!({"from": "hand", "from_port": "to", "to": "done", "to_port": "in"}));
        }
        root["edges"]
            .as_array_mut()
            .unwrap()
            .retain(|e| !(e["from"] == "do" && e["to"] == "done"));
        // The specialist definition allowlists NOTHING - and that is
        // legal: it is not the giver.
        let input = synth_input(
            root,
            Vec::new(),
            vec![
                ("worker", mini_agent("worker", sref("payload"))),
                ("receiver", mini_agent("receiver", sref("payload"))),
            ],
            &["cap.run"],
        );
        compile(&input).expect("an agent.as_tool upstream is not a handoff giver (statuted)");

        // The RESPONSIBLE giver is the orchestrating agent.invoke UPSTREAM
        // of the as_tool call: the walk passes THROUGH the tool node and
        // holds the parent to its allowlist — green when it allows the
        // target, red when it does not.
        let orchestrated = |parent_allows: bool| {
            let mut r = mini_graph();
            r["agent_refs"] = serde_json::json!([
                {"agent_id": "parent", "blake3": "patched"},
                {"agent_id": "worker", "blake3": "patched"},
                {"agent_id": "receiver", "blake3": "patched"}
            ]);
            r["nodes"]["orchestrate"] = serde_json::json!({
                "kind": "agent.invoke", "agent_id": "parent", "budget": big_budget()
            });
            r["nodes"]["tool"] = serde_json::json!({
                "kind": "agent.as_tool", "agent_id": "worker",
                "input_schema_ref": sref("payload"),
                "output_schema_ref": sref("payload"),
                "budget": big_budget()
            });
            r["nodes"]["hand"] = serde_json::json!({
                "kind": "handoff", "target_agent_id": "receiver",
                "payload_schema_ref": sref("payload"),
                "context_policy": {"allowed_context_fields": []}
            });
            // `do` (the default capability.call) is dropped — the chain
            // is start→orchestrate→tool→hand→done only.
            r["nodes"].as_object_mut().unwrap().remove("do");
            r["capability_requests"] = serde_json::json!([]);
            r["edges"] = serde_json::json!([
                {"from": "start", "from_port": "out", "to": "orchestrate", "to_port": "in"},
                {"from": "orchestrate", "from_port": "out", "to": "tool", "to_port": "in"},
                {"from": "tool", "from_port": "out", "to": "hand", "to_port": "in"},
                {"from": "hand", "from_port": "to", "to": "done", "to_port": "in"}
            ]);
            let mut parent = mini_agent("parent", sref("payload"));
            if parent_allows {
                parent["allowed_handoffs"] = serde_json::json!(["receiver"]);
            }
            synth_input(
                r,
                Vec::new(),
                vec![
                    ("parent", parent),
                    ("worker", mini_agent("worker", sref("payload"))),
                    ("receiver", mini_agent("receiver", sref("payload"))),
                ],
                &["cap.run"],
            )
        };
        compile(&orchestrated(true))
            .expect("the orchestrating parent allows the target — through the tool node");
        expect_error(
            &orchestrated(false),
            |e| matches!(e, CompileError::UnauthorizedHandoff { node_id, .. } if node_id == "hand"),
            "the walk holds the orchestrating agent.invoke to its allowlist through the tool",
        );
    }

    /// Codex r3: the opaque-downstream-effect arm has its own red test -
    /// a model-fed classification switch routing into a SUBWORKFLOW is
    /// refused (the child may be effectful; fail-closed).
    #[test]
    fn static_validation_rejects_classification_routing_into_subworkflow() {
        let child = {
            let mut c = mini_graph();
            c["workflow_id"] = serde_json::json!("child");
            c
        };
        let mut root = mini_graph();
        root["nodes"]["think"] = serde_json::json!({
            "kind": "model.invoke",
            "prompt_ref": {"prompt_id": "p", "blake3": pin_for("prompt:p")},
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "model_policy": {"provider_class": "local", "model_id": "qwen3:14b",
                              "require_structured_output": true},
            "budget": big_budget()
        });
        root["nodes"]["sw"] = serde_json::json!({
            "kind": "switch", "discriminant_field": "visibility", "branch_labels": ["go"]
        });
        root["nodes"]["deeper"] = serde_json::json!({
            "kind": "subworkflow",
            "subworkflow_ref": {"workflow_id": "child", "blake3": "patched"}
        });
        root["subworkflow_refs"] = serde_json::json!([
            {"workflow_id": "child", "blake3": "patched"}
        ]);
        root["nodes"]["sink"] = serde_json::json!({
            "kind": "output", "schema_ref": sref("payload"), "artifact_name": "sink"
        });
        root["artifact_outputs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "sink", "schema_ref": sref("payload"), "from_node": "sink"
            }));
        // ISOLATION (Codex r4): the ONLY downstream effect is the
        // subworkflow itself — the default `do` capability.call is replaced
        // by a pure transform, so a green here would prove `Subworkflow`
        // stopped counting as an effect, not some other effect aval.
        root["nodes"]["do"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {}
        });
        root["capability_requests"] = serde_json::json!([]);
        root["edges"] = serde_json::json!([
            {"from": "start", "from_port": "out", "to": "think", "to_port": "in"},
            {"from": "think", "from_port": "out", "to": "sw", "to_port": "in"},
            {"from": "sw", "from_port": "go", "to": "deeper", "to_port": "in"},
            {"from": "sw", "from_port": "default", "to": "sink", "to_port": "in"},
            {"from": "deeper", "from_port": "out", "to": "do", "to_port": "in"},
            {"from": "do", "from_port": "out", "to": "done", "to_port": "in"}
        ]);
        expect_error(
            &synth_input(root, vec![("child", child)], Vec::new(), &[]),
            |e| matches!(e, CompileError::ClassificationFromModel { node_id, .. } if node_id == "sw"),
            "routing a model-fed classification into a subworkflow (sole effect)",
        );
    }

    /// Codex r2: agent-adjacent edges ARE type-checked (the agents are
    /// loaded BEFORE the edge-level pass): an agent output pinning schema
    /// id `payload` under a digest different from the consumer's is refused.
    #[test]
    fn static_validation_rejects_agent_edge_port_type_mismatch() {
        let mut root = mini_graph();
        root["agent_refs"] = serde_json::json!([{"agent_id": "worker", "blake3": "patched"}]);
        root["nodes"]["invoke"] = serde_json::json!({
            "kind": "agent.invoke", "agent_id": "worker", "budget": big_budget()
        });
        root["nodes"]["after"] = serde_json::json!({
            "kind": "transform",
            "input_schema_ref": sref("payload"),
            "output_schema_ref": sref("payload"),
            "field_map": {}
        });
        let edges = root["edges"].as_array_mut().unwrap();
        edges.push(
            serde_json::json!({"from": "do", "from_port": "out", "to": "invoke", "to_port": "in"}),
        );
        edges.push(serde_json::json!({"from": "invoke", "from_port": "out", "to": "after", "to_port": "in"}));
        let drifted_out = serde_json::json!({
            "schema_id": "payload", "blake3": pin_for("schema:payload-agent-drift")
        });
        // The agent's own input uses a DISTINCT id so the drifted `payload`
        // output is first seen at the EDGE pass, not inside the agent's own
        // plan-wide registration.
        let mut worker = mini_agent("worker", drifted_out);
        worker["input_schema_ref"] = sref("agent-in");
        let input = synth_input(root, Vec::new(), vec![("worker", worker)], &["cap.run"]);
        expect_error(
            &input,
            |e| matches!(e, CompileError::PortTypeMismatch { detail, .. } if detail.contains("edge")),
            "agent output edge with same-id different-digest",
        );
    }

    /// Codex r2: the cost dimension has its own red arm.
    #[test]
    fn static_validation_rejects_cost_budget_overflow() {
        let mut hostile = golden_input("factory");
        remint_root(&mut hostile, |v| {
            v["nodes"]["drive-implementation"]["budget"]["max_cost_micro_usd"] =
                serde_json::json!(999_999_999_999u64);
        });
        expect_error(
            &hostile,
            |e| matches!(e, CompileError::BudgetOverflow { dimension, .. } if dimension == "max_cost_micro_usd"),
            "cost dimension overflow",
        );
    }

    /// Codex r2: the CLASSIFICATION family of §19.3 is pinned through a
    /// REAL classification-adjacent discriminant (the babel quality-gate
    /// switch body), not only through HITL decision labels.
    #[test]
    fn plan_pins_switch_discriminants() {
        let babel = compile(&golden_input("babel")).expect("babel compiles");
        let digest = compiled_plan_digest(&babel).unwrap();
        let value = serde_json::to_value(&babel).unwrap();
        let mut mutated = value.clone();
        let node = mutated["node_instances"]["/quality-gate"]["node"]
            .as_object_mut()
            .expect("babel quality-gate switch exists in the plan");
        node.insert("discriminant_field".into(), serde_json::json!("mutated"));
        assert_ne!(mutated, value);
        assert_ne!(
            digest_for(PackDigestKind::CompiledPlan, &mutated).unwrap(),
            digest,
            "a switch discriminant must be covered by the plan digest"
        );
    }
}
