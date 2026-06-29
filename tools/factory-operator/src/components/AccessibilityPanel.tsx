// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useState, type ReactNode } from 'react'
import type {
  AccessibilityPreferences,
  ContrastPreference,
  FontPreference,
  MotionPreference,
  PointerPreference,
  ScalePreference,
  ShortcutPreference,
  TextSpacingPreference,
  ThemePreference,
} from '../preferences/accessibility'
import {
  ACCESSIBILITY_CHANGE_EVENT,
  applyAccessibilityPreferences,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  readStoredAccessibilityPreferences,
  writeStoredAccessibilityPreferences,
} from '../preferences/accessibility'

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="a11y-field">
      <span>{label}</span>
      {children}
    </label>
  )
}

function systemReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

export function AccessibilityPanel() {
  const [preferences, setPreferences] = useState<AccessibilityPreferences>(() => {
    if (typeof window === 'undefined') return DEFAULT_ACCESSIBILITY_PREFERENCES
    return readStoredAccessibilityPreferences(window.localStorage)
  })

  const setPreference = <K extends keyof AccessibilityPreferences>(key: K, value: AccessibilityPreferences[K]) => {
    setPreferences((current) => ({ ...current, [key]: value }))
  }

  const resetPreferences = () => setPreferences(DEFAULT_ACCESSIBILITY_PREFERENCES)

  useEffect(() => {
    applyAccessibilityPreferences(preferences, document.documentElement, systemReducedMotion())
    writeStoredAccessibilityPreferences(window.localStorage, preferences)
    window.dispatchEvent(new Event(ACCESSIBILITY_CHANGE_EVENT))
  }, [preferences])

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
            <button
              type="button"
              onClick={resetPreferences}
              className="a11y-reset"
            >
              reset
            </button>
          </div>
          <Field label="theme">
            <select
              value={preferences.theme}
              onChange={(event) => setPreference('theme', event.target.value as ThemePreference)}
              className="a11y-select"
            >
              <option value="dark">sombre</option>
              <option value="light">clair</option>
            </select>
          </Field>
          <Field label="contraste">
            <select
              value={preferences.contrast}
              onChange={(event) => setPreference('contrast', event.target.value as ContrastPreference)}
              className="a11y-select"
            >
              <option value="standard">standard</option>
              <option value="high">eleve</option>
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
          <Field label="echelle">
            <select
              value={preferences.scale}
              onChange={(event) => setPreference('scale', event.target.value as ScalePreference)}
              className="a11y-select"
            >
              <option value="100">100%</option>
              <option value="112">112%</option>
              <option value="125">125%</option>
            </select>
          </Field>
          <Field label="raccourcis s/v">
            <select
              value={preferences.shortcuts}
              onChange={(event) => setPreference('shortcuts', event.target.value as ShortcutPreference)}
              className="a11y-select"
            >
              <option value="off">desactives</option>
              <option value="on">actives</option>
            </select>
          </Field>
    </div>
  )
}
