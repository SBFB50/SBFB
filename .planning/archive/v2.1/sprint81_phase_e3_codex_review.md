Audit statique du working tree actuel uniquement, sans historique de session. Tests non exécutés.

### Livrable 1 : `TopicSender::join_peers`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/gossip.rs:493`, `crates/nexus-core-rs/src/gossip.rs:522`
- Evidence :
```rust
pub async fn join_peers(&self, peers: Vec<String>) -> Result<()> {
    let parsed: Vec<PublicKey> = peers.into_iter().filter_map(|s| match PublicKey::from_str(&s) {
        Ok(pk) => Some(pk),
        Err(e) => { warn!(peer = %s, error = %e, "join_peers: skipping unparseable node id"); None }
    }).collect();
    if parsed.is_empty() { return Ok(()); }
```
La délégation est bien vers `self.inner.join_peers(parsed).await`, avec `inner: GossipSender`.

### Livrable 2 : `GossipCmd::JoinPeers` + bras runtime
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:1482`, `crates/nexus-shell-daemon/src/runtime.rs:1863`
- Evidence :
```rust
pub enum GossipCmd {
    Outbox(Vec<u8>),
    RequestBrowse,
    JoinPeers(Vec<String>),
}
```
```rust
Some(GossipCmd::JoinPeers(peers)) => {
    if let Err(e) = sender.join_peers(peers).await {
        debug!(error = %e, "hot join_peers failed");
    }
}
```

### Livrable 3 : push depuis `subscribe_curator`, après duress, producteur unique
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:878`, `crates/nexus-shell-daemon/src/http.rs:889`, `crates/nexus-shell-daemon/src/http.rs:905`
- Evidence :
```rust
if crate::noop_identity::curator_subscribe_in_duress(state.identity_mode)
    == crate::noop_identity::SubscribeOutcome::Noop
{
    return (StatusCode::OK, Json(SubscriptionsResponse { subscribed_curators: Vec::new() })).into_response();
}
match state.curator_runtime.subscribe(&req.curator_pubkey_hex) {
```
```rust
let _ = state.gossip_cmd_tx
    .send(crate::runtime::GossipCmd::JoinPeers(vec![req.curator_pubkey_hex.clone()]))
    .await;
```
`rg "GossipCmd::JoinPeers|JoinPeers\("` ne montre qu’un seul envoi prod : `http.rs:907`; le reste est enum, handler runtime et tests.

### Livrable 4 : test normal pousse exactement un `JoinPeers`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:6626`
- Evidence :
```rust
assert_eq!(resp.status(), StatusCode::OK);
let cmd = rx.try_recv().expect("subscribe must push a gossip command");
let crate::runtime::GossipCmd::JoinPeers(peers) = cmd else { panic!(...) };
assert_eq!(peers, vec![hex_key]);
assert!(matches!(rx.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
```

### Livrable 5 : test duress ne pousse rien
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:6674`
- Evidence :
```rust
assert_eq!(resp.status(), StatusCode::OK);
assert!(
    matches!(rx.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)),
    "duress subscribe must push nothing to the gossip task"
);
```

### Livrable 6 : test hex invalide = 400 + canal vide
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:6710`
- Evidence :
```rust
curator_pubkey_hex: "not-hex".to_string(),
...
assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
assert!(matches!(rx.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
```

### Livrable 7 : test core skip bad ids + enqueue valid
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/gossip.rs:775`
- Evidence :
```rust
sender.join_peers(vec![]).await.expect("empty join is a no-op");
let valid = hex::encode(KeyPair::generate().public_bytes());
sender.join_peers(vec![valid.clone()]).await.expect("valid id enqueues");
sender.join_peers(vec!["not a real key".into(), valid]).await
    .expect("mixed batch degrades per peer, no abort");
```

### Livrable 8 : artefacts Phase E3 + T2 JSON
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_plan.md:239`, `.planning/active/sprint81_t2_e3_hot_subscribe.json:9`
- Evidence :
```md
## Phase E3 — Hot-join gossip du curateur souscrit
> *Déclarée à l'exécution ...
- **Livrables** : 4 edits ci-dessus + artefact T2
  `sprint81_t2_e3_hot_subscribe.json`
```
```json
"hot_subscribe_convergence": {
  "verdict": "RIG-ABSENT",
  "criterion": "node A boots fresh ... WITHOUT restarting A ...",
  "observed": "not run at commit time ..."
}
```
Les anchors hermétiques sont listés dans le JSON lignes 16-18. Note : le JSON existe mais est non suivi (`??`) dans le working tree.

### Livrable 9 : invariant 0 bump wire / pins inchangés
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:41`, `Cargo.lock:3919`, `Cargo.lock:4095`, `crates/nexus-shell-daemon/src/runtime.rs:1482`
- Evidence :
```toml
[workspace.dependencies]
iroh = "=1.0.1"
iroh-docs = "=0.101.0"
iroh-gossip = "=0.101.0"
```
```rust
pub enum GossipCmd {
    Outbox(Vec<u8>),
    RequestBrowse,
    JoinPeers(Vec<String>),
}
```
`git diff --name-only` ne liste pas `Cargo.toml` ni `Cargo.lock`; `git diff --numstat -- Cargo.toml Cargo.lock` ne retourne aucune ligne. Le diff ne touche pas de constante `*_VERSION` / `DOMAIN_*_V1`; `GossipCmd` reste un enum interne mpsc sans sérialisation.

## Résumé final
- Total livrables : 9
- Confirmes : 9
- Gaps : 0
- Partiels : 0