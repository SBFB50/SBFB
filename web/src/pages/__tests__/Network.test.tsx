// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/my-network` — "offer my power" panel (Sprint 76 Phase A, D1).
 *
 * Covers the live caps gauge fed by the snapshot's `consent` field and
 * the intention-first CTA (no `consent/set` / `kind` / `provider`
 * jargon in the call to action).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import Network from "../Network";
import { primeAuthToken } from "@/api/auth";
import { useProjectStore } from "@/stores/projectStore";

const COORD_URL = "http://127.0.0.1:8765";

const consentConfig = {
  level: 4,
  caps: { max_watts: 400, max_vram_mb: 16384, max_hours_day: 12 },
  allowed_project_ids: [],
  own_node_id: "self",
  level_threat_note: "",
  residual_threats_acknowledged: [],
};

function workerStateRunning(consent: unknown) {
  return {
    running: true,
    stale: false,
    state: {
      schema_version: 1,
      node_id: "n".repeat(64),
      worker_version: "0.1.0",
      uptime_secs: 100,
      started_at: "2026-06-15T00:00:00Z",
      last_updated_at: "2026-06-15T00:01:00Z",
      gpu: null,
      projects_served: [],
      last_task: null,
      consent,
    },
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

function renderNetwork() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <Network />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  primeAuthToken("test-token");
  // Suppress the first-visit auto-open so the dialog doesn't steal focus.
  window.localStorage.setItem("sbfb-consent-seen-v1", "1");
  useProjectStore.setState({
    knownCoordinators: [{ url: COORD_URL, nickname: "test", nodeId: null }],
    activeCoordinatorUrl: COORD_URL,
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  primeAuthToken(null);
  window.localStorage.clear();
  useProjectStore.setState({
    knownCoordinators: [],
    activeCoordinatorUrl: null,
  });
});

describe("Network — offer my power panel (Sprint 76 Phase A)", () => {
  it("rend la jauge de caps + le niveau actif depuis le champ consent du snapshot", async () => {
    mockFetch({
      "/api/v1/worker/state": workerStateRunning({
        level: 4,
        max_hours_day: 12,
        hours_used_today: 3.5,
        max_watts: 400,
        max_vram_mb: 16384,
      }),
      "/api/v1/consent": consentConfig,
    });
    renderNetwork();

    const gauge = await screen.findByTestId("offer-power-hours-gauge");
    expect(gauge).toHaveTextContent("3.5 h / 12 h");
    expect(screen.getByTestId("offer-power-level")).toHaveTextContent(
      "Tous les projets publics",
    );
  });

  it("le CTA exprime une intention, jamais du jargon consent/set/kind/provider", async () => {
    mockFetch({
      "/api/v1/worker/state": workerStateRunning({
        level: 2,
        max_hours_day: 8,
        hours_used_today: 1,
        max_watts: 300,
        max_vram_mb: 8192,
      }),
      "/api/v1/consent": { ...consentConfig, level: 2 },
    });
    renderNetwork();

    const cta = await screen.findByTestId("offer-power-cta");
    expect(cta).toHaveTextContent("Offrir ma puissance au réseau");
    expect(cta.textContent ?? "").not.toMatch(/consent\/set|kind|provider/i);
  });
});

describe("Network — contributor dashboard (Sprint 76 Phase E, D4)", () => {
  it("rend les 3 métriques honnêtes (kudos effectifs, tâches servies, GPU-heures locales non attestées)", async () => {
    mockFetch({
      "/api/v1/worker/state": workerStateRunning({
        level: 4,
        max_hours_day: 12,
        hours_used_today: 2.5,
        max_watts: 400,
        max_vram_mb: 16384,
      }),
      "/api/v1/consent": consentConfig,
      "/api/v1/contributor": {
        worker_node_id: "n".repeat(64),
        effective_kudos: 4200,
        tasks_served: 7,
        per_project: [
          { project_id: "proj-1", effective_kudos: 4200, tasks_served: 7 },
        ],
      },
    });
    renderNetwork();

    await screen.findByTestId("contributor-card");
    // The contributor metrics resolve on a second async hop (the query is
    // keyed on the node_id from the worker snapshot), so await the value.
    await screen.findByText("4200");
    expect(screen.getByTestId("contributor-kudos")).toHaveTextContent("4200");
    expect(screen.getByTestId("contributor-tasks")).toHaveTextContent("7");
    // GPU-hours come from the LOCAL usage snapshot, not the ledger, and the
    // label must say so honestly (this machine, non-attested).
    const gpuHours = screen.getByTestId("contributor-gpu-hours");
    expect(gpuHours).toHaveTextContent("2.5 h");
    expect(gpuHours).toHaveTextContent(/cette machine/i);
    expect(gpuHours).toHaveTextContent(/non attestées/i);
  });
});
