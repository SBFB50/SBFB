// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { preflightTone, reviewTone, toneBg, toneText, VERIFY_ETAT } from './verdict'

describe('reviewTone (restituted review verdict)', () => {
  it('maps the recorded verdicts to honest tones', () => {
    expect(reviewTone('PASS')).toBe('ok')
    expect(reviewTone('PASS-PENDING')).toBe('warn')
    expect(reviewTone('CONCERN')).toBe('warn')
    expect(reviewTone('FAIL')).toBe('bad')
    expect(reviewTone(null)).toBe('neu')
    expect(reviewTone(undefined)).toBe('neu')
  })
})

describe('preflightTone (restituted preflight verdict)', () => {
  it('maps the four verdicts', () => {
    expect(preflightTone('EXECUTE')).toBe('ok')
    expect(preflightTone('PLAN-ADAPT')).toBe('info')
    expect(preflightTone('SCOPE-CUT-CONSISTENT')).toBe('info')
    expect(preflightTone('DESIGN-CONFLICT')).toBe('bad')
    expect(preflightTone('UNKNOWN')).toBe('neu')
  })
})

describe('tone classes are literal (Tailwind-detectable)', () => {
  it('returns literal text-/bg- utility names', () => {
    expect(toneText('ok')).toBe('text-ok')
    expect(toneText('neu')).toBe('text-tx3')
    expect(toneBg('bad')).toBe('bg-bad')
    expect(toneBg('neu')).toBe('bg-tx4')
  })
})

describe('VERIFY état slot never fabricates a verdict', () => {
  it('the named states never contain the recorded review word', () => {
    // The cardinal invariant + the scan-front-discipline gate: the live état
    // slot is a named state machine; it must never read like a PASS verdict.
    for (const text of Object.values(VERIFY_ETAT)) {
      expect(text).not.toMatch(/\bPASS\b/)
      expect(text).not.toMatch(/Vérifié|Approuvé/)
    }
  })
})
