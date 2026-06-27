// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — a pure, incremental SSE frame parser. Isolated from
// the React hook so the five line-buffering footguns the preflight named
// (S1a) are unit-testable without a DOM: (1) a partial line carried across
// chunk boundaries, (2) a partial frame carried across chunk boundaries,
// (3) a frame boundary = a blank line, (4) comment lines (`:` keep-alive)
// ignored for forward-compat, (5) CRLF / LF tolerance. The Operator emits
// one compact single-line JSON per `data:` frame (operator_server.rs:1160),
// so each emitted payload is a whole JSON string ready for `JSON.parse`.

/** Stateful decoder: `push` returns the data payloads completed by `text`. */
export interface SseDecoder {
  /** Feed a decoded text chunk; returns the `data:` payloads now complete. */
  push(text: string): string[]
  /** Stream ended: flush a trailing frame the server closed without a blank line. */
  end(): string[]
}

export function createSseDecoder(): SseDecoder {
  let line = '' // partial line carried across chunk boundaries
  let dataLines: string[] = [] // `data:` values of the frame in progress

  // Consume one complete logical line. Returns a frame payload when `line`
  // is the blank line that terminates a frame (and the frame had data).
  function consume(raw: string, frames: string[]): void {
    // Tolerate CRLF: a trailing CR belongs to the line terminator, not data.
    const l = raw.endsWith('\r') ? raw.slice(0, -1) : raw
    if (l === '') {
      if (dataLines.length) {
        frames.push(dataLines.join('\n'))
        dataLines = []
      }
      return
    }
    if (l.startsWith(':')) return // comment / keep-alive — ignored
    if (l.startsWith('data:')) {
      const v = l.slice(5)
      dataLines.push(v.startsWith(' ') ? v.slice(1) : v)
      return
    }
    // event: / id: / retry: — not used by the Operator wire; ignored.
  }

  return {
    push(text: string): string[] {
      const frames: string[] = []
      line += text
      let nl: number
      while ((nl = line.indexOf('\n')) !== -1) {
        consume(line.slice(0, nl), frames)
        line = line.slice(nl + 1)
      }
      return frames
    },
    end(): string[] {
      const frames: string[] = []
      // A final unterminated line (server closed right after `data: json`
      // with no trailing newline) still completes its frame.
      if (line.length) {
        consume(line, frames)
        line = ''
      }
      if (dataLines.length) {
        frames.push(dataLines.join('\n'))
        dataLines = []
      }
      return frames
    },
  }
}
