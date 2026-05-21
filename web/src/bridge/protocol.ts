// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — postMessage bridge protocol.
 * Sprint 15 Phase A — adds host → iframe push events.
 *
 * Zod schemas for the typed messages exchanged between sandboxed
 * iframe apps and the host shell via `window.postMessage`.
 *
 * Direction:
 *   iframe → host : BridgeRequest
 *   host → iframe : BridgeResponse (reply) or BridgeEvent (push, Sprint 15)
 */

import { z } from "zod";

// =================================================================
// Request (iframe → host)
// =================================================================

export const BridgeMethodSchema = z.enum([
  "task_submit",
  "storage_get",
  "storage_set",
  // Sprint 21 Phase B — client-side PII redaction. Dispatched
  // locally in the host shell (no coordinator round-trip) so apps
  // can redact prompts before calling `task_submit`.
  "pii_redact",
  // Sprint 56 Phase C — bridge extensions for pre-v1.0 apps.
  "storage_list",
  "storage_delete",
  "identity_pubkey",
  "node_status",
  "browse_list",
  // Sprint 58 Phase D — storage version polling for live updates.
  "storage_version",
  // Sprint 63 Phase C — verification bridge methods.
  "provenance_get",
  "provenance_verify",
  "feed_cursor_get",
  // Sprint 67 Phase B — FTS5 full-text search.
  "search",
  // Sprint 68 Phase A — ProofCard evidence score.
  "proof_card_get",
]);

export type BridgeMethod = z.infer<typeof BridgeMethodSchema>;

export const BridgeRequestSchema = z.object({
  type: z.literal("sbfb-bridge-request"),
  id: z.string().uuid(),
  method: BridgeMethodSchema,
  payload: z.record(z.unknown()),
});

export type BridgeRequest = z.infer<typeof BridgeRequestSchema>;

// =================================================================
// Method payload schemas — Sprint 21 Phase B
// =================================================================

/** Hard cap on text size submitted for redaction (50 KB). */
export const PII_REDACT_MAX_TEXT_LENGTH = 50_000;

/**
 * `pii_redact` payload. The `policy` field is an optional partial
 * override of the host-shell {@link DEFAULT_POLICY}; missing fields
 * fall back to the default so apps only need to specify diffs.
 */
export const PiiRedactPayloadSchema = z.object({
  text: z.string().max(PII_REDACT_MAX_TEXT_LENGTH),
  policy: z
    .object({
      enabled: z.boolean().optional(),
      entities: z.array(z.string()).optional(),
      replacement: z.string().optional(),
      confidence_threshold: z.number().min(0).max(1).optional(),
      use_model: z.boolean().optional(),
    })
    .optional(),
});

export type PiiRedactPayload = z.infer<typeof PiiRedactPayloadSchema>;

// =================================================================
// Response (host → iframe)
// =================================================================

export const BridgeResponseSchema = z.object({
  type: z.literal("sbfb-bridge-response"),
  id: z.string().uuid(),
  success: z.boolean(),
  data: z.unknown().optional(),
  error: z.string().optional(),
});

export type BridgeResponse = z.infer<typeof BridgeResponseSchema>;

// =================================================================
// Helpers
// =================================================================

export function createResponse(
  id: string,
  data: unknown,
): BridgeResponse {
  return {
    type: "sbfb-bridge-response",
    id,
    success: true,
    data,
  };
}

export function createErrorResponse(
  id: string,
  error: string,
): BridgeResponse {
  return {
    type: "sbfb-bridge-response",
    id,
    success: false,
    error,
  };
}

// =================================================================
// Event (host → iframe, push, Sprint 15 Phase A)
// =================================================================

/**
 * Fire-and-forget event pushed by the host toward a sandboxed iframe.
 *
 * Unlike {@link BridgeResponse}, events are not tied to any prior
 * request — the host decides when to push them (e.g. "task result
 * ready", "storage changed"). The iframe subscribes via
 * `bridge.onEvent(name, callback)`. No acknowledgement is expected.
 *
 * The `name` field is a free-form identifier (≤ 64 chars) so
 * individual apps can filter events client-side. The host does not
 * maintain a whitelist — apps only react to events they explicitly
 * subscribe to.
 */
export const BridgeEventSchema = z.object({
  type: z.literal("sbfb-bridge-event"),
  name: z.string().min(1).max(64),
  payload: z.unknown(),
});

export type BridgeEvent = z.infer<typeof BridgeEventSchema>;

/** Build a push event for a given name + payload. */
export function createEvent(name: string, payload: unknown): BridgeEvent {
  return {
    type: "sbfb-bridge-event",
    name,
    payload,
  };
}

// =================================================================
// Heartbeat (iframe → host, liveness ping, Sprint 15 Phase B)
// =================================================================

/**
 * Liveness ping emitted by the iframe bridge SDK once per second by
 * default. The host {@link useBridge} hook uses the delta between
 * "now" and the last received heartbeat to decide if the iframe is
 * healthy, stalled, or still starting up.
 *
 * Unlike {@link BridgeRequest}, heartbeats carry no correlation ID
 * — they're fire-and-forget periodic signals. The host does not
 * reply.
 */
export const BridgeHeartbeatSchema = z.object({
  type: z.literal("sbfb-bridge-heartbeat"),
  ts: z.number().positive(),
});

export type BridgeHeartbeat = z.infer<typeof BridgeHeartbeatSchema>;
