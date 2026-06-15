CONFIRMED API schemas/routes match Phase F: non-strict rows, strict `{nodes}` envelope, `listNodes`, `addAnchor`, optional `archive_hash`, nullable required `self_pin_enabled`; web/src/api/daemon.ts:448
CONFIRMED `/nodes` lists announced nodes plus subscribed-but-waiting anchors and cold-start AddAnchor CTA; web/src/pages/Nodes.tsx:133
CONFIRMED `AddAnchorDialog` starts inert/empty, validates 64-char lowercase hex after normalization, posts subscribe alias, invalidates nodes/curators/browse, and renders daemon failure reason; web/src/components/AddAnchorDialog.tsx:43
CONFIRMED `/nodes` and `/node/:nodeId` lazy routes are registered; web/src/App.tsx:68
CONFIRMED `/browse` remains additive with a “Parcourir par noeud” link while preserving the grid render path; web/src/pages/Browse.tsx:101
CONFIRMED `NodeCatalog` labels the node as discovery source, not authority, and not-found returns to `/nodes`; web/src/pages/NodeCatalog.tsx:137
CONFIRMED `NodeCatalog` publisher badge lookup is exact project/hash and excludes nodedirectory rows, preventing cross-version badge leakage; web/src/pages/NodeCatalog.tsx:253
CONFIRMED `NodeCatalog` Q7 probe is scoped to the selected anchor row and the exact catalog `archive_hash`; web/src/pages/NodeCatalog.tsx:263
CONFIRMED `NodeCatalog` “Ouvrir”, “Provenance”, and “Garder en ligne” use the expected project/version surfaces; web/src/pages/NodeCatalog.tsx:317
CONFIRMED `AvailabilitySheet` reconciles keep-online via `keepOnlineEcho ?? self_pin_enabled ?? true` and support via `supportEcho || selfSeeding`; web/src/components/AvailabilitySheet.tsx:123
CONFIRMED `AvailabilitySheet` posts `seedVoluntary` with `entry.archive_hash` and resets session echoes on project/hash changes; web/src/components/AvailabilitySheet.tsx:164
CONFIRMED Q7 copy renders only for offline rows with peers and keeps plain offline copy for zero peers; web/src/components/AvailabilitySheet.tsx:258
CONFIRMED Rust seed request accepts optional defaulted `archive_hash` and narrows direct/directory selection by requested version while preserving 400/404 branches; crates/nexus-shell-daemon/src/http.rs:1921
CONFIRMED directory resolver accepts `want_hash`, boot passes `None`, and seed voluntary passes the request hash; crates/nexus-shell-daemon/src/http.rs:1539
CONFIRMED seed-count returns raw tri-state `self_pin_enabled` before row-absent default collapse and keeps `self_seeding` version-scoped; crates/nexus-shell-daemon/src/http.rs:2169
CONFIRMED Browse status wire enum remains three-valued with no reachable-via-seeder variant; web/src/api/daemon.ts:128
CONFIRMED backend browse status enum also remains three-valued; crates/nexus-shell-daemon-core/src/browse.rs:148
CONFIRMED no host/target/publish-destination semantics are present in AddAnchor; web/src/components/__tests__/AddAnchorDialog.test.tsx:67
CONFIRMED web tests cover node lists, AddAnchor, catalog provenance/Q7/version seed body, AvailabilitySheet WEB-1, and Browse cohabitation; web/src/pages/__tests__/NodeCatalog.test.tsx:398
CONFIRMED targeted Rust tests cover version discriminator rejection, hash-aware directory resolution, seed-count tri-state intent, and reachable-via-seeder seed-count shape; crates/nexus-shell-daemon/src/http.rs:4904
CONFIRMED `npm --prefix web run test:unit` and targeted Rust tests passed locally; web/src/api/__tests__/daemon.test.ts:811
OVERALL: PASS
