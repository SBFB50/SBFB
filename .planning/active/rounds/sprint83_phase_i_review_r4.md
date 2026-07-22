# Sprint 83 Phase I — Review r4 (réconciliation Codex r3)

## Verdict: PASS-PENDING

Réconciliation du round Codex r3 (**GAP : 0 P0, 1 P1, 2 P2** — F3/F4
confirmés, F1 partiel doc, F2 gap). Fermetures :

1. **P1 F2 — validation restore devenue SÉMANTIQUE** : pragmas typés
   `index_list('outbox')` (l'index doit exister ET être UNIQUE) +
   `index_info` (colonnes exactes `(run_id, node_instance_id)`) ; flags
   PK vérifiés par table (`table_info` col 5) pour
   runs/run_events/outbox/inbox/snapshots/lease ; FK DÉCLARÉES exigées
   (`foreign_key_list` ≥ 1) pour les 4 tables enfants. Test NEUF : un
   VRAI backup dégradé (index remplacé par un homonyme NON-unique) →
   refus « not UNIQUE ». **Mutation M11** (check unicité neutralisé) →
   rouge. Bornes restantes documentées §19/OPS : types, CHECK et défauts
   non comparés.
2. **P2 F1 — doc de la sémantique absorbante** : Rustdoc
   `EngineError::StepHashMismatch` reformulé (RUNNING → terminal ;
   COMPLETED garde son histoire honnête) ; doc du test
   `store_replay_diverges_fails_closed` reformulée avec **supersession
   explicite** de l'assertion Phase E (`sprint83_e_durable_spike.json`
   délibérément NON réécrit — artefact historique) ; la claim r1 « harnais
   E adapté par pur renommage » est SUPERSÉDÉE (le round 3 a changé la
   sémantique du bras divergent-sur-completed, tracé).
3. **P2 F2 — docs** : §19 T-RUN-RESTORE et STORE_MIGRATION_OPS
   reformulés (« vérifié sémantiquement » + bornes honnêtes).

## Suites

Crates workflow 146 passed + 2 skipped ; **11/11 mutations tuées**
(M1..M11) ; fmt OK ; workspace Win + Docker relancés (compteurs au body).

## Prochaine étape

Codex r4 (clôture attendue) → promotion canonique
`sprint83_phase_i_review.md` à `## Verdict: PASS` → commit.
