// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * SBFB Bridge SDK — client library for iframe apps.
 *
 * Sprint 13 Phase C. Include this file in your app to communicate
 * with the SBFB network through the host shell:
 *
 *   <script src="/sbfb-bridge.js"></script>
 *   <script>
 *     const bridge = new SBFBBridge();
 *     const result = await bridge.submitTask({ prompt: "Hello" });
 *   </script>
 *
 * All communication goes through window.postMessage. The host shell
 * validates each request, forwards it to the coordinator API, and
 * sends back a typed response with a correlation ID.
 *
 * Sprint 15 Phase A — adds `onEvent(name, callback)` for push events
 *   pushed by the host toward the iframe (fire-and-forget).
 */

// eslint-disable-next-line no-unused-vars
class SBFBBridge {
  /**
   * @param {Object} [options]
   * @param {number} [options.timeout=10000] — ms before a request rejects
   */
  constructor(options) {
    this._timeout = (options && options.timeout) || 10000;
    this._pending = new Map();
    // Sprint 15 Phase A: subscribers for host push events, keyed by
    // event name. Each value is a Set<callback> so multiple consumers
    // of the same event can coexist.
    this._eventHandlers = new Map();

    this._onMessage = (event) => {
      const msg = event.data;
      if (!msg) return;

      if (msg.type === "sbfb-bridge-response") {
        const resolve = this._pending.get(msg.id);
        if (!resolve) return;
        this._pending.delete(msg.id);
        resolve(msg);
        return;
      }

      if (msg.type === "sbfb-bridge-event") {
        const handlers = this._eventHandlers.get(msg.name);
        if (!handlers) return;
        for (const cb of handlers) {
          try {
            cb(msg.payload);
          } catch (e) {
            // Swallow callback errors so one faulty handler doesn't
            // block the others or the bridge itself.
            // eslint-disable-next-line no-console
            if (typeof console !== "undefined") console.error("onEvent callback threw", e);
          }
        }
      }
    };

    window.addEventListener("message", this._onMessage);
  }

  /** Stop listening. Call when the app unmounts. */
  destroy() {
    window.removeEventListener("message", this._onMessage);
    // Reject all pending requests.
    for (const [id, resolve] of this._pending) {
      resolve({ type: "sbfb-bridge-response", id, success: false, error: "bridge destroyed" });
    }
    this._pending.clear();
    this._eventHandlers.clear();
  }

  /**
   * Subscribe to a push event from the host. Sprint 15 Phase A.
   *
   * @param {string} name — event name to listen for
   * @param {(payload: unknown) => void} callback — called on each event
   * @returns {() => void} — unsubscribe function (idempotent)
   *
   * @example
   *   bridge.onEvent("task_result_ready", (payload) => {
   *     console.log("task done:", payload.task_id);
   *   });
   */
  onEvent(name, callback) {
    if (typeof name !== "string" || !name.length) {
      throw new Error("onEvent: name must be a non-empty string");
    }
    if (typeof callback !== "function") {
      throw new Error("onEvent: callback must be a function");
    }
    let handlers = this._eventHandlers.get(name);
    if (!handlers) {
      handlers = new Set();
      this._eventHandlers.set(name, handlers);
    }
    handlers.add(callback);
    return () => {
      const current = this._eventHandlers.get(name);
      if (current) current.delete(callback);
    };
  }

  /**
   * Submit a compute task to the SBFB network.
   * @param {Object} payload — task parameters (prompt, task_type, etc.)
   * @returns {Promise<Object>} — coordinator response (task_id, etc.)
   */
  submitTask(payload) {
    return this._call("task_submit", payload || {});
  }

  /**
   * Read a value from the coordinator's typed storage.
   * @param {string} key — storage namespace key
   * @returns {Promise<*>} — stored value
   */
  getStorage(key) {
    return this._call("storage_get", { key: key });
  }

  /**
   * Write a value to the coordinator's typed storage.
   * @param {string} key — storage namespace key
   * @param {Object} value — data to persist
   * @returns {Promise<Object>} — {ok: true}
   */
  setStorage(key, value) {
    return this._call("storage_set", Object.assign({ key: key }, value || {}));
  }

  /**
   * @private
   * @param {string} method
   * @param {Object} payload
   * @returns {Promise<*>}
   */
  _call(method, payload) {
    var self = this;
    var id = self._uuid();

    return new Promise(function (resolve, reject) {
      var timer = setTimeout(function () {
        self._pending.delete(id);
        reject(new Error("bridge timeout after " + self._timeout + "ms"));
      }, self._timeout);

      self._pending.set(id, function (msg) {
        clearTimeout(timer);
        if (msg.success) {
          resolve(msg.data);
        } else {
          reject(new Error(msg.error || "bridge error"));
        }
      });

      parent.postMessage(
        {
          type: "sbfb-bridge-request",
          id: id,
          method: method,
          payload: payload,
        },
        "*",
      );
    });
  }

  /** @private */
  _uuid() {
    // crypto.randomUUID polyfill for older browsers.
    if (typeof crypto !== "undefined" && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
      var r = (Math.random() * 16) | 0;
      return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
    });
  }
}
