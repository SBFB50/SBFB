// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — typed client for the Operator backend (crate
// `sbfb-factory`, 0 daemon route). Every call is same-origin and relative:
// in prod the SPA is served by the Operator's ServeDir and authenticates
// via the HttpOnly `sbfb_operator` cookie (Phase A `a5ace8d`); in dev the
// Vite proxy injects the bearer header server-to-server. The browser
// attaches the cookie automatically — we set `credentials: 'same-origin'`
// (the default, made explicit) and NEVER `'omit'` (that would strip the
// cookie → 401) and NEVER read a token in JS (HttpOnly, unreadable).
// Preflight S3 / plan-adaptation #6.

/** A non-2xx Operator response. `status` is the HTTP code (e.g. 401, 404). */
export class OperatorError extends Error {
  readonly status: number
  readonly path: string
  constructor(status: number, path: string) {
    super(`operator ${path} → ${status}`)
    this.name = 'OperatorError'
    this.status = status
    this.path = path
  }
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(path, {
    credentials: 'same-origin',
    headers: { accept: 'application/json' },
    signal,
  })
  if (!res.ok) throw new OperatorError(res.status, path)
  return (await res.json()) as T
}

async function postJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(path, {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new OperatorError(res.status, path)
  return (await res.json()) as T
}

// --- response shapes (mirror operator_server.rs / process.rs) ---

/** `GET /api/context` — carries the working-tree counts the rail needs. */
export interface RepoContext {
  branch: string
  head: string
  sprint?: number
  phase?: string
  dirty_files: string[]
  staged_files: string[]
}

/** `POST /api/chat/session` → `{ id, context_pack }`. */
export interface SessionCreated {
  id: string
  context_pack: unknown
}

/**
 * `POST /api/chat/{id}/send` → `{ ok }` or `{ ok: false, requires_gate: true }`.
 * The send appends the user message and runs the SENSITIVE_ACTIONS keyword
 * gate BEFORE any spawn; a sensitive message returns `requires_gate` and the
 * front restitutes the MUR without ever opening the stream (0 spawn).
 */
export interface SendResult {
  ok: boolean
  requires_gate?: boolean
  provider?: string
}

/** `GET /api/prompt/{kind}` — the real assembled prompt (inspector). */
export interface PromptInspect {
  kind: string
  provider: string
  content: string
}

// --- calls ---

export function getContext(signal?: AbortSignal): Promise<RepoContext> {
  return getJson<RepoContext>('/api/context', signal)
}

/**
 * S4 diagnostic probe: the backend's prompt-adaptation provider set. Used as a
 * reachability signal for the rail, NOT as the chat execution axis — the chat
 * routes on `ExecutionTarget::from_provider` ({claude, local, network}), a
 * distinct axis from this list ({claude, codex, gpt, local, human}).
 */
export function getProviders(signal?: AbortSignal): Promise<{ providers: string[] }> {
  return getJson<{ providers: string[] }>('/api/providers', signal)
}

export function getPrompt(kind: string, provider: string, signal?: AbortSignal): Promise<PromptInspect> {
  const qs = new URLSearchParams({ provider }).toString()
  return getJson<PromptInspect>(`/api/prompt/${encodeURIComponent(kind)}?${qs}`, signal)
}

export function createSession(req: {
  provider: string
  intent: string
}): Promise<SessionCreated> {
  return postJson<SessionCreated>('/api/chat/session', req)
}

export function sendMessage(
  id: string,
  req: { message: string; provider: string; model?: string },
): Promise<SendResult> {
  return postJson<SendResult>(`/api/chat/${encodeURIComponent(id)}/send`, req)
}

/** The bodyless SSE GET. Auth rides the same-origin cookie automatically. */
export function streamUrl(id: string): string {
  return `/api/chat/${encodeURIComponent(id)}/stream`
}
