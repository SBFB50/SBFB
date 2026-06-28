// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the STEER turn orchestration: a sensitive intention is
// gated at /send (the MUR, no stream opened → 0 spawn); a benign intention
// opens the stream. The backend calls are mocked; the stream uses a real
// ReadableStream through a mocked fetch.
import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../api/operator'
import { useOperator } from './useOperator'

vi.mock('../api/operator', () => ({
  createSession: vi.fn(),
  sendMessage: vi.fn(),
  streamUrl: (id: string) => `/api/chat/${id}/stream`,
  OperatorError: class extends Error {},
}))

const createSession = vi.mocked(api.createSession)
const sendMessage = vi.mocked(api.sendMessage)
const enc = new TextEncoder()

function doneBody(): ReadableStream<Uint8Array> {
  return new ReadableStream<Uint8Array>({
    start(c) {
      c.enqueue(enc.encode(`data: ${JSON.stringify({ type: 'done', result: 'ok' })}\n\n`))
      c.close()
    },
  })
}

function liveBody(): ReadableStream<Uint8Array> {
  return new ReadableStream<Uint8Array>({ start() {} }) // stays open until aborted
}

beforeEach(() => {
  vi.clearAllMocks() // module-mock vi.fn()s accumulate calls otherwise
  createSession.mockResolvedValue({ id: 'sess-1', context_pack: {} })
  globalThis.fetch = vi.fn(async () => ({ ok: true, status: 200, body: doneBody() }) as unknown as Response)
})
afterEach(() => vi.restoreAllMocks())

describe('useOperator', () => {
  it('gates a sensitive intention at /send and never opens the stream (0 spawn)', async () => {
    sendMessage.mockResolvedValue({ ok: false, requires_gate: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('commit and push the branch', 'preflight'))
    await waitFor(() => expect(result.current.turn.gate).not.toBeNull())
    expect(globalThis.fetch).not.toHaveBeenCalled() // the stream was never opened
    expect(result.current.hasTurn).toBe(true)
  })

  it('dismisses the MUR back to the empty composer', async () => {
    sendMessage.mockResolvedValue({ ok: false, requires_gate: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('please commit', 'preflight'))
    await waitFor(() => expect(result.current.turn.gate).not.toBeNull())
    act(() => result.current.dismissGate())
    expect(result.current.turn.gate).toBeNull()
    expect(result.current.hasTurn).toBe(false)
  })

  it('opens the stream for a benign intention and completes the turn', async () => {
    sendMessage.mockResolvedValue({ ok: true, provider: 'claude' })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('prepare the phase plan', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    expect(createSession).toHaveBeenCalledOnce()
    expect(result.current.turn.result).toBe('ok')
  })

  it('reuses the session across turns (one createSession)', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('first intention', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    act(() => result.current.launch('second intention', 'phase-review'))
    await waitFor(() => expect(sendMessage).toHaveBeenCalledTimes(2))
    expect(createSession).toHaveBeenCalledOnce()
  })

  it('relaunch re-streams the same session as a new full-cost turn (no new send)', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('prepare the phase', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    act(() => result.current.relaunch())
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    expect(sendMessage).toHaveBeenCalledOnce() // relaunch does NOT re-send
    expect(globalThis.fetch).toHaveBeenCalledTimes(2) // two stream opens, full cost
  })

  it('interrupt stops listening (aborted), never an error', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    globalThis.fetch = vi.fn(async () => ({ ok: true, status: 200, body: liveBody() }) as unknown as Response)
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('prepare the phase', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('streaming'))
    act(() => result.current.interrupt())
    await waitFor(() => expect(result.current.turn.status).toBe('aborted'))
    expect(result.current.turn.error).toBeNull()
  })

  it('newSession drops the session and returns to the empty composer', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('prepare the phase', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    act(() => result.current.newSession())
    expect(result.current.hasTurn).toBe(false)
    expect(result.current.sessionId).toBeNull()
  })

  it('toggles the focal mode manually (D6)', () => {
    const { result } = renderHook(() => useOperator())
    expect(result.current.mode).toBe('steer')
    act(() => result.current.setMode('verify'))
    expect(result.current.mode).toBe('verify')
  })

  it('opens and closes a secondary inspector surface (Phase D)', () => {
    const { result } = renderHook(() => useOperator())
    expect(result.current.surface).toBeNull()
    act(() => result.current.openSurface('procede'))
    expect(result.current.surface).toBe('procede')
    act(() => result.current.closeSurface())
    expect(result.current.surface).toBeNull()
  })

  it('selecting a focal mode closes any open inspector', () => {
    const { result } = renderHook(() => useOperator())
    act(() => result.current.openSurface('sessions'))
    expect(result.current.surface).toBe('sessions')
    act(() => result.current.setMode('verify'))
    expect(result.current.mode).toBe('verify')
    expect(result.current.surface).toBeNull() // back to the focal scene
  })

  it('preparePack opens the Knowledge inspector (the MUR handoff brouillon)', () => {
    const { result } = renderHook(() => useOperator())
    act(() => result.current.preparePack())
    expect(result.current.surface).toBe('knowledge')
  })

  it('verifyReady (D6 availability) is false with no turn and mid-stream, never on abort', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    globalThis.fetch = vi.fn(async () => ({ ok: true, status: 200, body: liveBody() }) as unknown as Response)
    const { result } = renderHook(() => useOperator())
    expect(result.current.verifyReady).toBe(false) // no turn yet
    act(() => result.current.launch('prepare the phase', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('streaming'))
    expect(result.current.verifyReady).toBe(false) // mid-stream: not a complete turn
    act(() => result.current.interrupt())
    await waitFor(() => expect(result.current.turn.status).toBe('aborted'))
    expect(result.current.verifyReady).toBe(false) // aborted ≠ complete
  })

  it('verifyReady lights once a turn reaches a complete terminal (done)', async () => {
    sendMessage.mockResolvedValue({ ok: true })
    const { result } = renderHook(() => useOperator())
    act(() => result.current.launch('prepare the phase', 'preflight'))
    await waitFor(() => expect(result.current.turn.status).toBe('done'))
    expect(result.current.verifyReady).toBe(true)
  })
})
