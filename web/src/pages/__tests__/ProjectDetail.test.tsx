// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 76 Phase B (B9, FRONTEND-COVERAGE-GAP): smoke render for
 * `/project/:name` (was 0-test). Covers the "projet introuvable" branch and the
 * resolved-coordinator detail view (hero + tabs) without crashing.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import ProjectDetail from "../ProjectDetail";
import { primeAuthToken } from "@/api/auth";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

function renderAt(path: string) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route path="/project/:name" element={<ProjectDetail />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  primeAuthToken("test-token");
  // All coordinator endpoints 404 — the queries fail gracefully (retry:false),
  // the hero + tabs still render their empty/loading state (smoke).
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

describe("ProjectDetail (B9 smoke)", () => {
  it("nom inconnu → rend « Projet introuvable »", () => {
    useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
    renderAt("/project/unknown");
    expect(screen.getByText("Projet introuvable")).toBeInTheDocument();
  });

  it("coordinateur résolu → rend la vue détail (héros + onglets)", () => {
    useProjectStore.setState({
      knownCoordinators: [{ url: COORD_URL, nickname: "test", nodeId: null }],
      activeCoordinatorUrl: COORD_URL,
    });
    renderAt("/project/test");
    // Le héros rend le fallback de nom de projet même quand la query échoue.
    expect(screen.getByText("Projet")).toBeInTheDocument();
  });
});
