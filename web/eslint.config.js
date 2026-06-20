import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['dist', 'coverage']),
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
      // shadcn components export cva variant constants
      // (e.g. `buttonVariants`) alongside the component itself,
      // which is the recommended shadcn v4 pattern. Enabling
      // allowConstantExport keeps fast refresh working without
      // forcing us to split every `*-variants.ts` file out.
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],
    },
  },
  {
    // Vitest globals (describe/it/expect/vi/beforeEach/afterEach)
    // are injected via vitest.config.ts `test.globals=true`, so
    // ESLint must know about them in test files.
    files: ['**/*.{test,spec}.{ts,tsx}', 'src/test/**/*.ts'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
        describe: 'readonly',
        it: 'readonly',
        test: 'readonly',
        expect: 'readonly',
        vi: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
      },
    },
    rules: {
      // Tests freely use `any` for mock shapes — relax here only.
      '@typescript-eslint/no-explicit-any': 'off',
      'react-refresh/only-export-components': 'off',
    },
  },
  {
    // Playwright E2E specs/fixtures, the daemon global setup/teardown,
    // and the Playwright config run under Node (not the browser) and are
    // not React modules. Give them Node + browser globals and silence the
    // component-only fast-refresh rule.
    files: ['e2e/**/*.ts', 'tests/**/*.ts', 'playwright.config.ts'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      'react-refresh/only-export-components': 'off',
    },
  },
])
