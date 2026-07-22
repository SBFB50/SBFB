# Sprint 83 Phase I1 — Review round r1 (Workflow ultracode)

Date : 2026-07-22. Arbre : HEAD `e48d8280` + diff I1 non commité.
Méthode : Workflow `wf_5f80fb5d-b0f` — 6 dimensions (diff ligne-à-ligne, tests
sémantiques, scope/preflight compliance, sécurité deep, docs exactitude,
livrables/commit-readiness) + 2 agents adversariaux (réfutation + complétude),
8/8 agents Opus 4.8 1M, 174 tool-uses.

## Verdict r1 : CONCERN (0 P0, 0 P1, 5 P2, 10 P3) — corrections in-phase requises

## Constats POSITIFS structurants

- Routes enregistrées dans `authed_routes` AVANT le layer `auth_required`
  (`http.rs`), jamais `public_routes` ; compte 89→91 vérifié ; mapping
  RunStatus/LeaseState exact ; erreurs 500/400/200-honnête conformes ;
  Mutex jamais tenu à travers un `.await` ; boot miroir `coordinator.db` ;
  engine/core byte-intacts ; census 29 ; +1 ligne lock intra-workspace seule.
- Les 4 mutations pré-jouées (M1..M4) sont réellement discriminantes.
- Livrables I1 (promesse body I) intégralement fermés par la coupe read-only
  DATÉE (dim6 : « recevable »).

## Findings P2 (à corriger avant Codex)

1. **P2 REVIEW-1** — `workflow_api.rs:119` bras `Completed => "completed"`
   jamais exercé ; mutation `Completed => "running"` SURVIT. L'état est
   atteignable daemon-side via `record_run_completed` (**pub**, store.rs:1072)
   — le P3 « structurel » de dim1 est RÉFUTÉ par adv1. → test + mutation M-5.
2. **P2 REVIEW-2** — test duress vert-pour-mauvaise-raison (empty==empty) :
   ne prouve pas « les reads servent les données réelles sous duress » (A7).
   → seeder un run sous boot duress et asserter `found:true`.
3. **P2 REVIEW-3** — `LOOPBACK…:5` « 33 rows = 31 route-paths » : 31 ne
   correspond à aucune méthodologie (33 − 1 dup consent − 4 FUTURS = **28**
   sous la méthodologie du 27 antérieur). → corriger 31→28.
4. **P2 REVIEW-4** — `THREAT_MODEL.md` row T-RUN-DOUBLE-EMIT « le chemin
   daemon (I1/J) acquiert » non propagée (contredit le changelog v24 du même
   commit et le code : 0 acquire daemon en I1). → amender la row vivante.
5. **P2 REVIEW-5** — warning `dead_code` nouvellement surfacé :
   `record_intent_atomic`/`record_completion` (pub(crate) engine) morts pour
   le 1er consommateur spike-faults-OFF. `cargo clippy -p nexus-shell-daemon
   --all-targets -- -D warnings` exit 101 ; gate canonique `--workspace`
   exit 0 (unification spike-faults via self-dev-dep). Fix engine INTERDIT
   (byte-intact). → ACTER au body, résorption Phase J (le runtime consommera
   ces portes).

## Findings P3 (documenter / accepter)

- Delta tests réel = +9 à r1 (8 in-module + 1 famille golden ; les 2 sondes
  sont des itérations de `golden_families_stable`, pas des tests nextest).
- Enveloppe livrée IMBRIQUÉE `{found, run:{…}}`/`{found, lease:{…}}` vs
  esquisse plate/`held` du preflight A2 — bénin, cohérent shard-session,
  allowlist identique → à RATIFIER dans le preflight (note r1).
- Chemin 500 `store_error_response` non couvert (déclenchement = store
  corrompu ; faible valeur) → acté au body.
- Boot prod `RunStore::open` non unit-testé (même classe que coordinator.db,
  précédent accepté) ; couplage disponibilité daemon↔store fail-closed =
  choix assumé (garde downgrade LOUD) → acté.
- Doc-comment engine `store.rs:453` « The daemon path acquires » périmé pour
  I1 (édition interdite ; redevient vrai en J) → tracé, résorption J.
- Sondes stables : double-run consécutif à produire (§P75).
- `"completed"` documenté (DTO + REFERENCE) : devient exercé avec REVIEW-1.
- `main.rs`/`curators_api.rs` : vérifiés par adv2, zéro défaut.

## Suites au moment de r1

Win : fmt/clippy/nextest 2282+2skip/doctests/release VERTS. Docker :
fmt + nextest 2286+2skip VERTS (`--no-fail-fast` ; 2 flakes de charge des
classes documentées re-runs solo PASS). Frontier clean (census 29), SPDX 395,
routes 91, engine diff vide. Web 548 (re-run solo PASS), untouched.
