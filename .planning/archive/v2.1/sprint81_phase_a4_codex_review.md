Audit read-only du working tree. Je n’ai pas rejoué `cargo test`; les verdicts ci-dessous portent sur le code, les assertions et les artefacts.

### Livrable 1 : `DocHandle::start_sync(peers)`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/docs.rs:385`, `crates/nexus-core-rs/src/docs.rs:405`
- Evidence :
```rust
385: /// Enter this document's live sync-set, optionally dialing `peers`.
388: /// sync-set - verified against iroh-docs 0.98: only `start_sync`
392: /// is_syncing`, `engine/live.rs:714`) and (b) REJECTS every
404: /// internals against iroh-docs 0.101 at the version bump.
405: pub async fn start_sync(&self, peers: Vec<iroh::EndpointAddr>) -> Result<()> {
```
Délégation réelle à `self.inner.start_sync(peers)` et mapping `NexusError::Docs` à `docs.rs:406-409`.

### Livrable 2 : helper de boot `open_project_doc_for_dispatch`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:648`, `crates/nexus-shell-daemon/src/runtime.rs:2051`
- Evidence :
```rust
2054: let existing = docs_client
2057:     .context("failed to list project docs")?;
2058: let project_doc = if let Some(&first_id) = existing.first() {
2062:             .context("failed to open project doc")?
2070: project_doc.start_sync(Vec::new()).await.context(
```
Les messages d’erreur reprennent l’ancien bloc inline de `HEAD` (`failed to list/open/create project doc`). Le call-site boot appelle le helper et logue `sync-set entered` à `runtime.rs:648-652`.

### Livrable 3 : test CONTROL sans `start_sync`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:544`, `crates/nexus-shell-daemon/src/dispatch_loop.rs:556`, `crates/nexus-shell-daemon/src/dispatch_loop.rs:618`
- Evidence :
```rust
556: #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
557: async fn reopened_project_doc_without_start_sync_does_not_deliver() {
591: // Restart: same store, same identity, reopen WITHOUT start_sync.
610:             .start_sync(vec![a2_addr])
618:         assert!(
619:             !await_exact_key(
```
Le test a un coordinateur persistant avec `NodeConfig.with_secret_key(...).with_data_dir(...)` à `dispatch_loop.rs:523-528`, baseline convergente à `:576-589`, puis assertion négative bornée à `:618-629`.

### Livrable 4 : test GREEN via boot prod
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:642`, `crates/nexus-shell-daemon/src/dispatch_loop.rs:691`
- Evidence :
```rust
642: #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
643: async fn boot_path_reenters_sync_set_and_delivers_after_reopen() {
691: let doc_a2 = crate::runtime::open_project_doc_for_dispatch(&docs_a2)
694: assert_eq!(
713: assert!(
```
Le test vérifie le même `doc_id` à `dispatch_loop.rs:694-698`, re-dial worker accepté via `doc_b.start_sync(vec![a2_addr])` à `:700-707`, puis convergence post-restart à `:713-722`.

### Livrable 5 : hermétisme des 6 tests `consent_`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:8926`, `crates/nexus-shell-daemon/src/http.rs:9037`
- Evidence :
```rust
8926: async fn consent_get_returns_default_config() {
8930:     let tmp = tempfile::tempdir().expect("tmpdir");
8931:     let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
9037: async fn consent_whitelist_remove_missing_project_id_422() {
9042:     let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
```
Scan par fonction : les 6 tests demandés sont `NO_MK_STATE` et `USES_TEMP_HOME`; scan global `async fn consent_*` : `ALL_CONSENT_TESTS_NO_MK_STATE_AWAIT`.

### Livrable 6 : artefact différentiel A4
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_t2_a4_differential_098.json:1`
- Evidence :
```json
4:   "against_baseline": ".planning/active/sprint81_t2_baseline_098.json",
10:     "boot_window_pre_submit": {
11:       "verdict": "PASS",
12:       "criterion": "per the baseline a3b_differential_contract: after a coordinator restart and BEFORE any submit...",
13:       "observed": "boot logs 'project doc ready for coordinator dispatch (sync-set entered)'..."
```
`ConvertFrom-Json` passe. Scan local : `NO_SECRET_IP_USERNAME_OR_ABSOLUTE_PATH_PATTERNS`. Cohérence baseline confirmée par `.planning/active/sprint81_t2_baseline_098.json:49-51`.

### Livrable 7 : artefact preflight pointeur
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_phase_a4_preflight.md:1`
- Evidence :
```md
1: # Sprint 81 Phase A4 - Preflight G8 (POINTEUR, pas un re-jeu)
5: > (`sprint81_phase_a3_preflight.md`, Workflow `wf_7ffb4c95-8b6`, 11 agents,
8: > par phase committee) : le G8 d'A4 N'A PAS ete re-joue - il vit
11: > - **§2** - root-cause re-etablie au code
15: > - **§3 « A3b »** - l'approche corrigee que A4 implemente
```
Les sections `§6`, `§7`, `§10` sont aussi explicitement référencées à `preflight.md:20-24`.

### Livrable 8 : invariant 0-bump
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:35`, `crates/nexus-core-rs/src/doc_sync.rs:130`
- Evidence :
```rust
35: // Key prefix MUST match the worker scan in nexus-worker-core
41: let key = format!("task:{}", entry.task.task_id);
130: /// Re-issue `start_sync(peers)` if the cooldown has elapsed
143: match doc.inner().start_sync(peers.to_vec()).await {
```
Diff contrôlé : seul `crates/nexus-core-rs/src/docs.rs` change dans `crates/nexus-core-rs/src`; aucun hit diff pour `_VERSION`, `DOMAIN_`, `ALPN`, `canonical`, `JCS`; `crates/nexus-core-rs/src/doc_sync.rs` n’est pas modifié.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0