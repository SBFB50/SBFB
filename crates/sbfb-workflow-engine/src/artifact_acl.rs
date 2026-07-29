// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed artefacts and the read ACL (Sprint 83 Phase M): how a run publishes a
//! durable output WITHOUT the digest that names it becoming the right to read it.
//!
//! ## The invariant, and why a facade could not hold it (§17)
//!
//! `G-ARTIFACT-ACL` states one property: **knowing the digest of a local
//! artefact does not permit reading it**. The obvious implementation — a policy
//! check in front of a read API — cannot hold it here, and the preflight
//! measured why:
//!
//! - `/blob-serve` is mounted in the daemon's `public_routes`, documented there
//!   as "no bearer, no Host check, no Origin check"
//!   (`crates/nexus-shell-daemon/src/http.rs:272`);
//! - the node accepts `BLOBS_ALPN` with `BlobsProtocol::new(&blobs_store, None)`
//!   (`crates/nexus-core-rs/src/node.rs:442`), which resolves on the default
//!   event sender and serves any stored blob to any dialer.
//!
//! Both paths reach the blob store by digest WITHOUT passing through any code
//! this crate could gate. A facade ACL would therefore be a claim the network
//! contradicts. So the ACL here is STRUCTURAL: for the mode that carries a read
//! restriction, the PLAINTEXT never enters the store those two paths serve —
//! along the vault's publishing path, which is the only one this crate owns (see
//! [`ArtifactVault`] for the exact scope). What a digest addresses is
//! ciphertext; **the read right is the KEY**, held as a separate capability.
//!
//! That is the verify-cap / read-cap split of Tahoe-LAFS, and it is what
//! reconciles the two halves of the phase brief that otherwise contradict each
//! other — "content-addressed" (the digest must be derivable and shareable) and
//! "digest is not a read right".
//!
//! Consequence worth stating because it prevents a false alarm later: the S16
//! decision documented at `http.rs:246-256` — "the blob content is already
//! public by construction" — is NOT revoked by this phase. It stays true
//! precisely because a local artefact never submits plaintext to that store.
//!
//! ## Two modes, because the system only has two (PO arbitration, 2026-07-28)
//!
//! An artefact is either PUBLISHED — stored in the clear, therefore already
//! reachable by any peer — or LOCAL — sealed, therefore never leaving. There is
//! no third state, and the sprint says why: scope cut 10 of `sprint83_plan.md`
//! defers the recipient-scoped encrypted envelope, `G-NETWORK` requires a shared
//! Task to be "public-only sans envelope chiffré", and the Research Pack's Phase
//! P sets the floor — until that envelope exists, only `public` data may enter a
//! shared Task.
//!
//! Two earlier drafts of this module carried more:
//!
//! - a `ReadAudience`, a set of principals allowed to read. Removed: nothing can
//!   hand those principals the key, so the list named readers who could never
//!   read.
//! - four classes (`public`/`private`/`secret`/`network`), following the Research
//!   Pack's chapter 24 wording. Removed after measurement: `public` and `network`
//!   are the SAME mode, because a blob stored in the clear is already served to
//!   any dialer by the two paths above — the "network" label added nothing an
//!   attacker did not already have. And `private` versus `secret` rested on a
//!   render predicate with zero production callers, while credential material
//!   already has its own non-serialisable type since Phase L1
//!   ([`crate::secret::BoundSecret`]) and has no business being an artefact.
//!
//! What remains is one bit, and the three predicates below are its three honest
//! readings:
//!
//! | mode | stored | leaves the node | deletion |
//! |---|---|---|---|
//! | [`ArtifactMode::Published`] | plaintext | yes — immediately, to any peer | **impossible** |
//! | [`ArtifactMode::Local`] | sealed | no | **real** — the key is destroyed |
//!
//! The deletion column is the load-bearing difference, and it is the reason the
//! split falls here rather than anywhere else.
//!
//! ## What "deletion" means, honestly
//!
//! For a LOCAL artefact, deletion is CRYPTO-SHREDDING: the key is destroyed and
//! the ciphertext becomes unreadable TO ANYONE WHO DID NOT COPY THE KEY OUT OF
//! ITS CAPABILITY. Bytes are not erased —
//! `crates/nexus-core-rs/src/blobs.rs:130-133` says dropping the last tag makes a
//! blob GC-ELIGIBLE, "though no GC is scheduled today, so this makes the blob
//! GC-eligible, it does not free disk now" — but nobody can read them again,
//! which is the guarantee that matters.
//!
//! That qualifier is not pedantry and it is the bound this module is most often
//! read without: [`ReadCapability::expose`] hands out a borrow that `.to_vec()`
//! turns into an owned copy, and [`ReadCapability::new`] is public, so a holder
//! who copied the bytes rebuilds a working capability after the deletion. The
//! TYPE closes the serialisation route by construction; not copying is the
//! caller's discipline. Same shape as the COPY route [`crate::secret`] states
//! for `BoundSecret`, and bound 10 of §19.5 in the threat model.
//!
//! For a PUBLISHED artefact, deletion is REFUSED by [`ArtifactVault::delete`]
//! rather than performed badly. Its plaintext is in the store, the store answers
//! any peer, and copies may already be elsewhere: an API that reported success
//! would be telling the user their data is gone when it is not.
//!
//! ## At-rest bounds — and the custody that does NOT exist yet
//!
//! NOTHING here holds the sealing key. `nexus_core_rs::artifact_seal` mints it
//! from the OS CSPRNG at seal time and hands it straight back as the read
//! capability — "Nothing here stores it", in that module's own words — and this
//! crate passes it to the caller. There is no artefact custody in
//! `nexus_core_rs::vault` or `keystore`: neither exposes any artefact API.
//!
//! Two consequences, stated rather than implied:
//!
//! - **No at-rest bound applies to an artefact key today**, because no artefact
//!   key is at rest. The Argon2id(PIN) bound — with its OS-keyring second layer
//!   INERT (carry `KEYSTORE-LAYER2-INERT`: `keyring` is declared without a
//!   backend feature, so the builder is a mock) — is what
//!   `nexus_core_rs::vault` guarantees FOR ITS OWN entries. It is cited here as
//!   the pattern a future custody should follow, not as a protection this phase
//!   provides. This module claims no layer at all, let alone a double one.
//! - **[`ArtifactRetention::Keep`] has no persistence behind it.** A capability
//!   lives in the holder's memory and dies with the process, so a local artefact
//!   is de facto unreadable after a restart. Storing capabilities is the wiring
//!   phase's work, not M's.
//!
//! Cost stated rather than hidden: a fresh key per artefact means two identical
//! local payloads produce two different ciphertexts and two different digests, so
//! content-addressed DEDUPLICATION is lost for sealed artefacts. That is the
//! price of the shredding guarantee above.
//!
//! ## Pre-launch protocol / frontier
//!
//! [`ArtifactRef`] is additive wire (AM-12) and introduces NO `DOMAIN_*_V1`
//! signing family: the census stays frozen at 29 and no `*_VERSION` is bumped.
//! It is signed by nobody here — the provenance graph of Phase M1 is what binds
//! artefacts into a signed structure.
//!
//! TEST-ACTOR — "who reads this primitive that is not this crate's own code?":
//!
//! 1. **Another node (wire).** The compute wire of Phase P carries
//!    `Task`/`Result`/`ArtifactRef` as additive 0-bump payloads — and, under the
//!    public-only floor, may carry only `Published` payloads.
//! 2. **Another app (contract).** Phase Y resolves an `ArtifactRef` under this
//!    ACL, and acceptance WA-T2.7 has a second app consume the artefact.
//! 3. **A host implementer in another crate.** [`ArtifactSealer`] is implemented
//!    by `nexus-shell-daemon` (`artifact_sealer.rs`), linked against these types
//!    rather than re-deriving them from prose — same-process, so a paragraph
//!    rather than a machine tag (the reasoning [`crate::secret`] uses).
//!    [`ArtifactBlobs`] has NO production implementer yet: the daemon's
//!    `BlobsClient` is the intended one, and wiring it belongs to the phase that
//!    introduces artefact-producing effects. `nexus-core-rs` cannot implement
//!    either trait — it has no dependency on this crate, and must not gain one,
//!    or the layering becomes cyclic.
//!
//! No `// FRONTIER:` tag is posted: that opt-in registry requires a `DOMAIN`, a
//! resolvable `*_VERSION` and a `schema_for!`, so tagging an unsigned type would
//! make the gate demand a signing family that deliberately does not exist.

use crate::clock::Clock;

/// Whether an artefact is published (in the clear, therefore already reachable
/// by any peer) or local (sealed, therefore never leaving).
///
/// One bit. See the module doc for why the four-class taxonomy that preceded it
/// encoded distinctions the system does not make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactMode {
    /// Stored in the clear. Reachable by anyone holding the digest — including
    /// remote peers, since `BLOBS_ALPN` serves every stored blob without a
    /// filter. Deletion is refused, because it could not be honoured.
    Published,
    /// Sealed with a per-artefact key. The plaintext never enters the store, so
    /// it never leaves the node; deletion destroys the key and is real.
    Local,
}

impl ArtifactMode {
    /// Both modes, for the tests that iterate them.
    ///
    /// What holds this list, and what does NOT — the distinction matters because
    /// an earlier revision of this comment claimed the second as the first.
    ///
    /// HELD: `every_mode_is_listed_in_all_exactly_once` rejects a duplicate, and
    /// its `slot` match is exhaustive, so adding a variant fails to COMPILE at a
    /// line whose only subject is `ALL`. The three predicates below and
    /// `nexus-shell-daemon`'s `mode_label` fail to compile too, so a new mode
    /// cannot be smuggled in without deciding its behaviour everywhere.
    ///
    /// NOT HELD: nothing makes the compiler INSERT a variant into a literal
    /// array. Measured on this module — a third variant with all its arms
    /// written and `ALL` left alone kept every runtime test green. `ALL.len()` is a fact about
    /// the type `[ArtifactMode; 2]`, so an assertion on it can never go red. A
    /// `match` over the ELEMENTS is worse still: it proves each element is a
    /// variant, never the converse. The residual bound is carried in §19.5 of
    /// the threat model rather than papered over here.
    pub const ALL: [ArtifactMode; 2] = [ArtifactMode::Published, ArtifactMode::Local];

    /// Whether the bytes must be sealed before entering the blob store.
    ///
    /// Exhaustive `match`, never a `_` arm: a new variant must be a compile error
    /// here, not a silent inheritance of another mode's policy. Same for the two
    /// predicates below — together they make adding a mode impossible without
    /// deciding its three behaviours.
    pub fn requires_sealing(self) -> bool {
        match self {
            ArtifactMode::Local => true,
            ArtifactMode::Published => false,
        }
    }

    /// Whether an artefact of this mode may leave the node.
    ///
    /// For `Published` this is not a permission so much as an observation: the
    /// bytes are already served to any dialer. For `Local` it is the public-only
    /// floor — a sealed artefact that travelled could not be opened by anyone
    /// there, since nothing distributes its key (scope cut 10).
    pub fn may_leave_node(self) -> bool {
        match self {
            ArtifactMode::Published => true,
            ArtifactMode::Local => false,
        }
    }

    /// Whether deletion can be honoured at all.
    pub fn deletable(self) -> bool {
        match self {
            ArtifactMode::Local => true,
            ArtifactMode::Published => false,
        }
    }
}

/// How long an artefact stays readable.
///
/// Named `ArtifactRetention` rather than `Retention` because
/// [`crate::store::RetentionPolicy`] already governs the run store's rows; the
/// two are unrelated and must not read as one policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ArtifactRetention {
    /// Kept until explicitly deleted.
    ///
    /// "Kept" is about POLICY, not durability: nothing in this phase persists a
    /// read capability, so a local artefact's key dies with the process holding
    /// it and the ciphertext outlives its own readability. See the module's
    /// at-rest section.
    Keep,
    /// Unreadable once the host clock reaches this wall-clock millisecond.
    ///
    /// Expiry is enforced on the READ path, not by a reaper: no GC is scheduled
    /// in this tree, so a retention that claimed to remove bytes would be the
    /// same lie deletion would be. What it enforces is that the engine refuses
    /// to hand back the plaintext.
    ExpiresAtMs {
        /// Wall-clock millisecond at which the artefact stops being readable.
        at_ms: i64,
    },
}

impl ArtifactRetention {
    /// Whether the artefact is expired at `now_ms`. The bound is INCLUSIVE:
    /// `at_ms` itself is already expired.
    pub fn expired_at(self, now_ms: i64) -> bool {
        match self {
            ArtifactRetention::Keep => false,
            ArtifactRetention::ExpiresAtMs { at_ms } => now_ms >= at_ms,
        }
    }
}

/// The content address of what is STORED — which, for a local artefact, is the
/// ciphertext. Lowercase hex, blake3, as everywhere else in the tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ArtifactDigest(String);

impl ArtifactDigest {
    /// Compute the digest of the stored bytes.
    pub fn of_stored_bytes(stored: &[u8]) -> Self {
        Self(blake3::hash(stored).to_hex().to_string())
    }

    /// Rebuild a digest from its hex form — the only way a peer that received a
    /// serialised [`ArtifactRef`] can reconstruct one.
    ///
    /// Without this, "additive wire" was a promise nothing could keep: this type
    /// has no `Deserialize`, and `of_stored_bytes` needs the BYTES, which a peer
    /// holding only the JSON does not have YET. The reasoning is CIRCULAR, and
    /// that is exactly what makes it hold: the two public paths hand the
    /// ciphertext to anyone presenting the digest, so a peer CAN obtain those
    /// bytes — but only after decoding the digest, which is this function. Phase
    /// P carries `ArtifactRef` on the compute wire; the decoding end has to
    /// exist before that form is frozen.
    ///
    /// This is also where the constructor invariant the type invokes to refuse a
    /// `Deserialize` derive is finally APPLIED, rather than merely cited:
    /// exactly 64 lowercase hex characters, which is what
    /// `blake3::Hash::to_hex` emits. Uppercase is refused rather than
    /// normalised, so two references to the same artefact cannot differ by case
    /// and compare unequal.
    pub fn from_hex(hex: &str) -> Result<Self, ArtifactError> {
        const BLAKE3_HEX_LEN: usize = 64;
        if hex.len() != BLAKE3_HEX_LEN
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            // Echo a BOUNDED prefix, never the whole input: this parses
            // caller-supplied wire text, and copying it verbatim into an error
            // makes the error as large as the payload (measured: a 200 kB input
            // produced a 200 kB error). A digest is 64 characters; anything past
            // that adds no diagnostic value.
            const ECHO_LIMIT: usize = 72;
            let echo: String = hex.chars().take(ECHO_LIMIT).collect();
            return Err(ArtifactError::MalformedDigest {
                digest: if echo.len() < hex.len() {
                    format!("{echo}…")
                } else {
                    echo
                },
            });
        }
        Ok(Self(hex.to_string()))
    }

    /// The lowercase hex digest.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArtifactDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The reference that TRAVELS: in a compiled plan, a journal row, a receipt, a
/// compute payload. It names an artefact and states its policy; it carries no
/// key, so publishing it is safe by construction.
///
/// `Serialize` only, for the reason [`crate::secret::SecretRef`] gives: a
/// `Deserialize` derive would rebuild the struct straight from bytes and skip
/// any constructor invariant.
///
/// # Every field is private, and that is load-bearing
///
/// An earlier revision made `digest`, `mode` and `retention` `pub` and kept only
/// `origin` private, reasoning that a private origin was enough to tell an
/// authentic reference from a forged one. It was not, and the gap was MEASURED:
/// a private field blocks CONSTRUCTION, never assignment to a `pub` field of a
/// value one already owns. So
///
/// ```ignore
/// let (mut reference, _cap) = vault.publish(Published, Keep, bytes)?;
/// reference.mode = ArtifactMode::Local;   // origin is still ThisVault
/// vault.delete(&reference, Some(cap))?;   // Ok(()) — and the store keeps serving
/// ```
///
/// untagged a PUBLISHED blob and reported success while the plaintext was still
/// served to every peer; re-pointing `digest` in the same way untagged an
/// unrelated blob, which is an arbitrary de-tag primitive. The provenance guard
/// stopped a reference that had been REBUILT and let through one that had been
/// EDITED — the same distinction the four dimensions of the round-3 review each
/// found independently.
///
/// With the fields private, mutating any of them requires going back through
/// [`ArtifactRef::from_untrusted_wire`], which stamps [`RefOrigin::UntrustedWire`]
/// — so an edited reference is exactly as untrusted as a rebuilt one, and the
/// guard on `delete` covers both. This closes the CLASS rather than the `delete`
/// instance: [`ArtifactVault::read`], [`ArtifactPolicy::authorise_read`],
/// [`ArtifactPolicy::authorise_egress`] and [`ArtifactPolicy::authorise_delete`]
/// all read these fields off the reference, and none of them can now be handed a
/// field that was changed after minting.
///
/// What a caller CAN still do is build a fresh untrusted reference over any
/// digest it knows. What that buys, precisely:
///
/// - **Claiming the MODE buys nothing on the read path.** A reference claiming
///   `Published` over a sealed digest yields the stored bytes, which are
///   ciphertext — exactly what `/blob-serve` would have handed over anyway. No
///   plaintext is reachable through a forged label, because no plaintext is in
///   the store. A facade ACL would have been broken by this; this one is not.
/// - **Claiming the mode DOES move the egress door.**
///   [`ArtifactPolicy::authorise_egress`] reads the mode off the REFERENCE, so a
///   forged `Published` label makes a sealed artefact's ciphertext exportable.
///   That is a confidentiality non-event (nobody can open it) but an integrity
///   one, and callers must not treat a caller-supplied reference as
///   authenticated. Binding a reference to its producer is the provenance work
///   of Phase M1. `authorise_delete` is no longer reachable this way: `delete`
///   refuses a non-minted reference before consulting the mode at all.
/// - **Claiming the RETENTION resurrects an expired read.** Rebuilding a
///   reference with `Keep` over an expired digest makes [`ArtifactVault::read`]
///   hand the plaintext back to a capability holder who had been cut off. This
///   is the one case where a forged reference obtains what the authentic one is
///   refused. Retention is therefore a POLICY the holder can decline to honour,
///   not a revocation — revocation is destroying the key.
///
/// # Why `PartialEq` counts `origin`
///
/// Decided rather than inherited from the derive. Two references over the same
/// digest are NOT interchangeable when one was minted here and the other was
/// supplied by a caller: they authorise different operations, and `delete` acts
/// on exactly that difference. Making `==` ignore provenance would let a test —
/// or a future dedup — treat them as one. Use
/// [`ArtifactRef::addresses_same_artefact`] when the question is "same artefact?"
/// rather than "same reference?"; Phase Y's idempotent resolution wants that one.
/// No serialisation round-trip can observe the difference, because this type has
/// no `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ArtifactRef {
    /// Content address of the stored bytes.
    digest: ArtifactDigest,
    /// Published or local.
    mode: ArtifactMode,
    /// How long it stays readable.
    retention: ArtifactRetention,
    /// Where this reference came from. Never serialised: it is the one field no
    /// constructor lets a caller choose, which is what makes it worth checking.
    #[serde(skip)]
    origin: RefOrigin,
}

/// Whether a reference was produced by a vault on this node, or reconstructed
/// from an untrusted source.
///
/// Private to this module, with no public constructor: [`ArtifactVault::publish`]
/// is the only producer of `ThisVault`. That is what lets [`ArtifactVault::delete`]
/// tell an authentic reference from a forged one — a distinction a public
/// `mode` field alone can never make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RefOrigin {
    /// Minted by `publish` on this node.
    ThisVault,
    /// Rebuilt from bytes, a peer, an app, or anything else. Default, because
    /// the safe assumption about a reference of unknown provenance is that it is
    /// untrusted.
    #[default]
    UntrustedWire,
}

impl ArtifactRef {
    /// Rebuild a reference from an UNTRUSTED source — a wire payload, a stored
    /// plan, another app. A caller holding only the serialised form gets its
    /// digest back through [`ArtifactDigest::from_hex`]; that is the whole
    /// decoding path, since this type has no `Deserialize`.
    ///
    /// The name is the warning, and it is deliberately the only public
    /// constructor. Everything such a reference asserts about itself — its mode,
    /// its retention — is caller-supplied and unverified, which measurably
    /// matters:
    ///
    /// - re-labelling a `Published` digest as `Local` used to let
    ///   [`ArtifactVault::delete`] untag a published blob and report SUCCESS,
    ///   while the store kept serving the plaintext to every peer. `delete` now
    ///   refuses any reference that is not [`RefOrigin::ThisVault`];
    /// - re-labelling a `Local` digest as `Published` still makes
    ///   [`ArtifactPolicy::authorise_egress`] say yes. That one is NOT closed
    ///   here: what leaves is ciphertext nobody can open, so it is an integrity
    ///   event rather than a disclosure, and closing it properly needs the signed
    ///   provenance of Phase M1;
    /// - re-labelling the retention as `Keep` resurrects an expired read for a
    ///   capability holder.
    ///
    /// Treat what it says as a claim to verify, never as authority.
    ///
    /// This is also the ONLY way to change what a reference says: the fields are
    /// private, so an edited reference must be rebuilt here and arrives stamped
    /// [`RefOrigin::UntrustedWire`]. Editing and rebuilding are deliberately the
    /// same operation.
    ///
    /// Rebuilding one from a minted reference's own accessors is not a trick that
    /// preserves trust — it produces an untrusted reference, which is what the
    /// following will not compile past:
    ///
    /// ```compile_fail,E0616
    /// use sbfb_workflow_engine::artifact_acl::{ArtifactMode, ArtifactRef};
    ///
    /// fn must_not_compile(reference: &mut ArtifactRef) {
    ///     reference.mode = ArtifactMode::Local; // field `mode` is private
    /// }
    /// ```
    ///
    /// TWIN, and it is load-bearing: **rustdoc does NOT enforce the annotated
    /// error code**. Measured on rustc 1.95.0, editions 2021 AND 2024 — a
    /// `compile_fail,E0616` block whose real error is `E0308` or `E0432` still
    /// passes; only "it compiled" fails the block. So the block above would go
    /// green on its own if this module were renamed, moved or made private,
    /// while `mode` quietly went back to `pub`. The twin below shares its
    /// imports and its surface and COMPILES, so any such drift turns it red.
    ///
    /// ```
    /// use sbfb_workflow_engine::artifact_acl::{ArtifactMode, ArtifactRef};
    ///
    /// fn must_compile(reference: &ArtifactRef) -> ArtifactMode {
    ///     reference.mode()
    /// }
    /// ```
    pub fn from_untrusted_wire(
        digest: ArtifactDigest,
        mode: ArtifactMode,
        retention: ArtifactRetention,
    ) -> Self {
        Self {
            digest,
            mode,
            retention,
            origin: RefOrigin::UntrustedWire,
        }
    }

    /// The content address of the stored bytes.
    pub fn digest(&self) -> &ArtifactDigest {
        &self.digest
    }

    /// Published or local, as this reference claims. Authoritative only when
    /// [`ArtifactRef::is_locally_minted`] is true.
    pub fn mode(&self) -> ArtifactMode {
        self.mode
    }

    /// How long this reference claims the artefact stays readable.
    pub fn retention(&self) -> ArtifactRetention {
        self.retention
    }

    /// Whether this reference was minted by a vault on this node.
    pub fn is_locally_minted(&self) -> bool {
        self.origin == RefOrigin::ThisVault
    }

    /// Whether two references address the same stored bytes, ignoring provenance
    /// and policy.
    ///
    /// This is the comparison an idempotent resolution wants (Phase Y): the same
    /// artefact reached twice, once from a local publish and once from a wire
    /// payload, is one artefact. `==` deliberately answers a different question —
    /// see the type doc.
    pub fn addresses_same_artefact(&self, other: &ArtifactRef) -> bool {
        self.digest == other.digest
    }
}

/// A read capability: the key material that turns stored ciphertext back into
/// plaintext. Holding it IS the read right.
///
/// Same three closures as [`crate::secret::BoundSecret`], for the same reason:
/// no `Serialize` (so it can never become a field of a journalled event, a
/// receipt or a snapshot), a redacting `Debug`, and zeroisation on drop.
///
/// The absence of `Serialize` is pinned HERE by a compile-fail doctest PAIR, and
/// nowhere else. It is not pinned by the ACL tests: none of them serialises a
/// capability, so an `impl Serialize for ReadCapability` that wrote the key would
/// leak it and leave every one of them green. Measured on this very module:
/// injecting that impl turns the doctest red while every runtime test stays
/// green.
///
/// The annotated error code is DOCUMENTATION, not a control — rustdoc does not
/// enforce it (measured, rustc 1.95.0, editions 2021 and 2024). The twin that
/// compiles is what stops the pair going green for a foreign reason, such as a
/// rename that breaks the import.
///
/// ```compile_fail,E0277
/// use sbfb_workflow_engine::artifact_acl::ReadCapability;
///
/// fn must_not_compile(cap: &ReadCapability) -> String {
///     serde_json::to_string(cap).unwrap() // ReadCapability: !Serialize
/// }
/// ```
///
/// ```
/// use sbfb_workflow_engine::artifact_acl::ReadCapability;
///
/// fn must_compile(cap: &ReadCapability) -> usize {
///     cap.expose().len()
/// }
/// ```
pub struct ReadCapability {
    key: zeroize::Zeroizing<Vec<u8>>,
}

impl ReadCapability {
    /// Wrap freshly-minted key material. The host sealer is the only producer.
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key: zeroize::Zeroizing::new(key),
        }
    }

    /// Borrow the key for the duration of one unseal.
    ///
    /// Same reservation as [`crate::secret::BoundSecret::expose`], and it matters
    /// more here because deletion is sold as crypto-shredding: this borrow does
    /// not outlive the capability, but it is NOT a guarantee that no owned copy
    /// exists — `.to_vec()` is one keystroke away, and a caller who did that can
    /// rebuild a working capability with [`ReadCapability::new`] after the
    /// original was destroyed. The type closes the SERIALISATION route by
    /// construction; not copying is the caller's discipline.
    pub fn expose(&self) -> &[u8] {
        &self.key
    }

    /// Pin the zeroising WRAPPER, not merely the behaviour: swapping
    /// `Zeroizing<Vec<u8>>` for a bare `Vec<u8>` compiles and keeps every test
    /// green, so without this the shredding claim could rot silently. The
    /// ANNOTATION at the call site is what holds it — `.len()` exists on both
    /// types, so calling this accessor and reading a length proves nothing.
    #[cfg(test)]
    pub(crate) fn key_is_zeroizing(&self) -> &zeroize::Zeroizing<Vec<u8>> {
        &self.key
    }
}

impl std::fmt::Debug for ReadCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReadCapability(<redacted {} bytes>)", self.key.len())
    }
}

/// Everything that can refuse an artefact operation. Every variant is a REFUSAL:
/// there is no degraded "returned the ciphertext anyway" outcome.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArtifactError {
    /// The artefact's retention has elapsed; the plaintext is not handed back.
    #[error("artefact {digest} expired at {at_ms} (now {now_ms})")]
    Expired {
        /// The digest that was addressed.
        digest: String,
        /// When it expired.
        at_ms: i64,
        /// The clock reading that decided.
        now_ms: i64,
    },
    /// Deletion was asked of a published artefact, which cannot honour it.
    #[error(
        "a published artefact cannot be deleted: its plaintext is served to any peer and copies may already exist"
    )]
    DeletionRefused,
    /// A read capability was needed and none was supplied.
    #[error("artefact {digest} is sealed and no read capability was supplied")]
    MissingCapability {
        /// The digest that was addressed.
        digest: String,
    },
    /// A local artefact was asked to leave the node.
    #[error("a local artefact may not leave the node: nothing distributes its key")]
    MayNotLeaveNode,
    /// A destructive operation was asked of a reference this vault did not mint.
    #[error(
        "refusing a destructive operation on a reference this vault did not mint: its mode is a caller-supplied claim"
    )]
    UntrustedReference {
        /// The digest that was addressed.
        digest: String,
    },
    /// A retention deadline was asked of a published artefact, which cannot
    /// honour it.
    #[error(
        "a published artefact cannot carry a retention deadline: its plaintext is served to any peer, and refusing to hand it back here would change nothing"
    )]
    RetentionOnPublished,
    /// The host sealer refused. Never carries plaintext — only the reason.
    #[error("sealing operation refused: {reason}")]
    SealerRefused {
        /// Why the host refused. Plaintext-free by contract.
        reason: String,
    },
    /// A digest received as text is not 64 lowercase hex characters.
    ///
    /// Refused rather than normalised: accepting uppercase would let two
    /// references to the same artefact compare unequal.
    #[error("artefact digest must be 64 lowercase hex characters, got {digest:?}")]
    MalformedDigest {
        /// A BOUNDED prefix of the text that was offered — never the whole
        /// input. This parses caller-supplied wire text, so echoing it verbatim
        /// would make the error as large as the payload (measured: 200 kB in,
        /// 200 kB of error out). Truncated inputs carry a trailing ellipsis.
        digest: String,
    },
    /// The stored bytes do not hash to the digest the reference claims, or the
    /// store holds nothing at that address.
    ///
    /// The two cases are deliberately INDISTINGUISHABLE — see
    /// [`ArtifactVault::read`] — so the message names both rather than
    /// asserting a corruption that may not have happened.
    #[error("stored bytes for {digest} are absent or do not match their content address")]
    DigestMismatch {
        /// The digest that was claimed.
        digest: String,
    },
}

/// Why a deletion was refused, and the capability the call did NOT consume.
///
/// [`ArtifactVault::delete`] takes the capability BY VALUE because destroying it
/// is the deletion. That makes every early return destructive unless the value
/// is handed back, which is what this type is for: a refused call must not
/// crypto-shred anything, least of all an unrelated artefact whose key was
/// offered by mistake.
///
/// No `PartialEq`: [`ReadCapability`] has none, on purpose — comparing key
/// material is not an operation this crate offers. Assert on `error`.
#[derive(Debug)]
pub struct DeleteRefusal {
    /// Why the deletion was refused.
    pub error: ArtifactError,
    /// The capability as supplied, untouched. `None` when the caller supplied
    /// none, or when the refusal is precisely that none was supplied.
    pub capability: Option<ReadCapability>,
}

impl std::fmt::Display for DeleteRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for DeleteRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// The host seam that seals and unseals artefact bytes.
///
/// It lives on the HOST side for a measured reason: this crate is keyless by
/// charter (`activity.rs`) and its dependency set contains no source of entropy
/// — no `rand`, no `getrandom`, no AEAD crate. A fresh nonce is mandatory for
/// every seal, so sealing cannot happen here without breaking the charter. The
/// real implementer is the daemon, over `nexus_core_rs::artifact_seal`.
///
/// On wiring: the daemon implements this trait, but no production route CALLS it
/// — no effect publishes an artefact yet.
pub trait ArtifactSealer {
    /// Seal `plaintext` for `mode`, returning the bytes to store and the
    /// capability that reads them back.
    ///
    /// # Implementer obligation — load-bearing for `G-ARTIFACT-ACL`
    ///
    /// For a mode whose [`ArtifactMode::requires_sealing`] is true, the returned
    /// bytes MUST NOT permit recovering `plaintext` without the returned
    /// capability. [`ArtifactVault::publish`] does NOT re-check: it stores what
    /// this method hands back. An implementer returning the plaintext unchanged
    /// is type-correct and would put a local artefact's cleartext into the store
    /// that `/blob-serve` and `BLOBS_ALPN` answer — measured with a conforming
    /// identity sealer, which published successfully and left the canary in the
    /// unauthenticated store.
    ///
    /// This is an obligation in PROSE, not in the type, and the difference is
    /// stated rather than hidden: closing it structurally needs the
    /// `StoredBytes` newtype mentioned on [`ArtifactVault`], whose only
    /// constructor would be the sealing path — the same newtype that would close
    /// the [`ArtifactBlobs::put`] door. A `stored != plaintext` check would not
    /// substitute for it: `plaintext ‖ padding` walks straight past. Today there
    /// is exactly one implementer (`nexus-shell-daemon`, real AES-256-GCM,
    /// exercised end to end), and §19.5 of the threat model carries the bound.
    ///
    /// The mode is a PARAMETER, not a property of the implementer. An earlier
    /// draft bound it at construction — one sealer per class — on the theory that
    /// a per-call value could drift from the caller's intent. The opposite
    /// happened, and it was measured: a vault holding a sealer bound to one class
    /// published an artefact of another, sealing under the wrong label while the
    /// reference announced the right one. Nothing failed at publish time, and a
    /// correctly-built sealer could never open those bytes — silent data loss.
    fn seal(
        &self,
        mode: ArtifactMode,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, ReadCapability), ArtifactError>;

    /// Recover plaintext from stored bytes using `capability`, under the mode the
    /// reference declares.
    ///
    /// # Implementer obligation — the THIRD seam door
    ///
    /// This method MUST authenticate `capability`: it must fail for any key that
    /// did not seal these bytes. The obligation is as load-bearing as the one on
    /// [`ArtifactSealer::seal`], and it was missing until external verification
    /// pointed it out — the threat model's bound covered `seal` only.
    ///
    /// Why it matters more than it looks: [`ArtifactVault::read`] delegates the
    /// READ RIGHT to this method, and [`ArtifactVault::delete`] uses it as the
    /// SOLE proof of possession. `ReadCapability::new` is public, so an
    /// implementer that ignores its `capability` argument — type-correct, and
    /// this repository already contains a compiled example of exactly that shape
    /// in a test double — lets an invented capability read and then delete a
    /// local artefact. Measured with such an implementer: an empty invented
    /// capability did both.
    ///
    /// Like the `seal` obligation this is PROSE, not a type. No newtype closes
    /// it: `StoredBytes` can make the store door structural, but nothing in the
    /// type system can prove that an external implementer actually checked a
    /// key. §19.5 carries it as a bound.
    ///
    /// Returns `Zeroizing` so the recovered plaintext is wiped when the caller
    /// drops it. The repository already closed this exact defect at P1 severity
    /// on the vault side: "ex-review P1-2: a bare `Vec<u8>` left credential
    /// material in freed heap" (`nexus-core-rs/src/vault.rs:1096-1098`).
    fn unseal(
        &self,
        mode: ArtifactMode,
        stored: &[u8],
        capability: &ReadCapability,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, ArtifactError>;
}

/// The host seam that stores and serves bytes by content address — the blob
/// store the two public paths of the module doc reach.
///
/// Its `get` takes no principal and no capability, on purpose: that is a
/// faithful model of what `/blob-serve` and `BLOBS_ALPN` actually offer. Making
/// this trait look authorised would hide the very property the phase must prove.
///
/// # Synchronous and infallible, over a client that is neither
///
/// The intended implementer is the daemon's `BlobsClient`, whose nine transfer
/// methods are all `async fn … -> Result` (`nexus-core-rs/src/blobs.rs`; the
/// tenth, `new`, is a plain wrapper). This seam is
/// synchronous, and two of its three methods have no error channel at all. That
/// is a deliberate simplification for a phase with no production implementer,
/// and it has three consequences the wiring phase inherits — written here
/// because they are cheap now and frozen the moment someone implements this:
///
/// 1. **A failed write cannot be reported.** `put` must either succeed or panic;
///    returning a fabricated digest would make [`ArtifactVault::publish`] hand
///    back an `Ok` for bytes the store rejected.
/// 2. **`untag` is best-effort**, so the `Ok(())` of [`ArtifactVault::delete`]
///    asserts the destruction of the KEY, never that a tag was successfully
///    dropped.
/// 3. **Blocking is the implementer's problem.** Bridging to the async client
///    needs `spawn_blocking` or a channel, never a naive `block_on` inside a
///    runtime thread.
///
/// Note on what is NOT a defect here: `get_unauthenticated` returning `Option`
/// collapses "absent" and "I/O error", but both surface as
/// [`ArtifactError::DigestMismatch`], so the read path fails CLOSED. It is not
/// the `Path::exists()` fail-open class that Phase L1 closed on the wipe path.
/// The fail-open risk in this trait is on the WRITE side, listed above.
pub trait ArtifactBlobs {
    /// Store bytes, returning their content address.
    fn put(&self, stored: &[u8]) -> ArtifactDigest;

    /// Fetch stored bytes by content address, with no authorisation whatsoever.
    fn get_unauthenticated(&self, digest: &ArtifactDigest) -> Option<Vec<u8>>;

    /// Drop the keep-online tag. Makes the blob GC-eligible; frees nothing now.
    fn untag(&self, digest: &ArtifactDigest);
}

/// The pure decision surface: may this operation proceed?
///
/// Pure and host-free so it can be exercised exhaustively without a daemon; the
/// STRUCTURAL half of the ACL (plaintext never reaching the public store) is
/// enforced by [`ArtifactVault::publish`], not by these predicates.
pub struct ArtifactPolicy;

impl ArtifactPolicy {
    /// May the artefact be read at `now_ms`? Only retention can refuse here —
    /// the read right itself is the capability, which the caller either holds or
    /// does not.
    pub fn authorise_read(reference: &ArtifactRef, now_ms: i64) -> Result<(), ArtifactError> {
        if let ArtifactRetention::ExpiresAtMs { at_ms } = reference.retention
            && reference.retention.expired_at(now_ms)
        {
            return Err(ArtifactError::Expired {
                digest: reference.digest.as_str().to_string(),
                at_ms,
                now_ms,
            });
        }
        Ok(())
    }

    /// May this artefact be deleted?
    ///
    /// Checks PROVENANCE first, then the mode, and it is the ONLY place either is
    /// checked: [`ArtifactVault::delete`] delegates its whole refusal logic here
    /// rather than repeating it. That is deliberate and was measured — with the
    /// provenance check written in both places, removing either copy alone left
    /// the entire suite green, so neither was pinned by anything. A guard that
    /// cannot go red is not a guard.
    ///
    /// Provenance comes first because it is the stronger claim: a reference this
    /// vault did not mint says nothing trustworthy about its own mode, so
    /// consulting the mode first would answer a question built on a caller's
    /// assertion.
    pub fn authorise_delete(reference: &ArtifactRef) -> Result<(), ArtifactError> {
        if !reference.is_locally_minted() {
            return Err(ArtifactError::UntrustedReference {
                digest: reference.digest.as_str().to_string(),
            });
        }
        if reference.mode.deletable() {
            Ok(())
        } else {
            Err(ArtifactError::DeletionRefused)
        }
    }

    /// May this artefact leave the node — export, hand-off, network send?
    ///
    /// No production caller yet: this is a door the wiring must walk through, not
    /// a barrier standing in a path. Stating it the other way would describe an
    /// enforcement no code performs.
    pub fn authorise_egress(reference: &ArtifactRef) -> Result<(), ArtifactError> {
        if reference.mode.may_leave_node() {
            Ok(())
        } else {
            Err(ArtifactError::MayNotLeaveNode)
        }
    }
}

/// Binds the two host seams and the clock into the operations a run performs.
///
/// This is where the STRUCTURAL half lives: [`ArtifactVault::publish`] is the
/// only door to the blob store THROUGH THIS TYPE, and it seals before `put` for
/// every local artefact.
///
/// Scope, stated precisely because an earlier wording ("there is no second
/// door") overclaimed: [`ArtifactBlobs`] is a public trait, re-exported at the
/// crate root, and its `put` takes arbitrary bytes. Any code holding a blobs
/// implementation can store plaintext and then build an [`ArtifactRef`] labelled
/// `Local` over it — [`ArtifactRef::from_untrusted_wire`] is public, so such a
/// reference exists. It arrives stamped [`RefOrigin::UntrustedWire`], which
/// [`ArtifactVault::delete`] refuses and [`ArtifactPolicy::authorise_egress`]
/// does not. What the vault guarantees is that
/// ITS OWN publishing path never does so. Making that structural rather than
/// conventional needs a `StoredBytes` newtype whose only constructor is the
/// sealing path (the idiom `DecidedRef` uses on the HITL side); until then it is
/// the caller's discipline, and §19.5 of the threat model carries it as a stated
/// bound.
pub struct ArtifactVault<'a> {
    sealer: &'a dyn ArtifactSealer,
    blobs: &'a dyn ArtifactBlobs,
    clock: &'a dyn Clock,
}

impl<'a> ArtifactVault<'a> {
    /// Build a vault over the host's seams.
    pub fn new(
        sealer: &'a dyn ArtifactSealer,
        blobs: &'a dyn ArtifactBlobs,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            sealer,
            blobs,
            clock,
        }
    }

    /// Publish `plaintext` under `mode`, sealing first when the mode demands it,
    /// and return the reference plus the capability that reads it back (`None`
    /// for a published artefact, which is stored in the clear).
    /// A retention deadline on a PUBLISHED artefact is refused rather than
    /// accepted and quietly not honoured: measured, the vault does stop handing
    /// the plaintext back after the deadline, while the unauthenticated store
    /// keeps serving it to every peer. Accepting the request would let a caller
    /// believe in an expiry the network ignores.
    pub fn publish(
        &self,
        mode: ArtifactMode,
        retention: ArtifactRetention,
        plaintext: &[u8],
    ) -> Result<(ArtifactRef, Option<ReadCapability>), ArtifactError> {
        if !mode.requires_sealing() && matches!(retention, ArtifactRetention::ExpiresAtMs { .. }) {
            return Err(ArtifactError::RetentionOnPublished);
        }

        let (stored, capability) = if mode.requires_sealing() {
            let (sealed, capability) = self.sealer.seal(mode, plaintext)?;
            (sealed, Some(capability))
        } else {
            (plaintext.to_vec(), None)
        };

        let digest = self.blobs.put(&stored);
        let reference = ArtifactRef {
            digest,
            mode,
            retention,
            origin: RefOrigin::ThisVault,
        };
        Ok((reference, capability))
    }

    /// Read an artefact back, enforcing retention then unsealing.
    ///
    /// The capability is required exactly when the artefact is sealed; supplying
    /// one for a published artefact is accepted and ignored, which keeps callers
    /// uniform.
    ///
    /// An ABSENT blob and a blob whose bytes do not hash to the claimed address
    /// both yield [`ArtifactError::DigestMismatch`], deliberately: telling them
    /// apart would turn this call into an existence oracle over a store that
    /// answers by content address, letting a caller enumerate which digests this
    /// node holds. The message names both cases for that reason.
    pub fn read(
        &self,
        reference: &ArtifactRef,
        capability: Option<&ReadCapability>,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, ArtifactError> {
        ArtifactPolicy::authorise_read(reference, self.clock.now_ms())?;

        let stored = self.blobs.get_unauthenticated(&reference.digest).ok_or(
            ArtifactError::DigestMismatch {
                digest: reference.digest.as_str().to_string(),
            },
        )?;

        if ArtifactDigest::of_stored_bytes(&stored) != reference.digest {
            return Err(ArtifactError::DigestMismatch {
                digest: reference.digest.as_str().to_string(),
            });
        }

        if reference.mode.requires_sealing() {
            let capability = capability.ok_or(ArtifactError::MissingCapability {
                digest: reference.digest.as_str().to_string(),
            })?;
            self.sealer.unseal(reference.mode, &stored, capability)
        } else {
            Ok(zeroize::Zeroizing::new(stored))
        }
    }

    /// Delete a LOCAL artefact: drop the tag and consume the capability, which
    /// zeroes the key on drop. That destruction is the whole guarantee — for a
    /// holder who did not copy the key out of the capability first
    /// ([`ReadCapability::expose`] hands out a borrow that `.to_vec()` turns into
    /// an owned copy, and [`ReadCapability::new`] would rebuild a working one).
    ///
    /// Refused for a published artefact, and refused for a local one whose
    /// capability is not supplied — this call cannot destroy a key it was not
    /// given, and reporting success would tell the caller their data is gone
    /// when the read right is still alive somewhere.
    ///
    /// The capability must OPEN the artefact, not merely exist: the check is an
    /// unseal, not `Option::is_some`. Measured on the previous revision —
    /// `ReadCapability::new` being public, `delete(local, Some(any 32 bytes))`
    /// untagged the blob, dropped an unrelated key and returned `Ok(())` while
    /// the real read right stayed alive and the artefact stayed readable. The
    /// destruction claim was false in exactly the case it was meant to cover.
    ///
    /// Refused outright for a reference this vault did not mint. Measured on the
    /// previous revision: re-labelling a `Published` digest as `Local` made this
    /// call untag the blob and return `Ok(())` while the store kept serving the
    /// plaintext — a destructive operation authorised by a caller-supplied label,
    /// reported as a success that destroyed nothing.
    ///
    /// # A refusal gives the capability back
    ///
    /// The success path CONSUMES the capability — that is the whole guarantee,
    /// under the COPY bound stated on the type and in bound 10 of §19.5.
    /// Every refusal hands it back through [`DeleteRefusal`], because taking it
    /// by value made refusals destructive: since the value is moved into this
    /// call, an early `?` dropped and zeroised it. Measured on the previous
    /// revision, where the possession check had just been added:
    /// `delete(&victim, Some(other_artefacts_key))` returned `Err` AND killed
    /// `other_artefacts_key` — crypto-shredding an unrelated artefact through a
    /// call that refused to do anything. `ReadCapability` is deliberately not
    /// `Clone`, so the caller had no way to keep a copy.
    ///
    /// One consequence stays and is worth naming: if the blob is absent from the
    /// store, the artefact cannot be deleted THROUGH THIS API (there is nothing
    /// to unseal against, so possession cannot be established). The capability
    /// comes back, so the caller can still crypto-shred by dropping it — which
    /// is the guarantee that matters — but the tag stays.
    pub fn delete(
        &self,
        reference: &ArtifactRef,
        capability: Option<ReadCapability>,
    ) -> Result<(), DeleteRefusal> {
        let digest = || reference.digest.as_str().to_string();

        // Provenance AND mode both come from `authorise_delete`, and are
        // deliberately NOT re-checked here. An earlier revision guarded
        // provenance in both places, meaning to add defence in depth; measured,
        // it did the opposite — removing EITHER guard alone left 19/19 green,
        // because the other still refused. Two guards for one condition are two
        // guards no test can see. One condition, one place, one test that can go
        // red.
        if let Err(error) = ArtifactPolicy::authorise_delete(reference) {
            return Err(DeleteRefusal { error, capability });
        }

        let Some(capability) = capability else {
            return Err(DeleteRefusal {
                error: ArtifactError::MissingCapability { digest: digest() },
                capability: None,
            });
        };

        // Possession, not presence — and possession OF THIS ARTEFACT, which takes
        // two checks, not one. Unsealing alone proves the key opens the bytes the
        // seam HANDED BACK, never that those bytes are the ones this reference
        // names. Measured by external verification (Codex r1): with the store
        // returning a third artefact's valid ciphertext under the victim's
        // digest, `delete(victim, other_key)` returned `Ok(())` and untagged the
        // victim while its real key still opened it — deletion theatre, from the
        // one call whose entire job is not to perform it.
        //
        // So the content address is re-verified first, exactly as `read` does. It
        // is the same two-line check, and it belongs on both paths for the same
        // reason: an unauthenticated store is a seam, not a source of truth.
        let Some(stored) = self.blobs.get_unauthenticated(&reference.digest) else {
            return Err(DeleteRefusal {
                error: ArtifactError::DigestMismatch { digest: digest() },
                capability: Some(capability),
            });
        };
        if ArtifactDigest::of_stored_bytes(&stored) != reference.digest {
            return Err(DeleteRefusal {
                error: ArtifactError::DigestMismatch { digest: digest() },
                capability: Some(capability),
            });
        }
        if let Err(error) = self.sealer.unseal(reference.mode, &stored, &capability) {
            return Err(DeleteRefusal {
                error,
                capability: Some(capability),
            });
        }

        self.blobs.untag(&reference.digest);
        drop(capability);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// A canary that must never appear in the public store for a local artefact.
    const CANARY: &str = "SBFB-ARTIFACT-CANARY-6f2a1c-do-not-leak";

    /// The mode → policy table, written out LITERALLY.
    ///
    /// `(mode, requires_sealing, may_leave_node, deletable)`.
    ///
    /// Why a literal table rather than the predicates themselves: the tests below
    /// used to phrase their expectation as `if mode.requires_sealing() { … }`,
    /// which cannot fail when the predicate is what changed. Measured on the
    /// four-class version of this module — a COHERENT mutation left 24/24 green,
    /// and a single-arm mutation left 10/10 green. An expectation derived from
    /// the thing under test is not an expectation.
    const POLICY: [(ArtifactMode, bool, bool, bool); 2] = [
        (ArtifactMode::Published, false, true, false),
        (ArtifactMode::Local, true, false, true),
    ];

    /// A deterministic stand-in for the host sealer: a repeating-key XOR with a
    /// counter-derived key. NO crypto, no entropy, no vault — the shape
    /// `ScriptedSecrets` takes for secrets. It proves the STRUCTURE (plaintext
    /// does not reach the store, the key is separable and destroyable), never the
    /// cryptography. Real AES-GCM is exercised by `nexus-shell-daemon`'s
    /// `artifact_sealer` tests.
    struct ScriptedSealer {
        next_key: RefCell<u8>,
        /// The mode of the most recent `seal` call, so a test can assert the
        /// vault handed down the mode it actually published under.
        last_sealed_mode: RefCell<Option<ArtifactMode>>,
    }

    impl ScriptedSealer {
        fn new() -> Self {
            Self {
                next_key: RefCell::new(1),
                last_sealed_mode: RefCell::new(None),
            }
        }
    }

    /// Mask that turns a scripted key byte into its authentication tag. Any
    /// non-zero value does; it exists so the tag is not the key byte itself.
    const TAG_MASK: u8 = 0xA5;

    impl ArtifactSealer for ScriptedSealer {
        fn seal(
            &self,
            mode: ArtifactMode,
            plaintext: &[u8],
        ) -> Result<(Vec<u8>, ReadCapability), ArtifactError> {
            *self.last_sealed_mode.borrow_mut() = Some(mode);
            let mut next = self.next_key.borrow_mut();
            let key_byte = *next;
            *next = next.wrapping_add(1);
            let key = vec![key_byte; 32];
            // One authentication byte, so `unseal` can REFUSE a capability that
            // does not open these bytes. A pure XOR double cannot fail, and a
            // double that cannot fail makes any test of POSSESSION vacuous — it
            // can only ever observe presence. This is the one AEAD property the
            // structural tests need modelled; the cryptography itself is proved
            // against real AES-256-GCM in `nexus-shell-daemon`.
            let mut sealed = vec![key_byte ^ TAG_MASK];
            sealed.extend(plaintext.iter().map(|b| b ^ key_byte));
            Ok((sealed, ReadCapability::new(key)))
        }

        fn unseal(
            &self,
            _mode: ArtifactMode,
            stored: &[u8],
            capability: &ReadCapability,
        ) -> Result<zeroize::Zeroizing<Vec<u8>>, ArtifactError> {
            let key_byte =
                *capability
                    .expose()
                    .first()
                    .ok_or_else(|| ArtifactError::SealerRefused {
                        reason: "empty capability".to_string(),
                    })?;
            let (tag, body) = stored
                .split_first()
                .ok_or_else(|| ArtifactError::SealerRefused {
                    reason: "stored bytes carry no tag".to_string(),
                })?;
            if *tag != key_byte ^ TAG_MASK {
                return Err(ArtifactError::SealerRefused {
                    reason: "capability does not open this artefact".to_string(),
                });
            }
            Ok(zeroize::Zeroizing::new(
                body.iter().map(|b| b ^ key_byte).collect(),
            ))
        }
    }

    /// An in-memory model of the blob store the two PUBLIC paths reach. Its
    /// `get_unauthenticated` is what `/blob-serve/{digest}` and a `BLOBS_ALPN`
    /// dialer obtain.
    #[derive(Default)]
    struct MemoryBlobs {
        stored: RefCell<BTreeMap<String, Vec<u8>>>,
        tags: RefCell<BTreeMap<String, bool>>,
    }

    impl ArtifactBlobs for MemoryBlobs {
        fn put(&self, stored: &[u8]) -> ArtifactDigest {
            let digest = ArtifactDigest::of_stored_bytes(stored);
            self.stored
                .borrow_mut()
                .insert(digest.as_str().to_string(), stored.to_vec());
            self.tags
                .borrow_mut()
                .insert(digest.as_str().to_string(), true);
            digest
        }

        fn get_unauthenticated(&self, digest: &ArtifactDigest) -> Option<Vec<u8>> {
            self.stored.borrow().get(digest.as_str()).cloned()
        }

        fn untag(&self, digest: &ArtifactDigest) {
            self.tags
                .borrow_mut()
                .insert(digest.as_str().to_string(), false);
        }
    }

    impl MemoryBlobs {
        /// Everything the public surface would hand out, concatenated — the
        /// haystack the canary search runs over.
        fn public_surface(&self) -> Vec<u8> {
            self.stored.borrow().values().flatten().copied().collect()
        }

        fn is_tagged(&self, digest: &ArtifactDigest) -> bool {
            *self.tags.borrow().get(digest.as_str()).unwrap_or(&false)
        }
    }

    fn contains(haystack: &[u8], needle: &str) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
    }

    /// The predicates agree with the literal table, mode by mode, flag by flag.
    #[test]
    fn mode_policy_matches_the_literal_table() {
        assert_eq!(
            POLICY.len(),
            ArtifactMode::ALL.len(),
            "the table must cover every mode"
        );
        // Equal length and membership are BOTH satisfied by a table that lists
        // one mode twice and another never — measured on the four-class version
        // of this module, where exactly that mutation left the suite green and
        // the only red came from an unrelated test's `.expect()`. Deduplicating
        // is what makes the message above true.
        let mut listed: Vec<ArtifactMode> = POLICY.iter().map(|(mode, ..)| *mode).collect();
        listed.sort();
        listed.dedup();
        assert_eq!(
            listed.len(),
            POLICY.len(),
            "the table lists a mode twice, so every loop over it silently skips another"
        );
        for (mode, sealing, egress, deletable) in POLICY {
            assert!(
                ArtifactMode::ALL.contains(&mode),
                "{mode:?} is in the table but not in ALL"
            );
            assert_eq!(
                mode.requires_sealing(),
                sealing,
                "requires_sealing {mode:?}"
            );
            assert_eq!(mode.may_leave_node(), egress, "may_leave_node {mode:?}");
            assert_eq!(mode.deletable(), deletable, "deletable {mode:?}");
        }
    }

    /// `ArtifactMode::ALL` lists every variant, exactly once — as far as Rust
    /// can be made to check it. See the doc on `ALL` for the residual bound.
    #[test]
    fn every_mode_is_listed_in_all_exactly_once() {
        // Exhaustive, no `_` arm: a new variant stops the build HERE, which is
        // the one line in the suite whose only subject is the contents of `ALL`.
        // The author then has to decide, deliberately, whether to list it.
        fn slot(mode: ArtifactMode) -> usize {
            match mode {
                ArtifactMode::Published => 0,
                ArtifactMode::Local => 1,
            }
        }

        let mut covered = [false; ArtifactMode::ALL.len()];
        for mode in ArtifactMode::ALL {
            let slot = slot(mode);
            assert!(!covered[slot], "{mode:?} is listed twice in ALL");
            covered[slot] = true;
        }
        assert!(
            covered.iter().all(|seen| *seen),
            "a mode has a slot but no entry in ALL, so every loop over ALL skips it"
        );
    }

    /// G-ARTIFACT-ACL, the cardinal invariant: knowing the digest of a local
    /// artefact does not permit reading it.
    ///
    /// Four arms, because fewer would prove less than they appear to:
    ///
    /// 1. the PUBLIC-PATH arm — fetching the digest through the unauthenticated
    ///    store (the `/blob-serve` model) yields bytes that do NOT contain the
    ///    plaintext. This is the arm that covers the path that actually matters;
    ///    a policy check alone would be a facade, since neither `/blob-serve` nor
    ///    `BLOBS_ALPN` runs this crate's code;
    /// 2. the FORGED-LABEL arm — a reference re-labelled `Published` over the same
    ///    digest still yields only ciphertext;
    /// 3. the CAPABILITY arm — without the key there is no read, with it there is;
    /// 4. the POSITIVE CONTROL — a published artefact carrying the same canary IS
    ///    found by the same search. Without it, arm 1 would pass just as well if
    ///    `contains` were broken, or if nothing had been stored at all.
    #[test]
    fn digest_is_not_read_right() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(1_000);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (local_ref, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish local");
        let capability = capability.expect("a local artefact yields a capability");

        // Arm 1 — the public path.
        let served = blobs
            .get_unauthenticated(&local_ref.digest)
            .expect("the store serves the digest to anyone");
        assert!(
            !contains(&served, CANARY),
            "plaintext of a local artefact reached the unauthenticated store"
        );
        assert!(
            !contains(&blobs.public_surface(), CANARY),
            "plaintext of a local artefact is somewhere in the public surface"
        );

        // Arm 2 — a forged label buys nothing, because the store holds ciphertext.
        let forged = ArtifactRef::from_untrusted_wire(
            local_ref.digest.clone(),
            ArtifactMode::Published,
            ArtifactRetention::Keep,
        );
        let via_forged = vault.read(&forged, None).expect("reads the stored bytes");
        assert!(
            !contains(&via_forged, CANARY),
            "re-labelling a sealed artefact as Published must not surface plaintext"
        );

        // Arm 3 — the capability IS the read right.
        assert_eq!(
            vault.read(&local_ref, None),
            Err(ArtifactError::MissingCapability {
                digest: local_ref.digest.as_str().to_string(),
            })
        );
        assert_eq!(
            &vault
                .read(&local_ref, Some(&capability))
                .expect("holder reads")[..],
            CANARY.as_bytes()
        );

        // Arm 4 — positive control.
        let (published_ref, none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish published");
        assert!(none.is_none(), "a published artefact carries no capability");
        let served_published = blobs
            .get_unauthenticated(&published_ref.digest)
            .expect("stored");
        assert!(
            contains(&served_published, CANARY),
            "positive control failed: the search cannot find plaintext it should find"
        );
    }

    /// Sealing happens for exactly the modes the TABLE says — and the check is
    /// made on the STORE's contents, not on the return value.
    #[test]
    fn encryption_enforced() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        for (mode, sealing, _egress, _deletable) in POLICY {
            let (reference, capability) = vault
                .publish(mode, ArtifactRetention::Keep, CANARY.as_bytes())
                .expect("publish");

            let served = blobs
                .get_unauthenticated(&reference.digest)
                .expect("stored");
            if sealing {
                assert!(
                    !contains(&served, CANARY),
                    "mode {mode:?} must be sealed but plaintext is in the store"
                );
                assert!(
                    capability.is_some(),
                    "mode {mode:?} must yield a capability"
                );
            } else {
                assert!(
                    contains(&served, CANARY),
                    "mode {mode:?} is stored in the clear; the model must show it"
                );
                assert!(
                    capability.is_none(),
                    "mode {mode:?} must not yield a capability"
                );
            }
        }
    }

    /// The mode the vault publishes under is the mode the sealer seals under.
    ///
    /// Regression guard for a measured defect: with the mode bound to the sealer
    /// instead of passed per call, the label and the reference could disagree,
    /// yielding bytes no correct sealer could ever open.
    #[test]
    fn seal_label_follows_the_published_mode() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        for (mode, sealing, _e, _d) in POLICY {
            let (reference, _cap) = vault
                .publish(mode, ArtifactRetention::Keep, b"payload")
                .expect("publish");
            assert_eq!(reference.mode, mode);
            if sealing {
                assert_eq!(
                    sealer.last_sealed_mode.borrow().expect("a seal happened"),
                    mode,
                    "the sealer was handed a mode other than the published one"
                );
            }
        }
    }

    /// Retention is enforced on the READ path, and the inclusive bound is pinned
    /// on BOTH sides so an off-by-one cannot pass.
    #[test]
    fn retention_enforced() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(1_000);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::ExpiresAtMs { at_ms: 2_000 },
                CANARY.as_bytes(),
            )
            .expect("publish");
        let capability = capability.expect("sealed");

        clock.set(1_999);
        assert_eq!(
            &vault
                .read(&reference, Some(&capability))
                .expect("readable one millisecond before the deadline")[..],
            CANARY.as_bytes()
        );

        clock.set(2_000);
        assert_eq!(
            vault.read(&reference, Some(&capability)),
            Err(ArtifactError::Expired {
                digest: reference.digest.as_str().to_string(),
                at_ms: 2_000,
                now_ms: 2_000,
            }),
            "the bound is inclusive: at_ms itself is already expired"
        );

        clock.set(2_001);
        assert!(matches!(
            vault.read(&reference, Some(&capability)),
            Err(ArtifactError::Expired { .. })
        ));
    }

    /// A published artefact refuses deletion, because it could not honour it —
    /// and the refusal hands back any capability it was given.
    ///
    /// Two arms, because `delete` has FOUR physical return sites but FIVE
    /// semantic refusals: `authorise_delete` aggregates `UntrustedReference` and
    /// `DeletionRefused` into one. External verification (Codex r3) measured that
    /// the second was the only one never exercised WITH a capability — this test
    /// passed `None`, so it could not observe restitution at all, and falsifying
    /// it on `DeletionRefused` alone left 22/22 green. Counting return sites
    /// instead of refusals is what hid it.
    #[test]
    fn published_artefact_refuses_deletion() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (published_ref, none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                b"broadcast",
            )
            .expect("publish");

        // Arm 1 — no capability at all.
        let refusal = vault
            .delete(&published_ref, none)
            .expect_err("a published artefact cannot honour a deletion");
        assert_eq!(refusal.error, ArtifactError::DeletionRefused);
        assert!(refusal.capability.is_none());
        assert!(
            blobs.is_tagged(&published_ref.digest),
            "a refused deletion must not have dropped the tag as a side effect"
        );

        // Arm 2 — a REAL capability, belonging to another artefact, must come
        // back able to open it. This is the fifth semantic refusal.
        let (bystander, bystander_cap) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let bystander_cap = bystander_cap.expect("sealed");

        let refusal = vault
            .delete(&published_ref, Some(bystander_cap))
            .expect_err("a published artefact cannot honour a deletion");
        assert_eq!(refusal.error, ArtifactError::DeletionRefused);
        let returned = refusal
            .capability
            .expect("a refusal hands the capability back rather than zeroising it");
        assert_eq!(
            &vault
                .read(&bystander, Some(&returned))
                .expect("the returned capability is the one that was handed in")[..],
            CANARY.as_bytes()
        );
    }

    /// Deleting a local artefact destroys its key; without that key the call is
    /// refused rather than reported as a success that did not happen.
    #[test]
    fn deleting_a_local_artefact_requires_its_key() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let capability = capability.expect("sealed");

        let refusal = vault
            .delete(&reference, None)
            .expect_err("a keyless deletion must be refused");
        // The SIXTH semantic refusal, and the last of the family: nothing was
        // supplied, so nothing may come back. Left unasserted until external
        // verification (Codex r4) measured that returning a fabricated
        // capability here kept the module green.
        assert!(
            refusal.capability.is_none(),
            "a refusal must not invent a capability the caller never supplied"
        );
        assert_eq!(
            refusal.error,
            ArtifactError::MissingCapability {
                digest: reference.digest.as_str().to_string(),
            }
        );
        assert!(
            blobs.is_tagged(&reference.digest),
            "a refused deletion must not drop the tag"
        );
        // And the artefact is indeed still readable, which is what makes the
        // refusal meaningful rather than pedantic.
        assert_eq!(
            &vault.read(&reference, Some(&capability)).expect("read")[..],
            CANARY.as_bytes()
        );

        vault.delete(&reference, Some(capability)).expect("delete");
        assert!(!blobs.is_tagged(&reference.digest));
    }

    /// The public-only floor: a LOCAL artefact never leaves the node, because
    /// nothing can distribute its key (scope cut 10). Both directions asserted.
    #[test]
    fn local_artefacts_may_not_leave_the_node() {
        for (mode, _sealing, may_leave, _deletable) in POLICY {
            let reference = ArtifactRef::from_untrusted_wire(
                ArtifactDigest::of_stored_bytes(b"x"),
                mode,
                ArtifactRetention::Keep,
            );
            let egress = ArtifactPolicy::authorise_egress(&reference);
            if may_leave {
                assert!(egress.is_ok(), "mode {mode:?} may leave the node");
            } else {
                assert_eq!(
                    egress,
                    Err(ArtifactError::MayNotLeaveNode),
                    "mode {mode:?} must refuse egress"
                );
            }
        }
    }

    /// Stored bytes that do not hash to the claimed address are refused — the
    /// content address is verified, which is the half of the digest that IS a
    /// right (verify-cap).
    #[test]
    fn tampered_storage_is_refused() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, _none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                b"original",
            )
            .expect("publish");

        blobs
            .stored
            .borrow_mut()
            .insert(reference.digest.as_str().to_string(), b"tampered".to_vec());

        assert_eq!(
            vault.read(&reference, None),
            Err(ArtifactError::DigestMismatch {
                digest: reference.digest.as_str().to_string(),
            })
        );
    }

    /// A digest the store holds NOTHING for is refused exactly like a mismatch.
    ///
    /// The indistinction is the design (see [`ArtifactVault::read`]). What this
    /// test pins, stated to the exact strength MEASURED rather than to the
    /// strength one would like: it pins the PROPERTY (an absent blob fails
    /// closed instead of yielding empty plaintext to a capability holder), NOT
    /// which of the two guards holds it. Measured: replacing the `ok_or` with
    /// `unwrap_or_default()` leaves the whole module green, because the content-address
    /// check immediately after refuses the empty vector anyway — the `ok_or` is
    /// DEFENCE IN DEPTH, kept for its message and for the day someone skips the
    /// re-hash on a hot path, not a uniquely load-bearing guard. Removing BOTH
    /// guards turns this test and `tampered_storage_is_refused` red, which is
    /// what makes the property pinned at all.
    #[test]
    fn reading_an_absent_digest_is_refused_like_a_mismatch() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let never_stored = ArtifactRef::from_untrusted_wire(
            ArtifactDigest::of_stored_bytes(b"nothing was ever published under this"),
            ArtifactMode::Published,
            ArtifactRetention::Keep,
        );

        assert_eq!(
            vault.read(&never_stored, None),
            Err(ArtifactError::DigestMismatch {
                digest: never_stored.digest.as_str().to_string(),
            }),
            "an absent blob must fail closed, not yield empty plaintext"
        );

        // Same refusal for a SEALED reference, so the absent case cannot be told
        // from the mismatch case by which error arrives first either.
        let sealed_but_absent = ArtifactRef::from_untrusted_wire(
            never_stored.digest.clone(),
            ArtifactMode::Local,
            ArtifactRetention::Keep,
        );
        assert_eq!(
            vault.read(
                &sealed_but_absent,
                Some(&ReadCapability::new(vec![7u8; 32]))
            ),
            Err(ArtifactError::DigestMismatch {
                digest: never_stored.digest.as_str().to_string(),
            })
        );
    }

    /// The pure decision surface refuses on PROVENANCE as well as on mode.
    ///
    /// [`ArtifactPolicy`] is `pub` and re-exported at the crate root, and
    /// [`ArtifactVault::delete`] delegates its whole refusal logic here — so this
    /// is the only place the two conditions live, and this test is what pins
    /// them. Measured on the revision that checked provenance in BOTH places:
    /// removing either copy alone left every test green, because the other still
    /// refused. That is why the duplicate was removed rather than kept "for
    /// safety".
    #[test]
    fn authorise_delete_refuses_provenance_and_mode() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (local, _cap) = vault
            .publish(ArtifactMode::Local, ArtifactRetention::Keep, b"payload")
            .expect("publish");
        let (published, _none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                b"broadcast",
            )
            .expect("publish");

        // Minted here, and the mode can honour a deletion.
        assert_eq!(ArtifactPolicy::authorise_delete(&local), Ok(()));

        // Minted here, but a published artefact cannot honour one.
        assert_eq!(
            ArtifactPolicy::authorise_delete(&published),
            Err(ArtifactError::DeletionRefused)
        );

        // Not minted here: refused before the mode is consulted at all, so a
        // caller cannot talk this surface into a yes by rebuilding a reference
        // with the mode it wants.
        let rebuilt = ArtifactRef::from_untrusted_wire(
            local.digest().clone(),
            ArtifactMode::Local,
            ArtifactRetention::Keep,
        );
        assert_eq!(
            ArtifactPolicy::authorise_delete(&rebuilt),
            Err(ArtifactError::UntrustedReference {
                digest: local.digest.as_str().to_string(),
            })
        );
    }

    /// A destructive call on a reference this vault did not mint is REFUSED.
    ///
    /// Found by a review probe, and it was reachable: re-labelling a `Published`
    /// digest as `Local` made `delete` untag the blob and return `Ok(())` while
    /// the store kept serving the plaintext — a success report over a destruction
    /// that did not happen. The mode of a caller-supplied reference is a claim,
    /// not authority; `RefOrigin` is the one field a caller cannot set.
    #[test]
    fn delete_refuses_a_reference_this_vault_did_not_mint() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (published, _none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                b"BROADCAST",
            )
            .expect("publish");

        // The capability offered is a REAL one, for a real artefact — so the
        // restitution below can be checked FUNCTIONALLY. Asserting `is_some()`
        // would not do it: measured by external verification (Codex r1),
        // destroying the received capability and returning a fabricated
        // `[0; 32]` in its place left the whole module green.
        let (bystander, bystander_cap) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let bystander_cap = bystander_cap.expect("sealed");

        let forged = ArtifactRef::from_untrusted_wire(
            published.digest.clone(),
            ArtifactMode::Local,
            ArtifactRetention::Keep,
        );
        let refusal = vault
            .delete(&forged, Some(bystander_cap))
            .expect_err("a reference this vault did not mint must be refused");
        assert_eq!(
            refusal.error,
            ArtifactError::UntrustedReference {
                digest: published.digest.as_str().to_string(),
            }
        );
        let returned = refusal
            .capability
            .expect("a refusal must hand the capability back rather than zeroise it");
        assert_eq!(
            &vault
                .read(&bystander, Some(&returned))
                .expect("the returned capability is the one that was handed in")[..],
            CANARY.as_bytes(),
            "the refusal returned SOMETHING, but not the caller's key"
        );
        assert!(
            blobs.is_tagged(&published.digest),
            "a forged reference must not have dropped the tag"
        );

        // A locally-minted LOCAL artefact still deletes normally, so the guard is
        // a provenance check and not a blanket refusal.
        let (local, capability) = vault
            .publish(ArtifactMode::Local, ArtifactRetention::Keep, b"payload")
            .expect("publish");
        assert!(local.is_locally_minted());
        vault.delete(&local, capability).expect("delete");
    }

    /// A retention deadline on a PUBLISHED artefact is refused at publication.
    ///
    /// Found by the same probe: the vault stopped handing the plaintext back
    /// after the deadline while the unauthenticated store kept serving it to
    /// every peer. Accepting the request would sell an expiry the network
    /// ignores.
    #[test]
    fn retention_on_a_published_artefact_is_refused() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        assert_eq!(
            vault
                .publish(
                    ArtifactMode::Published,
                    ArtifactRetention::ExpiresAtMs { at_ms: 10 },
                    b"EXPIRES-BUT-PUBLIC",
                )
                .err(),
            Some(ArtifactError::RetentionOnPublished)
        );

        // Nothing was stored: the refusal happens before `put`.
        assert!(
            blobs.public_surface().is_empty(),
            "a refused publication must not have stored anything"
        );

        // `Keep` on a published artefact stays legal, and a deadline on a LOCAL
        // one stays legal — the refusal is the impossible COMBINATION, not either
        // half of it.
        vault
            .publish(ArtifactMode::Published, ArtifactRetention::Keep, b"ok")
            .expect("published + Keep is legal");
        vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::ExpiresAtMs { at_ms: 10 },
                b"ok",
            )
            .expect("local + deadline is legal");
    }

    /// The capability's redacting `Debug` never prints key bytes, and the
    /// zeroising wrapper is part of the compiled contract.
    #[test]
    fn capability_redacts_and_zeroises() {
        let capability = ReadCapability::new(vec![0xAB; 32]);
        let rendered = format!("{capability:?}");
        assert_eq!(rendered, "ReadCapability(<redacted 32 bytes>)");

        // The ANNOTATION is the control, not the call: `.len()` exists on both
        // `Zeroizing<Vec<u8>>` and a bare `Vec<u8>`, so calling the accessor and
        // reading its length compiles under either — measured, that mutation left
        // the whole suite green. Binding the type is what stops compiling.
        let wrapper: &zeroize::Zeroizing<Vec<u8>> = capability.key_is_zeroizing();
        assert_eq!(wrapper.len(), 32);
    }

    /// A reference carries no key material, so serialising one — which is what
    /// journalling a run does — cannot leak a read right.
    ///
    /// The RED ARM is why this proves anything. The previous version searched for
    /// the key in HEX only, and was MEASURED non-discriminant: adding a
    /// `key_material: Vec<u8>` field fed from the capability left it green,
    /// because `serde_json` renders a byte vector as a DECIMAL array. Same shape
    /// as [`crate::secret`]'s canary oracle, recopied rather than shared — the
    /// two test modules cannot see each other's helpers.
    #[test]
    fn reference_serialisation_carries_no_key() {
        /// Which encoding of `needle` appears in `haystack`, if any.
        fn find_material(haystack: &str, needle: &[u8]) -> Option<&'static str> {
            if haystack.contains(&String::from_utf8_lossy(needle).to_string()) {
                return Some("raw text");
            }
            let hex: String = needle.iter().map(|b| format!("{b:02x}")).collect();
            if haystack.contains(&hex) {
                return Some("hex");
            }
            let decimal = needle
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            if haystack.contains(&decimal) {
                return Some("json decimal array");
            }
            None
        }

        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let capability = capability.expect("sealed");

        // ---- RED ARM: a reference that DOES carry the key must be detected ----
        #[derive(serde::Serialize)]
        struct LeakyRef<'a> {
            digest: &'a ArtifactDigest,
            key_material: &'a [u8],
        }
        let leaky = serde_json::to_string(&LeakyRef {
            digest: reference.digest(),
            key_material: capability.expose(),
        })
        .expect("serialises");
        assert_eq!(
            find_material(&leaky, capability.expose()),
            Some("json decimal array"),
            "RED ARM: a reference carrying the key must be DETECTED — otherwise \
             the green arm below proves nothing"
        );

        // ---- Green arm ----
        let json = serde_json::to_string(&reference).expect("reference serialises");
        assert_eq!(
            find_material(&json, capability.expose()),
            None,
            "the serialised reference must not contain the key: {json}"
        );
        assert!(!contains(json.as_bytes(), CANARY));
    }

    /// The serialised SHAPE of a reference, pinned literally.
    ///
    /// [`ArtifactRef`] is declared additive wire (AM-12) and Phase P carries it
    /// on the compute path, so "0 bump" has to be checkable against a form
    /// someone actually wrote down. Measured before this test existed: renaming
    /// `digest` and switching [`ArtifactRetention`] to camelCase changed the
    /// payload every peer would parse and left the whole crate green, 168/168.
    /// `origin` is absent from both vectors, which is the `#[serde(skip)]`.
    #[test]
    fn reference_wire_form_is_pinned() {
        let digest = ArtifactDigest::of_stored_bytes(b"pinned");
        let hex = digest.as_str().to_string();

        let local = ArtifactRef::from_untrusted_wire(
            digest.clone(),
            ArtifactMode::Local,
            ArtifactRetention::Keep,
        );
        assert_eq!(
            serde_json::to_string(&local).expect("serialises"),
            format!(r#"{{"digest":"{hex}","mode":"local","retention":{{"kind":"keep"}}}}"#)
        );

        let published = ArtifactRef::from_untrusted_wire(
            digest,
            ArtifactMode::Published,
            ArtifactRetention::ExpiresAtMs { at_ms: 42 },
        );
        assert_eq!(
            serde_json::to_string(&published).expect("serialises"),
            format!(
                r#"{{"digest":"{hex}","mode":"published","retention":{{"kind":"expires_at_ms","at_ms":42}}}}"#
            )
        );
    }

    /// A peer holding only the serialised form can rebuild the reference.
    ///
    /// This is what makes "additive wire" true rather than aspirational: without
    /// [`ArtifactDigest::from_hex`] there was NO path from the JSON back to a
    /// digest — no `Deserialize`, and `of_stored_bytes` wants the bytes a peer
    /// does not have. The refusal arms are where the constructor invariant this
    /// type cites to justify having no `Deserialize` is actually applied.
    #[test]
    fn a_digest_round_trips_through_its_hex_form() {
        let digest = ArtifactDigest::of_stored_bytes(b"payload");
        assert_eq!(
            ArtifactDigest::from_hex(digest.as_str()).expect("hex round-trips"),
            digest
        );

        // The path a peer actually walks: parse the payload, take the field,
        // rebuild the digest, rebuild the reference.
        let sent = ArtifactRef::from_untrusted_wire(
            digest.clone(),
            ArtifactMode::Local,
            ArtifactRetention::Keep,
        );
        let payload = serde_json::to_string(&sent).expect("serialises");
        let parsed: serde_json::Value = serde_json::from_str(&payload).expect("parses");
        let received = ArtifactDigest::from_hex(parsed["digest"].as_str().expect("digest field"))
            .expect("a peer rebuilds the digest it received");
        assert_eq!(received, digest);
        assert!(
            ArtifactRef::from_untrusted_wire(
                received,
                ArtifactMode::Local,
                ArtifactRetention::Keep
            )
            .addresses_same_artefact(&sent)
        );

        // One refusal per way the invariant breaks.
        let upper = digest.as_str().to_uppercase();
        let short = "a".repeat(63);
        let long = "a".repeat(65);
        let non_hex = format!("{}g", &digest.as_str()[..63]);
        let huge_ascii = "a".repeat(200_000);
        let huge_unicode = "é".repeat(100_000);
        for bad in [
            "",
            "abc",
            short.as_str(),
            long.as_str(),
            upper.as_str(),
            non_hex.as_str(),
            huge_ascii.as_str(),
            huge_unicode.as_str(),
        ] {
            assert!(
                matches!(
                    ArtifactDigest::from_hex(bad),
                    Err(ArtifactError::MalformedDigest { .. })
                ),
                "from_hex must refuse {bad:?}"
            );
        }

        // The echo is BOUNDED. Without this arm, reverting to `hex.to_string()`
        // left the module green (measured, Codex r3): the loop above only checks
        // WHICH error variant comes back, never how big it is. Both a long ASCII
        // input and a long multi-byte one, since the limit counts characters.
        for huge in [huge_ascii.as_str(), huge_unicode.as_str()] {
            let Err(ArtifactError::MalformedDigest { digest: echoed }) =
                ArtifactDigest::from_hex(huge)
            else {
                panic!("expected MalformedDigest");
            };
            assert!(
                echoed.chars().count() <= 73,
                "the error echoed {} characters of a {}-character input",
                echoed.chars().count(),
                huge.chars().count()
            );
            assert!(echoed.ends_with('…'), "a truncated echo must say so");
            // The prefix IDENTITY, not merely its length: a truncation that
            // echoed the wrong characters would satisfy the bound above.
            let expected: String = huge.chars().take(72).collect();
            assert_eq!(echoed, format!("{expected}…"));
        }
    }

    /// `delete` requires a capability that OPENS the artefact, not merely one
    /// that exists.
    ///
    /// Measured on the previous revision, where the guard was
    /// `capability.is_none()`: since `ReadCapability::new` is public, any 32
    /// bytes untagged the blob, dropped an unrelated key and returned `Ok(())`
    /// while the artefact stayed readable with its real key. `Shredded` was the
    /// one thing the call could not honestly claim.
    #[test]
    fn deleting_with_a_foreign_capability_is_refused() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (victim, victim_cap) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let victim_cap = victim_cap.expect("sealed");
        let (other, other_cap) = vault
            .publish(ArtifactMode::Local, ArtifactRetention::Keep, b"unrelated")
            .expect("publish");
        let other_cap = other_cap.expect("sealed");

        // An authentic capability — for a DIFFERENT artefact.
        let refusal = vault
            .delete(&victim, Some(other_cap))
            .expect_err("a capability that does not open this artefact must be refused");
        assert!(matches!(refusal.error, ArtifactError::SealerRefused { .. }));
        let other_cap = refusal
            .capability
            .expect("a refusal hands the capability back rather than zeroising it");

        // THE point, and the reason the previous assertion (tag only) was not
        // enough: refusing must not have crypto-shredded the UNRELATED artefact
        // whose key was offered by mistake. `ReadCapability` is not `Clone`, so
        // a consumed capability would be gone for good.
        assert_eq!(
            &vault
                .read(&other, Some(&other_cap))
                .expect("the unrelated artefact is still readable")[..],
            b"unrelated"
        );

        // An invented capability, refused the same way and likewise handed back.
        let refusal = vault
            .delete(&victim, Some(ReadCapability::new(vec![0u8; 32])))
            .expect_err("an invented capability must be refused");
        assert!(matches!(refusal.error, ArtifactError::SealerRefused { .. }));
        assert!(refusal.capability.is_some());
        assert!(
            blobs.is_tagged(&victim.digest),
            "a refused deletion must not drop the tag"
        );
        // The victim's own read right is still alive, so reporting a shredding
        // would have been false.
        assert_eq!(
            &vault
                .read(&victim, Some(&victim_cap))
                .expect("still readable")[..],
            CANARY.as_bytes()
        );

        // The real key still deletes — a possession check, not a blanket refusal.
        vault.delete(&victim, Some(victim_cap)).expect("delete");
        assert!(!blobs.is_tagged(&victim.digest));
        assert!(
            blobs.is_tagged(&other.digest),
            "an unrelated artefact must be untouched"
        );
    }

    /// `delete` verifies the CONTENT ADDRESS before unsealing, so a store that
    /// returns someone else's ciphertext cannot be talked into a deletion.
    ///
    /// Found by external verification (Codex r1) and it was reachable: `delete`
    /// unsealed whatever the seam handed back, so with a third artefact's valid
    /// ciphertext served under the victim's digest, `delete(victim, other_key)`
    /// returned `Ok(())` and untagged the victim — while the victim's real key
    /// still opened its real bytes. Unsealing proves the key opens THOSE BYTES;
    /// only the re-hash proves those bytes are the ones the reference names.
    /// `read` had this check from the start; `delete` did not.
    #[test]
    fn delete_refuses_bytes_that_do_not_match_the_address() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (victim, victim_cap) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let victim_cap = victim_cap.expect("sealed");
        let (other, other_cap) = vault
            .publish(ArtifactMode::Local, ArtifactRetention::Keep, b"unrelated")
            .expect("publish");
        let other_cap = other_cap.expect("sealed");

        // The seam misbehaves: the victim's address now serves the other
        // artefact's perfectly valid ciphertext.
        let other_bytes = blobs
            .get_unauthenticated(&other.digest)
            .expect("the other artefact is stored");
        let victim_bytes = blobs
            .stored
            .borrow_mut()
            .insert(victim.digest.as_str().to_string(), other_bytes)
            .expect("the victim was stored");

        // `other_cap` DOES open the bytes now served — and that must not be
        // enough, because they are not the victim's bytes.
        let refusal = vault
            .delete(&victim, Some(other_cap))
            .expect_err("bytes that do not hash to the claimed address must be refused");
        assert_eq!(
            refusal.error,
            ArtifactError::DigestMismatch {
                digest: victim.digest.as_str().to_string(),
            }
        );
        // FUNCTIONAL, like the three other refusal arms. This one was born WITH
        // the re-hash and was the only one left asserting `is_some()` — measured
        // by external verification (Codex r2): destroying the received key and
        // returning a fabricated `[0; 32]` on this branch alone kept all 22 tests
        // green. A new refusal branch is a new restitution to guard.
        let returned = refusal
            .capability
            .expect("a refusal hands the capability back rather than zeroising it");
        assert_eq!(
            &vault
                .read(&other, Some(&returned))
                .expect("the returned capability is the one that was handed in")[..],
            b"unrelated"
        );
        assert!(
            blobs.is_tagged(&victim.digest),
            "a refused deletion must not drop the tag"
        );

        // And the victim is genuinely intact: put its own bytes back and its own
        // key still opens it. Without this arm the refusal above could be
        // refusing a victim that was already destroyed.
        blobs
            .stored
            .borrow_mut()
            .insert(victim.digest.as_str().to_string(), victim_bytes);
        assert_eq!(
            &vault
                .read(&victim, Some(&victim_cap))
                .expect("the victim survived the refused deletion")[..],
            CANARY.as_bytes()
        );
    }

    /// A deletion refused because the blob is ABSENT hands the capability back,
    /// so the caller can still crypto-shred by dropping it.
    ///
    /// The honest consequence of checking possession by unsealing: with nothing
    /// to unseal against, this API cannot establish possession and therefore
    /// cannot delete. Naming it in a test rather than leaving it to be
    /// discovered — the guarantee that matters (destroying the key) stays
    /// available to the caller, only the tag survives.
    #[test]
    fn deleting_an_artefact_whose_blob_vanished_returns_the_capability() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish");
        let capability = capability.expect("sealed");

        let rescued = blobs
            .stored
            .borrow_mut()
            .remove(reference.digest.as_str())
            .expect("the blob was there");

        let refusal = vault
            .delete(&reference, Some(capability))
            .expect_err("possession cannot be established against an absent blob");
        assert_eq!(
            refusal.error,
            ArtifactError::DigestMismatch {
                digest: reference.digest.as_str().to_string(),
            }
        );
        let returned = refusal
            .capability
            .expect("the caller must keep the ability to shred by dropping the key");

        // FUNCTIONAL check, not `is_some()`: put the bytes back and require the
        // returned key to open them. Measured by external verification (Codex
        // r1) — destroying the received capability and returning a fabricated
        // `[0; 32]` on this very branch left the whole module green, because
        // nothing asked what the returned value could DO.
        blobs
            .stored
            .borrow_mut()
            .insert(reference.digest.as_str().to_string(), rescued);
        assert_eq!(
            &vault
                .read(&reference, Some(&returned))
                .expect("the returned capability is the one that was handed in")[..],
            CANARY.as_bytes()
        );
    }

    /// Changing what a minted reference SAYS costs it its provenance.
    ///
    /// The runtime half of the private-fields decision; the compile-time half is
    /// the `compile_fail,E0616` doctest on [`ArtifactRef::from_untrusted_wire`].
    /// Measured on the previous revision, where `mode` was `pub`: assigning to it
    /// on a reference `publish` had just returned kept `origin == ThisVault`, so
    /// `delete` untagged a PUBLISHED blob and returned `Ok(())` while the store
    /// went on serving the plaintext. Re-pointing `digest` the same way untagged
    /// an unrelated artefact — an arbitrary de-tag primitive.
    #[test]
    fn an_edited_reference_loses_its_provenance() {
        let sealer = ScriptedSealer::new();
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (published, _none) = vault
            .publish(
                ArtifactMode::Published,
                ArtifactRetention::Keep,
                b"BROADCAST",
            )
            .expect("publish");
        assert!(published.is_locally_minted());

        // The fields are private, so re-labelling means rebuilding — and a
        // rebuilt reference is untrusted by construction.
        let edited = ArtifactRef::from_untrusted_wire(
            published.digest().clone(),
            ArtifactMode::Local,
            published.retention(),
        );
        assert!(
            !edited.is_locally_minted(),
            "a rebuilt reference must not inherit provenance"
        );
        assert_eq!(
            vault
                .delete(&edited, Some(ReadCapability::new(vec![1u8; 32])))
                .expect_err("an edited reference is untrusted")
                .error,
            ArtifactError::UntrustedReference {
                digest: published.digest.as_str().to_string(),
            }
        );
        assert!(blobs.is_tagged(&published.digest));
        assert!(
            contains(&blobs.public_surface(), "BROADCAST"),
            "the store still serves it — which is what made the old bug a lie"
        );

        // Same artefact, different reference: `==` counts provenance, and the
        // digest-only comparison is the one Phase Y wants.
        assert_ne!(edited, published);
        assert!(edited.addresses_same_artefact(&published));
    }
}
