# Sprint 40 — Kickoff (canary_input migration Tier 2 fin + Tier 3 batch)

**Ecrit** : 2026-04-29 (session fraiche post-audit gate S39 `f8fae0c`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1 : S40 pair).
**Tip master d'entree** : `f8fae0c` (chore(planning) audit findings
S39 PASS).
**Phase 0 audit Sprint 39** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint39_audit_findings.md` (verdict **PASS**,
0 P0/P1, 1 P2 carry confirme [HTTP integration test PII 2/3],
1 P3 nouveau [URL single-quote]).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-29) : HARDENING_ROADMAP last_validated
  `2026-04-29` (S39 CLOSED). 0 trigger actif (meme jour que S39
  close, aucun dep n'a change).

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - frost-ed25519 > 3.0 : stable 3.0 inchange — NOT FIRED
  - openai-agents-python > 0.7.0 : informationnel, pas de dep — NOT FIRED
  - Tous les autres (wasmtime, Tor PoW, NIST PQC, etc.) : NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S40** :
  - `strsim` (crates.io) : deja dep workspace (0.11). Utilise par
    output_filter.rs pour EED. Sera utilise pour Levenshtein
    normalized_similarity dans canary_input.rs (remplace rapidfuzz
    Python).
  - `ed25519-dalek` / `iroh-base` : deja dep transitives workspace
    (via iroh stack). Utilise pour key generation honeypot.rs
    (remplace PyNaCl Python).
  - `sha2` : deja dep transitive workspace. Utilise pour hash
    comparison redundancy.rs (remplace hashlib Python). Alternative :
    `blake3` deja dep directe — mais Python utilise SHA-256, garder
    parite pre-v1.0.
  - `hmac` + `sha2` : deja deps transitives. Utilise par
    watermark_detector.rs PRF score (remplace hmac+hashlib Python).
    Mirror exact du code worker Rust
    `crates/nexus-worker-core/src/llm/watermark.rs`.

- **Roadmap reference** : `.planning/roadmap_v1_migration_rust.md`
  §S40 — "Canary input + redundancy/re-run batch (Tier 2 fin +
  Tier 3)".

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem, LT-4
  biometric : tous requierent tag v1.0 ou condition externe →
  aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 39 CLOSED + audit gate PASS. 3 phases A-C livrees :
- Phase A : PiiRedactor Rust regex-only (7 patterns + Luhn + OnceLock)
- Phase B : CanaryRegistry Rust (observations + freshness + persist JSON)
- Phase C : wire PiiInputGuardrail + 3 routes HTTP canary + P2 launcher RESOLU

Roadmap migration Python→Rust : Tier 1 complet (OutputFilter +
Guardrails + PiiRedactor). Tier 2 debut (CanaryRegistry). Prochaine
etape = Tier 2 fin (canary_input) + Tier 3 (redundancy + rerun +
watermark_detector + honeypot).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-29 (S39 CLOSED). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `f8fae0c`)

| Suite | Count |
|---|---|
| Rust nextest | 991 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1994** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **migre canary_input.py de Python vers Rust** (Tier 2 fin :
injection canari Ed25519 signed + Levenshtein observation + hot-reload
policy) et **migre le batch Tier 3** (redundancy + rerun +
watermark_detector + honeypot) vers des modules Rust natifs dans
nexus-coordinator-rs, avec **dette pair obligatoire** resolvant les
items P2 a 2/3 avant escalade 3/3 MANDATORY.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 39

**DONE** — `f8fae0c`. Verdict PASS (0 P0/P1, 1 P2 carry confirme +
1 P3). Cf. `.planning/archive/v1.2/sprint39_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — canary_input.py migration : port direct avec adaptations ciblees

**Retenu** : porter `canary_input.py` (783 LOC) vers un module
`canary_input.rs` dans nexus-coordinator-rs. Strategie :

(a) **CanaryInputSet** : struct serde avec 5 prompts seed default.
    Signature Ed25519 via `nexus-core-rs::sign_bytes` /
    `verify_bytes` natif (pas PyO3). Format JSON `signable_json()`
    identique au Python (sort_keys canonique).
(b) **CanaryInputInjector** : sampling round-robin 1/N thread-safe
    (Mutex ou AtomicUsize counter). Injecte des prompts canari dans
    le flux de taches.
(c) **CanaryInputObserver** : post-result hook, compare reponse
    worker vs expected via `strsim::normalized_levenshtein()` (dep
    workspace existante, remplace `rapidfuzz`). Ring buffer borne
    pour divergences.
(d) **CanaryInputPolicy** : deserialise depuis TOML. Hot-reload
    pattern identique a OutputFilter/PowPolicyLoader (mtime
    debounce, Arc swap).
(e) **CanaryInputGuardrail** adapter : impl Guardrail trait,
    direction Input, mode **Tripwire** (pas Mutation). Le Python
    original mute le prompt (remplace par canary), le Rust signale
    via Tripwire que la tache est un canary — le dispatcher
    upstream gere l'injection. Ce choix est coherent avec
    PiiInputGuardrail (S39) et le trait Guardrail actuel qui ne
    supporte pas la mutation.

**Rejete** :
- Mutation guardrail (modifier le trait Guardrail pour supporter
  `Mutate` en plus de `Pass`/`Tripwire`) : S40 n'est pas le bon
  sprint pour refactorer le trait. Le carry P2-REVIEW-A-1-S39
  track ce refactor pour post-v1.0.
- Port avec dep `rapidfuzz-rs` (binding Rust rapidfuzz) : crate
  immature, `strsim` est standard et deja dans le workspace.
- Separation en crate dedie `nexus-canary-input` : 783 LOC Python
  trop petit pour un crate separe.

**Implications code** :
- `crates/nexus-coordinator-rs/src/canary_input.rs` (NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (pub mod)
- `crates/nexus-coordinator-rs/Cargo.toml` (+sha2 si necessaire)
- Tests : 10-14 tests (set/sign/verify + injector + observer +
  policy + guardrail)

### D2 — Tier 3 batch : 4 modules port direct

**Retenu** : porter les 4 modules Tier 3 vers des modules Rust
dans nexus-coordinator-rs :

(a) **redundancy.rs** : vote
    majorite SHA-256 hash comparison. `RedundancyDispatcher` struct
    in-memory. SHA-256 via crate `sha2` (deja transitive workspace)
    pour parite wire format Python. Post-v1.0 migration BLAKE3
    possible.
(b) **watermark_detector.rs** :
    z-test binomial SynthID-inspired PRF detection. Mirror exact
    du code worker `crates/nexus-worker-core/src/llm/watermark.rs`
    cote detection. HMAC-SHA256 via `hmac` + `sha2`.
(c) **rerun.rs** : spot-check
    re-dispatch. `RerunSampler` sampling + `DivergenceScorer` hash
    comparison. Le pattern Python `DispatchHook` async est remplace
    par un trait Rust `ResultHook` synchrone avec callback. Query
    DB via `rusqlite` (deja dep directe coordinator-rs).
(d) **honeypot.rs** : eclipse
    detection via canary peers ephemeres. Key generation Ed25519
    via `ed25519-dalek` (dep transitive iroh-base). `EclipseDetector`
    streak tracker + `CanaryRotationScheduler` cadence 6h.

**Rejete** :
- Un seul module fourre-tout `integrity.rs` pour les 4 : trop
  different fonctionnellement (vote vs detection vs honeypot).
  4 modules separes = testabilite + lisibilite.
- Crates separes par module : < 200 LOC chacun, trop petit.
- Port async Rust (tokio channels) pour rerun : le pattern
  coordinator actuel est synchrone (rusqlite, pas async). Ajouter
  tokio overhead = overengineering.

**Implications code** :
- `crates/nexus-coordinator-rs/src/{redundancy,watermark_detector,rerun,honeypot}.rs` (4 NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (+4 pub mod)
- `crates/nexus-coordinator-rs/Cargo.toml` (+sha2 +hmac deps
  directes si pas deja via workspace)
- Tests : 12-16 tests (vote/quarantine + z-test/threshold +
  sampler/scorer + eclipse/rotation)

### D3 — Dette pair : 5 items P2/P3 a 2/3

**Retenu** : resoudre les 5 items a 2/3 dans la Phase A dette
pour eviter l'escalade 3/3 MANDATORY en S41 :

(a) **P2-REVIEW-A-1-S38** result_event_tx dead code (2/3) :
    le champ `result_event_tx` dans DaemonHttpState est cree mais
    jamais utilise par les handlers HTTP. Wire dans le handler
    `coordinator_submit_result` pour envoyer un `ResultEvent` quand
    un result est valide. Cela complete le pipeline
    validator_loop ← HTTP.
(b) **P2-REVIEW-B-1-S38** substring O(n*m) (2/3) : ajouter
    early exit sur premier match dans `check_prompt_echo_substring`
    (actuellement continue l'iteration apres un match). Le
    `DEFAULT_SUBSTRING_MIN_LEN=40` existant est deja suffisant.
    L'algorithme reste quadratique worst-case mais le cas moyen
    est reduit significativement.
(c) **P2-REVIEW-C-1-S38** chain Arc singleton (2/3) : stocker
    `Arc<GuardrailChain>` dans `DaemonHttpState` au lieu de
    recreer `default_*_chain()` par requete. Les regex sont deja
    compile-once (OnceLock) mais la Box alloc chain est par-requete.
(d) **P2-REVIEW-C-1-S39** HTTP integration tests (2/3) : ajouter
    3 tests dans `http.rs` : (1) submit_task avec email dans
    prompt → 400 rejected, (2) canary_observed POST → 200,
    (3) canary_network_health GET → 200.
(e) **P3-AUDIT-A-2b-S38** lowercase divergence (2/3) : documenter
    la convention dans PATTERNS.md (Python lowercase vs Rust
    case-sensitive pour les identifiants wire).

**Rejete** :
- Reporter les items 2/3 a S41 en acceptant l'escalade 3/3
  MANDATORY : S41 est deja charge (Tier 4 infra batch).
  Mieux vaut liquider la dette maintenant.
- Aho-Corasick pour substring (dep `aho-corasick`) : overengineering
  pour un fix pre-v1.0. Le minimum length + early exit suffit.

**Implications code** :
- `crates/nexus-shell-daemon/src/http.rs` (wire result_event_tx +
  3 tests + chain singleton)
- `crates/nexus-shell-daemon/src/runtime.rs` (chain init)
- `crates/nexus-coordinator-rs/src/output_filter.rs` (substring fix)
- `crates/nexus-coordinator-rs/src/guardrails.rs` (Arc chain)
- `docs/rust/PATTERNS.md` (lowercase convention)

### D4 — P3-grammar/watermark 3/3+ resolution via Tier 3

**Retenu** : les carries P3-grammar executor (3/3+) et P3-watermark
executor (3/3+) sont resolus par la migration Tier 3 Phase C :

- P3-grammar → la pipeline re-run Rust (`rerun.rs`) remplace le
  scaffold Python pour la validation grammar via re-dispatch.
- P3-watermark → le detecteur Rust (`watermark_detector.rs`)
  remplace le scaffold Python pour la verification watermark.

Pas de phase dediee supplementaire — l'integration dans le Tier 3
batch est naturelle.

**Rejete** :
- Phase dediee grammar/watermark separee du Tier 3 : duplication
  de scope (les modules Python correspondants SONT le Tier 3).

### D5 — Scope cuts S40

1. **Wire canary_input HTTP routes** — S41 (Tier 4, les routes
   API canary_input sont dans `api/canary.py` 212 LOC, Tier 5 S43)
2. **Wire rerun/redundancy dans dispatcher** — S41 (le dispatcher
   Rust n'a pas encore le hook framework)
3. **canary_input gossip sync** — post-v1.0 (Niveau 2)
4. **Quarantine queue Rust** — S41 (Tier 4, 369 LOC)
5. **Upload queue Rust** — S41 (Tier 4, 396 LOC)
6. **Migration routes API** — S42-44 (Tier 5)
7. **Suppression coordinator Python** — S45
8. **CI multi-OS release** — S46
9. **VPS deployment** — S47
10. **Tag v1.0** — S48
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **CanaryInput mutation guardrail** — post-v1.0 (P2-REVIEW-A-1-S39)

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5).

D2 ⚠️ (1 finding) :
- SHA-256 parite wire Python non documentee dans kickoff.
  **Accept** : `redundancy.py` utilise bien `hashlib.sha256`
  (confirme par lecture code, L104-105 `hashlib.sha256(result_text
  .encode()).hexdigest()`). SHA-256 choisi pour parite wire format.
  Post-v1.0 migration BLAKE3 possible.

D3 ⚠️ (2 findings) :
- Substring min_len : kickoff dit "skip < 8 chars" mais code a
  `DEFAULT_SUBSTRING_MIN_LEN=40`. **Accept — corrige** : le fix
  Phase A est l'**early exit sur premier match**, pas le min_len
  (deja a 40, suffisant). Description D3b mise a jour.
- Chain Arc singleton non-impl : **Accept** — c'est precisement
  le but de Phase A item (c).

---

## §5 Plan Phase outline A..D

### Phase A — Dette pair MANDATORY

**But** : resoudre 5 items dette P2/P3 a 2/3 avant escalade.
- P2-REVIEW-A-1-S38 result_event_tx wire submit_result
- P2-REVIEW-B-1-S38 substring min length + early exit
- P2-REVIEW-C-1-S38 chain Arc singleton DaemonHttpState
- P2-REVIEW-C-1-S39 HTTP integration tests (+3 tests)
- P3-AUDIT-A-2b-S38 lowercase divergence doc
- Commit : `feat(sprint40): Sprint 40 Phase A — dette pair P2 batch
  2/3 items + HTTP integration tests`

### Phase B — canary_input.py migration (Tier 2 fin)

**But** : migrer canary_input.py (783 LOC) vers Rust.
- CanaryInputSet + CanaryPrompt structs serde
- CanaryInputInjector + Observer + Policy + Manager
- CanaryInputGuardrail adapter (Tripwire)
- Levenshtein via strsim, Ed25519 via nexus-core-rs
- Tests : 10-14 tests couvrant chaque composant
- Commit : `feat(sprint40): Sprint 40 Phase B — CanaryInput Rust
  canary_input.rs`

### Phase C — Tier 3 batch migration

**But** : migrer les 4 modules Tier 3 vers Rust.
- redundancy.rs (vote majorite SHA-256)
- watermark_detector.rs (z-test SynthID PRF)
- rerun.rs (spot-check re-dispatch + divergence scorer)
- honeypot.rs (eclipse detection canary peers)
- Resolves P3-grammar + P3-watermark 3/3+
- Tests : 12-16 tests couvrant les 4 modules
- Commit : `feat(sprint40): Sprint 40 Phase C — Tier 3 batch
  redundancy + watermark + rerun + honeypot Rust`

### Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint41_audit_plan.md
- SPRINT_LOG.md row S40
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S40
- Migration `.planning/active/sprint40_*` → `.planning/archive/v1.2/`
  (sauf sprint41 files)
- Commit : `chore(sprint40): Phase D — wrap-up + verification
  + audit plan S41 + migration`

---

## §6 Items carry/dette

### Resolus S40 (plan)

- [plan] canary_input.py migration : Phase B
- [plan] redundancy.py migration : Phase C
- [plan] rerun.py migration : Phase C
- [plan] watermark_detector.py migration : Phase C
- [plan] honeypot.py migration : Phase C
- [plan] P3-grammar executor 3/3+ : Phase C (rerun.rs)
- [plan] P3-watermark executor 3/3+ : Phase C (watermark_detector.rs)
- [plan] P2-REVIEW-A-1-S38 result_event_tx 2/3 : Phase A
- [plan] P2-REVIEW-B-1-S38 substring 2/3 : Phase A
- [plan] P2-REVIEW-C-1-S38 chain singleton 2/3 : Phase A
- [plan] P2-REVIEW-C-1-S39 HTTP integration tests 2/3 : Phase A
- [plan] P3-AUDIT-A-2b-S38 lowercase divergence 2/3 : Phase A

### Carries confirmes S41

- [carry] P2-A-1 rand blocker upstream 6+/3 : blocker externe
  inchange. Exemption §6.2.1 blocker externe.
- [carry] P2-AUDIT-2-S35 pre-release transitives iroh : condition
  heritee pin 0.98.
- [carry] P2-REVIEW-A-1-S39 Tripwire vs Mutation 1/3 : trait
  extension post-v1.0.
- [carry] P2-REVIEW-B-1-S39 warn threshold 1/3 : seuil cadence
  post-v1.0.
- [carry] P3-REVIEW-A-2-S39 LOC kickoff 1/3 : cosmetic.
- [carry] P3-REVIEW-B-2-S39 persist error silent 1/3 : robustness
  post-v1.0.
- [carry] P3-AUDIT-A-1-S39 URL single-quote 1/3 : cosmetic.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **Wire canary_input HTTP routes** — S41+ (Tier 4/5)
2. **Wire rerun/redundancy dispatcher** — S41 (hook framework)
3. **canary_input gossip sync** — post-v1.0
4. **Quarantine queue Rust** — S41 (Tier 4)
5. **Upload queue Rust** — S41 (Tier 4)
6. **Migration routes API** — S42-44 (Tier 5)
7. **Suppression coordinator Python** — S45
8. **CI multi-OS release** — S46
9. **VPS deployment** — S47
10. **Tag v1.0** — S48
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **CanaryInput mutation guardrail** — post-v1.0

---

## §8 Tracabilite scope (S39 → S40)

| Item S39 carry / scope cut | Ou dans S40 |
|---|---|
| SC-3 canary_input.py | §5 Phase B (Tier 2 fin) |
| SC-4 redundancy/re-run/honeypot | §5 Phase C (Tier 3 batch) |
| P3-grammar executor 3/3+ | §5 Phase C (rerun.rs) |
| P3-watermark executor 3/3+ | §5 Phase C (watermark_detector.rs) |
| P2-REVIEW-A-1-S38 dead code 2/3 | §5 Phase A dette |
| P2-REVIEW-B-1-S38 substring 2/3 | §5 Phase A dette |
| P2-REVIEW-C-1-S38 chain singleton 2/3 | §5 Phase A dette |
| P3-AUDIT-A-2b-S38 lowercase 2/3 | §5 Phase A dette |
| P2-REVIEW-C-1-S39 HTTP tests 2/3 | §5 Phase A dette |
| P2-REVIEW-A-1-S39 Tripwire vs Mutation 1/3 | §6 carry S41 |
| P2-REVIEW-B-1-S39 warn threshold 1/3 | §6 carry S41 |
| SC-5/6/7/8/9 Migration routes → v1.0 | §7 scope cuts inchanges |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | canary_input.py 783 LOC = plus gros module a migrer S40, debordement Phase B | Medium | Module bien structure (5 classes principales), patterns migration etablis S38-S39. Hot-reload pattern deja livre 3 fois. |
| R2 | strsim::normalized_levenshtein vs rapidfuzz divergence numerique | Low | Meme algorithme (edit distance / max(len1, len2)), tolerance 0.85 large. Tests de parite croises. |
| R3 | rerun.py depend du framework DispatchHook Python, port Rust non-trivial | Medium | Simplification : trait ResultHook synchrone + rusqlite direct. Pas de framework async comme en Python. |
| R4 | 4 modules Tier 3 en une seule phase = risque debordement | Medium | Modules individuellement petits (120-222 LOC chacun). Port direct sans adaptation pour 2/4 (redundancy, watermark). |
| R5 | honeypot key generation Ed25519 via ed25519-dalek vs PyNaCl divergence | Low | Meme spec Ed25519. Les clefs generees sont ephemeres (6h rotation), pas de persistence. |

---

## §10 Audit gate pattern — rappel

Phase 0 (audit S39) deja jouee. Phase D devra produire
`sprint41_audit_plan.md` pour la session suivante.

---

## §11 Checkpoint de validation

1. **D1** : canary_input port Tripwire (pas Mutation), acceptable ?
   → recommandation : oui, coherent avec PiiInputGuardrail S39.
   Mutation guardrail = refactor trait post-v1.0.
2. **D2** : 4 modules Tier 3 en une seule phase, faisable ?
   → recommandation : oui, 2/4 sont des ports triviaux (< 120 LOC),
   les 2 autres necessitent adaptation mineure.
3. **D3** : 5 items dette en Phase A, charge acceptable ?
   → recommandation : oui, tous < 100 LOC, la plus grosse piece
   est les 3 tests HTTP.
4. **D4** : P3-grammar/watermark resolution via Tier 3, pas de
   phase dediee ?
   → recommandation : oui, les modules Python SONT le Tier 3.
   Resolution naturelle.
5. **D5** : Wire canary_input routes differe S41+, acceptable ?
   → recommandation : oui, les routes HTTP sont dans api/canary.py
   (Tier 5 S43). Le module Rust est la logique metier, les routes
   viendront avec la migration API.
