# Sprint 26 — Kickoff (MCP server local + OS audit + @task_handler SDK + P2 batch)

**Ecrit** : 2026-04-22 (session fraiche post-audit gate S25 `3a6f235`).
**Type** : **sprint implementation** (exploitation D5 capabilities S25 :
MCP server local expose + production observability OS audit + SDK
auto-schema @task_handler).
**Tip master d'entree** : `3a6f235` (chore(sprint25): audit gate S25 PASS).
**Phase 0 audit Sprint 25** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint25_audit_findings.md` (verdict **PASS**,
0 P0/P1, 5 P2 pre-documentes, 2 P3 nits).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-22, re-validation S26 ouverture meme
  jour que S25 cloture — leverage S25 scan results) :
  - `arti-client > 1.x stable` : **INACTIVE** (context7 confirme :
    API still pre-1.0, "please expect a certain amount of breakage
    between now and us declaring arti-client 1.x", peut appeler
    `exit(1)` si consensus reseau le declare obsolete). Tor transport
    defere S27+.
  - `MCP spec revision Anthropic 2026+` : **INACTIVE** (spec version
    2025-11-25 inchangee). Vulnerabilite STDIO transport RCE avril
    2026 toujours active — B2 design integre mitigations (capability
    gate + Streamable HTTP pas stdio + validated schemas + Origin
    check).
  - `openai-agents-python > 0.7.0` : INACTIVE (v0.14.3 inchange).
  - `frost-ed25519 > 2.1` : INACTIVE (2.1.0 inchange).
  - `iroh release > 0.97` : INACTIVE.
  - `wasmtime LTS bump` : INACTIVE.
  - `microsoft/sudo > 24H2` : INACTIVE.
  - `NIST PQC FIPS 203/204 ecosystem default` : INACTIVE.
  - `NVIDIA H100 CCM driver release` : INACTIVE.
  - `RFC 9591 erratum` : INACTIVE.
  - `Sprint S+2 trigger` : S28 entries dans HARDENING_ROADMAP =
    Nym mixnet + MIG + audit prep. Non-bloquant pour S26.
- **context7 MCP spec 2025-11-25** (pre-gel D1) : Streamable HTTP
  transport = POST /mcp (client→server JSON-RPC) + GET /mcp (SSE
  server→client). Security best practices : validate Origin header,
  bind localhost, authorization tokens ou IPC restreint. Tools
  capability avec `listChanged` notification. JSON Schema pour tool
  parameters.
- **context7 arti-client** (pre-gel D5 scope) : crate 0.x semver
  unstable, lower-level `tor-*` crates encore plus instables. Pas
  de timeline 1.0.

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `3a6f235` — S25 audit gate PASS, 0 P0/P1.
- **Working tree** : propre (post-migration `sprint25_audit_findings.md`
  → `archive/v1.2/`).
- **v1.2** : continuation security hardening. Pas de nouvelle version.

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S26 prescrit : "Tor complete + curator reliable +
GPU lockup" (~5300 LOC cumul avec carries S25 non livres). Note
realisme ajoutee : la norme empirique est ~2500 LOC / sprint. Le
kickoff doit arbitrer.

**Arbitrage S26** : Tor bloque (arti pre-1.0), GPU lockup couple a A4
process roles (complexe). Prioriser l'exploitation immediate de D5
capabilities livres S25 Phase D : B2 MCP server expose (tool-calling
unlock), A3 OS audit channel (observabilite production). Ajout C2/B3
@task_handler SDK (dep B3 non livree, subsumable dans C2, SDK value
directe). P2 batch 5 items S25 audit.

Items prescrits HARDENING_ROADMAP S26 non retenus → S27+ backlog :
Tor transport, Arti library-embed, domain fronting, reliable-workers
curator, GPU lockup, no-sharing policy, A4 process role tagging, C1
SQLiteSession.

### 1.3 Compteurs tests entree (tip `3a6f235`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 790 | all pass |
| Rust doctests | pass | |
| Python SDK | 185 | all pass |
| Python coord | 372 pass + 32 fail + 5 skip | 32 fail = stale PyO3 wheel |
| Python gov | 46 | all pass |
| Vitest | 264 | all pass |
| Playwright | 43 | all pass |
| Size-limit | 7/7 | |
| **Total** | **~1712** | |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. Pas de
tolerant decoder multi-version. S26 n'introduit pas de nouveau wire
format gossip/P2P. L'endpoint MCP est local-only HTTP sur le
coordinator (pas de wire P2P, pas de FORMAT_VERSION).

---

## 2. Goal

Exploiter les capability gates D5 livres S25 pour exposer un serveur
MCP local (B2), ajouter une couche d'observabilite OS audit events
(A3), et doter le SDK d'un decorateur @task_handler auto-schema (C2).
P2 batch cleanup en Phase A.

**Critere SMART : 25+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase E wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 25

**Verdict** : PASS (0 P0/P1, 5 P2 pre-documentes, 2 P3 nits).
**Commit** : `3a6f235` — `sprint25_audit_findings.md` migre vers
`archive/v1.2/` dans ce commit d'ouverture.
**P2 carry S26** : 5 items (ADMIN-1, CAPS-1, REVOKE-1, HASH-1,
STAGE-1) absorbes Phase A ci-dessous.
**Reclassification G7** : P2-D-1 (redundancy persistence) + P2-E-1-iroh
(neighborhood enrichment) reclassifies long-term commitments apres 3+
sprints de carry consecutifs (regle §6.2.1). Entrees LT-5 + LT-6
ajoutees dans `docs/release/ROADMAP_COMMITMENTS.md`.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — B2 MCP server : Streamable HTTP local-only coordinator-side

**Retenu** : serveur MCP conforme spec 2025-11-25 integre au
coordinator FastAPI. Transport Streamable HTTP (POST `/mcp` JSON-RPC +
GET `/mcp` SSE) sur loopback uniquement (bind `127.0.0.1`, pas
`0.0.0.0`). 3 tools whitelist mirrorant le bridge S13 :

- `task_submit(project_id, prompt, model?)` → soumet une tache
- `storage_get(project_id, key)` → lit une valeur storage
- `storage_set(project_id, key, value)` → ecrit une valeur storage

Chaque tool defini avec JSON Schema strict (input_schema requis par
MCP spec). Capability-gate via `@require_capability("mcp_server_expose")`
(D5 S25). Bearer auth X-SBFB-Token reuse (pattern loopback S16).
Origin validation + Host allowlist (pattern S16 DNS rebinding
mitigation). Pas de `listChanged` notification (3 tools statiques).

**Rejete** :
- **stdio transport** : incompatible avec le coordinator FastAPI
  (processus async deja en ecoute HTTP). Requierait processus
  secondaire. De plus, vuln STDIO RCE avril 2026 (OX Security)
  confirme que stdio est un vecteur d'attaque actif — HTTP avec auth
  est plus secure pour SBFB. Cf. MCP spec security best practices :
  "servers should implement authorization tokens".
- **Plugin marketplace / auto-discovery** : surface d'attaque non
  bornee. SBFB expose uniquement ses propres 3 tools, pas un
  marketplace tiers. Zero auto-discovery, zero tool registration
  dynamique.
- **WebSocket transport** : non standardise dans la MCP spec
  2025-11-25 (seuls Streamable HTTP et stdio sont specifies).
  Introduire un transport non-standard = incompatibilite avec les
  MCP clients existants (Claude Code, Cursor, etc.).

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/mcp_server.py`
  (handler MCP JSON-RPC : initialize, tools/list, tools/call)
- Nouveau endpoint group dans le router FastAPI (`/mcp` POST + GET)
- Schemas JSON inline pour les 3 tools
- Reuse `task_service.py`, `storage_service.py` existants pour la logique

### D2 — A3 OS audit : SecurityEvent enum + audit trail file + platform writers

**Retenu** : nouveau crate `crates/nexus-events-core/` avec :
- `SecurityEvent` enum 12 variantes (consent_change, panic_fired,
  token_rotation, duress_unlock, quarantine_drop,
  sybil_admission_reject, pow_verify_fail, canary_published,
  canary_dead_mans_switch_tripped, transport_degraded,
  rate_limit_tier_breach, capability_changed)
- Trait `EventWriter` avec methode `write_event(&self, event:
  &SecurityEvent) -> Result<()>`
- `JsonFileWriter` impl (audit trail `~/.sbfb/audit.jsonl` — JSONL
  append-only, 1 event/ligne, avec timestamp ISO 8601 + event type +
  payload serialise). Impl universelle, fonctionne sur toutes les
  plateformes.
- `EtwWriter` impl Windows (cfg-gated `target_os = "windows"`) via
  crate `tracing-etw` — emet les events vers ETW provider
  `SBFB-SecurityEvents`. Disponible dans Event Viewer.
- `JournaldStub` + `OsLogStub` impls (cfg-gated, log warning "platform
  writer not yet implemented, using JsonFileWriter fallback"). Carry S27
  pour impls completes.
- PyO3 binding `nexus_core.emit_security_event(event_type, payload_json)`
  pour emission depuis Python coordinator.

Wire 4 events critiques S26 (les 8 restants → S27) :
- `capability_changed` dans `capability_store.py` (replace structlog)
- `consent_change` dans consent watcher Rust
- `token_rotation` dans `TokenRotator` Rust
- `panic_fired` dans panic wipe handler Rust

**Rejete** :
- **OTEL collector direct** : OTEL est pour la telemetrie distribuee
  (spans, metrics). Les security events sont des logs d'audit one-shot.
  `tracing-opentelemetry` est pour A2 S29 (post-audit externe). Premature
  S26 — over-engineering pour 4 events.
- **Fichier log structure seul (status quo)** : deja en place via
  structlog (Python) et tracing (Rust). Le gain A3 est le format
  unifie cross-langage JSONL + la possibilite d'integration avec les
  outils OS natifs (Event Viewer, journalctl, Console.app). Sans le
  crate unifie, chaque point d'emission reinvente le format.
- **Syslog** : obsolete face aux API natives modernes. Windows n'a
  jamais eu de syslog natif. macOS a deprecie syslog en faveur d'oslog
  depuis 10.12. Linux journald est le standard depuis systemd.

**Implications** :
- Nouveau crate `crates/nexus-events-core/` dans workspace Cargo.toml
- PyO3 path-dep dans `nexus-core-py` pour exposer le binding
- Update 4 fichiers existants pour wire les events
- Fichier `~/.sbfb/audit.jsonl` cree au premier event

### D3 — C2 @task_handler : decorateur SDK + Pydantic auto-schema

**Retenu** : nouveau decorateur `@task_handler` dans
`packages/nexus-sdk/src/nexus_sdk/decorators.py` qui introspecte la
signature Pydantic du handler pour auto-generer JSON Schemas. Workflow :

```python
from nexus_sdk import task_handler
from pydantic import BaseModel

class TranslateRequest(BaseModel):
    text: str
    target_lang: str

class TranslateResponse(BaseModel):
    translated: str

@task_handler(TranslateRequest, TranslateResponse)
async def translate(self, req: TranslateRequest) -> TranslateResponse:
    ...
```

Le decorateur utilise `TranslateRequest.model_json_schema()` (built-in
Pydantic v2) pour generer les schemas JSON. Les schemas sont stockes
en attributs du handler (`_request_schema`, `_response_schema`).

Nouveau endpoint coordinator `GET /app/<name>/manifest` qui collecte
les schemas de tous les `@task_handler` enregistres et retourne un
manifeste JSON (nom, description, schemas request/response). Cet
endpoint est public (pas de capability gate — le manifeste est
descriptif, pas executif).

Subsume B3 "Pydantic auto-derivation" qui n'avait pas ete livre
separement — le besoin est couvert in-place par `model_json_schema()`.

**Rejete** :
- **Schemas manuels par app** : deja fait pour les 3 tools MCP (D1,
  3 schemas statiques). Pour N apps tierce-partie, la generation
  manuelle ne scale pas. Auto-derivation depuis Pydantic est standard.
- **OpenAPI auto-gen FastAPI** : FastAPI fait deja l'OpenAPI auto pour
  ses propres endpoints. Mais le @task_handler genere des schemas pour
  les apps SBFB (qui ne sont pas des endpoints FastAPI — elles tournent
  dans des iframes ou via le dispatcher). Les schemas MCP-compatibles
  sont JSON Schema pur, pas OpenAPI.
- **TypeScript type generation** : les apps SBFB sont polytech (Python,
  HTML, React, Pyodide, WASM). JSON Schema est le format universel.
  Les apps TS peuvent utiliser `json-schema-to-typescript` cote build.

**Implications** :
- Update `packages/nexus-sdk/src/nexus_sdk/decorators.py` (+ registry)
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/api/manifest.py`
- Tests : decorator introspection + manifest endpoint + round-trip

### D4 — P2 batch : 5 items S25 audit en Phase A

**Retenu** : resoudre en Phase A les 5 P2 identifies par l'audit S25 :

| ID | Fix | LOC estime |
|---|---|---|
| P2-ADMIN-1 | NULL guard `GetSidSubAuthorityCount`/`GetSidSubAuthority` dans `admin_check.py` | ~15 |
| P2-CAPS-1 | Permissions restrictives 0o700 sur `~/.sbfb/` dans `capability_store.py` | ~10 |
| P2-REVOKE-1 | Log warning + reject stale `transition_start` dans `apply_verified()` key_rotation.rs | ~20 |
| P2-HASH-1 | Round-trip test `tomli_w.dumps` determinism dans test_capability_store.py | ~15 |
| P2-STAGE-1 | Validation cles `StageGuardrailMap` contre `GUARDRAIL_STAGES` frozenset dans guardrails.py | ~20 |

Total : ~80 LOC de fix + ~60 LOC de tests.

**Rejete** :
- **Distribuer les P2 dans les phases B-D** : alourdit le scope de
  chaque feature phase. Le pattern P2 batch en Phase A (S25 Phase A
  precedent) est etabli et propre : cleanup d'abord, features ensuite.
- **Defer S27+** : les 5 items sont tous < 20 LOC. Les defer serait
  du gaming du cap G7 (ils ne sont pas carry formels mais items
  d'audit).

### D5 — Scope management : ce que Sprint 26 NE fait PAS

**Retenu** : les items suivants sont differes pour garder le sprint
dans la norme ~2500 LOC :
1. **Tor transport phase 1** → S27 (arti-client toujours pre-1.0)
2. **Arti library-embed** → S27+ (conditionnel arti >= 1.0)
3. **Domain fronting implementation** → S27+ (legal review prereq)
4. **Reliable-workers curator list** → S27
5. **GPU exclusive lockup + no-sharing policy** → S27
6. **A4 process role tagging** → S27 (prereq GPU lockup)
7. **C1 SQLiteSession abstraction** → S27+
8. **C5 streaming bridge** → S27+ (complexe, Playwright matrix)
9. **RAG sanitization** → S27 (B2 prerequis ce sprint, pipeline S27)
10. **Pluggable transports lyrebird** → S27 (couple Tor)
11. **Full 12 events wire A3** → S27 (4 critiques S26, 8 restants S27)
12. **Platform writers journald + oslog** → S27 (JsonFileWriter + ETW S26)

---

## 4.5 Design Review Board findings (G1)

**Report** : `.planning/active/sprint26_design_review.md` (2026-04-22).
**Verdict** : a completer par agent Explore independant.

### Acknowledged review findings

(Sera complete apres le scoring report G1.)

---

## 5. Phase outline

### Phase A — P2 batch cleanup + reclassifications G7

- **Scope** : 5 P2 fixes (ADMIN-1, CAPS-1, REVOKE-1, HASH-1, STAGE-1) +
  reclassification P2-D-1/P2-E-1-iroh → ROADMAP_COMMITMENTS.md LT-5/LT-6 +
  HARDENING_ROADMAP `last_validated` update 2026-04-22 S26 + migration
  sprint25_audit_findings.md → archive/v1.2/
- **Critere** : 5 P2 resolus verts, `cargo nextest run -p nexus-core-rs`
  vert (REVOKE-1), `uv run pytest` vert (ADMIN-1, CAPS-1, HASH-1, STAGE-1)
- **Commit** : `feat(sprint26): Phase A — P2 batch S25 audit (5 fixes) +
  reclassification G7 LT-5/LT-6`

### Phase B — B2 MCP server local-only Streamable HTTP

- **Scope** : `mcp_server.py` (handler JSON-RPC initialize + tools/list +
  tools/call), FastAPI endpoints POST /mcp + GET /mcp (SSE), 3 tools
  whitelist (task_submit, storage_get, storage_set) avec JSON Schema,
  @require_capability("mcp_server_expose") gate, bearer auth + Origin
  validation
- **Critere** : 20+ tests coord (handler unit + integration MCP round-trip
  + capability gate 403 + Origin reject), `uv run pytest` vert
- **Commit** : `feat(sprint26): Phase B — B2 MCP server local-only
  Streamable HTTP 3 tools whitelist`

### Phase C — A3 OS audit SecurityEvent + JsonFileWriter + ETW

- **Scope** : crate `nexus-events-core` (SecurityEvent enum 12 variants +
  EventWriter trait + JsonFileWriter + EtwWriter cfg-gated + stubs
  journald/oslog), PyO3 binding `emit_security_event`, wire 4 events
  critiques (capability_changed, consent_change, token_rotation,
  panic_fired)
- **Critere** : 15+ tests Rust (enum serialize, JsonFileWriter round-trip,
  mock writer, ETW compile-check cfg), `cargo nextest run -p nexus-events-core`
  vert
- **Commit** : `feat(sprint26): Phase C — A3 OS audit SecurityEvent +
  JsonFileWriter + 4 events wired`

### Phase D — C2 @task_handler SDK + Pydantic auto-schema + manifest

- **Scope** : decorateur `@task_handler(RequestModel, ResponseModel)` dans
  `decorators.py` + registry integration, endpoint coordinator
  `GET /app/<name>/manifest`, tests decorator introspection + manifest
- **Critere** : 10+ tests (SDK decorator schema gen + coordinator manifest
  endpoint + round-trip), `uv run pytest packages/nexus-sdk/tests/` +
  `uv run pytest packages/nexus-coordinator/tests/` verts
- **Commit** : `feat(sprint26): Phase D — C2 @task_handler SDK +
  Pydantic auto-schema + manifest endpoint`

### Phase E — wrap-up + verification + audit plan S27

- **Scope** :
  - verification.md (25+ rows fail-fast)
  - audit_plan S27
  - SPRINT_LOG.md + CLAUDE.md updates
  - memory update tip + compteurs
  - migration planning active → archive/v1.2/
- **Critere** : 25+ rows fail-fast verts, docs coherents
- **Commit** : `chore(sprint26): Phase E — wrap-up + verification
  + audit plan S27 + migration planning archive/v1.2/`

---

## 6. Items carry/dette — reclassification S25 → S26

| Item | Source | Phase S26 | Classification |
|---|---|---|---|
| P2-ADMIN-1 Windows MIL null ptr | audit_findings S25 | Phase A | [x] resolve S26 |
| P2-CAPS-1 dir permissions | audit_findings S25 | Phase A | [x] resolve S26 |
| P2-REVOKE-1 RevocationCache overwrite | audit_findings S25 | Phase A | [x] resolve S26 |
| P2-HASH-1 tomli_w determinism | audit_findings S25 | Phase A | [x] resolve S26 |
| P2-STAGE-1 StageGuardrailMap key validation | audit_findings S25 | Phase A | [x] resolve S26 |
| P2-D-1 redundancy persistence | S23 → S24 → S25 → S26 = 4 carry | — | [reclassified] → LT-5 ROADMAP_COMMITMENTS |
| P2-E-1-iroh neighborhood | S23 → S24 → S25 → S26 = 4 carry | — | [reclassified] → LT-6 ROADMAP_COMMITMENTS |
| T-NN+2 iframe Rust-wasm | S22 carry | — | [deferred] PATTERNS §P34 |
| LT-2 Radicle | ROADMAP_COMMITMENTS | — | hors cap (trigger tag v1.0) |
| LT-3/LT-4 | ROADMAP_COMMITMENTS | — | hors-sprint (post-v1.0) |

**Cap G7 bilan** : 0/2 slots carry consommes. Les 5 P2 sont absorbes
en Phase A (items d'audit, pas carries formels). Les 2 re-carry (P2-D-1,
P2-E-1-iroh) sont reclassifies long-term. Sprint net-zero carry.

---

## 7. Scope cuts — ce que Sprint 26 NE fait PAS

1. **Tor transport phase 1** → S27 (arti-client pre-1.0)
2. **Arti library-embed** → S27+ (conditionnel API stable >= 1.0)
3. **Domain fronting implementation** → S27+ (legal review prereq)
4. **Reliable-workers curator list** → S27
5. **GPU exclusive lockup + no-sharing policy** → S27
6. **A4 process role tagging** → S27 (prereq GPU lockup)
7. **C1 SQLiteSession abstraction** → S27+
8. **C5 streaming bridge** → S27+ (Playwright matrix)
9. **RAG sanitization pipeline** → S27 (B2 prerequis ce sprint)
10. **Pluggable transports lyrebird** → S27 (couple Tor)
11. **Full 12 events wire A3** → S27 (4 critiques S26, 8 restants)
12. **Platform writers journald + oslog** → S27

---

## 8. Tracabilite scope — mapping carry S25 → S26

| Item carry S25 | Source | Phase S26 | Status |
|---|---|---|---|
| P2-ADMIN-1 | audit_findings S25 | Phase A | [x] resolve |
| P2-CAPS-1 | audit_findings S25 | Phase A | [x] resolve |
| P2-REVOKE-1 | audit_findings S25 | Phase A | [x] resolve |
| P2-HASH-1 | audit_findings S25 | Phase A | [x] resolve |
| P2-STAGE-1 | audit_findings S25 Phase C review | Phase A | [x] resolve |
| P2-D-1 redundancy | S23 carry → S24 → S25 | — | [reclassified] LT-5 |
| P2-E-1-iroh | S23 carry → S24 → S25 | — | [reclassified] LT-6 |
| Tor transport | HARDENING §3 S25 scope-cut | — | [deferred] → S27 |
| B2 MCP server | HARDENING §3 S25 scope-cut | Phase B | [x] S26 |
| A3 OS audit | HARDENING §3 S25 scope-cut | Phase C | [x] S26 (partial) |
| C2 SDK | HARDENING §3 S25 scope-cut | Phase D | [x] S26 |
| C5 streaming | HARDENING §3 S25 scope-cut | — | [deferred] → S27+ |
| RAG sanitization | HARDENING §3 S25 scope-cut | — | [deferred] → S27 |

---

## 9. Risk register (R1..R5)

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | MCP server expose surface d'attaque locale | Low | Medium | Capability gate OFF par defaut (D5 S25), bearer auth, Origin validation, 3 tools whitelist statique — pas de registration dynamique |
| R2 | SecurityEvent enum evolution post-S26 casse les parsers | Low | Low | JSONL append-only sans schema version (pre-launch, 0 parser externe). Post-v1.0, ajouter `event_version` field |
| R3 | ETW provider registration Windows requiert privileges | Medium | Low | JsonFileWriter comme fallback toujours actif. ETW en best-effort (si registration echoue, fallback silencieux) |
| R4 | @task_handler Pydantic v2 model_json_schema() breaking sur edge cases | Low | Medium | Test round-trip avec 3 types courants (str, int, Optional). Pydantic v2 est stable depuis 2023 |
| R5 | PyO3 binding emit_security_event overhead performance | Low | Low | Fire-and-forget async (tokio::spawn), pas de block sur le hot path |

---

## 10. Audit gate pattern — rappel

- Phase E produira `sprint26_verification.md` + `sprint26_audit_plan.md`
- Sprint 27 Phase 0 jouera l'audit gate en session fraiche
- Convention permanente depuis Sprint 7

---

## 11. Checkpoint de validation

- [x] Audit gate S25 PASS (0 P0/P1)
- [x] G2 trigger check : tous INACTIVE (meme jour que S25 scan)
- [x] G6 memory carry-over : S25 verification §5 deja dans
      nexus_grid_pivot.md tip `3a6f235`
- [x] G7 cap carry-overs : 0/2 slots (5 P2 absorbes Phase A,
      P2-D-1/P2-E-1-iroh reclassifies LT-5/LT-6)
- [x] G7 reclassification : P2-D-1 (4e sprint carry) + P2-E-1-iroh
      (4e sprint carry) → ROADMAP_COMMITMENTS.md
- [x] D1..D5 rediges
- [x] G1 Design Review Board : scoring report a completer
