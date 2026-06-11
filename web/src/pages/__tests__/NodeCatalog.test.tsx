// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 75 Phase F — `/node/:nodeId` (catalogue d'un nœud).
 *
 * Couvre l'exigence PO verrou 4 (lock-4) :
 *  (a) la carte ouvre la preuve de provenance signée AUTEUR
 *      (`VerificationDetail`, fetch par projectId) ;
 *  (b) une version non source-vérifiée annoncée par son éditeur
 *      (`is_open_source=false`, hash distinct) rend « Version dérivée »,
 *      jamais le badge de l'original ;
 *  (c) le nœud-catalogue est étiqueté source de découverte, jamais rendu
 *      comme autorité.
 * Plus le badge Q7 composé front-side (ancre unreachable + peer_count > 0)
 * et le CTA pull/seed qui épingle la version EXACTE affichée.
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

import NodeCatalog from "../NodeCatalog";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";
const ANCHOR = "ee".repeat(32);
const PID_VERIFIED = "ab".repeat(32);
const HASH_VERIFIED = "cd".repeat(32);
const PID_DERIVED = "12".repeat(32);
const HASH_DERIVED = "34".repeat(32);
const PID_ORPHAN = "56".repeat(32);
const HASH_ORPHAN = "78".repeat(32);

function makeNode(overrides: Record<string, unknown> = {}) {
  return {
    node_id: ANCHOR,
    revision: 7,
    app_count: 3,
    catalog: [
      {
        project_id: PID_VERIFIED,
        archive_hash: HASH_VERIFIED,
        project_name: "Babel",
        category: "outils",
        description: "Hub de traduction",
      },
      {
        project_id: PID_DERIVED,
        archive_hash: HASH_DERIVED,
        project_name: "Babel remix",
        category: "outils",
        description: "Version modifiee dans l'atelier",
      },
      // Catalogue-only : sa SEULE présence /browse est non-direct (entrées
      // nodedirectory + curator dans la fixture) — la carte ne doit porter
      // AUCUN badge (le scénario décisif du P1 review F).
      {
        project_id: PID_ORPHAN,
        archive_hash: HASH_ORPHAN,
        project_name: "Orpheline",
        category: "outils",
        description: "Decouverte uniquement par annuaire",
      },
    ],
    ...overrides,
  };
}

/**
 * Les annonces /browse croisées par les cartes. ORDRE DÉLIBÉRÉ (P1 review F,
 * pattern anti-faux-vert T1/S75-A) : les entrées NON-direct (nodedirectory,
 * curator) sont placées EN TÊTE, AVANT les annonces d'éditeur — ainsi un
 * `find()` sans le filtre `source === "direct"` matcherait d'abord une
 * entrée au flag hardcodé false et le badge vérifié disparaîtrait : le test
 * lock-4b casse si l'exclusion régresse (elle est load-bearing, plus
 * seulement décorative). En prod l'ordre du tri (project_id, curator_pubkey)
 * ne garantit pas l'éditeur d'abord — c'est précisément le danger.
 */
function browseFixture(anchorStatus: "reachable" | "unreachable") {
  return {
    entries: [
      // EN TÊTE : l'entrée nodedirectory de l'ANCRE pour la version vérifiée
      // — statut de sonde (badge Q7) + is_open_source=false PAR DÉFAUT (la
      // 3e boucle aggregate hardcode false, le catalogue ne porte pas le
      // flag) qui ne doit JAMAIS être lu comme un claim d'éditeur.
      {
        project_id: PID_VERIFIED,
        project_name: "Babel",
        category: "outils",
        description: "Hub de traduction",
        curator_pubkey: ANCHOR,
        curator_name: "Node catalog",
        source: "nodedirectory",
        status: anchorStatus,
        last_probed_at: null,
        archive_hash: HASH_VERIFIED,
        is_open_source: false,
      },
      // L'app orpheline n'existe QUE via des sources non-direct : son entrée
      // d'annuaire...
      {
        project_id: PID_ORPHAN,
        project_name: "Orpheline",
        category: "outils",
        description: "Decouverte uniquement par annuaire",
        curator_pubkey: ANCHOR,
        curator_name: "Node catalog",
        source: "nodedirectory",
        status: anchorStatus,
        last_probed_at: null,
        archive_hash: HASH_ORPHAN,
        is_open_source: false,
      },
      // ...et une entrée curator portant MÊME (pid, hash) — forme future
      // hypothétique (les entrées curator réelles n'ont pas de hash) qui
      // pinne que le prédicat éditeur exige source === "direct", pas
      // seulement != nodedirectory (piège latent review F : la boucle
      // curator hardcode aussi is_open_source:false).
      {
        project_id: PID_ORPHAN,
        project_name: "Orpheline",
        category: "outils",
        description: "Decouverte uniquement par annuaire",
        curator_pubkey: "44".repeat(32),
        curator_name: "Curator",
        source: "curator",
        status: "reachable",
        last_probed_at: null,
        archive_hash: HASH_ORPHAN,
        is_open_source: false,
      },
      // Les annonces des ÉDITEURS (source direct, le seul canal qui porte le
      // vrai flag) — APRÈS les non-direct, cf. doc de la fixture.
      {
        project_id: PID_VERIFIED,
        project_name: "Babel",
        category: "outils",
        description: "Hub de traduction",
        curator_pubkey: "99".repeat(32),
        curator_name: "Auteur",
        source: "direct",
        status: "reachable",
        last_probed_at: null,
        archive_hash: HASH_VERIFIED,
        provenance_hash: "55".repeat(32),
        is_open_source: true,
      },
      {
        project_id: PID_DERIVED,
        project_name: "Babel remix",
        category: "outils",
        description: "Version modifiee dans l'atelier",
        curator_pubkey: "77".repeat(32),
        curator_name: "Forkeur",
        source: "direct",
        status: "reachable",
        last_probed_at: null,
        archive_hash: HASH_DERIVED,
        is_open_source: false,
      },
    ],
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

function renderCatalog(nodeId: string = ANCHOR) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={[`/node/${nodeId}`]}>
        <Routes>
          <Route path="/node/:nodeId" element={<NodeCatalog />} />
          <Route path="/nodes" element={<div data-testid="nodes-page" />} />
          <Route
            path="/browse/:projectId"
            element={<div data-testid="project-page" />}
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

describe("NodeCatalog", () => {
  it("rend les cartes du catalogue avec l'en-tete source-pas-autorite (lock-4c)", async () => {
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/browse": browseFixture("reachable"),
    });
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    expect(cards).toHaveLength(3);
    // Le nœud est étiqueté SOURCE DE DÉCOUVERTE — jamais une autorité.
    const label = screen.getByTestId("node-source-label");
    expect(label).toHaveTextContent("pas une autorite");
    expect(label).toHaveTextContent("la signature de son auteur");
    // L'en-tête du nœud ne porte AUCUN badge d'autorité de provenance.
    const header = screen.getByTestId("node-catalog-header");
    expect(
      header.querySelector('[data-testid="catalog-verified-badge"]'),
    ).toBeNull();
  });

  it("lock-4b : la version derivee porte son marqueur, jamais le badge de l'original", async () => {
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/browse": browseFixture("reachable"),
    });
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    const verifiedCard = cards.find((c) => c.textContent?.includes("Babel") && !c.textContent?.includes("remix"))!;
    const derivedCard = cards.find((c) => c.textContent?.includes("Babel remix"))!;

    // La version de l'auteur (is_open_source=true chez son éditeur) porte le
    // badge vérifiable.
    await waitFor(() => {
      expect(
        verifiedCard.querySelector('[data-testid="catalog-verified-badge"]'),
      ).not.toBeNull();
    });
    expect(
      verifiedCard.querySelector('[data-testid="catalog-derived-badge"]'),
    ).toBeNull();

    // Le fork re-signé (hash distinct, is_open_source=false) porte « Version
    // dérivée » — jamais le badge de l'original.
    expect(
      derivedCard.querySelector('[data-testid="catalog-derived-badge"]'),
    ).not.toBeNull();
    expect(derivedCard).toHaveTextContent("Version dérivée");
    expect(
      derivedCard.querySelector('[data-testid="catalog-verified-badge"]'),
    ).toBeNull();
  });

  it("lock-4b decisif : une row catalogue sans annonce d'editeur ne porte AUCUN badge", async () => {
    // LE scénario P1 (review F) : la seule présence /browse d'Orpheline est
    // non-direct (nodedirectory + curator, flags hardcodés false par
    // l'aggregateur). Si l'exclusion source==="direct" régresse, cette app
    // légitime serait faussement marquée « Version dérivée » — violation du
    // verrou 4. Ni badge vérifié, ni marqueur dérivé : pas de claim.
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/browse": browseFixture("reachable"),
    });
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    const orphanCard = cards.find((c) => c.textContent?.includes("Orpheline"))!;
    expect(orphanCard).toBeDefined();
    // Laisse les queries se poser puis vérifie l'absence STABLE des deux
    // badges sur plusieurs frames.
    await waitFor(() => {
      expect(
        orphanCard.querySelector('[data-testid="catalog-derived-badge"]'),
      ).toBeNull();
      expect(
        orphanCard.querySelector('[data-testid="catalog-verified-badge"]'),
      ).toBeNull();
    });
  });

  it("lock-4a : la carte ouvre la preuve de provenance signee auteur", async () => {
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/browse": browseFixture("reachable"),
      "/provenance": {
        record: {
          repo_url: "https://codeberg.org/auteur/babel",
          commit_sha: "ab".repeat(20),
          artifact_hash: HASH_VERIFIED,
          signature: "sig".repeat(20),
          node_id: "99".repeat(32),
          timestamp: "2026-06-01T10:00:00Z",
          schema_version: 1,
        },
        verified: true,
        provenance_hash: "55".repeat(32),
      },
    });
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    const verifiedCard = cards.find((c) => c.textContent?.includes("Babel") && !c.textContent?.includes("remix"))!;
    await user.click(
      verifiedCard.querySelector<HTMLButtonElement>(
        '[data-testid="catalog-provenance"]',
      )!,
    );

    // VerificationDetail rend la signature AUTEUR — l'autorité, quel que
    // soit le nœud qui seede le catalogue.
    expect(await screen.findByTestId("verification-detail")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByTestId("verification-result")).toHaveTextContent(
        "Signature valide",
      );
    });
    expect(screen.getByTestId("prov-node-id")).toBeInTheDocument();
    // Le record prouve la version listée (artifact_hash identique) — pas
    // d'avertissement de version.
    expect(
      screen.queryByTestId("version-mismatch-warning"),
    ).not.toBeInTheDocument();
  });

  it("verrou 4 residuel : une preuve d'une AUTRE version est marquee comme telle", async () => {
    // Le record provenance est keyé par projectId : il peut prouver d'autres
    // octets que la version de la row cliquée. « Signature valide » seul
    // serait une demi-vérité — le dialog doit avertir (review F P3 pris
    // in-phase, exigence PO verrou 4).
    const user = userEvent.setup();
    mockFetch({
      "/api/daemon/nodes": { nodes: [makeNode()] },
      "/api/daemon/browse": browseFixture("reachable"),
      "/provenance": {
        record: {
          repo_url: "https://codeberg.org/auteur/babel",
          commit_sha: "ab".repeat(20),
          // Une AUTRE version que HASH_VERIFIED affichée par la row.
          artifact_hash: "99".repeat(32),
          signature: "sig".repeat(20),
          node_id: "99".repeat(32),
          timestamp: "2026-06-01T10:00:00Z",
          schema_version: 1,
        },
        verified: true,
        provenance_hash: "55".repeat(32),
      },
    });
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    const verifiedCard = cards.find((c) => c.textContent?.includes("Babel") && !c.textContent?.includes("remix"))!;
    await user.click(
      verifiedCard.querySelector<HTMLButtonElement>(
        '[data-testid="catalog-provenance"]',
      )!,
    );

    expect(await screen.findByTestId("verification-detail")).toBeInTheDocument();
    expect(
      await screen.findByTestId("version-mismatch-warning"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/une autre version que celle affichee/),
    ).toBeInTheDocument();
  });

  it("badge Q7 : ancre morte + seeder recent rend joignable-via-un-pair (version-exact)", async () => {
    // Spy : la requête seed-count doit porter le hash EXACT de la row
    // (sinon le signal redeviendrait version-agnostique — review F).
    const urls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        urls.push(url);
        const path = new URL(url).pathname;
        if (path.includes("/api/daemon/nodes")) {
          return new Response(JSON.stringify({ nodes: [makeNode()] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/browse")) {
          return new Response(JSON.stringify(browseFixture("unreachable")), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/seed-count")) {
          return new Response(
            JSON.stringify({
              peer_count: 1,
              self_seeding: false,
              self_pin_enabled: null,
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response("{}", { status: 404 });
      }),
    );
    renderCatalog();

    // Composé front-side depuis la paire honnête (PLAN-ADAPT preflight F) :
    // statut d'ancre unreachable + peer_count>0 version-exacte. Les DEUX
    // cartes à entrée d'annuaire unreachable (vérifiée + orpheline) portent
    // le badge ; la dérivée (sans entrée d'annuaire) n'interroge pas.
    // (waitFor : les deux requêtes seed-count résolvent indépendamment —
    // findAll* retournerait dès le PREMIER badge rendu.)
    await waitFor(() => {
      expect(screen.getAllByTestId("seeder-reach-badge")).toHaveLength(2);
    });
    const badges = screen.getAllByTestId("seeder-reach-badge");
    expect(badges[0]).toHaveTextContent("Joignable via un pair");
    // Version-exactitude de la requête (review F) : le hash de la row est
    // dans la query string.
    const seedCalls = urls.filter((u) => u.includes("/api/daemon/seed-count"));
    expect(
      seedCalls.some((u) => u.includes(`archive_hash=${HASH_VERIFIED}`)),
    ).toBe(true);
    expect(
      seedCalls.some((u) => u.includes(`archive_hash=${HASH_ORPHAN}`)),
    ).toBe(true);
  });

  it("badge Q7 absent quand aucun seeder recent (honnetete best-effort)", async () => {
    const urls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        urls.push(url);
        const path = new URL(url).pathname;
        if (path.includes("/api/daemon/nodes")) {
          return new Response(JSON.stringify({ nodes: [makeNode()] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/browse")) {
          return new Response(JSON.stringify(browseFixture("unreachable")), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/seed-count")) {
          return new Response(
            JSON.stringify({
              peer_count: 0,
              self_seeding: false,
              self_pin_enabled: null,
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response("{}", { status: 404 });
      }),
    );
    renderCatalog();
    await screen.findAllByTestId("catalog-card");
    // Gate de probance (review F) : attendre que la réponse seed-count ait
    // été ÉMISE avant d'asserter l'absence — sinon l'assertion passe à la
    // frame 0, avant toute réponse, et ne prouve rien.
    await waitFor(() => {
      expect(
        urls.some((u) => u.includes("/api/daemon/seed-count")),
      ).toBe(true);
    });
    await waitFor(() => {
      expect(screen.queryByTestId("seeder-reach-badge")).not.toBeInTheDocument();
    });
  });

  it("le CTA garder-en-ligne epingle la version exacte affichee", async () => {
    const user = userEvent.setup();
    const calls: Array<{ url: string; body: unknown }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init?: RequestInit) => {
        calls.push({
          url,
          body: init?.body ? JSON.parse(init.body as string) : null,
        });
        const path = new URL(url).pathname;
        if (path.includes("/api/daemon/nodes")) {
          return new Response(JSON.stringify({ nodes: [makeNode()] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/browse")) {
          return new Response(JSON.stringify(browseFixture("reachable")), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.endsWith("/api/daemon/seed")) {
          return new Response(
            JSON.stringify({ ok: true, seeding: PID_VERIFIED }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response("{}", { status: 404 });
      }),
    );
    renderCatalog();

    const cards = await screen.findAllByTestId("catalog-card");
    const verifiedCard = cards.find((c) => c.textContent?.includes("Babel") && !c.textContent?.includes("remix"))!;
    await user.click(
      verifiedCard.querySelector<HTMLButtonElement>(
        '[data-testid="catalog-support"]',
      )!,
    );

    await waitFor(() => {
      expect(calls.some((c) => c.url.endsWith("/api/daemon/seed"))).toBe(true);
    });
    const seedCall = calls.find((c) => c.url.endsWith("/api/daemon/seed"))!;
    // Le discriminateur de version (déféré review-D fermé en F).
    expect(seedCall.body).toEqual({
      project_id: PID_VERIFIED,
      archive_hash: HASH_VERIFIED,
    });
    expect(
      await screen.findByTestId("catalog-support-active"),
    ).toBeInTheDocument();
  });

  it("le CTA garder-en-ligne survit a un refresh (reconciliation self_seeding)", async () => {
    // Bug live-found (classe WEB-1, intent vs truth) : l'état « Gardée en
    // ligne » était un useState in-session — un refresh re-rendait le bouton
    // neutre alors que le daemon seede réellement (row M18 + pin posés).
    // Au mount (= après refresh), seed-count.self_seeding version-exacte est
    // la vérité : true ⇒ l'état actif se rend SANS aucun clic.
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        const path = new URL(url).pathname;
        if (path.includes("/api/daemon/nodes")) {
          return new Response(JSON.stringify({ nodes: [makeNode()] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/browse")) {
          return new Response(JSON.stringify(browseFixture("reachable")), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (path.includes("/api/daemon/seed-count")) {
          // Ce nœud seede déjà la version exacte demandée.
          return new Response(
            JSON.stringify({
              peer_count: 0,
              self_seeding: true,
              self_pin_enabled: true,
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response("{}", { status: 404 });
      }),
    );
    renderCatalog();

    // L'état actif apparaît sans interaction (réconcilié depuis le daemon).
    const actives = await screen.findAllByTestId("catalog-support-active");
    expect(actives.length).toBeGreaterThan(0);
  });

  it("catalogue inconnu : etat introuvable avec retour vers /nodes", async () => {
    mockFetch({
      "/api/daemon/nodes": { nodes: [] },
      "/api/daemon/browse": { entries: [] },
    });
    renderCatalog("00".repeat(32));
    expect(await screen.findByTestId("node-not-found")).toBeInTheDocument();
    expect(screen.getByTestId("back-to-nodes")).toBeInTheDocument();
  });
});
