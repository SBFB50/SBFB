// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — postMessage bridge protocol.
 *
 * Zod schemas for the typed messages exchanged between sandboxed
 * iframe apps and the host shell via `window.postMessage`.
 *
 * Direction:
 *   iframe → host : BridgeRequest
 *   host → iframe : BridgeResponse
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
