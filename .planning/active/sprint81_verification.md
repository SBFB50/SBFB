# Sprint 81 — Verification (wrap-up Phase K)

> Sprint de maintenance d'infrastructure **iroh 0.98 → =1.0.1**, DONE
> **BI-AXE** (décision PO C1 2026-07-02) : TRANSPORT-convergence ET
> SHARDING re-cert live (ex-S78 ABSORBÉ). 15 phases 0/A/A2/A3/A4/B→K.

## 1. HEAD entrée / HEAD sortie

- **Entrée** : `61412bb` (kickoff + plan + design_review actifs, archive S80).
- **Phase 0** : audit gate S80 joué AVANT l'activation (findings `dcc3eea`,
  P1 S80-K-1 résolu `2c85b28`) — les 2 commits précèdent `61412bb`.
- **Sortie** : commit Phase K (ce commit).

## 2. Commit stack (fenêtre S81)

| Phase | Commit | Titre court |
|---|---|---|
| (0) | `2c85b28` + `dcc3eea` | audit gate S80 : P1 S80-K-1 + findings |
| kickoff | `61412bb` (+`868aab3` staging, +`b1f174e` vérif ultracode C8/C9/C10) | activation |
| A | `1e7188f` | materializer fold déterministe (wf4) |
| A2 | `23f3be8` | self-heal ×2 fail-fast diagnostiquable |
| A3 | `7d6b9ea` | baseline transport LIVE 0.98 committée + rig |
| A4 | `fdb8ad7` | boot sync-set coordinateur (`start_sync`) |
| B | `c899d54` | bump =1.0.1/=0.101.0/=0.101.0/=0.103.0 + MSRV 1.91 |
| C | `f70fa5f` | storage+feed sync-set au boot duress-gated |
| D | `7bd3578` | blobs 0.103 re-cert + contrat string BlobTicket |
| E | `efb9667` | surfaces transport re-cert + tripwire pkarr |
| E2 | `82afd0b` (+`a085853` palier live) | PLAN B C8 zéro-n0 Topologie B |
| E3 | `e05338f` (+`8872596` palier live) | hot-join gossip curateur souscrit |
| F | `70dd845` | migration redb 2→4 prouvée sur COPIE |
| G | `50f05c1` | supply-chain gate + docs sécurité v15 |
| H | `12e3954` (+`bd5d680` flip live) | runbooks + FLIP LIVE 3 nœuds PASS |
| I | `bb6c4f9` + `58cef6d` | orchestrateur session shard in-vivo (ex-S78) |
| J | `43623a5` | inférence réelle `sbfb/shard/1` + benchmark live PASS |
| K | (ce commit) | wrap-up bi-axe : binding + T1/T2 + docs + roadmap |
| process | `e7ff73c` (Codex 5.5→5.6 Sol) + `9c52cb7` (recherche PO) | mi-sprint |

## 3. How to re-run

```bash
# Bloc Rust Windows (natif)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Bloc Docker canonique (image sbfb-ci = rust:1.94 + libgtk-3-dev)
MSYS_NO_PATHCONV=1 docker run --rm \
  -e CARGO_TARGET_DIR=/workspace/target-linux \
  -e SBFB_AUTH_TOKEN=<64hex> -e SBFB_TEST_HTTP_TIMEOUT_SECS=120 \
  -v "C:\Users\FlowUP\Documents\Code\nexus:/workspace" -w /workspace \
  sbfb-ci bash -c 'export PATH=/usr/local/cargo/bin:$PATH; \
    cargo fmt --all --check && \
    cargo clippy --workspace --all-targets --locked -- -D warnings && \
    cargo nextest run --workspace --locked && \
    cargo test --workspace --locked --doc'

# Bloc web
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && npm run build && \
  npm run size && bash scripts/scan-en-strings.sh)

# Bloc operator
(cd tools/factory-operator && npm run lint && npm run test:unit && \
  npm run build && npm run size && npm run test:e2e)

# Supply-chain (4 catégories)
cargo deny check

# Gates docs
bash scripts/check-frontier-contracts.sh
bash scripts/check-sharding-docs.sh
bash scripts/check-factory-docs.sh

# Classe relay-gated (jamais comptée CI-verte) — nightly/manuel :
#   .github/workflows/integration-nightly.yml (cron 03:00 UTC + dispatch)
# ou localement : SBFB_INTEGRATION=1 cargo nextest run --locked \
#   -p nexus-test-harness -p nexus-coordinator-rs -E 'binary(multi_daemon)'

# Acceptance live (T2) :
bash scripts/acceptance/flip_convergence_check.sh   # convergence par nœud
REDUNDANCY=2 bash scripts/acceptance/b3_live_pc_vps.sh  # palier quorum
bash scripts/acceptance/b3_shard_pipeline.sh        # axe shard
```

## 4. Checklist (Observed) — livrables (a)..(l) du plan §K

- **(a) T1 hermétique 6 sous-tests BLOQUANT + CI chaque push** : Observed.
  Bloquants sur chaque push via `rust-ci.yml` (3 OS) + Woodpecker
  `ci-linux.yml`. **Mapping committé** (le livrable réel du préflight —
  consolidation, pas construction) :
  1. *Convergence in-process 2-nœuds* :
     `dispatch_loop::tests::convergence_*` (11 scénarios, groupe nextest
     `two-node-convergence`) + `blobs::tests::two_nodes_fetch_blob_via_ticket`
     + `docs::tests::two_nodes_sync_via_share_import` (les deux passés
     **direct-only** en K : relais strippé des addrs échangées —
     hermétiques loopback, immunes à l'EOL n0 30/09) + seed ALPN
     handshake (`seed_protocol` tests) + ingest annuaire
     (`node_directory`/browse tests).
  2. *Convergence ingest hors-ordre* :
     `feed_materializer::tests::test_out_of_order_ingest_converges_full`
     + `..._incremental` + `public_feed::tests::test_verify_chain_out_of_order_insertion`
     (l'assertion centrale du fix A).
  3. *Fixture migration redb 2→4 + self-heal non déclenché ×2* :
     `store_migration::docs_store_with_legacy_tuple_tags_migrates_on_open`
     (forge redb_v3 + contrôle TableTypeMismatch + idempotence) +
     `fresh_blobs_store_round_trips_across_reload` +
     `boot_{storage,feed}_namespace_refuses_recreate_on_interrupted_migration`
     + **NEUFS K** : `boot_{storage,feed}_namespace_reuses_existing_namespace_without_self_heal`
     (ferme le trou T1(3) hermétiquement : avant K, seul le gate
     empirique env-gaté `real_vps_store_copy_migrates_and_survives`
     — tarball gitignoré, skip CI — prouvait le non-déclenchement).
  4. *Parse tickets persistés* :
     `docs::tests::doc_ticket_string_round_trips_under_current_lock` +
     `blobs::tests::blob_ticket_string_round_trips_under_current_lock`.
  5. *Handshake shard in-process* : `shard::tests::shard_alpn_registered_in_router`
     + `shard_handshake_admits_member` + `shard_handshake_rejects_non_member`
     + `shard_frame_roundtrip_two_nodes`.
  6. *Session shard in-process via l'orchestrateur* : `shard_session::tests::*`
     (mount/gates/drive/decode, 30+ tests dont les 3 fail-closed
     attestation K).
  **Libellé corrigé (livrable e)** : la classe relay-gated
  (`multi_daemon`, 2 crates) s'auto-skippe verte sans `SBFB_INTEGRATION=1`
  — elle n'est JAMAIS comptée comme couverture CI (headers des 2 fichiers
  corrigés : le commentaire `#[ignore]` était périmé, gating harmonisé
  `== "1"` des deux côtés). Couverture réelle = job
  **`integration-nightly.yml`** (cron 03:00 UTC + workflow_dispatch,
  calqué sur le précédent supply-chain.yml) + T2 live. État connu de la
  classe : baseline A3 = 4/10 verts (5 test-rot + 1 signal produit) →
  réparation routée `sprint82_audit_plan.md`.
- **(b) T2 JSON committé BI-AXE** : Observed —
  `sprint81_t2_acceptance.json` (agrégat unique forme S80 référençant les
  8 paliers committés + les runs K).
- **(c) Re-jeu acceptances** : Observed avec les corrections factuelles
  du préflight (PLAN-ADAPT) :
  - **S75 survives-VPS-death REJOUÉ K (2026-07-11) : PASS** — baseline
    capturée (sbfb-explorer `d9660a95…`, sha256 `a8a57f51…`), VPS
    `systemctl stop` → dev : browse 5/5 intact, `blob-serve/index.html`
    **byte-identique**, node_id assert fail-closed ; VPS restart → actif,
    santé ok, browse 4/4 reachable, 0 crash-loop (2e boot 1.0.1
    post-migration).
  - **b3 fetch blob cross-machine : équivalence ACTÉE** — couvert de
    facto par le flip H (paliers `flip_cross_postvps_dev`/`_mac` : blob
    sha256 byte-identique cross-machine post-upgrade, 6/6 PASS).
  - **Convergence `PublicRegistryView` cross-nœud LIVE : couverture
    COMPOSÉE actée** — la comparaison de vue matérialisée cross-nœud est
    inexécutable telle quelle (materializer 0 consommateur runtime, scope
    cut A `1e7188f` ; `GET /api/daemon/feed/entries` expose le feed
    PROPRE du nœud uniquement — vérifié K : authors disjoints dev/VPS,
    comportement attendu). Preuve composée : T1(2) hermétique (fold
    déterministe, vue identique quel que soit l'ordre — CI chaque push)
    + `post_flip_directory_convergence_vps_side` PASS (H) + chaînes feed
    servies post-migration des deux côtés (probe K : dev 33 entrées seq
    1→33, VPS 10 entrées seq 1→10, chaînes saines).
  - **S76 b3 quorum (`b3_p2_quorum`) : PASS 2026-07-11 — 1er quorum de
    l'histoire du projet (C10)**. Quorum redundancy=2 déterministe formé
    en **6s end-to-end** (budget 30s) : tâche soumise au VPS, exécutée
    par 2 identités DISTINCTES (PC RTX 5080 `a424d8e748` à +1s + Mac M2
    `81cfeab05c` à +4s, Ollama llama3.1:8b des deux côtés), 2 résultats
    byte-identiques validés, `result_text` répliqué over WAN.
    Provenance : le raw committé ne porte que `delay_s=6` + task_id ;
    les attributions per-worker (identités, +1s/+4s) sont
    operator-corroborated depuis des logs rig NON committés — contrat
    de provenance explicite dans la note de l'agrégat. Séquence
    honnête 3 runs dans l'agrégat : BLOCK consent-opt-in (valide le fix
    A3/A4 : la tâche ATTEINT la réplique worker over WAN) → geste
    opérateur `ConsentLevel::All(4)` sur les 2 hôtes (opt-in S76) →
    BLOCK stage=claim (worker PC boot-froid 3s avant submit rate
    l'entrée incrémentale ; cette même tâche converge une fois le worker
    stable, +2m08 — signal produit routé `sprint82_audit_plan.md`) →
    PASS 6s les 2 workers en régime établi. Raw embarqué dans l'agrégat
    + `scripts/acceptance/.b3_quorum_k.json`.
- **(d) Clôture docs-contrat (invariant #17)** : Observed. Inventaire
  TOUTE la fenêtre S81 (leçon S80-K-1) :
  - **W1 wire** `ShardStepRequest`/`ShardStepReply` + `SHARD_STEP_PAYLOAD_V`
    (Phase J) + `ShardStageAttestationRequest`/`ShardStageAttestation` +
    `SHARD_ATTEST_PAYLOAD_V` (Phase K) → indexés
    `SHARD_PROTOCOL_SPEC.md` §5.1/§5.2 ; drift-gate `spec_consts_exist`
    ÉTENDU (types + guards + kinds — la classe de drift exacte qui venait
    de se produire).
  - **L1/L2 loopback** : 6 lignes `/api/daemon/shard-session/*` indexées
    `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §3 (`last_validated` 2026-07-11,
    trigger re-fired post-G par Phase I) ; routes write/drive spécifiées
    `SHARD_PROTOCOL_SPEC.md` §6.
  - **Lot doc-stale docs/sharding** (couche intouchée pendant I/J) :
    `llms.txt` (statut LIVE-PROVEN + shard_session.rs indexé),
    `WIRING_SPEC.md` (statut, RUN-PROOF driver-live, référence pendante
    `live_shard_session` RÉPARÉE → registre réel).
  - **N-A justifiés** : `GossipCmd::JoinPeers` (mpsc in-process, aucun
    acteur externe) ; runbooks release (acteur = opérateur humain, livrés
    in-phase H/E2) ; migration redb = borne pre-launch tracée roadmap v5
    (store on-disk ≠ wire) ; **O4 CLI `shard-session serve|plan|identity`
    = N-A outil-local explicite** (acteur = l'opérateur humain sur la
    machine, pas un runtime distinct ; la frontière machine est l'API
    loopback, déjà indexée). Faits négatifs (scan S4) : les 5 routes
    shard-session = les SEULES routes HTTP ajoutées sur toute la
    fenêtre ; seules constantes env neuves = `ZERO_N0_ENV` +
    `ZERO_N0_PKARR_RELAYS_ENV` (documentées in-phase
    `IROH_SELFHOST_OPS.md`). Dette tracée S82 : schémas JSON des corps
    de requête shard-session (réponses schématisées, requêtes non).
  - LOT-LOOPBACK-DOC soldé en G : VÉRIFIÉ K (portée G intacte,
    `last_validated` précédent 2026-07-08 conservé en historique).
  - `check-frontier-contracts.sh` : PASS.
- **(e) Libellé T1 + job SBFB_INTEGRATION** : Observed (cf. (a)).
- **(f) Binding loaded-stage↔manifeste signé (carry P1 Phase J)** :
  Observed — **CLOSED**. Attestation applicative à l'établissement de
  CHAQUE stage-link d'une session réelle (chokepoint unique de tout frame
  de données : 1er drive, re-dial, fallback ; digest peer-contrôlé
  strictement validé 64-hex-lowercase au décodage + accès `get(..16)`
  non-panic dans les diagnostics — durcissement review S81-K-R-1, sous
  `panic="abort"` un slice hors char-boundary aurait été un kill daemon
  distant — micro-adaptation assumée vs
  la lettre du préflight « mount barrier » : le mount reste 0-frame
  [R-I-1 préservé, contrat one-bi-stream-per-connection], l'invariant de
  sécurité « aucun step frame vers un exec non attesté » est tenu
  STRICTEMENT plus largement puisque les re-dials des drives ultérieurs
  sont couverts aussi). Fail-closed sur digest/fenêtre/rôles vs manifeste
  signé + `ShardAssignment` ; interception protocole AVANT le forwarder ;
  echo byte-inchangé ; digest serve = `blake3_hash_file` STREAMING
  (jamais `std::fs::read` 16 Go). Self-claim N0 : ferme la
  MISCONFIGURATION, byzantin = SI-4 résiduel (THREAT v17).
- **(g) THREAT_MODEL v17** : Observed — sweep GLOBAL des 15 refs S78
  (vivantes requalifiées « LIVRÉ S81 I/J/K » ou « re-routé S82 » ;
  historiques v12-v14 verbatim) + §16 « Attestation loaded-stage +
  certification SI-9 » (périmètre honnête : hermétique + live post-drive,
  drop mid-decode live = 3e machine R-J-5) + note N/A motivée trigger
  GUARDRAILS (`sanitize_diagnostic` = hygiène diagnostics transport, pas
  un checker de task-output) + carry J-D5-1 conn_type → label agrégat.
- **(h) Roadmap v5 amendée** : Observed — insertion S81-iroh bi-axe +
  ex-S78 ABSORBÉ + Viewer→S82 + borne pre-launch store on-disk.
- **(i) SPRINT_LOG row 81 + CLAUDE.md + memories + PATTERNS** : Observed
  (§P73 : 4 patterns S81 — in-frame payloads, two-node sans relais
  externe + groupe nextest, seam env→plan pur, set-lu-1×-au-boot).
- **(j) `sprint82_audit_plan.md`** : Observed — 11 tracks + escalade
  BLOQUANTE boot-SEED OVERDUE 3/3 + supply-chain (P2-AUDIT-2-RESIDUEL +
  fait neuf ed25519-dalek 3.0.0 stable + RUSTSEC-2026-0185 note +
  yanked="deny" parade) + tous P2/P3 des 15 phase-reviews.
- **(k) Arbitrages PO** : présentés au wrap-up (fin de session K),
  NON tranchés dans les docs — C9 slot S82 BLOQUANT (roadmap v5 le note
  « à RE-CONFIRMER PO », jamais au passé) ; Phase L benchmarks standards ;
  push groupé (18+ commits, origin=`c899d54`). Rappels d'échéance :
  Topologie A-vs-B avant 25/08 ; gate calendaire C8 15/09.
- **(l) Pipeline fail-fast 3 blocs + deny complet** : Observed — §5.
  **Note d'honnêteté supply-chain** (corrige la moitié du claim G
  « cargo-deny / cargo-audit verts ») : `cargo audit` N'EST PAS installé
  sur la machine dev — le vert supply-chain est `cargo deny check`
  (4 catégories, graph-aware). RUSTSEC-2026-0185 (quinn-proto <0.11.15,
  HIGH, DoS OOM) n'est PAS trackée dans le repo : le deny vert est
  LÉGITIME (chemin unique = reqwest→quinn http3 OPTIONNELLE hors graphe
  résolu ; iroh 1.0 = fork `noq`) mais un scanner lock-based la
  flaggerait — résiduel borné vérifié, routé `sprint82_audit_plan.md`.
  Veille pré-push : iroh **1.0.2 existe** (2026-07-06, bug-fixes,
  0 breaking) — pin `=1.0.1` gardé PAR CHOIX documenté (`Cargo.toml:33-46`,
  1.0.2 épingle encore ed25519-dalek rc) ; re-check au push groupé
  (gate D1/C3). Trigger veille iroh-docs 0.102+ : PAS fired.

## 5. Métriques sprint

- Rust nextest Windows : **2014 (entrée) → 2084 (fin J) → 2095 (fin K)**,
  0 skip, run observé 2026-07-11 `2095/2095 passed` — delta K = **+11**
  (crypto `blake3_hash_file` +1 ; shard core attestation
  codecs/verify/two-node +3 ; daemon fail-closed ×3 + self-heal reuse ×2
  = +5 ; **+2 fixes review/Codex** : validation digest anti-panic
  [S81-K-R-1] + collision echo anti-swallow [Codex P1]). Delta sprint
  = **+81**.
- Rust nextest Docker canonique (sbfb-ci rust:1.94) : total **2018 → 2099**
  (miroir +81, +4 `#[cfg(unix)]`). Run observé 2026-07-11 :
  `2091 passed, 6 failed` — les 6 = EXACTEMENT la classe iroh-networked
  env-bloquée Docker-on-Windows documentée (CLAUDE.md + memory
  dual-platform : `multi_daemon ×4` + `cross_daemon_blob` +
  `blob_serve_coep`, daemon-spawn timeout 30s réseau hôte dégradé CE
  run ; ces 6 tests sont VERTS dans le run Win natif 2093 et au CI
  Linux réel ; le run J de la veille les avait verts in-Docker —
  l'état réseau Docker-on-Windows est instable, pas le code).
- Doctests : OK (Win + Docker). fmt 0 + clippy -D warnings 0 (2 plateformes ;
  1 violation fmt du code K attrapée par le gate dual-platform et corrigée
  AVANT commit).
- Vitest web : **411 → 412** (fenêtre S81, +1 duress) ; run à vide
  2026-07-11 `412/412 passed` ; sous contention 5-jobs, 2 tests
  `AddAnchorDialog` ont flaké (reproductible chargé, vert solo ET à
  vide — classe memory `vitest_env_variance`, pas une régression) ;
  coverage verte.
- Vitest operator : **201** (inchangé) ; E2E Playwright operator : **10**
  (inchangé) ; size-limit web 6/6 + operator verts.
- `cargo deny check` : advisories/bans/licenses/sources **4/4 ok**.
- Gates docs : `check-frontier-contracts.sh` + `check-sharding-docs.sh` +
  `check-factory-docs.sh` PASS.
- **0 bump wire SBFB** : 30 constantes `*VERSION*` byte-identiques aux
  2 bornes, sets `DOMAIN_*_V1` identiques, 2 seuls ALPN inchangés
  (`sbfb/seed/0`, `sbfb/shard/1`), `KNOWN_OP_TYPES` = 5 ops. Seules
  constantes applicatives neuves (pas wire) : `SHARD_STEP_PAYLOAD_V` (J),
  `SHARD_ATTEST_PAYLOAD_V` + kinds (K).
- **0 dépendance runtime ajoutée** (dev-deps seules : iroh test-utils,
  redb 4.1/redb_v3 3.1, rusqlite — fixtures F).

## 6. Surface nouvelle livrée (modules dominants)

- `crates/nexus-shell-daemon/src/shard_session.rs` (NEUF I/J/K) —
  orchestrateur in-vivo complet + attestation.
- `crates/nexus-core-rs/src/discovery_override.rs` (NEUF E2) — seam
  zéro-n0.
- `crates/nexus-core-rs/tests/store_migration.rs` (NEUF F) — preuves redb.
- `docs/release/{LIVE_FLIP_RUNBOOK,STORE_MIGRATION_OPS,IROH_SELFHOST_OPS}.md`
  (NEUFS H/E2).
- `.github/workflows/integration-nightly.yml` (NEUF K).
- Le reste = migration/re-cert de surfaces existantes (0.98→1.0.1) +
  fixes root-cause (A/A2/A4/C/E3).

## 7. Ce que le sprint n'a PAS livré (scope cuts §Out kickoff, respectés)

- Viewer fondation + Aperçu scellé/Proof Card → S82.
- Dette docs-contract 8 P2/11 P3 S80 → sprint dette nommé.
- P1 in-vivo app-authoring S79 → standing (hors corps S81).
- GuardianDB / autre upgrade → séparé postérieur (bisectabilité).
- Bump MSRV 1.95 → JAMAIS fait (D6 : 1.91 déclaré, toolchain 1.94).
- Pagination app-storage / features produit → backlog.
- Clôture P2-AUDIT-2 → NON pré-annoncée : **P2-AUDIT-2-RESIDUEL carry
  S82** (lock non convergent, `deny.toml` multiple-versions = warn ;
  fait neuf : ed25519-dalek 3.0.0 STABLE 2026-07-06 → déblocage possible
  à instruire S82).
- RunProofs per-worker / arbitrage litige / SI-5 padding / re-calibration
  → S82 (tracés THREAT v17).

## 8. Findings carry-over for memory (G6)

- L'attestation stage-link est le NOUVEAU chokepoint sécurité du chemin
  shard réel — toute évolution du drive doit préserver « aucun step frame
  vers un exec non attesté ».
- La classe relay-gated a un état connu 4/10 (A3) — le nightly l'expose ;
  ne jamais compter un run par défaut comme couverture réseau.
- `yanked = "deny"` + pins exacts = casse CI mécanique possible sans
  commit SBFB (parade documentée dans sprint82_audit_plan).
- Le Mac M2 fait partie du rig T2 nominal : sa disponibilité conditionne
  le palier quorum + tout re-jeu shard.

## 9. Checkpoint de clôture

- [x] T1 6 sous-tests mappés, BLOQUANTS, verts (Win + CI Linux).
- [x] T2 agrégat bi-axe committé, vocabulaire fermé.
- [x] Carry P1 sharding S77 : CLOSED (J). Carry P1 binding (J) : CLOSED (K).
- [x] THREAT v17 (0 ref S78 vivante non requalifiée), LOOPBACK §3 à jour,
  SPEC §5/§6 à jour, docs/sharding requalifiés.
- [x] Roadmap v5 + SPRINT_LOG + CLAUDE.md + PATTERNS §P73 + memories.
- [x] `sprint82_audit_plan.md` écrit (audit gate S81 = Phase 0 S82).
- [x] Pipeline fail-fast 3 blocs + deny 4/4 + gates docs verts.
- [ ] Arbitrages PO à acter (C9 slot S82 ; Phase L ; push groupé) —
  DEMANDÉS au wrap-up, jamais tranchés ici.
