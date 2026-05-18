# Roadmap v2 — Confiance Publique + RRV + Factory

## Date de redaction

2026-05-18

## Vision PO (citee integralement)

Decision PO 2026-05-13 :

> 6 sprints (5+1 reserve) pour credibilite publique protocole verifiable.
> S1 spec+feed local -> S2 sync P2P+anti-spam (gate scission Phase C) ->
> S3 verification tiers+UX -> S4 hardening public -> S5 go-live ->
> S6 reserve.

Amendement PO 2026-05-18, apres review croisee GPT 5.5 + recherches
S65-S75 :

> L'objectif n'est plus "go-live" mais "release publiquement defendable".
> Le focus est la credibilite, la durabilite et la reutilisabilite. Le
> projet est un commun logiciel anti-capture sous licence AGPL-3.0 (OSI),
> avec source verifiable et provenance verifiable. Le mot "open source"
> ne doit plus etre utilise generiquement pour decrire les apps du reseau
> -- elles sont a source verifiable, deployees depuis un depot public,
> avec provenance auto-attestee SLSA L1. Le terme "open source" reste
> reserve au code SBFB lui-meme (licence AGPL-3.0, qui EST une licence
> OSI).

---

## Principes directeurs (issus des reviews)

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
licence OSI. Si un logiciel tiers sur le reseau interdit certains usages,
il ne correspond pas a la definition OSI (criteres 5-6). L'ecosysteme
utilise le vocabulaire "source verifiable, provenance verifiable, commun
logiciel anti-capture" pour decrire les apps. Le mot "open source" est
reserve aux cas qui respectent effectivement les criteres OSI.

**P6 — Score de completude de preuve.** Le `confidence_score` dans les
Proof Cards (S71) est un score de completude de preuve, pas un "trust
score" social. Il mesure combien de couches de verification sont
presentes (provenance, signature live, curator endorsements, fraicheur,
licence). L'utilisateur decide de la confiance ; le score informe.

**P7 — Factory = module daemon/broker.** La Factory n'est pas une app
iframe. C'est un module Rust dans le daemon avec UI dans le shell React
(/factory). Les operations privilegiees (ecriture disque, git, builds)
passent par le broker cote daemon, pas par le bridge iframe. Le bridge
reste reserve aux apps sandboxees.

**P8 — Babel MVP = reader + storage + fixtures + provenance.** Le modele
NLLB-200 complet ne doit pas bloquer S75. L'app Babel fonctionne avec
des traductions en fixtures (textes pre-traduits, domaine public). La
traduction live via worker est un stretch goal.

**P9 — Feed version batching.** `CuratorVouched` (S67) et
`SearchManifestPublished` (S72) sont deux nouvelles operations feed.
Elles doivent si possible etre batchees dans un seul bump version
(FEED_FORMAT_VERSION v1 -> v2) pour eviter deux migrations successives.
La version est bumpee au premier sprint qui l'exige (S67), et S72
utilise la v2 directement.

**P10 — Topic gossip curator = global.** Le topic gossip des
annonces curator est global
(`BLAKE3("nexus-grid/curator-announce/v1")[..32]`), pas per-curator.
Les annonces contiennent la pubkey du curator et le BlobTicket de la
liste. Le CuratorRuntime filtre par attention set (quels curators
l'utilisateur suit). Ce n'est pas un topic per-curator.

---

## Gate 0 — Canonisation (cette etape)

Ce document est la source de verite pour les sprints S65-S75. Il
remplace et consolide :

- `.planning/research/public_verifiable_feed_roadmap.md` (roadmap S61-S66
  initiale)
- Les 7 documents de recherche S65-S75

**Criterium d'activation :** Le document est commit dans master et
reference dans CLAUDE.md. Les 7 documents de recherche restent en
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

## Arc 1 — Publiquement Defendable (S65-S69)

L'arc 1 construit la credibilite necessaire pour qu'un tiers puisse
examiner SBFB et conclure : "les promesses sont alignees avec le code,
les donnees survivent aux redemarrages, la gouvernance est lisible, et
le bundle de preuves est verifiable."

---

### S65 — Contrat Public

**Objectif produit :** Aligner chaque texte public (badges, labels,
docs) avec ce que le code garantit reellement.

**Valeur :** Pour tout utilisateur ou evaluateur : comprendre exactement
ce que SBFB prouve, ce qu'il ne prouve pas, et ce qu'il promet. Eliminer
les sur-promesses qui disqualifient la credibilite du projet.

**Resultat attendu :** Zero badge ou texte public qui sur-promet par
rapport aux garanties du code. Une taxonomie formelle a 6 niveaux de
confiance, documentee et appliquee dans toute l'UI.

#### Phases

**Phase A — Securite feed + taxonomie de confiance**

Livrable : Document `TRUST_TAXONOMY.md` dans `docs/protocol/` definissant
6 niveaux de confiance. Fixes securite MANDATORY absorbees.

Contenu :
- Fix P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY) : ajouter check
  auth tier dans `feed_insert()` handler (30-50 LOC dans
  `feed_sync.rs`). Un caller de tier T0 ne doit pas pouvoir inserer
  d'operations feed.
- Fix P2-VERIFY-ENTRY-VERSION-GUARD : ajouter
  `if entry.version != FEED_FORMAT_VERSION { return Err(...) }` en tete
  de `verify_entry()` (5 LOC dans `public_feed.rs`).
- Ecrire `TRUST_TAXONOMY.md` : 6 niveaux (Upload direct, Source
  lisible, Provenance auto-attestee SLSA L1, Signature verifiee live,
  Build reproductible futur, Feed verifie hash-chain) + niveaux
  transversaux (AGPL-3.0, Curator vouch, Sandbox).
- Corriger les textes du Protocol Explorer :
  - "Le code sur le reseau = le code du depot" -> "L'archive reseau est
    construite depuis le depot source par le noeud local. C'est une
    auto-attestation."
  - "Le modele F-Droid/Linux" -> "Inspire par F-Droid -- les apps
    publiques sont deployees depuis leur code source."
  - "Chaine de preuve" -> "Chaine de provenance"
- Corriger PUBLISH_MODEL.md : "open source verifie" -> "Release avec
  provenance auto-attestee"

**Phase B — Migration badges UI**

Livrable : tous les badges/labels UI migres vers la nouvelle taxonomie.

Contenu :
- Browse.tsx : "Verifie" + ShieldCheck -> "Provenance" + FileCheck
- BrowsedProject.tsx : idem + etat dynamique post-verification
- GpuConsentDialog.tsx L2 : "Projets open source verifies" ->
  "Apps deployees depuis un depot public (provenance auto-attestee)"
- Network.tsx : "L2 -- Open source" -> "L2 -- Depot public"
- Curators.tsx : "curator de confiance" -> "curator"
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

**Phase D — Non-regression wording + dette pair**

Livrable : script CI de non-regression sur les textes de confiance.
Carry items de process absorbes.

Contenu :
- Script `scan-trust-wording.sh` (grep les termes interdits dans l'UI :
  "verifie" sans qualification, "de confiance" dans un contexte
  automatique, "Le code sur le reseau = le code")
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

#### Criteres de validation (gate S65)

- [ ] Zero badge "Verifie" dans l'UI sans verification live prealable
- [ ] TRUST_TAXONOMY.md existe et couvre les 6 niveaux
- [ ] `feed_insert()` rejette les callers de tier T0
- [ ] `verify_entry()` rejette les entries avec version != FEED_FORMAT_VERSION
- [ ] Le Protocol Explorer ne contient plus "le code sur le reseau =
      le code du depot" ni "modele F-Droid/Linux" sans nuance
- [ ] `scan-trust-wording.sh` passe en CI sans faux positif
- [ ] Tous les tests verts (Rust + Vitest + size-limit)

#### Delta tests estime

- Rust : +8-12 (auth tier, version guard, wording scan)
- Vitest : +5-10 (badge dynamique, migration labels)
- Total sortie estimee : ~1619

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

Note de precision (correction GPT 5.5 #1) : iroh-docs **sait** persister
si `NodeConfig.with_data_dir()` est utilise -- c'est une feature mature
de la lib, backed par redb. Le gap est que le daemon ne le cable pas
(`runtime.rs` ne passe jamais `data_dir` au NodeConfig). iroh-blobs,
lui, est bien encore `MemStore::default()` ; la persistence necessite
le passage a `FsStore` (iroh-blobs 0.100, feature `fs-store`).

Note de precision (correction GPT 5.5 #2) : la correction de la "ligne
manquante pour data_dir" est simple en isolation, mais le vrai cout du
sprint est : tests de restart (stop -> start -> verify que rien n'est
perdu), gestion du data_dir root (creation, permissions, migration),
feed republish DB -> iroh-docs au boot, validation multi-daemon que la
persistence ne casse pas la sync P2P. D'ou 5 phases, pas 1.

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

Risques : les tests existants utilisent des tempdir -- s'assurer que
les tests unitaires continuent de tourner en mode in-memory.

**Phase B — iroh-blobs FsStore**

Livrable : les archives apps survivent au restart du daemon.

Contenu :
- `Cargo.toml` workspace : activer feature `fs-store` sur iroh-blobs
- `node.rs` : quand `data_dir` fourni, utiliser
  `FsStore::load(data_dir.join("blobs"))` au lieu de
  `MemStore::default()`
- `blobs.rs` : adapter `BlobsClient` pour fonctionner avec `FsStore`
  (pattern enum ou toujours FsStore avec tempdir en tests)
- Le type `Node` doit accepter le nouveau store (generique ou enum)
- Tests : deux noeuds, A ajoute un blob, reboot, B fetch via ticket
  depuis le blob persiste de A

Risques : refactoring profond du crate fondation. Tous les crates
downstream (`nexus-coordinator-rs`, `nexus-shell-daemon-core`,
`nexus-shell-daemon`, `nexus-worker-core`, `nexus-test-harness`)
compilent contre `Node` -- le changement de type du store est breaking
pour la signature des types.

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
  crash-safety renforcee (acceptable pour un daemon P2P)
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
- [ ] Aucune regression sur les 1619+ tests existants

#### Delta tests estime

- Rust : +15-25 (persistence, crash recovery, restart, multi-daemon)
- Vitest : 0 (pas de changement frontend)
- Total sortie estimee : ~1644

---

### S67 — Gouvernance De Confiance

**Objectif produit :** Rendre la confiance lisible, pluraliste et
dissentable.

**Valeur :** Pour tout utilisateur : voir qui approuve quoi, quand, et
avec quel desaccord. La gouvernance n'est plus binaire (subscribe/
unsubscribe) mais riche (scope, endorsement signe, dissent visible,
fraicheur).

**Resultat attendu :** Un endorsement signe `CuratorVouched` dans le
feed, une agregation multi-curator avec scope dans le Browse, un dissent
visible quand deux curators sont en desaccord.

#### Decisions architecturales

- **Endorsement via feed, pas via CuratorList.** Les endorsements
  granulaires (avec scope, commentaire, date) vont dans le feed comme
  `CuratorVouched`. La CuratorList reste le mecanisme de discovery
  rapide.
- **Pas de trust score numerique.** On montre les faits (qui endorse,
  qui objecte, quand) et laisse l'utilisateur decider.
- **Scope comme string libre.** Convention documentee ("security",
  "quality", "license") mais extensible sans upgrade coordonne.
- **Disendorsement = action publique.** Un `CuratorDisendorsed` est
  visible par tous. Pas de moderation opaque.
- **Topic gossip curator = global** (correction factuelle GPT 5.5
  #13). Les annonces curator transitent sur un topic global
  `BLAKE3("nexus-grid/curator-announce/v1")[..32]`. Le CuratorRuntime
  filtre par attention set localement. Ce n'est PAS un topic per-curator.

#### Feed version bumping (correction GPT 5.5 #11)

`CuratorVouched` et `CuratorDisendorsed` sont les premiers nouveaux
types d'operations feed depuis le lancement. Le `FEED_FORMAT_VERSION`
passe de 1 a 2 dans ce sprint. Le `SearchManifestPublished` (S72)
utilisera la meme v2 sans re-bump, puisqu'un noeud v2 sait deja
deserialiser des variants inconnus gracieusement via le tagged union
serde.

La politique post-v1.0 (tags poses) s'applique : chaque break bumpe la
version, chaque decoder accepte un range. Le bump v1 -> v2 ici EST le
premier break post-tag. Les noeuds pre-S67 ne pourront pas deserialiser
les operations `CuratorVouched` mais ignoreront gracieusement les
variants inconnus (propriete du tagged union serde avec
`#[serde(other)]`).

#### Phases

**Phase A — CuratorVouched + CuratorDisendorsed dans le feed**

Livrable : deux nouveaux types d'operations feed, signes et verifiables.

Contenu :
- Ajouter `CuratorVouched(CuratorVouchedPayload)` et
  `CuratorDisendorsed(CuratorDisendorsedPayload)` au enum
  `PublicFeedOperation`
- `CuratorVouchedPayload` : project_id, curator_pubkey, scope (String),
  comment (Option<String>, max 280 chars)
- `CuratorDisendorsedPayload` : project_id, curator_pubkey, reason
  (String), comment (Option<String>)
- Domain de validation pour les deux types
- Bumper FEED_FORMAT_VERSION a 2
- Ajouter `#[serde(other)]` variant pour forward-compat
- Tests unitaires : insert, replay, verify_chain avec les nouveaux types,
  serde roundtrip, adversarial forgery

**Phase B — Multi-curator trust overlay + scope**

Livrable : agregation multi-curator avec scope dans le Browse.

Contenu :
- Ajouter `scope: Option<String>` a `CuratorList` avec
  `#[serde(default)]`
- Propager le scope dans `BrowseEntry` :
  `endorsement_scopes: Vec<String>`
- Dans `BrowseAggregator.aggregate()`, agreger les endorsements de tous
  les curators pour chaque project_id
- Calculer le breakdown par scope (securite: 1, qualite: 2, etc.)
- Endpoint daemon `/api/daemon/browse` retourne les donnees
  d'agregation

**Phase C — UX confiance visible (badges, timeline, dissent)**

Livrable : la page Browse et Curators montrent la gouvernance.

Contenu :
- Page Curators : freshness ("derniere mise a jour il y a X"), scope
  du curator, badge "inactif" si > 90 jours
- Page Browse : nombre de curators, breakdown par scope, indicateur de
  dissent, freshness de la derniere verification
- Page BrowsedProject : timeline des endorsements/disendorsements avec
  commentaires et dates

**Phase D — Stale detection + tests adversariaux**

Livrable : detection automatique des sources perimees et tests
adversariaux sur la gouvernance.

Contenu :
- Timer coordinator qui re-verifie periodiquement les repos source des
  apps deployees
- Emission automatique de `SourceBecameStale` quand un repo diverge ou
  est unreachable
- Tests adversariaux : curator malveillant, split-brain curators, stale
  replay, forgery disendorsement, flood disendorsement (rate limited)

#### Carry items absorbes

Aucun carry item directement, mais le bump FEED_FORMAT_VERSION v2
prepare le terrain pour S72 (SearchManifestPublished).

#### Criteres de validation (gate S67)

- [ ] `CuratorVouched` et `CuratorDisendorsed` inserables et
      rejouables dans le feed
- [ ] `verify_chain()` passe sur un feed contenant les 4 types
      d'operations (Release, Stale, Vouched, Disendorsed)
- [ ] Agregation multi-curator visible dans le Browse
- [ ] Dissent visible quand deux curators sont en desaccord sur une app
- [ ] Freshness des endorsements affichee dans l'UI
- [ ] Tests adversariaux verts (forgery, flood, replay)

#### Delta tests estime

- Rust : +10-15 (feed ops, agregation, stale detection, adversarial)
- Vitest : +5-8 (composants UI, formatage, rendu conditionnel)
- Total sortie estimee : ~1667

---

### S68 — Pack De Preuves Release

**Objectif produit :** Assembler un bundle de preuves verifiable que
n'importe qui peut examiner hors connexion.

**Valeur :** Pour tout evaluateur (bailleur, auditeur, contributeur
potentiel) : un dossier unique contenant toutes les preuves de
credibilite du projet, verifiable avec un script bash et sha256sum.

**Resultat attendu :** Un proof pack generable par CLI, contenant
provenance, feed snapshot, canary, SBOM, attestations CI, et un script
de verification autonome.

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

Le manifest racine contient :
- Identite (creator_node_id, Ed25519 hex)
- Git (repo_url, commit_sha, tag, mirror_urls)
- Artefacts (name, blake3, sha256, size, slsa_attestation, rekor_entry)
- Provenance (methode, SLSA level qualifie "L1 local / L2 CI")
- Feed snapshot (total entries, last seq, last entry hash, authors count)
- Canary (date, status, headline, signature, signing model FROST K=2/N=3)
- Supply chain (cargo-deny clean, npm audit clean, SBOM file, lock hashes)
- Tests (Rust count, Vitest count, all green, CI run URL)

#### Phases

**Phase A — Structure proof pack + CLI generate**

Livrable : `sbfb proof-pack generate` produit un dossier complet.

Contenu :
- Schema Rust `ProofPackManifest` (serde JSON)
- CLI `nexus-shell-daemon proof-pack generate`
- Feed snapshot export : endpoint `GET /api/daemon/feed/snapshot`
- Domain signing `DOMAIN_PROOF_PACK_V1`
- Tests : generation, signature, round-trip parse

**Phase B — Attestation build CI + SBOM**

Livrable : le pipeline release produit des artefacts avec SBOM et
attestation GitHub.

Contenu :
- Ajouter `cargo-sbom` au CI : generer `sbom.cdx.json` (CycloneDX 1.6)
- Ajouter `actions/attest-build-provenance@v2` dans `release.yml`
- Capturer `cargo-deny check` dans un rapport fichier
- Documenter le Rekor entry UUID dans les release notes
- Signer le tag git avec SSH key

**Phase C — Feed snapshot + canary refresh**

Livrable : le proof pack contient un feed verifiable et un canary frais.

Contenu :
- Publier un nouveau CANARY.txt (refresh depuis le 2026-04-15)
- `feed-snapshot.json` : export complet ou resume signe
- Verifier que `verify_chain()` fonctionne sur le snapshot exporte
- Test E2E : generer proof pack -> verifier -> assertion OK

**Phase D — Outil de verification externe**

Livrable : `verify.sh` + `sbfb proof-pack verify` utilisables par un
tiers.

Contenu :
- `scripts/verify-proof-pack.sh` (bash portable : checksums, signature,
  canary freshness, optionnel cosign)
- `sbfb proof-pack verify --input <dir> --pubkey <hex>` (Rust complet)
- Documentation `docs/release/PROOF_PACK.md` (structure, 3 methodes de
  verification, interpretation des resultats)

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-PROVENANCE-404-BRIDGE (2/3) | Phase A — distinguer "projet inexistant" vs "pas de provenance" |
| P2-COVERAGE-DEPLOY-E2E (2/3) | Phase A — le test E2E deploy roundtrip EST le proof pack |

#### Criteres de validation (gate S68)

- [ ] Proof pack generable par CLI en < 60 secondes
- [ ] Proof pack verifiable par `verify.sh` sans dependance autre que
      bash + jq + sha256sum
- [ ] SBOM CycloneDX 1.6 genere et inclus
- [ ] Canary frais (< 30 jours au moment du pilote)
- [ ] Feed snapshot verifiable par `verify_chain()`
- [ ] Attestation GitHub sur le release workflow

#### Delta tests estime

- Rust : +8-12 (proof pack generation, verification, feed snapshot)
- Vitest : +3-5 (integration UI proof pack si applicable)
- Total sortie estimee : ~1684

---

### S69 — Pilote Ferme

**Objectif produit :** Valider SBFB avec 2-3 testeurs reels sur des
machines qui ne sont pas celle du developpeur.

**Valeur :** Pour le projet : premiere confrontation avec la realite
(installation, connexion P2P, stabilite 24h). Pour les testeurs :
contribuer a un commun logiciel avant sa release publiquement
defendable.

**Resultat attendu :** 2 testeurs installent, se connectent, synchronisent
le feed, deploient une app, et le daemon tourne 24h sans crash. Decision
go/no-go documentee pour l'arc 2.

#### Modele du pilote

- **Ferme** (2-3 personnes, pas public). Rationale : R-iroh-audit P0
  rend un pilote public irresponsable sans audit tiers de la pile iroh.
- **Zero telemetrie**. Pas de Sentry, pas de Google Forms, pas de
  crash reporting centralise. Le feedback passe par l'app Ideas Hub
  deployee sur le reseau (dogfooding).
- **Coherence philosophique**. La distribution se fait par invite
  (tickets feed + installeurs).
- **Crash logs locaux**. Les mecanismes existants suffisent pour 2-3
  testeurs (launcher.log, daemon JSON logs, SecurityEvent JSONL, ETW
  Windows). Un bouton "Exporter les logs" est ajoute.

#### Phases

**Phase A — Checklist prerequisites + mecanisme invite**

Livrable : tout est pret pour inviter des testeurs.

Contenu :
- Verifier tous les carry P2 critiques resolus ou acceptes
- Implementer endpoint HTTP pour distribuer un ticket de feed :
  `POST /api/v1/pilot/invite` -> genere invite token + feed ticket
- Mettre a jour OnboardingEmpty.tsx (enlever commandes Python obsoletes,
  ajouter "Entrez votre ticket d'invitation")
- Fix P2-VERIFY-LOCAL-KEY-ONLY : ajouter resolver pkarr dans le path
  de verification cross-node (50-80 LOC) -- necessaire avant exposition
  externe
- Script de generation des invites pour le mainteneur

**Phase B — Installeur cross-platform teste**

Livrable : les installeurs fonctionnent sur des machines propres.

Contenu :
- Tester NSIS sur VM Windows 11 fresh
- Tester .deb sur Ubuntu 24.04 LTS VM
- Tester .dmg sur macOS si machine dispo
- Fix les bugs d'installation trouves
- Documenter les prerequisites systeme

**Phase C — Feedback collector integre**

Livrable : les testeurs peuvent reporter des problemes via le reseau.

Contenu :
- Deployer Ideas Hub comme "Pilot Feedback" sur le reseau
- Bouton "Exporter les logs" dans le tray menu (zip 7 jours)
- Bouton "Rapport de bug" dans le frontend (formulaire structure)
- Guide testeur PDF/MD

**Phase D — Scenarios de test guides**

Livrable : chaque testeur a un parcours structure.

Contenu :
- 8 scenarios de test (installation, join, browse, deploy, feed sync,
  verification provenance, restart, stabilite 24h)
- Formulaire de resultat par scenario (passe/echoue/commentaire)
- Re-ecriture des specs Playwright pour les pages actuelles (partie 2
  de P2-PLAYWRIGHT-SPECS-STALE)

**Phase E — Analyse go/no-go**

Livrable : decision honnete documentee.

Contenu :
- Collecter tous les retours (Ideas Hub + emails)
- Categoriser : bugs critiques / UX / cosmetics / suggestions
- Document "Bilan pilote"
- Decision : go / go-with-fixes / no-go pour l'arc 2

#### Carry items absorbes

| Item | Traitement |
|------|------------|
| P2-VERIFY-LOCAL-KEY-ONLY (2/3) | Phase A |
| P2-PLAYWRIGHT-SPECS-STALE (2/3) | Phase D (re-ecriture) |

#### Criteres de validation (gate S69 = Gate 1)

| Critere | Go | No-Go |
|---------|-----|--------|
| Installation | 2/3 testeurs installent sans aide | 0/3 reussit ou 2/3 ont besoin d'aide |
| Premier lancement | Daemon demarre + browser ouvre en < 30s | Crash au demarrage |
| Connexion P2P | 2 noeuds se voient en < 5 min | Aucune connexion apres 15 min |
| Deploy app | 1 testeur deploie depuis source | Deploy echoue ou provenance invalide |
| Feed sync | Feed synchronise entre 2+ noeuds | Divergence ou corruption |
| Restart | Daemon redemarrage propre | State corrompu ou impossibilite |
| Stabilite 24h | Daemon tourne 24h sans crash | Crash, OOM, ou freeze |

Si > 5 bugs P0/P1 : sprint fix dedie avant S70. Le sprint reserve
(herite de l'ancien S6 de la roadmap Public Feed) est utilise ici.

#### Delta tests estime

- Rust : +5-10 (fixes bug reports pilote)
- Vitest : +10-15 (re-ecriture Playwright pour pages actuelles)
- Total sortie estimee : ~1709

---

## Gate 1 — Go/no-go Arc 2

**Conditions :**
- Le pilote ferme est operationnel (2+ testeurs actifs)
- Les bugs critiques decouverts sont fixes
- Le proof pack est complet et verifiable
- Le daemon survit a 24h sans intervention

**Decision :** PO evalue le feedback pilote. Si les criteres gate S69
sont remplis, l'arc 2 demarre. Sinon :
- Si < 3 bugs P0 : S70 absorbe les fixes
- Si >= 3 bugs P0 : sprint fix dedie, re-pilote

**Decision iroh 0.98 vs 1.0 :** Evaluee ici. Si le pilote revele des
bugs iroh-docs/iroh-blobs fixes uniquement en iroh 1.0 : l'upgrade
devient prioritaire (effort ~2-3 phases). Si le pilote tourne bien
sur 0.98 : rester sur 0.98 pour l'arc 2.

---

## Arc 2 — Intelligent et Verifiable (S70-S72)

L'arc 2 construit la capacite de chercher dans le reseau et de prouver
la qualite de chaque resultat. La recherche reste locale par defaut
(privacy by design). Les manifests sont opt-in.

---

### S70 — RRV LocalOnly

**Objectif produit :** Un moteur de recherche local qui indexe les apps,
le feed et la provenance du noeud.

**Valeur :** Pour tout utilisateur : trouver une app par mot-cle au lieu
de parcourir un listing exhaustif. Chaque resultat est cite (source,
hash, timestamp).

**Resultat attendu :** `GET /api/daemon/search?q=traduction` retourne
des resultats pertinents avec citations exactes, en < 50ms.

#### Choix technique : Tantivy, pas FTS5

**Moteur :** Tantivy (~0.22), bibliotheque Rust de recherche full-text
inspiree d'Apache Lucene. Embarquee dans le daemon, index sur disque
dans `~/.sbfb/search_index/`.

**Rationale :** BM25 + fuzzy + phrase queries + stemming 17 langues.
Independance (index separe de coordinator.db). Performances 2x Lucene.
Standard Rust (ParadeDB, Quickwit, Turso l'utilisent).

**Gate Tantivy (correction GPT 5.5 #8) :** Si l'integration Tantivy
derive (MSRV incompatible, breaking change, ou complexite d'integration
trop elevee), Phase A bascule sur SQLite FTS5 temporaire. Le produit ne
doit pas bloquer sur le moteur de recherche. FTS5 est deja disponible
via rusqlite et couvre le cas nominal (recherche metadata sans fuzzy).

#### Phases

**Phase A — Index local + API**

Livrable : un index Tantivy operationnel avec endpoint de recherche.

Contenu :
- Ajouter `tantivy` au workspace Cargo.toml
- `search_index.rs` dans nexus-coordinator-rs : schema Tantivy
  (project_name, description, category, keywords, repo_url,
  artifact_hash, timestamp), creation, indexation, recherche
- `search_api.rs` dans nexus-shell-daemon :
  `GET /api/daemon/search?q=...&limit=...&offset=...`
- Wire dans `http.rs` router
- Tests : index creation, search, empty results, special chars

**Phase B — Indexation au boot + incrementale + SBFB.json enrichi**

Livrable : l'index est peuple automatiquement.

Contenu :
- Au boot : indexer browse entries, feed entries, provenance records
- Trigger incrementale : re-indexer a chaque ProjectAnnouncement,
  deploy reussi, ou FeedEntry insere
- Enrichir SBFB.json : ajouter `description`, `keywords`, `license`
  (champs optionnels, `#[serde(default)]`)
- Indexer le contenu des README.md dans les archives zip
- Tests : rebuild d'index, indexation incrementale

**Phase C — Bridge method + citations**

Livrable : les apps iframe peuvent chercher via le bridge.

Contenu :
- Ajouter `search` au BridgeMethodSchema dans `protocol.ts`
- Handler dans `useBridge.ts`
- Chaque resultat retourne des citations exactes (source_type,
  entry_hash, file_path, line)
- Tests Vitest : bridge search dispatch, citation format

**Phase D — App sbfb-search MVP**

Livrable : une app de recherche standalone dans `examples/sbfb-search/`.

Contenu :
- HTML + JS vanilla (meme pattern que Explorer/Ideas Hub)
- Search bar + resultats avec extraits et citations
- Design dark theme coherent
- SBFB.json manifest v2
- Tests : taille bundle, fonctionnalite search

#### Criteres de validation (gate S70)

- [ ] `search?q=explorer` retourne l'app Protocol Explorer
- [ ] Temps de reponse < 50ms pour < 1000 documents
- [ ] Index reconstruit au boot en < 5s
- [ ] Citations exactes (source_type, entry_hash) dans chaque resultat
- [ ] FTS5 index >= 100 entrees (metadata de tous les projets, feed
      entries, provenance)

#### Delta tests estime

- Rust : +12-18 (index, search, incremental, citations)
- Vitest : +8-12 (bridge search, app sbfb-search)
- Total sortie estimee : ~1739

---

### S71 — RRV Proof Cards

**Objectif produit :** Enrichir chaque resultat de recherche avec un
"passeport de confiance" deterministe.

**Valeur :** Pour tout utilisateur : comprendre en un coup d'oeil
pourquoi un resultat est fiable (ou non). Le score est un score de
completude de preuve (correction GPT 5.5 #12), pas un "trust score"
social.

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

Facteurs de risque calcules automatiquement : `no_provenance`,
`stale_source`, `no_curator`, `single_curator`, `unverified_deploy`,
`old_release`, `no_open_source` (info seulement).

#### Phases

**Phase A — ProofCard data model + computation**

Livrable : `ProofCard` struct + calcul deterministe.

Contenu :
- `proof_card.rs` dans nexus-coordinator-rs : struct ProofCard (source,
  hash, license, freshness, provenance, risk, curation, confidence)
- `compute_proof_card(project_id, db, browse_entry)` -> ProofCard
- Formule de score documentee dans le code et dans un doc
- Tests : score computation, risk detection, edge cases, determinism

**Phase B — API + bridge**

Livrable : les proof cards accessibles via API et bridge.

Contenu :
- `GET /api/daemon/proof-card/{project_id}`
- Bridge method `proof_card_get(project_id)` -> ProofCard
- Tests : API response format, bridge dispatch

**Phase C — Integration dans search results**

Livrable : l'app sbfb-search affiche les proof cards.

Contenu :
- Composant ProofCard (HTML/JS) : score bar, facteurs, actions
- "Verifier la provenance" interactif via bridge
- Integration dans les resultats de recherche et dans Browse

**Phase D — Tests adversariaux**

Livrable : le score ne ment pas.

Contenu :
- Proof card spoofing : un projet sans provenance ne peut pas afficher
  score > 50
- Risk factor injection : HTML dans description sanitized
- Stale detection : SourceBecameStale -> risk "stale_source"
- Score determinism : memes entrees -> meme score (pas de randomness)

#### Criteres de validation (gate S71)

- [ ] ProofCard generable pour tout projet du Browse
- [ ] Score deterministe (memes inputs = meme score)
- [ ] Un projet sans provenance a un score <= 50
- [ ] Risk factors affiches dans l'UI
- [ ] Tests adversariaux verts

#### Delta tests estime

- Rust : +5-8 (proof card computation, adversarial)
- Vitest : +5-8 (composant ProofCard, integration search)
- Total sortie estimee : ~1755

---

### S72 — SearchManifest Opt-In

**Objectif produit :** Permettre aux noeuds de publier volontairement
un index signe, enrichissant la decouverte P2P.

**Valeur :** Pour le reseau : decouverte d'apps au-dela du voisinage
gossip immediat. Pour l'utilisateur : la recherche trouve des apps
hebergees par des noeuds distants, avec verification.

**Resultat attendu :** Un noeud peut publier un `SearchManifest` signe
opt-in. Les autres noeuds le recoivent, le verifient, et enrichissent
leurs resultats de recherche.

#### Decisions architecturales

- **Opt-in explicite.** Le daemon ne publie PAS de manifest par defaut.
  L'utilisateur active la publication via
  `POST /api/daemon/search/publish-manifest` ou checkbox dans le shell.
- **Privacy by design.** Les requetes de recherche de l'utilisateur ne
  sont jamais envoyees au reseau. Les manifests enrichissent la
  decouverte, pas la recherche elle-meme.
- **SearchManifestPublished dans le feed.** Nouvelle operation feed
  (utilise la v2 bumpee en S67, pas de nouveau bump).
- **Gossip topic dedie.** `BLAKE3("nexus-grid/search-manifest/v1")[..32]`
  pour la publication en temps reel. Le feed pour la decouverte
  retroactive.

#### Phases

**Phase A — SearchManifest format + signing**

Livrable : wire format defini, signable et verifiable.

Contenu :
- Domain constant `DOMAIN_SEARCH_MANIFEST_V1` dans `canonical.rs`
- `SearchManifest` struct : v, node_id, created_at, projects (max 256),
  feed_cursor, index_stats, signature
- Limits par champ : description <= 280 bytes, keywords <= 10 x 64
  bytes, total manifest <= 1 MB
- Sign/verify via canonical_bytes pattern existant
- Tests : sign, verify, reject tampered, reject oversized

**Phase B — Publication opt-in via iroh**

Livrable : un noeud peut publier son manifest sur le reseau.

Contenu :
- Gossip topic search-manifest
- `POST /api/daemon/search/publish-manifest`
- Stockage blob + annonce gossip
- Rate limiter : 1 publication par heure par noeud
- `SearchManifestPublished` feed operation type (variant du tagged union,
  v2 deja bumpee)
- Tests : publish, gossip announce, rate limit

**Phase C — Discovery + verification**

Livrable : les manifests des peers sont recus, verifies et exploites.

Contenu :
- Subscribe au gossip topic search-manifest
- Recevoir, parser, verifier signatures des manifests peers
- Cache DashMap similaire a CuratorRuntime
- `GET /api/daemon/search/manifests` -> liste des manifests recus
- Enrichir les resultats de recherche avec les projets des manifests
- Tests : receive, verify, cache, reject forged

**Phase D — Anti-spam + privacy analysis**

Livrable : le systeme resiste au spam et respecte la privacy.

Contenu :
- PoW optionnel 16-bit sur la publication de manifests
- Tests adversariaux : spam manifests, surdimensionnes, signatures
  invalides, replay ancien manifest
- Documentation privacy : ce qu'un manifest revele vs ne revele pas
- Audit de la surface d'exposition (fingerprinting potentiel)

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
- Total sortie estimee : ~1783

---

## Gate 2 — Go/no-go Arc 3

**Conditions :**
- SearchManifest fonctionne opt-in entre 3 noeuds
- RRV local trouve des briques (>= 100 entrees indexees)
- Proof Cards affichees dans les resultats de recherche
- Aucun bug P0 ouvert

**Decision :** PO evalue si RRV@dev est fonctionnel. Si oui, Factory
l'integre. Si non, Factory est standalone (templates statiques
uniquement, pas de recherche integreo).

**Contingence :** Si RRV est insuffisant, S73-S75 demarrent quand meme
mais sans integration RRV. L'arc 3 est le plus independant de la
roadmap.

---

## Arc 3 — Productif (S73-S75)

L'arc 3 construit la capacite de creer des apps SBFB efficacement et
valide l'ensemble du protocole avec une premiere app reelle (Babel).

**Decision architecturale (correction GPT 5.5 #9) :** Factory est un
module daemon/broker Rust, pas une app iframe. L'UI Factory est une
page React du shell (`/factory`), les operations privilegiees (ecriture
disque, git, builds) passent par le broker cote daemon via routes HTTP
authentifiees (`/api/v1/factory/*`). Le bridge iframe reste reserve aux
apps sandboxees.

---

### S73 — Code Factory Templates

**Objectif produit :** Un systeme de templates qui genere un projet SBFB
complet et deployable en une commande.

**Valeur :** Pour tout developpeur : `sbfb create --template static-storage
--name my-app` produit un projet pret a deployer en 10 secondes.

**Resultat attendu :** 3-4 templates fonctionnels (static-minimal,
static-storage, react-vite, pyodide-notebook), CLI `sbfb create`, et
manifest SBFB.json v2.

#### Phases

**Phase A — SBFB.json v2 + validation**

Livrable : manifest enrichi avec retro-compatibilite.

Contenu :
- Spec SBFB.json v2 : schema_version, display_name, description,
  category, license, lang, bridge.methods, bridge.events, tech.type,
  tech.build_command, requirements.min_bridge_version,
  requirements.offline_capable, requirements.estimated_size_kb
- Parser/validateur dans deploy.rs (support v1 + v2)
- Migration des deux apps existantes (Explorer, Ideas Hub) vers v2
- Tests : v1 compat, v2 parse, v2 reject invalid

**Phase B — Template engine + 3 templates**

Livrable : generateur Rust + templates fonctionnels.

Contenu :
- Generateur Rust natif : substitution variables, copie bridge SDK,
  init repo git
- Template `static-minimal` (HTML pur + bridge, ~100 LOC)
- Template `static-storage` (storage CRUD pattern Ideas Hub, ~300 LOC)
- Template `react-vite` (React 19 + Vite + hook useSBFBBridge, ~400 LOC)
- `template.json` pour chaque template (id, version, variables,
  post_create hooks, content_hash BLAKE3)
- Tests : generation snapshot, fichiers attendus, SBFB.json valide

**Phase C — CLI `sbfb create`**

Livrable : sous-commande CLI fonctionnelle.

Contenu :
- Sous-commande dans nexus-shell-daemon ou binary separe
- Mode interactif (prompts) et non-interactif (flags)
- Telechargement template depuis repo Git (optionnel, pour templates
  externes)
- Verification content_hash BLAKE3 du template
- Tests : CLI happy path, invalid template, path traversal

**Phase D — Template verification + factory.template.lock**

Livrable : tracabilite de la generation.

Contenu :
- `factory.template.lock` genere dans chaque projet cree (hash template,
  version, date)
- `factory.provenance.json` (lineage creation)
- Tests : lock hash stable, provenance structure

#### Criteres de validation (gate S73)

- [ ] `sbfb create --template static-storage --name test-app` genere un
      projet deployable
- [ ] Les 3 templates generent des projets qui passent le deploy verifie
- [ ] SBFB.json v2 retro-compatible avec v1
- [ ] content_hash BLAKE3 verifie avant generation

#### Delta tests estime

- Rust : +10-15 (templates, CLI, validation)
- Vitest : +8-12 (composants UI si applicables)
- Total sortie estimee : ~1810

---

### S74 — Code Factory Broker/Sandbox

**Objectif produit :** Un broker local qui mediate les operations
privilegiees de la Factory avec audit et preview.

**Valeur :** Pour tout developpeur : voir un diff avant d'appliquer, un
preview avant de publier, et un audit trail de chaque action.

**Resultat attendu :** Page React `/factory` avec template selector,
diff viewer, preview iframe, publish gate checklist, et audit log JSONL.

#### Phases

**Phase A — Broker architecture + routes API**

Livrable : module factory_broker avec routes HTTP.

Contenu :
- Module `factory_broker` dans nexus-shell-daemon-core
- Routes HTTP : `/api/v1/factory/templates` (list),
  `/api/v1/factory/create` (generate), `/api/v1/factory/diff` (preview
  changes), `/api/v1/factory/apply` (apply changes),
  `/api/v1/factory/preview` (serve preview)
- Path allowlist + canonicalize (meme rigueur que `validate_zip_path()`
  dans blob_serve.rs)
- Audit log `factory.audit.jsonl`
- Tests : path traversal denied, routes auth required

**Phase B — Diff generation + review API**

Livrable : le broker calcule les diffs avant application.

Contenu :
- Diff engine : fichiers modifies/ajoutes/supprimes entre workspace
  actuel et modifications proposees
- Format diff : JSON structure (pas unified diff text) pour affichage
  React
- Route `/api/v1/factory/apply` applique seulement si diff
  precedemment genere + user_confirmed=true
- Tests : diff calcul correct, apply sans diff = refuse, concurrent
  apply protection

**Phase C — Review UI (page React /factory)**

Livrable : interface de creation et review dans le shell.

Contenu :
- Page `/factory` avec : template selector, variables form, diff
  viewer (fichiers expandables), approve/reject buttons
- Composant DiffViewer reutilisable
- Tests Vitest : DiffViewer renders, approve mutation

**Phase D — Preview sandbox + publish gate**

Livrable : preview et checklist avant publication.

Contenu :
- Preview : le broker zippe le workspace, le sert via blob-serve,
  affiche dans iframe (meme chemin que deploy normal)
- Publish gate checklist : index.html existe, SBFB.json v2 valide,
  bridge methods declarees existent, no secrets detected (regex scan),
  build OK (si build_command present)
- Route `/api/v1/factory/publish-check` retourne la checklist
- Tests : publish gate pass/fail, preview serve, missing index.html

#### Criteres de validation (gate S74)

- [ ] Projet cree via /factory deploie correctement
- [ ] Diff viewer affiche les modifications avant application
- [ ] Preview iframe fonctionne (meme sandbox que deploy normal)
- [ ] Publish gate rejette un projet sans index.html
- [ ] Path traversal impossible (tests canonicalize)
- [ ] Audit log JSONL contient toutes les actions

#### Delta tests estime

- Rust : +15-25 (broker, diff, preview, publish gate, path traversal)
- Vitest : +5-8 (DiffViewer, Factory page)
- Total sortie estimee : ~1843

---

### S75 — Babel Dogfood / Domain Packs

**Objectif produit :** Valider l'ensemble du protocole SBFB avec une
premiere app reelle : un lecteur multilingue P2P avec provenance.

**Valeur :** Pour le projet : preuve que la Factory, le bridge, le
deploy verifie, le storage P2P et le feed fonctionnent ensemble de bout
en bout. Pour les langues sous-dotees : un outil concret de
preservation linguistique.

**Resultat attendu :** Babel Reader deploye via Factory, avec fixtures
multilingues, storage P2P, provenance SLSA L1, visible dans le Browse
et trouvable via RRV.

#### Decision NLLB-200 (correction GPT 5.5 #10)

Le modele NLLB-200 complet ne doit pas bloquer S75. Le MVP Babel
fonctionne avec des traductions en fixtures (textes pre-traduits,
3 textes domaine public, ~5 langues). La traduction live via
`task_submit` -> worker NLLB-200 est un stretch goal.

La traduction via worker SBFB (Option A) est le chemin naturel : l'app
reste legere (juste UI + bridge), le modele NLLB-200 tourne sur GPU
cote worker (pattern Ollama existant). Ni Pyodide ni Transformers.js
dans l'iframe (trop lourd, pas d'IndexedDB, pas de connect-src).

#### Phases

**Phase A — Domain pack format + Babel pack**

Livrable : format domain pack defini, Babel pack cree.

Contenu :
- Spec format domain pack (template.json extended, fixtures/, config/)
- Babel domain pack : textes fixtures (3 textes domaine public, ~5
  langues), languages.json, task_types.json, storage_schema.json
- Integration dans `sbfb create --domain-pack babel`
- Tests : domain pack parse, fixtures loaded

**Phase B — Babel reader app via Factory**

Livrable : app Babel creee par la Factory et deployable.

Contenu :
- Creer babel-reader via `sbfb create --domain-pack babel`
- UI reader : liste textes, lecteur plein ecran, toggle langue
- Storage : progression lecture (bridge storage_get/set)
- Identity : affichage pubkey lecteur (bridge identity_pubkey)
- Tests : app deploie, bridge storage fonctionne

**Phase C — Bridge integration (storage + tasks)**

Livrable : Babel utilise le protocole complet.

Contenu :
- Storage structure : `texts/{id}`, `translations/{lang}/{id}`,
  `bookmarks/{pubkey}/{id}`, `reviews/{translationId}/{pubkey}`
- task_submit pour traduction (mock backend ou NLLB si pret)
- onEvent pour recevoir resultats
- onStorageUpdate pour sync P2P
- Tests : storage CRUD, task submit mock, sync poll

**Phase D — Deploy verifie + feed publication**

Livrable : Babel est une app pleinement verifiable sur le reseau.

Contenu :
- Deploy babel-reader via deploy-from-repo
- Provenance SLSA L1 auto-attestee generee et verifiable
- Feed entry ReleasePublished
- Verification E2E : deploy -> browse -> ouvrir -> lire
- Tests : E2E deploy, provenance verify, feed entry

**Phase E — Spec domain pack + second pack (stretch)**

Livrable : format formalise, potentiellement un second pack.

Contenu :
- Formaliser le format domain pack dans un doc spec
  (`docs/factory/DOMAIN_PACKS.md`)
- Second domain pack candidat : Repair Notebook (manuels reparation
  offline, plus simple que Babel)
- Tests : second pack genere une app deployable

#### Criteres de validation (gate S75)

- [ ] Babel Reader deploye via Factory, visible dans Browse
- [ ] Textes fixtures lisibles dans 3+ langues
- [ ] Storage P2P fonctionne (progression, bookmarks)
- [ ] Provenance verifiable par Proof Card
- [ ] Trouvable via RRV search
- [ ] Feed entry ReleasePublished pour Babel

#### Delta tests estime

- Rust : +5-10 (domain pack, protocol tests)
- Vitest : +5-10 (Babel app tests, si applicable)
- Babel tests dans son repo separe
- Total sortie estimee : ~1863

---

## Graphe de dependances

### Dependances explicites

```
S65 Contrat Public
  |---> S67 Gouvernance (vocabulaire de confiance = fondation)
  |---> S68 Proof Pack (la taxonomie definit quoi prouver)
  |---> S71 RRV Proof Cards (les niveaux S65 = le schema proof cards)

S66 Durabilite
  |---> S69 Pilote (daemon qui perd ses donnees = inutilisable)
  |---> S70 RRV LocalOnly (indexation necessite un feed qui survit)

S67 Gouvernance
  |---> S72 SearchManifest (qui trust un manifest ?)

S68 Proof Pack
  |---> S69 Pilote (le proof pack EST le livrable du pilote)

S69 Pilote
  |---> S70 (le feedback pilote informe le design RRV)

S70 RRV LocalOnly
  |---> S71 Proof Cards (les resultats RRV portent les proof labels)
  |---> S75 Babel (RRV trouve les composants Babel)

S71 RRV Proof Cards
  |---> S72 SearchManifest (les proof cards enrichissent les manifests)

S73 Templates
  |---> S74 Broker/Sandbox (les templates sont utilises dans le broker)
  |---> S75 Babel (le template Babel sort de Factory)
```

### Dependances cachees

| ID | Dependance | Impact |
|----|-----------|--------|
| D-HIDDEN-1 | S65 -> S66 | Le fix auth tier (S65) doit etre fait AVANT que le feed devienne persistent (S66). Sinon, des operations non-autorisees seraient persistees indefiniment. |
| D-HIDDEN-2 | S66 -> S72 | Les SearchManifests doivent survivre aux restarts. Si le store est volatil, les manifests recus sont perdus. |
| D-HIDDEN-3 | iroh 1.0 -> S66/S69 | iroh 1.0.0-rc.0 sorti le 2026-05-11. Si n0 arrete de maintenir 0.98, les bugs decouverts en pilote n'auront pas de fix upstream. Decision point Gate 1. |
| D-HIDDEN-4 | S65 -> S73 | Factory affichera des badges de confiance. Le vocabulaire S65 doit etre utilise, sinon regression wording. |
| D-HIDDEN-5 | S67 -> S72 | `CuratorVouched` (defini mais pas implemente avant S67) doit etre present pour que la gouvernance soit effective dans les manifests. |
| D-HIDDEN-6 | wasmtime CVEs -> S74 | Si WASM/wasmtime est utilise pour isolation, pin >= 43.0.1. Decision : NE PAS utiliser wasmtime. Isolation OS-level (processus + filesystem sandbox). |

### Graphe ASCII

```
                    S65 Contrat Public
                   / | \           \
                  /  |  \           \
         S66 Durabilite  \      S67 Gouvernance
           |     \        \         |
           |      \     S68 Proof Pack
           |       \        |
           |    S69 Pilote Ferme
           |        |
       S70 RRV Local  (feedback S69 informe)
           |
       S71 Proof Cards
           |
       S72 SearchManifest
                              S73 Templates (independant)
                                  |
                              S74 Broker/Sandbox
                                  |
                              S75 Babel Dogfood
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
| P2-PROVENANCE-404-BRIDGE | 2/3 | Phase A — UX verification |
| P2-COVERAGE-DEPLOY-E2E | 2/3 | Phase A — deploy roundtrip E2E |

### S69 — Absorbe naturellement

| Item | Compteur | Traitement |
|------|----------|------------|
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | Phase A — cross-node verification |
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
| T-NN+2 iframe Rust-wasm | Triggers non actifs (tract opset 19, ort wasm32-browser, gline-rs wasm-bindgen) |
| LT-5 redundancy persistence | Post-S75 sauf si pilote S69 l'exige (workers tiers) |
| LT-7 quorum E2E | Post-S75 sauf si pilote S69 l'exige |

### Trigger-dependent

| Item | Trigger | Estimation |
|------|---------|-----------|
| LT-2 Radicle | Push tag v1.0 vers origin | S66 ou S67 si tag pousse |

---

## Decisions gelees pour cette roadmap

Ces decisions ne doivent PAS etre re-debattees pendant les sprints
S65-S75. Elles sont le resultat de l'analyse croisee des 7 recherches
et des 2 rounds de review.

### D-GEL-1 : iroh 0.98 pour l'arc 1

Rester sur iroh 0.98 pour S65-S69. Evaluer l'upgrade vers iroh 1.0
(si 1.0 stable sorti) au moment de Gate 1. L'upgrade est un effort
~2-3 phases avec breaking changes (MSRV 1.91, PathWatcher ->
PathList, ConnectionInfo -> WeakConnectionHandle, reexports elimines).

### D-GEL-2 : OS sandbox pour Factory, pas wasmtime

Factory S74 utilise l'isolation OS-level (processus + filesystem
sandbox via path allowlist et canonicalize), pas wasmtime. wasmtime
a 12 CVEs avril 2026 (dont 2 Critical CVSS 9.0 sandbox escape). Le
broker execute des commandes OS (git, npm, cargo), pas du code WASM.

### D-GEL-3 : Pilote ferme (2-3 personnes)

Le pilote S69 est ferme. R-iroh-audit P0 rend un pilote public
irresponsable sans audit tiers de la pile iroh. Elargir le pilote
requiert soit un audit iroh, soit une evaluation de risque formelle.

### D-GEL-4 : Sequentiel, arc 2 avant arc 3

Les sprints sont sequentiels (solo maintainer). L'arc 2 (RRV) precede
l'arc 3 (Factory) sauf si le feedback pilote S69 rend l'arc 2 non
prioritaire. La parallelisation n'est possible que si un contributeur
externe prend un arc.

### D-GEL-5 : Tantivy avec fallback FTS5

Le moteur de recherche S70 est Tantivy. Si l'integration Tantivy
derive (MSRV, breaking change, complexite), fallback SQLite FTS5
temporaire. Le produit ne bloque pas sur le moteur.

### D-GEL-6 : Babel MVP fixtures

S75 Babel fonctionne avec des traductions en fixtures (textes
pre-traduits). Le modele NLLB-200 live via worker est un stretch goal.

### D-GEL-7 : Feed v2 batche

`CuratorVouched` (S67) et `SearchManifestPublished` (S72) batchent
dans un seul bump FEED_FORMAT_VERSION v1 -> v2. Le bump est fait en
S67. S72 ajoute un variant sans re-bumper.

### D-GEL-8 : Vocabulaire "source verifiable"

Les apps du reseau sont decrites comme "source verifiable, provenance
verifiable, commun logiciel anti-capture". Le terme "open source"
est reserve au code SBFB lui-meme (AGPL-3.0, licence OSI). Les apps
tierces ne correspondent pas necessairement a la definition OSI.

---

## Risques et mitigations

### Risques critiques

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R1 | iroh 0.98 non maintenu apres 1.0 stable | MOYENNE | HAUT | Decision point Gate 1. Fallback : stocker tout en SQLite, iroh-docs comme transport seulement. |
| R2 | Pilote revele problemes fondamentaux | MOYENNE (30%) | HAUT | S66 (durabilite) est specifiquement concu pour prevenir. Sprint fix dedie si > 5 bugs P0. |
| R3 | R-iroh-audit P0 | BASSE | CRITIQUE | Pilote ferme (pas public). Le threat model documente iroh comme "upstream trust assumption". |
| R4 | iroh-docs persistence buggy sous charge | FAIBLE (15%) | HAUT | Test `persistent_data_dir_reboots` existe. Fallback : SQLite pour tout, iroh-docs comme transport P2P. |

### Risques moderes

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R5 | Tantivy MSRV incompatible | FAIBLE | MOYEN | Fallback FTS5 temporaire (D-GEL-5) |
| R6 | NLLB-200 backend pas pret S75 | MOYENNE | MOYEN | Fixtures pre-traduites (D-GEL-6) |
| R7 | R-wasmtime-cve P0 | N/A pour S74 | N/A | Decision : pas de wasmtime (D-GEL-2) |
| R8 | NAT/firewall bloque P2P en pilote | HAUTE | MOYEN | Relais iroh, VPS Helsinki pre-deploy |
| R9 | Testeurs ne donnent pas de feedback | MOYENNE | MOYEN | Guide structure + rappel apres 1 semaine |

### Risques faibles

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R10 | SBOM generation echoue | MOYENNE | FAIBLE | Fallback cargo-deny output seul |
| R11 | Reproductibilite bit-a-bit impossible (Rust) | HAUTE | FAIBLE | Documenter comme best-effort |
| R12 | SBFB.json v2 casse apps existantes | FAIBLE | MOYEN | schema_version absent/1 = ancien format, test compat |
| R13 | Path traversal dans Factory broker | FAIBLE | CRITIQUE | canonicalize + prefix check + tests traversal obligatoires |

### Zones rouges heritees

| Zone | Etat | Impact S65-S75 |
|------|------|---------------|
| R-iroh-audit P0 | Pas d'audit tiers publie | Pilote ferme seulement |
| R-wasmtime-cve P0 | 12 CVEs avril 2026, 2 Critical | Pas de wasmtime dans Factory |
| R-libcrux-hax P2 | Post-quantum hors scope | AUCUN impact S65-S75 |
| R-pyodide-escape | Iframe sandbox mitige | Acceptable pour S75 (meme surface que tout contenu web) |

---

## Calendrier previsionnel

| Semaine | Sprint | Arc | Theme | Risque |
|---------|--------|-----|-------|--------|
| S1-S2 (mai-juin 2026) | S65 | 1 | Contrat Public + 8 carry items | 2/5 |
| S3-S4 | S66 | 1 | Durabilite (persistence + crash recovery) | 4/5 |
| S5-S6 | S67 | 1 | Gouvernance (CuratorVouched + UI) | 3/5 |
| S7-S8 | S68 | 1 | Proof Pack (release pipeline + evidence) | 2/5 |
| S9-S11 | S69 | 1 | Pilote Ferme (deploy + feedback) | 5/5 |
| -- | **GATE 1** | -- | Go/no-go Arc 2 + decision iroh 0.98/1.0 | -- |
| S12-S13 | S70 | 2 | RRV LocalOnly (Tantivy index) | 3/5 |
| S14-S15 | S71 | 2 | Proof Cards (resultats enrichis) | 2/5 |
| S16-S17 | S72 | 2 | SearchManifest (P2P discovery) | 4/5 |
| -- | **GATE 2** | -- | Go/no-go Arc 3 | -- |
| S18-S19 | S73 | 3 | Templates (scaffolding) | 2/5 |
| S20-S21 | S74 | 3 | Broker/Sandbox (isolation OS) | 4/5 |
| S22-S24 | S75 | 3 | Babel Dogfood (premiere app) | 3/5 |

**Total : ~24 semaines = ~6 mois (mai 2026 -> novembre 2026).**

**Contingence :** +2-4 semaines pour fixes pilote, iroh upgrade
eventuel, ou gates echouees. Budget total realiste : 26-28 semaines.

---

## Tests delta projete cumule

| Sprint | Arc | Rust entree | Rust sortie | Vitest sortie | Total sortie |
|--------|-----|-------------|-------------|---------------|-------------|
| S64 (base) | -- | -- | 1326 | 265 | 1597 |
| S65 | 1 | 1326 | ~1338 | ~275 | ~1619 |
| S66 | 1 | ~1338 | ~1363 | ~275 | ~1644 |
| S67 | 1 | ~1363 | ~1378 | ~283 | ~1667 |
| S68 | 1 | ~1378 | ~1390 | ~288 | ~1684 |
| S69 | 1 | ~1390 | ~1400 | ~303 | ~1709 |
| S70 | 2 | ~1400 | ~1418 | ~315 | ~1739 |
| S71 | 2 | ~1418 | ~1426 | ~323 | ~1755 |
| S72 | 2 | ~1426 | ~1446 | ~331 | ~1783 |
| S73 | 3 | ~1446 | ~1461 | ~343 | ~1810 |
| S74 | 3 | ~1461 | ~1486 | ~351 | ~1843 |
| S75 | 3 | ~1486 | ~1496 | ~361 | ~1863 |

**Projection S75 : ~1863 tests totaux** (vs 1597 actuels, **+266 net**
sur 11 sprints, ~24 tests/sprint en moyenne).

La repartition est coherente avec l'historique (S64 : +21 Rust sur 6
phases). Les sprints techniques lourds (S66, S72, S74) produisent plus
de tests Rust. Les sprints UX (S65, S71, S75) produisent plus de tests
Vitest.

---

*Document canon. Toute modification ulterieure doit etre tracee par
commit avec reference au sprint qui l'exige.*
