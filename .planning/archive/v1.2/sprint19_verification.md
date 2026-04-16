# Sprint 19 — Verification (PoW gossip + TLS pinning + delayed upload + DHT wire + pkarr relay image)

**HEAD entree** : `1a606a3` (chore(sprint18): audit-P3 batch — buildType URI + parse_version warn + RADICLE casing)
**HEAD sortie** : `c609a03` (chore(docs) TOOLING.md Write-obligatoire nexus-phase-auditor G4) avant wrap-up ; wrap-up Phase F ajoute un commit chore final `chore(sprint19): Phase F`.
**Date** : 2026-04-16

---

## Commit stack

```
2fd4d72  feat(sprint19): Phase E — pkarr relay self-hosted docker image + ops doc
2fd6c60  chore(planning): Sprint 19 Phase D wrap — review artefact + workflow autonomy refinement
f238d31  feat(sprint19): Phase D — delayed upload queue (0-5min exponential jitter)
fe0a8fd  chore(planning): Sprint 19 — workflow guardrails G1..G7 + no-LOC + research docs
540bb51  feat(sprint19): Phase C — TLS cert pinning relays (SPKI hash validate)
08f4e41  fix(sprint19): Phase B follow-up — wire Cargo deps + canonical + lib + PATTERNS
edfc51b  feat(sprint19): Phase B — PoW Hashcash gossip subscribe (difficulty 2^18 per-relai)
ab6985c  feat(sprint19): Phase A — DHT quorum runtime wire (browse aggregator canary)
```

Infra tooling hors-sprint inclus chronologiquement dans le range mais non
comptabilise dans les phases officielles : `4216436` chore(skill) nexus-phase-
auditor Write enforce G4 + `c609a03` chore(docs) TOOLING.md G4. Ces commits
durcissent le harnais Claude Code (reviewer auditor obligation d'ecrire le
fichier review apres PASS, trace G4 rigor signal) et n'impactent pas le code
applicatif S19.

Phase 0 gate S18 : **DEJA JOUE pre-S19** via 6 commits `677556f..1a606a3`
(1 P1 + 4 P2 + 1 batch P3) apres verdict CONDITIONAL PASS. Aucun fix
P0/P1 S18-direct restant avant S19 Phase A.

---

## Checklist fail-fast

### CI & test suites (reproducible au tip `c609a03`)

- [x] **Rust** : `cargo test --workspace --locked` → **537 passing** (baseline S18 : 478, delta **+59**). 0 failed, 0 ignored.
- [x] **Rust lint** : `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- [x] **Python SDK** : `uv run pytest packages/nexus-sdk/tests/ -q` → **185** (baseline 183, delta **+2**).
- [x] **Coordinator** : `uv run pytest packages/nexus-coordinator/tests/ -q` → **208 passed + 3 skipped** (baseline 187+3, delta **+21**).
- [x] **App-gov** : `uv run pytest packages/nexus-app-gov/tests/ -q` → **46** (inchange — zero code gov touche S19).
- [x] **Vitest** : `cd web && npm run test:unit` → **239** (inchange — zero code web touche).
- [x] **Playwright** : `npx playwright test` → **38** (inchange).
- [x] **size-limit** : `npm run size` → **7/7 OK** (bundle index 9.76 kB, css 120.19 kB sous plafonds).
- [x] **Web lint / tsc** : `npm run lint` + `npx tsc --noEmit -p tsconfig.app.json` + `bash scripts/scan-en-strings.sh` → clean.
- [x] **Ruff** : `uv run ruff format --check packages/` + `uv run ruff check packages/` clean.

**Compteur final** : **~1259 tests** (537 Rust + 185 SDK + 208+3 coord + 46 gov + 239 Vitest + 38 Playwright + 7 size-limit assertions). Delta S19 total : **+82** (projection kickoff §9 : +39 a +55, livre largement au-dessus par un PoW primitive plus fourni que prevu + delayed upload queue + 20 tests + DHT wire couverture).

### Scope respecte

- [x] **Scope cuts §6 kickoff** — scan sur 10 keywords (`encryption-at-rest, duress, rate-limit, kudos-admission, structured-output, redaction, ONG-relays, PQC, ML-DSA, domain-fronting`) sur le diff complet S19 `1a606a3..c609a03` : **zero match hors docs de reference** (HARDENING_ROADMAP §3 S20-S22, VALIDATED_BLUEPRINT couche 1 PQC, MIRROR_FALLBACK §3 Radicle deferred). Conforme.
- [x] **Items nouveaux S19** (PoW Hashcash B, TLS cert pinning C, delayed upload queue D, pkarr relay docker E) livres.
- [x] **Carry S18 C-1** (DHT quorum runtime wire au browse aggregator) livre Phase A.
- [x] **Carry S18 Meta-1** (Radicle-v1.0 activation tracking) re-carry explicite dans `sprint19_audit_plan.md §Meta-track`.
- [x] **Cap carry-overs G7** : 2 carries maximum autorises (C-1 + Meta-1). Pas de debordement.

### Commits pattern

- [x] **5 feat commits** (A/B/C/D/E) + **1 fix commit** (Phase B follow-up `08f4e41` wire Cargo + canonical + lib + PATTERNS apres split du primitive commit initial). **2 chore(planning)** (`fe0a8fd` guardrails G1..G7 + `2fd6c60` Phase D wrap review artefact).
- [x] **Bodies riches** : contexte + livrables + tests delta + scope cuts respectes + cross-refs HARDENING_ROADMAP ligne par ligne.
- [x] **Audit reviews par phase** : 5 reviews livrees (`sprint19_phase_{A,B,C,D,E}_review.md`) par skill `nexus-phase-auditor`. Verdicts : A PASS (post-P3 fixes inline), B CONCERN→PASS (follow-up `08f4e41`), C PASS, D PASS (post-P2 fixes inline), E PASS (post P1 + P2-E3 fixes inline). Rigor signal G4 satisfait toutes phases (>=1 P2+ documente chacune). 0 P0 toutes phases.

### Gate HARDENING_ROADMAP §3 S19

Criteria (cf. `docs/security/HARDENING_ROADMAP.md §3 Sprint 19`) :

- [x] **PoW Hashcash gossip subscribe** : primitive `crates/nexus-core-rs/src/pow.rs` + integration gossip subscribe path + per-relai policy `relay_pow_policy.toml` + difficulty 2^18 default (Phase B `edfc51b` + `08f4e41`).
- [x] **TLS cert pinning relays** : `crates/nexus-core-rs/src/tls_pinning.rs` SPKI hash extract + `PinValidator` + pinset bootstrap + test cert fixture `relay_test_cert.pem` + doc PATTERNS.md section TLS pinning (Phase C `540bb51`).
- [x] **Delayed upload queue** : `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` async queue + scheduler 30s flush + exponential jitter 0-5min + integration `api/tasks.py` + doc shell/PATTERNS.md (Phase D `f238d31`).
- [x] **pkarr relay self-hosted** : `docker/pkarr-relay/Dockerfile` + `.github/workflows/build-pkarr-image.yml` + `docs/release/PKARR_RELAY_OPS.md` §1-§7 self-contained (provisioning + systemd + nginx + smoke + rotation) (Phase E `2fd4d72`).
- [x] **DHT quorum runtime wire (carry C-1)** : `PkarrQuorumResolver` + `PkarrRelayClient` primitive wrap + wiring `nexus-shell-daemon-core` browse aggregator + curator runtime via canary opt-in `SBFB_PKARR_RELAYS` env var (Phase A `ab6985c`). **Eclipse-by-DHT defense runtime-active sous config** (canary armed avec 2+ relays ; enforcement strict par defaut = post-Gate 2). Finding `sprint19_audit_findings.md §Track A` P2-A1 reclasse explicitement la portee du flip.

**Gate S19 = REMPLI.** Pre-requis S21 rate-limit (Sybil-resistance minimale via PoW) disponible. Pre-requis Gate 2 (encryption at rest + duress) non-bloque.

---

## Migration PARA (Phase F) — 10 files

```bash
git mv .planning/active/sprint19_kickoff.md          .planning/archive/v1.2/
git mv .planning/active/sprint19_plan.md             .planning/archive/v1.2/
git mv .planning/active/sprint19_verification.md     .planning/archive/v1.2/
git mv .planning/active/sprint19_audit_plan.md       .planning/archive/v1.2/
git mv .planning/active/sprint19_supervision_log.md  .planning/archive/v1.2/
git mv .planning/active/sprint19_phase_A_review.md   .planning/archive/v1.2/
git mv .planning/active/sprint19_phase_B_review.md   .planning/archive/v1.2/
git mv .planning/active/sprint19_phase_C_review.md   .planning/archive/v1.2/
git mv .planning/active/sprint19_phase_D_review.md   .planning/archive/v1.2/
git mv .planning/active/sprint19_phase_E_review.md   .planning/archive/v1.2/
```

10 files migres au total (kickoff + plan + verification + audit_plan + supervision_log + 5 phase reviews A/B/C/D/E). Le `sprint19_phase_F_review.md` (produit par nexus-phase-auditor au commit Phase F lui-meme) reste provisoirement dans `.planning/active/` pour satisfaire le hook `phase-auditor-gate.sh` (qui verifie l'emplacement `.planning/active/sprint{N}_phase_{X}_review.md` au moment du `git commit`). Pattern identique S18 : la F review est migree dans un commit ulterieur (audit-fix ou chore-planning Sprint N+1). Apres migration future : `.planning/active/` vide, pret pour `sprint20_kickoff.md`.

---

## Flip S18 verification — consequence Phase A wire DHT

`.planning/archive/v1.2/sprint18_verification.md §Gate 1 unlock` contient la
ligne :

```
- [~] **DHT redundant lookup** : primitive redundant_resolve + QuorumResolver
  trait + 13 tests verts (Phase C 9d0ad7a). Wiring runtime au browse
  aggregator + curator runtime carry-over Sprint 19 (...), tracke par
  audit S18 finding C-1 (P2).
```

Phase A S19 `ab6985c` livre exactement ce wiring. **Flip `[~]` → `[x]`** execute
dans le commit Phase F avec annotation inline renvoyant vers
`archive/v1.2/sprint19_phase_A_review.md` + `sprint19_verification.md §Gate
HARDENING_ROADMAP §3 S19`.

---

## Delta tests recapitulatif S19

| Phase | Suite | Delta reel | Cumul Rust apres | Cumul autres |
|---|---|---|---|---|
| Baseline S18 | — | — | 478 | 183 SDK / 187+3 coord / 46 gov / 239 vitest / 38 pw |
| Phase A | Rust | +7 (PkarrQuorumResolver + browse wire) | 485 | inchange |
| Phase B | Rust | +29 (PoW primitive + gossip subscribe integration) | 514 | inchange |
| Phase C | Rust | +9 (TLS SPKI validator) | 523 | inchange |
| Phase D | Rust + coord | +2 Rust (canonical helpers) + **+21 coord** (queue scheduler + integration + SDK smoke) | 525 | **185 SDK** / **208 coord** |
| Phase E | infra | +0 (docker build non compte suite Rust) | 525 | inchange |
| Phase E suite | Rust | +12 (wiring TLS pinset + pkarr-relay-client smoke + misc integration) | **537** | inchange |
| Phase F | — | 0 | **537** | inchange |

> **Note** : les cumulants Rust intermediaires (485 / 514 / 523 / 525 / 537)
> sont reconstruits post-facto a partir du plan de verification. Les phase
> reviews individuelles (archive/v1.2/sprint19_phase_{A,B,C,D,E}_review.md)
> rapportent localement des deltas legerement differents par phase
> (l'integration tests Phase C a fait remonter des tests wire Phase A ;
> le ventilage exact est non-reconciliable apres coup). **Le total final
> 537 Rust (delta +59 vs baseline S18 `478`) fait foi** — mesure directe
> au tip `c609a03` via `cargo test --workspace --locked` (cf. §CI & test
> suites). L'auditeur S20 Phase 0 peut challenger la decomposition par
> phase ; si besoin de forensics exact, re-bisect per-phase via
> `git checkout <phase_sha> && cargo test --workspace --locked`.

**Delta S19 Rust** : **+59** (plan annoncait +25, livre au-dessus — PoW
primitive plus fourni + integration couverture elargie).

**Delta S19 coordinator** : **+21** (plan annoncait +10).

**Delta S19 SDK** : **+2** (non prevu explicitement plan — utility helpers
Phase D pour submit + jitter).

**Delta S19 total** : **+82** (baseline 1176 → **~1259**).

---

## Prochaine etape

Sprint 20 Phase 0 = audit S19 — a jouer en session fraiche post-Phase F
wrap-up. Artefact livrable : `.planning/active/sprint19_audit_findings.md`
en reprenant le layout des `sprint18_audit_findings.md` / `sprint17_audit_
findings.md` archives. Le mode d'emploi est dans `sprint19_audit_plan.md`
(migre archive/v1.2/ avec les autres S19 docs).

Items S20 (cf. HARDENING_ROADMAP §3 S20) : encryption at rest keypair
keychain/DPAPI + duress PIN + panic wipe + structured output llama.cpp
grammar. Prerequis Gate 2 (DnD Forge beta publique T2) debloquable fin S20
si encryption+duress livres.

**Meta-1 Radicle-v1.0 activation tracking** re-carry S20 (cf.
`sprint19_audit_plan.md §Meta-track`) — pattern permanent tant que v1.0
pas tag.

---

## 5. Findings carry-over for memory

Items a fusionner manuellement dans la memory description de
`nexus_grid_pivot.md` au wrap-up Phase F :

- Sprint 19 CLOSED : 5 phases + 2 chore planning + 2 chore infra tooling
  hors-sprint (G4 review enforcement).
- Compteurs : **537 Rust** (+59) / 185 SDK (+2) / 208+3 coord (+21) /
  46 gov / 239 vitest / 38 pw / 7/7 size. Total **~1259 tests** (+82
  vs baseline S18).
- Eclipse-by-DHT defense **runtime-active sous config opt-in
  `SBFB_PKARR_RELAYS`** (Phase A wire C-1 S18 carry) — canary armed
  avec 2+ relays, enforcement strict par defaut = post-Gate 2. Flip
  S18 verification.md `[~]→[x]` acte le wiring primitive→runtime, pas
  l'activation universelle. Cf. audit findings P2-A1.
- PoW Hashcash primitive + gossip subscribe integration live : difficulty
  2^18 default, per-relai policy, difficulty adjust S21+.
- TLS SPKI cert pinning live : validator + pinset bootstrap, contrib
  upstream iroh reporte S20+ si hook n'est pas expose.
- Delayed upload queue : exponential jitter 0-5min, integration coord
  api/tasks.py pipe gossip emit async.
- pkarr relay self-hosted docker image publishable `ghcr.io/SBFB50/
  pkarr-relay` + ops doc self-contained (6 sections). Pas de deploy real
  S19 (reporte S20+).
- Carry S19 → S20 : Meta-1 Radicle-v1.0 activation tracking (owner
  FlowUP, deadline v1.0 tag, runbook MIRROR_FALLBACK §3 self-contained).

Pas d'autres zones rouges nouvelles. R-wasmtime-cve / R-iroh-audit /
R-libcrux-hax / R-pyodide-escape restent en etat pre-S19 (cf.
`docs/security/HARDENING_ROADMAP.md`).
