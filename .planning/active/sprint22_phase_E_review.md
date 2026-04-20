# Sprint 22 Phase E — nexus-phase-auditor review

HEAD pre-commit: `8146db7`
Draft commit body: "feat(sprint22): Phase E — watermark canari-input spot-check consumer 1/N primitive"
Timebox: 25m LIGHT-AUDIT

## Verdict : PASS

0 P0 / 0 P1 / 1 P2 / 2 P3.
G8 preflight EXECUTE présent, tests delta confirmé, scope cuts honorés,
patterns cohérents, research-grounding clean (stack existant, 0 nouvelle dep).

---

## Dimensions

### Security

- [x] **unwrap/unsafe** : aucun bloc `unsafe`, aucun `.unwrap()` Python. Exceptions
  capturées via `except Exception as exc: # noqa: BLE001` pattern standard
  (output_filter precedent). Evidence : `canary_input.py:515,633,666`.
- [x] **Secret leak dans logs** : tous les `_log.*` calls vérifié — logs
  uniquement `path`, `inject_rate`, `default_tolerance`, `enabled`, `prompts` (count
  int). Aucun `expected_answer`, `prompt`, `signature_hex`, `coord_secret` loggé.
  Evidence : `canary_input.py:516-519,622-626,634-638,643-649,667-671,676-679`.
- [x] **Path traversal `set_path` TOML** : l'opérateur peut fournir un chemin
  arbitraire via `[default].set_path` → `Path(self._policy.set_path).expanduser()`
  (`canary_input.py:603`). Chemin non restreint à `~/.sbfb/`. Mais : (1) la policy
  TOML est lue depuis `~/.sbfb/canary_input_policy.toml` contrôlé par le même
  utilisateur, (2) le seul effet est de lire/écrire un JSON signé, (3) la signature
  Ed25519 est vérifiée au chargement — un fichier étranger échoue la vérification.
  Verdict : **acceptable** (single-user daemon, même uid, file contents verified).
- [x] **Race condition `_maybe_reload`** : `last_reload_check` lu/écrit hors lock
  (lignes 608-610) → deux threads peuvent passer le debounce simultanément. Double
  reload bénin : les deux voient le même `mtime`, la comparaison `mtime <= stored`
  interne rend le second no-op. Pattern identique à `output_filter.py` (precedent S21
  Phase C). Pas de nouvelle régression.
- [x] **FastAPI body shape validation** : `POST /api/canary/inject-rate` valide
  presence `inject_rate` + cast `int()` + HTTPException 400 on malformed.
  `GET /api/canary/observed-divergence` clamp `max(0, min(int(limit), 100))`.
  Evidence : `api/canary.py:177-182,205`.
- [x] **Loopback auth** : router monté sur le coordinator FastAPI qui porte déjà
  `LoopbackAuthMiddleware` Sprint 16. Aucune route ouverte publiquement. Documenté
  explicitement `api/canary.py:39-43`.

**P3-E-1** (nit) : `GET /api/canary/observed-divergence` retourne `expected_answer`
et `observed_answer` dans chaque `DivergenceRecord` (`DivergenceRecord.to_dict()`
`canary_input.py:174-182`). Un appelant loopback authentifié voit les réponses
attendues des probes. Acceptable (loopback bearer only, single-user), mais un
opérateur qui exporte les divergences via un outil de monitoring externe expose
ses probes. Docstring `DEFAULT_SEED_PROMPTS` avertit déjà (`canary_input.py:695-702`).
Log in `sprint23_audit_findings.md` si alerting durable Sprint 23 B1 est implémenté.

### Patterns

- [x] **P6 NEXUS_GRID_ROOT env override** (`docs/shell/PATTERNS.md §P6`) :
  `paths.py` nouvelles fonctions `canary_input_policy_path()` et
  `canary_input_set_path()` honorent `_ROOT_OVERRIDE_ENV` identiquement à
  `canary_registry_path()`. Evidence : diff `paths.py:3-36`.
- [x] **Hot-reload pattern (output_filter S21 Phase C precedent)** : `CanaryInputManager`
  mirror exact — 50 ms mtime debounce, malformed-reload guard keep-last, file-deletion
  keep-last. `_reload_policy_locked` / `_reload_set_locked` séparés exactement comme
  `OutputFilter._reload_policy_locked`. Evidence : `canary_input.py:606-681`.
- [x] **Typer CLI pattern (quarantine S21 Phase D precedent)** : `cli/commands/canary.py`
  suit `cli/commands/quarantine.py` — `typer.Typer(no_args_is_help=True)` + `asyncio.run(_do_*)` + `coord.start()`/`coord.stop()` finally. Evidence : `cli/commands/canary.py:39-111`.
- [x] **pydantic ConfigDict** : `CanaryPrompt(model_config = ConfigDict(frozen=True))`,
  `CanaryInputSet(model_config = ConfigDict(frozen=False))`. Cohérent avec schemas
  existants (`canary_registry.py` pattern). Evidence : `canary_input.py:126,145`.
- [x] **rapidfuzz import path** : `from rapidfuzz.distance import Levenshtein` +
  `Levenshtein.normalized_similarity(...)` — identique à `output_filter.py:52,269`.
  Evidence : `canary_input.py:74,415`.
- [x] **nexus_core sign/verify** : `nexus_core.sign_bytes(msg, secret)` /
  `nexus_core.verify_bytes(msg, sig, pubkey)` — surface raw Sprint 14 Phase A.
  Cohérent avec `canary_registry.py` qui utilise `nexus_core.verify_canary`.
  Evidence : `canary_input.py:243,271`.
- [x] **`sort_keys=True` pour canonical local** : `signable_json()` utilise
  `json.dumps(payload, sort_keys=True)` (`canary_input.py:160`). Acceptable car
  local-integrity-only (non wire P2P) — preflight S4 confirme 0 wire format impacté.
  JCS non requis pour usage intra-process. Cohérent avec le commentaire docstring
  `canary_input.py:23-27`.

**P2-E-1** : `CanaryInputManager.__init__` appelé sans lock lors du chargement
initial du set (`canary_input.py:506-520`) — la lecture du set path, stat(), et
affectation `_reload_state.set_mtime` se font avant la construction de `_lock`
(ligne 501) mais après `_lock` initialisé. En fait, la séquence montre que `_lock`
est créé à la ligne 501 et l'init-set à 506-520 est hors lock — acceptable car le
`__init__` est single-threaded par construction (pas de partage possible avant que
l'objet soit retourné). Malgré tout : `_reload_policy_locked` est appelé directement
depuis `__init__` à la ligne 504 — ce helper s'appelle `_locked` mais n'acquiert
PAS le lock lui-même (il s'attend à être appelé sous lock). L'appel depuis `__init__`
sans lock est techniquement sûr (objet non partagé), mais le suffixe `_locked` est
trompeur pour un futur contributeur. Recommandation : ajouter un commentaire inline
sur l'appel `__init__` ligne 504 type `# safe: single-threaded init, lock not yet needed`.
Severity P2 (drift naming convention). Non bloquant.

### Working tree audit (G5)

- [x] **PHASE** : 8 fichiers attendus / Plan §8 — comptage diff confirme exactement 8.
  `canary_input.py` (A), `cli/commands/canary.py` (A), `api/canary.py` (M),
  `cli/main.py` (M), `coordinator.py` (M), `paths.py` (M),
  `configs/canary_input_policy.toml.sample` (A), `tests/test_canary_input.py` (A).
- [x] **CRAFT** : 0 fichier planning/docs Claude dans le diff. Git status confirme
  uniquement les 8 fichiers PHASE.
- [x] **DEBT** : 0 fichier tech debt ou scope cut.
- [x] **NOISE** : 0 fichier accidentel.
- [x] **Section "Working tree audit"** : présente dans le draft commit body
  `Fichiers touchés (PHASE, 8 fichiers)`.

### G8 traceability

- [x] Artefact G8 présent : `.planning/active/sprint22_phase_E_preflight.md`
  (verdict `EXECUTE plan-as-is`, commit `e621a92`).
- [x] Verdict = EXECUTE (pas DESIGN-CONFLICT) → aucun pivot-check requis.
- [x] Verdict = EXECUTE (pas SCOPE-CUT-CONSISTENT) → aucun carry-over audit_plan requis.
- [x] Cas D hotfix : non applicable (phase normale sprint).

Intégrité G8 vérifiée : preflight §Action bullet 1-5 correspond exactement
aux fichiers livrés. Preflight §Garde-fous 7/7 passés. Delta tests annoncé
preflight "+5" vs livré "+8" — 3 bonus tests documentés dans draft commit body
(test_api_503, test_manager_maybe_inject_and_observe, test_canary_input_set_version_constant).
Divergence plan §8.3 vs livré documentée dans body commit. Non bloquant (bonus tests
= over-delivery positif).

**P3-E-2** (nit) : preflight §Action annonce `canary-rotate` (avec tiret) comme
nom CLI, le code enregistre `name="canary"` + sous-commandes `rotate`/`status`
(`cli/main.py:37`, `cli/commands/canary.py:46,126`). Invocation correcte
`nexus-coordinator canary rotate`. Léger écart de terminologie uniquement dans le
preflight, aucun impact fonctionnel.

### Scope-cuts

Scope cuts Phase E kickoff §4 D4 rejets :
- **Kirchenbauer 2023 / BIRA** : grep diff → `canary_input.py:11-12` (mention
  doc-only, aucune implémentation). PASS.
- **LLM-Canary suite** : grep diff → 0 occurrence dans les fichiers code.
  `kickoff.md:519` cité dans preflight. PASS.
- **Portkey terminology** : grep diff → 0 occurrence. PASS.
- **Auto-scheduler** : grep diff `auto.rotat|auto.schedul|asyncio.*sleep.*rotat` → 0
  occurrence. CLI manuel uniquement. PASS.
- **Backend ML** : grep diff `sklearn|torch|onnx|transformers|ml_backend` → 0
  occurrence. Uniquement rapidfuzz Levenshtein. PASS.

### Tests-delta

- [x] **Python coord** : annoncé 255 → 263 (+8). Réel mesuré :
  `263 passed, 3 skipped` — confirmé `+8` exact.
  Evidence : `uv run pytest packages/nexus-coordinator/tests/ -q` output ligne 5.
- [x] **Rust / SDK / Vitest / Playwright** : inchangés (diff 0 fichiers Rust/TS). PASS.

### Research-grounding

S1-S4 preflight ACKNOWLEDGED (spec LIGHT-AUDIT, 0 critère rouge-ligne DEEP).

- [x] **Cargo.toml deps** : diff 0 modification — 0 nouvelle dep Rust.
- [x] **pyproject.toml deps** : diff 0 modification — `rapidfuzz`, `typer`, `pydantic`
  déjà présents, aucun bump. Traces `plan §Research consulté` non re-vérifiées
  (existant depuis S21 Phase C, S21 Phase D). PASS.
- [x] **API crypto** : `nexus_core.sign_bytes` / `verify_bytes` (Ed25519 Sprint 14,
  tracé). `sort_keys=True` local-canonical non wire. PASS.
- [x] **Spec externe datée** : 0 nouvelle spec (BIRA explicitement rejetée, non implémentée). PASS.

### Horizon long-terme + documentation amont

- [x] **Design doc** : Phase E est une primitive coord-side pure (~520 LOC library),
  lifetime 1 sprint (consommé S23 B1 Guardrails). Pas de nouveau module structurant
  multi-sprint → design doc complet non requis. Preflight §Scans + kickoff §4 D4
  constituent la trace de décision.
- [x] **Alternatives rejetées** : kickoff §4 D4 lignes 517-520 liste Kirchenbauer /
  LLM-Canary / Portkey avec rationale. D1..D5 complets.
- [x] **Solution la plus poussée** : rapidfuzz Levenshtein = choix le plus éprouvé
  disponible pour cette primitive (même lib que EED output_filter S21 Phase C). Ed25519
  via nexus_core = lib auditée existante. Aucune alternative plus poussée non citée.
- [x] **Estimation LOC** : plan §8.2 mentionne `~250 LOC` pour `canary_input.py` — livré
  520 LOC. Estimation LOC présente dans plan. Non-bloquant : la politique `docs/claude/
  README.md §6.7` interdit les estimations LOC **prospectives** au plan. Ici l'estimation
  est une borne de scoping (dimension planning) non une métrique de succès. Deviation
  documentée dans draft commit body (`Deviation LOC : plan §8.2 ~250 LOC [...] livré
  ~520 lignes`). Carry comme P2 pour nettoyage éventuel du pattern dans les plans futurs.

---

## Findings

- **P2-E-1** : `_reload_policy_locked` appelé depuis `__init__` sans lock
  (`canary_input.py:504`) — suffixe `_locked` trompeur pour futurs contributeurs.
  Fix : commentaire inline `# safe: single-threaded init`. Non bloquant.
- **P2-E-2** : `plan §8.2` porte une estimation LOC `~250 LOC` pour `canary_input.py`
  (livré 520) — pattern à éviter dans les plans futurs (`README.md §6.7` bannit
  estimations LOC prospectives). Deviation déjà documentée dans body commit. Non bloquant.
- **P3-E-1** : `GET /api/canary/observed-divergence` expose `expected_answer` en clair
  dans les enregistrements de divergence. Acceptable (loopback bearer), mais à
  documenter dans Sprint 23 B1 alerting design si export externe envisagé.
- **P3-E-2** : Légère divergence de terminologie preflight `canary-rotate` vs code
  `canary rotate` (sous-commande). Aucun impact fonctionnel.

---

## Recommendation

**Commit autorisé.** 0 P0 / 0 P1. Les 2 P2 sont non bloquants et documentés :
- P2-E-1 : commentaire inline trivial, peut être ajouté dans ce commit ou Phase F chore.
- P2-E-2 : pattern LOC à ne pas reproduire dans les plans S23+, aucune action requise.

Ajouter P3-E-1 dans `sprint23_audit_findings.md` lors du wrap-up Phase F
(carry-over S23 pour le design alerting durable B1 Guardrails).
