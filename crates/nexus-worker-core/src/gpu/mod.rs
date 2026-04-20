// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-vendor GPU monitoring trait for the worker engine.
//!
//! The worker needs a small, stable surface for
//! "how much VRAM is left / how hot is the card / how busy is
//! it", independent of the specific vendor. Sprint 3 ships the
//! NVIDIA backend only (NvmlBackend via `nvml-wrapper`) plus the
//! always-available NoopBackend that reports zero devices; the
//! trait is designed so that W1.1-era AMD ROCm and Apple Metal
//! implementations slot in as additional [`GpuMonitor`] impls
//! without touching the engine loop.
//!
//! ## Factory
//!
//! [`create_monitor`] is the runtime entry point. It tries the
//! NvmlBackend first and, if `Nvml::init` fails (no NVIDIA
//! driver, no `libnvidia-ml.so`, permission denied, containerized
//! runner without `--gpus all`, ...), logs a warning and returns
//! the NoopBackend. The caller gets a `Box<dyn GpuMonitor>`
//! and never has to branch on which backend actually loaded.
//!
//! Runtime fallback is correct here rather than a feature flag
//! because `nvml-wrapper` itself loads `libnvidia-ml` via
//! `libloading` at runtime — the crate compiles and links on
//! every platform even without an NVIDIA installation, and any
//! failure is caught by `Nvml::init`. See the Sprint 3 W4
//! decision in the SBFB plan.
//!
//! ## Separation of concerns
//!
//! - [`GpuInfo`] is *static* per device: name, UUID, total VRAM.
//!   The engine reads it once at boot and caches it.
//! - [`GpuStats`] is *dynamic*: free VRAM, utilization, power,
//!   temperature. The engine polls it on its state-machine
//!   tick in W9.

pub mod noop;
pub mod nvml;
pub mod profile;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use noop::NoopBackend;
pub use nvml::NvmlBackend;

// =================================================================
// Errors
// =================================================================

/// Errors that can arise from any [`GpuMonitor`] implementation.
#[derive(Debug, Error)]
pub enum GpuError {
    /// Backend-level initialization failed: driver missing,
    /// library missing, permission denied, containerized runtime
    /// without GPU access, etc. The worker falls back to the
    /// NoopBackend on this error.
    #[error("gpu backend unavailable: {0}")]
    Unavailable(String),

    /// The requested device index is out of range for this
    /// backend. Returned by [`GpuMonitor::snapshot`].
    #[error("no gpu device at index {0}")]
    NoSuchDevice(u32),

    /// A backend-specific call failed (NVML transient error, for
    /// instance). The engine should log and retry on its next
    /// tick rather than crash.
    #[error("gpu backend call failed: {0}")]
    BackendError(String),
}

// =================================================================
// Static device info
// =================================================================

/// Static, never-changing information about a single GPU.
///
/// Produced by [`GpuMonitor::probe`] once at worker boot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Zero-based device index as reported by the backend.
    pub index: u32,
    /// Backend name (e.g. "nvml", "noop") — useful for logs and
    /// the `stats` subcommand so users can tell which path the
    /// worker took.
    pub backend: String,
    /// Human-readable device name, e.g. `NVIDIA GeForce RTX 5080`.
    pub name: String,
    /// Stable per-device UUID, if the backend exposes one. The
    /// Noop backend leaves this empty.
    pub uuid: String,
    /// Total video memory in bytes.
    pub vram_total_bytes: u64,
}

// =================================================================
// Dynamic per-device stats
// =================================================================

/// A point-in-time snapshot of a single GPU's dynamic state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuStats {
    /// Device index the snapshot was taken from.
    pub index: u32,
    /// Free video memory in bytes at snapshot time.
    pub vram_free_bytes: u64,
    /// Used video memory in bytes at snapshot time.
    pub vram_used_bytes: u64,
    /// Total video memory in bytes (duplicated from GpuInfo for
    /// convenience so callers don't need both).
    pub vram_total_bytes: u64,
    /// GPU compute utilization 0..=100. The NVML "gpu" counter
    /// (roughly: percentage of time a kernel was running).
    pub gpu_utilization_percent: u8,
    /// Memory controller utilization 0..=100 — separate from
    /// VRAM *used* (that's a level, this is activity).
    pub memory_utilization_percent: u8,
    /// Current power draw in watts. NVML reports milliwatts
    /// internally; the backend converts before storing.
    pub power_draw_watts: f32,
    /// GPU core temperature in degrees Celsius. 0 when the
    /// backend does not expose it.
    pub temperature_celsius: u32,
}

impl GpuStats {
    /// Fraction of VRAM currently in use, as a value in `[0.0,
    /// 1.0]`. Returns 0.0 when `vram_total_bytes` is zero
    /// (Noop backend case) to avoid division by zero.
    pub fn vram_used_fraction(&self) -> f32 {
        if self.vram_total_bytes == 0 {
            return 0.0;
        }
        self.vram_used_bytes as f32 / self.vram_total_bytes as f32
    }

    /// Amount of VRAM, in bytes, that the worker is still allowed
    /// to consume given a config-supplied `max_vram_fraction`.
    ///
    /// ```text
    /// budget = (total * max_fraction).saturating_sub(used)
    /// ```
    ///
    /// Used by the engine (W9) to decide whether to accept a new
    /// task based on its reported VRAM requirement.
    pub fn vram_budget_remaining_bytes(&self, max_vram_fraction: f32) -> u64 {
        let max = max_vram_fraction.clamp(0.0, 1.0);
        let allowed = (self.vram_total_bytes as f64 * max as f64) as u64;
        allowed.saturating_sub(self.vram_used_bytes)
    }
}

// =================================================================
// Trait
// =================================================================

/// Uniform GPU monitoring API used by the worker engine.
///
/// Every backend is `Send + Sync` so the engine can share it
/// between tokio tasks without wrapping in a mutex. Calls are
/// intentionally *synchronous* — NVML / IOKit / ROCm queries
/// complete in microseconds and blocking-on-them in a poll loop
/// is cheaper than the overhead of tokio spawning.
pub trait GpuMonitor: Send + Sync {
    /// Static name of the backend (`"nvml"`, `"noop"`, ...).
    /// Stamped into every [`GpuInfo`] this backend produces.
    fn backend_name(&self) -> &'static str;

    /// Enumerate every GPU visible to this backend.
    ///
    /// Called once at worker boot. An empty `Vec` is a valid
    /// (non-error) result for the Noop backend or for an NVIDIA
    /// machine with no GPUs in a container.
    fn probe(&self) -> Result<Vec<GpuInfo>, GpuError>;

    /// Return a dynamic snapshot of the device at `index`.
    ///
    /// The engine polls this on its state-machine tick; backends
    /// should make it cheap (a few microseconds) and lock-free.
    fn snapshot(&self, index: u32) -> Result<GpuStats, GpuError>;
}

// =================================================================
// Factory
// =================================================================

/// Create the best available GPU monitor for this machine.
///
/// Preference order:
///
/// 1. [`NvmlBackend`] — works on any host with a recent NVIDIA
///    driver installed (Linux / Windows). Failure modes (no
///    driver, no devices, permission denied) fall through.
/// 2. [`NoopBackend`] — always available. Reports an empty device
///    list so the engine gracefully degrades to CPU-only mode.
pub fn create_monitor() -> Box<dyn GpuMonitor> {
    match NvmlBackend::try_new() {
        Ok(backend) => {
            tracing::info!("gpu monitor: using NVML backend");
            Box::new(backend)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "gpu monitor: NVML unavailable, falling back to NoopBackend (CPU-only mode)"
            );
            Box::new(NoopBackend::new())
        }
    }
}

// =================================================================
// Tests — trait-level only (backend-specific tests live alongside
// the backends themselves).
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_used_fraction_handles_zero_total() {
        let stats = GpuStats {
            index: 0,
            vram_free_bytes: 0,
            vram_used_bytes: 0,
            vram_total_bytes: 0,
            gpu_utilization_percent: 0,
            memory_utilization_percent: 0,
            power_draw_watts: 0.0,
            temperature_celsius: 0,
        };
        assert_eq!(stats.vram_used_fraction(), 0.0);
    }

    #[test]
    fn vram_used_fraction_computes_ratio() {
        let stats = GpuStats {
            index: 0,
            vram_free_bytes: 2 * 1024 * 1024 * 1024,
            vram_used_bytes: 6 * 1024 * 1024 * 1024,
            vram_total_bytes: 8 * 1024 * 1024 * 1024,
            gpu_utilization_percent: 50,
            memory_utilization_percent: 30,
            power_draw_watts: 250.0,
            temperature_celsius: 65,
        };
        assert!((stats.vram_used_fraction() - 0.75).abs() < 0.001);
    }

    #[test]
    fn vram_budget_remaining_respects_max_fraction() {
        let total = 16 * 1024 * 1024 * 1024u64;
        let used = 4 * 1024 * 1024 * 1024u64;
        let stats = GpuStats {
            index: 0,
            vram_free_bytes: total - used,
            vram_used_bytes: used,
            vram_total_bytes: total,
            gpu_utilization_percent: 0,
            memory_utilization_percent: 0,
            power_draw_watts: 0.0,
            temperature_celsius: 0,
        };

        // 90% of 16 GiB = ~14.4 GiB allowed; minus 4 GiB used
        // leaves ~10.4 GiB budget. Compute the expected value
        // the same way the backend does (f32 fraction ↦ f64
        // multiply ↦ u64 truncate) so f32→f64 precision loss
        // matches exactly.
        let max_frac: f32 = 0.9;
        let budget = stats.vram_budget_remaining_bytes(max_frac);
        let expected =
            ((stats.vram_total_bytes as f64 * max_frac as f64) as u64).saturating_sub(used);
        assert_eq!(budget, expected);
        // Sanity: roughly 10.4 GiB
        assert!(budget > 10 * 1024 * 1024 * 1024);
        assert!(budget < 11 * 1024 * 1024 * 1024);
    }

    #[test]
    fn vram_budget_clamps_fraction_above_one() {
        let total = 8 * 1024 * 1024 * 1024u64;
        let stats = GpuStats {
            index: 0,
            vram_free_bytes: total,
            vram_used_bytes: 0,
            vram_total_bytes: total,
            gpu_utilization_percent: 0,
            memory_utilization_percent: 0,
            power_draw_watts: 0.0,
            temperature_celsius: 0,
        };

        // A misconfigured 2.0 fraction should clamp to 1.0, not
        // give a budget > total.
        assert_eq!(stats.vram_budget_remaining_bytes(2.0), total);
    }

    #[test]
    fn factory_always_returns_a_backend() {
        // On a CI runner without an NVIDIA driver this returns
        // the NoopBackend. On a dev machine with NVML installed
        // it returns the NvmlBackend. Either way `probe()`
        // succeeds (zero or more devices).
        let monitor = create_monitor();
        let _ = monitor
            .probe()
            .expect("factory-built monitor must probe successfully");
    }

    #[test]
    fn info_and_stats_are_serde_round_trippable() {
        let info = GpuInfo {
            index: 0,
            backend: "nvml".into(),
            name: "NVIDIA GeForce RTX 5080".into(),
            uuid: "GPU-deadbeef".into(),
            vram_total_bytes: 16 * 1024 * 1024 * 1024,
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: GpuInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }
}
