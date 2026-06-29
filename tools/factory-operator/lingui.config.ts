// SPDX-License-Identifier: AGPL-3.0-or-later
import { defineConfig } from '@lingui/cli'

// Sprint 80 (front rapid-add) - i18n via Lingui (override of the in-house
// engine decided 2026-06-28; the design doc section 1.2 already named Lingui v6 the
// credible alternative). Catalogs are `.po`, compiled to eval-free token
// arrays by @lingui/vite-plugin at build time - the Operator CSP is
// `default-src 'self'; connect-src 'self'` with NO 'unsafe-eval'
// (operator_server.rs:354), so a runtime ICU-to-Function compiler is banned;
// Lingui's compiled catalogs are plain data, interpreted by @lingui/core
// without `new Function`/`eval`.
//
// `fr` is the source locale: a missing translation falls back to the source
// message. Gate C enforces key parity across every catalog found in
// src/i18n/locales/, while Gate B fails any locale that has no verdict-word
// guard list.
export default defineConfig({
  sourceLocale: 'fr',
  locales: ['fr', 'en', 'es', 'ar', 'zh', 'ru', 'de', 'nl', 'sv', 'pt-br', 'it', 'ro', 'pl', 'uk', 'cs', 'sr-latn', 'fi', 'hu', 'lt', 'el', 'hy', 'ka', 'he', 'am', 'fa', 'hi', 'bn', 'ur', 'pa', 'gu', 'mr', 'si', 'ta', 'te', 'kn', 'ml', 'tr', 'az', 'kk', 'zh-hant', 'ja', 'ko', 'vi', 'id', 'ms', 'th', 'km', 'my', 'sw', 'ha', 'yo'],
  catalogs: [
    {
      path: '<rootDir>/src/i18n/locales/{locale}',
      include: ['src'],
    },
  ],
})
