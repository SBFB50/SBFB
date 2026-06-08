// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 74 Phase A — AvailabilitySheet unit tests.
 *
 * Covers the three sealed-separately sections, the live-probe state mapping,
 * and the FUNCTIONAL "Garder en ligne" toggle (Sprint 74 Phase D: POSTs to
 * /api/daemon/keep-online, no fetch on mount).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import {
  AvailabilitySheet,
  type AvailabilitySheetProps,
} from "../AvailabilitySheet";
import type { BrowseEntry } from "@/api/daemon";

const COORD_URL = "http://127.0.0.1:8765";

function makeEntry(overrides: Partial<BrowseEntry> = {}): BrowseEntry {
  return {
    project_id: "aa".repeat(32),
    project_name: "gov",
    category: "gouvernance",
    description: "Application de gouvernance",
    curator_pubkey: "cc".repeat(32),
    curator_name: "FlowUP",
    source: "direct",
    status: "reachable",
    last_probed_at: "2026-06-07T12:00:00Z",
    archive_hash: "ab".repeat(32),
    provenance_hash: "bb".repeat(32),
    is_open_source: true,
    ...overrides,
  };
}

/**
 * Sprint 74 Phase F: the sheet fetches GET /api/daemon/seed-count/{id} on
 * mount (while open). Tests that stub `fetch` must answer that GET with a
 * valid `{ peer_count, self_seeding }` body, else the `.strict()` Zod parse
 * throws inside the query. This helper returns the seed-count Response when
 * the URL matches, or `null` to let the caller handle its own routes.
 */
function seedCountResponseFor(
  url: string,
  peer_count: number,
  self_seeding: boolean,
): Response | null {
  if (url.includes("/api/daemon/seed-count")) {
    return new Response(JSON.stringify({ peer_count, self_seeding }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  }
  return null;
}

function renderSheet(props: Partial<AvailabilitySheetProps> = {}) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <AvailabilitySheet
        open
        onOpenChange={() => {}}
        entry={makeEntry()}
        isOwn
        coordUrl={COORD_URL}
        {...props}
      />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  // Default: every route is "daemon unavailable" so the Phase F seed-count GET
  // fired on mount never reaches the real network (Node 18+ has a global
  // `fetch`). Tests that assert on specific routes override this via
  // `vi.stubGlobal("fetch", ...)`.
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      new Response("{}", {
        status: 503,
        headers: { "content-type": "application/json" },
      }),
    ),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("AvailabilitySheet", () => {
  it("availability_sheet_renders_author_state_seeders", async () => {
    renderSheet();
    expect(
      await screen.findByTestId("availability-section-author"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("availability-section-state"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("availability-section-seeders"),
    ).toBeInTheDocument();
    // The author section is sealed to the immutable signer (verrou §8(4)).
    expect(screen.getByText("Publiee par ton noeud")).toBeInTheDocument();
    expect(
      screen.getByText(/L'auteur est fige par la signature/),
    ).toBeInTheDocument();
    // "Signature verifiee" is gated on provenance_hash (present by default).
    expect(screen.getByText("Signature verifiee")).toBeInTheDocument();
  });

  it("hides 'Signature verifiee' when provenance_hash is absent", async () => {
    renderSheet({ entry: makeEntry({ provenance_hash: undefined }) });
    await screen.findByTestId("availability-section-author");
    expect(screen.queryByText("Signature verifiee")).not.toBeInTheDocument();
  });

  it("availability_state_maps_reachable_unreachable_unknown", async () => {
    // Reachable copy depends on ownership: a remote app the local node can
    // actually reach is "joignable par tous"; the user's own app uses the
    // honest NAT label "vu de ton noeud" (PO Q2 — never over-claim).
    const remote = renderSheet({
      entry: makeEntry({ status: "reachable" }),
      isOwn: false,
    });
    expect(
      await screen.findByText("En ligne — joignable par tous"),
    ).toBeInTheDocument();
    remote.unmount();

    const own = renderSheet({
      entry: makeEntry({ status: "reachable" }),
      isOwn: true,
    });
    expect(
      await screen.findByText("En ligne (vu de ton noeud)"),
    ).toBeInTheDocument();
    own.unmount();

    const unreachable = renderSheet({
      entry: makeEntry({ status: "unreachable" }),
    });
    expect(
      await screen.findByText(
        "Hors ligne — relance ton noeud pour la rediffuser",
      ),
    ).toBeInTheDocument();
    unreachable.unmount();

    renderSheet({ entry: makeEntry({ status: "unknown" }) });
    expect(await screen.findByText("Verification…")).toBeInTheDocument();
  });

  it("keep_online_toggle_is_functional", async () => {
    // Sprint 74 Phase D: the toggle starts ON, performs NO fetch on mount, and
    // POSTs the OFF intent to /api/daemon/keep-online on click, then reflects the
    // daemon's persisted state.
    const user = userEvent.setup();
    const calls: Array<{ url: string; body: unknown }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init?: RequestInit) => {
        calls.push({
          url,
          body: init?.body ? JSON.parse(init.body as string) : null,
        });
        const seed = seedCountResponseFor(url, 0, true);
        if (seed) return seed;
        return new Response(JSON.stringify({ ok: true, enabled: false }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }),
    );

    renderSheet({ entry: makeEntry({ status: "reachable" }), isOwn: true });

    const toggle = await screen.findByTestId("keep-online-toggle");
    // Starts ON, interactive (not disabled). The toggle itself does NOT POST on
    // mount (verrou §8(5): a real control, not a faux button) — only the
    // Phase F seed-count GET fires on open.
    expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(toggle).not.toBeDisabled();
    expect(
      calls.some((c) => c.url.includes("/api/daemon/keep-online")),
    ).toBe(false);

    await user.click(toggle);

    // It POSTed the OFF intent to the keep-online route...
    await waitFor(() => {
      expect(
        calls.some((c) => c.url.includes("/api/daemon/keep-online")),
      ).toBe(true);
    });
    const koCall = calls.find((c) =>
      c.url.includes("/api/daemon/keep-online"),
    )!;
    expect(koCall.body).toMatchObject({
      project_id: "aa".repeat(32),
      enabled: false,
    });
    // ...and reflected the daemon's persisted state (now OFF).
    await waitFor(() => {
      expect(screen.getByTestId("keep-online-toggle")).toHaveAttribute(
        "aria-pressed",
        "false",
      );
    });
  });

  it("reverify triggers a browse pull", async () => {
    const fetchSpy = vi.fn(async (url: string) => {
      const seed = seedCountResponseFor(url, 0, true);
      if (seed) return seed;
      return new Response(JSON.stringify({ requested: true }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });
    vi.stubGlobal("fetch", fetchSpy);
    const user = userEvent.setup();
    renderSheet({ entry: makeEntry({ status: "reachable" }) });

    // Freshness label reads the probe time.
    expect(await screen.findByTestId("availability-freshness")).toHaveTextContent(
      /Verifie/,
    );

    await user.click(screen.getByTestId("availability-reverify"));
    await waitFor(() => {
      const calls = fetchSpy.mock.calls as unknown as Array<[string]>;
      const calledPull = calls.some((c) =>
        String(c[0]).includes("/api/daemon/browse/pull"),
      );
      expect(calledPull).toBe(true);
    });
  });

  it("remote app shows the voluntary support CTA (functional)", async () => {
    renderSheet({ entry: makeEntry({ status: "reachable" }), isOwn: false });
    expect(
      await screen.findByTestId("support-seed-cta"),
    ).toBeInTheDocument();
    expect(screen.getByText("Publiee par un autre noeud")).toBeInTheDocument();
    // No own-node toggle for a remote app.
    expect(screen.queryByTestId("keep-online-toggle")).not.toBeInTheDocument();
  });

  it("voluntary_seed_distant_app_posts_and_confirms", async () => {
    // Sprint 74 Phase E: clicking the support CTA on a remote app POSTs to
    // /api/daemon/seed and, on success, flips to the "supporting" state.
    const user = userEvent.setup();
    const calls: Array<{ url: string; body: unknown }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init?: RequestInit) => {
        calls.push({
          url,
          body: init?.body ? JSON.parse(init.body as string) : null,
        });
        const seed = seedCountResponseFor(url, 0, false);
        if (seed) return seed;
        return new Response(
          JSON.stringify({ ok: true, seeding: "aa".repeat(32) }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      }),
    );

    renderSheet({ entry: makeEntry({ status: "reachable" }), isOwn: false });
    const cta = await screen.findByTestId("support-seed-cta");
    await user.click(cta);

    // It POSTed the voluntary seed intent for this project... (match the exact
    // /api/daemon/seed route, NOT the /api/daemon/seed-count GET that shares a
    // prefix and also fires on mount).
    await waitFor(() => {
      expect(calls.some((c) => c.url.endsWith("/api/daemon/seed"))).toBe(true);
    });
    const seedCall = calls.find((c) => c.url.endsWith("/api/daemon/seed"))!;
    expect(seedCall.body).toMatchObject({ project_id: "aa".repeat(32) });

    // ...and reflected the confirmed "supporting" state.
    expect(
      await screen.findByTestId("support-seed-active"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Tu gardes ce projet en ligne"),
    ).toBeInTheDocument();
  });

  it("multi_seed_state_rendered", async () => {
    // Sprint 74 Phase F: the "Copies de secours" section renders the live
    // best-effort multi-seed count fetched from /api/daemon/seed-count. With
    // remote seeders present it shows "Toi + N pairs"; with none it falls back
    // to the "Aucune copie de secours" warning.
    const withPeers = vi.fn(async (url: string) => {
      const seed = seedCountResponseFor(url, 2, true);
      if (seed) return seed;
      return new Response("{}", {
        status: 503,
        headers: { "content-type": "application/json" },
      });
    });
    vi.stubGlobal("fetch", withPeers);

    const present = renderSheet({ entry: makeEntry(), isOwn: true });
    const backup = await screen.findByTestId("backup-count");
    await waitFor(() => {
      expect(backup).toHaveTextContent("Toi + 2 pairs");
    });
    expect(backup).toHaveTextContent("vus récemment");
    present.unmount();
    vi.unstubAllGlobals();

    // No remote seeder → the sole-seeder warning.
    const noPeers = vi.fn(async (url: string) => {
      const seed = seedCountResponseFor(url, 0, true);
      if (seed) return seed;
      return new Response("{}", {
        status: 503,
        headers: { "content-type": "application/json" },
      });
    });
    vi.stubGlobal("fetch", noPeers);

    renderSheet({ entry: makeEntry(), isOwn: true });
    const backup2 = await screen.findByTestId("backup-count");
    await waitFor(() => {
      expect(backup2).toHaveTextContent("Aucune copie de secours");
    });
  });
});
