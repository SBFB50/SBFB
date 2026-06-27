// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { isTerminal, parseChunk } from './streamChunk'

describe('parseChunk', () => {
  it('parses the five Rust enum variants', () => {
    expect(parseChunk('{"type":"delta","text":"a"}')).toEqual({ type: 'delta', text: 'a' })
    expect(parseChunk('{"type":"thinking","text":"t"}')).toEqual({ type: 'thinking', text: 't' })
    expect(parseChunk('{"type":"done","cost_usd":0.1,"duration_ms":5,"result":"r"}')).toEqual({
      type: 'done',
      result: 'r',
      cost_usd: 0.1,
      duration_ms: 5,
    })
    expect(parseChunk('{"type":"error","message":"boom"}')).toEqual({ type: 'error', message: 'boom' })
    expect(parseChunk('{"type":"debug","label":"exit","content":"0"}')).toEqual({
      type: 'debug',
      label: 'exit',
      content: '0',
    })
  })

  it('parses the hand-forged requires_gate literal (the 6th value, outside serde)', () => {
    expect(parseChunk('{"type":"requires_gate","message":"gated"}')).toEqual({
      type: 'requires_gate',
      message: 'gated',
    })
  })

  it('returns null for an unknown type, malformed JSON, or a missing field', () => {
    expect(parseChunk('{"type":"unknown"}')).toBeNull()
    expect(parseChunk('not json')).toBeNull()
    expect(parseChunk('{"type":"delta"}')).toBeNull() // missing text
    expect(parseChunk('{"type":"done"}')).toBeNull() // missing result
    expect(parseChunk('123')).toBeNull()
    expect(parseChunk('null')).toBeNull()
  })
})

describe('isTerminal', () => {
  it('treats done / error / requires_gate as terminal', () => {
    expect(isTerminal({ type: 'done', result: 'r' })).toBe(true)
    expect(isTerminal({ type: 'error', message: 'e' })).toBe(true)
    expect(isTerminal({ type: 'requires_gate', message: 'g' })).toBe(true)
  })

  it('treats delta / thinking / debug as non-terminal', () => {
    expect(isTerminal({ type: 'delta', text: 'a' })).toBe(false)
    expect(isTerminal({ type: 'thinking', text: 't' })).toBe(false)
    expect(isTerminal({ type: 'debug', label: 'l', content: 'c' })).toBe(false)
  })
})
