// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — host-side bridge listener.
 * Sprint 15 Phase A — exposes `pushEvent` so the host can push
 *   fire-and-forget events toward the iframe.
 *
 * React hook that listens for postMessage events from sandboxed
 * iframes, validates them against the bridge protocol schema,
 * dispatches to the coordinator API, and sends typed responses.
 */

import { useCallback, useEffect, useRef } from "react";

import {
  BridgeRequestSchema,
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

  useEffect(() => {
    coordUrlRef.current = coordUrl;
    appNameRef.current = appName;
  });

  useEffect(() => {
    function handler(event: MessageEvent) {
      // Ignore messages that don't parse as bridge requests.
      const parsed = BridgeRequestSchema.safeParse(event.data);
      if (!parsed.success) return;

      const req = parsed.data;

      // Validate source: must come from our tracked iframe.
      const iframe = iframeRef.current;
      if (!iframe || event.source !== iframe.contentWindow) return;

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

  const pushEvent = useCallback(
    (name: string, payload: unknown) => {
      const iframe = iframeRef.current;
      if (!iframe || !iframe.contentWindow) return;
      iframe.contentWindow.postMessage(createEvent(name, payload), "*");
    },
    [iframeRef],
  );

  return { pushEvent };
}

function reply(target: Window, response: import("@/bridge/protocol").BridgeResponse) {
  target.postMessage(response, "*");
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
        const resp = await fetch(
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

      default:
        throw new Error(`unknown bridge method: ${req.method}`);
    }
  } finally {
    clearTimeout(timer);
  }
}
