// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the sealed context-pack inspector (fold S2) + hash-drift
// markers (fold D2) + the non-authoritative handoff brouillon (J13). It POSTs
// /api/context-pack to show the EXACT sealed, hashed pack a fresh agent
// session would receive — references only (`{path, hash, exists}`), never
// inlined content. When a chat session is active, it fetches that session's
// SEALED pack (/chat/{id}/log) and marks any reference whose on-disk hash has
// since changed as "◦ dérive — relu" (freshness, never a verdict). The pack is
// the ONLY forward affordance past the MUR: it is a handoff, never a PASS —
// the Operator cannot grant a verdict (mirrors operator_server.rs:732/754).
//
// State is written ONLY from async resolutions (the reset/loading state is
// DERIVED by key, never set synchronously in an effect — react-hooks).
import { useEffect, useState } from 'react'
import { getChatLog, postContextPack, OperatorError, type ContextPack, type HashRef } from '../../api/operator'
import { copyText } from '../../lib/clipboard'
import { AdaptiveSurface } from '../AdaptiveSurface'

interface PackGroup {
  label: string
  refs: HashRef[]
}

// Tolerates a PARTIAL pack: the chat/session pack sealed by the backend
// (operator_server.rs handle_chat_session) is REDUCED — it omits agent_system,
// specialized_prompt, process_docs and active_artifacts. Both the fresh full
// pack (render) and the sealed reduced pack (D2 drift baseline) flow through
// here, so every group coalesces a missing array to `[]` (Codex round-2: a raw
// `for (const r of pack.process_docs)` over an absent field would throw — caught
// by the effect's .catch, silently killing D2 drift detection on a live session).
function groups(pack: Partial<ContextPack>): PackGroup[] {
  const prompts = [
    pack.base_prompt,
    pack.universal_prompt,
    pack.handoff_prompt,
    pack.agent_system,
    pack.specialized_prompt ?? undefined,
  ].filter((r): r is HashRef => !!r)
  return [
    { label: 'prompts', refs: prompts },
    { label: 'docs de procédé', refs: pack.process_docs ?? [] },
    { label: 'knowledge consultatif', refs: pack.authoring_knowledge ?? [] },
    { label: 'artefacts du sprint actif', refs: pack.active_artifacts ?? [] },
  ]
}

function sealedMap(pack: Partial<ContextPack>): Map<string, string> {
  const map = new Map<string, string>()
  for (const g of groups(pack)) {
    for (const r of g.refs) if (r.hash) map.set(r.path, r.hash)
  }
  return map
}

function basename(path: string): string {
  const parts = path.split('/')
  return parts[parts.length - 1] || path
}

function HashRefRow({ entry, drifted }: { entry: HashRef; drifted: boolean }) {
  const [copied, setCopied] = useState<'path' | 'hash' | null>(null)

  useEffect(() => {
    if (copied === null) return
    const t = setTimeout(() => setCopied(null), 1200)
    return () => clearTimeout(t)
  }, [copied])

  const copy = (kind: 'path' | 'hash', text: string) => {
    void copyText(text).then((ok) => {
      if (ok) setCopied(kind)
    })
  }

  return (
    <div className="flex items-center gap-2 border border-dashed border-bd2 bg-s0 px-2.5 py-1.5">
      <span className="w-3 text-center font-mono text-meta text-tx4" aria-hidden>
        ◇
      </span>
      <button
        type="button"
        onClick={() => copy('path', entry.path)}
        data-testid="copy-path"
        title="copier le chemin"
        className="min-w-0 flex-1 truncate text-left font-mono text-meta text-tx2 hover:text-tx"
      >
        <span className="text-tx4">{entry.path.slice(0, entry.path.length - basename(entry.path).length)}</span>
        <span className="text-tx">{basename(entry.path)}</span>
      </button>
      {copied === 'path' ? <span className="font-mono text-meta text-ok">copié</span> : null}
      {entry.exists && entry.hash ? (
        <button
          type="button"
          onClick={() => copy('hash', entry.hash!)}
          data-testid="copy-hash"
          title="copier l'empreinte"
          className="rounded-sm border border-bd bg-s1 px-1 py-0.5 font-mono text-meta tabular-nums text-tx3 hover:text-tx"
        >
          {copied === 'hash' ? 'copié' : entry.hash}
        </button>
      ) : (
        <span className="font-mono text-meta text-warn">absent</span>
      )}
      {drifted ? (
        <span className="font-mono text-meta text-warn" title="le fichier a changé depuis le scellé de la session">
          ◦ dérive — relu
        </span>
      ) : null}
    </div>
  )
}

interface PackResolved {
  key: string
  pack: ContextPack | null
  error: string | null
}

interface SealedResolved {
  sid: string
  map: Map<string, string> | null
}

export function ContextPackInspector({
  sessionId,
  intent,
}: {
  sessionId?: string | null
  intent?: string
}) {
  const intentKey = intent ?? 'inspection du pack'
  const [packState, setPackState] = useState<PackResolved | null>(null)
  const [sealedState, setSealedState] = useState<SealedResolved | null>(null)

  useEffect(() => {
    let aborted = false
    postContextPack({ provider: 'claude', intent: intentKey })
      .then((p) => {
        if (!aborted) setPackState({ key: intentKey, pack: p, error: null })
      })
      .catch((err) => {
        if (aborted) return
        setPackState({
          key: intentKey,
          pack: null,
          error: err instanceof OperatorError ? `pack indisponible (${err.status})` : 'pack indisponible',
        })
      })
    return () => {
      aborted = true
    }
  }, [intentKey])

  useEffect(() => {
    if (!sessionId) return
    let aborted = false
    const controller = new AbortController()
    getChatLog(sessionId, controller.signal)
      .then((log) => {
        if (!aborted) setSealedState({ sid: sessionId, map: sealedMap(log.context_pack) })
      })
      .catch(() => {
        if (!aborted) setSealedState({ sid: sessionId, map: null })
      })
    return () => {
      aborted = true
      controller.abort()
    }
  }, [sessionId])

  const packReady = packState !== null && packState.key === intentKey
  const pack = packReady ? packState.pack : null
  const error = packReady ? packState.error : null
  const sealed = sessionId && sealedState?.sid === sessionId ? sealedState.map : null

  return (
    <AdaptiveSurface as="section" kind="context-pack" testId="context-pack-inspector" className="flex flex-col gap-3">
      <div className="rounded-md border border-bd bg-s1 px-4 py-3">
        <h2 className="mb-1 font-sans text-card font-semibold text-tx">
          Préparer le pack — brouillon de transmission
        </h2>
        <p className="font-sans text-body leading-relaxed text-tx2">
          Le pack scellé ci-dessous est la seule chose qui franchit le mur : il se transmet à une
          vraie session agent qui produit gates et preuves. L'Operator restitue ce qui est tracé — il
          ne clôt aucun verdict, il ne grave aucun « valider ».
        </p>
        <div className="adaptive-secondary mt-2 font-mono text-meta text-tx4">
          références hachées · contenu jamais inliné · historique de chat non-autoritaire
        </div>
      </div>

      {error ? (
        <div className="rounded-md border border-bd bg-s0 px-4 py-3 font-mono text-meta text-warn">{error}</div>
      ) : pack === null ? (
        <div className="rounded-md border border-bd bg-s0 px-4 py-3 font-mono text-meta text-tx4">
          scellement du pack…
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {groups(pack).map((g) => (
            <div key={g.label}>
              <div className="mb-1 eyebrow">
                {g.label} <span className="text-tx4">· {g.refs.length}</span>
              </div>
              <div className="flex flex-col gap-1">
                {g.refs.length === 0 ? (
                  <div className="px-2.5 py-1.5 font-mono text-meta text-tx4">—</div>
                ) : (
                  g.refs.map((r) => (
                    <HashRefRow key={r.path} entry={r} drifted={!!sealed && sealed.has(r.path) && sealed.get(r.path) !== r.hash} />
                  ))
                )}
              </div>
            </div>
          ))}
          <div className="font-mono text-meta text-tx4">{pack.notice}</div>
        </div>
      )}
    </AdaptiveSurface>
  )
}
