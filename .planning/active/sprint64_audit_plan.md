# Sprint 64 — Audit plan

**Sprint audite** : Sprint 63 (verification tiers + UX).
**Tip de reference** : `7198ae5`.
**Auditeur** : session fraiche, independante du sprint 63.

---

## §1 Perimetre audit

Sprint 63 a livre 4 phases (A-D) + 2 fix inter-phases sur le theme
"verification tiers + UX" (3eme sprint roadmap post-v1.0).

### Phases livrees
- **Phase A** : MANDATORY 3/3 (IMAGE-DEP png swap + PLAYWRIGHT-REFACTOR daemon Rust)
- **Phase B** : Provenance endpoint HTTP + stockage SQLite M12
- **Phase C** : Bridge verification (3 methodes) + UI VerificationDetail modal
- **Phase D** : Protocol Explorer section verification + wrap-up
- **Fix** : provenance hash linkage (`fa7cd52`) + provenance insert ordering (`5f6a77d`)

### Fichiers principaux touches
- `crates/nexus-launcher/Cargo.toml` + `src/tray.rs` (image → png)
- `crates/nexus-coordinator-rs/src/db.rs` (M12 provenance_records)
- `crates/nexus-shell-daemon/src/deploy.rs` (insert provenance)
- `crates/nexus-shell-daemon/src/http.rs` (provenance + bridge + feed cursor handlers)
- `web/tests/global-setup.ts` (PLAYWRIGHT-REFACTOR)
- `web/public/sbfb-bridge.js` (3 methodes : provenance_get, provenance_verify, feed_cursor_get)
- `web/src/hooks/useBridge.ts` (3 dispatch cases)
- `web/src/components/VerificationDetail.tsx` (NEW)
- `web/src/pages/BrowsedProject.tsx` (badge cliquable → modal)
- `examples/sbfb-explorer/index.html` + `app.js` + `style.css` (section verification)

---

## §2 Tracks d'audit

### Track 1 — Provenance pipeline integrity
- M12 migration : schema correct, index present, UNIQUE constraint
- Insert au deploy : timing correct (apres blob store, avant annonce)
- Endpoint HTTP : 200 avec record, 404 sans, verification Ed25519 live
- Bridge relay : handlers delegent correctement aux fonctions existantes
- Hash linkage : provenance_hash propagee correctement annonce → frontend

### Track 2 — MANDATORY 3/3 resolution
- IMAGE-DEP : `image` crate absente de `cargo tree -p nexus-launcher -d`
- PLAYWRIGHT-REFACTOR : `global-setup.ts` spawn daemon Rust, pas Python
- Tests existants non regresses

### Track 3 — UI proof-chain coherence
- VerificationDetail : 7 champs affiches, lazy fetch au clic, verify live
- BrowsedProject : badge cliquable ouvre modal
- Design system : shadcn Dialog, pas de composant custom non-standard
- Responsive : mobile-friendly

### Track 4 — Protocol Explorer demo
- Section 6 "Verification & Provenance" : contenu factuel, pas marketing
- Verification interactive : select project → verify → resultat live
- Escaping HTML/attributs : pas de XSS via noms de projet

### Track 5 — Process compliance
- 4 preflights G8 presents avec verdicts documentes
- 4 reviews PASS (A, B, C, D)
- Commit discipline : feat scope + body riche + delta tests
- Scope cuts respectes (10 items kickoff §7)

### Track 6 — Carries S64

| Item | Compteur | Owner | Trigger | Exit condition |
|---|---|---|---|---|
| F1 P2-VERSION-NOT-STORED | **3/3 MANDATORY** | planner S64 | §6.2.1 Regle 2 | version stockee en DB a l'insert provenance |
| F5 P2-IROH-INFRA-TIMEOUT | **3/3 MANDATORY** | planner S64 | §6.2.1 Regle 2 | SBFB_INTEGRATION tests stables (0 timeout 5 runs consecutifs) |
| P2-PROCESS-FORMAT | herite | planner S64 | audit S63 | supprimer §6 LOC plan.md OU ajouter exemption retroactive |
| P2-PROVENANCE-404-BRIDGE | 1/3 | planner S64+ | enrichissement provenance UX | endpoint retourne code distinct projet-inconnu vs provenance-absente |
| P2-BADGE-WORDING-PREMATURE | pre-existant S14 | planner S64 | UI pass verification | renommer badge "Provenance disponible" ou conditionner sur verified |
| P2-COMMIT-TITLE-FORMAT | 1/3 | planner S64 | process clarification | PROCESS.md accepte domain scopes OU commits alignes feat(sprintN) |
| P2-REVIEW-ORDER | 1/3 | planner S64 | process clarification | README.md/PROCESS.md clarifie review artifact timing |
| P2-PYTHON-BLOCK-EXEMPTION | 1/3 | planner S64 | process hygiene | SKILL.md Step 2 ajoute clause exemption projets sans Python |
| P2-FEED-INSERT-NO-AUTH-TIER | 2/3 | planner S64+ | auth tier feed | feed_insert handler verifie auth tier avant insert |
| P2-FEED-SUBSCRIBE-JOINHANDLE | 2/3 | planner S64 | subscribe cleanup | subscribe JoinHandle trackee + joined au shutdown |
| P2-BACKFILL-6PLUS-TEST | 2/3 | planner S64 | test coverage | test integration backfill >= 6 entries present |
| P2-FEED-PUBLISH-ORPHAN | 2/3 | planner S64 | feed hardening | retry/rollback split DB/iroh-docs insert |
| P2-SUBSCRIBE-STREAM-BREAK | 2/3 | planner S64 | feed resilience | subscribe reconnexion auto apres stream break |
| P2-A-1 rand blocker | exemption permanente | upstream | rand 0.9 release | crate rand publie 0.9 stable |
| P2-AUDIT-2 iroh transitives | exemption externe | upstream | iroh 1.0 upgrade sprint | iroh 1.0 stable + upgrade sprint dedie |
| P2-G-1 exe lock | monitoring | dev-env | reproductible 3x consecutif | root cause identifiee + fix |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 1/3 | planner S64+ | defensive hardening | escapeAttr inclut single quote |
| P2-PLAYWRIGHT-SPECS-STALE | 1/3 | planner S64 | test maintenance | specs Playwright reecrites pour daemon Rust (Phase A = setup only, pas specs) |
| P2-VERIFY-LOCAL-KEY-ONLY | 1/3 | planner S64+ | cross-node verification | resoudre cle publique depuis pkarr/node_id pour verification cross-node |
| P2-COVERAGE-DEPLOY-E2E | 1/3 | planner S64+ | test coverage | test integration deploy E2E (clone+build+provenance roundtrip) |

---

## §3 Compteurs attendus

| Suite | S63 entry | S63 exit | S64 baseline |
|---|---|---|---|
| Rust nextest | 1299 | 1305 | 1305 |
| Vitest | 258 | 265 | 265 |
| size-limit | 6/6 | 6/6 | 6/6 |
| Total | ~1563 | ~1576 | ~1576 |

---

## §4 Verdict attendu

L'auditeur verifie les 6 tracks ci-dessus. Verdict PASS si :
- 0 P0 (regression securite, crash prod, data loss)
- 0 P1 (bug fonctionnel confirmable avec commande reproductible)
- >= 1 P2 documente (rigor signal G4)

Findings P0/P1 = fix bloquant avant ouverture Sprint 64.
Findings P2/P3 = carry-over Sprint 64+.
