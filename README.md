# SBFB -- Decentralized P2P App Distribution and Compute

**Publish apps. Distribute them peer-to-peer. Verify what can be verified. Keep the limits visible.**

SBFB is currently in **closed pilot**. The core protocol and several developer
tools are code-backed, but this is not yet a public production network.

## What Is SBFB?

SBFB is a protocol and local runtime for publishing, distributing, and opening
web applications over a peer-to-peer network. A publisher can package a web app
(plain HTML, React, Pyodide, WASM, JupyterLite, or any build that produces
static web files), attach a manifest, sign provenance, and announce the app.
Other nodes discover the announcement, fetch the content-addressed archive, and
open it inside a browser sandbox served by their own local daemon.

The second axis is **shared AI/GPU compute**. Apps can request analysis,
translation, or generation tasks from volunteer CPU/GPU workers, but only under
explicit local consent, per-project resource caps, and signed task/result
records. The worker path exists as primitives and pilot tooling; public
multi-worker scale and anti-cheat are still validation work.

The design goal is simple: no app store, no cloud account, no central authority
deciding what may exist, and no hidden trust claim. Every public claim should
map to code, a local proof, or a clearly marked future layer.

## Current Maturity

| Area | Status | What this means |
|------|--------|-----------------|
| Local daemon, shell UI, iframe sandbox, bridge | Code-backed | A node can run a local shell, open sandboxed apps, and broker app requests through `postMessage`. |
| P2P app distribution | Code-backed / pilot | iroh gossip, blobs, docs, feed sync, and blob cache exist. LAN/WAN pilot validation is narrower than a public production network. |
| Source provenance | Code-backed self-attestation | `provenance.json` and Ed25519 signatures exist. This is not a third-party audit or independent reproducible build. |
| Factory CLI | Code-backed | `create`, `validate`, `preview`, and `publish` exist. The automated publish path currently covers a subset of the full FG0-FG10 gate model. |
| Factory Viewer / Operator | Code-backed local tools | Read-only viewer and local privileged operator exist for pilot/dev workflows. |
| Local search and Proof Cards | Code-backed local features | SQLite FTS5 and deterministic Proof Cards exist. Cross-node SearchManifest discovery is future work. |
| Curator primitives | Code-backed primitives | Signed vouch/disendorse/feed operations exist. Full governance timelines, dissent UI, and social process are future work. |
| Worker compute | Code-backed primitives / pilot | Consent, caps, worker signatures, task/result records, Ollama backend, optional llama.cpp, and kudos v1 exist. Public worker quorum and exact model/GPU proof are not complete. |
| Installers | Built and internally validated | Windows NSIS, Linux `.deb`, and macOS `.dmg` tooling exists. Closed pilot distribution uses maintainer-provided binaries and hashes until public release assets are finalized. |
| Gate 1 / CHATONS pilot | In progress | The pilot protocol exists for 2-3 testers. 24h stability, public release assets, and external feedback still determine the next gate. |

Not ready today: production CHATONS hosting, sensitive user data, public
multi-node worker quorum, independent security audit, third-party reproducible
builds, SearchManifest network discovery, and a full governance UI.

## Architecture

```
                          +------------------+
                          |  P2P Network     |
                          |  iroh gossip     |
                          |  iroh-blobs      |
                          |  iroh-docs       |
                          +--------+---------+
                                   |
                            QUIC / Ed25519
                                   |
                          +--------+---------+
                          |  Shell Daemon    |
                          |  Rust binary     |
                          |  local HTTP      |
                          |  bearer + Host   |
                          |  + Origin auth   |
                          +--------+---------+
                                   |
                         default: 127.0.0.1
                                   |
               +-------------------+-------------------+
               |                                       |
    +----------+----------+             +--------------+---------+
    |  Shell UI (React)   |             |  Blob-serve route      |
    |  Browse, Curators,  |             |  decompresses app zip  |
    |  Network, Deploy    |             |  LRU cache + CSP       |
    +----------+----------+             +--------------+---------+
               |                                       |
               |            postMessage bridge          |
               +---------------+-----------------------+
                               |
                    +----------+----------+
                    |  App iframe          |
                    |  sandbox="allow-     |
                    |  scripts"            |
                    |  no allow-same-      |
                    |  origin              |
                    |  CSP connect-src     |
                    |  'none'              |
                    +---------------------+
```

Blob-served app content is protected by the browser sandbox and CSP. It is not
currently a separate TCP origin; the iframe gets an opaque origin because
`allow-same-origin` is deliberately absent.

## Key Features

- **Peer-to-peer distribution** -- app archives propagate through gossip and
  iroh-blobs as BLAKE3-addressed content. No central host is required for the
  archive itself.
- **Source-verifiable publish path** -- deploy-from-repo writes a
  `provenance.json` self-attestation with Ed25519 signatures and source
  metadata. This proves a signed local publication path, not that an
  independent third party rebuilt the same archive.
- **Browser sandbox** -- app iframes run with `sandbox="allow-scripts"` and no
  `allow-same-origin`; untrusted content receives CSP including
  `connect-src 'none'`.
- **Bridge API** -- apps use `sbfb-bridge.js` and `window.postMessage` with
  UUID correlation. The host validates requests against a global method
  allowlist and source iframe. Per-app runtime enforcement of declared
  `SBFB.json bridge.methods` is still a hardening item.
- **Curator lists instead of central moderation** -- communities can sign
  recommendations or warnings. Users choose curators; no curator can delete an
  archive from the network.
- **App storage** -- the storage bridge supports app namespaces. Some
  namespaces, currently including `sbfb-ideas`, use iroh-docs replication;
  other app storage may remain local until explicitly wired.
- **Local full-text search** -- SQLite FTS5 indexes local browse/feed material
  and returns BM25-ranked results. SearchManifest-based cross-node discovery is
  future work.
- **Proof Cards** -- local deterministic evidence scores (0-100) summarize
  provenance, feed, curator, license, and freshness signals. A Proof Card is an
  explanation surface, not a signed security certificate.
- **Distributed compute primitives** -- workers enforce 4-level consent,
  watts/VRAM/hour caps, task signatures, result signatures, and kudos credit.
  Public worker quorum and exact model/GPU proof are not complete.
- **Kudos v1** -- compute/task contributions use a non-monetary,
  non-transferable ledger with logarithmic returns, EMA decay, BLAKE3
  hash-chain, and fairness metrics. Multi-family kudos and governance weighting
  are design work.
- **Append-only public feed** -- feed entries are Ed25519-signed and BLAKE3
  hash-chained per author. Nodes verify the history they have observed; global
  completeness across all peers remains a network/pilot question.
- **Key rotation** -- Ed25519 rotation announcements exist with a 7-day default
  transition window; other windows can be encoded by the announcement.
- **Cross-platform packaging** -- cargo-packager config exists for Windows
  NSIS, Linux `.deb`, and macOS `.dmg`. Public asset distribution is separate
  from the source tree.
- **AGPL-3.0-or-later** -- protocol, daemon, worker, and tooling remain free
  software. Apps published on the network keep their own licenses.

## How It Works

**1. Create** -- `sbfb-factory create` scaffolds an app with `SBFB.json`,
`index.html`, a template lock, and the bridge SDK.

**2. Develop** -- The app can be plain static HTML or built from any toolchain
that outputs browser files. `sbfb-factory validate` checks the manifest and
basic workspace safety.

**3. Preview** -- `sbfb-factory preview` zips the local project, sends it to the
local daemon preview store, and returns a blob-serve URL with a 30-minute TTL.

**4. Publish** -- `sbfb-factory publish --repo-url ...` talks to the local
daemon, runs the currently automated publish checks, deploys from the source
repository, and verifies the returned provenance. The full FG0-FG10 model is
documented as the target process; the code path today automates FG4, FG5, FG6,
deploy/publish, and FG8, with separate commands for preview and checks.

**5. Distribute** -- Other nodes discover the app through feed/gossip material,
fetch the archive through iroh-blobs, verify the available hashes/signatures,
and cache it locally.

**6. Open** -- Users open the app in the shell UI. The daemon serves app files
through blob-serve into a sandboxed iframe. The app can only reach SBFB
capabilities through the bridge methods exposed by the host.

## Quick Start

Closed pilot users receive binaries and hashes directly from the maintainer.
Public release asset links will be added when the pilot boundary is lifted.

From source:

```bash
# Build the launcher and daemon.
cargo build -p nexus-launcher -p nexus-shell-daemon --release

# Run the launcher (on Windows use target/release/nexus-launcher.exe).
./target/release/nexus-launcher
```

The launcher/daemon create local identity and auth material under the SBFB home
directory (`~/.sbfb` by default, or `SBFB_HOME` when set). Some launcher and
worker state uses platform data directories or `.nexus-grid` paths.

The daemon binds to loopback by default and authenticated HTTP routes require a
256-bit bearer token plus Host/Origin checks. The config comments require
loopback; non-loopback binds should be treated as a security bug, not a
supported production mode.

## Publishing An App

The Factory CLI is part of the source tree. It is not currently one of the
binary artifacts attached by the release workflow, so build it from source for
pilot work:

```bash
cargo build -p sbfb-factory --release

# Or install it into Cargo's bin directory.
cargo install --path crates/sbfb-factory
```

Example:

```bash
# Scaffold a static app.
sbfb-factory create myapp
cd myapp/

# Edit index.html, then validate.
sbfb-factory validate .

# Preview in the daemon sandbox.
sbfb-factory preview .

# Publish from a public source repository.
sbfb-factory publish . --repo-url https://github.com/<you>/<repo>
```

Factory gate status:

- Spec/process: FG0-FG10 are documented in `docs/factory/FACTORY_GATES.md`.
- Automated today: manifest pre-validation plus FG4 diff, FG5 sandbox/path
  checks, FG6 secret/dependency scan, deploy/publish, and FG8 provenance check.
- Separate command: FG7 preview readiness can be run through the preview check.
- Future/process: FG0-FG3 classification/scope/template/manifest depth and
  FG10 curator review are not a single fully enforced publish pipeline yet.

## SBFB.json Manifest

Every app carries a `SBFB.json` manifest:

```json
{
  "schema_version": 2,
  "name": "my-app",
  "display_name": "My App",
  "description": "What this app does",
  "category": "tools",
  "license": "AGPL-3.0-or-later",
  "lang": "en",
  "bridge": {
    "methods": ["storage_get", "storage_set", "identity_pubkey"]
  },
  "tech": {
    "type": "static-html",
    "build_command": null
  }
}
```

The Rust parser is forward-compatible: unknown fields are ignored and older or
newer schema versions can be accepted. That is intentional for network
evolution, but it also means README examples may include fields that are
documented for the protocol before every field is semantically enforced by the
current runtime.

## Example Apps

| App | Description | Bridge methods used |
|-----|-------------|---------------------|
| **Protocol Explorer** (`examples/sbfb-explorer/`) | Interactive protocol documentation with live network status and provenance demos. Pure HTML/JS. | `node_status`, `identity_pubkey`, `browse_list`, `provenance_verify` |
| **Ideas Hub** (`examples/sbfb-ideas/`) | Decentralized idea board with voting. This is the clearest current example of replicated app storage. Pure HTML/JS. | `storage_get`, `storage_set`, `storage_list`, `storage_delete`, `identity_pubkey` |
| **Factory Viewer** (`examples/sbfb-factory-viewer/`) | Read-only sandboxed app for browsing network apps with search and Proof Cards. | `browse_list`, `search`, `proof_card_get`, `storage_get` |

These are demonstration apps. They prove the bridge and sandbox model, but they
are not a complete app marketplace.

## Bridge SDK

Apps in iframes communicate with the host through `sbfb-bridge.js`, a small
dependency-free JavaScript library:

```html
<script src="/sbfb-bridge.js"></script>
<script>
  const bridge = new SBFBBridge({ timeout: 10000 });

  // Storage. Replication depends on the app namespace and daemon wiring.
  await bridge.setStorage("my-key", { value: 42 });
  const data = await bridge.getStorage("my-key");
  const all = await bridge.listStorage("prefix/");
  await bridge.deleteStorage("old-key");

  // Network introspection.
  const status = await bridge.getNodeStatus();
  const apps = await bridge.getBrowseList();

  // Provenance and evidence views.
  const proof = await bridge.verifyRelease(projectId);
  const card = await bridge.getProofCard(projectId);

  // Local full-text search.
  const results = await bridge.search("translation app", { limit: 20 });

  // Identity.
  const me = await bridge.getIdentityPubkey();

  // Compute task submission. Payload shape follows the coordinator task API
  // and pilot apps should avoid sending personal data.
  const redacted = await bridge.piiRedact(userInput);
  await bridge.submitTask({ prompt: redacted.redacted_text });

  // Poll-based live storage notifications.
  const unsubscribe = bridge.onStorageUpdate("my-app", () => refresh());
</script>
```

Every request has a UUID correlation ID and timeout. The host validates message
shape and iframe source before dispatch. The bridge surface is broader than the
current manifest parser in a few places; keep `SBFB.json bridge.methods` as the
declared intent, not as proof that every runtime call is per-app gated today.

## GPU And CPU Compute

A node can contribute GPU or CPU compute through `nexus-worker`. The current
worker stack supports:

- Ollama over HTTP as the default backend.
- Optional embedded llama.cpp when built with the `llm_llama_cpp` feature and a
  valid GGUF model path.
- Optional GPU ephemeral behavior such as VRAM wipe when the relevant feature
  and runtime prerequisites are enabled.
- 4 consent levels:
  - **L1** -- my projects only (default).
  - **L2** -- projects marked open-source/provenance-present.
  - **L3** -- manual allowlist.
  - **L4** -- all public projects.
- Per-project caps on watts, VRAM, and daily hours.
- Signed task, claim, and result records.
- Kudos credit after validation.

Important limits:

- Workers see task prompts in clear text. Apps should redact PII before
  submission.
- The validator checks signatures, task state, and quorum paths where enabled;
  it does not yet prove that a specific model file ran on a specific GPU.
- `model_digest`, `logprobs_hash`, and `output_token_ids` are useful signals,
  not a complete anti-cheat proof. Today, `model_digest` can be derived from
  the model name and `logprobs_hash` can be empty/zero depending on backend.
- SHA256 quorum exists as a foundation for build/redundancy paths; public
  multi-worker quorum at scale is still a pilot/future item.

## Security Model

The threat model follows STRIDE + LINDDUN. Current defense layers:

1. **Iframe sandbox** -- `sandbox="allow-scripts"` without
   `allow-same-origin`; app JavaScript cannot access shell DOM, cookies, or
   localStorage.
2. **Content Security Policy** -- untrusted app content receives strict CSP,
   including `connect-src 'none'` and sandbox directives.
3. **postMessage bridge** -- UUID correlation, typed request schema, global
   method allowlist, and iframe source validation.
4. **Loopback HTTP auth** -- 256-bit bearer token, Host allowlist, and Origin
   checks for browser/HTTP calls.
5. **Local IPC auth** -- Unix SO_PEERCRED and Windows Named Pipe DACLs are used
   for local non-browser clients when those transports are available.
6. **Provenance and feed integrity** -- Ed25519 signatures, BLAKE3 content
   hashes, and append-only hash-chain feed entries.

Additional protections include Hashcash anti-spam, GCRA rate limiting, age
witness primitives, iframe watchdog pings, and launcher/keystore paths for
encrypted key material. Direct daemon paths may still rely on user-only file
permissions for some secrets, so key encryption at rest should not be described
as universal until every launch path is covered.

Blob-serve is intentionally reachable without the normal bearer header because
iframe navigation cannot attach custom headers. The compensating controls are
content addressing, blob tickets where available, path traversal rejection, zip
validation, iframe sandboxing, and CSP.

Full threat model: [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md)

## Trust Levels

Six cumulative trust levels describe what a node can currently know about an
app. They are evidence labels, not endorsements.

| Level | Label | What it means |
|-------|-------|---------------|
| **N0** | Upload direct | An archive exists on the network. Origin unknown. |
| **N1** | Source readable | A public source repository is declared. |
| **N2** | Provenance attested | A local Ed25519 self-attestation links source metadata to an artifact hash. |
| **N3** | Signature verified | The local daemon verified the provenance signature and proof-chain fields it has. |
| **N4** | Reproducible build | A third party rebuilt from source and got the same hash. Future. |
| **N5** | Feed verified | The observed publication history is hash-chain integral. Global completeness still depends on network sync and cross-peer validation. |

Details: [`docs/trust/TRUST_TAXONOMY.md`](docs/trust/TRUST_TAXONOMY.md)

## Factory, RRV, And Babel

The repo now separates protocol primitives from product experiments:

- **Factory** is the app creation/publish toolchain. It has a CLI, read-only
  Factory Viewer app, local privileged Factory Operator, manifest generation,
  preview, secret scanning, sandbox checks, and provenance hooks.
- **RRV** means "Recherche Reseau Verifiable": a search/explanation layer that
  can inspect protocol facts, app metadata, proofs, and eventually source/code
  evidence. The local FTS5 and Proof Card pieces exist; SearchManifest and
  broader `@dev`/`@web` modes are future work.
- **Babel** is the intended dogfood/canary app for Factory and RRV. It is an
  application built on the protocol, not part of the protocol itself. The MVP
  should be a static reader with public-domain fixtures before any live
  translation network is treated as mature.

## Project Structure

```
sbfb/
+-- Cargo.toml                          # Rust workspace
+-- crates/
|   +-- nexus-core-rs/                  # iroh wrapper, crypto, canonical bytes
|   +-- nexus-coordinator-rs/           # DB, dispatcher, validator, kudos, search
|   +-- nexus-shell-daemon-core/        # shared daemon auth/config/blob helpers
|   +-- nexus-shell-daemon/             # local daemon: HTTP, blob-serve, feed, gossip
|   +-- nexus-launcher/                 # launcher: spawn daemon, token, browser
|   +-- nexus-worker-core/              # worker engine, consent, LLM backends
|   +-- nexus-worker/                   # worker binary
|   +-- nexus-events-core/              # security event writers
|   +-- nexus-executor/                 # build executor experiments
|   +-- nexus-trace-core/               # OpenTelemetry tracing helpers
|   +-- nexus-test-harness/             # integration test helpers
|   +-- sbfb-manifest/                  # SBFB.json parser shared by daemon/factory
|   +-- sbfb-factory/                   # Factory CLI/process/operator server
+-- web/                                # Shell UI (React + TypeScript + Vite)
+-- tools/
|   +-- factory-operator/               # Local management tool
|   +-- factory-ui/                     # Shared readonly/operator UI components
+-- examples/
|   +-- sbfb-explorer/                  # Protocol Explorer app
|   +-- sbfb-ideas/                     # Ideas Hub app
|   +-- sbfb-factory-viewer/            # Factory Viewer app
+-- docs/
|   +-- security/THREAT_MODEL.md        # STRIDE + LINDDUN threat model
|   +-- trust/TRUST_TAXONOMY.md         # N0-N5 trust levels
|   +-- factory/FACTORY_GATES.md        # FG0-FG10 target gate model
|   +-- release/GATE1_TEST_PROTOCOL.md  # Closed pilot procedures
+-- prompts/agent/                      # Portable agent prompts
+-- .planning/                          # Sprint plans, verification, research
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Protocol / daemon** | Rust 2021, iroh 0.98, iroh-blobs 0.100, iroh-docs |
| **Crypto** | Ed25519, BLAKE3, ChaCha20-Poly1305 for QUIC, Argon2id, AES-256-GCM, FROST primitives |
| **Storage** | SQLite, local app storage, selected iroh-docs replicated namespaces |
| **Search** | SQLite FTS5 with BM25 ranking |
| **LLM inference** | Ollama by default; llama.cpp behind feature flags |
| **Frontend** | React 19, TypeScript, Tailwind CSS, shadcn/ui, Zustand, React Query, Vite |
| **CI** | Self-hosted Woodpecker (`ci.sbfb.world`) plus GitHub Actions |
| **Installers** | cargo-packager config for NSIS, `.deb`, AppImage, and `.dmg` |

## Tests

Latest repo-recorded verification:

- **1486 Rust tests** in the workspace nextest suite.
- **279 Vitest tests** for frontend, bridge, and API schemas.
- **6/6 size-limit checks**.

Core commands:

```bash
# Rust format, lint, and tests.
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc

# Release builds.
cargo build -p nexus-shell-daemon --release
cargo build -p sbfb-factory --release

# Frontend.
cd web
npm install
npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run test:unit
npm run build
npm run size
```

## License

[AGPL-3.0-or-later](https://www.gnu.org/licenses/agpl-3.0.html)

The SBFB protocol, daemon, worker, and tooling are licensed under
AGPL-3.0-or-later. Modifications to these network services must keep source
available under the license.

Apps published on the SBFB network are **not** automatically AGPL. Each app
carries its own license. To be recommended as a higher-trust app, it should
publish source, declare a license, and provide verifiable provenance.

## Status

SBFB is in **closed pilot phase**.

The `v1.0` tag exists on the configured Git remotes. Public release assets,
hashes, and tester instructions are still a pilot boundary, not a broad public
distribution guarantee.

Works today:

- Local daemon, launcher, shell UI, iframe sandbox, and bridge.
- P2P app distribution primitives: gossip, blobs, feed, local cache.
- Deploy from source with Ed25519 self-attested provenance.
- Public feed, signed curator operations, endorsement/disendorsement, and
  subscriptions.
- Local FTS5 search and Proof Cards.
- Factory CLI, Factory Viewer, and Factory Operator.
- Worker consent/caps/signature/kudos primitives.
- Installer tooling for Windows/Linux/macOS.

Not ready:

- Formal external security audit of the full stack.
- Public production nodes.
- Full independent reproducible build network.
- SearchManifest cross-node search discovery.
- Full governance UI for curator timelines and dissent.
- Public worker quorum enforcement at scale.
- Complete model/GPU anti-cheat proof.
- Sensitive-data workloads.

Pilot target: 1-2 external testers willing to install nodes, publish small
non-sensitive apps, leave the daemon running, and report what breaks.

## Contributing

The project currently follows a solo-maintainer / closed-pilot model. Public
mirrors may exist, but contribution flow and issue intake are still controlled
while the pilot is active.

Before contributing:

- Read `docs/security/THREAT_MODEL.md` before touching security-sensitive code.
- Read `docs/agent/AGENT_SYSTEM.md` and `docs/agent/PROCESS.md` for workflow
  and sprint gate rules.
- Run the relevant Rust and frontend verification commands before submitting a
  change.
- Keep user-facing frontend strings in French.
- Preserve loopback, sandbox, bridge, signing, allowlist, and provenance
  invariants.

Built to stay independent: no foundation, no startup, no token, no central app
store.
