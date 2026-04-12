// SPDX-License-Identifier: AGPL-3.0-or-later
//! No-op GPU backend — always compiles, always works, always
//! reports zero devices.
//!
//! This is the CPU-only fallback that the factory picks when
//! NVML (and, eventually, ROCm and Metal) can't initialize. It
//! satisfies the [`GpuMonitor`] trait so the engine loop never
//! has to branch on "GPU present vs absent" beyond checking the
//! number of probed devices.

use super::{GpuError, GpuInfo, GpuMonitor, GpuStats};

/// A [`GpuMonitor`] implementation that reports no devices.
///
/// Used as the fallback when no vendor-specific backend is
/// available. Can also be instantiated directly in tests that
/// need a deterministic "no GPU" state.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopBackend;

impl NoopBackend {
    /// Construct a fresh `NoopBackend`. Const so it can be used
    /// in a `static` if ever needed.
    pub const fn new() -> Self {
        Self
    }
}

impl GpuMonitor for NoopBackend {
    fn backend_name(&self) -> &'static str {
        "noop"
    }

    fn probe(&self) -> Result<Vec<GpuInfo>, GpuError> {
        Ok(Vec::new())
    }

    fn snapshot(&self, index: u32) -> Result<GpuStats, GpuError> {
        // The NoopBackend reports zero devices, so every
        // snapshot call is out of bounds by construction.
        // Returning NoSuchDevice is the contract the engine
        // relies on to fall through to CPU mode.
        Err(GpuError::NoSuchDevice(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_returns_empty() {
        let backend = NoopBackend::new();
        assert!(backend.probe().unwrap().is_empty());
    }

    #[test]
    fn snapshot_always_errors() {
        let backend = NoopBackend::new();
        for idx in [0u32, 1, 42] {
            match backend.snapshot(idx) {
                Err(GpuError::NoSuchDevice(got)) => assert_eq!(got, idx),
                other => panic!("expected NoSuchDevice({idx}), got {other:?}"),
            }
        }
    }

    #[test]
    fn backend_name_is_noop() {
        assert_eq!(NoopBackend::new().backend_name(), "noop");
    }

    #[test]
    fn noop_is_trait_object_compatible() {
        // Compile-time assertion that NoopBackend can be stored
        // as `Box<dyn GpuMonitor>` — the factory relies on this.
        let _: Box<dyn GpuMonitor> = Box::new(NoopBackend::new());
    }
}
