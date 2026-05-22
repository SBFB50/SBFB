Audit fait sur `master` actuel.

### Livrable 1 : `PreviewStore`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon-core/src/preview.rs:17`, `:27`, `:40`, `:58`, `:78`, `:98`
- Evidence :
```rust
17: pub const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);
18: pub const MAX_PREVIEW_BYTES: usize = 10 * 1024 * 1024;
27: pub struct PreviewStore {
28:     inner: Arc<RwLock<HashMap<String, PreviewEntry>>>,
29:     ttl: Duration,
```
`load()` valide 10 MB et hash BLAKE3 (`:40-55`), `get()` vérifie le TTL (`:58-64`), `evict_expired()` existe (`:78-82`). Les 6 tests ont des assertions utiles (`:98-146`).

### Livrable 2 : `lib.rs` expose `preview`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon-core/src/lib.rs:64`
- Evidence :
```rust
64: pub mod paths;
65: pub mod pow_policy_loader;
66: pub mod preview;
67: pub mod publish;
```

### Livrable 3 : HTTP preview + fallback blob-serve
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:182`, `:239`, `:362`, `:432`, `:1116`, `:2015`, `:2301`, `:3069`, `:6310`
- Evidence :
```rust
2020: async fn preview_load(State(state): State<Arc<DaemonHttpState>>, body: Bytes) -> impl IntoResponse {
2021:     debug!(size = body.len(), "POST /api/v1/preview/load");
2022:     match state.preview_store.load(body.to_vec()) {
2023:         Ok(hash) => (StatusCode::OK, Json(PreviewLoadResponse { hash })).into_response(),
```
Code confirmé : champ `preview_store` (`:182-184`), route dans `authed_routes` (`:362`) protégée par `auth_required` (`:432`), fallback preview dans `blob_serve` (`:1131-1143`), CSP via middleware blob-serve (`:239-240`, `:481-496`), constructeurs de test mis à jour (`:2301-2303`, `:3069-3071`).

Manque : l’exigence “4 tests HTTP” est partielle. Il y a 4 tests sous la section preview (`:6310-6392`), mais `test_preview_eviction_after_ttl` (`:6363-6371`) teste directement `PreviewStore`, pas une requête HTTP. Donc 3 tests HTTP effectifs, 1 test store.

### Livrable 4 : wiring runtime + eviction task
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:844`, `:904`
- Evidence :
```rust
904:         {
905:             let store = http_state.preview_store.clone();
906:             tokio::spawn(async move {
907:                 let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
908:                 loop {
```
`PreviewStore::new(DEFAULT_TTL)` est câblé dans `DaemonHttpState` (`:844-846`) et la tâche appelle `store.evict_expired()` (`:909-910`).

### Livrable 5 : `daemon_client.rs`
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/daemon_client.rs:26`, `:66`, `:75`, `:88`, `:106`; comparaison `crates/nexus-shell-daemon-core/src/paths.rs:65`
- Evidence :
```rust
26:     pub fn discover() -> Result<Self, DaemonClientError> {
27:         let running_path = running_json_path()
28:             .ok_or(DaemonClientError::NotFound("cannot resolve running.json path"))?;
29:         let content = std::fs::read_to_string(&running_path).map_err(|_| {
```
Lecture `running.json` + `auth_token` confirmée (`:26-49`, `:75-89`), token exposé pour header HTTP, 1 test utile (`:106-112`).

Manque : la résolution par défaut de `running.json` ne correspond pas au daemon. Factory utilise `HOME/.nexus-grid` (`daemon_client.rs:72`), alors que le core daemon utilise `BaseDirs::data_dir().join("nexus-grid")` (`paths.rs:65-71`) puis `shell-daemon/running.json` (`paths.rs:86-87`). Cela peut casser la découverte sans `NEXUS_GRID_ROOT`.

### Livrable 6 : `preview_cmd.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/preview_cmd.rs:13`, `:23`, `:48`, `:88`
- Evidence :
```rust
13: pub fn run(path: &str) -> Result<(), Box<dyn std::error::Error>> {
14:     let project_dir = Path::new(path).canonicalize()?;
15:     if !project_dir.join("index.html").exists() {
16:         return Err("project directory must contain an index.html".into());
```
POST daemon avec `X-SBFB-Token` confirmé (`:23-30`). `zip_directory()` utilise `walkdir` + `strip_prefix` (`:56-65`) et les 2 tests ont assertions utiles (`:88-110`).

### Livrable 7 : `publish.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/publish.rs:23`, `:41`, `:63`, `:83`
- Evidence :
```rust
63: fn load_and_validate_manifest(
64:     dir: &Path,
65: ) -> Result<sbfb_manifest::SbfbManifest, Box<dyn std::error::Error>> {
66:     let manifest_path = dir.join("SBFB.json");
67:     if !manifest_path.exists() {
```
`SBFB.json` est lu et `name` non vide est vérifié (`:70-75`). `run()` POST `/api/v1/deploy-from-repo` avec `X-SBFB-Token` (`:41-48`). Les 2 tests ont assertions utiles (`:83-120`).

### Livrable 8 : subcommands `Preview` + `Publish`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:6`, `:44`, `:74`
- Evidence :
```rust
44:     /// Load an ephemeral preview into the local daemon
45:     Preview {
46:         /// Path to the project directory
47:         path: String,
```
Modules ajoutés (`:6-9`), subcommands `Preview` et `Publish` présents (`:44-58`), dispatch vers `preview_cmd::run` et `publish::run` (`:74-75`).

### Livrable 9 : dépendances `sbfb-factory`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/Cargo.toml:11`, `:16`, `:22`; workspace `Cargo.toml:187`, `:225`
- Evidence :
```toml
16: reqwest = { workspace = true, features = ["blocking"] }
21: walkdir = { workspace = true }
22: zip = { workspace = true }
```
`zip` et `reqwest` existent bien en workspace (`Cargo.toml:187`, `:225-228`). Recherche `rg "nexus-shell-daemon-core" crates/sbfb-factory` : aucun match, donc pas de dépendance directe à `nexus-shell-daemon-core`.

## Résumé final
- Total livrables : 9
- Confirmés : 7
- Gaps : 0
- Partiels : 2

Tests exécutés : `cargo test -p nexus-shell-daemon-core preview --locked` OK, `cargo test -p nexus-shell-daemon preview --locked` OK, `cargo test -p sbfb-factory --locked` OK.