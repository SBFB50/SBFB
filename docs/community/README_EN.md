# SBFB -- Decentralized P2P App Distribution and Compute

**Publish apps. Distribute them peer-to-peer. Verify everything. No central server.**

## What is SBFB?

SBFB is a protocol for publishing, distributing, and running web applications
over a peer-to-peer network. Anyone can package a web app (HTML, React, Python
via Pyodide, WASM, Jupyter notebooks -- anything that produces HTML), sign it
with Ed25519, and broadcast it. Other nodes download, cache, and verify the
archive. Users open apps in a sandboxed iframe through their local daemon. No
app store, no cloud account, no central authority.

The second axis is **shared AI/GPU compute**: apps can request translation,
analysis, or generation tasks from volunteer GPU/CPU workers on the network,
with explicit consent and resource caps.

## Architecture

```
                          +------------------+
                          |  P2P Network     |
                          |  (iroh 0.98)     |
                          |  gossip + blobs  |
                          |  + docs (CRDT)   |
                          +--------+---------+
                                   |
                            QUIC / Ed25519
                                   |
                          +--------+---------+
                          |  Shell Daemon    |
                          |  (Rust binary)   |
                          |  loopback HTTP   |
                          |  bearer + Host   |
                          |  + Origin auth   |
                          +--------+---------+
                                   |
                           127.0.0.1 only
                                   |
               +-------------------+-------------------+
               |                                       |
    +----------+----------+             +--------------+---------+
    |  Shell (React UI)   |             |  Blob-serve            |
    |  Browse, Curators,  |             |  (separate origin)     |
    |  Network, Deploy    |             |  decompresses zip      |
    |                     |             |  LRU cache             |
    +----------+----------+             +--------------+---------+
               |                                       |
               |            postMessage bridge          |
               +---------------+-----------------------+
                               |
                    +----------+----------+
                    |  App iframe          |
                    |  sandbox="allow-     |
                    |  scripts" (no        |
                    |  allow-same-origin)  |
                    |  CSP: connect-src    |
                    |  'none'              |
                    +---------------------+
```

## Key Features

- **Peer-to-peer distribution** -- apps propagate via gossip and iroh-blobs (content-addressed, BLAKE3 hashed). No hosting required.
- **Source-verifiable deploys** -- each published app carries a `provenance.json` linking a Git commit to the archive hash via Ed25519 signature (SLSA Level 1).
- **5-layer sandbox** -- iframe without `allow-same-origin`, strict CSP (`connect-src 'none'`), separate blob-serve origin, postMessage bridge with method allowlist, loopback auth with 256-bit bearer token.
- **Curator lists instead of moderation** -- communities sign Ed25519 lists of recommended (or warned-against) apps. Users choose which curators to follow. Nobody can remove an app from the network.
- **P2P data sync** -- app data stored in iroh-docs (CRDT, last-write-wins). Offline-first: partitions heal automatically on reconnection.
- **Full-text search** -- local FTS5 index over apps, feed entries, and curators. No remote search service.
- **Proof Cards** -- deterministic evidence-completeness score (0-100) for each app, computed locally from provenance, curators, license, and freshness.
- **Distributed GPU/CPU compute** -- volunteer workers run AI inference (Ollama or embedded llama.cpp) with 4-level consent (my projects only / verified open source / manual allowlist / all public).
- **Kudos reputation** -- non-monetary, non-transferable compute reputation with logarithmic returns, EMA decay, BLAKE3 hash-chain, and fairness metrics (Gini coefficient tracking).
- **Append-only public feed** -- BLAKE3 hash-chained, Ed25519-signed entries per author. Tamper-detectable history of publications, endorsements, and source changes.
- **Key rotation** -- Ed25519 key rotation with 14-day transition window for curator lists.
- **Cross-platform** -- Windows (NSIS installer), Linux (.deb), macOS (.dmg). Runs behind NAT via Pkarr DHT + DNS TXT fallback + WebSocket relay.
- **AGPL-3.0** -- the protocol and daemon code stay free. Apps on the network have their own licenses.

## How It Works

**1. Create** -- Use `sbfb-factory create myapp` to scaffold an app with `SBFB.json` manifest, `index.html`, and the bridge SDK.

**2. Develop** -- Build your app with any web technology. Use `sbfb-factory preview` to test it in the same sandbox users will see.

**3. Publish** -- `sbfb-factory publish` clones your Git repo, zips the archive, computes a BLAKE3 hash, signs an Ed25519 provenance attestation, and broadcasts the announcement via P2P gossip.

**4. Distribute** -- Other nodes discover the app via gossip, download the archive through iroh-blobs, verify the hash, and cache it locally.

**5. Open** -- Users browse apps in the shell UI. The daemon serves the archive through blob-serve into a sandboxed iframe. The app communicates with the network exclusively through the postMessage bridge.

## Quick Start

```bash
# Download the daemon binary for your platform (~21 MB)
# https://codeberg.org/sbfb/sbfb/releases

# Run it -- creates ~/.sbfb/ with your Ed25519 identity and auth token
./nexus-shell-daemon

# Opens your browser to the shell UI on 127.0.0.1
# Browse apps, subscribe to curators, check Proof Cards
```

The daemon listens on loopback only. No public ports exposed. No TLS
certificates to manage. RAM usage is ~150 MB idle.

## Publishing an App

```bash
# Install the factory CLI (included in the release)
# Or build from source:
cargo install --path crates/sbfb-factory

# Scaffold a new app
sbfb-factory create myapp
cd myapp/

# Edit index.html, add your code, then validate
sbfb-factory validate

# Preview in the sandbox (30-minute TTL)
sbfb-factory preview

# Publish to the network (requires a running daemon + public Git repo)
sbfb-factory publish
```

The publish pipeline runs 11 gates (FG0-FG10): classification, bridge scope
analysis, manifest validation, diff review, sandbox safety (path traversal,
symlinks), secret scanning (AWS keys, GitHub tokens, PEM files), preview,
provenance signing, P2P broadcast, and asynchronous curator review.

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

The parser is forward-compatible: unknown fields are ignored, older and newer
schema versions are accepted.

## Example Apps

| App | Description | Bridge methods used |
|-----|-------------|---------------------|
| **Protocol Explorer** (`examples/sbfb-explorer/`) | Interactive documentation of the SBFB protocol. Live network status panel via bridge. Provenance verification demo. Pure HTML/JS, zero npm dependencies. | `node_status`, `identity_pubkey`, `browse_list`, `provenance_verify` |
| **Ideas Hub** (`examples/sbfb-ideas/`) | Decentralized idea board with voting. One vote per Ed25519 identity, toggle on/off. Data syncs between nodes via P2P storage. Pure HTML/JS. | `storage_get`, `storage_set`, `storage_list`, `storage_delete`, `identity_pubkey` |
| **Factory Viewer** (`examples/sbfb-factory-viewer/`) | Read-only app displaying network apps with Proof Cards, search, and quality badges. Runs in the same sandbox as any other app. | `browse_list`, `search`, `proof_card_get`, `storage_get` |

## Bridge SDK

Apps in iframes communicate with the network through `sbfb-bridge.js` -- a
423-line JavaScript library with no dependencies:

```html
<script src="/sbfb-bridge.js"></script>
<script>
  const bridge = new SBFBBridge({ timeout: 10000 });

  // P2P storage (CRDT, syncs between nodes)
  await bridge.setStorage("my-key", { value: 42 });
  const data = await bridge.getStorage("my-key");
  const all  = await bridge.listStorage("prefix/");
  await bridge.deleteStorage("old-key");

  // Network introspection
  const status = await bridge.getNodeStatus();
  const apps   = await bridge.getBrowseList();

  // Provenance verification
  const proof  = await bridge.verifyRelease(projectId);
  const card   = await bridge.getProofCard(projectId);

  // Full-text search
  const results = await bridge.search("translation app", { limit: 20 });

  // Identity
  const me = await bridge.getIdentityPubkey();

  // Distributed compute (requires user GPU consent)
  await bridge.submitTask({ prompt: "Translate this text" });

  // PII redaction (GLiNER ONNX + regex fallback)
  const safe = await bridge.piiRedact(userInput);

  // Live sync notifications
  const unsub = bridge.onStorageUpdate("my-app", () => refresh());
</script>
```

All communication goes through `window.postMessage` with UUID correlation IDs
and a 10-second timeout. The host shell validates every request against the
method allowlist before forwarding to the daemon.

## GPU Compute

A node can contribute GPU or CPU compute to the network. The worker binary
(`nexus-worker`) runs AI inference via two backends: **Ollama** (HTTP, zero
build dependencies) or **embedded llama.cpp** with llguidance constrained
decoding (direct process control, VRAM wipe between tasks).

Consent is explicit at 4 levels:

- **L1** -- My projects only (default, zero exposure)
- **L2** -- Verified open-source projects (provenance required)
- **L3** -- Manual allowlist (project by project)
- **L4** -- All public projects

Each level enforces per-project caps on watts, VRAM, and daily hours. Results
are signed by the worker, validated by the coordinator (SHA256 quorum when
multiple workers respond), and tracked in a non-monetary kudos ledger with
logarithmic returns and Gini fairness monitoring.

## Security Model

The threat model follows STRIDE + LINDDUN methodology. Five defense layers:

1. **Iframe sandbox** -- `sandbox="allow-scripts"` without `allow-same-origin`. Apps cannot access the parent DOM, cookies, or localStorage of the shell.
2. **Content Security Policy** -- `connect-src 'none'` on untrusted app content. Zero outbound network requests from the iframe.
3. **postMessage bridge** -- Method allowlist with UUID correlation. Apps can only call declared bridge methods.
4. **Loopback authentication** -- 256-bit bearer token + Host header allowlist + Origin check + OS-level peer credentials (SO_PEERCRED on Linux, Named Pipe DACL on Windows).
5. **Provenance chain** -- Ed25519 signatures on archives, BLAKE3 content hashes, append-only feed with hash-chain integrity.

Additional protections: Hashcash 16-bit anti-spam on gossip, Sybil resistance
via 7-day age witness, GCRA rate limiting per author on feed operations, CPU
watchdog heartbeat on iframes (1-second liveness ping), key encryption at rest
(Argon2id + AES-256-GCM + OS keyring double-wrap).

Full threat model: [`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md)

## Trust Levels

Six cumulative trust levels for apps on the network:

| Level | Label | What it guarantees |
|-------|-------|-------------------|
| **N0** | Upload direct | An archive exists on the network. Origin unknown. |
| **N1** | Source readable | A public source repository is declared. |
| **N2** | Provenance attested | Ed25519 signature links a Git commit to the archive hash (SLSA L1 self-attestation). |
| **N3** | Signature verified | The local daemon has verified the provenance signature live. |
| **N4** | Reproducible build | A third party rebuilt the archive from source and got the same hash. *(future)* |
| **N5** | Feed verified | The full publication history is hash-chain integral from genesis. |

Details: [`docs/trust/TRUST_TAXONOMY.md`](../trust/TRUST_TAXONOMY.md)

## Project Structure

```
sbfb/
+-- Cargo.toml                          # Rust workspace
+-- crates/
|   +-- nexus-core-rs/                  # iroh wrapper (P2P, crypto, canonical bytes)
|   +-- nexus-coordinator-rs/           # DB, dispatcher, validator, kudos, capabilities
|   +-- nexus-shell-daemon-core/        # P2P discovery, curator runtime, search, feed
|   +-- nexus-shell-daemon/             # Shell daemon binary (HTTP + gossip)
|   +-- nexus-launcher/                 # Minimal launcher (spawn daemon + open browser)
|   +-- nexus-worker-core/              # Headless compute engine (Ollama + llama.cpp)
|   +-- nexus-worker/                   # Worker binary (CLI + TUI)
|   +-- nexus-events-core/              # Security event system (JSONL + ETW)
|   +-- nexus-executor/                 # Build executor for reproducible builds
|   +-- nexus-trace-core/               # OpenTelemetry tracing infrastructure
|   +-- nexus-test-harness/             # Integration test helpers
|   +-- sbfb-manifest/                  # SBFB.json parser (shared daemon <-> factory)
|   +-- sbfb-factory/                   # Factory CLI (create, validate, publish, gates)
+-- web/                                # Shell UI (React 19 + TypeScript + Tailwind + shadcn/ui)
+-- tools/
|   +-- factory-operator/               # Local management tool (Vite + React)
|   +-- factory-ui/                     # Shared UI components (readonly + operator)
+-- examples/
|   +-- sbfb-explorer/                  # Protocol Explorer app
|   +-- sbfb-ideas/                     # Ideas Hub app
|   +-- sbfb-factory-viewer/            # Factory Viewer app
+-- docs/
|   +-- security/THREAT_MODEL.md        # STRIDE + LINDDUN threat model
|   +-- trust/TRUST_TAXONOMY.md         # N0-N5 trust levels
|   +-- release/FACTORY_GATES.md        # FG0-FG10 publish gates
+-- prompts/agent/                      # Portable agent prompts (8 kinds)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Protocol / daemon** | Rust 1.85+, iroh 0.98 (gossip, blobs, docs), iroh-blobs 0.100 |
| **Crypto** | Ed25519 (ed25519-dalek), BLAKE3, ChaCha20-Poly1305 (QUIC), Argon2id, AES-256-GCM, FROST threshold signatures |
| **Storage** | SQLite (rusqlite 0.36), iroh-docs (CRDT P2P) |
| **Search** | FTS5 (SQLite virtual table, BM25 ranking) |
| **LLM inference** | Ollama (HTTP) or llama.cpp (embedded, llguidance constrained decoding) |
| **Frontend** | React 19, TypeScript, Tailwind CSS, shadcn/ui, Zustand, React Query, Vite |
| **CI** | Woodpecker (self-hosted ci.sbfb.world) + GitHub Actions |
| **Installers** | cargo-packager (NSIS / .deb / .dmg) |

## Tests

~1800 tests across the project:

- **1486 Rust tests** (unit + integration + adversarial + E2E multi-daemon)
- **279 Vitest tests** (frontend components + bridge + API schemas)
- **6/6 size-limit checks** (bundle size regression)

```bash
# Run all Rust tests
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc

# Run a single crate
cargo nextest run -p nexus-shell-daemon-core --locked

# Lint
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings

# Frontend
cd web && npm install && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size
```

## License

[AGPL-3.0-or-later](https://www.gnu.org/licenses/agpl-3.0.html)

The SBFB protocol, daemon, worker, and all tooling are licensed under
AGPL-3.0-or-later. This means any modification to the protocol or daemon
deployed as a network service must be distributed with its source code.

Apps published on the SBFB network are **not** automatically AGPL. Each app
carries its own license. To be recommended as a verified app, an app must
publish its source, declare a license, and provide verifiable provenance.

## Status

SBFB is in **closed pilot phase**. The tag v1.0 has been set locally but
not yet pushed to a public registry.

What works today:

- Local daemon, shell UI, iframe sandbox, postMessage bridge
- P2P app distribution (validated LAN Win/Mac + WAN dev/VPS Helsinki)
- Publish from source with Ed25519 provenance (SLSA L1)
- Curator lists, endorsement/disendorsement, subscription
- Full-text search, Proof Cards, public feed
- Factory CLI (create, validate, preview, publish, gates FG0-FG10)
- GPU/CPU worker with consent, caps, kudos ledger
- Windows/Linux/macOS installers

What is not yet ready:

- No formal security audit of the full stack
- No public nodes in production (iroh upstream audit pending)
- SearchManifest for cross-node search discovery
- Full governance UI for curator timelines and dissent
- Reproducible builds by independent third parties
- Worker quorum enforcement at scale

This is an advanced experiment, not a production service. We are looking for
1-2 testers willing to install nodes, publish small apps, and report what
breaks.

## Contributing

Issues and pull requests are welcome on
[Codeberg](https://codeberg.org/sbfb/sbfb).

Before contributing:

- Read the threat model (`docs/security/THREAT_MODEL.md`) if touching security-sensitive code
- Run the full test suite before submitting (`cargo nextest run --workspace --locked` + `cd web && npm run test:unit`)
- All contributions fall under AGPL-3.0-or-later

Solo maintainer model (inspired by OpenBSD). No foundation, no startup, no
funding. Built to stay independent.
