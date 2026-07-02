// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase I — deterministic upstream fixture for the full-stack SSE
// sub-test (3). A line-by-line Node port of the Rust test mock
// `spawn_mock_daemon` (crates/sbfb-factory/src/provider_router.rs:731-784)
// plus a fake Ollama `/api/generate`. The Operator's provider_router reads
// SBFB_DAEMON_ENDPOINT / SBFB_OLLAMA_ENDPOINT at request time
// (provider_router.rs:282,155), so playwright.config.ts points both at this
// server and the REAL Rust dispatch path (handle_chat_stream →
// from_provider → run, MUR gate included) streams a deterministic reply.
//
// Contract notes (preflight §10.1, S4b findings):
//  - Network arm reads `task.task_id` (nested), `status`, `result_text`.
//    Terminal statuses: completed → one Done; rejected/timed_out → Error.
//  - Ollama arm (ollama-rs 0.3.4) parses each HTTP chunk separately and
//    silently drops fragmented objects (S4b-F1) → one res.write per NDJSON
//    line. Every line MUST carry model/created_at/response/done (S4b-F2).
//    The final line uses response:"" so no delta precedes the Done (S4b-F7).
//  - Every response closes its connection (mirror of the Rust mock).
//  - GET / → 200 is the Playwright webServer readiness probe (LibreChat
//    pattern); GET /__calls exposes counters so the spec can assert the
//    PO-14 invariant (exactly ONE submit / ONE generate per turn) E2E.
import http from 'node:http'

// Single source of truth for ports + reply constants is e2e/fixtures.ts
// (imported by the specs AND by playwright.config.ts, which injects them
// here as env — an .mjs child process cannot import the .ts module).
const port = Number(process.env.FIXTURE_DAEMON_PORT || 3112)
const NETWORK_RESULT_TEXT = process.env.FIXTURE_NETWORK_RESULT || 'reponse-fixture-network'
const OLLAMA_DELTAS = (process.env.FIXTURE_OLLAMA_DELTAS || 'Bonjour |monde fixture').split('|')

const calls = { submit: 0, poll: 0, result: 0, generate: 0 }
/** polls seen per task id — first poll answers `dispatched`, then `completed`
 * (≥1 dispatched keeps the Debug network-poll frame observable). */
const pollsByTask = new Map()

function json(res, status, body) {
  const payload = JSON.stringify(body)
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(payload),
    Connection: 'close',
  })
  res.end(payload)
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://127.0.0.1:${port}`)
  const path = url.pathname

  // Readiness probe (Playwright webServer polls this URL).
  if (req.method === 'GET' && path === '/') {
    res.writeHead(200, { 'Content-Type': 'text/plain', Connection: 'close' })
    res.end('fixture-daemon ready')
    return
  }

  // Observability: the spec asserts exactly one submit/generate per turn.
  if (req.method === 'GET' && path === '/__calls') {
    json(res, 200, calls)
    return
  }

  // --- Network arm (mirror of spawn_mock_daemon, provider_router.rs:756-770) ---
  if (req.method === 'POST' && path.endsWith('/tasks/submit')) {
    calls.submit += 1
    // Drain the request body (submit carries JSON we do not need to parse).
    req.resume()
    req.on('end', () => json(res, 200, { task: { task_id: 'e2e-task-1' } }))
    return
  }
  if (req.method === 'GET' && path.endsWith('/result')) {
    calls.result += 1
    json(res, 200, {
      task_id: 'e2e-task-1',
      status: 'completed',
      result_text: NETWORK_RESULT_TEXT,
    })
    return
  }
  if (req.method === 'GET' && path.includes('/tasks/e2e-task-1')) {
    calls.poll += 1
    const seen = (pollsByTask.get('e2e-task-1') ?? 0) + 1
    pollsByTask.set('e2e-task-1', seen)
    json(res, 200, {
      task_id: 'e2e-task-1',
      status: seen <= 1 ? 'dispatched' : 'completed',
    })
    return
  }

  // --- Ollama arm (fake /api/generate, NDJSON) ---
  if (req.method === 'POST' && path === '/api/generate') {
    calls.generate += 1
    req.resume()
    req.on('end', () => {
      res.writeHead(200, {
        'Content-Type': 'application/x-ndjson',
        Connection: 'close',
      })
      const line = (obj) => `${JSON.stringify(obj)}\n`
      const base = { model: 'e2e-fixture', created_at: '2026-07-02T00:00:00Z' }
      // One atomic write per complete NDJSON object (S4b-F1).
      for (const delta of OLLAMA_DELTAS) {
        res.write(line({ ...base, response: delta, done: false }))
      }
      // Final frame: response:"" (S4b-F7) + total_duration in ns → 12 ms.
      res.end(line({ ...base, response: '', done: true, total_duration: 12_000_000 }))
    })
    return
  }

  json(res, 200, { error: 'unexpected' })
})

server.listen(port, '127.0.0.1', () => {
  console.log(`[fixture-daemon] listening on 127.0.0.1:${port}`)
})
