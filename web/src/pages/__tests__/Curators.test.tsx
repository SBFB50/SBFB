// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 76 Phase B (B9, FRONTEND-COVERAGE-GAP): smoke render for `/curators`
 * (was 0-test). Covers the no-active-node fallback and the main management page
 * (curator list query) without crashing.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import Curators from "../Curators";
import { primeAuthToken } from "@/api/auth";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

function renderPage() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <Curators />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  primeAuthToken("test-token");
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => {
      const path = new URL(url).pathname;
      if (path.includes("/api/daemon/curators")) {
        return new Response(
          JSON.stringify({ entries: [], subscribed_curators: [] }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      }
      return new Response("{}", { status: 404 });
    }),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  primeAuthToken(null);
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

describe("Curators (B9 smoke)", () => {
  it("aucun noeud actif → rend le fallback", () => {
    useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
    renderPage();
    expect(screen.getByText("Aucun noeud actif")).toBeInTheDocument();
  });

  it("noeud actif → rend la page de gestion des curators", async () => {
    useProjectStore.setState({
      knownCoordinators: [{ url: COORD_URL, nickname: "test", nodeId: null }],
      activeCoordinatorUrl: COORD_URL,
    });
    renderPage();
    expect(
      await screen.findByRole("heading", { name: "Curators" }),
    ).toBeInTheDocument();
  });
});
