// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cold-start benchmark: measures Ollama 7B inference latency on the
//! local GPU. Prerequisite for broker/executor split Phase C — the
//! total pipeline (spawn + IPC + inference) must stay under 5 seconds.
//!
//! IPC overhead is negligible per research: UDS ~4.8µs, Named Pipe
//! ~28.5µs (3tilley benchmark 2025). This bench measures the dominant
//! factor: Ollama model load (warm cache) + first token generation.
//!
//! Run: `cargo bench -p nexus-executor`
//! Requires: `ollama serve` running + `ollama pull llama3.1:8b`

use std::time::Instant;

const OLLAMA_URL: &str = "http://127.0.0.1:11434/api/generate";
const DEFAULT_MODEL: &str = "llama3.1:8b";
const COLD_START_BUDGET_MS: u128 = 5000;

#[tokio::main]
async fn main() {
    let model = std::env::var("SBFB_BENCH_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());

    println!("=== SBFB Cold-Start Benchmark ===");
    println!("Target: {model} on local GPU, budget < {COLD_START_BUDGET_MS}ms");
    println!("(override model via SBFB_BENCH_MODEL env var)");
    println!();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("HTTP client");

    // Probe Ollama availability
    let probe = client.get("http://127.0.0.1:11434/api/tags").send().await;
    if probe.is_err() {
        println!("SKIP: Ollama not running at 127.0.0.1:11434");
        println!("Start with: ollama serve");
        return;
    }

    let body = serde_json::json!({
        "model": model,
        "prompt": "Say hello in one word.",
        "stream": false,
        "options": {
            "num_predict": 8
        }
    });

    // Measure cold-start: request → complete response
    let t0 = Instant::now();
    let resp = client.post(OLLAMA_URL).json(&body).send().await;
    let elapsed = t0.elapsed();

    match resp {
        Ok(r) if r.status().is_success() => {
            let elapsed_ms = elapsed.as_millis();
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let response_text = json["response"].as_str().unwrap_or("(empty)");

            // Ollama returns timing in the response
            let load_ns = json["load_duration"].as_u64().unwrap_or(0);
            let eval_ns = json["eval_duration"].as_u64().unwrap_or(0);
            let prompt_ns = json["prompt_eval_duration"].as_u64().unwrap_or(0);

            println!("Response: {response_text}");
            println!();
            println!("--- Timing ---");
            println!("  Total wall clock:   {elapsed_ms}ms");
            println!("  Model load:         {}ms", load_ns / 1_000_000);
            println!("  Prompt eval:        {}ms", prompt_ns / 1_000_000);
            println!("  Token generation:   {}ms", eval_ns / 1_000_000);
            println!("  IPC overhead (est): <1ms (Named Pipe ~28µs)");
            println!();

            if elapsed_ms <= COLD_START_BUDGET_MS {
                println!("PASS: {elapsed_ms}ms <= {COLD_START_BUDGET_MS}ms budget");
            } else {
                println!("FAIL: {elapsed_ms}ms > {COLD_START_BUDGET_MS}ms budget");
                std::process::exit(1);
            }
        }
        Ok(r) => {
            println!("FAIL: Ollama returned {}", r.status());
            println!("Ensure model is pulled: ollama pull {model}");
            std::process::exit(1);
        }
        Err(e) => {
            println!("FAIL: request error: {e}");
            std::process::exit(1);
        }
    }
}
