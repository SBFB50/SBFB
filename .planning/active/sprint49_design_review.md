# Sprint 49 — Design Review Board (G1)

**Reviewer** : agent Explore independant (pas la session code).
**Documents evalues** : sprint49_kickoff.md §D1..D4.
**Code consulte** : runtime.rs (boot sequence), cli.rs (clap),
validator_loop.rs (interface), dispatcher.rs (interface),
docs.rs (iroh-docs wrapper), S49_coordinator_rust_migration.md.

---

## Scoring

| Decision | Verdict | Raison |
|---|---|---|
| D1 | ✅ | Single-project + eager doc creation alignes avec l'architecture daemon. Risk R1 mitigue par tests integration. Pas de conflit Day 0. Scope 1 phase. |
| D2 | ⚠️ | Concurrent doc writes (dispatch + HTTP) pas explicitement serialises. R2 assume "no contention" mais daemon async — checkpoint Phase A requis. |
| D3 | ⚠️ | DB access offline vs online pour subcommands one-shot non resolu au gel — risque redesign Phase B. |
| D4 | ✅ | Scope cut S50 correct. Core path non affecte. Governance Python-dependent acceptable. |

**Rigor** : 2/4 ✅, 2 ⚠️, 0 ❌.

---

## Details par decision

### D1 ✅ — Project doc lifecycle

Le daemon boot deja le node iroh (runtime.rs:256-272). Ajouter
create/reopen doc est ~50 LOC via DocsClient existant dans
nexus-core-rs/docs.rs (create_doc, open_doc, author_default).
Le pattern Arc-shared DaemonHttpState est eprouve (coordinator_db,
canary_registry, etc.). Pas de blind spot significatif.

### D2 ⚠️ — Dispatch pipeline

Le dispatch loop spawned dans start() et les endpoints HTTP
`/api/v1/tasks` peuvent potentiellement ecrire des TaskEntry dans
le doc de facon concurrente. Le daemon est async (tokio) — la
serialisation n'est pas garantie par le modele sequentiel du
coordinator Python. Le reviewer recommande de clarifier le pattern
d'ecriture (MPSC queue → dispatch loop drain → doc write
sequentiel).

**Ack planner** : le dispatch loop est le seul ecrivain dans le
doc. Les HTTP endpoints soumettent via channel tokio. Pattern MPSC
identique a result_event_tx.

### D3 ⚠️ — CLI subcommands

R3 note que les subcommands init/invite necesitent un DB handle.
Deux modes possibles : offline (open DB sans daemon) ou online
(HTTP client). Le choix n'est pas explicite dans D3, creant un
risque de redesign Phase B.

**Ack planner** : mode offline retenu pour subcommands one-shot.
Direct CoordinatorDb::open(). Pattern identique a nexus-worker.

### D4 ✅ — app-gov defer S50

Scope cut documente, core path non affecte. Risque residuel :
governance flow reste Python-dependent pendant S49 — acceptable
pre-v1.0.
