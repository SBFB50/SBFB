import path from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5174,
    proxy: {
      "/api/terminal/ws": {
        target: "ws://127.0.0.1:3001",
        ws: true,
      },
      "/api": "http://127.0.0.1:3001",
    },
  },
});
