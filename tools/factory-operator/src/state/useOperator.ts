// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the STEER orchestration hook. Owns the focal MODE
// (D6: manual STEER⇄VERIFY, never auto), the execution provider attribute,
// the chat session, and one turn's lifecycle on top of `useTokenStream`.
//
// Turn flow (matches the backend contract, operator_server.rs):
//   1. ensure a session — POST /api/chat/session {provider, intent:kind}
//   2. POST /api/chat/{id}/send {message, provider} — appends the user
//      message AND runs the SENSITIVE_ACTIONS gate. `requires_gate` →
//      restitute the MUR and STOP (the stream is never opened → 0 spawn).
//   3. otherwise open GET /api/chat/{id}/stream via useTokenStream.
//
// Relaunch is an explicit, FULL-COST new turn (plan-adaptation #1): the
// last user message persists in the session, so re-opening the stream
// re-runs a complete inference and appends another assistant message — it
// is never an idempotent no-op, and the UI says so.

import { useCallback, useState } from 'react'
import { createSession, sendMessage, streamUrl } from '../api/operator'
import { DEFAULT_PROVIDER, type ExecProvider } from '../catalog/intentions'
import { useTokenStream } from '../lib/useTokenStream'

export type FocalMode = 'steer' | 'verify'

/** The MUR copy shown when /send gates a sensitive intention (the /send
 * response carries no message body). This is the FR restitution of the
 * backend barrier; the SSE-stream path instead surfaces `stream.gate`, the
 * RAW backend `requires_gate` text (gate = sendGate ?? stream.gate). The two
 * are near-mutually-exclusive — the /send gate short-circuits BEFORE the
 * stream is ever opened, so the SSE-gate text is only reachable defensively. */
const GATE_MESSAGE =
  'Cette intention exige une vraie session agent avec des preuves visibles dans le dépôt.'

export interface OperatorTurn {
  /** The user intention currently in flight / shown. */
  message: string | null
  /** The technical prompt kind behind the active intention (folded UI). */
  kind: string | null
  status: ReturnType<typeof useTokenStream>['status']
  text: string
  thinking: string
  result: string | null
  error: string | null
  /** The MUR message when this turn is gated (from /send or the stream). */
  gate: string | null
  /** True while creating the session / sending (before the stream opens). */
  busy: boolean
  /** A launch failure (network / backend), distinct from a streamed error. */
  launchError: string | null
}

export interface Operator {
  mode: FocalMode
  setMode: (mode: FocalMode) => void
  provider: ExecProvider
  setProvider: (provider: ExecProvider) => void
  sessionId: string | null
  turn: OperatorTurn
  /** True once a turn exists — the scene leaves the empty/discovery state. */
  hasTurn: boolean
  launch: (text: string, kind: string) => void
  /** S5: re-run the last user message as a new full-cost turn. */
  relaunch: () => void
  /** S6a: stop listening (honest abort) — the server turn may continue. */
  interrupt: () => void
  /** Dismiss the MUR and return to the composer (keeps the session). */
  dismissGate: () => void
  /** Drop the session and return to the empty composer. */
  newSession: () => void
}

export function useOperator(): Operator {
  const [mode, setMode] = useState<FocalMode>('steer')
  const [provider, setProvider] = useState<ExecProvider>(DEFAULT_PROVIDER)
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const [kind, setKind] = useState<string | null>(null)
  const [sendGate, setSendGate] = useState<string | null>(null)
  const [launchError, setLaunchError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  const stream = useTokenStream()
  const { start: streamStart, stop: streamStop, reset: streamReset } = stream

  const launch = useCallback(
    (text: string, intentionKind: string) => {
      const trimmed = text.trim()
      if (!trimmed || busy) return
      setBusy(true)
      setLaunchError(null)
      setSendGate(null)
      streamReset()
      setMessage(trimmed)
      setKind(intentionKind)

      void (async () => {
        try {
          let id = sessionId
          if (!id) {
            const session = await createSession({ provider, intent: intentionKind })
            id = session.id
            setSessionId(id)
          }
          const result = await sendMessage(id, { message: trimmed, provider })
          if (result.requires_gate) {
            // MUR restitution — never open the stream, never a force path.
            setSendGate(GATE_MESSAGE)
            return
          }
          streamStart(streamUrl(id))
        } catch (err) {
          setLaunchError(err instanceof Error ? err.message : 'échec du lancement')
        } finally {
          setBusy(false)
        }
      })()
    },
    [busy, sessionId, provider, streamStart, streamReset],
  )

  const relaunch = useCallback(() => {
    if (!sessionId || message === null || busy) return
    setSendGate(null)
    streamReset()
    streamStart(streamUrl(sessionId))
  }, [sessionId, message, busy, streamStart, streamReset])

  const interrupt = useCallback(() => {
    streamStop()
  }, [streamStop])

  const dismissGate = useCallback(() => {
    setSendGate(null)
    streamReset()
    setMessage(null)
    setKind(null)
  }, [streamReset])

  const newSession = useCallback(() => {
    streamReset()
    setSessionId(null)
    setMessage(null)
    setKind(null)
    setSendGate(null)
    setLaunchError(null)
  }, [streamReset])

  const gate = sendGate ?? stream.gate

  return {
    mode,
    setMode,
    provider,
    setProvider,
    sessionId,
    hasTurn: message !== null,
    turn: {
      message,
      kind,
      status: stream.status,
      text: stream.text,
      thinking: stream.thinking,
      result: stream.result,
      error: stream.error,
      gate,
      busy,
      launchError,
    },
    launch,
    relaunch,
    interrupt,
    dismissGate,
    newSession,
  }
}
