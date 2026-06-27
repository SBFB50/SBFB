All contested facts confirmed against current code. Notably the skeptic's most dangerous finding holds: the self-heal at `runtime.rs:2515-2528` calls `create_doc()` (new namespace id) + overwrites the M8 row with **no `import_ticket`** of the old ticket — it is a destructive data-loss path, not a backstop. The materializer bug (`feed_materializer.rs:54-58` unconditional overwrite, `:95-101` unverified fold, `public_feed.rs:588` no prev_hash) and the 3-crate iroh declaration surface are also confirmed. Producing the canonical dossier.

---

# DOSSIER CANONIQUE — KICKOFF S81 (DRAFT DE STAGING)

> **STATUT : DRAFT de staging — S80 PAS ENCORE CLOS.** Ce dossier est la source unique des 3 rédacteurs (kickoff / plan / design_review). Tout ce qui dépend du verdict S80 (Phase 0, liste de carries entrants exacte, totaux de tests baseline) est marqué *provisoire* et sera figé à la clôture S80. Les corrections du sceptique sont **intégrées** (claims réfutés retirés) ; les contradictions inter-cartes sont tranchées en (C)/(E).
> **Sprint** : S81 — **Upgrade iroh 0.98 → 1.0** (insertion roadmap non-planifiée).
> **Faits vérifiés au code ce jour (2026-06-27)** : `Cargo.toml:37-41` = iroh/iroh-docs/iroh-gossip `0.98`, iroh-blobs `0.100` ; `Cargo.toml:24` rust-version `1.85` ; `Cargo.toml:58` ed25519-dalek `2.1`. 3 crates déclarent iroh en direct : `nexus-core-rs/Cargo.toml:19-22` (les 4), `nexus-shell-daemon/Cargo.toml:78,84` (iroh-blobs + iroh, **PAS wrapper-only** — `seed_protocol` impl `ProtocolHandler`), `nexus-shell-daemon-core/Cargo.toml:179,186` (dev-deps). Self-heal destructeur confirmé `runtime.rs:2515-2528`. Bug materializer confirmé `feed_materializer.rs:54-58,95-101` + `public_feed.rs:588-591`.

---

## (A) Objectif produit

Migrer toute la pile iroh de SBFB du pin `0.98` (ligne 0.9x, **maintenance coupée dès la GA 1.0**, relais publics N0 sunset **2026-09-30**) vers iroh 1.0 GA (`iroh 1.0.0` + `iroh-docs 0.101` + `iroh-gossip 0.101` + `iroh-blobs 0.103`), en préservant **sans perte** les données live déjà déployées (ancre VPS Hetzner S75, store `iroh-docs` sbfb-ideas, pins `keep_online` M18) à travers une migration on-disk redb 2→4 **one-way**, et en prouvant que la convergence transport cross-machine (doc-sync + gossip + blobs + seed/annuaire) survit au bump par un re-jeu LIVE. C'est une **maintenance d'infrastructure forcing-function-driven** : elle garantit la continuité de la découverte/connectivité avant le cutoff relais et solde la dette de version, sans rien ajouter au produit utilisateur.

---

## (B) Pourquoi maintenant

1. **Forcing function dure — 2026-09-30 (~3 mois).** Les relais publics N0 de la ligne 0.9x (qui **inclut le pin 0.98**) sont coupés au 2026-09-30 ; la ligne 1.0 est supportée « until End of Life » (iroh.computer/blog/v1). Surface concernée : `node.rs:318` `Endpoint::builder(presets::N0)` + retombée par défaut sur les 3 relais n0 prod via `relay_config.rs:17-20`. Sans upgrade, la 0.98 perd sa maintenance ET N0 cesse. L'escape-hatch opérateur `SBFB_CUSTOM_RELAYS`/`relays.json` → `RelayMode::Custom` (`node.rs:329,348`) est un palliatif de survie, **pas** un substitut à la migration.
2. **Dette de version.** Le pin 0.98 a ~2 mois de retard sur la GA et tombe hors train de patchs sécurité dès la 1.0. Le rename `Node→Endpoint` est **déjà absorbé** (le fantôme « Endpoint Takeover 153 call-sites » est mort : le code tourne post-rename — `endpoint.id()`, `presets::N0`, `MemoryLookup`, `EndpointAddr`, `ProtocolHandler` async/`AcceptError`). Le vrai travail = migration `iroh-docs` (wire + types) + 2 migrations on-disk redb 2→4.
3. **P2-AUDIT-2 (pré-release transitives).** Le lock 0.98 actuel tire DÉJÀ un fouillis crypto pré-release **dupliqué** (ed25519-dalek `3.0.0-pre.6` **ET** `3.0.0-rc.4`, sha2 `0.11.0-rc`, der/pkcs8/spki `0.8.0-rc`, curve25519 `5.0.0-pre.6`). **Correction intégrée (sceptique + G1 D6)** : l'upgrade **ne ferme PAS** P2-AUDIT-2 — iroh 1.0 épingle encore ed25519-dalek sur un `-rc`. La crypto SBFB security-critical (canary/curator/provenance/task, `ed25519-dalek 2.x` stable, `Cargo.toml:58`) est **isolée** et inchangée. L'upgrade améliore (déduplication) mais le RC reste un **résiduel** ; clôture conditionnée à `cargo tree -d` convergent (cf. D6).

---

## (C) Arbitrages PO load-bearing à trancher (avec reco Option)

> Ces arbitrages doivent être tranchés par le PO **avant** le 1er Edit. Ils englobent les 3 nommés (materializer / version / données-live) + les contradictions inter-cartes relevées par le sceptique.

**C1 — Scope « non-PROVISIONAL DONE » réaliste en 1 sprint ? (le plus structurant).**
Le sceptique réfute « S81 = 1 sprint propre vers un DONE non-PROVISIONAL » : le bar concentre migration live one-way + re-cert cross-machine + fix materializer + docs sécurité, **sous rigueur per-phase uniforme** (preflight 5 scans + review + Codex à CHAQUE phase). **Reco : scoper le DONE sur l'axe TRANSPORT-convergence uniquement** (doc-sync/gossip/blobs/seed/annuaire), **sortir l'axe SHARDING du T2** (re-cert live shard → S82, cf. C4/D4/D7). Sans ce cadrage, S81 finit PROVISIONAL sur l'axe shard exactement comme S77.

**C2 — Scope du fix materializer (wf4).** Contradiction inter-cartes : Carte 1 D8 → « sprint séparé postérieur » ; Cartes 3/4 + G1 D5 + sceptique → **Phase A DANS S81, AVANT le bump, commit propre séparé**. **Reco : Phase A in-sprint (adopter cartes 3/4).** Raison : le bug est convergence-critique et 0-bump (logique coordinator SQLite, indépendante d'iroh) ; le **gate de convergence cross-machine de S81 le révèle de toute façon** ; le corriger AVANT le bump établit une baseline 0.98 verte → préserve la bisectabilité (un échec post-bump = iroh, pas le materializer). **Discipline imposée** : jamais dans le commit de bump.

**C3 — Version cible.** **Correction intégrée (carte 3 + sceptique)** : « viser la dernière 1.0.x » est **actuellement insatisfiable** — au 2026-06-27, `1.0.0` (12 j) est la **seule** stable, **aucune 1.0.x patch n'existe**. **Reco (D1 option c) : coder sur `iroh = "=1.0.0"` maintenant + re-pin OBLIGATOIRE sur la 1re 1.0.x publiée AVANT le push live** ; si aucune patch au code-freeze → soak documenté + veille RustSEC sur 1.0.0 (interdiction de pousser la .0 brute si une patch existe). Le runway ~3 mois interdit d'attendre passivement.

**C4 — Stratégie données-live.** **Reco (D3 option c, hybride durcie)** : migration **IN-PLACE impérative** pour `docs.redb` (saut 0.98→0.101 DIRECT, feature défaut `redb-v2-migration`), validée sur **COPIE** du store VPS peuplé AVANT flip ; **self-heal `runtime.rs:2515` NEUTRALISÉ/gardé pendant la migration** (cf. C5) ; blobs in-place **avec test staging préalable** (pas un pari) + filet wipe **uniquement** pour les pins re-fetchables ; **tar snapshot** de `NEXUS_GRID_ROOT` avant 1er boot 0.101 (migration one-way → rollback = restore tar) ; **ancre VPS migrée EN DERNIER**, in-place, gardant `node_key`/`node_id` (re-install stock S75 **INTERDIT** : régénérerait l'identité → casse les locators abonnés).

**C5 — Le self-heal n'est PAS un backstop (correction critique du sceptique, vérifiée au code).** Carte 1 D2 recommandait « garder le self-heal en filet » — **c'est dangereux et REJETÉ**. À `runtime.rs:2515-2528`, la branche `None` appelle `create_doc()` (namespace id NEUF) + `set_storage_namespace` qui écrase la ligne M8, **sans `import_ticket` de l'ancien ticket** → orpheline les entries sbfb-ides répliquées + casse les DocTicket persistés, en `warn`-only silencieux. **Reco : garder le self-heal pour le cas légitime (DB importée d'un autre data-dir), mais le DÉSACTIVER/garder explicitement pendant la fenêtre de migration redb** (sinon un échec de migration déclenche une perte silencieuse au lieu d'un crash diagnostiquable).

**C6 — MSRV : contradiction inter-cartes.** Carte 4 → bump `1.94→1.95` inconditionnel ; cartes 1/3 → plancher réel `1.91` (iroh-docs 0.101) **déjà franchi** par le Docker canonique `rust:1.94`. **Reco (D6) : vérifier empiriquement (`cargo +1.94 build` Docker canonique) AVANT de budgéter ; NE PAS bumper 1.95 sans preuve cargo qu'une feuille l'exige.**

**C7 — Clôture P2-AUDIT-2.** **Reco** : ne PAS pré-annoncer CLOSED. Gate = `cargo tree -d` post-bump montre **un seul** arbre `ed25519-dalek` + 0 `*-pre`/`*-rc` dupliqués → si convergent, flipper `deny.toml:107` `multiple-versions warn→deny` ; sinon lever **P2-AUDIT-2-RESIDUEL** (carry S82). Vérifier aussi que le `ed25519-dalek 2.x` SBFB ne s'effondre PAS sur l'arbre RC d'iroh.

---

## (D) Scope In / Out

### IN
- Bump des 4 deps iroh en point unique `Cargo.toml:37-41` → `iroh 1.0.0` / `iroh-docs 0.101.0` / `iroh-gossip 0.101.0` / `iroh-blobs 0.103.0`, recompile mécanique sur les **3 crates** déclarant iroh.
- Cassure compile connue : `pkarr_resolver.rs:40,109` `CaRootsConfig→CaTlsConfig` (#4300) + re-vérif `PkarrRelayClient::new(url, tls)`.
- Migration profonde `iroh-docs` : wire `EntrySignature→iroh::Signature` (0.99.1), types `ed25519_dalek→iroh-base` (0.100.0), reconstruction raw-bytes `DocsNamespaceId::from` (`runtime.rs:2479`).
- Cascade `iroh-blobs` + redb4 (`FsStore`/`BlobsProtocol::new`/`Downloader`/tags) ; cascade `iroh-gossip` (pur recompile).
- Re-cert **compile + handshake** des surfaces fragiles : `shard.rs` (RTT/PathId), `seed_protocol.rs` (`ProtocolHandler`/`AcceptError`), relais (`relay_config.rs`, `node.rs RelayMode::Custom`).
- **Check nommé de survie URL pkarr** `pkarr_resolver.rs:54` (`dns.iroh.link/pkarr`) + default relay map sous 1.0.
- Migration on-disk redb 2→4 **validée sur copie** + fixture + neutralisation self-heal + migration LIVE ancre VPS (tar snapshot, VPS-last, runbook rollback).
- **Fix convergence materializer (wf4)** — Phase A, 0-bump, commit propre.
- Gate convergence crypto `cargo tree -d` + `deny.toml` flip-or-carry.
- Amendements docs sécurité : `THREAT_MODEL.md:22,128,195`, `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7`, `HARDENING_ROADMAP.md:5` (trigger iroh FIRED), `deny.toml:107`.
- Gate testabilité : T1 hermétique BLOQUANT + T2 JSON LIVE transport-convergence.

### OUT (reroutés / interdits dans S81)
- **Re-cert LIVE du sharding** (`shard.rs` RTT/PathId multipath noq, RIG-ABSENT, non-hermétique) → **S82** (après orchestrateur ex-S78). On ne « re-joue » PAS une acceptance (S77 b3_shard) qui n'a **jamais** passé.
- **Viewer fondation** (tools/factory-ui jeté S80) + Aperçu scellé/Proof Card → **S82**.
- **Dette docs-contract 8 P2 / 11 P3 (S80)** → **sprint dette nommé distinct** (jamais bundlé).
- **2 P1 in-vivo standing** (sharding S77, app-authoring S79) → restent ouverts, hors corps S81.
- **GuardianDB** / toute autre upgrade → séparé et postérieur (bisectabilité, directive séquencement #7).
- **Bump MSRV 1.95** inconditionnel → INTERDIT sans preuve cargo (C6).
- **Pagination app-storage**, features produit non liées à iroh → backlog.
- **Clôture pré-annoncée P2-AUDIT-2** → INTERDITE sans `cargo tree -d` convergent.

---

## (E) D1..D8 FINAUX

> Décision tranchée + reco + score G1 + note (avec corrections sceptique intégrées).

**D1 — Version cible.** **Tranché : coder sur `iroh = "=1.0.0"` maintenant ; re-pin sur la 1re 1.0.x patch AVANT push live ; sinon soak + veille RustSEC documentés.** Quatuor : iroh 1.0.0 / docs 0.101.0 / gossip 0.101.0 / blobs 0.103.0, bump en point unique `Cargo.toml:37-41`. **Score G1 : 4 (CONDITIONAL).** Note : « viser la dernière 1.0.x » est insatisfiable (0 patch publiée) — la reco wf3 est **réécrite** en « 1.0.0 + re-pin conditionnel ». Risque .0-fraîche (12 j) réel → pin exact, re-pin obligatoire.

**D2 — Relais / discovery post-EOL N0.** **Tranché : `presets::N0` par défaut (mis à jour <24 h après release par n0) + relais iroh self-hosted OPTIONNEL pour l'ancre VPS comme résilience.** Escape-hatch déjà câblé (`node.rs:329,348`, `SBFB_CUSTOM_RELAYS`). **Score G1 : 4 (CONDITIONAL).** Note **BLOQUANTE** : vérifier explicitement la survie de l'URL pkarr `pkarr_resolver.rs:54` (le blog n0 avertit « wire-breaking relay changes get new URLs ») — sinon discovery casse **silencieusement** (pas de crash). Check nommé, jamais plié dans « recompile ».

**D3 — Migration données on-disk redb 2→4.** **Tranché : hybride durci.** docs.redb = IN-PLACE impérative (saut 0.98→0.101 DIRECT) ; coordinator SQLite (M18, public_feed) intact ; blobs in-place **avec test staging** ; wipe blobs toléré **uniquement** pour pins re-fetchables. **Score G1 : 4 (CONDITIONAL).** Conditions cumulatives : (1) saut direct, jamais 0.99/0.100 contre l'ancien store ; (2) wipe docs INTERDIT ; (3) tar snapshot avant 1er boot 0.101 (one-way) ; (4) validation sur COPIE du store VPS peuplé ; (5) ancre VPS in-place gardant node_key/node_id (re-install stock S75 INTERDIT) ; (6) fixture redb 2→4 dans T1 ; **(7, ajout sceptique) self-heal `runtime.rs:2515` neutralisé/gardé pendant la migration — ce n'est PAS un backstop.**

**D4 — Stratégie test / convergence.** **Tranché : T1 hermétique convergence in-process (multi_daemon loopback/MemoryLookup) BLOQUANT Win natif + CI Linux (jamais Docker-on-Windows) ; T2 acceptance JSON LIVE transport-convergence PASS sur rig VPS+Win+Mac.** **Score G1 : 4 (CONDITIONAL).** Note **corrigée (sceptique, split verdict)** : l'axe **transport** (VPS Hetzner + dev Win + Mac M2) existe et PEUT atteindre PASS (S75 survives-VPS-death = LIVE PASS, LAN Win↔Mac validé). L'axe **sharding** (rig GPU 5080+M2 + orchestrateur in-vivo) était **chroniquement absent** (S76 DIFFERE, S77 RIG-ABSENT, orchestrateur reporté S78) → **hors T2 de S81**. RIG-ABSENT illégitime **sur l'axe transport** ; légitime/inapplicable sur l'axe sharding (reporté S82).

**D5 — Scope fix materializer (wf4).** **Tranché : IN S81, Phase A, AVANT le bump, commit propre dédié.** Fix = fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break `(timestamp, author, hash)` + garde monotone dans `apply()` (`feed_materializer.rs:54-58`) + `verify_entry` check `prev_hash` (`public_feed.rs:588`). **Score G1 : 4 (PASS).** Note : tranche le conflit Carte 1 (sprint séparé) vs cartes 3/4 (in-sprint) **en faveur de l'in-sprint** ; baseline 0.98 verte = bisectabilité préservée. Jamais mélangé au commit de migration.

**D6 — MSRV + sweep deps feuilles.** **Tranché : vérifier MSRV empiriquement (`cargo +1.94 build` Docker) ; rester 1.94 sauf preuve cargo qu'une feuille exige plus ; P2-AUDIT-2 reste OUVERT (résiduel) jusqu'à `cargo tree -d` convergent.** **Score G1 : 3 (CONDITIONAL).** Note : tranche la contradiction Carte 4 (1.95 inconditionnel) vs cartes 1/3 (plancher 1.91 déjà franchi) **en faveur de l'empirique**. Gate convergence crypto = `deny.toml:107` flip si un seul arbre ed25519-dalek + 0 `*-pre`/`*-rc` ; sinon carry **P2-AUDIT-2-RESIDUEL**.

**D7 — Carries + roadmap (séquencement).** **Tranché : S81 = iroh STRICTEMENT SEUL ; re-scanner les call-sites sur les 3 crates (core + 2 daemon), pas seulement nexus-core-rs ; amender roadmap_v5 (insertion S81-iroh + Viewer→S82) ; orchestrateur sharding séquencé APRÈS S81.** **Score G1 : 4 (PASS).** Note **corrigée** : blast-radius = 3 crates déclarent iroh (bump = point unique, mais call-sites API débordent côté daemon — `ProtocolHandler` seed). Clôture P2-AUDIT-2 GATÉE par D6.

**D8 — R-iroh-audit / posture release.** **Tranché : R-iroh-audit P0 INCHANGÉ. L'upgrade NE franchit PAS Gate 1/Gate 3, NE débloque PAS le pilote public ferme. Maintenance forcing-function-driven, pas levée de zone rouge.** **Score G1 : 5 (PASS).** Note : iroh 1.0 = 0 audit tiers public. Le wire-freeze 1.0 réduit le churn de la surface de désérialisation (THREAT_MODEL menace E) = neutre-à-positif, jamais un durcissement de confiance. Libellé explicite « upgrade ≠ Gate 1 » obligatoire dans kickoff + commit body.

---

## (F) Plan de phases

> **Phase 0 = audit gate S80 (placeholder)** — jouée à la clôture de S80 (convention permanente). Absorbe le verdict S80, fige la liste exacte des carries entrants (cf. H) et la baseline de tests. **Le corps S81 (A→I) ne démarre qu'après Phase 0 PASS.** Rigueur per-phase uniforme : deep preflight (5 scans) + review + Codex à CHAQUE phase.

---

**Phase A — Fix convergence materializer (wf4) [0-bump, AVANT le bump]**
- **Objectif** : éliminer la divergence `PublicRegistryView` cross-noeud sur ingest hors-ordre ; établir une baseline 0.98 verte (bisectabilité).
- **Livrables** : `crates/nexus-coordinator-rs/src/feed_materializer.rs` (`materialize_full` → fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break `(timestamp,author,hash)` ; garde monotone `apply()` ReleasePublished `:54-58`) ; `crates/nexus-coordinator-rs/src/public_feed.rs` (`verify_entry` check `prev_hash` `:588-591`) ; doc des fonctions `:89-94`.
- **Delta tests attendu** : **+4..6 Rust** (convergence ingest hors-ordre, tie-break déterministe, garde monotone, prev_hash rejeté).
- **Gate / scope-cut** : commit propre dédié, **JAMAIS** dans le commit de bump. 0-bump wire SBFB. Indépendant d'iroh.

**Phase B — Bump deps workspace + recompile mécanique + MSRV empirique**
- **Objectif** : `cargo build --workspace` vert sous iroh 1.0 ; corriger l'unique cassure connue ; fixer la MSRV réelle.
- **Livrables** : `Cargo.toml:37-41` (`=1.0.0`/`0.101.0`/`0.101.0`/`0.103.0`) ; `pkarr_resolver.rs:40,109` `CaRootsConfig→CaTlsConfig` + re-vérif `PkarrRelayClient::new` `:114` ; commentaires version (`Cargo.toml:33-35`, `node.rs:24`, `blobs.rs:87`, `docs.rs:54`, `discovery.rs:6-8`) ; `Cargo.lock` figé ; vérif `cargo +1.94 build` Docker canonique (décision MSRV, cf. D6). Checkpoint gossip (pur recompile).
- **Delta tests attendu** : **0 net** (recompile ; tests existants doivent rester verts).
- **Gate / scope-cut** : iroh SEUL. Pas de bump 1.95 sans preuve cargo. `Cargo.lock` capturé pour `cargo tree -d` (Phase G).

**Phase C — iroh-docs deep (wire + types iroh-base)**
- **Objectif** : adapter la surface docs aux types iroh-base 0.100 + wire `EntrySignature→iroh::Signature`.
- **Livrables** : `crates/nexus-core-rs/src/docs.rs:42-47,229,275,388-410` (AuthorId/NamespaceId/Entry/DocTicket/Query/ShareMode/AddrInfoOptions/LiveEvent) ; `node.rs:388-395` (`Docs::persistent/memory/spawn`) ; `runtime.rs:2479` (`DocsNamespaceId::from([u8;32])`) ; **suppression actée des zombies legacy-decode** du wire redéfini (pre-launch policy).
- **Delta tests attendu** : **+2..4 Rust** (round-trip signature/types) **− N zombies legacy-decode** (chaque suppression actée dans le body).
- **Gate / scope-cut** : 0 bump wire SBFB (JCS/DOMAIN_*_V1/FEED_FORMAT_VERSION). Vérifier stabilité format string `DocTicket` (colonne coordinator `doc_ticket`).

**Phase D — iroh-blobs cascade + redb4**
- **Objectif** : recompiler la couche blobs sous 0.103 + valider l'ouverture du store redb4.
- **Livrables** : `crates/nexus-core-rs/src/blobs.rs:85-252` (`add_bytes`/`TagInfo.hash`, `get_bytes`/`has`, `tags().set/delete/get`, `HashAndFormat::raw`, `Downloader::new+download`, `BlobTicket::new/into_parts`, `Hash::from_bytes`) ; `node.rs:47-50,375-398` (`FsStore::load`/`MemStore`/`BlobsProtocol::new`/store deref) ; re-vérif signatures `BlobsProtocol::new` + `Downloader::new`.
- **Delta tests attendu** : **+1..3 Rust** (ticket round-trip, tag set/get, blob fetch local).
- **Gate / scope-cut** : changelog 0.101-0.103 non détaillé côté signatures → découvrir au compile, documenter tout break.

**Phase E — Surfaces fragiles transport re-cert (3 crates)**
- **Objectif** : re-certifier compile + handshake des surfaces non-hermétiques ; **check nommé URL pkarr/relais**.
- **Livrables** : `crates/nexus-core-rs/src/shard.rs:60-63,171-181,299-327` (`Connection::rtt(PathId::ZERO)`, `closed`/`close`/`remote_id` — **traité UNVERIFIED-high-risk, pas « SAUVE »**, cf. I) ; `seed_protocol.rs:44-48,263-264` (`ProtocolHandler`/`AcceptError`, crate nexus-shell-daemon) ; `pkarr_resolver.rs:38-41,54,107-115` (+ **survie URL `dns.iroh.link/pkarr`**) ; `relay_config.rs:17-20,46` + `node.rs:318,329,348` (`RelayMode::Custom`, default_relay_map URLs) ; re-scan call-sites sur `nexus-shell-daemon` + `nexus-shell-daemon-core`.
- **Delta tests attendu** : **+1..2 Rust** (handshake seed 2-noeuds in-process ; pkarr resolver parse).
- **Gate / scope-cut** : re-cert LIVE shard multipath = **OUT** (→ S82). Provisionner relais self-hosted pour l'ancre (résilience).

**Phase F — Migration on-disk redb 2→4 validée sur COPIE**
- **Objectif** : prouver hors-prod que docs.redb + blobs survivent à la migration ; neutraliser le self-heal destructeur.
- **Livrables** : fixture de migration redb 2→4 (store peuplé namespace sbfb-ides) ; test ouverture blobs redb2 sous 0.103 (staging) ; **garde explicite autour de `runtime.rs:2515-2528`** (self-heal NON déclenché en fenêtre de migration — sinon perte silencieuse) ; inventaire « pins re-fetchables ailleurs ? » avant toute tolérance wipe blobs ; vérif parse `DocTicket` (DB) + `BlobTicket` (`anchors.json`) post-migration.
- **Delta tests attendu** : **+3..5 Rust** (fixture migration in-place, survie entries, parse tickets persistés, non-déclenchement self-heal).
- **Gate / scope-cut** : aucune migration LIVE ici — uniquement sur copie. Migration one-way → documenter rollback = restore tar.

**Phase G — CI / MSRV / convergence crypto + docs sécurité**
- **Objectif** : verts dual-platform + gate de convergence supply-chain + amendements docs.
- **Livrables** : `cargo tree -d` (gate convergence : un seul arbre ed25519-dalek, 0 `*-pre`/`*-rc`) → **flip `deny.toml:107` warn→deny** OU carry **P2-AUDIT-2-RESIDUEL** ; image CI/Docker canonique + `Cargo.toml:24` rust-version **seulement si** D6 l'exige ; `cargo-deny`/`cargo-audit` verts ; amendements `THREAT_MODEL.md:22,128,195` (0.98→1.0.0 + rationale wire-freeze, résiduel reste M), `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7` (0.97/0.99→1.0.0, note R-iroh-audit reconfirmée verbatim, rejouer checklist `cargo tree`), `HARDENING_ROADMAP.md:5` (trigger iroh FIRED + bump `last_validated`).
- **Delta tests attendu** : **0** (gates supply-chain + docs).
- **Gate / scope-cut** : NE PAS marquer P2-AUDIT-2 CLOSED si le lock ne converge pas. NE PAS rouvrir warrant canary / loopback / guardrails / capability toggles (aucun trigger iroh).

**Phase H — Migration LIVE ancre VPS + acceptance**
- **Objectif** : migrer le matériel live sans perte, dans l'ordre sûr.
- **Livrables** : runbook (`docs/` ou planning) : **tar snapshot** `NEXUS_GRID_ROOT` (docs.redb + blobs/) AVANT restart ; ordre **dev Win + Mac d'abord, VPS EN DERNIER** (wire docs/gossip non-rétrocompat intra-rollout) ; deploy binaire 1.0.x + restart systemd ; vérif 1er boot 0 crash-loop + docs.redb migré + `node_id` INCHANGÉ + feed/ides/pins intacts ; rollback = restore tar (one-way). `deploy/nexus-shell-daemon.service` inchangé (start --headless).
- **Delta tests attendu** : **0** (acceptance opérationnelle).
- **Gate / scope-cut** : re-install stock S75 INTERDIT sur ancre live. Migration VPS bloquée tant que la validation sur copie (F) n'est pas PASS.

**Phase I — Wrap-up + gate testabilité + roadmap**
- **Objectif** : T1 BLOQUANT + T2 JSON LIVE + clôture documentaire.
- **Livrables** : T1 hermétique (cf. G) câblé BLOQUANT + CI ; artefact **T2 JSON committé** (transport-convergence, cf. G) ; re-jeu acceptances S75 (survives-VPS-death) + S76 (b3 quorum) + **b3 PASS fetch blob cross-machine** ; amendement `roadmap_v5` (insertion S81-iroh + Viewer→S82 + orchestrateur sharding après S81) ; `SPRINT_LOG.md` + `CLAUDE.md` + memory (`nexus_grid_pivot.md`, `MEMORY.md`) ; `PATTERNS.md` ; `sprint82_audit_plan.md` (carries reroutés).
- **Delta tests attendu** : **+ tests T1** (convergence in-process + fixture redb) consolidés ; **delta net global attendu +10..20 Rust** (deletions zombies actées, total interdit de descendre silencieusement).
- **Gate / scope-cut** : T1 BLOQUANT non négociable ; T2 LIVE PASS (RIG-ABSENT illégitime sur axe transport). Axe sharding hors T2.

---

## (G) Gate de testabilité

**T1 — Hermétique, BLOQUANT (Win natif + CI Linux Woodpecker/GHA ; JAMAIS Docker-on-Windows car `multi_daemon` env-bloqué `create_node` hang) :**
1. **Convergence in-process** : `multi_daemon` 2-noeuds loopback / `MemoryLookup` sur la stack migrée — doc-sync (wire iroh-docs migré) + gossip + blobs fetch + seed ALPN `sbfb/seed/0` handshake + ingest annuaire.
2. **Convergence ingest hors-ordre** : assert `PublicRegistryView` identique cross-fold quel que soit l'ordre d'arrivée (couvre le fix materializer Phase A — l'assertion centrale).
3. **Fixture migration redb 2→4** : ouvrir un `docs.redb` redb2 peuplé (namespace sbfb-ides) sous 0.101 → entries survivent, namespace id INCHANGÉ, **self-heal non déclenché** ; ouvrir un store blobs redb2 sous 0.103.
4. **Parse tickets persistés** : `DocTicket` (string DB) + `BlobTicket` (`anchors.json`) re-parsent post-migration.
5. **Recompile + handshake shard** : `shard.rs` compile + handshake `sbfb/shard/1` in-process (PAS le RTT/multipath live).

**T2 — Acceptance JSON cross-machine committé (`PASS` / `BLOCK{diagnosis}` / `RIG-ABSENT`) :**
- **Axe transport (DANS S81, PASS obligatoire)** : rig réel VPS Hetzner + dev Win + Mac M2 — re-jeu **S75 survives-VPS-death** + **S76 b3 quorum** + **b3 PASS fetch blob cross-machine** post-upgrade ; convergence `PublicRegistryView` cross-noeud après migration LIVE. **`RIG-ABSENT` illégitime sur cet axe** (rig confirmé dispo, `live_acceptance_setup`) ; seul un rig **génuinement HS** le justifie.
- **Axe sharding (HORS S81)** : `shard.rs` RTT/PathId multipath noq + orchestrateur in-vivo = **non testable hermétiquement, rig GPU chroniquement absent** → **reporté S82** (après orchestrateur ex-S78). On ne re-joue PAS une acceptance (S77 b3_shard) jamais passée.

---

## (H) Carries entrants / sortants + amendement roadmap

> *Liste entrante provisoire — figée à la clôture S80 (Phase 0).*

**Entrants (S80 → S81)** :
- 2 P1 in-vivo OUVERTS (sharding RIG-ABSENT S77, app-authoring « Not evidenced » S79) — **standing**, hors corps S81.
- Viewer fondation + Aperçu scellé/Proof Card (tools/factory-ui jeté S80) — **réservés S81 à l'origine, reroutés S82**.
- 8 P2 / 11 P3 docs-contract S80 — **sprint dette nommé distinct**.
- Régression couverture (perte Vitest factory-operator + factory-ui) — re-couverte S80 Phase I (à confirmer).
- P2-AUDIT-2 (pin transitif iroh) — **traité par S81 mais NON pré-clôturé** (cf. C7/D6).

**Sortants (S81 → S82)** :
- **Re-cert LIVE sharding** (dépend de `shard.rs` re-vérifié sous 1.0 en Phase E) — séquencé APRÈS S81.
- **Viewer fondation** + Aperçu scellé/Proof Card.
- **P2-AUDIT-2-RESIDUEL** si `cargo tree -d` ne converge pas (sinon CLOSED en Phase G).
- 8 P2 / 11 P3 docs-contract → sprint dette.
- 2 P1 in-vivo restent standing.
- Tout P2/P3 issu des phase-reviews S81 → `sprint82_audit_plan.md`.

**Amendement roadmap (à acter Phase I)** :
- Roadmap v5 (CANON) **s'arrête à S77** ; S78/79/80 sont déjà des amendements. **Insérer S81-iroh (upgrade transport, non-planifié, forcing-function 2026-09-30).**
- **Viewer → S82.**
- **Orchestrateur sharding (ex-S78) séquencé APRÈS S81** (re-cert shard live S82).
- Tracer : « la pre-launch policy *wire modifiable librement* ne couvre PAS le store on-disk iroh-docs/blobs déjà déployé ».

---

## (I) Risques + garde-fous bloquants

| # | Risque | Garde-fou BLOQUANT |
|---|--------|--------------------|
| R1 | **Perte données live** (migration redb 2→4 one-way échoue/lossy sur docs.redb : entries sbfb-ides + namespace id + DocTicket persistés) — **RISQUE PRINCIPAL** | Validation sur COPIE du store VPS peuplé (Phase F) + tar snapshot avant 1er boot (Phase H) + **self-heal `runtime.rs:2515` neutralisé pendant migration** (sceptique : ce n'est PAS un backstop, il `create_doc()` un id NEUF sans `import_ticket`) + recovery pins via coordinator DB M18 + boot re-announce S74-F |
| R2 | **iroh-blobs ouverture in-place redb2 NON validée** (changelog silencieux ; le wipe+re-pull suppose un 2e détenteur — faux si l'ancre VPS est seul holder) | Test staging nommé (ouvrir le store blobs dev existant sous 0.103) **gate le flip VPS** ; inventaire « pin re-fetchable ailleurs ? » avant toute tolérance wipe (Phase F) |
| R3 | **URL pkarr/relais renommée en 1.0** (blog n0 : « wire-breaking relay changes get new URLs ») → discovery casse SILENCIEUSEMENT, ancre injoignable | Check nommé `pkarr_resolver.rs:54` + default_relay_map sous 1.0 (Phase E), pré-flip, jamais plié dans « recompile » |
| R4 | **Rollout demi-upgradé + piège one-way** (un noeud 0.98 + un 0.101 ne syncent plus ; migration irréversible → flotte non-convergente non-downgradable) | Ordre codifié **dev Win + Mac d'abord, VPS EN DERNIER** + runbook tar-restore testé (Phase H) |
| R5 | **`shard.rs` RTT/PathId multipath noq = UNVERIFIED-high-risk** (sceptique : pas « SAUVE » — API 1.0 post-cutoff, non-hermétique, rig absent) + cartes contradictoires | Re-cert **compile + handshake seulement** en S81 (Phase E) ; **re-cert LIVE multipath → S82** ; ne jamais claim « stable verbatim » |
| R6 | **P2-AUDIT-2 NON résolu** (iroh 1.0 épingle encore ed25519-dalek `-rc` ; lock peut ne pas converger ; SBFB 2.x risque de s'effondrer sur l'arbre RC) | `cargo tree -d` (Phase G) = un seul arbre + 0 `*-pre`/`*-rc` → flip `deny.toml:107` ; SINON carry **P2-AUDIT-2-RESIDUEL** + **ne PAS annoncer CLOSED** |
| R7 | **MSRV bumpé à tort** (Carte 4 « 1.95 inconditionnel » non vérifié ; plancher réel 1.91 déjà franchi par 1.94) | `cargo +1.94 build` Docker canonique AVANT budget Phase G ; bump 1.95 INTERDIT sans preuve feuille |
| R8 | **Ressource staging non budgétée** (valider sur copie exige de tirer le store VPS live vers dev + fixture namespace peuplée) | Provisionner le pull du store + fixture comme pré-requis explicite de Phase F |
| R9 | **Faux signal de durcissement** (présenter l'upgrade comme amélioration sécurité alors que R-iroh-audit P0 reste entier → relâche garde pilote) | Libellé explicite « upgrade ≠ Gate 1, R-iroh-audit P0 inchangé, pilote reste ferme » dans kickoff + commit body + docs sécurité (Phase G) |
| R10 | **Tentation de bundler** (Viewer/dette/GuardianDB/materializer-dans-le-bump → bisectabilité détruite) | iroh STRICTEMENT SEUL ; materializer en Phase A commit séparé AVANT bump ; tout le reste reroutés (D7) |
| R11 | **Chute silencieuse du total de tests** (zombies legacy-decode du wire iroh-docs redéfini à supprimer) | Acter CHAQUE deletion dans le body de commit ; delta net global +10..20 Rust attendu, total interdit de descendre sans justification |
| R12 | **Scope-realism / PROVISIONAL** (DONE non-PROVISIONAL en 1 sprint à risque sur l'axe shard, comme S77) | Scoper le DONE sur l'axe TRANSPORT (C1/D4) ; axe sharding explicitement hors T2 → S82 |

---

**Fichiers de référence pour les 3 rédacteurs (absolus)** :
- `C:\Users\FlowUP\Documents\Code\nexus\Cargo.toml:24,37-41,58`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-core-rs\Cargo.toml:19-22`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-shell-daemon\Cargo.toml:78,84`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-shell-daemon-core\Cargo.toml:179,186`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-shell-daemon\src\runtime.rs:2479-2541` (self-heal destructeur `:2515-2528`)
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-coordinator-rs\src\feed_materializer.rs:45-115`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-coordinator-rs\src\public_feed.rs:585-603`
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-core-rs\src\` : `shard.rs`, `seed_protocol.rs`, `pkarr_resolver.rs`, `relay_config.rs`, `node.rs`, `docs.rs`, `blobs.rs`, `discovery.rs`
- `C:\Users\FlowUP\Documents\Code\nexus\docs\security\` : `THREAT_MODEL.md:22,128,195`, `EXTERNAL_AUDIT_SCOPE.md` §2.4/§2.7, `HARDENING_ROADMAP.md:5`
- `C:\Users\FlowUP\Documents\Code\nexus\deny.toml:107`
- `C:\Users\FlowUP\Documents\Code\nexus\.planning\roadmap_v5_factory_complete_vision.md` (amendement Phase I)