// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import {
  ACCESSIBILITY_STORAGE_KEY,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  applyAccessibilityPreferences,
  readStoredAccessibilityPreferences,
  sanitizeAccessibilityPreferences,
} from './accessibility'

describe('accessibility preferences', () => {
  it('sanitizes hostile or unknown stored values', () => {
    expect(
      sanitizeAccessibilityPreferences({
        theme: 'light',
        contrast: 'extreme',
        pointer: 'large',
        textSpacing: 'loose',
        font: 'legible',
        motion: 'spin',
        scale: '125',
        shortcuts: 'on',
      }),
    ).toEqual({
      ...DEFAULT_ACCESSIBILITY_PREFERENCES,
      theme: 'light',
      pointer: 'large',
      textSpacing: 'loose',
      font: 'legible',
      scale: '125',
      shortcuts: 'on',
    })
  })

  it('reads valid storage and falls back on invalid JSON', () => {
    const storage = new Map<string, string>()
    storage.set(ACCESSIBILITY_STORAGE_KEY, '{"theme":"light","contrast":"high"}')
    expect(readStoredAccessibilityPreferences({ getItem: (key) => storage.get(key) ?? null })).toMatchObject({
      theme: 'light',
      contrast: 'high',
    })
    storage.set(ACCESSIBILITY_STORAGE_KEY, '{')
    expect(readStoredAccessibilityPreferences({ getItem: (key) => storage.get(key) ?? null })).toBe(
      DEFAULT_ACCESSIBILITY_PREFERENCES,
    )
  })

  it('applies every user-facing mode on the html element', () => {
    const root = document.createElement('html')
    applyAccessibilityPreferences(
      {
        ...DEFAULT_ACCESSIBILITY_PREFERENCES,
        theme: 'light',
        contrast: 'high',
        pointer: 'large',
        textSpacing: 'loose',
        font: 'legible',
        motion: 'system',
        scale: '112',
      },
      root,
      true,
    )
    expect(root).toHaveAttribute('data-theme', 'light')
    expect(root).toHaveAttribute('data-contrast', 'high')
    expect(root).toHaveAttribute('data-pointer', 'large')
    expect(root).toHaveAttribute('data-text-spacing', 'loose')
    expect(root).toHaveAttribute('data-font', 'legible')
    expect(root).toHaveAttribute('data-motion', 'reduced')
    expect(root).toHaveAttribute('data-scale', '112')
    expect(root).toHaveAttribute('data-shortcuts', 'off')
  })
})
