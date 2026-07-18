# Sprint 82 — Verification (wrap-up Phase T)

> Sprint de **dette docs-contrat + refactorisation** (arbitrage C9 PO
> 2026-07-11 ; PO-9 ratifie le décalage workflow-engine/Viewer ; amendement
> in-sprint PO-10 « S82 = une fin » 2026-07-15). Phase 0 (audit gate S81
> FAIL levé voie A) + **24 phases A→T**. Aucune feature produit : fermeture
> de dette S79/S80/S81, réparation d'un défaut LIVE (boot-SEED), http.rs
> rendu maintenable (fin S81 : 12460 l → 1513 l ; le repère 13130 = pic
> post-golden-M/pré-split-N, borne de la série N→S4).

## 1. HEAD entrée / HEAD sortie

- **Entrée** : `95ff46c` (chore Phase 0 voie A — l'audit gate S81 [FAIL : 0 P0,
  4 P1, 16 P2, 14 P3] est levé par `ad53940` fix(sprint81) + ce chore ;
  origin/master = `c899d54`, S81 Phase B).
- **Kickoff** : `6fc263b` (kickoff + plan + design review + archive S81).
- **Sortie** : commit Phase T (ce commit).

## 2. Commit stack (fenêtre S82)

| Phase | Commit | Titre court |
|---|---|---|
| (0) | `ad53940` + `95ff46c` | audit gate S81 : 4 P1 fixés voie A + findings |
| kickoff | `6fc263b` | activation + archive S81 |
| A | `19b92e6` | boot-SEED OVERDUE 3/3 : re-drive-on-ingest + catch-up worker (T2 live PASS) |
| chore | `34550c1` | acceptance T2 boot-SEED committée (artefact `sprint82_t2_bootseed.json`) |
| B | `1670251` | benchmarks standards sharding + canon T3 (BLOCK{rig} honnête) |
| C | `2931b82` | restauration surfaces CI (GTK, nightly calibré K-R-13/14) |
| D | `c7b6790` | multi_daemon relay-gated 6/6 réparés (+1 hermétique) |
| E | `f727f8c` | ledgers réconciliés D9 + staging workflow-engine SUPERSEDED (PO-9) |
| F | `21674f5` | PROMISE_RE élargi until/when + task_response passé immuable |
| G | `d2705b7` | schémas request-body shard-session + census DOMAIN 25 frozen (D8) |
| H | `32a23f6` | Track C fidélité + tripwire suffixe backup redb |
| I | `747470b` | doc-dette sécurité + catalog_len=0 accept-and-document (PO-8) |
| J | `57e19ad` | doc-dette process + vocab T2 palier ACTED/MIXED/NOT-RUN ratifié |
| K | `713f0fa` | hickory 0.24→0.26, 4 RUSTSEC clos (PO-7=A) |
| L | `013b611` | décomposition `DaemonRuntime::start()` 958→602 l |
| M | `29a9255` | golden net HTTP 9 tests + dédup harness (§P75) |
| N | `2e87eef` | split shard-session → `shard_session_http_api.rs` |
| PO-10 | `9ea7c05` | chore amendement « S82 = une fin » (N2 + S2→S4) |
| N2 | `c5be6e4` | `test_support.rs` : harness partagé + famille golden |
| O | `542254b` | split seed → `seed_api.rs` (discipline étendue) |
| P | `1aa7a0f` | split frost → `frost_api.rs` |
| Q | `7faa632` | split coordinator → `coordinator_api.rs` |
| R | `f7d42bc` | split curators → `curators_api.rs` |
| S | `be7e2be` | split publish → `publish_api.rs` |
| S2 | `b9b892a` | split browse+nodes → `browse_api.rs` |
| S3 | `0a32ffa` | split feed/search/preview → 3 modules |
| S4 | `32abfab` | sweep final : `blob_serve_http.rs` + canary/diagnostic ; **http.rs 1513 l, PO-10 PASS** |
| T | (ce commit) | clôture docs-contrat + roadmap + migration stores + gate push |

## 3. How to re-run

```bash
# Bloc Rust Windows (natif)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked          # 2108, 0 skip
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Bloc Docker canonique (image sbfb-ci = rust:1.94 + libgtk-3-dev)
MSYS_NO_PATHCONV=1 docker run --rm \
  -e CARGO_TARGET_DIR=/workspace/target-linux \
  -e SBFB_TEST_HTTP_TIMEOUT_SECS=120 \
  -v "C:\Users\FlowUP\Documents\Code\nexus:/workspace" -w /workspace \
  sbfb-ci bash -c 'cargo fmt --all --check && \
    cargo clippy --workspace --all-targets --locked -- -D warnings && \
    cargo nextest run --workspace --locked'     # 2112 (famille sigint/running_json
                                                # flake sous charge, re-run solo PASS)

# Bloc web
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && npm run build && \
  npm run size && bash scripts/scan-en-strings.sh)
(cd web && npm run test:e2e)                    # 44 passed / 2 skipped

# Bloc operator
(cd tools/factory-operator && npm run test:unit && npx tsc -b && \
  npm run lint && npm run test:e2e)             # 201 + 10 E2E

# Gates docs (critère machine DOCS-CONTRAT, désormais discriminants Phase T)
bash scripts/check-sharding-docs.sh
bash scripts/check-frontier-contracts.sh
bash scripts/check-factory-docs.sh

# T2 live (Phase T)
cat .planning/active/sprint82_t2_store_migration.json   # PASS
cat .planning/active/sprint82_t2_bootseed.json          # PASS (Phase A)
cat .planning/active/sprint82_t2_benchmarks.json        # BLOCK{rig} honnête (Phase B)
```

## 4. Checklist (Observed) — livrables du plan §Phase T

| Livrable | Observed | |
|---|---|---|
| Index 3 request-bodies — 4 surfaces | SPEC §3+§6.1 et WIRING_SPEC:179-186 DÉJÀ livrés Phase G (constat, 0 ré-écriture) ; travail T réel = `docs/sharding/llms.txt` (2 entrées) + GUIDE (`REFERENCE.md` Types +5 rows dont backfill `ShardSessionResultView`/`ShardSessionResultResponse` S81-I + renvoi `HOW_TO_WIRE.md` §6.1) | ✅ |
| Critère machine discriminant | +9 `anchor_present` ROW/ENTRY-level dans `check-sharding-docs.sh` (llms.txt entrée+2 snapshots, REFERENCE 3 rows de table, HOW_TO_WIRE 3 noms) — avant Phase T les 3 gates passaient SANS l'index (non-discriminant, constaté au preflight) ; discrimination PROUVÉE par mutation (row supprimée → MISSING ANCHOR exit 1, restauré vert) ; 3 gates exit 0 re-joués après éditions | ✅ |
| LOOPBACK §3 tier-target verrouillé (D7) | champ front-matter dédié `inventory_policy: representative-locked` (critère d'inclusion + familles + garde-fou triggers existants) + chapeau §3 + micro-fix ancre « §3 ligne 55 »→row par path ; track S82-DC-LOOPBACK-INVENTORY-EXHAUSTIVE = **CLOSED-BY-POLICY** (27/89 assumé) | ✅ |
| CLAUDE.md CI claim réconcilié | périmètre ÉLARGI à l'état réel (preflight + rectifs Codex round 1) : « Woodpecker opérationnel » STALE (codeberg gelé `f4b4600`, auth Codeberg CASSÉE — token non-vide masqué mais push en échec d'auth, PAS « secret manquant »), TOUS les workflows GHA master rouges (pas seulement « CI »), macos-14 classe watcher **10 tests uniques** (20 lignes TRY-2-FAIL, récap dupliqué) routée S83, réparations Phase C/K listées, état S82 DONE + compteurs 2108/2112 | ✅ |
| Roadmap v5 amendé | bloc daté « LIVRAISON 2026-07-17 (S82 Phase T) — S82 DONE » + **3 slots décalés tracés** (workflow-engine [staging SUPERSEDED, faits à re-valider], Viewer fondation, arc front `wip/factory-front-arc-post-s82` review+Codex groupés DUS) + note SUPERSEDED inline sur le claim stale C9 « pas encore tranché » (:87-90) | ✅ |
| SPRINT_LOG row 82 | insérée au-dessus de row 81 (newest-first, 5 colonnes modèle row 81 ; pas de row 78 — S78 absorbé, à ne pas « corriger ») | ✅ |
| Migration stores worker redb 2→4, 3 nœuds (D12) | `sprint82_t2_store_migration.json` **PASS** par CLASSE de store : PC standalone = migré (sibling observé puis rule-4 supprimé après capture) ; PC/VPS local-worker = recreate-fresh/absent PAR CONSTRUCTION (`local_worker.rs` provision wipe — la claim t2_h:48 « will auto-migrate » est RÉFUTÉE et corrigée) ; Mac = functional-by-construction (hôte froid ; preuve committée b3_p2_quorum PASS 2026-07-11 sous le lock 1.0.1) | ✅ |
| Note op consents L4 (kickoff [T]) | PC : `~/.sbfb/consent.json` level 4→1 APPLIQUÉ (worker éteint vérifié, caps+whitelist intacts) ; Mac : DIFFÉRÉ hôte injoignable (consigné artefact) | ✅ |
| Run réel integration-nightly (S81-J-2) | **au gate push post-commit** : le workflow n'existe pas sur origin (404 vérifié) — dispatch `--ref master` + `gh run download -n junit-integration` après le push groupé ; si le gate échoue, S81-J-2 reste PARTIAL et re-route S83 | ⏳ gate |
| rust-ci 3-OS vert sur le tip | **au gate push** : lecture bi-temporelle PO-4=C à ratifier PO (2/3 verts post-push par construction : nightly absent d'origin, Woodpecker gelé) ; séquence = branche staging `ci/s82-tip` + dispatch rust-ci (risque connu : classe watcher macos-14) → push master → Woodpecker/codeberg + nightly | ⏳ gate |
| verification + audit plan + agrégat | `sprint82_verification.md` (ce fichier) + `sprint83_audit_plan.md` + `sprint82_t2_acceptance.json` — livrables de fermabilité README §3.3 OMIS du plan §T, AJOUTÉS au scope par le preflight (précédents durs `8b3590c`/`d8246bd`) | ✅ |

## Acceptance (gate de testabilité — README §4)

### T1 — E2E hermétique (verdict ∈ {GREEN, RED, N-A-no-frontend-change})

- **Surface web** (`web/e2e/`, `npm run test:e2e`) : **GREEN** — 44 passed /
  2 skipped (env-gated `@shard` + `@compute`) / 0 failed, EXIT=0 (run
  2026-07-17, wrap-up T). Enregistré et non N-A : `web/src/api/daemon.ts` a
  été touché par R `f7d42bc` et S3 `0a32ffa` (re-points de commentaires,
  1 ligne chacun) et `web/e2e/app-authoring.spec.ts` par S4 — la règle
  kickoff (leçon S81-J-1) n'a pas d'exemption comment-only, le GREEN est
  donc joué et consigné avec ce rationale.
- **Surface operator** (`tools/factory-operator/e2e/`) : **GREEN** — 10/10
  Playwright hermétiques (run 2026-07-17).
- **Refacto (type 2, L..S4)** : T1 = GREEN non-régression via le gate D4 —
  nextest Win 2108 (0 baisse) + Docker 2112 + clippy -D warnings + fmt
  --check + 0 route path + 0 bump wire + goldens 9/9 (`golden_http_*`).
- **Docs-contrat (type 1, E..J, T)** : critère machine = 3 gates docs
  exit 0 (re-joués après chaque édition T ; 9 anchors neufs row/entry-level).

### T2 — acceptance JSON (status ∈ {PASS, BLOCK, RIG-ABSENT, N-A-no-cross-machine-feature})

- **status = PASS** (top-level) — `sprint82_t2_acceptance.json`, agrégat
  patron S80/S81 ; vocabulaire palier-level {ACTED{evidence}, MIXED,
  NOT-RUN} ratifié au canon par S82 Phase J et appliqué ici, sanction des
  ACTED au preflight T (`wf_818d5f99-6aa`).
- Paliers : `bootseed_live_replay` = **ACTED** (evidence :
  `sprint82_t2_bootseed.json` PASS, Phase A, exit (c) live < 30 s — jamais
  un RIG-ABSENT tapé main) ; `benchmarks_standard_t3` = **ACTED**
  (evidence : `sprint82_t2_benchmarks.json`, **BLOCK{rig: cold}** honnête,
  tier T3, rig engagé pour A donc jamais RIG-ABSENT) ;
  `worker_store_migration_3_nodes` = **PASS**
  (`sprint82_t2_store_migration.json`, produit Phase T, D12) ;
  `push_gate_po4c` = **NOT-RUN** (séquencement honnête : post-commit,
  actions sortantes à confirmer PO ; audité S83 Track CI).
- Lignée `b3_live` : le re-jeu live Phase A s'appuie sur le setup memory
  `live_acceptance_setup` (cibles vps/mac) ; le quorum `b3_p2_quorum` PASS
  2026-07-11 (S81 K) sert de preuve fonctionnelle committée au palier Mac
  de la migration stores.

## 5. Métriques sprint

| Suite | Avant (fin S81) | Après (T) | Delta |
|---|---|---|---|
| Rust nextest Win | 2095 | **2108** (run T : 2108/2108, 0 skip) | +13 (A +2, B +1, C +0, D +1, M +9 ; splits N→S4 **±0 EXACT**) |
| Rust nextest Docker | 2099 | **2112** (run T : 2112, 2 flakes famille sigint re-runs solo PASS) | +13 |
| Vitest web | 412 | **412** (run T) | 0 |
| E2E web | 44/2skip | **44 passed / 2 skipped, EXIT=0** (run T) | 0 |
| Vitest operator | 201 | **201** (run T) | 0 |
| E2E operator | 10 | **10/10** (run T) | 0 |
| Coverage web | ≥85/85/78/85 | 87.27 / 79.01 / 86.02 / 88.59 | seuils tenus |
| size-limit | 6/6 | 6/6 (css 129.02/130 kB) | tenu |
| `http.rs` | 12460 l (fin S81 `8b3590c`) | **1513 l** | **−10947 sprint (PO-10 PASS ; série split N→S4 : 13130 [pic post-M] → 1513 = −11617)** |
| fmt/clippy/doctests/release | verts | verts (Win + Docker) | 0 |

## 6. Surface nouvelle livrée (Phase T — docs/ops uniquement, 0 code produit)

- `docs/sharding/llms.txt` : +2 entrées index request-bodies ; re-route résiduels.
- `docs/sharding/REFERENCE.md` : +5 rows Types + note schématisation + re-route seuils.
- `docs/sharding/{README,EXPLANATION,HOW_TO_WIRE}.md`, `WIRING_SPEC.md`,
  `docs/protocol/SHARD_PROTOCOL_SPEC.md`, `examples/sign_verify.rs` : re-route
  honnêteté « routed S82 » → « différé par S82 (D6) vers le slot rig-chaud »
  (8 fichiers, tokens honesty-gate conservés : « S82 », « S82-pending tuning »,
  « admission ≠ confidentialité »).
- `scripts/check-sharding-docs.sh` : +9 anchors ROW/ENTRY-level (llms.txt
  entrée+snapshots, REFERENCE rows de table, HOW_TO_WIRE renvoi) — critère
  machine rendu discriminant à la SUPPRESSION de l'entrée indexée, pas
  seulement à la mention (durci suite Codex round 1 ; shellcheck local
  ABSENT, joué par le workflow GHA shellcheck au push).
- `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` : verrou D7 front-matter
  + chapeau §3 + micro-fix ancre.
- `docs/rust/PATTERNS.md` : **§P75** golden-characterization (le numéro §P74
  proposé par M-6 était DÉJÀ PRIS par `docs/shell/PATTERNS.md` Phase A —
  espace de numérotation commun).
- `docs/security/EXTERNAL_AUDIT_SCOPE.md` : frost-ed25519 2.1→3.0 (lock
  3.0.0) + chemin canary re-pointé `nexus-shell-daemon-core/src/canary/` +
  dalek 2.1→2.2.
- `docs/claude/TOOLING.md:291` : pointeur pourri `http.rs:483-494` re-pointé
  `publish_api.rs` (classe S2-F2, doc STAY hors gates).
- `CLAUDE.md`, `docs/claude/SPRINT_LOG.md`, `.planning/roadmap_v5_*.md` :
  réconciliation + row 82 + bloc S82 DONE.
- `.planning/active/` : `sprint82_phase_t_preflight.md`,
  `sprint82_t2_store_migration.json`, ce fichier, `sprint83_audit_plan.md`,
  `sprint82_t2_acceptance.json`.

## 7. Ce que le sprint n'a PAS livré (scope cuts respectés — kickoff §Out)

- ❌ Sharding feature/hardening (R-J-6 per-worker proofs + binding N0-N3, F2
  KV-cache, SI-12 TOCTOU, SHARD-TRUST-RECALIB, métriques cluster) → slot
  rig-chaud (D6).
- ❌ Fixes robustesse sharding bon-marché (J1b-3, D3-2, D4-2, J-D5-1) —
  dette d'audit hors-thème, non codée.
- ❌ Reprise arc front parqué `wip/factory-front-arc-post-s82` (87 fichiers,
  review+Codex groupés DUS, rebase conflit `provider_router.rs`) — POST-S82
  (PO-6).
- ❌ app-authoring S79 in-vivo `Not evidenced` — carry P1 STANDING OUVERT
  (ne PAS déclarer éteint ; distinct du carry sharding CLOSED S81).
- ❌ workflow-engine + Viewer fondation — DÉCALÉS (C9/PO-9), staging
  SUPERSEDED (Phase E), non codés ; tracés roadmap v5 (Phase T).
- ❌ Split fichiers secondaires >2000 l (shard_session.rs, iroh_runtime.rs,
  engine/runtime.rs, coordinator db.rs, public_feed.rs) — différé.
- ❌ Long tail split http.rs — **SUPERSEDED PO-10 : LIVRÉ in-sprint (S2→S4),
  0 carry « split différé »** (le scope cut d'origine n'existe plus).
- ❌ Tickets hors-thème (D10) : T20-wire, T21, T23 Docker@sha256, T25 FIPS,
  T26 Argon2id, T27 rpassword, nginx-DRY, firewall — statués au ledger
  Phase E, non codés (EXCEPTION : hickory IN, PO-7=A, Phase K).
- ❌ Veilles supply-chain standing — re-datées seulement (Phase E/K).
- ❌ Collapse-sites clippy MSRV — déjà résolus S81 B (clippy vert re-confirmé).
- ❌ Magic-number sweep comme phase dédiée — scope-cut nommé.
- ❌ Tagging exhaustif ~22 familles `DOMAIN_*_V1` + LOOPBACK §3 exhaustif —
  remplacés par accept-and-close incrémental (D8) + représentatif verrouillé
  (D7, verrou posé Phase T).
- ❌ Topologie A-vs-B — re-décision calendaire HORS-S82 (PO-5, due avant
  25/08 ; croise gate n0 15/09, EOL relais 30/09). 0 travail S82.

## 8. Findings carry-over for memory (G6)

1. **PIÈGE §P-numérotation** : l'espace `§P{n}` est COMMUN à
   `docs/rust/PATTERNS.md` et `docs/shell/PATTERNS.md` — toujours grep les
   DEUX fichiers avant de minter un numéro (collision §P74 évitée Phase T).
2. **Gate machine non-discriminant** : un critère « gate exit 0 » ne prouve
   un livrable docs QUE si le gate porte des anchors sur les surfaces
   éditées — vérifier la discrimination pré/post AVANT de citer le gate
   comme preuve (constaté Phase T : 3 gates verts sans l'index).
3. **Gate push PO-4=C bi-temporel** : « pousser sur 3 verts » est
   matériellement impossible à la lettre (nightly inexistant sur origin,
   Woodpecker gelé — auth Codeberg cassée, token non-vide masqué mais push
   en échec d'auth) — rust-ci pré-push via branche staging, Woodpecker +
   nightly post-push fix-forward ; classe watcher macos-14 (10 tests
   uniques, 2 runs observés : 28661119376 + 28592686238) = risque n°1 du
   gate, jamais traité.
4. **TEST-ISOLATION state.json — CLASSE, ≥2 sites** : tout test bootant un
   `Engine` réel sans confiner `NEXUS_GRID_ROOT` écrit le state.json worker
   RÉEL — 2 pollueurs observés 2026-07-17 (`result_sync.rs:494` « rsync
   xnode » + `engine/runtime.rs:1956/:2085` « rate-limit test », réécriture
   14:28Z pendant les suites T) ; routé S83 comme classe ; ne jamais citer
   state.json comme preuve live.
5. **Exit par classe de store** : « migration vérifiée » se résout
   différemment par classe (standalone = sibling ; local-worker =
   recreate-fresh par construction, JAMAIS de sibling) — la claim uniforme
   t2_h:48 était fausse ; expliciter le mode par palier dans l'artefact.

## 9. Checkpoint de clôture

- [x] 24 phases A→T committées (23 + T ce commit), 1 commit/phase + 2 chores.
- [x] Critère machine PO-10 : `wc -l http.rs` = **1513** ≤ 2500 TOTAL.
- [x] Suites §7.4 complètes vertes (Win 2108 + Docker 2112 + web 412/44E2E +
  operator 201/10E2E + fmt/clippy/doctests/release + 3 gates docs ×2).
- [x] T2 : `sprint82_t2_store_migration.json` PASS + agrégat
  `sprint82_t2_acceptance.json` (paliers sanctionnés au preflight T).
- [x] Clôture docs-contrat §6.12 : frontière neuve S82 (3 request-bodies
  Phase G) indexée 4 surfaces + anchors discriminants ; **écart lettre-D8
  consigné** : aucun tag `// FRONTIER:` littéral sur les 3 DTOs — le pattern
  snapshot+prose (classe `ShardSessionResultView`) a été choisi Phase G car
  un tag littéral ferait échouer le gate ; adaptation SAINE, ratifiée ici.
- [x] `sprint83_audit_plan.md` écrit (routage P2/P3 des 24 phase-reviews).
- [x] Working tree : les 3 fichiers hors-phase PO (blueprint modifié +
  2 untracked `workflow_*`) EXCLUS du staging.
- [x] **Gate push groupé PO-4=C JOUÉ 2026-07-17** (séquence ratifiée PO puis
  arbitrée) : rust-ci runs 29598778694 + 29601766920 — fmt/clippy/windows/
  ubuntu VERTS (1er ubuntu GHA vert depuis 2026-05, fix GTK Phase C prouvé ;
  clippy réparé `dd857d9`, drift toolchain 1.97) ; **macos-14 rouge =
  ARBITRAGE PO leg non-bloquant** (classe watcher pré-existante, routée
  S83) ; **PUSH GROUPÉ : origin c899d54→dd857d9 (48 commits) + codeberg
  f4b4600→dd857d9** ; Woodpecker #38 : steps lourds TOUS VERTS
  (rust+frontend), step spdx-check rouge = glitch env runner (les 2 blobs
  flaggés PROUVÉS conformes — tags ligne 1 au commit, auth.rs inchangé
  depuis S16 et vert au #37 à contenu identique, pipeline re-testée à la
  main dans l'image bash:5 exacte = MATCH ; routé S83 P3) ;
  **integration-nightly run 29603133484 SUCCESS + junit lisible (10 tests /
  0 failure) → S81-J-2 CLOSED**. Sprint 82 FERMÉ.
