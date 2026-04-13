import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Relative base so the built bundle works from any blob-serve path.
export default defineConfig({
  base: "./",
  plugins: [react()],
  build: {
    outDir: "dist",
    assetsDir: "assets",
    rollupOptions: {
      output: {
        // Single-bundle output keeps the zip small and predictable.
        manualChunks: undefined,
      },
    },
  },
});
