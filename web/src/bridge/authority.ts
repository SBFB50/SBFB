// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase C — host-side bridge authority (intersection) for the shell.
 *
 * TS mirror of the canonical Rust model `sbfb_manifest::grant_v0`. Renders
 * AUTHORITATIVE, per call, the intersection that governs whether a sandboxed app
 * may reach a bridge method:
 *
 *   effective = declared-by-release ∩ grant ∩ local-policy   (∩ consent, inert v0)
 *
 * Monotone attenuation (the ocap / macaroon rule): each PRESENT layer can only
 * SHRINK the set, never widen it. Phase C wires the host-side 5-layer subset of the
 * 7-layer model — the pack-version and compiled-plan layers belong to later phases
 * and are simply absent here (an absent layer is the identity element, imposing no
 * constraint).
 *
 * Two regimes, keyed on the app manifest (`SbfbManifest.bridge`):
 *   - `bridge` present -> `declared = Set(...)` : FAIL-CLOSED on the declared set.
 *   - `bridge` absent  -> `declared = null`     : grandfathered legacy (grant governs).
 *
 * `nowMs` is injected (the caller owns the clock) so expiry and revocation are
 * evaluated per call. Parity with the Rust allowlist is a security invariant gated
 * by scripts/check-frontier-contracts.sh.
 */
import { BRIDGE_METHODS, CHAT_METHODS, type BridgeMethod } from "@/bridge/protocol";

/** A time-bounded, revocable grant of bridge method capabilities to an app/release. */
export interface Grant {
  /** Method names this grant authorizes (a subset of the dispatch allowlist). */
  readonly methods: ReadonlySet<BridgeMethod>;
  /**
   * Absolute expiry, ms since the Unix epoch; `null` = never expires. When
   * non-null it MUST be a finite, non-negative integer (mirrors the Rust `u64`
   * `expires_at_ms`): a `NaN`/`Infinity` would make `nowMs >= expiresAtMs` false
   * and keep the grant live. Phase C only ever emits `null` here (see
   * `resolveDeclaredAuthority`), so the divergence is latent; grant PERSISTENCE
   * (a later phase) must validate a finite u64-range integer at the boundary.
   */
  readonly expiresAtMs: number | null;
  /** Explicit revocation, re-checked on every call (never cached). */
  readonly revoked: boolean;
}

/** Release-declaration layer: a Set (fail-closed) or `null` (grandfathered legacy). */
export type Declared = ReadonlySet<BridgeMethod> | null;

/** Local-policy layer: a Set (attenuates) or `null` (no local constraint, v0). */
export type Policy = ReadonlySet<BridgeMethod> | null;

/** The resolved per-app authority passed into the bridge host. */
export interface BridgeAuthority {
  readonly declared: Declared;
  readonly grant: Grant;
  readonly policy: Policy;
}

/** Methods live at `nowMs`: empty when revoked or expired (fail-closed). */
export function grantLiveMethods(grant: Grant, nowMs: number): Set<BridgeMethod> {
  if (grant.revoked) return new Set();
  if (grant.expiresAtMs !== null && nowMs >= grant.expiresAtMs) return new Set();
  return new Set(grant.methods);
}

function intersect(
  a: Set<BridgeMethod>,
  b: ReadonlySet<BridgeMethod>,
): Set<BridgeMethod> {
  const out = new Set<BridgeMethod>();
  for (const m of a) if (b.has(m)) out.add(m);
  return out;
}

/** The full dispatch set (every host-dispatched verb) as a `Set<BridgeMethod>`. */
export const ALL_DISPATCH_METHODS: ReadonlySet<BridgeMethod> = new Set(
  BRIDGE_METHODS,
);

/**
 * Verbs that are NEVER grandfathered and never granted by a release declaration
 * alone: they additionally need the user's explicit consent (chat v1 — an app
 * reaching them can make the node join conversations and hand out write access
 * to them). Mirror of Rust `SENSITIVE_BRIDGE_METHODS` (`sbfb-manifest`).
 */
export const SENSITIVE_METHODS: ReadonlySet<BridgeMethod> = CHAT_METHODS;

/** The verbs a legacy app keeps without asking: everything but the sensitive class. */
export const GRANDFATHER_METHODS: ReadonlySet<BridgeMethod> = new Set(
  [...ALL_DISPATCH_METHODS].filter((m) => !SENSITIVE_METHODS.has(m)),
);

/**
 * Effective, currently-authorized method set. Starts from the grant's live methods,
 * intersects every PRESENT constraint layer (a `null` layer imposes no constraint,
 * never widens), then is finally bounded by the dispatchable universe. Single source
 * of truth for the intersection.
 */
export function effectiveMethods(
  auth: BridgeAuthority,
  nowMs: number,
): Set<BridgeMethod> {
  let eff = grantLiveMethods(auth.grant, nowMs);
  if (auth.declared !== null) eff = intersect(eff, auth.declared);
  if (auth.policy !== null) eff = intersect(eff, auth.policy);
  // Domain bound (belt-and-suspenders, mirrors Rust `effective_methods`): never
  // exceed the dispatchable universe, so a grant carrying an off-allowlist name
  // authorizes nothing beyond the dispatch verbs — regardless of how it was built.
  // `req.method` is already Zod-bounded upstream; this is defense in depth.
  eff = intersect(eff, ALL_DISPATCH_METHODS);
  return eff;
}

/**
 * Whether `method` is authorized right now. Fail-closed: undeclared / ungranted /
 * expired / revoked all return `false`, regardless of a syntactically valid request.
 */
export function isAuthorized(
  auth: BridgeAuthority,
  method: BridgeMethod,
  nowMs: number,
): boolean {
  return effectiveMethods(auth, nowMs).has(method);
}

/** The legacy grandfather grant: every NON-sensitive dispatch method, no expiry. */
export const GRANDFATHER_GRANT: Grant = {
  methods: GRANDFATHER_METHODS,
  expiresAtMs: null,
  revoked: false,
};

/**
 * Grandfather authority for a legacy app (no `bridge` declaration): the release
 * layer is absent (`declared = null`) and the grant is the full dispatch set, so a
 * pre-existing app keeps reaching the methods it always could. The intersection is
 * STILL evaluated (never bypassed) — it resolves to "allowed for the non-sensitive
 * verbs" (the sensitive class needs a declaration AND consent).
 */
export const GRANDFATHER_AUTHORITY: BridgeAuthority = {
  declared: null,
  grant: GRANDFATHER_GRANT,
  policy: null,
};

/**
 * Build the authority for an app from its release-declared bridge methods.
 *  - `declaredMethods = null` (no `bridge` key, or manifest unavailable) ->
 *    grandfathered legacy (permissive over the dispatch set).
 *  - `declaredMethods = [...]` -> FAIL-CLOSED on that set. The v0 load-time grant
 *    equals the declared set (a direct-load app's grant = what its release
 *    declares), no expiry, not revoked. Off-allowlist names are dropped (they can
 *    never be dispatched anyway). Later phases attenuate via real install grants +
 *    local policy; a runtime revokes by flipping the grant.
 */
export function resolveDeclaredAuthority(
  declaredMethods: readonly string[] | null,
  consentedSensitive = false,
): BridgeAuthority {
  if (declaredMethods === null) return GRANDFATHER_AUTHORITY;
  const declared = new Set<BridgeMethod>();
  for (const m of declaredMethods) {
    if (ALL_DISPATCH_METHODS.has(m as BridgeMethod)) declared.add(m as BridgeMethod);
  }
  // The grant only carries a declared sensitive verb once the user consented.
  const granted = new Set(
    [...declared].filter((m) => consentedSensitive || !SENSITIVE_METHODS.has(m)),
  );
  return {
    declared,
    grant: { methods: granted, expiresAtMs: null, revoked: false },
    policy: null,
  };
}

/** Sensitive verbs a manifest declares (drives the consent prompt). */
export function declaredSensitiveMethods(manifest: unknown): BridgeMethod[] {
  if (manifest === null || typeof manifest !== "object") return [];
  const bridge = (manifest as { bridge?: unknown }).bridge;
  if (bridge === null || typeof bridge !== "object") return [];
  const raw = (bridge as { methods?: unknown }).methods;
  if (!Array.isArray(raw)) return [];
  return raw.filter(
    (m): m is BridgeMethod => typeof m === "string" && SENSITIVE_METHODS.has(m as BridgeMethod),
  );
}

/**
 * Resolve the bridge authority from a PARSED app manifest (SBFB.json). No `bridge`
 * key (or a non-object manifest) -> grandfathered legacy; a present `bridge` key
 * (even without a `methods` array) -> fail-closed on the declared set (possibly
 * empty). Off-allowlist names are dropped. Pure (no I/O), so the branch logic is
 * unit-tested directly; the fetch + graceful fallback wrapper lives in the shell.
 */
export function resolveAuthorityFromManifest(
  manifest: unknown,
  consentedSensitive = false,
): BridgeAuthority {
  if (manifest === null || typeof manifest !== "object") return GRANDFATHER_AUTHORITY;
  const bridge = (manifest as { bridge?: unknown }).bridge;
  // A missing OR explicitly `null` `bridge` -> grandfather. `null` is read as ABSENT
  // (matches Rust `SbfbManifest.bridge: Option<BridgeConfig>`, where JSON null => None):
  // a null value declares nothing, so treating it as "key present => fail-closed" would
  // brick on a serialization artefact. A present OBJECT (even `{}`) is fail-closed.
  if (bridge === undefined || bridge === null) return GRANDFATHER_AUTHORITY;
  const raw =
    typeof bridge === "object" ? (bridge as { methods?: unknown }).methods : undefined;
  const methods = Array.isArray(raw)
    ? raw.filter((m): m is string => typeof m === "string")
    : [];
  return resolveDeclaredAuthority(methods, consentedSensitive);
}
