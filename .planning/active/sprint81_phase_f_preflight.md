# Sprint 81 Phase F — Préflight G8 (Workflow ultracode) — migration on-disk redb 2→4 (docs) + ouverture blobs sous 0.103

> **Verdict : PLAN-ADAPT.** Le plan Phase F prévoit lui-même la bascule (« le préflight tranche
> (PLAN-ADAPT) », assouplissement C4/C5). Cinq scans + vérifications adversariales convergent, et
> **j'ai re-vérifié la preuve cardinale moi-même** (hexdump des en-têtes redb). La **prémisse routée
> Phase D est empiriquement FAUSSE sur son point le plus lourd** : `blobs.db` n'est **PAS** un store
> redb-v2 illisible qui hard-fail. **Les DEUX fichiers redb du store VPS réel sont on-disk
> `FILE_FORMAT_VERSION3`** (octet `0x03` aux offsets 64 ET 192, magic `redb`), écrits par redb 2.6.3
> (qui écrit déjà le format v3). Conséquences qui **simplifient fortement** Phase F, **sans toucher
> aucun Day-0** :
>
> 1. **[REFUTE Phase D] `blobs.db` s'ouvre PROPREMENT sous iroh-blobs 0.103, SANS migration, 0 perte
>    de pins.** Le hard-fail `UpgradeRequired` (`meta.rs:518-523`) ne fire **QUE** pour v1/v2 ;
>    notre fichier est v3 → `Database::create` réussit. Les tables iroh-blobs sont **non-tuple**
>    (`Hash`/`EntryState`/`Tag`/`HashAndFormat`) → le changement de type-tag redb 3.0 (qui casse les
>    tuples variable-width) **ne les touche pas** → aucun `TableTypeMismatch`. **La décision pendante
>    « discard+refetch (perte M18) vs shim » est SANS OBJET.** L'inventaire « pins re-fetchables
>    ailleurs ? » et toute tolérance-wipe blobs tombent. **Reste un gate EMPIRIQUE** (voir #6).
> 2. **[docs migre, NON in-place] `docs.redb` migre via `TableTypeMismatch`** (vieux type-tags tuples
>    écrits par redb 2.x, au format fichier v3), **automatiquement à l'ouverture**, **temp-file +
>    swap** (`migrate_redb_v2_tuples.rs:99/166/167`), one-way, avec `.backup-redb-v2-tuples` conservé.
>    **La clause plan « neutraliser le self-heal destructeur SI chemin in-place » tombe** : il n'y a
>    **pas** d'in-place (le chemin nominal préserve l'original jusqu'au swap final).
> 3. **[FAIT kickoff FAUX] La feature `redb-v2-migration` EXISTE et est DÉFAUT** (`iroh-docs
>    Cargo.toml:37-42,48`). Le kickoff C4/C5 « cette feature N'EXISTE PAS » est **factuellement faux** ;
>    la note Phase B et le root `Cargo.toml:36-42` sont **corrects**. La config SBFB est déjà bonne
>    (défauts ON, `nexus-core-rs/Cargo.toml:20` `{ workspace = true }`) → **0 action code**, mais
>    **tripwire recommandé** (un futur `default-features=false` désactiverait la migration
>    SILENCIEUSEMENT à la compile, hard-fail runtime).
> 4. **[ordre de boot — protection INVERSÉE] L'ordre `blobs`-avant-`docs` (`node.rs:419` avant `:432`)
>    est confirmé, mais son motif protecteur a DISPARU.** Puisque `blobs.db` v3 s'ouvre proprement, un
>    boot naïf **n'avorte plus** au blobs → il **PROCÈDE** à `Docs::persistent` → **déclenche la
>    migration docs one-way**. **La règle bloquante « aucun boot sur store réel avant F PASS » est
>    désormais l'UNIQUE garde-fou** contre une migration accidentelle irréversible du store live.
> 5. **[fixture] Deux niveaux, 0 dep neuve.** (i) **Empirique** sur une **COPIE FRAÎCHE** du tarball
>    gitignoré `data/vps-store-098/` (secrets → jamais committé) — **ré-extraire à chaque run** (la
>    migration mute la copie via swap → 2e run vacuant). (ii) **Hermétique committé DEP-FREE** : forger
>    un docs store synthétique v3 à type-tags tuples legacy via **`redb_v3` (3.1.3) déjà transitif**
>    (`Legacy<T>`, modèle upstream `fs.rs:1207`) → **aucun dev-dep redb-2.6.3** (qui, lui,
>    collisionnerait le flip `deny.toml:107` Phase G).
> 6. **[résidu réel] Fenêtre de crash `migrate_redb_v2_tuples.rs:166↔167`** (rename source→backup PUIS
>    persist temp→source) : un crash entre les deux laisse `docs.redb` ABSENT → au reboot
>    `Database::create` crée un store VIDE qui s'ouvre proprement → `open_doc(ns M8)` → « Replica not
>    found » → **RECREATE SILENCIEUX** (warn-only `runtime.rs:2632`), **NON capté par le fail-loud A2**
>    (c'est un `NotFound` légitime, pas une erreur docs). **C'est le SEUL vrai résidu de durabilité de
>    Phase F** — D3 cond.7 n'est **pas** garanti pour cet entrelacement. Mitigation : tar snapshot +
>    (optionnel, root-cause) refuser le recreate si le sibling `.backup-redb-v2-tuples` existe.
>
> **iroh strictement seul, 0 bump wire, 0 dep runtime neuve** (`redb_v3` dev-only = la MÊME 3.1.3 déjà
> lockée → 0 delta lock, 0 collision Phase G), toolchain 1.94, verrous S74/S75 intacts, duress
> orthogonal (migration = couche fichier ; gates = couche broadcast). **Ce n'est pas un
> DESIGN-CONFLICT** : Phase F ne contredit aucune décision gelée Day-0 — elle **corrige deux prémisses
> internes erronées** (découverte Phase D + faits kickoff C4/C5) avec de la preuve OSS vérifiée, ce qui
> est **exactement** la définition d'un PLAN-ADAPT. Le plan lui-même délègue la décision au préflight.
>
> G8 : 5 scans (S1a-oss lecture upstream vendored / S1b-deps graphe+CVE / S2-decisions historique /
> S3-threat couverture menace / S4-wire invariants persistence) + 5 vérifications adversariales. Bilan :
> **1 REFUTED matériel** (S2-F3 « blobs in-place impossible » — réfuté par les octets), **7 PARTIAL
> requalifiés** (impacts sur-estimés effondrés par la découverte v3, API de test corrigée, résidu
> crash-window surfacé), **le reste CONFIRMED**. La preuve cardinale (bytes v3) est **re-vérifiée par
> la synthèse elle-même**, pas seulement héritée des scans.

---

## 1. Contexte + gates calendaires

**Phase F (plan `sprint81_plan.md:239-268`)** : prouver **HORS-PROD** que `docs.redb` + `blobs` survivent
à la migration redb 2→4 ; neutraliser le self-heal destructeur **si** chemin in-place ; fixture de
migration + test d'ouverture blobs + garde self-heal vérifiée + inventaire pins + parse
DocTicket/BlobTicket post-migration. **Gate : AUCUNE migration LIVE** — uniquement sur COPIE. Delta
attendu +3..5 Rust. Assouplissement PO C4/C5 (« personne sur le réseau ») : chemin le plus simple
autorisé (wipe toléré si l'in-place résiste), le préflight tranche (PLAN-ADAPT).

**Gates C8 (rappel E2, `sprint81_kickoff.md:76-82`)** : 25/08 = si Phase F pas PASS, basculer la flotte
plan B. Aujourd'hui **2026-07-05** → fenêtre ouverte, aucune pression bloquante sur F elle-même.
Le corps S81 restant après E2 : **F..K**. E2 (zéro-n0) est LIVE + PASS (`a085853`).

**Note process (S2-F8 CONFIRMED)** : README §6.2.1 = escalade à **3** reports cross-sprint
(`README.md:1429-1431`). Les carries « routés F » (Phase C review re-parse ticket non-fatal, Phase D
review dualité redb / durabilité pins / anchors graceful-degrade) sont **INTRA-sprint** phase→phase →
**aucun MANDATORY** déclenché. Toute la surface redb-migration est neuve en S81 (forcing-function EOL
n0 30/09). Ne pas sur-scoper F avec des items non-F.

---

## 2. La preuve cardinale re-vérifiée par la synthèse (hexdump on-disk)

Trois adversaires ont hexdumpé le store ; **je l'ai re-fait moi-même** (extraction du tarball gitignoré
vers scratchpad, `od` sur les en-têtes redb). Résultat identique et non-ambigu :

| Fichier | taille | magic@0-3 | god@9 | ver@64 | ver@192 | interprétation |
|---|---|---|---|---|---|---|
| `iroh/docs.redb` | 634880 | `72 65 64 62` (`redb`) | `0x02` | `0x03` | `0x03` | **FILE_FORMAT_VERSION3**, primary=slot0, `RECOVERY_REQUIRED` set |
| `iroh/blobs/blobs.db` | 675840 | `72 65 64 62` (`redb`) | `0x03` | `0x03` | `0x03` | **FILE_FORMAT_VERSION3**, primary=slot1, `PRIMARY_BIT`+`RECOVERY_REQUIRED` set |

**Conséquences directes, chaînées au code upstream que j'ai lu :**

- **redb 2.6.3 écrit déjà le format on-disk v3** → « crate redb 2.x » ≠ « format fichier v2 ». C'est la
  confusion qui a inversé la conclusion blobs dans la découverte Phase D et les scans S1b/S2 d'origine.
- **`UpgradeRequired` ne fire QUE pour v1/v2** : `redb-4.1.0` `header.rs` accepte v3 nativement ; le
  hard-fail iroh-blobs `meta.rs:518-523` (`Err(DatabaseError::UpgradeRequired(v))`) et le hard-fail
  iroh-docs `fs.rs:95-97` (`open_database`) **ne peuvent pas se déclencher** sur ce store.
- **La vraie incompatibilité est l'ENCODAGE DES TUPLES variable-width** (`migrate_redb_v2_tuples.rs:1-7`
  doc-comment : « redb 3.0 changed the on-disk type tag for variable-width tuples » ; touche
  `records-1`/`records-by-key-1`/`latest-by-author-1`). Les tables **iroh-blobs sont non-tuple** → pas
  de `TableTypeMismatch` → blobs ouvre proprement. Les tables **iroh-docs tuples** → `TableTypeMismatch`
  → migration.
- **Les deux fichiers ont `recovery_required` set** (copie prise d'un daemon vivant, non fermé
  proprement) → l'ouverture déclenche un **repair redb** AVANT/PENDANT la migration. Ce chemin
  (repair + migration conjoints, sur le vrai volume/namespaces) n'est couvert par **aucun** test
  synthétique upstream (qui est CLEAN) → **impose le gate empirique sur la copie DIRTY réelle**.

**Layout complet du tarball** (56 entrées, préfixe `nexus-grid/`, perms 0600) : `shell-daemon/node_key`,
`shell-daemon/coordinator.db` + `coordinator.db-wal` (4.1 Mo) + `coordinator.db-shm`,
`iroh/docs.redb`, `iroh/blobs/blobs.db` + `iroh/blobs/data/*.data|.obao4|.sizes4`,
`iroh/default-author`, `anchors.json`, `.sbfb/directory_revision.json`, `subscriptions.json`,
`running.json`, `local-worker/data/allowlist.sqlite3` + `worker.key`, `local-worker/sbfb/consent.json`.

---

## 3. Constats par scan (evidence fichier:ligne vérifiée) + arbitrage adversarial

### S1a-oss (lecture upstream vendored + octets on-disk)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| 1 | `blobs.db` PAS redb-v2 illisible — ouvre sous 0.103 sans migration ni perte de pins (**REFUTE Phase D**) | CONFIRMED | **RETENU** (re-vérifié bytes v3 par la synthèse). Résidu = gate empirique runtime `FsStore::load` sur la copie dirty. |
| 2 | `docs.redb` migre via `TableTypeMismatch`, temp+swap NON in-place, one-way, backup conservé | CONFIRMED | **RETENU** (`migrate_redb_v2_tuples.rs:99/107/108/154/166/167` lus). |
| 3 | Fenêtre de crash `:166↔:167` (source absent → reboot store vide) | PARTIAL (caveat Windows-rename INEXACT ; vrai risque = Linux `rename(2)` clobber au retry) | **REQUALIFIÉ** : crash-window réel (résidu central, cf. §6) ; caveat corrigé → cible **VPS Linux**. |
| 4 | Feature `redb-v2-migration` EXISTE + défaut ; kickoff C4/C5 faux | CONFIRMED | **RETENU** (`Cargo.toml:37-42,48` lus). Tripwire → carry G. |
| 5 | Test upstream `fs.rs:1207` = modèle CLEAN ; store réel DIRTY (`recovery_required`) | CONFIRMED | **RETENU** : copie réelle = fixture primaire (repair+migration) ; synthétique = unit déterministe. |
| 6 | Self-heal recreate non déclenché si migration préserve `namespaces-2` | CONFIRMED (nominal) | **REQUALIFIÉ PARTIAL** : vrai pour le chemin nominal, **FAUX pour l'entrelacement crash-window** (§6). |
| 7 | Ordre boot `blobs`-avant-`docs` déjà en place ; motif « avorter avant migration docs » moot | CONFIRMED | **REQUALIFIÉ** : ordre confirmé (`node.rs:419`/`:432`), mais protection **INVERSÉE** (§5(d)). |
| 8 | « sbfb-ides » = coquille pour « sbfb-ideas » ; store contient aussi « sbfb-feed » ; ce sont des clés M8 | CONFIRMED | **RETENU** : fixture couvre `sbfb-ideas` ET `sbfb-feed` (2 arms). |
| 9 | DocTicket/BlobTicket = strings hors-redb (SQLite + anchors.json) → enjeu wire (Phase E), pas migration | CONFIRMED | **RETENU** : assert parse = vérif bon-marché, classée wire, pas preuve de migration. |

### S1b-deps (graphe + CVE)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| F1 | redb ×2 (3.1.3 + 4.1.0) = feature défaut `redb-v2-migration` ; feature EXISTE | CONFIRMED (réserve : la migration ÉCRIT via `redb_v3`, redb 4.1 ne fait que LIRE le résultat) | **RETENU-corrigé** : `Cargo.lock:7028`=3.1.3 (lecteur-legacy) + `:7037`=4.1.0 (courant), BY-DESIGN. |
| F2 | iroh-blobs 0.103 ne déclare aucun `redb_v3` → « blobs.db redb-v2 ILLISIBLE hard-fail » | dep-fact CONFIRMED ; **IMPACT REFUTED** (blobs v3 ouvre) | **REQUALIFIÉ** : asymétrie dep réelle (docs a un lecteur-legacy, blobs non), mais **illisibilité FAUSSE** (blobs est v3, non-tuple). |
| F3 | `hickory-proto 0.24.4` RUSTSEC-2026-0119 présent mais ISOLÉ à notre resolver, non-P0 | CONFIRMED | **RETENU** → carry G (pré-existant, hors scope F ; bump 0.24→0.26 en G). |
| F4 | RUSTSEC-2026-0118 (plage 0.25.x) ne nous touche pas | CONFIRMED | **RETENU** (verif négative). |
| F5 | Aucune version plus neuve ; 0 advisory redb/iroh | PARTIAL (externe non-vérifiable offline ; le risque F = **intégrité données**, pas CVE) | **RETENU-nuance** : terrain CVE propre côté redb/iroh ; le risque migration n'est pas un CVE. |

### S2-decisions (historique)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| F1 | Feature EXISTE + défaut, kickoff C4-bis faux | CONFIRMED | **RETENU**. |
| F2 | Fixture `data/` non git-trackée → tension T1.3 hermétique BLOQUANT | PARTIAL (tracking réel ; **« impossible de forger in-process » REFUTÉ** — `redb_v3` Legacy déjà transitif) | **REQUALIFIÉ** : voir §5(b) — fixture synthétique dep-free forgeable via `redb_v3`. |
| F3 | **DESIGN-CONFLICT : blobs in-place IMPOSSIBLE (hard-fail sans shim)** | **REFUTED** (octets : `blobs.db@64=0x03` v3 ; `UpgradeRequired` fire seulement v1/v2 ; tables non-tuple) | **ÉCARTÉ AVEC TRACE** : blobs ouvre in-place ; le « discard forcé » tombe. |
| F4 | « sbfb-ideas » seul identifiant réel (clé M8 + app id) | CONFIRMED | **RETENU**. |
| F5 | Line-drift refs self-heal (~90-100 l.) + AnchorLocator dans le mauvais crate | CONFIRMED | **RETENU** → carry K. Réels : `runtime.rs:2562/2602/2606` (storage), `:2693/:2722` (feed) ; `AnchorLocator` = `nexus-shell-daemon-core/src/iroh_runtime.rs:260`. |
| F6 | Migration NON strictement in-place (temp+backup) | CONFIRMED | **RETENU** ; redb 4.1 a RETIRÉ le shim Legacy → `redb_v3` 3.1.3 = lecteur-legacy (explique redb ×2). |
| F7 | Inventaire pins : M18 SQLite survit, `.data` locaux présents, TAG skip-GC perdu au discard | PARTIAL (discard MOOT car blobs ouvre → TAG préservé) | **REQUALIFIÉ** : survie SQLite/`.data` vraie mais secondaire ; **pas de discard** → pas de perte. |
| F8 | §6.2.1 = 3 reports ; carries F intra-sprint → aucun MANDATORY | CONFIRMED | **RETENU**. |
| F9 | État fixture + node_key préservé | PARTIAL (label « v2 » REFUTÉ par octets ; listing CONFIRMÉ) | **REQUALIFIÉ** : listing exact ; les deux DB sont v3, pas v2. |

### S3-threat (couverture menace)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| 1 | Aucune couverture threat-model de la migration/durabilité on-disk | CONFIRMED | **RETENU** → carry G (le sprint réserve les édits THREAT_MODEL à G). |
| 2 | Wipe-blobs = perte M18 ; §15 = intégrité pas durabilité | PARTIAL (contenu dans `iroh/blobs/data/*.data` survit ; + wipe MOOT) | **REQUALIFIÉ** : wipe sans objet ; angle « §15 = intégrité, pas durabilité » retenu (doc G). |
| 3 | Fuite secrets fixture ; `*.redb` insuffisant pour secrets sans extension | CONFIRMED (risque CONDITIONNEL à une relocation hors `data/`) | **RETENU** → fixture synthétique pour CI ; copie réelle reste sous `data/` (gitignoré). |
| 4 | Discriminateur « Replica not found » = contrat string fragile, stable en 0.101 | CONFIRMED | **RETENU** → tripwire test (K). |
| 5 | Self-heal non déclenchable en fenêtre de migration | **PARTIAL/REFUTED en partie** (crash-window `:166↔:167` → store vide → NotFound → recreate silencieux, NON capté par A2) | **REQUALIFIÉ — RÉSIDU CENTRAL** (§6). |
| 6 | Ordre boot tripwire one-shot | CONFIRMED | **RETENU** mais moot (blobs no-op). |
| 7 | Gates duress orthogonaux à la validation sur COPIE | CONFIRMED | **RETENU** : F offline/Normal jamais affecté par le duress. |
| 8 | « sbfb-ideas » tranché au code (clé M8, pas namespace nommé) | CONFIRMED | **RETENU**. |

### S4-wire (invariants persistence)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| 1 | 0-bump wire par construction (F = on-disk, jamais wire) | CONFIRMED | **RETENU** (`runtime.rs` boot = 0 constante wire). |
| 2 | Feature défaut-ON | CONFIRMED | **RETENU**. |
| 3 | Migration touche 2 fichiers ; identité/DocTicket/keep_online/invites survivent par copie | CONFIRMED | **RETENU** : node_key hors redb → node_id inchangé (H) garanti structurellement. |
| 4 | Fixture gitignorée ; committée collisionne Phase G (dev-dep redb-2.6.3) | CONFIRMED | **RETENU-nuance** : la collision n'existe QUE si redb-2.6.3 ; `redb_v3` 3.1.3 (déjà locké) l'évite. |
| 5 | Ordre boot → 2 tests séparés | PARTIAL (**recette API fausse** : `Store::persistent(fichier)` pas `Docs::persistent(dir)`) | **REQUALIFIÉ** : API corrigée §5(e). |
| 6 | Migration atomique + auto-backup + fenêtre crash | CONFIRMED | **RETENU** (§6). |
| 7 | Self-heal = TEST pas code neuf (A2 backstop) | CONFIRMED (nominal) | **RETENU** + résidu crash-window (§6). |
| 8 | « sbfb-ideas » | CONFIRMED | **RETENU**. |
| 9 | Aucune nouvelle frontière §6.12 ; F valide des frontières on-disk existantes | CONFIRMED | **RETENU** : Track K docs-contrat non déclenché par F. |

---

## 4. Les 5 questions structurantes — tranchées

### (a) Chemin blobs : discard+refetch vs shim → **NI L'UN NI L'AUTRE (ouverture in-place gratuite)**

`blobs.db` est `FILE_FORMAT_VERSION3` (octets §2), les tables iroh-blobs sont **non-tuple**
(`Hash`/`EntryState`/`Tag`/`HashAndFormat`), le schéma est **byte-identique 0.100→0.103**, `bao-tree`
reste `0.16` (format `.data`/`.obao4`/`.sizes4` stable). Donc `FsStore::load` (`node.rs:419`) **ouvre
proprement sans aucune migration** → **0 wipe, 0 refetch, 0 perte de pins keep-online M18, 0 shim.** La
décision pendante est **sans objet** ; supprimer de F l'inventaire « pins re-fetchables ailleurs ? » et
toute tolérance-wipe. **Réserve non-levable par lecture** : le comportement runtime de `FsStore::load`
sur le store DIRTY (repair redb + scan/cohérence `blobs.db`↔`data/`) est un **gate EMPIRIQUE** — il doit
être **exécuté sur la copie**, jamais conclu par revue de code.

### (b) Forger/committer la fixture (secrets VPS) → **deux niveaux, 0 dep neuve**

- **PRIMAIRE — empirique sur COPIE réelle (non-committée)** : le tarball `data/vps-store-098/…tar.gz`
  est **gitignoré** (`.gitignore:4 data/` ; ceinture `*.db:5`, `*.redb:11`) et contient des **secrets**
  (`node_key`, `.sbfb/auth_token` [absent de CE tar mais classe], `worker.key`, `default-author`,
  `NamespaceSecret` write-cap dans `docs.redb`) → **JAMAIS committé**. Le test s'exécute sur une **COPIE
  FRAÎCHE ré-extraite dans un tmpdir hors-repo à CHAQUE run**. Hygiène **load-bearing** : la migration
  MUTE la copie (rename+persist swap) et laisse `.backup-redb-v2-tuples` → un 2e run ouvrirait un store
  déjà-v4 = **test silencieusement vacuant**. Ré-extraction obligatoire.
- **HERMÉTIQUE COMMITTÉ — dep-free (candidat T1.3 BLOQUANT)** : forger **in-process** un `docs.redb`
  synthétique v3 à **type-tags tuples legacy** via **`redb_v3` (3.1.3), DÉJÀ dépendance transitive** (via
  la feature `redb-v2-migration` → `dep:redb_v3`) qui expose les wrappers `Legacy<T>`. Modèle **upstream
  direct** : `iroh-docs-0.101.0/src/store/fs.rs:1207` `test_migration_redb_v2_tuples` (crée un store
  `redb_v3`+`Legacy`, assert redb4 rejette en `TableTypeMismatch`, `Store::persistent` migre, assert
  `.backup-redb-v2-tuples`, vérifie survie `records`/`latest`/`by_key`). **Aucun dev-dep `redb = 2.6.3`**
  (qui ajouterait un 3e redb au lock et **collisionnerait** le flip `deny.toml:107 multiple-versions
  warn→deny` de Phase G). À vérifier au code : accéder à `redb_v3` en `[dev-dependencies]` de
  `nexus-core-rs` sous `{ package = "redb", version = "3.1" }` = **la même 3.1.3 déjà lockée** → **0
  delta lock, 0 collision G**. Si l'accès s'avère non-trivial → repli sur PRIMAIRE seul + T1.3 dégradé en
  test d'intégration env-gaté (`#[ignore]` par défaut, run si la copie est présente) — mais alors ne pas
  claimer « BLOQUANT hermétique ».

### (c) sbfb-ides vs sbfb-ideas → **`sbfb-ideas` (avec 'a'), tranché AU CODE**

`runtime.rs:694 let replicated_apps: &[&str] = &["sbfb-ideas"];` + tous les tests A2
(`:4334/:4340/:4350/:4411/:4419/:4433`) + `examples/sbfb-ideas/`. « sbfb-ides » = **coquille
planning-prose** (0 occurrence code). Ce sont des **clés M8 `app_name`** de la table
`storage_namespaces` (coordinator.db SQLite) → `NamespaceId` (32 octets) + `DocTicket` ; **PAS** des
namespaces iroh-docs nommés. Le store contient **DEUX** clés M8 : `sbfb-ideas` ET `sbfb-feed` → la
fixture couvre **les deux arms** (`boot_storage_namespace` + `boot_feed_namespace`).

### (d) Ordre de boot protecteur → **confirmé mais protection INVERSÉE**

`node.rs:419 FsStore::load(&blobs_dir)` s'exécute **avant** `node.rs:432 Docs::persistent(path)` +
`:437 .spawn(...)`. **MAIS** : puisque `blobs.db` v3 ouvre proprement (question (a)), un `create_node`
naïf **ne s'avorte plus au blobs** → il **atteint** `Docs::persistent` → **migre docs one-way**. Le
« tripwire qui avorte avant la migration docs irréversible » **n'existe plus**. Deux conséquences :
1. La **règle bloquante « aucun boot daemon/rig sur store réel avant F PASS »** est l'**UNIQUE**
   garde-fou → à **enforcer durement**.
2. Les tests ouvrent `docs.redb` (isolé, `Store::persistent`) et `blobs.db` (isolé, `FsStore::load`)
   **SÉPARÉMENT** sur des **copies fraîches** — **jamais** un `create_node` complet sur le store réel.
Un test de régression épinglant l'ordre reste utile (documentaire : blobs = no-op open), non prioritaire.

### (e) Périmètre exact des livrables + delta tests → **reframé (simplifié)**

**Livrables :**
1. **Validation EMPIRIQUE sur COPIE fraîche** de `data/vps-store-098/` (non-committée), asserts :
   - **blobs** : `FsStore::load(copy/iroh/blobs)` ouvre **SANS migration** ; aucun artefact `.backup`
     blobs créé ; `iroh/blobs/data/*.data|.obao4|.sizes4` intacts.
   - **docs** : `iroh_docs::store::Store::persistent(copy/iroh/docs.redb)` migre ; `docs.redb.backup-
     redb-v2-tuples` apparaît ; `open_doc(ns_id de M8 sbfb-ideas)` = Ok(Some) **ET** `open_doc(sbfb-feed)`
     = Ok(Some), même `namespace_id` (pas de recreate, pas de warn « recreating ») ; records survivent.
   - **coordinator.db** SQLite ouvre (WAL-replay ; le `-wal` 4.1 Mo porte l'essentiel de l'état M18) ;
     M8 `doc_ticket` / M18 `keep_online` / M19 invites intacts.
   - **node_key** byte-identique pre/post (structural — hors redb → **node_id inchangé pour H**).
   - **anchors.json** BlobTicket re-parse ; `default-author`, `directory_revision.json` (floor
     anti-rollback S75), `subscriptions.json` (set gossip, lié au carry « subscribe-à-chaud ») présents.
2. **Test hermétique committé DEP-FREE** (candidat T1.3) : docs synthétique v3-tuple-legacy via `redb_v3`
   → `TableTypeMismatch` → `Store::persistent` migre → `.backup` présent → survie lignes (modèle
   `fs.rs:1207`).
3. **Test hermétique committé** : ouverture blobs no-op — le plus simple = round-trip d'un store
   iroh-blobs 0.103 frais (prouve le chemin de schéma) ; l'ouverture-no-op du VRAI store dirty reste le
   gate empirique (#1).
4. **Assertion self-heal NON déclenché** (pas de code neuf) : la migration préserve `namespaces-2` →
   `open_doc` = Some → arm recreate non entré ; backstoppé par A2 (`runtime.rs:2602-2606`/`:2722-2726`).
5. **(Optionnel, root-cause) Garde crash-window** : refuser le recreate si le sibling
   `docs.redb.backup-redb-v2-tuples` existe (ferme le silent-loss §6). Dans la **doctrine A2** (fail-loud
   / refuse-silent-recreate). À trancher au code — région sensible ; sinon documenter + tar.
6. **Doc rollback** (runbook / body) : one-way confirmé → rollback = restore tar **OU** renommer
   `docs.redb.backup-redb-v2-tuples`. Documenter temp-file+backup, coût disque ~2x pendant la migration
   docs, résiduels `docs.db.migrate*` + `.backup*` à nettoyer, caveat Linux `rename(2)` clobber au retry.

**API de test (correction S4-adv) :** l'entrypoint isolé docs = **`iroh_docs::store::Store::persistent(
dir/docs.redb)`** (`Store` est `pub use fs::Store`, `store.rs:13` ; `pub fn persistent(path)`
`fs.rs:127` ; prend le **FICHIER**, déclenche la migration **sans endpoint**). **Ne PAS** utiliser
`Docs::persistent(dir)` (`protocol.rs:40`) qui prend le **RÉPERTOIRE** iroh, joint `docs.redb` en interne
(`:110`) et rend un `Builder` exigeant `.spawn(endpoint, blobs, gossip)`.

**Delta tests réaliste : +3..5 Rust** — ~2-3 hermétiques committés (docs synthétique + blobs no-op/round-
trip + self-heal-non-déclenché ; +1 si garde crash-window ajoutée) + ~1 intégration sur copie réelle
(env-gaté / non-committé). **0 bump wire, 0 dep runtime neuve** (`redb_v3` dev-only = 3.1.3 déjà lockée).

---

## 5. Contraintes code load-bearing (à honorer)

1. **NE PAS désactiver `default-features` sur iroh-docs** (`nexus-core-rs/Cargo.toml:20` = `{ workspace =
   true }`, OK aujourd'hui). Un `default-features = false` retirerait `redb-v2-migration` → hard-fail
   runtime sur le store v2-tuples. Tripwire nommé recommandé (assert que la feature reste dans les défauts
   d'iroh-docs) — analogue au pkarr tripwire E, carry G.
2. **Fixture = ré-extraction fraîche par run** (la migration est destructive-in-place-via-swap sur la
   copie). Copier dans un tmpdir per-test ou `tempfile`.
3. **Aucun `create_node` complet sur le store réel** — tests isolés `Store::persistent` (docs) +
   `FsStore::load` (blobs) séparés, sur copie. La règle bloquante no-boot est l'unique garde-fou (§4(d)).
4. **Self-heal = ASSERT, pas code de garde neuf** pour le chemin nominal. La garde crash-window (§4(e).5)
   est le SEUL code neuf éventuel, et il vit dans la doctrine A2 (`runtime.rs` boundary `:2596-2606` /
   `:2718-2726`) — **ne pas rouvrir** le fail-loud A2, seulement l'étendre (refuse si backup-sibling).
5. **`sbfb-ideas` partout** dans la fixture et les asserts (jamais `sbfb-ides`). Couvrir `sbfb-feed`.
6. **Repères de ligne réels** (les refs plan/kickoff sont périmées ~90-100 l.) : `boot_storage_namespace`
   `runtime.rs:2562`, discriminateur « Replica not found » `:2602`, fail-loud `:2606`, recreate arms
   `:2634`/`:2647` ; `boot_feed_namespace` `:2693`, discriminateur `:2722`, fail-loud `:2726`, recreate
   `:2753`/`:2764`. `AnchorLocator` = `nexus-shell-daemon-core/src/iroh_runtime.rs:260` (PAS nexus-core-rs).

---

## 6. Résidu de durabilité CENTRAL — fenêtre de crash migration → recreate silencieux

**Le seul vrai risque de durabilité de Phase F** (surfacé par S3-adv M1 + S4-adv, manqué par les scans
d'origine et la Phase D review) :

`migrate_redb_v2_tuples.rs:166 std::fs::rename(source, &backup)` PUIS `:167
target.persist_noclobber(source)`. **Entre les deux, `docs.redb` est ABSENT.** Au reboot :
`Store::persistent` → `open_database` → `Database::create(path)` **crée un store VIDE neuf**
(`fs.rs:93`) → `new_impl` réussit (vide, aucun tuple-mismatch) → `boot_storage_namespace` →
`open_doc(ns_id de la ligne M8)` → **« Replica not found »** → `None` (`runtime.rs:2602`) → **RECREATE
SILENCIEUX** (warn-only `:2632`, `create_doc()` `:2634`). **Ce chemin N'EST PAS capté par le fail-loud
A2** : c'est un `NotFound` **légitime**, pas une erreur docs → la garde A2 (« refusing to silently
recreate ») ne s'applique **pas**. La donnée ne survit alors que dans `docs.redb.backup-redb-v2-tuples`,
**non auto-restauré**. **D3 cond.7 (« self-heal non déclenché en fenêtre de migration ») n'est donc PAS
garanti pour cet entrelacement.**

**Mitigations** (par ordre de robustesse) : (i) **règle bloquante no-boot** + **tar snapshot** avant
toute migration réelle (filet opérationnel, déjà exigé) ; (ii) **garde root-cause optionnelle** : dans
`boot_storage_namespace`/`boot_feed_namespace`, avant le recreate sur `None`, **refuser** (fail-loud) si
un sibling `docs.redb.backup-redb-v2-tuples` existe (= migration interrompue, restaurer) ; (iii) runbook
documente le mécanisme + caveat Linux `rename(2)` clobber d'un `.backup` préexistant au retry +
`docs.db.migrate*` temp orphelin. **Recommandation** : documenter + tar (obligatoire) ; la garde (ii) est
un petit durcissement root-cause qui ferme une vraie fenêtre de perte silencieuse — **à trancher au code**
(région A2 sensible ; si ajoutée, +1 test). Sous C4/C5 (« personne sur le réseau »), le blast-radius est
borné à l'ancre solo, mais la perte reste **permanente** si le tar n'est pas restauré → **carry G threat**.

---

## 7. Règle bloquante store réel (renforcée)

- **AUCUNE migration LIVE.** Migration/ouverture **uniquement sur COPIE fraîche** ré-extraite du tarball
  gitignoré vers un tmpdir hors-repo, **par run**.
- **AUCUN boot daemon/rig sur store réel** Win/Mac/VPS avant Phase F PASS. **Renforcé** : l'ordre boot ne
  protège plus (blobs no-op → docs migre one-way à tout boot). C'est l'unique garde-fou.
- **Snapshot Mac PENDING** avant tout boot Mac (inchangé).
- **Migration one-way** : rollback = restore tar OU rename `.backup-redb-v2-tuples`. Toujours snapshotter
  avant.
- **Duress orthogonal** : F offline/Normal (`noop_identity.rs:136` gate seulement le sync-set broadcast,
  jamais le data-dir `runtime.rs:2054-2061`) → aucune interférence avec la validation sur copie.

---

## 8. Carries sortants (F → G, K)

1. **G (THREAT_MODEL — le sprint réserve les édits à G ; §17 pt.3 impose une §5.x STRIDE par nouveau
   composant)** :
   - **T-STORE-MIGRATION-CRASHWINDOW** (§6) : rename↔persist crash → store vide → recreate silencieux
     NON capté par A2 (résidu de durabilité central) ; mitigation = tar snapshot + garde backup-sibling
     optionnelle + caveat Linux rename-clobber. Residual L.
   - **T-STORE-FIXTURE-LEAK** : la copie porte `node_key`/`default-author`/`worker.key`/`NamespaceSecret`
     ; la migration **crée** en plus `docs.redb.backup-redb-v2-tuples` (ancien `NamespaceSecret`) +
     `docs.db.migrate<rand>` → aucun capté par `*.redb`/`*.db` si relocalisé hors `data/`. Mitigation =
     fixture synthétique pour CI, copie réelle sous `data/`. Residual Nil (conditionnel).
   - **T-BLOBS-DURABILITY (dégradé)** : wipe MOOT (blobs ouvre) → pas de perte ; note « §15 = intégrité,
     pas durabilité » + contenu re-importable depuis `iroh/blobs/data/*.data` (BLAKE3 local, sans réseau)
     si un futur break iroh-blobs l'exigeait. Residual M.
   - **Correction kickoff C4/C5** : feature `redb-v2-migration` EXISTE + défaut → réconcilier avec root
     `Cargo.toml:36-42` (déjà correct) ; **tripwire** que la feature reste dans les défauts + aucun
     `default-features=false` sur iroh-docs.
   - **hickory-proto 0.24.4 RUSTSEC-2026-0119** (pré-existant, isolé à `DnsFallbackResolver`) : bump
     `nexus-core-rs hickory-resolver 0.24→0.26` (clôt l'advisory + dédoublonne) — hors scope F.
2. **K (dette / doc)** :
   - Repères de ligne périmés dans plan/kickoff : self-heal `:2518/:2606` → réels `:2562/:2602/:2606`
     (storage) + `:2693/:2722` (feed) ; `AnchorLocator` = `nexus-shell-daemon-core/iroh_runtime.rs:260`
     (pas nexus-core-rs) ; corriger « sbfb-ides » → « sbfb-ideas » + « namespace » → « clé M8 app_name ».
   - **Tripwire test** assertant que l'erreur d'absence de replica upstream contient bien « Replica not
     found » sous iroh-docs 0.101 (`store.rs:26`) — protège le discriminateur A2 `runtime.rs:2602` d'un
     changement de `Display` upstream (direction perte-silencieuse si dérive).
   - Checklist « ce qui doit survivre à l'upgrade » à consigner : `node_key`, `coordinator.db`(+WAL),
     `anchors.json`, `default-author`, `directory_revision.json` (anti-rollback S75), `subscriptions.json`
     (convergence gossip — lié au carry « subscribe-à-chaud »), `allowlist.sqlite3` worker.
3. **Snapshot Mac PENDING** (règle bloquante) — porté tant qu'aucun boot Mac n'a été snapshotté.

---

## 9. Invariants & Day-0 (tenus)

- **0 bump wire** par construction (`runtime.rs` boot path = 0 constante wire ; F = format de FICHIER
  redb, pas un contrat réseau ; DocTicket/BlobTicket = strings re-parsés, enjeu wire déjà couvert Phase
  D/E — `docs.rs:506`).
- **0 dep runtime neuve** : `redb_v3` (3.1.3) est **déjà** transitif ; en dev-dep il reste la **même**
  version lockée → **0 delta lock, 0 collision Phase G** `deny.toml:107`. **Aucun** dev-dep `redb-2.6.3`.
- **iroh strictement seul** ; migration = feature défaut built-in iroh-docs, `iroh-blobs` inchangé
  (ouverture no-op).
- **Verrous S74/S75 intacts** : F touche la couche fichier redb + un éventuel garde boot (doctrine A2), ni
  seed accept-list, ni subscription-gating, ni directory ingest, ni app count.
- **Aucune nouvelle frontière §6.12** (S4-9 CONFIRMED) → **Track K docs-contrat non déclenché** par F
  (F re-vérifie la survie de contrats persistés existants à travers une transformation on-disk).
- Toolchain **1.94** ; tests hermétiques + 1 intégration env-gatée ; sonde live = néant (F est hors-prod
  par gate ; l'acceptance live reste Phase H).

---

## VERDICT FINAL : **PLAN-ADAPT**

Phase F ne contredit **aucune** décision gelée Day-0 → **pas de DESIGN-CONFLICT / pas de STOP arbitrage
PO**. Elle **corrige deux prémisses internes erronées** avec de la preuve OSS vérifiée (dont la preuve
cardinale re-hexdumpée par la synthèse) : (1) la découverte Phase D « blobs.db redb-v2 illisible,
discard+refetch vs shim, perte pins M18 » est **REFUTÉE** (blobs.db est v3, ouvre proprement, 0 perte) ;
(2) le kickoff C4/C5 « feature `redb-v2-migration` n'existe pas » est **FAUX** (elle existe, défaut).
L'ADAPT **simplifie** Phase F : pas de wipe blobs, pas de neutralisation self-heal in-place (la migration
docs n'est pas in-place), fixture hermétique **dep-free** via `redb_v3` déjà locké. Le **seul résidu
réel** est la fenêtre de crash `migrate_redb_v2_tuples.rs:166↔167` (recreate silencieux non capté par A2)
→ mitigation tar + garde optionnelle + carry G. Périmètre : validation empirique sur copie fraîche +
2-3 hermétiques committés + assertion self-heal, **+3..5 Rust, 0 bump wire, 0 dep runtime neuve**.
Décision PO **facultative** (informative, pas bloquante) : acter que F abandonne la décision Phase-D
« blobs illisible » et corrige le kickoff C4/C5 — aucune de ces deux n'est un Day-0.
