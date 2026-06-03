### Livrable 1 : pins workspace `ollama-rs` / `schemars`
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:111`, `Cargo.toml:342`, `Cargo.lock:5803`
- Evidence :
```text
Cargo.toml:111: ollama-rs = "0.3.4"
Cargo.toml:342: schemars = { version = "1.2", features = ["derive"] }
Cargo.lock:5803-5811: name = "ollama-rs" / version = "0.3.4" / "schemars 1.2.1"
```
- Note : commentaire obsolète restant dans `crates/nexus-core-rs/Cargo.toml:114-118` qui cite encore `ollama-rs 0.2.6` / `0.8.21`. Le pin effectif est bien workspace `1.2`.

### Livrable 2 : dépendance directe `sbfb-factory` vers `ollama-rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/Cargo.toml:35`
- Evidence :
```text
35: # via `ollama-rs` `generate_stream` (feature `stream`). Inherits the
36: # workspace pin `0.3.4`
38: ollama-rs = { workspace = true, features = ["stream"] }
```

### Livrable 3 : migration worker Ollama vers API 0.3.4
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/ollama.rs:29`
- Evidence :
```text
29: use ollama_rs::models::ModelOptions;
163-165: Some(FormatType::StructuredJson(Box::new(ollama_json_structure())))
233-234: prompt_tokens: response.prompt_eval_count,
         completion_tokens: response.eval_count,
249: fn deterministic_options(params: &GenerateParams) -> Option<ModelOptions>
```

### Livrable 4 : migration executor vers `ModelOptions`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-executor/src/task_runner.rs:5`
- Evidence :
```text
5:  use ollama_rs::models::ModelOptions;
17: // ollama-rs 0.3.4 renamed `GenerationOptions`
19: let opts = ModelOptions::default().num_predict(params.max_tokens as i32);
20: let req = GenerationRequest::new(...).options(opts);
```
- Vérification : `cargo test -p nexus-executor --locked` - 11 tests passés.

### Livrable 5 : snapshot `TaskResponse` schemars 1.2
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/schemas/task_response.schema.json:2`, `crates/nexus-core-rs/src/schemas/task_response.rs:66`
- Evidence :
```text
schema.json:2:  "$defs": {
schema.json:22: "$schema": "https://json-schema.org/draft/2020-12/schema"
schema.json:57-60: required = ["version", "domain", "content"]
task_response.rs:66: #[derive(..., JsonSchema)]
```
- Vérification : `schema_snapshot_matches_struct` et `schema_includes_required_fields` passent.

### Livrable 6 : nouveau `provider_router.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provider_router.rs:58`
- Evidence :
```text
58: pub type ProviderStream = Pin<Box<dyn Stream<Item = StreamChunk> + Send + 'static>>;
63-70: enum ExecutionTarget { Claude, Ollama, Network }
79-90: "ollama" | "local" -> Ollama, "network" -> Network, _ -> Claude
103-106: Claude delegue a spawn_claude_stream ; Ollama -> ollama_stream
177-184: generate_stream Err -> StreamChunk::Error puis return
210-221: response -> Delta ; done -> Done
245-247: Network -> Error "Sprint 72 Phase D"
```
- Le `Result` externe de `generate_stream` est bien géré : Ollama injoignable produit un seul `StreamChunk::Error`, pas un stream vide.

### Livrable 7 : déclaration `mod provider_router`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:15`
- Evidence :
```text
15-17: commentaire Phase C / Phase D wires operator_server later
19: #[allow(dead_code)]
20: mod provider_router;
```

### Livrable 8 : tests provider router + non-régression R7
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/provider_router.rs:257`, `crates/nexus-worker-core/src/llm/ollama.rs:408`, `crates/nexus-coordinator-rs/src/validator.rs:471`
- Evidence :
```text
257-285: parsing closed set avec assert_eq! sur Claude/Ollama/local/Network/default
301-318: Claude compare via_target vs spawn_claude_stream direct
339-366: Ollama unreachable assert len == 1 + Error diagnostic
400-413: Network assert Error contient "Phase D"
408-418: deterministic_options_wire_temperature_and_seed assert temperature/seed
471,501,525: tests quorum R7 presents
```
- PARTIEL : `ollama_stream_maps_to_chunks` existe (`provider_router.rs:370-397`) mais accepte un “skip” silencieux si Ollama renvoie un seul `Error` :
```text
382: let only_error = chunks.len() == 1 && matches!(chunks[0], StreamChunk::Error { .. });
383: if only_error {
384:     // Ollama absent or model not pulled - clean skip.
385:     return;
```
Donc le mapping `Delta`/`Done` n’est pas prouvé de façon déterministe sans Ollama local + modèle.

### Contrôles spécifiques
- `operator_server.rs` non câblé à `ExecutionTarget` : CONFIRME. `rg ExecutionTarget|provider_router crates/sbfb-factory/src/operator_server.rs` ne retourne rien, et `handle_chat_stream` appelle encore directement `llm_bridge::spawn_claude_stream` à `operator_server.rs:898`.
- Gate `SENSITIVE_ACTIONS` intact : CONFIRME. Déclaration `operator_server.rs:34`, gate SSE `operator_server.rs:866-878`.
- `*_VERSION` : CONFIRME. Aucun `pub const *_VERSION` modifié dans le diff staged ; seule occurrence diffée est du texte de description dans le JSON schema. Les constantes restent visibles par `rg _VERSION crates/nexus-core-rs/src`, dont `TASK_RESPONSE_VERSION = 1` à `task_response.rs:48`.

### Tests exécutés
- `cargo test -p sbfb-factory --locked provider_router` : 6 passed.
- `cargo test -p nexus-worker-core --locked verifiable_task_uses_greedy_seed` : passed.
- `cargo test -p nexus-worker-core --locked deterministic_options_wire_temperature_and_seed` : passed.
- `cargo test -p nexus-coordinator-rs --locked two_honest_workers_same_hash` : passed.
- `cargo test -p nexus-coordinator-rs --locked quorum_accepts_deterministic_redundancy` : passed.
- `cargo test -p nexus-coordinator-rs --locked quorum_rejects_nondeterministic_divergence` : passed.
- `cargo test -p nexus-core-rs --locked schema_snapshot_matches_struct` : passed.
- `cargo test -p nexus-core-rs --locked schema_includes_required_fields` : passed.
- `cargo test -p nexus-executor --locked` : 11 passed.

## Resume final
- Total livrables : 8
- Confirmes : 7
- Gaps : 0
- Partiels : 1