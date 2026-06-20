**Deliverable 1 — PARTIAL**

Pure rustfmt wrapping is confirmed. The diff keeps the same call, same six args/order, and same `.expect("credit")`: `crates/nexus-shell-daemon/src/http.rs:8531-8534`.

The stronger “real 1.94 violation, not drift” proof is not independently confirmed here. Local `cargo fmt --all --check` exits 0, but `rustup run 1.94.0 rustfmt --version` fails because toolchain `1.94.0-x86_64-pc-windows-msvc` is not installed, and Docker is inaccessible (`permission denied ... docker_engine`). The report states the Docker 1.94 byte-identical result in `.planning/active/sprint76_verification.md:34-45`, but I cannot treat that self-report as independent proof.

**Deliverable 2 — PARTIAL**

The `REDUNDANCY`/`VERIFIABLE` script mechanics are correct: `die()` exists before validation, `REDUNDANCY` defaults to 1, validates positive integer, canonicalizes via `10#`, sets `VERIFIABLE=true` for `>=2`, and prints both into JSON with `%s`: `scripts/acceptance/b3_live_pc_vps.sh:92-111`, `:175-179`. Git Bash `bash -n` passes. Simulated default gives `1 false`; `REDUNDANCY=02` gives `2 true`.

Root cause is mostly confirmed: dispatcher only carries `required_runtime` when `submission.verifiable && redundancy > 1`: `crates/nexus-coordinator-rs/src/dispatcher.rs:64-74`; runtime only forces deterministic params for `task.verifiable`: `crates/nexus-worker-core/src/engine/runtime.rs:1325-1345`; omitted `verifiable` defaults false: `crates/nexus-coordinator-rs/src/types.rs:95-102`.

Gap: the harness does not submit `required_runtime`. The HTTP handler passes `TaskSubmission` unchanged into `submit_task`: `crates/nexus-shell-daemon/src/http.rs:3306-3344`; the script JSON has no `required_runtime`: `scripts/acceptance/b3_live_pc_vps.sh:177-179`. So `verifiable=true` fixes deterministic inference and lets a manually homogeneous cohort form quorum, but the dispatcher’s actual `required_runtime` claim gate is not formed by this harness.

**Deliverable 3 — CONFIRMED**

No false-green LIVE rows found. `verification.md` explicitly says deferred assertions stay `DIFFERE`: `.planning/active/sprint76_verification.md:3-6`. Row #26 is `DIFFERE materiel operateur`: `:229`; row #30 is also `DIFFERE materiel operateur`: `:233`. Bilan is `36/38 verts ... + 2 rows LIVE ... DIFFERE`, not 38/38: `:243-244`.

The fmt diagnosis is retracted at `:34-45` and again at `:176-178`. The Docker recovery run is reported as 1808 at `:153-160`.

**Deliverable 4 — CONFIRMED**

`sprint77_audit_plan.md` carries the requested items: SYBIL-SEEDER-TAIL with named sharding exemption plus REVISION-HOME-DURABILITY, KNOWN-ENTRY-OVERCOUNT, seeder `catalog_len:0`, RE-DRIVE-ON-INGEST, T-NN+3, P3-D-3, and MEDIAN-DE-GROUPE: `.planning/active/sprint77_audit_plan.md:311-318`. It also includes extra B10-PARITE at `:319`, which does not drop a routed carry.

The three two-report carries are explicitly closed/not reconducted: `:325-330`. Tracks A-J cover suites, phases A-G, transverse wire policy, and meta-process: `:119`, `:147`, `:162`, `:183`, `:205`, `:227`, `:247`, `:263`, `:283`, `:292`.

**Deliverable 5 — CONFIRMED**

Long-life docs match the requested structure. `THREAT_MODEL.md` adds only v9 version history, and existing §15.1/§15.2/§15.3 remain in place: `docs/security/THREAT_MODEL.md:861`, `:895`, `:928`, v9 at `:1007-1026`. Duress B1 is referenced as already closed, not recreated as a new row: `:1018-1020`.

Rust patterns add free `§P62` after `§P61`: `docs/rust/PATTERNS.md:3416`, `:3454-3492`. Shell patterns add plain `P38` after `P37`: `docs/shell/PATTERNS.md:2231`, `:2274-2294`.

`SPRINT_LOG.md` inserts S76 before S75 at `docs/claude/SPRINT_LOG.md:19`, with A-F hashes matching `git show`; G is necessarily `[ce commit]` because the phase is uncommitted. `CLAUDE.md` says `0-76 CLOSED`, `S77 a ouvrir`, Arc 3.5 `6/6`, and `~2209 tests`: `CLAUDE.md:168-171`, `:316-328`. Roadmap has the S76 delivery block and §3 numbering note: `.planning/roadmap_v5_factory_complete_vision.md:33-47`, `:150-155`.

**Deliverable 6 — PARTIAL**

Confirmed locally: arithmetic is consistent (`1763+4+8+10+4+10+5+0 = 1804`; `1808-1804 = 4`; `398-367 = 31`). `cargo nextest list --workspace --locked` counted `nextest_count=1804`. `npm run test:unit -- --reporter=dot` with `NODE_OPTIONS=--no-experimental-webstorage` passed `398` tests.

Confirmed invariant checks: `git diff 73831c0~1..HEAD -- Cargo.lock` is empty; current Phase G worktree has no Cargo.lock diff. Current constants inventory shows protocol/schema constants at 1, with the known pre-existing exception `INVITE_FORMAT_VERSION = 2`; no current Phase G constant diff was found.

Unsupported part: Docker Linux 1808 and Docker rustfmt 1.94 cannot be independently replayed here because Docker API access is denied. That leaves the Docker counter/toolchain portion dependent on the repo self-report at `.planning/active/sprint76_verification.md:153-160`.

**Final Summary**

3/6 CONFIRMED, 3 PARTIAL, 0 GAP
