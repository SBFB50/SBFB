# Sprint 31 (reserved) — Contribution families Sybil matrix implementation

**STATUS** : **RESERVED STUB** — not yet opened. Sprint number `S31` is a **placeholder**. Actual sprint number depends on trigger firing date (could be S29, S32, S35, or never).

**Activable IF and ONLY IF** one of the LT-3 triggers fires (cf. `docs/release/ROADMAP_COMMITMENTS.md §LT-3`) :

- (a) tag `v1.0` posé + ≥3 contributeurs non-compute actifs sur ≥1 app Gate 2 (30j glissants)
- (b) `/diagnostic/fairness` endpoint (S23 Phase D) reporte Gini > 0.70 OR top-5% > 50%
- (c) Cure53/ToB audit S29 signale vulnérabilité contribution-family

Si aucun trigger ne fire : ce stub reste latent indéfiniment, **kudos reste compute-only**, familles non-compute reconnues socialement via `ContributorAttestation` S22 Couche 2 (binaire voice-per-project) sans score numérique.

---

## Activation procedure

1. Copy ce fichier vers `.planning/active/sprint{N}_kickoff.md` où `N` = sprint courant au moment du trigger.
2. Amender les sections §1 (But), §2 (État vérifié), §4 (D1..D5) sur la base de :
   - Data empirique collectée via `/diagnostic/fairness` depuis Gate 2 unlock (fin S22)
   - Findings Cure53/ToB si trigger (c)
   - Research doc `.planning/research/S22_contribution_family_sybil_matrix.md` (40+ sources)
3. Exécuter cycle kickoff → plan → phases A-F → verification → audit_plan normal.

---

## Scope pré-calibré (à ajuster sur data réelle au moment de l'activation)

### But du sprint

Livrer l'Option F consolidée (composition asymétrique 3 couches) avec coefficients + rate-caps + quality-gates **calibrés sur data empirique collectée depuis Gate 2 unlock**. Fermer la vulnérabilité LLM-farming des familles non-compute.

### D1..D5 placeholders

**D1 — Taxonomie ContributionType**

- Introduire enum `ContributionType::{Compute, Funding, Code, Docs, Review, Moderation, Design, Accessibility}` Rust + Python équivalent
- Bump `KUDOS_VERSION` si post-v1.0 tag posé (sinon redéfinition v1 en pre-launch policy)
- Schema `KudosEntry` étendu avec champ `contribution_type`
- Wire spec déjà documenté dans `docs/fairness/KUDOS_V2_WIRE.md` (livré S23 Phase C)

**D2 — Couche A (compute + funding)**

- Aucune change. Status quo. Cost-based Sybil defense naturelle.
- Rate-cap global StackOverflow-style éventuellement ajouté (200 kudos/jour global, 15 ans précédent empirique).

**D3 — Couche B (code + docs) : evaluator committee clawback**

- Pattern : RetroPGF-style (Optimism R1-R5 2022-2026)
- `KudosEntry` frozen 30 jours post-award
- Committee rotatif (membres = contributeurs compute confirmés trust ≥ 0.8 + time-weighted ≥ 6 mois)
- Endpoint `/api/kudos/evaluate/{id}` pour committee input
- Clawback si evaluation négative majority → kudos annulé
- Coefficient voting-weight final basé sur evaluation score (pas votable runtime)

**D4 — Couche C (review + moderation + design + accessibility) : invitation Protocol-Guild**

- Pattern : Protocol Guild ($100M+ 2022-2025, zéro capture) + Apache/Debian/Linux MAINTAINERS
- Non-votable, time-weighted `sqrt(months_active)` weight
- Invitation = signature d'un membre existant time-weighted ≥ 12 mois sur la famille
- **Pas de score numérique** (intentionnel — les familles intrinsèquement sociales ne se quantifient pas proprement)
- Représentation dans `ContributorAttestation` S22 Couche 2 étendue (binaire : membre ou non, avec timestamp d'invitation signé)

**D5 — Gouvernance coefficients**

- Coefficients finaux Couche B (code + docs) calibrés sur data Gate 2 empirique
- **Non-votable en gouvernance Type C** (précédent Steem 72h capture, SourceCred MakerDAO arrêt)
- Mise à jour via PR + audit gate (pattern SBFB standard)
- Si future-future need (post-LT-3), vote externe Type C peut être ajouté séparément

### Phases A-F projetées

- **Phase A** : `ContributionType` schema Rust + Python (+250 LOC + tests)
- **Phase B** : Couche B evaluator committee implementation (+600 LOC + tests)
- **Phase C** : Couche C invitation mechanism étendu `ContributorAttestation` (+400 LOC + tests)
- **Phase D** : KudosEntry schema migration + SQLite backfill (+250 LOC + tests)
- **Phase E** : Integration + observability update (`/diagnostic/fairness` étendu par famille) (+150 LOC + tests)
- **Phase F** : Wrap + verification + audit_plan

**Budget total estimé** : ~1650 LOC + ~200 tests. **Calibration à ajuster sur data empirique au moment de l'activation**.

### Research grounding

- `.planning/research/S22_contribution_family_sybil_matrix.md` (commit `dbc4ceb`) — 40+ sources agrégées
- `docs/fairness/CONTRIBUTION_FAMILIES_V1.md` (livré S23 Phase C, design-only)
- `docs/fairness/KUDOS_V2_WIRE.md` (livré S23 Phase C, design-only)

### Scope cuts préservés de l'Option D rejetée

- **Coefficients votables par gouvernance Type C** : REJETÉ (Steem 72h, SourceCred MakerDAO, BrightID capture). Remplacé par coefficients calibrés sur data + mise à jour via PR standard.
- **Rate caps par famille chiffrés a priori** (50 PR/sem, 20 docs/sem) : REJETÉ (log-normal kernel Linux, arbitraire). Remplacé par clawback post-hoc RetroPGF (pas a priori).
- **Règle workflow "ship lockstep"** : REJETÉ (aucun précédent dans 15 systèmes OSS étudiés). Remplacé par pattern empiriquement validé RetroPGF + Protocol Guild + Apache.

---

## Triggers de désactivation

Cet stub peut être **archivé / supprimé** si :

- `docs/FAIRNESS_VISION.md` est mis à jour pour dire explicitement que les familles non-compute restent **socialement reconnues sans score** à perpétuité (décision produit assumée)
- ET aucun trigger LT-3 (a/b/c) n'a fire 18+ mois après Gate 2 unlock
- ET le user (FlowUP) ou successor lead contribution-family pose un commit de désactivation explicite

Pattern : `chore(planning): archive LT-3 stub (sociale recognition sufficient, no quantification needed)` dans un sprint futur wrap-up.

---

**Dernière revue** : 2026-04-20 (création du stub, pattern `.planning/reserved/` nouveau). Ce stub n'apparaît ni dans `.planning/active/` ni dans `.planning/archive/` — il vit dans un limbe `reserved/` jusqu'à activation ou désactivation explicite.
