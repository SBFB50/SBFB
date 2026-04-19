// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 22 Phase B — decoder unit tests.
 *
 * These fixtures are synthetic tensors — we do not spin up ORT in
 * jsdom. A real end-to-end exercise lives in the Playwright drift
 * investigation queued for Sprint 23 (cf.
 * `sprint22_plan.md §5.3` "Optionnel Playwright" note).
 */

import { describe, expect, it } from "vitest";

import {
  PII_ENTITY_LABELS,
  type TokenOffset,
  decodeSpans,
  greedyDedup,
  toFinding,
} from "../decoder";

/**
 * Build a `(L, K, C)` logit tensor and write a single strong logit
 * at `(start, width-1, classIdx)`. Everything else stays at the
 * very-negative floor so it sigmoids to ~0.
 */
function buildLogits(
  L: number,
  K: number,
  C: number,
  hits: ReadonlyArray<{
    start: number;
    width: number;
    classIdx: number;
    logit: number;
  }>,
): Float32Array {
  const out = new Float32Array(L * K * C);
  out.fill(-20);
  for (const hit of hits) {
    const idx = hit.start * K * C + (hit.width - 1) * C + hit.classIdx;
    out[idx] = hit.logit;
  }
  return out;
}

/** Uniform char-aligned offsets : token `i` spans `[i, i+1)`. */
function uniformOffsets(numTokens: number): TokenOffset[] {
  return Array.from({ length: numTokens }, (_, i) => ({
    start: i,
    end: i + 1,
  }));
}

describe("decodeSpans", () => {
  it("promotes a single strong span to a RawSpan with the right entity", () => {
    const emailClassIdx = PII_ENTITY_LABELS.indexOf("EMAIL_ADDRESS");
    const logits = buildLogits(4, 3, PII_ENTITY_LABELS.length, [
      { start: 1, width: 2, classIdx: emailClassIdx, logit: 4.0 },
    ]);
    const spans = decodeSpans(
      logits,
      [4, 3, PII_ENTITY_LABELS.length],
      uniformOffsets(4),
      PII_ENTITY_LABELS,
    );
    expect(spans).toHaveLength(1);
    expect(spans[0].entity).toBe("EMAIL_ADDRESS");
    expect(spans[0].startToken).toBe(1);
    expect(spans[0].endToken).toBe(3);
    expect(spans[0].score).toBeGreaterThan(0.98);
  });

  it("drops spans below the sigmoid threshold", () => {
    const phoneIdx = PII_ENTITY_LABELS.indexOf("PHONE_NUMBER");
    // Logit 0.2 → sigmoid ≈ 0.55. Threshold 0.6 must reject it.
    const logits = buildLogits(2, 2, PII_ENTITY_LABELS.length, [
      { start: 0, width: 1, classIdx: phoneIdx, logit: 0.2 },
    ]);
    const kept = decodeSpans(
      logits,
      [2, 2, PII_ENTITY_LABELS.length],
      uniformOffsets(2),
      PII_ENTITY_LABELS,
      0.6,
    );
    expect(kept).toHaveLength(0);
  });

  it("refuses spans whose end token runs past the sequence length", () => {
    const ssnIdx = PII_ENTITY_LABELS.indexOf("SSN");
    // numTokens = 3, but request a width-3 span starting at token 2.
    // That span would end at token 5, past the end — must be dropped.
    const L = 4;
    const K = 4;
    const C = PII_ENTITY_LABELS.length;
    const logits = buildLogits(L, K, C, [
      { start: 2, width: 3, classIdx: ssnIdx, logit: 5.0 },
    ]);
    const spans = decodeSpans(
      logits,
      [L, K, C],
      uniformOffsets(3),
      PII_ENTITY_LABELS,
    );
    expect(spans).toHaveLength(0);
  });

  it("returns [] on empty tokens or empty tensor", () => {
    expect(
      decodeSpans(
        new Float32Array(),
        [0, 0, 0],
        [],
        PII_ENTITY_LABELS,
      ),
    ).toEqual([]);
    const logits = buildLogits(2, 2, PII_ENTITY_LABELS.length, []);
    expect(
      decodeSpans(
        logits,
        [2, 2, PII_ENTITY_LABELS.length],
        [],
        PII_ENTITY_LABELS,
      ),
    ).toEqual([]);
  });
});

describe("greedyDedup", () => {
  it("keeps the highest-scoring span among overlapping candidates", () => {
    const highest = {
      startToken: 1,
      endToken: 4,
      classIdx: 0,
      entity: "EMAIL_ADDRESS" as const,
      score: 0.92,
    };
    const overlapping = [
      highest,
      {
        startToken: 2,
        endToken: 5,
        classIdx: 1,
        entity: "PHONE_NUMBER" as const,
        score: 0.7,
      },
      {
        startToken: 0,
        endToken: 2,
        classIdx: 2,
        entity: "CREDIT_CARD" as const,
        score: 0.65,
      },
    ];
    const kept = greedyDedup(overlapping);
    // The 0.92 email span wins; both other candidates overlap its
    // [1, 4) range so they are dropped.
    expect(kept).toHaveLength(1);
    expect(kept[0]).toBe(highest);
  });

  it("preserves every span when none overlap", () => {
    const spans = [
      {
        startToken: 0,
        endToken: 2,
        classIdx: 0,
        entity: "PERSON" as const,
        score: 0.8,
      },
      {
        startToken: 2,
        endToken: 4,
        classIdx: 1,
        entity: "EMAIL_ADDRESS" as const,
        score: 0.7,
      },
      {
        startToken: 4,
        endToken: 6,
        classIdx: 2,
        entity: "PHONE_NUMBER" as const,
        score: 0.9,
      },
    ];
    const kept = greedyDedup(spans);
    expect(kept).toHaveLength(3);
    // Output is sorted by startToken for stable downstream iteration.
    expect(kept.map((s) => s.startToken)).toEqual([0, 2, 4]);
  });
});

describe("toFinding", () => {
  it("translates token-index spans to character offsets", () => {
    // "alice@example.com" split into 3 tokens at fixed char offsets.
    const offsets: TokenOffset[] = [
      { start: 0, end: 5 },   // "alice"
      { start: 5, end: 6 },   // "@"
      { start: 6, end: 17 },  // "example.com"
    ];
    const span = {
      startToken: 0,
      endToken: 3,
      classIdx: PII_ENTITY_LABELS.indexOf("EMAIL_ADDRESS"),
      entity: "EMAIL_ADDRESS" as const,
      score: 0.88,
    };
    const finding = toFinding(span, offsets);
    expect(finding).toEqual({
      entity: "EMAIL_ADDRESS",
      start: 0,
      end: 17,
      confidence: 0.88,
    });
  });
});
