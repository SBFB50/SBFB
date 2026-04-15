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

exit 0
