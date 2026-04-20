// SPDX-License-Identifier: AGPL-3.0-or-later
//! NVIDIA NVML backend for [`GpuMonitor`].
//!
//! Uses the `nvml-wrapper` crate, which dynamically loads
//! `libnvidia-ml.so` / `nvml.dll` at runtime via `libloading`.
//! This means the crate compiles on every target (including
//! macOS and headless Linux CI runners), and any "no NVIDIA
//! hardware" failure is caught at [`NvmlBackend::try_new`] time
//! rather than at link time.
//!
//! The factory in [`crate::gpu::create_monitor`] wraps
//! `try_new`'s error path and falls back to
//! [`crate::gpu::NoopBackend`], so the engine never needs to
//! handle NVML errors directly — it just reads the list of
//! devices the factory-supplied backend reports.

use std::sync::Arc;

use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;

use super::{GpuError, GpuInfo, GpuMonitor, GpuStats};

/// [`GpuMonitor`] implementation backed by NVIDIA NVML.
///
/// The [`Nvml`] handle is wrapped in an `Arc` because
/// `nvml-wrapper::Device` borrows its parent `Nvml` by reference
/// — keeping the `Nvml` alive by refcount lets the backend be
/// `Send + Sync` and cloned freely.
#[derive(Clone)]
pub struct NvmlBackend {
    nvml: Arc<Nvml>,
}

impl std::fmt::Debug for NvmlBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Nvml does not implement Debug; surface a stable label
        // so logs and `{:?}` formatting don't break.
        f.debug_struct("NvmlBackend").finish()
    }
}

impl NvmlBackend {
    /// Attempt to initialize NVML and construct the backend.
    ///
    /// Fails with [`GpuError::Unavailable`] when:
    ///
    /// - `libnvidia-ml` is not installed on the host (macOS, or
    ///   Linux machines without an NVIDIA driver)
    /// - the driver is present but the current user lacks
    ///   permission to talk to it (some container runtimes)
    /// - `Nvml::init` raises any other transient error
    pub fn try_new() -> Result<Self, GpuError> {
        let nvml = Nvml::init().map_err(map_nvml_init)?;
        Ok(Self {
            nvml: Arc::new(nvml),
        })
    }

    /// Expose the wrapped [`Nvml`] handle for advanced callers
    /// that need features this wrapper does not cover yet (e.g.
    /// per-process memory usage).
    pub fn inner(&self) -> &Nvml {
        &self.nvml
    }

    /// Clone the internal `Arc<Nvml>` so other GPU subsystems
    /// (currently [`crate::gpu::profile::NvmlProfile`]) can avoid
    /// a second `Nvml::init` call. Cheap — just bumps the
    /// refcount, never re-enters NVML.
    pub(super) fn shared_handle(&self) -> Arc<Nvml> {
        Arc::clone(&self.nvml)
    }
}

impl GpuMonitor for NvmlBackend {
    fn backend_name(&self) -> &'static str {
        "nvml"
    }

    fn probe(&self) -> Result<Vec<GpuInfo>, GpuError> {
        let device_count = self.nvml.device_count().map_err(map_nvml_call)?;
        let mut out = Vec::with_capacity(device_count as usize);
        for idx in 0..device_count {
            let device = self.nvml.device_by_index(idx).map_err(map_nvml_call)?;
            let name = device.name().map_err(map_nvml_call)?;
            let uuid = device.uuid().map_err(map_nvml_call)?;
            let mem = device.memory_info().map_err(map_nvml_call)?;
            out.push(GpuInfo {
                index: idx,
                backend: "nvml".to_string(),
                name,
                uuid,
                vram_total_bytes: mem.total,
            });
        }
        Ok(out)
    }

    fn snapshot(&self, index: u32) -> Result<GpuStats, GpuError> {
        let device = self.nvml.device_by_index(index).map_err(|e| match e {
            NvmlError::InvalidArg => GpuError::NoSuchDevice(index),
            other => map_nvml_call(other),
        })?;

        let mem = device.memory_info().map_err(map_nvml_call)?;
        let util = device.utilization_rates().map_err(map_nvml_call)?;
        // Several NVML calls are optional (older GPUs don't
        // report power, some WSL2 setups don't report
        // temperature). Treat missing values as zero instead of
        // propagating the error — the engine cares about
        // relative signal, not absolute correctness here.
        let power_mw = device.power_usage().unwrap_or(0);
        let temp_c = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);

        Ok(GpuStats {
            index,
            vram_free_bytes: mem.free,
            vram_used_bytes: mem.used,
            vram_total_bytes: mem.total,
            // Clamp just in case the driver ever returns a
            // weird value; engine code assumes 0..=100.
            gpu_utilization_percent: util.gpu.min(100) as u8,
            memory_utilization_percent: util.memory.min(100) as u8,
            power_draw_watts: power_mw as f32 / 1000.0,
            temperature_celsius: temp_c,
        })
    }
}

// =================================================================
// Error mapping
// =================================================================

/// Convert an NVML init failure to a GpuError::Unavailable so the
/// factory knows to fall back. Always keeps the original
/// [`NvmlError::Display`] text for debuggability.
fn map_nvml_init(err: NvmlError) -> GpuError {
    GpuError::Unavailable(format!("{err}"))
}

/// Convert a per-call NVML failure to GpuError::BackendError.
/// Unlike `map_nvml_init`, these are transient — the engine
/// should log and retry on the next tick instead of falling back
/// to the noop backend.
fn map_nvml_call(err: NvmlError) -> GpuError {
    GpuError::BackendError(format!("{err}"))
}

// =================================================================
// Tests
// =================================================================
//
// Real-hardware tests are gated on whether NVML is available on
// the runner. CI runners without an NVIDIA driver silently skip
// the hardware assertions (the init errors and we verify the
// error path instead). The dev machine with an RTX 5080 exercises
// the full path including device listing and a single snapshot.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_either_succeeds_or_reports_unavailable() {
        match NvmlBackend::try_new() {
            Ok(backend) => {
                // Hardware path: at least probe should succeed
                // (may still report zero devices in a container).
                let devices = backend.probe().expect("probe after init must succeed");
                // And every device must at least have a name.
                for d in &devices {
                    assert_eq!(d.backend, "nvml");
                    assert!(!d.name.is_empty(), "device name must be non-empty");
                    assert!(d.vram_total_bytes > 0, "vram_total must be > 0");
                }
            }
            Err(GpuError::Unavailable(msg)) => {
                // CI path: ensure the error carries the NVML
                // diagnostic text through for humans.
                assert!(!msg.is_empty(), "Unavailable message must be non-empty");
            }
            Err(other) => {
                panic!(
                    "NvmlBackend::try_new should only return Unavailable on failure, got {other:?}"
                )
            }
        }
    }

    #[test]
    fn snapshot_of_unknown_index_reports_no_such_device_on_hardware() {
        let Ok(backend) = NvmlBackend::try_new() else {
            // No NVML on this host — skip the hardware assertion.
            return;
        };

        // Find a definitely-out-of-range index. device_count
        // returns the number of devices, so `count` itself is
        // one past the last valid index.
        let count = backend.inner().device_count().unwrap();
        match backend.snapshot(count) {
            Err(GpuError::NoSuchDevice(got)) => assert_eq!(got, count),
            other => panic!("expected NoSuchDevice({count}), got {other:?}"),
        }
    }

    #[test]
    fn snapshot_of_device_zero_returns_live_stats_on_hardware() {
        let Ok(backend) = NvmlBackend::try_new() else {
            return;
        };
        let devices = backend.probe().unwrap();
        if devices.is_empty() {
            // Driver present but no devices visible (container,
            // `--gpus none`). Not an error, nothing to test.
            return;
        }

        let stats = backend.snapshot(0).expect("snapshot of device 0");
        assert_eq!(stats.index, 0);
        assert!(stats.vram_total_bytes > 0);
        // nvml-wrapper 0.11.0 switched the underlying NVML call
        // to `nvmlDeviceGetMemoryInfo` v2 (CHANGELOG entry "to
        // be consistent with nvidia-smi"). v2 reports
        // `total = free + used + reserved` where `reserved` is
        // driver/system overhead — so the strict equality
        // `total == free + used` from the v1 era no longer holds.
        // The relaxed invariant remains a useful sanity check:
        // memory accounting must never *over*-report relative to
        // the device total.
        assert!(
            stats.vram_free_bytes + stats.vram_used_bytes <= stats.vram_total_bytes,
            "memory accounting invariant: free + used must not exceed total \
             (v2 semantics: total = free + used + reserved-driver-overhead)"
        );
        assert!(stats.gpu_utilization_percent <= 100);
        assert!(stats.memory_utilization_percent <= 100);
    }

    #[test]
    fn nvml_backend_is_trait_object_compatible() {
        // Compile-time assertion: the factory returns
        // `Box<dyn GpuMonitor>`, so NvmlBackend must be
        // dyn-compatible.
        fn _assert_dyn(_: Box<dyn GpuMonitor>) {}
        if let Ok(backend) = NvmlBackend::try_new() {
            _assert_dyn(Box::new(backend));
        }
    }
}
