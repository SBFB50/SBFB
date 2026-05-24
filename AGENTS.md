# Repository Guidelines

## Project Structure & Module Organization

This monorepo powers `nexus-grid`, a decentralized P2P compute network. Rust
crates are under `crates/`; the React shell is in `web/`. Tests live in
`tests/`, crate-local Rust tests, `web/src/**/__tests__/`, and `web/tests/`.
Docs, deployment, configs, and assets live in `docs/`, `deploy/`, `configs/`,
and `assets/`. Legacy Python scripts remain in `scripts/agent/` for process
compatibility.

## Build, Test, and Development Commands

- `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo nextest run --workspace --locked`: Rust format, lint, and tests.
- `cargo test --workspace --locked --doc`: Rust doctests.
- `cargo build -p nexus-shell-daemon --release`: release build (daemon).
- `cargo build -p sbfb-factory --release`: release build (factory tooling).
- `cd web && npm run dev`: start the Vite shell.
- `cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run build && npm run size`: frontend lint, typecheck, unit tests, build, and size-limit.
- `python scripts/agent/agentctl.py context`: print vendor-neutral agent context (legacy Python).
- `python scripts/agent/agentctl.py prompt --kind universal --depth deep`: assemble full LLM handoff prompt (legacy Python).
- `git config core.hooksPath .githooks`: enable portable Git gates.

## Coding Style & Naming Conventions

Rust uses edition 2021; keep `cargo fmt` and Clippy clean. TypeScript/React
uses ESLint, strict TypeScript, PascalCase components, and camelCase
hooks/utilities. Keep user-facing frontend strings in French.

## Testing Guidelines

Name Rust integration tests by behavior, and frontend tests `*.test.ts(x)` or
Playwright `*.spec.ts`. Add focused tests near changed modules. Sprint phase
commits require full Rust and frontend verification blocks for every phase
without exception.

## Commit & Pull Request Guidelines

Commits follow `type(scope): description`, for example
`feat(daemon): add typed storage namespace`. Common types are `feat`,
`fix`, `docs`, `refactor`, `test`, and `chore`. PRs need a concise
description, relevant issue, test evidence, and screenshots for UI changes.

## Security & Protocol Notes

Preserve loopback, sandbox, SBFB bridge, signing, allowlist, and provenance
invariants. Treat `configs/*.sample`, `deploy/`, `docs/security/`, and protocol
schemas as contract surfaces.

## Agent Process

See `docs/agent/AGENT_SYSTEM.md` for the system map: roles, providers,
lifecycle modes, gate contracts, and prompt registry.

Use `docs/agent/PROCESS.md` as the vendor-neutral sprint workflow for Claude,
GPT, and local LLMs. Sprint phase commits require
`.planning/active/sprint{N}_phase_{X}_review.md` with `## Verdict: PASS`.
`scripts/agent/agentctl.py` (legacy Python) and `crates/sbfb-factory` (Rust)
enforce the portable gates.
