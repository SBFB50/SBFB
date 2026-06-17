#!/usr/bin/env bash
#
# .claude/hooks/session-start-autoinstall.sh
#
# SessionStart hook (matcher startup|resume) qui :
#   1. Detecte les composants optionnels absents (Trail of Bits, Semgrep)
#      et les signale a Claude via additionalContext
#   2. NE FAIT PAS d'install externe (npm/pip/cargo install) sans
#      demande explicite — ces commandes peuvent etre longues ou
#      echouer, mauvais user experience au SessionStart
#
# Idempotent : utilise un marker file pour ne signaler qu'une fois.
# Rapide : < 100ms typique (sauf le premier run qui installe le hook git).
# Fail-safe : toute erreur -> silent no-op, la session demarre normalement.

set -eo pipefail

REPO_ROOT=$(pwd)

# Scope nexus only
if [ ! -f "$REPO_ROOT/Cargo.toml" ] || [ ! -d "$REPO_ROOT/crates/nexus-core-rs" ]; then
  exit 0
fi

MARKER="$REPO_ROOT/.claude/_autoinstall_signaled.marker"
MISSING_COMPONENTS=()

# ------- 1. Detection composants optionnels -------

# jq OR python3 (pour les hooks JSON parse)
if ! command -v jq >/dev/null 2>&1 && ! command -v python3 >/dev/null 2>&1; then
  MISSING_COMPONENTS+=("jq OU python3 (requis pour hooks JSON parse)")
fi

# Trail of Bits skills (user-level)
if [ ! -d "$HOME/.claude/skills/trailofbits" ]; then
  MISSING_COMPONENTS+=("Trail of Bits skills : 'bash scripts/install-claude-tooling.sh' pour cloner")
fi

# Semgrep (pour les regles SBFB)
if ! command -v semgrep >/dev/null 2>&1; then
  MISSING_COMPONENTS+=("semgrep : 'pip install --user semgrep' (regles SBFB dans .semgrep/sbfb.yml ne tourneront pas)")
fi

# TDD Guard (opt-in)
if ! command -v tdd-guard >/dev/null 2>&1; then
  # Pas dans les warnings : c'est opt-in par design
  :
fi

# ------- 2. Emettre additionalContext (UN objet JSON, chaque session) -------

# 2a. Directive bootstrap README — TOUJOURS (chaque startup|resume).
# Une session fraiche n'auto-lit PAS docs/claude/README.md ; le
# additionalContext SessionStart, lui, atteint le modele. On impose donc
# la lecture de la source de verite AVANT toute action, ciblee par
# marqueurs (drift-proof : aucun numero de ligne code en dur).
CTX="[session-start] AVANT toute action (avant tout Read, avant le pre-flight, avant de detecter le cas) : ouvre docs/claude/README.md section 0 puis LIS INTEGRALEMENT le bloc bootstrap section 7.1. Cible la plage par Grep BOOTSTRAP:BEGIN et BOOTSTRAP:END dans docs/claude/README.md -> 2 numeros de ligne -> Read en UN appel (offset=ligne BEGIN, limit=END-BEGIN+5). Tu DOIS voir BOOTSTRAP:END dans ce que tu lis, sinon re-Read avant de continuer. Puis execute le pre-flight section 7.1, detecte le cas A/B/C/D, et RESTITUE en 6 lignes max : cas + signal + prochaine action + regle EXECUTER vs DEMANDER, AVANT de coder ou invoquer un agent. Mode ULTRACODE ON ; orchestration Workflow par etape ; PAS DE SUPERVISEUR (pas de teammate, pas de GO/BLOCK).\n"

# 2b. Avertissement tooling optionnel — une seule fois (marker-gated).
if [ ! -f "$MARKER" ] && [ ${#MISSING_COMPONENTS[@]} -gt 0 ]; then
  CTX="${CTX}[session-start] Composants process tooling manquants (optionnels) :\n"
  for comp in "${MISSING_COMPONENTS[@]}"; do
    CTX="${CTX}  - ${comp}\n"
  done
  CTX="${CTX}[session-start] Install all-in-one : bash scripts/install-claude-tooling.sh\n"
  CTX="${CTX}[session-start] Doc : docs/claude/TOOLING.md\n"
  touch "$MARKER" 2>/dev/null || true
fi

# 2c. Emettre UN objet JSON additionalContext (protocole Claude Code).
if command -v python3 >/dev/null 2>&1; then
  python3 -c "
import json
ctx = '''$CTX'''
print(json.dumps({
    'hookSpecificOutput': {
        'hookEventName': 'SessionStart',
        'additionalContext': ctx
    }
}))
"
else
  # Fallback : print en clair sur stdout (Claude le verra, moins proprement)
  echo -e "$CTX"
fi

exit 0
