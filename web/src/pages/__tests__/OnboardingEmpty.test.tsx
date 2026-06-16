// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 76 Phase B (B9, FRONTEND-COVERAGE-GAP): smoke render for the
 * empty-state onboarding page (was 0-test). Proves the page mounts and walks
 * the user through starting `nexus-shell-daemon` without crashing.
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import OnboardingEmpty from "../OnboardingEmpty";

function renderPage() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <OnboardingEmpty />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("OnboardingEmpty (B9 smoke)", () => {
  it("rend l'écran d'accueil + la commande de démarrage du daemon", () => {
    renderPage();
    expect(screen.getByText("Bienvenue sur nexus-grid")).toBeInTheDocument();
    expect(
      screen.getByText("nexus-shell-daemon start"),
    ).toBeInTheDocument();
  });
});
