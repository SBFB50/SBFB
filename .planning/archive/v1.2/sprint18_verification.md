# Sprint 18 — Verification (quick wins + supply chain + multi-relai)

**HEAD entree** : `4f0727b` (Phase 0 S17 audit-P1 docs-only)
**HEAD sortie** : `4453bfd` (Sprint 18 wrap-up commit, audit S18 reopened post-cloture pour fix D-1 → tip courant `677556f`)
**Date** : 2026-04-15

---

## Commit stack

```
4453bfd  chore(sprint18): Phase F — wrap-up + verification + audit plan S19 + migrate planning
95807b1  feat(sprint18): Phase E3 — Codeberg private disaster-recovery mirror
04c9621  feat(sprint18): Sprint 18 Phase E2 — warrant canary monthly Ed25519 gossip publish
9f4d19f  feat(sprint18): Phase E1 — NVIDIA driver CVE check at launcher startup
94cccb2  feat(sprint18): Phase D — coord-side TaskEntry wire-through + X-SBFB-Token rotation
9d0ad7a  feat(sprint18): Phase C — multi-relai federation + DHT quorum primitive
4ab0211  feat(sprint18): Phase B — reproducible builds + SLSA in-toto attestation
d7ab281  feat(sprint18): Phase A — supply chain CI (cargo-deny + pip-audit + npm audit + wasmtime pin)
1f5cf42  chore(planning): close S17 + open Sprint 18 — quick wins + supply chain baseline + multi-relai phase 1
4f0727b  fix(sprint17)/docs: Phase 0 audit S17 findings (docs-only P1)
```

[Audit gate fix commits, landed post-wrap-up dans le cycle Sprint 19 phase 0]

```
677556f  fix(sprint18): audit-P1 D-1 — wire TokenRotator into shell-daemon HTTP router
```

Entre `d7ab281` (Phase A) et `4ab0211` (Phase B), une dizaine de commits tooling `chore(claude)` parsemeent le range (SessionStart autoinstall, Semgrep architectural rules, TDD Guard opt-in, phase-auditor hook, statusline enriched, post-commit memory updater, nexus-phase-review skill, TOOLING.md). Ces commits tunent l'environnement Claude Code et ne font pas partie des phases S18 officielles — inclus dans le range `4f0727b..HEAD` par proximite chronologique, scope hors sprint.

Phase 0 gate S17 : **DEJA JOUE pre-S18** via `4f0727b` (verdict PASS apres 1 commit docs-only). Aucun fix P0/P1 S17-direct requis avant S18.

---

## Checklist fail-fast

### CI & test suites

- [x] **Rust** : `cargo test --workspace --locked` → **474 passing** (baseline S17 : 430, delta +44). 0 failed, 0 ignored.
- [x] **Rust lint** : `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- [ ] **Python SDK** : `uv run pytest packages/nexus-sdk/tests/ -q` → verif Phase F (~183 inchange attendu — Sprint 18 = 0 code SDK touché).
- [ ] **Coordinator** : `uv run pytest packages/nexus-coordinator/tests/ -q` → verif Phase F (187+3 skipped inchange attendu).
- [ ] **App-gov** : `uv run pytest packages/nexus-app-gov/tests/ -q` → 46 (inchange attendu).
- [ ] **Vitest** : `cd web && npm run test:unit` → 239 (inchange — 0 code web touché).
- [ ] **Playwright** : `npx playwright test` → 38 (inchange).
- [ ] **size-limit** : 7/7 OK (inchange — pas de changement bundle).
- [ ] **SPDX** : `grep -rL "SPDX" crates/ | wc -l` → 0 (Phase E3 ajoute SPDX sur `mirror-codeberg.yml`).

**Compteur final estime** : **~1172 tests** (1128 baseline S17 + 44 S18 Rust).

### Scope respecte

- [x] **Scope cuts §6 kickoff** — scan sur 10 keywords (`iroh-audit, pyodide-escape, PoW-gossip, encryption-at-rest, tls-pinning, pkarr-relay, ONG-relays, PQC, ML-DSA, ML-KEM`) sur le diff complet S18 : **zero match hors docs de reference (VALIDATED_BLUEPRINT, HARDENING_ROADMAP, MIRROR_FALLBACK §Radicle differe)**. Conforme.
- [x] **Items carry S16** (TaskEntry wire + token rotation) livres Phase D.
- [x] **Items nouveaux S18** (cargo-deny/pip-audit/npm audit A, repro builds B, multi-relai C, NVD driver E1, canary E2, Codeberg mirror E3) livres.
- [x] **Pivot Phase E3** (Radicle → Codeberg + Radicle differe v1.0) documente dans plan §Phase E3 block dedie + kickoff 8 occurrences mises a jour.

### Commits pattern

- [x] **8 commits feat/chore(sprint18)** alignes pattern `<type>(sprint18): Phase X — titre`.
- [x] **Bodies riches** : contexte + livrables + tests delta + scope cuts respectes.
- [x] **Audit reviews** : `sprint18_phase_B_review.md` (PASS), `C_review.md` (PASS), `D_review.md` (PASS), `E1_review.md` (PASS — produit par nexus-phase-auditor au commit `9f4d19f`), `E2_review.md` (PASS 0 P0/P1), `E3_review.md` (PASS 0 P0/P1, 4 P2 fixes inline sauf P2-2 tracking Radicle-v1.0 reporte ici). **6 reviews au total** (correction post-audit S18 finding F-1 — la version d'origine de cette ligne disait `E1_review.md (non-present)` par erreur, le fichier existe et a ete migre dans archive avec les 5 autres).

### Gate 1 unlock

Criteria (cf. `docs/security/HARDENING_ROADMAP.md §7` Gate 1 — DnD Forge beta fermee) :

- [x] **Supply chain CI baseline** : cargo-deny + pip-audit + npm audit + wasmtime pin (Phase A)
- [x] **Reproducible builds** : `--locked` + `SOURCE_DATE_EPOCH` + SHA256 SLSA in-toto (Phase B)
- [x] **Multi-relai federation phase 1** : n0 + 2 fallbacks + round-robin (Phase C)
- [x] **DHT redundant lookup** : primitive `redundant_resolve` + `QuorumResolver` trait + 13 tests verts (Phase C `9d0ad7a`). **Wiring runtime** livre Sprint 19 Phase A `ab6985c` (carry S18 C-1 resolu) — `PkarrQuorumResolver` + `PkarrRelayClient` wrap cables au browse aggregator + curator runtime. Eclipse-by-DHT defense desormais **pleinement active en runtime**. Cf. `.planning/archive/v1.2/sprint19_phase_A_review.md` + `sprint19_verification.md §Gate HARDENING_ROADMAP §3 S19`.
- [x] **Coord-side wire complete** : TaskEntry craft + estimate caps (Phase D `94cccb2`) + token rotation **wirée au router HTTP daemon** via `notify` watcher (audit S18 fix D-1 `677556f` — la livraison D shippait la primitive cote launcher seule, le wiring effectif au middleware `auth_required` etait carry-over admis dans le commit body D, ferme par le post-wrap-up fix D-1).
- [x] **Driver update check** : NVIDIA CVE NVD scrape au launcher startup (Phase E1)
- [x] **Warrant canary** : Ed25519 signed monthly gossip + CANARY.txt + verify-canary.sh (Phase E2)
- [x] **Code mirror redundancy** : Codeberg push-mirror prive (Phase E3, Radicle differe v1.0 post-go-public)

**Gate 1 = UNLOCKED.** DnD Forge beta fermee deployable au tag v1.0 (apres flip public repos + activation Radicle).

---

## Migration PARA (Phase F) — 10 files

```bash
git mv .planning/active/sprint18_kickoff.md .planning/archive/v1.2/
git mv .planning/active/sprint18_plan.md .planning/archive/v1.2/
git mv .planning/active/sprint18_verification.md .planning/archive/v1.2/
git mv .planning/active/sprint18_audit_plan.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_B_review.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_C_review.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_D_review.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_E1_review.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_E2_review.md .planning/archive/v1.2/
git mv .planning/active/sprint18_phase_E3_review.md .planning/archive/v1.2/
```

Apres migration : `.planning/active/` vide, pret pour `sprint19_kickoff.md`.

Note : 10 files migres (incluant `sprint18_phase_E1_review.md` produit par nexus-phase-auditor au commit `9f4d19f`).

---

## Delta tests recapitulatif S18

| Phase | Suite | Delta reel | Cumul apres |
|---|---|---|---|
| Baseline S17 | Rust | — | 430 |
| Phase A | Rust | 0 (ops CI pur) | 430 |
| Phase B | Rust | +5 (attestation verify) | 435 |
| Phase C | Rust | +20 (multi-relai + DHT quorum) | 455 |
| Phase D | Rust | +8 (TaskEntry + token rotation) | 463 |
| Phase E1 | Rust | +6 (NVD CVE check : exact match, cache hit, cache miss TTL, offline fallback, severity filter, range bounds) | 469 |
| Phase E2 | Rust | +5 (canary primitive : build, verify ok/tampered/wrong-pubkey, publish mock — la valeur `+10` initialement reportee agregeait E1+E2 par erreur, corrige post-audit S18 finding F-1) | 474 |
| Phase E3 | Rust | 0 (ops CI pur) | **474** |
| Phase F | Rust | 0 | **474** |

**Delta S18 Rust** : **+44** (plan annoncait +50-60, livre legerement en dessous mais conforme — la repartition par phase a ete realignee post-audit S18 finding F-1, le total cumulatif reste exact).

[Post-wrap-up audit gate] : +4 tests Rust ajoutes par `677556f` (fix D-1 wire TokenRotator + watcher tokens.json), total **478**. Detail dans `.planning/active/sprint18_audit_findings.md` §Track D + commit body D-1.

Autres suites : inchangees (0 code Python/Web/Vitest/Playwright modifie S18).

---

## Prochaine etape

Sprint 19 Phase 0 = audit S18 — **JOUE** (cf. `sprint18_audit_plan.md` livre meme commit). Session fraiche 2026-04-15 a lu le range `4f0727b..4453bfd` + les 6 audit reviews phase-par-phase + produit `.planning/active/sprint18_audit_findings.md` avec verdict CONDITIONAL PASS (0 P0, 1 P1, 5 P2, 6 P3). Le P1 D-1 (token rotation primitive non-cablee au router HTTP) est leve via `677556f` `fix(sprint18): audit-P1 D-1 — wire TokenRotator into shell-daemon HTTP router`. Sprint 19 phase suivante (kickoff + Phase A) peut demarrer apres traitement des P2 docs hygiene + wire DHT primitive (commits `fix(sprint18): audit-P2 ...`).

Items S19 (cf. HARDENING_ROADMAP §3 S19) : PoW gossip + TLS cert pinning relays + self-hosted pkarr relay + activation Radicle mirror si repo GitHub passe public au tag v1.0 avant S19 close.
