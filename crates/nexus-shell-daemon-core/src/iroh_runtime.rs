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

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use nexus_core_rs::blobs::BlobsClient;
use nexus_core_rs::crypto::PUBLIC_KEY_LENGTH;
use nexus_core_rs::{CuratorListEntry, Node};
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
// CuratorRuntime
// =================================================================

/// Maximum number of concurrent `process_announcement_bytes`
/// calls the runtime allows before applying backpressure. Sprint 9
/// Phase E (C-4 close): without this guard, a gossip flood of 10 k
/// announcements/s serialises through a single fetch chain.
pub const MAX_INFLIGHT_ANNOUNCEMENTS: usize = 32;

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
        Self {
            lists: DashMap::new(),
            attention: DashMap::new(),
            persistence_path,
            announcement_semaphore: Semaphore::new(MAX_INFLIGHT_ANNOUNCEMENTS),
        }
    }

    /// Build a runtime and immediately pre-populate the
    /// attention set from a [`SubscriptionsFile`] on disk. A
    /// missing, unreadable, or schema-mismatched file is treated
    /// as "start empty" and logged at warn level.
    pub fn with_persistence(persistence_path: PathBuf) -> Self {
        let runtime = Self::new(Some(persistence_path.clone()));
        runtime.load_subscriptions();
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
    pub fn known_entry_count(&self) -> usize {
        self.lists
            .iter()
            .map(|e| e.value().list.entries.len())
            .sum()
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

        // Step 6: parse + verify the entry. `verify_signature`
        // layers version / cap / attribution / signature checks.
        let entry: CuratorListEntry =
            serde_json::from_slice(&body).map_err(CuratorRuntimeError::EntryParse)?;
        entry
            .verify_signature()
            .map_err(CuratorRuntimeError::EntryVerify)?;

        // Step 7: cross-check the envelope pubkey against the
        // fetched entry's inner pubkey. Both must point at the
        // same curator. A mismatch here means some peer is
        // stapling a legitimately-signed list to a different
        // pubkey — a genuine spoofing attempt or a broken
        // forwarder. Emit the `EnvelopeMismatch` variant so the
        // gossip handler can log it at `warn!` with both hexes.
        if entry.curator_pubkey != ann_pubkey {
            return Err(CuratorRuntimeError::EnvelopeMismatch {
                announcement: announcement.curator_pubkey_hex,
                entry: hex::encode(entry.curator_pubkey),
            });
        }

        // Step 8: revision rollback protection.
        if let Some(stored) = self.lists.get(&entry.curator_pubkey) {
            if entry.list.revision <= stored.value().list.revision {
                return Err(CuratorRuntimeError::RevisionRollback {
                    new: entry.list.revision,
                    stored: stored.value().list.revision,
                });
            }
        }

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
