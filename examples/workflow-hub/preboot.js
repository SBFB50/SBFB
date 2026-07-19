// SPDX-License-Identifier: AGPL-3.0-or-later
/*
 * Sprint 83 Phase B — Workflow Hub accessibility preboot (anti-flash).
 *
 * Runs synchronously in <head> before first paint. The Hub is a sandboxed
 * iframe app: it NEVER sees the operator's needs (a different origin, no
 * shared storage). It applies a needs-free RESOLVED profile:
 *   - needs-free DEFAULTS (dark, standard) as the baseline;
 *   - the OS/browser signals it CAN read (reduced-motion, forced-colors,
 *     prefers-contrast) — the flash-sensitive axes.
 *
 * There is deliberately NO URL/hash transport for the profile (Codex S83-B P1):
 * a generic `#a11y=` hint would be APP-READABLE (`location.hash`) and could
 * carry the cause across the frontier even if it never reached the DOM. The
 * full resolved profile arrives ONLY via the bridge event (a11y.js), which the
 * host builds needs-free (`buildAccessibilityProfileEvent`). The flash-sensitive
 * axes are covered here by the OS signals; the theme resolves on the bridge
 * event.
 *
 * It sets the 15 effect `data-*` attributes and the oklch colour tokens, and
 * NEVER writes `data-needs`. This is the deliberate difference from the
 * reference tool's preboot.js:367 (which leaks `data-needs`).
 */
;(function () {
  var AXES = {
    theme: ['dark', 'light', 'calm', 'paper', 'forced'],
    contrast: ['standard', 'high'],
    colorVision: ['standard', 'safe', 'monochrome'],
    pointer: ['standard', 'large'],
    textSpacing: ['standard', 'loose'],
    font: ['standard', 'legible'],
    motion: ['standard', 'reduced'],
    transparency: ['standard', 'reduced'],
    density: ['standard', 'focus'],
    reading: ['standard', 'assist'],
    focus: ['standard', 'strong'],
    scale: ['100', '112', '125', '150'],
    shortcuts: ['off', 'on'],
    assistiveTech: ['standard', 'screen-reader'],
    captions: ['off', 'on'],
  }
  var DEFAULTS = {
    theme: 'dark',
    contrast: 'standard',
    colorVision: 'standard',
    pointer: 'standard',
    textSpacing: 'standard',
    font: 'standard',
    motion: 'standard',
    transparency: 'standard',
    density: 'standard',
    reading: 'standard',
    focus: 'standard',
    scale: '100',
    shortcuts: 'off',
    assistiveTech: 'standard',
    captions: 'off',
  }

  function media(query) {
    try {
      return window.matchMedia && window.matchMedia(query).matches
    } catch (_) {
      return false
    }
  }

  // Only the 15 effect axes with valid values survive — `needs` (or any other
  // smuggled key) is dropped here, by construction.
  function whitelist(source) {
    var out = {}
    for (var key in AXES) {
      if (!Object.prototype.hasOwnProperty.call(AXES, key)) continue
      var v = source && source[key]
      out[key] = AXES[key].indexOf(v) === -1 ? DEFAULTS[key] : v
    }
    return out
  }

  var forcedColors = media('(forced-colors: active)')
  var highContrast = media('(prefers-contrast: more)')
  var reducedMotion = media('(prefers-reduced-motion: reduce)')

  // No profile is known at preboot (sandboxed origin, no access to the
  // operator's prefs, and NO app-readable hint channel). Start from needs-free
  // DEFAULTS and fold in ONLY the OS/browser signals — the flash-sensitive
  // axes. The full resolved profile arrives later via the bridge (a11y.js).
  var profile = whitelist({})
  if (reducedMotion) profile.motion = 'reduced'
  if (highContrast || forcedColors) profile.contrast = 'high'
  if (forcedColors) profile.theme = 'forced'

  // Colour tokens for non-dark themes + high contrast + colour-vision.
  var colorKeys = ['s0', 's1', 's2', 's3', 'bd', 'bd2', 'field', 'tx', 'tx2', 'tx3', 'tx4', 'ok', 'warn', 'bad', 'info', 'neu', 'ok-bg', 'bad-bg']
  var lightColors = { s0: 'oklch(0.975 0.004 260)', s1: 'oklch(0.945 0.005 260)', s2: 'oklch(0.910 0.006 260)', s3: 'oklch(0.870 0.008 260)', bd: 'oklch(0.740 0.010 260)', bd2: 'oklch(0.610 0.012 260)', field: 'oklch(0.500 0.014 260)', tx: 'oklch(0.185 0.008 260)', tx2: 'oklch(0.315 0.010 260)', tx3: 'oklch(0.405 0.010 260)', tx4: 'oklch(0.470 0.010 260)', ok: 'oklch(0.420 0.130 150)', warn: 'oklch(0.470 0.120 78)', bad: 'oklch(0.475 0.170 25)', info: 'oklch(0.430 0.120 240)', neu: 'oklch(0.405 0.010 260)', 'ok-bg': 'oklch(0.910 0.030 150)', 'bad-bg': 'oklch(0.920 0.035 25)' }
  var calmColors = { s0: 'oklch(0.200 0.012 180)', s1: 'oklch(0.235 0.012 180)', s2: 'oklch(0.275 0.014 180)', s3: 'oklch(0.320 0.015 180)', bd: 'oklch(0.390 0.012 180)', bd2: 'oklch(0.500 0.014 180)', field: 'oklch(0.610 0.016 180)', tx: 'oklch(0.930 0.006 170)', tx2: 'oklch(0.760 0.008 170)', tx3: 'oklch(0.700 0.009 170)', tx4: 'oklch(0.640 0.010 170)', ok: 'oklch(0.760 0.090 155)', warn: 'oklch(0.800 0.090 80)', bad: 'oklch(0.760 0.105 25)', info: 'oklch(0.780 0.070 220)', neu: 'oklch(0.700 0.009 170)', 'ok-bg': 'oklch(0.310 0.032 155)', 'bad-bg': 'oklch(0.300 0.035 25)' }
  var paperColors = { s0: 'oklch(0.970 0.006 95)', s1: 'oklch(0.945 0.008 95)', s2: 'oklch(0.910 0.010 95)', s3: 'oklch(0.860 0.012 95)', bd: 'oklch(0.720 0.014 95)', bd2: 'oklch(0.560 0.016 95)', field: 'oklch(0.410 0.018 95)', tx: 'oklch(0.160 0.012 80)', tx2: 'oklch(0.290 0.014 80)', tx3: 'oklch(0.380 0.014 80)', tx4: 'oklch(0.455 0.014 80)', ok: 'oklch(0.400 0.120 150)', warn: 'oklch(0.455 0.115 78)', bad: 'oklch(0.455 0.160 25)', info: 'oklch(0.390 0.110 240)', neu: 'oklch(0.380 0.014 80)', 'ok-bg': 'oklch(0.900 0.032 150)', 'bad-bg': 'oklch(0.905 0.036 25)' }
  var forcedColorsMap = { s0: '#000000', s1: '#050505', s2: '#101010', s3: '#1a1a1a', bd: '#9f9f9f', bd2: '#ffffff', field: '#ffffff', tx: '#ffffff', tx2: '#f2f2f2', tx3: '#e6e6e6', tx4: '#d8d8d8', ok: '#ffffff', warn: '#ffff00', bad: '#ff6b6b', info: '#66ccff', neu: '#e6e6e6', 'ok-bg': '#111111', 'bad-bg': '#220000' }
  var highColors = { bd: 'oklch(0.560 0.010 260)', bd2: 'oklch(0.720 0.012 260)', field: 'oklch(0.760 0.014 260)', tx: 'oklch(0.985 0.004 260)', tx2: 'oklch(0.900 0.005 260)', tx3: 'oklch(0.840 0.006 260)', tx4: 'oklch(0.785 0.006 260)' }
  var lightHighColors = { bd: 'oklch(0.470 0.012 260)', bd2: 'oklch(0.300 0.014 260)', field: 'oklch(0.260 0.014 260)', tx: 'oklch(0.075 0.006 260)', tx2: 'oklch(0.175 0.008 260)', tx3: 'oklch(0.250 0.009 260)', tx4: 'oklch(0.315 0.010 260)' }
  var colorSafeColors = { ok: '#009e73', warn: '#e69f00', bad: '#d55e00', info: '#0072b2' }
  var monochromeColors = { ok: 'var(--color-tx)', warn: 'var(--color-tx2)', bad: 'var(--color-tx3)', info: 'var(--color-tx2)' }

  function assign(target, source) {
    for (var name in source) {
      if (Object.prototype.hasOwnProperty.call(source, name)) target[name] = source[name]
    }
  }

  // Apply a needs-free resolved profile to the document root: the 15 effect
  // `data-*` attributes + the oklch colour tokens. NEVER writes `data-needs`.
  // Reused by a11y.js for the live (bridge-pushed) profile — one apply path.
  function applyProfile(next) {
    var p = whitelist(next)
    var root = document.documentElement
    root.setAttribute('data-theme', p.theme)
    root.setAttribute('data-contrast', p.contrast)
    root.setAttribute('data-color-vision', p.colorVision)
    root.setAttribute('data-pointer', p.pointer)
    root.setAttribute('data-text-spacing', p.textSpacing)
    root.setAttribute('data-font', p.font)
    root.setAttribute('data-motion', p.motion)
    root.setAttribute('data-transparency', p.transparency)
    root.setAttribute('data-density', p.density)
    root.setAttribute('data-reading', p.reading)
    root.setAttribute('data-focus', p.focus)
    root.setAttribute('data-scale', p.scale)
    root.setAttribute('data-shortcuts', p.shortcuts)
    root.setAttribute('data-assistive-tech', p.assistiveTech)
    root.setAttribute('data-captions', p.captions)
    // NOTE: NO data-needs — the cause never reaches this DOM.

    for (var i = 0; i < colorKeys.length; i++) root.style.removeProperty('--color-' + colorKeys[i])
    var colors = {}
    if (p.theme === 'light') assign(colors, lightColors)
    if (p.theme === 'calm') assign(colors, calmColors)
    if (p.theme === 'paper') assign(colors, paperColors)
    if (p.theme === 'forced') assign(colors, forcedColorsMap)
    if (p.contrast === 'high' && p.theme === 'dark') assign(colors, highColors)
    if (p.contrast === 'high' && p.theme === 'light') assign(colors, lightHighColors)
    if (p.colorVision === 'safe') assign(colors, colorSafeColors)
    if (p.colorVision === 'monochrome') assign(colors, monochromeColors)
    for (var name in colors) {
      if (Object.prototype.hasOwnProperty.call(colors, name)) root.style.setProperty('--color-' + name, colors[name])
    }
    root.style.colorScheme = p.theme === 'light' || p.theme === 'paper' ? 'light' : 'dark'
    return p
  }

  // Expose the vocabulary + the single apply path for a11y.js (live profile).
  window.SbfbHubA11y = {
    AXES: AXES,
    DEFAULTS: DEFAULTS,
    whitelist: whitelist,
    applyProfile: applyProfile,
  }

  applyProfile(profile)
})()
