#!/usr/bin/env bash
#
# nexus/.claude/hooks/verify-on-write.sh
#
# PostToolUse hook (matcher Edit|Write) qui re-lance le linter scope au fichier
# modifie. Feedback rapide (<5s) pour catcher les erreurs avant accumulation.
#
# Input (stdin) : JSON Claude Code avec tool_input.file_path
# Output : exit 0 si clean, exit 2 si lint fail (bloque avec message d'erreur)
#
# Perimetre :
#   .rs        -> cargo clippy -p <crate detecte> --lib --tests -- -D warnings
#   .py        -> uv run ruff check <file>
#   .ts/.tsx   -> cd web && npx eslint <file>
#   autres     -> exit 0 (ignore)

set -eo pipefail

INPUT=$(cat)

# Extract tool_input.file_path from JSON. Try jq first (fast), fall back
# to python3 (ubiquitous). If neither -> exit 0 silently (fail-open).
if command -v jq >/dev/null 2>&1; then
  FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)
elif command -v python3 >/dev/null 2>&1; then
  FILE_PATH=$(echo "$INPUT" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("tool_input",{}).get("file_path","") or "")' 2>/dev/null || true)
else
  exit 0
fi

[[ -z "$FILE_PATH" ]] && exit 0

REPO_ROOT=$(pwd)
if [[ ! -f "$REPO_ROOT/Cargo.toml" ]] || [[ ! -d "$REPO_ROOT/crates/nexus-core-rs" ]]; then
  exit 0
fi

case "$FILE_PATH" in
  /*|[A-Za-z]:*) REL_PATH="${FILE_PATH#$REPO_ROOT/}" ;;
  *) REL_PATH="$FILE_PATH" ;;
esac
REL_PATH="${REL_PATH//\\//}"
REL_PATH="${REL_PATH#$REPO_ROOT/}"

case "$REL_PATH" in
  .planning/*|docs/*|target/*|node_modules/*|.venv/*|dist/*|build/*|.git/*|.claude/*)
    exit 0 ;;
esac

EXT="${REL_PATH##*.}"

case "$EXT" in
  rs)
    if [[ "$REL_PATH" =~ ^crates/([^/]+)/ ]]; then
      CRATE="${BASH_REMATCH[1]}"
      echo "[verify-on-write] rs: cargo clippy -p $CRATE" >&2
      if ! cargo clippy -p "$CRATE" --lib --tests --locked -- -D warnings 2>&1 | tail -50 >&2; then
        echo "[verify-on-write] BLOCK: clippy failed on $CRATE ($REL_PATH)" >&2
        exit 2
      fi
    else
      exit 0
    fi
    ;;

  py)
    if ! command -v uv >/dev/null 2>&1; then
      exit 0
    fi
    echo "[verify-on-write] py: ruff check $REL_PATH" >&2
    if ! uv run ruff check "$REL_PATH" 2>&1 | tail -30 >&2; then
      echo "[verify-on-write] BLOCK: ruff failed on $REL_PATH" >&2
      exit 2
    fi
    ;;

  ts|tsx|js|jsx)
    if [[ "$REL_PATH" != web/* ]]; then
      exit 0
    fi
    WEB_REL="${REL_PATH#web/}"
    if [[ ! -d "$REPO_ROOT/web/node_modules" ]]; then
      exit 0
    fi
    echo "[verify-on-write] ts: eslint $WEB_REL" >&2
    if ! (cd "$REPO_ROOT/web" && npx --no-install eslint "$WEB_REL" 2>&1 | tail -30 >&2); then
      echo "[verify-on-write] BLOCK: eslint failed on $REL_PATH" >&2
      exit 2
    fi
    ;;

  *)
    exit 0
    ;;
esac

# ------- Semgrep SBFB (optionnel) -------
# Si semgrep est dans le PATH et que .semgrep/sbfb.yml existe, scan le
# fichier modifie avec les regles SBFB custom. Seulement apres que le
# linter natif (clippy/ruff/eslint) est passe — on ne veut pas scanner
# du code qui ne compile meme pas.
#
# Exit 2 si Semgrep trouve des findings WARNING+ (ERROR, CRITICAL).
# INFO est non-bloquant (preventive, cf. sbfb-iroh-endpoint-pin).
if command -v semgrep >/dev/null 2>&1 && [ -f "$REPO_ROOT/.semgrep/sbfb.yml" ]; then
  # Scope a l'extension Semgrep support + au fichier specifique.
  # Semgrep print les findings sur stdout/stderr — on laisse tel quel
  # pour que l'agent Claude voie le detail.
  SEMGREP_OUTPUT=$(semgrep --config "$REPO_ROOT/.semgrep/sbfb.yml" \
    --severity WARNING --severity ERROR \
    --error \
    --quiet \
    "$REPO_ROOT/$REL_PATH" 2>&1 || true)

  if [ -n "$SEMGREP_OUTPUT" ] && echo "$SEMGREP_OUTPUT" | grep -qE 'findings|❯❱'; then
    echo "[verify-on-write] semgrep findings on $REL_PATH:" >&2
    echo "$SEMGREP_OUTPUT" | tail -30 >&2
    # Exit 2 seulement si findings bloquants (WARNING+ severity)
    echo "$SEMGREP_OUTPUT" | grep -qE 'Blocking|error' && {
      echo "[verify-on-write] BLOCK: semgrep SBFB rules flagged on $REL_PATH" >&2
      exit 2
    }
  fi
fi

exit 0
