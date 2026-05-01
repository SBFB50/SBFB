# Sprint 45 — Audit findings (Phase 0 gate S46)

**Auditeur** : session fraiche independante (pas la session S45).
**Tip audite** : `e1c31a5` (S45 Phase B, dernier feat commit).
**HEAD actuel** : `bbb126f` (post chore wrap-up).
**Date** : 2026-04-30.
**Methode** : 4 agents paralleles (1 par groupe de tracks),
spot-check securite supplementaire sur code nouveau.

---

## Verdict : PASS

0 P0, 0 P1, 1 P2 confirme (carry existant), 1 P3 nouveau.
G4 rigor : >=1 P2+ documente (A-4 confirme). S46 Phase A
peut demarrer sans fix prealable.

---

## Track A — Route portage (Phase A)

| Item | Verdict | Evidence |
|---|---|---|
| A-1 invite_api.rs 3 routes | ok | create/list/revoke, scope worker/observer, expiry>=60s |
| A-2 quarantine_api.rs 3 routes | ok | list/flush/drop, status pending/all, TTL 900s |
| A-3 http.rs 6 routes registrees | ok | L318-338 sous authed_routes, mod declarations main.rs |
| A-4 invite ID collision | P2 carry confirme | AtomicU64 `inv-{epoch}-{seq}`, carry P2-REVIEW-A-2-S45 1/3 |

Spot-check securite : 0 unwrap en prod, queries paramétrisées,
0 format! SQL. unwrap() uniquement dans #[cfg(test)].

**0 nouveau finding.**

## Track B — Carries resolus (Phase A)

| Item | Verdict | Evidence |
|---|---|---|
| B-1 SHA-256 → BLAKE3 | ok | redundancy.rs:24 blake3::hash(), 0 sha2 dans le fichier |
| B-2 tokio::fs | ok | worker_state_api.rs:35 tokio::fs::read_to_string |
| B-3 list_tasks status | ok | tasks_api.rs:49-63 VALID_STATES + 400 sur invalide |
| B-4 TOCTOU canary | ok + P3 | mtime sous lock, fenetre microseconde, mitigee Ed25519 |
| B-5 silent null diagnostic | ok | diagnostic_api.rs:34-42 Err → 500 |
| B-6 hex case-sensitivity | ok | contributor_api.rs:44-45,83,118-119 to_ascii_lowercase() |

**1 P3 nouveau (B-4).** Fenetre TOCTOU entre store mtime et
read_to_string — le prochain poll raterait une modification
intercalee. Risque theorique, mitige par debounce 50ms +
verification signature Ed25519 du canary set. Acceptable pre-v1.0.

## Track C — Coordinator Python gut (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| C-1 14 routes Python supprimees | ok | api/ ne contient que events.py, daemon.py, app.py, __init__.py |
| C-2 app.py routing | ok | L121-122 events_router + daemon_router uniquement |
| C-3 12 tests Python supprimes | ok | 0 fichier test correspondant aux routes supprimees |
| C-4 imports non casses | ok | grep 0 match import routes supprimees dans tests/ |
| C-5 coordinator.py boot | ok | L56-82 imports modules non-supprimes (scope cut S46-47) |
| C-6 coord pytest 323+23f+6s | ok | 38 fichiers test restants, coherent |

**0 finding.**

## Track D — Dead code Rust (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| D-1 coord_http_client | ok | grep 0 match dans crate |
| D-2 coord_base_url | ok | grep 0 match |
| D-3 resolve_coord_base_url() | ok | grep 0 match |
| D-4 COORD_BASE_URL_ENV + DEFAULT | ok | grep 0 match |
| D-5 test env_var | ok | grep 0 match |
| D-6 reqwest dep vivante | ok | deploy.rs:383-389 utilise reqwest::Client |

**0 finding.**

## Track E — Process / meta

| Item | Verdict | Evidence |
|---|---|---|
| E-1 G8 preflights 2/2 | ok | Phase A EXECUTE `12eee9c`, Phase B EXECUTE `5c4479f` |
| E-2 scope cuts 8/8 | ok | 8 listes kickoff §7, 0 violation dans diff |
| E-3 7 carries resolus | ok | 7/7 verifies dans le code actuel |
| E-4 G1 design review | ok | sprint45_design_review.md D1..D4 tous VERIFIED |
| E-5 sprint impair | ok | pas de phase dette, correct |

**0 finding.**

## Track F — Doc coherence

| Item | Verdict | Evidence |
|---|---|---|
| F-1 CLAUDE.md etat actuel | ok | S45 CLOSED, ~1948 total, 1132 Rust |
| F-2 SPRINT_LOG.md row S45 | ok | row presente, theme correct |
| F-3 HARDENING_ROADMAP.md | ok | last_validated 2026-04-30, compteurs coherents |
| F-4 review files 2/2 | ok | A + B presents dans active/ |
| F-5 preflight files 2/2 | ok | A + B presents dans active/ |

**0 finding.**

---

## Synthese findings

| Sev | Count | Items |
|---|---|---|
| P0 | 0 | — |
| P1 | 0 | — |
| P2 | 1 (carry confirme) | A-4 invite ID collision multi-daemon (existant, 1/3) |
| P3 | 1 (nouveau) | B-4 TOCTOU canary reload fenetre microseconde |

## Carries confirmes S46

Cf. sprint46_audit_plan.md §Carries S46 — 11 items, dont
P2-AUDIT-A-1-S43 integration test gap 3/3 **MANDATORY**.

## Recommendation

Commit autorise. 0 fix requis. S46 Phase A peut demarrer.
