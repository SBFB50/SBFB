Verification run:
- `cargo test -p sbfb-factory --test process_cli --locked` -> 13/13 passed.
- `cargo run -p sbfb-factory --locked -- process prompt --kind universal --depth deep` -> failed: `unknown prompt kind 'universal'`.
- Scope-cut scan found no `SearchManifest`, `/factory` route, or `tree-sitter` references in the listed Phase C surfaces.

**Deliverables**
1. `prompts/agent/handoff.md:51`-`121` - PASS: exactly 9 numbered sections; provider-neutral rules present at `:130`-`:139`.

2. `prompts/agent/preflight.md:240`-`277` - PASS: Finding Classification table, PLAN-ADAPT procedure, and Anti-Patterns are present.

3. `prompts/agent/phase-review.md:124`-`153` - PASS: dimensions 8-11 are present, and the prompt now has 12 review dimensions.

4. `prompts/agent/commit-body.md:117`-`143` - PASS: validation commands/regex are present, plus Anti-Patterns at `:143`.

5. `prompts/agent/audit-gate-checks.md:43`-`162` - PASS: exactly 9 tracks A-I; commands, P0-P3 classification, and verdict tree are present at `:181`-`:191`.

6. `prompts/agent/phase-auditor.md:22`-`40` - PASS: 7 Dimensions section and opinion-first pattern are present.

7. `crates/sbfb-factory/src/process.rs:5`-`18`, `:35`-`:43` - GAP-P1: `resolve_kind()` handles the requested 6 canonical kinds + 3 aliases, but Rust prompt assembly does not support `base` or `universal`, while docs advertise an 8-kind contract. Also GAP-P3 at `:37`: guarded `unwrap()` is not exploitable, but violates the stated no-unwrap standard.

8. `crates/sbfb-factory/src/main.rs:92`-`117`, `:175`-`:191` - PASS: `process context` and `process prompt` are wired. Minor caveat: `depth` is documented as standard/deep but not validated.

9. `crates/sbfb-factory/tests/process_cli.rs:9`-`277` - GAP-P2: 13 tests pass and are mostly meaningful, but alias coverage misses `audit -> audit-gate` (`:175`-`:205` only checks `review` and `auditor`), and the local cloud-stripping test at `:218`-`:239` is weak because the selected prompt has no `WebSearch/context7` strings to strip.

10. `docs/agent/PROCESS.md:137`-`155`, `:163`-`177` - GAP-P1: the 8-kind Prompt Contract and bootstrap matrix are present, but `:140` says Rust can assemble `{kind}` and `:233` explicitly uses `--kind universal`; the actual Rust CLI rejects that kind.

11. `docs/agent/TOOLING.md:109`-`151` - PASS: Rust Prompt Assembly section documents `process context`, `process prompt`, supported Phase C kinds, aliases, and local provider stripping.

12. `docs/agent/AGENT_SYSTEM.md:197`-`213` - PASS: Prompt Registry entries for `handoff` and `audit-gate` are present. Same global 8-kind/Rust-support caveat is counted under PROCESS/process.rs.

Overall verdict: PARTIAL.

Blocking/near-blocking gaps:
- GAP-P1: docs advertise Rust assembly for `universal`/8-kind contract, but `sbfb-factory process prompt --kind universal` fails.
- GAP-P2: process CLI tests miss one alias and do not truly prove cloud-reference stripping.
- GAP-P3: guarded `unwrap()` and unvalidated `depth` are small Rust hygiene issues, not security findings.
