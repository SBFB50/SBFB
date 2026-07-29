// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host side of the workflow artefact ACL (Sprint 83 Phase M): the daemon
//! implements the engine's [`ArtifactSealer`] seam over the AES-GCM primitive of
//! `nexus_core_rs::artifact_seal`.
//!
//! This crate is where the two halves can meet at all: it is the ONLY crate that
//! depends on `sbfb-workflow-engine`, and `nexus-core-rs` must not gain a
//! dependency on the engine to keep the layering acyclic. So the primitive stays
//! in `nexus-core-rs` (it is plain crypto, useful without a workflow) and the
//! TRAIT IMPL — the part that knows about both — lives here.
//!
//! ## Wiring status, stated plainly
//!
//! Nothing calls this yet. No loopback route publishes an artefact, and the
//! engine's effect contexts carry no vault. This is the same honest position
//! `secret.rs` takes for `SecretResolver` (DS-1): the seam and its host
//! implementation exist and are tested, the CALL SITE is the work of the phase
//! that introduces artefact-producing effects. Saying otherwise here would
//! describe a route that is not written.
//!
//! ## What the mode label binds
//!
//! The engine's [`ArtifactMode`] is mapped to a STABLE string bound into the
//! AEAD's additional authenticated data, so bytes sealed as `local` cannot be
//! opened as `published`. Only `local` bytes are ever sealed — a published
//! artefact is stored in the clear — but the label is bound for both, so a future
//! mode cannot silently inherit an existing one's ciphertexts.

use nexus_core_rs::artifact_seal::{open_artifact, seal_artifact};
use sbfb_workflow_engine::artifact_acl::{
    ArtifactError, ArtifactMode, ArtifactSealer, ReadCapability,
};
use zeroize::Zeroizing;

/// The stable AAD label of an artefact mode.
///
/// Exhaustive `match`, never a `_` arm: adding a variant to [`ArtifactMode`]
/// must be a compile error here, not a silent re-use of an existing label.
///
/// `dead_code` is allowed for the reason the module doc states: this binary has
/// no artefact-publishing route yet, so nothing outside the tests reaches it.
/// The allow is the honest marker of that state — it comes off with the first
/// call site, and its presence is what makes the absence visible.
#[allow(dead_code)]
fn mode_label(mode: ArtifactMode) -> &'static str {
    match mode {
        ArtifactMode::Published => "published",
        ArtifactMode::Local => "local",
    }
}

/// Seals artefacts with a fresh per-artefact key, under the mode it is handed.
///
/// STATELESS on purpose. An earlier version bound one sealer to one class, so
/// that the AAD label could not "drift from the class the caller believes it is
/// publishing under". That reasoning was backwards and the review measured it: a
/// vault holding a sealer bound to one class published an artefact of another,
/// sealing under the wrong label while the reference announced the right one —
/// no error, and bytes a correctly-built sealer could never open. Taking the mode
/// per call makes the label and the reference one fact.
///
/// `dead_code` allowed: no publishing route constructs one yet (module doc).
#[allow(dead_code)]
pub struct HostArtifactSealer;

impl ArtifactSealer for HostArtifactSealer {
    fn seal(
        &self,
        mode: ArtifactMode,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, ReadCapability), ArtifactError> {
        let sealed = seal_artifact(mode_label(mode), plaintext).map_err(|e| {
            ArtifactError::SealerRefused {
                reason: e.to_string(),
            }
        })?;
        // The capability takes ownership of a copy; the `Zeroizing` original is
        // wiped when `sealed` drops at the end of this call.
        let capability = ReadCapability::new(sealed.key.to_vec());
        Ok((sealed.stored, capability))
    }

    fn unseal(
        &self,
        mode: ArtifactMode,
        stored: &[u8],
        capability: &ReadCapability,
    ) -> Result<Zeroizing<Vec<u8>>, ArtifactError> {
        open_artifact(mode_label(mode), capability.expose(), stored).map_err(|e| {
            ArtifactError::SealerRefused {
                reason: e.to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::artifact_seal::ARTIFACT_KEY_LEN;

    const CANARY: &str = "SBFB-HOST-SEALER-CANARY-3b8e0a";

    #[test]
    fn host_sealer_round_trips_through_the_engine_seam() {
        let sealer = HostArtifactSealer;
        let (stored, capability) = sealer
            .seal(ArtifactMode::Local, CANARY.as_bytes())
            .expect("seal");

        assert!(
            !stored.windows(CANARY.len()).any(|w| w == CANARY.as_bytes()),
            "the stored form must not contain the plaintext"
        );
        assert_eq!(capability.expose().len(), ARTIFACT_KEY_LEN);

        let opened = sealer
            .unseal(ArtifactMode::Local, &stored, &capability)
            .expect("unseal");
        assert_eq!(&opened[..], CANARY.as_bytes());
    }

    /// Opening under a DIFFERENT mode fails: the label is in the AAD.
    #[test]
    fn opening_under_another_mode_does_not_work() {
        let sealer = HostArtifactSealer;
        let (stored, capability) = sealer
            .seal(ArtifactMode::Local, CANARY.as_bytes())
            .expect("seal");

        let refused = sealer.unseal(ArtifactMode::Published, &stored, &capability);
        assert!(
            matches!(refused, Err(ArtifactError::SealerRefused { .. })),
            "mode re-labelling must fail closed, got {refused:?}"
        );
    }

    /// Every mode maps to a DISTINCT and STABLE label.
    ///
    /// Distinctness alone is not enough: swapping the two labels, or renaming
    /// one, keeps a dedup check green while silently changing what every stored
    /// artefact's AAD binds. The literal expectations below are the at-rest
    /// contract — changing one is a format break, not a refactor.
    #[test]
    fn mode_labels_are_distinct_and_stable() {
        const EXPECTED: [(ArtifactMode, &str); 2] = [
            (ArtifactMode::Published, "published"),
            (ArtifactMode::Local, "local"),
        ];
        for (mode, label) in EXPECTED {
            assert_eq!(mode_label(mode), label, "AAD label drifted for {mode:?}");
        }

        let mut labels: Vec<&str> = ArtifactMode::ALL.iter().map(|m| mode_label(*m)).collect();
        let before = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), before, "two modes share an AAD label");
        assert_eq!(before, EXPECTED.len(), "a mode is missing from the pinning");
    }

    /// The engine's vault, driven by the REAL host sealer rather than the
    /// scripted one, keeps plaintext out of the blob store AND carries the mode
    /// through BOTH arms of the seam.
    ///
    /// The round-1 fix put the mode into `seal`; nothing pinned the symmetric
    /// half. Measured on the previous revision: replacing the reference's class
    /// with a hard-coded one at the unseal call site left 18/18 green across both
    /// crates, because the scripted double ignores its class argument and no test
    /// drove a sealed artefact through the REAL sealer. Failure mode is silent
    /// data loss — an artefact nobody can ever reopen.
    #[test]
    fn local_artefact_round_trips_end_to_end_under_real_crypto() {
        use sbfb_workflow_engine::artifact_acl::{
            ArtifactBlobs, ArtifactDigest, ArtifactRetention, ArtifactVault,
        };
        use sbfb_workflow_engine::clock::ManualClock;
        use std::cell::RefCell;
        use std::collections::BTreeMap;

        #[derive(Default)]
        struct MemoryBlobs {
            stored: RefCell<BTreeMap<String, Vec<u8>>>,
        }
        impl ArtifactBlobs for MemoryBlobs {
            fn put(&self, stored: &[u8]) -> ArtifactDigest {
                let digest = ArtifactDigest::of_stored_bytes(stored);
                self.stored
                    .borrow_mut()
                    .insert(digest.as_str().to_string(), stored.to_vec());
                digest
            }
            fn get_unauthenticated(&self, digest: &ArtifactDigest) -> Option<Vec<u8>> {
                self.stored.borrow().get(digest.as_str()).cloned()
            }
            fn untag(&self, _digest: &ArtifactDigest) {}
        }

        let sealer = HostArtifactSealer;
        let blobs = MemoryBlobs::default();
        let clock = ManualClock::at(0);
        let vault = ArtifactVault::new(&sealer, &blobs, &clock);

        let (reference, capability) = vault
            .publish(
                ArtifactMode::Local,
                ArtifactRetention::Keep,
                CANARY.as_bytes(),
            )
            .expect("publish local");
        let capability = capability.expect("a local artefact yields a capability");
        assert_eq!(reference.mode(), ArtifactMode::Local);

        let served = blobs
            .get_unauthenticated(reference.digest())
            .expect("the store serves any digest");
        assert!(
            !served.windows(CANARY.len()).any(|w| w == CANARY.as_bytes()),
            "plaintext reached the unauthenticated store under real crypto"
        );

        assert_eq!(
            &vault
                .read(&reference, Some(&capability))
                .expect("a local artefact must reopen under its own mode")[..],
            CANARY.as_bytes()
        );
    }
}
