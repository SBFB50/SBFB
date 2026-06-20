// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * FLAGSHIP compute E2E (@compute) — env-gated, runs LOCAL only.
 *
 * The UI mirror of `scripts/acceptance/phase_h_compute_local.sh`: it
 * drives the real compute-tester app inside its sandboxed iframe, lets
 * the host bridge inject the local project_id, submits a real
 * inference task to a real on-demand worker (Ollama on a GPU), and
 * asserts the GENERATED text is rendered back in the iframe. No mock —
 * a real model produces the answer.
 *
 * Gated because it needs an operator rig the hermetic CI lacks:
 *   SBFB_E2E_COMPUTE=1
 *   SBFB_E2E_PROJECT_ID=<compute-tester project id on that daemon>
 *   SBFB_E2E_BASE_URL=<that daemon, serving dist via --web-root>
 *   (optional) SBFB_E2E_MODEL=<ollama tag, default llama3.1:8b>
 * The daemon must have Ollama running and the compute-tester app
 * deployed (browsable, archive_hash present), then run
 * `npm run test:e2e:compute`. Set the env in YOUR shell first (the
 * script carries no inline env so it stays cross-platform):
 *   PowerShell:  $env:SBFB_E2E_COMPUTE="1"; $env:SBFB_E2E_PROJECT_ID="..."; \
 *                $env:SBFB_E2E_BASE_URL="http://127.0.0.1:7654"; npm run test:e2e:compute
 *   bash:        SBFB_E2E_COMPUTE=1 SBFB_E2E_PROJECT_ID=... \
 *                SBFB_E2E_BASE_URL=http://127.0.0.1:7654 npm run test:e2e:compute
 * A wrong-syntax env (e.g. bash form on PowerShell) leaves the flag unset
 * → the test SKIPS rather than runs; check the run shows it executing.
 *
 * `--grep-invert @compute` (the default `test:e2e`) excludes this spec
 * from the hermetic/CI run entirely.
 */

import { test, expect, computeFrame } from "./fixtures";

const PROJECT_ID = process.env.SBFB_E2E_PROJECT_ID ?? "";
const MODEL = process.env.SBFB_E2E_MODEL ?? "llama3.1:8b";
const ENABLED = process.env.SBFB_E2E_COMPUTE === "1" && PROJECT_ID.length > 0;

test.describe("@compute compute-tester flagship", () => {
  test.skip(
    !ENABLED,
    "gated: set SBFB_E2E_COMPUTE=1 + SBFB_E2E_PROJECT_ID + SBFB_E2E_BASE_URL (operator daemon w/ Ollama + compute-tester deployed)",
  );

  test("submits a prompt through the bridge and renders the generated text", async ({
    page,
  }) => {
    // Claim + GPU inference + poll can run ~12s+ ; allow generous slack.
    test.setTimeout(180_000);

    await page.goto(`/browse/${PROJECT_ID}`);

    const frame = computeFrame(page);
    await expect(frame.locator("#submit")).toBeVisible();

    await frame
      .locator("#prompt")
      .fill("Donne trois noms pour un chat roux. Reponds en une ligne.");
    await frame.locator("#model").fill(MODEL);
    await frame.locator("#submit").click();

    // The app flips #state to "Termine en …" and reveals #result with
    // the real generated text once getTaskResult is ready (app.js:84-91).
    await expect(frame.locator("#state")).toContainText("Termine en", {
      timeout: 150_000,
    });

    const result = frame.locator("#result");
    await expect(result).toBeVisible();
    await expect(result).not.toBeEmpty();

    // The metadata line carries the real task id.
    await expect(frame.locator("#meta")).toContainText("task_id");
  });
});
