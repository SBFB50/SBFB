# Sprint 82 Phase K — Preflight G8 (bump hickory-resolver 0.24→0.26)

## Contexte + méthode (Workflow multi-agents, 2026-07-15)

Phase K solde le carry **HICKORY-024-RUSTSEC** : bump `hickory-resolver` 0.24 → 0.26,
réécriture de la construction du resolver (churn API réel de la ligne 0.25/0.26), retrait
des 4 `ignore` `deny.toml`, clôture des 4 RUSTSEC vivants. Cas B (pre-code). PO-7=A, D11
(« 0 dep runtime ajoutée HORS hickory »), iroh reste `=1.0.1` STRICTEMENT.

Cinq scans factuels (S1a SOTA/API, S1b graphe deps+advisories, S2 décisions historiques,
S3 threat model, S4 wire/frontière) ont été produits puis **vérifiés adversarialement** à la
source primaire (docs.rs 0.26.1, source au tag `v0.26.1`, crates.io, advisory-db RustSec,
`cargo tree -i`, Context7, `Cargo.lock`, lecture disque du fichier réel). Le WRITER a
re-vérifié `dns_fallback.rs` intégralement (569 l) et l'API `Lookup`/`txt_lookup` 0.26 via
Context7 avant synthèse.

**Bilan des vérifications** : 6 claims REFUTED (dont 4 compile-breaking), toutes corrigées
ci-dessous ; la version du vérificateur (source primaire) fait foi. Le plan §Phase K est
réalisable mais **7 corrections concrètes** doivent être câblées dans le code (features
réécrites, fichier dans `nexus-core-rs`, extraction TXT champ-vs-méthode, `webpki-roots`
obligatoire, `build()` faillible, 1 test retiré, commentaires/docs rafraîchis). → **PLAN-ADAPT**.

**Drift plan à consigner (confirmé 4×)** : le kickoff/plan cite
`crates/nexus-shell-daemon/src/dns_fallback.rs` — ce fichier **n'existe pas**. Le consommateur
réel est **`crates/nexus-core-rs/src/dns_fallback.rs`** (569 l) ; l'intégration est
`crates/nexus-shell-daemon-core/src/browse.rs`.

---

## S1a — SOTA / API hickory 0.26 (squelette de code corrigé)

### Version cible & MSRV
- Dernière patch = **`0.26.1`** (2026-05-01 ; `0.26.0` = 2026-04-16). MSRV = **Rust 1.88**
  → compatible Rust 1.94 (marge). `[workspace.package] rust-version = "1.88"` au tag `v0.26.1`.

### Les 5 surfaces API qui changent (toutes CONFIRMED source primaire)

**1. `TokioAsyncResolver::tokio(config, opts)` → builder + provider.**
- `TokioAsyncResolver` **supprimé**. Type = `Resolver<P: ConnectionProvider>`, alias
  **`TokioResolver = Resolver<TokioRuntimeProvider>`**.
- Construction = `Resolver::builder_with_config(config: ResolverConfig, provider: P) ->
  ResolverBuilder<P>` (retourne le builder **directement, pas un `Result`**), puis `.build()`.
- **`build()` est FAILLIBLE** : `pub fn build(self) -> Result<Resolver<P>, NetError>`
  (l'ancien `::tokio` était infaillible → mapper `NetError` dans le type d'erreur courant).
- Options : `builder.options_mut() -> &mut ResolverOpts` (ou `with_options(self, ResolverOpts)`).
- Imports : `use hickory_resolver::Resolver;` + `use hickory_resolver::net::runtime::TokioRuntimeProvider;`
  (`pub use hickory_net as net;` au crate root). Le 404 docs.rs sur `net/runtime/...` est un
  artefact de rendu des re-exports, **pas** un chemin invalide (exemples doc verbatim).

**2. `NameServerConfig` — struct-literal MORT (`#[non_exhaustive]`).**
- Champs 0.26 : `pub ip: IpAddr`, `pub trust_negative_responses: bool`,
  `pub connections: Vec<ConnectionConfig>`. **Disparus** : `socket_addr`, `protocol`,
  `tls_dns_name`, `tls_config`, `bind_addr`. Le struct-literal `dns_fallback.rs:203-210`
  **ne compile plus**.
- Constructeur : `NameServerConfig::new(ip: IpAddr, trust_negative_responses: bool,
  connections: Vec<ConnectionConfig>) -> Self`.
- `ConnectionConfig` (`#[non_exhaustive]`) : `pub port: u16`, `pub protocol: ProtocolConfig`,
  `pub bind_addr: Option<SocketAddr>`. Ctors : `ConnectionConfig::tls(server_name: Arc<str>)`,
  `ConnectionConfig::https(server_name: Arc<str>, path: Option<Arc<str>>)`, `::new(ProtocolConfig)`.
- **TLS name PAR endpoint (P2-E-1) PRÉSERVÉ** : `server_name: Arc<str>` passé à chaque ctor →
  un nom par endpoint, jamais global. Conversion : `Arc::from(ep.tls_name.as_str())`.
- **Port configurable** : `::tls`/`::https` défaut 853/443 ; muter `conn.port = ep.port;` après
  construction (le champ `port` est `pub` ; `#[non_exhaustive]` interdit le struct-literal, PAS
  l'écriture d'un champ public sur une instance déjà obtenue).
- **`trust_negative_responses` : défaut basculé à `true` en 0.26** → passer `false` en 2ᵉ arg de
  `NameServerConfig::new(ip, false, ..)` est désormais **MANDATORY explicite** (préservation active,
  plus passive), sinon le caching négatif défait la course DoH/DoT (contrainte S2-3).

**3. `Protocol` (enum config) DISPARU → `ProtocolConfig`.**
- Plus d'enum `Protocol { Udp, Tcp, Tls, Https, ... }` dans le module config. Remplacé par
  `ProtocolConfig { Udp, Tcp, Tls{server_name}, Https{server_name,path}, Quic{..}, H3{..} }`.
- Conséquence : la signature `build_resolver(protocol: Protocol)` et le test
  `build_resolver_rejects_unsupported_protocol` (`Protocol::Udp`, `dns_fallback.rs:557`) **ne
  compilent plus**. Remplacer par un **enum local `DnsTransport { Doh, Dot }`** (2 variantes) →
  « protocole non supporté » devient irreprésentable par le type (durcissement type-safety).

**4. `ResolverConfig::from_parts` + `NameServerConfigGroup` supprimé.**
- `from_parts(domain: Option<Name>, search: Vec<Name>, name_servers: Vec<NameServerConfig>) -> Self`.
  On passe un **`Vec<NameServerConfig>` nu**. **`NameServerConfigGroup::with_capacity/push`
  N'EXISTE PLUS** → réécrire la boucle `dns_fallback.rs:201-211`.

**5. `txt_lookup` → `Lookup` générique (CHANGEMENT DE TYPE — piège compile-breaking).**
- `txt_lookup(&self, query: impl IntoName) -> Result<Lookup, NetError>`. `&str` implémente
  `IntoName` → l'argument `&str` actuel reste OK.
- **Retourne `Lookup` (générique), plus `TxtLookup`** (page 404 en 0.26.1). **Trois erreurs
  de l'ancien pattern `dns_fallback.rs:242-246`, toutes REFUTED à la source :**
  1. `Lookup::iter()` **n'existe plus** — le bloc `impl Lookup` (`crates/resolver/src/lookup.rs`
     v0.26.1) expose `answers()/authorities()/additionals() -> &[Record]`. Utiliser
     **`lookup.answers()`**.
  2. `Record::data` est un **champ public** de type `RData` (Context7 : `match &record.data
     { RData::CNAME(..) }`), **pas** une méthode.
  3. `TXT::txt_data` est un **champ public** `Box<[Box<[u8]>]>` (`crates/proto/src/rr/rdata/txt.rs`
     v0.26.1 : `pub txt_data: Box<[Box<[u8]>]>`), **pas** la méthode `txt_data()`. `&txt.txt_data`
     déréférence en `&[Box<[u8]>]` → `concat_txt_strings(&txt.txt_data)` reste valide, **signature
     `concat_txt_strings` inchangée**.
- Import : `use hickory_resolver::proto::rr::RData;` (`pub use hickory_proto as proto;`). Si le
  compilateur signale `record.data` comme méthode (`data() -> &RData` ou `Option<&RData>`), écrire
  `record.data()` / matcher `Some(RData::TXT(txt))` — le compilateur tranche ; les 3 faits porteurs
  (answers, RData::TXT, champ txt_data) sont fixés.

### Features Cargo — renommage + `webpki-roots` OBLIGATOIRE (correction de synthèse)
- Les anciennes features `dns-over-https-rustls` / `dns-over-rustls` **N'EXISTENT PLUS** (supprimées
  dès 0.25). Remplaçants : DoT `tls-ring` | `tls-aws-lc-rs` ; DoH `https-ring` | `https-aws-lc-rs` ;
  roots `webpki-roots` | `rustls-platform-verifier`.
- **Choix imposé D11 = variantes `-ring`** : `ring` est déjà dans l'arbre (via iroh + l'ancienne
  stack rustls) ; `aws-lc-rs` serait un **nouveau backend crypto** (toolchain C/NASM) = violation D11.
- **`webpki-roots` OBLIGATOIRE (correction vs S1b/S3 qui l'omettaient)** : en 0.26 `tls-ring`/
  `https-ring` **n'activent AUCUN fournisseur de racines** (`crates/net/Cargo.toml` v0.26.1 :
  defaults = `tokio` seul). Sans feature de roots, **le magasin de racines est vide → tous les
  handshakes DoH/DoT échouent au runtime** — invisible aux tests unitaires (pas de handshake réel).
  L'ancien pin 0.24 (`dns-over-*-rustls`) empaquetait implicitement `webpki-roots` ; l'activer
  explicitement **préserve le comportement 0.24** (roots Mozilla déterministes, cross-plateforme).
  Le crate `webpki-roots` (1.0.7) est **déjà dans le lock** → activer la feature **n'ajoute aucun
  crate**, D11 tenu. `rustls-platform-verifier` serait un **nouveau crate** → à éviter.

**Pin final** (`Cargo.toml:449`) :
```toml
hickory-resolver = { version = "0.26", features = ["tls-ring", "https-ring", "webpki-roots"] }
```

### Squelette de réécriture concret (0.26) — corrigé
```rust
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hickory_resolver::Resolver;
use hickory_resolver::config::{ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RData;
use tracing::debug;

use crate::error::{NexusError, Result};

// 0.26 : TokioAsyncResolver -> Resolver<TokioRuntimeProvider> (alias TokioResolver).
type TokioResolver = Resolver<TokioRuntimeProvider>;

// Remplace l'enum hickory `Protocol` (disparu). Le fallback ne parle QUE DoH/DoT :
// un enum à 2 variantes rend « protocole non supporté » irreprésentable (durcissement type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DnsTransport { Doh, Dot }

pub struct DnsFallbackResolver {
    doh_resolver: TokioResolver,
    dot_resolver: TokioResolver,
    domain_suffix: String,
}

impl DnsFallbackResolver {
    pub fn new(config: &DnsFallbackConfig) -> Result<Self> {
        let doh_resolver = Self::build_resolver(&config.doh_endpoints, DnsTransport::Doh, config)?;
        let dot_resolver = Self::build_resolver(&config.dot_endpoints, DnsTransport::Dot, config)?;
        Ok(Self { doh_resolver, dot_resolver, domain_suffix: config.domain_suffix.clone() })
    }

    fn build_resolver(
        endpoints: &[DnsEndpoint],
        transport: DnsTransport,
        config: &DnsFallbackConfig,
    ) -> Result<TokioResolver> {
        if endpoints.is_empty() {
            return Err(NexusError::Endpoint(format!(
                "no DNS endpoints configured for transport {transport:?}"
            )));
        }

        let mut name_servers = Vec::with_capacity(endpoints.len());
        for ep in endpoints {
            let server_name: Arc<str> = Arc::from(ep.tls_name.as_str()); // TLS name PAR endpoint (P2-E-1)
            let mut conn = match transport {
                DnsTransport::Doh => ConnectionConfig::https(server_name, None), // None => "/dns-query"
                DnsTransport::Dot => ConnectionConfig::tls(server_name),
            };
            conn.port = ep.port; // défaut 443/853 ; port configurable préservé (champ pub)
            // NameServerConfig::new(ip, trust_negative_responses=false, connections)
            name_servers.push(NameServerConfig::new(ep.ip, false, vec![conn]));
        }

        let resolver_config = ResolverConfig::from_parts(None, vec![], name_servers);
        let mut builder =
            Resolver::builder_with_config(resolver_config, TokioRuntimeProvider::default());
        {
            let opts: &mut ResolverOpts = builder.options_mut();
            opts.timeout = config.timeout;
            opts.attempts = 2; // défaut 0.26 = 2 déjà ; explicite inoffensif
        }
        builder
            .build()
            .map_err(|e| NexusError::Endpoint(format!("failed to build DNS resolver: {e}")))
    }

    async fn resolve_txt_via(
        resolver: &TokioResolver,
        query: &str,
        protocol_label: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let lookup = resolver
            .txt_lookup(query)
            .await
            .map_err(|e| anyhow::anyhow!("{protocol_label} TXT lookup failed for {query}: {e}"))?;
        let mut data = Vec::new();
        for record in lookup.answers() {            // &[Record] -> &Record
            if let RData::TXT(txt) = &record.data {  // record.data = champ RData (cf. note compile)
                for segment in txt.txt_data.iter() { // champ Box<[Box<[u8]>]> -> &Box<[u8]>
                    data.extend_from_slice(segment);
                }
            }
        }
        if data.is_empty() {
            anyhow::bail!("{protocol_label} TXT lookup returned empty data for {query}");
        }
        Ok(data)
    }
    // build_query_name / new(config) / resolve_node (tokio::select! race) INCHANGÉS.
}
```
**Imports à RETIRER** : `TokioAsyncResolver`, `Protocol`, `NameServerConfigGroup`,
`std::net::SocketAddr` (`Ipv4Addr` reste utilisé par les constantes). **À AJOUTER** : `Resolver`,
`ConnectionConfig`, `TokioRuntimeProvider`, `RData`, `std::sync::Arc`.
Le type d'erreur de `build_resolver`/`new` est **`crate::error::Result` (`NexusError`)** ;
`resolve_txt_via`/`resolve_node` restent en **`anyhow::Result`** (confirmé disque).

---

## S1b — Graphe deps + advisories  (verdict scan : EXECUTE, 6/6 CONFIRMED)

### Fait dominant : l'arbre hickory 0.26 est DÉJÀ dans le lock (via iroh=1.0.1)
Le bump est un **retrait NET** de l'arbre 0.24 legacy, pas un ajout. `cargo tree -i` prouve que
chacun de `hickory-resolver 0.24.4`, `hickory-proto 0.24.4`, `rustls 0.21.12`,
`rustls-webpki 0.101.7`, `tokio-rustls 0.24.1` a pour **unique** parent la chaîne hickory-0.24
ancrée sur le pin direct `nexus-core-rs`. Bumper le pin les **supprime intégralement** ; l'arbre
0.26 équivalent (déjà présent via iroh) collapse de 2 versions → 1 :

| Crate | 0.24 (pin, retiré) | 0.26 (déjà là, via iroh) |
|---|---|---|
| hickory-resolver / -proto | 0.24.4 | 0.26.1 |
| rustls | 0.21.12 | 0.23.40 |
| rustls-webpki | 0.101.7 | 0.103.13 |
| tokio-rustls | 0.24.1 | 0.26.4 |

### Provider crypto — D11 tenu (renforcé)
- `ring 0.17.14` présent ; **`aws-lc-rs`/`aws-lc-sys` ABSENTS** (`cargo tree -i aws-lc-rs` → 0 match ;
  grep `aws.lc` sur `Cargo.lock` → 0).
- iroh active déjà `https-ring` (niveau resolver) sur hickory 0.26.1 ; `hickory-net/https-ring`
  active transitivement `hickory-net/tls-ring` (`tls-ring = ["tokio-rustls/ring", "__tls"]`), donc
  `tokio-rustls 0.26.4` + `rustls 0.23.40` + `ring 0.17.14` sont **déjà résolus, ring-backed**.
- **Conséquence** : ajouter `tls-ring` + `https-ring` au pin ne fait que basculer des features
  nommées dont tous les effets-feuille sont déjà compilés → **0 nouveau crate**. `webpki-roots 1.0.7`
  est déjà dans le lock → l'activer n'ajoute rien non plus. **aws-lc-rs reste absent = D11 tenu.**

### Les 4 advisories : clôturées par le bump (seuils EXACTS — 1 correction S3)
| Advisory | Crate | 1ʳᵉ version patchée | Lock 0.26 | Statut |
|---|---|---|---|---|
| RUSTSEC-2026-0119 (hickory-proto O(n²) name-compression **DoS**) | hickory-proto | **≥ 0.26.1** (PAS ≥0.25 ni 0.26.0) | 0.26.1 | CLÔT |
| RUSTSEC-2026-0098 (rustls-webpki URI name-constraint, **authentification**) | rustls-webpki | ≥ 0.103.12 | 0.103.13 | CLÔT |
| RUSTSEC-2026-0099 (rustls-webpki wildcard name-constraint, **authentification**) | rustls-webpki | ≥ 0.103.12 | 0.103.13 | CLÔT |
| RUSTSEC-2026-0104 (rustls-webpki panic parsing CRL, **DoS/availability**) | rustls-webpki | **≥ 0.103.13** (seuil dur) | 0.103.13 | CLÔT (borne exacte) |

> **Correction load-bearing (S3 REFUTED)** : la fermeture de 0119 exige **hickory-proto ≥ 0.26.1**
> (affected `0.3.1..=0.26.0`). hickory-resolver 0.26.1 dépend de `hickory-proto ^0.26.0` → `cargo
> update` obtient 0.26.1 (ferme). **Un pin figeant 0.26.0 laisserait 0119 OUVERT** → `cargo deny
> check advisories` ROUGE. Vérifier au run que le lock résout bien hickory-proto ≥ 0.26.1 et
> rustls-webpki ≥ 0.103.13.

### Duplication / yanked / licences / sources
- **multiple-versions=warn** (non bloquant) : le bump **réduit** les groupes (rustls, rustls-webpki,
  tokio-rustls, hickory-* passent de 2 → 1 version). Ne pas toucher `[bans]`.
- **yanked=deny** : aucune cible yanked (les versions 0.26 sont déjà résolues via iroh + baseline
  verte). **0 nouveau crate → 0 nouvelle licence** ; `ring` déjà allowlisté ; `aws-lc-rs` non
  ajouté. Tout reste `crates.io-index`.

---

## S2 — Décisions historiques (contraintes à préserver)  (16/17 CONFIRMED)

Traversée `git log --follow` : création S24-E `e9d69db`, P2-E-1/E-2 S25-A `2b674db`, edition-2024
S54-A `1d010b0`, pose des 4 ignores S81-G `50f05c1`. **Aucune décision historique ne bloque le
bump** ; toutes = contraintes que la réécriture DOIT préserver :

1. **Deux resolvers protocole-spécifiques racés `tokio::select!`** (P2-E-2) — worst-case
   `1× timeout`, pas `2×`. Garder DEUX `TokioResolver` indépendants (`dns_fallback.rs:260-309`
   inchangé) ; ne PAS fusionner en un resolver multi-protocole.
2. **TLS name PER-ENDPOINT** (P2-E-1) — un `NameServerConfig` par endpoint avec son `tls_name`
   propre (Cloudflare→`cloudflare-dns.com`, Google→`dns.google`) ; jamais `endpoints[0]`. Tests
   `per_endpoint_tls_name_used_doh/_dot` (`:505-547`) **restent verts** (ils n'assertent que
   `label()`, insensibles à la structure interne).
3. **`trust_negative_responses = false`** — MANDATORY explicite en 0.26 (défaut basculé à `true`,
   cf. S1a-2).
4. **Opt-in default-off — FAIT CARDINAL** : `DnsFallbackConfig::default().enabled = false`
   (`:127`) + gate `SBFB_DNS_FALLBACK_ENABLED` + `load_dns_fallback_from_env()` → `Ok(None)` si var
   absente (`:322-329`). Plancher d'exposition référencé par THREAT_MODEL/deny.toml. **Ne PAS toucher
   `load_dns_fallback_from_env` (pur `std::env`, 0 hickory).**
5. **Résolveurs Cloudflare + Google épinglés + redondance 2-résolveurs DoH ET DoT**, via constantes
   nommées existantes `DOH_CLOUDFLARE_IP`/`DOH_GOOGLE_IP`/`..._TLS_NAME`/`DOH_PORT`/`DOT_PORT`
   (`:45-64`, §6.9 constante-unique). Ne pas ré-inliner.
6. **DNS = pas un ancrage de confiance** ; pkarr Ed25519 en aval ; `resolve_node` renvoie
   `Vec<u8>` non interprété (« le record existe »). Signature + sémantique inchangées.
7. **Intégration `AllFailed` seulement, `NoMajority` JAMAIS surchargé** (`browse.rs`
   `BrowseAggregator::with_dns_fallback`). Le trait `DnsFallbackResolve` (label + resolve_node)
   NE change PAS — `browse.rs` + ses mocks en dépendent. Réécriture confinée à `build_resolver`
   (privé).
8. **Garde protocole ≠ DoH/DoT + endpoints vides** — comportement de rejet préservé ; le test
   `build_resolver_rejects_unsupported_protocol` (`Protocol::Udp`) devient type-vacant → **retiré**
   (voir §Approche). `build_resolver_rejects_empty_endpoints` : remplacer `Protocol::Https` par
   `DnsTransport::Doh`, reste valide.
9. **`timeout` (5 s, `DEFAULT_DNS_TIMEOUT`) + `attempts = 2`** préservés via `options_mut()`.
10. **`domain_suffix` + `build_query_name` (64-hex) + réassemblage TXT wire-order** inchangés au
    niveau logique ; seule l'extraction TXT interne est réécrite (S1a-5).

**Doc de clôture — classes intrinsèques SÉPARÉES** (leçon Codex PARTIEL S81-G, à ne pas re-fusionner) :
0119/0104 = **DoS/availability** ; 0098/0099 = **laxité name-constraint / authentification**.

**HORS scope K (NE PAS jouer)** : P2-AUDIT-2-RESIDUEL (`ed25519-dalek` non-convergent, `[bans]
multiple-versions=warn` à `deny.toml:150`) reste INTACT ; le fait neuf `ed25519-dalek 3.0.0 stable
2026-07-06` s'instruit à l'audit gate, pas ici. quick-xml RUSTSEC-2026-0194/0195 (`deny.toml:87-98`,
via iroh) RESTE.

---

## S3 — Threat model (liste fichier → action)  (7/8 CONFIRMED)

Le threat model du module reste **vrai à l'identique** post-bump (le doc-comment
`dns_fallback.rs:16-31` « DNS is not a trust anchor — pkarr records Ed25519-signed » est
version-agnostique, aucune hypothèse renversée). La validation cert DoH/DoT est **renforcée**
(rustls 0.21.12 → 0.23 : name-constraints correctes, plus de panic CRL) → **0 nouvelle frontière
d'admission, 0 nouvelle row STRIDE, 0 bump wire**. `dns_fallback.rs` **n'émet aucun `SecurityEvent`**
(uniquement `tracing::debug`, import `:41`) ; `nexus-events-core` a 0 référence DNS/hickory →
**aucun event ni writer à toucher** ; conserver les `debug!(...)` (`:275,279,282,293,297,300`).

| Emplacement | Action Phase K |
|---|---|
| `deny.toml:62-82` (bloc contexte hickory + argument résiduel bounded) | **SUPPRIMER** — sans objet une fois les advisories closes (la laxité name-constraint est CORRIGÉE, l'argument « intermediates hors trust path » `deny.toml:78-80` disparaît). |
| `deny.toml:83-86` (4 ignores 0119/0098/0099/0104) | **SUPPRIMER les 4.** |
| `deny.toml:87-98` (quick-xml 0194/0195, via iroh) + `:60` (rand) + `[bans]` `:141-150` | **NE PAS TOUCHER.** |
| `Cargo.toml:445` « RustSec scan 2026-04-21 : zero active advisories » | **STALE/FAUX → RÉÉCRIRE** au passé immuable : ex. « RustSec 2026-07-15 (S82 K) : RUSTSEC-2026-0119/0098/0099/0104 fermées par le bump 0.26 (hickory-proto 0.26.1 + rustls-webpki 0.103.13) ». |
| `Cargo.toml:446-448` « no new TLS backend added » | **RÉÉCRIRE** : nommer `tls-ring`/`https-ring`/`webpki-roots` ; RE-AFFIRMER que `ring` reste le seul backend (aucun `aws-lc-rs`), claim RE-VALIDÉ VRAI au lock. |
| `Cargo.toml:449-452` | **BUMP** `"0.24"→"0.26"`, features `["tls-ring","https-ring","webpki-roots"]`. |
| `docs/security/HARDENING_ROADMAP.md:3` (`last_validated`) | **RE-DATER** 2026-07-15 (S82 K). |
| `HARDENING_ROADMAP.md` `triggers_revalidate` (`:5-16`) | **AJOUTER** un trigger standing : « hickory-resolver breaking > 0.26 OU nouvelle RUSTSEC sur la chaîne rustls 0.23 / rustls-webpki des features `https-*`/`tls-*` ». |
| `HARDENING_ROADMAP.md` `audited_findings` (`:17-28`, dernière = S81-G `:28`) | **AJOUTER** une entrée datée 2026-07-15 S82 K (bump joué, 4 ignores retirés, 4 RUSTSEC closes, classes séparées, ring conservé). Ne PAS éditer `:28`. |
| `THREAT_MODEL.md:1808-1809` (changelog v15, carry) + `:1827` (v17, dernière) | **NE PAS réécrire** le passé ; **AJOUTER v18 (S82 Phase K, 2026-07-15)** : fermeture des 4 RUSTSEC, validation cert renforcée, ring conservé, 0 frontière/STRIDE/wire. PAS de nouvelle §15.x. |
| `crates/nexus-core-rs/src/dns_fallback.rs:16-31` (prose threat model) | **AUCUN edit correctness** — les seuls edits du fichier = construction resolver (S1a). |
| `EXTERNAL_AUDIT_SCOPE.md:89`, `VALIDATED_BLUEPRINT.md:143/:609`, `HARDENING_ROADMAP.md:478-479` | Past-records historiques → **hors scope K** (notes optionnelles non bloquantes). |

---

## S4 — Wire format + frontière (preuves)  (5/6 CONFIRMED, frontier N/A prouvé)

- **0 wire SBFB touché** : `grep _VERSION dns_fallback.rs` = 0 ; `grep hickory crates/**/*.rs` =
  uniquement `dns_fallback.rs:37-38` (les 2 `use`) + `nexus-core-rs/Cargo.toml` (commentaire +
  dep `:127`). `Task`/`ProjectAnnouncement`/`CuratorList`/`FeedEntry`/`DOMAIN_*`/JCS/
  `FEED_FORMAT_VERSION` **ne dépendent d'aucun symbole hickory**.
- **Trait `DnsFallbackResolve` STABLE et hickory-free** (`:81-92`) : `label(&self)->&str` +
  `resolve_node(&self,&str)->anyhow::Result<Vec<u8>>`. Consommateur unique = `BrowseAggregator`
  via `dyn DnsFallbackResolve` (`browse.rs:304/380/478`). Mock cross-crate `DnsFallbackMock`
  (`browse.rs:1670`) imite le trait seul. Le bump ne peut pas modifier la signature.
- **`frontier_closure = N/A` PROUVÉ** : `grep -i dns http.rs` = seul `:267` (anti-rebinding, sans
  rapport) ; `grep dns web/src` = 0 ; `grep dns sbfb-factory` = 3 (tous anti-rebinding). L'unique
  observabilité `browse.rs` = un `impl Debug` (`label()` statique), **non sérialisé HTTP**. Aucune
  API loopback / schéma front/operator / contrat d'app ne LIT la primitive. **Renfort** :
  `DnsFallbackResolver::new`/`load_dns_fallback_from_env` **n'ont aucun appelant de production**
  (test-only) — la primitive n'est même pas câblée au daemon.
- **Churn hickory confiné à `nexus-core-rs`** — correction S4 : le type le plus retravaillé est
  **`TokioAsyncResolver`** (champs `:168-169`, retour `build_resolver` `:189`, ctor `:218`, param
  `resolve_txt_via` `:232`), pas seulement `Protocol`. Tous **privés** → invisibles cross-crate. La
  conclusion parapluie (0 traversée de crate) tient.

---

## Vérification adversariale — table des claims

| # | Scan | Claim | Verdict | Correction retenue (source primaire fait foi) |
|---|---|---|---|---|
| 1 | S1a | `TokioAsyncResolver`→`Resolver<TokioRuntimeProvider>`/builder | CONFIRMED | Param générique nommé `P` (cosmétique). |
| 2 | S1a | `builder_with_config` retourne le builder (pas Result) ; `build()` faillible `->Result<_,NetError>` | CONFIRMED | Mapper `NetError`→`NexusError::Endpoint`. |
| 3 | S1a | `NameServerConfig`/`ConnectionConfig`/`ProtocolConfig` non_exhaustive + ctors | CONFIRMED | — |
| 4 | S1a | **Extraction TXT `lookup.iter()`+`txt.txt_data()`** | **REFUTED** | `Lookup::iter()` supprimé → **`lookup.answers()`** (`&[Record]`) ; `record.data` = **champ** `RData` ; `txt.txt_data` = **champ** `Box<[Box<[u8]>]>`. |
| 5 | S1a | Justification roots « défaut 0.26 = rustls-platform-verifier » | **REFUTED** | Défaut = `tokio` seul, **aucun** roots provider. **Recommandation `webpki-roots` MAINTENUE** (magasin vide sinon). |
| 6 | S1b | Bump = retrait net arbre 0.24 ; ring présent / aws-lc-rs absent ; D11 tenu | CONFIRMED | iroh active `https-ring` (resolver) ; `tls-ring` s'active transitivement via `hickory-net` → D11 renforcé. |
| 7 | S1b/S3 | Features = `["https-ring","tls-ring"]` | **CORRIGÉ (synthèse)** | Ajouter **`webpki-roots`** (omis par S1b/S3) sinon handshakes runtime KO ; ordre indifférent : `["tls-ring","https-ring","webpki-roots"]`. |
| 8 | S3 | RUSTSEC-2026-0119 fermé « hickory-proto ≥0.25 » | **REFUTED** | Seuil réel = **≥ 0.26.1** (0.25.x et 0.26.0 restent vulnérables). rustls-webpki 0104 = **≥ 0.103.13** (seuil dur). |
| 9 | S2 | `trust_negative_responses` « rester false » (préservation passive) | CONFIRMED (durci) | Défaut basculé à **`true`** → `false` MANDATORY explicite. |
| 10 | S2 | Features `dns-over-*-rustls` « à vérifier si renommées » | **REFUTED** | **Supprimées** dès 0.25 → conserver l'array = compile-fail. |
| 11 | S4 | « seul type hickory en signature = `Protocol` » | **REFUTED** | `TokioAsyncResolver` aussi (4 sites) ; conclusion « confiné, privé » tient. |
| 12 | S1a/b/S2/S3/S4 | Drift fichier, `Cargo.toml:445` stale, trait stable, frontier N/A, 0 SecurityEvent, 0 wire | CONFIRMED | — |

**UNVERIFIABLE / à lever au compile** (aucun bloque le verdict) :
- `record.data` champ vs `data()` méthode : si méthode, écrire `record.data()` (voire matcher
  `Some(RData::TXT(_))` si `Option`). Faits fixés : `answers()`, `RData::TXT`, champ `txt_data`.
- Ports par défaut 443/853 des ctors : sans effet (le code force `conn.port = ep.port`).
- **Empty-root-store invisible à T1** : ni `cargo deny` ni nextest n'exercent un handshake réel →
  `webpki-roots` est la mitigation ; risque runtime borné (module opt-in default-off, sans appelant
  de production). Recommander une probe live opt-in au run (non bloquante pour T1).

---

## Approche d'implémentation (étapes ordonnées)

1. **`Cargo.toml:449-452`** — pin `version = "0.26"`, features `["tls-ring", "https-ring",
   "webpki-roots"]`. Réécrire les commentaires `:445` (passé immuable, advisories closes) et
   `:446-448` (backend `ring` seul, aucun `aws-lc-rs`). Le commentaire `:401` (`frost-ed25519`) est
   **hors scope**, ne pas y toucher.
2. **`cargo update -p hickory-resolver`** (+ propagation) — vérifier au lock que **hickory-proto ≥
   0.26.1** et **rustls-webpki ≥ 0.103.13** ; confirmer `aws-lc-rs`/`rustls-platform-verifier`
   ABSENTS (`grep aws.lc Cargo.lock` = 0 ; `cargo tree -i rustls-platform-verifier` = 0).
3. **`crates/nexus-core-rs/src/dns_fallback.rs`** — réécrire imports (retirer `TokioAsyncResolver`,
   `Protocol`, `NameServerConfigGroup`, `SocketAddr` ; ajouter `Resolver`, `ConnectionConfig`,
   `TokioRuntimeProvider`, `RData`, `Arc`) ; introduire `type TokioResolver` + enum local
   `DnsTransport{Doh,Dot}` ; réécrire `build_resolver` (ctors + `NameServerConfig::new(ip, false,
   vec![conn])` + `from_parts(Vec)` + `builder_with_config(..).options_mut()..build().map_err(..)`) ;
   réécrire `resolve_txt_via` (`answers()` + `RData::TXT` + `&txt.txt_data`). `new`, `build_query_name`,
   `resolve_node` (course `tokio::select!`), `load_dns_fallback_from_env`, `concat_txt_strings`,
   constantes, doc-comment threat model = **INCHANGÉS**.
4. **Tests** — **RETIRER** `build_resolver_rejects_unsupported_protocol` (`:549-560`, type-vacant :
   `DnsTransport` rend l'état irreprésentable ; honnête `-1 test`, à documenter comme durcissement
   type-safety dans le body) ; **ADAPTER** `build_resolver_rejects_empty_endpoints` (`:562-568` :
   `Protocol::Https` → `DnsTransport::Doh`) ; les 8 autres tests (config, query-name, concat, env,
   P2-E-1 doh/dot) restent verts.
5. **`deny.toml`** — supprimer `:62-86` (bloc contexte + 4 ignores). CONSERVER `:87-98` (quick-xml),
   `:60` (rand), `[bans]` `:141-150`.
6. **Docs sécurité** — `HARDENING_ROADMAP.md` : re-dater `:3`, ajouter trigger standing +
   entrée `audited_findings` datée (classes 0119/0104 DoS vs 0098/0099 authentification SÉPARÉES).
   `THREAT_MODEL.md` : ajouter **v18** au changelog §16 (ne pas réécrire v15/v17). Passé immuable
   partout (PROMISE_RE scanne `dns_fallback.rs` + `nexus-core-rs/Cargo.toml`).
7. **Vérification (T1)** — dual-platform :
   - `cargo deny check advisories` **vert SANS les 4 ignores** (critère machine principal).
   - `cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo fmt --all --check`.
   - `cargo nextest run --workspace --locked` **≥ baseline** : plancher plan = Win 2095 / Docker 2099 ;
     baseline courante = **Win 2100 / Docker 2104** ; après `-1` test → **2099 / 2103** (≥ plancher OK).
   - `cargo nextest run -p nexus-core-rs --locked` : tests DNS verts (non-régression).

**Critère T1 machine** : `cargo deny check advisories` exit 0 (4 ignores retirés)
**ET** `cargo nextest run --workspace --locked` ≥ 2099 Win / 2103 Docker. T2 = N/A.
frontier_closure = **N/A** (prouvé S4).

---

## Verdict: PLAN-ADAPT

Le plan §Phase K (bump 0.24→0.26, réécriture de la construction resolver, retrait des 4 ignores,
clôture des 4 RUSTSEC) est **réalisable** avec l'API 0.26 réelle — aucune décision Day-0 violée
(iroh `=1.0.1` intact, D11 tenu via `-ring` + `webpki-roots` déjà au lock, 0 bump wire, frontier
N/A). Mais **7 corrections concrètes** doivent être câblées, au-delà du texte « churn API 0.25 » :
(1) fichier réel `nexus-core-rs`, pas `nexus-shell-daemon` ; (2) features **`["tls-ring",
"https-ring","webpki-roots"]`** — les noms 0.24 sont supprimés ; (3) **`webpki-roots` obligatoire**
(magasin de racines vide sinon → handshakes runtime KO, invisible aux tests) ; (4) API réécrite
(`Resolver`/builder, `NameServerConfig::new`+`ConnectionConfig`, enum local `DnsTransport`,
`build()` faillible, `trust_negative_responses=false` explicite) ; (5) extraction TXT
`lookup.answers()`+`RData::TXT`+champ `txt.txt_data` (PAS `iter()`/`txt_data()`) ; (6) `-1` test
(garde protocole devenue type-vacante) ; (7) commentaires/docs rafraîchis (Cargo.toml:445 stale,
:446-448 re-validé, HARDENING_ROADMAP, THREAT_MODEL v18). Vérifier au run que le lock résout
hickory-proto **≥ 0.26.1** et rustls-webpki **≥ 0.103.13**, sinon `cargo deny` reste rouge.
