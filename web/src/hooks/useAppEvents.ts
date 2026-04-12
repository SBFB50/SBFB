// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `useAppEvents` — open a Server-Sent Events stream against
 * `${coordinatorUrl}/app/${appName}/events?pattern=…` and run a
 * React Query cache invalidation when an envelope's topic
 * matches.
 *
 * Sprint 9 Phase C (D2 consumer side). The mirror of
 * `AppContext.events.subscribe` on the Python side: the bus
 * publishes envelopes per app, the SSE bridge frames them on
 * the wire, and this hook re-projects them as React Query
 * invalidations so the live grids re-fetch without a manual
 * page reload.
 *
 * Behaviour:
 * - Opens a single `EventSource` per `(coordinatorUrl, appName,
 *   pattern)` combination on mount; tears it down on unmount.
 * - Parses each `data:` line through a Zod schema mirroring the
 *   Pydantic `EventEnvelope` (topic, payload, timestamp,
 *   trace_id). Malformed payloads are logged and skipped — they
 *   never crash the consumer.
 * - On every parsed envelope, calls
 *   `queryClient.invalidateQueries({ queryKey })` so the
 *   subscriber's React Query (the tab descriptor in the
 *   canonical case) re-fetches.
 * - Reconnects on `error` events with an exponential backoff
 *   capped at 30 s. The native `EventSource` auto-reconnects on
 *   most browsers but the cap is here so we never hammer the
 *   coordinator.
 */

import { useEffect, useRef } from "react";
import { useQueryClient, type QueryKey } from "@tanstack/react-query";
import { z } from "zod";

export const EventEnvelopeSchema = z.object({
  topic: z.string().min(1),
  payload: z.record(z.string(), z.unknown()),
  timestamp: z.string(),
  trace_id: z.string().min(1),
});

export type EventEnvelope = z.infer<typeof EventEnvelopeSchema>;

export interface UseAppEventsOptions {
  /** Coordinator base URL (the same one React Query uses). */
  coordinatorUrl: string | null;
  /** App name on the coordinator (e.g. `"gov"`). */
  appName: string | null;
  /** fnmatch glob pattern (e.g. `"party.refreshed"`). */
  pattern: string;
  /** Query key to invalidate on every matching envelope. */
  invalidateQueryKey: QueryKey;
  /** Optional callback fired with each parsed envelope. */
  onEvent?: (envelope: EventEnvelope) => void;
  /**
   * Override `EventSource` for tests. Defaults to
   * `globalThis.EventSource`. The override must implement the
   * standard subset (`onmessage`, `onerror`, `close`).
   */
  eventSourceFactory?: (url: string) => EventSource;
}

const _MAX_BACKOFF_MS = 30_000;
const _INITIAL_BACKOFF_MS = 500;

export function useAppEvents(options: UseAppEventsOptions): void {
  const queryClient = useQueryClient();
  const optsRef = useRef(options);
  optsRef.current = options;

  useEffect(() => {
    const { coordinatorUrl, appName, pattern } = options;
    if (!coordinatorUrl || !appName || !pattern) {
      return;
    }
    // Capture into local consts so the inner closures see a
    // narrowed (non-null) type — TypeScript can't track the
    // outer-scope null check across nested function bodies.
    const url: string = coordinatorUrl;
    const app: string = appName;
    const factory =
      options.eventSourceFactory ??
      ((u: string) => new globalThis.EventSource(u));

    let closed = false;
    let backoffMs = _INITIAL_BACKOFF_MS;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let source: EventSource | null = null;

    function buildUrl(): string {
      const base = url.replace(/\/+$/, "");
      const params = new URLSearchParams({ pattern });
      return `${base}/app/${app}/events?${params.toString()}`;
    }

    function connect(): void {
      if (closed) return;
      try {
        source = factory(buildUrl());
      } catch {
        // Browser refused to construct the EventSource (CSP,
        // invalid URL, etc). Backoff and retry.
        scheduleReconnect();
        return;
      }
      source.onmessage = (event: MessageEvent) => {
        backoffMs = _INITIAL_BACKOFF_MS;
        let raw: unknown;
        try {
          raw = JSON.parse(event.data);
        } catch {
          console.warn("useAppEvents: dropped malformed JSON SSE frame");
          return;
        }
        const parsed = EventEnvelopeSchema.safeParse(raw);
        if (!parsed.success) {
          console.warn(
            "useAppEvents: dropped envelope failing schema validation",
            parsed.error.issues,
          );
          return;
        }
        queryClient.invalidateQueries({
          queryKey: optsRef.current.invalidateQueryKey,
        });
        optsRef.current.onEvent?.(parsed.data);
      };
      source.onerror = () => {
        if (closed) return;
        if (source) {
          source.close();
          source = null;
        }
        scheduleReconnect();
      };
    }

    function scheduleReconnect(): void {
      if (closed) return;
      const wait = backoffMs;
      backoffMs = Math.min(backoffMs * 2, _MAX_BACKOFF_MS);
      reconnectTimer = setTimeout(connect, wait);
    }

    connect();

    return () => {
      closed = true;
      if (reconnectTimer !== null) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      if (source) {
        source.close();
        source = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    queryClient,
    options.coordinatorUrl,
    options.appName,
    options.pattern,
    options.eventSourceFactory,
  ]);
}
