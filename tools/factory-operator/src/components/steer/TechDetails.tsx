// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the "▸ détails techniques" disclosure: the jargon
// (kind · provider · preflight · hash) folded out of the primary intention
// CTA (intentions-pas-jargon). It hosts:
//   - S3/D3 the prompt inspector: the REAL assembled prompt for the active
//     intention, via GET /api/prompt/{kind}?provider= — read-only mono.
//   - S4 a provider reachability probe: GET /api/providers as an honest
//     "backend joignable" diagnostic (the prompt-adaptation set, a distinct
//     axis from the chat execution provider).
// Both are fetched lazily, only once opened. The loading state is DERIVED
// from a (kind, provider) key so the effect never calls setState
// synchronously (react-hooks/set-state-in-effect); the only state write is
// in the async resolution.

import { useEffect, useState } from 'react'
import { getPrompt, getProviders, OperatorError } from '../../api/operator'
import type { ExecProvider } from '../../catalog/intentions'

interface Inspect {
  key: string
  prompt: string | null
  promptError: string | null
  providers: string[] | null
}

export function TechDetails({
  open,
  kind,
  provider,
}: {
  open: boolean
  kind: string
  provider: ExecProvider
}) {
  const [state, setState] = useState<Inspect | null>(null)
  const currentKey = `${kind}::${provider}`

  useEffect(() => {
    if (!open) return
    const key = `${kind}::${provider}`
    const controller = new AbortController()
    Promise.allSettled([
      getPrompt(kind, provider, controller.signal),
      getProviders(controller.signal),
    ]).then(([promptRes, providersRes]) => {
      if (controller.signal.aborted) return
      const prompt = promptRes.status === 'fulfilled' ? promptRes.value.content : null
      const promptError =
        promptRes.status === 'rejected'
          ? promptRes.reason instanceof OperatorError
            ? `prompt indisponible (${promptRes.reason.status})`
            : 'prompt indisponible'
          : null
      const providers = providersRes.status === 'fulfilled' ? providersRes.value.providers : null
      setState({ key, prompt, promptError, providers })
    })
    return () => controller.abort()
  }, [open, kind, provider])

  if (!open) return null

  // Stale (or never-loaded) result for this kind/provider → still loading.
  const ready = state !== null && state.key === currentKey
  const providers = ready ? state.providers : null
  const prompt = ready ? state.prompt : null
  const promptError = ready ? state.promptError : null

  return (
    <div
      data-testid="tech-details"
      className="flex flex-col gap-2 rounded-md border border-bd bg-s0 px-3 py-2.5 font-mono text-meta leading-relaxed text-tx2"
    >
      <div>
        kind&nbsp;&nbsp;&nbsp;&nbsp;= <span className="text-tx">{kind}</span>
      </div>
      <div>
        provider = <span className="text-tx">{provider}</span>
        {providers ? (
          <span className="text-tx4"> · backend joignable ({providers.length})</span>
        ) : !ready ? (
          <span className="text-tx4"> · sonde…</span>
        ) : (
          <span className="text-tx4"> · backend muet</span>
        )}
      </div>
      <div className="border-t border-bd pt-2">
        <div className="mb-1 eyebrow">
          prompt assemblé (lecture seule)
        </div>
        {!ready ? (
          <div className="text-tx4">assemblage…</div>
        ) : prompt !== null ? (
          <pre className="max-h-40 overflow-auto whitespace-pre-wrap break-words font-mono text-meta text-tx3">
            {prompt}
          </pre>
        ) : (
          <div className="text-warn">{promptError}</div>
        )}
      </div>
    </div>
  )
}
