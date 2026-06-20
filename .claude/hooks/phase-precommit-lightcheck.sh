#!/usr/bin/env bash
#
# .claude/hooks/phase-precommit-lightcheck.sh
#
# PreToolUse hook (matcher Bash) — verifications legeres pre-commit
# qui completent (sans remplacer) le gate phase-auditor-gate.sh sur
# tous les commits sprint (inconditionnel depuis S24 process review).
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
#   4. Wire-format staging alert (WARN) — fichiers canonical.rs/schemas/
#      _VERSION modifies → verifier preflight S4 full scan.
#   5. G1 Design Review Board gate (STRICT, BLOCK Phase A) — verifier
#      sprint{N}_design_review.md existe avant Phase A.
#   6. LOC guard plans (STRICT, BLOCK) — bloquer plans avec estimations LOC.
#   7. Codex verification artifact (STRICT, BLOCK Phase) — verifier
#      sprint{N}_phase_{X}_codex_review.md existe et ne ressemble pas
#      a un resume Claude reecrit. Zero exemption LOC.
#   8. Preflight G8 presence (STRICT, BLOCK feat Phase) — verifier
#      sprint{N}_phase_{X}_preflight.md existe.
#   9. Commit body sections (STRICT, BLOCK feat/docs Phase) — verifier
#      les 9 headers ## obligatoires du body (§4.1 README).
#
# Exit 0 : autorise (avec warnings stderr eventuels)
# Exit 2 : bloque (erreur stricte)

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

REPO_ROOT=$(pwd)
[ ! -f "$REPO_ROOT/Cargo.toml" ] && exit 0
[ ! -d "$REPO_ROOT/crates/nexus-core-rs" ] && exit 0

ERRORS=0
WARNINGS=0

# === Check 0 : Whitespace / conflict-marker sanity (STRICT, BLOCK) ===
DIFF_CHECK=$(git diff --cached --check 2>&1 || true)
if [ -n "$DIFF_CHECK" ]; then
  echo "" >&2
  echo "[lightcheck] BLOCK: git diff --cached --check failed" >&2
  echo "$DIFF_CHECK" >&2
  echo "" >&2
  ERRORS=$((ERRORS + 1))
fi

# === Extract SPRINT + PHASE from commit title only (shared by checks 4, 5) ===
# Extract the commit title (first line of the message, before any newline).
# Supports both `-m "message"` and `-F filename` syntax.
COMMIT_TITLE=$(echo "$CMD" | sed -n "s/.*-m[[:space:]]*[\"']\?\([^\n\"]*\).*/\1/p" | head -1 || true)
if [ -z "$COMMIT_TITLE" ]; then
  # Fallback: extract title from -F filename (git commit -F path)
  COMMIT_FILE=$(echo "$CMD" | grep -oE -- '-F[[:space:]]+[^ ]+' | sed 's/-F[[:space:]]*//' | head -1 || true)
  if [ -n "$COMMIT_FILE" ] && [ -f "$COMMIT_FILE" ]; then
    COMMIT_TITLE=$(head -1 "$COMMIT_FILE" 2>/dev/null || true)
  fi
fi
[ -z "$COMMIT_TITLE" ] && COMMIT_TITLE="$CMD"
# Primary: scope-based detection (feat(sprint64): Sprint 64 Phase A)
SPRINT=$(echo "$COMMIT_TITLE" | grep -oE '(feat|fix|docs|chore|test|refactor)\(sprint[0-9]+\)' | head -1 | grep -oE '[0-9]+' || true)
TITLE_SPRINT=$(echo "$COMMIT_TITLE" | grep -oE 'Sprint[[:space:]]+[0-9]+[[:space:]]+Phase[[:space:]]+[A-Z]+' | head -1 | grep -oE '[0-9]+' || true)
if [ -n "$SPRINT" ] && [ -n "$TITLE_SPRINT" ] && [ "$SPRINT" != "$TITLE_SPRINT" ]; then
  echo "[lightcheck] BLOCK: commit scope sprint${SPRINT} conflicts with title Sprint ${TITLE_SPRINT}" >&2
  ERRORS=$((ERRORS + 1))
fi
# Fallback: title-based detection (feat(feed): Sprint 64 Phase A)
if [ -z "$SPRINT" ]; then
  SPRINT="$TITLE_SPRINT"
fi
PHASE_RAW=$(echo "$COMMIT_TITLE" | grep -oE 'Sprint[[:space:]]+[0-9]+[[:space:]]+Phase[[:space:]]+[A-Z]+[0-9]?' | head -1 | awk '{print $NF}' || true)
PHASE=$(echo "$PHASE_RAW" | tr '[:upper:]' '[:lower:]' || true)
PHASE_UPPER=$(echo "$PHASE" | tr '[:lower:]' '[:upper:]' || true)

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

# === Check 6 : LOC guard — block plans with LOC estimations (STRICT) ===
# P2-REVIEW-A-1 MANDATORY 3/3. Enforcement mecanique de §6.7.
# Grep staged sprint*_plan.md for amont LOC estimation patterns.
# Exception: HARDENING_ROADMAP.md (bornes indicatives admises §6.7).
STAGED_PLANS=$(git diff --cached --name-only 2>/dev/null | grep -E 'sprint[0-9]+_plan\.md$' | grep -v '/archive/' || true)
if [ -n "$STAGED_PLANS" ]; then
  while IFS= read -r plan_file; do
    [ -z "$plan_file" ] && continue
    LOC_HITS=$(git diff --cached -- "$plan_file" 2>/dev/null | grep -E '^\+' | grep -iE '~[[:space:]]*[0-9]+[[:space:]]*(LOC|lignes)|environ[[:space:]]+[0-9]+[[:space:]]+LOC|budget[[:space:]]+LOC|LOC[[:space:]]+total' || true)
    if [ -n "$LOC_HITS" ]; then
      echo "" >&2
      echo "[lightcheck] BLOCK: LOC estimation detected in staged plan (§6.7)" >&2
      echo "  File: $plan_file" >&2
      echo "$LOC_HITS" | head -3 | while IFS= read -r hit; do
        echo "  $hit" >&2
      done
      echo "" >&2
      echo "  Sprint scope is dimensioned by functional goal, not LOC budget." >&2
      echo "  Remove the LOC estimation from the plan. Cf. docs/claude/README.md §6.7." >&2
      echo "" >&2
      ERRORS=$((ERRORS + 1))
    fi
  done <<< "$STAGED_PLANS"
fi

# === Check 4 : wire-format staging alert (WARN) ===
# Filter: skip files with whitespace-only changes (edition 2024
# import reorder, rustfmt drift) to avoid false positives (§P45).
WIRE_FILES=$(git diff --cached --name-only 2>/dev/null | grep -E 'canonical\.rs|schemas/|_VERSION' || true)
SUBSTANTIVE_WIRE=""
if [ -n "$WIRE_FILES" ]; then
  while IFS= read -r wf; do
    [ -z "$wf" ] && continue
    NON_WS_DIFF=$(git diff --cached --ignore-all-space -- "$wf" 2>/dev/null)
    if [ -n "$NON_WS_DIFF" ]; then
      SUBSTANTIVE_WIRE="$SUBSTANTIVE_WIRE
$wf"
    fi
  done <<< "$WIRE_FILES"
  SUBSTANTIVE_WIRE=$(echo "$SUBSTANTIVE_WIRE" | sed '/^$/d')
fi
if [ -n "$SUBSTANTIVE_WIRE" ]; then
  echo "[lightcheck] WARN: wire-format files staged:" >&2
  echo "$SUBSTANTIVE_WIRE" | while IFS= read -r wf; do
    echo "  $wf" >&2
  done
  PREFLIGHT_PHASE=""
  if [ -n "$SPRINT" ] && [ -n "$PHASE" ]; then
    PREFLIGHT_PHASE=".planning/active/sprint${SPRINT}_phase_${PHASE}_preflight.md"
  fi
  if [ -n "$PREFLIGHT_PHASE" ] && [ -f "$PREFLIGHT_PHASE" ]; then
    if ! grep -qE 'S4.*full|full.*S4|FULL SCAN' "$PREFLIGHT_PHASE" 2>/dev/null; then
      echo "[lightcheck] WARN: preflight S4 may be fast-path — wire-format requires full S4 scan" >&2
      echo "  Pre-launch protocol applies. Verify VERSION=1 + Day 0 preserved." >&2
      WARNINGS=$((WARNINGS + 1))
    fi
  else
    echo "[lightcheck] WARN: no preflight found — verify S4 wire-format invariants manually" >&2
    WARNINGS=$((WARNINGS + 1))
  fi
fi

# === Check 5 : G1 Design Review Board gate (STRICT for Phase A) ===
# §6.1.1 : G1 obligatoire sauf sprint pure-docs ou trivial.
# Le hook bloque Phase A si sprint{N}_design_review.md n'existe nulle part.
# Exemption : kickoff contient "G1 skipped" (decision documentee §3.5).
if [ "$PHASE" = "A" ] && [ -n "$SPRINT" ]; then
  DR_ACTIVE=".planning/active/sprint${SPRINT}_design_review.md"
  DR_ARCHIVE=$(ls .planning/archive/v*/sprint${SPRINT}_design_review.md 2>/dev/null | head -1 || true)

  if [ ! -f "$DR_ACTIVE" ] && [ -z "$DR_ARCHIVE" ]; then
    # Check exemption in kickoff
    KICKOFF_FILE=""
    [ -f ".planning/active/sprint${SPRINT}_kickoff.md" ] && KICKOFF_FILE=".planning/active/sprint${SPRINT}_kickoff.md"
    [ -z "$KICKOFF_FILE" ] && KICKOFF_FILE=$(ls .planning/archive/v*/sprint${SPRINT}_kickoff.md 2>/dev/null | head -1 || true)

    G1_EXEMPT=0
    if [ -n "$KICKOFF_FILE" ]; then
      grep -qiE 'G1[[:space:]]+(skip|exempt)|design[[:space:]]+review[[:space:]]+(skip|exempt)|Phase 0 audit skipped' "$KICKOFF_FILE" 2>/dev/null && G1_EXEMPT=1
    fi

    if [ "$G1_EXEMPT" -eq 0 ]; then
      echo "" >&2
      echo "[lightcheck] BLOCK: G1 Design Review Board manquant (§6.1.1)" >&2
      echo "" >&2
      echo "  Sprint ${SPRINT} Phase A requiert sprint${SPRINT}_design_review.md" >&2
      echo "  Attendu: ${DR_ACTIVE}" >&2
      echo "" >&2
      echo "  Action: lancer un agent Explore independant pour scorer D1..D5" >&2
      echo "    du kickoff avant de coder Phase A (cf. README §6.1.1)." >&2
      echo "" >&2
      echo "  Exemption: ajouter 'G1 skipped per user decision YYYY-MM-DD'" >&2
      echo "    dans le kickoff si sprint pure-docs ou trivial (§3.5)." >&2
      echo "" >&2
      ERRORS=$((ERRORS + 1))
    fi
  fi
fi

# === Check 7 : Codex verification artifact (STRICT for Phase) ===
# §4.5 : Codex verification croisee obligatoire. Zero exemption.
# Seul bypass : body contient "PO skip codex" explicite.
if [ -n "$SPRINT" ] && [ -n "$PHASE" ]; then
  # Enforce on all phase commits including chore(sprintN) Phase.
  IS_PHASE_IMPL=$(echo "$COMMIT_TITLE" | grep -cE '^(feat|fix|docs|chore|test|refactor)\(' || true)
  HAS_SPRINT_PHASE_7=$(echo "$COMMIT_TITLE" | grep -cE 'Sprint[[:space:]]+[0-9]+[[:space:]]+Phase[[:space:]]+[A-Z]+' || true)
  if [ "$IS_PHASE_IMPL" -gt 0 ] && [ "$HAS_SPRINT_PHASE_7" -gt 0 ]; then
    CODEX_REVIEW=".planning/active/sprint${SPRINT}_phase_${PHASE}_codex_review.md"
    if [ ! -f "$CODEX_REVIEW" ]; then
      echo "" >&2
      echo "[lightcheck] BLOCK: Codex review manquant (§4.5)" >&2
      echo "  Attendu: ${CODEX_REVIEW}" >&2
      echo "  Procedure: ecrire prompt .git/CODEX_SPRINT${SPRINT}_PHASE_${PHASE_UPPER}.txt," >&2
      echo "    lancer codex exec, corriger GAPs, re-stage." >&2
      echo "  Zero exemption: une phase commit exige un artefact Codex." >&2
      echo "" >&2
      ERRORS=$((ERRORS + 1))
    else
      CODEX_TRACKED=0
      git ls-files --error-unmatch "$CODEX_REVIEW" >/dev/null 2>&1 && CODEX_TRACKED=1
      CODEX_STAGED=0
      git diff --cached --name-only -- "$CODEX_REVIEW" | grep -qxF "$CODEX_REVIEW" && CODEX_STAGED=1
      CODEX_UNSTAGED=0
      git diff --name-only -- "$CODEX_REVIEW" | grep -qxF "$CODEX_REVIEW" && CODEX_UNSTAGED=1
      if [ "$CODEX_TRACKED" -eq 0 ] && [ "$CODEX_STAGED" -eq 0 ]; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review existe mais n'est pas stage (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      if [ "$CODEX_UNSTAGED" -eq 1 ]; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review a des changements non stages (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      if [ ! -s "$CODEX_REVIEW" ]; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review vide (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      if grep -qiE '^\s*#\s*Codex Review|Auditeur.*Claude|agent independant' "$CODEX_REVIEW"; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review semble reecrit par Claude (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "  Attendu: output brut de codex exec -o, sans en-tete Auditeur/Claude." >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      if ! grep -qiE 'CONFIRME|CONFIRM[EÉ]|GAP|PARTIEL|PARTIAL|CONFIRMED' "$CODEX_REVIEW"; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review sans verdict par livrable (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      if ! grep -qiE 'Evidence|Fichier|File|ligne|line|:[0-9]{1,5}' "$CODEX_REVIEW"; then
        echo "" >&2
        echo "[lightcheck] BLOCK: Codex review sans evidence fichier:ligne (§4.5)" >&2
        echo "  Fichier: ${CODEX_REVIEW}" >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
      CODEX_HAS_PARTIAL=0
      grep -qiE 'Statut[[:space:]]*:[[:space:]]*PARTIEL|Partiels?[[:space:]]*:[[:space:]]*[1-9]' "$CODEX_REVIEW" && CODEX_HAS_PARTIAL=1
      if [ "$CODEX_HAS_PARTIAL" -eq 1 ] && [ -n "$BODY" ]; then
        if echo "$BODY" | grep -qiE '0[[:space:]]+PARTIEL'; then
          echo "" >&2
          echo "[lightcheck] BLOCK: body Codex annonce 0 PARTIEL mais l'artefact en contient (§4.5)" >&2
          echo "  Fichier: ${CODEX_REVIEW}" >&2
          echo "" >&2
          ERRORS=$((ERRORS + 1))
        elif ! echo "$BODY" | grep -qiE '[1-9][0-9]*[[:space:]]+PARTIEL|PARTIELS?'; then
          echo "" >&2
          echo "[lightcheck] BLOCK: body Codex ne reporte pas les PARTIELS de l'artefact (§4.5)" >&2
          echo "  Fichier: ${CODEX_REVIEW}" >&2
          echo "" >&2
          ERRORS=$((ERRORS + 1))
        fi
      fi
      CODEX_HAS_GAP=0
      grep -qiE 'Statut[[:space:]]*:[[:space:]]*GAP|Gaps?[[:space:]]*:[[:space:]]*[1-9]' "$CODEX_REVIEW" && CODEX_HAS_GAP=1
      if [ "$CODEX_HAS_GAP" -eq 1 ] && [ -n "$BODY" ]; then
        if echo "$BODY" | grep -qiE '0[[:space:]]+GAP'; then
          echo "" >&2
          echo "[lightcheck] BLOCK: body Codex annonce 0 GAP mais l'artefact en contient (§4.5)" >&2
          echo "  Fichier: ${CODEX_REVIEW}" >&2
          echo "" >&2
          ERRORS=$((ERRORS + 1))
        elif ! echo "$BODY" | grep -qiE '[1-9][0-9]*[[:space:]]+GAP|GAPS?'; then
          echo "" >&2
          echo "[lightcheck] BLOCK: body Codex ne reporte pas les GAPs de l'artefact (§4.5)" >&2
          echo "  Fichier: ${CODEX_REVIEW}" >&2
          echo "" >&2
          ERRORS=$((ERRORS + 1))
        fi
      fi
    fi
  fi
fi

# === Check 8 : Preflight G8 presence (STRICT for all Phase commits) ===
# §6.9 : preflight obligatoire avant code pour chaque phase.
if [ -n "$SPRINT" ] && [ -n "$PHASE" ]; then
  IS_PHASE_IMPL_8=$(echo "$COMMIT_TITLE" | grep -cE '^(feat|fix|docs|chore|test|refactor)\(' || true)
  HAS_SPRINT_PHASE_8=$(echo "$COMMIT_TITLE" | grep -cE 'Sprint[[:space:]]+[0-9]+[[:space:]]+Phase[[:space:]]+[A-Z]+' || true)
  if [ "$IS_PHASE_IMPL_8" -gt 0 ] && [ "$HAS_SPRINT_PHASE_8" -gt 0 ]; then
    PREFLIGHT_FILE=".planning/active/sprint${SPRINT}_phase_${PHASE}_preflight.md"
    if [ ! -f "$PREFLIGHT_FILE" ]; then
      echo "" >&2
      echo "[lightcheck] BLOCK: Preflight G8 manquant (§6.9)" >&2
      echo "  Attendu: ${PREFLIGHT_FILE}" >&2
      echo "  Lancer skill nexus-phase-preflight avant commit." >&2
      echo "" >&2
      ERRORS=$((ERRORS + 1))
    fi
  fi
fi

# === Check 9 : Commit body sections (STRICT for phase impl commits) ===
# §4.1 : 9 sections ## obligatoires dans le body de chaque commit
# feat/fix/docs/test/refactor contenant "Sprint N Phase X" dans le titre.
# Zero exemption : Codex verification ne se skippe pas par body marker.
if [ -n "$SPRINT" ] && [ -n "$PHASE" ] && [ -n "$BODY" ]; then
  IS_PHASE_IMPL=$(echo "$COMMIT_TITLE" | grep -cE '^(feat|fix|docs|chore|test|refactor)\(' || true)
  HAS_SPRINT_PHASE=$(echo "$COMMIT_TITLE" | grep -cE 'Sprint[[:space:]]+[0-9]+[[:space:]]+Phase[[:space:]]+[A-Z]+' || true)
  if [ "$IS_PHASE_IMPL" -gt 0 ] && [ "$HAS_SPRINT_PHASE" -gt 0 ]; then
      MISSING_SECTIONS=""
      MISSING_COUNT=0

      # 1. ## Contexte
      if ! echo "$BODY" | grep -qE '^## Contexte'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Contexte\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 2. ## Fichiers
      if ! echo "$BODY" | grep -qE '^## Fichiers'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Fichiers\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 3. ## Delta tests
      if ! echo "$BODY" | grep -qE '^## Delta tests'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Delta tests\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 4. ## Verification (tolerant: Verification, Vérification, Verification §7.4)
      if ! echo "$BODY" | grep -qE '^## V[eé]rification'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Verification\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 5. ## Scope cuts (tolerant: respectés, honoured)
      if ! echo "$BODY" | grep -qE '^## Scope cuts'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Scope cuts\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 6. ## G8 traceability
      if ! echo "$BODY" | grep -qE '^## G8 traceability'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## G8 traceability\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 7. ## Pre-launch protocol
      if ! echo "$BODY" | grep -qE '^## Pre-launch protocol'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Pre-launch protocol\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 8. ## Codex verification
      if ! echo "$BODY" | grep -qE '^## Codex verification'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Codex verification\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi
      # 9. ## Carry closure (tolerant: / Unblock)
      if ! echo "$BODY" | grep -qE '^## Carry closure'; then
        MISSING_SECTIONS="${MISSING_SECTIONS}    - ## Carry closure\n"
        MISSING_COUNT=$((MISSING_COUNT + 1))
      fi

      if [ "$MISSING_COUNT" -gt 0 ]; then
        echo "" >&2
        echo "[lightcheck] BLOCK: ${MISSING_COUNT} section(s) ## obligatoire(s) manquante(s) dans le body (§4.1)" >&2
        echo "" >&2
        echo -e "  Sections manquantes :" >&2
        echo -e "${MISSING_SECTIONS}" >&2
        echo "  Le body d'un commit phase doit contenir les 9 headers ##" >&2
        echo "  prescrits par docs/claude/README.md §4.1." >&2
        echo "" >&2
        ERRORS=$((ERRORS + 1))
      fi
  fi
fi

if [ "$ERRORS" -gt 0 ]; then
  echo "" >&2
  echo "[lightcheck] BLOCK: ${ERRORS} erreur(s) pre-commit" >&2
  echo "  Corriger le process artifact/body avant commit." >&2
  echo "" >&2
  exit 2
fi

if [ "$WARNINGS" -gt 0 ]; then
  echo "[lightcheck] ${WARNINGS} warning(s) (non-bloquants)" >&2
fi

exit 0
