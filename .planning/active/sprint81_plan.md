# Sprint 81 — Plan : Upgrade iroh 0.98 → 1.0 + sweep deps (migration transport one-way + re-cert sharding)

> **ACTIVÉ le 2026-07-02** (Cas C ; Phase 0 = audit gate S80 JOUÉE : CONDITIONAL PASS →
> PASS effectif, findings `dcc3eea`, fix P1 `2c85b28`). Source unique :
> `sprint81_kickoff.md` (+ bloc **Décisions PO à l'activation 2026-07-02**, qui fait
> AUTORITÉ — intègre le registre de la **vérification ultracode 02/07**
> `verification_2026-07-02.md` : pin `=1.0.1`, auto-migration redb #105, self-heal ×2
> `:2518`/`:2606`, MSRV tranchée 1.91, phases A2/A3, C8/C9/C10) + dossier canonique S81
> (corrections sceptique intégrées : materializer `feed_materializer.rs:54-58`,
> 3 crates déclarent iroh). **14 phases : 0 + A, A2, A3, B→K.**

> Phases dimensionnées par le **travail**, JAMAIS par LOC. **Phase 0 = audit gate S80** (JOUÉE).
> S81 = **iroh STRICTEMENT SEUL** (bisectabilité ; materializer en Phase A commit séparé AVANT le
> bump ; tout le reste rerouté S82/dette — le sharding I/J n'est PAS un bundle : il re-certifie la
> stack migrée). 1 commit atomique par phase `feat(scope): Sprint 81 Phase X — titre` (ou
> `fix(...)`/`chore(...)` selon nature) ; **rigueur per-phase uniforme** : deep preflight (5 scans)
> → review Workflow → Codex avant CHAQUE commit ; T1 hermétique grandit incrémentalement (BLOQUANT
> au wrap-up + CI chaque push), T2 artefact JSON **committé** (BI-AXE). Migration on-disk redb 2→4 =
> **one-way** (rollback = restore tar).

> **Cadrage DONE (décision PO C1 à l'activation, 2026-07-02 — CONTRE la reco staging).** Le DONE
> non-PROVISIONAL de S81 est **BI-AXE** : **TRANSPORT-convergence** (doc-sync / gossip / blobs /
> seed / annuaire) ET **SHARDING re-cert LIVE** (`shard.rs` RTT/PathId multipath via
> l'orchestrateur de session in-vivo ex-S78 — Phase I le construit, Phase J joue le benchmark
> live b3_shard, le wrap-up devient Phase K). Verdict T2 axe shard au vocabulaire fermé émis par
> le harness préflight ; `RIG-ABSENT` légitime UNIQUEMENT si une machine (5080/M2) est génuinement
> HS — le rig nominal est le même matériel que l'axe transport.

---

## Phase 0 — Audit gate S80 : JOUÉE le 2026-07-02

- **Verdict** : **CONDITIONAL PASS → PASS effectif** (0 P0, 1 P1 résolu in-gate, 4 P2, 10 P3 ;
  Workflow ultracode 11 tracks + adversarial, Track I rejouée après stub).
- **Commits** : `2c85b28` (fix P1 S80-K-1 : 5e frontière docs-contrat
  `GET /api/project-documents` indexée) + `dcc3eea` (findings + fixes hygiène F-1/A-1/E-3/J-1).
- **Baseline tests FIGÉE** : Rust nextest **2014** Win natif 0-skip / **2018** Docker canonique
  (+4 `#[cfg(unix)]`) ; Vitest `web/` **411** (38 fichiers) ; Vitest operator **201** (35
  fichiers) ; E2E Playwright operator **10** ; doctests 6 ; size-limit operator 8/8.
- **Carries figés** (cf. §Carries kickoff + bloc Décisions PO à l'activation) : P1 in-vivo
  app-authoring S79 `Not evidenced` standing ; **P1 sharding S77 RIG-ABSENT adressé par les
  Phases I/J** (décision PO C1) ; LOT-LOOPBACK-DOC (S80-H-1/2/3/4) → Phase G ; TOOLCHAIN-LABEL
  → préflight Phase G ; Viewer fondation → S82 ; 8 P2 / 11 P3 docs-contract S80 → **sprint dette
  nommé distinct** ; **P2-AUDIT-2** → traité par S81 mais **NON pré-clôturé**.

## Phase A — Fix convergence materializer (wf4) [0-bump, AVANT le bump]

- **But** : éliminer la divergence `PublicRegistryView` cross-noeud sur ingest hors-ordre et établir
  une **baseline 0.98 verte** (bisectabilité : un échec post-bump = iroh, pas le materializer).
  Indépendant d'iroh.
- **Jobs/surfaces** : logique coordinator SQLite (0-bump wire SBFB). Crate `nexus-coordinator-rs`.
- **Livrables** : `feed_materializer.rs` — `materialize_full` → **fold APRÈS `verify_chain`** + tri
  topologique `prev_hash` + tie-break déterministe `(timestamp, author, hash)` + **garde monotone**
  dans `apply()` (ReleasePublished, `:54-58`, fin de l'overwrite inconditionnel) + doc des fonctions
  (`:89-94`, fold non-vérifié `:95-101` corrigé) ; `public_feed.rs` — `verify_entry` vérifie
  `prev_hash` (`:588-591`, aujourd'hui absent).
- **Delta tests attendu** : **+4..6 Rust** (convergence ingest hors-ordre, tie-break déterministe,
  garde monotone, `prev_hash` rejeté).
- **T1** : alimente le sous-test (2) **convergence ingest hors-ordre** — assert `PublicRegistryView`
  identique cross-fold quel que soit l'ordre d'arrivée (l'assertion centrale du fix).
- **Gate / scope-cut** : **commit propre dédié, JAMAIS dans le commit de bump** (R10). 0-bump wire
  SBFB (JCS / `DOMAIN_*_V1` / `FEED_FORMAT_VERSION` intacts).

## Phase A2 — Self-heal root-cause ×2 [0-bump, AVANT le bump — vérification 02/07]

- **But** : fermer à la racine la classe « perte silencieuse warn-only » des DEUX sites
  self-heal destructeurs AVANT toute migration (un échec de migration doit être un crash
  diagnostiquable, jamais une re-création silencieuse de namespace).
- **Jobs/surfaces** : `crates/nexus-shell-daemon/src/runtime.rs` — `boot_storage_namespace`
  `:2456-2549` (recreate `:2518`) ET miroir `boot_feed_namespace` `:2555-2633` (recreate
  `:2606`).
- **Livrables** : sur les 2 sites, `Err` → **fail-fast diagnostiquable** (plus jamais
  `warn` + recreate) ; seul `Ok(None)` (cas légitime : DB importée d'un autre data-dir)
  recrée un namespace neuf.
- **Delta tests attendu** : **+2..4 Rust** (Err fail-fast ×2, Ok(None) recrée ×2).
- **T1** : durcit le sous-test (3) — « self-heal non déclenché » couvre les 2 sites.
- **Gate / scope-cut** : 0-bump, indépendant d'iroh ; commit séparé (bisectabilité).

## Phase A3 — Baseline transport LIVE 0.98 + fix WAN task-delivery (C10) [0-bump]

- **But** : mesurer la baseline transport LIVE réelle sous 0.98 AVANT le bump (jamais
  mesurée : les 5 tests `multi_daemon` relay-gated early-returnent verts EN SILENCE en
  CI — ni Woodpecker ni GHA ne posent `SBFB_INTEGRATION=1`) ET fermer le blocker WAN
  task-delivery S77 (C10 ratifié) pour que le palier quorum b3 ait un PASS atteignable.
- **Jobs/surfaces** : harness b3 par palier + run relay-gated Win + Ollama Mac + blocker
  WAN task-delivery (0-bump). Split de phase possible si le préflight juge le fix WAN
  trop gros (précédent : split E').
- **Livrables** : artefact **JSON b3 par palier COMMITTÉ** (baseline 0.98 : à re-jouer à
  l'identique post-bump = différentiel propre) ; run Win `SBFB_INTEGRATION=1` archivé ;
  **Ollama installé sur le Mac** ; **copie du store VPS rapatriée** (ressource Phase F) ;
  **fix WAN task-delivery** (root-cause S77, 0-bump).
- **Delta tests attendu** : **+1..4 Rust** (selon la forme du fix WAN ; le harness gagne
  des checks préflight).
- **T1** : aucun nouveau sous-test hermétique (baseline live) ; le fix WAN peut ajouter
  une assertion in-process.
- **Gate / scope-cut** : 0-bump strict ; la baseline b3 0.98 est COMMITTÉE avant tout
  bump (différentiel avant/après = preuve de non-régression transport).

## Phase B — Bump deps workspace + recompile mécanique

- **But** : `cargo build --workspace` vert sous iroh 1.0.1 ; corriger l'unique cassure compile
  connue.
- **Jobs/surfaces** : point unique de bump + recompile mécanique des 3 crates déclarant iroh. Crates
  `nexus-core-rs`, `nexus-shell-daemon`, `nexus-shell-daemon-core` (dev-deps).
- **Livrables** : `Cargo.toml:37-41` → `iroh "=1.0.1"` / `iroh-docs "=0.101.0"` / `iroh-gossip
  "=0.101.0"` / `iroh-blobs "=0.103.0"` (pins exacts, D1 amendée — 1.0.1 re-checkée jour J) ;
  deps relogées éventuelles (`iroh-tickets`/`iroh-metrics`) + `irpc` 0.14→0.17 ;
  `pkarr_resolver.rs:40,109`
  `CaRootsConfig→CaTlsConfig` (#4300) + re-vérif `PkarrRelayClient::new(url, tls)` (`:114`) ;
  commentaires de version (`Cargo.toml:33-35`, `node.rs:24`, `blobs.rs:87`, `docs.rs:54`,
  `discovery.rs:6-8`) ; `Cargo.lock` figé et **capturé** pour `cargo tree -d` (Phase G) ; checkpoint
  gossip (pur recompile, aucun changement attendu).
- **Deps / build** : **bump iroh = point unique** ; MSRV **tranchée 1.91** (vérification
  02/07 : rust_version crates.io ×5 — toolchain 1.94 suffit, confirmation au build, pas
  de re-débat).
- **Delta tests attendu** : **0 net** (recompile ; les tests existants doivent rester verts).
- **T1** : aucune nouvelle assertion ; la **baseline T1 0.98 (Phases A/A2)** doit rester verte sous
  le nouveau lock (filet de non-régression du bump).
- **Gate / scope-cut** : iroh SEUL. **Bump toolchain 1.95 INTERDIT** (D6 tranchée). Veille
  **1.0.2**/RustSEC jusqu'au push live (re-check code-freeze).

## Phase C — iroh-docs deep (wire + types iroh-base)

- **But** : adapter la surface docs aux types **iroh-base 0.100** + au wire `EntrySignature →
  iroh::Signature` (0.99.1) ; le vrai travail de migration (pas un recompile).
- **Jobs/surfaces** : wire docs + reconstruction raw-bytes namespace. Crate `nexus-core-rs` (+
  `nexus-shell-daemon` runtime).
- **Livrables** : `docs.rs:42-47,229,275,388-410` (AuthorId / NamespaceId / Entry / DocTicket /
  Query / ShareMode / AddrInfoOptions / LiveEvent re-typés iroh-base) ; `node.rs:388-395`
  (`Docs::persistent/memory/spawn`) ; `runtime.rs:2479` (`DocsNamespaceId::from([u8;32])`,
  reconstruction raw-bytes) ; **suppression actée des zombies legacy-decode** du wire redéfini
  (pre-launch policy : tests de version antérieure = zombies à supprimer immédiatement).
- **Delta tests attendu** : **+2..4 Rust** (round-trip signature / types) **− N zombies legacy-decode**
  (chaque suppression **actée dans le body** de commit, R11).
- **T1** : alimente le sous-test (1) **doc-sync** (wire iroh-docs migré, in-process) + le sous-test
  (4) parse `DocTicket` persisté (string colonne coordinator `doc_ticket`).
- **Gate / scope-cut** : 0 bump wire SBFB (JCS / `DOMAIN_*_V1` / `FEED_FORMAT_VERSION`). Vérifier la
  **stabilité du format string `DocTicket`** (colonne coordinator persistée).

## Phase D — iroh-blobs cascade + redb4

- **But** : recompiler la couche blobs sous **0.103** + valider l'ouverture du store **redb4**.
- **Jobs/surfaces** : surface blobs + tags + downloader + tickets. Crate `nexus-core-rs`.
- **Livrables** : `blobs.rs:85-252` (`add_bytes` / `TagInfo.hash`, `get_bytes` / `has`,
  `tags().set/delete/get`, `HashAndFormat::raw`, `Downloader::new + download`, `BlobTicket::new /
  into_parts`, `Hash::from_bytes`) ; `node.rs:47-50,375-398` (`FsStore::load` / `MemStore` /
  `BlobsProtocol::new` / store deref) ; re-vérif signatures `BlobsProtocol::new` + `Downloader::new`.
- **Delta tests attendu** : **+1..3 Rust** (ticket round-trip, tag set/get, blob fetch local).
- **T1** : alimente le sous-test (1) **blobs fetch** in-process + le sous-test (4) parse `BlobTicket`
  (`anchors.json`).
- **Gate / scope-cut** : changelog 0.101→0.103 non détaillé côté signatures → **découvrir au compile,
  documenter tout break** dans le body.

## Phase E — Surfaces fragiles transport re-cert (3 crates)

- **But** : re-certifier **compile + handshake** des surfaces non-hermétiques + **check nommé de
  survie URL pkarr/relais** (discovery casse silencieusement sinon).
- **Jobs/surfaces** : shard / seed-protocol / pkarr / relais. Crates `nexus-core-rs`,
  `nexus-shell-daemon` (`seed_protocol` impl `ProtocolHandler`), `nexus-shell-daemon-core`.
- **Livrables** : `shard.rs:60-63,171-181,299-327` (`Connection::rtt(PathId::ZERO)`, `closed` /
  `close` / `remote_id` — **traité UNVERIFIED-high-risk, jamais « SAUVE/stable verbatim »**, cf. R5) ;
  **liste canonique des retraits rc.0 re-ancrée au préflight** (vérification 02/07 :
  `Connection::to_info()`→`weak_handle()`, `PathWatcher/PathInfo`→`paths()/PathList` +
  `PathEvent #[non_exhaustive]`, `Incoming::local_ip`→`local_addr`, ClientBuilder
  `query_param`→`auth_token`) ;
  `seed_protocol.rs:44-48,263-264` (`ProtocolHandler` / `AcceptError`, crate `nexus-shell-daemon`) ;
  `pkarr_resolver.rs:38-41,54,107-115` (+ **survie URL `dns.iroh.link/pkarr`** `:54` — check nommé,
  jamais plié dans « recompile ») ; `relay_config.rs:17-20,46` + `node.rs:318,329,348` (`RelayMode::
  Custom`, `default_relay_map` URLs, `presets::N0`) ; **re-scan des call-sites** sur
  `nexus-shell-daemon` + `nexus-shell-daemon-core` (pas seulement `nexus-core-rs`, D7) ;
  **PLAN B C8 PRÉ-PROVISIONNÉ (2-4 j)** : relais iroh self-hosted wire-compat + pkarr
  self-hosted + **acceptance zéro-n0** (le réseau tient sans aucun service n0).
- **Delta tests attendu** : **+1..2 Rust** (handshake seed 2-noeuds in-process ; pkarr resolver parse).
- **T1** : alimente le sous-test (5) **recompile + handshake shard** `sbfb/shard/1` in-process (PAS le
  RTT/multipath live) + le sous-test (1) **seed ALPN** `sbfb/seed/0` handshake.
- **Gate / scope-cut** : la Phase E ne fait que **compile + handshake** — la re-cert LIVE shard
  multipath vit en Phases I/J (décision PO C1 ; R5 amendé). **Split E' possible** si le portage
  shard dépasse le mécanique. Default `presets::N0` conservé ; le plan B C8 est OBLIGATOIRE
  (gates calendaires 01/08 / 25/08 / 15/09 au kickoff).

## Phase F — Migration on-disk redb 2→4 validée sur COPIE

> *Assoupli à l'activation (décision PO C4/C5 : « il n'y a personne sur le réseau ») : le chemin
> le plus simple est AUTORISÉ (wipe + re-pull toléré si l'in-place résiste) ; le préflight de
> phase tranche (PLAN-ADAPT). La fixture migration reste souhaitable comme preuve du chemin
> d'upgrade pour les futurs nœuds tiers. La neutralisation self-heal n'est requise que si le
> chemin in-place est retenu.*

- **But** : prouver **hors-prod** que `docs.redb` + blobs survivent à la migration redb 2→4 ;
  **neutraliser le self-heal destructeur** (si chemin in-place).
- **Jobs/surfaces** : migration on-disk + fixtures + garde self-heal. Crates `nexus-core-rs`,
  `nexus-shell-daemon`.
- **Livrables** : fixture de migration redb 2→4 (store peuplé namespace **sbfb-ides** — trancher
  l'incohérence `sbfb-ides`/`sbfb-ideas` AU CODE au préflight —, saut
  **0.98→0.101 DIRECT** — jamais 0.99/0.100 contre l'ancien store, D3 cond.1 ; la migration est
  **AUTOMATIQUE à l'ouverture**, iroh-docs PR #105 : le préflight LIT le code upstream —
  atomicité, comportement crash mid-migration — la fixture VALIDE, elle n'active rien) ; test
  ouverture store blobs redb2 sous 0.103 (staging) ; **garde vérifiée sur les DEUX sites
  self-heal** (`:2518` + miroir `:2606`, fixés root-cause en A2) : NON déclenchés en fenêtre de
  migration (D3 cond.7) ;
  inventaire « pins re-fetchables ailleurs ? » avant toute tolérance wipe blobs ; vérif parse
  `DocTicket` (DB) + `BlobTicket` (`anchors.json`) post-migration.
- **Delta tests attendu** : **+3..5 Rust** (fixture migration in-place, survie entries + namespace id
  INCHANGÉ, parse tickets persistés, **non-déclenchement self-heal**).
- **T1** : alimente le sous-test (3) **fixture migration redb 2→4** (entries survivent, namespace id
  inchangé, self-heal non déclenché ; store blobs redb2 ouvert sous 0.103) + le sous-test (4) parse
  tickets persistés.
- **Gate / scope-cut** : **aucune migration LIVE ici** — uniquement sur **COPIE** du store VPS peuplé.
  Migration one-way → **documenter rollback = restore tar**. Ressource staging (pull du store live +
  fixture peuplée) budgétée comme pré-requis explicite (R8).

## Phase G — CI / MSRV / convergence crypto + docs sécurité

- **But** : verts dual-platform + **gate de convergence supply-chain** + amendements docs sécurité.
- **Jobs/surfaces** : supply-chain (`cargo tree -d` / `deny.toml`) + docs `docs/security/`. Workspace.
- **Livrables** : `cargo tree -d` (gate de convergence : **un seul** arbre `ed25519-dalek` + **0
  `*-pre`/`*-rc` dupliqués**) → **flip `deny.toml:107` `multiple-versions warn→deny`** OU lever
  **P2-AUDIT-2-RESIDUEL** (carry S82) ; vérif que le `ed25519-dalek 2.x` SBFB ne s'effondre PAS sur
  l'arbre RC d'iroh (`Cargo.toml:58`) ; **rust-version DÉCLARÉE `Cargo.toml:24` 1.85→1.91** (D6
  tranchée — image CI/Docker INCHANGÉE, toolchain 1.94 suffit) ; **trigger de veille iroh-docs
  0.102+** (wire pré-1.0, 2 casses en 6 semaines) ; `cargo-deny` / `cargo-audit` verts ; amendements
  `THREAT_MODEL.md:22,128,195` (0.98→1.0.1 + rationale wire-freeze réduit le churn désérialisation,
  **résiduel reste M**), `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7` (note R-iroh-audit **reconfirmée
  verbatim**, rejouer checklist `cargo tree`), `HARDENING_ROADMAP.md:5` (trigger iroh **FIRED** + bump
  `last_validated`) ; **LOT-LOOPBACK-DOC (audit S80 H-1/2/3/4)** : revalidation
  `LOOPBACK_ENDPOINTS_TRUST_TIERS` §3.1 (routes git/diff+gates + double transport cookie +
  description terminal/ws PTY) + nit §14 EventSource + `last_validated` ; TOOLCHAIN-LABEL
  (décision pin rust-toolchain.toml au préflight).
- **Delta tests attendu** : **0** (gates supply-chain + docs).
- **T1** : aucun nouveau sous-test (gates supply-chain + docs).
- **Gate / scope-cut** : **NE PAS marquer P2-AUDIT-2 CLOSED si le lock ne converge pas** (R6/C7).
  Libellé explicite **« upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 inchangé, pilote reste ferme »**
  (R9/D8). NE PAS rouvrir warrant canary / loopback / guardrails / capability toggles (aucun trigger
  iroh).

## Phase H — Migration LIVE ancre VPS + acceptance

- **But** : migrer le matériel live **sans perte**, dans l'ordre sûr, avec une **fenêtre
  d'incompatibilité BORNÉE** (vérification 02/07 : les flottes relais 0.98/1.0 diffèrent →
  partition possiblement totale pendant la fenêtre).
- **Jobs/surfaces** : runbook opérationnel + déploiement VPS. `deploy/`, ancre Hetzner S75.
- **Livrables** : runbook (`docs/` ou planning) : **tar snapshot sur les 3 NŒUDS**
  (`NEXUS_GRID_ROOT` : `docs.redb` + `blobs/`) AVANT restart (one-way → rollback = restore tar) ;
  **flip same-day en UNE session** : ordre codifié dev Win + Mac puis **VPS EN DERNIER** (wire
  docs/gossip non-rétrocompat intra-rollout, R4) + **gel publish/ingest pendant la fenêtre** +
  **convergence vérifiée après CHAQUE nœud** + re-annonce post-flip ; deploy binaire
  1.0.1 + restart systemd ; vérif 1er boot **0 crash-loop** + `docs.redb` migré + **`node_id`
  INCHANGÉ** + feed / ides / pins intacts ; `deploy/nexus-shell-daemon.service` inchangé
  (`start --headless`).
- **Delta tests attendu** : **0** (acceptance opérationnelle).
- **T1** : aucun (acceptance live) ; alimente **T2** (axe transport).
- **Gate / scope-cut** : **re-install stock S75 INTERDIT sur l'ancre live** (régénérerait
  `node_key`/`node_id` → casse les locators abonnés, D3 cond.5/R1 — conservé même sous C4/C5
  assoupli : coût nul). Migration VPS **bloquée tant que
  la validation sur copie (Phase F) n'est pas PASS** (R2). Gate calendaire C8 : **15/09** —
  Phase H pas faite → plan B ACTIF.

## Phase I — Orchestrateur de session sharding in-vivo (ex-S78) [décision PO C1]

- **But** : livrer l'orchestrateur de session in-vivo dont l'absence a rendu S77/S76
  RIG-ABSENT/DIFFERE — sur la stack iroh 1.0 migrée (dépend de E : `shard.rs` recompilé +
  handshake `sbfb/shard/1` vert ; et de B-D : stack bumpée).
- **Jobs/surfaces** : session lifecycle sharding (annonce/placement/dispatch/collecte)
  côté daemon/coordinator ; référence de scope : `archive/v2.1/sprint78_audit_plan.md`
  §7/§10 (le cœur du S78 différé Factory-first). Le préflight de phase (5 scans) précise
  les livrables exacts depuis le dossier S78 + l'état réel du code shard (N0-N3 S77).
- **Livrables** : orchestrateur runnable par l'opérateur (CLI/harness) qui monte une session
  shard 2-machines réelle (placement Parallax + routing + data-plane `sbfb/shard/1`) ;
  partie hermétiquement testable couverte T1 (session lifecycle in-process 2-nœuds loopback).
- **Delta tests attendu** : **+4..8 Rust** (lifecycle in-process, placement, dispatch, erreurs).
- **T1** : nouveau sous-test (6) **session shard in-process** via l'orchestrateur (loopback,
  sans GPU réel).
- **Gate / scope-cut** : 0 bump wire SBFB (`sbfb/shard/1` inchangé) ; l'orchestrateur est un
  OUTIL opérateur, pas une feature produit nouvelle (re-cert d'un livrable S77 PROVISIONAL).

## Phase J — Benchmark live sharding 2-machines + T2 axe shard [décision PO C1]

- **But** : jouer b3_shard LIVE (RTX 5080 dev Win + Mac M2) via l'orchestrateur (I) sur la
  stack migrée finale (après H) ; solder le carry P1 sharding S77.
- **Jobs/surfaces** : acceptance live + artefact T2 axe shard. Harness `b3_shard`/scripts live
  (memory `live_acceptance_setup`).
- **Livrables** : run b3_shard cross-machine documenté ; verdict JSON au vocabulaire fermé
  (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT` émis par le harness préflight) intégré à l'artefact
  T2 bi-axe ; si PASS → carry P1 sharding S77 CLOSED ; si BLOCK → diagnostic machine-lisible
  + carry re-routé avec cause racine (jamais de prose DIFFERE-*).
- **Delta tests attendu** : **0 Rust** (acceptance live) ; le harness lui-même peut gagner des
  checks préflight.
- **T1** : aucun (live) ; alimente **T2 axe shard**.
- **Gate / scope-cut** : `RIG-ABSENT` légitime UNIQUEMENT si une machine est génuinement HS
  (même matériel que l'axe transport) ; le benchmark tourne sur la stack POST-migration
  (après H) — jamais un mélange 0.98/1.0.

## Phase K — Wrap-up + gate testabilité + roadmap

- **But** : T1 BLOQUANT + T2 JSON LIVE **BI-AXE** + clôture documentaire + carries figés.
- **Jobs/surfaces** : test infra + verification + docs/mémoire + roadmap.
- **Livrables** : **T1 hermétique** (6 sous-tests, cf. §Gate de testabilité) câblé **BLOQUANT** + CI
  chaque push (Win natif + CI Linux Woodpecker/GHA ; **JAMAIS Docker-on-Windows** — `multi_daemon`
  env-bloqué `create_node` hang) ; artefact **T2 JSON committé BI-AXE** (transport-convergence +
  sharding, `PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`) ; re-jeu acceptances **S75 survives-VPS-death** +
  **S76 b3 quorum** + **b3 PASS fetch blob cross-machine** post-upgrade ; convergence
  `PublicRegistryView` cross-noeud après migration LIVE ; amendement `roadmap_v5` (**insertion
  S81-iroh bi-axe** + **Viewer → S82** + **orchestrateur sharding ex-S78 ABSORBÉ par S81 I/J** ;
  tracer « la pre-launch policy *wire modifiable librement* ne couvre PAS le store on-disk
  iroh-docs/blobs déjà déployé ») ; **clôture docs-contrat** (DoD (d) : frontières de la fenêtre
  S81 indexées ou `N-A-no-new-frontier` explicite — leçon S80-K-1 : l'inventaire couvre TOUTE la
  fenêtre du sprint, pas seulement les phases) + **LOT-LOOPBACK-DOC soldé en Phase G vérifié ici** ;
  pipeline **fail-fast 3 blocs** (Rust dual-platform Win + Docker `sbfb-ci` rust:1.94 + frontend
  lint/tsc/vitest/coverage/build/`size`/`scan-en-strings`) ; **libellé T1 corrigé**
  (vérification 02/07 : distinguer hermétique-CI vs relay-gated-local ; câbler un job
  `SBFB_INTEGRATION=1` nightly/manuel OU acter la couverture T2-live — plus jamais de
  early-return vert silencieux non documenté) ; **arbitrage slot S82 BLOQUANT** (C9) ;
  `SPRINT_LOG.md` row 81 + `CLAUDE.md`
  S81 DONE + `nexus_grid_pivot.md` + `MEMORY.md` + `PATTERNS.md` ; `sprint82_audit_plan.md` (carries
  reroutés).
- **Delta tests attendu** : **+ tests T1** consolidés (convergence in-process + fixture redb +
  session shard I) ; **delta net global attendu +14..28 Rust** (deletions zombies actées en Phase C ;
  **total interdit de descendre silencieusement**, R11).
- **T1/T2** : **T1 BLOQUANT-vert complet** (6 sous-tests) ; **T2 JSON committé BI-AXE** (transport
  `PASS` obligatoire ; shard au vocabulaire fermé, `PASS` visé).
- **Gate / scope-cut** : T1 BLOQUANT non négociable ; **T2 LIVE — `RIG-ABSENT` ILLÉGITIME sur
  l'axe transport** (rig VPS Hetzner + dev Win + Mac M2 confirmé dispo, `live_acceptance_setup` ; seul
  un rig génuinement HS le justifie) ; **axe shard : verdict fermé émis par le harness (R12 amendé
  décision PO C1)**.

---

## Récap deps iroh (point unique `Cargo.toml:37-41`) — 3 crates déclarent iroh

| Phase | Acte deps / build Rust |
|---|---|
| A | **aucun** — fix coordinator SQLite 0-bump, **AVANT** le bump (commit séparé, bisectabilité) |
| A2 | **aucun** — self-heal ×2 fail-fast (`runtime.rs:2518`/`:2606`), 0-bump |
| A3 | **aucun** — baseline b3 LIVE 0.98 committée + fix WAN task-delivery, 0-bump |
| B | **bump point unique** : iroh `=1.0.1` / docs `=0.101.0` / gossip `=0.101.0` / blobs `=0.103.0` (+ `iroh-tickets`/`iroh-metrics` si relogement, `irpc` 0.14→0.17) ; `pkarr` `CaRootsConfig→CaTlsConfig` ; `Cargo.lock` figé ; MSRV 1.91 tranchée |
| C | recompile + migration **iroh-docs** (wire + types iroh-base) — `nexus-core-rs` (+ `runtime.rs`) |
| D | recompile **iroh-blobs** + redb4 — `nexus-core-rs` |
| E | recompile call-sites **3 crates** (core + `nexus-shell-daemon` `ProtocolHandler` + dev-deps core) |
| F | fixture migration (dev-dep test) + garde self-heal — `nexus-core-rs` / `nexus-shell-daemon` |
| G | **`deny.toml:107` flip-or-carry** (convergence `cargo tree -d`) ; `Cargo.toml:24` rust-version **si** D6 l'exige |
| H | binaire release VPS (deploy) — aucun changement deps |
| I | **aucun** — orchestrateur session shard (code sur stack déjà bumpée) |
| J | **aucun** — benchmark live b3_shard (acceptance) |
| K | — (consolidation T1/T2 bi-axe + docs) |

Le bump est un **point unique** ; les **call-sites API débordent côté daemon** (`seed_protocol` impl
`ProtocolHandler`) → re-scan des 3 crates obligatoire (D7). Déclarations vérifiées :
`nexus-core-rs/Cargo.toml:19-22` (les 4), `nexus-shell-daemon/Cargo.toml:78,84` (iroh-blobs + iroh),
`nexus-shell-daemon-core/Cargo.toml:179,186` (dev-deps).

## Gate de testabilité (rappel — cf. kickoff §Gate de testabilité)

- **T1 hermétique BLOQUANT** (Win natif + CI Linux ; jamais Docker-on-Windows ; **libellé
  honnête** : les tests relay-gated `SBFB_INTEGRATION=1` sont une classe SÉPARÉE, jamais
  comptée « CI-verte » sans run réel — Phase K câble le job ou acte la couverture T2) :
  (1) convergence
  in-process `multi_daemon` 2-noeuds loopback/`MemoryLookup` (doc-sync + gossip + blobs + seed ALPN +
  ingest annuaire) ; (2) convergence ingest hors-ordre (`PublicRegistryView` identique cross-fold,
  couvre Phase A) ; (3) fixture migration redb 2→4 (entries survivent, namespace id inchangé,
  self-heal non déclenché — **les 2 sites A2** ; blobs redb2 sous 0.103) ; (4) parse tickets
  persistés (`DocTicket` DB +
  `BlobTicket` `anchors.json`) ; (5) recompile + handshake shard `sbfb/shard/1` in-process (PAS le
  RTT/multipath live) ; (6) session shard in-process via l'orchestrateur Phase I (loopback, sans
  GPU réel).
- **T2 acceptance JSON committé — BI-AXE (décision PO C1)** :
  **axe TRANSPORT (PASS obligatoire)** : rig réel VPS + dev Win + Mac M2 — re-jeu S75
  survives-VPS-death + S76 b3 quorum + b3 PASS fetch blob cross-machine + convergence
  `PublicRegistryView` cross-noeud après migration LIVE ;
  **axe SHARDING (Phases I/J)** : b3_shard LIVE 5080+M2 via l'orchestrateur, verdict fermé émis
  par le harness préflight (`RIG-ABSENT` = machine génuinement HS uniquement).

## Scope cuts (rappel — cf. kickoff §Out ; amendé décision PO C1 : le sharding N'EST PLUS un cut)

**Viewer fondation** + Aperçu scellé/Proof Card (→ S82) · dette docs-contract
**8 P2 / 11 P3** S80 (→ sprint dette nommé distinct, jamais bundlé) · **P1 in-vivo app-authoring
S79** standing (le P1 sharding S77 est ADRESSÉ par I/J) · **GuardianDB** / toute autre upgrade
(séparé et postérieur, bisectabilité) · **bump MSRV 1.95 inconditionnel** (INTERDIT sans preuve
cargo) · pagination app-storage + features produit non liées à iroh (backlog) · **clôture
pré-annoncée P2-AUDIT-2** (INTERDITE sans `cargo tree -d` convergent) · **bundler le materializer
dans le commit de bump** (INTERDIT — Phase A est un commit séparé AVANT le bump).