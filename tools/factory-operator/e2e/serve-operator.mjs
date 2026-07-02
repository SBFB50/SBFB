// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Hermetic Operator launcher for the Playwright T1 (Sprint 80 Phase B,
// hermetic workspace Phase I). Builds the greenfield front, seeds a
// PER-RUN git fixture workspace + SBFB_HOME temp dir, then spawns the
// REAL `sbfb-factory` Operator server WITH cwd=<fixture workspace> — the
// Operator resolves repo_root() (and every sprint-history git subprocess)
// off its cwd, so diff/gates/procédé/documents all read the sealed
// fixture, never the real repo (closes TEST-ISOLATION-SBFB-HOME).
//
// The Operator binary is resolved from SBFB_FACTORY_BIN (CI pre-builds it
// and passes the path); without it we `cargo build` FROM THE REPO ROOT and
// then spawn the built binary ourselves. Never `cargo run` with
// cwd=<fixture>: cargo discovers `.cargo/config.toml` from the CWD, so a
// Temp-dir cwd silently drops the repo's rustflags (/Brepro,
// incremental=false), invalidates every fingerprint and rebuilds the whole
// workspace — which is how the webServer timed out on first wiring. The
// server prints `READY 127.0.0.1:<port>` on stdout once it is listening;
// Playwright's webServer waits on the URL.
//
// Teardown: Playwright tree-kills this process and its children (the
// reliable mechanism on win32 — SIGTERM relaying is emulated there); the
// temp dirs are removed best-effort on exit and otherwise left to the OS
// temp cleaner.
import { spawn, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { build } from 'vite'
import { seedFixtureWorkspace } from './fixture-workspace.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(here, '..') // tools/factory-operator
const repoRoot = path.resolve(root, '..', '..') // real repo — cargo manifest only
const port = process.env.OPERATOR_TEST_PORT || '3111'

// ALWAYS rebuild so the T1 exercises the CURRENT source, never a stale
// gitignored `bundle/` left over from a previous run (Codex P1 — strict
// auto-hermeticity, independent of CI ordering).
console.log('[serve-operator] building the front bundle…')
await build({ root, configFile: path.join(root, 'vite.config.ts') })

console.log('[serve-operator] seeding the hermetic git workspace…')
const workspace = seedFixtureWorkspace(path.join(root, 'bundle'))
const sbfbHome = fs.mkdtempSync(path.join(os.tmpdir(), 'sbfb-op-e2e-home-'))
console.log(`[serve-operator] workspace=${workspace} SBFB_HOME=${sbfbHome}`)

let bin = process.env.SBFB_FACTORY_BIN
if (!bin) {
  console.log('[serve-operator] cargo build -p sbfb-factory (from the repo root)…')
  const build = spawnSync('cargo', ['build', '--locked', '-p', 'sbfb-factory'], {
    cwd: repoRoot, // cargo config discovery stays anchored at the repo
    stdio: 'inherit',
  })
  if (build.status !== 0) process.exit(build.status ?? 1)
  bin = path.join(
    repoRoot,
    'target',
    'debug',
    process.platform === 'win32' ? 'sbfb-factory.exe' : 'sbfb-factory',
  )
}

console.log(`[serve-operator] spawning Operator on 127.0.0.1:${port} (${process.env.SBFB_FACTORY_BIN ? 'prebuilt' : 'freshly built'})`)
const child = spawn(bin, ['operator', 'serve', '--port', port], {
  cwd: workspace, // ← repo_root() + sprint-history resolve HERE (hermetic)
  stdio: 'inherit',
  env: { ...process.env, SBFB_HOME: sbfbHome },
})

function cleanup() {
  for (const dir of [workspace, sbfbHome]) {
    try {
      fs.rmSync(dir, { recursive: true, force: true, maxRetries: 3 })
    } catch {
      // best-effort: on win32 the tree-kill can race an open handle; the
      // OS temp cleaner owns the leftovers (per-run unique names).
    }
  }
}

child.on('exit', (code) => {
  cleanup()
  process.exit(code ?? 0)
})
for (const sig of ['SIGTERM', 'SIGINT']) {
  process.on(sig, () => child.kill(sig))
}
