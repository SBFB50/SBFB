// SPDX-License-Identifier: AGPL-3.0-or-later
// SBFB Bridge SDK — starter template.
// Full SDK: web/public/sbfb-bridge.js in the SBFB repository.
class SBFBBridge {
  constructor(options) {
    this._timeout = (options && options.timeout) || 10000;
    this._pending = new Map();
    var self = this;
    this._onMessage = function(event) {
      var msg = event.data;
      if (!msg || msg.type !== "sbfb-bridge-response") return;
      var resolve = self._pending.get(msg.id);
      if (!resolve) return;
      self._pending.delete(msg.id);
      resolve(msg);
    };
    window.addEventListener("message", this._onMessage);
  }

  destroy() {
    window.removeEventListener("message", this._onMessage);
    for (var entry of this._pending) {
      entry[1]({ type: "sbfb-bridge-response", id: entry[0], success: false, error: "bridge destroyed" });
    }
    this._pending.clear();
  }

  submitTask(payload) { return this._call("task_submit", payload || {}); }
  getTaskResult(taskId) { return this._call("task_result", { task_id: taskId }); }
  getStorage(key) { return this._call("storage_get", { key: key }); }
  setStorage(key, value) { return this._call("storage_set", Object.assign({ key: key }, value || {})); }
  getIdentityPubkey() { return this._call("identity_pubkey", {}); }
  getNodeStatus() { return this._call("node_status", {}); }

  _call(method, payload) {
    var self = this;
    var id = self._uuid();
    return new Promise(function(resolve, reject) {
      var timer = setTimeout(function() {
        self._pending.delete(id);
        reject(new Error("bridge timeout after " + self._timeout + "ms"));
      }, self._timeout);
      self._pending.set(id, function(msg) {
        clearTimeout(timer);
        if (msg.success) resolve(msg.data);
        else reject(new Error(msg.error || "bridge error"));
      });
      parent.postMessage({ type: "sbfb-bridge-request", id: id, method: method, payload: payload }, "*");
    });
  }

  _uuid() {
    if (typeof crypto !== "undefined" && crypto.randomUUID) return crypto.randomUUID();
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function(c) {
      var r = (Math.random() * 16) | 0;
      return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
    });
  }
}
