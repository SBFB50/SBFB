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
   un preflight dans `.planning/active/` avec verdict documente.
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

## 2. Calibration severite

- **P0** : securite active (bypass auth, leak data, code exec) ou
  regression fonctionnelle bloquante.
- **P1** : securite passive (absence de guard documente, surface
  non protegee) ou regression tests (test vert devenu rouge hors env).
- **P2** : hygiene code (missing test edge case, doc inconsistance,
  pattern non-idiomatique, robustesse).
- **P3** : nit (style, naming, commentaire).

---

## 3. Checklist audit gate

- [ ] 0 P0 pour verdict PASS
- [ ] 0 P1 pour verdict PASS (ou fixes commits)
- [ ] >=1 P2 documente pour signal rigor (G4)
- [ ] Chaque finding a un ID, severite, fichier, ligne, description,
      et recommendation
