// Fonctions pures : aucune I/O, testables sous `claude plugin test`.
// Les motifs reprennent scripts/agent/agentctl.py et le Check 0bis de
// .claude/hooks/phase-precommit-lightcheck.sh du repo nexus.

import type { CiStatus, CodexStatus, NexusSnapshot } from '../types'

export type Entry = { name: string; mtimeMs: number }

// agentctl.py:35 FINAL_PASS_RE
const FINAL_PASS = /^## Verdict\s*:\s*PASS\s*$/m
const PREFLIGHT_VERDICT = /\b(EXECUTE|PLAN-ADAPT|SCOPE-CUT-CONSISTENT|DESIGN-CONFLICT)\b/
// Check 0bis : fichiers-artefacts et marqueurs de mutation
const RESIDUE_FILE = /(^|\/)(zz_)|probe|_scratch|\.bak$/i
const MUTATION_MARKER = /MUTATION[-: ]|MUTATION$|TEMPORARY review probe/
const CODE_ROOT = /^(crates|web\/src)\//

export function maxSprint(names: readonly string[]): number | null {
  let best: number | null = null
  for (const name of names) {
    const m = /^sprint(\d+)_/.exec(name)
    if (m) best = Math.max(best ?? 0, Number(m[1]))
  }
  return best
}

/** Phase dont le preflight est le plus récent (agentctl ne détecte jamais la phase). */
export function currentPhase(entries: readonly Entry[], sprint: number): string | null {
  const re = new RegExp(`^sprint${sprint}_phase_([a-z0-9]+)_preflight\\.md$`, 'i')
  let best: { phase: string; mtimeMs: number } | null = null
  for (const e of entries) {
    const m = re.exec(e.name)
    if (m && (best === null || e.mtimeMs > best.mtimeMs)) best = { phase: m[1]!, mtimeMs: e.mtimeMs }
  }
  return best?.phase ?? null
}

export function preflightVerdict(text: string | null): string | null {
  if (text === null) return null
  return PREFLIGHT_VERDICT.exec(text)?.[1] ?? 'présent'
}

/** Miroir de review_gate_errors (agentctl.py:711) : PASS exact, pas PASS-PENDING, Codex non en attente. */
export function isReviewPass(text: string | null): boolean {
  if (text === null || !FINAL_PASS.test(text)) return false
  const verdict = text.split('\n').find(l => l.startsWith('## Verdict')) ?? ''
  if (verdict.includes('PASS-PENDING')) return false
  return !/Codex.*EN ATTENTE|EN ATTENTE.*Codex|Ready for Codex verification/i.test(text)
}

export function reviewVerdict(text: string | null): string | null {
  if (text === null) return null
  const line = text.split('\n').find(l => l.startsWith('## Verdict'))
  if (line === undefined) return 'sans verdict'
  return line.replace(/^## Verdict\s*:?\s*/, '').trim() || 'sans verdict'
}

/** Marqueurs de codex_review_errors (agentctl.py:570). */
export function codexStatus(text: string | null): CodexStatus {
  if (text === null || !text.trim()) return 'absent'
  if (/Statut\s*:\s*GAP|Gaps?\s*:\s*[1-9]/i.test(text)) return 'gap'
  if (/Statut\s*:\s*PARTIEL|Partiels?\s*:\s*[1-9]/i.test(text)) return 'partiel'
  // `\b` de JS ignore les lettres accentuées (Python les compte) : limites Unicode explicites.
  if (/(?<![\p{L}\d_])(CONFIRME|CONFIRM[EÉ]|GAP|PARTIEL|PARTIAL|CONFIRMED)(?![\p{L}\d_])|\*\*(PASS|FAIL)/iu.test(text)) return 'ok'
  return 'sans-verdict'
}

export function maxRound(names: readonly string[], sprint: number, phase: string): number {
  const re = new RegExp(`^sprint${sprint}_phase_${phase}_.*_r(\\d+)\\.md$`, 'i')
  let best = 0
  for (const name of names) {
    const m = re.exec(name)
    if (m) best = Math.max(best, Number(m[1]))
  }
  return best
}

export function porcelainPaths(porcelain: string): { code: string; path: string }[] {
  return porcelain
    .split('\n')
    .filter(l => l.length > 3)
    .map(l => ({ code: l.slice(0, 2), path: l.slice(3).replace(/^"|"$/g, '') }))
}

export function untrackedCode(porcelain: string): string[] {
  return porcelainPaths(porcelain)
    .filter(p => p.code === '??' && CODE_ROOT.test(p.path))
    .map(p => p.path)
}

export function residueFiles(paths: readonly string[]): string[] {
  return paths.filter(p => RESIDUE_FILE.test(p))
}

export function mutationMarkers(diff: string): string[] {
  return diff
    .split('\n')
    .filter(l => l.startsWith('+') && !l.startsWith('+++') && MUTATION_MARKER.test(l))
    .map(l => l.slice(1).trim())
}

export function goldenSources(goldenJson: string | null): string[] {
  if (goldenJson === null) return []
  try {
    const golden = JSON.parse(goldenJson) as { sources?: unknown }
    if (!Array.isArray(golden.sources)) return []
    return golden.sources.flatMap(s =>
      s !== null && typeof s === 'object' && typeof (s as { path?: unknown }).path === 'string'
        ? [(s as { path: string }).path]
        : [],
    )
  } catch {
    return []
  }
}

export const PROFILE_GOLDEN = 'crates/sbfb-workflow-core/src/factory_profile.golden.json'

/** Miroir de profile_golden_errors (agentctl.py:634) / Check 11. */
export function goldenErrors(staged: readonly string[], sources: readonly string[]): string[] {
  if (staged.includes(PROFILE_GOLDEN)) return []
  const touched = sources.filter(s => staged.includes(s))
  return touched.length === 0
    ? []
    : [`source(s) du golden factory_profile stagée(s) sans le golden : ${touched.join(', ')}`]
}

export function isGoldenSource(path: string, root: string, sources: readonly string[]): boolean {
  const norm = path.replace(/\\/g, '/')
  const base = root.replace(/\\/g, '/').replace(/\/$/, '')
  const rel = norm.toLowerCase().startsWith(base.toLowerCase() + '/') ? norm.slice(base.length + 1) : norm
  return sources.includes(rel)
}

type GhRun = { name?: string; workflowName?: string; status?: string; conclusion?: string }

/** Dernier run par workflow (gh run list --json name,workflowName,status,conclusion). */
export function ciSummary(json: string | null): { ci: CiStatus; failed: string[] } {
  if (json === null) return { ci: 'inconnu', failed: [] }
  let runs: GhRun[]
  try {
    runs = JSON.parse(json) as GhRun[]
  } catch {
    return { ci: 'inconnu', failed: [] }
  }
  if (!Array.isArray(runs) || runs.length === 0) return { ci: 'inconnu', failed: [] }
  const latest = new Map<string, GhRun>()
  for (const run of runs) {
    const key = run.workflowName ?? run.name ?? '?'
    if (!latest.has(key)) latest.set(key, run)
  }
  const failed = [...latest]
    .filter(([, r]) => r.conclusion === 'failure' || r.conclusion === 'startup_failure' || r.conclusion === 'timed_out')
    .map(([k]) => k)
  if (failed.length > 0) return { ci: 'rouge', failed }
  if ([...latest.values()].some(r => r.status !== 'completed')) return { ci: 'en-cours', failed }
  return { ci: 'vert', failed }
}

/** La réponse finale annonce-t-elle une fin de travail ? */
export function claimsDone(text: string | undefined): boolean {
  if (text === undefined) return false
  return /(?<![\p{L}\d_])(termin[ée]e?s?|finie?s?|achev[ée]e?s?|prête? à (commit|être commit)\w*|ready to commit|tout est (vert|bon|ok)|all (green|done)|done|## Verdict\s*:?\s*PASS)(?![\p{L}\d_])/iu.test(
    text,
  )
}

const MARK = { ok: '✓', ko: '✗' }

function codexMark(c: CodexStatus): string {
  return c === 'ok' ? MARK.ok : c === 'absent' ? MARK.ko : c
}

function ciMark(c: CiStatus): string {
  return c === 'vert' ? MARK.ok : c === 'rouge' ? MARK.ko : c === 'en-cours' ? '…' : '?'
}

export function statusLine(s: NexusSnapshot): string {
  const head = `S${s.sprint ?? '?'}·${(s.phase ?? '?').toUpperCase()}`
  return [
    head,
    `rev ${s.isReviewPass ? MARK.ok : MARK.ko}`,
    `codex ${codexMark(s.codex)}`,
    `r${s.rounds}`,
    `±${s.dirty}`,
    `↑${s.ahead ?? '?'}`,
    `CI ${ciMark(s.ci)}`,
  ].join(' │ ')
}

export function alerts(s: NexusSnapshot): string[] {
  const out: string[] = []
  if (s.mutationMarkers.length > 0) out.push(`marqueur de mutation dans crates/ ou web/src (${s.mutationMarkers.length})`)
  if (s.residueFiles.length > 0) out.push(`artefact de review : ${s.residueFiles.slice(0, 3).join(', ')}`)
  out.push(...s.goldenErrors)
  if (s.ci === 'rouge') out.push(`CI rouge : ${s.ciFailed.slice(0, 4).join(', ')}`)
  if ((s.ahead ?? 0) >= 10) out.push(`${s.ahead} commits non poussés`)
  return out
}

export type Check = { label: string; isOk: boolean; detail: string }

export function checklist(s: NexusSnapshot): Check[] {
  return [
    { label: 'preflight G8', isOk: s.preflightVerdict !== null && s.preflightVerdict !== 'DESIGN-CONFLICT', detail: s.preflightVerdict ?? 'absent' },
    { label: 'review', isOk: s.isReviewPass, detail: s.reviewVerdict ?? 'absent' },
    { label: 'Codex brut', isOk: s.codex === 'ok', detail: s.codex },
    { label: 'golden factory_profile', isOk: s.goldenErrors.length === 0, detail: s.goldenErrors.length === 0 ? 'cohérent' : 'à rafraîchir' },
    { label: 'résidus de review', isOk: s.mutationMarkers.length + s.residueFiles.length === 0, detail: `${s.mutationMarkers.length + s.residueFiles.length}` },
    { label: 'rounds de review', isOk: s.rounds <= 3, detail: `${s.rounds}` },
  ]
}

/** Problèmes qui justifient de bloquer une fin de tour annoncée comme terminée. */
export function stopProblems(s: NexusSnapshot): string[] {
  const out: string[] = []
  if (s.mutationMarkers.length > 0) out.push(`mutation(s) de review laissée(s) dans le code : ${s.mutationMarkers.slice(0, 3).join(' | ')}`)
  if (s.residueFiles.length > 0) out.push(`fichier(s) sonde/scratch : ${s.residueFiles.slice(0, 5).join(', ')}`)
  out.push(...s.goldenErrors)
  return out
}

// ---- Historique et sprints prévus ----

export type PlanPhase = { id: string; title: string; isMarkedDone: boolean; children: { id: string; title: string }[] }

function cleanTitle(title: string): string {
  return title
    .replace(/\*\*|`/g, '')
    .replace(/\s*\(≈[^)]*\)\s*$/, '')
    .replace(/\s*[:.]\s*$/, '')
    .trim()
}

/** Garde la partie d'un plan multi-livrables qui parle du sprint visé (`# Partie … S84 …`). */
export function sprintPart(md: string, sprint: number): string {
  const lines = md.split('\n')
  const start = lines.findIndex(l => /^# /.test(l) && new RegExp(`\\bS${sprint}\\b`).test(l))
  if (start < 0) return md
  const end = lines.findIndex((l, i) => i > start && /^# /.test(l))
  return lines.slice(start, end < 0 ? undefined : end).join('\n')
}

/** `## Phase X — titre` (+ sous-étapes `**0a. titre**` sous la phase). */
export function planPhases(md: string): PlanPhase[] {
  const out: PlanPhase[] = []
  for (const line of md.split('\n')) {
    const phase = /^## Phase ([A-Z0-9]+)\s*[—–-]\s*(.+)$/.exec(line)
    if (phase) {
      out.push({
        id: phase[1]!,
        title: cleanTitle(phase[2]!),
        // marqueur explicite après deux-points (« : JOUÉE le … ») ; un « LIVRÉ » dans le titre ne compte pas
        isMarkedDone: /:\s*(JOU[ÉE]E|DONE|LIVR[ÉE]E?)(?![\p{L}])/iu.test(phase[2]!),
        children: [],
      })
      continue
    }
    const step = /^\*\*(\d+[a-z])\.\s+(.+?)\*\*/.exec(line)
    if (step && out.length > 0) out[out.length - 1]!.children.push({ id: step[1]!, title: cleanTitle(step[2]!) })
  }
  return out
}

export type PhaseCommit = { sha: string; date: string; phase: string }

/** Lignes `sha|date|sujet` de git log ; seuls les commits de phase comptent (pas chore). */
export function phaseCommits(log: string, sprint: number): PhaseCommit[] {
  const re = new RegExp(`^(feat|fix|docs|test|refactor)\\([^)]*\\):\\s*Sprint\\s+${sprint}\\s+Phase\\s+([A-Z]+[0-9]?)\\b`)
  const out: PhaseCommit[] = []
  for (const line of log.split('\n')) {
    const [sha, date, ...rest] = line.split('|')
    const m = re.exec(rest.join('|'))
    if (sha && date && m) out.push({ sha, date, phase: m[2]! })
  }
  return out
}

export function phaseRows(
  phases: readonly PlanPhase[],
  commits: readonly PhaseCommit[],
  current: string | null,
): { id: string; title: string; status: 'fait' | 'en-cours' | 'prévu'; commit: string | null; date: string | null; children: { id: string; title: string }[] }[] {
  return phases.map(p => {
    // git log est du plus récent au plus ancien : le dernier trouvé est le premier commit de la phase
    const hits = commits.filter(c => c.phase.toLowerCase() === p.id.toLowerCase())
    const first = hits[hits.length - 1]
    const isCurrent = current !== null && current.toLowerCase() === p.id.toLowerCase()
    const status = first !== undefined || p.isMarkedDone ? 'fait' : isCurrent ? 'en-cours' : 'prévu'
    return { id: p.id, title: p.title, status, commit: first?.sha.slice(0, 8) ?? null, date: first?.date ?? null, children: p.children }
  })
}

/** Tableaux `| N | État | …` de docs/claude/SPRINT_LOG.md, groupés par `## vX — …`. */
export function sprintLog(md: string): { version: string; sprint: number; state: string }[] {
  const out: { version: string; sprint: number; state: string }[] = []
  let version = ''
  for (const line of md.split('\n')) {
    const head = /^## (v[\d.]+)\b/.exec(line)
    if (head) {
      version = head[1]!
      continue
    }
    const row = /^\|\s*(\d+)\s*\|\s*([^|]*)\|/.exec(line)
    if (row && version) {
      const state = row[2]!.replace(/\*\*|`/g, '').replace(/\s+/g, ' ').trim()
      out.push({ version, sprint: Number(row[1]), state: state.length > 110 ? `${state.slice(0, 109)}…` : state })
    }
  }
  return out
}
