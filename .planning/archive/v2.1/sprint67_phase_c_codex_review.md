Branche vérifiée : `master`. Test exécuté : `cargo test -p sbfb-factory --locked` => `11 passed`.

### Livrable 1 : crate `sbfb-factory` + dépendances
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/Cargo.toml:1`, `crates/sbfb-factory/Cargo.toml:11`
- Evidence :
```toml
1: [package]
2: name = "sbfb-factory"
11: [dependencies]
12: sbfb-manifest = { path = "../sbfb-manifest" }
13: blake3 = { workspace = true }
14: clap = { workspace = true }
```
- Les autres deps requises sont présentes lignes `15-20`, et `tempfile` en dev-dep lignes `22-23`.

### Livrable 2 : CLI clap `Create` / `Validate`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:17`
- Evidence :
```rust
17: #[derive(Subcommand)]
18: enum Command {
20:     Create {
22:         #[arg(long, default_value = "static")]
23:         template: String,
```
- Dispatch confirmé lignes `44-53` vers `template_engine::create()` et `template_engine::validate()`.

### Livrable 3 : `create()` + template static + manifest v2 + lock
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:40`, `:69`, `:90`, `:104`
- Evidence :
```rust
90:     let manifest = SbfbManifest {
91:         schema_version: Some(2),
92:         name: Some(name.to_string()),
100:     manifest.validate()?;
104:     let lock = TemplateLock::generate("static", "1.0.0", &template_files, name, version);
```
- Les fichiers embarqués `include_str!` sont déclarés lignes `40-61`; substitution lignes `63-67`; écriture fichiers lignes `80-87`.

### Livrable 4 : `validate()` projet SBFB
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:111`
- Evidence :
```rust
111: pub fn validate(path: &str) -> Result<(), FactoryError> {
112:     if path.contains("..") {
128:     let manifest_path = dir.join("SBFB.json");
133:         match SbfbManifest::parse(&content) {
143:     for entry in WalkDir::new(dir).follow_links(false) {
```
- Symlink rejeté lignes `145-148`; scan secrets appelé lignes `151-158`.

### Livrable 5 : `TemplateLock`, hash BLAKE3 trié
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_lock.rs:5`
- Evidence :
```rust
5: #[derive(Debug, Serialize)]
6: pub struct TemplateLock {
7:     pub template_id: String,
8:     pub template_version: String,
9:     pub template_hash: String,
```
- `generate()` lignes `14-37`, `to_json()` lignes `39-41`, tri avant hash lignes `44-52`.

### Livrable 6 : `secret_scanner` + 3 regex + findings
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/secret_scanner.rs:8`, `:19`, `:34`
- Evidence :
```rust
19: const PATTERNS: &[Pattern] = &[
21:         name: "AWS access key",
22:         regex: r"AKIA[0-9A-Z]{16}",
25:         name: "GitHub token",
30:         regex: r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
```
- `SecretFinding { file, line, pattern_name }` lignes `8-12`; scan récursif lignes `34-70`.

### Livrable 7 : templates static embarqués
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/templates/static/index.html:6`, `README.md:1`, `sbfb-bridge.js:1`, `gitignore:1`
- Evidence :
```rust
42:         name: "index.html",
43:         content: include_str!("templates/static/index.html"),
47:         name: "sbfb-bridge.js",
48:         content: include_str!("templates/static/sbfb-bridge.js"),
57:         name: ".gitignore",
```
- Les 4 fichiers existent. `index.html` contient `{{name}}` lignes `6` et `10`; `README.md` contient `{{name}}` / `{{version}}` lignes `1-3`.

### Livrable 8 : workspace `Cargo.toml`
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:3`, `Cargo.toml:188`
- Evidence :
```toml
3: members = [
15:     "crates/sbfb-manifest",
16:     "crates/sbfb-factory",
188: walkdir = "2"
```

### Livrable 9 : 8 tests `template_engine.rs`
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:180`, `:193`, `:204`, `:218`, `:229`, `:238`, `:249`, `:255`
- Evidence :
```rust
180:     #[test]
181:     fn test_create_generates_sbfb_json_v2() {
188:         assert_eq!(m.effective_schema_version(), 2);
189:         assert_eq!(m.name.as_deref(), Some("test-app"));
190:         assert!(m.validate().is_ok());
```
- Les 8 tests existent et 7 ont des assertions substantielles directes.
- Gap partiel : `test_symlink_rejected` peut passer sans assertion sur Windows si `symlink_file()` échoue :
```rust
288:         #[cfg(windows)]
290:             if std::os::windows::fs::symlink_file(&target, &link).is_ok() {
291:                 let result = validate(out.to_str().unwrap());
292:                 assert!(result.is_err());
```

### Livrable 10 : 3 tests `secret_scanner.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/secret_scanner.rs:78`, `:92`, `:106`
- Evidence :
```rust
87:         let findings = scan_directory(tmp.path());
88:         assert_eq!(findings.len(), 1);
89:         assert_eq!(findings[0].pattern_name, "AWS access key");
115:         let findings = scan_directory(tmp.path());
116:         assert_eq!(findings.len(), 1);
```
- Les tests GitHub token et PEM vérifient aussi `pattern_name` lignes `117` et `103`.

## Resume final

- Total livrables : 10
- Confirmes : 9
- Gaps : 0
- Partiels : 1