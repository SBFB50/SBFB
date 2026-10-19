import { describe, expect, test } from 'claude-code/testing'

import type { NexusSnapshot } from '../types'
import {
  alerts,
  checklist,
  ciSummary,
  claimsDone,
  codexStatus,
  currentPhase,
  goldenErrors,
  goldenSources,
  isGoldenSource,
  isReviewPass,
  maxRound,
  maxSprint,
  mutationMarkers,
  preflightVerdict,
  residueFiles,
  reviewVerdict,
  statusLine,
  stopProblems,
  untrackedCode,
  phaseCommits,
  phaseRows,
  planPhases,
  sprintLog,
  sprintPart,
} from './lib'

const SNAP: NexusSnapshot = {
  root: 'C:/nexus',
  sprint: 83,
  phase: 'm1',
  preflightVerdict: 'PLAN-ADAPT',
  reviewVerdict: null,
  isReviewPass: false,
  codex: 'absent',
  rounds: 0,
  dirty: 31,
  untrackedCode: ['crates/a/src/x.rs'],
  residueFiles: [],
  mutationMarkers: [],
  goldenErrors: [],
  ahead: 40,
  ci: 'rouge',
  ciFailed: ['CI', 'Rust CI'],
  updatedAt: 0,
}

describe('sprint et phase', () => {
  test('le sprint est le plus grand N', async () => {
    expect(maxSprint(['sprint82_x.md', 'sprint83_plan.md', 'README.md', 'sprint9_a.md'])).toBe(83)
    expect(maxSprint(['README.md'])).toBe(null)
  })

  test('la phase est celle du preflight le plus récent', async () => {
    const entries = [
      { name: 'sprint83_phase_m_preflight.md', mtimeMs: 10 },
      { name: 'sprint83_phase_m1_preflight.md', mtimeMs: 20 },
      { name: 'sprint82_phase_z_preflight.md', mtimeMs: 99 },
      { name: 'sprint83_phase_m1_review.md', mtimeMs: 30 },
    ]
    expect(currentPhase(entries, 83)).toBe('m1')
    expect(currentPhase([], 83)).toBe(null)
  })

  test('verdict de preflight', async () => {
    expect(preflightVerdict('## Verdict G8 : PLAN-ADAPT\n')).toBe('PLAN-ADAPT')
    expect(preflightVerdict('rien')).toBe('présent')
    expect(preflightVerdict(null)).toBe(null)
  })
})

describe('review et Codex', () => {
  test('PASS exact uniquement', async () => {
    expect(isReviewPass('x\n## Verdict: PASS\n')).toBe(true)
    expect(isReviewPass('## Verdict : PASS')).toBe(true)
    expect(isReviewPass('## Verdict: PASS-PENDING')).toBe(false)
    expect(isReviewPass('## Verdict: PASS\nCodex EN ATTENTE')).toBe(false)
    expect(isReviewPass(null)).toBe(false)
    expect(reviewVerdict('## Verdict: CONCERN\n')).toBe('CONCERN')
  })

  test('statut Codex', async () => {
    expect(codexStatus(null)).toBe('absent')
    expect(codexStatus('Statut : GAP')).toBe('gap')
    expect(codexStatus('Partiels : 2')).toBe('partiel')
    expect(codexStatus('D1 CONFIRMÉ fichier.rs:12')).toBe('ok')
    expect(codexStatus('du texte')).toBe('sans-verdict')
  })

  test('rounds', async () => {
    const names = ['sprint83_phase_m_review_r3.md', 'sprint83_phase_m_review_r7.md', 'sprint83_phase_m1_review_r2.md']
    expect(maxRound(names, 83, 'm')).toBe(7)
    expect(maxRound(names, 83, 'm1')).toBe(2)
    expect(maxRound(names, 83, 'n')).toBe(0)
  })
})

describe('arbre de travail', () => {
  test('?? sous crates/ ou web/src seulement', async () => {
    const porcelain = '?? crates/a/src/new.rs\n?? outputs/x\n M crates/b.rs\n?? web/src/p.ts\n'
    expect(untrackedCode(porcelain)).toEqual(['crates/a/src/new.rs', 'web/src/p.ts'])
  })

  test('motifs du Check 0bis', async () => {
    expect(residueFiles(['crates/x/zz_probe.rs', 'crates/x/real.rs', 'a/b_scratch.rs', 'c.bak'])).toEqual([
      'crates/x/zz_probe.rs',
      'a/b_scratch.rs',
      'c.bak',
    ])
    const diff = '+++ b/crates/x.rs\n+let n = 0; // MUTATION: nonce constant\n+ok\n-// MUTATION retirée\n'
    expect(mutationMarkers(diff)).toEqual(['let n = 0; // MUTATION: nonce constant'])
  })

  test('golden factory_profile', async () => {
    const sources = goldenSources('{"sources":[{"path":"a.rs"},{"path":"b.md"},{"x":1}]}')
    expect(sources).toEqual(['a.rs', 'b.md'])
    expect(goldenSources('pas du json')).toEqual([])
    expect(goldenErrors(['a.rs'], sources).length).toBe(1)
    expect(goldenErrors(['a.rs', 'crates/sbfb-workflow-core/src/factory_profile.golden.json'], sources)).toEqual([])
    expect(isGoldenSource('C:\\nexus\\a.rs', 'C:/nexus', sources)).toBe(true)
    expect(isGoldenSource('C:/nexus/z.rs', 'C:/nexus', sources)).toBe(false)
  })
})

describe('CI', () => {
  test('dernier run par workflow', async () => {
    const json = JSON.stringify([
      { workflowName: 'CI', status: 'completed', conclusion: 'failure' },
      { workflowName: 'CI', status: 'completed', conclusion: 'success' },
      { workflowName: 'Docs', status: 'completed', conclusion: 'success' },
    ])
    expect(ciSummary(json)).toEqual({ ci: 'rouge', failed: ['CI'] })
    expect(ciSummary(JSON.stringify([{ workflowName: 'CI', status: 'completed', conclusion: 'success' }])).ci).toBe('vert')
    expect(ciSummary(null).ci).toBe('inconnu')
    expect(ciSummary('oops').ci).toBe('inconnu')
  })
})

describe('affichage et garde', () => {
  test('barre de statut', async () => {
    expect(statusLine(SNAP)).toBe('S83·M1 │ rev ✗ │ codex ✗ │ r0 │ ±31 │ ↑40 │ CI ✗')
  })

  test('alertes et checklist', async () => {
    expect(alerts(SNAP)).toEqual(['CI rouge : CI, Rust CI', '40 commits non poussés'])
    expect(checklist(SNAP).find(c => c.label === 'review')?.isOk).toBe(false)
  })

  test('la garde ne bloque que sur un vrai résidu', async () => {
    expect(stopProblems(SNAP)).toEqual([])
    expect(stopProblems({ ...SNAP, mutationMarkers: ['x // MUTATION'] }).length).toBe(1)
  })

  test('détection d’une fin annoncée', async () => {
    expect(claimsDone("C'est terminé, tout est vert.")).toBe(true)
    expect(claimsDone('Phase terminée.')).toBe(true)
    expect(claimsDone('Les tests sont finis')).toBe(true)
    expect(claimsDone('Je définis la suite.')).toBe(false)
    expect(claimsDone('Prêt à commit.')).toBe(true)
    expect(claimsDone('Je continue la phase.')).toBe(false)
    expect(claimsDone(undefined)).toBe(false)
  })
})

describe('historique et sprint prévu', () => {
  test('phases du plan et sous-étapes', async () => {
    const md = [
      '## Phase 0 — Audit gate S82 : JOUÉE le 2026-07-17',
      '## Phase A — Cadrage (≈ 1 sem)',
      '**0a. Finir M1 puis suspendre S83** — détail',
      '## Phase M1 — Canonicalisation signée',
      '## Phase AC — Factory dogfood : verdict (premier golden LIVRÉ)',
      '## Pas une phase',
    ].join('\n')
    const phases = planPhases(md)
    expect(phases.map(p => p.id)).toEqual(['0', 'A', 'M1', 'AC'])
    expect(phases[0]!.isMarkedDone).toBe(true)
    expect(phases[3]!.isMarkedDone).toBe(false)
    expect(phases[1]!.title).toBe('Cadrage')
    expect(phases[1]!.children).toEqual([{ id: '0a', title: 'Finir M1 puis suspendre S83' }])
  })

  test('statut par commit de phase, chore exclu', async () => {
    const log = [
      'aaaaaaaa1|2026-07-29|feat(x): Sprint 83 Phase M — artefacts',
      'bbbbbbbb2|2026-07-22|chore(planning): Sprint 83 Phase H — round Codex r7',
      'cccccccc3|2026-07-22|feat(y): Sprint 83 Phase H — compilateur',
      'dddddddd4|2026-07-01|feat(z): Sprint 82 Phase H — autre sprint',
    ].join('\n')
    const commits = phaseCommits(log, 83)
    expect(commits.map(c => c.phase)).toEqual(['M', 'H'])
    const rows = phaseRows(
      [
        { id: 'H', title: 'h', isMarkedDone: false, children: [] },
        { id: 'M1', title: 'm1', isMarkedDone: false, children: [] },
        { id: 'N', title: 'n', isMarkedDone: false, children: [] },
      ],
      commits,
      'm1',
    )
    expect(rows.map(r => r.status)).toEqual(['fait', 'en-cours', 'prévu'])
    expect(rows[0]!.commit).toBe('cccccccc')
  })

  test('partie du plan propre au sprint visé', async () => {
    const md = '# Partie 1 — Mods\n## Phase X — pas ici\n# Partie 2 — S84 « Messagerie »\n## Phase 0 — Clôture\n# Annexe\n## Phase Z — non'
    expect(planPhases(sprintPart(md, 84)).map(p => p.id)).toEqual(['0'])
    expect(planPhases(sprintPart('## Phase A — a', 84)).map(p => p.id)).toEqual(['A'])
  })

  test('SPRINT_LOG groupé par version', async () => {
    const md = '## v2.2 — Workflow (OPEN)\n| Sprint | Etat |\n|---|---|\n| 83 | OPEN — kickoff `550a913` | — |\n## v2.1 — x\n| 82 | **DONE** + audit | t |'
    expect(sprintLog(md)).toEqual([
      { version: 'v2.2', sprint: 83, state: 'OPEN — kickoff 550a913' },
      { version: 'v2.1', sprint: 82, state: 'DONE + audit' },
    ])
  })
})
