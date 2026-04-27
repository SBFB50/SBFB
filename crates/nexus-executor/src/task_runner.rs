// SPDX-License-Identifier: AGPL-3.0-or-later

use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::Ollama;

use crate::ipc::{TaskExecuteParams, TaskExecuteResult};

pub async fn execute_task(
    params: &TaskExecuteParams,
    ollama: Option<&Ollama>,
) -> Result<TaskExecuteResult, String> {
    let Some(client) = ollama else {
        return Ok(stub_result(params));
    };

    let opts = GenerationOptions::default().num_predict(params.max_tokens as i32);
    let req = GenerationRequest::new(params.model.clone(), params.prompt.clone()).options(opts);
    let resp = client.generate(req).await.map_err(|e| e.to_string())?;

    Ok(TaskExecuteResult {
        task_id: params.task_id.clone(),
        output: resp.response,
        output_token_ids: Vec::new(),
        model_used: resp.model,
        duration_ms: resp.total_duration.map(|ns| ns / 1_000_000).unwrap_or(0),
        gpu_vram_peak_mb: 0,
    })
}

fn stub_result(params: &TaskExecuteParams) -> TaskExecuteResult {
    TaskExecuteResult {
        task_id: params.task_id.clone(),
        output: String::new(),
        output_token_ids: Vec::new(),
        model_used: params.model.clone(),
        duration_ms: 0,
        gpu_vram_peak_mb: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> TaskExecuteParams {
        TaskExecuteParams {
            task_id: "test-123".into(),
            model: "llama3:8b".into(),
            prompt: "hello world".into(),
            watermark_config: None,
            grammar: None,
            max_tokens: 100,
            task_token: "tok".into(),
        }
    }

    #[tokio::test]
    async fn execute_task_stub_mode_returns_empty() {
        let result = execute_task(&test_params(), None).await.unwrap();
        assert_eq!(result.task_id, "test-123");
        assert!(result.output.is_empty());
        assert_eq!(result.model_used, "llama3:8b");
        assert_eq!(result.duration_ms, 0);
        assert_eq!(result.gpu_vram_peak_mb, 0);
    }

    #[tokio::test]
    async fn execute_task_ollama_mock_maps_response() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let _ = stream.read(&mut buf).await;

            let body = r#"{"model":"mock-7b","created_at":"2026-01-01T00:00:00Z","response":"42 is the answer","done":true,"total_duration":2000000000,"eval_count":10,"prompt_eval_count":5}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let ollama = Ollama::new("http://127.0.0.1", port);
        let result = execute_task(&test_params(), Some(&ollama)).await.unwrap();
        assert_eq!(result.output, "42 is the answer");
        assert_eq!(result.model_used, "mock-7b");
        assert_eq!(result.duration_ms, 2000);
    }

    #[tokio::test]
    async fn execute_task_ollama_mock_respects_max_tokens() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 16384];
            let n = stream.read(&mut buf).await.unwrap();
            let raw = String::from_utf8_lossy(&buf[..n]).to_string();
            let body_start = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
            let _ = tx.send(raw[body_start..].to_string());

            let resp_body = r#"{"model":"m","created_at":"2026-01-01T00:00:00Z","response":"ok","done":true,"total_duration":1000000}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                resp_body.len(),
                resp_body
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
        });

        let ollama = Ollama::new("http://127.0.0.1", port);
        let mut params = test_params();
        params.max_tokens = 256;
        let _ = execute_task(&params, Some(&ollama)).await.unwrap();

        let sent_body = rx.await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&sent_body).unwrap();
        assert_eq!(json["options"]["num_predict"], 256);
    }

    #[tokio::test]
    async fn execute_task_error_when_unreachable() {
        let ollama = Ollama::new("http://127.0.0.1", 1);
        let result = execute_task(&test_params(), Some(&ollama)).await;
        assert!(result.is_err());
    }
}
