// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host-neutral secret seam (Sprint 83 Phase L1): how a workflow effect
//! reaches a credential WITHOUT the engine ever seeing, storing, journaling or
//! serialising the material.
//!
//! ## The opaque reference, and why the engine stays blind (§14.3)
//!
//! A pack, a plan and every journal row carry only an opaque logical handle that
//! NAMES a slot, never a value (the n8n lesson, kickoff §D1: a schema that lets
//! credential material or even credential NAMES leak into the workflow document
//! is the failure mode this design refuses). The reference is safe to log,
//! journal and gossip: it is a slot name.
//!
//! Precisely: a pack carries `secret_refs: Vec<String>` — plain strings, checked
//! only by the compiler's `looks_like_materialised_secret` heuristic. Converting
//! them into a validated [`SecretRef`] is an obligation OF THE WIRING, not a
//! property the pack format already has.
//!
//! The MATERIAL bound to a ref never enters this crate's durable surfaces, with
//! the exact scope stated below: the type closes the SERIALISATION route, and
//! not copying the exposed bytes is the adapter's discipline.
//!
//! The binding is MEANT to be just-in-time and host-side: a [`SecretResolver`]
//! the HOST will supply (the daemon's keystore-backed vault,
//! `nexus_core_rs::vault`) is to resolve a ref to a [`BoundSecret`] immediately
//! before the effect call, in the effect adapter. This module defines the SEAM
//! — exactly as [`crate::clock::Clock`] and [`crate::signal::SignalSource`]
//! define time and signals. **Nothing implements or calls it in production
//! yet**: [`crate::activity::EffectContext`] has no resolver field, so the
//! sentence above describes the CONTRACT, not a wired path (0 call-site, DS-1).
//!
//! ## What the TYPE forecloses (and what it does not)
//!
//! [`BoundSecret`] is the type material lives in. What the TYPE forecloses is
//! precise, and narrower than "the material cannot leak": it cannot be
//! SERIALISED. A caller that `expose()`s it and copies the bytes into a `String`
//! defeats it — that copy is the effect adapter's responsibility, and the red
//! arm of `canary_absent_everywhere` exists exactly because the type alone does
//! not close it. What IS closed by construction:
//!
//! - it has NO `Serialize`/`Deserialize` impl, so it can never be a field of a
//!   [`crate::event::RunEvent`], an [`crate::activity::ActivityReceipt`] or a
//!   snapshot — a compile error, not a review comment;
//! - its `Debug` REDACTS (length only, never bytes), so a stray `{:?}` in a
//!   log line or an error path cannot spill it;
//! - it zeroes its buffer on drop ([`zeroize::Zeroizing`]), so the material
//!   does not linger in freed heap after the effect call.
//!
//! The canary test `canary_absent_everywhere` (G-SECRET) proves the runtime
//! half: a resolved canary value appears in NONE of the engine's serialisation
//! outputs, with a positive control proving the search would have caught a
//! leak. The compile-time half is the missing `Serialize` impl itself.
//!
//! ## Pre-launch protocol / frontier
//!
//! Nothing here is a signing family or a wire format: [`SecretRef`] is a plain
//! opaque string, [`BoundSecret`] is never serialised. The census of
//! `DOMAIN_*_V1` stays at 29 and no `*_FORMAT_VERSION` is touched. The
//! TEST-ACTOR that will read this seam is the host effect adapter — code in
//! `nexus-shell-daemon`, i.e. a DIFFERENT crate, but the same process, linked
//! against these very types rather than re-implementing them from a
//! description. That is what makes it not a frontier: no other node, no external
//! client, no app, no other LLM reads this contract, so it carries no frontier
//! tag (same reasoning as [`crate::clock`]).

use crate::hash::{HashError, validate_identity_component};

/// An OPAQUE logical handle naming a secret slot. It is the ONLY secret-related
/// value the engine ever handles: it travels in packs, plans and journals
/// because it is a name, not material. Resolution to the actual credential is
/// the host's job, just-in-time, via a [`SecretResolver`].
///
/// Built through [`SecretRef::new`], the validation gate: a ref is a bounded,
/// control-character-free string, same domain as an identity component, so a ref
/// obtained through `new` cannot smuggle a separator or a newline into a
/// line-oriented log.
///
/// `Serialize` only, deliberately: a `Deserialize` impl would rebuild the newtype
/// straight from bytes and SKIP that gate, which is the one thing this type
/// exists to prevent. Nothing in the tree deserialises a `SecretRef` today
/// (packs carry plain strings), so the derive would have bought a bypass for no
/// caller (ex-review r2 P3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct SecretRef(String);

impl SecretRef {
    /// Build a validated opaque secret reference.
    pub fn new(name: impl Into<String>) -> Result<Self, HashError> {
        let name = name.into();
        validate_identity_component("secret_ref", &name)?;
        Ok(Self(name))
    }

    /// The opaque slot name. Safe to log — it is a name, never material.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SecretRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A resolver could not bind a reference.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    /// No slot in the host vault answers this reference. Fail-closed: the
    /// effect that needed it must not run.
    #[error("unknown secret ref: {0}")]
    Unknown(String),
    /// The host vault refused the binding (locked, revoked, backend error).
    /// The message NEVER carries material — only the ref and the reason.
    #[error("secret binding refused for {reference}: {reason}")]
    Refused {
        /// The opaque reference that failed to bind.
        reference: String,
        /// Why the host refused. Material-free by contract.
        reason: String,
    },
}

/// Credential material, bound just-in-time: no `Serialize`, redacting `Debug`,
/// zeroed on drop. Those three close the SERIALISATION route — they do not close
/// the COPY route, which is the adapter's discipline (see the module doc). The
/// effect adapter [`expose`](BoundSecret::expose)s it for the duration of one
/// call and drops it immediately after.
///
/// The absence of `Serialize` is pinned HERE, by a compile-fail doctest, and
/// nowhere else. The canary test does NOT pin it: none of its five green
/// surfaces serialises a `BoundSecret`, so adding
/// `impl Serialize for BoundSecret { /* writes self.expose() */ }` would leak
/// the material and leave that whole test GREEN (Codex round 1 and round 2,
/// both times). A doctest is the right shape because the property is "this
/// code must not COMPILE", which no runtime assertion can express:
///
/// The error code is PINNED (`E0277`, unsatisfied trait bound). A bare
/// `compile_fail` passes on ANY compilation error — a typo, a missing crate, a
/// renamed type — so it can go green for a reason that has nothing to do with
/// the property. Pinning the code makes the test fail loudly if the reason
/// changes, which is the difference between a control and a decoration.
///
/// ```compile_fail,E0277
/// use sbfb_workflow_engine::BoundSecret;
///
/// fn must_not_compile(secret: &BoundSecret) -> String {
///     serde_json::to_string(secret).unwrap() // BoundSecret: !Serialize
/// }
/// ```
pub struct BoundSecret {
    material: zeroize::Zeroizing<Vec<u8>>,
}

impl BoundSecret {
    /// Wrap freshly-resolved material. The caller (a host resolver) is the
    /// only producer; the buffer is zeroed when this value drops.
    pub fn new(material: Vec<u8>) -> Self {
        Self {
            material: zeroize::Zeroizing::new(material),
        }
    }

    /// Borrow the material for the duration of one effect call.
    ///
    /// This borrow is the only ACCESSOR the API offers, and it does not outlive
    /// the `BoundSecret`. What it is NOT is a guarantee that no owned copy can
    /// exist: `.to_vec()` is one keystroke away, and that is precisely the RED
    /// arm of `canary_absent_everywhere`. The type closes the SERIALISATION
    /// route by construction; not making a copy is the adapter's discipline, as
    /// the module doc says. Stating it more strongly here than the module states
    /// it would be the discrepancy that makes a reader trust the wrong half.
    pub fn expose(&self) -> &[u8] {
        &self.material
    }

    /// Length of the material (the only fact the redacting `Debug` reveals).
    ///
    /// That length IS an oracle on the FORMAT of the credential — 51 bytes reads
    /// as an OpenAI `sk-…`, 40 as an AWS secret, 64 hex characters as an HMAC key.
    /// Bounded, not closed; see the honest bounds in §19.4 of the threat model.
    pub fn len(&self) -> usize {
        self.material.len()
    }

    /// Whether the material is empty.
    pub fn is_empty(&self) -> bool {
        self.material.is_empty()
    }

    /// Pin the WRAPPER type, not just the behaviour (ex-review r2 P2-4).
    ///
    /// Replacing `Zeroizing<Vec<u8>>` with a bare `Vec<u8>` compiles and keeps
    /// every test green — `&self.material` and `.len()` are identical through
    /// both — so a tidy-up refactor could silently drop the zeroisation while the
    /// threat model kept claiming it. This assertion makes the wrapper part of the
    /// compiled contract.
    ///
    /// What it pins, exactly (ex-review r3 D2-4 — the earlier wording claimed
    /// two guarantees and could support only one): it pins the ZEROISING
    /// WRAPPER. It does NOT prove the absence of `Serialize`. `zeroize` DOES
    /// expose an optional `serde` feature — Cargo synthesises it from the
    /// optional dependency, and `Zeroizing<Z>` has a `#[cfg(feature = "serde")]`
    /// `Serialize` impl upstream. This workspace simply does not ACTIVATE it,
    /// and features unify across the whole graph, so that is a BOUND, not a
    /// proof — a `Cargo.lock` records no features at all and cannot be the
    /// evidence. The absence of `Serialize` on `BoundSecret` ITSELF is pinned by
    /// the `compile_fail` doctest on the type, NOT by the canary's red arm —
    /// that arm copies `expose()` by hand into a receipt and never serialises a
    /// `BoundSecret`, so it would stay green against a leaking `Serialize` impl
    /// (Codex r1 and r2). Claim corrected in review round 10.
    #[cfg(test)]
    pub(crate) fn material_is_zeroizing(&self) -> &zeroize::Zeroizing<Vec<u8>> {
        &self.material
    }
}

// REDACTING Debug: the length is the only fact printed. A `{:?}` in a log line
// or an error path can never spill the bytes. (The absence of `Serialize` is
// the compile-time guarantee; this is the accidental-`Debug` guardrail.)
impl std::fmt::Debug for BoundSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BoundSecret(<redacted {} bytes>)", self.material.len())
    }
}

/// The host-side seam that resolves an opaque [`SecretRef`] to its material,
/// just-in-time.
///
/// The daemon's effect adapter WILL hold a `&dyn SecretResolver` and call it ONLY
/// from the effect path, never from a journaling one — the division
/// [`crate::signal::SignalSource`] uses for signals. No such adapter exists yet:
/// nothing in production implements or calls this trait, and
/// [`crate::activity::EffectContext`] has no resolver field (0 call-site, DS-1).
/// The present tense here would describe a wiring that is not written.
///
/// ## Notes for the binding phase — WHO may bind WHICH slot (DS-1)
///
/// `resolve` takes a ref and nothing else: no principal, no run, no capability,
/// no pack digest. **Naming a slot is obtaining it**, and slot names travel in
/// clear inside a signed pack. This is a real hole and it is deliberately left
/// to the wiring — but the ARBITRATION is written here so the wiring does not
/// inherit an implicit one (threat model §19.4, bound 23):
///
/// 1. `capability_ref` and `secret_ref` are **TWO namespaces**, not one.
///    `CapabilityClass::SecretUse` gates the right to use secrets AS A CLASS;
///    it never names a slot. Collapsing them would turn a successful capability
///    intersection into an arbitrary read.
/// 2. The authorisation belongs to the **SCOPE OF THE RESOLVER INSTANCE**, not
///    to an extra parameter. This trait is an object handed to one effect
///    context, so building it with exactly the run's authorised slots is the
///    object-capability answer — and [`ScriptedSecrets`] is already that shape.
///    It requires no signature change, hence no risk to a frozen preimage.
/// 3. An `authority: &SecretAuthority` parameter would be checked BY the caller
///    it is supposed to constrain — the shortest bypass, and the same class
///    [`crate::hitl::HostVerifiedApproval`] closed on the HITL side.
///
/// So the wiring owes two things: construct the resolver with the run's
/// authorised slot set, and journal the refusal.
pub trait SecretResolver {
    /// Bind `secret_ref` to its material. Fail-closed: an unknown or refused
    /// reference returns [`SecretError`], and the effect that needed it does
    /// not run.
    fn resolve(&self, secret_ref: &SecretRef) -> Result<BoundSecret, SecretError>;
}

/// A resolver that binds nothing — the default for a run with no secret-using
/// effect. Every reference is `Unknown` (fail-closed).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSecrets;

impl SecretResolver for NoSecrets {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<BoundSecret, SecretError> {
        Err(SecretError::Unknown(secret_ref.as_str().to_string()))
    }
}

/// A deterministic in-memory resolver for tests and golden fixtures: a fixed
/// `ref → material` map. Honest about what it is — no vault, no crypto, no
/// disk; the real host resolver is the keystore-backed vault in
/// `nexus-core-rs`. The material it holds is zeroed on drop, like the real
/// thing.
///
/// Not claimed: that a canary cannot linger in memory. A test canary is a `const`
/// string living in `.rodata` for the whole process, and the tests make several
/// owned copies of it — the zeroisation here concerns THIS map's buffers, nothing
/// more.
#[derive(Debug, Default)]
pub struct ScriptedSecrets {
    slots: std::collections::BTreeMap<String, zeroize::Zeroizing<Vec<u8>>>,
}

impl ScriptedSecrets {
    /// An empty resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Same wrapper pin as [`BoundSecret::material_is_zeroizing`]: the test
    /// double must not be the place where the zeroisation quietly disappears.
    #[cfg(test)]
    pub(crate) fn slots_are_zeroizing(
        &self,
    ) -> &std::collections::BTreeMap<String, zeroize::Zeroizing<Vec<u8>>> {
        &self.slots
    }

    /// Bind `secret_ref` to `material` in this test resolver.
    pub fn with(mut self, secret_ref: &SecretRef, material: impl Into<Vec<u8>>) -> Self {
        self.slots.insert(
            secret_ref.as_str().to_string(),
            zeroize::Zeroizing::new(material.into()),
        );
        self
    }
}

impl SecretResolver for ScriptedSecrets {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<BoundSecret, SecretError> {
        match self.slots.get(secret_ref.as_str()) {
            Some(material) => Ok(BoundSecret::new(material.to_vec())),
            None => Err(SecretError::Unknown(secret_ref.as_str().to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{ACTIVITY_RECEIPT_KIND, ActivityReceipt};
    use crate::event::{RUN_EVENT_FORMAT_VERSION, RunEvent, RunEventKind};
    use crate::hash::{StepRequest, step_request_hash};
    use std::collections::BTreeMap;

    /// A recognisable, unlikely-to-occur-by-accident canary value.
    const CANARY: &str = "CANARY-SECRET-9d3f7a1e-material-MUST-NOT-LEAK";

    fn secret_ref(name: &str) -> SecretRef {
        SecretRef::new(name).expect("valid ref")
    }

    /// The constructor is the ONLY gate on a ref's shape, and nothing else in
    /// this module exercised its REFUSING branch: every other test builds refs
    /// through the helper above, which unwraps. Dropping the
    /// `validate_identity_component` call from `SecretRef::new` therefore left
    /// this whole file green — the branch existed, no test reached it.
    /// The three arms below are the three shapes the validator rejects: empty,
    /// over-long, and carrying a character outside the identity domain.
    #[test]
    fn secret_ref_new_refuses_what_the_identity_domain_forbids() {
        assert!(SecretRef::new("").is_err(), "empty ref must be refused");
        assert!(
            SecretRef::new("a".repeat(crate::hash::IDENTITY_MAX_LEN + 1)).is_err(),
            "over-long ref must be refused"
        );
        assert!(
            SecretRef::new("stripe\u{202E}live").is_err(),
            "a bidi override in a ref must be refused"
        );
        // And the accepting branch, so the test cannot pass by refusing
        // everything — the failure mode of a guard test written in a hurry.
        assert!(SecretRef::new("stripe_live_key").is_ok());
    }

    /// Search a rendered surface for the material under every encoding a
    /// credential PLAUSIBLY takes on its way out, and say WHICH one matched.
    ///
    /// A raw-ASCII scan is not enough: `serde_json` renders a `Vec<u8>` as a
    /// DECIMAL ARRAY, and `{:?}` of a byte slice renders it as a space-separated
    /// one — a `contains(CANARY)` oracle is blind to both (ex-review P1-6b/P1-6d).
    /// Neither is that enough: an adapter that puts a credential in an HTTP
    /// `Authorization: Basic` header emits BASE64, a signing helper emits HEX, and
    /// any JSON serialiser emits the value ESCAPED as soon as it contains a quote,
    /// a backslash or a control byte. Each of those is a rendering the earlier
    /// three arms would have called clean (ex-review r2 P2-5), so the class is
    /// closed here rather than one arm at a time.
    ///
    /// The encoders are written out by hand: adding `base64`/`hex` as a dependency
    /// of this crate for a test helper would spend a frozen Day-0 (0 new runtime
    /// dep) on a convenience.
    ///
    /// Mirrored DELIBERATELY in `nexus_core_rs::vault`'s tests — recopied, not
    /// shared, since the two crates do not depend on one another; that twin has
    /// FEWER arms (it searches a sealed blob, not a rendered document).
    fn find_material(haystack: &str, needle: &[u8]) -> Option<&'static str> {
        if haystack.contains(&String::from_utf8_lossy(needle).to_string()) {
            return Some("raw text");
        }
        let compact = needle
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if haystack.contains(&compact) {
            return Some("json decimal array");
        }
        let spaced = needle
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        if haystack.contains(&spaced) {
            return Some("debug decimal array");
        }
        // JSON-escaped: what a serialiser actually writes. Derived FROM the
        // serialiser (quotes stripped) rather than guessed, so the arm cannot
        // drift from what `serde_json` emits.
        if let Ok(quoted) = serde_json::to_string(&String::from_utf8_lossy(needle).to_string()) {
            let escaped = quoted.trim_matches('"');
            if escaped != String::from_utf8_lossy(needle) && haystack.contains(escaped) {
                return Some("json-escaped text");
            }
        }
        if haystack.contains(&hex_lower(needle)) {
            return Some("hex");
        }
        if haystack.contains(&base64_standard(needle)) {
            return Some("base64");
        }
        if haystack.contains(&base64_url_safe(needle)) {
            return Some("base64url");
        }
        None
    }

    fn hex_lower(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// RFC 4648 base64, hand-rolled (see `find_material` on why there is no dep).
    fn base64_with(alphabet: &[u8; 64], pad: bool, bytes: &[u8]) -> String {
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = *chunk.get(1).unwrap_or(&0) as u32;
            let b2 = *chunk.get(2).unwrap_or(&0) as u32;
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(alphabet[(triple >> 18) as usize & 63] as char);
            out.push(alphabet[(triple >> 12) as usize & 63] as char);
            if chunk.len() > 1 {
                out.push(alphabet[(triple >> 6) as usize & 63] as char);
            } else if pad {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(alphabet[triple as usize & 63] as char);
            } else if pad {
                out.push('=');
            }
        }
        out
    }

    fn base64_standard(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        base64_with(ALPHABET, true, bytes)
    }

    fn base64_url_safe(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        base64_with(ALPHABET, false, bytes)
    }

    #[test]
    fn bound_secret_debug_redacts_material() {
        let bound = BoundSecret::new(CANARY.as_bytes().to_vec());
        // EXACT equality (ex-review P1-6a): `!contains(CANARY)` + `contains
        // ("redacted")` both stay true if someone appends `{:?}` of the byte
        // slice, which spills the entire credential in decimal. Pinning the
        // whole rendering closes that one-line leak.
        assert_eq!(
            format!("{bound:?}"),
            format!("BoundSecret(<redacted {} bytes>)", CANARY.len())
        );
        // And the search used by the canary test detects the decimal form, so
        // the assertion above is not the only line defending it.
        assert_eq!(
            find_material(&format!("{bound:?}"), CANARY.as_bytes()),
            None
        );
    }

    #[test]
    fn resolver_binds_and_fails_closed() {
        let cred = secret_ref("openai_api_key");
        let resolver = ScriptedSecrets::new().with(&cred, CANARY.as_bytes().to_vec());

        let bound = resolver.resolve(&cred).expect("known ref binds");
        assert_eq!(bound.expose(), CANARY.as_bytes());

        let missing = secret_ref("absent_ref");
        match resolver.resolve(&missing) {
            Err(SecretError::Unknown(r)) => assert_eq!(r, "absent_ref"),
            other => panic!("unknown ref must fail closed, got {other:?}"),
        }

        // The default resolver binds nothing.
        assert!(matches!(
            NoSecrets.resolve(&cred),
            Err(SecretError::Unknown(_))
        ));
    }

    /// G-SECRET. The canary bound value appears in NONE of the engine's
    /// serialisation surfaces. The realistic path: a secret-using effect
    /// declares the OPAQUE REF in its journaled input (never the value), the
    /// host resolver binds the material transiently, and every durable
    /// artifact is searched.
    ///
    /// ## Why this test is discriminating (ex-review P1-6)
    ///
    /// Its first version built every surface itself, never routed the material
    /// anywhere, and "proved" absence with a control that only exercised
    /// `str::contains` on a string it had just concatenated. NOTHING about the
    /// production path could have made it red — not even deriving `Serialize`
    /// on [`BoundSecret`], i.e. destroying the compile-time guarantee the
    /// module advertises.
    ///
    /// Three changes fix that — for what a RUNTIME test can reach. None of
    /// them closes the `Serialize` example above: no surface here serialises a
    /// `BoundSecret`, so that class is pinned by the compile-fail doctest on
    /// the TYPE, not by this test (Codex rounds 1 to 5). (1) A **RED ARM**
    /// builds the SAME
    /// [`ActivityReceipt`] with its `facts` fed from `bound.expose()` — the
    /// realistic Phase-N leak, where a provider error echoes the credential —
    /// and REQUIRES the search to find it. So the search is proven able to fail
    /// on the very surface the green arm then declares clean. (2) The search
    /// covers SEVEN arms for six encoding families (`find_material`): raw text,
    /// `serde_json` decimal array, `Debug` decimal array, JSON-escaped text,
    /// hex, base64 standard and base64url. (3) The REF is asserted
    /// PRESENT in the journaled input, so "clean" cannot be obtained by
    /// journaling nothing at all.
    #[test]
    fn canary_absent_everywhere() {
        let cred = secret_ref("cloud_provider_token");
        let resolver = ScriptedSecrets::new().with(&cred, CANARY.as_bytes().to_vec());

        // The effect's journaled input carries the REF, never the material.
        let mut input = BTreeMap::new();
        input.insert(
            "credential".to_string(),
            serde_json::to_string(cred.as_str()).unwrap(),
        );
        let request = StepRequest::new("root/call#1", "cloud-model", input.clone())
            .expect("valid step request");

        // Host binds the material JUST-IN-TIME for the call, then it drops.
        let bound = resolver.resolve(&cred).expect("binds");
        assert_eq!(bound.expose(), CANARY.as_bytes());

        // The durable artifacts the engine actually writes.
        let request_jcs = serde_json::to_string(&request).unwrap();
        let request_hash = step_request_hash(&request).unwrap();
        let receipt = ActivityReceipt {
            kind: ACTIVITY_RECEIPT_KIND.to_string(),
            run_id: "run-1".to_string(),
            node_instance_id: "root/call#1".to_string(),
            effect_id: "eff-1".to_string(),
            attempt: 1,
            request_hash: request_hash.clone(),
            activity: "cloud-model".to_string(),
            dedup_scope: "cloud-model".to_string(),
            fencing_epoch: 0,
            facts: BTreeMap::new(),
            output: serde_json::json!({"status": "ok"}),
        };
        let receipt_render = receipt.render().unwrap();
        let event = RunEvent {
            format_version: RUN_EVENT_FORMAT_VERSION,
            run_id: "run-1".to_string(),
            sequence: 7,
            node_instance_id: Some("root/call#1".to_string()),
            request_hash: Some(request_hash.clone()),
            kind: RunEventKind::EffectCompleted {
                effect_id: "eff-1".to_string(),
                receipt_digest: crate::hash::receipt_digest(&receipt_render),
            },
        };
        let event_json = serde_json::to_string(&event).unwrap();

        // The OPAQUE REF must be PRESENT in the journaled input: "no material"
        // must not be obtainable by journaling nothing at all.
        assert!(
            request_jcs.contains(cred.as_str()),
            "the opaque ref belongs in the journaled input: {request_jcs}"
        );

        // ---- RED ARM: the same production surface, WITH the material ----
        // A leaky activity that puts the exposed credential into its outcome
        // facts — the realistic Phase-N failure (a provider echoing the token
        // in an error body). The search MUST find it here, which is what makes
        // its silence on the green arm below meaningful.
        let mut leaky_facts = BTreeMap::new();
        leaky_facts.insert(
            "provider_error".to_string(),
            String::from_utf8(bound.expose().to_vec()).unwrap(),
        );
        let leaky_receipt = ActivityReceipt {
            facts: leaky_facts,
            ..receipt.clone()
        };
        let leaky_render = leaky_receipt.render().unwrap();
        assert_eq!(
            find_material(&leaky_render, CANARY.as_bytes()),
            Some("raw text"),
            "RED ARM: a receipt carrying the material must be DETECTED — \
             otherwise the green arm below proves nothing"
        );
        // Same, for the byte-array encoding a `Vec<u8>` output would take.
        let leaky_output = ActivityReceipt {
            output: serde_json::json!({ "echo": CANARY.as_bytes().to_vec() }),
            ..receipt.clone()
        };
        assert_eq!(
            find_material(&leaky_output.render().unwrap(), CANARY.as_bytes()),
            Some("json decimal array"),
            "RED ARM: the decimal-array encoding must also be detected"
        );

        // ---- GREEN ARM: the surfaces the engine actually writes ----
        let surfaces = [
            ("step_request_jcs", request_jcs),
            ("step_request_hash", request_hash),
            ("activity_receipt_render", receipt_render.clone()),
            ("run_event_json", event_json),
            ("bound_secret_debug", format!("{bound:?}")),
        ];
        // Pin the COUNT too (ex-review r3 L1-D5-2): removing an entry from this
        // table shrinks what the canary covers while keeping the test green, and
        // the threat model names "5 surfaces" explicitly.
        assert_eq!(
            surfaces.len(),
            5,
            "a surface was dropped from the canary sweep"
        );
        for (name, surface) in &surfaces {
            assert_eq!(
                find_material(surface, CANARY.as_bytes()),
                None,
                "canary material leaked into {name}: {surface}"
            );
        }
    }

    /// RED ARMS for the encodings added in ex-review r2 P2-5. Each is a rendering
    /// the previous oracle called CLEAN, which is the whole reason they exist:
    /// without these controls the new arms could be silently broken (or the
    /// alphabets typo'd) and `canary_absent_everywhere` would keep passing.
    #[test]
    fn the_canary_oracle_detects_base64_hex_and_json_escaping() {
        let needle = CANARY.as_bytes();

        // The single most plausible real leak: HTTP Basic auth is base64.
        let basic = format!(
            "{{\"authorization\":\"Basic {}\"}}",
            base64_standard(needle)
        );
        assert_eq!(find_material(&basic, needle), Some("base64"));

        // A URL-safe token. Distinguishing the two alphabets needs material whose
        // standard encoding actually USES `+`/`/` — for a 45-byte ASCII canary the
        // two spellings are byte-identical (45 % 3 == 0, so no padding either),
        // and the standard arm legitimately matches first.
        let binary = [0xfb_u8, 0xff, 0xbf];
        assert_eq!(base64_standard(&binary), "+/+/");
        let urlsafe = format!("?token={}", base64_url_safe(&binary));
        assert_eq!(find_material(&urlsafe, &binary), Some("base64url"));

        // A signing helper rendering the key as hex.
        let hexed = format!("key={}", hex_lower(needle));
        assert_eq!(find_material(&hexed, needle), Some("hex"));

        // And the escaping a serialiser applies on its own: a credential holding
        // a quote or a backslash (PEM keys, JSON-in-JSON) is rendered escaped, so
        // a raw-text scan misses it.
        let awkward = b"secret\"with\\quotes";
        let rendered =
            serde_json::to_string(&serde_json::json!({ "k": String::from_utf8_lossy(awkward) }))
                .expect("render");
        assert_eq!(find_material(&rendered, awkward), Some("json-escaped text"));

        // The 7th arm — the space-separated decimal a `{:?}` of a byte slice
        // emits — had no positive control at all, while the threat model
        // enumerated it and claimed "each with its RED arm" (ex-review r3
        // L1-D5-2). Changing `join(", ")` to `join(",")` made arm 2 answer first
        // with a different label and nothing noticed.
        let debug_rendered = format!("{:?}", needle);
        assert_eq!(
            find_material(&debug_rendered, needle),
            Some("debug decimal array")
        );

        // Sanity: the oracle is not simply always-positive.
        assert_eq!(find_material("nothing to see here", needle), None);
    }

    /// The zeroisation of both the real type and the test double is pinned at the
    /// TYPE level, not merely observed (ex-review r2 P2-4): swapping
    /// `Zeroizing<Vec<u8>>` for a bare `Vec<u8>` compiles and leaves every other
    /// test green, so nothing else here would notice the guarantee disappearing.
    #[test]
    fn material_wrappers_are_pinned_to_zeroizing() {
        fn assert_zeroizing<T: zeroize::Zeroize>(_: &zeroize::Zeroizing<T>) {}

        let bound = BoundSecret::new(CANARY.as_bytes().to_vec());
        assert_zeroizing(bound.material_is_zeroizing());

        let scripted = ScriptedSecrets::new().with(&secret_ref("slot"), CANARY.as_bytes().to_vec());
        for buffer in scripted.slots_are_zeroizing().values() {
            assert_zeroizing(buffer);
        }
    }
}
