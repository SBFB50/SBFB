# Sprint 22 Phase D — preflight G8

Date : 2026-04-20
HEAD : `df8a7a1`
Verdict : **SCOPE-CUT-CONSISTENT** (cohérent attendu plan §7.1)

Memory tip nexus_grid_pivot.md (`9676bd9`) en retard de 3 commits
sur HEAD : Phase C `cf3918c` + chore `dfd6222` + chore `df8a7a1`.
Compteurs tests fiables (rebased ci-dessous via §11 plan).

## Scans

### S1 — SOTA 2026 vs design

Lib ciblée : `nvml-wrapper` (workspace) bump `0.10` → `0.12.1`.

- Existence + date publication confirmées via crates.io API
  (`https://crates.io/api/v1/crates/nvml-wrapper`) — versions
  intermédiaires : `0.11.0` 2025-05-28, `0.12.0` 2026-02-11,
  `0.12.1` 2026-03-30. `last_seen_timestamp` API present
  (CHANGELOG 0.11.0 — gating utilization sample period).
  docs.rs/nvml-wrapper/0.12.1 OK (modules `device`, `enum_wrappers`,
  `struct_wrappers`, etc.).
- CHANGELOG diff `0.10.0..0.12.1` (WebFetch
  `github.com/rust-nvml/nvml-wrapper/blob/main/CHANGELOG.md`) :
  - `0.11.0` : **`memory_info` upgraded à `nvmlDeviceGetMemoryInfo`
    v2** ("to be consistent with nvidia-smi"). Pas listé comme
    breaking, mais utilisé déjà par `gpu/nvml.rs:81` snapshot. Risk
    valeurs reportées subtilement décalées (la v2 corrige certains
    bugs comptabilité Volta+). Mitigation : `cargo nextest -p
    nexus-worker-core` post-bump valide invariant
    `free + used = total` (test existant `snapshot_of_device_zero_
    returns_live_stats_on_hardware` ligne 217-218 gpu/nvml.rs).
  - `0.12.0` : `running_compute_processes` ajoute variante MPS
    (Volta+) — additive, n'altère pas la signature standard
    utilisée par NvmlProfile.
  - `0.12.1` : no changes.
- WebSearch `rustsec advisory nvml-wrapper 2026` : aucun avis.
- Verdict : **finding mineur** non-bloquant (semver-stable
  PATCH/MINOR, pas breaking listed) — surveiller test
  `snapshot_of_device_zero_returns_live_stats_on_hardware` post-
  bump.

### S2 — Decisions historiques traversées

```
git log --all --grep="nvml\|gpu_monitor\|gpu.theft\|compute.theft"
git log --all --grep="DEVIATION|rejected|scope-cut|deliberate"
  -- crates/nexus-worker-core/src/gpu/
grep -rE "DEVIATION|rejected|scope-cut" .planning/archive/v*/sprint*_*.md
  | grep -iE "nvml|gpu monitor|compute.theft"
```

- **Sprint 3 W4 `19ef014`** : `feat(worker): Sprint 3 W4 — GpuMonitor
  trait + NVML + Noop backends`. Module `crates/nexus-worker-core/
  src/gpu/{mod,noop,nvml}.rs` déjà en place :
  - `pub trait GpuMonitor { backend_name() / probe() / snapshot() }`
  - `pub struct NvmlBackend { nvml: Arc<Nvml> }` avec
    `pub fn inner(&self) -> &Nvml` ligne 64-66 — point d'entrée
    explicite "advanced callers that need features this wrapper
    does not cover yet (e.g. per-process memory usage)" (commentaire
    canonical ligne 61-63 nvml.rs).
  - Tests existants : 4 unit tests `#[cfg(test)]` gating runner GPU
    présence. Pattern factory `create_monitor() -> Box<dyn
    GpuMonitor>` mod.rs ligne 196-210.
- Reverse-commit check : `git log 19ef014..HEAD -- crates/nexus-
  worker-core/src/gpu/` ne montre **pas de revert** ni de
  rationale pour rejeter le module gpu/ (commits ultérieurs
  consomment GpuMonitor au runtime). Pas de DESIGN-CONFLICT.
- Plan §7.2 propose fichier flat `crates/nexus-worker-core/src/
  nvml_profile.rs` **sans référence** au module gpu/ existant ni
  à `NvmlBackend::inner()`. Lecture la plus charitable : oubli
  documentation (le plan a été écrit en parallèle de research G2,
  l'agent planner n'a pas grep le code existant).
- Memory feedback scan (`feedback_*.md`) : pas de règle "ne JAMAIS
  réutiliser gpu/ module" ni "toujours fichier flat per concern".
  Pattern par défaut côté `nexus-worker-core` = module dossier
  pour concern multi-fichiers (`engine/`, `gpu/`, `llm/`),
  fichier flat pour concern unique (`allowlist.rs`,
  `rate_limit.rs`).
- Verdict : **finding important non-bloquant**. Recommandation
  intégration : Option A créer `crates/nexus-worker-core/src/gpu/
  profile.rs` + `pub mod profile` dans `gpu/mod.rs` + réutiliser
  `NvmlBackend::inner()` pour partager `Arc<Nvml>` (évite double
  `Nvml::init` parallèle qui peut échouer si runner WSL2 sans
  driver complet). Cohérence module + factorisation init = win-win
  vs Option B (suivre plan littéral fichier flat).

### S3 — Threat model coverage

- `docs/security/HARDENING_ROADMAP.md` :
  - §3 ligne 252-253 (Sprint 22 — Sybil resistance composition 3
    couches + compute detection baseline + watermark primitive) :
    item `NVML util + duree profile worker-core, log-only
    baseline stats-only (foundation S24, pas anomaly detection)
    — ~300 LOC` ligne 280-281. Plan Phase D ~250 LOC = dans le
    budget.
  - Pipeline ligne 793-794 : `S22 NVML baseline profile ────────>
    S24 random re-run sampling ( C-ComputeTheft ) ( C-ComputeTheft
    detection )` confirme que Phase D est foundation **observation-
    only**, pas detection (S24 owns detection).
- `docs/security/THREAT_MODEL.md` §7 mitigations table ligne 85 :
  `C-ComputeTheft | Compute theft / mining | T3 | M | ⚠️ (caps S16-
  C) | M | NVML-profile`. Phase D livre la primitive listée.
- Threats T0-T5 mapping :
  - T3 Crypto-mining via GPU share (A-S4 ligne 67) : **partiellement
    couvert** déjà via consent caps S16-C (`should_accept_task`
    consent.rs). Phase D ajoute **observation baseline** (pas
    enforcement). Pas de regression sur S16-C.
  - T2/T4/T5 : non touchés par scope log-only stats-only.
- Pas de pre-requirement S22 ouvert restant : Phase A (rate-limit
  wire), Phase B (GLiNER), Phase C (Sybil 3 couches) déjà livrées.
  Foundation S24 = Phase D output unique.
- Verdict : **clean**. 0 regression flag, 0 pre-requirement
  manquant.

### S4 — Wire format / pre-launch invariants

- `grep -rE "_VERSION\s*[:=]\s*[0-9]+" crates/nexus-core-rs/src/`
  : aucun nouveau version field pour Phase D (NVML profile reste
  local SQLite, jamais publié wire).
- `crates/nexus-core-rs/src/canonical.rs` : non touché.
- `~/.sbfb/nvml_profile.db` (plan §7.2) : SQLite local, **pas un
  wire format protocole** (pas de gossip, pas de blob iroh, pas
  de coord HTTP). Schema interne = libre, no `*_VERSION` à
  protéger.
- D1..D5 S22 (kickoff §4) : Sybil composition / GLiNER decoder /
  agents_sudo integration / NVML scope log-only / watermark
  canari. Phase D consomme **D4 NVML scope log-only** verbatim.
  Pas de Day 0 rebattu.
- Memory `nexus_grid_pivot.md §Decisions actées` : non touchées
  (Phase D = code worker interne, hors P2P wire).
- Pre-launch protocol policy CLAUDE.md ligne 333+ : OK (pas de
  bump `*_VERSION`, pas de tolerant decoder multi-version, pas
  de `#[serde(default)]` ajouté, pas de `BLOB_VERSION` /
  `TASK_RESPONSE_VERSION` / `CANARY_VERSION` modifié).
- Verdict : **clean**.

## Findings (SCOPE-CUT-CONSISTENT)

1. **S1-1 — `memory_info` v1→v2 NVML wrapper 0.11.0** : surveiller
   test `snapshot_of_device_zero_returns_live_stats_on_hardware`
   (gpu/nvml.rs ligne 201-223) post-bump 0.10→0.12.1. Si red →
   ajuster invariant. Carry-over : **non requis**, résolvable
   inline Phase D si trigger.
2. **S2-1 — module `gpu/` pré-existe avec `NvmlBackend::inner()`
   point d'entrée explicite "advanced callers"** : décision
   placement fichier à prendre Phase D :
   - **Option A recommandée** : `crates/nexus-worker-core/src/gpu/
     profile.rs` + `pub mod profile` dans `gpu/mod.rs`. Réutiliser
     `NvmlBackend::inner()` pour partager `Arc<Nvml>` (évite double
     `Nvml::init`).
   - Option B : `crates/nexus-worker-core/src/nvml_profile.rs`
     flat (suit plan §7.2 littéral, double init NVML).
   - Option C : `crates/nexus-worker-core/src/gpu_profile.rs`
     flat (compromis nommage proche de gpu/).
   Carry-over : **non requis**, résolvable inline Phase D
   (déviation mineure scope-cut-consistent, à documenter dans
   commit body §Working tree audit + section "Code organization
   deviation").

## Action

Procède code Phase D — NVML baseline log-only foundation S24.

Note implémentation Option A intégration recommandée :
- `crates/nexus-worker-core/src/gpu/profile.rs` nouveau fichier
- `pub mod profile;` ajouté dans `crates/nexus-worker-core/src/
  gpu/mod.rs`
- Constructeur `NvmlProfile::new(backend: &NvmlBackend, db_path:
  PathBuf)` accepte référence backend pour réutiliser
  `Arc<Nvml>` via `inner()` clone. Fallback `NvmlProfile::
  try_new_standalone(db_path: PathBuf)` pour callers sans backend
  (init Nvml indépendant) — pattern factory similaire `gpu/
  mod.rs::create_monitor`.
- Tests +5 visés (plan §7.3) : new_creates_schema, sampling_
  persists_row, stats_for_window_empty, stats_for_window_
  computes_avg_p95, handles_no_gpu_gracefully. Pattern existant
  `#[cfg(test)] mod tests` avec runner-presence gate.
- Workspace bump `nvml-wrapper = "0.12.1"` (Cargo.toml ligne 99,
  remplace `"0.10"`).
- `~/.sbfb/nvml_profile.db` schema simple `nvml_samples
  (timestamp INTEGER, gpu_util INTEGER, vram_used_mb INTEGER,
  compute_processes_json TEXT)`. Pas de migration, schema first-
  run idempotent `CREATE TABLE IF NOT EXISTS`.
- Document Working tree audit §G5 + Option A déviation dans body
  commit `feat(sprint22): Phase D — NVML util+duree profile log-
  only baseline foundation S24`.

Aucun carry-over Sprint 23 recommandé depuis ce preflight.
