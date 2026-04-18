# Sprint 20 — Audit findings (session fraîche Sprint 21 Phase 0)

**Auditeur** : session fraîche 2026-04-18 (pattern permanent depuis
Sprint 7, cf. `docs/claude/README.md §3`).
**Timebox effective** : ~1 h (lecture ciblée Cas A + délégation
agent Explore thorough multi-track + vérifications critiques
indépendantes en parallèle).
**Range audité** : `3a7f0a3..131f32b` (26 commits, Sprint 20 CLOSED
+ chore migration Phase F review file ajouté pré-audit par cette
session pour rattraper une omission `f209168`).
**Tip audité** : `131f32b` (chore(planning): sprint20 — migrate
Phase F review overlooked in f209168).

---

## 1. Verdict global : **PASS**

0 P0 + 0 P1. **4 P2 carry-over actifs** (tous documentés ci-dessous,
2 destinés à PATTERNS.md tech debt, 1 docs fix, 1 meta-track
re-carry). **6 P2 résolus in-phase** pendant le sprint (non-action
audit). **6 P3 cosmétiques** non-bloquants.

**Rigor signal G4** : **SATISFAIT**. ≥ 1 P2+ documenté (règle
calibration auditeur). L'audit a exploré 7 tracks (A-F + meta
Radicle) + la **dimension G8 traceability supplémentaire** sur
Phase E (premier pivot G8 effectif du projet), conformément au
plan écrit en Phase F.

**Sprint 21 Phase A non bloqué.** Kickoff S21 peut être ouvert
immédiatement après archivage de ce document et ouverture du
`chore(planning): open Sprint 21 ...`.

---

## 2. Contexte

### 2.1 Range audit (26 commits)

```
131f32b chore(planning): sprint20 — migrate Phase F review overlooked in f209168   ← tip audit
54b0303 chore(sprint20): resolve Phase F SHA placeholders to f209168 + carry S17 wrap-up tip
f209168 chore(sprint20): Phase F — wrap-up + verification + audit plan S21 + migrate planning
1ad2def chore(sprint20): fmt fix frost.rs residual + .gitignore test-results — Phase E hygiene
b7d8d74 chore(gitignore): exclude .claude/narrate-action.lock runtime lockfile
6a3f199 feat(canary): Sprint 20 Phase E — federation foundations + WSS fallback observability
3c18908 chore(hooks): narrate-action mutex to cap Haiku subprocess stacking
e653619 chore(planning): Sprint 20 Phase E — G8 preflight + S1 finding E.6 inline absorption
b634c23 chore(skill): G8 robustness follow-up — 4 edge cases + auditor G8 traceability dimension
b6da3a4 chore(skill): eliminate node hook process leaks + orphan cleanup on SessionStart
e2e8595 chore(workflow): tighten bootstrap §7.1 G8 references after first real application
bd16e64 chore(planning): Sprint 20 Phase E — pivot G8 to federation foundations + WSS fallback
59225ee chore(workflow): introduce G8 phase pre-flight factual evolution check + nexus-phase-preflight skill
7ea68a6 chore(sprint20): Phase D audit P2-1 follow-up — honest llama_cpp.rs sampler comment
c85397b feat(sprint20): Phase D — structured output dual-backend LlmBackend
2e045f1 chore(planning): Sprint 20 Phase C — audit review archive (P2-C-SEC-1 levé in-phase)
16b94ba feat(sprint20): Phase C — PoW runtime wire gossip subscribe
c32ecb3 feat(sprint20): Phase B — duress PIN (fake keypair noop) + panic wipe 5-tap gesture
7ff22a0 chore(planning): Sprint 20 Phase B — duress PIN + panic wipe design doc
3380f76 fix(sprint18): token_rotation overlap window uses wall-clock to survive short-uptime
667e122 chore(tooling): gitattributes LF hardening + nextest adoption
c12878e chore(skill): sidecar unified terminal (narration tail + delivery reverse channel)
4f4a30a chore(skill): sidecar input terminal (live BTW injection via PostToolUse)
98139e3 chore(skill): narration terminal + gitignore comment fix
05271fa feat(sprint20): Phase A — encryption at rest keypair (Argon2id + AES-256-GCM + double layer OS keyring)
1b1f9cb chore(planning): open Sprint 20 — encryption at rest + duress + panic wipe + PoW wire + structured output + canary auto-publish + dual-transport
```

5 commits `feat` (phases A/B/C/D/E) + Phase F wrap-up chore +
13 chores planning/workflow/skill/hooks/tooling + 1 fix carry S18.

### 2.2 Compteurs tests vérifiés à la main (tip `131f32b`)

| Suite | verification.md | Observé audit | Delta vs S19 | Match |
|---|---|---|---|---|
| Rust workspace nextest | 642 | **642 pass, 0 skip** (re-run auditeur 11.5 s) | +104 vs 538 | ✓ |
| Python SDK | 185 | 185 (fallback verification) | 0 | ~ |
| Python coordinator | 213 + 3 skip | 213+3 (fallback) | +5 | ~ |
| Python app-gov | 46 | 46 (fallback) | 0 | ~ |
| Vitest unit | 241 | 241 (fallback) | +2 | ~ |
| Playwright | 38 | 38 (fallback) | 0 | ~ |
| size-limit | 7/7 | 7/7 (fallback) | 0 | ~ |
| SPDX license hooks | 246+ | 246+ (fallback) | +2 | ~ |
| **Total** | **~1371** | **~1371** | **+111** | ✓ |

Le `~` marque les suites non rejouées (fallback accepté pour
suites stables sur plusieurs sprints et non touchées par ce
sprint — Python SDK / Playwright / size-limit inchangés
numériquement S20). Rust a été rejouée parce que la majeure
partie du delta S20 y vit (+104 vs +7 autres suites).

### 2.3 Pre-launch protocol policy vérifiée

```
crates/nexus-core-rs/src/keystore.rs:108              BLOB_VERSION:        u8  = 0x01
crates/nexus-core-rs/src/schemas/task_response.rs:48  TASK_RESPONSE_VERSION: u8 = 1
crates/nexus-shell-daemon-core/src/canary/mod.rs:70   CANARY_VERSION:      u16 = 1
crates/nexus-shell-daemon-core/src/iroh_runtime.rs:92 ANNOUNCEMENT_VERSION: u16 = 1
```

**Tous inchangés S20.** Aucun tolerant decoder multi-version
introduit. Convention CLAUDE.md §Pre-launch protocol policy
respectée. ✓

### 2.4 Dead-man switch intégrité (vérification critique G8)

Grep exhaustif `CanarySigner::sign` + workflow `.github/workflows/
canary-monthly.yml` lu en entier.

- Header du workflow (lignes 6–27) documente explicitement le
  rejet de l'auto-publish pattern, citant littéralement le
  threat-model « gag-ordered maintainer », « Ed25519 key in
  GitHub Actions secrets = GHA-runner breach equivalent », et
  aligne sur le modèle classique rsync.net / IVPN.
- Cron weekly (ligne 60) = **vérification de fraîcheur
  uniquement** (email GitHub natif si stale > 45 jours). Pas de
  signature.
- CLI manuel `sbfb canary publish --headline ...` uniquement
  (ligne 110/114 du workflow, instructions au mainteneur).
- Sprint 20 Phase E n'a **pas** introduit de scheduler/cron/GHA
  appelant `CanarySigner::sign` dans le diff. Vérifié par grep.

**La décision S18 E2 `04c9621` est honorée par construction
Phase E.** Le pivot G8 Option C (foundations federation) préserve
intégralement cette décision — `CanarySigner` trait +
`FrostCanarySigner` impl sont des **primitives invocables
seulement manuellement** (CLI `sbfb canary publish` + `sbfb canary
ack`).

### 2.5 Vérification indépendante des fixes in-phase (B-1, D-1)

L'audit a personnellement vérifié le code sur les 2 points
les plus critiques flagués en pre-review intra-phase :

**B-1 double-wipe handler** (pre-review Phase B P2-B1) :
- `crates/nexus-shell-daemon/src/panic.rs:183-200` montre
  `exit_only(exit_code: i32) -> !` comme primitive séparée qui
  n'appelle PAS `execute()`. Le commentaire (lignes 186-189)
  cite explicitement « the two operations are kept as
  independent primitives so the wipe cannot be accidentally
  executed twice on a single request (what used to be a single
  `execute_and_exit` entry point re-ran the wipe inside the
  delay task) ».
- `crates/nexus-shell-daemon/src/http.rs:759-767` appelle
  `service.execute()` synchronously puis spawn `service.exit_
  only(0)` dans le delay task.
- **Statut : RESOLVED IN-PHASE** ✓. Pas de carry.

**D-1 llguidance version** (pre-review Phase D P2-2) :
- `.planning/archive/v1.2/sprint20_kickoff.md §D4 ligne 470`
  écrit `llguidance = "1.7"` (post-discovery update, pas la
  valeur 0.7 initiale).
- **Statut : RESOLVED IN-PHASE** ✓. Pas de carry.

### 2.6 Calibration G4 : ≥ 1 P2+ documenté

Le findings ci-dessous liste **4 P2 carry-over actifs** + 6 P2
résolus in-phase + 6 P3. Rigor signal satisfait.

---

## 3. Tracks

Ordre : A → B → C → D → E (avec dimension G8 retrospective) →
F → meta Radicle.

### Track A — Phase A : encryption at rest keypair (commit `05271fa`)

**Verdict** : **PASS**.

**P2 résolus in-phase** (non-action) :
- `aes-gcm 0.10` + `argon2 0.5` + `keyring 3.6` traces dans
  `sprint20_plan.md §3.1 Research consulté` avec rationale NASM
  Windows build, RFC 5116, advisory checks, migration path T25
  → `aws-lc-rs fips`.
- `T25` (FIPS one-file swap), `T26` (bench cal Pi 4 post-
  telemetry), `T27` (NASM Windows build variant) loggés
  `docs/rust/PATTERNS.md §Sprint 20.1` avec owners et deadlines.
- Test `param_downgrade_attack_rejected` adresse invariant AAD
  (modification params invalide MAC) — `keystore.rs:1124-1144`.
- `SBFB_IDENTITY_SECRET_HEX` env var T24 scope dev/smoke
  clarifié, UDS sidecar secret channel S22+ tracé PATTERNS
  §Sprint 20.2.
- `rotate_pin()` error mapping `KeyStoreError::WrongPin`
  variante séparée — `keystore.rs:193-200`.

**A-1 — design doc Phase A absent** (audit verdict : **P3
accept rétro**) :
- Commit body `05271fa` réfère `.planning/research/S20_phase_A_
  encryption_at_rest_design.md` jamais écrit. Alternatives
  (HPKE, age, TPM/SE/StrongBox, scrypt/PBKDF2/bcrypt, Argon2d/i
  vs id) sont documentées dans `sprint20_kickoff.md §D1-D2`
  (≈ 6 pages avec rationale Sygnia 2024 DPAPI, Signal SVR, RFC
  9106, OWASP 2024).
- Ce contenu **est** le design doc — il vit juste dans le
  kickoff au lieu d'un fichier séparé.
- **Accepté rétro, classification downgrade P3 cosmétique**
  — à éviter pour S21+ (un design doc séparé par phase big
  rock structurant, pattern B + D respecté mais A non).

**P3 non-bloquants** :
- A-3 `keystore.rs:860-862` 3× `try_into::<[u8;4]>().unwrap()`
  sans commentaire `// INFALLIBLE: guaranteed by length check
  above`. Invariant réel tenu par `blob.len() >= BLOB_HEADER_
  LEN + TAG_LEN`.
- A-4 `write_atomic` pas de cleanup `.enc.tmp` sur échec
  rename — `keystore.rs:888-901`.
- A-5 plan §4 annonçait +25 mais réel +28 tests (bonus `param_
  downgrade_attack_rejected` + tests launcher `unlock.rs`).
  Over-delivery documentée body `05271fa`.

### Track B — Phase B : duress PIN + panic wipe (commit `c32ecb3`)

**Verdict** : **PASS**.

**P2 résolus in-phase** (vérifiés audit §2.5 ci-dessus) :
- **B-1 double-wipe handler** : `exit_only(0)` primitive séparée
  (`panic.rs:183-200`), `http.rs:759-767` appelle `execute()`
  synchronously puis `exit_only(0)` dans le delay task. Pas de
  ré-exécution du wipe. ✓
- **B-2 CRAFT design doc** stagé dans commit phase au lieu de
  `chore(planning)` séparé : dérogation G5 mineure documentée
  dans le body commit section `Working tree audit`. Acceptable
  rétro (design doc atomiquement lié au code qui l'implémente).
  Tracé pour discipline future.

**B-4 — timing side-channel `unlock_differential`** (audit
verdict : **P3 accept**) :
- Kickoff §5 + D3 documentent explicitement le scope-cut
  « parallel KDF cancel S23+ ».
- Attaquant observing wall-clock peut distinguer `wrong-PIN`
  (1× Argon2id échec) vs `duress-PIN` (1× Argon2id succès mais
  2e slot = 2× Argon2id total).
- Scope cut maintenu, documenté `docs/security/DURESS.md` +
  `sprint20_kickoff.md §D3`. Non-action audit gate.

**P3 non-bloquant** :
- B-3 delta annoncé +18 Rust vs 20 observés (2 tests existants
  modifiés non comptés comme nouveaux). Body `c32ecb3`
  reconciled.

### Track C — Phase C : PoW runtime wire gossip subscribe (commits `16b94ba` + `2e045f1`)

**Verdict** : **PASS**.

**P2 résolus in-phase** :
- C-SEC-1 **RwLock poisoned inconsistency** : `wrap_payload_
  with_pow` + `PowPolicyWatcher::current()` utilisent
  `poisoned.into_inner().clone()` (graceful degradation).
  `http.rs` + `runtime.rs:841-844` cohérence unifiée. Audit
  review intra-phase `2e045f1`.
- **Grep no-bypass verify** : `gossip.subscribe\b` hors
  `subscribe_with_pow` → **0 match** dans `crates/nexus-shell-
  daemon-core/src/` et `crates/nexus-shell-daemon/src/` (row 22
  verification.md fail-fast vérifié). ✓
- C-3 **canary broadcast wrap PoW** : `main.rs:237` n'enveloppe
  pas le canary publish dans PoW. Scope Phase E (warrant
  canary). Accepté — le canary publish hors-PoW est délibéré
  (signal cryptographique, pas vector de spam gossip).

**C-PLAN-1 — plan docs divergence** (audit verdict : **P2 carry
S21 docs fix**) :
- `sprint20_plan.md §6.2` cite `crates/nexus-shell-daemon-core/
  src/iroh_runtime.rs::GossipClient::subscribe()` comme wire-
  point.
- Vrai call-site : `crates/nexus-shell-daemon/src/runtime.rs::
  spawn_gossip_subscribe_task()`. `browse.rs::subscribe()` dans
  `-core` gère l'attention set, pas le transport gossip.
- **Code correct** (Track C vérifié), **plan textuel divergent**.
- **Fix attendu S21** : `chore(sprint20-followup): fix plan
  §6.2 wire-point reference (post-audit)` OU note en tête du
  plan archivé. Carry S21 Phase 0.

**P3 non-bloquant** :
- C-2 delta +18 Rust vs +10 plan = over-delivery (+8 unit tests
  loader bonus). Body `16b94ba` reconciled.

### Track D — Phase D : structured output dual-backend (commits `c85397b` + `7ea68a6`)

**Verdict** : **PASS**.

**P2 résolus in-phase** (D-1 vérifié audit §2.5) :
- **D-1 llguidance version drift** : kickoff §D4 ligne 470
  spécifie `"1.7"` (post-discovery update). Design doc +
  `Cargo.toml` + inline comment tous cohérents `1.7`. ✓
- **D-2 PATTERNS §P30 affirmation logit-bias** : chore follow-
  up `7ea68a6` ajoute la note honnête Sprint 20 état
  (`llama_cpp.rs:307-308` + §P30) — matcher avance via
  `ff_tokens` + `consume_token` post-sélection, pas logit-bias
  wire au sampler. Validation `validate_task_response` = garde-
  fou effectif. Logit-bias wire S21+ carry documenté. ✓

**P3 non-bloquants** :
- D-3 `task_response.rs:146` `serde_json::to_value(RootSchema).
  expect(...)` sans commentaire `// INFALLIBLE: RootSchema is
  purely structural`. Convention projet cosmétique (1-liner).
- D-4 `llama_cpp.rs:442-448` `expand_tilde` path traversal local
  (pas de validation `path.components().any(|c| c == ParentDir)`).
  Config trust-root opérateur, impact faible. Acceptable
  pre-launch.
- D-5 delta +27 Rust vs +12 plan = over-delivery (+15 bonus).
  Body `c85397b` reconciled.

### Track E — Phase E : warrant canary federation + WSS fallback (commits `6a3f199` + `bd16e64` + `e653619`)

**Verdict** : **PASS** (avec **dimension G8 retrospective** —
premier pivot G8 effectif du projet).

#### 3.E.1 Dimension G8 traceability retrospective

Pivot G8 documenté conformément à `docs/claude/README.md §6.9`
et `.claude/skills/nexus-phase-preflight/SKILL.md` :

1. `sprint20_phase_E_preflight.md` écrit AVANT 1re ligne de
   code Phase E. Verdict initial `DESIGN-CONFLICT` suite scan
   S2 historical decisions :
   - Scan S2 attrape `04c9621` (S18 E2) qui rejette
     explicitement l'auto-publish scheduler pour raison
     threat-model (clef Ed25519 accessible au scheduler =
     compromission dead-man switch sous gag order).
   - Scan S1 attrape `relay_wss_only` client-side n'existe pas
     iroh 0.97 (WSS = unique mode depuis 0.91 upstream).

2. `sprint20_phase_E_pivot_proposal.md` produit, 3 options
   (A rollback / B scope-cut / C deep-evolution federation
   foundations).

3. Arbitrage user 2026-04-18 : **Option C**.

4. `bd16e64` (chore planning) **antérieur** au commit code
   `6a3f199` met à jour `sprint20_plan.md §Phase E` vers pivot
   Option C. **Plan mis à jour AVANT code** (convention G8).

5. Post-crash re-validation `e653619` → `sprint20_phase_E_
   preflight.md` verdict final **SCOPE-CUT-CONSISTENT** avec
   finding S1 E.6 absorbé inline (probe UDP QUIC diagnostic-
   only + log warn degraded, pas `RelayMode::Custom` qui n'existe
   pas).

6. 7 sous-tâches livrées et vérifiées :
   - `crates/nexus-shell-daemon-core/src/canary/signer.rs`
     (CanarySigner trait + Ed25519 impl baseline)
   - `crates/nexus-shell-daemon-core/src/canary/frost.rs`
     (FrostCanarySigner K-of-N RFC 9591 jan 2025,
     `frost-ed25519 = "2.1"` audit Trail of Bits 2023 Zcash
     Foundation, test
     `frost_sig_verifiable_by_standard_ed25519_verifier`)
   - `crates/nexus-shell-daemon-core/src/canary/duress_ack.rs`
     (CLI `sbfb canary ack`, domain `DOMAIN_DURESS_ACK_V1`
     distinct `DOMAIN_WARRANT_CANARY_V1`)
   - `crates/nexus-shell-daemon-core/src/canary/attestation.rs`
     (`AttestationProvider` trait + `NoopAttestation` prep TEE
     S25-30)
   - `crates/nexus-shell-daemon-core/src/transport_probe.rs`
     (UDP QUIC probe 3×10 s → log warn observability-only)
   - `packages/nexus-coordinator/src/nexus_coordinator/
     canary_registry.py` + `api/canary.py` (observational-only
     registry, `POST /api/canary/observed` + `GET /api/canary/
     network-health`)
   - `docs/security/WARRANT_CANARY_HARDENING.md` (threat model
     4 couches L0-L2 + FROST DKG cross-juridiction + TEE
     roadmap S25-30 + operator runbook)

7. **Dead-man switch intact** (vérifié §2.4 ci-dessus) : aucun
   scheduler/cron/GHA appelle `CanarySigner::sign()`, CLI manuel
   uniquement.

8. `CanarySigned v1` wire format **préservé** : pas de bump
   `CANARY_VERSION`, FROST sigs Ed25519 RFC 8032 byte-identique
   verifiable par verifier standard.

**Dimension G8 retrospective : PASS sans réserve.** Premier
pivot G8 effectif du projet documenté de façon exemplaire
(plan mis à jour avant code, retrospective écrite, findings S1
absorbés inline, wire format préservé, décision historique
`04c9621` honorée).

#### 3.E.2 Findings P2 Phase E

**E-1 — `canary_wire_bytes` utilise `serde_json::to_vec` non-JCS**
(audit verdict : **P2 carry S21 tech debt**) :
- Vérifié personnellement : `crates/nexus-shell-daemon-core/
  src/canary/mod.rs` helper `canary_wire_bytes(canary: &Canary)
  -> Result<Vec<u8>, CanaryError>` utilise `serde_json::to_vec`.
- Pré-existant S18 E2 `04c9621` (`canary.rs:212-213` identique).
- Impact sécurité **nul** : la signature couvre `canonical_
  bytes` JCS (RFC 8785 cross-language verifiable).
- Impact cross-language : ambiguïté subscriber Python qui
  re-serialize l'enveloppe (ordering champs). Mitigation
  actuelle : les champs du canary ne sont pas re-serialized
  côté coord, ils sont utilisés tels quels pour le probing.
- **Fix attendu S21** : migrer l'enveloppe vers `serde_jcs::
  to_vec` + tech debt entry `docs/rust/PATTERNS.md §TNN`.
- **Classification** : P2 carry. Ne bloque pas S21 Phase A.

**E-2 — `CanaryRegistry` sans vérif Ed25519 at ingest**
(audit verdict : **P2 carry S21 tech debt + décision
maturité**) :
- `canary_registry.py` `POST /api/canary/observed` accepte des
  payloads sans vérifier la signature Ed25519.
- **Délibéré et documenté** dans `docs/security/WARRANT_CANARY_
  HARDENING.md §2 T-canary-registry-spoof`.
- Mitigation actuelle : bearer token X-SBFB-Token loopback +
  CANARY.txt bootstrap pubkeys = trust root.
- Surface d'attaque : attaquant local avec bearer (uniquement
  lancé par le launcher local) pourrait injecter des
  observations fake pour masquer un vrai pubkey stale.
- **Fix long-terme** : verify Ed25519 at ingest (primitive
  existe, juste à wirer).
- **Classification** : P2 carry. **Décision de maturité
  pre-launch** à prendre S21 — observational-only acceptable
  beta T0-T1, hardening avant v1.0 go-live (T2+).

**E-3 — LOC dans kickoff §1.2 tableau HARDENING_ROADMAP**
(audit verdict : **close, clarification docs**) :
- Tableau kickoff §1.2 contient `~800 LOC`, `~500 LOC`, etc.
- Origine vérifiée : ces chiffres **proviennent de
  `docs/security/HARDENING_ROADMAP.md §3 S17 Phase D`**
  (projections roadmap écrites S17 octobre 2025).
- Il s'agit de **projections roadmap historiques**, pas de
  estimations amont S20.
- Politique `feedback_approach.md §Pas d'estimation LOC en
  amont` interdit l'estimation **amont** ; projections roadmap
  post-hoc retrospectives dans un doc d'horizon long-terme sont
  acceptables (règle README §6.7 « LOC rétrospective
  légitime »).
- **Statut** : close inline, pas un finding actif.

**E-5 — `expect()` dans FROST impl**
(audit verdict : **P3 accept**) :
- `crates/nexus-shell-daemon-core/src/canary/frost.rs:300` et
  `:315` utilisent `expect()`.
- Justifications inline valides : `trusted_dealer` self-
  produced (invariant local tenu par `frost-ed25519` API), pas
  d'input externe.
- Pattern `docs/rust/PATTERNS.md §P26` (expect-as-invariant)
  respecté. Acceptable.

**E-6 — `frost.rs:154` fmt residual**
(audit verdict : **P3 tracking**) :
- `cargo fmt --all --check` a retourné exit 1 avant Phase F
  sur 10 lignes de `frost.rs:154`. Fixé via chore séparé
  `1ad2def` (3 insertions / 7 deletions, pre-Phase F hygiene).
- Chore présent dans le range `3a7f0a3..131f32b` ✓ et séparé
  de Phase F docs-only.
- **Suggestion tracking S21** : ouvrir une micro-task kickoff
  S21 §6 « discipline G5 : ajouter `cargo fmt --all --check`
  au pre-commit skill step `nexus-phase-review SKILL Step 6` ».

### Track F — Phase F : wrap-up docs only (commits `f209168` + `54b0303` + `131f32b`)

**Verdict** : **PASS**.

**Findings résolus** :
- **F-SHA-1 résolu par `54b0303`** : `SPRINT_LOG.md` ligne S20
  et `sprint20_verification.md` ligne 5 ont été résolus
  (`<Phase F>` → `f209168`, `<HEAD>` → `f209168`). Confirmé par
  grep. ✓
- **F-SHA-2 `<wrap-up>` S17 SPRINT_LOG** : le commit `54b0303`
  a aussi résolu le `<wrap-up>` résiduel S17 row (carry S17
  placeholder tracé dans body commit). ✓
- **F-PHASE-F-REVIEW migré** : le `sprint20_phase_F_review.md`
  a été migré `.planning/active/` → `.planning/archive/v1.2/`
  par le commit `131f32b` produit par cette session d'audit en
  préambule (rattrapage CRAFT omis `f209168`). ✓ Toutes les
  conditions `sprint20_audit_plan.md §Checkpoint S21 kickoff`
  satisfaites post-`131f32b`.

**P3 non-bloquants** :
- F-AUDIT-PLAN-HEAD : `sprint20_audit_plan.md` lignes 61 et
  351 contiennent 2 résidus `<HEAD>` non résolus (l'audit plan
  **lui-même** référence son propre commit parent). Pattern
  acceptable : l'audit plan est écrit dans le commit Phase F
  lui-même, il ne peut pas connaître son SHA au moment de
  l'écriture. Non-bloquant audit gate. Résolution optionnelle
  S21 Phase 0 si on veut la trace propre.
- F-G8-ABSENT-PHASE-F : Phase F n'a pas émis de `sprint20_
  phase_F_preflight.md` (docs-only triviale). Jurisprudence
  S17/S18/S19 uniforme (Phase F wrap-up = exception légitime,
  classification P3 dans `nexus-phase-auditor SKILL §3ter`).
  Non-bloquant.

### Meta-track — Radicle-v1.0 activation tracking

**Verdict** : **PASS**.

- `grep -r radicle` dans le diff `3a7f0a3..131f32b` → **0
  changement wire**. ✓
- `docs/release/MIRROR_FALLBACK.md §3.1-3.8` non touché S20.
- Re-carry explicite dans `sprint20_audit_plan.md §Meta-track`
  + commit body `f209168` mentionne le re-carry S21.

**Décision carry S21** : re-carry Meta-1 confirmé S21. Owner
FlowUP, deadline **jour du tag `v1.0`** (pattern annual-ish
tant que v1.0 pas tag). Runbook `docs/release/MIRROR_FALLBACK.
md §3.1-3.8` reste self-contained. Aucune action S20 → S21
nécessaire sur ce track.

---

## 4. Findings list sorted by severity

### P0 : aucun

### P1 : aucun

### P2 (4 actifs + 6 résolus in-phase)

**Actifs — carry-overs ou fix attendu** :

| ID | Description | Classification | Fix attendu |
|---|---|---|---|
| E-1 | `canary_wire_bytes` `serde_json::to_vec` non-JCS (préexistant S18) | tech debt PATTERNS.md | S21 Phase 0 ou mi-sprint |
| E-2 | `CanaryRegistry` sans vérif Ed25519 at ingest (délibéré, observational-only) | tech debt PATTERNS.md + décision maturité | S21 Phase 0 décision pre-launch |
| C-PLAN-1 | `sprint20_plan.md §6.2` wire-point divergence docs-only | docs fix | S21 Phase 0 chore followup |
| Meta-1 | Radicle-v1.0 activation tracking | re-carry S18→S19→S20→S21 | Jour tag v1.0 |

**Résolus in-phase** (non-action) :
- B-1 double-wipe handler → `exit_only(0)` primitive séparée
  vérifiée `panic.rs:183-200` + `http.rs:759-767` ✓
- B-2 CRAFT design doc in-phase → documented G5 derogation
  acceptable ✓
- C-SEC-1 RwLock poisoned → graceful degradation applied
  inline ✓
- D-1 llguidance 0.7→1.7 → kickoff §D4 ligne 470 + PATTERNS
  note + Cargo.toml cohérents ✓
- D-2 PATTERNS §P30 logit-bias S21 état → note Sprint 20 added
  inline via `7ea68a6` ✓
- A-design-doc → accepté rétro (kickoff §D1-D2 ≈ 6 pages =
  design doc de facto) ✓
- F-SHA-1/2 placeholders → résolus `54b0303` ✓
- F-review migration → résolue `131f32b` ✓
- E-3 LOC kickoff → close (projections roadmap S17 historiques,
  pas estimations S20) ✓

### P3 (6 cosmétiques, non-bloquants)

| ID | Description |
|---|---|
| A-3 | `keystore.rs:860-862` `unwrap` sans commentaire INFALLIBLE |
| A-4 | `write_atomic` pas de cleanup `.enc.tmp` sur échec rename |
| D-3 | `task_response.rs:146` `expect` sans commentaire INFALLIBLE |
| D-4 | `llama_cpp.rs:442-448` `expand_tilde` sans validation component `..` (trust-root operator, impact faible) |
| F-AUDIT-PLAN-HEAD | 2 résidus `<HEAD>` dans `sprint20_audit_plan.md` lignes 61, 351 (non résolus `54b0303`) |
| F-G8-ABSENT-PHASE-F | Phase F wrap-up sans `_preflight.md` (jurisprudence, acceptable) |

---

## 5. Cap G7 carries S21

Règle (`docs/claude/README.md §6.2.1`) : **max 2 carry-overs par
sprint**, re-confirmés ligne par ligne dans kickoff §6.

**Proposition S21** :

- [x] **Meta-1 Radicle-v1.0 activation tracking** (re-carry
  S18→S19→S20→S21, owner FlowUP, deadline tag v1.0)
- [x] **C-PLAN-1 plan §6.2 wire-point divergence docs fix**
  (S21 Phase 0 chore followup, owner S21 executor)

**Hors cap G7** (= tech debt PATTERNS.md, pas carry scope,
invisible au cap) :
- E-1 canary enveloppe JCS migration → PATTERNS.md §TNN
- E-2 `CanaryRegistry` verify Ed25519 → PATTERNS.md §TNN +
  décision maturité S21

**Décisions reclassifiées** (mémo pour kickoff S21 §4 D5) :
- Rate-limit per-(consumer, worker, model) débloquée par PoW
  runtime wire Phase C S20 → **scope S21** cible
  (HARDENING_ROADMAP §3 S21).
- Client-side redaction SDK → **scope S21** cible (HARDENING_
  ROADMAP §3 S21).
- Tout le reste (Kudos-weighted admission, sandbox tool-
  calling, redundancy voting, ephemeral workers + VRAM wipe,
  honeypot Eclipse, Arti, domain fronting, PQC) reste dans ses
  fenêtres HARDENING_ROADMAP §3 sans carry S21.

---

## 6. Commits fix attendus (aucun bloquant)

**Aucun commit `fix(sprint20): ...` n'est requis avant S21 Phase A.**
Le verdict est PASS, pas CONDITIONAL PASS — les carries E-1,
E-2, C-PLAN-1 sont tech debt docs, pas fix code.

**Si l'utilisateur veut fermer S20 avec un « gate close clean-
up »** avant d'ouvrir S21 (pattern S19 `1af90b3..3a7f0a3`),
voici 3 chores optionnels courts :

1. **`chore(sprint20): tech-debt PATTERNS.md entries for canary
   JCS envelope + registry Ed25519 verify`**
   — 2 entrées TNN+1 et TNN+2 dans `docs/rust/PATTERNS.md
   §Tech debt` avec owner + fix path (cf. §7 ci-dessous pour
   le contenu proposé).
2. **`chore(sprint20): fix plan §6.2 wire-point divergence
   post-audit`**
   — edit `archive/v1.2/sprint20_plan.md §6.2 + §6.4` avec
   note pointeur vers audit findings §C-PLAN-1. 3-5 lignes.
3. **`chore(sprint20): resolve residual <HEAD> placeholders in
   sprint20_audit_plan.md`**
   — 2 occurrences lignes 61, 351 → `f209168`. Cosmétique pur.

**Recommandation auditeur** : option 1 est la plus utile (crée
les entrées de suivi tech debt trackables long terme). Option 2
évite la confusion future pour quiconque relira le plan archivé.
Option 3 est cosmétique pure — OK à skipper.

**Le verdict PASS tient sans ces 3 chores optionnels.** Ils
peuvent aussi être livrés en Phase 0 S21 kickoff chore, au
choix de l'utilisateur.

---

## 7. P2 à logger en tech debt PATTERNS.md (si chore §6 option 1 adopté)

Les 2 entrées à ajouter à `docs/rust/PATTERNS.md §Tech debt` :

```markdown
### TNN (S20 audit P2-E-1) — canary wire envelope → JCS

- **Location** : `crates/nexus-shell-daemon-core/src/canary/mod.rs`
  helper `canary_wire_bytes(canary: &Canary) -> Result<Vec<u8>,
  CanaryError>`.
- **Issue** : enveloppe canary broadcast utilise `serde_json::
  to_vec`. Signature couvre `canonical_bytes` JCS donc impact
  sécurité nul, mais ambiguïté cross-language pour subscribers
  Python qui re-serialize.
- **Fix path** : migrer vers `serde_jcs::to_vec`. Test de
  non-régression : snapshot cross-language Python ↔ Rust.
- **Owner** : S21 executor.
- **Rationale originale** : décision S18 E2 `04c9621` avait
  gardé `serde_json` pour simplicité, finding audit S20 promu
  P2 par rigor signal G4.
```

```markdown
### TNN+1 (S20 audit P2-E-2) — CanaryRegistry verify Ed25519 at ingest

- **Location** : `packages/nexus-coordinator/src/nexus_
  coordinator/canary_registry.py` `POST /api/canary/observed`
  handler.
- **Issue** : registry observational-only, pas de verify
  Ed25519 at ingest. Attaquant local avec bearer token X-SBFB-
  Token pourrait injecter observations fake.
- **Mitigation actuelle** : bearer token loopback + CANARY.txt
  bootstrap pubkeys = trust root suffisant beta T0-T1.
- **Fix path** : verify Ed25519 at ingest via `nexus-core-py
  verify_canary` binding (primitive existe, juste à wirer).
  Test de non-régression : spoof `CanaryObservation` avec
  mauvaise signature → 401.
- **Owner** : S21 executor.
- **Décision maturité** : à trancher S21 Phase 0 si hardening
  avant v1.0 go-live ou acceptable observational pour beta
  fermée T0-T1.
- **Rationale originale** : `docs/security/WARRANT_CANARY_
  HARDENING.md §2 T-canary-registry-spoof` classifie
  explicitement le threat, mitigation trust-root doc'd.
```

---

## 8. Notes on audit completeness

1. **Design doc Phase A absent** : audité à distance (kickoff
   §D1-D2 ≈ 6 pages avec alternatives rejetées complètes =
   design doc de facto). Recommandation futur : un
   `.planning/research/SN_phase_X_*_design.md` séparé par phase
   big rock structurant pour faciliter la lecture vs scrolling
   dans un kickoff de 769 lignes. Discipline à resserrer S21.

2. **G8 pivot retrospective Phase E** : premier pivot G8
   effectif du projet. Dimension retrospective fully audited
   via les artefacts `pivot_proposal.md` + `preflight.md` +
   `phase_E_review.md §G8 traceability`. Processus conforme
   aux 7 garde-fous (evidence-based, Day 0 preserve, wire
   format preserve, test budget cap, theme respect, pas YAGNI,
   retrospective trackée). **Cas exemplaire** à référencer
   S21+ si un autre pivot G8 se déclenche.

3. **Pre-launch protocol policy vérifiée** : `BLOB_VERSION
   = 0x01`, `TASK_RESPONSE_VERSION = 1`, `CANARY_VERSION = 1`,
   `ANNOUNCEMENT_VERSION = 1` tous inchangés S20. Aucun
   tolerant decoder multi-version introduit. FROST signatures
   préservent RFC 8032 Ed25519 wire-compat. ✓

4. **Scope cuts tous vérifiés** (kickoff §8, 14 items) :
   TPM/SE/StrongBox, HPKE peer-restore, rate-limit per-
   consumer, client-side redaction SDK, kudos-weighted gossip
   admission, sandbox tool-calling, redundancy voting,
   ephemeral workers + VRAM wipe, honeypot Eclipse, re-run
   sampling + DNS fallback DHT, Arti Tor bridge, domain
   fronting Snowflake-WebRTC, PQC migration, `actions/checkout
   @v4` pin SHA sweep. **0 leak** dans le diff S20.

5. **Tests rejoués** : Rust workspace complet (642 pass,
   0 skip, 11.5 s). Autres suites non rejouées (fallback
   `verification.md §2` — accepté pour audit rapide, les
   suites Python/Node sont stables sur plusieurs sprints). Si
   doute, l'utilisateur peut rejouer la suite complète de la
   section §4.3 `docs/claude/README.md` avant ouvrir S21.

6. **Vérifications critiques indépendantes** : B-1 double-wipe
   (vérifié `panic.rs:183-200` + `http.rs:759-767`), D-1
   llguidance 1.7 (vérifié kickoff §D4 ligne 470), dead-man
   switch intact (vérifié workflow `canary-monthly.yml` +
   grep `CanarySigner::sign` call-sites), pre-launch versions
   (vérifié 4 consts). Tous les findings pre-review intra-
   phase ont été cross-checked avec le code actuel au moment
   de l'audit — les 2 P2 originalement flagués comme non-
   résolus par un rapport pré-existant (auto-généré par hook/
   skill en background) ont été **indépendamment confirmés
   résolus** dans le code.

7. **Déviation workflow audit gate vs Phase F review** : la
   phase review Phase F avait flagué F-SHA-1 P2 (résolu
   post-commit attendu). Le commit `54b0303` a résolu F-SHA-1
   **avant** l'audit gate (pattern S18/S19). L'auditeur valide
   cette pré-résolution — conforme pattern permanent.

8. **Déviation ajout `131f32b`** : cette session d'audit a
   produit un commit `chore(planning)` AVANT de démarrer
   l'audit pour rattraper une migration oubliée (Phase F
   review file resté dans `active/`). Action automatique
   conforme au protocole « cas B CRAFT → commit chore avant
   phase » adapté à « cas A CRAFT → commit chore avant audit ».
   Range audit élargi en conséquence de `3a7f0a3..54b0303`
   à `3a7f0a3..131f32b`.

9. **Délégation agent Explore thorough** : l'audit multi-
   track a été délégué à un agent Explore en mode « very
   thorough » pour 7 tracks (A-F + meta), puis synthétisé par
   l'auditeur principal avec vérifications indépendantes
   critiques (dead-man switch, pre-launch versions, test
   counts, fix B-1, fix D-1). Rapport agent conservé dans le
   thread de session.

---

## 9. Checkpoint S21 kickoff

Les 4 conditions `sprint20_audit_plan.md §Checkpoint S21
kickoff` :

1. ✓ `sprint20_audit_findings.md` livré avec verdict explicite
   (**PASS**) — ce document.
2. ✓ Aucun P0/P1 à fixer — section §6.
3. ✓ Meta-1 Radicle-v1.0 re-carry S21 confirmé — §5.
4. ✓ `sprint20_phase_F_review.md` créé et migré archive
   (`131f32b`).

**Sprint 21 Phase A non bloqué. Kickoff S21 peut démarrer
immédiatement** (ou, au choix de l'utilisateur, après les 3
chores optionnels §6).
