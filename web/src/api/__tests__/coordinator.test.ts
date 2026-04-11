/**
 * Sprint 8 Phase A — unit tests for the new Sprint 8 helpers in
 * `src/api/coordinator.ts`:
 *
 * - ``submitAppTask`` — D1 task submission wrapper
 * - ``listAppCommands`` — D2 command palette fetch
 * - ``invokeAppCommand`` — D2 command invocation
 * - ``getAppTabDescriptor`` — simplified two-state result after
 *   D4 removal of the legacy fallback
 * - ``CommandDescriptorSchema`` — Zod mirror of the Python
 *   ``CommandDescriptor``
 *
 * Every test stubs ``globalThis.fetch`` so we never need a live
 * coordinator.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  CommandDescriptorSchema,
  getAppTabDescriptor,
  invokeAppCommand,
  listAppCommands,
  submitAppTask,
} from "@/api/coordinator";

const COORD = "http://127.0.0.1:8765";

type FetchArgs = [input: RequestInfo | URL, init?: RequestInit];

function jsonResponse(status: number, body: unknown): Response {
  return new Response(typeof body === "string" ? body : JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

/**
 * Narrow an untyped `spy.mock.calls[i]` lookup back to
 * `[RequestInfo|URL, RequestInit|undefined]`. Vitest infers
 * `vi.fn(async () => Response)` as an empty-parameter function
 * because the implementation function takes no named args,
 * even though the stub ultimately gets called with
 * `(input, init)` by fetch. The cast is local, documented, and
 * lets individual test bodies destructure the call args
 * without fighting the tsc tuple-type inference.
 */
function fetchCall(
  spy: ReturnType<typeof vi.fn>,
  index: number,
): FetchArgs {
  const calls = spy.mock.calls as unknown as FetchArgs[];
  const call = calls[index];
  if (call === undefined) {
    throw new Error(`fetch spy had no call at index ${index}`);
  }
  return call;
}

beforeEach(() => {
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

// ---------------------------------------------------------------------------
// CommandDescriptorSchema
// ---------------------------------------------------------------------------

describe("CommandDescriptorSchema", () => {
  it("parses a minimal Python-emitted descriptor", () => {
    const parsed = CommandDescriptorSchema.parse({
      schema_version: 1,
      name: "detect",
      description: "Détecter",
      icon: "sparkles",
      group: "Actions",
    });
    expect(parsed.name).toBe("detect");
    expect(parsed.description).toBe("Détecter");
  });

  it("rejects extra fields (strict)", () => {
    expect(() =>
      CommandDescriptorSchema.parse({
        schema_version: 1,
        name: "x",
        description: "y",
        icon: "sparkles",
        group: "Actions",
        surprise: 42,
      }),
    ).toThrow();
  });

  it("rejects schema_version != 1", () => {
    expect(() =>
      CommandDescriptorSchema.parse({
        schema_version: 2,
        name: "x",
        description: "y",
        icon: "sparkles",
        group: "Actions",
      }),
    ).toThrow();
  });

  it("enforces Python-side length caps", () => {
    expect(() =>
      CommandDescriptorSchema.parse({
        schema_version: 1,
        name: "a".repeat(65),
        description: "y",
        icon: "sparkles",
        group: "Actions",
      }),
    ).toThrow();
    expect(() =>
      CommandDescriptorSchema.parse({
        schema_version: 1,
        name: "x",
        description: "a".repeat(281),
        icon: "sparkles",
        group: "Actions",
      }),
    ).toThrow();
  });
});

// ---------------------------------------------------------------------------
// submitAppTask
// ---------------------------------------------------------------------------

describe("submitAppTask", () => {
  it("POSTs the body and returns {task_id}", async () => {
    const spy = vi.fn(async () => jsonResponse(200, { task_id: "task-42" }));
    vi.stubGlobal("fetch", spy);
    const result = await submitAppTask(COORD, "gov", {
      worker: "rag_search",
      payload: { query: "hello" },
      priority: 7,
      parent_task_id: "parent-1",
    });
    expect(result.task_id).toBe("task-42");
    expect(spy).toHaveBeenCalledOnce();
    const [url, init] = fetchCall(spy, 0);
    expect(String(url)).toBe(`${COORD}/app/gov/tasks/submit`);
    expect(init?.method).toBe("POST");
    const body = JSON.parse(init?.body as string);
    expect(body.worker).toBe("rag_search");
    expect(body.payload).toEqual({ query: "hello" });
    expect(body.priority).toBe(7);
    expect(body.parent_task_id).toBe("parent-1");
  });

  it("URL-encodes the app name in the path", async () => {
    const spy = vi.fn(async () => jsonResponse(200, { task_id: "t" }));
    vi.stubGlobal("fetch", spy);
    await submitAppTask(COORD, "weird name", {
      worker: "w",
      payload: {},
      priority: 5,
      parent_task_id: null,
    });
    const [url] = fetchCall(spy, 0);
    expect(String(url)).toBe(`${COORD}/app/weird%20name/tasks/submit`);
  });

  it("throws CoordinatorHttpError on HTTP 422 from the worker resolver", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => jsonResponse(422, { detail: "worker 'ghost' not found" })),
    );
    await expect(
      submitAppTask(COORD, "gov", {
        worker: "ghost",
        payload: {},
        priority: 5,
        parent_task_id: null,
      }),
    ).rejects.toThrow(/422/);
  });
});

// ---------------------------------------------------------------------------
// listAppCommands + invokeAppCommand
// ---------------------------------------------------------------------------

describe("listAppCommands", () => {
  it("parses an array of command descriptors", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        jsonResponse(200, [
          {
            schema_version: 1,
            name: "detect",
            description: "Détecter",
            icon: "sparkles",
            group: "Gov",
          },
          {
            schema_version: 1,
            name: "refresh",
            description: "Rafraîchir",
            icon: "refresh",
            group: "Gov",
          },
        ]),
      ),
    );
    const cmds = await listAppCommands(COORD, "gov");
    expect(cmds).toHaveLength(2);
    expect(cmds[0].name).toBe("detect");
    expect(cmds[1].group).toBe("Gov");
  });
});

describe("invokeAppCommand", () => {
  it("POSTs to the invoke endpoint and returns the result envelope", async () => {
    const spy = vi.fn(async () =>
      jsonResponse(200, {
        result: { navigation: { path: "/app/gov/tabs/contradictions" } },
      }),
    );
    vi.stubGlobal("fetch", spy);
    const r = await invokeAppCommand(COORD, "gov", "detect");
    expect(r.result).toEqual({
      navigation: { path: "/app/gov/tabs/contradictions" },
    });
    const [url, init] = fetchCall(spy, 0);
    expect(String(url)).toBe(`${COORD}/app/gov/commands/detect/invoke`);
    expect(init?.method).toBe("POST");
  });

  it("URL-encodes reserved characters in cmd name", async () => {
    const spy = vi.fn(async () => jsonResponse(200, { result: null }));
    vi.stubGlobal("fetch", spy);
    await invokeAppCommand(COORD, "gov", "has space");
    const [url] = fetchCall(spy, 0);
    expect(String(url)).toBe(`${COORD}/app/gov/commands/has%20space/invoke`);
  });
});

// ---------------------------------------------------------------------------
// getAppTabDescriptor — D4 removal of the legacy branch
// ---------------------------------------------------------------------------

describe("getAppTabDescriptor (Sprint 8 D4)", () => {
  it("returns kind=schema on a valid TabView response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        jsonResponse(200, {
          descriptor: {
            schema_version: 1,
            tab_name: "hello",
            title: null,
            blocks: [
              // TabBlockHeadingSchema is strict — level/text only,
              // no `muted` (that field lives on the text block).
              { kind: "heading", level: 1, text: "Hi" },
            ],
          },
        }),
      ),
    );
    const result = await getAppTabDescriptor(COORD, "hello", "Hello");
    expect(result.kind).toBe("schema");
    if (result.kind !== "schema") throw new Error("unreachable");
    expect(result.tabView.tab_name).toBe("hello");
  });

  it("returns kind=error on HTTP 422 (coordinator rejected the descriptor)", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(
          JSON.stringify({
            detail: "tab 'Old' on app 'legacy' returned an invalid descriptor",
          }),
          {
            status: 422,
            statusText: "Unprocessable Entity",
            headers: { "content-type": "application/json" },
          },
        ),
      ),
    );
    const result = await getAppTabDescriptor(COORD, "legacy", "Old");
    expect(result.kind).toBe("error");
    if (result.kind !== "error") throw new Error("unreachable");
    expect(result.message).toMatch(/422/);
  });

  it("returns kind=error when the descriptor fails Zod TabView", async () => {
    // Coordinator shipped a descriptor that the shell's Zod mirror
    // disagrees with — this used to route to `legacy` but now
    // surfaces as a regular error (Sprint 8 D4 removal).
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        jsonResponse(200, {
          descriptor: { not: "a tabview" },
        }),
      ),
    );
    const result = await getAppTabDescriptor(COORD, "gov", "Dash");
    expect(result.kind).toBe("error");
    if (result.kind !== "error") throw new Error("unreachable");
    expect(result.message).toContain("Zod");
  });
});
