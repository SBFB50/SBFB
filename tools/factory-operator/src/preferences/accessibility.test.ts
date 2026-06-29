// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest'
import {
  ACCESSIBILITY_STORAGE_KEY,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  LEGACY_ACCESSIBILITY_STORAGE_KEY,
  applyAccessibilityPreferences,
  readStoredAccessibilityPreferences,
  resolveAccessibilityPreferences,
  sanitizeAccessibilityPreferences,
} from './accessibility'

describe('accessibility preferences', () => {
  it('sanitizes hostile or unknown stored values', () => {
    expect(
      sanitizeAccessibilityPreferences({
        needs: ['lowVision', 'unknown', 'lowVision', 'motor'],
        theme: 'light',
        contrast: 'extreme',
        colorVision: 'safe',
        pointer: 'large',
        textSpacing: 'loose',
        font: 'legible',
        motion: 'spin',
        transparency: 'glass',
        density: 'focus',
        reading: 'assist',
        focus: 'strong',
        scale: '150',
        shortcuts: 'on',
        assistiveTech: 'screen-reader',
        captions: 'on',
      }),
    ).toEqual({
      ...DEFAULT_ACCESSIBILITY_PREFERENCES,
      needs: ['lowVision', 'motor'],
      theme: 'light',
      colorVision: 'safe',
      pointer: 'large',
      textSpacing: 'loose',
      font: 'legible',
      density: 'focus',
      reading: 'assist',
      focus: 'strong',
      scale: '150',
      shortcuts: 'on',
      assistiveTech: 'screen-reader',
      captions: 'on',
    })
  })

  it('reads v2 storage, migrates v1 shape, and falls back on invalid JSON', () => {
    const storage = new Map<string, string>()
    storage.set(ACCESSIBILITY_STORAGE_KEY, '{"theme":"paper","contrast":"high","needs":["dyslexia"]}')
    expect(readStoredAccessibilityPreferences({ getItem: (key) => storage.get(key) ?? null })).toMatchObject({
      theme: 'paper',
      contrast: 'high',
      needs: ['dyslexia'],
    })
    storage.clear()
    storage.set(LEGACY_ACCESSIBILITY_STORAGE_KEY, '{"theme":"light","contrast":"high"}')
    expect(readStoredAccessibilityPreferences({ getItem: (key) => storage.get(key) ?? null })).toMatchObject({
      theme: 'light',
      contrast: 'high',
      needs: [],
    })
    storage.set(ACCESSIBILITY_STORAGE_KEY, '{')
    expect(readStoredAccessibilityPreferences({ getItem: (key) => storage.get(key) ?? null })).toBe(
      DEFAULT_ACCESSIBILITY_PREFERENCES,
    )
  })

  it('stacks disability needs instead of letting the last profile win', () => {
    const resolved = resolveAccessibilityPreferences(
      {
        ...DEFAULT_ACCESSIBILITY_PREFERENCES,
        needs: ['lowVision', 'dyslexia', 'motor', 'photosensitive', 'auditory'],
      },
      { reducedMotion: false },
    )

    expect(resolved.theme).toBe('forced')
    expect(resolved.contrast).toBe('high')
    expect(resolved.scale).toBe('125')
    expect(resolved.pointer).toBe('large')
    expect(resolved.font).toBe('legible')
    expect(resolved.textSpacing).toBe('loose')
    expect(resolved.reading).toBe('assist')
    expect(resolved.motion).toBe('reduced')
    expect(resolved.transparency).toBe('reduced')
    expect(resolved.focus).toBe('strong')
    expect(resolved.shortcuts).toBe('off')
    expect(resolved.captions).toBe('on')
  })

  it('applies resolved modes on the html element', () => {
    const root = document.createElement('html')
    const resolved = applyAccessibilityPreferences(
      {
        ...DEFAULT_ACCESSIBILITY_PREFERENCES,
        needs: ['colorVision', 'cognitive'],
        theme: 'auto',
        contrast: 'auto',
        motion: 'system',
        scale: '112',
        shortcuts: 'on',
      },
      root,
      { reducedMotion: false, highContrast: false, forcedColors: false },
    )

    expect(resolved.theme).toBe('paper')
    expect(root).toHaveAttribute('data-theme', 'paper')
    expect(root).toHaveAttribute('data-contrast', 'high')
    expect(root).toHaveAttribute('data-color-vision', 'safe')
    expect(root).toHaveAttribute('data-pointer', 'standard')
    expect(root).toHaveAttribute('data-text-spacing', 'loose')
    expect(root).toHaveAttribute('data-reading', 'assist')
    expect(root).toHaveAttribute('data-density', 'focus')
    expect(root).toHaveAttribute('data-motion', 'standard')
    expect(root).toHaveAttribute('data-scale', '112')
    expect(root).toHaveAttribute('data-shortcuts', 'off')
    expect(root).toHaveAttribute('data-needs', 'colorVision cognitive')
  })
})
