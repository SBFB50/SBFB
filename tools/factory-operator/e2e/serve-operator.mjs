// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Hermetic Operator launcher for the Playwright T1 (Sprint 80 Phase B).
// Ensures the greenfield front is built into tools/factory-operator/bundle,
// then spawns the REAL `sbfb-factory` Operator server over it. The server
// reads SBFB_AUTH_TOKEN + SBFB_HOME from the environment injected by
// playwright.config.ts (no real ~/.sbfb is ever touched).
//
// The Operator binary is resolved from SBFB_FACTORY_BIN (CI pre-builds it
// and passes the path); without it we fall back to `cargo run` (slower,
// fine for local dev). The server prints `READY 127.0.0.1:<port>` on
// stdout once it is listening; Playwright's webServer waits on the URL.
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { build } from 'vite'

const here = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(here, '..') // tools/factory-operator
const repoRoot = path.resolve(root, '..', '..') // repo root — the Operator resolves the bundle relative to here
const port = process.env.OPERATOR_TEST_PORT || '3111'

// ALWAYS rebuild so the T1 exercises the CURRENT source, never a stale
// gitignored `bundle/` left over from a previous run (Codex P1 — strict
// auto-hermeticity, independent of CI ordering).
console.log('[serve-operator] building the front bundle…')
await build({ root, configFile: path.join(root, 'vite.config.ts') })

const bin = process.env.SBFB_FACTORY_BIN
const cmd = bin || 'cargo'
const args = bin
  ? ['operator', 'serve', '--port', port]
  : ['run', '--quiet', '-p', 'sbfb-factory', '--', 'operator', 'serve', '--port', port]

console.log(`[serve-operator] spawning Operator on 127.0.0.1:${port} (${bin ? 'prebuilt' : 'cargo run'})`)
const child = spawn(cmd, args, { cwd: repoRoot, stdio: 'inherit', env: process.env })

child.on('exit', (code) => process.exit(code ?? 0))
for (const sig of ['SIGTERM', 'SIGINT']) {
  process.on(sig, () => child.kill(sig))
}
