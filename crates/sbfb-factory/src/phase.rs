// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canonical phase-token discovery and ordering for sprint status/history.
//!
//! The SBFB process contract (`docs/claude/README.md` §4 "Budget de phases")
//! makes the phase count UNBOUNDED: a phase label matches `[A-Z]+[0-9]?` —
//! `A`..`Z`, then `AA`, `AB`, ... with an optional sub-phase digit (`F1`/`F2`,
//! cf. S77). This module is the single source of truth that replaces every
//! hardcoded `['A'..'G']` alphabet and every `format!("..._phase_{letter}_...")`
//! path that was built from an UPPERCASE letter.
//!
//! It closes two cross-cutting defects the `['A'..'G']` arrays carried:
//!  - the A-G cap — phases `H`, `I`, ... `AA` were invisible to status/history
//!    (S77 reached phase `N`);
//!  - a case mismatch — the code built `sprint{N}_phase_A_...` (UPPERCASE)
//!    while active-sprint artifacts are lowercase (`sprint{N}_phase_a_...`).
//!    That matched only on a case-insensitive filesystem (Windows); on Linux /
//!    CI / the VPS (ext4) every `.exists()` returned false, freezing status and
//!    emitting false "missing review/codex" audit issues. The archive is
//!    MIXED-case (sprint65 + v1.2/v2.0 uppercase, sprint66+ lowercase), so the
//!    fix is a case-insensitive `read_dir` scan that returns the REAL on-disk
//!    path — never a path rebuilt from a normalized letter.

use std::path::{Path, PathBuf};

/// One discovered phase artifact: the canonical lowercase `label` (e.g. `"a"`,
/// `"aa"`, `"f1"`) and the REAL `path` on disk (case preserved, so it opens on
/// every filesystem regardless of how the file was named).
pub(crate) struct PhaseArtifact {
    pub label: String,
    pub path: PathBuf,
}

/// Canonical phase ordering key: shorter labels first (`a` < … < `z` < `aa`),
/// then lexicographic. A naive string sort would place `"aa"` before `"b"`,
/// which is wrong for a bijective base-26 sequence.
pub(crate) fn phase_order_key(label: &str) -> (usize, String) {
    (label.len(), label.to_ascii_lowercase())
}

/// True if `label` (already lowercased) is a valid phase token: one or more
/// ASCII letters followed by an optional single digit — the lowercased mirror
/// of the `[A-Z]+[0-9]?` commit-title regex. `Phase 0` (the audit gate, which
/// carries no letter) is intentionally NOT a tracked phase here.
fn is_phase_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_lowercase() {
        i += 1;
    }
    if i == 0 {
        return false; // need at least one letter
    }
    if i == bytes.len() {
        return true; // letters only
    }
    // at most one trailing digit
    i == bytes.len() - 1 && bytes[i].is_ascii_digit()
}

/// Parse the phase label out of `sprint{sprint}_phase_{label}_{kind}.md`,
/// case-insensitively. Returns the lowercase label when the name matches.
fn label_from_filename(name: &str, sprint: u32, kind: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let prefix = format!("sprint{sprint}_phase_");
    let suffix = format!("_{kind}.md");
    let mid = lower.strip_prefix(&prefix)?.strip_suffix(&suffix)?;
    if is_phase_label(mid) {
        Some(mid.to_string())
    } else {
        None
    }
}

/// Discover every `sprint{sprint}_phase_{label}_{kind}.md` in `dir`, sorted in
/// canonical phase order. `kind` is e.g. `"review"`, `"preflight"`,
/// `"codex_review"`. Each entry keeps the REAL on-disk path, so a subsequent
/// read succeeds whatever the file's case.
pub(crate) fn discover_phase_artifacts(dir: &Path, sprint: u32, kind: &str) -> Vec<PhaseArtifact> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Some(label) = label_from_filename(name, sprint, kind) {
                out.push(PhaseArtifact { label, path });
            }
        }
    }
    out.sort_by_key(|a| phase_order_key(&a.label));
    out
}

/// Canonical-ordered set of phase labels present for `sprint` in `dir`, taking
/// the union across the `review` / `preflight` / `codex_review` artifact kinds
/// (a phase exists if ANY of its artifacts is on disk). Labels are lowercase.
pub(crate) fn discover_phase_labels(dir: &Path, sprint: u32) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    for kind in ["preflight", "review", "codex_review"] {
        for art in discover_phase_artifacts(dir, sprint, kind) {
            if !labels.contains(&art.label) {
                labels.push(art.label);
            }
        }
    }
    labels.sort_by_key(|a| phase_order_key(a));
    labels
}

/// Case-insensitive lookup of a single `sprint{sprint}_phase_{label}_{kind}.md`
/// in `dir`, returning the REAL on-disk path. `label` is matched case-folded,
/// so an UPPERCASE archive file and a lowercase active file both resolve.
pub(crate) fn find_phase_artifact(
    dir: &Path,
    sprint: u32,
    label: &str,
    kind: &str,
) -> Option<PathBuf> {
    let want = label.to_ascii_lowercase();
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if label_from_filename(name, sprint, kind).as_deref() == Some(want.as_str()) {
            return Some(path);
        }
    }
    None
}

/// Bijective base-26 successor of a phase label (`a`->`b`, `z`->`aa`,
/// `az`->`ba`, `zz`->`aaa`). Any digit sub-phase suffix is dropped — the
/// successor of `f1` is `g`. Used only to name the NEXT phase that has no
/// artifact on disk yet (display is uppercased by the caller).
pub(crate) fn next_phase_label(label: &str) -> String {
    let mut chars: Vec<u8> = label
        .bytes()
        .filter(|b| b.is_ascii_alphabetic())
        .map(|b| b.to_ascii_lowercase())
        .collect();
    if chars.is_empty() {
        return "a".to_string();
    }
    let mut i = chars.len();
    loop {
        if i == 0 {
            chars.insert(0, b'a');
            break;
        }
        i -= 1;
        if chars[i] == b'z' {
            chars[i] = b'a';
        } else {
            chars[i] += 1;
            break;
        }
    }
    String::from_utf8(chars).expect("ascii lowercase letters")
}

/// Uppercased display form of a canonical lowercase label (`"aa"` -> `"AA"`,
/// `"f1"` -> `"F1"`) — the convention used in commit titles and prose.
pub(crate) fn display_label(label: &str) -> String {
    label.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_phase_label_base26_rollover() {
        assert_eq!(next_phase_label("a"), "b");
        assert_eq!(next_phase_label("f"), "g");
        assert_eq!(next_phase_label("z"), "aa");
        assert_eq!(next_phase_label("az"), "ba");
        assert_eq!(next_phase_label("zz"), "aaa");
        // sub-phase digit is dropped: successor of f1 is g
        assert_eq!(next_phase_label("f1"), "g");
        // empty / no-letter falls back to the first phase
        assert_eq!(next_phase_label(""), "a");
    }

    #[test]
    fn phase_order_is_length_then_lexico() {
        let mut v = vec!["b", "aa", "a", "z", "c"];
        v.sort_by_key(|a| phase_order_key(a));
        assert_eq!(
            v,
            vec!["a", "b", "c", "z", "aa"],
            "aa sorts AFTER z, not after a"
        );
    }

    #[test]
    fn is_phase_label_accepts_tokens_rejects_noise() {
        assert!(is_phase_label("a"));
        assert!(is_phase_label("g"));
        assert!(is_phase_label("aa"));
        assert!(is_phase_label("f1"));
        assert!(!is_phase_label(""));
        assert!(!is_phase_label("1"), "needs a letter");
        assert!(!is_phase_label("a12"), "at most one trailing digit");
        assert!(!is_phase_label("a_b"), "no separators");
    }

    #[test]
    fn discover_is_unbounded_and_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        // lowercase active-sprint convention, beyond the old A-G cap (h, i),
        // plus a multi-letter phase (aa) and a sub-phase (f1).
        for label in ["a", "f1", "h", "i", "aa"] {
            std::fs::write(p.join(format!("sprint79_phase_{label}_review.md")), "x").unwrap();
        }
        // an UPPERCASE archive-style file for the same sprint must also resolve
        std::fs::write(p.join("sprint79_phase_B_review.md"), "x").unwrap();
        // noise that must be ignored
        std::fs::write(p.join("sprint79_phase_a_preflight.md"), "x").unwrap();
        std::fs::write(p.join("sprint79_kickoff.md"), "x").unwrap();

        let labels: Vec<String> = discover_phase_artifacts(p, 79, "review")
            .into_iter()
            .map(|a| a.label)
            .collect();
        // canonical order: single-letter labels first (a,b,h,i), then the
        // length-2 labels lexicographically (aa before f1).
        assert_eq!(labels, vec!["a", "b", "h", "i", "aa", "f1"]);

        // case-insensitive single lookup returns the REAL on-disk path
        let found = find_phase_artifact(p, 79, "B", "review").unwrap();
        assert_eq!(
            found.file_name().unwrap().to_str().unwrap(),
            "sprint79_phase_B_review.md"
        );
        assert!(find_phase_artifact(p, 79, "z", "review").is_none());
    }
}
