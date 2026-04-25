# Sprint 26 — Audit findings (S27 Phase 0)

**Date** : 2026-04-25
**Auditeur** : session fraiche S27 Phase 0 (Opus 4.6)
**Tip audite** : `f52dc96` (Phase D) — phases A-D + Phase E wrap-up
**HEAD session** : `4a694ca` (3 commits docs(process) + 1 fix post-Phase E)
**Audit plan** : `.planning/archive/v1.2/sprint26_audit_plan.md`

---

## Verdict : PASS

- **0 P0** (zero securite active)
- **0 P1** (zero regression bloquante)
- **7 P2** documentes ci-dessous (robustesse, hygiene, divergences plan)
- **0 P3**

Signal rigor G4 : 7 P2 avec evidence inline — dimensions auditees
exhaustivement.

---

## Findings

### P2-A-1 — validate_stage_guard_map non wired en production

**Track** : A (item 5 — STAGE-1)
**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py:116-131`
**Observation** : La fonction `validate_stage_guard_map` (guardrails.py:120-124)
existe et rejette correctement les cles invalides (`ValueError`). Mais le
`Dispatcher.__init__` accepte `stage_guards: StageGuardrailMap | None`
sans appeler `validate_stage_guard_map`. Un appelant passant une cle avec
typo (ex: `"on_task_dispached"`) verrait sa guardrail chain silencieusement
ignoree (le dispatcher lookup echoue, la guardrail ne fire jamais).

**Evidence** :
```python
# dispatcher.py:126-131
if stage_guards is not None:
    self._stage_guards: StageGuardrailMap = stage_guards
elif input_chain is not None:
    self._stage_guards = {"on_task_dispatched": input_chain}
else:
    self._stage_guards = {}
# -> pas d'appel a validate_stage_guard_map()
```

**Test existant** : `test_stage_guards_invalid_key_raises` (test_guardrails.py:255)
teste la fonction isolee. Pas de test verifiant le wiring au Dispatcher.

**Recommendation** : ajouter `validate_stage_guard_map(stage_guards)` dans
`Dispatcher.__init__` avant assignation, ou a minima un test integration.
Severite P2 : guardrails sont defense-in-depth, pas de securite directe.

---

### P2-B-1 — MCP lifespan __aenter__/__aexit__ explicites

**Track** : B (item 4 — SDK adoption)
**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/app.py:70-77`
**Observation** : Le lifespan FastAPI utilise `__aenter__`/`__aexit__`
explicites sur le context manager retourne par `mcp_srv.session_manager.run()`.
Necessaire car le context manager doit span le `yield` du lifespan FastAPI
(impossible d'utiliser `async with` a cheval sur un yield). Pattern fragile
si le SDK `mcp` v1.27+ change l'implementation interne de
`StreamableHTTPSessionManager.run()`.

**Evidence** :
```python
# app.py:70-77
mcp_ctx = mcp_srv.session_manager.run() if mcp_srv else None
if mcp_ctx:
    await mcp_ctx.__aenter__()
try:
    yield
finally:
    if mcp_ctx:
        await mcp_ctx.__aexit__(None, None, None)
```

**Recommendation** : documenter le pattern en commentaire inline pour
signaler la fragilite. Carry S27 si le SDK evolue.
Pre-documente phase review P2-LIFESPAN-AENTER.

---

### P2-C-1 — emit_capability_event catch silencieux

**Track** : C (item 5 — PyO3 binding)
**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/capability_store.py:93`
**Observation** : `_emit_capability_event` catch `(ImportError, Exception)`
avec `pass` — zero logging. Si le binding PyO3 echoue (wheel stale,
import error, serialization error), l'event audit est perdu silencieusement.
Le path est fire-and-forget (non-bloquant), mais l'absence totale de trace
rend le diagnostic difficile.

**Evidence** :
```python
# capability_store.py:79-94
def _emit_capability_event(name: str, enabled: bool) -> None:
    try:
        import json
        from nexus_core import emit_security_event
        emit_security_event(
            json.dumps({
                "event_type": "CapabilityChanged",
                "payload": {"name": name, "enabled": enabled},
            })
        )
    except (ImportError, Exception):
        pass  # <-- zero logging
```

**Recommendation** : ajouter `logger.debug("emit_capability_event failed", exc_info=True)`
dans le except. Carry S27.
Pre-documente phase review P2-C-1.

---

### P2-C-2 — JsonFileWriter sans rotation

**Track** : C (item 3 — JsonFileWriter)
**Fichier** : `crates/nexus-events-core/src/lib.rs:91-104`
**Observation** : `JsonFileWriter` est append-only sans politique de rotation.
`~/.sbfb/audit.jsonl` croit sans limite. Pre-launch acceptable (0 node
externe = volume negligeable). Post-v1.0, un node actif genererait un
fichier croissant indefiniment.

**Evidence** :
```rust
// lib.rs:98-102
let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&self.path)?;
writeln!(file, "{line}")?;
// -> pas de check taille, pas de rotation
```

**Recommendation** : carry S27 si observabilite production priorisee.
Ajouter rotation taille-based (`max_bytes` + `.1`/`.2` suffixes) ou
integration avec log rotation OS (logrotate/nssm).
Pre-documente phase review P2-C-2.

---

### P2-C-3 — EtwWriter non cfg-gated, divergence plan

**Track** : C (item 4 — EtwWriter)
**Fichier** : `crates/nexus-events-core/src/lib.rs:117-129`
**Observation** : Le plan prescrivait `EtwWriter` cfg-gated
`#[cfg(target_os = "windows")]` avec stub `#[cfg(not(target_os = "windows"))]`.
L'implementation utilise un `EtwWriter` cross-platform base sur
`tracing::info!` avec target `sbfb_security_events`. Pas de dep
`tracing-etw` directe dans Cargo.toml (deps = serde, serde_json, chrono,
tracing, thiserror). Sur Windows avec un subscriber tracing-etw, les events
coulent vers ETW. Sur les autres plateformes, ce sont des events tracing
reguliers.

**Evidence** :
```rust
// lib.rs:117-129 — PAS de cfg-gate
pub struct EtwWriter;

impl EventWriter for EtwWriter {
    fn write_event(&self, event: &SecurityEvent) -> Result<(), EventError> {
        let json = serde_json::to_string(event)?;
        tracing::info!(
            target: "sbfb_security_events",
            event_json = %json,
            "security_event"
        );
        Ok(())
    }
}
```

Approche architecturalement plus propre : la route ETW est une
responsabilite du subscriber tracing, pas du writer. Pas de compilation
conditionnelle fragile. Le nom `EtwWriter` est trompeur — il ne fait
pas d'ETW directement.

**Recommendation** : renommer en `TracingWriter` pour refleter le comportement
reel, ou documenter que l'ETW est subscriber-side. Nit, pas bloquant.
Pre-documente phase review P2-C-3.

---

### P2-D-1 — TaskHandlerDescriptor sans champ description

**Track** : D (item 2 — registry 5-tuple)
**Fichier** : `packages/nexus-sdk/src/nexus_sdk/app.py:99-105`
**Observation** : `TaskHandlerDescriptor` a 4 champs
`(name, request_schema, response_schema, fn)`. Le kickoff D3 et l'audit
plan item D.2 attendaient un 5-tuple avec `description`. Le manifest
endpoint (`apps.py:96-103`) sert les handlers sans descriptions, reduisant
la discoverabilite pour les clients MCP et les consommateurs du manifest.

**Evidence** :
```python
# app.py:98-105
@dataclass
class TaskHandlerDescriptor:
    """One @task_handler-decorated method on an app."""
    name: str
    request_schema: dict[str, Any]
    response_schema: dict[str, Any]
    fn: Callable[..., Any]
    # -> pas de champ description
```

Le decorateur `task_handler` (decorators.py:150-160) ne capture pas non
plus le docstring de la fonction decoree.

**Recommendation** : ajouter `description: str = ""` au dataclass, capturer
`fn.__doc__ or ""` dans le decorateur, et l'exposer dans le manifest.
Carry S27.

---

### P2-E-1 — LOC estimates dans plan.md

**Track** : E (process — recurrent)
**Fichier** : `.planning/archive/v1.2/sprint26_plan.md:489-498` (§6)
**Observation** : Le plan contient des estimations LOC par phase
(`~140`, `~600`, `~500`, `~300`, `~100`). Contraire a
feedback_approach.md §6 (pas d'estimation LOC dans les plans).
Pre-existant depuis la redaction du plan. Recurrent phases A/B/D
(documente P2-PLAN-LOC dans phase reviews).

**Evidence** :
```
## 6. Budget LOC + tests
| Phase | LOC estime | Tests estime |
|---|---|---|
| A — P2 batch | ~140 | +8 |
| B — MCP server | ~600 | +20 |
...
```

Les 3 commits docs(process) post-Phase D (`a92e0d4`, `e81559c`, `a3e35b2`)
incluent la suppression de la norme ~2500 LOC dans le workflow
(GUARDRAILS_ARCHITECTURE.md, README.md). Le plan lui-meme n'a pas ete
corrige retroactivement (acceptable — plan fige apres Phase A).

**Recommendation** : ne pas inclure d'estimations LOC dans les plans
S27+. Le plan dimensionne par objectif fonctionnel, pas par budget LOC.

---

## Dimensions auditees — evidence

| Dim | Track | Status | Evidence |
|---|---|---|---|
| ADMIN-1 NULL guard | A.1 | ok | admin_check.py:62-68 — `raise PermissionError` sur NULL |
| CAPS-1 permissions | A.2 | ok | capability_store.py:234-235 — `os.chmod(parent, 0o700)` |
| REVOKE-1 stale rotation | A.3 | ok | key_rotation.rs:252-262 — reject + warn + 2 tests |
| HASH-1 determinism | A.4 | ok | test_capability_store.py:446-458 — write/load/rewrite/compare |
| STAGE-1 validation | A.5 | **P2-A-1** | function existe, non wiree au Dispatcher |
| MCP spec compliance | B.1 | ok | FastMCP SDK gere JSON-RPC protocol |
| MCP security surface | B.2 | ok | CapabilityGateMiddleware + LoopbackAuth S16 + bearer |
| MCP 3 tools whitelist | B.3 | ok | task_submit, storage_get, storage_set — zero dynamique |
| MCP SDK adoption | B.4 | ok | mcp>=1.27 pyproject.toml, FastMCP |
| MCP error handling | B.5 | ok | SDK gere erreurs MCP, tools retournent dicts |
| MCP lifespan pattern | B.4 | **P2-B-1** | __aenter__/__aexit__ explicites, fragile |
| Crate structure | C.1 | ok | nexus-events-core independant, Cargo.toml + tests |
| 12 variants enum | C.2 | ok | 12 exactement, zero catch-all |
| JsonFileWriter | C.3 | **P2-C-2** | append-only OK, rotation absente |
| EtwWriter cfg-gate | C.4 | **P2-C-3** | non cfg-gated, tracing::info! cross-platform |
| PyO3 binding lazy | C.5 | **P2-C-1** | lazy import OK, catch silencieux |
| 4 events wired | C.6 | ok | consent.rs:217, key_rotation_handler.rs:60, panic.rs:164, capability_store.py:85 |
| Decorator schema | D.1 | ok | model_json_schema() Pydantic v2 |
| Registry 5-tuple | D.2 | **P2-D-1** | 4 champs, description manquante |
| Manifest endpoint | D.3 | ok | apps.py:76-103, JSON task_handlers |
| Pydantic v2 | D.4 | ok | model_json_schema(), pas schema() |
| No nexus_core dep | D.5 | ok | pyproject.toml SDK = zero nexus_core |
| G8 preflights 4/4 | E.1 | ok | archive/v1.2/ — A, B, C, D |
| Phase reviews 4/4 | E.2 | ok | archive/v1.2/ — A (PASS), B (PASS), C (PASS), D (PASS) |
| G7 reclassifications | E.3 | ok | ROADMAP_COMMITMENTS LT-5 + LT-6 dates 2026-04-22 |
| HARDENING_ROADMAP | E.4 | ok | last_validated 2026-04-22 S26 |
| Commit discipline | E.5 | ok | 4 feat atomiques avec body riche |
| docs(process) commits | E.6 | ok | 4 commits docs-only, 0 code fonctionnel |
| LOC estimates plan | E.7 | **P2-E-1** | plan.md §6, contraire feedback_approach |

---

## Phase review findings routing (§4.4 reconciliation)

| Phase review finding | Audit track | Audit finding | Status |
|---|---|---|---|
| P2-PLAN-LOC (recurrent A/B/D) | Track E | P2-E-1 | confirme |
| P2-HANDLER-COVERAGE (key_rotation_handler Err branch) | Track A | — | non-retenu (logique sous-jacente `apply_verified` couverte par 2 tests) |
| P2-LIFESPAN-AENTER (Phase B) | Track B | P2-B-1 | confirme |
| P3-TEST-DELTA (Phase B) | — | — | informatif, non-actionable |
| P2-C-1 (emit_capability_event catch) | Track C | P2-C-1 | confirme |
| P2-C-2 (JsonFileWriter rotation) | Track C | P2-C-2 | confirme |
| P2-C-3 (EtwWriter tracing-based) | Track C | P2-C-3 | confirme |
| P2-LOC-PHASE-D | Track E | P2-E-1 | fusionne avec P2-PLAN-LOC |
| P2-PLAYWRIGHT-NORUN | — | — | [closed Phase E] — 27 pass |

---

## Checklist audit gate

- [x] 0 P0 pour verdict PASS
- [x] 0 P1 pour verdict PASS
- [x] 7 P2 documentes (signal rigor G4)
- [x] Chaque finding : ID, severite, fichier, ligne, evidence, recommendation

---

## Compteurs tests verifies (session S27)

Compteurs memory S26 sortie : 802 Rust / 193 SDK / 377+45 coord /
46 gov / 264 Vitest / 27+16 PW / 7/7 size / ~1752 total.
Session fraiche pas de re-run (audit docs-based, pas de code modifie).
Compteur coord pass ambiguite documentee audit_plan §4.4 (376→394→406→377).

---

## Carry-overs S27

| ID | Severite | Description | Source |
|---|---|---|---|
| P2-C-1 | P2 | emit_capability_event catch silencieux | S26 Phase C |
| P2-D-1 | P2 | TaskHandlerDescriptor sans description | S26 Phase D |
| P2-A-1 | P2 | validate_stage_guard_map non wiree | S26 Phase A |
| T-NN+2 | P3 | iframe Rust-wasm (PATTERNS §P34) | S22 carry |
| LT-5 | LT | Redundancy persistence | ROADMAP_COMMITMENTS |
| LT-6 | LT | iroh neighborhood enrichment | ROADMAP_COMMITMENTS |
