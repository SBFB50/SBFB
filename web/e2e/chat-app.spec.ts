// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * T1 chat v1 (plan Phase G) — hermetic E2E of the SBFB Chat app
 * (`examples/sbfb-chat`) inside the PRODUCTION iframe host.
 *
 * Same pattern as `app-authoring.spec.ts`: the app is seeded into the REAL
 * hermetic daemon spawned by `tests/global-setup.ts` (publish-blob ->
 * publish), then opened from Browse into `BrowsedProject` (opaque-origin
 * iframe, real CSP, real postMessage bridge, real `/app/{name}/chat/*`
 * routes, real SSE push). Nothing is mocked. The archive is packed at test
 * time from the app SOURCE tree, so the spec always runs the current app.
 *
 * Scope — ONE node. The harness runs a single singleton daemon (frozen Day-0
 * decision), so the second side of a conversation (accepting the invite,
 * receiving on another node) is NOT exercised here; cross-node convergence
 * belongs to the Rust multi-node tests and the live acceptance. No second
 * side is stubbed: every step asserted below is single-side and real.
 *
 * Mailbox: the hermetic config is written BEFORE first boot, so the daemon
 * does not add the public default mailbox (the anchor) — the hermetic node
 * has no mailbox server and never dials out. The toggle is therefore
 * asserted in its "no server configured" state, and the daemon's refusal to
 * turn it on is asserted at the route. Turning it on for real needs a mailbox
 * server, i.e. a second daemon.
 *
 * Publishes into the shared daemon (no unpublish route): it runs in the
 * `chromium-authoring` project, AFTER the read-only empty-state specs.
 */

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type { APIRequestContext } from "@playwright/test";

import { test, expect, computeFrame } from "./fixtures";
import { makeZip } from "./fixtures/zip";
import { TEST_AUTH_TOKEN, TEST_COORD_URL } from "../tests/global-setup";

const __dirname = dirname(fileURLToPath(import.meta.url));
const APP_DIR = resolve(__dirname, "../../examples/sbfb-chat");
const APP_FILES = [
  "SBFB.json",
  "index.html",
  "app.js",
  "sbfb-bridge.js",
  "style.css",
];
const PROJECT_NAME = "sbfb-chat-e2e";

const HERMETIC = !process.env.SBFB_E2E_BASE_URL;
const AUTH = { "x-sbfb-token": TEST_AUTH_TOKEN };

interface Seeded {
  projectId: string;
  /** `${curator_pubkey}:${project_name}` — the shell's chat app key. */
  chatApp: string;
}

let seeded: Seeded | null = null;

/** Pack the chat app from source and publish it once into the daemon. */
async function seedChatApp(request: APIRequestContext): Promise<Seeded> {
  if (seeded) return seeded;
  const zip = makeZip(
    APP_FILES.map((f) => [f, readFileSync(resolve(APP_DIR, f))]),
  );
  const blobRes = await request.post(
    `${TEST_COORD_URL}/api/daemon/publish-blob`,
    {
      headers: { ...AUTH, "content-type": "application/octet-stream" },
      data: zip,
    },
  );
  expect(blobRes.ok(), "publish-blob sbfb-chat").toBeTruthy();
  const { hash } = (await blobRes.json()) as { hash: string };

  const pubRes = await request.post(`${TEST_COORD_URL}/api/daemon/publish`, {
    headers: { ...AUTH, "content-type": "application/json" },
    data: {
      project_name: PROJECT_NAME,
      category: "communication",
      description: "SBFB Chat (E2E)",
      apps: ["app"],
      archive_hash: hash,
    },
  });
  expect(pubRes.ok(), "publish sbfb-chat").toBeTruthy();

  const browseRes = await request.get(`${TEST_COORD_URL}/api/daemon/browse`, {
    headers: AUTH,
  });
  expect(browseRes.ok(), "browse").toBeTruthy();
  const browse = (await browseRes.json()) as {
    entries: {
      project_id: string;
      project_name: string;
      curator_pubkey: string;
      archive_hash?: string | null;
    }[];
  };
  const entry = browse.entries.find(
    (e) => e.project_name === PROJECT_NAME && e.archive_hash === hash,
  );
  expect(entry, "seeded sbfb-chat entry present in /browse").toBeTruthy();
  seeded = {
    projectId: entry!.project_id,
    chatApp: `${entry!.curator_pubkey}:${PROJECT_NAME}`,
  };
  return seeded;
}

test.describe("SBFB Chat app — hermetic one-node flow (T1 chat v1)", () => {
  test.skip(!HERMETIC, "hermetic-only: seeds the app into the spawned daemon");

  test("Refuser keeps the chat verbs closed to the app", async ({
    page,
    request,
  }) => {
    const { projectId } = await seedChatApp(request);
    await page.goto(`/browse/${projectId}`);

    await expect(page.getByTestId("chat-consent")).toBeVisible();
    await page.getByTestId("chat-consent-deny").click();
    await expect(page.getByTestId("chat-consent")).toBeHidden();

    // The remounted app meets a refusal and says so in French.
    const app = computeFrame(page);
    await expect(app.locator("#identity-label")).toHaveText("Indisponible");
    await expect(app.locator("#notice")).toContainText(
      "La messagerie n'est pas autorisée pour cette app",
    );
    await expect(app.locator("#layout")).toBeHidden();
  });

  test("Browse -> Autoriser -> name -> conversation -> invite -> message -> mailbox", async ({
    page,
    request,
  }) => {
    const { projectId, chatApp } = await seedChatApp(request);

    await test.step("open the app from Browse", async () => {
      await page.goto("/browse");
      await page
        .getByTestId("browse-card")
        .filter({ hasText: PROJECT_NAME })
        .first()
        .click();
      await expect(page).toHaveURL(new RegExp(`/browse/${projectId}$`));
      await expect(page.getByTestId("remote-iframe-element")).toBeVisible();
    });

    const app = computeFrame(page);

    await test.step("consent banner -> Autoriser -> the app boots", async () => {
      await expect(page.getByTestId("chat-consent")).toBeVisible();
      await expect(
        page.getByText("Cette app demande la messagerie chiffrée"),
      ).toBeVisible();
      await page.getByTestId("chat-consent-grant").click();
      await expect(page.getByTestId("chat-consent")).toBeHidden();
      // Fresh node: no profile name yet -> onboarding.
      await expect(app.locator("#onboarding")).toBeVisible();
      await expect(app.locator("#layout")).toBeHidden();
    });

    await test.step("onboarding: choose a name", async () => {
      await app.locator("#profile-name").fill("Alice");
      await app.locator("#profile-save").click();
      await expect(app.locator("#onboarding")).toBeHidden();
      await expect(app.locator("#layout")).toBeVisible();
      await expect(app.locator("#identity-label")).toContainText("Alice · ");
      await expect(app.locator("#conv-list")).toContainText(
        "Aucune conversation.",
      );
    });

    await test.step("create a conversation", async () => {
      await app.locator("#new-conv-title").fill("Équipe projet");
      await app.locator("#new-conv-btn").click();
      await expect(app.locator("#chat-title")).toHaveText("Équipe projet");
      await expect(app.locator("#conv-list")).toContainText("Équipe projet");
      await expect(app.locator("#conv-list")).toContainText("1 membre(s)");
      await expect(app.locator("#notice")).toContainText("Conversation créée.");
    });

    await test.step("invite link is shown", async () => {
      await expect(app.locator("#invite-box")).toBeHidden();
      await app.locator("#invite-btn").click();
      await expect(app.locator("#invite-box")).toBeVisible();
      await expect(app.locator("#invite-link")).toHaveValue(/^sbfb-chat:\S+$/);
      await expect(app.locator("#invite-expiry")).toContainText(
        "Valable jusqu'au",
      );
    });

    // Markup in the body must render as TEXT, never as HTML.
    const body = "<b>Bonjour</b> & <img src=x onerror=alert(1)>";

    await test.step("send a message: it renders as text", async () => {
      await app.locator("#composer-text").fill(body);
      await app.locator("#composer-text").press("Enter");
      const msg = app.locator("#messages li.message");
      await expect(msg).toHaveCount(1);
      await expect(msg.locator(".message-body")).toHaveText(body);
      await expect(msg).toContainText("Vous");
      await expect(app.locator("#messages b")).toHaveCount(0);
      await expect(app.locator("#messages img")).toHaveCount(0);
      await expect(app.locator("#composer-text")).toHaveValue("");
    });

    await test.step("history survives a reload (consent remembered)", async () => {
      await page.reload();
      await expect(page.getByTestId("chat-consent")).toBeHidden();
      await expect(app.locator("#layout")).toBeVisible();
      await app
        .locator("#conv-list button")
        .filter({ hasText: "Équipe projet" })
        .click();
      await expect(
        app.locator("#messages li.message .message-body"),
      ).toHaveText([body]);
    });

    await test.step("mailbox toggle: off, no server configured", async () => {
      await expect(app.locator("#mailbox-state")).toHaveText(
        "Aucun nœud boîte aux lettres configuré.",
      );
      await expect(app.locator("#mailbox-btn")).toBeDisabled();
      await expect(app.locator("#mailbox-btn")).toHaveAttribute(
        "aria-pressed",
        "false",
      );
      // The daemon refuses to turn it on without a server (not just the UI).
      const route = `${TEST_COORD_URL}/app/${encodeURIComponent(chatApp)}/chat/mailbox`;
      const on = await request.post(route, {
        headers: { ...AUTH, "content-type": "application/json" },
        data: { enabled: true },
      });
      expect(on.status()).toBe(400);
      expect(await on.text()).toContain("no mailbox server configured");
      const view = await request.get(route, { headers: AUTH });
      expect(view.ok()).toBeTruthy();
      expect(((await view.json()) as { enabled: boolean }).enabled).toBe(
        false,
      );
    });
  });
});
