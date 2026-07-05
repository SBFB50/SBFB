# Sprint 81 Phase D — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT.** La lettre du plan Phase D (« recompiler la couche blobs sous
> **0.103** + **valider l'ouverture du store redb4** ») a été **rédigée AVANT le bump Phase B
> (`c899d54`)**. Or B a déjà posé les pins (=1.0.1 / docs=0.101.0 / gossip=0.101.0 /
> blobs=0.103.0) et le workspace est VERT deux plateformes (nextest Win 2038 0-skip / Docker
> 2036-2042, clippy, doctests, release), **sans AUCUNE cassure compile côté blobs**. Conséquence
> tranchée item par item par 5 scans + vérifications adversariales : **les 11 items d'API de la
> lettre sont NO-OP compile-prouvés** avec double evidence (byte-diff des sources vendored
> `iroh-blobs-0.100.0` vs `0.103.0` : `hash.rs`/`api/tags.rs`/`net_protocol.rs`/`store/mem.rs`
> byte-identiques ; `ticket.rs` = 4 lignes de renommage de trait ; `add_bytes`/`get_bytes`/`has`/
> `Downloader::new`+`download`/`FsStore::load`/`BlobsProtocol::new` signatures inchangées). Le VRAI
> périmètre code de Phase D se réduit à **DEUX items minces**, plus **UN risque routé F** :
> 1. **[DOC-ONLY] blobs.rs:437** — commentaire de test « in-order consumption is iroh-blobs
>    **0.100** documented behavior » → `0.103`. Le comportement (blanket `ContentDiscovery for
>    IntoIterator` = ordre d'itération préservé) est **VÉRIFIÉ toujours vrai** en 0.103
>    (`downloader.rs:562-573` byte-identique) ; seul le numéro de version est périmé. Carry B → D
>    fermé ici.
> 2. **[TEST-A-AJOUTER] +1 round-trip BlobTicket pur** (mint EndpointAddr **peuplé** →
>    `to_string` → `from_str` → `into_parts`, hash+format préservés) verrouillant le contrat de
>    **format string persisté `anchors.json`** (`AnchorLocator.ticket`). +1..2 nets, PAS +3 : le
>    fetch local + tags + round-trip ticket 2-nœuds sont **DÉJÀ couverts** (blobs.rs:310 / :360-406
>    / :529-576).
> 3. **[RISQUE-BLOQUANT → routé Phase F, JAMAIS Phase D] Migration redb DUALE.** `iroh-blobs
>    0.103` (redb 4.1.0) **N'A AUCUN shim de migration** : `FsStore::load` (`node.rs:375`)
>    **hard-fail `UpgradeRequired`** sur un vrai `blobs.db` redb-v2 (`store/fs/meta.rs:516-523`,
>    le `db.upgrade()` de 0.100 a été RETIRÉ). ASYMÉTRIQUE à `iroh-docs 0.101` qui, lui, MIGRE
>    `docs.redb` v2→v4 **ONE-WAY destructif** (`Docs::persistent` `node.rs:388`, feature défaut
>    `redb-v2-migration`, `migrate_redb_v2_tuples.rs`). Le « valider l'ouverture redb4 » de la
>    lettre **n'est PAS faisable en Phase D** (exige COPIE staging + décision prod) → **différé F**.
>
> Aucun Day-0 touché ; **0 bump wire SBFB tenu par construction** (blobs.rs/node.rs ne
> DÉFINISSENT aucune constante `DOMAIN_*`/`_FORMAT_VERSION` ; node.rs porte 2 références
> DOCUMENTAIRES — doc-links `:67`/`:78` vers `seed.rs`/`compute_group.rs` — non touchées ;
> précision post-Codex round 1, la formule initiale « grep vide » était trop large) ;
> iroh strictement seul ; toolchain 1.94 inchangée.
> **Ce 4e PLAN-ADAPT pour cause « lettre pré-bump » est ATTENDU et structurel** (A/A2/A4 pour
> A2/A4, C aussi), pas un signal méta nouveau. Aucune question PO bloquante (0 DESIGN-CONFLICT).
> **Sémantiques compile-invisibles TRANCHÉES documentation-only** (0 code) : `tag = racine-GC`
> (keep-online) et `ordre providers + retry` (fetch_hash_multi anchor-first) sont **byte-identiques
> 0.100→0.103** (S2-5/S2-6 adversarial), **pas de DESIGN-CONFLICT keep-online**.
> G8 : 5 scans (S1a OSS iroh-blobs 0.103 vendored / S1b deps-CVE-lock / S2 décisions historiques /
> S3 threat model / S4 wire) + 5 vérifications adversariales. Bilan verdicts : **CONFIRMED
> dominants ; 1 REFUTED matériel (S2-10, réhabilite le test anchors.json) ; ADJUSTED substantiels
> sur la dualité redb (S2-7/S3-2/S3-9) et la portée byte-identité ticket (S4-2)** — tous
> réconciliés ci-dessous.

---

## 1. Rappel de la lettre du plan (sprint81_plan.md:165-177)

Phase D « iroh-blobs cascade + redb4 ». **But** : recompiler la couche blobs sous **0.103** +
valider l'ouverture du store **redb4**. **Jobs/surfaces** : blobs + tags + downloader + tickets,
crate `nexus-core-rs`. **Livrables** : `blobs.rs:85-252` (`add_bytes` / `TagInfo.hash`, `get_bytes`
/ `has`, `tags().set/delete/get`, `HashAndFormat::raw`, `Downloader::new + download`,
`BlobTicket::new / into_parts`, `Hash::from_bytes`) ; `node.rs:47-50,375-398` (`FsStore::load` /
`MemStore` / `BlobsProtocol::new` / store deref) ; re-vérif signatures `BlobsProtocol::new` +
`Downloader::new`. **Delta tests** : +1..3 Rust (ticket round-trip, tag set/get, blob fetch local).
**T1** : alimente sous-test (1) blobs fetch in-process + sous-test (4) parse `BlobTicket`
(`anchors.json`). **Gate/scope-cut** : changelog 0.101→0.103 non détaillé côté signatures →
découvrir au compile, documenter tout break au body ; 0 bump wire SBFB.

**Le nœud du PLAN-ADAPT** : comme en C, le libellé décrit une « recompilation » que le bump B a
déjà faite (2038 tests verts sous blobs 0.103) et une « validation d'ouverture redb4 » qui,
prise à la lettre, **ordonnerait d'ouvrir un store réel** — interdit avant Phase F (migration
one-way pour docs, hard-fail pour blobs). La lettre n'a PAS anticipé (a) que le bump absorberait
toutes les signatures, (b) que la « validation redb4 » relèverait de F sur COPIE, (c) que le
store blobs réel serait **illisible** par 0.103 (pas juste « à migrer »). Ci-dessous le vrai
périmètre, evidence-adossé item par item.

---

## 2. Pourquoi PLAN-ADAPT — les items du libellé sont NO-OP compile-prouvés (double evidence)

Chaque verdict NO-OP est adossé à **DEUX** sources concordantes : (a) baseline B verte acquise
(nextest 2038 Win 0-skip @ `c899d54`, blobs.rs+node.rs recompilés vert) ; (b) **byte-diff des
sources vendored** `iroh-blobs-0.100.0` (pile 0.98) vs `iroh-blobs-0.103.0` (cible) — pas une
simple inférence de compile. Les 5 scans convergent, 0 réfutation de conséquence sur ces items.

| Livrable de la lettre | Verdict | Preuve upstream (0.100↔0.103) | Ancre SBFB |
|---|---|---|---|
| `add_bytes` + `TagInfo.hash` (champ) | **NO-OP compile-prouvé** | `add_bytes(impl Into<Bytes>) -> AddProgress` inchangé (`api/blobs.rs:186`) ; `pub hash: Hash` (`:736`) ; hors tout hunk de diff | `blobs.rs:85-96` (doc `:87` déjà « 0.103 ») |
| `get_bytes` | **NO-OP** | `api/blobs.rs:381` inchangé, hors hunk | `blobs.rs:103-112` |
| `has` | **NO-OP** | `api/blobs.rs:517` sig inchangée (route via `status()`) | `blobs.rs:145-152` |
| `tags().set/delete/get` + `HashAndFormat::raw` | **NO-OP** | `diff api/tags.rs` = **VIDE (byte-identique)** ; `hash.rs` diff **VIDE** (`raw`/`from_bytes`/`as_bytes`/Display/FromStr stables) | `blobs.rs:120-141` (prod) + `:567` (test) |
| `Downloader::new` + `download` | **NO-OP** | `downloader.rs:389`(new)/`:404`(download) sig inchangées ; blanket `ContentDiscovery for IntoIterator` byte-identique (`:562-573`) | `blobs.rs:193-197`, `:246-250` |
| `BlobTicket::new` / `into_parts` | **NO-OP** | `diff ticket.rs` = **4 lignes** (renommage trait `Ticket` : `to_bytes→encode_bytes`…), struct/champs identiques ; enveloppe string `blob`+BASE32 inchangée | `blobs.rs:183-186` (from_str/into_parts) + `:386/:548` (new, test) |
| `Hash::from_bytes` | **NO-OP** | `hash.rs` octet-pour-octet identique | `blobs.rs:104,121,146,245` |
| `FsStore::load` / `MemStore` / store deref | **NO-OP (signature)** | `store/fs.rs:1390` load inchangé (`blobs.db`) ; `store/mem.rs` diff **VIDE** ; `Default` `:100` | `node.rs:375,380` (repères EXACTS post-B) |
| `BlobsProtocol::new(&store, None)` | **NO-OP** | `diff net_protocol.rs` = **VIDE** ; `BLOBS_ALPN` ré-exporté inchangé | `node.rs:398` + import `:47-50` |

**Repères de la lettre re-vérifiés post-B** : `blobs.rs:85-252` **TOUJOURS EXACT** (`add_bytes` à
85, fin `fetch_hash_multi` à 252) — MAIS `BlobTicket::new` et `tags().get` **ne vivent QUE dans le
code de test** (`:386/:548`, `:567`), pas dans la surface prod. `node.rs:47-50` et `375-398`
**EXACTS** (node.rs non collapsé par clippy, contrairement à blobs.rs). Grep vérifié :
`blobs.rs:87` dit déjà « 0.103 », `blobs.rs` porte **0** constant wire.

**Conclusion §2** : sur les 11 items d'API de la lettre, **aucun n'est du code à écrire**. Le bump
Phase B les a absorbés et le byte-diff des sources le confirme indépendamment. Phase D **vérifie**
ces surfaces (au run des tests existants), elle ne les re-type pas.

---

## 3. Le VRAI travail — item 1 (DOC-ONLY) : blobs.rs:437 « 0.100 » → « 0.103 »

Carry B routé D confirmé. Grep authoritative :

```
blobs.rs:437:  // consumption is iroh-blobs 0.100 documented behavior (blanket
```

C'est le doc-comment du test `fetch_falls_back_to_seeder_when_anchor_offline`. La **classe** de la
revendication reste **VRAIE** en 0.103 : le blanket `impl<C,I> ContentDiscovery for C where
C: IntoIterator` (`downloader.rs:562-573`) fait `n0_future::stream::iter(providers.into_iter()
.map(Into::into))` — **ordre d'itération préservé, PAS shufflé** (contraste `Shuffled` à `:585`
qui, lui, appelle `.shuffle()`). Seul le numéro de version est périmé. **Action** : éditer `0.100`
→ `0.103` (ou `0.100+`). C'est l'unique mention numérique périmée de `blobs.rs` (l'autre, `:87`,
est déjà correcte). **`node.rs` = 0 doc-stale numérique** (refs génériques « iroh-blobs »/« redb »
sans numéro — ne pas inventer d'item). 0 impact wire.

---

## 4. Le VRAI travail — item 2 (TEST-A-AJOUTER) : round-trip BlobTicket (contrat anchors.json)

### 4.1 anchors.json PERSISTE bien un BlobTicket string (REFUTED S2-10 → réhabilité)

**Point le plus important à faire remonter côté test** : le scan S2 avait affirmé « anchors.json
stocke un HASH BLAKE3 NU, PAS un BlobTicket → le test round-trip serait conceptuellement faux ».
La vérification adversariale l'a **REFUTED** et le byte-code le confirme : `AnchorLocator.ticket`
est **un `pub ticket: String` = un BlobTicket string persisté sur disque** (`crates/
nexus-shell-daemon-core/src/iroh_runtime.rs:260-268`), écrit par l'ingest prod
(`announcement.blob_ticket`, `:1221`) et sérialisé dans `<shell-daemon>/anchors.json`
(schéma-versionné `ANCHORS_SCHEMA_VERSION=1`, `:1438-1491`). Au boot, `repull_one_directory` →
`BlobsClient::fetch_ticket` → `BlobTicket::from_str` (`blobs.rs:183`) le **re-parse**. La lettre
D (« BlobTicket round-trip type anchors.json ») est donc **CORRECTE** ; c'est le NodeDirectory
**catalog** (`node_directory.rs:138`, `CatalogApp.archive_hash`) qui, lui, stocke un hash hex nu
— S2 avait conflaté les deux.

### 4.2 Le format string BlobTicket est byte-stable 0.100↔0.103 (S4-2, nuance)

`ticket.rs` = 4 lignes de renommage de trait ; `iroh-tickets 0.5.0 serialize` == `1.0.0
encode_string` (préfixe `KIND="blob"` + `BASE32_NOPAD.encode_append` + lowercase, **byte-identique**).
Donc un round-trip **intra-0.103** est trivialement stable, et un ticket persisté sous 0.98
re-parse sous 0.103 **par forme de struct**. **Nuance adversariale honnête (S4-2 ADJUSTED)** : le
hexdump `test_ticket_base32` ne prouve la byte-identité stricte que du **ticket VIDE**
(`relay_url=None`, 0 adresse). Un ticket SBFB réel porte un `EndpointAddr` **peuplé** ; sa
byte-identité stricte dépend du postcard de `RelayUrl`/`SocketAddr`/`EndpointId` **non diffé
indépendamment** ici. Sans impact : (a) re-parse piloté par la forme (identique) ; (b) **self-heal
design** — un `from_str` KO au boot = branche vide, `repull` rend 0, le catalogue re-arrive au
prochain live-announce (`runtime.rs:2278` `if let Ok(ticket)=…`, `AnchorLocator` doc `:254`
« stale ticket tolerated ») ; (c) la **HASH** est le seul champ load-bearing (adresse éphémère
re-résolue pkarr). **Conséquence pour le test** : minter avec un `EndpointAddr` **PEUPLÉ** (comme
`two_nodes_fetch_blob_via_ticket` via `my_endpoint_addr`), jamais le cas vide.

### 4.3 Couverture existante — NE PAS dupliquer

- `add_then_get_roundtrip` (`blobs.rs:310`) — add/get local.
- `two_nodes_fetch_blob_via_ticket` (`blobs.rs:360-406`) — `BlobTicket::new`→`to_string`→
  `from_str`→`into_parts` **round-trip end-to-end vert sous 0.103**, addr peuplé.
- `seeder_fetches_tags_pins_blob` (`blobs.rs:529-576`) — `fetch_and_pin` + `tags().get`.
- `fetch_hash_multi_rejects_empty_providers` (`blobs.rs:408-420`) — empty-reject.
- `archive_hash_from_ticket_decodes_the_hash` (daemon `http.rs:7388`) — `add_bytes`→`BlobTicket::new`
  →`archive_hash_from_ticket`→`into_parts`.

Le fetch local (sous-test T1 (1)) et la mécanique ticket (sous-test T1 (4)) sont donc **déjà
alimentés**. Le vrai **gap** = un **round-trip BlobTicket PUR** (sans nœud) qui verrouille
spécifiquement le contrat de format-string de `anchors.json` comme une garde dédiée.

---

## 5. Le RISQUE-BLOQUANT — migration redb DUALE (routé Phase F, JAMAIS Phase D)

Le point le plus lourd du préflight, confirmé par 4 scans (S1a-12 CONFIRMED, S1b-3 CONFIRMED,
S2-7 ADJUSTED, S3-2/S3-9 ADJUSTED). La lettre « valider l'ouverture du store redb4 » **ne peut
PAS s'exécuter en Phase D** : elle exige d'ouvrir un store, ce qui déclenche une bascule
irréversible ; la validation appartient à Phase F sur COPIE staging.

### 5.1 DUALITÉ docs-migrate vs blobs-hard-fail (correction load-bearing S2-7)

Le libellé « iroh-blobs cascade + redb4 » et le scan S2-7 pointaient tous deux le **mauvais
store**. Réalité, deux sites dans `node.rs` avec des comportements **OPPOSÉS** :

- **`iroh-DOCS` (`Docs::persistent`, `node.rs:388`)** : MIGRE `docs.redb` v2→v4 **ONE-WAY
  destructif** — feature **DÉFAUT** `redb-v2-migration` (`iroh-docs-0.101.0/Cargo.toml`, lecteur
  `redb_v3` ^3.1 embarqué), `migrate_redb_v2_tuples.rs` (NamedTempFile + swap). C'est la migration
  Phase F déjà cadrée en B.
- **`iroh-BLOBS` (`FsStore::load`, `node.rs:375`)** : **AUCUN shim**. `store/fs/meta.rs:516-523`
  fait `Database::create` et sur `Err(DatabaseError::UpgradeRequired(v))` **hard-error** :
  « migration from redb v{v} no longer supported; upgrade with an older redb version first ». Le
  `db.upgrade()` **présent en 0.100** (redb 2.6.3, appelé inconditionnellement à chaque open) a
  été **RETIRÉ** en 0.103 (redb 4.1.0). `redb-4.1.0/CHANGELOG` : « Removes support for file format
  v2 ».

**Conséquence prod** : un `blobs.db` réel resté en redb-v2 (jamais ré-ouvert par 0.100 — ex.
snapshot) est **ILLISIBLE par 0.103** (hard-fail, store non muté, **pas** de migration ni recreate
silencieux). Un `blobs.db` déjà booté par le daemon prod actuel est **déjà en v3** (0.100 upgradait
à chaque open) → 0.103 l'ouvre proprement. **Interaction d'ordre de boot** (S3 missed) : `node.rs`
ouvre **blobs D'ABORD** (`:369-381`) puis docs (`:383-395`) → un boot accidentel sur store réel
**avorte à `FsStore::load` AVANT** que la migration docs (irréversible) ne s'exécute.

### 5.2 Décision F à NOMMER maintenant (pas « juste verify on copy »)

Le vrai `blobs.db` prod héberge les **pins keep-online (tags M18, S74)**. Si F résout le
`UpgradeRequired` blobs par **discard+refetch** (les blobs sont content-addressés + re-fetchables,
probablement acceptable), **chaque pin keep-online est silencieusement perdu** — surface
d'atteinte à la garantie de durabilité « héberger ≠ publier » à travers l'upgrade. **Décision F
pendante** : discard+refetch (perte pins) vs outil d'upgrade redb intermédiaire vs shim. Ce n'est
pas un DESIGN-CONFLICT Phase D (aucune décision Day-0 ne l'interdit), mais un **item F réel à
nommer**, pas un « verify on copy » anodin. Corollaire S1b missed : la migration **docs** est
elle-même 2-sauts (v2→via-3→v4) et **NON validée** — `docs.redb` exige AUSSI la validation COPIE
en F, pas seulement blobs.

### 5.3 Pourquoi Phase D n'est PAS bloquée

La garde d'isolation est **VÉRIFIÉE en place** (S3-3 CONFIRMED) : tous les tests `blobs.rs` bootent
en **`MemStore`** via `create_node()` (`blobs.rs:305-307`, data_dir=None → `node.rs:380`
`MemStore::default()` — 0 fichier disque, 0 migration possible) ; les seuls tests `FsStore`
(`node.rs:490/556/580`) utilisent `tempfile::tempdir()` (redb4 **FRAIS** sous tmp, PAS migration).
Point décisif : `create_node` **ne lit AUCUNE variable d'environnement** pour `data_dir`
(exclusivement `cfg.data_dir` ; les seuls env-reads du crate = relay/DNS/pkarr, aucun n'ouvre un
store) → **impossible qu'un test pointe accidentellement le store réel via l'env**. Phase D telle
que re-cadrée (doc + 1 test tempdir/MemStore/pur) est **sûre par construction**.

---

## 6. Sémantiques compile-invisibles — TRANCHÉES documentation-only (0 code)

Deux contrats que le compile **ne prouve pas** ; le byte-diff adversarial les a **résolus** (S2-5 /
S2-6, downgradés d'« ouvert » à documentation-only, **pas de DESIGN-CONFLICT**) :

1. **`tag = racine-GC` (keep-online, S74D)** : `api/tags.rs` est **byte-identique** 0.100↔0.103 ;
   `store/gc.rs` ne diffère que d'un ajout doc de 3 lignes, les racines viennent toujours de
   `store.tags().list()` + `list_temp_tags()`. Invariant **PRÉSERVÉ**. Bénin de plus (S1a-17) : le
   GC est de toute façon **désactivé** (`Options.gc = None` par défaut, `store/fs/options.rs:124`,
   `FsStore::load` ne configure aucun GC) → les tags keep-online n'ont **aucun reaper runtime à
   protéger aujourd'hui**. Aucun risque, 0 code.
2. **Ordre providers + retry (fetch_hash_multi anchor-first, S75D, verrou 4 §15.1)** : le blanket
   `ContentDiscovery` + `execute_get` (« try each provider in order … fully sequential »,
   `downloader.rs:490-508`) sont **byte-identiques**. Cap `MAX_FETCH_PROVIDERS=16` (`blobs.rs:60`)
   + `truncate` DANS la primitive (`:244`) + empty-check (`:237-241`) **intacts post-B**.
   Intégrité BLAKE3 = seule frontière. **PRÉSERVÉ**, 0 code.

**Deltas sémantiques bénins découverts** (à mentionner au body, 0 action) :
- **`has(Hash::EMPTY) == true` inconditionnel** en 0.103 (`status()` court-circuite `Hash::EMPTY`
  sans backend, `#238`). Bénin : le test `has_returns_false_for_unknown_hash` utilise all-zeros
  `[0u8;32]` qui **n'est PAS** `Hash::EMPTY` (`blake3("")=af1349b9…`) → tape le backend → false.
  SBFB ne dépend jamais de `has(EMPTY)==false`.
- **Downloader « reap completed tasks » (0.101, `#225`)** : fix fuite mémoire (refactor
  `DownloaderActor` : `WaitIdle`/`idle_waiters`/`JoinSet` reaping ; API additive `wait_idle()`
  inutilisée par SBFB). Bénéfice runtime gratuit pour fetch_ticket/fetch_hash_multi. **Orthogonal
  à l'ordre providers** (S3-1 ADJUSTED : `downloader.rs` **n'est PAS** byte-identique comme fichier
  — 157 lignes changées — mais `new`/`download`/`ContentDiscovery` le sont).
- **`futures-lite 2.6.0` retiré** des deps directes d'iroh-blobs 0.100→0.103 ; **`iroh-util 0.6.0`**
  NEW dep ; **`getrandom 0.4.2`** devient dep directe (S1b-6 missed). Internes, bénins.
- **Enum `Request` = 10 variantes** (`Get, Observe, Slot2..Slot7, Push, GetMany`, réservées
  forward-compat), **PAS 4** (S3-1 mis-énuméré) ; identique aux 2 versions.

---

## 7. Restitution des scans (fan-out 5 + adversarial)

| Scan | Verdict-hint | Findings clés retenus (CONFIRMED / ADJUSTED-corrigé) | Adversarial |
|---|---|---|---|
| **S1a** OSS iroh-blobs 0.103 (byte-diff vendored) | PLAN-ADAPT | 11 items API = NO-OP byte-prouvé (hash.rs/tags.rs/net_protocol.rs/mem.rs identiques ; ticket.rs 4 renames) ; **S1a-12 RISQUE redb hard-fail asymétrique** ; S1a-14 anchors.json ticket persisté (graceful degrade) ; S1a-15 doc-stale :437 ; S1a-18 test +1..2 | **19/19 CONFIRMED**. Corrections : S1a-13 span test 360-382 (≠364-379) ; S1a-19 le commentaire Cargo.toml **ne couvre PAS** la copy-validation blobs |
| **S1b** deps/CVE/lock | PLAN-ADAPT | S1b-1/2 redb {3.1.3, 4.1.0} conforme B (2.6.3 disparu) ; **S1b-3 RISQUE asymétrie migration (blobs sans shim)** ; MSRV 1.91 tenu ; deny/lock inchangés (code-only) | 6 CONFIRMED. **3 ADJUSTED** : S1b-4/5 (CVE quinn-proto / absence advisory = **web-sourcés, non rejouables offline**, bonus non-bloquant) ; S1b-6 (omet la **suppression `futures-lite`**) |
| **S2** décisions historiques | PLAN-ADAPT | S2-1 signatures NO-OP ; S2-3 doc-stale :437 (carry B) ; S2-5/S2-6 sémantiques → **documentation-only** ; S2-7 site migration ; S2-9 test warranted ; S2-13 discriminateur BlobStore Mem/Fs | 10 CONFIRMED, **3 ADJUSTED, 1 REFUTED**. **S2-10 REFUTED** (anchors.json EST un ticket persisté) ; **S2-7 ADJUSTED** (dualité docs-migrate vs blobs-hard-fail) ; S2-12 (« pas de persistance ticket long-terme » faux) |
| **S3** threat model / sécurité | PLAN-ADAPT | S3-1 surface réseau 0 delta (BlobsProtocol/Request/Downloader) ; S3-3 isolation hermétique (0 env→data_dir) ; S3-4 cap+ordre intacts ; S3-5 duress gates daemon fermés, primitives blobs PURES | 6 CONFIRMED, **3 ADJUSTED**. S3-1 (Request 10 variantes ; downloader.rs 157 l changées mais API stable) ; S3-2 (dualité migration) ; **S3-9 (résolu : blobs.db réel ILLISIBLE, pas « question ouverte »)** |
| **S4** wire producteur→consommateur | PLAN-ADAPT | S4-1 **0 constant wire dans blobs.rs/node.rs** (grep vide) ; S4-2 BlobTicket byte-stable (INVERSE du DocTicket C) ; S4-3/4 sites persistés (anchors.json + gossip_outbox SQLite) re-parsent + self-heal ; S4-5 hash = hex SBFB (découplé Hash::Display) | 9 CONFIRMED, **1 ADJUSTED** (S4-2 : byte-identité stricte prouvée du seul ticket VIDE ; peuplé = re-parse-safe par forme + self-heal — dire « re-parse-safe » pas « byte-identique » catégorique) |

**Convergence** : 5 scans PLAN-ADAPT. Aucun REFUTED n'inverse le verdict ; le seul REFUTED
matériel (S2-10) **renforce** un livrable (le test anchors.json est légitime). Les ADJUSTED
substantiels (dualité redb, portée byte-identité ticket) sont **intégrés** aux §4/§5.

---

## 8. Plan de tests concret (delta recalculé)

**Contrainte de greffe** : le round-trip BlobTicket **pur** ne nécessite AUCUN nœud → test unit
hermétique dans `crates/nexus-core-rs/src/blobs.rs`, module test existant, **0 helper à hoister**
(contraste Phase C qui touchait des `async fn` privées). Pas de dette WS-3/PD-5 déclenchée ici.

| Test | Type | Assertion | Garde anti-store-réel |
|---|---|---|---|
| `blob_ticket_string_round_trips_under_current_lock` | GREEN | `BlobTicket::new(addr_PEUPLÉ, Hash::from_bytes(h), Raw)` → `to_string` → `from_str` → `into_parts` : hash + format préservés + idempotence `parsed.to_string()==s` | **AUCUN nœud, AUCUN store** — construit `EndpointAddr` peuplé en mémoire |
| (optionnel) round-trip `fetch_hash_multi` tronque >16 → 16 | GREEN | trou de couverture pré-existant (S3-7/S1b) — **non-régression**, non bloquant ; à pondérer vs budget | `MemStore` via `create_node()` ou EndpointId leurres |

**Explicitement NE PAS ajouter** : add/get, tags().get, ticket round-trip 2-nœuds → **déjà
couverts** (§4.3). **NE PAS forger de fixture 0.98 genuine** (aucun littéral committé ; crate 0.98
hors lock ; garde un non-scénario pre-launch = zombie à supprimer, cf. politique CLAUDE.md).

**Delta tests réaliste : +1..2 Rust, −0 zombies.** Le libellé plan « +1..3 » est **honnête mais
haut** : viser **+1 net** (round-trip pur) — le fetch local + tags sont déjà verts. Acter au body
que +3 sur-estime (redondance avec l'existant). **Aucun test d'ouverture redb4** en Phase D (=
Phase F sur COPIE).

**Note routée F/daemon (hors D core-rs)** : un round-trip serde de `AnchorLocator`
(`iroh_runtime.rs`, ticket string in → `serde_json` → out → re-`from_str`) verrouillerait le
contrat `anchors.json` end-to-end, mais vit dans `nexus-shell-daemon-core` (hors scope crate D) —
mentionner sans l'implémenter en D.

---

## 9. Règles bloquantes re-jouées (COPIE staging / one-way / Mac PENDING)

- **JAMAIS ouvrir/booter un store réel avant Phase F PASS.** 4 stores redb réels énumérés sur la
  machine : `%APPDATA%\nexus-grid\shell-daemon\iroh\docs.redb` + `iroh\blobs\blobs.db`,
  `local-worker\data\docs.redb` + `local-worker\data\blobs`. `FsStore::load` (`node.rs:375`,
  hard-fail blobs) et `Docs::persistent` (`node.rs:388`, migration docs one-way) les
  déclencheraient à l'ouverture.
- **Toute validation d'ouverture redb4 = COPIE décompressée du snapshot staging**
  (`C:\Users\FlowUP\sbfb-snapshots\s81-phase-b\` côté Win, contient `docs.redb` + `blobs.db` +
  `data/` ; store VPS rapatrié `data/vps-store-098/`), **jamais** le chemin `%APPDATA%`/`SBFB_HOME`
  réel. = Phase F, pas D.
- **Snapshot Mac PENDING** → **AUCUN binaire 0.101/0.103 déployé ni booté sur le Mac** avant
  capture du snapshot.
- **Migration ONE-WAY** : docs = rewrite destructif irréversible (backup upstream non garanti
  couvrant tous les chemins) ; blobs = hard-fail non-migrant (store non muté mais **illisible**).
  Garder les snapshots.
- **Phase D = code-only** : NE PAS toucher `deny.toml` ni `Cargo.lock` (le warn redb dupliqué est
  attendu, hors-scope gate G). Un edit deps sortirait du scope → escalade.

---

## 10. Carries sortants (créés / re-routés D → F, G, K)

1. **Phase F (BLOQUANT à nommer)** :
   (a) **Dualité redb** — valider sur COPIE : `docs.redb` migre-t-il proprement (one-way,
   2-sauts v2→3→4) ? `blobs.db` hard-fail `UpgradeRequired` attendu → **décision prod pendante**
   discard+refetch (perte pins keep-online) vs outil upgrade intermédiaire vs shim. Nommer les
   DEUX stores, comportements OPPOSÉS.
   (b) **Durabilité keep-online** : un discard+refetch du blob store **droppe silencieusement les
   pins keep-online (M18/S74)** — surface « héberger ≠ publier » à travers l'upgrade ; trancher en F.
   (c) **anchors.json graceful-degrade** : au boot COPIE, vérifier qu'un `anchors.json` écrit sous
   0.98 dégrade sans panic (re-pull via live-announce, chemin `runtime.rs:2278` Ok-guard non-fatal).
   (d) **Snapshot Mac** à capturer AVANT tout boot 0.101 Mac.
   (e) Layout des fichiers de données blob (non-meta) non diffé indépendamment → couvert
   empiriquement par le boot-COPIE (assomption content-addressée, pas diff-prouvée).
2. **Phase G (THREAT_MODEL / doc)** : (a) ligne migration redb (docs one-way + blobs hard-fail)
   subsumant la note B ; (b) bénéfice sécurité collatéral **quinn-proto 0.11.14** (patch
   RUSTSEC-2026-0037/CVE-2026-31812 DoS QUIC obtenu gratuitement par le bump iroh — **web-sourcé,
   PLAUSIBLE, à re-confirmer**) ; (c) note ticket **iroh-tickets 0.5→1.0** = renommage pur, string
   inchangée.
3. **Phase K (dette)** : test troncature-16 `fetch_hash_multi` (trou pré-existant, non-régression,
   optionnel) ; WS-3/PD-5 hoisting **NON déclenché** en D (test pur, 0 helper) ; surfaces BlobTicket
   **daemon** hors core-rs (`runtime.rs:2136/2199/2278/2348`, `http.rs:3188` — mint/parse
   compile-prouvés stables par B, non re-testés en D core-rs).
4. **Veille continue** : re-jouer crates.io (1.0.2/0.103.1 ?) + RustSEC avant le push live (règle
   plan inchangée ; à 2026-07-03, 0.103.0 et 1.0.1 = plafonds stables, pins conformes).

---

## 11. Invariants & Day-0 (tenus)

- **Bisectabilité (précédent S32 `90aff27`)** : la recompilation appartient à B ; Phase D =
  **doc-comment + tests uniquement** (0 changement fonctionnel de `blobs.rs`/`node.rs`). Ne pas
  rétro-amender B.
- **0 bump wire SBFB (par construction)** : `blobs.rs` + `node.rs` ne DÉFINISSENT aucune
  constante `DOMAIN_*` / `_FORMAT_VERSION` / `FEED_FORMAT_VERSION` (node.rs `:67`/`:78` =
  doc-links documentaires vers `seed.rs`/`compute_group.rs`, non touchés — précision
  post-Codex round 1). Le format string BlobTicket est celui
  d'iroh (compat-upgrade byte-stable), pas un wire SBFB. La hash exposée au front
  (`/blob-serve/{hash}`), en DB (keep_online), dans les feeds/`NodeDirectoryEntry`/`SeedRequest` =
  **hex SBFB** (`hex::encode`/`decode_hash_hex`), **découplée de `iroh Hash::Display`** → insensible
  à tout changement Display. Pas de nouvelle frontière docs-contract (test-acteur §6.12 : aucune
  étiquette requise).
- **iroh STRICTEMENT SEUL (D7)** : 0 dep ajoutée par D (code-only ; lock/deny inchangés). Le NEW
  `iroh-util`/`getrandom` + retrait `futures-lite` = conséquences transitives du bump B, pas de D.
- **Toolchain 1.94 inchangée (D6)** : crates cibles `rust-version 1.91` ; aucun bump MSRV requis.
- **Tests hermétiques uniquement** : in-process `create_node`/`MemStore` ou `tempfile::tempdir`,
  jamais store réel Win/Mac/VPS avant Phase F PASS. `create_node` ne lit aucun env pour `data_dir`.
- **Total de tests jamais en baisse silencieuse** : +1..2, −0.

---

## 12. Risques résiduels

- **Résidu `EndpointAddr` serde peuplé** (S4-2) : non byte-diffé pour `Relay`/`Ip` addrs
  0.98→0.103 ; risque faible (enum + dérives + ordre identiques), fermé empiriquement pour 0.103
  par le round-trip §8 (addr peuplé) + self-heal boot ; le cas 0.98-genuine reste un non-scénario
  pre-launch (route NOTE F).
- **Décision F blobs.db illisible** : discard+refetch (perte pins keep-online) vs shim — pendante,
  à trancher AVANT tout boot store réel. Ne pas la traiter en D.
- **Claims CVE/advisory web-sourcés** (S1b-4/5) : quinn-proto 0.11.14 dans le lock = fait repo
  vérifié ; l'attribution RUSTSEC-2026-0037 + « 0 advisory redb » sont **web-sourcés non rejouables
  offline** → traiter comme bonus PLAUSIBLE, 0 impact action D.
- **Réserve d'honnêteté NO-OP** : les verdicts « compile-prouvés » héritent de la baseline B verte
  (nextest 2038 Win) **plus** un byte-diff indépendant des sources vendored (inférence solide, pas
  seulement héritée).
- **Env session (report 03/07)** : kills Bash `run_in_background` → jouer les suites en avant-plan ;
  classe env Docker-on-Windows (tests iroh-networked / HTTP loopback lents) inchangée — non
  pertinente pour D (tests core-rs unit/MemStore/pur).

---

## 13. Commit shape (indicatif)

`chore(deps): Sprint 81 Phase D — re-cert couche blobs sous iroh-blobs 0.103, doc-stale + round-trip
BlobTicket (0-bump wire)` — body : 11 items API NO-OP byte-prouvés (hash.rs/tags.rs/net_protocol.rs/
mem.rs identiques, ticket.rs 4 renames, add_bytes/get_bytes/has/Downloader::new+download/FsStore::load/
BlobsProtocol::new signatures inchangées, absorbés par bump B) + doc-only `blobs.rs:437` « 0.100 »→
« 0.103 » (comportement blanket ContentDiscovery vérifié toujours vrai) + round-trip BlobTicket pur
(addr peuplé, contrat anchors.json — S2-10 réhabilité : anchors.json PERSISTE un ticket) + sémantiques
compile-invisibles TRANCHÉES documentation-only (tag=racine-GC keep-online + ordre providers = byte-
identiques 0.100→0.103, GC désactivé par défaut) + deltas bénins notés (has(EMPTY)==true, reap 0.101,
futures-lite retiré, iroh-util neuf, Request 10 variantes) + **RISQUE redb DUAL routé F** (docs migre
one-way `Docs::persistent` vs blobs hard-fail `UpgradeRequired` `FsStore::load` — blobs.db réel
illisible sous 0.103, décision discard+refetch/shim pendante, perte pins keep-online à trancher) +
carries F (dualité redb + durabilité keep-online + anchors graceful-degrade + snapshot Mac)/G
(THREAT_MODEL migration + quinn-proto CVE bonus)/K (troncature-16, sites BlobTicket daemon) + delta
tests +1..2 Rust −0 zombies + 0 bump wire SBFB (blobs.rs/node.rs = 0 DOMAIN_/FORMAT_VERSION) + iroh
strictement seul + toolchain 1.94 + tests hermétiques (jamais store réel avant F PASS, COPIE staging,
Mac PENDING).
