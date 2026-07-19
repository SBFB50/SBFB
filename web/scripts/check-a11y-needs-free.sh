#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 83 Phase B — a11y "profile sans needs[]" static DOM-WRITE guard.
#
# SCOPE OF THIS GATE (Codex S83-B): it is a shallow DOM-write guard, NOT the
# whole proof of the cardinal invariant. The AUTHORITATIVE proof that the
# host->app projection is needs-free is the TYPE SYSTEM (the Rust
# `ResolvedAccessibilityProfile` and the TS interface have no `needs` field —
# compile_fail doctest + `debug_output_carries_no_need`) plus the strict Hub
# receiver + the web unit tests. This grep only catches the most common DOM
# regression: a `data-needs` WRITE on a needs-free surface (the Workflow Hub
# archive + the shared web a11y module) — `setAttribute(...data-needs...)`
# (any quote incl. backtick, any whitespace), `dataset.needs`, `dataset[...needs]`,
# or an HTML/template `data-needs=` attribute. It does NOT (and cannot, being a
# grep) cover a leak via postMessage/log/prompt/trace of `prefs.needs` — those
# are guarded by the type system (the projection has no needs) and the tests.
# SCOPED to the needs-free surfaces — the Factory Operator legitimately writes
# `data-needs` to its OWN local DOM (a first-party privileged host tool, under
# its own CSP — not the sandbox's), so it is OUT of scope. Prose that merely mentions `data-needs` (comments, test assertions) is
# not a write and does not trip.
#
# NOT fail-open: the scope paths MUST exist, and a `grep` error (exit >1) FAILS
# the gate rather than silently passing via `|| true`. BEST-EFFORT and
# line-oriented: it catches the common single-line DOM regression but NOT a
# split-line `setAttribute(` \n `'data-needs'` nor a computed key — the
# AUTHORITATIVE guarantee is the TYPE SYSTEM (no `needs` field) + the tests, not
# this grep. Exit 0 = clean, 1 = a leak or an internal error.
set -uo pipefail
cd "$(dirname "$0")/../.."

SCOPE_DIRS="examples/workflow-hub web/src/a11y"

for d in $SCOPE_DIRS; do
  if [ ! -e "$d" ]; then
    echo "check-a11y-needs-free: FAILED (needs-free scope path missing: $d)"
    exit 1
  fi
done

# A real DOM write / dataset access — NOT prose. Covers any quote (incl.
# backtick) + whitespace inside setAttribute, dot + bracket dataset access,
# and the HTML/template attribute form.
PATTERN='setAttribute\([^)]*data-needs|dataset\.needs|dataset\[[^]]*needs|data-needs[[:space:]]*='

hits="$(grep -rnE "$PATTERN" $SCOPE_DIRS 2>/dev/null)"
rc=$?
if [ "$rc" -gt 1 ]; then
  echo "check-a11y-needs-free: FAILED (grep error rc=$rc)"
  exit 1
fi
if [ "$rc" -eq 0 ] && [ -n "$hits" ]; then
  echo "check-a11y-needs-free: a needs-free surface WRITES data-needs (the cause must never reach the DOM):"
  printf '%s\n' "$hits"
  echo "  -> remove the write; the projection carries only the 15 effect axes."
  exit 1
fi

echo "check-a11y-needs-free: clean (Hub + web/src/a11y write no data-needs; projection is needs-free)"
