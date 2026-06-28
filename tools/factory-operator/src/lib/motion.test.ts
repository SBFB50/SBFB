// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import { MOTION_SIGNATURES, MOTION_SIGNATURE_IDS, MOTION_TRAVEL_PX } from './motion'

// The motion allowlist is doctrinal (no script enforces it) — this test pins it
// so a sixth "decorative" signature cannot be added silently: motion = sens.
describe('motion vocabulary', () => {
  it('declares exactly the five allowed signatures, all distinct', () => {
    expect(MOTION_SIGNATURE_IDS).toHaveLength(5)
    expect(new Set(MOTION_SIGNATURE_IDS).size).toBe(5)
  })

  it('names each signature by its meaning', () => {
    expect(MOTION_SIGNATURES).toEqual({
      tokenSettle: 'token-settle',
      gateFlip: 'gate-flip',
      verificationReveal: 'verification-reveal',
      altitudeShift: 'altitude-shift',
      confirmationGravity: 'confirmation-gravity',
    })
  })

  it('uses a positional travel (transform) so it collapses under reduced motion', () => {
    expect(MOTION_TRAVEL_PX).toBeGreaterThan(0)
  })
})
