# Sprint 81 — Plan : Upgrade iroh 0.98 → 1.0 + sweep deps (migration transport one-way)

> **⚠️ DRAFT DE STAGING — S80 PAS ENCORE CLOS.** Ce plan est un brouillon de pré-positionnement
> pour S81. Tout ce qui dépend du verdict S80 (Phase 0, liste exacte des carries entrants, totaux
> de tests baseline) est marqué *provisoire* et sera **figé à la clôture de S80** (rejeu réel de la
> Phase 0). Le corps S81 (A→I) ne démarre qu'après **Phase 0 PASS** (audit gate S80). Source unique :
> `sprint81_kickoff.md` + dossier canonique S81 (corrections sceptique intégrées : self-heal
> `runtime.rs:2515` destructeur, materializer `feed_materializer.rs:54-58`, 3 crates déclarent iroh).

> Phases dimensionnées par le **travail**, JAMAIS par LOC. **Phase 0 = audit gate S80** (joué à
> l'ouverture réelle, convention permanente). S81 = **iroh STRICTEMENT SEUL** (bisectabilité ;
> materializer en Phase A commit séparé AVANT le bump ; tout le reste rerouté S82/dette). 1 commit
> atomique par phase `feat(scope): Sprint 81 Phase X — titre` (ou `fix(...)`/`chore(...)` selon
> nature) ; **rigueur per-phase uniforme** : deep preflight (5 scans) → review Workflow → Codex avant
> CHAQUE commit ; T1 hermétique grandit incrémentalement (BLOQUANT au wrap-up + CI chaque push),
> T2 artefact JSON **committé** (axe transport-convergence). Migration on-disk redb 2→4 = **one-way**
> (rollback = restore tar).

> **Cadrage DONE (Arbitrage PO C1/D4).** Le DONE non-PROVISIONAL de S81 est scopé sur l'**axe
> TRANSPORT-convergence** (doc-sync / gossip / blobs / seed / annuaire). L'**axe SHARDING** (re-cert
> LIVE `shard.rs` RTT/PathId multipath + orchestrateur in-vivo) est **explicitement hors T2 → S82**
> (rig GPU chroniquement absent ; on ne re-joue PAS l'acceptance S77 b3_shard jamais passée). Sans ce
> cadrage, S81 finirait PROVISIONAL sur l'axe shard exactement comme S77.

---

## Phase 0 — Audit gate S80 (À JOUER À L'OUVERTURE RÉELLE — *provisoire*)

> Placeholder de staging. Rejouée à la clôture de S80 (convention permanente). Absorbe le verdict S80,
> **fige la liste exacte des carries entrants** (cf. §Carries) et la **baseline de tests**.

- **Verdict** : *à établir à la clôture S80* (attendu : PASS / CONDITIONAL PASS). 0 P0 visé ; les P1
  nouveaux éventuels sont résolus hors corps avant ouverture du corps S81.
- **Commits** : *à figer* (findings + routing audit S80 ; note de résolution si P1 nouveau).
- **Baseline tests à figer** : totaux Rust nextest (Win natif + Docker canonique) + Vitest `web/`
  post-S80 (intègre le delta de couverture S80 Phase I : re-couverture SSE single-Done + jettison
  factory-operator/factory-ui acté).
- **Carries figés** (*provisoire — cf. §Carries entrants*) : 2 P1 in-vivo OUVERTS (sharding S77
  RIG-ABSENT, app-authoring S79 `Not evidenced`) ; Viewer fondation + Aperçu scellé/Proof Card
  (réservés S81 à l'origine → **reroutés S82**) ; 8 P2 / 11 P3 docs-contract S80 → **sprint dette
  nommé distinct** ; **P2-AUDIT-2** (pin transitif iroh) → traité par S81 mais **NON pré-clôturé**.

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

## Phase B — Bump deps workspace + recompile mécanique + MSRV empirique

- **But** : `cargo build --workspace` vert sous iroh 1.0 ; corriger l'unique cassure compile connue ;
  fixer la MSRV **réelle** (empirique, pas budgétée).
- **Jobs/surfaces** : point unique de bump + recompile mécanique des 3 crates déclarant iroh. Crates
  `nexus-core-rs`, `nexus-shell-daemon`, `nexus-shell-daemon-core` (dev-deps).
- **Livrables** : `Cargo.toml:37-41` → `iroh "=1.0.0"` / `iroh-docs "0.101.0"` / `iroh-gossip
  "0.101.0"` / `iroh-blobs "0.103.0"` (pin exact, D1) ; `pkarr_resolver.rs:40,109`
  `CaRootsConfig→CaTlsConfig` (#4300) + re-vérif `PkarrRelayClient::new(url, tls)` (`:114`) ;
  commentaires de version (`Cargo.toml:33-35`, `node.rs:24`, `blobs.rs:87`, `docs.rs:54`,
  `discovery.rs:6-8`) ; `Cargo.lock` figé et **capturé** pour `cargo tree -d` (Phase G) ; checkpoint
  gossip (pur recompile, aucun changement attendu).
- **Deps / build** : **bump iroh = point unique** ; vérif `cargo +1.94 build` Docker canonique
  (décision MSRV, D6) — **rester 1.94 sauf preuve cargo qu'une feuille exige plus**.
- **Delta tests attendu** : **0 net** (recompile ; les tests existants doivent rester verts).
- **T1** : aucune nouvelle assertion ; la **baseline T1 0.98 (Phase A)** doit rester verte sous le
  nouveau lock (filet de non-régression du bump).
- **Gate / scope-cut** : iroh SEUL. **Bump MSRV 1.95 INTERDIT sans preuve cargo** (R7). `iroh =
  "=1.0.0"` provisoire → **re-pin OBLIGATOIRE sur la 1re 1.0.x patch AVANT push live** (D1/C3) ;
  interdiction de pousser la `.0` brute si une patch existe.

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
  `seed_protocol.rs:44-48,263-264` (`ProtocolHandler` / `AcceptError`, crate `nexus-shell-daemon`) ;
  `pkarr_resolver.rs:38-41,54,107-115` (+ **survie URL `dns.iroh.link/pkarr`** `:54` — check nommé,
  jamais plié dans « recompile ») ; `relay_config.rs:17-20,46` + `node.rs:318,329,348` (`RelayMode::
  Custom`, `default_relay_map` URLs, `presets::N0`) ; **re-scan des call-sites** sur
  `nexus-shell-daemon` + `nexus-shell-daemon-core` (pas seulement `nexus-core-rs`, D7).
- **Delta tests attendu** : **+1..2 Rust** (handshake seed 2-noeuds in-process ; pkarr resolver parse).
- **T1** : alimente le sous-test (5) **recompile + handshake shard** `sbfb/shard/1` in-process (PAS le
  RTT/multipath live) + le sous-test (1) **seed ALPN** `sbfb/seed/0` handshake.
- **Gate / scope-cut** : **re-cert LIVE shard multipath = OUT → S82** (R5). Provisionner un relais
  iroh self-hosted **optionnel** pour l'ancre VPS (résilience, D2). Default `presets::N0` conservé.

## Phase F — Migration on-disk redb 2→4 validée sur COPIE

- **But** : prouver **hors-prod** que `docs.redb` + blobs survivent à la migration redb 2→4 ;
  **neutraliser le self-heal destructeur**.
- **Jobs/surfaces** : migration on-disk + fixtures + garde self-heal. Crates `nexus-core-rs`,
  `nexus-shell-daemon`.
- **Livrables** : fixture de migration redb 2→4 (store peuplé namespace **sbfb-ides**, saut
  **0.98→0.101 DIRECT** — jamais 0.99/0.100 contre l'ancien store, D3 cond.1) ; test ouverture store
  blobs redb2 sous 0.103 (staging) ; **garde explicite autour de `runtime.rs:2515-2528`** : le
  self-heal (branche `None` → `create_doc()` namespace id NEUF + `set_storage_namespace` écrasant la
  ligne M8 **sans `import_ticket`**) est **NON déclenché en fenêtre de migration** — ce n'est **PAS un
  backstop**, c'est une perte silencieuse `warn`-only (correction critique sceptique, D3 cond.7) ;
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
  l'arbre RC d'iroh (`Cargo.toml:58`) ; image CI / Docker canonique + `Cargo.toml:24` rust-version
  **seulement si** D6 l'exige (preuve cargo) ; `cargo-deny` / `cargo-audit` verts ; amendements
  `THREAT_MODEL.md:22,128,195` (0.98→1.0.0 + rationale wire-freeze réduit le churn désérialisation,
  **résiduel reste M**), `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7` (note R-iroh-audit **reconfirmée
  verbatim**, rejouer checklist `cargo tree`), `HARDENING_ROADMAP.md:5` (trigger iroh **FIRED** + bump
  `last_validated`).
- **Delta tests attendu** : **0** (gates supply-chain + docs).
- **T1** : aucun nouveau sous-test (gates supply-chain + docs).
- **Gate / scope-cut** : **NE PAS marquer P2-AUDIT-2 CLOSED si le lock ne converge pas** (R6/C7).
  Libellé explicite **« upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 inchangé, pilote reste ferme »**
  (R9/D8). NE PAS rouvrir warrant canary / loopback / guardrails / capability toggles (aucun trigger
  iroh).

## Phase H — Migration LIVE ancre VPS + acceptance

- **But** : migrer le matériel live **sans perte**, dans l'ordre sûr.
- **Jobs/surfaces** : runbook opérationnel + déploiement VPS. `deploy/`, ancre Hetzner S75.
- **Livrables** : runbook (`docs/` ou planning) : **tar snapshot** `NEXUS_GRID_ROOT` (`docs.redb` +
  `blobs/`) AVANT restart (one-way → rollback = restore tar) ; **ordre codifié : dev Win + Mac
  d'abord, VPS EN DERNIER** (wire docs/gossip non-rétrocompat intra-rollout, R4) ; deploy binaire
  1.0.x + restart systemd ; vérif 1er boot **0 crash-loop** + `docs.redb` migré + **`node_id`
  INCHANGÉ** + feed / ides / pins intacts ; `deploy/nexus-shell-daemon.service` inchangé
  (`start --headless`).
- **Delta tests attendu** : **0** (acceptance opérationnelle).
- **T1** : aucun (acceptance live) ; alimente **T2** (axe transport).
- **Gate / scope-cut** : **re-install stock S75 INTERDIT sur l'ancre live** (régénérerait
  `node_key`/`node_id` → casse les locators abonnés, D3 cond.5/R1). Migration VPS **bloquée tant que
  la validation sur copie (Phase F) n'est pas PASS** (R2).

## Phase I — Wrap-up + gate testabilité + roadmap

- **But** : T1 BLOQUANT + T2 JSON LIVE (axe transport) + clôture documentaire + carries figés.
- **Jobs/surfaces** : test infra + verification + docs/mémoire + roadmap.
- **Livrables** : **T1 hermétique** (5 sous-tests, cf. §Gate de testabilité) câblé **BLOQUANT** + CI
  chaque push (Win natif + CI Linux Woodpecker/GHA ; **JAMAIS Docker-on-Windows** — `multi_daemon`
  env-bloqué `create_node` hang) ; artefact **T2 JSON committé** (transport-convergence,
  `PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`) ; re-jeu acceptances **S75 survives-VPS-death** + **S76 b3
  quorum** + **b3 PASS fetch blob cross-machine** post-upgrade ; convergence `PublicRegistryView`
  cross-noeud après migration LIVE ; amendement `roadmap_v5` (**insertion S81-iroh** + **Viewer →
  S82** + **orchestrateur sharding ex-S78 séquencé APRÈS S81** ; tracer « la pre-launch policy *wire
  modifiable librement* ne couvre PAS le store on-disk iroh-docs/blobs déjà déployé ») ; pipeline
  **fail-fast 3 blocs** (Rust dual-platform Win + Docker `sbfb-ci` rust:1.94 + frontend
  lint/tsc/vitest/coverage/build/`size`/`scan-en-strings`) ; `SPRINT_LOG.md` row 81 + `CLAUDE.md`
  S81 DONE + `nexus_grid_pivot.md` + `MEMORY.md` + `PATTERNS.md` ; `sprint82_audit_plan.md` (carries
  reroutés).
- **Delta tests attendu** : **+ tests T1** consolidés (convergence in-process + fixture redb) ;
  **delta net global attendu +10..20 Rust** (deletions zombies actées en Phase C ; **total interdit de
  descendre silencieusement**, R11).
- **T1/T2** : **T1 BLOQUANT-vert complet** (5 sous-tests) ; **T2 JSON `PASS` committé** (axe
  transport).
- **Gate / scope-cut** : T1 BLOQUANT non négociable ; **T2 LIVE PASS — `RIG-ABSENT` ILLÉGITIME sur
  l'axe transport** (rig VPS Hetzner + dev Win + Mac M2 confirmé dispo, `live_acceptance_setup` ; seul
  un rig génuinement HS le justifie). **Axe sharding hors T2 → S82** (R12).

---

## Récap deps iroh (point unique `Cargo.toml:37-41`) — 3 crates déclarent iroh

| Phase | Acte deps / build Rust |
|---|---|
| A | **aucun** — fix coordinator SQLite 0-bump, **AVANT** le bump (commit séparé, bisectabilité) |
| B | **bump point unique** : iroh `=1.0.0` / docs `0.101.0` / gossip `0.101.0` / blobs `0.103.0` ; `pkarr` `CaRootsConfig→CaTlsConfig` ; `Cargo.lock` figé ; **MSRV empirique** (`cargo +1.94 build` Docker) |
| C | recompile + migration **iroh-docs** (wire + types iroh-base) — `nexus-core-rs` (+ `runtime.rs`) |
| D | recompile **iroh-blobs** + redb4 — `nexus-core-rs` |
| E | recompile call-sites **3 crates** (core + `nexus-shell-daemon` `ProtocolHandler` + dev-deps core) |
| F | fixture migration (dev-dep test) + garde self-heal — `nexus-core-rs` / `nexus-shell-daemon` |
| G | **`deny.toml:107` flip-or-carry** (convergence `cargo tree -d`) ; `Cargo.toml:24` rust-version **si** D6 l'exige |
| H | binaire release VPS (deploy) — aucun changement deps |
| I | — (consolidation T1/T2 + docs) |

Le bump est un **point unique** ; les **call-sites API débordent côté daemon** (`seed_protocol` impl
`ProtocolHandler`) → re-scan des 3 crates obligatoire (D7). Déclarations vérifiées :
`nexus-core-rs/Cargo.toml:19-22` (les 4), `nexus-shell-daemon/Cargo.toml:78,84` (iroh-blobs + iroh),
`nexus-shell-daemon-core/Cargo.toml:179,186` (dev-deps).

## Gate de testabilité (rappel — cf. kickoff §Gate de testabilité)

- **T1 hermétique BLOQUANT** (Win natif + CI Linux ; jamais Docker-on-Windows) : (1) convergence
  in-process `multi_daemon` 2-noeuds loopback/`MemoryLookup` (doc-sync + gossip + blobs + seed ALPN +
  ingest annuaire) ; (2) convergence ingest hors-ordre (`PublicRegistryView` identique cross-fold,
  couvre Phase A) ; (3) fixture migration redb 2→4 (entries survivent, namespace id inchangé,
  self-heal non déclenché ; blobs redb2 sous 0.103) ; (4) parse tickets persistés (`DocTicket` DB +
  `BlobTicket` `anchors.json`) ; (5) recompile + handshake shard `sbfb/shard/1` in-process (PAS le
  RTT/multipath live).
- **T2 acceptance JSON committé — AXE TRANSPORT (PASS obligatoire DANS S81)** : rig réel VPS + dev Win
  + Mac M2 — re-jeu S75 survives-VPS-death + S76 b3 quorum + b3 PASS fetch blob cross-machine +
  convergence `PublicRegistryView` cross-noeud après migration LIVE.
- **AXE SHARDING — HORS S81** : `shard.rs` RTT/PathId multipath + orchestrateur in-vivo = non testable
  hermétiquement, rig GPU chroniquement absent → **reporté S82** (après orchestrateur ex-S78).

## Scope cuts (rappel — cf. kickoff §Out)

Re-cert LIVE du **sharding** (→ S82, après orchestrateur ex-S78 ; on ne re-joue PAS S77 b3_shard
jamais passée) · **Viewer fondation** + Aperçu scellé/Proof Card (→ S82) · dette docs-contract
**8 P2 / 11 P3** S80 (→ sprint dette nommé distinct, jamais bundlé) · **2 P1 in-vivo** standing
(sharding S77, app-authoring S79) · **GuardianDB** / toute autre upgrade (séparé et postérieur,
bisectabilité) · **bump MSRV 1.95 inconditionnel** (INTERDIT sans preuve cargo) · pagination
app-storage + features produit non liées à iroh (backlog) · **clôture pré-annoncée P2-AUDIT-2**
(INTERDITE sans `cargo tree -d` convergent) · **bundler le materializer dans le commit de bump**
(INTERDIT — Phase A est un commit séparé AVANT le bump).