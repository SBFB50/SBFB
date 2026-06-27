// SPDX-License-Identifier: AGPL-3.0-or-later
import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

// Sprint 80 Phase B — discipline gates carried at the lint layer:
//
// gate (4) anti-`motion.*`-nu (Day-0 D4): the SINGLE motion entrypoint
//   is `m` (from 'motion/react-m') under <LazyMotion features={domAnimation}>.
//   The full `motion` export (~34 kb) and `<motion.*>` JSX bust the hero
//   budget (~4.6 kb LazyMotion+m). Two BLOCKING rules forbid both;
//   `<LazyMotion strict>` is the runtime twin. Allowed: LazyMotion,
//   MotionConfig, AnimatePresence, domAnimation (from 'motion/react'),
//   and `m` (from 'motion/react-m').
//
// gate (1) anti-`@radix-ui`-runtime (Day-0 D3): Base UI is the SOLE
//   runtime primitive dependency. No `@radix-ui/*` import survives in
//   src/. (The package.json + production-tree layers live in
//   scripts/check-no-radix-runtime.sh.)
export default defineConfig([
  globalIgnores(['bundle', 'dist', 'node_modules', 'playwright-report', 'test-results']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    rules: {
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: 'motion/react',
              importNames: ['motion'],
              message:
                "Sprint 80 D4: import `m` from 'motion/react-m' under <LazyMotion features={domAnimation}>, never the full `motion` export (hero size budget).",
            },
          ],
          patterns: [
            {
              group: ['@radix-ui/*'],
              message:
                'Sprint 80 D3: Base UI (@base-ui/react) is the sole runtime primitive dep; no @radix-ui at runtime (gate 1).',
            },
          ],
        },
      ],
      'no-restricted-syntax': [
        'error',
        {
          selector: "JSXMemberExpression[object.name='motion']",
          message:
            "Sprint 80 D4: use <m.*> from 'motion/react-m', never <motion.*> (hero LazyMotion budget).",
        },
        {
          // `no-restricted-imports` does not lint dynamic import(); these
          // ImportExpression selectors close the dynamic-import bypass of
          // gates 1 and 4 (Codex P1).
          selector: "ImportExpression[source.value=/^@radix-ui\\//]",
          message:
            'Sprint 80 D3: no @radix-ui at runtime, including dynamic import() (gate 1).',
        },
        {
          selector: "ImportExpression[source.value='motion/react']",
          message:
            "Sprint 80 D4: import `m` from 'motion/react-m'; never dynamic import('motion/react') (hero budget).",
        },
      ],
    },
  },
])
