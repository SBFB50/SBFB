# Sprint 30 — Plan d'execution detaille

**Ecrit** : 2026-04-26 (meme session que kickoff).
**Tip master** : `dcdda7e`.

---

## §1 Etat verifie a l'entree

| Metrique | Valeur |
|---|---|
| Tip master | `dcdda7e` |
| Rust tests | 856 passed |
| SDK tests | 195 passed |
| Coordinator tests | 393 passed + 36 failed (PyO3 stale) + 6 skipped |
| Gov tests | 46 passed |
| Vitest | 269 passed |
| Playwright | ~43 (41+2f env) |
| size-limit | 4/4 |
| clippy warnings | 0 |
| cargo fmt | clean |

---

## §2 Decisions Day 0 (gelees) — rappel synthetique

- **D1** : Niveau 1 warrant canary via trusted dealer DKG frost-ed25519
  2.1. NoopAttestation maintenu. CLI wiring seulement, pas recrutement.
- **D2** : CI cross-platform via GitHub Actions multi-OS (ubuntu + macOS).
  Scope nexus-events-core platform writers.
- **D3** : blob-serve isolation via COOP/COEP headers. Full process
  isolation = LT.
- **D4** : G2 HARDENING_ROADMAP refresh (3 triggers, last_validated S30,
  S31 Tor transport). Pas d'upgrade iroh/arti S30.
- **D5** : Split inference research = design doc only, pas de code.

---

## §3 Research consulte

- context7 arti-client (Arti, /git_gitlab_torproject_org/tpo_core_arti,
  1597 snippets, reputation High) : TorClient API create_bootstrapped +
  connect + SOCKS proxy. arti 2.0.0 stable 2026-02-07.
- WebSearch nym-sdk : publication crates.io pausee, Lewes Protocol.
- WebSearch frost-ed25519 : toujours 2.1.0. API stable.
- WebSearch iroh : 0.98.0 publie 2026-04-17, 0.97.0 = 2026-03-16.
- WebSearch arti : 2.0.0 stable 2026-02-07, LTS branche 2.x.
- WebSearch openai-agents-python : 0.14.6, sandbox agents, openai v2.x.
- WebSearch NVIDIA H100 CC : GA CUDA 12.4 r550, pas de nouveau driver.
- HARDENING_ROADMAP triggers : 3/12 actifs.
- WARRANT_CANARY_HARDENING.md §4 : DKG trusted dealer procedure,
  generate_with_dealer, round1/round2/aggregate sequence.
- Codebase grep : canary/frost.rs FrostCanarySigner existant, pas de
  DKG ceremony code. blob_serve.rs CSP connect-src 'none', pas
  COOP/COEP. Platform writers cfg-gated, 0 CI cross-platform.

---

## §4 Phase A — P2 batch S29 audit

### §4.1 Scope

Fixes les 7 items P2/P3 de l'audit S29 + documentation gap status :

1. **P2-AUDIT-1** : `docs/security/HARDENING_ROADMAP.md` ligne ~734
   "opentelemetry 0.27" → "0.31". Sed 1 ligne.
2. **P3-AUDIT-1** : `crates/nexus-trace-core/src/lib.rs` ligne 8
   docstring "OpenTelemetry 0.28+" → "OpenTelemetry 0.31". Sed 1 ligne.
3. **P2-REVIEW-B-1** : `packages/nexus-coordinator/src/nexus_coordinator/
   consent.py` — refactorer `_populate_threat_fields()` en pure function
   qui retourne un dict au lieu de muter le `config` dict en place. Les
   callers (GET/POST consent) recoivent le resultat et le mergent.
4. **P2-REVIEW-D-1** : `crates/nexus-executor/src/main.rs` ligne ~34
   chemin relatif `"traces/executor.jsonl"` — evaluer si resolution
   depuis `ShellDaemonPaths` est souhaitable ou si l'asymetrie est
   intentionnelle (isolation processus). Si intentionnelle : ajouter
   commentaire inline expliquant pourquoi. Si resolution : passer le
   base path via env var ou argument CLI du broker.
5. **P2-REVIEW-B-2** : `docs/security/THREAT_MODEL.md` §9.5 — ajouter
   note explicite "output filter designed S23, wire e2e S31 target".
6. **P2-REVIEW-C-1** : `crates/nexus-executor/src/task_runner.rs` —
   ajouter commentaire defense-in-depth confirmant que le stub ne peut
   pas executer de code arbitraire + carry S31 note.
7. **P3-REVIEW-C-1** / **P3-REVIEW-D-1** : cosmetiques si memes
   fichiers touches, sinon skip.

### §4.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/security/HARDENING_ROADMAP.md` | Fix "0.27"→"0.31" |
| `crates/nexus-trace-core/src/lib.rs` | Fix docstring "0.28+"→"0.31" |
| `packages/nexus-coordinator/src/nexus_coordinator/consent.py` | Refactor pure function |
| `packages/nexus-coordinator/tests/test_consent.py` | Adapter tests au refactor |
| `crates/nexus-executor/src/main.rs` | Trace path fix ou commentaire |
| `crates/nexus-executor/src/task_runner.rs` | Defense-in-depth commentaire |
| `docs/security/THREAT_MODEL.md` | §9.5 gap status note |

### §4.3 Tests plan

1. `test_consent_populate_pure_function` — verifie que la nouvelle
   pure function retourne les champs sans muter l'input
2. `test_consent_residual_threats_field` — regression existing
3. `test_consent_level_threat_notes` — regression existing
4. Tests Rust existants nexus-trace-core — regression (pas de nouveau)
5. Tests Rust existants nexus-executor — regression (pas de nouveau)

### §4.4 Critere d'acceptation

```bash
# Rust
cargo nextest run -p nexus-trace-core --locked
cargo nextest run -p nexus-executor --locked
cargo clippy --workspace --all-targets --locked -- -D warnings

# Python
uv run ruff format --check packages/nexus-coordinator/
uv run ruff check packages/nexus-coordinator/
uv run pytest packages/nexus-coordinator/tests/test_consent.py -q

# Verification
grep -n "0\.27" docs/security/HARDENING_ROADMAP.md | grep -i otel
# → 0 matches (fix confirme)
```

### §4.5 Commit cible

```
feat(sprint30): Sprint 30 Phase A — P2 batch S29 audit (7 items)

## Scope

Fix 7 items P2/P3 audit S29 : HARDENING_ROADMAP otel 0.27→0.31,
nexus-trace-core docstring, consent.py pure function refactor,
executor trace path, THREAT_MODEL §9.5 gap note, task_runner
defense-in-depth.

## Test delta cumulatif

+1 test coord (test_consent_populate_pure_function). Regression
verte sur nexus-trace-core, nexus-executor, test_consent.py.
```

---

## §5 Phase B — Phase dette (sprint pair §6.2.1 Regle 1)

Dependencies : Phase A (tip propre pour CI)

### §5.1 Scope

Phase reservee exclusivement aux items differes (§6.2.1 Regle 1).

**P2-B-1-S28 CI Linux/macOS writers (3/3 MANDATORY)** :
- Creer `.github/workflows/ci-cross-platform.yml`
- Matrice : `ubuntu-latest` + `macos-latest`
- Jobs : `cargo nextest run -p nexus-events-core --locked` +
  `cargo clippy -p nexus-events-core --locked -- -D warnings` +
  `cargo fmt -p nexus-events-core --check`
- Trigger : push master + PR
- Timeout : 15 min par job
- Ajuster cfg-gates si compile errors Linux/macOS (probable :
  `libsystemd` dep Linux-only)

**P2-C-1-S28 blob-serve isolation (2/3)** :
- Ajouter headers COOP/COEP dans `blob_serve.rs` :
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`
  - `X-Content-Type-Options: nosniff` (si absent)
- Verifier `sandbox="allow-scripts"` iframe toujours en place
- Test unitaire nouveau : `test_blob_serve_security_headers`
- Test Playwright : regression apps iframe (hello-world-app)

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `.github/workflows/ci-cross-platform.yml` | Nouveau workflow CI multi-OS |
| `crates/nexus-events-core/src/lib.rs` | cfg-gate adjustments si compile errors |
| `crates/nexus-events-core/Cargo.toml` | deps conditionnelles Linux/macOS si besoin |
| `crates/nexus-shell-daemon-core/src/blob_serve.rs` | Headers COOP/COEP |
| `web/tests/` | Regression Playwright si besoin |

### §5.3 Tests plan

1. `test_blob_serve_security_headers` — verifie COOP + COEP +
   X-Content-Type-Options presents dans la reponse
2. `test_blob_serve_csp_unchanged` — regression CSP connect-src
   'none' toujours present
3. CI cross-platform : les tests nexus-events-core passent sur
   ubuntu-latest + macos-latest (verification manuelle premier run)
4. Playwright regression hello-world-app iframe (si applicable)

### §5.4 Critere d'acceptation

```bash
# Local (Windows)
cargo nextest run -p nexus-events-core --locked
cargo nextest run -p nexus-shell-daemon-core --locked
cargo clippy --workspace --all-targets --locked -- -D warnings

# CI verification
# Push + verifier que GitHub Actions job passe sur ubuntu + macOS

# blob-serve headers
cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(blob_serve)'
```

### §5.5 Commit cible

```
feat(sprint30): Sprint 30 Phase B — dette pair CI cross-platform +
blob-serve isolation

## Scope

Phase dette sprint pair (§6.2.1 Regle 1).
- P2-B-1-S28 (3/3 MANDATORY) : GitHub Actions ci-cross-platform.yml
  matrice ubuntu-latest + macos-latest, scope nexus-events-core.
- P2-C-1-S28 (2/3→3/3) : blob-serve COOP/COEP + X-Content-Type-Options
  headers.

## Test delta cumulatif

+2 tests Rust (blob_serve_security_headers, blob_serve_csp_unchanged).
CI cross-platform : nexus-events-core compile + teste sur Linux/macOS.
```

---

## §6 Phase C — Warrant canary Niveau 1 FROST DKG code wiring

Dependencies : Phase A (canary module existant), pas de dep Phase B

### §6.1 Scope

Implementer le code path DKG ceremony pour Niveau 1 warrant canary :

1. **DKG trusted dealer** : nouveau module `canary/dkg.rs` dans
   `nexus-shell-daemon-core`. Wrapper autour de
   `frost_ed25519::keys::generate_with_dealer()` avec params K/N.
   Output : `KeyPackage` par participant + `PublicKeyPackage` public.
   Serialisation JSON pour distribution.

2. **Signing ceremony** : nouveau module `canary/ceremony.rs`.
   Wrapper autour des rounds FROST :
   - `round1_commit()` : genere commitment + nonces
   - `round2_sign()` : signe avec signing_package + share
   - `aggregate()` : combine K signatures en signature Ed25519
   Chaque step lit/ecrit un fichier JSON temporaire (air-gapped
   workflow).

3. **CLI endpoints** : etendre le daemon HTTP API pour exposer
   les operations DKG/ceremony en endpoints admin (trust tier T0) :
   - `POST /api/canary/frost/trusted-dealer` (params: k, n)
   - `POST /api/canary/frost/round1`
   - `POST /api/canary/frost/round2`
   - `POST /api/canary/frost/aggregate`

4. **Config** : `configs/canary.toml.sample` avec les chemins
   shares, pubkey package, K/N.

5. **WARRANT_CANARY_HARDENING.md** : update §4 ops runbook avec
   les commandes reelles (actuellement pseudo-commandes `sbfb
   canary frost ...` → remplir avec les endpoints HTTP ou CLI).

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon-core/src/canary/dkg.rs` | NEW : DKG trusted dealer wrapper |
| `crates/nexus-shell-daemon-core/src/canary/ceremony.rs` | NEW : signing ceremony rounds |
| `crates/nexus-shell-daemon-core/src/canary/mod.rs` | Re-export dkg + ceremony |
| `crates/nexus-shell-daemon-core/Cargo.toml` | dep frost-ed25519 (deja dans workspace) |
| `crates/nexus-shell-daemon/src/` ou `api/` | CLI/HTTP endpoints DKG |
| `configs/canary.toml.sample` | NEW : config ceremony |
| `docs/security/WARRANT_CANARY_HARDENING.md` | §4 ops runbook update |

### §6.3 Tests plan

1. `test_dkg_trusted_dealer_roundtrip` — genere K=2/N=3, verifie 3
   key packages + 1 pubkey package, serialise/deserialise JSON
2. `test_dkg_params_validation` — K > N rejete, K=0 rejete, N=0 rejete
3. `test_signing_ceremony_3_participants` — round1→round2→aggregate
   avec 3 participants (K=2, seuls 2 signent), verifie signature
   Ed25519 valide via pubkey package
4. `test_signing_ceremony_insufficient_signers` — seul 1 signer
   sur K=2 → erreur
5. `test_signing_ceremony_tamper_detect` — message altere apres
   signature → verification echoue
6. `test_canary_niveau1_compatible_niveau0` — la signature
   aggregee est byte-identique a une signature Ed25519 single-key
   (wire format inchange)

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(canary)'
cargo nextest run -p nexus-shell-daemon --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

### §6.5 Commit cible

```
feat(sprint30): Sprint 30 Phase C — warrant canary Niveau 1 FROST
DKG code wiring

## Scope

DKG trusted dealer wrapper frost-ed25519 2.1 generate_with_dealer
(K=2/N=3). Signing ceremony round1/round2/aggregate wrappers.
HTTP admin endpoints (T0). Config canary.toml.sample. Ops runbook
§4 WARRANT_CANARY_HARDENING.md.

Scope strict code wiring : pas de recrutement mainteneurs (ops
post-v1.0), pas de TEE attestation (NoopAttestation maintenu).

## Test delta cumulatif

+6 tests Rust (dkg_roundtrip, params_validation, ceremony_3p,
insufficient_signers, tamper_detect, niveau1_compatible_niveau0).
```

---

## §7 Phase D — G2 HARDENING refresh + split inference research

Dependencies : Phase A (HARDENING_ROADMAP deja touche Phase A pour
fix otel), idempotent avec Phase B/C

### §7.1 Scope

Phase docs-heavy, 2 livrables :

1. **HARDENING_ROADMAP.md refresh** (D4) :
   - `last_validated: 2026-04-26` avec commentaire G2 S30
   - §3 S30 : statut reel des 4 items (Nym re-defer, TEE scope-cut,
     warrant canary Phase C, split inference Phase D)
   - §3 S31 (nouvelle section) : "Tor transport phase 1 avec arti
     2.0 stable" + carries S30 resolus + items differes
   - `audited_findings` : entree 2026-04-26 documenter les 3
     triggers actifs (iroh 0.98, arti 2.0, openai-agents 0.14.6)
   - Timeline graph §6 : ajouter S30→S31 edges
   - Notes realisme : Nym re-defer rationale

2. **SPLIT_INFERENCE_DESIGN.md** (D5) :
   - §1 Contexte : pourquoi le split inference est pertinent pour
     SBFB (C-PromptLeak threat, compute distribue sans serveur
     central)
   - §2 Patterns existants : BOINC (verification hash deterministique
     — inapplicable LLM stochastique), Truebit (interactive
     verification game — applicable partiel), Golem (task markets +
     reputation — similaire SBFB kudos), split learning (model
     partitioning — applicable partiel)
   - §3 Threat model implications : partitionnement du modele expose
     des couches intermediaires, confidentialite des activations
   - §4 Recommendations : sprint dedie post-Gate 4, focus Truebit-
     style interactive verification adapte au compute LLM
   - §5 References (academiques + OSS)

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/security/HARDENING_ROADMAP.md` | G2 refresh complet |
| `docs/security/SPLIT_INFERENCE_DESIGN.md` | NEW : research doc |
| `docs/security/VALIDATED_BLUEPRINT.md` | Coherence check (probable no-op) |

### §7.3 Tests plan

Pas de tests — phase docs-only.

### §7.4 Critere d'acceptation

```bash
# Verification doc coherence
grep "last_validated" docs/security/HARDENING_ROADMAP.md
# → 2026-04-26
grep -c "S31" docs/security/HARDENING_ROADMAP.md
# → >= 1 (nouvelle section)
test -f docs/security/SPLIT_INFERENCE_DESIGN.md
# → true
```

### §7.5 Commit cible

```
docs(sprint30): Sprint 30 Phase D — G2 HARDENING refresh + split
inference research

## Scope

HARDENING_ROADMAP.md G2 refresh : last_validated S30, 3 triggers
actifs documentes (iroh 0.98, arti 2.0, openai-agents 0.14.6),
§3 S30 statut reel (Nym re-defer, TEE scope-cut), §3 S31 Tor
transport phase 1 (arti 2.0 stable). SPLIT_INFERENCE_DESIGN.md
research doc (BOINC/Truebit/Golem/split learning patterns,
threat model, recommendations).

## Test delta cumulatif

+0 tests (docs only). Regression verte full workspace.
```

---

## §8 Phase E — Wrap-up + verification + audit plan S31

Dependencies : Phases A-D toutes completees

### §8.1 Scope

Standard wrap-up :
- `sprint30_verification.md` : fail-fast 30+ rows, test counts,
  LOC surface
- `sprint30_carry_summary.md` : carry-overs S31 avec compteur
  reports
- `sprint31_audit_plan.md` : tracks pour audit S30 par S31 Phase 0
- `docs/claude/SPRINT_LOG.md` : row S30
- `CLAUDE.md §Etat actuel` : update
- Migration active/ → archive/v1.2/ (tous fichiers S30)
- Memory update `nexus_grid_pivot.md` + `MEMORY.md`

### §8.2 Commit cible

```
chore(sprint30): Phase E — wrap-up + verification + audit plan S31
+ migration
```

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile | `cargo build --workspace --locked` | 0 errors | |
| 2 | Rust tests | `cargo nextest run --workspace --locked` | 856+ passed | |
| 3 | Rust doctests | `cargo test --workspace --locked --doc` | 0 failures | |
| 4 | Rust clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 5 | Rust fmt | `cargo fmt --all --check` | 0 diffs | |
| 6 | Release build | `cargo build -p nexus-shell-daemon --release` | 0 errors | |
| 7 | Python ruff format | `uv run ruff format --check packages/` | 0 diffs | |
| 8 | Python ruff check | `uv run ruff check packages/` | 0 errors | |
| 9 | SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 195+ passed | |
| 10 | Coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | 393+ passed (36f PyO3 stale) | |
| 11 | Gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46+ passed | |
| 12 | Frontend lint | `cd web && npm run lint` | 0 errors | |
| 13 | Frontend tsc | `cd web && npx tsc --noEmit -p tsconfig.app.json` | 0 errors | |
| 14 | Vitest | `cd web && npm run test:unit` | 269+ passed | |
| 15 | Frontend build | `cd web && npm run build` | 0 errors | |
| 16 | size-limit | `cd web && npm run size` | 4/4 within budget | |
| 17 | Playwright | `cd web && npx playwright test` | 41+ passed | |
| 18 | en-strings | `cd web && bash scripts/scan-en-strings.sh` | 0 violations | |
| 19 | P2-AUDIT-1 fix | `grep "0\.27" docs/security/HARDENING_ROADMAP.md \| grep -i otel` | 0 matches | |
| 20 | P3-AUDIT-1 fix | `grep "0\.28" crates/nexus-trace-core/src/lib.rs` | 0 matches | |
| 21 | consent pure fn | `uv run pytest packages/nexus-coordinator/tests/test_consent.py -q` | all passed | |
| 22 | CI workflow | `test -f .github/workflows/ci-cross-platform.yml` | exists | |
| 23 | blob-serve COOP | `cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(blob_serve)'` | security headers tests pass | |
| 24 | DKG roundtrip | `cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(dkg)'` | all passed | |
| 25 | Ceremony test | `cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(ceremony)'` | all passed | |
| 26 | Canary compat | `cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(niveau1)'` | compatible Niveau 0 wire | |
| 27 | HARDENING G2 | `grep "last_validated.*2026-04-26" docs/security/HARDENING_ROADMAP.md` | found (S30) | |
| 28 | Split inference | `test -f docs/security/SPLIT_INFERENCE_DESIGN.md` | exists | |
| 29 | S31 Tor entry | `grep "S31.*Tor\|S31.*arti" docs/security/HARDENING_ROADMAP.md` | found | |
| 30 | FORMAT_VERSION | `grep -rE "_VERSION\s*[:=]\s*[0-9]+" crates/nexus-core-rs/src/ \| grep -v "= 1"` | 0 matches (all v1) | |
| 31 | commits | `git log --oneline HEAD...{tip_entree}` | 5 commits (A+B+C+D+E) | |
| 32 | planning docs | `ls .planning/active/sprint30_*.md` | kickoff + plan + verification + carry_summary | |

---

## §10 Git plan

| # | Commit | Scope |
|---|---|---|
| 0 | `chore(planning): sprint 30 kickoff + plan + design review` | Kickoff + plan + G1 design review + migration audit plan |
| 1 | G8 preflight Phase A | `sprint30_phase_A_preflight.md` |
| 2 | `feat(sprint30): Sprint 30 Phase A — P2 batch S29 audit` | 7 P2/P3 fixes |
| 3 | G8 preflight Phase B | `sprint30_phase_B_preflight.md` |
| 4 | `feat(sprint30): Sprint 30 Phase B — dette pair CI + blob-serve` | MANDATORY CI + COOP/COEP |
| 5 | G8 preflight Phase C | `sprint30_phase_C_preflight.md` |
| 6 | `feat(sprint30): Sprint 30 Phase C — warrant canary Niveau 1 DKG` | DKG + ceremony + config |
| 7 | G8 preflight Phase D | `sprint30_phase_D_preflight.md` |
| 8 | `docs(sprint30): Sprint 30 Phase D — G2 HARDENING + split inference` | Docs refresh |
| 9 | `chore(sprint30): Phase E — wrap-up + verification + audit plan S31` | Wrap-up + migration |

---

## §11 Scope cuts (copie kickoff §7)

1. Tor transport phase 1 → S31
2. Nym mixnet phase 1 → S32+
3. TEE H100 attestation → scope-cut (pas hardware)
4. DKG distribue FROST → post-v1.0
5. Recrutement mainteneurs → ops post-v1.0
6. iroh 0.98 upgrade → sprint dedie
7. openai-agents-python upgrade → pas de dep
8. task_runner implementation → S31
9. §9.5 output filter wire → S31
10. Full process isolation blob-serve → LT
11. Tor PoW spec update → trigger inactif
12. MCP spec revision → trigger inactif
13. CI full workspace cross-platform → scope CI = events-core

---

## §12 Risks (copie kickoff §9)

| ID | Risque | Mitigation |
|---|---|---|
| R1 | macOS runner GitHub Actions | Fallback ubuntu-only, macOS allow-failure |
| R2 | frost-ed25519 API change | Pin exact `=2.1` |
| R3 | COOP/COEP casse apps iframe | Test Playwright regression |
| R4 | Scope creep Niveau 1 | Code wiring only, 0 ops |
| R5 | Sprint trop charge | Phase D docs-only absorbe |
| R6 | arti defer conteste | Evidence dette + carries dans kickoff |
| R7 | LT-6 iroh trigger | Document awareness, Day 0 #3 pin |

---

## §13 Checkpoint de cloture

S30 est ferme quand :
- 32/32 fail-fast rows vertes
- 5 commits (A+B+C+D+E) + planning commits
- 3 docs planning ecrits (verification, carry_summary, audit_plan)
- SPRINT_LOG.md row S30 ajoute
- Memory mise a jour
- active/ migre vers archive/v1.2/
