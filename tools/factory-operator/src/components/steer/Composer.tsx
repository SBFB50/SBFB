// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the intention composer. STEER variante B: in the empty
// state it shows in GRAND (centred, discoverable); with a live session it
// docks compact at the bottom while the atelier dominates. The CTA speaks in
// INTENTIONS ("Préparer la phase"…), never in jargon; the technical knobs
// (kind · provider · prompt) fold under "▸ détails techniques". The provider
// is a discreet attribute on the execution axis {claude, local, network}.

import { useState } from 'react'
import { cn } from '../../lib/cn'
import { EXEC_PROVIDERS, INTENTIONS, type ExecProvider } from '../../catalog/intentions'
import { TechDetails } from './TechDetails'

export function Composer({
  variant,
  provider,
  onProvider,
  busy,
  onLaunch,
}: {
  variant: 'grand' | 'dock'
  provider: ExecProvider
  onProvider: (provider: ExecProvider) => void
  busy: boolean
  onLaunch: (text: string, kind: string) => void
}) {
  const [intentId, setIntentId] = useState(INTENTIONS[0].id)
  const [text, setText] = useState('')
  const [showDetails, setShowDetails] = useState(false)

  const active = INTENTIONS.find((i) => i.id === intentId) ?? INTENTIONS[0]
  const canLaunch = text.trim().length > 0 && !busy

  function submit() {
    if (!canLaunch) return
    onLaunch(text, active.kind)
    setText('')
  }

  const grand = variant === 'grand'

  return (
    <section
      data-testid="composer"
      aria-label="Composeur d'intention"
      className={cn('flex flex-col', grand ? 'gap-4 p-7' : 'gap-3 border-t border-bd bg-s1 p-4')}
    >
      <div className="font-sans text-[9px] font-semibold uppercase tracking-[0.15em] text-tx4">
        composeur d'intention
      </div>

      <div className="flex flex-wrap gap-2">
        {INTENTIONS.map((intent) => {
          const selected = intent.id === intentId
          return (
            <button
              key={intent.id}
              type="button"
              aria-pressed={selected}
              onClick={() => setIntentId(intent.id)}
              className={cn(
                'rounded-sm border px-3.5 py-2 font-sans text-[12.5px] font-medium transition-none',
                selected
                  ? 'border-bd2 bg-s2 text-tx'
                  : 'border-bd bg-transparent text-tx2 hover:border-bd2',
              )}
            >
              {intent.label}
            </button>
          )
        })}
      </div>

      <textarea
        data-testid="composer-input"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
            e.preventDefault()
            submit()
          }
        }}
        placeholder={`Décrivez l'intention en clair… « ${active.hint} »`}
        rows={grand ? 4 : 2}
        className="resize-none rounded-md border border-bd2 bg-s1 px-4 py-3 font-sans text-[13.5px] leading-relaxed text-tx placeholder:text-tx3 focus:border-info focus:outline-none"
      />

      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3.5">
          <label className="flex items-center gap-1.5 font-mono text-[10.5px] text-tx3">
            <span className="text-tx4">agent</span>
            <select
              data-testid="provider-select"
              value={provider}
              onChange={(e) => onProvider(e.target.value as ExecProvider)}
              className="rounded-sm border border-bd bg-s2 px-1.5 py-1 font-mono text-[10.5px] text-tx2 focus:border-info focus:outline-none"
            >
              {EXEC_PROVIDERS.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label.toLowerCase()} · {p.note}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            aria-expanded={showDetails}
            onClick={() => setShowDetails((v) => !v)}
            className="font-mono text-[10.5px] text-tx4 hover:text-tx3"
          >
            {showDetails ? '▾' : '▸'} détails techniques
          </button>
        </div>

        <button
          type="button"
          data-testid="composer-launch"
          disabled={!canLaunch}
          onClick={submit}
          className={cn(
            'rounded-sm px-4 py-2.5 font-sans text-[12.5px] font-semibold transition-none',
            canLaunch ? 'bg-tx text-s0 hover:bg-tx/90' : 'cursor-not-allowed bg-s3 text-tx4',
          )}
        >
          {busy ? 'lancement…' : "Lancer l'intention"}
        </button>
      </div>

      <TechDetails open={showDetails} kind={active.kind} provider={provider} />
    </section>
  )
}
