//! Worker finite state machine.
//!
//! The SBFB worker has a small but strict lifecycle:
//!
//! ```text
//!     Idle
//!       │ Start
//!       ▼
//!   Connecting ──────NeedModel──────► PullingModel
//!       │                                │
//!       │ Connected                      │ ModelReady
//!       ▼                                ▼
//!    Processing ◄───────────────────────────
//!       ▲  │
//!       │  │ Pause               (Any state can take
//!       │  ▼                      Fail → Error  or
//!       │ Paused                   Shutdown → Shutdown)
//!       │  │ Resume
//!       │  ▼
//!       └─Connecting
//! ```
//!
//! `Error(reason)` and `Shutdown` are sinks: the only way out
//! of `Error` is an explicit `Clear` event (manual intervention);
//! the only thing that leaves `Shutdown` is process exit.
//!
//! The state machine is deliberately written as a plain
//! `match` instead of a transition-table DSL so reviewers can
//! eyeball every legal transition on one screen and so adding
//! a new state does not require a third-party crate upgrade.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// =================================================================
// State
// =================================================================

/// Every possible state of the worker engine.
///
/// The enum is `Serialize + Deserialize` so the W10 TUI and W11
/// telemetry emitters can render it directly, and so tests can
/// store expected states as JSON fixtures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerState {
    /// Freshly booted, no network activity yet. This is the
    /// state right after `nexus-worker register`.
    Idle,

    /// Attempting to reach a coordinator through one or more
    /// enrolled projects.
    Connecting,

    /// Pulling a model blob from the Ollama daemon. Carries the
    /// target model name and the last reported percent
    /// completion so the TUI can render a progress bar.
    PullingModel { model: String, progress: u8 },

    /// Connected, model present, serving tasks. Carries the
    /// number of tasks currently in flight so `stats` and the
    /// TUI can render a counter without querying the engine.
    Processing { active_tasks: usize },

    /// User-requested pause. No new task claims; current tasks
    /// already in flight drain to completion.
    Paused,

    /// Unrecoverable runtime error. The engine sits here until
    /// a `Clear` event (manual intervention) brings it back to
    /// Idle.
    Error { reason: String },

    /// Terminal state after a `Shutdown` event. The engine
    /// stops polling and the binary exits.
    Shutdown,
}

impl WorkerState {
    /// Short, stable machine name used in logs, metrics labels,
    /// TUI titles and the `stats` subcommand output. Does NOT
    /// include parameters (model name, task count, etc.) so it
    /// is safe to use as a dimension in Prometheus etc.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Connecting => "connecting",
            Self::PullingModel { .. } => "pulling_model",
            Self::Processing { .. } => "processing",
            Self::Paused => "paused",
            Self::Error { .. } => "error",
            Self::Shutdown => "shutdown",
        }
    }

    /// Returns true for states from which no transition is
    /// possible (with the specific exception of `Error -> Idle`
    /// via `Clear`, which is not considered "terminal" here —
    /// only `Shutdown` is).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Shutdown)
    }

    /// Returns true when the worker is willing to accept new
    /// task claims. Used by the engine loop in W9 to gate the
    /// `get_many_by_prefix("task:")` poll.
    pub fn can_claim_tasks(&self) -> bool {
        matches!(self, Self::Processing { .. })
    }
}

impl fmt::Display for WorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Connecting => write!(f, "connecting"),
            Self::PullingModel { model, progress } => {
                write!(f, "pulling_model({model}, {progress}%)")
            }
            Self::Processing { active_tasks } => {
                write!(f, "processing({active_tasks} active)")
            }
            Self::Paused => write!(f, "paused"),
            Self::Error { reason } => write!(f, "error({reason})"),
            Self::Shutdown => write!(f, "shutdown"),
        }
    }
}

// =================================================================
// Events
// =================================================================

/// Every event that can possibly affect the worker state.
///
/// The engine loop in W9 translates external stimuli (timer
/// ticks, iroh doc events, Ollama results, user commands) into
/// these events and feeds them to the state machine via
/// [`StateMachine::apply`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerEvent {
    /// Boot the engine: Idle → Connecting.
    Start,

    /// Reached a coordinator: Connecting → Processing (all
    /// required models already pulled).
    Connected,

    /// Reached a coordinator but the required model is not yet
    /// present on disk: Connecting → PullingModel.
    NeedModel { model: String },

    /// Progress update during model pull (0..=100 percent).
    /// PullingModel stays in PullingModel.
    ModelProgress { percent: u8 },

    /// Model pull completed: PullingModel → Processing.
    ModelReady,

    /// A task claim was accepted and a worker thread started
    /// executing it: Processing active_tasks += 1.
    TaskStarted,

    /// A running task finished (either success or failure —
    /// verification happens elsewhere): Processing active_tasks
    /// -= 1, saturating at zero.
    TaskCompleted,

    /// User-requested pause: any active state → Paused.
    Pause,

    /// User-requested resume from pause: Paused → Connecting.
    Resume,

    /// Unrecoverable runtime failure: any state → Error(reason).
    Fail { reason: String },

    /// Clear an Error state and return to Idle. No-op if the
    /// current state is not Error.
    Clear,

    /// User- or signal-requested shutdown: any state → Shutdown.
    Shutdown,
}

impl WorkerEvent {
    /// Short machine name for the event, used in error messages
    /// and logs. The `Fail` / `NeedModel` payloads are omitted
    /// so labels are stable across instances.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Connected => "connected",
            Self::NeedModel { .. } => "need_model",
            Self::ModelProgress { .. } => "model_progress",
            Self::ModelReady => "model_ready",
            Self::TaskStarted => "task_started",
            Self::TaskCompleted => "task_completed",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Fail { .. } => "fail",
            Self::Clear => "clear",
            Self::Shutdown => "shutdown",
        }
    }
}

// =================================================================
// Transition error
// =================================================================

/// Error returned from [`StateMachine::apply`] when the caller
/// tried to fire an event that is not legal in the current
/// state. Carries both pieces of context so the engine can log
/// exactly what mis-sequence happened.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid transition: event {event} not allowed from state {state}")]
pub struct TransitionError {
    pub state: String,
    pub event: String,
}

impl TransitionError {
    fn new(state: &WorkerState, event: &WorkerEvent) -> Self {
        Self {
            state: state.label().to_string(),
            event: event.label().to_string(),
        }
    }
}

// =================================================================
// State machine
// =================================================================

/// Owns the current [`WorkerState`] and enforces the legal
/// transition graph.
///
/// Start from [`StateMachine::new`] (which sits in `Idle`),
/// then feed events with [`StateMachine::apply`]. Every legal
/// transition returns `Ok(previous_state)` so the engine loop
/// can tell whether the apply actually changed anything and so
/// logs can show "X → Y on Event".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine {
    state: WorkerState,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    /// Construct a new state machine in [`WorkerState::Idle`].
    pub fn new() -> Self {
        Self {
            state: WorkerState::Idle,
        }
    }

    /// Return a reference to the current state.
    pub fn state(&self) -> &WorkerState {
        &self.state
    }

    /// Apply an event and transition to the new state.
    ///
    /// Returns `Ok(previous_state)` on a legal transition (so
    /// the caller can log "X → Y on Event"), `Err` on an
    /// illegal one. Illegal transitions never mutate the
    /// machine state.
    pub fn apply(&mut self, event: WorkerEvent) -> Result<WorkerState, TransitionError> {
        let next = self.next_state(&event)?;
        Ok(std::mem::replace(&mut self.state, next))
    }

    fn next_state(&self, event: &WorkerEvent) -> Result<WorkerState, TransitionError> {
        use WorkerEvent as E;
        use WorkerState as S;

        // Shutdown and Fail are universally accepted (except
        // from the already-terminal `Shutdown` state where
        // they are no-ops).
        match (&self.state, event) {
            (S::Shutdown, _) => return Err(TransitionError::new(&self.state, event)),

            (_, E::Shutdown) => return Ok(S::Shutdown),
            (_, E::Fail { reason }) => {
                return Ok(S::Error {
                    reason: reason.clone(),
                })
            }
            _ => {}
        }

        let next = match (&self.state, event) {
            // ---- Idle ------------------------------------------
            (S::Idle, E::Start) => S::Connecting,

            // ---- Connecting ------------------------------------
            (S::Connecting, E::Connected) => S::Processing { active_tasks: 0 },
            (S::Connecting, E::NeedModel { model }) => S::PullingModel {
                model: model.clone(),
                progress: 0,
            },
            (S::Connecting, E::Pause) => S::Paused,

            // ---- PullingModel ----------------------------------
            (S::PullingModel { model, .. }, E::ModelProgress { percent }) => S::PullingModel {
                model: model.clone(),
                progress: (*percent).min(100),
            },
            (S::PullingModel { .. }, E::ModelReady) => S::Processing { active_tasks: 0 },
            (S::PullingModel { .. }, E::Pause) => S::Paused,

            // ---- Processing ------------------------------------
            (S::Processing { active_tasks }, E::TaskStarted) => S::Processing {
                active_tasks: active_tasks.saturating_add(1),
            },
            (S::Processing { active_tasks }, E::TaskCompleted) => S::Processing {
                active_tasks: active_tasks.saturating_sub(1),
            },
            (S::Processing { .. }, E::Pause) => S::Paused,

            // ---- Paused ----------------------------------------
            (S::Paused, E::Resume) => S::Connecting,

            // ---- Error -----------------------------------------
            (S::Error { .. }, E::Clear) => S::Idle,

            // ---- everything else is illegal --------------------
            _ => return Err(TransitionError::new(&self.state, event)),
        };

        Ok(next)
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers ----

    fn sm(state: WorkerState) -> StateMachine {
        StateMachine { state }
    }

    fn expect_ok(sm: &mut StateMachine, event: WorkerEvent, expected_next: &WorkerState) {
        sm.apply(event.clone()).unwrap_or_else(|e| {
            panic!("expected transition to {expected_next:?} on {event:?} to succeed, got {e}")
        });
        assert_eq!(
            sm.state(),
            expected_next,
            "after {event:?}, state should be {expected_next:?}"
        );
    }

    fn expect_invalid(sm: &mut StateMachine, event: WorkerEvent) {
        let before = sm.state().clone();
        let err = sm.apply(event.clone()).unwrap_err();
        assert_eq!(
            sm.state(),
            &before,
            "illegal transitions must not mutate state"
        );
        assert_eq!(err.state, before.label());
        assert_eq!(err.event, event.label());
    }

    // ---- state helpers ----

    #[test]
    fn new_starts_in_idle() {
        assert_eq!(StateMachine::new().state(), &WorkerState::Idle);
    }

    #[test]
    fn label_is_stable_across_payloads() {
        assert_eq!(
            WorkerState::PullingModel {
                model: "a".into(),
                progress: 0
            }
            .label(),
            "pulling_model"
        );
        assert_eq!(
            WorkerState::PullingModel {
                model: "b".into(),
                progress: 90
            }
            .label(),
            "pulling_model"
        );
        assert_eq!(
            WorkerState::Processing { active_tasks: 0 }.label(),
            "processing"
        );
        assert_eq!(
            WorkerState::Processing { active_tasks: 7 }.label(),
            "processing"
        );
    }

    #[test]
    fn can_claim_tasks_only_in_processing() {
        assert!(!WorkerState::Idle.can_claim_tasks());
        assert!(!WorkerState::Connecting.can_claim_tasks());
        assert!(!WorkerState::PullingModel {
            model: "m".into(),
            progress: 50,
        }
        .can_claim_tasks());
        assert!(WorkerState::Processing { active_tasks: 0 }.can_claim_tasks());
        assert!(WorkerState::Processing { active_tasks: 3 }.can_claim_tasks());
        assert!(!WorkerState::Paused.can_claim_tasks());
        assert!(!WorkerState::Error {
            reason: "nope".into()
        }
        .can_claim_tasks());
        assert!(!WorkerState::Shutdown.can_claim_tasks());
    }

    #[test]
    fn is_terminal_only_for_shutdown() {
        assert!(!WorkerState::Idle.is_terminal());
        assert!(!WorkerState::Connecting.is_terminal());
        assert!(!WorkerState::Error { reason: "e".into() }.is_terminal());
        assert!(WorkerState::Shutdown.is_terminal());
    }

    // ---- happy path ----

    #[test]
    fn full_happy_path_idle_to_processing_and_back() {
        let mut m = StateMachine::new();
        // Idle → Connecting
        expect_ok(&mut m, WorkerEvent::Start, &WorkerState::Connecting);
        // Connecting → PullingModel
        expect_ok(
            &mut m,
            WorkerEvent::NeedModel {
                model: "llama2:latest".into(),
            },
            &WorkerState::PullingModel {
                model: "llama2:latest".into(),
                progress: 0,
            },
        );
        // PullingModel progress updates
        expect_ok(
            &mut m,
            WorkerEvent::ModelProgress { percent: 42 },
            &WorkerState::PullingModel {
                model: "llama2:latest".into(),
                progress: 42,
            },
        );
        // PullingModel → Processing
        expect_ok(
            &mut m,
            WorkerEvent::ModelReady,
            &WorkerState::Processing { active_tasks: 0 },
        );
        // TaskStarted increments, TaskCompleted decrements
        expect_ok(
            &mut m,
            WorkerEvent::TaskStarted,
            &WorkerState::Processing { active_tasks: 1 },
        );
        expect_ok(
            &mut m,
            WorkerEvent::TaskStarted,
            &WorkerState::Processing { active_tasks: 2 },
        );
        expect_ok(
            &mut m,
            WorkerEvent::TaskCompleted,
            &WorkerState::Processing { active_tasks: 1 },
        );
        // Pause → Paused, Resume → Connecting
        expect_ok(&mut m, WorkerEvent::Pause, &WorkerState::Paused);
        expect_ok(&mut m, WorkerEvent::Resume, &WorkerState::Connecting);
    }

    #[test]
    fn connected_from_connecting_skips_pulling_model() {
        let mut m = StateMachine::new();
        expect_ok(&mut m, WorkerEvent::Start, &WorkerState::Connecting);
        expect_ok(
            &mut m,
            WorkerEvent::Connected,
            &WorkerState::Processing { active_tasks: 0 },
        );
    }

    #[test]
    fn model_progress_clamps_above_100() {
        let mut m = sm(WorkerState::PullingModel {
            model: "m".into(),
            progress: 50,
        });
        expect_ok(
            &mut m,
            WorkerEvent::ModelProgress { percent: 250 },
            &WorkerState::PullingModel {
                model: "m".into(),
                progress: 100,
            },
        );
    }

    #[test]
    fn task_completed_saturates_at_zero() {
        let mut m = sm(WorkerState::Processing { active_tasks: 0 });
        expect_ok(
            &mut m,
            WorkerEvent::TaskCompleted,
            &WorkerState::Processing { active_tasks: 0 },
        );
    }

    // ---- illegal transitions ----

    #[test]
    fn idle_rejects_unknown_events() {
        let mut m = sm(WorkerState::Idle);
        for ev in [
            WorkerEvent::Connected,
            WorkerEvent::NeedModel { model: "m".into() },
            WorkerEvent::ModelReady,
            WorkerEvent::TaskStarted,
            WorkerEvent::TaskCompleted,
            WorkerEvent::Pause,
            WorkerEvent::Resume,
            WorkerEvent::Clear,
            WorkerEvent::ModelProgress { percent: 10 },
        ] {
            expect_invalid(&mut m, ev);
        }
    }

    #[test]
    fn processing_rejects_connected_and_start() {
        let mut m = sm(WorkerState::Processing { active_tasks: 0 });
        expect_invalid(&mut m, WorkerEvent::Start);
        expect_invalid(&mut m, WorkerEvent::Connected);
    }

    #[test]
    fn error_only_accepts_clear_shutdown_or_fail() {
        let mut m = sm(WorkerState::Error {
            reason: "boom".into(),
        });
        expect_invalid(&mut m, WorkerEvent::Start);
        expect_invalid(&mut m, WorkerEvent::Pause);
        expect_invalid(&mut m, WorkerEvent::Resume);
        // Clear → Idle
        expect_ok(&mut m, WorkerEvent::Clear, &WorkerState::Idle);
    }

    #[test]
    fn shutdown_is_terminal_and_rejects_everything() {
        let mut m = sm(WorkerState::Shutdown);
        for ev in [
            WorkerEvent::Start,
            WorkerEvent::Connected,
            WorkerEvent::ModelReady,
            WorkerEvent::TaskStarted,
            WorkerEvent::Pause,
            WorkerEvent::Resume,
            WorkerEvent::Clear,
            WorkerEvent::Shutdown,
            WorkerEvent::Fail { reason: "x".into() },
        ] {
            expect_invalid(&mut m, ev);
        }
    }

    // ---- universally-accepted events ----

    #[test]
    fn fail_from_any_non_shutdown_state_goes_to_error() {
        for initial in [
            WorkerState::Idle,
            WorkerState::Connecting,
            WorkerState::PullingModel {
                model: "m".into(),
                progress: 10,
            },
            WorkerState::Processing { active_tasks: 5 },
            WorkerState::Paused,
            WorkerState::Error {
                reason: "old".into(),
            },
        ] {
            let mut m = sm(initial.clone());
            expect_ok(
                &mut m,
                WorkerEvent::Fail {
                    reason: "boom".into(),
                },
                &WorkerState::Error {
                    reason: "boom".into(),
                },
            );
        }
    }

    #[test]
    fn shutdown_from_any_non_shutdown_state_goes_to_shutdown() {
        for initial in [
            WorkerState::Idle,
            WorkerState::Connecting,
            WorkerState::PullingModel {
                model: "m".into(),
                progress: 0,
            },
            WorkerState::Processing { active_tasks: 0 },
            WorkerState::Paused,
            WorkerState::Error { reason: "e".into() },
        ] {
            let mut m = sm(initial);
            expect_ok(&mut m, WorkerEvent::Shutdown, &WorkerState::Shutdown);
        }
    }

    // ---- apply return value ----

    #[test]
    fn apply_returns_previous_state_on_success() {
        let mut m = StateMachine::new();
        let prev = m.apply(WorkerEvent::Start).unwrap();
        assert_eq!(prev, WorkerState::Idle);
        assert_eq!(m.state(), &WorkerState::Connecting);
    }

    #[test]
    fn apply_does_not_mutate_on_error() {
        let mut m = sm(WorkerState::Connecting);
        let err = m.apply(WorkerEvent::ModelReady).unwrap_err();
        assert_eq!(err.state, "connecting");
        assert_eq!(err.event, "model_ready");
        assert_eq!(m.state(), &WorkerState::Connecting);
    }

    // ---- serde round-trip (state + event + machine) ----

    #[test]
    fn state_serde_round_trip() {
        let states = [
            WorkerState::Idle,
            WorkerState::Connecting,
            WorkerState::PullingModel {
                model: "llama2".into(),
                progress: 50,
            },
            WorkerState::Processing { active_tasks: 3 },
            WorkerState::Paused,
            WorkerState::Error {
                reason: "nope".into(),
            },
            WorkerState::Shutdown,
        ];
        for s in states {
            let json = serde_json::to_string(&s).unwrap();
            let back: WorkerState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back, "round-trip must be stable for {s:?}");
        }
    }
}
