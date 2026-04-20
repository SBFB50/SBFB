// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B (scaffold) + Sprint 22 Phase B (decoder wire) —
 * GLiNER ONNX runtime wrapper.
 *
 * Lazy-loads `onnxruntime-web` and `@huggingface/transformers` the
 * first time {@link GlinerPiiDetector.detect} is called. If the
 * model asset is missing (404) or the WASM runtime fails to boot,
 * the detector transparently falls back to {@link fallbackDetect}
 * so an iframe app never sees a hard error — it just gets
 * regex-only findings. The layer-2 Presidio redactor (Phase C)
 * catches anything that slipped through.
 *
 * Sprint 22 Phase B replaces the scaffold `return []` post-inference
 * with the pure {@link decodeSpans} + {@link greedyDedup} pipeline
 * from `decoder.ts` — the wrapper now owns only the ORT / tokenizer
 * glue (offset-mapping, tensor picking, rank-4 batch squeeze).
 *
 * The detector is a module-scope singleton: one model load for
 * the whole shell, shared across every iframe that calls
 * `bridge.piiRedact()`.
 */

import type { PiiFinding, PiiPolicy } from "./policy";
import { filterFindings } from "./policy";
import { fallbackDetect } from "./fallback";
import {
  DEFAULT_THRESHOLD,
  PII_ENTITY_LABELS,
  type TokenOffset,
  decodeSpans,
  greedyDedup,
  toFinding,
} from "./decoder";

/** Default location served by Vite from `web/public/models/`. */
export const DEFAULT_MODEL_URL = "/models/gliner-pii-edge-v1.0.onnx";

/** Model repo id on HuggingFace — used by the tokenizer loader. */
const TOKENIZER_REPO = "knowledgator/gliner-pii-edge-v1.0";

/**
 * Thin seam so tests can inject a deterministic loader instead of
 * hitting `onnxruntime-web` + `@huggingface/transformers` dynamic
 * imports (which would need the real WASM runtime in jsdom).
 */
export interface ModelLoader {
  load(modelUrl: string): Promise<ModelHandle>;
}

export interface ModelHandle {
  /**
   * `threshold` is the sigmoid cutoff applied before greedy dedup —
   * keeping it at the handle boundary lets callers feed the caller-
   * specific policy threshold all the way down to the decoder,
   * instead of running a default floor of 0.5 and re-filtering later
   * (which would silently clamp looser policies).
   */
  detect(text: string, threshold?: number): Promise<PiiFinding[]>;
}

class DefaultModelLoader implements ModelLoader {
  async load(modelUrl: string): Promise<ModelHandle> {
    const [ort, tf] = await Promise.all([
      import("onnxruntime-web"),
      import("@huggingface/transformers"),
    ]);
    const session = await ort.InferenceSession.create(modelUrl);
    // `@huggingface/transformers` v4 exposes AutoTokenizer via a
    // named export; we access it defensively so the shape mismatch
    // between typings and runtime doesn't trip strict mode.
    const transformers = tf as unknown as {
      AutoTokenizer: {
        from_pretrained(repo: string): Promise<TokenizerLike>;
      };
    };
    const tokenizer = await transformers.AutoTokenizer.from_pretrained(
      TOKENIZER_REPO,
    );
    return new OnnxModelHandle(session, tokenizer);
  }
}

interface OffsetMappingLike {
  data: ReadonlyArray<ReadonlyArray<number>> | Int32Array;
  dims?: readonly number[];
}

interface TokenizerEncoding {
  input_ids: { data: BigInt64Array | Int32Array };
  attention_mask: { data: BigInt64Array | Int32Array };
  /**
   * Populated when the caller passes `{ return_offsets_mapping:
   * true }`. Each pair is `[charStart, charEnd]` mapping the token
   * at the same index back to the original text. Optional because
   * older tokenizers / test stubs may omit it.
   */
  offset_mapping?: OffsetMappingLike;
}

interface TokenizerLike {
  (text: string, opts?: Record<string, unknown>): TokenizerEncoding;
}

/**
 * Mirrors the tiny slice of the real ORT `Tensor` type that the
 * decoder glue reads. The `data` field is `unknown` because ORT
 * widens it to a union of eleven typed arrays + `string[]` ; we
 * narrow it at runtime in {@link toFloat32Array} so this interface
 * stays compatible with both the real `InferenceSession` and the
 * synthetic stub used in unit tests.
 */
interface OrtTensorLike {
  data: unknown;
  dims: readonly number[];
}

interface OrtSessionLike {
  run(inputs: Record<string, unknown>): Promise<Record<string, OrtTensorLike>>;
  readonly outputNames?: readonly string[];
}

/**
 * Bridge between the ONNX session output and {@link PiiFinding}.
 * The span-logits decoder lives in `decoder.ts` ; this class owns
 * only the ORT / tokenizer glue : tokenise with offset-mapping, run
 * inference, pick the logits tensor, then delegate to the pure
 * decoder. If inference throws or returns an unexpected shape, the
 * caller's try/catch falls back to regex for that call.
 */
class OnnxModelHandle implements ModelHandle {
  private readonly session: OrtSessionLike;
  private readonly tokenizer: TokenizerLike;

  constructor(session: OrtSessionLike, tokenizer: TokenizerLike) {
    this.session = session;
    this.tokenizer = tokenizer;
  }

  async detect(text: string, threshold: number = DEFAULT_THRESHOLD): Promise<PiiFinding[]> {
    const encoded = this.tokenizer(text, { return_offsets_mapping: true });
    const inputIds = encoded.input_ids.data;
    const attentionMask = encoded.attention_mask.data;
    const tokenOffsets = extractOffsets(encoded.offset_mapping);
    if (tokenOffsets.length === 0) return [];
    const outputs = await this.session.run({
      input_ids: inputIds,
      attention_mask: attentionMask,
    });
    const logits = pickLogitsTensor(outputs, this.session.outputNames);
    const shape = squeezeBatchDim(logits.dims);
    const rawData = toFloat32Array(logits.data);
    const spans = decodeSpans(
      rawData,
      shape,
      tokenOffsets,
      PII_ENTITY_LABELS,
      threshold,
    );
    return greedyDedup(spans).map((s) => toFinding(s, tokenOffsets));
  }
}

/**
 * Narrow the wide ORT `Tensor.data` union into a `Float32Array` the
 * decoder can index. Accepts raw `Float32Array` (production path),
 * a plain `number[]` (test fixtures), or any other typed-array the
 * ORT runtime might surface (int32 / float64 etc. — the span-logits
 * head always exports float32 but we stay defensive). Throws if the
 * data is non-numeric (e.g. the ORT `string[]` branch, which a
 * valid GLiNER logits tensor never takes).
 */
function toFloat32Array(data: unknown): Float32Array {
  if (data instanceof Float32Array) return data;
  if (ArrayBuffer.isView(data)) {
    // DataView / generic TypedArray — Float32Array.from accepts any
    // ArrayLike<number>, and TypedArrays (except BigInt64Array /
    // BigUint64Array) expose numeric indexing.
    return Float32Array.from(data as unknown as ArrayLike<number>);
  }
  if (Array.isArray(data)) {
    if (data.length > 0 && typeof data[0] !== "number") {
      throw new Error("GLiNER logits tensor contains non-numeric data");
    }
    return Float32Array.from(data as readonly number[]);
  }
  throw new Error("GLiNER logits tensor data has an unsupported layout");
}

/**
 * Normalise the tokenizer's `offset_mapping` field into a flat
 * array of {@link TokenOffset}. Supports both the `@huggingface/
 * transformers` v4 nested-array layout and the Int32Array layout
 * emitted by fast tokenizers in some builds.
 */
function extractOffsets(
  offsetMapping: OffsetMappingLike | undefined,
): TokenOffset[] {
  if (!offsetMapping) return [];
  const { data, dims } = offsetMapping;
  if (data instanceof Int32Array) {
    // Flat row-major [[s0, e0], [s1, e1], ...]. When dims is missing
    // we assume `data.length / 2` pairs.
    const pairs = dims && dims.length === 2 ? dims[0] : data.length / 2;
    const out: TokenOffset[] = [];
    for (let i = 0; i < pairs; i++) {
      out.push({ start: data[i * 2], end: data[i * 2 + 1] });
    }
    return out;
  }
  const nested = data as ReadonlyArray<ReadonlyArray<number>>;
  return nested.map((pair) => ({ start: pair[0], end: pair[1] }));
}

/**
 * Select the span-logits tensor from the ORT run output. Prefers an
 * output named `logits` (the GLiNER ONNX export convention) and
 * falls back to the first entry otherwise.
 */
function pickLogitsTensor(
  outputs: Record<string, OrtTensorLike>,
  outputNames?: readonly string[],
): OrtTensorLike {
  if (outputs.logits) return outputs.logits;
  if (outputNames && outputNames.length > 0) {
    const first = outputs[outputNames[0]];
    if (first) return first;
  }
  const values = Object.values(outputs);
  if (values.length === 0) {
    throw new Error("ONNX session returned no output tensors");
  }
  return values[0];
}

/**
 * The ONNX export ships a `(B, L, K, C)` rank-4 tensor. We only ever
 * pass `batch_size == 1`, so the decoder works on the rank-3 view
 * `(L, K, C)`. Accepts rank-3 input unchanged for test fixtures.
 */
function squeezeBatchDim(
  dims: readonly number[],
): readonly [number, number, number] {
  if (dims.length === 4) {
    return [dims[1], dims[2], dims[3]] as const;
  }
  if (dims.length === 3) {
    return [dims[0], dims[1], dims[2]] as const;
  }
  throw new Error(
    `unexpected GLiNER output rank ${dims.length} (expected 3 or 4)`,
  );
}

/**
 * Host-shell singleton PII detector. One instance per page load;
 * `GlinerPiiDetector.shared` is the canonical accessor used by the
 * bridge dispatch.
 */
export class GlinerPiiDetector {
  private handle: ModelHandle | null = null;
  private loadError: Error | null = null;
  private loadPromise: Promise<ModelHandle> | null = null;
  private readonly loader: ModelLoader;
  private readonly modelUrl: string;

  constructor(
    loader: ModelLoader = new DefaultModelLoader(),
    modelUrl: string = DEFAULT_MODEL_URL,
  ) {
    this.loader = loader;
    this.modelUrl = modelUrl;
  }

  /**
   * Force the model to load (or fail). Subsequent calls are cached.
   * Tests use this to seed the detector in a known state.
   */
  async ensureLoaded(): Promise<boolean> {
    if (this.handle) return true;
    if (this.loadError) return false;
    if (!this.loadPromise) {
      this.loadPromise = this.loader.load(this.modelUrl);
    }
    try {
      this.handle = await this.loadPromise;
      return true;
    } catch (err) {
      this.loadError = err instanceof Error ? err : new Error(String(err));
      this.handle = null;
      return false;
    }
  }

  /**
   * Detect PII findings in `text` honouring `policy`. Model path is
   * tried first when `policy.use_model === true`; on any failure we
   * delegate to the regex fallback so callers always get a result.
   */
  async detect(text: string, policy: PiiPolicy): Promise<PiiFinding[]> {
    if (!policy.enabled) return [];
    if (!policy.use_model) return fallbackDetect(text, policy);
    const ready = await this.ensureLoaded();
    if (!ready || !this.handle) return fallbackDetect(text, policy);
    try {
      const raw = await this.handle.detect(text, policy.confidence_threshold);
      const filtered = filterFindings(raw, policy);
      if (filtered.length === 0) {
        // Model returns 0 findings = no PII detected. Fallback kept
        // as defense-in-depth (regex catches formats the model misses).
        return fallbackDetect(text, policy);
      }
      return filtered;
    } catch {
      return fallbackDetect(text, policy);
    }
  }

  /** Reset cached state — test-only. */
  reset(): void {
    this.handle = null;
    this.loadError = null;
    this.loadPromise = null;
  }
}

let sharedInstance: GlinerPiiDetector | null = null;

/** Module-scope singleton used by the bridge. */
export function getSharedDetector(): GlinerPiiDetector {
  if (!sharedInstance) sharedInstance = new GlinerPiiDetector();
  return sharedInstance;
}

/** Replace the singleton — test-only. */
export function setSharedDetectorForTests(
  detector: GlinerPiiDetector | null,
): void {
  sharedInstance = detector;
}
