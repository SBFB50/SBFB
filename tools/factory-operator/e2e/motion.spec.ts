// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

// T1 sub-test (Phase E) — the anti-déco invariant. Under prefers-reduced-motion
// the STEER⇄VERIFY altitude shift (signature 4) must land on its FINAL state
// with NO animation. Two precise assertions (getAnimations alone is timing-
// flaky and conflates the View Transition with the reveal/flip):
//   1. the native View Transition is never even STARTED (the JS guard in
//      altitudeShift skips it — the VT is NOT auto-gated by MotionConfig), and
//   2. after the bascule, no animation is left running (the reveal/flip render
//      PLAIN under reduced motion → zero WAAPI animation).
// "Le mouvement est du sens" — a reduced-motion user gets the end state at once.
test('altitude shift lands instantly under reduced motion (anti-déco)', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })

  // Spy on the native View Transition before any app script runs.
  await page.addInitScript(() => {
    const w = window as unknown as { __vtCalls: number }
    w.__vtCalls = 0
    const doc = document as Document & { startViewTransition?: (cb: () => void) => unknown }
    const native = doc.startViewTransition
    if (typeof native === 'function') {
      doc.startViewTransition = (cb: () => void) => {
        w.__vtCalls += 1
        return native.call(doc, cb)
      }
    }
  })

  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByTestId('operator-rail')).toBeVisible()
  await expect(page.getByTestId('steer-scene')).toBeVisible()

  // Bascule to VERIFY via the rail toggle (this is the altitude shift).
  await page.getByRole('button', { name: 'VERIFY' }).click()

  // The VERIFY scene is shown in its final state, STEER is gone.
  await expect(page.getByTestId('verify-scene')).toBeVisible()
  await expect(page.getByTestId('steer-scene')).toHaveCount(0)

  // (1) the View Transition was skipped (the reduced-motion guard fired).
  const vtCalls = await page.evaluate(() => (window as unknown as { __vtCalls: number }).__vtCalls)
  expect(vtCalls, 'native View Transition skipped under reduced motion').toBe(0)

  // (2) nothing is left animating (reveal/flip rendered plain).
  const running = await page.evaluate(
    () => document.getAnimations().filter((a) => a.playState === 'running').length,
  )
  expect(running, 'no running animation under reduced motion').toBe(0)
})
