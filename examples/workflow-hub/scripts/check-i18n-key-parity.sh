#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 83 Phase B — Workflow Hub i18n gate: key parity. Every SOURCE (fr)
# message key exists in every other locale catalog. Guards against a target .po
# drifting out of key-sync with the source. Re-derived from the Factory
# Operator's gate (`check-i18n-key-parity.sh`) — NOT imported. Checks msgid
# PRESENCE only, never msgstr CONTENT (translation completeness is a separate
# concern; empty msgstr is allowed by the source-fallback policy). Target
# locales are derived by globbing, so a new catalog is covered automatically.
set -euo pipefail
cd "$(dirname "$0")/.."

DIR="i18n/locales"
SOURCE="fr"
# REQUIRED locales are asserted explicitly so the gate is not vacuous: deleting
# en.po/ar.po must FAIL (glob-only targets would silently accept zero locales).
REQUIRED_LOCALES="fr en ar"
for loc in $REQUIRED_LOCALES; do
  if [ ! -f "$DIR/$loc.po" ]; then
    echo "check-i18n-key-parity: FAILED (required locale catalog $DIR/$loc.po missing)"
    exit 1
  fi
done

node --input-type=module <<'NODE'
import { readFileSync, readdirSync } from 'node:fs'

const DIR = 'i18n/locales'
const SOURCE = 'fr'
const PREFIX = 'check-i18n-key-parity'

function msgids(text) {
  const ids = new Set()
  let cur = null
  for (const line of text.split(/\r?\n/)) {
    if (/^#/.test(line)) { cur = null; continue }
    const m = line.match(/^msgid\s+"(.*)"\s*$/)
    if (m) { cur = { v: m[1] }; continue }
    const c = line.match(/^"(.*)"\s*$/)
    if (c && cur) { cur.v += c[1]; continue }
    if (/^msgstr/.test(line)) {
      if (cur && cur.v !== '') ids.add(cur.v)
      cur = null
    }
  }
  return ids
}

const source = msgids(readFileSync(`${DIR}/${SOURCE}.po`, 'utf8'))

// NOT vacuous (Codex S83-B): an empty/degenerate source catalog (headers only,
// or a parser break) yields zero keys, so every target trivially "has all keys".
// Require the Phase B baseline keys so emptying fr.po FAILS instead of passing.
const REQUIRED_KEYS = [
  'hub.title', 'hub.tagline', 'hub.a11y.heading', 'hub.a11y.resolved',
  'hub.mode.heading', 'hub.dir.heading', 'hub.dir.toRtl', 'hub.dir.toLtr', 'hub.skip',
]
const sourceMissing = REQUIRED_KEYS.filter((k) => !source.has(k))
if (source.size === 0 || sourceMissing.length) {
  console.log(`${PREFIX}: source catalog ${SOURCE}.po lacks baseline keys: ${sourceMissing.join(', ') || '(empty)'}`)
  process.exit(1)
}

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
  console.log(`${PREFIX}: ${missing.length} source key(s) missing from a locale catalog:`)
  for (const { loc, id } of missing) console.log(`  ${loc}.po: "${id}"`)
  process.exit(1)
}
NODE

echo "check-i18n-key-parity: clean (every fr key present in each locale)"
