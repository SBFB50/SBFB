// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — host→app bridge egress guards.
 *
 * Sprint 15's generic {@link import("./useBridge").UseBridgeHandle.pushEvent}
 * carries a `payload: unknown` (fire-and-forget notifications, filtered
 * app-side). That untyped egress is fine for notifications but MUST NOT be a
 * smuggling channel for the operator's accessibility `needs[]` (the CAUSE —
 * health/disability data). Two guards defend the cardinal invariant at the
 * single host egress:
 *
 * 1. {@link isReservedEvent} — a reserved event (e.g. the accessibility
 *    profile) may be emitted ONLY through its dedicated typed sender
 *    (`useBridge.pushAccessibilityProfile`); the generic `pushEvent` refuses
 *    it outright. Backed by a `Set` so an inherited property name
 *    (`toString`, `constructor`, `__proto__`) is never mistaken for a
 *    reserved event.
 * 2. {@link payloadCarriesCause} — a NAME-INDEPENDENT, BEST-EFFORT guard: an
 *    outgoing payload carrying an accidental `needs` KEY (at ANY depth, incl.
 *    Map/Set entries, cycle-safe) is refused, so spreading host state that
 *    includes the cause is dropped under any event name — not only the reserved
 *    one. Defense-in-depth against a developer ACCIDENT, NOT a comprehensive
 *    cause validator (see its doc for the scoped residual). The BY-CONSTRUCTION
 *    guarantee for the accessibility projection is its TYPE (no `needs` field),
 *    emitted only via the dedicated sender.
 */
import { A11Y_PROFILE_EVENT } from "@/a11y/profile";

/**
 * Reserved host→app event names — emitted only via a dedicated typed sender,
 * never through the generic `pushEvent`. A `Set` (not a plain object) so
 * `has("toString")` / `has("constructor")` are `false`, never an inherited
 * `Object.prototype` member.
 */
export const RESERVED_EVENT_NAMES: ReadonlySet<string> = new Set([A11Y_PROFILE_EVENT]);

/** Whether `name` is a reserved host→app event (refused on the generic channel). */
export function isReservedEvent(name: string): boolean {
  return RESERVED_EVENT_NAMES.has(name);
}

/**
 * Walk the FULL enumerable object graph of an outgoing payload for a `needs`
 * own-key — plain-object keys, array indices AND non-index own properties, Map
 * keys AND values, and Set entries, at ANY depth. A `seen` set makes cycles safe
 * (no depth cap, so a deeply nested accident cannot slip past).
 */
function payloadHasNeedsKey(value: unknown, seen: Set<object>): boolean {
  if (value === null || typeof value !== "object") return false;
  if (seen.has(value as object)) return false;
  seen.add(value as object);
  if (value instanceof Map) {
    // A `needs` may hide as a string key, a KEY-OBJECT, or a value.
    for (const [k, v] of value) {
      if (k === "needs") return true;
      if (payloadHasNeedsKey(k, seen) || payloadHasNeedsKey(v, seen)) return true;
    }
  } else if (value instanceof Set) {
    for (const entry of value) if (payloadHasNeedsKey(entry, seen)) return true;
  }
  // Own enumerable string-keyed properties — covers plain-object keys AND, for an
  // Array, BOTH the numeric indices AND any non-index own property (e.g.
  // `Object.assign([], { needs })`). `structuredClone` only clones these own
  // enumerable props, so the scan inspects exactly what `postMessage` would send.
  for (const [key, entry] of Object.entries(value as Record<string, unknown>)) {
    if (key === "needs") return true;
    if (payloadHasNeedsKey(entry, seen)) return true;
  }
  return false;
}

/**
 * NAME-INDEPENDENT, BEST-EFFORT cause-free egress guard. Rejects an outgoing
 * payload that carries a `needs` KEY anywhere in its enumerable object graph
 * (any depth: plain-object keys, array indices + non-index own props, Map keys +
 * values, Set entries, cycle-safe), so spreading host state that includes the
 * cause is dropped under any event name — not only the reserved one.
 *
 * SCOPED RESIDUAL (honest): this catches the realistic ACCIDENT — a developer
 * forwarding state that carries a `needs` key. It does NOT catch a DELIBERATE
 * encoding of the 13 need VALUES under an arbitrary key, as a bare array payload,
 * or in the event NAME — nor a getter that mutates between this scan and the
 * structured clone: value-scanning the tokens (`motor`, `attention`, …) would
 * false-positive on legitimate notifications, and the host that calls
 * `pushEvent` is trusted (it already holds `needs`). The BY-CONSTRUCTION
 * guarantee for the accessibility projection is its TYPE (no `needs` field),
 * emitted only via `useBridge.pushAccessibilityProfile`; comprehensive typed
 * egress authority for the generic channel is Sprint 83 Phase C (bridge authority).
 */
export function payloadCarriesCause(payload: unknown): boolean {
  return payloadHasNeedsKey(payload, new Set());
}
