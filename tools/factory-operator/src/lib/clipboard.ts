// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — a tiny clipboard helper. The Operator runs on
// localhost (a secure context), so `navigator.clipboard.writeText` is
// available; the write is local (not a `connect-src` request), so it is
// CSP-safe. Returns whether the copy succeeded so callers can show feedback.
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text)
      return true
    }
  } catch {
    // Clipboard denied / unavailable — report failure, never throw.
  }
  return false
}
