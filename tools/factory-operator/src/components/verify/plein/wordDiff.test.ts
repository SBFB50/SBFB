// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { wordDiff } from './wordDiff'

// The word diff refines the highlight WITHIN a del→add replacement. Invariant:
// the segments always concatenate back to the original line (it only marks, it
// never rewrites the content — the Rust hunk stays the source of truth).
describe('wordDiff (intra-line token LCS — fold V1)', () => {
  it('marks only the changed tokens, keeping the common run plain', () => {
    const wd = wordDiff('let ok = false;', 'let ok = true;')
    expect(wd.old.filter((s) => s.changed).map((s) => s.text)).toEqual(['false'])
    expect(wd.new.filter((s) => s.changed).map((s) => s.text)).toEqual(['true'])
    // segments reconstruct the original line exactly (never a rewrite)
    expect(wd.old.map((s) => s.text).join('')).toBe('let ok = false;')
    expect(wd.new.map((s) => s.text).join('')).toBe('let ok = true;')
  })

  it('marks nothing when the two lines are identical', () => {
    const wd = wordDiff('return ok;', 'return ok;')
    expect(wd.old.every((s) => !s.changed)).toBe(true)
    expect(wd.new.every((s) => !s.changed)).toBe(true)
  })

  it('handles empty inputs', () => {
    expect(wordDiff('', '')).toEqual({ old: [], new: [] })
  })

  it('degrades to one coarse segment past the token ceiling (no O(n·m) blow-up)', () => {
    const long = Array.from({ length: 300 }, (_, i) => `t${i}`).join(' ')
    const longer = `${long} extra`
    const wd = wordDiff(long, longer)
    expect(wd.old).toEqual([{ text: long, changed: true }])
    expect(wd.new).toEqual([{ text: longer, changed: true }])
  })
})
