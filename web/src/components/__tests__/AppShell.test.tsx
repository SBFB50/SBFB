// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 74 Phase A — AppShell rename smoke test.
 *
 * The shell vocabulary moves from "coordinateur" to "noeud"/"reseau"
 * (PO Q8 "toute l'UI"): the nav publishes under "Publier" and the picker's
 * empty state invites "Se connecter a un noeud".
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import { AppShell } from "../AppShell";
import { useProjectStore } from "@/stores/projectStore";

function renderShell() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={["/browse"]}>
        <Routes>
          <Route element={<AppShell />}>
            <Route path="/browse" element={<div data-testid="outlet" />} />
          </Route>
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      new Response(JSON.stringify({ detail: "not found" }), { status: 404 }),
    ),
  );
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

afterEach(() => {
  vi.unstubAllGlobals();
  useProjectStore.setState({ knownCoordinators: [], activeCoordinatorUrl: null });
});

describe("AppShell rename", () => {
  it("coordinator_renamed_to_node_in_shell", () => {
    renderShell();

    // Nav rail publishes under the "Publier" intention.
    expect(screen.getByText("Publier")).toBeInTheDocument();
    // The empty picker invites connecting to a node, not "adding a coordinator".
    expect(screen.getByText("Se connecter a un noeud")).toBeInTheDocument();

    // No visible "coordinateur" / "Coordinateurs" string survives.
    expect(screen.queryByText(/coordinateur/i)).not.toBeInTheDocument();
  });
});
