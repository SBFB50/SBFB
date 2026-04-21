// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dispatch hook trait — preparatory stub for S29 TraceProvider.
//!
//! Python-first implementation lives in
//! `packages/nexus-coordinator/src/nexus_coordinator/hooks.py`.
//! This Rust trait exists so S29 can implement a Rust-side
//! `DispatchHook` without breaking the Python API.

/// Lifecycle event fired by the dispatch pipeline.
///
/// Implementors observe task lifecycle without veto or retry
/// (fire-and-forget). Must not block the calling task.
pub trait DispatchHook: Send + Sync {
    /// Hook identifier for logging and diagnostics.
    fn name(&self) -> &str;

    /// Called on each lifecycle event.
    fn on_event(&self, event: &str, task_id: &str);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook;

    impl DispatchHook for TestHook {
        fn name(&self) -> &str {
            "test_hook"
        }

        fn on_event(&self, _event: &str, _task_id: &str) {}
    }

    #[test]
    fn dispatch_hook_trait_object_safe() {
        let hook: Box<dyn DispatchHook> = Box::new(TestHook);
        assert_eq!(hook.name(), "test_hook");
        hook.on_event("on_task_dispatched", "t-123");
    }
}
