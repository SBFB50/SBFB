# Sprint 50 — Plan (suppression Python + dette pair)

**Tip d'entree** : `da489a4` (audit findings S49 PASS).
**Phases** : A (dette pair) + B (suppression Python) +
C (docs + wrap-up).

---

## §Phase A — Dette pair obligatoire

### §A.1 Dispatch loop JoinHandle (P2-REVIEW-A-1-S49)

**Fichier** : `crates/nexus-shell-daemon/src/runtime.rs`

**Etat actuel** (L520-527) : le `tokio::spawn` du dispatch loop
drop le JoinHandle. Si le loop panic, le daemon ne le sait pas.

**Fix** : stocker le JoinHandle dans `DaemonRuntime`. A
`shutdown()`, drop le sender (channel close → loop exits) puis
join le handle.

```rust
// In DaemonRuntime struct:
dispatch_handle: Option<JoinHandle<()>>,

// In start():
let dispatch_handle = tokio::spawn(crate::dispatch_loop::run(
    task_dispatch_rx,
    doc_clone,
    doc_author,
));

// In shutdown():
// Closing task_dispatch_tx (via DaemonHttpState drop) signals
// the dispatch loop to exit. Then join.
if let Some(mut handle) = self.dispatch_handle.take() {
    if let Err(e) = (&mut handle).await {
        warn!(error = %e, "dispatch loop task join failed");
    }
}
```

Le sender est dans DaemonHttpState (drop order via
Arc refcount). Le channel close → `rx.recv()` retourne `None`
→ loop exit propre.

### §A.2 CLI handler integration tests (P2-REVIEW-B-1-S49)

**Fichier** : `crates/nexus-shell-daemon/src/main.rs` (tests)

**Etat actuel** : 8 parsing tests dans cli.rs mais 0 test
qui exercice les handlers (handle_init, handle_invite, etc.)
avec une DB reelle.

**Ajout** : tests integration dans main.rs ou un module
`coordinator_cli_tests.rs` qui :

1. `test_init_creates_db` : tempdir → handle_init → assert
   DB exists + can be opened.
2. `test_invite_create_list_revoke_cycle` : tempdir → init →
   invite create → invite list → invite revoke → list again.
3. `test_quarantine_list_empty` : tempdir → init → quarantine
   list → empty.
4. `test_capability_enable_disable` : tempdir → init →
   capability enable "compute" → list → disable → list.

Tests exercent le chemin complet handler → DB.

### §A.3 Memory tip stale close

P2-AUDIT-A-1-S49 : deja fixe dans la session audit (tip
memory mis a jour de `0cbfaab` vers `c72cf93`). CLOSE.

### §A.4 Commit

```
feat(sprint50): Sprint 50 Phase A — dette pair dispatch JoinHandle + CLI integration tests
```

Body : JoinHandle stored + joined at shutdown, 4 handler
integration tests, memory tip stale CLOSED.

---

## §Phase B — Suppression Python + PyO3

### §B.1 Suppression des packages Python

```bash
git rm -r packages/nexus-coordinator/
git rm -r packages/nexus-sdk/
git rm -r packages/nexus-app-gov/
git rm -r crates/nexus-core-py/
```

~30 853 LOC supprimes. ~505 tests Python supprimes.

### §B.2 Cleanup Cargo.toml workspace

**Fichier** : `Cargo.toml` (racine)

- Supprimer `"crates/nexus-core-py"` du `[workspace] members`
- Supprimer `pyo3 = "0.28"` et `pyo3-async-runtimes = "0.28"`
  des `[workspace.dependencies]`

### §B.3 Cleanup pyproject.toml + uv

**Fichier** : `pyproject.toml` (racine)

Si plus aucun Python dans le projet (hors scripts utilitaires) :
supprimer `pyproject.toml` et `uv.lock` entierement. Garder
`.python-version` si ruff/mypy sont encore utilises comme
dev-tools.

Decision a prendre au moment de la phase : verifier si
des scripts Python utilitaires restent (dans scripts/ ou
docs/ ou examples/). Si oui, garder un pyproject.toml minimal.
Si non, supprimer.

### §B.4 Cleanup frontend dead code

**Fichiers** : `web/src/`

1. Supprimer `useAppEvents` hook (SSE legacy)
2. Supprimer `AppTabPage` composant (rendu SDK legacy)
3. Grep systematique : `NexusApp|AppContext|coordinator` pour
   identifier d'autres references mortes
4. Supprimer les imports orphelins
5. Verifier : tsc + npm lint + Vitest + npm build

### §B.5 Commit

```
feat(sprint50): Sprint 50 Phase B — suppression Python packages + PyO3 + frontend dead code
```

Body : 4 packages supprimes (~30K LOC, ~505 tests), workspace
cleanup, frontend dead code cleanup, 0 LOC Python restant.

---

## §Phase C — Docs + verification + wrap-up

### §C.1 CLAUDE.md update

- §Commandes cles : supprimer le bloc Python (uv run ruff,
  uv run pytest, maturin develop)
- §Structure des crates/packages : supprimer les entries Python
- §Etat actuel : maj compteurs tests, supprimer mentions Python
- §Stack : simplifier (Rust + Node.js, pas de Python)

### §C.2 docs/claude/README.md update

- Fail-fast checklist : passer de 3 blocs (Rust+Python+Frontend)
  a 2 blocs (Rust+Frontend)
- Supprimer les references aux commandes pytest/ruff

### §C.3 Verification fail-fast 20+ checks

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1199, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 260 |
| 9 | build | `npm run build` (web/) | ok |
| 10 | size-limit | `npm run size` (web/) | 5/5 |
| 11 | Playwright | `npx playwright test` | >= 40 |
| 12 | Phase A preflight G8 | EXECUTE | present |
| 13 | Phase A review | PASS | present |
| 14 | Phase B preflight G8 | EXECUTE | present |
| 15 | Phase B review | PASS | present |
| 16 | Dispatch JoinHandle stored | runtime.rs | evidencie |
| 17 | CLI integration tests | 4 handler tests | pass |
| 18 | 0 LOC Python | packages/ + crates/nexus-core-py/ | absent |
| 19 | Cargo.toml clean | 0 pyo3 dep | confirmed |
| 20 | Scope cuts respectes | 8/8 | diff --stat |
| 21 | Frontend dead code removed | useAppEvents + AppTabPage | absent |
| 22 | CLAUDE.md updated | Python sections removed | confirmed |

### §C.4 Commit

```
chore(sprint50): Phase C — wrap-up + verification + audit plan S51 + counters
```

---

## §5 Notes de migration

### Python no longer in fail-fast

Apres Phase B, le fail-fast passe de 15 checks (S49) a 11
checks (S50) car les 5 checks Python (ruff format, ruff check,
pytest SDK, pytest coord, pytest gov) sont supprimes. Ceci est
une simplification attendue, pas une regression.

### Compteurs tests attendus post-S50

| Suite | Pre-S50 | Post-S50 | Delta |
|---|---|---|---|
| Rust nextest | 1195 | >= 1199 | +4 (Phase A handler tests) |
| Rust doctests | 6+1i | 6+1i | +0 |
| SDK pytest | 195 | 0 | -195 (DELETE) |
| Coord pytest | 264+17f+6s | 0 | -287 (DELETE) |
| Gov pytest | 46 | 0 | -46 (DELETE) |
| Vitest | 267 | >= 260 | -7 max (dead code cleanup) |
| Playwright | 42+2f | >= 40 | -4 max (page removal) |
| size-limit | 5/5 | 5/5 | +0 |
| **Total** | **~1947** | **~1509** | **-438** (Python) |

La baisse de -438 est entierement due a la suppression Python
intentionnelle. Pas de regression Rust ni Frontend.
