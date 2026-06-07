Verdict global : **CONFIRMÉ avec 1 gap de durcissement sur le livrable 3**. J’ai audité le working tree réel : `crates/sbfb-factory/src/fork.rs` existe mais est encore non suivi Git. Les tests ciblés passent : `cargo test -p sbfb-factory fork --locked`, `cargo test -p nexus-coordinator-rs browse_rowid_partitioned_from_feed_seq --locked`, `cargo test -p nexus-shell-daemon browse_index_rejects_open_source_without_provenance --locked`.

**1. Fork backend**
CONFIRMÉ. `#[allow(dead_code)] mod fork;` est câblé en attente Phase C dans `crates/sbfb-factory/src/main.rs:9-14`. `fork_from_search_hit` préfère forge HTTPS puis blob : `crates/sbfb-factory/src/fork.rs:114-130`. `fork_from_forge` valide HTTPS/SHA puis clone : `fork.rs:136-149`. `fork_from_blob` appelle l’extraction zip : `fork.rs:173-180`.

**2. Forge clone / injection Git**
CONFIRMÉ. Git CLI via `tokio::process::Command::new("git")` : `fork.rs:332-339`. Pas de `git2/gix` dans les dépendances Factory : `crates/sbfb-factory/Cargo.toml:11-38`, et `rg "git2|gix"` ne retourne rien dans `Cargo.toml/Cargo.lock`. La validation `commit_sha` 40-hex arrive avant tout clone/fetch : `fork.rs:144-149`, avec la même forme que `deploy.rs::is_valid_sha` : `crates/nexus-shell-daemon/src/deploy.rs:76-83` et `deploy.rs:482-484`. `--end-of-options` est bien avant le SHA au `fetch` : `fork.rs:303-317`. Timeouts 30s/10s et `kill_on_drop(true)` : `fork.rs:59-60`, `fork.rs:298-325`, `fork.rs:337-339`. Cap post-clone 500 MB : `fork.rs:56-57`, `fork.rs:150-156`, parité `deploy.rs:28` et `deploy.rs:104-112`.

**3. Reconstruction blob / zip non fiable**
CONFIRMÉ pour les exigences principales, avec gap de durcissement Windows. Cap compressé : `fork.rs:181-183`. Cap décompressé réel via `Read::take(remaining + 1)` puis rejet si dépassement : `fork.rs:225-235`; test zip-bomb : `fork.rs:562-569`. Symlink ignoré avant écriture : `fork.rs:202-205`; test : `fork.rs:541-558`. Zip-slip `..`, `/abs`, `\abs`, backslash : `fork.rs:207-210` et `fork.rs:258-264`; tests : `fork.rs:521-538` et `fork.rs:573-582`. Parité avec `blob_serve::validate_zip_path` : `crates/nexus-shell-daemon-core/src/blob_serve.rs:181-190`, tests `blob_serve.rs:311-325`.

GAP L3 : sur Windows, `is_safe_archive_path` ne rejette pas lexicalement un préfixe disque de type `C:/...` car il ne teste ni `:` ni les préfixes `std::path::Component::Prefix` (`fork.rs:258-264`). Le check canonique peut empêcher la création du fichier, mais il arrive après `create_dir_all(parent)` : `fork.rs:212-220`. Donc si “préfixe absolu” inclut les préfixes disque Windows, ce n’est pas pleinement bloqué avant toute touche disque.

**4. Workspace hors repo**
CONFIRMÉ selon la règle demandée : les fonctions reçoivent toutes `dest` du caller (`fork.rs:114-118`, `fork.rs:136-140`, `fork.rs:173-174`) et ne dérivent pas le workspace depuis `repo_root_pub`. Le seul appel `repo_root_pub()` dans `fork.rs` est dans le test d’invariant : `fork.rs:586-598`.

**5. C.3 rowid**
CONFIRMÉ. La logique existante `BROWSE_ROWID_BASE` / `browse_rowid` est là : `crates/nexus-coordinator-rs/src/search.rs:72-97`. Le test de régression ajouté existe et vérifie la partition browse/feed : `search.rs:349-378`. Je n’ai pas vu de modification de la logique de partition elle-même dans cette phase.

**6. B.6 open-source invariant au chokepoint**
CONFIRMÉ. Le gate HTTP `/publish` local existe encore : `crates/nexus-shell-daemon/src/http.rs:926-945`, mais le vrai chokepoint partagé redowngrade `is_open_source` si `repo_url` ou `provenance_hash` manque : `http.rs:984-1012`, avec log `warn` : `http.rs:1000-1004`. Les chemins prod passent par ce chokepoint : deploy et `/publish` via `publish_announcement` puis `index_browse_entry` : `crates/nexus-shell-daemon/src/deploy.rs:375-380` et `deploy.rs:445-468`; gossip live appelle `handle_project_announcement` : `crates/nexus-shell-daemon/src/runtime.rs:1464-1470`, construit l’entrée depuis l’annonce non fiable : `runtime.rs:1692-1710`, puis indexe via `index_browse_entry` : `runtime.rs:1711-1716`. Test B.6 présent : `http.rs:2248-2313`.

**7. Pas de fuite Phase C / wire format**
CONFIRMÉ sur le périmètre audité. Le nouveau module annonce explicitement “workspace only”, pas templates, redeploy, ni re-signature provenance : `fork.rs:19-22`. `main.rs` ne fait qu’ajouter le module dead-code, pas de commande CLI : `crates/sbfb-factory/src/main.rs:9-14`. Les changements C.3/B.6 sont locaux aux index/tests : `search.rs:349-378`, `http.rs:984-1012`, `http.rs:2248-2313`. Je n’ai pas trouvé de bump `*_VERSION`, de migration, de wire format ou de per-app `project_id` OFF-SPRINT-2b dans les fichiers modifiés.

GAPs finaux :
- L3 uniquement : absence de rejet lexical explicite des préfixes absolus Windows `C:/...` avant `create_dir_all(parent)`.
