# nexus-coordinator

Project-scoped coordinator for the nexus-grid P2P compute network.

A coordinator owns a project on the network, signs LLM tasks with
its Ed25519 key, dispatches them to workers via an iroh-docs
replica, validates the results (3-layer: signature + model digest
+ logprob fingerprint), and maintains an append-only kudos ledger
for the workers that did the work.

## Install (from the monorepo root)

```bash
uv sync
uv run nexus-coordinator --help
```

The PyO3 `nexus_core` wheel is built automatically by maturin
through the uv workspace dep on `crates/nexus-core-py/`.

## Quickstart

```bash
# Create a new project rooted at ~/.nexus-grid/projects/demo/
uv run nexus-coordinator init demo

# Boot the coordinator: iroh Node + tasks/claims/results doc +
# FastAPI control plane on 127.0.0.1:8765
uv run nexus-coordinator start demo --port 8765

# From another shell:
curl -s http://127.0.0.1:8765/health
```

Phase A (this release) covers the boot path and the `/health` +
`/project` endpoints. Phases B–D add dispatcher, validator,
kudos ledger, invite tokens, SDK, and app loader.

## Storage

State lives under `platformdirs.user_data_dir("nexus-grid")`:

```
~/.nexus-grid/
└── projects/
    └── demo/
        ├── coord.key           # Ed25519 secret (perm 600)
        ├── coordinator.toml    # persistent config
        └── iroh-data/          # iroh node storage (blobs + docs)
```
