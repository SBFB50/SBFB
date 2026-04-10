/**
 * Playwright globalSetup — spawns a real nexus-coordinator
 * subprocess against a hermetic NEXUS_GRID_ROOT directory, then
 * waits for `/health` to return 200 before handing off to the
 * test runner.
 *
 * The coordinator PID is written to `.playwright-state.json`
 * so `global-teardown.ts` can kill the process even if a test
 * fails mid-run. The hermetic root is `tests/.tmp/nexus-grid/`
 * relative to the web/ directory.
 */

import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export const TEST_COORD_URL = "http://127.0.0.1:18765";
export const TEST_COORD_NAME = "pw-demo";
const STATE_FILE = resolve(__dirname, ".playwright-state.json");

async function waitForHealth(url: string, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: string = "";
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${url}/health`);
      if (res.ok) return;
      lastError = `HTTP ${res.status}`;
    } catch (e) {
      lastError = e instanceof Error ? e.message : String(e);
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`coordinator /health did not respond in ${timeoutMs} ms: ${lastError}`);
}

async function initProject(gridRoot: string): Promise<void> {
  return new Promise((resolvePromise, reject) => {
    const proc = spawn(
      "uv",
      [
        "run",
        "--package",
        "nexus-coordinator",
        "nexus-coordinator",
        "init",
        TEST_COORD_NAME,
      ],
      {
        cwd: resolve(__dirname, "../.."),
        env: {
        ...process.env,
        NEXUS_GRID_ROOT: gridRoot,
        // Force Rich / structlog to write utf-8 so the checkmark
        // Rich prints in `init` success output does not crash the
        // subprocess on a Windows cp1252 code page.
        PYTHONIOENCODING: "utf-8",
      },
        stdio: ["ignore", "pipe", "pipe"],
        shell: true,
      },
    );

    let stderr = "";
    proc.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    proc.on("error", reject);
    proc.on("exit", (code) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`init exit ${code}: ${stderr}`));
    });
  });
}

async function globalSetup() {
  const gridRoot = resolve(__dirname, ".tmp/nexus-grid");
  rmSync(gridRoot, { recursive: true, force: true });
  mkdirSync(gridRoot, { recursive: true });

   
  console.log(`[pw] NEXUS_GRID_ROOT=${gridRoot}`);

   
  console.log("[pw] initialising test coordinator project");
  await initProject(gridRoot);

   
  console.log("[pw] spawning nexus-coordinator start");
  const startProc: ChildProcessWithoutNullStreams = spawn(
    "uv",
    [
      "run",
      "--package",
      "nexus-coordinator",
      "nexus-coordinator",
      "start",
      TEST_COORD_NAME,
      "--port",
      "18765",
    ],
    {
      cwd: resolve(__dirname, "../.."),
      env: {
        ...process.env,
        NEXUS_GRID_ROOT: gridRoot,
        // Force Rich / structlog to write utf-8 so the checkmark
        // Rich prints in `init` success output does not crash the
        // subprocess on a Windows cp1252 code page.
        PYTHONIOENCODING: "utf-8",
      },
      stdio: ["ignore", "pipe", "pipe"],
      shell: true,
    },
  );

  startProc.stdout.on("data", (chunk) => {
    process.stdout.write(`[coord stdout] ${chunk}`);
  });
  startProc.stderr.on("data", (chunk) => {
    process.stdout.write(`[coord stderr] ${chunk}`);
  });

  writeFileSync(
    STATE_FILE,
    JSON.stringify({ pid: startProc.pid, gridRoot }, null, 2),
    "utf-8",
  );

  await waitForHealth(TEST_COORD_URL, 30_000);
   
  console.log(`[pw] coordinator ready on ${TEST_COORD_URL}`);
}

export default globalSetup;
