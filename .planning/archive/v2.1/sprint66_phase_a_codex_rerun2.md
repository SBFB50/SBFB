CLEAN

Audité sur `HEAD=118ada0`, worktree propre.

Vérifié dans [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1422): plus de `.ok().flatten()` ni `let _ = db.set_storage_namespace` dans `boot_storage_namespace` / `boot_feed_namespace`. Les erreurs sont propagées avec messages distincts pour read, ticket backfill, recreate et create.

`storage_api.rs` et `feed_sync.rs` ne sont pas dans le delta de `118ada0`. Test ciblé passé : `cargo test -p nexus-shell-daemon --locked boot_` -> `5 passed`.