Verdict global: pas de P0/P1/P2 trouvé. Le fix ferme bien le warn-only destructif sur les deux namespaces M8. Je garde seulement un P3 de couverture test: le “loud” des logs n’est pas asserté.

**Livrables**

1. **Helpers internes `boot_storage_namespace` / `boot_feed_namespace`: OK**  
   Storage: `Err(e) if e.to_string().contains("Replica not found") => None`, puis tout autre `Err` retourne `refusing to silently recreate`: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2498). Recreate seulement dans `None`, avec `ns`: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2526).  
   Feed miroir: même ordre et même discrimination: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2593), recreate avec `ns`: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2621). Pas de bras générique avant le guard.

2. **Call-sites `DaemonRuntime::start`: OK**  
   Storage ne dégrade plus en session partielle, il `return Err(e.context(...))`: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:708). Feed idem: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:750). `main` propage par `?` sur `DaemonRuntime::start`: [main.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:203). Systemd est bien `Restart=on-failure` + 5s: [nexus-shell-daemon.service](C:/Users/FlowUP/Documents/Code/nexus/deploy/nexus-shell-daemon.service:30).

3. **Commentaires/doc-comment rectifiés: OK**  
   `DocsClient::open_doc` dit maintenant que 0.98 expose l’absence en `Err` contenant `"Replica not found"` et rappelle le préfixe `open failed`: [docs.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/docs.rs:151). Les deux blocs runtime documentent la même frontière NotFound vs corruption: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2487), [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:2590).

4. **Tests ajoutés: PARTIEL**  
   Les tests self-heal prouvent que la row M8 stale est remplacée par le nouveau namespace: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4177), [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4212). Les tests fail-fast prouvent le marqueur `refusing to silently recreate` et la row M8 intacte après shutdown: [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4242), [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:4281).  
   Partiel seulement parce que `recreates_loud...` n’assert pas le `warn`/champ `ns`; le comportement est dans le code, mais pas capturé par test.

**Invariants**

0-bump respecté: diff suivi limité à `docs.rs` et `runtime.rs`; pas de diff `Cargo.toml`/`Cargo.lock`/canonical/DB. Pins iroh inchangés côté workspace: [Cargo.toml](C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:38), lock iroh-docs 0.98.0: [Cargo.lock](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:4031). Schéma M8 inchangé: [db.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/db.rs:149). `set_storage_namespace` reste un upsert local sans migration: [db.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/db.rs:1023).

Autres `open_doc`: seul autre call-site workspace trouvé = `project_doc` à [runtime.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:650). Il ne recrée rien silencieusement; il fail-fast via `?`. Hors scope comme demandé.

**GAPs**

P3: les tests ne prouvent pas le caractère “loud” du self-heal, seulement la recréation et la réécriture M8. Aucun P0/P1/P2 trouvé.

