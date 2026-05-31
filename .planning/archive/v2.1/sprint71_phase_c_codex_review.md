### Livrable 1 : G2 Gate SSE
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:34`, `:807`, `:837`, `:847`, `:869`
- Evidence :
```rust
807: .iter()
810: .rev()
811: .find(|m| m.role == "user")
812: .map(|m| m.content.clone())
```
```rust
837: let is_sensitive = SENSITIVE_ACTIONS
840: if is_sensitive {
847:     return sse_gate(
869: let claude_stream = llm_bridge::spawn_claude_stream(&prompt, &model, &root);
```
Le spawn SSE n’est atteint qu’après le gate. Recherche globale : seul appel applicatif à `spawn_claude_stream` en `operator_server.rs:869`; `spawn_agent_stream` est appelé depuis `llm_bridge.rs` et tests.

### Livrable 2 : G9 Modele opus-4-8
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:51`, `:265`, `:700`, `:727`, `:862`
- Evidence :
```rust
265: fn default_model() -> String {
266:     "claude-opus-4-8[1m]".to_string()
267: }
```
```rust
727: // persist the requested model so the bodyless SSE GET
729: if !req.model.trim().is_empty() {
730:     session.model = req.model.clone();
```
`"sonnet"` reste seulement en commentaire/assertion de test, pas comme modèle exécuté. Le stream relit `session.model` avant `spawn_claude_stream`.

### Livrable 3 : G7 Token + Host + CORS
- Statut : PARTIEL
- Fichier(s) : `crates/sbfb-factory/src/auth.rs:102`, `:215`, `:229`, `:241`, `:252`; `crates/sbfb-factory/src/operator_server.rs:99`, `:112`, `:141`
- Evidence :
```rust
229: pub async fn auth_required(..., req: Request, next: Next) -> Response {
232:     let host_ok = headers.get(header::HOST)...
241:     if let Some(origin) = headers.get(header::ORIGIN) {
252:     let token_ok = headers.get(AUTH_HEADER)...
```
```rust
99: let cors = CorsLayer::new()
100:     .allow_origin(AllowOrigin::predicate(|origin, _parts| {
106:     .allow_methods([Method::GET, Method::POST])
141: // Inner: auth on every route. Outer: CORS, so OPTIONS
142: // preflights are answered before the token check.
```
Confirmé pour les routes GET/POST/WS déclarées, incluant `/api/artifacts/draft`, `/api/chat/{id}/stream`, `/api/terminal/ws`, toutes avant le `.layer(auth_required)`. `allow_origin(Any)` absent. Comparaison token constant-time sur longueur égale (`auth.rs:215-223`).
Partiel : l’ordre CORS externe répond aux OPTIONS preflight avant le token, explicitement documenté aux lignes 141-142. Ce n’est pas un spawn/write direct, mais ce n’est pas “auth sur chaque requête” au sens strict.

### Livrable 4 : G12 Timeout + diagnostic
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/llm_bridge.rs:18`, `:95`, `:161`, `:164`, `:199`, `:201`
- Evidence :
```rust
164: let message = if e.kind() == std::io::ErrorKind::NotFound {
166:     "agent CLI `{exe}` not found on PATH ..."
```
```rust
199: match tokio::time::timeout(idle, lines.next_line()).await {
201:     let _ = child.start_kill();
202:     let _ = child.wait().await;
205:     "agent timed out after {}s of inactivity — process killed"
```
`spawn_claude_stream` conserve `--permission-mode bypassPermissions` sur le happy path (`llm_bridge.rs:103-112`) et délègue au wrapper borné.

### Livrable 5 : Contrat §4 PO-2
- Statut : CONFIRME
- Fichier(s) : `docs/agent/RRV_FACTORY_CONTRACT.md:144`, `:152`, `:160`, `:164`, `:167`
- Evidence :
```md
146: L'Operator peut piloter un agent local privilegie
152: Gate d'action sensible (D3)
160: Auth loopback (D5)
164: Timeout subprocess (D6)
167: Limite assumee : le terminal PTY WebSocket
```
Le contrat autorise le pilotage agent local privilégié non sensible et l’encadre par gate sensible, auth loopback et timeout.

### Livrable 6 : Tests
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/operator_server.rs:24`, `:127`, `:138`, `:151`, `:184`, `:217`, `:251`; `crates/sbfb-factory/src/llm_bridge.rs:341`, `:368`
- Evidence :
```rust
29: .env("SBFB_AUTH_TOKEN", TEST_TOKEN)
34: .env("SBFB_CLAUDE_BIN", "sbfb-claude-test-nonexistent")
127: fn server_rejects_missing_token()
138: fn server_rejects_foreign_host()
151: fn cors_restricts_origin()
```
```rust
184: fn sse_gates_sensitive_action()
217: fn sse_allows_nonsensitive()
251: fn chat_stream_uses_opus_model()
341: async fn missing_claude_diagnostic()
368: async fn spawn_times_out()
```
Assertions utiles présentes. Tests ciblés exécutés : `auth::tests` 5/5 OK, `llm_bridge::tests` 5/5 OK, `operator_server` 29/29 OK.

## Resume final
- Total livrables : 6
- Confirmes : 5
- Gaps : 0
- Partiels : 1

Points vigilance : middleware appliqué aux routes déclarées, mais OPTIONS preflight contourne le token par ordre CORS/auth; `allow_origin(Any)` absent; comparaison token constant-time à longueur égale; gate SSE lit bien le dernier message user; `"PASS"` reste volontairement dans `SENSITIVE_ACTIONS` et le guard artifact PASS.
