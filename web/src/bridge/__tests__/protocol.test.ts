// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — bridge protocol schema tests.
 */

import { describe, expect, it } from "vitest";
import {
  BridgeRequestSchema,
  BridgeResponseSchema,
  createErrorResponse,
  createResponse,
} from "../protocol";

describe("BridgeRequestSchema", () => {
  it("accepts a valid task_submit request", () => {
    const req = {
      type: "sbfb-bridge-request",
      id: "550e8400-e29b-41d4-a716-446655440000",
      method: "task_submit",
      payload: { prompt: "Hello" },
    };
    expect(BridgeRequestSchema.safeParse(req).success).toBe(true);
  });

  it("rejects a request with wrong type", () => {
    const req = {
      type: "not-a-bridge-request",
      id: "550e8400-e29b-41d4-a716-446655440000",
      method: "task_submit",
      payload: {},
    };
    expect(BridgeRequestSchema.safeParse(req).success).toBe(false);
  });

  it("rejects a request with unknown method", () => {
    const req = {
      type: "sbfb-bridge-request",
      id: "550e8400-e29b-41d4-a716-446655440000",
      method: "hack_the_planet",
      payload: {},
    };
    expect(BridgeRequestSchema.safeParse(req).success).toBe(false);
  });

  it("rejects a request with invalid UUID", () => {
    const req = {
      type: "sbfb-bridge-request",
      id: "not-a-uuid",
      method: "storage_get",
      payload: {},
    };
    expect(BridgeRequestSchema.safeParse(req).success).toBe(false);
  });
});

describe("BridgeResponseSchema", () => {
  it("accepts a valid success response", () => {
    const resp = createResponse("550e8400-e29b-41d4-a716-446655440000", { task_id: "t-1" });
    expect(BridgeResponseSchema.safeParse(resp).success).toBe(true);
    expect(resp.success).toBe(true);
  });

  it("accepts a valid error response", () => {
    const resp = createErrorResponse("550e8400-e29b-41d4-a716-446655440000", "timeout");
    expect(BridgeResponseSchema.safeParse(resp).success).toBe(true);
    expect(resp.success).toBe(false);
    expect(resp.error).toBe("timeout");
  });
});
