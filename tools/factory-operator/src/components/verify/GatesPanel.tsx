// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase H — the permanent VERIFY gates band (fold V4-core). It
// restitutes `GET /api/gates` 1:1: each `GateEntryView` keeps its DISTINCT
// status (✓ tenue / ✕ bloquant / • informatif|non exécutée / — hors
// périmètre), NEVER flattened, NEVER aggregated into an overall verdict, NEVER
// a "PASS" word, NEVER a % score (cardinal "0 verdict calculé UI" + the
// scan-front-discipline gate, extended anti-score in Phase I). A single gate
// name can recur under two statuses (lint-planning: blocking errors AND
// informational warnings), so entries are keyed by `(gate, status)`.
//
// V5 (per-line gutter pulse) and V6 (filter-by-gate over the change-set) are
// DEGRADED to S81: the wire `GateIssueView.line` is always null and `file` is
// a `.planning/` basename, not a change-set path (preflight §3). Freshness:
// `run@<rev>` restitutes the head of the diff co-fetched in the same cycle
// (useVerifyData) — an honest restitution, never a verdict. There is NO
// "◦ obsolète" divergence badge in S80: the only live head (the rail's
// `/api/context`) is fetched once at mount and would lie after the first
// in-session commit (review P1-1) → carry S81 (re-polled head or a rev on
// `/api/gates`).
import { useState } from 'react'
import type { GateEntryView } from '../../api/operator'
import { gateStatusGlyph, gateStatusLabel, gateStatusTone, toneText } from '../../lib/verdict'

function entryKey(entry: GateEntryView): string {
  return `${entry.gate}:${entry.status}`
}

function GateGlyph({ entry }: { entry: GateEntryView }) {
  const tone = toneText(gateStatusTone(entry.status))
  return (
    <span className="inline-flex items-center gap-1" title={`${entry.gate} · ${gateStatusLabel(entry.status)}`}>
      <span className="text-tx3">{entry.gate}</span>
      <span className={tone} aria-hidden>
        {gateStatusGlyph(entry.status)}
      </span>
      {entry.issues.length > 0 ? <span className={tone}>{entry.issues.length}</span> : null}
      <span className="sr-only">{gateStatusLabel(entry.status)}</span>
    </span>
  )
}

export function GatesPanel({
  gates,
  loading,
  error,
  runRev,
  onReload,
}: {
  gates: GateEntryView[] | null
  loading: boolean
  error: string | null
  runRev: string | null
  onReload: () => void
}) {
  const [open, setOpen] = useState(false)

  return (
    <div data-testid="verify-gates" className="flex flex-col bg-s2">
      <div className="flex items-center gap-2 px-4 py-2 font-mono text-meta">
        <span className="eyebrow">gates</span>

        {loading ? (
          <span className="text-tx4">lecture des gates…</span>
        ) : error ? (
          <span className="text-warn">{error}</span>
        ) : gates && gates.length > 0 ? (
          <span className="flex flex-wrap items-center gap-x-3 gap-y-0.5">
            {gates.map((entry) => (
              <GateGlyph key={entryKey(entry)} entry={entry} />
            ))}
          </span>
        ) : (
          <span className="text-tx4">aucune gate restituée</span>
        )}

        <span className="ml-auto flex items-center gap-2.5 text-tx4">
          {runRev ? (
            <span className="text-tx4" title="révision (head) du diff co-récupéré">
              run@{runRev}
            </span>
          ) : null}
          <button
            type="button"
            data-testid="gates-reload"
            onClick={onReload}
            className="rounded-sm px-1.5 py-0.5 text-tx3 hover:bg-s3 hover:text-tx2"
            title="relancer la lecture du diff et des gates"
          >
            relancer
          </button>
          <button
            type="button"
            data-testid="gates-tray-toggle"
            aria-expanded={open}
            onClick={() => setOpen((v) => !v)}
            className="rounded-sm px-1.5 py-0.5 text-tx3 hover:bg-s3 hover:text-tx2"
          >
            détails {open ? '▾' : '▸'}
          </button>
        </span>
      </div>

      {open ? (
        <div data-testid="verify-gates-tray" className="border-t border-bd bg-s1 px-4 py-2.5">
          <div className="mb-1.5 eyebrow">
            diagnostic · aucun agrégat · chaque gate garde son état
          </div>
          {gates && gates.length > 0 ? (
            <ul className="flex flex-col gap-1">
              {gates.map((entry) => {
                const tone = toneText(gateStatusTone(entry.status))
                return (
                  <li key={entryKey(entry)} className="font-mono text-meta">
                    <div className="flex items-center gap-2">
                      <span className={`w-3 text-center ${tone}`} aria-hidden>
                        {gateStatusGlyph(entry.status)}
                      </span>
                      <span className="sr-only">{gateStatusLabel(entry.status)}</span>
                      <span className="text-tx2">{entry.gate}</span>
                      <span className={tone}>{gateStatusLabel(entry.status)}</span>
                    </div>
                    {entry.issues.length > 0 ? (
                      <ul className="ms-5 flex flex-col gap-0.5 py-0.5">
                        {entry.issues.map((issue, i) => (
                          <li key={i} className="text-tx3">
                            <span className="text-tx4" aria-hidden>
                              ·{' '}
                            </span>
                            {issue.file ? <span className="text-tx4">{issue.file} — </span> : null}
                            {issue.message}
                          </li>
                        ))}
                      </ul>
                    ) : null}
                  </li>
                )
              })}
            </ul>
          ) : (
            <div className="font-mono text-meta text-tx4">rien à détailler</div>
          )}
        </div>
      ) : null}
    </div>
  )
}
