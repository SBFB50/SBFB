// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::process;

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

pub fn build_router(root: PathBuf) -> Router {
    let state = OperatorState {
        root,
        action_log: Arc::new(Mutex::new(Vec::new())),
        chat_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        .route("/api/chat/{id}/log", get(handle_chat_log))
        .layer(cors)
        .with_state(state)
}

pub async fn run_server(port: u16, once_smoke: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::process::repo_root_pub();
    let app = build_router(root);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    let actual_port = listener.local_addr()?.port();
    println!("READY 127.0.0.1:{actual_port}");

    if once_smoke {
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let resp = reqwest::get(format!("http://127.0.0.1:{actual_port}/api/status")).await?;
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

async fn handle_audit(
    State(state): State<OperatorState>,
    Path(rev): Path<String>,
) -> impl IntoResponse {
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
        "intent": req.intent,
        "chat_history_authoritative": false,
        "notice": "private chat history is non-authoritative",
    });

    let session = ChatSession {
        context_pack: context_pack.clone(),
        messages: Vec::new(),
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
