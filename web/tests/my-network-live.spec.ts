/**
 * Sprint 5 Phase C — /my-network live worker card.
 *
 * Writes a fresh worker state.json fixture under the hermetic
 * NEXUS_GRID_ROOT that globalSetup provisioned, then loads
 * /my-network and asserts the shell renders the live cards
 * (identity, GPU, projects served, last task). The coordinator
 * proxy is what the shell hits — we don't spawn a real Rust
 * worker here because `test_worker_state_roundtrip.py` already
 * covers that path end-to-end at the Python layer.
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const GRID_ROOT = resolve(__dirname, ".tmp/nexus-grid");

function writeFreshSnapshot() {
  const dir = resolve(GRID_ROOT, "worker");
  mkdirSync(dir, { recursive: true });
  const nowIso = new Date().toISOString();
  const body = {
    schema_version: 1,
    node_id: "a".repeat(64),
    worker_version: "0.1.0-test",
    uptime_secs: 123,
    started_at: nowIso,
    last_updated_at: nowIso,
    gpu: {
      name: "NVIDIA GeForce RTX 5080",
      memory_total_mb: 16384,
      memory_used_mb: 5123,
      utilization_pct: 42,
      temperature_c: 61,
      power_draw_w: 180,
    },
    projects_served: [
      {
        project_name: "demo",
        doc_id: "deadbeef".repeat(8),
        kudos_total: 0,
        tasks_completed: 7,
      },
    ],
    last_task: {
      task_id: "t-" + "1".repeat(16),
      project_name: "demo",
      prompt_preview: "Hello from Playwright",
      status: "completed",
      completed_at: nowIso,
    },
  };
  writeFileSync(resolve(dir, "state.json"), JSON.stringify(body, null, 2));
}

test.beforeEach(async ({ page }) => {
  writeFreshSnapshot();
  await page.addInitScript(
    ([url, nickname]) => {
      window.localStorage.setItem(
        "nexus-grid:shell:v1",
        JSON.stringify({
          state: {
            knownCoordinators: [{ url, nickname, nodeId: null }],
            activeCoordinatorUrl: url,
          },
          version: 0,
        }),
      );
    },
    [TEST_COORD_URL, TEST_COORD_NAME],
  );
});

test("my-network renders live worker cards from the state snapshot", async ({
  page,
}) => {
  await page.goto("/my-network");

  await expect(
    page.getByRole("heading", { name: "Mon réseau" }),
  ).toBeVisible();

  // The four cards from the live snapshot
  await expect(page.getByText("Identité du worker")).toBeVisible({
    timeout: 10_000,
  });
  await expect(page.getByText("NVIDIA GeForce RTX 5080")).toBeVisible();
  await expect(page.getByText("Projets enrôlés")).toBeVisible();
  await expect(page.getByText("Dernière tâche")).toBeVisible();

  // Specific values from the fixture
  await expect(page.getByText("Hello from Playwright")).toBeVisible();
  await expect(page.getByText("7 tâches")).toBeVisible();
});
