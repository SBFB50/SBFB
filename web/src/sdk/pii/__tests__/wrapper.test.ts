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
});
