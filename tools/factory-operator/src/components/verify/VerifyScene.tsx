// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase H — the VERIFY-plein focal scene (variante B: "décomposition,
// jamais verdict"). The bespoke diff-viewer (working-tree hunks computed in
// Rust, fold V1/V2/V3) is the central investment; the permanent bottom band
// restitutes the live gates 1:1 (fold V4-core) and a NAMED état slot that never
// says a verdict. The 3 tabs are Diff (default) · Aperçu scellé · Preuve —
// the last two are DISABLED "à venir (S81)" (coding the sealed preview would
// reopen the in-vivo app-authoring P1). Gates is NOT a tab: it lives in the
// band, never hidden. The terminal-PTY stays reachable as a secondary tool so
// the live `claude` session is not lost — the diff is the default surface.
//
// Sprint 80 Phase E — this async surface confines the Motion-lib (the engine
// never reaches the hero `index` chunk). <MotionProvider> wraps it; the band
// REVEALS in a short stagger (signature 3) and the état FLIPS on change
// (signature 2). The flip animates a RESTITUTED état string — it never
// fabricates a verdict (the diff-viewer + gates panel themselves are motion-free
// and live in the dedicated `diff-viewer` chunk).
import { useState } from 'react'
import { Terminal } from './Terminal'
import { GatesPanel } from './GatesPanel'
import { DiffViewer } from './plein/DiffViewer'
import { pickVerifyEtat, VERIFY_ETAT } from '../../lib/verdict'
import { useVerifyData } from '../../state/useVerifyData'
import type { Operator } from '../../state/useOperator'
import { GateFlip } from '../motion/GateFlip'
import { MotionProvider } from '../motion/MotionProvider'
import { Reveal, RevealItem } from '../motion/Reveal'

type Tool = 'diff' | 'terminal'

function Tab({ label, active, disabled }: { label: string; active?: boolean; disabled?: boolean }) {
  if (disabled) {
    return (
      <span
        className="flex cursor-not-allowed items-center gap-1.5 py-2.5 font-sans text-[12px] text-tx4"
        title="à venir (S81)"
      >
        {label}
        <span className="rounded-sm bg-s3 px-1 py-px font-mono text-[8px] uppercase tracking-wide text-tx4">
          à venir
        </span>
      </span>
    )
  }
  return (
    <span
      className={
        active
          ? 'border-b-2 border-tx py-2.5 font-sans text-[12px] font-semibold text-tx'
          : 'py-2.5 font-sans text-[12px] text-tx3'
      }
    >
      {label}
    </span>
  )
}

/** Transient restitution of a hunk intention routed to the session: the wall
 * (requires_gate), a launch failure, or in-flight. The full transcript lives
 * in STEER — VERIFY only confirms the intention left for the session. */
function IntentStrip({ op }: { op: Operator }) {
  const t = op.turn
  if (t.gate) {
    return (
      <div
        data-testid="verify-intent-gate"
        className="flex items-center gap-2 border-t border-mur/40 bg-mur-bg px-4 py-2 font-mono text-[10px] text-mur"
      >
        <span aria-hidden>⛔</span>
        {t.gate}
      </div>
    )
  }
  if (t.launchError) {
    return (
      <div className="border-t border-bd2 bg-s2 px-4 py-2 font-mono text-[10px] text-warn">{t.launchError}</div>
    )
  }
  if (t.busy) {
    return (
      <div className="border-t border-bd2 bg-s2 px-4 py-2 font-mono text-[10px] text-tx3">
        transmission de l’intention à la session…
      </div>
    )
  }
  return null
}

export function VerifyScene({ op }: { op: Operator }) {
  const [tool, setTool] = useState<Tool>('diff')
  const { diff, gates, diffError, gatesError, loading, reload } = useVerifyData()

  // staged + unstaged: a partially staged file legitimately appears in BOTH
  // (git semantics) and is listed twice in the change-set — the caption keeps
  // the two counts. A merged single-row view is S81 (review P3-f).
  const files = diff ? [...diff.unstaged, ...diff.staged] : []
  const etat = diffError ? 'unavailable' : pickVerifyEtat({ loading, hasChanges: files.length > 0 })

  const onHunkIntent = (file: string, hunkHeader: string) => {
    op.launch(`Examiner et corriger ${file} — hunk ${hunkHeader}`, 'phase-review')
  }

  return (
    <MotionProvider>
      <div data-testid="verify-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
        <div className="flex items-center gap-2.5 border-b border-bd px-5 py-3">
          <span className="h-1.5 w-1.5 rounded-full bg-info" aria-hidden />
          <span className="font-sans text-xs font-semibold tracking-wide text-tx">VERIFY</span>
          <span className="font-sans text-xs text-tx3">— examiner le diff · lire les gates · aperçu scellé · preuve</span>
          <button
            type="button"
            data-testid="verify-tool-toggle"
            aria-pressed={tool === 'terminal'}
            onClick={() => setTool((t) => (t === 'terminal' ? 'diff' : 'terminal'))}
            className="ml-auto rounded-sm border border-bd px-2 py-0.5 font-mono text-[10px] text-tx3 hover:bg-s2"
            title="le terminal PTY reste accessible comme outil secondaire"
          >
            {tool === 'terminal' ? '← Diff' : 'Terminal ▸'}
          </button>
        </div>

        {tool === 'diff' ? (
          <>
            <div className="flex items-center gap-4 border-b border-bd px-5">
              <Tab label="Diff" active />
              <Tab label="Aperçu scellé" disabled />
              <Tab label="Preuve" disabled />
              <span className="ml-auto font-mono text-[9.5px] text-tx4">la vérité = git diff, pas un buffer</span>
            </div>
            {diffError ? (
              <div className="flex flex-1 items-center justify-center p-8 font-mono text-[11px] text-warn">
                {diffError}
              </div>
            ) : loading ? (
              <div className="flex flex-1 items-center justify-center p-8 font-mono text-[11px] text-tx4">
                lecture du diff…
              </div>
            ) : (
              <DiffViewer
                files={files}
                autoFocus
                caption={
                  diff
                    ? `working-tree · ${diff.unstaged.length} non indexés · ${diff.staged.length} indexés`
                    : 'working-tree'
                }
                truncated={diff?.truncated}
                emptyLabel="arbre de travail propre · rien à examiner"
                onHunkIntent={onHunkIntent}
              />
            )}
          </>
        ) : (
          <Terminal />
        )}

        {op.hasTurn ? <IntentStrip op={op} /> : null}

        {/* permanent gates + état band — restituted 1:1, never a verdict.
           Revealed in a short upward stagger (signature 3) on entry. */}
        <Reveal className="flex flex-col border-t border-bd2">
          <RevealItem>
            <GatesPanel
              gates={gates}
              loading={loading}
              error={gatesError}
              runRev={diff?.head ?? null}
              onReload={reload}
            />
          </RevealItem>
          <RevealItem className="flex items-center gap-3 border-t border-bd2 bg-s3 px-4 py-2">
            <span className="font-mono text-[8px] font-semibold uppercase tracking-wide text-info">état</span>
            <span data-testid="verify-etat" className="font-mono text-[11px] tabular-nums text-tx2">
              {/* gate flip (signature 2): the restituted named état flips on change. */}
              <GateFlip value={VERIFY_ETAT[etat]} />
            </span>
          </RevealItem>
        </Reveal>
      </div>
    </MotionProvider>
  )
}
