// SPDX-License-Identifier: AGPL-3.0-or-later
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { I18nProvider } from '@lingui/react'
// Geist sans/mono are vendored from fontsource via `@import` in
// index.css (CSS-first, mirrors web/) — see the file for the CSP
// rationale (Day-0 D10, 0 CDN).
import './index.css'
import { App } from './App'
import { ErrorBoundary } from './components/ErrorBoundary'
import { i18n, DEFAULT_LOCALE, dynamicActivate } from './i18n/i18n'

// Load + activate the default locale's catalog BEFORE the first paint so the
// shell never flashes message ids. The catalog is a tiny same-origin chunk
// (CSP `default-src 'self'` allows it); a runtime locale switcher (later slice)
// calls `dynamicActivate` again. <I18nProvider> re-renders subscribers
// (<Trans>, useLingui) on every activation.
async function bootstrap() {
  await dynamicActivate(DEFAULT_LOCALE)
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <I18nProvider i18n={i18n}>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </I18nProvider>
    </StrictMode>,
  )
}

void bootstrap()
