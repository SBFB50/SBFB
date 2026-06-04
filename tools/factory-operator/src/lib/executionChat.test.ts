// SPDX-License-Identifier: AGPL-3.0-or-later
//
// P2-OPERATOR-NO-TEST-RUNNER + P2-OLLAMA-MODEL-PICKER (S73 Phase B): unit
// coverage for the execution-chat client. Proves the model-picker payload
// (a non-Claude intention carries the selected model, an empty model lets the
// server resolve the per-provider default — never the Claude id), and the
// session / SSE wiring.

import { beforeEach, describe, expect, it, vi } from "vitest";

import { createSession, openStream, sendMessage } from "@/lib/executionChat";
import { MockEventSource } from "@/test/setup";

function mockFetchJson(body: unknown) {
  const spy = vi.fn(async () => ({
    ok: true,
    status: 200,
    json: async () => body,
    text: async () => JSON.stringify(body),
  }));
  vi.stubGlobal("fetch", spy);
  return spy;
}

describe("executionChat client", () => {
  beforeEach(() => {
    MockEventSource.reset();
  });

  it("sendMessage carries the selected model for a non-Claude intention", async () => {
    const spy = mockFetchJson({ ok: true });
    await sendMessage("sess-1", "hi", "ollama", "llama3.2:latest");

    expect(spy).toHaveBeenCalledOnce();
    const [url, init] = spy.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/chat/sess-1/send");
    expect(init.method).toBe("POST");
    expect(JSON.parse(init.body as string)).toEqual({
      message: "hi",
      provider: "ollama",
      model: "llama3.2:latest",
    });
  });

  it("sendMessage sends an empty model when none is selected (server resolves the default)", async () => {
    const spy = mockFetchJson({ ok: true });
    await sendMessage("sess-1", "hi", "ollama");

    const [, init] = spy.mock.calls[0] as unknown as [string, RequestInit];
    const body = JSON.parse(init.body as string) as { model: string };
    expect(body.model).toBe("");
    // The bug this fixes: Ollama must never be handed the Claude model id.
    expect(body.model).not.toBe("claude-opus-4-8[1m]");
  });

  it("sendMessage trims a whitespace-only model to empty", async () => {
    const spy = mockFetchJson({ ok: true });
    await sendMessage("sess-1", "hi", "network", "   ");

    const [, init] = spy.mock.calls[0] as unknown as [string, RequestInit];
    expect(JSON.parse(init.body as string).model).toBe("");
  });

  it("createSession posts the chosen provider and returns the id", async () => {
    const spy = mockFetchJson({ id: "chat-123" });
    const id = await createSession("network");

    expect(id).toBe("chat-123");
    const [url, init] = spy.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/chat/session");
    expect(JSON.parse(init.body as string)).toEqual({ provider: "network" });
  });

  it("openStream opens exactly one EventSource on the encoded session endpoint", () => {
    const es = openStream("chat 1/2") as unknown as MockEventSource;
    expect(MockEventSource.instances).toHaveLength(1);
    expect(es.url).toBe("/api/chat/chat%201%2F2/stream");
  });
});
