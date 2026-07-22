# Sprint 83 Phase I — Review r1 (fan-out 6 dimensions + adversarial)

## Verdict: PASS-PENDING

Run store durable SQLite (`crates/sbfb-workflow-engine/src/store.rs` +
`crates/sbfb-workflow-core/src/run_event.rs`). 0 P0/P1 confirmé après
correctifs in-phase. PASS-PENDING = review interne OK, Codex pas encore
joué — jamais un verdict final committable.

**Date** : 2026-07-22. **Base** : working tree sur HEAD `e034a574`,
preflight PLAN-ADAPT (23 adaptations, décisions B1-B8).

## Méthode (honnêteté d'exécution)

Review lancée en Workflow ultracode (6 dimensions + 2 adversariaux +
synthèse, run `wf_16c5c37e-cc5`). Le run a été **interrompu** (agents en
échec au cap StructuredOutput, arrêt utilisateur) après que 4 dimensions
ont rendu leur verdict — dont la dimension SÉCURITÉ rendue sous forme de
**placeholder vide** (même mode d'échec que le scan S1a du preflight,
écarté). Fallback canonique appliqué : les résultats des 4 dimensions
récupérés du journal du Workflow (`review_dims_cached.json`), puis
**D1 diff-correctness, D3 scope-plan et D4 sécurité REJOUÉES en agents
avant-plan** à retour texte, puis passe adversariale finale (réfutation
des 8 correctifs + complétude) en agent avant-plan. Synthèse main-thread
(ce fichier).

## Dimensions et verdicts

| Dim | Verdict | P0/P1 | Substantiel |
|---|---|---|---|
| D1 diff-correctness (rejouée) | PASS-PENDING | 0 | D1-1 P3 clock négatif ; D1-3 INFO ordre siblings restore |
| D2 tests-semantics (Workflow) | PASS-PENDING | 0 | D2-01/02/03/04 gaps de couverture (P2/P3) |
| D3 scope-plan (rejouée) | PASS-PENDING | 0 | 21/23 adaptations suivies, 2 partielles (reliquat = body I.5) ; 0 trou plan |
| D4 sécurité (rejouée) | PASS-PENDING | 0 | D4-1 P2 digest snapshot state-seul ; D4-2 P3 TOCTOU restore |
| D5 docs-honnêteté (Workflow) | CONCERN→levé | 0 | D5-1 P2 doc périmé lib.rs core ; D5-2 P3 doc fault.rs |
| D6 process-livrables (Workflow) | PASS-PENDING | 0 | P6-2 Docker (depuis vert) ; P6-3 formule « +2 skip » ; staging 17+ items |
| Adversarial final | PASS-PENDING | 0 | 8 correctifs CONFIRMÉS ; ADV-1/2/3/4 (P3/INFO) |

## Correctifs appliqués in-phase (tous re-vérifiés par l'adversarial)

1. **Monotonie du lease à travers release** (trouvé en instruisant D2-02) :
   `release_lease` faisait DELETE de la row → un re-acquire du même owner
   repartait à epoch 1 et un handle zombie `(owner, 1)` d'une incarnation
   antérieure repassait le CAS. Fix root-cause : la row lease est
   PERMANENTE, release marque libre (`owner=''`, expiry 0) en CONSERVANT
   l'epoch ; `lease_state` filtre le holder vide. Test
   `store_single_writer_renew_release_keep_epoch_monotone` (epoch 1→2 à
   travers release/re-acquire + probe CAS zombie = 0 match).
2. **D4-1 (P2)** : le digest de snapshot couvre désormais le TUPLE servi
   entier via `snapshot_digest_subject(version, upto_sequence, state)` aux
   deux sites (write + verify) ; bras de test « lying upto_sequence sous
   state intact → Poisoned » ; §19 T-RUN-SNAPSHOT-POISON reformulée.
   **Mutation M7** (digest state-seul) jouée → test rouge → restaurée.
3. **D2-01/02/03/04** : tests renew-succès (+ expiry prolongé + LeaseHeld
   jusqu'au nouveau expiry), release/re-acquire, restore PAR-DESSUS un
   store existant (siblings `-wal`/`-shm` plantés → supprimés, journal =
   celui du backup), gardes input lease (ttl<=0, overflow, owner à
   contrôle, clock négatif acquire ET renew [ADV-1]).
4. **D1-1 (P3)** : garde `now_ms < 0` → `InvalidInput{lease clock}` dans
   acquire et renew (une horloge négative rendait la row libérée
   inacquérable — fail-closed, mais refusé plutôt que documenté).
5. **D1-3** : restore supprime les siblings AVANT `fs::copy` (le mauvais
   entrelacement devient « old main + no wal », inoffensif, au lieu de
   « new main + old wal », corruption au rejeu).
6. **D4-2/ADV-2 (P3)** : doc restore porte le résiduel OS-local complet
   (fenêtre validation→copy, symlinks suivis, copy non-atomique — remède =
   rejouer le restore, le backup validé est toujours là).
7. **D4-3, D5-1, D5-2, ADV-3** : précisions doc (§19 sous-clause
   `format!`, lib.rs core « 6/7 as of Phase I », fault.rs point GC,
   contexte de digest snapshot).

## Findings restants (non bloquants, tous P3/INFO documentés)

- D1-2/complétude (INFO) : `contracts.schema.json` — l'ordre des 7 clés a
  flippé (IndexMap→BTreeMap selon le profil de build qui régénère) ;
  NON-BUG prouvé : `schema_snapshot_stub` compare des `serde_json::Value`
  (égalité indépendante de l'ordre) — aucun risque de rouge CI.
- D1-4 (INFO) : `backup_into` remplace un `.tmp` homonyme préexistant
  (documenté : un tmp = copie interrompue par définition).
- D2-05 (INFO) : la garde de fence n'a pas de porte dégradée dédiée —
  choix documenté `fault.rs` (CAS = no-op read-shaped), non-vacuité
  portée par les assertions du bras fenced-out + mutation M1.
- D3-4 (INFO) : seul `InsideGcTransaction` est un FaultPoint neuf ;
  lease/inbox/snapshot single-statement = atomicité SQLite (« a removal
  arm would test the database ») — écart à la lettre de ADD-C4, décision
  documentée dans `fault.rs`.
- ADV-4 (INFO) : la précondition « store fermé » du restore n'est
  mécaniquement gardée que sous Windows (sharing violation) — la doc le
  dit pour Windows ; sous Unix c'est la discipline d'appel.
- D5-3 (INFO, HORS-SCOPE Phase I) : `nexus-core-rs/src/canonical.rs:389`
  décrit encore `CompiledPlan.v1` comme « still an empty stub » (dette
  Phase H, fichier hors du diff I) — **routé à l'audit gate / wrap-up**.
- D3-3 (P3) : reliquat des adaptations 15/21 = obligations du corps de
  commit I.5 (citations `Cargo.toml:137-138`, arti-client `463-467`, note
  QPO-3 `>=0.37.0`, integration-nightly N-A, mapping noms de tests plan
  → livrés).

## Mutations actives (feedback_mutation_before_trust) : 7/7 tuées

M1 CAS lease neutralisé → fencing rouge ; M2 garde downgrade retirée
(core) → refuses-newer rouge ; M3 digest snapshot sauté → poison rouge ;
M4 garde GC pending retirée → gc rouge ; M5 garde user_version retirée →
schema-guard rouge ; M6 `INSERT OR IGNORE` dégradé → inbox rouge ;
M7 digest tuple→state-seul → lying-upto rouge. Suite restaurée verte
après chaque mutation.

## Suites (état à la clôture de r1)

- Win : fmt --check OK ; clippy workspace -D warnings OK ; nextest
  workspace **2269 passed + 2 skipped** (2251 fin H → +18 : +5 core
  run_event, +13 tests/run_store.rs ; les 2 skipped = les 2 childs
  `#[ignore]` out-of-process spawnés par leurs parents) ; doctests OK ;
  release build OK.
- Docker canonique `sbfb-ci` (1.94) : fmt --check OK ; crates workflow
  142/142 + 2 skipped ; workspace complet en cours au moment de r1
  (résultat au commit).
- Gates : `check-frontier-contracts.sh` clean (census DOMAIN 29 frozen) ;
  `check-spdx.sh` 394 OK ; filtres de gates `store`/`upgrade_replay`/
  `single_writer` matchent chacun des tests verts, 0 nom de test
  contenant `effect`.
- Web 548 + coverage seuils tenus + build + size ; operator 201 (aucun
  fichier front touché, T1 = N-A-no-frontend-change).

## Prochaine étape

Codex GPT-5.6 Sol (round r1) sur l'arbre corrigé, puis réconciliation et
promotion du canonique `sprint83_phase_i_review.md` à `## Verdict: PASS`.
