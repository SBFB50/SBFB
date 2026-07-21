// SPDX-License-Identifier: AGPL-3.0-or-later
import { readFileSync } from "node:fs";
import { resolve as pathResolve } from "node:path";

import { blake3 } from "@noble/hashes/blake3";
import { describe, expect, it } from "vitest";

/**
 * Cross-language parity for `SBFBPromptAgentPack.v1` digests.
 *
 * The Rust side (`sbfb-workflow-core::pack::tests::jcs_cross_lang_parity`)
 * reads the SAME two files and asserts the SAME digests. A pack signed on one
 * node and verified in a browser is only possible if both sides canonicalise
 * to identical bytes, so this test is the thing that makes "byte-identical
 * cross-language" a measured claim instead of an intention.
 *
 * The canonicaliser below is a first-party TEST UTILITY on purpose: shipping a
 * JCS dependency into the app bundle would be a runtime dependency added for a
 * verification concern the shell does not yet have. `@noble/hashes` is declared
 * as a devDependency for the same reason — it stays out of `dist/`.
 */

/** The acceptance boundary MUST match the Rust door, or the two agree on the
 * vectors while disagreeing on what they refuse — which is where a real
 * divergence would hide (external cross-verifier, round 4). */
class NonCanonical extends Error {}

/**
 * RFC 8785 (JCS) canonical JSON, with the SAME refusals as the Rust
 * canonicalisation door.
 *
 * Compatibility rests on three details:
 *  - Member ordering is `Array.prototype.sort()` on the keys, which sorts by
 *    UTF-16 code unit — exactly what RFC 8785 §3.2.3 prescribes, and NOT the
 *    same as code-point order for non-BMP keys (the discriminant vector is why
 *    the vector file includes a U+FFFF-vs-emoji case).
 *  - Scalars go through `JSON.stringify`, whose escaping (ES2019 well-formed)
 *    and number formatting are what the RFC points at.
 *  - Refusals mirror the Rust door where a VALUE-SIDE mirror can: `null`,
 *    non-finite numbers, and lone UTF-16 surrogates all throw here just as
 *    they are rejected there. The one Rust refusal a JS mirror cannot
 *    reproduce is the I-JSON integer bound: Rust decides it on the JSON TOKEN
 *    (`9007199254740993` is a u64, `1e21` is a float), a distinction lost once
 *    `JSON.parse` has turned both into one `number`. That class is refused
 *    Rust-side and lives in the Rust-only hostile corpus, never as a parity
 *    vector — see the vectors file comment.
 */
function canonicalize(value: unknown): string {
  if (value === null) {
    throw new NonCanonical("null has no canonical spelling");
  }
  if (Array.isArray(value)) {
    return `[${value.map(canonicalize).join(",")}]`;
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    // Keys walk the same surrogate check as string VALUES — a lone surrogate
    // in a member NAME is refused Rust-side too (external cross-verifier r5).
    for (const k of keys) assertWellFormed(k);
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalize(record[k])}`).join(",")}}`;
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new NonCanonical("non-finite number");
    }
  }
  if (typeof value === "string") {
    assertWellFormed(value);
  }
  return JSON.stringify(value) as string;
}

/** A lone surrogate has no scalar value; well-formed JCS bytes cannot carry one. */
function assertWellFormed(s: string): void {
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c >= 0xd800 && c <= 0xdbff) {
      if (i + 1 >= s.length || (s.charCodeAt(i + 1) & 0xfc00) !== 0xdc00) {
        throw new NonCanonical("lone high surrogate");
      }
      i++;
    } else if (c >= 0xdc00 && c <= 0xdfff) {
      throw new NonCanonical("lone low surrogate");
    }
  }
}

/** `<domain> 0x00 <JCS>` — the shared pre-image shape. */
function canonicalBytes(value: unknown, domain: string): Uint8Array {
  const encoder = new TextEncoder();
  const domainBytes = encoder.encode(domain);
  const bodyBytes = encoder.encode(canonicalize(value));
  const out = new Uint8Array(domainBytes.length + 1 + bodyBytes.length);
  out.set(domainBytes, 0);
  out[domainBytes.length] = 0;
  out.set(bodyBytes, domainBytes.length + 1);
  return out;
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

// vitest cwd is the `web/` dir; the fixtures live one level up in `crates/`.
const crateSrc = (name: string) =>
  pathResolve(process.cwd(), `../crates/sbfb-workflow-core/src/${name}`);

interface VectorFile {
  domain: string;
  vectors: Array<{ name: string; value: unknown }>;
}

const vectors = JSON.parse(readFileSync(crateSrc("pack_jcs_vectors.json"), "utf8")) as VectorFile;
// The golden pins BOTH the BLAKE3 digest of the full pre-image and the
// canonical JCS text itself: the claim under test is byte-identical
// canonicalisation, and a digest alone cannot show WHERE two implementations
// diverge when they do.
const golden = JSON.parse(readFileSync(crateSrc("pack_jcs_digests.golden.json"), "utf8")) as Record<
  string,
  { digest: string; canonical: string }
>;

describe("SBFBPromptAgentPack.v1 — JCS cross-language digest parity", () => {
  it("exercises the real pack signing domain", () => {
    expect(vectors.domain).toBe("nexus-workflow-pack-v1");
  });

  it("covers every golden entry, and only those (no silent drop)", () => {
    const fromVectors = vectors.vectors.map((v) => v.name).sort();
    expect(fromVectors).toEqual(Object.keys(golden).sort());
    expect(fromVectors.length).toBeGreaterThanOrEqual(10);
  });

  for (const vector of vectors.vectors) {
    it(`reproduces the Rust canonical text and digest for '${vector.name}'`, () => {
      // Canonical TEXT first: on divergence this names the exact bytes,
      // where a digest mismatch alone would only say "somewhere".
      expect(canonicalize(vector.value)).toBe(golden[vector.name].canonical);
      const digest = toHex(blake3(canonicalBytes(vector.value, vectors.domain)));
      expect(digest).toBe(golden[vector.name].digest);
    });
  }

  it("orders members by UTF-16 code unit, which is what makes the parity hold", () => {
    // Guard against "fixing" the canonicaliser with a locale-aware or
    // code-point comparator: both change the byte order for non-BMP keys and
    // would silently break every signature verified in a browser.
    const canonical = canonicalize({ "😀": 1, z: 2, a: 3 });
    expect(canonical).toBe('{"a":3,"z":2,"😀":1}');
  });

  it("canonicalises key order away: two spellings of one object, one digest", () => {
    // THE canonicaliser property (blake3 alone could never provide it): the
    // same members in a different source order digest identically, because
    // JCS fixes the member order before hashing.
    const a = toHex(blake3(canonicalBytes({ a: 1, b: 2, c: 3 }, vectors.domain)));
    const b = toHex(blake3(canonicalBytes({ c: 3, a: 1, b: 2 }, vectors.domain)));
    expect(a).toBe(b);
    // …while an actual VALUE change still separates them.
    const c = toHex(blake3(canonicalBytes({ a: 1, b: 2, c: 4 }, vectors.domain)));
    expect(a).not.toBe(c);
  });

  // The refusal boundary, mirrored from Rust: each of these classes throws
  // here exactly as the Rust door rejects it. Kept in step with the Rust
  // corpus so the two verifiers share ONE acceptance boundary, not two.
  it.each([
    ["explicit null", { x: null }],
    ["non-finite number", { x: Number.POSITIVE_INFINITY }],
    ["lone high surrogate", { x: "\uD800" }],
    ["lone low surrogate", { x: "\uDC00" }],
  ])("refuses %s just as the Rust door does", (_label, value) => {
    expect(() => canonicalize(value)).toThrow(NonCanonical);
  });

  it("accepts a well-formed surrogate PAIR (a real emoji is not a lone surrogate)", () => {
    expect(() => canonicalize({ x: "😀" })).not.toThrow();
  });
});
