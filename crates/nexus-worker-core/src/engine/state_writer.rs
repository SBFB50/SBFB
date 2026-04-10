//! Sprint 5 Phase A — worker state snapshot writer.
//!
//! The Sprint 5 shell needs a way to show a live view of the
//! worker running on the user's machine. Reaching into the
//! worker via HTTP would mean adding a new network server to
//! every binary; instead, the worker writes a small JSON
//! snapshot to a well-known path every few seconds, and the
//! Python coordinator's `/worker-state` endpoint reads it and
//! proxies it to the shell. See
//! [`crate::paths::worker_state_file`] for the on-disk location
//! and `.planning/sprint5_plan.md` §2.3 for the schema freeze.
//!
//! ## Why a file and not an HTTP endpoint
//!
//! Sprint 5 decision D3 option (c): the worker stays CLI-only,
//! the coordinator is the single point of HTTP integration, and
//! the on-disk file is the shared contract. See the plan for
//! the trade-offs against axum (D3 option a) and CLI exec (D3
//! option b).
//!
//! ## Schema versioning
//!
//! [`SCHEMA_VERSION`] is literal in every snapshot and MUST be
//! bumped on any breaking change (field rename, field removal,
//! type change). Additive changes — new optional fields — may
//! stay on the same version, but they must remain optional in
//! the Python `WorkerStateV1` validator so an older worker is
//! still readable by a newer coordinator.
//!
//! ## Atomicity
//!
//! [`serialize_to`] writes through a temporary sibling file and
//! renames it over the destination. On POSIX this is atomic;
//! on Windows [`std::fs::rename`] is atomic for same-directory
//! moves on NTFS as well. Readers that happen to poll during a
//! write will see either the previous full snapshot or the new
//! one — never a truncated file.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::allowlist::Allowlist;
use crate::gpu::{GpuInfo, GpuStats};

/// On-disk schema version. Bumped on breaking changes only.
pub const SCHEMA_VERSION: u32 = 1;

// =================================================================
// Snapshot types — these mirror the plan §2.3 schema exactly.
// =================================================================

/// A complete `state.json` payload.
///
/// Serde renames fields to the wire format the plan freezes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStateSnapshot {
    pub schema_version: u32,
    pub node_id: String,
    pub worker_version: String,
    pub uptime_secs: u64,
    /// RFC 3339 UTC string, e.g. `2026-04-10T14:00:00Z`.
    pub started_at: String,
    /// RFC 3339 UTC string refreshed on every flush.
    pub last_updated_at: String,
    /// `None` when no GPU is visible (Noop backend, CPU-only
    /// CI runners, containers without `--gpus all`).
    pub gpu: Option<GpuSnapshot>,
    pub projects_served: Vec<ProjectServed>,
    /// `None` until the worker has actually completed at least
    /// one task since boot.
    pub last_task: Option<LastTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSnapshot {
    pub name: String,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub utilization_pct: u8,
    pub temperature_c: u32,
    pub power_draw_w: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServed {
    pub project_name: String,
    pub doc_id: String,
    /// Always `0` in v1 — the coordinator is authoritative on
    /// kudos, and the worker does not subscribe to the kudos
    /// ledger in Sprint 5. Sprint 6+ can wire this once the
    /// worker listens to `/kudos` or reads the hash chain.
    pub kudos_total: u64,
    pub tasks_completed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTask {
    pub task_id: String,
    pub project_name: String,
    /// Prompt truncated to a safe UTF-8 preview length.
    pub prompt_preview: String,
    /// Free-form status string: `"completed"`, `"in_progress"`,
    /// `"failed"`. Kept as String not enum so future engine
    /// statuses don't require a schema bump.
    pub status: String,
    pub completed_at: String,
}

// =================================================================
// Inputs the builder needs
// =================================================================

/// Dynamic snapshot inputs the engine passes to the builder on
/// each flush tick.
///
/// Kept as a plain struct (rather than a trait on the Engine) so
/// the unit tests can build snapshots from arbitrary fixtures
/// without spinning up a full Engine + iroh Node.
pub struct SnapshotInputs<'a> {
    pub node_id: String,
    pub worker_version: &'static str,
    /// Wall-clock time at which the engine booted. Used to
    /// derive `started_at` and `uptime_secs`.
    pub boot_time: SystemTime,
    /// Optional per-device static info. Only the first device
    /// is reported in v1 — most workers run one GPU and the
    /// shell does not yet render per-device breakdowns.
    pub gpu_info: Option<&'a GpuInfo>,
    /// Live GPU stats for the first device, paired with the
    /// static info above. Both must be present or both absent.
    pub gpu_stats: Option<&'a GpuStats>,
    /// Allowlist handle the builder queries for enrolled
    /// projects. Read-only; the caller keeps ownership.
    pub allowlist: &'a Allowlist,
    /// Last completed task reported by the engine, if any.
    pub last_task: Option<LastTask>,
}

impl WorkerStateSnapshot {
    /// Build a snapshot from dynamic inputs.
    ///
    /// Errors are folded into the `projects_served` list being
    /// empty and the gpu field being `None` — the snapshot is a
    /// best-effort observation and must never panic or abort the
    /// flush tick.
    pub fn from_inputs(inputs: SnapshotInputs<'_>) -> Self {
        let now = SystemTime::now();
        let uptime_secs = now
            .duration_since(inputs.boot_time)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let started_at = iso_utc(inputs.boot_time);
        let last_updated_at = iso_utc(now);

        let gpu = match (inputs.gpu_info, inputs.gpu_stats) {
            (Some(info), Some(stats)) => Some(GpuSnapshot {
                name: info.name.clone(),
                memory_total_mb: stats.vram_total_bytes / (1024 * 1024),
                memory_used_mb: stats.vram_used_bytes / (1024 * 1024),
                utilization_pct: stats.gpu_utilization_percent,
                temperature_c: stats.temperature_celsius,
                power_draw_w: stats.power_draw_watts,
            }),
            _ => None,
        };

        let projects_served = match inputs.allowlist.list_enabled() {
            Ok(list) => list
                .into_iter()
                .map(|p| ProjectServed {
                    project_name: p.name,
                    doc_id: p.id,
                    kudos_total: 0,
                    tasks_completed: p.tasks_completed,
                })
                .collect(),
            Err(e) => {
                debug!(error = %e, "allowlist.list_enabled failed in snapshot builder");
                Vec::new()
            }
        };

        Self {
            schema_version: SCHEMA_VERSION,
            node_id: inputs.node_id,
            worker_version: inputs.worker_version.to_string(),
            uptime_secs,
            started_at,
            last_updated_at,
            gpu,
            projects_served,
            last_task: inputs.last_task,
        }
    }
}

// =================================================================
// Atomic writer
// =================================================================

/// Errors from [`serialize_to`]. All variants are caller-handled
/// (logged and ignored) so a transient filesystem problem never
/// bounces the engine through the Error state.
#[derive(Debug, thiserror::Error)]
pub enum StateWriterError {
    #[error("failed to serialize worker state to JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to create parent directory {path}: {source}")]
    MkDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write temporary state file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to rename {tmp} to {dest}: {source}")]
    Rename {
        tmp: PathBuf,
        dest: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Write a snapshot to `dest` atomically.
///
/// The function:
/// 1. Serializes `snapshot` into a `Vec<u8>` (pretty JSON so
///    the file is human-readable when debugging).
/// 2. Ensures every parent directory exists.
/// 3. Writes the bytes to `dest.with_extension("tmp")` (fsynced).
/// 4. Renames the temp file over `dest`.
///
/// If step 3 fails mid-write the temporary file is left behind
/// on disk but the original `dest` is intact. The next flush
/// tick will overwrite the temp file.
pub fn serialize_to(snapshot: &WorkerStateSnapshot, dest: &Path) -> Result<(), StateWriterError> {
    let body = serde_json::to_vec_pretty(snapshot)?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| StateWriterError::MkDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Write to a sibling `.tmp` file in the same directory so
    // the `rename` is a within-filesystem atomic move.
    let tmp = dest.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| StateWriterError::Write {
            path: tmp.clone(),
            source: e,
        })?;
        f.write_all(&body).map_err(|e| StateWriterError::Write {
            path: tmp.clone(),
            source: e,
        })?;
        f.sync_all().map_err(|e| StateWriterError::Write {
            path: tmp.clone(),
            source: e,
        })?;
    }

    fs::rename(&tmp, dest).map_err(|e| StateWriterError::Rename {
        tmp: tmp.clone(),
        dest: dest.to_path_buf(),
        source: e,
    })?;

    debug!(path = %dest.display(), "worker state snapshot flushed");
    Ok(())
}

/// Convenience wrapper: build + write in one call, swallowing
/// errors with a warning so the caller (the engine tick loop)
/// does not have to branch.
pub fn flush(inputs: SnapshotInputs<'_>, dest: &Path) {
    let snapshot = WorkerStateSnapshot::from_inputs(inputs);
    if let Err(e) = serialize_to(&snapshot, dest) {
        warn!(error = %e, "worker state flush failed — will retry on next tick");
    }
}

// =================================================================
// Helpers
// =================================================================

/// Format a `SystemTime` as an RFC 3339 UTC string. Falls back
/// to `"1970-01-01T00:00:00Z"` on a clock that precedes the
/// Unix epoch (impossible in practice but the formatter has to
/// be total).
fn iso_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()
        .and_then(|dt| dt.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    fn mk_allowlist() -> Allowlist {
        Allowlist::open_in_memory().expect("in-memory allowlist")
    }

    fn mk_inputs(al: &Allowlist) -> SnapshotInputs<'_> {
        SnapshotInputs {
            node_id: "deadbeef".repeat(8),
            worker_version: crate::VERSION,
            boot_time: SystemTime::now() - Duration::from_secs(42),
            gpu_info: None,
            gpu_stats: None,
            allowlist: al,
            last_task: None,
        }
    }

    #[test]
    fn state_writer_schema_version_is_1() {
        assert_eq!(SCHEMA_VERSION, 1, "bumping this is a breaking change");
    }

    #[test]
    fn snapshot_from_inputs_has_expected_shape() {
        let al = mk_allowlist();
        let snap = WorkerStateSnapshot::from_inputs(mk_inputs(&al));

        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.node_id.len(), 64);
        assert_eq!(snap.worker_version, crate::VERSION);
        assert!(snap.uptime_secs >= 42, "uptime should reflect boot_time");
        assert!(snap.started_at.ends_with('Z'));
        assert!(snap.last_updated_at.ends_with('Z'));
        assert!(snap.gpu.is_none());
        assert!(snap.projects_served.is_empty());
        assert!(snap.last_task.is_none());
    }

    #[test]
    fn snapshot_includes_null_gpu_when_stats_absent() {
        // Mirrors the NoopBackend / CPU-only path: the plan §2.3
        // explicitly says `gpu` is None on machines without NVML.
        let al = mk_allowlist();
        let snap = WorkerStateSnapshot::from_inputs(mk_inputs(&al));
        assert!(snap.gpu.is_none());

        // And the serialized form shows `"gpu": null`, not a
        // missing field — the shell relies on the key existing.
        let json = serde_json::to_value(&snap).unwrap();
        assert_eq!(json["gpu"], serde_json::Value::Null);
    }

    #[test]
    fn snapshot_lists_enabled_projects_from_allowlist() {
        let al = mk_allowlist();
        al.enroll(crate::allowlist::NewProject {
            id: "doc-abc".into(),
            name: "Alpha".into(),
            enabled: true,
            budget_joules: 0,
            tasks_doc_ticket: None,
        })
        .unwrap();
        al.enroll(crate::allowlist::NewProject {
            id: "doc-xyz".into(),
            name: "Xray".into(),
            enabled: true,
            budget_joules: 0,
            tasks_doc_ticket: None,
        })
        .unwrap();
        al.record_task("doc-abc", 0).unwrap();
        al.record_task("doc-abc", 0).unwrap();

        let snap = WorkerStateSnapshot::from_inputs(mk_inputs(&al));
        assert_eq!(snap.projects_served.len(), 2);

        let alpha = snap
            .projects_served
            .iter()
            .find(|p| p.project_name == "Alpha")
            .expect("Alpha listed");
        assert_eq!(alpha.doc_id, "doc-abc");
        assert_eq!(alpha.tasks_completed, 2);
        assert_eq!(alpha.kudos_total, 0); // always 0 in v1
    }

    #[test]
    fn serialize_to_writes_atomically_and_content_parses_back() {
        let al = mk_allowlist();
        let snap = WorkerStateSnapshot::from_inputs(mk_inputs(&al));

        let dir = tempdir().unwrap();
        let dest = dir.path().join("worker").join("state.json");

        serialize_to(&snap, &dest).expect("serialize_to succeeds");

        assert!(dest.exists(), "destination file must exist after flush");
        let body = fs::read_to_string(&dest).unwrap();
        let back: WorkerStateSnapshot = serde_json::from_str(&body).unwrap();
        assert_eq!(back.schema_version, snap.schema_version);
        assert_eq!(back.node_id, snap.node_id);
        assert_eq!(back.projects_served.len(), snap.projects_served.len());

        // The `.tmp` sibling must be gone after a successful
        // rename; any leftover indicates a non-atomic path.
        assert!(!dest.with_extension("json.tmp").exists());
    }

    #[test]
    fn serialize_to_creates_missing_parent_dirs() {
        let al = mk_allowlist();
        let snap = WorkerStateSnapshot::from_inputs(mk_inputs(&al));

        let dir = tempdir().unwrap();
        // Nested two levels deep, neither exists yet.
        let dest = dir.path().join("a").join("b").join("state.json");
        assert!(!dest.parent().unwrap().exists());

        serialize_to(&snap, &dest).expect("parents are created on demand");

        assert!(dest.exists());
    }

    #[test]
    fn serialize_to_overwrite_is_atomic() {
        // Writing over an existing file must leave the file
        // fully valid at every point — there's no "short read"
        // window where a reader could see a half-written body.
        let al = mk_allowlist();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("state.json");

        // First flush: a minimal snapshot.
        let snap_v1 = WorkerStateSnapshot::from_inputs(mk_inputs(&al));
        serialize_to(&snap_v1, &dest).unwrap();
        let v1_size = fs::metadata(&dest).unwrap().len();
        assert!(v1_size > 0);

        // Second flush: simulate a larger snapshot by adding
        // several projects, then re-write. Readers that poll
        // between the two flushes see either the old or new
        // full JSON.
        for i in 0..5 {
            al.enroll(crate::allowlist::NewProject {
                id: format!("doc-{i}"),
                name: format!("P{i}"),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: None,
            })
            .unwrap();
        }
        let snap_v2 = WorkerStateSnapshot::from_inputs(mk_inputs(&al));
        serialize_to(&snap_v2, &dest).unwrap();

        let back: WorkerStateSnapshot =
            serde_json::from_str(&fs::read_to_string(&dest).unwrap()).unwrap();
        assert_eq!(back.projects_served.len(), 5);
    }

    #[test]
    fn flush_swallows_errors_and_does_not_panic() {
        // Pointing the writer at a path whose parent cannot be
        // created (on POSIX: `/dev/null/state.json`) exercises
        // the warning-only failure path. On Windows this would
        // need a different unreachable path; for determinism we
        // use a path we know the filesystem will refuse and
        // assert the call simply returns.
        let al = mk_allowlist();
        let inputs = mk_inputs(&al);

        // Use a path under a file (not a dir) — create_dir_all
        // will refuse because the parent is already a file.
        let dir = tempdir().unwrap();
        let not_a_dir = dir.path().join("file");
        fs::write(&not_a_dir, b"marker").unwrap();
        let dest = not_a_dir.join("state.json");

        flush(inputs, &dest); // must not panic
        assert!(!dest.exists());
    }

    #[test]
    fn iso_utc_formats_rfc3339_with_trailing_z() {
        let s = iso_utc(UNIX_EPOCH + Duration::from_secs(1_712_754_000));
        assert!(s.ends_with('Z'), "got {s}");
        assert!(s.starts_with("20"), "got {s}");
        assert_eq!(s.len(), "2024-04-10T14:20:00Z".len(), "got {s}");
    }
}
