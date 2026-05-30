// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use futures::stream::Stream;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// G12 (D6): default idle timeout. The agent stream is killed if it
/// produces no output for this long — bounds a hung subprocess
/// without truncating a legitimately-streaming long run. Overridable
/// via `SBFB_CLAUDE_IDLE_TIMEOUT_SECS`.
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 120;

fn idle_timeout() -> Duration {
    std::env::var("SBFB_CLAUDE_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS))
}

/// Resolve the agent CLI. `SBFB_CLAUDE_BIN` overrides the default so
/// operators can point at a specific binary — and so tests never
/// spawn a real `claude` agent with `bypassPermissions`.
fn claude_exe() -> String {
    if let Ok(bin) = std::env::var("SBFB_CLAUDE_BIN") {
        if !bin.is_empty() {
            return bin;
        }
    }
    if cfg!(windows) {
        "claude.cmd".to_string()
    } else {
        "claude".to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StreamChunk {
    #[serde(rename = "delta")]
    Delta { text: String },
    #[serde(rename = "thinking")]
    Thinking { text: String },
    #[serde(rename = "done")]
    Done {
        cost_usd: f64,
        duration_ms: u64,
        result: String,
    },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "debug")]
    Debug { label: String, content: String },
}

pub fn assemble_prompt(
    context: &serde_json::Value,
    messages: &[(String, String)],
    new_msg: &str,
) -> String {
    let mut prompt = String::new();

    if let Some(sprint) = context.get("sprint") {
        prompt.push_str(&format!(
            "[Context: Sprint {}, Phase {}, HEAD {}]\n\n",
            sprint,
            context
                .get("phase")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            context
                .get("head")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
        ));
    }

    if !messages.is_empty() {
        prompt.push_str("--- Conversation history ---\n");
        for (role, content) in messages {
            prompt.push_str(&format!("{role}: {content}\n"));
        }
        prompt.push_str("---\n\n");
    }

    prompt.push_str(new_msg);
    prompt
}

pub fn spawn_claude_stream(
    prompt: &str,
    model: &str,
    cwd: &Path,
) -> impl Stream<Item = StreamChunk> + 'static {
    let exe = claude_exe();
    let mut command = Command::new(&exe);
    command
        .args([
            "-p",
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--model",
            model,
            "--permission-mode",
            "bypassPermissions",
        ])
        .current_dir(cwd);
    let command_label = format!(
        "{exe} -p --output-format stream-json --include-partial-messages \
         --model {model} --permission-mode bypassPermissions"
    );
    spawn_agent_stream(
        command,
        exe,
        command_label,
        prompt.to_owned(),
        idle_timeout(),
    )
}

/// Run an agent subprocess, streaming its stdout as [`StreamChunk`]s.
///
/// G12 (D6): bounded by an **idle timeout** — if the child produces
/// no output line for `idle`, it is killed (`start_kill` + reaped via
/// `wait` to avoid a zombie) and a timeout error is yielded. A
/// missing executable yields a clear diagnostic instead of an opaque
/// `Failed to spawn`. `kill_on_drop` is a safety net if the stream is
/// dropped mid-flight.
///
/// Factored out of [`spawn_claude_stream`] so the timeout / kill /
/// diagnostic mechanics are unit-testable with an arbitrary command.
pub(crate) fn spawn_agent_stream(
    mut command: Command,
    exe: String,
    command_label: String,
    prompt: String,
    idle: Duration,
) -> impl Stream<Item = StreamChunk> + 'static {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    async_stream::stream! {
        yield StreamChunk::Debug {
            label: "prompt".into(),
            content: prompt.clone(),
        };
        yield StreamChunk::Debug {
            label: "command".into(),
            content: command_label.clone(),
        };

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                let message = if e.kind() == std::io::ErrorKind::NotFound {
                    format!(
                        "agent CLI `{exe}` not found on PATH — install the Claude CLI \
                         (npm i -g @anthropic-ai/claude-code) or set SBFB_CLAUDE_BIN to its full path"
                    )
                } else {
                    format!("failed to spawn `{exe}`: {e}")
                };
                yield StreamChunk::Error { message };
                return;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prompt.as_bytes()).await;
            let _ = stdin.shutdown().await;
        }

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                yield StreamChunk::Error {
                    message: "no stdout from agent process".into(),
                };
                return;
            }
        };

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut event_count: u32 = 0;

        loop {
            let line = match tokio::time::timeout(idle, lines.next_line()).await {
                Err(_elapsed) => {
                    let _ = child.start_kill();
                    let _ = child.wait().await;
                    yield StreamChunk::Error {
                        message: format!(
                            "agent timed out after {}s of inactivity — process killed",
                            idle.as_secs()
                        ),
                    };
                    return;
                }
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => break,
                Ok(Err(_)) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            event_count += 1;
            yield StreamChunk::Debug {
                label: format!("ndjson#{event_count}"),
                content: line.clone(),
            };

            let parsed: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match msg_type {
                "stream_event" => {
                    if let Some(event) = parsed.get("event") {
                        let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if event_type == "content_block_delta" {
                            if let Some(delta) = event.get("delta") {
                                let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                if delta_type == "text_delta" {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        yield StreamChunk::Delta { text: text.to_owned() };
                                    }
                                } else if delta_type == "thinking_delta" {
                                    if let Some(text) = delta.get("thinking").and_then(|v| v.as_str()) {
                                        yield StreamChunk::Thinking { text: text.to_owned() };
                                    }
                                }
                            }
                        }
                    }
                }
                "result" => {
                    let cost = parsed
                        .get("total_cost_usd")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let duration = parsed
                        .get("duration_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let result_text = parsed
                        .get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned();
                    yield StreamChunk::Done {
                        cost_usd: cost,
                        duration_ms: duration,
                        result: result_text,
                    };
                }
                _ => {}
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let mut stderr_reader = BufReader::new(stderr);
            let mut stderr_buf = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr_reader, &mut stderr_buf).await;
            if !stderr_buf.trim().is_empty() {
                yield StreamChunk::Debug {
                    label: "stderr".into(),
                    content: stderr_buf,
                };
            }
        }

        let status = child.wait().await;
        if let Ok(s) = status {
            yield StreamChunk::Debug {
                label: "exit".into(),
                content: format!("{s}"),
            };
            if !s.success() {
                yield StreamChunk::Error {
                    message: format!("agent exited with {s}"),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_prompt_empty_history() {
        let ctx = serde_json::json!({"sprint": 70, "phase": "done", "head": "abc1234"});
        let result = assemble_prompt(&ctx, &[], "hello");
        assert!(result.contains("Sprint 70"));
        assert!(result.contains("hello"));
        assert!(!result.contains("Conversation history"));
    }

    #[test]
    fn test_assemble_prompt_with_history() {
        let ctx = serde_json::json!({"sprint": 71, "phase": "A", "head": "def5678"});
        let history = vec![
            ("user".to_string(), "what sprint?".to_string()),
            ("assistant".to_string(), "Sprint 71".to_string()),
        ];
        let result = assemble_prompt(&ctx, &history, "continue");
        assert!(result.contains("Sprint 71"));
        assert!(result.contains("Conversation history"));
        assert!(result.contains("user: what sprint?"));
        assert!(result.contains("continue"));
    }

    #[test]
    fn test_assemble_prompt_no_context() {
        let ctx = serde_json::json!({});
        let result = assemble_prompt(&ctx, &[], "test");
        assert_eq!(result, "test");
    }

    // G12: a missing agent CLI yields a clear diagnostic, not an
    // opaque "Failed to spawn".
    #[tokio::test]
    async fn missing_claude_diagnostic() {
        use futures::StreamExt;
        let exe = "sbfb-claude-does-not-exist-xyz";
        let command = Command::new(exe);
        let chunks: Vec<StreamChunk> = spawn_agent_stream(
            command,
            exe.to_string(),
            format!("{exe} -p"),
            "hi".to_string(),
            Duration::from_secs(5),
        )
        .collect()
        .await;

        let diagnostic = chunks.iter().any(|c| {
            matches!(c, StreamChunk::Error { message }
                if message.contains("not found") && message.contains("PATH"))
        });
        assert!(
            diagnostic,
            "expected a not-found diagnostic, got {chunks:?}"
        );
    }

    // G12: a subprocess that produces no output is killed once the
    // idle timeout elapses, and the stream returns a bounded error.
    #[tokio::test]
    async fn spawn_times_out() {
        use futures::StreamExt;

        // A silent, long-running, single-process command per OS.
        let command = if cfg!(windows) {
            let mut c = Command::new("waitfor");
            c.args(["/t", "30", "SbfbPhaseCTimeoutProbe"]);
            c
        } else {
            let mut c = Command::new("sleep");
            c.arg("30");
            c
        };

        let chunks: Vec<StreamChunk> = spawn_agent_stream(
            command,
            "sleeper".to_string(),
            "sleeper".to_string(),
            String::new(),
            Duration::from_millis(200),
        )
        .collect()
        .await;

        let timed_out = chunks
            .iter()
            .any(|c| matches!(c, StreamChunk::Error { message } if message.contains("timed out")));
        assert!(timed_out, "expected an idle timeout error, got {chunks:?}");
    }
}
