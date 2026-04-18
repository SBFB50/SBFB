// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — client-side PII redaction SDK (iframe-facing).
 *
 * Public entry point used by the host-shell bridge to redact PII
 * before apps push prompts to the coordinator. See
 * `.planning/research/S21_phase_B_iframe_pii_sdk_design.md` for the
 * rationale behind the architecture (host-shell singleton, lazy
 * ONNX, regex fallback) and the defense-in-depth layering with the
 * Presidio-based coord-side redactor (Phase C).
 */

export type { PiiEntity, PiiFinding, PiiPolicy } from "./policy";
export { DEFAULT_POLICY, resolvePolicy, filterFindings } from "./policy";
export { fallbackDetect, redact } from "./fallback";
export {
  DEFAULT_MODEL_URL,
  GlinerPiiDetector,
  getSharedDetector,
  setSharedDetectorForTests,
  type ModelHandle,
  type ModelLoader,
} from "./wrapper";

import type { PiiFinding, PiiPolicy } from "./policy";
import { resolvePolicy } from "./policy";
import { redact } from "./fallback";
import { getSharedDetector } from "./wrapper";

export interface RedactResult {
  /** Text with PII findings replaced by `policy.replacement`. */
  text: string;
  /** Findings discovered (filtered through the policy). */
  findings: PiiFinding[];
}

/**
 * High-level helper used by the bridge dispatch. Resolves the
 * policy (merging with DEFAULT_POLICY), runs detection via the
 * shared host-shell detector, and returns redacted text plus the
 * findings list. When `policy.enabled === false` the input text is
 * passed through unchanged with an empty findings array.
 */
export async function detectAndRedact(
  text: string,
  override?: Partial<PiiPolicy>,
): Promise<RedactResult> {
  const policy = resolvePolicy(override);
  if (!policy.enabled) return { text, findings: [] };
  const findings = await getSharedDetector().detect(text, policy);
  return { text: redact(text, findings, policy), findings };
}
