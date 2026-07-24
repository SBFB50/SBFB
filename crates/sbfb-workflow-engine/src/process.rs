// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process effects (Sprint 83 Phase K): spawn a command whose descendant
//! tree is killed on cancel/timeout/crash — CLOSED on Windows (Job
//! Object), BOUNDED on Unix (process group; see the honest bounds below).
//!
//! ## The two OS mechanisms (in-tree SOTA, lifted from `local_worker.rs`)
//!
//! - **Windows (closed)** — a kill-on-close Job Object: every descendant
//!   inherits the job (breakaway aside), `TerminateJobObject` kills the
//!   tree on cancel/timeout, and closing the last handle (which the OS
//!   does when THIS process dies, however abruptly) kills whatever is
//!   left — the crash-cleanup story needs no bookkeeping. Hardened over
//!   the daemon's module: the child is spawned `CREATE_SUSPENDED`,
//!   assigned to the job, THEN resumed (a fast-forking child cannot spawn
//!   a grandchild in the assign-after-spawn window; cargo/deno close it
//!   the same way), and an assign/resume failure KILLS the suspended
//!   child before surfacing.
//! - **Unix (bounded)** — `process_group(0)` (std) isolates the child in
//!   its own process group; `libc::kill(-pgid, SIGKILL)` kills the group.
//!   Linux adds `PR_SET_PDEATHSIG` on the DIRECT child as an
//!   abnormal-parent-death backstop. Honest bounds (same as the daemon's
//!   module — the THREAT row says the same): a descendant that calls
//!   `setsid()` leaves the group and escapes `kill(-pgid)` (a hard
//!   containment would need cgroups/PID namespaces, out of the std
//!   charter); macOS has no pdeathsig, so cleanup after an ABNORMAL
//!   parent death relies on the next resume's orphan-reap of the
//!   journaled pgid ([`kill_process_group`]); and the pgid handle is
//!   persisted only AFTER the spawn — a crash inside that window leaves
//!   no replayable handle.
//!
//! ## Injection posture (THREAT §19.2 T-RUN-PROCESS-SPAWN-INJECTION)
//!
//! The PROGRAM (ABSOLUTE-only — non-absolute spellings are refused at
//! `start`, see the field doc), its WORKING DIRECTORY, its ARGUMENTS and
//! the child's ENVIRONMENT are all fixed at construction by the host —
//! never derived from a model output or a pack string, and never read
//! from THIS process's ambient environment (the crate's host-neutral
//! charter: it reads no env var of its own). The child runs over a
//! CLEARED environment plus exactly the host-supplied map; the effect
//! INPUT reaches it through one variable ([`EFFECT_INPUT_ENV`], canonical
//! JCS). No shell, no interpolation, no inherited secrets.
//!
//! Honest residue (mono-user OS boundary): the input travels in an env var,
//! readable via `/proc/<pid>/environ` by a co-user process — the same
//! trust boundary the whole run store assumes. A future adapter may move
//! it to stdin.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use crate::activity::{Activity, ActivityError, EffectContext, PollOutcome};
use crate::hash::canonical_string;

/// Environment variable carrying the canonical JCS of the effect input.
pub const EFFECT_INPUT_ENV: &str = "SBFB_EFFECT_INPUT";

/// A spawned child whose descendant tree is killable — closed on Windows
/// (job), bounded on Unix (process group; `setsid` escapes, module doc).
#[derive(Debug)]
pub struct KillableChild {
    child: Child,
    /// Unix process-group id (equals the child pid under
    /// `process_group(0)`).
    #[cfg(unix)]
    pgid: i32,
    /// The kill-on-close job the tree lives in.
    #[cfg(windows)]
    job: windows_job::Job,
}

impl KillableChild {
    /// Spawn `cmd` so its descendant tree is killable (Windows: job;
    /// Unix: process group — see the module doc's bounds). The caller
    /// configures program/args/env; this function only adds the
    /// tree-binding (group / job) and platform backstops.
    pub fn spawn(cmd: &mut Command) -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
            #[cfg(target_os = "linux")]
            unsafe {
                cmd.pre_exec(|| {
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let child = cmd.spawn()?;
            let pgid = i32::try_from(child.id()).map_err(std::io::Error::other)?;
            Ok(Self { child, pgid })
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let job = windows_job::Job::create_kill_on_close()?;
            // CREATE_SUSPENDED → assign → resume: the child cannot fork a
            // grandchild before it is inside the job.
            cmd.creation_flags(windows_job::CREATE_SUSPENDED_FLAG);
            let mut child = cmd.spawn()?;
            // A suspended child that never gets assigned/resumed would leak
            // (kill-on-close cannot reach it — it is not in the job yet).
            // Kill it on either failure before propagating.
            if let Err(e) = job.assign(&child) {
                let _ = child.kill();
                return Err(e);
            }
            if let Err(e) = windows_job::resume_process_threads(child.id()) {
                let _ = child.kill();
                return Err(e);
            }
            Ok(Self { child, job })
        }
    }

    /// OS pid of the direct child.
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Durable handle an orphan-reaping resume can act on: the Unix pgid.
    /// `None` on Windows — the kill-on-close job already covers the
    /// crash case, and a kernel handle cannot be persisted.
    pub fn runtime_handle(&self) -> Option<String> {
        #[cfg(unix)]
        {
            Some(format!("pgid:{}", self.pgid))
        }
        #[cfg(windows)]
        {
            None
        }
    }

    /// Non-blocking exit check of the DIRECT child.
    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    /// Kill the tree (job / process group) and reap the direct child.
    pub fn kill_tree(&mut self) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            // The group, not the pid: descendants re-parented inside the
            // group die with it. Errors other than "no such process" are
            // surfaced.
            let rc = unsafe { libc::kill(-self.pgid, libc::SIGKILL) };
            if rc == -1 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() != Some(libc::ESRCH) {
                    return Err(err);
                }
            }
        }
        #[cfg(windows)]
        {
            self.job.terminate()?;
        }
        // Reap the direct child (grandchildren are the OS's to reap).
        let _ = self.child.wait();
        Ok(())
    }
}

/// Best-effort kill of an ORPHAN process group left by a previous
/// incarnation (the durable `runtime_handle`). Honest bounds: Unix pgids
/// are reusable, so this can only be best-effort diagnosis-driven
/// cleanup — never authority; on Windows the kill-on-close job already
/// died with the previous process and this is a no-op.
pub fn kill_process_group(runtime_handle: &str) -> std::io::Result<()> {
    let Some(raw) = runtime_handle.strip_prefix("pgid:") else {
        return Ok(());
    };
    #[cfg(unix)]
    {
        let pgid: i32 = raw
            .parse()
            .map_err(|_| std::io::Error::other("corrupt pgid handle"))?;
        // Guard the catastrophic domain: pgid 0/-1 would signal our own
        // group / every process we may signal.
        if pgid <= 1 {
            return Err(std::io::Error::other("refusing pgid <= 1"));
        }
        let rc = unsafe { libc::kill(-pgid, libc::SIGKILL) };
        if rc == -1 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ESRCH) {
                return Err(err);
            }
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        let _ = raw;
        Ok(())
    }
}

/// A process effect: run one FIXED command per node instance, with the
/// effect input passed by environment. Non-idempotent unless the
/// CONSTRUCTOR declares otherwise — running a program twice is not
/// collapsible by the engine, only the program's own semantics can claim
/// that.
#[derive(Debug)]
pub struct ProcessActivity {
    name: String,
    /// The program, ABSOLUTE-only (ex-Codex r5 K-PROC-PROGRAM-RESOLVE-05
    /// and r6 K-PROC-BARE-PATH-AMBIENT-06): frozen canonical at
    /// construction when it resolves. Anything non-absolute is REFUSED at
    /// `start` — a relative path with separators has a platform-ambiguous
    /// resolution (parent cwd vs child `current_dir`, std doc), and a
    /// BARE name cannot be bound to the scoped `env` `PATH` either: the
    /// Windows std resolver falls back to the parent executable's
    /// directory, the system directories and the PARENT's ambient `PATH`
    /// when the child map misses the name (`resolve_exe`), so one durable
    /// scope could still run two different executables. The engine
    /// resolves NOTHING itself (re-implementing executable lookup on a
    /// security gate is out of charter) — the HOST injects an absolute
    /// path. On WINDOWS the absolute path must additionally end in
    /// `.exe` (case-insensitive, refused at `start` otherwise): std
    /// re-resolves a suffix-less absolute by trying the `path.exe`
    /// SIBLING before `path` itself (`resolve_exe`), so one durable
    /// scope could cover two files and the pick could flip between
    /// attempts. An absolute program that does not resolve yet stays
    /// frozen raw-absolute and its spawn fails as a retryable `Io`
    /// bounded by the retry budget — a program produced by an upstream
    /// step may legitimately appear late.
    program: PathBuf,
    /// The child's working directory, host-supplied at construction and
    /// FROZEN (canonicalised once when resolvable, kept raw otherwise) —
    /// never the ambient cwd of the runtime (ex-Codex r4
    /// K-PROC-CWD-SCOPE-03: an inherited cwd let a resume in another
    /// directory keep the same scope while a relative program or its
    /// relative I/O designated another tree).
    cwd: PathBuf,
    args: Vec<String>,
    idempotent: bool,
    /// Whether a non-zero exit is worth retrying (a flaky build) or
    /// terminal (a deterministic failure).
    failure_retryable: bool,
    /// The child's environment, host-supplied at construction — the crate
    /// reads NO ambient env of its own (charter). The child sees exactly
    /// this map plus [`EFFECT_INPUT_ENV`], over an otherwise cleared env.
    env: BTreeMap<String, String>,
    /// In-flight children keyed by `IdempotencyKey::as_line`.
    running: BTreeMap<String, KillableChild>,
}

impl ProcessActivity {
    /// A process activity for one host-fixed command. `cwd` is the child's
    /// working directory, frozen here (canonical when it resolves, raw
    /// otherwise). Hand an EXISTING directory: a cwd frozen RAW because it
    /// did not resolve keeps a stable scope string while a RELATIVE raw
    /// spelling is resolved by the OS against the runtime's AMBIENT cwd at
    /// spawn — the very dependency the freeze removes — and a cwd still
    /// absent at spawn surfaces as a retryable spawn `Io` error, not a
    /// `Refused` (there is no FS-style state precondition backstop here).
    /// `env` is the exact environment the host grants the child (e.g. a
    /// minimal `PATH`/`SYSTEMROOT`); the crate never reads its own ambient
    /// environment to fill either.
    pub fn new(
        name: impl Into<String>,
        program: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
        args: Vec<String>,
        idempotent: bool,
        failure_retryable: bool,
        env: BTreeMap<String, String>,
    ) -> Self {
        let cwd = cwd.into();
        // Frozen ONCE here (same strategy as the FS root): the durable
        // identity and the spawn use one resolved form, not two live
        // resolutions that could disagree.
        let cwd = cwd.canonicalize().unwrap_or(cwd);
        let program = program.into();
        // An ABSOLUTE program is frozen canonical (aliasing spellings
        // converge to one durable token). Non-absolute spellings stay raw
        // here — canonicalising them against the AMBIENT construction cwd
        // would hide the defect — and are refused at `start` (see the
        // field doc: ABSOLUTE-only contract).
        let program = if program.is_absolute() {
            program.canonicalize().unwrap_or(program)
        } else {
            program
        };
        Self {
            name: name.into(),
            program,
            cwd,
            args,
            idempotent,
            failure_retryable,
            env,
            running: BTreeMap::new(),
        }
    }
}

impl Activity for ProcessActivity {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_idempotent(&self) -> bool {
        self.idempotent
    }

    fn dedup_scope(&self) -> String {
        // The token is the fixed command line, the FROZEN working
        // directory, the granted environment AND the failure policy —
        // every host-fixed input that changes the OBSERVABLE work or the
        // DECISION produced. A resume differing in any of them must NOT
        // pass the scope guard. The cwd is in the scope (ex-Codex r4
        // K-PROC-CWD-SCOPE-03): without it, a resume in another directory
        // kept the same token while a relative program (or its relative
        // I/O) targeted another tree. Canonical JCS (not a separator join,
        // to stay INJECTIVE on the strings: `["a\u{1f}b"]` and `["a","b"]`
        // must not collide — ex-Codex K-PROC-SCOPE-02). Honest bound
        // (ex-Codex r4 K-SCOPE-PATH-ENC-03): `display()` of a non-UTF-8
        // path is LOSSY (exotic Unix spellings can collide before JCS) —
        // same OS mono-user trust boundary as the rest of the store.
        let subject = serde_json::json!({
            "program": self.program.display().to_string(),
            "cwd": self.cwd.display().to_string(),
            "args": self.args,
            "env": self.env,
            "failure_retryable": self.failure_retryable,
        });
        canonical_string(&subject).unwrap_or_else(|_| {
            // Unreachable (no null/non-finite floats); a stable fallback
            // keeps the method total rather than panicking.
            format!("{}", self.program.display())
        })
    }

    fn start(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        // Gate BEFORE any spawn (pattern `confine`): the program is
        // ABSOLUTE-only. A relative path has a platform-ambiguous
        // resolution (parent cwd vs child `current_dir` — std `Command`
        // doc) and a bare name falls back to the PARENT's ambient `PATH`
        // on Windows when the scoped map misses it (`resolve_exe`) — in
        // both cases one durable scope could run two different
        // executables across a resume (ex-Codex r5/r6). Non-retryable:
        // the spelling is wrong, not the environment.
        if !self.program.is_absolute() {
            return Err(ActivityError::Refused(format!(
                "non-absolute program refused (ambient resolution): {}",
                self.program.display()
            )));
        }
        // Windows only: even a canonical ABSOLUTE path is re-resolved by
        // std when it does not end in `.exe` — `resolve_exe` first tries
        // the `path.exe` SIBLING and only then falls back to `path`
        // (windows.rs, ex-Codex r7 K-PROC-ABS-EXE-SUFFIX-07). Two files
        // under one durable scope, and the pick can flip between attempts
        // when the sibling appears or vanishes. Same surface reduction as
        // the other gates: require the exact spelling std will NOT
        // reinterpret. Unix `exec` has no such suffix resolution.
        #[cfg(windows)]
        if !self
            .program
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            return Err(ActivityError::Refused(format!(
                "absolute program without `.exe` refused (std resolves the `.exe` sibling first): {}",
                self.program.display()
            )));
        }
        let input_jcs = canonical_string(&ctx.input)?;
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args)
            // The FROZEN constructor cwd, never the runtime's ambient one —
            // the child's directory is part of the durable identity above.
            .current_dir(&self.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env_clear();
        // Exactly the host-granted environment — no ambient var of ours.
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        // LAST, so a host map carrying the reserved key cannot shadow the
        // engine-computed input.
        cmd.env(EFFECT_INPUT_ENV, input_jcs);
        let child = KillableChild::spawn(&mut cmd)?;
        self.running.insert(ctx.key.as_line(), child);
        Ok(())
    }

    fn poll(&mut self, ctx: &EffectContext) -> Result<PollOutcome, ActivityError> {
        let key = ctx.key.as_line();
        let Some(child) = self.running.get_mut(&key) else {
            return Err(ActivityError::UnknownAttempt);
        };
        match child.try_wait()? {
            None => Ok(PollOutcome::Pending),
            Some(status) => {
                // Exited: drop the tree binding (Windows: closing the job
                // handle reaps any straggler descendant).
                let mut done = self.running.remove(&key).expect("present above");
                let code = status.code().unwrap_or(-1);
                if code == 0 {
                    let _ = done.kill_tree();
                    let mut facts = BTreeMap::new();
                    facts.insert("exit_code".to_string(), "0".to_string());
                    Ok(PollOutcome::Ready {
                        output: serde_json::json!({ "exit_code": 0 }),
                        facts,
                    })
                } else {
                    let _ = done.kill_tree();
                    Ok(PollOutcome::Failed {
                        retryable: self.failure_retryable,
                        reason: format!("exit code {code}"),
                    })
                }
            }
        }
    }

    fn cancel(&mut self, ctx: &EffectContext) -> Result<(), ActivityError> {
        if let Some(mut child) = self.running.remove(&ctx.key.as_line()) {
            child.kill_tree()?;
        }
        Ok(())
    }

    fn runtime_handle(&self, ctx: &EffectContext) -> Option<String> {
        self.running
            .get(&ctx.key.as_line())
            .and_then(KillableChild::runtime_handle)
    }

    fn reap_orphan(
        &mut self,
        _ctx: &EffectContext,
        runtime_handle: Option<&str>,
    ) -> Result<(), ActivityError> {
        if let Some(handle) = runtime_handle {
            kill_process_group(handle)?;
        }
        Ok(())
    }
}

/// Minimal Windows Job Object wrapper — the daemon's `local_worker.rs`
/// module, plus suspended-spawn support.
#[cfg(windows)]
mod windows_job {
    use std::os::windows::io::AsRawHandle;

    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    };
    use windows::Win32::System::Threading::{
        CREATE_SUSPENDED, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
    };

    /// `CREATE_SUSPENDED` as the raw flag `CommandExt::creation_flags`
    /// takes.
    pub const CREATE_SUSPENDED_FLAG: u32 = CREATE_SUSPENDED.0;

    #[derive(Debug)]
    pub struct Job(HANDLE);

    // A kernel handle; moving it across threads is sound (closed once, on
    // drop).
    unsafe impl Send for Job {}

    impl Job {
        pub fn create_kill_on_close() -> std::io::Result<Self> {
            unsafe {
                let handle = CreateJobObjectW(None, None).map_err(std::io::Error::other)?;
                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
                .map_err(std::io::Error::other)?;
                Ok(Job(handle))
            }
        }

        pub fn assign(&self, child: &std::process::Child) -> std::io::Result<()> {
            let h = HANDLE(child.as_raw_handle() as isize);
            unsafe { AssignProcessToJobObject(self.0, h).map_err(std::io::Error::other) }
        }

        pub fn terminate(&self) -> std::io::Result<()> {
            unsafe { TerminateJobObject(self.0, 1).map_err(std::io::Error::other) }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // Kill-on-close: closing the last handle kills whatever is
            // left in the job — the crash-cleanup guarantee itself.
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    /// Resume every thread of `pid` — a `CREATE_SUSPENDED` process has
    /// exactly one, its primary thread, which std's `Child` does not
    /// expose; the toolhelp snapshot finds it.
    pub fn resume_process_threads(pid: u32) -> std::io::Result<()> {
        unsafe {
            let snapshot =
                CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(std::io::Error::other)?;
            let mut entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            let mut resumed = false;
            let mut more = Thread32First(snapshot, &mut entry).is_ok();
            while more {
                if entry.th32OwnerProcessID == pid
                    && let Ok(thread) = OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID)
                {
                    // ResumeThread returns the previous suspend count, or
                    // (DWORD)-1 on failure — only count a real resume.
                    let rc = ResumeThread(thread);
                    let _ = CloseHandle(thread);
                    if rc != u32::MAX {
                        resumed = true;
                    }
                }
                more = Thread32Next(snapshot, &mut entry).is_ok();
            }
            let _ = CloseHandle(snapshot);
            if resumed {
                Ok(())
            } else {
                Err(std::io::Error::other(format!(
                    "no thread of pid {pid} found to resume"
                )))
            }
        }
    }
}
