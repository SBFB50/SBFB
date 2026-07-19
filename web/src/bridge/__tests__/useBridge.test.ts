// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — useBridge hook tests.
 *
 * Tests the host-side bridge listener by firing synthetic
 * postMessage events and verifying the dispatched responses.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";

import { useBridge } from "../useBridge";
import { primeAuthToken } from "@/api/auth";
import type { BridgeRequest, BridgeResponse } from "../protocol";
import {
  A11Y_PROFILE_EVENT,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  RESOLVED_PROFILE_AXES,
} from "@/a11y/profile";

function makeRequest(overrides: Partial<BridgeRequest> = {}): BridgeRequest {
  return {
    type: "sbfb-bridge-request",
    id: "550e8400-e29b-41d4-a716-446655440000",
    method: "task_submit",
    payload: { prompt: "test" },
    ...overrides,
  };
}

describe("useBridge", () => {
  let iframeRef: React.RefObject<HTMLIFrameElement | null>;
  let fakeWindow: { postMessage: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    fakeWindow = { postMessage: vi.fn() };
    // Create a fake iframe ref whose contentWindow we control.
    iframeRef = {
      current: {
        contentWindow: fakeWindow as unknown as Window,
      } as HTMLIFrameElement,
    };
    primeAuthToken("deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcdef");
  });

  afterEach(() => {
    primeAuthToken(null);
    vi.restoreAllMocks();
  });

  it("ignores messages with wrong type", () => {
    renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

    window.dispatchEvent(
      new MessageEvent("message", {
        data: { type: "not-a-bridge", id: "x" },
        source: fakeWindow as unknown as Window,
      }),
    );

    expect(fakeWindow.postMessage).not.toHaveBeenCalled();
  });

  it("ignores messages from unknown source", () => {
    renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

    const otherWindow = {} as Window;
    window.dispatchEvent(
      new MessageEvent("message", {
        data: makeRequest(),
        source: otherWindow,
      }),
    );

    expect(fakeWindow.postMessage).not.toHaveBeenCalled();
  });

  it("sends error response when no coordinator URL", async () => {
    renderHook(() => useBridge(null, "gov", iframeRef));

    window.dispatchEvent(
      new MessageEvent("message", {
        data: makeRequest(),
        source: fakeWindow as unknown as Window,
      }),
    );

    // Wait for async dispatch.
    await vi.waitFor(() => {
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    });

    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.type).toBe("sbfb-bridge-response");
    expect(resp.success).toBe(false);
    expect(resp.error).toContain("no active coordinator");
  });

  describe("pii_redact dispatch (Sprint 21 Phase B)", () => {
    it("redacts without requiring a coordinator URL", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch");
      renderHook(() => useBridge(null, null, iframeRef));

      // `use_model: false` forces the regex fallback path — jsdom
      // has no WASM runtime, so the real ONNX loader would hang
      // otherwise. The model path is covered by the wrapper tests
      // that inject a stub ModelLoader.
      const req = makeRequest({
        id: "11111111-1111-4111-8111-111111111111",
        method: "pii_redact",
        payload: { text: "Email foo@bar.com", policy: { use_model: false } },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { redacted_text: string; findings_count: number };
      expect(data.redacted_text).toContain("[REDACTED:EMAIL_ADDRESS]");
      expect(data.redacted_text).not.toContain("foo@bar.com");
      expect(data.findings_count).toBeGreaterThan(0);

      // Local dispatch: the coord was never called.
      expect(fetchSpy).not.toHaveBeenCalled();
    });
  });

  describe("bridge extensions Sprint 56 Phase C", () => {
    it("dispatches storage_list via fetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ entries: [], count: 0 }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "22222222-2222-4222-8222-222222222222",
        method: "storage_list",
        payload: { prefix: "user:" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/app/gov/state?prefix=user%3A"),
        expect.any(Object),
      );
    });

    it("dispatches storage_delete via DELETE fetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ ok: true }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "33333333-3333-4333-8333-333333333333",
        method: "storage_delete",
        payload: { key: "temp" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/app/gov/state/temp"),
        expect.objectContaining({ method: "DELETE" }),
      );
    });

    it("dispatches identity_pubkey via daemon info", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ node_id: "abc123def456" }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "44444444-4444-4444-8444-444444444444",
        method: "identity_pubkey",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect((resp.data as { pubkey: string }).pubkey).toBe("abc123def456");
    });

    it("dispatches node_status with peers enrichment", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch")
        .mockResolvedValueOnce(
          new Response(
            JSON.stringify({ status: "ok", uptime_secs: 42 }),
            { status: 200 },
          ),
        )
        .mockResolvedValueOnce(
          new Response(
            JSON.stringify({ subscribed_curators: ["aaa", "bbb"] }),
            { status: 200 },
          ),
        );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "55555555-5555-4555-8555-555555555555",
        method: "node_status",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { status: string; peers: number };
      expect(data.status).toBe("ok");
      expect(data.peers).toBe(2);
      expect(fetchSpy).toHaveBeenCalledTimes(2);
    });

    it("dispatches browse_list via daemon browse", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({ entries: [{ project_name: "Hello" }] }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "66666666-6666-4666-8666-666666666666",
        method: "browse_list",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { entries: Array<{ project_name: string }> };
      expect(data.entries).toHaveLength(1);
      expect(data.entries[0].project_name).toBe("Hello");
    });

    it("injects x-sbfb-token header via authFetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ entries: [] }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "77777777-7777-4777-8777-777777777777",
        method: "browse_list",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const [, init] = fetchSpy.mock.calls[0];
      const headers = init?.headers as Headers;
      expect(headers.get("x-sbfb-token")).toBe(
        "deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcdef",
      );
    });
  });

  describe("verification bridge methods Sprint 63 Phase C", () => {
    it("dispatches provenance_get via project provenance endpoint", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({ record: { repo_url: "https://example.com" }, verified: true, provenance_hash: "abc123" }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "aa111111-1111-4111-8111-111111111111",
        method: "provenance_get",
        payload: { project_id: "proj42" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { record: { repo_url: string }; provenance_hash: string };
      expect(data.record.repo_url).toBe("https://example.com");
      expect(data.provenance_hash).toBe("abc123");
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/api/v1/project/proj42/provenance"),
        expect.any(Object),
      );
    });

    it("dispatches provenance_verify with verified field and hash", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({ record: { repo_url: "https://example.com" }, verified: true, provenance_hash: "abc123" }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "bb222222-2222-4222-8222-222222222222",
        method: "provenance_verify",
        payload: { project_id: "proj42" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { verified: boolean; record: { repo_url: string }; provenance_hash: string };
      expect(data.verified).toBe(true);
      expect(data.record.repo_url).toBe("https://example.com");
      expect(data.provenance_hash).toBe("abc123");
    });

    it("dispatches feed_cursor_get via daemon feed cursor endpoint", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({ last_seq: 99, last_entry_hash: "deadbeef" }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "cc333333-3333-4333-8333-333333333333",
        method: "feed_cursor_get",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { last_seq: number; last_entry_hash: string };
      expect(data.last_seq).toBe(99);
      expect(data.last_entry_hash).toBe("deadbeef");
    });
  });

  describe("proof_card_get Sprint 68 Phase A", () => {
    it("dispatches proof_card_get via daemon proof-card endpoint", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            project_id: "f".repeat(64),
            project_name: "test-app",
            confidence: 35,
            formula_version: 1,
          }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "dd444444-4444-4444-8444-444444444444",
        method: "proof_card_get",
        payload: { project_id: "f".repeat(64) },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { project_name: string; confidence: number; formula_version: number };
      expect(data.project_name).toBe("test-app");
      expect(data.confidence).toBe(35);
      expect(data.formula_version).toBe(1);
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/api/daemon/proof-card/"),
        expect.any(Object),
      );
    });
  });

  describe("compute bridge Sprint 76 Phase H", () => {
    it("task_submit injects the local project_id and returns the task id", async () => {
      const fetchSpy = vi
        .spyOn(globalThis, "fetch")
        // 1. host resolves the local project doc id
        .mockResolvedValueOnce(
          new Response(JSON.stringify({ project_doc_id: "doc-abc" }), {
            status: 200,
          }),
        )
        // 2. daemon-level submit returns the full signed TaskEntry
        .mockResolvedValueOnce(
          new Response(JSON.stringify({ task: { task_id: "t-1" } }), {
            status: 200,
          }),
        );
      renderHook(() =>
        useBridge("http://localhost:8000", "compute-tester", iframeRef),
      );

      const req = makeRequest({
        id: "77777777-7777-4777-8777-777777777777",
        method: "task_submit",
        payload: { prompt: "ping", model: "llama3.1:8b" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect((resp.data as { task_id: string }).task_id).toBe("t-1");
      // project-info resolved first, then the submit POST.
      expect(fetchSpy).toHaveBeenNthCalledWith(
        1,
        expect.stringContaining("/api/daemon/project-info"),
        expect.any(Object),
      );
      // The host injected the local project_id into the submit body so
      // the node's own worker (whitelisted to it) claims the task.
      const submitInit = fetchSpy.mock.calls[1][1] as RequestInit;
      const submitBody = JSON.parse(String(submitInit.body));
      expect(submitBody.project_id).toBe("doc-abc");
      expect(submitBody.prompt).toBe("ping");
      expect(submitBody.task_type).toBe("inference");
    });

    it("task_submit errors when the node has no project doc", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ project_doc_id: null }), { status: 200 }),
      );
      renderHook(() =>
        useBridge("http://localhost:8000", "compute-tester", iframeRef),
      );

      const req = makeRequest({
        id: "88888888-8888-4888-8888-888888888888",
        method: "task_submit",
        payload: { prompt: "ping", model: "llama3.1:8b" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(false);
      expect(resp.error).toContain("no local project doc");
    });

    it("task_result maps a 404 to a pending poll", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ error: "no result yet" }), {
          status: 404,
        }),
      );
      renderHook(() =>
        useBridge("http://localhost:8000", "compute-tester", iframeRef),
      );

      const req = makeRequest({
        id: "99999999-9999-4999-8999-999999999999",
        method: "task_result",
        payload: { task_id: "t-1" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect(resp.data).toEqual({ ready: false, status: "pending" });
    });

    it("task_result surfaces the completed result text", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            task_id: "t-1",
            status: "completed",
            result_text: "pong",
            result_hash: "deadbeef",
          }),
          { status: 200 },
        ),
      );
      renderHook(() =>
        useBridge("http://localhost:8000", "compute-tester", iframeRef),
      );

      const req = makeRequest({
        id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        method: "task_result",
        payload: { task_id: "t-1" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { ready: boolean; result_text: string };
      expect(data.ready).toBe(true);
      expect(data.result_text).toBe("pong");
    });
  });

  describe("pushEvent (Sprint 15 Phase A)", () => {
    it("posts a bridge-event to the iframe contentWindow", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );

      result.current.pushEvent("task_result_ready", { task_id: "t-42" });

      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      const [msg, targetOrigin] = fakeWindow.postMessage.mock.calls[0];
      expect(msg).toEqual({
        type: "sbfb-bridge-event",
        name: "task_result_ready",
        payload: { task_id: "t-42" },
      });
      expect(targetOrigin).toBe("*");
    });

    it("is a no-op when iframeRef.current is null", () => {
      const nullRef = { current: null } as React.RefObject<HTMLIFrameElement | null>;
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", nullRef),
      );

      expect(() => result.current.pushEvent("foo", {})).not.toThrow();
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("is a no-op when contentWindow is null", () => {
      const noWindowRef = {
        current: { contentWindow: null } as unknown as HTMLIFrameElement,
      } as React.RefObject<HTMLIFrameElement | null>;
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", noWindowRef),
      );

      expect(() => result.current.pushEvent("foo", null)).not.toThrow();
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("allows arbitrary payload types", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );

      result.current.pushEvent("string_payload", "hello");
      result.current.pushEvent("number_payload", 42);
      result.current.pushEvent("array_payload", [1, 2, 3]);

      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(3);
      expect(fakeWindow.postMessage.mock.calls[0][0].payload).toBe("hello");
      expect(fakeWindow.postMessage.mock.calls[1][0].payload).toBe(42);
      expect(fakeWindow.postMessage.mock.calls[2][0].payload).toEqual([1, 2, 3]);
    });
  });

  describe("reserved event egress guard (Sprint 83 Phase B)", () => {
    it("refuses the reserved a11y-profile event on the generic channel (any payload)", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // The reserved event is emitted ONLY through pushAccessibilityProfile;
      // the generic channel refuses it outright, valid projection or not.
      result.current.pushEvent(A11Y_PROFILE_EVENT, { theme: "dark" });
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("DROPS any event whose payload carries needs — regardless of the event name", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // Name-INDEPENDENT guard: a cause smuggled under a FREE-FORM name, at top
      // level or nested, is still dropped (Codex S83-B loop-7 P1).
      result.current.pushEvent("legacy-a11y", { needs: ["motor"] });
      result.current.pushEvent("some-notification", { nested: { needs: ["lowVision"] } });
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("drops a needs KEY beyond depth 6 and inside a Map (any-depth, Map-aware)", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // Deeper than the old depth-6 cap — the strengthened guard walks the full
      // graph (cycle-safe), so a deeply nested accident cannot slip past.
      const deep = { a: { b: { c: { d: { e: { f: { g: { needs: ["motor"] } } } } } } } };
      result.current.pushEvent("deep", deep);
      // A needs key carried by a Map entry (postMessage would structured-clone it).
      result.current.pushEvent("mapped", new Map([["needs", ["lowVision"]]]));
      // A non-index OWN property on an array (Codex S83-B final GAP).
      result.current.pushEvent("arrprop", Object.assign([], { needs: ["motor"] }));
      // A `needs` KEY-OBJECT inside a Map key (Codex S83-B final GAP).
      result.current.pushEvent("mapkey", new Map([[{ needs: ["lowVision"] }, 0]]));
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("documents the SCOPE BOUNDARY: the KEY guard does not value-scan need tokens", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // NOT a runtime leak: the host that calls pushEvent is trusted and no
      // production code does this. The name-independent guard scans for a `needs`
      // KEY, NOT for the 13 need VALUES (value-scanning `motor`/`attention`/…
      // would false-positive on legitimate notifications). A bare token array
      // under a free-form name is therefore forwarded — the deliberate
      // value/name-encoding residual is ACCEPTED in Sprint 83 Phase C (the host is
      // trusted). This test pins that accepted boundary.
      result.current.pushEvent("tags", ["motor", "electric"]);
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    });

    it("closes the getter-TOCTOU leg: clones ONCE so a flipping getter cannot smuggle the cause (Sprint 83 Phase C)", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // A racy getter returning benign on the FIRST read and the cause on a LATER
      // read. Before Phase C the guard read it once (benign -> passed) and
      // postMessage read it AGAIN (cause -> leaked). Now pushEvent clones ONCE
      // (structuredClone runs the getter exactly once) and scans AND sends that same
      // frozen snapshot, so what the scan approved is exactly what the wire carries.
      let reads = 0;
      const flipping = {
        get data() {
          reads += 1;
          return reads === 1 ? { benign: true } : { needs: ["motor"] };
        },
      };
      result.current.pushEvent("toctou", flipping);
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      const sent = fakeWindow.postMessage.mock.calls[0][0];
      // DISCRIMINATOR (red-before/green-after): on the OLD scan-then-postMessage-the-
      // ORIGINAL code, `sent` is a live ref whose getter yields the cause when
      // serialized -> "needs" appears. With clone-once-scan-send, `sent` is the frozen
      // benign snapshot -> no "needs" ever reaches the wire.
      expect(JSON.stringify(sent)).not.toContain("needs");
      // Secondary signal: the getter ran exactly once (at clone); the scan then reads
      // the plain snapshot, not the accessor.
      expect(reads).toBe(1);
    });

    it("still sends a free-form event whose payload carries no cause", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      result.current.pushEvent("task_result_ready", { task_id: "t-42" });
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      expect(fakeWindow.postMessage.mock.calls[0][0].name).toBe("task_result_ready");
    });

    it("does not treat inherited Object.prototype names as reserved (no throw)", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );
      // "toString"/"constructor" are Object.prototype members — a plain-object
      // registry would mis-handle them (Codex S83-B loop-7 P2). A cause-free
      // payload under such a name is a normal, non-reserved event.
      expect(() => {
        result.current.pushEvent("toString", { task_id: "a" });
        result.current.pushEvent("constructor", { task_id: "b" });
      }).not.toThrow();
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(2);
    });

    it("pushAccessibilityProfile sends a needs-free 15-axis projection", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );

      result.current.pushAccessibilityProfile({
        needs: ["motor", "lowVision"],
        theme: "auto",
        contrast: "auto",
        colorVision: "standard",
        pointer: "standard",
        textSpacing: "standard",
        font: "standard",
        motion: "system",
        transparency: "standard",
        density: "standard",
        reading: "standard",
        focus: "standard",
        scale: "100",
        shortcuts: "off",
        assistiveTech: "standard",
        captions: "off",
      });

      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      const [msg, targetOrigin] = fakeWindow.postMessage.mock.calls[0];
      expect(msg.name).toBe(A11Y_PROFILE_EVENT);
      expect(targetOrigin).toBe("*");
      // The cause NEVER crosses: no `needs` key, and the payload keys are
      // EXACTLY the 15 resolved effect axes — nothing else rides along.
      expect(msg.payload).not.toHaveProperty("needs");
      expect(Object.keys(msg.payload).sort()).toEqual([...RESOLVED_PROFILE_AXES].sort());
      expect(JSON.stringify(msg.payload)).not.toContain('"needs"');
    });

    it("is a no-op (no throw) when the iframe is unmounted", () => {
      const nullRef = { current: null } as React.RefObject<HTMLIFrameElement | null>;
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", nullRef),
      );

      expect(() =>
        result.current.pushAccessibilityProfile(DEFAULT_ACCESSIBILITY_PREFERENCES),
      ).not.toThrow();
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });
  });
});
