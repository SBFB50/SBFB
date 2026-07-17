# Sprint 83 — Audit plan (à jouer en Phase 0 de S83)

> Audite le **Sprint 82** (dette docs-contrat + refactorisation, Phase 0 +
> 24 phases A→T ; fenêtre `ad53940`..tip Phase T INCLUS = 29 commits :
> 2 Phase 0 voie A + kickoff `6fc263b` + chore acceptance `34550c1` +
> 23 commits de phase A→S4 + chore PO-10 `9ea7c05` + T). Écrit au
> wrap-up S82 Phase T, AVANT le gate push groupé PO-4=C — l'issue du gate
> (3 verts + push) est donc elle-même un objet d'audit (Track CI ci-dessous).

## 0. Mode d'emploi (session fraîche)

1. Lire ce fichier EN ENTIER avant tout autre artefact S82.
2. Former une opinion sur le diff AVANT de lire les phase-reviews et
   codex_reviews S82 (elles sont l'auto-évaluation du livreur, pas la tienne).
3. Ne PAS lire `sprint82_verification.md` §4/§9 avant d'avoir joué les
   commandes toi-même (self-report, README §2.3).
4. Livrable : `.planning/active/sprint82_audit_findings.md` (format §5).
5. Canon des tracks : `prompts/agent/audit-gate-checks.md` (11 tracks A..K).

## 1. Périmètre

- Fenêtre : `ad53940` (fix Phase 0 voie A) .. tip Phase T inclus
  (29 commits, décomposition dans le chapeau ci-dessus).
- Thème : AUCUNE feature produit. 3 familles de diff : (a) docs-contrat
  (E..J, T), (b) refacto behavior-preserving (L..S4 — gate D4 : ±0 test,
  0 route path, 0 bump wire, golden 9/9), (c) fixes ciblés (A boot-SEED,
  C/D CI, K hickory).
- Baselines d'invariance à re-vérifier : nextest Win **2108** / Docker
  **2112** (0 baisse) ; Vitest web **412** + E2E 44/2skip ; operator **201**
  + E2E 10 ; `wc -l crates/nexus-shell-daemon/src/http.rs` = **1513** ;
  routes `grep -c '.route(' http.rs` = **89** ; 3 gates docs exit 0.

## 2. Les 11 tracks (canon `prompts/agent/audit-gate-checks.md`)

Jouer A..K au complet. Signaux spécifiques S82 par track :

- **A (suites)** : rejouer les 4 blocs de `sprint82_verification.md` §3.
  Flakes ATTENDUS documentés : famille e2e sigint/running_json sous charge
  Docker-on-Windows (re-run solo = PASS ; 2 occurrences au run T). Un flake
  qui persiste EN SOLO = P1.
- **B (security)** : hickory K — vérifier `deny.toml` 0 ignore résiduel des
  4 RUSTSEC fermés + `cargo audit` propre ; LOOPBACK verrou D7 : le champ
  `inventory_policy` existe et la table §3 n'a PAS été réduite.
- **C (patterns)** : §P75 (rust) vs §P74 (shell, Phase A) — l'espace de
  numérotation est COMMUN ; toute nouvelle entrée S83 doit grep les DEUX
  fichiers (piège consigné verification §8.1).
- **D (scope)** : PO-10 « S82 = une fin » — vérifier 0 carry « split
  différé » ; les scope cuts kickoff §Out (13 items, verification §7)
  tous respectés.
- **E (tests delta)** : +13 Win/+13 Docker EXACT (A +2, B +1, C +0, D +1,
  M +9 ; splits ±0). Un delta caché dans un split = P1 (behavior-preserving
  violé).
- **F (review files)** : 24 phases × (preflight + review + codex_review)
  présents ; review S2 : vérifier l'incident « verdict pré-écrit » resté
  corrigé (PASS-PENDING avant Codex).
- **G (carry-overs)** : cf. §3.
- **H (HARDENING drift)** : standing.
- **I (meta-process)** : preflights Workflow (3 EXECUTE P/S2/S3, PLAN-ADAPT
  ailleurs, 0 DESIGN-CONFLICT) — vérifier que chaque PLAN-ADAPT porte son
  evidence disque.
- **J (testabilité standing)** : `sprint82_verification.md` §Acceptance —
  T1 {GREEN,RED,N-A-no-frontend-change} + T2 top-level
  {PASS,BLOCK,RIG-ABSENT,N-A} + paliers {ACTED{evidence},MIXED,NOT-RUN}.
  Vérifier : T1 = GREEN **enregistré** (daemon.ts touché R/S3 comment-only
  — PAS un N-A sec) ; ACTED des paliers bootseed/benchmarks sanctionnés au
  preflight T ; artefact store-migration PASS avec mode par palier.
- **K (docs-contract closure standing)** : frontière neuve S82 = les 3
  request-bodies Phase G — index 4 surfaces + 9 anchors ROW/ENTRY-level
  dans `check-sharding-docs.sh` (retirer la row de table REFERENCE ou
  l'entrée llms DOIT faire échouer le gate même si le nom survit en prose
  — tester par mutation) ; **écart lettre-D8 ratifié**
  (0 tag `// FRONTIER:` sur les 3 DTOs, pattern snapshot+prose — consigné
  verification §9, ne PAS le re-lever comme gap) ;
  S82-DC-LOOPBACK-INVENTORY-EXHAUSTIVE = CLOSED-BY-POLICY (27/89 par
  POLITIQUE — ne PAS re-lever le delta 62 paths comme drift).

## Track G1 presence (P1 bloquant si absent — canon README §2.4.3)

Vérifier que `sprint82_design_review.md` existe dans
`.planning/archive/v2.1/` (archivé au chore de kickoff S83) avec scoring.
Absent sur sprint non-trivial = **P1** (gate bypassé, précédent S26).
Présent sans scoring = P2. Présent avec scores = OK.

## Track HARDENING drift (P2 informatif — canon README §2.4.4)

Comparer `HARDENING_ROADMAP.md` (items prescrits fenêtre S82) vs livré.
S82 est un sprint dette : le drift attendu est faible ; tout item prescrit
non livré sans scope-cut kickoff ni blocker documenté = P2 informatif.

## Track CI — issue du gate push PO-4=C (spécifique S82, BLOQUANT)

Question centrale : le push groupé a-t-il RÉELLEMENT eu lieu sur 3 verts ?

- Si le push a été joué : `git rev-parse origin/master` == tip Phase T ;
  `gh run list --workflow rust-ci.yml --limit 5` → success 3-OS sur le tip
  (ou l'arbitrage PO macos-14 documenté) ; `gh run list --workflow
  integration-nightly.yml` → ≥1 run complété + artefact `junit-integration`
  téléchargeable ; Woodpecker ci.sbfb.world → pipeline vert sur
  codeberg/master à jour. Chaque vert manquant SANS arbitrage PO consigné
  = **P1**.
- Si le push n'a PAS été joué (PO a différé) : vérifier que la verification
  §9 l'a consigné et router le gate — PAS un finding contre S82.
- Bruit rouge HORS gate (ne pas confondre) : deploy.yml startup-failure 0s,
  Build worker binaries, canary-monthly (tag-triggered), Mirror tant que
  l'auth Codeberg n'est pas re-provisionnée.

## 3. Carries à router (inventaire nommé)

### Compteurs / escalades à surveiller

- **app-authoring S79 in-vivo `Not evidenced`** — P1 STANDING (S79→S82,
  4 sprints). Décision fermante DUE (§6.2.1) : parcours in-vivo OU
  requalification PO explicite. Ne pas re-reporter sec.
- **Arc front parqué** `wip/factory-front-arc-post-s82` — review + Codex
  groupés DUS à la reprise (PO-6 : post-S82 = MAINTENANT). Rebase conflit
  attendu `provider_router.rs`.

### P2 (nommés, évidence session T)

- **Classe watcher macos-14** : **10 tests uniques** fs-watcher/hot-reload
  TRY-2-FAIL (20 lignes, récap nextest dupliqué — rectif Codex T round 1)
  sur 2 runs Rust CI master observés (28661119376 head `c899d54` +
  28592686238 head `d8246bd`) — bloque « rust-ci 3-OS » ; jamais traité.
  Fix (override nextest/notify backend) OU arbitrage PO leg non-bloquant.
- **TEST-ISOLATION state.json — CLASSE, ≥2 sites** : des tests bootant un
  `Engine` réel sans confiner `NEXUS_GRID_ROOT` écrivent le state.json
  worker RÉEL (`%APPDATA%/nexus-grid/worker/state.json`). 2 pollueurs
  observés 2026-07-17 : test cross-node `result_sync.rs:494` (fixture
  « rsync xnode », `data_dir: None`) ET
  `nexus-worker-core/src/engine/runtime.rs:1956/:2085` (fixtures
  « rate-limit test »/« proj-rl »/« t-fresh », réécriture 14:28Z pendant
  les suites wrap-up T). Le fix S83 doit traiter la CLASSE (grep tous les
  `Engine::new`/`EngineBoot` de test sans confinement du root), pas un
  site unique (artefact store-migration §evidence_hygiene + review T).
- **0 golden feed/search/provenance/preview/proof-card/browse/nodes**
  (classe consignée Codex S2/S3/S4) + **browse_pull sans test direct**.
- **Famille dispatch_loop** candidate test-group nextest borné (flake
  parallélisme défaut, consigné Codex S4) + famille e2e sigint/running_json
  (2 flakes sous charge au run T).
- **Sandbox codex `elevated` CASSÉ** (« missing field sandboxPolicy », CLI
  0.144.1, depuis 2026-07-16) — workaround `--sandbox read-only` en place ;
  réparation à statuer PO.
- **Auth Codeberg CASSÉE** (rectif Codex T round 1 : le job Mirror masque
  un token NON-vide mais le push échoue en authentification — pas un
  « secret manquant » ; Mirror rouge → Woodpecker aveugle) —
  re-provisionner le secret = action manuelle PO settings GitHub, hors repo.
- **Worker sans recreate-guard** (analogue daemon absent — residual_risk de
  l'artefact store-migration ; sévérité basse, état re-dérivable).
- **Volet Mac différé** : observation on-disk store + hygiène rule 4 +
  redescente consent L4 — à jouer à la prochaine session Mac chaude
  (artefact store-migration).

### P3

- `deploy.yml` startup-failure 0s à chaque push master (trigger
  workflow_dispatch seul) — diagnostiquer ou décâbler.
- `SPRINT_LOG.md` row 76 « http.rs:8531 » : narration historique
  out-of-bounds TOLÉRÉE (passé immuable) — ne pas « corriger ».
- actionlint jamais joué localement (routé review D) ; shellcheck absent
  localement (les scripts S82 modifiés sont couverts par le workflow GHA
  shellcheck au push).
- M-7 nit « split target » (http.rs doc-comment, review M) — optionnel.

### Ré-extraction (anti dette-inventée)

Le texte exact des P2/P3 par phase vit dans les 24
`sprint82_phase_*_review.md` + `*_codex_review.md` (archivés v2.1 au
kickoff S83) — ré-extraire, ne PAS reformuler de mémoire.

## 4. Out-of-scope de l'audit

- Les Day-0 D1..D12 + PO-1..PO-10 (gelés, dont PO-10 supersede la clause
  « long tail différée » de D3).
- Le choix representative-locked D7 (27/89 = politique, pas drift).
- L'écart lettre-D8 ratifié (snapshot+prose sans tag littéral).
- Le décalage workflow-engine/Viewer/arc-front (C9/PO-9, tracé roadmap v5).
- Les pins de deps (iroh =1.0.1, hickory 0.26) et la politique pre-launch.

## 5. Format du livrable

`sprint82_audit_findings.md` : auditeur/durée, tip audité, verdict global
(PASS / CONDITIONAL PASS / FAIL — PASS exige ≥1 P2+ documenté, cf. G4),
une section par track (A..K + Track CI) avec verdict et findings
P0/P1/P2/P3 évidencés fichier:ligne, commits `fix(sprint82)` pour les
P0/P1.

## 6. Note

Le slot S83 n'est PAS pré-arbitré ici : candidats tracés roadmap v5 bloc
S82 (workflow-engine / Viewer fondation / reprise arc front — la reprise
arc front porte une dette de review NON auditée qui plaide pour tôt) +
re-décision calendaire Topologie A-vs-B due avant 2026-08-25 (PO-5).
L'arbitrage revient au PO au kickoff S83.
