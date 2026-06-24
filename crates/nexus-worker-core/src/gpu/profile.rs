// SPDX-License-Identifier: AGPL-3.0-or-later
//! NVML utilization + duration profile (Sprint 22 Phase D —
//! log-only baseline foundation for the Sprint 24 random re-run
//! sampling detection of `C-ComputeTheft`).
//!
//! This module is **observation-only**: it persists periodic
//! per-device snapshots (gpu_util, vram_used_mb, running compute
//! processes) into a small SQLite database so that a consumer-side
//! re-run sampler can join "task signed by worker N at time T" with
//! "what was worker N's GPU actually doing at time T". This module
//! ships no anomaly detector and no enforcement — that is out of
//! scope here per `docs/security/HARDENING_ROADMAP.md` §3 ligne
//! 280-281 and §15 pipeline ligne 793-794.
//!
//! ## Wire / pre-launch posture
//!
//! Schema lives in `<data_dir>/nvml_profile.sqlite3` (resolved
//! by [`crate::config::WorkerPaths::default_nvml_profile_db`]).
//! The file is **never published** over iroh, gossip, or the
//! coordinator HTTP proxy — purely a per-worker on-disk cache.
//! No `*_VERSION` field is therefore introduced (cf. CLAUDE.md
//! pre-launch protocol policy).
//!
//! ## Integration with existing `gpu/` module
//!
//! [`NvmlProfile::open_with_backend`] reuses the [`Arc<Nvml>`]
//! already initialised by [`crate::gpu::NvmlBackend`] via its
//! [`crate::gpu::NvmlBackend::inner`] accessor — so the worker
//! never calls `Nvml::init` twice when both the engine snapshot
//! loop (W9) and the profile sampler are active.
//! [`NvmlProfile::try_new_standalone`] gives a fallback path for
//! callers (tests, CLI tools) that do not already hold a backend.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nvml_wrapper::Nvml;
use nvml_wrapper::error::NvmlError;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;

use super::NvmlBackend;

/// Default sampling cadence. Conservative enough that on an
/// RTX 5080 the bench reported ≤ 1% CPU overhead (cf. plan
/// §7.4 acceptance criterion). Operators can override this via
/// [`NvmlProfile::with_sampling_interval`] if they need denser
/// or sparser samples.
pub const DEFAULT_SAMPLING_INTERVAL: Duration = Duration::from_secs(5);

// =================================================================
// Errors
// =================================================================

/// Errors raised by [`NvmlProfile`] operations.
#[derive(Debug, Error)]
pub enum NvmlProfileError {
    /// SQLite query failed — opening the file, applying the
    /// schema, inserting a sample row, or computing the window
    /// aggregate. Wraps the underlying [`rusqlite::Error`] for
    /// debuggability.
    #[error("nvml profile sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// NVML backend call failed — `Nvml::init` returned an error
    /// (no driver / no `libnvidia-ml`), or a per-call NVML query
    /// failed. The engine should treat this as "no profile this
    /// tick" rather than crash.
    #[error("nvml profile nvml error: {0}")]
    Nvml(#[from] NvmlError),

    /// Failed to create the parent directory for the database
    /// file. Surfaces the `std::io::Error` so callers can tell
    /// permission-denied apart from disk-full.
    #[error("nvml profile io error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Internal connection mutex was poisoned (a previous query
    /// panicked while holding it). Should be unreachable in
    /// practice — every call site uses scoped guards.
    #[error("nvml profile connection mutex poisoned")]
    Poisoned,

    /// JSON serialisation of the running-compute-processes
    /// payload failed. Wraps the underlying [`serde_json::Error`].
    #[error("nvml profile json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience alias used throughout this module.
pub type NvmlProfileResult<T> = Result<T, NvmlProfileError>;

// =================================================================
// Sample + window stats data structures
// =================================================================

/// Per-process snapshot stored inside `compute_processes_json`.
///
/// We keep `pid` and the NVML-reported `last_seen_timestamp` so
/// the Sprint 24 re-run sampler can decide whether a process was
/// still alive at the moment a result was returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvmlComputeProcess {
    pub pid: u32,
    pub used_gpu_memory_mb: u64,
    /// Wall-clock timestamp (UNIX seconds) at the moment the
    /// sample was captured. NVML's `ProcessInfo` does not expose
    /// a per-entry timestamp accessor in 0.12.1 — the field name
    /// matches the future-proof shape the Sprint 24 sampler
    /// expects (refresh to a real per-process timestamp once
    /// NVML wraps `nvmlDeviceGetProcessUtilization`'s sample
    /// timestamps; tracked in PATTERNS.md).
    pub last_seen_timestamp: i64,
}

/// One persisted profile row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NvmlSample {
    /// Wall-clock seconds since UNIX epoch. Persisted as
    /// SQLite INTEGER for cheap range queries.
    pub timestamp: i64,
    /// GPU compute utilization at sample time, 0..=100.
    pub gpu_util: u8,
    /// Used video memory in MiB at sample time.
    pub vram_used_mb: u64,
    /// Compute processes seen at sample time, JSON-encoded.
    pub compute_processes: Vec<NvmlComputeProcess>,
}

/// Aggregated stats over a backwards-looking window.
///
/// Returned by [`NvmlProfile::stats_for_window`]. `last_seen_
/// timestamps` is the deduplicated set of per-process timestamps
/// observed over the window so that the Sprint 24 sampler can
/// efficiently ask "was process X still alive at any moment in
/// the last N seconds".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NvmlWindowStats {
    pub gpu_util_avg: f32,
    pub gpu_util_p95: f32,
    pub vram_used_avg_mb: u64,
    pub compute_processes_count: u32,
    pub last_seen_timestamps: Vec<i64>,
}

// =================================================================
// SQL helpers (module-level so tests can drive them with a bare
// in-memory Connection without needing a real Nvml instance).
// =================================================================

const SCHEMA_SQL: &str = "\
    CREATE TABLE IF NOT EXISTS nvml_samples ( \
        timestamp INTEGER NOT NULL, \
        gpu_util INTEGER NOT NULL, \
        vram_used_mb INTEGER NOT NULL, \
        compute_processes_json TEXT NOT NULL \
    ); \
    CREATE INDEX IF NOT EXISTS nvml_samples_ts_idx \
        ON nvml_samples (timestamp);";

fn prepare_schema(conn: &mut Connection) -> NvmlProfileResult<()> {
    // Same pragma posture as `allowlist.rs`: WAL survives crashes
    // and concurrent readers, busy_timeout absorbs short-lived
    // contention without surfacing errors. NORMAL synchronous
    // is fine because losing the last few seconds of profile data
    // on a hard crash is a non-event.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(Duration::from_millis(500))?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

fn persist_sample_row(conn: &Connection, sample: &NvmlSample) -> NvmlProfileResult<()> {
    let json = serde_json::to_string(&sample.compute_processes)?;
    conn.execute(
        "INSERT INTO nvml_samples \
            (timestamp, gpu_util, vram_used_mb, compute_processes_json) \
         VALUES (?1, ?2, ?3, ?4)",
        params![
            sample.timestamp,
            sample.gpu_util as i64,
            sample.vram_used_mb as i64,
            json,
        ],
    )?;
    Ok(())
}

fn query_window_stats(
    conn: &Connection,
    window_start_ts: i64,
) -> NvmlProfileResult<NvmlWindowStats> {
    // Average + count via SQL aggregates. P95 is computed in Rust
    // because portable SQLite (without window functions enabled
    // by every distribution) makes percentile awkward.
    let mut stmt = conn.prepare(
        "SELECT gpu_util, vram_used_mb, compute_processes_json \
         FROM nvml_samples \
         WHERE timestamp >= ?1 \
         ORDER BY gpu_util ASC",
    )?;
    let rows = stmt.query_map(params![window_start_ts], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut utils: Vec<u8> = Vec::new();
    let mut vram_sum: u128 = 0;
    let mut processes_count: u32 = 0;
    let mut last_seen_set: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();

    for row in rows {
        let (util, vram, json) = row?;
        utils.push(util.clamp(0, 100) as u8);
        vram_sum = vram_sum.saturating_add(vram.max(0) as u128);
        let processes: Vec<NvmlComputeProcess> = serde_json::from_str(&json)?;
        processes_count = processes_count.saturating_add(processes.len() as u32);
        for p in processes {
            last_seen_set.insert(p.last_seen_timestamp);
        }
    }

    if utils.is_empty() {
        return Ok(NvmlWindowStats::default());
    }

    let n = utils.len() as u32;
    let gpu_util_avg = utils.iter().map(|u| *u as u32).sum::<u32>() as f32 / n as f32;
    // Nearest-rank P95 on a sorted-ascending slice. Index = ceil(0.95 * n) − 1,
    // clamped into bounds.
    let p95_idx = ((utils.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(utils.len() - 1);
    let gpu_util_p95 = utils[p95_idx] as f32;
    let vram_used_avg_mb = (vram_sum / n as u128) as u64;

    Ok(NvmlWindowStats {
        gpu_util_avg,
        gpu_util_p95,
        vram_used_avg_mb,
        compute_processes_count: processes_count,
        last_seen_timestamps: last_seen_set.into_iter().collect(),
    })
}

// =================================================================
// NvmlProfile handle
// =================================================================

/// Periodic-sampler handle to a per-worker NVML profile database.
///
/// Construct via [`NvmlProfile::open_with_backend`] (preferred —
/// reuses an existing [`NvmlBackend`]'s `Arc<Nvml>`) or
/// [`NvmlProfile::try_new_standalone`] (initialises a fresh NVML
/// handle, used by CLI tools that have no engine running).
pub struct NvmlProfile {
    nvml: Arc<Nvml>,
    conn: Mutex<Connection>,
    db_path: Option<PathBuf>,
    sampling_interval: Duration,
}

impl std::fmt::Debug for NvmlProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NvmlProfile")
            .field("db_path", &self.db_path)
            .field("sampling_interval", &self.sampling_interval)
            .finish()
    }
}

impl NvmlProfile {
    /// Open the on-disk profile database at `db_path` and reuse
    /// the [`NvmlBackend`]'s already-initialised NVML handle.
    ///
    /// Missing parent directories are created via
    /// `std::fs::create_dir_all` before the SQLite file is
    /// opened, mirroring [`crate::allowlist::Allowlist::open`]'s
    /// contract so callers do not need to `ensure_dirs`.
    pub fn open_with_backend(
        backend: &NvmlBackend,
        db_path: impl AsRef<Path>,
    ) -> NvmlProfileResult<Self> {
        let nvml = backend.shared_handle();
        Self::open_inner(nvml, db_path.as_ref())
    }

    /// Initialise a fresh NVML handle and open the on-disk
    /// profile database. Fails with [`NvmlProfileError::Nvml`]
    /// when no NVIDIA driver is reachable.
    pub fn try_new_standalone(db_path: impl AsRef<Path>) -> NvmlProfileResult<Self> {
        let nvml = Arc::new(Nvml::init()?);
        Self::open_inner(nvml, db_path.as_ref())
    }

    /// Open an ephemeral in-memory profile database backed by an
    /// existing [`NvmlBackend`]. Useful for integration tests
    /// that exercise the live sampler path without polluting the
    /// developer's `<data_dir>`.
    pub fn open_in_memory_with_backend(backend: &NvmlBackend) -> NvmlProfileResult<Self> {
        let nvml = backend.shared_handle();
        let mut conn = Connection::open_in_memory()?;
        prepare_schema(&mut conn)?;
        Ok(Self {
            nvml,
            conn: Mutex::new(conn),
            db_path: None,
            sampling_interval: DEFAULT_SAMPLING_INTERVAL,
        })
    }

    fn open_inner(nvml: Arc<Nvml>, path: &Path) -> NvmlProfileResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| NvmlProfileError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }
        let mut conn = Connection::open(path)?;
        prepare_schema(&mut conn)?;
        Ok(Self {
            nvml,
            conn: Mutex::new(conn),
            db_path: Some(path.to_path_buf()),
            sampling_interval: DEFAULT_SAMPLING_INTERVAL,
        })
    }

    /// Override the default 5-second sampling cadence. Returns
    /// `self` so the call can be chained on the constructor.
    pub fn with_sampling_interval(mut self, interval: Duration) -> Self {
        self.sampling_interval = interval;
        self
    }

    /// Path to the underlying SQLite file, or `None` for the
    /// in-memory test handle.
    pub fn db_path(&self) -> Option<&Path> {
        self.db_path.as_deref()
    }

    /// Sampling interval used by [`Self::start_sampling`].
    pub fn sampling_interval(&self) -> Duration {
        self.sampling_interval
    }

    /// Capture one snapshot of `gpu_index` and persist it.
    ///
    /// Returns the [`NvmlSample`] that was inserted so callers
    /// can log it without re-querying NVML. NVML errors are
    /// propagated (the caller decides whether to log + skip or
    /// abort the sampler loop).
    pub fn sample_now(&self, gpu_index: u32) -> NvmlProfileResult<NvmlSample> {
        let device = self.nvml.device_by_index(gpu_index)?;
        let util = device.utilization_rates()?;
        let mem = device.memory_info()?;
        // `running_compute_processes` returns Vec<ProcessInfo>.
        // The per-entry `last_seen_timestamp` accessor is what a
        // consumer-side re-run sampler joins against — until
        // 0.11.0 you had to call `process_utilization_stats`
        // separately. Keep both shapes in mind: ProcessInfo's
        // `used_gpu_memory` is a `UsedGpuMemory` enum (Used /
        // Unavailable) on recent wrapper versions.
        let processes_raw = device.running_compute_processes()?;
        let now = current_unix_seconds();
        let processes: Vec<NvmlComputeProcess> = processes_raw
            .into_iter()
            .map(|p| NvmlComputeProcess {
                pid: p.pid,
                used_gpu_memory_mb: extract_used_memory_mb(&p),
                // ProcessInfo does not currently expose a per-
                // entry timestamp — stamp the sample wall-clock
                // so the Sprint 24 sampler can still bound
                // "process was alive at T". When NVML exposes a
                // per-process last_seen this is the field to
                // refresh.
                last_seen_timestamp: now,
            })
            .collect();

        let sample = NvmlSample {
            timestamp: now,
            gpu_util: util.gpu.min(100) as u8,
            vram_used_mb: bytes_to_mib(mem.used),
            compute_processes: processes,
        };
        self.with_conn(|conn| persist_sample_row(conn, &sample))?;
        Ok(sample)
    }

    /// Aggregate the last `window` of samples into [`NvmlWindowStats`].
    ///
    /// Returns [`NvmlWindowStats::default()`] when no rows fall
    /// inside the window — callers are responsible for treating
    /// "no signal" as "no anomaly" rather than dividing by zero.
    pub fn stats_for_window(&self, window: Duration) -> NvmlProfileResult<NvmlWindowStats> {
        let now = current_unix_seconds();
        let window_start = now.saturating_sub(window.as_secs() as i64);
        self.with_conn(|conn| query_window_stats(conn, window_start))
    }

    /// Spawn a background tokio task that calls
    /// [`Self::sample_now`] every [`Self::sampling_interval`].
    ///
    /// Takes `Arc<Self>` because the spawned task needs a
    /// `'static` reference to the handle. The returned
    /// [`JoinHandle`] can be aborted by the caller during
    /// shutdown — a successful abort is reported via
    /// [`tokio::task::JoinError::is_cancelled`].
    pub fn start_sampling(self: Arc<Self>, gpu_index: u32) -> JoinHandle<()> {
        let interval_dur = self.sampling_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval_dur);
            // Skip missed ticks if the sampler is paused (sleeping
            // host, debugger break) so we don't burst-fire on
            // resume — the baseline cares about steady-state
            // signal, not contention spikes.
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                if let Err(err) = self.sample_now(gpu_index) {
                    tracing::debug!(
                        error = %err,
                        gpu_index,
                        "nvml profile sampler tick failed (will retry on next tick)"
                    );
                }
            }
        })
    }

    /// Hand a borrowed [`Connection`] to `f` under the internal
    /// mutex. Used by every read/write helper above.
    fn with_conn<T, F>(&self, f: F) -> NvmlProfileResult<T>
    where
        F: FnOnce(&Connection) -> NvmlProfileResult<T>,
    {
        let guard = self.conn.lock().map_err(|_| NvmlProfileError::Poisoned)?;
        f(&guard)
    }

    /// Number of rows currently persisted. Cheap because the
    /// table is single-column-indexed by timestamp; intended for
    /// tests and the `stats` CLI subcommand.
    pub fn row_count(&self) -> NvmlProfileResult<u64> {
        self.with_conn(|conn| {
            let count: Option<i64> = conn
                .query_row("SELECT COUNT(*) FROM nvml_samples", [], |row| row.get(0))
                .optional()?;
            Ok(count.unwrap_or(0).max(0) as u64)
        })
    }
}

// =================================================================
// Helpers
// =================================================================

fn current_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn bytes_to_mib(b: u64) -> u64 {
    b / (1024 * 1024)
}

fn extract_used_memory_mb(p: &nvml_wrapper::struct_wrappers::device::ProcessInfo) -> u64 {
    use nvml_wrapper::enums::device::UsedGpuMemory;
    match p.used_gpu_memory {
        UsedGpuMemory::Used(bytes) => bytes_to_mib(bytes),
        UsedGpuMemory::Unavailable => 0,
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn fresh_in_memory_conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("open in-memory");
        prepare_schema(&mut conn).expect("schema prepare");
        conn
    }

    #[test]
    fn new_creates_schema() {
        let conn = fresh_in_memory_conn();
        // The `nvml_samples` table must exist with the four
        // columns the persist helper writes to. SQLite's
        // `pragma_query_value` lets us list table columns
        // without parsing CREATE TABLE.
        let mut stmt = conn
            .prepare("PRAGMA table_info(nvml_samples)")
            .expect("pragma");
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query_map")
            .collect::<Result<_, _>>()
            .expect("collect");
        assert_eq!(
            cols,
            vec![
                "timestamp".to_string(),
                "gpu_util".to_string(),
                "vram_used_mb".to_string(),
                "compute_processes_json".to_string(),
            ],
            "table must expose exactly the four columns persist_sample_row writes"
        );
    }

    #[test]
    fn sampling_persists_row() {
        let conn = fresh_in_memory_conn();
        let sample = NvmlSample {
            timestamp: 1_700_000_000,
            gpu_util: 42,
            vram_used_mb: 1024,
            compute_processes: vec![NvmlComputeProcess {
                pid: 4321,
                used_gpu_memory_mb: 512,
                last_seen_timestamp: 1_700_000_000,
            }],
        };
        persist_sample_row(&conn, &sample).expect("persist");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nvml_samples", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 1);

        // Round-trip the JSON column to make sure the per-process
        // payload survives intact (this is what a consumer-side
        // re-run sampler joins against).
        let json: String = conn
            .query_row(
                "SELECT compute_processes_json FROM nvml_samples LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select json");
        let processes: Vec<NvmlComputeProcess> =
            serde_json::from_str(&json).expect("deserialise processes");
        assert_eq!(processes, sample.compute_processes);
    }

    #[test]
    fn stats_for_window_empty() {
        let conn = fresh_in_memory_conn();
        let stats = query_window_stats(&conn, 0).expect("stats");
        // Default = all zeros, no division by zero, no panic.
        assert_eq!(stats, NvmlWindowStats::default());
    }

    #[test]
    fn stats_for_window_computes_avg_p95() {
        let conn = fresh_in_memory_conn();
        // Insert 100 samples with util = 1, 2, ..., 100 inside
        // the window. avg should be 50.5, p95 nearest-rank on
        // sorted ascending of 100 values lands on index ceil(95)
        // − 1 = 94 → value 95 (1-indexed value 95).
        let base_ts = 1_700_000_000_i64;
        for i in 1..=100u8 {
            let s = NvmlSample {
                timestamp: base_ts + i as i64,
                gpu_util: i,
                vram_used_mb: i as u64 * 100,
                compute_processes: vec![NvmlComputeProcess {
                    pid: 1000 + i as u32,
                    used_gpu_memory_mb: i as u64,
                    last_seen_timestamp: base_ts + i as i64,
                }],
            };
            persist_sample_row(&conn, &s).expect("persist");
        }
        let stats = query_window_stats(&conn, 0).expect("stats");

        assert!(
            (stats.gpu_util_avg - 50.5).abs() < 0.001,
            "avg was {}",
            stats.gpu_util_avg
        );
        assert!(
            (stats.gpu_util_p95 - 95.0).abs() < 0.001,
            "p95 was {}",
            stats.gpu_util_p95
        );
        // vram_used average = (100 + 200 + ... + 10_000) / 100 = 5050.
        assert_eq!(stats.vram_used_avg_mb, 5050);
        assert_eq!(stats.compute_processes_count, 100);
        // Each row inserted one unique pid → 100 unique
        // `last_seen_timestamp` values aggregated.
        assert_eq!(stats.last_seen_timestamps.len(), 100);
    }

    #[test]
    fn handles_no_gpu_gracefully() {
        // On a host without an NVIDIA driver, `Nvml::init` returns
        // an error which `try_new_standalone` propagates as
        // `NvmlProfileError::Nvml`. On a host *with* a driver
        // (the dev RTX 5080 machine), init succeeds and we exit
        // the test path before asserting anything driver-specific
        // — the contract here is "no panic on either branch".
        let tmpdir = std::env::temp_dir();
        let db_path = tmpdir.join(format!("nvml_profile_test_{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&db_path);
        match NvmlProfile::try_new_standalone(&db_path) {
            Ok(profile) => {
                // Driver present — at minimum the schema must
                // have been applied so a window query does not
                // panic.
                let stats = profile.stats_for_window(Duration::from_secs(60));
                assert!(
                    stats.is_ok(),
                    "stats_for_window must not error on a fresh db"
                );
            }
            Err(NvmlProfileError::Nvml(_)) => {
                // CI runner without NVIDIA driver — exactly the
                // path Phase D promises to surface gracefully.
            }
            Err(other) => panic!(
                "try_new_standalone should only fail with NvmlProfileError::Nvml on no-GPU hosts, got {other:?}"
            ),
        }
        let _ = std::fs::remove_file(&db_path);
    }
}
