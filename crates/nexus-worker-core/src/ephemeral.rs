// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ephemeral worker lifecycle: restart-based rotation + VRAM wipe.
//!
//! Defense-in-depth against model weight extraction by a malicious
//! task N+1 reading residual VRAM from task N's inference. The
//! worker process self-terminates after `max_tasks` completions
//! (the supervisor restarts it), and between each task the engine
//! zeroes freed VRAM via the CUDA driver API (`cuMemsetD8`
//! equivalent through cudarc safe wrappers).

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[cfg(feature = "gpu-ephemeral")]
use tracing::warn;

// =================================================================
// Config
// =================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemeralConfig {
    #[serde(default = "default_max_tasks")]
    pub max_tasks: u32,
    #[serde(default = "default_vram_wipe")]
    pub vram_wipe: bool,
}

fn default_max_tasks() -> u32 {
    50
}

fn default_vram_wipe() -> bool {
    true
}

impl Default for EphemeralConfig {
    fn default() -> Self {
        Self {
            max_tasks: default_max_tasks(),
            vram_wipe: default_vram_wipe(),
        }
    }
}

// =================================================================
// Lifecycle state machine
// =================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Ready,
    Running,
    WipePending,
    RestartPending,
    Exiting,
}

pub struct EphemeralLifecycle {
    config: EphemeralConfig,
    state: LifecycleState,
    completed_count: u32,
}

impl EphemeralLifecycle {
    pub fn new(config: EphemeralConfig) -> Self {
        Self {
            config,
            state: LifecycleState::Ready,
            completed_count: 0,
        }
    }

    pub fn state(&self) -> LifecycleState {
        self.state
    }

    pub fn completed_count(&self) -> u32 {
        self.completed_count
    }

    pub fn config(&self) -> &EphemeralConfig {
        &self.config
    }

    pub fn start_task(&mut self) {
        if self.state == LifecycleState::Ready {
            self.state = LifecycleState::Running;
        }
    }

    pub fn complete_task(&mut self) {
        if self.state == LifecycleState::Running {
            self.completed_count += 1;
            if self.config.vram_wipe {
                self.state = LifecycleState::WipePending;
            } else if self.should_restart(self.completed_count) {
                self.state = LifecycleState::RestartPending;
            } else {
                self.state = LifecycleState::Ready;
            }
        }
    }

    pub fn wipe_done(&mut self) {
        if self.state == LifecycleState::WipePending {
            if self.should_restart(self.completed_count) {
                self.state = LifecycleState::RestartPending;
            } else {
                self.state = LifecycleState::Ready;
            }
        }
    }

    pub fn should_restart(&self, completed: u32) -> bool {
        completed >= self.config.max_tasks
    }

    pub fn request_exit(&mut self) {
        self.state = LifecycleState::Exiting;
        info!(
            completed = self.completed_count,
            max_tasks = self.config.max_tasks,
            "ephemeral lifecycle requesting process exit"
        );
    }
}

// =================================================================
// VRAM wipe
// =================================================================

#[cfg(feature = "gpu-ephemeral")]
pub async fn wipe_vram() -> anyhow::Result<()> {
    tokio::task::spawn_blocking(wipe_vram_sync).await?
}

#[cfg(feature = "gpu-ephemeral")]
fn wipe_vram_sync() -> anyhow::Result<()> {
    use cudarc::driver::{result as cuda_result, CudaDevice};

    let device_count = match CudaDevice::count() {
        Ok(n) => n,
        Err(e) => {
            warn!(error = ?e, "cudarc: cannot query device count; skipping VRAM wipe");
            return Ok(());
        }
    };

    if device_count == 0 {
        debug!("cudarc: no CUDA devices visible; VRAM wipe is a no-op");
        return Ok(());
    }

    for ordinal in 0..device_count {
        let dev = match CudaDevice::new(ordinal as usize) {
            Ok(d) => d,
            Err(e) => {
                warn!(ordinal, error = ?e, "cudarc: cannot open device; skipping");
                continue;
            }
        };

        // mem_get_info operates on the current context (set by
        // CudaDevice::new which pushes the primary context).
        let (free, _total) = match cuda_result::mem_get_info() {
            Ok(info) => info,
            Err(e) => {
                warn!(ordinal, error = ?e, "cudarc: mem_get_info failed; skipping device");
                continue;
            }
        };

        // Allocate 90% of free VRAM as zeroed memory, then drop it.
        // This overwrites residual inference data (KV cache,
        // activations) left by the previous task. We leave a 10%
        // margin so the driver's internal bookkeeping doesn't OOM.
        let wipe_size = (free as f64 * 0.9) as usize;
        if wipe_size == 0 {
            continue;
        }

        match dev.alloc_zeros::<u8>(wipe_size) {
            Ok(_slice) => {
                debug!(ordinal, wipe_bytes = wipe_size, "VRAM wipe completed");
            }
            Err(e) => {
                warn!(
                    ordinal,
                    wipe_bytes = wipe_size,
                    error = ?e,
                    "cudarc: alloc_zeros failed; VRAM wipe incomplete"
                );
            }
        }
    }

    info!(device_count, "ephemeral VRAM wipe pass completed");
    Ok(())
}

#[cfg(not(feature = "gpu-ephemeral"))]
pub async fn wipe_vram() -> anyhow::Result<()> {
    debug!("VRAM wipe: gpu-ephemeral feature disabled; no-op");
    Ok(())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_transitions() {
        let cfg = EphemeralConfig {
            max_tasks: 50,
            vram_wipe: true,
        };
        let mut lc = EphemeralLifecycle::new(cfg);
        assert_eq!(lc.state(), LifecycleState::Ready);

        lc.start_task();
        assert_eq!(lc.state(), LifecycleState::Running);

        lc.complete_task();
        assert_eq!(lc.state(), LifecycleState::WipePending);

        lc.wipe_done();
        assert_eq!(lc.state(), LifecycleState::Ready);
    }

    #[test]
    fn test_should_restart_at_max() {
        let cfg = EphemeralConfig {
            max_tasks: 3,
            vram_wipe: false,
        };
        let mut lc = EphemeralLifecycle::new(cfg);

        for _ in 0..3 {
            lc.start_task();
            lc.complete_task();
            if lc.state() == LifecycleState::RestartPending {
                break;
            }
        }

        assert_eq!(lc.completed_count(), 3);
        assert_eq!(lc.state(), LifecycleState::RestartPending);
    }

    #[test]
    fn test_should_not_restart_below_max() {
        let cfg = EphemeralConfig {
            max_tasks: 5,
            vram_wipe: false,
        };
        let mut lc = EphemeralLifecycle::new(cfg);

        lc.start_task();
        lc.complete_task();
        assert_eq!(lc.completed_count(), 1);
        assert_eq!(lc.state(), LifecycleState::Ready);
        assert!(!lc.should_restart(1));

        lc.start_task();
        lc.complete_task();
        assert_eq!(lc.completed_count(), 2);
        assert_eq!(lc.state(), LifecycleState::Ready);
        assert!(!lc.should_restart(2));
    }

    #[test]
    fn test_wipe_vram_no_gpu() {
        // Without gpu-ephemeral feature, wipe_vram is a no-op that
        // returns Ok(()). This test exercises the non-feature path.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(wipe_vram());
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_parse_toml() {
        let toml_str = r#"
            max_tasks = 100
            vram_wipe = false
        "#;
        let cfg: EphemeralConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.max_tasks, 100);
        assert!(!cfg.vram_wipe);
    }

    #[test]
    fn test_config_default_values() {
        let cfg = EphemeralConfig::default();
        assert_eq!(cfg.max_tasks, 50);
        assert!(cfg.vram_wipe);
    }

    #[test]
    fn test_restart_signal_sets_exit() {
        let cfg = EphemeralConfig::default();
        let mut lc = EphemeralLifecycle::new(cfg);
        lc.start_task();
        assert_eq!(lc.state(), LifecycleState::Running);

        lc.request_exit();
        assert_eq!(lc.state(), LifecycleState::Exiting);
    }

    #[test]
    fn test_wipe_pending_transitions_to_restart_at_max() {
        let cfg = EphemeralConfig {
            max_tasks: 2,
            vram_wipe: true,
        };
        let mut lc = EphemeralLifecycle::new(cfg);

        // Task 1
        lc.start_task();
        lc.complete_task();
        assert_eq!(lc.state(), LifecycleState::WipePending);
        lc.wipe_done();
        assert_eq!(lc.state(), LifecycleState::Ready);

        // Task 2 — hits max_tasks after wipe
        lc.start_task();
        lc.complete_task();
        assert_eq!(lc.state(), LifecycleState::WipePending);
        lc.wipe_done();
        assert_eq!(lc.state(), LifecycleState::RestartPending);
    }
}
