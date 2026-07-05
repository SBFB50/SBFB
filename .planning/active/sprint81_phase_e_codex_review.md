### Livrable 1 : tripwire pkarr hermétique
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/pkarr_resolver.rs:55`, `:211-240`; vendor `iroh-1.0.1/src/address_lookup/pkarr.rs:127`, `address_lookup.rs:128`
- Evidence :
```rust
55: pub const DEFAULT_PKARR_RELAY_URL: &str = "https://dns.iroh.link/pkarr";
211: #[test]
212: fn default_pkarr_url_matches_iroh_upstream_const() {
232:     assert_eq!(
233:         DEFAULT_PKARR_RELAY_URL,
234:         iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD,
```
```rust
237: let parsed: Url = DEFAULT_PKARR_RELAY_URL
238:     .parse()
239:     .expect("default pkarr relay URL must stay a valid URL");
240: assert_eq!(parsed.scheme(), "https", "pkarr relay must stay HTTPS");
```
- Indépendance confirmée : local = littéral ligne 55 ; upstream = littéral vendored `N0_DNS_PKARR_RELAY_PROD` ligne 127, seulement ré-exporté par `pub use pkarr::*` ligne 128.
- Test ciblé exécuté : `cargo nextest run -p nexus-core-rs --locked -E 'test(default_pkarr_url_matches)'` => 1 test passé.

### Livrable 2 : lot doc-stale transport
- Statut : CONFIRME
- Fichier(s) : `gossip.rs:258-260`, `:739-747`; `tls_pinning.rs:32-50`, `:55-63`; `transport_probe.rs:24-33`; `relay_config.rs:5-26`; `node.rs:324-330`; `docs/rust/PATTERNS.md:974-998`
- Evidence :
```rust
258: /// Owns a cheaply-cloned `Gossip` (internal `Arc<Inner>`, verified
259: /// unchanged in iroh-gossip 0.101 `net.rs:84-86` at the S81 Phase E
```
Vendor vérifié : `iroh-gossip-0.101.0/src/net.rs:85-86` contient bien `pub struct Gossip { pub(crate) inner: Arc<Inner>, }`.

```rust
741: // address injection (`MemoryLookup` under iroh 1.0.1) against an
746: // coverage at the transport layer lives in `shard.rs` (this
747: // crate) and `seed_protocol.rs` (nexus-shell-daemon) instead.
```
`shard.rs` est bien dans `crates/nexus-core-rs/src`, même crate que `gossip.rs`; `seed_protocol.rs` est dans `crates/nexus-shell-daemon/src`.

```rust
40: //! re-cert (iroh 1.0.1): the upstream blocker is GONE
41: //! `iroh_relay::tls::CaTlsConfig::custom_server_cert_verifier`
44: //! through `iroh::endpoint::Builder::ca_tls_config`
49: //! the live relay path remains WebPKI-only
```
Ancres vendor vérifiées : `iroh-relay-1.0.1/src/tls.rs:141` et `iroh-1.0.1/src/endpoint.rs:713`. La table fail-open/fail-closed reste présente `tls_pinning.rs:55-63`, et T2-T5 restent présents `:11-13`.

```rust
26: //! and this is still true under iroh-relay 1.0.1
28: //! `tokio_websockets` in `client/conn.rs`; the separate
29: //! `DEFAULT_RELAY_QUIC_PORT` 7842 serves QUIC *address
30: //! discovery*, not a relay data path).
```
Vendor vérifié : `iroh-relay-1.0.1/src/client/conn.rs:73-86` utilise `tokio_websockets`; `defaults.rs:3-7` documente `7842` pour QUIC address discovery.

```rust
5: //! iroh `presets::N0` wires the four n0-run relays (NA east, NA
18: //! 3. Fallback to `iroh::defaults::prod::default_relay_map()`
20: //!    (`use1-1`/`usw1-1`/`euc1-1`/`aps1-1` `.relay.n0.iroh.link`,
23: //!    they DID change at the 1.0 bump (the `iroh-canary` label was
```
Vendor vérifié : 1.0.1 a quatre hostnames `.relay.n0.iroh.link` aux lignes 27-33 ; 0.98.2 avait les mêmes quatre avec `iroh-canary` aux lignes 27-33.

### Livrable 3 : artefact T2 live
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_t2_e_discovery_survival.json:1-27`
- Evidence :
```json
10: "pkarr_relay_prod": {
11:   "verdict": "PASS",
12:   "criterion": "https://dns.iroh.link/pkarr ...",
13:   "observed": "GET /pkarr -> 404 ... GET /pkarr/<malformed-z32-key> -> 400 ..."
```
```json
15: "relay_fleet_prod_1_0": {
16:   "verdict": "PASS",
17:   "criterion": "the four n0 relay hostnames shipped by vendored iroh-1.0.1 defaults.rs ...",
18:   "observed": "use1-1... -> 200 ... usw1-1 -> 200 ... euc1-1 -> 200 ... aps1-1 -> 200 ..."
```
- Tripwire cité par nom exact ligne 23.
- Note “7 pre-existing tests” présente ligne 24.
- Residual risk honnête présent ligne 26 : point-in-time, warn-only, Phase G, split E’ et gates calendaires.
- JSON valide via `ConvertFrom-Json`.
- Scan anti-secret simple : aucun token/secret détecté ; seules IPs publiques n0 aux lignes 13 et 18.

### Livrable 4 : périmètre strict négatif
- Statut : CONFIRME
- Fichier(s) : diff git ; `node.rs:324-352`; `shard.rs:515-537`; `seed_protocol.rs:459-460`; `pkarr_resolver.rs:211-240`
- Evidence :
```text
git diff --name-only:
crates/nexus-core-rs/src/gossip.rs
crates/nexus-core-rs/src/node.rs
crates/nexus-core-rs/src/pkarr_resolver.rs
crates/nexus-core-rs/src/relay_config.rs
crates/nexus-core-rs/src/tls_pinning.rs
crates/nexus-shell-daemon-core/src/transport_probe.rs
docs/rust/PATTERNS.md
```
- Aucun `Cargo.toml`, `Cargo.lock`, `deny.toml`, `http.rs`, `age_witness.rs`, `shard.rs`, ni `seed_protocol.rs` modifié.
- Diff `node.rs` : commentaire seul ; logique `load_relay_map` / `RelayMode::Custom` intacte `node.rs:331-352`.
- Recherche dans le diff : 0 `clear_address_lookup`, 0 `PkarrPublisher::builder`, 0 `PkarrResolver::builder`, 0 nouveau `RelayMode`, 0 `FsStore::load`, 0 `redb`, 0 bump `DOMAIN_*` / `*_FORMAT_VERSION` / ALPN.
- Tests handshake existants non réimplémentés : `shard_handshake_admits_member` `shard.rs:515`, `shard_handshake_rejects_non_member` `:537`, seed tokio tests `seed_protocol.rs:459+`.

### Livrable 5 : artefacts process
- Statut : PARTIEL
- Fichier(s) : `.planning/active/sprint81_phase_e_preflight.md:3-26`, `:294-305`, `:311-326`; `.planning/active/sprint81_phase_e_review.md:23-50`, `:441-447`, `:463-478`
- Evidence confirmée :
```md
3: > **Verdict : PLAN-ADAPT.**
23: > 2. **[TEST-A-AJOUTER, +1 net]** ...
24: >    `DEFAULT_PKARR_RELAY_URL == iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD` + parse `Url`
```
```md
465: - **E-DOC-1 (P1) CORRIGÉ** — `gossip.rs` : « `shard.rs` (this crate) and
470: - **M1 (P3) CORRIGÉ + TRANCHÉ PAR GIT** — la vérité historique est
445:   **D4-3 (P3)** frère « byte-for-byte » stale MANQUÉ par le lot :
446:   `node.rs:324-328` — **CORRIGÉ**
```
- Sondage working tree confirmé : `gossip.rs:746-747` dit bien `shard.rs` `(this crate)` ; `tls_pinning.rs:33` dit `iroh 0.97 per the S19 Phase C lockfile`; `node.rs:327-330` retire le claim byte-for-byte.
- Point partiel : le preflight en tête reste formulé avec “5 doc-comments” (`sprint81_phase_e_preflight.md:27-30`) et ne reflète pas exactement le diff final post-review, qui inclut aussi `node.rs` D4-3 et `docs/rust/PATTERNS.md` status-update. La review réconcilie bien ces corrections, mais le preflight seul n’est pas “exactement le diff livré”.

### Livrable 6 : delta tests +1
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/pkarr_resolver.rs:211-240`
- Evidence :
```text
git diff --unified=0 -- . | rg '#[(tokio::)?test]|fn ...':
+    #[test]
+    fn default_pkarr_url_matches_iroh_upstream_const() {
```
- Aucun `#[test]` / `#[tokio::test]` supprimé dans le diff.
- Run ciblé : `PASS nexus-core-rs pkarr_resolver::tests::default_pkarr_url_matches_iroh_upstream_const`.

## Résumé final
- Total livrables : 6
- Confirmés : 5
- Gaps : 0
- Partiels : 1