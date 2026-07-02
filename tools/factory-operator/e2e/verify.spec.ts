// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

// Sprint 80 Phase H — T1 VERIFY-plein + arbre de procédé. Runs against the REAL
// Operator (serve-operator.mjs): the bespoke diff-viewer renders the REAL
// working-tree hunks of the SEALED fixture workspace (Phase I hermeticity —
// the Operator is spawned with cwd=<fixture>, computed in Rust /api/git/diff)
// and the gates band restitutes the REAL live gates 1:1 (/api/gates) — while
// the ÉTAT slot never fabricates a verdict (scan-front-discipline gate,
// runtime-checked).

test.beforeEach(async ({ page }) => {
  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByTestId('operator-rail')).toBeVisible()
})

test('VERIFY-plein shows the bespoke diff-viewer + the live gates band; ÉTAT never says a verdict', async ({
  page,
}) => {
  await page.getByRole('button', { name: 'VERIFY' }).click()
  await expect(page.getByTestId('verify-scene')).toBeVisible()

  // The diff-viewer (working-tree hunks computed in Rust) is the default tool…
  await expect(page.getByTestId('verify-diff')).toBeVisible()
  // …and the permanent gates band restitutes the REAL live gates 1:1.
  const gates = page.getByTestId('verify-gates')
  await expect(gates).toBeVisible()
  await expect(gates).toContainText('lint-planning') // gates_live_data always restitutes it

  // The ÉTAT slot is a NAMED state, never the recorded review word.
  const etat = await page.getByTestId('verify-etat').textContent()
  expect(etat ?? '').not.toMatch(/\bPASS\b/)

  // The gates band fabricates NO aggregated verdict and NO % score (cardinal).
  await expect(gates.getByText(/\bPASS\b/)).toHaveCount(0)
  await expect(gates.getByText(/\d+\s*%/)).toHaveCount(0)

  // The terminal-PTY stays reachable as a SECONDARY tool — no auto-spawn just
  // by switching mode (the explicit start CTA appears only after the toggle).
  await expect(page.getByTestId('terminal-start')).toHaveCount(0)
  await page.getByTestId('verify-tool-toggle').click()
  await expect(page.getByTestId('terminal-start')).toBeVisible()
})

test('the Procédé inspector restitutes ≥1 phase verdict (never a score) and the diff bi-usage renders a past commit', async ({
  page,
}) => {
  await page.getByTestId('rail-surface-procede').click()
  // getSprintHistory is a heavy REAL fetch (full sprint history of this repo:
  // a git diff per phase commit + the .planning files, several git subprocesses
  // on Windows) — give it a generous timeout, not the 5s default.
  await expect(page.getByTestId('procede-surface')).toBeVisible({ timeout: 25_000 })

  const phases = page.getByTestId('procede-phase')
  await expect(phases.first()).toBeVisible()

  // A RESTITUTED verdict pill shows a REAL recorded verdict (not the "—"
  // default) — proving the value is read off the artifacts, not fabricated.
  const realVerdict = page
    .getByTestId('verdict-pill')
    .filter({ hasText: /EXECUTE|PLAN-ADAPT|SCOPE-CUT|PASS|CONCERN|FAIL|DESIGN-CONFLICT/ })
  await expect(realVerdict.first()).toBeVisible()

  // No fabricated percentage score / gauge.
  await expect(page.getByTestId('procede-surface').getByText(/\d+\s*%/)).toHaveCount(0)

  // Bi-usage (fold V2/U7): expanding a phase renders the SAME bespoke
  // diff-viewer over a PAST commit's diff (/sprint-history/diff/{sha}).
  await phases.first().click()
  // the past-commit diff is another real git subprocess (commit_diff_data).
  await expect(page.getByTestId('diff-view').first()).toBeVisible({ timeout: 25_000 })
})
