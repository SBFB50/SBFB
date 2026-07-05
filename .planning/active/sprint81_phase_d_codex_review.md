### Livrable 1 : Doc-stale blobs.rs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/blobs.rs:87`, `crates/nexus-core-rs/src/blobs.rs:488`, vendored `iroh-blobs-0.103.0/src/api/downloader.rs:562`
- Evidence :
```rust
487:        // (`fetch_provider_ordering`, daemon side), and the in-order
488:        // consumption is iroh-blobs 0.103 documented behavior (blanket
489:        // `ContentDiscovery for IntoIterator` yields iteration order);
```
```rust
568:    fn find_providers(&self, _: HashAndFormat) -> n0_future::stream::Boxed<EndpointId> {
569:        let providers = self.clone();
570:        n0_future::stream::iter(providers.into_iter().map(Into::into)).boxed()
571:    }
```
`rg "0\.98|0\.100" crates/nexus-core-rs/src/blobs.rs` retourne `NO_MATCH`. Les seules mentions numériques restantes sont `0.103` aux lignes 87 et 488.

### Livrable 2 : Test pur BlobTicket round-trip
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/blobs.rs:359`, `crates/nexus-shell-daemon-core/src/iroh_runtime.rs:260`, `crates/nexus-shell-daemon-core/src/iroh_runtime.rs:1221`
- Evidence :
```rust
379:        let relay = iroh::RelayUrl::from_str("https://relay.sbfb.invalid./")
381:        let direct: SocketAddr = "192.0.2.7:4433".parse().expect("static socket addr parses");
382:        let addr = iroh::EndpointAddr::new(id)
383:            .with_relay_url(relay)
384:            .with_ip_addr(direct);
```
```rust
390:        let parsed =
391:            BlobTicket::from_str(&ticket_str).expect("a minted ticket string must re-parse");
392:        assert_eq!(
397:        let (got_addr, got_hash, got_format) = parsed.into_parts();
404:        assert_eq!(got_addr, addr,
```
Le test est synchrone `#[test]`, sans node/store/dial. Il vérifie idempotence string, hash, format, et `EndpointAddr` complet. Le contrat daemon est réel : `AnchorLocator.ticket` est `pub ticket: String` (`iroh_runtime.rs:265-267`) et l’ingest persiste `announcement.blob_ticket.clone()` (`iroh_runtime.rs:1221-1223`). Test ciblé lancé : `PASS blobs::tests::blob_ticket_string_round_trips_under_current_lock`.

### Livrable 3 : Périmètre strict négatif
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/blobs.rs:300`, `crates/nexus-core-rs/src/node.rs:375`
- Evidence :
```diff
@@ -358,0 +359,51 @@ mod tests {
+    #[test]
+    fn blob_ticket_string_round_trips_under_current_lock() {
@@ -437 +488 @@ mod tests {
-        // consumption is iroh-blobs 0.100 documented behavior (blanket
+        // consumption is iroh-blobs 0.103 documented behavior (blanket
```
`git diff --name-status` sur `node.rs`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `blobs.rs` ne montre que `M crates/nexus-core-rs/src/blobs.rs`. Le diff de `blobs.rs` ne contient aucun `FsStore::load`, `%APPDATA%`, `redb`, `DOMAIN_`, `_FORMAT_VERSION` ou `FEED_FORMAT_VERSION`. `blobs.rs` ne porte aucune constante wire SBFB.

### Livrable 4 : Artefact preflight
- Statut : PARTIEL
- Fichier(s) : `.planning/active/sprint81_phase_d_preflight.md:1`, `.planning/active/sprint81_phase_d_preflight.md:13`, `.planning/active/sprint81_phase_d_preflight.md:32`, `crates/nexus-core-rs/src/node.rs:67`
- Evidence :
```md
3:> **Verdict : PLAN-ADAPT.**
13:> périmètre code de Phase D se réduit à **DEUX items minces**, plus **UN risque routé F** :
14:> 1. **[DOC-ONLY] blobs.rs:437**
19:> 2. **[TEST-A-AJOUTER] +1 round-trip BlobTicket pur**
24:> 3. **[RISQUE-BLOQUANT → routé Phase F, JAMAIS Phase D] Migration redb DUALE.**
```
Écart partiel : le périmètre annoncé correspond bien au diff livré, mais l’artefact contient une affirmation de grep trop large :
```md
32:> Aucun Day-0 touché ; **0 bump wire SBFB tenu par construction** (blobs.rs/node.rs = 0
33:> `DOMAIN_*`/`_FORMAT_VERSION`, grep vide)
```
Or `crates/nexus-core-rs/src/node.rs` contient déjà des références documentaires existantes à `SEED_FORMAT_VERSION` et `COMPUTE_GROUP_FORMAT_VERSION` (`node.rs:67`, `node.rs:78`). Elles ne sont pas modifiées par cette phase, mais la phrase “grep vide” n’est pas exacte.

### Livrable 5 : Delta tests +1
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/blobs.rs:359`
- Evidence :
```diff
@@ -358,0 +359,51 @@ mod tests {
+    #[test]
+    fn blob_ticket_string_round_trips_under_current_lock() {
```
Le diff ajoute exactement un attribut de test et une fonction de test, sans suppression/renommage de test. Run ciblé exécuté : `cargo nextest run -p nexus-core-rs --locked -E 'test(blob_ticket_string_round_trips)'` => `1 passed`.

## Résumé final

- Total livrables : 5
- Confirmés : 4
- Gaps : 0
- Partiels : 1