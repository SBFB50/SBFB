// SPDX-License-Identifier: AGPL-3.0-or-later
;(function () {
  var key = 'factory-operator.accessibility.v1'
  var defaults = {
    theme: 'dark',
    contrast: 'standard',
    pointer: 'standard',
    textSpacing: 'standard',
    font: 'standard',
    motion: 'system',
    scale: '100',
    shortcuts: 'off',
  }

  function pick(value, allowed, fallback) {
    return allowed.indexOf(value) === -1 ? fallback : value
  }

  function read() {
    try {
      var raw = window.localStorage.getItem(key)
      if (!raw) return defaults
      var parsed = JSON.parse(raw)
      return {
        theme: pick(parsed.theme, ['dark', 'light'], defaults.theme),
        contrast: pick(parsed.contrast, ['standard', 'high'], defaults.contrast),
        pointer: pick(parsed.pointer, ['standard', 'large'], defaults.pointer),
        textSpacing: pick(parsed.textSpacing, ['standard', 'loose'], defaults.textSpacing),
        font: pick(parsed.font, ['standard', 'legible'], defaults.font),
        motion: pick(parsed.motion, ['system', 'reduced'], defaults.motion),
        scale: pick(parsed.scale, ['100', '112', '125'], defaults.scale),
        shortcuts: pick(parsed.shortcuts, ['off', 'on'], defaults.shortcuts),
      }
    } catch (_) {
      return defaults
    }
  }

  function reducedBySystem() {
    try {
      return window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches
    } catch (_) {
      return false
    }
  }

  var colorKeys = ['s0', 's1', 's2', 's3', 'bd', 'bd2', 'field', 'tx', 'tx2', 'tx3', 'tx4', 'ok', 'warn', 'bad', 'info']
  var lightColors = {
    s0: 'oklch(0.975 0.004 260)',
    s1: 'oklch(0.945 0.005 260)',
    s2: 'oklch(0.910 0.006 260)',
    s3: 'oklch(0.870 0.008 260)',
    bd: 'oklch(0.740 0.010 260)',
    bd2: 'oklch(0.610 0.012 260)',
    field: 'oklch(0.500 0.014 260)',
    tx: 'oklch(0.185 0.008 260)',
    tx2: 'oklch(0.315 0.010 260)',
    tx3: 'oklch(0.405 0.010 260)',
    tx4: 'oklch(0.470 0.010 260)',
    ok: 'oklch(0.420 0.130 150)',
    warn: 'oklch(0.470 0.120 78)',
    bad: 'oklch(0.475 0.170 25)',
    info: 'oklch(0.430 0.120 240)',
  }
  var highColors = {
    bd: 'oklch(0.560 0.010 260)',
    bd2: 'oklch(0.720 0.012 260)',
    field: 'oklch(0.760 0.014 260)',
    tx: 'oklch(0.985 0.004 260)',
    tx2: 'oklch(0.900 0.005 260)',
    tx3: 'oklch(0.840 0.006 260)',
    tx4: 'oklch(0.785 0.006 260)',
  }
  var lightHighColors = {
    bd: 'oklch(0.470 0.012 260)',
    bd2: 'oklch(0.300 0.014 260)',
    field: 'oklch(0.260 0.014 260)',
    tx: 'oklch(0.075 0.006 260)',
    tx2: 'oklch(0.175 0.008 260)',
    tx3: 'oklch(0.250 0.009 260)',
    tx4: 'oklch(0.315 0.010 260)',
  }

  function applyColors(root, prefs) {
    for (var i = 0; i < colorKeys.length; i++) root.style.removeProperty('--color-' + colorKeys[i])
    var colors = {}
    if (prefs.theme === 'light') {
      Object.assign(colors, lightColors)
      if (prefs.contrast === 'high') Object.assign(colors, lightHighColors)
    } else if (prefs.contrast === 'high') {
      Object.assign(colors, highColors)
    }
    Object.keys(colors).forEach(function (name) {
      root.style.setProperty('--color-' + name, colors[name])
    })
  }

  var prefs = read()
  var root = document.documentElement
  root.setAttribute('data-theme', prefs.theme)
  root.setAttribute('data-contrast', prefs.contrast)
  root.setAttribute('data-pointer', prefs.pointer)
  root.setAttribute('data-text-spacing', prefs.textSpacing)
  root.setAttribute('data-font', prefs.font)
  root.setAttribute('data-motion', prefs.motion === 'reduced' || reducedBySystem() ? 'reduced' : 'standard')
  root.setAttribute('data-scale', prefs.scale)
  root.setAttribute('data-shortcuts', prefs.shortcuts)
  applyColors(root, prefs)
  root.style.colorScheme = prefs.theme
})()
