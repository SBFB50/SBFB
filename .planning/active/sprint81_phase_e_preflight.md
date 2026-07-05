# Sprint 81 Phase E — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT.** La lettre du plan Phase E (« re-certifier **compile + handshake** des
> surfaces transport non-hermétiques `shard.rs` / `seed_protocol.rs` / `pkarr_resolver.rs` /
> `relay_config.rs` / `node.rs` + **check nommé** de survie URL pkarr/relais + **PLAN B C8**
> pré-provisionné ») a été **rédigée AVANT le bump Phase B (`c899d54`)**. C'est la **5e lettre
> pré-bump** — comme B/C/D, le corps de compile a déjà été absorbé par le bump (nextest Win 2039
> 0-skip / Docker 2043, clippy, doctests, release verts), et 6 scans + vérifications adversariales
> tranchent item par item. Résultat structurel identique aux phases précédentes : **PLAN-ADAPT,
> 0 DESIGN-CONFLICT, aucun Day-0 touché**. Le contenu réel de la Phase E se réduit à **quatre
> gestes minces + une décision de split** :
>
> 1. **[NO-OP re-cert]** Toute la surface transport `shard.rs` + `Connection` + `seed_protocol.rs`
>    + pkarr + relais est **recompile-prouvée sous 1.0.1** (byte-diff des sources vendored :
>    `Connection::rtt(&self, PathId)->Option<Duration>` byte-identique `connection.rs:970`↔`:1016`,
>    `PathId::ZERO` valide, `ProtocolHandler` = 1 méthode requise inchangée, `AcceptError`
>    `#[non_exhaustive]` non-cassant, retraits rc.0 = **0 call-site (vacuité)**). **Découverte
>    load-bearing** : le handshake `sbfb/seed/0` **ET** `sbfb/shard/1` in-process est **DÉJÀ vert
>    sous 1.0.1** (13 `#[tokio::test]` seed `seed_protocol.rs:459+` ; `shard_handshake_admits_member`
>    `shard.rs:515` + `shard_handshake_rejects_non_member` `:537` + RTT single-path `:564`). Le
>    sous-test T1 (5) « recompile + handshake shard » et T1 (1) « seed ALPN handshake » sont **déjà
>    satisfaits** — E ne comble aucun gap hermétique handshake.
> 2. **[TEST-A-AJOUTER, +1 net]** Un **tripwire hermétique de parité de constante** :
>    `DEFAULT_PKARR_RELAY_URL == iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD` + parse `Url`, qui
>    attrape tout drift amont au prochain bump. C'est le **support HERMÉTIQUE du CHECK NOMMÉ** — le
>    handshake seed/shard 2-nœuds du plan (« +1..2 Rust ») est déjà couvert → net-new handshake ≈ 0.
> 3. **[DOC-STALE, lot E]** 5 doc-comments à re-dater/corriger dans des fichiers transport E
>    (`gossip.rs:259`/`:740`, `tls_pinning.rs:32` **avec re-vérif du fond**, `transport_probe.rs:22-26`,
>    `relay_config.rs:5`+`:18-20`). **1 carry stale est DÉJÀ fermé** (`http.rs:3213` re-daté à Phase C
>    → retirer du lot).
> 4. **[CHECK NOMMÉ, artefact]** La survie URL pkarr/relais = **DEUX vérifications distinctes** : (a)
>    tripwire hermétique statique — **DÉJÀ PROUVÉ** : l'URL pkarr n'a PAS changé (`pkarr.rs:127` ==
>    notre `:55`), et les URLs relais SONT auto-adoptées par `presets::N0` (0 hardcode SBFB) ; (b)
>    **sonde LIVE** vers l'endpoint réel, qui n'est **JAMAIS** un unit test (interdit hermétisme +
>    fuite DNS) → artefact **acceptance T2** RIG-gated. Conflater « recompile = URL servie » est le
>    sophisme à bannir.
>
> **Adaptation structurelle #1 (le vrai résidu lourd)** : le livrable **PLAN B C8 « acceptance
> zéro-n0 » est SOUS-SCOPÉ par le plan.** Il n'est **PAS** atteignable par « re-cert `node.rs:318`
> presets::N0 » : `presets::N0` câble une **discovery ambiante** (`PkarrPublisher::n0_dns()` +
> `PkarrResolver::n0_dns()` + `DnsAddressLookup::n0_dns()`, `presets.rs:121/128/133`) vers
> `dns.iroh.link`, et `node.rs:318` `.address_lookup(memory_lookup)` est **ADDITIF**
> (`endpoint.rs:605` ajoute vs `:585` `clear_address_lookup` séparé), `node.rs:348` n'override que
> `relay_mode`. **Zéro-n0 exige donc du CODE NEUF** d'override discovery (`clear_address_lookup` +
> `PkarrPublisher::builder(self_url)` + `PkarrResolver::builder(self_url)` + `RelayMode::Custom`
> gated par env). **Ce n'est PAS un DESIGN-CONFLICT** — toutes les briques existent en 1.0.1 (pas
> d'arbitrage PO requis), mais le plan doit être corrigé : **provisioning seul insuffisant**.
>
> **Adaptation structurelle #2 (split E')** : le portage handshake étant **déjà vert** et le seul
> travail lourd étant le **CODE NEUF discovery-override + le provisionnement relais/pkarr
> self-hosted + l'acceptance zéro-n0 LIVE (2-4 j)**, ce bloc **déborde le scope-cut « compile +
> handshake SEULEMENT »**. **Décision : SPLIT E' = OUI**, périmètre = **PLAN B C8 intégral**
> (§7.3). E-core livre les gestes 1-4 ci-dessus ; E' livre C8. Le split respecte les gates
> calendaires (01/08 provisionner). Candidat de split confirmé par 4 scans (S2-20, S3-6, S1a2-21).
>
> **`shard.rs` reste UNVERIFIED-high-risk pour le SEUL RTT multipath LIVE** — jamais claimer
> « shard SAUVÉ ». La re-cert LIVE multipath vit en **Phases I/J** (décision PO C1). 0 bump wire
> (23 `DOMAIN_*_V1`, tous `*_FORMAT_VERSION=1`, `sbfb/seed/0` + `sbfb/shard/1` verbatim), iroh
> strictement seul, toolchain 1.94, `presets::N0` conservé par défaut.
> G8 : 6 scans (S1a API transport / S1a2 discovery pkarr+relais+PLAN B / S1b deps-CVE-lock / S2
> décisions historiques / S3 threat model / S4 wire+call-sites) + 6 vérifications adversariales.
> Bilan : **PLAN-ADAPT dominant ; 1 REFUTED matériel (S3-8, réhabilite le shard handshake comme
> DÉJÀ-vert) ; corrections de lignes/sources intégrées ; manques absorbés** (seed/shard tests
> existants, `IROH_FORCE_STAGING_RELAYS`, `DEFAULT_RELAY_QUIC_PORT=7842` confirmé, `discovery.rs`
> `TransportAddr`, relay_config.rs « three/byte-for-byte » stale).

---

## 1. Rappel de la lettre du plan (sprint81_plan.md:179-204)

Phase E « Surfaces fragiles transport re-cert (3 crates) ». **But** : re-certifier **compile +
handshake** des surfaces non-hermétiques + **check nommé de survie URL pkarr/relais** (discovery
casse silencieusement sinon). **Jobs/surfaces** : shard / seed-protocol / pkarr / relais ; crates
`nexus-core-rs`, `nexus-shell-daemon`, `nexus-shell-daemon-core`. **Livrables** :
`shard.rs:60-63,171-181,299-327` (`Connection::rtt(PathId::ZERO)`, `closed`/`close`/`remote_id` —
**UNVERIFIED-high-risk**) ; liste canonique des retraits rc.0 (`to_info`→`weak_handle`,
`PathWatcher/PathInfo`→`paths()/PathList` + `PathEvent #[non_exhaustive]`, `Incoming::local_ip`→
`local_addr`, ClientBuilder `query_param`→`auth_token`) ; `seed_protocol.rs:44-48,263-264`
(`ProtocolHandler`/`AcceptError`) ; `pkarr_resolver.rs:38-41,54,107-115` (+ **survie URL
`dns.iroh.link/pkarr` `:54` = check nommé**) ; `relay_config.rs:17-20,46` + `node.rs:318,329,348`
(`RelayMode::Custom`, `default_relay_map`, `presets::N0`) ; re-scan call-sites daemon + daemon-core
(D7) ; **PLAN B C8 PRÉ-PROVISIONNÉ (2-4 j)** : relais iroh self-hosted + pkarr self-hosted +
**acceptance zéro-n0**. **Delta tests** : **+1..2 Rust** (handshake seed 2-nœuds ; pkarr parse).
**T1** : sous-test (5) recompile + handshake shard `sbfb/shard/1` in-process + sous-test (1) seed
ALPN `sbfb/seed/0` handshake. **Scope-cut** : compile + handshake SEULEMENT ; re-cert LIVE shard
multipath → Phases I/J (PO C1) ; **split E' possible** ; `presets::N0` conservé ; PLAN B C8
OBLIGATOIRE (gates 01/08 / 25/08 / 15/09).

**Le nœud du PLAN-ADAPT** : comme B/C/D, le libellé décrit une « recompilation » que le bump B a
déjà faite (2039 verts sous 1.0.1) **et** un travail de handshake que le code **satisfait déjà**
(harness 2-nœuds seed/shard verts post-B). La lettre n'a PAS anticipé (a) que le bump absorberait
toutes les signatures transport, (b) que les handshakes seed/shard seraient déjà couverts (delta
handshake net ≈ 0), (c) que « acceptance zéro-n0 » exigerait du **CODE NEUF** d'override discovery
(presets::N0 câble n0 en dur, l'override actuel ne touche que le relais). Ci-dessous le vrai
périmètre, evidence-adossé item par item.

---

## 2. Pourquoi PLAN-ADAPT — la surface transport est NO-OP compile-prouvée (double evidence)

Chaque verdict NO-OP est adossé à **DEUX** sources : (a) baseline B verte (nextest 2039 Win 0-skip
@ `c899d54`, `shard.rs`/`seed_protocol.rs`/`pkarr_resolver.rs`/`relay_config.rs`/`node.rs`
recompilés vert, re-verts Docker 2043 @ Phase D) ; (b) **byte-diff / signature des sources
vendored** `iroh-1.0.1` + `noq-1.0.1` + `noq-proto-1.0.1` (pas une simple inférence de compile).

| Surface de la lettre | Verdict | Preuve upstream (vendored) | Ancre SBFB |
|---|---|---|---|
| Imports transport `shard.rs` (Connection, PathId, ReadExactError, RecvStream, SendStream, AcceptError, ProtocolHandler, DynProtocolHandler, MemoryLookup) | **NO-OP** | re-exports `iroh-1.0.1/src/endpoint.rs:108-113` + `protocol.rs:116/228/331` ; `address_lookup::memory::MemoryLookup` struct `memory.rs:75` + `add_endpoint_info :184` | `shard.rs:60-63` |
| `Connection::rtt(&self, PathId)->Option<Duration>` | **NO-OP (signature IDENTIQUE 0.98.2↔1.0.1)** | `iroh-0.98.2 connection.rs:970` ↔ `iroh-1.0.1 connection.rs:1016` (même délégation) — pas un break 0.98→1.0, continuité | `shard.rs:179-181` |
| `PathId::ZERO` | **NO-OP** | associated-const `noq-proto-1.0.1/src/connection/paths.rs:55` (`PathId` `:28`), reachable via `iroh::endpoint::PathId` | `shard.rs:180` |
| `ShardProtocol::accept` (`remote_id`/`close(VarInt)`/`accept_bi`/`closed`) | **NO-OP** | `Connection<HandshakeCompleted>::remote_id -> EndpointId` `iroh-1.0.1 connection.rs:1127` ↔ `0.98.2:1077` | `shard.rs:299-327` |
| `From<u32> for VarInt` | **NO-OP** | `noq-proto-1.0.1/src/varint.rs:93` | `shard.rs:306,509,531,585` |
| `ReadExactError::FinishedEarly(usize)` | **NO-OP** | `noq-1.0.1/src/recv_stream.rs:747-750` | `shard.rs:151-169` |
| `RecvStream::read_to_end/read_exact` + `SendStream::write_all/finish` + `Connection::accept_bi/open_bi` | **NO-OP** | `noq-1.0.1 recv_stream.rs:737-750` ; `iroh-1.0.1 connection.rs:885/901` (impl `811-1113`) | `shard.rs`/`seed_protocol.rs` |
| Retraits rc.0 (`to_info`→`weak_handle`, `PathWatcher/PathInfo`→`paths()/PathList`, `PathEvent`, `local_ip`→`local_addr`, `query_param`→`auth_token`) | **NO-OP par VACUITÉ** | grep `crates/` hors target : `to_info=0 weak_handle=0 PathWatcher=0 PathInfo=0 .paths(=0 PathList=0 PathEvent=0 local_ip=0` ; hits `query_param`/`auth_token` = **faux positifs** (`operator_server.rs:274` helper querystring + X-SBFB-Token `auth.rs`) | 3 crates |
| `ProtocolHandler` (1 méthode requise `accept`) | **NO-OP** | `iroh-1.0.1 protocol.rs:273` requis ; `on_accepting:248` + `shutdown:284` defaults == 0.98.2 | `seed_protocol.rs:262-283` + `shard.rs:294-327` |
| `AcceptError #[non_exhaustive]` | **NO-OP (non-cassant)** | `iroh-1.0.1 protocol.rs:115-160` (`from_err:137` + `From<io::Error>:152`) ; usages = `from_err` + `?` uniquement, 0 match exhaustif | `seed_protocol.rs:263-282` |
| `seed_protocol.rs::accept` (`remote_id`/`accept_bi`/`read_to_end(MAX)`/`closed`) | **NO-OP** | idem ProtocolHandler + `MAX_SEED_MSG_BYTES=64KiB` `seed_protocol.rs:61` borné inchangé | `seed_protocol.rs:262-282` |
| `pkarr_resolver.rs::new()` (`CaTlsConfig::default().client_config` + `PkarrRelayClient::new(url, tls, DnsResolver::new())`) | **NO-OP (fix fait EN B, pas E)** | cassure `CaRootsConfig→CaTlsConfig` #4300 réparée @ B ; doc `:92-105` déjà re-datée | `pkarr_resolver.rs:39-41,114-119` |
| `RelayMode::Custom(RelayMap)` | **NO-OP** | enum `iroh-1.0.1 endpoint.rs:1922`, variante `Custom` `:1933` | `node.rs:348` |
| `presets::N0` | **NO-OP** | `iroh-1.0.1 endpoint/presets.rs:112` (`pub struct N0`) | `node.rs:44,318` |
| `relay_config.rs:46` (`RelayConfig`/`RelayMap`/`RelayUrl`) + `validate_relay_url` (https-only, loopback-reject loud) | **NO-OP** | 3 types présents 1.0.1 ; `default_relay_map` `defaults.rs:36` (ref doc valide, jamais appelée — `load_relay_map` renvoie `Ok(None)`) | `relay_config.rs:46,110-148,176-220` |
| `DEFAULT_RELAY_QUIC_PORT` (doc `relay_config.rs:80-81` « 7842 ») | **NO-OP (valeur VÉRIFIÉE 1.0.1)** | `iroh-relay-1.0.1/src/defaults.rs:7 = 7842` re-exporté `iroh-1.0.1/src/defaults.rs:7` — doc EXACTE (correction adversariale S1a2-20) | `relay_config.rs:80-81` |
| `http.rs` call-sites (`EndpointId::from_str :2725`, `EndpointAddr::from :2809`) | **NO-OP** | types 1.0.1 stables (compile B). `remote_info` `:3215` = **doc-comment seul**, pas un call-site câblé (correction S4-8) | `http.rs` |
| `browse.rs` (`Endpoint::connect(id, iroh_blobs::ALPN)`), `transport_probe.rs`, `discovery.rs` (`TransportAddr::Relay/Ip` match) | **NO-OP** | compile B ; `TransportAddr` type 1.0 stable (manque S4 : `discovery.rs:30,84,134-137` — surface discovery à noter, compile-prouvée) | daemon-core |

**Conclusion §2** : sur toute la surface transport de la lettre, **rien n'est du code à écrire** en
E-core. Le bump B a absorbé toutes les signatures et le byte-diff vendored le confirme
indépendamment. Correction de comptes intégrée : **23** tags `DOMAIN_*_V1` (`canonical.rs:77-332`,
pas 21 — S4-3), les retraits rc.0 sont **des non-événements** pour SBFB.

---

## 3. Découverte load-bearing — le handshake seed ET shard est DÉJÀ vert (delta handshake ≈ 0)

Le point le plus important pour recadrer le delta tests. Le plan liste « +1..2 Rust (handshake
seed 2-nœuds) » comme livrable neuf et traite le handshake shard comme à re-certifier
(UNVERIFIED). **Les deux sont déjà couverts et verts sous 1.0.1** (baseline B 2039 0-skip / D
Docker 2043 0 fail, tests non-`#[ignore]`, non feature-gated) :

- **Seed `sbfb/seed/0`** : 13 `#[tokio::test]` dans `seed_protocol.rs` (dial `endpoint.connect(peer,
  SEED_ALPN)` `:307` → `accept` `:263` → sign/verify `SeedRequest`/`SeedResponse` → redeem invite),
  dont `seed_e2e_two_nodes_peer_keeps_app_reachable :459/:460`, `seed_requires_invite_and_approval
  :496`, + 9 tests handler adversariaux (dialer-mismatch `:627`, stale-ts `:650`, bad-version
  `:670`, replay `:689`, invite revoked/expired/exhausted `:715/:741/:768`, different-archive-hash
  `:804`, content-hash-mismatch `:834`). Le sous-test **T1 (1) « seed ALPN handshake » est déjà
  satisfait**.
- **Shard `sbfb/shard/1`** : `shard_handshake_admits_member :515` (fixture 2-nœuds vrai handshake
  QUIC 1.0.1 sur `conn.remote_id()`, frame echo) + `shard_handshake_rejects_non_member :537`
  (asserte le reject d'admission) + `shard_alpn_registered_in_router :460` +
  `shard_frame_roundtrip_two_nodes :488`. Fixture `two_node_shard_fixture_with :422` via
  `create_node_with_protocols`. Le sous-test **T1 (5) « recompile + handshake shard » est déjà
  satisfait**.

**Correction load-bearing (S3-8 REFUTED par sa propre vérif adversariale)** : le scan S3 avait
classé le handshake shard **A-TESTER / risque d'affaiblissement silencieux de l'admission**. C'est
**faux** : l'admission `is_member(conn.remote_id())` AVANT `accept_bi` (`shard.rs:299-308`) est
exercée par deux tests 2-nœuds passants sous 1.0.1 — exactement la classe NO-OP-test-proven que le
scan applique lui-même au seed. **Seul le RTT multipath LIVE reste non couvert** (le RTT
single-path loopback `rtt.is_some()` de `shard_conn_stats_exposes_rtt :564` est **déjà vert**).

**Conséquence delta tests** : le budget « +1..2 handshake seed » du plan est **redondant** ; net-new
handshake ≈ 0. Le vrai +1 net est le **tripwire pkarr** (§5). E doit **résister à ré-implémenter
des tests existants** (zombies) et acter au body que le delta hermétique réel = +1.

---

## 4. Le VRAI travail — item 1 (DOC-STALE, lot E)

5 doc-comments à corriger dans des fichiers transport de la Phase E, **1 carry déjà fermé** :

| Site | Contenu stale | Correction | Note |
|---|---|---|---|
| `gossip.rs:259` | « iroh-gossip 0.97 » | → « iroh-gossip 0.101 » | doc-only, pin B |
| `gossip.rs:740` | « iroh 0.98 API surface » | → « iroh 1.0.1 API surface » (ou générique) | doc-only |
| `tls_pinning.rs:32-37` | « iroh 0.98 relay client » + context7 2026-04-16 + chemin `relay::client::ClientBuilder` | re-dater **ET re-vérifier le FOND** : en 1.0.1 le client relay vit sous `iroh_relay` ; confirmer si un hook `ServerCertVerifier` public existe enfin AVANT de re-dater | **JAMAIS un sed aveugle** (leçon C §5.1 : reformuler le fond, pas la version ; T20 jamais câblé) |
| `transport_probe.rs:22-26` | « iroh 0.91 … 0.97 inherits that behaviour » | recalibrer (« still true under 1.0.1 — WSS/TLS over TCP 443 only ») | doc-only, fichier E |
| `relay_config.rs:5` + `:18-20` | « **three** n0-run relays » + « Matches pre-Sprint 18 behaviour **byte-for-byte** » | → **« four »** (`defaults.rs:35-42` = use1-1/usw1-1/euc1-1/aps1-1) + **retirer/corriger « byte-for-byte »** : les hostnames n0 ont changé `*.iroh-canary.iroh.link`→`*.iroh.link` (label `iroh-canary` retiré), le set N'EST PAS byte-identique post-upgrade | **manque S2 — NOUVEAU doc-stale**, bug pré-existant mais soldé ici ; auto-adopté par `presets::N0`, 0 hardcode SBFB à patcher |

**DÉJÀ FERMÉ — retirer du lot** : `http.rs:3213` porte désormais « re-checked against 1.0.1 at the
S81 Phase C bump: only per-peer `Endpoint::remote_info(EndpointId)` exists — the `remote_info_iter`
once expected "post-0.98" never landed ». Le repère mémoire « http.rs:3213 stale » est lui-même
périmé. **NO-OP en E.**

**Hors-scope-E → CARRY K** : `age_witness.rs:6/21` (« iroh 0.98 does not expose an intrinsic
node-id ») — fichier attestations, pas transport. Lot doc-stale global iroh-version.

---

## 5. Le VRAI travail — item 2 (TEST + CHECK NOMMÉ) : survie URL pkarr/relais

### 5.1 La survie URL est DEUX vérifications, jamais une seule

Le sophisme central à bannir : **« recompile = URL servie »**. Le CHECK NOMMÉ se scinde en :

**(a) Tripwire HERMÉTIQUE (le +1 net de E-core, A-TESTER)** — assertion statique de parité :
`DEFAULT_PKARR_RELAY_URL == iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD` + `Url::parse`. Le chemin
public est atteignable (`address_lookup.rs:123 pub mod pkarr; :128 pub use pkarr::*;` ;
`pkarr_resolver.rs:39` importe déjà `iroh::address_lookup::pkarr::PkarrRelayClient`). Ce test
**attrape tout drift amont** au prochain bump iroh. **Résultat déjà connu : PASSE** — l'URL est
**byte-identique** `pkarr_resolver.rs:55` = `iroh-1.0.1/src/address_lookup/pkarr.rs:127` =
`"https://dns.iroh.link/pkarr"` (et inchangée depuis 0.98.2 `pkarr.rs:133`). La moitié statique du
check est donc **PROUVÉE** ; les 8 tests hermétiques pkarr existants (`pkarr_resolver.rs:204-318`,
label/parse/env) sont déjà verts — **ne pas les re-tester**.

**(b) Sonde LIVE (artefact acceptance T2, JAMAIS un unit test)** — résolution réseau réelle vers
`dns.iroh.link/pkarr` + relais n0. C'est un **fait runtime** non prouvable par compile
(SEMANTIQUE-NON-VERIFIEE). Interdit en unit test (hermétisme + fuite DNS). → artefact
`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT` machine-lisible. **Nuance à mettre au body** : la discovery
**par défaut** ne passe PAS par notre `DEFAULT_PKARR_RELAY_URL` (opt-in `SBFB_PKARR_RELAYS`, doc
`:49-54` « deliberately does NOT wire this URL ») — elle passe par `presets::N0` →
`N0_DNS_PKARR_RELAY_PROD` **interne à iroh**, lui aussi inchangé. **Piège `IROH_FORCE_STAGING_RELAYS`
(manque S1a2)** : sous cet env, `n0_dns()` sélectionne `N0_DNS_PKARR_RELAY_STAGING`
(`pkarr.rs:177-178`) — la sonde live frappe la cible sélectionnée par l'env, pas toujours prod ;
à noter dans le CHECK NOMMÉ.

### 5.2 Survie URLs relais N0 (même classe, static-résolu POSITIVEMENT)

`presets::N0` (`node.rs:318`) + `default_relay_map()` récupèrent les défauts **vendored** — SBFB ne
code **aucun hostname en dur** → le bump bascule **automatiquement** sur la flotte 1.0 vivante
(0-code). **Constat vendored (renforce S2-10)** : les hostnames relais ont **concrètement changé**
`iroh-0.98.2/defaults.rs:27-33 (*.iroh-canary.iroh.link)` → `iroh-1.0.1/defaults.rs:27-33
(*.iroh.link)` — c'est exactement le scénario blog n0 « wire-breaking relay changes get new URLs ».
Le nouveau set est auto-adopté ; le résidu = **sonde live** des URLs 1.0.1 (T2). **EOL n0 confirmé
2026-09-30** — l'upgrade =1.0.1 EST la remédiation (canon-projet ; à traiter comme tel, pas comme
fait web fraîchement re-sourcé — correction S1a2-4).

---

## 6. Adaptation structurelle #1 — zéro-n0 exige du CODE NEUF (sous-scopé par le plan)

Le finding le plus lourd du préflight (S1a2-10/12, S3-6, confirmés adversarialement). La lettre
liste « re-cert `node.rs:318` presets::N0 » comme suffisant pour PLAN B C8. **C'est faux** :

- **`presets::N0` câble une discovery ambiante en dur** : `presets.rs:121
  builder.address_lookup(PkarrPublisher::n0_dns())` + `:128 PkarrResolver::n0_dns()` + `:133
  DnsAddressLookup::n0_dns()` — publie ET résout via `dns.iroh.link`.
- **`node.rs:318` `.address_lookup(memory_lookup)` est ADDITIF** : `endpoint.rs:605` **pousse**
  (`self.address_lookup.push`), il n'écrase pas ; `clear_address_lookup` est une méthode **séparée**
  (`endpoint.rs:585`). `node.rs:348` n'override que `relay_mode(RelayMode::Custom)`.
- **Corroboration** : un SEUL `Endpoint::builder` dans tout le workspace ; aucun
  `clear_address_lookup` / preset `Minimal` / `RelayMode::Disabled` en prod. `SBFB_CUSTOM_RELAYS`
  ne remplace QUE le relais. `SBFB_PKARR_RELAYS` ne nourrit QUE le canari quorum anti-eclipse
  (`browse.rs` `quorum_resolvers :294`, `runtime.rs:461-488` « browse probes use the default iroh
  N0 discovery path ») — **faux ami pour zéro-n0**, le régler ne retire aucune dépendance n0.

**Donc « le réseau tient sans aucun service n0 » exige un NOUVEAU surface de code** : preset
`Minimal` (`presets.rs:59`) ou `N0` + `clear_address_lookup` + `PkarrPublisher::builder(self_url)`
(`pkarr.rs:290`) + `PkarrResolver::builder(self_url)` (`pkarr.rs:507`) + `RelayMode::Custom(self
relay map)`, **gated par env**. **Ce n'est PAS un DESIGN-CONFLICT** — toutes les briques existent en
1.0.1, aucun arbitrage PO requis ; c'est une **correction de plan** (provisioning seul insuffisant).
Décision de design Phase E à trancher côté résolveur non-browser : `DnsAddressLookup` interroge du
**DNS** (`presets.rs:131-134 #[cfg(not(wasm_browser))]`) → zéro-n0 non-browser exige un
`iroh-dns-server`/`pkdns` OU une bascule sur `PkarrResolver` HTTP (self-hosted pkarr relay).

---

## 7. PLAN B C8 — périmètre EXACT du livrable + décision split E'

### 7.1 Les briques self-hosted existent toutes (0 dep neuve, pas de DESIGN-CONFLICT)

- **Relais** : `iroh-relay 1.0.1` = **MÊME crate déjà lockée** (`Cargo.lock:4160`), feature `server`
  (`iroh-relay-1.0.1/Cargo.toml:71`) + bin `iroh-relay` (`:119-120`, `--dev`→`localhost:3340`) +
  Docker `n0computer/iroh-relay`. **Wire-compat 1.0.1 par construction** (même version).
- **pkarr** : `pkarr` **N'EST PAS une dep** (`grep Cargo.lock 'name = pkarr'` = vide) — iroh
  implémente son propre `PkarrRelayClient` HTTP (`pkarr.rs:554-566`). « pkarr self-hosted » = faire
  tourner `iroh-dns-server` (= ce qu'EST `dns.iroh.link` : pkarr relay + DNS combinés) / `pkarr-relay`
  (pubky), OU basculer sur `PkarrResolver` HTTP pour éviter un serveur DNS.

### 7.2 Tension lettre-vs-histoire (réconciliable, pas un conflit)

Le **kickoff D2** qualifie le relais self-hosted d'**OPTIONNEL** (résilience VPS) ; le **plan E** le
rend **OBLIGATOIRE pré-provisionné**. Réconciliation : D2 = chemin normal (presets::N0 conservé par
défaut, verrou-3), C8 = **hedge fallback** armé face à l'EOL n0 30/09. Le default `presets::N0`
reste — C8 ne le retire pas, il **ajoute** un mode zéro-n0 activable. Aucun Day-0 touché.

### 7.3 Périmètre EXACT du livrable PLAN B C8 (= le contenu de E')

- **(a) [in-E' cheap]** Runbook committé + templates config (`relays.json` / `SBFB_CUSTOM_RELAYS`).
- **(b) [in-E' — le vrai travail, CODE NEUF]** Override discovery `node.rs` gated par env
  (`clear_address_lookup` + `PkarrPublisher::builder` + `PkarrResolver::builder` +
  `RelayMode::Custom`) — §6.
- **(c) [in-E' infra]** Build `iroh-relay` feature `server` OU Docker sur l'**ancre VPS Hetzner
  existante** (cf. `live_acceptance_setup`).
- **(d) [in-E' infra]** pkarr/DNS self-hosted (`iroh-dns-server`) OU `PkarrResolver` HTTP.
- **(e) [in-E' LIVE]** Acceptance zéro-n0 2-nœuds = artefact **T2** JSON
  (`PASS`/`BLOCK`/`RIG-ABSENT`) — RIG-gated comme les autres live ; **RIG-ABSENT tracé** acceptable
  si matériel absent.
- **Test hermétique modèle** : `iroh::test_utils::DnsPkarrServer` (pkarr relay + DNS in-process)
  existe (`address_lookup.rs:1243-1351`) MAIS derrière la feature `test-utils` **activée nulle part**
  (correction S1a2-16 : réutilisation possible mais exige d'ajouter `iroh features=["test-utils"]`
  en dev-dep — **PAS code-free** ; à peser vs budget, candidat E').

### 7.4 Décision split E' : **OUI**

**Périmètre E'** = **PLAN B C8 intégral** (7.3 a-e). **Justification** : (1) le portage
handshake/compile est **déjà vert** (§2/§3), donc E-core est mécanique et léger ; (2) le seul
travail lourd est le **CODE NEUF discovery-override + provisionnement relais/pkarr self-hosted +
acceptance zéro-n0 LIVE (2-4 j)**, qui **déborde explicitement « compile + handshake SEULEMENT »** ;
(3) précédent A3→A4 cité par le plan ; (4) 4 scans convergent (S2-19/S2-20, S3-6, S1a2-21). Le split
respecte les **gates calendaires** (01/08 = provisionner → E' ; 25/08 = basculer flotte ; 15/09 =
plan B ACTIF, EOL 30/09). **E-core = ce commit** (NO-OP re-cert + doc-stale + tripwire pkarr + CHECK
NOMMÉ artefact) ; **E' = commit suivant** (C8). Contrairement au risque envisagé par le plan
(« split si le portage shard dépasse le mécanique »), **le déclencheur réel du split n'est PAS le
shard** (déjà vert) mais **le provisionnement C8**.

---

## 8. Plan de tests concret (delta recalculé)

**Contrainte de greffe** : le tripwire pkarr est un test unit **hermétique** dans
`crates/nexus-core-rs/src/pkarr_resolver.rs` (module test existant), **0 helper à hoister**. Pas de
dette WS-3/PD-5.

| Test | Type | Assertion | Garde |
|---|---|---|---|
| `default_pkarr_url_matches_iroh_upstream_const` | GREEN | `DEFAULT_PKARR_RELAY_URL == iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD` + `Url::parse` OK ; tripwire drift amont | **hermétique, 0 réseau, 0 store** |
| (E', hermétique modèle) discovery-override assemble | GREEN | `Minimal`/`clear_address_lookup` + `PkarrPublisher/Resolver::builder(self_url)` construisent un endpoint sans n0 | exige dev-dep `test-utils` — peser en E' |
| (E', LIVE) zéro-n0 2-nœuds | T2 artefact | 2 nœuds convergent via relais+pkarr self-hosted, 0 service n0 ; `b3` PASS | RIG-gated → `RIG-ABSENT` traçable |

**Explicitement NE PAS ajouter** : handshake seed 2-nœuds, handshake shard admits/rejects, pkarr
parse/label/env → **déjà couverts et verts** (§3, `pkarr_resolver.rs:204-318`). **NE PAS forger de
fixture 0.98 genuine** (non-scénario pre-launch = zombie, cf. CLAUDE.md).

**Delta tests réaliste E-core : +1 net Rust** (tripwire pkarr), −0 zombies. Le libellé « +1..2 »
est **honnête mais haut** : le handshake est déjà vert. **Delta E' : +0..1 Rust hermétique** (si
`test-utils` adopté) **+ 1 artefact T2 LIVE**. Acter au body que le delta handshake réel = 0.

---

## 9. Restitution des scans (fan-out 6 + adversarial)

| Scan | Verdict-local | Findings clés retenus (CONFIRMED / corrigés) | Adversarial |
|---|---|---|---|
| **S1a** API transport (shard.rs + Connection + seed) | **EXECUTE** | 14 items : imports/rtt/PathId/ProtocolHandler/AcceptError/VarInt/ReadExactError NO-OP ; rc.0 retracts vacuité ; handshake seed/shard **déjà vert** ; **S1A-5 RTT multipath LIVE = SEMANTIQUE-NON-VERIFIEE → I/J** | **14/14 CONFIRMED**. Corrections de LIGNES intégrées (S1A-2 rtt `970↔1016` ; S1A-6 `remote_id` = `Connection<HandshakeCompleted>` `:1127`↔`:1077` pas `Connecting:563` ; S1A-3 source = `noq-proto paths.rs:55` ; S1A-11 `on_accepting:248`). Manques absorbés : `MemoryLookup.add_endpoint_info`, `accept_bi/read_to_end/write_all/finish` |
| **S1a2** discovery pkarr + relais + PLAN B C8 | **PLAN-ADAPT** | URL pkarr byte-identique (tripwire faisable) ; **S1a2-10/12 zéro-n0 = CODE NEUF (A-CODER)** ; relais/pkarr self-hosted = briques existantes ; check nommé = 2 vérifs ; repères plan périmés (`:54`→`:55`) | 17 CONFIRMED, 0 REFUTED. Corrections : S1a2-16 `DnsPkarrServer` **pas code-free** (feature `test-utils`) ; S1a2-20 `DEFAULT_RELAY_QUIC_PORT=7842` **VÉRIFIÉ** (pas sous-vérifié). Manques : `discovery.rs TransportAddr`, `IROH_FORCE_STAGING_RELAYS` |
| **S1b** deps / CVE / lock | **EXECUTE** | pins B conformes ; **iroh 1.0 = fork `noq`, plus quinn** ; **quinn-proto 0.11.14 = LE FIX** RUSTSEC-2026-0037 (via reqwest, hors iroh) ; pkarr absent du lock ; TLS/ring propres ; iroh-family = max crates.io ; **0 action E** | **11/11 CONFIRMED**. Corrections mineures (plage `deny.toml`, portée `cargo tree -d`). Manques non-bloquants : `rand 0.8` ignore VALIDE (carry standing) ; unmaintained workspace-scopés |
| **S2** décisions historiques | **PLAN-ADAPT** | ALPN shard/seed FIGÉS ; rc.0 = 0 call-site ; shard/seed handshake déjà verts ; pkarr fixé EN B ; **résidu = checks nommés + doc-stale + I/J** ; **PLAN B C8 = candidat split E'** | 19 CONFIRMED, 0 REFUTED. Corrections : S2-9 URL pkarr byte-identique 0.98↔1.0.1 (moitié statique du check PASSE) ; S2-3 « 7 tests shard » pas 5. **Manque MATÉRIEL** : `relay_config.rs:5/:18-20` stale « three/byte-for-byte » (§4) |
| **S3** threat model | **PLAN-ADAPT** | seed auth `DOMAIN_SEED_REQUEST_V1` intacte (crate hermétique) ; duress inchangée ; **S3-5 silent-loss discovery → CARRY G** ; **S3-6 zéro-n0 = A-CODER** ; verrous S74/S75 non touchés | 7 CONFIRMED, **1 REFUTED (S3-8** : shard admission **déjà testée** admits/rejects, pas A-TESTER). Manques : tests handshake shard existants, symétrie duress `ShardProtocol::accept`, `MAX_SEED_MSG_BYTES` |
| **S4** wire + call-sites D7 | **EXECUTE** | 2 ALPN + tous `*_FORMAT_VERSION`/`DOMAIN_*_V1` verbatim ; 0 format persisté par les tests E ; call-sites daemon/daemon-core compile B ; anchors/tickets = territoire D/F | 17 CONFIRMED. Corrections : **23** tags pas 21 (S4-3) ; `remote_info` `http.rs:3215` = doc seul pas call-site (S4-8) ; seul `rtt` non-couvert (S4-13, pas remote_id/close/closed). **Manque MAJEUR** : suite handshake seed/shard **déjà existante** (delta plan largement couvert) |

**Convergence** : 6 scans → **PLAN-ADAPT global** (S1a/S1b/S4 EXECUTE-local sans action E ; S1a2/S2/S3
PLAN-ADAPT sur zéro-n0 + checks nommés + split E'). Aucun REFUTED n'inverse le verdict : le seul
REFUTED matériel (S3-8) **renforce** le recadrage (shard handshake DÉJÀ vert → delta ≈ 0).

---

## 10. Carries sortants (créés / re-routés E → E', F, G, K, I-J)

1. **E' (PLAN B C8, gated 01/08)** : (a) CODE NEUF override discovery gated-env (§6) ; (b) relais
   iroh self-hosted (feature `server`/Docker VPS Hetzner) ; (c) pkarr self-hosted
   (`iroh-dns-server`/`PkarrResolver` HTTP) ; (d) acceptance zéro-n0 LIVE 2-nœuds = artefact T2
   (RIG-ABSENT traçable) ; (e) éventuel dev-dep `iroh features=["test-utils"]` pour `DnsPkarrServer`.
2. **F** (déjà ouvert par D) : dualité redb (docs one-way / blobs hard-fail), durabilité pins
   keep-online M18, anchors.json graceful-degrade, snapshot Mac. **E ne touche pas F.**
3. **G (THREAT_MODEL / doc)** : (a) **silent-loss discovery (S3-5)** — mort `dns.iroh.link` = warn-only
   iroh (`pkarr.rs:386-391`), 0 signal santé SBFB (grep `discovery_health` = 0) ; ligne THREAT_MODEL
   « availability-silence » + sonde boot bruyante optionnelle ; (b) delta menace PLAN B (relais
   self-hosted = opérateur voit métadonnées, même classe que n0 déplacée ; pkarr self-hosted =
   autorité résolution n0→opérateur bornée par paquet signé, « no central server » préservé BLAKE3 +
   Ed25519) ; recommandation ≥2 relais pkarr distincts dans le canari `dht_quorum` ; (c) EOL n0
   30/09 (remédié par upgrade) ; (d) **quinn-proto 0.11.14 = LE FIX** RUSTSEC-2026-0037 — reformuler
   le carry mémoire (« RÉSOLU », pas « PLAUSIBLE vulnérable ») ; (e) veille `noq-proto` fork-DoS ;
   (f) gate `cargo-deny` `multiple-versions warn→deny` ; (g) `rand 0.8` ignore VALIDE (standing) ;
   (h) symétrie duress `ShardProtocol::accept` non duress-gaté (risque nul : leurre non-membre).
4. **K (dette / doc)** : `age_witness.rs:6/21` doc-stale 0.98 ; repères plan périmés (`pkarr_resolver`
   URL `:54`→`:55`, `new()` `:107-115`→`:112-121`, `node.rs:329` = `load_relay_map` pas
   `default_relay_map`) ; sites BlobTicket daemon (territoire D).
5. **I-J** : **RTT multipath LIVE cross-machine** (`shard.rs` UNVERIFIED-high-risk, décision PO C1) —
   compile + handshake OK en E, **jamais claimer « shard SAUVÉ/stable verbatim »**.
6. **Veille continue** : re-jouer crates.io (1.0.2/0.102 ?) + RustSEC avant push live (à 2026-07-05 :
   iroh 1.0.1 / docs+gossip 0.101 / blobs 0.103 = plafonds stables).

---

## 11. Invariants & Day-0 (tenus)

- **0 bump wire SBFB (par construction)** : `SEED_ALPN=b"sbfb/seed/0"` (`node.rs:68`) +
  `SHARD_ALPN=b"sbfb/shard/1"` (`node.rs:80`) verbatim ; **23** tags `DOMAIN_*_V1`
  (`canonical.rs:77-332`) ni renommés ni retaggés ; tous `*_FORMAT_VERSION=1` ; le tripwire pkarr et
  l'override discovery E' ne créent **aucun format persisté** (URL `String`, config env).
- **iroh STRICTEMENT SEUL (D7)** : 0 dep runtime ajoutée. `iroh-relay`/`pkarr` self-hosted = MÊME
  crate déjà lockée / service externe (pas une dep). L'éventuel `test-utils` (E') = dev-dep, pas
  runtime.
- **Décisions Day-0 figées** : ALPN shard (S77) + seed (S74) non re-débattus ; posture WebPKI
  INCHANGÉE (B — `CaTlsConfig::default` == roots Mozilla 0.98) ; `presets::N0` conservé par défaut
  (C8 ajoute un mode zéro-n0, ne retire pas le défaut) ; verrous anti-recentralisation S74/S75
  intacts ; toolchain 1.94.
- **Tests hermétiques uniquement en E-core** : tripwire pkarr = 0 réseau/0 store ; handshakes
  in-process `create_node_with_protocols`. La sonde live = **acceptance T2**, jamais un unit test.
- **Bisectabilité** : la recompilation appartient à B ; E-core = **doc-comments + 1 test + artefact**
  (0 changement fonctionnel de code prod). Ne pas rétro-amender B.
- **Total de tests jamais en baisse silencieuse** : E-core +1, −0.

---

## 12. Risques résiduels

- **shard RTT multipath LIVE (P1, I/J)** : compile + handshake + RTT single-path loopback verts,
  mais per-path multipath cross-machine non prouvé → feature shard PROVISIONAL, borné I/J (PO C1).
  **NE JAMAIS déclarer « shard SAUVÉ » en E.**
- **Casse SILENCIEUSE de discovery (P1, S3-5 → G)** : si `dns.iroh.link/pkarr` ou les relais n0
  meurent, aucun crash — warn-only, 0 signal SBFB. Mitigé partiellement (MemoryLookup/tickets +
  keepalive S77 pour pairs connus) ; net-new discovery reste muet. Le CHECK NOMMÉ live (T2) +
  PLAN B C8 (E') sont la remédiation.
- **Débordement de scope via PLAN B C8 (P1)** : 2-4 j de provisionnement + CODE NEUF > « compile +
  handshake SEULEMENT ». **Résolu par le split E'** (§7.4).
- **Piège doc-stale `tls_pinning.rs:32` (P3)** : re-dater sans re-vérifier le hook
  `ServerCertVerifier` public en 1.0.1 produirait un commentaire FAUX (primitive S19 jamais câblée,
  T20). Appliquer la contrainte C §5.1 (reformuler le FOND).
- **Zombies/tests redondants (P2)** : le « +1..2 handshake » du plan est déjà couvert — E doit
  résister à ré-implémenter, acter delta handshake = 0 au body.
- **`IROH_FORCE_STAGING_RELAYS` (P3)** : sous cet env, l'endpoint résout via staging — la sonde live
  frappe la cible sélectionnée par l'env, à noter dans le CHECK NOMMÉ.
- **Env session (report 05/07)** : reboots/fins de session tuent les shells en vol → suites longues
  en avant-plan ou re-vérif output ; piège `cmd | tail && ...` masque les exit codes (pipefail).
  Non pertinent pour E-core (tests unit/hermétiques) ; pertinent pour l'acceptance E' LIVE.

---

## 13. Commit shape (indicatif — E-core)

`test(core): Sprint 81 Phase E — surfaces transport re-certifiées compile+handshake sous iroh 1.0.1
(NO-OP + doc-stale + tripwire pkarr, 0 bump wire)` — body : surface transport
shard.rs/seed_protocol.rs/pkarr_resolver.rs/relay_config.rs/node.rs **NO-OP compile-prouvée**
(byte-diff vendored : `Connection::rtt(PathId)` signature identique `970↔1016`, `PathId::ZERO`,
`ProtocolHandler` 1-méthode, `AcceptError #[non_exhaustive]` non-cassant, retraits rc.0 = 0
call-site vacuité, absorbés par bump B) + **handshake seed `sbfb/seed/0` ET shard `sbfb/shard/1`
in-process DÉJÀ verts** (13 tokio seed + admits/rejects shard — delta handshake ≈ 0, S3-8 réhabilité)
+ **+1 tripwire hermétique pkarr** (`DEFAULT_PKARR_RELAY_URL == iroh N0_DNS_PKARR_RELAY_PROD`,
byte-identique — moitié statique du CHECK NOMMÉ PASSE) + **CHECK NOMMÉ live = artefact T2**
(survie URL runtime, jamais unit test ; `IROH_FORCE_STAGING_RELAYS` noté) + doc-stale lot
(gossip.rs:259/:740, tls_pinning.rs:32 fond re-vérifié, transport_probe.rs:22-26,
relay_config.rs:5/:18-20 « three→four » + « byte-for-byte » corrigé — hostnames n0
iroh-canary→iroh.link ; http.rs:3213 DÉJÀ fermé C) + **RTT multipath LIVE → I/J** (shard
UNVERIFIED-high-risk, jamais « SAUVÉ ») + **PLAN B C8 zéro-n0 = CODE NEUF override discovery →
SPLIT E'** (presets::N0 câble n0 en dur, override actuel = relais seul ; briques 1.0.1 existent,
pas de DESIGN-CONFLICT) + carries E'(C8)/F(redb)/G(silent-loss+quinn-proto RÉSOLU+threat PLAN
B)/K(age_witness+repères)/I-J(RTT) + 0 bump wire (23 DOMAIN_*_V1, ALPN verbatim) + iroh strictement
seul + toolchain 1.94 + tests hermétiques.
