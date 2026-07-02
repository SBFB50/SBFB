// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase I — T2 acceptance harness (README §4 testability gate).
//
// Runs the full local front pipeline (discipline gates → build → size →
// vitest → hermetic Playwright T1) and writes the COMMITTED acceptance
// artifact `.planning/active/sprint80_t2_acceptance.json` (fixes P3-6 —
// the previous acceptance artifact was a gitignored dot-file).
//
// Determinism rules (prior-art: Node test/wpt/status, Deno WPT
// expectations — preflight §10.2): the artifact is a PROJECTION of the
// runner reports, with stable key order, an enum verdict per entry, and
// an ALLOWLIST of fields — never timestamps, durations, absolute paths,
// ports, env, response headers (the session cookie is a per-boot secret)
// nor raw runner output. Same green run ⇒ byte-identical artifact.
//
// `RIG-ABSENT` is illegitimate by construction (kickoff §T2): the
// Operator is 100 % loopback-bound (127.0.0.1) — there is NO rig to be
// absent, so this harness has no rig-detection branch at all. Verdicts:
// PASS | BLOCK{diagnosis}.
import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(here, '..') // tools/factory-operator
const repoRoot = path.resolve(root, '..', '..')
const ARTIFACT = path.join(repoRoot, '.planning', 'active', 'sprint80_t2_acceptance.json')

// win32: .cmd shims (npm/npx) cannot be spawned without a shell since the
// Node CVE-2024-27980 hardening — spawnSync returns EINVAL with status
// null, which would project EVERY gate to BLOCK. All argument tokens
// below are fixed literals, so shell:true is safe.
const useShell = process.platform === 'win32'
const npmCmd = 'npm'
const npxCmd = 'npx'

function run(cmd, args) {
  // Runner output stays on the console (diagnosis for humans); ONLY the
  // enum verdict is projected into the artifact.
  const res = spawnSync(cmd, args, { cwd: root, stdio: 'inherit', shell: useShell })
  return res.status === 0 ? 'PASS' : 'BLOCK'
}

// --- 1. Discipline gates (one verdict per gate, stable order) ---
const gates = {}
for (const [key, script] of [
  ['no-radix', 'gate:no-radix'],
  ['no-tw-config', 'gate:no-tw-config'],
  ['scan-front-discipline', 'gate:scan-front'],
  ['i18n-verdict', 'gate:i18n-verdict'],
  ['i18n-parity', 'gate:i18n-parity'],
  ['accessibility-system', 'gate:accessibility-system'],
]) {
  gates[key] = run(npmCmd, ['run', script])
}
gates['build'] = run(npmCmd, ['run', 'build'])
gates['size-limit'] = run(npmCmd, ['run', 'size'])
gates['vitest'] = run(npmCmd, ['run', 'test:unit'])

// --- 2. Hermetic Playwright T1 → per-scenario verdicts ---
// Known titles map to stable scenario ids; an unmapped title degrades to a
// deterministic slug (never dropped — silent truncation would read as
// "covered" when it is not).
const TITLE_TO_ID = new Map([
  ['boots cookie-authenticated and renders the shell, CSP-clean', 'boot_cookie_csp'],
  ['bootstrap mints an HttpOnly session cookie and 303-redirects to /', 'bootstrap_cookie_303'],
  [
    'Documents inspector restitutes the git-backed file map and pinned LLM inputs',
    'documents_map',
  ],
  ['altitude shift lands instantly under reduced motion (anti-déco)', 'motion_reduced'],
  ['sub-test (2): composing a benign intention creates a session', 'composer_session'],
  ['sub-test (3a): full-stack SSE — the local arm streams tokens then ONE Done', 'sse_local_token_done'],
  [
    'sub-test (3b): the Network arm — zero deltas, exactly ONE Done (PO-14 full-stack)',
    'sse_network_single_done',
  ],
  ['sub-test (4): a sensitive intention hits the MUR and never opens the stream', 'mur_gated_no_spawn'],
  [
    'VERIFY-plein shows the bespoke diff-viewer + the live gates band; ÉTAT never says a verdict',
    'verify_diff_gates',
  ],
  [
    'the Procédé inspector restitutes ≥1 phase verdict (never a score) and the diff bi-usage renders a past commit',
    'procede_verdict_bi_usage',
  ],
])

function slug(title) {
  return title
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, 60)
}

const pw = spawnSync(npxCmd, ['playwright', 'test', '--reporter=json'], {
  cwd: root,
  encoding: 'utf8',
  shell: useShell,
  maxBuffer: 64 * 1024 * 1024,
})

const scenarios = {}
let pwParseError = null
try {
  const report = JSON.parse(pw.stdout)
  const walk = (suite) => {
    for (const child of suite.suites ?? []) walk(child)
    for (const spec of suite.specs ?? []) {
      // Strict projection (review P3-3): only a spec whose every test run
      // is 'expected' projects PASS — skipped/flaky/unexpected all BLOCK
      // (spec.ok alone lets a skipped test read as covered).
      const statuses = (spec.tests ?? []).map((t) => t.status)
      const pass =
        spec.ok && statuses.length > 0 && statuses.every((s) => s === 'expected')
      scenarios[TITLE_TO_ID.get(spec.title) ?? slug(spec.title)] = pass ? 'PASS' : 'BLOCK'
    }
  }
  for (const suite of report.suites ?? []) walk(suite)
} catch {
  pwParseError = 'playwright json report unparsable'
}
if (Object.keys(scenarios).length === 0 && !pwParseError) {
  pwParseError = 'playwright report contains zero specs'
}
// A silently REMOVED spec must not read as covered (Codex livrable-8):
// every known scenario id must be present in the report, else BLOCK.
for (const id of TITLE_TO_ID.values()) {
  if (!(id in scenarios)) scenarios[id] = 'BLOCK'
}

// --- 3. Projection (stable key order, allowlisted fields only) ---
const failedGates = Object.keys(gates).filter((k) => gates[k] !== 'PASS')
const failedScenarios = Object.keys(scenarios)
  .sort()
  .filter((k) => scenarios[k] !== 'PASS')
const blocked = failedGates.length > 0 || failedScenarios.length > 0 || pwParseError !== null

const diagnosis = blocked
  ? [
      ...failedGates.map((g) => `gate:${g}`),
      ...failedScenarios.map((s) => `scenario:${s}`),
      ...(pwParseError ? [pwParseError] : []),
    ].join(', ')
  : null

const sortedScenarios = {}
for (const key of Object.keys(scenarios).sort()) sortedScenarios[key] = scenarios[key]

const artifact = {
  suite: 'sprint80-operator-t2',
  status: blocked ? 'BLOCK' : 'PASS',
  diagnosis,
  gates,
  scenarios: sortedScenarios,
}

fs.writeFileSync(ARTIFACT, `${JSON.stringify(artifact, null, 2)}\n`, 'utf8')
console.log(`[t2-acceptance] ${artifact.status}${diagnosis ? ` — ${diagnosis}` : ''}`)
console.log(`[t2-acceptance] artifact: ${path.relative(repoRoot, ARTIFACT)}`)
process.exit(blocked ? 1 : 0)
