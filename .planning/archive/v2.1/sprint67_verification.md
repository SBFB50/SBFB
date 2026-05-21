# Sprint 67 — Verification (Factory Foundation)

**Ecrit** : 2026-05-21.
**HEAD** : `c2af337` (Phase D derniere phase code).
**Compteurs sortie** : 1384 Rust / 270 Vitest / 6/6 size-limit.

---

## S1 Fail-fast checklist

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | PASS |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1381 | PASS (1384/1384) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | PASS |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok | PASS |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok | PASS |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors | PASS (5 warnings react-refresh T1 known) |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | PASS |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 270 | PASS (270/270) |
| 10 | npm build | `(cd web && npm run build)` | ok | PASS |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 | PASS |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | PASS |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | PASS |
| 14 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | PASS |
| 15 | sbfb-manifest tests | `cargo nextest run -p sbfb-manifest --locked` | 4+ pass | PASS (4/4) |
| 16 | sbfb-factory tests | `cargo nextest run -p sbfb-factory --locked` | 13+ pass | PASS (16/16) |
| 17 | search tests | `cargo nextest run -p nexus-coordinator-rs -E 'test(search)' --locked` | 6+ pass | PASS (7/7) |
| 18 | curator_vouched tests | `cargo nextest run -p nexus-coordinator-rs -E 'test(curator_vouched)' --locked` | 3+ pass | PASS (3/3) |
| 19 | feed_entries endpoint | `cargo nextest run -p nexus-shell-daemon -E 'test(feed_entries)' --locked` | 2+ pass | PASS (2/2) |
| 20 | deploy no node_id | `cargo nextest run -p nexus-shell-daemon -E 'test(deploy_from_repo)' --locked` | includes new | PASS (4/4) |
| 21 | search endpoint HTTP | `cargo nextest run -p nexus-shell-daemon -E 'test(search)' --locked` | 1+ pass | PASS (1/1) |
| 22 | bridge search Vitest | `(cd web && npm run test:unit)` includes search | pass | PASS (inclus dans 270) |
| 23 | SBFB.json v2 examples | `grep -q '"schema_version": 2' examples/sbfb-explorer/SBFB.json` | present | PASS |
| 24 | THREAT_MODEL search | `grep -q "T-SEARCH" docs/security/THREAT_MODEL.md` | present | PASS |
| 25 | THREAT_MODEL curator | `grep -q "T-CURATOR-VOUCH" docs/security/THREAT_MODEL.md` | present | PASS |
| 26 | PATTERNS P52 | `grep -q "P52" docs/rust/PATTERNS.md` | present | PASS |
| 27 | factory no daemon dep | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent | PASS |
| 28 | verification.md | `test -f .planning/active/sprint67_verification.md` | exists | PASS (ce fichier) |
| 29 | audit_plan S68 | `test -f .planning/active/sprint68_audit_plan.md` | exists | PASS |

**29/29 PASS.**

---

## S2 Delta tests cumule Sprint 67

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +11 | +0 | sbfb-manifest 4 + deploy 2 + curator_vouched 3 + feed_entries 2 (1349→1360) |
| B | +8 | +1 | search coordinator 7 + search http 1 + bridge search Vitest 1 (1360→1368, 269→270) |
| C | +11 | +0 | template_engine 8 + secret_scanner 3 (1368→1379) |
| D | +5 | +0 | provenance 3 + EXCLUDED_FILES 1 + P0-D-1 fix 1 (1379→1384) |
| E | +0 | +0 | documentation seulement |
| **Total** | **+35** | **+1** | **1349→1384 Rust, 269→270 Vitest** |

Plan estimait +32 Rust / +1 Vitest. Reel +35 Rust / +1 Vitest (+3 au-dessus de l'estimation, dont +1 test PEM post-review Phase C et +2 tests additionnels).

---

## S3 Commits Phase

| Phase | SHA | Titre |
|---|---|---|
| A | `4ee93ab` | `feat(daemon): Sprint 67 Phase A — sbfb-manifest + feed primitives + SBFB.json v2` |
| B | `f46bc66` | `feat(search): Sprint 67 Phase B — FTS5 search @protocole + THREAT_MODEL feed 3/3` |
| C | `49d6bcd` | `feat(factory): Sprint 67 Phase C — sbfb-factory CLI + template engine + create + validate` |
| D | `a4cc0ae` | `feat(factory): Sprint 67 Phase D — factory provenance + P52 BlobStore pattern + dette` |
| E | (ce commit) | `docs(sprint67): Sprint 67 Phase E — verification + wrap-up` |

---

## S4 Scope cuts respectes

| # | Item | Sprint cible | Respecte |
|---|---|---|---|
| 1 | Preview ephemere | S68 | oui |
| 2 | Diff engine avance | S68+ | oui |
| 3 | Page React /factory | S68+ | oui |
| 4 | Proof Cards computation | S68 | oui |
| 5 | SearchManifest wire format | S70+ | oui |
| 6 | Babel dogfood via Factory | S69 | oui |
| 7 | @dev index tree-sitter | S70+ | oui |
| 8 | Bridge method proof_card_get | S68+ | oui |
| 9 | Template react-vite | S69+ | oui |
| 10 | Factory audit log JSONL | S68+ | oui |
| 11 | CuratorVouched UI shell | S70+ | oui |
| 12 | Publish path factory→daemon | S68+ | oui |
| 13 | Feed format version bump | post-launch | oui |
| 14 | Fuzzing cargo-fuzz/proptest | post-audit | oui |

**14/14 scope cuts respectes.**

---

## S5 Carries residuels S68

| Item | Reports | Status | Justification |
|---|---|---|---|
| P2-A-1 rand blocker | exemption externe | reconduit | upstream rand 0.9 non publie, dep transitive iroh 0.98 |
| P2-AUDIT-2 iroh transitives | exemption externe | reconduit | herite pin iroh 0.98, evaluate Gate 1 |
| P2-G-1 exe lock intermittent | monitoring | reconduit | non reproductible depuis S62 (5 sprints) |
| T-NN+2 iframe Rust-wasm | bloque upstream | reconduit | toolchain gaps inchanges |
| P2-C-2 path traversal Windows | 1/3 | NEW S67 | validate rejette `..` mais pas test specifique Windows backslash. Phase C review P2. Low risk (sbfb-factory est un outil local) |
| LT-2 Radicle sortie | trigger PENDING | reconduit | tag v1.0 pose localement, pas pousse origin |
| LT-5 redundancy persistence | hors-sprint | reconduit | reclassifie S26 |
| LT-7 worker quorum E2E | post-tag | reconduit | quorum E2E carry post-tag |

Aucun item n'atteint 3/3 MANDATORY au S68.

---

## S6 Checkpoint de cloture

- [x] 29/29 fail-fast verts
- [x] 4 commits feat (Phase A, B, C, D) + 1 commit docs (Phase E)
- [x] verification.md + audit_plan S68 ecrits
- [x] PATTERNS.md mis a jour (P52 BlobStore)
- [x] THREAT_MODEL.md enrichi (T-SEARCH + T-CURATOR-VOUCH + T-SEARCH-DOS, §11)
- [x] CLAUDE.md + SPRINT_LOG.md a jour
- [x] sbfb-factory compile et `sbfb-factory create` fonctionne
- [x] sbfb-manifest partage entre daemon et factory
- [x] Factory ne depend PAS de nexus-shell-daemon-core
- [x] Memory nexus_grid_pivot.md tip + compteurs mis a jour
