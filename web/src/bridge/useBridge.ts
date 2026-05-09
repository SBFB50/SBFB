// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — host-side bridge listener.
 * Sprint 15 Phase A — exposes `pushEvent` so the host can push
 *   fire-and-forget events toward the iframe.
 * Sprint 15 Phase B — adds a CPU watchdog based on iframe
 *   heartbeats so the host can detect stalled apps and offer a
 *   reload overlay to the user.
 *
 * React hook that listens for postMessage events from sandboxed
 * iframes, validates them against the bridge protocol schema,
 * dispatches to the coordinator API, and sends typed responses.
 */

import { useCallback, useEffect, useRef, useState } from "react";

import {
  BridgeHeartbeatSchema,
  BridgeRequestSchema,
  PiiRedactPayloadSchema,
  createErrorResponse,
  createEvent,
  createResponse,
  type BridgeRequest,
} from "@/bridge/protocol";
import {
  submitAppTask,
  setAppState,
  SubmitAppTaskBodySchema,
} from "@/api/coordinator";
import { authFetch } from "@/api/auth";
import { detectAndRedact, type PiiPolicy } from "@/sdk/pii";

/**
 * Watchdog state for a single iframe.
 *
 * - `unknown`: the iframe hasn't emitted any heartbeat yet (initial
 *   load, or no bridge SDK installed). UI shows the normal iframe
 *   content (no overlay).
 * - `healthy`: at least one heartbeat has been received within the
 *   stall threshold. Business as usual.
 * - `stalled`: the last heartbeat was received more than
 *   {@link STALL_THRESHOLD_MS} ago. The UI shows the "Application ne
 *   repond plus" overlay so the user can reload or close the app.
 */
export type WatchdogState = "unknown" | "healthy" | "stalled";

/** Milliseconds without a heartbeat before the iframe is declared stalled. */
export const STALL_THRESHOLD_MS = 5000;

/** How often the host re-evaluates the staleness of the iframe. */
const WATCHDOG_CHECK_INTERVAL_MS = 2000;

/** Timeout for coordinator API calls (ms). */
const API_TIMEOUT = 10_000;

/** Return shape of {@link useBridge}. */
export interface UseBridgeHandle {
  /**
   * Push a fire-and-forget event to the iframe. Sprint 15 Phase A.
   *
   * No-op when the iframe ref is unmounted or has no contentWindow.
   * The host does not wait for acknowledgement — events are best-
   * effort notifications.
   */
  pushEvent: (name: string, payload: unknown) => void;

  /**
   * Current watchdog state for the iframe. Sprint 15 Phase B.
   *
   * `unknown` at mount, transitions to `healthy` on the first
   * heartbeat, and degrades to `stalled` if no heartbeat is received
   * for {@link STALL_THRESHOLD_MS}.
   */
  watchdogState: WatchdogState;

  /**
   * Reset the watchdog back to `unknown` (called when the caller
   * reloads the iframe). Sprint 15 Phase B.
   */
  resetWatchdog: () => void;
}

/**
 * Mount the bridge listener for a specific app on a specific
 * coordinator. The hook attaches a `message` event listener on
 * mount and removes it on unmount.
 *
 * @param coordUrl — base URL of the active coordinator
 * @param appName  — name of the app whose API we proxy to
 * @param iframeRef — ref to the iframe element (for source validation)
 * @returns handle with `pushEvent` for host → iframe push (Sprint 15)
 */
export function useBridge(
  coordUrl: string | null,
  appName: string | null,
  iframeRef: React.RefObject<HTMLIFrameElement | null>,
): UseBridgeHandle {
  const coordUrlRef = useRef(coordUrl);
  const appNameRef = useRef(appName);
  const lastHeartbeatRef = useRef<number | null>(null);
  const [watchdogState, setWatchdogState] = useState<WatchdogState>("unknown");

  useEffect(() => {
    coordUrlRef.current = coordUrl;
    appNameRef.current = appName;
  });

  useEffect(() => {
    function handler(event: MessageEvent) {
      // Sprint 15 Phase B: heartbeat path is checked before the
      // request path because heartbeats are emitted far more often
      // than tasks, and they don't need source validation (they
      // carry no side effects).
      const hb = BridgeHeartbeatSchema.safeParse(event.data);
      if (hb.success) {
        const iframe = iframeRef.current;
        if (!iframe || event.source !== iframe.contentWindow) return;
        lastHeartbeatRef.current = Date.now();
        setWatchdogState((prev) => (prev === "healthy" ? prev : "healthy"));
        return;
      }

      // Ignore messages that don't parse as bridge requests.
      const parsed = BridgeRequestSchema.safeParse(event.data);
      if (!parsed.success) return;

      const req = parsed.data;

      // Validate source: must come from our tracked iframe.
      const iframe = iframeRef.current;
      if (!iframe || event.source !== iframe.contentWindow) return;

      // Sprint 21 Phase B — pii_redact dispatches locally inside
      // the host shell (no coordinator round-trip), so it must be
      // reachable even when `coordUrl` is null (offline app, boot
      // sequence, degraded mode). Keep this branch ABOVE the coord
      // guard so the bridge stays usable without a live coord.
      if (req.method === "pii_redact") {
        void dispatchPiiRedact(req).then(
          (data) => reply(event.source as Window, createResponse(req.id, data)),
          (err) =>
            reply(
              event.source as Window,
              createErrorResponse(
                req.id,
                err instanceof Error ? err.message : String(err),
              ),
            ),
        );
        return;
      }

      const url = coordUrlRef.current;
      const app = appNameRef.current;
      if (!url || !app) {
        reply(event.source as Window, createErrorResponse(req.id, "no active coordinator or app"));
        return;
      }

      // Dispatch async — don't block the event loop.
      void dispatch(url, app, req).then(
        (data) => reply(event.source as Window, createResponse(req.id, data)),
        (err) =>
          reply(
            event.source as Window,
            createErrorResponse(req.id, err instanceof Error ? err.message : String(err)),
          ),
      );
    }

    window.addEventListener("message", handler);
    return () => window.removeEventListener("message", handler);
  }, [iframeRef]);

  // Sprint 15 Phase B: periodically check whether the last heartbeat
  // is stale. This runs independently of the message listener so a
  // frozen iframe (no new postMessage) can still be detected.
  useEffect(() => {
    const timer = setInterval(() => {
      const last = lastHeartbeatRef.current;
      if (last === null) return; // still "unknown"
      const age = Date.now() - last;
      if (age > STALL_THRESHOLD_MS) {
        setWatchdogState((prev) => (prev === "stalled" ? prev : "stalled"));
      }
    }, WATCHDOG_CHECK_INTERVAL_MS);
    return () => clearInterval(timer);
  }, []);

  const resetWatchdog = useCallback(() => {
    lastHeartbeatRef.current = null;
    setWatchdogState("unknown");
  }, []);

  const pushEvent = useCallback(
    (name: string, payload: unknown) => {
      const iframe = iframeRef.current;
      if (!iframe || !iframe.contentWindow) return;
      iframe.contentWindow.postMessage(createEvent(name, payload), "*");
    },
    [iframeRef],
  );

  return { pushEvent, watchdogState, resetWatchdog };
}

function reply(target: Window, response: import("@/bridge/protocol").BridgeResponse) {
  target.postMessage(response, "*");
}

async function dispatchPiiRedact(
  req: BridgeRequest,
): Promise<{ redacted_text: string; findings_count: number }> {
  const parsed = PiiRedactPayloadSchema.parse(req.payload);
  const override = parsed.policy as Partial<PiiPolicy> | undefined;
  const result = await detectAndRedact(parsed.text, override);
  return {
    redacted_text: result.text,
    findings_count: result.findings.length,
  };
}

async function dispatch(
  coordUrl: string,
  appName: string,
  req: BridgeRequest,
): Promise<unknown> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), API_TIMEOUT);

  try {
    switch (req.method) {
      case "task_submit": {
        const taskBody = SubmitAppTaskBodySchema.parse(req.payload);
        return await submitAppTask(coordUrl, appName, taskBody);
      }

      case "storage_get": {
        const key = String(req.payload.key ?? "");
        if (!key) throw new Error("storage_get requires payload.key");
        const resp = await authFetch(
          `${coordUrl}/app/${encodeURIComponent(appName)}/state/${encodeURIComponent(key)}`,
          { signal: controller.signal },
        );
        if (!resp.ok) throw new Error(`storage_get failed: ${resp.status}`);
        return await resp.json();
      }

      case "storage_set": {
        const key = String(req.payload.key ?? "");
        if (!key) throw new Error("storage_set requires payload.key");
        const body = { ...req.payload };
        delete body.key;
        return await setAppState(coordUrl, appName, key, body as Record<string, unknown>);
      }

      case "storage_list": {
        const prefix = String(req.payload.prefix ?? "");
        const qs = prefix ? `?prefix=${encodeURIComponent(prefix)}` : "";
        const resp = await authFetch(
          `${coordUrl}/app/${encodeURIComponent(appName)}/state${qs}`,
          { signal: controller.signal },
        );
        if (!resp.ok) throw new Error(`storage_list failed: ${resp.status}`);
        return await resp.json();
      }

      case "storage_delete": {
        const key = String(req.payload.key ?? "");
        if (!key) throw new Error("storage_delete requires payload.key");
        const resp = await authFetch(
          `${coordUrl}/app/${encodeURIComponent(appName)}/state/${encodeURIComponent(key)}`,
          { method: "DELETE", signal: controller.signal },
        );
        if (!resp.ok) throw new Error(`storage_delete failed: ${resp.status}`);
        return await resp.json();
      }

      case "identity_pubkey": {
        const resp = await authFetch(`${coordUrl}/api/daemon/info`, {
          signal: controller.signal,
        });
        if (!resp.ok) throw new Error(`identity_pubkey failed: ${resp.status}`);
        const info = await resp.json();
        return { pubkey: info.node_id };
      }

      case "node_status": {
        const [healthResp, infoResp] = await Promise.all([
          authFetch(`${coordUrl}/api/v1/coordinator/health`, {
            signal: controller.signal,
          }),
          authFetch(`${coordUrl}/api/daemon/info`, {
            signal: controller.signal,
          }),
        ]);
        if (!healthResp.ok) throw new Error(`node_status failed: ${healthResp.status}`);
        const health = await healthResp.json();
        const peers = infoResp.ok
          ? ((await infoResp.json()) as { known_browse_entries?: number; subscribed_curators?: string[] }).subscribed_curators?.length ?? 0
          : 0;
        return { ...health, peers };
      }

      case "browse_list": {
        const resp = await authFetch(`${coordUrl}/api/daemon/browse`, {
          signal: controller.signal,
        });
        if (!resp.ok) throw new Error(`browse_list failed: ${resp.status}`);
        return await resp.json();
      }

      default:
        throw new Error(`unknown bridge method: ${req.method}`);
    }
  } finally {
    clearTimeout(timer);
  }
}
