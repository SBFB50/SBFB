#!/usr/bin/env bash
#
# .claude/hooks/phase-auditor-gate.sh
#
# PreToolUse hook (matcher Bash) qui bloque un `git commit` sur une
# phase SBFB si le rapport `.planning/active/sprint{N}_phase_{X}_review.md`
# n'existe pas ou n'a pas le verdict PASS.
#
# Design : fail-closed seulement pour les commits feat|fix|docs|chore|test
# qui matchent un scope sprint + un titre "Phase X". Tout autre commit
# passe sans check (chore(claude), hotfixes, Merge, etc.).
#
# Bypass d'urgence : NEXUS_SKIP_PHASE_AUDITOR=1 git commit ...
#
# Exit 0 : autorise le commit
# Exit 2 : bloque avec message d'erreur visible par Claude

set -eo pipefail

INPUT=$(cat)

# Extract tool_input.command. Try jq, fall back to python3. Fail-open.
if command -v jq >/dev/null 2>&1; then
  CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null || true)
elif command -v python3 >/dev/null 2>&1; then
  CMD=$(echo "$INPUT" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("tool_input",{}).get("command","") or "")' 2>/dev/null || true)
else
  exit 0
fi

[ -z "$CMD" ] && exit 0

# Pas un git commit ? no-op
echo "$CMD" | grep -qE 'git[[:space:]]+commit' || exit 0

# Bypass d'urgence
[ "${NEXUS_SKIP_PHASE_AUDITOR:-0}" = "1" ] && exit 0

# Scope nexus only (cwd check)
REPO_ROOT=$(pwd)
if [ ! -f "$REPO_ROOT/Cargo.toml" ] || [ ! -d "$REPO_ROOT/crates/nexus-core-rs" ]; then
  exit 0
fi

# Extraire sprint N et Phase X du commit message (anywhere in cmd)
# Le titre est suppose etre sur une ligne (convention nexus §4.1 README).
SPRINT=$(echo "$CMD" | grep -oE '(feat|fix|docs|chore|test)\(sprint[0-9]+\)' | head -1 | grep -oE '[0-9]+' || true)
PHASE=$(echo "$CMD" | grep -oE 'Phase[[:space:]]+[A-Z][0-9]?' | head -1 | awk '{print $2}' || true)

# Pas de sprint+phase ? no-op (commit lambda)
[ -z "$SPRINT" ] && exit 0
[ -z "$PHASE" ] && exit 0

# === Amendement criteres conditional run 2026-04-20 ===
# (cf. .planning/archive/v1.2/agent_auditor_gate_amendment_proposal_ACCEPTED.md
#      + docs/claude/TOOLING.md §5.2)
# Auditor obligatoire seulement si AU MOINS UN critere C1-C8 est vrai.
# Sinon SKIP-LIGHTWEIGHT auto-stub review.md PASS + git add automatique.

STAGED_FILES=$(git diff --cached --name-only 2>/dev/null || true)

# C1 wire format / canonical
TOUCHES_WIRE=0
echo "$STAGED_FILES" | grep -qE '^crates/nexus-core-rs/src/(canonical|schemas/)' && TOUCHES_WIRE=1

# C2 *_VERSION bump (lignes ajoutees seulement)
TOUCHES_VERSION=0
git diff --cached -U0 -- 'crates/**/*.rs' 'packages/**/*.py' 2>/dev/null \
  | grep -qE '^\+[^+].*_VERSION[[:space:]]*[:=][[:space:]]*[0-9]+' && TOUCHES_VERSION=1

# C3 crypto / signature primitives (Rust + Python — meme esprit, peu
# importe la langue : un fichier qui matche un de ces tokens est crypto-
# touched par construction de la convention nommage SBFB)
TOUCHES_CRYPTO=0
echo "$STAGED_FILES" \
  | grep -qE '(canary|provenance|curator|invite|gossip|pow|tls_pinning|encryption|duress|frost|signing|signature|keypair)\.(rs|py)$' \
  && TOUCHES_CRYPTO=1

# C4 multi-langue >= 2 categories (crates / packages / web)
CATEGORY_COUNT=$(echo "$STAGED_FILES" \
  | awk -F/ '{print $1}' | sort -u \
  | grep -cE '^(crates|packages|web)$' || echo 0)

# C5 LOC effectif (>500, hors tests + docs + .md + lockfiles)
EFFECTIVE_LOC=0
while IFS=$'\t' read -r added removed file; do
  [ -z "$file" ] && continue
  [ "$added" = "-" ] && continue
  case "$file" in
    *test*|*tests/*|*/test_*|*spec.ts*|*.test.ts*|*.test.tsx*|*.md|docs/*|*.lock|Cargo.lock)
      continue ;;
  esac
  EFFECTIVE_LOC=$((EFFECTIVE_LOC + added))
done < <(git diff --cached --numstat 2>/dev/null || true)

# C6 G8 DESIGN-CONFLICT (pivot proposal present active)
PIVOT_FILE=$(ls .planning/active/sprint${SPRINT}_phase_${PHASE}_pivot_proposal*.md 2>/dev/null | head -1 || true)

# C7 Phase F wrap-up
IS_PHASE_F=0
echo "$PHASE" | grep -qE '^F[0-9]?$' && IS_PHASE_F=1

# C8 sentinelle override explicite
FORCE_FILE=".planning/active/sprint${SPRINT}_phase_${PHASE}_force_audit.txt"
FORCE_OVERRIDE=0
[ -f "$FORCE_FILE" ] && FORCE_OVERRIDE=1

if [ "$TOUCHES_WIRE" -eq 0 ] && [ "$TOUCHES_VERSION" -eq 0 ] \
   && [ "$TOUCHES_CRYPTO" -eq 0 ] && [ "$CATEGORY_COUNT" -lt 2 ] \
   && [ "$EFFECTIVE_LOC" -lt 500 ] && [ -z "$PIVOT_FILE" ] \
   && [ "$IS_PHASE_F" -eq 0 ] && [ "$FORCE_OVERRIDE" -eq 0 ]; then
  REVIEW_STUB=".planning/active/sprint${SPRINT}_phase_${PHASE}_review.md"
  if [ ! -f "$REVIEW_STUB" ]; then
    cat > "$REVIEW_STUB" <<EOSTUB
# Sprint ${SPRINT} Phase ${PHASE} — auditor skip (heuristique gate hook)

## Verdict : PASS

SKIP-LIGHTWEIGHT — phase ne remplit aucun critere C1-C8 du gate (cf.
docs/claude/TOOLING.md §5.2 amendement 2026-04-20) :

- C1 wire format / canonical : non
- C2 *_VERSION bump : non
- C3 crypto / sig primitives : non
- C4 multi-langue >= 2 categories : non (categories=${CATEGORY_COUNT})
- C5 >500 LOC effectif : non (LOC=${EFFECTIVE_LOC})
- C6 G8 DESIGN-CONFLICT : non (pas de pivot_proposal)
- C7 Phase F wrap-up : non
- C8 sentinelle force_audit : non

Hooks legers pre-commit appliques par
.claude/hooks/phase-precommit-lightcheck.sh
(staging coherence strict + refs lignes body warn + LOC deviation warn).

Si re-audit souhaite a posteriori (sweep audit gate fin sprint), creer
${FORCE_FILE} + re-commit triggerera l auditor sur le diff cumule.
EOSTUB
    git add "$REVIEW_STUB" 2>/dev/null || true
    echo "[phase-auditor-gate] SKIP-LIGHTWEIGHT (review stub auto-stage)" >&2
  fi
  exit 0
fi
# === fin amendement ===

REVIEW_ACTIVE=".planning/active/sprint${SPRINT}_phase_${PHASE}_review.md"
# Si le review a deja ete archive (commit fix post-audit, Phase F wrap-up,
# ou chore intermediaire), on accepte aussi l'archive. Evite de forcer
# une regeneration du fichier qui cree des duplicates factuellement
# divergents (observe session 2026-04-18).
#
# `|| true` : quand le glob `.planning/archive/v*/sprint..._review.md`
# ne matche aucun fichier, `ls` exits 2 sur Windows Git Bash. Avec
# `set -eo pipefail` en tete du script, cet exit non-zero se propage
# silencieusement et fait echouer le hook avant meme le check
# `-f $REVIEW_ACTIVE`. Le `|| true` en fin de pipeline protege le
# chemin "archive absent mais active present" contre ce faux-positif
# d'environnement (detecte session 2026-04-19).
REVIEW_ARCHIVE=$(ls .planning/archive/v*/sprint${SPRINT}_phase_${PHASE}_review.md 2>/dev/null | head -1 || true)

if [ -f "$REVIEW_ACTIVE" ]; then
  REVIEW="$REVIEW_ACTIVE"
elif [ -n "$REVIEW_ARCHIVE" ]; then
  REVIEW="$REVIEW_ARCHIVE"
else
  echo "" >&2
  echo "[phase-auditor-gate] BLOCK: missing review file" >&2
  echo "" >&2
  echo "  Expected (active):  $REVIEW_ACTIVE" >&2
  echo "  Expected (archive): .planning/archive/v{X}/sprint${SPRINT}_phase_${PHASE}_review.md" >&2
  echo "" >&2
  echo "  Avant de committer Sprint ${SPRINT} Phase ${PHASE}, lance l'agent :" >&2
  echo "    Task(subagent_type=\"nexus-phase-auditor\"," >&2
  echo "         prompt=\"Audit Sprint ${SPRINT} Phase ${PHASE}." >&2
  echo "                 ECRIRE OBLIGATOIREMENT via Write tool dans" >&2
  echo "                 ${REVIEW_ACTIVE} AVANT de retourner." >&2
  echo "                 Stdout ne suffit pas — le hook bloque sans le" >&2
  echo "                 fichier sur disque." >&2
  echo "                 Draft commit body: <coller ici>\")" >&2
  echo "" >&2
  echo "  Si l'agent ne Write PAS le fichier, ne PAS le transcrire toi-meme" >&2
  echo "  (defait l'independance G4) — relance l'agent avec le rappel ci-dessus." >&2
  echo "" >&2
  echo "  Bypass d'urgence : NEXUS_SKIP_PHASE_AUDITOR=1 git commit ..." >&2
  echo "" >&2
  exit 2
fi

# Review existe (active ou archive) — verdict PASS ?
if ! grep -qE '^## Verdict[[:space:]]*:[[:space:]]*PASS' "$REVIEW"; then
  VERDICT_LINE=$(grep -E '^## Verdict' "$REVIEW" | head -1 || echo "(unknown)")
  echo "" >&2
  echo "[phase-auditor-gate] BLOCK: review not PASS" >&2
  echo "" >&2
  echo "  File: $REVIEW" >&2
  echo "  Verdict: $VERDICT_LINE" >&2
  echo "" >&2
  echo "  Fix les findings P0/P1 listees dans le rapport, puis re-lance" >&2
  echo "  nexus-phase-auditor pour mettre a jour le verdict a PASS." >&2
  echo "" >&2
  echo "  Bypass d'urgence : NEXUS_SKIP_PHASE_AUDITOR=1 git commit ..." >&2
  echo "" >&2
  exit 2
fi

# Verdict PASS, autorise
exit 0
