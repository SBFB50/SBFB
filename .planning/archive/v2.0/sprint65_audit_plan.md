# Sprint 65 — Audit plan

**Sprint audite** : Sprint 64 (hardening public cible).
**Tip de reference** : sera le tip apres commit Phase E.
**Auditeur** : session fraiche, independante du sprint 64.

---

## §1 Perimetre audit

Sprint 64 a livre 4 phases code (A-D) + 4 fix inter-phases + 1
phase doc (E) sur le theme "hardening public cible" (Sprint 4/6
roadmap v2.0 Public Verifiable Protocol Feed).

### Phases livrees
- **Phase A** : MANDATORY F1 VERSION-NOT-STORED 3/3 (M13 `app_version`) + F5 IROH-INFRA-TIMEOUT code (timeout/retry/JoinHandle)
- **Phase B** : Dette pair obligatoire (5 items P2 CLOSED : joinhandle, backfill, orphan rollback, stream-break, process-format)
- **Phase C** : 6 tests adversariaux feed (fork-bomb, oversized, bad URL, bad hash, seq gap, cross-author forgery)
- **Phase D** : 4 tests adversariaux crypto (Ed25519 forgery, BLAKE3 tamper, PoW nonce, future timestamp) + 1 E2E nouveau noeud
- **Phase E** : PUBLIC_FEED_SPEC.md §10-12, verification.md, audit_plan S65, CLAUDE.md, SPRINT_LOG.md
- **Fix** : rate limiter per-author + split local/remote (`1f355b6`), tail-safe orphan rollback (`490e491`), Phase D cross-review 4 P1 (`21bc315`), E2E feed_status fix (`a67c1a7`)

### Fichiers principaux touches
- `crates/nexus-coordinator-rs/src/db.rs` (M13 migration)
- `crates/nexus-coordinator-rs/src/public_feed.rs` (10 tests adversariaux + rate limiter split + tail-safe)
- `crates/nexus-coordinator-rs/tests/multi_daemon.rs` (E2E nouveau noeud)
- `crates/nexus-shell-daemon/src/deploy.rs` (version insert)
- `crates/nexus-shell-daemon/src/http.rs` (provenance endpoint + feed status)
- `crates/nexus-shell-daemon/src/feed_sync.rs` (timeout + retry + reconnect)
- `crates/nexus-shell-daemon/src/runtime.rs` (JoinHandle tracked)
- `docs/protocol/PUBLIC_FEED_SPEC.md` (§10-12)
- `docs/claude/README.md` (§6.7 exemption LOC)

---

## §2 Tracks d'audit

### Track 1 — Tests adversariaux completude
- 10 tests adversariaux S64 (6 feed Phase C + 4 crypto Phase D) : verifier que chaque test couvre un vecteur d'attaque distinct et non-redondant avec les 5 tests pre-existants
- Fork-bomb : rate limiter rejette correctement au-dela du quota (5/min)
- Oversized : payload > MAX_OPERATION_JSON_SIZE rejete
- Bad URL/hash : validation stricte (HTTPS, hex-64, hex-40)
- Seq gap : verify_chain detecte broken linkage
- Cross-author : signature mismatch detecte
- Ed25519 forgery : random bytes rejete par verify_entry
- BLAKE3 tamper : 1 bit flip detecte entry_hash mismatch
- PoW nonce : random nonces echouent avec probabilite ecrasante
- Future timestamp : > 30 jours rejete

### Track 2 — MANDATORY 3/3 resolution
- F1 P2-VERSION-NOT-STORED : M13 `app_version TEXT` present, insert depuis SBFB.json, retourne dans endpoint
- F5 P2-IROH-INFRA-TIMEOUT : timeout 30s sur subscribe, retry backoff, JoinHandle joined at shutdown, E2E proof (test_new_node_full_sync_and_verify)

### Track 3 — Dette pair integrity
- 5 items P2 fermes Phase B : chacun a une preuve/test correspondante
- Tail-safe orphan rollback : DELETE refuse si entry chainee (fix `490e491`)
- Rate limiter split local/remote : local exempt, remote rate-limited (fix `1f355b6`)

### Track 4 — Nouveau noeud E2E
- `test_new_node_full_sync_and_verify` : daemon neuf → join ticket → sync 3 entries → poll feed_status → verify count + last_seq
- Gate SBFB_INTEGRATION=1 operationnel
- Timeout 60s avec retry polling 500ms

### Track 5 — PUBLIC_FEED_SPEC.md coherence
- §10 : 15 vecteurs documentes avec reference test (pas d'invention)
- §11 : algorithme bootstrap coherent avec le code multi_daemon.rs
- §12 : threat model coherent avec THREAT_MODEL.md, residual risks complets
- Pas de drift spec vs code

### Track 6 — Process compliance
- 5 preflights G8 presents (A-E) avec verdicts documentes
- 4 reviews PASS (A-D) + E review au pre-commit
- Commit discipline : feat/fix/docs scope + body riche + delta tests
- 12/12 scope cuts respectes
- Sprint pair : phase dette Phase B presente (§6.2.1 Regle 1)

### Track 7 — Carries S65

| Item | Compteur | Owner | Trigger | Exit condition |
|---|---|---|---|---|
| **P2-FEED-INSERT-NO-AUTH-TIER** | **3/3 MANDATORY** | planner S65 | §6.2.1 Regle 2 | feed_insert handler verifie auth tier avant insert |
| P2-A-1 rand blocker | exemption externe | upstream | rand 0.9 | crate rand 0.9 stable |
| P2-AUDIT-2 iroh transitives | exemption externe | upstream | iroh 1.0 | iroh 1.0 stable + upgrade sprint |
| P2-G-1 exe lock | monitoring | dev-env | reproductible 3x | root cause + fix |
| P2-PROVENANCE-404-BRIDGE | 2/3 | planner S65+ | UX enrichissement | code distinct 404 projet/provenance |
| P2-BADGE-WORDING-PREMATURE | pre-existant S14 | planner S65 | verification live | badge conditionne sur verified |
| P2-COMMIT-TITLE-FORMAT | 2/3 | planner S65 | process | PROCESS.md clarification |
| P2-REVIEW-ORDER | 2/3 | planner S65 | process | README.md clarification |
| P2-PYTHON-BLOCK-EXEMPTION | 2/3 | planner S65 | SKILL.md | Step 2 clause exemption |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | planner S65+ | hardening | escapeAttr single quote |
| P2-PLAYWRIGHT-SPECS-STALE | 2/3 | planner S65 | test maintenance | specs Playwright reecrites |
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | planner S65+ | cross-node | pkarr pubkey resolution |
| P2-COVERAGE-DEPLOY-E2E | 2/3 | planner S65+ | test coverage | deploy roundtrip E2E |
| P2-FEED-JOIN-HANDLE-LEAK | 1/3 | planner S65 | feed reconnect | shutdown channel + reconnect loop |
| P2-VERIFY-ENTRY-VERSION-GUARD | 1/3 | planner S65 | version policy | verify_entry checks version pre-launch |
| P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 | planner S65 | feed resilience | republish DB→iroh-docs |

---

## §3 Compteurs attendus

| Suite | S64 entry | S64 exit | S65 baseline |
|---|---|---|---|
| Rust nextest | 1305 | 1326 | 1326 |
| Vitest | 265 | 265 | 265 |
| size-limit | 6/6 | 6/6 | 6/6 |
| Total | ~1576 | ~1597 | ~1597 |

---

## §4 Verdict attendu

L'auditeur verifie les 7 tracks ci-dessus. Verdict PASS si :
- 0 P0 (regression securite, crash prod, data loss)
- 0 P1 (bug fonctionnel confirmable avec commande reproductible)
- >= 1 P2 documente (rigor signal G4)

Findings P0/P1 = fix bloquant avant ouverture Sprint 65.
Findings P2/P3 = carry-over Sprint 65+.
