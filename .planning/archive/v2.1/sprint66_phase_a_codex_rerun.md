Pas CLEAN.

GAPs :

- **P1** — [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1470) et [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1541) : les branches happy path `None => create_doc()` gardent `let _ = db.set_storage_namespace(...)`. Ce n’est pas cohérent avec le fix : si SQLite échoue après création du doc iroh, le daemon démarre avec un namespace vivant mais non persisté en DB. Au restart, il ne peut pas le retrouver. Même classe de risque que le recreate path, même si ce n’est pas un chemin recovery.

- **P1** — [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1422) et [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1498) : `db.get_storage_namespace(...).ok().flatten()` masque aussi les erreurs SQLite de lecture et les transforme en “pas de row”. Pour une phase persistence, une erreur DB au boot devrait échouer explicitement, pas déclencher une création de namespace.

- **P2** — le P2 tests reste ouvert : `boot_storage_namespace_persistent_reopen` et `boot_feed_namespace_persistent_reopen` passent, mais prouvent surtout boot/shutdown/reboot sans panic. Ils n’assertent pas le même `namespace_id`, et n’exercent pas les branches fallback recreate.

Constat positif : le fix `2b57d37` propage bien l’erreur DB dans les deux branches recreate (`storage` et `feed`). Les deux `let _` de ticket manquant sur row existante sont moins graves : le `namespace_id` est déjà en DB, donc pas le même risque d’orphelin.