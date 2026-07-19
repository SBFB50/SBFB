#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 83 Phase B — Workflow Hub i18n gate: no POSITIVE-VERDICT word in any
# locale's translation VALUES. Re-derived from the Factory Operator's
# `check-i18n-verdict-cross-locale.sh` — NOT imported. The cardinal invariant
# (a workflow surface RESTITUTES a verdict, never asserts one) must not leak
# through translation. A locale without a word-list FAILS loudly rather than
# degrading to a PASS-only scan. Scans msgstr only.
set -euo pipefail
cd "$(dirname "$0")/.."

DIR="i18n/locales"
if [ ! -d "$DIR" ] || [ -z "$(ls -1 "$DIR"/*.po 2>/dev/null || true)" ]; then
  echo "check-i18n-verdict-cross-locale: FAILED (no .po catalogs found under $DIR)"
  exit 1
fi

node --input-type=module <<'NODE'
import { readFileSync, readdirSync } from 'node:fs'

const DIR = 'i18n/locales'
const PREFIX = 'check-i18n-verdict-cross-locale'

const UNIVERSAL = ["PASS","PASSED","APPROVED","VERIFIED","VALIDATED","SUCCEEDED","SUCCESS"]

const PER_LOCALE = {
  "fr": { "strategy": "boundary", "negators": "", "words": [
    "Réussi","Réussie","Réussite","Validé","Validée","Vérifié","Vérifiée","Approuvé","Approuvée"
  ] },
  "en": { "strategy": "boundary", "negators": "", "words": [] },
  "ar": { "strategy": "boundary", "negators": "", "words": [
    "نجح","ناجح","نجاح","اجتاز","مقبول","معتمد","تم التحقق","مصادق عليه","موافق عليه"
  ] }
}
const GUARDED = new Set(Object.keys(PER_LOCALE))

const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
const BOUNDARY = '[\\p{L}\\p{M}]'

function parsePo(text) {
  const values = []
  let cur = null
  for (const line of text.split(/\r?\n/)) {
    if (/^#/.test(line)) { cur = null; continue }
    const m = line.match(/^msgstr\s+"(.*)"\s*$/)
    if (m) { cur = { v: m[1] }; values.push(cur); continue }
    const c = line.match(/^"(.*)"\s*$/)
    if (c && cur) { cur.v += c[1]; continue }
    if (/^msgid/.test(line) || line.trim() === '') cur = null
  }
  return values
    .map((x) => x.v)
    .filter((v) => v && !/MIME-Version|Content-Type|X-Generator|POT-Creation-Date|Language:/.test(v))
}

function langOf(text, file) {
  const m = text.match(/Language:\s*([A-Za-z0-9-]+)/)
  return (m ? m[1] : file.slice(0, -3)).toLowerCase()
}

function hits(value, locale) {
  const out = []
  // Case-INSENSITIVE (Codex S83-B): APPROVED / Approved / approved all count.
  for (const w of UNIVERSAL) if (new RegExp('\\b' + esc(w) + '\\b', 'i').test(value)) out.push(w)
  const conf = PER_LOCALE[locale]
  if (!conf) return out
  if (conf.strategy === 'boundary') {
    for (const w of conf.words) {
      if (new RegExp('(?<!' + BOUNDARY + ')' + esc(w) + '(?!' + BOUNDARY + ')', 'iu').test(value)) out.push(w)
    }
  } else if (conf.strategy === 'substring') {
    for (const w of conf.words) {
      let i = value.indexOf(w)
      while (i >= 0) {
        const prev = value[i - 1]
        if (!(prev && conf.negators.includes(prev))) { out.push(w); break }
        i = value.indexOf(w, i + 1)
      }
    }
  } else {
    throw new Error('unknown strategy for ' + locale + ': ' + conf.strategy)
  }
  return out
}

const violations = []
const unguarded = []
for (const f of readdirSync(DIR).filter((f) => f.endsWith('.po'))) {
  const text = readFileSync(DIR + '/' + f, 'utf8')
  const loc = langOf(text, f)
  if (!GUARDED.has(loc)) { unguarded.push(f); continue }
  for (const v of parsePo(text)) for (const w of hits(v, loc)) violations.push({ f, w, v })
}

if (unguarded.length) {
  console.log(PREFIX + ': unguarded locale(s) - add a verdict word-list before shipping: ' + unguarded.join(', '))
}
if (violations.length) {
  console.log(PREFIX + ': forbidden positive-verdict word in a translation value:')
  for (const { f, w, v } of violations) console.log('  ' + f + ': "' + w + '"  in  "' + v + '"')
}
if (unguarded.length || violations.length) process.exit(1)
NODE

echo "check-i18n-verdict-cross-locale: clean (no verdict word in any locale's translations)"
