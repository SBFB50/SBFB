// SPDX-License-Identifier: AGPL-3.0-or-later
import { test, expect, type ConsoleMessage, type Page } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

// T1 sub-test (1) + CSP risk #1 (preflight): the BUILT bundle boots
// cookie-authenticated — bootstrap `GET /?token` mints the HttpOnly
// `sbfb_operator` cookie and 303s to `/`, which loads the SPA without a
// 401 — and renders with 0 Content-Security-Policy violation under the
// Operator's self-origin CSP (`default-src 'self'; connect-src 'self'`).
// The other T1 sub-tests live in steer.spec.ts ((2)/(3a)/(3b)/(4), Phases
// C + I) and verify.spec.ts ((5), Phase H).

function collectCspViolations(page: Page): string[] {
  const hits: string[] = []
  page.on('console', (msg: ConsoleMessage) => {
    const text = msg.text()
    if (/content security policy|refused to (load|execute|apply|connect)/i.test(text)) {
      hits.push(text)
    }
  })
  page.on('pageerror', (err) => {
    if (/content security policy/i.test(err.message)) hits.push(err.message)
  })
  return hits
}

test('boots cookie-authenticated and renders the shell, CSP-clean', async ({ page }) => {
  const violations = collectCspViolations(page)

  // Bootstrap with the bearer in the query: the Operator mints the
  // session cookie and 303s to `/` (token dropped from the address bar).
  const resp = await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  expect(resp, 'navigation response').not.toBeNull()
  await expect(page).toHaveURL(/\/$/)

  // The Operator self-origin CSP (Phase A) is actually ENFORCED, not
  // merely un-violated: assert the header is present. Without this a
  // 100% same-origin bundle would report 0 violations even with no CSP
  // at all (review P3-b — make the CSP check non-vacuous).
  expect(
    resp?.headers()['content-security-policy'],
    'self-origin CSP header present on the bootstrap response',
  ).toContain("default-src 'self'")

  // The greenfield shell rendered — its assets loaded behind cookie auth
  // (no 401 under the self-origin CSP).
  await expect(page.getByTestId('operator-rail')).toBeVisible()

  // Reload with NO token in the URL: the HttpOnly cookie alone must
  // re-authenticate (proves the cookie transport, not just the bootstrap).
  const reload = await page.reload()
  expect(reload?.status(), 'reload status under cookie auth').toBeLessThan(400)
  await expect(page.getByTestId('operator-rail')).toBeVisible()

  expect(violations, 'CSP violations on the built bundle').toEqual([])
})

test('bootstrap mints an HttpOnly session cookie and 303-redirects to /', async ({ request }) => {
  // Explicit assertions on the redirect chain + cookie flags (Codex P1-B):
  // the functional test above proves cookie auth end-to-end, but the
  // SECURITY properties (303, HttpOnly, distinct cookie name) deserve a
  // direct check so a regression in the bootstrap handler is caught.
  const boot = await request.get(`/?token=${OPERATOR_TEST_TOKEN}`, { maxRedirects: 0 })
  expect(boot.status(), 'bootstrap returns 303 See Other').toBe(303)
  expect(boot.headers()['location'], '303 Location is /').toBe('/')
  const setCookie = boot.headers()['set-cookie'] ?? ''
  expect(setCookie, 'Set-Cookie carries the sbfb_operator session cookie').toContain('sbfb_operator=')
  expect(setCookie.toLowerCase(), 'session cookie is HttpOnly').toContain('httponly')
})
