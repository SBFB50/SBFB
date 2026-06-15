HELD-HASH-SPOOF: CONFIRMED FIXED

Reasoning: the live gossip receive branch now drops a parsed `ProjectAnnouncement` before `handle_project_announcement` when `ann.node_id == node.node_id()` (`crates/nexus-shell-daemon/src/runtime.rs:1534`, `:1546`, `:1902`). That prevents a remote self-node_id spoof from entering `BrowseAggregator::direct_entries`, so it cannot reach `own_entries` (`crates/nexus-shell-daemon-core/src/browse.rs:596`) or the signed directory. Legit remote announcements still fall through to the handler (`runtime.rs:1551`). Own apps are not blocked: deploy/publish direct-add writes local entries directly (`crates/nexus-shell-daemon/src/deploy.rs:698`), and boot restore calls the handler directly from trusted outbox bytes (`runtime.rs:2071`).

The 256 cap is effective: `publish_directory` breaks before `NODE_DIRECTORY_MAX_ENTRIES` (`crates/nexus-shell-daemon/src/http.rs:1112`), while signer/verifier reject only `> 256` (`crates/nexus-core-rs/src/node_directory.rs:236`, `:276`). Blob-presence defense still excludes unheld spoof entries (`http.rs:1135`). Archive hashes remain exact 64 lowercase hex at sign/verify (`node_directory.rs:305`, `:320`) and are skipped, not truncated, at authoring (`http.rs:1119`).

Prior fixes still hold: revision lock (`http.rs:1293`), directory-vs-curator discriminator requires directory AND NOT curator (`crates/nexus-shell-daemon-core/src/iroh_runtime.rs:208`), shared curator ingest order/error mapping preserved (`iroh_runtime.rs:395`, `:729`), node-directory domain is disjoint (`crates/nexus-core-rs/src/canonical.rs:239`), attribution sign/verify is enforced (`node_directory.rs:230`, `:284`), duress exits before signing (`http.rs:1080`), and existing format versions are not bumped.

Tests run: `cargo test -p nexus-shell-daemon` passed; `cargo test -p nexus-core-rs node_directory` passed; daemon-core node-directory/shared-ingest targeted tests passed.

NEW FINDINGS
none

OVERALL: PASS
