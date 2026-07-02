// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the PO-14 re-coverage (the intention the jettisoned
// `executionChat.test.ts` carried): one open, one Done, honest abort. Uses a
// real ReadableStream as the fetch body; the fetch is a lightweight mock
// (the hook only reads res.ok / res.status / res.body).
import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useTokenStream } from './useTokenStream'

const enc = new TextEncoder()
function frame(obj: unknown): string {
  return `data: ${JSON.stringify(obj)}\n\n`
}

/** A stream pre-filled with `chunks` then closed — completes on first read. */
function closedBody(chunks: string[]): ReadableStream<Uint8Array> {
  return new ReadableStream<Uint8Array>({
    start(c) {
      for (const ch of chunks) c.enqueue(enc.encode(ch))
      c.close()
    },
  })
}

/** A stream that stays open until `push`/`close` are called by the test. */
function liveBody(): {
  body: ReadableStream<Uint8Array>
  push: (s: string) => void
  close: () => void
} {
  let ctl!: ReadableStreamDefaultController<Uint8Array>
  const body = new ReadableStream<Uint8Array>({ start(c) { ctl = c } })
  return { body, push: (s) => ctl.enqueue(enc.encode(s)), close: () => ctl.close() }
}

function mockFetch(body: ReadableStream<Uint8Array>, ok = true, status = 200) {
  globalThis.fetch = vi.fn(async () => ({ ok, status, body }) as unknown as Response)
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('useTokenStream', () => {
  it('accumulates deltas and captures the result on Done', async () => {
    mockFetch(
      closedBody([
        frame({ type: 'delta', text: 'Hel' }),
        frame({ type: 'delta', text: 'lo' }),
        frame({ type: 'done', result: 'Hello', cost_usd: 0.1, duration_ms: 9 }),
      ]),
    )
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('done'))
    expect(result.current.text).toBe('Hello')
    expect(result.current.result).toBe('Hello')
    expect(result.current.error).toBeNull()
  })

  it('shows the Done result on the Network arm (zero deltas, PO-14)', async () => {
    mockFetch(closedBody([frame({ type: 'done', result: 'network-result' })]))
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('done'))
    expect(result.current.result).toBe('network-result')
    expect(result.current.text).toBe('')
  })

  it('latches the first terminal and ignores post-Done events (Claude arm)', async () => {
    mockFetch(
      closedBody([
        frame({ type: 'delta', text: 'x' }),
        frame({ type: 'done', result: 'x' }),
        frame({ type: 'debug', label: 'exit', content: '0' }),
        frame({ type: 'error', message: 'should be ignored' }),
      ]),
    )
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('done'))
    expect(result.current.status).toBe('done') // not flipped to 'error'
    expect(result.current.error).toBeNull()
  })

  it('restitutes requires_gate as a terminal gate (the MUR signal)', async () => {
    mockFetch(closedBody([frame({ type: 'requires_gate', message: 'gated' })]))
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('gate'))
    expect(result.current.gate).toBe('gated')
  })

  it('treats stop() as an honest abort, never an error', async () => {
    const live = liveBody()
    mockFetch(live.body)
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    act(() => live.push(frame({ type: 'delta', text: 'partial' })))
    await waitFor(() => expect(result.current.text).toBe('partial'))
    act(() => result.current.stop())
    await waitFor(() => expect(result.current.status).toBe('aborted'))
    expect(result.current.error).toBeNull()
    expect(result.current.text).toBe('partial') // partial output preserved
  })

  it('supersedes an in-flight stream when start() is called again (one live stream)', async () => {
    const live = liveBody()
    let call = 0
    globalThis.fetch = vi.fn(async (input: RequestInfo | URL) => {
      call += 1
      const url = String(input)
      if (url.endsWith('/first')) return { ok: true, status: 200, body: live.body } as unknown as Response
      return {
        ok: true,
        status: 200,
        body: closedBody([frame({ type: 'done', result: 'second' })]),
      } as unknown as Response
    })
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/first'))
    await waitFor(() => expect(result.current.status).toBe('streaming'))
    act(() => result.current.start('/api/chat/x/second'))
    await waitFor(() => expect(result.current.status).toBe('done'))
    expect(result.current.result).toBe('second')
    expect(call).toBe(2)
  })

  it('reports a real transport failure as an error', async () => {
    mockFetch(closedBody([]), false, 502)
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('error'))
    expect(result.current.error).toContain('502')
  })

  it('surfaces an error chunk that is the FIRST terminal (not after a Done)', async () => {
    mockFetch(
      closedBody([
        frame({ type: 'delta', text: 'partial' }),
        frame({ type: 'error', message: 'agent failed' }),
      ]),
    )
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('error'))
    expect(result.current.error).toBe('agent failed')
    expect(result.current.text).toBe('partial') // partial output kept
  })

  it('never re-opens the transport after a terminal — one fetch total (0 auto-reconnect)', async () => {
    // The 0-auto-reconnect half of PO-14 (the EventSource replay footgun the
    // fetch+reader transport exists to avoid): after the Done terminal, no
    // code path may re-dial. Real timers on purpose — the hook schedules NO
    // timer, so a buggy reconnect would surface as a second fetch call in
    // the microtask chain flushed below.
    mockFetch(closedBody([frame({ type: 'done', result: 'once' })]))
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('done'))
    await act(async () => {}) // flush pending microtasks after the terminal
    expect(globalThis.fetch).toHaveBeenCalledTimes(1)
  })

  it("ends honestly ('ended') when the stream closes with no terminal event", async () => {
    // The Claude arm can exit 0 with only a Debug 'exit' — no Done/Error/gate.
    mockFetch(
      closedBody([
        frame({ type: 'delta', text: 'wrote some' }),
        frame({ type: 'debug', label: 'exit', content: '0' }),
      ]),
    )
    const { result } = renderHook(() => useTokenStream())
    act(() => result.current.start('/api/chat/x/stream'))
    await waitFor(() => expect(result.current.status).toBe('ended'))
    expect(result.current.error).toBeNull() // 'ended' is not an error
    expect(result.current.text).toBe('wrote some') // partial output preserved
  })
})
