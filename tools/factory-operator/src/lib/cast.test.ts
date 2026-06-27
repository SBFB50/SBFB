// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { castOutput, parseCast } from './cast'

const HEADER = '{"version":2,"width":120,"height":30,"timestamp":1700000000}'
const ESC = String.fromCharCode(27) // the ESC byte (asciicast serialises it as \uXXXX)

describe('parseCast', () => {
  it('parses the header and the output events', () => {
    const raw = [HEADER, '[0.1, "o", "hello "]', '[0.2, "o", "world"]'].join('\n')
    const cast = parseCast(raw)
    expect(cast.header).toEqual({ version: 2, width: 120, height: 30 })
    expect(cast.events).toEqual([
      { time: 0.1, data: 'hello ' },
      { time: 0.2, data: 'world' },
    ])
  })

  it('skips malformed / partial lines without throwing (truncated tail tolerant)', () => {
    const raw = [HEADER, '[0.1, "o", "ok"]', '[0.2, "o", "trunca', ''].join('\n')
    const cast = parseCast(raw)
    expect(cast.events).toEqual([{ time: 0.1, data: 'ok' }])
  })

  it('ignores non-output ("i" input) events — only the screen is replayed', () => {
    const raw = [HEADER, '[0.1, "i", "ls\\n"]', '[0.2, "o", "a b c"]'].join('\n')
    const cast = parseCast(raw)
    expect(cast.events).toEqual([{ time: 0.2, data: 'a b c' }])
  })

  it('tolerates a recording with no header (events still parse)', () => {
    const cast = parseCast('[0.0, "o", "x"]')
    expect(cast.header).toBeNull()
    expect(cast.events).toHaveLength(1)
  })

  it('skips a too-short event array (length < 3) without throwing', () => {
    const raw = [HEADER, '[0.1, "o"]', '[0.2, "o", "ok"]'].join('\n')
    const cast = parseCast(raw)
    expect(cast.events).toEqual([{ time: 0.2, data: 'ok' }])
  })

  it('defaults a header missing width/height to 80x24', () => {
    const cast = parseCast('{"version":2}\n[0.1, "o", "x"]')
    expect(cast.header).toEqual({ version: 2, width: 80, height: 24 })
  })

  it('castOutput concatenates the output verbatim, ANSI bytes intact', () => {
    // `\\u001b` in the raw JSON is decoded by JSON.parse into the ESC byte; the
    // parser keeps it, so the concatenated output still holds ESC + ANSI.
    const raw = [HEADER, '[0.1, "o", "\\u001b[32mgreen\\u001b[0m"]', '[0.2, "o", " tail"]'].join('\n')
    expect(castOutput(parseCast(raw))).toBe(`${ESC}[32mgreen${ESC}[0m tail`)
  })
})
