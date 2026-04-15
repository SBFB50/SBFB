#!/usr/bin/env bash
#
# .claude/hooks/post-commit-memory.sh
#
# Git post-commit hook qui met a jour la memory externe quand un commit
# lie a un sprint vient d'atterrir sur master.
#
# Deploiement : installer comme .git/hooks/post-commit (cf. TOOLING.md §6.1)
#
# Declenche sur : feat|fix|docs|chore(sprint{N}) et fix(sprintN)
# Skip : chore(claude), chore(tooling), Merge, Revert, autres scopes
#
# Update cible : memory/nexus_grid_pivot.md frontmatter `Tip \`<sha>\``
#                (premiere occurrence uniquement — le primary tip)
#
# Idempotent : re-run sur meme commit = no-op (le SHA est deja a jour)
# Fail-safe : si memory absente ou sed fail, print warning, exit 0
#             (ne bloque jamais le commit, le hook est post-commit)

set -u

MEMORY_DIR="$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory"
PIVOT_FILE="$MEMORY_DIR/nexus_grid_pivot.md"
INDEX_FILE="$MEMORY_DIR/MEMORY.md"

# Early-exit si memory absente (nouveau clone, CI, etc.)
[ ! -f "$PIVOT_FILE" ] && exit 0
[ ! -f "$INDEX_FILE" ] && exit 0

# Infos du dernier commit
NEW_SHA=$(git rev-parse --short HEAD 2>/dev/null)
COMMIT_MSG=$(git log -1 --format=%s 2>/dev/null)

[ -z "$NEW_SHA" ] && exit 0
[ -z "$COMMIT_MSG" ] && exit 0

# Filtrer : seulement les commits lies a un sprint
# Match: feat(sprint18) | fix(sprint17) | docs(sprint16) | chore(sprint15) | ...
case "$COMMIT_MSG" in
  feat\(sprint*|fix\(sprint*|docs\(sprint*|chore\(sprint*|test\(sprint*)
    SCOPE="sprint-commit"
    ;;
  *)
    # chore(claude), chore(tooling), Merge, Revert, etc. -> skip
    exit 0
    ;;
esac

# Extraire l'ancien tip depuis le frontmatter (premiere occurrence)
OLD_SHA=$(grep -oE 'Tip `[a-f0-9]+`' "$PIVOT_FILE" | head -1 | grep -oE '[a-f0-9]+')

if [ -z "$OLD_SHA" ]; then
  echo "[post-commit-memory] WARN: no 'Tip \`<sha>\`' found in $PIVOT_FILE" >&2
  exit 0
fi

# Idempotent : meme tip, rien a faire
if [ "$OLD_SHA" = "$NEW_SHA" ]; then
  exit 0
fi

# Update nexus_grid_pivot.md : remplacer PREMIERE occurrence
# GNU sed + Git Bash sur Windows supportent le range "0,/pattern/"
if sed -i "0,/Tip \`$OLD_SHA\`/{s/Tip \`$OLD_SHA\`/Tip \`$NEW_SHA\`/}" \
      "$PIVOT_FILE" 2>/dev/null; then
  : # OK
else
  echo "[post-commit-memory] WARN: sed failed on $PIVOT_FILE" >&2
  exit 0
fi

# Update MEMORY.md : si la ligne index mentionne le old tip, bump aussi
if grep -q "Tip \`$OLD_SHA\`" "$INDEX_FILE"; then
  sed -i "s/Tip \`$OLD_SHA\`/Tip \`$NEW_SHA\`/g" "$INDEX_FILE" 2>/dev/null || true
fi

echo "[post-commit-memory] memory updated: Tip $OLD_SHA -> $NEW_SHA ($SCOPE)" >&2
exit 0
