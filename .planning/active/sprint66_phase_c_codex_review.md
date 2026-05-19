Audit statique sur `master`, worktree propre. Tests non exécutés.

### Livrable 1 : `runtime.rs` republish feed au boot
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:642`, `:645`, `:656`, `:663`, `:807`
- Evidence :
```rust
642: // iroh-docs at boot (one-shot, synchronous before HTTP).
645: let entries_result = {
649:     nexus_coordinator_rs::public_feed::replay_all(&db)
650: };
656: crate::feed_sync::publish_feed_entry_to_docs(fs, entry).await
663: info!(
```
- Le bloc est avant le spawn HTTP (`runtime.rs:807`). Le `MutexGuard` DB est bien limité au bloc `645-650` avant le `.await`.

### Livrable 2 : `feed_join_handles` + shutdown join
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:230`, `:674`, `:951`
- Evidence :
```rust
230: feed_join_handles: Option<Arc<std::sync::Mutex<Vec<JoinHandle<()>>>>>,
231: feed_join_shutdown: Option<Arc<tokio::sync::watch::Sender<bool>>>,
674: let (feed_join_shutdown_tx, _) = tokio::sync::watch::channel(false);
676: let feed_join_handles: Arc<std::sync::Mutex<Vec<JoinHandle<()>>>> =
```
```rust
951: if let Some(sender) = self.feed_join_shutdown.take() {
952:     let _ = sender.send(true);
954: if let Some(handles_arc) = self.feed_join_handles.take() {
959:     for mut h in handles {
960:         if let Err(e) = (&mut h).await {
```

### Livrable 3 : `feed_sync.rs` tracking/cap/shutdown de `feed_join`
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/feed_sync.rs:617`, `:622`, `:636`, `:661`
- Evidence :
```rust
617: let mut shutdown_rx = state.feed_join_shutdown.subscribe();
622: let handle = tokio::spawn(async move {
636: tokio::select! {
637:     event = live_stream.next() => {
652:     _ = shutdown_rx.changed() => {
```
```rust
661: if let Ok(mut handles) = state.feed_join_handles.lock() {
662:     handles.retain(|h| !h.is_finished());
663:     const MAX_FEED_JOINS: usize = 10;
664:     if handles.len() >= MAX_FEED_JOINS {
675:     handles.push(handle);
```
- Manque : le cap est contrôlé après `tokio::spawn`. Si `handles.len() >= 10`, la fonction retourne `429` avant `handles.push(handle)`, mais le `JoinHandle` déjà créé est détaché, non stocké et non join au shutdown. Le tracking/shutdown/select sont présents, mais le cap “10 joins actifs” n’est pas réellement garanti.

### Livrable 4 : provenance absente en HTTP 200
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:1764`
- Evidence :
```rust
1764: Ok(None) => (
1765:     StatusCode::OK,
1766:     Json(serde_json::json!({
1767:         "status": "absent",
1768:         "verified": false,
1769:         "record": null,
1770:         "provenance_hash": null,
```

### Livrable 5 : vérification provenance cross-node via `record.node_id`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:1738`
- Evidence :
```rust
1738: let (status, verified) = match hex::decode(&record.node_id) {
1740:     let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
1741:     let v = nexus_coordinator_rs::provenance::verify_provenance(
1742:         &record_json,
1743:         &pub_bytes,
```
- La vérification utilise la pubkey extraite de `record.node_id`, pas `state.pow_keypair.public_bytes()`.

### Livrable 6 : `DaemonHttpState` expose les handles feed join
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:176`, `crates/nexus-shell-daemon/src/runtime.rs:730`
- Evidence :
```rust
176: /// Sprint 66 Phase C: tracked JoinHandles for feed_join spawned
178: pub feed_join_handles: Arc<std::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>,
179: /// Sprint 66 Phase C: shutdown signal for feed_join tasks.
181: pub feed_join_shutdown: Arc<tokio::sync::watch::Sender<bool>>,
```
```rust
728: feed_sync_state,
729: feed_rate_limiter,
730: feed_join_handles: Arc::clone(&feed_join_handles),
731: feed_join_shutdown: Arc::clone(&feed_join_shutdown),
```

### Livrable 7 : `useBridge.ts` propage `status`
- Statut : CONFIRME
- Fichier(s) : `web/src/bridge/useBridge.ts:331`, `:333`, `:344`, `:346`
- Evidence :
```ts
331: if (resp.status === 404) return { record: null, provenance_hash: null, status: "absent" };
333: const data = (await resp.json()) as { record: unknown; provenance_hash: string; status: string };
334: return { record: data.record, provenance_hash: data.provenance_hash, status: data.status };
```
```ts
344: if (resp.status === 404) return { verified: false, status: "absent", record: null, provenance_hash: null };
346: const data = (await resp.json()) as { record: unknown; verified: boolean; provenance_hash: string; status: string };
347: return { verified: data.verified, status: data.status, record: data.record, provenance_hash: data.provenance_hash };
```

### Livrable 8 : badge provenance 4 états
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/BrowsedProject.tsx:183`, `:189`, `:302`, `:321`, `:326`, `:336`
- Evidence :
```tsx
183: const verifyQuery = useQuery({
189:   if (resp.status === 404) return { verified: false, status: "absent" as const };
197:   verified: data.verified,
198:   status: data.status as "verified" | "failed" | "absent",
```
```tsx
302: verifyQuery.isLoading
304:   : verifyQuery.isSuccess && verifyQuery.data.status === "verified"
306:     : verifyQuery.isSuccess && verifyQuery.data.status === "failed"
336: ) : (
339:   Provenance
```

### Livrable 9 : tests Rust attendus
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:5545`, `:5612`, `:5650`, `crates/nexus-shell-daemon/src/runtime.rs:1843`, `:1878`
- Evidence :
```rust
5545: async fn provenance_endpoint_absent_status() {
5557: assert_eq!(resp.status(), StatusCode::OK);
5560: assert_eq!(json["status"], "absent");
5561: assert_eq!(json["verified"], false);
```
```rust
5612: async fn provenance_cross_node_verified() {
5615: let other_kp = KeyPair::generate();
5620: &hex::encode(other_kp.public_bytes()),
5644: assert_eq!(json["verified"], true);
5645: assert_eq!(json["status"], "verified");
```
```rust
5650: async fn provenance_cross_node_tampered() {
5662: record.node_id = hex::encode(impostor_kp.public_bytes());
5684: assert_eq!(json["verified"], false);
5685: assert_eq!(json["status"], "failed");
```
- Manque : les 3 tests provenance sont utiles. En revanche `test_feed_republish_at_boot` (`runtime.rs:1843`) n’assert pas que l’entrée est réellement republiée dans iroh-docs, seulement DB=1 puis `feed_handle.is_some()` (`runtime.rs:1864-1872`). `test_feed_join_handles_tracked_and_shutdown` (`runtime.rs:1878`) ne crée aucun `feed_join`, donc n’assert ni stockage d’un handle réel ni drain effectif au shutdown.

### Livrable 10 : test Vitest badge absent
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/__tests__/BrowsedProject.test.tsx:403`
- Evidence :
```tsx
403: it("badge shows 'Provenance' when status is absent", async () => {
410: "/provenance": {
413:   status: "absent",
419: const badge = screen.getByTestId("verified-badge");
420: expect(badge).toHaveTextContent("Provenance");
```

## Resume final
- Total livrables : 10
- Confirmes : 8
- Gaps : 0
- Partiels : 2