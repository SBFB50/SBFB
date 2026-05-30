# Sprint 70 — Verification (Process Portable Complete + Gate 1 dogfood)

**Ecrit** : 2026-05-25.
**Tip master** : `6201f11` (Phase F + 2 chore).
**Roadmap** : v2.1 Arc 2.5 sprint 1/1 (Process Portable Complete).

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | PASS |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1486 | PASS 1486/1486 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | PASS |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok | PASS |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok | PASS |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors | PASS (5 warnings, 0 errors) |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | PASS |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 279 | PASS 279/279 |
| 10 | npm build | `(cd web && npm run build)` | ok | PASS |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 | PASS 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | PASS |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | PASS |
| 14 | RRV contract exists | `test -f docs/agent/RRV_FACTORY_CONTRACT.md` | exists | PASS |
| 15 | RRV 5 modes | `rg -c "@research\|@dev\|@audit\|@security\|@product" docs/agent/RRV_FACTORY_CONTRACT.md` | >= 5 | PASS |
| 16 | Factory split | `rg -c "Factory Viewer\|Factory Operator" docs/agent/RRV_FACTORY_CONTRACT.md` | >= 2 | PASS |
| 17 | verification exists | `test -f .planning/active/sprint70_verification.md` | exists | PASS (ce fichier) |
| 18 | audit plan S71 | `test -f .planning/active/sprint71_audit_plan.md` | exists | PASS |
| 19 | 7 phases aligned | `rg "7 phases\|A-G" docs/claude/SPRINT_LOG.md CLAUDE.md` | >= 1 match | PASS |
| 20 | AGENT_SYSTEM.md | `test -f docs/agent/AGENT_SYSTEM.md` | exists | PASS (Phase A) |
| 21 | PROVIDER_CONFIG.md | `test -f docs/agent/PROVIDER_CONFIG.md` | exists | PASS (Phase F) |
| 22 | handoff prompt | `test -f prompts/agent/handoff.md` | exists | PASS (Phase C) |
| 23 | sbfb-factory process | `cargo run -p sbfb-factory -- process --help` | subcommands | PASS (status-sprint, lint-planning, audit-commit, prompt, context) |
| 24 | sbfb-factory operator | `cargo run -p sbfb-factory -- operator --help` | serve subcommand | PASS |
| 25 | factory-ui Viewer | `test -d examples/sbfb-factory-viewer` | exists | PASS (Phase E) |
| 26 | factory-ui Operator | `test -d tools/factory-operator` | exists | PASS (Phase E) |
| 27 | hooks dynamiques | `! grep -q "sprint67\|sprint 67" .claude/hooks/*.sh` | no stale refs | PASS (Phase F) |

**Resultat : 27/27 PASS.**

---

## §2 Delta tests

| Phase | Rust delta | Vitest delta | Detail |
|---|---|---|---|
| A | +0 | +0 | docs-only (AGENT_SYSTEM.md + AGENTS.md) |
| B | +0 | +0 | docs-only (PATTERNS.md + README.md) |
| C | +16 | +0 | 8 prompt kinds + 3 aliases + 1 unknown + 2 provider + 2 context JSON |
| D | +32 | +0 | status-sprint, lint-planning, audit-commit, operator serve, endpoints, guards |
| E | +0 | +0 | Factory Viewer + Operator (build/lint/tsc checks, pas Rust unit) |
| F | +5 | +0 | chore gate, verdict exact, provider human, status spaced verdict |
| G | +0 | +0 | docs-only (RRV contract + verification + wrap-up) |
| **Total S70** | **+53** | **+0** | |

| Suite | Entree S70 | Sortie S70 |
|---|---|---|
| Rust nextest | 1433 | 1486 |
| Vitest | 279 | 279 |
| size-limit | 6/6 | 6/6 |
| **Total** | **~1718** | **~1771** |

**Estime plan** : +45 Rust. **Reel** : +53 (+8 : tests supplementaires
Phase C aliases/unknown/context + Phase D securite/regression).

---

## §3 Scope cuts compliance

| # | Item | Sprint cible | Respecte |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | OUI — aucun grep |
| 2 | Route /factory shell produit | S71+ | OUI — aucun composant factory dans web/src/pages/ |
| 3 | @dev index tree-sitter | S71+ | OUI — aucune dep tree-sitter |
| 4 | Template react-vite | S71+ | OUI — seuls templates static + static-reader |
| 5 | CuratorVouched UI shell | S71+ | OUI — feed vouch code-only |
| 6 | FG10 Review gate auto | S71+ | OUI — non implante |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | OUI — aucune dep fuzzing |
| 8 | Feed format version bump | post-launch | OUI — FEED_FORMAT_VERSION = 1 |
| 9 | ProofCard comme feed op | S71+ | OUI — ProofCard local compute seulement |
| 10 | iroh 1.0 upgrade | Gate 1 decision | OUI — iroh 0.98 pinne |
| 11 | CI multi-provider | S71 | OUI — absent du diff |
| 12 | Provider router multi-LLM | post-S75 | OUI — absent |
| 13 | sbfb-search crate | S71+ | OUI — absent |
| 14 | Ingestion OSS broad | post-S75 | OUI — absent |

**Resultat : 14/14 scope cuts respectes.**

---

## §4 G8 preflight bilan

| Phase | Verdict | Fichier |
|---|---|---|
| A | EXECUTE | sprint70_phase_a_preflight.md |
| B | EXECUTE | sprint70_phase_b_preflight.md |
| C | EXECUTE | sprint70_phase_c_preflight.md |
| D | EXECUTE | sprint70_phase_d_preflight.md |
| E | EXECUTE | sprint70_phase_e_preflight.md |
| F | EXECUTE | sprint70_phase_f_preflight.md |
| G | EXECUTE | sprint70_phase_g_preflight.md |

**7/7 phases G8, 0 DESIGN-CONFLICT, 7 EXECUTE, 0 PLAN-ADAPT.**
Cinquantieme sprint G8 systematique.

---

## §5 Carries

### Carries CLOSED Sprint 70

| Carry | Phase cloture | Detail |
|---|---|---|
| P2-I-3 body docs minimaliste 3/3 MANDATORY | Phase B (bb554f6) | Body complete avec doc references, TOOLING.md, README.md align |
| P2-G-1 exe lock intermittent | Phase B (bb554f6) | CLOSE — 8 sprints non-reproductible, 5 builds consecutifs Phase B |
| P2-C-1 canonical bytes duplication | Phase B (bb554f6) | Documente PATTERNS.md T-NN+3, Factory copie justified |
| P2-C-2 serde_json vs JCS | Phase B (bb554f6) | Documente PATTERNS.md, serde_json::Value pour raw-op |
| P2-I-1 docs dans chore | Phase B (bb554f6) | Documente README.md §4.1 types valides |

### Carries S71 (ouverts)

| Carry | Compteur | Statut | Route S71 |
|---|---|---|---|
| P2-A-1 rand upstream | exemption | Bloquer upstream dep, hors scope agent. | Carry S71 |
| P2-AUDIT-2 iroh transitives | herite | Pin iroh 0.98. Herite du pin. | Carry S71 |
| T-NN+2 iframe Rust-wasm | deferred | PATTERNS §P34. Hors scope pre-launch. | Carry S71 |
| LT-2 Radicle | trigger PENDING | Tag v1.0 pose localement, pas pousse origin. | Carry S71 |
| LT-5 redundancy persistence | reclassifie S26 | Post-v1.0 horizon long. | Carry S71 |
| LT-7 worker quorum E2E | post-tag | Tiers 1+2 DONE, quorum E2E carry. | Carry S71 |
| P2-F-3 prompt file coupling | 1/3 | Carry-over awareness Phase F. | Carry S71 |

---

## §6 Commits Sprint 70

| Phase | SHA | Titre |
|---|---|---|
| chore | 78e4413 | chore(planning): Sprint 70 kickoff + plan |
| chore | 1395020 | chore(planning): S70 plan adjustment — full prompt portability + provider config |
| chore | 3d6f4a9 | chore(planning): S70 plan v3 — Factory Process Dashboard + shadcn + 7 phases |
| chore | fa7ce72 | chore(planning): S70 plan v4 — recadrage PO + review 4 agents deep |
| chore | c4494a6 | chore(planning): S70 plan v5 — alignement audit_plan + design_review + CLAUDE.md avec Viewer/Operator split |
| A | 92a4d19 | docs(sprint70): Sprint 70 Phase A — AGENT_SYSTEM.md canon portable + AGENTS.md cleanup |
| chore | 990ae82 | chore(planning): reconcile Phase A review — promote PASS-PENDING to PASS |
| B | bb554f6 | docs(patterns): Sprint 70 Phase B — dette pair T-NN+3 + P2-G-1 CLOSE + chore/feat split |
| C | c68e989 | feat(agent): Sprint 70 Phase C — prompt portability full (8 kinds executables) |
| D | 69e3a06 | feat(factory): Sprint 70 Phase D — process Rust + Operator serve JSON API |
| E | c12aadb | feat(factory): Sprint 70 Phase E — Factory Viewer + Operator local action-gated |
| F | 6fb95df | feat(agent): Sprint 70 Phase F — agent refactor wrappers + hooks dynamises + provider config |
| chore | 287fd9d | chore(planning): fix false positive PASS-PENDING check in task gate hook |
| chore | 6201f11 | chore(factory): fix SprintOverview API mapping for operator serve response |
| G | (ce commit) | docs(sprint70): Sprint 70 Phase G — RRV/Factory contrat + verification + wrap-up |

---

## §7 Checkpoint de cloture

- [x] 27/27 fail-fast verts
- [x] 7 commits phase (A-G) + 7 chore planning/fix
- [x] verification.md + audit_plan S71 ecrits (ce commit)
- [x] AGENT_SYSTEM.md canon portable (Phase A)
- [x] Dette pair P2-I-3 3/3 MANDATORY CLOSED + 4 P2 absorbes (Phase B)
- [x] Prompt portability full 8 kinds executables (Phase C)
- [x] sbfb-factory process status-sprint/lint-planning/audit-commit + operator serve (Phase D)
- [x] Factory Viewer protocole + Factory Operator local action-gated (Phase E)
- [x] Hooks dynamises + provider config + dogfood (Phase F)
- [x] RRV_FACTORY_CONTRACT.md 5 modes @ + Factory split + sequencing (Phase G)
- [x] 14/14 scope cuts respectes
- [x] 7/7 phases G8 EXECUTE (0 DESIGN-CONFLICT)
- [x] Memory nexus_grid_pivot.md tip + compteurs a jour (ce commit)
- [x] SPRINT_LOG.md row S70 ajoutee (ce commit)
- [x] Arc 2.5 Process Portable Complete COMPLET

---

## S71 Routing Check

S71 est route comme premier sprint Arc 3 (Reseau Verifiable +
Industrialisation). Le contenu exact (SearchManifest opt-in vs RRV
Core vs dette pair) sera decide par l'audit S70 et le kickoff S71.
La roadmap v4 positionne S71 comme "SearchManifest opt-in ou RRV
Core selon audit S70". S70 livre le process portable complet qui
est prerequis pour S71.
