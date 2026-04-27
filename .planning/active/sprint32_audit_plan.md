# Sprint 31 — Audit plan (pour S32 Phase 0)

**Date** : 2026-04-27
**Tip sortie S31** : sera le commit Phase E (post-migration)
**Auditeur** : session fraiche S32 Phase 0 (pas la meme session)

---

## 1. Mode d'emploi pour la session fraiche

### Ordre de lecture impose

1. `.planning/archive/v1.2/sprint31_kickoff.md` — D1..D5 gelees
2. `.planning/archive/v1.2/sprint31_plan.md` — fail-fast checklist + phases A-E
3. Ce document (audit_plan) — tracks + methodes
4. Le code (via grep/read/explore)

### Fichiers a NE PAS lire avant d'avoir forme une opinion

- `.planning/archive/v1.2/sprint31_verification.md` — self-report biaise
- Memory files — ne pas importer le contexte du sprint

### Timebox suggere

2-3h. Priorite : Track A (task_runner reel) > Track B (output filter E2E)
> Track C (Tor transport) > Track D (P2 batch + HARDENING) > Meta-tracks.

### Delivrable final

`sprint31_audit_findings.md` dans `.planning/active/` avec verdict
PASS / CONDITIONAL PASS / FAIL.

---

## 2. Dimensions d'audit

### Track A — task_runner reel Ollama (Phase A `e85623a`)

Resolution carry MANDATORY P2-REVIEW-C-1 (S29→S30→S31).

1. **TR-1 LlmBackend wire** : verifier que `task_runner.rs` instancie un
   `OllamaBackend` via `--ollama-endpoint` CLI arg dans `main.rs`. Le stub
   doit avoir disparu (sauf en mode test). Grep `task_runner.rs` pour
   confirmer absence de `TaskExecuteResult::default()` ou retour vide.

2. **TR-2 Generate response mapping** : verifier que `GenerateResponse` du
   crate `ollama-rs` est correctement mappe en `TaskExecuteResult` (text,
   prompt_tokens, completion_tokens, duration). Test
   `execute_task_ollama_mock_maps_response` doit confirmer le mapping
   field-by-field.

3. **TR-3 Error path** : verifier que `LlmBackendError` (Ollama unreachable,
   timeout, malformed response) est propage en JSON-RPC error response
   au broker. Test `execute_task_error_when_unreachable` doit montrer
   l'error path.

4. **TR-4 Stub-mode fallback** : verifier que le mode `--ollama-endpoint`
   absent retourne le stub (TaskExecuteResult vide) — utile pour tests
   sans Ollama running. Test `execute_task_stub_mode_returns_empty`.

5. **TR-5 CLI argument parsing** : verifier que `cli_parses_ollama_endpoint`
   passe et que le format `--ollama-endpoint http://localhost:11434` est
   accepte.

6. **TR-6 schemars dep absente** : le plan §5.2 listait `schemars` comme
   dep a ajouter. Phase A review P3 confirme que ce n'etait pas necessaire
   (l'executor ne fait pas de schema enforcement). Verifier que
   `nexus-executor/Cargo.toml` ne contient PAS `schemars`.

### Track B — §9.5 output filter E2E (Phase B `0771dc8`)

Resolution carry MANDATORY P2-REVIEW-B-2 (S29→S30→S31).

1. **OF-1 Wire post-verify** : verifier que `OutputFilter.filter()` est
   appele **apres** la verification signature 3-layer du result, dans
   `validator.py` ou `coordinator.py`. Le worker non-trusted ne doit
   pas pouvoir bypasser le filter.

2. **OF-2 OutputFilter instantiation** : Phase B review P2 indique que
   `Coordinator.start()` instancie `OutputFilter`. Verifier que
   l'instanciation est bien dans le boot path coordinator et que le
   filter est partage (pas re-cree par result).

3. **OF-3 Rejected results** : verifier que les results invalides
   (filter detecte invisible text, prompt echo > 0.85, etc.) sont
   marques `rejected` (meme semantique que verify fail) et ne creditent
   PAS de kudos. Test 1 des 5 E2E doit couvrir ce path.

4. **OF-4 Tripwire logging** : verifier que les evidence (invisible_text
   detected, prompt_echo distance, risk_score) sont loggees dans l'audit
   log pour tuning futur. Pattern identique a `PiiInputGuardrail`.

5. **OF-5 Context threading** : verifier que `system_prompt`, `user_prompt`,
   `model_output` sont bien threades dans le call. Le filter a besoin
   de tous les 3 (l'echo necessite le user_prompt original).

6. **OF-6 5 tests E2E** : `test_result_guardrails.py` 5 tests passent.
   Tests : invisible text + prompt echo + clean (passthrough) + edge
   (empty output) + threshold tuning.

### Track C — Tor transport phase 1 (Phase C `687f6db`)

Feature principale S31 — partiellement livree (infra config + wire,
dep activation differee S32).

1. **TT-1 TorConfig parsing** : verifier que `TorConfig::from_toml()`
   parse correctement `configs/tor.toml.sample` (enabled bool,
   bootstrap_timeout, fallback_direct). Tests
   `test_tor_config_parse_toml`, `test_tor_config_default`,
   `test_tor_config_missing_file_returns_default`.

2. **TT-2 Feature gate** : verifier que le module `tor_transport.rs`
   est gate-d sous `#[cfg(feature = "tor")]` dans `nexus-core-rs`, et
   que la feature passe-through `tor = ["nexus-core-rs/tor"]` est
   declaree dans `nexus-core-py/Cargo.toml`. Sans la feature, le code
   ne doit pas compiler de symboles arti-client.

3. **TT-3 Disabled mode noop** : verifier que `TorConfig { enabled:
   false, .. }` ne tente pas de bootstrap arti et que le coordinator
   utilise le transport HTTP direct. Test
   `test_tor_config_disabled_noop`.

4. **TT-4 Fallback path** : verifier que si le bootstrap arti echoue
   (timeout, dep manquante), le coordinator fallback sur HTTP direct
   (warning log, pas crash). Test `test_tor_transport_fallback_on_failure`.

5. **TT-5 Coordinator wire** : verifier que
   `tor_client.py::TorClientWrapper` est instancie depuis
   `Coordinator.start()` selon la config et que les outbound HTTP du
   coordinator passent via le wrapper. Test `test_tor_client.py` 7
   tests.

6. **TT-6 Dep arti differee — P2 carry S32** : **P2 attendu** : Phase C
   review identifie que la dep arti-client n'est PAS encore activee
   (rusqlite version conflict). L'auditeur doit confirmer que :
   - `Cargo.toml` workspace ne contient PAS `arti-client` deps actives
   - Le module `tor_transport.rs` utilise des stubs / feature flags
     pour preparer l'integration sans la dep
   - La carry rusqlite + arti dep est documentee comme P2 1/3
   - Le bootstrap arti **n'est PAS testable E2E** (test environment
     ne peut pas vraiment passer par Tor)

7. **TT-7 Pas iroh relay over Tor** : verifier qu'aucune modification
   d'iroh relay (Endpoint::builder) n'a ete introduite. Phase 2 S32+.

### Track D — P2 batch S30 + G2 HARDENING (Phase D `ab09b5d`)

Items batch + G2 + HARDENING update.

1. **BD-1 WebAppFrame deletion** : verifier que `web/src/components/app/
   WebAppFrame.tsx` et `WebAppFrame.test.tsx` sont absents. P3-AUDIT-1
   ferme. Verifier aussi qu'aucun import dangling dans
   `web/src/components/app/` ne reference le composant.

2. **BD-2 VALIDATED_BLUEPRINT Couche 6** : verifier que la section
   Couche 6 reference SynthID (pas Kirchenbauer) et GLiNER (pas spaCy).
   `grep -c SynthID docs/security/VALIDATED_BLUEPRINT.md` >= 1.

3. **BD-3 SPLIT_INFERENCE confidence_score** : verifier que `§4.1`
   contient le champ `confidence_score: f64 (0.0–1.0)`. C'est un
   research doc (pas wire format), donc l'ajout est design intent.

4. **BD-4 HARDENING last_validated** : verifier
   `last_validated: 2026-04-27` dans la frontmatter de
   `HARDENING_ROADMAP.md`. Le commentaire G2 doit lister :
   - Tor transport phase 1 delivered (avec scope precis)
   - task_runner reel wired
   - §9.5 output filter wired E2E
   - WebAppFrame supprime
   - iroh 0.98 deferred S32 (Day 0 #3)
   - Carries S31 resolus

5. **BD-5 HTTP FROST tests** : `cargo nextest -p nexus-shell-daemon -E
   'test(frost_http)'` doit montrer 4 tests passants :
   `frost_http_trusted_dealer_returns_shares_and_pubkey`,
   `frost_http_round1_returns_commitment_and_nonces`,
   `frost_http_round2_returns_signature_share`,
   `frost_http_aggregate_returns_valid_signature`. Verifier que les
   tests exercent le full flow HTTP→FROST→Ed25519 verify.

6. **BD-6 Compteurs reconciliation** : Phase D review P2-REVIEW-D-1
   indique que les compteurs frontmatter HARDENING etaient inexacts
   (~401 coord vs reel 405-406). Verifier que le commit Phase E a
   corrige ces compteurs (~878 Rust / ~406+36f+6s coord / ~1877 total).

### Track E — G1 Design Review Board

Verifier que `sprint31_design_review.md` existe dans
`.planning/archive/v1.2/`. Present avec scoring D1-D5 = OK.
Sprint kickoff §4 doit acknowledger les findings (D3 ⚠️ Tor scope
narrower que HARDENING_ROADMAP §3 prescription).

### Track F — Phase review completeness

Phase reviews sources :

| Finding | Source | Track | Status |
|---|---|---|---|
| P2-REVIEW-A-1 LOC estimees plan §5.5 | Phase A review | meta-process | carry S32 1/3 |
| P3-REVIEW-A-1 schemars dep inutile | Phase A review | A (TR-6) | resolu plan over-spec |
| P2-REVIEW-B-1 plan stale (result_guardrails.py path) | Phase B review | B (OF-2) | observation meta-process, pas code carry |
| P3-REVIEW-B-1 output_filter_policy_path test absente | Phase B review | B (OF-1) | exercice par 5 tests E2E |
| P2-REVIEW-C-1 rusqlite + arti dep activation | Phase C review | C (TT-6) | carry S32 1/3 |
| P3-REVIEW-C-1 LOC estimees plan §7.2 | Phase C review | meta-process | nit informatif |
| P2-REVIEW-D-1 compteurs frontmatter HARDENING approximatifs | Phase D review | D (BD-6) | reconciliation Phase E intra-sprint |
| P3-REVIEW-D-1 confidence_score field cosmetique | Phase D review | D (BD-3) | cosmetique |

### Track G — HARDENING drift

Comparer HARDENING_ROADMAP §3 ligne S31 (items prescrits) vs livre :

| Item prescrit | Livre ? | Justification si non |
|---|---|---|
| Tor transport phase 1 (arti >= 1.0 debloque) | Partiel | Infra config + wire livres ; dep arti differee S32 (rusqlite conflict) |
| §9.5 output filter wire E2E (carry 2/3) | Oui | Phase B `0771dc8` |
| task_runner implementation reelle (carry 2/3) | Oui | Phase A `e85623a` |
| P2 carries S30 Phase B/C reviews | Oui | Phase D `ab09b5d` (VALIDATED_BLUEPRINT + confidence_score + HTTP FROST + WebAppFrame) |
| iroh 0.98 upgrade (roadmap Alexandria S31 Phase C prescription) | **Non** | Scope-cut D5 S31 (risque cascade + 3 hard deliverables) — scheduled S32 |

### Meta-track — G8 traceability

1. Verifier que les 4 phases A-D ont chacune un
   `sprint31_phase_{X}_preflight.md` dans `.planning/archive/v1.2/`.
2. Verifier que les 4 phases A-D ont chacune un
   `sprint31_phase_{X}_review.md` dans `.planning/archive/v1.2/`.
3. Verifier la coherence verdict G8 × commit (4 EXECUTE → 4 commits
   feat phase livres, 0 DESIGN-CONFLICT → 0 pivot_proposal).

### Meta-track — Sprint pair S32 phase dette

S32 est un sprint pair → phase dette obligatoire (§6.2.1 Regle 1).
Candidats obligatoires :

- **iroh 0.98 upgrade** : LT-6 trigger met (iroh 0.98.0 publie
  2026-04-17), Day 0 #3 pin a lever. **Scheduled S32 dedie** par D5
  S31. L'auditeur doit verifier que le kickoff S32 reserve une phase
  pour cet upgrade.
- **rusqlite 0.32→0.36 + arti-client dep activation** (P2-REVIEW-C-1
  S31, 1/3) : couplee avec iroh 0.98 si meme phase upgrade. Sinon
  phase dette dediee.

Candidats recommandes :
- P2-REVIEW-B-1-S30 Playwright COEP (2/3, MANDATORY S33 si non resolu
  S32) — necessite stabilisation env Playwright (coordinator running)
- P2-REVIEW-A-1 LOC plan meta-process (1/3) — discipline plan-writing,
  pas d'impact code direct

### Meta-track — Roadmap Alexandria deviation

Le roadmap v1.0 Alexandria (`c50976a` S30) prescrivait iroh 0.98 en
S31 Phase C. Le kickoff S31 §1.2 + D5 ont devie volontairement vers
S32. Verifier que :

- La deviation est documentee explicitement (kickoff §1.2 + D5)
- Le rationale est defendable (3 hard deliverables, risque cascade,
  sprint pair S32 = phase dette ideale)
- Le roadmap deviation est sub-section §1.2 ("Le kickoff a autorite
  sur le roadmap (§8 du roadmap : 'Ce document n'est PAS un kickoff
  sprint')")
- Le S32 kickoff devra reprendre cet item explicitement

---

## 3. Calibration rigor G4

L'audit DOIT trouver au minimum 1 P2+ pour verdict PASS. Sinon
verdict CONCERN et re-audit dimension supplementaire.

Phase reviews S31 ont produit 4 P2 + 4 P3 (1 par phase). L'auditeur
doit chercher des **angles non couverts** par les phase reviews
intra-sprint, par exemple :

- coordinator Tor wire est-il vraiment opt-in ? (TT-3) ou y a-t-il
  un effet de bord meme avec `enabled: false`
- output filter contexte threading est-il etanche ? (OF-5) le
  worker peut-il influencer system_prompt ?
- HTTP FROST tests exercent-ils des paths d'erreur reels ? (BD-5)
  ou seulement le happy path

---

## 4. Pre-launch protocol check

Verifier :
- `*_VERSION = 1` partout (aucun bump S31)
- Tor transport = config TOML + runtime arti, pas wire format P2P
- Output filter = post-verify result path coordinator-side, pas wire
- task_runner Ollama = HTTP local, pas wire P2P
- HTTP FROST tests = exercent endpoints existants, pas nouvelle wire surface
- Aucun tolerant decoder multi-version introduit S31
- Aucun test "legacy decode" zombie introduit S31
- `#[serde(default)]` avec rationale runtime tolerance (si introduit)

---

## 5. Out of scope pour l'audit

Ne PAS rebattre :
- D1..D5 gelees (task_runner Ollama wire, output filter post-verify,
  Tor coordinator outbound HTTP, P2 batch S30, iroh 0.98 scope-cut S32)
- Les 12 scope cuts kickoff §7
- Le choix Ollama vs llama.cpp executor
- Le choix coordinator-side filter vs worker-side
- Le choix arti-client direct API vs SOCKS proxy
- Pin iroh 0.97 (sera leve S32)

---

## 6. Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S32 Phase A (ou phase dette upgrade) demarre
  direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → S32 Phase A bloque
  tant que les `fix(sprint31): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

Note : sprint pair S32 = phase dette obligatoire. L'audit gate
debouche directement sur la phase dette si verdict PASS.
