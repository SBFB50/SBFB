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
