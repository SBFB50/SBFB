// SPDX-License-Identifier: AGPL-3.0-or-later
//! Runtime node-instance identity — the pure extension of the STRUCTURAL
//! `node_instance_id` of a compiled plan with map/repeat iteration indices
//! (Sprint 83 Phase J, preflight DÉC-2).
//!
//! ## Why two identities exist
//!
//! The Phase H compiler expands each node OCCURRENCE once per structural
//! path: the body of a `map` or `repeat` appears ONE time in
//! `CompiledPlan::node_instances`, under the scope of its expanding node
//! (`/variant-fanout/cv-in`). At runtime a map fans out N iterations and a
//! repeat loops up to its bound — and research §12.2 requires that two
//! iterations NEVER share an identity of effect. The runtime therefore
//! derives an EXTENDED id per iteration, while every plan lookup
//! (`capability_closure`, `node_budgets`, `human_gates`, `node_instances`)
//! stays keyed by the STRUCTURAL id.
//!
//! ## The encoding is unambiguous by construction
//!
//! Structural ids are RFC 6901 pointers: each segment escapes `~` as `~0`
//! and `/` as `~1` (compile.rs `escape_segment`). The escaped alphabet can
//! therefore never contain the two-character sequence `~2` — which is
//! exactly why `~2<index>` is the iteration marker: appending it to a scope
//! segment cannot collide with any structural id, and stripping every
//! `~2<digits>` suffix is a pure inverse. No separator that a node id could
//! legitimately contain (`#`, `@`, …) has that property.
//!
//! Both directions are total functions on valid inputs and are used by the
//! scheduler on EVERY dispatch, so they live here rather than in test
//! support.

/// The iteration marker appended to a scope segment: `~2` followed by the
/// decimal index. `~2` is unreachable in RFC 6901-escaped structural
/// segments (only `~0` and `~1` exist there), so the marker cannot collide.
const ITERATION_MARKER: &str = "~2";

/// Extend one SCOPE occurrence id with an iteration index:
/// `/variant-fanout` + 3 → `/variant-fanout~23`.
///
/// The extension applies to the EXPANDING node's occurrence id (the map or
/// repeat), never to a leaf: children of iteration 3 of `/variant-fanout`
/// are `/variant-fanout~23/cv-in`, obtained by extending the scope then
/// re-appending the child's remaining path — see [`extend_child_id`].
pub fn extend_scope(scope_occurrence_id: &str, iteration: u32) -> String {
    format!("{scope_occurrence_id}{ITERATION_MARKER}{iteration}")
}

/// Extend a CHILD structural id that lives under `scope` with an iteration
/// of that scope: `extend_child_id("/variant-fanout", 2,
/// "/variant-fanout/cv-in")` → `/variant-fanout~22/cv-in`.
///
/// Returns `None` when `child_structural_id` does not lie under `scope` —
/// the caller mixed up scopes, and silently returning the input would let
/// one iteration's effect memoise under another identity.
pub fn extend_child_id(
    scope_occurrence_id: &str,
    iteration: u32,
    child_structural_id: &str,
) -> Option<String> {
    let rest = child_structural_id.strip_prefix(scope_occurrence_id)?;
    if !rest.is_empty() && !rest.starts_with('/') {
        // `/variant-fanout-extra/x` starts with `/variant-fanout` as a
        // STRING but not as a PATH — refuse the prefix illusion.
        return None;
    }
    Some(format!(
        "{}{ITERATION_MARKER}{iteration}{rest}",
        scope_occurrence_id
    ))
}

/// Project a RUNTIME id back to its STRUCTURAL id by stripping every
/// `~2<digits>` iteration suffix from every segment. Pure inverse of the
/// extension: plan lookups (`node_instances`, `capability_closure`,
/// `node_budgets`, `human_gates`) are keyed structurally.
///
/// The scan works on BYTES (the marker and its digits are ASCII, and UTF-8
/// guarantees no continuation byte ever equals an ASCII byte), but bytes
/// that are not consumed by a marker are copied VERBATIM — an earlier
/// byte-as-`char` rebuild silently Latin-1-mangled any non-ASCII node id
/// into a key no plan contains (review r1 J-DIFF-2; node ids are
/// schema-unconstrained strings, so `/étape` is a valid signed id).
pub fn structural_id(runtime_id: &str) -> String {
    let mut out = Vec::with_capacity(runtime_id.len());
    let bytes = runtime_id.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'~' && i + 1 < bytes.len() && bytes[i + 1] == b'2' {
            // Consume `~2` and the decimal index that follows. At least one
            // digit is required — a bare `~2` was never produced by
            // `extend_scope`, and passing it through unchanged keeps the
            // function total without inventing a meaning for it.
            let digits_start = i + 2;
            let mut j = digits_start;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > digits_start {
                i = j;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    // Removing whole ASCII sequences from valid UTF-8 leaves valid UTF-8.
    String::from_utf8(out).expect("stripping ASCII markers preserves UTF-8")
}

/// The scope an id lives in: everything before its last `/` segment (the
/// root scope is `""`). Shared by the reducer and the orchestrator — one
/// definition, not two re-spellings (review r1 J-DIFF-3).
pub fn parent_scope(id: &str) -> &str {
    match id.rfind('/') {
        Some(pos) => &id[..pos],
        None => "",
    }
}

/// The STRUCTURAL id of a node id inside a scope — compile.rs
/// `instance_id`, re-spelt ONCE here for runtime consumers (join source
/// references name plain node ids of their scope).
pub fn child_structural(scope_structural: &str, node_id: &str) -> String {
    let escaped = node_id.replace('~', "~0").replace('/', "~1");
    format!("{scope_structural}/{escaped}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extend_and_project_are_inverse() {
        let scope = "/variant-fanout";
        for iteration in [0u32, 1, 7, 42, 1000] {
            let extended = extend_scope(scope, iteration);
            assert_eq!(structural_id(&extended), scope);
        }
        let child = extend_child_id("/variant-fanout", 3, "/variant-fanout/cv-in")
            .expect("child under scope");
        assert_eq!(child, "/variant-fanout~23/cv-in");
        assert_eq!(structural_id(&child), "/variant-fanout/cv-in");
    }

    #[test]
    fn nested_iterations_project_fully() {
        // A repeat inside a map: both suffixes strip.
        let id = "/fanout~22/inner-loop~25/step";
        assert_eq!(structural_id(id), "/fanout/inner-loop/step");
    }

    #[test]
    fn two_iterations_never_share_an_identity() {
        let a = extend_child_id("/m", 1, "/m/x").expect("under scope");
        let b = extend_child_id("/m", 2, "/m/x").expect("under scope");
        assert_ne!(a, b);
        assert_eq!(structural_id(&a), structural_id(&b));
    }

    /// The marker cannot collide with an ESCAPED structural segment: a node
    /// id containing `~2` literally is escaped to `~02` by RFC 6901, which
    /// the projection leaves intact.
    #[test]
    fn escaped_structural_tilde_survives_projection() {
        // compile.rs escape_segment("a~2b") == "a~02b".
        let structural = "/a~02b/leaf";
        assert_eq!(structural_id(structural), structural);
    }

    #[test]
    fn prefix_illusion_is_refused() {
        assert_eq!(
            extend_child_id("/variant-fanout", 1, "/variant-fanout-extra/x"),
            None
        );
        assert_eq!(extend_child_id("/a", 1, "/b/x"), None);
    }

    /// A bare `~2` with no digits was never produced by the extension; the
    /// projection passes it through rather than inventing a strip rule.
    #[test]
    fn bare_marker_without_digits_passes_through() {
        assert_eq!(structural_id("/x~2/y"), "/x~2/y");
    }

    /// Node ids are schema-unconstrained strings: a non-ASCII id must
    /// survive extension + projection byte-identically (review r1 J-DIFF-2
    /// red arm — the byte-as-char rebuild corrupted it).
    #[test]
    fn non_ascii_ids_survive_projection() {
        let structural = "/étape-clé/pièce";
        assert_eq!(structural_id(structural), structural);
        let extended = extend_child_id("/étape-clé", 2, "/étape-clé/pièce").expect("under scope");
        assert_eq!(extended, "/étape-clé~22/pièce");
        assert_eq!(structural_id(&extended), structural);
    }

    #[test]
    fn parent_scope_and_child_structural_are_inverse_shaped() {
        assert_eq!(parent_scope("/a/b"), "/a");
        assert_eq!(parent_scope("/a"), "");
        assert_eq!(parent_scope(""), "");
        assert_eq!(child_structural("/m", "x/y~z"), "/m/x~1y~0z");
    }
}
