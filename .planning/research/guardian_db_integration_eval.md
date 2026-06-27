# Évaluation GuardianDB × SBFB — protocole / Factory / apps + package-vs-code + upgrade iroh

> Recherche hors-sprint, 2026-06-27 (session S80 en cours, recherche pure — 0 code de sprint).
> 3 workflows ultracode parallèles, 33 agents Opus 4.8 1M, ~3,5M tokens, vérification adversariale à chaque étage.
> Runs : `wf_03124107-dd9` (usage 3 axes), `wf_ccea7a9b-6f8` (package-vs-code), `wf_7f37fec1-5a5` (upgrade-first).
> Cibles : guardian-db 0.17.0 (clone `873c5b6`, MIT OR Apache-2.0) vs SBFB tip `e036f65` (pin iroh 0.98 / rust 1.94 / AGPL-3.0).

## Qu'est GuardianDB
Ré-implémentation Rust d'OrbitDB (local-first, CRDT, content-addressé BLAKE3) bâtie nativement sur **iroh 1.0** (docs 0.101 / gossip 0.101 / blobs 0.103), edition 2024, MSRV 1.95, **mono-auteur, pré-1.0, repo = 1 commit squashé**. Trois stores : EventLogStore (Merkle-DAG `Entry`+LamportClock, réplication par échange de heads gossip maison) ; KeyValueStore + DocumentStore (**adossés iroh-docs** LWW range-sync — le « Willow » du README est un faux ami, aucun crate Willow dans le lock). ODM optionnel (Mongoose-like, `query.rs`/`update.rs` PURS iroh-indépendants). SDK TypeScript = **MemoryTransport seul** (éphémère, 0 réseau, 0 persistance, 0 binding WASM réel).

## Découverte centrale
**SBFB possède déjà ~80 % de GuardianDB.** `storage_api.rs` EST un mini-OrbitDB sur iroh-docs (namespace, LWW par `(key,author)`, préfixe `votes/` multi-auteur, tombstones, ticket/join, compteur version). La couche KV/Doc de GuardianDB tourne sur **le même moteur iroh-docs**. Delta réplication ≈ NUL. **Seul apport neuf = l'ODM** (portable ~3-5 j verbatim).

## Deux bloqueurs INDÉPENDANTS à adopter le moteur sur le protocole
1. **Unification iroh** — deux majeurs iroh (0.98/1.0) co-compilent (le lock SBFB porte déjà `rand`×3, `ed25519-dalek`×2) MAIS ne partagent pas d'Endpoint/Router/node. Résolu seulement par l'upgrade SBFB→1.0.
2. **Canonicalisation / vérifiabilité (ORTHOGONAL, non résolu par l'upgrade)** — GuardianDB = postcard/CBOR **sans signature par-entrée** ; SBFB = JCS RFC 8785 + Ed25519 par-entrée + `DOMAIN_*_V1`. Un record GuardianDB n'est pas cross-vérifiable par la chaîne de provenance SBFB. Rédhibitoire pour « source vérifiable » même post-upgrade.

## Défauts de correction GuardianDB 0.17 (lus dans le clone réel — wf1)
`unimplemented!()` dans des impls de trait **publiques** (`mod.rs:571/766/1247`) ; ACL fallback **silencieux** en write=`['*']` (`core.rs:1646-1709`) ; `can_append` (dispatch dyn) **saute `verify_identity`** (`acl_iroh.rs:381-402`) ; écriture JSON / relecture postcard incohérente (`entry.rs:222-247` vs `252-264`) ; **pas de signature par-entrée** → usurpation d'auteur ; `ed25519-dalek 3.0.0-rc.0` ; identité Ed25519 en clair (`identity.json`) ; tire **`serde_cbor 0.11.2` NON-MAINTENU (advisory RUSTSEC)** = régression de la mitigation threat-model « version pinnée + cargo-audit propre ». 219 `.unwrap()` + 55 `.expect()` hors-test.

## Package vs code (matrice O1–O4)
- **O1 crate as-is** : CONDITIONAL. Compile (méta-bloqueur : `cargo build` réel non lancé → collision `-sys`/`links` QUIC/TLS/relay/zstd-sys inconnue). Double-node. 5 `unimplemented!` publics → peut s'effondrer en fork (dep non patchable). Bus-factor HAUT (upstream ~1 minor/mois). Fit Factory/apps OK, protocole NUL.
- **O2 fork + downgrade 0.98** : downgrade iroh **TRIVIAL ~1 sem** (prouvé par `node.rs` SBFB qui appelle la même API sur 0.98 ; Willow/range-sync EST en docs 0.98) ; total **~4-6 sem** (seam d'injection d'Endpoint inexistant à créer + 5 stubs + transitif). **Seule voie node partagé.** Bus-factor BAS (snapshot figé possédé).
- **O3 extraire le cœur** : sous-ensemble pur (ODM+derive+serializer+identity+lamport+acl_simple) ~3-5 j verbatim ; log CRDT via shim `BlobStore` 2-méthodes ~1 sem (mapping 1:1 `blobs.rs:85/103`) ; **kv/document/base store = iroh-docs-natifs = réécriture pas extraction** (2-3 sem cœur / 5-8 avec stores).
- **O4 sidecar** : zéro conflit version, MSRV/SLSA intacts, mais ~5-8 j net-new (aucun binaire serveur turnkey) + tax IPC non chiffrée + treadmill upstream.
- **Licence** : non-bloquante, uniforme. MIT/Apache → AGPL OK (prendre bras MIT + notice `THIRD-PARTY-NOTICES`).

## Upgrade-first (réponse à « tout mettre à jour + iroh de toute façon »)
Prémisse **vraie à moitié** : ~15 deps feuilles convergent gratis ; iroh 0.98→1.0 n'est PAS routinier (décision Day-0 gelée, R-iroh-audit P0). Conclusion « donc GuardianDB drop-in » = **inversion logique** (l'upgrade se justifie seul, le drop-in est conséquence). **MAIS l'upgrade SE justifie seul = GO conditionnel** :
- forcing functions : `presets::N0` **coupe ses relais publics 0.9x au 2026-09-30** ; carry P2-AUDIT-2 (transitives pré-release) ne se résout que par l'upgrade ; 0.98 perd sa maintenance dès 1.0.
- blast-radius **CONTENU** dans `nexus-core-rs` (l'alarme « 153 call-sites » est un fantôme : rename Endpoint Takeover = iroh 0.94, AVANT le pin ; SBFB tourne déjà post-rename). Vrai travail = migration **iroh-docs** (wire `EntrySignature→iroh::Signature` + redb 2→4 on-disk) + 3 surfaces fragiles (`shard.rs` QUIC multipath, `pkarr_resolver`, `seed_protocol`) + bump MSRV 1.94→1.95. **~1 sprint dédié, bisectable.** Viser **1.0.x** (pas la `.0` de 12 j).
- **NE débloque PAS** : node partagé (dépend de l'API GuardianDB, jamais lue), ni le pilote public (R-iroh-audit P0 inchangé).
- risques : régression réseau silencieuse sur `shard.rs` (déjà PROVISIONAL/RIG-ABSENT) ; re-acceptance LIVE S75/S76 + **ancre VPS** (wire iroh-docs re-signe l'enveloppe du feed) ; `serde_cbor` = contribution GuardianDB non absorbée par l'upgrade.

## Recommandation
1. **Upgrade iroh 1.0** dans un **sprint dédié, sur ses propres mérites** (EOL relais 2026-09-30), viser 1.0.x. Garde-fous bloquants : `cargo build` dual-platform vert (~2370 tests) + `multi_daemon` vert sur **vrai rig 2-nœuds** + re-jeu acceptances S75/S76 + ancre VPS (artefact T2 JSON PASS).
2. **NE PAS adopter le moteur GuardianDB** (aucun besoin câblé `grep crdt|lamport|odm`=0 ; défauts correction/sécu ; interop JCS non résolue ; bus-factor 1 pré-1.0). NO par défaut.
3. **Récolter l'ODM** (code MIT) dans `storage_api.rs` SI besoin de requêtes riches / collections typées avéré.
4. **Apps** : meilleur ratio = **lever l'allowlist `REPLICATED_APPS`** sur le chemin iroh-docs existant (0 dep, 0 conflit).
5. **Factory** : durabiliser `action_log`/`chat_sessions` en **JSONL signé** via `nexus_core_rs::canonical_bytes` (déjà câblé), pas GuardianDB.

**Ordre imposé** : nommer un besoin → (si dep) SPIKE `cargo build` coexistence → **sprint upgrade iroh SEUL** → **go/no-go GuardianDB séparé et postérieur** (lire d'abord son `src/lib.rs` : accepte-t-il un Endpoint externe ou self-spawn ?). **Ne jamais bundler iroh + GuardianDB.**

## Décisions PO
1. Rouvrir « iroh 0.98 Day-0 » → OUI mais cadré (Day-0 prévoyait « évaluer 1.0 Gate 1 » ; forcing functions arrivées). Acter transition avant 2026-09-30.
2. L'upgrade ne franchit PAS Gate 1 (R-iroh-audit P0 inchangé, 0 audit tiers iroh 1.0) — prérequis, pas franchissement. Pilote reste fermé.
3. GuardianDB = décision d'adoption SÉPARÉE, NO par défaut, postérieure à iroh-1.0-vert + lecture API ; trancher `serde_cbor` (RUSTSEC) avant tout merge.
4. Ne pas laisser GuardianDB servir de prétexte à avancer un risque infra.

## Récolte chirurgicale pour le PROTOCOLE (wf4, `wf_cfa08123-8c8`, 6 agents)
**Code à porter ≈ 0.** Le bénéfice = la comparaison a révélé 2 améliorations du protocole SBFB lui-même + leçons conditionnelles. 7 vraies briques sur 25 candidates.

**🔴 BUG LATENT CONFIRMÉ (vérifié main-thread, pas seulement agent) — fold materializer non déterministe :**
- `public_feed.seq = AUTOINCREMENT` LOCAL (`db.rs:158`) ; `materialize_full` folde par `ORDER BY seq ASC` = ordre d'arrivée (`feed_materializer.rs:96-100`, `db.rs:1259`) ; `apply()` écrase `latest_release_hash` **sans garde timestamp** (`feed_materializer.rs:54-58`) ; ingest assigne seq à l'arrivée (`feed_sync.rs:358,369`) ; `verify_entry` **ne vérifie pas prev_hash** (`public_feed.rs:588-590`), chemin incrémental ne réordonne pas.
- → deux nœuds ingérant R1/R2 du même auteur en ordres opposés → `latest_release_hash` divergent (`PublicRegistryView` non byte-identique cross-nœud). Hoquet de cohérence éventuelle (rebuild à froid par préfixe `feed/{author}/{seq}` guérit) MAIS fenêtre LIVE réelle, invisible aux tests `multi_daemon` env-bloqués. `materialize_verified` (verify_chain) existe mais pas sur le chemin live.
- **Fix 0-bump** : folder APRÈS `verify_chain`, tri topo `prev_hash`→`entry_hash` puis tie-break `(timestamp, author_pubkey, entry_hash)` = la leçon `no_zeroes` guardian (`entry.rs:449-466`) à la façon SBFB + test cross-nœud ordres opposés. **Seul item à tracker en dette/fix court terme.**

**Liste de récolte** (TAKE code / LEARN design / LEAVE) :
1. Ordre total déterministe (LEARN→fix ci-dessus) — sans condition, maintenant.
2. Pagination app-storage `limit`/`offset`+ordre stable (TAKE le contrat, pas le trait `Datastore`) — `ListQuery` n'a que `prefix` (`storage_api.rs:82-86`), retour O(N) non borné = soft-DoS. Sans condition.
3. Store = manifeste signé content-adressé (`db_manifest.rs`) + `can_append` gate (LEARN) — **seulement si on lève `REPLICATED_APPS`** ; `StoreManifest{app_id,namespace_id,store_type,write_policy}` en `canonical_bytes` sous `DOMAIN_STORE_MANIFEST_V1` additif.
4. Capability par ticket d'écriture ALPN-gated (LEARN, lourd) — idem + besoin métier.
5. Moteur de prédicats `query.rs` `$eq/$gt/$in/$and` (TAKE mais **aucun consommateur** → réserve).
6. Index secondaire JCS (LEARN étroit).

**Pépite JCS** : `query/update.rs` préservent le type entier (`1025`≠`1025.0`, `update.rs:159`) — critique JCS (un float casserait la signature). À garder si portage ODM.

**Déjà dans SBFB, plus fort (ne pas reprendre)** : log signé par-entrée (guardian EventLog ne signe pas), adresse content-adressée blob-serve, ticket=droit-de-répliquer, tombstones LWW, anti-entropie tip-par-auteur+range-sync, keystore Argon2id+AES-GCM (guardian = SecretKey en clair), capability révocable/expirable+nonce (ledger seed-invite ; guardian sans revoke/expiry).

**Ne PAS importer** : validateur schéma runtime strict (conflit raw-op gelé), `#[derive(Model)]`, `$inc` répliqué, merge CRDT-DAG (anti-objectif vs linéaire-par-auteur+rejet-fork), LamportClock (superseded par fix #1), ACL write=* silencieux, manifeste CBOR non signé, serde_cbor, tout type iroh guardian.

**Séquencement** : #1+#2 maintenant (internes, 0-bump, sans iroh) → triptyque #3/#4 quand `REPLICATED_APPS` est levé → `query.rs` à l'apparition d'un consommateur.

## Caveat de confiance
Internes GuardianDB **ancrés** : wf1 a lu le clone réel (citations `entry.rs`, `mod.rs`...). wf2/wf3 ont noté « GuardianDB absent de la machine » (agents pointés registry/checkout, pas le clone scratchpad) → quelques faits wf2/wf3-spécifiques = lectures de cartes. Concordance inter-rapports forte. **Avant code d'adoption : checkout/vendor GuardianDB + lire `src/lib.rs`** (Endpoint externe vs self-spawn = pivot du verdict « node partagé »).
