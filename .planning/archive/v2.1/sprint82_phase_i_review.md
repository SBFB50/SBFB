# Sprint 82 Phase I — Review

Date : 2026-07-14. Review Workflow ultracode en 2 rounds : round 1 =
4 dimensions lancées (security + process abouties ; diff-fidelity +
livrables mortes au cap StructuredOutput — outputs trop volumineux),
round 2 = les 2 dimensions manquantes re-jouées en consignes concises
sur le diff POST-FIX round 1. Total 10 agents opus-4-8[1m] aboutis
(6 review + 4 vérification adversariale ; ~1.15M tokens, 211 tool
calls). Chaque dimension a sa vérification adversariale (citations
relues au disque, défauts re-prouvés, sévérités re-cotées). Arbre
stable pendant chaque round (les fixes SEC-I-* ont été appliqués ENTRE
les rounds, et le round 2 a re-vérifié les 3 fixes au disque).

## Verdict: PASS

Review Claude : 0 P0, **1 P1 corrigé in-phase**, 2 P3 corrigés
in-phase, 1 P3 REFUTÉ par l'adversarial (sur-cotation). Les 4
dimensions rendent PASS après fixes, avec 0 fabrication dans les
citations des reviewers (round 2 : « zero wrong pointer found »,
« 7/7 livrables tracés, 0 scope creep »). Codex GPT-5.6 Sol :
2 rounds — round 1 GAPS (7 écarts, 2 P1 + 4 P2 + 1 P3, TOUS
corrigés + vérifiés adversarialement), round 2 : fond CORRIGÉ
confirmé + 4 finitions P2/P3 corrigées et re-prouvées machine.
Verdict promu PASS après réconciliation (cf. § Codex
reconciliation ; critère d'arrêt boucle : 0 P0/P1 restant,
P2/P3 corrigés-documentés).

## Findings et dispositions

- **SEC-I-1 (P2 → UPGRADE P1 par l'adversarial, CORRIGÉ in-phase)**
  Mon édit `THREAT_MODEL.md` §5.6 (row consent.json) attribuait
  l'écriture atomique « cote worker (`atomic_write_json`,
  `nexus-worker-core/src/consent.rs`) » — FAUX sur l'attribution de
  process : le chemin UI réel est `POST /api/v1/consent/set` →
  `set_consent` → `save_consent` (daemon, `nexus-shell-daemon/src/
  consent.rs`, tmp+rename inline) ; le worker ne fait que LIRE via
  `ConsentWatcher`. Contradisait la doc du module cité ET 3 autres
  cellules du même diff (re-labellisation coordinator=daemon). La
  classe exacte « la passe de fidélité introduit l'erreur qu'elle
  corrige » (P1 en Phase H). **CORRIGÉ** : writer = daemon
  `save_consent`, worker = lecteur `ConsentWatcher`. Re-prouvé au code
  par le round 2 (dimension diff-fidelity + livrables, CONFIRMED ×2).
- **SEC-I-2 (P3, CONFIRMED, CORRIGÉ in-phase)** `SHARD_PROTOCOL_SPEC`
  §5.2 : ma borne disait « the attestation interception runs for every
  session of the ALPN » — imprécis : la branche est gardée par
  `is_real_stage` (un echo transport-only n'intercepte JAMAIS,
  `shard.rs`). **CORRIGÉ** : « carries the attestation-interception
  branch for every ALPN session, and that branch fires only for a real
  stage ». `check-sharding-docs.sh` re-joué exit 0 post-fix.
- **SEC-I-3 (P3, CONFIRMED, CORRIGÉ in-phase)** Le row npm-postinstall
  citait « audit-ci (npm) bloquants » sans le seuil réel :
  `web/audit-ci.json` = `"critical": true` (critical-only, pas
  moderate/high). **CORRIGÉ** : « audit-ci (npm, seuil critical-only) »
  — le res=L reste défendable (lockfile = défense primaire, R2 porte le
  résiduel).
- **P3-deploy-py-numeric-pointer (REFUTÉ par l'adversarial)** La plage
  `:1-475` conservée sur `deploy.py` (fichier purgé) dans THREAT §7
  n'est PAS un défaut : pointeur FIGÉ reproductible dans le commit
  `10bbc63` de la colonne Commit (ne peut plus pourrir), encadré par la
  note de lecture §7 ajoutée par la phase. Asymétrie voulue :
  fichiers VIVANTS = ancres symbole, fichiers PURGÉS = plage
  historique figée. 0 édit.

## Dimensions (4/4 PASS après fixes)

1. **diff-fidelity (round 2) : PASS, findings=[]** — chaque claim
   factuel NOUVEAU re-prouvé au lock/code/disk : versions (iroh 1.0.1,
   gossip 0.101.0, blobs 0.103.0, docs 0.101.0, blake3 1.8.5, dalek
   2.2.0, governor 0.10.4 — « 0.10 » = troncature sûre), ~30 symboles
   re-ancrés tous résolus, ASCII-art DFD cohérent, 0 ancre numérique
   périmée résiduelle (:229/:103/:822/:866/:898/:248 tous absents).
2. **livrables (round 2) : PASS, findings=[]** — matrice 7/7 livrables
   plan+preflight tracés au diff ; extension bornée même-doc
   (blake3/dalek/URL) documentée au preflight ; 0 scope creep ;
   `security_posture.md` volontairement non édité (artefact généré) ;
   gates exit 0 re-joués par l'agent.
3. **security (round 1) : PASS après fixes** — bornes K-R-7 fidèles au
   code (`is_real_stage`, echo digest-zeros) et n'affaiblissent aucune
   garantie ; catalog_len=0 exact au code et renforce
   « seeder != auteur », réouvertures wire-gated (pas des promesses) ;
   aucun claim « supply-chain green » (câblé-par-config seulement) ;
   re-labellisation coordinator exacte (coordinator-rs = lib sans
   axum ; surface HTTP = daemon) ; zones R-*/T20/HARDENING §3 INTACTS.
4. **process (round 1) : PASS, 0 défaut** — présent-vrai partout,
   passé immuable tenu (§7 note de lecture sans réécrire les rows S16,
   HARDENING :181/§3 intacts), ancres symbole sans nouveau pointeur
   numérique, discipline ASCII/accents par doc respectée,
   last_validated LOOPBACK 2026-07-14 avec précédents conservés (pas
   de champ fabriqué ailleurs), staging = 8 modifiés + les artefacts
   de phase untracked (preflight, puis review et codex_review produits
   par le process — exclus du payload docs audité), le diff suit le
   PLAN-ADAPT (étiqueté « ré-audités/re-dérivation », jamais
   « ré-extraits »). Précision (réconciliation Codex P3) : la row
   PA v5 de THREAT §7 a bien été ÉDITÉE (clarification VERSION=5
   prescrite par le preflight) — ce sont les AUTRES rows S16 qui
   restent intactes, encadrées par la note de lecture.

## Suites §7.4 (toutes vertes)

- **Rust Windows** : `cargo fmt --check` + `clippy -D warnings` +
  **nextest 2100/2100** + doctests + `build -p nexus-shell-daemon
  --release` — VERT (delta tests 0, attendu docs-only).
- **Rust Docker `sbfb-ci`** (rust:1.94 + GTK) : fmt + clippy verts ;
  **nextest 2104/2104, 0 skipped** + doctests verts (run final machine
  au calme).
- **Web** : lint (0 erreur) + tsc + unit **412/412** + coverage
  **87.27 / 79.01 / 86.02 / 88.59** (seuils 85/78/85/85) + build +
  size + `scan-en-strings` clean.
- **SPDX** : 352 fichiers conformes.
- **Critères machine de la phase** : `grep '0.97'
  VALIDATED_BLUEPRINT.md` = 0 hit ; `check-sharding-docs.sh` exit 0
  (re-joué post-SEC-I-2) ; `check-frontier-contracts.sh` exit 0
  (census DOMAIN 25 frozen) ; baseline wire 13 constantes `*_VERSION`
  toutes = 1 inchangées.

## Classes env observées (documentées, aucune comptée comme régression)

- **Vitest coverage sous charge croisée** : 1 timeout 5000ms
  (`GpuConsentDialog` whitelist) pendant que nextest Win + Docker +
  agents saturaient la machine ; solo 17/17 PASS puis full coverage
  re-run VERT. Classe `vitest_env_variance` connue.
- **2 e2e daemon iroh-networked sous Docker-on-Windows chargé**
  (`start_writes_running_json_and_responds_to_health`,
  `sigint_triggers_graceful_shutdown_and_removes_running_json`) :
  « GET /health must succeed within 5s » dépassé sous parallélisme +
  relais n0 réels ; solo PASS 2/2, full run calme 2104/2104 PASS.
  Voisine de la classe documentée des 6 iroh-networked.
- **`cargo test --workspace` shared-process Docker** (bloc README
  pré-push) : `auth::tests::run_dir_paths_resolve_under_sbfb_home`
  échoue par race d'env-var process-globale — le test set/unset
  `SBFB_HOME` avec la note in-code « SAFETY: test-only; nextest runs
  each test in its own process » : conçu pour le runner canonique
  nextest, non sérialisé sous `cargo test` threads partagés. Solo
  PASS. **Dette de test PRÉ-EXISTANTE** (code non touché par la
  phase, docs-only) : consignée pour le ledger — le bloc pré-push
  Phase T (PO-4=C) devra soit sérialiser ces tests env-var, soit
  basculer le bloc canonique README sur nextest. Candidat P2 routé
  au commit body.

## Codex reconciliation

**Round 1 (GPT-5.6 Sol, effort max) : verdict GAPS** — 3 CONFIRMÉ
(G-D5-1, K-2, flip PATTERNS + LOOPBACK), 4 PARTIEL, 1 GAP ; 7 écarts :
2 P1 + 4 P2 + 1 P3. Chaque claim Codex a été RE-PROUVÉ au code avant
correction (0 recopie aveugle) — tous tenaient :

- **P1-1 (baisse supply-chain H→L non fondée)** : un scanner
  d'advisories critical-only ne borne pas un postinstall malveillant
  frais sans advisory. CORRIGÉ : row §5.8 res restauré **H** avec
  rationale explicite ; R2 re-titré « M à H selon dep » + résiduel
  « le scanner ne borne que le CONNU ».
- **P1-2 (fausse sémantique de signature catalog_len)** : ma
  consignation disait « n'atteste que ce que le signataire publie » —
  le contrat `node_directory.rs` dit l'inverse (« I claim to host
  these hashes », catalogue « hosts (or seeds) » au wire ; verrou-4 ne
  couvre que la PROVENANCE). 3e occurrence de la classe « la passe
  introduit l'erreur qu'elle corrige », attrapée cette fois par le
  cross-model. CORRIGÉ (THREAT §15.1 + §P59.8) : signature =
  hébergement ; own-published-only = POLITIQUE daemon (`own_entries`),
  pas contrainte wire ; réouvertures scindées (a) inclusion non
  étiquetée = CODE-ONLY wire-compatible / (b) section « seeded »
  distincte = wire change / (c) SearchManifest.
- **P2 (DFD encore faux)** : drifts RATÉS par ma re-dérivation —
  `:7777` (réel : `api_port` défaut 0 éphémère publié dans
  `running.json`, `config.rs`), blob-serve présenté en origin/port
  séparé (réel : route publique du MÊME listener `public_routes`,
  isolation par origin OPAQUE sandbox), `deploy` attribué à
  coordinator-rs (réel : handler daemon `deploy.rs` ; lib = DB/
  dispatch/validator/kudos). CORRIGÉ (scope :18-:19 + DFD réécrit).
- **P2 (THREAT §16 contredit `is_real_stage`)** : la même imprécision
  que SEC-I-2, non reportée dans §16. CORRIGÉ (aligné SPEC §5.2 :
  branche gated `is_real_stage`).
- **P2 (pip-audit inopérant)** : le job S18 cible 3 packages Python
  purgés S50-S51 (Codex a exécuté `uv export` → exit 2). CORRIGÉ :
  qualifié INOPÉRANTE dans §5.8 + §7 + R2 + VALIDATED ; réparation/
  suppression du job CI **routée au ledger** (hors périmètre
  docs-only).
- **P2 (« bloque PR » non prouvé)** : reformulé « jobs déclenchés sur
  PR, fail non-zéro par config » partout ; puce historique
  HARDENING :181 conservée avec annotation datée S82 Phase I (passé
  immuable tenu, contradiction interne levée).
- **P3 (cohérence du review)** : réconcilié ci-dessus (§ dimensions,
  point 4).

**Vérification adversariale des 7 corrections** (Workflow 2 agents
opus-4-8[1m]) : 5 CONFIRMED + 1 CORRECTED (gap résiduel
HARDENING :181, fermé par l'annotation datée ci-dessus) + confirmation
que la classe récursive n'a PAS récidivé (symboles re-prouvés,
ASCII-art DFD re-lu boîtes fermées, 0 accent ajouté aux docs
ASCII, 8 fichiers .md seuls, 0 constante wire).

**Boucle complète re-jouée post-corrections** : gates
`check-sharding-docs` + `check-frontier-contracts` exit 0 ;
`grep '0.97'` = 0 ; `git diff --check` propre ; fmt + nextest Windows
re-run **2100/2100** (code inchangé, docs-only) ; Docker/web
inchangés depuis leurs runs verts (aucun fichier code/web touché
par les corrections).

**Round 2 (post-corrections, brut concaténé à la suite du round 1
dans le fichier codex_review)** : les écarts de FOND sont jugés
**CORRIGÉ** (P1-1 supply-chain H restauré, P1-2 sémantique signature,
§16 is_real_stage, DFD port/listener/deploy) et les non-régressions
confirmées (G-D5-1, K-2, flip PATTERNS, LOOPBACK, versions au lock).
Le round 2 a relevé 4 finitions P2/P3, TOUTES corrigées puis
re-prouvées machine dans la même fenêtre pré-commit :
(1) row scope :19 — chemin `blob_serve.rs` pré-existant pointait le
mauvais crate → corrigé `nexus-shell-daemon-core/src/blob_serve.rs`
(+ handler daemon `http.rs`), vérifié `ls` ;
(2) POST_CHATONS « chaque PR » sur-large → « chaque PR vers
`master`/`main` » (= `supply-chain.yml` on.pull_request.branches) ;
(3) le routage ledger de la réparation pip-audit, affirmé par cette
review sans être matérialisé → entrée datée
**PIP-AUDIT-JOB-INOPERANT (P2, ROUTED slot CI/hardening)** ajoutée au
ledger E ;
(4) le séquencement de la concaténation round 2 → faite (le fichier
codex_review porte les DEUX rounds bruts, séparateur méta seul ajout).
**Critère d'arrêt de boucle atteint** (règle : CLEAN ou P2/P3
documentés — jugement de sévérité porté ici) : 0 P0/P1 restant, les
4 finitions sont corrigées ET vérifiées machine, les gates re-joués
verts après chaque vague (`check-sharding-docs` + `check-frontier-
contracts` exit 0, `grep '0.97'` = 0, `git diff --check` propre,
0 fichier code/lock touché — contrôles également re-joués par Codex
round 2 lui-même : « tous propres »).
