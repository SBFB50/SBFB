// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shell daemon state snapshot exposed via HTTP `/info`.
//!
//! The shell (via the coordinator proxy) polls the daemon's
//! `/info` endpoint to render the Browse / Curators page
//! headers. The body is a [`DaemonStateSnapshot`] serialized as
//! JSON. The schema is **frozen** in Phase A so Phase C/D can
//! populate the currently-empty fields without bumping the
//! version.
//!
//! ## Additive evolution
//!
//! - The base snapshot carries: `schema_version`, `node_id`,
//!   `daemon_version`, `uptime_secs`, `started_at`,
//!   `last_updated_at`, `api_host`, `api_port`.
//! - `subscribed_curators`, `known_lists`, `known_browse_entries`
//!   are additive fields the snapshot can carry once a feeder
//!   supplies them; they default to empty/zero otherwise.
//!
//! All such fields are optional keys with `#[serde(default)]` so an
//! older daemon's `/info` response stays parseable by a newer shell.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// On-wire schema version for [`DaemonStateSnapshot`]. Must be
/// bumped on any breaking change (field rename, field removal,
/// type change). Additive changes may keep the same value.
pub const SCHEMA_VERSION: u32 = 1;

/// The complete `/info` response body.
///
/// Built by the binary's [`crate::state::DaemonStateSnapshot::from_inputs`]
/// helper from a [`StateInputs`] struct that the HTTP handler
/// fills in from its shared [`std::sync::Arc`] state. The shape
/// is frozen at schema_version=1 — Phase C and later append new
/// optional fields without bumping the version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonStateSnapshot {
    /// Always [`SCHEMA_VERSION`].
    pub schema_version: u32,

    /// Ed25519 public key hex (64 lowercase chars) of the running
    /// iroh endpoint. Empty string only during the brief
    /// bootstrap window before `create_node()` returns.
    pub node_id: String,

    /// Compile-time version of `nexus-shell-daemon-core` (and,
    /// by convention, of the `nexus-shell-daemon` binary).
    /// The shell compares this against its own compiled-in
    /// schema version to detect a mismatched daemon.
    pub daemon_version: String,

    /// Wall-clock time since `DaemonRuntime::start()`, in seconds.
    /// Derived at snapshot time from [`StateInputs::boot_time`].
    pub uptime_secs: u64,

    /// RFC 3339 UTC timestamp of the daemon's boot.
    pub started_at: String,

    /// RFC 3339 UTC timestamp refreshed on every `from_inputs`
    /// call. The shell shows this as "last refreshed" so users
    /// can tell whether the polling loop is healthy.
    pub last_updated_at: String,

    /// Host the HTTP server is bound to. `"127.0.0.1"` under the
    /// D1 loopback-only contract.
    pub api_host: String,

    /// Real port the HTTP listener bound to. Resolved from the
    /// `TcpListener::local_addr()` after `bind`, not from the
    /// config, so the ephemeral port 0 case works.
    pub api_port: u16,

    /// Curator pubkeys (hex) the daemon is currently interested
    /// in. Defaults to empty; a feeder can supply it from the
    /// `ShellDaemonRuntime`'s `HashSet<CuratorPubkey>` attention
    /// set.
    #[serde(default)]
    pub subscribed_curators: Vec<String>,

    /// Number of `CuratorListEntry` blobs the daemon has
    /// received + verified since boot. Defaults to 0; a feeder
    /// can supply it from the `DashMap<pubkey, entry>` size.
    #[serde(default)]
    pub known_lists: u32,

    /// Number of project entries cached across every known
    /// curator list. Defaults to 0; a feeder can supply it from
    /// the browse aggregator.
    #[serde(default)]
    pub known_browse_entries: u32,
}

/// Dynamic inputs the binary hands to [`DaemonStateSnapshot::from_inputs`]
/// on every HTTP `/info` request.
///
/// Kept as a plain struct rather than an ambient context on the
/// runtime so the unit tests can build snapshots from fixed
/// fixtures without spinning up a live iroh node.
pub struct StateInputs {
    pub node_id: String,
    pub daemon_version: String,
    pub boot_time: SystemTime,
    pub api_host: String,
    pub api_port: u16,
    pub subscribed_curators: Vec<String>,
    pub known_lists: u32,
    pub known_browse_entries: u32,
}

impl DaemonStateSnapshot {
    /// Build a snapshot from dynamic inputs.
    ///
    /// The function is infallible: any clock-skew edge case
    /// (boot_time in the future, negative uptime) collapses to
    /// zero seconds rather than propagating an error through
    /// the HTTP path.
    pub fn from_inputs(inputs: StateInputs) -> Self {
        let now = SystemTime::now();
        let uptime_secs = now
            .duration_since(inputs.boot_time)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let started_at = iso_utc(inputs.boot_time);
        let last_updated_at = iso_utc(now);

        Self {
            schema_version: SCHEMA_VERSION,
            node_id: inputs.node_id,
            daemon_version: inputs.daemon_version,
            uptime_secs,
            started_at,
            last_updated_at,
            api_host: inputs.api_host,
            api_port: inputs.api_port,
            subscribed_curators: inputs.subscribed_curators,
            known_lists: inputs.known_lists,
            known_browse_entries: inputs.known_browse_entries,
        }
    }
}

fn iso_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()
        .and_then(|dt| dt.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn mk_inputs() -> StateInputs {
        StateInputs {
            node_id: "deadbeef".repeat(8),
            daemon_version: crate::VERSION.to_string(),
            boot_time: SystemTime::now() - Duration::from_secs(42),
            api_host: "127.0.0.1".to_string(),
            api_port: 45678,
            subscribed_curators: Vec::new(),
            known_lists: 0,
            known_browse_entries: 0,
        }
    }

    #[test]
    fn schema_version_is_one() {
        assert_eq!(SCHEMA_VERSION, 1, "bumping this is a breaking change");
    }

    #[test]
    fn snapshot_from_inputs_has_expected_shape() {
        let snap = DaemonStateSnapshot::from_inputs(mk_inputs());
        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.node_id.len(), 64);
        assert_eq!(snap.daemon_version, crate::VERSION);
        assert!(snap.uptime_secs >= 42);
        assert!(snap.started_at.ends_with('Z'));
        assert!(snap.last_updated_at.ends_with('Z'));
        assert_eq!(snap.api_host, "127.0.0.1");
        assert_eq!(snap.api_port, 45678);
        assert!(snap.subscribed_curators.is_empty());
        assert_eq!(snap.known_lists, 0);
        assert_eq!(snap.known_browse_entries, 0);
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let snap = DaemonStateSnapshot::from_inputs(mk_inputs());
        let body = serde_json::to_string(&snap).unwrap();
        let back: DaemonStateSnapshot = serde_json::from_str(&body).unwrap();
        assert_eq!(snap, back);
    }

    #[test]
    fn snapshot_populates_curator_fields_when_present() {
        // Base inputs are all zero, but the snapshot must still
        // propagate non-zero values a feeder may supply — this is
        // the forward-compat test for the additive schema
        // evolution.
        let mut inputs = mk_inputs();
        inputs.subscribed_curators = vec!["abcd".to_string(), "1234".to_string()];
        inputs.known_lists = 2;
        inputs.known_browse_entries = 7;

        let snap = DaemonStateSnapshot::from_inputs(inputs);
        assert_eq!(snap.subscribed_curators.len(), 2);
        assert_eq!(snap.known_lists, 2);
        assert_eq!(snap.known_browse_entries, 7);
    }

    #[test]
    fn snapshot_future_boot_time_gives_zero_uptime() {
        // Clock-skew edge case: boot_time is in the future. The
        // infallible builder collapses the negative duration to
        // zero rather than panicking.
        let mut inputs = mk_inputs();
        inputs.boot_time = SystemTime::now() + Duration::from_secs(120);

        let snap = DaemonStateSnapshot::from_inputs(inputs);
        assert_eq!(snap.uptime_secs, 0);
    }
}
