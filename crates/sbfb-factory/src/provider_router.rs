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
    /// The SBFB network via submit→poll against the local daemon.
    /// Wiring lands in Sprint 72 Phase D.
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
            // Network arm lands in Phase D. Until then, surface a clear
            // diagnostic rather than silently producing nothing.
            ExecutionTarget::Network { .. } => Box::pin(network_not_implemented()),
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

/// Placeholder for the network execution target until Phase D wires the
/// submit→poll client.
fn network_not_implemented() -> impl Stream<Item = StreamChunk> + Send + 'static {
    async_stream::stream! {
        yield StreamChunk::Error {
            message: "network execution target is not implemented yet (Sprint 72 Phase D)".to_owned(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

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
    async fn ollama_unreachable_yields_diagnostic() {
        // Point at a port with nothing listening: the connect fails
        // inside `generate_stream` and the arm must surface a single
        // helpful diagnostic, never a silent empty stream. nextest runs
        // each test in its own process, so this env override is isolated.
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

    #[tokio::test]
    async fn network_target_reports_not_implemented() {
        let chunks: Vec<StreamChunk> = ExecutionTarget::Network {
            project_id: "proj".into(),
            model: "m".into(),
        }
        .run("hi".into(), std::env::temp_dir())
        .collect()
        .await;
        assert_eq!(chunks.len(), 1);
        assert!(
            matches!(&chunks[0], StreamChunk::Error { message } if message.contains("Phase D")),
            "network arm must signal it is not implemented yet, got {chunks:?}"
        );
    }
}
