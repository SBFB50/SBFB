# Sprint 57 — Plan d'execution

**Ecrit** : 2026-05-09
**Tip d'entree** : `e7a9b93`
**Decisions Day 0** : D1..D4 gelees (cf. kickoff §4)

---

## §1 Etat verifie a l'entree

| Suite | Count |
|---|---|
| Rust nextest | 1227 |
| Rust doctests | 6p 1i |
| Vitest | 256 |
| Playwright | 42 + 2 fail env |
| size-limit | 6/6 |
| clippy | 0 warnings |
| cargo fmt | 0 diff |
| npm lint | 0 error (5 warnings pre-existants) |
| tsc | 0 error |

---

## §2 Decisions Day 0 (gelees, rappel)

- **D1** : Apps dans `examples/` monorepo, HTML/CSS/JS pur
- **D2** : windows-test = audit cfg + doc CI cross-platform
  (GHA a deja un job Windows dans rust-ci.yml)
- **D3** : E2E multi-noeuds = gossip test via DaemonCluster
  existant, gate SBFB_INTEGRATION=1
- **D4** : Storage persistence = SQLite M7 dans coordinator.db

---

## §3 Research consulte

- `crates/nexus-test-harness/src/lib.rs` : DaemonHandle + DaemonCluster
  existants (S33 Phase C). spawn() avec TempDir isolees + port 0
  allocation OS + health check polling + shutdown graceful.
- `crates/nexus-test-harness/tests/multi_daemon.rs` : 4 tests
  existants (boot, discovery, blob transfer, task stub). **Aucun
  test gossip** — c'est le gap E2E a combler.
- `.github/workflows/rust-ci.yml` : job test matrix ubuntu/windows/macos
  avec nextest + doctests. **Windows CI deja operationnel**.
- `crates/nexus-coordinator-rs/src/db.rs` : 6 migrations (M1-M6).
  M7 sera la prochaine.
- `crates/nexus-shell-daemon/src/storage_api.rs` : 172 LOC,
  HashMap<String, HashMap<String, Value>> in-memory. 2 tests.
- `web/public/sbfb-bridge.js` : SDK bridge 9 methodes. A copier
  dans les apps.
- `examples/hello-world-app/` : precedent monorepo pour app
  exemple (Python, non pertinent mais valide le pattern directory).

---

## §4 Phases

### Phase A — MANDATORY carries (windows-test + E2E multi-noeuds)

**Dependencies** : aucune (phase autonome).

#### §A.1 Scope

**windows-test-cfg-unix** : auditer les 21 `cfg(unix)` et 12
`cfg(windows)` dans 11 fichiers Rust. Verifier que chaque test
utilisant du code platform-specific est correctement gate avec
`#[cfg(target_family = "...")]`. Documenter la strategie
cross-platform dans PATTERNS.md §P46 (quelles features sont
unix-only, pourquoi, et comment le CI les couvre). Le GHA
rust-ci.yml a deja un job `windows-latest` dans la matrice test
(ubuntu/windows/macos) — verifier que les tests gates passent
correctement sur Windows CI (pas de nouveau workflow a creer,
le job existe deja).

**E2E multi-noeuds** : ajouter a `multi_daemon.rs` un test
qui verifie la communication gossip entre 2 daemons :
1. Spawn 2 daemons via DaemonCluster
2. Daemon B souscrit a Daemon A comme curator (POST /curators/subscribe)
3. Daemon A publie une annonce projet (POST /api/daemon/publish)
4. Daemon B recoit l'annonce via browse (GET /api/daemon/browse)

Le test est gate par `SBFB_INTEGRATION=1` (consistent avec
les tests existants). Timeout 30s pour le gossip relay.

DaemonHandle enrichi : methode `subscribe_curator(node_id)` et
`publish_project(name)` et `browse_projects()` pour encapsuler
les appels HTTP.

#### §A.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-test-harness/src/lib.rs` | +3 methodes DaemonHandle (subscribe_curator, publish_project, browse_projects) |
| `crates/nexus-test-harness/tests/multi_daemon.rs` | +1 test gossip E2E (test_cross_daemon_gossip_exchange) |
| `docs/rust/PATTERNS.md` | +§P46 cross-platform cfg strategy |
| 0-5 fichiers crates/*.rs | cfg gate corrections si manquantes |

#### §A.3 Tests plan

1. `test_cross_daemon_gossip_exchange` : 2 daemons, subscribe +
   publish → browse receives. Gate SBFB_INTEGRATION=1.
2. Tests cfg existants : verifier qu'ils passent sur Windows
   (couvert par GHA rust-ci.yml matrix).

#### §A.4 Critere d'acceptation

```bash
cargo nextest run --workspace --locked
# Windows : GHA rust-ci.yml passe (verifie manuellement ou via
# dernier run GHA)
# E2E : SBFB_INTEGRATION=1 cargo nextest run -p nexus-test-harness
#   (sur machine dev avec connectivity iroh relay)
grep "§P46" docs/rust/PATTERNS.md
```

#### §A.5 Commit cible

```
feat(sprint57): Sprint 57 Phase A — MANDATORY carries windows-test + E2E multi-noeuds

§P46 cross-platform cfg strategy documented in PATTERNS.md.
21 cfg(unix) / 12 cfg(windows) audited across 11 files.
GHA rust-ci.yml already runs ubuntu/windows/macos matrix.

test_cross_daemon_gossip_exchange added to multi_daemon.rs:
2 daemons, curator subscribe + project publish → browse
receives. Gated SBFB_INTEGRATION=1 (requires iroh relay).

DaemonHandle enriched: subscribe_curator(), publish_project(),
browse_projects() helpers for integration tests.

CLOSE P2-S54-windows-test-cfg-unix (3/3 MANDATORY).
CLOSE P2-S54-test-E2E-multi-noeuds (3/3 MANDATORY).

Delta tests: +1 Rust (1227→1228, gated SBFB_INTEGRATION).
Scope cuts: verified deploy E2E S58, full gossip validation S58.
```

---

### Phase B — Protocol Explorer MVP

**Dependencies** : Phase A (pas fonctionnelle, mais sequencage
commit — Phase B peut etre codee en parallele).

#### §B.1 Scope

Creer `examples/sbfb-explorer/` avec :
- `index.html` : page principale, navigation par ancres (#)
- `style.css` : dark theme minimaliste, responsive, CSS Grid
- `app.js` : bridge integration F3 (status live)
- `sbfb-bridge.js` : copie depuis `web/public/`
- 5 sections de contenu :
  1. Architecture du reseau (noeud → daemon → coordinator → workers)
  2. Cycle de vie d'une app (repo Git → verified deploy → zip → iframe)
  3. Cycle de vie d'une tache (submit → dispatch → validation → kudos)
  4. Modele de securite (loopback, sandbox, CSP, curators, Sybil)
  5. Philosophie (zero admin, open source par construction)
- F2 : liens directs vers les fichiers source du repo
- F3 : panneau live status (node_id, peers, version, uptime,
  apps disponibles) via bridge `node_status` + `browse_list` +
  `identity_pubkey`. Degrade gracieusement si daemon non connecte.

Cible : < 500KB zip, 0 dependance externe, fonctionne offline
(F1+F2 statiques).

#### §B.2 Fichiers touches

| Fichier | Role |
|---|---|
| `examples/sbfb-explorer/index.html` | Structure + contenu 5 sections + F3 live status panel |
| `examples/sbfb-explorer/style.css` | Dark theme CSS Grid responsive |
| `examples/sbfb-explorer/app.js` | Bridge F3 : node_status + browse_list + identity_pubkey |
| `examples/sbfb-explorer/sbfb-bridge.js` | Copie SDK bridge |

#### §B.3 Tests plan

1. Test Vitest : `useBridge` dispatch correctement les methodes
   existantes (couverture pre-existante S56).
2. Test manuel : zip examples/sbfb-explorer/ → blob-serve →
   iframe → F3 live status affiche.
3. Validation taille : `du -sh examples/sbfb-explorer/` < 500KB.

#### §B.4 Critere d'acceptation

```bash
# App statique, pas de build step
ls examples/sbfb-explorer/index.html
# Validation taille
du -sh examples/sbfb-explorer/
# Validation HTML (pas d'erreurs de syntaxe critiques)
grep -c "</html>" examples/sbfb-explorer/index.html
```

#### §B.5 Commit cible

```
feat(sprint57): Sprint 57 Phase B — Protocol Explorer MVP (sbfb-explorer)

First SBFB app: interactive protocol documentation deployed
as a static HTML/CSS/JS archive in the iframe sandbox.

5 sections: architecture, app lifecycle, task lifecycle,
security model, philosophy. Links to source code (F2).
Live node status panel via bridge postMessage: node_status +
browse_list + identity_pubkey (F3). Graceful degradation
when daemon is offline.

Pure HTML/CSS/JS, zero dependencies, < 500KB.
SDK sbfb-bridge.js copied from web/public/.

Delta tests: +0 (static HTML app, no Rust/Vitest changes).
Scope cuts: F4 tutorial interactif S58, gossip stats S58.
```

---

### Phase C — Ideas Hub MVP + storage persistence SQLite

**Dependencies** : Phase A (pas fonctionnelle). Phase B
(sequencage). D4 storage persistence requis pour Ideas Hub.

#### §C.1 Scope

**Storage persistence** : migration M7 dans coordinator.db.
Table `app_storage(app_name TEXT NOT NULL, key TEXT NOT NULL,
value TEXT NOT NULL, PRIMARY KEY(app_name, key))`. Le champ
`value` stocke du JSON serialise (TEXT, pas BLOB, pour debug
SQLite). Helpers DB : `load_all_storage()` retourne
`HashMap<String, HashMap<String, Value>>`, `upsert_storage(app, key, value)`,
`delete_storage(app, key)`. Le storage_api.rs charge le HashMap
au boot depuis la DB et ecrit-through sur chaque mutation
(set → upsert, delete → delete). Le HashMap in-memory reste
le cache de lecture (zero changement pour les readers).

**Ideas Hub** : creer `examples/sbfb-ideas/` avec :
- `index.html` : formulaire proposition + liste des idees + vote
- `style.css` : dark theme minimaliste
- `app.js` : bridge CRUD
- `sbfb-bridge.js` : copie SDK
- Schema donnees :
  - `ideas/{uuid}` → `{ title, description, author, created_at, votes_count }`
  - `votes/{idea_id}/{voter}` → `{ timestamp }`
- Fonctionnalites :
  - F1 : proposer une idee (titre + description, auteur = identity_pubkey)
  - F2 : voter (1 vote par identite par idee, toggle upvote)
  - Liste triee par votes_count, plus recents en haut a egalite
  - Identite : `identity_pubkey` via bridge (pas de login)
- Vote non-pondere Kudos (MVP). Ponderation = S58.

Cible : < 300KB zip, 0 dependance externe.

#### §C.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` | Migration M7 + load_all_storage() + upsert_storage() + delete_storage() |
| `crates/nexus-shell-daemon/src/storage_api.rs` | Boot load from DB + write-through on set/delete |
| `examples/sbfb-ideas/index.html` | Formulaire + liste + vote UI |
| `examples/sbfb-ideas/style.css` | Dark theme CSS |
| `examples/sbfb-ideas/app.js` | Bridge CRUD : storage_set + storage_list + storage_delete + identity_pubkey |
| `examples/sbfb-ideas/sbfb-bridge.js` | Copie SDK bridge |

#### §C.3 Tests plan

1. `test_storage_persistence_survives_reopen` : insert → close DB →
   reopen → load → verify present. (Rust, dans db.rs)
2. `test_upsert_storage_overwrite` : double upsert meme cle →
   derniere valeur gagne. (Rust, dans db.rs)
3. `test_delete_storage_nonexistent` : delete cle absente → pas
   d'erreur. (Rust, dans db.rs)
4. `test_storage_api_boot_loads_persisted` : integration storage_api
   charge depuis DB au boot. (Rust, dans storage_api.rs)
5. Test manuel : Ideas Hub dans iframe → creer idee → restart
   daemon → idee survit.
6. Validation taille : `du -sh examples/sbfb-ideas/` < 300KB.

#### §C.4 Critere d'acceptation

```bash
cargo nextest run --workspace --locked
# >= 1231 (1227 + 4 storage tests)
ls examples/sbfb-ideas/index.html
du -sh examples/sbfb-ideas/
```

#### §C.5 Commit cible

```
feat(sprint57): Sprint 57 Phase C — Ideas Hub MVP + storage persistence SQLite

Storage persistence: migration M7 app_storage table in
coordinator.db. load_all_storage() at boot, write-through
upsert/delete on mutation. In-memory HashMap remains read
cache. Data survives daemon restart.

Ideas Hub MVP: propose ideas (title + description) and vote
(1 upvote per identity per idea). Identity via bridge
identity_pubkey. Data stored via bridge storage_set/list/delete.
Pure HTML/CSS/JS, < 300KB.

CLOSE P2-STORAGE-SQLITE (1/3).

Delta tests: +4 Rust (1228→1232, storage persistence tests).
Scope cuts: F3 repo links S58, F4 groups post-v1.0,
Kudos-weighted voting S58.
```

---

### Phase D — Wrap-up + verification + audit plan S58

**Dependencies** : Phase A + B + C toutes livrees.

#### §D.1 Scope

- CLAUDE.md : update S57 CLOSED, carries S58, compteurs
- HARDENING_ROADMAP.md : update last_validated S57
- docs/claude/SPRINT_LOG.md : ajouter row S57
- verification.md : 24+ fail-fast rows
- sprint58_audit_plan.md : 7+ tracks
- memory : update nexus_grid_pivot.md

#### §D.2 Fichiers touches

| Fichier | Role |
|---|---|
| `CLAUDE.md` | S57 CLOSED, carries S58 |
| `docs/security/HARDENING_ROADMAP.md` | last_validated S57 |
| `docs/claude/SPRINT_LOG.md` | +1 row S57 |
| `.planning/active/sprint57_verification.md` | NEW 24+ rows |
| `.planning/active/sprint58_audit_plan.md` | NEW 7+ tracks |

#### §D.3 Commit cible

```
chore(sprint57): Phase D — wrap-up + verification + audit plan S58
```

---

## §5 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1232, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 256 | |
| 9 | npm build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 12 | Phase A preflight G8 | verdict | EXECUTE | |
| 13 | Phase A review | verdict | PASS | |
| 14 | Phase B preflight G8 | verdict | EXECUTE | |
| 15 | Phase B review | verdict | PASS | |
| 16 | Phase C preflight G8 | verdict | EXECUTE | |
| 17 | Phase C review | verdict | PASS | |
| 18 | §P46 cross-platform doc | `grep "§P46" docs/rust/PATTERNS.md` | present | |
| 19 | E2E gossip test exists | `grep "gossip_exchange" crates/nexus-test-harness/tests/multi_daemon.rs` | present | |
| 20 | Protocol Explorer exists | `ls examples/sbfb-explorer/index.html` | present | |
| 21 | Ideas Hub exists | `ls examples/sbfb-ideas/index.html` | present | |
| 22 | Storage persistence | `grep "app_storage" crates/nexus-coordinator-rs/src/db.rs` | present | |
| 23 | Scope cuts | 13/13 respectes | all checked | |
| 24 | Delta tests | cumule documente | documented | |

---

## §6 Git plan

| # | Scope | Commit title |
|---|---|---|
| 1 | chore | `chore(planning): Sprint 57 kickoff + plan + design review + migration S56→archive` |
| 2 | Phase A | `feat(sprint57): Sprint 57 Phase A — MANDATORY carries windows-test + E2E multi-noeuds` |
| 3 | Phase B | `feat(sprint57): Sprint 57 Phase B — Protocol Explorer MVP (sbfb-explorer)` |
| 4 | Phase C | `feat(sprint57): Sprint 57 Phase C — Ideas Hub MVP + storage persistence SQLite` |
| 5 | Phase D | `chore(sprint57): Phase D — wrap-up + verification + audit plan S58` |

---

## §7 Scope cuts

(Copie kickoff §7)

1. LT-7 Tier 3 (N builders, auto-deploy) — S58+
2. Verified deploy E2E from repos Git separes — S58
3. Protocol Explorer F3 avance (gossip stats, latence) — S58
4. Protocol Explorer F4 (tutoriel interactif) — S58
5. Ideas Hub F3 (lier repos Git) — S58
6. Ideas Hub F4 (groupes de travail) — post-v1.0
7. Ideas Hub F5 (integration reseau, gossip notifications) — post-v1.0
8. Kudos-weighted voting — S58
9. AppStorage replication P2P (iroh-docs sync) — post-v1.0
10. Rate-limit retain_recent housekeeping — S58
11. P2-JITTER-SCOPE test integration — S58 (3/3 MANDATORY)
12. P2-INVITE-U16-WIRE doc post-v1.0 — S58 (3/3 MANDATORY)
13. LT-1 Kudos-v2 fairness reform — S58+

---

## §8 Risks (R1..R5)

(Copie kickoff §9)

| # | Risque | Mitigation |
|---|---|---|
| R1 | E2E gossip flaky | Timeout 30s + SBFB_INTEGRATION gate + #[ignore] fallback |
| R2 | Protocol Explorer trop ambitieux | MVP = 5 pages essentielles |
| R3 | Ideas Hub schema evolue | Pre-launch policy, pas de compat |
| R4 | Migration M7 conflit M6 | Migrations additives independantes |
| R5 | Windows CI minutes Actions | Job Windows nextest uniquement |

---

## §9 Checkpoint de cloture

1. [ ] 24/24 fail-fast rows verts
2. [ ] 5 commits (1 chore planning + 3 feat phases + 1 chore wrap-up)
3. [ ] 2 MANDATORY FERMES (windows-test + E2E multi-noeuds)
4. [ ] 2 apps fonctionnelles dans examples/
5. [ ] Storage persistence SQLite operationnel
6. [ ] CLAUDE.md + HARDENING_ROADMAP + SPRINT_LOG a jour
7. [ ] verification.md + sprint58_audit_plan.md ecrits
8. [ ] PATTERNS.md §P46 cross-platform strategy
