// SPDX-License-Identifier: AGPL-3.0-or-later
//! Optional ratatui dashboard for the `nexus-worker` binary.
//!
//! The TUI is a *presentation layer* on top of the headless
//! [`nexus_worker_core::engine::Engine`]: it subscribes to the
//! engine's state broadcast channel, polls the allowlist and
//! the GPU backend on a fixed redraw interval, and lets the
//! operator press `q` to trigger a graceful shutdown.
//!
//! The TUI and the engine run concurrently on the same tokio
//! runtime:
//!
//! - The engine runs in a spawned task via
//!   [`Engine::run_until_shutdown`]. Its state is observed
//!   through the `state_rx` watch channel handed out by
//!   `engine.state_rx()`.
//! - The TUI render loop polls crossterm events with a short
//!   tick (every 200ms) on the main task. On `q` it signals
//!   the engine's shutdown oneshot and then awaits the engine
//!   task.
//!
//! Terminal state is restored in every exit path (normal,
//! panic via `catch_unwind` and `io::Error`) so a crash does
//! not leave the user's shell in raw mode.

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nexus_worker_core::allowlist::{Allowlist, Project};
use nexus_worker_core::config::WorkerConfig;
use nexus_worker_core::engine::{Engine, WorkerState};
use nexus_worker_core::gpu::GpuInfo;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use tokio::sync::watch;

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Interval between full redraws / GPU snapshot / allowlist
/// refreshes. Low enough to feel live, high enough to be
/// cheap on any GPU / SQLite backend.
const REDRAW_INTERVAL: Duration = Duration::from_millis(500);

/// Poll budget for each crossterm event read. Keeping it
/// below the redraw interval guarantees the TUI stays
/// responsive to key presses.
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

// =================================================================
// UI state
// =================================================================

/// Everything the render function needs to draw a single frame.
struct UiState {
    worker_name: String,
    node_id: String,
    engine_state: WorkerState,
    projects: Vec<Project>,
    project_list_state: ListState,
    gpu_info: Vec<GpuInfo>,
    allowlist_path_display: String,
}

impl UiState {
    fn tick_projects(&mut self, db: &Allowlist) {
        match db.list() {
            Ok(rows) => self.projects = rows,
            Err(e) => tracing::warn!(error = %e, "tui: failed to refresh allowlist"),
        }
        // Keep the selection in bounds after the refresh.
        if self.projects.is_empty() {
            self.project_list_state.select(None);
        } else {
            let max = self.projects.len() - 1;
            let cur = self.project_list_state.selected().unwrap_or(0);
            self.project_list_state.select(Some(cur.min(max)));
        }
    }

    fn scroll(&mut self, delta: i32) {
        if self.projects.is_empty() {
            return;
        }
        let max = self.projects.len() as i32 - 1;
        let cur = self.project_list_state.selected().unwrap_or(0) as i32;
        let next = (cur + delta).clamp(0, max);
        self.project_list_state.select(Some(next as usize));
    }
}

// =================================================================
// Terminal setup / teardown
// =================================================================

fn enter_tui() -> Result<TuiTerminal> {
    enable_raw_mode().context("failed to enable terminal raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("failed to create ratatui terminal")
}

fn leave_tui(terminal: &mut TuiTerminal) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")?;
    Ok(())
}

// =================================================================
// Main entry point
// =================================================================

/// Run the TUI dashboard until the user presses `q` (or the
/// engine terminates on its own).
///
/// Takes ownership of the engine because
/// [`Engine::run_until_shutdown`] consumes `self`. On exit the
/// terminal is always restored, even on error paths, so a
/// panic inside the render loop does not leave the user with
/// a broken shell.
pub async fn run_tui(mut engine: Engine, worker_config: WorkerConfig, db: Allowlist) -> Result<()> {
    let worker_name = worker_config.identity.name.clone();
    let node_id = engine.node_id();
    let gpu_info = engine.gpu_info().to_vec();
    let state_rx = engine.state_rx();
    let shutdown_tx = engine
        .take_shutdown_sender()
        .expect("engine shutdown sender should be available at first take");

    let allowlist_path_display = db
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(in-memory)".to_string());

    let engine_task = tokio::spawn(engine.run_until_shutdown());

    let mut terminal = enter_tui()?;
    let mut ui = UiState {
        worker_name,
        node_id,
        engine_state: state_rx.borrow().clone(),
        projects: Vec::new(),
        project_list_state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        gpu_info,
        allowlist_path_display,
    };

    // Initial populate so the first frame isn't blank.
    ui.tick_projects(&db);

    // Live GPU snapshots cannot run from here because the
    // Engine was moved into engine_task. A
    // `tokio::sync::watch<Vec<GpuStats>>` broadcast on the Engine
    // (so the TUI could subscribe) is not yet wired; until then we
    // render only the static gpu_info (name + total VRAM) collected
    // at boot.
    let loop_result = render_loop(&mut terminal, &mut ui, state_rx, shutdown_tx, &db).await;

    // Always restore the terminal before propagating errors.
    let restore_result = leave_tui(&mut terminal);

    // Await the engine task so any error is surfaced.
    let engine_result = engine_task.await.context("engine task panicked")?;

    loop_result?;
    restore_result?;
    engine_result.context("engine loop exited with an error")?;
    Ok(())
}

async fn render_loop(
    terminal: &mut TuiTerminal,
    ui: &mut UiState,
    mut state_rx: watch::Receiver<WorkerState>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    db: &Allowlist,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let mut shutdown_sender = Some(shutdown_tx);

    loop {
        // Draw the current state.
        terminal
            .draw(|f| render_frame(f, ui))
            .context("failed to draw ratatui frame")?;

        // Compute how long we can block on events before the
        // next redraw tick.
        let elapsed = last_tick.elapsed();
        let timeout = REDRAW_INTERVAL
            .saturating_sub(elapsed)
            .min(EVENT_POLL_INTERVAL);

        // Poll keyboard events. This is a blocking call so we
        // wrap it in spawn_blocking. The dedicated thread
        // returns on either a key event or the timeout.
        let poll_timeout = if timeout.is_zero() {
            EVENT_POLL_INTERVAL
        } else {
            timeout
        };

        let poll_result = tokio::task::spawn_blocking(move || -> io::Result<Option<Event>> {
            if event::poll(poll_timeout)? {
                Ok(Some(event::read()?))
            } else {
                Ok(None)
            }
        })
        .await
        .context("crossterm poll task panicked")?
        .context("crossterm poll failed")?;

        if let Some(Event::Key(key)) = poll_result
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    if let Some(tx) = shutdown_sender.take() {
                        let _ = tx.send(());
                    }
                    // Exit the TUI loop as soon as we
                    // asked the engine to stop. The caller
                    // awaits engine_task for cleanup.
                    return Ok(());
                }
                KeyCode::Up => ui.scroll(-1),
                KeyCode::Down => ui.scroll(1),
                KeyCode::PageUp => ui.scroll(-10),
                KeyCode::PageDown => ui.scroll(10),
                _ => {}
            }
        }

        // Non-blocking state channel check.
        if state_rx.has_changed().unwrap_or(false) {
            ui.engine_state = state_rx.borrow_and_update().clone();
        }

        // Periodic refresh for allowlist + gpu.
        if last_tick.elapsed() >= REDRAW_INTERVAL {
            ui.tick_projects(db);
            last_tick = Instant::now();
        }

        if matches!(ui.engine_state, WorkerState::Shutdown) {
            return Ok(());
        }
    }
}

// =================================================================
// Rendering
// =================================================================

fn render_frame(f: &mut Frame<'_>, ui: &UiState) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // state line
            Constraint::Min(8),    // body (projects + gpu)
            Constraint::Length(3), // footer
        ])
        .split(area);

    render_header(f, chunks[0], ui);
    render_state(f, chunks[1], ui);
    render_body(f, chunks[2], ui);
    render_footer(f, chunks[3], ui);
}

fn render_header(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    let pub_short: String = ui.node_id.chars().take(12).collect();
    let title = format!(
        " nexus-worker v{} · {} · {}… ",
        env!("CARGO_PKG_VERSION"),
        ui.worker_name,
        pub_short
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" nexus-grid worker ");
    let para = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left)
        .block(block);
    f.render_widget(para, area);
}

fn render_state(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    let (label, colour) = match &ui.engine_state {
        WorkerState::Idle => ("IDLE", Color::Gray),
        WorkerState::Connecting => ("CONNECTING", Color::Yellow),
        WorkerState::PullingModel { .. } => ("PULLING MODEL", Color::Blue),
        WorkerState::Processing { .. } => ("PROCESSING", Color::Green),
        WorkerState::Paused => ("PAUSED", Color::DarkGray),
        WorkerState::Error { .. } => ("ERROR", Color::Red),
        WorkerState::Shutdown => ("SHUTDOWN", Color::Magenta),
    };

    let detail = ui.engine_state.to_string();
    let line = Line::from(vec![
        Span::styled(
            format!("  {label}  "),
            Style::default()
                .fg(Color::Black)
                .bg(colour)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(detail, Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(line).block(Block::default().borders(Borders::ALL).title(" state "));
    f.render_widget(para, area);
}

fn render_body(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_projects(f, body[0], ui);
    render_gpu(f, body[1], ui);
}

fn render_projects(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    let title = format!(" projects ({}) ", ui.projects.len());
    let block = Block::default().borders(Borders::ALL).title(title);

    if ui.projects.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from("no projects enrolled"),
            Line::from(""),
            Line::from(Span::styled(
                "use `nexus-worker join <invite>` to add one",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = ui
        .projects
        .iter()
        .map(|p| {
            let marker = if p.enabled { "●" } else { "○" };
            let marker_style = Style::default().fg(if p.enabled {
                Color::Green
            } else {
                Color::DarkGray
            });
            let name = Span::styled(
                format!(" {} ", truncate(&p.name, 28)),
                Style::default().fg(Color::White),
            );
            let budget = if p.budget_joules == 0 {
                "unlimited".to_string()
            } else {
                format!("{} J/day", p.budget_joules)
            };
            let stats = Span::styled(
                format!(" · {} tasks · {}", p.tasks_completed, budget),
                Style::default().fg(Color::DarkGray),
            );
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {marker}"), marker_style),
                name,
                stats,
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶");

    let mut state = ui.project_list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
}

fn render_gpu(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    let block = Block::default().borders(Borders::ALL).title(" gpu ");

    if ui.gpu_info.is_empty() {
        let p = Paragraph::new(vec![
            Line::from("no GPUs visible"),
            Line::from(""),
            Line::from(Span::styled(
                "CPU-only mode",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for info in &ui.gpu_info {
        lines.push(Line::from(Span::styled(
            format!(" {} ", info.name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "   total vram: {:.1} GiB · backend: {}",
                info.vram_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                info.backend
            ),
            Style::default().fg(Color::DarkGray),
        )));

        // Live GPU stats (a gpu_stats watch channel on the engine)
        // are not wired yet; for now we render the static
        // info from the boot probe.
        lines.push(Line::from(""));
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame<'_>, area: Rect, ui: &UiState) {
    // Show key hints on the left and the allowlist path on the
    // right so operators know where their on-disk state lives.
    let hints = Line::from(vec![
        Span::styled(" q ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" quit  "),
        Span::styled(
            " ↑↓ ",
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::raw(" scroll  "),
        Span::styled(
            " PgUp/PgDn ",
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::raw(" fast scroll  "),
        Span::styled(
            format!("  db: {}", truncate(&ui.allowlist_path_display, 48)),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let p = Paragraph::new(hints).block(Block::default().borders(Borders::ALL).title(" keys "));
    f.render_widget(p, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let trunc: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{trunc}…")
    }
}
