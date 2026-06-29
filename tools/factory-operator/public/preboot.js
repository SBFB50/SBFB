// SPDX-License-Identifier: AGPL-3.0-or-later
;(function () {
  var key = 'factory-operator.accessibility.v2'
  var legacyKey = 'factory-operator.accessibility.v1'
  var needs = [
    'lowVision',
    'blindAssistive',
    'colorVision',
    'noColor',
    'vestibular',
    'photosensitive',
    'motor',
    'cognitive',
    'dyslexia',
    'attention',
    'auditory',
    'speech',
    'sensory',
  ]
  var defaults = {
    needs: [],
    theme: 'auto',
    contrast: 'auto',
    colorVision: 'standard',
    pointer: 'standard',
    textSpacing: 'standard',
    font: 'standard',
    motion: 'system',
    transparency: 'standard',
    density: 'standard',
    reading: 'standard',
    focus: 'standard',
    scale: '100',
    shortcuts: 'off',
    assistiveTech: 'standard',
    captions: 'off',
  }

  function pick(value, allowed, fallback) {
    return allowed.indexOf(value) === -1 ? fallback : value
  }

  function pickNeeds(value) {
    if (!Array.isArray(value)) return []
    var next = []
    for (var i = 0; i < value.length; i++) {
      if (typeof value[i] !== 'string' || needs.indexOf(value[i]) === -1 || next.indexOf(value[i]) !== -1) continue
      next.push(value[i])
    }
    return next
  }

  function sanitize(parsed) {
    parsed = parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {}
    return {
      needs: pickNeeds(parsed.needs),
      theme: pick(parsed.theme, ['auto', 'dark', 'light', 'calm', 'paper', 'forced'], defaults.theme),
      contrast: pick(parsed.contrast, ['auto', 'standard', 'high'], defaults.contrast),
      colorVision: pick(parsed.colorVision, ['standard', 'safe', 'monochrome'], defaults.colorVision),
      pointer: pick(parsed.pointer, ['standard', 'large'], defaults.pointer),
      textSpacing: pick(parsed.textSpacing, ['standard', 'loose'], defaults.textSpacing),
      font: pick(parsed.font, ['standard', 'legible'], defaults.font),
      motion: pick(parsed.motion, ['system', 'reduced'], defaults.motion),
      transparency: pick(parsed.transparency, ['standard', 'reduced'], defaults.transparency),
      density: pick(parsed.density, ['standard', 'focus'], defaults.density),
      reading: pick(parsed.reading, ['standard', 'assist'], defaults.reading),
      focus: pick(parsed.focus, ['standard', 'strong'], defaults.focus),
      scale: pick(parsed.scale, ['100', '112', '125', '150'], defaults.scale),
      shortcuts: pick(parsed.shortcuts, ['off', 'on'], defaults.shortcuts),
      assistiveTech: pick(parsed.assistiveTech, ['standard', 'screen-reader'], defaults.assistiveTech),
      captions: pick(parsed.captions, ['off', 'on'], defaults.captions),
    }
  }

  function read() {
    try {
      var raw = window.localStorage.getItem(key) || window.localStorage.getItem(legacyKey)
      return raw ? sanitize(JSON.parse(raw)) : defaults
    } catch (_) {
      return defaults
    }
  }

  function media(query) {
    try {
      return window.matchMedia && window.matchMedia(query).matches
    } catch (_) {
      return false
    }
  }

  function hasNeed(prefs, need) {
    return prefs.needs.indexOf(need) !== -1
  }

  function hasAnyNeed(prefs, list) {
    for (var i = 0; i < list.length; i++) {
      if (hasNeed(prefs, list[i])) return true
    }
    return false
  }

  var scaleRank = { 100: 0, 112: 1, 125: 2, 150: 3 }
  function atLeastScale(current, minimum) {
    return scaleRank[current] >= scaleRank[minimum] ? current : minimum
  }

  function resolveTheme(prefs, system) {
    if (prefs.theme !== 'auto') return prefs.theme
    if (system.forcedColors || hasAnyNeed(prefs, ['lowVision', 'blindAssistive', 'noColor'])) return 'forced'
    if (hasAnyNeed(prefs, ['vestibular', 'photosensitive', 'attention', 'sensory'])) return 'calm'
    if (hasAnyNeed(prefs, ['cognitive', 'dyslexia'])) return 'paper'
    return 'dark'
  }

  function resolveContrast(prefs, system) {
    if (
      prefs.contrast === 'high' ||
      system.highContrast ||
      system.forcedColors ||
      hasAnyNeed(prefs, ['lowVision', 'blindAssistive', 'colorVision', 'noColor'])
    ) {
      return 'high'
    }
    return 'standard'
  }

  function resolve(prefs, system) {
    var next = {
      needs: prefs.needs,
      theme: resolveTheme(prefs, system),
      contrast: resolveContrast(prefs, system),
      colorVision: prefs.colorVision,
      pointer: prefs.pointer,
      textSpacing: prefs.textSpacing,
      font: prefs.font,
      motion: prefs.motion === 'reduced' || system.reducedMotion ? 'reduced' : 'standard',
      transparency: prefs.transparency,
      density: prefs.density,
      reading: prefs.reading,
      focus: prefs.focus,
      scale: prefs.scale,
      shortcuts: prefs.shortcuts,
      assistiveTech: prefs.assistiveTech,
      captions: prefs.captions,
    }
    if (hasNeed(prefs, 'lowVision')) {
      next.scale = atLeastScale(next.scale, '125')
      next.pointer = 'large'
      next.focus = 'strong'
    }
    if (hasNeed(prefs, 'blindAssistive')) {
      next.assistiveTech = 'screen-reader'
      next.density = 'focus'
      next.focus = 'strong'
      next.shortcuts = 'off'
    }
    if (hasNeed(prefs, 'colorVision')) {
      next.colorVision = next.colorVision === 'monochrome' ? 'monochrome' : 'safe'
      next.focus = 'strong'
    }
    if (hasNeed(prefs, 'noColor')) {
      next.colorVision = 'monochrome'
      next.focus = 'strong'
    }
    if (hasAnyNeed(prefs, ['vestibular', 'photosensitive', 'attention', 'sensory'])) {
      next.motion = 'reduced'
      next.transparency = 'reduced'
    }
    if (hasNeed(prefs, 'motor')) {
      next.pointer = 'large'
      next.focus = 'strong'
      next.shortcuts = 'off'
    }
    if (hasNeed(prefs, 'cognitive')) {
      next.density = 'focus'
      next.reading = 'assist'
      next.textSpacing = 'loose'
      next.shortcuts = 'off'
    }
    if (hasNeed(prefs, 'dyslexia')) {
      next.font = 'legible'
      next.textSpacing = 'loose'
      next.reading = 'assist'
      next.scale = atLeastScale(next.scale, '112')
    }
    if (hasNeed(prefs, 'auditory')) next.captions = 'on'
    if (hasNeed(prefs, 'speech')) next.shortcuts = 'off'
    return next
  }

  var colorKeys = [
    's0',
    's1',
    's2',
    's3',
    'bd',
    'bd2',
    'field',
    'tx',
    'tx2',
    'tx3',
    'tx4',
    'ok',
    'warn',
    'bad',
    'info',
    'neu',
    'ok-bg',
    'bad-bg',
  ]
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
    neu: 'oklch(0.405 0.010 260)',
    'ok-bg': 'oklch(0.910 0.030 150)',
    'bad-bg': 'oklch(0.920 0.035 25)',
  }
  var calmColors = {
    s0: 'oklch(0.200 0.012 180)',
    s1: 'oklch(0.235 0.012 180)',
    s2: 'oklch(0.275 0.014 180)',
    s3: 'oklch(0.320 0.015 180)',
    bd: 'oklch(0.390 0.012 180)',
    bd2: 'oklch(0.500 0.014 180)',
    field: 'oklch(0.610 0.016 180)',
    tx: 'oklch(0.930 0.006 170)',
    tx2: 'oklch(0.760 0.008 170)',
    tx3: 'oklch(0.700 0.009 170)',
    tx4: 'oklch(0.640 0.010 170)',
    ok: 'oklch(0.760 0.090 155)',
    warn: 'oklch(0.800 0.090 80)',
    bad: 'oklch(0.760 0.105 25)',
    info: 'oklch(0.780 0.070 220)',
    neu: 'oklch(0.700 0.009 170)',
    'ok-bg': 'oklch(0.310 0.032 155)',
    'bad-bg': 'oklch(0.300 0.035 25)',
  }
  var paperColors = {
    s0: 'oklch(0.970 0.006 95)',
    s1: 'oklch(0.945 0.008 95)',
    s2: 'oklch(0.910 0.010 95)',
    s3: 'oklch(0.860 0.012 95)',
    bd: 'oklch(0.720 0.014 95)',
    bd2: 'oklch(0.560 0.016 95)',
    field: 'oklch(0.410 0.018 95)',
    tx: 'oklch(0.160 0.012 80)',
    tx2: 'oklch(0.290 0.014 80)',
    tx3: 'oklch(0.380 0.014 80)',
    tx4: 'oklch(0.455 0.014 80)',
    ok: 'oklch(0.400 0.120 150)',
    warn: 'oklch(0.455 0.115 78)',
    bad: 'oklch(0.455 0.160 25)',
    info: 'oklch(0.390 0.110 240)',
    neu: 'oklch(0.380 0.014 80)',
    'ok-bg': 'oklch(0.900 0.032 150)',
    'bad-bg': 'oklch(0.905 0.036 25)',
  }
  var forcedColors = {
    s0: '#000000',
    s1: '#050505',
    s2: '#101010',
    s3: '#1a1a1a',
    bd: '#9f9f9f',
    bd2: '#ffffff',
    field: '#ffffff',
    tx: '#ffffff',
    tx2: '#f2f2f2',
    tx3: '#e6e6e6',
    tx4: '#d8d8d8',
    ok: '#ffffff',
    warn: '#ffff00',
    bad: '#ff6b6b',
    info: '#66ccff',
    neu: '#e6e6e6',
    'ok-bg': '#111111',
    'bad-bg': '#220000',
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
  var colorSafeColors = {
    ok: '#009e73',
    warn: '#e69f00',
    bad: '#d55e00',
    info: '#0072b2',
  }
  var monochromeColors = {
    ok: 'var(--color-tx)',
    warn: 'var(--color-tx2)',
    bad: 'var(--color-tx3)',
    info: 'var(--color-tx2)',
  }

  function assign(target, source) {
    Object.keys(source).forEach(function (name) {
      target[name] = source[name]
    })
  }

  function applyColors(root, prefs) {
    for (var i = 0; i < colorKeys.length; i++) root.style.removeProperty('--color-' + colorKeys[i])
    var colors = {}
    if (prefs.theme === 'light') assign(colors, lightColors)
    if (prefs.theme === 'calm') assign(colors, calmColors)
    if (prefs.theme === 'paper') assign(colors, paperColors)
    if (prefs.theme === 'forced') assign(colors, forcedColors)
    if (prefs.contrast === 'high' && prefs.theme === 'dark') assign(colors, highColors)
    if (prefs.contrast === 'high' && prefs.theme === 'light') assign(colors, lightHighColors)
    if (prefs.colorVision === 'safe') assign(colors, colorSafeColors)
    if (prefs.colorVision === 'monochrome') assign(colors, monochromeColors)
    Object.keys(colors).forEach(function (name) {
      root.style.setProperty('--color-' + name, colors[name])
    })
  }

  var system = {
    reducedMotion: media('(prefers-reduced-motion: reduce)'),
    highContrast: media('(prefers-contrast: more)'),
    forcedColors: media('(forced-colors: active)'),
  }
  var resolved = resolve(read(), system)
  var root = document.documentElement
  root.setAttribute('data-theme', resolved.theme)
  root.setAttribute('data-contrast', resolved.contrast)
  root.setAttribute('data-color-vision', resolved.colorVision)
  root.setAttribute('data-pointer', resolved.pointer)
  root.setAttribute('data-text-spacing', resolved.textSpacing)
  root.setAttribute('data-font', resolved.font)
  root.setAttribute('data-motion', resolved.motion)
  root.setAttribute('data-transparency', resolved.transparency)
  root.setAttribute('data-density', resolved.density)
  root.setAttribute('data-reading', resolved.reading)
  root.setAttribute('data-focus', resolved.focus)
  root.setAttribute('data-scale', resolved.scale)
  root.setAttribute('data-shortcuts', resolved.shortcuts)
  root.setAttribute('data-assistive-tech', resolved.assistiveTech)
  root.setAttribute('data-captions', resolved.captions)
  root.setAttribute('data-needs', resolved.needs.join(' '))
  applyColors(root, resolved)
  root.style.colorScheme = resolved.theme === 'light' || resolved.theme === 'paper' ? 'light' : 'dark'
})()
