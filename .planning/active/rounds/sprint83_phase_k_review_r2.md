# Review r2 — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: PASS-PENDING

Round r2 = re-review Workflow `wf_2a1cbc43` (3 agents Opus 4.8 1M
adversariaux, 578 649 tokens, 145 tool uses, mutations de prod appliquées
puis reverties byte-exact) sur les corrections du round r1, + corrections
du P2 trouvé par r2 lui-même. Rapports : scratchpad session
`rereview_k/{fs_process,orch_reducer,tests_docs}.md`.

## Corrections r1 vérifiées (3 agents)

- **FIX 1 symlink feuille** : CORRECT+COMPLET — refus ferme
  T-RUN-FS-EFFECT-ESCAPE (`confine` renvoie le chemin parent-canonicalisé
  réutilisé à l'identique par read+create), TOCTOU documenté honnêtement,
  test unix discriminant, précondition d'état AVANT confine/write.
- **FIX 2 charte env** : CORRECT+COMPLET — 0 lecture d'env ambiante en prod
  (grep), charte `lib.rs:104` + THREAT désormais VRAIES, env_clear + map
  hôte + EFFECT_INPUT_ENV, résidu /proc documenté.
- **FIX 3 fail-closed 3b** : CORRECT — avec-arête route, sans-arête ferme
  `node_failed`, fail-workflow ferme ; mutant « drop has_error_edge » tué ;
  `route_error_port_delivers_error_edge` non régressé.
- **FIX 4 reconcile binding** : CORRECT — 4 champs liés, echo brut accepté
  légitimement (borné par request_hash à la porte completion), mutant
  « drop effect_id » tué.
- **FIX reducer join_fired** : behavior-preserving, reducer PUR, 2 tests
  join verts.
- **FIX 5b kill child (Windows)** : les 2 branches d'erreur tuent le child.
- 3 mutations de prod (gel cohorte join, garde blanchiment, reap_orphan
  no-op) ont fait rougir les tests attendus — suite NON vacueuse.

## P2 trouvé par r2 — CORRIGÉ

- **Ordre non-atomique du chemin non-park** (introduit par MON fix r1 du
  P3 « row attempts ouverte ») : `close_open_attempt` (tx1) PUIS
  `record_terminal` (tx2) — un crash entre les deux laissait un run
  Running avec une row attempt fermée → collision PK de
  `record_attempt_started` au resume → run définitivement bloqué (prouvé
  empiriquement par l'agent, UNIQUE constraint). FIX : ordre INVERSÉ —
  `record_terminal` D'ABORD (le diagnostic est durable), puis
  `close_open_attempt` ; un crash entre les deux laisse l'état bénin
  pré-fix (row ouverte sur run clos, purgée par GC, jamais repris) ;
  doc-comments store+orchestrator réécrits, assertion
  `open_attempts == []` ajoutée à
  `non_idempotent_without_receipt_becomes_unknown`.

## P3 r2 (corrigés ou actés)

- Précédence env : `EFFECT_INPUT_ENV` appliqué EN DERNIER (une clé
  réservée dans la map hôte ne peut plus l'ombrer) — corrigé.
- Lien dur feuille même-FS non distingué par `symlink_metadata` — borne
  honnête ajoutée à la row THREAT (frontière OS mono-utilisateur).
- T-RUN-CANCEL-DOS champ Tests complété (test K du drain).
- `fs_effect_leaf_symlink_refused` cfg(unix) : exécuté via Docker
  (212/212 crates workflow) — acté.
- Orphan test : incarnations partagent le même flag idempotent (ne
  discrimine pas claim-journalé vs live) — le code de prod lit bien le
  flag journalé ; acté P3, mutation reap_orphan tuée par le test.

## Vérification §7.4 (état final)

- Windows : fmt 0 ; clippy workspace -D warnings OK ; nextest workspace
  **2348/2348 + 6 skip** (relance ciblée engine post-P2 : 100/100 + 6
  skip, fmt/clippy re-verts) ; doctests OK ; release build OK.
- Docker sbfb-ci : fmt OK ; clippy OK ; crates workflow **212/212 + 6
  skip** (inclut `fs_effect_leaf_symlink_refused` unix) ; workspace
  2345/2345 (run canonique pré-fixes) puis rouge UNIQUEMENT sur 2 e2e
  daemon de la classe flake documentée Docker-on-Windows (carry 7) —
  re-run SOLO 2/2 PASS.
- Web : bloc complet (lint/tsc/vitest/build/size/scan-en) exit 0 ;
  operator vitest exit 0 (aucun fichier web/ touché).
- Gates : check-frontier-contracts clean ; census DOMAIN 29 ; goldens
  run_event + orchestrateur inchangés ; 0 nœud lock neuf.

## Codex reconciliation

(sera renseignée au round Codex — PASS-PENDING = review OK, Codex pas
encore fait ; jamais un verdict final committable)
