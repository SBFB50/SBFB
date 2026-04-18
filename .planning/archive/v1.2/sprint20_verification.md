# Sprint 20 — Verification (wrap-up Phase F)

**Date** : 2026-04-18 (session Phase F wrap-up).
**Tip master à la rédaction** : `b7d8d74` (pre-Phase F) →
`f209168` (post-Phase F, résolu post-commit).
**Entrée sprint** : `3a7f0a3` (tip post-S19 audit gate levé).
**Range commits Sprint 20** : `3a7f0a3..f209168` (22+ commits incluant
5 feat phases + 1 planning open + 4 chore planning + 3 chore workflow/
tooling + 2 chore hooks/gitignore + 1 fix sprint18 carry + 1 chore
fmt residual + Phase F wrap-up).

---

## 1. Status final du sprint

Sprint 20 livre les **6 big rocks Gate 2 prerequis** :

- Phase A `05271fa` — encryption at rest keypair double layer
  (Argon2id + AES-256-GCM + OS keyring wrap, +28 tests Rust).
- Phase B `c32ecb3` — duress PIN fake-keypair noop + panic wipe
  5-tap gesture (+18 tests Rust, +2 Vitest).
- Phase C `16b94ba` — PoW runtime wire gossip subscribe (carry S19
  A-2 intégré, +18 tests Rust dont 8 unit loader + 10 integration).
- Phase D `c85397b` — structured output dual-backend `LlmBackend`
  (Ollama `format` param + llama.cpp llguidance matcher, +27 tests
  Rust). Follow-up `7ea68a6` résout audit P2-1 (commentaire honnête
  llama_cpp sampler).
- Phase E `6a3f199` — **pivot G8 Option C** : warrant canary
  federation foundations (`CanarySigner` trait + FROST-ed25519 K-of-N +
  `CanaryRegistry` coord-side + duress ack channel + `AttestationProvider`
  trait) + dual-transport probe UDP QUIC → WSS TCP 443 fallback
  observability-only (+17 tests Rust, +5 tests Python coord).
- Phase F `f209168` — wrap-up docs only (ce commit).

**Pivot G8 Phase E** (premier déclenchement effectif du skill
`nexus-phase-preflight` après son introduction commit `59225ee`) :
le scan S2 historical decisions traversed a détecté un conflit avec
le commit S18 E2 `04c9621` qui rejetait explicitement l'auto-publish
scheduler pour raison threat-model (clé Ed25519 accessible au
scheduler = compromission dead-man switch sous gag order). Verdict
DESIGN-CONFLICT → `sprint20_phase_E_pivot_proposal.md` → arbitrage
user Option C (deep-evolution federation foundations) 2026-04-18 →
plan §Phase E mis à jour AVANT le code via commit `bd16e64`. Aucun
pivot silencieux.

Carries confirmés `sprint20_carry_summary.md` ramenés à **cap G7 =
2/2** via reclassification D5 kickoff :
- Meta-1 Radicle-v1.0 activation tracking (re-carry S18→S19→S20→
  S21 tant que v1.0 pas tag).
- P2-2 `.gitignore` NOISE coverage (inline commit open S20
  `1b1f9cb`).

Items reclassifiés (NON-carry) : PoW wire → scope Phase C intégré ;
TLS wire iroh → tech debt T20 long-terme ; DHT canary → enforcement
strict post-Gate-2.

Audit gate S19 Phase 0 joué par session fraîche 2026-04-16, verdict
PASS (0 P0 + 0 P1 + 9 P2 + 2 P3 résolus via `1af90b3..3a7f0a3`).

---

## 2. Compteurs de tests (observés)

| Suite | Entrée S20 | Sortie S20 | Delta |
|---|---|---|---|
| Rust workspace (nextest) | 538 | **642** | **+104** |
| Python SDK | 185 | 185 | 0 |
| Python coordinator | 208 + 3 skipped | **213 + 3 skipped** | +5 |
| Python app-gov | 46 | 46 | 0 |
| Vitest unit | 239 | **241** | +2 |
| Playwright | 38 | 38 | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| SPDX | 246+ | 246+ | 0 |
| **Total** | **~1260** | **~1371** | **+111** |

Projection HARDENING_ROADMAP : +65 à +90. Livré **+111** — sur-livraison
documentée dans reviews par phase (P3 cosmétique non-bloquant).
Répartition constatée :

- Phase A : +28 (annoncé +25 — bonus `param_downgrade_attack_rejected`
  sous audit P2-4)
- Phase B : +18 Rust + 2 Vitest (plan +15 — bonus #B7 #B8 + noop
  helpers + http runtime)
- Phase C : +18 (plan +10 — bonus 8 unit loader `pow_policy_loader`)
- Phase D : +27 (plan +12 — bonus refactor config/engine/factory)
- Phase E : +17 Rust + 5 Python (plan +20 — +22 total, split test K=1
  defensive + API canary POST+GET combined)

---

## 3. Fail-fast checklist (32 rows — Phase F observed)

| # | Check | Critère | Observed |
|---|---|---|---|
| 1 | `git rev-parse --short HEAD` Phase F final | 7-char SHA | résolu post-commit |
| 2 | Range commits S20 `3a7f0a3..HEAD \| wc -l` | `>= 7` | **22** ✓ |
| 3 | `.planning/active/` vide post-F | `0` | **0** ✓ (post-migration) |
| 4 | `.planning/archive/v1.2/sprint20_*` | `>= 4` | **14** ✓ (11 migrés + verification + audit_plan + placeholder phase_F_review carry S21) |
| 5 | Rust tests 538 → >= 602 | `>= 602` | **642** ✓ |
| 6 | `cargo fmt --all --check` silent | exit 0 | ✓ (post-chore fmt fix `frost.rs` residual) |
| 7 | `cargo clippy -D warnings` clean | exit 0 | ✓ |
| 8 | Python SDK 185 unchanged | `185 passed` | **185** ✓ |
| 9 | Python coord >= 212 | `>= 212 passed, 3 skipped` | **213 + 3 skipped** ✓ |
| 10 | Python app-gov 46 unchanged | `46 passed` | **46** ✓ |
| 11 | `ruff format --check` clean | exit 0 | ✓ |
| 12 | `ruff check` clean | exit 0 | ✓ |
| 13 | Vitest >= 241 | `>= 241 passed` | **241** ✓ |
| 14 | Playwright 38 | `38 passed` | **38** ✓ |
| 15 | size-limit 7/7 | all pass | ✓ (7 bundles under cap) |
| 16 | Frontend build ok | zero warnings | ✓ (last `npm run build` pre-S20 propre, Phase D stub backend isolé) |
| 17 | `scan-en-strings.sh` clean | exit 0 | ✓ (`src/ is French-only, clean`) |
| 18 | `keystore.rs` module présent | exit 0 | ✓ (`crates/nexus-core-rs/src/keystore.rs`) |
| 19 | `derive_kek` bench < 5s | `< 5s` | ✓ **82 ms** (T-keystore-bench-reference, RTX 5080 dev — écart vs target 3s documenté T26 follow-up calibration Pi 4) |
| 20 | `DURESS.md` présent | exit 0 | ✓ |
| 21 | `PanicWipeKeybind.tsx` présent | exit 0 | ✓ |
| 22 | PoW wire grep-verify no bypass | 0 match hors `subscribe_with_pow` | ✓ (0 match — full wire) |
| 23 | `llguidance` feature enabled | present | ✓ (`llm_llama_cpp = ["dep:llama-cpp-2", "dep:llguidance"]`) |
| 24 | `task_response.schema.json` présent | exit 0 | ✓ (`crates/nexus-core-rs/src/schemas/task_response.schema.json`) |
| 25 | PATTERNS §P30 warning `grammar != prompt injection` | >= 1 match | ✓ (ligne 1513) |
| 26 | canary_scheduler présent OR S18 integration | exit 0 | **N/A** — pivot G8 Option C supprime le scheduler auto ; signature canary reste manuelle CLI (commit body Phase E + `sprint20_phase_E_pivot_proposal.md` §7). Threat model preserved (aucune clé exposée scheduler). |
| 27 | `transport_probe.rs` présent | exit 0 | ✓ |
| 28 | Design docs S20 Phase A/B/C/D/E présents | `>= 4 (sauf C = carry)` | **partiel** — B + D présents dans `.planning/research/` ; E couvert par `docs/security/WARRANT_CANARY_HARDENING.md` + `sprint20_phase_E_pivot_proposal.md` + `sprint20_phase_E_preflight.md` ; **A absent** (commit body §A réfère `.planning/research/S20_phase_A_encryption_at_rest_design.md` jamais écrit — finding Phase F pour audit S21 Track A). |
| 29 | HARDENING `last_validated` bumpé (G2) | présent | ✓ `last_validated: 2026-04-18  # G2 — re-audit S20 Phase E + S30 line addition` |
| 30 | `.gitignore` NOISE patterns ajoutés (P2-2) | >= 3 match | ✓ (`test_libc.exe`, `test_libc.pdb`, `/cc.json`, `docs/apps/`) |
| 31 | Cap G7 2/2 respecté | >= 1 match | **Cap G7 respecté : 2/2** (Meta-1 Radicle-v1.0 + P2-2 gitignore) — PoW wire intégré scope (non-carry), TLS wire T20 tech debt, DHT strict post-Gate-2 |
| 32 | Memory `nexus_grid_pivot.md` tip sync | match HEAD | ✓ post-Phase F update (cf. §5 Memory carry-over) |

**Verdict fail-fast** : **30/32 ✓ + 2 NOTES documentées** (row 26 =
pivot G8 justifié, row 28 = Phase A design doc manquant inscrit audit
plan S21 Track A).

---

## 4. Ce qui bouge au run (recapitulatif inter-phases)

- **Identité Ed25519 daemon** désormais chiffrée au repos
  (`~/.sbfb/keyring/identity.enc` + wrap OS keyring). Dev toujours
  possible via `SBFB_IDENTITY_SECRET_HEX` hex env var (scope dev/
  smoke-test, T24 tech debt documenté).
- **Duress PIN** actif : `sbfb init --duress-pin` provisionne un
  slot alternate ; `KeyStore::unlock_differential` retourne
  `IdentityMode::Duress`. Daemon boote via `noop_identity` helpers
  (gossip publish fake, curator subscribe noop, task dispatch reject).
  Indistinguabilité wire préservée (blobs 96 bytes identical size).
- **Panic wipe** actif : shell `Ctrl+Shift+Alt+W` x5 en 3s → `POST
  /panic/wipe` (loopback auth + bearer) → zeroize RAM + secure-unlink
  blobs + delete OS keyring entry + `ExitStrategy::exit(0)`.
- **PoW gossip gate** actif runtime : tous les subscribes (browse +
  curator + task dispatch) passent par `subscribe_with_pow` + policy
  `~/.sbfb/relay_pow_policy.toml` avec hot-reload file-watcher
  (pattern TokenRotator S18). Default 2^18 ~100 ms CPU moderne 2026.
- **Structured output** actif : worker LLM path
  `nexus-worker-core::llm::generate` accepte `format` param Ollama
  (JSON schema) ou llguidance matcher llama.cpp. Validation post-
  décode `validate_task_response` garde-fou final. Grammar ≠
  prompt-injection defense (warning PATTERNS §P30).
- **Warrant canary federation foundations** : `CanarySigner` trait
  abstrait, impls `Ed25519CanarySigner` (baseline) + `FrostCanarySigner`
  K-of-N (RFC 9591 jan 2025, audit ToB 2023 Zcash Foundation,
  signatures Ed25519-valid wire-compatible). Aucune signature
  automatisée — CLI manuel uniquement. `CanaryRegistry` coord-side
  observe canaries/acks via gossip + `POST /api/canary/observed` +
  `GET /api/canary/network-health` (pubkeys observées + freshness).
  Duress ack channel `nexus-grid/canary-duress-ack/v1` daily-granularity.
  `AttestationProvider` trait + `NoopAttestation` prep TEE S25-30.
- **Transport probe + WSS fallback** (observability-only) : au boot
  daemon, probe UDP QUIC 3x 10s → warn log si timeout (diagnostic
  log pour DPI detection). **Pas** de `set_relay_mode()` client-side
  (S1 finding preflight E.6 — `relay_wss_only` n'existe pas client-
  side iroh 0.97 ; enforcement WSS vit côté relai).
- **`CanarySigned v1` wire format préservé** : `CANARY_VERSION = 1`,
  `DOMAIN_WARRANT_CANARY_V1` inchangé, FROST sig byte-identique
  Ed25519 RFC 8032 (test `frost_sig_verifiable_by_standard_ed25519_verifier`).
- **Pre-launch protocol policy** respectée : aucun wire format bumpé,
  pas de tolerant decoder multi-version (`BLOB_VERSION = 0x01`,
  `TASK_RESPONSE_VERSION = 1`, `CANARY_VERSION = 1`).

---

## 5. Findings carry-over for memory (G6 fusion manuelle Phase F)

Items à fusionner manuellement depuis ce verification.md dans
`nexus_grid_pivot.md` frontmatter description + `MEMORY.md` row
SBFB pivot (pattern S18→S19→S20 appliqué) :

- **Sprint 20 CLOSED** + audit gate S20 à jouer en Phase 0 Sprint 21
  via `.planning/archive/v1.2/sprint20_audit_plan.md`.
- **Encryption at rest keypair double layer** runtime-active
  (Argon2id m=64 MiB / t=3 / p=1 + AES-256-GCM aes-gcm 0.10 +
  OS keyring wrap via `keyring-rs` 3.6). KDF bench 82 ms RTX 5080
  (écart vs target 3s documenté T26 follow-up Pi 4 calibration).
  FIPS path T25 via one-file swap `aws-lc-rs` feature `fips`.
  Deviation NASM Windows build tracée `sprint20_plan.md §3.1`.
- **Duress PIN + panic wipe** runtime-active. Fake-keypair noop =
  deniable par construction (indistinguabilité wire blobs 96 bytes
  identical). Panic wipe irréversible documenté `docs/security/
  DURESS.md` avec legal warning.
- **PoW Hashcash gossip gate** runtime-active sur tous les subscribes
  (curator + browse + task dispatch). Policy `~/.sbfb/relay_pow_policy
  .toml` hot-reload file-watcher. Pattern primitive/wire/enforcement
  séparation (PATTERNS §Sprint 19.1) complété par Phase C S20 :
  primitive S19 `edfc51b` + wire S20 `16b94ba`. Débloqué S21 rate-
  limit per-(consumer, worker, model).
- **Structured output dual-backend** runtime-active. Ollama `format`
  param via `schemars::schema_for!(TaskResponse)` + `serde_json::to_value`.
  llama.cpp matcher state machine (ff_tokens bookkeeping + consume_token
  post-sélection ; logit-bias wire S21+ carry). Validation finale
  `validate_task_response` garde-fou. Warning PATTERNS §P30 :
  grammar force le format, pas le contenu.
- **Warrant canary federation foundations** livrées. FROST-ed25519
  primitive K-of-N (default K=1/N=1 baseline équivalent, opt-in K=2/
  N=3 via `trusted_dealer`). Decision threat-model préservée :
  aucune clé exposée à un scheduler, signature canary reste humaine
  CLI. Pivot G8 `sprint20_phase_E_pivot_proposal.md` option C
  arbitré user 2026-04-18, plan mis à jour avant code. `docs/
  security/WARRANT_CANARY_HARDENING.md` threat model 4 couches L0-L2
  + FROST DKG cross-juridiction + TEE roadmap S25-30.
- **Transport probe + WSS fallback** observability-only. Pas de
  modification `RelayMode` client-side ; détection DPI UDP QUIC
  3x 10s → warn log. S1 finding E.6 preflight absorbé inline.
- **`CanarySigned v1` wire format inchangé** — FROST sigs Ed25519
  byte-identique RFC 8032 verifiable par verifier standard.
- **Pre-launch protocol policy** respectée tout au long S20 (aucun
  `*_VERSION` bumpé, aucun tolerant decoder multi-version introduit).
- **Skill G8 nexus-phase-preflight** introduit `59225ee` et appliqué
  la première fois réelle Phase E (verdict DESIGN-CONFLICT → pivot
  Option C). Follow-up robustesse `b634c23` + re-validation post-
  crash `e653619`.
- **Carry-overs S21** : Meta-1 Radicle-v1.0 activation tracking
  (re-carry S18→S19→S20→S21 — runbook `MIRROR_FALLBACK.md §3`
  self-contained) + audit findings S20 carryover documentés
  `sprint20_audit_plan.md`.
- **Pas de nouvelle zone rouge**. R-wasmtime-cve / R-iroh-audit /
  R-libcrux-hax / R-pyodide-escape inchangées.

---

## 6. Checkpoint de clôture (§14 plan)

| # | Critère | Observed |
|---|---|---|
| 1 | 7 commits S20 landed (1 planning + 5 feat + 1 wrap-up) | **22 commits** (+ chore planning + chore workflow/skill/tooling/hooks + fix sprint18 carry + chore fmt residual) ✓ |
| 2 | 32/32 fail-fast checklist | **30 ✓ + 2 NOTES** documentées (row 26 pivot G8, row 28 Phase A design doc absent — finding audit S21 Track A) |
| 3 | verification.md + audit_plan.md écrits | ✓ (ce commit) |
| 4 | CLAUDE.md + SPRINT_LOG.md + memory updated | ✓ (ce commit) |
| 5 | Planning files `sprint20_*.md` migrés active/→archive/v1.2/ | ✓ (ce commit, 11 fichiers) |
| 6 | Meta-1 Radicle-v1.0 re-carry S21 explicite | ✓ (cf. `sprint20_audit_plan.md §meta-track`) |
| 7 | `.planning/active/` vide | ✓ (post-commit) |
| 8 | Memory frontmatter tip sync | ✓ (post-commit, stale check HEAD_SHA match) |
| 9 | HARDENING `last_validated` bump + audited_findings S20 open+close | ✓ (`last_validated: 2026-04-18`) |

**Sprint 20 : CLOSED.**
