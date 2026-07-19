// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase C — regression guard for the bridge authority intersection.
 *
 * Phase C makes an app's declared `bridge.methods` AUTHORITATIVE (fail-closed):
 * a declared app can reach ONLY the methods it lists. So every example app that
 * declares a `bridge` config MUST declare a SUPERSET of the methods it actually
 * reaches — INCLUDING methods reached TRANSITIVELY through the SDK (e.g.
 * `onStorageUpdate` polls `getStorageVersion` -> `storage_version`).
 *
 * The prior hand-check ("declare ⊇ appele, verifie en review") inspected only
 * DIRECT bridge.* call sites and missed the transitive `storage_version` in
 * sbfb-ideas — under Phase C the intersection silently refused it once the
 * declared authority resolved past the grandfather load window, killing the
 * Ideas Hub live-sync. This test machine-checks the invariant so that class of
 * under-declaration can no longer regress in the vendored example apps.
 *
 * NOTE: this is the deterministic guard for the REPO's example apps (which
 * follow the `const bridge = new SBFBBridge()` convention). The general
 * transitive-analysis CI gate for arbitrary third-party apps stays a carry.
 */
import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, it, expect } from "vitest";

import { BRIDGE_METHODS, type BridgeMethod } from "@/bridge/protocol";

/** Locate the repo's `examples/` by walking up from cwd to the workspace root
 *  (marked by `Cargo.toml`) — robust whether vitest runs from `web/` or root. */
function findExamplesDir(): string {
  let dir = process.cwd();
  for (let i = 0; i < 8; i++) {
    if (existsSync(join(dir, "Cargo.toml")) && existsSync(join(dir, "examples"))) {
      return join(dir, "examples");
    }
    const parent = join(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`could not locate examples/ from cwd=${process.cwd()}`);
}

const EXAMPLES_DIR = findExamplesDir();

/**
 * Mirror of `examples/<app>/sbfb-bridge.js`: each PUBLIC SDK helper -> the bridge
 * method(s) it reaches when called. Most map 1:1 to their own `_call(...)`;
 * `onStorageUpdate` is TRANSITIVE — it starts a 3s poll of `getStorageVersion`,
 * i.e. `storage_version`. Kept honest by `HELPER_MAP_COVERS_SDK` below (any new
 * `_call("x")` in the SDK that no helper maps to fails the drift check).
 */
const HELPER_TO_METHODS: Record<string, BridgeMethod[]> = {
  submitTask: ["task_submit"],
  getTaskResult: ["task_result"],
  getStorage: ["storage_get"],
  setStorage: ["storage_set"],
  piiRedact: ["pii_redact"],
  listStorage: ["storage_list"],
  deleteStorage: ["storage_delete"],
  getIdentityPubkey: ["identity_pubkey"],
  getNodeStatus: ["node_status"],
  getBrowseList: ["browse_list"],
  getStorageVersion: ["storage_version"],
  getProvenanceRecord: ["provenance_get"],
  verifyRelease: ["provenance_verify"],
  getPublicFeedCursor: ["feed_cursor_get"],
  search: ["search"],
  getProofCard: ["proof_card_get"],
  onStorageUpdate: ["storage_version"], // transitive: polls getStorageVersion
};

const ALLOWLIST = new Set<string>(BRIDGE_METHODS);

/** Example directories (each holds an `SBFB.json`). */
function exampleDirs(): string[] {
  return readdirSync(EXAMPLES_DIR)
    .map((name) => join(EXAMPLES_DIR, name))
    .filter((p) => {
      try {
        return statSync(p).isDirectory() && existsSync(join(p, "SBFB.json"));
      } catch {
        return false;
      }
    });
}

/** The declared bridge methods of an app, or `null` if it declares no `bridge`. */
function declaredMethods(dir: string): string[] | null {
  const manifest = JSON.parse(readFileSync(join(dir, "SBFB.json"), "utf8"));
  const bridge = manifest?.bridge;
  if (bridge === undefined || bridge === null) return null; // grandfathered legacy
  return Array.isArray(bridge.methods) ? bridge.methods : [];
}

/** Non-SDK JS sources of an app (recursive, excludes the vendored `sbfb-bridge.js`).
 *  Covers .js/.mjs/.cjs at any depth; HTML-inline scripts are out of scope (the
 *  vendored examples load bridge code via `<script src>`, never inline). */
function appSources(dir: string): string[] {
  const out: string[] = [];
  const walk = (d: string) => {
    for (const e of readdirSync(d, { withFileTypes: true })) {
      if (e.isDirectory()) {
        if (e.name !== "node_modules" && !e.name.startsWith(".")) walk(join(d, e.name));
      } else if (/\.(js|mjs|cjs)$/.test(e.name) && e.name !== "sbfb-bridge.js") {
        out.push(readFileSync(join(d, e.name), "utf8"));
      }
    }
  };
  walk(dir);
  return out;
}

/**
 * Methods an app REACHES: SDK helper usage (transitive included) + any inline
 * `_call("method")`. `search` collides with `String.prototype.search`, so it is
 * anchored to a `bridge.` receiver; the other helper names are distinctive
 * enough to match on any receiver (defends against a non-`bridge` alias).
 */
function reachedMethods(dir: string): Set<string> {
  const out = new Set<string>();
  for (const src of appSources(dir)) {
    for (const [helper, methods] of Object.entries(HELPER_TO_METHODS)) {
      const re =
        helper === "search"
          ? /\bbridge\.search\s*\(/
          : new RegExp(`\\.${helper}\\s*\\(`);
      if (re.test(src)) for (const m of methods) out.add(m);
    }
    for (const match of src.matchAll(/_call\(\s*["']([a-z0-9_]+)["']/g)) {
      out.add(match[1]);
    }
  }
  return out;
}

describe("example bridge declarations (Sprint 83 Phase C fail-closed invariant)", () => {
  it("discovers the vendored example apps", () => {
    // Guards the fixture path: if this drops to 0, path resolution broke and the
    // per-app assertions below would vacuously pass.
    expect(exampleDirs().length).toBeGreaterThanOrEqual(5);
  });

  it.each(exampleDirs().map((d) => [d.split(/[\\/]/).pop()!, d] as const))(
    "%s declares a superset of the bridge methods it reaches (direct + transitive)",
    (_name, dir) => {
      const declared = declaredMethods(dir);
      if (declared === null) return; // no `bridge` key -> grandfathered, no constraint

      const declaredSet = new Set(declared);
      // Every declared name must be a real dispatch verb (else validate() rejects it).
      for (const m of declared) expect(ALLOWLIST.has(m)).toBe(true);

      const reached = reachedMethods(dir);
      const missing = [...reached].filter((m) => !declaredSet.has(m));
      expect(
        missing,
        `${_name} reaches ${JSON.stringify([...reached])} but declares ` +
          `${JSON.stringify(declared)}; undeclared (fail-closed refused): ` +
          `${JSON.stringify(missing)}`,
      ).toEqual([]);
    },
  );

  it("sbfb-ideas declares storage_version (transitive via onStorageUpdate)", () => {
    // Pins the exact regression the completeness audit found: the live-sync poll.
    const declared = declaredMethods(join(EXAMPLES_DIR, "sbfb-ideas"));
    expect(declared).not.toBeNull();
    expect(declared).toContain("storage_version");
  });

  it("HELPER_MAP_COVERS_SDK: every SDK _call() method is mapped by a helper", () => {
    // Drift guard: if the SDK adds a new `_call("x")`, the helper map must learn
    // it (so a future app using the new helper is correctly checked). Read a
    // canonical vendored SDK.
    const sdkPath = join(EXAMPLES_DIR, "sbfb-ideas", "sbfb-bridge.js");
    const sdk = readFileSync(sdkPath, "utf8");
    const sdkMethods = new Set<string>();
    for (const m of sdk.matchAll(/_call\(\s*["']([a-z0-9_]+)["']/g)) sdkMethods.add(m[1]);
    const mapped = new Set<string>(Object.values(HELPER_TO_METHODS).flat());
    const unmapped = [...sdkMethods].filter((m) => !mapped.has(m));
    expect(
      unmapped,
      `SDK exposes methods no helper maps: ${JSON.stringify(unmapped)} — ` +
        `update HELPER_TO_METHODS`,
    ).toEqual([]);
    // And no phantom methods in the map that the allowlist doesn't know.
    for (const m of mapped) expect(ALLOWLIST.has(m)).toBe(true);
  });

  it("HELPER_COVERAGE: every public SDK helper is mapped (a NEW helper forces a map update)", () => {
    // Closes the gap: the method-set drift guard alone misses a NEW helper that
    // maps to an ALREADY-known method (e.g. `readStorage() -> _call("storage_get")`):
    // no method set changes, yet `.readStorage()` in an app would go undetected.
    // Assert every public dispatch helper of the SDK is a key of HELPER_TO_METHODS,
    // so adding one to the SDK forces the map (and the reach scan) to learn it.
    const sdk = readFileSync(join(EXAMPLES_DIR, "sbfb-ideas", "sbfb-bridge.js"), "utf8");
    const NON_DISPATCH = new Set(["constructor", "destroy", "onEvent"]); // no _call path
    const publicHelpers = new Set<string>();
    for (const m of sdk.matchAll(/^\s{2}([a-zA-Z][a-zA-Z0-9]*)\s*\(/gm)) {
      const name = m[1];
      if (name.startsWith("_") || NON_DISPATCH.has(name)) continue;
      publicHelpers.add(name);
    }
    expect(publicHelpers.size).toBeGreaterThanOrEqual(10); // parser sanity
    const unmapped = [...publicHelpers].filter((h) => !(h in HELPER_TO_METHODS));
    expect(
      unmapped,
      `SDK public helpers absent from HELPER_TO_METHODS: ${JSON.stringify(unmapped)} — ` +
        `map each (with any transitive method) so the reach scan detects its usage`,
    ).toEqual([]);
  });
});
