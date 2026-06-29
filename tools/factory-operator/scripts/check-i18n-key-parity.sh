#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 (front rapid-add) — i18n gate (7): every SOURCE (fr) message key
# exists in every other locale catalog. Guards against a target .po drifting
# out of key-sync with the source (a removed/renamed key that would silently
# render the source fallback forever, or a translator hand-editing a catalog).
#
# Checks msgid PRESENCE only, NEVER msgstr CONTENT — translation completeness is
# a separate, deferred concern: the source-fallback policy (lingui.config.ts
# §1.9) allows an empty msgstr, so es/ar/zh being untranslated is NOT a failure.
#
# Source locale = fr (lingui.config.ts sourceLocale). Target locales are DERIVED
# by globbing the catalog dir (not hardcoded), so a newly added locale catalog
# is covered automatically and a dropped one can't leave a stale hardcoded entry.
# KNOWN LIMITATION: keys on msgid alone; if Lingui `context` (msgctxt) macros are
# ever introduced, switch the key to `msgctxt\x04msgid` (Lingui emits no msgctxt
# today).
set -euo pipefail
cd "$(dirname "$0")/.."

DIR="src/i18n/locales"
SOURCE="fr"
if [ ! -f "$DIR/$SOURCE.po" ]; then
  echo "check-i18n-key-parity: FAILED (source catalog $DIR/$SOURCE.po missing)"
  exit 1
fi

node --input-type=module <<'NODE'
import { readFileSync, readdirSync } from 'node:fs'

const DIR = 'src/i18n/locales'
const SOURCE = 'fr'
const PREFIX = 'check-i18n-key-parity'

// Extract the set of msgids: skip comment lines (# and obsolete #~) and the
// header entry (msgid ""), concatenate multi-line msgid continuations.
function msgids(text) {
  const ids = new Set()
  let cur = null
  for (const line of text.split(/\r?\n/)) {
    if (/^#/.test(line)) { cur = null; continue }
    const m = line.match(/^msgid\s+"(.*)"\s*$/)
    if (m) { cur = { v: m[1] }; continue }
    const c = line.match(/^"(.*)"\s*$/) // continuation of the current msgid
    if (c && cur) { cur.v += c[1]; continue }
    if (/^msgstr/.test(line)) {
      if (cur && cur.v !== '') ids.add(cur.v) // drop the header (msgid "")
      cur = null
    }
  }
  return ids
}

const source = msgids(readFileSync(`${DIR}/${SOURCE}.po`, 'utf8'))
const targets = readdirSync(DIR)
  .filter((f) => f.endsWith('.po'))
  .map((f) => f.slice(0, -3))
  .filter((l) => l !== SOURCE)

const missing = []
for (const loc of targets) {
  const ids = msgids(readFileSync(`${DIR}/${loc}.po`, 'utf8'))
  for (const id of source) if (!ids.has(id)) missing.push({ loc, id })
}

if (missing.length) {
  console.log(`${PREFIX}: ${missing.length} source key(s) missing from a locale catalog (run \`npm run i18n:extract\`):`)
  for (const { loc, id } of missing) console.log(`  ${loc}.po: "${id}"`)
  process.exit(1)
}
NODE

echo "check-i18n-key-parity: clean (every fr key present in each locale)"
