# Recherche S49 — Migration finale coordinator Python → Rust

**Date** : 2026-05-01 (post-S48)
**Contexte** : le roadmap prescrivait la suppression Python a S45.
S46-S48 ont derive (tests/carries/dette). Ce document prépare la
reprise de la migration pour S49.

---

## 1. Inventaire factuel Python restant

### 1.1 Modules avec equivalent Rust (Python ACTIF, Rust INACTIF)

Ces modules existent en double — Python actif dans le coordinator,
Rust present dans nexus-coordinator-rs mais servant uniquement les
endpoints HTTP du daemon. La migration = wirer le daemon pour
utiliser les modules Rust dans son lifecycle, pas cote endpoint.

| Python | LOC | Rust equivalent | LOC Rust |
|---|---|---|---|
| dispatcher.py | 359 | dispatcher.rs | 205 |
| validator.py | 356 | validator.rs | 226 |
| kudos.py | 344 | kudos_ledger.rs | 236 |
| canary_input.py | 782 | canary_input.rs | 893 |
| canary_registry.py | 366 | canary_registry.rs | 426 |
| pii_redactor.py | 483 | pii_redactor.rs | 366 |
| output_filter.py | 397 | output_filter.rs | 266 |
| upload_queue.py | 396 | upload_queue.rs | 198 |
| quarantine_queue.py | 369 | quarantine_queue.rs | 209 |
| contributor_registry.py | 281 | contributor_registry.rs | 202 |
| capability_store.py | 274 | capability_store.rs | 222 |
| invite.py | 216 | invite.rs | 229 |
| rerun.py | 192 | rerun.rs | 131 |
| guardrails.py | 137 | guardrails.rs | 301 |
| **Total** | **4,952** | | **3,910** |

**Action** : ces 4952 LOC Python sont DELETE pur — les modules
Rust existent deja. Il suffit de wirer le daemon pour les appeler
dans son lifecycle (dispatch, validate, credit, etc.) au lieu de
proxier au coordinator Python.

### 1.2 Modules SANS equivalent Rust (a porter ou supprimer)

| Python | LOC | Action proposee |
|---|---|---|
| coordinator.py | 833 | **ABSORBER dans runtime.rs** — le daemon boot deja le node iroh, il suffit d'ajouter le dispatch loop + validator subscription |
| api/daemon.py | 290 | **DELETE** — proxy inutile quand daemon = coordinator |
| api/app.py | 134 | **DELETE** — FastAPI factory, remplacee par l'axum router existant |
| api/events.py | 195 | **PORTER** — SSE bridge via axum (tower-http SSE ou tokio broadcast) |
| mcp_server.py | 176 | **PORTER ou DEFER** — MCP server Rust (mcp crate), evaluer si critique v1.0 |
| keystore.py | 114 | **DELETE** — le daemon gere les keypairs via iroh + nexus_core_rs::LocalFileKeyStore |
| hooks.py | 94 | **ABSORBER** — integrer dans dispatcher.rs (hooks = callbacks pre/post dispatch) |
| tor_client.py | 92 | **DELETE** — arti-client deja feature-gated dans nexus-core-rs |
| peer_creds.py | 92 | **ABSORBER** — integrer dans auth.rs daemon |
| admin_check.py | 74 | **ABSORBER** — 2 fonctions, integrer dans launcher |
| db/migrations.py | 46 | **DELETE** — rusqlite_migration 2.2 deja en place |
| CLI 8 fichiers | 1,025 | **PORTER** — clap subcommands dans nexus-shell-daemon |
| **Total** | **3,165** | |

### 1.3 Packages entiers a supprimer

| Package | LOC | Action |
|---|---|---|
| packages/nexus-coordinator/ | ~9,400 | DELETE entier |
| packages/nexus-sdk/ | ~4,088 | DELETE — modele archive depuis S12 |
| packages/nexus-app-gov/ | ~2,800 | DELETE ou convertir en archive HTML |
| crates/nexus-core-py/ | ~2,000 | DELETE — PyO3 bindings inutiles |
| **Total** | **~18,288** | |

---

## 2. Analyse du coordinator.py — ce qui doit etre absorbe

Le `Coordinator.start()` fait 14 etapes. Analyse par etape :

| # | Etape | LOC | Le daemon le fait deja ? | Action |
|---|---|---|---|---|
| 1 | Load keypair | 10 | OUI (iroh keypair) | DELETE |
| 2 | Init CanaryInputManager | 5 | OUI (http_state.canary_input) | DELETE |
| 3 | Boot iroh Node | 15 | OUI (runtime.rs create_node) | DELETE |
| 4 | Create/reopen Doc | 20 | PARTIEL (daemon create, pas reopen) | PORTER ~20 LOC |
| 5 | Mint write ticket | 5 | NON | PORTER ~10 LOC |
| 6 | Save config | 10 | NON (daemon pas de coordinator.toml) | EVALUER |
| 7 | Init Dispatcher | 15 | OUI (dispatcher.rs existe) | WIRER |
| 8 | Init KudosLedger | 10 | OUI (kudos_ledger.rs) | WIRER |
| 9 | Init UploadQueue + start | 15 | OUI (upload_queue.rs) | WIRER |
| 10 | Init QuarantineQueue + start | 15 | OUI (quarantine_queue.rs) | WIRER |
| 11 | Init TorClient | 10 | OUI (arti-client feature) | WIRER |
| 12 | Init Validator + start | 20 | OUI (validator.rs + validator_loop.rs) | WIRER |
| 13 | Init InviteLedger | 10 | OUI (invite.rs) | WIRER |
| 14 | Discover SDK apps | 150 | NON — mais LEGACY (modele archive S12) | DELETE |

**Bilan** : sur 833 LOC, ~650 LOC sont deja couvertes par le
daemon Rust (DELETE ou WIRER). ~30 LOC de doc iroh a porter.
~150 LOC d'app discovery a supprimer (legacy SDK).

---

## 3. Le systeme d'apps — pourquoi c'est DELETE pas PORT

Depuis Sprint 12, le modele de rendu est **archive-based** :
- Les apps publient un zip (index.html + assets)
- Le daemon blob-serve decompresse et sert dans un iframe sandbox
- Le bridge postMessage (3 methodes) est le seul canal
- Le SDK Python NexusApp est un vestige pre-S12

**app-gov** est la seule app SDK restante. Options :
1. Convertir en archive HTML (React build → zip) — 1 phase
2. Supprimer et recreer post-v1.0 — si non critique v1.0
3. Garder comme legacy Python — **NON** (contredit migration)

L'option 1 est la plus coherente : app-gov devient une app
SBFB standard deployee comme toutes les autres.

---

## 4. Plan de migration en phases

### S49 — Coordinator lifecycle → daemon (3 phases)

**Phase A : dispatch loop + validator subscription dans daemon**
- runtime.rs : au demarrage, le daemon cree un iroh doc project
  (ou le reopen si existant), configure le dispatcher.rs pour
  ecrire les TaskEntry, lance le validator_loop.rs pour subscriber
  et valider les resultats, credite kudos via kudos_ledger.rs
- Le coordinator Python n'est plus necessaire pour le core path
- ~200 LOC Rust (wiring dans runtime.rs + integration tests)

**Phase B : CLI migration**
- Les commandes `init`, `start`, `canary`, `invite`, `quarantine`,
  `capability` deviennent des subcommands clap dans le binaire
  nexus-shell-daemon. `nexus-shell-daemon start` = ce que fait
  aujourd'hui `nexus-coordinator start` + `nexus-shell-daemon` en
  parallele. Un seul process.
- ~400 LOC Rust (clap derive, delegue aux modules existants)

**Phase C : conversion app-gov → archive HTML**
- Build React du frontend app-gov (si UI existante) ou creer
  une UI HTML minimale pour les 19 tabs gov
- Publier comme archive zip standard sur le reseau
- Supprimer la dependance NexusApp/AppContext

### S50 — Suppression Python + cleanup (2-3 phases)

**Phase A : DELETE packages Python**
- `git rm -r packages/nexus-coordinator/`
- `git rm -r packages/nexus-sdk/`
- `git rm -r packages/nexus-app-gov/`
- `git rm -r crates/nexus-core-py/`
- Nettoyer pyproject.toml workspace, Cargo.toml (pyo3 deps)
- Nettoyer uv.lock, .venv references dans docs

**Phase B : porter modules restants**
- events SSE : axum SSE handler (tokio broadcast channel)
- hooks.py : integrer dans dispatcher.rs
- peer_creds + admin_check : integrer dans daemon
- MCP server : evaluer si port Rust ou defer post-v1.0

**Phase C : tests + docs**
- Adapter CLAUDE.md (supprimer sections Python)
- Adapter README, BUILDING.md
- Verifier que les 3 blocs fail-fast deviennent 2 (Rust + Frontend)
- Tests E2E : daemon seul boot → dispatch → validate → kudos

### S51+ — CI/CD + deploy + polish (cf. roadmap S51-S56)

---

## 5. Dependencies Python a supprimer

Toutes les deps Python ont un equivalent Rust deja en place :

| Python dep | Rust equivalent | Statut |
|---|---|---|
| fastapi + uvicorn | axum + hyper | EN PLACE |
| pydantic | serde | EN PLACE |
| httpx | reqwest | EN PLACE |
| typer + rich | clap | EN PLACE |
| aiosqlite | rusqlite | EN PLACE |
| pynacl | ed25519-dalek | EN PLACE |
| structlog | tracing | EN PLACE |
| platformdirs | dirs | EN PLACE |
| tomli-w | toml | EN PLACE |
| jcs | jcs crate | EN PLACE |
| nexus-core-py (PyO3) | nexus-core-rs | EN PLACE |

**Zero gap de dep.** La migration est purement du wiring.

---

## 6. Risques

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | Doc iroh reopen semantique differente Rust vs Python | Medium | Tests integration pre-commit |
| R2 | Validator subscription loop complexe a wirer | Medium | validator_loop.rs existe deja, enrichir |
| R3 | app-gov conversion HTML perd des features | Low | app-gov est WIP, features minimales |
| R4 | MCP server port Rust = dep mcp crate instable | Medium | Defer post-v1.0 si crate pas mature |
| R5 | Tests Python non convertibles en Rust | Low | La plupart testent du code deja couvert Rust |
