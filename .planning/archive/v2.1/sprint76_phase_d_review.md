# Sprint 76 Phase D Review

## Verdict: PASS

> Driver-side deep review (Claude, 1M-context) → PASS-PENDING, then Codex
> `codex exec` (GPT-5.5) run and reconciled → promoted to PASS. Codex returned
> 0 P0/P1 GAP (PASS 4 / PARTIAL 2); both PARTIALs were resolved at the root
> (see `## Codex reconciliation`) and all suites re-run GREEN afterward. The
> driver review found 0 P0/P1, 2 P2, 2 P3 — above the G4 rigor floor (>= 1 P2+).
>
> **POST-CODEX CHANGE (recorded honestly):** the 3-node in-process E2E
> `quorum_redundancy_two_stubworkers_byte_identical` was REMOVED in
> reconciliation (it timed out under the `cargo test` shared-process gate —
> Codex P2). Final delta is **+4 Rust tests (1785→1789)**, not +5/1790. The
> tables below that still cite the pre-removal +5/1790 are the driver's
> original snapshot; the `## Codex reconciliation` section is authoritative for
> the committed state.

## Scope And Staging

Working-tree diff (uncommitted), HEAD `5b07472`. 5 files, +620/-9:

| File | Nature | LOC |
|---|---|---|
| `crates/nexus-shell-daemon/src/result_sync.rs` | **prod fix** (dedup key) + 3 tests | +416/-9 |
| `crates/nexus-coordinator-rs/src/validator.rs` | 1 test (verrou) | +65/-0 |
| `crates/nexus-worker-core/src/engine/runtime.rs` | 1 test | +36/-0 |
| `docs/rust/PATTERNS.md` | §P60 (+§P60.1/.2/.3) | +74/-0 |
| `docs/security/THREAT_MODEL.md` | §15.2 + threat table | +29/-0 |

**Scope amendment accepted (NOT scope creep).** The G8 preflight originally said
"test + doc, zero prod change" with a load-bearing S4 feasibility claim that was
WRONG. A read-only pre-implementation pass found a confirmed production gap that
blocks Phase D's #1 deliverable cross-machine. The preflight Addendum (2026-06-17)
records the PO **Option A** arbitration: fix in Phase D + prove it. The amended
scope = test + targeted prod fix + doc. The production change is in-scope and is
the heart of the phase. I verified the Addendum is present, dated, and traces the
gap with multi-evidence (preflight L30-92).

**Atomicity.** All 5 files belong to one coherent unit: the dedup fix + the 4
Rust tests that prove it + the 2 doc rows that record it. No `pub mod` added (grep
clean). No planning/cache/build/unrelated-refactor leakage. The only untracked
file is `sprint76_phase_d_preflight.md` (planning, committed separately or with
the phase per repo convention). Atomic.

## Three-Block Verification

Re-run by the driver this session (Windows native):

- `cargo fmt --all --check` — **clean**.
- `cargo clippy -p nexus-shell-daemon -p nexus-coordinator-rs -p nexus-worker-core --all-targets --locked -- -D warnings` — **0 warnings**.
- Targeted nextest, the 5 new tests — **5/5 PASS** (seed 0.05s, validator_unchanged
  0.05s, 2 hermetic bridge 0.28s each, cross-node E2E 4.08s).
- Full nextest on the 3 touched crates — **881/881 passed, 0 skipped**.
- Driver §7.4 report (Windows, GREEN): fmt OK / clippy OK / **nextest workspace
  1790/1790 0-skip (1785→1790 = +5)** / doctests OK / release daemon build OK.
- Frontend / Python: **0 delta** (no `web/` or `packages/` file touched) — correct
  for a Rust-only quorum phase, no suite to run.
- **Docker canonical Linux deferred to pre-push recovery** (S76-C documented env
  pattern; push deferred). This is the one residual verification gap — see Residual
  Risk. Non-blocking for PASS-PENDING because the diff is platform-agnostic Rust
  (no `#[cfg(unix)]`, no FS-path-specific logic) and the Windows gate is GREEN.

## Delta Tests

+5 Rust tests, matching the §7.4 1785→1790 delta exactly:

| Test | Home | Kind | Proves |
|---|---|---|---|
| `quorum_redundancy_two_workers_reach_validator` | result_sync.rs | hermetic 2-author | both votes forwarded → quorum Completed |
| `quorum_redundancy_diverging_outputs_rejected` | result_sync.rs | hermetic 2-author | divergent → Rejected, both seen first |
| `quorum_redundancy_two_stubworkers_byte_identical` | result_sync.rs | 3-node real-iroh E2E | dispatch→sync→worker→result→bridge→validator→DB |
| `verifiable_seed_is_cross_worker_stable` | runtime.rs | unit | seed = u32 LE of blake3(task_id)[..4], cross-worker stable |
| `validator_quorum_unchanged` | validator.rs | in-DB verrou | self-inflation blocked + 2-distinct accept |

No suite has an unexplained 0 delta. Frontend/Python 0 is correct (no file touched).

## Modified-File Branch Coverage

The only production behavior change is `forward_result_entry`'s dedup key
(result_sync.rs:124-152). I verified branch coverage **semantically**, not by grep:

1. **Dedup-suppress branch** (`if !seen.insert(dedup_key)`): exercised by the
   hermetic tests (boot catch-up + InsertRemote both feed the same `seen`,
   result_sync.rs:172/181/231) and by the validator-loop idempotent backstop.
2. **Send-failure un-mark** (`seen.remove(&dedup_key)`): uses the SAME composite
   key as the insert (verified read of L131 vs L146). Both sides key on
   `format!("{worker_id}:{task_id}")` — no asymmetry that would leak a permanent
   block. (No dedicated test hits the receiver-dropped path; it is a 1-line
   defensive branch with the main path covered — P3, see Findings.)
3. **redundancy>1 enable**: the central claim. I ran the **red-before-green proof
   myself**: reverted the dedup key back to `task_id` alone in-place, re-ran the
   two hermetic tests → **both FAILED with `Elapsed` (timeout in AwaitingQuorum)
   at 10s**, exactly as the fix predicts. Restored the fix; diff stat back to
   416/9. The guard is GENUINE, not a tautology.

The 3 `result_sync` tests are real (not mocked at the frontier): the E2E boots two
real `Engine` instances on their own iroh nodes joining by `share_write` ticket,
the only stub being the deterministic `StubBackend`. Assertions are specific
(`status == Completed`, `result_hash == agreed`, `get_task_results().len() == 2`),
not `is_ok()`.

## Security And Protocol

This phase touches the compute-quorum trust surface — full DEEP audit performed.

- **Validator INCHANGE verrou — VERIFIED.** `git diff --stat validator.rs` =
  65 insertions, **0 deletions**, all inside `mod tests`. `validate_quorum_pre_guardrail`
  body is byte-identical to HEAD. The trust boundary did not move.
- **Dedup-key identity match — VERIFIED at the source.** The fix keys on
  `hex::encode(entry.worker_pubkey)` (result_sync.rs:130). The validator loop
  derives `worker_id = hex::encode(entry.worker_pubkey)` (validator_loop.rs:108)
  and the DB table `task_results` is `UNIQUE (task_id, worker_id)` with
  `INSERT OR IGNORE` (db.rs:126/549). The bridge now mirrors the SAME identity the
  validator and DB already use. Two layers, one identity — correct.
- **No new Sybil surface.** Before: bridge collapsed all workers to the first vote
  (quorum never formed). After: one vote forwarded per distinct `worker_pubkey`; a
  single worker still cannot vote twice (same pubkey deduped at the bridge AND the
  DB). Quorum inflation still requires N real keypairs = the pre-existing Sybil cost
  (PoW/AgeWitness + closed pilot), unchanged. The exact-match strict-majority +
  outlier rejection (validator.rs ~290-336, confirmed present) remains the trust
  boundary. The THREAT_MODEL §15.2 row states this honestly (Sybil residual = M,
  assumed cost; cohort gate = advisory routing, not a trust boundary).
- **No panic surface added.** The prod change adds only `hex::encode`, `format!`,
  HashSet ops — no unwrap/panic/expect/unsafe/todo in non-test code (grep clean).
- **No wire/protocol change.** `TASK_FORMAT_VERSION` stays 1, no new `DOMAIN_*`, no
  `serde(default)`, no tolerant multi-version decoder, no Cargo.toml/lock change
  (all grep-verified). Daemon-internal bridge logic only.
- **Red-line DEEP triggers checked:** no canonical.rs/schemas edit, no new unsafe
  or `#[allow(dead_code)]`, no crypto change, no loopback-auth or zip-extract edit.
  THREAT_MODEL is touched (additive §15.2 row) — reviewed line by line; claims
  match code.

## Research And G8

G8 preflight present (`sprint76_phase_d_preflight.md`), EXECUTE verdict, with a
dated Addendum recording the Option A scope amendment. S1a is APPROACH-ALIGNED
(BOINC homogeneous-redundancy byte-for-byte + TOPLOC cross-hardware deferral,
both cited with URL+date 2026-06-16). S1b clean (0 dep). S2 clean (validator last
touched `0daff81` S71, INCHANGE held). S3/S4 clean except the now-corrected "no
prod change" claim (the Addendum is the correction). No new crypto/spec/dep
introduced this phase, so research grounding is satisfied. PATTERNS §P60 records
the lesson; THREAT_MODEL §15.2 records the surface. Doc claims I spot-checked
(logprobs_hash `[u8;32]` at task.rs:511; `quorum_rejects_nondeterministic_divergence`
at validator.rs:765; outlier logic ~290-336) all match code.

## Scope Cuts

All honored (kickoff §7 / plan §D.5, re-grepped against the diff):
- TOPLOC étage-2 = design note only, **0 code** — verified (PATTERNS §P60.3 +
  THREAT_MODEL row, no signing domain, slot already v1).
- Cross-GPU heterogeneous exact-match = post-S77 — documented as expected-not-bug
  (anti-false-green), not implemented.
- `validate_quorum_pre_guardrail` INCHANGE — diff-empty verrou, VERIFIED.
- iroh 0.98 pinned, 0 wire bump, 0 new dependency, kudos non-monetary untouched —
  all verified.
- LIVE acceptance (palier-2 quorum redundancy=2 VPS+PC+Mac) deferred to Phase G
  (same posture as B-3 in Phase C, material on user hardware). Tracked, not dropped.

## Codex reconciliation

Codex `codex exec` (GPT-5.5) ran over the diff; raw output in
`sprint76_phase_d_codex_review.md` (NOT rewritten). **Overall: PARTIAL, no
P0/P1 GAP. Count PASS 4 / PARTIAL 2 / GAP 0.** Per-deliverable: FIX CORRECTNESS
PASS, VALIDATOR UNCHANGED PASS (independently confirmed 0-deletion in the
function body), SCOPE/WIRE/DAY-0 PASS, SECURITY PASS; TEST SEMANTICS PARTIAL +
DOCS ACCURACY PARTIAL. The two PARTIALs were resolved at the ROOT (not
documented-and-shipped):

- **Codex P2 — 3-node E2E unstable under `cargo test` shared-process.** Codex
  reproduced a timeout of `quorum_redundancy_two_stubworkers_byte_identical`
  under default-parallel `cargo test` (passes alone, under `--test-threads=1`,
  and under nextest). I confirmed it: under full-crate `cargo test`
  shared-process the test panicked at the 120s backstop (124.17s), while the two
  hermetic tests passed. A literal multi-worker E2E needs THREE iroh nodes
  (each `Engine` boots its own node), which is too heavy for the shared-process
  gate that P2-A-1 closed. **Resolution: REMOVED the 3-node test.** The
  redundancy>1 quorum is proven by composition (2 hermetic two-author tests over
  the real bridge, red-before-green; + pre-existing cross-node single-worker
  replication test; + Phase G LIVE for literal cross-machine). A
  `// NOTE` in result_sync.rs records the decision. Final delta: **+4 Rust
  (1785→1789)**.
- **Codex P3 — "cross-process" overstated.** The E2E was in-process. Fixed the
  wording in the test comment, PATTERNS §P60.1, and THREAT_MODEL §15.2
  ("in-process" / "by composition", "cross-machine = Phase G LIVE only").
- **Codex P3 — seed little-endian precision.** Added the explicit "u32
  little-endian truncation of the first 4 bytes of blake3(task_id)" to PATTERNS
  §P60.2 (already locked by `verifiable_seed_is_cross_worker_stable`).

**Suites re-run AFTER the reconciliation edits (Windows, GREEN):** fmt --check
OK; clippy --workspace --all-targets -D warnings OK (0 unused from the removal);
`cargo test -p nexus-shell-daemon --locked` **383 + 6 + 7 = all pass, 0 fail
(daemon binary unit tests in 10.65s — the P2 shared-process gate now GREEN)**;
`cargo nextest run --workspace --locked` **1789/1789 passed, 0 skipped (+4)**.
doctests + release daemon build unchanged (no non-test prod code changed after
the green §7.4 run — only test removal + comments + docs). Docker canonical
Linux still deferred to pre-push recovery (P2-D-1, S76-C pattern).

Codex did NOT need a re-run: it found 0 P0/P1 GAP, and the reconciliation only
removed a redundant runner-fragile test + tightened doc wording — the verified
logic (the fix, the hermetic tests, validator-unchanged, wire/scope, security)
is byte-unchanged.

Security delta (unchanged by reconciliation): the dedup fix forwards one vote
per distinct worker pubkey where it previously collapsed to one — no new Sybil
surface; trust boundary (exact-match quorum) unchanged.

## Commit Body Draft

```
feat(daemon+coordinator): Sprint 76 Phase D — redundancy>1 quorum over the bridge (PO Option A)

## Contexte
D3 etage 1 palier 2 : prouver le quorum deterministe redundancy_factor>1
de bout en bout cross-process. Le G8 preflight disait "test+doc seul" ; une
passe read-only de pre-implementation a trouve un gap prod confirme qui
bloquait l'objectif cross-machine. PO arbitrage Option A (2026-06-17) :
fix root-cause dans Phase D + le prouver. Addendum preflight enregistre.

## Fichiers
- result_sync.rs : fix dedup forward_result_entry task_id -> (worker_pubkey,
  task_id) [~5 l prod] + 2 tests hermetiques 2-auteurs (accept + diverge) +
  une NOTE expliquant pourquoi le 3-noeuds in-process est exclu (Codex P2 :
  trop lourd pour le gate cargo test shared-process ; couverture par
  composition).
- validator.rs : test validator_quorum_unchanged (verrou INCHANGE :
  self-inflation bloquee + 2-distinct accepte). Fonction diff-vide.
- runtime.rs : test verifiable_seed_is_cross_worker_stable (seed = u32 LE de
  blake3(task_id)[..4], pas le digest entier — contrat honnete).
- PATTERNS.md §P60 (+.1 dedup-mirror / .2 exact-match homogene / .3 TOPLOC
  etage-2). THREAT_MODEL.md §15.2 + table menaces compute-quorum.

## Delta tests
Rust +4 (workspace 1785->1789, 0 skip). Frontend/Python 0 (aucun fichier
touche). Red-before-green verifie : sans le fix les 2 tests hermetiques
timeout en AwaitingQuorum a 10s.

## Verification
fmt OK / clippy --workspace --all-targets -D warnings OK / nextest 1789/1789
0-skip / `cargo test -p nexus-shell-daemon` shared-process 383+6+7 0-fail
(gate P2 vert) / doctests OK / release daemon OK (Windows). Docker canonique
differe recovery avant push (pattern S76-C, diff platform-agnostique). Note
doc : seed decrit honnetement comme troncature u32 LE, pas le digest complet.

## Scope cuts
TOPLOC etage-2 = design note 0 code ; cross-GPU heterogene = post-S77 ;
validate_quorum_pre_guardrail INCHANGE (verrou diff-vide) ; iroh 0.98 pin ;
0 bump wire ; 0 dep ; LIVE acceptance redundancy=2 differee Phase G.

## G8 traceability
Preflight EXECUTE + Addendum Option A (gap multi-evidence, fix ~5 l, note
securite). S1a APPROACH-ALIGNED (BOINC + TOPLOC). 0 DESIGN-CONFLICT.

## Pre-launch protocol
0 *_VERSION bump, 0 DOMAIN_ nouveau, 0 serde(default), 0 decoder multi-version,
0 Cargo.(toml|lock). Logique bridge daemon-interne.

## Codex verification
Codex GPT-5.5 (codex exec, output brut sprint76_phase_d_codex_review.md) :
PARTIAL, 0 P0/P1 GAP (PASS 4 / PARTIAL 2). Les 2 PARTIAL resolus a la racine :
(P2) 3-noeuds E2E instable sous cargo test shared-process -> test RETIRE
(couverture par composition : 2 hermetiques + cross-node redundancy=1 existant
+ Phase G LIVE) ; (P3) "cross-process" surdit -> "in-process"/"composition" ;
(P3) precision seed u32 LE. Suites relancees apres reconciliation : GREEN
(nextest 1789/1789, daemon shared-process 0-fail, clippy/fmt). Pas de re-run
Codex (0 P0/P1 ; logique verifiee inchangee). Security delta : une voix par
worker pubkey distinct (avant : collapse sur la 1re) ; 0 nouvelle surface
Sybil ; frontiere exact-match quorum INCHANGE.

## Carry closure / Unblock
Debloque le quorum cross-machine redundancy>1 (Phase D #1 + acceptance D2
falsifiable Phase G). result_sync.rs (prod) touche. 2 P2 (Docker differe,
3-noeuds E2E retire) + 2 P3 (send-failure untested, log cosmetique) routes
verification/audit_plan (voir review + Codex reconciliation).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

## Findings

**P2-D-1 — Docker canonical Linux run deferred (process gap, not code).**
The §7.4 GREEN is Windows-only; the canonical Linux gate is deferred to pre-push
recovery (documented S76-C pattern). The diff is platform-agnostic Rust (no
`#[cfg(unix)]`, no path/FS-specific logic), so the risk is low, but the dual-platform
policy (`feedback_dual_platform.md`) is not yet satisfied for this phase. Owner:
driver. Trigger: before push. Exit: Docker nextest GREEN on the 3 crates (or
workspace). Route → `sprint76_verification.md`.

**P2-D-2 — E2E test wall-clock budget (40s) is the only timing guard.**
`quorum_redundancy_two_stubworkers_byte_identical` asserts quorum within 40s over
real iroh sync. On a loaded CI box (the S76-C OOM-from-concurrent-builds incident
is on record) this could flake. The hermetic 2-author tests (10s, no engine boot)
de-risk the core property, so a flake here is a harness timing issue, not a
correctness regression — but it should be run sequentially (not concurrent with
other heavy builds) per the S76-C lesson. Owner: driver. Trigger: CI flake.
Exit: stable green across 3 sequential runs, or budget raised with rationale.
Route → `sprint76_verification.md`.

**P3-D-3 — send-failure un-mark path (`seen.remove`) has no dedicated test.**
The receiver-dropped branch (result_sync.rs:143-151) is a defensive 1-liner; it
now correctly uses the composite key (verified by read), but no test exercises the
"validator loop gone → un-mark → retry" sequence. Main path covered. Low priority.
Route → `sprint77_audit_plan.md`.

**P3-D-4 — `worker = %&worker_id[..16.min(...)]` log slice is cosmetic.**
The truncated-pubkey debug log is fine but introduces a `min` guard that only
matters for empty/short hex (never the case for a 32-byte pubkey = 64 hex chars).
Harmless defensive code; noting for completeness. No action required.

## Residual Risk

The fix is minimal, root-cause, and proven red-before-green by my own revert
experiment. The validator trust boundary is verifiably unchanged (0-deletion diff).
The one real residual is **process, not code**: the canonical Docker Linux run is
deferred (P2-D-1) — acceptable for PASS-PENDING given the platform-agnostic diff
and GREEN Windows gate, but it MUST clear before push. The LIVE redundancy=2
acceptance (D2 falsifiable criterion) is correctly deferred to Phase G on user
hardware; the in-process E2E is its faithful analogue and is GREEN. No P0/P1.
Ready for Codex.
