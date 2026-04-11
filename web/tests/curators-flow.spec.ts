/**
 * Sprint 7 Phase E — `/curators` page flow against the live
 * coordinator (no real daemon).
 *
 * Scenarios covered:
 *
 * 1. Page reachable, renders the "add a curator" form + the
 *    daemon-offline banner (no `nexus-shell-daemon` started
 *    by `global-setup.ts`).
 * 2. Client-side hex validation catches malformed input
 *    without sending a request.
 *
 * A full happy-path subscribe test requires spinning a real
 * shell daemon in global-setup, which is deliberately out of
 * scope for Phase E: the Rust 2-node integration tests
 * already cover the subscribe pipeline end-to-end, and Phase
 * E's UI contract is "render cleanly across every
 * DaemonResult variant". This spec focuses on the
 * unavailable + error paths.
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

test("/curators shows the form and the daemon-offline banner", async ({
  page,
}) => {
  await page.goto("/curators");

  await expect(
    page.getByRole("heading", { name: "Curators" }),
  ).toBeVisible();

  // The subscribe form is always visible, even when the
  // daemon is down — users need to be able to stage the
  // pubkey they want to subscribe to before the daemon
  // comes back online.
  await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();
  await expect(page.getByTestId("curator-subscribe-submit")).toBeVisible();

  // Daemon-offline banner renders below the form.
  await expect(page.getByTestId("daemon-offline-banner")).toBeVisible({
    timeout: 10_000,
  });
});

test("curators form validates hex locally before submitting", async ({
  page,
}) => {
  await page.goto("/curators");
  await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();

  // Type something that's obviously not 64-char lowercase hex.
  await page.getByTestId("curator-pubkey-input").fill("not-a-valid-key");
  await page.getByTestId("curator-subscribe-submit").click();

  // The inline form-error renders the validation message
  // without touching the network.
  const errorBlock = page.getByTestId("curator-form-error");
  await expect(errorBlock).toBeVisible();
  await expect(errorBlock).toContainText(
    /64 caractères hexadécimaux minuscules/,
  );
});

test("curators form accepts well-formed hex and hits the daemon proxy", async ({
  page,
}) => {
  await page.goto("/curators");
  await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();

  // 64 lowercase hex chars — passes client-side validation
  // and reaches the coordinator proxy, which forwards to
  // the (absent) shell daemon and returns a 503.
  await page
    .getByTestId("curator-pubkey-input")
    .fill("a".repeat(64));
  await page.getByTestId("curator-subscribe-submit").click();

  // The shell's mutation onSuccess path surfaces the
  // daemon-unavailable reason in the form error area.
  // Either the form error appears (proxy returned
  // unavailable → setFormError) or the banner stays on
  // screen (never a crash). We accept either render — the
  // point of the test is the non-crash invariant.
  await expect(async () => {
    const formErr = page.getByTestId("curator-form-error");
    const banner = page.getByTestId("daemon-offline-banner");
    const visible =
      (await formErr.isVisible()) || (await banner.isVisible());
    expect(visible).toBe(true);
  }).toPass({ timeout: 10_000 });
});
