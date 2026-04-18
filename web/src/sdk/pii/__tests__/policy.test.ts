// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — policy type tests.
 */

import { describe, expect, it } from "vitest";

import {
  DEFAULT_POLICY,
  filterFindings,
  resolvePolicy,
  type PiiFinding,
} from "../policy";

describe("DEFAULT_POLICY", () => {
  it("enables redaction and covers the standard entity set", () => {
    expect(DEFAULT_POLICY.enabled).toBe(true);
    expect(DEFAULT_POLICY.use_model).toBe(true);
    expect(DEFAULT_POLICY.entities).toEqual(
      expect.arrayContaining([
        "EMAIL_ADDRESS",
        "PHONE_NUMBER",
        "CREDIT_CARD",
        "SSN",
        "IBAN",
        "PERSON",
      ]),
    );
    expect(DEFAULT_POLICY.replacement).toContain("{ENTITY}");
  });
});

describe("filterFindings", () => {
  it("drops findings below the confidence threshold", () => {
    const findings: PiiFinding[] = [
      { entity: "EMAIL_ADDRESS", start: 0, end: 5, confidence: 0.9 },
      { entity: "EMAIL_ADDRESS", start: 10, end: 15, confidence: 0.3 },
    ];
    const out = filterFindings(findings, DEFAULT_POLICY);
    expect(out).toHaveLength(1);
    expect(out[0].confidence).toBeCloseTo(0.9);
  });
});

describe("resolvePolicy (disabled pass-through)", () => {
  it("treats enabled:false as a no-op sentinel", () => {
    const policy = resolvePolicy({ enabled: false });
    expect(policy.enabled).toBe(false);
    // All other fields should fall back to the defaults so a caller
    // that later flips enabled:true gets consistent behavior.
    expect(policy.replacement).toBe(DEFAULT_POLICY.replacement);
    expect(policy.entities).toEqual(DEFAULT_POLICY.entities);
  });
});
