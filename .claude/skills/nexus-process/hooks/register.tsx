import { atom, read, update } from 'claude-code'
import type { EngineInterface, ProcessRunResult, Register } from 'claude-code'

import type { NexusHistory, NexusSnapshot, PhaseRow } from '../types'
import {
  PROFILE_GOLDEN,
  phaseCommits,
  phaseRows,
  planPhases,
  sprintLog,
  sprintPart,
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
  porcelainPaths,
  preflightVerdict,
  residueFiles,
  reviewVerdict,
  statusLine,
  stopProblems,
  untrackedCode,
} from './lib'

type $ = EngineInterface

const PANE = 'nexus-phase'
const PANE_TITLE = 'nexus — phase'
const snapshot = atom({ plugin: 'nexus-process', key: 'snapshot' } as const, null)
const lightcheck = atom({ plugin: 'nexus-process', key: 'lightcheck' } as const, null)
const history = atom({ plugin: 'nexus-process', key: 'history' } as const, null)

const CI_TTL_MS = 10 * 60 * 1000
const REFRESH_MS = 60 * 1000
const DEBOUNCE_MS = 2000

// État du module : repart de zéro à chaque rechargement, ce qui est voulu
// (la racine est redétectée, le cache CI vidé).
const ctx: {
  configuredRoot: string
  nextPlan: string
  root: string | null | undefined
  ci: { at: number; json: string | null }
  inFlight: Promise<NexusSnapshot | null> | null
  pending: { cancel: () => void } | null
} = { configuredRoot: '', nextPlan: '', root: undefined, ci: { at: 0, json: null }, inFlight: null, pending: null }

async function readOrNull($: $, path: string): Promise<string | null> {
  try {
    return await $.fs.read(path)
  } catch {
    return null
  }
}

async function resolveRoot($: $): Promise<string | null> {
  if (ctx.root !== undefined) return ctx.root
  const candidates: string[] = []
  if (ctx.configuredRoot) candidates.push(ctx.configuredRoot)
  try {
    const top = await $.process.run(['git', 'rev-parse', '--show-toplevel'], { cwd: await $.session.cwd(), timeoutMs: 5000 })
    if (top.exitCode === 0) candidates.push(top.stdout.trim())
  } catch {
    // pas de git : seule la config compte
  }
  ctx.root = null
  for (const c of candidates) {
    const base = c.replace(/\\/g, '/').replace(/\/$/, '')
    if (await $.fs.exists(`${base}/scripts/agent/agentctl.py`)) {
      ctx.root = base
      break
    }
  }
  return ctx.root
}

async function git($: $, root: string, args: readonly string[]): Promise<string> {
  try {
    const out = await $.process.run(['git', ...args], { cwd: root, timeoutMs: 20_000 })
    return out.exitCode === 0 ? out.stdout : ''
  } catch {
    return ''
  }
}

async function ciJson($: $, root: string): Promise<string | null> {
  const now = await $.clock.now()
  if (now - ctx.ci.at < CI_TTL_MS) return ctx.ci.json
  try {
    const out = await $.process.run(
      ['gh', 'run', 'list', '--branch', 'master', '--limit', '12', '--json', 'name,workflowName,status,conclusion'],
      { cwd: root, timeoutMs: 15_000 },
    )
    ctx.ci = { at: now, json: out.exitCode === 0 ? out.stdout : null }
  } catch {
    ctx.ci = { at: now, json: null }
  }
  return ctx.ci.json
}

async function collect($: $): Promise<NexusSnapshot | null> {
  const root = await resolveRoot($)
  if (root === null) return null
  const active = `${root}/.planning/active`
  const entries = await $.fs.list(active).catch(() => [])
  const sprint = maxSprint(entries.map(e => e.name))
  const phase = sprint === null ? null : currentPhase(entries, sprint)
  const prefix = sprint !== null && phase !== null ? `${active}/sprint${sprint}_phase_${phase}` : null
  const [preflight, review, codex, roundNames] = await Promise.all([
    prefix ? readOrNull($, `${prefix}_preflight.md`) : null,
    prefix ? readOrNull($, `${prefix}_review.md`) : null,
    prefix ? readOrNull($, `${prefix}_codex_review.md`) : null,
    $.fs.list(`${active}/rounds`).then(l => l.map(e => e.name)).catch(() => [] as string[]),
  ])

  const [status, codeStatus, staged, added, worktreeDiff, golden, aheadOut, ciRaw] = await Promise.all([
    git($, root, ['status', '--porcelain=v1']),
    git($, root, ['status', '--porcelain=v1', '--untracked-files=all', '--', 'crates', 'web/src']),
    git($, root, ['diff', '--cached', '--name-only']),
    git($, root, ['diff', '--cached', '--name-only', '--diff-filter=A']),
    git($, root, ['diff', 'HEAD', '-U0', '--', 'crates', 'web/src']),
    readOrNull($, `${root}/${PROFILE_GOLDEN}`),
    git($, root, ['rev-list', '--count', '@{upstream}..HEAD']),
    ciJson($, root),
  ])

  const untracked = untrackedCode(codeStatus)
  const ciInfo = ciSummary(ciRaw)
  const aheadNum = Number.parseInt(aheadOut.trim(), 10)

  return {
    root,
    sprint,
    phase,
    preflightVerdict: preflightVerdict(preflight),
    reviewVerdict: reviewVerdict(review),
    isReviewPass: isReviewPass(review),
    codex: codexStatus(codex),
    rounds: sprint !== null && phase !== null ? maxRound(roundNames, sprint, phase) : 0,
    dirty: porcelainPaths(status).length,
    untrackedCode: untracked,
    residueFiles: residueFiles([...added.split('\n').filter(Boolean), ...untracked]),
    mutationMarkers: mutationMarkers(worktreeDiff),
    goldenErrors: goldenErrors(staged.split('\n').filter(Boolean), goldenSources(golden)),
    ahead: Number.isNaN(aheadNum) ? null : aheadNum,
    ci: ciInfo.ci,
    ciFailed: ciInfo.failed,
    updatedAt: await $.clock.now(),
  }
}

async function expandHome($: $, path: string): Promise<string> {
  if (!path.startsWith('~')) return path
  const home = (await $.env.get('USERPROFILE')) ?? (await $.env.get('HOME')) ?? ''
  return `${home.replace(/\\/g, '/')}${path.slice(1)}`
}

/** Plan du sprint suivant : `.planning/sprint{N}_plan.md` (précédent S78), `active/`, puis l'option nextSprintPlan. */
async function nextSprintPlan($: $, root: string, sprint: number): Promise<{ source: string; text: string } | null> {
  const candidates = [`${root}/.planning/sprint${sprint}_plan.md`, `${root}/.planning/active/sprint${sprint}_plan.md`]
  if (ctx.nextPlan) candidates.push(await expandHome($, ctx.nextPlan))
  for (const path of candidates) {
    const text = await readOrNull($, path)
    if (text !== null && planPhases(sprintPart(text, sprint)).length > 0) return { source: path, text: sprintPart(text, sprint) }
  }
  return null
}

async function collectHistory($: $, snap: NexusSnapshot): Promise<NexusHistory> {
  const root = snap.root
  const sprint = snap.sprint
  const [log, plan, sprintLogMd] = await Promise.all([
    sprint === null ? '' : git($, root, ['log', '--format=%h|%ad|%s', '--date=short', `--grep=Sprint ${sprint} Phase`]),
    sprint === null ? null : readOrNull($, `${root}/.planning/active/sprint${sprint}_plan.md`),
    readOrNull($, `${root}/docs/claude/SPRINT_LOG.md`),
  ])
  const phases: PhaseRow[] = sprint === null || plan === null ? [] : phaseRows(planPhases(plan), phaseCommits(log, sprint), snap.phase)
  const nextPlan = sprint === null ? null : await nextSprintPlan($, root, sprint + 1)
  return {
    sprint,
    phases,
    next:
      sprint === null || nextPlan === null
        ? null
        : { sprint: sprint + 1, source: nextPlan.source, phases: phaseRows(planPhases(nextPlan.text), [], null) },
    sprints: sprintLog(sprintLogMd ?? ''),
  }
}

async function doRefresh($: $): Promise<NexusSnapshot | null> {
  const snap = await collect($)
  await update($, snapshot, () => snap)
  if (snap !== null) {
    const hist = await collectHistory($, snap)
    await update($, history, () => hist)
  }
  $.ui.status(snap === null ? undefined : statusLine(snap))
  return snap
}

/** Un seul rafraîchissement à la fois ; un appel concurrent attend celui en cours. */
async function refresh($: $): Promise<NexusSnapshot | null> {
  if (ctx.inFlight === null) {
    ctx.inFlight = doRefresh($).finally(() => {
      ctx.inFlight = null
    })
  }
  return ctx.inFlight
}

function refreshSoon($: $): void {
  ctx.pending?.cancel()
  ctx.pending = $.clock.after(DEBOUNCE_MS, () => {
    ctx.pending = null
    void refresh($)
  })
}

async function agentctl($: $, root: string, args: readonly string[], timeoutMs: number): Promise<ProcessRunResult> {
  const script = `${root}/scripts/agent/agentctl.py`
  for (const head of [['python'], ['py', '-3'], ['python3']]) {
    try {
      return await $.process.run([...head, script, ...args], { cwd: root, timeoutMs })
    } catch {
      // interpréteur absent : on essaie le suivant
    }
  }
  return {
    exitCode: 127,
    stdout: '',
    stderr: 'aucun interpréteur Python trouvé (python, py -3, python3)',
    isStdoutTruncated: false,
    isStderrTruncated: false,
  }
}

async function runLightcheck($: $): Promise<string> {
  const root = await resolveRoot($)
  if (root === null) return 'Pas dans le repo nexus.'
  const out = await agentctl($, root, ['precommit-lightcheck', '--scope', 'staged'], 120_000)
  const lines = `${out.stdout}\n${out.stderr}`.split('\n').filter(l => /BLOCK|WARN|OK|PASS|error/i.test(l))
  const text = `lightcheck (staged) → exit ${out.exitCode}\n${lines.slice(0, 30).join('\n') || '(aucune ligne BLOCK/WARN)'}`
  await update($, lightcheck, () => text)
  return text
}

async function codexPromptPath($: $): Promise<string> {
  const root = await resolveRoot($)
  const snap = (await read($, snapshot)) ?? (await refresh($))
  if (root === null) return 'Pas dans le repo nexus.'
  if (snap === null || snap.sprint === null || snap.phase === null) return 'Sprint ou phase introuvable.'
  const out = await agentctl(
    $,
    root,
    ['codex-prompt-path', '--sprint', String(snap.sprint), '--phase', snap.phase, '--absolute'],
    20_000,
  )
  const path = out.stdout.trim()
  if (out.exitCode !== 0 || !path) return `codex-prompt-path a échoué : ${out.stderr.trim()}`
  const copied = await $.ui.copy({ text: path })
  return `${path}${copied.isCopied ? ' (copié)' : ''}`
}

function sprintText(snap: NexusSnapshot): string {
  return [
    statusLine(snap),
    ...checklist(snap).map(c => `${c.isOk ? '✓' : '✗'} ${c.label} : ${c.detail}`),
    `fichiers sales : ${snap.dirty}${snap.untrackedCode.length ? ` (dont ${snap.untrackedCode.length} ?? sous crates/ ou web/src)` : ''}`,
    ...alerts(snap).map(a => `⚠ ${a}`),
  ].join('\n')
}

async function openPane($: $): Promise<void> {
  await $.ui.open({ id: PANE, title: PANE_TITLE })
}

export const register: Register = (on, options) => {
  ctx.configuredRoot = typeof options.nexusRoot === 'string' ? options.nexusRoot.trim() : ''
  ctx.nextPlan = typeof options.nextSprintPlan === 'string' ? options.nextSprintPlan.trim() : ''

  on('session.start', async ($, e, next) => {
    const started = await next(e)
    if ((await resolveRoot($)) === null) return started
    await $.command.register({ name: 'nexus', description: 'Ouvre le panneau de la phase nexus en cours' })
    await $.command.register({ name: 'nexus-sprint', description: "Résumé de l'état du process nexus" })
    await $.command.register({ name: 'nexus-lightcheck', description: 'Lance agentctl precommit-lightcheck --scope staged' })
    await $.command.register({ name: 'nexus-codex', description: 'Chemin du prompt Codex de la phase (copié)' })
    void refresh($)
    $.clock.every(REFRESH_MS, () => void refresh($))
    return started
  })

  on('command.run', { command: 'nexus' }, async $ => {
    await refresh($)
    await openPane($)
    return { text: 'Panneau nexus ouvert.' }
  })

  on('command.run', { command: 'nexus-sprint' }, async $ => {
    const snap = await refresh($)
    if (snap === null) return { text: "Pas dans le repo nexus (réglez l'option nexusRoot)." }
    const hist = await read($, history)
    const done = hist?.phases.filter(p => p.status === 'fait').length ?? 0
    const next = hist?.next ? `S${hist.next.sprint} prévu : ${hist.next.phases.map(p => p.id).join(', ')}` : 'prochain sprint : aucun plan'
    return {
      text: `${sprintText(snap)}\nphases S${snap.sprint} : ${done}/${hist?.phases.length ?? 0} faites · ${next} · historique : ${hist?.sprints.length ?? 0} sprints`,
    }
  })

  on('command.run', { command: 'nexus-lightcheck' }, async $ => ({ text: await runLightcheck($) }))

  on('command.run', { command: 'nexus-codex' }, async $ => ({ text: await codexPromptPath($) }))

  on('tool.call', async ($, e, next) => {
    const ran = await next(e)
    const root = ctx.root
    if (root === null || root === undefined) return ran
    if (e.tool === 'Edit' || e.tool === 'Write') {
      const sources = goldenSources(await readOrNull($, `${root}/${PROFILE_GOLDEN}`))
      if (isGoldenSource(e.file_path, root, sources)) {
        $.ui.toast('nexus : source du golden factory_profile modifiée — rafraîchir le golden dans ce commit (check 11)')
      }
    }
    if (e.tool === 'Edit' || e.tool === 'Write' || e.tool === 'Bash') refreshSoon($)
    return ran
  }).catch(($, e, next) => next(e))

  on('classic.Stop', async ($, e, next) => {
    const result = await next(e)
    if (result.block !== undefined || e.stop_hook_active || (await resolveRoot($)) === null) return result
    const snap = await refresh($)
    if (snap === null) return result
    const problems = stopProblems(snap)
    if (problems.length === 0) return result
    if (claimsDone(e.last_assistant_message)) {
      return {
        ...result,
        block:
          `[nexus-process] Fin annoncée mais l'arbre n'est pas propre :\n- ${problems.join('\n- ')}\n` +
          'Corrige (revert de la mutation, suppression de la sonde, golden rafraîchi) avant de conclure.',
      }
    }
    $.ui.toast(`nexus : ${problems[0]}`)
    return result
  }).catch(($, e, next) => next(e))

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const snap = await read($, snapshot)
    if (snap === null || e.props.hasSurvey) return next(e)
    const list = alerts(snap)
    if (list.length === 0) return next(e)
    const { Box, Button, Text } = $.ui.resolve(e)
    return (
      <Box flexDirection="column">
        {list.slice(0, 4).map(a => (
          <Text color="yellow">⚠ nexus · {a}</Text>
        ))}
        <Button key="open" label="Panneau" onPress={() => openPane($)} />
      </Box>
    )
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Button, Text } = $.ui.resolve(e)
    const snap = await read($, snapshot)
    const hist = await read($, history)
    const check = await read($, lightcheck)
    if (snap === null) {
      return <Text dimColor>Pas de repo nexus détecté (option nexusRoot).</Text>
    }
    const phase = (snap.phase ?? '').toUpperCase()
    const width = Math.max(30, e.props.bodyColumns - 2)
    const fit = (text: string, room: number) => (text.length > room ? `${text.slice(0, Math.max(1, room - 1))}…` : text)
    const mark = { fait: '✓', 'en-cours': '▶', prévu: '·' } as const
    const tone = { fait: 'green', 'en-cours': 'yellow', prévu: undefined } as const
    const done = hist?.phases.filter(p => p.status === 'fait').length ?? 0
    const phaseLine = (p: PhaseRow) => {
      const tail = p.commit ? `  ${p.date} ${p.commit}` : ''
      return fit(`${mark[p.status]} ${p.id.padEnd(3)} ${p.title}`, width - tail.length) + tail
    }
    let version = ''
    return (
      <Box flexDirection="column">
        <Text bold>État</Text>
        <Text>{statusLine(snap)}</Text>
        {checklist(snap).map(c => (
          <Text color={c.isOk ? 'green' : 'red'}>
            {c.isOk ? '✓' : '✗'} {c.label} <Text dimColor>{c.detail}</Text>
          </Text>
        ))}
        <Text>
          {'  '}sales {snap.dirty} · ?? code {snap.untrackedCode.length} · ↑{snap.ahead ?? '?'} · CI {snap.ci}
        </Text>
        {alerts(snap).map(a => (
          <Text color="yellow">⚠ {a}</Text>
        ))}
        <Box flexWrap="wrap">
          <Button key="lightcheck" label="lightcheck" hotkey="l" onPress={() => runLightcheck($)} />
          <Button key="codex" label="prompt Codex" hotkey="c" onPress={async () => $.ui.toast(await codexPromptPath($))} />
          <Button
            key="preflight"
            label="preflight"
            hotkey="p"
            onPress={() => $.prompt.fill({ text: `Lance le preflight G8 de S${snap.sprint} phase ${phase} selon le canon.` })}
          />
          <Button key="refresh" label="rafraîchir" hotkey="r" onPress={() => refresh($)} />
        </Box>
        {check !== null && <Text dimColor>{check}</Text>}

        <Text> </Text>
        <Text bold>
          Phases S{snap.sprint} — {done}/{hist?.phases.length ?? 0} faites
        </Text>
        {(hist?.phases.length ?? 0) === 0 && <Text dimColor>Plan du sprint introuvable.</Text>}
        {hist?.phases.map(p => (
          <Text color={tone[p.status]} dimColor={p.status === 'prévu'}>
            {phaseLine(p)}
          </Text>
        ))}

        <Text> </Text>
        <Text bold>Prochain sprint{hist?.next ? ` S${hist.next.sprint}` : ''} — prévu</Text>
        {hist?.next == null ? (
          <Text dimColor>Aucun plan trouvé (.planning/sprint{(snap.sprint ?? 0) + 1}_plan.md ou option nextSprintPlan).</Text>
        ) : (
          <Box flexDirection="column">
            <Text dimColor>{fit(hist.next.source, width)}</Text>
            {hist.next.phases.map(p => (
              <Box flexDirection="column">
                <Text>{fit(`· Phase ${p.id} — ${p.title}`, width)}</Text>
                {p.children.map(c => (
                  <Text dimColor>{fit(`    ${c.id}. ${c.title}`, width)}</Text>
                ))}
              </Box>
            ))}
          </Box>
        )}

        <Text> </Text>
        <Text bold>Historique — {hist?.sprints.length ?? 0} sprints</Text>
        {hist?.sprints.map(s => {
          const header = s.version !== version
          version = s.version
          return (
            <Box flexDirection="column">
              {header && <Text color="cyan">{s.version}</Text>}
              <Text dimColor={!/OPEN/.test(s.state)}>{fit(`  S${String(s.sprint).padEnd(3)} ${s.state}`, width)}</Text>
            </Box>
          )
        })}
      </Box>
    )
  })
}
