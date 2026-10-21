// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Chat v1 — relay the daemon's chat SSE stream to the app iframe as
 * `chat_event` push events (ADR_CHAT_V1, plan Phase D).
 *
 * Uses `fetch` streaming through `authFetch` (never `EventSource`, which
 * cannot carry the loopback auth header). Reconnects with capped backoff.
 * Runs only while the app holds the consented chat capability.
 */

import { useEffect, useRef } from "react";

import { authFetch } from "@/api/auth";
import { CHAT_EVENT } from "@/bridge/protocol";

const RECONNECT_MIN_MS = 1_000;
const RECONNECT_MAX_MS = 30_000;

/**
 * Split buffered SSE text into complete `data:` payloads; returns the
 * unconsumed remainder. Pure (unit-tested).
 */
export function parseSse(buffer: string): { data: string[]; rest: string } {
  const normalized = buffer.replace(/\r\n/g, "\n");
  const data: string[] = [];
  let rest = normalized;
  let cut = rest.indexOf("\n\n");
  while (cut >= 0) {
    const frame = rest.slice(0, cut);
    rest = rest.slice(cut + 2);
    const lines = frame
      .split("\n")
      .filter((l) => l.startsWith("data:"))
      .map((l) => l.slice(5).replace(/^ /, ""));
    if (lines.length > 0) data.push(lines.join("\n"));
    cut = rest.indexOf("\n\n");
  }
  return { data, rest };
}

export function useChatStream(
  coordUrl: string | null,
  chatApp: string | null,
  enabled: boolean,
  pushEvent: (name: string, payload: unknown) => void,
): void {
  const pushRef = useRef(pushEvent);
  useEffect(() => {
    pushRef.current = pushEvent;
  });

  useEffect(() => {
    if (!enabled || !coordUrl || !chatApp) return;
    const controller = new AbortController();
    let stopped = false;

    void (async () => {
      let delay = RECONNECT_MIN_MS;
      while (!stopped) {
        try {
          const resp = await authFetch(
            `${coordUrl}/app/${encodeURIComponent(chatApp)}/chat/stream`,
            { signal: controller.signal, headers: { accept: "text/event-stream" } },
          );
          if (!resp.ok || !resp.body) throw new Error(`chat stream ${resp.status}`);
          delay = RECONNECT_MIN_MS;
          const reader = resp.body.pipeThrough(new TextDecoderStream()).getReader();
          let buffer = "";
          for (;;) {
            const { value, done } = await reader.read();
            if (done) break;
            buffer += value;
            const parsed = parseSse(buffer);
            buffer = parsed.rest;
            for (const raw of parsed.data) {
              try {
                pushRef.current(CHAT_EVENT, JSON.parse(raw));
              } catch {
                // malformed frame: skip
              }
            }
          }
        } catch {
          if (stopped) return;
        }
        await new Promise((r) => setTimeout(r, delay));
        delay = Math.min(delay * 2, RECONNECT_MAX_MS);
      }
    })();

    return () => {
      stopped = true;
      controller.abort();
    };
  }, [coordUrl, chatApp, enabled]);
}
