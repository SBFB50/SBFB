// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { createSseDecoder } from './sseFrames'

describe('createSseDecoder', () => {
  it('emits one payload per complete frame', () => {
    const d = createSseDecoder()
    expect(d.push('data: {"type":"delta","text":"a"}\n\n')).toEqual([
      '{"type":"delta","text":"a"}',
    ])
  })

  it('carries a partial FRAME across chunk boundaries', () => {
    const d = createSseDecoder()
    expect(d.push('data: {"type":"delta",')).toEqual([])
    expect(d.push('"text":"hi"}\n\n')).toEqual(['{"type":"delta","text":"hi"}'])
  })

  it('carries a partial LINE across chunk boundaries', () => {
    const d = createSseDecoder()
    expect(d.push('data: {"x":1}')).toEqual([]) // no newline yet
    expect(d.push('\n')).toEqual([]) // line complete, frame not terminated
    expect(d.push('\n')).toEqual(['{"x":1}']) // blank line → frame boundary
  })

  it('splits multiple frames in a single chunk', () => {
    const d = createSseDecoder()
    expect(d.push('data: 1\n\ndata: 2\n\ndata: 3\n\n')).toEqual(['1', '2', '3'])
  })

  it('ignores comment / keep-alive lines (forward-compat)', () => {
    const d = createSseDecoder()
    expect(d.push(': keep-alive\n\ndata: {"ok":true}\n\n')).toEqual(['{"ok":true}'])
  })

  it('tolerates CRLF terminators', () => {
    const d = createSseDecoder()
    expect(d.push('data: {"x":1}\r\n\r\n')).toEqual(['{"x":1}'])
  })

  it('flushes a frame the server closed without a trailing blank line', () => {
    const d = createSseDecoder()
    expect(d.push('data: {"type":"done","result":"x"}\n')).toEqual([])
    expect(d.end()).toEqual(['{"type":"done","result":"x"}'])
  })

  it('flushes a final unterminated line on end()', () => {
    const d = createSseDecoder()
    expect(d.push('data: tail')).toEqual([])
    expect(d.end()).toEqual(['tail'])
  })
})
