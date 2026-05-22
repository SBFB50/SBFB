# Sprint 68 — Verification (Proof Cards + Publish Gate)

**Ecrit** : 2026-05-22.
**HEAD** : `ecb25c5` (Phase D derniere phase code).
**Compteurs sortie** : 1419 Rust / 279 Vitest / 6/6 size-limit.

---

## S1 Fail-fast checklist

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | PASS |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1407 | PASS (1419/1419) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | PASS |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok | PASS |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok | PASS |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors | PASS (5 warnings react-refresh T1 known) |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | PASS |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 275 | PASS (279/279) |
| 10 | npm build | `(cd web && npm run build)` | ok | PASS |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 | PASS |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | PASS |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | PASS |
| 14 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | PASS |
| 15 | proof_card unit | `cargo nextest run -p nexus-coordinator-rs -E 'test(proof_card)' --locked` | >= 6 | PASS (8/8) |
| 16 | proof_card endpoint | `cargo nextest run -p nexus-shell-daemon -E 'test(proof_card)' --locked` | >= 1 | PASS (2/2) |
| 17 | preview tests | `cargo nextest run -p nexus-shell-daemon -E 'test(preview)' --locked` | >= 4 | PASS (4/4) |
| 18 | publish tests | `cargo nextest run -p sbfb-factory -E 'test(publish)' --locked` | >= 2 | PASS (2/2) |
| 19 | fg5 sandbox | `cargo nextest run -p sbfb-factory -E 'test(fg5)' --locked` | >= 4 | PASS (4/4) |
| 20 | fg6 lockfile | `cargo nextest run -p sbfb-factory -E 'test(fg6)' --locked` | >= 2 | PASS (3/3) |
| 21 | diff tests | `cargo nextest run -p sbfb-factory -E 'test(diff)' --locked` | >= 2 | PASS (3/3) |
| 22 | ProofCard UI Vitest | `(cd web && npm run test:unit)` includes ProofCard | >= 4 | PASS (8 tests ProofCard inclus dans 279) |
| 23 | proof_card_get allowlist | `grep -q "proof_card_get" crates/sbfb-manifest/src/lib.rs` | present | PASS |
| 24 | THREAT_MODEL ProofCard | `grep -q "T-PROOFCARD" docs/security/THREAT_MODEL.md` | present | PASS |
| 25 | factory subcommands | `cargo run -p sbfb-factory -- --help` | preview, publish, diff, scan-secrets | PASS |
| 26 | factory no daemon dep | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent | PASS |
| 27 | dunce in factory deps | `grep -q "dunce" crates/sbfb-factory/Cargo.toml` | present | PASS |
| 28 | verification.md | `test -f .planning/active/sprint68_verification.md` | exists | PASS (ce fichier) |
| 29 | audit_plan S69 | `test -f .planning/active/sprint69_audit_plan.md` | exists | PASS |
| 30 | preflight Phase E | `test -f .planning/active/sprint68_phase_e_preflight.md` | exists | PASS |

**30/30 PASS.**

---

## S2 Delta tests cumule Sprint 68

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +11 | +1 | proof_card 8 + endpoint 2 + bridge 1 Vitest (1384→1395, 270→271) |
| B | +14 | +0 | preview 4 + publish 2 + preview_cmd 3 + preview_eviction 1 + preview_max_size 1 + http wiring 3 (1395→1409) |
| C | +10 | +0 | fg5 4 + fg6 3 + diff 3 (1409→1419) |
| D | +0 | +8 | ProofCard UI 8 Vitest (271→279) |
| E | +0 | +0 | documentation seulement |
| **Total** | **+35** | **+9** | **1384→1419 Rust, 270→279 Vitest** |

Plan estimait +23 Rust / +5 Vitest. Reel +35 Rust / +9 Vitest (+12 Rust au-dessus de l'estimation, +4 Vitest au-dessus). Couverture plus large que prevue grace a des tests additionnels preview/gates/endpoint.

---

## S3 Commits Phase

| Phase | SHA | Titre |
|---|---|---|
| A | `f9d722e` | `feat(proof-card): Sprint 68 Phase A — ProofCard computation + daemon endpoint` |
| B | `1d53f18` | `feat(factory): Sprint 68 Phase B — preview ephemere + publish path` |
| C | `a201b3e` | `feat(factory): Sprint 68 Phase C — Factory gates FG4-FG7 + path traversal fix` |
| D | `ecb25c5` | `feat(shell): Sprint 68 Phase D — Proof Card UI + Browse integration` |
| E | (ce commit) | `docs(sprint68): Sprint 68 Phase E — verification + wrap-up` |

---

## S4 Scope cuts respectes

| # | Item | Sprint cible | Respecte |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S70 | oui |
| 2 | Page React /factory | S69+ | oui |
| 3 | Babel dogfood via Factory | S69 | oui |
| 4 | @dev index tree-sitter | S70+ | oui |
| 5 | Template react-vite | S69+ | oui |
| 6 | Factory audit log JSONL | S69+ | oui |
| 7 | CuratorVouched UI shell | S70+ | oui |
| 8 | FG8 Provenance Ed25519 | S69 | oui |
| 9 | FG9 Publish gate complete | S69 | oui |
| 10 | FG10 Review gate | S69 | oui |
| 11 | Fuzzing cargo-fuzz/proptest | post-audit | oui |
| 12 | Feed format version bump | post-launch | oui |
| 13 | ProofCard comme feed op | S70+ | oui |
| 14 | Diff engine avance | S69+ | oui |

**14/14 scope cuts respectes.**

---

## S5 Carries residuels S69

| Item | Reports | Status | Justification |
|---|---|---|---|
| P2-A-1 rand blocker | exemption externe | reconduit | upstream rand 0.9 non publie, dep transitive iroh 0.98 |
| P2-AUDIT-2 iroh transitives | exemption externe | reconduit | herite pin iroh 0.98, evaluer Gate 1 |
| P2-G-1 exe lock intermittent | monitoring | reconduit | non reproductible depuis S62 (6 sprints) |
| T-NN+2 iframe Rust-wasm | bloque upstream | reconduit | toolchain gaps inchanges |
| P2-I-2 delta body | 2/3 | reconduit | attention 3/3 S69 — Phase A body disait +10 mais retrospectivement +11 (corrige Phase B). Pattern a surveiller. |
| LT-2 Radicle sortie | trigger PENDING | reconduit | tag v1.0 pose localement, pas pousse origin |
| LT-5 redundancy persistence | hors-sprint | reconduit | reclassifie S26 |
| LT-7 worker quorum E2E | post-tag | reconduit | quorum E2E carry post-tag |

P2-C-2 path traversal Windows : **RESOLVED S68 Phase C** (dunce::canonicalize + prefix check + 4 tests fg5).

Aucun item n'atteint 3/3 MANDATORY au S69.

---

## S6 Checkpoint de cloture

- [x] 30/30 fail-fast verts
- [x] 4 commits feat (Phase A, B, C, D) + 1 commit docs (Phase E)
- [x] verification.md + audit_plan S69 ecrits
- [x] THREAT_MODEL.md enrichi (§12 T-PROOFCARD-FORMULA-GAME, 4 vecteurs + mitigations)
- [x] CLAUDE.md + SPRINT_LOG.md a jour
- [x] ProofCard struct + formule score 0-100 + 7 risk factors + formula_version 1
- [x] Preview ephemere POST /api/v1/preview/load + TTL eviction + blob-serve integration
- [x] Publish path sbfb-factory publish → daemon deploy-from-repo
- [x] Gates FG4-FG7 (diff + sandbox dunce + lockfile + secrets + preview check)
- [x] ProofCard.tsx composant UI (expandable card, 6 couches preuve, badge risque)
- [x] BrowsedProject.tsx integration useQuery proof-card
- [x] Factory ne depend PAS de nexus-shell-daemon
- [x] Memory nexus_grid_pivot.md tip + compteurs mis a jour
