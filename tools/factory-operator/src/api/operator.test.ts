// SPDX-License-Identifier: AGPL-3.0-or-later
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createSession,
  getContext,
  getPrompt,
  getProviders,
  OperatorError,
  sendMessage,
  streamUrl,
} from './operator'

function ok(data: unknown, status = 200) {
  return { ok: status >= 200 && status < 300, status, json: async () => data } as unknown as Response
}

afterEach(() => vi.restoreAllMocks())

describe('operator API client', () => {
  it('parses a 200 and throws OperatorError on a non-2xx status', async () => {
    globalThis.fetch = vi.fn(async () => ok({ branch: 'master', head: 'h', dirty_files: [], staged_files: [] }))
    await expect(getContext()).resolves.toMatchObject({ branch: 'master' })
    globalThis.fetch = vi.fn(async () => ok({}, 500))
    await expect(getContext()).rejects.toBeInstanceOf(OperatorError)
  })

  it('sends the same-origin cookie automatically (never omit, never a JS token)', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => ok({ dirty_files: [], staged_files: [], branch: 'm', head: 'h' }))
    globalThis.fetch = fetchMock
    await getContext()
    const init = fetchMock.mock.calls[0][1]!
    expect(init.credentials).toBe('same-origin')
    // No Authorization / X-SBFB-Token header is set in JS (HttpOnly cookie).
    expect(JSON.stringify(init.headers)).not.toMatch(/token|authorization/i)
  })

  it('getProviders parses the diagnostic provider list', async () => {
    globalThis.fetch = vi.fn(async () => ok({ providers: ['claude', 'codex', 'local'] }))
    await expect(getProviders()).resolves.toEqual({ providers: ['claude', 'codex', 'local'] })
  })

  it('getPrompt encodes the kind and carries the provider query', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => ok({ kind: 'phase-review', provider: 'claude', content: 'X' }))
    globalThis.fetch = fetchMock
    await getPrompt('phase-review', 'claude')
    expect(String(fetchMock.mock.calls[0][0])).toBe('/api/prompt/phase-review?provider=claude')
  })

  it('createSession POSTs the provider + intent', async () => {
    const fetchMock = vi.fn<typeof fetch>(async () => ok({ id: 'sess-1', context_pack: {} }))
    globalThis.fetch = fetchMock
    await expect(createSession({ provider: 'claude', intent: 'preflight' })).resolves.toMatchObject({
      id: 'sess-1',
    })
    const init = fetchMock.mock.calls[0][1]!
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({ provider: 'claude', intent: 'preflight' })
  })

  it('sendMessage returns requires_gate as data (200), not an error', async () => {
    globalThis.fetch = vi.fn(async () => ok({ ok: false, requires_gate: true }))
    await expect(sendMessage('sess-1', { message: 'commit', provider: 'claude' })).resolves.toEqual({
      ok: false,
      requires_gate: true,
    })
  })

  it('streamUrl builds the bodyless SSE path', () => {
    expect(streamUrl('chat-1')).toBe('/api/chat/chat-1/stream')
  })

  it('OperatorError carries the status and path', () => {
    const err = new OperatorError(401, '/api/x')
    expect(err.status).toBe(401)
    expect(err.path).toBe('/api/x')
    expect(err).toBeInstanceOf(Error)
  })
})
