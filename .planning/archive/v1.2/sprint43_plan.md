# Sprint 43 — Plan

**Kickoff ref** : `sprint43_kickoff.md` (D1..D3).
**Roadmap ref** : `.planning/roadmap_v1_migration_rust.md` §S43.
**Tip d'entree** : `e1f7f00`.
**Sprint impair** — pas de phase dette obligatoire.

---

## §Phase A — MANDATORY batch (7 items)

### A.1 Scope

Resoudre les 7 items MANDATORY/OVERDUE documentes D1.

### A.2 Fichiers touches

| Fichier | Changement |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` | `pub fn conn()` → `pub(crate) fn conn()` |
| `crates/nexus-coordinator-rs/src/canary_registry.rs` | L158,168 `let _ = self.persist()` → `if let Err(e) = self.persist() { tracing::warn!(...) }` |
| `crates/nexus-coordinator-rs/src/canary_input.rs` | L366-376 consolider 3 Mutex mtime/check → 1 `Mutex<ReloadState>` struct |
| `crates/nexus-coordinator-rs/src/rerun.rs` | L76-82 `DefaultHasher` → `blake3::hash()` deterministe |
| `crates/nexus-coordinator-rs/src/invite.rs` | L27-36 ajouter `MintRequest::new()` constructor |

### A.3 Criteres d'acceptation

- [ ] conn() est `pub(crate)` et compile sans erreur
- [ ] persist() erreurs loggees via tracing::warn
- [ ] CanaryInputManager a un seul Mutex<ReloadState> pour les
  3 champs mtime/check
- [ ] rerun simple_hash() est deterministe cross-process (test)
- [ ] MintRequest::new() existe et est utilise dans les tests
- [ ] URL single-quote grep retourne 0 match dans crates/
- [ ] LOC kickoff : plan S43 ne contient pas d'estimation LOC
  prospective (seulement retrospective — mesure gap Python source)

### A.4 Delta tests attendu

+2-4 (test rerun hash deterministe + test MintRequest::new au
minimum).

### A.5 Commit

```
feat(sprint43): Sprint 43 Phase A — MANDATORY batch 7 items
conn+persist+mutex+hash+mint+process
```

---

## §Phase B — Routes files + consent (Python → Rust)

### B.1 Scope

Porter `api/files.py` (323 LOC) et `api/consent.py` (255 LOC)
vers des handlers axum dans le daemon.

### B.2 Routes attendues

**files.py** :
- `GET /api/v1/files` — liste des fichiers
- `POST /api/v1/files/upload` — upload fichier
- `GET /api/v1/files/:hash` — download fichier par hash

**consent.py** :
- `GET /api/v1/consent` — GPU consent level courant
- `POST /api/v1/consent` — set GPU consent level

(A confirmer apres lecture du Python en Phase B — les routes
exactes sont derivees du code source, pas estimees.)

### B.3 Fichiers nouveaux/touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/files.rs` | NEW — handlers files |
| `crates/nexus-shell-daemon/src/consent.rs` | NEW — handlers consent |
| `crates/nexus-shell-daemon/src/http.rs` | +routes registration |
| `crates/nexus-shell-daemon/src/main.rs` | +mod declarations |

### B.4 Criteres d'acceptation

- [ ] Toutes les routes files.py reproduites en Rust
- [ ] Toutes les routes consent.py reproduites en Rust
- [ ] Tests integration HTTP pour chaque endpoint
- [ ] Pas d'unwrap() sur input utilisateur

### B.5 Delta tests attendu

+10-15 (integration HTTP per route, based on S42 pattern deploy=9
apps=8).

### B.6 Commit

```
feat(sprint43): Sprint 43 Phase B — files + consent API Rust
```

---

## §Phase C — Routes canary + contributor (Python → Rust)

### C.1 Scope

Porter `api/canary.py` (212 LOC) et `api/contributor.py` (141 LOC)
vers des handlers axum dans le daemon.

### C.2 Routes attendues

**canary.py** :
- `GET /api/v1/canary/status` — canary status
- `POST /api/v1/canary/observed` — observe canary result
- `GET /api/v1/canary/network-health` — network canary health

**contributor.py** :
- `POST /api/v1/contributors/register` — register contributor
- `GET /api/v1/contributors/:id` — contributor detail

(A confirmer apres lecture du Python en Phase C.)

### C.3 Fichiers nouveaux/touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/canary_api.rs` | NEW — handlers canary |
| `crates/nexus-shell-daemon/src/contributor_api.rs` | NEW — handlers contributor |
| `crates/nexus-shell-daemon/src/http.rs` | +routes registration |
| `crates/nexus-shell-daemon/src/main.rs` | +mod declarations |

### C.4 Criteres d'acceptation

- [ ] Toutes les routes canary.py reproduites en Rust
- [ ] Toutes les routes contributor.py reproduites en Rust
- [ ] Tests integration HTTP pour chaque endpoint
- [ ] Pas d'unwrap() sur input utilisateur

### C.5 Delta tests attendu

+6-10 (integration HTTP per route).

### C.6 Commit

```
feat(sprint43): Sprint 43 Phase C — canary + contributor API Rust
```

---

## §Phase D — Wrap-up

Livrable : `sprint43_verification.md` (28+ rows fail-fast) +
`sprint44_audit_plan.md` + update compteurs
HARDENING_ROADMAP/CLAUDE.md/SPRINT_LOG.md + migration S43 active →
archive/v1.2/.

---

## §Research consulte

- **axum** : deja dep daemon, meme pattern S42 (handlers, extractors,
  Router merge). Pas de recherche supplementaire.
- **BLAKE3** : deja dep workspace, utilise dans provenance.rs S42.
  Pas de recherche supplementaire.
- **tracing** : deja dep workspace, pattern warn! utilise partout.
  Pas de recherche supplementaire.
