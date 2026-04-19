// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — GlinerPiiDetector wrapper tests.
 *
 * We exercise the detector against an injected `ModelLoader` so the
 * tests stay hermetic (jsdom has no WASM runtime to host
 * onnxruntime-web, and downloading 45 MB ONNX assets in CI is a
 * non-starter). The real model path is exercised manually in dev
 * once the operator drops the ONNX file into `web/public/models/`.
 */

import { describe, expect, it } from "vitest";

import { DEFAULT_POLICY, type PiiFinding, type PiiPolicy } from "../policy";
import {
  GlinerPiiDetector,
  type ModelHandle,
  type ModelLoader,
} from "../wrapper";

class StubLoaderAlwaysFails implements ModelLoader {
  async load(): Promise<ModelHandle> {
    throw new Error("model asset 404 — download gliner-pii-edge-v1.0.onnx");
  }
}

class StubLoaderReturns implements ModelLoader {
  private readonly findings: PiiFinding[];

  constructor(findings: PiiFinding[]) {
    this.findings = findings;
  }

  async load(): Promise<ModelHandle> {
    const snapshot = this.findings;
    return {
      detect: async () => [...snapshot],
    };
  }
}

describe("GlinerPiiDetector", () => {
  it("falls back to regex when the model load fails", async () => {
    const detector = new GlinerPiiDetector(new StubLoaderAlwaysFails());
    const findings = await detector.detect(
      "Email foo@bar.com for help",
      DEFAULT_POLICY,
    );
    // Regex fallback catches the email; the failed load does not
    // bubble as an exception — apps are decoupled from asset state.
    expect(findings.some((f) => f.entity === "EMAIL_ADDRESS")).toBe(true);
  });

  it("respects the entity whitelist from the policy", async () => {
    // Simulate a model that returns a PERSON finding. The policy
    // whitelist drops anything not asked for.
    const loader = new StubLoaderReturns([
      { entity: "PERSON", start: 0, end: 4, confidence: 0.9 },
    ]);
    const detector = new GlinerPiiDetector(loader);
    const policy: PiiPolicy = {
      ...DEFAULT_POLICY,
      entities: ["EMAIL_ADDRESS"],
      use_model: true,
    };
    const findings = await detector.detect("Alice loves bread", policy);
    expect(findings.some((f) => f.entity === "PERSON")).toBe(false);
  });

  it("detects at least two entities on the Phase B fixture text", async () => {
    // Sprint 22 Phase B regression fixture (plan §5.3 test 6) :
    // the real ONNX path cannot run in jsdom, so this exercises the
    // regex fallback combined with the threshold filter. We expect
    // the detector to surface ≥ 2 findings on a prompt that mixes an
    // email with an international phone — catching the case where a
    // future refactor silently regresses the fallback wiring when
    // the model path returns zero spans.
    const detector = new GlinerPiiDetector(new StubLoaderAlwaysFails());
    const findings = await detector.detect(
      "Contact: alice@example.com and +33123456789",
      DEFAULT_POLICY,
    );
    expect(findings.length).toBeGreaterThanOrEqual(2);
    const entities = new Set(findings.map((f) => f.entity));
    expect(entities.has("EMAIL_ADDRESS")).toBe(true);
    expect(entities.has("PHONE_NUMBER")).toBe(true);
  });
});
