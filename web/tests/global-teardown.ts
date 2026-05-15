/**
 * Playwright globalTeardown — kills the daemon subprocess
 * spawned by `global-setup.ts`. Reads the PID from the state
 * file and issues `process.kill` (or taskkill on Windows).
 */

import { existsSync, readFileSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const STATE_FILE = resolve(__dirname, ".playwright-state.json");

async function globalTeardown() {
  if (!existsSync(STATE_FILE)) {
     
    console.log("[pw] no state file; nothing to tear down");
    return;
  }
  const { pid, gridRoot } = JSON.parse(readFileSync(STATE_FILE, "utf-8")) as {
    pid: number;
    gridRoot: string;
  };

   
  console.log(`[pw] killing daemon pid=${pid}`);
  if (process.platform === "win32") {
    // /T kills the entire process tree. /F is force.
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
      stdio: "inherit",
    });
  } else {
    try {
      process.kill(pid);
    } catch (e) {
       
      console.log(`[pw] kill failed: ${e}`);
    }
  }

  // Give the process a beat to release the port before deleting
  // its data dir, otherwise Windows complains about file locks.
  await new Promise((r) => setTimeout(r, 500));

  try {
    rmSync(gridRoot, { recursive: true, force: true });
  } catch {
    /* tolerate stragglers */
  }
  try {
    rmSync(STATE_FILE);
  } catch {
    /* tolerate */
  }
}

export default globalTeardown;
