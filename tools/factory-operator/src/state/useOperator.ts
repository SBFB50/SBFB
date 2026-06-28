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
import type { SecondarySurface } from '../catalog/surfaces'
import { altitudeShift } from '../lib/altitudeShift'
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
  /** The open secondary inspector (procédé / sessions / knowledge), or null
   * when the focal scene (STEER / VERIFY) is shown. */
  surface: SecondarySurface | null
  openSurface: (surface: SecondarySurface) => void
  closeSurface: () => void
  /** The MUR forward action: prepare the sealed handoff pack for a real agent
   * session (opens the Knowledge inspector's context-pack — the brouillon that
   * refuses PASS). The ONLY way past the wall, and it is not "execute". */
  preparePack: () => void
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
  const [mode, setModeState] = useState<FocalMode>('steer')
  const [surface, setSurface] = useState<SecondarySurface | null>(null)
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

  // Selecting a focal MODE returns to that scene, closing any open inspector.
  // The bascule shifts altitude via a native View Transition (signature 4):
  // `altitudeShift` flushSync-applies the state inside `startViewTransition` so
  // the new focal pane is captured, and degrades to an instant apply under
  // prefers-reduced-motion (the VT is not auto-gated by MotionConfig).
  const setMode = useCallback((m: FocalMode) => {
    altitudeShift(() => {
      setModeState(m)
      setSurface(null)
    })
  }, [])
  const openSurface = useCallback((s: SecondarySurface) => setSurface(s), [])
  const closeSurface = useCallback(() => setSurface(null), [])
  // "Préparer le pack": the MUR's only forward affordance — it hands off to a
  // real agent session via the sealed context-pack, never executes the action.
  const preparePack = useCallback(() => setSurface('knowledge'), [])

  const gate = sendGate ?? stream.gate

  return {
    mode,
    setMode,
    surface,
    openSurface,
    closeSurface,
    preparePack,
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
