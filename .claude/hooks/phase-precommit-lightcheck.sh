#!/usr/bin/env bash
#
# .claude/hooks/phase-precommit-lightcheck.sh
#
# PreToolUse hook (matcher Bash) — 3 verifications legeres pre-commit
# qui completent (sans remplacer) le gate phase-auditor-gate.sh sur les
# phases SKIP-LIGHTWEIGHT (criteres C1-C8 amendement 2026-04-20).
#
# Verifications :
#   1. Coherence staging (STRICT, BLOCK) — pour chaque `+pub mod X;`
#      Rust ajoute, verifier que `<dir>/X.rs` ou `<dir>/X/mod.rs` existe
#      sur disque ET est staged ou tracked dans HEAD.
#   2. Refs fichiers body (WARN, non-bloquant) — pour chaque
#      `<path>.{md,rs,py,ts,tsx,toml}` cite dans le commit body, verifier
#      que le file existe dans le repo.
#   3. LOC deviation (WARN, non-bloquant) — si body cite `~XXX LOC` et
#      diff stat reel >2.5x, demander mention "deviation LOC" explicite.
#
# Bypass d'urgence : NEXUS_SKIP_PHASE_AUDITOR=1 git commit ...
#
# Exit 0 : autorise (avec warnings stderr eventuels)
# Exit 2 : bloque (erreur stricte coherence staging seulement)

set -eo pipefail
INPUT=$(cat)

if command -v jq >/dev/null 2>&1; then
  CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null || true)
elif command -v python3 >/dev/null 2>&1; then
  CMD=$(echo "$INPUT" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("tool_input",{}).get("command","") or "")' 2>/dev/null || true)
else
  exit 0
fi

[ -z "$CMD" ] && exit 0
echo "$CMD" | grep -qE 'git[[:space:]]+commit' || exit 0
[ "${NEXUS_SKIP_PHASE_AUDITOR:-0}" = "1" ] && exit 0

REPO_ROOT=$(pwd)
[ ! -f "$REPO_ROOT/Cargo.toml" ] && exit 0
[ ! -d "$REPO_ROOT/crates/nexus-core-rs" ] && exit 0

ERRORS=0
WARNINGS=0

# === Check 1 : coherence staging pub mod (STRICT) ===
CURRENT_FILE=""
while IFS= read -r line; do
  if echo "$line" | grep -qE '^\+\+\+ b/.*\.rs$'; then
    CURRENT_FILE=$(echo "$line" | sed 's|^+++ b/||')
    continue
  fi
  if echo "$line" | grep -qE '^\+pub[[:space:]]+mod[[:space:]]+[a-z_][a-z0-9_]*[[:space:]]*;'; then
    [ -z "$CURRENT_FILE" ] && continue
    MOD_NAME=$(echo "$line" | grep -oE 'pub[[:space:]]+mod[[:space:]]+[a-z_][a-z0-9_]*' | awk '{print $NF}')
    CURRENT_DIR=$(dirname "$CURRENT_FILE")
    EXPECTED_FILE="$CURRENT_DIR/$MOD_NAME.rs"
    EXPECTED_MOD_DIR="$CURRENT_DIR/$MOD_NAME/mod.rs"
    if [ ! -f "$EXPECTED_FILE" ] && [ ! -f "$EXPECTED_MOD_DIR" ]; then
      echo "[lightcheck] ERROR: pub mod ${MOD_NAME} ajoute dans ${CURRENT_FILE} mais ni ${EXPECTED_FILE} ni ${EXPECTED_MOD_DIR} n'existe sur disque (commit incompilable)" >&2
      ERRORS=$((ERRORS + 1))
      continue
    fi
    STAGED_LIST=$(git diff --cached --name-only 2>/dev/null || true)
    HEAD_HAS_FILE=0
    HEAD_HAS_MOD=0
    git ls-files --error-unmatch "$EXPECTED_FILE" >/dev/null 2>&1 && HEAD_HAS_FILE=1
    git ls-files --error-unmatch "$EXPECTED_MOD_DIR" >/dev/null 2>&1 && HEAD_HAS_MOD=1
    STAGED_HAS_FILE=0
    STAGED_HAS_MOD=0
    echo "$STAGED_LIST" | grep -qFx "$EXPECTED_FILE" && STAGED_HAS_FILE=1
    echo "$STAGED_LIST" | grep -qFx "$EXPECTED_MOD_DIR" && STAGED_HAS_MOD=1
    if [ "$HEAD_HAS_FILE" -eq 0 ] && [ "$HEAD_HAS_MOD" -eq 0 ] \
       && [ "$STAGED_HAS_FILE" -eq 0 ] && [ "$STAGED_HAS_MOD" -eq 0 ]; then
      echo "[lightcheck] ERROR: pub mod ${MOD_NAME} ajoute mais le file ${EXPECTED_FILE} (ou ${EXPECTED_MOD_DIR}) est untracked / unstaged" >&2
      ERRORS=$((ERRORS + 1))
    fi
  fi
done < <(git diff --cached -U0 2>/dev/null || true)

# === Recuperer le body commit (-m "..." OU -F file) ===
BODY=""
COMMIT_MSG_FILE=$(echo "$CMD" | grep -oE '\-F[[:space:]]+[^[:space:]]+' | awk '{print $2}' | head -1 || true)
if [ -n "$COMMIT_MSG_FILE" ] && [ -f "$COMMIT_MSG_FILE" ]; then
  BODY=$(cat "$COMMIT_MSG_FILE")
else
  # -m "string" — capture entre quotes (single or double, tolerant)
  BODY=$(echo "$CMD" | sed -n 's/.*-m[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
  [ -z "$BODY" ] && BODY=$(echo "$CMD" | sed -n "s/.*-m[[:space:]]*'\([^']*\)'.*/\1/p" | head -1)
fi

# === Check 2 : refs fichiers body (WARN) ===
if [ -n "$BODY" ]; then
  while IFS= read -r ref_path; do
    [ -z "$ref_path" ] && continue
    case "$ref_path" in
      http*|*://*|*.example.*|*example.com*) continue;;
    esac
    case "$ref_path" in
      */*) ;;
      *) continue;;
    esac
    if [ ! -e "$ref_path" ] && [ ! -e "${REPO_ROOT}/${ref_path}" ]; then
      echo "[lightcheck] WARN: body cite \`${ref_path}\` mais le fichier n'existe pas dans le repo" >&2
      WARNINGS=$((WARNINGS + 1))
    fi
  done < <(echo "$BODY" | grep -oE '[a-zA-Z0-9_./~-]+\.(md|rs|py|ts|tsx|toml|sh|json|yml|yaml)' | sort -u)
fi

# === Check 3 : LOC deviation (WARN) ===
if [ -n "$BODY" ]; then
  CITED_LOC=$(echo "$BODY" | grep -oE '~[[:space:]]*[0-9]+[[:space:]]*LOC' | head -1 | grep -oE '[0-9]+' || true)
  if [ -n "$CITED_LOC" ] && [ "$CITED_LOC" -gt 0 ]; then
    ACTUAL_LOC=0
    while IFS=$'\t' read -r added removed file; do
      [ -z "$file" ] && continue
      [ "$added" = "-" ] && continue
      case "$file" in
        *test*|*tests/*|*/test_*|*spec.ts*|*.test.ts*|*.test.tsx*|*.md|docs/*|*.lock|Cargo.lock)
          continue ;;
      esac
      ACTUAL_LOC=$((ACTUAL_LOC + added))
    done < <(git diff --cached --numstat 2>/dev/null || true)
    THRESHOLD=$((CITED_LOC * 5 / 2))
    if [ "$ACTUAL_LOC" -gt "$THRESHOLD" ]; then
      if ! echo "$BODY" | grep -qiE 'deviation[[:space:]]+LOC|LOC[[:space:]]+deviation|ecart[[:space:]]+LOC'; then
        echo "[lightcheck] WARN: body cite ~${CITED_LOC} LOC, diff stat reel=${ACTUAL_LOC} (>${THRESHOLD} = 2.5x), aucune mention 'deviation LOC' / 'ecart LOC' dans body" >&2
        WARNINGS=$((WARNINGS + 1))
      fi
    fi
  fi
fi

if [ "$ERRORS" -gt 0 ]; then
  echo "" >&2
  echo "[lightcheck] BLOCK: ${ERRORS} erreur(s) coherence staging" >&2
  echo "  Fix les fichiers untracked (git add) OU drop les pub mod ajoutes" >&2
  echo "  Bypass d'urgence : NEXUS_SKIP_PHASE_AUDITOR=1 git commit ..." >&2
  echo "" >&2
  exit 2
fi

if [ "$WARNINGS" -gt 0 ]; then
  echo "[lightcheck] ${WARNINGS} warning(s) (non-bloquants)" >&2
fi

exit 0
