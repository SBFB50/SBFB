# Sprint 37 — Kickoff (hash-chain KudosLedger + MANDATORY carries 3/3)

**Ecrit** : 2026-04-28 (session fraiche post-audit gate S36 `743fd24`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1 : S37 impair).
**Tip master d'entree** : `743fd24` (chore(planning) audit gate
S36 PASS).
**Phase 0 audit Sprint 36** : **DEJA JOUE** — findings dans
`.planning/active/sprint36_audit_findings.md` (verdict **PASS**,
0 P0/P1, 2 P2, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-28) : HARDENING_ROADMAP last_validated
  `2026-04-28` (S36 Phase A — compteurs stale, fix S37 Phase A).
  0 trigger actif (memes triggers verifies S36 kickoff).

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - wasmtime LTS bump : pas de dep directe — INACTIVE
  - Tor PoW spec, NIST PQC, RFC 9591 erratum : NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S37** :
  - `tracing-appender 0.2` : deja workspace dep (daemon + worker).
    Launcher l'ajoutera.
  - `blake3 1.5` + `serde_jcs 0.2` : deja workspace deps via
    nexus-core-rs. coordinator-rs ajoutera `blake3` en dep directe
    pour le hash-chain. `canonical_bytes()` et `DOMAIN_KUDOS_V1`
    sont dans nexus-core-rs (re-exportes via lib.rs).
  - `icns` crate : nouvelle dep build-only pour generation .icns
    cross-platform. Pas de dep runtime. Recherche crates.io :
    `icns 0.3` (pure Rust, Read/Write ICNS, MIT license, 7 ans
    maintenu). Alternative `apple-icns` non evaluee (plus recente
    mais moins testee).

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem, LT-4
  biometric, LT-5 redundancy : tous requierent tag v1.0 ou
  condition externe → aucun declenche. LT-6 : RESOLVED (S32).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 36 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : dette pair + DaemonHttpState persistent CoordinatorDb
  singleton Arc<Mutex<>> (+3 tests)
- Phase B : result submission endpoint POST /api/v1/results/submit
  + validate_result free fn (+4 tests)
- Phase C : KudosLedger Rust natif credit() + get_project_kudos()
  + GET /api/v1/kudos/{project_id} + wire post-validation (+5 tests)

Audit gate S36 : **PASS** (0 P0/P1, 2 P2 [HARDENING compteurs
stale + unwrap_or_default() handlers], 1 P3 [clone pow_keypair]).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-28 (S36 Phase A — compteurs stale,
cf. P2-AUDIT-1). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `743fd24`)

| Suite | Count |
|---|---|
| Rust nextest | 936 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1939** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **implemente le hash-chain cryptographique du
KudosLedger** (BLAKE3 + JCS canonical, `DOMAIN_KUDOS_V1`) et
**ferme les 2 MANDATORY 3/3** (log convergence launcher/daemon +
.icns macOS), plus le batch P2 audit/review S36.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 36

**DONE** — `743fd24`. Verdict PASS (0 P0/P1, 2 P2 + 1 P3).
Cf. `.planning/active/sprint36_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Log convergence : shared directory `~/.sbfb/logs/`

**Retenu** : unifier launcher et daemon dans `~/.sbfb/logs/`.
Le daemon ecrit deja `daemon.log` via `tracing_appender::rolling::daily`
dans `~/.sbfb/shell-daemon/logs/`. Le changement :
(a) Deplacer le daemon log dir de `~/.sbfb/shell-daemon/logs/`
vers `~/.sbfb/logs/` (1 ligne dans `paths.rs`).
(b) Convertir le launcher de `File::create("~/.sbfb/launcher.log")`
(truncate a chaque lancement) vers `tracing_appender::rolling::daily`
dans `~/.sbfb/logs/launcher.log` (append, rotation quotidienne,
format structure identique au daemon).
(c) Le worker garde son propre log dir (`~/.sbfb/worker/logs/`)
inchange — il tourne sur des machines differentes.

**Rejete** :
- Single log file (contention launcher/daemon sur le meme fichier,
  entrelacement illisible).
- syslog/journald (platform-dependant, deja gere par
  `nexus-events-core` platform writers comme couche separee).
- Garder les paths separees (viole MANDATORY 3/3).

**Implications code** :
- `crates/nexus-shell-daemon-core/src/paths.rs` (log_dir → `logs/`)
- `crates/nexus-launcher/src/main.rs` (tracing-appender au lieu
  de File::create)
- `crates/nexus-launcher/Cargo.toml` (+tracing-appender dep)

### D2 — .icns macOS via crate `icns` (cross-platform, pas macOS-only)

**Retenu** : ajouter une etape dans `scripts/bundle-macos.sh` qui
genere `nexus-launcher.icns` depuis le PNG existant
(`assets/nexus-launcher.png`) en utilisant le crate `icns 0.3`
(pure Rust, MIT). Execution en script Rust inline ou petit binaire
de build. Pas besoin de macOS ni de `iconutil`.

**Rejete** :
- `iconutil` (macOS-only, dev env = Windows, blocker externe).
- `png2icns` (outil externe, pas dans le workspace).
- Garder PNG fallback (viole MANDATORY 3/3, 3 reports S34-S36).
- `tauri-icns` fork (meme code que mdsteele, pas de valeur ajoutee).
- Ajouter `icns` comme workspace dep (overkill pour un outil de
  build one-shot ; un script Rust standalone avec `cargo script`
  ou un petit binary dans `tools/` suffit).

**Implications code** :
- `scripts/bundle-macos.sh` (etape .icns)
- `tools/png-to-icns/` (petit binaire Rust, ou script inline)
- `configs/macos/Info.plist` (pointer vers .icns au lieu de .png)

### D3 — KudosLedger hash-chain (BLAKE3 + canonical_bytes)

**Retenu** : implementer le hash-chain dans `kudos_ledger::credit()`.
Chaque entree kudos recoit :
- `prev_hash` = `entry_hash` de la derniere entree du projet
  (ou `"genesis"` pour la premiere)
- `entry_hash` = `hex(BLAKE3(canonical_bytes(hashable_entry,
  DOMAIN_KUDOS_V1)))` ou `hashable_entry` est le `KudosEntry`
  avec `entry_hash` vide (pour eviter la circularite)

Composants existants utilises sans nouvelle dep :
- `nexus_core_rs::canonical::canonical_bytes()` (JCS + domain
  separation, RFC 8785)
- `nexus_core_rs::canonical::DOMAIN_KUDOS_V1` (deja reserve)
- `blake3::hash()` (workspace dep, ajouter a coordinator-rs)
- `hex::encode()` (deja dep coordinator-rs)

Nouvelle query DB : `get_last_entry_hash(project_id) -> String`
pour recuperer le `entry_hash` de la derniere entree du projet
(ORDER BY created_at DESC LIMIT 1).

**Rejete** :
- SHA256 (blake3 deja dans le workspace, plus rapide, meme
  securite 128-bit post-quantum).
- Pas de hash (deja differe S36, P2-REVIEW-C-1 — le hash-chain
  est le fondement de l'intégrité du ledger).
- Hash global (pas per-project) : complique le verify et melange
  les projets. Per-project permet la verification locale par
  projet.
- External JCS crate : `serde_jcs` est deja dans le workspace,
  `canonical_bytes` est le pattern standard du projet.

**Implications code** :
- `crates/nexus-coordinator-rs/Cargo.toml` (+blake3 dep)
- `crates/nexus-coordinator-rs/src/kudos_ledger.rs` (hash computation)
- `crates/nexus-coordinator-rs/src/db.rs` (+get_last_entry_hash query)

### D4 — P2 batch audit S36 + phase reviews (Phase A)

**Retenu** : resoudre les 5 P2 en Phase A :
- P2-AUDIT-1 : HARDENING_ROADMAP compteurs → 936 Rust / ~1939 total
- P2-AUDIT-2 : `unwrap_or_default()` → `map_err` → 500 dans 2 handlers
- P2-REVIEW-A-1/B-1 : 3 tests mutex poisoned (submit_task,
  submit_result, get_kudos)
- P2-REVIEW-C-2 : double query project_id → refactorer
  validate_result() pour retourner le TaskRecord avec le verdict

**Rejete** :
- Commits separes par P2 (overkill pour des fixes triviaux).
- Differer a S38 (les P2 sont tous < 20 LOC chacun, total < 100
  LOC pour la phase, les regrouper en Phase A est efficace).

**Implications code** :
- `docs/security/HARDENING_ROADMAP.md` (compteurs)
- `crates/nexus-shell-daemon/src/http.rs` (unwrap_or_default +
  3 mutex tests)
- `crates/nexus-coordinator-rs/src/validator.rs` (retour TaskRecord)

### D5 — Validator loop LiveEvents = scope cut S38

**Retenu** : la subscription tokio aux iroh LiveEvents est
**explicitement differee a S38**. Compteur : 2/3 → 3/3
**MANDATORY S38**. Raison : le Doc handle iroh vit dans
`CuratorRuntimeHandle` et n'est pas expose pour le coordinator.
L'extraction requiert un refactor du runtime (exposer `Arc<Doc>`
ou un channel de LiveEvents). S37 a deja 2 MANDATORY + hash-chain.
Le chemin HTTP (`POST /api/v1/results/submit`) reste suffisant.

**Rejete** :
- Forcer inclusion S37 (scope creep, 2 MANDATORY + hash-chain
  sont deja un sprint charge).
- Reclassifier LT (< 500 LOC, interdit §6.2.1).

**Implications code** : aucune — decision de non-action. Le carry
S38 est cree dans §6 avec compteur 3/3.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ✅, D5 ⚠️ (note, pas gap).
Rigor signal G4 satisfait (1 ⚠️ + 4 ✅ sur 5).

D2 ⚠️ (2 findings) :
- D2-1 "apple-icns" cite comme alternative non-evaluee n'existe pas
  sur crates.io. L'alternative reelle est `tauri-icns` (fork Tauri
  Foundation de mdsteele/rust-icns). **Clarification** : `icns 0.3`
  (mdsteele) est le crate original, `tauri-icns` est un fork. Les deux
  partagent le meme code. On utilise l'original (plus stable, meme API).
  La mention "apple-icns" est retiree du texte D2.
- D2-2 couverture tailles icones macOS 11+ non verifiee. **Accept** :
  le risk register R1 couvre le fallback. La verification sera faite
  en Phase A au moment du code (test empirique sur les tailles).

D3 notes (non-gaps, informationnelles) :
- Linear chain vs Merkle tree : accept — linear per-project est correct
  pour les volumes pre-v1.0. Si le ledger atteint des millions d'entrees
  post-v1.0, la migration vers Merkle tree sera evaluee (ajout au risk
  register R2).
- Append-only enforcement : accept — ajout d'un commentaire SQL dans
  le schema migration + documentation dans kudos_ledger.rs. Pas de
  constraint SQL hard (les queries existantes n'ont pas d'UPDATE/DELETE).

---

## §5 Plan Phase outline A..C

### Phase A — MANDATORY batch + P2 batch audit/review

**But** : fermer les 2 MANDATORY 3/3 + les 5 P2 audit/review.
- Log convergence : daemon log_dir → `~/.sbfb/logs/`, launcher
  tracing-appender, rotation daily
- .icns : script/tool Rust genere .icns depuis PNG
- HARDENING compteurs fix (936 Rust / ~1939 total)
- unwrap_or_default() → proper error handling (2 handlers)
- 3 mutex poisoned tests
- validate_result() retourne TaskRecord avec verdict (double query fix)
- Commit : `feat(sprint37): Sprint 37 Phase A — MANDATORY log
  convergence + .icns + P2 batch audit/review S36`

### Phase B — KudosLedger hash-chain (BLAKE3 + JCS)

**But** : chaque entree kudos porte un hash-chain cryptographique.
- `get_last_entry_hash(project_id)` query
- `credit()` compute `entry_hash` via `canonical_bytes` + BLAKE3
- `prev_hash` = derniere entree (ou "genesis")
- `verify_chain(project_id)` read-only verification
- Tests : genesis, multi-entry, chain integrity, cross-project
  isolation
- Commit : `feat(sprint37): Sprint 37 Phase B — KudosLedger
  hash-chain BLAKE3 + JCS canonical`

### Phase C — Wrap-up

- verification.md fail-fast 28+ rows
- sprint38_audit_plan.md
- SPRINT_LOG.md row S37
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S37
- Migration active/ → archive/v1.2/
- Commit : `chore(sprint37): Phase C — wrap-up + verification
  + audit plan S38 + migration`

---

## §6 Items carry/dette

### Resolus S37 (plan)

- [x] P2-B-1-S34 log convergence 3/3 **MANDATORY** : Phase A
- [x] P2-C-1-S34 .icns macOS 3/3 **MANDATORY** : Phase A
- [x] P2-AUDIT-1 HARDENING compteurs stale : Phase A
- [x] P2-AUDIT-2 unwrap_or_default() handlers : Phase A
- [x] P2-REVIEW-A-1/B-1 mutex poisoned tests : Phase A
- [x] P2-REVIEW-C-2 double query project_id : Phase A
- [x] P2-REVIEW-C-1 hash-chain vide : Phase B

### Carries confirmes S38

- [carry] P2-A-1 rand blocker upstream 6+/3 : blocker externe
  inchange (frost-core rand_core 0.6 + iroh stack disjoints). Pas
  de convergence observee. Exemption §6.2.1 blocker externe.
- [carry] P2-REVIEW-C-1-S35 validator_loop tokio 2/3 → 3/3
  **MANDATORY S38** : defer S38 (D5 scope cut). Requiert refactor
  CuratorRuntimeHandle.
- [carry] P2-AUDIT-2-S35 pre-release transitives iroh : condition
  heritee pin 0.98.

### MANDATORY evalues — DEFER justifie

- P3-grammar executor 3/3+ : **DEFER** — pipeline Rust natif en
  cours. Exemption dependance sequentielle interne.
- P3-watermark executor 3/3+ : **DEFER** — meme justification.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **Migration complete coordinator** — S38+ (S37 = hash-chain
   seulement, pas OutputFilter/PiiRedactor/CanaryRegistry)
2. **Suppression coordinator Python** — post-migration complete
3. **OutputFilter/PiiRedactor Rust** — S38+ (tier 2, guardrail)
4. **CanaryRegistry Rust** — S38+ (tier 2, compliance)
5. **Validator loop LiveEvents** — S38 MANDATORY 3/3 (D5)
6. **CI pipeline multi-OS** — S38+ (inchange)
7. **VPS deployment** — S38+ (inchange)
8. **Code signing macOS** — post-v1.0 (inchange)
9. **P3 grammar/watermark** — post-pipeline Rust (defer justifie)
10. **SDK Python rewrite** — hors-scope (reste Python pour binding)
11. **Kudos debit/stake** — interdit (Day 0 decision #7, non-monnaie)
12. **verify_chain endpoint HTTP** — S38 (S37 = function read-only
    interne, endpoint expose en S38)

---

## §8 Tracabilite scope (S36 → S37)

| Item S36 carry / audit | Ou dans S37 |
|---|---|
| P2-B-1-S34 log convergence 3/3 | §5 Phase A — MANDATORY |
| P2-C-1-S34 .icns macOS 3/3 | §5 Phase A — MANDATORY |
| P2-AUDIT-1 HARDENING compteurs | §5 Phase A |
| P2-AUDIT-2 unwrap_or_default | §5 Phase A |
| P2-REVIEW-A-1/B-1 mutex poisoned | §5 Phase A |
| P2-REVIEW-C-1 hash-chain vide | §5 Phase B |
| P2-REVIEW-C-2 double query | §5 Phase A |
| Validator loop LiveEvents | §7.5 scope cut S38 MANDATORY |
| Migration complete coordinator | §7.1 scope cut S38+ |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | `icns` crate ne supporte pas les tailles d'icones requises par macOS | Low | Fallback : generer les tailles supportees, macOS downscale automatiquement. |
| R2 | Hash-chain JCS canonical_bytes diverge Python ↔ Rust | Medium | S37 hash-chain est Rust-only (le ledger Python cohabite mais ne compute pas de hash). La verification cross-language sera testee quand le ledger Python sera supprime. |
| R3 | Deplacement log_dir casse des scripts/paths existants | Low | Grep workspace pour `shell-daemon/logs` avant modification. Le daemon est le seul consommateur. |
| R4 | validate_result retour TaskRecord change l'API publique | Low | La fonction libre `validate_result` est interne au crate. Le handler dans http.rs est le seul consommateur. |
