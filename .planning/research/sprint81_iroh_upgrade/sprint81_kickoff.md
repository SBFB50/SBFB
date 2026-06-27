# Sprint 81 — Kickoff : Upgrade iroh 0.98 → 1.0 + sweep deps **(DRAFT DE STAGING)**

> **⚠ STATUT : DRAFT DE STAGING — S80 N'EST PAS ENCORE CLOS.** Ce kickoff est
> rédigé en avance pour staging S81 ; il ne s'active qu'à la clôture effective de
> S80. **Phase 0 = audit gate S80** (jouée à l'ouverture réelle, convention
> permanente) — son verdict figera la liste exacte des carries entrants (cf.
> §Carries) et la baseline de tests. **Tout ce qui dépend de S80 est marqué
> *provisoire*.** Sprint de **maintenance d'infrastructure forcing-function-driven**
> (insertion roadmap non-planifiée) : migrer toute la pile iroh `0.98 → 1.0` GA
> avant le cutoff relais N0 du **2026-09-30**, **sans perte** des données live
> (ancre VPS Hetzner S75, store `iroh-docs` sbfb-ides, pins `keep_online` M18),
> et prouver la convergence transport cross-machine par un re-jeu **LIVE**.
> **Décision-grade, pas rubber-stamp** : faits re-vérifiés au code ce jour
> (2026-06-27) ; corrections du sceptique **intégrées** (claims réfutés retirés) ;
> contradictions inter-cartes tranchées (cf. §Arbitrages PO + §Day-0).

**Écrit** : 2026-06-27 (staging — pré-clôture S80).
**Type** : **sprint de maintenance d'infrastructure** (upgrade transport ; orthogonal
au produit utilisateur — n'ajoute aucune feature).
Le travail touche **3 crates déclarant iroh en direct** (`nexus-core-rs`,
`nexus-shell-daemon`, `nexus-shell-daemon-core` dev-deps), point de bump unique
`Cargo.toml:37-41`, plus le fix convergence materializer 0-bump dans
`nexus-coordinator-rs` (Phase A) + 2 migrations on-disk redb 2→4 + une migration
LIVE de l'ancre VPS.
**Budget de phases** : Phase 0 (audit gate S80) + **A→I** (le nombre de phases n'est
jamais plafonné, README §4 ; dimensionné par le travail, JAMAIS par LOC). Rigueur
per-phase **uniforme** : deep preflight (5 scans) + review + Codex à **CHAQUE** phase.
**Numéro/version archive** : **S81**, v2.1 (OPEN) — *provisoire, dépend de la clôture S80*.

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
scopé sur l'axe TRANSPORT-convergence uniquement** (doc-sync/gossip/blobs/seed/annuaire) ;
l'axe **sharding** (re-cert live `shard.rs` RTT/PathId multipath, rig GPU chroniquement
absent) est **hors T2 de S81** et reporté S82 (cf. Arbitrage C1/D4).

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
   sécurité, **sous rigueur per-phase uniforme**. **Reco : scoper le DONE sur l'axe
   TRANSPORT-convergence uniquement**, **sortir l'axe SHARDING du T2** (re-cert live shard
   → S82). Sans ce cadrage, S81 finit PROVISIONAL sur l'axe shard exactement comme S77.
2. **C2 — Scope du fix materializer (wf4).** Contradiction inter-cartes : Carte 1 D8 →
   « sprint séparé postérieur » ; Cartes 3/4 + sceptique → **Phase A DANS S81, AVANT le
   bump, commit propre séparé**. **Reco : Phase A in-sprint.** Raison : le bug est
   convergence-critique et 0-bump (logique coordinator SQLite, indépendante d'iroh) ; le
   **gate de convergence cross-machine de S81 le révèle de toute façon** ; le corriger
   AVANT le bump établit une baseline 0.98 verte → préserve la bisectabilité (un échec
   post-bump = iroh, pas le materializer). **Discipline imposée** : jamais dans le commit
   de bump.
3. **C3 — Version cible.** **Correction intégrée (carte 3 + sceptique)** : « viser la
   dernière 1.0.x » est **actuellement insatisfiable** — au 2026-06-27, `1.0.0` (12 j)
   est la **seule** stable, **aucune 1.0.x patch n'existe**. **Reco : coder sur
   `iroh = "=1.0.0"` maintenant + re-pin OBLIGATOIRE sur la 1re 1.0.x publiée AVANT le
   push live** ; si aucune patch au code-freeze → soak documenté + veille RustSEC
   (interdiction de pousser la .0 brute si une patch existe). Le runway ~3 mois interdit
   d'attendre passivement.
4. **C4 — Stratégie données-live.** **Reco (hybride durcie)** : migration **IN-PLACE
   impérative** pour `docs.redb` (saut 0.98→0.101 DIRECT, feature défaut
   `redb-v2-migration`), validée sur **COPIE** du store VPS peuplé AVANT flip ; **self-heal
   `runtime.rs:2515` NEUTRALISÉ/gardé pendant la migration** (cf. C5) ; blobs in-place
   **avec test staging préalable** (pas un pari) + filet wipe **uniquement** pour les pins
   re-fetchables ; **tar snapshot** de `NEXUS_GRID_ROOT` avant 1er boot 0.101 (one-way →
   rollback = restore tar) ; **ancre VPS migrée EN DERNIER**, in-place, gardant
   `node_key`/`node_id` (re-install stock S75 **INTERDIT** : régénérerait l'identité →
   casse les locators abonnés).
5. **C5 — Le self-heal n'est PAS un backstop (correction critique du sceptique, vérifiée
   au code).** Carte 1 recommandait « garder le self-heal en filet » — **dangereux et
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

### In (Phase 0 + A→I, axe TRANSPORT, 0 defer du cœur transport)

- **Phase 0 — Audit gate S80** (jouée à l'ouverture réelle) : absorbe le verdict S80, fige
  la liste exacte des carries entrants et la baseline de tests. Le corps S81 (A→I) ne
  démarre qu'**après Phase 0 PASS**.
- **A — Fix convergence materializer (wf4) [0-bump, AVANT le bump]** : éliminer la
  divergence `PublicRegistryView` cross-noeud sur ingest hors-ordre (`feed_materializer.rs:54-58`
  overwrite inconditionnel + `:95-101` fold non-vérifié + `public_feed.rs:588` sans
  `prev_hash`). Fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break
  `(timestamp, author, hash)` + garde monotone dans `apply()`. **Établit une baseline
  0.98 verte = bisectabilité.** Commit propre dédié, **JAMAIS** dans le commit de bump.
- **B — Bump deps workspace + recompile mécanique + MSRV empirique** : `Cargo.toml:37-41`
  → `iroh 1.0.0` / `iroh-docs 0.101.0` / `iroh-gossip 0.101.0` / `iroh-blobs 0.103.0` ;
  cassure compile connue `pkarr_resolver.rs:40,109` `CaRootsConfig→CaTlsConfig` (#4300) +
  re-vérif `PkarrRelayClient::new(url, tls)` ; commentaires version ; `Cargo.lock` figé ;
  vérif `cargo +1.94 build` Docker canonique (décision MSRV, C6). Checkpoint gossip (pur
  recompile).
- **C — iroh-docs deep (wire + types iroh-base)** : wire `EntrySignature→iroh::Signature`
  (0.99.1) + types `ed25519_dalek→iroh-base` (0.100.0) + reconstruction raw-bytes
  `DocsNamespaceId::from` (`runtime.rs:2479`) ; surface `docs.rs` (AuthorId/NamespaceId/
  Entry/DocTicket/Query/ShareMode/LiveEvent), `node.rs:388-395` (`Docs::persistent/memory/
  spawn`). **Suppression actée des zombies legacy-decode** du wire redéfini (pre-launch
  policy).
- **D — iroh-blobs cascade + redb4** : recompiler `blobs.rs:85-252` sous 0.103
  (`FsStore`/`BlobsProtocol::new`/`Downloader`/tags/`HashAndFormat`/`BlobTicket`) +
  `node.rs:47-50,375-398` + valider l'ouverture du store redb4.
- **E — Surfaces fragiles transport re-cert (3 crates)** : `shard.rs` (RTT/PathId —
  traité **UNVERIFIED-high-risk**, pas « SAUVE ») compile + handshake ; `seed_protocol.rs`
  (`ProtocolHandler`/`AcceptError`, crate nexus-shell-daemon) ; relais (`relay_config.rs`,
  `node.rs RelayMode::Custom`, default_relay_map URLs) ; **check nommé de survie URL pkarr**
  `pkarr_resolver.rs:54` (`dns.iroh.link/pkarr`) ; re-scan call-sites sur `nexus-shell-daemon`
  + `nexus-shell-daemon-core`.
- **F — Migration on-disk redb 2→4 validée sur COPIE** : fixture migration (store peuplé
  namespace sbfb-ides) + test ouverture blobs redb2 sous 0.103 (staging) + **garde explicite
  autour de `runtime.rs:2515-2528`** (self-heal NON déclenché en fenêtre de migration) +
  inventaire « pins re-fetchables ailleurs ? » avant toute tolérance wipe + vérif parse
  `DocTicket` (DB) + `BlobTicket` (`anchors.json`). Aucune migration LIVE ici.
- **G — CI / MSRV / convergence crypto + docs sécurité** : `cargo tree -d` (gate convergence :
  un seul arbre ed25519-dalek + 0 `*-pre`/`*-rc`) → **flip `deny.toml:107` warn→deny** OU
  carry **P2-AUDIT-2-RESIDUEL** ; image CI/Docker + `Cargo.toml:24` rust-version **seulement
  si** D6 l'exige ; `cargo-deny`/`cargo-audit` verts ; amendements `THREAT_MODEL.md:22,128,195`,
  `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7`, `HARDENING_ROADMAP.md:5` (trigger iroh FIRED).
- **H — Migration LIVE ancre VPS + acceptance** : runbook **tar snapshot** `NEXUS_GRID_ROOT`
  AVANT restart ; ordre **dev Win + Mac d'abord, VPS EN DERNIER** ; deploy binaire 1.0.x +
  restart systemd ; vérif 1er boot 0 crash-loop + docs.redb migré + `node_id` INCHANGÉ +
  feed/ides/pins intacts ; rollback = restore tar (one-way).
- **I — Wrap-up + gate testabilité + roadmap** : T1 hermétique BLOQUANT + CI ; artefact
  **T2 JSON committé** (transport-convergence) ; re-jeu acceptances S75 (survives-VPS-death)
  + S76 (b3 quorum) + **b3 PASS fetch blob cross-machine** ; amendement `roadmap_v5`
  (insertion S81-iroh + Viewer→S82 + orchestrateur sharding après S81) ; `SPRINT_LOG.md` +
  `CLAUDE.md` + mémoire (`nexus_grid_pivot.md`, `MEMORY.md`) + `PATTERNS.md` +
  `sprint82_audit_plan.md` (carries reroutés).

### Out (reroutés / interdits dans S81)

- **Re-cert LIVE du sharding** (`shard.rs` RTT/PathId multipath noq, RIG-ABSENT,
  non-hermétique) → **S82** (après orchestrateur ex-S78). On ne « re-joue » PAS une
  acceptance (S77 b3_shard) qui n'a **jamais** passé.
- **Viewer fondation** (tools/factory-ui jeté S80) + Aperçu scellé/Proof Card → **S82**.
- **Dette docs-contract 8 P2 / 11 P3 (S80)** → **sprint dette nommé distinct** (jamais bundlé).
- **2 P1 in-vivo standing** (sharding S77, app-authoring S79) → restent ouverts, hors corps S81.
- **GuardianDB** / toute autre upgrade → séparé et postérieur (bisectabilité, directive
  séquencement D7).
- **Bump MSRV 1.95** inconditionnel → INTERDIT sans preuve cargo (C6).
- **Pagination app-storage**, features produit non liées à iroh → backlog.
- **Clôture pré-annoncée P2-AUDIT-2** → INTERDITE sans `cargo tree -d` convergent (C7).

## Day-0 — décisions gelées (NE PAS re-débattre)

> Décisions D1..D8 **tranchées** (corrections sceptique intégrées). Le scoring G1
> (perspective indépendante, 0-5 par décision) vit dans `sprint81_design_review.md`.

1. **D1 — Version cible** : coder sur `iroh = "=1.0.0"` maintenant ; re-pin sur la 1re
   1.0.x patch **AVANT push live** ; sinon soak + veille RustSEC documentés. Quatuor :
   iroh 1.0.0 / docs 0.101.0 / gossip 0.101.0 / blobs 0.103.0, bump en point unique
   `Cargo.toml:37-41`. « Viser la dernière 1.0.x » est insatisfiable (0 patch publiée) →
   reco réécrite en « 1.0.0 + re-pin conditionnel ». Risque .0-fraîche (12 j) réel → pin
   exact, re-pin obligatoire.
2. **D2 — Relais / discovery post-EOL N0** : `presets::N0` par défaut (mis à jour <24 h
   après release par n0) + relais iroh self-hosted **OPTIONNEL** pour l'ancre VPS comme
   résilience. Escape-hatch déjà câblé (`node.rs:329,348`, `SBFB_CUSTOM_RELAYS`). **Note
   BLOQUANTE** : vérifier explicitement la survie de l'URL pkarr `pkarr_resolver.rs:54`
   (le blog n0 avertit « wire-breaking relay changes get new URLs ») — sinon discovery
   casse **silencieusement** (pas de crash). Check nommé, jamais plié dans « recompile ».
3. **D3 — Migration données on-disk redb 2→4** : **hybride durci**. docs.redb = IN-PLACE
   impérative (saut 0.98→0.101 DIRECT) ; coordinator SQLite (M18, public_feed) intact ;
   blobs in-place **avec test staging** ; wipe blobs toléré **uniquement** pour pins
   re-fetchables. Conditions cumulatives : (1) saut direct, jamais 0.99/0.100 contre
   l'ancien store ; (2) wipe docs INTERDIT ; (3) tar snapshot avant 1er boot 0.101
   (one-way) ; (4) validation sur COPIE du store VPS peuplé ; (5) ancre VPS in-place
   gardant node_key/node_id (re-install stock S75 INTERDIT) ; (6) fixture redb 2→4 dans
   T1 ; **(7, ajout sceptique) self-heal `runtime.rs:2515` neutralisé/gardé pendant la
   migration — ce n'est PAS un backstop.**
4. **D4 — Stratégie test / convergence** : T1 hermétique convergence in-process
   (`multi_daemon` loopback/`MemoryLookup`) BLOQUANT Win natif + CI Linux (**jamais
   Docker-on-Windows** car `multi_daemon` env-bloqué `create_node` hang) ; T2 acceptance
   JSON LIVE transport-convergence PASS sur rig VPS+Win+Mac. **Split verdict (sceptique)** :
   l'axe **transport** (VPS Hetzner + dev Win + Mac M2) existe et PEUT atteindre PASS (S75
   survives-VPS-death = LIVE PASS, LAN Win↔Mac validé) ; l'axe **sharding** (rig GPU
   5080+M2 + orchestrateur in-vivo) était **chroniquement absent** (S76 DIFFERE, S77
   RIG-ABSENT, orchestrateur reporté S78) → **hors T2 de S81**. `RIG-ABSENT` illégitime
   **sur l'axe transport** ; légitime/inapplicable sur l'axe sharding (reporté S82).
5. **D5 — Scope fix materializer (wf4)** : **IN S81, Phase A, AVANT le bump, commit propre
   dédié**. Fix = fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break
   `(timestamp, author, hash)` + garde monotone dans `apply()` (`feed_materializer.rs:54-58`)
   + `verify_entry` check `prev_hash` (`public_feed.rs:588`). Tranche le conflit Carte 1
   (sprint séparé) vs cartes 3/4 (in-sprint) **en faveur de l'in-sprint** ; baseline 0.98
   verte = bisectabilité préservée. Jamais mélangé au commit de migration.
6. **D6 — MSRV + sweep deps feuilles** : vérifier MSRV empiriquement (`cargo +1.94 build`
   Docker) ; rester 1.94 sauf preuve cargo qu'une feuille exige plus ; **P2-AUDIT-2 reste
   OUVERT (résiduel)** jusqu'à `cargo tree -d` convergent. Tranche la contradiction Carte 4
   (1.95 inconditionnel) vs cartes 1/3 (plancher 1.91 déjà franchi) **en faveur de
   l'empirique**. Gate convergence crypto = `deny.toml:107` flip si un seul arbre
   ed25519-dalek + 0 `*-pre`/`*-rc` ; sinon carry **P2-AUDIT-2-RESIDUEL**.
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
- **T2 — Acceptance JSON cross-machine committé** (`PASS` / `BLOCK{diagnosis}` / `RIG-ABSENT`) :
  - **Axe transport (DANS S81, PASS obligatoire)** : rig réel VPS Hetzner + dev Win + Mac M2
    — re-jeu **S75 survives-VPS-death** + **S76 b3 quorum** + **b3 PASS fetch blob
    cross-machine** post-upgrade ; convergence `PublicRegistryView` cross-noeud après
    migration LIVE. **`RIG-ABSENT` illégitime sur cet axe** (rig confirmé dispo,
    `live_acceptance_setup`) ; seul un rig **génuinement HS** le justifie.
  - **Axe sharding (HORS S81)** : `shard.rs` RTT/PathId multipath noq + orchestrateur
    in-vivo = **non testable hermétiquement, rig GPU chroniquement absent** → **reporté S82**
    (après orchestrateur ex-S78). On ne re-joue PAS une acceptance (S77 b3_shard) jamais passée.

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

> Les arbitrages load-bearing (C1..C7) sont TRANCHÉS ci-dessus (Day-0 D1..D8). Les points
> suivants sont des détails de preflight ; défaut recommandé entre parenthèses.

- **[B/D6]** Plancher MSRV réel après bump (`cargo +1.94 build` Docker canonique) — *à
  mesurer AVANT de budgéter Phase G ; rester 1.94 si la preuve cargo le permet, bump 1.95
  INTERDIT sans feuille qui l'exige.*
- **[D]** Changelog iroh-blobs 0.101→0.103 non détaillé côté signatures — *découvrir au
  compile, documenter tout break ; valider l'ouverture redb4 sur store dev existant.*
- **[E]** Survie de l'URL pkarr `dns.iroh.link/pkarr` + default_relay_map sous 1.0 — *check
  nommé pré-flip ; provisionner un relais iroh self-hosted optionnel pour l'ancre (résilience).*
- **[F]** Feature flag de migration redb (`redb-v2-migration` défaut on ?) + ressource
  staging (pull du store VPS live vers dev + fixture namespace peuplée) — *provisionner le
  pull du store + fixture comme pré-requis explicite de Phase F.*
- **[F]** Tolérance wipe blobs : inventaire « ce pin est-il re-fetchable ailleurs ? » —
  *wipe toléré UNIQUEMENT pour les pins re-fetchables ; jamais pour docs.*
- **[C3]** Re-pin 1.0.x : existe-t-il une patch au code-freeze ? — *si oui re-pin
  obligatoire AVANT push live ; si non, soak + veille RustSEC documentés (interdiction de
  pousser la .0 brute si une patch existe).*
- **[G/C7]** `cargo tree -d` converge-t-il (un seul arbre ed25519-dalek, 0 `*-pre`/`*-rc`) ?
  — *si oui flip `deny.toml:107` warn→deny ; sinon carry P2-AUDIT-2-RESIDUEL, NE PAS annoncer
  CLOSED ; vérifier que le 2.x SBFB ne s'effondre pas sur l'arbre RC d'iroh.*
- **[H]** Ordre de rollout (dev Win + Mac d'abord, VPS EN DERNIER) + runbook tar-restore
  testé — *bloquer la migration VPS tant que la validation sur copie (F) n'est pas PASS.*

## Carries entrants

> *Liste entrante provisoire — figée à la clôture S80 (Phase 0).*

- **2 carries P1 in-vivo OUVERTS** (sharding S77 RIG-ABSENT, app-authoring S79 « Not
  evidenced ») — **standing**, hors corps S81 (un upgrade transport ne les ferme pas).
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

- **Re-cert LIVE sharding** (dépend de `shard.rs` re-vérifié sous 1.0 en Phase E) — séquencé
  APRÈS S81 (après orchestrateur ex-S78).
- **Viewer fondation** + Aperçu scellé/Proof Card.
- **P2-AUDIT-2-RESIDUEL** si `cargo tree -d` ne converge pas (sinon CLOSED en Phase G).
- 8 P2 / 11 P3 docs-contract → sprint dette nommé.
- 2 P1 in-vivo restent standing.
- Tout P2/P3 issu des phase-reviews S81 → `sprint82_audit_plan.md`.

## Amendement roadmap (à acter Phase I)

- Roadmap v5 (CANON) **s'arrête à S77** ; S78/79/80 sont déjà des amendements. **Insérer
  S81-iroh** (upgrade transport, non-planifié, forcing-function 2026-09-30).
- **Viewer → S82.**
- **Orchestrateur sharding (ex-S78) séquencé APRÈS S81** (re-cert shard live S82).
- Tracer : « la pre-launch policy *wire modifiable librement* ne couvre PAS le store
  on-disk iroh-docs/blobs déjà déployé ».

## Références (chemins absolus)

- **Pin iroh + MSRV + crypto** : `C:\Users\FlowUP\Documents\Code\nexus\Cargo.toml:24,37-41,58`.
- **Déclarations directes iroh (3 crates)** :
  `crates\nexus-core-rs\Cargo.toml:19-22` (les 4),
  `crates\nexus-shell-daemon\Cargo.toml:78,84` (iroh-blobs + iroh, **PAS wrapper-only** —
  `seed_protocol` impl `ProtocolHandler`),
  `crates\nexus-shell-daemon-core\Cargo.toml:179,186` (dev-deps).
- **Self-heal destructeur (NON backstop)** :
  `crates\nexus-shell-daemon\src\runtime.rs:2479-2541` (branche `None` `:2515-2528`).
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