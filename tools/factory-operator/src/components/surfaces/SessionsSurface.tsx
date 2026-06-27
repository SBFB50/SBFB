// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the Sessions inspector. Three restituted surfaces, all
// read-only:
//   - the node JOURNAL + the MUR REFUSAL REGISTER (folds S8/U5): every
//     allowlisted action and every rejection (PASS / traversal / non-allowlist)
//     with its reason, from /api/actions/log. A register, never a "retry by
//     forcing" — the refusals are evidence, not a retry path.
//   - recorded terminal SESSIONS for replay (folds U6/V9, S7): the `.cast`
//     recordings, click to replay the RAW output in a write-only xterm.
//   - the live STEER session log (fold S7): the current chat session messages
//     (incl. the gated intention), /chat/{id}/log. NOT a multi-agent board
//     (cut); just the single in-flight session. Disk persistence is S81.
import { useEffect, useState } from 'react'
import {
  getActionLog,
  getChatLog,
  getTerminalSessions,
  OperatorError,
  type ActionLogEntry,
  type ChatLog,
  type TerminalCast,
} from '../../api/operator'
import { CastReplay } from './CastReplay'

function isRefusal(result: string): boolean {
  return result.startsWith('rejected')
}

function Journal({ entries }: { entries: ActionLogEntry[] }) {
  if (entries.length === 0) return <div className="font-mono text-[10.5px] text-tx4">aucune action enregistrée</div>
  return (
    <ul className="flex flex-col gap-0.5" data-testid="action-journal">
      {entries.map((e, i) => (
        <li key={i} className="flex items-baseline gap-2 font-mono text-[10px]">
          <span className="text-tx4 tabular-nums">{e.timestamp.slice(11, 19)}</span>
          <span className="text-tx2">{e.action}</span>
          <span className={isRefusal(e.result) ? 'text-warn' : 'text-tx4'}>
            {isRefusal(e.result) ? `⛔ ${e.result}` : e.result}
          </span>
        </li>
      ))}
    </ul>
  )
}

export function SessionsSurface({ sessionId }: { sessionId: string | null }) {
  const [log, setLog] = useState<ActionLogEntry[] | null>(null)
  const [casts, setCasts] = useState<TerminalCast[] | null>(null)
  const [chatState, setChatState] = useState<{ sid: string; chat: ChatLog | null } | null>(null)
  const [selected, setSelected] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    Promise.allSettled([
      getActionLog(controller.signal),
      getTerminalSessions(controller.signal),
    ]).then(([logRes, sessRes]) => {
      if (controller.signal.aborted) return
      if (logRes.status === 'fulfilled') setLog(logRes.value)
      else setError(logRes.reason instanceof OperatorError ? `journal indisponible (${logRes.reason.status})` : 'journal indisponible')
      if (sessRes.status === 'fulfilled') setCasts(sessRes.value.sessions)
      else setCasts([])
    })
    return () => controller.abort()
  }, [])

  useEffect(() => {
    if (!sessionId) return
    const controller = new AbortController()
    getChatLog(sessionId, controller.signal)
      .then((c) => {
        if (!controller.signal.aborted) setChatState({ sid: sessionId, chat: c })
      })
      .catch(() => {
        if (!controller.signal.aborted) setChatState({ sid: sessionId, chat: null })
      })
    return () => controller.abort()
  }, [sessionId])

  const chat = sessionId && chatState?.sid === sessionId ? chatState.chat : null

  return (
    <div data-testid="sessions-surface" className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <section className="mb-5">
        <div className="mb-1.5 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
          journal de bord · registre des refus du mur
        </div>
        {error ? (
          <div className="font-mono text-[10.5px] text-warn">{error}</div>
        ) : log === null ? (
          <div className="font-mono text-[10.5px] text-tx4">lecture du journal…</div>
        ) : (
          <Journal entries={log} />
        )}
      </section>

      {sessionId ? (
        <section className="mb-5">
          <div className="mb-1.5 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
            session STEER en cours <span className="text-tx4">· {chat?.messages.length ?? 0} messages (non-autoritaire)</span>
          </div>
          <div className="flex flex-col gap-1">
            {(chat?.messages ?? []).map((m, i) => (
              <div key={i} className="rounded-sm border border-bd bg-s1 px-2.5 py-1.5">
                <span className="mr-2 font-mono text-[9px] uppercase text-tx4">{m.role}</span>
                <span className="font-sans text-[11px] text-tx2">{m.content.slice(0, 200)}</span>
              </div>
            ))}
          </div>
        </section>
      ) : null}

      <section className="min-h-0 flex-1">
        <div className="mb-1.5 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
          enregistrements terminal · rejeu
        </div>
        {casts === null ? (
          <div className="font-mono text-[10.5px] text-tx4">lecture des enregistrements…</div>
        ) : casts.length === 0 ? (
          <div className="font-mono text-[10.5px] text-tx4">aucun enregistrement</div>
        ) : (
          <div className="flex flex-wrap gap-1.5">
            {casts.map((c) => (
              <button
                key={c.name}
                type="button"
                onClick={() => setSelected((cur) => (cur === c.name ? null : c.name))}
                className={`rounded-sm border px-2 py-1 font-mono text-[9.5px] ${
                  selected === c.name ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:border-bd2'
                }`}
              >
                {c.name} <span className="text-tx4">· {Math.round(c.size_bytes / 1024)} ko</span>
              </button>
            ))}
          </div>
        )}
        {selected ? (
          <div className="mt-2 flex h-72 flex-col overflow-hidden rounded-md border border-bd">
            <CastReplay name={selected} />
          </div>
        ) : null}
      </section>
    </div>
  )
}
