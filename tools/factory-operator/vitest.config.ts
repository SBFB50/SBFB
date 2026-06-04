// SPDX-License-Identifier: AGPL-3.0-or-later
//
// P2-OPERATOR-NO-TEST-RUNNER (S73 Phase B): the factory-operator front shipped
// with no test runner. This mirrors `web/vitest.config.ts` (jsdom + a global
// setup) so the execution-chat logic — the model-picker payload, the SSE
// wiring, the StreamChunk mapping and the requires_gate path — gets unit
// coverage. Kept separate from `vite.config.ts` (which carries the dev proxy
// and bearer-token injection, irrelevant under jsdom).

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
    exclude: ["**/node_modules/**", "**/dist/**"],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary"],
      include: ["src/lib/executionChat.ts", "src/pages/ExecutionChat.tsx"],
    },
  },
});
