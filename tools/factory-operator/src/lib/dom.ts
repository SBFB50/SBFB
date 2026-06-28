// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — shared DOM predicate. Reused by useFocalKeys
// (don't fire the s/v switch while typing) and DiffViewer (don't steal focus
// from a text field). One source so the two stay in lock-step.
export function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable
}
