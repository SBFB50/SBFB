// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::{BufRead, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Fixed 64-hex token injected via `SBFB_AUTH_TOKEN` so the harness
/// can authenticate without reading the developer's real `~/.sbfb`.
const TEST_TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const AUTH_HEADER: &str = "x-sbfb-token";

fn factory_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sbfb-factory"))
}

/// Client read timeout for the operator HTTP harness.
///
/// P2-OPERATOR-TIMEOUT (S73 Phase B): several handlers shell git
/// (`git diff`/`git log` for `/api/sprint-history*` and `/api/audit/*`),
/// which is slow on native Windows under parallel test load — the former
/// hardcoded 5s was too tight and flaked `operator_sprint_history_endpoint`
/// (passes in isolation). The generous 30s default absorbs that latency
/// without serialising the suite (so the canonical CI Linux nextest run
/// stays fast and parallel); `SBFB_TEST_HTTP_TIMEOUT_SECS` lets a slow box
/// tune it without a code change. This is a too-tight-timeout fix, not a
/// hang mask: a genuinely stuck handler still fails, just after 30s.
fn client_timeout() -> Duration {
    std::env::var("SBFB_TEST_HTTP_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(30))
}

struct TestServer {
    child: std::process::Child,
    port: u16,
    // Sandboxes `~/.sbfb` and keeps the dir alive for the server's
    // lifetime. Held only for the Drop side effect.
    _home: tempfile::TempDir,
}

impl TestServer {
    fn start() -> Self {
        // Default: point the Ollama arm at a dead port so provider-routing
        // tests are deterministic (a quick "unreachable" diagnostic) and
        // never depend on a real local Ollama.
        Self::start_inner("http://127.0.0.1:1")
    }

    /// Like [`start`] but points the Ollama arm at a live mock endpoint, so a
    /// test can observe the model the operator actually sends to Ollama
    /// (P2-OLLAMA-MODEL-PICKER, S73 Phase B).
    fn start_with_ollama(endpoint: &str) -> Self {
        Self::start_inner(endpoint)
    }

    fn start_inner(ollama_endpoint: &str) -> Self {
        let home = tempfile::tempdir().expect("tempdir");
        let mut child = factory_bin()
            .args(["operator", "serve", "--port", "0"])
            .env("SBFB_AUTH_TOKEN", TEST_TOKEN)
            .env("SBFB_HOME", home.path())
            // Point the agent spawn at a non-existent binary so no
            // test ever launches a real `claude` bypassPermissions
            // agent; the SSE paths fail fast with a diagnostic.
            .env("SBFB_CLAUDE_BIN", "sbfb-claude-test-nonexistent")
            // Sprint 72 Phase D: the Claude arm is unaffected.
            .env("SBFB_OLLAMA_ENDPOINT", ollama_endpoint)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start server");

        let stdout = child.stdout.take().expect("stdout");
        let reader = std::io::BufReader::new(stdout);

        let mut port = 0u16;
        for line in reader.lines() {
            let line = line.expect("read line");
            if let Some(addr) = line.strip_prefix("READY ") {
                if let Some(p) = addr.rsplit(':').next() {
                    port = p.parse().expect("parse port");
                }
                break;
            }
        }
        assert!(port > 0, "server should print READY with port");
        Self {
            child,
            port,
            _home: home,
        }
    }

    fn get(&self, path: &str) -> reqwest::blocking::Response {
        reqwest::blocking::Client::new()
            .get(format!("http://127.0.0.1:{}{path}", self.port))
            .header(AUTH_HEADER, TEST_TOKEN)
            .timeout(client_timeout())
            .send()
            .expect("request failed")
    }

    fn post_json(&self, path: &str, body: serde_json::Value) -> reqwest::blocking::Response {
        reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{}{path}", self.port))
            .header(AUTH_HEADER, TEST_TOKEN)
            .json(&body)
            .timeout(client_timeout())
            .send()
            .expect("request failed")
    }

    /// Raw HTTP/1.1 GET so a test controls Host / Origin / token
    /// headers exactly (reqwest derives Host from the URL). Returns
    /// the full response text including the status line.
    fn raw_get(&self, path: &str, extra_headers: &str) -> String {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", self.port)).expect("connect");
        stream
            .set_read_timeout(Some(client_timeout()))
            .expect("set timeout");
        let req = format!("GET {path} HTTP/1.1\r\n{extra_headers}Connection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).expect("write");
        let mut buf = String::new();
        let _ = stream.read_to_string(&mut buf);
        buf
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn operator_once_smoke() {
    let home = tempfile::tempdir().expect("tempdir");
    let output = factory_bin()
        .args(["operator", "serve", "--port", "0", "--once-smoke"])
        .env("SBFB_AUTH_TOKEN", TEST_TOKEN)
        .env("SBFB_HOME", home.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "once-smoke should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("READY"), "should print READY");
    assert!(stdout.contains("smoke: /api/status OK"), "smoke OK");
}

// G7 (D5): every route requires the bearer token + a loopback Host;
// a non-loopback Origin is rejected. Raw HTTP so the headers are
// fully controlled.

#[test]
fn server_rejects_missing_token() {
    let server = TestServer::start();
    let resp = server.raw_get("/api/status", "Host: 127.0.0.1\r\n");
    assert!(
        resp.starts_with("HTTP/1.1 401"),
        "missing token must be 401, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn server_rejects_foreign_host() {
    let server = TestServer::start();
    let resp = server.raw_get(
        "/api/status",
        &format!("Host: evil.com\r\n{AUTH_HEADER}: {TEST_TOKEN}\r\n"),
    );
    assert!(
        resp.starts_with("HTTP/1.1 403"),
        "foreign Host must be 403, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn cors_restricts_origin() {
    let server = TestServer::start();
    let resp = server.raw_get(
        "/api/status",
        &format!("Host: 127.0.0.1\r\nOrigin: http://evil.com\r\n{AUTH_HEADER}: {TEST_TOKEN}\r\n"),
    );
    assert!(
        resp.starts_with("HTTP/1.1 403"),
        "foreign Origin must be 403, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn token_request_succeeds() {
    let server = TestServer::start();
    // `/api/providers` is always 200 (independent of active sprint).
    let resp = server.raw_get(
        "/api/providers",
        &format!("Host: 127.0.0.1\r\n{AUTH_HEADER}: {TEST_TOKEN}\r\n"),
    );
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "valid token + loopback Host must be 200, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

// Sprint 80 Phase A: cookie-authenticated bootstrap (HttpOnly +
// SameSite=Strict + `Sec-Fetch-Site: same-origin` cross-port guard).

/// Drive the public bootstrap with a valid `?token` and return the
/// session-secret cookie value set on the 303 response.
fn bootstrap_cookie(server: &TestServer) -> String {
    let resp = server.raw_get(&format!("/?token={TEST_TOKEN}"), "Host: 127.0.0.1\r\n");
    assert!(
        resp.starts_with("HTTP/1.1 303"),
        "bootstrap must 303, got: {}",
        resp.lines().next().unwrap_or("")
    );
    let needle = "sbfb_operator=";
    let start = resp.find(needle).expect("set-cookie sbfb_operator present") + needle.len();
    let rest = &resp[start..];
    let end = rest.find(';').unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

#[test]
fn bootstrap_valid_token_sets_cookie_and_303() {
    let server = TestServer::start();
    let resp = server.raw_get(&format!("/?token={TEST_TOKEN}"), "Host: 127.0.0.1\r\n");
    let lower = resp.to_lowercase();
    assert!(
        resp.starts_with("HTTP/1.1 303"),
        "valid ?token must 303, got: {}",
        resp.lines().next().unwrap_or("")
    );
    assert!(
        lower.contains("set-cookie: sbfb_operator="),
        "must set cookie"
    );
    assert!(lower.contains("httponly"), "cookie must be HttpOnly");
    assert!(
        lower.contains("samesite=strict"),
        "cookie must be SameSite=Strict"
    );
    assert!(lower.contains("path=/"), "cookie must be Path=/");
    assert!(!lower.contains("secure"), "no Secure on loopback http");
    assert!(lower.contains("location: /"), "must redirect to /");
    assert!(
        lower.contains("referrer-policy: no-referrer"),
        "no-referrer set"
    );
    // P1-B: the cookie value must NOT be the bearer token.
    let cookie = bootstrap_cookie(&server);
    assert_ne!(
        cookie, TEST_TOKEN,
        "cookie must carry a distinct session secret"
    );
    assert_eq!(cookie.len(), 64, "session secret is 64 hex chars");
}

#[test]
fn bootstrap_invalid_token_no_cookie() {
    let server = TestServer::start();
    let resp = server.raw_get(
        "/?token=00000000000000000000000000000000000000000000000000000000deadbeef",
        "Host: 127.0.0.1\r\n",
    );
    assert!(
        !resp.starts_with("HTTP/1.1 303"),
        "wrong token must not redirect, got: {}",
        resp.lines().next().unwrap_or("")
    );
    assert!(
        !resp.to_lowercase().contains("set-cookie"),
        "wrong token must not set a cookie (no oracle)"
    );
}

#[test]
fn bootstrap_rejects_non_loopback_host() {
    let server = TestServer::start();
    // The public bootstrap bypasses `auth_required`, so it must re-do
    // its own loopback Host check (anti DNS-rebinding). A foreign Host
    // is 403 even with a valid `?token`. The CSP middleware is layered
    // OUTER of the merge, so even this 403 carries the self-origin CSP.
    let resp = server.raw_get(&format!("/?token={TEST_TOKEN}"), "Host: evil.com\r\n");
    assert!(
        resp.starts_with("HTTP/1.1 403"),
        "non-loopback Host on bootstrap must be 403, got: {}",
        resp.lines().next().unwrap_or("")
    );
    assert!(
        !resp.to_lowercase().contains("set-cookie"),
        "rejected bootstrap must not set a cookie"
    );
    assert!(
        resp.to_lowercase()
            .contains("content-security-policy: default-src 'self'"),
        "CSP must be present even on a 403 (middleware is OUTER)"
    );
}

#[test]
fn cookie_auth_succeeds_with_sec_fetch_site() {
    let server = TestServer::start();
    let cookie = bootstrap_cookie(&server);
    // No bearer header — only the cookie + the same-origin discriminant.
    let resp = server.raw_get(
        "/api/providers",
        &format!(
            "Host: 127.0.0.1\r\nCookie: sbfb_operator={cookie}\r\nSec-Fetch-Site: same-origin\r\n"
        ),
    );
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "cookie + Sec-Fetch-Site must authenticate, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn cookie_auth_rejected_without_sec_fetch_site() {
    let server = TestServer::start();
    let cookie = bootstrap_cookie(&server);
    // Cross-port CSRF guard: a valid cookie WITHOUT the forbidden
    // `Sec-Fetch-Site` header (a forged cross-port request) is rejected.
    let resp = server.raw_get(
        "/api/providers",
        &format!("Host: 127.0.0.1\r\nCookie: sbfb_operator={cookie}\r\n"),
    );
    assert!(
        resp.starts_with("HTTP/1.1 401"),
        "cookie without Sec-Fetch-Site must be 401, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn cookie_auth_rejected_with_wrong_value() {
    let server = TestServer::start();
    let resp = server.raw_get(
        "/api/providers",
        "Host: 127.0.0.1\r\nCookie: sbfb_operator=not-the-secret\r\nSec-Fetch-Site: same-origin\r\n",
    );
    assert!(
        resp.starts_with("HTTP/1.1 401"),
        "wrong cookie value must be 401, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn header_wins_over_bad_cookie() {
    let server = TestServer::start();
    // Header-first: a valid bearer header authenticates even with a
    // garbage cookie and no Sec-Fetch-Site (the CLI/Vite path).
    let resp = server.raw_get(
        "/api/providers",
        &format!(
            "Host: 127.0.0.1\r\n{AUTH_HEADER}: {TEST_TOKEN}\r\nCookie: sbfb_operator=garbage\r\n"
        ),
    );
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "valid header must win regardless of cookie, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn operator_csp_header_present() {
    let server = TestServer::start();
    let resp = server.raw_get(
        "/api/providers",
        &format!("Host: 127.0.0.1\r\n{AUTH_HEADER}: {TEST_TOKEN}\r\n"),
    );
    let lower = resp.to_lowercase();
    assert!(
        lower.contains("content-security-policy: default-src 'self'"),
        "self-origin CSP must be present"
    );
    assert!(
        lower.contains("connect-src 'self'"),
        "connect-src 'self' for SSE/ws must be present"
    );
}

// G2 (D3): the SSE chat-stream gates sensitive actions before
// spawning a bypassPermissions agent.

#[test]
fn sse_gates_sensitive_action() {
    let server = TestServer::start();
    let session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "claude"}),
        )
        .json()
        .unwrap();
    let id = session["id"].as_str().unwrap();

    // Inject a sensitive last user message via /send (it gates and
    // still records the message).
    server.post_json(
        &format!("/api/chat/{id}/send"),
        serde_json::json!({"message": "please commit and push my changes"}),
    );

    let body = server
        .get(&format!("/api/chat/{id}/stream"))
        .text()
        .unwrap();
    assert!(
        body.contains("requires_gate"),
        "sensitive SSE must gate, got: {body}"
    );
    assert!(
        !body.contains("--permission-mode"),
        "gated SSE must not assemble an agent spawn command, got: {body}"
    );
}

#[test]
fn sse_allows_nonsensitive() {
    let server = TestServer::start();
    let session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "claude"}),
        )
        .json()
        .unwrap();
    let id = session["id"].as_str().unwrap();

    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "what is the current phase?"}),
    );

    let body = server
        .get(&format!("/api/chat/{id}/stream"))
        .text()
        .unwrap();
    assert!(
        !body.contains("requires_gate"),
        "benign SSE must not gate, got: {body}"
    );
    // Proceeded to the spawn attempt (no real claude — the bin is
    // overridden to a non-existent path), proving the happy-path is
    // not gated.
    assert!(
        body.contains("not found"),
        "benign SSE should attempt the agent spawn, got: {body}"
    );
}

#[test]
fn chat_stream_uses_opus_model() {
    let server = TestServer::start();
    let session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "claude"}),
        )
        .json()
        .unwrap();
    let id = session["id"].as_str().unwrap();

    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "hello"}),
    );

    let body = server
        .get(&format!("/api/chat/{id}/stream"))
        .text()
        .unwrap();
    assert!(
        body.contains("claude-opus-4-8[1m]"),
        "stream must spawn with the opus-4-8 model, got: {body}"
    );
    assert!(
        !body.contains("--model sonnet"),
        "stream must not use the prior hardcoded sonnet model, got: {body}"
    );
}

// Sprint 72 Phase D: the SSE stream routes by the session's execution
// target. `--permission-mode` (the Claude agent command label) is the
// unique fingerprint of the Claude arm; the Ollama/Network arms never
// emit it. So its presence/absence proves which arm ran.
const CLAUDE_ARM_FINGERPRINT: &str = "--permission-mode";

#[test]
fn chat_stream_routes_by_session_provider() {
    let server = TestServer::start();

    // Session created with provider "ollama" → the stream routes to the
    // Ollama arm (dead endpoint → quick diagnostic), NOT Claude.
    let ollama_session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "ollama"}),
        )
        .json()
        .unwrap();
    let ollama_id = ollama_session["id"].as_str().unwrap();
    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": ollama_id, "message": "hello"}),
    );
    let ollama_body = server
        .get(&format!("/api/chat/{ollama_id}/stream"))
        .text()
        .unwrap();
    assert!(
        !ollama_body.contains(CLAUDE_ARM_FINGERPRINT),
        "provider=ollama must NOT run the Claude arm, got: {ollama_body}"
    );
    assert!(
        ollama_body.to_lowercase().contains("ollama"),
        "provider=ollama must run the Ollama arm, got: {ollama_body}"
    );

    // Session created with provider "claude" (default pilot) → Claude arm.
    let claude_session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "claude"}),
        )
        .json()
        .unwrap();
    let claude_id = claude_session["id"].as_str().unwrap();
    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": claude_id, "message": "hello"}),
    );
    let claude_body = server
        .get(&format!("/api/chat/{claude_id}/stream"))
        .text()
        .unwrap();
    assert!(
        claude_body.contains(CLAUDE_ARM_FINGERPRINT),
        "provider=claude must run the Claude arm, got: {claude_body}"
    );
}

#[test]
fn chat_session_persists_provider() {
    let server = TestServer::start();

    // Session created with the default provider (claude); the per-send
    // override to "ollama" must persist into the session (symmetry with
    // `model`) so the bodyless SSE GET routes to Ollama, not Claude.
    let session: serde_json::Value = server
        .post_json("/api/chat/session", serde_json::json!({}))
        .json()
        .unwrap();
    let id = session["id"].as_str().unwrap();

    server.post_json(
        &format!("/api/chat/{id}/send"),
        serde_json::json!({"message": "hello", "provider": "ollama"}),
    );

    let body = server
        .get(&format!("/api/chat/{id}/stream"))
        .text()
        .unwrap();
    assert!(
        !body.contains(CLAUDE_ARM_FINGERPRINT),
        "a /send provider override to ollama must persist and route away from Claude, got: {body}"
    );
    assert!(
        body.to_lowercase().contains("ollama"),
        "the persisted ollama provider must route to the Ollama arm, got: {body}"
    );
}

#[test]
fn sensitive_action_gated_regardless_of_provider() {
    let server = TestServer::start();

    // The SENSITIVE_ACTIONS gate runs BEFORE provider dispatch, so it
    // fires for every execution target — a sensitive message never
    // reaches the Ollama or Network arm.
    for provider in ["ollama", "network"] {
        let session: serde_json::Value = server
            .post_json(
                "/api/chat/session",
                serde_json::json!({"provider": provider}),
            )
            .json()
            .unwrap();
        let id = session["id"].as_str().unwrap();

        server.post_json(
            &format!("/api/chat/{id}/send"),
            serde_json::json!({"message": "please commit and push", "provider": provider}),
        );

        let body = server
            .get(&format!("/api/chat/{id}/stream"))
            .text()
            .unwrap();
        assert!(
            body.contains("requires_gate"),
            "provider={provider}: sensitive action must gate, got: {body}"
        );
        assert!(
            !body.contains(CLAUDE_ARM_FINGERPRINT),
            "provider={provider}: gated stream must not dispatch to any arm, got: {body}"
        );
    }
}

// P2-OLLAMA-MODEL-PICKER (S73 Phase B): a non-Claude intention must run with
// the SELECTED model, never the Claude id `claude-opus-4-8[1m]` (which does
// not exist in a local Ollama and made every non-Claude turn fail).
// End-to-end proof: a mock Ollama captures the `model` the operator sends to
// `/api/generate`.
#[test]
fn non_claude_intent_uses_selected_model() {
    use std::sync::{Arc, Mutex};

    // Blocking mock Ollama: capture the `model` from the first
    // `/api/generate` request body, then reply with a minimal valid NDJSON
    // `done` line so the arm terminates cleanly.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock ollama");
    let port = listener.local_addr().unwrap().port();
    let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_w = captured.clone();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
            // Read until the body carrying `"model"` is buffered.
            let mut raw = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        raw.extend_from_slice(&chunk[..n]);
                        if String::from_utf8_lossy(&raw).contains("\"model\"") || raw.len() > 65536
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let text = String::from_utf8_lossy(&raw);
            // Crude extraction: "model" : "VALUE".
            if let Some(idx) = text.find("\"model\"") {
                let after = &text[idx + "\"model\"".len()..];
                if let Some(colon) = after.find(':') {
                    let rest = &after[colon + 1..];
                    if let Some(q1) = rest.find('"')
                        && let Some(q2) = rest[q1 + 1..].find('"')
                    {
                        *captured_w.lock().unwrap() = Some(rest[q1 + 1..q1 + 1 + q2].to_string());
                    }
                }
            }
            let body = "{\"model\":\"m\",\"created_at\":\"2026-01-01T00:00:00Z\",\"response\":\"ok\",\"done\":true,\"total_duration\":1000000}\n";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });

    let server = TestServer::start_with_ollama(&format!("http://127.0.0.1:{port}"));
    let session: serde_json::Value = server
        .post_json(
            "/api/chat/session",
            serde_json::json!({"provider": "ollama"}),
        )
        .json()
        .unwrap();
    let id = session["id"].as_str().unwrap();

    // The model-picker payload: a non-Claude intention selects a concrete
    // model that is NOT the Claude default.
    server.post_json(
        &format!("/api/chat/{id}/send"),
        serde_json::json!({"message": "hi", "provider": "ollama", "model": "qwen2.5-coder:7b"}),
    );
    // Drive the Ollama arm (connects to the mock above).
    let _ = server.get(&format!("/api/chat/{id}/stream")).text();

    // Poll the capture (the stream call already completed, so this returns
    // immediately; the deadline is only a no-hang safety net).
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let model = loop {
        if let Some(m) = captured.lock().unwrap().clone() {
            break Some(m);
        }
        if std::time::Instant::now() >= deadline {
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    assert_eq!(
        model.as_deref(),
        Some("qwen2.5-coder:7b"),
        "the operator must send the SELECTED model to Ollama, got {model:?}"
    );
    assert_ne!(
        model.as_deref(),
        Some("claude-opus-4-8[1m]"),
        "Ollama must never receive the Claude model id"
    );
}

#[test]
fn operator_status_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/status");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("sprint").is_some(), "should have sprint");
    assert!(body.get("current_phase").is_some(), "should have phase");
}

#[test]
fn operator_lint_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/lint");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("ok").is_some(), "should have ok field");
}

// Sprint 80 Phase G: the live gate registry. Shape-only against the live
// repo (the planning-lint status is non-deterministic here); the
// deterministic semantics (>=1 not_run + >=1 passed, errors/warnings split)
// live in the hermetic `gates_live_data` unit tests in `gates.rs`.
#[test]
fn operator_gates_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/gates");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();

    let gates = body["gates"].as_array().expect("gates array");
    assert!(
        !gates.is_empty(),
        "registry should restitute the known gates"
    );

    // The publish gates are repo-independent (hardcoded `not_run`) and the
    // CSP gate is always `not_applicable` to the SBFB repo, so these two
    // snake_case status strings are deterministic even against the live
    // repo — this pins the `#[serde(rename_all = "snake_case")]` wire
    // contract the Phase H panel reads (the lint status alone varies).
    let statuses: Vec<&str> = gates.iter().filter_map(|g| g["status"].as_str()).collect();
    assert!(
        statuses.contains(&"not_run"),
        "FG gates restitute the snake_case `not_run` status"
    );
    assert!(
        statuses.contains(&"not_applicable"),
        "the CSP gate restitutes the snake_case `not_applicable` status"
    );

    // 1:1 diagnostic — no aggregate verdict anywhere (the cardinal
    // "0 verdict calculé" invariant): no flattened bool at the root, and
    // every entry carries a distinct `status` string, never a `passed:bool`.
    assert!(
        body.get("overall").is_none(),
        "no aggregate verdict at root"
    );
    assert!(
        body.get("all_passed").is_none(),
        "no aggregate verdict at root"
    );
    assert!(body.get("passed").is_none(), "no flattened bool at root");
    for g in gates {
        assert!(
            g.get("status").and_then(|s| s.as_str()).is_some(),
            "each gate restitutes a distinct status string"
        );
        assert!(
            g.get("issues").map(|i| i.is_array()).unwrap_or(false),
            "each gate carries an issues array"
        );
        assert!(
            g.get("passed").is_none(),
            "a gate entry never collapses to passed:bool"
        );
    }
}

// The gate registry sits behind `auth_required` (the `authed` sub-router),
// never on the public bootstrap router — a missing token is 401.
#[test]
fn operator_gates_requires_auth() {
    let server = TestServer::start();
    let resp = server.raw_get("/api/gates", "Host: 127.0.0.1\r\n");
    assert!(
        resp.starts_with("HTTP/1.1 401"),
        "gates must require a token, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn operator_audit_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/audit/HEAD");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("title").is_some(), "should have title");
    assert!(body.get("ok").is_some(), "should have ok");
}

#[test]
fn operator_prompt_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/prompt/preflight");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("content").is_some(), "should have content");
    let content = body["content"].as_str().unwrap();
    assert!(content.contains("Preflight"), "should contain prompt text");
}

#[test]
fn operator_context_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/context");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("head").is_some(), "should have head");
    assert!(body.get("branch").is_some(), "should have branch");
}

#[test]
fn operator_providers_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/providers");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    let providers = body["providers"].as_array().unwrap();
    assert!(providers.len() >= 5, "should list providers");
}

#[test]
fn operator_actions_log_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/actions/log");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.is_array(), "log should be an array");
}

#[test]
fn operator_action_rejects_unlisted_command() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/actions/run",
        serde_json::json!({"command": "rm -rf /", "args": {}}),
    );
    assert_eq!(resp.status(), 403, "should reject unlisted command");
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("error").is_some());
}

#[test]
fn operator_context_pack_schema_complete() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/context-pack",
        serde_json::json!({
            "provider": "claude",
            "intent": "test",
            "role": "driver",
            "specialized_kind": "preflight"
        }),
    );
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("base_prompt").is_some(), "base_prompt");
    assert!(body.get("universal_prompt").is_some(), "universal_prompt");
    assert!(body.get("handoff_prompt").is_some(), "handoff_prompt");
    assert!(
        body.get("specialized_prompt").is_some(),
        "specialized_prompt"
    );
    assert!(body.get("runtime_context").is_some(), "runtime_context");
    assert!(body.get("agent_system").is_some(), "agent_system");
    assert!(body.get("process_docs").is_some(), "process_docs");
    assert!(body.get("active_artifacts").is_some(), "active_artifacts");
    assert!(body.get("operator_intent").is_some(), "operator_intent");
}

#[test]
fn operator_context_pack_includes_base_snapshot() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/context-pack",
        serde_json::json!({
            "provider": "claude",
            "intent": "test phase D"
        }),
    );
    let body: serde_json::Value = resp.json().unwrap();
    let base = &body["base_prompt"];
    assert!(base.get("path").is_some(), "base should have path");
    assert!(base.get("hash").is_some(), "base should have hash");
    let rt = &body["runtime_context"];
    assert!(rt.get("head").is_some(), "should have HEAD");
    assert!(rt.get("sprint").is_some(), "should have sprint");
    assert!(rt.get("phase").is_some(), "should have phase");
    let intent = &body["operator_intent"];
    assert_eq!(intent["provider"], "claude");
    assert_eq!(intent["intent"], "test phase D");
}

#[test]
fn operator_context_pack_rejects_chat_history_authority() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/context-pack",
        serde_json::json!({"provider": "claude"}),
    );
    let body: serde_json::Value = resp.json().unwrap();
    assert_eq!(
        body["chat_history_authoritative"], false,
        "chat_history_authoritative must be false"
    );
    assert!(
        body.get("chat_history").is_none(),
        "should not have chat_history field"
    );
    assert_eq!(
        body["notice"], "private chat history is non-authoritative",
        "should contain non-authoritative notice"
    );
}

#[test]
fn operator_chat_session_starts_from_context_pack() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude", "intent": "test"}),
    );
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("id").is_some(), "should have session id");
    let cp = &body["context_pack"];
    assert!(
        cp.get("base_prompt").is_some(),
        "context_pack should have base"
    );
    assert!(
        cp.get("universal_prompt").is_some(),
        "should have universal"
    );
    assert!(cp.get("handoff_prompt").is_some(), "should have handoff");
    assert_eq!(cp["chat_history_authoritative"], false);
}

// Sprint 79 Phase D: a fresh session receives the authoring knowledge matrix
// as a hashed path reference, verifiable by recompute.
#[test]
fn operator_context_pack_includes_authoring_knowledge() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/context-pack",
        serde_json::json!({"provider": "claude", "intent": "test phase D"}),
    );
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    let ak = body["authoring_knowledge"]
        .as_array()
        .expect("authoring_knowledge should be an array");
    let animejs = ak
        .iter()
        .find(|e| {
            e["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("animejs/MANIFEST.json"))
        })
        .expect("authoring_knowledge should reference the animejs MANIFEST");
    assert_eq!(animejs["exists"], true, "animejs MANIFEST should exist");

    // The hash is verifiable by recompute: blake3(MANIFEST bytes)[..8] (8 hex
    // of the file itself), distinct from the 16-hex per-layer hashes inside
    // MANIFEST.hashes that `tests/animejs_manifest.rs` already covers.
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let bytes = std::fs::read(repo_root.join("docs/factory/knowledge/animejs/MANIFEST.json"))
        .expect("read animejs MANIFEST");
    let expected = blake3::hash(&bytes).to_hex()[..8].to_string();
    assert_eq!(
        animejs["hash"].as_str().unwrap(),
        expected,
        "authoring_knowledge hash must equal recomputed blake3(MANIFEST)[..8]"
    );

    // Sprint 80 Phase D (fold D1): the daisyui pack is now surfaced too, so the
    // Knowledge advisory inspector lists both consumed packs. Guards the const
    // edit point (AUTHORING_KNOWLEDGE_MANIFESTS) — the corpus bytes themselves
    // are provenance-checked by `tests/daisyui_manifest.rs`.
    let daisyui = ak
        .iter()
        .find(|e| {
            e["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("daisyui/MANIFEST.json"))
        })
        .expect("authoring_knowledge should reference the daisyui MANIFEST (S80 Phase D)");
    assert_eq!(daisyui["exists"], true, "daisyui MANIFEST should exist");
}

// Sprint 79 Phase D: handle_chat_session rebuilds its own context_pack literal
// (it does not inherit handle_context_pack), so the authoring_knowledge field
// must be present there too — guards the dual-write invariant.
#[test]
fn operator_chat_session_includes_authoring_knowledge() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude", "intent": "test phase D"}),
    );
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    let cp = &body["context_pack"];
    let ak = cp["authoring_knowledge"].as_array().expect(
        "chat session context_pack should carry authoring_knowledge (dual-write invariant)",
    );
    assert!(
        ak.iter().any(|e| e["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("animejs/MANIFEST.json"))),
        "chat session authoring_knowledge should reference the animejs MANIFEST"
    );
    // Sprint 80 Phase D (fold D1): daisyui is dual-written here too — assert it
    // is referenced AND exists (non-vacant; `file_hash` only sets exists:true
    // after a successful read, so this proves the daisyui bytes are read at the
    // chat/session site as well as the context-pack site).
    let daisyui = ak
        .iter()
        .find(|e| {
            e["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("daisyui/MANIFEST.json"))
        })
        .expect(
            "chat session authoring_knowledge should reference the daisyui MANIFEST (S80 Phase D)",
        );
    assert_eq!(
        daisyui["exists"], true,
        "chat session daisyui MANIFEST should exist"
    );
    // The authority invariant stays intact alongside the new field.
    assert_eq!(cp["chat_history_authoritative"], false);
}

#[test]
fn operator_project_documents_maps_repo_and_session_refs() {
    let server = TestServer::start();
    let session_resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude", "intent": "docs map"}),
    );
    assert_eq!(session_resp.status(), 200);
    let session: serde_json::Value = session_resp.json().unwrap();
    let id = session["id"].as_str().unwrap();

    let resp = server.get(&format!("/api/project-documents?session={id}"));
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(
        body["total"].as_u64().unwrap_or_default() > 0,
        "document map should include repo files"
    );
    assert_eq!(body["session"]["id"], id);
    assert_eq!(body["session"]["chat_history_authoritative"], false);
    assert!(
        body["documents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["path"] == "AGENTS.md"),
        "AGENTS.md should be in the git-backed project inventory"
    );
    assert!(
        body["pinned"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["path"] == "prompts/agent/base.md" && p["role"] == "use"),
        "the active LLM session should pin the base prompt as a used document"
    );
}

#[test]
fn operator_chat_message_endpoint() {
    let server = TestServer::start();
    let session_resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude"}),
    );
    let session: serde_json::Value = session_resp.json().unwrap();
    let id = session["id"].as_str().unwrap();

    let msg_resp = server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "what is the current phase?"}),
    );
    assert_eq!(msg_resp.status(), 200);
    let body: serde_json::Value = msg_resp.json().unwrap();
    assert!(body.get("response").is_some(), "should have response");
    assert_eq!(
        body["requires_gate"], false,
        "non-sensitive should not gate"
    );
}

#[test]
fn operator_chat_log_endpoint() {
    let server = TestServer::start();
    let session_resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude"}),
    );
    let session: serde_json::Value = session_resp.json().unwrap();
    let id = session["id"].as_str().unwrap();

    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "hello"}),
    );

    let log_resp = server.get(&format!("/api/chat/{id}/log"));
    assert_eq!(log_resp.status(), 200);
    let body: serde_json::Value = log_resp.json().unwrap();
    assert!(body.get("messages").is_some(), "should have messages");
    let messages = body["messages"].as_array().unwrap();
    assert!(messages.len() >= 2, "should have user + assistant messages");
}

#[test]
fn operator_chat_logs_messages_and_actions() {
    let server = TestServer::start();
    let session_resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude"}),
    );
    let session: serde_json::Value = session_resp.json().unwrap();
    let id = session["id"].as_str().unwrap();

    server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "status please"}),
    );

    let log_resp = server.get("/api/actions/log");
    let body: serde_json::Value = log_resp.json().unwrap();
    let log = body.as_array().unwrap();
    assert!(
        log.iter()
            .any(|e| e["action"].as_str() == Some("chat-session")),
        "should log session creation"
    );
    assert!(
        log.iter()
            .any(|e| e["action"].as_str() == Some("chat-message")),
        "should log messages"
    );
}

#[test]
fn operator_chat_rejects_sensitive_action_execution() {
    let server = TestServer::start();
    let session_resp = server.post_json(
        "/api/chat/session",
        serde_json::json!({"provider": "claude"}),
    );
    let session: serde_json::Value = session_resp.json().unwrap();
    let id = session["id"].as_str().unwrap();

    let msg_resp = server.post_json(
        "/api/chat/message",
        serde_json::json!({"session_id": id, "message": "please commit and push my changes"}),
    );
    let body: serde_json::Value = msg_resp.json().unwrap();
    assert_eq!(
        body["requires_gate"], true,
        "should require gate for commit"
    );
    assert_eq!(
        body["requires_external_agent"], true,
        "should require external agent"
    );
}

#[test]
fn operator_artifact_draft_rejects_pass_verdict() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/artifacts/draft",
        serde_json::json!({
            "path": ".planning/active/sprint70_phase_D_review.md",
            "content": "## Verdict: PASS\nAll good."
        }),
    );
    assert_eq!(resp.status(), 403, "should reject PASS verdict");
}

#[test]
fn operator_artifact_draft_logs_action() {
    let server = TestServer::start();
    server.post_json(
        "/api/artifacts/draft",
        serde_json::json!({
            "path": ".planning/active/test_draft_log.md",
            "content": "# Draft\nTest content."
        }),
    );

    let log_resp = server.get("/api/actions/log");
    let body: serde_json::Value = log_resp.json().unwrap();
    let log = body.as_array().unwrap();
    assert!(
        log.iter()
            .any(|e| e["action"].as_str() == Some("artifact-draft")),
        "should log artifact draft action"
    );
    let _ = std::fs::remove_file(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".planning/active/test_draft_log.md"),
    );
}

#[test]
fn operator_artifact_draft_rejects_path_traversal() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/artifacts/draft",
        serde_json::json!({
            "path": ".planning/active/../../.env",
            "content": "SECRET=leaked"
        }),
    );
    assert_eq!(resp.status(), 403, "should reject path traversal");
    let body: serde_json::Value = resp.json().unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("path traversal"),
        "error should mention path traversal"
    );
}

#[test]
fn operator_artifact_draft_allows_pass_pending() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/artifacts/draft",
        serde_json::json!({
            "path": ".planning/active/test_pass_pending_draft.md",
            "content": "## Verdict: PASS-PENDING\nReview en cours."
        }),
    );
    assert_eq!(resp.status(), 200, "PASS-PENDING drafts should be allowed");
    let _ = std::fs::remove_file(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".planning/active/test_pass_pending_draft.md"),
    );
}

#[test]
fn operator_action_run_allowed_command() {
    let server = TestServer::start();
    let resp = server.post_json(
        "/api/actions/run",
        serde_json::json!({"command": "status-sprint", "args": {}}),
    );
    assert_eq!(resp.status(), 200, "allowed command should succeed");
    let body: serde_json::Value = resp.json().unwrap();
    assert!(
        body.get("sprint").is_some() || body.get("error").is_some(),
        "should return sprint data or error"
    );
}

// Sprint 71 Phase D / G6: the off-sprint sprint-history + commit-diff +
// terminal-sessions endpoints shipped with no coverage. These exercise the
// real HTTP surface through the authenticated `TestServer` harness (the
// substitution endorsed by the Phase D preflight: the planned
// `chat_session_lifecycle` was already covered in Phase C, so the genuinely
// uncovered endpoints are tested instead).

#[test]
fn operator_sprint_history_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/sprint-history");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("sprint").is_some(), "should have sprint");
    assert!(body.get("phases").is_some(), "should have phases");
    assert!(body["phases"].is_array(), "phases is an array");
}

#[test]
fn operator_sprint_history_all_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/sprint-history/all");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    let sprints = body["sprints"].as_array().expect("sprints array");
    assert!(!sprints.is_empty(), "repo history has sprints");
    assert!(body.get("total").is_some(), "should have total");
}

// Sprint 80 Phase F: GET /api/git/diff returns the working-tree diff
// envelope. Shape-only (the live repo is dirty during the run, so the
// content is non-deterministic; hunk classification is asserted in the
// hermetic unit test in sprint_history.rs).
#[test]
fn operator_git_diff_endpoint_returns_envelope() {
    let server = TestServer::start();
    let resp = server.get("/api/git/diff");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("head").is_some(), "envelope has head");
    assert!(body["unstaged"].is_array(), "unstaged is an array");
    assert!(body["staged"].is_array(), "staged is an array");
    assert!(body["truncated"].is_boolean(), "truncated is a bool");
}

// The working-tree diff is behind the same auth as every /api route
// (it reveals uncommitted source) — no token => 401.
#[test]
fn operator_git_diff_requires_auth() {
    let server = TestServer::start();
    let resp = server.raw_get("/api/git/diff", "Host: 127.0.0.1\r\n");
    assert!(
        resp.starts_with("HTTP/1.1 401"),
        "git diff without token must be 401, got: {}",
        resp.lines().next().unwrap_or("")
    );
}

#[test]
fn operator_commit_diff_endpoint_returns_inline_code() {
    let server = TestServer::start();
    // `HEAD` always resolves; the handler shells `git diff HEAD^..HEAD` and
    // returns the structured file/hunk/line tree. The exhaustive line-kind
    // assertions live in the hermetic `parse_unified_diff` unit test; here we
    // prove the endpoint is wired and returns the inline-code structure.
    let resp = server.get("/api/sprint-history/diff/HEAD");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body.get("title").is_some(), "should have commit title");
    assert!(body.get("files").is_some(), "should have files");
    assert!(body["files"].is_array(), "files is an array");
}

#[test]
fn operator_commit_diff_rejects_invalid_sha() {
    let server = TestServer::start();
    // A too-short sha is rejected before any git call (guards path-traversal
    // and malformed revs).
    let resp = server.get("/api/sprint-history/diff/ab");
    assert_eq!(resp.status(), 400, "sha shorter than 4 chars must be 400");
}

#[test]
fn operator_terminal_sessions_endpoint() {
    let server = TestServer::start();
    let resp = server.get("/api/terminal/sessions");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body["sessions"].is_array(), "should have sessions array");
    assert!(
        body["claude_sessions"].is_array(),
        "should have claude_sessions array"
    );
}

// Sprint 71 Phase D / G5 retro-Codex P1: the diff and audit endpoints
// shelled `git log`/`git diff` with a raw rev. A rev that git parses as an
// option (e.g. `--output=<path>`) writes an arbitrary file (git option
// injection). The handlers must reject any option-like rev BEFORE the git
// call. These assert the 400 AND that no file was written.

#[test]
fn operator_commit_diff_rejects_option_injection() {
    let server = TestServer::start();
    let status = server
        .get("/api/sprint-history/diff/--output=sbfb_inject_diff")
        .status();
    // Capture + clean the would-be artifact BEFORE asserting, so a
    // regression (file written) never leaks a file even if an assert fails.
    let leaked = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sbfb_inject_diff");
    let existed = leaked.exists();
    let _ = std::fs::remove_file(&leaked);
    assert!(!existed, "git option injection must not write a file");
    assert_eq!(status, 400, "option-like sha must be 400");
}

#[test]
fn operator_audit_rejects_option_injection() {
    let server = TestServer::start();
    let status = server.get("/api/audit/--output=sbfb_inject_audit").status();
    let leaked = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sbfb_inject_audit");
    let existed = leaked.exists();
    let _ = std::fs::remove_file(&leaked);
    assert!(!existed, "git option injection must not write a file");
    assert_eq!(status, 400, "option-like rev must be 400");
}

#[test]
fn operator_terminal_session_content_rejects_traversal() {
    let server = TestServer::start();
    // `..` (percent-encoded so the URL layer does not normalise it away)
    // must not escape `.planning/terminal/`. Either the guard (400) or the
    // router (404) rejects it — never a 200 reading an outside file.
    let resp = server.get("/api/terminal/sessions/%2e%2e%5c%2e%2e%5csecret");
    assert!(
        resp.status() == 400 || resp.status() == 404,
        "traversal session name must be rejected, got {}",
        resp.status()
    );

    // Windows drive-prefix escape (`C:foo` is drive-relative and `join`
    // discards the terminal dir) — the phase-Codex live-probed this exact
    // bypass. `%3A` = `:`. Must be rejected, never a 200.
    let drive = server.get("/api/terminal/sessions/C%3Asbfb_drive_probe");
    assert!(
        drive.status() == 400 || drive.status() == 404,
        "drive-prefix session name must be rejected, got {}",
        drive.status()
    );
}
