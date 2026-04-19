# Sprint 21 Phase D — Quarantine queue : design doc

**Date** : 2026-04-19 (pre-Phase D, session de design pure ;
livre dans le commit `chore(planning)` de realignement coord-Python).
**Auteur** : agent design pre-Phase D (kickoff §D4 + plan §7
realignes ; ce design materialise les requirements §7.1).
**Tip master ref** : `17035c3` (post-S21 Phase C +
chore docs fairness).
**Statut** : design accepte, l'implementation Phase D suit ce
document. Toute deviation Phase D vs ce design = doit etre
justifiee par G8 preflight (pas un drift silencieux).
**G8 preflight ref** : `.planning/active/sprint21_phase_D_
preflight.md` verdict SCOPE-CUT-CONSISTENT 4 findings non-
bloquants.

---

## 1. Probleme adresse

### 1.1 Threat model — DoS gossip + audit trail post-mortem

`docs/security/HARDENING_ROADMAP.md §3` ligne C-DosFlood S21
(impact 4 / likelihood 4 / detect 3 / risque 5.3) decrit le
scenario : un peer malicieux ou compromis emet un volume de
messages gossip (task submit, ProjectAnnouncement, claim, etc.)
qui depasse la capacite de traitement raisonnable du coordinator
local. Sans defense en profondeur, le coord est sature CPU/IO et
les flux legitimes degradent.

Triangle defense-in-depth (vision Sprint 21 closing) :

1. **Rate-limit** sliding-window multi-tier (Phase A `63afe4e`,
   `governor 0.10.2` GCRA worker-engine gate R1, key triplet
   `(consumer, worker, model)`) : reject **avant l'ingest**
   coord/worker des messages au-dela du quota.
2. **PoW gossip subscribe** (S20 Phase C `16b94ba`,
   `pow_policy_loader.rs` hot-reload, wire au runtime
   `spawn_gossip_subscribe_task`) : require Hashcash proof
   sur chaque message gossip avant que le subscriber le passe
   au runtime — **anti-spam structurel** au niveau wire.
3. **Quarantine hold + manual flush** (Phase D, ce document) :
   pour les messages qui ont *passe* PoW + rate-limit mais
   declenchent un **soft signal d'alerte** (rate-strikes
   accumules, pow_status='valid' but suspicious volume from
   single sender, etc.), hold-en-quarantine 15 min et permettre
   l'operateur de **manually flush** (accept dans gossip)
   ou **drop** (definitivement reject) via CLI.

Phase D ferme le triangle : rate-limit reject les flux abusifs,
PoW reject les flux unsigned, quarantine **observe et permet
l'operateur de juger** les cas borderline avec un audit trail
persistent.

### 1.2 Pourquoi pas in-memory ou gossip-wide

Kickoff §D4 lignes 602-614 a deja rejete :

- **In-memory `BTreeMap` TTL** : volatil au crash daemon, perte
  des messages quarantine = perte d'audit trail qui defait le
  but meme.
- **Gossip-wide diffusion** (quarantine propage entre peers) :
  amplification DoS vector + complexite consensus + pas
  d'equivalent libp2p/Tor/Filecoin avec **manual flush**.
- **Redis externe** : ajoute infra-creep hors-scope pre-launch.
- **Admin UI custom** : sans precedent SBFB (CLI = canon
  cf. `sbfb canary ack` S20 Phase E).

SQLite WAL local + CLI = pattern coherent S19 D `f238d31`
`upload_queue.py` deja battle-tested.

---

## 2. Schema SQLite

### 2.1 Table `quarantine_messages`

```sql
CREATE TABLE IF NOT EXISTS quarantine_messages (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    topic                 TEXT NOT NULL,
    sender_pubkey         BLOB NOT NULL,         -- Ed25519 32 bytes
    payload_bytes         BLOB NOT NULL,
    received_at_epoch_s   INTEGER NOT NULL,      -- secondes UNIX wall-clock
    rate_strikes          INTEGER NOT NULL,      -- compteur rate-limit S21 Phase A
    pow_status            TEXT NOT NULL,         -- 'valid' | 'missing' | 'invalid'
    flush_status          TEXT NOT NULL DEFAULT 'pending'
                                                 -- 'pending' | 'flushed' | 'dropped'
);

CREATE INDEX IF NOT EXISTS idx_quarantine_received
  ON quarantine_messages(received_at_epoch_s);
CREATE INDEX IF NOT EXISTS idx_quarantine_sender
  ON quarantine_messages(sender_pubkey);
```

Pragmas runtime (cohrents `upload_queue.py` ligne 145-146) :

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

`WAL` = durable across crashes + concurrent reads pendant ecriture.
`synchronous = NORMAL` = compromise debit/durabilite acceptable
pour une queue auditable (les pertes au moment exact d'un crash
sont tolerables — la quarantine est un buffer temporaire 15 min,
pas un ledger crypto).

### 2.2 Path

`<project_dir>/quarantine.sqlite` (project-scoped, coherent avec
`upload_queue.sqlite` ligne 295 `coordinator.py`). Pas de
`~/.sbfb/quarantine.db` global — chaque coord par projet a sa
propre quarantine, evite la collision entre projets.

Le path est resolved par le `Coordinator` au demarrage et passe
au `QuarantineQueue` constructor (pattern `upload_queue_db =
self.project_dir / "upload_queue.sqlite"`).

### 2.3 Evolution schema

Pre-launch protocol policy CLAUDE.md §Pre-launch protocol policy
ligne 1 : pas de wire format VERSION applicable ici (SQLite
local). Si le schema evolue, on edit la table directement et
suprimme/regenere la DB locale (acceptable car TTL 15 min = aucun
historique long-life persisté).

Post-v1.0 release : si on garde la quarantine apres go-live, on
ajoute une migration runner (pattern S9 `migration_runner.py`
deja en place coord-side). Hors-scope Phase D.

---

## 3. TTL clock semantics

### 3.1 Source-of-truth temporelle

`received_at_epoch_s = int(time.time())` au moment `add()`. Wall-
clock secondes UNIX (pas `monotonic_ns`) parce que :

- Survies aux crashes daemon (monotonic reset au boot)
- Lecture humaine simple via `datetime.fromtimestamp(received_at_
  epoch_s)` dans CLI `quarantine list`
- Indexation `idx_quarantine_received` simple sur INTEGER
- Difference 1-2 secondes vs monotonic non-pertinente (TTL = 900
  secondes, granularite seconde suffit)

### 3.2 NTP sync : pas de dependance

Le TTL est **relatif** (15 min apres `received_at`) donc une
derive NTP < 60 secondes est imperceptible. La quarantine
**n'est pas** un timestamp authoritative cross-network — c'est
local au coord. Si le coord a une horloge bizarre, le TTL est
juste decale d'autant pour ce coord. Pas d'impact securite.

### 3.3 Sweep loop

Tokio-style asyncio task qui tourne toutes les 30 secondes :

```python
async def _ttl_sweep_loop(self) -> None:
    while not self._stopping.is_set():
        try:
            await asyncio.wait_for(self._stopping.wait(), timeout=30.0)
        except asyncio.TimeoutError:
            pass
        await self._auto_drop_expired()

async def _auto_drop_expired(self) -> None:
    cutoff = int(time.time()) - self.ttl_seconds  # ttl_seconds = 900
    async with self._lock:
        async with aiosqlite.connect(self.db_path) as db:
            cursor = await db.execute(
                "DELETE FROM quarantine_messages "
                "WHERE received_at_epoch_s < ? AND flush_status = 'pending'",
                (cutoff,),
            )
            deleted = cursor.rowcount or 0
            await db.commit()
    if deleted > 0:
        _log.info("quarantine TTL sweep dropped expired entries", count=deleted)
```

**Decision design** : seules les entries `flush_status='pending'`
sont auto-dropped au TTL. Les entries deja `flushed` ou `dropped`
sont **conservees plus longtemps** comme audit trail (cleanup
manuel plus tard ou via retention policy operateur, hors-scope
Phase D).

Drop silencieux = log info-level (pas warn, kickoff §D4 ligne
591-592 explicite).

---

## 4. Security manual flush

### 4.1 Auth pattern S16

Reuse 100% du pattern `crates/nexus-shell-daemon/src/http.rs`
`/panic/wipe` (S20 Phase B `c32ecb3`) :

- **Bearer token** : header `X-SBFB-Token` matche le token genere
  par le launcher (perm 0600 `~/.sbfb/loopback_token`). Verifie
  via FastAPI `Depends()` injection.
- **Host check** : header `Host` doit etre dans l'allowlist
  `{localhost, 127.0.0.1, [::1]}` (mitigation CVE-2025-49596 DNS
  rebinding).
- **Origin check** : header `Origin` (si present) doit matcher
  loopback. Reject 403 sinon.

Implementation : reuse `nexus_coordinator.api.auth.bearer_token_
required` dependency (deja en place coord-side, pattern API
canary). Ajouter Host/Origin check via une dependency
`loopback_origin_required` si pas deja agglutinee. Si oui, juste
plug.

### 4.2 Endpoints REST

```
GET  /quarantine/list                  → liste paginated
POST /quarantine/flush/{id}            → flush_status='flushed'
POST /quarantine/drop/{id}             → flush_status='dropped'
```

`flush_status='flushed'` action semantics : **marquer accept
pour audit**. La re-injection dans le gossip layer est laissee
hors-scope Phase D (depend du Sybil/kudos work S22+) ; pour
Phase D le `flush` est un statement operateur "j'ai juge ce
message legitime", l'effet est une trace audit.

`flush_status='dropped'` action semantics : **marquer reject
final**. Aucune re-injection nulle part, juste persiste comme
audit du jugement operateur.

Cette separation `flush_status='pending|flushed|dropped'` permet
au CLI `quarantine list` de filtrer par statut.

### 4.3 CLI Typer

```
nexus-coordinator quarantine list [--status <pending|flushed|dropped|all>] [--json]
nexus-coordinator quarantine flush <id>
nexus-coordinator quarantine drop <id>
```

Implementation : Typer commands dans `cli/commands/quarantine.py`
appelant `httpx>=0.27` loopback (pattern `cli/commands/invite.py`
qui appelle deja les endpoints invites coord-side). Le bearer
token est lu depuis `~/.sbfb/loopback_token` (pattern partage
launcher/coord).

---

## 5. Interaction gossip layer

### 5.1 Ingest path : qui appelle `quarantine_queue.add()` ?

Le subscriber gossip (cote coord ou via daemon proxy) verifie deja
en amont :

1. **PoW gate** (S20 Phase C wire `spawn_gossip_subscribe_task`) :
   reject brutal des messages sans PoW valide.
2. **Rate-limit** (S21 Phase A `governor 0.10.2`) : reject brutal
   au-dela du quota `(consumer, worker, model)`.

Quand un message **passe** ces deux gates mais matche un
heuristique soft (cumul rate-strikes recent depasse threshold,
pattern suspicious detection module — definition de
"borderline" reste a l'operateur via config policy hot-reload
Phase F+ ou S22), le subscriber appelle `quarantine_queue.add(
topic, sender_pubkey, payload_bytes, rate_strikes, pow_status)`
au lieu de passer le message au handler runtime.

**Pour Phase D scope minimal** : on livre la primitive
`QuarantineQueue` + REST + CLI **sans** wire-up automatique
depuis le subscriber gossip (c'est le `coordinator.py` test
harness qui exerce add/list/flush/drop directement). Le wire-up
arrive S22+ avec les soft heuristics (qui requiert Sybil/kudos
context). Cf. carry-over plan F.

### 5.2 PoW re-verify ? Non.

Le `pow_status` est persisted dans la table comme **audit
metadata uniquement**. La quarantine ne re-verifie pas PoW au
flush — le subscriber a deja fait le travail au moment de
l'ingest. Re-verifier serait :

- Couteux (Hashcash compute non-trivial)
- Inutile (le hash est deja cryptographique)
- Source de bug (re-verify avec policy differente = false
  negative)

Si l'operateur veut re-verifier manuellement, il peut le faire
hors-CLI via le `payload_bytes` retrieve (qui contient le
proof-of-work blob).

---

## 6. Cardinality + benchmarks

### 6.1 Steady-state estimate

Assomption operationnelle :

- **Volume gossip moyen** : 1000 msg/min/coord (tres pessimiste,
  coord SBFB beta = ~10 msg/min realiste)
- **Quarantine rate** : ~1% des messages soft-flagged = 10
  msg/min
- **TTL** : 900 secondes (15 min) = entries vivantes ~150
- **Steady-state cardinality** : ~150 entries pending +
  cumul flushed/dropped recents

Bench cible (validation Phase F si besoin) :

- `add()` p99 < 5 ms (SQLite WAL commit one-row trivially)
- `list(status=pending)` p99 < 50 ms pour 10k entries (index
  `idx_quarantine_received`)
- `_auto_drop_expired()` sweep 30s sans contention (DELETE batch
  with LIMIT 1000 + loop si > 1000, evite long lock)
- Total disk usage : < 10 MB pour 10k entries (payload moyen ~1
  KB)

### 6.2 Bench harness Phase D test 5

`test_cardinality_10k_entries_no_panic` :

```python
async def test_cardinality_10k_entries_no_panic(tmp_path):
    queue = QuarantineQueue(db_path=tmp_path / "q.sqlite", ttl_seconds=900)
    await queue.start()
    try:
        for i in range(10_000):
            await queue.add(
                topic=f"topic-{i % 10}",
                sender_pubkey=b"\x01" * 32,
                payload_bytes=b"x" * 256,
                rate_strikes=0,
                pow_status="valid",
            )
        rows = await queue.list(status="pending")
        assert len(rows) == 10_000
        # Force TTL sweep with mocked clock at +901s
        with mock.patch.object(queue, "_now", return_value=int(time.time()) + 901):
            await queue._auto_drop_expired()
        rows = await queue.list(status="pending")
        assert rows == []
    finally:
        await queue.stop()
```

---

## 7. Files livres Phase D

### 7.1 Code (coord-Python)

- `packages/nexus-coordinator/src/nexus_coordinator/quarantine_
  queue.py` — `class QuarantineQueue` core
- `packages/nexus-coordinator/src/nexus_coordinator/api/
  quarantine.py` — FastAPI router
- `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/
  quarantine.py` — Typer commands
- `packages/nexus-coordinator/src/nexus_coordinator/coordinator.
  py` — wiring (instantiation + start/stop)
- `packages/nexus-coordinator/src/nexus_coordinator/cli/main.
  py` — register `quarantine` Typer subapp
- `packages/nexus-coordinator/src/nexus_coordinator/config.py` —
  `class QuarantineQueue(BaseModel)` config + integrate dans
  `class CoordinatorConfig`
- `packages/nexus-coordinator/src/nexus_coordinator/api/__init__.
  py` — register router

### 7.2 Tests (Python coord, +8)

`packages/nexus-coordinator/tests/test_quarantine_queue.py` (5) :

1. `test_add_then_list_returns_entry`
2. `test_ttl_15min_auto_drop` (mock clock +900s)
3. `test_manual_flush_marks_status`
4. `test_manual_drop_sets_status`
5. `test_cardinality_10k_entries_no_panic`

`packages/nexus-coordinator/tests/test_quarantine_api.py` (2) :

6. `test_bearer_auth_required` (sans bearer → 401)
7. `test_host_origin_check` (wrong origin → 403)

`packages/nexus-coordinator/tests/test_quarantine_cli.py` (1) :

8. `test_list_json_outputs_schema` (Typer CliRunner smoke)

### 7.3 Hors-scope Phase D (carry S22+)

- Wire-up automatique subscriber gossip → `quarantine_queue.
  add()` (depend Sybil/kudos heuristics)
- Re-injection automatique sur `flush` vers gossip (depend
  decision Sybil-resistance)
- Bench harness production-grade (Phase F si besoin)
- Migration runner schema evolution post-v1.0

---

## 8. References

- `f238d31` (S19 Phase D) — `upload_queue.py` pattern source
- `c32ecb3` (S20 Phase B) — `/panic/wipe` auth pattern S16
- `16b94ba` (S20 Phase C) — PoW wire `spawn_gossip_subscribe_
  task`
- `63afe4e` (S21 Phase A) — `governor 0.10.2` rate-limit
- `.planning/active/sprint21_phase_D_preflight.md` — G8 verdict
  SCOPE-CUT-CONSISTENT
- `.planning/active/sprint21_kickoff.md §D4` lignes 571-637 —
  Day 0 D4 figee
- `.planning/active/sprint21_plan.md §7` (post-realignement) —
  fichiers cibles + tests + commit cible
- `docs/security/HARDENING_ROADMAP.md §3` ligne C-DosFlood S21
