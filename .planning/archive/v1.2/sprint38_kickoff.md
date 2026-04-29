# Sprint 38 — Kickoff (validator_loop MANDATORY + OutputFilter/Guardrails Rust migration Tier 1)

**Ecrit** : 2026-04-29 (session fraiche post-audit gate S37 `d2eb4c0`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1 : S38 pair).
**Tip master d'entree** : `d2eb4c0` (chore(planning) audit findings
S37 PASS).
**Phase 0 audit Sprint 37** : **DEJA JOUE** — findings dans
`.planning/active/sprint37_audit_findings.md` (verdict **PASS**,
0 P0/P1, 1 P2 [Cargo.lock desync fixe `c727f74`], 1 P3
[rowid documentation carry]).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-29) : HARDENING_ROADMAP last_validated
  `2026-04-29` (S37 CLOSED). 0 trigger actif.

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - wasmtime LTS bump : pas de dep directe — INACTIVE
  - frost-ed25519 > 3.0 : stable 3.0 inchange — NOT FIRED
  - Tor PoW spec, NIST PQC, RFC 9591, openai-agents, MCP spec,
    microsoft/sudo, NVIDIA H100 CCM : tous NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S38** :
  - `iroh-docs 0.98` : deja workspace dep. CuratorRuntimeHandle
    expose DashMap snapshots mais pas de LiveEvents stream.
    context7 confirme pattern Watcher (stream asynchrone sur
    changement, cancel-safe). API ciblee : `Doc::subscribe()`
    ou equivalent 0.98.
  - `strsim 0.11` : nouvelle dep workspace pour edit distance
    (Levenshtein). MIT, pure Rust, 0 dep transitive. Alternative
    `edit-distance` (plus simple mais moins complet) et
    `rapidfuzz` Rust binding (overhead FFI). `strsim` retenu :
    API la plus complete (levenshtein, jaro_winkler, osa_distance),
    pure Rust, 0 dep.
  - `toml 0.8` : deja dans le workspace (config hot-reload
    pattern existant dans pow_policy_loader.rs et output_filter
    Python).

- **Roadmap reference** : `.planning/roadmap_v1_migration_rust.md`
  §S38 — "validator_loop MANDATORY + OutputFilter Rust (Tier 1
  part 1)".

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem, LT-4
  biometric, LT-5 redundancy : tous requierent tag v1.0 ou
  condition externe → aucun declenche. LT-6 : RESOLVED (S32).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 37 CLOSED. 2 phases A-B livrees + Phase C wrap-up :
- Phase A : 2 MANDATORY fermes (log convergence + .icns) +
  5 P2 batch audit/review S36
- Phase B : KudosLedger hash-chain BLAKE3+JCS canonical
  per-project (prev_hash + entry_hash + verify_chain)

Audit gate S37 : **PASS** (0 P0/P1, 1 P2 [Cargo.lock desync
fixe `c727f74`], 1 P3 [rowid doc carry 2/3]).

Roadmap migration Python→Rust : core path livre (dispatcher +
validator + kudos_ledger, ~1276 LOC Rust, 3 endpoints HTTP live).
Prochaine etape = Tier 1 part 1 (OutputFilter + Guardrails).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-29 (S37 CLOSED). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `d2eb4c0`)

| Suite | Count |
|---|---|
| Rust nextest | 946 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1949** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **ferme le MANDATORY validator_loop tokio 3/3** (refactor
CuratorRuntimeHandle LiveEvents), **migre OutputFilter + Guardrails
pipeline de Python vers Rust** (Tier 1 part 1 roadmap migration),
et **absorbe la dette pair S38** (P2 batch carries S37).
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 37

**DONE** — `d2eb4c0`. Verdict PASS (0 P0/P1, 1 P2 fixe + 1 P3).
Cf. `.planning/active/sprint37_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — validator_loop : tokio task + LiveEvents channel

**Retenu** : refactorer CuratorRuntimeHandle pour exposer un
channel `tokio::sync::broadcast` de LiveEvents depuis le Doc
handle iroh-docs. Spawner un `validator_loop` tokio task dans
le daemon runtime qui :
(a) Subscribe aux events du document de resultats
(b) Filtre les events de type ContentReady (nouveau resultat)
(c) Deserialise le ResultEntry depuis le blob
(d) Appelle `validate_result()` + `kudos_ledger::credit()` si Accepted
(e) Log les rejections avec tracing

Le HTTP POST `/api/v1/results/submit` reste comme chemin alternatif
(fallback, tests, clients externes). Le validator_loop est le
chemin principal event-driven.

**Rejete** :
- Exposer `Arc<Doc>` directement (leak internal, couplage fort
  coordinator-rs ↔ iroh-docs).
- WebSocket endpoint daemon→coordinator (les deux tournent dans
  le meme process, overhead inutile).
- Polling HTTP periodique (latence, CPU waste, pas event-driven).
- Ne rien faire (MANDATORY 3/3, interdit de re-reporter).

**Implications code** :
- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` (expose
  LiveEvents broadcast)
- `crates/nexus-shell-daemon/src/runtime.rs` (spawn validator_loop)
- `crates/nexus-shell-daemon/src/http.rs` (DaemonHttpState extension)
- Tests : mock LiveEvent + validation chain

### D2 — OutputFilter Rust : port direct output_filter.py

**Retenu** : porter `output_filter.py` (397 LOC) vers un module
`output_filter.rs` dans nexus-coordinator-rs. Port direct des
3 layers :
(a) Invisible text scanner : strip Unicode zero-width chars
    (U+200B-200F, U+2060, U+FEFF), Private Use Area
    (U+E000-F8FF, Planes 15-16), Tag chars (U+E0020-E007F).
    Whitelist bidi format chars (U+202A-E, U+2066-69) pour i18n.
(b) Prompt echo cascade : exact match → substring (40+ char
    slices) → EED (edit distance / Levenshtein similarity >=0.85).
(c) FilterVerdict struct : is_valid, reason enum, risk_score,
    sanitized_output.

Dep nouvelle : `strsim = "0.11"` (Levenshtein, MIT, pure Rust).

Policy hot-reload depuis `output_filter_policy.toml` : pattern
identique a `pow_policy_loader.rs` (read + deser + fallback default).

**Rejete** :
- ML-based filter Rust (ONNX runtime Rust pas stable pour browser,
  overkill pour les checks pre-v1.0 qui sont regex+Unicode).
- Regex-only sans EED (rate les prompt echoes fuzzy — le Python
  utilise EED pour bonne raison).
- Garder en Python (objectif roadmap = suppression coordinator
  Python, Tier 1 = ce sprint).
- `edit-distance` crate (moins complet que strsim, pas de
  normalized distance).

**Implications code** :
- `crates/nexus-coordinator-rs/Cargo.toml` (+strsim dep)
- `crates/nexus-coordinator-rs/src/output_filter.rs` (NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (pub mod)
- Tests : invisible text, prompt echo exact/substring/EED, policy

### D3 — Guardrails pipeline Rust : trait + chain

**Retenu** : porter `guardrails.py` (137 LOC) vers un module
`guardrails.rs` dans nexus-coordinator-rs. Design :
(a) `Guardrail` trait : `name()`, `direction()` (Input/Output),
    `check(&self, ctx, value) -> GuardrailOutcome`.
(b) `GuardrailChain` : sequence ordonnee, short-circuit sur
    Tripwire outcome.
(c) `GuardrailOutcome` enum : Pass / Flag(reason) / Tripwire(reason).
(d) `OutputSafetyGuardrail` : adapter qui wrap OutputFilter et
    implemente le trait Guardrail.

Le pipeline est wire dans le handler `coordinator_submit_result`
APRES `validate_result()` Accepted et AVANT `kudos_ledger::credit()`.
Un resultat qui tripwire le guardrail = pas de credit kudos.

**Rejete** :
- Async trait (les checks sont CPU-bound et synchrones, pas d'IO).
- Crate separe (137 LOC Python → ~150 LOC Rust, trop petit pour
  justifier un crate).
- Middleware axum (le check s'applique au contenu du resultat
  deserialise, pas au transport HTTP).

**Implications code** :
- `crates/nexus-coordinator-rs/src/guardrails.rs` (NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (pub mod)
- `crates/nexus-shell-daemon/src/http.rs` (wire dans submit_result)
- Tests : chain pass-through, tripwire short-circuit, flag accumulate

### D4 — P2 batch dette pair (Phase A)

**Retenu** : resoudre les 3 items dette en Phase A :
- P2-REVIEW-B-1-S37 rowid documentation 2/3 : ajouter commentaire
  inline dans db.rs expliquant l'invariant rowid tiebreaker +
  section dans docs/shell/PATTERNS.md
- P2-REVIEW-A-1-S37 launcher logging test 1/3 : ajouter un test
  qui verifie que `launcher_log_dir()` retourne le meme path que
  `paths::log_dir()` (coherence daemon/launcher)
- SC-12-S37 verify_chain endpoint HTTP : ajouter
  `GET /api/v1/kudos/{project_id}/verify` qui appelle
  `verify_chain()` et retourne `{valid: bool}`

**Rejete** :
- Differer rowid doc a S39 (atteindrait 3/3 MANDATORY, interdit).
- Differer verify_chain endpoint (explicitement planifie S38 dans
  S37 SC-12).

**Implications code** :
- `crates/nexus-coordinator-rs/src/db.rs` (commentaires rowid)
- `docs/shell/PATTERNS.md` (section rowid)
- `crates/nexus-launcher/src/main.rs` (1 test)
- `crates/nexus-shell-daemon/src/http.rs` (1 route + handler)

### D5 — Scope cuts S38

Liste exhaustive des items NON inclus dans S38 :

1. **PiiRedactor Rust** — S39 (Tier 1 part 2, dep ONNX a evaluer)
2. **CanaryRegistry Rust** — S39 (Tier 2 debut)
3. **Migration complete coordinator** — S41+ (Tier 4 jalon)
4. **Suppression coordinator Python** — S45
5. **CI pipeline multi-OS release** — S46
6. **VPS deployment** — S47
7. **Tag v1.0** — S48
8. **Code signing macOS** — post-v1.0
9. **P3 grammar/watermark executor** — S40 (Tier 3, integre re-run)
10. **SDK Python rewrite** — hors-scope
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **Validator loop consumer coordinator Python** — le validator_loop
    tourne cote daemon Rust, pas besoin de wire le coord Python

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (3 ⚠️ + 2 ✅ sur 5).

D1 ⚠️ (1 finding) :
- iroh-docs 0.98 LiveEvents API non prouvee via context7 (docs
  retournees concernent version plus recente). **Accept** : R1
  anticipe le gap. Phase A G8 preflight S1b verifiera l'API
  exacte dans le source iroh-docs 0.98 avant code.

D2 ⚠️ (2 findings) :
- edit-distance vs strsim non compare quantitativement.
  **Accept** : strsim expose `normalized_levenshtein()` (range
  0.0-1.0), edit-distance expose raw count seulement — le choix
  est correct pour le seuil 0.85. Pas de benchmark necessaire.
- rapidfuzz FFI overhead non benchmark. **Accept** : le rejet
  est architectural (pure Rust vs FFI binding), pas performance.
  R2 couvre la divergence numerique, pas la latence.

D3 ⚠️ (1 finding) :
- trait sync-only, S39 PiiRedactor pourrait etre async (ONNX).
  **Accept** : pas de sur-design. Le trait sync est correct pour
  S38. Si S39 requiert async, refactor signature trait a ce moment
  (1 changement de signature, backward-compat via wrapper sync).
  Documente dans le code.

---

## §5 Plan Phase outline A..D

### Phase A — dette pair : MANDATORY validator_loop + P2 batch

**But** : fermer le MANDATORY 3/3 + 3 items P2 dette.
- validator_loop tokio : CuratorRuntimeHandle expose broadcast,
  daemon spawne task, validate + credit event-driven
- rowid documentation : inline comment db.rs + PATTERNS section
- launcher logging test : coherence log_dir test
- verify_chain HTTP endpoint : GET route + handler
- Commit : `feat(sprint38): Sprint 38 Phase A — MANDATORY
  validator_loop tokio + dette pair P2 batch`

### Phase B — OutputFilter Rust migration

**But** : output_filter.py → output_filter.rs (Tier 1 part 1).
- Invisible text scanner (Unicode categories)
- Prompt echo cascade (exact + substring + EED strsim)
- FilterVerdict struct + reason enum
- Policy hot-reload TOML
- Tests : 8-10 tests couvrant chaque layer
- Commit : `feat(sprint38): Sprint 38 Phase B — OutputFilter
  Rust migration output_filter.rs`

### Phase C — Guardrails pipeline Rust + wire

**But** : guardrails.py → guardrails.rs + wire submit_result.
- Guardrail trait + GuardrailChain
- OutputSafetyGuardrail adapter
- Wire dans coordinator_submit_result handler
- Wire dans validator_loop
- Tests : 4-6 tests chain/tripwire/flag
- Commit : `feat(sprint38): Sprint 38 Phase C — Guardrails
  pipeline Rust + wire submit_result`

### Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint39_audit_plan.md
- SPRINT_LOG.md row S38
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S38
- Migration `.planning/active/sprint37_*.md` → `.planning/archive/v1.2/`
- Commit : `chore(sprint38): Phase D — wrap-up + verification
  + audit plan S39 + migration`

---

## §6 Items carry/dette

### Resolus S38 (plan)

- [x] P2-REVIEW-C-1-S35 validator_loop tokio 3/3 **MANDATORY** : Phase A
- [x] P2-REVIEW-B-1-S37 rowid documentation 2/3 : Phase A
- [x] P2-REVIEW-A-1-S37 launcher logging test 1/3 : Phase A
- [x] SC-12-S37 verify_chain endpoint HTTP : Phase A

### Carries confirmes S39

- [carry] P2-A-1 rand blocker upstream 6+/3 : blocker externe
  inchange (frost-core rand_core 0.6 + iroh stack disjoints). Pas
  de convergence observee. Exemption §6.2.1 blocker externe.
- [carry] P2-AUDIT-2-S35 pre-release transitives iroh : condition
  heritee pin 0.98.

### MANDATORY evalues — DEFER justifie

- P3-grammar executor 3/3+ : **DEFER** — S40 Tier 3 integre dans
  re-run/redundancy migration. Exemption dependance sequentielle
  interne.
- P3-watermark executor 3/3+ : **DEFER** — S40 Tier 3 meme
  justification.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **PiiRedactor Rust** — S39 (Tier 1 part 2)
2. **CanaryRegistry Rust** — S39 (Tier 2 debut)
3. **Migration routes API** — S42-44 (Tier 5)
4. **Suppression coordinator Python** — S45
5. **CI multi-OS release** — S46
6. **VPS deployment** — S47
7. **Tag v1.0** — S48
8. **Code signing macOS** — post-v1.0
9. **P3 grammar/watermark** — S40 (Tier 3)
10. **SDK Python rewrite** — hors-scope
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **Coordinator Python consumer** — pas de wire validator_loop
    cote Python (Rust-only path)

---

## §8 Tracabilite scope (S37 → S38)

| Item S37 carry / scope cut | Ou dans S38 |
|---|---|
| P2-REVIEW-C-1-S35 validator_loop 3/3 MANDATORY | §5 Phase A |
| P2-REVIEW-B-1-S37 rowid documentation 2/3 | §5 Phase A |
| P2-REVIEW-A-1-S37 launcher logging test 1/3 | §5 Phase A |
| SC-12 verify_chain endpoint HTTP | §5 Phase A |
| SC-1 Migration complete coordinator | §7.3 scope cut S42-44 |
| SC-3 OutputFilter/PiiRedactor Rust | §5 Phase B (OutputFilter) + §7.1 (Pii S39) |
| SC-4 CanaryRegistry Rust | §7.2 scope cut S39 |
| SC-5 Validator loop LiveEvents | §5 Phase A — RESOLVING |
| SC-12 verify_chain endpoint HTTP | §5 Phase A — RESOLVING |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | iroh-docs 0.98 API LiveEvents differe de la doc context7 (version plus recente) | Medium | Verifier API dans le code source iroh-docs 0.98 avant Phase A. Fallback : channel manuel via DashMap watch. |
| R2 | strsim normalized_levenshtein diverge de rapidfuzz Python (seuils EED differents) | Low | Tests comparatifs avec les memes inputs que les tests Python existants. Ajuster seuil si necessaire. |
| R3 | OutputFilter policy TOML format diverge Python ↔ Rust | Low | Le format TOML est identique (meme schema). Le coordinator Python et Rust ne lisent pas le meme fichier simultanement. |
| R4 | validator_loop + HTTP POST double-process un resultat | Medium | Guard idempotent : `set_task_result()` retourne false si deja completed (WHERE status IN ('pending', 'dispatched')). Le 2eme path est noop. |
| R5 | Wire guardrails dans submit_result casse les tests existants | Low | Les tests existants n'ont pas de content a filtrer (stubs). Ajouter guardrails avec policy permissive par defaut. |

---

## §10 Audit gate pattern — rappel

Phase 0 (audit S37) deja jouee. Phase D devra produire
`sprint39_audit_plan.md` pour la session suivante.

---

## §11 Checkpoint de validation

1. **D1** : le validator_loop tokio est le bon chemin pour fermer
   le MANDATORY ? (vs. exposer seulement l'API et differ le loop
   a S39) → recommandation : loop complet, c'est un MANDATORY 3/3
2. **D2** : strsim 0.11 pour EED, ou impl maison de Levenshtein ?
   → recommandation : strsim (MIT, 0 dep, mature)
3. **D3** : wire guardrails dans submit_result ET validator_loop,
   ou seulement submit_result ? → recommandation : les deux
   (meme chemin de validation, DRY)
4. **D4** : verify_chain endpoint en Phase A dette, ou Phase C
   avec les guardrails ? → recommandation : Phase A (trivial,
   ~15 LOC, libere SC-12)
5. **D5** : PiiRedactor differe S39, acceptable ? → recommandation :
   oui, dep ONNX Rust a evaluer d'abord (R&D S39 kickoff)
