// SPDX-License-Identifier: AGPL-3.0-or-later
export const ACCESSIBILITY_STORAGE_KEY = 'factory-operator.accessibility.v1'
export const ACCESSIBILITY_CHANGE_EVENT = 'factory-operator:accessibility'

export const THEMES = ['dark', 'light'] as const
export const CONTRASTS = ['standard', 'high'] as const
export const POINTERS = ['standard', 'large'] as const
export const TEXT_SPACINGS = ['standard', 'loose'] as const
export const FONTS = ['standard', 'legible'] as const
export const MOTIONS = ['system', 'reduced'] as const
export const SCALES = ['100', '112', '125'] as const
export const SHORTCUTS = ['off', 'on'] as const

export type ThemePreference = (typeof THEMES)[number]
export type ContrastPreference = (typeof CONTRASTS)[number]
export type PointerPreference = (typeof POINTERS)[number]
export type TextSpacingPreference = (typeof TEXT_SPACINGS)[number]
export type FontPreference = (typeof FONTS)[number]
export type MotionPreference = (typeof MOTIONS)[number]
export type ScalePreference = (typeof SCALES)[number]
export type ShortcutPreference = (typeof SHORTCUTS)[number]

export interface AccessibilityPreferences {
  theme: ThemePreference
  contrast: ContrastPreference
  pointer: PointerPreference
  textSpacing: TextSpacingPreference
  font: FontPreference
  motion: MotionPreference
  scale: ScalePreference
  shortcuts: ShortcutPreference
}

export const DEFAULT_ACCESSIBILITY_PREFERENCES: AccessibilityPreferences = {
  theme: 'dark',
  contrast: 'standard',
  pointer: 'standard',
  textSpacing: 'standard',
  font: 'standard',
  motion: 'system',
  scale: '100',
  shortcuts: 'off',
}

const COLOR_KEYS = [
  's0',
  's1',
  's2',
  's3',
  'bd',
  'bd2',
  'field',
  'tx',
  'tx2',
  'tx3',
  'tx4',
  'ok',
  'warn',
  'bad',
  'info',
] as const

const LIGHT_COLORS: Partial<Record<(typeof COLOR_KEYS)[number], string>> = {
  s0: 'oklch(0.975 0.004 260)',
  s1: 'oklch(0.945 0.005 260)',
  s2: 'oklch(0.910 0.006 260)',
  s3: 'oklch(0.870 0.008 260)',
  bd: 'oklch(0.740 0.010 260)',
  bd2: 'oklch(0.610 0.012 260)',
  field: 'oklch(0.500 0.014 260)',
  tx: 'oklch(0.185 0.008 260)',
  tx2: 'oklch(0.315 0.010 260)',
  tx3: 'oklch(0.405 0.010 260)',
  tx4: 'oklch(0.470 0.010 260)',
  ok: 'oklch(0.420 0.130 150)',
  warn: 'oklch(0.470 0.120 78)',
  bad: 'oklch(0.475 0.170 25)',
  info: 'oklch(0.430 0.120 240)',
}

const HIGH_COLORS: Partial<Record<(typeof COLOR_KEYS)[number], string>> = {
  bd: 'oklch(0.560 0.010 260)',
  bd2: 'oklch(0.720 0.012 260)',
  field: 'oklch(0.760 0.014 260)',
  tx: 'oklch(0.985 0.004 260)',
  tx2: 'oklch(0.900 0.005 260)',
  tx3: 'oklch(0.840 0.006 260)',
  tx4: 'oklch(0.785 0.006 260)',
}

const LIGHT_HIGH_COLORS: Partial<Record<(typeof COLOR_KEYS)[number], string>> = {
  bd: 'oklch(0.470 0.012 260)',
  bd2: 'oklch(0.300 0.014 260)',
  field: 'oklch(0.260 0.014 260)',
  tx: 'oklch(0.075 0.006 260)',
  tx2: 'oklch(0.175 0.008 260)',
  tx3: 'oklch(0.250 0.009 260)',
  tx4: 'oklch(0.315 0.010 260)',
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function pick<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value) ? (value as T) : fallback
}

export function sanitizeAccessibilityPreferences(value: unknown): AccessibilityPreferences {
  const source = isRecord(value) ? value : {}
  return {
    theme: pick(source.theme, THEMES, DEFAULT_ACCESSIBILITY_PREFERENCES.theme),
    contrast: pick(source.contrast, CONTRASTS, DEFAULT_ACCESSIBILITY_PREFERENCES.contrast),
    pointer: pick(source.pointer, POINTERS, DEFAULT_ACCESSIBILITY_PREFERENCES.pointer),
    textSpacing: pick(source.textSpacing, TEXT_SPACINGS, DEFAULT_ACCESSIBILITY_PREFERENCES.textSpacing),
    font: pick(source.font, FONTS, DEFAULT_ACCESSIBILITY_PREFERENCES.font),
    motion: pick(source.motion, MOTIONS, DEFAULT_ACCESSIBILITY_PREFERENCES.motion),
    scale: pick(source.scale, SCALES, DEFAULT_ACCESSIBILITY_PREFERENCES.scale),
    shortcuts: pick(source.shortcuts, SHORTCUTS, DEFAULT_ACCESSIBILITY_PREFERENCES.shortcuts),
  }
}

export function readStoredAccessibilityPreferences(storage: Pick<Storage, 'getItem'>): AccessibilityPreferences {
  try {
    const raw = storage.getItem(ACCESSIBILITY_STORAGE_KEY)
    return raw === null ? DEFAULT_ACCESSIBILITY_PREFERENCES : sanitizeAccessibilityPreferences(JSON.parse(raw))
  } catch {
    return DEFAULT_ACCESSIBILITY_PREFERENCES
  }
}

export function writeStoredAccessibilityPreferences(
  storage: Pick<Storage, 'setItem'>,
  preferences: AccessibilityPreferences,
): void {
  try {
    storage.setItem(ACCESSIBILITY_STORAGE_KEY, JSON.stringify(preferences))
  } catch {
    // Storage may be blocked by browser policy; the DOM data-* state still applies for this session.
  }
}

export function resolveMotionPreference(preferences: AccessibilityPreferences, systemReduced: boolean): 'standard' | 'reduced' {
  return preferences.motion === 'reduced' || systemReduced ? 'reduced' : 'standard'
}

function applyColorPreferences(preferences: AccessibilityPreferences, root: HTMLElement): void {
  for (const key of COLOR_KEYS) root.style.removeProperty(`--color-${key}`)
  const colors =
    preferences.theme === 'light'
      ? { ...LIGHT_COLORS, ...(preferences.contrast === 'high' ? LIGHT_HIGH_COLORS : {}) }
      : preferences.contrast === 'high'
        ? HIGH_COLORS
        : {}
  for (const [key, value] of Object.entries(colors)) {
    if (value) root.style.setProperty(`--color-${key}`, value)
  }
}

export function applyAccessibilityPreferences(
  preferences: AccessibilityPreferences,
  root: HTMLElement,
  systemReduced = false,
): void {
  root.setAttribute('data-theme', preferences.theme)
  root.setAttribute('data-contrast', preferences.contrast)
  root.setAttribute('data-pointer', preferences.pointer)
  root.setAttribute('data-text-spacing', preferences.textSpacing)
  root.setAttribute('data-font', preferences.font)
  root.setAttribute('data-motion', resolveMotionPreference(preferences, systemReduced))
  root.setAttribute('data-scale', preferences.scale)
  root.setAttribute('data-shortcuts', preferences.shortcuts)
  applyColorPreferences(preferences, root)
  root.style.colorScheme = preferences.theme
}
