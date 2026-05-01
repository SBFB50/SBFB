# Sprint 50 — Kickoff (suppression Python + dette pair)

**Ecrit** : 2026-05-01 (post-audit gate S49 PASS `da489a4`).
**Type** : **sprint pair** — phase dette obligatoire (§6.2.1
Regle 1).
**Tip master d'entree** : `da489a4`.
**Phase 0 audit Sprint 49** : **DEJA JOUE** — `da489a4` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-01. 5 fichiers
  security + PATTERNS avec triggers_revalidate. 0 trigger actif
  (iroh reste 0.98, wasmtime absent, aucun evenement externe).
  Pas de pre-research.

- **Technologies S50** : aucune nouvelle dep externe. Sprint
  purement soustractif (suppression Python) + fixes dette Rust
  existants. Aucune lib a consulter via context7.

- **ROADMAP_COMMITMENTS check** : LT-1..LT-5 latents (tag v1.0
  non pose). LT-6 RESOLVED. 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S50 prescrite.

- **Roadmap migration** : `roadmap_v1_migration_rust.md` prescrit
  S50 = "suppression Python + cleanup".

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 49 CLOSED + audit PASS. Le daemon Rust est desormais
le coordinator pour le projet local — project doc iroh-docs,
dispatch loop MPSC, 4 CLI subcommands offline. Le coordinator
Python n'est plus necessaire pour le core path.

**Etat factuel Python** (inventaire S50) :
- `packages/nexus-coordinator/` : ~16 585 LOC Python, 33 test
  files, 264+17f+6s pytest. Les 14 modules metier ont TOUS un
  equivalent Rust actif dans `nexus-coordinator-rs` + le daemon.
  Le FastAPI HTTP server est remplace par l'axum router du daemon
  (54 routes natives depuis S38).
- `packages/nexus-sdk/` : ~7 992 LOC Python, 13 test files,
  195 pytest. SDK NexusApp ABC — vestige pre-S12 (modele archive
  depuis S12).
- `packages/nexus-app-gov/` : ~4 912 LOC Python, 5 test files,
  46 pytest. WIP governance app utilisant le SDK obsolete.
- `crates/nexus-core-py/` : ~1 364 LOC Rust (PyO3 bindings).
  Plus aucun consommateur actif cote daemon.
- **Total a supprimer** : ~30 853 LOC + ~505 tests Python
- **Deps workspace** : `pyo3 = "0.28"` + `pyo3-async-runtimes`
  dans `Cargo.toml` racine, `pyproject.toml` workspace

**Frontend impact** : le hook `useAppEvents` (SSE) dans
`AppTabPage.tsx` cible le coordinator Python. AppTabPage est
le rendu SDK legacy (pre-S12, remplace par l'iframe archive
model). Avec la suppression du coordinator, ce hook devient
dead code. Le composant lui-meme est dead code car les apps
SBFB utilisent le chemin archive (iframe via blob-serve).

**MCP server** : `mcp_server.py` (176 LOC) — standalone, 0
import depuis le coordinator. Pas de consommateur en prod.
Port Rust ou defer post-v1.0.

### §1.2 Ancrage roadmap migration

`roadmap_v1_migration_rust.md` §S50 : "suppression Python +
cleanup — DELETE packages Python + porter modules restants +
tests + docs". S49 a livre le lifecycle (Phase A) et la CLI
(Phase B). S50 complete la migration par la suppression.

### §1.3 Compteurs tests entree (tip `da489a4`)

| Suite | Count |
|---|---|
| Rust nextest | 1195 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 264 + 17 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 5/5 |
| **Total** | **~1947** |

**Post-S50 attendu** : ~1509 (suppression ~505 Python tests
+ 17 fails + 6 skips — tests Python deviennent inapplicables,
pas de regression Rust/Frontend).

### §1.4 Pre-launch protocol policy (rappel)

Sprint purement soustractif — aucun wire format touche, aucun
`*_FORMAT_VERSION` impacte.

---

## §2 Goal

Eliminer completement le code Python du projet : les 4 packages
(coordinator, SDK, app-gov, core-py) sont supprimes, les deps
workspace nettoyees, les docs mises a jour, et le fail-fast
checklist passe de 3 blocs (Rust+Python+Frontend) a 2 blocs
(Rust+Frontend). Le daemon Rust est le seul point d'entree
pour le coordinateur, les CLI, et les API HTTP.
**Critere SMART : 20+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up. 0 LOC Python restant dans
packages/ et crates/nexus-core-py/.**

---

## §3 Phase 0 — Audit gate S49

**DEJA JOUE** : commit `da489a4` PASS (0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint49_audit_findings.md`.
8 carries documentes pour S50 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Suppression totale vs incrementale

**Retenu** : suppression en bloc des 4 packages Python +
crate PyO3 en une seule phase. Le daemon Rust couvre 100% du
core path (dispatch, validate, kudos, CLI, HTTP API) depuis
S49. Les 14 modules metier Python ont tous un equivalent Rust
actif. Aucune dependance runtime Rust→Python ne subsiste.

**Rejete** :
- Suppression incrementale package par package : ajoute de la
  complexite inter-phases pour zero benefice (aucun package n'a
  de consommateur Rust residuel).
- Garder nexus-sdk pour un futur SDK Python : le modele est
  archive-based (S12). Un futur SDK serait TypeScript/WASM, pas
  Python.

**Implications code** : `git rm -r` des 4 directories + cleanup
pyproject.toml + Cargo.toml workspace (pyo3 deps).

### D2 — app-gov disposition : supprimer sans conversion

**Retenu** : supprimer app-gov sans conversion prealable en
archive HTML. app-gov est WIP (19 tabs migration S8, jamais
achevee), utilise le SDK Python obsolete (NexusApp ABC),
et n'est pas critique pour v1.0. Recreer en tant qu'app SBFB
standard (React archive) post-v1.0 si gouvernance necessaire.

**Rejete** :
- Convertir app-gov en archive HTML puis supprimer : effort
  disproportionne (19 tabs → React app standalone) pour une app
  WIP qui n'a pas de users. Le format SBFB (archive zip) est le
  bon modele — mais la creation doit etre un travail dedie, pas
  un effet de bord de la suppression Python.
- Garder app-gov comme seul package Python : contredit le goal
  "0 LOC Python".

### D3 — SSE events et AppTabPage : dead code cleanup

**Retenu** : supprimer le hook `useAppEvents` et le composant
`AppTabPage` du frontend. Ces composants servent le modele SDK
pre-S12 (apps Python rendues server-side). Le modele actuel
est archive-based (iframe via blob-serve + bridge postMessage).
Avec la suppression du coordinator Python, le endpoint SSE
disparait et ces composants deviennent non-fonctionnels.

**Rejete** :
- Porter SSE dans le daemon Rust : pas de consommateur cote
  frontend (AppTabPage est legacy). Un futur SSE daemon-native
  (monitoring, live status) serait un ajout post-v1.0 avec un
  design propre.
- Garder les composants comme dead code : dette frontend
  gratuite a eliminer.

### D4 — MCP server : supprimer, evaluer port post-v1.0

**Retenu** : supprimer `mcp_server.py` avec le coordinator.
Le MCP server Python (176 LOC) est standalone (0 import
depuis le coordinator), n'a aucun consommateur en production,
et un port Rust necessiterait la crate `mcp` dont la maturite
n'est pas evaluee. Le sprint S50 est soustractif — pas de
nouvelle dep.

**Rejete** :
- Porter le MCP server en Rust dans S50 : ajoute une dep
  nouvelle (crate mcp) dans un sprint soustractif.
- Garder le MCP server Python seul : impossible si le goal est
  0 LOC Python.

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ⚠️.
Rigor signal G4 satisfait (1 ⚠️ sur 4, 0 ❌).

D4 ⚠️ (MCP server spec perte) : le reviewer signale que
`mcp_server.py` contient du code defensif (capability gate,
loopback auth, tool whitelist `task_submit`/`storage_get`/
`storage_set`) qui represente un intent reel (Sprint 26 Phase B).
Decision : la spec MCP (3 methodes whitelist + capability gate)
est DEJA documentee dans `docs/security/CAPABILITY_TOGGLES.md`
et dans `CLAUDE.md §Modele de rendu` (bridge postMessage =
memes 3 methodes). Le port Rust utilisera cette spec comme
reference. Pas de perte d'information a la suppression du .py.
Le scope cut §7 documente explicitement "MCP server Rust —
post-v1.0".

---

## §5 Plan Phase outline A..C

### Phase A — Dette pair obligatoire (S50 pair, §6.2.1 Regle 1)

**But** : resoudre 2 carries P2 de S49 + fermer le carry
memory tip (deja fixe audit session).

- P2-REVIEW-A-1-S49 dispatch loop JoinHandle : stocker le
  handle dans DaemonRuntime, abort on shutdown.
- P2-REVIEW-B-1-S49 CLI handler integration tests : ajouter
  des tests qui ouvrent un CoordinatorDb tempdir et exercent
  les handlers init/invite/quarantine/capability.
- P2-AUDIT-A-1-S49 memory tip stale : CLOSE (fixe dans la
  session audit S49).
- Commit : `feat(sprint50): Sprint 50 Phase A — dette pair
  dispatch JoinHandle + CLI integration tests`

### Phase B — Suppression Python + PyO3

**But** : supprimer 100% du code Python (4 packages + crate
PyO3) et nettoyer les configs workspace.

- `git rm -r packages/nexus-coordinator/`
- `git rm -r packages/nexus-sdk/`
- `git rm -r packages/nexus-app-gov/`
- `git rm -r crates/nexus-core-py/`
- Nettoyer `pyproject.toml` (supprimer workspace members, garder
  le fichier si des dev-deps Python restent pour tooling — sinon
  supprimer)
- Nettoyer `Cargo.toml` workspace : supprimer `pyo3` et
  `pyo3-async-runtimes` des `[workspace.dependencies]`,
  supprimer `nexus-core-py` des `members`
- Supprimer les references Python dans `uv.lock` (regenerer ou
  supprimer)
- Supprimer les composants frontend dead code : `useAppEvents`
  hook, `AppTabPage` composant (modele SDK pre-S12)
- Commit : `feat(sprint50): Sprint 50 Phase B — suppression
  Python packages + PyO3 + frontend dead code`

### Phase C — Docs + verification + wrap-up

**But** : mettre a jour la documentation pour refleter la
suppression Python et le passage a un projet Rust+Frontend
pur.

- CLAUDE.md : supprimer les commandes Python du §Commandes cles,
  mettre a jour la structure des crates/packages, ajuster les
  compteurs tests
- docs/claude/README.md : fail-fast checklist passe de 3 blocs
  a 2 blocs (Rust + Frontend)
- Verification fail-fast 20+ checks (2 blocs restants)
- sprint51_audit_plan.md
- Compteurs tests post-suppression
- Commit : `chore(sprint50): Phase C — wrap-up + verification +
  audit plan S51 + counters`

---

## §6 Items carry/dette

### Carries confirmes S50

- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption
  blocker externe. Justification renouvelee : pas de release rand
  0.9 ni fix getrandom upstream.
- [carry] **P2-AUDIT-2** pre-release transitives iroh : herite
  pin 0.98 (Day 0 #3).
- [carry] **P2-REVIEW-A-1-S48** canary reload size cap 1/3 :
  pre-v1.0 fichier local controle par operateur.
- [carry] **P2-REVIEW-B-1-S48** auth.rs set_var residuel 1/3 :
  4 set_var dans auth.rs. Non bloquant.
- [carry] **P2-AUDIT-A-1-S48** carry doc accuracy reload_policy
  1/3 : asymetrie lock scope documentation.
- [dette] **P2-REVIEW-A-1-S49** dispatch loop JoinHandle 1/3 :
  **ADRESSE Phase A** → CLOSE attendu.
- [dette] **P2-REVIEW-B-1-S49** CLI handler integration tests
  1/3 : **ADRESSE Phase A** → CLOSE attendu.
- [close] **P2-AUDIT-A-1-S49** memory tip stale 1/3 : **DEJA
  FIXE** session audit → CLOSE.

### Sprint pair — phase dette obligatoire

S50 pair → Phase A reservee dette (§6.2.1 Regle 1). 2 items
adresses (#6, #7) + 1 ferme (#8).
0 item a 2/3. 0 item a 3/3. Aucune escalade §6.2.1 Regle 2.

### Carries residuels post-S50

5 carries non adresses (items #1-#5). Tous a 1/3 ou exemption
externe. Compteurs incrementes a 2/3 pour #3-#5.

| Item | Compteur S51 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 2/3 | S48 review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 2/3 | S48 review |
| P2-AUDIT-A-1-S48 carry doc accuracy | 2/3 | S48 audit |

---

## §7 Scope cuts

1. **Events SSE daemon-native** — post-v1.0 (design propre si
   besoin, pas port du Python legacy)
2. **MCP server Rust** — post-v1.0 (evaluer crate mcp maturite)
3. **app-gov recreation** — post-v1.0 (recreer en React archive
   SBFB si gouvernance necessaire)
4. **CI/CD + binaires + installer** — S51
5. **VPS deployment + smoke test** — S52
6. **Kudos debit/stake** — interdit (Day 0 #7)
7. **Pagination SQL-side LIMIT/OFFSET** — S51+
8. **Test infra mk_state() refactoring** — S51+

---

## §8 Tracabilite scope (S49 → S50)

| S49 scope cut | S50 disposition |
|---|---|
| app-gov conversion archive HTML — S50 | **D2** : supprimer sans conversion |
| events.py SSE streaming — S50 | **D3** : supprimer (legacy SDK) |
| MCP server migration Rust — S50 | **D4** : supprimer, evaluer port post-v1.0 |
| PyO3 bindings removal — S50 | **Phase B** DELETE |
| Suppression complete coordinator Python — S50 | **Phase B** DELETE |
| Suppression SDK Python — S50 | **Phase B** DELETE |
| CI/CD + binaires + installer — S51 | Scope cut reporte S51 |
| VPS deployment + smoke test — S52 | Scope cut reporte S52 |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Frontend compile errors apres suppression composants legacy (imports orphelins) | Medium | Low | tsc + npm lint + Vitest en fail-fast |
| R2 | pyproject.toml / uv.lock cleanup incomplete (ruff/mypy cessent de fonctionner pour le tooling restant) | Low | Low | Si aucun Python reste, supprimer pyproject.toml entierement |
| R3 | Cargo.toml workspace cleanup casse le build (dep pyo3 residuelle non supprimee) | Low | Medium | cargo build --workspace apres cleanup |
| R4 | Dead code frontend supplementaire non detecte (autres composants SDK legacy) | Medium | Low | Grep systematique "NexusApp\|AppContext\|coordinator" dans web/src/ |
| R5 | Playwright tests touchent des pages qui referent aux composants supprimes | Low | Medium | npx playwright test en fail-fast |

---

## §10 Audit gate pattern — rappel

Phase 0 S49 jouee (PASS `da489a4`). Phase C produira
sprint51_audit_plan.md pour la session fraiche S51.

---

## §11 Checkpoint de validation

1. **D1** : suppression totale vs incrementale ?
   → totale (daemon 100% autonome, 0 dep Rust→Python)
2. **D2** : app-gov conversion vs suppression ?
   → suppression (WIP, effort disproportionne, post-v1.0)
3. **D3** : SSE port vs dead code cleanup ?
   → cleanup (legacy SDK model, 0 consommateur actuel)
4. **D4** : MCP port vs suppression ?
   → suppression (standalone, 0 consommateur, crate immature)
