// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — GLiNER ONNX runtime wrapper.
 *
 * Lazy-loads `onnxruntime-web` and `@huggingface/transformers` the
 * first time {@link GlinerPiiDetector.detect} is called. If the
 * model asset is missing (404) or the WASM runtime fails to boot,
 * the detector transparently falls back to {@link fallbackDetect}
 * so an iframe app never sees a hard error — it just gets
 * regex-only findings. The layer-2 Presidio redactor (Phase C)
 * catches anything that slipped through.
 *
 * The detector is a module-scope singleton: one model load for
 * the whole shell, shared across every iframe that calls
 * `bridge.piiRedact()`.
 */

import type { PiiFinding, PiiPolicy } from "./policy";
import { filterFindings } from "./policy";
import { fallbackDetect } from "./fallback";

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
  detect(text: string): Promise<PiiFinding[]>;
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

interface TokenizerLike {
  (text: string, opts?: Record<string, unknown>): {
    input_ids: { data: BigInt64Array | Int32Array };
    attention_mask: { data: BigInt64Array | Int32Array };
  };
}

interface OrtSessionLike {
  run(inputs: Record<string, unknown>): Promise<Record<string, unknown>>;
}

/**
 * Bridge between the ONNX session output and {@link PiiFinding}.
 * The exact post-processing depends on the GLiNER head layout; for
 * Sprint 21 Phase B the model inference path is wired but not
 * end-to-end exercised in tests (covered by manual dev runs once
 * the model asset is downloaded locally). If inference throws, we
 * rethrow so the caller falls back to regex for that call.
 */
class OnnxModelHandle implements ModelHandle {
  private readonly session: OrtSessionLike;
  private readonly tokenizer: TokenizerLike;

  constructor(session: OrtSessionLike, tokenizer: TokenizerLike) {
    this.session = session;
    this.tokenizer = tokenizer;
  }

  async detect(text: string): Promise<PiiFinding[]> {
    // Tokenize. GLiNER edge expects the text split into candidate
    // spans with entity labels appended via special tokens. For the
    // Phase B scaffold we use a minimal single-query pass; a more
    // refined multi-label querying strategy is a follow-up once
    // the model is exercised against real prompts.
    const encoded = this.tokenizer(text);
    const inputIds = encoded.input_ids.data;
    const attentionMask = encoded.attention_mask.data;
    await this.session.run({
      input_ids: inputIds,
      attention_mask: attentionMask,
    });
    // Post-processing: map span logits to findings. The scaffolded
    // loader returns an empty set until the head decoder lands; the
    // regex fallback fills the gap until then.
    return [];
  }
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
      const raw = await this.handle.detect(text);
      const filtered = filterFindings(raw, policy);
      if (filtered.length === 0) {
        // Scaffold path returns empty; augment with regex so apps
        // in dev (without the model) still get meaningful redaction.
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
