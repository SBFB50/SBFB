// SPDX-License-Identifier: AGPL-3.0-or-later
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App'
import { fetchAuthToken, primeAuthToken } from './api/auth'

// Sprint 16 Phase A (D1): resolve the loopback bearer token
// before mounting the app so every API call has it cached. The
// token is returned by the launcher on 127.0.0.1:<ephemeral>.
// Playwright's `page.addInitScript` can pre-seed the cache via
// `window.__SBFB_AUTH_TOKEN` so E2E tests do not depend on a
// live launcher.
const seededToken = (
  window as unknown as { __SBFB_AUTH_TOKEN?: unknown }
).__SBFB_AUTH_TOKEN
if (typeof seededToken === 'string' && seededToken.length === 64) {
  primeAuthToken(seededToken)
} else {
  void fetchAuthToken().catch((err) => {
    console.warn('[sbfb] could not resolve launcher token:', err)
  })
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
