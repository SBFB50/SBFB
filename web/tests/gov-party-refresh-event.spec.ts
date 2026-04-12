/**
 * Sprint 9 Phase C — `useAppEvents` end-to-end against a live
 * coordinator + SSE bridge.
 *
 * Pins the consumer half of the AppContext.events story:
 *
 *  1. Open the gov Politiciens tab and let the descriptor query
 *     settle (initial fetch).
 *  2. Drive a publish onto the per-app bus via the dedicated
 *     admin route ``POST /app/gov/events/_publish``. The route
 *     bypasses the worker dispatcher (which would need a live
 *     worker daemon) and lands the envelope directly on the
 *     bus — the bus then fans it out through the SSE bridge,
 *     exactly the same code path a real ``@nexus_worker``
 *     publisher would take.
 *  3. The shell's ``useAppEvents`` hook (mounted on the
 *     Politiciens tab) receives the SSE-framed envelope and
 *     invalidates the descriptor React Query — the test waits
 *     for a SECOND HTTP fetch of the same descriptor URL and
 *     asserts it lands within a generous timeout.
 *
 * The assertion shape (a network re-fetch in response to a bus
 * publish) is the load-bearing contract: if SSE never frames
 * the envelope, or the hook never invalidates, no second
 * descriptor request happens.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

async function openPoliticiansTab(page: import("@playwright/test").Page) {
  // Two-step navigation: visit a shell page first so the
  // Zustand persist middleware hydrates from localStorage,
  // then deep-link to the AppTabPage. The AppTabPage mounts
  // ``useAppEvents`` for the gov Politiciens combo (Sprint 9
  // Phase C), which the in-line ``AppsTab.tsx`` view does not.
  await page.goto(`/my-projects`);
  await expect(page.getByText(TEST_COORD_NAME).first()).toBeVisible({
    timeout: 10_000,
  });
  await page.goto(`/app/gov/tabs/Politiciens`);
  await expect(
    page.getByRole("heading", { name: "Politiciens", exact: true }),
  ).toBeVisible({ timeout: 10_000 });
}

test("gov party.refreshed SSE event invalidates the Politiciens descriptor", async ({
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

  // 1. Open Politiciens — the initial descriptor fetch settles.
  await openPoliticiansTab(page);

  // 2. Stage a wait for the SECOND descriptor fetch BEFORE we
  //    trigger the publish so the listener is in place when
  //    the SSE event lands.
  const refetchPromise = page.waitForResponse(
    (response) =>
      response
        .url()
        .includes("/app/gov/tabs/Politiciens/descriptor") &&
      response.status() === 200,
    { timeout: 15_000 },
  );

  // 3. Publish a `party.refreshed` envelope directly on the
  //    per-app bus via the admin route. The bus fans it out
  //    onto every matching subscriber — including the SSE
  //    bridge that the shell's `useAppEvents` hook holds open.
  const response = await page.request.post(
    `${TEST_COORD_URL}/app/gov/events/_publish`,
    {
      data: {
        topic: "party.refreshed",
        payload: { count: 0, refreshed_at: new Date().toISOString() },
      },
    },
  );
  expect(response.ok()).toBe(true);
  const body = await response.json();
  expect(body.topic).toBe("party.refreshed");
  expect(body.status).toBe("published");

  // 4. The hook should have caught the SSE envelope and
  //    invalidated the descriptor React Query — wait for the
  //    second fetch to confirm the round-trip.
  await refetchPromise;
});
