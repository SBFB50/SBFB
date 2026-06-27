Audit fait sur branche `master`, à partir de `git diff` du working tree non commité.

### Livrable 1 : route authentifiée `/api/gates`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:166`, `:170`, `:198`, `:209`
- Evidence :
```rust
166:     // Authenticated surface: every `/api/*` route + the asset
170:     let authed = Router::new()
197:         .route("/api/git/diff", get(handle_git_diff))
198:         .route("/api/gates", get(handle_gates))
209:         .layer(axum::middleware::from_fn_with_state(
211:             auth::auth_required,
```
Recherche transverse : `/api/gates` n’apparaît pas dans `crates/nexus-shell-daemon`; seule occurrence route côté `sbfb-factory`.

### Livrable 2 : handler `handle_gates`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:1328`
- Evidence :
```rust
1328: /// Sprint 80 Phase G: the live registry of Factory gates as a 1:1
1332: /// "PASS"). Read-only, 0 user input, reads `state.root`; runs no publish
1334: async fn handle_gates(State(state): State<OperatorState>) -> Json<serde_json::Value> {
1335:     let gates = crate::gates::gates_live_data(&state.root);
1336:     Json(serde_json::json!(gates))
```
Signature `State` seule, pas de `Path`, `Query`, body ou input utilisateur. L’enveloppe vient de `GatesView { gates }`.

### Livrable 3 : `gates_live_data(root) -> GatesView`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:131`, `:135`, `:149`, `:156`, `:165`
- Evidence :
```rust
131: pub fn gates_live_data(root: &Path) -> GatesView {
135:     for gate in [
142:         gates.push(GateEntryView {
144:             status: GateStatus::NotRun,
149:     gates.push(GateEntryView {
151:         status: GateStatus::NotApplicable,
156:     let lint = crate::process::lint_planning_data(root);
```
Split mustFix confirmé :
```rust
165:     if !lint.errors.is_empty() {
168:             status: GateStatus::Blocking,
172:     if !lint.warnings.is_empty() {
175:             status: GateStatus::Informational,
179:     if lint.errors.is_empty() && lint.warnings.is_empty() {
182:             status: GateStatus::Passed,
```
Aucun appel `run_gate_fg*` dans `gates_live_data`; les `WalkDir` restent dans les runners publish séparés (`gates.rs:222`, `:560`).

### Livrable 4 : structs-vue sérialisables et `GateResult` non sérialisé
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:29`, `:73`, `:97`, `:105`, `:119`
- Evidence :
```rust
29: #[derive(Debug)]
30: pub struct GateResult {
31:     pub gate: &'static str,
32:     pub passed: bool,
33:     pub issues: Vec<String>,
```
```rust
73: #[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
74: #[serde(rename_all = "snake_case")]
75: pub enum GateStatus {
97: pub struct GateIssueView {
105: pub struct GateEntryView {
119: pub struct GatesView {
120:     pub gates: Vec<GateEntryView>,
```
`GateResult` reste `Debug` only ; la route sérialise `GatesView`, pas `GateResult`.

### Livrable 5 : constantes nommées des gates
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:20`, `:201`, `:303`, `:607`, `:614`, `:688`
- Evidence :
```rust
20: pub const GATE_FG4_DIFF: &str = "FG4-diff";
21: pub const GATE_FG5_SANDBOX: &str = "FG5-sandbox";
22: pub const GATE_FG6_SECRETS: &str = "FG6-secrets";
23: pub const GATE_CSP_AUTHORING: &str = "FG-CSP-authoring";
24: pub const GATE_FG7_PREVIEW: &str = "FG7-preview";
25: pub const GATE_FG8_PROVENANCE: &str = "FG8-provenance";
27: pub const GATE_LINT_PLANNING: &str = "lint-planning";
```
Utilisation aux points de définition confirmée, par exemple `gate: GATE_FG4_DIFF` (`gates.rs:202`), `GATE_FG6_SECRETS` (`:304`), `GATE_CSP_AUTHORING` (`:608`), `GATE_FG7_PREVIEW` (`:617/:623`), `GATE_FG8_PROVENANCE` (`:688/:690`). `pipeline.rs` garde ses littéraux de comparaison substring inchangés.

### Livrable 6 : tests unitaires et HTTP
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/gates.rs:711`, `:760`, `crates/sbfb-factory/tests/operator_server.rs:754`, `:812`
- Evidence :
```rust
711: fn gates_live_data_restitutes_distinct_statuses_on_a_clean_repo() {
720:         assert!(
721:             view.gates.iter().any(|g| g.status == GateStatus::NotRun),
724:         assert!(
725:             view.gates.iter().any(|g| g.status == GateStatus::Passed),
749:         assert_eq!(csp.status, GateStatus::NotApplicable);
```
```rust
760: fn gates_live_data_splits_lint_errors_and_warnings_into_distinct_entries() {
783:         let blocking = lint
785:             .find(|g| g.status == GateStatus::Blocking)
787:         assert!(
788:             lint.iter().any(|g| g.status == GateStatus::Informational),
```
```rust
754: fn operator_gates_endpoint() {
756:     let resp = server.get("/api/gates");
771:     let statuses: Vec<&str> = gates.iter().filter_map(|g| g["status"].as_str()).collect();
785:         body.get("overall").is_none(),
789:         body.get("all_passed").is_none(),
792:     assert!(body.get("passed").is_none(), "no flattened bool at root");
812: fn operator_gates_requires_auth() {
816:         resp.starts_with("HTTP/1.1 401"),
```
Tests exécutés : `cargo test -p sbfb-factory gates_live_data --locked` et `cargo test -p sbfb-factory --test operator_server operator_gates --locked` passent tous les deux.

### Vérifications transverses
- Aucun scan publish déclenché par GET : CONFIRME. `handle_gates -> gates_live_data -> lint_planning_data`; pas d’appel `run_gate_fg*`.
- Aucun agrégat racine `overall`/`all_passed`/`passed`/`score` : CONFIRME. `GatesView` ne contient que `gates`.
- Aucune fuite de secret : CONFIRME. FG6 est restitué `NotRun` avec `issues: Vec::new()` via `gates.rs:138-145`.
- Doc-comments promissoires Phase G : CONFIRME. Les commentaires Phase G sont au présent descriptif ; pas de promesse `will/adds/ships`.
- Split errors/warnings : CONFIRME. Deux entrées distinctes sont créées si les deux collections sont non vides.
- Tests déterministes/non tautologiques : CONFIRME. Fixtures `TempDir`, assertions sur statuts, absence d’agrégat et auth 401.

## Resume final
- Total livrables : 6
- Confirmes : 6
- Gaps : 0
- Partiels : 0
- Findings de severite : aucun P0/P1/P2/P3.