// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 11 Phase C — BrowsedProject page unit tests.
 *
 * Tests cover:
 * - routing / back link
 * - "no coordinator" state
 * - "project not found" state (daemon offline or missing entry)
 * - sidebar rendering with project metadata
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
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import BrowsedProject from "../BrowsedProject";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";
const LOCAL_NODE_ID = "aa".repeat(32);
const REMOTE_NODE_ID = "bb".repeat(32);

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
  it("renders a back link to /browse", () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: { entries: [] },
      },
      "/health": {
        status: "ok",
        project: "test",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
    });
    renderPage(LOCAL_NODE_ID);
    const backLink = screen.getByTestId("back-to-browse");
    expect(backLink).toBeInTheDocument();
    expect(backLink).toHaveAttribute("href", "/browse");
  });

  it("renders 'no coordinator' when no active coordinator is set", () => {
    useProjectStore.setState({
      knownCoordinators: [],
      activeCoordinatorUrl: null,
    });
    renderPage(LOCAL_NODE_ID);
    expect(
      screen.getByText(/Aucun coordinateur sélectionné/),
    ).toBeInTheDocument();
  });

  it("renders 'project not found' when entry is not in browse list", async () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: { entries: [] },
      },
      "/health": {
        status: "ok",
        project: "test",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
    });
    renderPage("ff".repeat(32));
    await waitFor(() => {
      expect(screen.getByTestId("project-not-found")).toBeInTheDocument();
    });
    expect(screen.getByText(/Projet introuvable/)).toBeInTheDocument();
  });

  it("renders sidebar with project metadata when entry is found", async () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: { entries: [makeBrowseEntry()] },
      },
      "/health": {
        status: "ok",
        project: "gov",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("project-sidebar")).toBeInTheDocument();
    });
    expect(screen.getByText("gouvernance")).toBeInTheDocument();
    expect(
      screen.getByText("Application de gouvernance"),
    ).toBeInTheDocument();
    expect(screen.getByText("FlowUP")).toBeInTheDocument();
  });

  it("renders remote placeholder for non-local project", async () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: {
          entries: [makeBrowseEntry({ project_id: REMOTE_NODE_ID })],
        },
      },
      "/health": {
        status: "ok",
        project: "test",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
    });
    renderPage(REMOTE_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("remote-placeholder")).toBeInTheDocument();
    });
    expect(screen.getByText(/Projet distant/)).toBeInTheDocument();
    expect(
      screen.getByText(/noeud distant/),
    ).toBeInTheDocument();
  });

  it("renders local project apps view for matching node_id", async () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: { entries: [makeBrowseEntry()] },
      },
      "/health": {
        status: "ok",
        project: "gov",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
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
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: { entries: [makeBrowseEntry()] },
      },
      "/health": {
        status: "ok",
        project: "gov",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("no-apps")).toBeInTheDocument();
    });
    expect(
      screen.getByText(/Aucune application installée/),
    ).toBeInTheDocument();
  });

  it("renders source badge 'Auto-publié' for direct entries in sidebar", async () => {
    mockFetch({
      "/daemon/browse": {
        kind: "data",
        status: 200,
        body: {
          entries: [makeBrowseEntry({ source: "direct" })],
        },
      },
      "/health": {
        status: "ok",
        project: "gov",
        node_id: LOCAL_NODE_ID,
        doc_id: null,
        author_id: null,
        version: "1.0.0",
      },
      "/app": { apps: [], count: 0 },
    });
    renderPage(LOCAL_NODE_ID);
    await waitFor(() => {
      expect(screen.getByTestId("project-sidebar")).toBeInTheDocument();
    });
    expect(screen.getByText("Auto-publié")).toBeInTheDocument();
  });
});
