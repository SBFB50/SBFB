# Sprint 24 — Kickoff (Guardrails refactor + TaskDispatchHooks + re-run sampling + DNS fallback)

**Ecrit** : 2026-04-21 (session fraiche post-audit gate S23 `9628e63`
+ fix P1 C-1 `34c77ce` + 4 chore process review `466f826..91589ea`).
**Type** : **sprint implementation** (architecture guardrails + hooks
observabilite + detection compute theft + resilience DNS transport).
**Tip master d'entree** : `91589ea` (chore batch cleanup process review).
**Phase 0 audit Sprint 23** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint23_audit_findings.md` (verdict
**CONDITIONAL PASS**, 1 P1 C-1 fixe `34c77ce`, leve). Migre vers
`archive/v1.2/` dans ce commit d'ouverture S24.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **context7** `/openai/openai-agents-python` (2026-04-21) — API
  guardrails stable v0.14.3 : `@input_guardrail / @output_guardrail`
  + `GuardrailFunctionOutput(output_info, tripwire_triggered: bool)`
  + exceptions `InputGuardrailTripwireTriggered /
  OutputGuardrailTripwireTriggered`. Pattern B1 design doc
  `GUARDRAILS_ARCHITECTURE.md` inchange.
- **Nouveau package** `openai/openai-guardrails-python` (distinct de
  l'agents SDK) : wrapper OpenAI-specific safety/compliance, non
  pertinent SBFB (nous implementons nos propres guardrails).
- **G2 trigger check** (5 triggers scannees 2026-04-21) :
  - `openai-agents-python > 0.7.0` : **ACTIVE** (0.14.3) — API
    guardrails stable, pas d'impact design B1.
  - `frost-ed25519 > 2.1` : INACTIVE (2.1.0).
  - `iroh > 0.97` : INACTIVE (0.97.x).
  - `wasmtime LTS bump` : INACTIVE (43.0.x non-LTS, LTS 36.0.7 ok).
  - `microsoft/sudo > 24H2` : INACTIVE.

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `91589ea` — 5 commits au-dessus du gate S23 `9628e63`
  (1 fix P1 `34c77ce` + 4 chore process review).
- **Working tree** : propre (post-migration `sprint23_audit_findings.md`
  → `archive/v1.2/`).
- **v1.2** : continuation security hardening. Pas de nouvelle version.

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S24 : "Re-run sampling + DNS fallback + key
rotation + A1 TaskDispatchHooks + C3 handoffs". Le sprint absorbe
les 4 premiers items + B1 guardrails (prereq A1, carry S23 D5). Key
rotation et C3 handoffs scope-cut S25 (cf. §7).

### 1.3 Compteurs tests entree (tip `91589ea`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 743 | all pass |
| Rust doctests | pass | |
| Python SDK | 185 | all pass |
| Python coord | 272 pass + 32 fail + 3 skip | 32 fail = stale PyO3 wheel (pre-existing) |
| Python gov | 46 | all pass |
| Vitest | 264 | all pass |
| Playwright | 43 | all pass |
| Size-limit | 7/7 | |
| **Total** | **~1563** | |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. Pas de
tolerant decoder multi-version. Cf. `CLAUDE.md §Pre-launch protocol
policy`.

---

## 2. Goal

Unifier les 6 guardrails ad-hoc (S21-S22) en pipeline declaratif
composable, poser l'infrastructure hooks lifecycle A1 pour
l'observabilite, livrer la detection compute theft par re-run sampling,
et durcir la resilience transport via DNS fallback DHT.

**Critere SMART : 30+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase F wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 23

**Verdict** : CONDITIONAL PASS → **LEVE** (P1 C-1 fixe `34c77ce`
excluant `redundancy_factor` du canonical bytes).
**Commit stack gate** :
```
9628e63 chore(sprint23): audit gate S23 — findings CONDITIONAL PASS (1 P1 C-1)
34c77ce fix(sprint23): exclude redundancy_factor from canonical bytes (R3 mitigation)
```
**P2 carry S24** absorbes Phase A ci-dessous (cleanup batch).

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — B1 guardrails refactor : Guardrail ABC + GuardrailChain pipeline declaratif

**Retenu** : ABC Python `Guardrail` avec `check(ctx, payload) ->
GuardrailOutcome` + dataclass `GuardrailOutcome(passed, info,
tripwire)` + `GuardrailChain([g1, g2, ...])` execution ordonnee avec
short-circuit sur tripwire + exceptions typees `InputTripwire /
OutputTripwire`. Pattern emprunte a openai-agents-python (G2 valide
v0.14.3, API stable). Retrofit des 4 primitives coord-side :
`PiiRedactor` → `PiiInputGuardrail`, `OutputFilter` (validator) →
`OutputSafetyGuardrail`, `QuarantineQueue` → `QuarantineGuardrail`,
`CanaryInputInjector` → `CanaryInputGuardrail`. Les 2 primitives
hors-coord (rate-limit Rust worker-side S22 + PII iframe TS S21-S22)
restent independantes (scope-cut : pas de GuardrailChain cross-process).

**Rejete** :
- **Strategy pattern + plugin registry** : over-engineered pour 4
  primitives connues. Le pipeline n'a pas besoin de decouverte
  dynamique.
- **Middleware chain Express-style** (`next()` cascade) : ordre
  implicite, error handling complexe, composition opaque pour
  l'auditeur. Le pattern guardrail openai-agents est explicite
  (list ordonnee, chaque guardrail retourne un outcome typee).
- **Conserver if/else ad-hoc** : chaque nouveau checker S24+ ajoute
  10-15 lignes de branching dans `dispatcher.py`. N=4 S22, N=6 post-
  B1, N=8+ S25 → ingerable. Deja flagge S22 kickoff.

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/guardrails.py`
  (ABC + GuardrailOutcome + GuardrailChain + exceptions)
- Retrofit `pii_redactor.py`, `output_filter.py`, `quarantine_queue.py`,
  `canary_input.py` pour implementer `Guardrail`
- `dispatcher.py` remplace les N if/else par
  `GuardrailChain.run(ctx, payload)`
- Design doc `docs/security/GUARDRAILS_ARCHITECTURE.md` (S22) = reference

### D2 — A1 TaskDispatchHooks : lifecycle events types injectables

**Retenu** : ABC Python `DispatchHook` avec 5 events types :
`on_claim_broadcast`, `on_task_dispatched`, `on_result_received`,
`on_validator_post_task`, `on_quarantine_enqueue`. Injectable via
`Dispatcher.__init__(hooks=[...])`. Chaque event recoit un `HookContext`
(task_id, timestamp, metadata dict). Les hooks sont fire-and-forget
(pas de veto, pas de retry). Consumer initial : `DivergenceScorer`
(Phase D).

**Rejete** :
- **Event bus pub/sub global** (in-process message broker) :
  over-engineered pour coordinator single-process. Pas de subscribers
  distribues pre-v1.0.
- **Direct method overrides** (subclass `Dispatcher`) : couplage
  fort, impossible de composer plusieurs observers. Test coverage
  fragile (mock de la classe entiere).
- **AOP/aspect decorators** (wrap method calls) : implicite, hard
  to debug, execution order opaque. Mauvais DX pour auditeur
  externe.

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/hooks.py`
  (ABC + HookContext + HookRunner composite)
- Integration `dispatcher.py::scan_and_execute_tasks()` fire events
  aux 5 points
- Trait Rust `DispatchHook` + PyO3 binding pour consumer Rust-side
  futur (S29 TraceProvider) — stub only S24, implementation Python-
  first

### D3 — Re-run sampling : detection divergence 1-5%

**Retenu** : coordinator selectionne aleatoirement 1-5% des tasks
completees (taux configurable `rerun_sample_rate: float` dans
`coordinator.toml`) pour re-dispatch a un second worker. Le
`DivergenceScorer` (consumer hook `on_result_received` D2) compare
le hash BLAKE3 du resultat canonique original vs re-run. Score
divergence > seuil (defaut 0.0, i.e. mismatch binaire exact) →
auto-report au curator + quarantine du worker divergent.

**Rejete** :
- **Worker self-report** : fox-guarding-henhouse, un worker compromis
  ne signale pas sa propre divergence.
- **Full redundancy systematique** (factor=3 pour tout) : deja couvert
  par S23 `redundancy_factor`. Le re-run sampling est un spot-check
  statistique peu couteux, complementaire au vote majoritaire.
- **Client-side re-run** : le client n'a pas le modele, impossible
  de verifier le resultat cote consumer.

**Implications** :
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/rerun.py`
  (RerunSampler + DivergenceScorer + config TOML)
- Integration via hooks D2 : `DivergenceScorer` s'enregistre comme
  hook `on_result_received`
- Dep : aucune nouvelle (BLAKE3 deja dans workspace via `blake3` crate,
  cote Python via PyO3 `nexus_core.blake3_hash` ou hashlib fallback
  SHA-256 si binding absent)

### D4 — DNS-based DHT fallback (DoH + DoT)

**Retenu** : quand les relais pkarr sont injoignables (timeout 10s
sur les 3 paralleles S18 `PkarrQuorumResolver`), fallback vers
resolution DNS via DoH (RFC 8484, Cloudflare `1.1.1.1/dns-query` +
Google `dns.google/dns-query`) et DoT (RFC 7858, port 853). Encode
les enregistrements pkarr dans des TXT records DNS (format deja
DNS-compatible par design pkarr). Implemente cote Rust dans
`nexus-core-rs` via `hickory-resolver` (anciennement trust-dns,
crate mature, 4.4k stars GitHub, Apache-2.0). Le fallback est
**additif** — il n'intervient que si le quorum pkarr echoue.

**Rejete** :
- **DNS plain (RFC 1035)** : pas de chiffrement, MITM trivial par
  ISP/operateur. Les records pkarr sont signes Ed25519 donc le
  contenu est integre, mais le metadata (qui resout quoi) fuite.
- **DNSCrypt** : protocole de niche, support inconsistant cote
  resolvers publics. DoH/DoT couvrent 95%+ des resolvers en 2026.
- **Custom bootstrap protocol** : reinvention inutile quand DNS est
  le plus grand reseau de nommage distribue existant.

**Implications** :
- Nouveau `crates/nexus-core-rs/src/dns_fallback.rs` (DnsFallbackResolver
  + DoH/DoT config + TXT record parsing)
- Integration `browse_aggregator` (fallback chain pkarr → DNS)
- Dep : `hickory-resolver` ajout workspace Cargo.toml
- Design doc domain fronting outline (design-only, pas d'implementation
  S24 — legal review prerequis)

### D5 — Scope management : key rotation + C3 handoffs → S25

**Retenu** : key rotation ceremony (`~500 LOC`) et C3 handoffs
semantic dispatcher (`~700 LOC`) differes S25. Raisons :
- Key rotation requiert revocation list gossip topic + ceremony UX
  → complexite standalone, pas de dep sur B1/A1.
- C3 handoffs depend de B1 GuardrailChain stable + A1 hooks testes
  → premier sprint sur ces fondations, stabiliser avant d'empiler.
- Budget S24 : ~2400 LOC sans ces items. Les ajouter = ~3600 LOC,
  bien au-dessus de la norme ~2500.

**Rejete** :
- **Inclure les deux** (LOC > 3500) : risque scope-creep + phases
  trop lourdes.
- **Inclure key rotation seul** (pas de dep B1) : moins prioritaire
  que DNS fallback pour la resilience transport Gate 3.
- **Inclure C3 seul** (depend de B1+A1) : risque implementation
  sur fondations pas encore testees en production-like conditions.

---

## 4.5 Design Review Board findings (G1)

**Report** : `.planning/active/sprint24_design_review.md` (2026-04-21).
**Verdict** : 0 ❌ + 4 ⚠️ + 1 ✅. Proceder Phase A.

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (4 ⚠️ sur 5, toutes competitive-analysis
gaps — pas de contradiction technique).

### Acknowledged review findings

- **⚠️ D1-G1-1 (alternatives non comparees)** : ACKNOWLEDGED.
  LangChain middleware hooks = architecture fondamentalement differente
  (state graph), inadaptee a notre dispatcher lineaire. NeMo Guardrails
  Colang DSL = over-engineered pour 4 primitives connues. Guardrails AI
  Guard = structurellement similaire a notre GuardrailChain, confirme le
  pattern. Decision : maintenir ABC+Chain, ajouter note comparative
  dans `GUARDRAILS_ARCHITECTURE.md §1.3` Phase B pre-commit.

- **⚠️ D2-G1-1 (5-event set non justifie)** : ACKNOWLEDGED. Le set
  couvre le lifecycle dispatch complet tel que wire dans dispatcher.py
  + validator.py actuels. `on_task_assigned` = equivalent
  `on_task_dispatched`. `on_worker_timeout` + `on_retry` = Phase C
  scope-cut S25 extensions (veto semantics, pas fire-and-forget).
  Decision : documenter les 3 events candidats S25 dans hooks.py
  docstring. Pas de changement D2 S24.

- **⚠️ D3-G1-1 (taux non source)** : ACKNOWLEDGED. Le taux 1-5% est
  un parametre configurable, pas un hardcode — le default 1% est
  conservateur. BOINC/Folding@Home utilisent replication complete, pas
  spot-check, car leurs tasks sont deterministes. Nos tasks LLM sont
  stochastiques, le spot-check est le seul pattern applicable. Decision :
  ajouter note statistique dans `rerun.py` docstring (1% sur 100
  tasks/jour = 1 re-run, detection latence ~1 jour pour worker
  divergent systematique). Pas de changement D3 S24.

- **⚠️ D4-G1-1 (alternatives non documentees)** : ACKNOWLEDGED.
  hickory-resolver est le resolver DNS Rust le plus mature (4.4k stars,
  DoH+DoT+DNSSEC natif, tokio async, Apache-2.0). `doh_dns` = DoH-only
  (pas DoT), maintenance inconnue. `reqwest custom` = reimplementation
  inutile. Decision : ajouter note `dns_fallback.rs` header documenting
  choice vs alternatives. Pas de changement D4 S24.

---

## 5. Phase outline

### Phase A — P2 cleanup batch S23 audit + PATTERNS §P35/P36

- **Scope** : absorber les P2 du gate S23 + ecrire PATTERNS manquants
  - P2-B-1 : `pow.rs` saturer `exponent` avant cast `as i32`
    (`exponent.min(i32::MAX as u64)`)
  - P2-C-2 : documenter deviation SHA-256 vs BLAKE3 dans kickoff
    carry notes + commentaire `redundancy.py`
  - P2-F-1 : ecrire PATTERNS.md §P35 (ephemeral worker lifecycle)
    + §P36 (redundancy voting pattern)
  - P2-E-2 : `pyproject.toml` dep floor `pynacl >= 1.6.2`
    (CVE-2025-69277)
  - P2-F-1bis : documenter procedure `maturin develop --release`
    dans `docs/shell/PATTERNS.md` (PyO3 rebuild stale)
  - P2-E-3 : `KudosLedger` public API cleanup (expose
    `get_total_kudos()` + `get_top_contributors(n)` pour fairness
    diagnostic)
  - HARDENING_ROADMAP `last_validated` update → 2026-04-21 S24
- **Critere** : `cargo nextest run -p nexus-core-rs` vert (B-1
  fix), `uv run ruff check` vert, PATTERNS diff visible
- **Commit** : `feat(sprint24): Phase A — P2 cleanup batch S23
  audit + PATTERNS §P35 ephemeral + §P36 redundancy`

### Phase B — B1 guardrails pipeline refactor

- **Scope** : `guardrails.py` (ABC Guardrail + GuardrailOutcome +
  GuardrailChain + InputTripwire + OutputTripwire) + retrofit 4
  primitives coord-side (PiiRedactor → PiiInputGuardrail, OutputFilter
  → OutputSafetyGuardrail, QuarantineQueue → QuarantineGuardrail,
  CanaryInputInjector → CanaryInputGuardrail) + `dispatcher.py`
  integration GuardrailChain + contract tests
- **Critere** : 40+ tests contract (8 per guardrail + 8 chain
  composition), `uv run pytest packages/nexus-coordinator/tests/` vert
  (net des 32 stale)
- **Commit** : `feat(sprint24): Phase B — B1 guardrails pipeline
  declaratif Guardrail ABC + GuardrailChain + retrofit 4 primitives`

### Phase C — A1 TaskDispatchHooks lifecycle events

- **Scope** : `hooks.py` (ABC DispatchHook + HookContext + HookRunner
  composite + 5 events) + `dispatcher.py` integration fire events +
  trait Rust `DispatchHook` stub + PyO3 binding stub + tests
- **Critere** : 30+ tests (5 events × 6 scenarios : fire, multi-hook,
  error-resilience, context propagation, ordering, noop)
- **Commit** : `feat(sprint24): Phase C — A1 TaskDispatchHooks
  5 lifecycle events + HookRunner composite + dispatcher integration`

### Phase D — Re-run sampling + divergence detection

- **Scope** : `rerun.py` (RerunSampler + DivergenceScorer) + config
  TOML `rerun_sample_rate` + hook integration (DivergenceScorer
  registered as on_result_received hook) + auto-report curator +
  quarantine divergent worker
- **Critere** : 25+ tests (sampling rate distribution, divergence
  scoring exact/fuzzy, quarantine trigger, edge cases 0%/100% rate,
  no-rerun passthrough)
- **Commit** : `feat(sprint24): Phase D — re-run sampling 1-5%
  divergence detection + auto-report curator + quarantine divergent`

### Phase E — DNS-based DHT fallback (DoH + DoT)

- **Scope** : `dns_fallback.rs` (DnsFallbackResolver + DoH + DoT +
  TXT record parse) + integration browse_aggregator fallback chain
  pkarr → DNS + `domain_fronting_design.md` outline (design-only)
- **Critere** : 20+ tests Rust (DoH resolve mock, DoT resolve mock,
  TXT parse pkarr format, fallback trigger on pkarr timeout, config
  parse, error handling)
- **Commit** : `feat(sprint24): Phase E — DNS-based DHT fallback
  DoH+DoT via hickory-resolver + domain fronting design doc`

### Phase F — wrap-up + verification + audit plan S25

- **Scope** :
  - verification.md (30+ rows fail-fast)
  - audit_plan S25
  - migration planning active → archive/v1.2/
  - SPRINT_LOG.md + CLAUDE.md updates
  - memory update tip + compteurs
- **Critere** : 30+ rows fail-fast verts, docs coherents, tous
  PATTERNS mis a jour
- **Commit** : `chore(sprint24): Phase F — wrap-up + verification
  + audit plan S25 + migration planning archive/v1.2/`

---

## 6. Items carry/dette — reclassification S23 → S24

| Item | Source | Phase S24 | Classification |
|---|---|---|---|
| P2-B-1 exponent saturation | audit_findings §P2 | Phase A | [x] resolve S24 |
| P2-C-2 SHA256→BLAKE3 doc | audit_findings §P2 | Phase A | [x] resolve S24 (doc deviation) |
| P2-F-1 PATTERNS P35/P36 | audit_findings §P2 | Phase A | [x] resolve S24 |
| P2-E-2 pynacl dep floor | audit_plan §3 | Phase A | [x] resolve S24 |
| P2-E-3 KudosLedger API | audit_plan §3 | Phase A | [x] resolve S24 |
| P2-F-1bis PyO3 rebuild doc | verification §5 | Phase A | [x] resolve S24 (doc) |
| P2-D-1 redundancy persistence | audit_plan §3 | — | [deferred] → S25 (tech debt) |
| P2-D-2 quarantine alerting | audit_plan §3 | — | [deferred] → S25 (tech debt) |
| P2-E-1 iroh neighborhood | audit_plan §3 | — | [deferred] → S25 |
| T-NN+2 iframe Rust-wasm | S22 carry | — | [deferred] PATTERNS §P34 |
| LT-2 Radicle | S22 carry | — | hors cap formel (trigger tag v1.0) |
| LT-3/LT-4 | S22 carry | — | hors-sprint (post-v1.0) |

**Cap G7 bilan** : 2/2 slots carry formels consommes (key rotation
+ C3 handoffs, cf. §7). P2-D-1/D-2/E-1 sont tech debt non-bloquant,
pas carry formels. T-NN+2/LT-2/LT-3/LT-4 hors cap.

---

## 7. Scope cuts — ce que Sprint 24 NE fait PAS

1. **Ed25519 key rotation ceremony + revocation list** → S25 (~500
   LOC, pas de dep sur B1/A1, resilience DNS plus prioritaire)
2. **C3 handoffs semantic dispatcher** → S25 (~700 LOC, depend de
   B1+A1 stables, stabiliser fondations d'abord)
3. **GuardrailChain cross-process** (rate-limit Rust + PII iframe
   TS) → S26+ (require bridge extension ou IPC)
4. **P2-D-1 redundancy persistence SQLite** → S25 (in-memory OK
   pre-v1.0)
5. **P2-D-2 quarantine curator alerting** → S25 (queue-only OK)
6. **P2-E-1 iroh neighborhood enrichment** → S25
7. **Domain fronting implementation** → S25+ (design doc outline
   S24 Phase E, legal review prerequis)
8. **T-NN+2 iframe Rust-wasm** → PATTERNS §P34 (triggers non actives)
9. **LT-2 Radicle** → trigger tag v1.0
10. **LT-3/LT-4** → post-v1.0

---

## 8. Tracabilite scope — mapping carry S23 → S24

| Item carry | Source | Phase S24 | Status |
|---|---|---|---|
| B1 guardrails refactor | S23 D5 scope cut | Phase B | [x] confirme |
| A1 TaskDispatchHooks | HARDENING §3 S24 | Phase C | [x] confirme |
| Re-run sampling | HARDENING §3 S24 | Phase D | [x] confirme |
| DNS fallback DHT | HARDENING §3 S24 | Phase E | [x] confirme |
| P2-B-1 exponent | audit_findings §P2 | Phase A | [x] confirme |
| P2-C-2 SHA256 doc | audit_findings §P2 | Phase A | [x] confirme |
| P2-F-1 PATTERNS | audit_findings §P2 | Phase A | [x] confirme |
| P2-E-2 pynacl | audit_plan §3 | Phase A | [x] confirme |
| P2-E-3 KudosLedger | audit_plan §3 | Phase A | [x] confirme |
| P2-F-1bis PyO3 doc | verification §5 | Phase A | [x] confirme |
| Key rotation | HARDENING §3 S24 | — | [deferred] → S25 |
| C3 handoffs | HARDENING §3 S24 | — | [deferred] → S25 |

**Cap G7 bilan** : 2/2 carry formels (key rotation + C3 handoffs).

---

## 9. Risk register (R1..R5)

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | B1 retrofit casse les 272 coord tests existants | Medium | High | Contract tests d'abord, retrofit incremental par guardrail, run suite apres chaque retrofit |
| R2 | hickory-resolver incompatibilite iroh transport layer | Low | Medium | DNS fallback est un path additionnel independant d'iroh, pas de conflit deps |
| R3 | Re-run sampling overhead performance (5% = 5 tasks extra / 100) | Low | Low | Taux configurable 1-5%, default 1%. Fire-and-forget hook, pas de blocking |
| R4 | GuardrailChain ordering sensible (PII before canary?) | Medium | Medium | Ordre explicite dans config, doc dans design doc, tests ordering |
| R5 | PyO3 stale wheel bloque tests coord Phase B+ | Medium | High | Phase A documenter rebuild, CI note, tester rebuild local |

---

## 10. Audit gate pattern — rappel

- Phase F produira `sprint24_verification.md` + `sprint24_audit_plan.md`
- Sprint 25 Phase 0 jouera l'audit gate en session fraiche
- Convention permanente depuis Sprint 7

---

## 11. Checkpoint de validation

- [x] Audit gate S23 CONDITIONAL PASS leve (P1 C-1 fixe `34c77ce`)
- [x] G2 trigger check : 1 ACTIVE (openai-agents-python 0.14.3,
      API stable, pas d'impact design B1)
- [x] G6 memory carry-over : items S23 verification §5 deja captures
      dans nexus_grid_pivot.md tip `7b00475`
- [x] G7 cap carry-overs : 2/2 (key rotation + C3 handoffs → S25)
- [x] D1..D5 rediges
- [x] B1 timing : S23 D5 arbitrage Option B → S24 Phase B confirme
- [x] G1 Design Review Board scoring report (0 ❌ + 4 ⚠️ + 1 ✅)
- [x] Acknowledged review findings (4 ⚠️ inline adjustments)
