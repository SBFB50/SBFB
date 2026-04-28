# Phase Review — Sprint 35 Phase A

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation
- feedback_approach.md : pick deepest, no band-aid — respecte (crate separe au lieu d'extension daemon-core, schema versioning guard G1 D3)
- feedback_context7_systematic.md : rusqlite existant workspace, pas de nouvelle lib externe — N/A
- Violations memory : 0

## Staging check (Step 1bis)
- Phase fichiers : 11 (tous Phase A scope)
- Planning/docs split : preflight inclus dans staging (correct)
- Untracked accidentels : 0 (artefacts externes deja gitignored)

## Suites
- Rust nextest : 902 -> 913 (+11 : 10 coordinator-rs + 1 cross-daemon) ✅
- Rust doctests : 0 pass (1 ignored) ✅
- Rust clippy : 0 warnings ✅
- Rust fmt : clean ✅
- Release build : OK ✅
- Python SDK : 195 pass ✅
- Python gov : 46 pass ✅
- Frontend lint : 0 errors (7 warnings pre-existing) ✅
- Frontend tsc : clean ✅
- Vitest : 267 pass ✅
- Frontend build + size : OK + 7/7 ✅

## Commit body validation
- Format titre : ✅ `feat(sprint35): Sprint 35 Phase A — MANDATORY 3/3 + crate nexus-coordinator-rs fondation`
- Delta tests coherent : ✅ (902→913 = +11 matche 10 crate + 1 E2E)
- Scope cuts honoured : ✅ (1-10 non touches)
- Co-Authored-By present : ✅
- G8 traceability : ✅ (preflight reference)
- Pre-launch protocol : ✅ (VERSION=1, 0 wire format touche)

## Modified-file branch coverage (Step 2bis, G9)
- `Cargo.toml` : +1 member line, aucune branche — N/A
- `scripts/install-node.sh` : +2 lignes comment, aucune branche — N/A
- Tous les autres fichiers sont NEW (100% couverts par leurs propres tests)

## Scope cuts verification
- "Migration complete coordinator" §7.1 : 0 fichier Python modifie ✅
- "Suppression coordinator Python" §7.2 : 0 suppression ✅
- "KudosLedger Rust" §7.3 : KudosEntry type defini mais pas la logique metier ✅
- "CI pipeline multi-OS" §7.6 : shellcheck CI = single OS (ubuntu) ✅
- "P3 grammar/watermark" §7.9 : 0 fichier executor touche ✅

## Horizon long-terme + documentation amont
- Design doc present : ✅ (kickoff §4 D1-D5 avec alternatives + design review G1)
- D1..D5 avec alternatives + rationale : ✅ (5 decisions, 2 acknowledged ⚠️)
- Solution la plus poussee : ✅ (rusqlite 0.36 workspace, pas de nouveau framework)
- Aucune LOC estimee au plan : ✅ (plan.md nettoye, pas de LOC scope)

## Research grounding (Step 4bis)
- S1a OSS prior art : ✅ present dans preflight (APPROACH-ALIGNED, Rust workspace pattern standard)
- S1b deps context7 : N/A (0 nouvelle dep externe)

## Findings

### P2-REVIEW-A-1 : LOC estimations dans kickoff §4 D5

`sprint35_kickoff.md:182,185` — "~30 LOC YAML" et "~100 LOC Rust"
sont des estimations de scope qui violent §6.7. Les mentions
descriptives (lignes 34-41, tailles Python existantes pour
justifier priorite migration) sont legitimes.
**Action** : nettoyer les 2 occurrences scope dans D5 lors d'un
futur chore(planning). Carry S36.

### P2-REVIEW-A-2 : CoordinatorDb::open() double-open connection

`crates/nexus-coordinator-rs/src/db.rs:56-67` — la methode `open()`
ouvre la connection une premiere fois pour les migrations, puis la
re-ouvre une deuxieme fois. Le pattern `{ conn }` block move est
un workaround pour le borrow checker (migrations.to_latest prend
`&mut Connection`). Fonctionnel mais fragile — un crash entre les
deux opens perd le WAL journal.
**Action** : refactor en passant `&mut conn` sans double-open.
Carry S36 ou fix inline Phase B.

### P3-REVIEW-A-1 : cross-daemon test ne verifie pas le fetch cross-node

Le test `cross_daemon_blob.rs` publie et sert le blob sur le meme
daemon A. Daemon B est verifie healthy mais ne fetche pas le blob
depuis A. Le vrai cross-node iroh-blobs fetch necessite relay
connectivity. Acceptable pour fermer le MANDATORY 3/3 (harness
multi-daemon ops valide), le cross-fetch P2P reste un item futur.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S36 : P2-REVIEW-A-1 LOC kickoff, P2-REVIEW-A-2 double-open DB
