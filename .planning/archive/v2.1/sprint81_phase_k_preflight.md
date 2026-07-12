# Preflight S81 Phase K — Wrap-up bi-axe (T1/T2, binding manifeste, docs-contrat, arbitrages PO)

## Verdict: PLAN-ADAPT

- **Phase** : K (Sprint 81, Cas B) — wrap-up : livrables (a)..(l) du plan §K (T1 hermétique 6 sous-tests
  BLOQUANT, T2 JSON bi-axe, re-jeu acceptances S75/S76, clôture docs-contrat #17, libellé T1, carry P1
  binding loaded-stage↔manifeste, certif SI-9, roadmap v5, SPRINT_LOG/CLAUDE.md/memories/PATTERNS,
  sprint82_audit_plan, arbitrages PO, pipeline fail-fast).
- **Date** : 2026-07-11. HEAD = `43623a5` (Phase J). Fenêtre S81 = `61412bb..43623a5` (23 commits,
  17 non poussés, origin/master = `c899d54`).
- **Méthode** : Workflow fan-out 5 scans (S1a SOTA/OSS delta, S1b deps/supply-chain local, S2 décisions
  historiques + collecteur carries, S3 threat model + carry binding, S4 wire/frontières invariant #17)
  + vérification adversariale par-scan (claims REFUTED écartés de cette synthèse ; OMISSIONS décisives
  intégrées à la work-list).

Le but de Phase K ne heurte AUCUNE Day-0 ni décision PO gelée (0 bump wire PROUVÉ aux deux bornes,
iroh strictement seul, 0 dep runtime sur la fenêtre, upgrade != Gate 1, R-iroh-audit P0 inchangé).
Mais le **snapshot du plan §K doit être corrigé par des faits établis** sur au moins 7 points (détail
§Amendements) : §16 THREAT_MODEL existe déjà (update v17 + sweep 15 refs S78, pas une rédaction) ;
le palier « b3 fetch blob cross-machine » est couvert de facto par le flip H byte-identique ; la
« convergence PublicRegistryView LIVE » telle qu'écrite est inexécutable (materializer 0 consommateur
runtime, scope cut Phase A) ; le binding manifeste exige un **message applicatif NOUVEAU** sur
`sbfb/shard/1` (readiness actuelle 0-frame ; 0-wire par précédent J, mais du code neuf, pas un check) ;
la forme T2 = agrégat unique bi-axe (précédent S80) référençant 8 paliers existants ; les 6 sous-tests
T1 sont déjà majoritairement câblés et bloquants-CI (livrable = mapping + 2 trous, pas construction) ;
`sprint81_verification.md` est un livrable dû absent de la lettre du plan §K. Aucun de ces faits ne
contredit une décision PO → **PLAN-ADAPT**, pas DESIGN-CONFLICT.

## Rationale du verdict

Signaux : **S1a EXECUTE-avec-corrections** (1 claim REFUTED sur la lettre : RUSTSEC-2026-0185 quinn-proto
existe, juin 2026 — résiduel borné vérifié), **S1b EXECUTE** (9/9 confirmés, 0 dep, pins intacts),
**S2 PLAN-ADAPT** (18/18 confirmés + 3 omissions structurantes : verification.md, §16 existant, 6 tests
multi_daemon), **S3 PLAN-ADAPT** (trou binding confirmé au code ; 1 demi-claim REFUTED : le body J
CONTIENT le carry ; sweep S78 = 15 refs, pas 2), **S4 PLAN-ADAPT** (16/16 confirmés ; couche docs/sharding
INTOUCHÉE pendant que I/J changeaient la réalité ; référence pendante WIRING_SPEC:147).

### Réfutations arbitrées (exclues de la synthèse)

1. **S1a claim 5 REFUTED (lettre)** — « aucune advisory juin-juillet ne touche quinn-proto » est FAUX :
   RUSTSEC-2026-0185 (`date = 2026-06-22`, CVSS 7.5 HIGH, DoS OOM out-of-order) affecte quinn-proto
   <0.11.15 et le lock porte 0.11.14 (`Cargo.lock:6786-6788`). **Résiduel borné vérifié** : tiré
   UNIQUEMENT par reqwest 0.12.28 → quinn 0.11.9 (dep optionnelle http3 hors graphe feature-resolved),
   iroh 1.0 = fork noq ; `cargo deny check advisories` = « advisories ok » exécuté 2026-07-11 avec DB
   contenant 0185 (graph-aware, vert légitime). Passe en work-list comme item de TRAÇAGE (K-8/K-10),
   pas de remédiation code.
2. **S3 claim 4 demi-REFUTED** — le body `43623a5` CONTIENT le carry manifeste-binding en toutes lettres
   (lignes 143-144 + 165-174 « CARRY P1 NEUF … binding loaded-stage <-> manifeste signe … = Phase K +
   THREAT_MODEL §16 + audit gate S82 »). Le carry est DÉJÀ figé durablement ; l'action K (implémentation
   + v17 + routage S82) reste due, l'urgence « seul le review le porte » est écartée.
3. **Dates advisories S1a décalées +1 jour** (0205 = 07-06, 0203 = 07-05, 0202 = 07-04, 0200 = 07-03) —
   corrigées, sans incidence.
4. **Ancres dérivées corrigées** (substance intacte) : re-probes fallback = `shard_session.rs:1130` et
   `:1373` (pas :1138/:1376) ; note relais public n0 = `.config/nextest.toml:52-54` (pas :7-8) ;
   `HOP_DEADLINE_DEFAULT_MS` = `:132` ; hickory ignores = `deny.toml:83-86` ; ancre plan « deny.toml:107 »
   périmée → `deny.toml:150`.

## S1a — SOTA / veille (post-arbitrage) — corrections intégrées

- **iroh 1.0.2 existe** (crates.io max_stable, 2026-07-06 ; release GitHub = bug-fixes + 1 feature
  iroh-relay rate-limit live, 0 breaking). Pin `=1.0.1` (`Cargo.toml:48`) gardé PAR CHOIX documenté
  (`Cargo.toml:33-46`, rafraîchi G : 1.0.2 épingle encore ed25519-dalek =3.0.0-rc.0). **Aucun re-flip
  requis en K** ; re-check pré-push (gate D1/C3) au moment du push groupé.
- Trigger veille iroh-docs 0.102+ (`sprint81_plan.md:313-314`) **PAS fired** (max 0.101.0 = pin) ;
  iroh-gossip 0.101.0 / iroh-blobs 0.103.0 = max_stable = pins. Zéro delta.
- **RUSTSEC-2026-0185 NON-TRACKÉE dans le repo** (grep 0 hit ; `deny.toml` sans ignore) — vert deny
  légitime mais un `cargo audit` lock-based la flaggerait ; or `cargo audit` N'EST PAS installé et le
  claim G du plan (`sprint81_plan.md:314` « cargo-deny / cargo-audit verts ») est donc inexact sur la
  moitié cargo-audit → note d'honnêteté due en K (verification.md + sprint82_audit_plan).
- crossbeam-epoch 0.9.20 (`Cargo.lock:1370-1372`) = exactement la version patchée RUSTSEC-2026-0204 → no-op.
- **ed25519-dalek 3.0.0 STABLE publiée 2026-07-06** (non yanked ; rc.0 non yanked → trigger yank pas
  fired) — matière neuve pour le déblocage P2-AUDIT-2-RESIDUEL à router dans `sprint82_audit_plan.md`.
- **Job SBFB_INTEGRATION** : aucun workflow ne pose la variable (grep 0 match `.github/workflows/`) ;
  les tests relay-gated s'auto-skippent runtime (PAS `#[ignore]` — commentaire `multi_daemon.rs:9`
  périmé) et passent verts-en-skippant sur `rust-ci.yml:142`. Forme canonique minimale = workflow
  cron+dispatch calqué `supply-chain.yml:30-34` (2e précédent `canary-monthly.yml:55-61` ;
  `rust-ci.yml:34` a déjà `workflow_dispatch`) ; Woodpecker a `manual` mais 0 cron. L'argument « pas de
  précédent » n'est PAS disponible pour écarter le job. Gating asymétrique à harmoniser : harness
  `== "1"` (`nexus-test-harness/tests/multi_daemon.rs:14`) vs coordinator `.is_ok()`
  (`nexus-coordinator-rs/tests/multi_daemon.rs:7`).
- Benchmarks PO (memory `po_benchmarks_standards_llm_sharding.md:19-47`) : llama-bench pp512/tg128 +
  perplexity-parity wikitext-2 + artefact T2 versionné TTFT/TPOT/ITL ; arbitrage « Phase L S81 rig
  chaud vs S82 » à POSER au wrap-up ; répercussions canon (tier T3, README §4) AU MOMENT de la
  ratification, jamais mid-phase.

## S1b — Deps / supply-chain local — EXECUTE (9/9 confirmés)

- Pins intacts `Cargo.toml:48-51`, lock cohérent, `git log 50f05c1..HEAD -- Cargo.lock` VIDE (0 bump
  post-G). `cargo deny check advisories` = ok ; `bans` = ok avec exactement 72 `warning[duplicate]`
  (reproduit le body G).
- **0 dep runtime sur la fenêtre** : 5 fichiers manifest/config touchés seulement (3 Cargo.toml +
  `deny.toml` + `.config/nextest.toml`) ; ajouts = dev-deps nexus-core-rs (iroh test-utils, redb 4.1 +
  redb_v3 3.1, rusqlite) + `[features]` opt-in shell-daemon `default = []`. Diff JS strictement vide.
  Invariant TENU.
- `rust-version = "1.91"` (`Cargo.toml:24`, bump Phase B `c899d54`) ; 2 arbres ed25519-dalek
  (2.2.0 + 3.0.0-rc.0, chemin RC exclusivement iroh/iroh-base 1.0.1) ; `deny.toml:150`
  `multiple-versions = "warn"` + commentaire :141-149 = P2-AUDIT-2-RESIDUEL carry S82.
- **OMISSION intégrée — `deny.toml:45` `yanked = "deny"` ACTIF** : un yank d'ed25519-dalek 3.0.0-rc.0
  rend le pipeline advisories MÉCANIQUEMENT ROUGE (pin exact `=1.0.1` interdit toute résolution
  alternative) → mode de casse CI spontané à documenter avec parade dans `sprint82_audit_plan.md`.
- **OMISSION intégrée — `.config/nextest.toml` a bougé dans la fenêtre** (Phase C) : groupe
  `two-node-convergence` `max-threads = 2` + `slow-timeout = 60s` — le câblage T1 (K-2) doit citer et
  respecter ce mécanisme.
- **OMISSION intégrée** — le pipeline final K doit jouer `cargo deny check` COMPLET (4 catégories),
  pas seulement advisories+bans.

## S2 — Décisions historiques + carries — PLAN-ADAPT (18/18 confirmés)

- Fenêtre : 23 commits, delta tests **+70 Rust Win exactement** (2014→2084, somme des bodies re-calculée),
  Docker miroir +70 (2018→2088), Vitest web 411→412, operator 201, E2E 10 (standing S80, NON re-confirmé
  au body J — à re-mesurer au wrap-up). Trajectoire monotone, 0 suppression, écart vs plan « +14..28 »
  TRACÉ (préflights réviseurs C/I + arbitrage PO Option B J supersede documenté).
- **Carry P1 sharding S77 : CLOSED** (`43623a5` + `sprint81_t2_j_shard_inference.json` PASS) — NE PAS
  re-router. **Carry P1 NEUF** : binding loaded-stage↔manifeste (K-1).
- **T2 axe transport, état réel** : b3_p2_quorum **NOT-RUN** (`sprint81_t2_baseline_098.json` ; prérequis
  binaire worker arm64 Mac) ; S75 survives-VPS-death **AUCUN replay** fenêtre (scoping_note e2_zero_n0
  explicite) ; b3 fetch blob **couvert de facto** par flip H (blob sha256 byte-identique cross-machine,
  `sprint81_t2_h_live_flip.json` 6/6 PASS) → ACTER l'équivalence ou rejouer ; PublicRegistryView LIVE
  en TENSION avec « materializer 0 consommateur runtime » (`1e7188f` scope cut) → harness
  `materialize_full` sur 2 stores OU acter couverture T1(2)+directory-convergence-live.
- **T1** : les 6 sous-tests ont des candidats existants verts DÉJÀ bloquants-CI push (GHA
  `rust-ci.yml:141-145` + Woodpecker `ci-linux.yml:31`). Reste : mapping committé 6→tests-nommés ;
  trou (1) sous-test 1 éclaté 5 surfaces + 2 tests two_nodes dépendant du RELAIS PUBLIC N0
  (`.config/nextest.toml:52-54` — EOL 30/09) ; trou (3) « self-heal non déclenché ×2 » prouvé
  uniquement par le test empirique env-gaté (`store_migration.rs:314`, skip CI) → hermétiser.
- **6 tests multi_daemon concernés par la réparation/statut** (5 early-return + `test_cross_daemon_blob_transfer`
  gaté par assertion, `multi_daemon.rs:117-121`), pas 5 ; 1er run réel A3 = 4/10 verts, 5 test-rot +
  1 signal produit gossip.
- Forme T2 : précédent S80 = agrégat UNIQUE `.planning/archive/v2.1/sprint80_t2_acceptance.json`
  (clés `[suite,status,diagnosis,gates,scenarios]`) — K émet un agrégat bi-axe référençant les 8 paliers.
- **OMISSION intégrée — `sprint81_verification.md` ABSENT** de `.planning/active/` alors que S65→S80 en
  ont tous un : livrable K à part entière (K-7).
- **OMISSION intégrée — THREAT_MODEL §16 et §5.9 existent déjà** (`THREAT_MODEL.md:1199`, `:250`) :
  livrable (g) = UPDATE v17 in-place, pas création.
- Routages : escalade S75 boot-SEED OVERDUE 3/3 = audit gate S81 (Phase 0 S82, routée par `e05338f` +
  `50f05c1` + `8872596`) ; constats flip (`bd5d680`) seeder VPS catalog=0 + stores local-worker redb2 ;
  liste complète P2/P3 des 14 phase-reviews collectée (cf. contenu K-8).

## S3 — Threat model + binding manifeste — PLAN-ADAPT

- **THREAT_MODEL = v16** (S81 H, `:1648`), 1663 lignes, non touché par J → K écrit **v17**.
- **Trou de sécurité central CONFIRMÉ au code** : le chemin live `shard-session serve`
  (`crates/nexus-shell-daemon/src/main.rs:244-357`) choisit window/rôle via CLI et **ne voit jamais le
  manifeste** ; `authorize_claim` n'est appelé que depuis `runtime.rs:677` (chemin claim engine, mort
  sur serve) ; la readiness est transport-only (`probe_shard_readiness` `shard_session.rs:636`, 0 frame
  applicative, « deliberately NO probe frame ») et les re-probes fallback `:1130`/`:1373` non plus ne
  vérifient rien → un fallback mal-fenêtré passe et produit un résultat plausible-faux signé driver.
  Commentaire code actant le carry : `main.rs:336-337`.
- **Enforcement K-1 (design fixé par les faits)** : échange d'attestation applicative à la readiness
  (mount barrier `:730` + CHAQUE re-probe fallback) — le stage déclare `{model_digest, layer_start,
  layer_end, is_first, is_last}` du backend réellement chargé ; le driver compare à l'`ShardAssignment`
  signé + `manifest.model_digest` ; mismatch → **fail-closed** (même chemin que « no fallback » SI-9).
  Chemin echo (`model_digest == [0u8;32]`) exempté byte-inchangé. Digest côté serve = **blake3
  streaming** (interdit `std::fs::read` 16 Go, `main.rs:334-335`). Attestation = **self-claim d'un
  membre admis** (famille N0) : ferme la classe MISCONFIGURATION, pas le byzantin délibéré — caveat
  honnêteté v17.
- **Tension wire RÉSOLUE par précédent J** : message applicatif NOUVEAU dans les frames opaques
  `sbfb/shard/1` = 0 bump au sens du sprint (pattern `SHARD_STEP_PAYLOAD_V`, `shard.rs:379`) ; NE PAS
  étendre `ShardStepRequest` (`deny_unknown_fields` `:389`/`:434`) — message distinct.
- **Certif SI-9 v17** : (a) SI-9 data-plane ARMÉ S81 I/J (deadline par-hop open+write+read
  `:779-808`/`:975-1000`, mid-decode fallback+replay stateless `:1341-1385` ; write-path fermé
  `hop_deadline_bounds_the_write_path` `:2726`, pré-condition `sprint81_phase_i_review.md:261`
  satisfaite) avec périmètre HONNÊTE = hermétique + coupe live post-drive comptée SEULEMENT (drop
  mid-decode live = 3e machine rig absente, R-J-5, `t2_j.json:39`) ; (b) SI-9 N3-reveal (arbitrage
  litige in-vivo) reste carry → S82 ; (c) **sweep GLOBAL des 15 refs « S78 »** (pas 2) : `:145`
  (« orchestrateur PAS câblé » FAUX depuis I), `:259`/`:260` (RunProof in-vivo émis depuis I/J),
  `:1031`, `:1264` (fingerprint livré J), `:1483` (SI-9 livré I/J), `:1292`/`:1333`/`:1417`/`:1460`
  (re-nomenclature ex-S78 ABSORBÉ → « livré S81 » ou « re-routé S82 »).
- **Trigger LOOPBACK re-fired POST-solde G** : les 5 routes `/api/daemon/shard-session/*` de Phase I
  (`http.rs:323-336`, `bb6c4f9` 2026-07-10 > `50f05c1` 2026-07-08) déclenchent le trigger
  (`LOOPBACK_ENDPOINTS_TRUST_TIERS.md:7`) et le doc a 0 occurrence « shard » → inventaire §3 stale,
  action K due (LOT-LOOPBACK vérifié SOLDÉ pour la portée G, `last_validated: 2026-07-08`).
- **Trigger GUARDRAILS lecture stricte FIRE** : `sanitize_diagnostic` (`shard_session.rs:823`,
  `58cef6d`) = scrubber d'output ad-hoc hors GuardrailChain → trancher par une note N/A motivée d'une
  ligne (hygiène diagnostic transport, pas checker de task-output), pas par silence.
- Carries v17 additionnels : conn_type direct non machine-asserté au readiness-barrier (J-D5-1,
  `t2_j.json:36` « stays a CARRY for Phase K ») → livrer l'assertion ou labelliser ; RunProofs
  per-worker + binding N0-N3 (canal control-plane feed raw-op, R-J-6) → re-router S82 dans la v17.
- Aucun trigger CAPABILITY_TOGGLES / WARRANT_CANARY dans la fenêtre.
- Delta tests binding estimé : **+6..10 Rust** (3-4 codecs attestation core-rs + 3-5 shard_session
  mount/re-probe/echo + 1 digest streaming).

## S4 — Wire + frontières invariant #17 — PLAN-ADAPT (16/16 confirmés)

- **Invariant 0-bump SBFB TENU, prouvé aux DEUX bornes** : 30 constantes `*VERSION*` byte-identiques
  (seul delta = déplacement de ligne `tls_pinning.rs:102→111`) ; sets `DOMAIN_*_V1` diff exit 0 ;
  2 seuls ALPN inchangés (`sbfb/seed/0`, `sbfb/shard/1`) ; `KNOWN_OP_TYPES` mêmes 5 ops ;
  `SHARD_STEP_PAYLOAD_V` (`shard.rs:379`) = SEULE constante de versionnage neuve (additive, frames
  opaques inchangées). Le flag-day iroh 0.98→1.0.1 = wire EXTERNE.
- **Faits négatifs citables pour le N-A global** : les 5 routes shard-session = les SEULES routes HTTP
  ajoutées sur TOUTE la fenêtre (grep diff complet `crates/`) ; seules constantes env neuves =
  `ZERO_N0_ENV` + `ZERO_N0_PKARR_RELAYS_ENV` (documentées in-phase `IROH_SELFHOST_OPS.md`).
- **Frontières à indexer en K** :
  - **W1** `ShardStepRequest`/`ShardStepReply` + `SHARD_STEP_PAYLOAD_V` + champ `tokens` →
    `SHARD_PROTOCOL_SPEC.md` (0 hit actuellement ; carry explicite body J « J6-1/D4-3/J6-2 »).
  - **L1/L2** : 6 lignes `shard-session` dans LOOPBACK §3 (5 neuves Phase I + la GET S77 héritée,
    gap hérité) ; étiquette générée PARTIELLE (2 schémas réponse neufs + 2 régénérés `bb6c4f9` ;
    corps de requête `ShardGroupMintRequest`/`MountSessionRequest`/`ShardGenerateRequest` sans schéma).
  - **Lot doc-stale docs/sharding** (couche INTOUCHÉE dans la fenêtre, `git diff` = 0 octet) :
    `llms.txt:8` « PROVISIONAL » (carry S77 CLOSED depuis J) ; `WIRING_SPEC.md:29/:33` « until S78 » ;
    `:119` « RUN-PROOF PROVISIONAL/S78 » ; **`:147` = référence PENDANTE** (`live_shard_session`
    SUPPRIMÉ Phase I, `http.rs:2140`).
  - **Drift-gate** : `spec_consts_exist` (`schemas/shard.rs:514`) n'asserte que tags+caps+ALPN,
    aveugle aux types J → l'étendre (types + éventuellement résolution des source_refs WIRING_SPEC,
    exactement la classe de drift qui vient de se produire), sinon l'index re-pourrit.
  - N-A justifiés : `GossipCmd::JoinPeers` in-process ; O1/O2 runbooks ops (acteur humain, docs
    livrées in-phase) ; O3 migration redb = borne pre-launch → roadmap v5 (K-6) ; O4 CLI
    `serve`/`plan` → ligne index ou N-A outil-local explicite (à trancher en K).
- `check-frontier-contracts.sh` PASS exit 0 aujourd'hui, BLOQUANT 3 surfaces — mais AVEUGLE aux gaps
  ci-dessus (registre opt-in) : c'est l'audit Track K qui les attrape, d'où la clôture K-5.
- Phase A a tracé une frontière RÉELLE in-phase (PUBLIC_FEED_SPEC §6/§7), A2 un non-frontière motivé ;
  toutes les autres phases N-A — 0 frontière pendante hors shard.

## WORK-LIST PHASE K ORDONNÉE

### BLOC CODE

**K-1 — Binding loaded-stage↔manifeste signé (livrable f, CARRY P1 — prioritaire, débloque K-4)**
- Livrable : attestation applicative de readiness sur `sbfb/shard/1` — message NOUVEAU (pattern
  `SHARD_STEP_PAYLOAD_V`, jamais d'extension `ShardStepRequest`), stage déclare
  `{model_digest, layer_start, layer_end, is_first, is_last}` du backend chargé, driver compare à
  l'`ShardAssignment` signé + `manifest.model_digest`, mismatch fail-closed ; exigé au mount barrier
  ET à chaque re-probe fallback (`:1130`, `:1373`) ; chemin echo (digest zeros) exempté byte-inchangé ;
  digest serve = blake3 streaming.
- Fichiers : `crates/nexus-core-rs/src/shard.rs` (codec attestation + garde v + deny_unknown_fields),
  `crates/nexus-shell-daemon/src/shard_session.rs` (barrier `:730`, re-probes `:1130`/`:1373`),
  `crates/nexus-shell-daemon/src/main.rs` (serve `:244-357`).
- Delta tests attendu : **+6..10 Rust** (codecs roundtrip/v-guard/deny_unknown/rejet croisé ;
  mount rejette window-mismatch + digest-mismatch ; fallback re-probe rejette mal-fenêtré ; echo
  inchangé ; digest streaming).
- Gate : nextest Win + Docker verts ; 0 bump wire (message applicatif frames opaques, précédent J) ;
  clippy/fmt.

**K-2 — Câblage T1 + libellé + job SBFB_INTEGRATION (livrables a + e)**
- Livrable : mapping committé 6 sous-tests → tests nommés (sous-test 6 = consolidation
  `shard_session.rs` 23 tests + 3 decode J) ; trou (1) : statuer le sous-test 1 éclaté 5 surfaces +
  traiter la dépendance relais public N0 des 2 two_nodes (`.config/nextest.toml:52-54`, EOL 30/09) ;
  trou (3) : hermétiser « self-heal non déclenché ×2 » (aujourd'hui seul le test env-gaté
  `store_migration.rs:314` le prouve, skip CI) ; libellé hermétique-CI vs relay-gated-local corrigé
  (y c. commentaire `#[ignore]` périmé `multi_daemon.rs:9`) ; harmoniser le gating `== "1"` vs
  `.is_ok()` ; statuer/réparer les **6** tests multi_daemon (5 early-return + blob_transfer) ; job
  `SBFB_INTEGRATION=1` nightly/manuel calqué `supply-chain.yml:30-34` OU acter la couverture T2-live
  (les deux voies du plan restent ouvertes ; le précédent cron+dispatch EXISTE).
- Fichiers : `.github/workflows/` (nouveau workflow ou job), `crates/nexus-test-harness/tests/multi_daemon.rs`,
  `crates/nexus-coordinator-rs/tests/multi_daemon.rs`, `crates/nexus-core-rs/tests/store_migration.rs`,
  `.config/nextest.toml` (respecter le groupe `two-node-convergence`), doc libellé T1.
- Delta tests attendu : **+2..6 Rust** (hermétisation self-heal + éventuel test unifié) ; les réparations
  test-rot ne comptent pas comme net-new.
- Gate : T1 6/6 mappés verts BLOQUANTS chaque push (Win natif + CI Linux, JAMAIS Docker-on-Windows
  pour multi_daemon) ; total tests jamais en baisse silencieuse.

### BLOC ACCEPTANCE LIVE

**K-3 — Paliers T2 restants + agrégat BI-AXE (livrables b + c)**
- Livrable : (1) `b3_p2_quorum` — 1er PASS de l'histoire visé (C10) ; prérequis binaire worker arm64
  Mac (source au HEAD via bundle/scp) ; (2) replay S75 survives-VPS-death (aucun artefact fenêtre) ;
  (3) ACTER l'équivalence b3-fetch-blob ↔ flip H byte-identique (sha256 = baseline, 2 paliers
  cross-machine) OU rejouer le palier b3-lignée ; (4) PublicRegistryView : harness `materialize_full`
  sur les 2 stores OU acter la couverture T1(2) hermétique + `post_flip_directory_convergence_vps_side`
  PASS — trancher et DOCUMENTER dans l'artefact ; (5) agrégat UNIQUE bi-axe (forme S80 :
  `[suite,status,diagnosis,gates,scenarios]`) référençant les 8 paliers committés + les nouveaux runs ;
  (6) vérifier 0 préfixe pubkey 16-hex dans tout artefact committé (R-J-7) ; (7) carry J-D5-1
  conn_type : assertion machine au readiness-barrier OU label honnête dans l'agrégat.
- Fichiers : `.planning/active/sprint81_t2_acceptance.json` (agrégat), scripts acceptance existants,
  `rig.local.env`.
- Delta tests attendu : 0 Rust (acceptance) sauf assertion conn_type si câblée (+1).
- Gate : vocabulaire FERMÉ `PASS/BLOCK{diagnosis}/RIG-ABSENT` émis par le harness, jamais prose ;
  `RIG-ABSENT` légitime UNIQUEMENT si machine génuinement HS.

### BLOC DOCS

**K-4 — THREAT_MODEL v17 (livrable g) — UPDATE in-place, pas rédaction**
- Livrable : certif SI-9 scopée honnête (hermétique + coupe live post-drive comptée ; drop mid-decode
  live = 3e machine, R-J-5) ; section binding manifeste (self-claim N0, ferme MISCONFIGURATION) ;
  **sweep GLOBAL 15 refs S78** (`:145`, `:259`, `:260`, `:1031`, `:1264`, `:1483`, `:1292`, `:1333`,
  `:1417`, `:1460`…) ; SI-9 N3-reveal + RunProofs per-worker N0-N3 re-routés S82 ; note N/A motivée
  trigger GUARDRAILS (`sanitize_diagnostic`) ; entrée historique v17.
- Fichiers : `docs/security/THREAT_MODEL.md`. Gate : 0 ref S78 pendante non-requalifiée.

**K-5 — Clôture docs-contrat invariant #17 (livrable d)**
- Livrable : LOOPBACK §3 +6 lignes shard-session + bump `last_validated` (trigger re-fired post-G) ;
  `SHARD_PROTOCOL_SPEC.md` index `ShardStepRequest/Reply` + `SHARD_STEP_PAYLOAD_V` + champ `tokens` ;
  lot doc-stale docs/sharding (`llms.txt:8`, `WIRING_SPEC.md:29/:33/:119` + réparation référence
  pendante `:147`) ; extension `spec_consts_exist` (types J + option check source_refs) ; N-A global
  citant les faits négatifs S4 (5 routes seules, 2 env seules) ; trancher O4 CLI (index vs N-A
  outil-local) ; schémas corps de requête shard = livrer ou tracer en dette S82.
- Fichiers : `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`, `docs/protocol/SHARD_PROTOCOL_SPEC.md`,
  `docs/sharding/llms.txt`, `docs/sharding/WIRING_SPEC.md`, `crates/nexus-core-rs/src/schemas/shard.rs`.
- Delta : +0..2 Rust (extension drift-test). Gate : `check-frontier-contracts.sh` PASS + audit Track K
  satisfiable (frontières fenêtre indexées ou N-A explicite).

**K-6 — Amendement roadmap v5 (livrable h)** : insertion S81 bi-axe + Viewer→S82 + ex-S78 ABSORBÉ +
  borne pre-launch store on-disk (migration redb 2→4, O3). Fichier :
  `.planning/roadmap_v5_factory_complete_vision.md`.

**K-7 — `sprint81_verification.md` (OMISSION S2 — livrable standard dû)** : 9+ sections forme S80,
  état par livrable (a)..(l), delta tests final, invariants. Fichier : `.planning/active/`.

**K-8 — `sprint82_audit_plan.md` (livrable j)** — contenu collecté :
- Périmètre diff `61412bb..<tip S81>` (23+ commits, dont 4 chores acceptance + 2 chores process/research ;
  bascule Codex 5.5→5.6 mi-sprint `e7ff73c`).
- **Escalade Phase 0 BLOQUANTE** : S75 re-drive-on-ingest boot-SEED OVERDUE 3/3 — fermer ou re-justifier.
- Constats flip : seeder VPS catalog=0 one-shot ; stores local-worker encore redb2.
- Supply-chain : P2-AUDIT-2-RESIDUEL (+ fait neuf ed25519-dalek 3.0.0 STABLE sortie) ;
  HICKORY-024-RUSTSEC ; quick-xml 0194/0195 ; **RUSTSEC-2026-0185 quinn-proto** (non-trackée, résiduel
  borné reqwest-http3-only, note cargo-audit lock-based) ; **`yanked="deny"` = mode casse CI mécanique**
  si yank RC (parade documentée) ; G-D5-1 VALIDATED_BLUEPRINT « iroh 0.97 ».
- Carries P1 standing : binding manifeste (si résiduel post-K) ; app-authoring in-vivo `Not evidenced` ;
  sharding S77 = CLOSED, NE PAS re-router.
- Tous P2/P3 des 14 phase-reviews (règle kickoff) : A author→project_id, MAX_FEED_ENTRIES, BinaryHeap ;
  A2 import_ticket, per-app fail-fast ; A3/A4 P2-PROJECT-DOC-SELECTOR, keepalive NeighborDown ;
  C duress 45s, WS-3/PD-5 hoisting ; D BlobTicket sites, outbox stale, troncature-16 ; E doc-stale
  age_witness, MANQUE-3 ; E2 D2-2 ; E3 from_subscribed à chaud, unsubscribe asymétrie ; I TOCTOU
  202/202, resume mid-pipeline mono-étage, mappings HTTP fins, D4-1 RunProof projection ; J R-J-6
  RunProofs per-worker, F2 KV-cache, p95=moyenne, ttft_ms résolution 1s, fallback windows, 16-hex
  churn, path-type WAN, cold/warm ttft.
- Standing : Viewer fondation S82 candidat C9 ; 8 P2/11 P3 docs-contract S80 ; arc front parqué
  `wip/factory-front-arc-post-s82` (review+Codex groupés DUS) ; Tracks J/K standing ; externes
  (P2-A-1 rand, T-NN+2, P3-OS-1, LT-2 ARMÉ, R-iroh-audit P0 INCHANGÉ).

**K-9 — SPRINT_LOG row 81 + CLAUDE.md + memories + PATTERNS (livrable i)** : candidats PATTERNS =
  §P73 MSRV-lints (B), groupe two-node-convergence (C), seam décision-pure env→builder (E2),
  « set lu 1× au boot = mutation inerte » (E3) ; memories post-commit (nexus_grid_pivot + MEMORY.md).

### GATE FINAL

**K-10 — Pipeline fail-fast 3 blocs + honnêteté supply-chain + commit (livrable l)** :
  fmt/clippy/nextest/doctests/release Win + Docker canonique + web (lint/tsc/test/coverage/build/size/
  scan-en-strings) ; `cargo deny check` COMPLET 4 catégories ; note honnêteté « cargo-audit non
  installé + RUSTSEC-2026-0185 résiduel borné » (corrige la moitié du claim G) ; re-check crates.io/
  RUSTSEC pré-push (iroh 1.0.2 : pin gardé par choix, gate D1/C3) ; commit phase K body riche
  delta cumulé.

## DÉCISIONS PO À PRÉSENTER (DEMANDER, jamais trancher)

1. **C9 — slot S82 (BLOQUANT, double source kickoff `:83-85` + `b1f174e`)** : re-confirmer
   S82 = workflow-engine vs Viewer fondation vs dette docs-contract.
2. **Phase L benchmarks standards LLM/sharding** (memory PO 2026-07-10) : llama-bench pp512/tg128 +
   perplexity-parity wikitext-2 + artefact T2 versionné TTFT/TPOT/ITL — **Phase L S81 (rig chaud)
   vs report S82** ; si ratifié STANDING → amendements canon (tier T3, README §4, Track audit,
   invariant kickoff) AU MOMENT de la ratification.
3. **Push groupé LOCAL→origin** : 17 commits non poussés (origin = `c899d54` Phase B) — GO/attendre ;
   pré-push dû (veille crates.io + RUSTSEC code-freeze ; iroh 1.0.2 existe, pin `=1.0.1` par choix
   documenté — pas de re-flip proposé).
4. **Rappel d'échéance (pas à trancher en K)** : Topologie A-vs-B avant 25/08 (B déployée live
   `a085853`, re-décision explicitement OUVERTE `50f05c1` §15.4 + `bd5d680`).
5. **Rappel routage (info)** : escalade S75 boot-SEED OVERDUE 3/3 = audit gate S81 (Phase 0 de S82),
   pas une décision K ; si K ne ferme pas K-1 entièrement, escalade P1 à l'audit gate S82.
6. **Micro-arbitrages d'exécution K à restituer au fil de l'eau** (option par défaut proposée entre
   parenthèses) : équivalence b3-fetch-blob ↔ flip H (acter) ; PublicRegistryView harness-vs-acter
   (harness si coût faible, sinon acter documenté) ; job SBFB_INTEGRATION cron+dispatch vs couverture
   T2-live actée (job GHA calqué supply-chain.yml — précédent existant).

## Amendements au plan (§K) — constatés, aucun ne contredit une décision PO

1. **(g) « rédiger §16 »** → §16 EXISTE (`THREAT_MODEL.md:1199`, v16) : livrable = update v17 +
   sweep 15 refs S78 (le plan sous-estimait le périmètre du rafraîchissement).
2. **(c) palier « b3 fetch blob cross-machine »** → couvert de facto par flip H byte-identique
   (`sprint81_t2_h_live_flip.json`) : rejeu optionnel, équivalence à ACTER.
3. **(c) « convergence PublicRegistryView post-migration LIVE »** → inexécutable telle quelle
   (materializer 0 consommateur runtime, scope cut A `1e7188f`) : harness dédié OU couverture actée.
4. **(f) binding** → exige un message applicatif NOUVEAU sur `sbfb/shard/1` (readiness actuelle
   0-frame) : 0-wire par précédent J mais phase-code réelle (+6..10 tests), pas un simple check.
5. **(b) forme T2** → agrégat UNIQUE bi-axe (précédent S80) référençant 8 paliers, pas une collection
   de fichiers de plus.
6. **(a) T1** → les 6 sous-tests sont déjà majoritairement câblés ET bloquants-CI : livrable réel =
   mapping committé + 2 trous (sous-test 1 relais-N0, self-heal hermétique) + libellé.
7. **Ancre « deny.toml:107 » du plan périmée** (→ `:150`) ; claim G « cargo-audit vert » inexact
   (non installé) → note d'honnêteté.
8. **+ livrable implicite** : `sprint81_verification.md` (présent S65→S80, absent de la lettre §K).

## Risques résiduels

- **R-K-1 (MAJOR)** — b3_p2_quorum : 1er run de l'histoire, prérequis binaire arm64 Mac + Ollama ;
  toute défaillance soft sort en `BLOCK{diagnosis}`, jamais `RIG-ABSENT` (rig présent).
- **R-K-2 (MAJOR)** — S75 replay survives-VPS-death : dépend du VPS Hetzner live ; fenêtre morte
  1er boot connue (re-drive-on-ingest OVERDUE) peut produire un BLOCK légitime → diagnostic, pas
  de faux-vert.
- **R-K-3 (MAJOR)** — Binding K-1 : risque de sur-design (rester self-claim N0, ne PAS tenter le
  byzantin) et de régression echo-path (exemption byte-inchangée à locker par test).
- **R-K-4 (MINOR)** — Sweep S78 v17 : risque d'en oublier (15 refs listées, grep final `S78`
  = 0 pendante non-requalifiée en gate).
- **R-K-5 (MINOR)** — Extension `spec_consts_exist` : ne pas transformer le drift-test en
  spec-parser fragile ; assertions de présence simples.
- **R-K-6 (INFO)** — iroh 1.0.2 : aucun re-flip en K ; si le fix « Windows transient receive
  errors » devient pertinent, c'est une décision de bump S82+ (pin par choix documenté).

## Verdict final motivé

**PLAN-ADAPT.** Le but de Phase K (fermer S81 bi-axe : testabilité, acceptance, sécurité, docs-contrat,
canon) est intégralement exécutable et ne heurte AUCUNE Day-0 ni décision PO gelée — 0 bump wire prouvé
aux deux bornes, 0 dep runtime, iroh strictement seul, trajectoire tests monotone +70. Mais la lettre du
plan §K, écrite avant I/J, doit être corrigée par 8 faits établis (§Amendements) : §16 à mettre à jour
(pas rédiger) avec un sweep S78 ×15 ; deux paliers T2 du (c) déjà couverts ou inexécutables tels quels ;
le binding (f) = code applicatif neuf 0-wire-par-précédent (+6..10 tests) et non un simple contrôle ;
la forme T2 = agrégat unique ; le T1 = consolidation-mapping et non construction ; plus un livrable
standard manquant (verification.md) et deux corrections d'honnêteté (ancre deny.toml, claim cargo-audit).
Ces adaptations sont toutes DANS le mandat wrap-up de K et relèvent de l'exécution, sous réserve des
arbitrages PO listés (C9 BLOQUANT + Phase L + push groupé) qui sont à DEMANDER au wrap-up, comme le
plan le prévoit déjà.
