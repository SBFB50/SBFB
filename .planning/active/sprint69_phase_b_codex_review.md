### Livrable 1 : FG8 provenance Ed25519
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:184`, `crates/sbfb-factory/src/gates.rs:201`, `crates/sbfb-factory/src/gates.rs:240`, `crates/sbfb-factory/src/gates.rs:422`
- Evidence :
```rust
201:     let mut result = Vec::with_capacity(DOMAIN_PROVENANCE_V1.len() + 1 + json_bytes.len());
202:     result.extend_from_slice(DOMAIN_PROVENANCE_V1);
203:     result.push(0x00);
204:     result.extend_from_slice(json_bytes.as_bytes());
240:     match nexus_core_rs::crypto::verify(node_public_key, &canonical, &sig) {
```
Les clés sont reconstruites dans l’ordre alphabétique aux lignes `192-199`, identique à `crates/nexus-coordinator-rs/src/provenance.rs:110-123`. Les 3 tests utiles existent avec assertions : valid signature `422-432`, wrong key `435-447`, tampered JSON `450-461`. Recherche source : `0` occurrence de `#[allow(dead_code)]` dans `crates/sbfb-factory/**/*.rs`.

### Livrable 2 : pipeline FG9
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/pipeline.rs:9`, `crates/sbfb-factory/src/pipeline.rs:15`, `crates/sbfb-factory/src/pipeline.rs:23`, `crates/sbfb-factory/src/pipeline.rs:48`, `crates/sbfb-factory/src/pipeline.rs:56`
- Evidence :
```rust
23:     let fg4 = gates::run_gate_fg4_diff(workspace)?;
27:     if !skip_gates {
28:         let fg5 = gates::run_gate_fg5_sandbox(workspace)?;
37:         let fg6 = gates::run_gate_fg6_secrets(workspace)?;
48:     let (hash, provenance_hash) = post_deploy_from_repo(workspace, repo_url)?;
56:     let fg8 = gates::run_gate_fg8_provenance(&provenance_json, &node_public_key)?;
```
Confirmé : module nouveau, `PipelineResult`, POST deploy, FG5/FG6 bloquants, FG8 post-publish bloquant et hors `skip_gates`.
Gaps : `--skip-gates` ne saute pas FG4 alors que le plan/CLI parlent des pre-publish gates FG4/FG5/FG6 (`pipeline.rs:23-27`, `main.rs:63-65`). Le test `test_pipeline_aborts_on_path_traversal` peut sortir sans assertion si le symlink Windows n’est pas créé (`pipeline.rs:168-172`), donc ce test n’est pas toujours probant.

### Livrable 3 : publish refactor
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/publish.rs:10`, `crates/sbfb-factory/src/publish.rs:14`, `crates/sbfb-factory/src/publish.rs:43`, `crates/sbfb-factory/src/publish.rs:59`
- Evidence :
```rust
10: pub fn run(path: &str, repo_url: &str, skip_gates: bool) -> Result<(), Box<dyn std::error::Error>> {
11:     let project_dir = dunce::canonicalize(path)?;
12:     validate_manifest(&project_dir)?;
14:     let result = pipeline::run_publish_pipeline(&project_dir, repo_url, skip_gates)?;
```
Les tests sont adaptés : `publish_requires_running_json` crée un vrai projet via `template_engine::create` (`43-49`) et `publish_pre_validates_manifest` garde la prévalidation du manifest (`59-77`).

### Livrable 4 : daemon_client
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/daemon_client.rs:59`, `crates/sbfb-factory/src/daemon_client.rs:87`
- Evidence :
```rust
59:     pub fn get_node_id(&self) -> Result<[u8; 32], DaemonClientError> {
60:         let url = format!("{}/api/daemon/info", self.base_url);
64:             .header("X-SBFB-Token", &self.token)
65:             .header("Host", "127.0.0.1")
80:         let bytes = hex::decode(node_id_hex)
```
`get_provenance(project_id)` appelle bien `/api/v1/project/{project_id}/provenance`, passe `X-SBFB-Token` et `Host`, puis sérialise `record` en JSON (`daemon_client.rs:87-112`).

### Livrable 5 : main CLI wiring
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:10`, `crates/sbfb-factory/src/main.rs:55`, `crates/sbfb-factory/src/main.rs:80`, `crates/sbfb-factory/src/main.rs:188`
- Evidence :
```rust
10: mod pipeline;
55:     Publish {
63:         /// Skip pre-publish gates (FG4/FG5/FG6) for debugging
65:         skip_gates: bool,
80:     /// Run FG5 sandbox check (symlinks, path traversal)
```
`Sandbox` appelle `run_gate_fg5_sandbox` puis `check_path_containment` (`main.rs:188-202`). `PreviewCheck` appelle `run_gate_fg7_preview` (`main.rs:205-212`).

### Livrable 6 : Cargo deps PLAN-ADAPT
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/Cargo.toml:11`
- Evidence :
```toml
11: [dependencies]
12: sbfb-manifest = { path = "../sbfb-manifest" }
13: nexus-core-rs = { path = "../nexus-core-rs" }
17: hex = { workspace = true }
```
Aucune dépendance `nexus-coordinator-rs` dans `crates/sbfb-factory/Cargo.toml`.

### Resume final
- Total livrables : 6
- Confirmes : 5
- Gaps : 0
- Partiels : 1

Note vérification : je n’ai pas pu exécuter `cargo test -p sbfb-factory --locked` à cause d’un refus d’accès sur `target/debug/.cargo-lock`. L’audit ci-dessus est donc basé sur lecture source avec lignes exactes.