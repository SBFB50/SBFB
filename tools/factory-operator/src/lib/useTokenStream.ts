// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — `useTokenStream`: the single, tested SSE primitive
// for the Operator transcript. It encapsulates the whole transport so the
// preflight footguns are handled in ONE place (plan-adaptation #3-#8):
//
//   - fetch() + response.body.getReader() + AbortController — NEVER
//     EventSource (it auto-reconnects → re-replays the last turn, busting
//     PO-14; and it cannot carry auth). credentials default to same-origin
//     so the HttpOnly cookie rides automatically; we never set 'omit'.
//   - Done-unique (PO-14): latch the FIRST terminal {done|error|
//     requires_gate}, ignore everything after it, close without reopening.
//   - Network arm: the Done carries the full `result` with ZERO deltas
//     (provider_router.rs:69-71); the accumulator shows `result` regardless.
//   - Honest abort (S6a): stop() aborts the fetch AND cancels the reader
//     from the abort listener (releaseLock alone never signals the
//     disconnect — elysia#1768); an aborted run is a CLEAN stop
//     ('arrête d'écouter'), never an 'error'.
//   - No auto-reconnect: a reconnect would silently re-spawn a server turn
//     (each GET /stream re-runs a full inference, operator_server.rs:1156).
//   - start() supersedes: it bumps a run id and aborts any in-flight stream
//     so a double invocation yields ONE live stream / ONE Done; only the
//     CURRENT run is allowed to write state.

import { useCallback, useEffect, useReducer, useRef } from 'react'
import { createSseDecoder } from './sseFrames'
import { isTerminal, parseChunk, type StreamChunk } from './streamChunk'

export type StreamStatus = 'idle' | 'streaming' | 'done' | 'aborted' | 'error' | 'gate' | 'ended'

export interface StreamState {
  status: StreamStatus
  /** Accumulated `delta` text — the visible assistant output. */
  text: string
  /** Accumulated `thinking` text (reasoning tokens). */
  thinking: string
  /** `result` captured from the first Done (also the Network arm's payload). */
  result: string | null
  /** A real error message (network failure / error chunk) — never an abort. */
  error: string | null
  /** A `requires_gate` message — the MUR restitution, never a retry path. */
  gate: string | null
}

const INITIAL: StreamState = {
  status: 'idle',
  text: '',
  thinking: '',
  result: null,
  error: null,
  gate: null,
}

type Action =
  | { kind: 'start' }
  | { kind: 'chunk'; chunk: StreamChunk }
  | { kind: 'aborted' }
  | { kind: 'ended' }
  | { kind: 'failed'; message: string }
  | { kind: 'reset' }

function applyChunk(state: StreamState, c: StreamChunk): StreamState {
  switch (c.type) {
    case 'delta':
      return { ...state, text: state.text + c.text }
    case 'thinking':
      return { ...state, thinking: state.thinking + c.text }
    case 'debug':
      return state // informative only; not surfaced as transcript text
    case 'done':
      return { ...state, status: 'done', result: c.result }
    case 'error':
      return { ...state, status: 'error', error: c.message }
    case 'requires_gate':
      return { ...state, status: 'gate', gate: c.message }
  }
}

function reducer(state: StreamState, action: Action): StreamState {
  switch (action.kind) {
    case 'start':
      return { ...INITIAL, status: 'streaming' }
    case 'reset':
      return INITIAL
    case 'aborted':
      // Honest stop: we stopped listening; the server turn may continue.
      return state.status === 'streaming' ? { ...state, status: 'aborted' } : state
    case 'ended':
      // The stream closed with no terminal event (no Done/Error/gate) — e.g.
      // the Claude arm exited 0 with no result line. Honest: closed, not done.
      return state.status === 'streaming' ? { ...state, status: 'ended' } : state
    case 'failed':
      return state.status === 'streaming' ? { ...state, status: 'error', error: action.message } : state
    case 'chunk':
      // Latch: once terminal, ignore every later event (post-Done Debug/Error).
      return state.status === 'streaming' ? applyChunk(state, action.chunk) : state
  }
}

export interface UseTokenStream extends StreamState {
  /** Begin streaming `url`, aborting any in-flight stream first. */
  start: (url: string) => void
  /** Stop listening (honest abort): status → 'aborted', never 'error'. */
  stop: () => void
  /** Clear back to idle. */
  reset: () => void
}

export function useTokenStream(): UseTokenStream {
  const [state, dispatch] = useReducer(reducer, INITIAL)
  const controllerRef = useRef<AbortController | null>(null)
  const runIdRef = useRef(0)

  const start = useCallback((url: string) => {
    // Bump the run id FIRST (this run becomes current), then supersede the
    // previous loop — its dispatches are now ignored (runId mismatch).
    const myRun = ++runIdRef.current
    controllerRef.current?.abort()
    const controller = new AbortController()
    controllerRef.current = controller
    const { signal } = controller
    const isCurrent = () => runIdRef.current === myRun
    dispatch({ kind: 'start' })

    void (async () => {
      let reader: ReadableStreamDefaultReader<Uint8Array> | null = null
      const onAbort = () => {
        // S6a: cancel the reader from the abort listener so the server sees
        // the disconnect (releaseLock alone does not — elysia#1768).
        reader?.cancel().catch(() => {})
      }
      let latched = false
      try {
        const res = await fetch(url, {
          credentials: 'same-origin',
          headers: { accept: 'text/event-stream' },
          signal,
        })
        if (!res.ok || !res.body) throw new Error(`stream ${url} → ${res.status}`)
        reader = res.body.getReader()
        signal.addEventListener('abort', onAbort, { once: true })
        const decoder = createSseDecoder()
        const textDecoder = new TextDecoder()

        const handle = (payload: string): boolean => {
          const chunk = parseChunk(payload)
          if (!chunk) return false
          if (isCurrent()) dispatch({ kind: 'chunk', chunk })
          return isTerminal(chunk)
        }

        read: while (true) {
          const { value, done } = await reader.read()
          if (done) break
          for (const payload of decoder.push(textDecoder.decode(value, { stream: true }))) {
            if (handle(payload)) {
              latched = true
              break read
            }
          }
        }
        if (!latched) {
          // Flush the TextDecoder (any buffered multibyte tail) then any
          // trailing frame the server closed without a blank line.
          const tail = textDecoder.decode()
          const trailing = tail ? [...decoder.push(tail), ...decoder.end()] : decoder.end()
          for (const payload of trailing) {
            if (handle(payload)) {
              latched = true
              break
            }
          }
        }
        // The read loop ended with no terminal event. Either we were aborted
        // (honest stop) or the stream simply closed with no Done/Error/gate
        // (e.g. the Claude arm exited 0 with only a Debug 'exit'). Never leave
        // the UI stuck on 'streaming' with a dead Interrupt button.
        if (!latched && isCurrent()) {
          dispatch({ kind: signal.aborted ? 'aborted' : 'ended' })
        }
      } catch (err) {
        if (!isCurrent()) return
        if (signal.aborted || (err instanceof DOMException && err.name === 'AbortError')) {
          dispatch({ kind: 'aborted' })
        } else {
          dispatch({ kind: 'failed', message: err instanceof Error ? err.message : 'stream error' })
        }
      } finally {
        signal.removeEventListener('abort', onAbort)
        // Cancel (not just releaseLock) so a terminal/EOF deterministically
        // closes the body: a server that kept streaming past the terminal would
        // otherwise leave the connection open (Codex P3). Idempotent with the
        // abort-listener cancel and a no-op on an already-closed stream.
        reader?.cancel().catch(() => {})
        if (controllerRef.current === controller) controllerRef.current = null
      }
    })()
  }, [])

  const stop = useCallback(() => {
    // Keep the run id: the current loop stays current and transitions to
    // 'aborted'. Only abort — do not null the ref before the loop reacts.
    controllerRef.current?.abort()
  }, [])

  const reset = useCallback(() => {
    runIdRef.current++ // invalidate the current run: it dispatches nothing more
    controllerRef.current?.abort()
    controllerRef.current = null
    dispatch({ kind: 'reset' })
  }, [])

  // Abort any live stream on unmount (no dangling fetch / server-turn listener).
  useEffect(
    () => () => {
      runIdRef.current++
      controllerRef.current?.abort()
      controllerRef.current = null
    },
    [],
  )

  return { ...state, start, stop, reset }
}
