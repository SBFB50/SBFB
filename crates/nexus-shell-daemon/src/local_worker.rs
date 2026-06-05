// SPDX-License-Identifier: AGPL-3.0-or-later
//! On-demand local compute worker (2026-06-05 platform remediation,
//! hotfix #5 maillon A).
//!
//! `result_sync` (maillon B) carries a worker's result back into the
//! coordinator DB, but a fresh node has **no worker** to execute the
//! task in the first place — `EXECUTE Reseau` therefore submitted a
//! task that nothing ever claimed and the network arm timed out.
//!
//! This module spawns a co-located `nexus-worker` process **on
//! demand** — lazily, the first time a task is submitted — so a node
//! is its own executor out of the box without the user running
//! `nexus-worker register/join/start` by hand (PO decision: on-demand,
//! cold-start at first use is accepted).
//!
//! ## Why the daemon (not the launcher) spawns it
//!
//! The daemon holds the submit signal (`/api/v1/tasks/submit`) and is
//! the coordinator that mints the invite ticket for its own project
//! doc, so it is the natural owner. The worker runs as a **separate OS
//! process** (not an in-daemon engine task) to preserve the worker /
//! daemon split: the executor keeps its own consent gate, GPU caps and
//! process isolation.
//!
//! ## Provisioning (in-process, via `nexus-worker-core`)
//!
//! A dedicated, daemon-managed worker home under the shell-daemon dir
//! (`local-worker/`) is recreated fresh on first spawn so the enrolled
//! `tasks_doc_ticket` always carries the live node addresses. The
//! daemon writes `worker.toml`, the Ed25519 key, the allowlist
//! (enrolling its project doc by a fresh `share_write` ticket) and a
//! `consent.json`. **Consent is `Whitelist[project_doc_id]`** — least
//! privilege: the local worker only serves the node's own project doc,
//! not "any task" (the default `OwnProjects` level would *reject* the
//! coordinator's tasks because their `project_id` is the doc id, not
//! the worker's node id — a live-smoke finding).
//!
//! ## Lifetime
//!
//! Killed on graceful daemon shutdown. For an abrupt daemon kill (the
//! launcher `.kill()`s the daemon on Windows, so the daemon's
//! `shutdown()` never runs) the worker is tied to the daemon's life by
//! a **Windows Job Object** (`KILL_ON_JOB_CLOSE`) / **Unix
//! `PR_SET_PDEATHSIG`**, so it never outlives its parent.

use std::path::PathBuf;
use std::sync::Arc;

use nexus_core_rs::docs::DocHandle;
use nexus_worker_core::allowlist::{Allowlist, NewProject};
use nexus_worker_core::config::{WorkerConfig, WorkerPaths};
use nexus_worker_core::consent::{ConsentConfig, ConsentLevel};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Env var to opt a node out of running a local compute worker
/// entirely (the daemon then never spawns one; `EXECUTE Reseau` falls
/// back to whatever remote workers exist).
const DISABLE_ENV: &str = "SBFB_NO_LOCAL_WORKER";
/// Env var (tests / CI) to make the spawned worker use the
/// deterministic no-network Ollama stub instead of a real backend.
const STUB_ENV: &str = "SBFB_LOCAL_WORKER_STUB";

/// Supervises the at-most-one co-located worker process.
pub struct LocalWorkerSupervisor {
    state: Mutex<SupervisorState>,
}

impl std::fmt::Debug for LocalWorkerSupervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Opaque: the inner `Child` / Job Object handle carry nothing
        // useful for a debug dump and reading them would need the lock.
        f.debug_struct("LocalWorkerSupervisor")
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
struct SupervisorState {
    child: Option<std::process::Child>,
    /// Windows: the Job Object the worker is assigned to. Held for
    /// the daemon's whole life — dropping/closing it kills the
    /// worker (`KILL_ON_JOB_CLOSE`). `()` on non-Windows.
    #[cfg(windows)]
    job: Option<windows_job::Job>,
}

impl Default for LocalWorkerSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalWorkerSupervisor {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(SupervisorState::default()),
        }
    }

    /// Ensure a local worker is running, provisioning + spawning one
    /// on first call. Idempotent: a no-op while a worker is alive.
    /// Safe to call (fire-and-forget) on every task submit.
    pub async fn ensure_spawned(&self, project_doc: Arc<DocHandle>) {
        if std::env::var_os(DISABLE_ENV).is_some() {
            return;
        }

        let mut st = self.state.lock().await;

        // Already running? Reap a dead child, otherwise no-op.
        if let Some(child) = st.child.as_mut() {
            match child.try_wait() {
                Ok(None) => return, // still alive
                Ok(Some(status)) => {
                    warn!(?status, "local worker exited; respawning on demand");
                    st.child = None;
                    #[cfg(windows)]
                    {
                        st.job = None;
                    }
                }
                Err(e) => {
                    warn!(error = %e, "local worker status check failed; assuming dead");
                    st.child = None;
                    #[cfg(windows)]
                    {
                        st.job = None;
                    }
                }
            }
        }

        match self.provision_and_spawn(&project_doc).await {
            Ok((child, _job_opt)) => {
                info!(pid = child.id(), "on-demand local worker spawned");
                st.child = Some(child);
                #[cfg(windows)]
                {
                    st.job = _job_opt;
                }
            }
            Err(e) => {
                warn!(error = %e, "failed to spawn on-demand local worker");
            }
        }
    }

    /// Kill the worker on graceful daemon shutdown. The Job Object /
    /// PDEATHSIG covers the abrupt path; this is the clean one.
    pub async fn shutdown(&self) {
        let mut st = self.state.lock().await;
        if let Some(mut child) = st.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            info!("on-demand local worker stopped");
        }
        #[cfg(windows)]
        {
            st.job = None;
        }
    }

    #[cfg(windows)]
    async fn provision_and_spawn(
        &self,
        project_doc: &Arc<DocHandle>,
    ) -> anyhow::Result<(std::process::Child, Option<windows_job::Job>)> {
        let (paths, sbfb_home) = provision(project_doc).await?;
        let mut cmd = base_command(&paths.config_file, &sbfb_home)?;
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
        // Spawn then immediately assign to a kill-on-close Job Object.
        // The worker spawns no children, so the sub-millisecond window
        // before assignment carries no escape risk.
        let child = cmd.spawn()?;
        let job = windows_job::Job::create_kill_on_close()?;
        if let Err(e) = job.assign(&child) {
            warn!(error = %e, "could not assign local worker to Job Object; \
                   it will still be killed on graceful shutdown");
        }
        Ok((child, Some(job)))
    }

    #[cfg(unix)]
    async fn provision_and_spawn(
        &self,
        project_doc: &Arc<DocHandle>,
    ) -> anyhow::Result<(std::process::Child, Option<()>)> {
        let (paths, sbfb_home) = provision(project_doc).await?;
        let mut cmd = base_command(&paths.config_file, &sbfb_home)?;
        // Tie the worker's life to the daemon's: if the daemon dies
        // (even on SIGKILL) the kernel sends the worker SIGTERM.
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = cmd.spawn()?;
        Ok((child, None))
    }
}

/// Build the `nexus-worker ... start --headless` command, resolving
/// the worker binary next to the daemon exe (then PATH).
fn base_command(
    config_file: &std::path::Path,
    sbfb_home: &std::path::Path,
) -> anyhow::Result<std::process::Command> {
    let exe = std::env::current_exe().ok();
    let sibling = exe
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join(worker_bin_name()));
    let program = match sibling {
        Some(p) if p.exists() => p.into_os_string(),
        _ => worker_bin_name().into(),
    };

    let mut cmd = std::process::Command::new(program);
    cmd.arg("--config")
        .arg(config_file)
        .arg("start")
        .arg("--headless")
        // Consent + usage live under SBFB_HOME, isolated from any
        // hand-run worker the user owns.
        .env("SBFB_HOME", sbfb_home);
    if std::env::var_os(STUB_ENV).is_some() {
        cmd.arg("--stub-ollama");
    }
    Ok(cmd)
}

fn worker_bin_name() -> &'static str {
    if cfg!(windows) {
        "nexus-worker.exe"
    } else {
        "nexus-worker"
    }
}

/// Resolve the dedicated worker home, (re)provision a fresh
/// `worker.toml` + key + allowlist (enrolling the project doc by a
/// fresh write ticket) + `consent.json`. Returns the worker paths and
/// the SBFB_HOME the consent file lives in.
async fn provision(project_doc: &Arc<DocHandle>) -> anyhow::Result<(WorkerPaths, PathBuf)> {
    let home = local_worker_home()?;
    // Fresh each provision so the enrolled ticket always carries the
    // live node addresses (a stale ticket from a previous boot points
    // at dead UDP addrs). Only reached when no worker is alive.
    let _ = std::fs::remove_dir_all(&home);

    let paths = WorkerPaths::resolve(Some(home.join("worker.toml")))
        .map_err(|e| anyhow::anyhow!("worker paths resolve: {e}"))?;
    paths
        .ensure_dirs()
        .map_err(|e| anyhow::anyhow!("worker ensure_dirs: {e}"))?;

    let mut cfg = WorkerConfig::default();
    cfg.identity.name = "local-worker".to_string();
    cfg.save(&paths.config_file)
        .map_err(|e| anyhow::anyhow!("worker.toml save: {e}"))?;

    let key_path = cfg.resolve_secret_key_path(&paths);
    nexus_core_rs::KeyPair::load_or_generate(&key_path)
        .map_err(|e| anyhow::anyhow!("worker keypair: {e}"))?;

    let project_id = project_doc.id().to_string();
    let ticket = project_doc
        .share_write()
        .await
        .map_err(|e| anyhow::anyhow!("share_write project doc: {e}"))?
        .to_string();

    {
        let allowlist = Allowlist::open(paths.default_allowlist_db())
            .map_err(|e| anyhow::anyhow!("open allowlist: {e}"))?;
        allowlist
            .enroll(NewProject {
                id: project_id.clone(),
                name: "Local node".to_string(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: Some(ticket),
            })
            .map_err(|e| anyhow::anyhow!("enroll project doc: {e}"))?;
    } // drop the SQLite connection before the worker opens the same db

    // SBFB_HOME for consent/usage, isolated under the worker home.
    let sbfb_home = home.join("sbfb");
    std::fs::create_dir_all(&sbfb_home)?;
    let mut consent = ConsentConfig::default_for("local-worker");
    // Whitelist the node's own project doc only (least privilege).
    consent.level = ConsentLevel::Whitelist;
    consent.allowed_project_ids.insert(project_id);
    consent
        .save_atomic(&sbfb_home.join("consent.json"))
        .map_err(|e| anyhow::anyhow!("consent.json save: {e}"))?;

    Ok((paths, sbfb_home))
}

/// `<shell_daemon_dir>/local-worker/`.
fn local_worker_home() -> anyhow::Result<PathBuf> {
    let dir = nexus_shell_daemon_core::paths::shell_daemon_dir()
        .ok_or_else(|| anyhow::anyhow!("could not resolve shell-daemon dir for local worker"))?;
    Ok(dir.join("local-worker"))
}

/// Minimal Windows Job Object wrapper — assigns the worker process so
/// the daemon's death (even an abrupt `TerminateProcess` from the
/// launcher) closes the job and kills the worker.
#[cfg(windows)]
mod windows_job {
    use std::os::windows::io::AsRawHandle;

    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };

    pub struct Job(HANDLE);

    // The Job handle is just a kernel handle; moving it across threads
    // is sound (we only ever close it on drop).
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    impl Job {
        pub fn create_kill_on_close() -> anyhow::Result<Self> {
            unsafe {
                let handle = CreateJobObjectW(None, None)
                    .map_err(|e| anyhow::anyhow!("CreateJobObject: {e}"))?;
                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
                .map_err(|e| anyhow::anyhow!("SetInformationJobObject: {e}"))?;
                Ok(Job(handle))
            }
        }

        pub fn assign(&self, child: &std::process::Child) -> anyhow::Result<()> {
            let h = HANDLE(child.as_raw_handle() as isize);
            unsafe {
                AssignProcessToJobObject(self.0, h)
                    .map_err(|e| anyhow::anyhow!("AssignProcessToJobObject: {e}"))?;
            }
            Ok(())
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provisioning is the unit-testable half (the OS process spawn is
    // covered by the live smoke). Drives a real iroh doc + the real
    // nexus-worker-core provisioning and asserts the files the worker
    // will read are correct: consent Whitelists the doc, the allowlist
    // enrolls it with a non-empty ticket, worker.toml exists.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn provision_writes_consent_allowlist_and_config() {
        // Isolate the shell-daemon dir so we use a temp local-worker home.
        let tmp = tempfile::tempdir().expect("tmp");
        unsafe {
            std::env::set_var(
                nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV,
                tmp.path(),
            );
        }

        let node = nexus_core_rs::create_node().await.expect("node");
        let docs = nexus_core_rs::docs::DocsClient::new(node.docs());
        let doc = Arc::new(docs.create_doc().await.expect("doc"));
        let doc_id = doc.id().to_string();

        let (paths, sbfb_home) = provision(&doc).await.expect("provision");

        // worker.toml written.
        assert!(paths.config_file.exists(), "worker.toml must exist");

        // Allowlist enrolled the project doc with a real ticket.
        let allowlist = Allowlist::open(paths.default_allowlist_db()).expect("open allowlist");
        let projects = allowlist.list().expect("list");
        let p = projects
            .iter()
            .find(|p| p.id == doc_id)
            .expect("project doc enrolled");
        assert!(p.enabled, "enrolled project must be enabled");
        assert!(
            p.tasks_doc_ticket.as_deref().is_some_and(|t| !t.is_empty()),
            "enrolled project must carry a non-empty tasks_doc_ticket"
        );

        // Consent Whitelists exactly the project doc (least privilege),
        // NOT the default OwnProjects (which would reject the task).
        let consent = ConsentConfig::load_or_default(&sbfb_home.join("consent.json"), "x")
            .expect("load consent");
        assert_eq!(consent.level, ConsentLevel::Whitelist);
        assert!(
            consent.allowed_project_ids.contains(&doc_id),
            "consent must whitelist the node's project doc"
        );

        unsafe {
            std::env::remove_var(nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV);
        }
    }
}
