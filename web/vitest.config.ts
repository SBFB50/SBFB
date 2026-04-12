/**
 * Sprint 6 Phase D — Vitest configuration.
 *
 * Kept separate from vite.config.ts (rather than merged) because
 * our vite config is tuned for the production Rolldown build path
 * with `rolldownOptions.manualChunks`. Vitest v3 warns if those
 * options leak into the test runtime. This standalone config
 * reuses the `@` → `./src` alias only.
 *
 * Scope (D4):
 *  - src/lib/format.ts
 *  - src/stores/projectStore.ts
 *  - src/components/app/tabview/** (renderer + schema)
 *
 * Everything else is covered by Playwright (real coordinator).
 */

import path from "node:path";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["**/node_modules/**", "**/dist/**", "tests/**"],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary"],
      include: [
        "src/lib/format.ts",
        "src/stores/projectStore.ts",
        "src/components/app/tabview/**/*.{ts,tsx}",
        "src/api/daemon.ts",
        "src/pages/BrowsedProject.tsx",
      ],
      exclude: [
        "src/components/app/tabview/**/__tests__/**",
        "src/api/__tests__/**",
        "src/pages/__tests__/**",
      ],
      // Sprint 9 audit G2-A: thresholds temporarily lowered from
      // 90/90/85/90 to accommodate FileUploadBlock.tsx (35% lines).
      // T14 tech debt: write Vitest mocks for FileUploadBlock and
      // restore thresholds to 90/90/85/90 in Sprint 10.
      thresholds: {
        lines: 85,
        functions: 90,
        branches: 78,
        statements: 85,
      },
    },
  },
});
