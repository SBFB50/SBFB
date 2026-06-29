// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useMemo, useState, type ReactNode } from 'react'
import type {
  AccessibilityNeed,
  AccessibilityPreferences,
  AssistiveTechPreference,
  CaptionPreference,
  ColorVisionPreference,
  ContrastPreference,
  DensityPreference,
  FocusPreference,
  FontPreference,
  MotionPreference,
  PointerPreference,
  ReadingPreference,
  ScalePreference,
  ShortcutPreference,
  TextSpacingPreference,
  ThemePreference,
  TransparencyPreference,
} from '../preferences/accessibility'
import {
  ACCESSIBILITY_CHANGE_EVENT,
  ACCESSIBILITY_NEED_LABELS,
  applyAccessibilityPreferences,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  NEEDS,
  readStoredAccessibilityPreferences,
  resolveAccessibilityPreferences,
  writeStoredAccessibilityPreferences,
} from '../preferences/accessibility'

const NEED_GROUPS: ReadonlyArray<{ label: string; needs: readonly AccessibilityNeed[] }> = [
  { label: 'Perception', needs: ['lowVision', 'blindAssistive', 'colorVision', 'noColor'] },
  { label: 'Mouvement', needs: ['vestibular', 'photosensitive', 'motor'] },
  { label: 'Lecture', needs: ['cognitive', 'dyslexia', 'attention', 'sensory'] },
  { label: 'Communication', needs: ['auditory', 'speech'] },
]

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="a11y-field">
      <span>{label}</span>
      {children}
    </label>
  )
}

function systemMatches(query: string): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
  return window.matchMedia(query).matches
}

function systemPreferences() {
  return {
    reducedMotion: systemMatches('(prefers-reduced-motion: reduce)'),
    highContrast: systemMatches('(prefers-contrast: more)'),
    forcedColors: systemMatches('(forced-colors: active)'),
  }
}

export function AccessibilityPanel() {
  const [preferences, setPreferences] = useState<AccessibilityPreferences>(() => {
    if (typeof window === 'undefined') return DEFAULT_ACCESSIBILITY_PREFERENCES
    return readStoredAccessibilityPreferences(window.localStorage)
  })
  const [system] = useState(systemPreferences)
  const selectedNeeds = useMemo(() => new Set(preferences.needs), [preferences.needs])
  const resolved = resolveAccessibilityPreferences(preferences, system)

  const setPreference = <K extends keyof AccessibilityPreferences>(key: K, value: AccessibilityPreferences[K]) => {
    setPreferences((current) => ({ ...current, [key]: value }))
  }

  const toggleNeed = (need: AccessibilityNeed, enabled: boolean) => {
    setPreferences((current) => ({
      ...current,
      needs: enabled
        ? NEEDS.filter((candidate) => candidate === need || current.needs.includes(candidate))
        : current.needs.filter((candidate) => candidate !== need),
    }))
  }

  const resetPreferences = () => setPreferences(DEFAULT_ACCESSIBILITY_PREFERENCES)

  useEffect(() => {
    applyAccessibilityPreferences(preferences, document.documentElement, system)
    writeStoredAccessibilityPreferences(window.localStorage, preferences)
    window.dispatchEvent(new Event(ACCESSIBILITY_CHANGE_EVENT))
  }, [preferences, system])

  return (
    <div
      id="accessibility-panel"
      data-testid="accessibility-panel"
      role="dialog"
      aria-label="Preferences d'accessibilite"
      className="a11y-panel"
    >
      <div className="a11y-panel-head">
        <span className="eyebrow">accessibilite</span>
        <button type="button" onClick={resetPreferences} className="a11y-reset">
          reset
        </button>
      </div>

      <div className="a11y-resolved" aria-live="polite">
        actif : {resolved.theme} / {resolved.contrast} / {resolved.scale}%
      </div>

      <div className="a11y-section">
        {NEED_GROUPS.map((group) => (
          <fieldset className="a11y-need-group" key={group.label}>
            <legend>{group.label}</legend>
            <div className="a11y-need-grid">
              {group.needs.map((need) => (
                <label className="a11y-need" key={need}>
                  <input
                    type="checkbox"
                    checked={selectedNeeds.has(need)}
                    onChange={(event) => toggleNeed(need, event.currentTarget.checked)}
                  />
                  <span>{ACCESSIBILITY_NEED_LABELS[need]}</span>
                </label>
              ))}
            </div>
          </fieldset>
        ))}
      </div>

      <div className="a11y-section a11y-grid">
        <Field label="theme">
          <select
            value={preferences.theme}
            onChange={(event) => setPreference('theme', event.target.value as ThemePreference)}
            className="a11y-select"
          >
            <option value="auto">auto</option>
            <option value="dark">sombre</option>
            <option value="light">clair</option>
            <option value="calm">calme</option>
            <option value="paper">lecture</option>
            <option value="forced">force</option>
          </select>
        </Field>
        <Field label="contraste">
          <select
            value={preferences.contrast}
            onChange={(event) => setPreference('contrast', event.target.value as ContrastPreference)}
            className="a11y-select"
          >
            <option value="auto">auto</option>
            <option value="standard">standard</option>
            <option value="high">eleve</option>
          </select>
        </Field>
        <Field label="couleur">
          <select
            value={preferences.colorVision}
            onChange={(event) => setPreference('colorVision', event.target.value as ColorVisionPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="safe">distincte</option>
            <option value="monochrome">mono</option>
          </select>
        </Field>
        <Field label="texte">
          <select
            value={preferences.font}
            onChange={(event) => setPreference('font', event.target.value as FontPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="legible">lisible</option>
          </select>
        </Field>
        <Field label="espacement">
          <select
            value={preferences.textSpacing}
            onChange={(event) => setPreference('textSpacing', event.target.value as TextSpacingPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="loose">aere</option>
          </select>
        </Field>
        <Field label="lecture">
          <select
            value={preferences.reading}
            onChange={(event) => setPreference('reading', event.target.value as ReadingPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="assist">assistee</option>
          </select>
        </Field>
        <Field label="cibles">
          <select
            value={preferences.pointer}
            onChange={(event) => setPreference('pointer', event.target.value as PointerPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="large">grandes</option>
          </select>
        </Field>
        <Field label="focus">
          <select
            value={preferences.focus}
            onChange={(event) => setPreference('focus', event.target.value as FocusPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="strong">fort</option>
          </select>
        </Field>
        <Field label="mouvement">
          <select
            value={preferences.motion}
            onChange={(event) => setPreference('motion', event.target.value as MotionPreference)}
            className="a11y-select"
          >
            <option value="system">systeme</option>
            <option value="reduced">reduit</option>
          </select>
        </Field>
        <Field label="transparence">
          <select
            value={preferences.transparency}
            onChange={(event) => setPreference('transparency', event.target.value as TransparencyPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="reduced">reduite</option>
          </select>
        </Field>
        <Field label="densite">
          <select
            value={preferences.density}
            onChange={(event) => setPreference('density', event.target.value as DensityPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="focus">focus</option>
          </select>
        </Field>
        <Field label="echelle">
          <select
            value={preferences.scale}
            onChange={(event) => setPreference('scale', event.target.value as ScalePreference)}
            className="a11y-select"
          >
            <option value="100">100%</option>
            <option value="112">112%</option>
            <option value="125">125%</option>
            <option value="150">150%</option>
          </select>
        </Field>
        <Field label="raccourcis">
          <select
            value={preferences.shortcuts}
            onChange={(event) => setPreference('shortcuts', event.target.value as ShortcutPreference)}
            className="a11y-select"
          >
            <option value="off">desactives</option>
            <option value="on">actives</option>
          </select>
        </Field>
        <Field label="lecteur">
          <select
            value={preferences.assistiveTech}
            onChange={(event) => setPreference('assistiveTech', event.target.value as AssistiveTechPreference)}
            className="a11y-select"
          >
            <option value="standard">standard</option>
            <option value="screen-reader">renforce</option>
          </select>
        </Field>
        <Field label="sous-titres">
          <select
            value={preferences.captions}
            onChange={(event) => setPreference('captions', event.target.value as CaptionPreference)}
            className="a11y-select"
          >
            <option value="off">off</option>
            <option value="on">on</option>
          </select>
        </Field>
      </div>
    </div>
  )
}
