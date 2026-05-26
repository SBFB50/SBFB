# nexus-grid

> Decentralized P2P compute network for LLM apps.
> No central server. No admin. Just protocol.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

---

## Repository availability

Current source of truth is the GitHub repository:
`https://github.com/SBFB50/SBFB`.

A Codeberg mirror already exists at `https://codeberg.org/SBFB/SBFB`,
but it is private during the pre-launch phase. It is a
disaster-recovery maintainer mirror, push-synchronised from GitHub, not
yet a public clone URL for users or pilots.

At v1.0 go-live, GitHub and Codeberg are intended to be made public in
the same release window, with Radicle activation following the documented
mirror plan. See [`docs/release/MIRROR_FALLBACK.md`](docs/release/MIRROR_FALLBACK.md).

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
# Pre-launch maintainer/pilot access. Public availability is not open yet.
git clone https://github.com/SBFB50/SBFB.git
cd SBFB

# Codeberg mirror is prepared but private pre-launch:
# https://codeberg.org/SBFB/SBFB

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

## Security

nexus-grid's security model combines five layers:

- **Iframe sandboxing** (Sprint 12/13): apps run in
  `sandbox="allow-scripts"` iframes with CSP `connect-src 'none'`
  on a dedicated blob-serve origin (`:7000`). All network I/O
  goes through the postMessage bridge — no direct
  fetch/WebSocket/CDN loading is possible.
- **postMessage bridge** (Sprint 13/15): three whitelisted
  methods (`task_submit`, `storage_get`, `storage_set`) plus a
  fire-and-forget event channel (`sbfb-bridge-event`), validated
  with Zod schemas + source check (`event.source ===
  iframe.contentWindow`). Correlation IDs prevent cross-app
  response confusion. CPU watchdog via heartbeat (5s timeout)
  surfaces a "app not responding" overlay.
- **Verified deployment** (Sprint 14): public apps are cloned
  from their Git repo by the coordinator itself (depth 1,
  500 MB cap, 30s timeout, `.git/` excluded, path traversal
  rejected), verified against an Ed25519-signed `SBFB.json`
  (Keyoxide pattern), and published with a SLSA L1
  `provenance.json` signature pinned on `commit_sha` (40 hex).
- **Loopback hardening** (Sprint 16): triple-check middleware on
  every HTTP request — X-SBFB-Token 256-bit bearer (mitigation
  for CVE-2025-49596 Anthropic MCP Inspector DNS rebinding,
  CVSS 9.4) + Host allowlist `{localhost, 127.0.0.1, [::1]}` +
  Origin check. UDS with SO_PEERCRED (Unix) and Named Pipes with
  DACL user-only via SDDL (Windows) as orthogonal peer-creds
  bypass for CLI and daemon-to-coord calls. `/health` is the
  single unauthenticated probe.
- **GPU consent opt-in** (Sprint 16): worker refuses to claim
  a task unless the user has opted into one of four sharing
  levels (own projects / verified open source / manual whitelist
  / all public) with hard caps on watts, VRAM and hours-per-day.
  Config is live-reloaded via a `notify` file watcher (50 ms
  debounce); daily counter resets at local midnight. GDPR
  Art.6(1)(a) lawful basis via explicit opt-in; Art.7(3)
  withdrawal via the same dialog.

**Documentation**:

- [`docs/security/README.md`](docs/security/README.md) — index
  and contribution guide.
- [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md) —
  assets, adversaries, DFD, STRIDE per component, LINDDUN per
  flow, mitigations table with commit SHAs, residual risks
  (post-Sprint 16 baseline).
- [`docs/security/RUNTIME_ISOLATION.md`](docs/security/RUNTIME_ISOLATION.md) —
  Sprint 17+ roadmap for invisible VM isolation (WSL2 /
  Virtualization.framework / systemd-nspawn) that closes the
  keypair-at-rest residual risk identified in the threat model.
- [`docs/security/ADVERSARIES.md`](docs/security/ADVERSARIES.md) —
  6-tier adversary taxonomy (T0 misconfigured user → T5 targeted
  state actor) with budget, capabilities, and motivations per tier.
- [`docs/security/ATTACK_SCENARIOS.md`](docs/security/ATTACK_SCENARIOS.md) —
  12 concrete T1-T5 attack scenarios (CSP bypass, supply chain,
  dragnet correlation, checkpoint seize, turned contributor, etc.)
  with chain + current mitigation status.
- [`docs/security/P2P_THREATS.md`](docs/security/P2P_THREATS.md) —
  7 P2P network attack surfaces (Sybil, Eclipse, gossip, DHT,
  BGP/relay, traffic analysis, ISP block) with per-vector SBFB
  coverage + sequenced mitigations + academic references.
- [`docs/security/COMPUTE_THREATS.md`](docs/security/COMPUTE_THREATS.md) —
  7 GPU compute-sharing threat classes (prompt leakage, result
  spoofing, compute theft, model extraction, prompt injection,
  side-channel GPU, DoS task flood) with 2020-2026 references.
- [`docs/security/HARDENING_ROADMAP.md`](docs/security/HARDENING_ROADMAP.md) —
  threat × mitigation matrix (27 threats), prioritization framework
  (impact × likelihood / effort), Sprint 18-30 sequenced roadmap,
  quick-wins, big-rocks, dependency graph, Gates 1-4 unlocking.
- [`docs/security/VALIDATED_BLUEPRINT.md`](docs/security/VALIDATED_BLUEPRINT.md) —
  long-term 13-layer maximalist design, each OSS brick validated
  against 2026 upstream docs + advisories + CVE database,
  positioning vs Signal / Tor / Briar / SecureDrop / Mozilla /
  Bytecode Alliance state-of-the-art.

For the full sprint history (grouped by released version) see
[`.planning/README.md`](.planning/README.md) and
[`docs/claude/SPRINT_LOG.md`](docs/claude/SPRINT_LOG.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[AGPL-3.0-or-later](LICENSE)

This license guarantees that nexus-grid and all derivative works remain
free and open source. If you deploy a modified version as a network
service, you must share your source code.
