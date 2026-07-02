# Sprint 80 — Verification (self-report fail-fast)

> Self-report : l'agent qui a écrit le code écrit aussi la vérification —
> valeur bornée par construction, l'audit gate S81 (Phase 0) fait foi.
> 9 sections canoniques README §2.3.

## 1. HEAD entrée / HEAD sortie

- **Entrée** : `f4b4600` (tip S79 — `docs(factory): Sprint 79 Phase I — couche GUIDE llms.txt + WIRING_SPEC + Diataxis Factory + wrap-up`).
- **Sortie** : le commit Phase J qui porte ce fichier (`docs(sprint80): verification + audit plan for Sprint 81 + clôture docs-contrat`).

## 2. Commit stack

`git log --oneline master ^f4b4600` → **49 commits avant Phase J** :
Phase 0 (audit S79 : `96ed018`+`c0a2ffe`+`7f51438` ; `3d5d9dc` = chore
planning S80 adjacent) + kickoff
`5bed616` + phases A `a5ace8d` / B `37daa09` / C `6991d51` / D `152df25` /
E `d59ee32` / F `bb35d39` / G `ed00b4a` / H `5d39a8f` / I `782796c` +
hi-fi/catalogue/fold (`604cc2c`, `0023b45`, `91c0616`, `19da665`, `e036f65`)
+ **arc off-sprint rapid-add** (19 commits `8fa715a`..`94eb030` : lots
rapides, a11y WCAG U1+L3, i18n Lingui socle+gates+51 locales — review
groupée + Codex groupé DUS à la reprise post-S82, memory
`rapid_front_add_session`) + hooks `d1864dc` + process `a6b4ca4` + notes
planning. Phase J clôt la pile.

## 3. How to re-run

```bash
# Bloc Rust (Windows natif)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Bloc Rust (Docker canonique sbfb-ci, dual-platform — AVANT push)
# SBFB_TEST_HTTP_TIMEOUT_SECS : échappatoire documentée
# (crates/sbfb-factory/tests/operator_server.rs) — les tests HTTP loopback
# operator_server dépassent 30s sous Docker-on-Windows (classe env-lenteur,
# verts <1s en natif) ; 120s les laisse converger sans masquer un vrai hang.
MSYS_NO_PATHCONV=1 docker run --rm -v "C:\Users\FlowUP\Documents\Code\nexus:/w" -w /w \
  -v sbfb-ci-target:/target -e CARGO_TARGET_DIR=/target \
  -e SBFB_TEST_HTTP_TIMEOUT_SECS=120 \
  -v sbfb-ci-cargo-registry:/usr/local/cargo/registry sbfb-ci:latest \
  bash -c "cargo fmt --all --check && cargo nextest run --workspace --locked"

# Bloc frontend factory-operator (greenfield S80)
cd tools/factory-operator && npm ci && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && \
  npm run build && npm run size && npm run gates && npm run test:e2e && \
  npm run t2   # régénère .planning/active/sprint80_t2_acceptance.json

# Bloc frontend web/ (shell, non touché par S80)
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && npm run build && \
  npm run size && bash scripts/scan-en-strings.sh

# Doc-lints (clôture docs-contrat)
bash scripts/check-factory-docs.sh
bash scripts/check-frontier-contracts.sh
```

## 4. Checklist (Observed)

| Check | Observed | Verdict |
|---|---|---|
| cargo fmt --check (Win 1.94) | 0 diff | ✅ |
| cargo clippy -D warnings | 0 warning | ✅ |
| cargo nextest (Win natif) | **2014/2014 passed, 0 skipped** | ✅ |
| doctests + release build | verts (bloc Phase I) | ✅ |
| cargo fmt --check (Docker 1.94) | 0 diff (`FMT-OK`) | ✅ |
| cargo nextest (Docker sbfb-ci) | **2018/2018 passed** avec `SBFB_TEST_HTTP_TIMEOUT_SECS=120` ; 2 runs SANS échappatoire = 2016/2018 (2 TimedOut 30s reproductibles, verts natif → classe env-lenteur Docker-on-Windows, carry 19 audit_plan S81) | ✅ (env-note) |
| factory-operator lint+tsc | verts | ✅ |
| Vitest factory-operator | **201/201 (35 fichiers)** | ✅ |
| build + size-limit | verts (hero 37.16/40, verify-surface ≤96, diff-viewer ≤22) | ✅ |
| gates discipline (7 : no-radix / no-tw-config / scan-front+anti-score self-testé / i18n-verdict / i18n-parity / accessibility-system) | tous verts | ✅ |
| T1 E2E Playwright hermétique | **10/10** (workspace fixture scellé, 0 lecture du vrai repo, 0 spawn réel claude) | ✅ **GREEN** |
| T2 acceptance | `sprint80_t2_acceptance.json` **PASS** (9 gates + 10 scénarios), COMMITTÉ `782796c` | ✅ **PASS** |
| Vitest web/ | **411/411** + coverage 87.27/79.01/86.02/88.59 ≥ seuils | ✅ |
| check-factory-docs.sh (après clôture) | clean (links+anchors+honesty+french-body+source-ref 15/15+fiche) | ✅ |
| check-frontier-contracts.sh | clean | ✅ |

## 5. Métriques sprint

| Suite | Avant (entrée S80) | Après | Delta |
|---|---|---|---|
| Rust nextest Windows | **1994** (fin S79, `SPRINT_LOG` row 79 : 1991→1994) | **2014** | **+20** (routes sbfb-factory A/F/G ; S80 front-dominant ; 1949 = fin S77, pas S79) |
| Rust nextest Docker Linux | 1998 (fin S79 = Win+4) | **2018** | **+20** (+4 `#[cfg(unix)]` vs Win, invariant conservé) |
| Vitest factory-operator | 7 (ancien front, jeté Phase B) | **201** (35 fichiers) | suite REBÂTIE de 0 : C 52 → D 77 → E 92 → H 137 → I 201 (+ arc off-sprint) |
| E2E Playwright operator | 0 | **10** | +10 (T1 hermétique) |
| Vitest web/ | 411 | **411** | 0 (shell non touché) |
| size-limit operator | n/a (greenfield) | 6/6 budgets verts | hero 37.16/40 KB |

## 6. Surface nouvelle livrée (LOC par module)

- `tools/factory-operator/` (greenfield complet) : ~9 500 LOC src/ + e2e/ +
  scripts/ (React 19 + Compiler, Base UI seule dep primitives, Tailwind v4
  oklch, motion 5 signatures, i18n Lingui 51 locales, harness hermétique).
- `crates/sbfb-factory/src/operator_server.rs` : +~700 (bootstrap cookie +
  routes diff/gates + router split).
- `crates/sbfb-factory/src/gates.rs` : +~250 (GatesView/GateEntryView/
  GateIssueView + gates_live_data).
- `crates/sbfb-factory/src/sprint_history.rs` : +~300 (working_tree_diff_data).
- `crates/sbfb-factory/src/auth.rs` : +~150 (cookie fallback + session_secret).
- `docs/factory/` : clôture docs-contrat (llms.txt H2 Operator + REFERENCE
  §Operator + EXPLANATION pointeur).

## 7. Ce que le sprint n'a PAS livré (scope cuts respectés — §Out kickoff, exhaustif)

- ❌ **Aperçu scellé + Proof Card (Viewer scellé)** — reporté S81+ (fondation Viewer re-planifiée, carry 12/21 audit_plan).
- ❌ **Verbe `publish` via l'Operator** — publish reste CLI ; PASS refusé via Operator (invariant tenu, testé).
- ❌ **Éditeur CM6 riche** — différé (le terminal xterm reste le cœur).
- ❌ **Palette transversale ⌘K** — non livrée comme cadre (un raccourci focal clavier a été livré par l'arc off-sprint : accélérateur, pas cadre — conforme).
- ❌ **Multi-session board / Mission-Control** — coupé ; simple liste de sessions tiroir livrée (conforme au cut).
- ❌ **Timeline-canvas de procédé** — différé ; l'arbre de procédé restitué (Phase D) n'est pas un canvas.
- ❌ **i18next + router complexe** — i18next jamais introduit ; note honnête : l'arc OFF-SPRINT a livré **Lingui** sur OVERRIDE PO explicite (memory `factory_universal_ux_pivot`, « à ne pas re-débattre ») — ce n'est pas une violation du cut (qui visait i18next), c'est un pivot PO tracé hors cadre de phase.
- ❌ **Auto-bascule STEER→VERIFY arrachée au stream** — INTERDITE et non livrée (bascule MANUELLE D6, testée e2e).
- ❌ (héritage Phase H) V5/V6 niveau ligne + marqueur-gate-par-fichier + fraîcheur live — dégradés, carries 11/12/13 audit_plan S81.
- ❌ Sharding S77 in-vivo + app-authoring in-vivo — P1 standing inchangés (aucune prétention S80).

## 8. Findings carry-over for memory (G6)

1. **Leçon hook** : le gate commit matche le PREMIER `^## Verdict` du
   review.md — jamais deux headers verdict (ancré `feedback_body_section_headers`).
2. **Canon amendé `a6b4ca4`** : clôture docs-contrat = livrable de
   fermabilité (DoD (d)) avec 3 porteurs — 1re application = cette Phase J.
3. **Classe env Docker-on-Windows** : tests HTTP loopback operator_server
   TimedOut 30s reproductibles (verts natif) → toujours passer
   `SBFB_TEST_HTTP_TIMEOUT_SECS=120` au run sbfb-ci local.
4. **§P72** : jamais `cargo run` depuis un cwd Temp (perte .cargo/config) ;
   jamais spawnSync `.cmd` win32 sans shell.
5. **Arc off-sprint** : review groupée + Codex groupé DUS à la reprise
   post-S82 (`wip/factory-front-arc-post-s82` + 16 commits sur master).

## 9. Checkpoint de clôture

- [x] 10 phases A-J livrées, 1 commit atomique chacune, artefacts
      preflight/review/codex présents pour chaque phase de code.
- [x] DoD (a) : objectifs roadmap S80 atterris (greenfield bi-focal
      STEER/VERIFY complet + auth cookie + 2 routes backend + testabilité).
- [x] DoD (b) : carries routés — CLOSED (TEST-ISOLATION-SBFB-HOME, P3-6,
      S2-F2, P2-1) ou re-routés avec rationale (`sprint81_audit_plan.md §3`,
      21 items, zombies filtrés).
- [x] DoD (c) : gate testabilité VERT — **T1 GREEN** (10/10 hermétique,
      CI chaque push) + **T2 PASS** (JSON committé, RIG-ABSENT illégitime
      par construction).
- [x] DoD (d) — NEUF (`a6b4ca4`) : **clôture docs-contrat livrée** — les
      4 frontières S80 (auth cookie `a5ace8d`, GET /api/git/diff `bb35d39`,
      GET /api/gates `ed00b4a`, contrat SSE `6991d51`) indexées dans
      `docs/factory/llms.txt` (H2 + 15 source-refs) + `REFERENCE.md`
      (§Operator control-plane API, 4 contrats) + `EXPLANATION.md`
      (pointeur FR) ; `WIRING_SPEC.md` NON concerné (sous-domaine
      sealed-iframe distinct) ; `check-factory-docs.sh` clean.
- [x] Dual-platform : fmt 0 sous les 2 toolchains ; nextest Win 2014 +
      Docker 2018 verts (échappatoire 120s documentée §4).
- [x] verification.md + sprint81_audit_plan.md écrits (ce commit).
- [x] SPRINT_LOG row 80 + CLAUDE.md + memory mis à jour (ce commit + G6).
