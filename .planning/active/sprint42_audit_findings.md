# Sprint 42 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S42).
**Tip d'entree** : `92ec12b` (S42 Phase D wrap-up).
**Tip audite** : `87ee663` (S42 Phase C, dernier feat commit).
**Date** : 2026-04-30.

## Verdict global : PASS

0 P0, 0 P1, 1 P2 (informational), 3 P3 (informational).
S43 Phase A peut demarrer directement.

---

## Track A — Dette pair (Phase A)

| Item | Verdict | Evidence |
|---|---|---|
| A-1 rand::thread_rng canary_input.rs | PASS | gen_range(1..=rate) L230, tests injector_rate_always L739 + guardrail_tripwire_on_inject L840 |
| A-2 rand::thread_rng upload_queue.rs | PASS | gen::\<f64\>() L107, test jitter_in_range L178 (20 iter, bornes 0..300) |
| A-3 Mutation variant exhaustif | PASS | 4 arms match L60-96, test chain_mutation_collects_and_passes L266 |
| A-4 warn threshold PATTERNS.md | PASS | §P41 L2215, coherent avec canary_registry.rs:15-16 (30/45j) |
| A-5 Mutation non emis (scope cut) | PASS | 3 impls Guardrail (Output/Pii/Canary) : aucune n'emet Mutation. Carry S43+ |

**P3-AUDIT-A-1-S42** : couverture path RNG rate>1 indirecte (test
injector_rate_always prend le shortcut rate<=1, lib `rand` fiable).

## Track B — Deploy API (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| B-1 forge.rs 5 ForgeType variants | PASS | 8 tests L106-138, 5/5 variants couverts |
| B-2 provenance roundtrip | PASS | 4 tests L127-186 (generate+verify, wrong key, tampered, BLAKE3 deterministic) |
| B-3 deploy.rs validations | PASS | 9 tests L602-678 (SHA hex, zip validate, zip create, dir_size, SBFB.json) |
| B-4 pow_keypair provenance | PASS | Meme pattern que Python coord.keypair. Carry S43+ doc |
| B-5 routes http.rs | PASS | POST /api/v1/deploy + POST /api/v1/deploy-from-repo L279-283 |
| Security | PASS | 0 unwrap prod, path traversal filtre L522-530, bornes tailles (100/500 MB), timeouts git (30/10s) |
| Tests delta | PASS | +21 annonce, +21 compte (forge 8 + provenance 4 + deploy 9) |

0 findings.

## Track C — Apps API (Phase C)

| Item | Verdict | Evidence |
|---|---|---|
| C-1 to_summary/to_detail fields | PASS | apps.rs:73-105, 10 champs summary / 14 champs detail, 4 tests L190-250 |
| C-2 BrowseStatus+BrowseSource coverage | PASS | Debug+to_lowercase couvre tous variants, tests L232-242 |
| C-3 query defaults + filters | PASS | 8 tests L166-275, category+open_source filtres |
| C-4 aggregate() cache TTL | PASS | browse.rs:341 cached(), probe_ttl 60s, 3 tests cache L788-802 |

**P3-AUDIT-C-1-S42** : status_str/source_str utilisent Debug trait
au lieu de serde — risque divergence si futur variant avec
serde(rename). Pas actionable pre-v1.0.

**P3-AUDIT-C-2-S42** : AppListQuery ne supporte pas limit/offset
(pagination). Acceptable pre-v1.0 (nombre d'apps faible).

## Track D — Process / meta

| Item | Verdict | Evidence |
|---|---|---|
| D-1 G8 preflights 3/3 | PASS | 3 fichiers presents, 3/3 EXECUTE plan-as-is |
| D-2 scope cuts 8/8 | PASS | 0 scope leak dans diff d6f8191..87ee663 |
| D-3 4/4 dette resolus | PASS | rand_range + pseudo_random + Mutation + warn threshold dans Phase A |

## Track E — Overdue items

| Item | Verdict | Evidence |
|---|---|---|
| E-1 5 P3 a 4/3 | **P2** | Conditionnels "si temps" au kickoff L129/169-171. Budget Phase A consomme par 4 P2. MANDATORY S43 |
| E-2 conn() pub 2/3→3/3 | PASS | Pas encore MANDATORY a l'entree S42 (D1 ciblait 4 P2 plus anciens) |
| E-3 MintRequest 2/3→3/3 | PASS | Idem — montee mecanique compteur, MANDATORY S43 |

**P2-AUDIT-E-1-S42** : 5 items P3 (LOC kickoff, persist error, URL
single-quote, Manager Mutex, rerun hash) a 4/3 OVERDUE. Le kickoff
les marquait "si temps" et Phase A n'avait pas le budget. Pas de
signal P1 (aucun < 50 LOC confirme). Tous MANDATORY S43.

## Track F — Doc coherence

| Item | Verdict | Evidence |
|---|---|---|
| F-1 HARDENING_ROADMAP compteurs | PASS | 1089 Rust / ~2092 total (L3, last_validated 2026-04-30) |
| F-2 CLAUDE.md etat actuel | PASS | S42 CLOSED + carries S43 (L124-126) |
| F-3 SPRINT_LOG.md S42 row | PASS | Presente L19, 3 phases, tip 87ee663 |
| F-4 Phase reviews 3/3 | PASS | A (73L) + B (54L) + C (50L) dans active/ |
| F-5 Phase preflights 3/3 | PASS | A (25L) + B (26L) + C (25L) dans active/ |

---

## Resume findings

| # | Severite | Track | Description | Action |
|---|---|---|---|---|
| P2-AUDIT-E-1-S42 | P2 | E | 5 P3 OVERDUE 4/3 non resolus malgre MANDATORY | MANDATORY S43, budget dette |
| P3-AUDIT-A-1-S42 | P3 | A | Couverture path RNG rate>1 indirecte | Informational |
| P3-AUDIT-C-1-S42 | P3 | C | Debug vs serde pour status/source str | Informational post-v1.0 |
| P3-AUDIT-C-2-S42 | P3 | C | Pas de pagination limit/offset apps | Informational post-v1.0 |

## Carries confirmes S43

Voir `sprint43_audit_plan.md` §Carries S43 (14 items). L'audit
confirme les compteurs et ajoute 1 P2 + 3 P3 informationnels.

## Verdict

**PASS** — 0 P0, 0 P1. S43 Phase A demarre sans fix prealable.
