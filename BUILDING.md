# Building nexus-grid

Instructions for auditors and contributors to build the full
project from source. All commands assume a clean clone on the
target platform.

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | 1.94+ | `rustup update stable` |
| Python | 3.13+ | system or `uv python install 3.13` |
| Node.js | 22+ | system or `nvm install 22` |
| uv | 0.7+ | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| Ollama | latest | `ollama serve` (optional, for worker benchmarks) |

## Clone

```bash
git clone https://github.com/SBFB50/SBFB.git
cd SBFB
```

## Rust workspace

```bash
# Build all crates (debug)
cargo build --workspace --locked

# Build release binaries (daemon + worker + executor + launcher)
cargo build -p nexus-shell-daemon --release
cargo build -p nexus-worker --release
cargo build -p nexus-executor --release
cargo build -p nexus-launcher --release

# Run all tests
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc

# Lint
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### Crate map (security-relevant)

| Crate | Role | Audit priority |
|---|---|---|
| `nexus-core-rs` | Ed25519 crypto, canonical bytes (JCS), iroh wrapper | **Critical** |
| `nexus-core-py` | PyO3 bindings for sign/verify | High |
| `nexus-events-core` | SecurityEvent enum, EventWriter trait | Medium |
| `nexus-worker-core` | Consent enforcement, GPU monitor, Ollama client | High |
| `nexus-worker` | Worker binary (CLI + TUI) | Medium |
| `nexus-shell-daemon-core` | P2P discovery, curator runtime, auth middleware | **Critical** |
| `nexus-shell-daemon` | Daemon binary (HTTP + gossip) | **Critical** |
| `nexus-executor` | Executor binary (IPC JSON-RPC 2.0) | High |
| `nexus-launcher` | Minimal launcher (spawn daemon + open browser) | Medium |

## Python workspace

```bash
# Install dependencies
uv sync

# Lint
uv run ruff format --check packages/
uv run ruff check packages/

# Tests (run separately — collision de nom tests/)
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q
```

### PyO3 wheel (optional)

The coordinator imports `nexus_core` for Ed25519 sign/verify via
PyO3. To build the wheel into the uv venv:

```bash
unset CONDA_PREFIX CONDA_DEFAULT_ENV
VIRTUAL_ENV=$PWD/.venv maturin develop --release \
  --manifest-path crates/nexus-core-py/Cargo.toml
```

## Frontend (React shell)

```bash
cd web
npm install
npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run test:unit
npm run build
npm run size
npx playwright test
```

## Security-relevant files

For a focused audit, start with:

- `docs/security/THREAT_MODEL.md` — full STRIDE + LINDDUN model
- `docs/security/HARDENING_ROADMAP.md` — mitigation roadmap
- `docs/security/EXTERNAL_AUDIT_SCOPE.md` — audit scope document
- `docs/security/RUNTIME_ISOLATION.md` — isolation strategy
- `crates/nexus-core-rs/src/crypto.rs` — Ed25519 sign/verify
- `crates/nexus-core-rs/src/canonical.rs` — JCS canonical bytes
- `crates/nexus-shell-daemon-core/src/auth.rs` — loopback auth
- `crates/nexus-shell-daemon/src/blob_serve.rs` — iframe CSP
- `crates/nexus-worker-core/src/consent.rs` — consent enforcement

## Environment variables

| Variable | Purpose | Default |
|---|---|---|
| `SBFB_HOME` | Override `~/.sbfb/` data directory | `~/.sbfb/` |
| `SBFB_COORDINATOR_URL` | Coordinator base URL | `http://127.0.0.1:8765` |
| `SBFB_DAEMON_URL` | Daemon base URL | `http://127.0.0.1:7777` |
| `RUST_LOG` | Rust tracing filter | `info` |

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE)
