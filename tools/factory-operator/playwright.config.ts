// SPDX-License-Identifier: AGPL-3.0-or-later
import { defineConfig, devices } from '@playwright/test'
import {
  FIXTURE_DAEMON_PORT,
  FIXTURE_NETWORK_RESULT,
  FIXTURE_OLLAMA_DELTAS,
  OPERATOR_TEST_PORT,
  OPERATOR_TEST_TOKEN,
} from './e2e/fixtures'

// Sprint 80 Phase B — hermetic T1 skeleton; Phase I — hermetic workspace +
// deterministic upstream fixture. `serve-operator.mjs` builds the greenfield
// front, seeds a PER-RUN git fixture workspace + SBFB_HOME temp dir, and
// spawns the REAL Operator Rust server over the BUILT bundle with
// cwd=<fixture> (repo_root() and the sprint-history subprocesses resolve
// there — the real repo never leaks into the T1). The second webServer is
// the upstream fixture daemon (Network submit/poll/result + fake Ollama
// NDJSON): SBFB_DAEMON_ENDPOINT / SBFB_OLLAMA_ENDPOINT point the REAL
// dispatch (from_provider → run, MUR gate included) at it, giving the
// full-stack SSE sub-test (3) a deterministic token→Done stream — one Done
// (PO-14), zero new prod code.
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: `http://127.0.0.1:${OPERATOR_TEST_PORT}`,
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: [
    {
      command: 'node e2e/serve-operator.mjs',
      url: `http://127.0.0.1:${OPERATOR_TEST_PORT}/`,
      timeout: 180_000,
      reuseExistingServer: false,
      stdout: 'pipe',
      stderr: 'pipe',
      env: {
        OPERATOR_TEST_PORT: String(OPERATOR_TEST_PORT),
        SBFB_AUTH_TOKEN: OPERATOR_TEST_TOKEN,
        // SBFB_HOME is minted per-run by serve-operator.mjs (mkdtemp).
        // Deterministic upstream for sub-test (3): the env is read at
        // request time (provider_router.rs:282,155), so boot order with
        // the fixture entry below is indifferent. Poll/timeout mirror the
        // Rust tests (provider_router.rs:795-796).
        SBFB_DAEMON_ENDPOINT: `http://127.0.0.1:${FIXTURE_DAEMON_PORT}`,
        SBFB_NETWORK_POLL_INTERVAL_MS: '20',
        SBFB_NETWORK_TIMEOUT_SECS: '30',
        SBFB_OLLAMA_ENDPOINT: `http://127.0.0.1:${FIXTURE_DAEMON_PORT}`,
      },
    },
    {
      command: 'node e2e/serve-fixture-daemon.mjs',
      url: `http://127.0.0.1:${FIXTURE_DAEMON_PORT}/`,
      timeout: 30_000,
      reuseExistingServer: false,
      stdout: 'pipe',
      stderr: 'pipe',
      env: {
        FIXTURE_DAEMON_PORT: String(FIXTURE_DAEMON_PORT),
        FIXTURE_NETWORK_RESULT,
        FIXTURE_OLLAMA_DELTAS: FIXTURE_OLLAMA_DELTAS.join('|'),
      },
    },
  ],
})
