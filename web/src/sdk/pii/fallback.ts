// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — curated regex-based PII fallback detector.
 *
 * Used when the GLiNER ONNX model is unavailable (absent in CI, CSP
 * blocker, WASM runtime fail, or when `policy.use_model === false`).
 * Covers the five highest-frequency entities that do not require
 * NER context to disambiguate: EMAIL_ADDRESS, PHONE_NUMBER,
 * CREDIT_CARD (with Luhn check), SSN, IBAN. PERSON is deliberately
 * NOT covered by regex — layer 2 Presidio coord-side handles it.
 */

import type { PiiEntity, PiiFinding, PiiPolicy } from "./policy";
import { filterFindings } from "./policy";

// -----------------------------------------------------------------
// Regex rules — each rule returns raw candidates, a post-filter
// function decides whether to keep them.
// -----------------------------------------------------------------

interface Rule {
  entity: PiiEntity;
  pattern: RegExp;
  postFilter?: (raw: string) => boolean;
  confidence: number;
}

/**
 * Luhn mod-10 check for credit-card number validation. Filters out
 * accidental 16-digit sequences that happen to look like card
 * numbers (UUID fragments, OCR noise, version strings padded to 16
 * digits, etc.).
 */
function luhnValid(digits: string): boolean {
  const stripped = digits.replace(/[\s-]/g, "");
  if (!/^\d{13,19}$/.test(stripped)) return false;
  let sum = 0;
  let double = false;
  for (let i = stripped.length - 1; i >= 0; i--) {
    let d = stripped.charCodeAt(i) - 48;
    if (double) {
      d *= 2;
      if (d > 9) d -= 9;
    }
    sum += d;
    double = !double;
  }
  return sum % 10 === 0;
}

/**
 * SSN structural validation. U.S. Social Security numbers with
 * area number 000, 666, or 900-999 are never issued and are
 * commonly used as test fixtures — rejected to cut false positives.
 */
function ssnValid(raw: string): boolean {
  const m = raw.match(/^(\d{3})-(\d{2})-(\d{4})$/);
  if (!m) return false;
  const area = parseInt(m[1], 10);
  const group = parseInt(m[2], 10);
  const serial = parseInt(m[3], 10);
  if (area === 0 || area === 666 || area >= 900) return false;
  if (group === 0) return false;
  if (serial === 0) return false;
  return true;
}

const RULES: Rule[] = [
  {
    entity: "EMAIL_ADDRESS",
    pattern: /[\w.+-]+@[\w-]+\.[\w.-]{2,}/g,
    confidence: 0.95,
  },
  {
    entity: "PHONE_NUMBER",
    // E.164 international + common U.S. "(XXX) XXX-XXXX" layout.
    pattern:
      /\+[1-9]\d{1,14}|\(\d{3}\)\s?\d{3}-\d{4}|\b\d{3}-\d{3}-\d{4}\b/g,
    confidence: 0.9,
  },
  {
    entity: "CREDIT_CARD",
    pattern: /\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/g,
    postFilter: luhnValid,
    confidence: 0.95,
  },
  {
    entity: "SSN",
    pattern: /\b\d{3}-\d{2}-\d{4}\b/g,
    postFilter: ssnValid,
    confidence: 0.85,
  },
  {
    entity: "IBAN",
    // Country code + 2 check digits + up to 30 alphanumeric. We
    // deliberately keep this permissive; layer 2 Presidio applies
    // MOD-97 validation for high-stakes paths.
    pattern: /\b[A-Z]{2}\d{2}[A-Z0-9\s]{11,30}\b/g,
    confidence: 0.75,
  },
];

/**
 * Run the regex rules over the given text and return all findings
 * that pass their post-filter. The policy is then used to drop
 * entities the caller did not whitelist. Findings are sorted by
 * start offset ascending so `redact()` can walk left-to-right.
 */
export function fallbackDetect(text: string, policy: PiiPolicy): PiiFinding[] {
  const raw: PiiFinding[] = [];
  for (const rule of RULES) {
    rule.pattern.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = rule.pattern.exec(text)) !== null) {
      const [value] = match;
      if (rule.postFilter && !rule.postFilter(value)) continue;
      raw.push({
        entity: rule.entity,
        start: match.index,
        end: match.index + value.length,
        confidence: rule.confidence,
      });
    }
  }
  raw.sort((a, b) => a.start - b.start);
  return filterFindings(raw, policy);
}

/**
 * Replace all findings in `text` with `policy.replacement`, where
 * `{ENTITY}` in the template is substituted with the actual entity
 * type of each finding. Overlapping findings are resolved left-most
 * wins (skip any finding that starts before the previous one ended).
 */
export function redact(
  text: string,
  findings: PiiFinding[],
  policy: PiiPolicy,
): string {
  if (findings.length === 0) return text;
  const sorted = [...findings].sort((a, b) => a.start - b.start);
  let out = "";
  let cursor = 0;
  for (const f of sorted) {
    if (f.start < cursor) continue;
    out += text.slice(cursor, f.start);
    out += policy.replacement.replace("{ENTITY}", f.entity);
    cursor = f.end;
  }
  out += text.slice(cursor);
  return out;
}
