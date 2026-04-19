# Sprint 21 Phase D — preflight G8

Date : 2026-04-19
HEAD : `17035c3`
Verdict : **SCOPE-CUT-CONSISTENT**

Phase cible : **Quarantine queue SQLite WAL + manual flush CLI**
(plan §7 lignes 536-595, kickoff §D4 lignes 571-637).

---

## 1. Résumé verdict

Day 0 D4 préservée intégralement (schema 8-colonnes, TTL 15 min
auto-drop, CLI `sbfb quarantine list|flush|drop`, auth bearer
X-SBFB-Token + Host + Origin pattern S16, persistance SQLite
local). Quatre findings non-bloquants détectés sur le plan
§7.2 implementation paths — réalignement au pattern coord-Python
existant (cohérent `f238d31` que le plan dit réutiliser). Procéder
Phase D **après** un commit `chore(planning)` préalable qui
réaligne §7.2 + §7.1 et livre le design doc `S21_phase_D_
quarantine_design.md` réclamé en pré-requis explicite.

---

## 2. Scans

### S1 — SOTA 2026 vs design

Libs/deps scannés : aucune nouvelle dépendance externe introduite
par Phase D. Stack 100% recyclée :

- `aiosqlite>=0.20` (déjà déclaré `pyproject.toml:57`, validé S19 D
  `upload_queue.py`)
- `fastapi>=0.111`, `typer>=0.12` (déjà déclarés)
- `nexus-core-py` (déjà bind, pas d'API touchée)
- SQLite WAL pragma `PRAGMA journal_mode = WAL` (mode batt-tested
  S19 D, support natif SQLite ≥ 3.7.0 — Python 3.13 stdlib bundles
  SQLite 3.45)

CVE / advisory check : pas de zone rouge applicable
(R-wasmtime-cve / R-iroh-audit / R-libcrux-hax / R-pyodide-escape
hors-scope coord-Python SQLite quarantine).

**Verdict S1 : clean.**

### S2 — Décisions historiques traversées

Commandes lancées :

```bash
git log --all --oneline --grep="quarantine\|upload_queue\|hold queue"
git log --all --oneline --grep="DEVIATION\|rejected\|scope-cut\|sqlite\|SQLite" -- packages/nexus-coordinator/
```

**Décisions historiques pertinentes** :

- `f238d31` (S19 Phase D) — **delayed upload queue** établit le
  pattern coord-Python `aiosqlite` WAL + `class QueueFullError`
  + tokio-style task scheduler. Source explicite réutilisée par
  D4 kickoff §D4 ligne 599-600 « Pattern réutilise S19 Phase D
  `f238d31` `upload_queue.py` (validation production delayed
  upload queue) ».
- `b34d451` (S21 open) — `chore(planning): open Sprint 21` figé
  Day 0 D4 schema + TTL 15 min + CLI + auth pattern S16. **Aucun
  rejet historique** sur quarantine queue (zero precedent SBFB).
- Kickoff §D4 lignes 602-614 liste les 4 alternatives **rejetées**
  (in-memory BTreeMap volatil / gossip-wide diffusion DoS /
  Redis externe infra-creep / Admin UI custom UX-creep) — toutes
  préservées dans Phase D scope. Aucune réintroduction.

**Findings drift plan-vs-code (non-bloquants)** :

#### Finding S2-D1 — `crates/nexus-shell-daemon/src/api/quarantine.rs` n'existe pas

Plan §7.2 ligne 554 spécifie ce path. Inventaire daemon réel :
`crates/nexus-shell-daemon/src/{cli, http, logging, main, named_
pipe_server, noop_identity, panic, runtime, uds_server}.rs`.
**Aucun sous-module `api/`**. Le daemon agrège les routes REST
dans `http.rs` (cf. `/panic/wipe` route `crates/nexus-shell-
daemon/src/http.rs` registered via `.route("/panic/wipe", post
(panic_wipe))`). Le pattern `api/<resource>.rs` cité par le plan
n'a jamais existé daemon-side.

**Reverse-commit check** : `git log --all --grep="quarantine\|api"
-- crates/nexus-shell-daemon/src/api/` → vide. Aucun commit n'a
créé puis supprimé ce sous-module. Le path est purement
**aspirational** dans le plan §7.2.

Classification : drift **naming/path post-Day-0** (Day 0 D4 ne
prescrit pas le langage du REST endpoint, juste « endpoint
loopback `/quarantine/*` avec auth bearer pattern S16 »).

#### Finding S2-D2 — `crates/nexus-launcher/src/cli.rs` n'existe pas

Plan §7.2 ligne 557-558 spécifie ce path. Inventaire launcher
réel : `crates/nexus-launcher/src/{auth, driver_check, main,
token_rotation, unlock}.rs`. **Aucun `cli.rs`**. Le launcher
parse les sous-commandes via `unlock::Subcommand::{Init,
InitDuress, Unlock}` dispatché dans `main.rs`. Pas de pattern
CLI agrégateur multi-resource.

Le launcher est volontairement **minimal** (init/unlock keypair
flow uniquement) selon décision `nexus-launcher` Sprint 13. Tout
le reste (canary CLI, REST endpoints applicatifs) vit côté
coordinateur Python.

**Reverse-commit check** : `git log --all -- crates/nexus-
launcher/src/cli.rs` → vide. Aucun cli.rs n'a jamais été créé.

Classification : drift **naming/path post-Day-0**.

#### Finding S2-D3 — Plan §7.2 contredit son propre rationale « réutilise pattern S19 D `f238d31` »

Le pattern `f238d31` est **100% coord-Python** :

- `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py`
  → `class UploadQueue` + `aiosqlite` + WAL + `QueueFullError`
  + `db_path = self.project_dir / "upload_queue.sqlite"`
- `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py:
  98-116` consume `coord.upload_queue.schedule(submit_req)`
- `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py:
  72,294-307` instancie + démarre via `await self.upload_queue.
  start()`

Aligner Phase D sur **le pattern réellement réutilisé** :

| Plan §7.2 (drift) | Réalignement coord-Python |
|---|---|
| `crates/nexus-shell-daemon/src/api/quarantine.rs` (Rust REST + proxy vers coord) | `packages/nexus-coordinator/src/nexus_coordinator/api/quarantine.py` (FastAPI router direct, pattern `api/canary.py`) |
| `crates/nexus-launcher/src/cli.rs sbfb quarantine` | `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/quarantine.py` (Typer command, pattern `cli/commands/invite.py`) |

Côté daemon : aucun module supplémentaire requis. Le CLI Typer
appelle les endpoints loopback FastAPI via `httpx>=0.27` (déjà
dep) — pattern identique à `sbfb canary publish/ack` documenté
`packages/nexus-coordinator/src/nexus_coordinator/api/canary.py:
11`.

Classification : **incohérence interne plan §7.2 ↔ pattern reuse
explicite §D4 ligne 599-600**. Réalignement restaure cohérence
sans rebattre Day 0 D4.

#### Finding S2-D4 — Design doc pré-requis manquant

Plan §7.1 ligne 540 + Kickoff §D4 ligne 634-637 **réclament
explicitement** `.planning/research/S21_phase_D_quarantine_
design.md` couvrant :

- Schema SQLite evolution
- TTL clock semantics (received_at vs inserted_at, NTP sync)
- Security manual flush (bearer X-SBFB-Token + Host + Origin
  pattern S16)
- Interaction gossip layer (PoW gate at flush time vs pre-hold)
- Expected cardinality + benchmarks (~1000 msg/min/15min TTL
  estimate)

**Inventaire `.planning/research/`** : présents `S19_phase_*`,
`S20_phase_*`, `S21_phase_B_iframe_pii_sdk_design.md`, `S21_phase_
C_output_filter_design.md`. **Absent : `S21_phase_D_quarantine_
design.md`**. Phase A et C avaient leur design doc préalable —
Phase D doit l'avoir aussi pour cohérence procédurale.

Classification : **pré-requis procédural manquant**. À livrer
dans le commit `chore(planning)` de réalignement avant le commit
`feat` Phase D.

**Verdict S2 : 4 findings non-bloquants** (drift naming + 1
pré-requis doc).

### S3 — Threat model coverage

HARDENING_ROADMAP `docs/security/HARDENING_ROADMAP.md` §3
ligne C-DosFlood S21 (impact 4 / likelihood 4 / detect 3 /
risque 5.3) :

> Sprint 21 : rate-limit per-(consumer, worker, model) débloqué
> par PoW wire Phase C + client-side redaction SDK.

Triangle de défense couvrant C-DosFlood :

1. **Rate-limit** sliding-window multi-tier — Phase A `63afe4e`
   (governor 0.10.2 GCRA worker-engine gate R1 livré)
2. **PoW gossip subscribe** — Sprint 20 Phase C `16b94ba`
   (`pow_policy_loader.rs` hot-reload, wire au runtime
   `spawn_gossip_subscribe_task`)
3. **Quarantine hold + manual flush** — Phase D (cette phase)

Pre-requirements vérifiés présents au tip `17035c3` : ✅ (1+2
livrés). Phase D ferme le triangle defense-in-depth.

**Régression check** :

- T0 (network spam DoS) : rate-limit ✅, PoW ✅, quarantine
  ajoute audit trail post-mortem ✅. Pas de régression.
- T1 (Sybil) : pré-requis kudos Sybil-resistant (S22+) — **gap
  documenté hors-scope Phase D**, pas une régression.
- T2 (extraction) : pas applicable Phase D.
- T3 (eclipse) : pas applicable Phase D.
- T4 (data leak) : SQLite local `~/.sbfb/quarantine.db` contient
  `payload_bytes` raw. **Considération mineure** : pas de PII
  filter ici (Phase B+C s'en sont occupé in-flight). Acceptable
  car local-only + perm 0600 (pattern S16/S20 keystore).
- T5 (compromise) : pattern auth bearer + Host + Origin pattern
  S16 = identique à `/panic/wipe` audit-clean S20.

**Aucune régression introduite.**

**Verdict S3 : clean.**

### S4 — Wire format / pre-launch invariants

Scan `_VERSION` :

```
crates/nexus-core-rs/src/schemas/mod.rs:
  //! ## `*_VERSION = 1` pre-launch protocol policy
```

`BLOB_VERSION = 0x01`, `TASK_RESPONSE_VERSION = 1`, `CANARY_
VERSION = 1`, `ANNOUNCEMENT_VERSION = 1` — **tous inchangés**.

Phase D introduit :

- Schema SQLite local `~/.sbfb/quarantine.db` table `quarantine_
  messages` — **purement local, hors wire protocol P2P**.
- Format wire : aucun (le coordinator ne diffuse pas le contenu
  quarantine, seulement hold + flush manuel via CLI loopback).
- `#[serde(default)]` ajouts : aucun prévu (pas de struct wire
  touchée).

Day 0 D4 vérification ligne par ligne (kickoff §D4 lignes 575-
597) :

- ✅ Schema 8-colonnes (id / topic / sender_pubkey BLOB /
  payload_bytes BLOB / received_at_epoch_s INTEGER / rate_strikes
  INTEGER / pow_status TEXT / flush_status TEXT) **préservé**
- ✅ TTL automatique 15 min `received_at_epoch_s < now - 900`
  **préservé**
- ✅ CLI `sbfb quarantine list|flush|drop` **préservé** (sera
  livré via `nexus-coordinator quarantine ...` Typer command —
  voir Finding S2-D3 réalignement)
- ✅ REST `/quarantine/list`, `/quarantine/flush/{id}`,
  `/quarantine/drop/{id}` auth bearer X-SBFB-Token + Host +
  Origin pattern S16 **préservé**
- ✅ Pattern S19 D `f238d31` `upload_queue.py` reuse **renforcé**
  (Finding S2-D3 réalignement va exactement dans ce sens)
- ✅ Drop silencieux (log info, pas warn) **préservé**

Day 0 D4 décisions actées non rebattues (cf. nexus_grid_pivot.md
§Decisions actees) : ✅.

**Verdict S4 : clean.**

---

## 3. Findings consolidés

| ID | Scan | Type | Sévérité | Action |
|---|---|---|---|---|
| S2-D1 | S2 | Drift path daemon `crates/nexus-shell-daemon/src/api/quarantine.rs` | Non-bloquant | Réaligner plan §7.2 → coord-Python `api/quarantine.py` |
| S2-D2 | S2 | Drift path launcher `crates/nexus-launcher/src/cli.rs` | Non-bloquant | Réaligner plan §7.2 → coord-Python `cli/commands/quarantine.py` |
| S2-D3 | S2 | Plan §7.2 contredit pattern réutilisé `f238d31` | Non-bloquant | Réalignement S2-D1+S2-D2 restaure cohérence interne |
| S2-D4 | S2 | Design doc `S21_phase_D_quarantine_design.md` manquant | Non-bloquant procédural | Livrer dans commit chore préalable |

**0 finding bloquant** (S1, S3, S4 tous clean ; S2 = 4 findings
de drift naming / pré-requis doc, aucun ne touche Day 0 D4).

**Règle d'agrégation §6 G8** : `0 bloquant + ≥1 non-bloquant` →
**SCOPE-CUT-CONSISTENT**.

---

## 4. Action

### 4.1 Commit chore(planning) préalable

Ordre recommandé avant le commit feat Phase D :

1. **Update plan §7.1** : mention design doc livré au même commit.
2. **Update plan §7.2** : remplacer les 2 paths drift par les
   paths réalignés coord-Python, supprimer la mention `crates/
   nexus-launcher/src/cli.rs (modifié)` et `crates/nexus-shell-
   daemon/src/api/quarantine.rs (nouveau)`. Ajouter `packages/
   nexus-coordinator/src/nexus_coordinator/{quarantine_queue.py,
   api/quarantine.py, cli/commands/quarantine.py}` + tests
   `packages/nexus-coordinator/tests/test_quarantine_queue.py`
   et `packages/nexus-coordinator/tests/test_quarantine_api.py`
   et `packages/nexus-coordinator/tests/test_quarantine_cli.py`
   (split selon pattern S19 D existant).
3. **Update plan §7.3 tests** : ajuster (les tests Rust
   `quarantine_integration.rs` deviennent tests Python coord
   FastAPI TestClient).
4. **Livrer `.planning/research/S21_phase_D_quarantine_design.md`**
   couvrant les 5 sections réclamées.

Commit cible :

```
chore(planning): sprint21 §7 Phase D realignement coord-Python + design doc

Realignement Phase D au pattern S19 D f238d31 explicitement
reutilise (kickoff §D4 ligne 599-600). Plan original §7.2
ciblait crates/nexus-shell-daemon/src/api/quarantine.rs et
crates/nexus-launcher/src/cli.rs — chemins inexistants au tip
17035c3 (le daemon agrege les routes dans http.rs, le launcher
n'a pas de cli.rs agregateur). Le pattern coord-Python natif
(upload_queue.py + api/canary.py + cli/commands/invite.py) est
le pattern source que la phase D dit reutiliser.

Realignement :
- Module: packages/nexus-coordinator/src/nexus_coordinator/
  quarantine_queue.py (cf. upload_queue.py)
- API: packages/nexus-coordinator/src/nexus_coordinator/api/
  quarantine.py (cf. api/canary.py)
- CLI: packages/nexus-coordinator/src/nexus_coordinator/cli/
  commands/quarantine.py (cf. cli/commands/invite.py)

Day 0 D4 preserved integralement : schema 8-cols + TTL 15 min
auto-drop + CLI sbfb quarantine list|flush|drop + auth bearer
X-SBFB-Token + Host + Origin pattern S16.

Inclut : .planning/research/S21_phase_D_quarantine_design.md
(prerequisite explicite plan §7.1 + kickoff §D4 ligne 634-637).

G8 preflight : sprint21_phase_D_preflight.md verdict
SCOPE-CUT-CONSISTENT 4 findings non-bloquants documentes.

Working tree audit
- PHASE: aucun
- CRAFT: .planning/active/sprint21_plan.md (realignement §7.1+§7.2+§7.3)
         .planning/research/S21_phase_D_quarantine_design.md (prerequisite)
         .planning/active/sprint21_phase_D_preflight.md (G8 output)
- DEBT: aucun
- NOISE: aucun
```

### 4.2 Commit feat Phase D (post-chore)

Implémenter selon plan réaligné :

- `quarantine_queue.py` (`class QuarantineQueue` + `aiosqlite` WAL
  + tokio-style task TTL 15 min + `add()` + `list()` + `flush()`
  + `drop()` + `start()`/`stop()`)
- `api/quarantine.py` (FastAPI router `/quarantine/list`,
  `/quarantine/flush/{id}`, `/quarantine/drop/{id}` avec
  dependency injection auth pattern S16)
- `cli/commands/quarantine.py` (Typer commands `list`, `flush`,
  `drop` appelant `httpx` loopback)
- Wiring dans `coordinator.py` : `self.quarantine_queue =
  QuarantineQueue(...)` + `await self.quarantine_queue.start()`
  (pattern `upload_queue` lignes 294-307)
- Wiring dans `cli/main.py` : `app.add_typer(quarantine_cmds.app,
  name="quarantine")` (pattern `invite_cmds`)
- Config `class QuarantineQueue(BaseModel)` dans `config.py`
  (pattern `class UploadQueue(BaseModel)` ligne 108)

Tests (delta visé : ~+8 Python coord) :

1. `test_quarantine_queue.py::test_add_then_list_returns_entry`
2. `test_quarantine_queue.py::test_ttl_15min_auto_drop` (mock
   clock +900s)
3. `test_quarantine_queue.py::test_manual_flush_accept_sends_to_
   gossip`
4. `test_quarantine_queue.py::test_manual_drop_sets_status`
5. `test_quarantine_queue.py::test_cardinality_10k_entries_no_
   panic`
6. `test_quarantine_api.py::test_bearer_auth_required` (sans
   bearer → 401)
7. `test_quarantine_api.py::test_host_origin_check` (wrong
   origin → 403)
8. `test_quarantine_cli.py::test_list_json_schema` (CLI smoke)

Commit cible (inchangé) :

```
feat(sprint21): Phase D — quarantine queue SQLite WAL + manual flush CLI
```

### 4.3 Carry-over docs

Aucun carry-over S+1 nécessaire — réalignement absorbé inline
sans deferring.

---

## 5. Garde-fous §6 G8 vérifiés

- [x] **Evidence-based** : 4 findings sourcés sur git log + plan
      lignes précises + inventaire filesystem
- [x] **Day 0 respect** : D4 préservée intégralement, aucun item
      schema/TTL/CLI/auth touché
- [x] **Wire format** : `*_VERSION = 1` pre-launch policy intacte
- [x] **Test budget cap** : ~+8 tests Python coord = identique
      au plan original (juste relocaliser Rust → Python tests)
- [x] **Theme sprint** : quarantine queue = item D4 figée kickoff
      §1 « rate-limit + PII SDK + output filter + quarantine »
- [x] **Pas YAGNI** : ferme triangle DoS-defense (rate-limit +
      PoW + quarantine), pas de scaffolding spéculatif
- [x] **Retrospective trackée** : findings ajoutés
      `sprint21_audit_plan.md` (track Phase D realignement)
      à la rédaction Phase F wrap-up

**Tous garde-fous green** — verdict SCOPE-CUT-CONSISTENT
définitif.
