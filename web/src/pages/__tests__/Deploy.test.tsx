// SPDX-License-Identifier: AGPL-3.0-or-later

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

import Deploy from "../Deploy";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

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

function renderPage() {
  const qc = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={["/deploy"]}>
        <Routes>
          <Route path="/deploy" element={<Deploy />} />
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

describe("Deploy", () => {
  it("affiche le formulaire avec les champs requis", () => {
    mockFetch({});
    renderPage();
    expect(screen.getByTestId("repo-url")).toBeDefined();
    expect(screen.getByTestId("project-name")).toBeDefined();
    expect(screen.getByTestId("description")).toBeDefined();
    expect(screen.getByTestId("deploy-submit")).toBeDefined();
  });

  it("affiche le resultat apres un deploiement reussi", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/v1/deploy-from-repo": {
        deployed: true,
        hash: "abc123def456",
        provenance_hash: "prov789",
        commit_sha: "cccc".repeat(10),
      },
    });
    renderPage();

    await user.type(screen.getByTestId("repo-url"), "https://github.com/test/repo.git");
    await user.type(screen.getByTestId("project-name"), "test-app");
    await user.click(screen.getByTestId("deploy-submit"));

    await waitFor(() => {
      expect(screen.getByTestId("deploy-success")).toBeInTheDocument();
    });
    // Sprint 74 Phase A — the technical hashes are folded behind "Details
    // techniques" by default; expand them before asserting.
    await user.click(screen.getByTestId("deploy-tech-toggle"));
    await waitFor(() => {
      expect(screen.getByText("abc123def456")).toBeInTheDocument();
    });
    expect(screen.getByText("prov789")).toBeInTheDocument();
  });

  // Sprint 74 Phase A — the success card folds the cryptographic detail and
  // exposes the human truth, with ZERO host/target field anywhere.
  it("publish_success_card_folds_hashes", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/v1/deploy-from-repo": {
        deployed: true,
        hash: "abc123def456",
        provenance_hash: "prov789",
        commit_sha: "cccc".repeat(10),
      },
    });
    renderPage();

    await user.type(
      screen.getByTestId("repo-url"),
      "https://github.com/test/repo.git",
    );
    await user.type(screen.getByTestId("project-name"), "test-app");
    await user.click(screen.getByTestId("deploy-submit"));

    await waitFor(() => {
      expect(screen.getByTestId("deploy-success")).toBeInTheDocument();
    });

    // Human truth surfaced, hash folded by default.
    expect(screen.getByTestId("deploy-online-pill")).toBeInTheDocument();
    expect(screen.getByText("App publiee et en ligne")).toBeInTheDocument();
    expect(screen.queryByTestId("deploy-tech-details")).not.toBeInTheDocument();
    expect(screen.queryByText("abc123def456")).not.toBeInTheDocument();

    // No host/target field: publishing is a local signed identity act.
    expect(screen.queryByText(/Mon serveur/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/h[oô]te/i)).not.toBeInTheDocument();

    // Expanding reveals the technical detail.
    await user.click(screen.getByTestId("deploy-tech-toggle"));
    await waitFor(() => {
      expect(screen.getByTestId("deploy-tech-details")).toBeInTheDocument();
    });
    expect(screen.getByText("abc123def456")).toBeInTheDocument();
  });

  // Sprint 74 Phase A — greffe D : "La remettre en ligne" prefills the form.
  it("prefills repo_url and project_name from query params", () => {
    mockFetch({});
    const qc = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0 },
        mutations: { retry: false },
      },
    });
    render(
      <QueryClientProvider client={qc}>
        <MemoryRouter
          initialEntries={[
            "/deploy?repo_url=https%3A%2F%2Fcodeberg.org%2Fme%2Fapp.git&project_name=app",
          ]}
        >
          <Routes>
            <Route path="/deploy" element={<Deploy />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    );
    expect(screen.getByTestId("repo-url")).toHaveValue(
      "https://codeberg.org/me/app.git",
    );
    expect(screen.getByTestId("project-name")).toHaveValue("app");
  });

  // Sprint 74 Phase A — the rename surfaces in the empty-state wall.
  it("renders the renamed empty-state wall", () => {
    useProjectStore.setState({
      knownCoordinators: [],
      activeCoordinatorUrl: null,
    });
    mockFetch({});
    renderPage();
    expect(screen.getByText("Aucun noeud actif")).toBeInTheDocument();
    expect(screen.queryByText(/coordinateur/i)).not.toBeInTheDocument();
  });
});
