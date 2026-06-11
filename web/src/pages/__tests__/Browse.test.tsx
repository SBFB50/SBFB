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

  it("dedups the same app reaching the grid from two discovery channels", async () => {
    // Sprint 75 fix — an app pushed by gossip (source "direct", re-minted PUSH)
    // AND listed in a subscribed node directory (source "nodedirectory", PULL)
    // is ONE content address (same project_id + archive_hash). The aggregator
    // emits both rows additively; the grid must render ONE card, keep the
    // publisher provenance badge, and stay reachable if either channel is.
    const pid = "ab".repeat(32);
    const hash = "cd".repeat(32);
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: pid,
            project_name: "sbfb-explorer",
            archive_hash: hash,
            provenance_hash: "ef".repeat(32),
            curator_pubkey: "",
            curator_name: "Self-published",
            source: "direct",
            status: "reachable",
          }),
          makeBrowseEntry({
            project_id: pid,
            project_name: "sbfb-explorer",
            archive_hash: hash,
            curator_pubkey: "99".repeat(32),
            curator_name: "anchor",
            source: "nodedirectory",
            status: "unreachable",
          }),
        ],
      },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    // Exactly ONE card for the app reaching the grid from two channels.
    expect(screen.getAllByTestId("browse-card")).toHaveLength(1);
    // The publisher representative wins → its provenance badge survives.
    expect(screen.getByTestId("verified-badge")).toBeInTheDocument();
    // The "Upload direct" source badge is kept (publisher entry chosen).
    expect(screen.getByTestId("source-badge-direct")).toBeInTheDocument();
  });

  it("keeps distinct versions of one app as separate cards", async () => {
    // Two DIFFERENT archive_hash for the same project_id are distinct
    // deployables — the dedup keys on (project_id, archive_hash), so they
    // must NOT be collapsed.
    const pid = "ab".repeat(32);
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: pid,
            archive_hash: "11".repeat(32),
            source: "direct",
            curator_pubkey: "",
          }),
          makeBrowseEntry({
            project_id: pid,
            archive_hash: "22".repeat(32),
            source: "nodedirectory",
            curator_pubkey: "99".repeat(32),
          }),
        ],
      },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    expect(screen.getAllByTestId("browse-card")).toHaveLength(2);
  });

  it("ux-arrival : un direct inconnu va dans la section decouverte, jamais la grille", async () => {
    // Décision PO C-hybride : la grille = MES sources. Une annonce gossip
    // poussée par un nœud que je ne suis pas (`from_subscribed` absent/false)
    // rend dans la section séparée « Découvert sur le réseau » — jamais
    // mélangée, jamais « En vedette ».
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_name: "pushed-app",
            source: "direct",
            curator_pubkey: "",
            archive_hash: "aa".repeat(32),
          }),
        ],
      },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(
        screen.getByTestId("browse-discovered-section"),
      ).toBeInTheDocument();
    });
    // La carte vit dans la grille découverte, la grille principale est vide
    // (état vide honnête) et rien n'est mis « En vedette ».
    expect(screen.getByTestId("browse-discovered-grid")).toBeInTheDocument();
    expect(screen.getByText("pushed-app")).toBeInTheDocument();
    expect(screen.queryByTestId("browse-grid")).not.toBeInTheDocument();
    expect(screen.queryByText("En vedette")).not.toBeInTheDocument();
    // L'état vide est contextualisé (UXC-1) : « Aucune app » nu au-dessus
    // d'une section pleine serait déroutant.
    expect(
      screen.getByText("Aucune app dans tes sources"),
    ).toBeInTheDocument();
  });

  it("ux-arrival : un direct d'un noeud abonne reste dans la grille", async () => {
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_name: "followed-app",
            source: "direct",
            curator_pubkey: "",
            from_subscribed: true,
          }),
        ],
      },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    expect(screen.getByTestId("browse-card")).toBeInTheDocument();
    expect(
      screen.queryByTestId("browse-discovered-section"),
    ).not.toBeInTheDocument();
  });

  it("ux-arrival : la section decouverte vide n'est pas rendue", async () => {
    // Une grille 100% sollicitée (curator ici) ne rend aucune section vide.
    mockFetch({
      "/api/daemon/browse": { entries: [makeBrowseEntry()] },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    expect(
      screen.queryByTestId("browse-discovered-section"),
    ).not.toBeInTheDocument();
  });

  it("ux-arrival : la section decouverte est cappee a l'affichage", async () => {
    // 26 annonces non sollicitées distinctes → 24 cartes affichées (le cap),
    // le compteur reste honnête (« 24 sur 26 »).
    const entries = Array.from({ length: 26 }, (_, i) =>
      makeBrowseEntry({
        project_id: i.toString(16).padStart(64, "0"),
        project_name: `ambient-${i}`,
        source: "direct",
        curator_pubkey: "",
        archive_hash: i.toString(16).padStart(64, "f"),
      }),
    );
    mockFetch({
      "/api/daemon/browse": { entries },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(
        screen.getByTestId("browse-discovered-section"),
      ).toBeInTheDocument();
    });
    const grid = screen.getByTestId("browse-discovered-grid");
    expect(grid.querySelectorAll('[data-testid="browse-card"]')).toHaveLength(
      24,
    );
    expect(screen.getByTestId("browse-discovered-count")).toHaveTextContent(
      "24 sur 26",
    );
  });

  it("ux-arrival : le merge dedup OR-e from_subscribed (abonne + inconnu = mes sources)", async () => {
    // LE scénario décisif du preflight : la même app (pid, hash) arrive en
    // `direct` inconnu (représentant le plus riche) ET en `nodedirectory`
    // abonné. La classification se joue sur l'entrée FUSIONNÉE : sans le OR
    // du merge, le représentant riche (direct, from_subscribed false) la
    // ferait tomber dans « Découvert sur le réseau » — faux non-sollicité.
    const pid = "ab".repeat(32);
    const hash = "cd".repeat(32);
    mockFetch({
      "/api/daemon/browse": {
        entries: [
          makeBrowseEntry({
            project_id: pid,
            project_name: "shared-app",
            archive_hash: hash,
            provenance_hash: "ef".repeat(32),
            source: "direct",
            curator_pubkey: "",
          }),
          makeBrowseEntry({
            project_id: pid,
            project_name: "shared-app",
            archive_hash: hash,
            source: "nodedirectory",
            curator_pubkey: "99".repeat(32),
            from_subscribed: true,
          }),
        ],
      },
      "/api/daemon/search": { results: [], total: 0, took_ms: 0 },
    });

    renderBrowse();

    await waitFor(() => {
      expect(screen.getByTestId("browse-grid")).toBeInTheDocument();
    });
    // UNE carte, dans la grille (mes sources), AUCUNE section découverte.
    expect(screen.getAllByTestId("browse-card")).toHaveLength(1);
    expect(
      screen.queryByTestId("browse-discovered-section"),
    ).not.toBeInTheDocument();
    // Le représentant riche (provenance du direct) a bien survécu au merge.
    expect(screen.getByTestId("verified-badge")).toBeInTheDocument();
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
