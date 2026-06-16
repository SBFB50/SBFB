Tests non exécutés, conformément à ta contrainte.

### Livrable 1 : RuntimeTuple
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:103`, `crates/nexus-core-rs/src/task.rs:129`, `crates/nexus-core-rs/src/lib.rs:156`, `crates/nexus-core-rs/src/task.rs:1029`, `crates/nexus-core-rs/src/task.rs:1038`
- Evidence :
```rust
pub fn matches(&self, requirement: &RuntimeTuple) -> bool {
    runtime_field_matches(&requirement.model, &self.model)
        && runtime_field_matches(&requirement.quant, &self.quant)
        && runtime_field_matches(&requirement.runtime_family, &self.runtime_family)
}
```
Le type est sérialisable, wildcard-sur-vide via `runtime_field_matches`, réexporté à la racine, et les deux tests demandés existent avec assertions utiles.

### Livrable 2 : Task.required_runtime signé
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:39`, `crates/nexus-core-rs/src/task.rs:43`, `crates/nexus-core-rs/src/task.rs:315`, `crates/nexus-core-rs/src/task.rs:357`, `crates/nexus-core-rs/src/task.rs:405`, `crates/nexus-core-rs/src/task.rs:436`, `crates/nexus-core-rs/src/task.rs:451`, `crates/nexus-core-rs/src/task.rs:1066`, `crates/nexus-core-rs/src/task.rs:1089`, `crates/nexus-core-rs/src/task.rs:1111`, `crates/nexus-core-rs/src/task.rs:1123`
- Evidence :
```rust
if let Some(obj) = val.as_object_mut() {
    obj.remove("redundancy_factor");
}
```
```rust
#[serde(default)]
pub required_runtime: Option<RuntimeTuple>,
```
`task_canonical_bytes` ne retire que `redundancy_factor`; `required_runtime` est donc signé par `TaskEntry::sign`/`verify_signature`. Les tests couvrent canonical bytes, signature différente, roundtrip wire et défaut `None`.

### Livrable 3 : LlmBackend::runtime_tuple
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/mod.rs:340`, `crates/nexus-worker-core/src/llm/ollama.rs:240`, `crates/nexus-worker-core/src/llm/ollama.rs:251`, `crates/nexus-worker-core/src/llm/ollama.rs:357`, `crates/nexus-worker-core/src/llm/ollama.rs:405`
- Evidence :
```rust
RuntimeTuple {
    model: model.to_string(),
    quant: String::new(),
    runtime_family: "ollama".to_string(),
}
```
Ollama laisse bien `quant` vide et documente explicitement l’absence d’accessor fiable. Le commentaire refuse le parsing fragile du modelfile. `StubBackend` porte `quant`, `runtime_family`, `with_runtime_tuple`, et retourne ces champs.

### Livrable 4 : Claim-gate cohorte au worker
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/runtime.rs:916`, `crates/nexus-worker-core/src/engine/runtime.rs:1001`, `crates/nexus-worker-core/src/engine/runtime.rs:1046`, `crates/nexus-worker-core/src/engine/runtime.rs:1057`, `crates/nexus-worker-core/src/engine/runtime.rs:1061`, `crates/nexus-worker-core/src/engine/runtime.rs:1083`, `crates/nexus-worker-core/src/engine/runtime.rs:1146`, `crates/nexus-worker-core/src/engine/runtime.rs:1905`, `crates/nexus-worker-core/src/engine/runtime.rs:1960`
- Evidence :
```rust
if let Some(required) = task_entry.task.required_runtime.as_ref() {
    let local = self.llm.runtime_tuple(&task_entry.task.model).await;
    if !local.matches(required) {
        continue;
    }
}
let task_started_at = Instant::now();
```
Le gate est après `verify_signature` et le rate-limit, avant `task_started_at`, `claim:` et `result:`. Le test blocage assert bien tâche encore live, zéro claim et zéro result (`runtime.rs:2003`, `runtime.rs:2010`, `runtime.rs:2015`).

### Livrable 5 : model_digest DOC-NOTE
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/runtime.rs:1118`, `crates/nexus-worker-core/src/engine/runtime.rs:1350`, `crates/nexus-worker-core/src/engine/runtime.rs:1362`, `crates/nexus-core-rs/src/task.rs:484`, `crates/nexus-core-rs/src/verification.rs:23`, `.planning/codebase/protocol_wire_formats.md:168`, `.planning/codebase/security_posture.md:247`, `crates/nexus-worker-core/src/engine/runtime.rs:1815`
- Evidence :
```rust
fn model_name_digest(model: &str) -> [u8; 32] {
    blake3_hash(model.as_bytes())
}
```
```rust
assert_ne!(
    model_name_digest(model),
    blake3_hash(pretend_weight_bytes),
```
Pas de hash GGUF codé sur le chemin runtime. Les docs live modifiées disent name-hash et reportent le poids/fichier à S77. Le `Verifier` a seulement été corrigé en doc, pas raccordé au chemin live.

### Livrable 6 : Dispatcher + TaskSubmission
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/types.rs:110`, `crates/nexus-coordinator-rs/src/dispatcher.rs:70`, `crates/nexus-coordinator-rs/src/dispatcher.rs:102`, `crates/nexus-coordinator-rs/src/dispatcher.rs:212`, `crates/nexus-coordinator-rs/src/dispatcher.rs:235`, `crates/nexus-coordinator-rs/src/dispatcher.rs:248`, `crates/nexus-coordinator-rs/src/dispatcher.rs:259`
- Evidence :
```rust
let required_runtime = if submission.verifiable && redundancy > 1 {
    submission.required_runtime.clone()
} else {
    None
};
```
Les trois branches sont assertées : `verifiable && redundancy>1` garde le tuple, `redundancy==1` donne `None`, best-effort donne `None`.

### Livrable 7 : Littéraux Task/TaskSubmission complétés
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:87`, `crates/nexus-shell-daemon/src/result_sync.rs:423`, `crates/nexus-shell-daemon/src/http.rs:8041`, `crates/nexus-coordinator-rs/src/db.rs:1865`
- Evidence :
```rust
required_runtime: None,
```
Les littéraux de test explicitement touchés ont bien le champ additif à `None`; pas de changement comportemental visible dans le diff, seulement la complétion structurale.

### Livrable 8 : Acceptance LIVE B-3
- Statut : CONFIRME
- Fichier(s) : `scripts/acceptance/b3_live_pc_vps.sh:91`, `scripts/acceptance/b3_live_pc_vps.sh:95`, `scripts/acceptance/b3_live_pc_vps.sh:104`, `scripts/acceptance/b3_live_pc_vps.sh:106`, `scripts/acceptance/b3_live_pc_vps.sh:118`, `scripts/acceptance/b3_live_pc_vps.sh:135`, `scripts/acceptance/b3_live_pc_vps.sh:136`, `scripts/acceptance/b3_live_pc_vps.sh:141`
- Evidence :
```bash
-d '{"scope":"worker"}' '$VPS_DAEMON/api/v1/invite/create'")
INVITE="$(printf '%s' "$INVITE_JSON" | sed -n 's/.*\"wire\":\"\([^\"]*\)\".*/\1/p' | head -n1)"
"$WORKER_BIN" join "$INVITE"
"$WORKER_BIN" start --headless &
```
Le script utilise les routes réelles : `invite/create` retourne `wire`, `tasks/submit` existe, `GET /api/v1/tasks/{id}/result` retourne `result_text`. Le budget par défaut est 30s et le dépassement finit en `BLOCK` à diagnostiquer, pas en timeout rallongé.

### Invariants vérifiés
- `TASK_FORMAT_VERSION` reste `1` : `crates/nexus-core-rs/src/task.rs:61`.
- Aucun `DOMAIN_*` nouveau dans le diff : `canonical.rs` n’est pas modifié.
- Aucune nouvelle dépendance : aucun `Cargo.toml`, `Cargo.lock` ou manifeste package dans `git diff --name-only`.
- `validate_quorum_pre_guardrail` n’est pas modifié : `crates/nexus-coordinator-rs/src/validator.rs` absent du diff; fonction actuelle à `crates/nexus-coordinator-rs/src/validator.rs:219`.
- Le gate `e2e_network_execute_gate_real_http_no_frontier_mock` n’est pas touché : `crates/nexus-shell-daemon/src/runtime.rs` absent du diff; fonction actuelle à `crates/nexus-shell-daemon/src/runtime.rs:3965`.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0