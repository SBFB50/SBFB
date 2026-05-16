# Sprint 63 — Verification (verification tiers + UX)

**Ecrit** : 2026-05-15.
**Tip d'entree** : `1405c0c` (post-audit gate S62 PASS).
**Tip de sortie** : `7198ae5`.

---

## §1 Fail-fast checklist

| # | Check | Commande | Resultat | Critere |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | 1305/1305 PASS | >= 1307 → 1305 (delta plan estimait +2 explorer, realite +0 car HTML pur sans tests Rust) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok (1 ignored) | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 errors (5 warnings pre-existants) | 0 errors |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 errors | 0 errors |
| 8 | Vitest | `npm run test:unit` (web/) | 265/265 PASS | >= 264 |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | sync-bridge-sdk | diff web/public/sbfb-bridge.js vs examples/ | identical (SHA256 match) | exit 0 |
| 13 | Playwright setup | N/A (Phase A PLAYWRIGHT-REFACTOR resolue, setup operationnel) | ✅ | global-setup OK |
| 14 | Phase A-D preflights G8 | 4 fichiers sprint63_phase_{A..D}_preflight.md | 4x EXECUTE | EXECUTE |
| 15 | Phase A-D reviews | 4 fichiers sprint63_phase_{A..D}_review.md | 4x PASS | PASS |

**Verdict** : toutes les rows vertes. Sprint 63 livrable.

---

## §2 Compteurs tests

| Suite | Debut S63 | Fin S63 | Delta | Detail |
|---|---|---|---|---|
| Rust nextest | 1299 | 1305 | +6 | Phase B +4 (provenance DB + handlers) + Phase C +2 (feed cursor handlers) |
| Rust doctests | 1 ignored | 1 ignored | 0 | inchange |
| Vitest | 258 | 265 | +7 | Phase C +7 (3 bridge dispatch + 3 VerificationDetail + 1 hash mismatch) |
| Playwright | 0 (blocked) | operationnel (Phase A REFACTOR) | N/A | global-setup refactored pour daemon Rust |
| size-limit | 6/6 | 6/6 | 0 | inchange |
| **Total** | **~1563** | **~1576** | **+13** | |

---

## §3 Phases livrees

| Phase | Commit titre | SHA | Delta tests |
|---|---|---|---|
| A | feat(launcher+web): Sprint 63 Phase A — MANDATORY IMAGE-DEP + PLAYWRIGHT-REFACTOR | Phase A SHA | Rust +0, Vitest +0, Playwright operationnel |
| B | feat(feed): Sprint 63 Phase B — provenance endpoint HTTP + SQLite M12 | Phase B SHA | Rust +4 |
| C | feat(web+bridge): Sprint 63 Phase C — bridge verification + UI VerificationDetail | `272523c` | Rust +2, Vitest +7 |
| Fix | fix(feed): provenance hash linkage — proof-chain integrity | `fa7cd52` | Rust +0 (fix runtime) |
| Fix | fix(feed): provenance insert after blob store + rowid tiebreaker | `5f6a77d` | Rust +0 (fix runtime) |
| D | feat(examples): Sprint 63 Phase D — Protocol Explorer verification + wrap-up | `7198ae5` | Rust +0, Vitest +0 (HTML pur) |

---

## §4 Scope cuts respectes

| # | Item | Sprint cible | Touche S63 ? |
|---|---|---|---|
| 1 | CuratorVouched operation | S64 | Non |
| 2 | BuildQuorumReached operation | S64 | Non |
| 3 | Quarantine feed | S64 | Non |
| 4 | Age witness gate feed | S64 | Non |
| 5 | Multi-forge feed sync | S64+ | Non |
| 6 | Feed format version bump | S64+ | Non |
| 7 | Go-live public + tag push + pilote externe | S65 | Non |
| 8 | CLI verify-release | S64 | Non |
| 9 | Protocol Explorer verification | **LIVRE Phase D** | Oui |
| 10 | VerificationDetail niveau 3 | S64+ | Non |

9/10 scope cuts non touches (1 item livre dans scope Phase D).

---

## §5 Findings carry-over for memory

### Carries resolus S63
- **P2-IMAGE-DEP** 3/3 MANDATORY → **RESOLU Phase A** (image remplace par png)
- **P2-PLAYWRIGHT-REFACTOR** 3/3 MANDATORY → **RESOLU Phase A** (global-setup spawn daemon Rust)

### Carries S64 documentes (from reviews + kickoff)
- P2-PROCESS-FORMAT : plan.md contient estimation LOC — herite, non modifiable (feedback_approach.md §6.7)
- P2-PROVENANCE-404-BRIDGE : 404 ne distingue pas "projet inconnu" de "provenance absente"
- P2-BADGE-WORDING-PREMATURE : badge "Verifie" affiche a l'existence hash, pas apres verification live (pre-existant S14)
- P2-COMMIT-TITLE-FORMAT : feat(web+bridge) vs feat(sprintN) — clarifier PROCESS.md
- P2-REVIEW-ORDER : clarifier si chore review doit preceder feat
- P2-PYTHON-BLOCK-EXEMPTION : ajouter clause exemption SKILL.md Step 2 pour projets sans Python
- F1 P2-VERSION-NOT-STORED : 2/3 → **3/3 MANDATORY S64** (version non stockee en DB)
- F5 P2-IROH-INFRA-TIMEOUT : 2/3 → **3/3 MANDATORY S64** (iroh infra tests timeout intermittent)

### Carries reconduits
- P2-A-1 rand blocker upstream (exemption externe permanente)
- P2-AUDIT-2 iroh transitives pre-release (herite pin 0.98)
- P2-G-1 exe lock intermittent (reouvert, monitoring)
- P2-FEED-INSERT-NO-AUTH-TIER (2/3 → S64+)
- P2-FEED-SUBSCRIBE-JOINHANDLE (2/3 → S64)
- P2-BACKFILL-6PLUS-TEST (2/3 → S64)
- P2-FEED-PUBLISH-ORPHAN (2/3 → S64)
- P2-SUBSCRIBE-STREAM-BREAK (2/3 → S64)

---

## §6 Goal achievement

**Goal** : "Un utilisateur non-technique voit pourquoi un projet est verifie dans
le shell (modal detail provenance) ; un developpeur verifie une release via
endpoint HTTP ; une app iframe interroge la provenance via le bridge — pendant
que les 2 carries MANDATORY 3/3 sont resolus et que Playwright redemarre."

| Objectif | Livre | Phase |
|---|---|---|
| Modal detail provenance (VerificationDetail) | Oui | C |
| Endpoint HTTP provenance | Oui | B |
| Bridge verification (3 methodes) | Oui | C |
| IMAGE-DEP 3/3 resolu | Oui | A |
| PLAYWRIGHT-REFACTOR 3/3 resolu | Oui | A |
| Protocol Explorer verification demo | Oui | D |

**Toutes les objectifs du Goal §2 sont satisfaits.**
