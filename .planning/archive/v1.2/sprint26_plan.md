# Sprint 26 — Plan

**Ecrit** : 2026-04-22
**Kickoff** : `sprint26_kickoff.md` (meme date)
**Theme** : MCP server local + OS audit + @task_handler SDK + P2 batch

---

## 1. Phase A — P2 batch cleanup + reclassifications G7

### A.1 P2-ADMIN-1 : Windows MIL null pointer guard

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/admin_check.py`
**Lignes** : 62-64

Ajouter un check NULL sur les retours de `GetSidSubAuthorityCount` et
`GetSidSubAuthority` avant dereferencement. Si NULL, logger un warning
et retourner False (fail-closed — pas admin si SID malformed).

```python
count_ptr = GetSidSubAuthorityCount(token_sid)
if not count_ptr:
    logger.warning("admin_check_sid_malformed", detail="NULL SidSubAuthorityCount")
    return False
```

**Test** : ajouter test `test_admin_check_null_sid` avec mock
`GetSidSubAuthorityCount` retournant None.

### A.2 P2-CAPS-1 : permissions restrictives ~/.sbfb/

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/capability_store.py`
**Ligne** : 212

Apres `mkdir(parents=True, exist_ok=True)`, ajouter `os.chmod(path, 0o700)`
pour restreindre l'acces au repertoire au user courant. Sur Windows,
utiliser `icacls` ou laisser NTFS ACL par defaut (user-only si profil
standard).

```python
path.mkdir(parents=True, exist_ok=True)
if sys.platform != "win32":
    os.chmod(path, 0o700)
```

**Test** : test `test_capability_store_dir_permissions` verifie mode
0o700 post-creation (skip Windows).

### A.3 P2-REVOKE-1 : RevocationCache overwrite log + reject stale

**Fichier** : `crates/nexus-core-rs/src/key_rotation.rs`
**Ligne** : ~248 (`apply_verified`)

Avant `insert`, verifier si une entree existe deja pour
`old_public_key`. Si oui et que le `transition_start` de la nouvelle
est anterieur ou egal a l'existant, rejeter avec `Err` + log warning
"stale rotation rejected". Si le nouveau est strictement plus recent,
accepter avec log info "rotation updated".

```rust
if let Some(existing) = self.entries.get(&announcement.old_public_key) {
    if announcement.timestamp <= existing.transition_start {
        tracing::warn!("stale_rotation_rejected");
        return Err(Error::StaleRotation);
    }
    tracing::info!("rotation_updated");
}
```

**Tests** : `test_revocation_reject_stale_rotation`,
`test_revocation_accept_newer_rotation`.

### A.4 P2-HASH-1 : tomli_w determinism guard

**Fichier** : `packages/nexus-coordinator/tests/test_capability_store.py`

Ajouter test `test_toml_roundtrip_determinism` qui write → load →
re-write → compare bytes pour verifier que `tomli_w.dumps` est
deterministe across deux appels avec les memes donnees.

### A.5 P2-STAGE-1 : StageGuardrailMap key validation

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/guardrails.py`
**Ligne** : ~117

Ajouter validation dans `GuardrailChain` ou dans le constructeur du
`Dispatcher` qui verifie que chaque cle de `stage_guards` est dans
`GUARDRAIL_STAGES`. Cles invalides → `ValueError` avec message clair.

```python
invalid = set(stage_guards.keys()) - GUARDRAIL_STAGES
if invalid:
    raise ValueError(f"Invalid guardrail stages: {invalid}")
```

**Test** : `test_stage_guards_invalid_key_raises`.

### A.6 Reclassifications G7

- Ajouter LT-5 (P2-D-1) et LT-6 (P2-E-1-iroh) dans
  `docs/release/ROADMAP_COMMITMENTS.md`
- Mettre a jour l'index table

### A.7 HARDENING_ROADMAP update

- `last_validated: 2026-04-22` avec commentaire S26 kickoff
- Note S26 phase post-delivery dans la section §3 S26

### A.8 Commit

```
feat(sprint26): Phase A — P2 batch S25 audit (5 fixes) +
reclassification G7 LT-5/LT-6
```

---

## 2. Phase B — B2 MCP server local-only Streamable HTTP

### B.1 Handler MCP JSON-RPC

**Nouveau fichier** : `packages/nexus-coordinator/src/nexus_coordinator/mcp_server.py`

Implementer un handler MCP JSON-RPC qui gere les methodes :

- `initialize` → retourne capabilities `{ "tools": {} }`
- `tools/list` → retourne les 3 tools avec name + description +
  inputSchema (JSON Schema)
- `tools/call` → dispatch vers le service correspondant
  (task_service, storage_service)

Structure :
```python
class McpHandler:
    def __init__(self, task_service, storage_service, capability_store):
        ...

    async def handle_request(self, request: dict) -> dict:
        method = request.get("method")
        if method == "initialize":
            return self._initialize(request)
        elif method == "tools/list":
            return self._list_tools(request)
        elif method == "tools/call":
            return await self._call_tool(request)
        else:
            return self._error(request, -32601, "Method not found")
```

### B.2 Tool definitions avec JSON Schema

3 tools statiques :

```python
TOOLS = [
    {
        "name": "task_submit",
        "description": "Submit a compute task to the SBFB network",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "prompt": {"type": "string"},
                "model": {"type": "string"}
            },
            "required": ["project_id", "prompt"]
        }
    },
    # storage_get, storage_set similarly
]
```

### B.3 FastAPI endpoints

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/mcp.py`

- `POST /mcp` : recoit JSON-RPC request, dispatch a McpHandler,
  retourne JSON-RPC response. Header `Accept: application/json`.
- `GET /mcp` : SSE stream (pour server-initiated messages). S26
  = stub qui retourne 405 Method Not Allowed (pas de server-push
  use case avec 3 tools statiques). Carry S27 si besoin SSE reel.

Decorateurs :
- `@require_capability("mcp_server_expose")` sur les 2 endpoints
- Bearer auth via `Depends(verify_bearer_token)`
- Origin validation via `Depends(verify_origin)`

### B.4 Integration router

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/__init__.py`
ou equivalent router setup.

Ajouter le router MCP au FastAPI app.

### B.5 Tests

- `test_mcp_initialize` : retourne capabilities
- `test_mcp_list_tools` : retourne 3 tools avec schemas
- `test_mcp_call_task_submit` : dispatch correct + retour
- `test_mcp_call_storage_get` : dispatch correct
- `test_mcp_call_storage_set` : dispatch correct
- `test_mcp_unknown_method` : erreur -32601
- `test_mcp_invalid_json` : erreur -32700
- `test_mcp_capability_gate_disabled` : 403
- `test_mcp_capability_gate_enabled` : 200
- `test_mcp_origin_reject` : 403 bad Origin
- `test_mcp_bearer_auth_required` : 401 sans token
- Integration round-trip : initialize → list → call × 3

**~20 tests, ~600 LOC total Phase B.**

### B.6 Commit

```
feat(sprint26): Phase B — B2 MCP server local-only Streamable HTTP
3 tools whitelist
```

---

## 3. Phase C — A3 OS audit SecurityEvent + JsonFileWriter + ETW

### C.1 Nouveau crate nexus-events-core

**Repertoire** : `crates/nexus-events-core/`

Cargo.toml :
```toml
[package]
name = "nexus-events-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"

[target.'cfg(windows)'.dependencies]
tracing-etw = "0.2"

[dev-dependencies]
tempfile = "3"
```

### C.2 SecurityEvent enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum SecurityEvent {
    ConsentChange { previous: String, current: String },
    PanicFired { trigger: String },
    TokenRotation { rotated_at: String },
    DuressUnlock { mode: String },
    QuarantineDrop { task_id: String, reason: String },
    SybilAdmissionReject { node_id: String, reason: String },
    PowVerifyFail { difficulty: u32, peer: String },
    CanaryPublished { version: u32 },
    CanaryDeadMansSwitchTripped { last_seen: String },
    TransportDegraded { mode: String, reason: String },
    RateLimitTierBreach { consumer: String, tier: String },
    CapabilityChanged { name: String, enabled: bool },
}
```

### C.3 EventWriter trait + JsonFileWriter

```rust
pub trait EventWriter: Send + Sync {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), Error>;
}

pub struct JsonFileWriter {
    path: PathBuf,
}

impl EventWriter for JsonFileWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), Error> {
        let record = AuditRecord {
            timestamp: Utc::now().to_rfc3339(),
            event: event.clone(),
        };
        let line = serde_json::to_string(&record)?;
        // Append to file (create if needed)
        let mut file = OpenOptions::new()
            .create(true).append(true).open(&self.path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}
```

### C.4 EtwWriter (cfg-gated Windows)

```rust
#[cfg(target_os = "windows")]
pub struct EtwWriter { /* tracing-etw provider */ }

#[cfg(not(target_os = "windows"))]
pub struct EtwWriter; // stub
```

Implementation via `tracing-etw` provider name `SBFB-SecurityEvents`.
Si registration echoue (pas admin), fallback silencieux vers
JsonFileWriter.

### C.5 Stubs journald + oslog

```rust
pub struct JournaldWriter;
pub struct OsLogWriter;

impl EventWriter for JournaldWriter {
    fn write_event(&self, _event: &SecurityEvent) -> Result<(), Error> {
        tracing::debug!("journald writer stub — using JsonFileWriter fallback");
        Ok(())
    }
}
// idem OsLogWriter
```

### C.6 Global emitter + PyO3 binding

Registry global `SECURITY_EMITTER: OnceLock<Box<dyn EventWriter>>`
initialise au startup du daemon.

PyO3 binding dans `nexus-core-py` :
```rust
#[pyfunction]
fn emit_security_event(event_type: &str, payload_json: &str) -> PyResult<()> {
    // parse event_type + payload, emit via global emitter
}
```

### C.7 Wire 4 events critiques

| Event | Fichier | Point d'emission |
|---|---|---|
| CapabilityChanged | `capability_store.py` | apres `enable()`/`disable()` via PyO3 binding |
| ConsentChange | `crates/nexus-worker-core/src/consent.rs` | dans `ConsentWatcher::handle_change` |
| TokenRotation | `crates/nexus-shell-daemon-core/src/auth.rs` ou equivalent | dans rotation handler |
| PanicFired | `crates/nexus-shell-daemon/src/panic.rs` | dans `PanicWipeService::execute` |

### C.8 Tests

- `test_security_event_serialize_all_variants` (12 variantes)
- `test_json_file_writer_append` (2 events, verify JSONL)
- `test_json_file_writer_creates_file`
- `test_json_file_writer_invalid_path` (erreur propre)
- `test_audit_record_has_timestamp`
- `test_event_type_tag_correct`
- `test_mock_writer_receives_events`
- `test_etw_writer_compiles` (cfg-gated, compile-check)
- `test_stub_writers_noop`
- Integration : `test_emit_capability_changed_produces_jsonl`

**~15 tests, ~500 LOC total Phase C.**

### C.9 Commit

```
feat(sprint26): Phase C — A3 OS audit SecurityEvent + JsonFileWriter
+ 4 events wired
```

---

## 4. Phase D — C2 @task_handler SDK + Pydantic auto-schema + manifest

### D.1 Decorateur @task_handler

**Fichier** : `packages/nexus-sdk/src/nexus_sdk/decorators.py`

Nouveau decorateur qui accepte les classes Pydantic request/response
et les stocke en attributs :

```python
TASK_HANDLER_ATTR = "_nexus_task_handler"

def task_handler(
    request_model: type[BaseModel],
    response_model: type[BaseModel],
) -> Callable:
    def wrap(fn: Callable) -> Callable:
        schema_info = {
            "request_schema": request_model.model_json_schema(),
            "response_schema": response_model.model_json_schema(),
            "request_model": request_model,
            "response_model": response_model,
        }
        setattr(fn, TASK_HANDLER_ATTR, schema_info)
        return fn
    return wrap
```

### D.2 Registry integration

**Fichier** : `packages/nexus-sdk/src/nexus_sdk/registry.py`

Ajouter `TASK_HANDLER_ATTR` a la liste des attributs collectes.
Nouveau champ dans le registre : `task_handlers: list[dict]` contenant
les schemas auto-generes.

### D.3 Manifest endpoint

**Nouveau fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/manifest.py`

```python
@router.get("/app/{app_name}/manifest")
async def get_manifest(app_name: str):
    """Retourne les schemas JSON des task handlers de l'app."""
    # Lookup app in registry
    # Collect task_handler schemas
    # Return manifest JSON
```

Le manifest retourne :
```json
{
    "app_name": "translator",
    "task_handlers": [
        {
            "name": "translate",
            "request_schema": { ... },
            "response_schema": { ... }
        }
    ]
}
```

### D.4 Tests

- SDK tests :
  - `test_task_handler_stores_schema`
  - `test_task_handler_request_schema_valid_json_schema`
  - `test_task_handler_response_schema_valid_json_schema`
  - `test_task_handler_with_optional_fields`
  - `test_task_handler_registry_collects`
- Coordinator tests :
  - `test_manifest_endpoint_returns_schemas`
  - `test_manifest_endpoint_unknown_app_404`
  - `test_manifest_endpoint_no_handlers_empty`

**~10 tests, ~300 LOC total Phase D.**

### D.5 Commit

```
feat(sprint26): Phase D — C2 @task_handler SDK + Pydantic auto-schema
+ manifest endpoint
```

---

## 5. Phase E — wrap-up + verification + audit plan S27

### E.1 verification.md

25+ rows fail-fast couvrant les 3 blocs (Rust, Python, Frontend) +
checks specifiques S26 (MCP handler, SecurityEvent crate, @task_handler
decorator).

### E.2 audit_plan S27

Dimensions : Track A (P2 batch), Track B (MCP server), Track C (OS
audit), Track D (@task_handler), Track E (process/meta).

### E.3 Updates docs

- SPRINT_LOG.md : ajouter row S26
- CLAUDE.md : update compteurs tests + etat actuel
- Memory : update nexus_grid_pivot.md tip + MEMORY.md

### E.4 Migration planning

`git mv .planning/active/sprint26_*.md .planning/archive/v1.2/`

### E.5 Commit

```
chore(sprint26): Phase E — wrap-up + verification + audit plan S27
+ migration planning archive/v1.2/
```

---

## 6. Budget LOC + tests

| Phase | LOC estime | Tests estime |
|---|---|---|
| A — P2 batch | ~140 | +8 |
| B — MCP server | ~600 | +20 |
| C — OS audit | ~500 | +15 |
| D — @task_handler | ~300 | +10 |
| E — wrap-up | ~100 | 0 |
| **Total** | **~1640** | **+53** |

Sous la norme ~2500 LOC. Marge pour absorber complexite inattendue
(ETW registration, MCP edge cases).

---

## 7. Ordre d'execution

A → B → C → D → E (sequentiel, pas de parallelisation entre phases).
Chaque phase = un commit atomique. G8 preflight avant chaque phase.
Phase review avant chaque commit.
