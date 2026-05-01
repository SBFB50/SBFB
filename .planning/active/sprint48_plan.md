# Sprint 48 — Plan (dette pair carries resolution batch)

**Tip d'entree** : `3d14068` (audit findings S47 PASS).
**Phases** : A (dette pair 2/3) + B (S47 carries batch) + C
(wrap-up).

---

## §Phase A — dette pair 2/3 items resolution

### §A.1 TOCTOU canary reload fix

**Fichier** : `crates/nexus-coordinator-rs/src/canary_input.rs`

**Etat actuel** (lignes 500-548) :
- `reload_policy()` : lock reload mutex → check mtime → **drop
  lock** → read_to_string → lock policy → update
- `reload_set()` : meme pattern, drop lock avant read

**Fix** : reordonner pour garder le lock `reload` pendant le
read. Le scope du MutexGuard englobe check mtime + read file +
update. Le lock `policy`/`set` interne est pris brievement pour
le swap, pas de deadlock (ordre fixe reload → policy/set).

```rust
// Avant (TOCTOU) :
let should_reload = { let mut rs = self.reload.lock()...; };
let content = std::fs::read_to_string(&path)?;

// Apres (fix) :
let mut rs = self.reload.lock()...;
// check mtime sous lock
let content = std::fs::read_to_string(&path)?;
// parse + update sous lock
```

**Tests** : test existant `canary_input_policy_reload` couvre
deja le reload. Pas de nouveau test necessaire — le TOCTOU est
un race condition non-deterministique, le fix est structural.

### §A.2 kudos SQL pagination total_count

**Fichiers** :
- `crates/nexus-shell-daemon/src/kudos_api.rs:59-76`
- `web/src/components/project/KudosTab.tsx:41`

**Etat actuel** :
```rust
let entries = db.list_kudos_entries(project_id)?;
let entries: Vec<_> = entries.into_iter()
    .skip(query.offset.unwrap_or(0))
    .take(query.limit.unwrap_or(100).min(500))
    .collect();
// count = entries.len() ← page size, pas total
```

**Fix backend** :
```rust
let all_entries = db.list_kudos_entries(project_id)?;
let total_count = all_entries.len();
let entries: Vec<_> = all_entries.into_iter()
    .skip(query.offset.unwrap_or(0))
    .take(query.limit.unwrap_or(100).min(500))
    .collect();
// reponse JSON : { entries, count: entries.len(), total_count }
```

**Fix frontend** : `KudosTab.tsx` ligne ~41, remplacer
`query.data?.count` par `query.data?.total_count`.

**Schema Zod** : ajouter `total_count: z.number()` dans le
schema KudosList si un schema Zod existe pour cette route.
Verifier dans `coordinator.ts` ou `daemon.ts`.

### §A.3 app-specific schema drift exemption

**Action** : documenter l'exemption dans le commit body. L'item
P2-REVIEW-C-1-S46 est bloque par App Runtime Migration (routes
`/app/*` dependent de Python coordinator AppContext). Le bloqueur
est une dependance sequentielle interne multi-sprint. L'item est
reclassifie hors compteur carry actif — il ne peut pas atteindre
3/3 tant que le bloqueur existe. Il sera reactive quand les
routes `/app/*` seront portees en Rust.

### §A.4 Commit

```
feat(sprint48): Sprint 48 Phase A — dette pair TOCTOU canary fix + kudos total_count + schema drift exemption
```

---

## §Phase B — S47 carries batch

### §B.1 execute_batch_raw feature gate

**Fichiers** :
- `crates/nexus-coordinator-rs/Cargo.toml`
- `crates/nexus-coordinator-rs/src/db.rs:348`
- `crates/nexus-shell-daemon/Cargo.toml`

**Fix** :
1. Ajouter dans `nexus-coordinator-rs/Cargo.toml` :
   ```toml
   [features]
   test-support = []
   ```
2. Modifier `db.rs:348` :
   ```rust
   #[doc(hidden)]
   #[cfg(any(test, feature = "test-support"))]
   pub fn execute_batch_raw(&self, sql: &str) -> Result<(), CoordinatorError> {
   ```
3. Ajouter dans `nexus-shell-daemon/Cargo.toml` :
   ```toml
   [dev-dependencies]
   nexus-coordinator-rs = { path = "../nexus-coordinator-rs", features = ["test-support"] }
   ```
   Verifier que la dep existe deja et fusionner si necessaire.

### §B.2 invite format test

**Fichier** : `crates/nexus-shell-daemon/src/http.rs` — test
`invite_create_success` (~ligne 4162)

**Fix** : ajouter des assertions sur le format de l'ID :
```rust
let id = body["id"].as_str().unwrap();
assert!(id.starts_with("inv-"), "invite ID must start with inv-");
let parts: Vec<&str> = id.split('-').collect();
assert_eq!(parts.len(), 4, "format inv-{{node8}}-{{ts}}-{{seq}}");
assert_eq!(parts[1].len(), 8, "node_id prefix must be 8 hex chars");
```

### §B.3 sbfb_home refactor dans DaemonHttpState

**Fichiers** :
- `crates/nexus-shell-daemon/src/http.rs` — DaemonHttpState + mk_state + 7 tests
- `crates/nexus-shell-daemon-core/src/consent.rs` — handlers consent
- `crates/nexus-shell-daemon-core/src/files.rs` — handlers files

**Fix** :
1. Ajouter `pub sbfb_home: Option<PathBuf>` dans
   `DaemonHttpState`
2. Creer une fonction helper `resolve_sbfb_home(state_home:
   Option<&Path>) -> PathBuf` qui retourne state_home si Some,
   sinon `std::env::var("SBFB_HOME")`, sinon `~/.sbfb/`
3. Modifier `consent.rs` et `files.rs` pour appeler
   `resolve_sbfb_home(state.sbfb_home.as_deref())` au lieu de
   `sbfb_home()` directement
4. Modifier `mk_state()` pour accepter et passer le tmpdir
5. Supprimer les 7 `std::env::set_var("SBFB_HOME", ...)` dans
   les tests

**auth.rs** : les 4 set_var dans `auth.rs` suivent un pattern
save/restore different (backup + restore a la fin du test).
Evaluer si le refactoring s'etend naturellement. Si oui,
inclure. Si le pattern est plus complexe (interaction avec
d'autres env vars), carry S49.

### §B.4 deploy BlobsClient reclassification

**Action** : documenter la reclassification dans le commit body.
Le risque est inherent a mk_state() qui boot un iroh Node reel
pour 50+ tests. Fix = refactoring majeur de l'infra de test
(BlobsClient trait mock ou test server). Accepte pre-v1.0. Item
supprime du compteur carry. Si flake observe en CI future,
rouvrir comme P2 dedie.

### §B.5 Commit

```
feat(sprint48): Sprint 48 Phase B — S47 carries batch execute_batch_raw gate + invite test + sbfb_home refactor
```

---

## §Phase C — Wrap-up

Verification fail-fast 28+ checks. Sprint49_audit_plan.md.
Compteurs tests. Migration docs dans la session suivante si
necessaire.

```
chore(sprint48): Phase C — wrap-up + verification + audit plan S49 + counters
```

---

## §5 Verification fail-fast checklist (preview)

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1185, 0 fail |
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
| 20 | TOCTOU canary reload fix | mutex hold-across-read | evidencie dans diff |
| 21 | kudos total_count | champ present + frontend | evidencie dans diff |
| 22 | schema drift exemption | documentee | commit body |
| 23 | execute_batch_raw feature gate | cfg visible | evidencie dans diff |
| 24 | invite format test | assertions pattern | evidencie dans diff |
| 25 | sbfb_home dans state | set_var elimines | evidencie dans diff |
| 26 | BlobsClient reclassification | documentee | commit body |
| 27 | Scope cuts respectes | 10/10 | diff --stat |
| 28 | Delta tests documente | cumule | present |
