# S3 — Audit: `verification.rs` (Sprint 2)

**Auditeur**: Agent S3 — 2026-04-10  
**Fichier Rust**: `crates/nexus-core-rs/src/verification.rs`  
**Référence Python**: `nexus/compute/verification.py`

---

## Conforme

- Layer 1 fail → `trust_delta = -50`, `ban = true`. Rust L172–179 ✓ vs Python L230–232 ✓  
- Layer 2 fail → `trust_delta = -50`, `ban = true`. Rust L194–201 ✓ vs Python L240–242 ✓  
- Layer 3 fail → `trust_delta = -5`, `ban = false`. Rust L224 ✓ vs Python L250 ✓  
- `spot_check_rate` tiers: `>= 80 → 1%`, `>= 50 → 5%`, `else → 20%`. Rust L250–256 ✓ vs Python L259–264 ✓ — seuils inclusifs, identiques.  
- Early return on L1 failure skips L2/L3. Rust ✓ matches Python ✓  
- Empty whitelist → skip (pass), not fail. Both consistent.

## Manquant

- Python Layer 3 uses KL-divergence / max-diff on raw float logprob dicts (Python L126–169). Rust replaces with BLAKE3 hash equality. Documented intentionally (Rust doc comment L31–36) — v1.2 concern, not a port gap.  
- Python `spot_check_needed` calls `random.random() < rate`. Rust `spot_check_rate()` returns the rate only; the caller must do the random draw. Minor API delta, not a bug.

## Déviations

- **Layer 3 algorithm change (intentional)**: Python compares `dict[str, float]` with tolerance threshold `0.5`. Rust compares `[u8; 32]` BLAKE3 hashes. Behaviour differs but change is documented and deliberate.  
- **Layer 3 result when fail**: Python returns `{"passed": True, ...}` (L251) — counterintuitively `passed=true` on logprob fail. Rust sets `passed_overall = false` when `logprobs.status == Failed` (L229). **Semantic divergence**: Python L251 vs Rust L229. Not flagged as bug per plan (Rust behavior is more correct), but the Python contract is broken.

## Qualité

8 unit tests, full branch coverage of all 3 layers, boundary conditions for spot_check_rate (79/80, 49/50). Code is clean, well-documented, early-exit pattern matches Python control flow.

## Tests

```
cargo test -p nexus-core-rs --lib verification
running 8 tests — 8 passed, 0 failed
```

## Bugs (DO NOT FIX)

**BUG-S3-01 (LOW)** — `passed` field semantics differ on Layer 3 failure:  
- Python `nexus/compute/verification.py:251` returns `"passed": True` when logprob check fails (only `trust_delta = -5` signals the issue).  
- Rust `verification.rs:229` sets `passed_overall = false` when logprobs status is `Failed`.  
This is a contract break vs Python — callers relying on `passed=True` to mean "not banworthy, keep dispatching" will see `passed=False` from Rust and may incorrectly halt dispatch. Rust behavior is semantically sounder but is not a faithful port on this point.
