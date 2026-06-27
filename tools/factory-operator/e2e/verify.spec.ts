// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

// Sprint 80 Phase D — T1 VERIFY-bootstrap + arbre de procédé. Runs against the
// REAL Operator (serve-operator.mjs): the Procédé inspector restitutes the
// real sprint_history of THIS repo, so a recorded verdict is rendered while the
// ÉTAT slot never fabricates one (scan-front-discipline gate, runtime-checked).

test.beforeEach(async ({ page }) => {
  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByTestId('operator-rail')).toBeVisible()
})

test('VERIFY mode shows the bootstrap scene; the ÉTAT slot never says a verdict', async ({ page }) => {
  await page.getByRole('button', { name: 'VERIFY' }).click()
  await expect(page.getByTestId('verify-scene')).toBeVisible()

  // The terminal does NOT auto-start (no claude spawn just by switching mode):
  // the explicit start CTA is shown instead.
  await expect(page.getByTestId('terminal-start')).toBeVisible()

  // The permanent ÉTAT slot is a named state, never the recorded review word.
  const etat = await page.getByTestId('verify-etat').textContent()
  expect(etat ?? '').not.toMatch(/\bPASS\b/)
})

test('the Procédé inspector restitutes ≥1 phase with its recorded verdict (never a score)', async ({ page }) => {
  await page.getByTestId('rail-surface-procede').click()
  await expect(page.getByTestId('procede-surface')).toBeVisible()

  // At least one phase node of the real sprint history is rendered…
  const phases = page.getByTestId('procede-phase')
  await expect(phases.first()).toBeVisible()

  // …carrying a RESTITUTED verdict pill that shows a REAL recorded verdict
  // (not the "—" empty default) — proving the value is read off the artifacts,
  // not fabricated. The committed phases of this repo carry EXECUTE/PLAN-ADAPT
  // preflights and PASS reviews.
  const realVerdict = page
    .getByTestId('verdict-pill')
    .filter({ hasText: /EXECUTE|PLAN-ADAPT|SCOPE-CUT|PASS|CONCERN|FAIL|DESIGN-CONFLICT/ })
  await expect(realVerdict.first()).toBeVisible()

  // …and the surface shows no fabricated percentage score / gauge.
  await expect(page.getByTestId('procede-surface').getByText(/\d+\s*%/)).toHaveCount(0)
})
