// SPDX-License-Identifier: AGPL-3.0-or-later
//! Host-side bridge authority model v0 (Sprint 83 Phase C).
//!
//! Renders AUTHORITATIVE, per call, the intersection that governs whether a
//! sandboxed app may reach a bridge method:
//!
//! ```text
//!   effective = declared-by-release ∩ grant ∩ local-policy   (∩ consent, inert v0)
//! ```
//!
//! Monotone attenuation (the ocap / macaroon rule): each PRESENT layer can only
//! SHRINK the set, never widen it. This is the CANONICAL model; the web shell host
//! mirrors it in TS (`web/src/bridge/authority.ts`) under the parity gate. Phase C
//! wires the host-side **5-layer subset** of the 7-layer model — the pack-version
//! and compiled-plan layers belong to later phases and are deliberately not modelled
//! here (no inert stubs: absence of a layer imposes no constraint, which is exactly
//! the monotone-attenuation identity element).
//!
//! Two regimes, keyed on the app manifest (`SbfbManifest.bridge`):
//!   - `bridge` PRESENT -> `declared = Some(set)` : FAIL-CLOSED on the declared set.
//!   - `bridge` ABSENT  -> `declared = None`      : grandfathered legacy (no release
//!     constraint; the grant governs). Pre-existing test/demo apps that never
//!     declared a bridge config keep working untouched (T-BRIDGE-LEGACY-DEFAULT,
//!     accepted pre-launch residual; the migrate-all-apps flip is post-v1.0).
//!
//! Pure data + set algebra; no crypto, no I/O. `now_ms` is injected so the caller
//! owns the clock (deterministic tests; matches the "no `Date.now()` in core" rule).
//!
//! Off-allowlist handling (both fail-closed): a manifest declaring a method NOT in
//! the allowlist is REJECTED whole at parse ([`crate::SbfbManifest::validate`]), the TS
//! resolver (`resolveDeclaredAuthority`) DROPS the off-list name at resolution, and
//! [`effective_methods`] finally bounds the result to the allowlist on BOTH sides — so
//! an off-list name never AUTHORIZES, whatever path built the grant.
//!
//! Serde is deliberately NOT derived on [`Grant`] in Phase C: no call-site
//! (de)serializes a grant yet. It lands with grant PERSISTENCE (a later phase),
//! where the defaults MUST be fail-closed — a missing `revoked`/`expires_at_ms`
//! must never silently widen a grant, so no permissive `#[serde(default)]`.

use std::collections::BTreeSet;

/// A time-bounded, revocable grant of bridge method capabilities to an app/release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    /// Method names this grant authorizes (a subset of the manifest allowlist).
    pub methods: BTreeSet<String>,
    /// Absolute expiry, ms since the Unix epoch; `None` = never expires.
    pub expires_at_ms: Option<u64>,
    /// Explicit revocation, re-checked on every call (never cached).
    pub revoked: bool,
}

impl Grant {
    /// Methods live at `now_ms`: empty when revoked or expired (fail-closed).
    pub fn live_methods(&self, now_ms: u64) -> BTreeSet<String> {
        if self.revoked || self.is_expired(now_ms) {
            return BTreeSet::new();
        }
        self.methods.clone()
    }

    /// Whether this grant has reached or passed its expiry at `now_ms`.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        matches!(self.expires_at_ms, Some(exp) if now_ms >= exp)
    }
}

/// Release-declaration layer. `Some(set)` = the manifest declared a `bridge` config
/// (fail-closed on this set); `None` = no `bridge` key (grandfathered legacy, no
/// release constraint).
pub type Declared = Option<BTreeSet<String>>;

/// Local-policy layer. `None` = no local constraint yet (v0 inert, permissive);
/// `Some(set)` = an operator allow-list that a later phase tightens.
pub type Policy = Option<BTreeSet<String>>;

/// The effective, currently-authorized method set. Starts from the grant's LIVE
/// methods and intersects every PRESENT constraint layer — a `None` layer imposes no
/// constraint (never widens). Single source of truth for the intersection.
pub fn effective_methods(
    declared: &Declared,
    grant: &Grant,
    policy: &Policy,
    now_ms: u64,
) -> BTreeSet<String> {
    let mut eff = grant.live_methods(now_ms);
    if let Some(d) = declared {
        eff = eff.intersection(d).cloned().collect();
    }
    if let Some(p) = policy {
        eff = eff.intersection(p).cloned().collect();
    }
    // Domain bound (belt-and-suspenders, mirrored in TS `effectiveMethods`): the
    // effective set can never exceed the dispatchable universe (the manifest
    // allowlist), so a grant carrying an off-allowlist name authorizes nothing beyond
    // the host-dispatched verbs — regardless of how the grant was built. The
    // dispatch itself is already bounded upstream by the Zod enum (`req.method` ∈ 16);
    // this is defense in depth, not the sole guard.
    eff.retain(|m| crate::SbfbManifest::bridge_method_allowlist().contains(&m.as_str()));
    eff
}

/// Whether `method` is authorized right now. Fail-closed: undeclared / ungranted /
/// expired / revoked all return `false`, regardless of a syntactically valid request.
pub fn is_authorized(
    declared: &Declared,
    grant: &Grant,
    policy: &Policy,
    method: &str,
    now_ms: u64,
) -> bool {
    effective_methods(declared, grant, policy, now_ms).contains(method)
}

/// The legacy grandfather grant: every NON-sensitive host-dispatched method, no
/// expiry, not revoked. Paired with `declared = None` for an app whose manifest
/// declares no `bridge` config, so a pre-existing app keeps reaching the methods it
/// always could — but never the consent-gated [`crate::SENSITIVE_BRIDGE_METHODS`].
pub fn grandfather_grant() -> Grant {
    Grant {
        methods: crate::SbfbManifest::bridge_method_allowlist()
            .iter()
            .filter(|m| !crate::SENSITIVE_BRIDGE_METHODS.contains(m))
            .map(|m| (*m).to_string())
            .collect(),
        expires_at_ms: None,
        revoked: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|x| (*x).to_string()).collect()
    }

    /// A syntactically valid call to a method the release never declared is refused
    /// (the P0 the phase closes: a valid iframe message is not sufficient authority).
    #[test]
    fn usurpation_refused() {
        let declared = Some(s(&["storage_get"]));
        let grant = Grant {
            methods: s(&["storage_get"]),
            expires_at_ms: None,
            revoked: false,
        };
        assert!(is_authorized(&declared, &grant, &None, "storage_get", 0));
        // Undeclared method — refused despite a well-formed request.
        assert!(!is_authorized(&declared, &grant, &None, "task_submit", 0));
    }

    /// A grant past its expiry authorizes nothing.
    #[test]
    fn expiry_refused() {
        let declared = Some(s(&["search"]));
        let grant = Grant {
            methods: s(&["search"]),
            expires_at_ms: Some(1000),
            revoked: false,
        };
        assert!(is_authorized(&declared, &grant, &None, "search", 999));
        assert!(!is_authorized(&declared, &grant, &None, "search", 1000)); // at expiry
        assert!(!is_authorized(&declared, &grant, &None, "search", 5000)); // past expiry
    }

    /// Revocation is honoured on the very next call (never cached across calls).
    #[test]
    fn revocation_per_call() {
        let declared = Some(s(&["search"]));
        let mut grant = Grant {
            methods: s(&["search"]),
            expires_at_ms: None,
            revoked: false,
        };
        assert!(is_authorized(&declared, &grant, &None, "search", 0));
        grant.revoked = true; // revoked between two calls
        assert!(!is_authorized(&declared, &grant, &None, "search", 0));
    }

    /// A legacy app (no `bridge` key -> `declared = None`) keeps reaching every
    /// host-dispatched method through the grandfather grant, but nothing else.
    #[test]
    fn grandfather_legacy_allows_dispatch_set() {
        let declared: Declared = None;
        let grant = grandfather_grant();
        for m in crate::SbfbManifest::bridge_method_allowlist() {
            let sensitive = crate::SENSITIVE_BRIDGE_METHODS.contains(m);
            assert_eq!(
                is_authorized(&declared, &grant, &None, m, 0),
                !sensitive,
                "grandfather must allow {m} iff it is not sensitive"
            );
        }
        assert!(!is_authorized(&declared, &grant, &None, "not_a_method", 0));
    }

    /// A declared-but-EMPTY release layer (`bridge: { methods: [] }`, e.g. the
    /// workflow-hub scaffold) is fail-closed on EVERYTHING — even with a full grant.
    #[test]
    fn declared_empty_refuses_all() {
        let declared: Declared = Some(BTreeSet::new());
        let grant = grandfather_grant(); // every method granted, yet...
        for m in crate::SbfbManifest::bridge_method_allowlist() {
            assert!(
                !is_authorized(&declared, &grant, &None, m, 0),
                "empty declared must refuse {m}"
            );
        }
    }

    /// A grant carrying an off-allowlist name (however it was built) authorizes
    /// NOTHING beyond the dispatchable verbs — the domain bound holds even with
    /// `declared = None` (grandfather-shaped) and no policy.
    #[test]
    fn off_allowlist_grant_authorizes_nothing_extra() {
        let mut methods: BTreeSet<String> = crate::SbfbManifest::bridge_method_allowlist()
            .iter()
            .map(|m| (*m).to_string())
            .collect();
        methods.insert("evil_method".to_string());
        let grant = Grant {
            methods,
            expires_at_ms: None,
            revoked: false,
        };
        assert!(!is_authorized(&None, &grant, &None, "evil_method", 0));
        assert!(is_authorized(&None, &grant, &None, "search", 0));
    }

    /// The policy layer can only ATTENUATE (monotone): a present policy shrinks the
    /// set, an absent one (`None`) imposes no constraint.
    #[test]
    fn policy_layer_attenuates_only() {
        let declared = Some(s(&["storage_get", "storage_set"]));
        let grant = Grant {
            methods: s(&["storage_get", "storage_set"]),
            expires_at_ms: None,
            revoked: false,
        };
        assert_eq!(
            effective_methods(&declared, &grant, &None, 0),
            s(&["storage_get", "storage_set"])
        );
        let policy = Some(s(&["storage_get"]));
        assert_eq!(
            effective_methods(&declared, &grant, &policy, 0),
            s(&["storage_get"])
        );
    }
}
