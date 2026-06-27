// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — a pure asciicast v2 parser for the gate/terminal replay
// (folds U6/V9). The Operator records every PTY session to a `.cast` file
// (terminal.rs write_asciicast_header / write_asciicast_event): a first JSON
// HEADER line `{"version":2,"width":..,"height":..,..}` then one JSON event
// array per line `[time, "o", data]` ("o" = output). We replay by writing each
// event's `data` into an xterm instance, so this parser is isolated and
// unit-testable WITHOUT a DOM. Malformed lines are skipped (forward-compat),
// never thrown — a truncated tail of a live recording must still replay.

export interface CastHeader {
  version: number
  width: number
  height: number
}

export interface CastEvent {
  /** Seconds since the recording start. */
  time: number
  /** The raw terminal output chunk (ANSI included) to xterm.write(). */
  data: string
}

export interface Cast {
  header: CastHeader | null
  events: CastEvent[]
}

export function parseCast(raw: string): Cast {
  let header: CastHeader | null = null
  const events: CastEvent[] = []

  for (const line of raw.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed) continue

    let value: unknown
    try {
      value = JSON.parse(trimmed)
    } catch {
      continue // a partial/garbled line — skip, never throw
    }

    // The header is the first object-with-version line.
    if (!header && value && typeof value === 'object' && !Array.isArray(value)) {
      const h = value as Record<string, unknown>
      if ('version' in h) {
        header = {
          version: typeof h.version === 'number' ? h.version : 2,
          width: typeof h.width === 'number' ? h.width : 80,
          height: typeof h.height === 'number' ? h.height : 24,
        }
        continue
      }
    }

    // An output event: [time, "o", data]. We only replay "o" (output) events;
    // "i" (input) events, if any, are not part of the recorded screen.
    if (Array.isArray(value) && value.length >= 3 && value[1] === 'o' && typeof value[2] === 'string') {
      events.push({
        time: typeof value[0] === 'number' ? value[0] : 0,
        data: value[2],
      })
    }
  }

  return { header, events }
}

/** The concatenated output of a cast — the full final screen buffer, used for
 * an instant (non-timed) replay into xterm. */
export function castOutput(cast: Cast): string {
  return cast.events.map((e) => e.data).join('')
}
