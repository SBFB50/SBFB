/**
 * Playwright globalSetup — spawns a real nexus-shell-daemon
 * subprocess against a hermetic directory, then waits for
 * `/health` to return 200 before handing off to the test runner.
 *
 * The daemon PID is written to `.playwright-state.json`
 * so `global-teardown.ts` can kill the process even if a test
 * fails mid-run. The hermetic root is `tests/.tmp/nexus-grid/`
 * relative to the web/ directory.
 *
 * Sprint 63 Phase A: rewritten from Python coordinator spawn
 * to Rust daemon spawn (Python removed S50-S51).
 */

import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export const TEST_COORD_URL = "http://127.0.0.1:18765";
const STATE_FILE = resolve(__dirname, ".playwright-state.json");
const TEST_PORT = 18765;

/** Sprint 16 Phase A (D1): fixed 64-char hex token shared by the
 *  coordinator subprocess and the Playwright page. */
export const TEST_AUTH_TOKEN =
  "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

function findDaemonBin(): string {
  if (process.env.SBFB_DAEMON_BIN) return process.env.SBFB_DAEMON_BIN;
  const ext = process.platform === "win32" ? ".exe" : "";
  const repoRoot = resolve(__dirname, "../..");
  const release = resolve(repoRoot, `target/release/nexus-shell-daemon${ext}`);
  if (existsSync(release)) return release;
  const debug = resolve(repoRoot, `target/debug/nexus-shell-daemon${ext}`);
  if (existsSync(debug)) return debug;
  return `nexus-shell-daemon${ext}`;
}

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
  throw new Error(`daemon /health did not respond in ${timeoutMs} ms: ${lastError}`);
}

async function initDaemon(configPath: string, daemonBin: string): Promise<void> {
  return new Promise((resolvePromise, reject) => {
    const proc = spawn(daemonBin, ["--config", configPath, "init"], {
      env: { ...process.env, SBFB_AUTH_TOKEN: TEST_AUTH_TOKEN },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stderr = "";
    proc.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    proc.on("error", reject);
    proc.on("exit", (code) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`daemon init exit ${code}: ${stderr}`));
    });
  });
}

async function globalSetup() {
  const gridRoot = resolve(__dirname, ".tmp/nexus-grid");
  rmSync(gridRoot, { recursive: true, force: true });
  mkdirSync(gridRoot, { recursive: true });

  const configPath = resolve(gridRoot, "config.toml");
  writeFileSync(
    configPath,
    `[network]\napi_host = "127.0.0.1"\napi_port = ${TEST_PORT}\n`,
    "utf-8",
  );

  const daemonBin = findDaemonBin();
  console.log(`[pw] daemon binary: ${daemonBin}`);
  console.log(`[pw] hermetic root: ${gridRoot}`);

  console.log("[pw] initialising daemon");
  await initDaemon(configPath, daemonBin);

  console.log("[pw] spawning daemon start");
  const startProc: ChildProcessWithoutNullStreams = spawn(
    daemonBin,
    ["--config", configPath, "start"],
    {
      env: { ...process.env, SBFB_AUTH_TOKEN: TEST_AUTH_TOKEN },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  startProc.stdout.on("data", (chunk) => {
    process.stdout.write(`[daemon stdout] ${chunk}`);
  });
  startProc.stderr.on("data", (chunk) => {
    process.stdout.write(`[daemon stderr] ${chunk}`);
  });

  writeFileSync(
    STATE_FILE,
    JSON.stringify({ pid: startProc.pid, gridRoot }, null, 2),
    "utf-8",
  );

  await waitForHealth(TEST_COORD_URL, 30_000);
  console.log(`[pw] daemon ready on ${TEST_COORD_URL}`);
}

export default globalSetup;
