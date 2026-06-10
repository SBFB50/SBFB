CONFIRMED node-directory ingest reuses the shared SignedList gate for signature, attribution, and rollback checks at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:453
CONFIRMED directory announcements are subscription-gated before blob fetch at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:967
CONFIRMED persisted directory anchors store locator metadata, not catalog rows, at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1241
CONFIRMED boot re-pull filters to subscribed anchors before any fetch at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1058
CONFIRMED re-pulled directory blobs are revalidated through the same ingest gate at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1125
CONFIRMED daemon startup invokes directory re-pull from persisted locators at crates/nexus-shell-daemon/src/runtime.rs:1459
CONFIRMED runtime dispatch replaces the directory drop path with real directory ingest at crates/nexus-shell-daemon/src/runtime.rs:1568
CONFIRMED known_entry_count is additive and includes subscribed directory entries at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:703
CONFIRMED BrowseSource serializes node-directory rows as nodedirectory at crates/nexus-shell-daemon-core/src/browse.rs:127
CONFIRMED aggregator emits directory rows additively with anchor node_id and author archive_hash at crates/nexus-shell-daemon-core/src/browse.rs:764
CONFIRMED WIRE-1 release fields are additive/defaulted serde fields at crates/nexus-coordinator-rs/src/public_feed.rs:46
CONFIRMED deploy producer sets project_name and category in release payloads at crates/nexus-shell-daemon/src/deploy.rs:496
CONFIRMED search extracts project_name/category from release payloads at crates/nexus-coordinator-rs/src/search.rs:239
CONFIRMED WIRE-2 seed counting supports exact-version and cross-version distinct seeder modes at crates/nexus-shell-daemon/src/seed_registry.rs:155
CONFIRMED seed-count HTTP route accepts optional archive_hash and passes it to the registry at crates/nexus-shell-daemon/src/http.rs:1501
CONFIRMED DBQ-1 keep_online UPSERT preserves existing archive_hash when the new value is NULL at crates/nexus-coordinator-rs/src/db.rs:709
CONFIRMED frontend daemon schema accepts nodedirectory source rows at web/src/api/daemon.ts:136
CONFIRMED frontend seed-count requests pass archive_hash for version-aware counts at web/src/components/AvailabilitySheet.tsx:103
CONFIRMED durability test uses a fresh runtime and re-pull path, not a RAM-only tautology, at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:2026
OVERALL: PASS
