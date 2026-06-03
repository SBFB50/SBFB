// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 72 Phase E — client for the provider-routed execution chat.
//
// The Operator backend (Phase D, `operator_server.rs`) exposes
// `POST /api/chat/session`, `POST /api/chat/{id}/send` (carrying the
// chosen execution target) and `GET /api/chat/{id}/stream` (SSE of
// `StreamChunk`). No front consumer existed: the original EventSource
// chat was removed in `c3f4813` in favour of the Claude Code PTY
// terminal, which bypasses `ExecutionTarget` entirely. This module
// rebuilds the consumer for the three-target router (D4/D5).
//
// All paths are RELATIVE so the Vite dev proxy injects `X-SBFB-Token`
// (`vite.config.ts`): the browser `EventSource` cannot set request
// headers itself, so a same-origin relative URL proxied to `:3001` is
// the only auth-correct way to open the stream.

import { postApi } from "@/hooks/useApi";

/** The execution axis (D5): WHERE the chat inference runs — distinct from
 * the 5-value prompt-adaptation `provider` of `AgentSelector`. Mapped 1:1
 * to `ChatSendRequest.provider` on the wire (`claude`/`ollama`/`network`,
 * the closed set `ExecutionTarget::from_provider` parses). */
export type ExecutionIntent = "claude" | "ollama" | "network";

/** Mirror of the backend `StreamChunk` (`llm_bridge.rs`,
 * `#[serde(tag = "type")]`). `requires_gate` is the single event the SSE
 * handler emits when the last user message is sensitive. */
export type StreamChunk =
  | { type: "delta"; text: string }
  | { type: "thinking"; text: string }
  | { type: "done"; cost_usd: number; duration_ms: number; result: string }
  | { type: "error"; message: string }
  | { type: "debug"; label: string; content: string }
  | { type: "requires_gate"; message: string };

interface SessionResponse {
  id: string;
}

/** The `handle_chat_send` JSON reply: `{ok}` on a queued turn, or
 * `{ok:false, requires_gate:true}` when the message is sensitive. */
export interface SendResult {
  ok?: boolean;
  requires_gate?: boolean;
}

/** Create a chat session bound to the chosen execution target. `project_id`
 * is left to the server default (`operator-chat`); per-project routing is
 * the S74 atelier, out of scope here. The model is left to the server
 * default too — model selection is a separate axis (D5), not Phase E. */
export async function createSession(intent: ExecutionIntent): Promise<string> {
  const res = await postApi<SessionResponse>("/chat/session", {
    provider: intent,
  });
  return res.id;
}

/** Send a turn, carrying the chosen execution target (the load-bearing
 * wire). A sensitive message returns `requires_gate` — the server never
 * spawns an autonomous agent for it, regardless of the target. */
export async function sendMessage(
  sessionId: string,
  message: string,
  intent: ExecutionIntent,
): Promise<SendResult> {
  return postApi<SendResult>(`/chat/${encodeURIComponent(sessionId)}/send`, {
    message,
    provider: intent,
  });
}

/** Open the SSE reply stream. Relative path → proxied with the bearer
 * token. The caller MUST `close()` it on the terminal chunk
 * (`done`/`error`/`requires_gate`) to defuse `EventSource` auto-reconnect,
 * which would otherwise re-run the last turn on the server. */
export function openStream(sessionId: string): EventSource {
  return new EventSource(`/api/chat/${encodeURIComponent(sessionId)}/stream`);
}
