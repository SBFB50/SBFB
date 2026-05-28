// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::process::Stdio;

use futures::stream::Stream;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

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
    let prompt = prompt.to_owned();
    let model = model.to_owned();
    let cwd = cwd.to_owned();

    async_stream::stream! {
        let exe = if cfg!(windows) { "claude.cmd" } else { "claude" };
        let cli_args = [
            "-p",
            "--output-format", "stream-json",
            "--include-partial-messages",
            "--model", &model,
            "--permission-mode", "bypassPermissions",
        ];

        yield StreamChunk::Debug {
            label: "prompt".into(),
            content: prompt.clone(),
        };
        yield StreamChunk::Debug {
            label: "command".into(),
            content: format!("{exe} {}", cli_args.join(" ")),
        };
        yield StreamChunk::Debug {
            label: "cwd".into(),
            content: cwd.display().to_string(),
        };

        let child = Command::new(exe)
            .args(cli_args)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                yield StreamChunk::Error {
                    message: format!("Failed to spawn claude: {e}"),
                };
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
                yield StreamChunk::Error {
                    message: "No stdout from claude process".into(),
                };
                return;
            }
        };

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut event_count: u32 = 0;

        while let Ok(Some(line)) = lines.next_line().await {
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
                    message: format!("claude exited with {s}"),
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
}
