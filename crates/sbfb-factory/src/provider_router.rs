// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 72 Phase C (D1/D2) — `ExecutionTarget`: the closed-set
//! dispatch point that sits behind the Operator chat SSE.
//!
//! Today `handle_chat_stream` calls [`spawn_claude_stream`] directly.
//! This module introduces a single enum that routes a chat sub-task to
//! one of three execution targets — **Claude cloud** (default, the
//! principal pilot, behaviour byte-identical to Sprint 71), **Ollama
//! local**, or **the network** (Phase D) — while every arm produces the
//! SAME [`StreamChunk`] contract, so the SSE layer stays provider-
//! agnostic.
//!
//! ## Why an enum, not a `dyn Provider` trait
//!
//! The set of targets is closed and known at compile time. An
//! `async-trait` `Box<dyn Provider>` would double-box (a `Pin<Box<dyn
//! Future>>` per call AND a `Pin<Box<dyn Stream>>` for the result) and
//! lose inlining for no benefit. Enum-dispatch is static, type-safe and
//! extensible cleanly for the network arm (Phase D) and a future GPU
//! arm (Sprint 75). Each arm boxes its heterogeneous `impl Stream` into
//! the shared [`ProviderStream`] alias at the dispatch boundary, which
//! is exactly what the SSE layer already expects.
//!
//! ## Three orthogonal axes (D5, see `docs/rust/PATTERNS.md §P55`)
//!
//! `ExecutionTarget` (where the chat inference runs) is deliberately a
//! distinct axis from the prompt-adaptation `Provider` of `process.rs`
//! (which agent consumes a portable prompt) and the worker `LlmBackend`
//! (the quorum runtime). The type name `ExecutionTarget` (not
//! `Provider`) anchors that distinction in the type system.

use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use futures::stream::{Stream, StreamExt};

use crate::llm_bridge::{StreamChunk, spawn_claude_stream};

/// Default idle timeout for the Ollama arm: the stream is abandoned if
/// Ollama produces no chunk for this long, bounding a hung daemon
/// without truncating a legitimately slow generation. Overridable via
/// `SBFB_OLLAMA_IDLE_TIMEOUT_SECS` (mirrors the Claude arm's
/// `SBFB_CLAUDE_IDLE_TIMEOUT_SECS`, D6 Sprint 71).
const DEFAULT_OLLAMA_IDLE_TIMEOUT_SECS: u64 = 120;

fn ollama_idle_timeout() -> Duration {
    std::env::var("SBFB_OLLAMA_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_OLLAMA_IDLE_TIMEOUT_SECS))
}

/// The unified stream type every execution target produces. Identical to
/// the boxed SSE stream the Operator already serves, so an arm's output
/// drops straight into `handle_chat_stream` (Phase D wiring).
pub type ProviderStream = Pin<Box<dyn Stream<Item = StreamChunk> + Send + 'static>>;

/// Where a chat sub-task is executed. Parsed from the wire `provider`
/// string (`ChatSendRequest.provider`) by [`ExecutionTarget::from_provider`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionTarget {
    /// Claude cloud via the agent CLI — the default pilot (PO-14).
    Claude { model: String },
    /// Ollama on the local machine via `ollama-rs` `generate_stream`.
    Ollama { model: String },
    /// The SBFB network: submit→poll against the local daemon, then a
    /// single `Done` carrying the completed task's text (Sprint 72
    /// Phase D, PO-14 — never token-by-token over the WAN).
    Network { project_id: String, model: String },
}

impl ExecutionTarget {
    /// Parse the wire `provider` string into a closed target. Unknown
    /// values and the empty string fall back to Claude — the principal
    /// pilot (PO-14), never a hard error, matching the
    /// `#[serde(default = "default_provider")]` runtime tolerance on
    /// `ChatSendRequest.provider`.
    pub fn from_provider(provider: &str, model: &str, project_id: &str) -> Self {
        match provider {
            "ollama" | "local" => ExecutionTarget::Ollama {
                model: model.to_owned(),
            },
            "network" => ExecutionTarget::Network {
                project_id: project_id.to_owned(),
                model: model.to_owned(),
            },
            // "claude" and anything unknown → Claude (default pilot).
            _ => ExecutionTarget::Claude {
                model: model.to_owned(),
            },
        }
    }

    /// Dispatch to the matching provider. Each arm returns a stream of
    /// the SAME [`StreamChunk`] contract. `prompt` and `cwd` are owned by
    /// the returned stream, so the `'static` boxed stream holds no borrow.
    pub fn run(self, prompt: String, cwd: PathBuf) -> ProviderStream {
        match self {
            // Claude arm: delegate verbatim to the Sprint 71 path. The
            // idle-timeout + gate behaviour is unchanged — this arm only
            // boxes the existing stream into `ProviderStream`.
            ExecutionTarget::Claude { model } => {
                Box::pin(spawn_claude_stream(&prompt, &model, &cwd))
            }
            ExecutionTarget::Ollama { model } => Box::pin(ollama_stream(model, prompt)),
            ExecutionTarget::Network { project_id, model } => {
                Box::pin(network_stream(project_id, model, prompt))
            }
        }
    }
}

/// Map an `ollama-rs` error string to an operator-facing diagnostic.
/// Connection failures get an actionable install/run hint; everything
/// else is surfaced verbatim under an `Ollama error:` prefix.
fn ollama_diagnostic(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let looks_unreachable = lower.contains("connection refused")
        || lower.contains("tcp connect error")
        || lower.contains("failed to connect")
        || lower.contains("connection reset")
        || lower.contains("error sending request") // reqwest wraps connect errors
        || lower.contains("os error 10061") // WSAECONNREFUSED on Windows
        || lower.contains("econnrefused");
    if looks_unreachable {
        format!(
            "Ollama unreachable ({raw}) — install from https://ollama.com/download and run `ollama serve`"
        )
    } else {
        format!("Ollama error: {raw}")
    }
}

/// Stream an Ollama completion as [`StreamChunk`]s.
///
/// `ollama-rs` 0.3.4 `generate_stream` returns
/// `Result<Pin<Box<dyn Stream<Item = Result<Vec<GenerationResponse>>>>>>`:
/// the outer `Result` carries the connect failure (→ one diagnostic
/// `Error`), and each polled item is a `Vec` of token-sized responses.
/// Every non-empty `response` becomes a [`StreamChunk::Delta`]; the
/// response flagged `done` becomes the terminal [`StreamChunk::Done`]
/// (cost 0 — local inference is free — and the accumulated text as the
/// result, mirroring the Claude arm). The whole arm is bounded by an
/// idle timeout.
fn ollama_stream(
    model: String,
    prompt: String,
) -> impl Stream<Item = StreamChunk> + Send + 'static {
    use ollama_rs::Ollama;
    use ollama_rs::generation::completion::request::GenerationRequest;

    let idle = ollama_idle_timeout();
    let endpoint = std::env::var("SBFB_OLLAMA_ENDPOINT")
        .ok()
        .filter(|s| !s.is_empty());

    async_stream::stream! {
        // `Ollama::default()` targets the loopback daemon (127.0.0.1:11434),
        // inside the hardened loopback boundary. `SBFB_OLLAMA_ENDPOINT`
        // overrides it (default stays loopback).
        let ollama = match endpoint {
            Some(ep) => match Ollama::try_new(ep) {
                Ok(o) => o,
                Err(e) => {
                    yield StreamChunk::Error {
                        message: format!("invalid SBFB_OLLAMA_ENDPOINT: {e}"),
                    };
                    return;
                }
            },
            None => Ollama::default(),
        };

        let request = GenerationRequest::new(model, prompt);

        let mut stream = match ollama.generate_stream(request).await {
            Ok(s) => s,
            Err(e) => {
                yield StreamChunk::Error {
                    message: ollama_diagnostic(&e.to_string()),
                };
                return;
            }
        };

        let mut accumulated = String::new();
        let mut saw_done = false;

        loop {
            match tokio::time::timeout(idle, stream.next()).await {
                Err(_elapsed) => {
                    yield StreamChunk::Error {
                        message: format!(
                            "Ollama stream timed out after {}s of inactivity",
                            idle.as_secs()
                        ),
                    };
                    return;
                }
                Ok(None) => break,
                Ok(Some(Err(e))) => {
                    yield StreamChunk::Error {
                        message: ollama_diagnostic(&e.to_string()),
                    };
                    return;
                }
                Ok(Some(Ok(responses))) => {
                    for resp in responses {
                        if !resp.response.is_empty() {
                            accumulated.push_str(&resp.response);
                            yield StreamChunk::Delta { text: resp.response };
                        }
                        if resp.done {
                            let duration_ms =
                                resp.total_duration.map(|ns| ns / 1_000_000).unwrap_or(0);
                            yield StreamChunk::Done {
                                cost_usd: 0.0,
                                duration_ms,
                                result: std::mem::take(&mut accumulated),
                            };
                            saw_done = true;
                        }
                    }
                }
            }
        }

        // The stream ended without an explicit `done` marker — emit a
        // terminal Done so the SSE consumer closes cleanly.
        if !saw_done {
            yield StreamChunk::Done {
                cost_usd: 0.0,
                duration_ms: 0,
                result: accumulated,
            };
        }
    }
}

/// Default poll cadence and global deadline for the network arm.
/// Overridable via `SBFB_NETWORK_POLL_INTERVAL_MS` /
/// `SBFB_NETWORK_TIMEOUT_SECS` (the tests drive them small).
const DEFAULT_NETWORK_POLL_INTERVAL_MS: u64 = 2000;
const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 600;

fn network_poll_interval() -> Duration {
    std::env::var("SBFB_NETWORK_POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|ms| *ms > 0)
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(DEFAULT_NETWORK_POLL_INTERVAL_MS))
}

fn network_timeout() -> Duration {
    std::env::var("SBFB_NETWORK_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|s| *s > 0)
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_NETWORK_TIMEOUT_SECS))
}

/// Resolve the daemon base URL + auth token for the network arm.
///
/// `SBFB_DAEMON_ENDPOINT` (+ optional `SBFB_DAEMON_TOKEN`) overrides the
/// discovery so tests can target a mock without a `running.json`.
/// Otherwise [`DaemonConnection::discover`] reads `running.json` +
/// `auth_token` from the standard SBFB paths (R3: the Operator and the
/// daemon share the same token file).
///
/// P2-SYNC-FS-ASYNC (S73 Phase B): this is the only async-context caller of
/// the sync `DaemonConnection::discover()` (which reads `running.json` +
/// `auth_token` via `std::fs`). `discover()` itself stays sync — it pairs
/// with the `reqwest::blocking` daemon helpers and four sync CLI call sites
/// (`pipeline.rs`, `gates.rs`, `preview_cmd.rs`) — so here we offload the
/// blocking file IO to the runtime's blocking pool instead of stalling an
/// async executor worker on the network poll path.
async fn resolve_daemon() -> Result<(String, String), String> {
    if let Ok(ep) = std::env::var("SBFB_DAEMON_ENDPOINT") {
        if !ep.is_empty() {
            let token = std::env::var("SBFB_DAEMON_TOKEN").unwrap_or_default();
            return Ok((ep.trim_end_matches('/').to_string(), token));
        }
    }
    match tokio::task::spawn_blocking(crate::daemon_client::DaemonConnection::discover).await {
        Ok(Ok(conn)) => Ok((conn.base_url, conn.token)),
        Ok(Err(e)) => Err(format!("daemon not reachable: {e}")),
        Err(e) => Err(format!("daemon discovery task failed: {e}")),
    }
}

/// Extract a string field from a (possibly nested) JSON path.
fn json_str<'a>(v: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = v;
    for key in path {
        cur = cur.get(key)?;
    }
    cur.as_str()
}

/// Submit a chat sub-task to the SBFB network via the local daemon and
/// stream its lifecycle as [`StreamChunk`]s.
///
/// Unlike the Claude/Ollama arms, the worker result is **not** streamed
/// token-by-token over the WAN (PO-14): the daemon's task API is
/// submit→poll. The arm submits a [`TaskSubmission`]-shaped body to
/// `POST /api/v1/tasks/submit`, polls `GET /api/v1/tasks/{id}` on a
/// fixed interval (emitting a `Debug` progress label per tick, never a
/// `Delta`), and on `completed` fetches the accepted text from
/// `GET /api/v1/tasks/{id}/result` (the Sprint 72 Phase D primitive) to
/// emit **exactly one** terminal [`StreamChunk::Done`]. `rejected` /
/// `timed_out` / the global timeout each yield a single
/// [`StreamChunk::Error`].
///
/// The submit body is built inline with `serde_json` (the daemon fills
/// the rest of `TaskSubmission` via serde defaults) so the Factory crate
/// stays free of a `nexus-coordinator-rs` dependency (crate isolation).
fn network_stream(
    project_id: String,
    model: String,
    prompt: String,
) -> impl Stream<Item = StreamChunk> + Send + 'static {
    let poll_interval = network_poll_interval();
    let global_timeout = network_timeout();

    async_stream::stream! {
        let (base_url, token) = match resolve_daemon().await {
            Ok(v) => v,
            Err(e) => {
                yield StreamChunk::Error { message: format!("network target: {e}") };
                return;
            }
        };

        let client = reqwest::Client::new();

        // --- submit ---
        let submit_body = serde_json::json!({
            "project_id": project_id,
            "task_type": "inference",
            "prompt": prompt,
            "model": model,
        });
        let submit = client
            .post(format!("{base_url}/api/v1/tasks/submit"))
            .header("X-SBFB-Token", &token)
            .header("Host", "127.0.0.1")
            .json(&submit_body)
            .send()
            .await;
        let task_id = match submit {
            Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
                Ok(json) => match json_str(&json, &["task", "task_id"]) {
                    Some(id) => id.to_owned(),
                    None => {
                        yield StreamChunk::Error {
                            message: "network submit: response missing task.task_id".to_owned(),
                        };
                        return;
                    }
                },
                Err(e) => {
                    yield StreamChunk::Error {
                        message: format!("network submit: invalid response: {e}"),
                    };
                    return;
                }
            },
            Ok(resp) => {
                yield StreamChunk::Error {
                    message: format!("network submit rejected: HTTP {}", resp.status()),
                };
                return;
            }
            Err(e) => {
                yield StreamChunk::Error { message: format!("network submit failed: {e}") };
                return;
            }
        };

        // --- poll until terminal or global timeout ---
        let status_url = format!("{base_url}/api/v1/tasks/{task_id}");
        let result_url = format!("{base_url}/api/v1/tasks/{task_id}/result");
        let deadline = tokio::time::Instant::now() + global_timeout;
        let mut interval = tokio::time::interval(poll_interval);
        // P2-POLL-DIAGNOSTIC-LOSS (S73 Phase B): remember the most recent
        // transient poll failure so a global timeout surfaces *why* polling
        // never succeeded (e.g. "HTTP 401") instead of a bare "timed out".
        // Token-free: the daemon token rides in the header, never the URL.
        let mut last_err: Option<String> = None;

        loop {
            interval.tick().await; // immediate on the first iteration, then every poll_interval
            if tokio::time::Instant::now() >= deadline {
                let detail = last_err
                    .as_ref()
                    .map(|e| format!(" (last error: {e})"))
                    .unwrap_or_default();
                yield StreamChunk::Error {
                    message: format!(
                        "network task {task_id} timed out after {}s{detail}",
                        global_timeout.as_secs()
                    ),
                };
                return;
            }

            let status = match client
                .get(&status_url)
                .header("X-SBFB-Token", &token)
                .header("Host", "127.0.0.1")
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
                    Ok(j) => j.get("status").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
                    // Transient parse blip — keep polling, bounded by the
                    // timeout, but remember why (P2-POLL-DIAGNOSTIC-LOSS).
                    Err(e) => {
                        last_err = Some(format!("status parse error: {e}"));
                        continue;
                    }
                },
                // Non-success HTTP status (401/404/500…) — keep polling, but
                // remember the code so a timeout is actionable.
                Ok(r) => {
                    last_err = Some(format!("status poll HTTP {}", r.status()));
                    continue;
                }
                // Transient transport error/blip — keep polling, remember it.
                Err(e) => {
                    last_err = Some(format!("status poll failed: {e}"));
                    continue;
                }
            };

            yield StreamChunk::Debug {
                label: "network-poll".to_owned(),
                content: format!("status: {status}"),
            };

            match status.as_str() {
                "completed" => {
                    match client
                        .get(&result_url)
                        .header("X-SBFB-Token", &token)
                        .header("Host", "127.0.0.1")
                        .send()
                        .await
                    {
                        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
                            Ok(j) => {
                                let text = j
                                    .get("result_text")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_owned();
                                yield StreamChunk::Done {
                                    cost_usd: 0.0,
                                    duration_ms: 0,
                                    result: text,
                                };
                                return;
                            }
                            Err(e) => {
                                yield StreamChunk::Error {
                                    message: format!("network result: invalid response: {e}"),
                                };
                                return;
                            }
                        },
                        Ok(r) => {
                            yield StreamChunk::Error {
                                message: format!("network result fetch: HTTP {}", r.status()),
                            };
                            return;
                        }
                        Err(e) => {
                            yield StreamChunk::Error {
                                message: format!("network result fetch failed: {e}"),
                            };
                            return;
                        }
                    }
                }
                "rejected" | "timed_out" => {
                    yield StreamChunk::Error {
                        message: format!("network task {task_id} {status}"),
                    };
                    return;
                }
                // pending / dispatched / awaiting_quorum → keep polling.
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    // P2-A-1 review P1 (S73 Phase B): every test below that mutates a
    // process-global env var carries `#[serial(sbfb_env)]`. nextest isolates
    // per process, but the `cargo test --workspace` gates (ci.yml,
    // verify.sh, .woodpecker) share one process per crate, so without this
    // the set_var/remove_var calls race across the parallel test threads.
    use serial_test::serial;

    #[test]
    fn execution_target_from_provider_parses_closed_set() {
        assert_eq!(
            ExecutionTarget::from_provider("claude", "m", "p"),
            ExecutionTarget::Claude { model: "m".into() }
        );
        assert_eq!(
            ExecutionTarget::from_provider("ollama", "m", "p"),
            ExecutionTarget::Ollama { model: "m".into() }
        );
        // "local" is an alias for Ollama (UX intention "run locally").
        assert_eq!(
            ExecutionTarget::from_provider("local", "m", "p"),
            ExecutionTarget::Ollama { model: "m".into() }
        );
        assert_eq!(
            ExecutionTarget::from_provider("network", "m", "proj"),
            ExecutionTarget::Network {
                project_id: "proj".into(),
                model: "m".into()
            }
        );
        // Unknown and empty fall back to Claude (default pilot, PO-14).
        assert_eq!(
            ExecutionTarget::from_provider("gpt-9", "m", "p"),
            ExecutionTarget::Claude { model: "m".into() }
        );
        assert_eq!(
            ExecutionTarget::from_provider("", "m", "p"),
            ExecutionTarget::Claude { model: "m".into() }
        );
    }

    #[tokio::test]
    async fn claude_target_is_behaviorally_unchanged() {
        // The Claude arm must delegate to `spawn_claude_stream` verbatim.
        // We consume only the deterministic pre-spawn prefix (the
        // `prompt` + `command` Debug chunks emitted BEFORE any process is
        // spawned) and drop the stream — so no real `claude` is ever
        // launched — then assert the two paths produce the identical
        // prefix.
        let cwd = std::env::temp_dir();
        let prompt = "hello".to_string();
        let model = "test-model".to_string();

        let via_target: Vec<StreamChunk> = ExecutionTarget::Claude {
            model: model.clone(),
        }
        .run(prompt.clone(), cwd.clone())
        .take(2)
        .collect()
        .await;

        let via_direct: Vec<StreamChunk> = Box::pin(spawn_claude_stream(&prompt, &model, &cwd))
            .take(2)
            .collect()
            .await;

        assert_eq!(via_target.len(), 2, "expected the prompt + command prefix");
        assert_eq!(
            format!("{via_target:?}"),
            format!("{via_direct:?}"),
            "Claude arm must delegate to spawn_claude_stream verbatim"
        );
    }

    #[test]
    fn ollama_diagnostic_flags_connection_refused() {
        let unreachable = ollama_diagnostic(
            "error sending request for url: tcp connect error: Connection refused (os error 10061)",
        );
        assert!(
            unreachable.to_lowercase().contains("unreachable")
                && unreachable.contains("ollama serve"),
            "connect failure must get an actionable hint, got: {unreachable}"
        );
        let other = ollama_diagnostic("model 'llama3.2' not found, try pulling it first");
        assert!(
            other.starts_with("Ollama error:"),
            "non-connect errors surface verbatim, got: {other}"
        );
    }

    #[tokio::test]
    #[serial(sbfb_env)]
    async fn ollama_unreachable_yields_diagnostic() {
        // Point at a port with nothing listening: the connect fails
        // inside `generate_stream` and the arm must surface a single
        // helpful diagnostic, never a silent empty stream. The env override
        // is safe under both runners: nextest isolates per process, and
        // `#[serial(sbfb_env)]` serializes it under plain `cargo test`.
        unsafe {
            std::env::set_var("SBFB_OLLAMA_ENDPOINT", "http://127.0.0.1:1");
        }
        let chunks: Vec<StreamChunk> = ExecutionTarget::Ollama {
            model: "any-model".into(),
        }
        .run("hi".into(), std::env::temp_dir())
        .collect()
        .await;
        unsafe {
            std::env::remove_var("SBFB_OLLAMA_ENDPOINT");
        }

        assert_eq!(
            chunks.len(),
            1,
            "expected exactly one diagnostic, got {chunks:?}"
        );
        assert!(
            matches!(&chunks[0], StreamChunk::Error { message } if message.to_lowercase().contains("ollama")),
            "unreachable Ollama must yield an Ollama diagnostic, got {chunks:?}"
        );
    }

    #[tokio::test]
    #[serial(sbfb_env)]
    async fn ollama_stream_maps_to_chunks_via_mock() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // A mock HTTP server replies to Ollama's /api/generate with a
        // streamed NDJSON body (3 GenerationResponse lines). This proves
        // the Delta/Done mapping DETERMINISTICALLY, with no live Ollama —
        // mirroring the worker's `execute_task_ollama_mock_*` pattern.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let _ = stream.read(&mut buf).await;
            let body = concat!(
                "{\"model\":\"m\",\"created_at\":\"2026-01-01T00:00:00Z\",\"response\":\"Hello\",\"done\":false}\n",
                "{\"model\":\"m\",\"created_at\":\"2026-01-01T00:00:00Z\",\"response\":\" world\",\"done\":false}\n",
                "{\"model\":\"m\",\"created_at\":\"2026-01-01T00:00:00Z\",\"response\":\"\",\"done\":true,\"total_duration\":2000000}\n"
            );
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes()).await;
            let _ = stream.flush().await;
        });

        unsafe {
            std::env::set_var("SBFB_OLLAMA_ENDPOINT", format!("http://127.0.0.1:{port}"));
        }
        let chunks: Vec<StreamChunk> = ExecutionTarget::Ollama { model: "m".into() }
            .run("hi".into(), std::env::temp_dir())
            .collect()
            .await;
        unsafe {
            std::env::remove_var("SBFB_OLLAMA_ENDPOINT");
        }

        let deltas: Vec<String> = chunks
            .iter()
            .filter_map(|c| match c {
                StreamChunk::Delta { text } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            deltas,
            vec!["Hello".to_string(), " world".to_string()],
            "each non-empty response must map to a Delta, got {chunks:?}"
        );
        match chunks.last() {
            Some(StreamChunk::Done {
                result,
                duration_ms,
                cost_usd,
            }) => {
                assert_eq!(result, "Hello world", "Done accumulates the streamed text");
                assert_eq!(*duration_ms, 2, "2_000_000 ns -> 2 ms");
                assert_eq!(*cost_usd, 0.0, "local inference is free");
            }
            other => panic!("expected a terminal Done, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ollama_stream_maps_to_chunks() {
        // Availability-gated (B-3 pattern): when no Ollama is reachable
        // (CI), the arm yields a single Error — accepted as a clean skip.
        // When Ollama IS up with the model, the stream must map to
        // Delta(s) terminated by a Done.
        let chunks: Vec<StreamChunk> = ExecutionTarget::Ollama {
            model: "llama3.2:latest".into(),
        }
        .run("Reply with a single word.".into(), std::env::temp_dir())
        .collect()
        .await;

        let only_error = chunks.len() == 1 && matches!(chunks[0], StreamChunk::Error { .. });
        if only_error {
            // Ollama absent or model not pulled — clean skip.
            return;
        }

        assert!(
            chunks
                .iter()
                .any(|c| matches!(c, StreamChunk::Delta { .. })),
            "a reachable Ollama must produce at least one Delta, got {chunks:?}"
        );
        assert!(
            matches!(chunks.last(), Some(StreamChunk::Done { .. })),
            "the Ollama stream must terminate with a Done, got {chunks:?}"
        );
    }

    /// A mock daemon: replies to the network arm's submit + poll + result
    /// HTTP calls deterministically, no real daemon. `complete_after_polls`
    /// status polls return `dispatched`, then `completed`. Each response
    /// carries `Connection: close` so reqwest opens a fresh connection per
    /// request and the accept loop handles one request per connection.
    async fn spawn_mock_daemon(complete_after_polls: usize) -> u16 {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let polls = Arc::new(AtomicUsize::new(0));
            while let Ok((mut stream, _)) = listener.accept().await {
                let polls = polls.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let first = req.lines().next().unwrap_or("");
                    let mut parts = first.split_whitespace();
                    let method = parts.next().unwrap_or("");
                    let path = parts.next().unwrap_or("");

                    let body = if method == "POST" && path.ends_with("/tasks/submit") {
                        r#"{"task":{"task_id":"net-task-1"}}"#.to_string()
                    } else if method == "GET" && path.ends_with("/result") {
                        r#"{"task_id":"net-task-1","status":"completed","result_text":"the network reply"}"#.to_string()
                    } else if method == "GET" && path.contains("/tasks/net-task-1") {
                        let seen = polls.fetch_add(1, Ordering::SeqCst);
                        let status = if seen >= complete_after_polls {
                            "completed"
                        } else {
                            "dispatched"
                        };
                        format!(r#"{{"task_id":"net-task-1","status":"{status}"}}"#)
                    } else {
                        r#"{"error":"unexpected"}"#.to_string()
                    };

                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                });
            }
        });

        port
    }

    // PO-14: the network arm submits → polls → and on `completed` emits
    // EXACTLY ONE terminal Done carrying the retrieved result text, never
    // a token Delta. Deterministic via the mock daemon (no live daemon).
    #[tokio::test]
    #[serial(sbfb_env)]
    async fn network_provider_submit_poll_yields_single_done() {
        let port = spawn_mock_daemon(1).await; // completes on the 2nd status poll
        unsafe {
            std::env::set_var("SBFB_DAEMON_ENDPOINT", format!("http://127.0.0.1:{port}"));
            std::env::set_var("SBFB_NETWORK_POLL_INTERVAL_MS", "20");
            std::env::set_var("SBFB_NETWORK_TIMEOUT_SECS", "10");
        }
        let chunks: Vec<StreamChunk> = ExecutionTarget::Network {
            project_id: "proj".into(),
            model: "m".into(),
        }
        .run("hi".into(), std::env::temp_dir())
        .collect()
        .await;
        unsafe {
            std::env::remove_var("SBFB_DAEMON_ENDPOINT");
            std::env::remove_var("SBFB_NETWORK_POLL_INTERVAL_MS");
            std::env::remove_var("SBFB_NETWORK_TIMEOUT_SECS");
        }

        let dones: Vec<&StreamChunk> = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::Done { .. }))
            .collect();
        let deltas = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::Delta { .. }))
            .count();
        assert_eq!(dones.len(), 1, "exactly one terminal Done, got {chunks:?}");
        assert_eq!(
            deltas, 0,
            "no token Delta over the WAN (PO-14), got {chunks:?}"
        );
        match dones[0] {
            StreamChunk::Done { result, .. } => {
                assert_eq!(result, "the network reply", "Done carries the /result text");
            }
            _ => unreachable!(),
        }
    }

    // The global timeout bounds a task that never completes: the arm
    // yields exactly one terminal Error and never a Done.
    #[tokio::test]
    #[serial(sbfb_env)]
    async fn network_provider_poll_timeout() {
        let port = spawn_mock_daemon(usize::MAX).await; // always dispatched
        unsafe {
            std::env::set_var("SBFB_DAEMON_ENDPOINT", format!("http://127.0.0.1:{port}"));
            std::env::set_var("SBFB_NETWORK_POLL_INTERVAL_MS", "20");
            std::env::set_var("SBFB_NETWORK_TIMEOUT_SECS", "1");
        }
        let chunks: Vec<StreamChunk> = ExecutionTarget::Network {
            project_id: "proj".into(),
            model: "m".into(),
        }
        .run("hi".into(), std::env::temp_dir())
        .collect()
        .await;
        unsafe {
            std::env::remove_var("SBFB_DAEMON_ENDPOINT");
            std::env::remove_var("SBFB_NETWORK_POLL_INTERVAL_MS");
            std::env::remove_var("SBFB_NETWORK_TIMEOUT_SECS");
        }

        assert!(
            chunks
                .iter()
                .all(|c| !matches!(c, StreamChunk::Done { .. })),
            "a never-completing task must not yield a Done, got {chunks:?}"
        );
        let errors: Vec<&StreamChunk> = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::Error { .. }))
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "exactly one terminal Error, got {chunks:?}"
        );
        match errors[0] {
            StreamChunk::Error { message } => {
                assert!(
                    message.contains("timed out"),
                    "timeout diagnostic, got {message}"
                );
            }
            _ => unreachable!(),
        }
    }

    // P2-POLL-DIAGNOSTIC-LOSS (S73 Phase B): when status polls keep failing
    // (e.g. HTTP 401), the global-timeout Error must surface the LAST
    // transient failure, not a bare "timed out" — so an operator can tell a
    // genuinely stuck task from an auth/endpoint misconfiguration.
    #[tokio::test]
    #[serial(sbfb_env)]
    async fn network_provider_surfaces_last_error_on_timeout() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Mock daemon: submit succeeds (200 + task_id), but every status poll
        // returns HTTP 401, so the arm never reaches a terminal status and
        // hits the global timeout carrying `last_err = HTTP 401`.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let first = req.lines().next().unwrap_or("");
                    let mut parts = first.split_whitespace();
                    let method = parts.next().unwrap_or("");
                    let path = parts.next().unwrap_or("");
                    let (status_line, body) = if method == "POST" && path.ends_with("/tasks/submit")
                    {
                        ("200 OK", r#"{"task":{"task_id":"net-task-1"}}"#.to_string())
                    } else {
                        // Every status poll fails with an auth error.
                        (
                            "401 Unauthorized",
                            r#"{"error":"unauthorized"}"#.to_string(),
                        )
                    };
                    let resp = format!(
                        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                });
            }
        });

        unsafe {
            std::env::set_var("SBFB_DAEMON_ENDPOINT", format!("http://127.0.0.1:{port}"));
            std::env::set_var("SBFB_NETWORK_POLL_INTERVAL_MS", "20");
            std::env::set_var("SBFB_NETWORK_TIMEOUT_SECS", "1");
        }
        let chunks: Vec<StreamChunk> = ExecutionTarget::Network {
            project_id: "proj".into(),
            model: "m".into(),
        }
        .run("hi".into(), std::env::temp_dir())
        .collect()
        .await;
        unsafe {
            std::env::remove_var("SBFB_DAEMON_ENDPOINT");
            std::env::remove_var("SBFB_NETWORK_POLL_INTERVAL_MS");
            std::env::remove_var("SBFB_NETWORK_TIMEOUT_SECS");
        }

        let errors: Vec<&StreamChunk> = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::Error { .. }))
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "exactly one terminal Error, got {chunks:?}"
        );
        match errors[0] {
            StreamChunk::Error { message } => {
                assert!(
                    message.contains("timed out"),
                    "still a timeout diagnostic, got {message}"
                );
                assert!(
                    message.contains("401"),
                    "timeout must surface the last poll error (HTTP 401), got {message}"
                );
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    #[serial(sbfb_env)]
    async fn resolve_daemon_uses_env_override_without_fs() {
        // P2-SYNC-FS-ASYNC (S73 Phase B): the endpoint-override branch returns
        // synchronously (no fs) and trims a trailing slash. `#[serial(sbfb_env)]`
        // keeps the env mutation safe under plain `cargo test` too.
        unsafe {
            std::env::set_var("SBFB_DAEMON_ENDPOINT", "http://127.0.0.1:9999/");
            std::env::set_var("SBFB_DAEMON_TOKEN", "tok-1");
        }
        let resolved = resolve_daemon().await;
        unsafe {
            std::env::remove_var("SBFB_DAEMON_ENDPOINT");
            std::env::remove_var("SBFB_DAEMON_TOKEN");
        }
        assert_eq!(
            resolved,
            Ok(("http://127.0.0.1:9999".to_string(), "tok-1".to_string())),
        );
    }

    #[tokio::test]
    #[serial(sbfb_env)]
    async fn resolve_daemon_discovers_off_blocking_pool_when_no_env() {
        // P2-SYNC-FS-ASYNC: with no override, discovery reads running.json via
        // spawn_blocking. Pointed at an empty root it must surface a clean error
        // (the fs read runs off the blocking pool, never panics, never blocks an
        // async executor worker).
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::remove_var("SBFB_DAEMON_ENDPOINT");
            std::env::set_var("NEXUS_GRID_ROOT", tmp.path());
        }
        let resolved = resolve_daemon().await;
        unsafe {
            std::env::remove_var("NEXUS_GRID_ROOT");
        }
        assert!(
            resolved.is_err(),
            "discovery off an empty root must surface an error, got {resolved:?}"
        );
    }
}
