# Chore planning — amendement critères hook `phase-auditor-gate.sh`

**Date proposition** : 2026-04-20
**Date arbitrage** : 2026-04-20 (Accept full §6 option A — patch hook +
hooks légers + TOOLING.md update)
**Origine** : Sprint 22 Phase D `56211f2` — ROI auditor jugé faible
(~97k tokens / 6.5 min pour 1 finding utile sur 4 sur Rust-only 643
LOC bien préparée par préflight G8 SCOPE-CUT-CONSISTENT).
**Statut** : **ACCEPTED**, livré dans le commit qui archive ce doc :
patch `.claude/hooks/phase-auditor-gate.sh` §4 + nouveau
`.claude/hooks/phase-precommit-lightcheck.sh` §5 + update
`docs/claude/TOOLING.md §5.2` + ajout 2ème entrée PreToolUse Bash dans
`.claude/settings.json`.

## 1. Problème observé

Le hook `.claude/hooks/phase-auditor-gate.sh` actuel bloque
**systématiquement** tout commit dont le titre matche
`(feat|fix|docs|chore|test)(sprint{N}) — Phase {X}` jusqu'à la
présence d'un `sprint{N}_phase_{X}_review.md` avec `## Verdict : PASS`.

Cas Phase D Sprint 22 (`56211f2`) — coût/bénéfice mesuré :

| Métrique | Valeur |
|---|---|
| Tokens auditor 1 run | ~97 000 |
| Durée 1 run | ~6.5 min |
| Findings produits | 4 (1 P1 + 2 P2 + 1 P3) |
| Findings à valeur réelle | 1 (P2 ref ligne THREAT_MODEL/HARDENING_ROADMAP) |
| Findings triviaux attrapables par hook léger | 3 (P1 staging via `git status` ; P2 LOC via `wc -l` ; P3 doc commentaire fastidieux mais détectable par grep cohérence claim/code) |
| Re-run coût pour valider fixes | ~97 000 tokens supplémentaires |

Le pattern devient anti-économique sur **phases courtes mono-langue
sans wire format touché**. À l'inverse, l'auditor reste **vraie
valeur** sur phases multi-langue, avec wire format / canonical /
crypto, ou avec G8 verdict DESIGN-CONFLICT (pivot retrospectivement
audité).

## 2. Critères actuels (hook §1-3)

```
SI titre matche `(feat|fix|docs|chore|test)\(sprint[0-9]+\)` ET
SI titre matche `Phase [A-Z][0-9]?` ET
SI cwd = nexus repo (Cargo.toml + crates/nexus-core-rs/) ET
SI NEXUS_SKIP_PHASE_AUDITOR != 1
ALORS exiger sprint{N}_phase_{X}_review.md avec verdict PASS
```

Granularité : **toute phase**, sans tenir compte de la nature du diff.

## 3. Critères proposés

Auditor obligatoire si **au moins UN** des critères suivants
est vérifié :

| Critère | Détection bash |
|---|---|
| **C1** Wire format / canonical touché | `git diff --cached --name-only \| grep -E 'crates/nexus-core-rs/src/(canonical\|schemas/)'` |
| **C2** `*_VERSION` field touché | `git diff --cached -U0 \| grep -E '\+.*_VERSION\s*[:=]\s*[0-9]+'` |
| **C3** Crypto / signature primitive touché | `git diff --cached --name-only \| grep -E '(canary\|provenance\|curator\|invite\|gossip\|pow\|tls_pinning\|encryption\|duress\|frost)\.rs$'` |
| **C4** Multi-langue (≥2 catégories Rust/Python/TS) | `git diff --cached --name-only \| awk -F/ '{print $1}' \| sort -u \| grep -cE 'crates\|packages\|web' >= 2` |
| **C5** Phase >500 LOC effectif (hors tests/doc) | `git diff --cached --stat -- ':(exclude)*tests*' ':(exclude)*test*.py' \| tail -1 \| grep -E '[0-9]+ insertion' avec count >500` |
| **C6** G8 verdict DESIGN-CONFLICT (préflight pivot proposal présent) | `test -f .planning/active/sprint{N}_phase_{X}_pivot_proposal*.md` |
| **C7** Audit gate fin sprint (Phase F wrap-up + audit findings) | titre commit matche `Phase F` OU diff contient `sprint{N}_audit_findings.md` |
| **C8** Override explicite | `git diff --cached --name-only \| grep -q 'sprint{N}_phase_{X}_force_audit.txt'` (fichier sentinelle créé manuel) |

Sinon → **skip auditor** + tracer dans review.md auto-stub :

```markdown
# Sprint {N} Phase {X} — auditor skip (criteres C1-C7 non remplis)

Verdict : SKIP-LIGHTWEIGHT (heuristique gate hook §3.1)

Phase Rust-only / mono-langue / <500 LOC effectif / pas de wire format /
pas de crypto / pas de DESIGN-CONFLICT / pas Phase F wrap-up.

Hooks légers pre-commit appliqués :
- git status cohérence diff (catch P1 staging)
- grep ref ligne body claims vs fichiers (catch P2 ref ligne)
- LOC count diff stat vs plan §X estimé (catch P2 LOC déviation >2x)

Si re-audit souhaité a posteriori (sweep audit gate fin sprint),
toucher fichier sentinelle .planning/active/sprint{N}_phase_{X}_force_audit.txt
+ re-commit triggerera l'auditor.
```

## 4. Patch hook proposé (diff `.claude/hooks/phase-auditor-gate.sh`)

```diff
@@ Lignes 50-95 (decision branch) @@
 SPRINT=...
 PHASE=...

+# === Amendement critères 2026-04-20 ===
+# Skip auditor si phase ne touche aucun critère C1-C7 (cf.
+# .planning/agent_auditor_gate_amendment_proposal.md).
+
+STAGED_FILES=$(git diff --cached --name-only)
+
+# C1 : wire format / canonical
+TOUCHES_WIRE=$(echo "$STAGED_FILES" | grep -cE 'crates/nexus-core-rs/src/(canonical|schemas/)' || true)
+# C2 : *_VERSION bump
+TOUCHES_VERSION=$(git diff --cached -U0 -- 'crates/**/*.rs' 'packages/**/*.py' 2>/dev/null | grep -cE '^\+.*_VERSION\s*[:=]\s*[0-9]+' || true)
+# C3 : crypto / signature
+TOUCHES_CRYPTO=$(echo "$STAGED_FILES" | grep -cE '(canary|provenance|curator|invite|gossip|pow|tls_pinning|encryption|duress|frost)\.rs$' || true)
+# C4 : multi-langue (>=2 catégories)
+CATEGORY_COUNT=$(echo "$STAGED_FILES" | awk -F/ '{print $1}' | sort -u | grep -cE '^(crates|packages|web)$' || true)
+# C5 : LOC effectif (excluding tests + docs)
+EFFECTIVE_LOC=$(git diff --cached --stat -- ':(exclude)*test*' ':(exclude)*.md' ':(exclude)docs/**' 2>/dev/null | tail -1 | grep -oE '[0-9]+ insertion' | head -1 | grep -oE '[0-9]+' || echo 0)
+# C6 : G8 DESIGN-CONFLICT pivot
+PIVOT_FILE=$(ls .planning/active/sprint${SPRINT}_phase_${PHASE}_pivot_proposal*.md 2>/dev/null | head -1 || true)
+# C7 : Phase F wrap-up
+IS_PHASE_F=$(echo "$PHASE" | grep -cE '^F[0-9]?$' || true)
+# C8 : override sentinelle
+FORCE_FILE=".planning/active/sprint${SPRINT}_phase_${PHASE}_force_audit.txt"
+
+if [ "$TOUCHES_WIRE" -eq 0 ] && [ "$TOUCHES_VERSION" -eq 0 ] && \
+   [ "$TOUCHES_CRYPTO" -eq 0 ] && [ "$CATEGORY_COUNT" -lt 2 ] && \
+   [ "$EFFECTIVE_LOC" -lt 500 ] && [ -z "$PIVOT_FILE" ] && \
+   [ "$IS_PHASE_F" -eq 0 ] && [ ! -f "$FORCE_FILE" ]; then
+  # Auto-stub review.md SKIP-LIGHTWEIGHT
+  cat > "$REVIEW_ACTIVE" <<EOSTUB
+# Sprint ${SPRINT} Phase ${PHASE} — auditor skip (heuristique hook §3.1)
+
+## Verdict : PASS
+
+SKIP-LIGHTWEIGHT — phase ne remplit aucun critère C1-C7 (Rust-only /
+mono-langue / <500 LOC effectif / pas de wire format / pas de crypto /
+pas de DESIGN-CONFLICT / pas Phase F wrap-up).
+
+Hooks légers pre-commit appliqués (cf. amendement gate 2026-04-20).
+EOSTUB
+  exit 0
+fi
+# === fin amendement ===
+
 REVIEW_ACTIVE=".planning/active/sprint${SPRINT}_phase_${PHASE}_review.md"
 ...
```

## 5. Hooks légers complémentaires (file séparé)

Optionnel : ajouter un hook léger `phase-precommit-lightcheck.sh`
qui applique 3 vérifications systématiques (peu importe C1-C7) :

1. **Cohérence staging** : pas de fichier `??` dans `git status`
   matchant un `pub mod X;` ajouté au diff (catch P1 Phase D Sprint 22).
2. **Refs lignes body** : si commit body cite `<file>.md ligne {N}`,
   vérifier que la ligne {N} de ce fichier matche le claim (catch
   P2 ref Phase D Sprint 22).
3. **LOC déviation** : si commit body cite `~{X} LOC` et diff stat
   >2.5×{X}, demander mention déviation explicite dans le body
   (catch P2 LOC Phase D Sprint 22).

Ces 3 hooks peuvent run en <1 sec sans tokens — ils remplacent
pratiquement le besoin auditor sur phases simples.

## 6. Suite

Si user accepte amendement :

1. Patch `.claude/hooks/phase-auditor-gate.sh` selon §4
2. Optionnel : créer `.claude/hooks/phase-precommit-lightcheck.sh`
   selon §5
3. Update `docs/claude/TOOLING.md §5.2` pour refléter les nouveaux
   critères
4. Commit `chore(hook): phase-auditor-gate amendement criteres
   conditional run + lightweight pre-commit checks`

Si user refuse : archiver ce doc dans
`.planning/archive/v1.2/agent_auditor_gate_amendment_proposal_REJECTED.md`
avec rationale dans body commit.

## 7. Risques

- **Faux négatif** : phase légère mais avec subtle wire-impact non
  détecté par grep — mitigation = sentinelle C8 force_audit explicite.
- **Réduction discipline** : risque que des phases >500 LOC mono-langue
  ne passent jamais par auditor — mitigation = audit gate fin sprint
  (Phase F wrap-up = C7 obligatoire) qui re-balaye sur le diff cumulé
  du sprint.
- **Drift docs** : critères C1-C7 codés dans le hook bash + dupliqués
  TOOLING.md → désync future. Mitigation = ajouter assertion CI qui
  fail si TOOLING.md §5.2 ne mentionne pas la liste C1-C8 actuelle.
