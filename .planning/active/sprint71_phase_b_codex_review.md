Audit fait sur le working tree actuel (`git status --short`, `git diff HEAD`, fichiers sur disque). Résultat : les 10 livrables sont confirmés. Un contrôle transverse est à nuancer : des lignes ajoutées documentent explicitement les scope-cuts S75, mais je n’ai pas trouvé de code qui les implémente.

### Livrable 1 : `Task.verifiable` signé
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:39`, `:167`, `:247`, `:268`, `:825`
- Evidence :
```rust
39: fn task_canonical_bytes(task: &Task, domain: &[u8]) -> Result<Vec<u8>> {
42:     if let Some(obj) = val.as_object_mut() {
43:         obj.remove("redundancy_factor");
44:     }
```
`verifiable` a `#[serde(default)]` à `:167`, défaut `false` à `:247`, builder `with_verifiable` à `:268-270`. Aucun `remove("verifiable")` trouvé. Tests utiles à `:825-841`, `:845-857`, `:869-885`.

### Livrable 2 : `GenerateParams.seed` + builders
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/mod.rs:188`, `:199`, `:248`, `:259`
- Evidence :
```rust
248:     pub fn with_seed(mut self, seed: u32) -> Self {
249:         self.seed = Some(seed);
250:         self
259:     pub fn deterministic(mut self, seed: u32) -> Self {
260:         self.temperature = Some(0.0);
261:         self.seed = Some(seed);
```

### Livrable 3 : Câblage Ollama
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/ollama.rs:187`, `:239`, `:244`, `:247`
- Evidence :
```rust
187:             if let Some(opts) = deterministic_options(&params) {
188:                 req = req.options(opts);
239: fn deterministic_options(params: &GenerateParams) -> Option<GenerationOptions> {
244:     if let Some(t) = params.temperature {
245:         opts = opts.temperature(t);
247:     if let Some(s) = params.seed {
251:         opts = opts.seed(s as i32);
```
Test utile : `deterministic_options_wire_temperature_and_seed` à `:398-408`.

### Livrable 4 : Soumission worker déterministe SSI `verifiable`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/runtime.rs:1047`, `:1243`, `:1252`, `:1265`
- Evidence :
```rust
1047:                 let params = build_generate_params(&task_entry.task, &self.worker_config.watermark);
1252:     if task.verifiable {
1253:         params.deterministic(deterministic_seed(&task.task_id))
1265: fn deterministic_seed(task_id: &str) -> u32 {
1266:     let digest = blake3_hash(task_id.as_bytes());
1267:     u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
```
Le test `verifiable_task_uses_greedy_seed` assert `temperature = Some(0.0)`, `seed = deterministic_seed(task_id)`, stabilité même task id, différence autre task id, et `None` pour non-verifiable (`:1324-1356`).

### Livrable 5 : Quorum validator doc + tests
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/validator.rs:86`, `:130`, `:471`, `:501`, `:525`
- Evidence :
```rust
86: /// Workers agree by **exact equality of `result_text`**. The
87: /// `sha256` parameter is the worker's raw `result_text`: the column
130:     let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
132:         *counts.entry(r.sha256.as_str()).or_insert(0) += 1;
142:     if best_count > majority_threshold {
```
Logique inchangée dans le diff : ajout doc-comment + tests seulement. Les trois tests ont des assertions utiles : `distinct.len() == 1` (`:497`), `Accepted` + `Completed` + `result_hash` (`:517-521`), `QuorumRejected` + `Rejected` (`:547-550`).

### Livrable 6 : Suppression `RedundancyDispatcher`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/lib.rs:24-35`
- Evidence :
```rust
28: pub mod proof_card;
29: pub mod provenance;
30: pub mod public_feed;
31: pub mod quarantine_queue;
32: pub mod rerun;
```
`crates/nexus-coordinator-rs/src/redundancy.rs` n’existe plus (`Test-Path` = `False`). Grep vivant sur `RedundancyDispatcher::new|register_task(|collect_result(|redundancy::|pub mod redundancy` dans les crates : 0 hit.

### Livrable 7 : `execute_build` dormant + roadmap
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/build_executor.rs:127`, `docs/release/ROADMAP_COMMITMENTS.md:347`
- Evidence :
```rust
127: /// **Dormant entry point (Sprint 71 Phase B / D8).** Tier 2 of the
128: /// LT-7 self-hosted build pipeline. The worker dispatch does not yet
129: /// route `task_type == "build"` to this path, so neither
130: /// `execute_build` nor [`execute_build_with_timeout`] has a live
```
Aucun `#[deprecated]` trouvé dans `build_executor.rs`. Roadmap S71/LT-7 à `ROADMAP_COMMITMENTS.md:347-356`.

### Livrable 8 : Provider/backend documenté
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/process.rs:24`, `docs/rust/PATTERNS.md:2797`
- Evidence :
```rust
24: /// Prompt-adaptation **providers** the Factory targets when it
27: /// axis from the worker's runtime **execution backend** (the
28: /// `LlmBackend` in `nexus-worker-core`: Ollama / llama_cpp), which
32: /// intentionally NOT unified. (Sprint 71 Phase B / D8 ; rationale in
34: const PROVIDERS: &[&str] = &["claude", "codex", "gpt", "local", "human"];
```
Section P53 présente à `docs/rust/PATTERNS.md:2736`, provider/backend à `:2797-2806`.

### Livrable 9 : Craft path `verifiable`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/types.rs:94`, `crates/nexus-coordinator-rs/src/dispatcher.rs:72`, `:88`, `:175`
- Evidence :
```rust
94:     /// Request deterministic (greedy, fixed-seed) inference so a
100:     #[serde(default)]
101:     pub verifiable: bool,
72:     let task = Task {
88:         verifiable: submission.verifiable,
92:     let entry =
93:         TaskEntry::sign(task, keypair)
```
Le test `submit_propagates_verifiable_flag` assert le flag dans le `Task` signé et vérifie la signature (`dispatcher.rs:175-189`).

### Livrable 10 : G13 deps CVE
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint71_phase_b_preflight.md:162`, `docs/rust/PATTERNS.md:2823`, `Cargo.lock:368`, `:2587`, `:6368`
- Evidence :
```md
164: - **Scanned** (versions extraites de `Cargo.lock` via grep) :
165:   - `portable-pty 0.9.0` (G13)
166:   - `async-stream 0.3.6` (G13)
167:   - `futures 0.3.32` (G13)
197: - **Finding S1b** : **clean (non-bloquant)**. Aucune CVE critique/high
```
Cargo.lock confirme `async-stream 0.3.6` (`:368-370`), `futures 0.3.32` (`:2587-2589`), `portable-pty 0.9.0` (`:6368-6370`). P53 mentionne aussi G13 à `docs/rust/PATTERNS.md:2823-2831`.

### Contrôles transverses
- Scope cuts : PARTIEL au sens littéral. Des lignes ajoutées documentent S75/cross-machine : `build_executor.rs:134-135`, `ROADMAP_COMMITMENTS.md:352-353`, `PATTERNS.md:2785-2788`, `PATTERNS.md:2820`. Elles sont des deferrals/dormant refs, pas une implémentation. Aucun ajout trouvé pour `ProviderRouter`, `S72`, `S76`, `sharding`, `watermark V2` dans le diff code.
- Legacy decode : CONFIRME. `TASK_FORMAT_VERSION` reste `1` (`task.rs:61`) et `task_wire_default_verifiable_false` décode un JSON sans `verifiable` avec `false` (`task.rs:869-885`).
- Seed vs watermark : CONFIRME. `build_generate_params` garde `with_watermark(... task.watermark_seed ...)` (`runtime.rs:1246-1251`) puis applique `deterministic_seed(task_id)` séparément (`:1252-1253`). Les docs disent explicitement que ce seed est distinct du PRF watermark (`runtime.rs:1259-1263`, `llm/mod.rs:195-197`).

Tests ciblés exécutés et passés :
`cargo test -p nexus-core-rs verifiable --locked`, `cargo test -p nexus-worker-core verifiable_task_uses_greedy_seed --locked`, `cargo test -p nexus-worker-core deterministic_options --locked`, `cargo test -p nexus-worker-core best_effort_params_attach_no_options --locked`, les trois tests quorum, et `submit_propagates_verifiable_flag`.

## Resume final
- Total livrables : 10
- Confirmes : 10
- Gaps : 0
- Partiels : 0