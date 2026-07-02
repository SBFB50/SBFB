// SPDX-License-Identifier: AGPL-3.0-or-later
// Shared constants for the hermetic T1 harness (playwright.config.ts +
// specs + e2e/serve-operator.mjs + e2e/serve-fixture-daemon.mjs). A fixed
// 64-hex bearer keeps the boot deterministic; SBFB_HOME and the git
// workspace are per-run temp dirs minted by serve-operator.mjs so no real
// ~/.sbfb — and no real repo — is ever touched (Sprint 80 Phase I,
// closes TEST-ISOLATION-SBFB-HOME).
export const OPERATOR_TEST_PORT = Number(process.env.OPERATOR_TEST_PORT || 3111)
export const OPERATOR_TEST_TOKEN = 'a'.repeat(64)

// Fixture upstream daemon (sub-test (3), full-stack deterministic SSE).
// The Operator's SBFB_DAEMON_ENDPOINT / SBFB_OLLAMA_ENDPOINT point here;
// playwright.config.ts forwards the reply constants as env (the .mjs child
// cannot import this .ts module).
export const FIXTURE_DAEMON_PORT = Number(process.env.FIXTURE_DAEMON_PORT || 3112)
/** The Network arm's result_text — asserted rendered, PO-14 single Done. */
export const FIXTURE_NETWORK_RESULT = 'reponse-fixture-network'
/** The Ollama arm's NDJSON deltas — asserted streamed then joined. */
export const FIXTURE_OLLAMA_DELTAS = ['Bonjour ', 'monde fixture'] as const
export const FIXTURE_OLLAMA_TEXT = FIXTURE_OLLAMA_DELTAS.join('')
