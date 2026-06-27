// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the live PTY terminal (J12), the bootstrap VERIFY tool.
// This module STATICALLY imports @xterm/xterm (~345 kB raw) and is loaded ONLY
// via a dynamic import() from <Terminal> (lazy boundary) — preflight
// plan-adaptation #1/#2: a static import would land xterm in the 40 kB `index`
// hero chunk and bust the size-limit gate; here it is split into the measured
// async `vendor-xterm` chunk (.size-limit.json), loaded only when the operator
// starts a session.
//
// Wire (terminal.rs handle_terminal_ws): PTY output → WS Binary frames
// (xterm.write bytes); keystrokes → WS Text frames (raw); resize →
// `{"type":"resize",cols,rows}` Text frame. Auth rides the same-origin
// HttpOnly cookie on the WS handshake (a WS cannot set a custom header — the
// reason the cookie is the 1st gesture, Day-0 #5). The server records the
// session to a `.cast` (replayable in the Sessions inspector, U6/V9).
import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { terminalWsUrl } from '../../api/operator'

/** Read an oklch design token off :root (Tailwind v4 @theme mirrors them) so
 * the terminal matches the shell; a fallback keeps it readable pre-layout. */
function cssVar(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return v || fallback
}

export type TerminalStatus = 'connexion…' | 'session active' | 'session close' | 'erreur de liaison'

export default function TerminalXterm({
  resume,
  onStatus,
}: {
  resume?: string
  onStatus?: (status: TerminalStatus) => void
}) {
  const hostRef = useRef<HTMLDivElement>(null)
  // Keep onStatus in a ref so a new callback identity from the parent never
  // tears down the PTY (which would re-spawn the server-side `claude` child).
  // The ref is updated in an effect (never during render — react-hooks/refs).
  const onStatusRef = useRef(onStatus)
  useEffect(() => {
    onStatusRef.current = onStatus
  }, [onStatus])

  useEffect(() => {
    const host = hostRef.current
    if (!host) return

    const term = new Terminal({
      convertEol: false,
      cursorBlink: true,
      fontFamily: "'Geist Mono Variable', ui-monospace, monospace",
      fontSize: 12,
      theme: {
        background: cssVar('--color-s0', '#15140f'),
        foreground: cssVar('--color-tx', '#e8e7e2'),
        cursor: cssVar('--color-tx2', '#b4b2ab'),
      },
    })
    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(host)
    try {
      fit.fit()
    } catch {
      // host not laid out yet — the ResizeObserver below fits once it is.
    }

    onStatusRef.current?.('connexion…')
    const ws = new WebSocket(terminalWsUrl(resume))
    ws.binaryType = 'arraybuffer'
    let live = false

    ws.onopen = () => {
      live = true
      onStatusRef.current?.('session active')
      ws.send(JSON.stringify({ type: 'resize', cols: term.cols, rows: term.rows }))
    }
    ws.onmessage = (ev) => {
      if (typeof ev.data === 'string') term.write(ev.data)
      else term.write(new Uint8Array(ev.data as ArrayBuffer))
    }
    ws.onclose = () => {
      live = false
      onStatusRef.current?.('session close')
    }
    ws.onerror = () => onStatusRef.current?.('erreur de liaison')

    const dataSub = term.onData((d) => {
      if (live) ws.send(d)
    })
    const resizeSub = term.onResize(({ cols, rows }) => {
      if (live) ws.send(JSON.stringify({ type: 'resize', cols, rows }))
    })
    const ro = new ResizeObserver(() => {
      try {
        fit.fit()
      } catch {
        // transient layout — ignore
      }
    })
    ro.observe(host)

    return () => {
      ro.disconnect()
      dataSub.dispose()
      resizeSub.dispose()
      try {
        ws.close()
      } catch {
        // already closing
      }
      term.dispose()
    }
  }, [resume])

  return <div ref={hostRef} data-testid="terminal-xterm" className="h-full w-full" />
}
