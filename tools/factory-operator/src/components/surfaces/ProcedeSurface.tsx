// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the arbre de procédé (folds A1/U1, the SBFB signature:
// process-as-artifact). It restitutes the gisement ALREADY computed by Rust
// (sprint_history.rs): sprint → phase → commit → artefact, with each phase's
// RESTITUTED preflight/review/Codex verdicts. The cardinal rule holds end to
// end: every verdict is read off the on-disk artifacts (the UI computes
// nothing, scores nothing). Each verdict carries its SOURCE artifact filename
// (fold U2, provenance-de-verdict — preflight_bilan.phases[].file). The frise
// (fold V8) is the condensed verdict strip; clicking a phase commit opens its
// diff (fold J11, DiffView) and conformity card (folds U3/A9/V10). The in-app
// artifact CONTENT reader is S81; here the provenance is the named source.
import { useEffect, useState } from 'react'
import { getSprintHistory, OperatorError, type SprintHistory } from '../../api/operator'
import { preflightTone, reviewTone, toneBg, toneText } from '../../lib/verdict'
import { DiffView } from './DiffView'
import { ConformiteCard } from './ConformiteCard'
import { useCommitDiff } from '../../state/useCommitDiff'

function VerdictPill({ verdict, tone }: { verdict: string | null; tone: string }) {
  return (
    <span className={`font-mono text-[10px] ${tone}`} data-testid="verdict-pill">
      {verdict ?? '—'}
    </span>
  )
}

function PhaseNode({
  letter,
  title,
  commitSha,
  preflightVerdict,
  preflightFile,
  reviewVerdict,
  codex,
  rustDelta,
  vitestDelta,
  deliverables,
  findings,
  expanded,
  onToggle,
}: {
  letter: string
  title: string
  commitSha: string | null
  preflightVerdict: string | null
  preflightFile: string | null
  reviewVerdict: string | null
  codex: { confirmed: number | null; partial: number | null; gap: number | null }
  rustDelta: number
  vitestDelta: number
  deliverables: string[]
  findings: { severity: string; code: string; description: string; status: string }[]
  expanded: boolean
  onToggle: () => void
}) {
  return (
    <div className="border-l-2 border-bd pl-3">
      <button
        type="button"
        data-testid="procede-phase"
        onClick={onToggle}
        className="flex w-full items-center gap-2.5 py-1.5 text-left hover:bg-s1"
      >
        <span className="font-mono text-[11px] font-semibold text-tx">{letter}</span>
        <span className="min-w-0 flex-1 truncate font-sans text-[12px] text-tx2">{title || '—'}</span>
        <VerdictPill verdict={preflightVerdict} tone={toneText(preflightTone(preflightVerdict))} />
        <span className="text-tx4" aria-hidden>
          ·
        </span>
        <VerdictPill verdict={reviewVerdict} tone={toneText(reviewTone(reviewVerdict))} />
        {commitSha ? <span className="font-mono text-[9.5px] text-tx4">{commitSha}</span> : null}
        <span className="font-mono text-[10px] text-tx4" aria-hidden>
          {expanded ? '▾' : '▸'}
        </span>
      </button>
      {expanded ? (
        <div className="mb-2 ml-1 flex flex-col gap-2 border-l border-bd pl-3">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10px] text-tx3">
            <span>
              préflight <span className={toneText(preflightTone(preflightVerdict))}>{preflightVerdict ?? '—'}</span>
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
              review <span className={toneText(reviewTone(reviewVerdict))}>{reviewVerdict ?? '—'}</span>
            </span>
            <span>
              codex {codex.confirmed ?? 0}✓ {codex.partial ?? 0}~ {codex.gap ?? 0}⚠
            </span>
            <span>
              Δ +{rustDelta} Rust · +{vitestDelta} Vitest
            </span>
          </div>
          {deliverables.length > 0 ? (
            <div data-testid="phase-deliverables">
              <div className="mb-0.5 font-mono text-[8.5px] uppercase tracking-wider text-tx4">livrables</div>
              <ul className="flex flex-col gap-0.5">
                {deliverables.map((d, i) => (
                  <li key={i} className="font-mono text-[10px] text-tx3">
                    <span className="text-tx4" aria-hidden>
                      ·{' '}
                    </span>
                    {d}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {findings.length > 0 ? (
            <div data-testid="phase-findings">
              <div className="mb-0.5 font-mono text-[8.5px] uppercase tracking-wider text-tx4">
                findings review <span className="text-tx4">· {findings.length}</span>
              </div>
              <ul className="flex flex-col gap-0.5">
                {findings.map((f, i) => (
                  <li key={i} className="font-mono text-[10px] text-tx3">
                    <span className={f.status === 'resolved' ? 'text-ok' : 'text-warn'}>
                      {f.severity}
                    </span>{' '}
                    {f.description}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {commitSha ? <PhaseDiff sha={commitSha} /> : null}
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
        <div className="font-mono text-[10px] text-tx4">diff du commit…</div>
      ) : error ? (
        <div className="font-mono text-[10px] text-warn">{error}</div>
      ) : diff ? (
        <DiffView diff={diff} />
      ) : null}
    </div>
  )
}

export function ProcedeSurface() {
  const [history, setHistory] = useState<SprintHistory | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<string | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    getSprintHistory(undefined, controller.signal)
      .then((h) => {
        if (!controller.signal.aborted) setHistory(h)
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setError(err instanceof OperatorError ? `procédé indisponible (${err.status})` : 'procédé indisponible')
      })
    return () => controller.abort()
  }, [])

  if (error) return <div className="p-5 font-mono text-[11px] text-warn">{error}</div>
  if (history === null) return <div className="p-5 font-mono text-[11px] text-tx4">lecture du procédé…</div>

  const fileByPhase = new Map(history.preflight_bilan.phases.map((p) => [p.phase, p.file]))

  return (
    <div data-testid="procede-surface" className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <div className="mb-3 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-tx2">
        <span className="font-sans text-[13px] font-semibold text-tx">Sprint {history.sprint}</span>
        <span className="text-tx3">{history.status}</span>
        <span className="text-tx4">·</span>
        <span className="text-tx">{history.branch}</span>
        <span className="text-tx4">▸ {history.head}</span>
        <span className="text-tx4">·</span>
        <span className="text-tx3">
          {history.phase_commits} commits de phase · {history.chore_commits} chore
        </span>
      </div>

      {/* preflight bilan + verdict frise (fold V8) */}
      <div className="mb-4 flex flex-wrap items-center gap-2 rounded-md border border-bd bg-s1 px-3 py-2">
        <span className="font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">frise</span>
        <span className="font-mono text-[10px] text-tx3">
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

      <div className="flex flex-col">
        {history.phases.map((p) => (
          <PhaseNode
            key={p.letter}
            letter={p.letter}
            title={p.title}
            commitSha={p.commit_sha}
            preflightVerdict={p.preflight_verdict}
            preflightFile={fileByPhase.get(p.letter) ?? null}
            reviewVerdict={p.review_verdict}
            codex={{ confirmed: p.codex_confirmed, partial: p.codex_partial, gap: p.codex_gap }}
            rustDelta={p.rust_delta}
            vitestDelta={p.vitest_delta}
            deliverables={p.deliverables}
            findings={p.findings}
            expanded={expanded === p.letter}
            onToggle={() => setExpanded((cur) => (cur === p.letter ? null : p.letter))}
          />
        ))}
      </div>

      {history.scope_cuts.length > 0 ? (
        <div className="mt-4 border-t border-bd pt-3">
          <div className="mb-1 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
            scope cuts restitués
          </div>
          <ul className="flex flex-col gap-0.5">
            {history.scope_cuts.map((s) => (
              <li key={s.number} className="font-mono text-[10px] text-tx3">
                <span className={s.respected ? 'text-ok' : 'text-warn'}>{s.respected ? '◦' : '×'}</span> {s.item}{' '}
                <span className="text-tx4">→ {s.target}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  )
}
