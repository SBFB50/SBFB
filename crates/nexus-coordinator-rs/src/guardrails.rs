// SPDX-License-Identifier: AGPL-3.0-or-later
//! Guardrails pipeline — composable safety check chain for input
//! and output filtering (Sprint 38 Phase C).
//!
//! Port of `packages/nexus-coordinator/src/nexus_coordinator/guardrails.py`
//! (137 LOC Python → Rust). The pipeline sequences `Guardrail`
//! trait objects and short-circuits on `Tripwire` outcomes.

use crate::output_filter::OutputFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardrailDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardrailOutcome {
    Pass,
    Flag { reason: String },
    Tripwire { reason: String },
}

pub struct GuardrailContext<'a> {
    pub system_prompt: &'a str,
    pub user_prompt: &'a str,
    pub model_output: &'a str,
}

pub trait Guardrail: Send + Sync {
    fn name(&self) -> &str;
    fn direction(&self) -> GuardrailDirection;
    fn check(&self, ctx: &GuardrailContext<'_>) -> GuardrailOutcome;
}

pub struct ChainResult {
    pub passed: bool,
    pub flags: Vec<String>,
    pub tripwire: Option<String>,
}

pub struct GuardrailChain {
    guardrails: Vec<Box<dyn Guardrail>>,
}

impl GuardrailChain {
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
        }
    }

    pub fn push(mut self, g: Box<dyn Guardrail>) -> Self {
        self.guardrails.push(g);
        self
    }

    pub fn run(&self, ctx: &GuardrailContext<'_>) -> ChainResult {
        let mut flags = Vec::new();

        for g in &self.guardrails {
            match g.check(ctx) {
                GuardrailOutcome::Pass => {}
                GuardrailOutcome::Flag { reason } => {
                    tracing::info!(guardrail = g.name(), %reason, "guardrail flagged");
                    flags.push(reason);
                }
                GuardrailOutcome::Tripwire { reason } => {
                    tracing::warn!(guardrail = g.name(), %reason, "guardrail tripwire");
                    return ChainResult {
                        passed: false,
                        flags,
                        tripwire: Some(reason),
                    };
                }
            }
        }

        ChainResult {
            passed: true,
            flags,
            tripwire: None,
        }
    }
}

impl Default for GuardrailChain {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OutputSafetyGuardrail {
    filter: OutputFilter,
}

impl OutputSafetyGuardrail {
    pub fn new(filter: OutputFilter) -> Self {
        Self { filter }
    }
}

impl Default for OutputSafetyGuardrail {
    fn default() -> Self {
        Self::new(OutputFilter::default())
    }
}

impl Guardrail for OutputSafetyGuardrail {
    fn name(&self) -> &str {
        "output_safety"
    }

    fn direction(&self) -> GuardrailDirection {
        GuardrailDirection::Output
    }

    fn check(&self, ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
        let verdict = self
            .filter
            .filter(ctx.system_prompt, ctx.user_prompt, ctx.model_output);
        if verdict.is_valid {
            GuardrailOutcome::Pass
        } else {
            GuardrailOutcome::Tripwire {
                reason: format!("{:?}", verdict.reason),
            }
        }
    }
}

pub fn default_output_chain() -> GuardrailChain {
    GuardrailChain::new().push(Box::new(OutputSafetyGuardrail::default()))
}

pub fn default_input_chain() -> GuardrailChain {
    GuardrailChain::new().push(Box::new(crate::pii_redactor::PiiInputGuardrail::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysPass;
    impl Guardrail for AlwaysPass {
        fn name(&self) -> &str {
            "always_pass"
        }
        fn direction(&self) -> GuardrailDirection {
            GuardrailDirection::Output
        }
        fn check(&self, _ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
            GuardrailOutcome::Pass
        }
    }

    struct AlwaysFlag;
    impl Guardrail for AlwaysFlag {
        fn name(&self) -> &str {
            "always_flag"
        }
        fn direction(&self) -> GuardrailDirection {
            GuardrailDirection::Output
        }
        fn check(&self, _ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
            GuardrailOutcome::Flag {
                reason: "flagged".into(),
            }
        }
    }

    struct AlwaysTripwire;
    impl Guardrail for AlwaysTripwire {
        fn name(&self) -> &str {
            "always_tripwire"
        }
        fn direction(&self) -> GuardrailDirection {
            GuardrailDirection::Output
        }
        fn check(&self, _ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
            GuardrailOutcome::Tripwire {
                reason: "tripped".into(),
            }
        }
    }

    fn ctx(output: &str) -> GuardrailContext<'_> {
        GuardrailContext {
            system_prompt: "",
            user_prompt: "",
            model_output: output,
        }
    }

    #[test]
    fn chain_empty_passes() {
        let chain = GuardrailChain::new();
        let result = chain.run(&ctx("anything"));
        assert!(result.passed);
        assert!(result.flags.is_empty());
        assert!(result.tripwire.is_none());
    }

    #[test]
    fn chain_pass_through() {
        let chain = GuardrailChain::new().push(Box::new(AlwaysPass));
        let result = chain.run(&ctx("text"));
        assert!(result.passed);
    }

    #[test]
    fn chain_flag_accumulates() {
        let chain = GuardrailChain::new()
            .push(Box::new(AlwaysFlag))
            .push(Box::new(AlwaysFlag));
        let result = chain.run(&ctx("text"));
        assert!(result.passed);
        assert_eq!(result.flags.len(), 2);
    }

    #[test]
    fn chain_tripwire_short_circuits() {
        let chain = GuardrailChain::new()
            .push(Box::new(AlwaysTripwire))
            .push(Box::new(AlwaysPass));
        let result = chain.run(&ctx("text"));
        assert!(!result.passed);
        assert!(result.tripwire.is_some());
    }

    #[test]
    fn output_safety_guardrail_passes_clean() {
        let chain = default_output_chain();
        let c = GuardrailContext {
            system_prompt: "You are helpful.",
            user_prompt: "Hi",
            model_output: "Hello! How can I help?",
        };
        let result = chain.run(&c);
        assert!(result.passed);
    }

    #[test]
    fn output_safety_guardrail_trips_on_invisible() {
        let chain = default_output_chain();
        let c = GuardrailContext {
            system_prompt: "system",
            user_prompt: "user",
            model_output: "clean\u{200B}hidden\u{200B}text",
        };
        let result = chain.run(&c);
        assert!(!result.passed);
        assert!(result.tripwire.unwrap().contains("InvisibleText"));
    }
}
