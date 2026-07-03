# Sprint 81 Phase C — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT.** La lettre du plan Phase C (« adapter la surface docs aux types
> iroh-base 0.100 + wire `EntrySignature → iroh::Signature` ; le vrai travail de migration,
> pas un recompile ») a été **rédigée AVANT le bump Phase B (`c899d54`)**. Or B a déjà posé
> les pins (=1.0.1 / docs=0.101.0 / gossip=0.101.0 / blobs=0.103.0) et le workspace est VERT
> deux plateformes (nextest Win 2028 0-skip / Docker 2030-2032, clippy, doctests, release),
> avec **une seule cassure compile absorbée en B (pkarr)**. Conséquence tranchée item par
> item par 6 scans + vérifications adversariales : **la quasi-totalité des « Livrables » du
> libellé est NO-OP compile-prouvé ou PHANTOM** (re-typage `docs.rs`, `DocsNamespaceId::from`,
> `node.rs`, matcher `Replica not found`, zombies legacy-decode, `EntrySignature`). Le VRAI
> périmètre code de Phase C se réduit à **TROIS items** :
> 1. **[CŒUR] Fix P2-SIBLING-SYNC-SET** — `boot_storage_namespace` (`runtime.rs:2499`) et
>    `boot_feed_namespace` (`:2605`) n'appellent `start_sync` sur AUCUN bras → un coordinateur
>    qui rouvre son namespace persisté reste hors sync-set (ne broadcast pas ses écritures,
>    rejette les syncs entrants). Fix = pattern A4 `start_sync(Vec::new())` fail-fast au point
>    de fusion unique de chaque fonction + tests miroir CONTROL/GREEN de convergence 2-nœuds.
> 2. **[BLOQUANT — NOUVEAU, révélé par S3] Gate duress du `start_sync` sibling** — sous duress
>    le store iroh + coordinator DB sont les VRAIS (seule la keypair est un leurre) ; un
>    `start_sync` inconditionnel re-dialerait ≤5 vrais pairs persistés PAR doc et servirait le
>    vrai replica sous la clé leurre, **régressant la clôture DURESS-BOOT-LEAK (§15.1)**. Le fix
>    DOIT gater sous `IdentityMode::Duress` (miroir `gossip_publish_in_duress==Noop`) + tests
>    duress no-op. Collatéral : le project doc A4 (`:648`) est DÉJÀ inconditionnel → réconcilier
>    les 3 docs sous un prédicat duress unique (régression-free, cf. §5.2).
> 3. **[DOC-ONLY] Recalibration ~4 mentions « 0.98 » → 0.101** (dont une NON mécanique :
>    `http.rs:3213-3216` — vérifié `remote_info_iter` TOUJOURS absent en 1.0.1, reformuler le
>    fond, pas sed-la-version).
>
> Aucun Day-0 touché ; 0 bump wire SBFB tenu par construction ; iroh strictement seul. **Ce
> 3e PLAN-ADAPT pour cause « lettre pré-bump » est ATTENDU et structurel** (A et A2 l'étaient
> déjà pour la même raison), pas un signal méta nouveau. Aucune question PO bloquante
> (0 DESIGN-CONFLICT). G8 : 6 scans (S0 réalité-code / S1a OSS iroh-docs 0.101 vendored /
> S1b deps-CVE / S2 décisions historiques / S3 threat model / S4 wire) + 6 vérifications
> adversariales (0 REFUTED de conséquence sur le verdict ; 1 REFUTED matériel S4-#5 =
> 3e site de parse worker, réconcilié en NOTE Phase F).

---

## 1. Rappel de la lettre du plan (sprint81_plan.md:147-163)

Phase C « iroh-docs deep (wire + types iroh-base) ». **But** : adapter la surface docs aux
types iroh-base 0.100 + wire `EntrySignature → iroh::Signature` (0.99.1) ; « le vrai travail
de migration (pas un recompile) ». **Livrables** : `docs.rs:42-47,229,275,388-410`
(AuthorId/NamespaceId/Entry/DocTicket/Query/ShareMode/AddrInfoOptions/LiveEvent re-typés) ;
`node.rs:388-395` (`Docs::persistent/memory/spawn`) ; `runtime.rs:2479`
(`DocsNamespaceId::from([u8;32])` reconstruction raw-bytes) ; suppression actée des zombies
legacy-decode. **Delta tests** : +2..4 Rust (round-trip signature / types) − N zombies.
**T1** : alimente sous-test (1) doc-sync in-process + sous-test (4) parse `DocTicket` persisté
(colonne coordinator `doc_ticket`). **Gate/scope-cut** : 0 bump wire SBFB (JCS / `DOMAIN_*_V1`
/ `FEED_FORMAT_VERSION`) + vérifier stabilité du format string `DocTicket`.

**Le nœud du PLAN-ADAPT** : le libellé décrit une migration de types que le bump B a déjà
faite (les imports `docs.rs:42-47` tirent DÉJÀ tous ces types d'`iroh_docs` 0.101 et compilent
vert). La lettre n'a PAS anticipé (a) que le bump absorberait le re-typage, (b) que le carry
sibling sync-set serait le seul vrai code, (c) que ce fix ouvrirait une régression duress à
gater. Ci-dessous le vrai périmètre, evidence-adossé item par item.

---

## 2. Pourquoi PLAN-ADAPT — les items du libellé sont NO-OP compile-prouvés

Chaque verdict NO-OP est adossé à une lecture première-main sous le lock 0.101 (les 6 scans
convergent, 0 réfutation). **Réserve d'honnêteté portée des vérifications** : les verdicts
« compile-prouvés » héritent de la baseline Phase B verte ACQUISE (nextest 2028 Win 0-skip
@ `c899d54`), pas d'une recompilation indépendante dans ces scans — mais tous les
call-sites/types/imports cités ont été relus et existent dans du code du build vert.

| Livrable de la lettre | Verdict | Ancre ACTUELLE (post-bump) | Preuve |
|---|---|---|---|
| `docs.rs:42-47` re-typage AuthorId/NamespaceId/Entry/DocTicket/Query/ShareMode/AddrInfoOptions/LiveEvent | **NO-OP compile-prouvé** | `docs.rs:42-47` (imports 0.101) + `:446-450` (re-exports) | Imports tirent déjà d'`iroh_docs` 0.101 ; commentaires re-datés 0.101 en B (`:53`, `:156-161`, `:388-409`) |
| Wire `EntrySignature → iroh::Signature (0.99.1)` | **PHANTOM (0 site réel)** | — | `git grep EntrySignature\|AuthorPublicKey\|NamespacePublicKey\|SignedEntry -- crates/**/*.rs` = **0 hit** ; SBFB ne lit l'`Entry` iroh-docs que via `content_hash()` (`docs.rs:644,712,732`) |
| `node.rs:388-395` `Docs::persistent/memory/spawn` | **NO-OP** (node.rs INTOUCHÉ en B) | `node.rs:383-395` | Compile vert ; absent du `git stat c899d54` |
| `runtime.rs:2479` `DocsNamespaceId::from([u8;32])` | **NO-OP ; repère plan PÉRIMÉ** | sites réels **`:2522`** (storage) + **`:2627`** (feed) — `:2479` = `restore_browse_from_outbox` | Résout via `derive_more::From` owned (`keys.rs:343`) ; compile vert |
| Matcher `.contains("Replica not found")` | **NO-OP** (fermé par constat en B) | `runtime.rs:2538` (storage) / `:2633` (feed) | `store.rs:26` `#[error("Replica not found")]` byte-identique 0.98/0.101 ; 4 tests A2 verts sous 0.101 (`:4216/:4251/:4281/:4320`) |
| Suppression zombies legacy-decode | **NO-OP (N=0)** | — | `grep legacy_decode\|decode_legacy\|old_format` = 0 ; aucune fixture iroh versionnée n'existe ; le `− N zombies` de la lettre est VIDE |

**Conclusion §2** : sur ~6 « Livrables » + 5 carries de re-typage, aucun n'est du code à écrire.
Le bump Phase B les a absorbés. Les re-coder serait du travail mort. Phase C **vérifie** ces
surfaces (au run des tests existants), elle ne les re-type pas.

---

## 3. Le VRAI travail — item 1 : fix P2-SIBLING-SYNC-SET (CŒUR)

### 3.1 Le bug (confirmé première-main + upstream)

`git grep -n start_sync -- crates/nexus-shell-daemon/src/runtime.rs` → **un seul call-site
prod : `:2065`** (`open_project_doc_for_dispatch`). NI `boot_storage_namespace` (`:2499-2599`)
NI `boot_feed_namespace` (`:2605-2690`) n'appellent `start_sync` sur AUCUN de leurs 3 bras :
- **bras ticket-persisté** `Some(row) → open_doc(ns_id) → Some(doc)`, `row.doc_ticket=Some(t)`
  (storage `:2549-2562` / feed `:2644-2657`) : retourne `(doc, ticket_str)` **hors sync-set** ;
- **bras recreate** `None`/replica absente + **bras first-boot** `None`/pas de row M8 :
  `create_doc` + `share_write()` → entrent le sync-set en **side-effect fragile** (`doc_share`
  appelle inconditionnellement `start_sync(vec![])`, `iroh-docs api/actor.rs:405`).

Mécanisme 0.101 VÉRIFIÉ INCHANGÉ (vendored `iroh-docs-0.101.0`) : hors sync-set =
(a) `accept_request` renvoie `AbortReason::NotFound` → rejette tout sync entrant
(`engine/state.rs:96-97`) ; (b) broadcast `LocalInsert` gardé par `is_syncing` → n'émet
jamais (`engine/live.rs:713`) ; (c) `start_connect` abandonne. Seul `start_sync` insère dans
`SyncState` (`live.rs:408-413`, `state.insert`) puis re-dial les pairs persistés bornés
`PEERS_PER_DOC_CACHE_SIZE=5` (`store.rs:17`). Le feed est **network-visible** → le CONTROL
feed a une portée réelle (S75 PULL = mitigation partielle seulement).

Ce carry est **STATUÉ « Phase C FIXE »** par A4 (`fdb8ad7`) ET B (`c899d54`), même root-cause
que le fix project doc A4. Repères des bodies A4/B (`:2552-2564` / `:2647-2659`) sont
**légèrement périmés post-bump** ; utiliser les ancres ACTUELLES ci-dessous.

### 3.2 Le fix (pattern A4 `open_project_doc_for_dispatch`)

Patron à répliquer (`runtime.rs:2046-2070`, `start_sync` à `:2065`) :
```rust
project_doc.start_sync(Vec::new()).await.context(
    "failed to enter the project doc sync-set at boot \
     (coordinator would neither broadcast task: writes nor accept worker syncs)",
)?;
```
Insérer un chokepoint UNIQUE au **point de fusion** de chaque boot fn — après résolution du
match `(doc, ticket_str)` et AVANT le `Arc::new(doc)` de la construction du state :
- **storage** : entre `:2591` (`};` fin de match) et `:2593` (`Ok(StorageNamespaceState {`) ;
- **feed** : entre `:2683` (`};` fin de match) et `:2685` (`Ok(FeedSyncState {`).

`use anyhow::Context` déjà en scope module (`runtime.rs:36`). `start_sync(Vec::new())` est
**idempotent** sur un doc déjà-syncing (`docs.rs:408`) → sûr sur les bras recreate/first-boot
qui ont share_write. Il **DOIT être fail-fast** (`.context(...)?`, doctrine A2), jamais
warn-only : une entrée sync-set silencieusement ratée ré-ouvrirait la classe « perte
silencieuse » fermée en A2. Ne PAS régresser le discriminateur A2 (`:2536-2546` /
`:2631-2641`, `NotFound → recreate loud / autre Err → fail-fast`) : le `start_sync` s'ajoute
APRÈS l'obtention du doc, tous bras confondus.

**Fail-fast = pas de DoS-at-boot nouveau** (S3, caveat déchargé upstream) : `start_sync(vec![])`
est une op LOCALE (insert `SyncState` + merge pairs persistés) ; le dial réel `DirectJoin` est
SPAWNÉ non-awaité (`engine/live.rs:383`). Une erreur = échec acteur/store local, même classe
que corruption `open_doc`. Dégrader produirait un daemon silencieusement cassé (ni dispatch ni
sync), pire. GARDER la politique fail-fast A2/A4 pour storage ET feed.

### 3.3 Préservation de la tripwire CONTROL A4 (carry #7)

Le fix touche `boot_storage_namespace`/`boot_feed_namespace`, **PAS** le corps du CONTROL
`reopened_project_doc_without_start_sync_does_not_deliver` (`dispatch_loop.rs:557`, assertion
NÉGATIVE 8s « if this starts converging … recalibrate the A4 boot fix »). Ce CONTROL DOIT
rester NON-convergent sous 0.101 (tripwire re-joué PASS en B) ; le GREEN #5
`boot_path_reenters_sync_set_and_delivers_after_reopen` (`:643`) reste convergent. **Si le
CONTROL flippe → STOP + recalibrage de la prémisse A4 avant tout autre travail.**

---

## 4. Le VRAI travail — item 2 : gate duress du `start_sync` sibling (BLOQUANT)

Révélé par S3 (scan + vérif adversariale, **14/14 claims CONFIRMED**), absent du libellé plan.
C'est le point le plus important à faire remonter : appliquer le fix §3 **inconditionnellement
régresserait une clôture sécurité Day-0**.

### 4.1 Modèle duress = clé leurre, MÊME store/DB (décisif)

`run_unlock_and_export_env` (`nexus-launcher/src/unlock.rs:194-201`) : sous duress, on exporte
UNIQUEMENT le secret leurre + `SBFB_IDENTITY_MODE_ENV="duress"`. **Aucun swap de `SBFB_HOME`
/ data-dir.** Côté daemon `runtime.rs:344` : `iroh_data_dir = opts.paths.root/iroh`,
INDÉPENDANT de `identity_mode`. Donc sous duress : **store iroh (redb) + coordinator DB = les
VRAIS**, seule la keypair du node est un leurre. C'est exactement pourquoi DURESS-BOOT-LEAK
(§15.1, `THREAT_MODEL.md:979`) a gaté le republish feed + seed re-announce + boot seed driver.

### 4.2 Le fix inconditionnel régresse la clôture

Sous duress, `start_sync(vec![])` inconditionnel sur feed+storage : re-dial ≤5 VRAIS pairs
persistés PAR doc (issus du redb partagé) sous la clé LEURRE en annonçant le VRAI namespace id
→ corrélation immédiate decoy↔vrai data root ; **ET** réconcilie/SERT le contenu réel du
replica feed/storage déjà persisté à tout pair qui sync. Le no-op republish
(`feed_sync_for_republish=None`, `runtime.rs:768-775`) NE COUVRE PAS ça : il bloque seulement
les nouveaux writes SQLite→docs, PAS la réconciliation du replica déjà persisté (les 2 chemins
sont orthogonaux). Statu quo AVANT fix (reopen sans start_sync) = 0 dial / 0 serve → le fix
inconditionnel AJOUTE la fuite.

### 4.3 Exigence Phase C

Gater le `start_sync` sibling sous `IdentityMode::Duress` : skip (miroir
`gossip_publish_in_duress(mode)==Noop` / `noop_identity.rs:83-88`). Le prédicat duress doit
être passé aux boot fns privées (changement de signature minimal : 1 param `identity_mode` ou
un bool `should_enter_sync_set`, calculé aux call-sites `:696` storage / `:738` feed). Tests
duress no-op (analogues `reannounce_seeds_noop_in_duress`) : `boot_{storage,feed}_namespace`
sous duress = **0 entrée sync-set (0 dial, 0 serve)**. C'est le pattern duress canonisé
(`noop_identity.rs`) — un route non-gaté est traité comme green-flag reviewer
(module doc `noop_identity.rs:39-41`).

---

## 5. Le VRAI travail — item 3 (doc-only) + réconciliations

### 5.1 Recalibration mentions « 0.98 » → 0.101 (carry #6)

Prose seule, 0 wire. Sites (ancres confirmées) :
- `crates/nexus-core-rs/src/doc_sync.rs:13/17/18/33` (« installed iroh-docs 0.98 source » +
  numéros `live.rs:711-718`/`:409-414`/`api.rs:220-225`) → recalibrer 0.101 (docs.rs a déjà
  recalibré `live.rs:408-414`/`:713`).
- `dispatch_loop.rs:547` (doc-comment du CONTROL, « iroh-docs 0.98: only start_sync inserts
  into SyncState ») → re-dater le COMMENTAIRE seulement, **ne pas toucher le corps du CONTROL**.
- `runtime.rs:2019-2045` (header `open_project_doc_for_dispatch`, dit lui-même « Recalibrate
  the cited internals against iroh-docs 0.101 at the Phase B/C bump ») → recalibrer `414→408-414`,
  `714→713`.
- `crates/nexus-core-rs/Cargo.toml:42-46` (« matching iroh 0.98 PkarrRelayClient::new
  signature ») → stale post-fix pkarr Phase B (3-arg 1.0.1) ; `url` reste dep directe requise.

**NON mécanique — vérification factuelle obligatoire** (S1a-vérif) : `http.rs:3213-3216`
affirme « remote_info_iter landed post-0.98 ». Grep dans `iroh-1.0.1` : `remote_info_iter`
**TOUJOURS ABSENT** (seul `remote_info(EndpointId)` singulier, `endpoint.rs:1620`). Un sed
aveugle produirait un commentaire FAUX — reformuler le FOND, pas la version. Idem re-checker
`age_witness.rs:6/21`, `gossip.rs:740`, `pkarr_resolver.rs:95`, `tls_pinning.rs:32` avant
reformulation (ces mentions descriptives runtime sont HORS surface docs, mentionner sans
bloquer).

### 5.2 Réconciliation project doc A4 (ADAPT, recommandé régression-free)

Collatéral révélé par S3 : `open_project_doc_for_dispatch` (`:648`, `start_sync` `:2065`)
appelle `start_sync(vec![])` **DÉJÀ SANS gate duress** → la fuite boot-re-dial du vrai doc
task/result sous la clé leurre a **probablement déjà été livrée en A4**. Phase C touche le même
chemin sync-set : **recommandation = gater les 3 docs (project + storage + feed) sous un
prédicat duress UNIQUE en C**. Argument régression-free (vérif S3) : aucun dispatch réel n'a
lieu sous duress (`task_dispatch_in_duress => 503`, `noop_identity.rs:99-103`), donc un project
doc hors sync-set en duress ne casse rien fonctionnellement. Le GREEN #5 (mode normal) reste
vert (le gate `if !duress` exécute start_sync) ; le CONTROL #4 utilise `open_doc` direct, non
touché. **Alternative de repli** (si le plan veut borner le scope) : documenter le task doc
hors-scope + flag orchestrateur. Reco du préflight : gater les 3 (cohérence + ferme la fuite
A4). Ce n'est PAS un DESIGN-CONFLICT (cohérent avec DURESS-BOOT-LEAK Day-0), donc pas une
question PO bloquante.

### 5.3 Test one-shot « ticket DocTicket parse sous le nouveau lock » (carry #2)

**Point wire tranché OUI** (S1a + S4, byte-diff `ticket.rs` 0.98↔0.101 + hexdump test
identique) : un `DocTicket` string persisté sous 0.98 RE-PARSE sous 0.101 — struct /
`TicketWireFormat::Variant0` / `KIND="doc"` / postcard body / `Capability` / `EndpointAddr`
byte-identiques ; seuls 4 noms de méthode du trait `Ticket` renommés
(`to_bytes→encode_bytes`…), **invisibles à SBFB** qui n'utilise que Display (`.to_string()`)
+ FromStr (`.parse()`). Résidu honnête : `Vec<EndpointAddr>` (ex-`Vec<NodeAddr>`) avec ≥1
`Relay(RelayUrl)`/`Ip(SocketAddr)` — serde non byte-diffé, risque faible (enum + dérives +
ordre identiques).

**Design du test (S4 Option A, hermétique)** : round-trip sous le lock 0.101, dans
`nexus-core-rs` — `create_doc → doc.share_write()` (produit un `DocTicket` 0.101) →
`let s = ticket.to_string()` (== ce que la colonne `doc_ticket` persiste) →
`DocsTicket::from_str(&s)` (== ce que `body.ticket.parse()` fait aux endpoints JOIN) →
assert `parsed.capability.id() == doc.id()` (NamespaceId préservé) + `parsed.to_string() == s`
(idempotence). **PAS de fixture 0.98 genuine** (aucun littéral committé
`grep '"doc[a-z2-7]{40,}"' crates/` = 0 ; crate 0.98 hors lock ; garde un non-scénario
pre-launch). +1..2 tests. La mécanique `to_string().parse()` est déjà prouvée verte
incidemment (`feed_sync.rs:1048`) ; l'ancre `docs.rs:563` du scan S4 est un test 2-nœuds
networked, PAS un round-trip string — écrire un test hermétique dédié.

**DocTicket hostile en DB = pas de panic** (S3-Q3) : le boot ne parse JAMAIS `row.doc_ticket`
(reconstruit via `namespace_id` bytes) ; les consommateurs parsent en `match …parse()` gardé
(erreur HTTP, pas panic). Ajouter une variante « string hostile → assert `Err` » (pas de fix
code, seulement le test).

---

## 6. Statut EXPLICITE des 7 carries B → C

| # | Carry | Statut Phase C | Action |
|---|-------|----------------|--------|
| 1 | **P2-SIBLING-SYNC-SET** (A4, « Phase C FIXE ») | **RETENU — CŒUR, OUVERT** | `start_sync(Vec::new())` fail-fast dans les 2 boot fns au point de fusion (storage `@2591→2593`, feed `@2683→2685`), gaté `!Duress`, discriminateur A2 préservé + CONTROL/GREEN convergence (§3, §7) |
| 2 | Test one-shot « ticket string 0.98 parse » | **REQUALIFIÉ** → round-trip hermétique 0.101 (Option A), PAS fixture 0.98 | +1..2 tests mint→to_string→from_str (§5.3) ; couvre aussi le parse worker-boot (même type `DocsTicket`) |
| 3 | RE-SCOPER `DocsNamespaceId::from` | **NO-OP compile-prouvé — retiré du code** | Seule correction d'ancre plan (`:2479` périmé → `:2522`/`:2627`) ; aucune reconstruction à réécrire |
| 4 | Matcher `.contains("Replica not found")` re-calibrage | **NO-OP — fermé par constat en B** | Constat SUFFIT au body ; les 4 tests A2 verts sous 0.101 couvrent le vif à travers irpc 0.17 → pas de test dédié |
| 5 | Zombies legacy-decode | **NO-OP (N=0)** | Confirmer no-op au body ; le `− N zombies` de la lettre est vide |
| 6 | Mentions versions stale routées C | **RETENU — doc-only** | Sweep §5.1 avec re-check factuel `http.rs` remote_info_iter (NON mécanique) |
| 7 | CONTROL A4 non-convergent sous 0.101 = tripwire | **CONTRAINTE — préservé par construction** | Le fix sibling ne touche pas le CONTROL ; s'il flippe → STOP (§3.3) |

**Réconciliations sortantes actées en plus** (au-delà des 7 carries) : gate duress project doc
A4 (§5.2, ADAPT recommandé régression-free) ; corrections d'ancres bodies A4/B pour le plan
(§8).

---

## 7. Plan de tests concret (delta recalculé)

Les tests `_persistent_reopen` existants (`runtime.rs:4184`/`:4196`) ne vérifient QUE « rouvre
sans erreur » (+ `feed_handle.is_some()`) → **ils PASSENT même avec le bug** ; une garde neuve
est indispensable.

**Contrainte de greffe** : `boot_storage_namespace`/`boot_feed_namespace` sont des `async fn`
PRIVÉES au module runtime ; les helpers convergence (`boot_persistent_coordinator`, `seed_addr`,
`await_exact_key`, `await_neighbor`, `addr_of`) vivent UNIQUEMENT dans le module test de
`dispatch_loop.rs` (`git grep` dans runtime.rs test module = 0). **Deux voies** :
- **Voie 1 (recommandée, friction minimale)** : rendre les 2 boot fns `pub(crate)`, écrire les
  tests miroir DANS `dispatch_loop.rs` à côté de #4/#5, réutilisant `boot_persistent_coordinator`
  + `seed_addr` + `await_exact_key` (setup additionnel : `CoordinatorDb::open_in_memory()` +
  author, appeler `boot_storage_namespace(&docs, &db, "sbfb-ideas", author)`, enroller B via
  `state.ticket`, restart, re-boot bras reopen, write via `state.doc`, assert convergence).
- **Voie 2** : hoister les helpers dans un `#[cfg(test)] pub(crate) mod test_support` partagé
  (ferme la dette WS-3/PD-5 mais plus de surface d'édition). Si Voie 1 → WS-3/PD-5 reste dette/K.

**Tests à écrire (in-process `create_node`, hermétiques, jamais réseau/store réel)** :

| Test | Type | Assertion |
|---|---|---|
| `boot_storage_namespace` reopen sans start_sync ne délivre pas | CONTROL (rouge-avant-vert) | doc rouvert hors sync-set → `!await_exact_key` (non-livraison) |
| `boot_feed_namespace` reopen sans start_sync ne délivre pas | CONTROL | idem, feed |
| `boot_storage_namespace` re-entre sync-set + converge après reopen | GREEN | après fix → convergence, ns_id inchangé |
| `boot_feed_namespace` re-entre sync-set + converge après reopen | GREEN | idem, feed |
| `boot_storage_namespace` sous duress = 0 entrée sync-set | duress no-op | mode Duress → pas de start_sync, 0 dial/serve |
| `boot_feed_namespace` sous duress = 0 entrée sync-set | duress no-op | idem, feed |
| DocTicket round-trip mint→to_string→from_str (+ variante hostile) | GREEN + Err | `nexus-core-rs` ; NamespaceId préservé + idempotence ; string hostile → `Err` |

**Delta tests réaliste : +6..8 Rust, −0 zombies.** (2 CONTROL sibling + 2 GREEN sibling +
2 duress-noop + 1..2 ticket round-trip/hostile.) Le libellé plan « +2..4 − N zombies »
**sous-estime** le fix sibling + le gate duress et **sur-estime** les suppressions (N=0). Delta
à acter au body du commit.

---

## 8. Corrections d'ancres pour le plan / le body (repères périmés)

Les bodies A4 (`fdb8ad7`) et B (`c899d54`) citent des repères pré-bump ; le préflight B §8.1(d)
aussi. **Utiliser les ancres ACTUELLES (git grep sous lock 0.101), pas les repères commit-body** :
- `runtime.rs:2479` (plan, `DocsNamespaceId::from`) = **PÉRIMÉ** (`restore_browse_from_outbox`)
  → sites réels **`:2522`** (storage) + **`:2627`** (feed).
- Siblings bras ticket (bodies A4/B `:2552-2564` / `:2647-2659`) = post-bump **`:2549-2562`
  (storage)** / **`:2644-2657` (feed)** ; fns `boot_storage_namespace` `:2499` /
  `boot_feed_namespace` `:2605` ; discriminateur A2 `:2536-2546` / `:2631-2641` ; chokepoint
  d'insertion `@2591→2593` / `@2683→2685`.
- Préflight B §8.1(d) `DocsNamespaceId::from` `:2527/:2630` = légèrement off → `:2522/:2627`.

---

## 9. Invariants & Day-0 (tenus)

- **Bisectabilité (précédent S32 `90aff27`)** : le fix fonctionnel appartient EXCLUSIVEMENT à
  Phase C, laissé non-fixé en B intentionnellement (bump = Cargo.toml+lock seuls). Ne pas
  rétro-amender B.
- **0 bump wire SBFB** : `start_sync` / entrée sync-set = état interne moteur iroh-docs, 0 JCS
  / `DOMAIN_*_V1` / `FEED_FORMAT_VERSION`. Les 13 `*_FORMAT_VERSION` + 23 `DOMAIN_*_V1` vivent
  dans `nexus-core-rs`, hors du graphe de modif (S4 EXECUTE). Le format string `DocTicket` est
  celui d'iroh-docs (compat-upgrade), pas un wire SBFB — stabilité vérifiée par le round-trip
  §5.3. Le fix ne modifie NI la string servie au front (`storage_api.rs:486`/`feed_sync.rs:686`,
  byte-identique) NI les endpoints de parse JOIN → **pas de nouvelle frontière docs-contract**
  (test-acteur §6.12 : aucune étiquette requise).
- **iroh STRICTEMENT SEUL (D7)** : 0 dep ajoutée (S1b confirmé — lock inchangé, `start_sync`
  api.rs:437 déjà présente, cap 5). Deps Phase B strictement inchangées ; RustSEC propre ;
  aucune 1.0.2/0.102 upstream (veille standing au code-freeze, `plan:145`).
- **Tests hermétiques uniquement** : in-process `create_node`, jamais store réel Win/Mac/VPS
  avant Phase F PASS (auto-migration redb one-way ; `migrate_redb_v2_tuples.rs` = Phase F).
- **P2-PROJECT-DOC-SELECTOR = dette/K, PAS C** : `list_docs().first()` non-déterministe
  affecte `open_project_doc_for_dispatch` (`:2053`) mais PAS les siblings (sélection par
  `namespace_id` M8 déterministe, `:2511`/`:2616`). Ne pas élargir C à la persistance du
  namespace id.

---

## 10. Carries sortants (créés / re-routés C → G, F, K)

1. **Phase G (THREAT_MODEL.md)** : (a) ligne §15.x « re-dial boot des pairs persistés »
   couvrant les 3 docs (project+storage+feed), bornée `PEERS_PER_DOC_CACHE_SIZE=5`/doc
   (worst-case **≤15 pairs = ≤5/doc**, plafond cache non garanti — `Ok(None)` → 0), trust model
   pkarr inchangé, aucune frontière d'admission nouvelle — subsume la note A4 pending ;
   (b) ligne miroir DURESS-BOOT-LEAK pour la mitigation duress codée en C (le CODE atterrit en
   C, la DOC en G) ; (c) delta DNS pkarr déjà routé carry B → note THREAT_MODEL G.
2. **Phase F (NOTE, non-bloquante)** : le worker RE-PARSE un ticket persisté au boot
   (`worker-core/src/engine/runtime.rs:360`, `p.tasks_doc_ticket` issu de `allowlist.list_enabled()`
   SQLite) — REFUTED du scan S4 réconcilié : la sûreté boot F vient de l'import worker
   **NON-FATAL** (Err → warn + continue, `:350-351/387-390`), pas de « jamais re-parsé ». Un
   ticket 0.98 avec ≥1 addr non re-décodable dégrade (projet skippé), ne casse pas le boot.
   Surveiller le diagnostic « failed to import task doc » si l'acceptance F boote un worker sur
   un store/allowlist reporté d'un build 0.98 (jonction risque `Vec<EndpointAddr>`).
3. **Phase K** : P2-PROJECT-DOC-SELECTOR (inchangé, dette/K) ; hoisting helpers WS-3/PD-5 si
   Voie 1 retenue (§7) ; P2-PROJECT-DOC-SELECTOR reste hors C.
4. **Veille continue** : re-jouer crates.io (1.0.2 ?) + RustSEC avant le push live (règle plan
   inchangée) ; note veille 1.0.2/pkarr.

---

## 11. Risques résiduels

- **Signature des boot fns** : le gate duress nécessite de passer `identity_mode` (ou un bool)
  aux 2 boot fns privées — changement minimal mais à cadrer au plan (par-param vs par-call-site).
- **Greffe des tests** : Voie 1 (pub(crate)) vs Voie 2 (hoist) — trancher au plan ; Voie 1
  recommandée, laisse WS-3/PD-5 en K.
- **CONTROL A4 tripwire** : doit rester non-convergent sous 0.101 ; s'il flippe au run post-fix
  → STOP + recalibrage prémisse A4 (§3.3).
- **Résidu `Vec<EndpointAddr>` serde** : non byte-diffé pour `Relay`/`Ip` addrs (0.98→0.101) ;
  risque faible, le round-trip §5.3 ferme empiriquement le cas 0.101 ; le cas 0.98→0.101 genuine
  reste un non-scénario pre-launch (route NOTE Phase F).
- **Réserve d'honnêteté NO-OP** : les verdicts « compile-prouvés » héritent de la baseline B
  verte (nextest 2028 Win), non re-compilée dans ces scans ; inférence solide (call-sites relus)
  mais leg 'compile' héritée.

---

## 12. Commit shape (indicatif)

`fix(daemon): Sprint 81 Phase C — coordinateur entre storage+feed dans le sync-set au boot,
gate duress (P2-SIBLING-SYNC-SET, 0-bump wire)` — body : fix `start_sync(Vec::new())` fail-fast
`boot_storage_namespace`/`boot_feed_namespace` (ancres `@2591→2593`/`@2683→2685`, discriminateur
A2 préservé) + gate duress des 3 docs (project A4 réconcilié, miroir DURESS-BOOT-LEAK §15.1) +
tests CONTROL/GREEN convergence 2-nœuds + duress-noop + DocTicket round-trip 0.101 + recalibration
doc-only mentions 0.98→0.101 (dont `http.rs` remote_info_iter reformulé, TOUJOURS absent 1.0.1) +
NO-OP statués (re-typage/DocsNamespaceId::from/node.rs/matcher/zombies = absorbés par bump B) +
CONTROL A4 non-convergent préservé + carries sortants G (THREAT_MODEL re-dial boot + duress)/F
(worker parse non-fatal)/K (WS-3/PD-5, P2-PROJECT-DOC-SELECTOR) + delta tests +6..8 Rust, −0
zombies + 0 bump wire SBFB (start_sync = état interne iroh-docs) + iroh strictement seul.
