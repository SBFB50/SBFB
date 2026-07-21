// SPDX-License-Identifier: AGPL-3.0-or-later
//! `SBFBAgentDefinition.v1` — one bounded agent island (Sprint 83 Phase G).
//!
//! An agent definition is REUSABLE: several workflows may invoke the same
//! agent, and a single workflow may invoke it several times. What a definition
//! pins is everything a runtime needs to hold the agent to its bounds —
//! instructions, typed input/output, model policy, tools, allowed handoff
//! targets, context/memory/approval policies, and a multi-dimensional budget.
//! A `agent.invoke` node in a graph then fixes the per-invocation context and
//! may only ATTENUATE these bounds, never widen them (the invoke override
//! surface offers no widening field by construction; the numeric child<=parent
//! comparison is the Phase H compiler's job).
//!
//! Every field is REQUIRED — no `Option`, no `#[serde(default)]`. This crate's
//! signed-format rule (see [`crate::pack::canonical_bytes`]) rejects `null`
//! everywhere, and a defaultable bound is a bound that silently deserialises
//! to "unlimited" (preflight S3-05: budgets are a security field, not a
//! runtime-tolerance field — the pre-launch `#[serde(default)]` legitimacy
//! argument does NOT apply here).
//!
//! All `*Ref` types pin their referent BY DIGEST with the `PACK.lock`
//! semantics already frozen in Phase F: `blake3` is the BLAKE3 hex of the
//! referenced FILE's bytes (checkout-newline-canonicalised in the golden
//! tests, exactly like [`crate::factory_profile`] source digests). No new
//! digest rule is introduced by this module.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A pinned reference to a JSON Schema artefact.
///
/// `blake3` carries the `PACK.lock` pin of the schema file. Port compatibility
/// in v1 is NOMINAL BY DIGEST — two ports are compatible when they pin the
/// same schema bytes; any richer subtyping rule is a Phase H compiler
/// decision, not a format property (preflight S1a-F6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SchemaRef.v1")]
pub struct SchemaRef {
    /// Identity of the schema artefact, unique within the pack.
    pub schema_id: String,
    /// BLAKE3 hex of the pinned schema file bytes.
    pub blake3: String,
}

/// A pinned reference to a prompt/instructions artefact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PromptRef.v1")]
pub struct PromptRef {
    /// Identity of the prompt artefact, unique within the pack.
    pub prompt_id: String,
    /// BLAKE3 hex of the pinned prompt file bytes.
    pub blake3: String,
}

/// A pinned reference to a tool specification.
///
/// Tool schemas/descriptions are UNTRUSTED CONTENT (D4 point 6, MCP tool
/// poisoning): pinning them by digest is what makes a poisoned description a
/// detectable substitution instead of a silent one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ToolRef.v1")]
pub struct ToolRef {
    /// Identity of the tool specification, unique within the pack.
    pub tool_id: String,
    /// BLAKE3 hex of the pinned tool specification file bytes.
    pub blake3: String,
}

/// A pinned reference to a guardrail specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "GuardrailRef.v1")]
pub struct GuardrailRef {
    /// Identity of the guardrail specification, unique within the pack.
    pub guardrail_id: String,
    /// BLAKE3 hex of the pinned guardrail specification file bytes.
    pub blake3: String,
}

/// Which class of provider may run this agent's model calls.
///
/// Mirrors the adapter phases of the sprint plan: local/cloud model adapters
/// (Phase N) and the Claude Code / Codex CLI adapters (Phase O). The
/// provider-vs-structured-output compatibility refusal of research §10.5
/// ("provider incompatible with the model or the structured output") is a
/// Phase H compiler rule over this declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderClass {
    /// Local model runtime (e.g. Ollama on the node).
    Local,
    /// Remote cloud model API.
    Cloud,
    /// The Claude Code CLI adapter.
    ClaudeCode,
    /// The Codex CLI adapter.
    Codex,
}

/// Model selection and output-shape policy for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ModelPolicy.v1")]
pub struct ModelPolicy {
    /// Provider class allowed to serve this agent.
    pub provider_class: ProviderClass,
    /// Model identity the policy names (e.g. `"claude-opus-4-8"`); the
    /// adapter decides how strictly it is honoured (Phase N/O).
    pub model_id: String,
    /// When true, every model call MUST produce schema-conforming structured
    /// output; a provider that cannot guarantee it is incompatible (§10.5).
    pub require_structured_output: bool,
}

/// Context minimisation policy: the allowlist of context fields an agent (or a
/// handoff payload) may receive.
///
/// Minimisation is the DEFAULT (research §11.1): everything not named here is
/// withheld. The allowlist is REQUIRED wherever the type appears — an absent
/// filter that meant "everything" would reproduce the OpenAI Agents SDK
/// default this format explicitly rejects (D4: "the handoff never implicitly
/// transmits the whole history"; preflight B3/S1a-F1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "ContextPolicy.v1")]
pub struct ContextPolicy {
    /// Names of the context fields allowed through. An empty list is a valid,
    /// maximally-minimised policy (payload-only).
    pub allowed_context_fields: Vec<String>,
}

/// What memory an agent may keep across invocations.
///
/// Declarative in v1; the store, its permissions and its separation from
/// checkpoints (research §11.2) are host concerns (Phase I/L). No P2P
/// personal memory exists in any variant (§11.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryPolicy {
    /// No memory beyond the current invocation.
    None,
    /// Scratch memory scoped to one run, discarded at the end.
    RunScoped,
    /// Local permissioned long-term store, scoped to this agent identity.
    AgentScoped,
}

/// When an agent's actions require human approval.
///
/// Declarative in v1: the HITL chrome, its signed approval binding and the
/// sensitive-effect classes are Phase L/L1. `intent` never equals approval
/// (D4 point 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    /// No human approval required by this agent's own declaration (host
    /// policy may still require it — layers only ever attenuate).
    Never,
    /// Approval required before any sensitive effect.
    OnSensitiveEffect,
    /// Approval required before every effect.
    Always,
}

/// A multi-dimensional budget. Every dimension is REQUIRED and separate —
/// a scalar budget makes the child<=parent comparison undefined across units
/// (preflight S1a-F5), and a defaultable dimension deserialises to
/// "unlimited" (S3-05). Units are in the field names.
///
/// The same shape serves the workflow root, the per-invocation override and
/// the agent definition, so the Phase H compiler compares child against
/// parent DIMENSION BY DIMENSION with no unit conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "BudgetPolicy.v1")]
pub struct BudgetPolicy {
    /// Maximum total tokens (input + output) across all model calls in scope.
    pub max_total_tokens: u64,
    /// Maximum spend in micro-USD (1_000_000 = 1 USD) in scope.
    pub max_cost_micro_usd: u64,
    /// Maximum wall-clock seconds in scope.
    pub max_wall_clock_seconds: u64,
    /// Maximum agent turns (model-call iterations) in scope.
    pub max_turns: u32,
}

// Workflow-language frontier (Sprint 83 Phase G, doctrine §7): one bounded
// agent island of the signed pack, read by a distinct runtime — the Rust
// compiler (Phase H), the host engine (Phases I/J) and the sandboxed Workflow
// Hub (Phase T). It is pinned BY DIGEST inside `SBFBPromptAgentPack.v1`
// (PACK.lock), never signed or gossiped on its own, so it carries no
// `// FRONTIER:` domain/version tag (that opt-in registry is for
// `DOMAIN_*_V1` families and inventing one here would trip the frozen
// census of 29). Its machine contract is the generated schema snapshot pinned
// by `contracts::all_contract_schemas` (identity 3/7, len==7) plus the four
// workflow goldens; the GUIDE + llms.txt closure is deferred to the S83
// wrap-up phase per AM-3.
/// `SBFBAgentDefinition.v1` — one bounded agent island (D4). Everything the
/// runtime needs to hold the agent to its declared bounds, every field
/// required, every referent pinned by digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBAgentDefinition.v1")]
pub struct SbfbAgentDefinition {
    /// Stable identity of the agent, unique within the pack.
    pub agent_id: String,
    /// Monotonic version of THIS agent identity.
    pub agent_version: u32,
    /// Human-facing display name.
    pub display_name: String,
    /// Human-facing description of the agent's role.
    pub description: String,
    /// Pinned instructions (system prompt) artefact.
    pub instructions_ref: PromptRef,
    /// Pinned schema of the payload this agent accepts.
    pub input_schema_ref: SchemaRef,
    /// Pinned schema of the output this agent must produce.
    pub output_schema_ref: SchemaRef,
    /// Model selection and structured-output policy.
    pub model_policy: ModelPolicy,
    /// Tools this agent may use, each pinned by digest. May be empty.
    pub tool_refs: Vec<ToolRef>,
    /// Agent ids this agent may hand control off to. A handoff node whose
    /// upstream agent does not list its target here is invalid (refused by
    /// the Phase H compiler: "handoff not authorised by the target agent"
    /// family of §10.5). May be empty — most agents hand off to no one.
    pub allowed_handoffs: Vec<String>,
    /// Context allowlist for this agent's own invocations.
    pub context_policy: ContextPolicy,
    /// Memory scope this agent is allowed.
    pub memory_policy: MemoryPolicy,
    /// Guardrails applied around this agent's calls, pinned by digest. May be
    /// empty.
    pub guardrail_refs: Vec<GuardrailRef>,
    /// When this agent's actions require human approval.
    pub approval_policy: ApprovalPolicy,
    /// The agent's own budget ceiling. An `agent.invoke` node declares the
    /// effective per-invocation budget, which must not exceed this (Phase H).
    pub budget: BudgetPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> serde_json::Value {
        serde_json::json!({
            "agent_id": "sample-agent",
            "agent_version": 1,
            "display_name": "Sample",
            "description": "A bounded sample agent.",
            "instructions_ref": { "prompt_id": "sample-instructions", "blake3": "aa" },
            "input_schema_ref": { "schema_id": "sample-in", "blake3": "bb" },
            "output_schema_ref": { "schema_id": "sample-out", "blake3": "cc" },
            "model_policy": {
                "provider_class": "claude-code",
                "model_id": "claude-opus-4-8",
                "require_structured_output": true
            },
            "tool_refs": [],
            "allowed_handoffs": ["other-agent"],
            "context_policy": { "allowed_context_fields": ["brief"] },
            "memory_policy": "run-scoped",
            "guardrail_refs": [],
            "approval_policy": "on-sensitive-effect",
            "budget": {
                "max_total_tokens": 200000,
                "max_cost_micro_usd": 5000000,
                "max_wall_clock_seconds": 3600,
                "max_turns": 40
            }
        })
    }

    #[test]
    fn sample_agent_parses_and_round_trips() {
        let value = sample_json();
        let agent: SbfbAgentDefinition = serde_json::from_value(value.clone()).expect("parses");
        assert_eq!(serde_json::to_value(&agent).expect("serialises"), value);
    }

    /// Budgets are a security field: an agent with no budget must be a parse
    /// error, never an unlimited default (preflight S3-05).
    /// Every negative below first proves the unmutated base parses, so an
    /// `expect_err` can never pass for the WRONG reason (review r1 D2-01).
    fn control_base_parses() {
        serde_json::from_value::<SbfbAgentDefinition>(sample_json())
            .expect("control: base fixture parses");
    }

    #[test]
    fn missing_budget_is_a_parse_error() {
        control_base_parses();
        let mut value = sample_json();
        value.as_object_mut().unwrap().remove("budget");
        let err = serde_json::from_value::<SbfbAgentDefinition>(value).unwrap_err();
        assert!(
            err.to_string().contains("budget"),
            "unexpected error: {err}"
        );
    }

    /// A missing budget DIMENSION is equally fatal — a scalar or partial
    /// budget would make child<=parent comparison undefined (S1a-F5).
    #[test]
    fn missing_budget_dimension_is_a_parse_error() {
        control_base_parses();
        let mut value = sample_json();
        value["budget"]
            .as_object_mut()
            .unwrap()
            .remove("max_cost_micro_usd");
        serde_json::from_value::<SbfbAgentDefinition>(value).unwrap_err();
    }

    /// The context filter is REQUIRED: absence must never mean
    /// "everything" (D4 / preflight B3).
    #[test]
    fn missing_context_policy_is_a_parse_error() {
        control_base_parses();
        let mut value = sample_json();
        value.as_object_mut().unwrap().remove("context_policy");
        serde_json::from_value::<SbfbAgentDefinition>(value).unwrap_err();
    }

    #[test]
    fn unknown_field_is_rejected() {
        control_base_parses();
        let mut value = sample_json();
        value
            .as_object_mut()
            .unwrap()
            .insert("surprise".into(), serde_json::json!(1));
        serde_json::from_value::<SbfbAgentDefinition>(value).unwrap_err();
    }

    /// One canonical kebab spelling per enum value — a signed JCS format
    /// cannot afford two spellings of one meaning (preflight S1a-F4 applies
    /// to every enum of the language, not only node kinds).
    #[test]
    fn enum_spellings_are_canonical_kebab() {
        assert_eq!(
            serde_json::to_value(ProviderClass::ClaudeCode).unwrap(),
            serde_json::json!("claude-code")
        );
        assert_eq!(
            serde_json::to_value(MemoryPolicy::RunScoped).unwrap(),
            serde_json::json!("run-scoped")
        );
        assert_eq!(
            serde_json::to_value(ApprovalPolicy::OnSensitiveEffect).unwrap(),
            serde_json::json!("on-sensitive-effect")
        );
        serde_json::from_value::<ProviderClass>(serde_json::json!("ClaudeCode")).unwrap_err();
    }
}
