# Sprint 76 Phase C Review

## Verdict: PASS

Driver-side deep review (independent process, no execution-session history),
promoted to PASS after Codex (GPT-5.5) cross-model verification + reconciliation
(see `## Codex reconciliation` below). Initial review found two P2
acceptance-script defects + one P3; zero P0/P1; verification green. The two P2
were fixed in-phase before Codex; Codex round-tripped to a clean 8/8.
Rigor signal G4 honored: real trade-offs surfaced, not a quota.

---

## Scope And Staging

Working tree, 11 Rust files + 1 new acceptance script + the preflight artifact.
Atomic and coherent for one phase. Staging buckets:

- **Phase code (signed wire + claim-gate)**: `nexus-core-rs/src/task.rs`
  (+260, `RuntimeTuple` + `Task.required_runtime` + builder + canonical/wire/
  signature tests + `model_digest` doc), `nexus-core-rs/src/lib.rs` (re-export),
  `nexus-worker-core/src/llm/{mod.rs,ollama.rs}` (`runtime_tuple` accessor),
  `nexus-worker-core/src/engine/runtime.rs` (claim-gate + `model_name_digest`
  seam + 3 tests), `nexus-coordinator-rs/src/{types.rs,dispatcher.rs}`
  (`TaskSubmission.required_runtime` + conditional set + 1 test).
- **Mechanical test-literal completions** (additive field, `None`):
  `nexus-coordinator-rs/src/db.rs`, `nexus-shell-daemon/src/{dispatch_loop.rs,
  http.rs,result_sync.rs}` — 1 line each, all in `mod tests`. Necessary for
  compile; not scope leak.
- **Acceptance deliverable**: `scripts/acceptance/b3_live_pc_vps.sh` (new).
- **Planning**: `sprint76_phase_c_preflight.md` (untracked, commits with phase
  per convention; the commit body MUST cite it for the PLAN-ADAPT deviation).

No accidental files, no build output, no unrelated refactor. `git diff --cached
'*.rs' | rg '^\+pub mod '` => none (no module-graph change). Module coherence OK.

The diff faithfully follows the **PLAN-ADAPT** (not the original plan): advert
moved from `capability_store.rs` (a 6-flag feature-toggle store, wrong substrate)
to a signed `Task.required_runtime`; routing moved from a (nonexistent)
dispatcher assignment loop to the worker CLAIM-GATE (PULL-correct);
`model_digest` GGUF-hash → doc-note. All three pivots match the preflight
`## Plan Adaptation` verbatim.

## Three-Block Verification

Independently re-run (not trusted from the prompt):

- **Rust (touched crates)** — `cargo nextest run -p nexus-core-rs
  -p nexus-worker-core -p nexus-coordinator-rs --locked`:
  **835 passed, 0 skipped**. The 10 new Phase C tests confirmed by name filter
  (`-E 'test(cohort_gate) or test(required_runtime) or test(runtime_tuple) or
  test(model_digest_is_name) or test(submit_sets_required_runtime)'`):
  **10 passed**.
- **Clippy** — `cargo clippy -p nexus-core-rs -p nexus-worker-core
  -p nexus-coordinator-rs --all-targets --locked -- -D warnings`: exit 0,
  **0 warnings**.
- **Full workspace (driver-reported, accepted)**: Windows fmt 0 / clippy 0 /
  nextest 1785/1785 0-skip / doctests 0 / release 0; Docker sbfb-ci canonical
  fmt + clippy + nextest 1789/1789 0-skip + doctests GREEN. The +4 Docker delta
  is the pre-existing `#[cfg(unix)]` set, consistent with prior sprints.
- **Frontend / Python**: N/A (0 web, 0 py in diff). Documented as acceptable —
  this is a Rust-only compute phase.

The anti-regression gate `e2e_network_execute_gate_real_http_no_frontier_mock`
lives in `nexus-shell-daemon/src/runtime.rs`, which is **NOT in the diff** —
gate untouched, runs green in the full workspace count.

## Delta Tests

`+10 Rust` (6 in `task.rs`, 1 dispatcher, 3 runtime). Plan §C.3 forecast `+4`;
the PLAN-ADAPT signed-field approach legitimately over-delivers (canonical
inclusion + signature + wire-roundtrip + serde-default tests that the original
`capability_store` advert would not have needed). Over-delivery, not a gap.

| Suite | Delta | Note |
|---|---|---|
| nexus-core-rs (task) | +6 | wildcard match (both sides), canonical inclusion, signed-identity, wire roundtrip, serde-default |
| nexus-coordinator-rs (dispatcher) | +1 | 3 internal branches asserted (verifiable+r>1 carries; r==1 drops; best-effort drops) |
| nexus-worker-core (runtime) | +3 | claim-gate ADMITS, claim-gate BLOCKS, model_digest doc-note pin |
| Frontend / Python | 0 | N/A — no web/py touched (acceptable) |
| Acceptance LIVE | n/a | `b3_live_pc_vps.sh` — manual, never-CI; WAN run deferred to operator hardware |

## Modified-File Branch Coverage

Semantic verification (tests read in full, not grep-matched). The three new
behaviors are genuinely exercised — **not tautological**:

1. **Claim-gate BLOCKS on mismatch** — `cohort_gate_blocks_non_homogeneous_worker`:
   real engine pump (`run_until_shutdown`), StubBackend reports family `"stub"`,
   task requires `"ollama"` → asserts task **stays live** (1 task entry,
   key `task:t-heterog`), **0 claim**, **0 result**. Generous rate budget
   isolates the cohort gate as sole cause. Virtual-time (current_thread,
   `pause`+`advance`) per the P2-A-1 / PATTERNS §P54 discipline — correct choice
   for a "nothing happens" assertion.
2. **Claim-gate ADMITS on match** — `cohort_gate_admits_homogeneous_worker`:
   same harness, require family `"stub"` (matches) → asserts **1 result**
   (`result:t-homog`) + **1 claim** via a real `result:` doc write.
   `multi_thread` real-time (waits on `result:`), matching the established
   pattern for pump-completion tests. The two tests are a true both-sides pair.
3. **Dispatcher conditional set** — `submit_sets_required_runtime_only_for_
   verifiable_redundant`: all three branches — (verifiable && r>1) carries the
   tuple AND the entry `verify_signature()` passes; (verifiable && r==1) drops to
   `None`; (best-effort && r>1) drops to `None`.

Supporting unit coverage:
- `RuntimeTuple::matches` wildcard-on-empty proven on **all three axes**
  (`runtime_tuple_empty_requirement_matches_any_worker` +
  `runtime_tuple_nonempty_field_must_match_exactly`): empty req = wildcard;
  family/quant/model each discriminate when pinned; both match and no-match
  asserted per axis. The one-line helper `runtime_field_matches` is fully
  covered.
- `runtime_tuple` accessor: Ollama arm (family `"ollama"`, quant `""`) and Stub
  arm (`with_runtime_tuple`) are exercised by the engine tests above; the trait
  default (pure wildcard) is the documented S77/llama_cpp shape.

No untested new business logic over ~10 lines. The `debug!` mismatch log line is
a defensive non-asserted branch (acceptable — main path covered).

## Security And Protocol

Red-line DEEP audit (signed canonical bytes + crypto identity touched):

- **`required_runtime` IS in the signed canonical bytes.** `task_canonical_bytes`
  (`task.rs:39-52`) removes only `redundancy_factor`; nothing strips
  `required_runtime`. Proven by `task_canonical_includes_required_runtime`
  (two tasks differing only by the tuple produce **different** canonical bytes,
  and the JCS body contains `"required_runtime"` + `"runtime_family":"ollama"`)
  and `task_entry_different_required_runtime_different_signature` (different
  signature; `verify_signature()` passes). This correctly follows the
  `verifiable` signed-identity precedent and the S23 `34c77ce` distinction
  (dispatch-policy vs identity). **Correct.**
- **Gate reads the field only AFTER signature verification.** `verify_signature()`
  at `runtime.rs:916` precedes the cohort gate at `runtime.rs:1046`. An
  application-level MITM cannot redirect a task to a heterogeneous cohort without
  breaking the coordinator signature. Documented in the field doc-comment and
  the signature test. **Correct.**
- **PULL-correctness / no side effects on mismatch.** The gate sits **between**
  the rate-limit gate (1001-1030) and `task_started_at = Instant::now()` (1061)
  + claim sign/write (1064). A mismatch `continue`s with **zero** side effects:
  no chrono start, no `ClaimEntry`, no result, task stays live — identical
  semantics to the rate-limit defer. **Correct.**
- **`model_digest` honesty.** Doc-comment (`task.rs` ResultPayload) + the
  `model_name_digest` seam + the `runtime_tuple`/Ollama comments all state the
  truth: `blake3(model NAME)`, not a weight-file hash; `Verifier` has 0 prod
  callers; live path is the quorum over `result_text`. `model_digest_is_name_
  hash_doc_note_s77` pins the contract (asserts `== blake3(name)` AND
  `!= blake3(pretend-weight-bytes)`), so any future switch to a file hash is a
  deliberate reviewed break. Nothing over-promises a weight attestation.
  **Honest.**
- **Validator INCHANGE.** `validator.rs`, `capability_store.rs`,
  `verification.rs`, `canonical.rs` are **not in the diff**. The cohort
  homogeneity is enforced 100% at the worker claim-gate; the quorum
  (`validate_quorum_pre_guardrail`) has zero awareness of `required_runtime` and
  remains the real trust boundary. The tuple advert is correctly documented as
  **advisory routing, not a trust boundary**. **Correct.**
- **No prod `unwrap()`/`panic!`/`unsafe`/`todo!` introduced.** All such tokens in
  the added lines are test-only. **Clean.**
- **HTTP deserialization complete.** `coordinator_submit_task`
  (`http.rs:3300`) takes `axum::Json<TaskSubmission>`; serde drives
  `required_runtime` with `#[serde(default)]`; full submission flows into
  `submit_task`. No manual plumbing missing. `task_wire_default_required_runtime_
  none` proves an omitting client JSON decodes to `None`, not a 422.

## Research And G8

G8 preflight present: `sprint76_phase_c_preflight.md`, verdict **PLAN-ADAPT**,
supervisor GO-PREFLIGHT. S1a/S1b/S2/S3/S4 all documented with context7 sources
(ollama-rs 0.3.4 `LocalModel`/`ModelInfo` carry no digest; Ollama `/api/show`
`quantization_level`/`family`) dated 2026-06-16. **Zero new dependency** (blake3
1.8.5 / sha2 0.10.9 already vendored; `cargo tree -d` shows only pre-existing
iroh/base64 duplicates). Crypto-adjacent work (signed Task field) is fully
research-grounded. The diff matches the preflight's corrected approach
faithfully. **G8 satisfied (PLAN-ADAPT honored).**

## Scope Cuts

Kickoff §7 / §C.5 scope cuts — all honored, verified against the diff:

- **No push-scheduler** — the dispatcher only signs+persists; routing is PULL
  claim-gate. ✓
- **No synchronous worker-to-worker RPC** — none added. ✓
- **No custom DHT** — iroh-docs/gossip path untouched. ✓
- **Cross-GPU heterogeneous = post-S77** — the tuple deliberately leaves
  `quant` empty (wildcard) for Ollama; real weight/quant attestation explicitly
  deferred to `llm_llama_cpp` / S77 (D3 étage 2), documented not silently
  dropped. ✓
- **Validator INCHANGE** — confirmed (file not in diff). ✓
- **iroh 0.98 pinned, zero wire bump, zero new dep** — `TASK_FORMAT_VERSION`
  stays 1 (additive `#[serde(default)]` field, pre-launch policy), no new DOMAIN
  constant, no dependency change. ✓

Pre-launch protocol: `required_runtime` is an additive signed field on v1; the
`#[serde(default)]` rationale ("runtime tolerance — minimal client decodes to
None, not historical compat") is written into the field doc, exactly per
CLAUDE.md policy.

## Codex verification

NOT YET RUN. Required before commit. Suggested Codex focus (the load-bearing
seams a second auditor should independently confirm):

1. `required_runtime` truly inside `task_canonical_bytes` and NOT stripped
   (the `obj.remove` touches only `redundancy_factor`).
2. Claim-gate ordering: after `verify_signature`, before any claim/chrono;
   `continue` leaves zero side effects.
3. `model_digest` doc-note does not over-promise (name-hash, Verifier dormant).
4. `required_runtime=None` default path is byte-for-byte the pre-S76 compute
   flow (consent → rate-limit → claim → generate → result → sign unchanged).
5. The two acceptance-script P2 defects below (independent reproduction).

Security delta: one NEW surface (worker self-asserted cohort tuple) added, NO
T0-T5 regression. Mitigation shipped: tuple lives in **signed** Task (a lie is
attributable, not anonymous MITM), and the unchanged exact-match quorum rejects
a divergent result as an outlier regardless of the advertised tuple. A
THREAT_MODEL "compute cohort homogeneity (advisory-not-trust)" row is owed at
Phase G (carry below).

## Commit Body Draft

```
feat(coordinator+daemon): Sprint 76 Phase C — cross-machine compute B-3 (live) + homogeneous cohort routing

## Contexte
D2 lève B-3 par acceptance LIVE scriptée (PC RTX 5080 worker réel ↔ VPS
coordinateur/ancre, redundancy=1), transport forcé iroh 0.98 + modèle S75
prouvé. D3 étage 1 pose le routing cohorte-homogène (pré-condition du quorum
déterministe Phase D). 1er critère falsifiable : convergence `result:` WAN
mesurée, >30s = BLOCK diagnostiqué. PLAN-ADAPT (cf.
sprint76_phase_c_preflight.md) : le plan proposait advert dispatcher-side +
capability_store + hash GGUF ; le preflight S1a a établi que l'archi est PULL
(claim-gate), que capability_store est un feature-toggle store, et qu'Ollama
n'expose aucun digest GGUF propre ; adapté en un champ signé Task.required_runtime
appliqué au CLAIM-GATE worker, model_digest gardé en name-hash + doc-note
(hash GGUF → llama_cpp/S77).

## Fichiers
- nexus-core-rs/src/task.rs : RuntimeTuple{model,quant,runtime_family} +
  matches() (wildcard-sur-vide) + Task.required_runtime (signé, #[serde(default)])
  + builder + doc-comment model_digest resserré + 6 tests.
- nexus-core-rs/src/lib.rs : ré-export RuntimeTuple.
- nexus-worker-core/src/llm/{mod.rs,ollama.rs} : LlmBackend::runtime_tuple
  (default wildcard ; Ollama family="ollama" quant="" ; Stub configurable).
- nexus-worker-core/src/engine/runtime.rs : claim-gate cohorte (continue
  sans claim sur mismatch, après verify_signature) + helper model_name_digest
  + 3 tests.
- nexus-coordinator-rs/src/{types.rs,dispatcher.rs} : TaskSubmission.required_runtime
  + pose conditionnelle (verifiable && redundancy>1) + 1 test.
- nexus-coordinator-rs/src/db.rs, nexus-shell-daemon/src/{dispatch_loop.rs,
  http.rs,result_sync.rs} : littéraux de test complétés (required_runtime: None).
- scripts/acceptance/b3_live_pc_vps.sh : harness SSH PC↔VPS (manuel, jamais CI).

## Delta tests
+10 Rust (6 task + 1 dispatcher + 3 runtime). Win nextest 1775→1785,
Docker 1779→1789. Frontend/Python N/A (0 web/py).

## Verification
Windows : fmt 0 · clippy 0 · nextest 1785/1785 0-skip · doctests 0 · release 0.
Docker sbfb-ci : fmt + clippy + nextest 1789/1789 0-skip + doctests GREEN.
Gate e2e_network_execute_gate_real_http_no_frontier_mock non touché, vert.
Acceptance LIVE B-3 : run WAN DIFFÉRÉ au matériel opérateur (env dev sans
VPS/PC) — comme S74 a différé son re-run dual à recovery ; script livré, trace
à consigner sprint76_verification.md au run.

## Scope cuts
Pas de scheduler push, pas de RPC synchrone, pas de DHT custom, cross-GPU
hétérogène = post-S77, validator INCHANGÉ, iroh 0.98 pinné, 0 bump wire, 0 dep.

## G8 traceability
Preflight sprint76_phase_c_preflight.md PLAN-ADAPT, GO-PREFLIGHT superviseur.
S1a/S1b/S2/S3/S4 sourcés (context7 ollama-rs 0.3.4, Ollama /api/show, 2026-06-16).
0 dep nouvelle ; blake3/sha2 déjà vendored.

## Pre-launch protocol
0 bump TASK_FORMAT_VERSION (champ additif signé sur v1), 0 nouveau DOMAIN.
required_runtime #[serde(default)] = tolérance runtime (client minimal → None,
pas 422), pas compat historique. model_digest/logprobs_hash existaient déjà v1.

## Codex verification
<à compléter — output brut codex exec -o, jamais réécrit>

## Carry closure / Unblock
B-3 LEVÉ (acceptance LIVE livrée, run WAN différé matériel). Débloque le quorum
déterministe Phase D (palier 2 réutilise le harness + le claim-gate cohorte).
Carries ouverts (vers sprint76_verification.md + audit gate suivant) : voir
Findings P2/P3 ci-dessous.
```

## Findings

**P2-ACCEPT-SCRIPT-WIRE-FIELD** (`scripts/acceptance/b3_live_pc_vps.sh`, step 2):
the invite extraction `sed -n 's/.*\"\(invite\|token\)\":\"...'` targets a JSON
key that does not exist. `POST /api/v1/invite/create` returns the token under
`"wire"` (`invite_api.rs:154-162`: `{"id","wire","scope","project_id",...}`),
NOT `"invite"`/`"token"`. As written the script extracts nothing and aborts with
`die "invite/create returned no token"` on every real run. Fix: match `"wire"`.
The token VALUE is correct (`rec.wire` is exactly what `Invite::decode` /
`nexus-worker join` expects) — only the key name is wrong. Owner: driver.
Trigger: any operator run. Exit: change the sed alternation to `wire` (+ re-grep
the live response shape).

**P2-ACCEPT-SCRIPT-WORKER-SUBCMD** (`scripts/acceptance/b3_live_pc_vps.sh`,
step 3 + the manual hint): the script invokes `"$WORKER_BIN" run &` and prints
`nexus-worker join $INVITE && nexus-worker run`. The worker CLI has **no `run`
subcommand** — the engine starts via `Start` (`cli.rs:100`, no
`visible_alias("run")`); clap would reject `run` with "unrecognized
subcommand". Correct invocation is `nexus-worker start --headless` (the
`--headless` flag also matters: backgrounded with `&`, `Start` may otherwise
attach the ratatui TUI on a terminal stdout). Owner: driver. Trigger: any
operator run with WORKER_BIN set. Exit: replace `run` with `start --headless`
in both the call and the printed hint.

These two are non-blocking for PASS-PENDING because (a) the LIVE acceptance is
explicitly DEFERRED to operator hardware (no VPS/PC in this env), exactly as S74
deferred its dual re-run to recovery before push, and (b) they are pure script
text with no effect on the compiled/tested compute path. They are, however,
real defects in a committed deliverable and would burn an operator cycle on
first run. **Recommend fixing both before commit** (cheap, and the script is the
B-3 deliverable). If not fixed pre-commit they MUST carry into
`sprint76_verification.md` as blocking-for-the-LIVE-run items, since a green
PASS-PENDING here must not be read as "the acceptance script runs".

**P3-THREAT-MODEL-COHORT-ROW** (THREAT_MODEL.md): the new worker-self-asserted
cohort tuple is a NEW surface. The preflight S3 prescribes a Phase-G
THREAT_MODEL row documenting "compute cohort homogeneity = advisory routing,
not a trust boundary; the exact-match quorum stays the real defense". Not owed
in Phase C, but must land at Phase G. Owner: Phase G wrap-up. Trigger: Phase G.
Exit: THREAT_MODEL compute-cohort row written.

## Residual Risk

- **The LIVE B-3 acceptance is unproven in this environment.** The script's
  falsifiable <30s convergence criterion (BLOCK, no timeout inflation) is
  well-encoded and references the S75 `SeedAnnounced peer_count:0` precedent
  honestly — but the WAN run is deferred to operator hardware, and the two P2
  defects mean the script as committed would not run end-to-end without the
  one-line fixes. The deferral itself is acceptable for PASS-PENDING (S74
  precedent); the script-text defects are the residual risk and should be
  cleared before the operator attempts the run.
- **Cohort tuple is advisory.** A worker lying about its tuple to enter a cohort
  is caught only by the downstream exact-match quorum, not by the gate. This is
  by design and documented; the quorum (unchanged) is the real boundary. No
  regression, but the THREAT_MODEL row (P3) must record it so the gate is never
  mistaken for a security control.
- **`quant` is a wildcard for Ollama** (`""`), so two Ollama workers running the
  same model name at different quantizations would both pass the cohort gate and
  could split the quorum. This is the honest limit of the Ollama black box,
  explicitly deferred to S77/llama_cpp, and the quorum still rejects the
  divergent result as an outlier — so correctness (not just routing) is
  preserved. Acceptable for D3 étage 1.

---

**Process note (independence / G4):** this is a driver-side deep review produced
as an independent process with no execution-session transcript. It does not
ratify the diff; each choice was challenged and the signed-bytes / claim-gate /
doc-note seams were independently re-derived and re-tested. The verdict is
`PASS-PENDING` — Codex must run and reconcile, then the auditor writes exact
`## Verdict: PASS`. It is not committable as-is.

---

## Codex reconciliation

Codex (GPT-5.5, `codex exec`) cross-model verification was run on the working
tree (raw output in `sprint76_phase_c_codex_review.md`, never rewritten). Three
rounds:

- **Round 1 — 6 CONFIRME / 0 GAP / 2 PARTIEL.** No P0/P1. The 2 PARTIEL were
  documentation-honesty items, not functional defects:
  - L5 (model_digest): the worker code was confirmed correct (name-hash via
    `model_name_digest`, test pins `== blake3(name)` and `!= blake3(weight-bytes)`,
    zero GGUF hash coded) — but `verification.rs:17` doc + `.planning/codebase/*`
    still said "model weights file", over-promising a weights attestation the
    doc-note disclaims.
  - L8 (acceptance script): the measurement was labelled as the `result:`
    replication delay but actually measures submit -> result-visible (end-to-end).
- **Resolution (root-cause, no carry):**
  - L5: tightened `verification.rs` (module Layer-2 doc + `digest_whitelist` field
    doc) to the honest name-hash framing (Verifier dormant, live path = quorum over
    `result_text`, weights digest gated on `llm_llama_cpp`/S77) + corrected the two
    tracked codebase maps (`protocol_wire_formats.md`, `security_posture.md`). A
    repo-wide grep confirmed 0 residual "model weights" over-promise in the main
    tree (the `.claude/worktrees/*` copies and the `*_rnd.md` sharding docs are
    out of scope — the latter reference actual weights, correctly).
  - L8: relabelled the script (header + measurement comment + verdict log) to state
    honestly that `DELAY` is the end-to-end submit -> VPS-visible time, an UPPER
    BOUND on the `result:` WAN convergence (claim + inference is a few seconds for
    the tiny deterministic prompt, so a >30s delay implicates WAN convergence).
- **Round 2 — 7 CONFIRME / 0 GAP / 1 PARTIEL.** L8 resolved. L5 still PARTIEL on a
  *second* doc occurrence Codex surfaced: `protocol_wire_formats.md:553` ("BLAKE3
  hash of model weights file"). Fixed (1-line doc correction) + repo-wide re-grep
  confirmed clean.
- **Round 3 — 8 CONFIRME / 0 GAP / 0 PARTIEL (CLEAN).** All deliverables confirmed;
  invariants re-confirmed (TASK_FORMAT_VERSION=1, no new DOMAIN/dep, validator and
  `e2e_network_execute_gate_real_http_no_frontier_mock` untouched).

The two review-deep P2 (acceptance-script: token under `wire` key, `nexus-worker
start --headless`) were fixed in-phase before Codex round 1, and Codex confirmed
the fixes (round-1 L8 noted the correct routes/subcommand).

Suites after the doc-only fixes: Windows §7.4 GREEN (fmt/clippy/nextest
1785/1785/doctests/release all 0). Docker canonical passed 1789/1789 on the
functional code (pre-doc-fix round); the Docker re-run on the doc-only delta is
env-blocked by a Docker Desktop Linux-engine wedge (500 on `/_ping`, the known
WSL-wedge), deferred to engine recovery before push — the doc-only delta
(comments + `.md` + `.sh`) is platform-agnostic and Windows-confirmed.

The P3 (THREAT_MODEL compute-cohort "advisory routing, not trust boundary" row)
is routed to Phase G per the preflight.
