// SPDX-License-Identifier: AGPL-3.0-or-later
//! `SBFBWorkflowGraph.v1` — the data-flow workflow graph (Sprint 83 Phase G).
//!
//! The signed graph IS the plan (D4, Plan-Then-Execute/CaMeL): no LLM extends
//! the graph, grants a capability, changes a destination or produces an
//! authoritative process verdict at runtime. This module bakes that invariant
//! into the SHAPE of the format wherever a shape can carry it:
//!
//! - the node roster is a CLOSED, internally-tagged union
//!   ([`WorkflowNode`], tag `kind`, one canonical dotted spelling per kind,
//!   `deny_unknown_fields` on every variant body) — an unknown node kind or
//!   an unknown field is a PARSE ERROR, never tolerated data. This is an
//!   ASSUMED divergence from the additive-tolerant feed raw-op policy
//!   (`CLAUDE.md §Pre-launch`): a plan that will be granted capabilities must
//!   fail closed on anything it does not understand (AM-12 tension, recorded
//!   in `sprint83_node_profile_freeze.md`);
//! - loops are bounded STRUCTURALLY: [`MapNode`]/[`RepeatNode`] carry a
//!   required non-zero constant bound and delegate their body to a PINNED
//!   subworkflow. The format itself has NO back-edge primitive — iteration
//!   exists only through those two bounded nodes, whose body is a
//!   content-addressed child (self-recursion cannot be expressed) — but a
//!   hand-authored `edges` list can still spell an arbitrary cycle: rejecting
//!   ANY cycle over arbitrary edges is a Phase H compiler refusal (§10.5),
//!   consigned in the freeze artefact. Note the bounds have no UPPER cap
//!   (`u32::MAX` parses): the bound stays a signed constant — the residual
//!   risk is DoS-within-budget, a Phase H concern, not an authority hole;
//! - routing nodes ([`SwitchNode`], [`ClassifierNode`]) expose a STATIC label
//!   set; a model may SELECT a label, never mint a destination — the
//!   reserved `default` port catches every non-matching value
//!   deterministically;
//! - [`HandoffNode`] REQUIRES its payload schema and context allowlist
//!   (never optional, never defaulted — preflight B3): a handoff transfers
//!   control with a filtered, schematised payload and has NO return port,
//!   which is exactly what distinguishes it from [`AgentAsToolNode`], where
//!   the parent keeps synthesis and guardrails and the specialist RETURNS;
//! - effects go through [`CapabilityCallNode`]: a capability REFERENCE that
//!   must be requested by the pack, args typed by a pinned schema — no
//!   endpoint, URL, host, command or inline code exists anywhere in the
//!   format;
//! - budgets are multi-dimensional, REQUIRED, and appear in the signed
//!   format at the workflow root and on every model-calling node
//!   ([`crate::agent_definition::BudgetPolicy`]); the only invoke-time
//!   override surface is the budget itself — attenuation is STRUCTURAL for
//!   tools/handoffs/context/memory (no field exists that could widen them),
//!   while the budget MAGNITUDE is not clamped at parse: the numeric
//!   child<=parent comparison is a Phase H compiler refusal;
//! - every field of every struct is REQUIRED (no `Option`, no
//!   `#[serde(default)]`): the pack's canonicalisation door rejects `null`,
//!   and absence-means-something formats are how bounds silently become
//!   unlimited. "Zero secret" holds by CONSTRUCTION only for the absence of
//!   any secret-bearing field; the free-text fields (descriptions, ids,
//!   `field_map` values, …) are NOT credential-scanned here — graph/agent
//!   files are pinned by digest, never embedded in the pack root that
//!   `scan_for_disclosures` walks — so a best-effort scan of pinned bytes
//!   is routed to the pack-assembly/Hub phases (H/T).
//!
//! Node identity: `nodes` is a map keyed by node id, so id uniqueness is
//! structural and no divergent inline `id` field can exist. The id is the
//! stable scope the run engine later extends into `node_instance_id`
//! (research §12.2) and the HITL approval binding (Phase L1). Edge endpoint
//! integrity (every edge names an existing node) and reachability are Phase H
//! compiler refusals — consigned, with the rest of §10.5, in
//! `sprint83_node_profile_freeze.md`.
//!
//! What this module does NOT do (Phase H and later): compile, validate
//! statically (§10.5), close capabilities, resolve refs, or execute. Phase G
//! is the language, four committed golden workflows that VALIDATE, and the
//! dated node-profile freeze.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent_definition::{BudgetPolicy, ContextPolicy, ModelPolicy, PromptRef, SchemaRef};

/// The FROZEN node-kind roster, in research §10.3 table order — the dated
/// freeze decision lives in `.planning/active/sprint83_node_profile_freeze.md`
/// and this constant mirrors it (the `node_profile_frozen` test measures the
/// EMITTED schema against this set, so adding or removing a variant without
/// consciously touching both the freeze artefact and this mirror is red).
///
/// `sequence` (named by kickoff D4 among composition families) is NOT a node
/// kind: in a data-flow graph, sequencing IS the edge wiring. The freeze
/// artefact records that reconciliation.
pub const FROZEN_NODE_KINDS: [&str; 18] = [
    "input",
    "transform",
    "model.invoke",
    "agent.invoke",
    "agent.as_tool",
    "handoff",
    "capability.call",
    "knowledge.query",
    "switch",
    "classifier",
    "parallel",
    "map",
    "join",
    "repeat",
    "human.review",
    "subworkflow",
    "wait.signal",
    "output",
];

/// A pinned reference to an agent definition used by this graph.
///
/// `blake3` is the `PACK.lock` pin of the agent-definition file. Invoking
/// nodes name agents by `agent_id`; the closure "every invoked agent is
/// declared here" is a Phase H refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AgentRef.v1")]
pub struct AgentRef {
    /// Identity of the agent definition, unique within the pack.
    pub agent_id: String,
    /// BLAKE3 hex of the pinned agent-definition file bytes.
    pub blake3: String,
}

/// A pinned reference to a child workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SubworkflowRef.v1")]
pub struct SubworkflowRef {
    /// Identity of the child workflow, unique within the pack.
    pub workflow_id: String,
    /// BLAKE3 hex of the pinned child workflow file bytes.
    pub blake3: String,
}

/// A pinned reference to a knowledge corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CorpusRef.v1")]
pub struct CorpusRef {
    /// Identity of the corpus, unique within the pack.
    pub corpus_id: String,
    /// BLAKE3 hex of the pinned corpus index/manifest bytes.
    pub blake3: String,
}

/// One directed data-flow edge between two node ports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "WorkflowEdge.v1")]
pub struct WorkflowEdge {
    /// Source node id.
    pub from: String,
    /// Source port. Routing nodes emit on their static labels (plus the
    /// reserved `default`); every other node emits on `out`.
    pub from_port: String,
    /// Destination node id.
    pub to: String,
    /// Destination port (`in` unless the destination declares otherwise).
    pub to_port: String,
}

/// `input` — validate and introduce the typed workflow input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "InputNode.v1")]
pub struct InputNode {
    /// Pinned schema the incoming payload must satisfy.
    pub schema_ref: SchemaRef,
}

/// `transform` — pure declarative field mapping. No code, no expressions:
/// each output field names the input field it copies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "TransformNode.v1")]
pub struct TransformNode {
    /// Pinned schema of the accepted payload.
    pub input_schema_ref: SchemaRef,
    /// Pinned schema of the produced payload.
    pub output_schema_ref: SchemaRef,
    /// Output field name → input field name, copied verbatim.
    pub field_map: BTreeMap<String, String>,
}

/// `model.invoke` — one structured LLM call, no autonomous loop, no tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ModelInvokeNode.v1")]
pub struct ModelInvokeNode {
    /// Pinned prompt artefact.
    pub prompt_ref: PromptRef,
    /// Pinned schema of the payload rendered into the prompt.
    pub input_schema_ref: SchemaRef,
    /// Pinned schema the model output must satisfy.
    pub output_schema_ref: SchemaRef,
    /// Provider/model policy for this single call.
    pub model_policy: ModelPolicy,
    /// Budget for this single call.
    pub budget: BudgetPolicy,
}

/// `agent.invoke` — run a declared agent with bounded tools/handoffs.
///
/// The ONLY override surface is the effective budget: no field exists here
/// that could widen the agent's tools, handoffs, context or memory beyond
/// its definition, so an invocation can only ATTENUATE (preflight S7-B; the
/// numeric budget<=definition comparison is Phase H).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AgentInvokeNode.v1")]
pub struct AgentInvokeNode {
    /// Id of the invoked agent; must be declared in the graph's `agent_refs`
    /// (Phase H refusal).
    pub agent_id: String,
    /// Effective budget of THIS invocation; must not exceed the agent
    /// definition's budget nor the workflow budget (Phase H).
    pub budget: BudgetPolicy,
}

/// `agent.as_tool` — call a specialist agent as a tool: the parent keeps
/// synthesis and guardrails, and the specialist's output RETURNS to the
/// parent flow (unlike [`HandoffNode`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AgentAsToolNode.v1")]
pub struct AgentAsToolNode {
    /// Id of the specialist agent; must be declared in `agent_refs`.
    pub agent_id: String,
    /// Pinned schema of the bounded sub-task payload.
    pub input_schema_ref: SchemaRef,
    /// Pinned schema of the result returned to the parent.
    pub output_schema_ref: SchemaRef,
    /// Effective budget of this call.
    pub budget: BudgetPolicy,
}

/// `handoff` — explicit transfer of control with a filtered, schematised
/// payload. The handoff never implicitly transmits the giver's whole
/// history, memory or tools (D4): both the payload schema and the context
/// allowlist are REQUIRED. A handoff has NO return port — control continues
/// downstream at the receiver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "HandoffNode.v1")]
pub struct HandoffNode {
    /// Id of the receiving agent. When the upstream node is an agent
    /// invocation, the giver's definition must list this target in
    /// `allowed_handoffs` (Phase H refusal).
    pub target_agent_id: String,
    /// Pinned schema of the transferred payload. REQUIRED — never optional.
    pub payload_schema_ref: SchemaRef,
    /// Context allowlist accompanying the payload. REQUIRED — an absent
    /// filter meaning "everything" is exactly the default this format
    /// rejects (preflight B3).
    pub context_policy: ContextPolicy,
}

/// `capability.call` — one authorised native/MCP/business activity.
///
/// A capability is a REFERENCE into the pack's `requested_capabilities`;
/// requesting is not obtaining (the host authority intersects elsewhere).
/// No endpoint, URL, host, command or inline code exists in this node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CapabilityCallNode.v1")]
pub struct CapabilityCallNode {
    /// Logical capability reference; must appear in the graph's
    /// `capability_requests` (Phase H refusal).
    pub capability_ref: String,
    /// Pinned schema of the call arguments.
    pub args_schema_ref: SchemaRef,
    /// Pinned schema of the call result.
    pub output_schema_ref: SchemaRef,
}

/// `knowledge.query` — search a PINNED corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "KnowledgeQueryNode.v1")]
pub struct KnowledgeQueryNode {
    /// Pinned corpus to search.
    pub corpus_ref: CorpusRef,
    /// Pinned schema of the query payload.
    pub query_schema_ref: SchemaRef,
    /// Pinned schema of the result payload.
    pub output_schema_ref: SchemaRef,
}

/// `switch` — deterministic routing on already-validated data. The label set
/// is STATIC; a value matching no label leaves on the reserved `default`
/// port. No destination can be minted at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SwitchNode.v1")]
pub struct SwitchNode {
    /// Name of the payload field whose string value is matched against the
    /// labels.
    pub discriminant_field: String,
    /// The closed set of output labels (= output ports). The reserved
    /// `default` port exists in addition and must be wired.
    pub branch_labels: Vec<String>,
}

/// `classifier` — a model PROPOSES a label from a closed set; the runtime
/// validates it. A proposal outside `branch_labels` routes to the reserved
/// `default` port deterministically — prompt injection can influence WHICH
/// declared branch is taken, never create a new destination (§19.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ClassifierNode.v1")]
pub struct ClassifierNode {
    /// Pinned prompt artefact for the classification call.
    pub prompt_ref: PromptRef,
    /// Provider/model policy for the call.
    pub model_policy: ModelPolicy,
    /// The closed set of proposable labels (= output ports), plus the
    /// reserved `default` port.
    pub branch_labels: Vec<String>,
    /// Budget for the classification call.
    pub budget: BudgetPolicy,
}

/// `parallel` — static concurrent branches, one output port per label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ParallelNode.v1")]
pub struct ParallelNode {
    /// The static branch labels (= output ports), all activated.
    pub branch_labels: Vec<String>,
}

/// `map` — bounded dynamic fan-out over a collection, each item processed by
/// a PINNED subworkflow. `max_cardinality` is a constant of the signed
/// format — the runtime collection size is data, the bound is not; a
/// collection larger than the bound is a runtime refusal, never a bound
/// stretch. The fan-in is intrinsic: the node emits the aggregated results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "MapNode.v1")]
pub struct MapNode {
    /// Name of the payload field holding the collection to fan out over.
    pub collection_field: String,
    /// Structural upper bound on the fan-out, ≥ 1, constant in the signed
    /// graph. Never an expression, never a model output.
    pub max_cardinality: NonZeroU32,
    /// Pinned child workflow applied to each item.
    pub body_subworkflow_ref: SubworkflowRef,
}

/// The explicit fan-in policy of a [`JoinNode`].
///
/// Internally tagged like the node roster, with `deny_unknown_fields`
/// variant bodies, for the same reason: one canonical spelling, no tolerated
/// junk next to a policy that decides when a barrier opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode")]
pub enum JoinPolicy {
    /// Wait for ALL declared sources.
    #[serde(rename = "all")]
    All(JoinAllPolicy),
    /// First declared source to arrive wins; the rest are discarded. The
    /// exact fit for exclusive routing branches.
    #[serde(rename = "any")]
    Any(JoinAnyPolicy),
    /// Wait for at least `quorum` of the declared sources.
    #[serde(rename = "quorum")]
    Quorum(JoinQuorumPolicy),
}

/// Body of [`JoinPolicy::All`] (no parameters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "JoinAllPolicy.v1")]
pub struct JoinAllPolicy {}

/// Body of [`JoinPolicy::Any`] (no parameters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "JoinAnyPolicy.v1")]
pub struct JoinAnyPolicy {}

/// Body of [`JoinPolicy::Quorum`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "JoinQuorumPolicy.v1")]
pub struct JoinQuorumPolicy {
    /// Minimum number of sources that must arrive, ≥ 1.
    pub quorum: NonZeroU32,
}

/// `join` — fan-in with an EXPLICIT policy over DECLARED sources.
/// Satisfiability against the upstream topology (an `all` join fed by
/// mutually-exclusive branches deadlocks a signed plan) is a Phase H
/// refusal (research §10.5 "join dont les branches et la politique sont
/// incohérentes").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "JoinNode.v1")]
pub struct JoinNode {
    /// The fan-in policy.
    pub policy: JoinPolicy,
    /// Node ids whose outputs this join waits on, statically declared.
    pub source_node_ids: Vec<String>,
}

/// `repeat` — bounded loop: the PINNED body subworkflow runs up to
/// `max_iterations` times, stopping early when the boolean
/// `until_output_flag` field of the body's output is true. The bound is a
/// constant of the signed format (structurally stronger than a global
/// recursion limit — kickoff D4 rejected alternative). This node introduces
/// NO back-edge — its body is a content-addressed pinned child — but
/// rejecting cycles spelled by arbitrary `edges` stays a Phase H compiler
/// refusal (§10.5, freeze artefact §8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "RepeatNode.v1")]
pub struct RepeatNode {
    /// Structural upper bound on iterations, ≥ 1, constant in the signed
    /// graph.
    pub max_iterations: NonZeroU32,
    /// Pinned child workflow executed each iteration.
    pub body_subworkflow_ref: SubworkflowRef,
    /// Name of the boolean field of the body's output that, when true,
    /// stops the loop before the bound. A deterministic data condition —
    /// never free-form model text.
    pub until_output_flag: String,
}

/// `human.review` — durable interruption: the HOST renders the payload and a
/// human picks one of the static decision labels (= output ports). The
/// signed HITL approval binding is Phase L1; `intent` never equals approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "HumanReviewNode.v1")]
pub struct HumanReviewNode {
    /// Pinned schema of the payload presented to the human.
    pub payload_schema_ref: SchemaRef,
    /// The closed set of decisions the human may take (= output ports).
    pub decision_labels: Vec<String>,
}

/// `subworkflow` — invoke a PINNED child plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SubworkflowNode.v1")]
pub struct SubworkflowNode {
    /// Pinned child workflow.
    pub subworkflow_ref: SubworkflowRef,
}

/// `wait.signal` — durable wait for a TYPED external signal delivered by the
/// host. The signal name is static; the payload is validated against a
/// pinned schema. No model output can synthesise a signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "WaitSignalNode.v1")]
pub struct WaitSignalNode {
    /// Static name of the awaited signal.
    pub signal_name: String,
    /// Pinned schema the signal payload must satisfy.
    pub payload_schema_ref: SchemaRef,
}

/// `output` — validate and publish a durable artefact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "OutputNode.v1")]
pub struct OutputNode {
    /// Pinned schema the published artefact must satisfy.
    pub schema_ref: SchemaRef,
    /// Name of the artefact, unique among the workflow's outputs and
    /// declared in the graph's `artifact_outputs` (Phase H refusal).
    pub artifact_name: String,
}

/// The CLOSED node roster — an internally-tagged union over the 18 frozen
/// kinds ([`FROZEN_NODE_KINDS`]). Each variant is a newtype over a dedicated
/// `deny_unknown_fields` struct, so an unknown field INSIDE a node body is a
/// parse error (the `unknown_field_rejected_for_every_kind` test measures
/// this for every kind — serde's internally-tagged deserialisation is
/// subtle enough that the property is proven, not assumed; preflight S7-A).
/// An unknown `kind` is refused by the enum itself. No `untagged`, no
/// aliases: one canonical dotted spelling per kind, because a signed JCS
/// format cannot afford two byte spellings of one meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum WorkflowNode {
    /// See [`InputNode`].
    #[serde(rename = "input")]
    Input(InputNode),
    /// See [`TransformNode`].
    #[serde(rename = "transform")]
    Transform(TransformNode),
    /// See [`ModelInvokeNode`].
    #[serde(rename = "model.invoke")]
    ModelInvoke(ModelInvokeNode),
    /// See [`AgentInvokeNode`].
    #[serde(rename = "agent.invoke")]
    AgentInvoke(AgentInvokeNode),
    /// See [`AgentAsToolNode`].
    #[serde(rename = "agent.as_tool")]
    AgentAsTool(AgentAsToolNode),
    /// See [`HandoffNode`].
    #[serde(rename = "handoff")]
    Handoff(HandoffNode),
    /// See [`CapabilityCallNode`].
    #[serde(rename = "capability.call")]
    CapabilityCall(CapabilityCallNode),
    /// See [`KnowledgeQueryNode`].
    #[serde(rename = "knowledge.query")]
    KnowledgeQuery(KnowledgeQueryNode),
    /// See [`SwitchNode`].
    #[serde(rename = "switch")]
    Switch(SwitchNode),
    /// See [`ClassifierNode`].
    #[serde(rename = "classifier")]
    Classifier(ClassifierNode),
    /// See [`ParallelNode`].
    #[serde(rename = "parallel")]
    Parallel(ParallelNode),
    /// See [`MapNode`].
    #[serde(rename = "map")]
    Map(MapNode),
    /// See [`JoinNode`].
    #[serde(rename = "join")]
    Join(JoinNode),
    /// See [`RepeatNode`].
    #[serde(rename = "repeat")]
    Repeat(RepeatNode),
    /// See [`HumanReviewNode`].
    #[serde(rename = "human.review")]
    HumanReview(HumanReviewNode),
    /// See [`SubworkflowNode`].
    #[serde(rename = "subworkflow")]
    Subworkflow(SubworkflowNode),
    /// See [`WaitSignalNode`].
    #[serde(rename = "wait.signal")]
    WaitSignal(WaitSignalNode),
    /// See [`OutputNode`].
    #[serde(rename = "output")]
    Output(OutputNode),
}

/// What happens when a node fails after its retries are exhausted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NodeFailureAction {
    /// The workflow fails terminally.
    FailWorkflow,
    /// The failure routes on the node's reserved `error` port (which must
    /// then be wired — Phase H).
    RouteErrorPort,
}

/// Workflow-wide error policy (declarative; retry semantics land in
/// Phase K).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ErrorPolicy.v1")]
pub struct ErrorPolicy {
    /// Action once a node's retries are exhausted.
    pub on_node_failure: NodeFailureAction,
    /// Retries allowed per node before `on_node_failure` applies. May be 0.
    pub max_retries_per_node: u32,
}

/// Workflow-wide timeout policy. Seconds, non-zero: a zero timeout is not a
/// disabled timeout, it is nonsense the format refuses at parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "TimeoutPolicy.v1")]
pub struct TimeoutPolicy {
    /// Wall-clock ceiling of the whole run, ≥ 1 second.
    pub workflow_timeout_seconds: NonZeroU32,
    /// Default per-node wall-clock ceiling, ≥ 1 second.
    pub default_node_timeout_seconds: NonZeroU32,
}

/// Workflow-wide cancellation policy: which DETERMINISTIC host conditions
/// cancel the run. No field routes a model output to a cancel — cancellation
/// authority is host/budget/timeout only (research §12.7; runtime honesty is
/// Phase J).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CancellationPolicy.v1")]
pub struct CancellationPolicy {
    /// Cancel when any budget dimension is exhausted.
    pub cancel_on_budget_exhausted: bool,
    /// Cancel when `workflow_timeout_seconds` is reached.
    pub cancel_on_timeout: bool,
    /// The host operator may always cancel; declaring `false` here is
    /// refused by the host (a pack cannot opt out of operator authority —
    /// enforcement is Phase J/L, the field keeps the claim explicit).
    pub allow_host_operator_cancel: bool,
}

/// One durable artefact the workflow publishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ArtifactOutput.v1")]
pub struct ArtifactOutput {
    /// Artefact name, unique in this workflow.
    pub name: String,
    /// Pinned schema of the artefact.
    pub schema_ref: SchemaRef,
    /// Id of the `output` node that publishes it.
    pub from_node: String,
}

// Workflow-language frontier (Sprint 83 Phase G, doctrine §7): the data-flow
// plan of the signed pack, read by a distinct runtime — the Rust compiler
// (Phase H), the host engine (Phases I/J) and the sandboxed Workflow Hub
// (Phase T). It is pinned BY DIGEST inside `SBFBPromptAgentPack.v1`
// (PACK.lock), never signed or gossiped on its own, so it carries no
// `// FRONTIER:` domain/version tag (that opt-in registry is for
// `DOMAIN_*_V1` families and inventing one here would trip the frozen census
// of 29). Its machine contract is the generated schema snapshot pinned by
// `contracts::all_contract_schemas` (identity 2/7, len==7) plus the four
// committed workflow goldens; the GUIDE + llms.txt closure is deferred to
// the S83 wrap-up phase per AM-3.
/// `SBFBWorkflowGraph.v1` — the signed data-flow plan (D4). See the module
/// doc for the invariants the shape carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBWorkflowGraph.v1")]
pub struct SbfbWorkflowGraph {
    /// Stable logical identity of the workflow.
    pub workflow_id: String,
    /// Monotonic version of THIS workflow identity.
    pub workflow_version: u32,
    /// Human-facing description of what the workflow does.
    pub description: String,
    /// Pinned schema of the workflow input.
    pub input_schema_ref: SchemaRef,
    /// Pinned schema of the workflow output.
    pub output_schema_ref: SchemaRef,
    /// Ids of the nodes where execution starts. Non-emptiness and
    /// reachability are Phase H refusals.
    pub entrypoints: Vec<String>,
    /// The nodes, keyed by node id — id uniqueness is structural.
    pub nodes: BTreeMap<String, WorkflowNode>,
    /// The static data-flow wiring.
    pub edges: Vec<WorkflowEdge>,
    /// Workflow-wide error policy.
    pub error_policy: ErrorPolicy,
    /// Workflow-wide timeout policy.
    pub timeout_policy: TimeoutPolicy,
    /// The workflow's total budget; every nested budget must fit under it
    /// (Phase H).
    pub budget_policy: BudgetPolicy,
    /// Workflow-wide cancellation policy.
    pub cancellation_policy: CancellationPolicy,
    /// Logical capabilities this plan requests; every `capability.call`
    /// must reference one of them (Phase H). Requesting is never obtaining.
    pub capability_requests: Vec<String>,
    /// The agents this plan may invoke, pinned by digest.
    pub agent_refs: Vec<AgentRef>,
    /// The child workflows this plan may run, pinned by digest.
    pub subworkflow_refs: Vec<SubworkflowRef>,
    /// The durable artefacts this plan publishes.
    pub artifact_outputs: Vec<ArtifactOutput>,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::agent_definition::SbfbAgentDefinition;

    const GOLDEN_NAMES: [&str; 4] = ["factory", "ideas", "conception", "babel"];

    fn src_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    fn golden_path(name: &str) -> PathBuf {
        src_dir().join(format!("{name}.workflow.golden.json"))
    }

    /// The per-golden pinned artefact store
    /// (`src/workflow_golden_artifacts/<name>/`).
    struct ArtifactStore {
        dir: PathBuf,
    }

    impl ArtifactStore {
        fn open(name: &str) -> Self {
            Self {
                dir: src_dir().join("workflow_golden_artifacts").join(name),
            }
        }

        fn file(&self, category: &str, id: &str, ext: &str) -> PathBuf {
            self.dir.join(category).join(format!("{id}.{ext}"))
        }

        /// The `PACK.lock` pin of one artefact file: BLAKE3 hex of the
        /// checkout-newline-canonicalised bytes. A real pack hashes raw
        /// archive bytes (archives never ride git's newline translation);
        /// committed fixtures do, so they reuse the
        /// [`crate::factory_profile`] source-digest canonicalisation.
        fn pin(&self, category: &str, id: &str, ext: &str) -> String {
            let path = self.file(category, id, ext);
            let bytes = std::fs::read(&path)
                .unwrap_or_else(|e| panic!("golden artefact {} must exist ({e})", path.display()));
            blake3::hash(&crate::factory_profile::canonicalise(&bytes))
                .to_hex()
                .to_string()
        }

        /// Ids of the artefacts of one category, by filename suffix, sorted.
        fn ids(&self, category: &str, suffix: &str) -> Vec<String> {
            let dir = self.dir.join(category);
            if !dir.exists() {
                return Vec::new();
            }
            let mut out: Vec<String> = std::fs::read_dir(&dir)
                .expect("store category readable")
                .map(|entry| entry.expect("dir entry").file_name())
                .filter_map(|n| {
                    n.to_str()
                        .and_then(|n| n.strip_suffix(suffix))
                        .map(str::to_string)
                })
                .collect();
            out.sort();
            out
        }
    }

    fn parse_json(path: &Path) -> serde_json::Value {
        let bytes =
            std::fs::read(path).unwrap_or_else(|e| panic!("{} must exist ({e})", path.display()));
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("{} must be valid JSON ({e})", path.display()))
    }

    fn parse_graph(path: &Path) -> (serde_json::Value, SbfbWorkflowGraph) {
        let value = parse_json(path);
        let graph = serde_json::from_value(value.clone())
            .unwrap_or_else(|e| panic!("{} must parse strictly ({e})", path.display()));
        (value, graph)
    }

    fn parse_agent(path: &Path) -> (serde_json::Value, SbfbAgentDefinition) {
        let value = parse_json(path);
        let agent = serde_json::from_value(value.clone())
            .unwrap_or_else(|e| panic!("{} must parse strictly ({e})", path.display()));
        (value, agent)
    }

    fn load_agents(store: &ArtifactStore) -> Vec<SbfbAgentDefinition> {
        store
            .ids("agents", ".agent.json")
            .iter()
            .map(|id| parse_agent(&store.file("agents", id, "agent.json")).1)
            .collect()
    }

    fn load_subworkflows(store: &ArtifactStore) -> Vec<SbfbWorkflowGraph> {
        store
            .ids("subworkflows", ".workflow.json")
            .iter()
            .map(|id| parse_graph(&store.file("subworkflows", id, "workflow.json")).1)
            .collect()
    }

    fn write_pretty<T: serde::Serialize>(path: &Path, value: &T) {
        let text = serde_json::to_string_pretty(&serde_json::to_value(value).expect("serialises"))
            .expect("pretty-prints");
        std::fs::write(path, format!("{text}\n")).expect("golden written");
    }

    fn fill_schema_ref(r: &mut SchemaRef, s: &ArtifactStore) {
        r.blake3 = s.pin("schemas", &r.schema_id, "schema.json");
    }

    fn fill_prompt_ref(r: &mut PromptRef, s: &ArtifactStore) {
        r.blake3 = s.pin("prompts", &r.prompt_id, "md");
    }

    fn fill_corpus_ref(r: &mut CorpusRef, s: &ArtifactStore) {
        r.blake3 = s.pin("corpora", &r.corpus_id, "md");
    }

    fn fill_agent_ref(r: &mut AgentRef, s: &ArtifactStore) {
        r.blake3 = s.pin("agents", &r.agent_id, "agent.json");
    }

    fn fill_subworkflow_ref(r: &mut SubworkflowRef, s: &ArtifactStore) {
        r.blake3 = s.pin("subworkflows", &r.workflow_id, "workflow.json");
    }

    fn fill_node(node: &mut WorkflowNode, s: &ArtifactStore) {
        match node {
            WorkflowNode::Input(n) => fill_schema_ref(&mut n.schema_ref, s),
            WorkflowNode::Transform(n) => {
                fill_schema_ref(&mut n.input_schema_ref, s);
                fill_schema_ref(&mut n.output_schema_ref, s);
            }
            WorkflowNode::ModelInvoke(n) => {
                fill_prompt_ref(&mut n.prompt_ref, s);
                fill_schema_ref(&mut n.input_schema_ref, s);
                fill_schema_ref(&mut n.output_schema_ref, s);
            }
            WorkflowNode::AgentInvoke(_) => {}
            WorkflowNode::AgentAsTool(n) => {
                fill_schema_ref(&mut n.input_schema_ref, s);
                fill_schema_ref(&mut n.output_schema_ref, s);
            }
            WorkflowNode::Handoff(n) => fill_schema_ref(&mut n.payload_schema_ref, s),
            WorkflowNode::CapabilityCall(n) => {
                fill_schema_ref(&mut n.args_schema_ref, s);
                fill_schema_ref(&mut n.output_schema_ref, s);
            }
            WorkflowNode::KnowledgeQuery(n) => {
                fill_corpus_ref(&mut n.corpus_ref, s);
                fill_schema_ref(&mut n.query_schema_ref, s);
                fill_schema_ref(&mut n.output_schema_ref, s);
            }
            WorkflowNode::Switch(_) | WorkflowNode::Parallel(_) | WorkflowNode::Join(_) => {}
            WorkflowNode::Classifier(n) => fill_prompt_ref(&mut n.prompt_ref, s),
            WorkflowNode::Map(n) => fill_subworkflow_ref(&mut n.body_subworkflow_ref, s),
            WorkflowNode::Repeat(n) => fill_subworkflow_ref(&mut n.body_subworkflow_ref, s),
            WorkflowNode::HumanReview(n) => fill_schema_ref(&mut n.payload_schema_ref, s),
            WorkflowNode::Subworkflow(n) => fill_subworkflow_ref(&mut n.subworkflow_ref, s),
            WorkflowNode::WaitSignal(n) => fill_schema_ref(&mut n.payload_schema_ref, s),
            WorkflowNode::Output(n) => fill_schema_ref(&mut n.schema_ref, s),
        }
    }

    fn fill_graph(g: &mut SbfbWorkflowGraph, s: &ArtifactStore) {
        fill_schema_ref(&mut g.input_schema_ref, s);
        fill_schema_ref(&mut g.output_schema_ref, s);
        for node in g.nodes.values_mut() {
            fill_node(node, s);
        }
        for r in &mut g.agent_refs {
            fill_agent_ref(r, s);
        }
        for r in &mut g.subworkflow_refs {
            fill_subworkflow_ref(r, s);
        }
        for a in &mut g.artifact_outputs {
            fill_schema_ref(&mut a.schema_ref, s);
        }
    }

    fn fill_agent(a: &mut SbfbAgentDefinition, s: &ArtifactStore) {
        fill_prompt_ref(&mut a.instructions_ref, s);
        fill_schema_ref(&mut a.input_schema_ref, s);
        fill_schema_ref(&mut a.output_schema_ref, s);
        for t in &mut a.tool_refs {
            t.blake3 = s.pin("tools", &t.tool_id, "md");
        }
        for g in &mut a.guardrail_refs {
            g.blake3 = s.pin("guardrails", &g.guardrail_id, "md");
        }
    }

    /// The serde-authoritative kind string of a node.
    fn kind_of(node: &WorkflowNode) -> String {
        serde_json::to_value(node).expect("serialises")["kind"]
            .as_str()
            .expect("kind is a string")
            .to_string()
    }

    /// G.4 exit condition: the four goldens parse STRICTLY into the typed
    /// contracts and round-trip losslessly (compared as `serde_json::Value`,
    /// never as strings — the workspace `preserve_order` feature via
    /// ollama-rs→schemars changes member order between build graphs).
    ///
    /// Equivalence note (preflight B5 decision): the emitted JSON Schemas
    /// deliberately carry NO value constraint beyond types / required /
    /// closed properties / the NonZero minimums serde itself enforces, so
    /// the strict typed parse IS schema conformance. The negative controls
    /// below keep this test honest.
    #[test]
    fn four_goldens_validate() {
        for name in GOLDEN_NAMES {
            let store = ArtifactStore::open(name);
            let (value, graph) = parse_graph(&golden_path(name));
            assert_eq!(
                serde_json::to_value(&graph).expect("serialises"),
                value,
                "{name}: golden must round-trip losslessly"
            );
            assert!(!graph.nodes.is_empty(), "{name}: golden must not be empty");
            for id in store.ids("agents", ".agent.json") {
                let path = store.file("agents", &id, "agent.json");
                let (avalue, agent) = parse_agent(&path);
                assert_eq!(
                    serde_json::to_value(&agent).expect("serialises"),
                    avalue,
                    "{name}/{id}: agent must round-trip losslessly"
                );
                assert_eq!(agent.agent_id, id, "{name}: agent file name == agent_id");
            }
            for id in store.ids("subworkflows", ".workflow.json") {
                let path = store.file("subworkflows", &id, "workflow.json");
                let (svalue, sub) = parse_graph(&path);
                assert_eq!(
                    serde_json::to_value(&sub).expect("serialises"),
                    svalue,
                    "{name}/{id}: subworkflow must round-trip losslessly"
                );
                assert_eq!(
                    sub.workflow_id, id,
                    "{name}: subworkflow file name == workflow_id"
                );
            }
        }
    }

    /// Every digest pin inside the goldens resolves to the committed artefact
    /// bytes. STRICT: a missing artefact or a stale pin is a failure.
    /// Refresh after an intentional artefact change with
    /// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core golden_pins_resolve`
    /// run under the WORKSPACE build graph (preserve_order keeps the
    /// hand-authored member order when files are rewritten).
    #[test]
    fn golden_pins_resolve() {
        let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();
        for name in GOLDEN_NAMES {
            let store = ArtifactStore::open(name);
            // Leaf-to-root: agents first (their pins feed the graph's
            // agent_refs), then subworkflows, then the top graph.
            for id in store.ids("agents", ".agent.json") {
                let path = store.file("agents", &id, "agent.json");
                let (_, parsed) = parse_agent(&path);
                let mut refilled = parsed.clone();
                fill_agent(&mut refilled, &store);
                if update {
                    write_pretty(&path, &refilled);
                } else {
                    assert_eq!(refilled, parsed, "stale pins in {}", path.display());
                }
            }
            for id in store.ids("subworkflows", ".workflow.json") {
                let path = store.file("subworkflows", &id, "workflow.json");
                let (_, parsed) = parse_graph(&path);
                let mut refilled = parsed.clone();
                fill_graph(&mut refilled, &store);
                if update {
                    write_pretty(&path, &refilled);
                } else {
                    assert_eq!(refilled, parsed, "stale pins in {}", path.display());
                }
            }
            let path = golden_path(name);
            let (_, parsed) = parse_graph(&path);
            let mut refilled = parsed.clone();
            fill_graph(&mut refilled, &store);
            if update {
                write_pretty(&path, &refilled);
            } else {
                assert_eq!(refilled, parsed, "stale pins in {}", path.display());
            }
        }
    }

    /// The negative controls that keep `four_goldens_validate` non-vacuous
    /// (preflight B5/M3): a mutated golden MUST go red, and the unmutated
    /// value green — the control of the control.
    #[test]
    fn golden_negative_controls_reject() {
        let (value, _) = parse_graph(&golden_path("factory"));
        serde_json::from_value::<SbfbWorkflowGraph>(value.clone())
            .expect("control: the unmutated golden parses");

        // 1. Unknown top-level member (a reserved presentation name, at that).
        let mut m = value.clone();
        m.as_object_mut()
            .unwrap()
            .insert("layout".into(), serde_json::json!({}));
        serde_json::from_value::<SbfbWorkflowGraph>(m)
            .expect_err("unknown top-level field must be refused");

        // 2. Unknown field smuggled into a node body.
        let mut m = value.clone();
        m["nodes"]["phase-brief"]
            .as_object_mut()
            .unwrap()
            .insert("smuggled".into(), serde_json::json!(1));
        serde_json::from_value::<SbfbWorkflowGraph>(m)
            .expect_err("unknown node field must be refused");

        // 3. Unknown node kind (fail-closed, the assumed divergence from the
        // tolerant feed raw-op).
        let mut m = value.clone();
        m["nodes"]["phase-brief"]["kind"] = serde_json::json!("telepathy");
        serde_json::from_value::<SbfbWorkflowGraph>(m)
            .expect_err("unknown node kind must be refused");

        // 4. Missing kind.
        let mut m = value.clone();
        m["nodes"]["phase-brief"]
            .as_object_mut()
            .unwrap()
            .remove("kind");
        serde_json::from_value::<SbfbWorkflowGraph>(m).expect_err("missing kind must be refused");

        // 5. Missing workflow budget — never an unlimited default.
        let mut m = value.clone();
        m.as_object_mut().unwrap().remove("budget_policy");
        serde_json::from_value::<SbfbWorkflowGraph>(m)
            .expect_err("missing budget_policy must be refused");

        // 6. Zero loop bound — the structural bound is >= 1 by TYPE, so
        // serde enforces exactly what the emitted schema's minimum says.
        let mut m = value.clone();
        m["nodes"]["codex-rounds"]["max_iterations"] = serde_json::json!(0);
        serde_json::from_value::<SbfbWorkflowGraph>(m)
            .expect_err("zero repeat bound must be refused");

        // 7. Null — the pack canonicalisation door rejects null; the typed
        // format never produces one.
        let mut m = value.clone();
        m["description"] = serde_json::Value::Null;
        serde_json::from_value::<SbfbWorkflowGraph>(m).expect_err("null must be refused");
    }

    /// `handoff` and `agent.as_tool` are structurally DISTINCT primitives
    /// (D4, preflight B3): a handoff transfers control (payload schema +
    /// context allowlist REQUIRED, no return schema), agent.as_tool returns
    /// to the parent (output schema REQUIRED).
    #[test]
    fn handoff_not_agent_as_tool() {
        let handoff = serde_json::json!({
            "kind": "handoff",
            "target_agent_id": "someone",
            "payload_schema_ref": {"schema_id": "p", "blake3": ""},
            "context_policy": {"allowed_context_fields": []}
        });
        serde_json::from_value::<WorkflowNode>(handoff.clone()).expect("handoff parses");

        let mut with_return = handoff.clone();
        with_return.as_object_mut().unwrap().insert(
            "output_schema_ref".into(),
            serde_json::json!({"schema_id": "o", "blake3": ""}),
        );
        serde_json::from_value::<WorkflowNode>(with_return)
            .expect_err("a handoff with a return schema is not a handoff");

        let mut without_filter = handoff.clone();
        without_filter
            .as_object_mut()
            .unwrap()
            .remove("context_policy");
        serde_json::from_value::<WorkflowNode>(without_filter)
            .expect_err("the context filter is REQUIRED — absence never means everything");

        let mut without_payload = handoff.clone();
        without_payload
            .as_object_mut()
            .unwrap()
            .remove("payload_schema_ref");
        serde_json::from_value::<WorkflowNode>(without_payload)
            .expect_err("the payload schema is REQUIRED");

        let as_tool = serde_json::json!({
            "kind": "agent.as_tool",
            "agent_id": "someone",
            "input_schema_ref": {"schema_id": "i", "blake3": ""},
            "output_schema_ref": {"schema_id": "o", "blake3": ""},
            "budget": {"max_total_tokens": 1, "max_cost_micro_usd": 1,
                        "max_wall_clock_seconds": 1, "max_turns": 1}
        });
        serde_json::from_value::<WorkflowNode>(as_tool.clone()).expect("agent.as_tool parses");
        let mut tool_without_return = as_tool.clone();
        tool_without_return
            .as_object_mut()
            .unwrap()
            .remove("output_schema_ref");
        serde_json::from_value::<WorkflowNode>(tool_without_return)
            .expect_err("agent.as_tool RETURNS to the parent — its output schema is required");

        // In the goldens: the factory handoff's single outgoing edge is the
        // control transfer (`to`), while the ideas agent.as_tool returns on
        // `out` into the parent's own synthesis node.
        let (_, factory) = parse_graph(&golden_path("factory"));
        let handoff_out: Vec<_> = factory
            .edges
            .iter()
            .filter(|e| e.from == "handoff-to-review")
            .collect();
        assert_eq!(
            handoff_out.len(),
            1,
            "a handoff transfers control exactly once"
        );
        assert_eq!(handoff_out[0].from_port, "to");

        let (_, ideas) = parse_graph(&golden_path("ideas"));
        assert!(
            ideas
                .edges
                .iter()
                .any(|e| e.from == "repro-check" && e.from_port == "out" && e.to == "synthesise"),
            "the agent.as_tool result must return into the parent flow"
        );
    }

    fn collect_kind_consts(v: &serde_json::Value, out: &mut BTreeSet<String>) {
        if let Some(obj) = v.as_object() {
            if let Some(kind) = obj.get("properties").and_then(|p| p.get("kind")) {
                if let Some(c) = kind.get("const").and_then(|c| c.as_str()) {
                    out.insert(c.to_string());
                }
                if let Some(one) = kind.get("enum").and_then(|e| e.as_array())
                    && let [serde_json::Value::String(only)] = one.as_slice()
                {
                    out.insert(only.clone());
                }
            }
            for child in obj.values() {
                collect_kind_consts(child, out);
            }
        } else if let Some(arr) = v.as_array() {
            for child in arr {
                collect_kind_consts(child, out);
            }
        }
    }

    /// The node profile is FROZEN (dated decision in
    /// `.planning/active/sprint83_node_profile_freeze.md`, mirrored by
    /// [`FROZEN_NODE_KINDS`]). This is a SET check against the kind tags of
    /// the EMITTED schema — adding or removing one enum variant goes red
    /// without touching this test (preflight M7: never `count == 18`).
    #[test]
    fn node_profile_frozen() {
        let frozen: BTreeSet<String> = FROZEN_NODE_KINDS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            frozen.len(),
            FROZEN_NODE_KINDS.len(),
            "the frozen mirror must be duplicate-free"
        );

        let schema =
            serde_json::to_value(schemars::schema_for!(WorkflowNode)).expect("schema serialises");
        let mut emitted = BTreeSet::new();
        collect_kind_consts(&schema, &mut emitted);
        assert_eq!(
            emitted, frozen,
            "the emitted node-kind roster diverged from the freeze \
             (sprint83_node_profile_freeze.md + FROZEN_NODE_KINDS)"
        );
    }

    fn collect_property_names(v: &serde_json::Value, out: &mut BTreeSet<String>) {
        if let Some(obj) = v.as_object() {
            for (key, child) in obj {
                if key == "properties"
                    && let Some(props) = child.as_object()
                {
                    for name in props.keys() {
                        out.insert(name.clone());
                    }
                }
                collect_property_names(child, out);
            }
        } else if let Some(arr) = v.as_array() {
            for child in arr {
                collect_property_names(child, out);
            }
        }
    }

    /// No semantic field of the two contracts may use a name the pack
    /// RESERVES for presentation (preflight A9/M4): `graph_semantic_view`
    /// strips those names at every depth, so a colliding field would make
    /// two different graphs share one semantic digest.
    #[test]
    fn schema_property_names_disjoint_from_presentation_keys() {
        let mut names = BTreeSet::new();
        collect_property_names(
            &serde_json::to_value(schemars::schema_for!(SbfbWorkflowGraph)).unwrap(),
            &mut names,
        );
        collect_property_names(
            &serde_json::to_value(schemars::schema_for!(SbfbAgentDefinition)).unwrap(),
            &mut names,
        );
        assert!(!names.is_empty(), "walk must find the contract fields");
        for key in crate::pack::PRESENTATION_KEYS {
            assert!(
                !names.contains(key),
                "field name `{key}` collides with pack::PRESENTATION_KEYS — \
                 graph_semantic_view would silently strip it"
            );
        }
    }

    /// Every golden flows UNCHANGED through the committed pack machinery
    /// (preflight M5): the semantic view is the identity (no member is
    /// presentation-reserved) and the presentation view is empty. The
    /// canvas document model is an ENVELOPE around the graph (freeze
    /// artefact §document-model); the semantic graph itself never carries
    /// presentation members.
    #[test]
    fn goldens_pass_pack_semantic_view_unchanged() {
        for name in GOLDEN_NAMES {
            let store = ArtifactStore::open(name);
            let mut documents = vec![parse_json(&golden_path(name))];
            for id in store.ids("subworkflows", ".workflow.json") {
                documents.push(parse_json(&store.file(
                    "subworkflows",
                    &id,
                    "workflow.json",
                )));
            }
            for value in documents {
                assert_eq!(
                    crate::pack::graph_semantic_view(&value),
                    value,
                    "{name}: a semantic golden must be presentation-free"
                );
                assert_eq!(
                    crate::pack::presentation_view(&value),
                    serde_json::json!({}),
                    "{name}: no presentation member may exist in a semantic golden"
                );
            }
        }
    }

    /// Richness floor (preflight B5): each golden exercises at least one
    /// handoff whose upstream agent allowlists the target, at least one
    /// bounded repeat and at least one capability.call; together the four
    /// goldens exercise the ENTIRE frozen roster — the freeze is grounded in
    /// exercised nodes, not in a list nobody used.
    #[test]
    fn golden_richness_floor() {
        let mut exercised: BTreeSet<String> = BTreeSet::new();
        for name in GOLDEN_NAMES {
            let store = ArtifactStore::open(name);
            let agents = load_agents(&store);
            let (_, top) = parse_graph(&golden_path(name));
            let mut graphs = vec![top];
            graphs.extend(load_subworkflows(&store));

            let mut allowlisted_handoff = false;
            let mut bounded_repeat = false;
            let mut capability_call = false;

            for g in &graphs {
                assert!(
                    !g.entrypoints.is_empty(),
                    "{name}: entrypoints must not be empty"
                );
                assert!(!g.edges.is_empty(), "{name}: edges must not be empty");
                for (id, node) in &g.nodes {
                    exercised.insert(kind_of(node));
                    match node {
                        WorkflowNode::Repeat(_) => bounded_repeat = true,
                        WorkflowNode::CapabilityCall(c) => {
                            capability_call = true;
                            assert!(
                                g.capability_requests.contains(&c.capability_ref),
                                "{name}/{id}: capability.call must reference a requested capability"
                            );
                        }
                        WorkflowNode::Handoff(h) => {
                            for e in g.edges.iter().filter(|e| &e.to == id) {
                                let upstream = g.nodes.get(&e.from).unwrap_or_else(|| {
                                    panic!("{name}: edge from unknown node {}", e.from)
                                });
                                if let WorkflowNode::AgentInvoke(ai) = upstream {
                                    let giver = agents
                                        .iter()
                                        .find(|a| a.agent_id == ai.agent_id)
                                        .unwrap_or_else(|| {
                                            panic!("{name}: giver agent {} must exist", ai.agent_id)
                                        });
                                    assert!(
                                        giver.allowed_handoffs.contains(&h.target_agent_id),
                                        "{name}/{id}: handoff target {} not allowlisted by giver {}",
                                        h.target_agent_id,
                                        giver.agent_id
                                    );
                                    allowlisted_handoff = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            assert!(
                allowlisted_handoff,
                "{name}: >= 1 handoff must have an allowlisting agent upstream"
            );
            assert!(bounded_repeat, "{name}: >= 1 bounded repeat required");
            assert!(capability_call, "{name}: >= 1 capability.call required");
        }
        let frozen: BTreeSet<String> = FROZEN_NODE_KINDS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            exercised, frozen,
            "the four goldens together must exercise the entire frozen roster"
        );
    }

    /// Serde's internally-tagged deserialisation is subtle: prove, kind by
    /// kind over REAL golden exemplars, that an unknown field inside a node
    /// body is rejected (preflight S7-A — measured, not assumed).
    #[test]
    fn unknown_field_rejected_for_every_kind() {
        let mut exemplars: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        for name in GOLDEN_NAMES {
            let store = ArtifactStore::open(name);
            let mut documents = vec![parse_json(&golden_path(name))];
            for id in store.ids("subworkflows", ".workflow.json") {
                documents.push(parse_json(&store.file(
                    "subworkflows",
                    &id,
                    "workflow.json",
                )));
            }
            for doc in documents {
                for node in doc["nodes"].as_object().expect("nodes object").values() {
                    let kind = node["kind"].as_str().expect("kind").to_string();
                    exemplars.entry(kind).or_insert_with(|| node.clone());
                }
            }
        }
        let frozen: BTreeSet<String> = FROZEN_NODE_KINDS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            exemplars.keys().cloned().collect::<BTreeSet<_>>(),
            frozen,
            "every frozen kind must have a golden exemplar"
        );
        for (kind, exemplar) in exemplars {
            serde_json::from_value::<WorkflowNode>(exemplar.clone())
                .unwrap_or_else(|e| panic!("control: {kind} exemplar parses ({e})"));
            let mut mutated = exemplar;
            mutated
                .as_object_mut()
                .unwrap()
                .insert("smuggled".into(), serde_json::json!(1));
            serde_json::from_value::<WorkflowNode>(mutated)
                .map(|_| ())
                .expect_err(&format!("kind {kind} must reject an unknown field"));
        }
    }

    /// [`JoinPolicy`] is as strict as the node roster: canonical spellings,
    /// no junk members, non-zero quorum.
    #[test]
    fn join_policy_variants_are_strict() {
        for ok in [
            serde_json::json!({"mode": "all"}),
            serde_json::json!({"mode": "any"}),
            serde_json::json!({"mode": "quorum", "quorum": 2}),
        ] {
            serde_json::from_value::<JoinPolicy>(ok).expect("valid policy parses");
        }
        for bad in [
            serde_json::json!({"mode": "all", "quorum": 2}),
            serde_json::json!({"mode": "quorum", "quorum": 0}),
            serde_json::json!({"mode": "quorum"}),
            serde_json::json!({"mode": "majority"}),
        ] {
            serde_json::from_value::<JoinPolicy>(bad.clone())
                .map(|_| ())
                .expect_err(&format!("{bad} must be refused"));
        }
    }

    /// Is `to` reachable from `from` over the edges, never stepping on
    /// `excluded`?
    fn reachable_without(g: &SbfbWorkflowGraph, from: &str, to: &str, excluded: &str) -> bool {
        let mut queue = vec![from.to_string()];
        let mut seen: BTreeSet<String> = BTreeSet::new();
        while let Some(current) = queue.pop() {
            if current == excluded || !seen.insert(current.clone()) {
                continue;
            }
            if current == to {
                return true;
            }
            for e in g.edges.iter().filter(|e| e.from == current) {
                queue.push(e.to.clone());
            }
        }
        false
    }

    /// The factory golden must MATCH the committed Factory process contract
    /// (`FactoryProcessProfile.v1`, Phase D golden) instead of merely
    /// validating (preflight M1/M2 — the analogue of the Phase C sbfb-ideas
    /// miss): distinct driver/reviewer/codex agents mapped to declared
    /// process roles, a commit dominated by review, Codex rounds and human
    /// ratification (`reconciliation_required_before_pass`), a raw Codex
    /// output path (`codex_output_rewritable == false`), and no handoff
    /// that would merge the independent roles.
    #[test]
    fn factory_golden_matches_process_profile() {
        let profile: crate::factory_profile::FactoryProcessProfile =
            serde_json::from_value(parse_json(&src_dir().join("factory_profile.golden.json")))
                .expect("committed process profile parses");
        let role_ids: BTreeSet<String> = profile.roles.iter().map(|r| r.id.clone()).collect();

        let store = ArtifactStore::open("factory");
        let agents = load_agents(&store);
        let ids: BTreeSet<String> = agents.iter().map(|a| a.agent_id.clone()).collect();
        assert_eq!(
            ids,
            [
                "factory-driver",
                "factory-reviewer",
                "factory-codex-verifier"
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            "the factory golden encodes exactly the three independent agents"
        );
        let instruction_ids: BTreeSet<String> = agents
            .iter()
            .map(|a| a.instructions_ref.prompt_id.clone())
            .collect();
        assert_eq!(
            instruction_ids.len(),
            3,
            "instructions must be distinct per agent"
        );

        for agent in &agents {
            let role = agent
                .agent_id
                .strip_prefix("factory-")
                .expect("factory agent ids are role-prefixed");
            assert!(
                role_ids.contains(role),
                "agent {} must map to a declared process role (got roles {:?})",
                agent.agent_id,
                role_ids
            );
        }

        // The commit is DOMINATED by independent review, Codex rounds and
        // human ratification: removing any one of them disconnects it.
        let (_, graph) = parse_graph(&golden_path("factory"));
        assert!(
            reachable_without(&graph, "phase-brief", "commit-phase", "__none__"),
            "control: the commit is reachable at all"
        );
        for gate in ["independent-review", "codex-rounds", "po-ratification"] {
            assert!(
                !reachable_without(&graph, "phase-brief", "commit-phase", gate),
                "no path may reach the commit around {gate}"
            );
        }

        // Codex output is raw: the profile says so, and in the round
        // subworkflow the verifier's output flows STRAIGHT to the round
        // output — no rewriting node in between.
        assert!(!profile.invariants.codex_output_rewritable);
        assert!(!profile.invariants.pass_pending_committable);
        assert!(profile.invariants.reconciliation_required_before_pass);
        let subs = load_subworkflows(&store);
        let round = subs
            .iter()
            .find(|s| s.workflow_id == "factory-codex-round")
            .expect("codex round subworkflow exists");
        assert!(
            round
                .edges
                .iter()
                .any(|e| e.from == "codex-verify" && e.to == "round-out"),
            "the Codex output must flow straight to the round output"
        );

        // No merging handoffs: driver→reviewer is the only handoff in the
        // whole golden; reviewer and codex hand off to no one.
        let mut handoff_targets = Vec::new();
        for g in std::iter::once(&graph).chain(subs.iter()) {
            for node in g.nodes.values() {
                if let WorkflowNode::Handoff(h) = node {
                    handoff_targets.push(h.target_agent_id.clone());
                }
            }
        }
        assert_eq!(
            handoff_targets,
            vec!["factory-reviewer".to_string()],
            "driver→reviewer is the only handoff of the factory golden"
        );
        for agent in &agents {
            if agent.agent_id != "factory-driver" {
                assert!(
                    agent.allowed_handoffs.is_empty(),
                    "{} must not hand off (role independence)",
                    agent.agent_id
                );
            }
        }
    }

    /// Ideas: PUBLICATION IS A HUMAN DECISION (Codex r1 GAP1 / research
    /// §19.3 "a model never reclassifies data"). No field of the ideas
    /// golden carries a model-produced visibility, and every path from the
    /// entrypoint to the publishing `capability.call` is dominated by the
    /// `human.review` gate.
    #[test]
    fn ideas_publication_dominated_by_human_review() {
        let (_, ideas) = parse_graph(&golden_path("ideas"));
        assert!(
            matches!(
                ideas.nodes.get("publish-decision"),
                Some(WorkflowNode::HumanReview(_))
            ),
            "the publication gate must be a human.review node"
        );
        assert!(
            reachable_without(&ideas, "idea-in", "store-idea", "__none__"),
            "control: the publishing capability is reachable at all"
        );
        assert!(
            !reachable_without(&ideas, "idea-in", "store-idea", "publish-decision"),
            "no path may reach the publishing capability around the human gate"
        );
        // No schema of the ideas store models a visibility field at all —
        // there is nothing a model output could reclassify.
        let store = ArtifactStore::open("ideas");
        for id in store.ids("schemas", ".schema.json") {
            let schema = parse_json(&store.file("schemas", &id, "schema.json"));
            assert!(
                schema["properties"].get("visibility").is_none(),
                "ideas/{id}: no visibility field may exist (publication is a \
                 human decision, never data a model could produce)"
            );
        }
    }

    /// The shape-carrying members of a JSON Schema object, for structural
    /// comparison (descriptions and `$schema` wrappers differ legitimately).
    fn shape_of(schema: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "type": schema.get("type"),
            "properties": schema.get("properties"),
            "required": schema.get("required"),
            "additionalProperties": schema.get("additionalProperties"),
        })
    }

    /// Conception: the map's collection ITEM type IS the body subworkflow's
    /// input type (Codex r1 GAP2) — each fan-out invocation receives one
    /// `design-angle`, never the whole container.
    #[test]
    fn conception_map_item_type_matches_body_input() {
        let store = ArtifactStore::open("conception");
        let container = parse_json(&store.file("schemas", "angle-list", "schema.json"));
        let item = &container["properties"]["angles"]["items"];
        assert!(item.is_object(), "the collection must type its items");
        let body_input = parse_json(&store.file("schemas", "design-angle", "schema.json"));
        assert_eq!(
            shape_of(item),
            shape_of(&body_input),
            "the map item shape must equal the body subworkflow's input shape"
        );
    }

    /// Conception: the judge's declared input matches what actually arrives
    /// on its two in-edges (Codex r1 GAP3) — the aggregated `variant`
    /// objects from the map and the `checks-report` from the join.
    #[test]
    fn conception_fanin_types_align() {
        let store = ArtifactStore::open("conception");
        let judge_input = parse_json(&store.file("schemas", "judge-input", "schema.json"));
        let variant = parse_json(&store.file("schemas", "variant", "schema.json"));
        assert_eq!(
            shape_of(&judge_input["properties"]["variants"]["items"]),
            shape_of(&variant),
            "judge-input.variants items must be the map's variant objects"
        );
        let checks = parse_json(&store.file("schemas", "checks-report", "schema.json"));
        assert_eq!(
            shape_of(&judge_input["properties"]["checks"]),
            shape_of(&checks),
            "judge-input.checks must be the join's checks-report object"
        );
    }
}
