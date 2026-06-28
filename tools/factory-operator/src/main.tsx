// SPDX-License-Identifier: AGPL-3.0-or-later
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
// Geist sans/mono are vendored from fontsource via `@import` in
// index.css (CSS-first, mirrors web/) — see the file for the CSP
// rationale (Day-0 D10, 0 CDN).
import './index.css'
import { App } from './App'
import { ErrorBoundary } from './components/ErrorBoundary'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
)
