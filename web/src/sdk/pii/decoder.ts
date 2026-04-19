// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 22 Phase B — GLiNER span-logits decoder (pure module).
 *
 * Translates the raw span-level logits tensor produced by the
 * `knowledgator/gliner-pii-edge-v1.0` ONNX head into a list of
 * {@link PiiFinding} objects. The algorithm mirrors the canonical
 * upstream decoder (urchade/GLiNER, `decoding/decoder.py`) :
 *
 *   1. apply sigmoid to the (L, K, C) span logits tensor,
 *   2. threshold-filter (default 0.5) + drop spans that extend past
 *      the tokenised sequence,
 *   3. build {@link RawSpan} objects mapped to {@link PiiEntity},
 *   4. greedy deduplication keeps the highest-confidence non-
 *      overlapping spans (flat NER, single label per token range),
 *   5. translate token-index spans back to character offsets via the
 *      tokenizer's offset-mapping.
 *
 * The module is deliberately pure (no ORT / Transformers.js import)
 * so Vitest can exercise it with synthetic fixtures in jsdom — the
 * live ONNX path is covered manually once the operator drops the
 * 45 MB model asset into `web/public/models/`.
 *
 * ## G8 preflight finding S1-B-1 (2026-04-19)
 * Plan §5.2 pseudocode destructures 3 tensors
 * (`[start_logits, end_logits, span_logits]`). The canonical GLiNER
 * span head exports a **single** `(B, L, K, C)` tensor ; this module
 * follows the canonical upstream and wrapper integration adapts the
 * ONNX output accordingly. Cf.
 * `.planning/active/sprint22_phase_B_preflight.md §S1`.
 */

import type { PiiEntity, PiiFinding } from "./policy";

/**
 * Character offsets for a single token, returned by the tokenizer
 * when `return_offsets_mapping: true`. `[start, end)` is a half-open
 * interval indexing into the original text string.
 */
export interface TokenOffset {
  start: number;
  end: number;
}

/**
 * Intermediate representation produced by {@link decodeSpans} and
 * consumed by {@link greedyDedup}. Kept internal to the module.
 */
export interface RawSpan {
  /** Token-index of the first token in the span (inclusive). */
  startToken: number;
  /** Token-index one past the last token in the span (exclusive). */
  endToken: number;
  /** GLiNER class id — indexes into the class-labels vector. */
  classIdx: number;
  /** Resolved PII entity label. */
  entity: PiiEntity;
  /** Sigmoid-activated confidence in [0, 1]. */
  score: number;
}

/** GLiNER canonical decoding threshold (`decoder.py` default). */
export const DEFAULT_THRESHOLD = 0.5;

/**
 * Class-label order expected by the `knowledgator/gliner-pii-edge-
 * v1.0` ONNX export. The GLiNER head is label-agnostic : the ONNX
 * graph takes entity prompts as input and the output class axis is
 * aligned with the prompt order. This constant documents the order
 * the SDK feeds at inference time (mirrored in the wrapper).
 */
export const PII_ENTITY_LABELS: readonly PiiEntity[] = [
  "PERSON",
  "EMAIL_ADDRESS",
  "PHONE_NUMBER",
  "CREDIT_CARD",
  "SSN",
  "IBAN",
];

/**
 * Numerically-stable sigmoid. Avoids `exp(x)` overflow for strongly
 * positive logits and underflow for strongly negative ones — the
 * ONNX output can contain logits > 50 when the model is highly
 * confident.
 */
function sigmoid(x: number): number {
  return x >= 0
    ? 1 / (1 + Math.exp(-x))
    : Math.exp(x) / (1 + Math.exp(x));
}

/**
 * Walk the `(L, K, C)` span-logits tensor and return every
 * above-threshold span that also fits within the tokenised sequence.
 * The input is a flat `Float32Array` (or plain array, for tests)
 * indexed row-major : `idx = start * K * C + width * C + class`.
 *
 * Returns an empty array when the tensor or tokens are empty so
 * callers don't need to guard.
 */
export function decodeSpans(
  modelOutput: Float32Array | readonly number[],
  shape: readonly [number, number, number],
  tokenOffsets: readonly TokenOffset[],
  classLabels: readonly PiiEntity[],
  threshold: number = DEFAULT_THRESHOLD,
): RawSpan[] {
  const [L, K, C] = shape;
  const numTokens = tokenOffsets.length;
  if (L === 0 || K === 0 || C === 0 || numTokens === 0) return [];

  const out: RawSpan[] = [];
  const walkL = Math.min(L, numTokens);
  const walkC = Math.min(C, classLabels.length);
  for (let s = 0; s < walkL; s++) {
    for (let k = 0; k < K; k++) {
      const endToken = s + k + 1;
      if (endToken > numTokens) continue;
      for (let c = 0; c < walkC; c++) {
        const idx = s * K * C + k * C + c;
        const logit = modelOutput[idx];
        const prob = sigmoid(logit);
        if (prob <= threshold) continue;
        out.push({
          startToken: s,
          endToken,
          classIdx: c,
          entity: classLabels[c],
          score: prob,
        });
      }
    }
  }
  return out;
}

/**
 * Flat-NER greedy deduplication : sort candidates by descending
 * score, then iteratively keep a span only if it does not overlap
 * with any previously-kept span. Output is re-sorted by start token
 * so downstream consumers see findings in document order.
 *
 * Matches the `greedy_search(spans, flat_ner=True, multi_label=False)`
 * behaviour of the upstream decoder.
 */
export function greedyDedup(spans: readonly RawSpan[]): RawSpan[] {
  if (spans.length === 0) return [];
  const sorted = [...spans].sort((a, b) => b.score - a.score);
  const kept: RawSpan[] = [];
  for (const span of sorted) {
    const collides = kept.some(
      (k) =>
        !(span.endToken <= k.startToken || span.startToken >= k.endToken),
    );
    if (!collides) kept.push(span);
  }
  kept.sort((a, b) => a.startToken - b.startToken);
  return kept;
}

/**
 * Translate a token-index span into a character-offset
 * {@link PiiFinding} using the tokenizer's offset mapping.
 * Pre-condition : `span.startToken` and `span.endToken - 1` are both
 * valid indices into `tokenOffsets` (enforced by {@link decodeSpans}).
 */
export function toFinding(
  span: RawSpan,
  tokenOffsets: readonly TokenOffset[],
): PiiFinding {
  const charStart = tokenOffsets[span.startToken].start;
  const charEnd = tokenOffsets[span.endToken - 1].end;
  return {
    entity: span.entity,
    start: charStart,
    end: charEnd,
    confidence: span.score,
  };
}
