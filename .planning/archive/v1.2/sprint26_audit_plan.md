# Sprint 26 — Audit plan (pour S27 Phase 0)

**Date** : 2026-04-24
**Tip sortie S26** : `f52dc96` (Phase D)
**Auditeur** : session fraiche S27 Phase 0 (pas la meme session)

---

## 1. Dimensions d'audit

### Track A — P2 batch fixes (Phase A)

1. **ADMIN-1 NULL guard** : verifier que `GetSidSubAuthorityCount`
   et `GetSidSubAuthority` ont des guards NULL dans `admin_check.py`
   (pas seulement un try/except generique).
2. **CAPS-1 permissions** : verifier que `~/.sbfb/` et les fichiers
   sensibles (`capabilities.toml`, `audit.jsonl`) sont crees avec
   permissions restrictives (`0o700` dir, `0o600` files). Verifier
   la coherence `os.makedirs(mode=)` vs `os.chmod()`.
3. **REVOKE-1 stale transition** : verifier dans `key_rotation.rs`
   `apply_verified()` que `transition_start` stale (> MAX_TRANSITION_DAYS)
   est rejectee avec log warning, pas silencieusement acceptee.
4. **HASH-1 determinism** : verifier que le test `tomli_w.dumps`
   round-trip existe et couvre le cas "cle ajoutee puis retiree →
   hash stable".
5. **STAGE-1 validation** : verifier que `StageGuardrailMap` rejette
   les cles hors `GUARDRAIL_STAGES` frozenset avec erreur explicite.

### Track B — MCP server (Phase B)

1. **Spec compliance** : verifier que `/mcp` POST accepte JSON-RPC
   2.0 avec `method` dans `{initialize, tools/list, tools/call}`
   et retourne le format correct (result/error + id).
2. **Security surface** : verifier binding `127.0.0.1` only (pas
   `0.0.0.0`), `@require_capability("mcp_server_expose")` gate,
   bearer auth `X-SBFB-Token`, Origin validation.
3. **3 tools whitelist** : verifier que seuls `task_submit`,
   `storage_get`, `storage_set` sont exposes (pas de registration
   dynamique, pas de `listChanged`).
4. **SDK adoption** : verifier que la dep `mcp>=1.27` est dans
   `pyproject.toml` et que le handler utilise l'API officielle (pas
   un wrapper maison). Cf. PLAN-ADAPT G8 Phase B.
5. **Error handling** : verifier que les tool calls invalides
   retournent des erreurs MCP standard (-32600, -32601, -32602) et
   ne leakent pas de stack traces.

### Track C — OS audit SecurityEvent (Phase C)

1. **Crate structure** : verifier que `nexus-events-core` est un
   crate workspace independant (pas merge dans nexus-core-rs) avec
   ses propres tests.
2. **12 variantes enum** : verifier que `SecurityEvent` a exactement
   12 variantes (pas moins, pas de variante catch-all `Other`).
3. **JsonFileWriter** : verifier JSONL append-only, pas de truncate
   ou overwrite. Verifier que le fichier est cree avec permissions
   restrictives.
4. **EtwWriter** : verifier cfg-gate `target_os = "windows"`, pas
   de compilation conditionnelle fragile. Verifier que le fallback
   JsonFileWriter est toujours actif.
5. **PyO3 binding** : verifier que `emit_security_event` est un
   lazy import dans `capability_store.py` (pas top-level, pour ne
   pas casser le coordinator si le wheel est stale).
6. **4 events wires** : verifier que `capability_changed`,
   `consent_change`, `token_rotation`, `panic_fired` sont
   effectivement emis aux bons endroits (pas de structlog residuel
   qui doublerait l'emission).

### Track D — @task_handler SDK (Phase D)

1. **Decorateur** : verifier que `@task_handler(RequestModel,
   ResponseModel)` enregistre correctement le handler dans le
   registry avec schema JSON genere via `model_json_schema()`.
2. **Registry 5-tuple** : verifier (name, handler, request_model,
   response_model, description) — pas de champs manquants ou
   redondants.
3. **Manifest endpoint** : verifier que `GET /app/<name>/manifest`
   retourne un JSON avec les schemas de tous les handlers
   enregistres, pas seulement le premier.
4. **Pydantic v2** : verifier que `model_json_schema()` est utilise
   (Pydantic v2), pas `schema()` (Pydantic v1 deprecated).
5. **No nexus_core dep** : verifier que le SDK (@task_handler) ne
   depende PAS de `nexus_core` (PyO3) — c'est un package pur Python.

### Track E — Process / meta

1. **G8 preflights** : verifier que les 4 phases A-D ont chacune
   un preflight dans `.planning/archive/v1.2/` avec verdict documente.
2. **Phase reviews** : verifier que les 4 phases ont un review avec
   verdict PASS.
3. **Reclassifications G7** : verifier LT-5 et LT-6 dans
   `ROADMAP_COMMITMENTS.md` avec conditions de sortie correctes.
4. **HARDENING_ROADMAP** : verifier `last_validated` = 2026-04-22
   et que la note S26 "arbitrage" est correcte.
5. **Commit discipline** : verifier que chaque phase a un commit
   atomique `feat(sprint26): Phase X — ...` avec body riche
   incluant delta tests et scope cuts.
6. **docs(process) commits** : 3 commits `docs(process)` post-Phase D
   (`a92e0d4`, `e81559c`, `a3e35b2`) — verifier qu'ils sont
   coherents et ne touchent pas de code fonctionnel.

---

## 2. Findings des phase reviews (§4.4 routing)

Phase reviews presentes : **4/4** (A, B, C, D — ratio complet).

### Track A findings (sprint26_phase_A_review.md)

- **P2-PLAN-LOC** : plan.md §6 contient des estimations LOC
  prospectives par phase (~140, ~600, ~500, ~300, ~100). Contraire
  a feedback_approach.md §6. Pre-existant depuis redaction du plan.
  → **Track E process** (transverse, recurrent Phases A/B/D)
- **P2-HANDLER-COVERAGE** : `key_rotation_handler.rs` branche Err
  du match `apply_verified` (4 LOC logging) pas directement testee.
  Logique sous-jacente `apply_verified` couverte par 2 tests Rust.
  Risque faible. → **Track A item 3 (REVOKE-1)**

### Track B findings (sprint26_phase_B_review.md)

- **P2-LIFESPAN-AENTER** : `api/app.py` lifespan utilise
  `__aenter__`/`__aexit__` explicites au lieu d'`async with` pour
  le MCP session manager. Necessaire car le context manager doit
  span le `yield` du lifespan FastAPI. Fragile si le SDK mcp change
  l'implementation de `.run()`.
  → **Track B item 4 (SDK adoption)** — verifier fragilite
- **P3-TEST-DELTA** : 12 tests vs ~20 estimes plan. Delta explique
  par PLAN-ADAPT (SDK elimine tests framing/dispatch/error codes).
  → informatif, pas d'action

### Track C findings (sprint26_phase_C_review.md)

- **P2-C-1** : `_emit_capability_event` catch `(ImportError,
  Exception)` sans logging. Devrait logger a debug level. Low risk
  (audit path fire-and-forget). Carry S27.
  → **Track C item 5 (PyO3 binding)** — verifier logging
- **P2-C-2** : `JsonFileWriter` sans rotation de logs. Fichier
  JSONL croit sans limite. Acceptable pre-launch.
  → **Track C item 3 (JsonFileWriter)** — verifier politique rotation
- **P2-C-3** : plan disait `tracing-etw = "0.2"` comme dep directe.
  Implementation utilise `tracing::info!` target-based (pas de dep
  tracing-etw). Architecturalement plus propre. Divergence plan
  documentee dans preflight.
  → **Track C item 4 (EtwWriter)** — verifier coherence approche

### Track D findings (sprint26_phase_D_review.md)

- **P2-LOC-PHASE-D** : meme P2-PLAN-LOC que Phases A/B (LOC
  estimees au plan). Pre-existant, pas introduit par Phase D.
  → **Track E process** (fusionne avec P2-PLAN-LOC)
- **P2-PLAYWRIGHT-NORUN** : tests Playwright (27) non executes
  Phase D. Phase D pure Python, 0 fichier frontend modifie. Risque
  regression cross-stack nul. Executes au Phase E verification.md.
  → **[closed Phase E]** — Playwright execute Phase E, 27 pass

---

## 3. Items connus (carry-over S26)

| ID | Severite | Description | Source |
|---|---|---|---|
| T-NN+2 | P3 | iframe Rust-wasm (PATTERNS §P34, triggers inactive) | S22 carry |
| LT-5 | LT | Redundancy persistence (ex-P2-D-1, reclassifie S26 Phase A) | S23→S26, ROADMAP_COMMITMENTS |
| LT-6 | LT | iroh neighborhood enrichment (ex-P2-E-1-iroh, reclassifie S26 Phase A) | S23→S26, ROADMAP_COMMITMENTS |

---

## 4. Zones a risque (attention supplementaire S27 Phase 0)

1. **MCP lifespan async-with** (P2-LIFESPAN-AENTER) :
   `api/app.py` lifespan utilise `__aenter__`/`__aexit__` explicites.
   Si le SDK `mcp` v1.27+ change l'implementation de
   `StreamableHTTPSessionManager`, le lifespan cassera silencieusement.
   Verifier que le pattern est documente ou wrape.

2. **JsonFileWriter croissance illimitee** (P2-C-2) :
   `~/.sbfb/audit.jsonl` est append-only sans rotation. Pre-launch
   c'est acceptable (0 node externe). Post-v1.0, un node actif
   genererait un fichier croissant indefiniment. Carry S27 si
   observabilite production devient prioritaire.

3. **emit_capability_event catch silencieux** (P2-C-1) :
   `capability_store.py:83` catch `(ImportError, Exception)` sans
   aucun logging. Si le binding PyO3 echoue en production, l'event
   audit est perdu silencieusement. Devrait au minimum logger a
   debug level.

4. **Coord pass count ambiguite** : les phase reviews montrent des
   compteurs coord cumulatifs qui ne s'additionnent pas lineairement
   (376→394→406→377 pass a la sortie). Le recomptage entre phases
   (certains tests passant de stale a pass ou inversement selon
   l'etat du wheel PyO3) rend le delta coord peu fiable. Verifier
   le compteur reel en relancant `uv run pytest` a la session S27.

---

## 5. G8 retrospective

Sixieme sprint consecutif (S21-S26) avec G8 systematique toutes
phases. 25 preflights cumules :
- 22 EXECUTE
- 3 SCOPE-CUT-CONSISTENT (S21 B/C/E, S23 E)
- 1 PLAN-ADAPT (S26 Phase B — adoption SDK mcp v1.27 officiel)
- 0 DESIGN-CONFLICT

Premier PLAN-ADAPT effectif : Phase B a adopte le SDK officiel
`mcp` v1.27 (PyPI) au lieu d'une implementation maison JSON-RPC.
Le scan S1a OSS prior art a identifie le SDK comme meilleure
approche. Le code a suivi l'approche corrigee, pas le plan original.

Maturite confirmee : G1 Design Review Board pre-gel est suffisant
depuis S21 pour eliminer les conflits en amont.

---

## 6. Calibration severite

- **P0** : securite active (bypass auth, leak data, code exec) ou
  regression fonctionnelle bloquante.
- **P1** : securite passive (absence de guard documente, surface
  non protegee) ou regression tests (test vert devenu rouge hors env).
- **P2** : hygiene code (missing test edge case, doc inconsistance,
  pattern non-idiomatique, robustesse).
- **P3** : nit (style, naming, commentaire).

---

## 7. Checklist audit gate

- [ ] 0 P0 pour verdict PASS
- [ ] 0 P1 pour verdict PASS (ou fixes commits)
- [ ] >=1 P2 documente pour signal rigor (G4)
- [ ] Chaque finding a un ID, severite, fichier, ligne, description,
      et recommendation
