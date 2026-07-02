# Sprint 81 — Kickoff : Upgrade iroh 0.98 → 1.0 + sweep deps

> **STATUT : ACTIVÉ le 2026-07-02** (Cas C, après Phase 0 = audit gate S80
> CONDITIONAL PASS → PASS effectif, findings `dcc3eea`, fix P1 `2c85b28`).
> Sprint de **maintenance d'infrastructure forcing-function-driven**
> (insertion roadmap non-planifiée) : migrer toute la pile iroh `0.98 → 1.0` GA
> avant le cutoff relais N0 du **2026-09-30**, prouver la convergence transport
> cross-machine par un re-jeu **LIVE**, **et re-certifier le sharding live**
> (décision PO C1 ci-dessous). **Décision-grade, pas rubber-stamp** : faits
> re-vérifiés au code le 2026-06-27 ; corrections du sceptique **intégrées** ;
> contradictions inter-cartes tranchées (cf. §Arbitrages PO + §Day-0).

## Décisions PO à l'activation (2026-07-02) — AUTORITÉ sur tout passage contraire

> Confirmation des arbitrages C1..C10 recueillie à l'activation (procédure
> README staging §3). Ce bloc intègre AUSSI le **registre d'amendements de la
> vérification ultracode du 2026-07-02**
> (`.planning/research/sprint81_iroh_upgrade/verification_2026-07-02.md`,
> `wf_8ef303fb-526`) : plan 12→14 phases (A2/A3 + I/J sharding), pin `=1.0.1`,
> auto-migration redb #105, self-heal ×2, MSRV tranchée 1.91. Ce bloc SUPERSEDE
> tout passage du kickoff/plan rédigé sous une hypothèse antérieure.
> **Re-check crates.io jour J (prérequis #6, 2026-07-02)** : iroh max_stable =
> **1.0.1** (29/06, pas de 1.0.2) ; iroh-docs 0.101.0 ; iroh-gossip 0.101.0 ;
> iroh-blobs 0.103.0 — la vérification est CONFIRMÉE fraîche. (Attention outil :
> `cargo info` local lisait un index caché périmé « 1.0.0-rc.1 » — toujours
> vérifier via l'API crates.io.)

- **C1 — TRANCHÉ CONTRE la reco : le sharding est INCLUS au T2 de S81.**
  Le DONE de S81 est **bi-axe** : transport-convergence ET re-cert live
  sharding. Conséquences structurantes : l'**orchestrateur de session
  in-vivo (ex-S78)** entre au sprint (nouvelle Phase I) + le **benchmark
  live 2-machines b3_shard** (nouvelle Phase J) ; le wrap-up devient
  Phase K. Le verdict T2 axe shard suit le vocabulaire fermé
  (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT` émis par le harness préflight) :
  le rig nominal (RTX 5080 dev Win + Mac M2) est le MÊME matériel que
  l'axe transport, et l'orchestrateur n'est plus absent (Phase I le
  livre) — `RIG-ABSENT` n'est légitime que si une machine est génuinement
  HS. Cohérent directive PO « sprints ultra-complets » (0 defer du cœur).
- **C2 — reco confirmée** : fix materializer Phase A in-sprint, AVANT le
  bump, commit séparé. La vérification 02/07 ajoute **Phase A2** (self-heal
  root-cause **×2** : `boot_storage_namespace` :2456-2549/recreate :2518 ET
  le miroir `boot_feed_namespace` :2555-2633/recreate :2606 — Err→fail-fast
  diagnostiquable, Ok(None) seul recrée) et **Phase A3** (baseline transport
  LIVE 0.98 : artefact JSON b3 par palier committé + run Win
  `SBFB_INTEGRATION=1` archivé — les 5 tests multi_daemon relay-gated
  early-returnent verts EN SILENCE en CI depuis toujours — + Ollama sur le
  Mac + copie store VPS rapatriée + **fix WAN task-delivery C10**).
- **C3 — confirmée puis AMENDÉE par la vérification 02/07** : la 1re 1.0.x
  EST publiée (iroh **1.0.1**, 29/06, re-checkée jour J) → coder directement
  sur `iroh = "=1.0.1"` + compagnons exacts `=0.101.0`/`=0.101.0`/`=0.103.0` ;
  veille 1.0.2 + RustSEC jusqu'au push live. Pins EXACTS délibérés
  (reproductibilité + bisectabilité ; iroh-docs pré-1.0 wire-instable,
  2 casses en 6 semaines → trigger de veille 0.102+).
- **C4/C5 — ASSOUPLI par le PO : « il n'y a personne sur le réseau pour
  l'instant ».** Aucun nœud tiers n'existe : la contrainte de
  préservation ne protège que nos propres données. Le chemin de données
  le plus SIMPLE est AUTORISÉ (wipe + re-pull toléré au-delà des seuls
  pins re-fetchables si l'in-place résiste) ; restent recommandés à bas
  coût : tar snapshot avant 1er boot 0.101 (rollback trivial) +
  conservation `node_key`/`node_id` VPS. La neutralisation du self-heal
  (`runtime.rs:2515`) n'est REQUISE que si le chemin in-place est retenu
  (pas de fenêtre de migration silencieuse sur le chemin wipe). Les
  préflights des Phases F/H tranchent le chemin final (PLAN-ADAPT) avec
  cette liberté ; la fixture migration redb 2→4 (T1.3) reste souhaitable
  comme preuve du chemin d'upgrade pour les futurs nœuds tiers.
- **C6 — TRANCHÉE par la vérification 02/07** : `rust_version=1.91` confirmé
  crates.io pour les 5 crates → toolchain 1.94 SUFFIT, image CI inchangée ;
  résiduel = bump de la rust-version DÉCLARÉE `Cargo.toml:24` 1.85→1.91
  (Phase G). **C7 — reco confirmée** : P2-AUDIT-2 non pré-clôturé (gate
  `cargo tree -d` flip-or-carry).
- **C4-bis (vérification 02/07)** : la « feature défaut `redb-v2-migration` »
  **N'EXISTE PAS** — la migration redb est AUTOMATIQUE à l'ouverture
  (iroh-docs PR #105 ; saut réel redb ^2.6.3→^4.1). La fixture Phase F
  VALIDE l'auto-migration (atomicité, crash mid-migration : préflight F lit
  le code upstream), elle n'active rien.
- **C8 — RATIFIÉ PO 2026-07-02 : plan B relais/discovery self-hosted
  PRÉ-PROVISIONNÉ** (Phase E, 2-4 j : relais iroh self-hosted wire-compat +
  pkarr self-hosted + acceptance zéro-n0) + **3 gates calendaires** :
  **01/08** corps S81 pas ouvert → provisionner immédiatement ; **25/08**
  Phase F pas PASS → basculer la flotte sur le plan B ; **15/09** Phase H
  pas faite → plan B ACTIF (2 semaines de vérification zéro-n0 avant l'EOL
  30/09).
- **C9 — arbitrage slot S82** : séquencement PO acté (S82 = workflow-engine)
  re-confirmé BLOQUANT en Phase K (les prétendants restants : fondation
  Viewer, dette docs-contract — le sharding live est absorbé par S81 C1).
- **C10 — RATIFIÉ PO 2026-07-02 : fixer le blocker WAN task-delivery en
  0-bump AVANT le bump** (logé en Phase A3, split possible si le préflight
  le juge trop gros) + Ollama installé sur le Mac → le palier quorum b3
  RESTE dans le T2 transport avec un PASS atteignable (aucun b3 quorum
  PASS complet n'a jamais existé — S81 vise le premier).
- **Carries d'audit S80 actés à l'activation** :
  LOT-LOOPBACK-DOC (S80-H-1/2/3/4) → livrable additionnel **Phase G**
  (qui amende déjà les docs sécurité) ; TOOLCHAIN-LABEL (S80-A-2) →
  décision au préflight **Phase G** (rust-toolchain.toml ou statu quo
  Docker-canonique) ; DOC-LINT-SEMANTIC (S80-G-1) → **ACCEPT-AND-CLOSE
  acté** : le doc-lint reste existence-only, la vérification sémantique
  des claims = revue LLM adversariale par sprint (review de phase +
  audit gate), non automatisable en shell — exit condition remplie,
  l'item sort des carries ; TRAILER (S80-I-1) → **accepté, pas
  d'enforcement hook** (cosmétique) ; DELTA-DISCIPLINE (S80-E-1/E-2) →
  vigilance comptage delta aux wrap-ups ; S79-P2-1 ancres
  (task_response.rs:14,:84-85,:93,:95 + PROMISE_RE) → sprint dette
  nommé (iroh STRICTEMENT SEUL interdit le bundle).
- **Push** : groupé après le commit d'activation kickoff S81.

**Écrit** : 2026-06-27 (staging) ; **activé** : 2026-07-02 (post-audit S80).
**Type** : **sprint de maintenance d'infrastructure** (upgrade transport ; orthogonal
au produit utilisateur — n'ajoute aucune feature).
Le travail touche **3 crates déclarant iroh en direct** (`nexus-core-rs`,
`nexus-shell-daemon`, `nexus-shell-daemon-core` dev-deps), point de bump unique
`Cargo.toml:37-41`, plus le fix convergence materializer 0-bump dans
`nexus-coordinator-rs` (Phase A) + 2 migrations on-disk redb 2→4 + une migration
LIVE de l'ancre VPS.
**Budget de phases** : Phase 0 (audit gate S80, JOUÉE) + **A, A2, A3, B→K**
(A2 = self-heal ×2 + A3 = baseline live 0.98 + WAN C10, ajouts vérification
02/07 ; I = orchestrateur sharding ex-S78, J = benchmark live shard, K =
wrap-up — décision PO C1 ; 14 phases au total ; le nombre de phases n'est
jamais plafonné, README §4 ; dimensionné par le travail, JAMAIS par LOC).
Rigueur per-phase **uniforme** : deep preflight (5 scans) + review + Codex à
**CHAQUE** phase.
**Numéro/version archive** : **S81**, v2.1 (OPEN).

---

## Objectif produit

Migrer toute la pile iroh de SBFB du pin `0.98` (ligne 0.9x, **maintenance coupée dès
la GA 1.0**, relais publics N0 sunset **2026-09-30**) vers iroh 1.0 GA (`iroh 1.0.0`
+ `iroh-docs 0.101` + `iroh-gossip 0.101` + `iroh-blobs 0.103`), en préservant **sans
perte** les données live déjà déployées (ancre VPS Hetzner S75, store `iroh-docs`
sbfb-ides, pins `keep_online` M18) à travers une migration on-disk redb 2→4
**one-way**, et en prouvant que la convergence transport cross-machine (doc-sync +
gossip + blobs + seed/annuaire) survit au bump par un re-jeu **LIVE**.

C'est une **maintenance d'infrastructure forcing-function-driven** : elle garantit la
continuité de la découverte/connectivité **avant** le cutoff relais et solde la dette
de version, **sans rien ajouter** au produit utilisateur. Le **DONE non-PROVISIONAL est
BI-AXE** (décision PO C1 à l'activation, 2026-07-02) : axe **TRANSPORT-convergence**
(doc-sync/gossip/blobs/seed/annuaire) ET axe **SHARDING** (re-cert live `shard.rs`
RTT/PathId multipath via l'orchestrateur de session in-vivo ex-S78, Phases I/J).

---

## Pourquoi maintenant

1. **Forcing function dure — 2026-09-30 (~3 mois).** Les relais publics N0 de la ligne
   0.9x (qui **inclut le pin 0.98**) sont coupés au 2026-09-30 ; la ligne 1.0 est
   supportée « until End of Life » (iroh.computer/blog/v1). Surface concernée :
   `node.rs:318` `Endpoint::builder(presets::N0)` + retombée par défaut sur les 3 relais
   n0 prod via `relay_config.rs:17-20`. Sans upgrade, la 0.98 perd sa maintenance ET N0
   cesse. L'escape-hatch opérateur `SBFB_CUSTOM_RELAYS`/`relays.json` → `RelayMode::Custom`
   (`node.rs:329,348`) est un palliatif de survie, **pas** un substitut à la migration.
2. **Dette de version.** Le pin 0.98 a ~2 mois de retard sur la GA et tombe hors train de
   patchs sécurité dès la 1.0. Le rename `Node→Endpoint` est **déjà absorbé** (le fantôme
   « Endpoint Takeover 153 call-sites » est mort : le code tourne post-rename —
   `endpoint.id()`, `presets::N0`, `MemoryLookup`, `EndpointAddr`, `ProtocolHandler`
   async/`AcceptError`). Le vrai travail = migration `iroh-docs` (wire + types) + 2
   migrations on-disk redb 2→4.
3. **P2-AUDIT-2 (pré-release transitives).** Le lock 0.98 actuel tire DÉJÀ un fouillis
   crypto pré-release **dupliqué** (ed25519-dalek `3.0.0-pre.6` **ET** `3.0.0-rc.4`,
   sha2 `0.11.0-rc`, der/pkcs8/spki `0.8.0-rc`, curve25519 `5.0.0-pre.6`). **Correction
   intégrée (sceptique + D6)** : l'upgrade **ne ferme PAS** P2-AUDIT-2 — iroh 1.0 épingle
   encore ed25519-dalek sur un `-rc`. La crypto SBFB security-critical (canary/curator/
   provenance/task, `ed25519-dalek 2.x` stable, `Cargo.toml:58`) est **isolée** et
   inchangée. L'upgrade améliore (déduplication) mais le RC reste un **résiduel** ;
   clôture conditionnée à `cargo tree -d` convergent (cf. C7/D6).

---

## Arbitrages PO (à trancher avant le 1er Edit — load-bearing)

> Ces arbitrages doivent être tranchés par le PO **avant** le 1er Edit. Ils englobent
> les 3 nommés (materializer / version / données-live) + les contradictions inter-cartes
> relevées par le sceptique. Les recommandations (Option) ci-dessous sont **intégrées
> comme tranchées** dans le §Day-0 ; elles restent confirmables au preflight.

1. **C1 — Scope « non-PROVISIONAL DONE » réaliste en 1 sprint ? (le plus structurant).**
   Le sceptique réfute « S81 = 1 sprint propre vers un DONE non-PROVISIONAL » : le bar
   concentre migration live one-way + re-cert cross-machine + fix materializer + docs
   sécurité, **sous rigueur per-phase uniforme**. Reco initiale : scoper le DONE sur l'axe
   TRANSPORT-convergence uniquement, sortir l'axe SHARDING du T2 (re-cert live shard
   → S82). **TRANCHÉ PO 2026-07-02 CONTRE la reco : sharding INCLUS au T2** (bi-axe,
   Phases I/J ajoutées — cf. bloc Décisions PO à l'activation, qui fait autorité).
2. **C2 — Scope du fix materializer (wf4).** Contradiction inter-cartes : Carte 1 D8 →
   « sprint séparé postérieur » ; Cartes 3/4 + sceptique → **Phase A DANS S81, AVANT le
   bump, commit propre séparé**. **Reco : Phase A in-sprint.** Raison : le bug est
   convergence-critique et 0-bump (logique coordinator SQLite, indépendante d'iroh) ; le
   **gate de convergence cross-machine de S81 le révèle de toute façon** ; le corriger
   AVANT le bump établit une baseline 0.98 verte → préserve la bisectabilité (un échec
   post-bump = iroh, pas le materializer). **Discipline imposée** : jamais dans le commit
   de bump.
3. **C3 — Version cible.** *(PÉRIMÉE-RÉSOLUE à l'activation : iroh 1.0.1 publiée le
   29/06 et re-checkée jour J → coder directement `=1.0.1`, cf. bloc Décisions PO.
   Texte historique du 27/06 conservé ci-dessous.)* **Correction intégrée (carte 3 +
   sceptique)** : « viser la
   dernière 1.0.x » est **actuellement insatisfiable** — au 2026-06-27, `1.0.0` (12 j)
   est la **seule** stable, **aucune 1.0.x patch n'existe**. **Reco : coder sur
   `iroh = "=1.0.0"` maintenant + re-pin OBLIGATOIRE sur la 1re 1.0.x publiée AVANT le
   push live** ; si aucune patch au code-freeze → soak documenté + veille RustSEC
   (interdiction de pousser la .0 brute si une patch existe). Le runway ~3 mois interdit
   d'attendre passivement.
4. **C4 — Stratégie données-live.** *(ASSOUPLI PO 2026-07-02 : « il n'y a personne sur
   le réseau pour l'instant » — chemin simple autorisé, cf. bloc Décisions PO à
   l'activation ; le durcissement ci-dessous reste le chemin PRÉFÉRÉ si son coût est
   raisonnable, les préflights F/H tranchent. NOTE vérification 02/07 : la « feature
   défaut redb-v2-migration » citée ci-dessous N'EXISTE PAS — auto-migration à
   l'ouverture, PR #105, cf. C4-bis.)* **Reco (hybride durcie)** : migration **IN-PLACE
   impérative** pour `docs.redb` (saut 0.98→0.101 DIRECT, feature défaut
   `redb-v2-migration`), validée sur **COPIE** du store VPS peuplé AVANT flip ; **self-heal
   `runtime.rs:2515` NEUTRALISÉ/gardé pendant la migration** (cf. C5, ×2 depuis A2) ; blobs in-place
   **avec test staging préalable** (pas un pari) + filet wipe **uniquement** pour les pins
   re-fetchables ; **tar snapshot** de `NEXUS_GRID_ROOT` avant 1er boot 0.101 (one-way →
   rollback = restore tar) ; **ancre VPS migrée EN DERNIER**, in-place, gardant
   `node_key`/`node_id` (re-install stock S75 **INTERDIT** : régénérerait l'identité →
   casse les locators abonnés).
5. **C5 — Le self-heal n'est PAS un backstop (correction critique du sceptique, vérifiée
   au code).** *(Portée ajustée à l'activation : neutralisation REQUISE seulement si le
   chemin in-place est retenu — cf. bloc Décisions PO C4/C5.)* Carte 1 recommandait « garder le self-heal en filet » — **dangereux et
   REJETÉ**. À `runtime.rs:2515-2528`, la branche `None` appelle `create_doc()` (namespace
   id NEUF) + `set_storage_namespace` qui écrase la ligne M8, **sans `import_ticket` de
   l'ancien ticket** → orpheline les entries sbfb-ides répliquées + casse les DocTicket
   persistés, en `warn`-only silencieux. **Reco : garder le self-heal pour le cas légitime
   (DB importée d'un autre data-dir), mais le DÉSACTIVER/garder explicitement pendant la
   fenêtre de migration redb** (sinon un échec de migration déclenche une perte silencieuse
   au lieu d'un crash diagnostiquable).
6. **C6 — MSRV : contradiction inter-cartes.** Carte 4 → bump `1.94→1.95` inconditionnel ;
   cartes 1/3 → plancher réel `1.91` (iroh-docs 0.101) **déjà franchi** par le Docker
   canonique `rust:1.94`. **Reco : vérifier empiriquement (`cargo +1.94 build` Docker
   canonique) AVANT de budgéter ; NE PAS bumper 1.95 sans preuve cargo qu'une feuille
   l'exige.**
7. **C7 — Clôture P2-AUDIT-2.** **Reco** : ne PAS pré-annoncer CLOSED. Gate = `cargo tree -d`
   post-bump montre **un seul** arbre `ed25519-dalek` + 0 `*-pre`/`*-rc` dupliqués → si
   convergent, flipper `deny.toml:107` `multiple-versions warn→deny` ; sinon lever
   **P2-AUDIT-2-RESIDUEL** (carry S82). Vérifier aussi que le `ed25519-dalek 2.x` SBFB ne
   s'effondre PAS sur l'arbre RC d'iroh.

## Scope

### In (Phase 0 + A→K, bi-axe transport + sharding, 0 defer du cœur)

- **Phase 0 — Audit gate S80 : JOUÉE le 2026-07-02.** Verdict **CONDITIONAL PASS →
  PASS effectif** (findings `dcc3eea` ; P1 unique S80-K-1 résolu `2c85b28` — 5e
  frontière docs-contrat /api/project-documents indexée). Baseline de tests FIGÉE :
  Rust nextest **2014** Win / **2018** Docker (+4 cfg(unix)) ; Vitest web **411** ;
  Vitest operator **201** (35 fichiers) ; E2E Playwright operator **10** ; T2 S80
  JSON PASS committé. Carries entrants figés (cf. §Carries + bloc Décisions PO).
- **A — Fix convergence materializer (wf4) [0-bump, AVANT le bump]** : éliminer la
  divergence `PublicRegistryView` cross-noeud sur ingest hors-ordre (`feed_materializer.rs:54-58`
  overwrite inconditionnel + `:95-101` fold non-vérifié + `public_feed.rs:588` sans
  `prev_hash`). Fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break
  `(timestamp, author, hash)` + garde monotone dans `apply()`. **Établit une baseline
  0.98 verte = bisectabilité.** Commit propre dédié, **JAMAIS** dans le commit de bump.
- **A2 — Self-heal root-cause ×2 [0-bump, vérification 02/07]** : les DEUX sites
  destructeurs (`boot_storage_namespace` `runtime.rs:2456-2549`/recreate :2518 ET le
  miroir `boot_feed_namespace` :2555-2633/recreate :2606) passent en Err→fail-fast
  diagnostiquable ; seul `Ok(None)` (cas légitime : DB importée d'un autre data-dir)
  recrée. Ferme la classe « perte silencieuse warn-only » à la racine.
- **A3 — Baseline transport LIVE 0.98 + fix WAN task-delivery (C10) [0-bump]** :
  artefact JSON b3 par palier COMMITTÉ + run Win `SBFB_INTEGRATION=1` archivé (les 5
  tests multi_daemon relay-gated early-returnent verts EN SILENCE en CI — baseline
  relais jamais mesurée) + Ollama installé sur le Mac + copie du store VPS rapatriée
  (ressource Phase F) + **fix du blocker WAN task-delivery S77** (C10 ratifié — split
  de phase possible si le préflight le juge trop gros).
- **B — Bump deps workspace + recompile mécanique** : `Cargo.toml:37-41`
  → `iroh =1.0.1` / `iroh-docs =0.101.0` / `iroh-gossip =0.101.0` / `iroh-blobs =0.103.0` ;
  cassure compile connue `pkarr_resolver.rs:40,109` `CaRootsConfig→CaTlsConfig` (#4300) +
  re-vérif `PkarrRelayClient::new(url, tls)` ; deps relogées éventuelles
  (`iroh-tickets`/`iroh-metrics`) + `irpc` 0.14→0.17 ; commentaires version ;
  `Cargo.lock` figé ; MSRV : 1.91 tranchée (toolchain 1.94 suffit — confirmation au
  build, pas de re-débat). Checkpoint gossip (pur recompile).
- **C — iroh-docs deep (wire + types iroh-base)** : wire `EntrySignature→iroh::Signature`
  (0.99.1) + types `ed25519_dalek→iroh-base` (0.100.0) + reconstruction raw-bytes
  `DocsNamespaceId::from` (`runtime.rs:2479`) ; surface `docs.rs` (AuthorId/NamespaceId/
  Entry/DocTicket/Query/ShareMode/LiveEvent), `node.rs:388-395` (`Docs::persistent/memory/
  spawn`). **Suppression actée des zombies legacy-decode** du wire redéfini (pre-launch
  policy).
- **D — iroh-blobs cascade + redb4** : recompiler `blobs.rs:85-252` sous 0.103
  (`FsStore`/`BlobsProtocol::new`/`Downloader`/tags/`HashAndFormat`/`BlobTicket`) +
  `node.rs:47-50,375-398` + valider l'ouverture du store redb4.
- **E — Surfaces fragiles transport re-cert (3 crates) + PLAN B pré-provisionné (C8)** :
  `shard.rs` (RTT/PathId — traité **UNVERIFIED-high-risk**, pas « SAUVE ») compile +
  handshake ; liste canonique des retraits rc.0 re-ancrée au préflight
  (`Connection::to_info()`→`weak_handle()`, `PathWatcher/PathInfo`→`paths()/PathList` +
  `PathEvent #[non_exhaustive]`, `Incoming::local_ip`→`local_addr`, ClientBuilder
  `query_param`→`auth_token`) ; `seed_protocol.rs`
  (`ProtocolHandler`/`AcceptError`, crate nexus-shell-daemon) ; relais (`relay_config.rs`,
  `node.rs RelayMode::Custom`, default_relay_map URLs) ; **check nommé de survie URL pkarr**
  `pkarr_resolver.rs:54` (`dns.iroh.link/pkarr`) ; re-scan call-sites sur `nexus-shell-daemon`
  + `nexus-shell-daemon-core` ; **PLAN B C8** : relais iroh self-hosted wire-compat +
  pkarr self-hosted + acceptance zéro-n0 (2-4 j) — split E' possible si le portage shard
  dépasse le mécanique.
- **F — Migration on-disk redb 2→4 validée sur COPIE** : fixture migration (store peuplé
  namespace sbfb-ides) + test ouverture blobs redb2 sous 0.103 (staging) + **garde explicite
  autour de `runtime.rs:2515-2528`** (self-heal NON déclenché en fenêtre de migration) +
  inventaire « pins re-fetchables ailleurs ? » avant toute tolérance wipe + vérif parse
  `DocTicket` (DB) + `BlobTicket` (`anchors.json`). Aucune migration LIVE ici.
- **G — CI / MSRV / convergence crypto + docs sécurité** : `cargo tree -d` (gate convergence :
  un seul arbre ed25519-dalek + 0 `*-pre`/`*-rc`) → **flip `deny.toml:107` warn→deny** OU
  carry **P2-AUDIT-2-RESIDUEL** ; rust-version DÉCLARÉE `Cargo.toml:24` 1.85→1.91 (D6
  tranchée, image CI/Docker inchangée) ; trigger de veille iroh-docs 0.102+ (wire pré-1.0
  instable) ; `cargo-deny`/`cargo-audit` verts ; amendements `THREAT_MODEL.md:22,128,195`,
  `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7`, `HARDENING_ROADMAP.md:5` (trigger iroh FIRED) ;
  **LOT-LOOPBACK-DOC (audit S80 H-1/2/3/4)** : revalidation `LOOPBACK_ENDPOINTS_TRUST_TIERS`
  §3.1 (2 routes S80 git/diff+gates + double transport cookie + description terminal/ws PTY)
  + nit §14 EventSource + `last_validated` bump ; TOOLCHAIN-LABEL (décision pin
  rust-toolchain.toml Windows ou statu quo).
- **H — Migration LIVE ancre VPS + acceptance** : runbook **tar snapshot sur les 3
  NŒUDS** AVANT restart ; **fenêtre d'incompatibilité BORNÉE** (vérification 02/07 :
  les flottes relais 0.98/1.0 diffèrent → partition possiblement totale pendant la
  fenêtre) : **flip same-day en UNE session** (ordre dev Win + Mac puis VPS) + gel
  publish/ingest + convergence vérifiée après CHAQUE nœud + re-annonce post-flip ;
  deploy binaire 1.0.1 + restart systemd ; vérif 1er boot 0 crash-loop + docs.redb
  migré + `node_id` INCHANGÉ + feed/ides/pins intacts ; rollback = restore tar
  (one-way).
- **I — Orchestrateur de session sharding in-vivo (ex-S78) [décision PO C1]** :
  construire l'orchestrateur de session in-vivo dont l'absence a rendu S77
  RIG-ABSENT (référence : `archive/v2.1/sprint78_audit_plan.md` §7/§10) — sur la
  stack iroh 1.0 migrée (après E : `shard.rs` recompilé + handshake vert). Partie
  hermétiquement testable (session lifecycle, placement, dispatch shard) couverte
  T1 ; le préflight de phase (5 scans) précise les livrables exacts depuis le
  dossier S78.
- **J — Benchmark live sharding 2-machines + T2 axe shard [décision PO C1]** :
  re-jeu b3_shard LIVE (RTX 5080 dev Win + Mac M2) via l'orchestrateur (I) sur la
  stack migrée (après H) ; verdict JSON au vocabulaire fermé
  (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT` émis par le harness préflight —
  `RIG-ABSENT` légitime UNIQUEMENT si une machine est génuinement HS, le rig
  nominal étant le même matériel que l'axe transport).
- **K — Wrap-up + gate testabilité + roadmap** : T1 hermétique BLOQUANT + CI ; artefact
  **T2 JSON committé BI-AXE** (transport-convergence + sharding) ; re-jeu acceptances
  S75 (survives-VPS-death) + S76 (b3 quorum) + **b3 PASS fetch blob cross-machine** ;
  amendement `roadmap_v5` (insertion S81-iroh bi-axe + Viewer→S82 + orchestrateur
  sharding ABSORBÉ par S81 Phases I/J) ; `SPRINT_LOG.md` + `CLAUDE.md` + mémoire
  (`nexus_grid_pivot.md`, `MEMORY.md`) + `PATTERNS.md` + `sprint82_audit_plan.md`
  (carries reroutés) + clôture docs-contrat (DoD (d), invariant #17 : frontières
  de la fenêtre S81 indexées ou `N-A-no-new-frontier` explicite) ; **libellé T1
  corrigé** (vérification 02/07 : distinguer hermétique-CI vs relay-gated-local ;
  job `SBFB_INTEGRATION=1` nightly/manuel OU couverture T2-live actée) ; **T2 par
  paliers** (C10 : quorum DANS le T2, WAN fixé en A3) ; **arbitrage slot S82
  BLOQUANT** (C9 : re-confirmer S82 = workflow-engine vs Viewer/dette).

### Out (reroutés / interdits dans S81)

- **Viewer fondation** (tools/factory-ui jeté S80) + Aperçu scellé/Proof Card → **S82**.
- **Dette docs-contract 8 P2 / 11 P3 (S80)** → **sprint dette nommé distinct** (jamais bundlé).
- **P1 in-vivo app-authoring S79** (`Not evidenced`) → reste ouvert, hors corps S81.
  (Le P1 sharding S77 RIG-ABSENT est désormais ADRESSÉ par les Phases I/J — décision
  PO C1 ; sa fermeture dépend du verdict T2 axe shard.)
- **GuardianDB** / toute autre upgrade → séparé et postérieur (bisectabilité, directive
  séquencement D7).
- **Bump MSRV 1.95** inconditionnel → INTERDIT sans preuve cargo (C6).
- **Pagination app-storage**, features produit non liées à iroh → backlog.
- **Clôture pré-annoncée P2-AUDIT-2** → INTERDITE sans `cargo tree -d` convergent (C7).

## Day-0 — décisions gelées (NE PAS re-débattre)

> Décisions D1..D8 **tranchées** (corrections sceptique intégrées). Le scoring G1
> (perspective indépendante, 0-5 par décision) vit dans `sprint81_design_review.md`.

1. **D1 — Version cible** *(amendée vérification 02/07 + re-check jour J)* : coder sur
   `iroh = "=1.0.1"` (publiée 29/06 — la condition « re-pin sur la 1re 1.0.x » de la
   reco initiale est déjà remplie). Quatuor : iroh **=1.0.1** / docs =0.101.0 /
   gossip =0.101.0 / blobs =0.103.0, bump en point unique `Cargo.toml:37-41`. Pins
   EXACTS (reproductibilité + bisectabilité ; iroh-docs pré-1.0 wire-instable) ;
   veille 1.0.2 + RustSEC jusqu'au push live.
2. **D2 — Relais / discovery post-EOL N0** : `presets::N0` par défaut (mis à jour <24 h
   après release par n0) + relais iroh self-hosted **OPTIONNEL** pour l'ancre VPS comme
   résilience. Escape-hatch déjà câblé (`node.rs:329,348`, `SBFB_CUSTOM_RELAYS`). **Note
   BLOQUANTE** : vérifier explicitement la survie de l'URL pkarr `pkarr_resolver.rs:54`
   (le blog n0 avertit « wire-breaking relay changes get new URLs ») — sinon discovery
   casse **silencieusement** (pas de crash). Check nommé, jamais plié dans « recompile ».
3. **D3 — Migration données on-disk redb 2→4** *(assouplie C4/C5 PO « personne sur le
   réseau » — les conditions ci-dessous restent le chemin PRÉFÉRÉ, les préflights F/H
   tranchent ; amendée vérification 02/07 : la migration est AUTOMATIQUE à l'ouverture,
   iroh-docs PR #105, saut réel redb ^2.6.3→^4.1 — aucune « feature redb-v2-migration »
   n'existe)* : **hybride durci**. docs.redb = IN-PLACE préférée (saut 0.98→0.101
   DIRECT) ; coordinator SQLite (M18, public_feed) intact ; blobs in-place **avec test
   staging** ; wipe toléré au-delà des pins re-fetchables si l'in-place résiste (C4/C5).
   Conditions (chemin in-place) : (1) saut direct, jamais 0.99/0.100 contre
   l'ancien store ; (2) wipe docs évité ; (3) tar snapshot avant 1er boot 0.101
   (one-way) ; (4) validation sur COPIE du store VPS peuplé ; (5) ancre VPS in-place
   gardant node_key/node_id ; (6) fixture redb 2→4 dans
   T1 ; **(7, ajout sceptique + vérification ×2) self-heal neutralisé/gardé pendant la
   migration — ce n'est PAS un backstop, et il y a DEUX sites : `boot_storage_namespace`
   `runtime.rs:2456-2549` (recreate :2518) ET le miroir `boot_feed_namespace`
   :2555-2633 (recreate :2606), traités root-cause en Phase A2.**
4. **D4 — Stratégie test / convergence** : T1 hermétique convergence in-process
   (`multi_daemon` loopback/`MemoryLookup`) BLOQUANT Win natif + CI Linux (**jamais
   Docker-on-Windows** car `multi_daemon` env-bloqué `create_node` hang) ; T2 acceptance
   JSON LIVE **BI-AXE** (amendé décision PO C1 2026-07-02). **Split verdict** : l'axe
   **transport** (VPS Hetzner + dev Win + Mac M2) existe et PEUT atteindre PASS (S75
   survives-VPS-death = LIVE PASS, LAN Win↔Mac validé) — `RIG-ABSENT` ILLÉGITIME sur
   cet axe ; l'axe **sharding** entre au T2 : l'orchestrateur in-vivo (cause racine des
   RIG-ABSENT S76/S77) est LIVRÉ en Phase I, le benchmark live joué en Phase J sur le
   MÊME matériel que l'axe transport (5080 + M2) — `RIG-ABSENT` n'y est légitime que si
   une machine est génuinement HS (émis par le harness préflight, jamais en prose).
5. **D5 — Scope fix materializer (wf4)** : **IN S81, Phase A, AVANT le bump, commit propre
   dédié**. Fix = fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break
   `(timestamp, author, hash)` + garde monotone dans `apply()` (`feed_materializer.rs:54-58`)
   + `verify_entry` check `prev_hash` (`public_feed.rs:588`). Tranche le conflit Carte 1
   (sprint séparé) vs cartes 3/4 (in-sprint) **en faveur de l'in-sprint** ; baseline 0.98
   verte = bisectabilité préservée. Jamais mélangé au commit de migration.
6. **D6 — MSRV + sweep deps feuilles** *(TRANCHÉE vérification 02/07)* :
   `rust_version=1.91` confirmé crates.io pour les 5 crates → toolchain 1.94 SUFFIT,
   image CI/Docker inchangée ; résiduel Phase G = bump de la rust-version DÉCLARÉE
   `Cargo.toml:24` 1.85→1.91. **P2-AUDIT-2 reste OUVERT (résiduel)** jusqu'à
   `cargo tree -d` convergent. Gate convergence crypto = `deny.toml:107` flip si un
   seul arbre ed25519-dalek + 0 `*-pre`/`*-rc` ; sinon carry **P2-AUDIT-2-RESIDUEL**.
7. **D7 — Carries + roadmap (séquencement)** : S81 = iroh **STRICTEMENT SEUL** ; re-scanner
   les call-sites sur les 3 crates (core + 2 daemon), pas seulement nexus-core-rs ; amender
   roadmap_v5 (insertion S81-iroh + Viewer→S82) ; orchestrateur sharding séquencé APRÈS
   S81. Blast-radius = 3 crates déclarent iroh (bump = point unique, mais call-sites API
   débordent côté daemon — `ProtocolHandler` seed). Clôture P2-AUDIT-2 GATÉE par D6.
8. **D8 — R-iroh-audit / posture release** : **R-iroh-audit P0 INCHANGÉ.** L'upgrade NE
   franchit PAS Gate 1/Gate 3, NE débloque PAS le pilote public ferme. **Maintenance
   forcing-function-driven, pas levée de zone rouge.** iroh 1.0 = 0 audit tiers public. Le
   wire-freeze 1.0 réduit le churn de la surface de désérialisation (THREAT_MODEL menace E)
   = neutre-à-positif, jamais un durcissement de confiance. Libellé explicite « upgrade ≠
   Gate 1 » obligatoire dans kickoff + commit body.
9. **Pre-launch policy — borne tracée** : la politique « wire modifiable librement avant
   v1.0 » **ne couvre PAS** le store on-disk `iroh-docs`/`blobs` déjà déployé (ancre VPS).
   La migration redb 2→4 est **one-way** ; le rollback = restore tar. Les zombies
   legacy-decode du wire iroh-docs redéfini sont **supprimés immédiatement** (chaque
   deletion actée au body de commit).
10. **iroh STRICTEMENT SEUL (anti-bundle)** : aucun autre upgrade (Viewer, dette, GuardianDB)
    dans S81 ; materializer en Phase A commit séparé AVANT bump ; tout le reste reroutés
    (D7). Bisectabilité = invariant cardinal du sprint.

## Gate de testabilité par-sprint (README §4, NON-négociable)

- **T1 — Hermétique, BLOQUANT** (Win natif + CI Linux Woodpecker/GHA ; **JAMAIS
  Docker-on-Windows** car `multi_daemon` env-bloqué `create_node` hang) :
  1. **Convergence in-process** : `multi_daemon` 2-noeuds loopback / `MemoryLookup` sur la
     stack migrée — doc-sync (wire iroh-docs migré) + gossip + blobs fetch + seed ALPN
     `sbfb/seed/0` handshake + ingest annuaire.
  2. **Convergence ingest hors-ordre** : assert `PublicRegistryView` identique cross-fold
     quel que soit l'ordre d'arrivée (couvre le fix materializer Phase A — l'assertion
     centrale).
  3. **Fixture migration redb 2→4** : ouvrir un `docs.redb` redb2 peuplé (namespace
     sbfb-ides) sous 0.101 → entries survivent, namespace id INCHANGÉ, **self-heal non
     déclenché** ; ouvrir un store blobs redb2 sous 0.103.
  4. **Parse tickets persistés** : `DocTicket` (string DB) + `BlobTicket` (`anchors.json`)
     re-parsent post-migration.
  5. **Recompile + handshake shard** : `shard.rs` compile + handshake `sbfb/shard/1`
     in-process (PAS le RTT/multipath live).
- **T2 — Acceptance JSON cross-machine committé** (`PASS` / `BLOCK{diagnosis}` / `RIG-ABSENT`),
  **BI-AXE (décision PO C1 2026-07-02)** :
  - **Axe transport (PASS obligatoire)** : rig réel VPS Hetzner + dev Win + Mac M2
    — re-jeu **S75 survives-VPS-death** + **S76 b3 quorum** + **b3 PASS fetch blob
    cross-machine** post-upgrade ; convergence `PublicRegistryView` cross-noeud après
    migration LIVE. **`RIG-ABSENT` illégitime sur cet axe** (rig confirmé dispo,
    `live_acceptance_setup`) ; seul un rig **génuinement HS** le justifie.
  - **Axe sharding (DANS S81, Phases I/J)** : b3_shard LIVE via l'orchestrateur de
    session in-vivo (Phase I, ex-S78) sur la stack migrée, rig RTX 5080 + Mac M2
    (même matériel que l'axe transport). Verdict au vocabulaire fermé émis par le
    harness préflight ; `RIG-ABSENT` légitime UNIQUEMENT si une machine est
    génuinement HS — l'excuse « orchestrateur absent » disparaît (Phase I le livre).

## Invariants

- **Aucune perte de données live** : la migration redb 2→4 one-way préserve entries
  sbfb-ides + namespace id + DocTicket + pins re-fetchables ; validée sur COPIE (F) AVANT
  flip ; tar snapshot avant 1er boot (H) ; self-heal `runtime.rs:2515` neutralisé pendant
  la migration (ce n'est PAS un backstop : `create_doc()` namespace NEUF sans `import_ticket`).
- **Identité du noeud préservée** : l'ancre VPS migre in-place gardant `node_key`/`node_id` ;
  re-install stock S75 INTERDIT sur ancre live (régénérerait l'identité → casse les locators
  abonnés). `heberger != publier, seeder != auteur` tenu.
- **Bisectabilité** : iroh STRICTEMENT SEUL ; materializer en Phase A commit séparé AVANT
  bump (baseline 0.98 verte) ; tout le reste reroutés. Un échec post-bump = iroh, pas le
  materializer.
- **Discovery jamais cassé silencieusement** : check nommé de survie URL pkarr
  (`pkarr_resolver.rs:54`) + default_relay_map sous 1.0, pré-flip, jamais plié dans
  « recompile ».
- **0 bump wire SBFB** : JCS/DOMAIN_*_V1/FEED_FORMAT_VERSION inchangés ; le bump iroh-docs
  ne touche pas le canonical SBFB. Zombies legacy-decode du wire iroh redéfini supprimés
  (chaque deletion actée au body).
- **Total de tests interdit de descendre silencieusement** : delta net global attendu
  **+10..20 Rust** (deletions zombies actées) ; chute = justification obligatoire au body.
- **upgrade ≠ durcissement** : R-iroh-audit P0 inchangé, pilote reste ferme ; libellé
  explicite dans kickoff + commit body + docs sécurité (Phase G).
- **Frozen tenu** : Factory hors daemon ; browser = client ; AGPL-3.0 ; 0 dépendance
  non-permissive réintroduite par le bump.
- **Discipline commit** : 1 commit par phase `feat(scope): Sprint 81 Phase X — titre`, body
  riche (delta tests cumulé + scope cuts) ; deep preflight (5 scans) → review → Codex avant
  CHAQUE commit (rigueur per-phase uniforme).

## Questions ouvertes — à trancher au preflight de phase (défauts recommandés)

> Les arbitrages load-bearing (C1..C10) sont TRANCHÉS ci-dessus (bloc Décisions PO +
> Day-0 D1..D8). Les points suivants sont des détails de preflight ; défaut recommandé
> entre parenthèses. *(3 questions du staging sont RÉSOLUES par la vérification 02/07 :
> MSRV → tranchée 1.91 [D6] ; feature redb → n'existe pas, auto-migration #105 [C4-bis] ;
> re-pin 1.0.x → 1.0.1 publiée, pinnée d'entrée [C3/D1].)*

- **[D]** Changelog iroh-blobs 0.101→0.103 non détaillé côté signatures — *découvrir au
  compile, documenter tout break ; valider l'ouverture redb4 sur store dev existant.*
- **[E]** Survie de l'URL pkarr `dns.iroh.link/pkarr` + default_relay_map sous 1.0 — *check
  nommé pré-flip ; le plan B C8 (relais + pkarr self-hosted) est désormais OBLIGATOIRE,
  plus optionnel.*
- **[F]** Préflight F lit le CODE upstream de l'auto-migration (iroh-docs PR #105 :
  atomicité, comportement crash mid-migration) + ressource staging (pull du store VPS
  live vers dev + fixture namespace peuplée, rapatriée dès A3) — *pré-requis explicites.*
- **[F]** Tolérance wipe : inventaire « ce pin est-il re-fetchable ailleurs ? » —
  *chemin simple autorisé (C4/C5 PO) ; préférence in-place si coût raisonnable.*
- **[F]** Trancher l'incohérence de nommage `sbfb-ides` vs `sbfb-ideas` AU CODE
  (héritée du staging, jamais tranchée).
- **[G/C7]** `cargo tree -d` converge-t-il (un seul arbre ed25519-dalek, 0 `*-pre`/`*-rc`) ?
  — *si oui flip `deny.toml:107` warn→deny ; sinon carry P2-AUDIT-2-RESIDUEL, NE PAS annoncer
  CLOSED ; vérifier que le 2.x SBFB ne s'effondre pas sur l'arbre RC d'iroh.*
- **[H]** Ordre de rollout (dev Win + Mac d'abord, VPS EN DERNIER) + runbook tar-restore
  testé — *bloquer la migration VPS tant que la validation sur copie (F) n'est pas PASS.*

## Carries entrants

> *Liste FIGÉE à la Phase 0 (audit gate S80 joué le 2026-07-02, findings `dcc3eea`).
> S'y ajoutent les carries d'audit S80 actés au bloc Décisions PO : LOT-LOOPBACK-DOC
> → Phase G ; TOOLCHAIN-LABEL → préflight Phase G ; DELTA-DISCIPLINE → wrap-up K ;
> DOC-LINT-SEMANTIC = ACCEPT-AND-CLOSE acté ; TRAILER = accepté ; S79-P2-1 ancres →
> sprint dette.*

- **2 carries P1 in-vivo OUVERTS** : sharding S77 RIG-ABSENT — **ADRESSÉ par les
  Phases I/J** (décision PO C1) ; app-authoring S79 « Not evidenced » — standing,
  hors corps S81.
- **Viewer fondation + Aperçu scellé/Proof Card** (tools/factory-ui jeté S80) — réservés
  S81 à l'origine, **reroutés S82** (D7).
- **8 P2 / 11 P3 docs-contract S80** — **sprint dette nommé distinct** (jamais bundlé).
- **Régression couverture** (perte Vitest factory-operator + factory-ui) — re-couverte S80
  Phase I (*à confirmer à la clôture S80*).
- **P2-AUDIT-2** (pin transitif iroh) — **traité par S81 mais NON pré-clôturé** (cf. C7/D6) ;
  devient **P2-AUDIT-2-RESIDUEL** carry S82 si `cargo tree -d` ne converge pas.
- **Externes inchangés** : iframe Rust-wasm (§P34), P3-OS-1 ; LT-2 Radicle ARMÉ (flip =
  décision PO hors-sprint).

## Carries sortants (S81 → S82)

- **Viewer fondation** + Aperçu scellé/Proof Card.
- **P2-AUDIT-2-RESIDUEL** si `cargo tree -d` ne converge pas (sinon CLOSED en Phase G).
- 8 P2 / 11 P3 docs-contract → sprint dette nommé.
- 2 P1 in-vivo restent standing.
- Tout P2/P3 issu des phase-reviews S81 → `sprint82_audit_plan.md`.

## Amendement roadmap (à acter Phase K)

- Roadmap v5 (CANON) **s'arrête à S77** ; S78/79/80 sont déjà des amendements. **Insérer
  S81-iroh** (upgrade transport bi-axe, non-planifié, forcing-function 2026-09-30).
- **Viewer → S82.**
- **Orchestrateur sharding (ex-S78) ABSORBÉ par S81** (Phases I/J, décision PO C1
  2026-07-02) ; le S78 différé se solde dans S81.
- Tracer : « la pre-launch policy *wire modifiable librement* ne couvre PAS le store
  on-disk iroh-docs/blobs déjà déployé ».

## Références (chemins absolus)

- **Pin iroh + MSRV + crypto** : `C:\Users\FlowUP\Documents\Code\nexus\Cargo.toml:24,37-41,58`.
- **Déclarations directes iroh (3 crates)** :
  `crates\nexus-core-rs\Cargo.toml:19-22` (les 4),
  `crates\nexus-shell-daemon\Cargo.toml:78,84` (iroh-blobs + iroh, **PAS wrapper-only** —
  `seed_protocol` impl `ProtocolHandler`),
  `crates\nexus-shell-daemon-core\Cargo.toml:179,186` (dev-deps).
- **Self-heal destructeur (NON backstop, ×2 — vérification 02/07)** :
  `crates\nexus-shell-daemon\src\runtime.rs` — `boot_storage_namespace` `:2456-2549`
  (recreate `:2518`) ET miroir `boot_feed_namespace` `:2555-2633` (recreate `:2606`).
- **Bug materializer (Phase A)** :
  `crates\nexus-coordinator-rs\src\feed_materializer.rs:45-115` (overwrite `:54-58`, fold
  `:95-101`),
  `crates\nexus-coordinator-rs\src\public_feed.rs:585-603` (sans `prev_hash` `:588-591`).
- **Surfaces transport** : `crates\nexus-core-rs\src\` : `shard.rs`, `seed_protocol.rs`,
  `pkarr_resolver.rs` (`:40,54,109`), `relay_config.rs` (`:17-20`), `node.rs` (`:318,329,348,388-395`),
  `docs.rs`, `blobs.rs:85-252`, `discovery.rs`.
- **Docs sécurité (Phase G)** : `docs\security\THREAT_MODEL.md:22,128,195`,
  `docs\security\EXTERNAL_AUDIT_SCOPE.md` §2.4/§2.7, `docs\security\HARDENING_ROADMAP.md:5`.
- **Supply-chain** : `deny.toml:107` (`multiple-versions warn→deny`).
- **Roadmap (amendement Phase I)** : `.planning\roadmap_v5_factory_complete_vision.md`.
- **Setup acceptance live (T2 axe transport)** : memory `live_acceptance_setup` (cibles SSH
  vps/mac, PROJECT_ID, auth `x-sbfb-token`).
- **Audit gate S80** : `.planning\active\sprint80_audit_plan.md` (Phase 0, à jouer à
  l'ouverture réelle).