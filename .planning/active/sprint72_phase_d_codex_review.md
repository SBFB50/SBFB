Branche vérifiée : `master`.

### Livrable 1 : Migration DB M16 `result_text`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:223`
- Evidence :
```rust
// M16: persist the accepted result text so the Operator's network
// execution arm can retrieve a completed task's output over HTTP
M::up("ALTER TABLE tasks ADD COLUMN result_text TEXT;");
];
```

### Livrable 2 : Persistance `result_text` à l’acceptation
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:394`, `crates/nexus-coordinator-rs/src/validator.rs:74`, `crates/nexus-coordinator-rs/src/validator.rs:155`
- Evidence :
```rust
pub fn set_task_result(..., result_hash: &str, result_text: &str, updated_at: u64)
"UPDATE tasks SET status = 'completed', worker_node_id = ?1,
 result_hash = ?2, result_text = ?3, updated_at = ?4"
```
```rust
db.set_task_result(&entry.payload.task_id, &worker_id, &result_hash,
    &entry.payload.result_text, now)?;
db.set_task_result(&task.task_id, worker_id, best_hash, best_hash, now)?;
```
Tous les call-sites `set_task_result(` trouvés ont le nouveau paramètre texte; les tests ciblés compilent.

### Livrable 3 : Lecture `get_task_result`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:242`, `crates/nexus-coordinator-rs/src/db.rs:413`
- Evidence :
```rust
pub struct TaskResultDetail {
    pub status: String,
    pub result_text: Option<String>,
    pub result_hash: Option<String>,
}
.prepare("SELECT status, result_text, result_hash FROM tasks WHERE task_id = ?1")?;
```

### Livrable 4 : Route daemon `/api/v1/tasks/{task_id}/result`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/tasks_api.rs:160`, `crates/nexus-shell-daemon/src/http.rs:406`, `crates/nexus-shell-daemon/src/http.rs:436`
- Evidence :
```rust
pub async fn get_task_result(...) -> impl IntoResponse {
    match db.get_task_result(&task_id) {
        Ok(Some(detail)) => match detail.result_text {
            Some(text) => (StatusCode::OK, Json(... "result_text": text ...)),
            None => (StatusCode::NOT_FOUND, Json(...)),
```
```rust
.route("/api/v1/tasks/{task_id}/result", get(crate::tasks_api::get_task_result))
.layer(middleware::from_fn_with_state(auth, auth_required));
```

### Livrable 5 : Bras `network_stream`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provider_router.rs:312`, `crates/sbfb-factory/src/provider_router.rs:329`, `crates/sbfb-factory/src/provider_router.rs:332`, `crates/sbfb-factory/src/provider_router.rs:377`, `crates/sbfb-factory/src/provider_router.rs:430`
- Evidence :
```rust
let client = reqwest::Client::new();
let submit_body = serde_json::json!({
    "project_id": project_id, "task_type": "inference",
    "prompt": prompt, "model": model,
});
```
```rust
let result_url = format!("{base_url}/api/v1/tasks/{task_id}/result");
...
yield StreamChunk::Done { cost_usd: 0.0, duration_ms: 0, result: text };
return;
```
Tests PO-14 : `provider_router.rs:778` asserte `dones.len() == 1`, `provider_router.rs:779`-`782` asserte `deltas == 0`. Recherche `network_not_implemented` : aucun résultat.

### Livrable 6 : Câblage provider backend
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:53`, `crates/sbfb-factory/src/operator_server.rs:787`, `crates/sbfb-factory/src/operator_server.rs:896`, `crates/sbfb-factory/src/operator_server.rs:934`
- Evidence :
```rust
struct ChatSession {
    model: String,
    provider: String,
    project_id: String,
}
```
```rust
if !req.provider.trim().is_empty() {
    session.provider = req.provider.clone();
}
...
if is_sensitive { return sse_gate(...); }
let target = provider_router::ExecutionTarget::from_provider(&provider, &model, &project_id);
let provider_stream = target.run(prompt, root);
```
L’ordre gate-avant-dispatch est confirmé : gate `SENSITIVE_ACTIONS` lignes `896`-`910`, dispatch lignes `934`-`935`.

### Livrable 7 : `docs/rust/PATTERNS.md` §P55
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:2889`
- Evidence :
```md
## §P55 — Sprint 72 Phase C/D : three orthogonal LLM axes (D5)
| Execution target | `ExecutionTarget { Claude, Ollama, Network }` |
| Prompt-adapt provider | `Provider` |
| Worker backend | `LlmBackend` |
```

### Livrable 8 : Catalogue menace et trust tiers
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:780`, `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:56`
- Evidence :
```md
Nouvelle route de lecture `GET /api/v1/tasks/{id}/result`
expose le `result_text` accepte d'une tache `completed`.
tier T0 loopback, lecture seule,
sous le meme middleware `auth_required`
```
```md
| `GET /api/v1/tasks/{id}/result` | S72 Phase D (option A) | T0 | T0 |
Lecture seule du `result_text` accepté ... 404 si pending/inconnu |
```

### Livrable 9 : Tests demandés avec assertions réelles
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:1151`, `crates/nexus-coordinator-rs/src/db.rs:1183`, `crates/nexus-coordinator-rs/src/validator.rs:289`, `crates/nexus-shell-daemon/src/http.rs:5070`, `crates/sbfb-factory/src/provider_router.rs:750`, `crates/sbfb-factory/src/provider_router.rs:794`, `crates/sbfb-factory/tests/operator_server.rs:293`, `crates/sbfb-factory/tests/operator_server.rs:347`, `crates/sbfb-factory/tests/operator_server.rs:379`
- Evidence :
```rust
assert_eq!(done.result_text.as_deref(), Some("hello from the network"));
assert_eq!(body["result_text"], "the network reply");
assert_eq!(dones.len(), 1);
assert_eq!(deltas, 0);
assert!(body.contains("requires_gate"));
```
Le fichier effectif est `crates/sbfb-factory/tests/operator_server.rs` pour les tests opérateur, pas `tests/operator_server.rs` à la racine.

## Résumé final
- Total livrables : 9
- Confirmés : 9
- Gaps : 0
- Partiels : 0

Tests ciblés exécutés et passés : DB `task_result`, validator `accepts_valid_result_and_transitions_to_completed`, daemon `task_result_route_404_then_text_on_completed`, Factory `network_provider`, `chat_stream_routes_by_session_provider`, `chat_session_persists_provider`, `sensitive_action_gated_regardless_of_provider`. Full gates Rust/frontend non exécutés.