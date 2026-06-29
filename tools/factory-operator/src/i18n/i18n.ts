// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — Lingui bootstrap. The engine (@lingui/core +
// @lingui/react) is pinned to the EAGER `vendor-i18n` chunk (vite.config.ts
// manualChunks) so its cost is measured by its own size-limit entry and never
// silently inflates the 45 KB `app` hero. The per-locale catalogs stay LAZY:
// `dynamicActivate` code-splits the `.po` import so only the active language's
// chunk is fetched (`default-src 'self'` allows the same-origin chunk).
import { i18n } from '@lingui/core'

// The supported locales + their endonyms. The UI derives its language list from
// this single source (README §6.9 named-constant rule) — adding a locale here +
// its `.po` is the only change needed; `dir` is resolved from the code below.
export const LOCALES = {
  fr: 'Français',
  en: 'English',
  es: 'Español',
  ar: 'العربية',
  zh: '中文',
} as const

export type Locale = keyof typeof LOCALES
export const DEFAULT_LOCALE: Locale = 'fr'

// Right-to-left scripts (design doc §1.3 static fallback). Kept as a plain set
// rather than `Intl.Locale.prototype.getTextInfo` for determinism + zero
// browser-API variance; the i18n surface is small and curated.
const RTL_LOCALES = new Set(['ar', 'he', 'fa', 'ur', 'ps', 'sd', 'ug', 'yi'])

/** Text direction of a locale (CSP-safe, no eval). */
export function directionOf(locale: string): 'ltr' | 'rtl' {
  const base = locale.split('-')[0]
  return RTL_LOCALES.has(base) ? 'rtl' : 'ltr'
}

/**
 * Lazily load + activate one locale catalog. The `.po` import is code-split per
 * language, so switching locales fetches only that language's chunk. Also
 * reflects the language + direction on `<html>` (the document `lang` for
 * assistive tech, `dir` for RTL mirroring — `index.html` ships `lang="fr"` but
 * no `dir`, so this is where direction first lands).
 */
export async function dynamicActivate(locale: Locale): Promise<void> {
  const { messages } = await import(`./locales/${locale}.po`)
  i18n.load(locale, messages)
  i18n.activate(locale)
  const el = document.documentElement
  el.lang = locale
  el.dir = directionOf(locale)
}

export { i18n }
