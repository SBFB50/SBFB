**Verdict**

B-1 : **CONFIRME**.
Le writer de prod écrit bien `task:{id}` dans `crates/nexus-shell-daemon/src/dispatch_loop.rs:35-41`, puis persiste cette clé via `doc.set(...)` à `crates/nexus-shell-daemon/src/dispatch_loop.rs:49`. Le test interne vérifie maintenant le même préfixe: scan `b"task:"` à `crates/nexus-shell-daemon/src/dispatch_loop.rs:116`, `starts_with("task:")` à `:127`, et égalité `task:{task_id}` à `:130`.

Le reader worker est aligné: `get_many_by_prefix(b"task:")` à `crates/nexus-worker-core/src/engine/runtime.rs:847`, puis `strip_prefix("task:")` à `crates/nexus-worker-core/src/engine/runtime.rs:859`.
Scan complet `crates/` sur `tasks/`: aucune ligne active de reader/writer docs iroh en `tasks/`. Les restes sont commentaires ou routes HTTP REST: `crates/nexus-shell-daemon/src/dispatch_loop.rs:39`, `:136`, `crates/nexus-shell-daemon/src/http.rs:306`, `:405`, `:4351`, `:4459`, `:5052`, `crates/nexus-shell-daemon/src/tasks_api.rs:6`, `:111`, `crates/nexus-worker-core/src/rate_limit.rs:15`, `crates/nexus-core-rs/src/crypto.rs:57`.

B-3 : **CONFIRME**, avec une précision: c’est un E2E réel dispatch-loop + worker Engine dans le même test process, pas un lancement OS multi-process.
Le test existe à `crates/nexus-shell-daemon/src/dispatch_loop.rs:146`. Il utilise un vrai `Engine` (`:150`, boot à `:191`) avec `StubBackend` déterministe (`:151`, `:187`). Il crée le doc depuis l’accessor `engine.docs()` (`:195-197`), envoie la tâche dans le channel (`:199-204`), puis lance le vrai writer de prod `run(rx, ...)` à `:201`. Il ne contourne pas le dispatch par un `doc.set` manuel.

Le test serait sensible à une régression B-1: il vérifie d’abord que le dispatcher a écrit sous `b"task:"` à `crates/nexus-shell-daemon/src/dispatch_loop.rs:211-212`; ensuite il enregistre ce doc dans le worker (`:215`), lance `engine.run_until_shutdown()` (`:217`), attend une entrée `result:` (`:219-229`), puis assert `claim:` et `result:` (`:231-234`). Si le writer revenait à `tasks/`, l’assert `task:` échouerait, et le worker ne verrait pas la tâche via `runtime.rs:847`.

L’accessor public `Engine::docs()` est additif et correct: il retourne `DocsClient::new(self.node.docs())` sans mutation d’état à `crates/nexus-worker-core/src/engine/runtime.rs:562-564`.

G1 : **CONFIRME**.
`crates/sbfb-factory/src/terminal.rs` reste sur asciicast: extension `.cast` à `crates/sbfb-factory/src/terminal.rs:27`, fonctions `write_asciicast_header` et `write_asciicast_event` à `:30` et `:41`, appels câblés à `:133` et `:143`. Recherche locale: aucun `.log` ni `PlainTextWriter` dans ce fichier. Le `git status --short` final ne montre pas `terminal.rs` modifié; seuls `dispatch_loop.rs`, `runtime.rs` et des fichiers planning non suivis apparaissent.

Tests: non conclusifs. `cargo test -p nexus-shell-daemon dispatch_loop --locked -- --nocapture` a expiré à 10 min sans sortie exploitable; les deux filtres individuels ont aussi expiré. Donc l’audit code est **confirmé**, mais je ne peux pas attester un PASS runtime Cargo sur cette machine.
