// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Tests for `<GpuConsentDialog>` (Sprint 16 Phase C).
 *
 * Pairs with the worker-side enforcement tests in
 * `crates/nexus-worker-core/src/consent.rs`. The dialog produces
 * the JSON the worker reads back via the `notify` watcher — if
 * the wire shape drifts, those Rust unit tests catch it on the
 * worker side and these vitest cases catch it on the shell side.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { GpuConsentDialog } from "../GpuConsentDialog";
import { primeAuthToken } from "@/api/auth";
import { type ConsentConfig, DEFAULT_CONSENT } from "@/api/consent";

const NODE_ID_A =
  "a".repeat(64);

const NOOP = () => {};

const baseConfig: ConsentConfig = {
  level: 1,
  caps: {
    max_watts: 400,
    max_vram_mb: 16 * 1024,
    max_hours_day: 12.0,
  },
  allowed_project_ids: [],
  own_node_id: "self",
};

beforeEach(() => {
  primeAuthToken("test-token");
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  primeAuthToken(null);
});

function renderDialog(overrides: Partial<React.ComponentProps<typeof GpuConsentDialog>> = {}) {
  return render(
    <GpuConsentDialog
      open
      onOpenChange={NOOP}
      coordinatorUrl="http://127.0.0.1:7777"
      initialConfig={baseConfig}
      {...overrides}
    />,
  );
}

describe("<GpuConsentDialog>", () => {
  it("affiche les 4 niveaux avec L1 par défaut", () => {
    renderDialog();
    expect(screen.getByTestId("consent-level-1")).toBeInTheDocument();
    expect(screen.getByTestId("consent-level-2")).toBeInTheDocument();
    expect(screen.getByTestId("consent-level-3")).toBeInTheDocument();
    expect(screen.getByTestId("consent-level-4")).toBeInTheDocument();
    // L1 is the default selection (GDPR-safe).
    expect(screen.getByTestId("consent-level-1")).toHaveAttribute(
      "data-checked",
      "",
    );
  });

  it("masque la section whitelist L3 quand L1 est sélectionné", () => {
    renderDialog();
    expect(
      screen.queryByTestId("consent-whitelist-section"),
    ).not.toBeInTheDocument();
  });

  it("affiche la section whitelist quand L3 est sélectionné", async () => {
    renderDialog({ initialConfig: { ...baseConfig, level: 3 } });
    expect(screen.getByTestId("consent-whitelist-section")).toBeInTheDocument();
  });

  it("rejette un node_id mal formé dans la whitelist", async () => {
    const user = userEvent.setup();
    renderDialog({ initialConfig: { ...baseConfig, level: 3 } });

    await user.type(screen.getByTestId("consent-whitelist-input"), "not-hex");
    await user.click(screen.getByTestId("consent-whitelist-add"));

    expect(screen.getByTestId("consent-whitelist-error")).toHaveTextContent(
      /node_id hex/i,
    );
  });

  it("ajoute un node_id valide à la whitelist", async () => {
    const user = userEvent.setup();
    renderDialog({ initialConfig: { ...baseConfig, level: 3 } });

    await user.type(screen.getByTestId("consent-whitelist-input"), NODE_ID_A);
    await user.click(screen.getByTestId("consent-whitelist-add"));

    // The truncated form (16 first + 8 last hex chars) should
    // appear in the rendered list.
    expect(screen.getByText(/aaaaaaaa/i)).toBeInTheDocument();
  });

  it("retire un node_id depuis la liste", async () => {
    const user = userEvent.setup();
    renderDialog({
      initialConfig: { ...baseConfig, level: 3, allowed_project_ids: [NODE_ID_A] },
    });

    await user.click(screen.getByTestId(`consent-whitelist-remove-${NODE_ID_A}`));
    expect(
      screen.queryByTestId(`consent-whitelist-remove-${NODE_ID_A}`),
    ).not.toBeInTheDocument();
  });

  it("refuse les doublons dans la whitelist", async () => {
    const user = userEvent.setup();
    renderDialog({
      initialConfig: { ...baseConfig, level: 3, allowed_project_ids: [NODE_ID_A] },
    });

    await user.type(screen.getByTestId("consent-whitelist-input"), NODE_ID_A);
    await user.click(screen.getByTestId("consent-whitelist-add"));

    expect(screen.getByTestId("consent-whitelist-error")).toHaveTextContent(
      /déjà/i,
    );
  });

  it("affiche les caps W / VRAM / heures avec leurs valeurs initiales", () => {
    renderDialog();
    expect(screen.getByTestId("consent-cap-watts")).toHaveTextContent("400 W");
    expect(screen.getByTestId("consent-cap-vram")).toHaveTextContent("16 GB");
    expect(screen.getByTestId("consent-cap-hours")).toHaveTextContent("12 h");
  });

  it("POST /consent/set au save avec le payload courant", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    fetchMock.mockResolvedValue(
      new Response(
        JSON.stringify({
          ...baseConfig,
          level: 4,
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );
    const onSaved = vi.fn();
    const user = userEvent.setup();

    renderDialog({
      initialConfig: { ...baseConfig, level: 4 },
      onSaved,
    });

    await user.click(screen.getByTestId("consent-save"));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("http://127.0.0.1:7777/consent/set");
    expect(init?.method).toBe("POST");
    const body = JSON.parse(init?.body as string);
    expect(body.level).toBe(4);
    expect(body.caps.max_watts).toBe(400);

    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));
  });

  it("affiche une erreur quand /consent/set échoue", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    fetchMock.mockResolvedValue(
      new Response("boom", { status: 500, statusText: "Internal Server Error" }),
    );
    const user = userEvent.setup();

    renderDialog();
    await user.click(screen.getByTestId("consent-save"));

    expect(await screen.findByTestId("consent-save-error")).toHaveTextContent(
      /HTTP 500/,
    );
  });

  it("Annuler ferme le dialog sans POST", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    const onOpenChange = vi.fn();
    const user = userEvent.setup();

    renderDialog({ onOpenChange });
    await user.click(screen.getByRole("button", { name: /annuler/i }));

    expect(fetchMock).not.toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("re-sync l'état local quand initialConfig change pendant une réouverture", () => {
    const { rerender } = renderDialog({ initialConfig: baseConfig });
    expect(screen.getByTestId("consent-level-1")).toHaveAttribute(
      "data-checked",
      "",
    );

    rerender(
      <GpuConsentDialog
        open
        onOpenChange={NOOP}
        coordinatorUrl="http://127.0.0.1:7777"
        initialConfig={{ ...baseConfig, level: 4 }}
      />,
    );
    expect(screen.getByTestId("consent-level-4")).toHaveAttribute(
      "data-checked",
      "",
    );
  });

  it("expose DEFAULT_CONSENT compatible avec le coordinator par défaut", () => {
    // Sanity check on the wire-level default — the GET /consent/get
    // endpoint returns the same shape, so the dialog can render
    // before the first save.
    expect(DEFAULT_CONSENT.level).toBe(1);
    expect(DEFAULT_CONSENT.caps.max_watts).toBe(400);
  });
});
