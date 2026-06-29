// SPDX-License-Identifier: AGPL-3.0-or-later
import { expect, test } from '@playwright/test'
import { OPERATOR_TEST_TOKEN } from './fixtures'

test('Documents inspector restitutes the git-backed file map and pinned LLM inputs', async ({ page }) => {
  await page.goto(`/?token=${OPERATOR_TEST_TOKEN}`)
  const documentsResponse = page.waitForResponse(
    (r) => r.url().includes('/api/project-documents') && r.status() === 200,
    { timeout: 30_000 },
  )
  await page.getByTestId('rail-surface-documents').click()
  const payload = await (await documentsResponse).json()
  expect(payload.pinned.some((p: { path: string }) => p.path === 'prompts/agent/base.md')).toBe(true)

  await expect(page.getByTestId('documents-surface')).toBeVisible()
  await expect(page.getByTestId('documents-pinned')).toContainText('prompts/agent/base.md', { timeout: 30_000 })
  await expect(page.getByTestId('documents-surface')).toContainText(/fichiers/)
  await expect(page.getByTestId('project-document-card').filter({ hasText: 'AGENTS.md' }).first()).toBeVisible()
})
