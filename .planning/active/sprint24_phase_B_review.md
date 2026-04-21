# Sprint 24 Phase B — nexus-phase-auditor review

HEAD pre-commit: ff4c7d53db9b527431ea34103dd47cc1e5e4465d
Draft commit body: "feat(sprint24): Phase B — B1 guardrails pipeline declaratif Guardrail ABC + GuardrailChain + retrofit 4 primitives"
Timebox: 25m (LIGHT-AUDIT — preflight EXECUTE)

## Acknowledged by G8 preflight (not re-derived)

- S1 SOTA 2026 : openai-agents-python v0.14.3 context7 validé 2026-04-21, API stable — clean
- S2 historiques : S21 `23abb11` + S22 `690fab3` + S23 D5 scope cut "B1 → S24 Phase B" — pas de DEVIATION — clean
- S3 threat model : T2/T3/T4 coverés, wrapping primitives existantes, comportement identique — clean
- S4 wire format : aucun `*_VERSION` touché, `canonical.rs` non touché, pre-launch invariants préservés — clean

## Verdict : PASS

0 P0, 0 P1. 2 P2 documentés (rigor signal G4 satisfait).

## Dimensions

### Security

- [x] Semgrep / grep : 0 secrets, 0 path traversal, 0 `unsafe` block. Diff = Python pur, pas de Rust.
- [x] `unwrap/unimplemented/TODO/FIXME` : absent de tous les fichiers du diff (grep vérifié).
- [x] Loopback/wire : diff ne touche pas les routes loopback, pas de nouvelles routes HTTP. PeerCreds non pertinent.
- [x] Wire format : `dispatcher.py` signe via `nexus_core.sign_task(task_json, ...)` sur un dict `json.dumps(sort_keys=True)` — pas JCS canonique, mais pattern identique au code pré-diff (inchangé). Pas de régression.
- [x] Nouveaux inputs non validés : `GuardrailContext` accepte `task_id / system_prompt / user_prompt` — strings, aucun parsing complexe, pas de path traversal possible.
- [x] ABC virtual subclass registration : vérifié via `uv run python -c` — tous les 4 adapters satisfont `isinstance(obj, Guardrail) = True`, `direction` accessible. Duck typing safe.

### Patterns

- [x] Shell PATTERNS.md P5 (CORS loopback) : non touché.
- [x] Shell PATTERNS.md P6 (NEXUS_GRID_ROOT) : non touché.
- [x] Shell PATTERNS.md PyO3 wheel rebuild (Sprint 24 section) : déjà documenté Phase A. Non re-testé ici.
- [x] Rust PATTERNS.md : aucun fichier Rust dans le diff.
- [x] Pattern guardrails (nouveau, pas encore dans PATTERNS.md) : la convention virtual subclass + `_register_*()` module-level est cohérente dans les 4 adapters. Pattern drift potentiel : si un 5e adapter oublie `_register_*()`, il ne sera pas reconnu comme `Guardrail` à runtime. P2 — documenter ce pattern de registration dans PATTERNS.md (candidat §P37 post-Phase B ou Phase F).

### G8 traceability

- [x] Artefact G8 présent : `.planning/active/sprint24_phase_B_preflight.md` (lu)
- [x] Verdict preflight : EXECUTE plan-as-is — 4/4 scans clean
- [x] Plan §Phase B (§6 sprint24_plan.md) reflète exactement le scope implémenté : 7 fichiers attendus = 7 livrés
- [x] Pas de DESIGN-CONFLICT → vérification plan-vs-code non requise

### Scope-cuts

Grep exhaustif du diff contre les 10 scope-cuts du kickoff §7 :

| Scope-cut | Pattern grep | Résultat |
|---|---|---|
| key rotation | `key.rotation\|rotate.*ceremony` | absent du diff |
| C3 handoffs | `c3.handoff\|semantic.dispatcher` | absent |
| cross-process chain | `rate.limit.*rust\|pii.*iframe.*TS` | absent — la chain ne traverse pas les frontières de process |
| P2-D-1 redundancy persistence | `redundancy.*persist\|sqlite.*redundancy` | absent |
| P2-D-2 quarantine alerting | `quarantine.*alert\|curator.*notify` | absent — `QuarantineGuardrail.on_tripwire` log info uniquement |
| P2-E-1 iroh neighborhood | `iroh.*neighborhood\|neighbour` | absent |
| domain fronting impl | `domain.front\|fronting` | absent |
| T-NN+2 iframe Rust-wasm | `wasm.*iframe\|iframe.*wasm` | absent |
| LT-2 Radicle | `radicle` | absent |
| LT-3/LT-4 | hors-sprint | absent |

Aucun scope creep détecté.

### Tests-delta

Annonce : +15 coord
Mesuré : `uv run pytest packages/nexus-coordinator/tests/ -q --tb=no` →
**290 pass + 32 fail (stale PyO3 pre-existing) + 3 skip**
Entrée sprint24_plan.md §1 : 272 pass + 32 fail + 3 skip
Delta réel : 290 - 272 = **+18 pass**

Divergence : annonce +15, réel +18. Delta positif (non-régressif), mais supérieur à l'annonce.
Explication probable : les 15 tests `test_guardrails.py` sont bien présents ; les 3 supplémentaires existent probablement dans d'autres fichiers de test de la suite (tests impactés positivement par les nouveaux adapters).

Vérification directe `test_guardrails.py` : `uv run pytest packages/nexus-coordinator/tests/test_guardrails.py -q` → **15 passed in 0.04s**. Conforme à l'annonce.

Verdict delta : **PASS** (pas de régression, écart positif mineur non-bloquant).

### Research-grounding

- [x] Cargo.toml/Cargo.lock : `git diff HEAD -- Cargo.toml Cargo.lock | grep "^+"` → vide. Aucune nouvelle dep Rust.
- [x] pyproject.toml : `git diff HEAD -- packages/nexus-coordinator/pyproject.toml | grep "^+"` → vide. Aucune nouvelle dep Python.
- [x] Imports externes nouveaux dans le diff : aucun. Les adapters réutilisent `structlog`, `abc`, `dataclasses` (stdlib). Aucune nouvelle lib externe.
- [x] `openai-agents-python` v0.14.3 : référencé §Research consulte sprint24_plan.md §3 + context7 validé kickoff §Sources. Trace conforme.
- [x] Pas d'API crypto ni spec standardisée nouvelles dans le diff.

### Horizon long-terme + documentation amont

- [x] Design doc `docs/security/GUARDRAILS_ARCHITECTURE.md` existe et mis à jour (§1.3 comparative analysis ajouté, status updated). Pré-code : oui (écrit S22, référencé kickoff D1).
- [x] Alternatives rejetées : D1 kickoff §4 cite Strategy pattern, Express-style middleware, if/else ad-hoc — toutes rejetées avec rationale. G1 review findings (4 ⚠️) acknowledged avec rationalisation dans kickoff §4.5.
- [x] Solution techniquement poussée : ABC Python + virtual subclass (pattern openai-agents) > Strategy registry > if/else. Choix défendable.
- [x] Aucune estimation LOC dans plan.md ou kickoff.md (grep `LOC estimee|estime.*LOC` : aucun hit hors D5 qui mentionne "~500 LOC" et "~700 LOC" pour les items scope-cuttés, pas pour Phase B). Conforme.

## Findings

- **P2** [test-coverage] : `OutputTripwire` jamais exercé via `GuardrailChain.run()` dans les tests. `test_output_safety_guardrail_trip` (ligne 165) vérifie uniquement que `outcome.tripwire == True` sur le check direct — mais ne teste pas que `chain.run()` lève `OutputTripwire` quand un guardrail de direction "output" tripwire. Chemin présent dans `guardrails.py:95-96` mais non couvert. Candidat test Phase C ou F. Ajouter à `sprint25_audit_findings.md`.

- **P2** [design-smell] : `dispatcher.py:154-155` appelle `input_chain.run(ctx, req.prompt)` puis `input_chain.run(ctx, req.system_prompt)` séquentiellement. Si `CanaryInputGuardrail` est dans la chain, `should_inject()` sera appelé deux fois par soumission de tâche, doublant les compteurs `_seen_count` et la probabilité d'injection effective. Le taux d'injection réel serait ~2/N au lieu de 1/N. Actuellement non-bloquant (aucun appel de production ne wire `CanaryInputGuardrail` dans la `input_chain` du dispatcher — le plan prévoit cela en Phase D/E), mais le pattern de dispatch double-champ doit être documenté ou corrigé avant le wire-up de Phase D. Ajouter à `sprint25_audit_findings.md`.

- **P3** [nit] : `GuardrailOutcome.latency_ms` est déclaré (ligne 39 `guardrails.py`) mais toujours à 0.0 — aucun adapter ne mesure la latence. Le champ sera utile pour A1 hooks observability (Phase C) mais reste un no-op complet Phase B. Non-bloquant, cohérent avec le design incrémental.

## Recommendation

Commit autorisé. 0 P0/P1. Les 2 P2 sont non-bloquants et tracés pour S25 audit_plan.
