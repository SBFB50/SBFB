// SPDX-License-Identifier: AGPL-3.0-or-later
/*
 * Sprint 83 Phase B — Workflow Hub live accessibility channel.
 *
 * The host projects the operator's needs-free resolved profile to this
 * sandboxed app via a bridge event (`sbfb-bridge-event`, name `a11y-profile`).
 * This script subscribes, validates the payload, and re-applies it WITHOUT a
 * reload — the "propagée sans reload" contract.
 *
 * Validation is STRICT (mirrors the host-side Zod `.strict()`): a payload is
 * REJECTED outright if it carries ANY key outside the 15 effect axes — a
 * smuggled `needs` (or any other extra key) means the whole message is dropped,
 * not silently sanitised. The message must also come from the parent window.
 * The app can never be tricked into rendering the cause. Depends on preboot.js
 * having run first (it exposes `window.SbfbHubA11y`).
 */
;(function () {
  var A11Y_PROFILE_EVENT = 'a11y-profile'
  var api = window.SbfbHubA11y
  if (!api || typeof api.applyProfile !== 'function') {
    // preboot.js did not run — nothing to wire. The default profile is
    // already on the DOM (or the head order is wrong); fail safe, do nothing.
    return
  }

  function isBridgeEvent(data) {
    return (
      data &&
      typeof data === 'object' &&
      data.type === 'sbfb-bridge-event' &&
      data.name === A11Y_PROFILE_EVENT
    )
  }

  // STRICT (mirrors the host Zod `.strict()`): the payload must have EXACTLY
  // the 15 effect-axis keys, each with an allowed value. A missing axis, an
  // invalid value, or ANY extra key (a smuggled `needs`) rejects the WHOLE
  // payload — no silent default-filling.
  function isStrictProfile(payload) {
    if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return false
    var axisKeys = Object.keys(api.AXES)
    if (Object.keys(payload).length !== axisKeys.length) return false
    for (var i = 0; i < axisKeys.length; i++) {
      var k = axisKeys[i]
      if (api.AXES[k].indexOf(payload[k]) === -1) return false
    }
    return true
  }

  // Must come from THIS iframe's parent host. Rejects a null/undefined source
  // and the top-level (no-parent) mode — Codex S83-B P2.
  function fromParent(event) {
    return !!(window.parent && window.parent !== window && event && event.source === window.parent)
  }

  function onMessage(event) {
    if (!fromParent(event)) return
    var data = event && event.data
    if (!isBridgeEvent(data)) return
    var payload = data.payload
    if (!isStrictProfile(payload)) return // reject incomplete/invalid/smuggled outright
    api.applyProfile(payload)
    // Let the app react (e.g. re-render a resolved-axes readout).
    try {
      window.dispatchEvent(
        new CustomEvent('sbfb-hub:a11y-applied', {
          detail: { axes: Object.keys(api.AXES) },
        }),
      )
    } catch (_) {
      /* CustomEvent unsupported — the DOM is already updated. */
    }
  }

  window.addEventListener('message', onMessage)

  // Signal readiness so the host can push the initial profile if it waits for
  // the app rather than baking the hint into the iframe src.
  try {
    if (window.parent && window.parent !== window) {
      window.parent.postMessage({ type: 'sbfb-hub:a11y-ready' }, '*')
    }
  } catch (_) {
    /* cross-origin parent — the host still pushes on its own cadence. */
  }
})()
