// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase C — bridge authority (intersection) unit tests.
 *
 * Pure model tests (no React) mirroring the canonical Rust
 * `sbfb_manifest::grant_v0`. The useBridge integration lives in
 * `intersection_authoritative.test.ts`.
 */
import { describe, expect, it } from "vitest";
import {
  ALL_DISPATCH_METHODS,
  GRANDFATHER_AUTHORITY,
  effectiveMethods,
  grantLiveMethods,
  isAuthorized,
  resolveAuthorityFromManifest,
  resolveDeclaredAuthority,
  type BridgeAuthority,
  type Grant,
} from "../authority";
import { BRIDGE_METHODS, type BridgeMethod } from "../protocol";

const set = (...m: BridgeMethod[]): Set<BridgeMethod> => new Set(m);

describe("bridge authority (Sprint 83 Phase C)", () => {
  it("ALL_DISPATCH_METHODS mirrors the 16 protocol methods (literal, drift-catching)", () => {
    // Compare against a LITERAL list — NOT another derivation of BRIDGE_METHODS — so
    // a renamed / added / removed method is actually caught here, not silently
    // mirrored on both sides of the assertion.
    const EXPECTED = [
      "task_submit", "storage_get", "storage_set", "pii_redact", "storage_list",
      "storage_delete", "identity_pubkey", "node_status", "browse_list",
      "storage_version", "provenance_get", "provenance_verify", "feed_cursor_get",
      "search", "proof_card_get", "task_result",
    ];
    expect(new Set(BRIDGE_METHODS)).toEqual(new Set(EXPECTED));
    expect([...ALL_DISPATCH_METHODS].sort()).toEqual([...EXPECTED].sort());
    expect(ALL_DISPATCH_METHODS.size).toBe(16);
  });

  it("no verb grants HITL/approval/signing authority to an iframe (Sprint 83 Phase L1, G-HITL)", () => {
    // The structural reason an untrusted iframe cannot approve an effect is that
    // this allowlist is CLOSED and contains no approval verb. Sprint 83 Phase L1
    // (review r2 R5) first tried to assert that from the e2e spec, where the
    // allowlist is not published to the page — the test could only ever `return`.
    // The assertion belongs HERE, where the symbols exist, and it must name the
    // property so a future verb is refused for the RIGHT reason: the literal
    // count above catches any drift, this one catches the drift that matters.
    //
    // A `hitl_decide` / `approval_list` verb added in lockstep on both mirrors
    // keeps `scripts/check-frontier-contracts.sh` GREEN — it checks PARITY, not
    // membership — so this test, not that gate, is the enforcement.
    //
    // HONEST SCOPE of this pattern (ex-review r3 L1-D5-6): it is a NAMING
    // tripwire, not the closure. What actually closes the allowlist at 16 is the
    // literal list in the test above, updated by the same edit. The pattern only
    // pays off for a verb added in lockstep — and the first version missed the
    // vocabulary this repo actually uses for the concept (`human_decision`,
    // `effect_authorize`, `ack_effect`: "decision" does not contain "decide",
    // while `HUMAN_DECISION_CHANNEL` and `ApprovalDecision` are the names in the
    // engine). Widened accordingly, and kept as a list of FORBIDDEN authority
    // stems rather than a clever substring heuristic.
    const AUTHORITY_STEMS =
      /hitl|approv|decid|decision|sign|consent|authori[sz]|grant|allow|confirm|attest|permit|human|verdict/i;
    const authorityVerbs = BRIDGE_METHODS.filter((m) =>
      AUTHORITY_STEMS.test(m),
    );
    expect(authorityVerbs).toEqual([]);
  });

  describe("effectiveMethods — monotone attenuation", () => {
    it("declared ∩ grant keeps only what BOTH admit", () => {
      const auth: BridgeAuthority = {
        declared: set("storage_get", "storage_set", "search"),
        grant: {
          methods: set("storage_set", "search", "task_submit"),
          expiresAtMs: null,
          revoked: false,
        },
        policy: null,
      };
      expect(effectiveMethods(auth, 0)).toEqual(set("storage_set", "search"));
    });

    it("a null layer imposes no constraint (never widens)", () => {
      const grant: Grant = {
        methods: set("storage_get", "storage_set"),
        expiresAtMs: null,
        revoked: false,
      };
      expect(
        effectiveMethods({ declared: null, grant, policy: null }, 0),
      ).toEqual(set("storage_get", "storage_set"));
    });

    it("policy can only attenuate", () => {
      const declared = set("storage_get", "storage_set");
      const grant: Grant = { methods: declared, expiresAtMs: null, revoked: false };
      expect(
        effectiveMethods({ declared, grant, policy: set("storage_get") }, 0),
      ).toEqual(set("storage_get"));
    });
    it("bounds the result to the dispatch allowlist (off-allowlist grant authorizes nothing)", () => {
      // Domain bound: even a grandfather-shaped authority (declared=null) cannot
      // authorize a name outside the 16 dispatchable verbs.
      const grant: Grant = {
        methods: new Set([...BRIDGE_METHODS, "evil_method" as BridgeMethod]),
        expiresAtMs: null,
        revoked: false,
      };
      const auth: BridgeAuthority = { declared: null, grant, policy: null };
      expect(effectiveMethods(auth, 0).has("evil_method" as BridgeMethod)).toBe(false);
      expect(effectiveMethods(auth, 0).has("search")).toBe(true);
    });
  });

  describe("grant liveness", () => {
    it("expired grant authorizes nothing (at and past expiry)", () => {
      const grant: Grant = { methods: set("search"), expiresAtMs: 1000, revoked: false };
      expect(grantLiveMethods(grant, 999).size).toBe(1);
      expect(grantLiveMethods(grant, 1000).size).toBe(0);
      expect(grantLiveMethods(grant, 5000).size).toBe(0);
    });

    it("revoked grant authorizes nothing", () => {
      const grant: Grant = { methods: set("search"), expiresAtMs: null, revoked: true };
      expect(grantLiveMethods(grant, 0).size).toBe(0);
    });
  });

  describe("isAuthorized — fail-closed", () => {
    const declared = set("storage_get");
    const grant: Grant = { methods: set("storage_get"), expiresAtMs: null, revoked: false };

    it("allows a declared + granted method", () => {
      expect(isAuthorized({ declared, grant, policy: null }, "storage_get", 0)).toBe(true);
    });
    it("refuses an undeclared method (usurpation)", () => {
      expect(isAuthorized({ declared, grant, policy: null }, "task_submit", 0)).toBe(false);
    });
    it("refuses after expiry", () => {
      const g: Grant = { methods: set("storage_get"), expiresAtMs: 100, revoked: false };
      expect(isAuthorized({ declared, grant: g, policy: null }, "storage_get", 100)).toBe(false);
    });
    it("refuses after revocation", () => {
      const g: Grant = { methods: set("storage_get"), expiresAtMs: null, revoked: true };
      expect(isAuthorized({ declared, grant: g, policy: null }, "storage_get", 0)).toBe(false);
    });
  });

  describe("resolveDeclaredAuthority", () => {
    it("null -> grandfather (all 16 verbs allowed)", () => {
      const auth = resolveDeclaredAuthority(null);
      expect(auth).toBe(GRANDFATHER_AUTHORITY);
      for (const m of BRIDGE_METHODS) expect(isAuthorized(auth, m, 0)).toBe(true);
    });
    it("declared list -> fail-closed on that set", () => {
      const auth = resolveDeclaredAuthority(["storage_get", "search"]);
      expect(isAuthorized(auth, "storage_get", 0)).toBe(true);
      expect(isAuthorized(auth, "search", 0)).toBe(true);
      expect(isAuthorized(auth, "task_submit", 0)).toBe(false);
    });
    it("empty declared list -> refuses everything (declared-but-empty)", () => {
      const auth = resolveDeclaredAuthority([]);
      for (const m of BRIDGE_METHODS) expect(isAuthorized(auth, m, 0)).toBe(false);
    });
    it("drops off-allowlist names (never dispatchable)", () => {
      const auth = resolveDeclaredAuthority(["storage_get", "evil_method"]);
      expect(isAuthorized(auth, "storage_get", 0)).toBe(true);
      expect(
        (auth.declared as Set<BridgeMethod>).has("evil_method" as BridgeMethod),
      ).toBe(false);
    });
  });

  describe("resolveAuthorityFromManifest", () => {
    it("null / non-object manifest -> grandfather", () => {
      expect(resolveAuthorityFromManifest(null)).toBe(GRANDFATHER_AUTHORITY);
      expect(resolveAuthorityFromManifest("nope")).toBe(GRANDFATHER_AUTHORITY);
      expect(resolveAuthorityFromManifest(42)).toBe(GRANDFATHER_AUTHORITY);
    });
    it("no `bridge` key (or null) -> grandfather (legacy)", () => {
      expect(resolveAuthorityFromManifest({ name: "legacy" })).toBe(GRANDFATHER_AUTHORITY);
      expect(resolveAuthorityFromManifest({ name: "x", bridge: null })).toBe(
        GRANDFATHER_AUTHORITY,
      );
    });
    it("present `bridge` with methods -> fail-closed on that set", () => {
      const auth = resolveAuthorityFromManifest({
        bridge: { methods: ["search", "storage_get"] },
      });
      expect(isAuthorized(auth, "search", 0)).toBe(true);
      expect(isAuthorized(auth, "storage_get", 0)).toBe(true);
      expect(isAuthorized(auth, "task_submit", 0)).toBe(false);
    });
    it("present `bridge` without a methods array -> fail-closed empty", () => {
      for (const manifest of [
        { bridge: {} },
        { bridge: { methods: [] } },
        { bridge: { methods: "weird" } },
        { bridge: "weird" },
      ]) {
        const auth = resolveAuthorityFromManifest(manifest);
        for (const m of BRIDGE_METHODS) expect(isAuthorized(auth, m, 0)).toBe(false);
      }
    });
    it("drops non-string / off-allowlist declared entries", () => {
      const auth = resolveAuthorityFromManifest({
        bridge: { methods: ["search", 3, "evil_method"] },
      });
      expect(isAuthorized(auth, "search", 0)).toBe(true);
      expect((auth.declared as Set<BridgeMethod>).size).toBe(1);
    });
  });
});
