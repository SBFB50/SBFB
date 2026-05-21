Audit effectué sur `master`. Suite ciblée exécutée : `cargo test -p sbfb-factory --locked --all-targets` => 31 tests passés.

### Livrable 1 : gates.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:10`, `:35`, `:46`, `:65`, `:118`, `:128`, `:169`
- Evidence :
```rust
46: pub fn run_gate_fg4_diff(workspace: &Path) -> Result<GateResult, FactoryError> {
47:     let entries = diff::diff_workspace(workspace)?;
48:     let mut lines = Vec::new();
49:     for entry in &entries {
```
GateResult, Display, FG4-FG7 et `check_path_containment` sont présents. FG5 utilise `dunce::canonicalize` + `WalkDir`; FG6 appelle `secret_scanner` et compare `template_hash`.

### Livrable 2 : diff.rs
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/diff.rs:10`, `:17`, `:22`, `:28`, `:43`, `:60`
- Evidence :
```rust
60:         if rel.starts_with('.') || METADATA_FILES.contains(&rel.as_str()) {
61:             continue;
62:         }
63:
64:         if let Some((_, expected_content)) = expected.iter().find(|(n, _)| n == &rel) {
```
Diff enum, struct, lockfile read, `expected_files()` et comparaison Added/Modified/Deleted sont implémentés. Gap partiel : le skip des fichiers cachés ne couvre que les chemins relatifs qui commencent par `.`, donc un fichier caché imbriqué comme `src/.env` ne serait pas ignoré.

### Livrable 3 : template_engine.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:119`, `:141`, `:169`, `:180`
- Evidence :
```rust
141: pub fn validate(path: &str) -> Result<(), FactoryError> {
142:     let canonical = dunce::canonicalize(path)
143:         .map_err(|e| FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", path)))?;
144:
```
`expected_files()` est public lignes 119-139. `validate()` utilise le chemin canonique pour `WalkDir` lignes 169-170 et secret scan lignes 180-182. Aucune occurrence restante de `path.contains("..")` trouvée dans ce fichier.

### Livrable 4 : main.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:7`, `:8`, `:62`, `:68`, `:100`, `:107`
- Evidence :
```rust
100: fn run_diff(path: &str) -> Result<(), Box<dyn std::error::Error>> {
101:     let workspace = dunce::canonicalize(path)?;
102:     let result = gates::run_gate_fg4_diff(&workspace)?;
103:     eprintln!("{result}");
```
`mod diff` et `mod gates` sont déclarés. `Diff` et `ScanSecrets` existent et passent par `dunce::canonicalize`; ScanSecrets appelle `run_gate_fg6_secrets` lignes 107-109.

### Livrable 5 : Cargo.toml
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:187`, `crates/sbfb-factory/Cargo.toml:23`
- Evidence :
```toml
187: zip = { version = "8.5", default-features = false, features = ["deflate"] }
188: walkdir = "2"
189: dunce = "1.0"
```
La crate déclare bien `dunce = { workspace = true }` à `crates/sbfb-factory/Cargo.toml:23`.

### Livrable 6 : preview_cmd.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/preview_cmd.rs:13`
- Evidence :
```rust
13: pub fn run(path: &str) -> Result<(), Box<dyn std::error::Error>> {
14:     let project_dir = dunce::canonicalize(path)?;
15:     if !project_dir.join("index.html").exists() {
16:         return Err("project directory must contain an index.html".into());
```

### Livrable 7 : publish.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/publish.rs:23`
- Evidence :
```rust
23: pub fn run(path: &str, repo_url: &str) -> Result<(), Box<dyn std::error::Error>> {
24:     let project_dir = dunce::canonicalize(path)?;
25:
26:     let manifest = load_and_validate_manifest(&project_dir)?;
```

### Livrable 8 : 7 tests gates.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:199`, `:212`, `:234`, `:266`, `:281`, `:294`, `:316`
- Evidence :
```rust
304:         let result = run_gate_fg6_secrets(&workspace).unwrap();
305:         assert!(!result.passed, "tampered provenance should fail FG6");
306:         assert!(
307:             result
```
Les 7 tests demandés existent et contiennent des assertions substantielles. Note : le test symlink a une branche Windows conditionnelle si la création de symlink échoue.

### Livrable 9 : 3 tests diff.rs
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/diff.rs:109`, `:127`, `:145`
- Evidence :
```rust
151:         let diff = diff_workspace(&workspace).unwrap();
152:         let deleted: Vec<_> = diff
153:             .iter()
154:             .filter(|e| e.status == DiffStatus::Deleted)
```
Les 3 tests créent un projet factory, modifient le workspace, puis vérifient Added/Modified/Deleted avec assertions utiles.

## Resume final
- Total livrables : 9
- Confirmes : 8
- Gaps : 0
- Partiels : 1