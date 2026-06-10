CONFIRMED fetch_hash_multi rejects empty provider lists, truncates to MAX_FETCH_PROVIDERS, and downloads the requested bare hash through the iroh downloader; targeted core tests passed: crates/nexus-core-rs/src/blobs.rs:237
CONFIRMED multi-provider fallback and lacks-hash integrity are covered by live iroh tests that passed: crates/nexus-core-rs/src/blobs.rs:466
CONFIRMED SeedRegistry clamps future seen_at to local now, normalizes project/hash/seeder keys, and sweeps on receive-clock time: crates/nexus-shell-daemon/src/seed_registry.rs:142
CONFIRMED SeedRegistry enforces both bucket and per-bucket caps with stale-newcomer drop; registry tests passed: crates/nexus-shell-daemon/src/seed_registry.rs:155
CONFIRMED seeders_recent is production-visible, TTL-purges, normalizes reads, and returns deterministic provider ids: crates/nexus-shell-daemon/src/seed_registry.rs:315
CONFIRMED feed ingest validates the op before registry insertion and passes both feed timestamp and receive clock to record_announced: crates/nexus-shell-daemon/src/feed_sync.rs:271
CONFIRMED /api/daemon/nodes is additive and sits under the authenticated router layer: crates/nexus-shell-daemon/src/http.rs:298
CONFIRMED BrowseEntry.node_id remains #[serde(skip)], preserving /browse wire shape: crates/nexus-shell-daemon-core/src/browse.rs:204
CONFIRMED directory_pull_providers is anchor-first, deduped, self-excluding, malformed-id tolerant, and capped: crates/nexus-shell-daemon/src/http.rs:1475
CONFIRMED directory-only voluntary seed uses the Multi plan with timeout, h == want_hash guard, keep_online row, and best-effort SeedAnnounced; E2E test passed: crates/nexus-shell-daemon/src/http.rs:1662
CONFIRMED directory-only blob_serve resolves only directory-advertised hashes, timeout-bounds fetch_hash_multi, then reads back by the requested hash: crates/nexus-shell-daemon/src/http.rs:2144
CONFIRMED web diff is empty and no version/domain/proof-age drift was present in git diff -- crates web; the only new route is additive: crates/nexus-shell-daemon/src/http.rs:298
OVERALL: PASS
