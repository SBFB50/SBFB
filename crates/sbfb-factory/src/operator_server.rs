// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::header::HeaderName;
use axum::http::{Method, StatusCode, header};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::auth::{self, AuthState};
use crate::llm_bridge;
use crate::process;
use crate::provider_router;

const ACTION_ALLOWLIST: &[&str] = &["status-sprint", "lint-planning", "audit-commit", "prompt"];

const ARTIFACT_DRAFT_ALLOWLIST: &[&str] = &[
    ".planning/active/",
    "docs/agent/",
    "docs/claude/",
    "prompts/agent/",
    "AGENTS.md",
    "CLAUDE.md",
];

const SENSITIVE_ACTIONS: &[&str] = &["shell", "commit", "push", "PASS"];

#[derive(Clone)]
pub struct OperatorState {
    root: PathBuf,
    action_log: Arc<Mutex<Vec<ActionLogEntry>>>,
    chat_sessions: Arc<Mutex<HashMap<String, ChatSession>>>,
}

#[derive(Serialize, Clone)]
struct ActionLogEntry {
    timestamp: String,
    action: String,
    args: serde_json::Value,
    result: String,
}

#[derive(Clone)]
struct ChatSession {
    context_pack: serde_json::Value,
    messages: Vec<ChatMessage>,
    /// Agent model for the streamed spawn. Defaults to
    /// [`default_model`]; overridable per-send via `ChatSendRequest`.
    /// Persisted here because the SSE GET `/chat/{id}/stream` carries
    /// no body to read it from.
    model: String,
    /// Execution target for the streamed turn — `claude` (default
    /// pilot), `ollama`/`local`, or `network` (Sprint 72 Phase D). Set
    /// at session creation and overridable per-send (symmetry with
    /// `model`); read back by the bodyless SSE GET to build the
    /// [`provider_router::ExecutionTarget`].
    provider: String,
    /// Project the `network` target submits the task under. Defaults to
    /// [`default_project_id`]; only consumed by the network arm.
    project_id: String,
}

#[derive(Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn log_action(state: &OperatorState, action: &str, args: serde_json::Value, result: &str) {
    if let Ok(mut log) = state.action_log.lock() {
        log.push(ActionLogEntry {
            timestamp: now_rfc3339(),
            action: action.to_string(),
            args,
            result: result.to_string(),
        });
    }
}

pub fn build_router(root: PathBuf, auth_state: AuthState) -> Router {
    let state = OperatorState {
        root,
        action_log: Arc::new(Mutex::new(Vec::new())),
        chat_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    // G7 (D5): restrict CORS to loopback origins, explicit methods,
    // and the bearer header — replaces the prior `Any/Any/Any`. This
    // is browser-side defence in depth; the server-side enforcement
    // is the `auth::auth_required` middleware below (Host + Origin +
    // token on every route).
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _parts| {
            origin
                .to_str()
                .map(auth::is_loopback_origin)
                .unwrap_or(false)
        }))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static(auth::AUTH_HEADER),
        ]);

    Router::new()
        .route("/api/status", get(handle_status))
        .route("/api/lint", get(handle_lint))
        .route("/api/audit/{rev}", get(handle_audit))
        .route("/api/prompt/{kind}", get(handle_prompt))
        .route("/api/context", get(handle_context))
        .route("/api/context-pack", post(handle_context_pack))
        .route("/api/providers", get(handle_providers))
        .route("/api/actions/run", post(handle_action_run))
        .route("/api/actions/log", get(handle_action_log))
        .route("/api/artifacts/draft", post(handle_artifact_draft))
        .route("/api/chat/session", post(handle_chat_session))
        .route("/api/chat/message", post(handle_chat_message))
        .route("/api/chat/{id}/send", post(handle_chat_send))
        .route("/api/chat/{id}/stream", get(handle_chat_stream))
        .route("/api/chat/{id}/log", get(handle_chat_log))
        .route("/api/sprint-history", get(handle_sprint_history))
        .route("/api/sprint-history/all", get(handle_all_sprints))
        .route(
            "/api/sprint-history/{sprint}",
            get(handle_sprint_history_by_number),
        )
        .route("/api/sprint-history/diff/{sha}", get(handle_commit_diff))
        .route("/api/terminal/ws", get(handle_terminal_ws))
        .route("/api/terminal/sessions", get(handle_terminal_sessions))
        .route(
            "/api/terminal/sessions/{name}",
            get(handle_terminal_session_content),
        )
        // Inner: `auth_required` guards every data-bearing request
        // (GET/POST/WS upgrade) with Host + Origin + token. Outer:
        // CORS, so a browser's OPTIONS *preflight* is answered before
        // the token check. That is intentional and harmless: a
        // preflight carries no body and triggers no handler — it only
        // tells the browser whether the *real* request is allowed, and
        // that real request still passes through `auth_required` (a
        // foreign Host/Origin or a missing token is rejected there).
        // CORS must stay outer; if auth were outer it would 401 every
        // preflight (browsers never send the bearer on a preflight) and
        // break legitimate cross-origin use.
        .layer(axum::middleware::from_fn_with_state(
            auth_state,
            auth::auth_required,
        ))
        .layer(cors)
        .with_state(state)
}

pub async fn run_server(port: u16, once_smoke: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::process::repo_root_pub();
    let token = auth::load_or_generate_token()?;
    let app = build_router(root, AuthState::new(token.clone()));

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    let actual_port = listener.local_addr()?.port();
    println!("READY 127.0.0.1:{actual_port}");

    if once_smoke {
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let resp = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{actual_port}/api/status"))
            .header(auth::AUTH_HEADER, &token)
            .send()
            .await?;
        if resp.status().is_success() {
            println!("smoke: /api/status OK");
        } else {
            eprintln!("smoke: /api/status returned {}", resp.status());
            std::process::exit(1);
        }
        handle.abort();
        Ok(())
    } else {
        axum::serve(listener, app).await?;
        Ok(())
    }
}

// --- Handlers ---

async fn handle_status(State(state): State<OperatorState>) -> impl IntoResponse {
    match process::status_sprint_data(&state.root) {
        Some(result) => Json(serde_json::to_value(result).unwrap()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active sprint"})),
        )
            .into_response(),
    }
}

async fn handle_lint(State(state): State<OperatorState>) -> Json<serde_json::Value> {
    let result = process::lint_planning_data(&state.root);
    Json(serde_json::to_value(result).unwrap())
}

/// Reject a user-supplied git rev/sha that git could parse as an option
/// (leading `-`) or that carries whitespace/control bytes. The Operator
/// shells `git log`/`git diff` with the raw rev; without this a value such
/// as `--output=<path>` is interpreted as a git option and writes an
/// arbitrary file (git option injection, live-demonstrated by the S71
/// Phase D retro-Codex). Defense in depth: the git calls themselves pass
/// `--end-of-options` (see `sprint_history.rs` / `process.rs`).
fn is_safe_git_rev(rev: &str) -> bool {
    !rev.is_empty()
        && !rev.starts_with('-')
        && !rev.contains(|c: char| c.is_whitespace() || c.is_control())
}

async fn handle_audit(
    State(state): State<OperatorState>,
    Path(rev): Path<String>,
) -> impl IntoResponse {
    if !is_safe_git_rev(&rev) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid rev"})),
        )
            .into_response();
    }
    match process::audit_commit_data(&state.root, &rev) {
        Ok(result) => Json(serde_json::to_value(result).unwrap()).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct PromptQuery {
    provider: Option<String>,
    depth: Option<String>,
}

async fn handle_prompt(
    Path(kind): Path<String>,
    Query(params): Query<PromptQuery>,
) -> impl IntoResponse {
    let provider = params.provider.as_deref().unwrap_or("claude");
    let depth = params.depth.as_deref().unwrap_or("standard");
    match process::prompt_data(&kind, depth, provider) {
        Ok(content) => {
            Json(serde_json::json!({"kind": kind, "provider": provider, "content": content}))
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn handle_context(State(state): State<OperatorState>) -> Json<serde_json::Value> {
    Json(process::context_data(&state.root))
}

#[derive(Deserialize)]
struct ContextPackRequest {
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default)]
    intent: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    specialized_kind: String,
}

fn default_provider() -> String {
    "claude".to_string()
}

/// Default project the `network` execution target submits under
/// (Sprint 72 Phase D). Only consumed by the network arm; the
/// Claude/Ollama arms ignore it.
fn default_project_id() -> String {
    "operator-chat".to_string()
}

/// G9 (D4): default agent model for the Claude arm. The frozen model rule
/// mandates the explicit id `claude-opus-4-8[1m]` everywhere a Claude agent
/// is invoked — never the alias `opus` nor the prior hardcoded `sonnet`.
fn default_model() -> String {
    "claude-opus-4-8[1m]".to_string()
}

/// Per-provider default model (P2-OLLAMA-MODEL-PICKER, S73 Phase B).
///
/// The model rule (`claude-opus-4-8[1m]` everywhere) governs **Claude**
/// invocations only. Ollama and Network must NOT inherit that id — it does
/// not exist in a local Ollama, so every non-Claude intention that omitted a
/// model failed. Each non-Claude provider gets a sensible local default,
/// overridable via env (`SBFB_OLLAMA_DEFAULT_MODEL` / `SBFB_NETWORK_DEFAULT_MODEL`)
/// so an operator can match the models they have pulled without a rebuild.
/// The frontend model-picker sends an explicit model for non-Claude
/// intentions; this default is the safety net when none is chosen.
fn default_model_for_provider(provider: &str) -> String {
    match provider.trim().to_lowercase().as_str() {
        "ollama" | "local" => std::env::var("SBFB_OLLAMA_DEFAULT_MODEL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "llama3.2:latest".to_string()),
        "network" => std::env::var("SBFB_NETWORK_DEFAULT_MODEL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "llama3.2:latest".to_string()),
        // claude / "" / unknown → the default pilot model (model rule).
        _ => default_model(),
    }
}

fn file_hash(root: &std::path::Path, rel: &str) -> serde_json::Value {
    let path = root.join(rel);
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            let hash = blake3::hash(&bytes);
            return serde_json::json!({
                "path": rel,
                "hash": &hash.to_hex()[..8],
                "exists": true,
            });
        }
    }
    serde_json::json!({"path": rel, "exists": false})
}

async fn handle_context_pack(
    State(state): State<OperatorState>,
    Json(req): Json<ContextPackRequest>,
) -> Json<serde_json::Value> {
    let root = &state.root;

    let ctx = process::context_data(root);
    let sprint = ctx.get("sprint").and_then(|v| v.as_u64()).unwrap_or(0);
    let phase = ctx
        .get("phase")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let active_artifacts: Vec<serde_json::Value> = crate::process::list_active_artifacts_pub(root)
        .into_iter()
        .map(|name| {
            let rel = format!(".planning/active/{name}");
            file_hash(root, &rel)
        })
        .collect();

    let specialized = if !req.specialized_kind.is_empty() {
        let filename = crate::process::prompt_filename_pub(&req.specialized_kind);
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            serde_json::json!({"error": "invalid specialized_kind"})
        } else {
            let rel = format!("prompts/agent/{filename}");
            file_hash(root, &rel)
        }
    } else {
        serde_json::Value::Null
    };

    let pack = serde_json::json!({
        "base_prompt": file_hash(root, "prompts/agent/base.md"),
        "universal_prompt": file_hash(root, "prompts/agent/universal.md"),
        "handoff_prompt": file_hash(root, "prompts/agent/handoff.md"),
        "specialized_prompt": specialized,
        "runtime_context": {
            "repo": ctx.get("repo"),
            "branch": ctx.get("branch"),
            "head": ctx.get("head"),
            "sprint": sprint,
            "phase": phase,
            "dirty_files": ctx.get("dirty_files"),
            "staged_files": ctx.get("staged_files"),
            "recent_commits": ctx.get("recent_commits"),
        },
        "agent_system": file_hash(root, "docs/agent/AGENT_SYSTEM.md"),
        "process_docs": [
            file_hash(root, "docs/agent/PROCESS.md"),
            file_hash(root, "docs/agent/TOOLING.md"),
            file_hash(root, "AGENTS.md"),
            file_hash(root, "CLAUDE.md"),
        ],
        "active_artifacts": active_artifacts,
        "operator_intent": {
            "intent": req.intent,
            "role": req.role,
            "provider": req.provider,
        },
        "chat_history_authoritative": false,
        "notice": "private chat history is non-authoritative",
    });

    log_action(
        &state,
        "context-pack",
        serde_json::json!({"provider": req.provider}),
        "ok",
    );
    Json(pack)
}

async fn handle_providers() -> Json<serde_json::Value> {
    let providers: Vec<serde_json::Value> = process::providers_list()
        .into_iter()
        .map(|p| serde_json::Value::String(p.to_string()))
        .collect();
    Json(serde_json::json!({"providers": providers}))
}

#[derive(Deserialize)]
struct ActionRunRequest {
    command: String,
    #[serde(default)]
    args: serde_json::Value,
}

async fn handle_action_run(
    State(state): State<OperatorState>,
    Json(req): Json<ActionRunRequest>,
) -> impl IntoResponse {
    if !ACTION_ALLOWLIST.contains(&req.command.as_str()) {
        log_action(
            &state,
            &req.command,
            req.args.clone(),
            "rejected: not in allowlist",
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "command not in allowlist",
                "allowed": ACTION_ALLOWLIST,
            })),
        )
            .into_response();
    }

    let result = match req.command.as_str() {
        "status-sprint" => match process::status_sprint_data(&state.root) {
            Some(r) => serde_json::to_value(r).unwrap(),
            None => serde_json::json!({"error": "no active sprint"}),
        },
        "lint-planning" => serde_json::to_value(process::lint_planning_data(&state.root)).unwrap(),
        "audit-commit" => {
            let rev = req
                .args
                .get("rev")
                .and_then(|v| v.as_str())
                .unwrap_or("HEAD");
            match process::audit_commit_data(&state.root, rev) {
                Ok(r) => serde_json::to_value(r).unwrap(),
                Err(e) => serde_json::json!({"error": e.to_string()}),
            }
        }
        "prompt" => {
            let kind = req
                .args
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("base");
            let provider = req
                .args
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");
            let depth = req
                .args
                .get("depth")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            match process::prompt_data(kind, depth, provider) {
                Ok(c) => serde_json::json!({"kind": kind, "content": c}),
                Err(e) => serde_json::json!({"error": e.to_string()}),
            }
        }
        _ => unreachable!(),
    };

    log_action(&state, &req.command, req.args.clone(), "ok");
    Json(result).into_response()
}

async fn handle_action_log(State(state): State<OperatorState>) -> Json<serde_json::Value> {
    let log = state.action_log.lock().unwrap().clone();
    Json(serde_json::to_value(log).unwrap())
}

#[derive(Deserialize)]
struct ArtifactDraftRequest {
    path: String,
    content: String,
}

async fn handle_artifact_draft(
    State(state): State<OperatorState>,
    Json(req): Json<ArtifactDraftRequest>,
) -> impl IntoResponse {
    let normalized = req.path.replace('\\', "/");

    if normalized.contains("..") {
        log_action(
            &state,
            "artifact-draft",
            serde_json::json!({"path": req.path}),
            "rejected: path traversal",
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "path traversal rejected"
            })),
        )
            .into_response();
    }

    if normalized.to_lowercase().contains("verdict") && normalized.to_lowercase().contains("pass") {
        log_action(
            &state,
            "artifact-draft",
            serde_json::json!({"path": req.path}),
            "rejected: PASS verdict",
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "cannot write PASS verdict via Operator — use review/gate flow"
            })),
        )
            .into_response();
    }

    let has_final_pass = req.content.lines().any(|l| {
        let t = l.trim();
        (t.starts_with("## Verdict") || t.starts_with("## Verdict"))
            && t.contains("PASS")
            && !t.contains("PASS-PENDING")
    });
    if has_final_pass {
        log_action(
            &state,
            "artifact-draft",
            serde_json::json!({"path": req.path}),
            "rejected: PASS verdict in content",
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "cannot write PASS verdict via Operator — use review/gate flow"
            })),
        )
            .into_response();
    }

    let allowed = ARTIFACT_DRAFT_ALLOWLIST.iter().any(|entry| {
        if entry.ends_with('/') {
            normalized.starts_with(entry)
        } else {
            normalized == *entry
        }
    });

    if !allowed {
        log_action(
            &state,
            "artifact-draft",
            serde_json::json!({"path": req.path}),
            "rejected: path not in allowlist",
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "path not in artifact draft allowlist",
                "allowed_prefixes": ARTIFACT_DRAFT_ALLOWLIST,
            })),
        )
            .into_response();
    }

    let target = state.root.join(&normalized);
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&target, &req.content) {
        Ok(()) => {
            log_action(
                &state,
                "artifact-draft",
                serde_json::json!({"path": req.path, "bytes": req.content.len()}),
                "ok",
            );
            Json(serde_json::json!({"ok": true, "path": req.path})).into_response()
        }
        Err(e) => {
            log_action(
                &state,
                "artifact-draft",
                serde_json::json!({"path": req.path}),
                &format!("error: {e}"),
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct ChatSessionRequest {
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default)]
    intent: String,
    // Sprint 72 Phase D: project the `network` target submits under.
    // Runtime tolerance (pre-launch policy): omitted → `operator-chat`.
    #[serde(default = "default_project_id")]
    project_id: String,
}

async fn handle_chat_session(
    State(state): State<OperatorState>,
    Json(req): Json<ChatSessionRequest>,
) -> Json<serde_json::Value> {
    let id = format!("chat-{}", now_rfc3339().replace(':', "-"));

    let root = &state.root;
    let ctx = process::context_data(root);

    let context_pack = serde_json::json!({
        "base_prompt": file_hash(root, "prompts/agent/base.md"),
        "universal_prompt": file_hash(root, "prompts/agent/universal.md"),
        "handoff_prompt": file_hash(root, "prompts/agent/handoff.md"),
        "runtime_context": {
            "head": ctx.get("head"),
            "sprint": ctx.get("sprint"),
            "phase": ctx.get("phase"),
        },
        "provider": req.provider,
        "project_id": req.project_id,
        "intent": req.intent,
        "chat_history_authoritative": false,
        "notice": "private chat history is non-authoritative",
    });

    let session = ChatSession {
        context_pack: context_pack.clone(),
        messages: Vec::new(),
        // P2-OLLAMA-MODEL-PICKER (S73 Phase B): seed the session with the
        // chosen provider's default model, not the Claude id unconditionally.
        model: default_model_for_provider(&req.provider),
        provider: req.provider.clone(),
        project_id: req.project_id.clone(),
    };

    state
        .chat_sessions
        .lock()
        .unwrap()
        .insert(id.clone(), session);

    log_action(
        &state,
        "chat-session",
        serde_json::json!({"id": &id, "provider": req.provider}),
        "created",
    );

    Json(serde_json::json!({
        "id": id,
        "context_pack": context_pack,
    }))
}

#[derive(Deserialize)]
struct ChatMessageRequest {
    session_id: String,
    message: String,
}

async fn handle_chat_message(
    State(state): State<OperatorState>,
    Json(req): Json<ChatMessageRequest>,
) -> impl IntoResponse {
    let mut sessions = state.chat_sessions.lock().unwrap();
    let session = match sessions.get_mut(&req.session_id) {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "session not found"})),
            )
                .into_response();
        }
    };

    let lower = req.message.to_lowercase();
    let is_sensitive = SENSITIVE_ACTIONS
        .iter()
        .any(|a| lower.contains(&a.to_lowercase()));

    session.messages.push(ChatMessage {
        role: "user".into(),
        content: req.message.clone(),
        action: None,
    });

    if is_sensitive {
        let response = ChatMessage {
            role: "system".into(),
            content: "This action requires external verification via a real agent session with repo-visible proofs.".into(),
            action: Some("requires_gate".into()),
        };
        session.messages.push(response.clone());

        log_action(
            &state,
            "chat-message",
            serde_json::json!({"session": req.session_id, "sensitive": true}),
            "requires_gate",
        );

        return Json(serde_json::json!({
            "response": response.content,
            "requires_gate": true,
            "requires_external_agent": true,
        }))
        .into_response();
    }

    let response = ChatMessage {
        role: "assistant".into(),
        content: "Agent integration pending — connect a provider session to execute.".into(),
        action: None,
    };
    session.messages.push(response.clone());

    log_action(
        &state,
        "chat-message",
        serde_json::json!({"session": req.session_id}),
        "ok",
    );

    Json(serde_json::json!({
        "response": response.content,
        "requires_gate": false,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct ChatSendRequest {
    message: String,
    #[serde(default = "default_provider")]
    provider: String,
    // Runtime tolerance (pre-launch policy): a client omitting `model` sends
    // an empty string (not a 422); the handler then resolves the model from
    // the chosen provider (P2-OLLAMA-MODEL-PICKER) rather than forcing the
    // Claude id onto Ollama/Network.
    #[serde(default)]
    model: String,
}

async fn handle_chat_send(
    State(state): State<OperatorState>,
    Path(id): Path<String>,
    Json(req): Json<ChatSendRequest>,
) -> impl IntoResponse {
    let mut sessions = state.chat_sessions.lock().unwrap();
    let session = match sessions.get_mut(&id) {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "session not found"})),
            )
                .into_response();
        }
    };

    // Sprint 72 Phase D: persist the requested execution target so the
    // bodyless SSE GET routes to the chosen provider. Resolved BEFORE the
    // model so the model default matches the (possibly updated) provider.
    if !req.provider.trim().is_empty() {
        session.provider = req.provider.clone();
    }
    // G9 (D4) + P2-OLLAMA-MODEL-PICKER (S73 Phase B): persist the model for
    // THIS turn's provider so the bodyless SSE GET reads it back. An explicit
    // client model wins; otherwise it resolves to the provider's own default
    // — Ollama/Network never inherit the Claude id `claude-opus-4-8[1m]`.
    let requested_model = req.model.trim();
    session.model = if requested_model.is_empty() {
        default_model_for_provider(&session.provider)
    } else {
        requested_model.to_string()
    };

    let lower = req.message.to_lowercase();
    let is_sensitive = SENSITIVE_ACTIONS
        .iter()
        .any(|a| lower.contains(&a.to_lowercase()));

    session.messages.push(ChatMessage {
        role: "user".into(),
        content: req.message.clone(),
        action: None,
    });

    if is_sensitive {
        session.messages.push(ChatMessage {
            role: "system".into(),
            content: "This action requires external verification via a real agent session.".into(),
            action: Some("requires_gate".into()),
        });

        return Json(serde_json::json!({
            "ok": false,
            "requires_gate": true,
        }))
        .into_response();
    }

    log_action(
        &state,
        "chat-send",
        serde_json::json!({"session": &id, "provider": &req.provider, "model": &req.model}),
        "queued",
    );

    Json(serde_json::json!({
        "ok": true,
        "provider": req.provider,
    }))
    .into_response()
}

type SseStream = std::pin::Pin<Box<dyn futures::Stream<Item = Result<Event, Infallible>> + Send>>;

fn sse_error(msg: &str) -> Sse<SseStream> {
    let json = format!(r#"{{"type":"error","message":"{}"}}"#, msg);
    let stream: SseStream = Box::pin(futures::stream::once(async move {
        Ok::<_, Infallible>(Event::default().data(json))
    }));
    Sse::new(stream)
}

/// G2 (D3): single-event SSE telling the client the action is gated.
/// Coherent with the `requires_gate` signal emitted by
/// `handle_chat_message` and `handle_chat_send`.
fn sse_gate(msg: &str) -> Sse<SseStream> {
    let json = format!(r#"{{"type":"requires_gate","message":"{}"}}"#, msg);
    let stream: SseStream = Box::pin(futures::stream::once(async move {
        Ok::<_, Infallible>(Event::default().data(json))
    }));
    Sse::new(stream)
}

async fn handle_chat_stream(
    State(state): State<OperatorState>,
    Path(id): Path<String>,
) -> Sse<SseStream> {
    let (context_pack, history, last_user_msg, model, provider, project_id) = {
        let sessions = state.chat_sessions.lock().unwrap();
        match sessions.get(&id) {
            Some(session) => {
                let hist: Vec<(String, String)> = session
                    .messages
                    .iter()
                    .filter(|m| m.role == "user" || m.role == "assistant")
                    .map(|m| (m.role.clone(), m.content.clone()))
                    .collect();
                let last = session
                    .messages
                    .iter()
                    .rev()
                    .find(|m| m.role == "user")
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                (
                    session.context_pack.clone(),
                    hist,
                    last,
                    session.model.clone(),
                    session.provider.clone(),
                    session.project_id.clone(),
                )
            }
            None => return sse_error("session not found"),
        }
    };

    if last_user_msg.is_empty() {
        return sse_error("no user message");
    }

    // G2 (D3): the SSE was the only chat path that bypassed the
    // SENSITIVE_ACTIONS gate — it spawned a `bypassPermissions` agent
    // with no confirmation. Gate it like `/chat/message` and
    // `/chat/send`: a sensitive last user message returns
    // `requires_gate` and never spawns an autonomous agent. The
    // bypassPermissions happy-path (PO-2) is preserved for benign
    // turns.
    let lower = last_user_msg.to_lowercase();
    let is_sensitive = SENSITIVE_ACTIONS
        .iter()
        .any(|a| lower.contains(&a.to_lowercase()));
    if is_sensitive {
        log_action(
            &state,
            "chat-stream",
            serde_json::json!({"session": id, "sensitive": true}),
            "requires_gate",
        );
        return sse_gate(
            "This action requires external verification via a real agent session with repo-visible proofs.",
        );
    }

    let runtime_ctx = context_pack
        .get("runtime_context")
        .cloned()
        .unwrap_or_default();
    let prompt = llm_bridge::assemble_prompt(&runtime_ctx, &history[..], &last_user_msg);

    let root = state.root.clone();
    let state_clone = state.clone();
    let session_id = id.clone();

    // G9 (D4) + P2-OLLAMA-MODEL-PICKER (S73 Phase B): use the session model;
    // if somehow empty, fall back to the provider's own default (Claude →
    // `claude-opus-4-8[1m]`, Ollama/Network → their local default), never the
    // prior hardcoded `sonnet` and never the Claude id forced onto Ollama.
    let model = if model.trim().is_empty() {
        default_model_for_provider(&provider)
    } else {
        model
    };

    // Sprint 72 Phase D: route the turn to the session's execution target
    // (Claude / Ollama / Network) instead of always spawning Claude. The
    // SENSITIVE_ACTIONS gate above (:866) already ran BEFORE this dispatch
    // — provider-independent, so no provider can bypass it.
    let target = provider_router::ExecutionTarget::from_provider(&provider, &model, &project_id);
    let provider_stream = target.run(prompt, root);

    let sse_stream: SseStream = Box::pin(provider_stream.map(move |chunk| {
        let json = serde_json::to_string(&chunk).unwrap_or_default();

        if let llm_bridge::StreamChunk::Done { result, .. } = &chunk {
            let mut sessions = state_clone.chat_sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(&session_id) {
                session.messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: result.clone(),
                    action: None,
                });
            }
        }

        Ok::<_, Infallible>(Event::default().data(json))
    }));

    Sse::new(sse_stream)
}

async fn handle_chat_log(
    State(state): State<OperatorState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.chat_sessions.lock().unwrap();
    match sessions.get(&id) {
        Some(session) => Json(serde_json::json!({
            "id": id,
            "context_pack": session.context_pack,
            "messages": session.messages,
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "session not found"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct TerminalWsQuery {
    resume: Option<String>,
}

async fn handle_terminal_ws(
    State(state): State<OperatorState>,
    Query(params): Query<TerminalWsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let root = state.root.clone();
    let resume = params.resume;
    ws.on_upgrade(move |socket| async move {
        crate::terminal::handle_terminal_ws(socket, &root, resume.as_deref()).await;
    })
}

async fn handle_terminal_sessions(State(state): State<OperatorState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "sessions": crate::terminal::list_sessions(&state.root),
        "claude_sessions": crate::terminal::list_claude_sessions(&state.root),
    }))
}

async fn handle_terminal_session_content(
    State(state): State<OperatorState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    // Reject path-traversal / separators / Windows drive-prefix (`C:`) before
    // the name is joined into `.planning/terminal/{name}.cast` (S71 Phase D
    // retro-Codex + phase-Codex hardening: a `C:` drive-relative name escaped
    // the terminal dir on Windows).
    if name.is_empty()
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid session name"})),
        )
            .into_response();
    }
    let term_dir = state.root.join(".planning").join("terminal");
    let path = term_dir.join(format!("{name}.cast"));
    // Structural backstop: the resolved file must live DIRECTLY in term_dir
    // (defeats any residual prefix/separator trick the denylist missed).
    if path.parent() != Some(term_dir.as_path()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid session name"})),
        )
            .into_response();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "session not found"})),
        )
            .into_response(),
    }
}

async fn handle_sprint_history(State(state): State<OperatorState>) -> impl IntoResponse {
    match crate::sprint_history::sprint_history_data(&state.root) {
        Some(result) => Json(serde_json::json!(result)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active sprint found"})),
        )
            .into_response(),
    }
}

async fn handle_all_sprints(State(state): State<OperatorState>) -> impl IntoResponse {
    let result = crate::sprint_history::all_sprints_data(&state.root);
    Json(serde_json::json!(result)).into_response()
}

async fn handle_sprint_history_by_number(
    State(state): State<OperatorState>,
    Path(sprint): Path<u32>,
) -> impl IntoResponse {
    match crate::sprint_history::sprint_history_for(&state.root, sprint) {
        Some(result) => Json(serde_json::json!(result)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("sprint {} not found", sprint)})),
        )
            .into_response(),
    }
}

async fn handle_commit_diff(Path(sha): Path<String>) -> impl IntoResponse {
    if sha.len() < 4 || sha.contains("..") || sha.contains('/') || !is_safe_git_rev(&sha) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid sha"})),
        )
            .into_response();
    }
    match crate::sprint_history::commit_diff_data(&sha) {
        Some(result) => Json(serde_json::json!(result)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "commit not found"})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // P2-OLLAMA-MODEL-PICKER (S73 Phase B): the per-provider default model.
    #[test]
    fn claude_provider_keeps_the_frozen_opus_model() {
        // The model rule (`claude-opus-4-8[1m]`) governs Claude invocations.
        assert_eq!(default_model_for_provider("claude"), "claude-opus-4-8[1m]");
        // Empty / unknown providers fall back to the Claude default pilot.
        assert_eq!(default_model_for_provider(""), "claude-opus-4-8[1m]");
        assert_eq!(default_model_for_provider("gpt-9"), "claude-opus-4-8[1m]");
    }

    #[test]
    fn non_claude_providers_do_not_inherit_the_claude_model() {
        // The headline bug: Ollama/Network must NOT default to the Claude id,
        // which does not exist in a local Ollama.
        for provider in ["ollama", "local", "network"] {
            let model = default_model_for_provider(provider);
            assert_ne!(
                model, "claude-opus-4-8[1m]",
                "provider={provider} must not inherit the Claude model"
            );
            assert!(
                !model.is_empty(),
                "provider={provider} must resolve to a concrete default"
            );
        }
    }

    #[test]
    #[serial(sbfb_env)]
    fn ollama_default_model_is_env_overridable() {
        // `#[serial(sbfb_env)]` (P2-A-1 review P1): this mutates a
        // process-global env var, so it must not run concurrently with the
        // other env-mutating tests under plain `cargo test`.
        unsafe {
            std::env::set_var("SBFB_OLLAMA_DEFAULT_MODEL", "qwen2.5-coder:7b");
        }
        let model = default_model_for_provider("ollama");
        unsafe {
            std::env::remove_var("SBFB_OLLAMA_DEFAULT_MODEL");
        }
        assert_eq!(
            model, "qwen2.5-coder:7b",
            "an operator must be able to match their pulled model via env"
        );
    }
}
