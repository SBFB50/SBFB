# Contributing to nexus-grid

Thank you for your interest in nexus-grid. This project is open to all
contributions.

## How to contribute

### Report a bug

1. Check that it does not already exist in the
   [Issues](https://github.com/SBFB50/SBFB/issues)
2. Create an issue with: description, steps to reproduce, expected vs
   observed behavior
3. Add relevant logs if possible

### Propose a feature

1. Open a [Discussion](https://github.com/SBFB50/SBFB/discussions)
2. Describe the need, proposed approach, and expected impact
3. Wait for feedback before coding

### Submit code

1. Fork the repo
2. Create a branch from `master`: `git checkout -b feature/my-feature`
3. Code, test, commit
4. Open a Pull Request with a clear description
5. Review within 48h

## Monorepo structure

```
nexus-grid/
├── crates/          # Rust workspace (cargo)
├── web/             # React frontend (npm)
├── examples/        # SBFB app archives (hello-world, explorer, ideas)
├── scripts/         # verify.sh, check-spdx.sh, docs gates
└── deploy/          # VPS provisioning scripts
```

The project's core workspaces are Rust + Frontend since S50-S51 (the
Python-era `packages/` workspace was removed — see `docs/DEPRECATED.md`;
`examples/` may carry app-archive sources in any language).

## Code standards

### Rust
- `cargo fmt --all --check` — no exceptions
- `cargo clippy --workspace --all-targets -- -D warnings`
- Tests: `cargo test --workspace --locked`

### TypeScript/React
- ESLint config from the project
- `npx tsc --noEmit` strict mode
- Vitest for unit tests, Playwright for e2e
- French strings for user-facing text (`scripts/scan-en-strings.sh`)

### Commits
- Format: `type(scope): description`
- Types: feat, fix, docs, refactor, test, chore
- Example: `feat(sdk): add typed namespace for AppContext.storage`

## Verification

Before submitting, run the full verification suite:

```bash
./scripts/verify.sh
```

This runs 16 steps covering Rust, frontend, Playwright E2E, SPDX
license and docs-contract gates. All steps must pass.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
