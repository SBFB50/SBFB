/**
 * Sprint 7 Phase E — `/browse` renders the daemon-offline
 * banner when no `nexus-shell-daemon` is running.
 *
 * This spec runs against the real coordinator spawned by
 * `tests/global-setup.ts`. That coordinator does NOT boot a
 * shell daemon, so the `/daemon/browse` proxy call returns
 * 503 `{kind: "unavailable"}`. The React shell's
 * `DaemonOfflineBanner` must surface the reason instead of
 * hanging on a spinner or throwing through the error boundary
 * (G-4 regression — we fought this in the Sprint 6 audit).
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test.beforeEach(async ({ page }) => {
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

test("/browse shows the daemon-offline banner when no daemon is running", async ({
  page,
}) => {
  await page.goto("/browse");

  await expect(
    page.getByRole("heading", { name: "Explorer" }),
  ).toBeVisible();

  // The page header reveals the Browse intent even while the
  // daemon is down — no crashy error boundary, no indefinite
  // spinner.
  const banner = page.getByTestId("daemon-offline-banner");
  await expect(banner).toBeVisible({ timeout: 10_000 });
  // shadcn's vendored CardTitle is a <div>, not an <h3>, so
  // we cannot assert via role="heading". Match the copy
  // directly scoped to the banner so the assertion still
  // catches a future rename.
  await expect(banner.getByText(/Daemon indisponible/)).toBeVisible();

  // The reason string carries something transport-ish (the
  // coordinator proxy returned "shell-daemon not running"
  // when no running.json exists, or "connect failed" if a
  // stale file points at a dead port).
  const reasonBlock = page.getByText(/Détail technique/);
  await expect(reasonBlock).toBeVisible();

  // No browse card must render.
  await expect(page.getByTestId("browse-card")).toHaveCount(0);
});

test("/browse is still reachable from the nav and never 404s", async ({
  page,
}) => {
  // Direct navigation must work (the route exists in the
  // router map) and the daemon-offline banner is the
  // expected render even after a full page reload.
  await page.goto("/browse");
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Explorer" }),
  ).toBeVisible();
  await expect(page.getByTestId("daemon-offline-banner")).toBeVisible({
    timeout: 10_000,
  });
});
