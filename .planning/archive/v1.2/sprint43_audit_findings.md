# Sprint 43 — Audit findings

**Auditeur** : session fraiche (indépendante de la session S43).
**Tip d'entrée** : `16d5fa4` (HEAD, post wrap-up S43).
**Diff audité** : `e1f7f00..0ec0458` (3 feat commits S43).
**Documents source** : sprint43_kickoff.md, sprint43_plan.md,
sprint43_verification.md, sprint44_audit_plan.md.
**Date** : 2026-04-30.

---

## Verdict : PASS

0 P0 / 0 P1 / 1 P2 / 2 P3. G4 rigor satisfait (1 P2 documente
avec evidence). S44 Phase A peut demarrer directement.

---

## Track A — MANDATORY batch (Phase A) : 7/7 PASS

| Item | Resultat | Evidence |
|---|---|---|
| A-1 conn() pub(crate) | PASS | `db.rs:306` → `pub(crate) fn conn` confirme |
| A-2 persist tracing::warn | PASS | `canary_registry.rs:158-160,170-172` → `if let Err(e) = self.persist() { tracing::warn!(...) }` |
| A-3 Mutex consolidation | PASS | `canary_input.rs:366-368` ReloadState struct 3 champs + `:388` `reload: Mutex<ReloadState>` |
| A-4 rerun blake3 | PASS | `rerun.rs:78` `blake3::hash(s.as_bytes())`, 0 DefaultHasher, test `simple_hash_deterministic` :124 |
| A-5 MintRequest::new() | PASS | `invite.rs:38-39` impl + `:174` `mk_req` utilise `MintRequest::new(...)` |
| A-6 URL single-quote | PASS | grep `'https?://` sur crates/ = 0 match |
| A-7 LOC kickoff | PASS | grep `LOC estim` sur plan.md = 0 match |

## Track B — Files + consent API (Phase B) : 5/5 PASS

| Item | Resultat | Evidence |
|---|---|---|
| B-1 consent.rs 4 routes | PASS | get `:147`, set `:153`, whitelist_add `:179`, whitelist_remove `:210`. Validation hex 64 `:143-145`. Atomic tmp+rename `:137-139`. Threat notes `:81-111` |
| B-2 files.rs CAS SHA-256 | PASS | `sha2::Sha256` `:79-84`. 3 routes upload `:46`, manifest `:119`, stream `:137`. validate_sha256 hex 64 `:34-36`. MAX_UPLOAD_BYTES 50MB `:20` |
| B-3 header injection | PASS | content_type CRLF filter `:68`, original_name CRLF+quote filter `:76` |
| B-4 routes http.rs | PASS | 4 consent `:297-305` + 3 files `:307-312` = 7 routes enregistrees |
| B-5 sha2 dep | PASS | `Cargo.toml:64` `sha2 = { workspace = true }` |

## Track C — Canary + contributor API (Phase C) : 5/5 PASS

| Item | Resultat | Evidence |
|---|---|---|
| C-1 canary_api.rs 2 handlers | PASS | set_inject_rate `:27`, observed_divergence `:60`. Option 503 check `:31-34` et `:64-67` |
| C-2 contributor_api.rs 3 handlers | PASS | verify `:35`, list `:73`, envelope `:106`. ContributorRegistry via `state.coordinator_db.lock()` |
| C-3 proxy supprime | PASS | `proxy_contributor_verify` supprime (diff -84 lignes), `is_64_lowercase_hex` supprime. Seule mention dans docstring `:3` |
| C-4 canary_input delegates | PASS | Debug impl `:372`, set_inject_rate `:254+471`, recent_divergences `:342+475` |
| C-5 #[allow(dead_code)] | PASS | coord_http_client `http.rs:140`, coord_base_url `http.rs:147`. Raison documentee (proxy supprime, cleanup S45) |

## Track D — Process / meta : 3/3 PASS

| Item | Resultat | Evidence |
|---|---|---|
| D-1 G8 preflights | PASS | 3/3 EXECUTE : Phase A (`sprint43_phase_A_preflight.md`), Phase B (`...B...`), Phase C (`...C...`) |
| D-2 scope cuts 6/6 | PASS | diff --stat `e1f7f00..0ec0458` = 14 fichiers, tous dans nexus-coordinator-rs + nexus-shell-daemon. 0 match scope cuts (health, shell, tasks, kudos, Python, CI, middleware, background loops) |
| D-3 7/7 MANDATORY | PASS | Phase A commit `130db9b` couvre les 7 items per diff |

## Track E — Doc coherence : 5/5 PASS

| Item | Resultat | Evidence |
|---|---|---|
| E-1 HARDENING_ROADMAP | PASS | `last_validated: 2026-04-30` mentionne 1111 Rust / ~2114 total |
| E-2 CLAUDE.md | PASS | `:124` "Sprints 0-43 CLOSED" |
| E-3 SPRINT_LOG.md | PASS | S43 row presente |
| E-4 review files 3/3 | PASS | phase_A_review, phase_B_review, phase_C_review dans active/ |
| E-5 preflight files 3/3 | PASS | phase_A_preflight, phase_B_preflight, phase_C_preflight dans active/ |

---

## Findings

### P2-AUDIT-A-1-S43 — Gap test integration plus large que carry documente

Le carry P3-REVIEW-B-1-S43 dit "tests HTTP integration manquants
pour les 7 nouvelles routes consent+files". En realite, **12 routes
sur 12** nouvelles manquent de tests HTTP integration a travers le
router axum :

- consent : 4 routes (0 integration test)
- files : 3 routes (0 integration test)
- canary_api : 2 routes (0 integration test, et `canary_input: None`
  dans `mk_state()` rend le test impossible sans refactor du harness)
- contributor_api : 3 routes (1 test validation rejection herite du
  proxy, 0 happy-path)

Impact : le carry sous-estime le scope du gap (7 vs 12). Le planning
S44 doit allouer l'effort pour les 12 routes, pas 7. Le harness de
test daemon (`mk_state()`) doit etre enrichi avec un
`canary_input: Some(...)` pour pouvoir couvrir les routes canary.

### P3-AUDIT-A-2-S43 — Silent null fallback canary_api

`canary_api.rs:72` : `serde_json::to_value(r).unwrap_or_default()`
produit `Value::Null` si la serialisation echoue. DivergenceRecord
devrait toujours serialiser (Serialize derive), donc le risque est
theorique. Mais le pattern masque les erreurs plutot que de les
reporter — un client pourrait recevoir `[null]` sans explication.

### P3-AUDIT-A-3-S43 — Changement case-sensitivity validation hex

La suppression de `is_64_lowercase_hex` (lowercase-only) au profit
de `validate_hex` (case-insensitive via `is_ascii_hexdigit()`) dans
contributor_api.rs change le comportement : le endpoint `/api/
contributor/verify/` accepte maintenant `AABB...` (uppercase) la ou
l'ancien proxy renvoyait 400. Si le DB stocke en lowercase, un query
uppercase retourne `verified: false` (faux negatif) au lieu de 400.
Non-bloquant, comportement plus permissif est preferable, mais a
documenter.

---

## Carries actualises pour S44

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 9+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 5/3 | exemption dep S45 |
| P2-REVIEW-A-1-S42 ChainResult mutations target | 2/3 | |
| P2-REVIEW-B-1-S42 pow_keypair identity doc | 2/3 | |
| P2-REVIEW-B-1-S43 coord dead_code cleanup | 1/3 | |
| **P2-AUDIT-A-1-S43 integration test gap 12 routes** | **1/3** | **NEW** |
| P3-REVIEW-A-2-S42 babel-scraper untracked | 2/3 | |
| P3-REVIEW-C-1-S42 list_apps aggregate probe | 2/3 | |
| P3-AUDIT-A-1-S42 couverture RNG rate>1 | 2/3 | |
| P3-AUDIT-C-1-S42 Debug vs serde | 2/3 | |
| P3-AUDIT-C-2-S42 pagination limit/offset | 2/3 | |
| P3-REVIEW-B-1-S43 tests HTTP integration | 1/3 | subsume par P2-AUDIT-A-1-S43 |
| P3-REVIEW-C-1-S43 prefix route contributor | 1/3 | |
| P3-REVIEW-A-1-S43 TOCTOU canary reload | 1/3 | |
| P3-AUDIT-A-2-S43 silent null canary_api | 1/3 | NEW |
| P3-AUDIT-A-3-S43 hex case-sensitivity | 1/3 | NEW |

**Note** : P3-REVIEW-B-1-S43 est subsume par le nouveau
P2-AUDIT-A-1-S43 (scope plus large). Le compteur P2 demarre a 1/3.

---

## Resolus S43 (confirmes)

- P2-REVIEW-A-1-S41 conn() pub(crate) : `db.rs:306` confirme
- P3-REVIEW-A-2-S39 LOC kickoff : 0 estimation dans plan
- P3-REVIEW-B-2-S39 persist error : tracing::warn confirme
- P3-AUDIT-A-1-S39 URL single-quote : 0 instance
- P3-REVIEW-B-1-S40 Manager Mutex : ReloadState confirme
- P3-REVIEW-C-1-S40 rerun hash : blake3 confirme
- P3-REVIEW-B-1-S41 MintRequest : new() confirme
