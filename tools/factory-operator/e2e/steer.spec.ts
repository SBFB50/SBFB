// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

// Sprint 80 Phase C — T1 sub-tests (2) composeur → session and (4) the MUR
// gates a sensitive intention WITHOUT execution. Both run against the REAL
// Operator server (serve-operator.mjs). The deterministic single-Done SSE
// sub-test (3) is covered by the unit suite (useTokenStream.test.ts, a
// mocked ReadableStream); its full-stack echo-target variant lands Phase I.

test.beforeEach(async ({ page }) => {
  // Bootstrap: mint the HttpOnly cookie, then load the shell at /.
  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByTestId('operator-rail')).toBeVisible()
})

test('sub-test (2): composing a benign intention creates a session', async ({ page }) => {
  const sessionCreated = page.waitForResponse(
    (r) => r.url().includes('/api/chat/session') && r.request().method() === 'POST',
  )

  await page.getByTestId('composer-input').fill('prépare le plan de la phase')
  await page.getByTestId('composer-launch').click()

  const resp = await sessionCreated
  expect(resp.status(), 'POST /api/chat/session is 200').toBe(200)
  const body = (await resp.json()) as { id?: string }
  expect(body.id, 'the session has an id').toBeTruthy()

  // The scene left the empty state for the observable atelier.
  await expect(page.getByTestId('atelier')).toBeVisible()
})

test('sub-test (4): a sensitive intention hits the MUR and never opens the stream', async ({ page }) => {
  let streamOpened = false
  page.on('request', (req) => {
    if (req.url().includes('/stream')) streamOpened = true
  })

  const sendGated = page.waitForResponse((r) => r.url().includes('/send'))

  await page.getByTestId('composer-input').fill('commit and push the branch feat/x')
  await page.getByTestId('composer-launch').click()

  const resp = await sendGated
  const body = (await resp.json()) as { requires_gate?: boolean }
  expect(body.requires_gate, '/send returns requires_gate for a sensitive message').toBe(true)

  // The MUR is restituted; there is no Forcer/Override and the atelier never ran.
  await expect(page.getByTestId('mur')).toBeVisible()
  await expect(page.getByText(/aucun « Forcer »/)).toBeVisible()
  await expect(page.getByTestId('atelier')).toHaveCount(0)

  // 0 spawn: the SSE stream was never opened.
  expect(streamOpened, 'the SSE stream is never opened for a gated intention').toBe(false)
})
