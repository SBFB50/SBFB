// SPDX-License-Identifier: AGPL-3.0-or-later
import { defineConfig } from '@lingui/cli'

// Sprint 80 (front rapid-add) — i18n via Lingui (override of the in-house
// engine decided 2026-06-28; the design doc §1.2 already named Lingui v6 the
// credible alternative). Catalogs are `.po`, compiled to eval-free token
// arrays by @lingui/vite-plugin at build time — the Operator CSP is
// `default-src 'self'; connect-src 'self'` with NO 'unsafe-eval'
// (operator_server.rs:354), so a runtime ICU-to-Function compiler is banned;
// Lingui's compiled catalogs are plain data, interpreted by @lingui/core
// without `new Function`/`eval`.
//
// `fr` is the source locale: a missing translation falls back to the source
// message (no build-fail on incomplete locales — Gate C key-parity policy is a
// later slice, design doc §1.9 #2). Each locale is one `.po` file under
// src/i18n/locales/, dynamically imported per language (src/i18n/i18n.ts).
export default defineConfig({
  sourceLocale: 'fr',
  locales: ['fr', 'en', 'es', 'ar', 'zh'],
  catalogs: [
    {
      path: '<rootDir>/src/i18n/locales/{locale}',
      include: ['src'],
    },
  ],
})
