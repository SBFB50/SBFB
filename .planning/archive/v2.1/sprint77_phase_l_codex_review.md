Audit effectué sur `master`, working tree courant.

Tests ciblés exécutés :
- `cargo test -p nexus-core-rs schemas::shard::tests --locked` : 5 passed
- `cargo test -p nexus-shell-daemon shard_session --locked` : 2 passed

### Livrable 1 : `JsonSchema` sur 8 payloads wire
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/shard_plan.rs:63`, `:116`, `:129`, `:143`, `:188`, `:234`, `:379`, `:408`; `crates/nexus-core-rs/src/compute_group.rs:53`, `:87`
- Evidence :
```rust
63: use schemars::JsonSchema;
116: #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
143: #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
379: #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema)]
408: #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
```
- Les entrées signées ne dérivent pas `JsonSchema` : `ShardedSessionManifestEntry` à `shard_plan.rs:306`, `RunProofEntry` à `shard_plan.rs:487`, `ComputeGroupEntry` à `compute_group.rs:160`.

### Livrable 2 : `schemas/shard.rs`, DTO, fonctions, tests
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/schemas/shard.rs:72`, `:87`, `:102-142`, `:183`, `:197`, `:282`, `:314`, `:351`
- Evidence :
```rust
72: #[derive(Debug, Clone, Serialize, JsonSchema)]
73: pub struct ShardSessionView {
87: #[derive(Debug, Clone, Serialize, JsonSchema)]
88: pub struct ShardSessionStatusResponse {
102: pub fn compute_group_schema() -> serde_json::Value {
```
- Les 8 fonctions `schema_for!` sont présentes lignes `102-142`.
- Les 5 tests ont des assertions utiles : objet + `$schema` lignes `183-188`, required inclus/exclus lignes `210-274`, whitelist exacte lignes `290-304`, snapshot drift lignes `314-341`, spec consts lignes `351-412`.

### Livrable 3 : 8 JSON schemas générés
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/schemas/*.schema.json`
- Evidence :
```text
compute_group.schema.json:2 "$schema": "https://json-schema.org/draft/2020-12/schema"
shard_assignment.schema.json:24 "$schema": "https://json-schema.org/draft/2020-12/schema"
shard_plan.schema.json:114 "$schema": "https://json-schema.org/draft/2020-12/schema"
shard_session_view.schema.json:20 "title": "ShardSessionView"
```
- Les 8 fichiers sont net-new/untracked : `compute_group`, `run_metrics`, `run_proof`, `shard_assignment`, `shard_plan`, `shard_session_status_response`, `shard_session_view`, `sharded_session_manifest`.

### Livrable 4 : `pub mod shard` + re-exports
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/schemas/mod.rs:42-50`, `crates/nexus-core-rs/src/lib.rs:173-178`
- Evidence :
```rust
42: pub mod shard;
45: pub use shard::{
46:     ShardSessionStatusResponse, ShardSessionView, compute_group_schema, run_metrics_schema,
47:     run_proof_schema, shard_assignment_schema, shard_plan_schema,
```

### Livrable 5 : DTO supprimés de `http.rs`, importés depuis core
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:2094`, `:2100-2134`, `:2147-2149`, `:5208-5291`
- Evidence :
```rust
2094: use nexus_core_rs::{ShardSessionStatusResponse, ShardSessionView};
2100: fn project_shard_session(manifest: &nexus_core_rs::ShardedSessionManifest) -> ShardSessionView {
2124: fn shard_session_response(session_id: &str) -> ShardSessionStatusResponse {
2147: async fn shard_session(Path(session_id): Path<String>) -> impl IntoResponse {
```
- Recherche `struct ShardSessionView|ShardSessionStatusResponse` : plus de définition locale dans `http.rs`, seulement dans `schemas/shard.rs`.
- Réponse inchangée couverte par tests : `{found, session}` exactement lignes `5217-5228`, vue `{session_id, member_count}` exactement lignes `5269-5291`.

### Livrable 6 : `docs/protocol/SHARD_PROTOCOL_SPEC.md`
- Statut : CONFIRME
- Fichier(s) : `docs/protocol/SHARD_PROTOCOL_SPEC.md:3-12`, `:55-61`, `:65-84`, `:172-188`, `:210-225`
- Evidence :
```md
3: **Status:** Sprint 77 ... **feature PROVISIONAL**
4: PROVISIONAL** (live cross-machine benchmark `RIG-ABSENT`;
5: session orchestrator + benchmark are a carry to **Sprint 78**).
57: | Compute group allowlist | `nexus-compute-group-v1` | initiator |
```
- Tags vérifiés contre `canonical.rs:258`, `:276`, `:290`, `:310`, `:332`.
- ALPN vérifié contre `node.rs:80`.
- Caps exacts contre `shard.rs:85`, `:97`, `shard_plan.rs:88`, `:92`, `:97`, `:103`, `:108`, `compute_group.rs:76`, `:80`.
- Pas de sur-promesse détectée : status provisional, benchmark `RIG-ABSENT`, carry Sprint 78 explicités.

### Invariants
- 0-bump : CONFIRME. `SHARD_PLAN_FORMAT_VERSION = 1` à `shard_plan.rs:77`, `RUN_PROOF_FORMAT_VERSION = 1` à `:81`, `COMPUTE_GROUP_FORMAT_VERSION = 1` à `compute_group.rs:67`.
- 0-dep : CONFIRME. Aucun `Cargo.toml` modifié ; `schemars = { version = "1.2", features = ["derive"] }` existe déjà à `Cargo.toml:350`, workspace utilisé à `crates/nexus-core-rs/Cargo.toml:120`.
- no-float : CONFIRME. `RunMetrics` reste `u64/u32` à `shard_plan.rs:382-399`; recherche `f32/f64` ne trouve que des commentaires/docs.
- derive-additif : CONFIRME. Le diff des types ajoute seulement `use schemars::JsonSchema` et `JsonSchema` dans les derives ; aucun changement `#[serde(...)]`.
- whitelist SI-3/SI-4 : CONFIRME. DTO `ShardSessionView` expose seulement `session_id` + `member_count` à `schemas/shard.rs:73-78`; test d’égalité exacte à `:290-304`; snapshot properties à `shard_session_view.schema.json:5`, `:11`, required à `:17-18`.
- Entry-non-derives : CONFIRME. Les trois entrées signées dérivent seulement `Serialize/Deserialize/...`, sans `JsonSchema` : `compute_group.rs:160`, `shard_plan.rs:306`, `shard_plan.rs:487`.

## Resume final
- Total livrables : 6
- Confirmes : 6
- Gaps : 0
- Partiels : 0