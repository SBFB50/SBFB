# Sprint 75 — Kickoff : Découverte PULL node-centrique + ancre VPS

> Pivot stratégique PO. La découverte d'apps passe de **PUSH-éphémère**
> (annonce PoW-gated qui expire à 30 min → apps anciennes invisibles aux
> nouveaux pairs) à **PULL node-centrique** (Browse = liste de nœuds → catalogue
> d'un nœud → download), avec le VPS always-on comme **ancre** (seed permanent +
> annuaire signé répliquable). Source : workflows d'audit `wk4fzrg8b` (gate S74
> PASS) + recherche/décision `wdeedndsh` (5 agents recherche profonde + panel
> adversarial 3-substrats).

## §0 Sources consultées (pre-gel, G9)

Recherche factuelle AVANT gel des D1-D5 (workflow `wdeedndsh`, WebSearch +
Context7 + lecture code OSS, accès 2026-06-09) :

**Prior art OSS découverte/pull décentralisée :**
- Nostr **NIP-65** Relay List Metadata (kind:10002 write/read relays) —
  `https://nips.nostr.com/65` ; **Outbox model** (abandon du blast-to-all,
  bootstrap indexeur, « no hard dependency on biggest player ») —
  `whynostr.org/.../What-is-the-Outbox-Model`, `nostrify.dev/relay/outbox`.
- **Radicle Heartwood** (Node/Inventory/Reference announcements, seeding
  scope-by-interest, seed-nodes ≠ dépendance centrale, anti-flood drop-already-
  seen) — `docs.radicle.xyz/guides/protocol`, `hackmd.io/@radicle/rJ2UH54P6`.
- **F-Droid** Security Model (dépôt = clé de signature unique, TOFU, pas
  d'autorité centrale, dépôts custom à égalité, index timestamp+expiry) —
  `f-droid.org/docs/Security_Model` ; anti-pattern « default-trust-in-binary »
  — `gitlab.com/fdroid/fdroidclient/issues/2557`.
- **IPFS** Provide Sweep (limite 5280 CIDs/22h, TTL 48h, bottleneck lookup) +
  delegated routing caching (fenêtre 24h) + IPNI indexeur centralisé —
  `ipshipyard.com/blog/2025-dht-provide-sweep`, `blog.ipfs.tech/2025-delegated-
  routing-caching`, `docs.ipfs.tech/concepts/ipni`.
- **BitTorrent** Mainline DHT Sybil (IDs auto-choisis, eclipse) — Wang 2012
  (`nymity.ch/sybilhunting`), BEP-5 security assessment ; **BEP-44** mutable
  items (signé clé publique, cap 1000 octets, expiry 1h, re-announce périodique)
  — `bittorrent.org/beps/bep_0044`.
- **SSB Rooms** (pubs surchargés à héberger les données de tous → les rooms ne
  stockent PAS, tunnel/meeting-point seulement) — `manyver.se/blog/announcing-
  ssb-rooms` ; Protocol Guide (réplication follow-graph, invite-only) —
  `ssbc.github.io/scuttlebutt-protocol-guide`.
- **ARES 2024** « The Sybil Attack Strikes Again » (broadcast réseau-large →
  censure/DoS mono-machine) — argument central du DEFER SearchManifest.

**Code SBFB ancré :** `pow.rs:109` (`MAX_PROOF_AGE_SECS=1800`, racine du bug) +
`:411-426` ; `runtime.rs:1488-1500` (drop Expired), `:1513/1544/1615` (replay),
`:1876-1897` (restore OWN-only) ; `curator.rs:100-342` (CuratorList Ed25519+JCS,
revision, caps) + `:589-602` (test séparation domaine) ; `canonical.rs:201-219`
(précédent `DOMAIN_SEED_REQUEST_V1` S74) ; `browse.rs:195` (node_id
`#[serde(skip)]`), `:272` (direct_entries in-memory) ; `seed_registry.rs:10-13`
(invariant content-addressing), `:38` ; `blobs.rs:170-193` (fetch_ticket single
endpoint) ; `config.rs:245-251` (default_curators vide) ; `feed_sync.rs:160-199`
(reannounce sans re-pin). **Doc D3 différé** :
`.planning/research/s73_searchmanifest_index_node_design.md` (lu en entier).
**THREAT_MODEL §15** (surface seed cross-nœud, over-count résidu row D).

## §1 Constat d'entrée

### §1.1 D'où on part
- **Tip** : HEAD `0e2fb6b` (post audit gate S74). `master...origin/master` 0
  ahead avant le commit kickoff, 1 ahead après l'audit findings (local).
- **Le PROBLÈME (live, cross-machine Win↔Mac LAN)** : la découverte est
  PUSH-éphémère. À la publication, le daemon construit un `ProjectAnnouncement`,
  l'enveloppe dans un `PowEnvelope` PoW-gated (difficulté 2^18), le gossipe, et
  persiste **les octets exacts** dans un outbox SQLite durable. Un récepteur
  **REJETTE** toute annonce dont le PoW `issued_at` > `MAX_PROOF_AGE_SECS=1800s`
  (`pow.rs:109` + `:420`). Mais le replay de l'outbox (NeighborUp / refresh /
  republish périodique / restore boot) re-broadcaste les octets **VERBATIM** avec
  le PoW d'origine — jamais re-tamponné. **Conséquence** : un pair frais ne
  découvre que les apps annoncées avec un PoW < 30 min. Les apps publiées plus
  tôt sont **invisibles**. Preuve live : `/api/daemon/info` du Mac =
  `known_browse_entries:0` ; log = « PoW proof too old (issued ~2.6M s ago) ».
- **Second trou (load-bearing, R4)** : `BrowseAggregator.direct_entries` est
  in-memory (`browse.rs:272`) ; au boot seules les apps OWN sont restaurées
  (`runtime.rs:1876`). Les apps distantes découvertes via gossip **ne survivent
  pas au reboot**. Et l'index FTS5 durable omet `node_id` + `archive_ticket` →
  trace inactionnable pour re-fetch.

### §1.2 Ancrage roadmap
Roadmap v5 (CANON) : S75 = GPU partagé cross-machine, S76 = sharding. **Ce
sprint AMENDE la roadmap** (cf. §12) : la découverte est *fondationale* (sans
elle, les apps publiées sont invisibles aux nouveaux = bug live), donc passe
AVANT le GPU. S75 = découverte PULL ; GPU → S76 ; sharding → S77.

### §1.3 Compteurs tests entrée
Re-vérifiés empiriquement cette session (audit gate S74, env récupéré) : Rust
Windows natif `nextest --workspace` **0 échec** (suite iroh-networked incluse,
~1675) + clippy `--all-targets` 0 + doctests 0 ; web tsc 0 / lint 0 / **Vitest
331** / coverage 86.91/78.63/85.82/88.23 / size 6/6 / scan FR. Source of truth
mesurable = `sprint75_verification.md §Fail-fast` (à produire en sortie).

### §1.4 Pre-launch protocol policy (rappel)
Le réseau n'a aucun déploiement live tiers. Le feed est extensible via raw-op
(0-bump `FEED_FORMAT_VERSION`). Un **nouveau type signé** (`NodeDirectoryEntry`)
= purement additif, son PROPRE `DOMAIN_NODE_DIRECTORY_V1`, 0-bump des
`*_FORMAT_VERSION` existants — exactement le pattern S74 `DOMAIN_SEED_REQUEST_V1`.
`CURATOR_LIST_FORMAT_VERSION`=1 reste librement redéfinissable pre-tag. Pas de
decoder legacy à porter.

## §2 Goal (une phrase)

> S75 remplace la découverte PUSH-éphémère cassée par une **découverte PULL
> node-centrique** : (1) corrige le bug live (FIX-A re-mint adresse+PoW au
> replay) ; (2) introduit `NodeDirectoryEntry` (annuaire signé répliquable de
> nœuds→catalogues, tiré sur abonnement) ; (3) ferme le gap de durabilité des
> catalogues distants ; (4) fait du VPS une ancre bornée (catalogue-publisher +
> seed permanent de MES apps), **prouvablement remplaçable** — le réseau survit à
> sa mort. **Critère SMART** : `sprint75_verification.md §Fail-fast` vert +
> acceptance « survives-VPS-death » démontrée cross-machine (Win/Mac/VPS).

## §3 Phase 0 — Audit gate S74 (DONE = PASS)

Joué cette session (Cas A, workflow anti-anchoring 9 tracks + skeptics). **PASS**
(`0e2fb6b`) : 0 P0, 0 P1, 15 P2, 10 P3 (tous CONFIRMED). Aucun `fix(sprint74)`
requis. Détail : `sprint74_audit_findings.md`. Les 15 P2 sont routés §8 (8
touchent directement le pivot, à concevoir dedans).

## §4 Garde-fou anti-recentralisation (NON NÉGOCIABLE, 5 verrous)

Invariant cardinal : **« No central server, no admin »**. Le VPS doit être **UNE
ancre**, **PAS LE serveur**. Les 5 verrous (s74 design :140-155, THREAT_MODEL
§15) s'appliquent à toute la conception :
1. **Zéro champ cible/hôte** nulle part (un dropdown « publier sur X » = serveur
   central de fait). L'annuaire est une projection **read-side**, jamais un
   sélecteur de destination write-side.
2. **Redondance additive jamais substitutive** : node-Browse cohabite avec / est
   un sur-ensemble strict de la grille curator-agrégée ; `known_browse_entries`
   compte TOUTES les apps découvrables honnêtement.
3. **VPS = « Mon serveur » (possessif)**, jamais défaut universel. Concrètement :
   `135.181.42.188`/son node_id vit dans MON `config.toml` `default_curators`
   (config-distribué, vide par défaut `config.rs:249-250`), **JAMAIS hard-codé
   dans un `default_curators` compilé livré à tous**. ← **DESIGN-CONFLICT
   tripwire** (C5 board).
4. **Provenance/signature toujours celles de l'auteur** quel que soit le seeder
   (Radicle : seed ≠ autorité). node-Browse rend la `provenance.json` de
   l'auteur comme badge d'autorité, JAMAIS le nœud hébergeur/seeder.
5. **Suggestion déclenchée par l'état observé**, jamais poussée au publish.

**Test cardinal — le réseau survit à la mort du VPS** : après destruction
permanente de `135.181.42.188`, (a) aucune découverte de pair n'est hard-câblée
sur ce node_id (binaire `default_curators=[]`) ; (b) n'importe qui monte sa
propre ancre, première-classe équivalente (une parmi N, façon dépôt F-Droid) ;
(c) les apps seedées restent joignables tant qu'un détenteur du hash BLAKE3
répond (content-addressing = vérité de joignabilité). **Triade anti-Sybil
obligatoire** sur tout artefact annuaire : signature Ed25519 (nouveau domaine) +
seuil réputation kudos pour l'agrégation + curation par signature curateur —
sans les trois, le résidu over-count THREAT_MODEL §15 row D régresse de M à H.
**Confidentialité default-OFF** : les requêtes utilisateur ne quittent JAMAIS la
machine ; le pull d'une ancre est un choix explicite, jamais un appel réseau
silencieux au boot. *Checklist complète 15 items : voir `design_review.md` C1-C6
+ §5 code-implications.*

## §5 Décisions Day 0 (D1-D5 gelées)

Décidées par panel adversarial (3 avocats + juge, `wdeedndsh`). Scoring G1 :
**D1 ✅ D2 ✅ D3 ⚠️ D4 ✅ D5 ✅** (détail board `design_review.md`).

### D1 — Substrat annuaire = `NodeDirectoryEntry` sibling
- **Retenu** : nouveau type signé `NodeDirectoryEntry` sous
  `DOMAIN_NODE_DIRECTORY_V1`, réutilisant *verbatim* la machinerie `CuratorList`
  (sign/verify, revision monotone, caps 256+par-champ, attention-set +
  `subscriptions.json`, ingest gossip 9-étapes subscription-gated, read-path
  `BrowseAggregator`). Porte `node_id` (le handle dialable durable manquant,
  aujourd'hui `#[serde(skip)]` `browse.rs:195`) + `Vec<{project_id, archive_hash,
  name, category, description}>` + `revision` monotone. Payload = liste
  **humainement affichable** (forme index F-Droid), PAS un digest Bloom.
- **Rejeté A** (surcharger `CuratorList`) : `CuratorProjectRef` n'a pas
  d'`archive_hash` et conflate `project_id==node_id` → 3 champs à ajouter,
  efface le reuse tout en gardant la dette sémantique + coût audit
  domain-overload. **Rejeté C** (SearchManifest) : mauvaise couche
  (search-coverage ≠ node-catalogue), trigger DEFER non atteint, plus net-new
  pour moins de valeur pivot. Prior art unanime : chacun a donné à
  l'auto-publication son propre type (NIP-65, Radicle INVENTORY, F-Droid,
  BEP-44).
- **Implications code** : nouveau struct `nexus-core-rs` ; `DOMAIN_NODE_
  DIRECTORY_V1` `canonical.rs` (copier précédent SeedRequest) ; sibling ingest
  arm `iroh_runtime.rs` réutilisant étapes 4-9 ; `BrowseSource::NodeDirectory` ;
  branche aggregator settant `node_id` depuis l'entrée (plus None) ; **helper
  générique `ingest<SignedList>`** (mitigation drift C1, Q2).

### D2 — Sort du push/PoW = FIX-A re-mint d'abord, indépendant
- **Retenu** : corriger le bug live EN PREMIER (Phase A), indépendamment du
  pivot. Stocker le payload `ProjectAnnouncement` **non-wrappé** dans l'outbox ;
  à chaque site replay (`runtime.rs:1513/1544/1615`) re-wrap PoW **frais**
  (`PowSolveCache` ≈ gratuit) ET **re-mint `EndpointAddr`/`BlobTicket`** depuis
  `my_endpoint_addr()`. **NE PAS affaiblir** `MAX_PROOF_AGE_SECS=1800` (le
  re-mint rend la fenêtre correcte, pas supprimée). Push retenu mais **rétréci**
  au burst NeighborUp live (issued_at frais par construction), jamais replay
  périmé. Le helper re-mint d'adresse est **réutilisé** par le path pull.
- **Rejeté** : FIX-B seul (rendre push caduc) → zéro découverte pendant le
  rollout tant que tous les pairs ne tournent pas le client pull. Affaiblir
  `MAX_PROOF_AGE_SECS` → anti-replay/flood/liveness intacts, re-mint est la
  correction.
- **Implications code** : forme outbox `PowEnvelope` figé → payload non-wrappé
  (`runtime.rs` persist + 3 boucles replay + `deploy.rs:661-687`) ; helper
  re-mint-adresse près `http.rs:1639-1662` (`mint_blob_ticket`). Pas de break
  wire (seul le QUAND du mint change).

### D3 — Modèle opérationnel VPS ⚠️ (arbitrage PO requis)
- **Retenu** : VPS = deux rôles bornés. (1) DIRECTORY (room SSB) : publie un
  `NodeDirectoryEntry`. (2) SEED (pub SSB, **borné**) : seede SEULEMENT MES apps
  + invites acceptées, budget disque + policy par-projet (Radicle), JAMAIS
  miroir universel. Headless : driver config-driven « seed ces project_ids au
  boot » + 1er appelant prod de `request_seed`. Ancre dans MON `config.toml`,
  jamais hard-codée.
- **Rejeté** : VPS miroir seed-everything (leçon SSB pubs surchargés) ; VPS DB
  centrale annuaire privilégiée (lock-3) ; tirer SearchManifest (C) comme
  l'annuaire (DEFER tient).
- **⚠️ + Implications** : D3 **touche la décision Day-0 D3/s73 (SearchManifest
  DEFER)** → exige `sprint75_pivot_proposal.md` (FAIT) + **sign-off PO** sur 3
  points AVANT Phase A. Le juge a clarifié : le pivot construit
  `NodeDirectoryEntry` (objet distinct), NE tire PAS SearchManifest → DEFER tient.
  Code : driver seed boot (`fetch_and_pin` project_ids non déployés localement) ;
  `request_seed` (`seed_protocol.rs:298` `#[allow(dead_code)]`) → 1er appelant ;
  `reannounce_seeds_at_boot` (`feed_sync.rs:160-200`) étendu acquire-then-pin.

### D4 — Durabilité catalogue distant = persister ancres + re-pull boot
- **Retenu** : forme F-Droid « fingerprint persiste, index re-fetché » — déjà
  l'archi (`iroh_runtime.rs:35-37` : attention-SET durable, entrées RAM-only).
  Persister les **node_ids d'ancre** + **re-pull actif au boot** des
  `NodeDirectoryEntry` abonnés. Ferme le gap load-bearing (`direct_entries`
  in-memory + restore OWN-only). **Foyer des carries S74** WIRE-1/WIRE-2/DBQ-1.
- **Rejeté** : persister les entrées distantes en durable (invite over-count/
  stale ; RAM-only+re-pull est le design prior-art).
- **Implications code** : routine boot itérant les pubkeys d'ancre abonnées,
  re-pull leurs blobs `NodeDirectoryEntry` (réutilise path curator gossip+blob).

### D5 — Wire additif 0-bump + pull multi-provider
- **Retenu** : purement ADDITIF 0-bump (nouveau DOMAIN + type, orthogonal aux
  `*_FORMAT_VERSION` — pattern S74 SeedRequest). Liveness = sonde pull live +
  BLAKE3 (OBSERVÉE, pas claim PoW-clock). **Fetch multi-provider IN-SCOPE**
  (carry PULL-2) : plumber les `seeder_node_id` de `SeedRegistry` dans le
  vecteur providers de `download()` (`fetch_ticket` dial un seul endpoint
  aujourd'hui).
- **Rejeté** : bump `*_FORMAT_VERSION` (policy pre-launch) ; single-provider
  (annuaire fragile si ancre offline) ; liveness PoW-clock (remplacée par sonde).
- **Implications code** : `DOMAIN_NODE_DIRECTORY_V1` ; serde `NodeDirectoryEntry`;
  multi-provider `download()` (lire `SeedRegistry` node_ids + node_id de l'entrée
  → `Vec<endpoint_id>`). BLAKE3 reste le gate intégrité.

## §6 Inventaire substrat (R4 — buildable vs net-new)

| Primitive | Statut | Gap |
|---|---|---|
| Machinerie crypto/ingest CuratorList (sign/verify, revision, caps, attention-set, gossip 9-étapes) | **buildable** (100% complet) | aucun — réutilisé verbatim |
| **Authoring write-path** (publier/signer/annoncer SON catalogue) | **net-new** | `POST /api/daemon/directory|curators/publish` ; aujourd'hui seuls `#[cfg(test)]` appellent `sign()` ; `create-curator-list.sh` écrit du JSON NON signé |
| **Durabilité catalogue distant** (primitive 5, **LOAD-BEARING**) | **net-new** | `direct_entries` in-memory, restore OWN-only ; FTS5 omet node_id+archive_ticket → trace inactionnable. **Le seul vrai trou archi.** |
| Multi-seeder pull | **partial** | `SeedRegistry` a N seeders mais `fetch_ticket` dial 1 endpoint ; `download()` accepte déjà un Vec |
| `request_seed` / driver seed headless | **net-new** | client `#[allow(dead_code)]`, 0 appelant prod ; tout seed = HTTP loopback interactif |
| Exposition node identity | **partial** | `node_id` `#[serde(skip)]` ; promouvoir (additif) OU `GET /api/daemon/nodes` ; + pages front Nodes/node-catalogue |
| Panneau Disponibilité + Curators UX | **buildable** | `AvailabilitySheet` déjà node-centrique ; `Curators.tsx` = template « ajouter une ancre » ; `App.tsx` lazy → `/nodes` + `/node/:nodeId` insèrent proprement |

**Chemin critique** : authoring (primitive 1) → durabilité (primitive 5). Les
deux sont le critical path ; multi-seeder/node-identity sont des enrichissements.

## §7 Plan Phase outline A-G

- **Phase 0** — Audit gate S74 (DONE = PASS `0e2fb6b`).
- **Phase A** — **FIX-A re-mint-on-replay** (D2) : outbox payload non-wrappé +
  re-wrap PoW frais + re-mint adresse aux 3 sites replay + boot restore. **E2E
  cross-machine Win↔Mac** (le bug live). Helper re-mint réutilisable. *Gate avant
  Phase C (C6).*
- **Phase B** — **`NodeDirectoryEntry` + `DOMAIN_NODE_DIRECTORY_V1` + authoring**
  (D1, primitive 1) : type signé sibling, domaine canonical, sign/verify réutil.
  CuratorList, route authoring `publish`, **helper générique `ingest<SignedList>`**
  (C1/Q2).
- **Phase C** — **Ingest annuaire + durabilité catalogue distant** (D4,
  primitive 5 LOAD-BEARING) : sibling ingest arm subscription-gated,
  `BrowseSource::NodeDirectory`, aggregator settant node_id, **re-pull boot des
  ancres abonnées**. Absorbe WIRE-1/WIRE-2/DBQ-1 dans le schéma.
- **Phase D** — **Pull multi-provider + node identity** (D5, carry PULL-2) :
  plumber `SeedRegistry` seeders → `download()` ; exposer node_id ; statut
  honnête « joignable-via-seeder » (Q7).
- **Phase E** — **Ancre VPS headless** (D3) : driver seed config-driven (section
  `[seed]`/`[directory]` lue au boot → `fetch_and_pin` + re-mint), 1er appelant
  prod `request_seed`, authoring VPS signé (builder boot ou endpoint loopback),
  unit systemd. + ack budget disque/GC.
- **Phase F** — **Browse node-centrique (front)** : pages `/nodes` +
  `/node/:nodeId` (`App.tsx` lazy), node-Browse cohabite/supersede (Q6),
  intention « ajouter une ancre » (template `Curators.tsx`), UX cold-start
  1er-run (C4), intégration `AvailabilitySheet`.
- **Phase G** — **Wrap-up + acceptance** : E2E « survives-VPS-death »
  cross-machine (Win/Mac/VPS via SSH), verification + audit_plan S76, hygiène
  carries S74 (CARRY-5 clamp, CARRY-2 Rejected-terminal, PULL-1 dedup, FORK-1
  entry-cap, WEB-1 toggle), doc META-1/CARRY-1 + PATTERNS.

Sprint pair-like (consolidation+feature) : Phase A est dette/fix dédiée (le bug
live) avant le feature body B-F. **Réaliste** : A-C = core critical path ; D-F =
enrichissement+UX+VPS ; G wrap. Si débordement → slice borné (cf. scope cuts).

## §8 Items carry / dette (15 P2 audit S74 + externes)

**8 P2 à CONCEVOIR dans le pivot** (pas rustiner) : WIRE-1 (indexer
ReleasePublished par nom) + WIRE-2 (seed-count keyé (pid,hash)) + WIRE-3
(croissance reprovide → propriété pre-launch) + SEED-1 (clamp ts registry) +
SEED-2 (cap nonce/registry) + PULL-2 (multi-provider, **devient D5**) + CARRY-3
(sanitize aggregator byzantine) + DBQ-1 (keep_online hash-SOT) → Phases C/D.
**5 hygiène hors-pivot** (Phase G) : CARRY-5 (clamp offset/q), CARRY-2
(Rejected-terminal sur trip), PULL-1 (dedup provenance), FORK-1 (entry-cap),
WEB-1 (seed toggle depuis selfSeeding). **2 doc/process** : META-1 (règle
PATTERNS GAP-carry), **CARRY-1 = LT-2 ARMÉ** (tag v1.0 DÉJÀ poussé sur origin ;
flipper LT-2 + dry-run Radicle privé — Phase G doc). **10 P3** : optionnels, pris
opportunément si une phase touche la zone. **Externes inchangés** : P2-A-1 rand,
P2-AUDIT-2 iroh, T-NN+2 wasm, P3-OS-1, LT-3/4/7.

## §9 Scope cuts (exhaustif)

1. **SearchManifest** (digest couverture Bloom, agrégation multi-curateurs,
   `DOMAIN_SEARCH_MANIFEST_V1`, rôle index-node, query fédérée) — **reste
   DIFFÉRÉ** (s73 §5, post-launch). L'annuaire ≠ SearchManifest (pivot_proposal).
2. **Tantivy** — gelé (gate post-S75 >50K docs). FTS5 reste l'engine.
3. **GC reaper / budget disque enforced** — acknowledged (C3) mais déféré
   post-launch ; S75 borne par policy config, pas de reaper LRU/TTL automatique.
4. **Recherche cross-nœud fédérée** — hors scope (c'est SearchManifest).
5. **Approbation pair pour seed distant** — le seed reste volontaire/invite (S74),
   pas de nouveau flux d'approbation.
6. **Mobile/Electron client** — le front reste le shell React ; pas de nouveau
   client.
7. **Migration wire post-tag** — pas de bump `*_FORMAT_VERSION` ; tout additif.
8. **GPU partagé cross-machine** — décalé S76 (amendement §12).
9. **Sharding pipeline** — décalé S77.
10. **Kudos-threshold tuning empirique** — le seuil réputation pour l'agrégation
    annuaire est posé conservateur, calibration post-launch (s73 §8 Q3).
11. **Multi-ancre UX avancée** (priorité/ordering d'ancres, fallback chains) —
    S75 livre l'abonnement à N ancres ; l'UX avancée de priorisation est différée.
12. **Bloom/Merkle digest** — non posé (c'est la forme SearchManifest rejetée).

## §10 Questions ouvertes (à trancher au plan/preflights, NON pré-décidées)

1. **Q1** champ set `NodeDirectoryEntry` : `archive_ticket` re-minté au pull
   (helper FIX-A) vs `archive_hash`-only (client re-dérive en dialant node_id).
2. **Q2** sibling ingest : helper générique `ingest<SignedList>` (mitigation
   drift) vs arm copié — call au preflight Phase B.
3. **Q3** config ancre : réutiliser `default_curators` (pubkeys) vs nouveau
   `default_anchors` (node_ids) — même forme, sémantique différente.
4. **Q4** policy SEED bornée : budget disque + accept-list par-projet, abstraite
   (pas de knob numérique pour user non-technique).
5. **Q5** ordering fetch multi-provider : node_id annuaire d'abord puis seeders
   SeedRegistry, ou race ; budget timeout (PULL-2).
6. **Q6** re-model Browse : node-centrique remplace la grille project-flat vs
   toggle cohabitation — design intention UX avant phase front.
7. **Q7** statut honnête « joignable-via-seeder » quand le nœud publisher est
   down mais un seeder détient le BLAKE3 — nouveau bucket de statut.
8. **Q8** carries S74 dans le schéma : WIRE-1/WIRE-2/DBQ-1 conçus au plan, phase
   ownership assignée.

## §11 Risk register

| # | Risque | L | I | Mitigation |
|---|---|---|---|---|
| R1 | **Drift ingest-arm dupliqué** (C1) | M | M | helper générique `ingest<SignedList>` = livrable Phase B (Q2) |
| R2 | **Stale-catalog/over-count résidu** (C2) | M | M | BLAKE3 + sonde live = autorité, jamais le compteur ; THREAT_MODEL §15 |
| R3 | **Gap acquisition seed headless VPS** (C3) | M | H | driver config-driven `fetch_and_pin` boot + 1er appelant `request_seed` = Phase E |
| R4 | **Cold-start vide vs Radicle/F-Droid** (lock-3, C4) | H | M | trade accepté ; UX « ajouter une ancre » 1er-run claire (Phase F), pas écran vide mort |
| R5 | **Tripwire lock-3** (C5) : hard-coder l'ancre dans le binaire | L | H | guard/review : tout `default_*` ancre non-vide compilé = DESIGN-CONFLICT |
| R6 | **Gap découverte fenêtre-rollout** (C6) | M | H | FIX-A landé + E2E cross-machine AVANT que pull soit gated dessus (Phase A→C) |
| R7 | **D3 touche Day-0** | — | — | `pivot_proposal.md` + sign-off PO AVANT Phase A (gate) |

## §12 Amendement roadmap v5

Roadmap v5 actuel : S75=GPU, S76=sharding. **Amendement** : la découverte PULL
est *fondationale* (bug live : apps invisibles aux nouveaux ; on ne peut pas
partager du GPU entre nœuds qu'on ne découvre pas) → **S75 = découverte PULL
node-centrique ; GPU → S76 ; sharding → S77**. À enregistrer dans
`roadmap_v5_factory_complete_vision.md`. Confirmé par le choix PO « kickoff
complet direct ».

## §13 Checkpoint de validation (arbitrage PO AVANT plan détaillé/code)

5 questions — dernier moment pour pivoter sans coût :
1. **D3 / pivot_proposal** : valides-tu les 3 sign-offs (annuaire =
   `NodeDirectoryEntry` ≠ SearchManifest différé ; ancre = catalogue-publisher
   borné pas aggregator ; substrat B-hybride) ? *(gate Phase A)*
2. **D2 FIX-A d'abord** : OK pour corriger le bug live en Phase A indépendante
   (re-mint adresse+PoW, sans affaiblir la fenêtre 1800s) avant le feature body ?
3. **Amendement roadmap** : OK S75=découverte, GPU→S76, sharding→S77 ?
4. **Scope** : A-C critical path garanti ; D-F (multi-provider/VPS/front) +
   G wrap — OK pour slice borné si débordement, ou tout-ou-rien ?
5. **VPS test live** : utiliser les assets SSH (mac 192.168.1.53 + vps
   135.181.42.188) pour l'acceptance « survives-VPS-death » cross-machine en G ?

Audit gate pattern rappel : S75 produira en sortie `sprint75_verification.md` +
`sprint76_audit_plan.md` (Phase G).
