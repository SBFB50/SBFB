# Kickoff prompt — Sprint 75 (proposé) : Découverte PULL node-centrique + Ancre VPS « la base »

> À COLLER dans une session Claude Code fraîche (repo + memory, sans l'historique de la conversation qui a produit ce prompt). Ce document **ouvre et crée** le sprint. Il te donne les faits, les contraintes et les questions ouvertes — **pas** une solution pré-mâchée. C'est à la phase recherche/kickoff de trancher les choix d'implémentation.

---

## 0. Avant toute chose — protocole de session fraîche

1. **Lis dans cet ordre, intégralement** :
   - `docs/claude/README.md` (source de vérité du workflow : cycle sprint, audit gate, G8 preflights, gate Codex, discipline commit 9 sections, dual-platform fail-fast, bootstrap §7.1 routing Cas A/B/C/D).
   - `CLAUDE.md` (état projet, décisions gelées, pre-launch wire policy, principe « sessions fraîches », langue).
   - Memory : `MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md` + `feedback_approach.md` + `feedback_wsl_before_push.md` + `feedback_dual_platform.md` + `feedback_codex_gate_strict.md`.
   - **⚠ ATTENTION mémoire périmée** : le tip de `nexus_grid_pivot.md` (et `MEMORY.md`) peut encore afficher HEAD=`bede850`, « 62 ahead », WSL/Docker bloqué, et une instruction « recovery machine (reboot) + re-run dual COMPLET AVANT push ». **C'est PÉRIMÉ.** La vérité terrain = git, pas la mémoire : lance `git log --oneline -3` + `git status -sb` AVANT toute décision push/ahead. Au moment d'écriture de ce prompt : HEAD=`43215f7`, **0 ahead, tout poussé** (`master...origin/master` à jour) ; les deux hotfixes `6ca9702`+`43215f7` ont déjà landé ET été poussés ; l'instruction « 62-ahead / pre-push-recovery » est **OBSOLÈTE**. **Première écriture mémoire de la session = mettre à jour ce tip.**
2. **Suis le workflow SBFB établi sans le ré-inventer** :
   - **Routing** : le main thread est un ROUTEUR. Détecte le Cas (A/B/C/D) via bootstrap §7.1 d'après le contenu de `.planning/active/`, puis invoque l'agent spécialisé (`nexus-audit-gate`, `nexus-sprint-kickoff`, `nexus-phase-preflight-deep`, `nexus-phase-review-deep`). Les agents écrivent leurs artefacts dans `.planning/active/`.
   - **Règle modèle** : ne JAMAIS passer le paramètre `model` aux appels `Agent()`. Toujours `claude-opus-4-8[1m]`, jamais l'alias `opus`.
   - **Cycle** : Phase 0 audit gate de S74 → kickoff.md + plan.md + design_review.md (board G1) → phases A-G (1 commit atomique chacune) → verification.md + audit_plan.md en phase de sortie.
   - **Chaque commit de phase** : G8 preflight (5 scans S1a/S1b/S2/S3/S4, verdict EXECUTE/PLAN-ADAPT/SCOPE-CUT-CONSISTENT/DESIGN-CONFLICT) → code → suites vertes (Rust fmt/clippy/nextest/doctests + web tsc/lint/test:unit/test:coverage/build/size/scan-en-strings, **dual-platform Windows + Docker Linux fail-fast**) → review-deep → **gate Codex GPT-5.5 bloquante** (sortie brute `codex exec -o`, zéro exemption) → réconcilier en PASS → commit body 9 sections → **update memory** (`nexus_grid_pivot.md` + `MEMORY.md`) AVANT de rendre la main.
   - **Discipline** : pas de band-aid (toujours root cause), pas d'emoji, scope cuts stricts, recherche/doc AVANT code, option technique la plus profonde.

### 0bis. Première chose concrète : Phase 0 = audit gate de S74

`.planning/active/sprint75_audit_plan.md` **existe déjà** et liste ce qu'il faut rejouer. **Joue-le d'abord** (session fraîche → `audit_findings.md`, P0/P1 corrigés en commits `fix(sprint74)` AVANT Phase A). Carries connus re-routés vers cet audit : `FRESHNESS-RELEASE-UNINDEXED` (wire), `KEEP-ONLINE-HASH-SOT`, re-crédit invite, tests E.3/H.2/shared-blob/R6, clamp q/offset, `publish_returns_200` flaky, surfaces seed Sybil/over-count, + externes PENDING (P2-A-1 rand, P2-AUDIT-2 iroh, T-NN+2 wasm, P3-OS-1, LT-2 Radicle). **Le thème ci-dessous ne commence qu'après le gate.**

---

## 1. Le PROBLÈME (constaté en vrai, cross-machine Win↔Mac sur LAN)

La découverte d'apps aujourd'hui est **PUSH éphémère** : à la publication, le daemon construit un `ProjectAnnouncement` (avec `project_id = blake3(name)`, un `BlobTicket` portant le hash d'archive + l'adresse du nœud hôte, repo/provenance/is_open_source), l'enveloppe dans un `PowEnvelope` PoW-gated (difficulté 2^18), le broadcaste sur le topic gossip curator, et persiste **les octets exacts de l'enveloppe** dans un outbox SQLite durable.

**Le conflit de conception :**

- Un récepteur **REJETTE** toute annonce dont le PoW `issued_at` est plus vieux que `MAX_PROOF_AGE_SECS = 1800s` (30 min) — `crates/nexus-core-rs/src/pow.rs:105-109` (constante), `:411-427` (check d'âge → `Expired`). Le drop a lieu à `crates/nexus-shell-daemon/src/runtime.rs:1488-1500`.
- Mais le **replay de l'outbox** (NeighborUp / `browse_request` « Rafraîchir » / republish périodique 30-60s jitter / restore au boot) re-broadcaste **les octets stockés VERBATIM avec le PoW d'origine** — jamais re-calculé : `runtime.rs:1502-1551`, `:1610-1626`, `:1876-1894` ; `deploy.rs:624-687`. L'outbox est un `Vec<Vec<u8>>`. Le cache de solve PoW ne vit qu'une `SESSION_WINDOW` de 15 min (`pow_gossip.rs:82-87`, `:206-235` — pas de re-stamp).
- **CONSÉQUENCE** : un pair frais ne peut découvrir que les apps annoncées avec un PoW de moins de 30 min. Toute app publiée plus tôt (ex. il y a ~30 jours) est **invisible** aux nouveaux arrivants.

**Preuve live** : le `/api/daemon/info` du Mac montrait `known_browse_entries:0` ; le log daemon répétait `"PoW verify failed: PoW proof too old (issued ~2.6M s ago, max 1800s) delivered_from=<windows node_id>"`. Le Mac ne voyait aucune des apps du Windows.

**Pourquoi la fenêtre de fraîcheur existe (intention de conception, à ne pas casser bêtement)** : (1) anti-replay (une annonce capturée ne se rejoue pas indéfiniment) ; (2) anti-flood (PoW par annonce fraîche borne le spam, sans modérateur central) ; (3) signal de vivacité (« cette app est offerte par quelqu'un en ligne MAINTENANT »). Le bug : la conception supposait une re-annonce fraîche continue, le code rejoue le tampon périmé.

> **Note importante** : il y a DEUX correctifs orthogonaux. (a) Le bug PUSH immédiat (replay verbatim du PoW périmé) ; (b) le pivot stratégique PULL/annuaire ci-dessous. **Le pivot n'excuse pas de laisser le bug de replay** ; et le fix du bug seul ne livre PAS la découverte node-centrique. Le sprint doit garder les deux distincts et décider lequel/comment.

---

## 2. La DIRECTION DÉCIDÉE PAR LE PO

**Pivoter la découverte de PUSH-éphémère vers PULL node-centrique** + faire du VPS toujours-allumé une **ancre**.

- **Browse devient node-centrique** (modèle mental F-Droid / profil) : Browse montre les **NŒUDS** qui ont publié des apps ; tu ouvres le « profil / catalogue » d'un nœud et tu **télécharges (pull)** les apps que tu veux. Plus d'annonce éphémère à expirer.
- **Joignabilité reste honnête** : tu pulles → si le nœud (ou un seeder détenant le même hash BLAKE3) est en ligne, tu l'obtiens ; sinon c'est offline. **Content-addressing BLAKE3 = la vérité de joignabilité** (invariant projet existant S74 : une annonce forgée peut sur-estimer, elle ne sert jamais d'octets absents).
- **Le VPS de l'utilisateur** (always-on, Hetzner, host SSH `vps` = `135.181.42.188`, déjà validé comme nœud P2P WAN) devient une **ANCRE à deux rôles complémentaires** :
  1. **SEED PERMANENT** — garde des copies des apps de l'utilisateur en ligne même quand PC/Mac sont éteints (extension du mécanisme S74 `keep_online`/seed à un hôte always-on).
  2. **ANNUAIRE / « la base »** — un `node_id` stable et connu où des nœuds + leurs apps publiées sont listés, pour qu'un nouveau nœud bootstrappe en l'interrogeant puis pulle.

UX cible explicite : **Browse = liste de nœuds → catalogue d'un nœud → download**.

> **Garde-fou possessif (lock 3, à graver dès la conception)** : le VPS concret (`135.181.42.188`) est **MON** ancre par défaut — config-distribué, dans **MON** `config.toml` (`default_curators`), jamais une entrée `default_curators` codée en dur et expédiée à tous les binaires. Tout design où ce `node_id` apparaît hard-codé dans une liste livrée au réseau = violation lock 3 = **DESIGN-CONFLICT**. Cf. §4.

---

## 3. Connexion à la conception différée (SearchManifest / node-index)

L'idée « annuaire / la base » du PO mappe presque exactement sur la conception **différée** du SearchManifest (décision **D3 DEFER**, PO-13), capturée dans `.planning/research/s73_searchmanifest_index_node_design.md` (le fichier existe et est le bon pointeur). **Lis ce doc en entier** au kickoff.

Ce doc a déjà conçu la bonne forme ET rejeté la naïve :
- **REJETÉ (§2)** : le « broadcast-everywhere » (chaque nœud pousse son index complet à tous via un op aggregé par tous) — rejeté car surface Sybil/censure/DoS mono-machine, coût d'annonce continu non borné, spam sans coût d'identité.
- **CORRECT (§4)** : un **node-index opt-in, signé Ed25519, DEFAULT OFF** (modèle relay-Nostr / seed-node-Radicle / index-F-Droid-signé) où un nœud normal n'émet rien et **les requêtes utilisateur ne sont JAMAIS envoyées au réseau** ; un client **PULL** un index détaillé seulement depuis un index-node qu'il choisit explicitement. Anti-spam = signature Ed25519 sur JCS canonical sous un **nouveau domaine** `DOMAIN_SEARCH_MANIFEST_V1` + seuil de réputation kudos + curation par signature curator. Le manifeste publie un **DIGEST de couverture** (« je connais ces project_ids jusqu'à seq N »), pas l'index full-text — PULL, jamais PUSH.

**Le kickoff doit explicitement RE-OUVRIR D3** : le PO tire cette conception en avant et la **reframe comme le modèle de découverte PRIMAIRE** (browse-by-node + pull), pas seulement comme couche search full-text. Ne traite PAS « différé » comme une vérité technique gelée (principe « sessions fraîches » de CLAUDE.md).

**Input recherche (ARGUMENT, pas conclusion)** : la recherche apporte un argument FORT pour le substrat **curator-list** (signé/répliqué, `revision` monotone SANS fenêtre PoW, attention-set opt-in, caps DoS déjà construits) plutôt que le SearchManifest full-text — mais c'est un **INPUT à la décision §6 Q1/Q2, pas une conclusion gelée**. Le kickoff doit **comparer explicitement** les trois options (curator-list étendue node-centrique vs `NodeDirectoryEntry` sibling vs SearchManifest tiré en avant) et **justifier** le choix, pas l'hériter de cette note. Quelle que soit l'option, les notes anti-spam §4.3 et l'invariant confidentialité §4.4 du doc restent réutilisables.

---

## 4. GARDE-FOU DUR — anti-recentralisation (NON NÉGOCIABLE)

L'invariant cardinal du projet : **« No central server, no admin »** + les **5 verrous anti-recentralisation** (`.planning/research/s74_disponibilite_ux_design.md:145-150`, repris THREAT_MODEL §15). Le VPS doit être **UNE ancre** (un curator/seed par défaut), **PAS LE serveur**.

Contraintes verbatim qui s'appliquent à toute conception du sprint :
1. **Zéro champ cible/hôte nulle part** (un dropdown « publier sur X » = serveur central de facto).
2. **Redondance additive, jamais substitutive.**
3. **VPS = « Mon serveur » (possessif), jamais défaut universel ni suggestion d'office.** ← contraint DIRECTEMENT comment le VPS-ancre est exposé : il peut être MON curator/seed par défaut, jamais le défaut du réseau. **Concrètement** : `135.181.42.188` peut vivre dans MON `config.toml` (`default_curators`, config-distribué), JAMAIS code en dur dans une liste `default_curators` livrée dans le binaire de tous les utilisateurs. Un `node_id` ancre hard-codé expédié au réseau = violation = DESIGN-CONFLICT.
4. **Provenance/signature toujours celles de l'auteur quel que soit le seeder** (modèle Radicle : seed ≠ autorité).
5. **Suggestion déclenchée par l'état observé, jamais poussée au publish.**

Et (THREAT_MODEL §15) : **seeder != auteur**. Un seeder signe une revendication de seed (sa propre annonce), JAMAIS la provenance de l'app. **Content-addressing BLAKE3 reste la VÉRITÉ de joignabilité** : une annonce forgée ne sert jamais d'octets qu'on ne détient pas.

**Le test que toute conception « annuaire » doit passer** : *le réseau doit survivre à la mort du VPS*. N'importe qui peut faire tourner sa propre ancre (une parmi plusieurs, comme un dépôt F-Droid parmi d'autres, un relay Nostr parmi d'autres), **prouvablement remplaçable**. « La base » = une **liste SIGNÉE, RÉPLIQUABLE, opt-in, bornée** (forme curator-list : Ed25519 + gossip + blobs), **JAMAIS une DB centrale privilégiée**. Frame le VPS comme « mon curator/seed par défaut », jamais « le serveur du réseau ».

Triade anti-Sybil qui doit voyager avec tout annuaire (§4.3) : signature Ed25519 (nouveau domaine) + seuil de réputation kudos pour l'agrégation + curation par signature curator. Sans les trois, le résidu over-count THREAT_MODEL §15 (row D, sévérité M) régresse vers H.

---

## 5. CE QUI EXISTE DÉJÀ (substrat à réutiliser, avec refs)

- **Curator-lists = le primitif « liste signée répliquée » déjà battle-tested** : `CuratorList`/`CuratorListEntry` Ed25519 sur JCS canonical sous `DOMAIN_CURATOR_LIST_V1` (`crates/nexus-core-rs/src/curator.rs:100-342`) ; propagation 2-étages (`CuratorAnnouncement{v,curator,blob_ticket}` sur `curator_topic_id()` + blob signé fetché par ticket) ; **protection rollback par `revision` strictement croissant** (PAS de fenêtre PoW 30 min — c'est la sémantique de fraîcheur que l'annuaire veut) ; attention-set opt-in (drop AVANT fetch des non-souscrits) ; caps DoS (256 entries + caps par champ) ; ingest 9 étapes avec cross-check enveloppe-vs-payload (`crates/nexus-shell-daemon-core/src/iroh_runtime.rs:471-590`). **`default_curators` config → auto-subscribe au boot** (`runtime.rs:403-427`) = LE hook pour livrer le VPS comme « mon curator par défaut ». **Note pre-launch** : `CURATOR_LIST_FORMAT_VERSION` (=1) est librement REDÉFINISSABLE avant le tag v1.0 — étendre le canonical curator-list lui-même (pas seulement la voie raw-op feed) est dans le périmètre éditable, le kickoff n'est PAS contraint au feed-ops-only.
  - **GAP** : aucun code de prod ne **construit/signe/annonce** la propre liste curator d'un nœud. `CuratorListEntry::sign()` n'est appelé qu'en tests + un script VPS périmé (`deploy/create-curator-list.sh`, JSON non signé). La moitié authoring (ex. `POST /api/daemon/curators/publish`) est à construire. Et les entries décrivent des PROJETS, pas des NŒUDS → soit étendre la sémantique, soit définir un type sibling `NodeDirectoryEntry` réutilisant sign/verify/revision/caps avec son **propre** `DOMAIN_*_V1` (jamais réutiliser `DOMAIN_CURATOR_LIST_V1`, miroir de `DOMAIN_SEED_REQUEST_V1`).
- **Seed / keep_online S74** : M18 `keep_online` (table `(project_id, enabled, archive_hash, pinned_at)`, absent=ON par défaut R6) + tag skip-GC `keep-online/<project_id>` ; `fetch_and_pin` ; self-seed-on-deploy ; voie VOLONTAIRE `POST /api/daemon/seed` ; protocole authentifié `sbfb/seed/0` (`SeedRequest` Ed25519+JCS `DOMAIN_SEED_REQUEST_V1`, invite M19 lié à la paire `(project_id, archive_hash)`) ; `reannounce_seeds_at_boot` ; `SeedAnnounced` raw-op feed ; `SeedRegistry` (in-memory, TTL 48h, best-effort). Refs : `crates/nexus-coordinator-rs/src/db.rs:690-882`, `crates/nexus-core-rs/src/blobs.rs:113-220`, `crates/nexus-shell-daemon/src/seed_protocol.rs`, `.../seed_registry.rs`, `.../feed_sync.rs:121-199`, `.../http.rs:1056-1244`. **Extensible quasi-verbatim** pour faire du VPS un seed permanent.
  - **GAPS** : pas de driver de seed headless/config-driven (tout seed est user-initié via HTTP loopback — un VPS n'a pas de session UI) ; pas de re-PIN au boot pour les rows keep_online (seul le re-announce feed existe) ; le client `sbfb/seed/0` (`request_seed`) n'a pas d'appelant prod (`#[allow(dead_code)]` `seed_protocol.rs:298-299`) ; pas de notion « seed le catalogue ENTIER d'un nœud » ; pas de budget disque/GC reaper (post-launch).
- **Pull content-adressé** : `add_bytes` → hash BLAKE3 (`blobs.rs:78-89`) ; `fetch_ticket` seed `MemoryLookup` avec l'`EndpointAddr` du provider puis download (`blobs.rs:170-193`) ; `fetch_and_pin` (`:208-220`) ; acquisition 3-tier dans `GET /blob-serve/{hash}/{path}` (`http.rs:1538-1623`) ; `find_archive_ticket_by_hash` (`browse.rs:585-594`). Flow « pull cette app depuis ce nœud » prouvé E2E (tests `two_nodes_fetch_blob_via_ticket`, `blob_serve_p2p_downloads_archive_from_announcer`).
  - **GAPS** : le ticket porte un `EndpointAddr` **snapshot à l'announce** — un modèle PULL/ancre doit **rafraîchir l'adresse** (re-mint), pas rejouer les octets stockés (même root cause que le bug PoW, dimension adresse) ; `fetch_ticket` télécharge d'UN SEUL `endpoint_id` (pas de fallback multi-seeder alors que le content-addressing le permettrait — il faut plumber les pairs du seed-registry dans le `download()`) ; pas de RPC « interroge le nœud X pour son catalogue » ; les entries gossip directes de nœuds distants **ne sont pas persistées** (seules les apps OWN sont restaurées de l'outbox au boot) → un modèle annuaire doit persister les catalogues distants.
- **Identité node + provenance par nœud** : `BrowseEntry.node_id: Option<String>` existe (l'Ed25519 dialable de l'hôte, set à announce + deploy local) MAIS est `#[serde(skip)]` — délibérément jamais exposé au front (`crates/nexus-shell-daemon-core/src/browse.rs:178-196`). Promouvoir ce champ (additif raw-op-style, 0-bump pre-launch) OU un nouveau `GET /api/daemon/nodes` est le primitif exact d'un Browse node-centrique. `is_own` traverse déjà au front via `BrowseEntryView` (`http.rs:884-918`).
- **Le panneau Disponibilité est DÉJÀ node-centrique** : `web/src/components/AvailabilitySheet.tsx` encode déjà « seeder != auteur » en 4 sections scellées (Auteur immuable, État live probe, Qui le garde en ligne + toggle keep-online + soutien volontaire, Copies de secours + seed-count). UX `Curators.tsx` = pattern « s'abonner à une liste signée Ed25519 » = template exact pour « ajouter une ancre/annuaire ». Route table front = `createBrowserRouter` plat avec `lazy()` (`web/src/App.tsx:39-74`) → `/nodes` et `/node/:nodeId` s'insèrent sans changement structurel.

---

## 6. QUESTIONS DE CONCEPTION OUVERTES (à résoudre au kickoff/recherche — NE PAS pré-décider)

1. **Bootstrap de l'annuaire** : curator-lists existantes (étendues node-centrique) vs nœud-annuaire dédié signé (`NodeDirectoryEntry` + `DOMAIN_*_V1`) vs les deux ? Comment un nouveau nœud apprend QUI est une ancre au-delà du `default_curators` config (statique) — y a-t-il une bootstrap « liste signée des ancres connues », ou config-distribué suffit pour un défaut ?
2. **C'est quoi un « catalogue de nœud »** ? Un endpoint interrogeable (RPC pull) ? Une replica iroh-docs ? Un manifeste signé blob (digest de couverture façon §4 SearchManifest) ? Granularité : 256 nœuds par liste + pull par-nœud, ou liste chunked ? (cap `CURATOR_LIST_MAX_ENTRIES=256`).
3. **Sort de la couche PUSH-gossip** : on la retire ? on la garde comme signal de vivacité court / reconnexion ? curator-lists uniquement ? Le bug de replay verbatim (§1) doit-il être corrigé indépendamment (re-mint adresse + re-stamp PoW sur self-replay) OU rendu caduc par le pivot ? Les 3 intentions de la fenêtre (anti-replay, anti-flood, vivacité) doivent être re-localisées si la découverte quitte le push éphémère.
4. **Modèle opérationnel du VPS** : daemon-as-service headless always-on (systemd ?), seed driver config-driven (liste de `project_id`/catalogue à seeder), re-PIN + re-mint au boot, budget disque. Le VPS n'a pas de session UI — comment l'opérateur déclare ce qu'il seed/liste sans HTTP loopback interactif ?
5. **Intégration disponibilité** : comment le pull node-centrique se branche sur le panneau Disponibilité existant et le seed-registry (best-effort, peut sur-estimer) sans en faire une autorité.
6. **Wire-format** : tout nouvel op feed annuaire = raw-op additif (`serde_json::Value`, **0-bump `FEED_FORMAT_VERSION`** per pre-launch policy ; variante `SearchManifestPublished` déjà nommée non-implémentée dans le doc-comment de l'enum `PublicFeedOperation` `crates/nexus-coordinator-rs/src/public_feed.rs:99`, à côté du set validé `KNOWN_OP_TYPES` `:335-341`). Un nouveau type de liste signée = son propre `DOMAIN_*_V1`. Un-skipper `node_id` ou ajouter `/api/daemon/nodes` doit passer le scan S4 wire-invariant et garder `/browse` JSON forward-compatible. **Concevoir le wire MAINTENANT tant qu'il est librement éditable (avant tag v1.0)** — cela vaut aussi pour `CURATOR_LIST_FORMAT_VERSION`/`SEED_FORMAT_VERSION` (=1, redéfinissables pre-tag), pas seulement la voie raw-op feed.
7. **Migration UX Browse** : node-Browse REMPLACE la grille project-centrique curator-aggregée, ou cohabite (onglet `/nodes`) ? `known_browse_entries` ne reflète que les curator entries (`http.rs:196-214`) — un modèle annuaire a besoin d'un compte honnête de toutes les apps/nœuds découvrables.
8. **Sybil/abus dans un annuaire** : la triade anti-Sybil §4.3 (signature + seuil kudos + curation curator) ; l'invariant « annonce ne substitue jamais une vraie probe/pull » ; le résidu over-count THREAT_MODEL §15 row D ; confidentialité §4.4 (« default OFF, requêtes utilisateur jamais envoyées au réseau ») à réconcilier avec « VPS = ancre par défaut » (le pull annuaire doit être un choix utilisateur explicite, pas un appel réseau silencieux par défaut).
9. **Persistance des catalogues distants** : un modèle annuaire/pull a besoin de **persister les catalogues des AUTRES nœuds**. Aujourd'hui `BrowseAggregator.direct_entries` est **in-memory** et n'est restauré au boot QUE pour les apps OWN du nœud (depuis son propre outbox) — les apps découvertes d'autres nœuds via gossip **ne survivent pas au reboot**. Quelle durabilité : replica iroh-docs, blob signé re-fetché, table SQLite ? C'est load-bearing pour le pivot (sans ça, un nouveau nœud reperd tout son annuaire à chaque redémarrage).
10. **Fallback multi-seeder** : `fetch_ticket` télécharge d'UN SEUL `endpoint_id` alors que le content-addressing BLAKE3 autoriserait un fallback multi-provider. Brancher les pairs du `SeedRegistry` dans `download()` (liste de providers, pas un seul) pour la résilience pull = dans le scope de ce sprint, ou différé ? C'est précisément ce qui transforme « un seeder en ligne » en « N seeders redondants » — le cœur de la promesse de résilience du pivot.

**Risque G8 DESIGN-CONFLICT** : reframer SearchManifest de DEFER à découverte primaire touche une décision Day-0/roadmap → exige un `pivot_proposal.md` + arbitrage utilisateur, pas un PLAN-ADAPT inline. Anticipe ça au kickoff.

---

## 7. ASSETS pour la session

> **NOTE état HEAD — vérifie git, PAS la recherche ni la mémoire** : la mémoire (`nexus_grid_pivot.md`) peut être périmée (cf. §0 : tip `bede850` / 62 ahead OBSOLÈTE). Le snapshot de recherche utilisé pour produire ce prompt est ANTÉRIEUR aux deux hotfixes ci-dessous et ne les mentionne pas. **La seule source fraîche de l'état HEAD = `git log --oneline -3` + `git status -sb`**, pas la recherche.

- **SSH live cross-machine pour test réel** : host `mac` (`192.168.1.53`, macOS arm64, repo `~/nexus-grid`, daemon build & run) + host `vps` (`root@135.181.42.188`, always-on, nœud P2P WAN déjà validé). Utilise-les pour valider le pull Win↔Mac↔VPS en vrai. **Rappel lock 3** : `135.181.42.188` est MON ancre par défaut (mon `config.toml`), jamais une entrée `default_curators` codée en dur dans le binaire de tous.
- **Deux hotfixes déjà landés cette session** (sur `master`, **poussés** — ils POST-DATENT le snapshot de recherche, vérifie-les avec `git log`, pas avec la recherche) : `6ca9702` (self-heal storage/feed namespace sur DB pointer périmé, `open_doc` Err) + `43215f7` (`fix(daemon): gate worker PR_SET_PDEATHSIG to Linux (macOS build portability)`).
- **HEAD = `master` `43215f7`, 0 ahead (tout poussé, `master...origin/master` à jour).** L'instruction mémoire « 62 ahead / recovery-avant-push » est PÉRIMÉE — ne l'applique pas.
- **BLOCKER ENV connu (carry pre-push)** : WSL wedge (`wsl -l -v` hang) → Docker engine 500 + réseau hôte dégradé → tout test montant un nœud iroh (`create_node` → relay/holepunch) timeout ~90s. **NE PLUS faire `wsl --shutdown`** (c'est la cause du 500 ; récupération = reboot machine, hors-portée autonome). Le re-run dual-platform Docker Linux doit se faire **APRÈS recovery, AVANT push** (`feedback_wsl_before_push`). Les tests iroh-networked + le pull cross-machine nécessitent les assets SSH live. (Ce blocker concerne les NOUVEAUX tests réseau du sprint, PAS l'état push actuel qui est propre — voir ci-dessus.)

---

## 8. STATUT ROADMAP — c'est un AMENDEMENT

Ce thème (« découverte PULL node-centrique + ancre VPS ») est une **NOUVELLE direction PO**, PAS encore dans la roadmap. `roadmap_v5_factory_complete_vision.md` liste actuellement S75 = GPU partagé cross-machine, S76 = sharding. La découverte étant **fondamentale** (sans elle, les apps publiées sont invisibles aux nouveaux arrivants — bug live), elle passe **probablement AVANT le GPU** — mais c'est **un appel PO à confirmer au kickoff**. Au kickoff : enregistre le pivot dans la roadmap v5 (ré-ordonnancement S75/décalage GPU), documente la ré-ouverture de D3, et note l'amendement explicitement.

---

## DÉMARRAGE

1. Lis `docs/claude/README.md` + `CLAUDE.md` + les memory files listés au §0 (en notant que le tip mémoire peut être périmé — git fait foi).
2. Détecte le Cas via bootstrap §7.1 d'après le contenu **RÉEL** de `.planning/active/` au moment de l'exécution (ne présume pas). Au moment d'écriture de ce prompt : `sprint75_audit_plan.md` présent, `sprint75_kickoff.md` + `audit_findings.md` absents → **Cas A** → joue d'abord la **Phase 0 audit gate de S74** (invoque `nexus-audit-gate`, écris `audit_findings.md`, corrige P0/P1 en `fix(sprint74)` AVANT toute nouvelle feature). **Si `audit_findings.md` existe déjà**, l'audit gate est entamé/fait — adapte (mid-A ou A-complete → bascule Cas C).
3. Une fois le gate PASS, bascule **Cas C** : invoque `nexus-sprint-kickoff` pour produire `sprint75_kickoff.md` + `sprint75_plan.md` + `sprint75_design_review.md`. Le kickoff DOIT : poser le problème §1 (avec refs), porter la direction PO §2, ré-ouvrir D3 §3 (comparer explicitement les 3 substrats, ne pas hériter), graver le garde-fou §4 comme non-négociable (lock 3 → pas de `node_id` ancre hard-codé livré au réseau), inventorier le substrat §5, **lister les questions ouvertes §6 (1-10) sans pré-décider l'implémentation**, et enregistrer l'amendement roadmap §8. Si le reframe SearchManifest touche une décision Day-0 → `pivot_proposal.md` + arbitrage utilisateur AVANT de coder.
4. Ne code aucune phase A-G avant que kickoff + plan + design_review soient écrits et le board G1 posé.