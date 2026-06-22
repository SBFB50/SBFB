// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 77 Phase J — ShardSessionPanel (« Calcul en réseau ») unit tests.
 *
 * Couvre : l'état vide par défaut + les deux intentions FR (PO-9, zéro jargon),
 * le lookup « rejoindre » qui interroge GET /api/daemon/shard-session/{id}
 * (identifiant inconnu → « aucune session »), et le rendu du statut AGRÉGÉ
 * d'une session trouvée — le nombre de machines, JAMAIS une identité de membre
 * (THREAT_MODEL §16 SI-3/SI-4).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import ShardSessionPanel from "../ShardSessionPanel";
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

function renderPanel() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={["/compute"]}>
        <ShardSessionPanel />
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
  useProjectStore.setState({
    knownCoordinators: [],
    activeCoordinatorUrl: null,
  });
});

describe("ShardSessionPanel", () => {
  it("rend les deux intentions FR et l'état vide par défaut", () => {
    // Aucune session vivante en Phase J (pas de store côté daemon) → l'état par
    // défaut est « Aucune session active », pas un écran cassé. Les CTA sont des
    // intentions utilisateur, zéro jargon shard/ALPN/ComputeGroup (PO-9).
    mockFetch({});
    renderPanel();

    expect(screen.getByTestId("shard-session-panel")).toBeInTheDocument();
    expect(screen.getByTestId("cta-launch-large-model")).toHaveTextContent(
      "Lancer un gros modèle en réseau",
    );
    expect(screen.getByTestId("cta-join-compute-group")).toHaveTextContent(
      "Rejoindre un groupe de calcul",
    );
    expect(screen.getByTestId("shard-session-empty")).toBeInTheDocument();
    expect(screen.getByText("Aucune session active")).toBeInTheDocument();

    // Le jargon technique ne fuit jamais en surface (PO-9).
    expect(screen.queryByText(/shard/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/ALPN/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/ComputeGroup/i)).not.toBeInTheDocument();
  });

  it("rejoindre : un identifiant inconnu affiche « aucune session »", async () => {
    // Phase J : le daemon n'a pas de session vivante → il renvoie toujours
    // { found:false, session:null } (200, pas 404). Le panneau le rend comme un
    // état honnête, pas comme une erreur transport.
    mockFetch({
      "/api/daemon/shard-session": { found: false, session: null },
    });
    renderPanel();

    await userEvent.click(screen.getByTestId("cta-join-compute-group"));
    await userEvent.type(
      screen.getByTestId("shard-group-id-input"),
      "session-inconnue",
    );
    await userEvent.click(screen.getByTestId("shard-join-submit"));

    expect(
      await screen.findByTestId("shard-session-not-found"),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("shard-session-status")).not.toBeInTheDocument();
  });

  it("rejoindre : une session trouvée affiche le nombre de machines, jamais une identité", async () => {
    // Le statut n'expose qu'un AGRÉGAT : member_count. Aucune pubkey de membre
    // ne transite (whitelist producteur, SI-3/SI-4). Assertion NÉGATIVE de bout
    // en bout : même si le producteur fuyait une identité dans la row, le schéma
    // Zod tolérant la strippe et le panneau ne peut pas la rendre.
    const LEAK =
      "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    mockFetch({
      "/api/daemon/shard-session": {
        found: true,
        session: {
          session_id: "session-xyz-1234567890",
          member_count: 3,
          // Sentinelles d'identité — doivent être strippées par la row schema
          // (PAS .strict()) et ne JAMAIS atteindre le DOM.
          worker_pubkey: LEAK,
          initiator: LEAK,
        },
      },
    });
    renderPanel();

    await userEvent.click(screen.getByTestId("cta-join-compute-group"));
    await userEvent.type(
      screen.getByTestId("shard-group-id-input"),
      "session-xyz-1234567890",
    );
    await userEvent.click(screen.getByTestId("shard-join-submit"));

    const status = await screen.findByTestId("shard-session-status");
    expect(status).toBeInTheDocument();
    expect(screen.getByTestId("shard-member-count")).toHaveTextContent("3");
    expect(screen.getByText("Machines participantes")).toBeInTheDocument();
    // Aucune identité de membre n'atteint jamais le DOM.
    expect(screen.queryByText(LEAK)).not.toBeInTheDocument();
  });

  it("sans noeud actif, invite à se connecter", () => {
    useProjectStore.setState({
      knownCoordinators: [],
      activeCoordinatorUrl: null,
    });
    renderPanel();
    expect(screen.getByText("Aucun noeud actif")).toBeInTheDocument();
    expect(screen.queryByTestId("shard-session-panel")).not.toBeInTheDocument();
  });
});
