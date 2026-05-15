# Sprint 64 — Audit plan

**Sprint audite** : Sprint 63 (verification tiers + UX).
**Tip de reference** : `<Phase D commit SHA>` (a completer).
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
- 3 reviews PASS + 1 review Phase D (a verifier)
- Commit discipline : feat scope + body riche + delta tests
- Scope cuts respectes (10 items kickoff §7)

### Track 6 — Carries S64

Items a verifier :
- F1 P2-VERSION-NOT-STORED **3/3 MANDATORY** : plan obligatoire S64
- F5 P2-IROH-INFRA-TIMEOUT **3/3 MANDATORY** : plan obligatoire S64
- 6 P2 cosmetic/process carries (PROCESS-FORMAT, PROVENANCE-404-BRIDGE,
  BADGE-WORDING-PREMATURE, COMMIT-TITLE-FORMAT, REVIEW-ORDER,
  PYTHON-BLOCK-EXEMPTION)
- 5 carries reconduits 2/3 (FEED-INSERT-NO-AUTH-TIER,
  FEED-SUBSCRIBE-JOINHANDLE, BACKFILL-6PLUS-TEST,
  FEED-PUBLISH-ORPHAN, SUBSCRIBE-STREAM-BREAK)
- 3 carries permanents (P2-A-1 rand, P2-AUDIT-2 iroh, P2-G-1 exe lock)

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
