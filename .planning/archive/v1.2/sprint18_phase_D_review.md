# Sprint 18 Phase D — nexus-phase-auditor review

**HEAD pre-commit** : `9d0ad7a` (Phase C). 13 fichiers M/A staged, scope restreint au launcher + shell-daemon-core + shell-daemon tests + coordinator + SDK + app-gov + 2 fichiers Rust nouveaux.
**Draft commit body** : `feat(sprint18): Phase D — coord-side TaskEntry wire-through + X-SBFB-Token rotation`.
**Audit timebox** : ~38 min.

---

## Verdict : PASS

0 finding P0. 0 finding P1. 0 finding P2. 4 findings P3.

Commit autorise. Les findings P3 sont des nits documentaires (drift plan→livrable documente, noms de fichiers de test, decision RwLock vs Mutex justifiee en tete de module).

---

## Dimensions

### Security

- **Semgrep** : 0 finding sur les fichiers Rust du diff (`.semgrep/sbfb.yml` regles). Le Python est hors-cible Semgrep.
- **`unsafe`** : le seul bloc `unsafe` du launcher est pre-existant (`main.rs:318`, `libc::kill(child.id() as i32, SIGTERM)` sous `#[cfg(unix)]`). Aucun nouveau bloc introduit par Phase D. `#![forbid(unsafe_code)]` maintenu dans les autres crates touches.
- **`unwrap()` / `expect()` en production** :
  - `TokenRotator::load` utilise `unwrap_or_else(Instant::now)` sur `checked_sub` — defensif, non-paniquant.
  - `unix_now()` utilise `unwrap_or(0)` sur `duration_since(UNIX_EPOCH)` — defensif.
  - Tous les `unwrap()` / `expect()` restants sont dans des blocs `#[cfg(test)]`. Conforme.
- **Secrets** : aucun pattern `AKIA`, `ghp_`, `pat_`, `sbfb_[a-z]+_[a-zA-Z0-9]{20,}` detecte.
- **Loopback hardening** : Phase D ajoute un primitif (`TokenRotator` + `validate_token_with_rotator`) sans modifier le middleware `auth_required`. Le gate `PeerCredsVerified` reste intact. La wire-up daemon est un carry-over explicite — l'overlap window n'est pas encore utilisee par les requetes reelles. Aucune regression de surface.
- **Wire format / JCS** : `dispatcher.py:107` — `json.dumps(task_dict, sort_keys=True)` est le chemin canonique pour les 4 champs Phase D (`is_open_source`, `estimated_watts`, `estimated_vram_mb`, `estimated_hours`) avant la signature `nexus_core.sign_task`. Le `Task` Rust a deja ces champs depuis S16 avec `#[serde(default)]` — deserialization cross-process preservee. Conforme.
- **Atomic write `tokens.json`** : `TokenRotator::write_atomic` fait `fs::write(tmp)` puis `fs::rename(tmp, path)`, pattern tempfile+rename correct. Mode `0o600` sur le fichier, `0o700` sur le parent, via `set_mode` sous `#[cfg(unix)]`. Sur Windows le fichier est cree sans restriction explicite — equivalent a `auth_token` existant.
- **Path traversal `tokens.json`** : `tokens_file_path()` derive son chemin de `sbfb_home()` (env `SBFB_HOME` ou home OS) + literal `"tokens.json"`. Aucune entree user-controlled dans le chemin. `path.with_extension("tmp")` derive du meme chemin fixe.
- **TokensFile serde** : `#[serde(deny_unknown_fields)]` present — conforme au pattern G-3 etabli en Sprint 8.
- **Client cannot override `is_open_source` / estimates** : le test `test_api_ignores_client_attempt_to_set_is_open_source` prouve explicitement l'invariant S16 D-1. Le `TaskCreateBody` Pydantic ignore les champs inconnus par defaut, et le handler re-derive server-side exclusivement. Regression-proof.
- **Constant-time comparison** : `validate_token_with_rotator` delegue a `constant_time_eq` existant pour chaque slot (current et previous). Pas de fuite via timing differential.

### Patterns

`docs/rust/PATTERNS.md` est un carnet d'apprentissage sans patterns numerotes formels. `docs/shell/PATTERNS.md` contient P1..P7 (frontend + coordinator).

- **Error handling (`thiserror` vs `anyhow`)** : `TokenRotator::write_atomic` et `TokenRotator::load` retournent `std::io::Result`. `spawn_rotation_loop` utilise `tracing::warn!` pour les erreurs de persist et continue — comportement acceptable pour un background loop (ne pas kill le rotator sur une erreur disk transitoire). Pas d'`anyhow` introduit dans la lib.
- **Async/tokio** : `spawn_rotation_loop` utilise `tokio::spawn(async move {...})` avec `ticker.tick().await`. Le `Arc<RwLock<TokenRotator>>` est `Send + 'static`. Premier tick `interval` skippe pour ne pas invalider le token du daemon a T+0. Conforme.
- **`RwLock` vs `Mutex`** : le plan §502 specifiait `Arc<Mutex<TokenRotator>>`. Le diff livre `Arc<RwLock<TokenRotator>>`. Upgrade justifie (N lecteurs concurrents, 1 ecrivain) et documente en tete de `token_rotation.rs`. Pattern drift P3 (plan non-mis-a-jour).
- **`SbfbHomeGuard` + `env_lock`** : le pattern `EnvSnapshot + Mutex ENV_GUARD` (carry-over P2 Phase C pour documentation dans `docs/rust/PATTERNS.md`) est etendu ici avec une fusion crate-wide : `test_util::env_lock()` dans `main.rs` partage le meme `OnceLock<Mutex>` entre `auth::tests` et `token_rotation::tests` pour empecher la race sur `SBFB_HOME`. Correction legitime trouvee au premier run full-workspace.
- **`deny_unknown_fields`** : `TokensFile` respecte le pattern G-3 (Sprint 8). Le nouveau champ `identity.repo_url` dans `CoordinatorConfig` est `Optional[str] = None` — pas de breakage `tomllib` pour les configs existantes (le champ manquant deserialize a None).
- **`cost_estimate()` sur `NexusApp`** : livree comme methode concrete avec default `(100, 2000, 0.1)` plutot que `@abstractmethod` comme le plan §523 le stipulait. Decision superieure : pas de breakage backward pour les apps existantes sans override. P3 (drift plan→livrable documente).

### Scope-cuts

Grep exhaustif des fichiers du diff contre chaque item §6 du kickoff S18 :

| Scope cut §6 | Grep result |
|---|---|
| PoW gossip | absent |
| TLS cert pinning relays | absent |
| Encryption at rest keypair | absent |
| Iroh audit externe | absent |
| Pyodide sandbox escape | absent |
| ML-DSA / ML-KEM / PQC | absent |
| Self-hosted pkarr relay | absent |
| Federated ONG-run relays concrets | absent |
| NVIDIA CVE / NVD check (Phase E1) | absent |
| Warrant canary (Phase E2) | absent |
| Radicle mirror (Phase E3) | absent |
| THREAT_MODEL.md cross-ref S17 | absent |

**Zero scope creep.**

**Sur la non-integration daemon router (carry-over revendique)** : le plan §529 listait `crates/nexus-shell-daemon/src/loopback.rs` comme livrable conditionnel pour le switch `AuthState::new` → `Arc<RwLock<TokenRotator>>`. L'executeur reporte explicitement cette wire-up car elle requiert un file-watcher `notify` sur `tokens.json` cote daemon (pour relire les rotations ecrites par le launcher). Scope +50 LOC + risques de flakiness sur le watcher — jugement acceptable pour isoler dans un commit dedie Phase F ou Sprint 19. Le primitive `validate_token_with_rotator` est publiquement expose et entierement couvert par 3 tests daemon (`crates/nexus-shell-daemon/tests/loopback_token.rs`).

### Tests-delta

| Source | Annonce | Realite |
|---|---|---|
| Plan §Tests Phase D | +15 total | — |
| Plan §Commit Phase D | "+7 coord/SDK pytest + +8 Rust tests" | — |
| Draft commit body | +16 (explication +1 bonus) | +16 confirme |
| SDK (`test_sdk.py`) | +2 | +2 (cost_estimate_default, cost_estimate_override) |
| Coord (`test_dispatcher.py`) | +5 plan, +6 livre | +6 (defaults_closed, writes_verbatim, api_derives_from_repo_url, api_uses_registered_app, api_falls_back_when_missing, api_ignores_client_override) |
| Rust launcher (`token_rotation.rs`) | +5 | +5 (rotates_after_interval, keeps_previous, discards_after_overlap, concurrent_safe, persists_atomically) |
| Rust daemon (`loopback_token.rs`) | +3 | +3 (accepts_current, accepts_previous_overlap, rejects_previous_after_overlap) |
| **Total** | **+15** | **+16** |
| `cargo test --workspace --locked` | 450 → 458 | +8 Rust confirme |
| `uv run pytest sdk` | 183 → 185 | +2 confirme |
| `uv run pytest coord` | 187+3sk → 193+3sk | +6 confirme |
| `uv run pytest app-gov` | 46 → 46 | inchange (cost_estimate override ne casse aucun test gov existant) |

**Match +16 realite vs +16 annonce body.** Le +1 bonus par rapport au plan §548-565 est `test_api_falls_back_to_sdk_defaults_when_app_missing` — logical consequence du handler fallback, cite dans le body en "+6 vs plan +5". En faveur du livrable. Non-bloquant.

**Suites hors-scope** : Vitest (+0), Playwright (+0), size-limit (+0), SPDX (+0) — aucun de ces composants n'est touche par Phase D. Conforme.

---

## Findings

### P3 — `cost_estimate()` livree concrete vs `@abstractmethod` du plan

Le plan §523 disait `abstract method`. Le diff livre une methode concrete avec default `(100, 2000, 0.1)`. La decision livree est semantiquement superieure (pas de breakage backward sur les apps existantes sans override — GovApp reste le seul a override actuellement, ColdCase/Forensics inexistent dans le workspace).

**Action recommandee** : mettre a jour le plan pour refleter la decision. Non-bloquant. Fichier : `packages/nexus-sdk/src/nexus_sdk/app.py:357`.

### P3 — Noms de fichiers de test dans le plan incorrects

Le plan §539 cite `test_tasks.py` pour les tests coord Phase D ; les tests ont atterri dans `test_dispatcher.py` (fichier pre-existant, naming coherent avec `Dispatcher.submit`). Le plan §548 cite `test_app.py` pour SDK ; les tests ont atterri dans `test_sdk.py` (fichier pre-existant). Divergence purement documentaire.

**Action recommandee** : aucune, le plan est archive apres le sprint. Non-bloquant.

### P3 — `concurrent_rotation_safe` sans assertion explicite sur les readers

Le test `token_rotation::tests::concurrent_rotation_safe` spawne 8 readers qui appellent `validate_token_with_rotator` et ignorent le resultat (commentaire inline explique que l'invariant strict est couvert par les tests single-threaded). La valeur du test est la detection de deadlock / panic sur 50 rotations concurrentes × 200 probes = 10400 accesses, pas la correctude de validation.

**Action recommandee** : OK en l'etat. Un commentaire plus explicite "readers ne paniquent pas = l'assertion" ameliorerait la lisibilite mais n'est pas requis. Fichier : `crates/nexus-launcher/src/token_rotation.rs:220-270`.

### P3 — Drift plan `Mutex` → code `RwLock`

Le plan §502 specifiait `Arc<Mutex<TokenRotator>>`. Le diff livre `Arc<RwLock<TokenRotator>>`. Justification documentee en tete de `token_rotation.rs:5` (upgrade legitime : N lecteurs, 1 ecrivain).

**Action recommandee** : aucune. Le choix livre est techniquement superieur et documente. Non-bloquant.

---

## Verifications effectuees

| Check | Resultat |
|---|---|
| `cargo test --workspace --locked` | 458 passed (+8 vs baseline 450) |
| `cargo fmt --all --check` | exit 0 (apres auto-fix sur `token_rotation.rs`) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `uv run ruff format --check packages/` | 101 files already formatted |
| `uv run ruff check packages/` | All checks passed |
| `uv run pytest packages/nexus-sdk/tests/ -q` | 185 passed (+2) |
| `uv run pytest packages/nexus-coordinator/tests/ -q` | 193 passed 3 skipped (+6 coord) |
| `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed (inchange) |
| Grep secrets (AKIA, ghp_, pat_) | 0 match |
| Grep scope cuts §6 kickoff | 0 match sur les 12 items |
| `unsafe` blocks introduits | 0 (le bloc `libc::kill` existe deja pre-Phase D) |
| Test count reconciliation (5+6+2+3=16) | match annonce body +16 |
| Wire format JCS (json.dumps sort_keys=True) | present dispatcher.py:107 |
| Path traversal `tokens.json` | nom fichier constant, pas de traversal possible |
| Atomic write `tokens.json` | tempfile + rename, perm 0600 / dir 0700 sous Unix |
| `#[serde(deny_unknown_fields)]` sur nouveaux types | present sur `TokensFile` |
| Invariant S16 D-1 (client ne peut pas forger `is_open_source`) | regression-proof via `test_api_ignores_client_attempt_to_set_is_open_source` |

---

## Recommendation

**Commit autorise.** 0 finding P0/P1/P2. Les 4 findings P3 sont des nits non-bloquants (drift plan→livrable justifie, noms de fichiers de test, RwLock-vs-Mutex documente).

Avant de committer, le body draft mentionne deja :

- Wire-up TokenRotator daemon router + file-watcher (carry-over Phase D → F/S19).
- Override `cost_estimate()` sur ColdCase + Forensics quand/si ces apps reviennent dans le workspace.

Inutile d'ajouter des P3 au body — ils sont non-actionables cross-sprint.

Carry-overs herites Phase C a tracker en Phase F wrap-up :

- Wire `nexus-shell-daemon` browse aggregator sur `redundant_resolve` + vrais pkarr resolvers.
- Factoriser `sbfb_home()` — 3 occurrences Rust (`consent.rs`, `auth.rs`, `relay_config.rs`) + 1 Python (`coordinator.auth`) — dans `nexus-core-rs`.
- Log `home_relay=` URL active dans `node.rs` pour diagnosabilite operateur.
- Documenter pattern `EnvSnapshot + ENV_GUARD` dans `docs/rust/PATTERNS.md`.
- Document pattern bash scripts (Phase B).

Gate 1 threat-model (roadmap S18 §kickoff) : prerequis 4/4 cleared cote launcher/loopback (S16 A-D + S18 D). Phase E1 (NVIDIA CVE) + Phase F (wrap-up) restent pour fermer la v1.2 Sprint 18.
