// SPDX-License-Identifier: AGPL-3.0-or-later
export const ACCESSIBILITY_STORAGE_KEY = 'factory-operator.accessibility.v2'
export const LEGACY_ACCESSIBILITY_STORAGE_KEY = 'factory-operator.accessibility.v1'
export const ACCESSIBILITY_CHANGE_EVENT = 'factory-operator:accessibility'

export const NEEDS = [
  'lowVision',
  'blindAssistive',
  'colorVision',
  'noColor',
  'vestibular',
  'photosensitive',
  'motor',
  'cognitive',
  'dyslexia',
  'attention',
  'auditory',
  'speech',
  'sensory',
] as const

export const THEMES = ['auto', 'dark', 'light', 'calm', 'paper', 'forced'] as const
export const RESOLVED_THEMES = ['dark', 'light', 'calm', 'paper', 'forced'] as const
export const CONTRASTS = ['auto', 'standard', 'high'] as const
export const COLOR_VISIONS = ['standard', 'safe', 'monochrome'] as const
export const POINTERS = ['standard', 'large'] as const
export const TEXT_SPACINGS = ['standard', 'loose'] as const
export const FONTS = ['standard', 'legible'] as const
export const MOTIONS = ['system', 'reduced'] as const
export const TRANSPARENCIES = ['standard', 'reduced'] as const
export const DENSITIES = ['standard', 'focus'] as const
export const READINGS = ['standard', 'assist'] as const
export const FOCUS_MODES = ['standard', 'strong'] as const
export const SCALES = ['100', '112', '125', '150'] as const
export const SHORTCUTS = ['off', 'on'] as const
export const ASSISTIVE_TECHS = ['standard', 'screen-reader'] as const
export const CAPTIONS = ['off', 'on'] as const

export type AccessibilityNeed = (typeof NEEDS)[number]
export type ThemePreference = (typeof THEMES)[number]
export type ResolvedThemePreference = (typeof RESOLVED_THEMES)[number]
export type ContrastPreference = (typeof CONTRASTS)[number]
export type ColorVisionPreference = (typeof COLOR_VISIONS)[number]
export type PointerPreference = (typeof POINTERS)[number]
export type TextSpacingPreference = (typeof TEXT_SPACINGS)[number]
export type FontPreference = (typeof FONTS)[number]
export type MotionPreference = (typeof MOTIONS)[number]
export type TransparencyPreference = (typeof TRANSPARENCIES)[number]
export type DensityPreference = (typeof DENSITIES)[number]
export type ReadingPreference = (typeof READINGS)[number]
export type FocusPreference = (typeof FOCUS_MODES)[number]
export type ScalePreference = (typeof SCALES)[number]
export type ShortcutPreference = (typeof SHORTCUTS)[number]
export type AssistiveTechPreference = (typeof ASSISTIVE_TECHS)[number]
export type CaptionPreference = (typeof CAPTIONS)[number]

export interface AccessibilityPreferences {
  needs: AccessibilityNeed[]
  theme: ThemePreference
  contrast: ContrastPreference
  colorVision: ColorVisionPreference
  pointer: PointerPreference
  textSpacing: TextSpacingPreference
  font: FontPreference
  motion: MotionPreference
  transparency: TransparencyPreference
  density: DensityPreference
  reading: ReadingPreference
  focus: FocusPreference
  scale: ScalePreference
  shortcuts: ShortcutPreference
  assistiveTech: AssistiveTechPreference
  captions: CaptionPreference
}

export interface AccessibilitySystemPreferences {
  reducedMotion?: boolean
  highContrast?: boolean
  forcedColors?: boolean
}

export interface ResolvedAccessibilityPreferences extends Omit<AccessibilityPreferences, 'theme' | 'contrast' | 'motion'> {
  theme: ResolvedThemePreference
  contrast: 'standard' | 'high'
  motion: 'standard' | 'reduced'
}

export const DEFAULT_ACCESSIBILITY_PREFERENCES: AccessibilityPreferences = {
  needs: [],
  theme: 'auto',
  contrast: 'auto',
  colorVision: 'standard',
  pointer: 'standard',
  textSpacing: 'standard',
  font: 'standard',
  motion: 'system',
  transparency: 'standard',
  density: 'standard',
  reading: 'standard',
  focus: 'standard',
  scale: '100',
  shortcuts: 'off',
  assistiveTech: 'standard',
  captions: 'off',
}

export const ACCESSIBILITY_NEED_LABELS: Record<AccessibilityNeed, string> = {
  lowVision: 'basse vision',
  blindAssistive: 'cecite / lecteur',
  colorVision: 'daltonisme',
  noColor: 'sans couleur',
  vestibular: 'vestibulaire',
  photosensitive: 'photosensible',
  motor: 'moteur',
  cognitive: 'cognitif',
  dyslexia: 'dyslexie',
  attention: 'attention',
  auditory: 'auditif',
  speech: 'parole',
  sensory: 'sensoriel',
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
  'neu',
  'ok-bg',
  'bad-bg',
] as const

type ColorKey = (typeof COLOR_KEYS)[number]
type ColorMap = Partial<Record<ColorKey, string>>

const LIGHT_COLORS: ColorMap = {
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
  neu: 'oklch(0.405 0.010 260)',
  'ok-bg': 'oklch(0.910 0.030 150)',
  'bad-bg': 'oklch(0.920 0.035 25)',
}

const CALM_COLORS: ColorMap = {
  s0: 'oklch(0.200 0.012 180)',
  s1: 'oklch(0.235 0.012 180)',
  s2: 'oklch(0.275 0.014 180)',
  s3: 'oklch(0.320 0.015 180)',
  bd: 'oklch(0.390 0.012 180)',
  bd2: 'oklch(0.500 0.014 180)',
  field: 'oklch(0.610 0.016 180)',
  tx: 'oklch(0.930 0.006 170)',
  tx2: 'oklch(0.760 0.008 170)',
  tx3: 'oklch(0.700 0.009 170)',
  tx4: 'oklch(0.640 0.010 170)',
  ok: 'oklch(0.760 0.090 155)',
  warn: 'oklch(0.800 0.090 80)',
  bad: 'oklch(0.760 0.105 25)',
  info: 'oklch(0.780 0.070 220)',
  neu: 'oklch(0.700 0.009 170)',
  'ok-bg': 'oklch(0.310 0.032 155)',
  'bad-bg': 'oklch(0.300 0.035 25)',
}

const PAPER_COLORS: ColorMap = {
  s0: 'oklch(0.970 0.006 95)',
  s1: 'oklch(0.945 0.008 95)',
  s2: 'oklch(0.910 0.010 95)',
  s3: 'oklch(0.860 0.012 95)',
  bd: 'oklch(0.720 0.014 95)',
  bd2: 'oklch(0.560 0.016 95)',
  field: 'oklch(0.410 0.018 95)',
  tx: 'oklch(0.160 0.012 80)',
  tx2: 'oklch(0.290 0.014 80)',
  tx3: 'oklch(0.380 0.014 80)',
  tx4: 'oklch(0.455 0.014 80)',
  ok: 'oklch(0.400 0.120 150)',
  warn: 'oklch(0.455 0.115 78)',
  bad: 'oklch(0.455 0.160 25)',
  info: 'oklch(0.390 0.110 240)',
  neu: 'oklch(0.380 0.014 80)',
  'ok-bg': 'oklch(0.900 0.032 150)',
  'bad-bg': 'oklch(0.905 0.036 25)',
}

const FORCED_COLORS: ColorMap = {
  s0: '#000000',
  s1: '#050505',
  s2: '#101010',
  s3: '#1a1a1a',
  bd: '#9f9f9f',
  bd2: '#ffffff',
  field: '#ffffff',
  tx: '#ffffff',
  tx2: '#f2f2f2',
  tx3: '#e6e6e6',
  tx4: '#d8d8d8',
  ok: '#ffffff',
  warn: '#ffff00',
  bad: '#ff6b6b',
  info: '#66ccff',
  neu: '#e6e6e6',
  'ok-bg': '#111111',
  'bad-bg': '#220000',
}

const HIGH_COLORS: ColorMap = {
  bd: 'oklch(0.560 0.010 260)',
  bd2: 'oklch(0.720 0.012 260)',
  field: 'oklch(0.760 0.014 260)',
  tx: 'oklch(0.985 0.004 260)',
  tx2: 'oklch(0.900 0.005 260)',
  tx3: 'oklch(0.840 0.006 260)',
  tx4: 'oklch(0.785 0.006 260)',
}

const LIGHT_HIGH_COLORS: ColorMap = {
  bd: 'oklch(0.470 0.012 260)',
  bd2: 'oklch(0.300 0.014 260)',
  field: 'oklch(0.260 0.014 260)',
  tx: 'oklch(0.075 0.006 260)',
  tx2: 'oklch(0.175 0.008 260)',
  tx3: 'oklch(0.250 0.009 260)',
  tx4: 'oklch(0.315 0.010 260)',
}

const COLOR_SAFE_COLORS: ColorMap = {
  ok: '#009e73',
  warn: '#e69f00',
  bad: '#d55e00',
  info: '#0072b2',
}

const MONOCHROME_COLORS: ColorMap = {
  ok: 'var(--color-tx)',
  warn: 'var(--color-tx2)',
  bad: 'var(--color-tx3)',
  info: 'var(--color-tx2)',
}

const NEED_SET = new Set<string>(NEEDS)
const SCALE_RANK: Record<ScalePreference, number> = { '100': 0, '112': 1, '125': 2, '150': 3 }

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function pick<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value) ? (value as T) : fallback
}

function pickNeeds(value: unknown): AccessibilityNeed[] {
  if (!Array.isArray(value)) return []
  const next: AccessibilityNeed[] = []
  for (const item of value) {
    if (typeof item !== 'string' || !NEED_SET.has(item) || next.includes(item as AccessibilityNeed)) continue
    next.push(item as AccessibilityNeed)
  }
  return next
}

function hasNeed(preferences: Pick<AccessibilityPreferences, 'needs'>, need: AccessibilityNeed): boolean {
  return preferences.needs.includes(need)
}

function hasAnyNeed(preferences: Pick<AccessibilityPreferences, 'needs'>, needs: readonly AccessibilityNeed[]): boolean {
  return needs.some((need) => hasNeed(preferences, need))
}

function atLeastScale(current: ScalePreference, minimum: ScalePreference): ScalePreference {
  return SCALE_RANK[current] >= SCALE_RANK[minimum] ? current : minimum
}

export function sanitizeAccessibilityPreferences(value: unknown): AccessibilityPreferences {
  const source = isRecord(value) ? value : {}
  return {
    needs: pickNeeds(source.needs),
    theme: pick(source.theme, THEMES, DEFAULT_ACCESSIBILITY_PREFERENCES.theme),
    contrast: pick(source.contrast, CONTRASTS, DEFAULT_ACCESSIBILITY_PREFERENCES.contrast),
    colorVision: pick(source.colorVision, COLOR_VISIONS, DEFAULT_ACCESSIBILITY_PREFERENCES.colorVision),
    pointer: pick(source.pointer, POINTERS, DEFAULT_ACCESSIBILITY_PREFERENCES.pointer),
    textSpacing: pick(source.textSpacing, TEXT_SPACINGS, DEFAULT_ACCESSIBILITY_PREFERENCES.textSpacing),
    font: pick(source.font, FONTS, DEFAULT_ACCESSIBILITY_PREFERENCES.font),
    motion: pick(source.motion, MOTIONS, DEFAULT_ACCESSIBILITY_PREFERENCES.motion),
    transparency: pick(source.transparency, TRANSPARENCIES, DEFAULT_ACCESSIBILITY_PREFERENCES.transparency),
    density: pick(source.density, DENSITIES, DEFAULT_ACCESSIBILITY_PREFERENCES.density),
    reading: pick(source.reading, READINGS, DEFAULT_ACCESSIBILITY_PREFERENCES.reading),
    focus: pick(source.focus, FOCUS_MODES, DEFAULT_ACCESSIBILITY_PREFERENCES.focus),
    scale: pick(source.scale, SCALES, DEFAULT_ACCESSIBILITY_PREFERENCES.scale),
    shortcuts: pick(source.shortcuts, SHORTCUTS, DEFAULT_ACCESSIBILITY_PREFERENCES.shortcuts),
    assistiveTech: pick(source.assistiveTech, ASSISTIVE_TECHS, DEFAULT_ACCESSIBILITY_PREFERENCES.assistiveTech),
    captions: pick(source.captions, CAPTIONS, DEFAULT_ACCESSIBILITY_PREFERENCES.captions),
  }
}

export function readStoredAccessibilityPreferences(storage: Pick<Storage, 'getItem'>): AccessibilityPreferences {
  try {
    const raw = storage.getItem(ACCESSIBILITY_STORAGE_KEY) ?? storage.getItem(LEGACY_ACCESSIBILITY_STORAGE_KEY)
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

function resolveTheme(preferences: AccessibilityPreferences, system: AccessibilitySystemPreferences): ResolvedThemePreference {
  if (preferences.theme !== 'auto') return preferences.theme
  if (system.forcedColors || hasAnyNeed(preferences, ['lowVision', 'blindAssistive', 'noColor'])) return 'forced'
  if (hasAnyNeed(preferences, ['vestibular', 'photosensitive', 'attention', 'sensory'])) return 'calm'
  if (hasAnyNeed(preferences, ['cognitive', 'dyslexia'])) return 'paper'
  return 'dark'
}

function resolveContrast(preferences: AccessibilityPreferences, system: AccessibilitySystemPreferences): 'standard' | 'high' {
  if (
    preferences.contrast === 'high' ||
    system.highContrast ||
    system.forcedColors ||
    hasAnyNeed(preferences, ['lowVision', 'blindAssistive', 'colorVision', 'noColor'])
  ) {
    return 'high'
  }
  return 'standard'
}

export function resolveAccessibilityPreferences(
  preferences: AccessibilityPreferences,
  system: AccessibilitySystemPreferences = {},
): ResolvedAccessibilityPreferences {
  let scale = preferences.scale
  let pointer = preferences.pointer
  let textSpacing = preferences.textSpacing
  let font = preferences.font
  let colorVision = preferences.colorVision
  let transparency = preferences.transparency
  let density = preferences.density
  let reading = preferences.reading
  let focus = preferences.focus
  let shortcuts = preferences.shortcuts
  let assistiveTech = preferences.assistiveTech
  let captions = preferences.captions
  let motion: 'standard' | 'reduced' =
    preferences.motion === 'reduced' || system.reducedMotion ? 'reduced' : 'standard'

  if (hasNeed(preferences, 'lowVision')) {
    scale = atLeastScale(scale, '125')
    pointer = 'large'
    focus = 'strong'
  }
  if (hasNeed(preferences, 'blindAssistive')) {
    assistiveTech = 'screen-reader'
    density = 'focus'
    focus = 'strong'
    shortcuts = 'off'
  }
  if (hasNeed(preferences, 'colorVision')) {
    colorVision = colorVision === 'monochrome' ? 'monochrome' : 'safe'
    focus = 'strong'
  }
  if (hasNeed(preferences, 'noColor')) {
    colorVision = 'monochrome'
    focus = 'strong'
  }
  if (hasAnyNeed(preferences, ['vestibular', 'photosensitive', 'attention', 'sensory'])) {
    motion = 'reduced'
    transparency = 'reduced'
  }
  if (hasNeed(preferences, 'motor')) {
    pointer = 'large'
    focus = 'strong'
    shortcuts = 'off'
  }
  if (hasNeed(preferences, 'cognitive')) {
    density = 'focus'
    reading = 'assist'
    textSpacing = 'loose'
    shortcuts = 'off'
  }
  if (hasNeed(preferences, 'dyslexia')) {
    font = 'legible'
    textSpacing = 'loose'
    reading = 'assist'
    scale = atLeastScale(scale, '112')
  }
  if (hasNeed(preferences, 'auditory')) {
    captions = 'on'
  }
  if (hasNeed(preferences, 'speech')) {
    shortcuts = 'off'
  }

  return {
    ...preferences,
    theme: resolveTheme(preferences, system),
    contrast: resolveContrast(preferences, system),
    colorVision,
    pointer,
    textSpacing,
    font,
    motion,
    transparency,
    density,
    reading,
    focus,
    scale,
    shortcuts,
    assistiveTech,
    captions,
  }
}

function applyColorPreferences(preferences: ResolvedAccessibilityPreferences, root: HTMLElement): void {
  for (const key of COLOR_KEYS) root.style.removeProperty(`--color-${key}`)
  const colors: ColorMap = {}
  if (preferences.theme === 'light') Object.assign(colors, LIGHT_COLORS)
  if (preferences.theme === 'calm') Object.assign(colors, CALM_COLORS)
  if (preferences.theme === 'paper') Object.assign(colors, PAPER_COLORS)
  if (preferences.theme === 'forced') Object.assign(colors, FORCED_COLORS)
  if (preferences.contrast === 'high' && preferences.theme === 'dark') Object.assign(colors, HIGH_COLORS)
  if (preferences.contrast === 'high' && preferences.theme === 'light') Object.assign(colors, LIGHT_HIGH_COLORS)
  if (preferences.colorVision === 'safe') Object.assign(colors, COLOR_SAFE_COLORS)
  if (preferences.colorVision === 'monochrome') Object.assign(colors, MONOCHROME_COLORS)
  for (const [key, value] of Object.entries(colors)) {
    if (value) root.style.setProperty(`--color-${key}`, value)
  }
}

export function applyAccessibilityPreferences(
  preferences: AccessibilityPreferences,
  root: HTMLElement,
  system: AccessibilitySystemPreferences | boolean = {},
): ResolvedAccessibilityPreferences {
  const resolved = resolveAccessibilityPreferences(
    preferences,
    typeof system === 'boolean' ? { reducedMotion: system } : system,
  )
  root.setAttribute('data-theme', resolved.theme)
  root.setAttribute('data-contrast', resolved.contrast)
  root.setAttribute('data-color-vision', resolved.colorVision)
  root.setAttribute('data-pointer', resolved.pointer)
  root.setAttribute('data-text-spacing', resolved.textSpacing)
  root.setAttribute('data-font', resolved.font)
  root.setAttribute('data-motion', resolved.motion)
  root.setAttribute('data-transparency', resolved.transparency)
  root.setAttribute('data-density', resolved.density)
  root.setAttribute('data-reading', resolved.reading)
  root.setAttribute('data-focus', resolved.focus)
  root.setAttribute('data-scale', resolved.scale)
  root.setAttribute('data-shortcuts', resolved.shortcuts)
  root.setAttribute('data-assistive-tech', resolved.assistiveTech)
  root.setAttribute('data-captions', resolved.captions)
  root.setAttribute('data-needs', resolved.needs.join(' '))
  applyColorPreferences(resolved, root)
  root.style.colorScheme = resolved.theme === 'light' || resolved.theme === 'paper' ? 'light' : 'dark'
  return resolved
}
