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

import { DEFAULT_POLICY, type PiiFinding, type PiiEntity, type PiiPolicy } from "../policy";
import {
  PII_ENTITY_LABELS,
  type TokenOffset,
  decodeSpans,
  greedyDedup,
  toFinding,
} from "../decoder";
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

interface SpanInjection {
  tokenStart: number;
  tokenWidth: number;
  entity: PiiEntity;
  logit: number;
}

class DecoderExercisingLoader implements ModelLoader {
  private readonly injections: SpanInjection[];

  constructor(injections: SpanInjection[]) {
    this.injections = injections;
  }

  async load(): Promise<ModelHandle> {
    const injections = this.injections;
    return {
      detect: async (text: string, threshold?: number) => {
        const words = text.split(/\s+/);
        const offsets: TokenOffset[] = [];
        let pos = 0;
        for (const word of words) {
          const idx = text.indexOf(word, pos);
          offsets.push({ start: idx, end: idx + word.length });
          pos = idx + word.length;
        }

        const L = words.length;
        const K = Math.max(
          1,
          injections.reduce((mx, i) => Math.max(mx, i.tokenWidth), 0),
        );
        const C = PII_ENTITY_LABELS.length;

        const data = new Float32Array(L * K * C).fill(-20);
        for (const inj of injections) {
          const classIdx = PII_ENTITY_LABELS.indexOf(inj.entity);
          if (classIdx === -1) continue;
          const idx =
            inj.tokenStart * K * C + (inj.tokenWidth - 1) * C + classIdx;
          if (idx >= 0 && idx < data.length) data[idx] = inj.logit;
        }

        const spans = decodeSpans(
          data,
          [L, K, C],
          offsets,
          PII_ENTITY_LABELS,
          threshold,
        );
        return greedyDedup(spans).map((s) => toFinding(s, offsets));
      },
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

describe("GlinerPiiDetector — decoder pipeline (SC-10 ONNX CI fixture)", () => {
  it("exercises decoder pipeline end-to-end via mock ORT session", async () => {
    // "Contact alice@example.com now"
    // Tokens (space-split): ["Contact", "alice@example.com", "now"]
    // Inject EMAIL_ADDRESS at token 1, width 1, strong logit
    const loader = new DecoderExercisingLoader([
      { tokenStart: 1, tokenWidth: 1, entity: "EMAIL_ADDRESS", logit: 5.0 },
    ]);
    const detector = new GlinerPiiDetector(loader);
    const findings = await detector.detect(
      "Contact alice@example.com now",
      DEFAULT_POLICY,
    );
    expect(findings).toHaveLength(1);
    expect(findings[0].entity).toBe("EMAIL_ADDRESS");
    expect(findings[0].start).toBe(8);
    expect(findings[0].end).toBe(25);
    expect(findings[0].confidence).toBeGreaterThan(0.99);
  });

  it("exercises greedy dedup on overlapping mock spans", async () => {
    // "Call 555-1234 or email test@foo.com"
    // Tokens: ["Call", "555-1234", "or", "email", "test@foo.com"]
    // Inject PHONE at token 1 width 1 (strong), EMAIL overlapping
    // at token 1 width 2 (weaker), EMAIL at token 4 width 1
    const loader = new DecoderExercisingLoader([
      { tokenStart: 1, tokenWidth: 1, entity: "PHONE_NUMBER", logit: 6.0 },
      { tokenStart: 1, tokenWidth: 2, entity: "EMAIL_ADDRESS", logit: 3.0 },
      { tokenStart: 4, tokenWidth: 1, entity: "EMAIL_ADDRESS", logit: 5.0 },
    ]);
    const detector = new GlinerPiiDetector(loader);
    const findings = await detector.detect(
      "Call 555-1234 or email test@foo.com",
      DEFAULT_POLICY,
    );
    // PHONE wins the overlap; non-overlapping EMAIL kept
    expect(findings).toHaveLength(2);
    expect(findings[0].entity).toBe("PHONE_NUMBER");
    expect(findings[0].start).toBe(5);
    expect(findings[0].end).toBe(13);
    expect(findings[1].entity).toBe("EMAIL_ADDRESS");
    expect(findings[1].start).toBe(23);
    expect(findings[1].end).toBe(35);
  });

  it("respects threshold — weak logits produce no findings", async () => {
    // logit -1.0 → sigmoid ≈ 0.27, below default threshold 0.5
    const loader = new DecoderExercisingLoader([
      { tokenStart: 0, tokenWidth: 1, entity: "PERSON", logit: -1.0 },
    ]);
    const detector = new GlinerPiiDetector(loader);
    const findings = await detector.detect("Alice", {
      ...DEFAULT_POLICY,
      use_model: true,
    });
    // Model returns 0 findings → falls back to regex. "Alice" is not
    // a regex-detectable entity, so result is empty.
    expect(
      findings.every((f) => f.entity !== "PERSON"),
    ).toBe(true);
  });

  it("multi-width span detection across token boundary", async () => {
    // "Send to Alice Johnson today"
    // Tokens: ["Send", "to", "Alice", "Johnson", "today"]
    // PERSON spanning tokens 2-3 (width 2)
    const loader = new DecoderExercisingLoader([
      { tokenStart: 2, tokenWidth: 2, entity: "PERSON", logit: 4.5 },
    ]);
    const detector = new GlinerPiiDetector(loader);
    const findings = await detector.detect(
      "Send to Alice Johnson today",
      DEFAULT_POLICY,
    );
    expect(findings).toHaveLength(1);
    expect(findings[0].entity).toBe("PERSON");
    expect(findings[0].start).toBe(8);
    expect(findings[0].end).toBe(21);
  });
});
