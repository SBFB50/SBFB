import fs from "node:fs";
import os from "node:os";
import nodePath from "node:path";
import path from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// G7 (D5): the Operator server now requires the loopback bearer
// token on every route. The browser must not read ~/.sbfb, so the
// Vite dev proxy — a trusted server-to-server client of :3001 —
// injects `X-SBFB-Token` on each proxied request. Source order:
// SBFB_AUTH_TOKEN env, then <SBFB_HOME|~>/.sbfb/auth_token.
function operatorToken(): string {
  const env = process.env.SBFB_AUTH_TOKEN;
  if (env && env.trim()) return env.trim();
  try {
    const home = process.env.SBFB_HOME || nodePath.join(os.homedir(), ".sbfb");
    return fs.readFileSync(nodePath.join(home, "auth_token"), "utf8").trim();
  } catch {
    return "";
  }
}

const OPERATOR_TOKEN = operatorToken();

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
        configure: (proxy) => {
          proxy.on("proxyReqWs", (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader("x-sbfb-token", OPERATOR_TOKEN);
          });
        },
      },
      "/api": {
        target: "http://127.0.0.1:3001",
        configure: (proxy) => {
          proxy.on("proxyReq", (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader("x-sbfb-token", OPERATOR_TOKEN);
          });
        },
      },
    },
  },
});
