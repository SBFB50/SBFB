# Sprint 51 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S51).
**Tip audite** : `0bf2c83` (S51 Phase C wrap-up, HEAD master).
**Tip d'entree audit plan** : `54e8af0` (S51 Phase B, dernier feat).
**Date** : 2026-05-02.
**Documents source** : sprint52_audit_plan.md, sprint51_plan.md,
sprint51_kickoff.md, sprint51_verification.md.

---

## Verdict : PASS

0 P0, 0 P1, 1 P2, 2 P3.
S52 Phase A peut demarrer directement.

---

## Track A — Legacy deletion completeness (Phase A)

| # | Check | Evidence | Status |
|---|---|---|---|
| A-1 | `git ls-files nexus/ tests/*.py worker/ pyproject.toml uv.lock` | 0 fichiers | PASS |
| A-2 | `git ls-files .github/workflows/build-wheels.yml` | 0 fichiers | PASS |
| A-3 | `ls scripts/ci-smoke/` = 4 scripts | attestation-schema.sh, pkarr-relay-healthcheck.sh, reproducible-build.sh, supply-chain-green.sh | PASS |
| A-4 | `grep pip.audit scripts/ci-smoke/supply-chain-green.sh` | 0 matches | PASS |

## Track B — CI workflows coherence (Phase A)

| # | Check | Evidence | Status |
|---|---|---|---|
| B-1 | `grep -inE "python\|pytest\|maturin\|ruff\|uv" ci.yml` | 0 matches | PASS |
| B-2 | `grep -inE "nexus-core-py\|maturin\|wheel\|pypi" release.yml` | 0 matches | PASS |
| B-3 | `grep ci-smoke build-pkarr-image.yml` | `scripts/ci-smoke/pkarr-relay-healthcheck.sh` (line 127) | PASS |
| B-4 | `grep nexus-core-py scripts/release-attest.sh` | 0 matches | PASS |

## Track C — Carries resolution (Phase B)

| # | Check | Evidence | Status |
|---|---|---|---|
| C-1 | Canary reload size cap | `MAX_DURESS_ACK_MESSAGE_LEN = 256` (duress_ack.rs:55) + test `duress_ack_rejects_oversize_message` (line 238). `MAX_HEADLINE_LEN = 512` (mod.rs:89) + test `build_canary_rejects_oversize_headline` (line 538) + test `at_cap` (line 547). | PASS |
| C-2 | auth.rs set_var all #[cfg(test)] | Lines 1073, 1077, 1086, 1096, 1114, 1118 — tous dans `#[test]` fn (sbfb_home_honours_override, run_dir_paths_resolve, windows_pipe_names). Pattern save/restore avec `prev`. 0 set_var en production dans auth.rs. | PASS |
| C-3 | `_reload_policy_locked` absent code | `grep -rn _reload_policy_locked crates/ web/ docs/` → 0 matches dans .rs/.ts. Occurrences uniquement dans .planning/ (historique). Issue Python supprimee avec le monolithe. | PASS |

## Track D — Documentation coherence (Phase C)

| # | Check | Evidence | Status |
|---|---|---|---|
| D-1 | CLAUDE.md carries S52 | **P2** — CLAUDE.md ligne 127 liste `P2-REVIEW-A-1-S51 release-attest.sh dead code 1/3` comme carry actif. Mais verification.md §3 montre CLOSE Phase C (nexus-core-py path supprime). verification.md §4 et audit_plan §Carries S52 listent correctement 5 items (sans cet item). CLAUDE.md a 6 items — entree stale. | P2 |
| D-2 | HARDENING_ROADMAP.md last_validated | `last_validated: 2026-05-01 # G2 — Sprint 51 CLOSED` (line 3) | PASS |
| D-3 | SPRINT_LOG.md row S51 | Row S51 presente avec theme + phases + compteurs | PASS |

## Track E — Process / meta

| # | Check | Evidence | Status |
|---|---|---|---|
| E-1 | G8 preflights | 3/3 presents (A, B, C) — tous verdict EXECUTE. Audit plan disait "2/2" (comptait probablement A+B feat sans Phase C chore). Overcomplete. | PASS |
| E-2 | Scope cuts 8/8 | `git diff --stat 610b521..54e8af0` : 262 files, +966/-72335. 0 fichiers web/ touches (pas de scope creep frontend). Diff entierement aligne avec D1 (legacy delete) + D3 (CI cleanup). | PASS |
| E-3 | Phase reviews | 3/3 presents (A, B, C). Overcomplete vs audit plan "2/2". | PASS |
| E-4 | Delta tests cumule | 1199 Rust / 250 Vitest / 42+2f PW / 6/6 size = ~1455. Delta 0 (sprint soustractif). Coherent avec verification.md §2. | PASS |
| E-5 | Sprint impair, pas de dette | S51 impair → 0 phase dette obligatoire (§6.2.1 Regle 1). Confirme : 3 phases A-C, aucune labellee "dette". 3 carries P2 resolus proactivement Phase B (pas obligation). | PASS |
| E-6 | clippy print_stub | `print_stub` fn presente dans nexus-worker/src/main.rs:464 et nexus-shell-daemon/src/main.rs:718 (avant #[cfg(test)] modules). Clippy 0 warnings (verification.md check #2). Fix items_after_test_module confirme. | PASS |

## Track F — .gitignore hygiene

| # | Check | Evidence | Status |
|---|---|---|---|
| F-1 | .gitignore contient packages/ + tests/ | `packages/` line 93, `tests/` line 154 | PASS |
| F-2 | .gitignore sans nexus-core-py ref | `grep nexus-core-py .gitignore` → 0 matches | PASS |
| F-3 | git status clean post-wrap-up | **P3** — `git status --short` montre `.gitignore` modifie (ajout `docker/affine/.env`) + 7 dossiers/fichiers untracked (docker/affine/, docs/affine-sbfb/, docs/architecture/, scripts/*.mjs, tools/project-viz/, utputFormat). Tous sont du travail utilisateur post-sprint (AFFiNE dashboard, architecture docs, project viz tooling). `utputFormat` est un fichier accidentel (dump HTTP 500). Non imputable a S51. | P3 |

---

## Findings

### P2

- **D-1** : CLAUDE.md ligne 127 — `P2-REVIEW-A-1-S51 release-attest.sh dead code 1/3` est liste comme carry actif S52 mais a ete CLOSE en Phase C (verification.md §3 : "nexus-core-py path supprime"). Entree stale : 6 carries dans CLAUDE.md vs 5 dans verification.md §4 et audit_plan §Carries S52. Fix : supprimer cette ligne de CLAUDE.md au prochain commit planning (S52 kickoff).

### P3

- **E-1/E-3** : audit plan comptait "2/2" preflights et reviews mais S51 avait 3 phases feat/chore (A, B, C) avec 3/3 preflights et 3/3 reviews. Audit plan sous-estimait le nombre de phases. Nit cosmétique sur le plan, pas sur le sprint.

- **F-3** : working tree non-pristine apres wrap-up. Cause : travail utilisateur post-sprint (AFFiNE, architecture whiteboard, project-viz, scripts). + 1 fichier accidentel `utputFormat` (dump HTTP 500 Express). Aucune implication S51.

---

## Carries confirmes S52

| Item | Compteur | Source | Note |
|---|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe | pas de release rand 0.9 |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 (Day 0 #3) | |
| P2-REVIEW-A-1-S50 dispatch join order | 2/3 | S50 review | **attention 3/3 MANDATORY S53** |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 1/3 | NEW S51 Phase B review | |
| P2-REVIEW-A-2-S51 docs legacy orphelines | 1/3 | NEW S51 Phase A review | 21 docs legacy |
| P2-D-1-AUDIT CLAUDE.md stale carry | 1/3 | NEW S51 audit | fix S52 kickoff |

S52 pair → phase dette obligatoire (§6.2.1 Regle 1).
P2-REVIEW-A-1-S50 dispatch join order a 2/3 — si non adresse S52, 3/3 MANDATORY S53.

---

## Dimensions G4 rigor check

| Dimension | Explored | Findings | Evidence |
|---|---|---|---|
| A Legacy deletion | 4 checks, git ls-files + ls + grep | 0 finding | 0 fichier Python tracke |
| B CI coherence | 4 checks, grep 4 workflows | 0 finding | 0 reference legacy |
| C Carries resolution | 3 checks, grep + Read code | 0 finding | caps + tests + test-only set_var |
| D Documentation | 3 checks, Read + grep CLAUDE.md + ROADMAP + LOG | 1 P2 | stale carry CLAUDE.md:127 |
| E Process | 6 checks, ls preflights/reviews + diff stat + verification | 0 finding + 1 P3 | overcomplete G8 |
| F .gitignore | 3 checks, grep patterns | 0 finding + 1 P3 | post-sprint user work |

1 P2 + 2 P3 documentes sur 6 dimensions → G4 signal satisfait (>=1 P2+).
