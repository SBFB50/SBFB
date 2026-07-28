// SPDX-License-Identifier: AGPL-3.0-or-later
//! Effective-capability computation (Sprint 83 Phase L): the host-neutral,
//! monotone-attenuating intersection of the SEVEN authority layers of
//! research §14.1, over SEVENTEEN typed capability classes (§14.2).
//!
//! ```text
//!   effective_capabilities =
//!       OS capability toggle                       (layer 1, host-injected)
//!     ∩ local node policy                          (layer 2, host-injected)
//!     ∩ grants to the app RELEASE                  (layer 3, data)
//!     ∩ grants to the pack VERSION                 (layer 4, data)
//!     ∩ the compiled plan's capability closure     (layer 5, H-delivered data)
//!     ∩ the CURRENT EFFECT's declared need         (layer 6, data)
//!     ∩ a still-VALID HITL decision                (layer 7, data — empty in L)
//! ```
//!
//! ## The one invariant, stated precisely
//!
//! Each PRESENT layer can only SHRINK the authorized set — never widen it.
//! Neither the app, the pack, an agent, nor an MCP server can enlarge what
//! the intersection yields (research §14.1, §19.3 Plan-Then-Execute). This
//! is the ocap / macaroon / Biscuit rule: composition is conjunctive, so
//! stacking more constraints is monotone attenuation. The module PROVES it
//! by property tests (`intersection_only_attenuates`, the algebraic laws),
//! not by assertion.
//!
//! ## Mirror of `grant_v0`, never a shared function (the real danger)
//!
//! The canonical attenuation algebra already lives in
//! `sbfb-manifest::grant_v0` (Phase C), but its result is bounded to the
//! **16 bridge verbs** (`grant_v0::effective_methods` retains only the
//! bridge allowlist) — the WRONG domain for the 17 effect classes here, and
//! this engine crate deliberately does not depend on `sbfb-manifest`. So
//! `effective_capabilities` is NECESSARILY a second algebra. The discipline
//! is to reproduce grant_v0's DOCUMENTED conventions VERBATIM — a `None`
//! layer is the attenuation identity element (never a widening), expiry is
//! fail-closed AT THE BOUND (`now_ms >= exp`), `revoked` is re-checked on
//! every call and never cached, and the result is domain-bounded on BOTH
//! sides (a grant for a ref outside the plan closure authorizes nothing) —
//! and to PIN those conventions with tests, not to reach across crates for a
//! function whose domain does not fit. Generalising grant_v0 to serve both
//! callers was rejected: it would touch the frozen Phase C pair
//! `grant_v0.rs` ↔ `web/src/bridge/authority.ts` whose ALGEBRA parity is not
//! machine-gated (`check-frontier-contracts.sh` covers only allowlist
//! parity), so a refactor would drift the TS mirror silently.
//!
//! ## Field-level meet: direction PER FIELD
//!
//! A capability is not just a class tag: a §14.2 ref carries destinations,
//! quotas, a timeout, a data policy and an approval level. The intersection
//! is therefore a MEET defined field by field, and the direction of "more
//! restrictive" INVERTS per field:
//!   - destinations: set INTERSECTION (fewer reachable endpoints = stricter);
//!   - quota, timeout: `min` (a smaller ceiling is stricter);
//!   - approval level: `max` (a HIGHER requirement is stricter);
//!   - data policy: UNION of independent constraint flags (each `no_egress` /
//!     `no_persist` OR'd — never a total `max` that would drop an orthogonal
//!     axis).
//!
//! A `min` copy-pasted onto the approval level would be an accidental
//! WIDENING invisible to a class-set-only test — which is exactly why the
//! model carries the fields and the property tests exercise each direction.
//!
//! ## Positive authority vs attenuators (where "absent = identity" applies)
//!
//! Not every layer is an attenuator with an identity element. The GRANT
//! layers (3 release, 4 pack) are POSITIVE authority: a ref is authorized
//! only if a live grant confers it in BOTH audiences. Absent, empty, expired
//! or all-inert grants for an audience confer NOTHING, so the ref drops —
//! fail-closed, the secure default of a capability system ("no grant = no
//! authority", the digest-≠-read-right posture). This mirrors grant_v0,
//! where the grant is the mandatory BASE (`grant.live_methods`, empty when
//! revoked/expired) and only `declared`/`policy` are the `None`-is-identity
//! attenuators.
//!
//! The ATTENUATOR layers do follow grant_v0's Option doctrine exactly: the
//! absence of the layer imposes no constraint (the identity element), while
//! the absence of a matching entry WITHIN a present layer is fail-closed.
//! [`ClassConstraint::Unconstrained`] / [`RefConstraint::Unconstrained`] /
//! [`EffectNeed::Any`] are the identities of layers 1, 2 and 6.
//!
//! Layer 7 is the sharp case of "empty layer ≠ identity": the HITL-decisions
//! set is EMPTY in Phase L (no host chrome produces one yet — that is Phase
//! L1). An empty layer-7 is NOT identity — it drops every capability whose
//! effective approval level is `Human`, because a human-approval-requiring
//! effect with no valid decision must yield the empty set, never pass
//! through. A capability whose approval is `Auto` (or `Notify`, non-blocking)
//! is unaffected. Confusing "no layer" with "empty layer" is the fail-open
//! bug this distinction closes.
//!
//! ## Charter — pure, host-neutral, clockless-by-injection
//!
//! No I/O, no crypto, no `serde` derive (nothing here is serialized —
//! census of `DOMAIN_*_V1` and every wire `*_FORMAT_VERSION` are untouched),
//! no ambient environment read. The OS toggle and the local policy come from
//! an injected [`CapabilityPolicy`] trait (the seam already used by
//! [`crate::clock::Clock`] and [`crate::signal::SignalSource`]); `now_ms` is
//! a parameter. This module is the authoritative COMPUTATION only — wiring
//! the intersection into the effect-dispatch path (`orchestrator::run_effect`)
//! and journaling authority refusals belongs to the phase that ships the run
//! surface (DS-1 of the Phase L preflight); the retry/replay re-authorization
//! notes live in that phase's design, not as call-sites here.
//!
//! ## Out of scope, honestly bounded
//!
//! Delegation depth (`DelegationGrant.v1`, research §14.5) is a post-MVP
//! jump; the only "depth" this module bounds is the monotone chain of layers
//! (`depth_enforced`: stacking one more layer can only attenuate). The rich
//! ref descriptor's port SCHEMAS are the compiler's concern (port typing is
//! Phase G/H), not re-modeled here. Secret binding and the signed HITL
//! principal LANDED in Phase L1 ([`crate::secret`], [`crate::hitl`]): layer 7
//! still carries only an OPAQUE decision reference here, and the PRODUCER of
//! those references now exists — but nothing wires it yet (0 call-site, the
//! same DS-1 posture as this module).
//!
//! ## Notes for the binding phase (DS-1)
//!
//! When a later phase wires this computation into the effect dispatch, three
//! properties of the CALLER (not of this module) must hold, or a fail-closed
//! drop becomes a surprise:
//! - The approval/data-policy FLOOR of a capability must come from `declared`
//!   / the compiled plan (a layer that meets by `max`/union), never rest on a
//!   single co-audience grant — a well-formed grant set carries ONE bound per
//!   ref per audience, so the caller should not rely on the multi-grant
//!   combination to impose a floor.
//! - A well-formed grant set has AT MOST ONE bound per ref per audience.
//!   Several same-audience grants naming the same ref combine ORDER-
//!   INDEPENDENTLY: if ONE bound dominates all the others (every bound refines
//!   it) that dominating grant is kept; otherwise (a genuinely incomparable
//!   set with no maximum, or a class mismatch) the ref is DROPPED fail-closed.
//!   Because dropping is SILENT and monotone-surprising — adding a grant can
//!   REMOVE authority — the binding phase should treat a ref with several
//!   incomparable same-audience grants as a CONFIG ERROR surfaced to the
//!   operator, not rely on the silent drop (Codex r2). The caller owns keeping
//!   the grant set well-formed.
//! - Layer 5 takes `plan_closure: &BTreeSet<String>` — the KEYS of the
//!   compiler's `capability_closure: BTreeMap<String, Vec<String>>`. Passing
//!   the values would yield an empty closure and drop everything (fail-closed
//!   but wrong).
//! - The PROVENANCE of layer 7's set is a caller obligation too (ex-review r2
//!   R2). [`HitlValidity`] may only be populated with [`DecidedRef`]s obtained
//!   from [`crate::hitl::decide_ref`]. Building it from the PENDING queue — the
//!   shortest path from a queue of requests to a set of strings — would let
//!   every `Human` capability with an unanswered (or expired, or REFUSED)
//!   request cross layer 7, consulting neither the signature, nor the expiry,
//!   nor the verdict, nor the ref binding. `pub(crate)` makes that
//!   uncompilable from another crate but NOT from this one.
//!
//! ## Layer 7 is inert on BOTH sides, not just one
//!
//! The DECISION side being empty is disclosed above. The NEED side is just as
//! empty and easier to miss: no production code constructs a
//! [`BoundedCapability`] or sets [`ApprovalLevel::Human`] anywhere (the pack
//! format has no class or approval field at all — every test here builds those
//! values itself). So the binding phase must ship the approval FLOOR together
//! with the producer of decisions. Wire only the decisions and layer 7 gates
//! nothing, forever, silently — a no-op that no test can catch, since the
//! fail-closed drop it is supposed to perform never has an input.

use std::collections::{BTreeMap, BTreeSet};

/// The seventeen typed capability classes (research §14.2). A capability is
/// authorized by TYPED class identity and typed field bounds — never by a
/// substring of a method name. Two of these classes (`Commit`, `Push`) share
/// a spelling with the Operator's `SENSITIVE_ACTIONS` substring gate, which
/// is precisely why authorization here is by enum, not by string matching:
/// this engine crate does not import `operator_server::SENSITIVE_ACTIONS` and
/// performs no substring authorization at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityClass {
    /// Read public, unrestricted data.
    ReadPublic,
    /// Read private data within an explicit scope.
    ReadPrivateScoped,
    /// Write a draft (reversible, non-authoritative).
    WriteDraft,
    /// Write into a workspace (files under a fenced root).
    WriteWorkspace,
    /// Reach a FIXED, pre-declared network destination.
    NetworkFixedDestination,
    /// Reach an open set of network destinations.
    NetworkOpen,
    /// Use remote compute (sharded / networked workers).
    RemoteCompute,
    /// Invoke a local model.
    ModelLocal,
    /// Invoke a cloud model.
    ModelCloud,
    /// Call a native (in-process) tool.
    ToolNative,
    /// Call an MCP tool (external, permissioned adapter).
    ToolMcp,
    /// Use a secret (bound just-in-time by the host keystore — the binding
    /// itself is Phase L1; here it is only a capability class).
    SecretUse,
    /// Publish to the network.
    Publish,
    /// Install a pack/app.
    Install,
    /// Commit to a repository.
    Commit,
    /// Push to a remote.
    Push,
    /// A destructive operation (delete / overwrite / irreversible).
    Destructive,
}

impl CapabilityClass {
    /// Every class, in declaration order. The length of this slice is the
    /// authoritative "17" the gate and `seventeen_classes_typed` check.
    pub const ALL: [CapabilityClass; 17] = [
        CapabilityClass::ReadPublic,
        CapabilityClass::ReadPrivateScoped,
        CapabilityClass::WriteDraft,
        CapabilityClass::WriteWorkspace,
        CapabilityClass::NetworkFixedDestination,
        CapabilityClass::NetworkOpen,
        CapabilityClass::RemoteCompute,
        CapabilityClass::ModelLocal,
        CapabilityClass::ModelCloud,
        CapabilityClass::ToolNative,
        CapabilityClass::ToolMcp,
        CapabilityClass::SecretUse,
        CapabilityClass::Publish,
        CapabilityClass::Install,
        CapabilityClass::Commit,
        CapabilityClass::Push,
        CapabilityClass::Destructive,
    ];
}

/// The approval level a capability requires. ORDERED from least to most
/// oversight; the meet takes the MAXIMUM (a layer can raise the requirement,
/// never lower it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ApprovalLevel {
    /// No approval; the effect may run unattended.
    Auto,
    /// The human is NOTIFIED but not blocked (non-blocking).
    Notify,
    /// A valid host-signed HITL decision is REQUIRED before the effect runs
    /// (the decision itself is Phase L1; layer 7 gates on its presence).
    Human,
}

/// A per-field ceiling that attenuates by `min`. `Unbounded` is the identity
/// (no ceiling); `AtMost(n)` caps at `n`, and the meet of two ceilings is the
/// tighter (smaller) one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ceiling {
    /// No ceiling — the attenuation identity for this field.
    Unbounded,
    /// A ceiling of at most `n`.
    AtMost(u64),
}

impl Ceiling {
    /// The tighter of two ceilings (`min`-attenuating). `Unbounded` is the
    /// identity element.
    pub fn meet(self, other: Ceiling) -> Ceiling {
        match (self, other) {
            (Ceiling::Unbounded, x) | (x, Ceiling::Unbounded) => x,
            (Ceiling::AtMost(a), Ceiling::AtMost(b)) => Ceiling::AtMost(a.min(b)),
        }
    }

    /// Whether `self` is at least as tight as `other` (`self ⊑ other` for
    /// this field): a smaller-or-equal ceiling refines a larger one, and any
    /// ceiling refines `Unbounded`.
    pub fn refines(self, other: Ceiling) -> bool {
        match (self, other) {
            (_, Ceiling::Unbounded) => true,
            (Ceiling::Unbounded, Ceiling::AtMost(_)) => false,
            (Ceiling::AtMost(a), Ceiling::AtMost(b)) => a <= b,
        }
    }
}

/// The set of reachable destinations, attenuating by INTERSECTION. `Any` is
/// the identity (no destination constraint); `Only(set)` restricts to `set`,
/// and the meet of two is the intersection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationBound {
    /// No destination constraint — the attenuation identity for this field.
    Any,
    /// Reachable destinations restricted to this explicit (possibly empty)
    /// set. An empty set reaches nothing.
    Only(BTreeSet<String>),
}

impl DestinationBound {
    /// The intersection of two destination bounds (attenuating). `Any` is the
    /// identity element.
    pub fn meet(&self, other: &DestinationBound) -> DestinationBound {
        match (self, other) {
            (DestinationBound::Any, x) | (x, DestinationBound::Any) => x.clone(),
            (DestinationBound::Only(a), DestinationBound::Only(b)) => {
                DestinationBound::Only(a.intersection(b).cloned().collect())
            }
        }
    }

    /// Whether `self` refines `other` (`self ⊑ other`): a subset refines a
    /// superset, and any bound refines `Any`.
    pub fn refines(&self, other: &DestinationBound) -> bool {
        match (self, other) {
            (_, DestinationBound::Any) => true,
            (DestinationBound::Any, DestinationBound::Only(_)) => false,
            (DestinationBound::Only(a), DestinationBound::Only(b)) => a.is_subset(b),
        }
    }
}

/// A data-handling policy: INDEPENDENT constraint flags, NOT a linear order.
/// Locality (no egress) and persistence (no persist) are ORTHOGONAL axes — a
/// datum can be ephemeral yet still leave the node in transit, or stay local
/// yet be persisted. A single ordered enum (`Unrestricted < LocalOnly <
/// Ephemeral`) with a `max` meet would silently DROP a locality constraint
/// when meeting `LocalOnly` with `Ephemeral` (the "stricter" `Ephemeral`
/// does not imply `LocalOnly`). Modelling each axis as its own flag, meeting
/// by UNION of constraints, keeps every constraint any layer imposed — the
/// same per-field care PA-5 raised for the approval level. The partial order
/// is by flag-wise `>=`; there is deliberately NO total `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DataPolicy {
    /// The data must stay within the local node (no egress).
    pub no_egress: bool,
    /// The data must not be persisted beyond the effect.
    pub no_persist: bool,
}

impl DataPolicy {
    /// No data-handling restriction — the attenuation identity (all flags off).
    pub const UNRESTRICTED: DataPolicy = DataPolicy {
        no_egress: false,
        no_persist: false,
    };

    /// The meet: the UNION of the two policies' constraints (each flag OR'd).
    /// More constraints = stricter; no constraint any layer imposed is ever
    /// lost. Commutative, associative, idempotent; `UNRESTRICTED` is identity.
    pub fn meet(self, other: DataPolicy) -> DataPolicy {
        DataPolicy {
            no_egress: self.no_egress || other.no_egress,
            no_persist: self.no_persist || other.no_persist,
        }
    }

    /// Whether `self` is at least as strict as `other` (`self ⊑ other`): it
    /// carries every constraint `other` carries (flag-wise `>=`).
    pub fn refines(self, other: DataPolicy) -> bool {
        self.no_egress >= other.no_egress && self.no_persist >= other.no_persist
    }
}

/// A bounded capability: a class plus the field ceilings that attenuate
/// independently. This is the value each layer constrains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedCapability {
    /// Which class of effect this authorizes.
    pub class: CapabilityClass,
    /// Reachable destinations (network / tool). `Any` = unconstrained.
    pub destinations: DestinationBound,
    /// A ceiling on the number of invocations. `Unbounded` = no ceiling.
    pub quota: Ceiling,
    /// A ceiling on the effect's wall-clock budget in ms. `Unbounded` = no
    /// ceiling.
    pub timeout_ms: Ceiling,
    /// The data-handling policy (union-attenuating, orthogonal flags).
    pub data_policy: DataPolicy,
    /// The required approval level (max-attenuating).
    pub approval: ApprovalLevel,
}

impl BoundedCapability {
    /// The most permissive capability of a class: no ceilings, unrestricted
    /// data, no approval. The top of the field lattice for that class — every
    /// layer can only attenuate away from it.
    pub fn top(class: CapabilityClass) -> BoundedCapability {
        BoundedCapability {
            class,
            destinations: DestinationBound::Any,
            quota: Ceiling::Unbounded,
            timeout_ms: Ceiling::Unbounded,
            data_policy: DataPolicy::UNRESTRICTED,
            approval: ApprovalLevel::Auto,
        }
    }

    /// The field-wise meet of two capabilities of the SAME class. Returns
    /// `None` for a class mismatch — an incompatible pair intersects to
    /// nothing (fail-closed), never to a silently coerced capability.
    pub fn meet(&self, other: &BoundedCapability) -> Option<BoundedCapability> {
        if self.class != other.class {
            return None;
        }
        Some(BoundedCapability {
            class: self.class,
            destinations: self.destinations.meet(&other.destinations),
            quota: self.quota.meet(other.quota),
            timeout_ms: self.timeout_ms.meet(other.timeout_ms),
            // stricter-wins fields: the data policy meets by union of
            // constraints (never loses one), the approval takes the max.
            data_policy: self.data_policy.meet(other.data_policy),
            approval: self.approval.max(other.approval),
        })
    }

    /// Whether `self` refines `other` (`self ⊑ other`): same class, every
    /// `min`/intersection field at least as tight, every `max` field at least
    /// as high.
    pub fn refines(&self, other: &BoundedCapability) -> bool {
        self.class == other.class
            && self.destinations.refines(&other.destinations)
            && self.quota.refines(other.quota)
            && self.timeout_ms.refines(other.timeout_ms)
            && self.data_policy.refines(other.data_policy)
            && self.approval >= other.approval
    }
}

/// The effective, currently-authorized capabilities: `capability_ref →
/// bounded capability`. Fewer keys and tighter bounds = more attenuated.
/// `inner` is private and there is no public `insert`/`widen`/`extend`, so no
/// path can ENLARGE an already-computed set. Honest bound (Codex r3): the ONE
/// arbitrary constructor, [`from_declared`], produces the SAME type as the
/// result — a computed authority and a caller-forged candidate are not
/// distinguishable at the type level yet. The binding phase (DS-1) owns a
/// non-forgeable result newtype so a future dispatch API cannot accept a
/// forged set (T-CAP-WIDEN-BYPASS is thus CLOSED for layer widening, BOUNDED
/// for result forgeability).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectiveCapabilities {
    inner: BTreeMap<String, BoundedCapability>,
}

impl EffectiveCapabilities {
    /// Build from a ref → capability map (the plan-declared candidate set;
    /// layer 5/6 source). This is the ONLY constructor that takes arbitrary
    /// entries — it is the maximal starting universe, and every layer after
    /// it can only attenuate.
    pub fn from_declared(map: BTreeMap<String, BoundedCapability>) -> EffectiveCapabilities {
        EffectiveCapabilities { inner: map }
    }

    /// The empty set — the absorbing element of the meet.
    pub fn empty() -> EffectiveCapabilities {
        EffectiveCapabilities {
            inner: BTreeMap::new(),
        }
    }

    /// Read the bounded capability authorized for `capability_ref`, if any.
    pub fn get(&self, capability_ref: &str) -> Option<&BoundedCapability> {
        self.inner.get(capability_ref)
    }

    /// The authorized capability refs.
    pub fn refs(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Number of authorized capability refs.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether nothing is authorized.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The meet of two effective sets: keys are the INTERSECTION, and each
    /// shared ref is the field-wise meet of the two bounds. A shared ref
    /// whose bounds have incompatible classes is DROPPED (fail-closed) — it
    /// authorizes nothing rather than being coerced. Commutative and
    /// associative (proven by property test).
    pub fn meet(&self, other: &EffectiveCapabilities) -> EffectiveCapabilities {
        let mut out = BTreeMap::new();
        for (k, a) in &self.inner {
            if let Some(m) = other.inner.get(k).and_then(|b| a.meet(b)) {
                out.insert(k.clone(), m);
            }
        }
        EffectiveCapabilities { inner: out }
    }

    /// Whether `self` refines `other` (`self ⊑ other`): every ref of `self`
    /// is present in `other` with a bound `self` refines. The monotone-
    /// attenuation order — `apply(layer, caps).refines(caps)` for every layer.
    pub fn refines(&self, other: &EffectiveCapabilities) -> bool {
        self.inner.iter().all(|(k, a)| match other.inner.get(k) {
            Some(b) => a.refines(b),
            None => false,
        })
    }

    /// Drop every ref whose bound does not pass `keep` (a filter layer). The
    /// result always refines the input (removing refs and never touching the
    /// survivors' bounds).
    fn filtered<F: Fn(&str, &BoundedCapability) -> bool>(&self, keep: F) -> EffectiveCapabilities {
        EffectiveCapabilities {
            inner: self
                .inner
                .iter()
                .filter(|(k, v)| keep(k, v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// The seven layers.
// ---------------------------------------------------------------------------

/// Layer 1 / a class-scoped constraint. `Unconstrained` is the identity (the
/// OS permits every class); `Only(set)` permits exactly the listed classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassConstraint {
    /// No class constraint — the attenuation identity.
    Unconstrained,
    /// Only these classes are permitted.
    Only(BTreeSet<CapabilityClass>),
}

/// Layer 2 / a ref-scoped constraint that may also tighten field bounds.
/// `Unconstrained` is the identity; `Only(map)` restricts to the listed refs
/// AND meets each survivor with the policy's bound (so an operator can both
/// deny refs and tighten the ones it allows).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefConstraint {
    /// No ref constraint — the attenuation identity.
    Unconstrained,
    /// Only these refs are permitted, each meet-tightened by its bound.
    Only(BTreeMap<String, BoundedCapability>),
}

/// The audience a grant is scoped to. A release-audience grant NEVER
/// satisfies the pack-version layer and vice versa (T-CAP-GRANT-AUDIENCE-
/// CONFUSION): applying a grant to the wrong layer contributes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantAudience {
    /// Granted to the app RELEASE (layer 3).
    Release,
    /// Granted to the pack VERSION (layer 4).
    Pack,
}

/// A time-bounded, revocable, AUDIENCE-scoped grant of bounded capabilities
/// (the layer-3 / layer-4 source). Mirrors `grant_v0::Grant`'s conventions:
/// `expires_at_ms` is fail-closed at the bound (`now_ms >= exp`), `revoked`
/// is re-checked on every call and never cached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGrant {
    /// Which layer this grant serves; it contributes to NO other audience.
    pub audience: GrantAudience,
    /// The bounded capabilities this grant confers, `capability_ref → bound`.
    pub granted: BTreeMap<String, BoundedCapability>,
    /// Absolute expiry, ms since the Unix epoch; `None` = never expires.
    pub expires_at_ms: Option<u64>,
    /// Explicit revocation, re-checked on every call (never cached).
    pub revoked: bool,
}

impl CapabilityGrant {
    /// Whether this grant has reached or passed its expiry at `now_ms`
    /// (fail-closed AT the bound, mirroring `grant_v0::Grant::is_expired`).
    pub fn is_expired(&self, now_ms: u64) -> bool {
        matches!(self.expires_at_ms, Some(exp) if now_ms >= exp)
    }

    /// The grant's live contribution FOR a given audience layer: its granted
    /// map when the audience matches and the grant is neither revoked nor
    /// expired, else nothing. A grant offered to the wrong audience layer is
    /// inert (audience confusion closed here).
    pub fn live_for(
        &self,
        audience: GrantAudience,
        now_ms: u64,
    ) -> Option<&BTreeMap<String, BoundedCapability>> {
        if self.audience == audience && !self.revoked && !self.is_expired(now_ms) {
            Some(&self.granted)
        } else {
            None
        }
    }
}

/// Layer 6 / the current effect's declared need. `Any` is the identity (no
/// per-effect narrowing — used when computing the whole authority);
/// `One(ref)` narrows to the single ref the current effect needs (everything
/// else drops).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectNeed {
    /// No per-effect narrowing — the identity (whole-authority computation).
    Any,
    /// Narrow to exactly this capability ref.
    One(String),
}

/// A capability ref that PASSED the HITL gate: minted only by
/// [`crate::hitl::decide_ref`], after a host-verified, granted, unexpired
/// approval bound to that very ref.
///
/// Its constructor is `pub(crate)` ON PURPOSE (ex-review r2 R2). Typing
/// `decide_ref`'s INPUT while leaving its OUTPUT set publicly constructible
/// would lock a door next to an open wall: an out-of-crate caller could write
/// `HitlValidity { decided_refs: … }` and grant itself layer-7 clearance
/// without ever presenting an approval. Now the only way for another crate to
/// obtain one is to go through the gate.
///
/// SCOPE OF THAT GUARANTEE, stated because it is a type property and no test
/// can fail on it: `pub(crate)` closes the door for other CRATES, not for other
/// MODULES of this one. The wiring this crate documents (`orchestrator`) could
/// still write `HitlValidity::from_decided([DecidedRef::new(k)])` and skip the
/// gate entirely — it would compile, and nothing here would go red. Whoever
/// wires layer 7 must call [`crate::hitl::decide_ref`]; minting a `DecidedRef`
/// directly is the same bypass this newtype exists to prevent, spelled from
/// inside.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecidedRef(String);

impl DecidedRef {
    /// Mint a decided ref. Crate-internal: the ONLY legitimate caller is
    /// [`crate::hitl::decide_ref`], which mints it after `verify_binding`.
    pub(crate) fn new(capability_ref: impl Into<String>) -> DecidedRef {
        DecidedRef(capability_ref.into())
    }

    /// The capability ref that was decided.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Layer 7 / the set of capability refs that currently hold a VALID host
/// HITL decision. An empty set is NOT the identity: it drops every capability
/// whose effective approval is [`ApprovalLevel::Human`] (fail-closed).
/// Capabilities with `Auto`/`Notify` approval are unaffected.
///
/// Since Phase L1 the PRODUCER exists — [`crate::hitl::decide_ref`] turns a
/// host-verified, granted, unexpired approval bound to THAT capability ref into
/// a member of this set (`hitl::tests::decided_ref_gates_layer_seven` proves the
/// composition). It stays EMPTY in production until the run surface wires it:
/// no production code builds a `HitlValidity` yet.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HitlValidity {
    /// Refs with a valid, unexpired decision. PRIVATE: membership is a
    /// conclusion of the gate, not an input a caller may assert (see
    /// [`DecidedRef`]).
    decided_refs: BTreeSet<String>,
}

impl HitlValidity {
    /// The Phase-L reality: no decisions at all.
    pub fn none() -> HitlValidity {
        HitlValidity::default()
    }

    /// Build the layer-7 set from refs that WENT THROUGH the gate.
    pub fn from_decided(decided: impl IntoIterator<Item = DecidedRef>) -> HitlValidity {
        HitlValidity {
            decided_refs: decided.into_iter().map(|d| d.0).collect(),
        }
    }

    /// Whether `capability_ref` holds a valid decision.
    pub fn is_decided(&self, capability_ref: &str) -> bool {
        self.decided_refs.contains(capability_ref)
    }

    /// Whether no ref is decided (the fail-closed default).
    pub fn is_empty(&self) -> bool {
        self.decided_refs.is_empty()
    }

    /// Test-only shortcut that bypasses the gate, for the property tests and
    /// mutation controls of THIS crate. Deliberately `pub(crate)` +
    /// `#[cfg(test)]`: it must never be reachable from a host.
    #[cfg(test)]
    pub(crate) fn from_refs_for_test(refs: BTreeSet<String>) -> HitlValidity {
        HitlValidity { decided_refs: refs }
    }
}

/// The host-neutral policy seam (layers 1 and 2). The engine never reads the
/// OS or an environment variable itself; the host supplies the OS capability
/// toggle and the operator's local policy, exactly as [`crate::clock::Clock`]
/// and [`crate::signal::SignalSource`] supply time and signals.
pub trait CapabilityPolicy {
    /// Layer 1: which capability CLASSES the OS/host environment permits.
    fn os_toggle(&self) -> ClassConstraint;
    /// Layer 2: the operator's local ref allow/tighten policy.
    fn local_policy(&self) -> RefConstraint;
}

/// A policy that constrains nothing — both layers are the identity. The
/// default for a node with no OS toggle and no operator policy configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenPolicy;

impl CapabilityPolicy for OpenPolicy {
    fn os_toggle(&self) -> ClassConstraint {
        ClassConstraint::Unconstrained
    }
    fn local_policy(&self) -> RefConstraint {
        RefConstraint::Unconstrained
    }
}

/// Compute the effective capabilities: the monotone-attenuating intersection
/// of the seven layers, over the plan-declared candidate set.
///
/// `declared` is the maximal candidate universe (layer 5's closure refs with
/// their declared bounds); every other argument is a layer that can only
/// attenuate it. The result refines `declared` (proven by property test).
///
/// ## Order of application (NOT freely commutable)
///
/// The layers below are applied: plan closure → OS toggle → local policy →
/// release grants → pack grants → effect need → HITL. Layers 1-6 are
/// key-filters (closure / OS toggle / effect need) and field-meets (local
/// policy / grants); their RESULT is order-independent because meet is
/// commutative/associative and the filters only remove keys. **Layer 7 (the
/// HITL approval gate) MUST run AFTER every approval-RAISING meet** (layers 2
/// local-policy, 3 release, 4 pack) — the precise invariant, not literally
/// "last" (its order relative to the key-only filters 5/6 is immaterial, they
/// never touch `approval`). It reads `cap.approval`, a field those meets can
/// RAISE to `Human` (`BoundedCapability::meet` takes the `max`). Running the
/// HITL filter BEFORE those meets would judge a stale, lower approval
/// level and let a capability the grants then raise to `Human` pass with no
/// decision — a fail-open HITL. `hitl_gate_must_run_after_approval_raise`
/// pins this.
#[allow(clippy::too_many_arguments)]
pub fn effective_capabilities(
    declared: &EffectiveCapabilities,
    policy: &dyn CapabilityPolicy,
    release_grants: &[CapabilityGrant],
    pack_grants: &[CapabilityGrant],
    plan_closure: &BTreeSet<String>,
    effect_need: &EffectNeed,
    hitl: &HitlValidity,
    now_ms: u64,
) -> EffectiveCapabilities {
    // Layer 5 domain bound, applied ONCE and sufficient: a ref outside the
    // compiled plan's closure authorizes nothing, whatever a grant claims
    // (mirror of grant_v0's allowlist retain). A single application suffices
    // because NO later layer inserts a key — every subsequent step is a
    // `meet` (output keys ⊆ input keys) or a `filtered` (removes only) — so
    // the closure-bounded key-set can never grow back. A future layer that
    // ADDED keys would need this filter re-applied terminally.
    let mut eff = declared.filtered(|k, _| plan_closure.contains(k));

    // Layer 1 — OS capability toggle (class-scoped filter).
    if let ClassConstraint::Only(classes) = policy.os_toggle() {
        eff = eff.filtered(|_, cap| classes.contains(&cap.class));
    }

    // Layer 2 — local node policy (ref-scoped meet + filter).
    if let RefConstraint::Only(allow) = policy.local_policy() {
        eff = meet_with_allow(&eff, &allow);
    }

    // Layer 3 — grants to the app release (audience-scoped meet).
    eff = meet_with_grants(&eff, release_grants, GrantAudience::Release, now_ms);

    // Layer 4 — grants to the pack version (audience-scoped meet).
    eff = meet_with_grants(&eff, pack_grants, GrantAudience::Pack, now_ms);

    // Layer 6 — the current effect's declared need (single-ref narrowing).
    if let EffectNeed::One(needed) = effect_need {
        eff = eff.filtered(|k, _| k == needed);
    }

    // Layer 7 — a still-valid HITL decision (approval-gated filter). Empty in
    // Phase L: any Human-approval capability drops fail-closed.
    eff = eff.filtered(|k, cap| cap.approval != ApprovalLevel::Human || hitl.is_decided(k));

    eff
}

/// Meet an effective set with a local-policy allow-map: a ref absent from the
/// allow-map is dropped, a present one is meet-tightened by the policy bound.
/// Class-mismatched pairs drop (fail-closed).
fn meet_with_allow(
    eff: &EffectiveCapabilities,
    allow: &BTreeMap<String, BoundedCapability>,
) -> EffectiveCapabilities {
    let policy_set = EffectiveCapabilities::from_declared(allow.clone());
    eff.meet(&policy_set)
}

/// Meet an effective set with the LIVE contributions of a grant slice for one
/// audience. A ref granted by NONE of the live grants drops (fail-closed: a
/// grant is required, its absence is not identity). When SEVERAL live grants
/// of this audience name the SAME ref, they are reduced by [`dominating_grant`]
/// — the unique global maximum if one exists, else a fail-closed DROP (a
/// field-wise least upper bound would fabricate a capability neither grant
/// conferred; the outer meet re-bounds each coordinate but cannot restore the
/// cross-field correlations a naive LUB destroys — Codex r1/r2 P1).
fn meet_with_grants(
    eff: &EffectiveCapabilities,
    grants: &[CapabilityGrant],
    audience: GrantAudience,
    now_ms: u64,
) -> EffectiveCapabilities {
    // Collect EVERY live grant bound for this audience, keyed by ref, THEN
    // reduce each ref's bounds by a single permutation-invariant pass
    // ([`dominating_grant`]). A left-fold that combined pairwise would depend
    // on order (three grants A={a}, B={b}, C={a,b}: A,B poison in [A,B,C] but
    // C dominates in [C,A,B] — same multiset, two results, Codex r2 P1).
    let mut per_ref: BTreeMap<String, Vec<BoundedCapability>> = BTreeMap::new();
    for g in grants {
        if let Some(map) = g.live_for(audience, now_ms) {
            for (k, cap) in map {
                per_ref.entry(k.clone()).or_default().push(cap.clone());
            }
        }
    }
    let grant_set = EffectiveCapabilities::from_declared(
        per_ref
            .into_iter()
            .filter_map(|(k, bounds)| dominating_grant(&bounds).map(|cap| (k, cap)))
            .collect(),
    );
    eff.meet(&grant_set)
}

/// Reduce several live grants of the SAME audience for the SAME ref to a
/// single sound bound. Grants confer authority DISJUNCTIVELY, so the
/// combination should be the union of the authorities — but a single
/// [`BoundedCapability`] can represent that union only when ONE bound
/// DOMINATES all the others (every bound refines it), in which case the union
/// IS that dominating bound (an actual grant, never a synthesised least upper
/// bound that would authorise combinations no grant conferred — Codex r1 P1).
/// The dominating bound is the unique MAXIMUM of the refinement order and is
/// found order-independently (Codex r2 P1). If no single grant dominates all
/// the others (a genuinely incomparable set, or a class mismatch — different
/// classes never refine each other), the ref is DROPPED, fail-closed. A
/// well-formed grant set has ONE bound per ref per audience and never reaches
/// the multi-element path; an ambiguous one confers nothing for that ref (the
/// binding phase should surface that as a config error rather than rely on the
/// silent drop — see the module-level binding notes).
fn dominating_grant(bounds: &[BoundedCapability]) -> Option<BoundedCapability> {
    // A bound `m` dominates iff every bound refines it (m is the maximum).
    // `refines` is antisymmetric, so at most one distinct maximum exists.
    bounds
        .iter()
        .find(|m| bounds.iter().all(|b| b.refines(m)))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- deterministic seeded generator (0-dep; proptest is absent from
    // the workspace, so "proptest-style" means the style, not the crate —
    // the repo's precedent is seeded deterministic generation, never
    // Math.random-style non-determinism). splitmix64.

    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Rng {
            Rng(seed)
        }
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            debug_assert!(n > 0, "below(0) would divide by zero");
            self.next_u64() % n
        }
        fn class(&mut self) -> CapabilityClass {
            CapabilityClass::ALL[(self.below(17)) as usize]
        }
        fn dest(&mut self) -> DestinationBound {
            match self.below(3) {
                0 => DestinationBound::Any,
                1 => DestinationBound::Only(BTreeSet::new()),
                _ => {
                    let mut s = BTreeSet::new();
                    for _ in 0..self.below(4) {
                        s.insert(format!("d{}", self.below(5)));
                    }
                    DestinationBound::Only(s)
                }
            }
        }
        fn ceiling(&mut self) -> Ceiling {
            match self.below(2) {
                0 => Ceiling::Unbounded,
                _ => Ceiling::AtMost(self.below(100)),
            }
        }
        fn data_policy(&mut self) -> DataPolicy {
            DataPolicy {
                no_egress: self.below(2) == 0,
                no_persist: self.below(2) == 0,
            }
        }
        fn approval(&mut self) -> ApprovalLevel {
            match self.below(3) {
                0 => ApprovalLevel::Auto,
                1 => ApprovalLevel::Notify,
                _ => ApprovalLevel::Human,
            }
        }
        fn cap(&mut self) -> BoundedCapability {
            BoundedCapability {
                class: self.class(),
                destinations: self.dest(),
                quota: self.ceiling(),
                timeout_ms: self.ceiling(),
                data_policy: self.data_policy(),
                approval: self.approval(),
            }
        }
        fn cap_of(&mut self, class: CapabilityClass) -> BoundedCapability {
            let mut c = self.cap();
            c.class = class;
            c
        }
        fn effset(&mut self) -> EffectiveCapabilities {
            let mut m = BTreeMap::new();
            for _ in 0..self.below(6) {
                let r = format!("r{}", self.below(8));
                let class = self.class();
                m.insert(r, self.cap_of(class));
            }
            EffectiveCapabilities::from_declared(m)
        }
    }

    fn all_refs() -> BTreeSet<String> {
        (0..8).map(|i| format!("r{i}")).collect()
    }

    fn s(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|x| (*x).to_string()).collect()
    }

    // -------------------------------------------------------------------
    // seventeen_classes_typed — the gate's headline property.
    // -------------------------------------------------------------------

    /// Exactly 17 classes, all distinct, each a typed enum variant — and the
    /// authorization mechanism is TYPED, not a substring: `commit`/`push` are
    /// legitimately two of the classes, so a "module contains no sensitive
    /// substring" test would be unsatisfiable and wrong. The real guarantee
    /// is that this crate does not depend on `sbfb-factory` /
    /// `operator_server::SENSITIVE_ACTIONS` (structural: not in Cargo.toml)
    /// and that `effective_capabilities` keys on `CapabilityClass`, never on
    /// a method-name string.
    #[test]
    fn seventeen_classes_typed() {
        assert_eq!(CapabilityClass::ALL.len(), 17);
        let distinct: BTreeSet<CapabilityClass> = CapabilityClass::ALL.iter().copied().collect();
        assert_eq!(distinct.len(), 17, "all 17 classes must be distinct");
        // Commit and Push ARE two of the seventeen — the point of typing.
        assert!(distinct.contains(&CapabilityClass::Commit));
        assert!(distinct.contains(&CapabilityClass::Push));
        // Partial drift guard (Codex r1 P2, honest bound r3): this
        // WILDCARD-FREE match forces a COMPILE ERROR if a variant is added
        // without extending the match — a reviewer prompt to reconsider `ALL`.
        // It does NOT hard-guarantee `ALL` stays in sync: adding a variant to
        // the enum AND the match but forgetting `ALL` still compiles and the
        // loop (over `ALL`) stays green. Closing that fully needs a source-
        // generating macro or `variant_count` (unstable); until then the match
        // + the distinctness/len assertions above are the guard.
        for c in CapabilityClass::ALL {
            let named = match c {
                CapabilityClass::ReadPublic
                | CapabilityClass::ReadPrivateScoped
                | CapabilityClass::WriteDraft
                | CapabilityClass::WriteWorkspace
                | CapabilityClass::NetworkFixedDestination
                | CapabilityClass::NetworkOpen
                | CapabilityClass::RemoteCompute
                | CapabilityClass::ModelLocal
                | CapabilityClass::ModelCloud
                | CapabilityClass::ToolNative
                | CapabilityClass::ToolMcp
                | CapabilityClass::SecretUse
                | CapabilityClass::Publish
                | CapabilityClass::Install
                | CapabilityClass::Commit
                | CapabilityClass::Push
                | CapabilityClass::Destructive => true,
            };
            assert!(named && CapabilityClass::ALL.contains(&c));
        }
    }

    // -------------------------------------------------------------------
    // intersection_only_attenuates — the cardinal property.
    // -------------------------------------------------------------------

    /// Every single layer, over 2000 seeded random inputs, returns a set that
    /// REFINES its input (⊑). Adding a layer can only attenuate.
    ///
    /// `refines` is `all(...)`, vacuously true on the empty set — so the
    /// property alone could pass while never actually exercising a survivor.
    /// Witness counters assert the seed genuinely produced discriminating
    /// inputs: a non-empty result after all layers, and a survivor STRICTLY
    /// tightened away from its declared bound (else no attenuation was tested).
    #[test]
    fn intersection_only_attenuates() {
        let mut rng = Rng::new(0x5BFB_0007 ^ 0x00C0_FFEE);
        let mut saw_nonempty_after_all_layers = 0u32;
        let mut saw_survivor_strictly_tightened = 0u32;
        for _ in 0..2000 {
            // At least one declared ref so a survivor is POSSIBLE (an empty
            // declared universe would make the whole property vacuous — the
            // witness assertions below would then legitimately fail).
            let declared = {
                let mut d = rng.effset();
                if d.is_empty() {
                    let cl = rng.class();
                    d = EffectiveCapabilities::from_declared(
                        [("r0".to_string(), rng.cap_of(cl))].into_iter().collect(),
                    );
                }
                d
            };
            // Closure keeps each declared ref with prob 3/4 (else the grants
            // could never let anything through — layer 5 is fail-closed), plus
            // an occasional extra ref to exercise the domain-drop path.
            let declared_refs: Vec<String> = declared.refs().cloned().collect();
            let mut closure: BTreeSet<String> = BTreeSet::new();
            for r in &declared_refs {
                if rng.below(4) != 0 {
                    closure.insert(r.clone());
                }
            }
            for r in all_refs() {
                if rng.below(4) == 0 {
                    closure.insert(r);
                }
            }
            let os = match rng.below(2) {
                0 => ClassConstraint::Unconstrained,
                _ => {
                    let mut set = BTreeSet::new();
                    for _ in 0..rng.below(18) {
                        set.insert(rng.class());
                    }
                    ClassConstraint::Only(set)
                }
            };
            let local = match rng.below(2) {
                0 => RefConstraint::Unconstrained,
                _ => {
                    let mut m = BTreeMap::new();
                    for _ in 0..rng.below(6) {
                        let r = format!("r{}", rng.below(8));
                        let class = rng.class();
                        m.insert(r, rng.cap_of(class));
                    }
                    RefConstraint::Only(m)
                }
            };
            struct P(ClassConstraint, RefConstraint);
            impl CapabilityPolicy for P {
                fn os_toggle(&self) -> ClassConstraint {
                    self.0.clone()
                }
                fn local_policy(&self) -> RefConstraint {
                    self.1.clone()
                }
            }
            let policy = P(os, local);
            // Grants that COVER the declared refs — same CLASS (so the meet
            // never class-drops), random field BOUNDS (so field-meet
            // attenuation is exercised), live — each ref granted in an audience
            // with prob 3/4. A ref granted in BOTH audiences can survive; the
            // 1/4 misses exercise the fail-closed grant-drop path. Expiry /
            // revocation have their own dedicated tests; keeping grants live
            // here is what makes survivors frequent enough to be non-vacuous.
            let mk_grant = |rng: &mut Rng, audience: GrantAudience| {
                let mut m = BTreeMap::new();
                for r in declared.refs() {
                    if rng.below(4) != 0 {
                        let class = declared.get(r).unwrap().class;
                        m.insert(r.clone(), rng.cap_of(class));
                    }
                }
                vec![CapabilityGrant {
                    audience,
                    granted: m,
                    expires_at_ms: None,
                    revoked: false,
                }]
            };
            let release = mk_grant(&mut rng, GrantAudience::Release);
            let pack = mk_grant(&mut rng, GrantAudience::Pack);
            let need = match rng.below(2) {
                0 => EffectNeed::Any,
                _ => EffectNeed::One(format!("r{}", rng.below(8))),
            };
            let hitl = HitlValidity::from_refs_for_test(
                all_refs()
                    .into_iter()
                    .filter(|_| rng.below(2) == 0)
                    .collect(),
            );
            let now = rng.below(2000);

            // The domain-bounded base every layer attenuates from.
            let base = declared.filtered(|k, _| closure.contains(k));
            let eff = effective_capabilities(
                &declared, &policy, &release, &pack, &closure, &need, &hitl, now,
            );
            assert!(
                eff.refines(&base),
                "effective must refine the plan-closed declared base"
            );
            // And it refines the original declared set too (closure only removed).
            assert!(eff.refines(&declared));

            // Witnesses (non-vacuity): a survivor exists, and at least once a
            // survivor is STRICTLY tighter than its declared bound.
            if !eff.is_empty() {
                saw_nonempty_after_all_layers += 1;
                for r in eff.refs() {
                    let d = declared.get(r).unwrap();
                    let e = eff.get(r).unwrap();
                    if e != d && e.refines(d) {
                        saw_survivor_strictly_tightened += 1;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_nonempty_after_all_layers > 0,
            "vacuous: no input ever produced a survivor through all 7 layers"
        );
        assert!(
            saw_survivor_strictly_tightened > 0,
            "vacuous: no survivor was ever strictly attenuated below its declared bound"
        );
    }

    // -------------------------------------------------------------------
    // Algebraic laws of the meet.
    // -------------------------------------------------------------------

    #[test]
    fn meet_laws_commutative_associative_idempotent_absorbing() {
        let mut rng = Rng::new(0xA11CE);
        for _ in 0..2000 {
            let a = rng.effset();
            let b = rng.effset();
            let c = rng.effset();
            // commutative
            assert_eq!(a.meet(&b), b.meet(&a));
            // associative
            assert_eq!(a.meet(&b).meet(&c), a.meet(&b.meet(&c)));
            // idempotent
            assert_eq!(a.meet(&a), a);
            // absorbing: meeting with empty yields empty
            assert_eq!(
                a.meet(&EffectiveCapabilities::empty()),
                EffectiveCapabilities::empty()
            );
            // meet refines both operands
            let m = a.meet(&b);
            assert!(m.refines(&a));
            assert!(m.refines(&b));
        }
    }

    /// With every ATTENUATOR layer at its identity — `OpenPolicy` (layers 1
    /// and 2 unconstrained), closure = the whole universe, `EffectNeed::Any`
    /// — and full top-bound grants confirming every declared ref in BOTH
    /// audiences (so the positive grant layers do not attenuate either), the
    /// result equals the declared set MINUS its Human-approval refs (layer 7
    /// empty is not identity). This isolates the attenuator identities from
    /// the mandatory grant layers.
    #[test]
    fn absent_layers_are_identity() {
        let mut rng = Rng::new(0x1DE7);
        for _ in 0..1000 {
            let declared = rng.effset();
            let closure = all_refs(); // closure = universe → no closure drop
            // Grants conferring each declared ref at TOP bounds (top ⊓ x = x,
            // so grants are identity in the meet) in both audiences.
            let top_grant = |audience: GrantAudience| CapabilityGrant {
                audience,
                granted: declared
                    .refs()
                    .map(|r| {
                        (
                            r.clone(),
                            BoundedCapability::top(declared.get(r).unwrap().class),
                        )
                    })
                    .collect(),
                expires_at_ms: None,
                revoked: false,
            };
            let eff = effective_capabilities(
                &declared,
                &OpenPolicy,
                &[top_grant(GrantAudience::Release)],
                &[top_grant(GrantAudience::Pack)],
                &closure,
                &EffectNeed::Any,
                &HitlValidity::none(),
                0,
            );
            let expected = declared.filtered(|_, cap| cap.approval != ApprovalLevel::Human);
            assert_eq!(eff, expected);
        }
    }

    // -------------------------------------------------------------------
    // expiry_enforced (mirror grant_v0 conventions, PA-1).
    // -------------------------------------------------------------------

    #[test]
    fn expiry_enforced() {
        let cap = BoundedCapability::top(CapabilityClass::ModelCloud);
        let declared = EffectiveCapabilities::from_declared(
            [("m".to_string(), cap.clone())].into_iter().collect(),
        );
        let closure = s(&["m"]);
        let grant = |audience: GrantAudience, exp: Option<u64>| CapabilityGrant {
            audience,
            granted: [("m".to_string(), cap.clone())].into_iter().collect(),
            expires_at_ms: exp,
            revoked: false,
        };
        // A never-expiring pack grant so the pack layer does not itself deny —
        // this test isolates expiry on the release layer.
        let pack = [grant(GrantAudience::Pack, None)];
        let run = |g: &[CapabilityGrant], now: u64| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                g,
                &pack,
                &closure,
                &EffectNeed::Any,
                &HitlValidity::none(),
                now,
            )
        };
        let rel = |exp: Option<u64>| [grant(GrantAudience::Release, exp)];
        // Before expiry: granted.
        assert_eq!(run(&rel(Some(1000)), 999).len(), 1);
        // AT the bound: refused (>=, mirroring grant_v0).
        assert_eq!(run(&rel(Some(1000)), 1000).len(), 0);
        // Past: refused.
        assert_eq!(run(&rel(Some(1000)), 5000).len(), 0);
        // Never-expiring: granted at any time.
        assert_eq!(run(&rel(None), u64::MAX).len(), 1);
    }

    /// Revocation is honoured on the very next computation (never cached).
    #[test]
    fn revocation_per_call() {
        let cap = BoundedCapability::top(CapabilityClass::Publish);
        let declared = EffectiveCapabilities::from_declared(
            [("p".to_string(), cap.clone())].into_iter().collect(),
        );
        let closure = s(&["p"]);
        let mk = |revoked: bool| {
            vec![CapabilityGrant {
                audience: GrantAudience::Release,
                granted: [("p".to_string(), cap.clone())].into_iter().collect(),
                expires_at_ms: None,
                revoked,
            }]
        };
        let pack = vec![CapabilityGrant {
            audience: GrantAudience::Pack,
            granted: [("p".to_string(), cap.clone())].into_iter().collect(),
            expires_at_ms: None,
            revoked: false,
        }];
        let run = |g: &[CapabilityGrant]| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                g,
                &pack,
                &closure,
                &EffectNeed::Any,
                &HitlValidity::none(),
                0,
            )
            .len()
        };
        assert_eq!(run(&mk(false)), 1);
        assert_eq!(run(&mk(true)), 0); // revoked → refused, no cache
    }

    /// Expiry and revocation are honoured symmetrically in the PACK audience
    /// too — a mutation that skips them only for pack grants is caught (Codex
    /// r3 P2; `expiry_enforced`/`revocation_per_call` exercised release only).
    #[test]
    fn pack_audience_also_enforces_expiry_and_revocation() {
        let cap = BoundedCapability::top(CapabilityClass::Install);
        let declared = EffectiveCapabilities::from_declared(
            [("p".to_string(), cap.clone())].into_iter().collect(),
        );
        let closure = s(&["p"]);
        let release = vec![CapabilityGrant {
            audience: GrantAudience::Release,
            granted: [("p".to_string(), cap.clone())].into_iter().collect(),
            expires_at_ms: None,
            revoked: false,
        }];
        let pack = |expires: Option<u64>, revoked: bool| {
            vec![CapabilityGrant {
                audience: GrantAudience::Pack,
                granted: [("p".to_string(), cap.clone())].into_iter().collect(),
                expires_at_ms: expires,
                revoked,
            }]
        };
        let run = |pk: Vec<CapabilityGrant>, now: u64| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                &release,
                &pk,
                &closure,
                &EffectNeed::Any,
                &HitlValidity::none(),
                now,
            )
            .len()
        };
        assert_eq!(run(pack(None, false), 0), 1); // live pack grant → authorized
        assert_eq!(run(pack(Some(1000), false), 999), 1); // before expiry
        assert_eq!(run(pack(Some(1000), false), 1000), 0); // AT the bound → refused
        assert_eq!(run(pack(None, true), 0), 0); // revoked pack grant → refused
    }

    // -------------------------------------------------------------------
    // audience_enforced (T-CAP-GRANT-AUDIENCE-CONFUSION).
    // -------------------------------------------------------------------

    /// A release-audience grant does NOT satisfy the pack layer, and a
    /// pack-audience grant does NOT satisfy the release layer — a ref needs a
    /// live grant in BOTH audiences to survive.
    #[test]
    fn audience_enforced() {
        let cap = BoundedCapability::top(CapabilityClass::ToolMcp);
        let declared = EffectiveCapabilities::from_declared(
            [("t".to_string(), cap.clone())].into_iter().collect(),
        );
        let closure = s(&["t"]);
        let release = CapabilityGrant {
            audience: GrantAudience::Release,
            granted: [("t".to_string(), cap.clone())].into_iter().collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let pack = CapabilityGrant {
            audience: GrantAudience::Pack,
            granted: [("t".to_string(), cap.clone())].into_iter().collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let run = |rel: &[CapabilityGrant], pk: &[CapabilityGrant]| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                rel,
                pk,
                &closure,
                &EffectNeed::Any,
                &HitlValidity::none(),
                0,
            )
            .len()
        };
        use std::slice::from_ref;
        // Both audiences present → authorized.
        assert_eq!(run(from_ref(&release), from_ref(&pack)), 1);
        // Release grant offered to BOTH slots: the pack layer sees a
        // release-audience grant, treats it as inert → pack layer empty →
        // ref drops.
        assert_eq!(run(from_ref(&release), from_ref(&release)), 0);
        // SYMMETRIC: a pack-audience grant in the RELEASE slot is inert too →
        // release layer empty → drops (Codex r1: the pack→release direction).
        assert_eq!(run(from_ref(&pack), from_ref(&pack)), 0);
        // Pack grant only, no release grant → release layer empty → drops.
        assert_eq!(run(&[], from_ref(&pack)), 0);
        // Release grant only, no pack grant → pack layer empty → drops.
        assert_eq!(run(from_ref(&release), &[]), 0);
    }

    // -------------------------------------------------------------------
    // depth_enforced (PA-3: monotone chain, NOT delegation depth).
    // -------------------------------------------------------------------

    /// "Depth" in scope for Phase L is the length of the LAYER chain: adding
    /// one more layer to the fold can only attenuate. Delegation depth
    /// (research §14.5 `DelegationGrant.v1`) is post-MVP and NOT modeled here.
    /// This proves the meet-fold is monotone in the number of layers.
    #[test]
    fn depth_enforced() {
        let mut rng = Rng::new(0xDEB7);
        for _ in 0..1000 {
            // Build a chain of N random effect-sets; folding one more can only
            // shrink relative to folding fewer.
            let sets: Vec<EffectiveCapabilities> =
                (0..1 + rng.below(6)).map(|_| rng.effset()).collect();
            let mut acc = sets[0].clone();
            for next in &sets[1..] {
                let deeper = acc.meet(next);
                assert!(deeper.refines(&acc), "one more layer must not widen");
                acc = deeper;
            }
        }
    }

    // -------------------------------------------------------------------
    // undeclared_capability_refused (PA-6: WA-T5.2 leg 2, RUNTIME).
    // -------------------------------------------------------------------

    /// A capability ref that is NOT in the compiled plan's closure is refused
    /// at the intersection even if a grant names it — the domain bound holds
    /// on both sides (mirror of grant_v0's allowlist retain). This is the
    /// runtime half of WA-T5.2 (the compiler's static refusal is Phase H).
    #[test]
    fn undeclared_capability_refused() {
        let cap = BoundedCapability::top(CapabilityClass::Commit);
        // BOTH refs are in the declared base AND fully granted in both
        // audiences — so the ONLY thing that can drop "smuggled" is the
        // compiled-plan closure (layer 5). Putting "smuggled" only in a grant
        // would let the declared-base key intersection block it for the wrong
        // reason (a vacuous pass that a closure-bypass mutation survives).
        let declared = EffectiveCapabilities::from_declared(
            [
                ("known".to_string(), cap.clone()),
                ("smuggled".to_string(), cap.clone()),
            ]
            .into_iter()
            .collect(),
        );
        // Closure closes only "known"; "smuggled" is NOT in the plan closure.
        let closure = s(&["known"]);
        let grant = |audience| CapabilityGrant {
            audience,
            granted: [
                ("known".to_string(), cap.clone()),
                ("smuggled".to_string(), cap.clone()),
            ]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let eff = effective_capabilities(
            &declared,
            &OpenPolicy,
            &[grant(GrantAudience::Release)],
            &[grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::none(),
            0,
        );
        assert!(eff.get("known").is_some(), "closed + granted ref survives");
        assert!(
            eff.get("smuggled").is_none(),
            "a declared, fully-granted ref OUTSIDE the plan closure is still refused"
        );
        // (The layer-6 EffectNeed narrowing is proven non-vacuously by
        // `effect_need_narrows_to_single_surviving_ref`; narrowing to a ref
        // the closure ALREADY removed would pass whatever layer 6 does.)
    }

    // -------------------------------------------------------------------
    // HITL double-semantics (PA-2): empty layer 7 is not identity.
    // -------------------------------------------------------------------

    /// A Human-approval capability drops when no decision exists (Phase L
    /// reality), passes when a decision is present; Auto/Notify pass either
    /// way. This is the "absence of a layer ≠ absence within a layer"
    /// distinction that keeps an empty HITL set fail-closed.
    #[test]
    fn human_approval_requires_valid_decision() {
        let mk = |approval: ApprovalLevel| {
            let mut c = BoundedCapability::top(CapabilityClass::Push);
            c.approval = approval;
            EffectiveCapabilities::from_declared([("x".to_string(), c)].into_iter().collect())
        };
        let closure = s(&["x"]);
        // A neutral (top-bound, Auto-approval) grant in both audiences so the
        // grant layer does not itself RAISE the approval — the declared cap's
        // own approval level flows through and layer 7 judges THAT.
        let grant = |audience| CapabilityGrant {
            audience,
            granted: [(
                "x".to_string(),
                BoundedCapability::top(CapabilityClass::Push),
            )]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let run = |declared: &EffectiveCapabilities, hitl: &HitlValidity| {
            effective_capabilities(
                declared,
                &OpenPolicy,
                &[grant(GrantAudience::Release)],
                &[grant(GrantAudience::Pack)],
                &closure,
                &EffectNeed::Any,
                hitl,
                0,
            )
            .len()
        };
        // Human approval, no decision → dropped (empty layer 7 ≠ identity).
        assert_eq!(run(&mk(ApprovalLevel::Human), &HitlValidity::none()), 0);
        // Human approval, decision present FOR THIS ref → passes.
        assert_eq!(
            run(
                &mk(ApprovalLevel::Human),
                &HitlValidity::from_refs_for_test(s(&["x"]))
            ),
            1
        );
        // Human approval, decision present but for ANOTHER ref → still drops
        // (the gate is per-ref, not "any decision exists" — Codex r1).
        assert_eq!(
            run(
                &mk(ApprovalLevel::Human),
                &HitlValidity::from_refs_for_test(s(&["other"]))
            ),
            0
        );
        // Auto approval → passes with no decision.
        assert_eq!(run(&mk(ApprovalLevel::Auto), &HitlValidity::none()), 1);
        // Notify (non-blocking) → passes with no decision.
        assert_eq!(run(&mk(ApprovalLevel::Notify), &HitlValidity::none()), 1);
    }

    /// The `ApprovalLevel` order is `Auto < Notify < Human` and the meet takes
    /// the max — pinned so a mutation reordering the enum (e.g. `Notify` after
    /// `Human`) is caught (Codex r1). And `DataPolicy::refines` is a genuine
    /// flag-wise `>=`, not a constant `true`.
    #[test]
    fn ordering_invariants_pinned() {
        // Approval order and max-meet.
        assert!(ApprovalLevel::Auto < ApprovalLevel::Notify);
        assert!(ApprovalLevel::Notify < ApprovalLevel::Human);
        assert_eq!(
            ApprovalLevel::Notify.max(ApprovalLevel::Human),
            ApprovalLevel::Human
        );
        assert_eq!(
            ApprovalLevel::Auto.max(ApprovalLevel::Notify),
            ApprovalLevel::Notify
        );
        // DataPolicy refines is real: stricter refines looser, not vice versa.
        let none = DataPolicy::UNRESTRICTED;
        let egress = DataPolicy {
            no_egress: true,
            no_persist: false,
        };
        let both = DataPolicy {
            no_egress: true,
            no_persist: true,
        };
        assert!(both.refines(egress) && egress.refines(none));
        assert!(!none.refines(egress), "looser must not refine stricter");
        assert!(!egress.refines(both));
        // Orthogonal flags are incomparable: neither refines the other.
        let persist = DataPolicy {
            no_egress: false,
            no_persist: true,
        };
        assert!(!egress.refines(persist) && !persist.refines(egress));
    }

    /// The approval field meets by MAX, not min: a layer that raises a
    /// capability to `Human` makes it need a decision — a `min` bug here would
    /// silently widen (the class-set-only test would not catch it). This pins
    /// the per-field direction (PA-5).
    #[test]
    fn approval_meets_by_max_not_min() {
        let auto = {
            let mut c = BoundedCapability::top(CapabilityClass::Destructive);
            c.approval = ApprovalLevel::Auto;
            c
        };
        let human = {
            let mut c = BoundedCapability::top(CapabilityClass::Destructive);
            c.approval = ApprovalLevel::Human;
            c
        };
        let m = auto.meet(&human).unwrap();
        assert_eq!(
            m.approval,
            ApprovalLevel::Human,
            "approval must take the MAX"
        );
        // And the tighter ceilings still take min/intersection.
        let mut lo = BoundedCapability::top(CapabilityClass::Destructive);
        lo.quota = Ceiling::AtMost(3);
        lo.timeout_ms = Ceiling::AtMost(10);
        lo.destinations = DestinationBound::Only(s(&["a", "b"]));
        // Orthogonal data-policy axes: `lo` forbids persistence only.
        lo.data_policy = DataPolicy {
            no_egress: false,
            no_persist: true,
        };
        let mut hi = BoundedCapability::top(CapabilityClass::Destructive);
        hi.quota = Ceiling::AtMost(9);
        hi.timeout_ms = Ceiling::AtMost(4);
        hi.destinations = DestinationBound::Only(s(&["b", "c"]));
        // `hi` forbids egress only.
        hi.data_policy = DataPolicy {
            no_egress: true,
            no_persist: false,
        };
        let m = lo.meet(&hi).unwrap();
        assert_eq!(m.quota, Ceiling::AtMost(3));
        assert_eq!(m.timeout_ms, Ceiling::AtMost(4));
        assert_eq!(m.destinations, DestinationBound::Only(s(&["b"])));
        // The UNION keeps BOTH constraints — a linear `max` would have kept
        // only one, silently dropping the other axis (F3).
        assert_eq!(
            m.data_policy,
            DataPolicy {
                no_egress: true,
                no_persist: true
            }
        );
    }

    /// Class mismatch on the same ref fails closed (drops), never coerces.
    #[test]
    fn class_mismatch_drops_fail_closed() {
        let a = EffectiveCapabilities::from_declared(
            [(
                "r".to_string(),
                BoundedCapability::top(CapabilityClass::ReadPublic),
            )]
            .into_iter()
            .collect(),
        );
        let b = EffectiveCapabilities::from_declared(
            [(
                "r".to_string(),
                BoundedCapability::top(CapabilityClass::Destructive),
            )]
            .into_iter()
            .collect(),
        );
        assert!(
            a.meet(&b).is_empty(),
            "incompatible classes intersect to nothing"
        );
    }

    /// `refines` is a REAL partial order at every level, not a constant `true`:
    /// negative cases pin each axis (Codex r2 P2 — a `refines → true` mutation
    /// otherwise survives). Fewer keys / tighter bounds refine; extra keys /
    /// looser bounds do NOT.
    #[test]
    fn refines_negative_cases_pinned() {
        // Ceiling: tighter refines looser, not vice versa.
        assert!(Ceiling::AtMost(3).refines(Ceiling::AtMost(5)));
        assert!(!Ceiling::AtMost(5).refines(Ceiling::AtMost(3)));
        assert!(Ceiling::AtMost(3).refines(Ceiling::Unbounded));
        assert!(!Ceiling::Unbounded.refines(Ceiling::AtMost(3)));
        // DestinationBound: subset refines superset, not vice versa.
        let a = DestinationBound::Only(s(&["a"]));
        let ab = DestinationBound::Only(s(&["a", "b"]));
        assert!(a.refines(&ab) && !ab.refines(&a));
        assert!(a.refines(&DestinationBound::Any) && !DestinationBound::Any.refines(&a));
        // EffectiveCapabilities: a set with an EXTRA key does not refine a
        // subset; a looser bound on a shared key does not refine a tighter one.
        let tight = {
            let mut c = BoundedCapability::top(CapabilityClass::ReadPublic);
            c.quota = Ceiling::AtMost(1);
            c
        };
        let one = EffectiveCapabilities::from_declared(
            [("x".to_string(), tight.clone())].into_iter().collect(),
        );
        let two = EffectiveCapabilities::from_declared(
            [
                ("x".to_string(), tight.clone()),
                (
                    "y".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
            ]
            .into_iter()
            .collect(),
        );
        assert!(one.refines(&two), "fewer keys refine more keys");
        assert!(!two.refines(&one), "an extra key must NOT refine a subset");
        let loose = EffectiveCapabilities::from_declared(
            [(
                "x".to_string(),
                BoundedCapability::top(CapabilityClass::ReadPublic),
            )]
            .into_iter()
            .collect(),
        );
        assert!(one.refines(&loose), "a tighter bound refines a looser one");
        assert!(
            !loose.refines(&one),
            "a looser bound must NOT refine a tighter one"
        );

        // `BoundedCapability::refines` checks EVERY axis: for each field, a
        // strict cap refines a looser one, and the looser one does NOT refine
        // the strict — so a mutation dropping any single axis check is caught
        // (Codex r3 P2). Base = top() of one class; vary exactly one axis.
        let base = || BoundedCapability::top(CapabilityClass::NetworkOpen);
        let strict_loose = |strict: BoundedCapability, loose: BoundedCapability, axis: &str| {
            assert!(strict.refines(&loose), "strict must refine loose on {axis}");
            assert!(
                !loose.refines(&strict),
                "loose must NOT refine strict on {axis}"
            );
        };
        // class: a class mismatch never refines (both directions false).
        let other_class = BoundedCapability::top(CapabilityClass::ReadPublic);
        assert!(
            !base().refines(&other_class),
            "class mismatch never refines"
        );
        assert!(!other_class.refines(&base()));
        // quota
        let mut q = base();
        q.quota = Ceiling::AtMost(1);
        strict_loose(q, base(), "quota");
        // timeout
        let mut t = base();
        t.timeout_ms = Ceiling::AtMost(1);
        strict_loose(t, base(), "timeout");
        // destinations
        let mut d = base();
        d.destinations = DestinationBound::Only(s(&["a"]));
        strict_loose(d, base(), "destinations");
        // data_policy
        let mut dp = base();
        dp.data_policy = DataPolicy {
            no_egress: true,
            no_persist: false,
        };
        strict_loose(dp, base(), "data_policy");
        // approval
        let mut ap = base();
        ap.approval = ApprovalLevel::Human;
        strict_loose(ap, base(), "approval");
    }

    /// The OS class toggle drops refs whose class is not permitted.
    #[test]
    fn os_toggle_class_filter() {
        struct OnlyRead;
        impl CapabilityPolicy for OnlyRead {
            fn os_toggle(&self) -> ClassConstraint {
                ClassConstraint::Only(s_classes(&[CapabilityClass::ReadPublic]))
            }
            fn local_policy(&self) -> RefConstraint {
                RefConstraint::Unconstrained
            }
        }
        fn s_classes(c: &[CapabilityClass]) -> BTreeSet<CapabilityClass> {
            c.iter().copied().collect()
        }
        let declared = EffectiveCapabilities::from_declared(
            [
                (
                    "ok".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
                (
                    "no".to_string(),
                    BoundedCapability::top(CapabilityClass::Push),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let closure = s(&["ok", "no"]);
        let grant = |audience| CapabilityGrant {
            audience,
            granted: [
                (
                    "ok".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
                (
                    "no".to_string(),
                    BoundedCapability::top(CapabilityClass::Push),
                ),
            ]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let eff = effective_capabilities(
            &declared,
            &OnlyRead,
            &[grant(GrantAudience::Release)],
            &[grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::none(),
            0,
        );
        assert!(eff.get("ok").is_some());
        assert!(
            eff.get("no").is_none(),
            "OS toggle removes the non-permitted class"
        );
    }

    /// The HITL gate (layer 7) MUST run AFTER EVERY approval-raising meet
    /// (layers 2 local-policy, 3 release, 4 pack). Each case raises the
    /// approval to `Human` via EXACTLY ONE layer, the others staying `Auto`,
    /// so a mutation moving the gate before THAT layer (even between release
    /// and pack) is caught — not merely "before the first raiser" (Codex r1
    /// P2). Declared approval is `Auto`; with no decision the raised
    /// capability must drop.
    #[test]
    fn hitl_gate_must_run_after_approval_raise() {
        let auto_declared = || {
            EffectiveCapabilities::from_declared(
                [(
                    "x".to_string(),
                    BoundedCapability::top(CapabilityClass::Push), // approval Auto
                )]
                .into_iter()
                .collect(),
            )
        };
        let closure = s(&["x"]);
        let human_cap = || {
            let mut c = BoundedCapability::top(CapabilityClass::Push);
            c.approval = ApprovalLevel::Human;
            c
        };
        let auto_grant = |audience| CapabilityGrant {
            audience,
            granted: [(
                "x".to_string(),
                BoundedCapability::top(CapabilityClass::Push),
            )]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let human_grant = |audience| CapabilityGrant {
            audience,
            granted: [("x".to_string(), human_cap())].into_iter().collect(),
            expires_at_ms: None,
            revoked: false,
        };
        // Policy that raises "x" to Human via layer 2 only.
        struct RaiseLocal(BoundedCapability);
        impl CapabilityPolicy for RaiseLocal {
            fn os_toggle(&self) -> ClassConstraint {
                ClassConstraint::Unconstrained
            }
            fn local_policy(&self) -> RefConstraint {
                RefConstraint::Only([("x".to_string(), self.0.clone())].into_iter().collect())
            }
        }

        // Case A — raised ONLY by the local policy (layer 2); grants Auto.
        let eff = effective_capabilities(
            &auto_declared(),
            &RaiseLocal(human_cap()),
            &[auto_grant(GrantAudience::Release)],
            &[auto_grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::none(),
            0,
        );
        assert!(
            eff.is_empty(),
            "layer-2-raised Human must drop with no decision"
        );

        // Case B — raised ONLY by the release grant (layer 3); pack Auto.
        let eff = effective_capabilities(
            &auto_declared(),
            &OpenPolicy,
            &[human_grant(GrantAudience::Release)],
            &[auto_grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::none(),
            0,
        );
        assert!(
            eff.is_empty(),
            "release-raised Human must drop with no decision"
        );

        // Case C — raised ONLY by the pack grant (layer 4); release Auto. This
        // is the case a gate placed BETWEEN release and pack would miss.
        let eff = effective_capabilities(
            &auto_declared(),
            &OpenPolicy,
            &[auto_grant(GrantAudience::Release)],
            &[human_grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::none(),
            0,
        );
        assert!(
            eff.is_empty(),
            "pack-raised Human must drop with no decision"
        );

        // And with a decision, the pack-raised case passes as Human.
        let eff = effective_capabilities(
            &auto_declared(),
            &OpenPolicy,
            &[auto_grant(GrantAudience::Release)],
            &[human_grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::from_refs_for_test(s(&["x"])),
            0,
        );
        assert_eq!(eff.get("x").map(|c| c.approval), Some(ApprovalLevel::Human));
    }

    /// Several live grants of the SAME audience for the SAME ref combine
    /// order-independently: COMPARABLE bounds collapse to the more permissive,
    /// INCOMPARABLE bounds poison the ref to a fail-closed drop — never a
    /// field-wise LUB that fabricates a capability neither grant conferred
    /// (Codex r1 P1, T-CAP-WIDEN-BYPASS).
    #[test]
    fn multi_grant_same_audience_combines_soundly() {
        let cap = |dest: &[&str], quota: u64, approval: ApprovalLevel| {
            let mut c = BoundedCapability::top(CapabilityClass::NetworkFixedDestination);
            c.destinations = DestinationBound::Only(s(dest));
            c.quota = Ceiling::AtMost(quota);
            c.approval = approval;
            c
        };
        let declared = EffectiveCapabilities::from_declared(
            [(
                "r".to_string(),
                BoundedCapability::top(CapabilityClass::NetworkFixedDestination),
            )]
            .into_iter()
            .collect(),
        );
        let closure = s(&["r"]);
        let release_grants = |bounds: &[BoundedCapability]| {
            bounds
                .iter()
                .map(|b| CapabilityGrant {
                    audience: GrantAudience::Release,
                    granted: [("r".to_string(), b.clone())].into_iter().collect(),
                    expires_at_ms: None,
                    revoked: false,
                })
                .collect::<Vec<_>>()
        };
        let pack = vec![CapabilityGrant {
            audience: GrantAudience::Pack,
            granted: [(
                "r".to_string(),
                BoundedCapability::top(CapabilityClass::NetworkFixedDestination),
            )]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        }];
        let run = |bounds: &[BoundedCapability]| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                &release_grants(bounds),
                &pack,
                &closure,
                &EffectNeed::Any,
                &HitlValidity::from_refs_for_test(s(&["r"])),
                0,
            )
        };
        // All 6 permutations of a 3-element slice.
        let perms = |x: &BoundedCapability, y: &BoundedCapability, z: &BoundedCapability| {
            [
                vec![x.clone(), y.clone(), z.clone()],
                vec![x.clone(), z.clone(), y.clone()],
                vec![y.clone(), x.clone(), z.clone()],
                vec![y.clone(), z.clone(), x.clone()],
                vec![z.clone(), x.clone(), y.clone()],
                vec![z.clone(), y.clone(), x.clone()],
            ]
        };

        // INCOMPARABLE pair (the Codex r1 counter-example): {a}/100/Human vs
        // {b}/1/Auto. A LUB would yield {a,b}/100/Auto — authorizing `a` with
        // 100 calls and NO approval, which neither grant allowed. No grant
        // dominates → DROP. Order-independent.
        let inca = cap(&["a"], 100, ApprovalLevel::Human);
        let incb = cap(&["b"], 1, ApprovalLevel::Auto);
        assert!(run(&[inca.clone(), incb.clone()]).is_empty());
        assert!(run(&[incb.clone(), inca.clone()]).is_empty());

        // THREE grants where two are pairwise incomparable but BOTH refine the
        // third (Codex r2 counter-example): A={a}, B={b}, C={a,b} (same
        // quota/approval). C DOMINATES both → C survives, in EVERY permutation
        // (a left-fold would poison A,B first and drop C in some orders).
        let ga = cap(&["a"], 10, ApprovalLevel::Human);
        let gb = cap(&["b"], 10, ApprovalLevel::Human);
        let gc = cap(&["a", "b"], 10, ApprovalLevel::Human);
        for p in perms(&ga, &gb, &gc) {
            let eff = run(&p);
            let got = eff
                .get("r")
                .expect("the dominating grant C survives in every permutation");
            assert_eq!(got.destinations, DestinationBound::Only(s(&["a", "b"])));
            assert_eq!(got.quota, Ceiling::AtMost(10));
        }

        // THREE genuinely incomparable grants with NO dominator → DROP in every
        // permutation.
        let na = cap(&["a"], 1, ApprovalLevel::Auto);
        let nb = cap(&["b"], 1, ApprovalLevel::Auto);
        let nc = cap(&["c"], 1, ApprovalLevel::Auto);
        for p in perms(&na, &nb, &nc) {
            assert!(run(&p).is_empty(), "no dominating grant → drop");
        }

        // COMPARABLE pair: {a}/1/Human refines {a,b}/100/Human → keep the more
        // permissive ({a,b}/100/Human), in BOTH orders.
        let narrow = cap(&["a"], 1, ApprovalLevel::Human);
        let wide = cap(&["a", "b"], 100, ApprovalLevel::Human);
        for order in [
            vec![narrow.clone(), wide.clone()],
            vec![wide.clone(), narrow.clone()],
        ] {
            let eff = run(&order);
            let got = eff
                .get("r")
                .expect("comparable grants keep the more permissive");
            assert_eq!(got.destinations, DestinationBound::Only(s(&["a", "b"])));
            assert_eq!(got.quota, Ceiling::AtMost(100));
            assert_eq!(got.approval, ApprovalLevel::Human);
        }

        // FOUR grants where D={a,b,c} dominates A={a},B={b},C={c} (Codex r3:
        // a "return first for len>=4" mutation would miss this). D survives
        // with D LAST, and a 4-grant set with NO dominator drops.
        let fa = cap(&["a"], 5, ApprovalLevel::Auto);
        let fb = cap(&["b"], 5, ApprovalLevel::Auto);
        let fc = cap(&["c"], 5, ApprovalLevel::Auto);
        let fd = cap(&["a", "b", "c"], 5, ApprovalLevel::Auto);
        let eff = run(&[fa.clone(), fb.clone(), fc.clone(), fd.clone()]);
        assert_eq!(
            eff.get("r").map(|c| c.destinations.clone()),
            Some(DestinationBound::Only(s(&["a", "b", "c"]))),
            "the dominator survives even when it is the 4th grant"
        );
        // Remove the dominator → no maximum → drop.
        assert!(run(&[fa, fb, fc]).is_empty(), "4→3 with no dominator drops");
    }

    /// Layer 6 (`EffectNeed::One`) narrows to exactly the needed ref and drops
    /// every OTHER surviving ref. Two refs survive layers 1-5, then
    /// `One("a")` leaves only "a" (Codex r1 P2 — the old assertion narrowed to
    /// a ref the closure had already removed, which was vacuous).
    #[test]
    fn effect_need_narrows_to_single_surviving_ref() {
        let declared = EffectiveCapabilities::from_declared(
            [
                (
                    "a".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
                (
                    "b".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let closure = s(&["a", "b"]);
        let grant = |audience| CapabilityGrant {
            audience,
            granted: [
                (
                    "a".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
                (
                    "b".to_string(),
                    BoundedCapability::top(CapabilityClass::ReadPublic),
                ),
            ]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let run = |need: EffectNeed| {
            effective_capabilities(
                &declared,
                &OpenPolicy,
                &[grant(GrantAudience::Release)],
                &[grant(GrantAudience::Pack)],
                &closure,
                &need,
                &HitlValidity::none(),
                0,
            )
        };
        // Without narrowing: both survive.
        let both = run(EffectNeed::Any);
        assert!(both.get("a").is_some() && both.get("b").is_some());
        // Narrowed to "a": only "a" remains (removing the layer-6 filter would
        // leave "b" too → this fails).
        let only_a = run(EffectNeed::One("a".to_string()));
        assert!(only_a.get("a").is_some(), "the needed ref survives");
        assert!(
            only_a.get("b").is_none(),
            "every other ref is narrowed away"
        );
        assert_eq!(only_a.len(), 1);
    }

    /// The local node policy (layer 2) both DENIES refs absent from its
    /// allow-map and TIGHTENS the field bounds of those it allows. A no-op
    /// implementation of `meet_with_allow` would survive every other test
    /// (they all use `OpenPolicy`); this is its only discriminant guard (F2).
    #[test]
    fn local_policy_denies_and_tightens() {
        struct Op;
        impl CapabilityPolicy for Op {
            fn os_toggle(&self) -> ClassConstraint {
                ClassConstraint::Unconstrained
            }
            fn local_policy(&self) -> RefConstraint {
                // Allows only "keep", tightened to quota 1 + Human approval;
                // "drop" is absent from the allow-map.
                let mut c = BoundedCapability::top(CapabilityClass::WriteWorkspace);
                c.quota = Ceiling::AtMost(1);
                c.approval = ApprovalLevel::Human;
                RefConstraint::Only([("keep".to_string(), c)].into_iter().collect())
            }
        }
        let declared = EffectiveCapabilities::from_declared(
            [
                (
                    "keep".to_string(),
                    BoundedCapability::top(CapabilityClass::WriteWorkspace), // quota Unbounded, Auto
                ),
                (
                    "drop".to_string(),
                    BoundedCapability::top(CapabilityClass::WriteWorkspace),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let closure = s(&["keep", "drop"]);
        let grant = |audience| CapabilityGrant {
            audience,
            granted: [
                (
                    "keep".to_string(),
                    BoundedCapability::top(CapabilityClass::WriteWorkspace),
                ),
                (
                    "drop".to_string(),
                    BoundedCapability::top(CapabilityClass::WriteWorkspace),
                ),
            ]
            .into_iter()
            .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        // A decision so the Human-raised "keep" survives layer 7.
        let eff = effective_capabilities(
            &declared,
            &Op,
            &[grant(GrantAudience::Release)],
            &[grant(GrantAudience::Pack)],
            &closure,
            &EffectNeed::Any,
            &HitlValidity::from_refs_for_test(s(&["keep"])),
            0,
        );
        // "drop" is absent from the allow-map → denied.
        assert!(
            eff.get("drop").is_none(),
            "layer 2 denies a ref off the allow-map"
        );
        // "keep" survives, meet-tightened to the policy's EXACT bounds.
        let kept = eff.get("keep").expect("allowed ref survives layer 2");
        assert_eq!(kept.quota, Ceiling::AtMost(1), "layer 2 tightens the quota");
        assert_eq!(
            kept.approval,
            ApprovalLevel::Human,
            "layer 2 raises the approval requirement"
        );
    }
}
