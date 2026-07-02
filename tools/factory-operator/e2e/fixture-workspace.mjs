// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase I — hermetic git workspace for the Operator under test.
//
// The Operator resolves EVERYTHING off the spawned process: `repo_root()`
// is `git rev-parse --show-toplevel` from the process cwd
// (process.rs:56-67) and the sprint-history git subprocesses run in the
// cwd too (sprint_history.rs `git_cmd`). Spawning the binary with
// cwd=<this workspace> therefore points state.root (diff/gates/documents)
// AND sprint-history at the SAME sealed fixture — no Rust override needed,
// and the REAL repo (including in-flight phase work) never leaks into the
// T1 assertions (preflight S3-F2, closes TEST-ISOLATION-SBFB-HOME).
//
// The workspace carries exactly what the specs assert on:
//  - a git repo with a `Sprint 1 Phase A` commit (sprint_history PHASE_RE)
//    whose .planning artifacts record verdicts (EXECUTE / PASS) — the
//    Procédé inspector restitutes them, never fabricates;
//  - an UNSTAGED edit so GET /api/git/diff yields real working-tree hunks;
//  - prompts/agent/base.md + AGENTS.md (documents surface pinned inputs);
//  - the freshly BUILT front bundle copied at tools/factory-operator/bundle
//    (run_server resolves the ServeDir bundle off the same root,
//    operator_server.rs:232-236).
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

function git(cwd, ...args) {
  execFileSync('git', ['-C', cwd, ...args], { stdio: 'pipe' })
}

function write(root, rel, content) {
  const abs = path.join(root, rel)
  fs.mkdirSync(path.dirname(abs), { recursive: true })
  fs.writeFileSync(abs, content, 'utf8')
}

/** The phase-commit body: enough sections for extract_body_sections to
 * restitute something real; the parsers are tolerant of missing ones. */
const PHASE_BODY = `## Objectif
Seed hermétique du workspace de test (fixture T1).

## Livrables
- greeting.txt enrichi (diff de commit non vide pour le bi-usage V2/U7)

## Tests
Delta cumulé : fixture seed (aucune suite réelle ne tourne ici).

## Scope cuts respectés
Aucun.
`

/**
 * Seeds a fresh hermetic workspace under a per-run temp dir and returns
 * its absolute path. `bundleSrc` is the freshly built front bundle to copy
 * at tools/factory-operator/bundle inside the workspace.
 */
export function seedFixtureWorkspace(bundleSrc) {
  const ws = fs.mkdtempSync(path.join(os.tmpdir(), 'sbfb-op-e2e-ws-'))

  git(ws, 'init', '-q', '-b', 'main')
  // Local identity only — never touches the user's global config.
  git(ws, 'config', 'user.email', 'fixture@sbfb.test')
  git(ws, 'config', 'user.name', 'SBFB Fixture')
  git(ws, 'config', 'commit.gpgsign', 'false')
  git(ws, 'config', 'core.hooksPath', path.join(ws, '.no-hooks'))

  // --- Documents surface inputs (pinned LLM inputs + file map) ---
  write(ws, 'AGENTS.md', '# AGENTS (fixture)\n\nCanon portable factice du workspace de test.\n')
  write(
    ws,
    'prompts/agent/base.md',
    '# Base prompt (fixture)\n\nPrompt de base factice, épinglé par la surface Documents.\n',
  )
  write(ws, 'README.md', '# Fixture workspace\n\nWorkspace hermétique du T1 Operator.\n')
  write(ws, 'src/greeting.txt', 'bonjour\n')

  // --- .planning artifacts the Procédé inspector restitutes ---
  write(
    ws,
    '.planning/active/sprint1_kickoff.md',
    '# Sprint 1 — Kickoff (fixture)\n\nSprint factice du workspace hermétique.\n\n## Roadmap\nFixture.\n',
  )
  write(ws, '.planning/active/sprint1_plan.md', '# Sprint 1 — Plan (fixture)\n\n## Phase A\nSeed.\n')
  write(
    ws,
    '.planning/active/sprint1_phase_a_preflight.md',
    '# Sprint 1 Phase A — Preflight (fixture)\n\n## Verdict: EXECUTE\n',
  )
  write(
    ws,
    '.planning/active/sprint1_phase_a_review.md',
    '# Sprint 1 Phase A — Review (fixture)\n\n## Verdict: PASS\n',
  )
  write(
    ws,
    '.planning/active/sprint1_phase_a_codex_review.md',
    '# Sprint 1 Phase A — Codex (fixture)\n\n1 CONFIRMÉ / 0 PARTIEL / 0 GAP\n',
  )

  git(ws, 'add', '-A')
  // The seed commit anchors `find_entry_tip(1)` (it greps "Sprint 0 Phase"):
  // without it the commit range degrades to HEAD~50..HEAD, which is INVALID
  // on a 2-commit repo → zero commits collected → the Phase A row loses its
  // commit and the bi-usage diff has nothing to render. (The HEAD~50
  // fallback being brittle on young repos is a latent sprint_history.rs
  // robustness gap — carried as dette, not patched from a test harness.)
  git(ws, 'commit', '-q', '-m', 'chore(fixture): Sprint 0 Phase A — seed hermetic workspace')

  // The PHASE commit (PHASE_RE: `feat(scope): Sprint N Phase X — titre`) —
  // it touches a file so commit_diff_data renders bi-usage hunks.
  write(ws, 'src/greeting.txt', 'bonjour\nmonde\n')
  git(ws, 'add', 'src/greeting.txt')
  git(ws, 'commit', '-q', '-m', 'feat(fixture): Sprint 1 Phase A — hermetic seed change', '-m', PHASE_BODY)

  // UNSTAGED working-tree edit → GET /api/git/diff yields real hunks.
  fs.appendFileSync(path.join(ws, 'README.md'), '\nligne non-stagée pour le diff working-tree.\n')

  // --- The built front bundle, where run_server resolves it ---
  const bundleDst = path.join(ws, 'tools', 'factory-operator', 'bundle')
  fs.cpSync(bundleSrc, bundleDst, { recursive: true })

  return ws
}
