#!/usr/bin/env bash
#
# .claude/hooks/session-start-autoinstall.sh
#
# SessionStart hook (matcher startup|resume) qui :
#   1. Installe automatiquement le git post-commit hook (leger, idempotent)
#   2. Detecte les composants optionnels absents (Trail of Bits, Semgrep,
#      TDD Guard) et les signale a Claude via additionalContext
#   3. NE FAIT PAS d'install externe (npm/pip/cargo install) sans
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

# ------- 1. Git post-commit hook (auto-install, leger) -------
POST_COMMIT_HOOK="$REPO_ROOT/.git/hooks/post-commit"
POST_COMMIT_SOURCE="$REPO_ROOT/.claude/hooks/post-commit-memory.sh"

if [ ! -f "$POST_COMMIT_HOOK" ] && [ -f "$POST_COMMIT_SOURCE" ]; then
  cat > "$POST_COMMIT_HOOK" <<'EOF'
#!/usr/bin/env bash
# Auto-installed by .claude/hooks/session-start-autoinstall.sh
exec bash "$(git rev-parse --show-toplevel)/.claude/hooks/post-commit-memory.sh"
EOF
  chmod +x "$POST_COMMIT_HOOK" 2>/dev/null || true
  AUTOINSTALLED_POSTCOMMIT=1
fi

# ------- 2. Detection composants optionnels -------

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

# ------- 3. Emettre additionalContext -------
# Seulement la premiere fois par session, OU si on vient d'auto-installer
# le git hook (pour signaler ce qu'on a fait)

SHOULD_EMIT=0
[ "${AUTOINSTALLED_POSTCOMMIT:-0}" = "1" ] && SHOULD_EMIT=1
[ ! -f "$MARKER" ] && [ ${#MISSING_COMPONENTS[@]} -gt 0 ] && SHOULD_EMIT=1

if [ "$SHOULD_EMIT" = "1" ]; then
  CTX=""
  if [ "${AUTOINSTALLED_POSTCOMMIT:-0}" = "1" ]; then
    CTX="[session-start] Auto-installed .git/hooks/post-commit delegator (memory tip updater active).\n"
  fi

  if [ ${#MISSING_COMPONENTS[@]} -gt 0 ]; then
    CTX="${CTX}[session-start] Composants process tooling manquants (optionnels) :\n"
    for comp in "${MISSING_COMPONENTS[@]}"; do
      CTX="${CTX}  - ${comp}\n"
    done
    CTX="${CTX}[session-start] Install all-in-one : bash scripts/install-claude-tooling.sh\n"
    CTX="${CTX}[session-start] Doc : docs/claude/TOOLING.md\n"
  fi

  # Format JSON pour additionalContext (protocole Claude Code)
  if command -v python3 >/dev/null 2>&1; then
    python3 -c "
import json, sys
ctx = '''$CTX'''
print(json.dumps({
    'hookSpecificOutput': {
        'hookEventName': 'SessionStart',
        'additionalContext': ctx
    }
}))
"
  else
    # Fallback : print en clair sur stdout (Claude le verra mais moins proprement)
    echo -e "$CTX"
  fi

  # Marquer qu'on a signale (jusqu'a prochain cleanup ou install manuel)
  touch "$MARKER" 2>/dev/null || true
fi

exit 0
