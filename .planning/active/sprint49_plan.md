# Sprint 49 — Plan (coordinator lifecycle → daemon Rust)

**Tip d'entree** : `ebf14e7` (audit findings S48 PASS).
**Phases** : A (coordinator lifecycle) + B (CLI subcommands) +
C (wrap-up).

---

## §Phase A — Coordinator lifecycle dans le daemon

### §A.1 Project doc management dans runtime.rs

**Fichier** : `crates/nexus-shell-daemon/src/runtime.rs`

**Etat actuel** (ligne 202-585) :
- `start()` boot le node iroh, bind TCP, cree coordinator_db,
  spawn validator_loop, spawn gossip subscribe.
- PAS de document iroh-docs cree/reopen.
- Le validator_loop recoit des ResultEvent via broadcast channel
  mais aucune source ne produit ces events depuis le doc.

**Ajout** : apres le boot node (ligne 272) et avant le build
HTTP state (ligne 483), inserer :

```rust
// 2b. Create or reopen the project iroh-docs document.
let docs_client = nexus_core_rs::docs::DocsClient::new(
    Arc::clone(&node),
);
let author_id = docs_client.author_default().await
    .context("failed to get/create default author")?;
let project_doc = match docs_client.list_docs().await? {
    ref ids if !ids.is_empty() => {
        docs_client.open_doc(ids[0]).await?
            .ok_or_else(|| anyhow!("project doc listed but failed to open"))?
    }
    _ => docs_client.create_doc().await?,
};
info!(
    doc_id = %project_doc.id(),
    author = %author_id,
    "project doc ready for coordinator dispatch"
);
```

Le DocHandle est stocke dans un nouveau champ `project_doc:
Option<Arc<nexus_core_rs::docs::DocHandle>>` dans
DaemonHttpState. Le runtime (mode daemon) passe `Some(...)`,
les tests existants passent `None` (pas de changement mk_state).

### §A.2 Dispatch loop MPSC

**Fichiers** :
- `crates/nexus-shell-daemon/src/runtime.rs` (spawn loop)
- `crates/nexus-shell-daemon/src/dispatch_loop.rs` (NEW module)

**Pattern** (G1 D2 ack) :
- Un channel MPSC tokio `(task_submission_tx, task_submission_rx)`
  est cree dans start().
- `task_submission_tx` est stocke dans DaemonHttpState.
- Les endpoints HTTP `/api/v1/tasks` soumettent dans le channel
  au lieu d'ecrire directement dans la DB.
- Le dispatch loop drain le receiver, appelle
  `TaskDispatcher::submit()` pour signer+persister, puis ecrit
  la TaskEntry dans le project doc via `project_doc.set_bytes()`.

```rust
// dispatch_loop.rs
pub async fn run(
    mut rx: mpsc::Receiver<TaskSubmission>,
    dispatcher: TaskDispatcher,
    doc: Arc<DocHandle>,
    author: AuthorId,
) {
    while let Some(submission) = rx.recv().await {
        match dispatcher.submit(submission) {
            Ok(entry) => {
                let key = format!("tasks/{}", entry.task_id);
                let bytes = entry.canonical_bytes();
                if let Err(e) = doc.set_bytes(author, key, bytes).await {
                    warn!(error = %e, "failed to write task entry to doc");
                }
            }
            Err(e) => {
                warn!(error = %e, "dispatch rejected task submission");
            }
        }
    }
}
```

### §A.3 Doc subscription → result_event_tx

**Fichier** : `crates/nexus-shell-daemon/src/runtime.rs`

Le project doc est subscribe pour les LiveEvent (insertions par
les workers). Quand un worker ecrit un ResultEntry dans le doc,
l'event est forwarde vers `result_event_tx` pour que le
validator_loop existant (Sprint 38) puisse valider + crediter
kudos.

```rust
// In start(), after doc creation:
let sub_doc = Arc::clone(&project_doc_arc);
let sub_tx = result_event_tx.clone();
tokio::spawn(async move {
    let mut events = sub_doc.subscribe().await.unwrap();
    while let Some(event) = events.next().await {
        if let LiveEvent::InsertRemote { entry, .. } = event {
            let key = std::str::from_utf8(entry.key())
                .unwrap_or_default();
            if key.starts_with("results/") {
                let content = sub_doc.read_to_bytes(&entry).await;
                if let Ok(bytes) = content {
                    // parse ResultEntry, forward to validator
                    if let Ok(result_entry) = parse_result_entry(&bytes) {
                        let _ = sub_tx.send(ResultEvent::from(result_entry));
                    }
                }
            }
        }
    }
});
```

### §A.4 Integration test

**Fichier** : `crates/nexus-shell-daemon/src/runtime.rs` (tests)

Un test E2E pipeline verifie : daemon start() → submit task
via channel → dispatch loop ecrit dans doc → mock result entry →
validator processes → kudos credited.

Le test reutilise le pattern mk_state() existant (boot iroh node
reel).

### §A.5 Commit

```
feat(sprint49): Sprint 49 Phase A — coordinator lifecycle in daemon dispatch + validator doc wiring
```

Body riche avec : project doc create/reopen, dispatch loop MPSC
pattern, doc subscription → result_event_tx, integration test
pipeline.

---

## §Phase B — CLI coordinator subcommands

### §B.1 Subcommands clap

**Fichier** : `crates/nexus-shell-daemon/src/cli.rs`

Ajouter 4 subcommands au enum `Command` :

```rust
/// Initialize a new project for coordination.
Init {
    /// Project name.
    #[arg(long)]
    name: String,
    /// Project description.
    #[arg(long)]
    description: Option<String>,
},

/// Manage project invitations.
Invite(InviteCommand),

/// Manage quarantine queue.
Quarantine(QuarantineCommand),

/// Manage capability toggles.
Capability(CapabilityCommand),
```

Avec enums derives pour les sous-sous-commandes :

```rust
#[derive(Debug, Subcommand)]
pub enum InviteCommand {
    Create,
    List,
    Revoke { id: String },
}
```

Pattern identique pour `QuarantineCommand` (List, Flush, Drop)
et `CapabilityCommand` (List, Set, Get).

### §B.2 Handlers offline (G1 D3 ack)

**Fichier** : `crates/nexus-shell-daemon/src/main.rs` ou
`crates/nexus-shell-daemon/src/coordinator_cli.rs` (NEW module)

Les handlers one-shot operent en mode **offline** : ils ouvrent
le CoordinatorDb directement, sans daemon running.

```rust
async fn handle_init(paths: &ShellDaemonPaths, name: String, desc: Option<String>) -> Result<()> {
    let db_path = paths.root.join("coordinator.db");
    let db = CoordinatorDb::open(&db_path)?;
    // Create project record in DB
    // Print project ID + instructions
    Ok(())
}

async fn handle_invite(paths: &ShellDaemonPaths, cmd: InviteCommand) -> Result<()> {
    let db_path = paths.root.join("coordinator.db");
    let db = CoordinatorDb::open(&db_path)?;
    match cmd {
        InviteCommand::Create => {
            let entry = db.create_invite()?;
            println!("Created invite: {}", entry.id);
        }
        InviteCommand::List => {
            let invites = db.list_invites()?;
            for inv in invites { println!("{}", inv.id); }
        }
        InviteCommand::Revoke { id } => {
            db.revoke_invite(&id)?;
            println!("Revoked: {id}");
        }
    }
    Ok(())
}
```

### §B.3 Tests

- Test parsing CLI pour chaque subcommand (clap try_parse)
- Test handler init : cree un projet, verifie DB contient record
- Test handler invite create/list/revoke : cycle complet

Tests unitaires pour chaque subcommand + handler.

### §B.4 Commit

```
feat(sprint49): Sprint 49 Phase B — CLI coordinator subcommands init + invite + quarantine + capability
```

---

## §Phase C — Wrap-up

Verification fail-fast 28+ checks. Sprint50_audit_plan.md.
Compteurs tests. Migration docs.

```
chore(sprint49): Phase C — wrap-up + verification + audit plan S50 + counters
```

---

## §5 Verification fail-fast checklist (preview)

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1190, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | ruff format | `uv run ruff format --check packages/` | 0 diff |
| 7 | ruff check | `uv run ruff check packages/` | 0 error |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | 264+17f+6s |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 |
| 11 | npm lint | `npm run lint` (web/) | 0 error |
| 12 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error |
| 13 | Vitest | `npm run test:unit` (web/) | >= 267 |
| 14 | build | `npm run build` (web/) | ok |
| 15 | size-limit | `npm run size` (web/) | 5/5 |
| 16 | Phase A preflight G8 | EXECUTE | present |
| 17 | Phase A review | PASS | present |
| 18 | Phase B preflight G8 | EXECUTE | present |
| 19 | Phase B review | PASS | present |
| 20 | Project doc create/reopen | integration test | pass |
| 21 | Dispatch loop MPSC | TaskEntry in doc | evidencie dans diff |
| 22 | Doc subscription → validator | result forwarding | evidencie dans diff |
| 23 | CLI init subcommand | DB project record | test |
| 24 | CLI invite create/list/revoke | cycle complet | test |
| 25 | CLI quarantine/capability | handlers wired | test |
| 26 | Scope cuts respectes | 12/12 | diff --stat |
| 27 | Delta tests documente | cumule | present |
| 28 | G1 D2/D3 acks respectes | MPSC + offline | evidencie dans diff |
