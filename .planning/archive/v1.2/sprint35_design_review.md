# Sprint 35 — Design Review Board (G1)

**Date** : 2026-04-28
**Reviewer** : agent Explore independant (session fraiche)
**Input** : sprint35_kickoff.md §4 D1..D5

## Scoring

| Decision | Score | Rationale |
|---|---|---|
| D1 — crate nexus-coordinator-rs separe | ✅ | Separation nette business logic vs P2P networking. Coherent avec l'architecture existante. |
| D2 — migration graduelle | ✅ | Risque minimal, endpoints paralleles sans conflit port. |
| D3 — rusqlite + coordinator.db separe | ⚠️ | Pas de strategie schema versioning pendant cohabitation Python/Rust. |
| D4 — dispatcher Rust natif sans PyO3 | ✅ | Elimine le round-trip, utilise les types existants. |
| D5 — MANDATORY 3/3 resolution | ⚠️ | REPO_URL blocker externe = carry silencieux malgre TODO. |

## Blind spots identifies

1. **Schema versioning coordinator.db** (D3) : pendant la migration
   graduelle, Python et Rust ecrivent dans des DBs separees mais
   avec le meme schema logique. Si le schema Rust diverge avant
   que Python soit retire, les donnees ne seront pas migrables.
   **Recommandation** : ajouter `schema_version` table dans Phase A.

2. **REPO_URL carry risk** (D5) : le TODO comment ne bloque pas
   mecaniquement le carry. L'audit gate S36 doit explicitement
   verifier si le repo est public.
   **Recommandation** : l'audit plan S36 doit avoir un check
   "REPO_URL resolved or blocker still valid".

3. **Dispatcher parity testing** (D4/Phase B) : aucune mention de
   test de parite entre le dispatcher Rust et le dispatcher Python
   pour verifier que les TaskEntry signes sont byte-identical.
   **Recommandation** : Phase B doit inclure un test qui signe la
   meme TaskSubmission via Python (PyO3) ET Rust natif, et assert
   canonical bytes identiques.

## Phase feasibility

- Phase A : serree mais realiste (~300 LOC nouveau + 3 MANDATORY)
- Phase B : 200-300 LOC avec types existants, faisable
- Phase C : complexite async medium (tokio subscription loop),
  risque R1 identifie dans le risk register
- Phase D : routine

**Verdict** : scope realiste pour 3 phases feat + 1 wrap-up.

## Rigor signal

2/5 ⚠️, 0 ❌. G4 satisfait.
