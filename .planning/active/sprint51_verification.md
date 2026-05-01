# Sprint 51 — Verification

**Tip sortie** : `54e8af0` (Phase B) + Phase C wrap-up.
**Theme** : suppression legacy + CI post-Python + carries 2/3.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings (apres fix print_stub order) |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1199, 0 fail | ✅ 1199 passed |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ✅ ok (0 passed, 1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ✅ ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | ✅ 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | ✅ 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | ✅ 250 |
| 9 | build | `npm run build` (web/) | ok | ✅ ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | ✅ 6/6 |
| 11 | Phase A preflight G8 | EXECUTE | ✅ | ✅ |
| 12 | Phase A review | PASS (2 P2, 1 P3) | ✅ | ✅ |
| 13 | Phase B preflight G8 | EXECUTE | ✅ | ✅ |
| 14 | Phase B review | PASS (1 P2, 1 P3) | ✅ | ✅ |
| 15 | 0 fichier Python workspace | `git ls-files nexus/ tests/*.py worker/ pyproject.toml uv.lock` | 0 | ✅ 0 |
| 16 | build-wheels.yml supprime | `git ls-files .github/workflows/build-wheels.yml` | 0 | ✅ 0 |
| 17 | ci.yml sans Python | grep Python ci.yml | 0 | ✅ 0 |
| 18 | release.yml sans nexus-core-py | grep nexus-core-py release.yml | 0 | ✅ 0 |
| 19 | ci-smoke 4 scripts preserves | `ls scripts/ci-smoke/` | 4 | ✅ 4 |
| 20 | 3 carries S48 CLOSED | Phase B evidence | ✅ | ✅ |
| 21 | release-attest.sh nettoye | grep nexus-core-py scripts/release-attest.sh | 0 | ✅ 0 |
| 22 | CLAUDE.md mis a jour | Python refs removed | ✅ | ✅ |
| 23 | Scope cuts respectes | 8/8 | ✅ | ✅ |

## §2 Delta tests cumule

| Suite | Entree S51 | Sortie S51 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1199 | 1199 | +0 | Sprint soustractif |
| Rust doctests | 6+1i | 6+1i | +0 | |
| Vitest | 250 | 250 | +0 | |
| Playwright | 42+2f | 42+2f | +0 | non execute (env) |
| size-limit | 6 entries | 6 entries | +0 | |
| **Total** | **~1455** | **~1455** | **+0** | Sprint soustractif |

## §3 Carries resolus S51

| Item | Action |
|---|---|
| P2-REVIEW-A-1-S48 canary reload size cap 2/3 | **CLOSE** Phase B (cap implemente + teste) |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel 2/3 | **CLOSE** Phase B (test-only, 0 production) |
| P2-AUDIT-A-1-S48 carry doc accuracy 2/3 | **CLOSE** Phase B (Python supprime, Rust naming OK) |
| P2-REVIEW-B-1-S50 nexus/ legacy monolith 1/3 | **CLOSE** Phase A (DELETE) |
| P2-REVIEW-A-1-S51 release-attest.sh dead code | **CLOSE** Phase C (nexus-core-py path supprime) |

## §4 Carries S52

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S50 dispatch join order | 2/3 | S50 review |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 1/3 | NEW S51 Phase B review |
| P2-REVIEW-A-2-S51 docs legacy orphelines | 1/3 | NEW S51 Phase A review |

S52 pair → phase dette obligatoire (§6.2.1 Regle 1).
P2-REVIEW-A-1-S50 dispatch join order passe a 2/3 — si non adresse
S52, il devient 3/3 MANDATORY S53.

## §5 Findings carry-over for memory

- S51 supprime le monolithe nexus/ (188 fichiers) + worker/ + tests/ + pyproject.toml + uv.lock — -72 335 LOC total
- Plus aucun fichier Python dans le workspace git
- CI simplifie : 2 blocs (Rust + Frontend), 13 steps ci.yml (vs 18)
- release.yml : 1 job build-binaries (vs 3 avec PyO3/PyPI)
- 3 carries S48 a 2/3 CLOSED sans modification code (verification factuelle)
- release-attest.sh nettoye (dead code path nexus-core-py supprime)
- 21 fichiers docs/ legacy encore presents (BENCHMARK.md, ARCHITECTURE.md, etc.) — carry P2-REVIEW-A-2-S51
- Clippy fix : print_stub deplace avant #[cfg(test)] module (items_after_test_module lint Rust 1.94)
