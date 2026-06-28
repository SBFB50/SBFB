// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D → front rapid-add — the Sessions inspector. Restituted,
// read-only surfaces:
//   - the node JOURNAL + the MUR REFUSAL REGISTER (folds S8/U5): every
//     allowlisted action and every rejection with its reason, from
//     /api/actions/log. A register, never a "retry by forcing".
//   - RESUMABLE claude sessions (terminal.rs list_claude_sessions, scoped to
//     this repo): click "reprendre" to re-attach a `claude --resume` PTY
//     inline (terminalWsUrl(resume)) — the xterm stays code-split.
//   - recorded terminal SESSIONS for replay (folds U6/V9, S7): `.cast`
//     recordings, several openable at once.
//   - the live STEER session log (fold S7): the current chat session messages
//     (incl. the gated intention), expandable. NOT a multi-agent board.
import { lazy, Suspense, useEffect, useState } from 'react'
import {
  getActionLog,
  getChatLog,
  getTerminalSessions,
  OperatorError,
  type ActionLogEntry,
  type ChatLog,
  type ClaudeSession,
  type TerminalCast,
} from '../../api/operator'
import type { TerminalStatus } from '../verify/TerminalXterm'
import { CastReplay } from './CastReplay'

const ResumeTerminal = lazy(() => import('../verify/TerminalXterm'))

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

const MSG_CLAMP = 200

function ChatMessage({ role, content }: { role: string; content: string }) {
  const [open, setOpen] = useState(false)
  const long = content.length > MSG_CLAMP
  const shown = open || !long ? content : `${content.slice(0, MSG_CLAMP)}…`
  return (
    <div className="rounded-sm border border-bd bg-s1 px-2.5 py-1.5">
      <span className="mr-2 font-mono text-[9px] uppercase text-tx4">{role}</span>
      <span className="font-sans text-[11px] text-tx2">{shown}</span>
      {long ? (
        <button
          type="button"
          onClick={() => setOpen((o) => !o)}
          className="ml-1.5 font-mono text-[9.5px] text-info hover:underline"
          data-testid="message-expand"
        >
          {open ? 'voir moins' : 'voir plus'}
        </button>
      ) : null}
    </div>
  )
}

function formatSessionDate(updated: number): string {
  if (!updated) return ''
  const ms = updated > 1e12 ? updated : updated * 1000
  const d = new Date(ms)
  return Number.isNaN(d.getTime()) ? '' : d.toLocaleString('fr-FR')
}

function ResumePanel({ sessions }: { sessions: ClaudeSession[] }) {
  const [resume, setResume] = useState<string | null>(null)
  const [status, setStatus] = useState<TerminalStatus | 'inactif'>('inactif')
  if (sessions.length === 0) {
    return <div className="font-mono text-[10.5px] text-tx4">aucune session claude reprenable pour ce dépôt</div>
  }
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-col gap-1" data-testid="claude-sessions">
        {sessions.map((s) => (
          <div
            key={s.session_id}
            className="flex items-center gap-2 rounded-sm border border-bd bg-s1 px-2.5 py-1.5"
          >
            <span className="shrink-0 rounded-sm border border-bd px-1 font-mono text-[8.5px] uppercase text-tx4">
              claude
            </span>
            <span className="min-w-0 flex-1 truncate font-sans text-[11px] text-tx2">{s.name || s.session_id}</span>
            <span className="shrink-0 font-mono text-[9.5px] text-tx4">{formatSessionDate(s.updated_at)}</span>
            <button
              type="button"
              onClick={() => setResume((cur) => (cur === s.session_id ? null : s.session_id))}
              data-testid="claude-resume"
              className={`shrink-0 rounded-sm border px-2 py-0.5 font-mono text-[9.5px] ${
                resume === s.session_id ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:border-bd2'
              }`}
            >
              {resume === s.session_id ? 'fermer' : 'reprendre'}
            </button>
          </div>
        ))}
      </div>
      {resume ? (
        <div className="flex flex-col overflow-hidden rounded-md border border-bd" data-testid="resume-terminal">
          <div className="flex items-center gap-2 border-b border-bd bg-s1 px-3 py-1 font-mono text-[10px] text-tx3">
            <span className="text-tx2">reprise · {resume.slice(0, 12)}</span>
            <span className="ml-auto text-tx4">{status}</span>
          </div>
          <div className="h-72 min-h-0 bg-s0 p-2">
            <Suspense fallback={<div className="p-3 font-mono text-[11px] text-tx4">chargement du terminal…</div>}>
              <ResumeTerminal resume={resume} onStatus={setStatus} />
            </Suspense>
          </div>
        </div>
      ) : null}
    </div>
  )
}

export function SessionsSurface({ sessionId }: { sessionId: string | null }) {
  const [log, setLog] = useState<ActionLogEntry[] | null>(null)
  const [casts, setCasts] = useState<TerminalCast[] | null>(null)
  const [claudeSessions, setClaudeSessions] = useState<ClaudeSession[] | null>(null)
  const [chatState, setChatState] = useState<{ sid: string; chat: ChatLog | null } | null>(null)
  const [selectedCasts, setSelectedCasts] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    Promise.allSettled([getActionLog(controller.signal), getTerminalSessions(controller.signal)]).then(
      ([logRes, sessRes]) => {
        if (controller.signal.aborted) return
        if (logRes.status === 'fulfilled') setLog(logRes.value)
        else
          setError(
            logRes.reason instanceof OperatorError
              ? `journal indisponible (${logRes.reason.status})`
              : 'journal indisponible',
          )
        if (sessRes.status === 'fulfilled') {
          setCasts(sessRes.value.sessions)
          setClaudeSessions(sessRes.value.claude_sessions)
        } else {
          setCasts([])
          setClaudeSessions([])
        }
      },
    )
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

  const toggleCast = (name: string) =>
    setSelectedCasts((cur) => {
      const next = new Set(cur)
      if (next.has(name)) next.delete(name)
      else next.add(name)
      return next
    })

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

      <section className="mb-5">
        <div className="mb-1 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
          sessions reprenables
        </div>
        {/* Honest provider scope: only the Claude CLI persists resumable
           sessions (~/.claude/sessions, `claude --resume`). Local (Ollama) /
           network intentions run through the STEER chat session below — there
           is no CLI-resume store for them in the backend today. */}
        <p className="mb-1.5 font-mono text-[10px] text-tx4">
          reprise via le CLI Claude (<span className="text-tx3">claude --resume</span>). Les intentions
          local (Ollama) / réseau passent par la session STEER ci-dessous — pas de reprise CLI côté
          backend aujourd'hui.
        </p>
        {claudeSessions === null ? (
          <div className="font-mono text-[10.5px] text-tx4">lecture des sessions…</div>
        ) : (
          <ResumePanel sessions={claudeSessions} />
        )}
      </section>

      {sessionId ? (
        <section className="mb-5">
          <div className="mb-1.5 font-sans text-[8.5px] font-semibold uppercase tracking-[0.14em] text-tx4">
            session STEER en cours{' '}
            <span className="text-tx4">· {chat?.messages.length ?? 0} messages (non-autoritaire)</span>
          </div>
          <div className="flex flex-col gap-1">
            {(chat?.messages ?? []).map((m, i) => (
              <ChatMessage key={i} role={m.role} content={m.content} />
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
                onClick={() => toggleCast(c.name)}
                className={`rounded-sm border px-2 py-1 font-mono text-[9.5px] ${
                  selectedCasts.has(c.name) ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:border-bd2'
                }`}
              >
                {c.name} <span className="text-tx4">· {Math.round(c.size_bytes / 1024)} ko</span>
              </button>
            ))}
          </div>
        )}
        {[...selectedCasts].map((name) => (
          <div key={name} className="mt-2 flex h-72 flex-col overflow-hidden rounded-md border border-bd">
            <div className="flex items-center gap-2 border-b border-bd bg-s1 px-3 py-1 font-mono text-[10px] text-tx3">
              <span className="min-w-0 flex-1 truncate text-tx2">{name}</span>
              <button
                type="button"
                onClick={() => toggleCast(name)}
                className="shrink-0 text-tx4 hover:text-tx2"
              >
                fermer
              </button>
            </div>
            <CastReplay name={name} />
          </div>
        ))}
      </section>
    </div>
  )
}
