# Phase Review — Sprint 73 Phase A (guardrail AVANT persist + lot doc menace)

## Verdict: PASS

Review Claude OK + Codex §4.5 FAIT et reconcilié (7/7 CONFIRME). Committable
après self-check supervisor G-COMMIT.

(Rigor signal : **2 findings P2+ documentés** dont 1 fermé en-phase / ≥1 requis.)

## Staging check (Step 1bis)
- Phase fichiers (6) : `crates/nexus-coordinator-rs/src/validator.rs`,
  `crates/nexus-shell-daemon/src/http.rs`,
  `crates/nexus-shell-daemon/src/validator_loop.rs`,
  `docs/security/THREAT_MODEL.md`,
  `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`,
  `docs/security/HARDENING_ROADMAP.md`.
- Artefacts à stager dans le commit phase : `sprint73_phase_a_preflight.md`,
  `sprint73_phase_a_review.md`, `sprint73_phase_a_codex_review.md`.
- Planning/docs split : N/A (3 docs security dans le scope Phase A) → commit
  phase atomique unique, pas de chore(planning) intermédiaire.
- Untracked accidentels : 0.

## Suites (§7.4)
| Suite | Avant (S72) | Après | Delta | Plateforme |
|---|---|---|---|---|
| coordinator-rs nextest | 254 | **255** | +1 (`quorum_guardrail_runs_on_agreed_text`) | Windows + Docker Linux, vert |
| shell-daemon nextest (excl `dispatch_loop`) | 285 | **289** | +4 (2 HTTP + 2 validator_loop) | Windows natif, vert |
| Rust workspace (canonique CI Linux) | 1544 | **1549** | +5 | Docker Linux : fmt+clippy+test+doctest verts* |
| Vitest (`web/`) | 279 | 279 | +0 (web intouché) | Windows, vert (23 files) |
| size-limit | 6/6 | 6/6 | — | vert |
| clippy `--workspace --all-targets` | 0 warn | 0 warn | — | Windows (2 crates) + Docker Linux (workspace) |
| fmt | clean | clean | — | Windows + Docker, exit 0 |

\* Docker Linux : **tout vert SAUF** `crates/sbfb-factory/tests/operator_server.rs`
(17 tests) qui paniquent sur `.expect("request failed")` (le harness ne joint
pas le serveur Operator spawné). C'est **P2-OPERATOR-TIMEOUT — bind-mount
Docker-sur-Windows NON fidèle** (memory : « canonique = CI Linux natif »),
**hors scope Phase A** : 0 fichier `sbfb-factory` au diff, et c'est le SEUL
binaire en échec (toutes les autres suites = 0 failed, **dont les 2
`dispatch_loop` worker-pump qui passent sur Linux**). Sur CI Linux natif
(GHA, qui a GTK + FS natif) ces 17 passent → base 1544.

Note Windows natif : les 2 `dispatch_loop::tests::*` (worker-pump iroh-docs)
exclus du run Windows ciblé = carry **P2-A-1 3/3** (hang `current_thread`
Windows, fix = **Phase B** D6). Phase A ne touche pas `dispatch_loop.rs`.

## Commit body validation
- Format titre : `fix(sprint73): Sprint 73 Phase A — guardrail before result_text persist (2 paths) + Operator tier + hardening-roadmap recadre` ✅
- Delta tests cohérent (+5) ✅ · Scope cuts honoured ✅ · Co-Authored-By (Opus 4.8 1M) ✅

## Body format validation (Step 4bis, §4.1) — 9/9 headers
`## Contexte` · `## Fichiers` · `## Delta tests` · `## Verification §7.4` ·
`## Scope cuts` · `## G8 traceability` · `## Pre-launch protocol` ·
`## Codex verification` · `## Carry closure` — tous présents.

## Modified-file branch coverage (Step 2bis, G9)
- `validate_result_pre_guardrail` / `_post_guardrail` / `validate_quorum_pre_guardrail`
  / `PendingResultPersist` → exercés (11 tests unitaires + quorum_guardrail). ✅
- `http.rs` guardrail reject/accept → `submit_result_rejected_by_guardrail_persists_nothing`
  + `submit_result_accepted_persists_after_guardrail`. ✅
- `validator_loop.rs` guardrail reject/accept → `validator_loop_rejected_result_not_persisted`
  + `validator_loop_accepted_result_persisted`. ✅
- P3 : branche d'erreur `post_guardrail → Err` (défensive) non testée — acceptable.

## Scope cuts verification (kickoff §7)
SearchManifest (#1), search/open/fork (#2), triplet SearchResult (D2/Phase D),
barre shell (D4/Phase E), worker-pump (D6/Phase B), Tantivy/@dev (gelés) :
0 fichier diff chacun. ✅ Phase A strictement sécurité + doc.

## Horizon long-terme + documentation amont (Step 4quater)
Preflight présent (EXECUTE) ✅ · D5 alternatives rejetées + rationale ✅ ·
solution SOTA (pas band-aid) ✅ · 0 LOC estimée au plan ✅.

## Research grounding (Step 4ter)
- 4ter-A : preflight 5 scans (S1a-S4), S1a ≥1 OSS (NeMo/openai-agents/guardrails-ai/Datadog),
  S2 régression confirmée (S72 D `110c003`). PASS.
- 4ter-B : 0 dep ajoutée (S1b). PASS.

## Memory consultation (Step 1.5)
`feedback_approach` (pick deepest / no band-aid) **respecté** : guardrail-before-persist
SOTA, rollback rejeté, ET le résiduel P2-1 fermé en-phase plutôt que carry
(option la plus poussée). Zone sécurité — pas d'autre contrainte memory.

## Findings (rigor signal — 2 P2+)
- **P2-1 — CLOSED en-phase** : `ResultValidator::validate` + shim composaient
  un persist-sans-guardrail (pub API cross-crate) → vecteur latent de
  ré-introduction du bug D5. Flaggé par la review ET Codex run-1 (PARTIEL
  Livrable 1). **Corrigé** : `ResultValidator` gaté `#[cfg(test)]`
  (validator.rs) → invariant API fermé, aucun chemin prod guardrail-less.
  Codex run-2 : Livrable 1 CONFIRME. **Pas reporté en carry.**
- **P2-2 — documenté (comportement HTTP, non régressif)** : sur rejet
  guardrail, la ligne n'est plus `completed` (avant D5 elle l'était = bug).
  Risque R4 audité : 4 tests `result_submit_*` + `e2e_*` restent verts ;
  aucun test n'assumait `result_text` lisible après rejet.
- **P3** : branche `post_guardrail → Err` non testée (défensive).
- **P3** : doc-comment « Quorum path » précède `validate_quorum_pre_guardrail`
  (exacte, cosmétique).

## Codex gate (§4.5) — zero exemption
- Status : **FAIT**.
- Run 1 : 6 CONFIRME + 1 PARTIEL (Livrable 1, `ResultValidator` pub guardrail-less).
- Fix appliqué (gate `#[cfg(test)]`) → re-suites vertes (coordinator 255/255
  Windows + Docker Linux) → Codex run-2.
- Run 2 (final, artefact `sprint73_phase_a_codex_review.md`) : **7 CONFIRME,
  0 GAP, 0 PARTIEL**.

## Codex reconciliation
- Status : **FAIT**. Rapport Codex run-2 lu : 7/7 CONFIRME, 0 GAP, 0 PARTIEL.
  Le seul PARTIEL (run-1) a été corrigé par code (cfg(test)), pas documenté en
  band-aid. Suites relancées après correction (coordinator-rs 255/255 sur les 2
  plateformes). Artefact Codex brut (non réécrit). Review promu PASS.

## Recommendation
- Ready to commit : **OUI** (verdict PASS final).
- Carry-overs S74 : **aucun** (P2-1 fermé en-phase).

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + phase + compteurs 1549/279/6)
- [ ] Update `MEMORY.md` (ligne index)
- [ ] preflight.md + review.md + codex_review.md stagés dans le commit phase
