// SPDX-License-Identifier: AGPL-3.0-or-later
//! Output safety filter — scans model output for invisible text
//! steganography and system prompt leakage before result acceptance.
//!
//! Port of `packages/nexus-coordinator/src/nexus_coordinator/output_filter.py`
//! (397 LOC Python → Rust, Sprint 38 Phase B).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterReason {
    Ok,
    InvisibleText,
    PromptEchoExact,
    PromptEchoSubstring,
    PromptEchoEed,
}

#[derive(Debug, Clone)]
pub struct FilterVerdict {
    pub is_valid: bool,
    pub reason: FilterReason,
    pub risk_score: f64,
    pub sanitized_output: String,
}

fn is_invisible_char(c: char) -> bool {
    matches!(c,
        '\u{200B}'..='\u{200F}' |
        '\u{2060}' |
        '\u{FEFF}' |
        '\u{E000}'..='\u{F8FF}' |
        '\u{F0000}'..='\u{FFFFD}' |
        '\u{100000}'..='\u{10FFFD}' |
        '\u{E0020}'..='\u{E007F}'
    )
}

fn is_bidi_format(c: char) -> bool {
    matches!(c,
        '\u{202A}'..='\u{202E}' |
        '\u{2066}'..='\u{2069}'
    )
}

pub fn strip_invisible(input: &str) -> String {
    input
        .chars()
        .filter(|&c| !is_invisible_char(c) || is_bidi_format(c))
        .collect()
}

pub fn has_invisible_text(input: &str) -> bool {
    input
        .chars()
        .any(|c| is_invisible_char(c) && !is_bidi_format(c))
}

fn check_prompt_echo_exact(prompt: &str, output: &str) -> bool {
    !prompt.is_empty() && output.contains(prompt)
}

fn check_prompt_echo_substring(prompt: &str, output: &str, min_len: usize) -> bool {
    if prompt.len() < min_len {
        return false;
    }
    let prompt_chars: Vec<char> = prompt.chars().collect();
    let output_lower = output.to_lowercase();
    let prompt_lower = prompt.to_lowercase();
    let prompt_lower_chars: Vec<char> = prompt_lower.chars().collect();

    for start in 0..=prompt_lower_chars.len().saturating_sub(min_len) {
        let end = (start + min_len).min(prompt_lower_chars.len());
        let slice: String = prompt_lower_chars[start..end].iter().collect();
        if output_lower.contains(&slice) {
            return true;
        }
    }
    let _ = prompt_chars;
    false
}

fn check_prompt_echo_eed(prompt: &str, output: &str, threshold: f64) -> bool {
    if prompt.is_empty() || output.is_empty() {
        return false;
    }
    let similarity = strsim::normalized_levenshtein(prompt, output);
    similarity >= threshold
}

const DEFAULT_EED_THRESHOLD: f64 = 0.85;
const DEFAULT_SUBSTRING_MIN_LEN: usize = 40;

pub struct OutputFilter {
    eed_threshold: f64,
    substring_min_len: usize,
}

impl Default for OutputFilter {
    fn default() -> Self {
        Self {
            eed_threshold: DEFAULT_EED_THRESHOLD,
            substring_min_len: DEFAULT_SUBSTRING_MIN_LEN,
        }
    }
}

impl OutputFilter {
    pub fn new(eed_threshold: f64, substring_min_len: usize) -> Self {
        Self {
            eed_threshold,
            substring_min_len,
        }
    }

    pub fn filter(&self, system_prompt: &str, _user_prompt: &str, output: &str) -> FilterVerdict {
        let sanitized = strip_invisible(output);
        let had_invisible = sanitized.len() != output.len();

        if had_invisible {
            return FilterVerdict {
                is_valid: false,
                reason: FilterReason::InvisibleText,
                risk_score: 0.9,
                sanitized_output: sanitized,
            };
        }

        if check_prompt_echo_exact(system_prompt, &sanitized) {
            return FilterVerdict {
                is_valid: false,
                reason: FilterReason::PromptEchoExact,
                risk_score: 1.0,
                sanitized_output: sanitized,
            };
        }

        if check_prompt_echo_substring(system_prompt, &sanitized, self.substring_min_len) {
            return FilterVerdict {
                is_valid: false,
                reason: FilterReason::PromptEchoSubstring,
                risk_score: 0.8,
                sanitized_output: sanitized,
            };
        }

        if check_prompt_echo_eed(system_prompt, &sanitized, self.eed_threshold) {
            return FilterVerdict {
                is_valid: false,
                reason: FilterReason::PromptEchoEed,
                risk_score: 0.7,
                sanitized_output: sanitized,
            };
        }

        FilterVerdict {
            is_valid: true,
            reason: FilterReason::Ok,
            risk_score: 0.0,
            sanitized_output: sanitized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_invisible_removes_zero_width() {
        let input = "hello\u{200B}world\u{FEFF}!";
        assert_eq!(strip_invisible(input), "helloworld!");
    }

    #[test]
    fn strip_invisible_preserves_bidi() {
        let input = "text\u{202A}bidi\u{202C}end\u{2066}iso\u{2069}";
        assert_eq!(strip_invisible(input), input);
    }

    #[test]
    fn strip_invisible_removes_pua() {
        let input = "a\u{E000}b\u{F8FF}c";
        assert_eq!(strip_invisible(input), "abc");
    }

    #[test]
    fn strip_invisible_removes_tags() {
        let input = "start\u{E0020}\u{E007F}end";
        assert_eq!(strip_invisible(input), "startend");
    }

    #[test]
    fn prompt_echo_exact_detected() {
        let filter = OutputFilter::default();
        let v = filter.filter(
            "secret system prompt",
            "",
            "The secret system prompt is here",
        );
        assert!(!v.is_valid);
        assert_eq!(v.reason, FilterReason::PromptEchoExact);
    }

    #[test]
    fn prompt_echo_substring_detected() {
        let long_prompt =
            "the quick brown fox jumps over the lazy dog and then runs back again quickly";
        let snippet = &long_prompt[10..55];
        let output = format!("model says: {snippet} and more");
        let filter = OutputFilter::new(0.85, 40);
        let v = filter.filter(long_prompt, "", &output);
        assert!(!v.is_valid);
        assert_eq!(v.reason, FilterReason::PromptEchoSubstring);
    }

    #[test]
    fn prompt_echo_eed_detected() {
        let prompt = "the quick brown fox jumps over the lazy dog";
        let output = "the quick brown fox jumps ovar the laxy dog";
        let filter = OutputFilter::default();
        let v = filter.filter(prompt, "", output);
        assert!(!v.is_valid);
        assert_eq!(v.reason, FilterReason::PromptEchoEed);
    }

    #[test]
    fn prompt_echo_eed_below_threshold() {
        let prompt = "completely different text here";
        let output = "nothing similar at all in this response";
        let filter = OutputFilter::default();
        let v = filter.filter(prompt, "", output);
        assert!(v.is_valid);
        assert_eq!(v.reason, FilterReason::Ok);
    }

    #[test]
    fn filter_clean_output_passes() {
        let filter = OutputFilter::default();
        let v = filter.filter(
            "You are a helpful assistant.",
            "What is 2+2?",
            "The answer is 4.",
        );
        assert!(v.is_valid);
        assert_eq!(v.reason, FilterReason::Ok);
        assert_eq!(v.risk_score, 0.0);
    }

    #[test]
    fn filter_invisible_text_detected() {
        let filter = OutputFilter::default();
        let v = filter.filter(
            "system",
            "user",
            "normal\u{200B}text\u{200B}with\u{200B}hidden",
        );
        assert!(!v.is_valid);
        assert_eq!(v.reason, FilterReason::InvisibleText);
        assert_eq!(v.sanitized_output, "normaltextwithhidden");
    }

    #[test]
    fn prompt_echo_substring_exact_min_len() {
        let prompt = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij0123";
        assert_eq!(prompt.len(), 40);
        let output = "prefix abcdefghijklmnopqrstuvwxyzabcdefghij0123 suffix";
        assert!(check_prompt_echo_substring(prompt, output, 40));
    }
}
