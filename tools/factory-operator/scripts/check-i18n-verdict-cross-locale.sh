#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 (front rapid-add) — i18n gate (6): no POSITIVE-VERDICT word in ANY
# locale's translation VALUES. Cross-locale twin of scan-front-discipline.sh.
#
# The cardinal invariant — the Operator RESTITUTES a verdict, never asserts one
# — leaks through translation: scan-front-discipline.sh (gate 5) only scans the
# FR-source literals in src/*.tsx, so a translator could reintroduce
# "PASS"/"APPROVED"/"通过"/"معتمد" in a non-FR .po msgstr and bypass it. This
# gate scans the msgstr VALUES of every src/i18n/locales/*.po.
#
# Per-script matching (adversarially verified — a single \p{L} boundary is
# WRONG for CJK, where Chinese has no word delimiters so the boundary only
# fires at msgstr start):
#   - Latin (fr/es + the universal SCREAMING-case EN badges): case-sensitive,
#     capitalised, Unicode-letter boundary — mirrors gate 5, so a lowercase
#     verb ("l'agent approuve le diff") never trips.
#   - CJK (zh): SUBSTRING match with a negator-prefix exclusion (不未没… so the
#     FAIL form 不合格 stays clean while 测试通过 "test passed" is caught).
#   - Arabic (ar): boundary match, BEST-EFFORT — a clitic-attached form (ونجح)
#     can slip; regex cannot be both correct-positive and correct-negative for
#     connected Arabic, so human/agent review is the real backstop there.
#
# An UNGUARDED locale (a .po whose language has no word list) FAILS loudly
# rather than silently degrading to PASS-only — load-bearing for "i18n toutes
# langues". Scans msgstr only (skips the header, # comments, obsolete #~).
set -euo pipefail
cd "$(dirname "$0")/.."

DIR="src/i18n/locales"
if [ ! -d "$DIR" ] || [ -z "$(ls -1 "$DIR"/*.po 2>/dev/null || true)" ]; then
  echo "check-i18n-verdict-cross-locale: FAILED (no .po catalogs found under $DIR)"
  exit 1
fi

# The parsing + per-script matching is far more robust in Node than in grep
# (CJK substring + negator exclusion, Unicode-letter boundaries). Node exits 1
# on any violation, which `set -e` propagates as the gate failure.
node --input-type=module <<'NODE'
import { readFileSync, readdirSync } from 'node:fs'

const DIR = 'src/i18n/locales'
const PREFIX = 'check-i18n-verdict-cross-locale'

// Locales we know how to guard. A scanned .po whose language is NOT here fails
// the gate, forcing an explicit word-list decision for any new locale.
const GUARDED = new Set(['fr', 'en', 'es', 'ar', 'zh'])

// Universal SCREAMING-case Latin badges — language-agnostic (a pasted English
// "APPROVED" in any locale's msgstr is a verdict). ASCII, case-sensitive.
const UNIVERSAL = ['PASS', 'PASSED', 'APPROVED', 'VERIFIED', 'VALIDATED', 'SUCCEEDED', 'SUCCESS']

// Per-locale positive-verdict words + the script's matching strategy. The
// neutral status labels the front DOES render (tenue/met, bloquant, …) are
// deliberately absent, so they never trip.
const PER_LOCALE = {
  fr: { script: 'latin', words: ['Réussi', 'Réussie', 'Réussite', 'Validé', 'Validée', 'Vérifié', 'Vérifiée', 'Verifie', 'Verifiee', 'Approuvé', 'Approuvée', 'Approuve'] },
  en: { script: 'latin', words: [] }, // covered by UNIVERSAL
  es: { script: 'latin', words: ['Aprobado', 'Aprobada', 'Validado', 'Validada', 'Verificado', 'Verificada', 'Superado', 'Superada', 'Exitoso', 'Exitosa'] },
  ar: { script: 'arabic', words: ['نجح', 'ناجح', 'نجاح', 'اجتاز', 'مقبول', 'معتمد', 'تم التحقق', 'مصادق عليه', 'موافق عليه'] },
  zh: { script: 'cjk', words: ['通过', '已通过', '验证通过', '已验证', '批准', '已批准', '核准', '成功', '合格', '通過', '已驗證', '核準'] },
}
const CJK_NEGATORS = '不未没否無毋勿別别'

const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

// Parse a .po → its translation VALUES (msgstr). Skips comment lines (# and
// obsolete #~), concatenates multi-line continuations, and drops the header
// entry (its msgstr carries Content-Type/Language metadata).
function parsePo(text) {
  const values = []
  let cur = null
  for (const line of text.split(/\r?\n/)) {
    if (/^#/.test(line)) { cur = null; continue }
    const m = line.match(/^msgstr\s+"(.*)"\s*$/)
    if (m) { cur = { v: m[1] }; values.push(cur); continue }
    const c = line.match(/^"(.*)"\s*$/) // continuation of the current msgstr
    if (c && cur) { cur.v += c[1]; continue }
    if (/^msgid/.test(line) || line.trim() === '') cur = null
  }
  return values
    .map((x) => x.v)
    .filter((v) => v && !/MIME-Version|Content-Type|X-Generator|POT-Creation-Date|Language:/.test(v))
}

function langOf(text, file) {
  const m = text.match(/Language:\s*([A-Za-z]+)/)
  return (m ? m[1] : file.slice(0, -3)).toLowerCase()
}

function hits(value, locale) {
  const out = []
  for (const w of UNIVERSAL) if (new RegExp(`\\b${esc(w)}\\b`).test(value)) out.push(w)
  const conf = PER_LOCALE[locale]
  if (!conf) return out
  if (conf.script === 'latin' || conf.script === 'arabic') {
    // case-sensitive, Unicode-letter boundary (mirrors gate 5 capitalised-first)
    for (const w of conf.words) if (new RegExp(`(?<!\\p{L})${esc(w)}(?!\\p{L})`, 'u').test(value)) out.push(w)
  } else if (conf.script === 'cjk') {
    // substring, skipping a hit immediately preceded by a negator (不合格 = FAILED)
    for (const w of conf.words) {
      let i = value.indexOf(w)
      while (i >= 0) {
        const prev = value[i - 1]
        if (!(prev && CJK_NEGATORS.includes(prev))) { out.push(w); break }
        i = value.indexOf(w, i + 1)
      }
    }
  }
  return out
}

const violations = []
const unguarded = []
for (const f of readdirSync(DIR).filter((f) => f.endsWith('.po'))) {
  const text = readFileSync(`${DIR}/${f}`, 'utf8')
  const loc = langOf(text, f)
  if (!GUARDED.has(loc)) { unguarded.push(f); continue }
  for (const v of parsePo(text)) for (const w of hits(v, loc)) violations.push({ f, w, v })
}

if (unguarded.length) {
  console.log(`${PREFIX}: unguarded locale(s) — add a verdict word-list before shipping: ${unguarded.join(', ')}`)
}
if (violations.length) {
  console.log(`${PREFIX}: forbidden positive-verdict word in a translation value (the front restitutes a verdict, never asserts one):`)
  for (const { f, w, v } of violations) console.log(`  ${f}: "${w}"  in  "${v}"`)
}
if (unguarded.length || violations.length) process.exit(1)
NODE

echo "check-i18n-verdict-cross-locale: clean (no verdict word in any locale's translations)"
