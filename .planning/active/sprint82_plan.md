# Sprint 82 — Plan : Dette docs-contrat + Refactorisation (20 phases A→T)

> **ACTIVABLE** (Cas C ; Phase 0 = audit gate S81 JOUÉE — verdict FAIL levé voie A).
> Source unique : `sprint82_kickoff.md` + le SPEC VERROUILLÉ (AUTORITÉ : décisions PO
> PO-1..PO-9 + Day-0 D1..D12 gelés). S82 = **sprint DETTE DOCS-CONTRAT + REFACTORISATION**
> (C9 tranché PO 2026-07-11) ; workflow-engine + Viewer fondation DÉCALÉS (futurs slots,
> ratifiés Phase E / PO-9). **20 phases : 0 + A→T.** Phases dimensionnées par le **travail**,
> JAMAIS par LOC (README §4 ne plafonne pas ; le split http.rs 1-domaine=1-commit=1-phase
> [D3] porte le sprint à 20).

> 1 commit atomique par phase `feat(scope): Sprint 82 Phase X — titre` (ou
> `fix/docs/chore(...)` selon nature) ; **rigueur per-phase uniforme** : deep preflight
> (5 scans) → review Workflow → Codex avant CHAQUE commit ; body 9 sections ;
> `## Acceptance` (vocab fermé) écrit à CHAQUE phase (enforced lightcheck + audit Track J)
> sur le squelette voie-A `sprint81_verification.md`. Invariants tenus tout le sprint :
> **0 bump wire SBFB**, **0 dep runtime ajoutée hors hickory (PO-7=A)**, **iroh reste
> =1.0.1**, refacto = **0 changement de comportement observable** (count nextest = preuve),
> `heberger != publier, seeder != auteur`, Factory hors daemon.

> **Cadrage rig-chaud (D1).** Les Phases A (boot-SEED, exit-condition (c) live) et B
> (benchmarks) partagent le rig Mac+PC+VPS allumé — **clusterées et ordonnancées ensemble**.
> Phase A est CI-INDÉPENDANTE (rust-ci 3-OS vert, 0 GTK) : c'est ce qui autorise
> boot-SEED-first malgré la CI cassée. Le push groupé (Phase T) est gaté sur 3 verts
> (Woodpecker + rust-ci 3-OS + run nightly réel), APRÈS la réparation CI (C).

---

## Phase 0 — Audit gate S81 : JOUÉE le 2026-07-12

- **Verdict** : **FAIL** (0 P0, **4 P1**, 16 P2, 14 P3 ; Workflow ultracode 12 tracks
  A..K + vérification adversariale) — le cœur S81 est SOLIDE (`b3_p2_quorum` PASS, 1er
  quorum de l'histoire projet ; nextest Win 2095 / Docker 2099) ; les 4 P1 = gate /
  hygiène / consignation / escalade, pas des défauts de fond.
- **Gate LEVÉ voie A** (arbitrage PO) par `fix(sprint81)` `ad53940` (A3-1 E2E web
  ROUGE→GREEN 44/2skip via split 2 projets Playwright `dependencies` + J-1 section
  `## Acceptance` + A3-2 claim CI GHA-rouge requalifié + A3-3/I-1/K-1 P2) + chore
  `95ff46c` (findings, 352 l). **Ne PAS re-jouer.**
- **Carry P1 S81-G-ESC-1** (boot-SEED OVERDUE 3/3) → **routé Phase A** par construction
  (PO-1=B : réparer + preuve LIVE exigée). Les **16 P2 / 14 P3 doc-dette** (Tracks C/F/H/I/J/K
  + drifts iroh/honnêteté + catalog_len=0 + PROMISE_RE + ledgers zombies) → **routés Phases
  E-J** (dette éclatée par domaine, canon « nommé jamais bundlé »). S81-A3-2-infra GTK +
  integration-nightly → Phase C ; S81-G-1 migration stores → Phase T (artefact T2).
- **Baseline tests FIGÉE** (invariance refacto D4) : Rust nextest **2095** Win natif /
  **2099** Docker canonique ; Vitest `web/` **412** ; Vitest operator **201** ; E2E
  Playwright operator **10** ; E2E web **44 / 2 skip** (post fix voie A). Tip master
  `95ff46c` (LOCAL, non poussé ; origin/master `c899d54`, 20 commits d'avance).

---

## Cluster RIG-CHAUD (A + B) — rig Mac+PC+VPS allumé, ordonnancées ensemble

## Phase A — Convergence cold-boot catch-up (escalade boot-SEED, fermeture par construction)

- **Goal** : fermer S81-G-ESC-1 (OVERDUE 3/3) sous l'invariant unifié « le broadcast
  gossip est un HINT non fiable ; l'état durable synchronisé est la VÉRITÉ ; tout
  consommateur cold-boot RECONCILIE contre cet état une fois le neighborhood formé ».
  2 livrables conçus ENSEMBLE (D2) : (1) **ANCRE** = re-drive de `run_boot_seed_driver`
  à l'ingest annuaire (threader `boot_driver_state` + `keep_online_projects` dans la
  boucle gossip, re-drive narrowed idempotent par set pinné, duress-gate hérité, borne
  1/batch) ; (2) **WORKER** = catch-up des `task:` pendantes au cold-boot (réconciliation
  d'état forcée `start_sync(peers)` au `run_until_shutdown` après import + keepalive
  borné, scan `get_many_by_prefix` inchangé). 0 wire, 0 dep, 0 bump canonical.
- **Covers** : S82-ESC-BOOT-SEED-CATCHUP, LEDGER-ESC-BOOT-SEED, tests T1/T2 boot-SEED.
- **Livrables** : hook re-drive-on-ingest boucle gossip (`nexus-shell-daemon/src/runtime.rs`) ;
  réconciliation worker cold-boot (`nexus-worker-core/src/runtime.rs`) ; 2 tests nextest 2-nœuds
  hermétiques NEUFS avec contrôle red revert-proof ; mode `BOOT_AFTER_SUBMIT` dans
  `b3_live_pc_vps.sh` ; artefact `sprint82_t2_bootseed.json` ; doc-comment `http.rs:1819-1826`
  (driver ONE-SHOT) + `THREAT_MODEL.md` mis à jour à la fermeture ; invariant « broadcast=HINT,
  état durable=VÉRITÉ » consigné PATTERNS.
- **Testabilité** : **GATE PLEIN** (seule phase à comportement cross-machine). T1 = (a)
  test ancre 2-nœuds hermétique — SUBSCRIBE puis ingest annuaire couvrant le pid
  keep_online ⇒ app pinnée (tag skip-GC + `blob has()==true`) SANS restart, BLOQUANT-vert ;
  (b) test worker 2-nœuds — `task:` écrit par le vrai `dispatch_loop` AVANT boot worker ⇒
  claim+result dans le budget engine, BLOQUANT-vert ; les deux avec contrôle red
  revert-proof. **T2 = artefact JSON committé re-jeu run-2 (worker boot-froid 3 s avant
  submit) status=PASS delay<30 s — OBLIGATOIRE pour la clôture (PO-1=B). Rig indisponible ⇒
  escalade PO explicite, PAS de clôture silencieuse (jamais un 4e report sec).**
- **Risque** : **high**. Convergence cross-nœud = bloqueur dominant connu ; le T1
  hermétique DOIT être BLOQUANT-vert. PO-1=B ajoute une dépendance rig dure pour (c) — rig
  indispo ⇒ l'escalade OVERDUE 3/3 ne ferme pas ⇒ escalade PO. Couplage A↔L : le hook
  re-drive ajouté ici est absorbé par la décomposition Phase L — ne pas l'écraser.
- **frontier_closure** : N/A (0 nouvelle frontière loopback ; canal retour = docs/feed existant).

## Phase B — Benchmarks standards sharding + amendement canon T3 (PO-2)

- **Goal** : mesurer les perfs du sharding pipeline-parallel avec des outils standards :
  **llama-bench** (débit/latence) + **perplexity-parity** (entier-vs-shardé) + métriques
  **TTFT/TPOT/ITL** versionnées. Ratifier l'amendement canon T3 (README §4 : tier
  « benchmark » + track audit dédié + invariant kickoff). G8 design dédié au preflight de
  la phase (nouvelle primitive : harness de mesure + amendement canon). Cluster avec A (rig
  déjà chaud pour l'exit-condition (c)).
- **Covers** : S82-BENCHMARKS-STANDARDS, S82-CANON-T3-AMENDMENT.
- **Livrables** :
  - harness benchmark versionné (déterministe, runnable) ;
  - artefact JSON métriques committé (`sprint82_t2_benchmarks.json`, schéma versionné) ;
  - README §4 amendé (tier T3 benchmark + track audit dédié + invariant kickoff) ;
  - note perplexity-parity entier-vs-shardé.
- **Testabilité** : T1 = **N-A-no-frontend-change** (mesure back-end). Critère machine =
  harness runnable déterministe + artefact JSON métriques schéma versionné. T2 = artefact
  benchmark **PASS**, rig-gated (Mac/PC/VPS éteint ⇒ `BLOCK{rig}`, JAMAIS `RIG-ABSENT` — le
  rig est engagé pour A).
- **Risque** : **med** (rig-dépendant). Mitigation : clustering rig avec A ; l'amendement
  canon T3 est indépendant du rig (ratifiable même si BLOCK{rig} sur la mesure).
- **frontier_closure** : canon README §4 amendé (frontière process — tier benchmark T3).

## Restauration CI + assainissement

## Phase C — Restauration des surfaces CI (GHA/GTK, Woodpecker E2E, integration-nightly)

- **Goal** : rendre le volet « + CI chaque push » du gate testabilité OPÉRANT avant le
  refacto et le push. Trancher ARB-CI-1 : installer `libgtk-3-dev` dans les 2 jobs `ci.yml`
  (miroir image sbfb-ci) ⇒ GHA vert (matrice 3-OS clippy + operator vitest + E2E restaurés) ;
  OU décâbler honnêtement GHA des claims ET câbler les 2 suites Playwright E2E + coverage web
  + `scan-en-strings` dans `.woodpecker/ci-linux.yml`. Calibrer `integration-nightly.yml` :
  slow-timeout `binary(multi_daemon) >= 120 s` (K-R-13), save-if master-only (K-R-14).
- **Covers** : S81-A3-2-INFRA, S81-J-2-NIGHTLY-NEVER-RAN, S81-K-R-13, S81-K-R-14.
- **Livrables** : `ci.yml` corrigé (GTK dans les 2 jobs) OU décâblage honnête + E2E/coverage/scan
  câblés dans `.woodpecker/ci-linux.yml` ; `integration-nightly.yml` calibré (slow-timeout
  120 s + save-if master-only) ; claim CI CLAUDE.md réconcilié (finalisé Phase T).
- **Testabilité** : T1 = N-A-no-frontend-change. Critère machine = `gh run list --workflow
  CI --limit 5` = success sur le tip OU workflow retiré/renommé honnêtement + 2 E2E
  Playwright verts Woodpecker ; `integration-nightly.yml` actionlint-valide. T2 = N-A (run
  réel vérifié Phase T).
- **Risque** : **med**. La réparation GHA peut échouer côté runner ⇒ fallback Woodpecker
  E2E OBLIGATOIRE (jamais laisser le claim CI non tenu).
- **frontier_closure** : CLAUDE.md CI claim (frontière process — réconcilié Phase T).

## Phase D — Réparation relay-gated multi_daemon (baseline A3 4/10)

- **Goal** : réparer OU requalifier chaque test rouge `binary(multi_daemon)`
  (`nexus-test-harness` + `nexus-coordinator-rs`) : distinguer les **5 test-rot d'infra**
  du **1 signal produit réel** (gossip discovery). Compte final committé + rationale par
  test requalifié.
- **Covers** : RELAY-GATED-MULTI-DAEMON-REPAIR.
- **Livrables** : chaque `multi_daemon` rouge réparé (fix produit) OU requalifié (rationale
  committée) ; compte final committé + note env explicite.
- **Testabilité** : T1 = non-régression + `multi_daemon` ciblés verts/requalifiés. Critère
  machine = run local `SBFB_INTEGRATION=1` slow-timeout 120 s ; run distant vérifié Phase T.
  Note env : `multi_daemon` env-instable Docker-on-Windows (CLAUDE.md:408-412) — **re-run
  solo avant conclusion**. T2 = N-A.
- **Risque** : **med**. Env-instable Docker-on-Windows ; calibrer le timeout 120 s AVANT la
  réparation, ne pas confondre flake env et régression produit.
- **frontier_closure** : N/A.

## Dette docs-contrat (éclatée par domaine — canon « nommé, jamais bundlé »)

## Phase E — Réconciliation des ledgers de dette + purge zombies (D9)

- **Goal** : re-audit COMPLET des ~80 tickets T\* `docs/{rust,shell}/PATTERNS.md` : statuer
  chacun CLOSED / ZOMBIE-supprimé / OPEN-avec-ancre-grep-résolvable. Purger les zombies
  Python (T44-T51, ère Python — `git ls-files packages/` = 0 vérifié) ; résoudre la double
  numérotation T15/T16 (shell). Retirer les steps Python morts de `verify.sh`. Corriger le
  décompte stale « 8 P2/11 P3 docs-contract S80 » → réel **4 P2/10 P3**. Marquer
  `.planning/research/sprint82_workflow_engine/` **SUPERSEDED** + corriger
  `sprint82_audit_plan §6` (scope mis-scopé, PO-9). Statuer (SANS coder) les tickets
  hors-thème et les router à leurs owners.
- **Covers** : REFACTO-PATTERNS-LEDGER-ZOMBIE, CLEANUP-VERIFY-SH-DEAD-PY,
  S82-DC-S80-LOT-RECONCILE, LEDGER-STAGING-STALE, TICKETS-OUT-OF-THEME.
- **Livrables** : `docs/{rust,shell}/PATTERNS.md` re-audités (statut par ticket) ; zombies
  Python supprimés + collision T15/T16 résolue ; `verify.sh` sans step Python mort ; décompte
  S80 corrigé ; staging workflow-engine SUPERSEDED + `sprint82_audit_plan §6` corrigé (PO-9).
- **Testabilité** : T1 = N-A. Critère machine = `git ls-files packages/` = 0 ; tout T\* OPEN
  pointe un fichier existant (grep résout) ; 0 collision d'ID ; `verify.sh` s'exécute sans
  abort sur checkout frais ; 3 gates docs exit 0. T2 = N-A.
- **Risque** : **low**. Ré-extraction : ne PAS fabriquer de dette — statuer sur l'existant.
- **frontier_closure** : N/A.

## Phase F — PROMISE_RE aveugle + ancres task_response.rs (S79-P2-1 / S80-G-2, 2 reports)

- **Goal** : élargir `PROMISE_RE` (`check-frontier-contracts.sh:66`) pour matcher la classe
  « until/when Sprint N activates/lands » (aujourd'hui aveugle, vérifié). Réécrire/requalifier
  les 4 commentaires `task_response.rs` (:14 « empty at S20 — S22+ sandbox activates »,
  :84-85, :95 « does not bump when S22 lands » confirmés présents verbatim) vers le **passé
  immuable**. Test-mutation de non-vacuité du motif.
- **Covers** : S82-DC-PROMISE-RE-TASK-RESPONSE, LEDGER-S79-P2-1.
- **Livrables** : `check-frontier-contracts.sh:66` — `PROMISE_RE` élargi (classe until/when
  Sprint N) ; `task_response.rs` :14/:84-85/:95 réécrits au passé immuable ; fixture de
  non-vacuité prouvant que le motif détecte « until Sprint N activates ».
- **Testabilité** : T1 = N-A. Critère machine = `check-frontier-contracts.sh` exit 0 sur
  l'état final ET détecte « until Sprint N activates » sur un fixture (non-vacuité prouvée).
  T2 = N-A.
- **Risque** : **low**. 2 reports antérieurs (S79-P2-1 / S80-G-2) — aucun report sec de plus
  (§6.2.1).
- **frontier_closure** : N/A.

## Phase G — Contrat des corps de requête shard-session + registre FRONTIER (D8, fusion axe1+axe5)

- **Goal** : schématiser les 3 request-bodies loopback shard-session (`ShardGroupMintRequest`,
  `MountSessionRequest`, `ShardGenerateRequest` — `#[derive(Deserialize)]` seuls vérifiés,
  réponses déjà schématisées). Option (a) recommandée : `#[derive(JsonSchema)]` + snapshots
  `*.schema.json` drift-gatés, miroir des réponses ; repli (b) tables Request-body
  `SHARD_PROTOCOL_SPEC §6` + `FRONTIER-NO-SCHEMA` motivé. Figer la métrique exacte des
  familles `DOMAIN_*_V1` non-schématisées (grep déterministe committé, fin du flottement
  21/22/23). Acter la décision accept-and-close incrémental (D8) + trancher S80-G-1 doc-lint
  (3 reports, §6.2.1).
- **Covers** : SCHEMAS-SHARD-REQ, S82-DC-FRONTIER-REGISTRY-COVERAGE, S82-DC-S80-DOCLINT-ACCEPT-CLOSE.
- **Livrables** : 3 request-bodies shard-session schématisés (option a JsonSchema + snapshots
  drift-gatés, ou repli b tables SPEC §6 + FRONTIER-NO-SCHEMA motivé) ; métrique `DOMAIN_*_V1`
  non-schématisées figée (grep déterministe committé) ; D8 accept-and-close + S80-G-1 doc-lint tranché.
- **Testabilité** : T1 = N-A (si `web/src/api` touché ⇒ `npm run test:e2e` GREEN enregistré,
  leçon S81-J-1). Critère machine = `check-frontier-contracts.sh` + `check-sharding-docs.sh`
  exit 0 couvrant les 3 requêtes ; snapshots drift-gatés verts. T2 = N-A.
- **Risque** : **low**.
- **frontier_closure** : **Frontière NEUVE (3 request-bodies shard-session)** — indexée Phase T.

## Phase H — Doc-dette patterns Track C + tripwire suffixe backup (S81-C-1/C-2/C-3)

- **Goal** : fermer la dette patterns S81 Track C : re-ancrer T20 relay-cert-pinning au
  pointeur code valide (S81-C-3, `PATTERNS.md:974` pointeur faux confirmé), résoudre/requalifier
  C-1/C-2, vérifier §P73 fidèle. Factoriser/tripwire le magic-string suffixe backup
  redb-v2-tuples dupliqué 2 crates (F-D5-01, §6.9 — string OWNED par upstream iroh-docs).
  **Ré-extraire le texte exact C-1/C-2 depuis les phase-reviews** (les findings ne détaillent
  que les 4 P1 — NE PAS fabriquer).
- **Covers** : S82-DC-S81-C-PATTERNS, S81-C-3-T20-POINTER, F-D5-01-BACKUP-SUFFIX-TRIPWIRE.
- **Livrables** : T20 re-ancré au pointeur code valide ; C-1/C-2 résolus/requalifiés ; §P73
  vérifié ; tripwire/factorisation du suffixe backup redb-v2-tuples (2 crates).
- **Testabilité** : T1 = N-A (si tripwire = test Rust : nextest +1). Critère machine = grep du
  pointeur T20 résout ; tripwire asserte suffixe produit == littéral daemon `runtime.rs:2580`.
  T2 = N-A.
- **Risque** : **low**. Ré-extraction depuis les reviews obligatoire (dette réelle, non inventée).
- **frontier_closure** : N/A.

## Phase I — Doc-dette sécurité (Track H + drifts iroh/honnêteté + catalog_len=0)

- **Goal** : corriger/requalifier la doc-dette sécurité S81 pré-existante : H-1/H-2/H-3
  (THREAT_MODEL / LOOPBACK / HARDENING_ROADMAP), `VALIDATED_BLUEPRINT.md:156-157`
  iroh 0.97→=1.0.1 + gossip 0.101 (G-D5-1, vérifié stale), K-R-7 qualificatif « session
  réelle » + « byte-identical » sur-large (THREAT:148, SPEC §5.2, LOOPBACK §3), K-2 prose
  résiduelle, honnêteté claim cargo-audit (non installé — trancher câbler vs note subsomption
  cargo-deny). **catalog_len=0 seeder (PO-8) : accept-and-document consigné ici**
  (THREAT/PATTERNS, sort des carries — report répété depuis S75, décision fermante due §6.2.1).
  **Ré-extraire le texte exact H-1/H-2 des reviews.**
- **Covers** : S82-DC-S81-H-HARDENING, S82-DC-VALIDATED-BLUEPRINT-IROH-STALE, G-D5-1,
  S82-DC-SHARD-SPEC-HONESTY-KR7, S82-DC-S81-K2-PROSE, CARGO-AUDIT-CLAIM-HONESTY,
  S81-G-3-CATALOG-LEN-ACCEPT-DOC.
- **Livrables** : H-1/H-2/H-3 corrigés/requalifiés ; `VALIDATED_BLUEPRINT.md:156-157` iroh
  0.97→=1.0.1 + gossip 0.101 ; K-R-7 qualificatifs bornés + K-2 prose résiduelle nettoyée ;
  claim cargo-audit tranché (câbler vs note subsomption cargo-deny) ; catalog_len=0 seeder
  accept-and-document (THREAT/PATTERNS, PO-8).
- **Testabilité** : T1 = N-A. Critère machine = `grep '0.97' VALIDATED_BLUEPRINT.md` = 0 hit ;
  `check-sharding-docs.sh` exit 0 ; aucune contradiction code↔doc résiduelle (`last_validated`
  cohérent). T2 = N-A.
- **Risque** : **low**. Ré-extraction depuis les reviews ; catalog_len=0 = décision fermante
  (pas un report de plus).
- **frontier_closure** : N/A.

## Phase J — Doc-dette process/meta + ratification vocabulaire T2 (Tracks F, I, J)

- **Goal** : fermer/requalifier la dette d'artefact process : F-1..5 (hygiène fichiers-review :
  headers `## Verdict`, PASS-PENDING, cohérence identité), I-2/I-3 (bodies 9-sections / G8 /
  traçabilité), J-3/J-4/J-5 (consignation testabilité T1/T2). **RATIFIER au canon README §4**
  le vocabulaire palier-level T2 étendu **ACTED/MIXED/NOT-RUN** (S81-J-3) — frontière
  docs-contrat-sur-process lue par l'agent audit Track J. **Ré-extraire F-1..5, I-2, J-4/J-5
  depuis l'agent audit — NE PAS fabriquer leur contenu.**
- **Covers** : S82-META-S81-F-REVIEWFILES, S82-META-S81-I2, S82-TEST-S81-J-CONSIGNATION,
  S82-TEST-T2-VOCAB-RATIFY.
- **Livrables** : F-1..5 / I-2/I-3 / J-3/J-4/J-5 statués (0 perdu) ; README §4 amendé : tokens
  palier-level T2 ACTED/MIXED/NOT-RUN listés.
- **Testabilité** : T1 = N-A. Critère machine = grep conformité headers `## Verdict` vert ;
  README §4 liste les tokens palier-level ; tout P2/P3 Track F/I/J statué (0 perdu). T2 = N-A.
- **Risque** : **low**. Ré-extraction depuis l'agent audit obligatoire.
- **frontier_closure** : canon testabilité README §4 (frontière process — vocab T2 ratifié).

## Sécurité supply-chain (PO-7=A)

## Phase K — Bump hickory-resolver 0.24→0.26 (PO-7=A, supply-chain bornée)

- **Goal** : mettre à jour hickory-resolver (churn API réel du resolver 0.25 dans
  `dns_fallback.rs`), retirer les 4 ignores `deny.toml`, clore 4 RUSTSEC vivants. Le blocage
  S81 « iroh STRICTEMENT SEUL » est levé (sprint refacto = bon slot).
- **Covers** : HICKORY-024-RUSTSEC, hickory-bump.
- **Livrables** : construction resolver réécrite (`dns_fallback.rs`, churn API 0.25) ; 4 ignores
  `deny.toml` retirés ; 4 RUSTSEC vivants clos.
- **Testabilité** : T1 = non-régression + tests dns verts. Critère machine =
  `cargo deny check advisories` vert (4 ignores retirés) + nextest --workspace >= baseline
  (Win 2095 / Docker 2099). T2 = N-A.
- **Risque** : **med**. Churn API réel du resolver 0.25 ; découvrir au compile, documenter
  tout break dans le body. Seule dep runtime autorisée hors invariant (PO-7=A / D11).
- **frontier_closure** : N/A.

## Refacto (behavior-preserving, golden-gardée)

## Phase L — Refacto : décomposition DaemonRuntime::start() (~950 l)

- **Goal** : éclater `DaemonRuntime::start()` (`runtime.rs:276-1224`, ~950 l monolithiques)
  en sous-fonctions boot nommées <~150 l (identité, namespaces storage/feed, wiring gossip,
  listeners) ; regrouper les helpers annonce/outbox (handle_announcement/directory/project,
  normalize/serveable/prune/remint). Refactor pur : renommer/déplacer, JAMAIS toucher la
  logique ni la séquence de boot observable. **NOTE couplage : L absorbe le hook
  re-drive-on-ingest ajouté Phase A dans la boucle gossip — ne pas l'écraser (critic gap A↔L).**
- **Covers** : REFACTO-DAEMON-RUNTIME-START.
- **Livrables** : `start()` décomposé en sous-fonctions boot nommées <~150 l ; helpers
  annonce/outbox regroupés ; hook re-drive-on-ingest Phase A préservé dans la nouvelle structure.
- **Testabilité** : T1 = GREEN non-régression (suite existante verte + tests de boot verts +
  count nextest >= baseline). Critère machine (D4) = `cargo fmt --check` + `clippy
  --all-targets -D warnings` + nextest count invariant + 0 changement de séquence boot
  observable. T2 = N-A (chemin boot couvert par tests existants + T2 acceptance boot-SEED
  Phase A).
- **Risque** : **med**. Couplage dur avec le hook Phase A ; la séquence de boot observable
  ne doit pas bouger (count nextest = preuve).
- **frontier_closure** : N/A.

## Phase M — Refacto : golden de caractérisation HTTP + dédup harness de test

- **Goal** : **fondation du split http.rs** : établir un test de caractérisation/golden
  verrouillant l'identité pre/post des réponses HTTP sur les surfaces à extraire, AVANT tout
  déplacement de domaine. Consolider les 4+ `build_test_router*` dupliqués
  (`4645/4649/7534/8380`) en un seul constructeur paramétré (cors/web_root en options).
- **Covers** : REFACTO-HTTP-TEST-HARNESS-DEDUP, S82-TEST-REFACTO-NONREG.
- **Livrables** : test golden/caractérisation verrouillant l'identité pre/post des réponses
  HTTP ; `build_test_router*` consolidés en un seul constructeur paramétré.
- **Testabilité** : T1 = GREEN golden vert sur l'état actuel (baseline) + count nextest >=
  baseline. Critère machine (D4) = fmt + clippy + nextest invariant. T2 = N-A.
- **Risque** : **med**. Le golden est le filet des splits N→S ; il doit être vert sur l'état
  ACTUEL avant tout déplacement.
- **frontier_closure** : N/A.

## Phases N→S — Split http.rs, 1 domaine = 1 commit = 1 phase (D3, PO=ambitieux)

> **Discipline commune (les 6 phases N→S)** : co-déplacer **handler + DTO + tests** (le test
> module fait ~7915 l, region 4546-12460 — JAMAIS orphelin) ; route inchangée dans
> `build_router` pointant `crate::<domaine>::<handler>` ; **golden Phase M vert post-split** ;
> **0 route path modifiée, 0 bump wire**. Cible cumulée : région production http.rs
> < ~2500 l après S ; long tail (feed/search/preview/canary/kudos/apps) DÉFÉRÉE.
> Testabilité de chaque : T1 = GREEN golden vert + nextest invariant + fmt/clippy verts (D4) ;
> T2 = N-A. frontier_closure : N/A (routes inchangées ; si signature DTO lue par `web/src`
> touchée ⇒ index Phase T). Risque : **med** chacune (conflit rebase attendu avec l'arc front
> parqué [`provider_router.rs`] + axe sharding ; mitigation incrémentale D3 + golden + count).
> Covers (chacune) : REFACTO-HTTP-SPLIT.

> Chaque phase N→S applique la **discipline commune** ci-dessus (T1 = GREEN golden Phase M +
> nextest invariant + fmt/clippy [D4] ; T2 = N-A ; risque **med** ; frontier_closure N/A sauf
> DTO lu par `web/src` ⇒ index Phase T). Seuls le domaine et le fichier cible changent.

## Phase N — Split http.rs : domaine shard-session http → shard_session_http_api.rs

- **Goal** : extraire le domaine shard-session http (`http.rs:2154-2509`, **6 handlers**) vers
  `shard_session_http_api.rs` — handler + DTO + tests co-déplacés, route inchangée.
- **Livrables** : `shard_session_http_api.rs` (6 handlers + DTO + tests) ; `build_router`
  pointant `crate::shard_session_http_api::<handler>`.

## Phase O — Split http.rs : domaine seed → seed_api.rs

- **Goal** : extraire le domaine seed (`http.rs:2489-3263`) vers `seed_api.rs` — handler + DTO
  + tests co-déplacés, route inchangée.
- **Livrables** : `seed_api.rs` (handlers + DTO + tests) ; `build_router` pointant
  `crate::seed_api::<handler>`.

## Phase P — Split http.rs : domaine frost → frost_api.rs

- **Goal** : extraire le domaine frost (`http.rs:3559-3722`, **4 handlers**) vers `frost_api.rs`
  — handler + DTO + tests co-déplacés, route inchangée.
- **Livrables** : `frost_api.rs` (4 handlers + DTO + tests) ; `build_router` pointant
  `crate::frost_api::<handler>`.

## Phase Q — Split http.rs : domaine coordinator → coordinator_api.rs

- **Goal** : extraire le domaine coordinator (`http.rs:3722-4023` :
  submit_task/submit_result/get_kudos/verify_chain) vers `coordinator_api.rs` — handler + DTO
  + tests co-déplacés, route inchangée.
- **Livrables** : `coordinator_api.rs` (submit_task/submit_result/get_kudos/verify_chain + DTO
  + tests) ; `build_router` pointant `crate::coordinator_api::<handler>` (DTO lus par `web/src`
  ⇒ index Phase T si signature touchée).

## Phase R — Split http.rs : domaine curators → curators_api.rs

- **Goal** : extraire le domaine curators (`http.rs:884-1102`) vers `curators_api.rs` — handler
  + DTO + tests co-déplacés, route inchangée.
- **Livrables** : `curators_api.rs` (handlers + DTO + tests) ; `build_router` pointant
  `crate::curators_api::<handler>`.

## Phase S — Split http.rs : domaine publish → publish_api.rs

- **Goal** : extraire le domaine publish (`http.rs:1159-1727`) vers `publish_api.rs` — handler
  + DTO + tests co-déplacés, route inchangée. **Cible cumulée atteinte** : région production
  http.rs < ~2500 l après cette phase.
- **Livrables** : `publish_api.rs` (handlers + DTO + tests) ; `build_router` pointant
  `crate::publish_api::<handler>` ; vérif région production http.rs < ~2500 l.

## Clôture

## Phase T — CLÔTURE docs-contrat + amendement roadmap + gate push groupé + migration stores

- **Goal** : livrable de fermabilité (README §6.12) en UNE phase : indexer TOUTE frontière
  neuve du sprint (3 request-body shard-session Phase G) dans la couche GUIDE + llms.txt +
  WIRING_SPEC + SHARD_PROTOCOL_SPEC, en un passage. Trancher LOOPBACK §3 = tier-target
  représentatif verrouillé en front-matter (D7, pas exhaustif). Réconcilier CLAUDE.md CI claim
  à l'état réel post-repair. Amender roadmap v5 (insérer S82 DONE + slots décalés
  workflow-engine/Viewer/arc-front tracés non-perdus) + SPRINT_LOG row 82. **Vérifier la
  migration stores worker redb2→4 sur 3 nœuds au push (S81-G-1) + artefact
  `sprint82_t2_store_migration.json` (critic gap #2).** Note opérationnelle : redescente
  consents L4 PC+Mac post-quorum si applicable. **Déclencher `workflow_dispatch
  integration-nightly`** (ferme S81-J-2, ≥1 run réel lisible) et vérifier rust-ci 3-OS vert
  sur le tip ⇒ **gate push groupé (PO-4=C)**. Le push lui-même = action sortante, à confirmer
  à l'exécution.
- **Covers** : S82-DC-LOOPBACK-INVENTORY-EXHAUSTIVE, CLAUDE-CI-CLAIM-RECONCILE,
  LEDGER-ROADMAP-V5-AMEND, S82-TEST-META-ACCEPTANCE, S82-TEST-INTEGRATION-NIGHTLY-REAL,
  LEDGER-STORE-MIGRATION-G1.
- **Livrables** : GUIDE + llms.txt + WIRING_SPEC + SHARD_PROTOCOL_SPEC index des 3 request-body
  Phase G ; LOOPBACK §3 périmètre représentatif verrouillé (front-matter, D7) ; CLAUDE.md CI
  claim réconcilié ; roadmap v5 amendé (S82 DONE + slots décalés tracés) + SPRINT_LOG row 82 ;
  artefact `sprint82_t2_store_migration.json` (migration redb2→4 vérifiée 3 nœuds, D12) ; run
  réel `integration-nightly` (workflow_dispatch, junit) + rust-ci 3-OS vert sur le tip.
- **Testabilité** : T1 = N-A. Critère machine = les 3 gates docs (frontier + sharding +
  factory) exit 0 sur l'état final ; `gh run list --workflow integration-nightly` ≥1 run
  complété junit lisible ; rust-ci 3-OS success sur le tip ; store-migration artefact PASS.
  T2 = agrégat store-migration + rappel artefacts A/B.
- **Risque** : **low**. Le push est une action sortante confirmée à l'exécution, gaté sur 3
  verts.
- **frontier_closure** : GUIDE + llms.txt + WIRING_SPEC + SHARD_PROTOCOL_SPEC index des 3
  request-body shard-session ; LOOPBACK §3 périmètre représentatif verrouillé ; 3 gates docs
  exit 0.

---

## Récap technique — cibles refacto (faits load-bearing vérifiés disque)

| Cible | Fait vérifié | Phase(s) |
|---|---|---|
| `http.rs` | **12460 l** ; région production visée < ~2500 l post-S | M→S |
| `runtime.rs` | **5096 l** ; `DaemonRuntime::start()` ~950 l monolithiques (l.276-1224) | L, A |
| Modules `*_api.rs` | **11 modules** de précédent existent — pattern d'extraction PROUVÉ | N→S |
| Test module http | ~**7915 l** (region 4546-12460) — co-déplacé sans orphelin | M→S |
| `build_test_router*` | 4+ dupliqués (`4645/4649/7534/8380`) → un constructeur paramétré | M |
| Driver boot-SEED | **ONE-SHOT** `http.rs:1819-1826` (intact) — re-drive-on-ingest en A | A, L |
| Promesses `task_response.rs` | :14 / :84-85 / :95 présents verbatim (until/when Sprint N) | F |
| Split http.rs cibles | N shard-session http `2154-2509` (6 handlers) ; O seed `2489-3263` ; P frost `3559-3722` (4 h) ; Q coordinator `3722-4023` ; R curators `884-1102` ; S publish `1159-1727` | N→S |

## Cluster rig-chaud

Les **Phases A** (fermeture boot-SEED — exit-condition (c) live PASS<30 s exigée par PO-1=B)
et **B** (benchmarks standards sharding — llama-bench + perplexity-parity + TTFT/TPOT/ITL)
partagent le **rig Mac+PC+VPS allumé** : elles sont **ordonnancées ensemble** pour n'engager
le matériel qu'une fois. Cibles SSH et procédure : memory `live_acceptance_setup` (cibles
`vps`/`mac`, `PROJECT_ID`, auth `x-sbfb-token`, script `b3_live_pc_vps.sh` — étendu ici par
le mode `BOOT_AFTER_SUBMIT` de la Phase A). **Rig indisponible ⇒ escalade PO explicite**
(A ne ferme pas silencieusement — jamais un 4e report sec ; B émet `BLOCK{rig}`, jamais
`RIG-ABSENT`, le rig étant engagé pour A).

## Gate de testabilité (rappel — cf. kickoff)

Stratégie T1/T2 **PAR TYPE DE PHASE**, avec `## Acceptance` (vocab fermé) écrit à CHAQUE
phase (enforced lightcheck + audit Track J) sur le squelette voie-A `sprint81_verification.md` :
1. **DOCS-CONTRAT PURE (E,F,G,H,I,J,T)** : T1 = N-A-no-frontend-change (sauf `web/src/api`
   touché ⇒ `npm run test:e2e` GREEN enregistré, leçon S81-J-1) ; critère machine = 3 gates
   docs exit 0 + frontière neuve indexée. T2 = N-A.
2. **REFACTO PURE (L,M,N,O,P,Q,R,S)** : T1 = GREEN non-régression = suite verte + count
   nextest >= baseline + clippy -D warnings + fmt --check + 0 route path + 0 bump wire (D4) ;
   golden Phase M pour les splits http.rs. T2 = N-A.
3. **CI (C,D)** : critère machine = gh run success sur le tip OU décâblage honnête + E2E
   Woodpecker verts ; ≥1 run integration-nightly réel (workflow_dispatch, junit) vérifié Phase T.
4. **BOOT-SEED (A)** — SEULE phase cross-machine, **GATE PLEIN** : 2 tests hermétiques 2-nœuds
   BLOQUANT-vert (contrôle red revert-proof) PRÉREQUIS DUR ; **T2 = re-jeu live (c) PASS<30 s
   OBLIGATOIRE (PO-1=B), rig indispo ⇒ escalade PO.**
5. **BENCHMARK (B)** : artefact JSON métriques versionné + rig-gated ; amendement canon T3
   ratifié.

GLOBAL : ratifier README §4 les tokens palier-level T2 **ACTED/MIXED/NOT-RUN** (Phase J) ;
push groupé (Phase T) gaté sur **3 verts** (Woodpecker + rust-ci 3-OS + run nightly réel).
Garde standing E2E web : tout nouveau spec semeur ⇒ projet chromium-authoring / cleanup (ne
pas re-casser browse-search empty-state). Baseline d'invariance (D4) : nextest Win **2095** /
Docker **2099** (0 baisse) + Vitest web **412** + operator **201**.

## Scope cuts (rappel — cf. kickoff §Out)

- **Sharding feature/hardening** : R-J-6 (RunProof per-worker + binding N0-N3), F2 (KV-cache
  cross-step), SI-12 (TOCTOU load↔hash), SHARD-TRUST-RECALIB (N3-reveal/SI-5/SI-7/SI-11),
  métriques-honnêteté cluster batché benchmarks. Seul **SCHEMAS-SHARD-REQ (doc-contrat)**
  foldé Phase G (D6).
- **Fixes robustesse sharding bon-marché** (J1b-3 cap participants, D3-2 charset piece, D4-2
  préfixe 16-hex, J-D5-1 assertion conn_type) : dette d'AUDIT hors-thème, foldés seulement
  comme hygiène de slack étiquetée hors-thème.
- **Reprise arc front parqué** `wip/factory-front-arc-post-s82` (87 fichiers, review+Codex
  groupés, rebase conflit `provider_router.rs`) — POST-S82 (PO-6).
- **app-authoring S79 in-vivo `Not evidenced`** — carry P1 STANDING OUVERT (distinct du carry
  sharding CLOSED par `b3_p2_quorum`). NE PAS déclarer éteint (audit_plan §3 le conflate).
- **workflow-engine + Viewer fondation** — DÉCALÉS (C9/PO-9) ; ratifiés décalés + staging
  SUPERSEDED (Phase E), non codés.
- **Split fichiers secondaires >2000 l** (shard_session.rs, iroh_runtime.rs, engine/runtime.rs,
  coordinator db.rs, public_feed.rs) — différé.
- **Long tail split http.rs** (feed/search/preview/canary/kudos/apps) — après N→S.
- **Tickets hors-thème (D10)** statués au ledger, non codés (T20-wire, T21, T23 Docker@sha256,
  T25 FIPS, T26 Argon2id, T27 rpassword, nginx-DRY, firewall). **EXCEPTION : hickory IN**
  (PO-7=A, Phase K).
- **Veilles supply-chain standing** trigger-driven — re-datées seulement (sauf hickory PO-7=A).
- **Collapse-sites clippy MSRV** — DÉJÀ résolus S81 Phase B (vérifier clippy vert seul).
- **Magic-number sweep comme phase dédiée** — scope-cut nommé (aucun résiduel concret S81).
- **Tagging exhaustif ~22 familles `DOMAIN_*_V1` + LOOPBACK §3 exhaustif** — remplacés par
  accept-and-close incrémental (D8) + représentatif verrouillé (D7).
- **Topologie A-vs-B** — re-décision calendaire hors-S82 (PO-5, avant 25/08 ; croise gate n0
  15/09, EOL relais 30/09). 0 travail S82.
