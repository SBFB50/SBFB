# Roadmap v3 — Confiance + Factory Canari + RRV

## Date de redaction

2026-05-18

## Statut

CANON — fusion validee de `roadmap_v2_public_trust_rrv_factory.md`
et `s65_s75_factory_babel_canary_research.md` (pivot PO). Corrections
finales (ReleasePublished en S65, FG0-FG10 renommage) integrees
apres review GPT 5.5 round 3.

## Documents source fusionnes

| Document | Role |
|----------|------|
| `roadmap_v2_public_trust_rrv_factory.md` | Canon precedent (1667 lignes) |
| `s65_s75_factory_babel_canary_research.md` | Pivot PO Factory-first (851 lignes) |
| `feed_version_bump_strategy.md` | Strategie raw-op adoptee |
| `tantivy_vs_fts5_decision.md` | FTS5 first adoptee |
| `factory_deploy_constraint_research.md` | node_id retrait adopte |
| `licence_anti_capture_research.md` | AGPL + vocabulaire adopte |

---

## Vision PO (mise a jour avec le pivot Factory-first)

Decision PO 2026-05-13 (initiale) :

> 6 sprints (5+1 reserve) pour credibilite publique protocole verifiable.

Amendement PO 2026-05-18 (vocabulaire + positionnement) :

> L'objectif est "release publiquement defendable". Le focus est la
> credibilite, la durabilite et la reutilisabilite. Le projet est un
> commun logiciel anti-capture sous licence AGPL-3.0 (OSI), avec source
> verifiable et provenance verifiable. Le terme "open source" est reserve
> au code SBFB lui-meme (licence AGPL-3.0 OSI). Les apps du reseau sont
> a source verifiable, deployees depuis un depot public, avec provenance
> auto-attestee SLSA L1.

Pivot PO 2026-05-18 (Factory-first + Babel canari) :

> Factory est l'atelier generique. Babel est le premier dogfood app.
> RRV est le moteur de decouverte et preuve qui inspecte ce que Factory
> produit. La Factory ne doit pas etre repoussee en S73-S75 : elle doit
> etre un livrable concret des S67-S68. Babel Reader doit etre la
> premiere app canari generee par Factory, integree au pilote ferme S69.
> RRV vient ensuite pour observer, indexer et expliquer les preuves
> Factory/Babel.

---

## Principes directeurs (13 principes fusionnes)

**P1 — Vocabulaire exact.** Chaque badge, label et texte public exprime
exactement ce que le code prouve. "Provenance disponible" et non
"Verifie". "Provenance auto-attestee SLSA L1" et non "SLSA L1" seul.
"Source verifiable, commun logiciel anti-capture" et non "open source"
pour les apps reseau.

**P2 — SLSA qualifie.** Le projet est a SLSA L1 complet cote
provenance locale (le coordinator clone, build, signe). Il n'est PAS a
SLSA L2 tant que la build platform, l'isolation du builder et la preuve
independante ne sont pas en place. Dire "SLSA presque L2" est interdit.

**P3 — Endorsement public, pas implicite.** Les curator lists SONT
signees Ed25519. Ce qui manque est un endorsement PUBLIC dans le feed
-- type `CuratorVouched` -- visible, rejouable, dissentable. L'inclusion
dans une liste reste un mecanisme de discovery rapide ; l'endorsement
signe dans le feed est la source de verite verifiable.

**P4 — Persistence reelle.** iroh-docs SAIT persister si
`NodeConfig.with_data_dir()` est utilise, mais le daemon ne le cable
pas. iroh-blobs, lui, est bien encore MemStore. La correction de cette
ligne est simple, mais le vrai cout est tests restart, migrations root,
feed republish, validation multi-daemon. C'est un sprint entier (S66),
pas un one-liner.

**P5 — Licence AGPL-3.0.** La licence AGPL-3.0 du projet SBFB est une
licence OSI. Les apps tierces sur le reseau ne correspondent pas
necessairement a la definition OSI. L'ecosysteme utilise le vocabulaire
"source verifiable, provenance verifiable, commun logiciel anti-capture".
Le mot "open source" est reserve aux cas qui respectent effectivement
les criteres OSI.

**P6 — Score de completude de preuve.** Le `confidence_score` dans les
Proof Cards (S71) est un score de completude de preuve, pas un "trust
score" social. Il mesure combien de couches de verification sont
presentes. L'utilisateur decide de la confiance ; le score informe.

**P7 — Factory = module daemon/broker.** La Factory n'est pas une app
iframe. C'est un module Rust dans le daemon avec UI dans le shell React
(/factory). Les operations privilegiees (ecriture disque, git, builds)
passent par le broker cote daemon, pas par le bridge iframe.

**P8 — Babel MVP = reader + storage + fixtures + provenance.** Le modele
NLLB-200 complet ne doit pas bloquer Babel. L'app Babel fonctionne avec
des traductions en fixtures (textes pre-traduits, domaine public). La
traduction live via worker est un stretch goal post-S75.

**P9 — Feed raw-op (serde_json::Value).** Les nouvelles operations feed
(CuratorVouched, SearchManifestPublished) ne bumpent PAS
FEED_FORMAT_VERSION. Le champ `op` dans FeedEntry passe a
`serde_json::Value` (JSON opaque), ce qui rend l'ajout de variants
non-breaking. Les anciens noeuds stockent, verifient (hash + signature),
et propagent les operations inconnues. Pattern SSB/Bitcoin.

**P10 — Topic gossip curator = global.** Le topic gossip des annonces
curator est global (`BLAKE3("nexus-grid/curator-announce/v1")[..32]`),
pas per-curator. Les annonces contiennent la pubkey du curator et le
BlobTicket de la liste. Le CuratorRuntime filtre par attention set.

**P11 — FTS5 d'abord, Tantivy en gate.** Le moteur de recherche S70
est SQLite FTS5 (deja compile dans le binaire via rusqlite bundled).
Tantivy est un gate conditionnel si volume > 50K docs, p95 > 100ms, ou
fuzzy search demande par les utilisateurs. Zero dependance ajoutee
pour S70.

**P12 — node_id retire du manifeste.** Le `node_id` dans SBFB.json est
deprecie puis supprime. L'attribution du deployeur est dans la
provenance signee Ed25519, pas dans le manifeste app. SBFB.json devient
un manifeste applicatif pur et portable. L'artefact hash est
reproductible par n'importe quel deployeur.

**P13 — Factory avancee, canari avant RRV.** La Factory est un livrable
concret en S67-S68, pas un bloc repousse post-RRV. Babel Reader est la
premiere app canari generee par Factory en S69. RRV observe et explique
ce que Factory produit a partir de S70. Cette inversion de sequence
rend le risque Factory visible plus tot.

---

## Gate 0 — Canonisation (cette etape)

Ce document est la source de verite pour les sprints S65-S75. Il
remplace et consolide :

- `.planning/roadmap_v2_public_trust_rrv_factory.md` (v2 canonique)
- `.planning/research/s65_s75_factory_babel_canary_research.md` (pivot)
- Les decisions adoptees des 4 documents de recherche supplementaires

**Criterium d'activation :** Le document est commit dans master et
reference dans CLAUDE.md. Les documents de recherche restent en
`.planning/research/` comme evidence.

**Etat au moment de la canonisation :**

| Metrique | Valeur |
|----------|--------|
| Tests Rust | 1326 |
| Tests Vitest | 265 |
| Tests size-limit | 6/6 |
| Total | 1597 |
| Tag v1.0 | Pose localement, pas pousse |
| Dernier sprint | S64 (hardening public) |
| Carry items | 20 (detail en section dediee) |
| Zones rouges | R-iroh-audit P0, R-wasmtime-cve P0, R-libcrux-hax P2, R-pyodide-escape |

---

## Arc 1 — Fondations Defendables (S65-S66)

L'arc 1 assure que chaque texte public est honnete et que le daemon
survit aux redemarrages. Prerequis absolu pour tout ce qui suit.

---

### S65 — Contrat Public

**Objectif produit :** Aligner chaque texte public (badges, labels,
docs) avec ce que le code garantit reellement, et preparer le terrain
technique pour les operations feed extensibles.

**Valeur :** Pour tout utilisateur ou evaluateur : comprendre exactement
ce que SBFB prouve, ce qu'il ne prouve pas, et ce qu'il promet. Eliminer
les sur-promesses qui disqualifient la credibilite du projet.

**Resultat attendu :** Zero badge ou texte public qui sur-promet par
rapport aux garanties du code. Une taxonomie formelle a 6 niveaux de
confiance. Feed extensible via raw-op (serde_json::Value). Spec et
CLAUDE.md alignes.

#### Phases

**Phase A — Securite feed + taxonomie de confiance + feed raw-op**

Livrable : Document `TRUST_TAXONOMY.md`, fixes securite MANDATORY,
migration feed vers serde_json::Value.

Contenu :
- Fix P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY) : ajouter check
  auth tier dans `feed_insert()` handler (30-50 LOC dans
  `feed_sync.rs`). Un caller de tier T0 ne doit pas pouvoir inserer
  d'operations feed.
- Fix P2-VERIFY-ENTRY-VERSION-GUARD : ajouter
  `if entry.version != FEED_FORMAT_VERSION { return Err(...) }` en tete
  de `verify_entry()` (5 LOC dans `public_feed.rs`).
- Migration `FeedEntry.op` : `PublicFeedOperation` -> `serde_json::Value`.
  Ajouter `try_parse_op()` et `op_type()`. Adapter
  `validate_feed_operation()`, `insert_feed_operation_inner()`,
  `ingest_doc_entry()`, `replay_all()`. ~150 LOC changees.
- Mettre a jour spec `PUBLIC_FEED_SPEC.md` section 9 : "Adding a new
  variant is NOT a breaking change" + nouvelle section 9.1 Forward
  Compatibility.
- Ecrire `TRUST_TAXONOMY.md` : 6 niveaux (Upload direct, Source
  lisible, Provenance auto-attestee SLSA L1, Signature verifiee live,
  Build reproductible futur, Feed verifie hash-chain) + niveaux
  transversaux (AGPL-3.0, Curator vouch, Sandbox).
- Ecrire `COMMONS.md` a la racine (convention anti-capture, ~150 lignes).
- Wiring deploy -> feed : dans `deploy.rs`, apres
  `publish_announcement()`, inserer automatiquement une operation
  `ReleasePublished` dans le feed via `insert_feed_operation()` +
  `publish_feed_entry_to_docs()` (~30 LOC, rollback si insert echoue).
- Fix `deploy.rs` http -> https : rejeter les repo_url en `http://`
  (le feed exige HTTPS). 1 LOC.
- Carry absorbe : P2-COVERAGE-DEPLOY-E2E (test E2E deploy -> feed roundtrip).
- Tests : "unknown op_type can be stored, verified, propagated",
  "canonical bytes identical for Value vs typed struct", auth tier,
  version guard, "deploy -> ReleasePublished auto-inserted",
  "deploy with http:// repo_url rejected",
  "deploy failure -> no feed entry created".

**Phase B — Migration badges UI**

Livrable : tous les badges/labels UI migres vers la nouvelle taxonomie.

Contenu :
- Corriger les textes du Protocol Explorer :
  - "Le code sur le reseau = le code du depot" -> "L'archive reseau est
    construite depuis le depot source par le noeud local. C'est une
    auto-attestation."
  - "Le modele F-Droid/Linux" -> "Inspire par F-Droid -- les apps
    publiques sont deployees depuis leur code source."
  - "Chaine de preuve" -> "Chaine de provenance"
  - "Open source par construction" -> "Source verifiable par construction"
  - Footer : "open source deployee" -> "a source verifiable deployee"
- Browse.tsx : "Verifie" + ShieldCheck -> "Provenance" + FileCheck
- BrowsedProject.tsx : idem + etat dynamique post-verification
- GpuConsentDialog.tsx L2 : "Projets open source verifies" ->
  "Apps deployees depuis un depot public (provenance auto-attestee)"
- Network.tsx : "L2 -- Open source" -> "L2 -- Depot public"
- Curators.tsx : "curator de confiance" -> "curator"
- Corriger PUBLISH_MODEL.md : "open source verifie" -> "Release avec
  provenance auto-attestee"
- Mise a jour des tests existants (BrowsedProject.test.tsx,
  VerificationDetail.test.tsx, Deploy.test.tsx)

**Phase C — Badge dynamique post-verification**

Livrable : le badge "Provenance" dans BrowsedProject passe
dynamiquement a "Signature verifiee" (vert) ou "Echoue" (rouge) apres
verification API automatique a l'ouverture de la page.

Contenu :
- Appel `provenance_verify` automatique a l'ouverture de BrowsedProject
- Etat transitoire : "Verification..." pendant l'appel API
- Cache du resultat pour la session
- Nouveau test Vitest pour l'etat dynamique

**Phase D — Non-regression wording + dette pair + gates Factory**

Livrable : script CI de non-regression sur les textes de confiance.
Definition des gates Factory FG0-FG10. Carry items de process absorbes.

Contenu :
- Script `scan-trust-wording.sh` (grep les termes interdits dans l'UI :
  "verifie" sans qualification, "de confiance" dans un contexte
  automatique, "Le code sur le reseau = le code")
- Documenter les gates Factory FG0-FG10 dans
  `docs/factory/FACTORY_GATES.md` (Classification FG0, Scope FG1,
  Template FG2, Manifest FG3, Diff FG4, Sandbox FG5, Secrets/deps FG6,
  Preview FG7, Provenance FG8, Publish FG9, Review FG10).
- Definir le manifest app v2 spec dans `docs/protocol/SBFB_JSON_V2.md`.
- Definir les artefacts sprint app (ce que Factory doit generer).
- Fix P2-COMMIT-TITLE-FORMAT : hook pre-commit regex (20 LOC)
- Fix P2-REVIEW-ORDER : doc amendment (10 LOC)
- Reclassification P2-PYTHON-BLOCK-EXEMPTION : resolved by pivot S50
- Fix P2-EXPLORER-ESCAPE-SINGLE-QUOTE : 1 LOC escape `'` dans
  `escapeAttr()`
- Suppression des 12 fichiers Playwright zombies
  (P2-PLAYWRIGHT-SPECS-STALE partie 1 : suppression)

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-FEED-INSERT-NO-AUTH-TIER (3/3) | Phase A — MANDATORY |
| P2-VERIFY-ENTRY-VERSION-GUARD (1/3) | Phase A |
| P2-BADGE-WORDING-PREMATURE (pre-S14) | Phase B — coeur du sprint |
| P2-COMMIT-TITLE-FORMAT (2/3) | Phase D |
| P2-REVIEW-ORDER (2/3) | Phase D |
| P2-PYTHON-BLOCK-EXEMPTION (2/3) | Phase D — reclassifie resolved |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE (2/3) | Phase D |
| P2-PLAYWRIGHT-SPECS-STALE (2/3) | Phase D (suppression) + S69 (re-ecriture) |
| P2-COVERAGE-DEPLOY-E2E (2/3) | Phase A — test E2E deploy -> feed |

#### Criteres de validation (gate S65)

- [ ] Zero badge "Verifie" dans l'UI sans verification live prealable
- [ ] TRUST_TAXONOMY.md existe et couvre les 6 niveaux
- [ ] COMMONS.md existe (convention anti-capture documentee)
- [ ] `feed_insert()` rejette les callers de tier T0
- [ ] `verify_entry()` rejette les entries avec version != FEED_FORMAT_VERSION
- [ ] `FeedEntry.op` est `serde_json::Value`, unknown ops stored + verified
- [ ] PUBLIC_FEED_SPEC.md section 9 mise a jour (raw-op forward-compat)
- [ ] Gates Factory FG0-FG10 documentees
- [ ] SBFB.json v2 spec documentee
- [ ] Le Protocol Explorer ne contient plus "le code sur le reseau =
      le code du depot" ni "modele F-Droid/Linux" sans nuance
- [ ] `deploy-from-repo` insere automatiquement un `ReleasePublished` dans le feed
- [ ] `deploy-from-repo` rejette les repo_url en `http://`
- [ ] `scan-trust-wording.sh` passe en CI sans faux positif
- [ ] Tous les tests verts (Rust + Vitest + size-limit)

#### Delta tests estime

- Rust : +15-22 (auth tier, version guard, raw-op migration, wording scan,
  unknown op roundtrip, canonical bytes Value vs typed, deploy->feed wiring,
  http reject, deploy E2E roundtrip)
- Vitest : +5-10 (badge dynamique, migration labels)
- Total sortie estimee : ~1630

---

### S66 — Durabilite

**Objectif produit :** Faire survivre le daemon SBFB aux redemarrages
sans perte de donnees.

**Valeur :** Pour tout operateur de noeud : le daemon ne perd ni ses
apps, ni son feed, ni ses subscriptions au restart. Prerequis absolu
pour le pilote ferme (S69).

**Resultat attendu :** Un daemon qui redemarrage 10 fois consecutives
sans perdre une seule entree feed, une seule archive app, ni une seule
subscription curator.

#### Etat actuel de la persistence (diagnostic)

| Composant | Etat | Impact au restart |
|-----------|------|-------------------|
| Identite Ed25519 | Persistant (keystore) | OK |
| Coordinator DB (13 tables SQLite WAL) | Persistant | OK |
| iroh-docs contenu | EN MEMOIRE (NodeConfig sans data_dir) | Namespaces recrees vides |
| iroh-blobs | EN MEMOIRE (MemStore) | Archives apps perdues |
| Curator lists cache | EN MEMOIRE (DashMap) | Re-broadcast gossip requis |
| Browse entries | EN MEMOIRE (DashMap) | Re-aggregation requise |
| RevocationCache | EN MEMOIRE (HashMap) | Cles revoquees oubliees |
| Feed entries | Persistant (SQLite) | OK localement mais PAS republishees vers iroh-docs |
| Gossip outbox | Persistant (SQLite) | Propres annonces replayed au NeighborUp |

#### Phases

**Phase A — iroh data_dir + iroh-docs persistence**

Livrable : les iroh-docs survivent au restart du daemon.

Contenu :
- `runtime.rs` : ajouter `.with_data_dir(opts.paths.root.join("iroh"))`
  au NodeConfig (~1 ligne + gestion creation dir)
- Adapter la logique de `boot_storage_namespace` et
  `boot_feed_namespace` : les namespaces existent deja dans redb apres
  restart, ne pas les recreer
- Tests : stop -> restart -> verifier que les entries iroh-docs sont
  intactes

**Phase B — iroh-blobs FsStore**

Livrable : les archives apps survivent au restart du daemon.

Contenu :
- `Cargo.toml` workspace : activer feature `fs-store` sur iroh-blobs
- `node.rs` : quand `data_dir` fourni, utiliser
  `FsStore::load(data_dir.join("blobs"))` au lieu de
  `MemStore::default()`
- `blobs.rs` : adapter `BlobsClient` pour fonctionner avec `FsStore`
- Le type `Node` doit accepter le nouveau store (generique ou enum)
- Tests : deux noeuds, A ajoute un blob, reboot, B fetch via ticket

Risques : refactoring profond du crate fondation. Tous les crates
downstream compilent contre `Node` -- le changement de type du store
est breaking pour la signature des types.

**Phase C — Feed republish au boot + feed_join handle**

Livrable : le feed P2P fonctionne apres restart.

Contenu :
- Au boot dans `runtime.rs`, apres `boot_feed_namespace`, iterer les
  entries en SQLite (`replay_all`) et les ecrire dans le namespace
  iroh-docs via `publish_feed_entry_to_docs`
- Stocker le JoinHandle de `feed_join` dans DaemonHttpState, joindre
  au shutdown
- Tests : inserer 5 entries feed, restart daemon, verifier que les
  entries sont presentes dans iroh-docs ET accessibles via l'API

Carry resolus : P2-ORPHAN-REPUBLISH-RECOVERY (3/3),
P2-FEED-JOIN-HANDLE-LEAK (3/3).

**Phase D — RevocationCache persistence + SQLite synchronous**

Livrable : securite et durabilite des caches critiques.

Contenu :
- Migration M14 : table `key_rotations` (old_pubkey, new_pubkey,
  timestamp, transition_days, signature)
- Au boot, charger le RevocationCache depuis SQLite
- Ajouter `synchronous = FULL` au `CoordinatorDb::open()` pour WAL
  crash-safety renforcee
- Tests : rotation de cle, restart, verifier que la revocation est
  toujours active

**Phase E — Test E2E restart complet**

Livrable : preuve que le daemon survit a un cycle
stop -> start -> verify.

Tests :
1. Daemon boot -> publish app via deploy -> insert feed entry ->
   subscribe curator -> stop propre
2. Daemon restart -> verifier : app accessible (blob persiste), feed
   entries presentes (SQLite + iroh-docs), curator subscription active
   (subscriptions.json), meme node_id
3. Deux daemons : A publie, B sync, A restart -> B a toujours les
   donnees, A retrouve les siennes
4. Crash simule (drop sans shutdown) -> restart -> tout fonctionne

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-FEED-JOIN-HANDLE-LEAK (1/3) | Phase C |
| P2-ORPHAN-REPUBLISH-RECOVERY (1/3) | Phase C |

#### Criteres de validation (gate S66)

- [ ] 10 restarts consecutifs sans perte feed/blob/storage
- [ ] Archives apps accessibles apres restart (FsStore operationnel)
- [ ] Feed entries republishees vers iroh-docs au boot
- [ ] RevocationCache chargee depuis SQLite au boot
- [ ] JoinHandle de feed_join track et joined au shutdown
- [ ] Test E2E multi-daemon restart passe
- [ ] Aucune regression sur les 1625+ tests existants

#### Delta tests estime

- Rust : +15-25 (persistence, crash recovery, restart, multi-daemon)
- Vitest : 0 (pas de changement frontend)
- Total sortie estimee : ~1650

---

## Arc 2 — Factory + Canari (S67-S69)

L'arc 2 construit la Factory comme atelier generique de creation d'apps,
valide le tout avec Babel Reader comme premiere app canari, et confronte
le systeme a des testeurs reels via un pilote ferme.

**Decision architecturale :** Factory est un module daemon/broker Rust,
pas une app iframe. L'UI Factory est une page React du shell (`/factory`),
les operations privilegiees (ecriture disque, git, builds) passent par le
broker cote daemon via routes HTTP authentifiees (`/api/v1/factory/*`).
Le bridge iframe reste reserve aux apps sandboxees.

---

### S67 — Factory Foundation + Feed Extensible

**Objectif produit :** Factory sait creer un repo app minimal avec un
process de sprint comme SBFB, et le feed accepte de nouvelles operations
sans breaking change.

**Valeur :** Pour tout developpeur : `sbfb create --template static-storage
--name my-app` produit un projet pret a deployer. Pour le protocole : les
CuratorVouched sont inserables dans le feed grace au raw-op de S65.

**Resultat attendu :** 3 templates fonctionnels, CLI `sbfb create`,
SBFB.json v2 operationnel, node_id retire du deploy path, CuratorVouched
et CuratorDisendorsed dans le feed.

#### Phases

**Phase A — SBFB.json v2 + retrait node_id + CuratorVouched feed**

Livrable : manifest enrichi avec retro-compatibilite, node_id deprecie,
endorsement signe dans le feed.

Contenu :
- Implementer SBFB.json v2 dans deploy.rs : struct `SbfbJson` enrichi
  (schema_version, name, display_name, description, category, license,
  lang, bridge.methods, bridge.events, tech.type, tech.build_command,
  requirements).
- Rendre `node_id` optionnel/deprecie : `Option<String>` avec
  `#[serde(default)]`, supprimer le bloc de verification stricte
  (lignes 119-128 deploy.rs), logger un warning si present.
- Parser/validateur dans deploy.rs (support v1 + v2).
- Ajouter `CuratorVouched(CuratorVouchedPayload)` et
  `CuratorDisendorsed(CuratorDisendorsedPayload)` au enum
  `PublicFeedOperation`.
- `CuratorVouchedPayload` : project_id, curator_pubkey, scope (String),
  comment (Option<String>, max 280 chars).
- `CuratorDisendorsedPayload` : project_id, curator_pubkey, reason
  (String), comment (Option<String>).
- Domain de validation pour les deux types.
- FEED_FORMAT_VERSION reste a 1 (raw-op, pas de bump).
- Tests : v1 compat, v2 parse, v2 reject invalid, node_id warning,
  CuratorVouched insert/replay/verify_chain, serde roundtrip,
  adversarial forgery.

**Phase B — Template engine + 3 templates**

Livrable : generateur Rust + templates fonctionnels.

Contenu :
- Generateur Rust natif dans nexus-shell-daemon-core : substitution
  variables, copie bridge SDK, init repo git.
- Template `static-minimal` (HTML pur + bridge, ~100 LOC)
- Template `static-storage` (storage CRUD pattern Ideas Hub, ~300 LOC)
- Template `react-vite` (React 19 + Vite + hook useSBFBBridge, ~400 LOC)
  **STRETCH** — reportable en S68/S73 si S67 deborde
- `template.json` pour chaque template (id, version, variables,
  post_create hooks, content_hash BLAKE3)
- Migration des deux apps existantes (Explorer, Ideas Hub) vers v2
  (retrait node_id, ajout schema_version, bridge, tech, requirements).
- Tests : generation snapshot, fichiers attendus, SBFB.json valide.

**Phase C — CLI `sbfb create` + factory.template.lock**

Livrable : sous-commande CLI fonctionnelle.

Contenu :
- Sous-commande `sbfb create --template <id> --name <name>` dans
  nexus-shell-daemon.
- Mode interactif (prompts) et non-interactif (flags).
- Verification content_hash BLAKE3 du template.
- `factory.template.lock` genere dans chaque projet cree (hash template,
  version, date).
- `factory.provenance.json` (lineage creation).
- `factory.audit.jsonl` (log de chaque operation Factory).
- Sprint skeleton genere (.planning/active/ avec kickoff, plan templates).
- Tests : CLI happy path, invalid template, path traversal.

**Phase D — Multi-curator trust overlay + stale detection**
**STRETCH** — reportable en S68 Phase C ou S73 si S67 deborde. Gate
obligatoire S67 = Phases A-C seulement.

Livrable : agregation multi-curator avec scope dans le Browse. Detection
automatique des sources perimees.

Contenu :
- Ajouter `scope: Option<String>` a `CuratorList` avec
  `#[serde(default)]`.
- Propager le scope dans `BrowseEntry` :
  `endorsement_scopes: Vec<String>`.
- Dans `BrowseAggregator.aggregate()`, agreger les endorsements de tous
  les curators pour chaque project_id.
- Timer coordinator qui re-verifie periodiquement les repos source des
  apps deployees.
- Emission automatique de `SourceBecameStale` quand un repo diverge ou
  est unreachable.
- Tests adversariaux : curator malveillant, split-brain curators, stale
  replay, forgery disendorsement, flood disendorsement (rate limited).

#### Carry items absorbes

Aucun carry item directement, mais :
- Le retrait node_id resout le probleme PLACEHOLDER des exemples.
- CuratorVouched prepare le terrain pour S71 Proof Cards.
- Les gates Factory FG0-FG3 sont implantees dans ce sprint.

#### Criteres de validation (gate S67)

- [ ] `sbfb create --template static-storage --name test-app` genere un
      projet deployable
- [ ] Les 3 templates generent des projets qui passent le deploy verifie
- [ ] SBFB.json v2 retro-compatible avec v1
- [ ] node_id absent du manifeste = deploy accepte
- [ ] content_hash BLAKE3 verifie avant generation
- [ ] `CuratorVouched` et `CuratorDisendorsed` inserables et
      rejouables dans le feed
- [ ] `verify_chain()` passe sur un feed contenant les 4 types
      d'operations (Release, Stale, Vouched, Disendorsed)
- [ ] Explorer/Ideas Hub restent compatibles (migration v2)
- [ ] Aucune ecriture hors workspace autorise

#### Delta tests estime

- Rust : +18-28 (templates, CLI, validation, feed ops, adversarial,
  node_id deprecation)
- Vitest : +8-12 (composants UI si applicables)
- Total sortie estimee : ~1690

---

### S68 — Broker / Preview / Publish Gate + Gouvernance Minimale

**Objectif produit :** Factory ne se contente plus de scaffold ; elle
sait prevenir une publication fragile. La gouvernance est visible dans
l'UI.

**Valeur :** Pour tout developpeur : voir un diff avant d'appliquer, un
preview avant de publier. Pour tout utilisateur : voir qui endorse quoi,
quand, avec quel desaccord.

**Resultat attendu :** Page React `/factory` avec template selector,
diff viewer, preview iframe, publish gate checklist. UX confiance visible
dans Browse et Curators (endorsements, freshness, dissent).

#### Phases

**Phase A — Broker architecture + routes API**

Livrable : module factory_broker avec routes HTTP.

Contenu :
- Module `factory_broker` dans nexus-shell-daemon-core.
- Routes HTTP : `/api/v1/factory/templates` (list),
  `/api/v1/factory/create` (generate), `/api/v1/factory/diff` (preview
  changes), `/api/v1/factory/apply` (apply changes),
  `/api/v1/factory/preview` (serve preview).
- Path allowlist + canonicalize (meme rigueur que `validate_zip_path()`
  dans blob_serve.rs).
- Audit log `factory.audit.jsonl`.
- Tests : path traversal denied, routes auth required, symlink refused.

**Phase B — Diff generation + review API + publish gate**

Livrable : le broker calcule les diffs et verifie avant publication.

Contenu :
- Diff engine : fichiers modifies/ajoutes/supprimes entre workspace
  actuel et modifications proposees.
- Format diff : JSON structure (pas unified diff text) pour affichage
  React.
- Route `/api/v1/factory/apply` applique seulement si diff
  precedemment genere + user_confirmed=true.
- Publish gate checklist via `/api/v1/factory/publish-check` :
  index.html existe, SBFB.json v2 valide, bridge methods declarees
  existent, no secrets detected (regex scan), build OK (si
  build_command present), repo HTTPS pour le feed.
- Integration deploy-from-repo : Factory peut declencher
  `POST /api/v1/deploy-from-repo` apres publish-check.
- Insertion `ReleasePublished` automatique dans le feed si le chemin
  feed est cable.
- Tests : diff calcul correct, apply sans diff = refuse, concurrent
  apply protection, publish gate pass/fail.

**Phase C — UX confiance visible (badges, timeline, dissent) + Factory UI**

Livrable : la page Browse et Curators montrent la gouvernance. La page
/factory est fonctionnelle.

Contenu :
- Page Curators : freshness ("derniere mise a jour il y a X"), scope
  du curator, badge "inactif" si > 90 jours.
- Page Browse : nombre de curators, breakdown par scope, indicateur de
  dissent, freshness de la derniere verification.
- Page BrowsedProject : timeline des endorsements/disendorsements avec
  commentaires et dates.
- Page `/factory` avec : template selector, variables form, diff
  viewer (fichiers expandables), approve/reject buttons.
- Composant DiffViewer reutilisable.
- Tests Vitest : DiffViewer renders, approve mutation.

**Phase D — Preview sandbox + Factory proof pack minimal**

Livrable : preview et evidence minimale avant publication.

Contenu :
- Preview : le broker zippe le workspace, le sert via blob-serve,
  affiche dans iframe (meme chemin que deploy normal).
- `factory.provenance.json` capture dans chaque app generee :
  generator version, template hash, variables hash, source commit.
- Evidence pack minimal : factory.provenance.json + factory.audit.jsonl
  + factory.template.lock — pas le proof pack complet (reporte en S75).
- Tests : preview serve, missing index.html rejected, evidence pack
  structure.

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-PROVENANCE-404-BRIDGE (2/3) | Phase B — distinguer "projet inexistant" vs "pas de provenance" |

#### Criteres de validation (gate S68)

- [ ] Projet cree via /factory deploie correctement
- [ ] Diff viewer affiche les modifications avant application
- [ ] Preview iframe fonctionne (meme sandbox que deploy normal)
- [ ] Publish gate rejette un projet sans index.html
- [ ] Publish gate rejette manifest invalide, methode bridge inconnue,
      secret, path traversal, symlink, repo non HTTPS
- [ ] Path traversal impossible (tests canonicalize)
- [ ] Audit log JSONL contient toutes les actions
- [ ] Endorsements visibles dans Browse/Curators
- [ ] Dissent visible quand deux curators sont en desaccord
- [ ] Freshness des endorsements affichee dans l'UI
- [ ] Deploy roundtrip produit archive + provenance + Browse + feed entry

#### Delta tests estime

- Rust : +18-25 (broker, diff, preview, publish gate, path traversal,
  deploy roundtrip)
- Vitest : +10-15 (DiffViewer, Factory page, endorsement UI, timeline)
- Total sortie estimee : ~1733

---

### S69 — Babel Reader Canari + Pilote Ferme

**Objectif produit :** Babel est la premiere app reelle produite par
Factory, testee par 2-3 personnes, mais sans promesse publique large.
Valider SBFB avec des testeurs sur des machines qui ne sont pas celle
du developpeur.

**Valeur :** Pour le projet : premiere confrontation avec la realite
(installation, connexion P2P, stabilite 24h, app reelle via Factory).
Pour les testeurs : contribuer a un commun logiciel avant sa release.

**Resultat attendu :** Babel Reader deploye via Factory, avec fixtures
multilingues, storage P2P, provenance SLSA L1, visible dans le Browse.
2 testeurs installent, se connectent, synchronisent le feed, et le daemon
tourne 24h sans crash. Decision go/no-go documentee pour l'arc 3.

#### Modele du pilote

- **Ferme** (2-3 personnes, pas public). R-iroh-audit P0 rend un
  pilote public irresponsable.
- **Zero telemetrie**. Feedback via Ideas Hub (dogfooding).
- **Coherence philosophique**. Distribution par invite (tickets feed +
  installeurs).

#### Phases

**Phase A — Domain pack Babel + app babel-reader via Factory**

Livrable : Babel creee par Factory et deployable.

Contenu :
- Spec format domain pack (template.json extended, fixtures/, config/).
- Babel domain pack : textes fixtures (3 textes domaine public, ~5
  langues), languages.json, source_manifests/ (origin, hash, droits,
  licence par texte).
- Creer babel-reader via `sbfb create --domain-pack babel`.
- UI reader : liste textes, lecteur plein ecran, toggle langue.
- Storage : progression lecture (bridge storage_get/set), bookmarks.
- Identity : affichage pubkey lecteur (bridge identity_pubkey).
- Provenance app visible, feed cursor visible.
- Tests Factory acceptance (de la liste des 22) :
  - app generee contient `index.html`, `SBFB.json`, SDK bridge
  - app generee contient planning sprint
  - app generee contient `factory.template.lock`
  - app generee contient `factory.provenance.json`
  - Babel affiche textes fixtures
  - Babel lit/ecrit progression storage
  - Babel appelle `identity_pubkey`
  - Babel affiche provenance
  - Babel lit feed cursor
  - test negatif: aucune methode `babel_*`, `factory_*`, `shell_*`

**Phase B — Deploy verifie + feed publication + installeur teste**

Livrable : Babel est une app pleinement verifiable. Installeurs testes.

Contenu :
- Deploy babel-reader via deploy-from-repo.
- Provenance SLSA L1 auto-attestee generee et verifiable.
- Feed entry ReleasePublished creee automatiquement.
- Verification E2E : deploy -> browse -> ouvrir -> lire.
- Tests Factory acceptance :
  - `deploy-from-repo` accepte app sans `node_id`
  - `deploy-from-repo` refuse manifest invalide
  - `deploy-from-repo` refuse methode bridge inconnue
  - `deploy-from-repo` refuse repo non HTTPS pour feed
  - Babel deploy -> Browse -> open -> provenance verify
  - feed `ReleasePublished` cree
- Tester NSIS sur VM Windows 11 fresh.
- Tester .deb sur Ubuntu 24.04 LTS VM.
- Fix les bugs d'installation trouves.

**Phase C — Mecanisme invite + feedback collector**

Livrable : tout est pret pour inviter des testeurs.

Contenu :
- Implementer endpoint HTTP pour distribuer un ticket de feed :
  `POST /api/v1/pilot/invite` -> genere invite token + feed ticket.
- Mettre a jour OnboardingEmpty.tsx (enlever commandes Python obsoletes,
  ajouter "Entrez votre ticket d'invitation").
- Fix P2-VERIFY-LOCAL-KEY-ONLY : ajouter resolver pkarr dans le path
  de verification cross-node (50-80 LOC).
- Deployer Ideas Hub comme "Pilot Feedback" sur le reseau.
- Bouton "Exporter les logs" dans le tray menu (zip 7 jours).
- Guide testeur.
- Tests Factory acceptance :
  - path traversal refuse
  - symlink refuse
  - secret fixture refuse
  - preview iframe smoke test

**Phase D — Scenarios de test guides + re-ecriture Playwright**

Livrable : chaque testeur a un parcours structure.

Contenu :
- 8 scenarios de test (installation, join, browse, deploy, feed sync,
  verification provenance, restart, stabilite 24h).
- Formulaire de resultat par scenario.
- Re-ecriture des specs Playwright pour les pages actuelles (partie 2
  de P2-PLAYWRIGHT-SPECS-STALE).
- Tests Factory acceptance restants :
  - FTS5 p95 non requis ici (S70)
  - RRV trouve Babel (S70+, pas gate S69)

**Phase E — Analyse go/no-go**

Livrable : decision honnete documentee.

Contenu :
- Collecter tous les retours (Ideas Hub + emails).
- Categoriser : bugs critiques / UX / cosmetics / suggestions.
- Document "Bilan pilote".
- Decision : go / go-with-fixes / no-go pour l'arc 3.
- Evaluer les tests RRV post-S70 :
  - RRV trouve Babel localement (deferred)
  - Proof Card affiche source, artifact, provenance, feed (deferred)

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-VERIFY-LOCAL-KEY-ONLY (2/3) | Phase C |
| P2-PLAYWRIGHT-SPECS-STALE (2/3) | Phase D (re-ecriture) |

#### Criteres de validation (gate S69 = Gate 1)

| Critere | Go | No-Go |
|---------|-----|--------|
| Installation | 2/3 testeurs installent sans aide | 0/3 reussit ou 2/3 ont besoin d'aide |
| Premier lancement | Daemon demarre + browser ouvre en < 30s | Crash au demarrage |
| Connexion P2P | 2 noeuds se voient en < 5 min | Aucune connexion apres 15 min |
| Deploy app | 1 testeur deploie depuis source | Deploy echoue ou provenance invalide |
| Babel via Factory | Babel genere par Factory, deploy, visible Browse | Factory echoue ou Babel pas deployable |
| Feed sync | Feed synchronise entre 2+ noeuds | Divergence ou corruption |
| Restart | Daemon redemarrage propre | State corrompu ou impossibilite |
| Stabilite 24h | Daemon tourne 24h sans crash | Crash, OOM, ou freeze |

Si > 5 bugs P0/P1 : sprint fix dedie avant S70. Le sprint reserve
est utilise ici.

#### Delta tests estime

- Rust : +8-15 (domain pack, deploy roundtrip, invite, fixes pilote)
- Vitest : +10-15 (re-ecriture Playwright, Babel tests, onboarding)
- Babel tests dans son repo : +12-18 (acceptance tests section 12)
- Total sortie estimee : ~1773

---

## Gate 1 — Go/no-go Arc 3

**Conditions :**
- Le pilote ferme est operationnel (2+ testeurs actifs)
- Babel deploye via Factory et visible dans Browse
- Les bugs critiques decouverts sont fixes
- Le daemon survit a 24h sans intervention
- Les 22 tests d'acceptance Babel (section 12 canary doc) sont couverts
  (20/22 en S69, 2 deferred a S70+)

**Decision :** PO evalue le feedback pilote. Si les criteres gate S69
sont remplis, l'arc 3 demarre. Sinon :
- Si < 3 bugs P0 : S70 absorbe les fixes
- Si >= 3 bugs P0 : sprint fix dedie, re-pilote

**Decision iroh 0.98 vs 1.0 :** Evaluee ici. Si le pilote revele des
bugs iroh-docs/iroh-blobs fixes uniquement en iroh 1.0 : l'upgrade
devient prioritaire (effort ~2-3 phases). Si le pilote tourne bien
sur 0.98 : rester sur 0.98 pour l'arc 3.

---

## Arc 3 — Intelligence Verifiable (S70-S72)

L'arc 3 construit la capacite de chercher dans le reseau et de prouver
la qualite de chaque resultat. La recherche reste locale par defaut
(privacy by design). Les manifests sont opt-in. RRV observe et explique
ce que Factory et Babel ont produit.

---

### S70 — RRV LocalOnly (FTS5)

**Objectif produit :** Un moteur de recherche local qui indexe les apps,
le feed et la provenance du noeud. Babel trouvable.

**Valeur :** Pour tout utilisateur : trouver une app par mot-cle au lieu
de parcourir un listing exhaustif. Chaque resultat est cite (source,
hash, timestamp).

**Resultat attendu :** `GET /api/daemon/search?q=traduction` retourne
des resultats pertinents avec citations exactes, en < 50ms.

#### Choix technique : FTS5, pas Tantivy

**Moteur :** SQLite FTS5, deja compile dans le binaire via
`rusqlite { features = ["bundled"] }`. Zero dependance ajoutee.

**Rationale :** BM25 + phrase queries + boolean queries + prefix search +
highlight() + snippet(). Dataset < 500 documents pre-launch. Jointures
SQL natives avec provenance/feed/curators. Zero impact build, binaire, CI.

**Gate Tantivy :** Si volume > 50K docs, p95 > 100ms, ou fuzzy search
demande par 3+ utilisateurs, migration vers Tantivy via trait
`SearchEngine`.

#### Phases

**Phase A — Index local FTS5 + API**

Livrable : un index FTS5 operationnel avec endpoint de recherche.

Contenu :
- Migration M15 : `CREATE VIRTUAL TABLE search_index USING fts5(...)`
  dans coordinator.db.
- `search.rs` dans nexus-coordinator-rs : schema FTS5 (project_id,
  project_name, description, category, keywords, repo_url, source_type),
  creation, indexation, recherche.
- `search_api.rs` dans nexus-shell-daemon :
  `GET /api/daemon/search?q=...&limit=...&offset=...`
- Labels separes dans les resultats : `Local generated`, `Local tested`,
  `SBFB verified`, `SBFB stale`.
- Wire dans `http.rs` router.
- Tests : index creation, search, empty results, special chars, BM25
  ranking.

**Phase B — Indexation au boot + incrementale + SBFB.json enrichi**

Livrable : l'index est peuple automatiquement.

Contenu :
- Au boot : indexer browse entries, feed entries, provenance records.
- Trigger incrementale : re-indexer a chaque ProjectAnnouncement,
  deploy reussi, ou FeedEntry insere.
- Indexer les apps Factory (metadata SBFB.json v2).
- Indexer le contenu des README.md dans les archives zip.
- Tests : rebuild d'index, indexation incrementale, Babel trouvable.

**Phase C — Bridge method + citations**

Livrable : les apps iframe peuvent chercher via le bridge.

Contenu :
- Ajouter `search` au BridgeMethodSchema dans `protocol.ts`.
- Handler dans `useBridge.ts`.
- Chaque resultat retourne des citations exactes (source_type,
  entry_hash, file_path, line).
- Tests Vitest : bridge search dispatch, citation format.

**Phase D — App sbfb-search MVP**

Livrable : une app de recherche standalone dans `examples/sbfb-search/`.

Contenu :
- HTML + JS vanilla (meme pattern que Explorer/Ideas Hub).
- Search bar + resultats avec extraits et citations.
- Design dark theme coherent.
- SBFB.json manifest v2.
- Tests : taille bundle, fonctionnalite search.

#### Criteres de validation (gate S70)

- [ ] `search?q=babel` retourne l'app Babel Reader
- [ ] `search?q=explorer` retourne l'app Protocol Explorer
- [ ] Temps de reponse < 50ms pour < 1000 documents
- [ ] Index reconstruit au boot en < 5s
- [ ] Citations exactes (source_type, entry_hash) dans chaque resultat
- [ ] FTS5 index >= 100 entrees
- [ ] Babel trouvable localement (test d'acceptance #21)

#### Delta tests estime

- Rust : +12-18 (index, search, incremental, citations)
- Vitest : +8-12 (bridge search, app sbfb-search)
- Total sortie estimee : ~1803

---

### S71 — Proof Cards

**Objectif produit :** Enrichir chaque resultat de recherche avec un
"passeport de confiance" deterministe.

**Valeur :** Pour tout utilisateur : comprendre en un coup d'oeil
pourquoi un resultat est fiable (ou non). Le score est un score de
completude de preuve, pas un "trust score" social.

**Resultat attendu :** Chaque resultat de recherche affiche une Proof
Card avec score (0-100), facteurs de risque, fraicheur, provenance,
curators, et licence.

#### Calcul du score de completude de preuve

Le score est un entier 0-100 calcule deterministe, transparent et
reproductible :

```
Base: 30 points (le resultat existe)

+ 20 si provenance.verified == true
+ 10 si is_open_source == true
+ 10 si freshness.state == "fresh" (< 7j)
+  5 si freshness.state == "aging" (7-30j)
+ 10 si curation.curator_count >= 1
+ 10 si curation.curator_count >= 3
+  5 si license.spdx != null
+  5 si hash.archive_hash != null

- 10 si risk contient "stale_source"
- 15 si risk contient "no_provenance"
- 10 si risk contient "unverified_deploy"
-  5 si risk contient "old_release"

Clamp: min(0, max(100, score))
```

#### Phases

**Phase A — ProofCard data model + computation**

Livrable : `ProofCard` struct + calcul deterministe.

Contenu :
- `proof_card.rs` dans nexus-coordinator-rs : struct ProofCard (source,
  hash, license, freshness, provenance, risk, curation, confidence).
- `compute_proof_card(project_id, db, browse_entry)` -> ProofCard.
- Formule de score documentee dans le code et dans un doc.
- Proof Card Babel comme premier test case.
- Tests : score computation, risk detection, edge cases, determinism,
  un projet sans provenance ne peut pas afficher score > 50.

**Phase B — API + bridge**

Livrable : les proof cards accessibles via API et bridge.

Contenu :
- `GET /api/daemon/proof-card/{project_id}`
- Bridge method `proof_card_get(project_id)` -> ProofCard.
- Test d'acceptance #22 : Proof Card affiche source, artifact,
  provenance, feed.
- Test d'acceptance #23 : Proof Card indique stale si source cassable.
- Tests : API response format, bridge dispatch.

**Phase C — Integration dans search results + Browse**

Livrable : l'app sbfb-search et le Browse affichent les proof cards.

Contenu :
- Composant ProofCard (HTML/JS) : score bar, facteurs, actions.
- "Verifier la provenance" interactif via bridge.
- Integration dans les resultats de recherche et dans Browse.
- Warning si Factory generation locale ne devient pas confiance reseau.
- Tests Vitest : ProofCard renders, integration search.

**Phase D — Tests adversariaux**

Livrable : le score ne ment pas.

Contenu :
- Proof card spoofing : un projet sans provenance ne peut pas afficher
  score > 50.
- Risk factor injection : HTML dans description sanitized.
- Stale detection : SourceBecameStale -> risk "stale_source".
- Score determinism : memes entrees -> meme score (pas de randomness).
- Test d'acceptance #24 : score de completude deterministe.

#### Criteres de validation (gate S71)

- [ ] ProofCard generable pour tout projet du Browse
- [ ] Proof Card Babel avec provenance, curators, feed, licence
- [ ] Score deterministe (memes inputs = meme score)
- [ ] Un projet sans provenance a un score <= 50
- [ ] Risk factors affiches dans l'UI
- [ ] Tests adversariaux verts

#### Delta tests estime

- Rust : +5-8 (proof card computation, adversarial)
- Vitest : +5-8 (composant ProofCard, integration search)
- Total sortie estimee : ~1821

---

### S72 — SearchManifest Opt-In

**Objectif produit :** Permettre aux noeuds de publier volontairement
un index signe, enrichissant la decouverte P2P.

**Valeur :** Pour le reseau : decouverte d'apps au-dela du voisinage
gossip immediat, dont Babel et les apps Factory.

**Resultat attendu :** Un noeud peut publier un `SearchManifest` signe
opt-in. Les autres noeuds le recoivent, le verifient, et enrichissent
leurs resultats de recherche.

#### Decisions architecturales

- **Opt-in explicite.** Le daemon ne publie PAS de manifest par defaut.
- **Privacy by design.** Les requetes de recherche ne sont jamais
  envoyees au reseau.
- **SearchManifestPublished dans le feed.** Nouvelle operation feed
  (utilise le raw-op de S65, pas de bump FEED_FORMAT_VERSION).
- **Gossip topic dedie.** `BLAKE3("nexus-grid/search-manifest/v1")[..32]`
  pour la publication en temps reel.

#### Phases

**Phase A — SearchManifest format + signing**

Livrable : wire format defini, signable et verifiable.

Contenu :
- Domain constant `DOMAIN_SEARCH_MANIFEST_V1` dans `canonical.rs`.
- `SearchManifest` struct : v, node_id, created_at, projects (max 256),
  feed_cursor, index_stats, signature.
- Limits par champ : description <= 280 bytes, keywords <= 10 x 64
  bytes, total manifest <= 1 MB.
- Sign/verify via canonical_bytes pattern existant.
- Ajouter `SearchManifestPublished(SearchManifestPublishedPayload)` au
  enum `PublicFeedOperation` (raw-op, pas de bump).
- Tests : sign, verify, reject tampered, reject oversized.

**Phase B — Publication opt-in via iroh**

Livrable : un noeud peut publier son manifest sur le reseau.

Contenu :
- Gossip topic search-manifest.
- `POST /api/daemon/search/publish-manifest`
- Stockage blob + annonce gossip.
- Rate limiter : 1 publication par heure par noeud.
- Tests : publish, gossip announce, rate limit.

**Phase C — Discovery + verification**

Livrable : les manifests des peers sont recus, verifies et exploites.

Contenu :
- Subscribe au gossip topic search-manifest.
- Recevoir, parser, verifier signatures des manifests peers.
- Cache DashMap similaire a CuratorRuntime.
- `GET /api/daemon/search/manifests` -> liste des manifests recus.
- Enrichir les resultats de recherche avec les projets des manifests.
- Tests : receive, verify, cache, reject forged.

**Phase D — Anti-spam + privacy analysis**

Livrable : le systeme resiste au spam et respecte la privacy.

Contenu :
- PoW optionnel 16-bit sur la publication de manifests.
- Tests adversariaux : spam manifests, surdimensionnes, signatures
  invalides, replay ancien manifest.
- Documentation privacy : ce qu'un manifest revele vs ne revele pas.
- Audit de la surface d'exposition (fingerprinting potentiel).

#### Criteres de validation (gate S72)

- [ ] Manifest publie opt-in, recu et verifie par un peer
- [ ] Manifest sync stable entre 3 noeuds
- [ ] Les resultats de recherche incluent les projets des manifests
      distants
- [ ] Rate limiter respecte (1 manifest/heure/noeud)
- [ ] Privacy : aucune requete utilisateur envoyee au reseau
- [ ] Tests adversariaux verts

#### Delta tests estime

- Rust : +15-20 (manifest format, publish, discover, adversarial)
- Vitest : +5-8 (UI opt-in, manifest list)
- Total sortie estimee : ~1854

---

## Gate 2 — Go/no-go Arc 4

**Conditions :**
- SearchManifest fonctionne opt-in entre 3 noeuds
- RRV local trouve des apps (>= 100 entrees indexees)
- Proof Cards affichees dans les resultats de recherche
- Babel trouvable et avec Proof Card complete
- Aucun bug P0 ouvert

**Decision :** PO evalue si RRV@dev est fonctionnel. Si oui, l'arc 4
integre les resultats. Si non, l'arc 4 avance sans integration RRV.

---

## Arc 4 — Industrialisation (S73-S75)

L'arc 4 transforme le canari Babel en outil reutilisable, introduit la
gouvernance complete, durcit Factory, et assemble le pack produit
defendable.

---

### S73 — Gouvernance Complete + Factory Hardening

**Objectif produit :** Transformer le chemin Babel canari en outil
reutilisable et finaliser la gouvernance verifiable.

**Valeur :** Pour tout developpeur : creer une deuxieme app via Factory
sans regression. Pour le reseau : gouvernance complete avec evidence
trail.

**Resultat attendu :** Templates additionnels stables, compat v1/v2
maintenue, deuxieme app simple deployee, gouvernance documentee et
verifiable.

#### Phases

**Phase A — Templates additionnels + template lock stable**

Livrable : templates consolides, second app generee.

Contenu :
- Template `pyodide-notebook` (Pyodide + JupyterLite, ~500 LOC).
- Migration app v1/v2 robuste (tests compat).
- Template lock BLAKE3 hash stable.
- Deuxieme app simple via Factory (candidat : Repair Notebook — manuels
  reparation offline).
- Erreurs UX propres dans la page /factory.
- Docs create/publish dans `docs/factory/`.
- Tests : deux apps generees sans regression, hash templates stable,
  compat v1 maintenue.

**Phase B — Gouvernance formelle + evidence trail**

Livrable : gouvernance documentee avec trail verifiable.

Contenu :
- Spec formelle du processus CuratorVouched/Disendorsed dans
  `docs/protocol/GOVERNANCE.md`.
- Evidence trail : chaque decision de governance (vouch, disendorse,
  stale) est tracable dans le feed avec timestamp, auteur, et raison.
- Endpoint `/api/daemon/governance/timeline` : historique complet des
  actions de gouvernance par projet.
- Curator rotation : processus documente pour quand un curator key est
  compromise.
- Tests : governance timeline coherent, rotation scenario.

**Phase C — Factory hardening security**

Livrable : Factory resistante aux attaques.

Contenu :
- Tests adversariaux Factory :
  - Injection de secret dans un template (regex scan refuse)
  - Template malveillant avec path traversal
  - Concurrent apply race condition
  - Template hash mismatch detection
  - Tentative d'ecriture hors workspace via Factory
- Audit de securite interne : review systematique du broker.
- Documentation `docs/factory/SECURITY.md`.

**Phase D — Sprint process app + docs**

Livrable : documentation complete du processus Factory.

Contenu :
- Spec domain pack formelle dans `docs/factory/DOMAIN_PACKS.md`.
- Tutorial "Creer votre premiere app SBFB" (dev experience).
- Sprint template genere par Factory documente.
- Exemples de workflows : static app, storage app, bridge app.

#### Criteres de validation (gate S73)

- [ ] Deux apps generees par Factory sans regression
- [ ] Hash templates stable (content_hash BLAKE3)
- [ ] Compat v1 maintenue (apps existantes deploient)
- [ ] Gouvernance documentee et verifiable dans le feed
- [ ] Evidence trail fonctionne pour Babel et les autres apps
- [ ] Tests adversariaux Factory verts

#### Delta tests estime

- Rust : +10-15 (templates, gouvernance, adversarial Factory)
- Vitest : +5-8 (Factory UX, governance UI)
- Total sortie estimee : ~1879

---

### S74 — Babel Translation Beta + Domain Packs

**Objectif produit :** Tester la traduction comme fonctionnalite, sans
la rendre bloquante pour la preuve Factory.

**Valeur :** Pour les langues sous-dotees : un outil concret de
preservation linguistique. Pour le protocole : preuve que task_submit
et le bridge fonctionnent de bout en bout avec une app reelle.

**Resultat attendu :** Babel Reader avec traduction mock ou worker local,
reviews utilisateur, provenance du draft. Si NLLB/worker non pret, le
sprint reste valide via fixtures.

#### Phases

**Phase A — task_submit traduction mock + resultat stocke**

Livrable : Babel peut demander une traduction.

Contenu :
- `task_submit` traduction : mock backend ou worker local feature-gated.
- Resultat stocke dans storage P2P.
- Provenance du draft de traduction.
- Fallback fixtures officiel si NLLB/worker non pret.
- Tests : task submit, resultat stocke, fallback.

**Phase B — Review utilisateur + progression**

Livrable : les lecteurs peuvent reviewer les traductions.

Contenu :
- Interface de review minimale dans Babel.
- Reviews stockees dans storage P2P
  (`reviews/{translation_id}/{pubkey}`).
- Progression : affichage du nombre de reviews par traduction.
- Tests : review CRUD, progression affichage.

**Phase C — Domain pack enrichissement**

Livrable : domain packs ameliores.

Contenu :
- Domain pack Babel enrichi avec metadata traduction.
- Evaluation d'un second domain pack (candidat : Repair Notebook si pas
  fait en S73, ou Documentation Offline).
- Tests : domain pack parse, enrichissement correct.

**Phase D — Bridge integration avancee**

Livrable : Babel utilise le protocole complet.

Contenu :
- onEvent pour recevoir resultats de traduction.
- onStorageUpdate pour sync P2P des traductions.
- Provenance verifiable pour les traductions (pas juste l'app).
- Tests : bridge events, storage sync, provenance traduction.

#### Criteres de validation (gate S74)

- [ ] Si NLLB/worker pret : traduction fonctionnelle de bout en bout
- [ ] Si non pret : sprint valide via fixtures, aucune affirmation
      "compute correct" sans verification
- [ ] Reviews stockees et visibles
- [ ] Provenance du draft de traduction tracee
- [ ] Domain pack enrichi fonctionne

#### Delta tests estime

- Rust : +5-10 (task submit, storage, provenance traduction)
- Vitest : +5-10 (review UI, bridge events)
- Babel repo : +8-12 (traduction tests)
- Total sortie estimee : ~1907

---

### S75 — Pack Produit Defendable (Proof Pack + Evidence)

**Objectif produit :** Assembler SBFB + Factory + Babel + RRV en
demonstration defendable. Produire le proof pack complet que n'importe
qui peut examiner hors connexion.

**Valeur :** Pour tout evaluateur (bailleur, auditeur, contributeur
potentiel) : un dossier unique contenant toutes les preuves de
credibilite du projet. Pour le projet : decision go/no-go public
documentee.

**Resultat attendu :** Proof pack generable par CLI, contenant
provenance, feed snapshot, canary, SBOM, attestations CI, et un script
de verification autonome. Babel visible dans Browse, trouvable via RRV,
avec Proof Card complete. Release narrative documentee.

#### Structure du proof pack

```
proof-pack-v1.0.0/
  proof-pack-v1.0.0.json           # Manifest signe Ed25519
  proof-pack-v1.0.0.json.sig       # Signature detachee
  sbom.cdx.json                    # CycloneDX 1.6 SBOM
  CANARY.txt                       # Warrant canary frais
  cargo-deny-report.txt            # Sortie cargo-deny
  feed-snapshot.json               # Etat du feed
  artifacts/
    nexus-launcher-<os>-<arch>     # Binaires
    *.sha256                       # Checksums
    *.intoto.jsonl                 # Attestations SLSA v1
    *.intoto.jsonl.sig             # Signatures cosign
  verify.sh                        # Script verification autonome
```

#### Phases

**Phase A — Structure proof pack + CLI generate**

Livrable : `sbfb proof-pack generate` produit un dossier complet.

Contenu :
- Schema Rust `ProofPackManifest` (serde JSON).
- CLI `nexus-shell-daemon proof-pack generate`.
- Feed snapshot export : endpoint `GET /api/daemon/feed/snapshot`.
- Domain signing `DOMAIN_PROOF_PACK_V1`.
- Evidence pack final : factory.provenance.json + Babel proof card +
  Factory proof card + feed snapshot.
- Tests : generation, signature, round-trip parse.

**Phase B — Attestation build CI + SBOM**

Livrable : le pipeline release produit des artefacts avec SBOM et
attestation GitHub.

Contenu :
- Ajouter `cargo-sbom` au CI : generer `sbom.cdx.json` (CycloneDX 1.6).
- Ajouter `actions/attest-build-provenance@v2` dans `release.yml`.
- Capturer `cargo-deny check` dans un rapport fichier.
- Documenter le Rekor entry UUID dans les release notes.
- Signer le tag git avec SSH key.

**Phase C — Feed snapshot + canary refresh + verification externe**

Livrable : le proof pack contient un feed verifiable et un canary frais.
Outil de verification externe.

Contenu :
- Publier un nouveau CANARY.txt (refresh depuis le 2026-04-15).
- `feed-snapshot.json` : export complet ou resume signe.
- Verifier que `verify_chain()` fonctionne sur le snapshot exporte.
- `scripts/verify-proof-pack.sh` (bash portable : checksums, signature,
  canary freshness, optionnel cosign).
- `sbfb proof-pack verify --input <dir> --pubkey <hex>` (Rust complet).
- Documentation `docs/release/PROOF_PACK.md`.
- Test E2E : generer proof pack -> verifier -> assertion OK.

**Phase D — Release narrative + decision go/no-go**

Livrable : decision honnete documentee pour la release publique.

Contenu :
- Release narrative : document de ~2 pages expliquant ce que SBFB est,
  ce qu'il prouve, ce qu'il ne prouve pas.
- Babel proof card complete visible dans Browse.
- Factory proof card complete.
- RRV local + SearchManifest si gates OK.
- Decision go/no-go public documentee :
  - Pas de promesse "public" si durabilite/provenance/feed/pilote
    restent incomplets.
  - Le produit montre une app creee par Factory, pas seulement une spec.

#### Criteres de validation (gate S75)

- [ ] Proof pack generable par CLI en < 60 secondes
- [ ] Proof pack verifiable par `verify.sh` sans dependance autre que
      bash + jq + sha256sum
- [ ] SBOM CycloneDX 1.6 genere et inclus
- [ ] Canary frais (< 30 jours au moment de la release)
- [ ] Feed snapshot verifiable par `verify_chain()`
- [ ] Attestation GitHub sur le release workflow
- [ ] Babel Reader deploye via Factory, visible dans Browse
- [ ] Textes fixtures lisibles dans 3+ langues
- [ ] Provenance verifiable par Proof Card
- [ ] Trouvable via RRV search
- [ ] Feed entry ReleasePublished pour Babel

#### Delta tests estime

- Rust : +10-15 (proof pack generation, verification, feed snapshot)
- Vitest : +3-5 (integration UI proof pack)
- Total sortie estimee : ~1930

---

## Gates Factory FG0-FG10 — Section Transversale

Les gates Factory sont des criteres de qualite cumulatifs appliques a
chaque app generee par la Factory, quel que soit le sprint. Ils ne sont
pas sprint-specifiques mais progressivement implantes :

| Gate | Nom | Implante en | Description |
|------|-----|-------------|-------------|
| FG0 | Classification app | S67 Phase A | Domaine, risque donnees, bridge methods, network/compute needs |
| FG1 | Scope | S67 Phase A | MVP borne, non-goals explicites |
| FG2 | Template | S67 Phase B | Template id/version, hash BLAKE3, lockfile |
| FG3 | Manifest | S67 Phase A | Schema v2 valide, no `node_id`, bridge allowlist |
| FG4 | Diff | S68 Phase B | Preview obligatoire, approbation utilisateur |
| FG5 | Sandbox | S68 Phase A | Canonicalize, prefix check, symlink deny, no shell depuis iframe |
| FG6 | Secrets/deps | S68 Phase B | Scan secrets regex, lockfile, SBOM si publish |
| FG7 | Preview | S68 Phase D | Iframe sandbox, CSP, no external fetch par defaut |
| FG8 | Provenance | S68 Phase D | `factory.provenance.json`, generator version, template hash, variables hash, source commit |
| FG9 | Publish | S69 Phase B | Repo HTTPS, commit 40 hex, artifact hash, provenance, Browse, feed |
| FG10 | Review | S69 Phase E | Sprint review, verdict PASS, evidence pack |

Chaque app generee par Factory doit passer les gates disponibles au
moment de sa creation. Les gates sont documentes dans
`docs/factory/FACTORY_GATES.md` (cree en S65 Phase D, implementes
progressivement S67-S69).

---

## Graphe de dependances revise

### Dependances explicites

```
S65 Contrat Public
  |---> S67 Factory Foundation (vocabulaire + raw-op + SBFB.json v2 spec)
  |---> S70 RRV LocalOnly (vocabulaire de confiance + labels)
  |---> S71 Proof Cards (les niveaux S65 = le schema proof cards)

S66 Durabilite
  |---> S69 Pilote (daemon qui perd ses donnees = inutilisable)
  |---> S70 RRV LocalOnly (indexation necessite un feed qui survit)

S67 Factory Foundation
  |---> S68 Broker/Preview (les templates sont utilises dans le broker)
  |---> S69 Babel Canari (Babel est generee par Factory)
  |---> S72 SearchManifest (CuratorVouched prerequis)

S68 Broker/Preview
  |---> S69 Babel Canari (publish path complet)
  |---> S73 Factory Hardening (broker durcit)

S69 Babel Canari + Pilote
  |---> S70 RRV (le feedback pilote informe le design RRV, Babel = objet a indexer)
  |---> S74 Babel Translation (Babel reader existe)

S70 RRV LocalOnly
  |---> S71 Proof Cards (les resultats RRV portent les proof labels)

S71 Proof Cards
  |---> S72 SearchManifest (les proof cards enrichissent les manifests)
  |---> S75 Pack Produit (proof cards dans le proof pack)

S72 SearchManifest
  |---> S75 Pack Produit (SearchManifest dans le proof pack)

S73 Factory Hardening
  |---> S74 Babel Translation (templates enrichis)
  |---> S75 Pack Produit (Factory proof card)
```

### Dependances cachees

| ID | Dependance | Impact |
|----|-----------|--------|
| D-HIDDEN-1 | S65 -> S66 | Le fix auth tier (S65) doit etre fait AVANT que le feed devienne persistent (S66). Sinon, des operations non-autorisees seraient persistees indefiniment. |
| D-HIDDEN-2 | S66 -> S72 | Les SearchManifests doivent survivre aux restarts. |
| D-HIDDEN-3 | iroh 1.0 -> S66/S69 | iroh 1.0.0-rc.0 sorti le 2026-05-11. Decision point Gate 1. |
| D-HIDDEN-4 | S65 -> S67 | Factory affichera des badges de confiance. Le vocabulaire S65 doit etre utilise. |
| D-HIDDEN-5 | S67 -> S72 | `CuratorVouched` doit etre present pour que la gouvernance soit effective dans les manifests. |
| D-HIDDEN-6 | wasmtime CVEs -> S68 | Pas de wasmtime. Isolation OS-level. |
| D-HIDDEN-7 | S65 raw-op -> S67 + S72 | La migration serde_json::Value en S65 est prerequis pour que S67 et S72 ajoutent des ops feed sans bump. |

### Graphe ASCII

```
            S65 Contrat Public + raw-op
           / |
          /  |
S66 Durabilite  S67 Factory Foundation + CuratorVouched
    |           |     \
    |     S68 Broker/Preview + Gouvernance UI
    |           |
    +----> S69 Babel Canari + Pilote Ferme
                |
                |  (Gate 1)
                |
          S70 RRV LocalOnly (FTS5)
                |
          S71 Proof Cards
                |
          S72 SearchManifest Opt-In
                |
                |  (Gate 2)
                |
          S73 Factory Hardening + Gouvernance Complete
                |
          S74 Babel Translation Beta + Domain Packs
                |
          S75 Pack Produit Defendable
```

---

## Carry items — distribution complete

### S65 — OBLIGATOIRE

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-FEED-INSERT-NO-AUTH-TIER | 3/3 | Phase A — MANDATORY, securite feed |
| P2-VERIFY-ENTRY-VERSION-GUARD | 1/3 | Phase A — 5 LOC |
| P2-BADGE-WORDING-PREMATURE | pre-S14 | Phase B — coeur du sprint |

### S65 — Dette pair

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-COMMIT-TITLE-FORMAT | 2/3 | Phase D — hook pre-commit |
| P2-REVIEW-ORDER | 2/3 | Phase D — doc amendment |
| P2-PYTHON-BLOCK-EXEMPTION | 2/3 | Phase D — reclassifie "resolved by pivot S50" |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | Phase D — 1 LOC |
| P2-PLAYWRIGHT-SPECS-STALE (partie 1) | 2/3 | Phase D — suppression 12 fichiers zombies |

### S66 — Absorbe naturellement

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-FEED-JOIN-HANDLE-LEAK | 1/3 | Phase C — shutdown lifecycle |
| P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 | Phase C — crash recovery |

### S68 — Absorbe naturellement

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-PROVENANCE-404-BRIDGE | 2/3 | Phase B — UX verification |

### S69 — Absorbe naturellement

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | Phase C — cross-node verification |
| P2-PLAYWRIGHT-SPECS-STALE (partie 2) | 2/3 | Phase D — re-ecriture pour pages actuelles |

### Monitoring continu (pas de sprint specifique)

| Item | Status | Raison |
|------|--------|--------|
| P2-A-1 rand blocker | UPSTREAM | Pas d'action possible, rand 0.8 pin |
| P2-AUDIT-2 iroh transitives | UPSTREAM | Decision point Gate 1 (iroh 0.98 vs 1.0) |
| P2-G-1 exe lock | INTERMITTENT | Non reproductible, monitoring continu |

### Hors scope S65-S75

| Item | Raison |
|------|--------|
| T-NN+2 iframe Rust-wasm | Triggers non actifs |
| LT-5 redundancy persistence | Post-S75 sauf si pilote S69 l'exige |
| LT-7 quorum E2E | Post-S75 sauf si pilote S69 l'exige |

### Trigger-dependent

| Item | Trigger | Estimation |
|------|---------|-----------|
| LT-2 Radicle | Push tag v1.0 vers origin | S66 ou S67 si tag pousse |

---

## Decisions gelees (revisees pour v3)

Ces decisions ne doivent PAS etre re-debattues pendant les sprints
S65-S75. Elles sont le resultat de l'analyse croisee des recherches,
des 2 rounds de review, et du pivot PO.

### D-GEL-1 : iroh 0.98 pour les arcs 1-2

Rester sur iroh 0.98 pour S65-S69. Evaluer l'upgrade vers iroh 1.0
au moment de Gate 1. L'upgrade est un effort ~2-3 phases.

### D-GEL-2 : OS sandbox pour Factory, pas wasmtime

Factory S68 utilise l'isolation OS-level (processus + filesystem
sandbox via path allowlist et canonicalize), pas wasmtime (12 CVEs
avril 2026, dont 2 Critical CVSS 9.0).

### D-GEL-3 : Pilote ferme (2-3 personnes)

Le pilote S69 est ferme. R-iroh-audit P0 rend un pilote public
irresponsable sans audit tiers de la pile iroh.

### D-GEL-4 : Factory avant RRV (pivot v3)

L'arc 2 (Factory + Canari) precede l'arc 3 (RRV). La Factory est un
livrable concret en S67-S68 pour que Babel Reader soit la premiere app
canari en S69. RRV observe et explique ce que Factory produit. C'est
l'inversion majeure par rapport a la v2.

### D-GEL-5 : FTS5 d'abord, Tantivy en gate conditionnel

Le moteur de recherche S70 est SQLite FTS5. Si l'un des criteres de
gate est atteint (volume > 50K docs, p95 > 100ms, fuzzy demande par
3+ utilisateurs), migration vers Tantivy via trait `SearchEngine`.
Zero dependance ajoutee pour S70.

### D-GEL-6 : Babel MVP fixtures

S69 Babel fonctionne avec des traductions en fixtures (textes
pre-traduits). La traduction live via worker NLLB-200 est un stretch
goal de S74, non bloquant.

### D-GEL-7 : Feed raw-op (serde_json::Value)

Le champ `op` dans FeedEntry passe a `serde_json::Value`. Ajouter de
nouvelles operations au feed ne bumpe PAS FEED_FORMAT_VERSION. Les
anciens noeuds stockent, verifient (hash + sig), et propagent les ops
inconnues. FEED_FORMAT_VERSION ne bumpe que si la structure de
FeedEntry change (nouveau champ obligatoire, changement hash algo).

### D-GEL-8 : Vocabulaire "source verifiable"

Les apps du reseau sont decrites comme "source verifiable, provenance
verifiable, commun logiciel anti-capture". Le terme "open source"
est reserve au code SBFB lui-meme (AGPL-3.0, licence OSI).

### D-GEL-9 : node_id retire du manifeste

Le `node_id` dans SBFB.json est deprecie (optionnel, ignore, warning
log) puis retire en v2. L'attribution du deployeur est dans la
provenance signee Ed25519. SBFB.json devient un manifeste applicatif
portable. L'artefact hash est reproductible.

### D-GEL-10 : Factory dans le workspace nexus pour le MVP

Factory est un module du workspace Rust `nexus` (pas un repo separe).
Extraction en repo sibling possible post-S75 si la surface grandit.

---

## Risques et mitigations

### Risques critiques

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R1 | iroh 0.98 non maintenu apres 1.0 stable | MOYENNE | HAUT | Decision point Gate 1. Fallback : SQLite pour tout, iroh-docs comme transport. |
| R2 | Pilote revele problemes fondamentaux | MOYENNE (30%) | HAUT | S66 (durabilite) + S68 (Factory publish gate) specifiquement concus pour prevenir. Sprint fix dedie si > 5 bugs P0. |
| R3 | R-iroh-audit P0 | BASSE | CRITIQUE | Pilote ferme (pas public). |
| R4 | iroh-docs persistence buggy sous charge | FAIBLE (15%) | HAUT | Fallback : SQLite pour tout, iroh-docs comme transport P2P. |

### Risques moderes

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R5 | Factory complexity creep S67-S68 | MOYENNE | MOYEN | Gates Factory FG0-FG10 comme garde-fous. MVP strict. |
| R6 | NLLB-200 backend pas pret S74 | MOYENNE | MOYEN | Fixtures pre-traduites (D-GEL-6). Sprint valide sans NLLB. |
| R7 | NAT/firewall bloque P2P en pilote | HAUTE | MOYEN | Relais iroh, VPS Helsinki pre-deploy. |
| R8 | Testeurs ne donnent pas de feedback | MOYENNE | MOYEN | Guide structure + rappel apres 1 semaine. |
| R9 | Babel domain pack trop ambitieux | MOYENNE | MOYEN | 3 textes, 5 langues, fixtures uniquement. Pas de corpus massif. |

### Risques faibles

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R10 | SBOM generation echoue | MOYENNE | FAIBLE | Fallback cargo-deny output seul. |
| R11 | Reproductibilite bit-a-bit impossible (Rust) | HAUTE | FAIBLE | Documenter comme best-effort. |
| R12 | SBFB.json v2 casse apps existantes | FAIBLE | MOYEN | schema_version absent/1 = ancien format, test compat. |
| R13 | Path traversal dans Factory broker | FAIBLE | CRITIQUE | canonicalize + prefix check + tests traversal obligatoires. |
| R14 | FTS5 latence insuffisante post-launch | FAIBLE | MOYEN | Gate Tantivy mesurable. Migration triviale via trait. |

### Zones rouges heritees

| Zone | Etat | Impact S65-S75 |
|------|------|---------------|
| R-iroh-audit P0 | Pas d'audit tiers publie | Pilote ferme seulement |
| R-wasmtime-cve P0 | 12 CVEs avril 2026, 2 Critical | Pas de wasmtime dans Factory |
| R-libcrux-hax P2 | Post-quantum hors scope | AUCUN impact S65-S75 |
| R-pyodide-escape | Iframe sandbox mitige | Acceptable pour S74 template |

---

## Calendrier previsionnel

| Semaine | Sprint | Arc | Theme | Risque |
|---------|--------|-----|-------|--------|
| S1-S2 (mai-juin 2026) | S65 | 1 | Contrat Public + raw-op + gates Factory spec | 3/5 |
| S3-S4 | S66 | 1 | Durabilite (persistence + crash recovery) | 4/5 |
| S5-S6 | S67 | 2 | Factory Foundation + SBFB.json v2 + CuratorVouched | 3/5 |
| S7-S8 | S68 | 2 | Broker/Preview/Publish + Gouvernance UI | 4/5 |
| S9-S11 | S69 | 2 | Babel Canari + Pilote Ferme | 5/5 |
| -- | **GATE 1** | -- | Go/no-go Arc 3 + decision iroh 0.98/1.0 | -- |
| S12-S13 | S70 | 3 | RRV LocalOnly (FTS5) | 2/5 |
| S14-S15 | S71 | 3 | Proof Cards (resultats enrichis) | 2/5 |
| S16-S17 | S72 | 3 | SearchManifest (P2P discovery) | 4/5 |
| -- | **GATE 2** | -- | Go/no-go Arc 4 | -- |
| S18-S19 | S73 | 4 | Gouvernance Complete + Factory Hardening | 3/5 |
| S20-S21 | S74 | 4 | Babel Translation Beta + Domain Packs | 3/5 |
| S22-S24 | S75 | 4 | Pack Produit Defendable (Proof Pack + Evidence) | 3/5 |

**Total : ~24 semaines = ~6 mois (mai 2026 -> novembre 2026).**

**Contingence :** +2-4 semaines pour fixes pilote, iroh upgrade
eventuel, ou gates echouees. Budget total realiste : 26-28 semaines.

---

## Tests delta projete cumule

| Sprint | Arc | Rust entree | Rust sortie | Vitest sortie | Total sortie |
|--------|-----|-------------|-------------|---------------|-------------|
| S64 (base) | -- | -- | 1326 | 265 | 1597 |
| S65 | 1 | 1326 | ~1344 | ~275 | ~1625 |
| S66 | 1 | ~1344 | ~1369 | ~275 | ~1650 |
| S67 | 2 | ~1369 | ~1397 | ~287 | ~1690 |
| S68 | 2 | ~1397 | ~1422 | ~305 | ~1733 |
| S69 | 2 | ~1422 | ~1437 | ~320 | ~1773 |
| S70 | 3 | ~1437 | ~1455 | ~342 | ~1803 |
| S71 | 3 | ~1455 | ~1463 | ~352 | ~1821 |
| S72 | 3 | ~1463 | ~1483 | ~365 | ~1854 |
| S73 | 4 | ~1483 | ~1498 | ~375 | ~1879 |
| S74 | 4 | ~1498 | ~1508 | ~393 | ~1907 |
| S75 | 4 | ~1508 | ~1523 | ~401 | ~1930 |

**Projection S75 : ~1930 tests totaux** (vs 1597 actuels, **+333 net**
sur 11 sprints, ~30 tests/sprint en moyenne).

La repartition est coherente avec l'historique. Les sprints techniques
lourds (S66, S68, S72) produisent plus de tests Rust. Les sprints UX
(S65, S69, S71) produisent un mix Rust + Vitest. Les sprints Factory
(S67-S68) produisent un delta significatif des deux cotes.

---

## Comparaison v2 vs v3 — ce qui a change

| Aspect | v2 | v3 | Raison |
|--------|----|----|--------|
| Sequence Factory | Arc 3 (S73-S75) | Arc 2 (S67-S68) | Pivot PO : Factory visible plus tot, risque mesure |
| Sequence Babel | S75 (fin) | S69 (milieu) | Canari integre au pilote ferme |
| Sequence RRV | Arc 2 (S70-S72) | Arc 3 (S70-S72) | RRV observe ce que Factory produit |
| Gouvernance | S67 sprint entier | S67 Phase A+D (feed) + S68 Phase C (UI) + S73 Phase B (complete) | Repartie : minimal en S67-S68, complete en S73 |
| Proof Pack | S68 sprint entier | S75 (complet) avec evidence minimale en S68 Phase D | Evidence minimale tot, proof pack complet tard |
| Moteur recherche | Tantivy (D-GEL-5 fallback FTS5) | FTS5 first (D-GEL-5 gate Tantivy) | Recherche tantivy_vs_fts5_decision.md |
| Feed versioning | Bump v1->v2 en S67 (D-GEL-7) | Raw-op serde_json::Value, pas de bump (D-GEL-7) | Recherche feed_version_bump_strategy.md |
| node_id manifeste | Present obligatoire | Deprecie/retire (D-GEL-9) | Recherche factory_deploy_constraint_research.md |
| Vocabulaire | "source verifiable" (D-GEL-8) | Idem + COMMONS.md (D-GEL-8) | Recherche licence_anti_capture_research.md |
| Arc 3 nom | "Productif" | "Intelligence Verifiable" | RRV est le coeur de l'arc 3, pas Factory |
| Arc 4 ajout | N/A | "Industrialisation" (S73-S75) | Gouvernance complete + Factory hardening + evidence |
| Tests delta total | ~1863 | ~1930 | +67 tests grace a Factory + Babel acceptance |

---

## Tests d'acceptance Babel — distribution dans les sprints

Les 22 tests d'acceptance de la section 12 du document canari sont
distribues comme suit (+ 5 tests post-S70) :

| # | Test | Sprint | Phase |
|---|------|--------|-------|
| 1 | `deploy-from-repo` accepte app sans `node_id` | S69 | B |
| 2 | `deploy-from-repo` refuse manifest invalide | S69 | B |
| 3 | `deploy-from-repo` refuse methode bridge inconnue | S69 | B |
| 4 | `deploy-from-repo` refuse repo non HTTPS pour feed | S69 | B |
| 5 | app generee contient `index.html`, `SBFB.json`, SDK bridge | S69 | A |
| 6 | app generee contient planning sprint | S69 | A |
| 7 | app generee contient `factory.template.lock` | S69 | A |
| 8 | app generee contient `factory.provenance.json` | S69 | A |
| 9 | path traversal refuse | S69 | C |
| 10 | symlink refuse | S69 | C |
| 11 | secret fixture refuse | S69 | C |
| 12 | preview iframe smoke test | S69 | C |
| 13 | Babel affiche textes fixtures | S69 | A |
| 14 | Babel lit/ecrit progression storage | S69 | A |
| 15 | Babel appelle `identity_pubkey` | S69 | A |
| 16 | Babel affiche provenance | S69 | A |
| 17 | Babel lit feed cursor | S69 | A |
| 18 | Babel deploy -> Browse -> open -> provenance verify | S69 | B |
| 19 | feed `ReleasePublished` cree | S69 | B |
| 20 | test negatif: aucune methode `babel_*`, `factory_*`, `shell_*` | S69 | A |
| 21 | RRV trouve Babel localement | S70 | B |
| 22 | Proof Card affiche source, artifact, provenance, feed | S71 | B |
| 23 | Proof Card indique stale si source cassable | S71 | D |
| 24 | score de completude deterministe | S71 | D |
| 25 | FTS5 p95 acceptable sur corpus test | S70 | A |

---

*Document draft v3. Validation PO requise avant canonisation.
Toute modification ulterieure doit etre tracee par commit avec
reference au sprint qui l'exige.*
