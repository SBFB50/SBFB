Audit effectué sur l’état courant du repo. Aucun fichier modifié par moi.

### Livrable 1 : helper `git_cmd_in(root, args)`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:725`
- Evidence :
```rust
725:fn git_cmd_in(root: &Path, args: &[&str]) -> String {
726:    std::process::Command::new("git")
727:        .current_dir(root)
728:        .args(args)
```
Le helper utilise bien `current_dir(root)` et pas `git -C <path>`.

### Livrable 2 : enveloppe `WorkingTreeDiff`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:964`, `crates/sbfb-factory/src/sprint_history.rs:1004`
- Evidence :
```rust
1004:pub struct WorkingTreeDiff {
1005:    pub head: String,
1006:    pub unstaged: Vec<FileDiff>,
1007:    pub staged: Vec<FileDiff>,
1008:    pub truncated: bool,
```
`FileDiff` reste inchangé et ne porte pas `truncated` :
```rust
964:#[derive(Serialize)]
965:pub struct FileDiff {
966:    pub path: String,
967:    pub insertions: u32,
968:    pub deletions: u32,
```

### Livrable 3 : `working_tree_diff_data(root)` et réutilisation du parser
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:1011`, `crates/sbfb-factory/src/sprint_history.rs:1068`
- Evidence :
```rust
1011:pub fn working_tree_diff_data(root: &Path) -> WorkingTreeDiff {
1012:    let head = git_cmd_in(root, &["rev-parse", "--short", "HEAD"]);
1013:    let (unstaged_raw, t_unstaged) = bounded_working_tree_diff(root, false);
1014:    let (staged_raw, t_staged) = bounded_working_tree_diff(root, true);
```
```rust
1017:        unstaged: parse_unified_diff(&unstaged_raw),
1018:        staged: parse_unified_diff(&staged_raw),
```
`parse_unified_diff` reste dans `sprint_history.rs`, privé (`fn`, pas `pub`) à `:1068`. Le `git diff --unified=0` courant ne montre aucun hunk modifiant le corps du parser.

### Livrable 4 : déterminisme Git / absence d’input utilisateur
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:1028`
- Evidence :
```rust
1029:    let mut args: Vec<&str> = vec![
1030:        "-c",
1031:        "color.ui=false",
1032:        "diff",
1033:        "--no-color",
1034:        "-U3",
1035:        "--no-ext-diff",
```
`--cached` est ajouté uniquement depuis le booléen interne `cached` (`:1037-1039`). Le chemin Phase F ne reçoit aucun rev/pathspec/user input ; `root` passe par `current_dir`.

### Livrable 5 : borne `MAX_DIFF_LINES` et troncature enveloppe
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:991`, `crates/sbfb-factory/src/sprint_history.rs:1041`
- Evidence :
```rust
991:const MAX_DIFF_LINES: usize = 20_000;
```
```rust
1041:    let mut lines: Vec<&str> = raw.lines().collect();
1042:    let truncated = lines.len() > MAX_DIFF_LINES;
1043:    if truncated {
1044:        lines.truncate(MAX_DIFF_LINES);
```
Le flag est bien au niveau `WorkingTreeDiff.truncated` (`:1008`, `:1019`), pas dans `FileDiff`.

### Livrable 6 : route `GET /api/git/diff` sous `authed` + handler
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:170`, `:197`, `:208`, `:1316`
- Evidence :
```rust
166:    // Authenticated surface: every `/api/*` route + the asset
170:    let authed = Router::new()
```
```rust
197:        .route("/api/git/diff", get(handle_git_diff))
208:        .layer(axum::middleware::from_fn_with_state(
210:            auth::auth_required,
```
```rust
1316:async fn handle_git_diff(State(state): State<OperatorState>) -> Json<serde_json::Value> {
1317:    let diff = crate::sprint_history::working_tree_diff_data(&state.root);
1318:    Json(serde_json::json!(diff))
```
Test 401 sans token confirmé dans `crates/sbfb-factory/tests/operator_server.rs:1220-1227`.

### Livrable 7 : tests hermétiques + HTTP
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/sprint_history.rs:1216`, `:1244`, `:1262`, `:1304`; `crates/sbfb-factory/tests/operator_server.rs:1206`, `:1220`
- Evidence :
```rust
1244:        let diff = working_tree_diff_data(root);
1245:        assert!(
1246:            !diff.head.is_empty(),
```
```rust
1262:        assert!(a_kinds.contains(&"add"), "add line classified");
1263:        assert!(a_kinds.contains(&"del"), "del line classified");
1264:        assert!(a_kinds.contains(&"ctx"), "context line classified");
```
```rust
1304:        let diff = working_tree_diff_data(root);
1305:        assert!(
1306:            diff.truncated,
```
Tests exécutés et passés :
`cargo test -p sbfb-factory working_tree_diff --locked` : 2 passed.  
`cargo test -p sbfb-factory --test operator_server operator_git_diff --locked` : 2 passed.

### Invariants transverses
- CONFIRME : aucune occurrence Phase F dans `crates/nexus-shell-daemon*` (`rg /api/git/diff|WorkingTreeDiff|working_tree_diff_data` sans résultat).
- CONFIRME : aucun `use nexus_shell_daemon*` ajouté dans `sbfb-factory`.
- CONFIRME : `git diff --name-only -- Cargo.toml crates/*/Cargo.toml` vide, donc aucune dépendance ajoutée.
- CONFIRME : les fichiers modifiés sont uniquement `operator_server.rs`, `sprint_history.rs`, `tests/operator_server.rs`; aucun fichier daemon/core/wire/schema versionné touché.

## Resume final

- Total livrables : 7
- Confirmes : 7
- Gaps : 0
- Partiels : 0