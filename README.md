# nexus-grid

> Decentralized P2P compute network for LLM apps.
> No central server. No admin. Just protocol.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

---

## What is nexus-grid?

nexus-grid is a peer-to-peer network where anyone can host LLM-powered
applications and anyone can contribute compute. Projects are
self-contained: each coordinator signs and dispatches tasks to workers
via iroh-docs, with Ed25519 cryptographic identity and zero central
authority.

**Key properties:**
- **No master server** — each project runs its own coordinator
- **No admin** — curator lists propagate via gossip, not moderation
- **Allowlist per-worker** — contributors choose what they serve
- **Kudos per-project** — append-only hash-chain reputation ledger

## Architecture

```
                     +-----------+
                     |  Worker   |   Rust binary (nexus-worker)
                     | (GPU/CPU) |   Ollama runtime, Ed25519 id
                     +-----+-----+
                           |
                    iroh-docs (QUIC)
                           |
+-----------+        +-----+------+        +-----------+
|   Shell   | HTTP   | Coordinator| iroh   |  Shell    |
|  (React)  +------->+  (FastAPI) +------->+  Daemon   |
|           |        |  SDK apps  |  gossip|  (DHT)    |
+-----------+        +-----+------+        +-----------+
                           |
                     +-----+-----+
                     |  NexusApp  |   Python SDK (nexus-sdk)
                     | gov, cold- |   TabView, events, storage,
                     | case, etc. |   migrations, file upload
                     +-----------+
```

**Rust workspace** (`crates/`): `nexus-core-rs` (iroh 0.97 wrapper),
`nexus-core-py` (PyO3 bindings), `nexus-worker-core` + `nexus-worker`
(headless compute binary), `nexus-shell-daemon-core` +
`nexus-shell-daemon` (P2P discovery layer).

**Python workspace** (`packages/`): `nexus-coordinator` (FastAPI +
dispatcher + kudos ledger), `nexus-sdk` (NexusApp ABC + TabView),
`nexus-app-gov` / `-coldcase` / `-forensics` (official apps).

**Frontend** (`web/`): React + Vite + TypeScript + Tailwind + shadcn/ui.

## Quick start

### Prerequisites

- Rust 1.94+ (via [rustup](https://rustup.rs/))
- Python 3.13+ (via [uv](https://docs.astral.sh/uv/))
- Node.js 20+
- maturin 1.13 (`uv tool install maturin`)

### Setup

```bash
git clone https://github.com/SBFB50/SBFB.git
cd SBFB

# Build the PyO3 wheel + sync Python deps
./scripts/setup.sh

# Run the full verification suite
./scripts/verify.sh
```

### Write an app

```python
from nexus_sdk import NexusApp, nexus_tab
from nexus_sdk.view import TabView, heading, text

class HelloWorld(NexusApp):
    name = "hello-world"

    @nexus_tab(name="Greeting", icon="hand-wave")
    async def greet(self, ctx):
        return TabView(blocks=[
            heading("Hello from nexus-grid!"),
            text("This app runs on a P2P compute network."),
        ])
```

### Run a worker

Download the `nexus-worker` binary from
[Releases](https://github.com/SBFB50/SBFB/releases) and run:

```bash
nexus-worker start --stub-ollama   # dry-run without GPU
nexus-worker start                  # real Ollama runtime
```

### Host a project

```bash
pip install nexus-sdk nexus-coordinator
nexus-coordinator init
nexus-coordinator start
```

The coordinator starts a local web shell on `http://localhost:8765`.

## Development

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python
uv run ruff format --check packages/
uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json
npm run test:unit && npm run test:coverage
npm run build && npm run size
npx playwright test
```

Or use the all-in-one script:

```bash
./scripts/verify.sh          # full run
./scripts/verify.sh --quick  # skip Playwright
```

## Test counts (v1.0.0)

| Suite | Count |
|---|---|
| Rust workspace | 312 |
| Python SDK | 167 |
| Python coordinator | 83 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 161 |
| Playwright e2e | 27 |
| size-limit | 7/7 |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[AGPL-3.0-or-later](LICENSE)

This license guarantees that nexus-grid and all derivative works remain
free and open source. If you deploy a modified version as a network
service, you must share your source code.
