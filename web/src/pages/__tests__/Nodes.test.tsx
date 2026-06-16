// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 75 Phase F — `/nodes` (la lentille par-nœud de la découverte PULL).
 *
 * Couvre : la liste des nœuds-catalogues + les lignes « en attente »
 * (subscription sans annuaire ingéré), le cold-start state-triggered
 * (verrou 5), et l'état daemon indisponible.
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
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import Nodes from "../Nodes";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";
const NODE_A = "ee".repeat(32);
const WAITING_B = "ff".repeat(32);

function makeNode(overrides: Record<string, unknown> = {}) {
  return {
    node_id: NODE_A,
    revision: 4,
    app_count: 2,
    catalog: [
      {
        project_id: "ab".repeat(32),
        archive_hash: "cd".repeat(32),
        project_name: "Babel",
        category: "outils",
        description: "Hub de traduction",
      },
      {
        project_id: "12".repeat(32),
        archive_hash: "34".repeat(32),
        project_name: "Atlas",
        category: "cartes",
        description: "Atlas P2P",
      },
    ],
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
      return new Response(JSON.stringify({ error: "not found" }), {
        status: 404,
        headers: { "content-type": "application/json" },
      });
    }),
  );
}

function renderNodes() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={["/nodes"]}>
        <Routes>
          <Route path="/nodes" element={<Nodes />} />
          <Route
            path="/node/:nodeId"
            element={<div data-testid="node-catalog-page" />}
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
  useProjectStore.setState({
    knownCoordinators: [],
    activeCoordinatorUrl: null,
  });
});

describe("Nodes", () => {
  it("liste les noeuds-catalogues et les abonnements en attente", async () => {
    // NODE_A a un annuaire ingéré ; WAITING_B est abonné mais n'a pas encore
    // annoncé (subscribe n'est pas une ingestion synchrone) → ligne en
    // attente, jamais un écran vide menteur.
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/curators": {
        entries: [],
        subscribed_curators: [NODE_A, WAITING_B],
      },
    });
    renderNodes();

    const rows = await screen.findAllByTestId("node-row");
    expect(rows).toHaveLength(1);
    expect(rows[0]).toHaveTextContent("2 apps");
    expect(rows[0]).toHaveTextContent("rev. 4");

    const waiting = await screen.findAllByTestId("node-waiting-row");
    expect(waiting).toHaveLength(1);
    // B6 (discriminateur) : WAITING_B est abonné mais absent des `entries`
    // (aucune liste de curation ingérée) → c'est une ANCRE en attente de sa
    // première annonce, pas un curateur. Copy honnête (review F) : on décrit
    // l'état observé sans promettre une annonce future.
    expect(waiting[0]).toHaveAttribute("data-kind", "anchor");
    expect(waiting[0]).toHaveTextContent(
      "Ancre abonnée — en attente de sa première annonce d'annuaire.",
    );
    // L'identité abonnée AVEC annuaire ne double pas en ligne d'attente.
    expect(screen.queryByTestId("nodes-cold-start")).not.toBeInTheDocument();
  });

  it("b6 : distingue un curateur (liste ingérée) d'une ancre en attente", async () => {
    // L'attention set est UNIQUE (Q3/DQ3) : une identité abonnée dont on a
    // déjà ingéré une liste de curation signée curate — même sans annuaire de
    // nœud. On la libelle « curateur », distincte d'une ancre muette en
    // attente de sa première annonce. CURATOR_B est dans `entries`, ANCHOR_C
    // ne l'est pas ; aucun des deux n'a d'annuaire ingéré (absent de /nodes).
    const CURATOR_B = "bb".repeat(32);
    const ANCHOR_C = "cc".repeat(32);
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/curators": {
        entries: [
          {
            list: {
              version: 1,
              curator_pubkey: Array.from({ length: 32 }, () => 0xbb),
              curator_name: "Curateur B",
              created_at: 1_700_000_000,
              revision: 1,
              entries: [],
            },
            curator_pubkey: Array.from({ length: 32 }, () => 0xbb),
            signature: Array.from({ length: 64 }, () => 0),
          },
        ],
        subscribed_curators: [NODE_A, CURATOR_B, ANCHOR_C],
      },
    });
    renderNodes();

    const rows = await screen.findAllByTestId("node-waiting-row");
    expect(rows).toHaveLength(2);
    const curatorRow = rows.find(
      (r) => r.getAttribute("data-kind") === "curator",
    );
    const anchorRow = rows.find(
      (r) => r.getAttribute("data-kind") === "anchor",
    );
    expect(curatorRow).toBeDefined();
    expect(anchorRow).toBeDefined();
    expect(curatorRow).toHaveTextContent(
      "Curateur — listes de curation signées suivies ; aucun annuaire de nœud publié.",
    );
    expect(anchorRow).toHaveTextContent(
      "Ancre abonnée — en attente de sa première annonce d'annuaire.",
    );
  });

  it("rend une carte erreur sur un drift de schema (branche isError, lecon SEARCH-VIEW)", async () => {
    // Un body /nodes qui viole le schéma (.strict() enveloppe) fait jeter
    // ApiProtocolError dans la query → isError. Sans cette branche la page
    // resterait sur « Chargement... » pour toujours (classe
    // SEARCH-VIEW-THROW-SKELETON, carry S73).
    mockFetch({
      "/api/daemon/nodes": { nodes: [], unexpected_envelope_key: 1 },
      "/api/daemon/curators": { entries: [], subscribed_curators: [] },
    });
    renderNodes();
    expect(await screen.findByText("Erreur reseau")).toBeInTheDocument();
  });

  it("navigue vers le catalogue du noeud au clic", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/curators": { entries: [], subscribed_curators: [NODE_A] },
    });
    renderNodes();
    await user.click(await screen.findByTestId("node-row"));
    expect(await screen.findByTestId("node-catalog-page")).toBeInTheDocument();
  });

  it("cold-start : aucun noeud connu rend le CTA ajouter une ancre (verrou 5)", async () => {
    // La suggestion est déclenchée par l'état observé (0 nœud), jamais
    // poussée au publish — et elle ouvre le dialog, pas d'auto-subscribe.
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/nodes": { nodes: [] },
      "/api/daemon/curators": { entries: [], subscribed_curators: [] },
    });
    renderNodes();

    expect(await screen.findByTestId("nodes-cold-start")).toBeInTheDocument();
    expect(
      screen.getByText("Aucun noeud-catalogue connu"),
    ).toBeInTheDocument();
    await user.click(screen.getByTestId("cold-start-add-anchor"));
    expect(await screen.findByTestId("add-anchor-dialog")).toBeInTheDocument();
  });

  it("rend la banniere daemon indisponible sur 503", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response("{}", { status: 503 })),
    );
    renderNodes();
    expect(
      await screen.findByTestId("daemon-offline-banner"),
    ).toBeInTheDocument();
  });

  it("ux-arrival : les noeuds observes rendent une section dediee + CTA s'abonner pre-rempli", async () => {
    // Un éditeur d'annuaire entendu par gossip SANS abonnement apparaît en
    // métadonnée seulement (le daemon ne fetch jamais son catalogue) avec un
    // CTA d'abonnement explicite qui ouvre le dialog pré-rempli sur SON
    // identité — la soumission reste un geste utilisateur (verrou 3/5).
    const user = userEvent.setup();
    const OBSERVED_C = "aa".repeat(32);
    mockFetch({
      "/api/daemon/nodes": {
        nodes: [makeNode()],
        observed: [{ node_id: OBSERVED_C, last_seen: 1_700_000_000 }],
      },
      "/api/daemon/curators": { entries: [], subscribed_curators: [NODE_A] },
    });
    renderNodes();

    expect(
      await screen.findByTestId("nodes-observed-section"),
    ).toBeInTheDocument();
    const row = screen.getByTestId("node-observed-row");
    // Copy prudente (SEC-UXARR-3) : on décrit l'annonce ENTENDUE, jamais une
    // agence prouvée du nœud (l'identité d'enveloppe n'est pas authentifiée).
    expect(row).toHaveTextContent(
      "Annonce entendue sur le réseau — abonne-toi pour voir son catalogue.",
    );
    await user.click(screen.getByTestId("observed-subscribe-cta"));
    expect(await screen.findByTestId("add-anchor-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("anchor-pubkey-input")).toHaveValue(OBSERVED_C);
  });

  it("ux-arrival : le prefill est efface a la reouverture manuelle du dialog", async () => {
    // CTA observed → champ pré-rempli ; fermeture ; réouverture par le
    // bouton manuel → champ VIDE. Le pré-remplissage ne survit jamais à
    // l'intention explicite qui l'a déclenché (verrou 3 : placeholder
    // inerte par défaut).
    const user = userEvent.setup();
    const OBSERVED_C = "aa".repeat(32);
    mockFetch({
      "/api/daemon/nodes": {
        nodes: [makeNode()],
        observed: [{ node_id: OBSERVED_C, last_seen: 1_700_000_000 }],
      },
      "/api/daemon/curators": { entries: [], subscribed_curators: [NODE_A] },
    });
    renderNodes();

    await user.click(await screen.findByTestId("observed-subscribe-cta"));
    expect(screen.getByTestId("anchor-pubkey-input")).toHaveValue(OBSERVED_C);
    await user.keyboard("{Escape}");
    await waitFor(() => {
      expect(
        screen.queryByTestId("add-anchor-dialog"),
      ).not.toBeInTheDocument();
    });

    await user.click(screen.getByTestId("nodes-add-anchor"));
    expect(await screen.findByTestId("add-anchor-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("anchor-pubkey-input")).toHaveValue("");
  });

  it("ux-arrival : un noeud observe seul suffit a ecarter le cold-start", async () => {
    // « Aucun nœud-catalogue connu » serait un mensonge : on en a entendu un.
    mockFetch({
      "/api/daemon/nodes": {
        nodes: [],
        observed: [{ node_id: "bb".repeat(32), last_seen: 1_700_000_000 }],
      },
      "/api/daemon/curators": { entries: [], subscribed_curators: [] },
    });
    renderNodes();

    expect(
      await screen.findByTestId("nodes-observed-section"),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("nodes-cold-start")).not.toBeInTheDocument();
  });

  it("ux-arrival : tolerance Zod — cle observed absente et row additive", async () => {
    // (1) Un daemon antérieur à UX-ARRIVAL n'émet pas `observed` : la page
    // parse et rend sans section. (2) Une row observed portant un champ
    // additif futur ne brique pas la page (règle P37 : rows tolérantes,
    // seule l'ENVELOPPE est .strict()).
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/curators": { entries: [], subscribed_curators: [NODE_A] },
    });
    const first = renderNodes();
    expect(await screen.findByTestId("nodes-list")).toBeInTheDocument();
    expect(
      screen.queryByTestId("nodes-observed-section"),
    ).not.toBeInTheDocument();
    first.unmount();

    mockFetch({
      "/api/daemon/nodes": {
        nodes: [],
        observed: [
          {
            node_id: "cc".repeat(32),
            last_seen: 1_700_000_000,
            future_additive_field: "ignored",
          },
        ],
      },
      "/api/daemon/curators": { entries: [], subscribed_curators: [] },
    });
    renderNodes();
    expect(
      await screen.findByTestId("nodes-observed-section"),
    ).toBeInTheDocument();
  });

  it("pas de cold-start quand l'etat des subscriptions est INCONNU (GAP Codex R2)", async () => {
    // /nodes répond une liste vide mais /curators est indisponible : les
    // abonnements sont INCONNUS, pas vides — afficher « aucun noeud connu »
    // serait faux. Le CTA cold-start exige des subscriptions CONNUES-vides.
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        const path = new URL(url).pathname;
        if (path.includes("/api/daemon/nodes")) {
          return new Response(JSON.stringify({ nodes: [] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        // /curators indisponible.
        return new Response("{}", { status: 503 });
      }),
    );
    renderNodes();
    // La page rend (pas de skeleton infini)...
    expect(await screen.findByTestId("nodes-add-anchor")).toBeInTheDocument();
    // ...mais JAMAIS le claim « aucun noeud-catalogue connu ».
    await waitFor(() => {
      expect(screen.queryByTestId("nodes-cold-start")).not.toBeInTheDocument();
    });
  });
});
