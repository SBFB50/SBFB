// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the SSE event union, HARDCODED (preflight
// plan-adaptation #3). The Rust enum `StreamChunk` (#[serde(tag="type")],
// llm_bridge.rs:42-59) has only FIVE variants — delta, thinking, done,
// error, debug. The sixth value, `requires_gate`, is a hand-forged JSON
// literal emitted by `sse_gate()` OUTSIDE serde (operator_server.rs, fn
// sse_gate — anchor by name, the line drifts).
// Deriving the union from the 5-variant enum alone would drop
// `requires_gate` and leave the MUR mute on the front — so the six values
// live here as one explicit source of truth.

export type StreamChunk =
  | { type: 'delta'; text: string }
  | { type: 'thinking'; text: string }
  | { type: 'done'; cost_usd?: number; duration_ms?: number; result: string }
  | { type: 'error'; message: string }
  | { type: 'debug'; label: string; content: string }
  | { type: 'requires_gate'; message: string }

/**
 * Terminal events latch the accumulator: the FIRST of these closes the turn
 * and every later event is ignored. The Claude arm keeps emitting Debug
 * 'exit' (and an Error on a non-zero exit) AFTER its Done (llm_bridge.rs:
 * 317-328), so latching the first terminal is load-bearing for PO-14.
 */
export function isTerminal(chunk: StreamChunk): boolean {
  return chunk.type === 'done' || chunk.type === 'error' || chunk.type === 'requires_gate'
}

/**
 * Parse one `data:` payload into a known chunk, or `null` for an unknown /
 * malformed value (dropped for forward-compatibility, never thrown). The
 * payload is the compact single-line JSON the Operator writes per frame.
 */
export function parseChunk(data: string): StreamChunk | null {
  let value: unknown
  try {
    value = JSON.parse(data)
  } catch {
    return null
  }
  if (typeof value !== 'object' || value === null) return null
  const v = value as Record<string, unknown>
  switch (v.type) {
    case 'delta':
      return typeof v.text === 'string' ? { type: 'delta', text: v.text } : null
    case 'thinking':
      return typeof v.text === 'string' ? { type: 'thinking', text: v.text } : null
    case 'done':
      return typeof v.result === 'string'
        ? {
            type: 'done',
            result: v.result,
            cost_usd: typeof v.cost_usd === 'number' ? v.cost_usd : undefined,
            duration_ms: typeof v.duration_ms === 'number' ? v.duration_ms : undefined,
          }
        : null
    case 'error':
      return typeof v.message === 'string' ? { type: 'error', message: v.message } : null
    case 'requires_gate':
      return typeof v.message === 'string' ? { type: 'requires_gate', message: v.message } : null
    case 'debug':
      return typeof v.label === 'string' && typeof v.content === 'string'
        ? { type: 'debug', label: v.label, content: v.content }
        : null
    default:
      return null
  }
}
