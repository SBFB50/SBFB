# Post-S75 Mini-Cycle UX-ARRIVAL Preflight (deep G8)

Date: 2026-06-11
HEAD: `10a311c`
Verdict: **PLAN-ADAPT**

> Mini-cycle hors-sprint (equivalent Cas B a phase unique). Pas de section
> plan numerotee : la source de verite est le handoff
> `.planning/active/post_s75_ux_arrival_handoff.md` §3 (design fige PO) +
> la decision PO §2 (C-hybride + rate-limit, GELEE, non rebattue ici).

## Evidence Rules
- Claim policy : chaque affirmation cite un chemin/ligne, une sortie de
  commande, une URL datee, ou une hypothese explicite.
- Local sources read : `prompts/agent/preflight.md` ;
  `.planning/active/post_s75_ux_arrival_handoff.md` ;
  `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (gate ingest,
  `NodeDirectoryAnnouncement`, `CuratorRuntime` fields, `is_subscribed`) ;
  `crates/nexus-core-rs/src/node_directory.rs` (wire `NodeDirectory`/
  `CatalogApp`/`NodeDirectoryEntry`) ;
  `crates/nexus-shell-daemon/src/runtime.rs` (gossip dispatch + PoW verify +
  `handle_directory_announcement` + `handle_project_announcement`) ;
  `crates/nexus-shell-daemon/src/seed_registry.rs` (pattern cap/TTL/eviction/
  clamp/lowercase) ; `crates/nexus-shell-daemon/src/http.rs` (`BrowseEntryView`,
  `list_nodes`, `NodesResponse`/`NodeSummary`, `nodes_response`) ;
  `crates/nexus-shell-daemon-core/src/browse.rs` (`BrowseEntry`, `node_id`
  `#[serde(skip)]`, 3 bras aggregate) ;
  `crates/nexus-shell-daemon-core/src/browse_limiter.rs` (GCRA governor) ;
  `web/src/api/daemon.ts` (Zod) ; `web/src/pages/Browse.tsx`
  (`dedupeBrowseEntries` inline) ; `web/src/pages/Nodes.tsx` ;
  `docs/rust/PATTERNS.md` §P58.2/§P59.2/§P59.3 ;
  `docs/shell/PATTERNS.md` §P37 ; `docs/security/THREAT_MODEL.md` §15/§15.1 ;
  `.planning/active/sprint76_audit_plan.md` (track discriminateur).
- Commands run : `git rev-parse --short HEAD` -> `10a311c` ;
  `git log --oneline -8 -- iroh_runtime.rs` (S75 B/C/D recents) ;
  `grep dashmap Cargo.toml` -> `dashmap = { workspace = true }` (dep deja la,
  0 nouvelle dep) ; recherches web datees 2026-06-11 (libp2p PR#577, BitTorrent
  DRDoS WOOT15, Nostr NIP-65).

## Scope
- Plan source : handoff §3 (3.1 registre observed daemon ; 3.2 cle `observed`
  sur `/nodes` ; 3.3 flag `from_subscribed` serialize-only sur `/browse` ;
  3.4 split front grille/section + section observed `/nodes` ; 3.5 garde-fous).
- Target files :
  - `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (champ
    `observed_directories` + gate observe a l'etape 4 + accessor snapshot).
  - `crates/nexus-shell-daemon/src/http.rs` (`BrowseEntryView` + `from_subscribed`,
    `NodesResponse` + `observed`, `list_nodes`).
  - `crates/nexus-shell-daemon/src/runtime.rs` (rate-limit per-node_id avant
    `handle_directory_announcement`, ou capture observed apres drop d'abonnement).
  - `crates/nexus-shell-daemon-core/src/browse_limiter.rs` (pattern GCRA reutilise
    pour le rate-limit observed, ou nouveau module `observed_limiter.rs`).
  - `web/src/api/daemon.ts` (Zod `from_subscribed` + `observed`/`ObservedNode`).
  - `web/src/pages/Browse.tsx` (split grille/section decouverte).
  - `web/src/pages/Nodes.tsx` (section observed + CTA S'abonner).
  - Tests Rust (iroh_runtime, http, runtime) + Vitest (Browse, Nodes, daemon).
- Deps/APIs/specs : AUCUNE nouvelle (dashmap + governor deja workspace deps ;
  iroh 0.98 pin inchange ; serde inchange).
- Security/protocol surfaces : registre RAM ingest non-sollicite (nouvelle
  surface) ; `/browse` + `/nodes` (loopback-local, additif) ; aucun wire P2P
  nouveau (0 DOMAIN, 0 `*_FORMAT_VERSION` bump).
- Tests expected : cf. handoff §3.5 (registre observed cap/TTL/rate-limit/
  eviction/clamp/lowercase/abonne-exclu ; `from_subscribed` own/abonne/inconnu ;
  `/nodes` enveloppe+observed pinnes producteur ; Vitest split grille/section,
  cap affichage, section vide non rendue, /nodes observed + CTA, Zod tolerant).

## S1a OSS Prior Art
- Domain : "observed peers / unsolicited announcements quarantine" + bounded
  metadata cache + rate-limited ingest dans les reseaux P2P.
- Sources :
  - go-libp2p PR #577 "identify: be more careful about the addresses we store"
    (Stebalien) + peerstore TTL constants (`TempAddrTTL` = 2 min pour adresses
    non verifiees, `ConnectedAddrTTL` quasi-permanent) —
    https://github.com/libp2p/go-libp2p/pull/577 ,
    https://pkg.go.dev/github.com/libp2p/go-libp2p/core/peerstore (consulte
    2026-06-11). Lecon directe : "the protocol avoids recording a remote
    peer's address in the peerstore if they aren't advertising it, since
    recording and sharing bad ephemeral addresses can cause address explosion".
  - BitTorrent DRDoS (USENIX WOOT15, Adamsky et al.) : amplification factor
    jusqu'a 50x via reflection UDP ; mitigation DHT = jeton get_peers/FIND_NODE
    requis avant announce_peer (10 min) —
    https://www.usenix.org/system/files/conference/woot15/woot15-paper-adamsky.pdf
    (consulte 2026-06-11). Lecon directe : une annonce non-sollicitee ne doit
    JAMAIS declencher un fetch/dial sortant (vecteur d'amplification reflexive).
  - Nostr NIP-65 (kind:10002 relay list) = **replaceable event** : seul le
    dernier par (pubkey, kind) est stocke, les anciens jetes ; spam toolkit =
    PoW + WoT + rate-limit + quotas — https://nips.nostr.com/65 ,
    https://nips.nostr.com/1 (consulte 2026-06-11). Lecon directe : dedup
    revision-keyed par node_id (garder le dernier), rate-limit l'ingest.
  - Prior art cite IN-CODE (`node_directory.rs:22`) : NIP-65, Radicle
    INVENTORY, F-Droid per-repo index, BEP-44 — le type signe de
    self-publication existe deja et est aligne.
- Finding : **APPROACH-ALIGNED** sur la forme (registre borne TTL+cap+rate-limit,
  dedup revision-keyed, ne pas dialer sur non-sollicite) — le pattern
  `SeedRegistry` (seed_registry.rs) materialise deja exactement cette posture
  pour les seeders ; le registre observed est sa transposition node-directory.
  **MAIS** un sous-finding bloquant cote DESIGN-DU-HANDOFF (pas cote OSS) : le
  handoff §3.1 demande de retenir `{node_id, revision, app_count, last_seen}`
  pour un noeud NON-ABONNE — or `revision` et `app_count` ne sont PAS dans
  l'enveloppe gossip cheap (cf. S4), ils vivent dans le BLOB signe que le code
  actuel ne fetch QUE pour les abonnes (etape 5, apres le gate etape 4). Les
  fetch pour un non-abonne contredirait frontalement la mitigation
  anti-amplification §15.1 (cf. S3). -> adaptation requise (PLAN-ADAPT), pas
  un rejet du produit : retenir seulement la metadata cheap-envelope.
- Impact : adaptation cataloguee en `## Plan adaptation` ci-dessous.

## S1b Dependencies, CVEs, Release Notes
- Scanned : dashmap, governor, serde, iroh 0.98 (toutes deja compilees en prod).
- Commands/sources :
  - `grep dashmap crates/nexus-shell-daemon-core/Cargo.toml` ->
    `dashmap = { workspace = true }` (l.73) ; `use dashmap::DashMap`
    (iroh_runtime.rs:68). Le registre observed = `DashMap<[u8;32], _>` sibling,
    0 nouvelle dep.
  - `browse_limiter.rs:11` `use governor::{DefaultKeyedRateLimiter, Quota,
    RateLimiter}` — GCRA per-key deja en prod (quota per_minute). Le rate-limit
    observed per-node_id le reutilise verbatim, 0 nouvelle dep.
  - Aucune dep ajoutee ni bumpee -> graphe transitif (`Cargo.lock` /
    `cargo tree -d`) NON sollicite (P2-PREFLIGHT-TRANSITIVE-DEPTH : la regle
    declenche sur "ajoute/bump une dep" ; ici 0 dep). Lecon S72-C/D
    (schemars 0.8 vs 1.2) non applicable.
- Finding : **clean**. Mini-cycle 100% code + tests sur deps figees.

## S2 Historical Decisions
- Commands : `git log --oneline -8 -- iroh_runtime.rs` (821aa8c C / f6637d3 B /
  ... S54/S10/S9) ; `git log --oneline -5 -- seed_registry.rs` (0010450 D /
  821aa8c C / 66a9409 S74-F) ; `grep -n "discriminateur" sprint76_audit_plan.md`.
- Decisions crossed :
  - **S75 Phase C** (`821aa8c`) a DELIBEREMENT pose le drop des annonces
    non-abonnees a l'etape 4 (`iroh_runtime.rs:967-976`, doc l.940-943 : "Drop a
    non-subscribed anchor BEFORE any fetch... the curation leg of the anti-Sybil
    triad"). Reversion status : PAS une reversion — l'UX-ARRIVAL ne RETIRE pas
    ce drop, il l'ETEND (capter la metadata cheap AVANT de retourner
    `NotSubscribed`). Le fetch reste interdit pour les non-abonnes. Non-bloquant.
  - **Verrou 5 anti-recentralisation** (resolution UNIQUEMENT sur annuaires
    ABONNES, THREAT_MODEL §15.1 row oracle) : decision figee, rationale valide.
    L'observed registry NE doit PAS le violer (pas de fetch/dial observe). Le
    design adapte (cf. S3 + plan adaptation) le respecte -> pas de conflit.
  - **Discriminateur curator-vs-ancre** (`sprint76_audit_plan.md:294`, track
    P2 route S76) : recoupe la zone `/nodes` mais ne bloque PAS — l'UX-ARRIVAL
    ajoute une famille de lignes (observed) distincte des "en-attente"
    (subscribed-sans-catalogue) ; le discriminateur reste un P2 S76. Non-bloquant.
  - Reverse-commit check : aucun commit `revert`/`rejected`/`scope-cut` ne
    touche `observed`/`unsolicited`/`non-subscribed` (grep sources = 0 hit hors
    planning docs). Aucune decision anterieure n'interdit un registre observed.
- Finding : **clean** (aucune decision gelee contredite ; le drop S75-C est
  etendu, pas inverse).

## S3 Local Patterns And Threat Model
- Threats/contracts checked : THREAT_MODEL §15.1 (surface PULL node-centrique).
- **Finding BLOQUANT-de-conception (resolu par adaptation, pas DESIGN-CONFLICT)** :
  la ligne §15.1 "Oracle blob-serve drive-by + amplification de dials"
  (`THREAT_MODEL.md:876`) mitige l'amplification par "resolution UNIQUEMENT
  sur annuaires ABONNES". Si le registre observed FETCHE le blob d'un noeud
  non-abonne (pour lire `revision`/`app_count`), il OUVRE un nouveau vecteur
  d'amplification : annonce non-sollicitee -> fetch_ticket -> dials sortants
  vers une source non-abonnee. Confirme SOTA (BitTorrent DRDoS 50x, libp2p
  "don't store what you didn't ask for"). **Mitigation = NE PAS fetcher pour
  observed** : ne retenir que la metadata de l'enveloppe cheap (node_id + ts de
  reception), bornee (cap+TTL+rate-limit, pattern SeedRegistry §P59.2/.3). Ceci
  N'EST PAS une regression — c'est une nouvelle surface, modelisee neuve, dont
  la mitigation est la NON-introduction du fetch. THREAT_MODEL §15.1 a etendre
  d'une row (nouvelle surface "registre observed RAM") dans le commit.
- Autres contrats :
  - §P59.2 (cap DANS la primitive) : le registre observed doit clamper
    cap/TTL/`last_seen=min(now,claimed)` a l'interieur, jamais par convention
    d'appelant. SeedRegistry::record (seed_registry.rs:142-202) = modele exact.
  - §P59.3 (hex lowercase write+read) : node_id stocke + lu en lowercase
    (anti-monopolisation 2^64 variantes de casse). Le pubkey est deja parse en
    `[u8;32]` (cle DashMap binaire = naturellement insensible a la casse hex),
    mais toute serialisation hex de sortie doit etre lowercase (cf. `hex::encode`
    deja lowercase, http.rs:1881).
  - §P59.4 (TERMINAL guardrail) : N/A (pas de chemin result-ingress ici).
  - Verrous anti-recentralisation 1-5 : additive jamais substitutive (la grille
    reste le superset MES-sources, la section observed est l'ambiant separe) ;
    rien pre-rempli (verrou 3) ; CTA observed = `addAnchor` = subscribe explicite
    (verrou 5, l'utilisateur choisit). Respectes.
- HARDENING_ROADMAP status : aucune pre-requirement de sprint ouverte sur cette
  surface (mini-cycle hors-sprint ; S76 pas encore ouvert).
- Finding : **non-bloquant APRES adaptation** — la surface observed est neuve
  (pas de regression d'un T0-T5 couvert), sa mitigation (no-fetch + bornes) est
  un ajout. Bloquant SEULEMENT si le code fetchait pour observed -> l'adaptation
  l'interdit explicitement. THREAT_MODEL §15.1 +1 row a livrer dans le commit.

## S4 Protocol And Wire Invariants
- Wire/security files checked : `node_directory.rs` (wire signe) ;
  `iroh_runtime.rs` `NodeDirectoryAnnouncement` (enveloppe gossip) ; `http.rs`
  `NodesResponse`/`BrowseEntryView` (loopback JSON) ; `daemon.ts` (Zod) ;
  `browse.rs` `BrowseEntry`.
- VERSION/domain/canonical status :
  - `NODE_DIRECTORY_FORMAT_VERSION = 1` (node_directory.rs:84) INCHANGE ;
    `ANNOUNCEMENT_VERSION` INCHANGE ; `DOMAIN_NODE_DIRECTORY_V1` INCHANGE ;
    0 nouveau DOMAIN. Aucun wire P2P touche.
  - `observed` (cle `/nodes`) et `from_subscribed` (cle `/browse`) sont
    LOOPBACK-LOCAL (daemon -> shell same-origin), PAS des wire P2P : 0 bump
    requis (precedent exact `self_pin_enabled` S75-F, `is_own` S74-G).
- **Trace producteur -> consommateur (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH)** :

  1. **Enveloppe gossip `NodeDirectoryAnnouncement`** (iroh_runtime.rs:166-181) :
     `{ v: u16 (rename "v"), node_pubkey_hex: String (rename "node"),
     blob_ticket: String (rename "ticket") }`. **NE CONTIENT NI `revision` NI
     `app_count`** — ces deux champs vivent dans le BLOB `NodeDirectoryEntry`
     (`directory.revision`, `directory.catalog.len()`, node_directory.rs:173/178)
     fetche a l'etape 5 (iroh_runtime.rs:978-991), APRES le gate abonnement
     (etape 4, l.968). CONSEQUENCE DURE : la metadata observed disponible SANS
     fetch = `{ node_id (depuis node_pubkey_hex), last_seen (clock local) }`
     uniquement. `revision`/`app_count` exigeraient un fetch interdit (S3).

  2. **`from_subscribed` (`/browse`)** : producteur = `BrowseEntryView`
     serialize-only (http.rs:903-908, pattern §P58.2). `entry.node_id`
     (`#[serde(skip)]`, browse.rs:204-205) EST disponible au point de
     serialisation : pose a `Some(ann.node_id)` pour les annonces `direct`
     recues via gossip (runtime.rs:2130), `Some(node_id_hex)` pour
     `nodedirectory` (browse.rs:780), `None` pour les `direct` self-publies
     (browse.rs:914) et `curator` (browse.rs:671). Calcul a la serialisation :
     `from_subscribed = is_own || (entry.node_id ∈ attention set)` via
     `state.curator_runtime.is_subscribed(&parse_pubkey)` (accessor existe,
     iroh_runtime.rs:681) OU `subscribed_pubkeys_hex()` (l.670, lecture sans
     lock-order neuf — DashMap concurrent, deja appele dans `list_browse`
     contexte). Consommateur = Zod `BrowseEntrySchema` (daemon.ts:152-187) :
     ajouter `from_subscribed: z.boolean().optional()` (meme tolerance runtime
     que `is_own` l.185). `/browse` reste byte-identique POUR LES CHAMPS
     EXISTANTS ; `from_subscribed` est un nouvel champ additif serialize-only
     (le test `BrowseListResponse` `#[serde(deny_unknown_fields)]` cote
     #[cfg(test)] http.rs:698 ignore le champ flatten car il deserialise
     `BrowseEntry` nu — a VERIFIER : `BrowseEntryView` flatten ajoute la cle au
     niveau entry, le test deserialise `entries: Vec<BrowseEntry>` qui n'a pas
     `deny_unknown_fields` sur `BrowseEntry` lui-meme -> OK, comme `is_own`).

  3. **`observed` (`/nodes`)** : producteur = `NodesResponse` (http.rs:1852-1855,
     enveloppe `.strict()` cote Zod) -> ajouter `observed: Vec<ObservedNode>`.
     `ObservedNode { node_id: String, last_seen: i64/u64 }` (PAS `revision`/
     `app_count` — cf. trace 1). Consommateur = `NodesResponseSchema`
     (daemon.ts:481-485, `.strict()` ENVELOPPE) -> ajouter
     `observed: z.array(ObservedNodeSchema)`. Comme l'enveloppe est `.strict()`
     ET producteur+consommateur shippent dans le MEME commit, le Rust DOIT
     toujours emettre la cle `observed` (jamais absente) ; le row
     `ObservedNodeSchema` reste tolerant (pas `.strict()`, regle P37). Pinne par
     un test producteur (forme enveloppe) cote Rust.
- Day 0 status : **preserved** — iroh 0.98 pin, visibilite 2 etats, archive zip,
  feed raw-op extensible, pre-launch additive policy. Aucune Day 0 touchee.
- Finding : **clean sur les invariants wire** (0 bump, 0 DOMAIN, loopback-local
  additif, traces faites). Le SEUL ecart est la metadata observed reduite a
  `{node_id, last_seen}` (S4 trace 1 + S3) -> capture en plan adaptation.

## Plan Adaptation
PLAN-ADAPT motive par S1a (APPROACH-ALIGNED globalement, mais le champ-set
observed du handoff est irrealisable sans violer S3/S4).

- **Original plan** (handoff §3.1 + §3.2) : registre observed
  `{node_id, revision, app_count, last_seen}` ; `/nodes` `observed` avec ces 4
  champs.
- **Evidence requiring adaptation** :
  - `NodeDirectoryAnnouncement` (iroh_runtime.rs:166-181) ne porte QUE
    `{v, node, ticket}` — `revision`/`app_count` sont dans le blob signe, fetche
    seulement a l'etape 5 APRES le gate abonnement (l.968 vs l.978).
  - THREAT_MODEL §15.1 (THREAT_MODEL.md:876) mitige l'amplification par
    "resolution UNIQUEMENT sur annuaires ABONNES" ; fetcher le blob d'un
    non-abonne reintroduit le vecteur (confirme SOTA : BitTorrent DRDoS 50x,
    libp2p PR#577).
- **Corrected approach** :
  1. **Registre observed = metadata CHEAP-ENVELOPPE seulement** :
     `observed_directories: DashMap<[u8;32], ObservedDirectory>` ou
     `ObservedDirectory { last_seen: u64 }` (le node_id = la cle). PAS de
     `revision`, PAS de `app_count` (les obtenir = fetch interdit). Champ sibling
     de `directories` dans `CuratorRuntime` (iroh_runtime.rs:539) — meme struct,
     meme DashMap, 0 lock-order neuf (reponse point 6 : pas de collision de
     responsabilite, `directories`=abonnes/full-catalog vs `observed`=non-abonnes/
     metadata, mutuellement exclusifs par le gate abonnement).
  2. **Point de capture** : a l'etape 4 de `process_directory_announcement_bytes`
     (iroh_runtime.rs:967-976), AVANT le `return Err(NotSubscribed)`, enregistrer
     `observed_directories.insert(ann_pubkey, ObservedDirectory{ last_seen=now })`
     borne (cap ~256 + eviction stalest + TTL 48h purge paresseuse, pattern
     SeedRegistry seed_registry.rs:154-179 + 93-99), `last_seen=min(now,claimed)`
     clampe DANS la primitive (§P59.2) — ici `now` = clock local, pas de ts
     reclame dans l'enveloppe donc clamp trivial mais pose pour coherence.
     Un noeud ABONNE n'entre JAMAIS dans observed (le gate l.968 ne passe la
     branche observed que sur `!is_subscribed`). Eviction de l'entree observed
     d'un node si/quand il devient abonne (transition subscribe -> purge observed).
  3. **Rate-limit per-node_id** (point 2) : 2e etage distinct du semaphore global
     `MAX_INFLIGHT_ANNOUNCEMENTS` (iroh_runtime.rs:491, qui borne la CONCURRENCE
     pas la frequence par-noeud). Reutiliser le pattern GCRA `governor`
     (browse_limiter.rs:11-32, quota per_minute keyed-by-String) : un
     `ObservedIngestLimiter` quota ~1/min keyed-by-node_id_hex, verifie a la
     reception AVANT de toucher le registre observed. Place : dans
     `runtime.rs` `handle_directory_announcement` (la ou `browse_limiter` est
     deja appele pour browse_request, runtime.rs:1631) OU dans
     `process_directory_announcement_bytes` etape 4. Decision d'implementation :
     pour les NON-abonnes uniquement (les abonnes passent par le fetch+verify
     deja borne par le semaphore + le dedup revision strict). Note ordering
     (reponse point 2) : le rate-limit observed s'applique APRES le PoW verify
     (runtime.rs:1616, deja le 1er filtre) et APRES le gate abonnement
     (seulement les non-abonnes l'atteignent) ; c'est une EXTENSION du meme
     etage, pas un fetch. La signature Ed25519 du blob n'est PAS verifiee pour
     observed (pas de fetch) — la garantie d'authenticite de la metadata observed
     repose sur le PoW gossip seul ; documenter honnetement (un observed node_id
     est "un pubkey qui a emis une annonce PoW-valide", pas "un pubkey
     Ed25519-verifie"). C'est suffisant pour une METADATA d'amorce non-autoritaire
     (le CTA force un subscribe explicite avant tout fetch/verify reel).
  4. **`/nodes` `observed`** : `ObservedNode { node_id: String, last_seen }`
     (2 champs, pas 4). Snapshot via nouvel accessor
     `CuratorRuntime::observed_snapshot()` parallele a `directory_snapshot()`
     (http.rs:1902). `nodes_response()` (http.rs:1876) etendu pour peupler
     `observed`. Zod `ObservedNodeSchema` tolerant ; enveloppe `.strict()` +1 cle.
  5. **Front** : section `/nodes` "Noeuds decouverts sur le reseau" depuis
     `result.body.observed` (CTA S'abonner = `addAnchor`, copy "s'annonce sur le
     reseau — abonne-toi pour voir son catalogue") — SANS app_count/revision
     (non disponibles ; afficher node_id tronque + "vu recemment"). `cold-start`
     `isEmpty` (Nodes.tsx:147) doit aussi compter `observed.length` (un observed
     non-vide n'est pas vide). Grille `/browse` split inchange (utilise
     `from_subscribed`, lui DISPONIBLE).
- **File/test delta vs plan** :
  - SUPPRIME du scope : champs `revision` + `app_count` sur la metadata observed
    (daemon + `/nodes` `observed` + front). Le handoff §3.5 "tests registre
    (... )" reste, MOINS l'assertion revision/app_count observed.
  - AJOUTE : test "observed node ne declenche AUCUN fetch" (anti-amplification,
    assertion centrale S3 — ex. un `process_directory_announcement_bytes` sur un
    non-abonne ne touche pas `BlobsClient`/`fetch_ticket`) ; test "rate-limit
    per-node_id observed" (2e etage) ; test "subscribe purge l'entree observed".
  - AJOUTE : THREAT_MODEL §15.1 +1 row (surface "registre observed RAM" : ingest
    non-sollicite borne, no-fetch, no-dial ; residual = metadata PoW-only
    non-Ed25519-verifiee, acceptable car non-autoritaire + CTA subscribe gate).
  - INCHANGE : `from_subscribed` (§3.3, realisable tel quel) ; split grille/
    section (§3.4, realisable, mais cf. risques) ; bornes registre (§3.1, cap/
    TTL/eviction/clamp/lowercase tels quels).

## Risks And Scope Cuts
- Blocking risks : aucun apres adaptation (le seul bloquant — fetch observe —
  est explicitement interdit par le plan adapte).
- Non-blocking risks (carry-over / vigilance review) :
  - **Split front sur l'ensemble DEDUPE** : `dedupeBrowseEntries`
    (Browse.tsx:188) collapse `(project_id, archive_hash)` AVANT le rendu. Si une
    app arrive a la fois en `direct`-inconnu ET `nodedirectory`-abonne, le dedup
    garde le representant le plus riche ; la CLASSIFICATION grille-vs-section doit
    se faire sur l'entree FUSIONNEE (un merge ou un cote est abonne =>
    `from_subscribed` doit etre OR-e dans le merge, sinon l'app abonnee tomberait
    dans "decouverte"). Le merge actuel (Browse.tsx:203) ne propage pas
    `from_subscribed` -> l'etendre (OR sur `from_subscribed`, comme le OR sur
    `status` reachable). Test Vitest decisif requis.
  - **HeroSection `entries[0]`** (Browse.tsx:85) : apres le split, le hero doit
    pointer sur la GRILLE (sources MES), jamais sur la section decouverte (sinon
    une app non-sollicitee finit "En vedette"). Brancher le hero sur le 1er de la
    grille.
  - **Enveloppe `.strict()` + cle `observed`** : producteur Rust DOIT emettre
    `observed` toujours (meme `[]`), sinon Zod `.strict()` rejette un `/nodes`
    d'un daemon a jour. Comme les 2 cotes shippent ensemble c'est OK ; ne PAS
    rendre `observed` optionnel cote Rust.
  - **Authenticite observed** : metadata PoW-only (pas Ed25519). Acceptable
    documente (non-autoritaire, CTA subscribe gate avant tout fetch). Un node_id
    observe peut etre un pubkey qui n'a jamais signe de catalogue valide —
    s'abonner puis ne rien voir = la ligne "en-attente" existante (honnete).
  - **Discriminateur curator-vs-ancre** reste P2 S76 (sprint76_audit_plan.md:294)
    — non aggrave par ce mini-cycle (observed = famille distincte).
  - **Duress freres pre-existants** (THREAT_MODEL §15.1 row "surfaces front F")
    — non touche ici ; le CTA observed = subscribe (deja duress-gate cote
    handler ? a verifier en review, mais subscribe n'ajoute pas a l'attention set
    sous duress par construction, cf. http.rs test l.6882).
- Scope cuts still honored : 0 bump `*_FORMAT_VERSION` ; 0 nouveau DOMAIN ;
  0 nouvelle dep ; pre-launch additive policy (CLAUDE.md) ; verrous 1-5 (additif
  jamais substitutif ; rien pre-rempli ; subscribe explicite) ; iroh 0.98 pin ;
  "heberger != publier, seeder != auteur" (observed = juste un pubkey entendu,
  aucune claim de provenance).

## Action
- **PLAN-ADAPT** : implementer l'approche corrigee (registre observed metadata
  cheap-envelope SANS revision/app_count, no-fetch/no-dial pour non-abonnes,
  rate-limit per-node_id 2e etage GCRA, `from_subscribed` serialize-only,
  `/nodes` `observed` 2-champs, split front sur ensemble dedupe + hero sur
  grille). Le commit body doit citer ce fichier et documenter : "Plan handoff
  proposait observed={node_id,revision,app_count,last_seen} ; preflight S1a/S4
  a identifie que revision+app_count ne sont pas dans l'enveloppe gossip cheap
  et exigeraient un fetch interdit par THREAT_MODEL §15.1 (S3) ; adapte a
  observed={node_id,last_seen} no-fetch + THREAT_MODEL §15.1 +1 row."
- Pas de DESIGN-CONFLICT : aucun element de la decision PRODUIT (§2 C-hybride +
  rate-limit) n'est irrealisable. La grille reste MES-sources, la section
  decouverte reste l'ambiant separe, `/nodes` enrichi d'observed, l'ingest
  rate-limite. Seul le CHAMP-SET observed est reduit pour respecter la posture
  anti-amplification deja gelee. Le PO n'a pas a rebattre la decision produit ;
  l'adaptation est une contrainte de surface, pas un pivot.
