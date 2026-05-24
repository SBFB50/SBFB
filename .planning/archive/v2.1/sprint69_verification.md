# Sprint 69 — Verification (Babel dogfood via Factory + pilote ferme + Gate 1)

**Ecrit** : 2026-05-23.
**Tip master** : `9d9a1e8` (Phase D).
**Roadmap** : v2.1 Arc 2 sprint 3/3 (Factory + RRV @protocole + Canari).

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | PASS |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1430 | PASS 1433/1433 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | PASS |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok | PASS |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok | PASS |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors | PASS |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | PASS |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 279 | PASS 279/279 |
| 10 | npm build | `(cd web && npm run build)` | ok | PASS |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 | PASS 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | PASS |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | PASS |
| 14 | sync-bridge-sdk | `diff web/public/sbfb-bridge.js examples/*/sbfb-bridge.js` | identical | PASS (examples identiques, template factory = starter allege par conception) |
| 15 | preview cap test | `cargo nextest run -p nexus-shell-daemon-core -E 'test(preview)' --locked` | >= 6 | PASS 9/9 |
| 16 | audit_log tests | `cargo nextest run -p sbfb-factory -E 'test(audit)' --locked` | >= 2 | PASS 2/2 |
| 17 | fg8 tests | `cargo nextest run -p sbfb-factory -E 'test(fg8)' --locked` | >= 3 | PASS 3/3 |
| 18 | pipeline tests | `cargo nextest run -p sbfb-factory -E 'test(pipeline)' --locked` | >= 3 | PASS 3/3 |
| 19 | static-reader template | `cargo nextest run -p sbfb-factory -E 'test(static_reader)' --locked` | >= 3 | PASS 3/3 |
| 20 | dead_code cleanup | `grep -c "allow(dead_code)" crates/sbfb-factory/src/gates.rs` | 0 | PASS 0 |
| 21 | THREAT_MODEL preview | `grep -q "Preview" docs/security/THREAT_MODEL.md` | present | PASS |
| 22 | GATE1 test protocol | `test -f docs/release/GATE1_TEST_PROTOCOL.md` | exists | PASS |
| 23 | count-tests script | `test -f scripts/count-tests.sh` | exists | PASS |
| 24 | factory subcommands | `cargo run -p sbfb-factory -- --help` | 7 subcommands | PASS (create, validate, preview, publish, diff, scan-secrets, preview-check) |
| 25 | factory no daemon dep | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent | PASS |
| 26 | verification.md | `test -f .planning/active/sprint69_verification.md` | exists | PASS (ce fichier) |
| 27 | audit_plan S70 | `test -f .planning/active/sprint70_audit_plan.md` | exists | PASS (ecrit meme commit) |

**Resultat : 27/27 PASS.**

---

## §2 Delta tests

| Phase | Rust delta | Vitest delta | Detail |
|---|---|---|---|
| A | +5 | +0 | preview cap +3, audit_log +2 |
| B | +6 | +0 | fg8 +3, pipeline +3 |
| C | +3 | +0 | static_reader template +3 |
| D | +0 | +0 | documentation seulement |
| E | +0 | +0 | documentation seulement |
| **Total S69** | **+14** | **+0** | |

| Suite | Entree S69 | Sortie S69 |
|---|---|---|
| Rust nextest | 1419 | 1433 |
| Vitest | 279 | 279 |
| size-limit | 6/6 | 6/6 |
| **Total** | **~1704** | **~1718** |

**Estime plan** : +13 Rust. **Reel** : +14 (+1 : test preview
supplementaire absorbe Phase A).

**Note P3** : le cumul dans le commit body Phase D (`9d9a1e8`) liste
"Phase A +14, Phase B +14, Phase C +5" ce qui est incorrect. Les
deltas par phase dans les commit bodies individuels (A: +5, B: +6,
C: +3) sont corrects. Le cumul total 1419→1433 (+14) est correct.
Erreur cosmetique dans le recap Phase D, pas un delta reel divergent.

---

## §3 Scope cuts compliance

| # | Item | Sprint cible | Respecte |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | OUI — aucun grep |
| 2 | Page React /factory | S71+ | OUI — aucun composant factory dans web/src/ |
| 3 | @dev index tree-sitter | S71+ | OUI — aucune dep tree-sitter |
| 4 | Template react-vite | S71+ | OUI — seuls templates static + static-reader |
| 5 | CuratorVouched UI shell | S71+ | OUI — feed vouch S67 code-only |
| 6 | FG10 Review gate | S70 | OUI — non implante |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | OUI — aucune dep fuzzing |
| 8 | Feed format version bump | post-launch | OUI — FEED_FORMAT_VERSION = 1 |
| 9 | ProofCard comme feed op | S71+ | OUI — ProofCard local compute seulement |
| 10 | Diff engine avance | S71+ | OUI — diff basique fichier-level |
| 11 | Multi-template switching UI | S71+ | OUI — CLI template choice |
| 12 | Factory update-check | post-launch | OUI — pas de telemetrie |
| 13 | Babel traduction live | post-launch | OUI — reader statique fixtures |
| 14 | iroh 1.0 upgrade | Gate 1 decision | OUI — iroh 0.98 pinne |

**Resultat : 14/14 scope cuts respectes.**

---

## §4 G8 preflight bilan

| Phase | Verdict | Fichier |
|---|---|---|
| A | EXECUTE | sprint69_phase_a_preflight.md |
| B | PLAN-ADAPT | sprint69_phase_b_preflight.md |
| C | EXECUTE | sprint69_phase_c_preflight.md |
| D | EXECUTE | sprint69_phase_d_preflight.md |
| E | EXECUTE | sprint69_phase_e_preflight.md |

**5/5 phases G8, 0 DESIGN-CONFLICT, 4 EXECUTE, 1 PLAN-ADAPT.**
Quarante-neuvieme sprint G8 systematique.

---

## §5 Carries

### Carries CLOSED Sprint 69

| Carry | Phase cloture | Detail |
|---|---|---|
| P2-I-2 delta body 3/3 MANDATORY | Phase D (9d9a1e8) | Compteurs reels chaque phase, script count-tests.sh |
| P3-I-2 dead_code gates | Phase B (aec036b) | FG5/FG7/check_path wires dans pipeline |
| P2-B-1 MAX_PREVIEW_ENTRIES | Phase A (c92e656) | cap 10 + PreviewError::TooManyEntries |

Note : P2-I-3 body docs minimaliste passe de 1/3 a 2/3 (pas
CLOSED). Carry S70. Un sprint supplementaire avec body docs
complet le fermera a 3/3.

### Carries S70 (ouverts)

| Carry | Compteur | Statut | Route S70 |
|---|---|---|---|
| P2-A-1 rand upstream | exemption | Bloquer upstream dep, hors scope agent. | Carry S70 |
| P2-AUDIT-2 iroh transitives | herite | Pin iroh 0.98. Herite du pin. | Carry S70 |
| P2-G-1 exe lock intermittent | monitoring | Non-reproductible 7 sprints consecutifs. | Carry S70, candidat CLOSE |
| T-NN+2 iframe Rust-wasm | deferred | PATTERNS §P34. Hors scope pre-launch. | Carry S70 |
| P2-I-3 body docs minimaliste | 2/3 | S69 Phase D = 2/3. | Carry S70, 3/3 si Phase E body complet |
| LT-2 Radicle | trigger PENDING | Tag v1.0 pose localement, pas pousse origin. | Carry S70 |
| LT-5 redundancy persistence | reclassifie S26 | Post-v1.0 horizon long. | Carry S70 |
| LT-7 worker quorum E2E | post-tag | Tiers 1+2 DONE, quorum E2E carry. | Carry S70 |

---

## §6 Commits Sprint 69

| Phase | SHA | Titre |
|---|---|---|
| chore | b930c34 | chore(planning): Sprint 69 kickoff + plan |
| A | c92e656 | feat(factory): Sprint 69 Phase A — preview cap + audit log + P2-I-2 template |
| B | aec036b | feat(factory): Sprint 69 Phase B — FG8 provenance Ed25519 + FG9 publish pipeline |
| chore | 3a0f8c4 | chore(planning): stage RRV app protocol research document |
| chore | 1edaaa6 | chore(planning): stage RRV research documents (LLM boundary + S70 intake) |
| C | faf4952 | feat(factory): Sprint 69 Phase C — Babel Reader template + dogfood E2E |
| chore | 9e8deb5 | chore(planning): stage S70 research + roadmap v4 D18 + CLAUDE.md state update |
| D | 9d9a1e8 | docs(release): Sprint 69 Phase D — Gate 1 test protocol + pilote ferme prep |
| E | (ce commit) | docs(sprint69): Sprint 69 Phase E — verification + wrap-up |

---

## §7 Checkpoint de cloture (plan §13)

- [x] 27/27 fail-fast verts
- [x] 3 commits feat (Phase A, B, C) + 1 commit docs (Phase D) + 1 commit docs (Phase E en cours)
- [x] verification.md + audit_plan S70 ecrits (ce commit)
- [x] audit_plan S70 route explicitement `Process Portable Complete + Gate 1 dogfood`
- [x] GATE1_TEST_PROTOCOL.md ecrit (Phase D)
- [x] P2-I-2 3/3 CLOSED (script count-tests + procedure)
- [x] P2-B-1 CLOSED (MAX_PREVIEW_ENTRIES = 10)
- [x] P3-I-2 CLOSED (dead_code retires Phase B)
- [x] FG8 + FG9 operationnels (Phase B)
- [x] Template static-reader fonctionnel (Phase C)
- [x] THREAT_MODEL.md §preview ajoute (Phase A §13)
- [x] Memory nexus_grid_pivot.md tip + compteurs a jour (ce commit)
- [x] SPRINT_LOG.md row S69 ajoutee (ce commit)

---

## S70 Routing Check

S70 est route comme **Process Portable Complete + Gate 1 dogfood**
conformement au recadrage PO 2026-05-22 (D18 roadmap v4) et a la
recherche `.planning/research/process_portable_complete_s70.md`.

Verification des preconditions :
- [x] S69 Phases A-D completes (3 feat + 1 docs)
- [x] 0 P0/P1 ouverts
- [x] Gate 1 test protocol ecrit (docs/release/GATE1_TEST_PROTOCOL.md)
- [x] Factory operationnelle (create + validate + preview + publish + pipeline FG4-FG8)
- [x] Template static-reader disponible pour Babel dogfood utilisateur
- [x] Recherche S70 stagee (3a0f8c4 + 1edaaa6 + 9e8deb5)

Non-goals S70 confirmes :
- Pas de RRV total
- Pas de SearchManifest
- Pas de Factory process UI
- Pas d'ingestion OSS broad
- Pas de compute prive/remote

---

## Gate 1 Dogfood Baseline

Etat des 9 criteres Gate 1 (roadmap v4) au tip `9d9a1e8` :

| # | Critere | Etat dev | Pret pilote |
|---|---|---|---|
| G1-1 | Installation Windows/macOS/Linux | Installeurs NSIS/deb/dmg S60 | OUI |
| G1-2 | Connexion P2P (2+ noeuds) | Valide LAN+WAN S53+S60 LT-7 Tier 3 | OUI |
| G1-3 | Deploy app via daemon | deploy-from-repo S59+S63 | OUI |
| G1-4 | Babel via Factory | Template static-reader S69 Phase C + publish pipeline S69 Phase B | OUI (dogfood utilisateur requis) |
| G1-5 | Feed sync entre noeuds | Feed P2P S62 + sync E2E S62 Phase C | OUI |
| G1-6 | Restart daemon propre | Persistence iroh-docs+blobs+feed S66 + E2E restart S66 Phase E | OUI |
| G1-7 | Stabilite 24h | Non teste — pilote ferme | A TESTER |
| G1-8 | RRV search trouve app | FTS5 search S67 Phase B operationnel | OUI (query `?q=babel` apres deploy) |
| G1-9 | Proof Card affichee | ProofCard S68 Phase A-D complet | OUI |

**8/9 criteres prets**, G1-7 (stabilite 24h) a valider par pilote
ferme. Le verdict Gate 1 est pris par l'utilisateur apres retour
testeurs.

---

## Open Carries Routed To S70

| Carry | Route | Justification |
|---|---|---|
| P2-A-1 rand | S70 carry | Upstream dep, exemption permanente |
| P2-AUDIT-2 iroh | S70 carry | Herite pin 0.98, pas d'action avant iroh 1.0 eval |
| P2-G-1 exe lock | S70 candidat CLOSE | Non-repro 7 sprints, monitoring seulement |
| T-NN+2 iframe wasm | S70 carry | PATTERNS §P34, hors scope pre-launch |
| P2-I-3 body docs | S70 3/3 | 2/3 atteint, un body docs(sprint) propre le ferme |
| LT-2 Radicle | S70 carry | Tag v1.0 pas encore pousse origin, trigger PENDING |
| LT-5 redundancy | S70 carry | Post-v1.0 long terme |
| LT-7 quorum E2E | S70 carry | Tier 1+2 DONE, E2E post-tag |

**8 carries ouverts** routes S70. 3 CLOSED ce sprint (P2-I-2,
P3-I-2, P2-B-1). P2-I-3 passe 1/3→2/3 (pas encore CLOSED).
