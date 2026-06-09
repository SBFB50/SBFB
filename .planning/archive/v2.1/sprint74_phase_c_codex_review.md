CONFIRMED crates/nexus-coordinator-rs/src/public_feed.rs:20 - `FEED_FORMAT_VERSION` remains `1`; no feed wire bump.

CONFIRMED crates/nexus-shell-daemon-core/src/publish.rs:24 - `PROJECT_ANNOUNCEMENT_VERSION` remains `1`; no project announcement wire bump.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:383 - `/api/v1/deploy-workspace` is registered in `authed_routes`, which receives `auth_required` at line 457.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:383 - `/api/v1/deploy-workspace` has `DefaultBodyLimit::max(MAX_DEPLOY_BYTES)`, closing the 2 MB axum default gap.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:243 - `deploy_workspace` still enforces the 100 MB handler ceiling before finalizing.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:254 - workspace upload must be a valid zip with root `index.html`.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:260 - lineage `repo_url` is normalized and rejected unless it starts with `https://`.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:274 - lineage `commit_sha` is rejected unless it is full 40-hex.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:320 - workspace redeploy hard-codes `is_open_source: false`; local bytes cannot self-upgrade to verified source.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:381 - `finalize_deploy` signs fresh provenance with `state.node_id` and `state.pow_keypair`.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:393 - contributor attestation is gated on `params.is_open_source`, so workspace redeploy never feeds the Sybil registry.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:473 - `ReleasePublished` feed mirroring is gated on `params.is_open_source`, so empty local lineage is not silently dropped.

CONFIRMED crates/nexus-shell-daemon/src/deploy.rs:450 - `deploy_from_repo` still publishes through canonical `publish_announcement` with the same project/app/provenance params.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:7033 - `fork_redeploy_resigns_provenance_as_local_node` proves local `node_id` and verifies the signature under the local keypair.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:7070 - fork redeploy E2E uses the real HTTP route, real browse aggregation, and real search path.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:7129 - gossip test decodes a real PoW envelope and asserts per-app `project_id` on the wire.

CONFIRMED crates/nexus-shell-daemon/src/http.rs:7184 - lineage test proves valid https `repo_url` plus 40-hex SHA still stays `is_open_source=false`.

CONFIRMED crates/nexus-shell-daemon-core/src/blob_serve.rs:128 - zip entry allocation is capped by remaining budget and read through `take(remaining + 1)`.

CONFIRMED crates/sbfb-factory/src/fork.rs:138 - `fork_from_search_hit` now verifies `blake3(blob_bytes) == archive_hash` before writing archive workspaces.

CONFIRMED crates/sbfb-factory/src/main.rs:75 - CLI exposes `--archive-hash`, and `atelier::fork` passes it into the triplet at atelier.rs:42.

CONFIRMED crates/sbfb-factory/src/atelier.rs:48 - sync CLI tokio usage is a one-shot current-thread runtime around the async fork primitive.

CONFIRMED crates/sbfb-factory/src/atelier.rs:100 - redeploy forwards `repo_url` only as lineage and intentionally omits `commit_sha`.

CONFIRMED crates/sbfb-factory/src/atelier.rs:155 - `zip_workspace` walks without following links, skips `.git`, skips symlinks, and uses relative archive names.

CONFIRMED crates/sbfb-factory/src/templates/react/index.html:12 - React runtime scripts are relative same-origin assets, not CDN URLs.

CONFIRMED crates/sbfb-factory/src/templates/react/react.production.min.js:2 - React UMD license header is preserved.

CONFIRMED crates/sbfb-factory/src/templates/react/htm.umd.js:1 - htm now has an Apache-2.0 license header.

CONFIRMED crates/sbfb-factory/src/templates/pyodide/index.html:16 - Pyodide scaffold explicitly states it does not run under the current sandbox.

CONFIRMED web/src/pages/BrowsedProject.tsx:445 - Source anchor is rendered only for `https://` `repo_url`, closing the immediate XSS vector.

CONFIRMED web/src/pages/BrowsedProject.tsx:458 - Fork CTA is an intention button opening an explanatory panel, not a fake one-click fork.

CONFIRMED web/src/pages/__tests__/BrowsedProject.test.tsx:596 - frontend tests cover the https-source fork CTA panel flow.

CONFIRMED web/src/pages/__tests__/BrowsedProject.test.tsx:654 - frontend tests cover non-https `repo_url` CTA suppression.

Overall verdict: PASS for Sprint 74 Phase C pre-commit review; no GAP found.
