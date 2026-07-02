// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect, type APIRequestContext } from '@playwright/test'
import {
  FIXTURE_DAEMON_PORT,
  FIXTURE_NETWORK_RESULT,
  FIXTURE_OLLAMA_TEXT,
  OPERATOR_TEST_TOKEN,
} from './fixtures'

// Sprint 80 Phase C — T1 sub-tests (2) composeur → session and (4) the MUR
// gates a sensitive intention WITHOUT execution. Sprint 80 Phase I — T1
// sub-test (3): the full-stack deterministic SSE. All run against the REAL
// Operator server (serve-operator.mjs); (3) additionally routes the REAL
// dispatch (handle_chat_stream → from_provider → run) at the upstream
// fixture daemon (serve-fixture-daemon.mjs) via SBFB_OLLAMA_ENDPOINT /
// SBFB_DAEMON_ENDPOINT — zero prod code, zero mocked browser fetch.
//
// PO-14 oracles for (3): the session log grows by EXACTLY one assistant
// message (operator_server.rs pushes one per Done crossing the SSE map),
// and the fixture's /__calls counters grow by EXACTLY one upstream call
// for the turn (delta-based — the counters are cumulative across the
// suite, so each test snapshots them before launching).

test.beforeEach(async ({ page }) => {
  // Bootstrap: mint the HttpOnly cookie, then load the shell at /.
  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByTestId('operator-rail')).toBeVisible()
})

test('sub-test (2): composing a benign intention creates a session', async ({ page }) => {
  const sessionCreated = page.waitForResponse(
    (r) => r.url().includes('/api/chat/session') && r.request().method() === 'POST',
  )

  // Hermeticity (Phase I): route this benign turn at the fixture too — the
  // default provider is `claude`, whose arm spawns the REAL `claude` CLI on
  // a dev machine (llm_bridge spawn_agent_stream) once the stream opens.
  // A hermetic T1 never spawns a real agent.
  await page.getByTestId('provider-select').selectOption('local')
  await page.getByTestId('composer-input').fill('prépare le plan de la phase')
  await page.getByTestId('composer-launch').click()

  const resp = await sessionCreated
  expect(resp.status(), 'POST /api/chat/session is 200').toBe(200)
  const body = (await resp.json()) as { id?: string }
  expect(body.id, 'the session has an id').toBeTruthy()

  // The scene left the empty state for the observable atelier.
  await expect(page.getByTestId('atelier')).toBeVisible()
})

/** Auth header for API-driven wire sessions (bearer root of trust). */
const AUTH = { 'x-sbfb-token': OPERATOR_TEST_TOKEN }

/**
 * Wire-level oracle (review P2-1/P3-2): run one full turn on a DEDICATED
 * API session and return the COMPLETE raw SSE transcript. The
 * request-context GET buffers the body to EOF — unlike the CDP body of a
 * page-consumed stream (`response.text()` throws "No data found" on a
 * fetch-consumed SSE). One GET only: each /stream call re-runs the turn.
 */
async function wireTranscript(
  request: APIRequestContext,
  provider: 'local' | 'network',
  message: string,
): Promise<string> {
  const session = await request.post('/api/chat/session', {
    headers: AUTH,
    data: { provider, intent: 't1-wire' },
  })
  expect(session.status(), 'wire session created').toBe(200)
  const { id } = (await session.json()) as { id: string }
  const sent = await request.post(`/api/chat/${id}/send`, {
    headers: AUTH,
    data: { message, provider },
  })
  expect(sent.status(), 'wire message sent').toBe(200)
  const stream = await request.get(`/api/chat/${id}/stream`, { headers: AUTH })
  expect(stream.status(), 'wire stream opened').toBe(200)
  return stream.text()
}

test('sub-test (3a): full-stack SSE — the local arm streams tokens then ONE Done', async ({
  page,
  request,
}) => {
  // Wire-level PO-14 first: the raw SSE transcript carries the two
  // streamed deltas and exactly ONE Done frame — the most direct oracle,
  // no UI indirection (rendering FIXTURE_OLLAMA_TEXT alone would be
  // satisfied by the Done payload, review P2-1).
  const frames = await wireTranscript(request, 'local', 'écris une salutation brève')
  expect(frames.match(/"type":"delta"/g) ?? [], 'two streamed delta frames').toHaveLength(2)
  expect(frames.match(/"type":"done"/g) ?? [], 'exactly ONE Done frame (PO-14)').toHaveLength(1)

  // Counters snapshot AFTER the wire session — the UI turn below must add
  // exactly ONE more upstream generate call.
  const before = (await (
    await request.get(`http://127.0.0.1:${FIXTURE_DAEMON_PORT}/__calls`)
  ).json()) as { generate: number }

  const sessionCreated = page.waitForResponse(
    (r) => r.url().includes('/api/chat/session') && r.request().method() === 'POST',
  )

  // Route the turn at the fixture Ollama (NDJSON delta+delta+done) through
  // the REAL Rust dispatch. The benign prompt avoids the MUR substrings
  // (shell/commit/push/pass — SENSITIVE_ACTIONS matches substrings).
  await page.getByTestId('provider-select').selectOption('local')
  await page.getByTestId('composer-input').fill('écris une salutation brève')
  await page.getByTestId('composer-launch').click()

  const session = (await (await sessionCreated).json()) as { id?: string }
  expect(session.id, 'the session has an id').toBeTruthy()

  // The deterministic deltas crossed provider_router → SSE → useTokenStream
  // and rendered; the turn settles on the terminal status (never a verdict).
  const atelier = page.getByTestId('atelier')
  await expect(atelier).toContainText(FIXTURE_OLLAMA_TEXT, { timeout: 15_000 })
  await expect(page.getByTestId('turn-status')).toContainText('terminé')

  // PO-14 oracle #1: exactly ONE Done → exactly one assistant message.
  const log = await request.get(`/api/chat/${session.id}/log`, {
    headers: { 'x-sbfb-token': OPERATOR_TEST_TOKEN },
  })
  expect(log.status()).toBe(200)
  const messages = ((await log.json()) as { messages: { role: string }[] }).messages
  expect(
    messages.filter((m) => m.role === 'assistant'),
    'exactly one assistant message appended by the stream',
  ).toHaveLength(1)

  // PO-14 oracle #2: the upstream saw exactly ONE more /api/generate call.
  const after = (await (
    await request.get(`http://127.0.0.1:${FIXTURE_DAEMON_PORT}/__calls`)
  ).json()) as { generate: number }
  expect(after.generate - before.generate, 'exactly one upstream generate call').toBe(1)
})

test('sub-test (3b): the Network arm — zero deltas, exactly ONE Done (PO-14 full-stack)', async ({
  page,
  request,
}) => {
  // Wire-level PO-14 (review P3-2): ZERO delta frames on the Network arm,
  // exactly ONE Done, and the dispatched poll path was actually exercised
  // (the Debug network-poll frame is observable on the wire).
  const frames = await wireTranscript(request, 'network', 'demande la réponse du réseau fixture')
  expect(frames.match(/"type":"delta"/g) ?? [], 'zero delta frames').toHaveLength(0)
  expect(frames.match(/"type":"done"/g) ?? [], 'exactly ONE Done frame (PO-14)').toHaveLength(1)
  expect(frames.includes('"label":"network-poll"'), 'the poll path was exercised').toBe(true)

  // Counters snapshot AFTER the wire session — the UI turn below must add
  // exactly ONE more upstream submit.
  const before = (await (
    await request.get(`http://127.0.0.1:${FIXTURE_DAEMON_PORT}/__calls`)
  ).json()) as { submit: number }

  const sessionCreated = page.waitForResponse(
    (r) => r.url().includes('/api/chat/session') && r.request().method() === 'POST',
  )

  await page.getByTestId('provider-select').selectOption('network')
  await page.getByTestId('composer-input').fill('demande la réponse du réseau fixture')
  await page.getByTestId('composer-launch').click()

  const session = (await (await sessionCreated).json()) as { id?: string }
  expect(session.id, 'the session has an id').toBeTruthy()

  // submit → poll (≥1 dispatched) → completed → result_text as the single
  // Done; the atelier shows the result (zero streamed deltas on this arm).
  const atelier = page.getByTestId('atelier')
  await expect(atelier).toContainText(FIXTURE_NETWORK_RESULT, { timeout: 15_000 })
  await expect(page.getByTestId('turn-status')).toContainText('terminé')

  const log = await request.get(`/api/chat/${session.id}/log`, {
    headers: { 'x-sbfb-token': OPERATOR_TEST_TOKEN },
  })
  expect(log.status()).toBe(200)
  const messages = ((await log.json()) as { messages: { role: string }[] }).messages
  expect(
    messages.filter((m) => m.role === 'assistant'),
    'exactly one assistant message appended by the stream',
  ).toHaveLength(1)

  const after = (await (
    await request.get(`http://127.0.0.1:${FIXTURE_DAEMON_PORT}/__calls`)
  ).json()) as { submit: number }
  expect(after.submit - before.submit, 'exactly one upstream submit (PO-14)').toBe(1)
})

test('sub-test (4): a sensitive intention hits the MUR and never opens the stream', async ({ page }) => {
  let streamOpened = false
  page.on('request', (req) => {
    if (req.url().includes('/stream')) streamOpened = true
  })

  const sendGated = page.waitForResponse((r) => r.url().includes('/send'))

  await page.getByTestId('composer-input').fill('commit and push the branch feat/x')
  await page.getByTestId('composer-launch').click()

  const resp = await sendGated
  const body = (await resp.json()) as { requires_gate?: boolean }
  expect(body.requires_gate, '/send returns requires_gate for a sensitive message').toBe(true)

  // The MUR is restituted; there is no Forcer/Override and the atelier never ran.
  await expect(page.getByTestId('mur')).toBeVisible()
  await expect(page.getByText(/aucun « Forcer »/)).toBeVisible()
  await expect(page.getByTestId('atelier')).toHaveCount(0)

  // 0 spawn: the SSE stream was never opened.
  expect(streamOpened, 'the SSE stream is never opened for a gated intention').toBe(false)
})
