# Sprint 35 — Kickoff (migration Rust native — Phase 1 fondations)

**Ecrit** : 2026-04-28 (session fraiche post-audit gate S34 `2a79c8e`).
**Type** : **sprint impair feature** — pas de phase dette obligatoire
(§6.2.1 Regle 1 : S35 impair). 3 MANDATORY 3/3 a resoudre + debut
migration coordinator Python → Rust natif.
**Tip master d'entree** : `2a79c8e` (chore: gitignore external agent
framework artifacts — post-audit S34).
**Phase 0 audit Sprint 34** : **DEJA JOUE** — findings dans
`.planning/active/sprint34_audit_findings.md` (verdict **PASS**,
0 P0/P1, 2 P2 fixes inline, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-28) : HARDENING_ROADMAP last_validated
  `2026-04-28` (S34 Phase D).

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 (0.98.1 patch dans range) — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - frost-ed25519 > 3.0 : 3.0.0 deja deploye S34 — SATISFAIT, trigger mis a jour > 3.0
  - wasmtime LTS bump : pas de dep directe — INACTIVE
  - Tous les autres triggers (RFC 9591 erratum, NIST PQC, Tor PoW, MCP spec) : NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Rust coordinator migration research** (2026-04-28) :
  3 agents paralleles (G2 triggers, carry analysis, Rust-native
  coordinator migration) :
  - Coordinator Python = 16 APIRouter modules (thin routing <100 LOC chacun)
    + business logic services : Dispatcher (359 LOC), Validator (356 LOC),
    KudosLedger (344 LOC), OutputFilter/PiiRedactor (~880 LOC),
    CanaryRegistry (~782 LOC), QuarantineQueue (~765 LOC)
  - Crypto primitives **deja en Rust** via PyO3 (sign_task, verify_result,
    build_canary, etc.) — le coordinator ne fait que les appeler
  - Axum HTTP server **deja en production** dans nexus-shell-daemon
    (8 routes, middleware auth + CORS + CSP)
  - Cibles tier 1 migration : Dispatcher + Validator + KudosLedger
    (~1060 LOC Python → Rust natif, elimine le round-trip PyO3)
  - SDK Python (`nexus-sdk`) reste Python pour compatibilite language binding

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  Tous les triggers (LT-1 Gini, LT-2 Radicle, LT-3 app ecosystem)
  requierent tag v1.0 → aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 34 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : dette MANDATORY (rand blocker upstream + COEP E2E + frost 3.0)
- Phase B : Windows launcher UX (icon + subsystem + file logging)
- Phase C : macOS .app bundle + Linux .desktop integration
- Phase D : wrap-up + fix running.json path

Audit gate S34 : **PASS** (0 P0/P1, 2 P2 fixes inline
[CREATE_NO_WINDOW + .gitignore], 1 P3).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-28 (S34 Phase D). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `2a79c8e`)

| Suite | Count |
|---|---|
| Rust nextest | 902 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1905** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint pose les **fondations de la migration coordinator Python →
Rust natif** en portant le dispatcher de taches et le validateur
de resultats dans un nouveau crate `nexus-coordinator-rs`, tout en
fermant les 3 items MANDATORY 3/3 (shellcheck CI + cross-daemon E2E +
REPO_URL).
**Critere SMART : 30+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 34

**DONE** — `2a79c8e`. Verdict PASS (0 P0/P1, 2 P2 + 1 P3).
Cf. `.planning/active/sprint34_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Nouveau crate `nexus-coordinator-rs` (pas extension daemon-core)

**Retenu** : creer `crates/nexus-coordinator-rs/` comme crate
bibliotheque dans le workspace Rust. Ce crate contient la business
logic coordinator (dispatcher, validator, kudos ledger) sans
serveur HTTP — le daemon l'appelle depuis son serveur axum existant.

**Rejete** :
- Etendre `nexus-shell-daemon-core` : violerait la separation des
  responsabilites. Le daemon-core gere le reseau P2P (iroh, gossip,
  browse, canary). Le coordinator gere la logique applicative
  (dispatch tasks, validate results, credit kudos).
- Etendre `nexus-shell-daemon` (binaire) : mettrait la business
  logic dans le binaire au lieu d'une lib testable.
- Nouveau binaire separe : compliquerait le deployment (2 processes
  Rust au lieu d'un). Le design cible est un daemon unique.

**Implications code** : `crates/nexus-coordinator-rs/Cargo.toml` (NEW),
`crates/nexus-coordinator-rs/src/lib.rs` (NEW), `Cargo.toml`
(workspace member).

### D2 — Migration graduelle endpoint-par-endpoint (pas big-bang)

**Retenu** : chaque endpoint migre est ajoute au serveur axum du
daemon et l'ancien endpoint Python reste fonctionnel. Le coordinator
Python tourne toujours en parallele pendant la transition.
Les endpoints migres sont marques `#[deprecated]` cote Python
(ruff N-xxx) pour tracking.

**Rejete** :
- Big-bang : risque trop eleve, impossible en un sprint.
- Proxy inverse : complexite reseau + latence supplementaire,
  overkill pour un processus local.

**Implications code** : `crates/nexus-shell-daemon/src/http.rs`
(routes coordinator), aucune suppression Python dans ce sprint.

### D3 — rusqlite pour persistence coordinator Rust

**Retenu** : le crate coordinator-rs utilise `rusqlite` (deja dans
le workspace via `nexus-worker-core`) avec son propre fichier
`~/.sbfb/coordinator.db`. Schema SQLite identique a celui du
coordinator Python pour migration transparente des donnees.

**Rejete** :
- Partager le DB file du daemon : couplage excessif.
- sled/redb : pas de SQL, migration schema plus complexe,
  pas de tooling d'inspection.
- PostgreSQL/etc. : viole le principe zero-infra P2P.

**Implications code** : `crates/nexus-coordinator-rs/Cargo.toml`
(dep rusqlite), `crates/nexus-coordinator-rs/src/db.rs` (NEW).

### D4 — Dispatcher task submission pipeline en Rust natif

**Retenu** : le dispatcher Rust appelle directement les primitives
`nexus-core-rs` (sign_task, canonical_bytes, iroh doc insert) sans
passer par PyO3. Les types `Task`, `TaskEntry` sont deja definis
en Rust — le dispatcher Rust les utilise nativement.

**Rejete** :
- Garder PyO3 comme bridge : elimine le gain principal de la
  migration (suppression du round-trip Python→Rust→Python).
- Nouveau format de tache : violerait pre-launch protocol policy.

**Implications code** : `crates/nexus-coordinator-rs/src/dispatcher.rs`
(NEW), types existants dans `nexus-core-rs`.

### D5 — MANDATORY 3/3 resolus en Phase A dette

**Retenu** :
- P2-B-1-S33 shellcheck CI 3/3 : creer `.github/workflows/shellcheck.yml`
  minimal (run shellcheck sur `scripts/*.sh`). ~30 LOC YAML.
- P2-C-1-S33 cross-daemon E2E 3/3 : etendre `nexus-test-harness`
  avec un test qui publie un blob sur daemon-A et le fetche depuis
  daemon-B via iroh-blobs. ~100 LOC Rust.
- P2-B-2-S33 REPO_URL 3/3 : **BLOQUE EXTERNE** — le repo n'est pas
  public. Documenter le blocker, remplacer le placeholder
  `https://github.com/user/nexus-grid.git` par un commentaire
  `TODO(v1.0): replace with actual public repo URL once published`.
  Ce n'est pas une resolution technique mais un acte de documentation
  qui empeche le carry silencieux.

**Rejete** :
- Ignorer les 3/3 : viole §6.2.1 Regle 2.
- Creer un faux repo public : premature et risque securite.

**Implications code** : `.github/workflows/shellcheck.yml` (NEW),
`crates/nexus-test-harness/tests/` (+1 test), `scripts/install-node.sh`
(TODO comment).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ⚠️.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 0 ❌).

D3 ⚠️ : schema coordinator.db sans strategie de versioning explicite
pendant la cohabitation Python/Rust. Decision : adjust — Phase A
ajoute une table `schema_version` dans coordinator.db avec version
guard au `open()`. Le schema Python et Rust partagent le meme
numero de version ; un mismatch = erreur au boot. Ca empeche les
drifts silencieux pendant la migration graduelle.

D5 ⚠️ : REPO_URL reste un carry silencieux malgre le TODO.
Decision : accept — le blocker est externe (pas de repo public).
L'audit gate S36 revalidera. Si le repo est public d'ici S36,
le TODO sera remplace. Sinon, carry 4/3 documente avec justification
"blocker externe inchange".

---

## §5 Plan Phase outline A..D

### Phase A — MANDATORY 3/3 + fondation crate coordinator-rs

**But** : fermer les 3 items MANDATORY + creer le squelette du
nouveau crate avec les types de base et le module DB.
- shellcheck CI : `.github/workflows/shellcheck.yml`
- cross-daemon E2E : test iroh-blobs publish + fetch cross-daemon
- REPO_URL : documentation du blocker externe
- `crates/nexus-coordinator-rs/` : Cargo.toml + lib.rs + db.rs
  (schema SQLite pour tasks + kudos) + types.rs (TaskSubmission,
  ValidationResult, KudosEntry)
- Commit : `feat(sprint35): Sprint 35 Phase A — MANDATORY 3/3 +
  crate nexus-coordinator-rs fondation`

### Phase B — Dispatcher Rust natif

**But** : le dispatcher de taches vit en Rust et signe/soumet
les taches nativement sans PyO3.
- `dispatcher.rs` : TaskDispatcher struct, submit() method
  (canonical_bytes + sign_task + iroh doc insert)
- Endpoint axum `POST /api/tasks/submit` dans le daemon
- Tests unitaires + integration via test-harness
- Commit : `feat(sprint35): Sprint 35 Phase B — Dispatcher Rust
  natif task submission pipeline`

### Phase C — Validator Rust natif

**But** : le validateur de resultats tourne en Rust avec un
subscription loop tokio sur les iroh LiveEvents.
- `validator.rs` : ResultValidator struct, validation loop
  (verify_result + verify_claim + kudos credit trigger)
- Wire au daemon runtime (spawn comme tokio task)
- Tests unitaires + mock iroh events
- Commit : `feat(sprint35): Sprint 35 Phase C — Validator Rust
  natif result verification loop`

### Phase D — Wrap-up

- verification.md fail-fast 30+ rows
- sprint36_audit_plan.md
- SPRINT_LOG.md row S35
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md last_validated S35
- Migration active/ → archive/v1.2/
- Commit : `chore(sprint35): Phase D — wrap-up + verification
  + audit plan S36 + migration`

---

## §6 Items carry/dette

### Resolus S35

- [x] P2-B-1-S33 shellcheck CI 3/3 : Phase A (`.github/workflows/shellcheck.yml`)
- [x] P2-C-1-S33 cross-daemon E2E 3/3 : Phase A (test harness blob cross-fetch)
- [x] P2-B-2-S33 REPO_URL 3/3 : Phase A (documentation blocker externe, TODO(v1.0))

### MANDATORY evalues — DEFER justifie

- P3-grammar executor 3/3+ : **DEFER** — le task_runner Python est
  en cours de remplacement par le dispatcher/validator Rust (S35+).
  Wiring grammar dans le legacy Python stub serait du travail jete.
  Le wiring se fera dans la version Rust quand le pipeline inference
  natif sera complet. Justification technique valide §6.2.1.
- P3-watermark executor 3/3+ : **DEFER** — meme justification.
  SynthID wiring depend du pipeline inference Rust.

### Carries confirmes S36

- [carry] P2-A-1 rand triple : blocker upstream inchange (frost-core
  rand_core 0.6 + iroh stack disjoints). Re-evaluer si convergence.
- [carry] P2-A-2 aggressive update PATTERNS.md : documentation <50 LOC.
- [carry] P2-B-1-S34 log convergence 2/3 : launcher.log + daemon log
  separes. Design partage log directory S36.
- [carry] P2-C-1-S34 .icns macOS 2/3 : .png fallback present.
  .icns necessite macOS ou outil tiers.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **Migration complete coordinator** — S36+ (S35 = fondations +
   dispatcher + validator seulement, pas kudos/output-filter/canary)
2. **Suppression coordinator Python** — post-migration complete
   (le coordinator Python reste fonctionnel pendant la transition)
3. **KudosLedger Rust** — S36 (tier 1 mais depends du validator)
4. **OutputFilter/PiiRedactor Rust** — S37+ (tier 2, guardrail)
5. **CanaryRegistry Rust** — S37+ (tier 2, compliance)
6. **CI pipeline multi-OS** — S36+ (shellcheck CI cree S35, mais
   build + test CI pas dans scope)
7. **VPS deployment** — S36+ (inchange)
8. **Code signing macOS** — post-v1.0 (inchange)
9. **P3 grammar/watermark** — post-pipeline Rust (defer justifie)
10. **SDK Python rewrite** — hors-scope (reste Python pour binding)

---

## §8 Tracabilite scope (S34 → S35)

| Item S34 NOT | Ou dans S35 |
|---|---|
| VPS deployment | §7.7 scope cut S36+ |
| Code signing macOS | §7.8 scope cut post-v1.0 |
| MSI/NSIS installer | §7 implicite |
| .deb/.rpm packages | §7 implicite |
| Auto-update | §7 implicite |
| Tray icon | §7 implicite |
| CI pipeline multi-OS | §7.6 scope cut S36+ |
| stop/status CLI | §7 implicite |
| Cross-node task Ollama | §7 implicite (remplace par dispatcher Rust) |
| Docker daemon/worker | §7 implicite |
| P3 grammar/watermark | §7.9 scope cut post-pipeline Rust |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | Complexite migration async Python→Rust tokio | Medium | Commencer par dispatcher (plus simple, pas de subscription loop) avant validator |
| R2 | Schema SQLite incompatible entre Python et Rust | Low | Utiliser le meme schema SQL, tester migration round-trip |
| R3 | iroh LiveEvent API change Rust vs PyO3 | Low | PyO3 bindings appellent deja l'API Rust native, pas de gap |
| R4 | Tests coordinator Python cassent si endpoints dupliques | Medium | Les endpoints Rust sont sur le port daemon, Python sur le sien — pas de conflit |
