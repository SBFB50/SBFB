// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — bridge protocol schema tests.
 * Sprint 15 Phase A — adds BridgeEventSchema coverage.
 */

import { describe, expect, it } from "vitest";
import {
  BridgeEventSchema,
  BridgeRequestSchema,
  BridgeResponseSchema,
  PII_REDACT_MAX_TEXT_LENGTH,
  PiiRedactPayloadSchema,
  createErrorResponse,
  createEvent,
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

describe("BridgeEventSchema (Sprint 15 Phase A)", () => {
  it("accepts a valid event with payload", () => {
    const evt = createEvent("task_result_ready", { task_id: "t-1", result: "ok" });
    expect(BridgeEventSchema.safeParse(evt).success).toBe(true);
    expect(evt.type).toBe("sbfb-bridge-event");
    expect(evt.name).toBe("task_result_ready");
  });

  it("accepts an event with null payload", () => {
    expect(BridgeEventSchema.safeParse(createEvent("ping", null)).success).toBe(true);
  });

  it("rejects an event with empty name", () => {
    const evt = { type: "sbfb-bridge-event", name: "", payload: null };
    expect(BridgeEventSchema.safeParse(evt).success).toBe(false);
  });

  it("rejects an event with name longer than 64 chars", () => {
    const evt = { type: "sbfb-bridge-event", name: "x".repeat(65), payload: null };
    expect(BridgeEventSchema.safeParse(evt).success).toBe(false);
  });

  it("rejects an event with wrong type discriminator", () => {
    const evt = { type: "sbfb-bridge-response", name: "foo", payload: null };
    expect(BridgeEventSchema.safeParse(evt).success).toBe(false);
  });
});

describe("PiiRedactPayloadSchema (Sprint 21 Phase B)", () => {
  it("accepts pii_redact as a valid bridge method", () => {
    const req = {
      type: "sbfb-bridge-request",
      id: "550e8400-e29b-41d4-a716-446655440000",
      method: "pii_redact",
      payload: { text: "Hello foo@bar.com" },
    };
    expect(BridgeRequestSchema.safeParse(req).success).toBe(true);
  });

  it("rejects a pii_redact payload larger than the hard cap", () => {
    const text = "x".repeat(PII_REDACT_MAX_TEXT_LENGTH + 1);
    expect(PiiRedactPayloadSchema.safeParse({ text }).success).toBe(false);
  });

  it("accepts an optional partial policy override", () => {
    const payload = {
      text: "sample",
      policy: { enabled: false, confidence_threshold: 0.2 },
    };
    expect(PiiRedactPayloadSchema.safeParse(payload).success).toBe(true);
  });
});
