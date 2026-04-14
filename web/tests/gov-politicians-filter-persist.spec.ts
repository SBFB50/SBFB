/**
 * Sprint 9 Phase B (D1 consumer end-to-end).
 *
 * Pins the AppContext.storage filter persist contract:
 *
 *  1. Open the gov Politiciens tab and assert the default
 *     filter summary block reads "Filtres : aucun".
 *  2. POST a populated PoliticiansFilter to the new
 *     `POST /app/gov/state/politicians_filter` route via
 *     `setAppState` (the typed namespace setter).
 *  3. Reload the page and re-open the Politiciens tab.
 *  4. Assert the filter summary block now reflects the
 *     persisted chamber + search values, proving the gov
 *     on_start hook re-registered the namespace and the
 *     handler reads it back from the on-disk storage.
 *
 * The test deliberately drives the filter through the API,
 * not through a UI input — the Phase B scope is the
 * persistence path, not a brand new filter UI. Sprint 10+
 * polish lands the React form bound to `setAppState`.
 *
 * The legacy gov DB is absent in CI so the descriptor falls
 * back to the empty-state branch; we only assert on the
 * filter summary text block which the handler emits
 * unconditionally (before the row check).
 */

import { test, expect } from "@playwright/test";
import { primeAuthToken } from "../src/api/auth";
import { setAppState } from "../src/api/coordinator";
import { TEST_AUTH_TOKEN, TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

// Sprint 16 Phase A (D1): seed the module-local token cache so
// the Node-side `setAppState` call inside this spec injects the
// loopback bearer on its fetch. `extraHTTPHeaders` from the
// Playwright config applies to browser contexts only.
primeAuthToken(TEST_AUTH_TOKEN);

async function openPoliticiansTab(page: import("@playwright/test").Page) {
  await page.goto(`/project/${TEST_COORD_NAME}`);
  await page.getByRole("tab", { name: "Apps" }).click();
  await expect(page.getByText(/gov/).first()).toBeVisible({ timeout: 10_000 });
  await page.getByText("gov", { exact: true }).click();
  await expect(page.getByText("Routes").first()).toBeVisible({
    timeout: 5_000,
  });
  const politiciansRow = page
    .locator("li")
    .filter({ hasText: "Politiciens" })
    .first();
  await politiciansRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();
  await expect(
    page.getByRole("heading", { name: "Politiciens" }),
  ).toBeVisible({ timeout: 10_000 });
}

test("gov Politiciens filter persists across page reload via AppContext.storage", async ({
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

  // 1. Initial state — no filter persisted yet, the summary
  //    block must read "Filtres : aucun".
  await openPoliticiansTab(page);
  await expect(page.getByText("Filtres : aucun")).toBeVisible({
    timeout: 5_000,
  });

  // 2. Push a populated filter to the typed namespace via the
  //    coordinator route. The body is sent as JSON; the gov
  //    on_start hook has already registered the namespace under
  //    `politicians_filter`.
  const response = await setAppState(TEST_COORD_URL, "gov", "politicians_filter", {
    chamber: "Assemblée",
    search: "Dupont",
  });
  expect(response.ok).toBe(true);

  // 3. Reload the page — this drops every client-side cache and
  //    forces the descriptor to be re-fetched from the coord.
  await page.reload();
  await openPoliticiansTab(page);

  // 4. The filter summary now reflects the persisted state. Both
  //    the chamber and the search field are surfaced as a single
  //    " · "-joined line so we assert each substring independently
  //    to keep the test resilient to formatting tweaks.
  const filterLine = page.locator("text=/Filtres :.*Chambre = Assemblée/");
  await expect(filterLine).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText(/Recherche = Dupont/)).toBeVisible();
});
