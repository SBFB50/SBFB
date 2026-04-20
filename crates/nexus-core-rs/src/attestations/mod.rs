// SPDX-License-Identifier: AGPL-3.0-or-later
//! Identity attestations for Sybil-resistance composition (Sprint 22).
//!
//! The composition has 3 layers arbitrated 2026-04-19 (plan §6 / kickoff
//! §4 D1) :
//!
//! - **Couche 1** — gossip mesh admission : [`AgeWitness`] peer-attests
//!   that a `node_id` was first seen at a given timestamp, enabling a
//!   ≥7-day age gate on top of the Sprint 19 Hashcash PoW already in
//!   place.
//! - **Couche 2** — governance-strong project curator admission :
//!   [`ContributorAttestation`] is an in-toto v1.0 predicate
//!   co-signed by the coordinator at verified-deploy time, asserting
//!   that a `node_id` successfully completed at least one verified-deploy
//!   for the subject project.
//! - **Couche 3** — multi-forge git-log cross-validation + external
//!   trust-web witnesses. Design-only S22 (cf.
//!   [`docs/security/CONTRIBUTOR_ATTESTATION_RFC.md`](
//!   ../../../docs/security/CONTRIBUTOR_ATTESTATION_RFC.md)),
//!   implementation distributed S23-S27.
//!
//! This module hosts the Rust crypto primitives for Couche 1 +
//! Couche 2. The Couche 3 `DelegationCert` is reserved (design-only)
//! and lands no code under `attestations/` in Sprint 22.
//!
//! Both attestations use RFC 8785 JCS canonical bytes + domain
//! separation via [`crate::canonical::canonical_bytes`]. Cross-stream
//! replay into `Task` / `Result` / `Claim` / `Invite` / `CuratorList`
//! / `Provenance` / `WarrantCanary` / `Pow` / `DuressAck` is
//! impossible by construction (each has its own domain tag).
//!
//! ## Matthew-effect caveat (LT-1)
//!
//! [`ContributorAttestation`] is binary — either a `node_id` is a
//! verified contributor for a project (1) or it is not (0). It does
//! **not** attest to fair distribution of contribution weight. The
//! Matthew effect reappears one layer deeper (high-kudos workers
//! publish more projects, earn more attestations). Fairness reform
//! lives in [`docs/release/ROADMAP_COMMITMENTS.md §LT-1`](
//! ../../../docs/release/ROADMAP_COMMITMENTS.md) scheduled
//! post-`v1.0`. Call-sites carry an inline TODO-LT-1 comment so the
//! follow-up is visible in the codebase.

pub mod age_witness;
pub mod contributor;

pub use age_witness::{
    AgeWitness, AgeWitnessError, MIN_AGE_DAYS, MIN_WITNESS_AGE_DAYS, SECONDS_PER_DAY,
};
pub use contributor::{
    ContributorAttestation, ContributorAttestationError, ContributorPredicate,
    CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE, CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE,
};
