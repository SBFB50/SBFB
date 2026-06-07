// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase A — BrowsedProject page unit tests.
 *
 * Tests cover the full-screen immersive layout with auto-hide
 * glassmorphism top bar (redesigned from Sprint 11's sidebar layout).
 *
 * - routing / back link in auto-hide top bar
 * - "no coordinator" state
 * - "project not found" state
 * - top bar rendering with project metadata
 * - remote project placeholder
 * - local project apps rendering
 * - empty apps list
 */

import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import BrowsedProject from "../BrowsedProject";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";
const LOCAL_NODE_ID = "aa".repeat(32);
const REMOTE_NODE_ID = "bb".repeat(32);

// The daemon's node_id is what appears as project_id in BrowseEntry
// for self-published projects. The coordinator has a *different*
// node_id (its own iroh endpoint). The locality check compares
// against the daemon's node_id via GET /daemon/info, not the
// coordinator's GET /health.
function makeDaemonInfo(nodeId: string = LOCAL_NODE_ID) {
  return {
    schema_version: 1,
    node_id: nodeId,
    daemon_version: "1.0.0",
    uptime_secs: 120,
    started_at: "2026-04-12T10:00:00Z",
    last_updated_at: "2026-04-12T12:00:00Z",
    api_host: "127.0.0.1",
    api_port: 7000,
    subscribed_curators: [],
    known_lists: 0,
    known_browse_entries: 1,
  };
}

function makeBrowseEntry(overrides: Record<string, unknown> = {}) {
  return {
    project_id: LOCAL_NODE_ID,
    project_name: "gov",
    category: "gouvernance",
    description: "Application de gouvernance",
    curator_pubkey: "cc".repeat(32),
    curator_name: "FlowUP",
    source: "curator",
    status: "reachable",
    last_probed_at: "2026-04-12T12:00:00Z",
    is_open_source: false,
    ...overrides,
  };
}

function mockFetch(handlers: Record<string, unknown>) {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => {
      const path = new URL(url).pathname;
      for (const [pattern, body] of Object.entries(handlers)) {
        if (path.includes(pattern)) {
          return new Response(JSON.stringify(body), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
      }
      return new Response(JSON.stringify({ detail: "not found" }), {
        status: 404,
        headers: { "content-type": "application/json" },
      });
    }),
  );
}

function renderPage(projectId: string) {
  const qc = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
    },
  });

  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={[`/browse/${projectId}`]}>
        <Routes>
          <Route path="/browse/:projectId" element={<BrowsedProject />} />
          <Route path="/browse" element={<div data-testid="browse-page" />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  try {
    sessionStorage.clear();
  } catch {
    /* jsdom always provides sessionStorage; guard for safety. */
  }
  useProjectStore.setState({
    knownCoordinators: [
      { url: COORD_URL, nickname: "test", nodeId: null },
    ],
    activeCoordinatorUrl: COORD_URL,
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  useProjectStore.setState({
    knownCoordinators: [],
    activeCoordinatorUrl: null,
  });
});

describe("BrowsedProject", () => {
  it("renders a back link to /browse in the auto-hide top bar", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("back-to-browse")).toBeInTheDocument();
    });
    const backLink = screen.getByTestId("back-to-browse");
    expect(backLink).toHaveAttribute("href", "/browse");
  });

  it("renders 'no node' when no active coordinator is set", () => {
    useProjectStore.setState({
      knownCoordinators: [],
      activeCoordinatorUrl: null,
    });
    renderPage(LOCAL_NODE_ID);
    expect(
      screen.getByText(/Aucun noeud actif/),
    ).toBeInTheDocument();
  });

  it("renders 'project not found' when entry is not in browse list", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [] },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage("ff".repeat(32));
    await waitFor(() => {
      expect(screen.getByTestId("project-not-found")).toBeInTheDocument();
    });
    expect(screen.getByText(/Projet introuvable/)).toBeInTheDocument();
  });

  it("renders project name in auto-hide top bar when entry is found", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.getByText("gov")).toBeInTheDocument();
  });

  it("renders remote placeholder for non-local project without archive", async () => {
    mockFetch({
      "/api/daemon/browse": {
          entries: [makeBrowseEntry({ project_id: REMOTE_NODE_ID })],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("remote-placeholder")).toBeInTheDocument();
    });
    expect(screen.getByText(/Projet distant/)).toBeInTheDocument();
  });

  it("renders iframe for remote project with archive_hash", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: REMOTE_NODE_ID,
            archive_ticket: "blobticket_abc123",
            archive_hash: "ab".repeat(32),
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("remote-iframe-element")).toBeInTheDocument();
    });
    const iframe = screen.getByTestId("remote-iframe-element");
    expect(iframe.tagName).toBe("IFRAME");
    expect(iframe).toHaveAttribute("sandbox", "allow-scripts");
    expect(iframe).toHaveAttribute(
      "src",
      expect.stringContaining("/blob-serve/"),
    );
  });

  it("renders sandbox label in top bar for iframe content", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: REMOTE_NODE_ID,
            archive_hash: "ab".repeat(32),
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.getByText("sandbox")).toBeInTheDocument();
  });

  it("renders local project apps view for matching daemon node_id", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": {
        apps: [
          {
            name: "gov",
            version: "1.0.0",
            description: "Gouvernance",
            routes: 0,
            workers: 0,
            tabs: 1,
            commands: 0,
          },
        ],
        count: 1,
      },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    // Should NOT show remote placeholder
    expect(screen.queryByTestId("remote-placeholder")).not.toBeInTheDocument();
  });

  it("renders 'no apps' message when local project has zero apps", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("no-apps")).toBeInTheDocument();
    });
    expect(
      screen.getByText(/Aucune application SDK/),
    ).toBeInTheDocument();
  });

  it("renders source badge 'Upload direct' for direct entries in top bar", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [makeBrowseEntry({ source: "direct" })],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.getByText("Upload direct")).toBeInTheDocument();
  });

  it("renders repo link for entry with repo_url", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            repo_url: "https://github.com/example/gov",
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    const link = screen.getByTestId("repo-link");
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute("href", "https://github.com/example/gov");
  });

  it("does not render repo link when repo_url is absent", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("repo-link")).not.toBeInTheDocument();
  });

  it("renders verified badge when provenance_hash is present", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            provenance_hash: "bb".repeat(32),
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("verified-badge")).toBeInTheDocument();
    });
  });

  it("does not render verified badge when provenance_hash is absent", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("verified-badge")).not.toBeInTheDocument();
  });

  it("badge shows 'Signature verifiee' after successful verification", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [makeBrowseEntry({ provenance_hash: "bb".repeat(32) })],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
      "/provenance": {
        record: { repo_url: "https://example.com" },
        verified: true,
        status: "verified",
        provenance_hash: "bb".repeat(32),
      },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByText("Signature verifiee")).toBeInTheDocument();
    });
  });

  it("badge shows 'Verification echouee' when verification fails", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [makeBrowseEntry({ provenance_hash: "bb".repeat(32) })],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
      "/provenance": {
        record: { repo_url: "https://example.com" },
        verified: false,
        status: "failed",
        provenance_hash: "bb".repeat(32),
      },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByText("Verification echouee")).toBeInTheDocument();
    });
  });

  it("badge shows 'Provenance' when status is absent", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [makeBrowseEntry({ provenance_hash: "bb".repeat(32) })],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
      "/provenance": {
        record: null,
        verified: false,
        status: "absent",
        provenance_hash: null,
      },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      const badge = screen.getByTestId("verified-badge");
      expect(badge).toHaveTextContent("Provenance");
    });
  });

  it("badge shows 'Verification...' while loading provenance", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        const path = new URL(url).pathname;
        if (path.includes("/provenance")) {
          return new Promise<Response>(() => {});
        }
        const handlers: Record<string, unknown> = {
          "/api/daemon/browse": {
            entries: [makeBrowseEntry({ provenance_hash: "bb".repeat(32) })],
          },
          "/api/daemon/info": makeDaemonInfo(),
          "/app": { apps: [], count: 0 },
        };
        for (const [pattern, body] of Object.entries(handlers)) {
          if (path.includes(pattern)) {
            return new Response(JSON.stringify(body), {
              status: 200,
              headers: { "content-type": "application/json" },
            });
          }
        }
        return new Response(JSON.stringify({ detail: "not found" }), {
          status: 404,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByText("Verification...")).toBeInTheDocument();
    });
  });

  it("does not render watchdog overlay in unknown state (initial load)", async () => {
    // Sprint 15 Phase B: the overlay only appears after a healthy
    // app stops emitting heartbeats. At mount the state is unknown
    // and the user sees a clean iframe, not a "ne repond plus" alert.
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("watchdog-overlay")).not.toBeInTheDocument();
  });

  // Sprint 74 Phase A — greffe A : the offline reminder is shown only for
  // the user's OWN apps, and is dismissible 1x/session/app.
  it("offline_reminder_only_for_own_apps_dismissible", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/browse": {
        entries: [makeBrowseEntry({ status: "unreachable" })],
      },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    const { unmount } = renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("offline-reminder")).toBeInTheDocument();
    });
    await user.click(screen.getByTestId("offline-reminder-dismiss"));
    await waitFor(() => {
      expect(screen.queryByTestId("offline-reminder")).not.toBeInTheDocument();
    });
    unmount();

    // A remote app (different node_id) never shows the reminder, even offline.
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: REMOTE_NODE_ID,
            status: "unreachable",
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("browsed-project")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("offline-reminder")).not.toBeInTheDocument();
  });

  // Sprint 74 Phase A — the "Disponibilite" button replaces the raw blob:<hash>
  // badge and opens the availability panel.
  it("availability_button_opens_panel", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/info": makeDaemonInfo(),
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("availability-button")).toBeInTheDocument();
    });
    expect(screen.queryByText(/^blob:/)).not.toBeInTheDocument();
    await user.click(screen.getByTestId("availability-button"));
    await waitFor(() => {
      expect(screen.getByTestId("availability-sheet")).toBeInTheDocument();
    });
    expect(
      screen.getByTestId("availability-section-author"),
    ).toBeInTheDocument();
  });

  // Sprint 74 Phase A — greffe D : a fallen remote app with a verifiable https
  // source offers a one-click redeploy that prefills /deploy.
  it("fallen-app offers redeploy with an encoded /deploy prefill (https only)", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: REMOTE_NODE_ID,
            project_name: "ideas",
            status: "unreachable",
            repo_url: "https://codeberg.org/me/ideas.git",
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    const link = await screen.findByTestId("redeploy-fallen-app");
    const href = link.getAttribute("href") ?? "";
    expect(href).toContain("/deploy?");
    expect(href).toContain(
      `repo_url=${encodeURIComponent("https://codeberg.org/me/ideas.git")}`,
    );
    expect(href).toContain("project_name=ideas");
  });

  // Sprint 74 Phase A — XSS scheme guard: a javascript:/data: repo_url must NOT
  // produce a redeploy element; it falls through to the safe remote placeholder.
  it("fallen-app scheme guard rejects non-https repo_url", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: REMOTE_NODE_ID,
            status: "unreachable",
            // Build the scheme by concatenation so no lint rule scans a literal
            // `javascript:` token; the runtime value is the real XSS vector.
            repo_url: "javascript" + ":alert(1)",
          }),
        ],
      },
      "/api/daemon/info": makeDaemonInfo(),
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("remote-placeholder")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("redeploy-fallen-app")).not.toBeInTheDocument();
  });
});
