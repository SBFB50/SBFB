# Repository Guidelines

## Project Structure & Module Organization

This monorepo powers `nexus-grid`, a decentralized P2P compute network for LLM apps. Python packages live in `packages/` with legacy/shared code in `nexus/`. Rust crates are under `crates/`; the React shell is in `web/`. Tests live in `tests/`, `packages/*/tests/`, crate-local Rust tests, `web/src/**/__tests__/`, and `web/tests/`. Docs, deployment, configs, and assets live in `docs/`, `deploy/`, `configs/`, and `assets/`.

## Build, Test, and Development Commands

- `./scripts/setup.sh`: build the PyO3 wheel and sync Python deps.
- `./scripts/verify.sh` or `./scripts/verify.sh --quick`: run full or quick verification.
- `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`: Rust format, lint, and tests.
- `uv run ruff format --check packages/ && uv run ruff check packages/ && uv run pytest packages/nexus-sdk/tests/ -q`: Python checks and a focused suite.
- `cd web && npm run dev`: start the Vite shell.
- `cd web && npm run lint && npm run test:unit && npm run build`: frontend lint, unit tests, and build.
- `python scripts/agent/agentctl.py context`: print vendor-neutral agent context.
- `python scripts/agent/agentctl.py prompt --kind universal --depth deep`: assemble the full universal LLM handoff prompt.
- `python scripts/agent/agentctl.py prompt --kind preflight --sprint 35 --phase A`: assemble a phase prompt.
- `git config core.hooksPath .githooks`: enable portable Git gates.

## Coding Style & Naming Conventions

Python targets 3.13, Ruff, 120-character lines, and snake_case. Rust uses edition 2021; keep `cargo fmt` and Clippy clean. TypeScript/React uses ESLint, strict TypeScript, PascalCase components, and camelCase hooks/utilities. Keep user-facing frontend strings in French.

## Testing Guidelines

Name Python tests `test_*.py`, Rust integration tests by behavior, and frontend tests `*.test.ts(x)` or Playwright `*.spec.ts`. Add focused tests near changed modules. Sprint phase commits require full Rust and frontend verification blocks for every phase without exception.

## Commit & Pull Request Guidelines

Commits follow `type(scope): description`, for example `feat(sdk): add typed storage namespace`. Common types are `feat`, `fix`, `docs`, `refactor`, `test`, and `chore`. PRs need a concise description, relevant issue, test evidence, and screenshots for UI changes.

## Security & Protocol Notes

Preserve loopback, sandbox, SBFB bridge, signing, allowlist, and provenance invariants. Treat `configs/*.sample`, `deploy/`, `docs/security/`, and protocol schemas as contract surfaces.

## Agent Process

Use `docs/agent/PROCESS.md` as the vendor-neutral workflow for Claude, GPT, and local LLMs. Sprint phase commits require `.planning/active/sprint{N}_phase_{X}_review.md` with `## Verdict: PASS`; `scripts/agent/agentctl.py` enforces the portable gates.
