# Sprint 81 Phase B — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT.** Le CŒUR de la lettre (« cargo build --workspace vert sous les 4
> pins D1 avec pkarr_resolver.rs comme unique correction de code ») est **CONFIRMÉ par sonde
> de compilation directe** contre les pins réels — la tension centrale « B mord-il sur
> C/D/E ? » est TRANCHÉE : NON, aucun découpage à renégocier (le finding contraire S4-1 est
> REFUTED par compile). Mais la LETTRE est corrigée sur TROIS points concrets, tous
> évidence-adossés, aucun ne touchant un Day-0 :
> 1. **Le fix pkarr n'est PAS un rename pur** : `PkarrRelayClient::new` exige un **3e argument
>    `DnsResolver`** en 1.0.1 (seule vraie cassure compile, E0061) ; le rename
>    `CaRootsConfig→CaTlsConfig` est un alias déprécié (soft) rendu bloquant par le gate
>    `clippy -D warnings` seulement.
> 2. **`rust-version = "1.85"` (Cargo.toml:24) devient mensongère** (pile iroh = 1.91) →
>    bump `"1.91"` dans le même commit (application MÉCANIQUE de D6, absent des livrables).
> 3. **Fenêtre one-way redb 2→4 SANS GARDE dès le commit B** : tout boot du daemon sur le
>    store dev réel (opération ROUTINE — le rig b3 live boote ce store par design) déclenche
>    l'auto-migration iroh-docs AVANT la validation sur COPIE (F) et le snapshot (H), violant
>    l'ordre D3 cond.3/4 → le tar snapshot est AVANCÉ au commit B + règle de boot explicite.
>
> Carries statués : **A2 (matcher) = fermable en B par constat, 0 code** (Display byte-identique
> 0.101, `open()` hardcode `Ok(Some)`) ; **A4 (sibling sync-set) = B STATUE au body, C FIXE**
> (mécanisme 0.101 vérifié inchangé ; un fix fonctionnel dans le commit de bump violerait la
> bisectabilité — précédent S32 `90aff27` : bump = Cargo.toml+lock SEULS) ; **CONTROL A4 gardé
> intact** (attendu non-convergent sous 0.101 ; flip = STOP+recalibrage).
> G8 : 5 scans (S1a/S1b/S2/S3/S4) + vérifications adversariales (dont 1 sonde de compile
> deux-bras) + synthèse. Re-check crates.io JOUR J 2026-07-03 via API HTTP conforme D1.

## 1. Rappel de la lettre du plan (sprint81_plan.md:124-145)

Phase B « Bump deps workspace + recompile mécanique ». But : `cargo build --workspace` vert
sous iroh 1.0.1 ; corriger l'unique cassure compile connue. Livrables : Cargo.toml:38-41 →
`iroh "=1.0.1"` / `iroh-docs "=0.101.0"` / `iroh-gossip "=0.101.0"` / `iroh-blobs "=0.103.0"`
(pins EXACTS, D1 amendée) ; deps relogées éventuelles (iroh-tickets/iroh-metrics) + irpc
0.14→0.17 ; pkarr_resolver.rs:40,109 `CaRootsConfig→CaTlsConfig` (#4300) + re-vérif
`PkarrRelayClient::new(url, tls)` (:114) ; commentaires de version (Cargo.toml:33-35,
node.rs:24, blobs.rs:87, docs.rs:54, discovery.rs:6-8) ; Cargo.lock figé et capturé pour
`cargo tree -d` (Phase G) ; checkpoint gossip (pur recompile). MSRV tranchée 1.91 (D6),
toolchain 1.94, bump 1.95 INTERDIT. Delta tests attendu : 0 net ; baseline T1 0.98 (A/A2/A4)
reste verte sous le nouveau lock. Gate : iroh STRICTEMENT SEUL (D7), veille 1.0.2/RustSEC
jusqu'au push live.

## 2. Tension centrale TRANCHÉE : le build vert est atteignable en B (preuve par compile)

La question ouverte du plan (« unique cassure compile = pkarr » est-elle crédible face aux
breaks C/D/E ?) est tranchée par DEUX niveaux de preuve convergents :

1. **Preuve par diff de sources upstream** (S1a/S1b/S2, tarballs tags v1.0.1/v0.101.0/v0.103.0
   diffés contre v0.98.0/v0.100.0) : la surface publique consommée par SBFB est byte-identique
   sur iroh-blobs 0.100→0.103 (api.rs, api/blobs.rs, api/tags.rs, downloader.rs, ticket.rs,
   net_protocol.rs — seul ajout `wait_idle`), iroh-gossip 0.98→0.101 (net.rs, api.rs) et
   iroh-docs 0.98→0.101 (api.rs, protocol.rs, ticket.rs, engine.rs LiveEvent, store.rs Query).
   Le churn breaking iroh-docs (PR #101 EntrySignature, PR #102 ed25519_dalek→iroh-base) est
   confiné à keys.rs/sync.rs sur des symboles que SBFB n'importe NULLE PART (Grep crates/ :
   0 hit EntrySignature/AuthorPublicKey/NamespacePublicKey/SignedEntry). Les retraits rc.0
   listés en Phase E (`to_info`, `PathWatcher`, `PathInfo`, `PathEvent`, `local_ip`,
   `query_param`) : 0 occurrence SBFB (seul homonyme = fn locale operator_server.rs:274, sans
   rapport) ; `Connection::rtt(PathId)` survit signature inchangée (iroh-1.0.1
   connection.rs:1016 + PathId réexporté endpoint.rs:108).
2. **Preuve par compilation directe** (adversarial S4-1, sonde deux-bras scratchpad) : un
   lib.rs reproduisant VERBATIM la surface d'appel iroh de SBFB (docs.rs, blobs.rs, node.rs,
   shard.rs, seed_protocol.rs, gossip.rs, relay_config.rs, pkarr_resolver.rs, les sites
   runtime.rs:2527/:2630 `DocsNamespaceId::from([u8;32])`) compile (a) PROPRE contre la
   baseline 0.98/0.98/0.98/0.100 (gate de fidélité de la sonde) et (b) contre le bump
   SIMULTANÉ =1.0.1/=0.101.0/=0.101.0/=0.103.0 avec **UNE SEULE erreur** : E0061
   « PkarrRelayClient::new takes 3 arguments but 2 supplied — argument #3 of type DnsResolver
   is missing » + 2 warnings `deprecated` CaRootsConfig. docs.rs, blobs.rs, shard.rs,
   seed_protocol.rs compilent INCHANGÉS.

Conséquence : **pas de découpage B-vs-C/D/E à renégocier**. C/D/E restent des passes de
re-certification sémantique/wire/on-disk, pas des fixes de compile. Le finding S4-1 (« le
bump simultané casse docs.rs/blobs.rs/shard.rs dès le commit B ») est **REFUTED** et ÉCARTÉ.
Limite honnête : la sonde couvre les patterns d'appel inventoriés (21 fichiers, 171
occurrences d'imports), pas le workspace ligne-à-ligne — la confirmation finale reste le
build lui-même, comme la lettre le prévoit (résidu = mécanique-B, cf. §4.6).

## 3. Pourquoi PLAN-ADAPT (3 corrections concrètes, aucun Day-0 touché)

1. **La cassure pkarr est DOUBLE, pas un rename** (S1a-1 ADJUSTED + S1b-1 CONFIRMED + S2-3
   CONFIRMED). `PkarrRelayClient::new` en 1.0.1 = `new(pkarr_relay_url: Url, tls_config:
   ClientConfig, dns_resolver: DnsResolver)` (iroh-1.0.1/src/address_lookup/pkarr.rs:590-605 ;
   docs.rs/iroh/1.0.1) vs call-site SBFB 2-args (pkarr_resolver.rs:114). Sans le 3e arg, le
   build reste ROUGE. `DnsResolver::new()` existe (iroh::dns, défauts cross-platform).
   Le rename `CaRootsConfig→CaTlsConfig` est lui un alias déprécié qui COMPILE
   (iroh-relay-1.0.1/src/tls.rs:28-29 `#[deprecated(since="1.0.0")] pub type CaRootsConfig =
   CaTlsConfig`) — obligatoire uniquement à cause du gate `clippy -D warnings` (lint rustc
   `deprecated` sur :40/:109 ; le lien intra-doc :92 ne déclenche PAS le lint). Zéro dérive
   sécurité : `CaTlsConfig::default()` = `Mode::EmbeddedWebPki` identique 0.98↔1.0.1,
   `client_config(&self, Arc<CryptoProvider>)` et `default_provider()` (gated feature
   `tls-ring`, dans les defaults 1.0.1) survivent même forme. La lettre anticipait par la
   clause « re-vérif :114 » — ce préflight EST le résultat de cette re-vérif.
2. **`rust-version = "1.85"` stale** (S1a-3 + S1b-2). Les 4 crates cibles déclarent
   `rust_version 1.91` (API crates.io jour J), toutes transitives ≤1.91 (noq 1.88, redb 4.1
   =1.89, irpc 0.17=1.91). Le build passe (resolver "2" non MSRV-aware, toolchain 1.94≥1.91)
   mais la déclaration devient mensongère. D6 TRANCHE déjà 1.91 — le bump du champ est
   l'application mécanique, absent de la liste de livrables. Le kickoff (:402-407) le routait
   en résiduel G ; l'avancer en B le colocalise avec la cause.
3. **Garde opérationnel manquant contre la fenêtre one-way** (S3a-1 MAJOR ADJUSTED, le seul
   MAJOR survivant qui AJOUTE un livrable). La lettre B (plan:124-145) n'a ni snapshot ni
   règle de boot ; D3 cond.3 exige « tar snapshot avant 1er boot 0.101 » et cond.4 la
   « validation sur COPIE » (kickoff:378-380), mais le snapshot n'est opérationnalisé qu'en
   Phase H (plan:267-268) et l'invariant kickoff:462 PRÉSUPPOSE que le 1er boot n'arrive
   qu'en H. Or booter le daemon sur le store dev réel est ROUTINE (le rig
   `b3_live_pc_vps.sh` d'A3/A4 boote ce store par design), et l'auto-migration iroh-docs
   redb 2→4 est AUTOMATIQUE à `Store::persistent` (PR #105, mergée 2026-06-01, one-way,
   original renommé `.backup-redb-v2-tuples`). Chemin code : paths.rs:65-72 (défaut daemon =
   `BaseDirs::data_dir()/nexus-grid`, i.e. `%APPDATA%\nexus-grid` Win /
   `~/Library/Application Support/nexus-grid` Mac — PAS le `~/.nexus-grid` du launcher
   unlock.rs:59-61) → runtime.rs:344-374 → node.rs:388 `Docs::persistent`. Perte de baseline
   non garantie sur le chemin nominal (backup préservé) mais réelle sur 3 chemins non
   couverts : crash mid-migration (atomicité NON vérifiée — pré-requis préflight F), store
   BLOBS sous 0.103 (migration distincte, hors #105, non vérifiée), écritures post-migration
   polluant le backup. La question ouverte [D] (kickoff:494-495 « valider l'ouverture redb4
   sur store dev existant ») aggrave : prise à la lettre elle ordonne d'ouvrir le store RÉEL.
   La correction APPLIQUE D3 cond.3 — Day-0 intact.

Ce n'est **pas** un DESIGN-CONFLICT : D1 (pins exacts, re-check jour J API — fait), D6 (MSRV
1.91 tranchée — appliquée), D7 (iroh strictement seul — 0 dep ajoutée, 0 .rs hors pkarr),
D8 (libellé « upgrade ≠ Gate 1 » au body) sont tous HONORÉS par l'approche corrigée. Les
tests hermétiques restent sans danger (e2e daemon pointent NEXUS_GRID_ROOT sur fixtures temp,
e2e.rs:6-10,133,153,171) — seul le boot manuel daemon/launcher/rig touche le store réel.

## 4. Approche corrigée (à coder — supersede la lettre)

1. **Bump point unique** Cargo.toml:38-41 → `iroh = "=1.0.1"` / `iroh-docs = "=0.101.0"` /
   `iroh-gossip = "=0.101.0"` / `iroh-blobs = "=0.103.0"` ; commentaire :33-36 réécrit
   (S81 Phase B, re-check crates.io 2026-07-03, mention feature défaut `redb-v2-migration`
   portant la migration Phase F) ; **`rust-version = "1.91"`** (:24, D6 mécanique, zéro
   re-débat). AUCUNE dep à reloger : irpc/iroh-tickets/iroh-metrics sont purement transitifs
   (Grep crates/**/*.rs : 0 occurrence irpc ; `DocTicket`/`BlobTicket` réexportés aux mêmes
   chemins). AUCUN ajout de feature (defaults préservés partout, S1a-9).
2. **Garde opérationnel AVANT tout boot 0.101** (avance D3 cond.3, prévu H) :
   (a) tar snapshot des stores dev Win + Mac sur la racine daemon RÉELLE
   (`%APPDATA%\nexus-grid` / `~/Library/Application Support/nexus-grid` : `docs.redb` +
   `blobs/`) au moment du commit B, coût ~0 ; (b) règle explicite au body/runbook : « ne pas
   booter le daemon ni rejouer b3_live_pc_vps.sh sur un store réel avant Phase F PASS » ;
   (c) requalifier la question ouverte [D] en « ouverture redb4 sur COPIE du store dev ».
3. **Fix pkarr_resolver.rs (UNIQUE correction de code)** :
   - `:40` `use iroh::tls::{CaTlsConfig, default_provider};` + `use iroh::dns::DnsResolver;`
   - `:109` `CaTlsConfig::default().client_config(default_provider())?` (port verbatim) ;
   - `:114` `PkarrRelayClient::new(pkarr_relay_url, tls_config, DnsResolver::new())` ;
   - `:92` lien intra-doc renommé (hygiène doc, non gate-breaking) + prose `:93` rafraîchie
     (« EmbeddedWebPki » décrivait l'enum 0.98 ; CaTlsConfig 1.0.1 = struct, Default +
     client_config identiques — posture sécurité INCHANGÉE).
   - **INTERDIT** (garde review S3a-2) : tout usage de `custom_server_cert_verifier` /
     `insecure_skip_verify` = régression sécurité à rejeter ; le diff doit rester verbatim.
4. **Commentaires de version** : node.rs:24, blobs.rs:87, docs.rs:54, discovery.rs:6-8 +
   **re-datage des doc-comments A2 datés 0.98** (docs.rs:151-158, runtime.rs:2528-2538 —
   consigne explicite du body 23f3be8 « datée explicitement pour le bump Phase B »).
   Re-Grep les ANCRES SÉMANTIQUES (« Replica not found », bras `Some(t)`), jamais les
   numéros de lignes du plan (repères plan:155 « runtime.rs:2479 » et kickoff:255 périmés,
   S2-4 ; les repères pkarr_resolver.rs:40/:92/:109/:114 et Cargo.toml:24/:33-41 sont EXACTS).
5. **Cargo.lock figé + capture `cargo tree -d`** (gate Phase G) avec deltas ATTENDUS
   documentés : ed25519-dalek dual-tree STATU QUO (3.0.0-pre.6 → `=3.0.0-rc.0`, pin exact
   upstream, veille RustSEC pre-release — convergence mono-arbre impossible : SBFB 2.1 +
   tor-llcrypto 2.x) ; **redb ×2 PAR DESIGN** (feature défaut iroh-docs `redb-v2-migration`
   → redb_v3 ^3.1 + redb ^4.1, requis Phase F — NE PAS désactiver les default features ;
   2.6.3 disparaît) ; rand ×3 / reqwest ×2 / hickory ×2 inchangés ; iroh-util = seule vraie
   nouvelle entrée ; noq 0.18→1.0.1, irpc 0.14→0.17, iroh-metrics 0.38.3→1.0.1.
6. **Build + gates full fail-fast** : `cargo build --workspace`, fmt, `clippy --workspace
   --all-targets -D warnings` (rend le rename pkarr OBLIGATOIRE), nextest workspace,
   doctests, release build, dual-platform Docker `sbfb-ci`. Toute cassure résiduelle
   imprévue (bounds génériques, inférence, lint deprecated hors surface sondée) : absorber
   en B **UNIQUEMENT si mécanique 1:1** (rename/arg sans sémantique) avec liste explicite
   fichier:ligne au body (précédent S32 Phase A `90aff27` assume l'impact compile
   workspace-wide) ; sinon cataloguer NOMINALEMENT pour C/D/E — jamais d'absorption
   silencieuse, jamais d'arbre non-compilable inter-phases.
7. **Baseline verte sous le nouveau lock** : nextest full (référence fdb8ad7 : Win 2028
   0-skip / Docker 2032) ; surveiller NOMINALEMENT les 4 tests A2 (runtime.rs:4212-4291) et
   le **CONTROL A4** (dispatch_loop.rs:544-633). CONTROL attendu NON-convergent sous 0.101
   (`doc_open` passe `OpenOpts::default()` → sync=false, v0.101.0 api/actor.rs:359-361 ;
   assertion NÉGATIVE 8s — la lenteur 1.0 ne peut PAS le faire échouer à tort). S'il
   CONVERGE → STOP, recalibrer la prémisse A4 (consigne doc-comment :553-555) avant tout
   autre travail. Rouge par compile de helpers de test → fix mécanique B.
8. **Checkpoint gossip « pur recompile » tenu tel quel** (diff net.rs/api.rs vide 0.98→0.101,
   changelogs 0.99/0.100/0.101 = bumps de dep seuls) ; si une cassure gossip surface au
   build, c'est un signal changelog-incomplet à DOCUMENTER, pas à absorber silencieusement.
9. **Body de commit** : statuer les 2 carries (§7), libellé D8 verbatim « upgrade ≠ Gate 1 /
   Gate 3, R-iroh-audit P0 inchangé, pilote reste fermé », registre des engagements
   A/A2/A3/A4 (S2-6 — pour que l'audit gate S81 retrouve chaque disposition sans re-fouiller
   les 4 bodies), snapshot D3 cond.3 avancé, deltas lock attendus, delta tests 0 net.

Delta tests : **0 net** (inchangé de la lettre — recompile ; les tests existants restent la
preuve d'exécution).

## 5. Restitution des scans (fan-out 5 + adversarial)

| Scan | Verdict-hint | Findings clés retenus | Adversarial |
|---|---|---|---|
| S1a OSS prior-art (diff sources tags) | EXECUTE | S1a-1 MAJOR (3e arg DnsResolver, rename gate-clippy) ; S1a-2 surfaces C/D/E ne cassent pas à la compile ; S1a-3 rust-version stale ; S1a-4/5 carries A2/A4 statués ; S1a-6 item C DocsNamespaceId = probable no-op ; S1a-7 tickets persistés compat ; S1a-8 irpc transitif-seul ; S1a-9 features/ALPN préservés ; S1a-10 limite de méthode | S1a-1 ADJUSTED (retenu corrigé : :92 = lien intra-doc non gate-breaking, prose :93 stale) |
| S1b deps/CVE/MSRV/licences (API jour J) | EXECUTE | S1b-1 MAJOR (3 args = LA vraie cassure) ; S1b-2 rust-version 1.91 ; S1b-3 alias deprecated soft + Default EmbeddedWebPki inchangé ; S1b-4 pins = max_stable exacts ; S1b-5 OSV zéro + licences permissives ; S1b-6 dalek statu quo ; S1b-7 redb ×2 by-design ; S1b-8 tension tranchée côté deps ; S1b-9 carry A2 no-op ; S1b-10 topologie lock | S1b-1 CONFIRMED |
| S2 historique (bodies + S32 + upstream) | EXECUTE | S2-1 carry A2 = re-dater doc-comments, 0 code ; S2-2 MAJOR (B statue / C fixe, bisectabilité) ; S2-3 MAJOR (pkarr = vraie cassure, docs source-compatible) ; S2-4 repères périmés ; S2-5 CONTROL sain ; S2-6 registre engagements | S2-2 + S2-3 CONFIRMED ligne à ligne (0 réfutation) |
| S3 threat model | PLAN-ADAPT | S3a-1 MAJOR (fenêtre one-way sans garde — LE driver du verdict) ; S3a-2 pas de downgrade TLS ; S3a-3 THREAT_MODEL:22 doublement périmé (« blobs 0.97 ») ; S3a-4 D8 au body ; S3a-5 events-core hors blast-radius | S3a-1 ADJUSTED (retenu corrigé : backup nominal préservé, boot = ROUTINE, racine daemon = paths.rs:65-72) |
| S4 wire producteur→consommateur | PLAN-ADAPT (driver RÉFUTÉ) | **S4-1 REFUTED → ÉCARTÉ** (sonde compile : 1 seule erreur = pkarr) ; S4-2 0-bump par construction ; S4-3 tickets M8/anchors re-parsables ; S4-4 EntrySignature = 0 delta wire ; S4-5 gossip pur recompile supporté ; S4-6/7 carries A2/A4 ; S4-8 zéro zombie legacy-decode ; S4-9 artefacts T2 = contrat de replay | S4-1 REFUTED par compilation directe deux-bras (baseline 0.98 propre → bump : E0061 pkarr + 2 warnings deprecated, rien d'autre) |

Convergence : 3 scans EXECUTE + 1 PLAN-ADAPT au driver réfuté + 1 PLAN-ADAPT au driver
survivant (garde opérationnel). Le verdict agrégé PLAN-ADAPT porte sur les 3 corrections §3 ;
le cœur code de la lettre est exécutable tel quel.

## 6. Faits vérifiés jour J (2026-07-03, API HTTP crates.io — jamais cargo info, D1)

1. **Pins D1 = max_stable exacts, aucun yank, pas de 1.0.2** : iroh 1.0.1 (pub 2026-06-29),
   iroh-docs 0.101.0 + iroh-gossip 0.101.0 (2026-06-15), iroh-blobs 0.103.0 (2026-06-15).
2. **MSRV** : `rust_version 1.91` sur les 4 cibles ; 19 transitives vérifiées ≤1.91
   (noq 1.88, redb 4.1=1.89, irpc 0.17=1.91, iroh-metrics 1.85) — D6 tient, toolchain 1.94
   suffit, zéro DESIGN-CONFLICT.
3. **OSV/RustSEC : zéro advisory** sur iroh/iroh-docs/iroh-blobs/iroh-gossip/redb/irpc/noq ;
   quinn (fixée ≤0.7.0, lock=0.11.9) et ed25519-dalek (fixée 2.0.0) hors-fenêtre. Licences
   toutes permissives (pile n0 MIT OR Apache-2.0, dalek 3.0.0-rc.0 BSD-3-Clause déjà dans
   l'arbre en 3.0.0-pre.6) — invariant AGPL sain. NB : les 8 advisories transitives ROUGE
   pré-existantes (hors iroh, carry A3) restent routées `cargo deny` Phase G.
4. **PkarrRelayClient::new 1.0.1 = 3 arguments** (Url, ClientConfig, DnsResolver) —
   docs.rs/iroh/1.0.1 + source pkarr.rs:590-605 ; `DnsResolver::new()` confirmé (iroh::dns) ;
   `PkarrRelayClientBuilder` exige AUSSI un DnsResolver (pas d'échappatoire 2-args).
5. **Carry A2 upstream** : `#[error("Replica not found")]` byte-identique (v0.101.0
   store.rs:24-27) ; `open()` hardcode `Ok(Some)` (api.rs:262-265, mêmes lignes que 0.98) ;
   absence → `Err(OpenError::NotFound)` (fs.rs:356) ; érasure RPC préservée
   (`RpcError = serde_error::Error` api.rs:47 + actor.rs:358-363).
6. **Carry A4 upstream** : `start_sync(&self, peers: Vec<EndpointAddr>)` signature identique
   (api.rs:437) ; insert sync-set UNIQUEMENT via start_sync (live.rs:408-414) ; broadcast
   gaté `is_syncing` (:713) ; reject `AbortReason::NotFound` (state.rs:97) ; cap
   `PEERS_PER_DOC_CACHE_SIZE=5` (store.rs:17) — diff 0.98→0.101 des 4 fichiers = 2 renames
   d'imports cosmétiques (futures_lite→n0_future).
7. **Wire iroh byte-stable** : PR #101 restaure le wire byte-identique 0.98 et le verrouille
   par 3 tests snapshot postcard ; EndpointAddr{id,addrs}/TransportAddr identiques 0.98↔1.0.1
   (ordre variants postcard préservé) ; format ticket extérieur inchangé (KIND +
   BASE32_NOPAD postcard, iroh-tickets 0.5→1.0 = renames de trait seulement) → strings
   persistées (M8 `doc_ticket`, `tasks_doc_ticket`, anchors.json) re-parsables.
8. **Migration redb** : auto-migration 2→4 derrière feature défaut `redb-v2-migration`
   (v0.101.0 Cargo.toml:47-48,76,80 + fs.rs:136 `migrate_redb_v2_tuples::run`), erreur
   explicite si feature off (fs.rs:143) ; AUTOMATIQUE à l'ouverture, one-way, backup
   `.backup-redb-v2-tuples` (PR #105).
9. **ALPN inchangés** : `/iroh-sync/1` (net.rs:18), `/iroh-bytes/4` (protocol.rs:406).
10. **Sonde compile deux-bras** (adversarial S4-1) : bump simultané des 4 pins → 1 seule
    erreur E0061 pkarr + 2 warnings deprecated. Fait décisif du préflight.
11. **UNVERIFIED (assumés, tracés)** : wrapping du Display à travers irpc 0.17 au vif
    (mitigé : `.contains` robuste + tests A2 + CONTROL au premier nextest) ; chemin de
    migration store BLOBS redb2 sous 0.103 (distinct de #105 — pré-requis préflight F) ;
    coupure relais N0 2026-09-30 (prémisse Day-0 non re-vérifiée ; indice cohérent release
    v1.0.0 « Updated relay URLs to 1.0 stable (#4341) »).

## 7. Carries entrants statués

1. **Carry A2 (matcher « Replica not found ») — FERMÉ PAR CONSTAT en B, 0 code.**
   Re-calibrage = no-op prouvé sur pièces (§6.5). Action B : re-dater les doc-comments 0.98
   (docs.rs:151-158, runtime.rs:2528-2538) + statuer au body avec référence
   v0.101.0 store.rs:24-27 + api.rs:262-265. GARDER `.contains` (jamais `==`), ne toucher ni
   la branche `Ok(opt)` défensive ni les 4 tests A2. La preuve d'exécution = run des tests
   A2 sous le nouveau lock (déjà dans « baseline T1 reste verte »).
2. **Carry A4 P2-SIBLING-SYNC-SET — B STATUE (body), C FIXE.** Le mécanisme 0.101 est
   vérifié INCHANGÉ (§6.6) : le trou reopen-sans-start_sync (runtime.rs:2552-2564 /
   :2647-2659, bras ticket-persisté) persiste identiquement — gap sémantique SBFB, pas un
   break upstream ; rien n'oblige à le traiter en B. Le fixer en B violerait la lettre
   (recompile pur, 0 net delta) ET la bisectabilité (précédent S32 `90aff27` : bump =
   Cargo.toml+lock seuls ; review Phase A:69 « JAMAIS dans le commit de bump »). Phase C
   applique le pattern `open_project_doc_for_dispatch` (runtime.rs:2051, start_sync
   fail-fast doctrine A2) aux 2 sites + tests miroir CONTROL/GREEN. Jamais de fold
   silencieux (review A4:398-405 « statuer sur les 3 sites d'un coup » — le 3e est fait A4).
3. **CONTROL A4 tripwire — gardé INTACT en B.** Attendu : reste non-convergent sous 0.101
   (§4.7). Flip vers la convergence = STOP + recalibrage prémisse A4 AVANT tout autre
   travail. Rouge par compile = fix mécanique B. La lenteur ne peut pas le faire échouer à
   tort (assertion négative à timeout).
4. **Question ouverte du plan (« unique cassure = pkarr » crédible ?) — TRANCHÉE : OUI**,
   prouvée par diff de sources + sonde de compile (§2). Le contenu exact de la cassure
   diffère du libellé (3e arg, pas le rename) — documenter l'écart au body.

## 8. Carries sortants (créés / re-routés vers C..K)

1. **Phase C** : (a) fix sibling sync-set + tests miroir (§7.2) ; (b) test one-shot « ticket
   string minté sous 0.98 (fixture inline) parse sous le nouveau lock » (ancre SBFB de la
   garantie structurelle §6.7) ; (c) vérif AU VIF que `.contains("Replica not found")`
   matche à travers irpc 0.17 (si un test A2 la couvre déjà sous le nouveau lock, constat
   suffit) ; (d) RE-SCOPER l'item « reconstruction DocsNamespaceId::from raw-bytes
   (runtime.rs:2479) » : probable NO-OP (`From<[u8;32]>` survit, prouvé à la compile ;
   repère :2479 périmé → sites réels :2527/:2630) — le retirer ou requalifier en simple
   vérification.
2. **Phase E** : allégée — les retraits rc.0 ont 0 call-site SBFB et `Connection::rtt(PathId)`
   survit signature inchangée (prouvé compile + source connection.rs:1016). Restent : check
   nommé survie URL pkarr (dns.iroh.link/pkarr, pkarr_resolver.rs:54), surfaces runtime
   non-compile (comportement AcceptError/ProtocolHandler au vif), relay_config.rs.
3. **Phase F** : (a) lire l'atomicité crash-mid-migration du PR #105 (pré-requis préflight
   F) ; (b) vérifier le chemin de migration du store BLOBS redb2 sous iroh-blobs 0.103
   (distinct de #105, NON vérifié) ; (c) le snapshot est DÉJÀ pris en B (avancé) — F valide
   sur COPIE comme D3 cond.4 l'exige.
4. **Phase G** : (a) THREAT_MODEL:22 se corrige depuis sa valeur RÉELLE (« blobs 0.97 »)
   vers 1.0.1/docs 0.101/gossip 0.101/blobs 0.103 (pas « 0.98→1.0.1 ») ; :195 résiduel
   menace E reste M (D8 : wire-freeze = neutre-à-positif, jamais un durcissement) ;
   (b) `cargo deny check advisories` post-bump (8 advisories transitives pré-existantes +
   entrée `deny.toml` RUSTSEC-2026-0097 périmée à nettoyer — carry A3) ; (c) gate
   `cargo tree -d` : whitelister dalek dual-tree (mention `=3.0.0-rc.0` veille pre-release)
   + redb ×2 by-design ; (d) résiduel « rust-version 1.85→1.91 » du kickoff:407 SOLDÉ PAR
   AVANCE en B.
5. **Phase H** : le snapshot D3 cond.3 est exécuté en B — H le VÉRIFIE/rejoue avant la
   bascule live, il ne le découvre plus.
6. **Veille continue** : re-jouer re-check crates.io (1.0.2 ?) + OSV avant le push live
   (règle du plan, inchangée).

## 9. Risques résiduels

- **Résidu compile hors surface sondée** (inférence de type, bounds génériques, lint
  deprecated résiduel) : possible, mécanique à corriger, dans le scope B (§4.6) ; escalade
  PLAN-ADAPT seulement si un résidu exige un choix sémantique (aucun candidat identifié).
- **Wrapping irpc 0.17 du Display** (UNVERIFIED §6.11) : `.contains` y est robuste par
  construction ; oracle = tests A2 + CONTROL au premier nextest post-bump.
- **Fenêtre B→F malgré le snapshot** : le snapshot protège la baseline mais ne rend pas le
  boot inoffensif (3 chemins de perte §3.3) — la règle « pas de boot store réel avant F
  PASS » est le vrai garde ; la re-jouer à CHAQUE session jusqu'à F.
- **Migration blobs redb2 non vérifiée** : si iroh-blobs 0.103 ne migre PAS un store 0.100,
  c'est un problème Phase F (COPIE), pas B — mais le snapshot B couvre `blobs/` pour ça.
- **Relais N0 EOL 2026-09-30** : prémisse Day-0 non re-vérifiée par ces scans — sans impact
  sur B (le bump la satisfait quoi qu'il arrive), à re-sourcer avant H si le calendrier live
  en dépend.
- **Scope creep interdit** : 0 fichier .rs fonctionnel hors pkarr_resolver.rs + commentaires ;
  ne pas toucher runtime.rs:2552-2564/:2647-2659 (carry C) ; iroh STRICTEMENT SEUL (D7) ;
  bump toolchain 1.95 INTERDIT.

## 10. Wire check (0-bump + test-acteur §6.12)

0-bump SBFB CONFIRMÉ PAR CONSTRUCTION (S4-2) : les fichiers du livrable B (Cargo.toml,
Cargo.lock, pkarr_resolver.rs, commentaires node.rs:24/blobs.rs:87/docs.rs:54/
discovery.rs:6-8) sont DISJOINTS de toutes les constantes wire (23 `DOMAIN_*_V1`
canonical.rs:77-332, `FEED_FORMAT_VERSION` public_feed.rs:20, 10+ `*_FORMAT_VERSION=1`,
`SEED_ALPN` node.rs:68, `SHARD_ALPN` node.rs:80). node.rs n'est touché qu'au commentaire :24
(à 44 lignes de SEED_ALPN) — la review vérifie que le diff node.rs est comment-only. Côté
wire iroh : byte-stable 0.98→cibles (§6.7), aucune conséquence sur JCS/enveloppes SBFB.
Aucun test « legacy decode » de fixture iroh versionnée n'existe (S4-8 : rien à zombifier,
pre-launch policy = no-op pour cet upgrade). Les artefacts T2 committés
(sprint81_t2_baseline_098.json + sprint81_t2_a4_differential_098.json) ne sont PAS invalidés :
leur replay_contract EST le différentiel post-bump — ne pas les modifier ni les rejouer en B ;
seul un verdict qui SE DÉGRADE au replay est imputable au bump (S4-9).
**Test-acteur** : bump de deps + fix compile interne = aucune frontière docs-contrat nouvelle
(aucune API loopback nouvelle lue par un runtime distinct) → aucune étiquette requise.

## 11. Commit shape

`chore(deps): Sprint 81 Phase B — bump iroh =1.0.1 + docs/gossip/blobs =0.101.0/=0.101.0/
=0.103.0, recompile mécanique (0 bump wire)` — body : pins exacts D1 (re-check crates.io
2026-07-03) + rust-version 1.91 (D6 mécanique) + fix pkarr DOUBLE (3e arg DnsResolver =
vraie cassure E0061 ; rename CaTlsConfig = gate clippy ; posture TLS EmbeddedWebPki
inchangée) + carries statués (A2 FERMÉ par constat store.rs:24-27/api.rs:262-265 ; A4
sibling STATUÉ → Phase C, mécanisme 0.101 vérifié inchangé) + registre engagements
A/A2/A3/A4 + D8 verbatim « upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 inchangé, pilote
reste fermé » + snapshot stores dev D3 cond.3 avancé + règle « pas de boot store réel avant
F PASS » + deltas lock attendus (dalek rc.0 statu quo, redb ×2 by-design, iroh-util neuf) +
CONTROL A4 non-convergent confirmé sous 0.101 + delta tests 0 net (baseline fdb8ad7 :
Win 2028 0-skip / Docker 2032).
