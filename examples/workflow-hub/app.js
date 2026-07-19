// SPDX-License-Identifier: AGPL-3.0-or-later
/*
 * Sprint 83 Phase B — Workflow Hub boot (amorce, vanilla).
 *
 * Renders the resolved-axes readout from the document's `data-*` attributes
 * (needs-free — there is no `data-needs` to read), the mode-guide contract
 * from the embedded JSON, and a LTR/RTL toggle (i18n foundation). Refreshes
 * the readout when a11y.js applies a live profile.
 */
;(function () {
  // The 15 effect axes and the `data-*` attribute each maps to.
  var AXES = [
    ['theme', 'data-theme'],
    ['contrast', 'data-contrast'],
    ['colorVision', 'data-color-vision'],
    ['pointer', 'data-pointer'],
    ['textSpacing', 'data-text-spacing'],
    ['font', 'data-font'],
    ['motion', 'data-motion'],
    ['transparency', 'data-transparency'],
    ['density', 'data-density'],
    ['reading', 'data-reading'],
    ['focus', 'data-focus'],
    ['scale', 'data-scale'],
    ['shortcuts', 'data-shortcuts'],
    ['assistiveTech', 'data-assistive-tech'],
    ['captions', 'data-captions'],
  ]

  function renderResolvedAxes() {
    var dl = document.getElementById('resolved-axes')
    if (!dl) return
    var root = document.documentElement
    var html = ''
    for (var i = 0; i < AXES.length; i++) {
      var label = AXES[i][0]
      var value = root.getAttribute(AXES[i][1]) || '—'
      html +=
        '<div class="hub-axis"><dt>' +
        label +
        '</dt><dd>' +
        value +
        '</dd></div>'
    }
    dl.innerHTML = html
  }

  function renderProjections() {
    var ul = document.getElementById('mode-projections')
    var raw = document.getElementById('mode-guide-data')
    if (!ul || !raw) return
    var data
    try {
      data = JSON.parse(raw.textContent || '{}')
    } catch (_) {
      return
    }
    var projections = (data && data.projections) || []
    var html = ''
    for (var i = 0; i < projections.length; i++) {
      var p = projections[i]
      html +=
        '<li class="hub-projection"><h3>' +
        p.label +
        '</h3><p>' +
        p.summary +
        '</p></li>'
    }
    ul.innerHTML = html
  }

  function wireDirToggle() {
    var btn = document.getElementById('dir-toggle')
    if (!btn) return
    btn.addEventListener('click', function () {
      var root = document.documentElement
      var next = root.getAttribute('dir') === 'rtl' ? 'ltr' : 'rtl'
      root.setAttribute('dir', next)
      btn.textContent = next === 'rtl' ? 'Repasser en LTR' : 'Passer en RTL'
    })
  }

  renderResolvedAxes()
  renderProjections()
  wireDirToggle()

  // Refresh the readout when the live profile is applied by a11y.js.
  window.addEventListener('sbfb-hub:a11y-applied', renderResolvedAxes)
})()
