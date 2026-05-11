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
      expect(screen.getByTestId("deploy-success")).toBeDefined();
    });
    expect(screen.getByText("abc123def456")).toBeDefined();
    expect(screen.getByText("prov789")).toBeDefined();
  });
});
