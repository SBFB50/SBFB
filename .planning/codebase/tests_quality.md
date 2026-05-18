# Tests, Quality & CI/CD

**Analysis Date:** 2026-05-18

## Test Count Summary

| Category | Count | Location |
|----------|-------|----------|
| Rust unit + integration tests | ~1344 | `crates/*/src/**/*.rs` + `crates/*/tests/*.rs` |
| Vitest unit tests | 265 | `web/src/**/__tests__/*.test.{ts,tsx}` (22 files) |
| Playwright E2E tests | 44 | `web/tests/*.spec.ts` (28 files) |
| Size-limit checks | 6 | `web/.size-limit.json` |
| Criterion benchmarks | 3 groups | `crates/nexus-core-rs/benches/pow.rs`, `keystore.rs`, `crates/nexus-executor/benches/cold_start.rs` |
| **Total** | **~1659** | |

## Rust Test Distribution by Crate

| Crate | Test Count | Top Files |
|-------|-----------|-----------|
| `nexus-core-rs` | ~311 | `task.rs` (22), `pow.rs` (26), `key_rotation.rs` (25), `keystore.rs` (23), `curator.rs` (21), `tls_pinning.rs` (17), `crypto.rs` (15) |
| `nexus-shell-daemon` | ~268 | `http.rs` (131), `cli.rs` (18), `deploy.rs` (12), `apps.rs` (10), `consent.rs` (8), `runtime.rs` (8) |
| `nexus-shell-daemon-core` | ~235 | `publish.rs` (23), `browse.rs` (22), `iroh_runtime.rs` (22), `auth.rs` (21), `registry.rs` (18), `blob_serve.rs` (16), `config.rs` (13) |
| `nexus-coordinator-rs` | ~225 | `public_feed.rs` (35), `db.rs` (21), `kudos_ledger.rs` (16), `canary_input.rs` (14), `pii_redactor.rs` (14), `output_filter.rs` (11) |
| `nexus-worker-core` | ~196 | `invite.rs` (18), `consent.rs` (18), `config.rs` (16), `allowlist.rs` (16), `engine/state.rs` (17), `ollama.rs` (13) |
| `nexus-launcher` | ~32 | `auth.rs` (9), `main.rs` (6), `unlock.rs` (6), `driver_check.rs` (6) |
| `nexus-worker` | ~22 | `tests/e2e.rs` (11), `cli.rs` (11) |
| `nexus-events-core` | ~19 | `lib.rs` (19) |
| `nexus-trace-core` | ~13 | `propagation.rs` (5), `lib.rs` (3), `signed.rs` (2), `batch_log.rs` (2) |
| `nexus-test-harness` | ~12 | `tests/multi_daemon.rs` (8), `tests/cross_daemon_blob.rs` (1), `tests/blob_serve_coep.rs` (1), `lib.rs` (1) |
| `nexus-executor` | ~11 | `ipc.rs` (5), `task_runner.rs` (4), `main.rs` (2) |

## Rust Test Framework

**Runner:** cargo-nextest for speed; `cargo test --doc` for doctests (nextest does not support doctests).

**Config:** `.config/nextest.toml`

**Profiles:**
- `default` (local dev): `retries = 0`, `slow-timeout = 30s`, `failure-output = "immediate"`, fail-fast on first failure
- `ci`: `retries = 1` (for iroh relay network flakes), `fail-fast = false`, `slow-timeout = 30s` (terminate after 3 periods = 90s), JUnit XML output to `target/nextest/ci/junit.xml`

**Run commands:**
```bash
# Targeted fast iteration (single crate)
cargo nextest run -p <crate> --locked

# Full workspace unit + integration tests
cargo nextest run --workspace --locked

# Full workspace with CI profile (1 retry, JUnit)
cargo nextest run --workspace --profile ci --locked

# Doctests only (nextest does not handle these)
cargo test --workspace --locked --doc

# Benchmarks
cargo bench --bench pow
cargo bench --bench keystore
cargo bench --bench cold_start
```

## Rust Unit Test Patterns

### Standard structure

Tests are co-located within source files using `#[cfg(test)] mod tests { ... }`. 143 source files contain `#[cfg(test)]` modules across the workspace.

```rust
// Pattern from crates/nexus-core-rs/src/crypto.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generate_is_random() {
        let a = KeyPair::generate();
        let b = KeyPair::generate();
        assert_ne!(a.secret_bytes(), b.secret_bytes());
        assert_ne!(a.public_bytes(), b.public_bytes());
    }

    #[test]
    fn sign_then_verify_roundtrip() {
        let kp = KeyPair::generate();
        let msg = b"hello SBFB";
        let sig = kp.sign(msg);
        verify(&kp.public_bytes(), msg, &sig).expect("valid signature");
    }

    #[test]
    fn verify_rejects_wrong_message() {
        let kp = KeyPair::generate();
        let sig = kp.sign(b"original");
        let result = verify(&kp.public_bytes(), b"tampered", &sig);
        assert!(result.is_err());
    }
}
```

### Async tests (tokio)

Used for any test involving iroh nodes, HTTP routes, or tokio primitives:

```rust
#[tokio::test]
async fn test_two_daemons_boot_and_respond() {
    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");
    // ...assertions...
    cluster.shutdown().await.expect("graceful shutdown");
}
```

### HTTP route tests (axum tower::ServiceExt)

The `http.rs` file (131 tests) uses axum's `ServiceExt::oneshot()` pattern to drive the production router without spawning an HTTP server:

```rust
// Pattern from crates/nexus-shell-daemon/src/http.rs
fn build_test_router(state: Arc<DaemonHttpState>) -> Router {
    build_router(state, AuthState::new(TEST_TOKEN.to_string()), &[], None)
        .layer(middleware::from_fn(|mut req, next| async move {
            // Inject auth headers automatically for all tests
            let h = req.headers_mut();
            if !h.contains_key(AUTH_HEADER_NAME) {
                h.insert(AUTH_HEADER_NAME, HeaderValue::from_static(TEST_TOKEN));
            }
            // ...
            next.run(req).await
        }))
}

// Each test builds a request, calls .oneshot(), and asserts on the response
async fn mk_state() -> Arc<DaemonHttpState> {
    let node = create_node().await.expect("boot test node");
    // ...configure ephemeral state with tempdir...
    Arc::new(DaemonHttpState { /* ... */ })
}
```

### In-memory DB tests

Coordinator tests use `CoordinatorDb::open_in_memory()` for SQLite-backed tests without disk I/O:

```rust
let db = CoordinatorDb::open_in_memory().unwrap();
// ...insert, query, assert...
```

### Tempdir fixtures

Tests that need filesystem use `tempfile::TempDir`:

```rust
let tmp = TempDir::new().expect("tempdir");
let path = tmp.path().join("config.toml");
std::fs::write(&path, content).unwrap();
// ...test...
// TempDir auto-cleans on drop
```

## Rust Integration Tests

Integration tests live in `crates/*/tests/*.rs`:

| File | Crate | Purpose |
|------|-------|---------|
| `crates/nexus-test-harness/tests/multi_daemon.rs` | test-harness | 8 tests: 2-daemon boot, discovery, blob transfer, gossip exchange, storage sync, feed sync, offline catch-up, feed replay idempotent |
| `crates/nexus-test-harness/tests/cross_daemon_blob.rs` | test-harness | Cross-daemon zip blob publish + serve |
| `crates/nexus-test-harness/tests/blob_serve_coep.rs` | test-harness | COEP/COOP/CSP header verification on real daemon |
| `crates/nexus-coordinator-rs/tests/multi_daemon.rs` | coordinator | New-node E2E: insert 3 feed entries, join via ticket, verify sync |
| `crates/nexus-shell-daemon/tests/e2e.rs` | shell-daemon | 7 tests: --version, --help, stop stub, status stub, start + health, singleton enforcement, SIGINT graceful shutdown (Unix) |
| `crates/nexus-shell-daemon/tests/loopback_token.rs` | shell-daemon | 7 tests: token rotation, overlap window, AuthState dispatch, file watcher reload |
| `crates/nexus-shell-daemon-core/tests/pow_wire.rs` | daemon-core | PoW wire format verification |
| `crates/nexus-core-rs/tests/keystore_integration.rs` | core-rs | Keystore encrypt/decrypt roundtrip with real filesystem |
| `crates/nexus-core-rs/tests/relay_federation.rs` | core-rs | Relay federation configuration |
| `crates/nexus-worker/tests/e2e.rs` | worker | 11 tests: binary CLI, allowlist, engine state |

### Integration gate: `SBFB_INTEGRATION=1`

Tests requiring real iroh relay connectivity are gated behind `SBFB_INTEGRATION=1` environment variable. Without it, they print a skip message and return early:

```rust
fn integration_enabled() -> bool {
    std::env::var("SBFB_INTEGRATION").unwrap_or_default() == "1"
}

#[tokio::test]
async fn test_cross_daemon_gossip_exchange() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable gossip E2E");
        return;
    }
    // ...test body...
}
```

## Test Harness: `nexus-test-harness`

**Location:** `crates/nexus-test-harness/`

**Purpose:** Multi-daemon integration test infrastructure. Spawns N isolated `nexus-shell-daemon` processes with hermetic directories and distinct iroh keypairs.

**Key types:**
- `DaemonHandle` — spawns a single daemon, discovers its HTTP port from `running.json`, reads auth token, provides health check + HTTP client helpers (`get_info()`, `subscribe_curator()`, `publish_project()`, `browse_projects()`)
- `DaemonCluster` — spawns N daemons, provides bulk `shutdown()`

**Dependencies:** `tokio`, `reqwest`, `serde_json`, `tempfile`, `zip`, `anyhow`, `tracing`

**Pattern:**
```rust
let mut cluster = DaemonCluster::spawn(2).await?;
let daemon_a = &cluster.nodes[0];
let daemon_b = &cluster.nodes[1];

// Use reqwest::Client for HTTP interactions
let client = reqwest::Client::new();
let resp = client
    .post(format!("{}/api/daemon/feed/insert", daemon_a.http_url()))
    .header("X-SBFB-Token", &daemon_a.auth_token)
    .header("Host", format!("127.0.0.1:{}", daemon_a.http_port))
    .json(&serde_json::json!({ /* ... */ }))
    .send().await?;

cluster.shutdown().await?;
```

## Sprint 64 Adversarial Tests

### Phase C: Feed adversarial (6 tests)

All in `crates/nexus-coordinator-rs/src/public_feed.rs`:

| Test | Vector | Assertion |
|------|--------|-----------|
| `test_adversarial_fork_bomb_spam_rejected` | 20 rapid inserts from same author | Rate limiter accepts exactly `FEED_RATE_LIMIT_PER_MINUTE` (5), rejects 15 |
| `test_adversarial_payload_oversized_rejected` | repo_url exceeds `MAX_OPERATION_JSON_SIZE` | `validate_feed_operation()` returns error containing "exceeds" |
| `test_adversarial_bad_repo_url_rejected` | `javascript:`, `file://`, `data:`, `ftp://`, empty, etc. | Non-HTTPS URLs rejected by validation |
| `test_adversarial_bad_artifact_hash_rejected` | Empty, short, long, non-hex, nulls, spaces | All rejected by validation |
| `test_adversarial_seq_gap_detection` | Entry with seq=5 pointing prev_hash to fabricated hash | `verify_chain()` detects "broken linkage or fork" |
| `test_adversarial_cross_author_forgery_rejected` | Attacker signs entry claiming to be from another author's pubkey | `verify_entry()` rejects "signature verification failed" |

### Phase D: Crypto adversarial (4 tests)

Also in `crates/nexus-coordinator-rs/src/public_feed.rs`:

| Test | Vector | Assertion |
|------|--------|-----------|
| `test_adversarial_ed25519_forgery_feed_entry` | Random 64-byte signature instead of valid Ed25519 | `verify_entry()` rejects "signature verification failed" |
| `test_adversarial_blake3_tamper_canonical` | Change timestamp by 1 (1-bit flip in canonical) while keeping original hash | `verify_entry()` detects "entry_hash mismatch" |
| `test_adversarial_pow_nonce_difficulty_check` | Brute-force 1000 random nonces against 16-bit PoW | Overwhelming majority fail (<= 2 pass out of 1000) |
| `test_adversarial_future_timestamp_rejected` | Timestamp 31 days in future | `validate_feed_entry_timestamp()` rejects "more than 30 days" |

### New-node E2E test

In `crates/nexus-coordinator-rs/tests/multi_daemon.rs`:

```rust
#[tokio::test]
async fn test_new_node_full_sync_and_verify() {
    // 1. Spawn 2-node cluster
    // 2. Insert 3 feed entries on daemon 1
    // 3. Get feed ticket from daemon 1
    // 4. Daemon 2 joins via ticket
    // 5. Poll daemon 2 feed/status until count >= 3 (60s timeout)
    // 6. Verify last_seq >= 3
}
```

## Frontend Test Framework

### Vitest (unit tests)

**Config:** `web/vitest.config.ts`
- Environment: jsdom
- Setup: `web/src/test/setup.ts` (jest-dom matchers, localStorage stub, matchMedia stub, ResizeObserver stub, scrollIntoView stub)
- Include: `src/**/*.{test,spec}.{ts,tsx}`
- Exclude: `tests/**` (Playwright), `node_modules/**`, `dist/**`
- Globals: `true` (describe/it/expect/vi injected)
- Mocking: `clearMocks: true`, `restoreMocks: true`
- Coverage provider: v8

**Coverage thresholds:**
```
lines: 85%
functions: 90%
branches: 78%
statements: 85%
```

**Coverage scope:** Only tracked files:
- `src/lib/format.ts`
- `src/stores/projectStore.ts`
- `src/components/app/tabview/**/*.{ts,tsx}`
- `src/api/daemon.ts`
- `src/pages/BrowsedProject.tsx`

**Test files (22):**

| File | Area | Test Count Approx |
|------|------|----------|
| `web/src/api/__tests__/daemon.test.ts` | Daemon API client | ~30 |
| `web/src/api/__tests__/auth.test.ts` | Auth API | ~10 |
| `web/src/api/__tests__/coordinator.test.ts` | Coordinator API | ~10 |
| `web/src/bridge/__tests__/protocol.test.ts` | Bridge protocol schemas (Zod) | ~15 |
| `web/src/bridge/__tests__/watchdog.test.ts` | Bridge watchdog heartbeat | ~8 |
| `web/src/bridge/__tests__/useBridge.test.ts` | useBridge hook | ~8 |
| `web/src/sdk/pii/__tests__/policy.test.ts` | PII redaction policy | ~15 |
| `web/src/sdk/pii/__tests__/decoder.test.ts` | PII decoder | ~10 |
| `web/src/sdk/pii/__tests__/wrapper.test.ts` | PII wrapper | ~10 |
| `web/src/sdk/pii/__tests__/fallback.test.ts` | PII fallback | ~8 |
| `web/src/stores/__tests__/projectStore.test.ts` | Zustand store | ~15 |
| `web/src/lib/__tests__/format.test.ts` | Formatting utils | ~15 |
| `web/src/components/__tests__/RouteErrorBoundary.test.tsx` | Error boundary | ~5 |
| `web/src/components/__tests__/PanicWipeKeybind.test.tsx` | Panic wipe | ~5 |
| `web/src/components/__tests__/GpuConsentDialog.test.tsx` | GPU consent | ~5 |
| `web/src/components/__tests__/VerificationDetail.test.tsx` | Verification UI | ~5 |
| `web/src/components/command-palette/__tests__/CommandPalette.test.tsx` | Command palette | ~10 |
| `web/src/components/command-palette/__tests__/useCommandPalette.test.ts` | Command palette hook | ~5 |
| `web/src/components/app/tabview/__tests__/FileUploadBlock.test.tsx` | File upload | ~8 |
| `web/src/components/app/tabview/__tests__/TabViewRenderer.test.tsx` | Tab view renderer | ~10 |
| `web/src/pages/__tests__/BrowsedProject.test.tsx` | Browse project page | ~10 |
| `web/src/pages/__tests__/Deploy.test.tsx` | Deploy page | ~8 |

**Vitest patterns:**

Fetch mocking with `vi.stubGlobal`:
```typescript
function mockFetchOk<T>(body: T): void {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      mockFetchResponse({ status: 200, body }),
    ),
  );
}

beforeEach(() => { vi.restoreAllMocks(); });
afterEach(() => { vi.unstubAllGlobals(); });
```

Zod schema validation testing:
```typescript
it("rejects invalid status discriminator via Zod", async () => {
  mockFetchOk({ entries: [{ status: "banana", /* ... */ }] });
  await expect(listBrowse(BASE)).rejects.toThrow(/protocol error/);
});
```

Component testing with Testing Library:
```typescript
import { render, screen } from "@testing-library/react";

it("renders the expected UI", () => {
  render(<Component {...props} />);
  expect(screen.getByText("expected")).toBeInTheDocument();
});
```

### Playwright (E2E tests)

**Config:** `web/playwright.config.ts`
- Browser: Chromium only (`Desktop Chrome`)
- Workers: 1 (sequential, single coordinator at a time)
- Timeout: 60s per test
- Retries: 1 in CI, 0 locally
- Locale: `fr-FR`
- Trace: `retain-on-failure`
- Screenshots: `only-on-failure`
- Viewport: 1440x900

**Global setup:** `web/tests/global-setup.ts`
- Spawns a real `nexus-shell-daemon` binary (found in `target/release/` or `target/debug/`)
- Uses hermetic `tests/.tmp/nexus-grid/` directory
- Injects fixed auth token `deadbeef...` via `SBFB_AUTH_TOKEN` env
- Waits for `/health` endpoint (30s timeout)
- Writes PID to `.playwright-state.json` for teardown

**Global teardown:** `web/tests/global-teardown.ts`
- Reads PID from `.playwright-state.json` and kills daemon process

**Auth injection:** All requests get `X-SBFB-Token` header via `extraHTTPHeaders` in config. Tests probing 401/403 paths override per-request.

**Spec files (28):**

| Spec | Focus |
|------|-------|
| `loopback-auth.spec.ts` | Auth middleware: public health, 401 missing token, 401 wrong token, 200 valid, 403 cross-origin |
| `browse-click-project.spec.ts` | Route navigation, error boundary |
| `browse-daemon-offline.spec.ts` | Graceful offline state |
| `curators-flow.spec.ts` | Curator subscribe/unsubscribe flow |
| `command-palette.spec.ts` | Cmd+K palette |
| `bridge-heartbeat.spec.ts` | Bridge postMessage heartbeat |
| `bridge-push-event.spec.ts` | Bridge push event |
| `bridge-pii-redact.spec.ts` | PII redaction E2E |
| `blob-serve-coep.spec.ts` | COEP/COOP headers on blob-serve |
| `shell-onboarding-empty-state.spec.ts` | Empty state onboarding |
| `shell-add-coordinator.spec.ts` | Add coordinator flow |
| `my-projects-live.spec.ts` | My projects page |
| `my-network-live.spec.ts` | Network page |
| `project-detail-manifest.spec.ts` | Project detail view |
| `apps-tab-render.spec.ts` | App tab rendering |
| `tabview-schema-driven.spec.ts` | Schema-driven tabview |
| `gov-*.spec.ts` (12 files) | Governance app UI flows |

**Playwright test pattern:**
```typescript
import { test, expect } from "@playwright/test";
import { TEST_COORD_URL, TEST_COORD_NAME } from "./global-setup";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(
    ([url, nickname]) => {
      window.localStorage.setItem("nexus-grid:shell:v1",
        JSON.stringify({ state: { knownCoordinators: [{ url, nickname }] } }));
    },
    [TEST_COORD_URL, TEST_COORD_NAME],
  );
});

test("route renders", async ({ page }) => {
  await page.goto("/browse/...");
  await expect(page.getByTestId("back-to-browse")).toBeVisible({ timeout: 10_000 });
});
```

## Size-Limit Checks (6 budgets)

**Config:** `web/.size-limit.json`

| Chunk | Limit | Measure |
|-------|-------|---------|
| `main` (index-*.js) | 50 KB | raw (no gzip/brotli) |
| `vendor-react` | 290 KB | raw |
| `vendor-query` | 120 KB | raw |
| `vendor-ui` | 270 KB | raw |
| `CommandPalette` | 20 KB | raw |
| `css` (index-*.css) | 130 KB | raw |

**Run:** `npm run size` (alias for `size-limit`)

## CI/CD Infrastructure

### GitHub Actions Workflows (11 total)

| Workflow | File | Trigger | Purpose |
|----------|------|---------|---------|
| CI | `.github/workflows/ci.yml` | push/PR to master | Full verification: Rust (fmt, clippy, test) + Frontend (tsc, eslint, vitest, coverage, build, size-limit, playwright, scan-en-strings, npm audit, SPDX) |
| Rust CI | `.github/workflows/rust-ci.yml` | push/PR touching `crates/**` | 3-OS matrix (ubuntu, windows, macos-14 ARM), rustfmt, clippy, nextest + doctests, JUnit upload |
| Supply Chain | `.github/workflows/supply-chain.yml` | push/PR + weekly cron (Mon 08:00 UTC) | cargo-deny (RUSTSEC), pip-audit (PyPI), audit-ci (npm) |
| Build Worker | `.github/workflows/build-worker.yml` | push/PR touching worker crates | 7-target cross-compile matrix (linux glibc/musl x86+arm, windows, macOS Intel+ARM) |
| Release | `.github/workflows/release.yml` | tag v* | 3-OS x 3-binary matrix, cosign keyless OIDC, SLSA in-toto provenance, GitHub Release draft |
| Canary Monthly | `.github/workflows/canary-monthly.yml` | push touching CANARY.txt + weekly cron | Warrant canary signature verification + 45-day staleness check |
| Phase Review Cross-Check | `.github/workflows/phase-review-cross-check.yml` | PR to master | Verifies each `feat(sprintN): Phase X` commit has matching review.md |
| ShellCheck | `.github/workflows/shellcheck.yml` | push/PR touching `scripts/**/*.sh` | ShellCheck severity=warning on all shell scripts |
| Deploy | `.github/workflows/deploy.yml` | manual dispatch | SSH deploy to VPS (EU/US/Asia regions) |
| Mirror Codeberg | `.github/workflows/mirror-codeberg.yml` | push any branch/tag | Push-mirror to Codeberg (disaster recovery) |
| Build pkarr Image | `.github/workflows/build-pkarr-image.yml` | push touching `docker/pkarr-relay/**` | Docker image build + cosign + Trivy scan |

### Woodpecker CI

**Config:** `.woodpecker/ci-linux.yml`
- Runs on Codeberg CI or self-hosted agent
- Triggers: push to master/main, PR, manual
- Images pinned to SHA256 digests for supply chain security
- Steps: rust-fmt, rust-clippy, rust-test, rust-doctest, frontend-deps, frontend-typecheck, frontend-lint, frontend-test, frontend-build, frontend-size, spdx-check
- Uses `rust:1.94@sha256:...` and `node:20@sha256:...`

### Docker CI Image

**Location:** `docker/ci/Dockerfile`
```dockerfile
FROM rust:1.94
RUN rustup component add rustfmt clippy && \
    curl -LsSf https://get.nexte.st/0.9.133/linux | tar zxf - -C /usr/local/bin
WORKDIR /workspace
```

Pinned to match Woodpecker CI's Rust version to prevent rustfmt drift. Used for local dual-platform verification (Windows PowerShell + Linux Docker in parallel).

### Local Verification Script

**Location:** `scripts/verify.sh`
- 18-step sequential fail-fast pipeline
- Steps: cargo fmt, clippy, test, ~~ruff (Python removed)~~, tsc, eslint, vitest, coverage, build, size-limit, playwright (skippable with `--quick`), scan-en-strings, npm audit, SPDX check
- **Note:** Steps 4-8 (Python) are historical artifacts from pre-S50 and will fail on current codebase (Python removed)

## Code Quality Tools

### Rust

| Tool | Purpose | Config | CI Enforcement |
|------|---------|--------|---------------|
| `cargo fmt` | Formatting | Default rustfmt settings (no rustfmt.toml) | `cargo fmt --all --check` in all CI pipelines |
| `cargo clippy` | Linting | `-D warnings` (all warnings are errors) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `cargo-deny` | Supply chain audit | `deny.toml` (RUSTSEC, licenses, bans, sources) | `supply-chain.yml` weekly + on PR |
| `cargo-nextest` | Test runner | `.config/nextest.toml` | `rust-ci.yml` 3-OS matrix |

### Frontend

| Tool | Purpose | Config | CI Enforcement |
|------|---------|--------|---------------|
| TypeScript | Type checking | `tsconfig.app.json` | `npx tsc --noEmit -p tsconfig.app.json` |
| ESLint | Linting | `web/eslint.config.js` (flat config, typescript-eslint + react-hooks + react-refresh) | `npm run lint` |
| Vitest | Unit tests | `web/vitest.config.ts` | `npm run test:unit` + `npm run test:coverage` |
| Playwright | E2E tests | `web/playwright.config.ts` | `npx playwright test` |
| size-limit | Bundle budget | `web/.size-limit.json` | `npm run size` |
| audit-ci | npm audit | `web/audit-ci.json` (critical severity) | `npm run audit:ci` |
| scan-en-strings | French-only UI | `web/scripts/scan-en-strings.sh` | Step in CI |
| check-spdx | License headers | `scripts/check-spdx.sh` | SPDX header in first 5 lines of all .rs/.ts/.tsx |
| shellcheck | Shell scripts | severity=warning | `.github/workflows/shellcheck.yml` |

### ESLint Config Details

`web/eslint.config.js` (flat config):
- Extends: `js.configs.recommended`, `tseslint.configs.recommended`, `reactHooks.configs.flat.recommended`, `reactRefresh.configs.vite`
- `allowConstantExport: true` for shadcn v4 pattern
- Test files: `@typescript-eslint/no-explicit-any` off, vitest globals registered

## Dual-Platform Verification Workflow

The project enforces dual-platform verification before every commit phase:

1. **Windows (PowerShell):** cargo fmt, clippy, nextest, build release
2. **Linux (Docker sbfb-ci image):** Same pipeline in `docker/ci/Dockerfile` container

Both must pass before committing. This catches OS-specific issues (named pipes on Windows, UDS on Unix, file path handling, etc.).

## Test Gaps & Carry Items

### Known gaps (from CLAUDE.md carry items)

| Item | Priority | Description |
|------|----------|-------------|
| `P2-FEED-INSERT-NO-AUTH-TIER` | 3/3 MANDATORY S65 | `feed_insert` handler does not verify auth tier before insert |
| `P2-COVERAGE-DEPLOY-E2E` | 2/3 | Missing E2E coverage for deploy flow |
| `P2-PLAYWRIGHT-SPECS-STALE` | 2/3 | Some Playwright specs may reference stale UI |
| `P2-VERIFY-ENTRY-VERSION-GUARD` | 1/3 | `verify_entry` does not check version field; exempted pre-launch |
| `P2-ORPHAN-REPUBLISH-RECOVERY` | 1/3 | No republish DB-to-iroh-docs after publish fail + tail-safe skip |
| `P2-FEED-JOIN-HANDLE-LEAK` | 1/3 | `feed_join` fire-and-forget with no shutdown channel |

### Coverage gaps in code

- **Frontend coverage is scoped:** Only 5 source files are tracked (`format.ts`, `projectStore.ts`, `tabview/**`, `daemon.ts`, `BrowsedProject.tsx`). The rest of `web/src/` has no coverage enforcement.
- **Integration tests gated:** All multi-daemon tests requiring iroh relay are behind `SBFB_INTEGRATION=1` and only run manually or in network-enabled CI.
- **No Rust coverage tracking:** The project does not run `cargo-llvm-cov` or `cargo-tarpaulin` in CI. Coverage is tracked by convention (test count) not by percentage.
- **Python test steps stale:** `scripts/verify.sh` steps 4-8 reference Python packages that were removed in S50-S51. The script will fail at step 4 on current codebase.

### Areas with low test density

- `crates/nexus-executor/` (11 tests) — task runner and IPC are lightly tested
- `crates/nexus-trace-core/` (13 tests) — OTel integration mostly structural
- `crates/nexus-launcher/` (32 tests) — unlock flow, tray icon, driver check

## Benchmarks

**Location:** `crates/nexus-core-rs/benches/`

| Bench | File | Purpose | Target |
|-------|------|---------|--------|
| PoW solving | `pow.rs` | Hashcash solver at 3 difficulty levels (12/18/20 bits) | 12-bit <50ms, 18-bit <500ms, 20-bit <2000ms |
| Keystore | `keystore.rs` | Argon2id key derivation + AES-GCM seal/unseal | Regression guard |
| Cold start | `crates/nexus-executor/benches/cold_start.rs` | Task runner cold start latency | Regression guard |

Run: `cargo bench --bench pow`, `cargo bench --bench keystore`, `cargo bench --bench cold_start`

## Supply Chain Checks

### Rust: cargo-deny (`deny.toml`)

- **Advisories:** RUSTSEC database, yanked = deny, unmaintained = workspace-only
- **Known ignore:** `RUSTSEC-2026-0097` (rand 0.8 ThreadRng unsoundness, SBFB uses OsRng only)
- **Licenses:** SPDX allowlist (Apache-2.0, MIT, BSD, ISC, Unicode, Zlib, CC0-1.0, MPL-2.0, AGPL-3.0)
- **Sources:** crates-io + git allowlist
- **Targets:** 5 platforms (linux x86/arm, windows, macOS Intel/ARM)

### npm: audit-ci (`web/audit-ci.json`)

- Severity threshold: critical
- Zero allowlist entries

### Weekly cron

`supply-chain.yml` runs every Monday 08:00 UTC to catch newly published advisories.

## Canary Verification

**Workflow:** `.github/workflows/canary-monthly.yml`
- Verifies `CANARY.txt` Ed25519 signature via `scripts/verify-canary.sh`
- Weekly cron checks staleness (>45 days = error, >30 days = warning)
- Dead-man-switch design: manual re-signing required, no automated publishing

## Release Build Verification

**Workflow:** `.github/workflows/release.yml`
- 3-OS x 3-binary matrix (nexus-worker, nexus-shell-daemon, nexus-launcher)
- Deterministic release profile: `codegen-units = 1`, `lto = "fat"`, `strip = "symbols"`, `panic = "abort"`
- Cosign keyless OIDC via GitHub Actions
- SLSA in-toto provenance attestation
- SHA256 checksum files

**Cross-compile builds:** `.github/workflows/build-worker.yml`
- 7 targets including static musl builds
- Static linking verification on x86_64-musl

---

*Tests & quality analysis: 2026-05-18*
