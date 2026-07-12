Audit statique du working tree uniquement. Je n’ai pas lancé `cargo build/test/nextest`.

### Livrable 1 : `discovery_override.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/discovery_override.rs:71`, `:88`, `:98`, `:131`, `:153`, `:173`, `:199`
- Evidence :
```rust
131:pub fn load_discovery_override() -> Result<Option<DiscoveryPlan>> {
132:    let raw = match env::var(ZERO_N0_ENV) {
133:        Ok(v) => v,
134:        Err(_) => return Ok(None),
```
- Les constantes, `DiscoveryPlan`, fail-loud gate inconnu, pkarr manquant, relais custom manquant, et `validate_zero_n0_pkarr_url` via `enforce_url_policy` sont présents.

### Livrable 2 : Refactor DRY `relay_config.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/relay_config.rs:198`, `:204`, `:219`, `:222`, `:236`
- Evidence :
```rust
198:pub fn validate_relay_url(raw: &str) -> Result<RelayUrl> {
199:    let url = RelayUrl::from_str(raw)
200:        .map_err(|e| NexusError::Endpoint(format!("relay url {raw:?} is not a valid URL: {e}")))?;
204:    enforce_url_policy(&url, raw, "relay url")?;
```
- `git diff` montre que les anciens littéraux `"relay url ..."` sont paramétrés par `what`, et l’appel garde `what = "relay url"`, donc les messages relay restent équivalents.

### Livrable 3 : Câblage `node.rs`
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-core-rs/src/node.rs:327`, `:349`, `:354`, `:375`, `:391`, `:401`, `:499`
- Evidence :
```rust
327:    let zero_n0_plan = crate::discovery_override::load_discovery_override()?;
349:                apply_zero_n0_discovery(Endpoint::builder(presets::Minimal), plan, &memory_lookup);
354:            let builder = Endpoint::builder(presets::N0).address_lookup(memory_lookup.clone());
```
- Confirmé : plan résolu avant bind, branche `Some` sur `presets::Minimal`, `PkarrPublisher` + `PkarrResolver`, `RelayMode::Custom`, log local, pas de gating duress ajouté.
- Écart : le chemin défaut n’est pas strictement identique au diff ancien : `relay_mode` est maintenant appliqué avant `secret_key` (`:375-393`) et le log final ajoute `zero_n0 = ...` (`:401-406`).

### Livrable 4 : Tests Tier A
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/discovery_override.rs:218`, `:220`, `:263`, `:282`, `:296`, `:320`, `:347`, `:386`
- Evidence :
```rust
263:    #[test]
264:    fn returns_none_when_unset_or_disabled() {
265:        let _g = ENV_GUARD.lock().unwrap();
266:        let _snap = EnvSnapshot::capture(ALL_KEYS);
```
- Six tests existent avec assertions utiles : off/unset, gate inconnu, pkarr manquant, relais manquant, plan complet + ON case-insensitive, parité policy URL.

### Livrable 5 : Test Tier B E2E hermétique
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/node.rs:703`, `:769`, `:805`, `:833`, `:852`, `:891`, `:908`; `crates/nexus-core-rs/Cargo.toml:136`, `:156`
- Evidence :
```rust
852:        let build = |plan: &DiscoveryPlan| {
853:            apply_zero_n0_discovery(
854:                Endpoint::builder(presets::Minimal),
856:                &MemoryLookup::new(),
```
- Confirmé : même fonction que prod, fake pkarr PUT+GET, `run_relay_server`, `iroh` `test-utils` seulement en dev-dependency, MemoryLookup frais, connexion par endpoint ID seul avec assertions/deadlines.

### Livrable 6 : Exports `lib.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/lib.rs:48`, `:113`
- Evidence :
```rust
48:pub mod discovery_override;
113:pub use discovery_override::{
114:    DiscoveryPlan, ZERO_N0_ENV, ZERO_N0_PKARR_RELAYS_ENV, load_discovery_override,
115:    validate_zero_n0_pkarr_url,
```

### Livrable 7 : Runbooks ops
- Statut : PARTIEL
- Fichier(s) : `docs/release/IROH_SELFHOST_OPS.md:90`, `:111`, `:139`, `:166`, `:189`, `:221`, `:251`; `docs/release/PKARR_RELAY_OPS.md:17`
- Evidence :
```bash
90:```bash
92:cargo install iroh-relay --version 1.0.1 --features server
97:cargo install iroh-dns-server --version 1.0.1
```
- Confirmé : nouveau runbook, TOML relais avec `metrics_bind_addr` loopback, TOML `iroh-dns-server`, env client, smoke test, résumé threat, blockquote de portée dans `PKARR_RELAY_OPS.md`.
- Écart : les units systemd ne sont pas fournies comme templates complets ; la section `:166-181` décrit contraintes et `ExecStart`, mais pas de blocs `[Unit]/[Service]/[Install]`.

### Livrable 8 : Artefact T2
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_t2_e2_zero_n0.json:7`, `:9`, `:14`, `:16`
- Evidence :
```json
9:    "selfhost_binaries_available": {
10:      "verdict": "PASS",
14:    "zero_n0_live_convergence": {
15:      "verdict": "RIG-ABSENT",
```
- Confirmé : PASS selfhost binaries, RIG-ABSENT live convergence avec diagnostic host/IP dédié, dates C8, et scoping note complémentaire à S75.

### Invariants transversaux
- 0 bump wire : CONFIRME. `SEED_ALPN` et `SHARD_ALPN` sont verbatim dans `node.rs:70` et `:82`; `git diff` ne touche pas `canonical.rs`, `seed.rs`, `shard_plan.rs`, `compute_group.rs`.
- Pins iroh : CONFIRME. `Cargo.toml:42-45` garde `=1.0.1 / =0.101.0 / =0.103.0`.
- `Cargo.lock` : CONFIRME avec nuance. Le diff ajoute des deps/packages liés à `test-utils`; aucune version existante n’est bumpée. Il y a seulement désambiguïsation textuelle `sha1` -> `sha1 0.10.6`.

### Résumé final
- Total livrables : 8
- Confirmés : 6
- Gaps : 0
- Partiels : 2