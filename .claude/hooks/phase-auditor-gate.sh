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

REVIEW_ACTIVE=".planning/active/sprint${SPRINT}_phase_${PHASE}_review.md"
# Si le review a deja ete archive (commit fix post-audit, Phase F wrap-up,
# ou chore intermediaire), on accepte aussi l'archive. Evite de forcer
# une regeneration du fichier qui cree des duplicates factuellement
# divergents (observe session 2026-04-18).
REVIEW_ARCHIVE=$(ls .planning/archive/v*/sprint${SPRINT}_phase_${PHASE}_review.md 2>/dev/null | head -1)

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
