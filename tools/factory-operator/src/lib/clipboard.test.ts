// SPDX-License-Identifier: AGPL-3.0-or-later
import { afterEach, describe, expect, it, vi } from 'vitest'
import { copyText } from './clipboard'

afterEach(() => {
  vi.restoreAllMocks()
  // Reset whatever the test assigned onto navigator.clipboard.
  Object.assign(navigator, { clipboard: undefined })
})

describe('copyText', () => {
  it('copie via navigator.clipboard et renvoie true', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })
    await expect(copyText('hello')).resolves.toBe(true)
    expect(writeText).toHaveBeenCalledWith('hello')
  })

  it('renvoie false sans jeter quand clipboard est absent', async () => {
    Object.assign(navigator, { clipboard: undefined })
    await expect(copyText('x')).resolves.toBe(false)
  })

  it('renvoie false sans jeter quand writeText rejette', async () => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockRejectedValue(new Error('denied')) } })
    await expect(copyText('x')).resolves.toBe(false)
  })
})
