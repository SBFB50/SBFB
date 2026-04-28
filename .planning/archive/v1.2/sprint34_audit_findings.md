# Sprint 34 — Audit findings (audit gate S34→S35)

**Auditeur** : session fraiche 2026-04-28
**Sprint audite** : S34 (UX launcher cross-platform + dette pair)
**Tip audite** : `65afb77` (HEAD master)
**Audit plan** : `.planning/active/sprint35_audit_plan.md`

## Verdict : PASS (0 P0, 0 P1, 2 P2, 1 P3)

Aucun bloqueur. Sprint 35 peut demarrer.
G4 rigor signal satisfait (2 P2 + 1 P3 documentes).

---

## Track A — Phase A correctness (dette)

| # | Check | Verdict | Detail |
|---|---|---|---|
| A1 | COEP E2E 3 headers | PASS | `blob_serve_coep.rs:55-74` — COOP, COEP, CSP tous asserts |
| A2 | frost 3.0 sig byte-identical | PASS | `frost.rs:343-368` — `verify()` Ed25519 RFC 8032 ok, `verify_canary()` round-trip |
| A3 | frost DKG ceremony | PASS | `dkg.rs:127-189` + `ceremony.rs:215-326` — 4+4 tests couvrent roundtrip, insufficient signers, tamper, canary compat |
| A4 | rand triple documented | PASS | Phase A commit body documente explicitement : frost-core rand_core 0.6 + iroh 0.9/0.10, sous-arbres disjoints |
| A5 | No RC/pre-release in lock | PASS | frost 3.0 chain propre (stable). Pre-releases existantes (curve25519-dalek 5.0.0-pre.6, hickory 0.26.0-beta.4, etc.) sont iroh transitives **pre-existantes** avant S34, pas regression Phase A |

## Track B — Phase B correctness (Windows UX)

| # | Check | Verdict | Detail |
|---|---|---|---|
| B1 | build.rs cross-platform | PASS | `build.rs:3-8` — tout le bloc winresource sous `#[cfg(windows)]` |
| B2 | windows_subsystem conditionnel | PASS | `main.rs:2` — `cfg_attr(not(debug_assertions), ...)` |
| B3 | Log before spawn | PASS | `setup_file_logging()` ligne 224 (premier appel dans main), spawn_daemon en aval |
| B4 | Panic hook to log | PASS | `main.rs:64-68` — panic hook ecrit dans le meme fichier log |
| B5 | ICO multi-resolution | PASS | 4 tailles : 16/32/48/256 (python struct verification) |

## Track C — Phase C correctness (macOS/Linux)

| # | Check | Verdict | Detail |
|---|---|---|---|
| C1 | Info.plist CFBundleExecutable | PASS | `Info.plist:7` = `nexus-launcher` = cargo target name |
| C2 | bundle-macos.sh structure | PASS | `MacOS` + `Resources` dirs, `chmod +x`, .icns fallback .png |
| C3 | .desktop freedesktop | PASS | Type, Name, Exec, Icon, Terminal=false, Categories presents |
| C4 | install-node.sh sed | PASS | `sed "s\|Exec=.*\|Exec=$launcher_bin\|"` — delimiteur `\|` evite conflit `/` dans paths |
| C5 | PNG 256x256 | PASS | Exactement 256x256 (python struct verification) |

## Track D — Cross-phase integration

| # | Check | Verdict | Detail |
|---|---|---|---|
| D1 | Log path consistent | PASS | `~/.sbfb/launcher.log` (main.rs:53), running.json delegue a daemon-core (fixe Phase D) |
| D2 | COEP real daemon | PASS | `DaemonCluster::spawn(1)` — aucun mock, vrai processus daemon |
| D3 | frost 3.0 Cargo.lock | PASS | frost-ed25519 3.0.0, frost-core 3.0.0, frost-rerandomized 3.0.0 — tous stables |

## Track E — Security & hardening

| # | Check | Verdict | Detail |
|---|---|---|---|
| E1 | COEP headers all responses | PASS | `http.rs:269-282` — CSP + x-content-type-options + COOP + COEP injectes sur chaque reponse blob-serve |
| E2 | Auth token correct path | PASS | `main.rs:501-504` — delegue a `daemon_core::auth::{auth_token_path, load_or_generate_token}` |
| E3 | No secrets in log | PASS | Aucun `lprint!`/`println!` ne log la valeur du token. Seuls les paths et erreurs sont traces |

## Track F — Meta-process

| # | Check | Verdict | Detail |
|---|---|---|---|
| F1 | G8 preflight 3/3 | PASS | `sprint34_phase_{A,B,C}_preflight.md` existent, 3 verdicts EXECUTE |
| F2 | Phase review 3/3 | PASS | `sprint34_phase_{A,B,C}_review.md` + Phase D review (4 fichiers) |
| F3 | Commit bodies structures | PASS | 3 feat commits avec delta tests, scope cuts, G8 trace, pre-launch protocol |
| F4 | Carry counters | PASS | verification.md §5 = 7 carries documentes, coherents avec reviews |
| F5 | MANDATORY 3/3 | PASS | P2-A-1 rand FERME, P2-B-1 tor-rtcompat FERME S33, P2-REVIEW-C-2 COEP FERME |

---

## Findings hors-plan

### P2-AUDIT-1 : main.rs uncommitted CREATE_NO_WINDOW

**Fichier** : `crates/nexus-launcher/src/main.rs:208-214`
**Constat** : un diff non committe ajoute `CREATE_NO_WINDOW`
(`0x08000000`) au `Command::new` de `spawn_daemon()` sur Windows.
Ce flag empeche la fenetre console du daemon enfant de flasher a
cote du launcher (qui a deja `windows_subsystem = "windows"` pour
sa propre console).

**Impact** : UX — sans ce flag, le launcher release cache sa
propre console mais le daemon child en ouvre une brievement.
**Classification** : P2 (UX degradee, pas de securite).
**Action** : integrer dans S35 Phase A ou committer comme
fix(launcher) standalone avant le kickoff S35.

### P2-AUDIT-2 : pre-release transitives dans Cargo.lock

**Constat** : le Cargo.lock contient des versions pre-release
(transitives iroh/ed25519-dalek chain) :
- `curve25519-dalek 5.0.0-pre.6`
- `ed25519 3.0.0-rc.4`, `ed25519-dalek 3.0.0-pre.6`
- `pkcs8 0.11.0-rc.11`, `signature 3.0.0-rc.10`
- `sha2 0.11.0-rc.5`
- `hickory-proto/resolver 0.26.0-beta.4`

**Impact** : ces versions sont utilisees en production par iroh
0.98 et n'ont pas d'alternative stable disponible (le graph de
dependances iroh les impose). Pre-existantes avant S34, pas une
regression Phase A. Cependant, elles representent un risque de
stabilite API si les RC evoluent.
**Classification** : P2 (risque transitive, carry existant).
**Action** : documenter dans PATTERNS.md §tech-debt + re-evaluer
a chaque upgrade iroh (0.99 quand dispo).

### P3-AUDIT-1 : fichiers untracked ambigus

**Constat** : working tree contient des fichiers untracked qui ne
font pas partie du workflow SBFB :
`.githooks/`, `AGENTS.md`, `docs/agent/`, `prompts/`,
`scripts/agent/`, `tests/test_agentctl.py`,
`packages/nexus-coordinator/nexus-coordinator.spec`.
**Impact** : pollution du working tree, risque de commit accidentel.
**Classification** : P3 (hygiene).
**Action** : decider si .gitignore, suppression, ou integration.

---

## Coherence verification.md

La verification S34 (31/31 verts) est coherente avec les checks
de cet audit. Aucun ecart entre les claims de verification et
l'etat reel du code.

## Compteurs tests confirmes

| Suite | Verification S34 | Audit recheck |
|---|---|---|
| Rust nextest | 902 | 902 (coherent) |
| Total | ~1905 | ~1905 (coherent) |
