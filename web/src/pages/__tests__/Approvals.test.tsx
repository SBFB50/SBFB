// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase L1 — `/approvals` : la file d'approbation HITL host-chrome.
 *
 * Couvre :
 *  (a) la file est VIDE en production — la fixture est injectée par les tests
 *      (correctif review P1-7 : une approbation de démonstration servie en prod
 *      est une donnée d'autorité fabriquée) ;
 *  (b) le tuple EXACT est rendu (WYSIWYS) — les 14 champs de la demande,
 *      valeurs comprises, et AUCUN verdict sur une demande pendante ;
 *  (c) décider retire UNIQUEMENT la carte décidée de la file host-owned et
 *      annonce un fait VÉRIFIABLE (pas « transmise pour signature » : rien
 *      n'est signé en L1) ;
 *  (d) AUCUN champ de saisie et AUCUN bouton désactivé (ni type-to-confirm ni
 *      délai imposé) ;
 *  (e) focus géré au montage ET après décision, sur le porteur du message ;
 *  (f) la file est HOST-OWNED : une décision survit au démontage de la page ;
 *  (g) une expiration sentinelle ne peut pas casser le rendu.
 */

import { afterEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";

import Approvals from "../Approvals";
import {
  __seedHostQueueForTest,
  sanitiseApprovalQueue,
  type HitlRequest,
  type PendingApproval,
} from "../approvalsQueue";

/**
 * Les 14 champs de la DEMANDE, en miroir littéral, DANS L'ORDRE DU TUPLE RUST
 * (`crates/sbfb-workflow-engine/src/hitl.rs` : … `nonce`, `seq`,
 * `expires_at_ms`). Le 15e champ Rust — `decision` — n'a PAS de place ici :
 * une demande pendante ne porte pas de verdict (review r2 R7).
 */
const EXPECTED_FIELDS: Array<keyof HitlRequest> = [
  "runId",
  "planDigest",
  "nodeInstanceId",
  "effectId",
  "capabilityRef",
  "operation",
  "destination",
  "outgoingDigest",
  "effectClass",
  "maxCost",
  "principal",
  "nonce",
  "seq",
  "expiresAtMs",
];

const FIXTURE: PendingApproval = {
  requestId: "hitl-request-fixture-0001",
  request: {
    runId: "run-2f9c",
    planDigest: "plan-a1b2c3",
    nodeInstanceId: "root/cloud-model#1",
    effectId: "eff-7d21",
    capabilityRef: "cloud_provider_token",
    operation: "model.cloud.invoke",
    destination: "https://api.cloud-provider.example/v1/messages",
    outgoingDigest: "out-9e44f0",
    effectClass: "ModelCloud",
    maxCost: "5000",
    principal: "operator@node.local",
    nonce: "wait-nonce-5f3a",
    seq: "1",
    expiresAtMs: "4102444800000",
  },
};

/**
 * Une SECONDE demande en attente. Indispensable : avec une file à une seule
 * entrée, `removeFromHostQueue` est indistinguable d'un vidage complet
 * (review r2 P2-20), et un run parallèle produit normalement plusieurs
 * demandes simultanées.
 */
const FIXTURE_2: PendingApproval = {
  requestId: "hitl-request-fixture-0002",
  request: {
    ...FIXTURE.request,
    runId: "run-8b41",
    effectId: "eff-0c55",
    operation: "net.http.post",
    destination: "https://hooks.other-provider.example/notify",
    seq: "2",
  },
};

afterEach(() => {
  __seedHostQueueForTest();
});

function renderPage() {
  return render(
    <MemoryRouter>
      <Approvals />
    </MemoryRouter>,
  );
}

describe("Approvals — HITL host chrome", () => {
  it("is EMPTY by default: no fabricated authority data in production", () => {
    renderPage();
    expect(screen.getByTestId("hitl-empty")).toBeInTheDocument();
    expect(screen.queryByTestId("hitl-approval-card")).not.toBeInTheDocument();
  });

  it("renders the exact request tuple (WYSIWYS): all 14 fields with values, NO verdict", () => {
    __seedHostQueueForTest([FIXTURE]);
    renderPage();
    const card = screen.getByTestId("hitl-approval-card");

    // Every field is present AND carries its value — a label alone would let a
    // silently-empty value pass.
    // Les 14 champs sans exception, y compris `expiresAtMs` : la PRÉIMAGE BRUTE
    // est asserée pour tous. Le ternaire précédent y substituait l'ISO, donc la
    // propriété-titre du rendu d'expiration (« la préimage est TOUJOURS
    // affichée ») n'était vérifiée sur AUCUN test de la branche ISO — la seule
    // où elle peut se perdre : retirer `{raw} (` … `)` compilait et laissait la
    // suite verte (mutation mesurée, review r5 L1-R5-D4-03).
    for (const field of EXPECTED_FIELDS) {
      const cell = card.querySelector(`[data-field="${field}"]`);
      expect(cell, `field ${field} must be rendered`).not.toBeNull();
      expect(cell?.textContent, `field ${field} must show its value`).toContain(
        String(FIXTURE.request[field]),
      );
    }
    // L'ISO est un confort AJOUTÉ à côté de la préimage, jamais un remplacement.
    const expiryCell = card.querySelector('[data-field="expiresAtMs"]');
    expect(expiryCell?.textContent).toContain(
      new Date(Number(FIXTURE.request.expiresAtMs)).toISOString(),
    );
    expect(
      expiryCell?.querySelector("[data-expiry-format='iso']"),
    ).not.toBeNull();
    // Exactly 14 request fields are rendered. PORTÉE HONNÊTE de ce contrôle
    // (review r2 P2-17) : il attrape un champ ajouté au miroir TS sans entrée
    // d'affichage, et un champ retiré. Il n'attrape PAS une édition RUST — rien
    // de Rust n'est atteignable depuis ce fichier ; c'est la jambe
    // `HITL-PARITY` de `scripts/check-frontier-contracts.sh` qui tient la parité
    // cross-langage.
    expect(card.querySelectorAll("[data-field]")).toHaveLength(14);
    expect(EXPECTED_FIELDS).toHaveLength(14);
    // And NO verdict is shown on a pending request (review r2 R7).
    expect(card).not.toHaveTextContent("Verdict");
    expect(card).not.toHaveTextContent("granted");
  });

  it("focuses the queue heading on mount (managed focus)", () => {
    renderPage();
    expect(screen.getByTestId("hitl-heading")).toHaveFocus();
  });

  it("has no text input and no disabled control (no type-to-confirm, no forced delay)", () => {
    __seedHostQueueForTest([FIXTURE]);
    renderPage();
    const card = screen.getByTestId("hitl-approval-card");
    expect(card.querySelectorAll("input, textarea")).toHaveLength(0);
    // Enabled at FIRST render: a control disabled behind a timer would be the
    // "forced delay" dark-pattern.
    expect(screen.getByTestId("hitl-approve")).not.toBeDisabled();
    expect(screen.getByTestId("hitl-reject")).not.toBeDisabled();
  });

  it("approving removes ONLY the decided card and announces a VERIFIABLE fact", async () => {
    const user = userEvent.setup();
    __seedHostQueueForTest([FIXTURE, FIXTURE_2]);
    renderPage();
    expect(screen.getAllByTestId("hitl-approval-card")).toHaveLength(2);

    await user.click(screen.getAllByTestId("hitl-approve")[0]);

    // The OTHER pending approval must survive: a decision is not a queue flush
    // (review r2 P2-20 — two effects would otherwise be stranded on
    // `waiting_human` with no visible request and no way to find them).
    const remaining = screen.getAllByTestId("hitl-approval-card");
    expect(remaining).toHaveLength(1);
    expect(remaining[0]).toHaveAttribute(
      "data-request-id",
      FIXTURE_2.requestId,
    );

    const status = screen.getByTestId("hitl-decision-status");
    // The claim must be what actually happened — NOT "transmitted for host
    // signature", since nothing leaves the browser in this phase.
    expect(status).toHaveTextContent("pris dans cet écran");
    expect(status).toHaveTextContent("Rien n'est transmis ni conservé");
    expect(status.textContent).not.toContain("transmise");
    // Focus lands on the element that CARRIES the message, not on the heading:
    // moving focus in the same tick as a `polite` update makes screen readers
    // drop the pending announcement (review r2, a11y residue).
    expect(status).toHaveFocus();
  });

  it("rejecting removes the card and states the SAME signing semantics as the engine", async () => {
    const user = userEvent.setup();
    __seedHostQueueForTest([FIXTURE]);
    renderPage();
    await user.click(screen.getByTestId("hitl-reject"));
    expect(screen.queryByTestId("hitl-approval-card")).not.toBeInTheDocument();
    const status = screen.getByTestId("hitl-decision-status");
    // ONE refusal semantics across the seam (review r2 R7): the engine models
    // and tests a SIGNED refusal (`a_refusal_never_decides_a_ref`), because
    // `decision` is inside the signed tuple. The screen must not claim the
    // opposite — an unsigned refusal would be indistinguishable from silence.
    expect(status).toHaveTextContent("Un refus se signe comme un accord");
    expect(status).toHaveTextContent("deux verdicts, deux digests");
    expect(status.textContent).not.toContain(
      "aucune signature n'est émise pour un refus",
    );
    // And it claims nothing beyond what happened (review r3 L1-R3-07): no
    // "recorded", no "auditable" — nothing is persisted anywhere in L1.
    expect(status).toHaveTextContent("Rien n'est transmis ni conservé");
    expect(status.textContent).not.toContain("auditable");
  });

  it("is HOST-OWNED: a decision survives unmounting the page", async () => {
    const user = userEvent.setup();
    __seedHostQueueForTest([FIXTURE]);
    const first = renderPage();
    await user.click(screen.getByTestId("hitl-approve"));

    // Remount from scratch — as a SPA route change does. The queue lives in the
    // host module, not in component state (review r2 P2-21): if it were held in
    // `useState`, the approval just decided would come back to life here.
    first.unmount();
    renderPage();
    expect(screen.getByTestId("hitl-empty")).toBeInTheDocument();
    expect(screen.queryByTestId("hitl-approval-card")).not.toBeInTheDocument();
  });

  it("renders an out-of-range expiry as RAW bytes instead of throwing", () => {
    // `i64::MAX` is simply the largest value the Rust field can hold — NOT a
    // documented "no expiry" sentinel (no such constant exists in the engine; the
    // mirror must not invent semantics for the other half of a frozen preimage).
    //
    // WHICH clause stops it, measured (review r5 L1-R5-D4-02): the FIDELITY one.
    // `Number("9223372036854775807")` is 9223372036854776000, so
    // `String(ms) !== trimmed` and the value never reaches `new Date(…)`. It is
    // NOT the date-range bound — that clause has its own bar in the `it.each`
    // above. Before either existed, `toISOString()` threw during render, took out
    // the whole route via the error boundary, and stranded every healthy approval
    // with it (review r2 P2-18).
    const sentinel: PendingApproval = {
      requestId: "hitl-request-sentinel",
      request: { ...FIXTURE.request, expiresAtMs: "9223372036854775807" },
    };
    __seedHostQueueForTest([sentinel]);
    renderPage();
    const cell = screen
      .getByTestId("hitl-approval-card")
      .querySelector('[data-field="expiresAtMs"]');
    // The raw value IS the signed preimage — the only honest thing to show when
    // formatting is impossible.
    expect(cell?.textContent).toBe("9223372036854775807");
    expect(cell?.querySelector("[data-expiry-format='raw']")).not.toBeNull();
  });

  /**
   * The guard must not FABRICATE a plausible date out of a non-string, and must
   * not throw on one either (ex-review r4 L1-R4-F01/F02).
   *
   * `false` used to render `1970-01-01T00:00:00.000Z` — a perfectly plausible
   * expiry, and false — while an omitted field rendered an EMPTY cell, both on
   * the screen whose only purpose is that the human reads the exact preimage.
   * `Option<i64> → null` is the natural slope of the Phase V mapper, so these
   * are wiring-reachable, not just console-reachable.
   */
  it.each([
    ["false", false, "false"],
    ["absent", undefined, "(absente)"],
    ["null", null, "(absente)"],
    [
      "an object whose toString would be invoked",
      { toString: (): string => "1700000000000" },
      "(valeur object hors contrat)",
    ],
    ["a non-faithful string", "1e3", "1e3"],
    // FAITHFUL but out of the representable Date range — the only input that
    // reaches the `MAX_TIME_VALUE_MS` clause. Without this bar that clause is
    // dead: removing it left all 7 pre-existing fixtures identical (mutation
    // measured, review r5 L1-R5-D4-02), while `new Date(…).toISOString()` would
    // throw DURING render and take the whole /approvals route down through the
    // error boundary — healthy approvals included.
    ["faithful but out of Date range", "8640000000001000", "8640000000001000"],
  ])("renders %s expiry verbatim, never as a fabricated date", (_label, value, expected) => {
    __seedHostQueueForTest([
      {
        requestId: "hitl-request-malformed-expiry",
        // The queue type says `string`; the point is what happens when the
        // wiring hands over something else.
        request: { ...FIXTURE.request, expiresAtMs: value as string },
      },
    ]);
    renderPage();
    const cell = screen
      .getByTestId("hitl-approval-card")
      .querySelector('[data-field="expiresAtMs"]');
    expect(cell?.textContent).toBe(expected);
    expect(cell?.querySelector("[data-expiry-format='raw']")).not.toBeNull();
  });

  /**
   * A malformed ITEM must be dropped, never the whole queue (ex-review r4
   * L1-R4-F03). Before the per-item type guard, `[{}]` threw while destructuring
   * the tuple, `RouteErrorBoundary` caught it, and /approvals died — carrying
   * every HEALTHY approval with it.
   *
   * Tested against `sanitiseApprovalQueue` DIRECTLY, not through
   * `__seedHostQueueForTest`: that helper writes the internal state without
   * going through the filter, so seeding through it would prove nothing here.
   */
  it("drops a malformed seeded entry and keeps the healthy one", () => {
    const kept = sanitiseApprovalQueue([
      {},
      null,
      "not an object",
      { requestId: "", request: FIXTURE.request },
      { requestId: "no-request" },
      { requestId: "missing-a-field", request: { ...FIXTURE.request, nonce: undefined } },
      { requestId: "wrong-type", request: { ...FIXTURE.request, seq: 2 } },
      FIXTURE,
    ]);
    expect(kept).toEqual([FIXTURE]);
  });

  it("rejects a non-array seed without throwing", () => {
    expect(sanitiseApprovalQueue(undefined)).toEqual([]);
    // DOM clobbering: `window.__SBFB_HITL_QUEUE` resolving to an element.
    expect(sanitiseApprovalQueue(document.createElement("div"))).toEqual([]);
  });
});
