// SPDX-License-Identifier: AGPL-3.0-or-later
//
// P2-OPERATOR-NO-TEST-RUNNER (S73 Phase B): component coverage for the SSE
// StreamChunk mapping, the requires_gate short-circuit, and the
// no-reconnect-storm contract (the terminal chunk must `close()` the
// EventSource exactly once). Drives the controllable MockEventSource from
// `src/test/setup.ts`.

import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { ExecutionChat } from "@/pages/ExecutionChat";
import { MockEventSource } from "@/test/setup";
// Initialise i18next (sync, inline resources) so `t()` returns real strings.
import "@/i18n";

/** Queue fetch replies: 1st call = createSession, 2nd = sendMessage. */
function mockChatFetch(sendReply: unknown) {
  let call = 0;
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => {
      call += 1;
      const body = call === 1 ? { id: "chat-1" } : sendReply;
      return {
        ok: true,
        status: 200,
        json: async () => body,
        text: async () => JSON.stringify(body),
      };
    }),
  );
}

async function typeAndSend(message: string) {
  const input = await screen.findByLabelText(
    "Décrivez ce que vous voulez faire...",
  );
  fireEvent.change(input, { target: { value: message } });
  fireEvent.click(screen.getByRole("button", { name: "Envoyer" }));
}

describe("ExecutionChat", () => {
  beforeEach(() => {
    MockEventSource.reset();
    localStorage.clear();
  });

  it("maps a done chunk to an assistant message and closes the stream exactly once", async () => {
    mockChatFetch({ ok: true });
    render(<ExecutionChat />);

    await typeAndSend("run something");

    // The stream opens only after createSession + sendMessage resolve.
    await waitFor(() => expect(MockEventSource.instances).toHaveLength(1));
    const es = MockEventSource.instances[0];

    await act(async () => {
      es.emit(
        JSON.stringify({
          type: "done",
          result: "the final answer",
          cost_usd: 0,
          duration_ms: 0,
        }),
      );
    });

    expect(await screen.findByText("the final answer")).toBeInTheDocument();
    // No-reconnect-storm contract: the terminal chunk closes the stream once.
    expect(es.closeCount).toBe(1);
  });

  it("short-circuits a gated turn without opening any stream", async () => {
    mockChatFetch({ ok: false, requires_gate: true });
    render(<ExecutionChat />);

    await typeAndSend("please commit and push");

    // A gated /send reply must NOT open an SSE stream (no autonomous agent).
    await waitFor(() =>
      expect(
        screen.getByText(
          "Cette action nécessite une vérification externe via une vraie session agent.",
        ),
      ).toBeInTheDocument(),
    );
    expect(MockEventSource.instances).toHaveLength(0);
  });
});
