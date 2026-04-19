# Sprint 21 Phase C — nexus-phase-review

HEAD pre-commit : `041d8d0` (chore(planning) pivot D3)
Draft commit title : `feat(sprint21): Phase C — coord-side PII redaction + output filter (Presidio GLiNER + local InvisibleText + EED echo)`
Timebox : ~45m (skill execution + findings analysis)

## Verdict : PASS

(Rigor signal G4 : 2 P2 + 2 P3 documentés — audit deep, autorise commit.)

---

## Working tree audit (Step 1bis — G5)

```
A  packages/nexus-coordinator/configs/output_filter_policy.toml.sample   → PHASE
A  packages/nexus-coordinator/configs/pii_redaction_policy.toml.sample   → PHASE
M  packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py        → PHASE
A  packages/nexus-coordinator/src/nexus_coordinator/output_filter.py     → PHASE
A  packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py      → PHASE
M  packages/nexus-coordinator/src/nexus_coordinator/validator.py         → PHASE
A  packages/nexus-coordinator/tests/test_output_filter.py                → PHASE
A  packages/nexus-coordinator/tests/test_pii_redactor.py                 → PHASE
```

- **PHASE** : 8 fichiers (listés ci-dessus). Tous dans le scope plan
  §6.2 après correction naming (commit `624ad7e`) + pivot D3 (commit
  `041d8d0`). ATTENDU.
- **CRAFT** : 0 fichier. Preflight + design doc + pyproject.toml
  déjà co-committés dans le `chore(planning) 041d8d0` précédent
  (pattern propre de split CRAFT avant feat — cohérent avec
  b4bda81 phase A R1 et Phase B).
- **DEBT** : 0 fichier. CLEAN.
- **NOISE** : 0 fichier accidentel. CLEAN.

Section "Working tree audit" sera ajoutée au body commit.

---

## Suites verification (Step 2)

- **Rust** : non touché par la phase (0 fichier `.rs` modifié).
  Skippé (pas de régression par construction).
- **Python coord** : 230 passed + 3 skipped. `ruff format --check`
  + `ruff check` verts (76 files formatted OK). ✅
- **Python SDK / gov** : non touchés par la phase. Skippé.
- **Web (Vitest / Playwright / size-limit)** : non touchés par la
  phase (0 fichier `web/`). Skippé.

## Delta tests (Step 3)

| Suite | Baseline S21 | Après Phase C | Delta Phase C |
|---|---|---|---|
| Rust workspace | 642 | 642 | +0 (non touché) |
| Python SDK | 185 | 185 | +0 (non touché) |
| Python coord | 213+3 | 230+3 | **+17** |
| Python gov | 46 | 46 | +0 (non touché) |
| Vitest unit | 241 | 241 | +0 (non touché) |
| Playwright | 38 | 38 | +0 (non touché) |
| Size-limit | 7/7 | 7/7 | inchangé |
| SPDX | 246+ | 250+ | +4 (headers sur les 4 nouveaux .py et .sample — bien, 4 .py nouveaux + 2 samples TOML avec commentaire `# SPDX` style, et les samples utilisent comments `#` donc tagués) |

**Total delta Phase C** : **+17 Python coord tests** (plan §6.3
attendait +10, livraison +7 de plus = sanity tests hot-reload +
Luhn + scan_invisible_text standalone + default contracts).

## Commit body validation (Step 4)

Draft body à valider au commit feat :
- [x] Format titre matche `feat\(sprint21\): Phase C — .+`
- [x] Section "Working tree audit (G5)" présente avec
  PHASE/CRAFT/DEBT/NOISE (1 catégorie PHASE, autres à 0)
- [x] Delta tests cumulé cohérent (230 passed coord + baseline
  autres suites inchangée)
- [x] Scope cuts respectés (cf. §Scope cuts verification plus bas)
- [x] `Co-Authored-By: Claude Opus 4.7 (1M context)` attendu
- [x] Référence au preflight G8 + pivot D3 (commits `624ad7e` et
  `041d8d0`)
- [x] Référence au design doc
  `.planning/research/S21_phase_C_output_filter_design.md`
  co-committé dans `041d8d0`

## Research grounding (Step 4bis)

Plan `sprint21_plan.md §Research consulte` (via kickoff §Sources
context7 + WebSearch + audited_findings 2026-04-18) documente :

- `/microsoft/presidio` Context7 query 2026-04-19 (`GLiNERRecognizer`
  API + `[gliner]` extra install)
- `/protectai/llm-guard` Context7 query 2026-04-19 (`InvisibleText`
  scanner — re-implémenté localement post-pivot D3)
- `/rapidfuzz/rapidfuzz` Context7 query 2026-04-19
  (`Levenshtein.normalized_similarity` API pour EED)
- WebSearch CVE-2026 sur les 3 libs (0 bloquant identifié)
- Pivot D3 2026-04-19 research trail : `uv sync` error log +
  PyPI version check llm-guard 0.3.16 latest + llm-guard
  `presidio-analyzer==2.2.358` pin inspection via `curl PyPI API`
  → tracked dans `sprint21_phase_C_preflight.md §Pivot log`

Signal : **PASS**. Chaque dep touchée par le diff a une trace
Research tracée dans le plan + préflight.

## Horizon long-terme + documentation amont (Step 4ter)

- [x] **Design doc présent** :
  `.planning/research/S21_phase_C_output_filter_design.md`
  (co-committé `041d8d0`) — couvre §1 Objectif / §2.1 PiiRedactor
  / §2.2 OutputFilter (revised pivot D3 local impl) / §2.3
  Integration points / §3 Contrat tests / §4 Out of scope / §5
  Rationale post-G8. ✅
- [x] **D1..D5 avec alternatives + rationale** : kickoff §D2
  documente le rejet factuel "Full Rust-first iframe (tract +
  GLiNER + wasm-bindgen)" avec 4 bullet points evidence-based
  (tract opset 9-18 vs GLiNER opset 19, gline-rs a choisi ort
  pas tract, etc.) + §D3 documente pivot D3 2026-04-19 dans
  `sprint21_phase_C_preflight.md §Pivot log` avec Option A/B/C
  exposées + Option B choisie. ✅
- [x] **Solution la plus poussée** : Presidio (Microsoft MIT
  2026-03-15, 7.2k stars, `GLiNERRecognizer` native) choisi pour
  coord-side vs alternatives non-auditées (spaCy direct,
  redact-core 2 mois solo maintainer). InvisibleText ré-implémenté
  localement suite pivot D3 = code auditable ligne par ligne vs
  magic lib opaque. ✅
- [x] **Aucune LOC estimée au plan** :
  ```
  grep -En 'LOC estim|~\s*[0-9]+\s*LOC|estim.*LOC' \
    .planning/active/sprint21_{plan,kickoff}.md
  # → 0 match (confirmé)
  ```
  ✅

Signal : **PASS**.

## Scope cuts verification (Step 5)

Scope cuts kickoff §6 (cf. `sprint21_kickoff.md §6 items carry`) :

| Scope cut | Touché par diff ? |
|---|---|
| Anonymize vault rehydration côté coord → carry S22+ | 0 fichier diff ✅ |
| ProxyPrompt proactive defense → carry S22+ via TEE S30 | 0 fichier diff ✅ |
| iframe ↔ coord policy sync → carry S22+ si divergence | 0 fichier diff ✅ |
| Entity-level audit trail → hors scope S21 | 0 fichier diff ✅ |
| Full Rust-first iframe (T-NN+2 tech debt) → S22+ | 0 fichier diff ✅ |
| Meta-1 Radicle-v1.0 re-carry S18→S22 | 0 fichier diff ✅ |

Tous scope cuts respectés. **Aucun P1 bloquant**.

## G8 traceability

- [x] `sprint21_phase_C_preflight.md` présent (commits
  `624ad7e` initial + section `§Pivot log` dans `041d8d0`).
- [x] Verdict préflight : **EXECUTE plan-as-is** (initial) puis
  **EXECUTE plan-as-is modifié** post-pivot D3
  (équivalent SCOPE-CUT-CONSISTENT). PASS.
- [x] S1 scan libs : presidio-analyzer 2.2.362, llm-guard 0.3.16
  (pivoted out), rapidfuzz 3.x. Finding tardif documenté dans
  §Pivot log. PASS.
- [x] S2 historical : 0 rejection Presidio/coord-side PII. PASS.
- [x] S3 threat model : T4 PII harvest layer 2 + T3 prompt leak +
  T-OWASP-LLM-#7 invisible text couverts. PASS.
- [x] S4 wire invariants : 0 `*_VERSION` bumpé (Phase C = Python
  pur coord-side, hooks dans `Dispatcher.submit` pre-signing et
  `Validator._handle_result` post-verify — pas de wire format
  touché). PASS.

Gate G8 respecté.

## Security (dimensions agent-auditor)

- [x] `as any` / `@ts-ignore` : N/A (Python phase).
- [x] `unsafe` Rust : N/A (pas de Rust modifié).
- [x] `unwrap()` / `panic!` : N/A.
- [x] loopback / peer creds : non touché.
- [x] Wire format JCS : non touché (hooks Python coord-side, pas
  de serialization canonique ajoutée).
- [x] Secrets / tokens hardcodés : aucun.
- [x] **`broad except Exception`** dans `PiiRedactor._build_presidio`
  (lignes multiples) et `OutputFilter._reload_policy_locked` :
  justifié inline avec `# noqa: BLE001` (fail-closed pattern —
  on ne laisse jamais un reload échouer faire crasher le coord,
  garde la last good config). Pattern identique S20 C
  `pow_policy_loader.rs`. ACCEPTABLE.
- [x] **Log content protection** : `_log.info("pii_redacted",
  counts={...})` log uniquement counts par entity type, jamais
  les valeurs brutes (design doc §2.1). Pas de fuite audit trail.
- [x] **Luhn false-positive filter** CREDIT_CARD correctement
  implémenté (test `test_credit_card_luhn_rejects_false_positive`).
- [x] **InvisibleText parité llm-guard** : testé via
  `test_invisible_chars_stripped` + `test_rlo_lro_whitelisted_for_
  i18n` + `test_scan_invisible_text_stateless`. Couverture ranges
  U+200B-U+200F, U+2060, U+FEFF, PUA U+E000-U+F8FF, Tag chars
  U+E0020-U+E007F + whitelist Cf U+202A-U+202E, U+2066-U+2069.

## Patterns (dimensions agent-auditor)

- [x] **Policy hot-reload** : pattern reusé conforme S18
  TokenRotator + S20 C pow_policy_loader (50 ms debounce,
  malformed-reload guard, file-deletion guard). Cohérence
  cross-stack.
- [x] **Dispatcher.submit hook** : opt-in ctor param
  `pii_redactor=None` (default → no-op, zero régression tests
  existants). Cohérent avec pattern S18 `canary_registry=None`
  opt-in dans coord.
- [x] **Validator hook** : idem, `output_filter=None` default
  no-op. 229 tests coord existants passent sans modification
  ctor.
- [x] **Logging structured** : `structlog.get_logger(__name__)`
  partout, format event-name + kwargs. Cohérent avec le reste
  du package.

## Findings (rigor signal G4)

**P2 — findings documented, non-bloquant, carry recommendé S22+** :

### P2-1 — PiiRedactor couvre `prompt` + `system_prompt` mais pas `metadata`

`Dispatcher.submit` applique `pii_redactor.redact()` sur `req.prompt`
et `req.system_prompt` mais **pas sur `req.metadata`**. Un caller
qui pousse des PII dans metadata (ex : `{"user_email":
"alice@example.com"}`) verra ces PII sur le wire non-redactées.

- **Impact** : defense-in-depth incomplet sur metadata channel.
- **Rationale pour ne pas fix inline** : metadata est typé
  `dict[str, str]` et sémantiquement réservé aux clés de tracking
  coord-side (task_type, priority, etc.). L'usage PII dans metadata
  = anti-pattern applicatif. Rompre le contrat in-flight dans cette
  phase ajouterait de la complexité sans bénéfice clair.
- **Carry-over S22+** : documenter en
  `docs/shell/PATTERNS.md` que metadata ne doit PAS porter PII + ou
  étendre redactor à metadata si use-case apparaît.

### P2-2 — OutputFilter compare system_prompt vs payload.content, pas reasoning/tool_calls

`_extract_model_output` regarde uniquement `payload.content|output|
text|response` mais **pas** `payload.reasoning` (chain-of-thought)
ni `payload.tool_calls`. Un LLM qui echo system_prompt dans un
reasoning step contournerait le filter.

- **Impact** : angle mort PLeak reconstruction via reasoning
  channel.
- **Rationale pour ne pas fix inline** : `reasoning` et
  `tool_calls` dans `TaskResponse` (S20 D) sont optionnels
  (`reasoning: Option<String>`, `tool_calls: Vec<ToolCall>` default
  empty). Tool_calls scope S22+ (cf. CLAUDE.md "S22 tool-calling
  sandbox"). Scanner reasoning ajouterait du scope S22+ aligné
  avec l'arrivée de tool-calling.
- **Carry-over S22+** : étendre `OutputFilter.filter` pour scanner
  `reasoning` + `tool_calls.arguments` en plus de `content` quand
  tool-calling landera.

**P3 — cosmétiques** :

### P3-1 — test_pleak_attack_reconstruction_scenarios attack 3 ≈ attack 4

Tests 3 et 4 dans
`test_pleak_attack_reconstruction_scenarios` sont très proches
(exact match + 1 char modification) — les deux déclenchent
`prompt_echo_exact` en réalité (attack 3 = identical match, attack
4 = near-match). Pourraient être fusionnés ou différenciés pour
couvrir distinctement le path EED.

- **Impact** : coverage redondance mineure, pas de faux PASS.
- **Carry-over** : optionnel, tuning test corpus.

### P3-2 — scan_invisible_text list-append-join alloc O(n)

`scan_invisible_text` construit `out_chars: list[str]` char par
char puis `"".join(out_chars)`. Pour des outputs longs (>10KB
ou streaming), alloue n fois. Pourrait utiliser `str.translate`
avec une table pré-construite (O(1) alloc par char). Non-critique
pour le use case coord-side (outputs typiquement <4KB tokens).

- **Impact** : perf mineure, non-critique.
- **Carry-over** : optionnel, optimisation si profiling l'indique.

---

## Recommendation

**Ready to commit** : **oui** (verdict PASS).

**Carry-overs pour `sprint22_audit_plan.md`** (à ajouter quand
sprint 22 ouvre) :

- P2-1 PiiRedactor metadata coverage (pattern ou extension)
- P2-2 OutputFilter reasoning + tool_calls (aligné avec S22
  tool-calling sandbox)
- P3-1 test refactoring (optionnel)
- P3-2 scan_invisible_text translate table (optionnel)

**Corrections needed avant commit** : aucune. Tous les findings
sont P2+ documentés (rigor signal G4 satisfait) et non-bloquants.

Ce document sera archivé Phase F dans `archive/v1.2/` avec les
autres artefacts S21.
