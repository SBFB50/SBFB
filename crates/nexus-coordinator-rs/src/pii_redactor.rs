// SPDX-License-Identifier: AGPL-3.0-or-later
//! PII redaction — regex-based detection and redaction of personally
//! identifiable information in task inputs.
//!
//! Port of `packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py`
//! (regex subset, Sprint 39 Phase A). ML/Presidio layers omitted
//! pre-v1.0 — regex covers 7 structured PII entities.

use std::sync::OnceLock;

use regex::Regex;

use crate::guardrails::{Guardrail, GuardrailContext, GuardrailDirection, GuardrailOutcome};

struct CompiledPattern {
    entity: &'static str,
    regex: Regex,
    luhn_check: bool,
}

fn patterns() -> &'static [CompiledPattern] {
    static PATTERNS: OnceLock<Vec<CompiledPattern>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            CompiledPattern {
                entity: "EMAIL_ADDRESS",
                regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                luhn_check: false,
            },
            CompiledPattern {
                entity: "PHONE_NUMBER",
                regex: Regex::new(r"(?:\+?\d{1,3}[\s-]?)?(?:\(?\d{3}\)?[\s-]?)\d{3}[\s-]?\d{4}")
                    .unwrap(),
                luhn_check: false,
            },
            CompiledPattern {
                entity: "CREDIT_CARD",
                regex: Regex::new(r"\b(?:\d[ -]?){13,19}\b").unwrap(),
                luhn_check: true,
            },
            CompiledPattern {
                entity: "IBAN_CODE",
                regex: Regex::new(r"\b[A-Z]{2}\d{2}[A-Z0-9]{4,30}\b").unwrap(),
                luhn_check: false,
            },
            CompiledPattern {
                entity: "US_SSN",
                regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
                luhn_check: false,
            },
            CompiledPattern {
                entity: "IP_ADDRESS",
                regex: Regex::new(
                    r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b",
                )
                .unwrap(),
                luhn_check: false,
            },
            CompiledPattern {
                entity: "URL",
                regex: Regex::new(r#"https?://[^\s<>"']+"#).unwrap(),
                luhn_check: false,
            },
        ]
    })
}

fn luhn_valid(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&digits.len()) {
        return false;
    }
    let checksum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();
    checksum % 10 == 0
}

#[derive(Debug, Clone)]
struct Span {
    start: usize,
    end: usize,
}

const DEFAULT_REPLACEMENT: &str = "[REDACTED]";

#[derive(Debug, Clone)]
pub struct RedactionPolicy {
    pub enabled_entities: Vec<String>,
    pub replacement: String,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            enabled_entities: vec![
                "EMAIL_ADDRESS".into(),
                "PHONE_NUMBER".into(),
                "CREDIT_CARD".into(),
                "IBAN_CODE".into(),
                "US_SSN".into(),
                "IP_ADDRESS".into(),
                "URL".into(),
            ],
            replacement: DEFAULT_REPLACEMENT.into(),
        }
    }
}

pub struct PiiRedactor {
    enabled: Vec<String>,
    replacement: String,
}

impl Default for PiiRedactor {
    fn default() -> Self {
        Self::new(&RedactionPolicy::default())
    }
}

impl PiiRedactor {
    pub fn new(policy: &RedactionPolicy) -> Self {
        Self {
            enabled: policy.enabled_entities.clone(),
            replacement: policy.replacement.clone(),
        }
    }

    pub fn has_pii(&self, text: &str) -> bool {
        for pat in patterns() {
            if !self.enabled.iter().any(|e| e == pat.entity) {
                continue;
            }
            for m in pat.regex.find_iter(text) {
                if pat.luhn_check && !luhn_valid(m.as_str()) {
                    continue;
                }
                return true;
            }
        }
        false
    }

    pub fn redact(&self, text: &str) -> String {
        let spans = self.extract_spans(text);
        if spans.is_empty() {
            return text.to_string();
        }
        self.rewrite(text, &spans)
    }

    fn extract_spans(&self, text: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        for pat in patterns() {
            if !self.enabled.iter().any(|e| e == pat.entity) {
                continue;
            }
            for m in pat.regex.find_iter(text) {
                if pat.luhn_check && !luhn_valid(m.as_str()) {
                    continue;
                }
                spans.push(Span {
                    start: m.start(),
                    end: m.end(),
                });
            }
        }
        dedupe_spans(&mut spans);
        spans
    }

    fn rewrite(&self, text: &str, spans: &[Span]) -> String {
        let mut result = text.to_string();
        for span in spans.iter().rev() {
            result.replace_range(span.start..span.end, &self.replacement);
        }
        result
    }
}

fn dedupe_spans(spans: &mut Vec<Span>) {
    spans.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
    let mut merged: Vec<Span> = Vec::new();
    for s in spans.drain(..) {
        if let Some(last) = merged.last() {
            if s.start < last.end {
                let len_s = s.end - s.start;
                let len_last = last.end - last.start;
                if len_s > len_last {
                    *merged.last_mut().unwrap() = s;
                }
                continue;
            }
        }
        merged.push(s);
    }
    *spans = merged;
}

pub struct PiiInputGuardrail {
    redactor: PiiRedactor,
}

impl PiiInputGuardrail {
    pub fn new(redactor: PiiRedactor) -> Self {
        Self { redactor }
    }
}

impl Default for PiiInputGuardrail {
    fn default() -> Self {
        Self::new(PiiRedactor::default())
    }
}

impl Guardrail for PiiInputGuardrail {
    fn name(&self) -> &str {
        "pii_input"
    }

    fn direction(&self) -> GuardrailDirection {
        GuardrailDirection::Input
    }

    fn check(&self, ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
        if self.redactor.has_pii(ctx.user_prompt) {
            GuardrailOutcome::Tripwire {
                reason: "PII detected in input".into(),
            }
        } else {
            GuardrailOutcome::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luhn_valid_visa() {
        assert!(luhn_valid("4111111111111111"));
    }

    #[test]
    fn luhn_invalid() {
        assert!(!luhn_valid("1234567890123456"));
    }

    #[test]
    fn redact_email() {
        let r = PiiRedactor::default();
        assert_eq!(
            r.redact("contact me at user@test.com please"),
            "contact me at [REDACTED] please"
        );
    }

    #[test]
    fn redact_phone() {
        let r = PiiRedactor::default();
        let result = r.redact("call 555-123-4567 now");
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("555-123-4567"));
    }

    #[test]
    fn redact_ssn() {
        let r = PiiRedactor::default();
        assert_eq!(r.redact("SSN 123-45-6789 here"), "SSN [REDACTED] here");
    }

    #[test]
    fn redact_credit_card_luhn_valid() {
        let r = PiiRedactor::default();
        let result = r.redact("card 4111 1111 1111 1111 ok");
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("4111"));
    }

    #[test]
    fn no_redact_non_luhn() {
        let r = PiiRedactor::default();
        let input = "ref 1234 5678 9012 3456 end";
        let result = r.redact(input);
        assert!(!result.contains("[REDACTED]"));
    }

    #[test]
    fn redact_ipv4() {
        let r = PiiRedactor::default();
        assert_eq!(
            r.redact("server at 192.168.1.1 port 80"),
            "server at [REDACTED] port 80"
        );
    }

    #[test]
    fn redact_url() {
        let r = PiiRedactor::default();
        let result = r.redact("visit https://example.com/path?q=1 today");
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("https://"));
    }

    #[test]
    fn redact_iban() {
        let r = PiiRedactor::default();
        let result = r.redact("pay to DE89370400440532013000 please");
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("DE89"));
    }

    #[test]
    fn has_pii_true() {
        let r = PiiRedactor::default();
        assert!(r.has_pii("email alice@example.com"));
    }

    #[test]
    fn has_pii_false() {
        let r = PiiRedactor::default();
        assert!(!r.has_pii("nothing sensitive here"));
    }

    #[test]
    fn pii_input_guardrail_passes_clean() {
        let g = PiiInputGuardrail::default();
        let ctx = GuardrailContext {
            system_prompt: "",
            user_prompt: "hello world",
            model_output: "",
        };
        assert_eq!(g.check(&ctx), GuardrailOutcome::Pass);
    }

    #[test]
    fn pii_input_guardrail_trips_on_pii() {
        let g = PiiInputGuardrail::default();
        let ctx = GuardrailContext {
            system_prompt: "",
            user_prompt: "my email is alice@example.com",
            model_output: "",
        };
        match g.check(&ctx) {
            GuardrailOutcome::Tripwire { reason } => {
                assert!(reason.contains("PII"));
            }
            other => panic!("expected Tripwire, got {other:?}"),
        }
    }
}
