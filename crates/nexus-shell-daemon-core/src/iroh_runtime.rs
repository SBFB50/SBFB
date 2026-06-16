// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phase C curator runtime — the crypto + storage layer the
//! shell-daemon uses to absorb signed curator lists announced
//! over gossip.
//!
//! The daemon binary wires this module into the iroh stack by:
//!
//! 1. Spawning a gossip `TopicReceiver` on
//!    [`CURATOR_TOPIC_SEED`] once the [`DaemonRuntime`] boot
//!    completes.
//! 2. On every [`nexus_core_rs::GossipEvent::Message`], handing
//!    the content bytes to [`CuratorRuntime::process_announcement_bytes`].
//! 3. That method parses the announcement JSON, checks the
//!    curator pubkey against the local attention set, fetches
//!    the referenced blob via
//!    [`nexus_core_rs::BlobsClient::fetch_ticket`], verifies the
//!    fetched [`CuratorListEntry`] (signature, attribution, cap,
//!    version), applies revision dedup, and stores the winner
//!    in a [`DashMap`] keyed by curator pubkey bytes.
//!
//! The runtime is **pure state + crypto**: it does not own an
//! iroh [`Node`], so unit tests can exercise it without any
//! network. The 2-node integration test
//! ([`tests::two_nodes_subscribe_and_fetch_curator_list`])
//! supplies a real `Node` pair to exercise the full fetch path,
//! but the binary's [`DaemonRuntime`] is the canonical caller.
//!
//! ## Attention set persistence (R7 mitigation)
//!
//! Sprint 7 plan §13 R7: a user who restarts the daemon must not
//! lose track of which curators they had subscribed to. The
//! runtime persists the curator pubkey set (hex-encoded) to
//! `<shell-daemon-dir>/subscriptions.json` atomically on every
//! subscribe / unsubscribe call, and re-loads it at boot. The
//! list **entries** themselves are RAM-only by design — they
//! re-arrive via gossip after a restart; a re-broadcast from
//! any live curator surfaces them within seconds.
//!
//! ## Gossip wire format
//!
//! Every gossip message sent on the curator topic is a JSON
//! object:
//!
//! ```json
//! { "v": 1, "curator": "<hex pubkey>", "ticket": "<blob ticket>" }
//! ```
//!
//! - `v` — announcement format version (currently 1).
//! - `curator` — lowercase hex of the curator's Ed25519 public
//!   key. The daemon uses this to drop announcements from
//!   curators that aren't in its attention set **before** it
//!   fetches anything, saving bandwidth.
//! - `ticket` — an `iroh_blobs::ticket::BlobTicket` string. The
//!   daemon fetches the referenced blob and parses it as a
//!   [`CuratorListEntry`].
//!
//! A valid announcement whose `curator` hex does NOT match the
//! fetched entry's `list.curator_pubkey` is rejected with a
//! warning — that's the attestation-level version of the
//! attribution split-brain mitigation [`CuratorListEntry::verify_signature`]
//! already applies at the crypto level.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use nexus_core_rs::blobs::BlobsClient;
use nexus_core_rs::crypto::PUBLIC_KEY_LENGTH;
use nexus_core_rs::{CuratorListEntry, Node, NodeDirectoryEntry, SignedList};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Seed string used to derive the SBFB curator list gossip
/// topic id. The topic is the first 32 bytes of
/// `blake3(CURATOR_TOPIC_SEED)`.
///
/// Sprint 7 D3 (kickoff §4) freezes this as a **single global
/// topic** — Sprint 8+ may add per-curator namespaced topics
/// (`"nexus-grid/curator/v1/<pubkey>"`), but v1 uses the global
/// one so a fresh daemon with no curators known yet still
/// receives announcements from every active curator in the
/// network.
pub const CURATOR_TOPIC_SEED: &[u8] = b"nexus-grid/curator/v1";

/// Announcement format version. Incremented on a wire-breaking
/// change to the gossip message JSON. The runtime rejects any
/// announcement whose `v` field is not equal to this.
pub const ANNOUNCEMENT_VERSION: u16 = 1;

/// Schema version for the `subscriptions.json` attention-set
/// persistence file. Independent from
/// [`ANNOUNCEMENT_VERSION`] — the former is a local disk layout,
/// the latter is an over-the-wire payload.
pub const SUBSCRIPTIONS_SCHEMA_VERSION: u32 = 1;

/// Compute the BLAKE3-derived 32-byte gossip topic id for the
/// curator list flow. Exposed as a function rather than a const
/// so tests can assert it against the literal seed + length
/// without depending on compile-time evaluation rules.
pub fn curator_topic_id() -> [u8; 32] {
    *blake3::hash(CURATOR_TOPIC_SEED).as_bytes()
}

// =================================================================
// Announcement wire format
// =================================================================

/// The JSON payload shell daemons broadcast on the curator
/// topic. See the module-level docs for the field contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuratorAnnouncement {
    /// Announcement version. Must equal [`ANNOUNCEMENT_VERSION`]
    /// to be accepted.
    #[serde(rename = "v")]
    pub version: u16,

    /// Lowercase hex of the curator's Ed25519 public key (64
    /// chars). Used for the cheap attention-set filter before
    /// fetching.
    #[serde(rename = "curator")]
    pub curator_pubkey_hex: String,

    /// `iroh_blobs::ticket::BlobTicket` string pointing at the
    /// signed `CuratorListEntry` JSON blob.
    #[serde(rename = "ticket")]
    pub blob_ticket: String,
}

impl CuratorAnnouncement {
    /// Construct a fresh announcement at the current version.
    pub fn new(curator_pubkey_bytes: [u8; PUBLIC_KEY_LENGTH], blob_ticket: String) -> Self {
        Self {
            version: ANNOUNCEMENT_VERSION,
            curator_pubkey_hex: hex::encode(curator_pubkey_bytes),
            blob_ticket,
        }
    }

    /// Serialize to a canonical JSON byte representation suitable
    /// for a gossip broadcast.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// The JSON payload a node broadcasts to announce its signed
/// [`nexus_core_rs::NodeDirectoryEntry`] blob (Sprint 75 Phase B).
///
/// Mirrors [`CuratorAnnouncement`] field-for-field but keys the
/// publisher under `"node"` (vs `"curator"`) so the receive-side
/// dispatch can tell a directory announcement apart from a curator-list
/// announcement by a clean parse, never a heuristic
/// ([`is_node_directory_announcement`]). The producer side (the `POST
/// /api/daemon/directory/publish` authoring route) landed in Phase B.
/// Sprint 75 Phase C wired the FULL ingest arm
/// ([`CuratorRuntime::process_directory_announcement_bytes`]): the gossip
/// dispatch now fetches + verifies (signature, attribution, revision floor) +
/// stores the referenced blob, subscription-gated on the same attention set as
/// curator lists. The clean directory-vs-curator discrimination keeps a
/// directory announcement from ever being mis-ingested as a curator list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeDirectoryAnnouncement {
    /// Announcement version. Must equal [`ANNOUNCEMENT_VERSION`] to be
    /// accepted.
    #[serde(rename = "v")]
    pub version: u16,

    /// Lowercase hex of the publishing node's Ed25519 public key (64
    /// chars). Used for the cheap attention-set filter before fetching.
    #[serde(rename = "node")]
    pub node_pubkey_hex: String,

    /// `iroh_blobs::ticket::BlobTicket` string pointing at the signed
    /// `NodeDirectoryEntry` JSON blob.
    #[serde(rename = "ticket")]
    pub blob_ticket: String,
}

impl NodeDirectoryAnnouncement {
    /// Construct a fresh announcement at the current version.
    pub fn new(node_pubkey_bytes: [u8; PUBLIC_KEY_LENGTH], blob_ticket: String) -> Self {
        Self {
            version: ANNOUNCEMENT_VERSION,
            node_pubkey_hex: hex::encode(node_pubkey_bytes),
            blob_ticket,
        }
    }

    /// Serialize to a canonical JSON byte representation suitable for a
    /// gossip broadcast.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Cheap discriminator for the gossip receive dispatch: is this PoW-unwrapped
/// payload UNAMBIGUOUSLY a [`NodeDirectoryAnnouncement`] (carries `node`) and NOT
/// a [`CuratorAnnouncement`] (carries `curator`)? serde ignores unknown fields,
/// so a hybrid `{v, node, curator, ticket}` parses as BOTH — requiring "parses
/// as directory AND does NOT parse as curator" keeps such a hybrid on the
/// curator path (its pre-Phase-B behaviour) instead of letting it silently
/// suppress a legitimate curator announcement. The dispatch uses this to drop a
/// pure directory announcement at `debug!` (its full ingest arm lands in
/// Phase C) instead of warn!-ing it through the curator arm. Mirrors the
/// `publish::is_*` partial-parse discriminators.
pub fn is_node_directory_announcement(payload: &[u8]) -> bool {
    serde_json::from_slice::<NodeDirectoryAnnouncement>(payload).is_ok()
        && serde_json::from_slice::<CuratorAnnouncement>(payload).is_err()
}

// =================================================================
// Subscriptions persistence
// =================================================================

/// On-disk shape of `<shell-daemon-dir>/subscriptions.json`.
///
/// Stores the set of curator pubkeys the user has subscribed
/// to, hex-encoded. The runtime rewrites this file atomically
/// on every subscribe / unsubscribe call and reloads it at
/// boot — see [`CuratorRuntime::load_subscriptions`] and
/// [`CuratorRuntime::persist_subscriptions`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionsFile {
    /// Always [`SUBSCRIPTIONS_SCHEMA_VERSION`]. A mismatch is
    /// treated the same as a missing file: the runtime logs a
    /// warning and starts with an empty attention set.
    pub schema_version: u32,

    /// The curator pubkeys the user is subscribed to, each 64
    /// lowercase hex chars. Serialized as an array rather than
    /// a set to keep the file format stable and diff-friendly.
    pub curators: Vec<String>,
}

/// Schema version for the `anchors.json` directory-locator persistence file
/// (Sprint 75 Phase C). Independent from [`SUBSCRIPTIONS_SCHEMA_VERSION`]: this
/// is a separate local disk layout.
pub const ANCHORS_SCHEMA_VERSION: u32 = 1;

/// One persisted anchor LOCATOR: where to re-fetch a subscribed anchor's signed
/// node directory after a reboot.
///
/// Sprint 75 Phase C (D4 durability, the F-Droid "fingerprint + repo URL"
/// shape): the node-directory ENTRIES are RAM-only (durably persisting remote
/// catalog content would invite over-count / staleness, D4). What persists is
/// only the **locator** — the anchor pubkey plus the last blob ticket we saw
/// advertise its directory — so the boot re-pull can actively re-fetch and
/// re-validate the catalog (signature + revision) instead of waiting for the
/// anchor to re-announce. A stale ticket address is tolerated: pkarr re-resolves
/// the node_id, and a re-fetched blob that fails verification or whose anchor is
/// offline simply yields nothing (the catalog reappears on the next live
/// announce). The persisted ticket is metadata about WHERE to re-fetch, never
/// the catalog itself.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorLocator {
    /// The anchor's Ed25519 public key, 64 lowercase hex chars. MUST also be in
    /// the attention set (`subscriptions.json`) for the boot re-pull to act on
    /// it — an unsubscribed locator is ignored (verrou 5: subscribed-only).
    pub pubkey: String,
    /// The most recent `iroh_blobs::ticket::BlobTicket` string seen announcing
    /// this anchor's `NodeDirectoryEntry` blob.
    pub ticket: String,
    /// The directory `revision` of the last entry we ingested from this anchor.
    /// Persisted so the boot re-pull can carry the rollback floor ACROSS a
    /// reboot: the RAM directory store starts empty, so without it the re-pull
    /// would accept any revision (the floor would be `None`). At re-pull the
    /// fetched blob must verify at a revision `>=` this value — the ticket pins
    /// an immutable content hash so a re-fetch already yields exactly this
    /// revision, but persisting it makes the anti-rollback guarantee explicit and
    /// reboot-durable rather than relying on content-addressing alone (Codex
    /// round-2 GAP). Mirrors F-Droid persisting the last index version next to
    /// the repo fingerprint.
    #[serde(default)]
    pub revision: u64,
}

/// On-disk shape of `<shell-daemon-dir>/anchors.json` — the set of anchor
/// directory locators to re-pull at boot. Mirrors [`SubscriptionsFile`]'s
/// schema-versioned, atomically-rewritten shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorsFile {
    /// Always [`ANCHORS_SCHEMA_VERSION`]. A mismatch is treated like a missing
    /// file: start with no locators (the catalogs re-arrive on the next live
    /// announce).
    pub schema_version: u32,
    /// The anchor locators, one per subscribed anchor whose directory we have
    /// ingested at least once.
    pub anchors: Vec<AnchorLocator>,
}

// =================================================================
// Errors
// =================================================================

/// Errors the curator runtime surfaces to callers. Individual
/// variants are matched by the binary's HTTP handlers to choose
/// status codes — most are 4xx because the inputs come from the
/// shell via the coordinator proxy.
#[derive(Debug, Error)]
pub enum CuratorRuntimeError {
    /// A subscribe / unsubscribe call was given a pubkey hex
    /// string that isn't 64 lowercase hex chars.
    #[error("invalid curator pubkey hex (expected 64 lowercase chars): {0}")]
    BadPubkeyHex(String),

    /// The gossip announcement JSON could not be parsed.
    #[error("gossip announcement parse failed: {0}")]
    AnnouncementParse(#[from] serde_json::Error),

    /// The gossip announcement version is unknown.
    #[error("unknown announcement version {got} (expected {expected})")]
    AnnouncementVersion { got: u16, expected: u16 },

    /// The announcement came from a curator the local attention
    /// set does not include. Expected in a healthy gossip network
    /// where many curators broadcast in parallel — the runtime
    /// drops these silently before any blob fetch so they do not
    /// waste bandwidth.
    ///
    /// Sprint 8 split (audit finding C-2): this variant replaces
    /// one half of the former `AnnouncementAttributionMismatch`.
    /// Splitting it lets the daemon's gossip handler log non-
    /// subscribed drops at `debug` and genuine envelope-vs-entry
    /// mismatches at `warn`, so a flood of routine drops does
    /// not drown out an actual spoofing attempt.
    #[error("dropped announcement from non-subscribed curator {curator}")]
    NotSubscribed { curator: String },

    /// The curator pubkey declared in the announcement envelope
    /// does not match the one inside the fetched entry — this is
    /// the gossip-layer equivalent of the attribution split-brain
    /// mitigation already enforced by
    /// [`CuratorListEntry::verify_signature`]. A peer reaching
    /// this branch has stapled a legitimately-signed list to a
    /// different pubkey, which is always a bug or a spoof
    /// attempt; the daemon's gossip handler logs it at `warn`
    /// with both hexes so the operator can act.
    ///
    /// Sprint 8 split (audit finding C-2): this variant replaces
    /// the other half of the former
    /// `AnnouncementAttributionMismatch`.
    #[error("announcement curator_pubkey {announcement} does not match fetched entry {entry}")]
    EnvelopeMismatch { announcement: String, entry: String },

    /// The fetched blob was not a valid [`CuratorListEntry`] in
    /// JSON form.
    #[error("curator list entry parse failed: {0}")]
    EntryParse(serde_json::Error),

    /// Verification of the fetched entry failed (bad version,
    /// oversized entries, attribution split-brain, tampered
    /// payload, wrong signer).
    #[error("curator list entry verify failed: {0}")]
    EntryVerify(nexus_core_rs::NexusError),

    /// The new entry's revision is not strictly greater than
    /// the one currently stored for this curator — rollback
    /// protection per Sprint 7 plan R6.
    #[error("revision rollback rejected: new revision {new} <= stored revision {stored}")]
    RevisionRollback { new: u64, stored: u64 },

    /// The blob fetch itself failed (network, peer unreachable,
    /// content hash mismatch, ...).
    #[error("blob fetch failed: {0}")]
    BlobFetch(nexus_core_rs::NexusError),

    /// Reading / writing `subscriptions.json` failed.
    #[error("subscriptions persistence failed on {path}: {source}")]
    Persistence {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// =================================================================
// Shared signed-list ingest gate (Sprint 75 Phase B)
// =================================================================

/// The outcome of the type-agnostic ingest gate
/// [`verify_signed_list_ingest`]. Each variant maps 1:1 to the
/// equivalent [`CuratorRuntimeError`] variant via the [`From`] impl
/// below, so refactoring the curator arm onto the shared gate preserves
/// its exact error surface — guarded end-to-end by the networked
/// `two_nodes_reject_*` tests; the shared gate's type-symmetry across
/// curator + node directory is guarded by `generic_ingest_helper_parity`.
#[derive(Debug)]
pub enum SignedListIngestError {
    /// Full signature verification failed (bad version, oversized
    /// payload, attribution split-brain inside the payload, tampered
    /// bytes, wrong signer).
    Verify(nexus_core_rs::NexusError),

    /// The pubkey declared in the gossip announcement does not match the
    /// signer pubkey inside the fetched, signed entry — a peer stapled a
    /// legitimately-signed list to a different pubkey.
    EnvelopeMismatch {
        /// Hex of the announcement's declared pubkey.
        announced: String,
        /// Hex of the fetched entry's actual signer.
        entry: String,
    },

    /// The new entry's revision is not strictly greater than the stored
    /// one — rollback protection.
    RevisionRollback {
        /// The rejected (non-greater) revision.
        new: u64,
        /// The currently stored revision.
        stored: u64,
    },
}

impl From<SignedListIngestError> for CuratorRuntimeError {
    fn from(e: SignedListIngestError) -> Self {
        match e {
            SignedListIngestError::Verify(err) => CuratorRuntimeError::EntryVerify(err),
            SignedListIngestError::EnvelopeMismatch { announced, entry } => {
                CuratorRuntimeError::EnvelopeMismatch {
                    announcement: announced,
                    entry,
                }
            }
            SignedListIngestError::RevisionRollback { new, stored } => {
                CuratorRuntimeError::RevisionRollback { new, stored }
            }
        }
    }
}

/// The three type-agnostic ingest checks every [`SignedList`] gossip
/// arm shares — factored out so a fix to one arm can never silently
/// skip the other (drift risk R1, design_review C1).
///
/// Given a fetched, parsed `entry`, the pubkey declared in the gossip
/// `announced_pubkey`, and the `stored_revision` currently held for that
/// publisher (or `None` if nothing is stored yet), this runs:
///
/// - **Step 6 — signature verification** ([`SignedList::verify`]):
///   version, caps, in-payload attribution, Ed25519 signature.
/// - **Step 7 — envelope cross-check**: the announcement's declared
///   pubkey MUST equal the entry's signer ([`SignedList::signer_pubkey`]).
/// - **Step 8 — revision rollback protection**: the entry's revision
///   MUST be strictly greater than `stored_revision`.
///
/// Returns `Ok(())` when the entry should be stored. Blob fetch
/// (step 5) and storage (step 9) stay in the per-type arm because they
/// touch type-specific transport + state.
pub fn verify_signed_list_ingest<T: SignedList>(
    entry: &T,
    announced_pubkey: &[u8; PUBLIC_KEY_LENGTH],
    stored_revision: Option<u64>,
) -> Result<(), SignedListIngestError> {
    // Step 6: full signature verification.
    entry.verify().map_err(SignedListIngestError::Verify)?;

    // Step 7: cross-check the announcement pubkey against the signer.
    let signer = entry.signer_pubkey();
    if signer != *announced_pubkey {
        return Err(SignedListIngestError::EnvelopeMismatch {
            announced: hex::encode(announced_pubkey),
            entry: hex::encode(signer),
        });
    }

    // Step 8: revision rollback protection.
    if let Some(stored) = stored_revision {
        if entry.list_revision() <= stored {
            return Err(SignedListIngestError::RevisionRollback {
                new: entry.list_revision(),
                stored,
            });
        }
    }

    Ok(())
}

// =================================================================
// CuratorRuntime
// =================================================================

/// Maximum number of concurrent `process_announcement_bytes`
/// calls the runtime allows before applying backpressure. Sprint 9
/// Phase E (C-4 close): without this guard, a gossip flood of 10 k
/// announcements/s serialises through a single fetch chain.
pub const MAX_INFLIGHT_ANNOUNCEMENTS: usize = 32;

/// Per-anchor timeout for the boot directory re-pull (Sprint 75 Phase C). Bounds
/// the worst case so a single dead/slow anchor cannot stall the gossip loop's
/// startup: a re-fetch that does not complete in this window is abandoned for
/// that anchor (its catalog reappears on the next live announce).
const REPULL_FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// UX-ARRIVAL (post-S75): hard cap on resident OBSERVED directory publishers
/// (non-subscribed nodes whose announcement arrived inside a PoW-valid gossip
/// envelope). Bounds the registry in SPACE against a burst of distinct Sybil
/// identities arriving faster than the TTL purge; when full, a fresher
/// newcomer evicts the stalest resident, a staler one is dropped (the
/// SeedRegistry SEED-2 policy).
///
/// Accepted residual (review UX-OBS-RATELIMIT-UNAUTH, honest framing): the
/// announcement's `node` field is an UNAUTHENTICATED claim and the PoW
/// envelope is bound to `(publisher, topic)`, NOT to the payload — so one
/// solved PoW can cover many announcements naming distinct forged pubkeys.
/// The per-node rate limit throttles each claimed identity's churn, and this
/// cap bounds the resident SIZE, but neither prices distinct forged
/// identities individually: a determined flood can fill the registry and
/// displace honest hints (same class as the SeedRegistry fresh-flood
/// residual, THREAT_MODEL §15.1). Impact is the visibility of a non-
/// authoritative HINT only — the catalog of an observed node is never
/// fetched, the subscribe CTA stays an explicit user action, and a forged
/// pubkey yields nothing but an honest "waiting for first announcement" row.
/// Binding the capture to the PoW publisher identity is routed to the S76
/// audit alongside the duress-sibling lot.
pub const MAX_OBSERVED_DIRECTORIES: usize = 256;

/// UX-ARRIVAL: an observed publisher not re-heard within this window drops out
/// of the registry (lazy purge on write + read). Same freshness horizon as the
/// SeedRegistry: a hint older than ~2 days has no arrival-screen value.
pub const OBSERVED_SEEN_TTL_SECS: u64 = 48 * 60 * 60;

/// UX-ARRIVAL: per-node ingest rate limit (PO requirement). One accepted
/// `last_seen` refresh per CLAIMED node identity per this window — a
/// re-publish spam loop on one identity cannot churn the registry (or the
/// `/nodes` payload) faster than this. It does NOT price distinct forged
/// identities (see the [`MAX_OBSERVED_DIRECTORIES`] residual note).
pub const OBSERVED_REFRESH_MIN_SECS: u64 = 60;

/// Local receive clock (unix secs) for the observed-directory registry.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Inner storage — two hashmaps, a semaphore and a file path.
/// Kept in a single struct so the public runtime handle can hand
/// out an `Arc<CuratorRuntime>` cheaply (DashMap is already Arc
/// inside).
#[derive(Debug)]
pub struct CuratorRuntime {
    /// Known curator lists keyed by curator pubkey bytes. The
    /// value is the most recent verified [`CuratorListEntry`].
    /// Revision-based dedup (plan R6) is enforced on insert.
    lists: DashMap<[u8; PUBLIC_KEY_LENGTH], CuratorListEntry>,

    /// Attention set: the curator pubkeys the user has chosen
    /// to subscribe to. Announcements from curators not in this
    /// set are silently dropped before any blob fetch happens
    /// so a random public announcer cannot waste the daemon's
    /// network budget.
    ///
    /// The value in the map is meaningless (`()` would require
    /// a second type parameter); we use `()` via a unit marker
    /// tuple and rely on `DashMap` as a concurrent set.
    attention: DashMap<[u8; PUBLIC_KEY_LENGTH], ()>,

    /// Path to the `subscriptions.json` attention-set persistence
    /// file. `None` disables persistence — used by the unit
    /// tests that don't want to touch disk.
    persistence_path: Option<PathBuf>,

    /// Sprint 9 Phase E (C-4 close): backpressure semaphore
    /// limiting concurrent `process_announcement_bytes` calls so a
    /// gossip flood cannot saturate the fetch pipeline unboundedly.
    announcement_semaphore: Semaphore,

    /// Sprint 75 Phase C: verified node directories, keyed by the publishing
    /// node's Ed25519 pubkey bytes. The value is the most recent verified
    /// [`NodeDirectoryEntry`] for that anchor (revision rollback protection
    /// applies on insert, via the shared `verify_signed_list_ingest` gate). A
    /// directory is gated by the SAME attention set as curator lists
    /// (`attention`): a node directory is signed by the node's keypair = the same
    /// key family as a curator pubkey, so one subscription covers both. RAM-only
    /// by design (D4) — re-pulled at boot from the persisted [`AnchorLocator`]s.
    directories: DashMap<[u8; PUBLIC_KEY_LENGTH], NodeDirectoryEntry>,

    /// Sprint 75 Phase C: the persisted re-fetch locator per anchor — the last
    /// blob `(ticket, revision)` seen advertising each anchor's directory.
    /// Populated on every successful directory ingest and rewritten to
    /// `anchors.json`. The boot re-pull iterates the SUBSCRIBED subset of these to
    /// re-fetch + re-validate the catalogs (D4: persist the locator, never the
    /// catalog content). The `revision` carries the anti-rollback floor across a
    /// reboot (the RAM `directories` map starts empty).
    anchor_locators: DashMap<[u8; PUBLIC_KEY_LENGTH], (String, u64)>,

    /// Path to the `anchors.json` locator persistence file. Derived from
    /// [`Self::persistence_path`] (the `subscriptions.json` sibling); `None`
    /// disables anchor persistence (the unit tests that do not touch disk).
    anchors_path: Option<PathBuf>,

    /// UX-ARRIVAL (post-S75): NON-subscribed directory publishers heard on
    /// gossip, keyed by node pubkey → `last_seen` (LOCAL receive clock, unix
    /// secs). Cheap-envelope METADATA only — the announcement's blob is NEVER
    /// fetched for a non-subscribed node (THREAT_MODEL §15.1: an unsolicited
    /// announcement must never trigger an outbound fetch/dial — the BitTorrent
    /// DRDoS / libp2p "don't store what you didn't ask for" lesson), so there
    /// is no `revision` / `app_count` here and the identity is PoW-backed, not
    /// Ed25519-verified. RAM-only by design: a hint with no freshness window
    /// has no arrival-screen value. Bounded IN the primitive
    /// ([`Self::record_observed_directory`]): cap + stalest eviction + TTL +
    /// per-node rate limit, never caller conventions (§P59.2). A `Mutex` (the
    /// SeedRegistry pattern), not a DashMap sibling: cap-check + eviction +
    /// insert must be one atomic step or two concurrent inserts overshoot the
    /// cap. Mutually exclusive with `directories` by the subscription gate;
    /// [`Self::subscribe`] purges the entry on the observed→subscribed
    /// transition.
    observed: Mutex<HashMap<[u8; PUBLIC_KEY_LENGTH], u64>>,
}

impl CuratorRuntime {
    /// Create a new runtime with no known lists and no
    /// subscriptions.
    ///
    /// If `persistence_path` is `Some`, the runtime will write
    /// the attention set to that file on every subscribe /
    /// unsubscribe call and read it back from
    /// [`Self::load_subscriptions`].
    pub fn new(persistence_path: Option<PathBuf>) -> Self {
        // The anchors locator file lives next to subscriptions.json.
        let anchors_path = persistence_path
            .as_ref()
            .map(|p| p.with_file_name("anchors.json"));
        Self {
            lists: DashMap::new(),
            attention: DashMap::new(),
            persistence_path,
            announcement_semaphore: Semaphore::new(MAX_INFLIGHT_ANNOUNCEMENTS),
            directories: DashMap::new(),
            anchor_locators: DashMap::new(),
            anchors_path,
            observed: Mutex::new(HashMap::new()),
        }
    }

    /// Build a runtime and immediately pre-populate the
    /// attention set from a [`SubscriptionsFile`] on disk. A
    /// missing, unreadable, or schema-mismatched file is treated
    /// as "start empty" and logged at warn level.
    pub fn with_persistence(persistence_path: PathBuf) -> Self {
        let runtime = Self::new(Some(persistence_path.clone()));
        runtime.load_subscriptions();
        // Sprint 75 Phase C: restore the anchor locators so the boot re-pull
        // (`repull_directories`) can re-fetch each subscribed anchor's catalog.
        runtime.load_anchors();
        runtime
    }

    // ---------------------------------------------------------
    // Attention set management
    // ---------------------------------------------------------

    /// Add a curator pubkey to the attention set. Idempotent:
    /// subscribing to an already-subscribed curator is a no-op.
    /// Persists the set to disk if a persistence path is set.
    ///
    /// Sprint 9 Phase E (D-3 close): persist-first rewrite. The
    /// attention DashMap is updated BEFORE persisting to disk so
    /// `persist_subscriptions` sees the new key, but if the persist
    /// fails the key is rolled back from RAM. This guarantees
    /// RAM and disk never diverge: a failed persist means the
    /// subscription is not accepted (the caller gets an error),
    /// and a successful persist means the next boot will find it.
    pub fn subscribe(
        &self,
        pubkey_hex: &str,
    ) -> Result<[u8; PUBLIC_KEY_LENGTH], CuratorRuntimeError> {
        let pubkey = parse_pubkey_hex(pubkey_hex)?;
        // Insert first so persist_subscriptions sees the new key.
        self.attention.insert(pubkey, ());
        if let Err(e) = self.persist_subscriptions() {
            // Rollback RAM on persist failure.
            self.attention.remove(&pubkey);
            return Err(e);
        }
        // UX-ARRIVAL: observed→subscribed transition — the node now belongs to
        // the attention set, so its "heard but not followed" hint is retired
        // (the two stores are mutually exclusive by the subscription gate).
        self.observed
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&pubkey);
        info!(curator = %pubkey_hex, "subscribed to curator");
        Ok(pubkey)
    }

    /// Remove a curator pubkey from the attention set. Also
    /// evicts any stored list from that curator so the shell
    /// stops showing its projects. Persists the set to disk.
    ///
    /// Sprint 9 audit I3-F1 fix: persist-first pattern — remove
    /// from RAM only after the disk write succeeds, so a disk
    /// failure does not leave RAM and disk diverged.
    pub fn unsubscribe(
        &self,
        pubkey_hex: &str,
    ) -> Result<[u8; PUBLIC_KEY_LENGTH], CuratorRuntimeError> {
        let pubkey = parse_pubkey_hex(pubkey_hex)?;
        // Save current values for rollback.
        let had_attention = self.attention.remove(&pubkey).is_some();
        let saved_list = self.lists.remove(&pubkey);
        // Persist the new state to disk.
        if let Err(e) = self.persist_subscriptions() {
            // Rollback RAM to the pre-unsubscribe state.
            if had_attention {
                self.attention.insert(pubkey, ());
            }
            if let Some((k, v)) = saved_list {
                self.lists.insert(k, v);
            }
            return Err(e);
        }
        info!(curator = %pubkey_hex, "unsubscribed from curator");
        // Sprint 75 Phase C: also drop any node directory + persisted locator
        // for this pubkey. An unsubscribed anchor must neither keep surfacing its
        // catalog in Browse nor be re-pulled at boot (verrou 5: subscribed-only).
        // Best-effort and AFTER the attention-set persist already succeeded: a
        // directory left in RAM on a failed locator rewrite is harmless because
        // `directory_snapshot` / `repull_directories` re-gate on `is_subscribed`.
        self.directories.remove(&pubkey);
        if self.anchor_locators.remove(&pubkey).is_some() {
            if let Err(e) = self.persist_anchors() {
                warn!(error = %e, "failed to rewrite anchors.json after unsubscribe");
            }
        }
        Ok(pubkey)
    }

    /// Return the attention set as a sorted vector of hex
    /// strings. Sorted so callers (the HTTP /curators endpoint,
    /// the Phase F verification, test assertions) see a stable
    /// order across DashMap internal shuffling.
    pub fn subscribed_pubkeys_hex(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .attention
            .iter()
            .map(|e| hex::encode(e.key()))
            .collect();
        out.sort();
        out
    }

    /// Whether `pubkey` is currently in the attention set.
    pub fn is_subscribed(&self, pubkey: &[u8; PUBLIC_KEY_LENGTH]) -> bool {
        self.attention.contains_key(pubkey)
    }

    // ---------------------------------------------------------
    // List storage / inspection
    // ---------------------------------------------------------

    /// Number of curator lists currently cached.
    pub fn known_list_count(&self) -> usize {
        self.lists.len()
    }

    /// Total number of project entries across every cached
    /// list. Used by [`crate::state::DaemonStateSnapshot`] to
    /// populate `known_browse_entries` without the shell
    /// polling every list individually.
    ///
    /// Sprint 75 Phase C (verrou 2 — honest count): also includes the catalog
    /// apps of every SUBSCRIBED node directory, so the discoverable-app count
    /// reflects PULL-discovered catalogs too, not just curator-vouched projects.
    /// Gated on `is_subscribed` for the same reason `directory_snapshot` is.
    pub fn known_entry_count(&self) -> usize {
        let curator: usize = self
            .lists
            .iter()
            .map(|e| e.value().list.entries.len())
            .sum();
        let directory: usize = self
            .directories
            .iter()
            .filter(|e| self.is_subscribed(e.key()))
            .map(|e| e.value().directory.catalog.len())
            .sum();
        curator + directory
    }

    /// Test-only: subscribe to the entry's anchor and insert a verified
    /// directory directly, bypassing the network fetch. Used by the browse
    /// aggregator unit test to populate the directory store without two nodes.
    #[cfg(test)]
    pub fn insert_directory_for_test(&self, entry: NodeDirectoryEntry) {
        self.attention.insert(entry.node_id, ());
        self.directories.insert(entry.node_id, entry);
    }

    /// Test-only: inject a raw anchor locator (pubkey → ticket) WITHOUT
    /// subscribing. Lets a test exercise the `repull_directories` `is_subscribed`
    /// filter and bad-locator tolerance on a present-but-untrusted locator.
    #[cfg(test)]
    pub fn insert_anchor_locator_for_test(&self, pubkey: [u8; PUBLIC_KEY_LENGTH], ticket: &str) {
        self.anchor_locators.insert(pubkey, (ticket.to_string(), 0));
    }

    /// Number of node directories currently cached from SUBSCRIBED anchors.
    pub fn known_directory_count(&self) -> usize {
        self.directories
            .iter()
            .filter(|e| self.is_subscribed(e.key()))
            .count()
    }

    /// Snapshot of every cached node directory from a SUBSCRIBED anchor,
    /// deep-cloned so the caller (the browse aggregator) can flatten it without
    /// holding a DashMap iterator guard across an async probe. Gated on
    /// `is_subscribed` at read time (defense-in-depth alongside the unsubscribe
    /// eviction): an anchor the user dropped never surfaces its catalog even if
    /// a stale entry lingers in RAM. Sorted by node_id for a deterministic
    /// `/browse` order. Sprint 75 Phase C.
    pub fn directory_snapshot(&self) -> Vec<NodeDirectoryEntry> {
        let mut out: Vec<NodeDirectoryEntry> = self
            .directories
            .iter()
            .filter(|e| self.is_subscribed(e.key()))
            .map(|e| e.value().clone())
            .collect();
        out.sort_by_key(|e| e.node_id);
        out
    }

    // ---------------------------------------------------------
    // Observed (non-subscribed) directory publishers — UX-ARRIVAL
    // ---------------------------------------------------------

    /// Record that the NON-subscribed `pubkey` emitted a PoW-valid directory
    /// announcement, observed at LOCAL clock `now` (unix secs). Returns `true`
    /// when the registry accepted the observation (insert or `last_seen`
    /// refresh), `false` when it was dropped.
    ///
    /// Defenses live IN the primitive, never as caller conventions (§P59.2):
    ///
    ///  - **Subscribed exclusion**: a node in the attention set never enters
    ///    `observed` (it has a real `directories` arm); the announce path only
    ///    reaches this on the `!is_subscribed` branch, this re-check is
    ///    defense-in-depth.
    ///  - **Rate limit (PO requirement)**: at most one accepted refresh per
    ///    node per [`OBSERVED_REFRESH_MIN_SECS`] — a one-identity re-publish
    ///    spam loop cannot churn the registry faster than this.
    ///  - **TTL**: entries older than [`OBSERVED_SEEN_TTL_SECS`] are lazily
    ///    purged on every write (the map is ≤ 256 entries, a full retain is
    ///    cheap) and on every read ([`Self::observed_snapshot`]).
    ///  - **Cap + stalest eviction**: at most [`MAX_OBSERVED_DIRECTORIES`]
    ///    resident; when full a fresher newcomer evicts the stalest resident
    ///    and a staler newcomer is dropped (SeedRegistry SEED-2 policy,
    ///    deterministic pubkey tie-break).
    ///  - **Clamp**: `last_seen` IS the local receive clock — the gossip
    ///    envelope carries no claimed timestamp, so `min(now, claimed)`
    ///    (SEED-1) is trivially satisfied by construction.
    ///
    /// The key is the PARSED pubkey (`[u8; 32]`): hex-case normalization
    /// (§P59.3) is structural — `parse_pubkey_hex` only accepts lowercase and
    /// every hex serialization back out (`hex::encode`) is lowercase.
    pub fn record_observed_directory(&self, pubkey: [u8; PUBLIC_KEY_LENGTH], now: u64) -> bool {
        if self.is_subscribed(&pubkey) {
            return false;
        }
        let mut observed = self.observed.lock().unwrap_or_else(|p| p.into_inner());
        // Lazy TTL purge (write side).
        let cutoff = now.saturating_sub(OBSERVED_SEEN_TTL_SECS);
        observed.retain(|_, ts| *ts >= cutoff);
        // Per-node rate limit: an entry refreshed less than the window ago
        // keeps its current `last_seen`. `saturating_sub` keeps a backwards
        // clock step rate-limited rather than panicking/underflowing.
        //
        // Limiter state IS the resident entry — deliberately (Codex R1 GAP,
        // adjudicated as designed): an identity evicted by the cap is indeed
        // re-accepted immediately if it re-announces, but ONE identity can
        // never exploit that to self-churn. Getting evicted requires being
        // the registry-wide STALEST, i.e. 256 distinct fresher identities
        // exist — already the multi-identity flood regime documented as the
        // accepted residual (THREAT_MODEL §15.1: forged identities are not
        // priced individually). Once re-admitted it is the freshest entry,
        // so this rate check holds it again for the full window; outside the
        // flood regime an entry only leaves through the 48h TTL (>> 60s), so
        // the limiter state cannot be lost while it matters. A limiter store
        // that survived eviction would itself need a cap, reopening the same
        // displacement question one level down — no extra defense, just
        // moved state.
        if let Some(ts) = observed.get(&pubkey) {
            if now.saturating_sub(*ts) < OBSERVED_REFRESH_MIN_SECS {
                return false;
            }
        }
        // Cap: only a NEW key can grow the map past the bound.
        if !observed.contains_key(&pubkey) && observed.len() >= MAX_OBSERVED_DIRECTORIES {
            let stalest = observed
                .iter()
                .map(|(k, ts)| (*k, *ts))
                .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            match stalest {
                Some((victim, victim_ts)) if now > victim_ts => {
                    observed.remove(&victim);
                }
                // A newcomer no fresher than every resident bounces off the
                // full registry — a stale flood cannot displace live hints.
                _ => return false,
            }
        }
        observed.insert(pubkey, now);
        true
    }

    /// Snapshot of the observed (non-subscribed) directory publishers still
    /// within the TTL at `now`, freshest first (then pubkey ascending, for a
    /// deterministic `/nodes` payload). Lazily purges expired entries, and
    /// re-gates on `!is_subscribed` at read time (defense-in-depth alongside
    /// the [`Self::subscribe`] purge): a node the user just followed never
    /// surfaces as "observed" even if a stale entry lingers.
    pub fn observed_snapshot(&self, now: u64) -> Vec<([u8; PUBLIC_KEY_LENGTH], u64)> {
        let mut observed = self.observed.lock().unwrap_or_else(|p| p.into_inner());
        let cutoff = now.saturating_sub(OBSERVED_SEEN_TTL_SECS);
        observed.retain(|_, ts| *ts >= cutoff);
        let mut out: Vec<([u8; PUBLIC_KEY_LENGTH], u64)> = observed
            .iter()
            .filter(|(k, _)| !self.is_subscribed(k))
            .map(|(k, ts)| (*k, *ts))
            .collect();
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }

    /// Number of observed publishers currently resident (test assertions).
    pub fn observed_count(&self) -> usize {
        self.observed
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len()
    }

    /// Snapshot of every cached curator list, deep-cloned so
    /// the caller can serialize the result without holding a
    /// DashMap iterator guard across the await boundary of the
    /// HTTP handler.
    pub fn list_snapshot(&self) -> Vec<CuratorListEntry> {
        let mut out: Vec<CuratorListEntry> = self.lists.iter().map(|e| e.value().clone()).collect();
        // Stable ordering: oldest curator pubkey first (bytewise
        // ascending). Keeps the HTTP response deterministic under
        // concurrent inserts.
        out.sort_by_key(|a| a.curator_pubkey);
        out
    }

    /// Return the cached entry for `curator_pubkey`, or `None`
    /// if nothing is stored yet.
    pub fn get_list(&self, curator_pubkey: &[u8; PUBLIC_KEY_LENGTH]) -> Option<CuratorListEntry> {
        self.lists.get(curator_pubkey).map(|e| e.value().clone())
    }

    // ---------------------------------------------------------
    // Gossip ingestion
    // ---------------------------------------------------------

    /// Acquire a backpressure permit then delegate to
    /// [`Self::process_announcement_bytes`]. Sprint 9 Phase E
    /// (C-4 close): callers from the gossip loop should use this
    /// method so the semaphore caps the in-flight blob-fetch
    /// concurrency at [`MAX_INFLIGHT_ANNOUNCEMENTS`].
    pub async fn process_announcement_bytes_throttled(
        &self,
        announcement_bytes: &[u8],
        node: &Node,
    ) -> Result<CuratorListEntry, CuratorRuntimeError> {
        let _permit = self
            .announcement_semaphore
            .acquire()
            .await
            .expect("announcement_semaphore is never closed");
        self.process_announcement_bytes(announcement_bytes, node)
            .await
    }

    /// Parse, verify, and (if applicable) store a gossip
    /// announcement + its referenced blob.
    ///
    /// Ordering of checks:
    ///
    /// 1. JSON-parse the announcement envelope.
    /// 2. Reject unknown `v` versions (fail fast, no fetch).
    /// 3. Parse the curator pubkey hex. Reject malformed hex.
    /// 4. If the curator is not in the attention set, drop the
    ///    announcement without fetching anything (bandwidth +
    ///    DoS mitigation).
    /// 5. Fetch the referenced blob via [`BlobsClient::fetch_ticket`]
    ///    and read the bytes.
    /// 6. Parse as [`CuratorListEntry`] and call
    ///    [`CuratorListEntry::verify_signature`] which enforces
    ///    version, cap, attribution split-brain, signature.
    /// 7. Cross-check: the curator pubkey declared in the
    ///    announcement MUST equal the one inside the fetched
    ///    entry. This catches a malicious announcer stapling a
    ///    different curator's legitimately-signed list to their
    ///    own pubkey.
    /// 8. Revision dedup (plan R6): new.revision MUST be
    ///    strictly greater than stored.revision. Equal or lower
    ///    → reject as rollback.
    /// 9. Store in the DashMap.
    ///
    /// Returns the stored [`CuratorListEntry`] on success so
    /// callers (the gossip loop, the integration test) can
    /// observe what landed.
    pub async fn process_announcement_bytes(
        &self,
        announcement_bytes: &[u8],
        node: &Node,
    ) -> Result<CuratorListEntry, CuratorRuntimeError> {
        // Step 1 + 2: parse envelope + version check.
        let announcement: CuratorAnnouncement = serde_json::from_slice(announcement_bytes)?;
        if announcement.version != ANNOUNCEMENT_VERSION {
            return Err(CuratorRuntimeError::AnnouncementVersion {
                got: announcement.version,
                expected: ANNOUNCEMENT_VERSION,
            });
        }

        // Step 3: parse pubkey hex.
        let ann_pubkey = parse_pubkey_hex(&announcement.curator_pubkey_hex)?;

        // Step 4: attention set filter.
        if !self.is_subscribed(&ann_pubkey) {
            debug!(
                curator = %announcement.curator_pubkey_hex,
                "ignoring announcement from non-subscribed curator"
            );
            // `NotSubscribed` is the benign sentinel — the
            // gossip handler matches on this variant explicitly
            // and silently drops at `debug!` without emitting a
            // warning. Kept distinct from `EnvelopeMismatch`
            // (audit C-2) so a flood of routine drops does not
            // mask a real spoofing attempt.
            return Err(CuratorRuntimeError::NotSubscribed {
                curator: announcement.curator_pubkey_hex.clone(),
            });
        }

        // Step 5: fetch the blob via the wrapped BlobsClient.
        let blobs = BlobsClient::new(node.blobs_store());
        let hash = blobs
            .fetch_ticket(
                node.endpoint(),
                node.memory_lookup(),
                &announcement.blob_ticket,
            )
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;
        let body = blobs
            .get_bytes(hash)
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;

        // Steps 6-8 via the shared signed-list ingest gate (Sprint 75
        // Phase B): signature verification, the envelope-vs-payload
        // attribution cross-check, and revision rollback protection are
        // factored into `verify_signed_list_ingest` so the curator arm
        // and the Phase C node-directory arm can never drift apart (R1,
        // design_review C1). The gate's errors map 1:1 back to the curator
        // error surface via `From<SignedListIngestError>`, so this refactor
        // preserves the curator arm's behaviour — guarded end-to-end by the
        // networked `two_nodes_reject_revision_rollback` /
        // `two_nodes_reject_attribution_mismatch_in_announcement` tests. Blob
        // fetch (step 5, above) and storage (step 9, below) stay type-specific.
        let entry: CuratorListEntry =
            serde_json::from_slice(&body).map_err(CuratorRuntimeError::EntryParse)?;
        let stored_revision = self.lists.get(&ann_pubkey).map(|e| e.value().list.revision);
        verify_signed_list_ingest(&entry, &ann_pubkey, stored_revision)?;

        // Step 9: store.
        self.lists.insert(entry.curator_pubkey, entry.clone());
        info!(
            curator = %hex::encode(entry.curator_pubkey),
            revision = entry.list.revision,
            entries = entry.list.entries.len(),
            "accepted curator list"
        );
        Ok(entry)
    }

    // ---------------------------------------------------------
    // Node directory ingestion (Sprint 75 Phase C)
    // ---------------------------------------------------------

    /// Acquire a backpressure permit then delegate to
    /// [`Self::process_directory_announcement_bytes`]. The gossip loop uses this
    /// so a directory-announcement flood shares the SAME in-flight fetch cap
    /// ([`MAX_INFLIGHT_ANNOUNCEMENTS`]) as the curator arm.
    pub async fn process_directory_announcement_bytes_throttled(
        &self,
        announcement_bytes: &[u8],
        node: &Node,
    ) -> Result<NodeDirectoryEntry, CuratorRuntimeError> {
        let _permit = self
            .announcement_semaphore
            .acquire()
            .await
            .expect("announcement_semaphore is never closed");
        self.process_directory_announcement_bytes(announcement_bytes, node)
            .await
    }

    /// Parse, verify, and store a node-directory gossip announcement + its
    /// referenced blob — the receive-side sibling of
    /// [`Self::process_announcement_bytes`], reusing the SAME shared ingest gate
    /// ([`verify_signed_list_ingest`]) so the curator and directory arms can
    /// never drift apart (R1, design_review C1). The pipeline mirrors the curator
    /// arm step-for-step:
    ///
    /// 1. Parse the [`NodeDirectoryAnnouncement`] envelope.
    /// 2. Reject an unknown `v`.
    /// 3. Parse the node pubkey hex.
    /// 4. Drop a non-subscribed anchor BEFORE any fetch — a node directory is
    ///    gated by the SAME attention set as a curator list (DQ3: one
    ///    subscription covers both, since the node signs with the same key
    ///    family). This is the curation leg of the anti-Sybil triad.
    /// 5. Fetch the referenced blob.
    /// 6. Verify the signature, cross-check envelope-vs-payload attribution, and
    ///    reject a revision rollback — steps 6-8 of the curator pipeline, all via
    ///    the shared `verify_signed_list_ingest` gate.
    /// 9. Store the verified entry (RAM) AND persist the re-fetch locator
    ///    (`anchors.json`) so the catalog survives a reboot (D4 durability).
    pub async fn process_directory_announcement_bytes(
        &self,
        announcement_bytes: &[u8],
        node: &Node,
    ) -> Result<NodeDirectoryEntry, CuratorRuntimeError> {
        // Step 1 + 2: parse envelope + version check.
        let announcement: NodeDirectoryAnnouncement = serde_json::from_slice(announcement_bytes)?;
        if announcement.version != ANNOUNCEMENT_VERSION {
            return Err(CuratorRuntimeError::AnnouncementVersion {
                got: announcement.version,
                expected: ANNOUNCEMENT_VERSION,
            });
        }

        // Step 3: parse pubkey hex.
        let ann_pubkey = parse_pubkey_hex(&announcement.node_pubkey_hex)?;

        // Step 4: attention-set filter (same set as curator lists, DQ3).
        if !self.is_subscribed(&ann_pubkey) {
            // UX-ARRIVAL (post-S75): retain the cheap-envelope METADATA of the
            // publisher (pubkey + local receive clock) BEFORE the drop, so the
            // arrival screen can list "nodes heard on the network" with a
            // subscribe CTA. The drop itself is UNCHANGED: no fetch, no dial,
            // no catalog ingest for a non-subscribed node (S75-C decision +
            // THREAT_MODEL §15.1 anti-amplification) — the registry only ever
            // stores what this envelope already gave us. Bounded + rate-limited
            // inside the primitive. Self-guard (review UX-OBS-SELF-NODE): this
            // node never observes ITSELF — neither via the gossip echo of its
            // own directory broadcast (a node is not subscribed to its own
            // key) nor via a remote announce forging our node_id (the claimed
            // pubkey is unauthenticated; the projects arm has the same guard,
            // `announcement_claims_own_node_id`).
            if announcement.node_pubkey_hex == node.node_id() {
                debug!(
                    "ignoring node directory announcement claiming OUR node_id (self-echo or forgery)"
                );
            } else {
                let recorded = self.record_observed_directory(ann_pubkey, unix_now());
                debug!(
                    node = %announcement.node_pubkey_hex,
                    observed = recorded,
                    "ignoring node directory announcement from non-subscribed anchor"
                );
            }
            return Err(CuratorRuntimeError::NotSubscribed {
                curator: announcement.node_pubkey_hex.clone(),
            });
        }

        // Step 5: fetch the blob via the wrapped BlobsClient.
        let blobs = BlobsClient::new(node.blobs_store());
        let hash = blobs
            .fetch_ticket(
                node.endpoint(),
                node.memory_lookup(),
                &announcement.blob_ticket,
            )
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;
        let body = blobs
            .get_bytes(hash)
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;

        // Steps 6-8 via the shared signed-list ingest gate. The rollback floor
        // depends on WHERE the last-seen revision lives, with two different
        // strictnesses (Codex rounds 4 + 6):
        //  - RAM holds a directory for this anchor → strict `>` dedup, exactly like
        //    the curator arm: a same-revision live re-announce is a duplicate.
        //  - RAM is empty but the PERSISTED locator carries revision P (a boot
        //    re-pull failed → the catalog was lost on reboot) → accept revision
        //    `>= P` so a same-revision live re-announce RESTORES the lost catalog,
        //    while a lower revision is still rejected as a rollback. The gate's
        //    strict `>` is fed `P - 1` to mean `>= P`.
        //  - Never seen → no floor.
        let entry: NodeDirectoryEntry =
            serde_json::from_slice(&body).map_err(CuratorRuntimeError::EntryParse)?;
        let ram_revision = self
            .directories
            .get(&ann_pubkey)
            .map(|e| e.value().directory.revision);
        let persisted_revision = self.anchor_locators.get(&ann_pubkey).map(|e| e.value().1);
        let stored_revision = match (ram_revision, persisted_revision) {
            (Some(r), _) => Some(r),
            (None, Some(p)) => Some(p.saturating_sub(1)),
            (None, None) => None,
        };
        verify_signed_list_ingest(&entry, &ann_pubkey, stored_revision)?;

        // Step 9: store the verified entry (RAM) + persist the re-fetch locator
        // (ticket + the revision just accepted, for the reboot rollback floor).
        self.directories.insert(ann_pubkey, entry.clone());
        self.anchor_locators.insert(
            ann_pubkey,
            (announcement.blob_ticket.clone(), entry.directory.revision),
        );
        if let Err(e) = self.persist_anchors() {
            // Best-effort: a locator-persist failure does not reject an already
            // verified+stored directory; it only weakens boot durability.
            warn!(error = %e, "failed to persist anchors.json after directory ingest");
        }
        info!(
            node = %hex::encode(entry.node_id),
            revision = entry.directory.revision,
            catalog = entry.directory.catalog.len(),
            "accepted node directory"
        );
        Ok(entry)
    }

    /// Re-pull every SUBSCRIBED anchor's node directory from its persisted
    /// locator (Sprint 75 Phase C — the D4 durability primitive). Called once at
    /// boot, after `subscriptions.json` + `anchors.json` are loaded: the in-memory
    /// `directories` map starts empty on every boot, so without this a remote
    /// catalog would not survive a reboot until the anchor next re-announces.
    ///
    /// For each `(pubkey, ticket, revision)` locator whose pubkey is in the
    /// attention set (verrou 5 — never fetch an unsubscribed anchor; an empty
    /// default attention set means a fresh install does ZERO boot network fetch /
    /// no silent leak), fetch the blob by ticket and run it through the SAME
    /// `verify_signed_list_ingest` gate as a live announce (signature +
    /// per-anchor revision floor). A re-fetch that fails verify, finds the anchor
    /// offline, or times out yields nothing for that anchor — the catalog
    /// reappears on the next live announce. Each fetch is bounded by
    /// [`REPULL_FETCH_TIMEOUT`] so a single dead anchor cannot stall boot.
    /// Returns the number of anchors successfully restored.
    pub async fn repull_directories(&self, node: &Node) -> usize {
        // Snapshot the subscribed locators so we don't hold a DashMap guard
        // across the awaits below.
        let locators: Vec<([u8; PUBLIC_KEY_LENGTH], String, u64)> = self
            .anchor_locators
            .iter()
            .filter(|e| self.is_subscribed(e.key()))
            .map(|e| (*e.key(), e.value().0.clone(), e.value().1))
            .collect();
        let mut restored = 0usize;
        for (pubkey, ticket, revision) in locators {
            match tokio::time::timeout(
                REPULL_FETCH_TIMEOUT,
                self.repull_one_directory(node, &pubkey, &ticket, revision),
            )
            .await
            {
                Ok(Ok(())) => restored += 1,
                Ok(Err(e)) => {
                    debug!(anchor = %hex::encode(pubkey), error = %e, "boot re-pull of anchor directory failed");
                }
                Err(_) => {
                    debug!(anchor = %hex::encode(pubkey), "boot re-pull of anchor directory timed out");
                }
            }
        }
        if restored > 0 {
            info!(
                restored,
                "re-pulled node directories from persisted anchors"
            );
        }
        restored
    }

    /// Fetch + verify + store ONE anchor's directory from its locator ticket.
    /// Shared by [`Self::repull_directories`]; split out so each anchor's fetch
    /// can be wrapped in its own timeout. Re-validates the re-fetched blob
    /// exactly as a live announce (the persisted ticket is an untrusted locator —
    /// the signature + revision gate is the authority, not the ticket).
    ///
    /// `persisted_revision` is the directory revision we last ingested from this
    /// anchor (from `anchors.json`). It carries the anti-rollback floor ACROSS the
    /// reboot: the RAM `directories` map is empty at boot, so the gate floor is
    /// taken from the persisted revision. The fetched blob must verify at a
    /// revision `>= persisted_revision` (the gate's strict `>` is fed
    /// `persisted_revision - 1`) so the re-fetch of the persisted blob RESTORES the
    /// catalog; the immutable content hash means a re-fetch already yields exactly
    /// that revision, so this rejects only a forged older-but-signed substitution —
    /// defense-in-depth on top of content-addressing (Codex round-2 GAP).
    async fn repull_one_directory(
        &self,
        node: &Node,
        pubkey: &[u8; PUBLIC_KEY_LENGTH],
        ticket: &str,
        persisted_revision: u64,
    ) -> Result<(), CuratorRuntimeError> {
        let blobs = BlobsClient::new(node.blobs_store());
        let hash = blobs
            .fetch_ticket(node.endpoint(), node.memory_lookup(), ticket)
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;
        let body = blobs
            .get_bytes(hash)
            .await
            .map_err(CuratorRuntimeError::BlobFetch)?;
        let entry: NodeDirectoryEntry =
            serde_json::from_slice(&body).map_err(CuratorRuntimeError::EntryParse)?;
        // Floor = `persisted_revision - 1`, so the re-fetch of the persisted blob
        // (revision == persisted) passes the gate's strict `>` and RESTORES the
        // catalog, while any older signed blob is rejected even though RAM started
        // empty at boot. The persisted ticket pins the LATEST revision (the locator
        // is rewritten on every ingest), so a re-fetch never yields an older one —
        // this floor is defense-in-depth on top of content-addressing; if RAM
        // already holds the same revision, re-storing it is idempotent.
        verify_signed_list_ingest(&entry, pubkey, Some(persisted_revision.saturating_sub(1)))?;
        self.directories.insert(*pubkey, entry);
        Ok(())
    }

    // ---------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------

    /// Atomically rewrite the `subscriptions.json` file from the
    /// current attention set. No-op when `persistence_path` is
    /// `None`. Errors are surfaced so a full-disk failure
    /// aborts the offending subscribe/unsubscribe call.
    fn persist_subscriptions(&self) -> Result<(), CuratorRuntimeError> {
        let Some(path) = self.persistence_path.as_ref() else {
            return Ok(());
        };
        let file = SubscriptionsFile {
            schema_version: SUBSCRIPTIONS_SCHEMA_VERSION,
            curators: self.subscribed_pubkeys_hex(),
        };
        let body =
            serde_json::to_vec_pretty(&file).map_err(|e| CuratorRuntimeError::Persistence {
                path: path.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CuratorRuntimeError::Persistence {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let tmp = path.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp).map_err(|e| CuratorRuntimeError::Persistence {
                path: tmp.clone(),
                source: e,
            })?;
            f.write_all(&body)
                .map_err(|e| CuratorRuntimeError::Persistence {
                    path: tmp.clone(),
                    source: e,
                })?;
            f.sync_all().map_err(|e| CuratorRuntimeError::Persistence {
                path: tmp.clone(),
                source: e,
            })?;
        }
        fs::rename(&tmp, path).map_err(|e| CuratorRuntimeError::Persistence {
            path: path.clone(),
            source: e,
        })?;
        debug!(path = %path.display(), "subscriptions.json rewritten");
        Ok(())
    }

    /// Populate the attention set from a `subscriptions.json`
    /// file at the configured persistence path. A missing,
    /// unreadable, or schema-mismatched file is treated as an
    /// empty set and logged at warn level — the runtime refuses
    /// to crash at boot over a stale disk file.
    pub fn load_subscriptions(&self) {
        let Some(path) = self.persistence_path.as_ref() else {
            return;
        };
        if !path.exists() {
            debug!(path = %path.display(), "no subscriptions.json — starting with empty attention set");
            return;
        }
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "failed to read subscriptions.json");
                return;
            }
        };
        let file: SubscriptionsFile = match serde_json::from_str(&body) {
            Ok(f) => f,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "subscriptions.json is not valid JSON");
                return;
            }
        };
        if file.schema_version != SUBSCRIPTIONS_SCHEMA_VERSION {
            warn!(
                path = %path.display(),
                found = file.schema_version,
                expected = SUBSCRIPTIONS_SCHEMA_VERSION,
                "subscriptions.json schema mismatch — ignoring"
            );
            return;
        }
        for hex_str in &file.curators {
            match parse_pubkey_hex(hex_str) {
                Ok(pk) => {
                    self.attention.insert(pk, ());
                }
                Err(e) => {
                    warn!(bad_hex = %hex_str, error = %e, "ignoring invalid pubkey in subscriptions.json");
                }
            }
        }
        info!(
            count = self.attention.len(),
            "attention set restored from subscriptions.json"
        );
    }

    /// Atomically rewrite `anchors.json` from the current locator set. No-op
    /// when `anchors_path` is `None`. Mirrors [`Self::persist_subscriptions`]'s
    /// tmp-file + fsync + rename durability. Sprint 75 Phase C.
    fn persist_anchors(&self) -> Result<(), CuratorRuntimeError> {
        let Some(path) = self.anchors_path.as_ref() else {
            return Ok(());
        };
        let mut anchors: Vec<AnchorLocator> = self
            .anchor_locators
            .iter()
            .map(|e| AnchorLocator {
                pubkey: hex::encode(e.key()),
                ticket: e.value().0.clone(),
                revision: e.value().1,
            })
            .collect();
        // Stable order so the file is diff-friendly across DashMap shuffling.
        anchors.sort_by(|a, b| a.pubkey.cmp(&b.pubkey));
        let file = AnchorsFile {
            schema_version: ANCHORS_SCHEMA_VERSION,
            anchors,
        };
        let body =
            serde_json::to_vec_pretty(&file).map_err(|e| CuratorRuntimeError::Persistence {
                path: path.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CuratorRuntimeError::Persistence {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let tmp = path.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp).map_err(|e| CuratorRuntimeError::Persistence {
                path: tmp.clone(),
                source: e,
            })?;
            f.write_all(&body)
                .map_err(|e| CuratorRuntimeError::Persistence {
                    path: tmp.clone(),
                    source: e,
                })?;
            f.sync_all().map_err(|e| CuratorRuntimeError::Persistence {
                path: tmp.clone(),
                source: e,
            })?;
        }
        fs::rename(&tmp, path).map_err(|e| CuratorRuntimeError::Persistence {
            path: path.clone(),
            source: e,
        })?;
        debug!(path = %path.display(), "anchors.json rewritten");
        Ok(())
    }

    /// Populate the anchor locator set from `anchors.json`. A missing,
    /// unreadable, or schema-mismatched file is treated as an empty set and
    /// logged at warn level — boot never crashes over a stale disk file.
    /// Sprint 75 Phase C.
    pub fn load_anchors(&self) {
        let Some(path) = self.anchors_path.as_ref() else {
            return;
        };
        if !path.exists() {
            debug!(path = %path.display(), "no anchors.json — no directories to re-pull at boot");
            return;
        }
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "failed to read anchors.json");
                return;
            }
        };
        let file: AnchorsFile = match serde_json::from_str(&body) {
            Ok(f) => f,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "anchors.json is not valid JSON");
                return;
            }
        };
        if file.schema_version != ANCHORS_SCHEMA_VERSION {
            warn!(
                path = %path.display(),
                found = file.schema_version,
                expected = ANCHORS_SCHEMA_VERSION,
                "anchors.json schema mismatch — ignoring"
            );
            return;
        }
        for loc in &file.anchors {
            match parse_pubkey_hex(&loc.pubkey) {
                Ok(pk) => {
                    self.anchor_locators
                        .insert(pk, (loc.ticket.clone(), loc.revision));
                }
                Err(e) => {
                    warn!(bad_hex = %loc.pubkey, error = %e, "ignoring invalid pubkey in anchors.json");
                }
            }
        }
        info!(
            count = self.anchor_locators.len(),
            "anchor locators restored from anchors.json"
        );
    }
}

/// Parse a 64-char lowercase hex string into a 32-byte Ed25519
/// public key. Rejects anything that isn't the exact shape — no
/// uppercase tolerance, no `0x` prefix — so the on-disk and
/// on-wire representation stays canonical.
pub fn parse_pubkey_hex(s: &str) -> Result<[u8; PUBLIC_KEY_LENGTH], CuratorRuntimeError> {
    if s.len() != PUBLIC_KEY_LENGTH * 2 || !s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(CuratorRuntimeError::BadPubkeyHex(s.to_string()));
    }
    let mut out = [0u8; PUBLIC_KEY_LENGTH];
    hex::decode_to_slice(s, &mut out)
        .map_err(|_| CuratorRuntimeError::BadPubkeyHex(s.to_string()))?;
    Ok(out)
}

// Convenience Arc shorthand used by the binary runtime.
pub type CuratorRuntimeHandle = Arc<CuratorRuntime>;

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::{
        CuratorList, CuratorListEntry, CuratorProjectRef, KeyPair, Node, create_node,
    };
    use tempfile::tempdir;

    fn mk_entry(kp: &KeyPair, revision: u64, entries: usize) -> CuratorListEntry {
        let mut list = CuratorList::new(kp.public_bytes(), "FlowUP test", 1_712_000_000, revision);
        for i in 0..entries {
            list.entries.push(CuratorProjectRef {
                project_id: format!("{:064x}", i),
                project_name: format!("proj{i}"),
                category: "misc".into(),
                description: "test fixture".into(),
            });
        }
        CuratorListEntry::sign(list, kp).expect("sign")
    }

    // ---------------------------------------------------------
    // Topic id + announcement wire format
    // ---------------------------------------------------------

    #[test]
    fn curator_topic_id_is_deterministic_and_32_bytes() {
        let a = curator_topic_id();
        let b = curator_topic_id();
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        // Regression against anyone silently changing the seed.
        assert_eq!(CURATOR_TOPIC_SEED, b"nexus-grid/curator/v1");
    }

    #[test]
    fn announcement_round_trips_through_json() {
        let ann = CuratorAnnouncement::new([0xAB; PUBLIC_KEY_LENGTH], "blobaaxxx".into());
        let bytes = ann.to_bytes().unwrap();
        let back: CuratorAnnouncement = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.version, ANNOUNCEMENT_VERSION);
        assert_eq!(back.curator_pubkey_hex.len(), 64);
    }

    // ---------------------------------------------------------
    // Shared signed-list ingest gate (Sprint 75 Phase B)
    // ---------------------------------------------------------

    fn mk_directory_entry(kp: &KeyPair, revision: u64) -> nexus_core_rs::NodeDirectoryEntry {
        let mut dir = nexus_core_rs::NodeDirectory::new(kp.public_bytes(), revision);
        dir.catalog.push(nexus_core_rs::CatalogApp {
            project_id: "a".repeat(64),
            archive_hash: "b".repeat(64),
            project_name: "Babel".into(),
            category: "translation".into(),
            description: "fixture".into(),
        });
        nexus_core_rs::NodeDirectoryEntry::sign(dir, kp).expect("sign directory")
    }

    #[test]
    fn node_directory_announcement_round_trips_through_json() {
        let ann = NodeDirectoryAnnouncement::new([0xCD; PUBLIC_KEY_LENGTH], "blobaaxxx".into());
        let bytes = ann.to_bytes().unwrap();
        let back: NodeDirectoryAnnouncement = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.version, ANNOUNCEMENT_VERSION);
        assert_eq!(back.node_pubkey_hex.len(), 64);
    }

    #[test]
    fn node_directory_announcement_is_not_a_curator_announcement() {
        // The two announcement types never parse as each other (directory has
        // `node`, curator has `curator`, neither with a serde default) — this
        // is what lets the dispatch discriminate cleanly and keeps a directory
        // announcement from ever being mis-ingested as a curator list.
        let ann = NodeDirectoryAnnouncement::new([0x11; PUBLIC_KEY_LENGTH], "blobaaxxx".into());
        let bytes = ann.to_bytes().unwrap();
        assert!(
            serde_json::from_slice::<CuratorAnnouncement>(&bytes).is_err(),
            "a node-directory announcement must NOT parse as a curator announcement"
        );
    }

    #[test]
    fn is_node_directory_announcement_discriminates() {
        // The dispatch discriminator: true for a directory announcement, false
        // for a curator one, so the gossip loop drops a directory announcement
        // at debug! (Phase C wires its full ingest) instead of warn!-ing it
        // through the curator arm.
        let dir = NodeDirectoryAnnouncement::new([0x22; PUBLIC_KEY_LENGTH], "blobaaxxx".into());
        let cur = CuratorAnnouncement::new([0x33; PUBLIC_KEY_LENGTH], "blobaaxxx".into());
        assert!(is_node_directory_announcement(&dir.to_bytes().unwrap()));
        assert!(!is_node_directory_announcement(&cur.to_bytes().unwrap()));
    }

    #[test]
    fn is_node_directory_announcement_rejects_hybrid() {
        // serde ignores unknown fields, so a hybrid carrying BOTH `node` and
        // `curator` parses as each type. It must NOT be classified as a directory
        // announcement (which would silently drop a legitimate curator
        // announcement) — it stays on the curator path, preserving the
        // pre-Phase-B dispatch behaviour. Cheap DoS/misclassification guard.
        let hybrid =
            serde_json::json!({ "v": 1, "node": "aa", "curator": "bb", "ticket": "blobx" });
        let bytes = serde_json::to_vec(&hybrid).unwrap();
        assert!(
            !is_node_directory_announcement(&bytes),
            "a node+curator hybrid must not be treated as a directory announcement"
        );
        assert!(
            serde_json::from_slice::<CuratorAnnouncement>(&bytes).is_ok(),
            "the hybrid still parses as a curator announcement (curator arm runs)"
        );
    }

    #[test]
    fn generic_ingest_helper_parity() {
        // The shared gate must produce the SAME verdict for a curator list
        // and a node directory on equivalent inputs: a valid entry passes, a
        // wrong announced pubkey is an EnvelopeMismatch, and a non-monotone
        // revision is a RevisionRollback. This guards the gate's TYPE-SYMMETRY
        // (both SignedList impls treated identically); the curator arm's
        // end-to-end behaviour-preservation across the refactor is guarded by
        // the networked two_nodes_reject_* tests, not this unit test.
        let kp = KeyPair::generate();
        let other = KeyPair::generate();

        let curator = mk_entry(&kp, 5, 1);
        let directory = mk_directory_entry(&kp, 5);

        // Valid: announced pubkey == signer, nothing stored yet.
        assert!(verify_signed_list_ingest(&curator, &kp.public_bytes(), None).is_ok());
        assert!(verify_signed_list_ingest(&directory, &kp.public_bytes(), None).is_ok());

        // Envelope mismatch: announced pubkey != signer.
        assert!(matches!(
            verify_signed_list_ingest(&curator, &other.public_bytes(), None),
            Err(SignedListIngestError::EnvelopeMismatch { .. })
        ));
        assert!(matches!(
            verify_signed_list_ingest(&directory, &other.public_bytes(), None),
            Err(SignedListIngestError::EnvelopeMismatch { .. })
        ));

        // Rollback: revision <= stored.
        assert!(matches!(
            verify_signed_list_ingest(&curator, &kp.public_bytes(), Some(5)),
            Err(SignedListIngestError::RevisionRollback { new: 5, stored: 5 })
        ));
        assert!(matches!(
            verify_signed_list_ingest(&directory, &kp.public_bytes(), Some(9)),
            Err(SignedListIngestError::RevisionRollback { new: 5, stored: 9 })
        ));
    }

    #[test]
    fn signed_list_ingest_error_maps_to_curator_error() {
        // The 1:1 mapping that keeps the curator arm's error surface
        // unchanged after the refactor onto the shared gate.
        let kp = KeyPair::generate();
        let curator = mk_entry(&kp, 1, 1);
        let err = verify_signed_list_ingest(&curator, &KeyPair::generate().public_bytes(), None)
            .unwrap_err();
        let mapped: CuratorRuntimeError = err.into();
        assert!(matches!(
            mapped,
            CuratorRuntimeError::EnvelopeMismatch { .. }
        ));
    }

    #[test]
    fn node_directory_revision_monotone_rollback() {
        // The shared gate rejects a node-directory revision that is not
        // strictly greater than the stored one (the rollback guard the
        // Phase C ingest arm relies on).
        let kp = KeyPair::generate();
        let entry = mk_directory_entry(&kp, 3);
        assert!(matches!(
            verify_signed_list_ingest(&entry, &kp.public_bytes(), Some(3)),
            Err(SignedListIngestError::RevisionRollback { .. })
        ));
        assert!(matches!(
            verify_signed_list_ingest(&entry, &kp.public_bytes(), Some(4)),
            Err(SignedListIngestError::RevisionRollback { .. })
        ));
        assert!(verify_signed_list_ingest(&entry, &kp.public_bytes(), Some(2)).is_ok());
    }

    // ---------------------------------------------------------
    // parse_pubkey_hex
    // ---------------------------------------------------------

    #[test]
    fn parse_pubkey_hex_accepts_lowercase_64_chars() {
        let s = "a".repeat(64);
        let out = parse_pubkey_hex(&s).unwrap();
        assert_eq!(out, [0xAA; PUBLIC_KEY_LENGTH]);
    }

    #[test]
    fn parse_pubkey_hex_rejects_wrong_length() {
        assert!(parse_pubkey_hex("").is_err());
        assert!(parse_pubkey_hex(&"a".repeat(63)).is_err());
        assert!(parse_pubkey_hex(&"a".repeat(65)).is_err());
    }

    #[test]
    fn parse_pubkey_hex_rejects_uppercase() {
        // Canonical representation is lowercase — uppercase
        // would produce a second valid string for the same key
        // and trip dedup in the future (sorted sets, DashMap
        // keys, etc.).
        let s = "A".repeat(64);
        assert!(parse_pubkey_hex(&s).is_err());
    }

    #[test]
    fn parse_pubkey_hex_rejects_non_hex() {
        let s = "z".repeat(64);
        assert!(parse_pubkey_hex(&s).is_err());
    }

    // ---------------------------------------------------------
    // Attention set management
    // ---------------------------------------------------------

    #[test]
    fn subscribe_and_unsubscribe_roundtrip_without_persistence() {
        let runtime = CuratorRuntime::new(None);
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        assert_eq!(runtime.subscribed_pubkeys_hex().len(), 0);

        runtime.subscribe(&hex_key).expect("subscribe");
        assert_eq!(runtime.subscribed_pubkeys_hex(), vec![hex_key.clone()]);
        assert!(runtime.is_subscribed(&kp.public_bytes()));

        // Idempotent — second subscribe must not duplicate.
        runtime.subscribe(&hex_key).unwrap();
        assert_eq!(runtime.subscribed_pubkeys_hex().len(), 1);

        runtime.unsubscribe(&hex_key).unwrap();
        assert!(!runtime.is_subscribed(&kp.public_bytes()));
    }

    #[test]
    fn unsubscribe_also_evicts_stored_list() {
        let runtime = CuratorRuntime::new(None);
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        runtime.subscribe(&hex_key).unwrap();
        // Inject a stored list directly so we don't need a Node
        // for this focused unit test.
        runtime.lists.insert(kp.public_bytes(), mk_entry(&kp, 1, 1));
        assert_eq!(runtime.known_list_count(), 1);

        runtime.unsubscribe(&hex_key).unwrap();
        assert_eq!(
            runtime.known_list_count(),
            0,
            "unsubscribing must evict the stored list"
        );
    }

    #[test]
    fn subscribe_rejects_bad_hex() {
        let runtime = CuratorRuntime::new(None);
        assert!(runtime.subscribe("not-hex").is_err());
        assert!(runtime.subscribe(&"A".repeat(64)).is_err()); // uppercase
    }

    #[test]
    fn subscribed_pubkeys_hex_is_sorted() {
        let runtime = CuratorRuntime::new(None);
        // Insert in non-sorted order — the sorted invariant is
        // what the HTTP handler and integration assertions rely
        // on.
        runtime.subscribe(&"b".repeat(64)).unwrap();
        runtime.subscribe(&"a".repeat(64)).unwrap();
        runtime.subscribe(&"c".repeat(64)).unwrap();
        let out = runtime.subscribed_pubkeys_hex();
        assert_eq!(out, vec!["a".repeat(64), "b".repeat(64), "c".repeat(64)]);
    }

    // ---------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------

    #[test]
    fn persist_and_reload_subscriptions_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("shell-daemon").join("subscriptions.json");

        let rt1 = CuratorRuntime::with_persistence(path.clone());
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        rt1.subscribe(&hex::encode(kp_a.public_bytes())).unwrap();
        rt1.subscribe(&hex::encode(kp_b.public_bytes())).unwrap();

        assert!(path.exists(), "subscriptions.json must be written");

        // Fresh runtime against the same file — attention set
        // should rebuild.
        let rt2 = CuratorRuntime::with_persistence(path.clone());
        assert!(rt2.is_subscribed(&kp_a.public_bytes()));
        assert!(rt2.is_subscribed(&kp_b.public_bytes()));
    }

    #[test]
    fn unsubscribe_persists_the_removal() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subscriptions.json");

        let rt1 = CuratorRuntime::with_persistence(path.clone());
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());
        rt1.subscribe(&hex_key).unwrap();
        rt1.unsubscribe(&hex_key).unwrap();

        let rt2 = CuratorRuntime::with_persistence(path);
        assert!(!rt2.is_subscribed(&kp.public_bytes()));
    }

    #[test]
    fn load_subscriptions_treats_corrupt_file_as_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subscriptions.json");
        fs::write(&path, b"not json").unwrap();
        let rt = CuratorRuntime::with_persistence(path);
        assert_eq!(rt.subscribed_pubkeys_hex().len(), 0);
    }

    #[test]
    fn load_subscriptions_ignores_schema_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subscriptions.json");
        fs::write(&path, br#"{"schema_version": 999, "curators": ["aa"]}"#).unwrap();
        let rt = CuratorRuntime::with_persistence(path);
        assert_eq!(rt.subscribed_pubkeys_hex().len(), 0);
    }

    // ---------------------------------------------------------
    // list_snapshot + known_entry_count
    // ---------------------------------------------------------

    #[test]
    fn list_snapshot_is_sorted_by_curator_pubkey() {
        let runtime = CuratorRuntime::new(None);
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        runtime
            .lists
            .insert(kp_a.public_bytes(), mk_entry(&kp_a, 1, 3));
        runtime
            .lists
            .insert(kp_b.public_bytes(), mk_entry(&kp_b, 1, 2));

        let snap = runtime.list_snapshot();
        assert_eq!(snap.len(), 2);
        assert!(snap[0].curator_pubkey <= snap[1].curator_pubkey);
        assert_eq!(runtime.known_entry_count(), 5);
    }

    // ---------------------------------------------------------
    // 2-node integration test — full fetch + verify + store
    // ---------------------------------------------------------

    async fn spawn_node() -> Node {
        create_node().await.expect("boot")
    }

    fn mint_ticket(blobs_a: &BlobsClient<'_>, bytes: [u8; 32], addr: iroh::EndpointAddr) -> String {
        use iroh_blobs::ticket::BlobTicket;
        use iroh_blobs::{BlobFormat, Hash};
        let _ = blobs_a; // kept for signature symmetry
        BlobTicket::new(addr, Hash::from_bytes(bytes), BlobFormat::Raw).to_string()
    }

    async fn publish_list_and_mint_announcement(
        node_a: &Node,
        entry: &CuratorListEntry,
    ) -> CuratorAnnouncement {
        // Serialize the entry, add it to node A's blob store,
        // and mint a BlobTicket pointing at it.
        let body = serde_json::to_vec(entry).unwrap();
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(&body).await.unwrap();

        let my_addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("publisher must expose an address");

        let ticket = mint_ticket(&blobs_a, hash, my_addr);
        CuratorAnnouncement::new(entry.curator_pubkey, ticket)
    }

    #[tokio::test]
    async fn two_nodes_subscribe_and_fetch_curator_list() {
        // Publisher node stores a signed list + mints a
        // ticket; subscriber daemon runtime ingests the
        // announcement end-to-end. Exercises fetch_ticket →
        // verify → store against live iroh endpoints.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry = mk_entry(&kp, 1, 2);
        let announcement = publish_list_and_mint_announcement(&node_a, &entry).await;

        let runtime = CuratorRuntime::new(None);
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");

        let accepted = runtime
            .process_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
            .await
            .expect("ingest must succeed on a well-formed announcement");
        assert_eq!(accepted, entry);
        assert_eq!(runtime.known_list_count(), 1);
        assert_eq!(runtime.known_entry_count(), 2);

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn two_nodes_reject_announcement_from_non_subscribed_curator() {
        // Same setup but the subscriber never subscribes to
        // this curator — the runtime must drop the
        // announcement at step 4 without fetching anything.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry = mk_entry(&kp, 1, 1);
        let announcement = publish_list_and_mint_announcement(&node_a, &entry).await;

        let runtime = CuratorRuntime::new(None);
        // No subscribe() call.

        let result = runtime
            .process_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
            .await;
        assert!(result.is_err(), "non-subscribed curator must be rejected");
        assert_eq!(runtime.known_list_count(), 0);

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn two_nodes_reject_revision_rollback() {
        // Ingest revision 5, then ingest revision 3 — the
        // rollback must be refused.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry_new = mk_entry(&kp, 5, 1);
        let entry_old = mk_entry(&kp, 3, 1);

        let ann_new = publish_list_and_mint_announcement(&node_a, &entry_new).await;
        let ann_old = publish_list_and_mint_announcement(&node_a, &entry_old).await;

        let runtime = CuratorRuntime::new(None);
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");

        runtime
            .process_announcement_bytes(&ann_new.to_bytes().unwrap(), &node_b)
            .await
            .expect("revision 5 must be accepted first");

        let rollback = runtime
            .process_announcement_bytes(&ann_old.to_bytes().unwrap(), &node_b)
            .await;
        assert!(matches!(
            rollback,
            Err(CuratorRuntimeError::RevisionRollback { .. })
        ));

        // The stored list must still be revision 5.
        let stored = runtime.get_list(&kp.public_bytes()).unwrap();
        assert_eq!(stored.list.revision, 5);

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn two_nodes_reject_attribution_mismatch_in_announcement() {
        // The announcement envelope claims curator A but the
        // fetched entry is signed by curator B. The runtime must
        // catch this before the DashMap insert and surface the
        // `EnvelopeMismatch` variant (audit C-2 split) so the
        // gossip handler can log it at `warn!`.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp_real = KeyPair::generate();
        let kp_attacker = KeyPair::generate();
        let entry = mk_entry(&kp_real, 1, 1);
        let real_ann = publish_list_and_mint_announcement(&node_a, &entry).await;

        // Hand-craft a dishonest announcement pointing at the
        // real curator's blob but attributing it to the
        // attacker's pubkey.
        let dishonest = CuratorAnnouncement::new(kp_attacker.public_bytes(), real_ann.blob_ticket);

        let runtime = CuratorRuntime::new(None);
        runtime
            .subscribe(&hex::encode(kp_attacker.public_bytes()))
            .unwrap();

        let result = runtime
            .process_announcement_bytes(&dishonest.to_bytes().unwrap(), &node_b)
            .await;
        match result {
            Err(CuratorRuntimeError::EnvelopeMismatch {
                announcement,
                entry,
            }) => {
                assert_eq!(announcement, hex::encode(kp_attacker.public_bytes()));
                assert_eq!(entry, hex::encode(kp_real.public_bytes()));
            }
            other => panic!("expected EnvelopeMismatch, got {other:?}"),
        }

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // 2-node node-directory integration tests (Sprint 75 Phase C)
    // ---------------------------------------------------------

    async fn publish_directory_and_mint_announcement(
        node_a: &Node,
        entry: &NodeDirectoryEntry,
    ) -> NodeDirectoryAnnouncement {
        let body = serde_json::to_vec(entry).unwrap();
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(&body).await.unwrap();
        let my_addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("publisher must expose an address");
        let ticket = mint_ticket(&blobs_a, hash, my_addr);
        NodeDirectoryAnnouncement::new(entry.node_id, ticket)
    }

    #[tokio::test]
    async fn node_directory_ingest_subscription_gated() {
        // A node directory from a non-subscribed anchor is dropped at the
        // attention gate (no fetch); the SAME announcement ingests once the
        // anchor is subscribed. Mirrors the curator subscription gate (DQ3: one
        // attention set covers both), reusing the shared ingest path.
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry = mk_directory_entry(&kp, 1);
        let announcement = publish_directory_and_mint_announcement(&node_a, &entry).await;

        let runtime = CuratorRuntime::new(None);
        // Not subscribed → dropped at step 4, nothing stored.
        let result = runtime
            .process_directory_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
            .await;
        assert!(
            matches!(result, Err(CuratorRuntimeError::NotSubscribed { .. })),
            "a non-subscribed anchor's directory must be dropped at the attention gate"
        );
        assert_eq!(runtime.known_directory_count(), 0);
        // UX-ARRIVAL: the drop retained the cheap-envelope observed hint
        // (pubkey + local clock), catalog still NOT ingested.
        assert_eq!(runtime.observed_count(), 1);
        let observed = runtime.observed_snapshot(unix_now());
        assert_eq!(observed.len(), 1);
        assert_eq!(
            observed[0].0,
            kp.public_bytes(),
            "the observed hint must carry the announcing node's pubkey"
        );

        // Subscribe to the anchor pubkey → the same announcement now ingests.
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");
        // The observed→subscribed transition retires the hint.
        assert_eq!(
            runtime.observed_count(),
            0,
            "subscribing must purge the node from the observed registry"
        );
        let accepted = runtime
            .process_directory_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
            .await
            .expect("a subscribed anchor's directory must ingest");
        assert_eq!(accepted, entry);
        assert_eq!(runtime.known_directory_count(), 1);
        // A subscribed ingest never re-enters the observed registry.
        assert_eq!(runtime.observed_count(), 0);
        // The catalog apps count toward the honest discoverable-app total (verrou 2).
        assert_eq!(runtime.known_entry_count(), entry.directory.catalog.len());

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // Observed directory registry (UX-ARRIVAL, post-S75)
    // ---------------------------------------------------------

    /// Deterministic distinct pubkey for registry tests.
    fn observed_pk(i: usize) -> [u8; PUBLIC_KEY_LENGTH] {
        let mut pk = [0u8; PUBLIC_KEY_LENGTH];
        pk[..8].copy_from_slice(&(i as u64).to_be_bytes());
        pk
    }

    #[test]
    fn observed_registry_rate_limits_per_node() {
        // PO requirement: one accepted refresh per node per window — a
        // one-identity re-publish spam loop cannot churn the registry.
        let runtime = CuratorRuntime::new(None);
        let pk = observed_pk(7);
        let t0 = 1_700_000_000u64;

        assert!(runtime.record_observed_directory(pk, t0));
        // Inside the window: dropped, last_seen unchanged.
        assert!(!runtime.record_observed_directory(pk, t0 + OBSERVED_REFRESH_MIN_SECS - 1));
        assert_eq!(runtime.observed_snapshot(t0 + 1), vec![(pk, t0)]);
        // At the window boundary: accepted, last_seen refreshed.
        let t1 = t0 + OBSERVED_REFRESH_MIN_SECS;
        assert!(runtime.record_observed_directory(pk, t1));
        assert_eq!(runtime.observed_snapshot(t1), vec![(pk, t1)]);
        // A backwards clock step stays rate-limited (saturating_sub), never
        // a panic or an accepted past-dated refresh.
        assert!(!runtime.record_observed_directory(pk, t0));
        assert_eq!(runtime.observed_snapshot(t1), vec![(pk, t1)]);
    }

    #[test]
    fn observed_registry_ttl_expires() {
        // An observed hint not re-heard within the TTL drops out on read,
        // and the lazy purge actually frees the entry.
        let runtime = CuratorRuntime::new(None);
        let pk = observed_pk(1);
        let t0 = 1_700_000_000u64;

        assert!(runtime.record_observed_directory(pk, t0));
        // Still resident at the TTL boundary (ts >= cutoff)...
        assert_eq!(
            runtime.observed_snapshot(t0 + OBSERVED_SEEN_TTL_SECS).len(),
            1
        );
        // ...gone one second past it, and physically purged.
        assert!(
            runtime
                .observed_snapshot(t0 + OBSERVED_SEEN_TTL_SECS + 1)
                .is_empty()
        );
        assert_eq!(runtime.observed_count(), 0);
    }

    #[test]
    fn observed_registry_cap_evicts_stalest() {
        // SEED-2 policy transposed: bounded in SPACE, fresher newcomer evicts
        // the stalest resident, staler newcomer bounces off the full registry.
        //
        // Decorrelation (review TI-1): pubkeys DESCEND while timestamps
        // ASCEND, so an eviction that picked its victim by key order instead
        // of by staleness would be unmasked (with correlated fixtures both
        // policies select the same victim — a tautology).
        let runtime = CuratorRuntime::new(None);
        let t0 = 1_700_000_000u64;
        let total = MAX_OBSERVED_DIRECTORIES + 10;

        for i in 0..total {
            assert!(runtime.record_observed_directory(observed_pk(total - i), t0 + i as u64));
        }
        assert_eq!(
            runtime.observed_count(),
            MAX_OBSERVED_DIRECTORIES,
            "exactly the cap after 1-for-1 eviction, never fewer"
        );
        let t_end = t0 + total as u64;
        let snapshot = runtime.observed_snapshot(t_end);
        // Victim choice pinned by STALENESS: the 10 oldest inserts (i=0..9,
        // i.e. the LARGEST pubkeys total..total-9) were evicted; the oldest
        // non-victim (i=10 → pk(total-10)=pk(MAX)) and the freshest (i=total-1
        // → pk(1)) both survive.
        assert!(
            !snapshot.iter().any(|(k, _)| *k == observed_pk(total - 9)),
            "the stalest resident must be the eviction victim"
        );
        assert!(
            snapshot
                .iter()
                .any(|(k, _)| *k == observed_pk(MAX_OBSERVED_DIRECTORIES)),
            "the oldest non-victim must survive"
        );
        let freshest = observed_pk(1);
        assert!(snapshot.iter().any(|(k, _)| *k == freshest));
        // Snapshot order: freshest first (deterministic /nodes payload).
        assert_eq!(snapshot[0].0, freshest);
        // Anti-displacement: a newcomer no fresher than the stalest resident
        // (ts = t0+10) is dropped — a stale flood cannot displace live hints.
        assert!(!runtime.record_observed_directory(observed_pk(999_999), t0 + 10));
        assert_eq!(runtime.observed_count(), MAX_OBSERVED_DIRECTORIES);
        // Post-bounce content pinned (review TI-3): the bounced newcomer is
        // NOT resident and the stalest resident it failed to displace is.
        let after = runtime.observed_snapshot(t_end);
        assert!(
            !after.iter().any(|(k, _)| *k == observed_pk(999_999)),
            "the bounced newcomer must not be resident"
        );
        assert!(
            after
                .iter()
                .any(|(k, _)| *k == observed_pk(MAX_OBSERVED_DIRECTORIES)),
            "the resident the newcomer failed to displace must survive intact"
        );
        // Eviction does erase the limiter state for the victim (limiter state
        // IS the entry — deliberate, Codex R1 adjudication): an evicted
        // identity re-announcing is re-admitted immediately... but once
        // re-admitted it is the FRESHEST entry, so the rate limit holds it
        // again for the full window — one identity can never self-churn.
        let evicted = observed_pk(total - 9);
        assert!(
            runtime.record_observed_directory(evicted, t_end + 1),
            "an evicted identity is re-admitted on its next announce"
        );
        assert!(
            !runtime.record_observed_directory(evicted, t_end + 2),
            "a re-admitted identity is immediately rate-limited again"
        );
        assert_eq!(runtime.observed_count(), MAX_OBSERVED_DIRECTORIES);
    }

    #[test]
    fn observed_registry_excludes_subscribed() {
        // The two stores are mutually exclusive: a subscribed node is never
        // recorded, the observed→subscribed transition purges the hint, and
        // the snapshot re-gates at read time on a lingering stale entry.
        let runtime = CuratorRuntime::new(None);
        let t0 = 1_700_000_000u64;

        // A subscribed node never enters observed (primitive-level guard).
        let kp = KeyPair::generate();
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");
        assert!(!runtime.record_observed_directory(kp.public_bytes(), t0));
        assert_eq!(runtime.observed_count(), 0);

        // observed → subscribed purges the resident hint.
        let kp2 = KeyPair::generate();
        assert!(runtime.record_observed_directory(kp2.public_bytes(), t0));
        assert_eq!(runtime.observed_count(), 1);
        runtime
            .subscribe(&hex::encode(kp2.public_bytes()))
            .expect("subscribe");
        assert_eq!(runtime.observed_count(), 0);
        assert!(runtime.observed_snapshot(t0).is_empty());

        // Read-time gate: an entry that lingers past a subscription that
        // bypassed the purge (direct attention insert) never surfaces.
        let kp3 = KeyPair::generate();
        assert!(runtime.record_observed_directory(kp3.public_bytes(), t0));
        runtime.attention.insert(kp3.public_bytes(), ());
        assert!(runtime.observed_snapshot(t0).is_empty());
    }

    #[test]
    fn observed_capture_is_availability_only() {
        // Sprint 76 Phase B (B1, publisher-binding — decision (b)): the observed
        // registry is AVAILABILITY-ONLY, not publisher-authenticated. At the
        // `process_directory_announcement_bytes` layer the claimed
        // `node_pubkey_hex` is unauthenticated — the gossip envelope's PoW binds
        // (publisher, topic), NOT the payload's self-declared pubkey, and the
        // verified envelope author is not plumbed to this layer (the call-site
        // passes only content + node). So ANY non-subscribed pubkey is recorded
        // as an availability hint, BY DESIGN. The real anti-flood defenses are:
        // (1) the self-forge guard — a claim of OUR node_id is dropped
        // (`observed_recorded_without_any_fetch`); (2) subscribed exclusion
        // (`observed_registry_excludes_subscribed`); (3) the bounded +
        // rate-limited registry (cap + stalest-eviction + 1/min + 48h TTL). This
        // test PINS decision (b): never mistake observed for an authenticated set
        // (THREAT_MODEL §15.1). Binding it to the PoW publisher would need the
        // envelope author at this layer — deferred, not over-promised here.
        let runtime = CuratorRuntime::new(None);
        let t0 = 1_700_000_000u64;
        // Two distinct, unrelated, unauthenticated pubkeys are BOTH recorded.
        assert!(runtime.record_observed_directory(observed_pk(101), t0));
        assert!(runtime.record_observed_directory(observed_pk(202), t0));
        assert_eq!(
            runtime.observed_count(),
            2,
            "observed is availability-only: any non-subscribed pubkey is a hint, no PoW binding"
        );
    }

    #[tokio::test]
    async fn observed_recorded_without_any_fetch() {
        // THE anti-amplification assertion (THREAT_MODEL §15.1): an
        // unsolicited directory announcement is retained as cheap-envelope
        // metadata WITHOUT any outbound fetch — the ticket here is
        // unfetchable, so reaching step 5 would error `BlobFetch`, never
        // `NotSubscribed`.
        let node = spawn_node().await;
        let kp = KeyPair::generate();
        let ann =
            NodeDirectoryAnnouncement::new(kp.public_bytes(), "fake-ticket-never-fetched".into());

        let runtime = CuratorRuntime::new(None);
        let result = runtime
            .process_directory_announcement_bytes(&ann.to_bytes().unwrap(), &node)
            .await;
        assert!(
            matches!(result, Err(CuratorRuntimeError::NotSubscribed { .. })),
            "the drop must fire at step 4, before any fetch"
        );
        assert_eq!(runtime.observed_count(), 1);
        let observed = runtime.observed_snapshot(unix_now());
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].0, kp.public_bytes());
        let first_seen = observed[0].1;

        // Rate limit through the FULL ingest path (review TI-4): the same
        // announcement re-emitted immediately is dropped by the primitive —
        // the registry stays at one entry and last_seen is NOT refreshed.
        let again = runtime
            .process_directory_announcement_bytes(&ann.to_bytes().unwrap(), &node)
            .await;
        assert!(matches!(
            again,
            Err(CuratorRuntimeError::NotSubscribed { .. })
        ));
        assert_eq!(runtime.observed_count(), 1);
        let after = runtime.observed_snapshot(unix_now());
        assert_eq!(
            after[0].1, first_seen,
            "an immediate re-publish must not refresh last_seen (1/min rate limit)"
        );

        // Self-guard (review UX-OBS-SELF-NODE): an announcement claiming OUR
        // node_id — the gossip echo of our own broadcast, or a remote forgery
        // of our identity — is never captured as observed.
        let self_pk = parse_pubkey_hex(&node.node_id())
            .expect("node_id() must be the same 64-hex form the announce wire uses");
        let self_ann = NodeDirectoryAnnouncement::new(self_pk, "fake-ticket-never-fetched".into());
        let result = runtime
            .process_directory_announcement_bytes(&self_ann.to_bytes().unwrap(), &node)
            .await;
        assert!(matches!(
            result,
            Err(CuratorRuntimeError::NotSubscribed { .. })
        ));
        assert_eq!(
            runtime.observed_count(),
            1,
            "an announcement claiming OUR node_id must never enter observed"
        );
        assert!(
            !runtime
                .observed_snapshot(unix_now())
                .iter()
                .any(|(k, _)| *k == self_pk)
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn boot_repull_restores_remote_catalogs() {
        // THE load-bearing durability assertion (D4): a remote catalog ingested
        // by node B survives B's reboot via the persisted anchor locator + boot
        // re-pull — even though the in-memory directory store is RAM-only and
        // starts empty on every boot.
        let tmp = tempdir().unwrap();
        let subs_path = tmp.path().join("subscriptions.json");

        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry = mk_directory_entry(&kp, 5);
        let announcement = publish_directory_and_mint_announcement(&node_a, &entry).await;

        // ---- B's first boot: subscribe + ingest A's directory ----
        {
            let runtime = CuratorRuntime::with_persistence(subs_path.clone());
            runtime
                .subscribe(&hex::encode(kp.public_bytes()))
                .expect("subscribe");
            runtime
                .process_directory_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
                .await
                .expect("ingest");
            assert_eq!(runtime.known_directory_count(), 1);
            // The subscription (subscriptions.json) + the anchor locator
            // (anchors.json) are now persisted next to each other on disk.
        }

        // The anchors.json locator persists the REVISION (5), not just the ticket,
        // so the anti-rollback floor survives the reboot (Codex round-2 GAP): the
        // RAM directory map is empty at boot, and without the persisted revision
        // the re-pull would have no floor.
        let anchors_path = subs_path.with_file_name("anchors.json");
        let anchors_file: AnchorsFile = serde_json::from_str(
            &std::fs::read_to_string(&anchors_path).expect("read anchors.json"),
        )
        .expect("parse anchors.json");
        assert_eq!(anchors_file.anchors.len(), 1);
        assert_eq!(
            anchors_file.anchors[0].revision, 5,
            "anchors.json must persist the last ingested revision as the rollback floor"
        );

        // ---- B reboots: a fresh runtime over the SAME persistence path ----
        let rebooted = CuratorRuntime::with_persistence(subs_path.clone());
        // The directory store is RAM-only → empty after reboot...
        assert_eq!(
            rebooted.known_directory_count(),
            0,
            "the RAM-only directory store must start empty on boot"
        );
        // ...but the subscription + the re-fetch locator survived.
        assert!(rebooted.is_subscribed(&kp.public_bytes()));

        // The boot re-pull re-fetches A's still-served blob and restores it.
        let restored = rebooted.repull_directories(&node_b).await;
        assert_eq!(restored, 1, "the subscribed anchor's catalog must re-pull");
        assert_eq!(rebooted.known_directory_count(), 1);
        let snap = rebooted.directory_snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(
            snap[0], entry,
            "the re-pulled catalog must match what A published"
        );
        assert_eq!(snap[0].directory.revision, 5);

        // The rollback floor is ACTIVE post-reboot: a live announce at a LOWER
        // revision than the re-pulled one (5) is rejected, even though RAM started
        // empty before the re-pull restored the floor from anchors.json.
        let older = mk_directory_entry(&kp, 4);
        let older_ann = publish_directory_and_mint_announcement(&node_a, &older).await;
        let rollback = rebooted
            .process_directory_announcement_bytes(&older_ann.to_bytes().unwrap(), &node_b)
            .await;
        assert!(
            matches!(rollback, Err(CuratorRuntimeError::RevisionRollback { .. })),
            "a lower-revision announce must be rejected after reboot+re-pull restored the floor"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn live_ingest_respects_persisted_floor_after_failed_repull() {
        // Codex round-4 GAP: even when the boot re-pull did NOT restore an anchor
        // (it was offline → RAM directory store empty for it), a LIVE announce at
        // a LOWER revision than the last-seen one must STILL be rejected, using the
        // persisted locator revision as the floor. Otherwise a failed re-pull opens
        // a rollback window.
        let tmp = tempdir().unwrap();
        let subs_path = tmp.path().join("subscriptions.json");
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;
        let kp = KeyPair::generate();

        // First boot: ingest revision 5 → persists locator revision 5.
        {
            let rt = CuratorRuntime::with_persistence(subs_path.clone());
            rt.subscribe(&hex::encode(kp.public_bytes())).unwrap();
            let entry5 = mk_directory_entry(&kp, 5);
            let ann5 = publish_directory_and_mint_announcement(&node_a, &entry5).await;
            rt.process_directory_announcement_bytes(&ann5.to_bytes().unwrap(), &node_b)
                .await
                .expect("ingest 5");
        }

        // Reboot: a fresh runtime loads anchors.json (revision 5) but does NOT
        // re-pull (simulate the anchor offline at boot — RAM stays empty).
        let rebooted = CuratorRuntime::with_persistence(subs_path.clone());
        assert_eq!(rebooted.known_directory_count(), 0);

        // A live announce at revision 4 must be rejected via the PERSISTED floor,
        // even though RAM is empty (no re-pull ran).
        let entry4 = mk_directory_entry(&kp, 4);
        let ann4 = publish_directory_and_mint_announcement(&node_a, &entry4).await;
        let res = rebooted
            .process_directory_announcement_bytes(&ann4.to_bytes().unwrap(), &node_b)
            .await;
        assert!(
            matches!(res, Err(CuratorRuntimeError::RevisionRollback { .. })),
            "a lower-revision live announce must be rejected via the persisted floor after a failed re-pull"
        );

        // But a SAME-revision live re-announce (5) must RESTORE the lost catalog
        // (Codex round-6): the persisted floor means `>= 5`, not `> 5` — otherwise
        // a failed re-pull would leave the catalog unrecoverable until the
        // publisher bumps its revision.
        let entry5b = mk_directory_entry(&kp, 5);
        let ann5b = publish_directory_and_mint_announcement(&node_a, &entry5b).await;
        rebooted
            .process_directory_announcement_bytes(&ann5b.to_bytes().unwrap(), &node_b)
            .await
            .expect(
                "a same-revision live re-announce must restore the catalog after a failed re-pull",
            );
        assert_eq!(rebooted.known_directory_count(), 1);

        // Once RAM holds revision 5 again, the strict same-revision dedup is back
        // (the curator-arm behaviour): a SECOND announce at 5 is now a duplicate.
        let dup = rebooted
            .process_directory_announcement_bytes(&ann5b.to_bytes().unwrap(), &node_b)
            .await;
        assert!(
            matches!(dup, Err(CuratorRuntimeError::RevisionRollback { .. })),
            "with RAM repopulated, a same-revision re-announce is deduped strictly"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn repull_skips_unsubscribed_anchor() {
        // verrou 5: a locator whose anchor is NOT in the attention set is never
        // re-pulled (a fresh install with an empty attention set does zero boot
        // fetch). Here we persist a locator then unsubscribe before reboot.
        let tmp = tempdir().unwrap();
        let subs_path = tmp.path().join("subscriptions.json");

        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let kp = KeyPair::generate();
        let entry = mk_directory_entry(&kp, 1);
        let announcement = publish_directory_and_mint_announcement(&node_a, &entry).await;

        let runtime = CuratorRuntime::with_persistence(subs_path.clone());
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");
        runtime
            .process_directory_announcement_bytes(&announcement.to_bytes().unwrap(), &node_b)
            .await
            .expect("ingest");
        // Unsubscribing evicts the directory + drops the locator (verrou 5).
        runtime
            .unsubscribe(&hex::encode(kp.public_bytes()))
            .expect("unsubscribe");
        assert_eq!(runtime.known_directory_count(), 0);
        // A re-pull now finds no subscribed locator → restores nothing.
        assert_eq!(runtime.repull_directories(&node_b).await, 0);

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn repull_filters_unsubscribed_locator() {
        // verrou 5 defense-in-depth: even if a locator lingers in `anchors.json`
        // for a pubkey that is NOT in the attention set, the boot re-pull filters
        // it out BEFORE any fetch (it never dials an unsubscribed anchor). Inject
        // a locator directly without subscribing and assert zero re-pull — no
        // network is even attempted (a real ticket is unnecessary).
        let node = spawn_node().await;
        let kp = KeyPair::generate();
        let runtime = CuratorRuntime::new(None);
        runtime.insert_anchor_locator_for_test(kp.public_bytes(), "any-ticket-string");
        // Not subscribed → the locator is filtered, nothing fetched or restored.
        assert_eq!(runtime.repull_directories(&node).await, 0);
        assert_eq!(runtime.known_directory_count(), 0);
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn repull_tolerates_bad_locator() {
        // The tolerated failure path the durability story rests on: a SUBSCRIBED
        // anchor whose persisted ticket is unusable (malformed / forged / stale)
        // must yield NOTHING gracefully — no panic, no poisoned store — rather
        // than aborting the whole re-pull pass. Here the ticket is malformed, so
        // the fetch errors fast (no 15s timeout); the same `Ok(Err)` branch covers
        // an offline anchor.
        let node = spawn_node().await;
        let kp = KeyPair::generate();
        let runtime = CuratorRuntime::new(None);
        runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .expect("subscribe");
        runtime.insert_anchor_locator_for_test(kp.public_bytes(), "not-a-valid-blob-ticket");
        // Subscribed but the locator is unusable → 0 restored, store stays empty,
        // no panic.
        assert_eq!(runtime.repull_directories(&node).await, 0);
        assert_eq!(runtime.known_directory_count(), 0);
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn not_subscribed_and_envelope_mismatch_are_distinct() {
        // Sprint 8 audit C-2 regression guard: the two benign-vs-
        // attack branches of step 4 (attention filter) and step 7
        // (envelope vs entry crosscheck) must surface as distinct
        // `CuratorRuntimeError` variants so the gossip handler
        // can log them at different severities.
        //
        // Branch 1 — non-subscribed curator. No blob fetch, no
        // network — the runtime bails at step 4 before touching
        // iroh. A throwaway `create_node` still has to be passed
        // in because the signature requires it; the handler
        // never reaches the step 5 fetch, so the node is
        // otherwise unused.
        let node = create_node().await.expect("spawn node");
        let kp_stranger = KeyPair::generate();
        // A syntactically valid announcement pointing at a
        // ticket nobody ever fetches.
        let ann = CuratorAnnouncement::new(
            kp_stranger.public_bytes(),
            "fake-ticket-never-fetched".into(),
        );

        let runtime = CuratorRuntime::new(None);
        // Attention set is empty → step 4 fires.
        let result = runtime
            .process_announcement_bytes(&ann.to_bytes().unwrap(), &node)
            .await;
        match result {
            Err(CuratorRuntimeError::NotSubscribed { curator }) => {
                assert_eq!(curator, hex::encode(kp_stranger.public_bytes()));
            }
            other => panic!("expected NotSubscribed, got {other:?}"),
        }

        // Branch 2 (envelope vs entry crosscheck) is covered by
        // `two_nodes_reject_attribution_mismatch_in_announcement`
        // which exercises the full fetch path. The variants are
        // structurally distinct at the type level — a match arm
        // that catches `NotSubscribed` will not catch
        // `EnvelopeMismatch`, so a regression that collapsed them
        // back into one would break either this test or the 2-node
        // test.

        node.shutdown().await.ok();
    }

    // ---------------------------------------------------------
    // C-4 — gossip semaphore backpressure
    // ---------------------------------------------------------

    #[tokio::test]
    async fn gossip_semaphore_limits_inflight_announcements() {
        // Sprint 9 Phase E (C-4 close): the semaphore must cap
        // concurrent announcement processing. We verify the field
        // exists and has the expected permit count. The runtime
        // path itself (process_announcement_bytes_throttled) is
        // exercised indirectly by the 2-node tests; this test pins
        // the concurrency limit at the value documented in
        // PATTERNS.md.
        let runtime = CuratorRuntime::new(None);
        assert_eq!(
            runtime.announcement_semaphore.available_permits(),
            MAX_INFLIGHT_ANNOUNCEMENTS,
        );
    }

    // ---------------------------------------------------------
    // D-3 — subscribe persist-first rollback
    // ---------------------------------------------------------

    #[test]
    fn subscribe_persist_first_rollback_on_disk_failure() {
        // Sprint 9 Phase E (D-3 close): if persist_subscriptions
        // fails, the subscribe call must roll back the RAM insert.
        // We trigger a persist failure by pointing at a path whose
        // parent directory is a plain file (not a dir), making
        // create_dir_all fail.
        let dir = tempdir().unwrap();
        let blocker = dir.path().join("blocker");
        // Create a *file* named "blocker" so that writing
        // "blocker/subscriptions.json" fails because "blocker"
        // is not a directory.
        fs::write(&blocker, b"I am a file, not a dir").unwrap();
        let bad_path = blocker.join("subscriptions.json");

        let runtime = CuratorRuntime::new(Some(bad_path));
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        let result = runtime.subscribe(&hex_key);
        assert!(result.is_err(), "subscribe must fail when persist fails");
        assert!(
            !runtime.is_subscribed(&kp.public_bytes()),
            "RAM must be rolled back on persist failure"
        );
    }
}
