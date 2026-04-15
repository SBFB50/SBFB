#!/usr/bin/env bash
#
# .claude/hooks/tdd-guard-wrapper.sh
#
# Wrapper gracieux pour `tdd-guard` (nizos/tdd-guard). Se contente de
# no-op (exit 0) si le binaire n'est pas installe.
#
# Permet d'enregistrer ce wrapper dans .claude/settings.json sans casser
# les sessions ou TDD Guard n'est pas installe. Le user qui veut
# l'activer fait :
#   npm install -g tdd-guard
#   cargo install tdd-guard-rust       # pour Rust
#   pip install tdd-guard-pytest       # pour Python
# Voir docs/claude/TOOLING.md §5.3.

set -eo pipefail

if ! command -v tdd-guard >/dev/null 2>&1; then
  # Non installe, no-op silent (fail-open)
  exit 0
fi

# Passe stdin/stdout directement a tdd-guard avec meme exit code
exec tdd-guard
