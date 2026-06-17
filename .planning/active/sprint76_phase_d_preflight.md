# Sprint 76 Phase D Preflight

Date: 2026-06-16 (addendum 2026-06-17)
HEAD: `5b07472`
Verdict: **EXECUTE** (scope amended — see Addendum: test + **targeted production fix** + doc)

> Phase D proves the `redundancy>1` deterministic quorum on byte-identical
> outputs. The 5 scans find **no blocking issue**. S1a is decisively
> APPROACH-ALIGNED: byte-for-byte homogeneous-redundancy quorum is the canonical
> BOINC volunteer-computing pattern, and the étage-2 TOPLOC deferral is correct
> prior art (TOPLOC exists *because* exact-match fails cross-GPU). S1b is clean
> (test + doc-note only, zero dependency added or bumped). S2 has no un-reverted
> decision in the way (the quorum was last touched S71 `0daff81`, rationale still
> valid, and Phase D explicitly keeps it INCHANGE). S3 regresses no T0-T5 (the
> phase adds a *test*, not a new surface). S4 is clean: zero `*_VERSION` bump,
> `logprobs_hash`/`model_digest` already v1, `required_runtime` already shipped in
> Phase C. **Crucially, the cross-process two-worker test is feasible with the
> current production code unchanged** — `task_already_handled_on_doc` dedups only
> on the worker's OWN author (`runtime.rs:1304`), so two distinct-author workers
> both claim+execute the same `redundancy=2` task by design.
>
> META-AUTONOMY SIGNAL: this is **EXECUTE**, NOT a 2nd consecutive PLAN-ADAPT
> after Phase C. The Phase C PLAN-ADAPT already did the heavy lifting (it added
> `RuntimeTuple` + `required_runtime` + the claim-gate + the StubBackend tuple
> stub); Phase D consumes that infrastructure to assert a property, so there is
> nothing left to adapt. Two non-blocking documentation imprecisions in the plan
> are recorded below (stale line refs; `seed` is a `u32` truncation of
> `blake3(task_id)`, not the full digest) — they do not change the verdict.

## Addendum (2026-06-17) — Production gap discovered at implementation prep; PO Option A

> **The S4 "feasibility" claim below was WRONG on one load-bearing point.** It
> asserted "no production change is needed for the cross-process redundancy>1
> test." A read-only pre-implementation pass found a confirmed production gap that
> blocks Phase D's central goal (cross-machine redundancy>1 quorum). The verdict
> stays **EXECUTE**, but the scope is amended to **test + a small root-cause
> production fix + doc**. PO arbitrated **Option A** (fix in Phase D + prove it)
> on 2026-06-17.

### The gap (multi-evidence, confirmed)
- `crates/nexus-shell-daemon/src/result_sync.rs:113` — `forward_result_entry`
  deduplicates forwarded results by `task_id` **alone**
  (`seen: HashSet<String>`, `if !seen.insert(task_id.clone()) { return; }`).
- Two workers on a `redundancy_factor=2` `verifiable` task write the **same key**
  `result:{task_id}` under **distinct iroh-docs authors** (`runtime.rs:1146`).
  iroh-docs keeps both as distinct `(author, key)` entries → two `InsertRemote`
  events (and the boot catch-up iterates both with the same `seen`).
- Consequence: the first result is forwarded; the second is dropped ("result
  already forwarded, skipping") **before** it reaches the validator. The validator
  sees only ONE result, `count` never reaches `redundancy_factor=2`, the task stays
  `AwaitingQuorum` forever (the B.2 early-reject only fires at redundancy≥4). The
  redundancy>1 quorum **never forms over the result-sync bridge**.
- The validator itself is correct: it dedups by `(worker_id = hex(worker_pubkey),
  task_id)` (`validator_loop.rs:108-113`, `insert_task_result`) and reaches quorum
  when both results arrive (proven by `validator.rs` det-pair tests L710-762 and
  `validator_loop.rs` two-result tests L439-441). **The bug is purely in the
  bridge's dedup key.**
- The synchronous HTTP submit path (`POST /api/v1/results/submit`) has no such
  dedup, so a co-located worker's redundancy>1 works. But the **cross-machine**
  scenario (PULL, iroh-docs — VPS+PC+Mac) is exactly the result-sync bridge path,
  so Phase D's #1 deliverable cannot pass over the real path and Phase G's LIVE
  acceptance (redundancy=2, the D2 falsifiable criterion) would fail.

### The fix (root-cause, small, no wire change)
- Change `forward_result_entry`'s `seen` key from `task_id` to
  `(task_id, worker_pubkey)` — mirroring the validator's own `(worker_id, task_id)`
  dedup. The original intent (suppress `InsertRemote` refire of the **same**
  worker's result + boot/live overlap) is preserved (same worker_pubkey + task_id
  still deduped); distinct workers' votes now all reach the validator.
- ~5 lines in `result_sync.rs`. **Zero** wire/format/domain change
  (`TASK_FORMAT_VERSION` stays 1, no new `DOMAIN_*`, no `serde(default)`).
  Daemon-internal bridge logic only.
- Security: no new surface. Before, the bridge collapsed all workers to one vote
  (quorum never formed); after, it forwards exactly one vote per distinct worker
  pubkey — a single worker still cannot vote twice (same pubkey deduped). Quorum
  inflation needs N distinct keypairs = the pre-existing Sybil concern (PoW /
  AgeWitness mitigations elsewhere), unchanged. The exact-match strict-majority +
  outlier rejection in `validate_quorum_pre_guardrail` remains the trust boundary,
  INCHANGÉ. A THREAT_MODEL compute-quorum row records this.

### Amended scope
- **NEW** target file (production fix): `crates/nexus-shell-daemon/src/result_sync.rs`
  (`forward_result_entry` dedup key, ~5 lines + doc-comment).
- The four Rust tests still hold; test #1
  (`quorum_redundancy_two_stubworkers_byte_identical`) is now sited in
  `result_sync.rs` as a **3-node** cross-process harness (coordinator + 2 worker
  engines) that exercises the **real** bridge → validator → coordinator-DB chain,
  so it genuinely proves the fix (without the fix it would hang at the 30s timeout
  — a true red-before-green, anti-faux-vert).
- Everything else (`validate_quorum_pre_guardrail` INCHANGÉ; TOPLOC étage-2 design
  note; LIVE acceptance deferred to Phase G; seed = u32 truncation of
  `blake3(task_id)`; zero dependency) is unchanged from the original analysis.

## Evidence Rules
- Claim policy: every claim below cites a repo path, a command output, a
  URL+date, or an explicit assumption.
- Local sources read: `prompts/agent/preflight.md` (full procedure);
  `.planning/active/sprint76_plan.md` (§7 Phase D L387-462, §8 Phase E L465-479);
  `.planning/active/sprint76_kickoff.md` (§1, §D1/§D2/§D3 sources);
  `.planning/active/sprint76_phase_c_preflight.md` (full — the precedent
  PLAN-ADAPT that shipped `RuntimeTuple`); `.planning/active/sprint76_design_review.md`
  (D3 review L59-96, the `model_digest`/`logprobs_hash` already-exist correction);
  `crates/nexus-coordinator-rs/src/validator.rs` (L200-360 quorum INCHANGE target +
  L690-768 existing det-pair quorum tests); `crates/nexus-core-rs/src/task.rs`
  (L1-405 RuntimeTuple/required_runtime/verifiable/redundancy_factor + L502/511
  model_digest/logprobs_hash); `crates/nexus-worker-core/src/engine/runtime.rs`
  (L820-919 task pump + dedup, L1025-1124 claim-gate + exec, L1288-1348
  task_already_handled_on_doc + build_generate_params + deterministic_seed,
  L1899-2018 cohort-gate tests); `crates/nexus-worker-core/src/llm/ollama.rs`
  (L305-411 StubBackend + with_runtime_tuple + runtime_tuple); `crates/nexus-worker-core/src/llm/mod.rs`
  (L328-344 default runtime_tuple); `crates/nexus-shell-daemon/src/dispatch_loop.rs`
  (L1-60 dispatch loop + L155-305 worker-pump E2E test);
  `crates/nexus-shell-daemon/src/result_sync.rs` (L350-518 the cross-node
  E2E that already wires dispatch->sync->validator_loop->coordinator DB);
  `crates/nexus-core-rs/src/canonical.rs` (DOMAIN_* + version grep);
  `docs/rust/PATTERNS.md` (§P53 L2736-2837 deterministic quorum + cross-GPU limit,
  §P54 cross-process E2E); `docs/security/THREAT_MODEL.md` (compute rows grep).
  External local memory (cited by CLAUDE.md): `feedback_approach.md` (pick deepest,
  no band-aid), `feedback_named_constants.md`, `feedback_context7_systematic.md`.
- Commands run (relevant outputs inline below):
  - `git rev-parse --short HEAD` => `5b07472`; `git log --oneline -12` confirms
    A `ce43894` / B `6904cdd` / C `1cc28e7` + verification `5b07472` closed.
  - `git status --short` => clean (no uncommitted Phase D diff yet).
  - `grep -n "RuntimeTuple|required_runtime|verifiable|redundancy_factor" task.rs`
    => `required_runtime: Option<RuntimeTuple>` (L316), `with_required_runtime`
    (L405), in canonical (NOT removed like `redundancy_factor`), `TASK_FORMAT_VERSION
    = 1` (L61).
  - `grep -n "deterministic_seed|build_generate_params" runtime.rs` => the
    determinism contract is at L1323-1348 (NOT plan's L1260-1285), seed is a
    `u32` from the first 4 bytes of `blake3(task_id)` (L1345-1348).
  - `git log --oneline -- validator.rs` => last quorum touch `0daff81` (S71 B-2,
    deterministic quorum); nothing since. INCHANGE constraint holdable.
  - `grep -A1 '^name = "..."' Cargo.lock` => tokio 1.52.3, tempfile 3.27.0,
    blake3 1.8.5, rusqlite 0.36.0 (all already vendored; Phase D adds none).
  - `cargo tree -d` => only the pre-existing iroh transitive duplicate stack
    (`base64` 0.21/0.22, `bitflags` 1/2, `curve25519-dalek` 4/5-pre, etc.); none
    introduced by Phase D.
- context7: not required this phase (no new library/API touched; the only
  external concepts are research-cited OSS schemes, covered by WebSearch). The
  Phase C preflight already exercised context7 on `/ollama/ollama` +
  `/pepperoni21/ollama-rs` for the live-acceptance backend; Phase D's LIVE
  acceptance reuses that same path unchanged.

## Scope
- Plan source: `.planning/active/sprint76_plan.md` §7 (Phase D, L387-462).
- Target files:
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (the worker-pump E2E test
    module, L155-305) OR `crates/nexus-shell-daemon/src/result_sync.rs` (the
    cross-node E2E, L360-518) — the natural home for a two-worker redundancy>1
    test (see S3/S4 feasibility note: result_sync already wires the FULL
    dispatch->sync->validator_loop->DB chain, so the quorum-accept assertion is
    cleanest there).
  - `crates/nexus-coordinator-rs/src/validator.rs` — **READ-ONLY verrou** (the
    `validate_quorum_pre_guardrail` INCHANGE assertion; existing det-pair tests
    L710-768 are the in-DB analogue).
  - `docs/rust/PATTERNS.md` (§P53 extension or a new §P-row: TOPLOC étage-2
    design note + cross-process redundancy>1 proof).
  - `docs/security/THREAT_MODEL.md` (one compute-quorum / cohort-determinism row).
  - `.planning/active/sprint76_verification.md` (§7.4 + LIVE acceptance checklist,
    deferred to Phase G as in Phase C).
- Deps/APIs/specs: **none** (test + doc-note). Zero `Cargo.toml` change.
- Security/protocol surfaces: `validate_quorum_pre_guardrail` (must stay
  INCHANGE); `Task.verifiable`/`redundancy_factor`/`required_runtime` (read, not
  changed); `ResultPayload.model_digest`/`logprobs_hash` (read, not changed — the
  TOPLOC slot is a DESIGN NOTE, no code).
- Tests expected (plan §D.3): `quorum_redundancy_two_stubworkers_byte_identical`,
  `quorum_diverging_outputs_rejected`, `verifiable_seed_is_cross_worker_stable`,
  `validator_quorum_unchanged`, LIVE `quorum_live_vps_pc_mac_consensus` (Phase G).

## S1a OSS Prior Art
- Domain: redundant volunteer-compute consensus by output agreement (BOINC /
  Folding@Home family) + verifiable LLM inference under hardware non-determinism
  (TOPLOC / Thinking Machines / SGLang).
- Sources (all accessed 2026-06-16):
  - **BOINC homogeneous redundancy + byte-for-byte validator**
    (`boinc.berkeley.edu/trac/wiki/JobReplication`,
    `github.com/BOINC/boinc/wiki/JobReplication`): "The validator is run when
    there are this many successful results. If a strict majority agree, they are
    considered correct. Set this to two or more if you want redundant computing.";
    "for apps that use **homogeneous redundancy** to achieve **bitwise agreement**
    between instances, BOINC supplies a validator that compares the output files
    **byte for byte**"; "If a consensus is reached, a particular result is
    designated as the 'canonical' result." `min_quorum` <= `target_nresults`.
  - **TOPLOC: A Locality Sensitive Hashing Scheme for Trustless Verifiable
    Inference** (Prime Intellect, arXiv:2501.16007, ICML 2025 poster;
    `primeintellect.ai/blog/toploc`, `github.com/PrimeIntellect-ai/toploc`):
    a top-k hidden-state LSH commitment "**robust across diverse hardware
    configurations, GPU types, and algebraic reorderings**", 258 bytes / 32
    tokens, "built-in mechanisms to handle floating-point discrepancies caused by
    non-deterministic computations common in GPU executions". It is *exactly* the
    cross-GPU verification scheme the plan defers to étage 2 — and it exists
    BECAUSE byte-exact cross-GPU agreement does not hold.
  - **Defeating Nondeterminism in LLM Inference** (Thinking Machines Lab,
    2025-09; `thinkingmachines.ai/blog/defeating-nondeterminism-in-llm-inference/`,
    Simon Willison 2025-09-11) + **SGLang deterministic inference** (LMSYS,
    2025-09-22): bitwise-identical (1000/1000 runs) reproducibility is achievable
    on the **same GPU** via batch-invariant kernels; the cause of divergence is
    batch-size/load variance and cross-hardware float reordering. Confirms (a) the
    homogeneous-cohort exact-match path is sound, (b) the "hétérogène-diverge
    expected" criterion is the honest, physically-grounded statement.
- Finding: **APPROACH-ALIGNED**. The plan's mechanism (deterministic +
  homogeneous redundancy -> strict-majority byte-identical quorum -> canonical
  result; cross-GPU exact-match expected to diverge; TOPLOC reserved for the
  cross-hardware étage 2) is a faithful instance of the mature BOINC pattern, and
  the TOPLOC deferral is correctly motivated by the SOTA. Not `APPROACH-NAIVE`.
- `LIB-EXISTS` check: the only candidate library is TOPLOC, but it is a Python
  scheme requiring access to model intermediate activations (top-k hidden
  states). The live worker path is the Ollama HTTP black box (no hidden-state
  access — established in the Phase C preflight). A real TOPLOC commitment is
  gated on a file/activation-exposing backend (`LlamaCppBackend`, feature
  `llm_llama_cpp`), which is the S77 étage-2 reserve the plan already names. So
  `LIB-EXISTS` does **not** fire for the in-scope (S76) work; it is the correct
  S77 path. Non-blocking.
- Impact: none. Proceed as written.

## S1b Dependencies, CVEs, Release Notes
- Scanned: the crates Phase D's test/doc touches — tokio 1.52.3, tempfile
  3.27.0, blake3 1.8.5, rusqlite 0.36.0 (all from `Cargo.lock`). Phase D adds and
  bumps **nothing**.
- Commands/sources: `grep -A1 '^name = "<crate>"' Cargo.lock` for each (versions
  above). `cargo tree -d` => only the standing iroh transitive duplicate set
  (`base64` 0.21/0.22, `bitflags` 1/2, `curve25519-dalek` 4.1.3 vs 5.0.0-pre.6,
  `crypto-common` 0.1/0.2, etc.) — none introduced here. No new crate, so the
  S72 lesson (ollama-rs 0.3.4 -> schemars 1.2 collision) cannot recur: there is
  no dependency edge to walk.
- Finding: **clean**. Zero dependency change; no CVE relevant (no crypto/wire/
  network/sandbox edge is added or modified). Test + doc-note only.

## S2 Historical Decisions
- Commands:
  - `git log --oneline -- crates/nexus-coordinator-rs/src/validator.rs` =>
    `8b53c38` (S75 G), `bede850` (S74 G), `6f5ff30` (S73 A guardrail-before-
    persist), `110c003` (S72 D), **`0daff81` (S71 B — deterministic quorum, the
    last logic touch)**, `0cb576d` (S55 C quorum SHA256), older S35-S40.
  - `git log --oneline -- crates/nexus-shell-daemon/src/result_sync.rs` =>
    `1cc28e7` (S76 C), `d30f949` (the original "bridge worker results into
    coordinator DB").
  - `grep -rn "TOPLOC|logprobs_hash|llm_llama_cpp|layer 3"` across docs/.planning.
- Decisions crossed / reversion status:
  - **`34c77ce` (S23 R3): `redundancy_factor` deliberately EXCLUDED from
    `task_canonical_bytes`** (`task.rs:39-52`, `obj.remove("redundancy_factor")`,
    test `task_canonical_excludes_redundancy_factor` L898). Rationale (dispatch
    policy is not cryptographic identity) **still valid, no reversion**. Phase D
    reads `redundancy_factor` to drive the quorum; it does not add it to the
    canonical. No conflict.
  - **`0daff81` (S71 B-2): `verifiable` => greedy + fixed seed; validator is
    mode-agnostic and INCHANGE.** This is the load-bearing decision Phase D builds
    on. No reversion since; Phase D explicitly re-affirms INCHANGE
    (`validate_quorum_pre_guardrail`, L219-338, including the B.2 quorum-impossible
    early-reject from S73/S74 and the outlier-logging L290-336 already present).
    Reverse-commit check: `git log 0daff81..HEAD -- validator.rs` shows only the
    guardrail-split (`6f5ff30`) and wrap-up hygiene commits — the deterministic
    quorum semantics are intact, never reverted.
  - **Design-review D3 correction (`sprint76_design_review.md` L59-96):**
    `model_digest`/`logprobs_hash` already exist in `ResultPayload` v1;
    `logprobs_hash` (NOT `result_hash`) is the documented "layer 3" TOPLOC slot.
    Phase D honors this exactly: the étage-2 TOPLOC design note points at
    `logprobs_hash` (task.rs:511) and adds **no code**. No reversion, decision
    consumed as written.
  - **Phase C PLAN-ADAPT (`sprint76_phase_c_preflight.md`):** shipped
    `RuntimeTuple` + `Task.required_runtime` (signed canonical) + the worker
    CLAIM-GATE + StubBackend tuple stub. Phase D consumes this, adds nothing to
    the wire. The "PULL claim-gate, not dispatcher assignment" finding from Phase
    C is the architecture Phase D's two-worker test relies on. No reversion.
- Finding: **clean**. No un-reverted decision contradicts Phase D. The only
  binding constraints (validator INCHANGE; `redundancy_factor` stays out of
  canonical; TOPLOC slot = `logprobs_hash`, not net-new) are all honored by the
  plan as written.

## S3 Local Patterns And Threat Model
- Threats/contracts checked (THREAT_MODEL compute surface):
  - **Sybil / lying-worker against the quorum (T-compute).** Already mitigated and
    **unchanged** by Phase D: a worker that produces a divergent `result_text`
    (whether by lying about its cohort tuple or by running different weights) is
    rejected as an outlier (`validator.rs:290-336`, `quorum outlier detected`).
    The cohort gate (Phase C) is advisory routing, not a trust boundary; the
    exact-match quorum is the real defense. Phase D *proves* this boundary with a
    test (`quorum_diverging_outputs_rejected`), it does not move it.
  - **Faux-vert (false-green) risk (T1, anti-rubber-stamp).** The plan's explicit
    "hétérogène-diverge expected" criterion is the mitigation: the LIVE acceptance
    must write BOTH issues (homogeneous -> consensus; heterogeneous -> divergence
    rejected as expected) so a future cross-GPU divergence is not silently read as
    a bug. Backed by Thinking Machines + Ingonyama evidence (S1a) — physically
    grounded, not hand-waved. PATTERNS §P53 already records this limit
    (L2787-2792: "determinism is guaranteed same-machine/same-backend/
    same-model-quant; cross-GPU float non-determinism can break bit-exactness").
  - **Zombie redundancy task (covered S73/S74).** The B.2 quorum-impossible
    early-reject (`validator.rs:248-282`) is preserved; Phase D's redundancy=2/3
    tests must not regress it (the `validator_quorum_unchanged` grep verrou
    guarantees the function body is byte-identical).
- HARDENING_ROADMAP status: no Sprint-76 pre-requirement is missed. Phase D is a
  proof/verification phase (test + doc), not a new security component. A
  compute-quorum / cohort-determinism THREAT_MODEL row (advisory-routing vs
  exact-match trust boundary, cross-GPU divergence as expected-not-bug) is the
  one doc addition — additive, not a fix to an open pre-req. Tracked as a Phase D
  livrable (plan §D.2 "Design note TOPLOC ... THREAT_MODEL row").
- Finding: **clean / non-blocking**. Phase D regresses no covered threat; it
  *demonstrates* an existing boundary and documents the cross-GPU honesty limit.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `canonical.rs` (DOMAIN_* + version grep — all
  `*_V1` constants stable, `TASK_FORMAT_VERSION = 1` at `task.rs:61`); `task.rs`
  (`RuntimeTuple`, `required_runtime`, `model_digest` L502, `logprobs_hash` L511 —
  all already v1 since Phase C / earlier); `validator.rs` (the quorum consumer);
  `result_sync.rs` + `dispatch_loop.rs` (the `task:`/`claim:`/`result:` producer
  -> consumer chain).
- VERSION/domain/canonical status:
  - `TASK_FORMAT_VERSION = 1` — **stays 1**. Phase D adds NO field (test + doc).
  - No new `DOMAIN_*` constant. The TOPLOC étage-2 note adds no signing domain.
  - `model_digest: [u8;32]` (task.rs:502) and `logprobs_hash: [u8;32]`
    (task.rs:511) **already exist** in the v1 signed `ResultPayload`; Phase D
    only *documents* `logprobs_hash` as the TOPLOC slot. **Zero bump.**
- Producer->consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH), the quorum path
  Phase D asserts — both ends read, all PRE-EXISTING:
  - **Producer #1 (worker):** `runtime.rs:1118-1124` builds `ResultPayload`
    {result_text = generated.text, ...}, signs `ResultEntry`, writes
    `result:{task_id}` (`runtime.rs` claim/result write path) onto the project
    doc. The StubBackend output is `format!("STUB[{model}]: {prompt}...")`
    (`ollama.rs:382-390`) — **independent of worker identity** (no node_id /
    keypair in the text), so two distinct workers running the same `verifiable`
    task produce byte-identical `result_text`. This is the determinism premise.
  - **Determinism contract:** `build_generate_params` (`runtime.rs:1323-1337`)
    forces `temperature=0` + `deterministic_seed(task_id)` when `task.verifiable`.
    `deterministic_seed` (`runtime.rs:1345-1348`) = `u32::from_le_bytes` of the
    first 4 bytes of `blake3(task_id.as_bytes())`. **NON-BLOCKING NOTE:** the plan
    §D.1/§D.3 calls this `seed=blake3(task_id)`; the actual seed is the 4-byte
    truncation. The cross-worker-stability property the plan wants
    (`verifiable_seed_is_cross_worker_stable`) still holds (same task_id => same
    u32), but the test/commit doc-comment should state the truncation honestly
    rather than imply the full digest is the seed.
  - **Transport:** iroh-docs replicates the `result:` entry (value = 32-byte
    BLAKE3); the payload travels via iroh-blobs (kickoff §D2). Unchanged.
  - **Consumer #1 (validator path):** `result_sync.rs` observes the remote
    `result:` insert -> forwards `ResultEvent::NewResult` to
    `validator_loop::run` -> `validate_quorum_pre_guardrail`
    (`validator.rs:219-338`) inserts each worker's `result_text` as the `sha256`
    column (raw text, PATTERNS §P53), counts identical values, and on
    `best_count > redundancy_factor/2` returns `Accepted` with `PendingResultPersist
    {result_hash = result_text = best_hash}`. **This is the consumer Phase D
    asserts; it is INCHANGE.** The existing in-DB analogue tests
    (`two_honest_workers_same_hash` L710, `quorum_accepts_deterministic_redundancy`
    L740) already prove the validator side at redundancy=2 — Phase D extends to a
    cross-process two-worker harness.
  - **Consumer #2 (HTTP retrieval):** `GET /api/v1/tasks/{id}/result` returns
    `{result_text}` (per Phase C trace). Unchanged.
  - **Feasibility of the two-worker test (decisive S4 finding):**
    `task_already_handled_on_doc` (`runtime.rs:1291-1311`) dedups ONLY on the
    worker's **own author** (`doc.get_exact(author, claim_key)`, L1304), and the
    doc-comment (L1298-1303) states: "If the key exists under a different author
    the check correctly returns false -> we attempt a write ... our ClaimEntry
    signature makes it unique." Therefore **two distinct-author workers BOTH
    claim+execute the same `redundancy=2` task by design** — no production change
    is needed for the cross-process redundancy>1 test. The test is a new harness
    (two `Engine` instances or two nodes), not a code edit to the prod path.
- Day 0 status: **preserved**. iroh 0.98 pinned (no upgrade — kickoff §D2 notes
  1.0.0-rc.1 exists but upgrade is a Gate-1/PO decision, NOT S76); validator
  INCHANGE; greedy+seed determinism (S71) intact; pre-launch additive policy
  respected (no field added at all this phase); "source verifiable" vocabulary
  untouched; validator quorum is the real trust boundary, cohort gate advisory.
- Finding: **clean**. No `*_VERSION` bump, no new domain, no tolerant
  multi-version decoder, no `serde(default)` added (no field added). One
  non-blocking documentation imprecision (seed = u32 truncation of blake3, plan
  says full blake3) to fix in the test/commit doc-comment.

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks / carry-over:
  - **Plan line-reference drift:** §D.1 cites `engine/runtime.rs:1260-1285` for
    the verifiable temp=0+seed contract; the real location is `1323-1348`. §D.3
    cites `dispatch_loop.rs:155-303` for the test base — that module's E2E is the
    single-worker pump; the **cleaner home for the redundancy>1 quorum assertion
    is `result_sync.rs:360-518`** (it already wires the full
    dispatch->sync->validator_loop->coordinator-DB chain, so the quorum-`Accepted`
    outcome is directly assertable). Recommend the implementer site the new test
    in `result_sync.rs` (cross-node, validator-DB observable) and/or add a
    two-engine variant near `dispatch_loop.rs`'s pump. Documentation/placement
    only; no design impact.
  - **`seed` naming:** state the `u32` truncation honestly (above).
  - **LIVE acceptance (palier 2 quorum) deferred to Phase G** — same posture as
    B-3 in Phase C (material on the user's hardware). The WAN-convergence /
    consensus measurement is an OBSERVATION; >30s convergence is a BLOCK to
    diagnose, not a timeout to inflate (kickoff §C ref S75 SeedAnnounced
    peer_count:0). Tracked, not silently dropped.
  - **Cross-GPU heterogeneous exact-match divergence** is expected, NOT a bug;
    the acceptance writes both outcomes (anti faux-vert). True cross-hardware
    verification = TOPLOC étage 2 = `llm_llama_cpp` / S77. Design note only.
- Scope cuts still honored (kickoff §7 / plan §D.5): TOPLOC étage 2 = design note
  NOT codé; cross-GPU heterogeneous = post-S77; validator INCHANGE (diff vide);
  no push scheduler / no synchronous RPC / no custom DHT; iroh 0.98 pinned; zero
  wire bump; zero new dependency; kudos non-monetary (untouched this phase).

## Action
- **EXECUTE (scope amended, PO Option A 2026-06-17)**: FIRST land the root-cause
  bridge fix (`result_sync.rs` `forward_result_entry` dedup key →
  `(task_id, worker_pubkey)`, mirroring the validator), THEN implement the four
  Rust tests — the cross-process redundancy>1 test now exercises the real bridge
  and is red-before-green against the fix. See the Addendum above for the full
  gap analysis, the fix, and the security note.
- Original analysis (still valid for everything except the now-corrected S4
  "no production change" claim): proceed with Phase D as planned. Implement the four Rust tests
  (cross-process redundancy>1 byte-identical, diverging-rejected,
  verifiable-seed-cross-worker-stable, validator-unchanged verrou), the LIVE
  acceptance checklist (deferred to Phase G), and the TOPLOC étage-2 design note
  (PATTERNS rust + one THREAT_MODEL compute-quorum row). Keep
  `validate_quorum_pre_guardrail` INCHANGE (grep-diff verrou). Apply the two
  non-blocking documentation corrections: (1) site the redundancy>1 quorum test
  in `result_sync.rs` (validator-DB observable) rather than only the
  single-worker `dispatch_loop.rs` pump, and cite the real determinism contract
  at `runtime.rs:1323-1348`; (2) describe the verifiable seed as the `u32`
  truncation of `blake3(task_id)`, not the full digest. No pivot proposal is
  needed (no DESIGN-CONFLICT). The commit body does not need to cite a plan
  adaptation (this is EXECUTE, not PLAN-ADAPT) but SHOULD note the two doc
  corrections under `## Verification` or `## Pre-launch protocol`.
