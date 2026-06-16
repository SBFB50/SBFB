# Sprint 76 Phase C Preflight

Date: 2026-06-16
HEAD: `6904cdd`
Verdict: **PLAN-ADAPT**

> Volet 1 (acceptance LIVE B-3) = EXECUTE as written. Volet 2 (routing cohorte
> homogene) carries the blocking S1a finding that forces PLAN-ADAPT: the plan's
> mechanism description ("le dispatcher n'assigne les replicas qu'aux workers
> homogenes", `capability_store.rs`, `model_digest` = GGUF file hash) does not
> match the real PULL architecture nor a feasible Ollama digest path. The
> corrected approach is documented in `## Plan Adaptation`. No Day-0 is touched;
> the validator stays INCHANGE; zero wire bump.

## Evidence Rules
- Claim policy: every claim cites a path, a command output, a URL+date, or an
  explicit assumption.
- Local sources read: `prompts/agent/preflight.md`; `.planning/active/sprint76_plan.md`
  (§6 Phase C L307-383, §7 Phase D L387-419); `.planning/active/sprint76_kickoff.md`
  (§1, §4 D3, §11 implied via D3 gel); `.planning/active/sprint76_design_review.md`
  (D3 review L59-97); `crates/nexus-core-rs/src/task.rs` (full);
  `crates/nexus-core-rs/src/verification.rs` (full);
  `crates/nexus-coordinator-rs/src/capability_store.rs` (full);
  `crates/nexus-coordinator-rs/src/dispatcher.rs` (L1-140);
  `crates/nexus-coordinator-rs/src/validator.rs` (L200-360, quorum);
  `crates/nexus-worker-core/src/engine/runtime.rs` (L850-1130 claim/exec/digest);
  `crates/nexus-worker-core/src/llm/mod.rs` (trait L300-375);
  `crates/nexus-worker-core/src/llm/ollama.rs` (L1-360);
  `crates/nexus-shell-daemon/src/dispatch_loop.rs` (L130-304 worker-pump E2E);
  `crates/nexus-shell-daemon/src/result_sync.rs` (L1-60);
  `crates/nexus-core-rs/src/canonical.rs` (DOMAIN/_VERSION grep);
  memory `feedback_approach.md`, `feedback_context7_systematic.md`.
- Commands run (relevant outputs inline below): `git rev-parse --short HEAD` =>
  `6904cdd`; `grep -A1 '^name = "ollama-rs"' Cargo.lock` => `0.3.4`;
  `grep '^name = "blake3"/"sha2"' Cargo.lock` => blake3 1.8.5 / sha2 0.10.9
  (both already vendored); `grep -c '^name = "gguf' Cargo.lock` => `0` (no GGUF
  crate); `cargo tree -d` => only the pre-existing `base64`/iroh duplicates, none
  introduced by this phase; `grep -rn "Verifier::new|Verifier::default|Verifier {"`
  outside `verification.rs` => **0 hits** (Verifier never instantiated in prod);
  `grep -rn "runtime_family|quant|homogene|cohort|capability_tuple"` in crates =>
  **0 worker-capability hits** (all "advertise" hits are node-directory).
- context7: `/ollama/ollama` (`/api/show` details: `quantization_level`,
  `family`, `format`, modelfile `FROM .../blobs/sha256:...`); `/pepperoni21/ollama-rs`
  (`LocalModel { name, modified_at, size }` — NO digest; `ModelInfo { license,
  modelfile, parameters, template, model_info, capabilities }` — NO top-level
  digest field), accessed 2026-06-16.

## Scope
- Plan source: `.planning/active/sprint76_plan.md` §6 (Phase C, L307-383).
- Target files (plan-stated): `crates/nexus-worker-core/src/engine/runtime.rs:1082`
  (`model_digest`); `crates/nexus-coordinator-rs/src/capability_store.rs`;
  `crates/nexus-coordinator-rs/src/dispatcher.rs:37-133`;
  `crates/nexus-worker-core/src/engine/runtime.rs:3629` (gate, real name at
  `crates/nexus-shell-daemon/src/runtime.rs:3965`); acceptance LIVE script (SSH).
- Target files (corrected, see Plan Adaptation): `crates/nexus-core-rs/src/task.rs`
  (Task new field, ResultPayload), `crates/nexus-worker-core/src/engine/runtime.rs`
  (claim-gate + digest doc/impl), and the cohort-gate point of application
  (worker claim-gate, NOT `dispatcher.rs:submit_task`).
- Deps/APIs/specs: ollama-rs 0.3.4 (`/api/tags`, `/api/show`); blake3 1.8.5,
  sha2 0.10.9 (both already in workspace). **Zero new dependency.**
- Security/protocol surfaces: `Task` (signed canonical, DOMAIN_TASK_V1),
  `ResultPayload.model_digest` (signed, DOMAIN_RESULT_V1), `Verifier` layer-2/3
  (dormant), `validate_quorum_pre_guardrail` (INCHANGE constraint), claim-gate.
- Tests expected (plan §C.3): `capability_advertises_homogeneity_tuple`,
  `dispatcher_routes_replicas_to_homogeneous_cohort`,
  `model_digest_hashes_gguf_file_or_documented`,
  `e2e_network_execute_gate_real_http_no_frontier_mock` (existing gate stays
  green), acceptance LIVE `b3_live_pc_vps_result_rendered`.

## S1a OSS Prior Art
- Domain: (1) cross-machine compute task-routing E2E (BOINC/Folding/Petals);
  (2) attesting *which model* a remote worker ran (compute-verification + LLM
  fingerprinting).
- Sources:
  - BOINC JobReplication / `min_quorum`/`target_nresults` consensus (kickoff §D2,
    `github.com/BOINC/boinc/wiki/CreditNew`, 2026-06-15) — redundancy + canonical
    consensus is the mature pattern; SBFB's `validate_quorum_pre_guardrail`
    mirrors it. **APPROACH-ALIGNED** for Volet 1.
  - Ollama `/api/show` (context7 `/ollama/ollama`, 2026-06-16): `details`
    exposes `quantization_level` (e.g. `Q4_0`), `family`, `format=gguf`, and the
    modelfile `FROM /.../blobs/sha256:<hex>` line. The model blob digest is the
    SHA256 of the GGUF, retrievable only by string-parsing the modelfile.
  - ollama-rs 0.3.4 `LocalModel` (context7 `/pepperoni21/ollama-rs`, 2026-06-16):
    `{ name, modified_at, size }` — NO digest field. `show_model_info()` ->
    `ModelInfo` has NO top-level `digest`; only the embedded `modelfile` string
    carries the `sha256:` blob ref.
  - Distributed-inference model attestation state of the art (WebSearch
    2026-06-16): the robust mechanisms are TEE-based hardware attestation
    (Red Hat confidential computing, 2025-10) and adversarial LLM fingerprinting
    (iSeal AAAI 2025; Instructional Fingerprinting). There is **no simple,
    black-box "file digest of the loaded weights" primitive** for an HTTP API
    like Ollama. https://next.redhat.com/2025/10/23/enhancing-ai-inference-security-with-confidential-computing-...,
    https://ojs.aaai.org/index.php/AAAI/article/view/40909
- Finding:
  - Volet 1 (acceptance LIVE B-3): **APPROACH-ALIGNED** (redundancy/consensus is
    BOINC-standard; the path is unchanged). Non-blocking.
  - Volet 2 model_digest "GGUF file hash via Ollama": **APPROACH-NAIVE / not
    cleanly feasible**. The plan offers "durcir -> hash du fichier GGUF (P1)".
    OSS evidence shows the worker (Ollama backend) has NO clean digest accessor:
    `LocalModel` lacks it, `ModelInfo` lacks it, and `LlmBackend` (trait at
    `llm/mod.rs:315`) exposes only `healthcheck()`+`generate()` — neither returns
    a file digest. The only Ollama route to a real SHA256 is fragile string
    extraction from the modelfile `FROM` line, which breaks the hermetic
    `StubBackend` path (a stub has no GGUF file, no blob store). **This is the
    blocking S1a finding -> PLAN-ADAPT.**
  - Volet 2 cohort tuple advertise: **APPROACH-NOVEL** (justified by Ollama
    black-box + P2P PULL context). Constrain the *cohort* (name+quant+runtime
    family), not the hardware — consistent with the D3 sources (Ingonyama,
    Thinking Machines: cross-GPU exact-match is not guaranteed). Non-blocking on
    its own; the adaptation makes the mechanism concrete.
- Impact: adaptation required for Volet 2 (see `## Plan Adaptation`). Volet 1
  proceeds as written.

## S1b Dependencies, CVEs, Release Notes
- Scanned: ollama-rs 0.3.4 (existing pin, no bump), blake3 1.8.5, sha2 0.10.9.
- Commands/sources: `Cargo.lock` shows ollama-rs 0.3.4 (`grep -A1 '^name =
  "ollama-rs"'`). `grep -c '^name = "gguf'` = 0 — the phase must NOT add a GGUF
  reader crate (the adaptation avoids reading GGUF directly, eliminating that
  need). `cargo tree -d` shows only pre-existing duplicate stacks (`base64`
  0.21/0.22 via ron/hickory vs attohttpc/iroh; the standard iroh transitive
  set) — none introduced here. No new dep, no transitive collision (the S72
  ollama-rs->schemars 1.2 lesson does not recur: no dep is added or bumped).
- Finding: **clean**. Zero dependency change. blake3/sha2 already vendored; the
  tuple-advertise + claim-gate use only existing crates.

## S2 Historical Decisions
- Commands: `git log --oneline -- crates/nexus-core-rs/src/verification.rs` =>
  `1d010b0` (S54 edition), `7bb656b` (S27 watermark), `9c281d0` (S10),
  `4c2cba6` (S2 original). `git log --oneline -- task.rs` => incl. `0daff81`
  (S71 Phase B deterministic quorum), `34c77ce` (S23 exclude redundancy_factor
  from canonical), `dc163ea` (S23 redundancy voting).
- Decisions crossed / reversion status:
  - `34c77ce` (S23, R3): `redundancy_factor` is **deliberately excluded** from
    `task_canonical_bytes` (`task.rs:39-52`, `task_canonical_excludes_redundancy_factor`
    test). Rationale: dispatch policy is not cryptographic identity. **Still
    valid, no reversion.** A NEW required-tuple field on `Task` must decide:
    signed-identity (like `verifiable`, included) vs dispatch-policy (like
    `redundancy_factor`, excluded). The adaptation pins it as **signed identity**
    (it changes what cohort may compute — same logic as `verifiable` at
    `task.rs:145-168`). Non-blocking once the field follows the existing
    signed-identity precedent.
  - `0daff81` (S71 Phase B / B-2): `verifiable` => greedy + fixed seed; validator
    is **mode-agnostic and INCHANGE**. The Phase C/D plan honors this (validator
    untouched). No reversion.
  - `model_digest` = `blake3(model_name)` at `runtime.rs:1082` is **original S2
    placeholder** behavior (`4c2cba6`), kept consistent with the
    `unprofiled_model_passes_digest` test (`verification.rs:355`) so that an
    empty/unconfigured whitelist passes. The design review `sprint76_design_review.md`
    L59-76 already flagged the doc/impl discordance (doc says "exact model file",
    impl hashes the name) as **pre-existing**, not a Phase C regression.
  - **Reverse-commit check on the Verifier path**: `grep -rn "Verifier::new|
    register_digest|.verify("` outside `verification.rs` (non-test) => **0
    production hits**. `Verifier::verify` (the sole consumer of
    `ResultPayload.model_digest` layer-2) is **never wired into any production
    crate**; the live result path is `validate_quorum_pre_guardrail`
    (`validator.rs:219`) over raw `result_text` (the `sha256` column holds text,
    PATTERNS §P53). So `model_digest` is currently a **dead field on the prod
    path** — changing how the worker computes it has **zero effect on any live
    verification today**.
- Finding: **clean** (no un-reverted decision contradicts the phase). One
  documented constraint to honor: a new `Task` tuple field must follow the
  `verifiable` signed-identity precedent (`34c77ce` distinguishes the two
  classes), and the validator must stay INCHANGE.

## S3 Local Patterns And Threat Model
- Threats/contracts checked (THREAT_MODEL.md compute surface, T0-T5):
  - **A worker lying about its capability tuple to enter a cohort.** Today there
    is NO cohort and NO capability advertise (`grep runtime_family|quant|cohort`
    => 0), so this is a NEW surface the phase introduces. Mitigation that must
    ship with Volet 2: the cohort gate is **advisory routing, not a trust
    boundary** — the real defense remains the existing exact-match quorum
    (`validate_quorum_pre_guardrail`): a worker that advertises a false tuple but
    produces a divergent `result_text` is rejected as an outlier
    (`validator.rs:290-336`, `quorum outlier detected`). The tuple advert MUST be
    carried inside the **signed** Claim/Result canonical (or the signed Task
    requirement), so a lie is attributable, not an anonymous MITM. Document this
    explicitly so the cohort gate is not mistaken for a security control.
  - **name-hash vs file-hash as a layer-2 attack surface.** Because `Verifier`
    is dormant (0 prod callers), neither name-hash nor file-hash is currently a
    live control. Keeping the name-hash (doc-noted) does NOT regress any
    *enforced* threat; it only keeps the dormant layer-2 honest about what it is.
    Hardening to a GGUF file hash would create a *false* sense of attestation
    (the worker self-reports it; nothing cross-checks it) while breaking the
    hermetic StubBackend test path. No T0-T5 regression either way.
  - Existing compute mitigations preserved: consent filter (`runtime.rs:929-982`),
    rate-limit gate `(coordinator, self, model)` (`runtime.rs:996-1016`),
    signed Claim attribution (`task.rs:533` `verify_signature` checks
    `claimed_by == worker_pubkey`), guardrail-before-persist (S73 Phase A).
- HARDENING_ROADMAP status: no Sprint-76 pre-requirement is missed; the cohort
  gate is additive hardening, not a fix to an open pre-req. THREAT_MODEL §15
  (seed surface) is unrelated; a new "compute cohort homogeneity" row should be
  added at wrap-up (Phase G) documenting the advisory-not-trust nature.
- Finding: **non-blocking**. The phase ADDS a surface; it does not regress a
  covered T0-T5. Requirement: the tuple advert lives in signed canonical and is
  documented as advisory (quorum remains the real gate). Track as a Phase-G
  THREAT_MODEL row.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `canonical.rs` (DOMAIN_* + _VERSION grep);
  `task.rs` (Task / ResultPayload / Claim canonical); `result_sync.rs`
  (result producer->consumer); `http.rs` (`/api/v1/tasks/{id}/result`
  consumer at L458, `result_text` at L8275/8326/9206).
- VERSION/domain/canonical status:
  - `TASK_FORMAT_VERSION = 1` (`task.rs:61`) — **must stay 1**. Pre-launch policy
    (CLAUDE.md, kickoff §1): additive fields on the current v1 do NOT bump; only
    an envelope-structure change bumps. The adaptation adds at most one signed
    `Task` field (`#[serde(default)]`) — a v1 redefinition, NOT a bump. **OK.**
  - `DOMAIN_TASK_V1`/`DOMAIN_RESULT_V1`/`DOMAIN_CLAIM_V1` unchanged. No new domain
    constant needed (the tuple advert rides inside an existing signed struct).
  - `model_digest: [u8;32]` (`task.rs:374`) and `logprobs_hash: [u8;32]`
    (`task.rs:383`) **already exist** in v1 signed `ResultPayload` — S76 uses,
    does not add (kickoff §4 D3, finding G1). Doc/impl discordance is pre-existing.
- Producer->consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH):
  - **`result:` convergence path (Volet 1 measurement) — UNCHANGED, both ends
    read:** producer = worker `doc.set("result:{id}", ResultEntry)`
    (`runtime.rs:1110-1114`); replication = iroh-docs (entry carries 32-byte
    BLAKE3, blob travels via iroh-blobs — kickoff §D2 doc); consumer #1 =
    `result_sync.rs` observes `InsertRemote`, decodes `ResultEntry`, forwards
    `ResultEvent::NewResult` to `validator_loop`; consumer #2 = HTTP `GET
    /api/v1/tasks/{id}/result` returns `{result_text}` (http.rs:458, body key
    `result_text` asserted at L9206). The WAN-convergence acceptance exercises
    exactly this chain with **zero serialization change** — "zero changement
    mecanique" is provable: both ends already agree on the shape.
  - **If a new `Task` tuple-requirement field is added (Volet 2):** producer =
    coordinator `submit_task` (`dispatcher.rs:72-90`, must set the new field);
    serialized = `task_canonical_bytes` (`task.rs:39`); consumer = worker reads
    `task_entry.task.<field>` after `verify_signature()` (`runtime.rs:916`), and
    the claim-gate compares it against the worker's local tuple. Both ends are in
    the same Rust crate set (`nexus-core-rs` struct, `nexus-coordinator-rs`
    producer, `nexus-worker-core` consumer) — no cross-language Zod consumer for
    `Task` exists. `#[serde(default)]` keeps a minimal client JSON decoding to
    the inert value (mirror of `verifiable`/`redundancy_factor`). **Additive,
    no bump.**
- Day 0 status: **preserved**. iroh 0.98 pinned (no upgrade); validator
  INCHANGE; greedy+seed determinism (S71) intact; pre-launch additive policy
  respected; "source verifiable" vocabulary untouched.
- Finding: **clean**. No `*_VERSION` bump, no new domain, no tolerant
  multi-version decoder, `serde(default)` justified as runtime tolerance.

## Plan Adaptation
PLAN-ADAPT triggered by the blocking S1a finding on Volet 2.

- Original plan (§C.2, §C.4): (a) "durcir `model_digest` ... -> hash du fichier
  GGUF (le champ existe deja `task.rs:374`) [P1 ... OU doc-note]"; (b) advertise
  the tuple "dans la capability worker (`capability_store.rs`)"; (c) "le
  dispatcher n'assigne les replicas `verifiable`+redundancy>1 qu'aux workers
  homogenes" at `dispatcher.rs:37-133`.

- Evidence requiring adaptation:
  1. **`capability_store.rs` is the wrong substrate** (read full, 223 lines): it
     is a 6-flag feature-toggle store (`KNOWN_CAPABILITIES` = biometric_gate,
     federation_canary, mcp_server_expose, rag_retrieval, streaming_bridge,
     tool_calling), with no per-worker entries and no model/quant/runtime concept.
     It cannot hold a worker capability tuple.
  2. **`dispatcher.rs:submit_task` does NOT assign workers** (L37-133): it only
     signs+persists a `TaskEntry`. The architecture is **PULL** — the worker
     writes `claim:{id}` (`runtime.rs:1054`) to race for ownership; the
     coordinator never pushes an assignment. "Le dispatcher assigne aux workers
     homogenes" is architecturally impossible at that call site.
  3. **No clean GGUF digest accessor** (context7 ollama-rs 0.3.4 + Ollama API):
     `LocalModel{name,modified_at,size}` and `ModelInfo{...}` expose no digest;
     `LlmBackend` (trait `llm/mod.rs:315`) exposes only `healthcheck`+`generate`.
     Hashing the real GGUF requires fragile modelfile string-parsing and breaks
     the hermetic `StubBackend` (no file). `Verifier` (the only consumer of
     `model_digest`) has **0 production callers** — hardening changes nothing live.

- Corrected approach:
  - **Q1 (model_digest): DOC-NOTE, do not harden to a GGUF file hash this sprint.**
    Justification = technical infeasibility through the Ollama black box + the
    field is dead on the prod path (Verifier dormant). Keep
    `blake3(task.model.as_bytes())` (`runtime.rs:1082`) and add a doc-note +
    test (`model_digest_hashes_gguf_file_or_documented`, doc-note branch) that
    asserts the name-hash and records the discordance for S77. **Tighten the
    field doc-comment** at `task.rs:370` to say "BLAKE3 of the model NAME the
    worker ran (placeholder; a real GGUF file digest is gated on a backend that
    exposes it, e.g. feature `llm_llama_cpp`, S77)" so doc and impl agree. A real
    GGUF hash belongs to the `LlamaCppBackend` path (C-API gives file access),
    feature-gated `llm_llama_cpp`, in S77 — exactly where the D3 etage-2 reserve
    already lives. This is consistent with the D3 gel ("durcir ... P1 ... OU
    doc-note si hors-scope") and the PO steer is honored by *naming the
    constraint*, not by shipping a false attestation.
  - **Q2/Q3 (cohort routing): apply at the worker CLAIM-GATE, not the dispatcher,
    and carry the tuple in a NEW per-worker advert mechanism (NOT
    `capability_store.rs`).** Two concrete options:
    - (A, recommended) **Task carries the required tuple; worker claim-gate
      enforces it.** Add one signed `Task` field, e.g. `required_runtime:
      Option<RuntimeRequirement{ model: String, quant: String, runtime_family:
      String }>` (or a flat `required_quant`/`required_runtime_family` pair),
      `#[serde(default)]`, in the SIGNED canonical (precedent: `verifiable`,
      `task.rs:145-168`; NOT excluded like `redundancy_factor`). The worker, at
      claim time (`runtime.rs` ~L916, after `verify_signature`), reads its own
      local tuple (model name from `task.model`; quant + family from Ollama
      `/api/show details.quantization_level` + `details.family`, fetched via a
      new `LlmBackend::model_info(name)` accessor that `StubBackend` stubs with a
      configured tuple) and **does not emit a `ClaimEntry` unless its tuple
      matches** the task's `required_runtime`. Effect: only homogeneous workers
      claim a `verifiable`+redundancy>1 task, so the unchanged quorum
      (`validate_quorum_pre_guardrail`) sees byte-identical `result_text` from a
      homogeneous cohort. The validator stays INCHANGE. This is the PULL-correct
      point of application.
    - (B, fallback if the StubBackend tuple-stub proves heavy) **Worker advertises
      its tuple inside the signed Claim/Result; coordinator-side cohort check at
      result-ingress filters which results feed the quorum.** Heavier (touches
      the validator-adjacent ingress) and risks brushing the INCHANGE constraint;
      prefer (A).
  - **`dispatcher.rs` change** is limited to setting the new `required_runtime`
    on the crafted `Task` when `verifiable && redundancy_factor > 1` (additive,
    `dispatcher.rs:72-90`), NOT an assignment loop.

- File/test delta vs original plan:
  - `crates/nexus-core-rs/src/task.rs`: +1 signed `Task` field
    (`required_runtime`/tuple), `#[serde(default)]`, builder `with_*`, canonical
    inclusion test (mirror `task_canonical_includes_verifiable`). Tighten
    `model_digest` doc-comment (L370). **Not** `capability_store.rs`.
  - `crates/nexus-worker-core/src/llm/mod.rs` + `ollama.rs` + (stub): add a
    `model_info`/tuple accessor on `LlmBackend` (Ollama -> `/api/show` details;
    Stub -> configured tuple). `model_digest` impl unchanged (doc-note).
  - `crates/nexus-worker-core/src/engine/runtime.rs`: claim-gate compares local
    tuple to `task.required_runtime` before emitting `ClaimEntry` (~L1034). Keep
    `model_digest = blake3(name)` (L1082) + doc-note.
  - `crates/nexus-coordinator-rs/src/dispatcher.rs`: set `required_runtime` when
    `verifiable && redundancy>1` (L72-90).
  - Tests: `capability_advertises_homogeneity_tuple` ->
    `worker_claim_gate_matches_required_runtime_tuple` (claim-gate semantics);
    `dispatcher_routes_replicas_to_homogeneous_cohort` ->
    `non_homogeneous_worker_does_not_claim_verifiable_redundant_task`;
    `model_digest_hashes_gguf_file_or_documented` => doc-note branch (asserts
    name-hash + records S77 discordance). `validate_quorum_pre_guardrail`
    INCHANGE; existing gate `e2e_network_execute_gate_real_http_no_frontier_mock`
    (`nexus-shell-daemon/src/runtime.rs:3965`) stays green.
  - The plan file is a snapshot and stays unchanged; the commit body must cite
    this preflight: "Plan proposed dispatcher-side homogeneous assignment +
    capability_store advert + GGUF file-hash; preflight S1a found the
    architecture is PULL (claim-gate), capability_store is a feature-toggle
    store, and Ollama exposes no clean GGUF digest; adapted to a signed Task
    required-tuple enforced at the worker claim-gate, with model_digest kept as
    name-hash + doc-note (GGUF hash deferred to llama_cpp/S77)."

## Resolution of the four load-bearing questions
1. **model_digest discordance:** **DOC-NOTE** (not harden). Ollama exposes no
   clean GGUF file digest (`LocalModel`/`ModelInfo` have none; `LlmBackend` only
   `healthcheck`+`generate`); a real hash needs the `llm_llama_cpp` C-API path
   (S77, D3 etage-2 reserve). The field is dead on the prod path (`Verifier` has
   0 production callers), so the name-hash regresses nothing. Keep
   `blake3(name)`, fix the `task.rs:370` doc-comment to match impl, add the
   doc-note test. PO steer ("durcir") is honored via the plan's explicit
   "OU doc-note si techniquement infaisable" escape, justified on feasibility.
2. **No capability advertise mechanism exists:** Confirmed. `capability_store.rs`
   is a 6-flag feature-toggle store, wrong substrate. There is no per-worker
   model/quant/runtime advert anywhere (`grep` => 0). The advert is NEW and must
   live in the signed Task requirement (option A) or signed Claim/Result
   (option B), not `capability_store.rs`.
3. **PULL not PUSH:** Confirmed. `submit_task` (`dispatcher.rs:37-133`) only
   signs+persists; the worker claims (`runtime.rs:1054`). The homogeneity routing
   MUST apply at the **worker claim-gate** (the worker does not claim unless its
   tuple matches the task's `required_runtime`). "Le dispatcher assigne aux
   workers homogenes" is an architectural mis-statement -> this is the core of
   the PLAN-ADAPT.
4. **Does Task carry a required-tuple field?** **No.** `Task` (`task.rs:74-213`)
   has `model`, `verifiable`, `redundancy_factor`, but no `required_capability`/
   `required_runtime`. Adding one is **additive on v1** (pre-launch policy): a
   signed-identity field (`#[serde(default)]`, precedent `verifiable`), NO
   `TASK_FORMAT_VERSION` bump, NO new domain constant. S4 clean.

## Risks And Scope Cuts
- Blocking risks: none remain after adaptation (the only blocker was the S1a
  Volet-2 mechanism mismatch, now mapped to PLAN-ADAPT).
- Non-blocking risks / carry-over:
  - The cohort tuple advert is **advisory routing, not a trust boundary**; the
    exact-match quorum stays the real defense (a lying worker producing a
    divergent result is rejected as an outlier). Add a THREAT_MODEL compute-cohort
    row at Phase G.
  - `model_digest` true GGUF attestation deferred to S77 / `llm_llama_cpp` (D3
    etage-2 reserve) — documented, not silently dropped.
  - WAN convergence measurement (Volet 1) is an OBSERVATION, not a code change;
    >30s is a BLOCK to diagnose (per §C, ref S75 `SeedAnnounced peer_count:0`),
    handled in acceptance, not by timeout inflation.
- Scope cuts still honored (kickoff §7 / §C.5): no push scheduler, no synchronous
  RPC, no custom DHT, cross-GPU heterogeneous = post-S77, validator INCHANGE,
  iroh 0.98 pinned, zero wire bump, zero new dependency.

## Action
- PLAN-ADAPT: proceed. Volet 1 (acceptance LIVE B-3 + WAN convergence
  measurement) executes as written — the `result:` producer->consumer chain is
  unchanged and verified both-ends. Volet 2 (cohort routing) follows the
  corrected approach: model_digest kept as name-hash + doc-note (GGUF hash ->
  S77/llama_cpp); homogeneity enforced at the worker CLAIM-GATE via a new signed
  `Task.required_runtime` tuple (additive v1, no bump), with the worker tuple
  sourced from a new `LlmBackend` model-info accessor (Ollama `/api/show`
  details; Stub configured); `dispatcher.rs` only sets the requirement, never
  assigns. The commit body MUST cite this file and document the deviation. The
  validator `validate_quorum_pre_guardrail` and the gate
  `e2e_network_execute_gate_real_http_no_frontier_mock` stay untouched and green.
