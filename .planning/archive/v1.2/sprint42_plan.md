# Sprint 42 — Plan d'execution

**Tip** : `7edf04b`. **Theme** : dette pair + Tier 5 routes API.

---

## §1 Etat verifie

Rust 1059, ~2062 total. fmt/clippy/nextest PASS.

## §2 D1..D5 rappel

- D1 : dette 4 items P2 Phase A
- D2 : Tier 5 deploy+apps Phase B+C
- D3 : scope cuts 8 items

## §3 Research

- axum handlers : pattern etabli http.rs (30+ handlers existants)
- verified deploy : `sprint14_keyoxide_decision.md` + code Python
- rand crate : dep workspace, `thread_rng().gen_range()`

---

## Phase A — Dette pair (4 items P2)

### §A.1 Scope

(a) rand_range → rand crate (canary_input.rs L265-283)
(b) pseudo_random_f64 → rand crate (upload_queue.rs L120-131)
(c) GuardrailOutcome +Mutation variant (guardrails.rs)
(d) warn threshold doc (PATTERNS.md)

### §A.2 Fichiers

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/canary_input.rs` | rand_range → rand |
| `crates/nexus-coordinator-rs/src/upload_queue.rs` | pseudo_random → rand |
| `crates/nexus-coordinator-rs/src/guardrails.rs` | +Mutation variant |
| `docs/rust/PATTERNS.md` | +§P41 warn threshold doc |

### §A.3 Tests

1. Existing tests continue to pass (rand behavior compatible)
2. New test: guardrail_mutation_outcome (verify Mutation variant)

### §A.5 Commit

```
feat(sprint42): Sprint 42 Phase A — dette pair P2 batch rand +
Mutation + warn threshold
```

---

## Phase B — api/deploy.py port (505 LOC)

### §B.1 Scope

Porter le verified deploy handler. POST /api/v1/deploy :
- Clone repo (git --depth 1)
- Verify Keyoxide Ed25519 (SBFB.json)
- Build zip archive
- Sign provenance.json SLSA L1
- Store blob via iroh-blobs

### §B.2 Fichiers

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/http.rs` | +deploy handler |

### §B.5 Commit

```
feat(sprint42): Sprint 42 Phase B — deploy API Rust
```

---

## Phase C — api/apps.py port (350 LOC)

### §C.1 Scope

Porter les app listing handlers.
- GET /api/v1/apps (list)
- GET /api/v1/apps/:id (detail)

### §C.2 Fichiers

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/http.rs` | +apps handlers |

### §C.5 Commit

```
feat(sprint42): Sprint 42 Phase C — apps API Rust
```

---

## §5 Fail-fast checklist

| # | Check | Observed |
|---|---|---|
| 1-15 | Standard suites (fmt/clippy/nextest/pytest/vitest/build) | |
| 16-21 | G8 preflights + reviews 3/3 | |
| 22 | Dette 4/4 P2 resolus | |
| 23 | deploy handler porte | |
| 24 | apps handlers portes | |
| 25 | Scope cuts 8/8 | |
| 26-28 | Delta tests Phase A/B/C | |

## §7 Scope cuts (copie §D3)

1-8 : cf. kickoff §D3
