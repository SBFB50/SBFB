# Sprint 57 — Design Review Board (G1)

**Date** : 2026-05-09
**Reviewer** : agent Explore independant (session fraiche)
**Kickoff audite** : `.planning/active/sprint57_kickoff.md`

---

## Scoring

| Decision | Verdict | Finding |
|---|---|---|
| D1 — Apps dans examples/ HTML/JS pur | ✅ | Solid. Pattern examples/ valide (hello-world-app). sbfb-bridge.js existe dans web/public/. |
| D2 — windows-test audit cfg + doc CI | ⚠️ | GHA rust-ci.yml a deja un job windows-latest (ligne 120). Le kickoff mentionnait "ajouter un job" — formulation corrigee inline. Le scope est un audit des gaps cfg, pas de la creation d'infra CI. |
| D3 — E2E multi-noeuds 2 daemons localhost | ✅ | DaemonCluster + DaemonHandle prouves (S33). 4 tests existants dans multi_daemon.rs. Pattern spawn + health + shutdown solide. |
| D4 — Storage persistence SQLite M7 | ✅ | 6 migrations existantes (M1-M6). Slot M7 disponible. Schema (app_name, key) PK coherent. Pas de conflit avec M6 outbox (tables independantes). |

**Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ✅.**
**Rigor signal G4 satisfait (1 ⚠️ sur 4).**

---

## D2 ⚠️ Detail

Le kickoff prescrivait "Ajouter un job cargo nextest run Windows
a GHA si absent". Verification factuelle : `rust-ci.yml` ligne
120 inclut deja `windows-latest` dans la matrice test (ubuntu /
windows / macos). Le job existe depuis avant S57.

Le scope Phase A est un **audit des gaps cfg** (21 cfg(unix) /
12 cfg(windows) dans 11 fichiers) + **documentation** de la
strategie cross-platform dans PATTERNS.md §P46. Pas de creation
d'infra CI nouvelle.

**Decision** : adjust — formulation kickoff corrigee inline
pour supprimer la mention "ajouter un job" et clarifier que
Phase A verifie que le job existant couvre les tests gates.

---

## Verification codebase

- `web/public/sbfb-bridge.js` : confirme, 247 lignes, 9 methodes
- `examples/hello-world-app/` : confirme, precedent monorepo
- `crates/nexus-test-harness/src/lib.rs` : DaemonCluster::spawn(n)
  + DaemonHandle avec health_check + get_info + shutdown
- `crates/nexus-test-harness/tests/multi_daemon.rs` : 4 tests
  (boot, discovery, blob, task stub). Aucun test gossip.
- `.github/workflows/rust-ci.yml` : job test matrix ubuntu /
  windows-latest / macos-14 avec nextest + doctests
- `crates/nexus-coordinator-rs/src/db.rs` : 6 migrations (M1-M6)
- `crates/nexus-shell-daemon/src/storage_api.rs` : 172 LOC,
  HashMap in-memory, 2 tests
