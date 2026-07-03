Audit statique uniquement du working tree (`git diff HEAD` + untracked). Aucun test/build lancé.

### Livrable 1 : Bump point unique Cargo.toml
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:24`, `Cargo.toml:34-45`
- Evidence :
```text
24:rust-version = "1.91"
42:iroh = "=1.0.1"
43:iroh-docs = "=0.101.0"
44:iroh-gossip = "=0.101.0"
45:iroh-blobs = "=0.103.0"
```
- `git diff --name-only HEAD -- ':(glob)**/Cargo.toml'` retourne uniquement `Cargo.toml`.

### Livrable 2 : Fix pkarr
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/pkarr_resolver.rs:40-41`, `:114-119`
- Evidence :
```rust
40:use iroh::dns::DnsResolver;
41:use iroh::tls::{CaTlsConfig, default_provider};
114:        let tls_config = CaTlsConfig::default()
115:            .client_config(default_provider())
119:        let client = PkarrRelayClient::new(pkarr_relay_url, tls_config, DnsResolver::new());
```
- Aucun match dans ce fichier pour `custom_server_cert_verifier`, `insecure_skip_verify`, `CaRootsConfig`.

### Livrable 3 : Cargo.lock + cargo tree -d
- Statut : CONFIRME
- Fichier(s) : `Cargo.lock:3918-3920`, `:3969-3970`, `:3988-3990`, `:4053-4055`, `:4093-4095`, `:4133-4134`, `:4160-4161`, `:4206-4221`, `:4234-4235`, `:5476-5477`, `:7004-7014`, `:2100-2117`; `.planning/active/sprint81_phase_b_cargo_tree_d.txt:832-836`
- Evidence :
```text
3918:name = "iroh"
3919:version = "1.0.1"
4053:name = "iroh-docs"
4054:version = "0.101.0"
7004:name = "redb"
7005:version = "3.1.3"
7013:name = "redb"
7014:version = "4.1.0"
```
- `redb 2.6.3` absent de `Cargo.lock` et du `cargo_tree_d`.

### Livrable 4 : Re-datages commentaires
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/docs.rs:53`, `:153-159`, `:390-398`; `crates/nexus-core-rs/src/discovery.rs:4`, `:117-120`; `crates/nexus-core-rs/src/blobs.rs:87-88`; `crates/nexus-shell-daemon/src/runtime.rs:2523-2530`, `:4209-4213`
- Evidence :
```rust
153:    /// In iroh-docs 0.101 the RPC layer never yields `Ok(None)`: an
157:    /// Phase B bump (upstream v0.101.0: `store.rs:24-27` keeps the
158:    /// byte-identical Display, `api.rs:262-265` still hardcodes
390:    /// Opening a doc (`open_doc`/`create_doc`) does NOT enter the
391:    /// sync-set — verified against iroh-docs 0.101
```

### Livrable 5 : Absorption mécanique MSRV-gated
- Statut : PARTIEL
- Fichier(s) : `.planning/active/sprint81_phase_b_collapse_sites.txt:1-9`; `crates/nexus-coordinator-rs/src/pii_redactor.rs:86`; `crates/nexus-worker-core/src/llm/shard.rs:170`; `crates/nexus-shell-daemon/src/http.rs:1397-1401`, `:1614-1619`, `:1951-1962`; `crates/nexus-events-core/src/lib.rs:315-319`
- Evidence :
```rust
1398:            if let Some(sender) = sender_guard.as_ref()
1399:                && let Err(e) = sender.broadcast(envelope).await
1617:            && let Err(e) = blobs.set_tag(&tag, arr).await
1953:            && let Err(e) = crate::feed_sync::emit_seed_announced(
317:        && let Err(e) = writer.write_event(event)
```
- Confirmé : artefact = 139 entrées ; `checksum.is_multiple_of(10)` et `hidden_len.is_multiple_of(n_embd)` présents.
- Gap : l’échantillon sensible viole la contrainte “aucun effet de bord déplacé d’un corps vers une condition” : `broadcast`, `set_tag`, `emit_seed_announced`, `write_event` sont maintenant dans des conditions `let`-chains.

### Livrable 6 : Delta tests 0 net + baseline intacte
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:4209-4216`, `:4251`, `:4281`, `:4320`; `crates/nexus-shell-daemon/src/dispatch_loop.rs:544-633`
- Evidence :
```rust
618:        assert!(
619:            !await_exact_key(
626:            "an incremental write from a reopened-but-never-started doc must NOT reach the \
628:             the upstream sync-set behaviour changed: recalibrate the A4 boot fix"
```
- `dispatch_loop.rs` n’a aucun diff. Aucun ajout/suppression de signature de test détecté dans le diff Rust.

### Livrable 7 : Artefacts planning
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_phase_b_preflight.md:3`, `:122-137`; `.planning/active/sprint81_phase_b_review.md:13`; `.planning/active/sprint81_phase_b_collapse_sites.txt:1`; `.planning/active/sprint81_phase_b_cargo_tree_d.txt:832-836`; `.planning/active/sprint81_kickoff.md:494-497`
- Evidence :
```text
3:> **Verdict : PLAN-ADAPT.**
13:## Verdict: PASS-PENDING
494:- **[D]** Changelog iroh-blobs 0.101→0.103 ...
495:  compile, documenter tout break ; valider l'ouverture redb4 sur COPIE du store dev
```
- Review : un seul header `## Verdict`.

### Livrable 8 : Invariants transverses
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/canonical.rs:77-332`; `crates/nexus-coordinator-rs/src/public_feed.rs:20`; `crates/nexus-core-rs/src/node.rs:68`, `:80`; `crates/nexus-shell-daemon/src/runtime.rs:2536-2564`, `:2631-2659`
- Evidence :
```rust
20:pub const FEED_FORMAT_VERSION: u16 = 1;
68:pub const SEED_ALPN: &[u8] = b"sbfb/seed/0";
80:pub const SHARD_ALPN: &[u8] = b"sbfb/shard/1";
```
- `node.rs`, `canonical.rs`, `web/`, `tools/`, et les deux artefacts T2 ne sont pas dans le diff. Les hunks `public_feed.rs` ne touchent pas `FEED_FORMAT_VERSION`. Les zones sync-set `runtime.rs` ne montrent pas de fix fonctionnel, seulement le commentaire A2 re-daté côté storage.

## Résumé final
- Total livrables : 8
- Confirmés : 7
- Gaps : 0
- Partiels : 1