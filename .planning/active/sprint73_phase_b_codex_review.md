Audit statique du working tree (`git diff HEAD` consulté). Je n’ai pas lancé les suites de tests.

### Livrable 1 : P2-A-1 worker-pump iroh-docs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:96`, `:119`, `:135`, `:175`, `:239`, `:250`; `crates/nexus-worker-core/src/engine/runtime.rs:1407`, `:1445`, `:1509`, `:1718`, `:1772`, `:1839`, `:1996`; `docs/rust/PATTERNS.md:2915`
- Evidence :
```rust
96: #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
119: tokio::time::timeout(std::time::Duration::from_secs(10), async {
126:     if n == 1 { return; }
135: let _ = shutdown_tx.send(());
```
Les 5 tests pump runtime sont en `multi_thread` aux lignes listées. Les deux tests virtual-time restent `#[tokio::test]` avec `tokio::time::pause()` (`runtime.rs:1772`, `:1817`, `:1839`, `:1876`). `rg "#\[cfg\(windows\)\]"` ne retourne rien dans les deux fichiers ciblés. La règle P54 documente la conversion et l’exception virtual-time (`PATTERNS.md:2915-2942`).

### Livrable 2 : P2-TEST-ZOMBIE
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/process_cli.rs:472`, `:480`, `:482`, `:545`, `:562`, `:567`, `:595`
- Evidence :
```rust
480: let dir = tempfile::tempdir().expect("tempdir");
482: Command::new("git").args(["init"])
545: let output = factory_bin()
546:     .args(["process", "audit-commit", "--rev", "HEAD", "--json"])
```
Les anciens SHA `6fb95df` et `c4494a6` ne restent qu’en commentaires (`:474`, `:563`). Aucune cible `--rev <sha>` ne subsiste ; les deux tests utilisent une fixture git locale et `--rev HEAD`.

### Livrable 3 : P2-OPERATOR-TIMEOUT
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/operator_server.rs:27`, `:96`, `:105`, `:118`
- Evidence :
```rust
27: fn client_timeout() -> Duration {
28:     std::env::var("SBFB_TEST_HTTP_TIMEOUT_SECS")
31:         .map(Duration::from_secs)
32:         .unwrap_or(Duration::from_secs(30))
```
Le timeout est utilisé dans `get` (`:100`), `post_json` (`:110`) et `raw_get` (`:121`).

### Livrable 4 : P2-OPERATOR-NO-TEST-RUNNER
- Statut : CONFIRME
- Fichier(s) : `tools/factory-operator/package.json:11`, `:43`, `:54`, `:58`; `tools/factory-operator/vitest.config.ts:21`; `tools/factory-operator/src/test/setup.ts:31`, `:66`, `:106`; `tools/factory-operator/src/lib/executionChat.test.ts:30`; `tools/factory-operator/src/pages/ExecutionChat.test.tsx:49`
- Evidence :
```ts
21: test: {
23:   environment: "jsdom",
24:   setupFiles: ["./src/test/setup.ts"],
25:   include: ["src/**/*.{test,spec}.{ts,tsx}"],
```
Les tests sont réels : `executionChat.test.ts` vérifie le payload `model` (`:30-42`, `:45-53`), et `ExecutionChat.test.tsx` vérifie le rendu SSE + fermeture et le court-circuit gate sans stream (`:49-72`, `:75-89`).

### Livrable 5 : P2-POLL-DIAGNOSTIC-LOSS
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provider_router.rs:391`, `:393`, `:398`, `:430`, `:888`, `:959`
- Evidence :
```rust
393: let mut last_err: Option<String> = None;
398: let detail = last_err.as_ref()
404:     "network task {task_id} timed out after {}s{detail}",
430: last_err = Some(format!("status poll HTTP {}", r.status()));
```
Le test `network_provider_surfaces_last_error_on_timeout` mocke un poll HTTP 401 (`:888-915`) et assert que le timeout contient `timed out` et `401` (`:959-965`).

### Livrable 6 : P2-SYNC-FS-ASYNC
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provider_router.rs:281`, `:288`, `:974`, `:993`; `crates/sbfb-factory/src/daemon_client.rs:26`
- Evidence :
```rust
281: async fn resolve_daemon() -> Result<(String, String), String> {
288: match tokio::task::spawn_blocking(crate::daemon_client::DaemonConnection::discover).await {
289:     Ok(Ok(conn)) => Ok((conn.base_url, conn.token)),
```
`DaemonConnection::discover()` reste sync (`daemon_client.rs:26`). Les tests couvrent l’override sans fs (`provider_router.rs:974-990`) et la découverte offloadée via erreur propre sur root vide (`:993-1011`).

### Livrable 7 : P2-OLLAMA-MODEL-PICKER
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:325`, `:813`, `:924`, `:957`, `:971`, `:1134`; `crates/sbfb-factory/tests/operator_server.rs:451`; `tools/factory-operator/src/lib/executionChat.ts:68`; `tools/factory-operator/src/pages/ExecutionChat.tsx:138`, `:304`; `tools/factory-operator/src/i18n/locales/fr.json:215`; `tools/factory-operator/src/i18n/locales/en.json:215`
- Evidence :
```rust
325: fn default_model_for_provider(provider: &str) -> String {
327:     "ollama" | "local" => std::env::var("SBFB_OLLAMA_DEFAULT_MODEL")
331:     "network" => std::env::var("SBFB_NETWORK_DEFAULT_MODEL")
336:     _ => default_model(),
```
Claude garde `claude-opus-4-8[1m]`, les providers non-Claude n’en héritent pas, et l’override env Ollama est testé (`operator_server.rs:1134-1174`). Le modèle est résolu côté send (`:819-824`) et stream (`:961-965`). Le gate `SENSITIVE_ACTIONS` reste avant dispatch provider (`:924-944`, puis dispatch `:971-972`). L’intégration capture le modèle envoyé à Ollama et assert `qwen2.5-coder:7b`, pas Claude (`tests/operator_server.rs:537-545`). Frontend : `sendMessage` porte `model` (`executionChat.ts:68-78`) et le champ est affiché uniquement hors Claude (`ExecutionChat.tsx:304-321`), avec clés i18n fr/en.

### Livrable 8 : Isolation des tests env
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:334`; `crates/sbfb-factory/Cargo.toml:44`; `crates/sbfb-factory/src/provider_router.rs:597`, `:629`, `:790`, `:835`, `:887`, `:973`, `:994`; `crates/sbfb-factory/src/operator_server.rs:1160`; `crates/sbfb-factory/src/daemon_client.rs:145`; `crates/sbfb-factory/src/auth.rs:305`; `crates/sbfb-factory/src/publish.rs:44`
- Evidence :
```rust
304: #[test]
305: #[serial(sbfb_env)]
306: fn env_token_takes_precedence_over_file() {
311:     unsafe { std::env::set_var(AUTH_TOKEN_ENV, &expected) };
```
`serial_test` est en workspace/dev-dep, et les tests mutants env dans les cinq fichiers demandés portent `#[serial(sbfb_env)]`. Les mutations listées par `rg std::env::set_var|remove_var` correspondent aux blocs sérialisés.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0