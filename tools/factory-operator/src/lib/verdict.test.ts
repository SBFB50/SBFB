// SPDX-License-Identifier: AGPL-3.0-or-later
import { afterAll, describe, expect, it } from 'vitest'
import {
  GATE_STATUS,
  gateStatusGlyph,
  gateStatusLabel,
  gateStatusTone,
  pickVerifyEtat,
  preflightTone,
  reviewTone,
  toneBg,
  toneText,
  VERIFY_ETAT,
} from './verdict'
import { i18n } from '../i18n/i18n'
import { messages as enMessages } from '../i18n/locales/en.po'

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

  it('pickVerifyEtat reads observable facts, not a verdict', () => {
    expect(pickVerifyEtat({ loading: true, hasChanges: false })).toBe('reading')
    expect(pickVerifyEtat({ loading: false, hasChanges: true })).toBe('inspecting')
    expect(pickVerifyEtat({ loading: false, hasChanges: false })).toBe('empty')
  })
})

describe('gate status restitution (Phase H — GET /api/gates)', () => {
  it('GATE_STATUS mirrors EXACTLY the five wire statuses', () => {
    expect(Object.values(GATE_STATUS).sort()).toEqual([
      'blocking',
      'informational',
      'not_applicable',
      'not_run',
      'passed',
    ])
  })

  it('maps each status to a distinct glyph (a tick, never the literal PASS)', () => {
    expect(gateStatusGlyph('passed')).toBe('✓')
    expect(gateStatusGlyph('blocking')).toBe('✕')
    expect(gateStatusGlyph('not_applicable')).toBe('—')
    expect(gateStatusGlyph('informational')).toBe('•')
    expect(gateStatusGlyph('not_run')).toBe('•')
  })

  it('maps each status to an honest tone', () => {
    expect(gateStatusTone('passed')).toBe('ok')
    expect(gateStatusTone('blocking')).toBe('bad')
    expect(gateStatusTone('informational')).toBe('warn')
    expect(gateStatusTone('not_run')).toBe('neu')
    expect(gateStatusTone('not_applicable')).toBe('neu')
  })

  it('labels never use a forbidden verdict word', () => {
    for (const status of Object.values(GATE_STATUS)) {
      expect(gateStatusLabel(status)).not.toMatch(/\b(PASS|Vérifié|Approuvé)\b/)
    }
  })
})

// End-to-end proof of the Lingui pipeline (front rapid-add étape 1): the `t`
// macro was extracted into the catalog, compiled to an eval-free module by the
// Vite plugin, loaded, and rendered for the ACTIVE locale. gateStatusLabel reads
// the global i18n, so activating `en` flips its output — and the EN labels stay
// non-verdict (`met`, never `passed`/`PASS`, scan-front discipline).
describe('gateStatusLabel renders the active locale (i18n pipeline proof)', () => {
  afterAll(() => {
    i18n.activate('fr') // restore the source locale for the rest of the suite
  })

  it('renders FR (source) by default', () => {
    expect(gateStatusLabel('passed')).toBe('tenue')
    expect(gateStatusLabel('blocking')).toBe('bloquant')
  })

  it('renders EN once the en catalog is loaded + activated', () => {
    i18n.load('en', enMessages)
    i18n.activate('en')
    expect(gateStatusLabel('passed')).toBe('met')
    expect(gateStatusLabel('blocking')).toBe('blocking')
    expect(gateStatusLabel('not_run')).toBe('not run')
  })
})
