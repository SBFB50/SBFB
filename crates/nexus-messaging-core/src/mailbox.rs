// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline mailbox (ADR_CHAT_V1 decision 5, plan Phase E): wire format of
//! the `sbfb/mailbox/0` exchange and the server-side registry policy.
//!
//! A member that opted in hands an always-on peer a READ ticket of a
//! conversation document. The peer imports a read-only replica — it can
//! serve every entry and forge none — so two members that are never online
//! at the same time still converge through it. What the peer learns is
//! metadata only: which nodes deposit which document, and when.
//!
//! The dialer is authenticated by the QUIC handshake (the connection's
//! remote id is the node key that signed it), so a request carries no
//! signature of its own: the server attributes every deposit to the
//! connection, never to a field of the request.
//!
//! [`MailboxRegistry`] is the whole quota/TTL policy, pure and clocked by
//! the caller like the rest of this crate. The daemon wraps it with I/O
//! (import, drop, persistence).
//!
//! Wire contract: read by another node, so a frontier — but not a signed
//! type, so no `// FRONTIER:` tag (that opt-in registry requires a DOMAIN
//! constant). The exact JSON bytes are pinned by
//! `tests::wire_round_trip_and_strict_fields`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Payload version under the `/0` ALPN generation.
pub const MAILBOX_FORMAT_VERSION: u32 = 1;

/// Upper bound on one request or response on the wire. A read ticket is a
/// few hundred bytes; the cap only exists to bound what a dialer can make
/// the server buffer.
pub const MAX_MAILBOX_MSG_BYTES: usize = 16 * 1024;

/// One request per connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailboxRequest {
    pub version: u32,
    pub op: MailboxOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailboxOp {
    /// Keep — or keep longer — the document this READ ticket names.
    /// Depositing again refreshes the dialer's registration.
    Deposit { ticket: String },
    /// Stop keeping the document for the dialer. The server drops it once
    /// no depositor is left.
    Withdraw { namespace: String },
}

impl MailboxRequest {
    pub fn deposit(ticket: String) -> Self {
        Self {
            version: MAILBOX_FORMAT_VERSION,
            op: MailboxOp::Deposit { ticket },
        }
    }

    pub fn withdraw(namespace: String) -> Self {
        Self {
            version: MAILBOX_FORMAT_VERSION,
            op: MailboxOp::Withdraw { namespace },
        }
    }
}

/// What the server did with a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailboxVerdict {
    /// The document is kept until `expires_at`.
    Kept,
    /// The dialer's registration is gone (or never existed).
    Released,
    /// Undecodable request, wrong version, malformed ticket or namespace.
    BadRequest,
    /// The ticket grants WRITE: refused, the server must never be able to
    /// author an entry in a conversation.
    WriteTicket,
    /// A global or per-node limit is reached.
    Quota,
    /// The server is not ready to keep documents.
    Unavailable,
    /// The replica could not be created locally.
    ImportFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailboxResponse {
    pub version: u32,
    pub verdict: MailboxVerdict,
    /// Unix seconds; `0` unless `verdict == Kept`.
    pub expires_at: u64,
}

impl MailboxResponse {
    pub fn new(verdict: MailboxVerdict, expires_at: u64) -> Self {
        Self {
            version: MAILBOX_FORMAT_VERSION,
            verdict,
            expires_at,
        }
    }
}

/// Server-side limits. The worst-case disk use of live documents is
/// `max_docs_total * max_doc_bytes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailboxLimits {
    /// Lifetime of one registration after its last deposit.
    pub ttl_secs: u64,
    /// Documents kept at once, all depositors together.
    pub max_docs_total: usize,
    /// Documents one node may keep registered at once.
    pub max_docs_per_node: usize,
    /// Size above which a document is evicted (entries + content).
    pub max_doc_bytes: u64,
}

/// Outcome of an accepted deposit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// The namespace was not held: the caller must import the replica.
    New,
    /// The namespace is already held: only the registration moved.
    Held,
}

/// Outcome of a withdrawal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// The dialer had no registration on this namespace.
    NotHeld,
    /// Other depositors keep the document alive.
    StillHeld,
    /// The last depositor left: the caller must drop the replica.
    Last,
}

/// Which node keeps which document until when.
///
/// Keys are opaque strings (the daemon uses the hex namespace id and the
/// hex endpoint id). A namespace stays held while at least one depositor's
/// registration is unexpired; expired registrations are swept by
/// [`MailboxRegistry::expire`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxRegistry {
    docs: BTreeMap<String, BTreeMap<String, u64>>,
}

impl MailboxRegistry {
    /// Register (or refresh) `node` as a depositor of `namespace`.
    ///
    /// Refusals happen before any mutation. A node already registered on
    /// the namespace is never refused: a refresh adds nothing to hold.
    pub fn deposit(
        &mut self,
        namespace: &str,
        node: &str,
        now: u64,
        limits: &MailboxLimits,
    ) -> Option<(Admission, u64)> {
        let expires_at = now.saturating_add(limits.ttl_secs);
        let held = self.docs.contains_key(namespace);
        let already = self
            .docs
            .get(namespace)
            .and_then(|d| d.get(node))
            .is_some_and(|&exp| exp > now);
        if !already {
            if !held && self.live_docs(now) >= limits.max_docs_total {
                return None;
            }
            if self.live_docs_of(node, now) >= limits.max_docs_per_node {
                return None;
            }
        }
        self.docs
            .entry(namespace.to_owned())
            .or_default()
            .insert(node.to_owned(), expires_at);
        let admission = if held {
            Admission::Held
        } else {
            Admission::New
        };
        Some((admission, expires_at))
    }

    /// Remove `node`'s registration on `namespace`.
    pub fn withdraw(&mut self, namespace: &str, node: &str) -> Release {
        let Some(depositors) = self.docs.get_mut(namespace) else {
            return Release::NotHeld;
        };
        if depositors.remove(node).is_none() {
            return Release::NotHeld;
        }
        if depositors.is_empty() {
            self.docs.remove(namespace);
            Release::Last
        } else {
            Release::StillHeld
        }
    }

    /// Sweep registrations that expired at or before `now`; returns the
    /// namespaces left without any depositor (the caller drops them).
    pub fn expire(&mut self, now: u64) -> Vec<String> {
        let mut gone = Vec::new();
        self.docs.retain(|ns, depositors| {
            depositors.retain(|_, &mut exp| exp > now);
            if depositors.is_empty() {
                gone.push(ns.clone());
                false
            } else {
                true
            }
        });
        gone
    }

    /// Forget a namespace whatever its depositors (oversize eviction).
    pub fn evict(&mut self, namespace: &str) -> bool {
        self.docs.remove(namespace).is_some()
    }

    /// Every namespace currently held, expired or not.
    pub fn namespaces(&self) -> Vec<String> {
        self.docs.keys().cloned().collect()
    }

    /// Namespaces with at least one unexpired registration.
    pub fn live_docs(&self, now: u64) -> usize {
        self.docs
            .values()
            .filter(|d| d.values().any(|&exp| exp > now))
            .count()
    }

    fn live_docs_of(&self, node: &str, now: u64) -> usize {
        self.docs
            .values()
            .filter(|d| d.get(node).is_some_and(|&exp| exp > now))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: MailboxLimits = MailboxLimits {
        ttl_secs: 100,
        max_docs_total: 3,
        max_docs_per_node: 2,
        max_doc_bytes: 1 << 20,
    };

    #[test]
    fn wire_round_trip_and_strict_fields() {
        let req = MailboxRequest::deposit("docaaa".into());
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(
            json,
            r#"{"version":1,"op":{"deposit":{"ticket":"docaaa"}}}"#
        );
        assert_eq!(serde_json::from_str::<MailboxRequest>(&json).unwrap(), req);
        let w = MailboxRequest::withdraw("ns".into());
        let back: MailboxRequest =
            serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        assert_eq!(back, w);
        // An extra top-level field is refused, not ignored.
        assert!(
            serde_json::from_str::<MailboxRequest>(
                r#"{"version":1,"op":{"deposit":{"ticket":"x"}},"from":"me"}"#
            )
            .is_err()
        );
        let resp = MailboxResponse::new(MailboxVerdict::WriteTicket, 0);
        assert_eq!(
            serde_json::to_string(&resp).unwrap(),
            r#"{"version":1,"verdict":"write_ticket","expires_at":0}"#
        );
    }

    #[test]
    fn deposit_new_then_held_then_refresh_moves_expiry() {
        let mut r = MailboxRegistry::default();
        assert_eq!(
            r.deposit("d1", "a", 10, &LIMITS),
            Some((Admission::New, 110))
        );
        assert_eq!(
            r.deposit("d1", "b", 20, &LIMITS),
            Some((Admission::Held, 120))
        );
        assert_eq!(
            r.deposit("d1", "a", 50, &LIMITS),
            Some((Admission::Held, 150))
        );
        // a's refreshed registration outlives b's.
        assert!(r.expire(130).is_empty());
        assert_eq!(r.withdraw("d1", "b"), Release::NotHeld, "b expired at 120");
        assert_eq!(r.expire(150), vec!["d1".to_string()]);
        assert!(r.namespaces().is_empty());
    }

    #[test]
    fn per_node_quota_counts_only_live_registrations() {
        let mut r = MailboxRegistry::default();
        assert!(r.deposit("d1", "a", 0, &LIMITS).is_some());
        assert!(r.deposit("d2", "a", 0, &LIMITS).is_some());
        assert_eq!(r.deposit("d3", "a", 0, &LIMITS), None, "a holds 2 = cap");
        // Refreshing a held registration is never refused at the cap.
        assert!(r.deposit("d1", "a", 1, &LIMITS).is_some());
        // Joining a namespace someone else holds still counts against a.
        assert!(r.deposit("d3", "b", 1, &LIMITS).is_some());
        assert_eq!(r.deposit("d3", "a", 1, &LIMITS), None);
        // Once d2's registration (t=0) has expired, a has room again.
        assert!(r.deposit("d3", "a", 100, &LIMITS).is_some());
    }

    #[test]
    fn global_quota_refuses_new_namespaces_only() {
        let mut r = MailboxRegistry::default();
        for (ns, node) in [("d1", "a"), ("d2", "b"), ("d3", "c")] {
            assert!(r.deposit(ns, node, 0, &LIMITS).is_some());
        }
        assert_eq!(r.deposit("d4", "d", 0, &LIMITS), None, "3 docs = cap");
        // A new depositor on an already-held namespace adds no document.
        assert_eq!(
            r.deposit("d1", "d", 0, &LIMITS),
            Some((Admission::Held, 100))
        );
        // A refused deposit mutates nothing.
        assert!(!r.namespaces().contains(&"d4".to_string()));
    }

    #[test]
    fn withdraw_reports_last_depositor() {
        let mut r = MailboxRegistry::default();
        r.deposit("d1", "a", 0, &LIMITS);
        r.deposit("d1", "b", 0, &LIMITS);
        assert_eq!(r.withdraw("d1", "c"), Release::NotHeld);
        assert_eq!(r.withdraw("d1", "a"), Release::StillHeld);
        assert_eq!(r.withdraw("d1", "b"), Release::Last);
        assert_eq!(r.withdraw("d1", "b"), Release::NotHeld);
        assert_eq!(r.withdraw("nope", "a"), Release::NotHeld);
    }

    #[test]
    fn evict_forgets_every_depositor() {
        let mut r = MailboxRegistry::default();
        r.deposit("d1", "a", 0, &LIMITS);
        r.deposit("d1", "b", 0, &LIMITS);
        assert!(r.evict("d1"));
        assert!(!r.evict("d1"));
        assert_eq!(r.live_docs(0), 0);
    }

    #[test]
    fn registry_persists_through_json() {
        let mut r = MailboxRegistry::default();
        r.deposit("d1", "a", 5, &LIMITS);
        let back: MailboxRegistry =
            serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(back, r);
    }
}
