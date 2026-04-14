# Sprint 3 verification plan

**Scope**: verify that Sprint 3 of the SBFB / nexus-grid pivot
(the `nexus-worker` binary, waves W1..W12) is complete, clean,
reproducible, and reflects the final state committed on
`master` at **2026-04-10**.

**Audience**: a Claude session opened in the
`C:\Users\FlowUP\Documents\Code\nexus` working directory
with no prior conversation context. This document must be
self-contained — every check is a command to run and an
expected result to compare against.

**Time budget**: ≈ 15-20 min wall-clock on the dev machine,
dominated by one cold `cargo build` and one nextest run.

---

## 0. Prerequisites

Before running any of the checks below, make sure the shell
has access to the Rust toolchain and the `cargo-nextest`
subcommand:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
rustc --version    # expect 1.94 or newer
cargo --version
cargo nextest --version
```

If `cargo-nextest` is missing, install it once:

```bash
cargo install --locked cargo-nextest
```

All commands below assume the working directory is the repo
root (`C:\Users\FlowUP\Documents\Code\nexus`).

---

## 1. Git state

Sprint 3 landed as **14 commits on top of commit 626d7eb**
(the Sprint 2 S9 PyO3 bindings commit). The commit range is
`626d7eb..9476be8`. Two audit commits precede W1.

```bash
git log --oneline 626d7eb..HEAD
```

Expected output (order top-down, 14 lines):

```
9476be8 feat(worker): Sprint 3 W12 — e2e integration tests + docs/WORKER.md
d1426e4 feat(worker): Sprint 3 W10 — ratatui dashboard as optional layer
da09555 feat(worker): Sprint 3 W11 — structured logging with rotating files
6fcfc40 feat(worker): Sprint 3 W9 — engine runtime + real start handler
f308a2c feat(worker): Sprint 3 W8 — nx1 invite tokens + join/projects handlers
335cdda feat(worker): Sprint 3 W7 — per-worker project allowlist (SQLite)
9ae83c5 feat(worker): Sprint 3 W6 — WorkerState finite state machine
5fda7d8 feat(worker): Sprint 3 W5 — Ollama client with healthcheck + retry
19ef014 feat(worker): Sprint 3 W4 — GpuMonitor trait + NVML + Noop backends
448b957 feat(worker): Sprint 3 W3 — layered TOML config + register handler
7dcafa5 ci(worker): Sprint 3 W2 — canonical Rust CI + build matrices
accb7a3 feat(worker): Sprint 3 W1 — nexus-worker-core crate + clap CLI skeleton
ed2ea76 feat(core-rs): Sprint 2 scope completion — query_prefix + fetch_ticket
de9589d fix(core-rs): graceful Node::shutdown via Router::shutdown
```

**Pass criteria**: all 14 SHAs present, in this exact order,
no extra commits interleaved, working tree clean
(`git status` reports nothing).

If the working tree is not clean, stop and investigate —
the verification plan assumes the committed state.

---

## 2. Workspace layout

Verify that every crate, module and workflow file that
Sprint 3 was supposed to create actually exists on disk.

### 2.1 Crates in the workspace

```bash
cat Cargo.toml | grep -A10 '^\[workspace\]'
```

Expected members:
- `crates/nexus-core-rs`
- `crates/nexus-core-py`
- `crates/nexus-worker-core` (**W1**)
- `crates/nexus-worker`

### 2.2 `nexus-worker-core` module tree

```bash
ls crates/nexus-worker-core/src/
ls crates/nexus-worker-core/src/engine/
ls crates/nexus-worker-core/src/gpu/
```

Expected files:

```
crates/nexus-worker-core/src/
├── allowlist.rs        (W7)
├── config.rs           (W3)
├── engine/
│   ├── mod.rs          (W6/W9)
│   ├── runtime.rs      (W9)
│   └── state.rs        (W6)
├── gpu/
│   ├── mod.rs          (W4)
│   ├── noop.rs         (W4)
│   └── nvml.rs         (W4)
├── invite.rs           (W8)
├── lib.rs              (W1 + expanded)
└── ollama.rs           (W5)
```

`lib.rs` must `pub mod` all six top-level modules:
`allowlist`, `config`, `engine`, `gpu`, `invite`, `ollama`.

### 2.3 `nexus-worker` binary tree

```bash
ls crates/nexus-worker/src/
ls crates/nexus-worker/tests/
```

Expected:

```
crates/nexus-worker/src/
├── cli.rs       (W1, clap derive definitions)
├── logging.rs   (W11)
├── main.rs      (handlers)
└── tui.rs       (W10)

crates/nexus-worker/tests/
└── e2e.rs       (W12, 11 integration tests)
```

### 2.4 CI workflows and tooling

```bash
ls .github/workflows/
ls .config/
```

Expected:

```
.github/workflows/
├── build-wheels.yml     (W2 — maturin-action, 6 wheels)
├── build-worker.yml     (W2 — 7-target binary matrix)
├── ci.yml               (legacy Python/React CI, untouched)
└── rust-ci.yml          (W2 — fmt/clippy/test 3-OS)

.config/
└── nextest.toml         (W2 — `ci` profile with JUnit)
```

### 2.5 Documentation

```bash
ls docs/WORKER.md
wc -l docs/WORKER.md
```

Expected: file exists, ≈300+ lines.

---

## 3. Build and lint checks

Run these in sequence from the repo root. Each step must
complete without errors or warnings.

### 3.1 Formatting

```bash
cargo fmt --all --check
```

**Pass**: exit code 0, no output.

### 3.2 Full workspace build

```bash
cargo build --workspace --locked
```

**Pass**: exit code 0, `Finished dev profile`. First cold
build takes ≈1-2 min because of iroh + tokio; subsequent
builds are incremental.

### 3.3 Clippy on every target

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

**Pass**: exit code 0. `-D warnings` promotes every lint to
an error, so any clippy drift from the committed state fails
this check.

### 3.4 PyO3 bindings compile

```bash
cargo check -p nexus-core-py --locked
```

**Pass**: exit code 0. This does not build a wheel (that
requires maturin), only validates that the PyO3 bindings
still compile against the current `nexus-core-rs` API.

---

## 4. Tests

### 4.1 Full nextest run with the CI profile

```bash
cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked
```

**Expected summary**:

```
Starting 161 tests across 4 binaries
Summary [~5s] 161 tests run: 161 passed, 0 skipped
```

Anything less than 161 tests, or any failure / skip other
than the deliberate ones, is a regression.

Breakdown by binary (in case a specific crate regresses):

| Binary | Test count |
|---|---|
| `nexus-core-rs` lib | 56 |
| `nexus-worker-core` lib | ≈79-80 |
| `nexus-worker` bin (CLI parser) | 11 |
| `nexus-worker` e2e integration | 10-11 (SIGINT test is unix-only) |

On Windows the `start_headless_boots_and_shuts_down_on_signal`
test is gated behind `#[cfg(unix)]` and nextest reports 160
on the Windows runner, 161 on Linux/macOS. Either count is
acceptable.

### 4.2 Doctests (nextest does not run them)

```bash
cargo test --workspace --exclude nexus-core-py --doc --locked
```

**Expected summary**: 5 doctests green, all from
`nexus-core-rs` module doc examples (`gossip.rs`, `blobs.rs`,
`docs.rs`, `node.rs`, `lib.rs`).

### 4.3 Sanity check on a specific wave

If any of the per-wave tests from the commit messages need
to be re-verified individually, run:

```bash
# W6 state machine (17 tests)
cargo nextest run -p nexus-worker-core engine::state --profile ci

# W7 allowlist (15 tests)
cargo nextest run -p nexus-worker-core allowlist --profile ci

# W8 invite (13 tests)
cargo nextest run -p nexus-worker-core invite --profile ci

# W9 engine runtime (4 tests)
cargo nextest run -p nexus-worker-core engine::runtime --profile ci

# W12 e2e (10-11 tests)
cargo nextest run -p nexus-worker --test e2e --profile ci
```

All must report green.

---

## 5. CLI smoke tests

These run the compiled binary against a temporary fixture
directory. They're a superset of the W12 e2e tests — run
them once to confirm the binary actually works end-to-end
on this machine, not just that cargo build succeeds.

Use a temp dir to avoid touching the real
`~/.config/nexus-grid/`:

```bash
FIXTURE=$(mktemp -d)
WORKER_BIN="./target/debug/nexus-worker"
CFG="$FIXTURE/worker.toml"

cargo build -p nexus-worker
```

### 5.1 Help + version

```bash
"$WORKER_BIN" --version
"$WORKER_BIN" --help
```

**Pass**: `--version` prints `nexus-worker 0.1.0` (or the
committed version in `Cargo.toml`). `--help` lists all seven
subcommands: `register`, `start`, `join`, `projects`,
`browse`, `stats`, `config`.

### 5.2 Register creates identity and config

```bash
"$WORKER_BIN" --config "$CFG" register --name "sprint3-verify"
ls "$FIXTURE"
cat "$CFG"
```

**Pass**:
- `$CFG` exists and contains `[identity]` with `name =
  "sprint3-verify"`
- `$FIXTURE/data/worker.key` exists (Ed25519 secret)
- stdout includes `public key (hex):` followed by 64 hex
  chars

### 5.3 Projects list is empty

```bash
"$WORKER_BIN" --config "$CFG" projects list
```

**Pass**: stdout includes `no projects enrolled yet`.

### 5.4 Register refuses a second time

```bash
"$WORKER_BIN" --config "$CFG" register --name "anotherone" || echo "refused (expected)"
```

**Pass**: non-zero exit, stderr contains `already
registered`.

### 5.5 `stats` reports the registered worker

```bash
"$WORKER_BIN" --config "$CFG" stats
```

**Pass**: stdout contains `sprint3-verify`.

### 5.6 Headless start (bounded)

On Unix:

```bash
"$WORKER_BIN" --config "$CFG" start --headless &
WORKER_PID=$!
sleep 2
kill -INT $WORKER_PID
wait $WORKER_PID
```

**Pass**: wait returns 0 (graceful exit). No zombie
process. Logs visible at `$FIXTURE/logs/worker-*.log`.

On Windows, run the equivalent PowerShell:

```powershell
$p = Start-Process -FilePath $WORKER_BIN `
    -ArgumentList "--config", $CFG, "start", "--headless" `
    -PassThru -NoNewWindow
Start-Sleep -Seconds 2
Stop-Process -Id $p.Id
```

(Windows does not hook ctrl+c as gracefully without a
console group, so Stop-Process is fine for the smoke
check.)

---

## 6. What the verification should NOT find

The following items are **deliberately absent** from Sprint
3 and are tracked as W9.1 / Sprint 4 follow-ups. Finding any
of them means the state has drifted past the committed
Sprint 3 checkpoint.

- **Real task claim / execute / result write-back code**.
  Grep must still find unchanged `TODO(W9.1)` markers in
  `crates/nexus-worker-core/src/engine/runtime.rs`:

  ```bash
  grep -n "TODO(W9.1)" crates/nexus-worker-core/src/engine/runtime.rs
  ```

  Expected: at least one match inside `Engine::tick`.

- **Live GPU stats in the TUI body**. The TUI's
  `render_gpu` must still render only the boot-time
  `GpuInfo` — no calls to `engine.gpu_snapshot()` from the
  render loop.

- **`browse` subcommand implementation**. The handler in
  `main.rs` must still call `print_stub("browse", ...)`.

- **Coordinator-side code**. There is no `nexus-coordinator`
  crate or Python package yet — that's Sprint 4.

- **`nexus-core-rs::task` struct changes**. The Sprint 2
  audit P1 tech debt about `canonical_bytes` cross-language
  ordering is NOT fixed in Sprint 3. It's logged and
  deferred to Sprint 4 Day 1.

---

## 7. Summary fail-fast table

Use this table as a quick scoreboard. If everything reports
"pass", Sprint 3 is intact and the next step is Sprint 4
(coordinator + SDK).

| Check | Command | Pass criterion |
|---|---|---|
| Git log | `git log --oneline 626d7eb..HEAD` | 14 commits, exact SHAs |
| Working tree | `git status --porcelain` | empty |
| Format | `cargo fmt --all --check` | exit 0 |
| Build | `cargo build --workspace --locked` | exit 0 |
| Clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| PyO3 compile | `cargo check -p nexus-core-py --locked` | exit 0 |
| Nextest | `cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked` | 160-161 pass |
| Doctests | `cargo test --workspace --exclude nexus-core-py --doc --locked` | 5 pass |
| CLI register | `nexus-worker register --name X` | creates worker.toml + key |
| TODO markers | `grep TODO(W9.1) crates/nexus-worker-core/src/engine/runtime.rs` | ≥1 match |
| Docs present | `test -s docs/WORKER.md` | exists, ~300 lines |

**If all 11 rows pass**, Sprint 3 is verified and the
repository matches the state that closed Sprint 3 at commit
`9476be8` on 2026-04-10. Proceed to Sprint 4.

**If any row fails**, investigate before starting new work.
Do not paper over regressions — re-run the specific failing
check with verbose output and compare against the commit
messages in the W1..W12 range for context.
