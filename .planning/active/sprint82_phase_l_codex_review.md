Verdict global : les 7 livrables sont confirmés. Audit effectué contre `HEAD 713f0fa7d60f9febe11b3af003e328536583e908`, sans utiliser la review Claude et sans relancer les suites lourdes.

### Livrable 1 : Décomposition de `start()`

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:276):276-877, 1071-1530 ; [main.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:643):643
- Evidence :

```rust
pub async fn start(opts: DaemonStartOptions) -> Result<Self> {
    opts.paths
        .ensure_dirs()
        .context("failed to create shell-daemon directories")?;
```

La signature est strictement identique à HEAD. `start()` occupe 276-877, soit 602 lignes inclusives. Les sept extractions sont présentes et restent sous 150 lignes hors doc-comments :

- `boot_node_identity` : 1071-1119, 49 lignes.
- `bind_api_listener` : 1128-1146, 19 lignes.
- `restore_revocation_cache` : 1153-1193, 41 lignes.
- `boot_feed_recovery` : 1203-1293, 91 lignes.
- `wire_auth` : 1306-1322, 17 lignes.
- `spawn_api_server` : 1350-1405, 56 lignes.
- `spawn_gossip_and_boot_seed_driver` : 1446-1530, 85 lignes.
- `ApiServerHandles` : 1340-1345.

L’appelant production reste `main.rs:643`. Le recensement donne toujours 27 appels de test, soit 28 appels au total avec la production.

### Livrable 2 : Pureté du refacto

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1071):1071-1530, 2147-3224
- Evidence :

```rust
let cfg = NodeConfig::default()
    .with_secret_key(secret_bytes)
    .with_data_dir(iroh_data_dir.to_path_buf());
let factory = crate::seed_protocol::seed_protocol_factory(
    Arc::clone(coordinator_db),
```

La reconstruction mécanique donne les correspondances suivantes :

- Identité : HEAD 345-388 → WT 1077-1116, identique après `root`, `to_path_buf()` et adaptation des références.
- Bind : HEAD 398-410 → WT 1132-1144, identique après passage de `api_host/api_port`.
- Révocations : HEAD 591-627 → WT 1156-1191, identique.
- Feed recovery : HEAD 792-879 → WT 1208-1291, identique.
- Auth : HEAD 1017-1031 → WT 1307-1321, identique après wrappers `Ok`.
- API : HEAD 1033-1095 → WT 1357-1404, ordre et arguments identiques, plus tuple de retour.
- Gossip/boot driver : HEAD 1114-1206 → WT 1452-1529, identique après accès via `state.*`, clones et wrapper `Result`.

Les 21 fonctions helpers déplacées sont textuellement identiques à HEAD. Le multiensemble des littéraux Rust de logs et erreurs supprimés/ajoutés est strictement identique. Aucun token de condition n’a été ajouté ou retiré, aucun nouvel `unwrap`, `expect`, `panic!`, `todo!` ou `unimplemented!` n’apparaît dans le diff. `git diff --check` est également propre.

### Livrable 3 : Séquence de boot et contraintes ordonnées

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:281):281-351, 678-705, 1132-1139, 1463-1518, 1768-1950, 2881-3221
- Evidence :

```rust
let feed_sync_for_republish =
    if crate::noop_identity::gossip_publish_in_duress(identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        None
```

Les 11 paires sont préservées :

1. PoW avant dispatch : vérification 1846-1859, premier dispatch à 1860.
2. Anti-spoof avant handler projet : 1908-1919.
3. `accepted &&` avant re-drive : 1928-1947.
4. Substitution duress : 678-685, appel 690 ; l’option est consommée aux deux endroits 1208 et 1236, sans accès à `feed_sync_state`.
5. Clamp loopback avant bind : 1132-1139.
6. `start_sync` dans le gate duress : 2881-2892, 3080-3094 et 3203-3217.
7. Garde migration avant `create_doc` : 3043-3052 et 3171-3179.
8. Singleton → node → bind → `running.json` : 282, 330, 341, 351.
9. DB et nonce cache avant node : 320-327 puis appel 330-336 ; factories SEED_ALPN 1084-1113.
10. Prune/restore/repull avant signal : 1768, 1794, 1809 puis `send` 1813-1815.
11. Lock commun et ré-annonce en premier : création 1468, consommateurs 1487/1494, ré-annonce 1511 avant lock/driver 1516-1518.

`identity_mode` est capturé à 447 puis transmis aux appels duress-gated à 544, 597, 644 et 703, ainsi qu’au `DaemonHttpState` à 757. Aucun `IdentityMode::Normal` par défaut n’existe dans le chemin production.

L’ordre des spawns reste : janitors 1359/1370/1381 → peer 1392 → HTTP 1395 → validator via l’appelant 836 → gossip 1469 → boot driver 1495.

### Livrable 4 : Couplage A↔L et re-drive

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1468):1468-1518, 1667-2117, 2364-2490
- Evidence :

```rust
boot_replay_done: Some(boot_replay_done_tx),
boot_driver_state: Some(Arc::clone(state)),
keep_online_projects: keep_online_projects.to_vec(),
seed_driver_lock: Arc::clone(&seed_driver_lock),
redrive_coord: Arc::new(tokio::sync::Mutex::new(RedriveCoord::default())),
```

```rust
let configured = keep_online_projects.to_vec();
let boot_driver_state = Arc::clone(state);
let seed_driver_lock = Arc::clone(&seed_driver_lock);
tokio::spawn(async move {
```

Le lock production est créé une seule fois à 1468. Il est cloné vers `GossipTaskConfig` à 1487 et vers le boot driver à 1494. Les autres créations trouvées sont dans les modules `#[cfg(test)]`, après `runtime.rs:3230` ou dans les tests HTTP.

`GossipTaskConfig` reste identique à HEAD avec 19 champs, lignes 1667-1703. `spawn_gossip_subscribe_task` est textuellement identique à HEAD sur 1708-2117. `REDRIVE_MIN_INTERVAL`, `RedriveCoord` et `maybe_redrive_seed_on_ingest` restent `pub(crate)` à 2364, 2373 et 2431, avec corps identiques à HEAD.

La ré-annonce directory demeure avant l’acquisition du lock et `run_boot_seed_driver`, lignes 1511-1518.

### Livrable 5 : Regroupement des helpers

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2125):2125-3224
- Evidence :

```rust
// =================================================================
// Announcement ingest helpers
// =================================================================
```

Les groupes sont contigus et dans l’ordre demandé :

- Announcement ingest : 2125-2354 — fonctions à 2147, 2193, 2242 et 2252.
- Re-drive-on-ingest : 2356-2490 — constante 2364, struct 2373, fonction 2431.
- Outbox/replay : 2492-2820 — fonctions à 2500, 2525, 2543, 2561, 2583, 2605, 2634, 2681, 2726, 2759 et 2800.
- Boot namespaces/migration : 2822-3224 — fonctions à 2861, 2912, 2931, 2975 et 3114.

Les corps de toutes ces fonctions sont identiques à HEAD. Le commentaire « project announcement » a été déplacé sans modification de HEAD 2226-2229 vers WT 2248-2251. Le commentaire « Remediation #7 » est identique de HEAD 2636-2655 vers WT 2780-2799.

### Livrable 6 : Ressources et guards

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:852):852-876, 1208-1273, 1340-1404, 1452-1529
- Evidence :

```rust
http_handle,
http_shutdown: Some(http_shutdown_tx),
gossip_handle,
gossip_shutdown: Some(gossip_shutdown_tx),
peer_handle,
```

```rust
tokens_watcher,
pow_policy_watcher: _pow_policy_watcher,
dispatch_handle: Some(dispatch_handle),
dispatch_shutdown: Some(dispatch_shutdown_tx),
result_sync_handle: Some(result_sync_handle),
```

Tous les éléments attendus atteignent `Ok(Self { .. })` :

- HTTP/gossip/peer et shutdowns : 856-861.
- Watchers auth/PoW : 862-863.
- Dispatch/result-sync : 864-867.
- Feed et feed-join : 869-872.
- Boot driver : 873.
- `bound_addr` et `revocation_cache` : 874-875.

Le tuple API est défini à 1340-1345, produit à 1404 et déstructuré à 823-829. Le tuple gossip est produit à 1529 et déstructuré à 844-850.

Les guards `std::sync` extraits sont bornés avant les `.await` : 1208-1214 avant 1219, 1244-1249 avant 1272, et 1454-1462 dans une fonction synchrone. Le guard inline `app_storage` reste fermé à 792 sans `.await`.

### Livrable 7 : Tests et symboles cross-module

- Statut : CONFIRME
- Fichier(s) : [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:3230):3230-5455 ; [dispatch_loop.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/dispatch_loop.rs:694):694-1077 ; [http.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:6201):6201-6292
- Evidence :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
```

Le module de tests de `runtime.rs`, de `#[cfg(test)]` jusqu’à EOF, est textuellement identique à HEAD ; son SHA-256 LF-normalisé reste `db620dd501d3452ba326b397ab20ec1508b935911cfdf0ef4f48f3e4d60fb790`. Aucun hunk de test n’existe dans le diff.

Les consommateurs cross-module restent présents :

- `dispatch_loop.rs` : `open_project_doc_for_dispatch` à 694/1077, `boot_storage_namespace` à 796/868, `boot_feed_namespace` à 809/881.
- `http.rs` : `GossipCmdTx` 93, `GossipCmd` 934/1113, `mint_ticket_for_hash` 3513, `RedriveCoord` 6203 et `maybe_redrive_seed_on_ingest` 6213-6292.
- Définitions visibles dans `runtime.rs` : `GossipCmd` 1644, `GossipCmdTx` 1665, re-drive 2364/2373/2431, mint 2525, boot helpers 2861/2975/3114.

Les tests d’ancrage ne sont pas vides : le test re-drive contient huit assertions utiles (`http.rs:6165-6307`), boot-restore neuf (`runtime.rs:4215-4313`), feed-republish deux (`runtime.rs:5147-5179`) et duress deux (`feed_sync.rs:954-1006`). Le test de reopen storage sans macro `assert!` vérifie néanmoins deux boots et shutdowns via `unwrap`, lignes 4724-4732.

Les suites annoncées par le mainteneur n’ont pas été relancées, conformément à la consigne ; aucun écart structurel ne contredit leurs résultats.

## Résumé final

- Total livrables : 7
- Confirmés : 7
- Gaps : 0
- Partiels : 0