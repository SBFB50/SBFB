<!-- ==== Codex GPT-5.6 Sol (reasoning max) — Round 1 : raw `codex exec -o` output ==== -->

Overall verdict: **GAP**. No P0 wire bump, dependency change, refactor, or committed filesystem-path leak is visible. However, the T3 harness has multiple **P1 false-green paths**, so Phase B should not receive an external `PASS`.

## New P1 findings

1. **P1 — A cold/incomplete benchmark rig can emit `PASS`.**

   Preflight requires only the model and `llama-bench` (`scripts/acceptance/benchmarks_standards.sh:190-205`). Missing `perplexity`/wikitext merely logs “skipped” (`:276-288`), and a missing or invalid B3 artefact merely produces `sharded: null` (`:294-318`). The script then unconditionally emits `PASS` after a valid head pp/tg result (`:408-412`).

   Therefore, model + llama-bench alone yields `PASS` with both `sharded` and `perplexity_parity` null. This directly contradicts the script’s own cold-rig contract (`:31-45`) and `SHARD_BENCHMARKS.md:153-160`.

2. **P1 — Sharded metrics can be stale, mismatched, incomplete, or BLOCKed and still enter a PASS artefact.**

   The B3 file is loaded without checking its `status`, model name/hash, quant, `n_shards`, proof, or correspondence to the current run (`benchmarks_standards.sh:294-317`). Presence of numeric `tokens` alone creates the `sharded` object, while all five fine metrics may remain null (`:374-384`).

   This can combine a current llama-bench baseline with an unrelated old sharded result. `b3_shard_pipeline.sh:427-441` extracts the new metrics but its final verdict path around `:484-490` does not require them.

3. **P1 — Artefact write failure still exits zero.**

   `emit_and_exit` suppresses directory-creation errors, does not check the JSON write or subsequent `cat`, then executes the requested `exit 0` (`benchmarks_standards.sh:126-168`). A non-writable `BENCH_ARTIFACT` therefore produces a false-green gate without the required parseable artefact.

4. **P1 provenance weakness — `PASS` does not require the claimed pins.**

   `LLAMACPP_COMMIT` and `QUANT` default to `unknown` (`benchmarks_standards.sh:73-79`), while model/corpus BLAKE3 is best-effort and becomes null without `b3sum` (`:110-121`, `:385-405`). The gate can thus pass without the exact llama.cpp snapshot or content hashes that supposedly make runs comparable.

## Per-deliverable verdicts

| Deliverable | Verdict | Evidence and assessment |
|---|---|---|
| Host-side metrics + view/projection | **PARTIAL** | Metric implementation itself is sound: timestamps are captured per reply (`shard_session.rs:1416-1421`, `:1563-1568`), TTFT is excluded through `windows(2)`, and TPOT/p50/p95 are derived deterministically (`:1621-1637`). Outcome/result propagation is present (`:280-297`, `:349-361`, `:502-513`) and HTTP projection is complete (`http.rs:2416-2427`). Signed `RunMetrics` changes only documentation (`shard_plan.rs:389-404`); signed schemas are description-only. **But:** the five Rust fields are required `Option<u64>` values documented to serialize as null (`schemas/shard.rs:152-193`), while both generated Draft-2020-12 schemas declare them required and `type: integer`, without allowing `null` (`shard_session_result_view.schema.json:4-71,87-96`; response schema `:10-76,94-103`). In-progress responses therefore violate their advertised JSON schema. P2 loopback-contract defect, not a signed-wire bump. |
| Hermetic tests | **PASS** | Pure percentile/TPOT behavior, empty/single-gap cases, nearest-rank ordering, and TTFT exclusion are pinned at `shard_session.rs:1704-1756`. The paced real decode exercises non-zero gaps and checks all five projected fields at `:2538-2556`, `:2656-2668`, and `:2693-2723`. No new P1 found here. |
| Harness + committed artefact | **GAP** | Fails the hard anti-false-green invariant through the P1 paths above. The committed JSON itself has a clean basename and no user path (`sprint82_t2_benchmarks.json:5`), but `model_blake3` is null and `llamacpp_commit` is unknown (`:6-9`), so it does not satisfy the stated “NAME + blake3” provenance invariant. It is an honest `BLOCK`, but not a reproducible benchmark baseline. |
| README T3 canon | **PARTIAL** | T3 vocabulary, opt-in rule, versioned artefact, and regression semantics are added at `docs/claude/README.md:615-629` and `:658-674`. Residual honesty issues remain: the surrounding prose still says “passe les trois” after declaring four tiers (`:654-655`), and the present-tense “Enforcement T3” claims Track-J and kickoff invariant `#16bis` enforcement that the supplied diff does not implement. This matches the disclosed non-blocking prompt/gate residual, but the wording overstates current mechanical enforcement. |
| `SHARD_BENCHMARKS.md` note | **PARTIAL** | Strong and honest on PPL-sharded being unwired (`docs/protocol/SHARD_BENCHMARKS.md:64-83`) and on the non-signed 0-bump metrics boundary (`:85-107`). However, its determinism claim (`:109-121`) is stronger than the harness, which accepts unknown/null provenance, and its rig-gate claim (`:153-160`) is false because missing PPL, corpus, or B3/shard-session data can still lead to `PASS`. The harness header also still says PPL-sharded “is emitted … and compared” (`benchmarks_standards.sh:15-17`), contradicting the later honest Phase-B disclaimer. |

Hard-invariant summary:

- Signed SBFB wire unchanged: **PASS**
- Cargo/runtime dependencies unchanged in the supplied complete diff: **PASS**
- Refactor = 0: **PASS**
- Cold rig never false-PASS: **FAIL, P1**
- Committed model filesystem-path hygiene: **PASS**
- Committed NAME + BLAKE3 provenance: **FAIL**
- Overall external cross-check gate: **GAP / not approvable as PASS**
<!-- ==== Codex GPT-5.6 Sol (reasoning max) — Round 2 : raw `codex exec -o` output ==== -->

## Round 2 verdict: GAP

No P0 found and no unrelated new P1 identified. However, the original false-green class is not fully closed: P1-2 remains open, which also prevents certifying P1-1, and P1-4 has a concrete bypass.

### P1 closure

| Finding | Verdict | Cross-check |
|---|---|---|
| P1-1 cold rig could PASS | PARTIAL | Missing b3 metrics now correctly causes `BLOCK{rig}`. However, a stale or incomplete same-name b3 artifact still satisfies the PASS gate, so a cold current shard rig can still PASS using old data. |
| P1-2 stale/mismatched b3 | GAP | Status and model name are checked, but only `ttft_ms` and `tokens` are required. TPOT, ITL p50/p95, throughput, types, model content, `n_shards`, and freshness/run identity are not validated. |
| P1-3 write failure exit 0 | PASS | The write result and non-empty file are checked; failure exits 2 with `FATAL`. |
| P1-4 PASS without provenance | GAP | A failed `b3sum` can yield an empty string, which passes `MODEL_B3 != unavailable` and becomes `model_blake3: null`. Any non-literal value such as `LLAMACPP_COMMIT=foo` also passes. |

### Residual P1s

**P1-A — incomplete/stale b3 data can still produce PASS**

This artifact passes the validator:

```json
{
  "status": "PASS",
  "model": "codellama-34b.gguf",
  "ttft_ms": 1,
  "tokens": 16
}
```

The validator only checks:

```python
d.get("ttft_ms") is not None and d.get("tokens") is not None
```

The final PASS gate likewise checks only `SH_TTFT_MS` and `SH_TOKENS`. Consequently, the emitted PASS artifact can contain:

- `tpot_ms: null`
- `itl_p50_ms: null`
- `itl_p95_ms: null`
- `decode_milli_tokens_per_sec: null`

Additional false-green variants remain:

- Non-numeric strings satisfy the non-`None` and shell non-empty checks. They can produce `sharded: null` while the top-level status remains `PASS`.
- A b3 artifact from another `n_shards` value is accepted, then relabelled with the current `N_SHARDS`.
- A same-basename model with different contents is accepted because b3 carries no comparable model hash.
- There is no timestamp, run ID, session binding, or other freshness mechanism. A genuinely stale same-model artifact is indistinguishable from a paired run.

**P1-B — provenance gate accepts an absent or malformed pin**

With `b3sum` installed but failing, `blake3_of` can return an empty value. This condition still succeeds:

```bash
[ "$MODEL_B3" != "unavailable" ]
```

`build_artifact` then converts the empty value to `null`, allowing a nominal PASS without the promised model content pin. The commit gate similarly checks only inequality with the exact lowercase string `unknown`.

### Per-deliverable verdicts

| Deliverable | Verdict |
|---|---|
| Rust TTFT/TPOT/ITL instrumentation, projection, and tests | PASS |
| Additive result-view fields and generated schemas | PASS |
| Signed `RunMetrics`/`RunProof` honesty descriptions | PASS |
| `b3_shard_pipeline.sh` metric emission and path redaction | PARTIAL — metrics are emitted, but the artifact lacks the provenance/run binding required by the consumer’s “current validated run” claim |
| `benchmarks_standards.sh` | GAP — residual P1 false-green paths |
| `docs/protocol/SHARD_BENCHMARKS.md` | PARTIAL — PPL limitations are honestly documented, but the stated all-fine-metrics/provenance enforcement is stronger than the implementation |
| `docs/claude/README.md` T3 amendment | PASS |
| `rig.local.env.example` | PASS |
| Committed `sprint82_t2_benchmarks.json` BLOCK artifact | PASS — honest BLOCK, parseable, and no filesystem path |
| Overall Sprint 82 Phase B Round 2 | GAP |

The signed-wire, zero-dependency, additive-view, privacy, and never-`RIG-ABSENT` invariants hold within the supplied bundle. The “cold rig never false-PASS” invariant does not yet hold.

Closure requires validating every required sharded metric as a correctly typed value, comparing b3 `n_shards` and model blake3 with the current run, adding an explicit freshness/run binding if “stale” must be rejected, and accepting provenance pins only when they match valid hash formats and were successfully computed.
<!-- ==== Codex GPT-5.6 Sol (reasoning max) — Round 3 FINAL : raw `codex exec -o` output ==== -->

## Round 3 verdict: PASS at the P0/P1 gate

P1-A is **CLOSED**. P1-B is **CLOSED**. No new P0/P1 is visible in the supplied diff. Only P2/P3 limitations remain.

This is a static cross-check of the inline bundle and stated mutation tests; no repository commands were rerun.

| Deliverable | Verdict | Assessment |
|---|---|---|
| Signed-metric honesty note and schemas | PASS | Correctly documents that signed `p95_token_latency_ms` is a mean without changing signed bytes. |
| Host-side TTFT/TPOT/ITL instrumentation | PASS | Correct definitions, first token excluded from ITL, nearest-rank math pinned, and real decode wiring tested with paced replies. |
| Additive result-view exposure | PASS | Five metrics are projected through registry, HTTP response, Rust schema, and JSON snapshots. |
| `b3_shard_pipeline.sh` producer | PASS | Emits basename plus model blake3 and all fine metrics; missing hash becomes null rather than a usable pin. |
| Standards benchmark/PASS gate | PASS | Requires baseline, validated current sharded metrics, valid commit syntax, valid model digest, assembled JSON, and successful write. |
| Versioned committed artifact | PASS as gate evidence | It is an honest `BLOCK{rig}` artifact, not fabricated benchmark evidence. Hot-rig measurements remain absent. |
| Perplexity parity | PARTIAL | Whole-model PPL is optional; sharded PPL and delta remain explicitly unwired. |
| T3 process integration | PARTIAL | Canonical rule is documented, but dedicated audit-prompt and kickoff enforcement remain explicit P3 follow-up. |
| Gaps | None | No deliverable has a P0/P1-level GAP. |

### P1-A closure

The b3 metrics become usable only when the atomic Python predicate verifies:

- exact `status == "PASS"`;
- model basename match;
- exact content-hash match;
- `n_shards` match;
- all five fine metrics plus `tokens` are genuine Python integers, excluding booleans;
- artifact freshness through `B3_MAX_AGE_MIN`.

Failure leaves the shard variables empty, and the final PASS gate requires validated sharded values. Even a manually forged `"model_blake3":"unavailable"` could not produce PASS because the later `is_blake3 "$MODEL_B3"` gate independently rejects it.

### P1-B closure

The PASS path now correctly requires:

- `LLAMACPP_COMMIT`: lowercase hexadecimal only, minimum seven characters;
- model digest: exactly 64 lowercase hexadecimal characters;
- `blake3_of`: either a validated digest or the explicit `unavailable` sentinel.

Empty, malformed, uppercase, short, or unavailable pins cannot reach PASS.

### Hard invariants

- **0 signed/protocol wire bump:** PASS. Only the non-signed loopback result view changes additively; signed structures and format versions remain unchanged.
- **0 dependency:** PASS. No Cargo/npm dependency surface changes.
- **refactor=0:** PASS. Instrumentation is additive; existing signed computation remains byte-stable.
- **NAME + blake3 hygiene:** PASS for generated artifacts.
- **No `RIG-ABSENT`:** PASS. Rig-input failures route through `BLOCK` with a `rig:` diagnosis.
- **Cold rig never false-PASS:** PASS. Missing tool/model, invalid baseline, invalid b3 input, missing provenance, JSON assembly failure, or write failure cannot return PASS.

Remaining non-blockers:

- **P2:** `LLAMACPP_COMMIT` is syntactically validated but still operator-asserted; it is not cryptographically bound to the actual benchmark executable.
- **P2:** freshness remains an acknowledged mtime proxy, bypassable by touching the file or clock anomalies.
- **P2:** semantic ranges are not revalidated for imported metrics—negative values, ordering, and positive-token constraints rely on the trusted Rust/b3 producer.
- **P2:** `assert_no_fs_path` is a targeted detector, not a universal absolute-path recognizer; constructive model redaction is what currently enforces the main invariant.
- **P3:** no final strict JSON parse-back after writing.
- **P3:** sharded PPL/delta and automated T3 kickoff/audit enforcement remain future work.
- **Pre-existing, excluded:** required `Option<u64>` schema fields still omit `null`, as already documented from S81-I.

**Final external gate: PASS — residual P1 count: 0; new P0/P1 count: 0. Only P2/P3 items remain.**