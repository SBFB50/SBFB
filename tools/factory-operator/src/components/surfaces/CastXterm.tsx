// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — write-only xterm for replaying a recorded `.cast`
// (folds U6/V9). Like <TerminalXterm> it STATICALLY imports @xterm/xterm, so
// it shares the async `vendor-xterm` chunk and is only ever reached via a
// dynamic import() (lazy boundary in <CastReplay>). No WebSocket, no PTY: it
// renders the recorded screen by writing each cast output chunk — the raw
// gate/terminal output verbatim (sortie brute), never re-interpreted.
import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { castOutput, parseCast } from '../../lib/cast'

function cssVar(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return v || fallback
}

export default function CastXterm({ raw }: { raw: string }) {
  const hostRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const host = hostRef.current
    if (!host) return
    const term = new Terminal({
      convertEol: false,
      disableStdin: true,
      fontFamily: "'Geist Mono Variable', ui-monospace, monospace",
      fontSize: 12,
      theme: {
        background: cssVar('--color-s0', '#15140f'),
        foreground: cssVar('--color-tx', '#e8e7e2'),
      },
    })
    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(host)
    try {
      fit.fit()
    } catch {
      // not laid out yet
    }
    term.write(castOutput(parseCast(raw)))
    const ro = new ResizeObserver(() => {
      try {
        fit.fit()
      } catch {
        // transient
      }
    })
    ro.observe(host)
    return () => {
      ro.disconnect()
      term.dispose()
    }
  }, [raw])

  return <div ref={hostRef} data-testid="cast-xterm" className="h-full w-full" />
}
