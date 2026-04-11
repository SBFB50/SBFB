/**
 * Sprint 8 Phase D — gov Recherche (RAG search) tab e2e.
 *
 * Phase D ships the Recherche tab as a static TabView that
 * does not depend on ``ctx.db`` — the handler renders a fixed
 * heading, an example section with a ``task_submit`` button
 * targeting the ``gov.rag_search`` worker, and a muted
 * explanatory text. The tab must render identically whether or
 * not the legacy ``govdata.db`` file is present, so this
 * spec asserts the heading, the worker/model key-value block,
 * and the button label all land in the DOM.
 *
 * We stop short of clicking the button: the Playwright harness
 * runs the coordinator without a live Rust worker daemon, so a
 * real task submission would fail at the dispatcher level.
 * Asserting the descriptor contract (button visible, right
 * label) is enough to catch regressions on the TabView →
 * ButtonBlock → task_submit wiring Phase A shipped.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Recherche tab exposes RAG search task_submit button", async ({
  page,
}) => {
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

  await page.goto(`/project/${TEST_COORD_NAME}`);
  await page.getByRole("tab", { name: "Apps" }).click();

  await expect(page.getByText(/gov/).first()).toBeVisible({ timeout: 10_000 });
  await page.getByText("gov", { exact: true }).click();

  await expect(page.getByText("Routes").first()).toBeVisible({
    timeout: 5_000,
  });

  // Recherche is the first of the two RAG tabs shipped Phase D.
  const searchRow = page
    .locator("li")
    .filter({ hasText: "Recherche" })
    .first();
  await searchRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block for the tab.
  await expect(
    page.getByRole("heading", { name: "Recherche RAG" }),
  ).toBeVisible({ timeout: 10_000 });

  // Key-value block inside the "Exemple" section exposes the
  // worker routing key and the model the RAG worker registers
  // against in its ``@nexus_worker`` decoration. The same model
  // name ``nomic-embed-text`` also appears in the Apps sidebar
  // card (under Workers), so the locator is scoped to the
  // ``<dd>`` rendered by the TabView KV block to avoid a strict
  // mode violation with the sidebar's ``<code>`` element.
  await expect(
    page.getByRole("definition").filter({ hasText: "gov.rag_search" }),
  ).toBeVisible();
  await expect(
    page.getByRole("definition").filter({ hasText: "nomic-embed-text" }),
  ).toBeVisible();

  // The ``task_submit`` button wired to ``gov.rag_search`` must
  // render with its Phase D label. Clicking it is out of scope
  // (no live worker daemon in the Playwright harness).
  await expect(
    page.getByRole("button", { name: "Lancer la recherche exemple" }),
  ).toBeVisible();

  // Sanity check: no legacy descriptor warning and no raw JSON.
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
