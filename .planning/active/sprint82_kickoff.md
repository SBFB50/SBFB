# Sprint 82 — Kickoff : Dette docs-contrat + Refactorisation (+ escalade boot-SEED, benchmarks, hickory)

> **STATUT : à ACTIVER** (Cas C, après Phase 0 = audit gate S81 **DÉJÀ JOUÉ** :
> verdict **FAIL** [0 P0, 4 P1, 16 P2, 14 P3] → **GATE LEVÉ voie A** par
> `fix(sprint81)` `ad53940` + chore `95ff46c`. **Ne PAS re-jouer.** Le carry P1
> `S81-G-ESC-1` boot-SEED est routé S82 par construction). Sprint de **DETTE
> DOCS-CONTRAT + REFACTORISATION** (C9 tranché PO 2026-07-11) : workflow-engine
> et Viewer fondation sont DÉCALÉS (futurs slots, PO-9 ratifie). **Décision-grade,
> pas rubber-stamp** : faits load-bearing re-vérifiés au disque le 2026-07-12
> (`http.rs` = 12460 l ; `runtime.rs` = 5096 l, `DaemonRuntime::start()` ~950 l
> l.276-1224 ; driver boot-SEED ONE-SHOT `http.rs:1819-1826` intact ; 3
> commentaires-promesse `task_response.rs` présents verbatim :14/:84-85/:95 ; 11
> modules `*_api.rs` de précédent — pattern d'extraction PROUVÉ).

**Type** : sprint **d'assainissement + refactorisation behavior-preserving**,
avec DEUX contraintes dures greffées (escalade boot-SEED OVERDUE 3/3, forcing
CI-avant-push) et une phase mesure (benchmarks). Il **n'ajoute aucune feature
produit** — il ferme la dette accumulée S79/S80/S81, répare un défaut LIVE, et
rend `http.rs` maintenable.
**Budget de phases** : Phase 0 (audit gate S81, JOUÉE) + **A→T (20 phases)** —
le split http.rs par domaine (D3, 1 domaine = 1 commit = 1 phase) porte le sprint
à 20 ; README §4 ne plafonne jamais le nombre de phases, dimensionné par le
travail. Rigueur per-phase **uniforme** : deep preflight (5 scans) + review +
Codex à **CHAQUE** phase.
**Tip** : master `95ff46c` (LOCAL, non poussé ; origin/master `c899d54`, **20
commits d'avance**). Baseline FIGÉE : nextest **Win 2095 / Docker 2099** ; Vitest
web **412** / operator **201** ; E2E operator **10** ; E2E web **44/2skip** (post
fix voie A). **Numéro/version archive** : S82, v2.1 (OPEN).

---

## Décisions PO tranchées à l'ouverture

> AUTORITÉ sur tout passage contraire. Récap ligne unique en fin de doc.

| # | Décision | Choix PO |
|---|---|---|
| PO-1 | Fermeture escalade boot-SEED OVERDUE 3/3 | **B — Réparer + preuve LIVE exigée** : fix complet ancre+worker + 2 tests hermétiques BLOQUANTS, ET clôture §6.2.1 conditionnée au re-jeu live (c) PASS<30s (engagement rig Mac+PC+VPS). Rig indisponible ⇒ escalade PO explicite, JAMAIS un 4e report sec silencieux. |
| PO-2 | Benchmarks standards (ex-Phase L) | **IN S82** — phase dédiée (llama-bench + perplexity-parity entier-vs-shardé + TTFT/TPOT/ITL versionnés) + amendement canon T3 (README §4) ratifié en S82. |
| PO-3 | Périmètre sharding-debt | **Doc-contrat SEUL** (SCHEMAS-SHARD-REQ, Phase G). Feature/hardening (R-J-6, F2, SI-12, N3-reveal) DIFFÉRÉ slot rig-chaud. |
| PO-4 | Condition push groupé | **C — Réparer CI (Phase C) puis pousser sur 3 verts** (Woodpecker ci-linux + rust-ci 3-OS + run `workflow_dispatch` integration-nightly lisible) en Phase T. |
| PO-5 | Topologie A-vs-B zéro-n0 | **Garder B** (déployée, éprouvée flip 3 nœuds). Re-décision calendaire HORS-S82, due avant 25/08 ; croise gate n0 15/09 (EOL relais 30/09). **0 travail S82.** |
| PO-6 | Reprise arc front parqué | **Après S82** (memory `rapid_front_add_session`). Scope-cut S82. |
| PO-7 | Bump hickory-resolver 0.24→0.26 | **A — Absorber en S82** (Phase K supply-chain bornée) : réécrire construction resolver + retirer 4 ignores `deny.toml` + clore 4 RUSTSEC vivants. |
| PO-8 | catalog_len=0 seeder (S81-G-3) | **Accept-and-document** (consigné Phase I sécurité-docs ; report répété depuis S75 → décision fermante due §6.2.1). |
| PO-9 | Ratifier décalage workflow-engine/Viewer | **Ratifier** : S82 = dette ; workflow-engine + Viewer = futurs slots ; supersede D6 actée ; staging `.planning/research/sprint82_workflow_engine/` marqué SUPERSEDED (Phase E) + `sprint82_audit_plan §6` corrigé. |

---

## Objectif produit

Solder la **dette accumulée** sur trois sprints (S79 app-authoring, S80 refonte
front, S81 iroh) et **fermer un défaut produit LIVE** avant tout nouveau chantier
de feature. Concrètement, S82 :

1. **Ferme l'escalade boot-SEED OVERDUE 3/3** (`S81-G-ESC-1`). Le driver de
   re-seed au boot est ONE-SHOT (`http.rs:1819-1826`) : une app `keep_online`
   dont l'annuaire arrive APRÈS le boot n'est jamais ré-pinnée, et les `task:`
   pendantes ne sont pas rattrapées au cold-boot worker. C'est un **bug LIVE
   observé** à l'acceptance S75. PO-1=B exige la réparation ET la preuve.
2. **Rend `http.rs` maintenable** : 12460 l dont ~7915 l de module test, et un
   `DaemonRuntime::start()` monolithique de ~950 l. Split par domaine
   (behavior-preserving, golden-gardé) et décomposition boot.
3. **Clôt 4 alertes sécurité** vivantes via le bump hickory-resolver 0.24→0.26
   (4 RUSTSEC, 4 ignores `deny.toml` retirés).
4. **Mesure les perfs du sharding** avec des outils standards (llama-bench,
   perplexity-parity, TTFT/TPOT/ITL versionnés) et grave le tier « benchmark »
   dans le canon (README §4, T3).
5. **Referme la dette docs-contrat** éclatée par domaine (canon « nommé, jamais
   bundlé ») : ledgers PATTERNS ré-audités, zombies purgés, frontières neuves
   indexées, honnêteté des claims restaurée.

Le DONE est **fermé par construction** : chaque phase a un critère
machine-checkable par son type (docs-gate exit 0 / count nextest invariant /
gh-run success / re-jeu live PASS / artefact JSON versionné).

---

## Pourquoi maintenant

1. **Audit S81 FAIL levé voie A.** Le cœur S81 est SOLIDE (b3_p2_quorum PASS =
   1er quorum de l'histoire) mais l'audit a compté **4 P1 + 16 P2 + 14 P3** — des
   items de gate/hygiène/consignation/escalade. Ils doivent être soldés avant de
   pousser et avant d'ouvrir workflow-engine.
2. **20 commits non poussés** (origin `c899d54`, LOCAL `95ff46c`). Le push est
   groupé et **gaté sur CI verte** (PO-4=C) : la CI est actuellement cassée (GHA
   rouge GTK depuis 2026-05, integration-nightly jamais joué). Réparer la CI
   (Phase C) est un pré-requis du gate testabilité « + CI chaque push ».
3. **Compteurs de reports durs** (§6.2.1) qu'on ne peut plus repousser :
   boot-SEED 3/3, S79-P2-1 (PROMISE_RE/task_response) 2, doc-lint S80-G-1 3,
   catalog_len=0 depuis S75. **Aucun report sec de plus** — chacun reçoit une
   décision fermante (réparé, accept-and-document, ou statué au ledger).
4. **Slot refacto = bon moment.** Le blocage S81 « iroh STRICTEMENT SEUL »
   (anti-bundle bisectabilité) est levé : le bump hickory (PO-7=A) et les splits
   structurels sont autorisés dans un sprint dédié.

---

## Arbitrages PO (tranchés)

- **PO-1 = B (Réparer + preuve LIVE).** Impact : Phase A devient la première
  phase substantielle (contrainte dure OVERDUE 3/3), GATE PLEIN, avec T2 re-jeu
  live obligatoire ; couplée à L (qui absorbe le hook re-drive) et clusterée
  rig-chaud avec B. Rig indispo ⇒ **escalade PO**, pas de clôture silencieuse.
- **PO-2 = benchmarks IN S82.** Impact : Phase B dédiée (rig-gated) + amendement
  canon T3 ratifié dans le sprint ; nouvelle primitive de mesure conçue au
  preflight.
- **PO-3 = doc-contrat seul.** Impact : seul `SCHEMAS-SHARD-REQ` (Phase G) entre ;
  toute la feature/hardening sharding est carry vers un slot rig-chaud.
- **PO-4 = C (réparer CI puis pousser sur 3 verts).** Impact : Phase C avant
  refacto/push ; gate push en Phase T sur Woodpecker + rust-ci 3-OS + run nightly
  réel.
- **PO-5 = garder B (zéro-n0).** Impact : **0 travail S82** ; re-décision
  calendaire hors-sprint, due avant 25/08 (croise gate n0 15/09, EOL 30/09).
- **PO-6 = après S82.** Impact : arc front parqué `wip/factory-front-arc-post-s82`
  (87 fichiers, review+Codex groupés, rebase conflit `provider_router.rs`) reste
  scope-cut.
- **PO-7 = A (absorber hickory).** Impact : Phase K supply-chain bornée ; churn
  API resolver 0.25 réécrit, 4 ignores retirés, 4 RUSTSEC clos.
- **PO-8 = accept-and-document.** Impact : catalog_len=0 seeder consigné Phase I
  (THREAT/PATTERNS), sort des carries — pas de fix code.
- **PO-9 = ratifier décalage.** Impact : supersede D6 actée ; staging
  workflow-engine SUPERSEDED (Phase E) ; audit_plan §6 corrigé ; workflow-engine
  + Viewer = futurs slots tracés non-perdus.
- **PO-10 = « S82 = une fin » (2026-07-15, amendement in-sprint post-Phase-N
  `2e87eef`).** Le split http.rs ne s'arrête pas au palier « production < 2500 l » :
  Phase N2 (harness de test partagé `test_support.rs`) insérée ; discipline O→S
  étendue aux tests router-driven ; long tail dé-déféré (Phases S2→S4) ; cible
  finale **http.rs ~≤2500 l TOTAL**, 0 carry « split différé ». Supersede la
  clause « long tail différée » de D3 (le reste de D3 — incrémental, 1 domaine =
  1 phase, golden AVANT — INCHANGÉ). Sprint : 20 → 24 phases. Détail : plan
  §Phases S2→S4 + memory `po_s82_http_split_est_une_fin.md`.

Les **4 réserves du critic** (verdict NEEDS-ADJUSTMENT) sont **levées** :
(1) granularité commit split → D3 (1 domaine = 1 commit = 1 phase, N→S) ;
(2) migration stores worker S81-G-1 → D12 (Phase T, artefact T2, plus « en marge ») ;
(3) hickory sans phase → PO-7=A ⇒ Phase K ; (4) fermeture §6.2.1 si (c) BLOCK{rig}
→ PO-1=B tranche : (c) live PASS EST exigée, escalade PO sinon.

---

## Scope

### In (Phase 0 jouée + A→T, 20 phases — **24 depuis PO-10** : +N2, +S2→S4)

- **Phase 0 — Audit gate S81 : JOUÉE.** Verdict FAIL levé voie A (`ad53940` +
  `95ff46c`). Baseline figée supra. Carries reroutés A-T.
- **A — Convergence cold-boot catch-up (escalade boot-SEED)** *(risk high)* :
  invariant unifié « broadcast gossip = HINT non fiable ; état durable synchronisé
  = VÉRITÉ ; tout consommateur cold-boot RECONCILIE ». 2 livrables conçus ensemble :
  (1) ANCRE = re-drive de `run_boot_seed_driver` à l'ingest annuaire (threader
  `boot_driver_state` + `keep_online_projects`, re-drive narrowed idempotent par
  set pinné, duress-gate hérité, borne 1/batch) ; (2) WORKER = catch-up des `task:`
  pendantes au cold-boot (`start_sync(peers)` après import + keepalive borné).
  0 wire, 0 dep, 0 bump.
- **B — Benchmarks standards sharding + amendement canon T3 (PO-2)** *(risk med,
  rig-dépendant)* : harness llama-bench + perplexity-parity entier-vs-shardé +
  TTFT/TPOT/ITL versionnés ; README §4 amendé (tier T3 + track audit + invariant
  kickoff). Clusteré rig-chaud avec A.
- **C — Restauration surfaces CI (GHA/GTK, Woodpecker E2E, integration-nightly)**
  *(risk med)* : trancher ARB-CI-1 (installer `libgtk-3-dev` dans les 2 jobs
  `ci.yml` OU décâbler honnêtement GHA + câbler les 2 E2E Playwright + coverage
  web + scan-en-strings dans `.woodpecker/ci-linux.yml`) ; calibrer
  `integration-nightly.yml` (slow-timeout ≥120s K-R-13, save-if master-only K-R-14).
- **D — Réparation relay-gated multi_daemon (baseline A3 4/10)** *(risk med)* :
  réparer OU requalifier chaque rouge `binary(multi_daemon)` ; distinguer 5
  test-rot d'infra du 1 signal produit réel (gossip discovery).
- **E — Réconciliation ledgers de dette + purge zombies (D9)** *(risk low)* :
  re-audit COMPLET des ~80 tickets T* PATTERNS ; purge zombies Python (T44-T51) ;
  double numérotation T15/T16 ; steps Python morts de `verify.sh` ; décompte stale
  « 8 P2/11 P3 S80 » → réel 4 P2/10 P3 ; staging workflow-engine SUPERSEDED +
  `audit_plan §6` corrigé (PO-9) ; tickets hors-thème statués et routés.
- **F — PROMISE_RE aveugle + ancres task_response.rs (S79-P2-1 / S80-G-2)**
  *(risk low)* : élargir `PROMISE_RE` (`check-frontier-contracts.sh:66`) pour la
  classe « until/when Sprint N activates/lands » ; réécrire les 4 commentaires
  `task_response.rs` (:14, :84-85, :95) vers le passé immuable ; test-mutation de
  non-vacuité.
- **G — Contrat request-bodies shard-session + registre FRONTIER (D8)** *(risk
  low)* : schématiser 3 request-bodies loopback (`ShardGroupMintRequest`,
  `MountSessionRequest`, `ShardGenerateRequest`). Option (a) `#[derive(JsonSchema)]`
  + snapshots drift-gatés ; repli (b) tables Request-body `SHARD_PROTOCOL_SPEC §6`
  + `FRONTIER-NO-SCHEMA`. Figer la métrique DOMAIN_*_V1 non-schématisées + acter
  accept-and-close incrémental + trancher S80-G-1 doc-lint.
- **H — Doc-dette patterns Track C + tripwire suffixe backup (S81-C-1/C-2/C-3)**
  *(risk low)* : re-ancrer T20 relay-cert-pinning (`PATTERNS.md:974` pointeur
  faux) ; résoudre C-1/C-2 ; §P73 fidèle ; tripwire du magic-string suffixe backup
  redb-v2-tuples dupliqué 2 crates (F-D5-01).
- **I — Doc-dette sécurité (Track H + drifts iroh/honnêteté + catalog_len=0)**
  *(risk low)* : H-1/H-2/H-3 (THREAT/LOOPBACK/HARDENING) ; `VALIDATED_BLUEPRINT.md:156-157`
  iroh 0.97→=1.0.1 + gossip 0.101 (G-D5-1) ; K-R-7 « session réelle »/« byte-identical »
  sur-large ; K-2 prose ; honnêteté claim cargo-audit ; **catalog_len=0 seeder
  (PO-8) accept-and-document consigné ici**.
- **J — Doc-dette process/meta + ratification vocabulaire T2 (Tracks F, I, J)**
  *(risk low)* : F-1..5 (hygiène fichiers-review) ; I-2/I-3 ; J-3/J-4/J-5
  (consignation T1/T2) ; RATIFIER au canon README §4 le vocabulaire palier-level
  T2 étendu ACTED/MIXED/NOT-RUN (S81-J-3).
- **K — Bump hickory-resolver 0.24→0.26 (PO-7=A, supply-chain bornée)** *(risk
  med)* : réécrire construction resolver (churn API 0.25 dans `dns_fallback.rs`),
  retirer 4 ignores `deny.toml`, clore 4 RUSTSEC vivants.
- **L — Refacto : décomposition `DaemonRuntime::start()` (~950 l)** *(risk med)* :
  éclater `runtime.rs:276-1224` en sous-fonctions boot nommées <~150 l ;
  regrouper helpers annonce/outbox. Refactor pur. **NOTE couplage : L absorbe le
  hook re-drive-on-ingest ajouté Phase A — ne pas l'écraser (gap A↔L).**
- **M — Refacto : golden de caractérisation HTTP + dédup harness** *(risk med)* :
  fondation du split — golden verrouillant l'identité pre/post des réponses HTTP
  AVANT tout déplacement ; consolider les 4+ `build_test_router*`
  (`4645/4649/7534/8380`) en un constructeur paramétré.
- **N→S — Split http.rs, 1 domaine = 1 commit = 1 phase (D3)** *(risk med
  chacune)* : co-déplacer handler + DTO + tests (module ~7915 l, jamais orphelin) ;
  route inchangée dans `build_router` ; golden Phase M vert post-split ; cible
  http.rs prod < ~2500 l ; long tail (feed/search/preview/canary/kudos/apps)
  DÉFÉRÉE. **[PO-10 : défèrement SUPERSEDÉ — N2 (harness partagé) + S2→S4
  (long tail) ajoutées, cible ~≤2500 l TOTAL ; cf. plan amendé.]**
  - **N** — shard-session (`2154-2509`, 6 handlers) → `shard_session_http_api.rs`.
  - **O** — seed (`2489-3263`) → `seed_api.rs`.
  - **P** — frost (`3559-3722`, 4 handlers) → `frost_api.rs`.
  - **Q** — coordinator (`3722-4023` : submit_task/submit_result/get_kudos/verify_chain) → `coordinator_api.rs`.
  - **R** — curators (`884-1102`) → `curators_api.rs`.
  - **S** — publish (`1159-1727`) → `publish_api.rs`.
- **T — CLÔTURE docs-contrat + amendement roadmap + gate push + migration stores**
  *(risk low)* : indexer TOUTE frontière neuve (3 request-body Phase G) dans GUIDE
  + llms.txt + WIRING_SPEC + SHARD_PROTOCOL_SPEC ; trancher LOOPBACK §3 tier-target
  représentatif (D7) ; réconcilier CLAUDE.md CI claim ; amender roadmap v5 (S82
  DONE + slots décalés tracés) + SPRINT_LOG row 82 ; **vérifier migration stores
  worker redb2→4 sur 3 nœuds + artefact `sprint82_t2_store_migration.json` (D12)** ;
  déclencher `workflow_dispatch integration-nightly` (ferme S81-J-2) + vérifier
  rust-ci 3-OS ⇒ gate push (PO-4=C). Push = action sortante à confirmer.

### Out (scope cuts)

- **Sharding feature/hardening** : R-J-6 (RunProof per-worker + binding N0-N3), F2
  (KV-cache cross-step), SI-12 (TOCTOU load↔hash), SHARD-TRUST-RECALIB
  (N3-reveal/SI-5/SI-7/SI-11), métriques-honnêteté cluster → slot rig-chaud. Seul
  `SCHEMAS-SHARD-REQ` foldé (D6).
- **Fixes robustesse sharding bon-marché** (J1b-3 cap participants, D3-2 charset
  piece, D4-2 préfixe 16-hex, J-D5-1 assertion conn_type) : dette d'AUDIT
  hors-thème, foldés seulement comme hygiène de slack, étiquetés hors-thème.
- **Reprise arc front parqué** `wip/factory-front-arc-post-s82` — POST-S82 (PO-6).
- **app-authoring S79 in-vivo Not evidenced** — carry P1 STANDING OUVERT (distinct
  du carry sharding CLOSED par b3_p2_quorum). NE PAS déclarer éteint.
- **workflow-engine + Viewer fondation** — DÉCALÉS (C9/PO-9) ; ratifiés décalés +
  staging SUPERSEDED (Phase E), non codés.
- **Split fichiers secondaires >2000 l** (shard_session.rs, iroh_runtime.rs,
  engine/runtime.rs, coordinator db.rs, public_feed.rs) — différé.
- **Long tail split http.rs** (feed/search/preview/canary/kudos/apps) — après N→S.
- **Tickets hors-thème (D10)** statués au ledger, non codés.
- **Veilles supply-chain standing** trigger-driven — re-datées seulement (sauf
  hickory PO-7=A).
- **Collapse-sites clippy MSRV** — DÉJÀ résolus S81 Phase B (vérifier clippy vert).
- **Magic-number sweep** comme phase dédiée — scope-cut nommé (aucun résiduel
  concret S81).
- **Tagging exhaustif ~22 familles DOMAIN_*_V1 + LOOPBACK §3 exhaustif** —
  remplacés par accept-and-close incrémental (D8) + représentatif verrouillé (D7).
- **Topologie A-vs-B** — re-décision calendaire hors-S82 (PO-5, avant 25/08).

---

## Day-0 — décisions gelées (NE PAS re-débattre)

- **D1 — Ordre** : boot-SEED Phase A (première substantielle, contrainte dure
  OVERDUE 3/3) + benchmarks Phase B clusterés rig-chaud ; CI Phase C avant refacto
  et push. Phase A est **CI-INDÉPENDANTE** (rust-ci 3-OS vert, 0 GTK) — c'est ce
  qui autorise boot-SEED-first malgré la CI cassée.
- **D2 — Design unifié boot-SEED** : broadcast=HINT, état durable synchronisé=VÉRITÉ ;
  2 livrables ancre+worker conçus ensemble ; clôture PO-1=B = (c) live PASS exigée.
- **D3 — Split http.rs** incrémental borné, 1 domaine = 1 phase (N→S, 6 domaines) ;
  golden Phase M AVANT ; long tail différée. Cible http.rs prod < ~2500 l.
  **[Clauses « long tail différée » + « cible prod » SUPERSEDÉES par PO-10
  (2026-07-15) : N2 + S2→S4 ajoutées, cible http.rs ~≤2500 l TOTAL — le coeur de
  D3 (incrémental, golden-gardé, 1 domaine = 1 phase) reste gelé.]**
- **D4 — Gate machine d'invariance pour TOUT refacto** : fmt --check + clippy
  --all-targets -D warnings + nextest count >= baseline (Win 2095/Docker 2099, 0
  baisse) + web vitest 412 + operator 201 + 0 route path + 0 bump wire + golden
  pour splits structurels.
- **D5 — Benchmarks IN S82 (PO-2)** — Phase B dédiée + amendement canon T3 ratifié
  en S82.
- **D6 — Sharding-debt = doc-contrat SEUL** (SCHEMAS-SHARD-REQ Phase G) ;
  feature/hardening (R-J-6, F2, SI-12, N3-reveal, métriques cluster) DIFFÉRÉ slot
  rig-chaud.
- **D7 — LOOPBACK §3 = tier-target représentatif verrouillé** (front-matter), pas
  exhaustif ; trigger nouvel endpoint = garde-fou incrémental.
- **D8 — Registre FRONTIER = accept-and-close incrémental** + métrique DOMAIN_*_V1
  figée ; tag `// FRONTIER:` (ou `FRONTIER-NO-SCHEMA` motivé) exigé pour toute
  frontière NEUVE (dont les 3 request-bodies Phase G) ; pas de tagging exhaustif
  des ~22 familles.
- **D9 — Ledgers PATTERNS = re-audit COMPLET** (pas purge ciblée).
- **D10 — Hors-thème NON codé** (statué au ledger seulement) : T20-wire, T21, T23
  Docker@sha256, T25 FIPS, T26 Argon2id, T27 rpassword, nginx-DRY, firewall ;
  veilles supply-chain standing (flip multiple-versions deny, yanked=deny,
  quinn-proto). **EXCEPTION : hickory est IN (PO-7=A, Phase K).**
- **D11 — hickory bump IN S82 (PO-7=A)** — Phase K supply-chain bornée.
- **D12 — Migration stores worker S81-G-1** = vérif live-ops rattachée Phase T avec
  artefact T2 committé (pas « en marge du push »).

---

## Gate de testabilité par-sprint (README §4)

Stratégie T1/T2 **PAR TYPE DE PHASE**, avec `## Acceptance` (vocab fermé) écrit à
CHAQUE phase (enforced lightcheck + audit Track J) sur le squelette voie-A
`sprint81_verification.md`.

1. **DOCS-CONTRAT PURE (E, F, G, H, I, J, T)** : T1 = N-A-no-frontend-change (sauf
   `web/src/api` touché ⇒ `npm run test:e2e` GREEN enregistré, leçon S81-J-1) ;
   critère machine = **3 gates docs exit 0** + frontière neuve indexée. T2 = N-A.
2. **REFACTO PURE (L, M, N, O, P, Q, R, S)** : T1 = GREEN non-régression = suite
   verte + count nextest >= baseline + clippy -D warnings + fmt --check + 0 route
   path + 0 bump wire (D4) ; **golden Phase M** pour les splits http.rs. T2 = N-A.
3. **CI (C, D)** : critère machine = **gh run success sur le tip** OU décâblage
   honnête + E2E Woodpecker verts ; ≥1 run integration-nightly réel
   (`workflow_dispatch`, junit) vérifié Phase T.
4. **BOOT-SEED (A)** — SEULE phase cross-machine, **GATE PLEIN** : 2 tests
   hermétiques 2-nœuds BLOQUANT-vert (contrôle red revert-proof) PRÉREQUIS DUR ;
   **T2 = re-jeu live (c) PASS<30s OBLIGATOIRE (PO-1=B), rig indispo ⇒ escalade PO.**
5. **BENCHMARK (B)** : artefact JSON métriques versionné + rig-gated (Mac/PC/VPS
   éteint ⇒ BLOCK{rig}, jamais RIG-ABSENT — le rig est engagé pour A) ; amendement
   canon T3 ratifié.

**GLOBAL** : ratifier README §4 les tokens palier-level T2 ACTED/MIXED/NOT-RUN
(Phase J) ; push groupé (Phase T) gaté sur **3 verts** (Woodpecker + rust-ci 3-OS
+ run nightly réel). Garde standing E2E web : tout nouveau spec semeur ⇒ projet
chromium-authoring / cleanup (ne pas re-casser browse-search empty-state).

---

## Invariants

- **0 bump wire SBFB** (Task / ProjectAnnouncement / CuratorList / FeedEntry
  byte-stables).
- **0 dep runtime ajoutée hors hickory** (PO-7=A) ; iroh reste `=1.0.1`.
- **Refacto = 0 changement de comportement observable** (count nextest = preuve).
- `heberger != publier, seeder != auteur` (cardinal).
- MUR jamais bouton, 0 verdict calculé UI, diff = vérité Rust, Factory hors daemon.
- Content-addressing BLAKE3 = vérité joignabilité.
- Pas de band-aid, root cause, pas d'emoji, 1 commit/phase, body 9 sections.
- **Ré-extraction findings** : `sprint81_audit_findings.md` ne détaille QUE les 4
  P1 ; le texte exact C-1/C-2, H-1/H-2, I-2, J-4/J-5, K-2, F-1..5 doit être
  ré-extrait des phase-reviews / agent audit — **NE PAS fabriquer** (dette inventée).

---

## Questions ouvertes — à trancher au preflight de phase (défauts recommandés)

> Les arbitrages load-bearing (PO-1..9) et les Day-0 (D1..D12) sont TRANCHÉS supra.
> Restent des détails de preflight ; défaut recommandé entre parenthèses.

- **[G]** Schéma request-bodies shard-session : option (a) `#[derive(JsonSchema)]`
  + snapshots `*.schema.json` drift-gatés (miroir des réponses) **vs** repli (b)
  tables Request-body `SHARD_PROTOCOL_SPEC §6` + `FRONTIER-NO-SCHEMA` motivé —
  *(défaut : (a) recommandée ; repli (b) si le derive introduit un churn de dep).*
- **[C]** ARB-CI-1 : installer `libgtk-3-dev` dans les 2 jobs `ci.yml` (miroir
  image sbfb-ci) pour GHA vert **vs** décâbler honnêtement GHA + câbler les 2 E2E
  Playwright + coverage web + scan-en-strings dans `.woodpecker/ci-linux.yml` —
  *(défaut : tenter GHA-GTK d'abord ; fallback Woodpecker E2E OBLIGATOIRE si le
  runner échoue).*
- **[N→S]** Découpage exact des 6 domaines split (bornes de lignes co-déplacées
  handler+DTO+tests) — *(défaut : bornes du spec §N→S ; ajuster au preflight si un
  handler chevauche deux domaines, jamais laisser un test orphelin).*
- **[T]** Redescente consents L4 PC+Mac post-quorum si applicable — *(note
  opérationnelle mineure ; vérifier au live-ops, pas un livrable code).*

---

## Carries entrants

> Vers S82, de l'audit S81 (`sprint81_audit_findings.md`, 4 P1 / 16 P2 / 14 P3).

- **S81-G-ESC-1 boot-SEED** (P1, OVERDUE 3/3) → **Phase A** (PO-1=B).
- **4 P2 / 10 P3 docs-contract S80 + doc-dette S81** (C-1/C-2/C-3, H-1/H-2, K-1/K-2,
  J-3/J-4, I-1/I-2, G-2, F-*) → **Phases E-J**.
- **S81-A3-2 volet infra GTK + integration-nightly** → **Phase C**.
- **S81-G-1 migration stores worker** → **Phase T** (artefact T2).
- **Réparation relay-gated multi_daemon 4/10** → **Phase D**.
- **Supply-chain** : hickory (PO-7=A) → **Phase K** ; veille iroh/ed25519-dalek →
  standing.
- **Benchmarks standards (PO-2)** → **Phase B**.

---

## Carries sortants (S82 → S83+)

- **Sharding feature/hardening** (R-J-6, F2, SI-12, N3-reveal, métriques) → slot
  rig-chaud.
- **Fixes robustesse sharding** (J1b-3, D3-2, D4-2, J-D5-1) → dette audit hors-thème.
- **Reprise arc front** → slot post-S82 dédié.
- **workflow-engine + Viewer fondation** → futurs slots.
- **Long tail split http.rs + fichiers secondaires >2000 l** → refacto futur.
- **Topologie A-vs-B** (avant 25/08) + gate n0 15/09 → calendaire.
- **app-authoring in-vivo Not evidenced** → standing.
- **Veilles supply-chain standing** → trigger-driven.

---

## Amendement roadmap (à acter Phase T)

- **roadmap v5** : insérer **S82 DONE** (dette docs-contrat + refacto http.rs +
  boot-SEED fermé + benchmarks + hickory) ; tracer les slots décalés
  (workflow-engine, Viewer fondation, reprise arc front) explicitement non-perdus.
- **Canon README §4** : amendement **T3** (tier benchmark + track audit + invariant
  kickoff) ratifié (Phase B) ; tokens palier-level T2 ACTED/MIXED/NOT-RUN (Phase J).
- **SPRINT_LOG.md** : row 82.

---

## Références (chemins absolus)

Racine : `C:\Users\FlowUP\Documents\Code\nexus\`.

- **Planning S82** : `.planning\active\sprint82_plan.md`,
  `.planning\active\sprint82_design_review.md`,
  `.planning\active\sprint82_audit_plan.md` (carries reroutés, §6 à corriger PO-9).
- **Audit gate S81 (Phase 0, JOUÉ)** : `.planning\active\sprint81_audit_findings.md`
  (reste en active/, findings N-1 ; détaille QUE les 4 P1 — ré-extraire le texte
  P2/P3 depuis les phase-reviews S81 archivées).
- **Artefacts S81 (ré-extraction dette Phases E-J)** : archivés dans
  `.planning\archive\v2.1\` — `sprint81_phase_*_review.md` (texte exact
  C-1/C-2, H-1/H-2, I-2, J-4/J-5, K-2, F-1..5), `sprint81_verification.md`,
  `sprint81_plan.md`, `sprint81_kickoff.md`, `sprint81_audit_plan.md`.
- **Canon workflow** : `docs\claude\README.md` (§6.12 clôture docs-contrat ; §4
  budget de phases + gate testabilité T1/T2/T3) ; `docs\claude\SPRINT_LOG.md` (row 82).
- **Ledgers de dette (re-audit COMPLET D9)** : `docs\rust\PATTERNS.md`,
  `docs\shell\PATTERNS.md`.
- **Docs sécurité (Phase I)** : `docs\security\THREAT_MODEL.md`,
  `...\LOOPBACK_ENDPOINTS_TRUST_TIERS.md`, `...\HARDENING_ROADMAP.md`,
  `...\VALIDATED_BLUEPRINT.md` (:156-157 iroh 0.97 stale).
- **Fichiers refacto** : `crates\nexus-shell-daemon\src\http.rs` (12460 l, split
  N→S) ; `...\runtime.rs` (5096 l ; `DaemonRuntime::start()` :276-1224 ; driver
  boot-SEED `http.rs:1819-1826`).
- **Contrat sharding + gates docs (Phase G)** : `SHARD_PROTOCOL_SPEC.md`,
  `check-frontier-contracts.sh` (:66 PROMISE_RE), `check-sharding-docs.sh`.
- **Supply-chain hickory (Phase K)** : `deny.toml`,
  `crates\nexus-shell-daemon\src\dns_fallback.rs`.
- **Roadmap (amendement Phase T)** : `.planning\roadmap_v5_factory_complete_vision.md`.
- **Staging workflow-engine (SUPERSEDED Phase E)** :
  `.planning\research\sprint82_workflow_engine\`.
- **Mémoire** : `po_benchmarks_standards_llm_sharding` (Phase B) ;
  `live_acceptance_setup` (T2 Phases A + T : SSH vps/mac, PROJECT_ID,
  `x-sbfb-token`, `b3_live_pc_vps.sh`).
