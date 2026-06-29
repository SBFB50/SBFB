// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) - Lingui bootstrap. The engine (@lingui/core +
// @lingui/react) is pinned to the EAGER `vendor-i18n` chunk (vite.config.ts
// manualChunks) so its cost is measured by its own size-limit entry and never
// silently inflates the 45 KB `app` hero. The per-locale catalog loader lives
// in catalogs.ts so the 51-locale import map stays outside the app chunk.
import { i18n } from '@lingui/core'

// The supported locales + their endonyms. The UI derives its language list from
// this single source (README section 6.9 named-constant rule) - adding a locale here +
// its `.po` is the only change needed; `dir` is resolved from the code below.
export const LOCALES = {
  'fr': "Français",
  'en': "English",
  'es': "Español",
  'ar': "العربية",
  'zh': "中文",
  'ru': "Русский",
  'de': "Deutsch",
  'nl': "Nederlands",
  'sv': "Svenska",
  'pt-br': "Português (Brasil)",
  'it': "Italiano",
  'ro': "Română",
  'pl': "Polski",
  'uk': "Українська",
  'cs': "Čeština",
  'sr-latn': "Srpski",
  'fi': "Suomi",
  'hu': "Magyar",
  'lt': "Lietuvių",
  'el': "Ελληνικά",
  'hy': "Հայերեն",
  'ka': "ქართული",
  'he': "עברית",
  'am': "አማርኛ",
  'fa': "فارسی",
  'hi': "हिन्दी",
  'bn': "বাংলা",
  'ur': "اردو",
  'pa': "ਪੰਜਾਬੀ",
  'gu': "ગુજરાતી",
  'mr': "मराठी",
  'si': "සිංහල",
  'ta': "தமிழ்",
  'te': "తెలుగు",
  'kn': "ಕನ್ನಡ",
  'ml': "മലയാളം",
  'tr': "Türkçe",
  'az': "Azərbaycan dili",
  'kk': "Қазақ тілі",
  'zh-hant': "繁體中文",
  'ja': "日本語",
  'ko': "한국어",
  'vi': "Tiếng Việt",
  'id': "Bahasa Indonesia",
  'ms': "Bahasa Melayu",
  'th': "ไทย",
  'km': "ខ្មែរ",
  'my': "မြန်မာ",
  'sw': "Kiswahili",
  'ha': "Hausa",
  'yo': "Yorùbá",
} as const

export type Locale = keyof typeof LOCALES
export const DEFAULT_LOCALE: Locale = 'fr'

// Right-to-left scripts from the curated locale set. Kept as a plain set rather
// than `Intl.Locale.prototype.getTextInfo` for determinism + zero browser-API
// variance in the accessibility-critical direction path.
const RTL_LOCALES = new Set(['ar', 'he', 'fa', 'ur'])

/** Text direction of a locale (CSP-safe, no eval). */
export function directionOf(locale: string): 'ltr' | 'rtl' {
  const base = locale.toLowerCase().split('-')[0]
  return RTL_LOCALES.has(locale.toLowerCase()) || RTL_LOCALES.has(base) ? 'rtl' : 'ltr'
}

export { i18n }
