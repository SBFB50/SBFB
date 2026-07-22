# Sprint 83 Phase I — Review r2 (réconciliation Codex r1)

## Verdict: PASS-PENDING

Round de réconciliation après Codex r1 (**GAP : 0 P0, 3 P1, 7 P2** —
`rounds/sprint83_phase_i_codex_review_r1.md`, artefact brut jamais réécrit).
Tous les findings jugés RÉELS et traités root-cause ; PASS-PENDING en
attente du re-run Codex r2 sur l'arbre corrigé.

## Traitement des 3 P1

1. **P1 L3/L9 — le lease ne fence pas l'effet EXTERNE** (writer suspendu
   entre intent committé et `target.apply`, déposé entre-temps, atteint
   encore la cible). Fait EXACT — c'est la frontière de COUCHE : le CAS
   fence les écritures du STORE ; la fenêtre d'effet est la couche effets.
   Traitement = reformulation honnête partout (la claim était trop forte,
   pas le code faux) : §19 T-RUN-DOUBLE-EMIT rescopée « fermée (couche
   STORE, mode leased) » + borne honnête (2) explicite (dedup de la CIBLE
   = clause 2 du spike ; fermeture par fencing-token vérifié PAR LA CIBLE
   = **Phase K**, carry tracé) ; `lib.rs` clause 3 amendée idem.
2. **P1 L6/L9 — TOCTOU statut GC** (statut lu au scan, purge branchée
   dessus : un diagnostic terminal posé entre scan et purge partait par la
   branche completed). FIX CODE : le scan ne remonte plus que les
   `run_id` ; statut + `closed_at_ms` + éligibilité + pending sont RELUS
   DANS la transaction de purge ; row disparue / rouverte / re-datée →
   skip. `FaultPoint::BeforeGcPurgeOfRun` NEUF (hors-txn, documenté) pour
   le test d'interleaving `store_gc_rereads_status_inside_purge_transaction`
   (hook qui flippe le run en terminal via une connexion SÉPARÉE →
   re-dispatch branche terminale, diagnostic survit). **Mutation M8**
   (relecture neutralisée) → test rouge → restaurée.
3. **P1 L7/L9 — validation backup insuffisante** (SQLite valide +
   user_version courant + table `runs` seule écrasait un store sain). FIX
   CODE : `user_version >= 1` exigé + INVENTAIRE COMPLET des tables par
   version réclamée (v1 = tables E ; v2 = les 6) + `PRAGMA quick_check`
   == ok, le tout AVANT tout remplacement. Test
   `store_backup_forged_minimal_refused_...` (forgeage minimal → refus).

## Traitement des P2

- **run_id absent du digest snapshot** → `snapshot_digest_subject` prend
  le run propriétaire (les 2 sites) ; test de substitution cross-run →
  `Poisoned` ; §19 reformulée « RUN PROPRIÉTAIRE + tuple servi entier ».
- **min_age_ms négatif** (cutoff dans le futur) → `InvalidInput` (+ clock
  négatif), testé.
- **alias backup** → garde : la destination (et son `.tmp`) ne peut pas
  résoudre vers le fichier DB vivant ni ses siblings WAL
  (`Connection::path()` + canonicalisation du parent) ; testé (db et
  `-wal` refusés).
- **bras out-of-process ne fermait pas le run** → le parent joue
  `record_run_completed` et asserte `Completed`.
- **`effect_id` injectable en prod (B7)** → `record_intent_atomic` /
  `record_completion` deviennent `pub(crate)` en build normal (jumeaux
  cfg : `pub` uniquement sous `spike-faults` pour le harnais) —
  l'effect_id est engine-dérivé, plus aucune porte publique n'en accepte.
- **M1 réindenté** → literal de la migration E restauré BYTE-IDENTIQUE
  (1728 octets, extrait de `e034a574` et réinséré tel quel) ; le claim
  forward-only redevient vrai au sens byte.
- **§19 / STORE_MIGRATION_OPS** → reformulations correspondantes faites
  (double-emit rescopé, GC relu, backup inventaire+quick_check, snapshot
  run-bound).

## Suites après corrections

Crates workflow Win : **145 passed + 2 skipped** (142 → +3 tests Codex).
Mutations : **8/8 tuées** (M1..M7 + M8 relecture GC). fmt --check OK.
Workspace Win + Docker canonique relancés (résultats consignés au body du
commit) ; rouge unique antérieur du workspace Docker
(`start_writes_running_json_and_responds_to_health`) = flake env
Docker-on-Windows, solo-PASS 0.65s, hors périmètre phase.

## Prochaine étape

Codex r2 sur l'arbre corrigé (un artefact périmé ferait réfuter des
corrections déjà faites), puis promotion du canonique
`sprint83_phase_i_review.md` à `## Verdict: PASS`.
