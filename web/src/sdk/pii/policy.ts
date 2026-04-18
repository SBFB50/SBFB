// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — PII redaction policy types.
 *
 * Defense-in-depth layer 1 (iframe-client-facing). The host shell
 * runs the SDK and exposes it to sandboxed apps through the
 * `pii_redact` bridge method. A twin Presidio-based detector runs
 * coord-side in Phase C as layer 2.
 */

export type PiiEntity =
  | "PERSON"
  | "EMAIL_ADDRESS"
  | "PHONE_NUMBER"
  | "CREDIT_CARD"
  | "SSN"
  | "IBAN"
  | "IP_ADDRESS"
  | "MEDICAL_LICENSE"
  | "US_PASSPORT"
  | "URL";

export interface PiiPolicy {
  /** If false, redact is a no-op (text passes through unchanged). */
  enabled: boolean;
  /** Whitelist of entity types to redact. */
  entities: PiiEntity[];
  /** Replacement template. `{ENTITY}` is substituted per finding. */
  replacement: string;
  /** Findings with `confidence < confidence_threshold` are dropped. */
  confidence_threshold: number;
  /**
   * If false, skip the GLiNER ONNX model path and use only the
   * curated regex fallback. Useful for environments where the
   * model asset is not available or where latency matters more
   * than NER coverage.
   */
  use_model: boolean;
}

export interface PiiFinding {
  entity: PiiEntity;
  start: number;
  end: number;
  confidence: number;
}

export const DEFAULT_POLICY: PiiPolicy = {
  enabled: true,
  entities: [
    "PERSON",
    "EMAIL_ADDRESS",
    "PHONE_NUMBER",
    "CREDIT_CARD",
    "SSN",
    "IBAN",
  ],
  replacement: "[REDACTED:{ENTITY}]",
  confidence_threshold: 0.5,
  use_model: true,
};

/**
 * Merge a partial override with DEFAULT_POLICY. Missing fields fall
 * back to the default, so apps only need to specify what changes.
 */
export function resolvePolicy(override?: Partial<PiiPolicy>): PiiPolicy {
  if (!override) return DEFAULT_POLICY;
  return {
    enabled: override.enabled ?? DEFAULT_POLICY.enabled,
    entities: override.entities ?? DEFAULT_POLICY.entities,
    replacement: override.replacement ?? DEFAULT_POLICY.replacement,
    confidence_threshold:
      override.confidence_threshold ?? DEFAULT_POLICY.confidence_threshold,
    use_model: override.use_model ?? DEFAULT_POLICY.use_model,
  };
}

/**
 * Filter findings through the policy: drop entities not in the
 * whitelist and drop findings below the confidence threshold.
 * Pure function for testability.
 */
export function filterFindings(
  findings: PiiFinding[],
  policy: PiiPolicy,
): PiiFinding[] {
  const allowed = new Set(policy.entities);
  return findings.filter(
    (f) => allowed.has(f.entity) && f.confidence >= policy.confidence_threshold,
  );
}
