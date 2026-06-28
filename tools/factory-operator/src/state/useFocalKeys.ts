// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — keyboard focal switch. D6 keeps the bascule
// MANUAL; a deliberate keypress IS a manual switch (never an auto-switch off
// the stream). `s` → STEER, `v` → VERIFY, fired ONLY when no text field is
// focused and no modifier is held — so it never fights typing in the composer
// or a browser/Composer shortcut (Ctrl/Cmd+Enter etc.).
import { useEffect } from 'react'
import type { FocalMode } from './useOperator'

function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable
}

export function useFocalKeys(setMode: (mode: FocalMode) => void): void {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey || e.repeat) return
      if (isTypingTarget(e.target)) return
      if (e.key === 's' || e.key === 'S') setMode('steer')
      else if (e.key === 'v' || e.key === 'V') setMode('verify')
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [setMode])
}
