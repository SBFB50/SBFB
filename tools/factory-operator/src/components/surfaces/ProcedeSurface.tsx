// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D → front rapid-add — the arbre de procédé (folds A1/U1, the
// SBFB signature: process-as-artifact). It restitutes the gisement ALREADY
// computed by Rust (sprint_history.rs): sprint → phase → commit → artefact,
// with each phase's RESTITUTED preflight/review/Codex verdicts. The cardinal
// rule holds end to end: every verdict is read off the on-disk artifacts (the
// UI computes nothing, scores nothing). Each verdict carries its SOURCE
// artifact filename (fold U2). The frise (fold V8) is the condensed verdict
// strip; clicking a phase commit opens its diff (fold J11) via the bespoke
// `DiffViewer` (the SAME viewer that renders the working tree in VERIFY, fold
// V2/U7) and a conformity card (folds U3/A9/V10).
//
// Rapid-add enrichment: a LIVE "où on en est" banner (current phase from
// /api/status, restituted before its commit exists), per-phase files-changed,
// the full sprint commit timeline, the carries register, the §1 verification
// table, the entry→exit test bilan, a local phase filter + multi-expand, and a
// glyph legend. Everything is RESTITUTED from the two backends; nothing is a
// fabricated verdict.
import { useEffect, useMemo, useState } from 'react'
import {
  getAllSprints,
  getSprintHistory,
  getStatus,
  OperatorError,
  type CarryItem,
  type CommitInfo,
  type OperatorStatus,
  type SprintHistory,
  type SprintSummary,
  type VerificationSummary,
} from '../../api/operator'
import { preflightTone, reviewTone, toneBg, toneText } from '../../lib/verdict'
import { DiffViewer } from '../verify/plein/DiffViewer'
import { ConformiteCard } from './ConformiteCard'
import { useCommitDiff } from '../../state/useCommitDiff'
import { AdaptiveSurface } from '../AdaptiveSurface'

function VerdictPill({ verdict, tone }: { verdict: string | null; tone: string }) {
  return (
    <span className={`font-mono text-meta ${tone}`} data-testid="verdict-pill">
      {verdict ?? '—'}
    </span>
  )
}

type FileChange = SprintHistory['phases'][number]['files_changed'][number]

function FilesChanged({ files }: { files: FileChange[] }) {
  if (files.length === 0) return null
  return (
    <div data-testid="phase-files">
      <div className="mb-0.5 eyebrow">
        fichiers <span className="text-tx4">· {files.length}</span>
      </div>
      <ul className="flex flex-col gap-0.5">
        {files.map((f, i) => (
          <li key={`${f.path}-${i}`} className="flex items-baseline gap-2 font-mono text-meta">
            <span className="w-3 shrink-0 text-tx4" aria-hidden>
              {f.status}
            </span>
            <span className="min-w-0 flex-1 truncate text-tx3">{f.path}</span>
            <span className="shrink-0 text-ok">+{f.insertions}</span>
            <span className="shrink-0 text-bad">−{f.deletions}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}

function PhaseNode({
  phase,
  preflightFile,
  expanded,
  onToggle,
}: {
  phase: SprintHistory['phases'][number]
  preflightFile: string | null
  expanded: boolean
  onToggle: () => void
}) {
  return (
    <div className="border-l-2 border-bd pl-3">
      <button
        type="button"
        data-testid="procede-phase"
        aria-expanded={expanded}
        onClick={onToggle}
        className="flex w-full items-center gap-2.5 py-1.5 text-left hover:bg-s1"
      >
        <span className="font-mono text-meta font-semibold text-tx">{phase.letter}</span>
        <span className="min-w-0 flex-1 truncate font-sans text-sec text-tx2">{phase.title || '—'}</span>
        <VerdictPill verdict={phase.preflight_verdict} tone={toneText(preflightTone(phase.preflight_verdict))} />
        <span className="text-tx4" aria-hidden>
          ·
        </span>
        <VerdictPill verdict={phase.review_verdict} tone={toneText(reviewTone(phase.review_verdict))} />
        {phase.commit_sha ? <span className="font-mono text-meta text-tx4">{phase.commit_sha}</span> : null}
        <span className="font-mono text-meta text-tx4" aria-hidden>
          {expanded ? '▾' : '▸'}
        </span>
      </button>
      {expanded ? (
        <div className="mb-2 ml-1 flex flex-col gap-2 border-l border-bd pl-3">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-meta text-tx3">
            <span>
              préflight{' '}
              <span className={toneText(preflightTone(phase.preflight_verdict))}>
                {phase.preflight_verdict ?? '—'}
              </span>
            </span>
            {preflightFile ? (
              <span
                className="text-tx4"
                title="verdict restitué depuis cet artefact .planning/ (lecture du contenu en S81)"
              >
                ⤷ {preflightFile}
              </span>
            ) : null}
            <span>
              review <span className={toneText(reviewTone(phase.review_verdict))}>{phase.review_verdict ?? '—'}</span>
            </span>
            <span>
              codex {phase.codex_confirmed ?? 0}✓ {phase.codex_partial ?? 0}~ {phase.codex_gap ?? 0}⚠
            </span>
            <span>
              Δ +{phase.rust_delta} Rust · +{phase.vitest_delta} Vitest
            </span>
          </div>
          {phase.deliverables.length > 0 ? (
            <div data-testid="phase-deliverables">
              <div className="mb-0.5 eyebrow">livrables</div>
              <ul className="flex flex-col gap-0.5">
                {phase.deliverables.map((d, i) => (
                  <li key={i} className="font-mono text-meta text-tx3">
                    <span className="text-tx4" aria-hidden>
                      ·{' '}
                    </span>
                    {d}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {phase.findings.length > 0 ? (
            <div data-testid="phase-findings">
              <div className="mb-0.5 eyebrow">
                findings review <span className="text-tx4">· {phase.findings.length}</span>
              </div>
              <ul className="flex flex-col gap-0.5">
                {phase.findings.map((f, i) => (
                  <li key={i} className="font-mono text-meta text-tx3">
                    <span className={f.status === 'resolved' ? 'text-ok' : 'text-warn'}>{f.severity}</span> {f.description}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          <FilesChanged files={phase.files_changed} />
          {phase.commit_sha ? <PhaseDiff sha={phase.commit_sha} /> : null}
        </div>
      ) : null}
    </div>
  )
}

function PhaseDiff({ sha }: { sha: string }) {
  const { diff, error, loading } = useCommitDiff(sha)
  return (
    <div className="flex flex-col gap-2">
      <ConformiteCard rev={sha} />
      {loading ? (
        <div className="font-mono text-meta text-tx4">diff du commit…</div>
      ) : error ? (
        <div className="font-mono text-meta text-warn">{error}</div>
      ) : diff ? (
        // Fold V2/U7: the SAME bespoke viewer renders a PAST commit here and the
        // working tree in VERIFY — one component, two usages, on the shared
        // FileDiff[] shape (the hunks computed in Rust, never a JS re-diff).
        <DiffViewer
          files={diff.files}
          caption={`commit ${diff.sha.slice(0, 10)} — ${diff.title}`}
          emptyLabel="aucun fichier dans ce diff"
          testid="diff-view"
        />
      ) : null}
    </div>
  )
}

/** The LIVE "où on en est" banner: restitutes the current sprint/phase from
 * /api/status. The current phase usually has NO commit yet (it is not in the
 * committed `phases` list) — this is the one place that surfaces it. */
function LiveProcessBanner({
  history,
  status,
}: {
  history: SprintHistory
  status: OperatorStatus | null
}) {
  const current = status?.current_phase ?? null
  const committed = new Set(history.phases.map((p) => p.letter))
  const currentDone = current !== null && committed.has(current)
  const liveProgress = current !== null ? status?.phases.find((p) => p.letter === current) ?? null : null
  return (
    <div
      data-testid="live-process"
      className="mb-4 flex flex-wrap items-center gap-x-4 gap-y-1 rounded-md border border-bd bg-s1 px-3 py-2"
    >
      <span className="eyebrow">où on en est</span>
      <span className="flex items-center gap-1.5 font-mono text-meta">
        <span className={`h-1.5 w-1.5 rounded-full ${currentDone ? 'bg-ok' : 'bg-warn'}`} aria-hidden />
        <span className="text-tx2">phase courante</span>
        <span className="font-semibold text-tx">{current ?? '—'}</span>
        <span className="text-tx4">
          {current === null
            ? ''
            : currentDone
              ? '· committée'
              : liveProgress
                ? `· ${liveProgress.has_preflight ? 'préflight' : 'démarrage'}${liveProgress.has_review ? '→review' : ''}${liveProgress.has_codex ? '→codex' : ''} · pas encore committée`
                : '· en cours · pas encore committée'}
        </span>
      </span>
      <span className="ml-auto font-mono text-meta text-tx3">
        {history.phases.length} phases committées · {history.total_commits} commits
      </span>
    </div>
  )
}

function TestsBilan({ tests }: { tests: SprintHistory['tests'] }) {
  const hasEntryExit = tests.rust_exit > 0 || tests.vitest_exit > 0
  return (
    <div
      data-testid="tests-bilan"
      className="mb-4 flex flex-wrap items-center gap-x-4 gap-y-1 rounded-md border border-bd bg-s1 px-3 py-2 font-mono text-meta text-tx3"
    >
      <span className="eyebrow">bilan tests</span>
      {hasEntryExit ? (
        <>
          <span>
            Rust <span className="text-tx2">{tests.rust_entry}→{tests.rust_exit}</span>{' '}
            <span className={tests.rust_delta >= 0 ? 'text-ok' : 'text-bad'}>
              ({tests.rust_delta >= 0 ? '+' : ''}
              {tests.rust_delta})
            </span>
          </span>
          <span>
            Vitest <span className="text-tx2">{tests.vitest_entry}→{tests.vitest_exit}</span>{' '}
            <span className={tests.vitest_delta >= 0 ? 'text-ok' : 'text-bad'}>
              ({tests.vitest_delta >= 0 ? '+' : ''}
              {tests.vitest_delta})
            </span>
          </span>
        </>
      ) : (
        <span className="text-tx4">entrée→sortie au wrap-up (verification.md)</span>
      )}
      <span>
        size-limit <span className="text-tx2">{tests.size_limit}</span>
      </span>
      {tests.per_phase.length > 0 ? (
        <span className="text-tx4">· {tests.per_phase.length} phases mesurées</span>
      ) : null}
    </div>
  )
}

function CommitTimeline({ commits }: { commits: CommitInfo[] }) {
  const [open, setOpen] = useState(false)
  if (commits.length === 0) return null
  return (
    <div className="mt-4 border-t border-bd pt-3" data-testid="commit-timeline">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className="mb-1 flex items-center gap-2 eyebrow hover:text-tx2"
      >
        <span aria-hidden>{open ? '▾' : '▸'}</span>
        timeline des commits · {commits.length}
      </button>
      {open ? (
        <ul className="flex flex-col gap-0.5">
          {commits.map((c) => (
            <li
              key={c.sha}
              data-testid="commit-row"
              className="flex items-baseline gap-2 rounded-sm px-1 py-0.5 font-mono text-meta hover:bg-s1"
            >
              <span className="shrink-0 text-info">{c.short}</span>
              <span className={`shrink-0 ${c.is_phase ? 'text-tx2' : 'text-tx4'}`}>
                {c.commit_type}
                {c.phase ? `·${c.phase}` : ''}
              </span>
              <span className="min-w-0 flex-1 truncate text-tx3">{c.title}</span>
              <span className="shrink-0 text-ok">+{c.insertions}</span>
              <span className="shrink-0 text-bad">−{c.deletions}</span>
              <span className="shrink-0 text-tx4">{c.date.slice(0, 10)}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  )
}

function CarriesSection({ open, closed }: { open: CarryItem[]; closed: CarryItem[] }) {
  if (open.length === 0 && closed.length === 0) return null
  return (
    <div className="mt-4 border-t border-bd pt-3" data-testid="carries">
      <div className="mb-1 eyebrow">
        dette portée · {open.length} ouverte{open.length > 1 ? 's' : ''} · {closed.length} fermée
        {closed.length > 1 ? 's' : ''}
      </div>
      <ul className="flex flex-col gap-0.5">
        {open.map((c, i) => (
          <li key={`o-${c.code}-${i}`} className="font-mono text-meta text-tx3">
            <span className="text-warn" aria-hidden>
              ○{' '}
            </span>
            <span className="text-tx2">{c.code}</span> {c.description}
            {c.disposition ? <span className="text-tx4"> → {c.disposition}</span> : null}
          </li>
        ))}
        {closed.map((c, i) => (
          <li key={`c-${c.code}-${i}`} className="font-mono text-meta text-tx4">
            <span className="text-ok" aria-hidden>
              ●{' '}
            </span>
            <span className="text-tx3">{c.code}</span> {c.description}
            {c.phase_closed ? <span> (phase {c.phase_closed})</span> : null}
          </li>
        ))}
      </ul>
    </div>
  )
}

function VerificationTable({ verification }: { verification: VerificationSummary | null }) {
  if (verification === null) {
    return (
      <div className="mt-4 border-t border-bd pt-3" data-testid="verification-table">
        <div className="eyebrow">
          vérification §1
        </div>
        <p className="mt-1 font-mono text-meta text-tx4">restituée au wrap-up (verification.md)</p>
      </div>
    )
  }
  return (
    <div className="mt-4 border-t border-bd pt-3" data-testid="verification-table">
      <div className="mb-1 eyebrow">
        vérification §1 · {verification.passed}/{verification.total_checks}
      </div>
      <ul className="flex flex-col gap-0.5">
        {verification.checks.map((c) => (
          <li key={c.number} className="flex items-baseline gap-2 font-mono text-meta">
            <span className={toneText(reviewTone(c.result))}>{c.result}</span>
            <span className="min-w-0 flex-1 truncate text-tx3">{c.name}</span>
            <span className="shrink-0 truncate text-tx4" title={c.command}>
              {c.command}
            </span>
          </li>
        ))}
      </ul>
    </div>
  )
}

function GlyphLegend() {
  return (
    <details className="mt-4 border-t border-bd pt-3" data-testid="glyph-legend">
      <summary className="cursor-pointer eyebrow hover:text-tx2">
        légende
      </summary>
      <div className="mt-1 flex flex-col gap-0.5 font-mono text-meta text-tx3">
        <span>
          codex : <span className="text-ok">✓</span> confirmé · <span className="text-warn">~</span> partiel ·{' '}
          <span className="text-bad">⚠</span> gap
        </span>
        <span>
          scope cut : <span className="text-ok">◦</span> respecté · <span className="text-warn">×</span> dévié
        </span>
        <span>
          dette : <span className="text-warn">○</span> ouverte · <span className="text-ok">●</span> fermée
        </span>
        <span className="text-tx4">verdicts restitués depuis les artefacts .planning/ — jamais calculés ici</span>
      </div>
    </details>
  )
}

/** The cross-sprint index (GET /api/sprint-history/all): a collapsible grid of
 * every detected sprint. Clicking a card drills into that sprint's procédé.
 * `phases_pass/phase_count` is a RESTITUTED count, never a fabricated score. */
function SprintIndex({
  sprints,
  viewing,
  onSelect,
}: {
  sprints: SprintSummary[]
  viewing: number
  onSelect: (sprint: number) => void
}) {
  const [open, setOpen] = useState(false)
  const sorted = useMemo(() => [...sprints].sort((a, b) => b.sprint - a.sprint), [sprints])
  if (sprints.length === 0) return null
  return (
    <div className="mb-4" data-testid="sprint-index">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className="mb-1 flex items-center gap-2 eyebrow hover:text-tx2"
      >
        <span aria-hidden>{open ? '▾' : '▸'}</span>
        tous les sprints · {sprints.length}
      </button>
      {open ? (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-1.5">
          {sorted.map((s) => (
            <button
              key={s.sprint}
              type="button"
              data-testid="sprint-card"
              onClick={() => onSelect(s.sprint)}
              title={`${s.version} · ${s.status}`}
              className={`flex flex-col gap-0.5 rounded-sm border px-2 py-1.5 text-left font-mono text-meta ${
                s.sprint === viewing ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:bg-s1'
              }`}
            >
              <span className="font-semibold">S{s.sprint}</span>
              <span className="text-tx4">{s.version}</span>
              <span className="text-tx4">
                {s.phases_pass}/{s.phase_count} ph{s.has_verification ? ' ✓v' : ''}
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  )
}

export function ProcedeSurface() {
  const [history, setHistory] = useState<SprintHistory | null>(null)
  const [status, setStatus] = useState<OperatorStatus | null>(null)
  const [allSprints, setAllSprints] = useState<SprintSummary[] | null>(null)
  const [selectedSprint, setSelectedSprint] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const [filter, setFilter] = useState('')

  // Global signals (once): the live position + the cross-sprint index.
  useEffect(() => {
    const controller = new AbortController()
    void Promise.allSettled([getStatus(controller.signal), getAllSprints(controller.signal)]).then(
      ([statR, allR]) => {
        if (controller.signal.aborted) return
        if (statR.status === 'fulfilled') setStatus(statR.value)
        if (allR.status === 'fulfilled') setAllSprints(allR.value.sprints)
      },
    )
    return () => controller.abort()
  }, [])

  // The displayed sprint history — re-fetched when the operator drills a sprint
  // (selectedSprint null = active sprint). The placeholder reset + per-sprint
  // view-state clear happen in `drillTo` (the click handler), not here, so the
  // effect never calls setState synchronously (react-hooks).
  useEffect(() => {
    const controller = new AbortController()
    getSprintHistory(selectedSprint ?? undefined, controller.signal)
      .then((h) => {
        if (controller.signal.aborted) return
        setHistory(h)
        setError(null)
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setError(err instanceof OperatorError ? `procédé indisponible (${err.status})` : 'procédé indisponible')
      })
    return () => controller.abort()
  }, [selectedSprint])

  const fileByPhase = useMemo(
    () => new Map((history?.preflight_bilan.phases ?? []).map((p) => [p.phase, p.file])),
    [history],
  )

  const filteredPhases = useMemo(() => {
    if (history === null) return []
    const q = filter.trim().toLowerCase()
    if (q === '') return history.phases
    return history.phases.filter((p) => p.letter.toLowerCase().includes(q) || p.title.toLowerCase().includes(q))
  }, [history, filter])

  if (error)
    return (
      <AdaptiveSurface kind="procede" testId="procede-surface-error" className="flex flex-col gap-2 p-5 font-mono text-meta text-warn">
        <span>{error}</span>
        {selectedSprint !== null ? (
          <button
            type="button"
            onClick={() => setSelectedSprint(null)}
            className="self-start rounded-sm border border-bd px-2 py-1 text-tx3 hover:bg-s1"
          >
            ← sprint actif
          </button>
        ) : null}
      </AdaptiveSurface>
    )
  if (history === null) return <AdaptiveSurface kind="procede" testId="procede-surface-loading" className="p-5 font-mono text-meta text-tx4">lecture du procédé…</AdaptiveSurface>

  const toggle = (letter: string) =>
    setExpanded((cur) => {
      const next = new Set(cur)
      if (next.has(letter)) next.delete(letter)
      else next.add(letter)
      return next
    })
  const expandAll = () => setExpanded(new Set(filteredPhases.map((p) => p.letter)))
  const collapseAll = () => setExpanded(new Set())

  // Drill to a sprint (null = active). Clears the placeholder + per-sprint view
  // state in the SAME click so a drill never flashes the previous tree nor
  // bleeds expanded rows / the filter across sprints. (Event handler, not an
  // effect — so no synchronous-setState-in-effect.)
  const drillTo = (target: number | null) => {
    setHistory(null)
    setExpanded(new Set())
    setFilter('')
    setSelectedSprint(target)
  }

  return (
    <AdaptiveSurface kind="procede" testId="procede-surface" className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <div className="mb-3 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-meta text-tx2">
        <h2 className="font-sans text-card font-semibold text-tx">Sprint {history.sprint}</h2>
        <span className="text-tx3">{history.status}</span>
        <span className="adaptive-secondary text-tx4">·</span>
        <span className="text-tx">{history.branch}</span>
        <span className="adaptive-secondary text-tx4">▸ {history.head}</span>
        <span className="adaptive-secondary text-tx4">·</span>
        <span className="adaptive-secondary text-tx3">
          {history.phase_commits} commits de phase · {history.chore_commits} chore
        </span>
      </div>

      {allSprints ? (
        <SprintIndex
          sprints={allSprints}
          viewing={selectedSprint ?? status?.sprint ?? history.sprint}
          onSelect={(n) => drillTo(n === (status?.sprint ?? history.sprint) ? null : n)}
        />
      ) : null}

      {selectedSprint === null ? (
        <LiveProcessBanner history={history} status={status} />
      ) : (
        <div
          data-testid="drill-banner"
          className="mb-4 flex items-center gap-3 rounded-md border border-bd bg-s1 px-3 py-2"
        >
          <span className="font-mono text-meta text-tx2">
            sprint {history.sprint} · {history.status} · archivé
          </span>
          <button
            type="button"
            onClick={() => drillTo(null)}
            className="ml-auto rounded-sm border border-bd px-2 py-0.5 font-mono text-meta text-tx3 hover:bg-s2"
          >
            ← sprint actif
          </button>
        </div>
      )}

      {/* preflight bilan + verdict frise (fold V8) */}
      <div className="mb-4 flex flex-wrap items-center gap-2 rounded-md border border-bd bg-s1 px-3 py-2">
        <span className="eyebrow">frise</span>
        <span className="font-mono text-meta text-tx3">
          préflights {history.preflight_bilan.total} · {history.preflight_bilan.execute} EXECUTE ·{' '}
          {history.preflight_bilan.plan_adapt} PLAN-ADAPT · {history.preflight_bilan.design_conflict} conflit
        </span>
        <span className="ml-auto flex items-center gap-1">
          {history.phases.map((p) => (
            <span
              key={p.letter}
              title={`phase ${p.letter} · ${p.review_verdict ?? '—'}`}
              className={`h-2 w-2 rounded-sm ${toneBg(reviewTone(p.review_verdict))}`}
              aria-hidden
            />
          ))}
        </span>
      </div>

      <TestsBilan tests={history.tests} />

      {/* filter + multi-expand controls */}
      <div className="mb-2 flex items-center gap-2">
        <input
          type="text"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="filtrer les phases…"
          aria-label="filtrer les phases"
          data-testid="phase-filter"
          className="min-w-0 flex-1 rounded-sm border border-field bg-s0 px-2 py-1 font-mono text-body text-tx placeholder:text-tx4 focus:border-info"
        />
        <button
          type="button"
          onClick={expandAll}
          className="rounded-sm border border-bd bg-s1 px-2 py-1 font-mono text-meta text-tx3 hover:bg-s2"
        >
          tout déplier
        </button>
        <button
          type="button"
          onClick={collapseAll}
          className="rounded-sm border border-bd bg-s1 px-2 py-1 font-mono text-meta text-tx3 hover:bg-s2"
        >
          tout replier
        </button>
      </div>

      <div className="flex flex-col">
        {filteredPhases.length === 0 ? (
          <div className="py-3 font-mono text-meta text-tx4">aucune phase ne correspond au filtre</div>
        ) : (
          filteredPhases.map((p) => (
            <PhaseNode
              key={p.letter}
              phase={p}
              preflightFile={fileByPhase.get(p.letter) ?? null}
              expanded={expanded.has(p.letter)}
              onToggle={() => toggle(p.letter)}
            />
          ))
        )}
      </div>

      {history.scope_cuts.length > 0 ? (
        <div className="mt-4 border-t border-bd pt-3">
          <div className="mb-1 eyebrow">
            scope cuts restitués
          </div>
          <ul className="flex flex-col gap-0.5">
            {history.scope_cuts.map((s) => (
              <li key={s.number} className="font-mono text-meta text-tx3">
                <span className={s.respected ? 'text-ok' : 'text-warn'}>{s.respected ? '◦' : '×'}</span> {s.item}{' '}
                <span className="text-tx4">→ {s.target}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      <CarriesSection open={history.carries_open} closed={history.carries_closed} />
      <VerificationTable verification={history.verification} />
      <CommitTimeline commits={history.commits} />
      <GlyphLegend />
    </AdaptiveSurface>
  )
}
