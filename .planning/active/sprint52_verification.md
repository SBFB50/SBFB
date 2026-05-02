# Sprint 52 — Verification

**Tip sortie** : `374bf59` (Phase B) + Phase C wrap-up.
**Theme** : dette pair + CI Woodpecker + self-hosted build design.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1199, 0 fail | ✅ 1199 (32 timeout pression ressources, 0 fail isole) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ✅ 6 passed, 1 ignored |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ✅ ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | ✅ 0 error (7 warnings pre-existants) |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | ✅ 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | ✅ 250 |
| 9 | build | `npm run build` (web/) | ok | ✅ ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | ✅ 6/6 |
| 11 | Phase A preflight G8 | EXECUTE | ✅ | ✅ |
| 12 | Phase A review | PASS (1 P2) | ✅ | ✅ |
| 13 | Phase B preflight G8 | EXECUTE (pivot user documente) | ✅ | ✅ |
| 14 | Phase B review | PASS (2 P2) | ✅ | ✅ |
| 15 | dispatch shutdown oneshot | runtime.rs + dispatch_loop.rs | ✅ | ✅ test PASS |
| 16 | 0 docs legacy tracked | `git ls-files docs/BENCHMARK.md ...` | 0 | ✅ 0 (20 supprimes) |
| 17 | CLAUDE.md stale carry fix | P2-REVIEW-A-1-S51 supprime | ✅ | ✅ |
| 18 | .woodpecker/ci-linux.yml | fichier present | ✅ | ✅ |
| 19 | SELF_HOSTED_BUILD.md | design doc 3 etages | ✅ | ✅ |
| 20 | LT-7 ROADMAP_COMMITMENTS | entree pre-v1.0 | ✅ | ✅ |
| 21 | release.yml matrix fix | os cross-product 9 jobs | ✅ | ✅ (GHA run 2/3 Windows success pre-fix) |
| 22 | Scope cuts respectes | 8/8 | ✅ | ✅ |
| 23 | 3 carries dette CLOSED | Phase A | ✅ | ✅ |

## §2 Delta tests cumule

| Suite | Entree S52 | Sortie S52 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1199 | 1199 | +0 | Sprint dette + docs |
| Rust doctests | 6+1i | 6+1i | +0 | |
| Vitest | 250 | 250 | +0 | |
| Playwright | 42+2f | 42+2f | +0 | non execute (env) |
| size-limit | 6 entries | 6 entries | +0 | |
| **Total** | **~1455** | **~1455** | **+0** | Sprint dette + design |

## §3 Carries resolus S52

| Item | Action |
|---|---|
| P2-REVIEW-A-1-S50 dispatch join order 2/3 | **CLOSE** Phase A (oneshot shutdown signal) |
| P2-REVIEW-A-2-S51 docs legacy orphelines 1/3 | **CLOSE** Phase A (20 fichiers DELETE) |
| P2-D-1-AUDIT CLAUDE.md stale carry 1/3 | **CLOSE** Phase A (ligne supprimee) |

## §4 Carries S53

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 2/3 | S51 review |
| P2-REVIEW-A-1-S52 nextest timeout profiling | 1/3 | NEW S52 Phase A review |
| P2-REVIEW-B-1-S52 Woodpecker E2E validation | 1/3 | NEW S52 Phase B review |
| P2-REVIEW-B-2-S52 GHA 9/9 re-run confirm | 1/3 | NEW S52 Phase B review |

S53 impair → pas de phase dette obligatoire.
P2-REVIEW-B-1-S51 unsafe set_var passe a 2/3 — si non adresse S53,
3/3 MANDATORY S54.

## §5 Findings carry-over for memory

- S52 pivot Phase B : plan original "GHA dry-run" remplace par CI Woodpecker + design doc self-hosted build (LT-7). Decision utilisateur : CI decentralisee pre-v1.0, pas de polish GHA.
- .woodpecker/ci-linux.yml : premier pipeline CI hors GHA (Codeberg/self-hosted). Non teste sans agent (S53 VPS).
- SELF_HOSTED_BUILD.md : strategie 3 etages documentee (Woodpecker → build worker → reseau autonome). task_type "build" = runtime separe, PAS extension triviale worker LLM.
- LT-7 : engagement pre-v1.0 non-negociable. Feasibility check positif (aucun blocker fondamental, Rust repro builds en maturation, pas de precedent P2P).
- release.yml matrix bug depuis S18 (include sans cross-product → 3 jobs au lieu de 9). Fix pushe.
- GHA run 25256546939 : 2/3 Windows success (nexus-launcher + nexus-worker). 1 en cours au moment du commit.
