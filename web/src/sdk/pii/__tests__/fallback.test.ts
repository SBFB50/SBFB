// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 21 Phase B — regex fallback detector tests.
 */

import { describe, expect, it } from "vitest";

import { DEFAULT_POLICY } from "../policy";
import { fallbackDetect, redact } from "../fallback";

describe("fallbackDetect", () => {
  it("detects a simple email address", () => {
    const text = "Contact me at user@example.com if needed";
    const findings = fallbackDetect(text, DEFAULT_POLICY);
    const emails = findings.filter((f) => f.entity === "EMAIL_ADDRESS");
    expect(emails).toHaveLength(1);
    expect(text.slice(emails[0].start, emails[0].end)).toBe(
      "user@example.com",
    );
  });

  it("detects phone numbers in E.164 and U.S. layouts", () => {
    const text = "Call +33123456789 or (415) 555-0199 for support";
    const findings = fallbackDetect(text, DEFAULT_POLICY);
    const phones = findings.filter((f) => f.entity === "PHONE_NUMBER");
    expect(phones.length).toBeGreaterThanOrEqual(2);
  });

  it("accepts credit card numbers that pass the Luhn check", () => {
    // 4111 1111 1111 1111 is a well-known Visa test number (valid Luhn).
    const valid = "Pay with 4111 1111 1111 1111 thanks";
    const invalid = "Order 1234 5678 9012 3456 shipped"; // fails Luhn
    const v = fallbackDetect(valid, DEFAULT_POLICY).filter(
      (f) => f.entity === "CREDIT_CARD",
    );
    const i = fallbackDetect(invalid, DEFAULT_POLICY).filter(
      (f) => f.entity === "CREDIT_CARD",
    );
    expect(v).toHaveLength(1);
    expect(i).toHaveLength(0);
  });

  it("accepts valid SSN and rejects structural sentinels", () => {
    const valid = "SSN 123-45-6789 for the record";
    const invalid = "Use 000-12-3456 as placeholder";
    const v = fallbackDetect(valid, DEFAULT_POLICY).filter(
      (f) => f.entity === "SSN",
    );
    const i = fallbackDetect(invalid, DEFAULT_POLICY).filter(
      (f) => f.entity === "SSN",
    );
    expect(v).toHaveLength(1);
    expect(i).toHaveLength(0);
  });

  it("detects IBAN-shaped identifiers", () => {
    const text = "Transfer to FR7630001007941234567890185 today";
    const findings = fallbackDetect(text, DEFAULT_POLICY);
    const ibans = findings.filter((f) => f.entity === "IBAN");
    expect(ibans).toHaveLength(1);
  });

  it("applies the replacement template with {ENTITY} substitution", () => {
    const text = "Email foo@bar.com or call +33123456789";
    const findings = fallbackDetect(text, DEFAULT_POLICY);
    const out = redact(text, findings, DEFAULT_POLICY);
    expect(out).toContain("[REDACTED:EMAIL_ADDRESS]");
    expect(out).toContain("[REDACTED:PHONE_NUMBER]");
    expect(out).not.toContain("foo@bar.com");
    expect(out).not.toContain("+33123456789");
  });
});
