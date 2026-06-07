GAP web/src/pages/BrowsedProject.tsx:131 - ownership still uses `daemon.node_id === projectId`, so per-app self-deploys keyed by `blake3(project_name)` never show the owner keep-online toggle.

GAP crates/nexus-shell-daemon/src/http.rs:1018 - handler derives `archive_hash` only from in-memory Browse, then overwrites M18 at :1028; if Browse lacks the hash, OFF erases DB hash and ON cannot re-tag.

CONFIRMED crates/nexus-coordinator-rs/src/db.rs:261 - M18 is local additive `keep_online`; no wire constants bumped.

CONFIRMED crates/nexus-coordinator-rs/src/public_feed.rs:20 - `FEED_FORMAT_VERSION` remains `1`.

CONFIRMED crates/nexus-shell-daemon-core/src/publish.rs:24 - `PROJECT_ANNOUNCEMENT_VERSION` remains `1`.

CONFIRMED crates/nexus-shell-daemon-core/src/iroh_runtime.rs:92 - gossip `ANNOUNCEMENT_VERSION` remains `1`.

CONFIRMED crates/nexus-coordinator-rs/src/provenance.rs:15 - `PROVENANCE_SCHEMA_VERSION` remains `1`.

CONFIRMED crates/nexus-core-rs/src/blobs.rs:113 - explicit blob tag is caller-named, so per-project `keep-online/<project_id>` pins are independent.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:457 - deploy tags the stored archive blob hash `hash_hex`, not the pre-injection artifact hash.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:306 - `deploy_workspace` also reaches `finalize_deploy`, so local forks are pinned despite `is_open_source=false` at :320.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:267 - keep-online route is inside `authed_routes` and gets `auth_required` at :459.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:1818 - keep_online DB/lock failures fall back to empty disabled set, warning and replaying all.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:1786 - empty disabled set fast-path replays all without decode.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:1789 - undecodable envelopes replay, never silently drop.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:1796 - per-app `project_id` is used, with legacy empty fallback to `node_id`.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:1468 - OFF gate covers browse_request replay, NeighborUp at :1500, and periodic republish at :1571.

CONFIRMED crates/nexus-shell-daemon/src/runtime.rs:811 - boot `rebuild_from_feed` failure is now logged as `error!`.

CONFIRMED web/src/api/daemon.ts:314 - `setKeepOnline` response schema is `.strict()`.

CONFIRMED web/src/components/AvailabilitySheet.tsx:235 - toggle is real and disabled only while pending at :237; no fetch-on-mount path exists.

Overall verdict: GAP, because Phase D backend policy is mostly correct but the frontend owner gate prevents the delivered toggle from appearing for current self-deployed per-app IDs.
