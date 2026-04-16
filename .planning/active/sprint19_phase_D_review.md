# Sprint 19 Phase D — nexus-phase-auditor review

**HEAD pre-commit** : `fe0a8fd`
**Draft commit title** : `feat(sprint19): Phase D — delayed upload queue (0-5min exponential jitter)`
**Auditor** : nexus-phase-auditor (session 2026-04-16)
**Timebox** : 8 min

---

## Verdict : PASS (après fix P2-1 + P2-2 apportés inline)

0 P0, 0 P1, 2 P2 identifiés et **résolus inline avant commit** (Option A
recommandée par l'auditeur). Signal rigor G4 satisfait (2 findings P2+
documentés). Commit autorisé.

---

## Dimensions

### Security

- **Semgrep / secrets patterns** : aucun pattern secret (`AKIA`, `ghp_`,
  `sbfb_*`), aucune credential hardcodée. PASS.
- **Path traversal** : `db_path` construit comme `self.project_dir /
  "upload_queue.sqlite"` via `project_dir()` coord-internal non-user-
  driven. Aucun vecteur traversal. PASS.
- **aiosqlite sans timeout explicite** : le fichier est exclusif au
  coordinator (single-writer, WAL). PASS pre-launch.
- **Loopback / PeerCredsVerified** : `/tasks/submit` protégé par
  middleware P27 pré-existant. Le diff s'insère après l'auth. PASS.
- **SPDX headers** : tous fichiers nouveaux carrying `# SPDX-License-
  Identifier: AGPL-3.0-or-later`. PASS.
- **QueueFullError → HTTP 429** : header `Retry-After: 30` cohérent
  avec `flush_interval_s=30` default. PASS.
- **Idempotency dispatcher** : double defense (SELECT existant → early
  return + INSERT OR IGNORE fallback) empêche double-emit. PASS.

### Patterns

- **P27 loopback** : non touché. PASS.
- **P15 DB migration runner** : couvre SDK MigrationRunner (apps), pas
  les tables infra coordinator. Non-applicable. PASS.
- **P29 (introduit par ce diff)** : conforme au code livré — SQLite
  WAL, exponential jitter, CSPRNG, injection clock/rng, tech debt
  T-S19-D-1/2/3 loggée. PASS.

### Working tree audit (G5)

| Fichier | Catégorie | Verdict |
|---|---|---|
| `docs/shell/PATTERNS.md` | PHASE (plan §7) | ✓ |
| `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py` | PHASE | ✓ |
| `packages/nexus-coordinator/src/nexus_coordinator/config.py` | PHASE | ✓ |
| `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py` | PHASE | ✓ |
| `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py` | PHASE | ✓ |
| `packages/nexus-coordinator/tests/test_upload_queue.py` | PHASE | ✓ |
| `packages/nexus-coordinator/tests/test_api_tasks_delayed.py` | PHASE | ✓ |
| `packages/nexus-coordinator/tests/test_dispatcher.py` | PHASE (ajout `upload_queue.enabled=False` sur 4 tests existants) | ✓ |

9 fichiers PHASE staged, 0 CRAFT, 0 DEBT, 0 NOISE staged. Untracked
NOISE (cc.json, site/, node_modules/, test_libc.*, docs/DND/VISION/
apps/, .claude/settings.local.json/worktrees/) confirmés hors-scope
pre-launch (kickoff §10).

### Scope-cuts

Grep systématique sur les items kickoff §6 dans le diff :
- `encryption.at.rest`, `duress.pin`, `rate.limit.sliding`,
  `kudos.weighted`, `structured.output`, `client.side.redaction`,
  `ML-DSA`, `ML-KEM`, `domain.fronting`, `Tor.bridge` : **aucun hit**.
PASS.

### Tests-delta

| Suite | Annoncé | Réel mesuré | Verdict |
|---|---|---|---|
| Coord pytest | +14 initial → **+15 après fix P2-1** | 13 primitive + 2 integration | ✓ |
| Rust workspace | 537 unchanged | 537 (0 Rust touché) | ✓ |
| SDK | 185 unchanged | 185 | ✓ |
| app-gov | 46 unchanged | 46 | ✓ |

15 tests livrés : 13 primitive (`test_schedule_*` ×3, `test_flush_due_*`
×2, `test_scheduler_loop_wakes_on_interval`, `test_shutdown_drains_
pending`, `test_concurrent_schedule_all_land`, `test_hard_cap_enforced_
under_concurrency` **nouveau — P2-2 fix**, `test_hard_cap_raises_queue_
full`, `test_start_rerandomizes_stale_rows`, `test_disabled_passthrough_
skips_db`, `test_bucket_partitions_delay_range`) + 2 integration API
(`test_api_submit_pipes_through_queue_and_eventually_lands`,
`test_api_submit_returns_429_past_hard_cap`). Zéro skip/ignore.

### Research-grounding

- `aiosqlite` + `structlog` : deps pré-S19, usage existant (kudos_
  ledger, state.sqlite). PASS.
- asyncio.create_task / Lock / Event : stdlib CPython 3.13, cités
  dans design doc §8.4 via context7. PASS.
- `secrets.SystemRandom` : stdlib CSPRNG. PASS.
- Distribution exponentielle : Cornell ESORICS 2006 + Loopix 2017
  cités dans design doc §3.2-3.3 + §2.2 avec liens. PASS.
- SQLite WAL : `tech-insider.org 2026` + `persist-queue` +
  `plainjob` dans design doc §8.4. PASS.
- Aucune nouvelle dep Cargo / pyproject / package.json. PASS.

### Horizon long-terme + documentation amont (§6.7)

- **Design doc présent** : `.planning/research/S19_phase_D_delayed_
  upload_queue_design.md` (1196 lignes, date 2026-04-16, pre-Phase D).
  PASS.
- **Alternatives rejetées citées** : design §3 couvre 5 distributions
  (uniforme, exponentielle retenue, Poisson Tor-style, Fixed pool
  Mixmaster, Adaptive) avec rationale rejet. PASS.
- **Solution la plus poussée** : upgrade plan "in-memory" → SQLite
  WAL documenté PATTERNS.md P29 avec rationale crash-safety (design
  §5.2). Loopix différé S25+ tracé tech debt. PASS.
- **Aucune estimation LOC au plan** : cleanup codifié dans
  `chore(planning) fe0a8fd` (§6.7 no-LOC). Les mentions "~345 LOC"
  dans PATTERNS.md P29 sont descriptives rétrospectives (pattern
  conforme P27/P28). PASS.

---

## Findings (résolus inline)

### P2-1 (résolu) — TOCTOU dans `schedule()` : cap check + INSERT pas atomiques

**Fichier** : `upload_queue.py:174-195` (avant fix)

**Description** : la docstring de `UploadQueue` affirmait *"the size
check + INSERT stays atomic against the flush loop's DELETE"*, mais
le code appelait `self._size()` ligne 176 **hors** du `_lock` acquis
ligne 197. L'`await self._size()` est un point de cession asyncio :
deux coroutines concurrentes (via `asyncio.gather`) pouvaient toutes
deux passer le check `size >= hard_cap` avec la même valeur avant
qu'une seule n'insère — le hard_cap pouvait être dépassé de `N-1`
lignes pour `N` submits gatherés au bord du cap.

**Fix appliqué** (Option A recommandée par auditeur) : cap check +
INSERT déplacés à l'intérieur du même `async with self._lock` bloc.
Un seul `aiosqlite.connect()` partagé dans la section locked fait le
SELECT COUNT(*) puis l'INSERT de façon serialisée.

```python
async with self._lock:
    async with aiosqlite.connect(self.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            row = await cursor.fetchone()
        size = int(row[0]) if row else 0
        if size >= self.hard_cap:
            raise QueueFullError(...)
        # ... INSERT
```

Docstring corrigée pour refléter la garantie réelle : *"the cap
check + INSERT stays atomic against itself (two gather()ed schedules
cannot both pass a near-the-cap check with the same size snapshot)
and against the flush loop's DELETE"*.

### P2-2 (résolu) — Couverture test manquante : cap enforcement sous concurrence

**Fichier** : `tests/test_upload_queue.py`

**Description** : `test_concurrent_schedule_all_land` utilisait les
caps par défaut (soft=10 000, hard=100 000) et ne pouvait donc pas
détecter la violation P2-1 à 50 submits concurrents.

**Fix appliqué** : ajout de `test_hard_cap_enforced_under_concurrency`
qui lance 20 schedules gatherés contre `hard_cap=5` et assert
exactement 5 succès + 15 `QueueFullError` + 5 rows en SQLite. Ce
test est une régression directe pour P2-1 — si le lock est retiré,
le test échoue en observant 6–20 rows au lieu de 5.

---

## Recommendation

**Commit autorisé.** Les 2 P2 sont résolus **avant** le commit (pas
reportés en carry-over S20), donc pas d'entrée requise dans
`sprint19_audit_plan.md §Phase D findings carry`. Le body commit
reflète le delta test révisé (+15 au lieu de +14 annoncé initialement)
et mentionne le fix P2-1 explicitement.

**Ordre de commit final** :
1. Le diff staged contient le fix P2-1 + test P2-2 (mesuré 15 tests
   passés localement).
2. Commit autorisé sous `feat(sprint19): Phase D — delayed upload
   queue (0-5min exponential jitter)`.

---

## Suites §7.4

- Rust : `cargo test --workspace --locked` → 537 passed (0 touched). ✓
- Coord pytest : 207+3 pre-Phase-D → **208+3 post-fix-P2-1** (15 tests
  Phase D livrés). ✓
- SDK pytest : 185 unchanged. ✓
- app-gov pytest : 46 unchanged. ✓
- Ruff format + check : packages/ clean. ✓

---

**Fichiers référencés** :
- `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py`
  (fix P2-1 appliqué dans `schedule()`, lignes 170-210 post-fix)
- `packages/nexus-coordinator/tests/test_upload_queue.py`
  (test P2-2 ajouté, `test_hard_cap_enforced_under_concurrency`)
- `.planning/research/S19_phase_D_delayed_upload_queue_design.md`
  (design doc 1196 lignes, pre-Phase D 2026-04-16)
- `docs/shell/PATTERNS.md` (P29 Delayed upload queue, section
  Sprint 19 patterns)
