Overall result: **GAP-P1**. The Rust profile and `PROFILE_STALE` logic are statically strong, but the enforcement-honesty claim is not yet clean: documented `chore` phase semantics contradict both the profile and the authoritative implementation, and the portable Git hooks are committed without executable bits.

## 1. Honesty of `documented | enforced` — GAP P1

Confirmed:

- Roles are the exact nine IDs from `docs/agent/AGENT_SYSTEM.md:33-47`, reproduced at `crates/sbfb-workflow-core/src/factory_profile.rs:422-442`.
- Modes are the exact ten IDs from `docs/agent/AGENT_SYSTEM.md:80-107`, reproduced at `factory_profile.rs:445-458`. All are `Documented`.
- The PROCESS Guarantee Matrix only names G1 and G5 as hook-backed at `docs/agent/PROCESS.md:50-61`.
- G1 blocks through the provider-neutral message gate at `scripts/agent/agentctl.py:631-648`, subject to the declared kickoff exemption at `agentctl.py:552-559`; the Claude mirror blocks at `.claude/hooks/phase-precommit-lightcheck.sh:252-287`.
- G5 staging/module coherence is blocking at `agentctl.py:107-114`, `agentctl.py:374-401`, and is invoked at `agentctl.py:621-648`; the Claude mirror is `.claude/hooks/phase-precommit-lightcheck.sh:92-129`.
- G8 is correctly `Documented`: only the Claude layer blocks it at `.claude/hooks/phase-precommit-lightcheck.sh:395-411`.
- FG-CSP runs before publication and outside `skip_gates`, returning `Err` on failure at `crates/sbfb-factory/src/pipeline.rs:27-62`; redeploy repeats it before upload at `crates/sbfb-factory/src/atelier.rs:85-101`.
- FG8 is correctly `Documented`: publish occurs first at `pipeline.rs:64-65`, FG8 is post-publish at `pipeline.rs:67-80`, redeploy has no FG8 at `atelier.rs:68-159`, and the N0 exception is documented at `docs/factory/FACTORY_GATES.md:210-217`.
- FG4 is explicitly always recorded as passed at `crates/sbfb-factory/src/gates.rs:190-205`; FG5/FG6 are bypassable at `pipeline.rs:27-45`. No other complete FG gate was found unconditionally blocking both Factory publish verbs.
- The five current enforced entries are exactly G1, G5, FG-CSP-authoring, and the two finality hooks at `factory_profile.rs:461-542`; the oracle asserts five at `factory_profile.rs:1038-1044`.

Residual P1 — `chore` finality scope is not honest:

- The profile says all `chore` commits are excluded “by design” at `factory_profile.rs:50-52`.
- Canonical tooling says the opposite: `chore` belongs to the phase regex at `docs/agent/TOOLING.md:57-60`, and `chore(sprintN): Sprint N Phase X` “no longer bypasses review/codex/body gates” at `TOOLING.md:167-172`.
- The Claude mirror enforces Codex evidence for `chore` at `.claude/hooks/phase-precommit-lightcheck.sh:289-304`.
- The authoritative Python implementation nevertheless excludes every `chore` from Codex/body validation at `scripts/agent/agentctl.py:562-566`, and completely exempts `chore(planning): ... Phase ...` from the auditor gate at `agentctl.py:672-679`.
- Therefore the machine entry “precommit gate for phase commits … Codex artifact” at `factory_profile.rs:526-534` is false for a documented class of authoritative-path phase commits. This is a process-gate bypass, hence P1 rather than a harmless scope-cut wording issue.

Residual P1 — portable hook activation:

- `.githooks/pre-commit` and `.githooks/commit-msg` are both indexed as mode `100644`, verified with `git ls-files -s`.
- Their bodies correctly invoke the gates at `.githooks/pre-commit:17` and `.githooks/commit-msg:17-18`.
- Installation only sets `core.hooksPath` at `docs/agent/TOOLING.md:62-85`; `agentctl install-hooks` only prints that command at `scripts/agent/agentctl.py:702-705`. No repository chmod/update-index step exists.
- A fresh POSIX checkout therefore does not get executable Git hooks from the documented installation path. Current Git-for-Windows execution is **Not evidenced statically**.

## 2. Non-vacuous tests — CONFIRMED statically

- Real anchor resolution reads the target and calls `contains(needle)` at `factory_profile.rs:985-995`.
- `Enforced ⇔ mechanism.is_some()`, every anchor resolves, count equals five, and bogus-needle plus missing-file controls fail at `factory_profile.rs:997-1063`.
- Unchanged, byte-mutated, whitespace-mutated, and missing-current-source controls are at `factory_profile.rs:805-850`.
- Empty, partial, duplicate, reordered, path-tampered, and duplicate-current cases are at `factory_profile.rs:852-909`.
- Content tampering covers G1 demotion, cleared gates, and a negated invariant at `factory_profile.rs:911-946`.
- Duplicate extraction is covered at `factory_profile.rs:948-959`.
- The instance golden is strict unless explicit `UPDATE_SNAPSHOTS` is present: missing and empty files fail at `factory_profile.rs:760-803`.
- CRLF/BOM and lone-CR behavior is covered at `factory_profile.rs:961-983`.
- Cardinalities and all G1–G10 IDs are checked at `factory_profile.rs:1066-1109`; missing-source extraction at `factory_profile.rs:1111-1123`.
- Every `StaleReason` and both `ExtractError` display arms are exercised at `factory_profile.rs:1148-1191`.

Test execution and the driver’s claimed green runs: **Not evidenced**; no suite was executed.

## 3. Invariants and wire — CONFIRMED

- The frontier prose at `factory_profile.rs:262-271` contains no literal `// FRONTIER: ` tag. The parser scans that exact token at `scripts/check-frontier-contracts.sh:134-170`.
- Static grep found no `const DOMAIN_*` or `*_FORMAT_VERSION` in `crates/sbfb-workflow-core`.
- Independent census reproduction found 25 distinct `DOMAIN_*_Vn` constants, matching the frozen value at `scripts/check-frontier-contracts.sh:181-192`.
- The re-export is at `crates/sbfb-workflow-core/src/contracts.rs:27-34`; Factory remains the fourth insertion at `contracts.rs:95-124`.
- `CONTRACT_NAMES` remains length seven with Factory fourth at `contracts.rs:132-147`.
- The refreshed schema contains seven top-level identities and the filled Factory schema at `contracts.schema.json:9-275`.
- `Cargo.lock:7513-7521` adds only the `blake3` edge. The already-existing single version node remains `blake3 1.8.5` at `Cargo.lock:714-727`.
- Direct BLAKE3 recomputation matched all six hashes recorded at `factory_profile.golden.json:373-403`.

## 4. Purity — CONFIRMED

- `extract` takes injected bytes at `factory_profile.rs:571-595`; `detect_stale` takes injected current bytes at `factory_profile.rs:663-666`.
- Production code before `#[cfg(test)]` at `factory_profile.rs:727` contains no filesystem, network, Tokio, or database access.
- All `std::fs` uses occur inside tests at `factory_profile.rs:739-745`, `780-786`, and `985-994`.
- Runtime dependencies remain serde/schemars plus pure hashing at `crates/sbfb-workflow-core/Cargo.toml:11-19`, consistent with the charter at `src/lib.rs:2-10`.

## 5. `PROFILE_STALE` fail-closed behavior — CONFIRMED

- Non-source content is checked first against `canonical_profile` at `factory_profile.rs:645-675`.
- Embedded sources must exactly equal `SourceId::ALL` in length and order at `factory_profile.rs:677-686`.
- Embedded paths are tied to IDs at `factory_profile.rs:688-701`.
- Current input must contain exactly one entry per ID at `factory_profile.rs:702-710`.
- Digest comparison fails immediately at `factory_profile.rs:711-716`; success occurs only after every source passes at `factory_profile.rs:718`.
- `extract` rejects both absent and duplicate inputs at `factory_profile.rs:580-592`.
- Canonicalisation strips only a leading BOM and folds only paired CRLF; lone CR is preserved at `factory_profile.rs:393-414`. No collision beyond the intentional BOM/CRLF equivalence was found.
- Given authoritative current bytes and normal BLAKE3 collision resistance, no remaining deserialized-profile fail-open path was found.

## 6. Dead code, panics, naming — GAP P3

- No unsafe, `todo!`, `unimplemented!`, dead-code allowance, or input-driven production unwrap was found.
- The only production `expect` is deterministic schemars-to-JSON serialization at `factory_profile.rs:721-725`.
- P3 wording drift: `factory_profile.rs:47-49` and `283-286` say agentctl “WARNs” missing G8 generally. In reality, the warning is reached only for a staged wire-format surface at `scripts/agent/agentctl.py:477-499`; ordinary missing-preflight commits receive no provider-neutral warning. The conservative `Documented` marking remains correct.
- P3 scope inventory: the live worktree also contains three untracked Phase-D planning artifacts omitted from the supplied file list: `sprint83_phase_d_preflight.md`, `sprint83_phase_d_review.md`, and `sprint83_phase_d_codex_review.md`.

## Reconciliation rounds

- **Round 2 — CONFIRMED, with later supersession:** source-shape closure and lone-CR behavior remain present. The former G8=Enforced/count=7 state was intentionally superseded; current G8 is Documented and count is five.
- **Round 3 — CONFIRMED:** duplicate input/current handling, path binding, and `Malformed` distinction are closed at `factory_profile.rs:549-592`, `677-716`, and `852-959`.
- **Round 4 — CONFIRMED:** canonical-content integrity and all stated tamper shapes are closed at `factory_profile.rs:645-675` and `911-946`; error display coverage and local-vs-signed prose are present at `factory_profile.rs:1148-1191`, `contracts.rs:15-22`, and `lib.rs:41-62`.
- **Round 5 — CONFIRMED with P3 wording gap:** G8 is honestly Documented and reordered sources are rejected, but the claim that agentctl always warns is too broad.
- **Round 6 — GAP-P1:** FG8 demotion and the six-source scope cut are honest at `factory_profile.rs:54-82`, but the asserted `chore` exclusion conflicts with both canonical tooling and actual mixed enforcement. Portable Git-hook executable activation is also not repository-guaranteed.

**OVERALL: GAP-P1 — fail-closed Rust/profile mechanics confirmed; enforcement-honesty deliverable not clean because `chore` finality and portable hook activation remain materially inconsistent.**

