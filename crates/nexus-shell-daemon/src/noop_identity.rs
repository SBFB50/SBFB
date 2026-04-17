// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase B — noop routing for the duress identity.
//!
//! When the daemon boots with an identity whose
//! [`IdentityMode::Duress`] flag is set, every outbound operation
//! that would publish data under the **fake** keypair must become
//! a no-op. The handler surface stays responsive — an observer
//! sees HTTP 200s and gossip-topic subscriptions — but nothing
//! reaches the wire under the decoy keypair.
//!
//! ## Surface covered
//!
//! - `gossip_publish_in_duress` : the `/publish` handler pipes
//!   every announcement through this check. In Normal mode the
//!   helper returns `PublishOutcome::Proceed` so the caller runs
//!   the real broadcast. In Duress mode it returns
//!   `PublishOutcome::Noop` and the caller replies 200 without
//!   touching [`nexus_core_rs::TopicSender::broadcast`].
//!
//! - `curator_subscribe_in_duress` : the `/curators/subscribe`
//!   handler calls this before mutating the attention set. In
//!   Duress mode the subscribe is silently dropped — the peer
//!   observer sees an ACK, the local attention set does not grow,
//!   and no `CuratorList` is ever fetched / re-signed with the
//!   fake identity.
//!
//! - `task_dispatch_in_duress` : the `/publish-blob` handler (the
//!   daemon-side path that would upload a task artifact for a
//!   later worker) returns 503 `service in maintenance mode` in
//!   Duress mode. A 503 is plausible (any daemon can hit a
//!   maintenance window) and carries no signal about why — an
//!   adversary cannot tell "duress" from "backend restart".
//!
//! ## Why a helper crate instead of inline ifs
//!
//! Concentrating the check in one module means the audit / review
//! pass only has to look at a single source to verify that every
//! duress-sensitive route is gated consistently. Adding a new
//! route in a future sprint that forgets the check is a green
//! flag for a code reviewer: "why isn't this routed through
//! `noop_identity` ?"
//!
//! All three helpers are pure (no I/O, no async), cheap (single
//! `match` on an `IdentityMode`) and trivially unit-testable.

use nexus_core_rs::IdentityMode;

/// Outcome of `gossip_publish_in_duress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishOutcome {
    /// Normal identity — the caller must invoke the real
    /// `TopicSender::broadcast` on the gossip client.
    Proceed,
    /// Duress identity — the caller must skip the broadcast and
    /// reply 200 with `{published: false}` so the UI does not
    /// flag the request as an error.
    Noop,
}

/// Outcome of `curator_subscribe_in_duress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeOutcome {
    /// Normal identity — the caller must add the pubkey to the
    /// attention set and fetch the list.
    Proceed,
    /// Duress identity — the caller must accept the HTTP request
    /// and reply 200, but must NOT mutate the attention set nor
    /// fetch a list under the fake keypair.
    Noop,
}

/// Outcome of `task_dispatch_in_duress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// Normal identity — proceed with the blob ingestion.
    Proceed,
    /// Duress identity — return 503 with a plausible maintenance
    /// message. Does not carry information about why.
    Reject503,
}

/// Gate a gossip publish on the current identity mode.
pub fn gossip_publish_in_duress(mode: IdentityMode) -> PublishOutcome {
    match mode {
        IdentityMode::Normal => PublishOutcome::Proceed,
        IdentityMode::Duress => PublishOutcome::Noop,
    }
}

/// Gate a curator subscribe on the current identity mode.
pub fn curator_subscribe_in_duress(mode: IdentityMode) -> SubscribeOutcome {
    match mode {
        IdentityMode::Normal => SubscribeOutcome::Proceed,
        IdentityMode::Duress => SubscribeOutcome::Noop,
    }
}

/// Gate a task / blob dispatch on the current identity mode.
pub fn task_dispatch_in_duress(mode: IdentityMode) -> DispatchOutcome {
    match mode {
        IdentityMode::Normal => DispatchOutcome::Proceed,
        IdentityMode::Duress => DispatchOutcome::Reject503,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_mode_always_proceeds() {
        assert_eq!(
            gossip_publish_in_duress(IdentityMode::Normal),
            PublishOutcome::Proceed
        );
        assert_eq!(
            curator_subscribe_in_duress(IdentityMode::Normal),
            SubscribeOutcome::Proceed
        );
        assert_eq!(
            task_dispatch_in_duress(IdentityMode::Normal),
            DispatchOutcome::Proceed
        );
    }

    #[test]
    fn duress_mode_noop_publishes() {
        assert_eq!(
            gossip_publish_in_duress(IdentityMode::Duress),
            PublishOutcome::Noop
        );
    }

    #[test]
    fn duress_mode_noop_subscribes() {
        assert_eq!(
            curator_subscribe_in_duress(IdentityMode::Duress),
            SubscribeOutcome::Noop
        );
    }

    #[test]
    fn duress_mode_rejects_dispatch() {
        assert_eq!(
            task_dispatch_in_duress(IdentityMode::Duress),
            DispatchOutcome::Reject503
        );
    }
}
