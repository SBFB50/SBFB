# Sprint 82 — Phase T — Préflight G8 (synthèse)

> Workflow `wf_818d5f99-6aa` — 8 agents (6 scans fan-out + 2 critics adversariaux),
> 988k tokens, 268 tool calls, exécuté 2026-07-17 sur tip `32abfab` (HEAD == memory Tip,
> 46 commits d'avance sur `origin/master` `c899d54`). Chaque fait ci-dessous porte une
> évidence disque re-vérifiée ; les 2 seules réfutations critic sont des corrections
> d'évidence (bornes de lignes REFERENCE.md off-by-2 ; typo de casse dans un grep),
> zéro réfutation de fond.

## Verdict final : **PLAN-ADAPT** — adaptations numérotées A1..A6

Le cœur de la Phase T (plan `:456-486`) est exécutable tel quel, mais les faits disque
imposent 6 adaptations. Aucun Day-0 touché : D7/D8/D12 respectés ; PO-4=C n'est pas
remis en cause dans sa substance (les 3 verts restent exigés), seule sa **séquence**
est matériellement impossible à la lettre — la lecture bi-temporelle (A4) sera
présentée au PO **au moment de la confirmation du push**, déjà obligatoire.

## 1. Contexte

Phase T = clôture S82 (24/24) : index frontière Phase G + LOOPBACK D7 + CLAUDE.md CI +
roadmap v5 + SPRINT_LOG + migration stores (D12) + nightly réel + rust-ci 3-OS ⇒ gate
push groupé PO-4=C. Type kickoff = DOCS-CONTRAT PURE, **mais** D12 + plan `:481`
priment sur le mapping générique : T2 ≠ N-A (agrégat store-migration). Working tree :
3 fichiers hors-phase PO (`verification_blueprint.md` modifié + 2 untracked
`workflow_*_2026-07-15.md`) — **EXCLUS du staging** (précédent s4_review:301-302,
jamais `git add -A`).

## 2. A1 — Scope ÉTENDU : 3 livrables de fermabilité canon OMIS par le plan

Le plan `:473-477` omet ce que README §3.3:533-552 rend OBLIGATOIRE dans le commit de
sortie (« Sans ces trois livrables, le sprint ne peut pas être fermé » :552), précédents
durs S80 `d8246bd` + S81 `8b3590c` (verification + audit_plan suivant + agrégat T2 +
roadmap + row log DANS le commit de clôture) :

1. **`sprint82_verification.md`** — 9 sections §2.3 + `## Acceptance` sur le squelette
   voie-A `sprint81_verification.md:258-295`. Vocab fermé (README:629-631, :682-711) :
   T1 ∈ {GREEN, RED, N-A-no-frontend-change} ; T2 top-level ∈ {PASS, BLOCK{diagnosis},
   RIG-ABSENT, N-A-no-cross-machine-feature} ; palier-level ∈ {ACTED{evidence}, MIXED,
   NOT-RUN} ; per_test ∈ {PASS, FAIL{cause}} ; T3 ∈ {PASS, BLOCK{rig},
   REGRESSION{metric}, N-A-no-benchmark}. DoD (b) : statuer CHAQUE carry routé du
   ledger Phase E (S81-G-1 → CLOSED si artefact PASS ; S81-J-2 PARTIAL → CLOSED si run
   junit réel). Consigner aussi l'écart lettre-D8 (cf. §8 risque 5).
2. **`sprint83_audit_plan.md`** — 6 sections §2.4 + Track J + routage §4.4 des P2/P3
   des 24 phase-reviews. Matière concrète à ne pas perdre (Codex S82) : 0 golden
   feed/search/provenance/preview/proof-card/browse/nodes (classe S2/S3/S4) ; famille
   `dispatch_loop` candidate test-group nextest borné (flake parallélisme) ;
   `browse_pull` sans test direct ; réparation classe watcher macos-14 si arbitrée
   non-bloquante (cf. A4).
3. **Agrégat `sprint82_t2_acceptance.json`** (patron README:682-685, précédent
   `sprint81_t2_acceptance.json` +89 l dans `8b3590c`) = substance du cover
   S82-TEST-META-ACCEPTANCE. **Sanction des paliers AU PRÉSENT PREFLIGHT**
   (README:693-694) :
   - `bootseed` = **ACTED**{evidence: `sprint82_t2_bootseed.json` status PASS, Phase A}
   - `benchmarks` = **ACTED**{evidence: `sprint82_t2_benchmarks.json` status
     BLOCK{rig: cold} consigné honnêtement, PO-2 rig-gated, jamais RIG-ABSENT}
   - `store-migration` = **à produire en T** (live-ops, D12) — entre dans l'agrégat
     avec son statut réel au moment du run.

**Archivage active/ → archive/v2.1 = HORS Phase T** : précédent `6fc263b` (chore de
kickoff S83). Ne pas archiver dans le commit T.

## 3. A2 — Index frontière Phase G : 4 surfaces → 2 surfaces réelles + anchors

État disque : **SPEC + WIRING_SPEC DÉJÀ COMPLETS** (livrés Phase G `d2705b7`,
gate-ancrés, corroboré ledger_reconciliation:132 « SCHEMAS-SHARD-REQ ROUTED Phase G
(schémas) + Phase T (indexation) ») :
- `docs/protocol/SHARD_PROTOCOL_SPEC.md` :103-104 (§3 rows), :112-118 (mount
  non-schématisé : enveloppe signée ComputeGroupEntry + iroh::EndpointAddr), :313-350
  (§6.1 trois tables + « The PATH is authoritative ») — **0 édition**.
- `docs/sharding/WIRING_SPEC.md` :179-186 (bloc « Request bodies (S82 G) », 3
  source-refs résolvables, REQUIRED_ANCHORS check-sharding-docs.sh:214-217) — **0 édition**.

Travail T réel (grep : 0 hit des 3 noms dans tout llms.txt/GUIDE) :
1. **`docs/sharding/llms.txt`** — 2 insertions : (a) entrée `:37`
   (shard_session_http_api.rs) : nommer les 3 request-bodies + pointeur SPEC §6.1 ;
   (b) entrée `:60` (Generated schemas) : nommer les 2 snapshots
   `shard_group_mint_request.schema.json` / `shard_generate_request.schema.json` +
   doctrine mount-non-schématisé. Backticks rank-1 auto-vérifiés par le
   source-ref-check (symboles prouvés résolvables : `schemas/shard.rs:239/:260`,
   `shard_session.rs:228`).
2. **GUIDE `docs/sharding/`** — REFERENCE.md : `## Types` à **:45**, header table
   **:51-52**, 8 rows **:53-60** (bornes CORRIGÉES par critic — pas :49-58). Ajouter
   rows `ShardGroupMintRequest` + `ShardGenerateRequest` (champs depuis
   `schemas/shard.rs:239/:260`, signed?=no) + row/note `MountSessionRequest`
   « NOT schematised → SPEC §6.1 » + **backfill même-classe**
   `ShardSessionResultView`/`ShardSessionResultResponse` (S81-I, snapshots présents
   disque, absents de REFERENCE — précédent exact du fix SPEC Phase G). HOW_TO_WIRE.md
   §START (:53-55/:100) : 1 ligne FR de renvoi « corps de requête exacts : SPEC §6.1 »
   (éviter EN_WORDS ; pas de duplication de table).
3. **Extension anchors `scripts/check-sharding-docs.sh`** : 3 anchors sur
   docs/sharding/llms.txt + 3 sur REFERENCE.md — SANS cela le critère machine « 3
   gates exit 0 » est NON-DISCRIMINANT (run réel pré-T : les 3 gates passent déjà
   EXIT=0) alors que frontier_closure `:484-486` le cite comme preuve de fermabilité
   §6.12. Shellcheck local avant commit (workflow shellcheck.yml existe sur origin).
4. **Ne PAS toucher** llms.txt racine (index borné 2 sous-systèmes, scope banner :8-13)
   ni docs/factory/* — consigner la décision au commit body.
5. **Contraintes des gates** : conserver les tokens littéraux honesty-gate (« S82 »,
   « S82-pending tuning », « admission ≠ confidentialité ») ; toute re-route des claims
   périmés « routed S82 » (per-worker proofs / dispute arbitration — llms:12-13,
   WIRING_SPEC:33-35/:133-136, SPEC:10-12/:55-56/:75, REFERENCE:98) se fait en
   formulation passé/statut « routed to the deferred rig-chaud slot (roadmap v5) » en
   GARDANT un token S82 par fichier gate-marqué, sinon amender le gate dans le même
   geste. Éviter les formes PROMISE_RE (« until S83 », « S83 will », « when Sprint 83
   lands ») partout — docs hors scope gate volet 1 mais doctrine STALE-PHASE-K en review.

## 4. A3 — LOOPBACK D7 (sous-verdict EXECUTE) : geste additif front-matter

Doc à jour (passe fidélité Phase I `747470b`, 0 pointeur cassé par N..S4, trigger
incrémental déjà présent :7 et déjà FIRED 2× S81 G/K) :
1. Nouveau champ front-matter dédié (ex. `inventory_policy:`) déclarant : §3 =
   tier-target REPRÉSENTATIF verrouillé, jamais exhaustif (27 route-paths réels
   distincts représentés / 89 routes réelles, count vérifiable `grep -c '.route(' http.rs`) ;
   critère d'inclusion = tout endpoint tier-cible ≠ T0 + un représentant par famille de
   justification T0 ; garde-fou = triggers existants :7 (daemon) + :294-295 (Operator).
   NE PAS gonfler la ligne :3 (last_validated déjà ~1900 chars) ; bump last_validated
   court « D7 : périmètre représentatif verrouillé ».
2. Table §3 (29 rows) **INTACTE** — geste additif + éventuelle phrase chapeau sous §3.
3. Micro-fix opportuniste `:163` : « §3 ligne 55 » → ancre stable
   « §3 row `POST /api/v1/tasks/submit` » (row réelle :71 ; classe pointeur-qui-pourrit).
4. Clôture track : S82-DC-LOOPBACK-INVENTORY-EXHAUSTIVE = **CLOSED-BY-POLICY** (D7,
   accept-and-close, 27/89 assumé) consigné body + verification.md — sinon l'audit S83
   Track K/H re-lèvera le delta 62 paths comme drift. NB : les 6 IDs « Covers » du plan
   `:470-472` sont des labels plan-locaux (0 match dans audit_plan — critic l'a prouvé) ;
   leurs définitions opposables : ledger:106/:110, findings:451-452, D7/D12 kickoff,
   phase_c_review:89, kickoff:299-322.

## 5. A4 — Gate push PO-4=C : lecture BI-TEMPORELLE à faire ratifier + risque macos-14

Faits durs : (a) `integration-nightly.yml` ABSENT d'origin/master (`git ls-tree` = 11
workflows ; `gh` → HTTP 404) ⇒ indispatchable pré-push ; (b) Woodpecker aveugle —
codeberg/master gelé `f4b4600` (2026-06-25), Mirror rouge « CODEBERG_TOKEN secret
missing », aucun fix dans les 46 commits ; (c) tous les triggers push GHA filtrent
branches [master,main] ⇒ un push de branche staging n'auto-déclenche RIEN ;
`rust-ci.yml` a `workflow_dispatch` (aussi sur origin :34, workflow enregistré actif).
**2 des 3 verts sont donc post-push par construction.** Le critère machine du plan
`:478-480` omet d'ailleurs Woodpecker que PO-4=C (kickoff:41) exige.

**Séquence opérationnelle** (chaque action sortante ⇒ confirmation PO explicite) :
1. [SORTANT] `git push origin 32abfab+:refs/heads/ci/s82-tip` (tip de Phase T commité
   — même commit ⇒ « sur le tip » satisfait) ;
2. `gh workflow run rust-ci.yml --ref ci/s82-tip` + `gh run watch`. Attendu : windows
   VERT (2 précédents), ubuntu VERT post-GTK Phase C, **macos-14 = RISQUE MAJEUR
   NON TRACKÉ** : 20 tests fs-watcher/hot-reload en TRY 2 FAIL sur les 2 derniers runs
   master (28661119376 head=c899d54 ; 28592686238 head=d8246bd) — classe absente du
   plan, des 46 commits et des overrides nextest. Si rouge après diagnostic :
   **arbitrage PO** (fix override/env vs leg macos non-bloquant + ticket S83) — le
   critère « 3-OS success » est à lui.
3. [Optionnel] tenter `gh workflow run integration-nightly.yml --ref ci/s82-tip`
   (enregistrement branch-only non prouvable sans pousser ; 404 probable — fallback
   garanti post-push).
4. [SORTANT — LE push groupé PO-4] `git push origin master`.
5. [SORTANT] Woodpecker : restaurer secret CODEBERG_TOKEN (action PO settings GitHub)
   OU `git push codeberg master` (remote local configuré) ; vérifier pipeline
   push-master sur ci.sbfb.world.
6. `gh workflow run integration-nightly.yml --ref master` puis
   `gh run list --workflow integration-nightly.yml` (≥1 completed) +
   `gh run download <id> -n junit-integration` (artefact défini :73-80) = preuve
   « run réel lisible » S81-J-2. Le cron 03:00 UTC ne compte pas : le dispatch OUI.
7. Nettoyage `git push origin :ci/s82-tip`.

Bruit rouge ATTENDU au push master (hors gate, à ne pas confondre avec un échec) :
`deploy.yml` failure 0s à chaque push (trigger workflow_dispatch seul :8-9,
startup-failure) ; `Build worker binaries` rouge (dernier run timeout 24h) ;
`Mirror to Codeberg` rouge tant que le secret manque ; `canary-monthly` rouge
2026-07-13 (flaggé Phase T par review C:71-89 — consigner, tag-triggered hors gate) ;
`supply-chain` PLAUSIBLEMENT vert post-hickory Phase K `713f0fa` (non prouvé).
**actionlint** à rejouer au gate push (routé review D:60-62) si disponible.

**Réconciliation CLAUDE.md ÉLARGIE** (le claim est plus faux que prévu) : `:394-400`
« GHA CI ROUGE » ET « Woodpecker opérationnel » STALE (gel f4b4600 + GTK pré-C) ;
`:411-412` « toujours verts ... Woodpecker » STALE ; TOUS les workflows GHA master
rouges, pas seulement « CI ». Même geste CLAUDE.md : mettre à jour « Etat actuel »
(S82 DONE 24 phases, http.rs 1513 l, deltas ±0) + compteur « ~2730 tests » +
post-commit memory (nexus_grid_pivot.md + MEMORY.md, règle feedback_memory_update).

## 6. A5 — Migration stores redb 2→4 : vérité PAR CLASSE de store (D12)

Finding opposable (seule prose) : findings:451-452 « exit = migration vérifiée 3 nœuds
(sibling backup OU store recréé + worker fonctionnel 1.0.1) ». Mécanique upstream :
iroh-docs 0.101 auto-migre docs.redb à l'ouverture, sibling
`.backup-redb-v2-tuples` (docs.rs:51-64) ; blobs.db sans migration.

État live re-vérifié (ls réels session) :
- **PC standalone : DÉJÀ MIGRÉ** — `%APPDATA%/FlowUP/nexus-grid/data/docs.redb.backup-redb-v2-tuples`
  (618496 o, Jul 11 19:40 = jour du quorum) + docs.redb v4 (Jul 13). Preuve acquise.
- **Mac standalone** : ssh `mac`, `ls ~/Library/Application Support/dev.FlowUP.nexus-grid/data/docs.redb*`.
- **VPS** : local-worker only sous `/var/lib/nexus-grid/shell-daemon/local-worker/data/`
  — store ABSENT ou recréé frais v4 sont les 2 états attendus : `provision()` fait
  `remove_dir_all` + reprovision (`local_worker.rs:284-288`) ⇒ **jamais de sibling par
  construction**. La claim `t2_h:48` « will auto-migrate on first spawn » est RÉFUTÉE
  par le code — corriger dans les notes de l'artefact. Local-worker PC (docs.redb
  708608 o Jun 22, redb2 sans sibling, jamais rouvert) : même classe recreate-on-spawn.
- Garde recreate anti crash-window = DAEMON seul (`runtime.rs:2908-2945` ;
  0 hit MIGRATION_BACKUP_SUFFIX dans nexus-worker*/) — résiduel worker « recreate
  silencieux » à consigner en residual_risk, PAS à coder (hors périmètre S82).

Artefact `sprint82_t2_store_migration.json` : modèle `sprint81_t2_f_store_migration.json`
(suite/kind/iroh_lock/date/paliers{verdict,criterion,observed}/hermetic_anchors/
residual_risk), vocab PASS/BLOCK{diagnosis}, **un palier par nœud avec champ `mode`** :
PC-standalone=`migrated-sibling-observed` ; Mac-standalone=à observer ;
VPS-local-worker=`recreated-fresh` OU `absent-nothing-to-migrate` (interprétation
consignée honnêtement — l'exit S81-G-1 ne nomme pas le cas absent). Clarifier
« 1.0.1 » = iroh (lock :3826), le champ worker_version reste « 1.0.0 » (version crate).

Séquencement live-ops : (i) **instruire l'anomalie `state.json`** (réécrit AUJOURD'HUI
12:19Z, données d'allure fixture « rsync xnode » + bloc GPU NVML réel — pollution
test/process à identifier AVANT de citer quoi que ce soit de live) ; (ii) captures de
preuve par nœud (ls sibling/absence + 0 orphelin `docs.db.migrate*`) ; (iii) hygiène
STORE_MIGRATION_OPS règle 4 : suppression sibling PC (et Mac si présent) APRÈS preuve
consignée (le sibling porte l'ancien NamespaceSecret) ; (iv) **redescente consents L4
APPLICABLE** (PC vérifié `~/.sbfb/consent.json` level=4 du 11/07) : POST
`/api/v1/consent/set {"level":1}` (ou 3) PC + Mac, écho vérifié GET /api/v1/consent,
hot-reload ConsentWatcher sans restart — note opérationnelle (kickoff [T] :
pas un livrable code), consignée artefact/body.

## 7. A6 — Roadmap v5 + SPRINT_LOG + petits items routés in-sprint

1. **roadmap v5** — 2 gestes : (a) nouveau bloc daté « LIVRAISON 2026-07-XX (S82
   Phase T) — S82 DONE » après :94, traçant les 3 slots décalés non-perdus :
   workflow-engine (re-valider les faits du blueprint SUPERSEDED à l'activation),
   Viewer fondation, **reprise arc front `wip/factory-front-arc-post-s82`** (review +
   Codex groupés DUS — AUCUNE ancre existante dans v5, à créer ex nihilo, source
   kickoff:385-386/:396-398) ; (b) note superseding INLINE sur le claim stale :87-90
   (« pas encore tranché » — C9/PO-9 ont tranché ; précédent :197-202).
2. **SPRINT_LOG** : row 82 insérée AU-DESSUS de :19 (newest-first), 5 colonnes modèle
   row 81 ; col3 « Phase T (ce commit) » ; **pas de row 78 à « corriger »** (S78 absorbé).
3. **RIEN à ratifier** : ACTED/MIXED/NOT-RUN déjà au canon README:682-711 (Phase J
   `57e19ad`) ; audit_plan §6 déjà corrigé Phase E `f727f8c`. Retirés de la checklist.
4. **Petits items routés Phase T par les reviews S82** (critic-complétés) :
   §P75 golden-characterization dans PATTERNS.md rust (M-6 P3 le proposait
   « §P74 » — numéro DÉJÀ PRIS par shell/PATTERNS Phase A, espace commun ;
   m_review:246/:298/:316) ;
   `EXTERNAL_AUDIT_SCOPE.md:35` chemin CORE mort + frost-ed25519 « 2.1 »→3.0.0
   (p_review:92 F2) ; micro-fix `TOOLING.md:291` pointeur « http.rs:483-494 » pourri
   (aujourd'hui = routes storage/feed ; viser le module _api réel — piège S2-F2 vivant
   hors gates) ; SPRINT_LOG:23 « http.rs:8531 » = narration historique row 76,
   TOLÉRÉE (passé immuable).
5. **T1 sprint-level ≠ N-A sec** (critic RÉFUTE le plan :478) : `web/src/api/daemon.ts`
   touché par R `f7d42bc` (1/1 comment) + S3 `0a32ffa` (1/1 comment), e2e spec par S4 —
   la règle kickoff:303-304 (leçon S81-J-1) n'a pas d'exemption comment-only ⇒
   enregistrer `npm run test:e2e` GREEN (44/2skip) + consigner le rationale comment-only
   dans verification.md.

## 8. Risques résiduels (ordonnés)

1. **macos-14 watcher-classe rouge** — peut bloquer le gate 3-verts ; traitement
   séquencé A4.2, arbitrage PO si persistant. (Le détail « 20 TRY 2 FAIL » vient du
   log du run c899d54 ; à re-confirmer sur le run du tip.)
2. **Bloc pré-push canonique** : `auth::tests::run_dir_paths_resolve_under_sbfb_home`
   flake par race env-var SBFB_HOME sous `cargo test --workspace` shared-process Docker
   (solo PASS ; i_review:128-138) — jouer le bloc via **nextest** (déjà le canon des
   suites) et garder le doctest-only pour `cargo test --doc` ; consigner si le flake mord.
3. **Woodpecker** : même après push codeberg, l'état pipeline n'est pas interrogeable
   depuis la session (badge vide) — la preuve du vert se prend sur ci.sbfb.world ;
   si l'instance n'observe plus le repo, escalade PO (action settings hors repo).
4. **Enregistrement GH d'un workflow branch-only** (nightly sur staging) : non
   prouvable sans pousser — fallback garanti = dispatch post-push --ref master.
5. **Écart lettre-D8** : kickoff:282-285 exige un tag `// FRONTIER:` (ou
   FRONTIER-NO-SCHEMA motivé) pour les 3 request-bodies — sur disque : 0 tag littéral,
   pattern snapshot+prose choisi en `d2705b7` (un tag littéral ferait échouer le gate).
   Adaptation SAINE mais jamais ratifiée : consigner « D8 adapté Phase G (pattern
   ShardSessionResultView) » dans verification.md, sinon Track K standing le relèvera.
6. **Anomalie state.json** (§6.i) : à instruire avant tout usage de preuve live.

## 9. Pièges ACTIVÉS (standing re-confirmés pour T)

Bash background tués ~2 min → vérifications lourdes AVANT-PLAN séquentiel timeout 600s
(l'overflow avant-plan survit) + docker orphelins à kill avant le run propre ; Docker
sbfb-ci mount `/workspace` OBLIGATOIRE + MSYS_NO_PATHCONV=1 ; `set -o pipefail` ;
codex `--sandbox read-only` (elevated cassé, réparation à statuer PO) ; chaîne `&&`
web avale post-FAIL ; jamais `cd web &&` chaîné ; commit body > 30 l → `git commit -F` ;
review.md reste PASS-PENDING jusqu'au verdict Codex réel (incident S2) ; 9 headers
body canoniques + Check 10 WARN attendu (verification.md stagée → tokens T1/T2
machine-lisibles inclus).

## 10. Ordre d'exécution retenu

1. Bloc ÉDITIONS repo (A2 index+anchors, A3 LOOPBACK, A6 roadmap/SPRINT_LOG/petits
   items, CLAUDE.md élargi A4) → 3 gates docs + shellcheck local.
2. Bloc LIVE-OPS (A5 : anomalie state.json → preuves 3 nœuds → hygiène rule 4 →
   consents L4) → `sprint82_t2_store_migration.json`.
3. Bloc FERMETURE (A1 : verification.md + audit_plan S83 + agrégat t2_acceptance).
4. Suites §7.4 complètes (3 blocs + release, nextest priorisé, dual-platform).
5. Review Workflow → PASS-PENDING → Codex → promote PASS → commit Phase T
   (staging SANS les 3 fichiers PO).
6. Gate push A4 (séquence 1→7) — **chaque action sortante confirmée PO** ; lecture
   bi-temporelle PO-4=C présentée à la confirmation du push.
