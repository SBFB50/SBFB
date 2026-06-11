CONFIRMED observed registry is `Mutex<HashMap<[u8;32],u64>>` with cap/TTL/rate constants and honest PoW residual comments at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:500
CONFIRMED `record_observed_directory` enforces subscribed exclusion, TTL purge, 1/min resident-entry rate limit, bounded stalest eviction, and no cap overshoot at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:863
CONFIRMED `observed_snapshot` purges TTL, re-gates subscribed nodes, and sorts freshest-first then pubkey at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:919
CONFIRMED `subscribe()` purges the observed entry on observed-to-subscribed transition at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:683
CONFIRMED non-subscribed directory announcements record only cheap metadata before returning `NotSubscribed`; fetch remains below the return at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1146
CONFIRMED self-guard rejects announcements claiming this node_id before observed capture at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1161
CONFIRMED full ingest-path test pins no-fetch, rate-limit, and self-guard behavior at crates/nexus-shell-daemon-core/src/iroh_runtime.rs:2353
CONFIRMED `/nodes` always serializes `observed: Vec<ObservedNodeView>` and row shape is exactly `node_id,last_seen` at crates/nexus-shell-daemon/src/http.rs:1944
CONFIRMED `/nodes` producer uses `observed_snapshot(now)` and includes the key even empty through `nodes_response` at crates/nexus-shell-daemon/src/http.rs:2027
CONFIRMED `/nodes` test pins envelope key-count 2, observed row field-count 2, and empty observed serialization at crates/nexus-shell-daemon/src/http.rs:4921
CONFIRMED `from_subscribed` is catalog-backed by verified directory `(project_id, archive_hash)` pairs, skips empty hashes, and normalizes case at crates/nexus-shell-daemon/src/http.rs:941
CONFIRMED `browse_views_derives_from_subscribed` covers own, catalog-listed, spoofed subscribed-node claim, unknown, mixed-case, no-hash, and None-node_id cases at crates/nexus-shell-daemon/src/http.rs:4257
CONFIRMED Browse computes split on the deduped set, caps ambient at 24, and keeps hero on the grid only at web/src/pages/Browse.tsx:78
CONFIRMED Browse source classification matches PO C: own, curator, nodedirectory, or `from_subscribed`; everything else is ambient at web/src/pages/Browse.tsx:183
CONFIRMED dedupe merge ORs `is_own` and full my-sources classification into `from_subscribed` at web/src/pages/Browse.tsx:249
CONFIRMED Browse tests pin ambient-only section, empty-section omission, cap counter, and dedupe OR behavior at web/src/pages/__tests__/Browse.test.tsx:292
CONFIRMED Nodes observed UI counts observed as non-empty, hides `nodes-list` when only observed rows exist, and opens subscribe CTA via explicit click at web/src/pages/Nodes.tsx:167
CONFIRMED AddAnchorDialog prefill is additive and state-initial only via `initialPubkey`; manual open remains empty through key remount/reset at web/src/components/AddAnchorDialog.tsx:47
CONFIRMED Nodes tests pin observed CTA prefill, manual reopen empty field, observed-only non-cold-start, and Zod tolerance at web/src/pages/__tests__/Nodes.test.tsx:194
CONFIRMED Zod keeps `BrowseEntrySchema.strict()` with optional `from_subscribed`, observed row tolerant, and `NodesResponseSchema.strict()` with optional `observed` at web/src/api/daemon.ts:152
CONFIRMED THREAT_MODEL §15.1 honestly documents payload-unbound PoW, claimed-identity rate limit, resident-entry limiter state, and S76 publisher-binding residual at docs/security/THREAT_MODEL.md:884
CONFIRMED THREAT_MODEL documents catalog-backed mitigation for unsigned `ProjectAnnouncement.node_id` spoofing at docs/security/THREAT_MODEL.md:885
CONFIRMED process review doc does not claim final PASS before this gate; it remains PASS-PENDING with Codex reconciliation open at .planning/active/post_s75_ux_arrival_review.md:10
OVERALL: PASS
