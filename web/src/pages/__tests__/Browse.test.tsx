// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 73 Phase E (D4) — Browse search bar unit tests.
 *
 * The dedicated search field wires `GET /api/daemon/search` (the
 * daemon FTS5 index) into the Browse page. Tests cover:
 *
 * - typing a query renders the enriched provenance hits
 * - an empty result set shows the French "no result" state
 * - an empty query keeps the normal browse grid (non-regression)
 *
 * `fetch` is stubbed and routed by pathname so the browse and
 * search endpoints can return distinct fixtures in the same test.
 */

import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import Browse from "../Browse";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

function makeSearchHit(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "aa".repeat(32),
    project_name: "verified-app",
    category: "outils",
    description: "App deployee depuis sa source",
    op_type: "ReleasePublished",
    source_type: "feed",
    score: 1.42,
    repo_url: "https://github.com/test/verified-app",
    commit_sha: "ab".repeat(20),
    archive_hash: "cd".repeat(32),
    provenance_hash: "ef".repeat(32),
    is_open_source: true,
    ...overrides,
  };
}

function makeBrowseEntry(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "11".repeat(32),
    project_name: "navigable-app",
    category: "gouvernance",
    description: "App listee via curator",
    curator_pubkey: "22".repeat(32),
    curator_name: "FlowUP",
    source: "curator",
    status: "reachable",
    last_probed_at: "2026-06-04T12:00:00Z",
    is_open_source: false,
    ...overrides,
  };
}

/**
 * Route the stubbed fetch by pathname. The first handler key whose
 * substring matches the request pathname wins, so keep `search`
 * and `browse` keys distinct.
 */
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
      return new Response(JSON.stringify({ error: "not found" }), {
        status: 404,
        headers: { "content-type": "application/json" },
      });
    }),
  );
}

function renderBrowse() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={["/browse"]}>
        <Routes>
          <Route path="/browse" element={<Browse />} />
          <Route
            path="/browse/:projectId"
            element={<div data-testid="project-page" />}
          />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  useProjectStore.setState({
    knownCoordinators: [{ url: COORD_URL, nickname: "test", nodeId: null }],
    activeCoordinatorUrl: COORD_URL,
  });
});
afterEach(() => {
  vi.unstubAllGlobals();
});

describe("Browse search bar", () => {
  it("browse_search_renders_enriched_results", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [] },
      "/api/daemon/search": {
        results: [makeSearchHit()],
        total: 1,
        took_ms: 2,
      },
    });

    renderBrowse();
    const input = await screen.findByTestId("browse-search-input");
    fireEvent.change(input, { target: { value: "verified" } });

    await waitFor(() => {
      expect(screen.getByText("verified-app")).toBeInTheDocument();
    });

    // Provenance triplet surfaced on the hit card.
    expect(screen.getByTestId("search-verified-badge")).toBeInTheDocument();
    const repoLink = screen.getByTestId("search-repo-link");
    expect(repoLink).toHaveAttribute(
      "href",
      "https://github.com/test/verified-app",
    );
  });

  it("does not render a non-https repo_url as a link (XSS guard)", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [] },
      "/api/daemon/search": {
        results: [
          makeSearchHit({ repo_url: "javascript:alert(1)" }),
        ],
        total: 1,
        took_ms: 1,
      },
    });

    renderBrowse();
    const input = await screen.findByTestId("browse-search-input");
    fireEvent.change(input, { target: { value: "verified" } });

    await waitFor(() => {
      expect(screen.getByText("verified-app")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("search-repo-link")).not.toBeInTheDocument();
  });

  it("browse_search_empty_state_french", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [] },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();
    const input = await screen.findByTestId("browse-search-input");
    fireEvent.change(input, { target: { value: "zzzznomatch" } });

    await waitFor(() => {
      expect(screen.getByTestId("browse-search-empty")).toBeInTheDocument();
    });
    expect(screen.getByText("Aucun résultat")).toBeInTheDocument();
    expect(
      screen.getByText(/Aucune app ne correspond à/),
    ).toBeInTheDocument();
  });

  it("keeps the browse grid when the query is empty (non-regression)", async () => {
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    // The entry renders in the grid (and again in the hero) — assert on
    // the card testid to avoid the duplicate-text ambiguity.
    expect(screen.getByTestId("browse-card")).toBeInTheDocument();
    // No search query fired → no search grid / empty state.
    expect(screen.queryByTestId("browse-search-grid")).not.toBeInTheDocument();
    expect(screen.queryByTestId("browse-search-empty")).not.toBeInTheDocument();
  });

  it("q6_cohabitation : le lien par-noeud est additif, la grille reste (verrou 2)", async () => {
    // Sprint 75 Phase F — node-Browse est une lentille SUPPLÉMENTAIRE
    // (écran Dépôts de F-Droid), jamais un remplacement silencieux de la
    // grille (le sur-ensemble honnête).
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    const link = screen.getByTestId("browse-by-node-link");
    expect(link).toHaveTextContent("Parcourir par noeud");
    expect(link).toHaveAttribute("href", "/nodes");
    // La grille co-existe avec le lien — pas de substitution.
    expect(screen.getByTestId("browse-card")).toBeInTheDocument();
  });
});
