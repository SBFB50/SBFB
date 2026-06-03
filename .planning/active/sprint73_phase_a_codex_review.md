Audit effectué sur le working tree actuel. Je n’ai pas modifié de fichier.

### Livrable 1 : Split validation / persist
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/validator.rs:41`, `:68`, `:117`, `:154`, `:158`, `:259`
- Evidence :
```rust
pub struct PendingResultPersist {
    pub task_id: String,
    pub worker_id: String,
    pub result_hash: String,
    pub result_text: String,
```
```rust
pub fn validate_result_pre_guardrail(
    db: &CoordinatorDb,
    entry: &ResultEntry,
) -> Result<(ValidationOutcome, Option<TaskRecord>, Option<PendingResultPersist>),
```
```rust
pub fn validate_result_post_guardrail(
    db: &CoordinatorDb,
    pending: &PendingResultPersist,
) -> Result<(), CoordinatorError> {
    db.set_task_result(
```
Le chemin quorum retourne `PendingResultPersist` avec `best_hash` comme texte (`validator.rs:259-269`). Recherche `set_task_result(` : hors définition et tests, le seul appel production est `validate_result_post_guardrail`.

### Livrable 2 : HTTP `coordinator_submit_result`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:1500`, `:1510`, `:1515`, `:1523`, `:1531`, `:1542`
- Evidence :
```rust
match nexus_coordinator_rs::validator::validate_result_pre_guardrail(&db, &entry) {
```
```rust
let guardrail_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
    system_prompt: "",
    user_prompt: "",
    model_output: &pending.result_text,
};
let gr = nexus_coordinator_rs::guardrails::default_output_chain().run(&guardrail_ctx);
```
```rust
return (
    StatusCode::BAD_REQUEST,
    Json(serde_json::json!({"outcome": "rejected", "reason": "guardrail_rejected"})),
)
```
Le post-persist (`validate_result_post_guardrail`) est après le guardrail (`http.rs:1531-1532`), et le kudos credit est encore après (`http.rs:1542-1548`).

### Livrable 3 : `validator_loop::process_result`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/validator_loop.rs:69`, `:76`, `:81`, `:82`, `:89`, `:91`, `:99`
- Evidence :
```rust
match validator::validate_result_pre_guardrail(&guard, entry) {
    Ok((ValidationOutcome::Accepted, Some(task_record), Some(pending))) => {
```
```rust
let guardrail_ctx = GuardrailContext {
    system_prompt: "",
    user_prompt: "",
    model_output: &pending.result_text,
};
let gr = default_output_chain().run(&guardrail_ctx);
```
```rust
if !gr.passed {
    let reason = gr.tripwire.unwrap_or_else(|| "guardrail_rejected".into());
    tracing::warn!(
    return;
}
```
Persist (`validator_loop.rs:91`) et kudos (`validator_loop.rs:99`) sont uniquement après passage du guardrail.

### Livrable 4 : Tests ajoutés
- Statut : CONFIRME
- Fichier(s) : `http.rs:4277`, `:4309`, `:4323`, `:4336`, `:4344`, `:4372`, `:4379`; `validator_loop.rs:260`, `:281`, `:295`, `:303`, `:323`; `validator.rs:717`, `:740`, `:751`, `:760`
- Evidence :
```rust
assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
assert_eq!(body["outcome"], "rejected");
assert_eq!(body["reason"], "guardrail_rejected");
```
```rust
assert_ne!(task.status, nexus_coordinator_rs::types::TaskStatus::Completed);
assert!(db.get_task_result(&task_entry.task.task_id)?.unwrap().result_text.is_none());
assert_eq!(db.get_project_kudos_total("test-project").expect("kudos"), 0);
```
```rust
assert_eq!(resp.status(), StatusCode::OK);
assert_eq!(body["outcome"], "accepted");
assert_eq!(...result_text.as_deref(), Some("clean answer"));
```
```rust
assert_eq!(pending.result_text, agreed, "the guardrail must run on the AGREED text");
let gr = crate::guardrails::default_output_chain().run(&ctx);
assert!(!gr.passed);
```
Assertions utiles présentes. Tests ciblés exécutés : `quorum_guardrail_runs_on_agreed_text`, `submit_result_`, `validator_loop_` passent.

### Livrable 5 : `THREAT_MODEL.md` section 14
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:780`, `:786`, `:789`, `:791`
- Evidence :
```md
Le `result_text` ne devient `completed`/lisible **qu'apres** passage du
guardrail de sortie. Sur les **deux** chemins d'ingestion d'un resultat —
HTTP `coordinator_submit_result` et la boucle gossip `validator_loop` —
le `default_output_chain` tourne AVANT `set_task_result`
```
```md
`validate_result_post_guardrail`). Un texte qui declenche un tripwire
n'est **jamais persiste** (aucune ligne `completed`, rien a relire) et ne
credite aucun kudos.
```
La vieille justification “filtré à l’acceptation” n’est pas présente dans les fichiers audités.

### Livrable 6 : `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3`, `:46`, `:53`, `:72`, `:246`, `:253`
- Evidence :
```md
last_validated: 2026-06-03
```
```md
### 2.1 Portée : deux serveurs loopback (daemon + Operator)
| **Daemon** (`nexus-shell-daemon`) | dynamique | ... |
| **Operator** (`sbfb-factory`, §3.1) | `:3001` | ... | **T0 uniformément** |
```
```md
| `GET /api/v1/tasks/{id}/result` | ... | Lecture seule du `result_text` —
persisté **uniquement APRÈS** passage de l'output guardrail ...
```
```md
### 8.1 Couverture Operator (`:3001`)
| AD2 — Malware user-mode lit le token bearer | Gate applicatif ... |
| AD3 — Multi-user OS ... | **Aucune** mitigation peer-creds ... |
| AD4 — Compromise du shell/Viewer | ... |
```

### Livrable 7 : `HARDENING_ROADMAP.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/HARDENING_ROADMAP.md:3`, `:155`, `:156`, `:164`, `:166`
- Evidence :
```md
last_validated: 2026-06-03
```
```md
> **Statut (mis à jour 2026-06-03, S73 Phase A — P2-HARDENING-ROADMAP-META-STALE).**
> Le backlog de séquencement S18-30 ci-dessous est **clos** : ces sprints
> sont tous CLOSED (le projet en est à S73, roadmap v5 Arc 3.5).
```
```md
> **La posture de sécurité courante vit dans les threat docs vivants**,
> [`THREAT_MODEL.md`](THREAT_MODEL.md) ...
> [`LOOPBACK_ENDPOINTS_TRUST_TIERS.md`](LOOPBACK_ENDPOINTS_TRUST_TIERS.md)
```

## Resume final
- Total livrables : 7
- Confirmes : 7
- Gaps : 0
- Partiels : 0