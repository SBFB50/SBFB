// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 76 Phase B (B9, FRONTEND-COVERAGE-GAP): smoke render for `/my-projects`
 * (was 0-test). Covers both branches — the OnboardingEmpty fallback when no
 * coordinator is known, and the coordinator-card grid when one is.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import Projects from "../Projects";
import { primeAuthToken } from "@/api/auth";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

function renderPage() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <Projects />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  primeAuthToken("test-token");
  // All coordinator calls 404 — the cards render their offline state, the page
  // header still renders (smoke).
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("{}", { status: 404 })),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  primeAuthToken(null);
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

describe("Projects (B9 smoke)", () => {
  it("aucun coordinateur connu → délègue à OnboardingEmpty", () => {
    useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
    renderPage();
    expect(screen.getByText("Bienvenue sur nexus-grid")).toBeInTheDocument();
  });

  it("un coordinateur connu → rend la grille « Mes projets »", () => {
    useProjectStore.setState({
      knownCoordinators: [{ url: COORD_URL, nickname: "test", nodeId: null }],
      activeCoordinatorUrl: COORD_URL,
    });
    renderPage();
    expect(screen.getByText("Mes projets")).toBeInTheDocument();
  });
});
