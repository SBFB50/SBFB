// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 75 Phase F — AddAnchorDialog (« ajouter une ancre »).
 *
 * Couvre : la validation locale de l'identité, la soumission vers la route
 * subscribe EXISTANTE (l'ancre est une subscription, kickoff Q3/DQ3), le
 * verrou 3 (placeholder inerte, aucun auto-subscribe au mount) et le lock-1
 * sur la nouvelle surface (aucun champ hôte/cible : l'ajout d'ancre est un
 * acte read-side, pas un sélecteur de destination de publication).
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

import { AddAnchorDialog } from "../AddAnchorDialog";

const COORD_URL = "http://127.0.0.1:8765";

function renderDialog(onOpenChange: (open: boolean) => void = () => {}) {
  const qc = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={qc}>
      <AddAnchorDialog open onOpenChange={onOpenChange} coordUrl={COORD_URL} />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response("{}", { status: 503 })),
  );
});
afterEach(() => {
  vi.unstubAllGlobals();
});

describe("AddAnchorDialog", () => {
  it("verrou 3 : placeholder inerte, aucun appel reseau au mount", async () => {
    const fetchSpy = vi.fn(async () => new Response("{}", { status: 503 }));
    vi.stubGlobal("fetch", fetchSpy);
    renderDialog();

    const input = await screen.findByTestId("anchor-pubkey-input");
    // Le champ démarre VIDE — jamais une clé pré-remplie qui s'abonnerait
    // d'elle-même (une ancre par défaut compilée serait le tripwire lock-3).
    expect(input).toHaveValue("");
    expect(input).toHaveAttribute("placeholder", "abcd1234...");
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("lock-1 : aucune notion de champ hote/cible ou de publication", async () => {
    renderDialog();
    await screen.findByTestId("add-anchor-dialog");
    // L'ajout d'ancre est read-side (s'abonner à une source de découverte),
    // jamais un « publier sur X ».
    expect(screen.queryByText(/h[oô]te/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/publier sur/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Mon serveur/i)).not.toBeInTheDocument();
    // Et le texte rappelle que l'ancre n'est pas une autorité.
    expect(screen.getByText(/jamais une autorit/)).toBeInTheDocument();
  });

  it("rejette localement une identite invalide sans toucher le reseau", async () => {
    const user = userEvent.setup();
    const fetchSpy = vi.fn(async () => new Response("{}", { status: 503 }));
    vi.stubGlobal("fetch", fetchSpy);
    renderDialog();

    await user.type(
      await screen.findByTestId("anchor-pubkey-input"),
      "pas-une-cle",
    );
    await user.click(screen.getByTestId("anchor-subscribe-submit"));

    expect(await screen.findByTestId("anchor-form-error")).toHaveTextContent(
      "64 caractères hexadécimaux",
    );
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("s'abonne via la route curators/subscribe existante et ferme le dialog", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    const calls: Array<{ url: string; body: unknown }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init?: RequestInit) => {
        calls.push({
          url,
          body: init?.body ? JSON.parse(init.body as string) : null,
        });
        return new Response(
          JSON.stringify({ subscribed_curators: ["ee".repeat(32)] }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      }),
    );
    renderDialog(onOpenChange);

    // La casse est normalisée localement (la route exige du minuscule).
    await user.type(
      await screen.findByTestId("anchor-pubkey-input"),
      "EE".repeat(32),
    );
    await user.click(screen.getByTestId("anchor-subscribe-submit"));

    await waitFor(() => {
      expect(
        calls.some((c) => c.url.includes("/api/daemon/curators/subscribe")),
      ).toBe(true);
    });
    const call = calls.find((c) =>
      c.url.includes("/api/daemon/curators/subscribe"),
    )!;
    expect(call.body).toEqual({ curator_pubkey_hex: "ee".repeat(32) });
    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it("affiche la raison du daemon quand la subscription echoue", async () => {
    const user = userEvent.setup();
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(JSON.stringify({ error: "bad key" }), {
          status: 400,
          headers: { "content-type": "application/json" },
        }),
      ),
    );
    renderDialog();

    await user.type(
      await screen.findByTestId("anchor-pubkey-input"),
      "ee".repeat(32),
    );
    await user.click(screen.getByTestId("anchor-subscribe-submit"));

    const err = await screen.findByTestId("anchor-form-error");
    expect(err).toHaveTextContent(/HTTP 400/);
    // GAP Codex R2 : la raison {"error": ...} du daemon est remontée — le
    // statusText générique seul cacherait la cause actionnable.
    expect(err).toHaveTextContent(/bad key/);
  });
});
