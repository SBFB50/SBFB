/// <reference types="vite/client" />

// Sprint 80 (front rapid-add) — `.po` catalogs are compiled to message modules
// by @lingui/vite-plugin at import time. Declare the shape so `import { messages
// } from './locales/xx.po'` (and the dynamic form in i18n.ts) type-checks.
declare module '*.po' {
  import type { Messages } from '@lingui/core'
  export const messages: Messages
}
