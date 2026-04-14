# Sprint 13 — Plan detaille (Bridge postMessage + open source + UI Netflix + launcher)

**Ecrit** : 2026-04-13, apres kickoff valide.
**Tip master d'entree** : `53a9e32`
**Decisions Day 0** : D1-D6 gelees (cf. `sprint13_kickoff.md` §4)

---

## 1. Etat verifie a l'entree

| Suite | Count | Status |
|---|---|---|
| Rust workspace | 362 | green |
| Python SDK | 182 | green |
| Python coordinator | 96 + 1 skipped | green |
| Python app-gov | 46 | green |
| Vitest unit | 180 (7 FAIL uncommitted) | yellow |
| Playwright | 30 | green |
| size-limit | 7/7 | green |
| SPDX check | 215/215 | green |

**Changements non commites a l'entree** :
- `web/src/components/AppShell.tsx` — redesign rail nav glassmorphism
- `web/src/index.css` — classes utilitaires `.glass-card` + `.glass-pill`
- `web/src/pages/Browse.tsx` — hero section + glass cards + glass pills
- `web/src/pages/BrowsedProject.tsx` — full-screen + auto-hide glass header
- Total : +630 / -577 lignes, 7 Vitest BrowsedProject FAIL

**Fichiers non trackes hors scope** :
- `cc.json`, `docs/DND_P2P_DESIGN.md`, `docs/VISION_USE_CASES.md`,
  `docs/apps/`, `site/` — design docs session 2026-04-13, pas dans
  ce sprint

**Infrastructure existante a etendre** :
- `POST /publish` (http.rs) : accepte metadata JSON, pas de `repo_url`
- `ProjectAnnouncement` (publish.rs) : v=1, champs existants + `archive_ticket`
- `BrowseEntry` (browse.rs) : champs existants + `archive_hash`
- `POST /app/{name}/tasks/submit` (coordinator) : endpoint REST existant
- `GET/PUT /app/{name}/state/{ns_key}` (coordinator) : storage KV existant
- `WebAppFrame.tsx` / `BrowsedProject.tsx` : iframe sandbox fonctionnel
- `useBrowseEntries` hook : React Query pour les entries

---

## 2. Decisions Day 0 (gelees — rappel synthetique)

- **D1** : public = open source (`repo_url` obligatoire), prive = libre
- **D2** : UI Netflix glassmorphism dark-first (uncommitted = base)
- **D3** : postMessage bridge MVP (request/response + correlation IDs)
- **D4** : launcher Rust minimal (pas Tauri, crate `open`)
- **D5** : T37-T40 fermes en Phase A
- **D6** : CPU watchdog differe Sprint 14

---

## 3. Phase A — UI Netflix glassmorphism + tech debt T37-T40

### 3.1 Formaliser les changements non commites

Les 4 fichiers modifies (AppShell.tsx, Browse.tsx, BrowsedProject.tsx,
index.css) sont adoptes comme base. Avant de commiter :

1. **Fix 7 Vitest BrowsedProject** : les tests cherchent des
   elements DOM qui ont change (back link, sidebar structure,
   iframe wrapper, banner). Mettre a jour
   `web/src/pages/__tests__/BrowsedProject.test.tsx` pour
   matcher la nouvelle structure HTML.

2. **Verifier** : `npm run test:unit` → 180+ passes, 0 fail.

### 3.2 Etendre glassmorphism aux pages restantes

Fichiers a modifier :

- **`web/src/pages/Projects.tsx`** (~40 LOC delta) :
  Remplacer les `<Card>` shadcn par des divs `.glass-card`.
  Ajouter hover glow effect. Status dots colores.

- **`web/src/pages/ProjectDetail.tsx`** (~80 LOC delta) :
  Hero gradient en haut. 5 tabs avec glass styling.
  Cards dans chaque tab → `.glass-card`.

- **`web/src/pages/Network.tsx`** (~60 LOC delta) :
  4 metric cards → `.glass-card` avec glow semantique.
  Progress bars avec containers glass.

- **`web/src/pages/Curators.tsx`** (~50 LOC delta) :
  Form container en glass. Curator list cards en `.glass-card`.
  Subscription badges en `.glass-pill`.

### 3.3 Tech debt T37 — CSP middleware blob-serve

**Fichier** : `crates/nexus-shell-daemon/src/http.rs`

Ajouter un middleware tower sur le groupe de routes `/blob-serve/*`
qui injecte les headers CSP sur TOUTES les reponses (200, 400, 404,
500). Utiliser `tower_http::set_header::SetResponseHeaderLayer` ou
un middleware custom `axum::middleware::from_fn`.

```rust
// Pseudo-code structure
async fn blob_serve_csp_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "content-security-policy",
        BLOB_SERVE_CSP.parse().unwrap(),
    );
    response.headers_mut().insert(
        "x-content-type-options",
        "nosniff".parse().unwrap(),
    );
    response
}
```

Retirer les headers CSP du handler 200 (maintenant geres par le
middleware). ~25 LOC.

**Test** : ajouter `blob_serve_error_responses_have_csp()` qui
verifie qu'un GET sur un hash inconnu retourne 404 AVEC les
headers CSP.

### 3.4 Tech debt T38 — SVG chart dimensions

**Fichier** : `packages/nexus-sdk/src/nexus_sdk/html_render.py`

Aligner les constantes sur les valeurs React :

| Parametre | Actuel | Cible (React) |
|-----------|--------|---------------|
| H | 180 | 120 |
| PAD_L | 45 | 32 |
| PAD_R | 10 | 16 |
| PAD_T | 10 | 16 (line) / 24 (bar) |
| PAD_B | 30 | 16 (line) / 24 (bar) |

Modifier `_render_chart_line()` et `_render_chart_bar()` avec les
bonnes constantes. ~35 LOC delta.

**Test** : les tests existants `test_render_chart_line` et
`test_render_chart_bar` valident les dimensions via les attributs
SVG viewBox/width/height.

### 3.5 Tech debt T39 — Test file_upload

**Fichier** : `packages/nexus-sdk/tests/test_html_render.py`

Ajouter `test_render_file_upload()` qui verifie que le block
`file_upload` produit un HTML contenant un `<form>` ou placeholder
avec le label echappe. ~20 LOC.

### 3.6 Tech debt T40 — nginx X-Real-IP

**Fichier** : `deploy/nginx-nexus.conf`

Ajouter `proxy_set_header X-Real-IP $remote_addr;` dans le bloc
`location /blob-serve/`. 1 ligne.

### 3.7 Critere d'acceptation Phase A

- `npm run test:unit` : 180+ passes, 0 fail
- `npx tsc --noEmit` : exit 0
- `npm run lint` : 0 errors
- `npm run build` : exit 0
- `npm run size` : 7/7 green
- `cargo test --workspace --locked` : 363+ (1 nouveau test CSP)
- `uv run pytest packages/nexus-sdk/tests/ -q` : 183+ (1 nouveau test file_upload)
- `cargo fmt --all --check` : exit 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0
- T37-T40 : grep CLOSED dans PATTERNS.md = 4 nouveaux
- Toutes les pages ont le design glassmorphism
- `bash scripts/scan-en-strings.sh` : clean

### 3.8 Commit cible

```
feat(web): Sprint 13 Phase A — UI Netflix glassmorphism + T37-T40

Formalise les changements UI glassmorphism non commites
(AppShell rail nav, Browse hero/glass-cards, BrowsedProject
full-screen auto-hide header) et etend le design aux pages
restantes (Projects, ProjectDetail, Network, Curators).

Fix les 7 Vitest BrowsedProject casses par le redesign.
Ferme T37 (CSP middleware blob-serve), T38 (SVG chart
dimensions alignees React), T39 (test file_upload), T40
(nginx X-Real-IP).

Rust workspace:           362 → 363+ (+1 CSP error test)
Python SDK:               182 → 183+ (+1 file_upload test)
Python coordinator:       96+1 → 96+1 (inchange)
Vitest unit:              180 → 180+ (7 fix + N nouveaux)
Playwright:               30 → 30 (inchange)

Scope cuts honoured:
- NOT branding SBFB (Sprint 14)
- NOT CPU watchdog (Sprint 14, D6)
```

---

## 4. Phase B — Open source enforcement

### 4.1 Rust : ProjectAnnouncement + BrowseEntry

**Fichier** : `crates/nexus-shell-daemon-core/src/publish.rs`

Ajouter a `ProjectAnnouncement` :
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub repo_url: Option<String>,
```

**Fichier** : `crates/nexus-shell-daemon-core/src/browse.rs`

Ajouter a `BrowseEntry` :
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub repo_url: Option<String>,
```

Propager `repo_url` de l'announcement vers l'entry dans
`process_announcement()`. ~15 LOC.

**Tests Rust** :
- `announcement_v3_with_repo_url_roundtrip()` : serde roundtrip
- `v2_announcement_parses_without_repo_url()` : backward compat
- `browse_entry_includes_repo_url()` : propagation
~30 LOC tests.

### 4.2 Coordinator : validation publish

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`

Dans `deploy_project()`, si `visibility == "public"` et `repo_url`
absent ou vide → HTTP 400 "Public projects require a repo_url
(link to public source code repository)".

**Fichier** : `crates/nexus-shell-daemon/src/http.rs`

Dans le handler `POST /publish`, accepter `repo_url` dans le JSON
body et le propager au `ProjectAnnouncement`.

**Tests Python** :
- `test_deploy_public_without_repo_url_rejected()` : 400
- `test_deploy_public_with_repo_url_accepted()` : 200
- `test_deploy_private_without_repo_url_accepted()` : 200
~40 LOC tests.

### 4.3 Frontend : Zod + UI

**Fichier** : `web/src/api/daemon.ts`

Ajouter `repo_url: z.string().optional()` dans `BrowseEntrySchema`
et `ProjectAnnouncementSchema`.

**Fichier** : `web/src/pages/Browse.tsx`

Sur chaque BrowseCard, si `entry.repo_url` existe, afficher un
lien glass-pill avec icone (lien externe) pointant vers le repo.

**Fichier** : `web/src/pages/BrowsedProject.tsx`

Dans la sidebar metadata, afficher le lien repo si present.
Glass-pill cliquable.

**Tests Vitest** :
- `renders repo link for public entry with repo_url`
- `does not render repo link when repo_url absent`
~25 LOC tests.

### 4.4 Critere d'acceptation Phase B

- `cargo test --workspace --locked` : +3 tests (365+)
- `uv run pytest packages/nexus-coordinator/tests/ -q` : +3 tests (99+1)
- `npm run test:unit` : +2 tests
- Backward compat : un BrowseEntry sans `repo_url` parse OK (Zod + serde)
- Un `POST /project/deploy` public sans `repo_url` → 400
- Un `POST /project/deploy` prive sans `repo_url` → OK
- Lien repo visible dans Browse + BrowsedProject

### 4.5 Commit cible

```
feat(p2p): Sprint 13 Phase B — open source enforcement for public apps

Public apps must provide a repo_url (link to public source code
repository). Private apps have no constraint. This is a P2P
security principle: code distributed to strangers must be
auditable.

ProjectAnnouncement v3 + BrowseEntry gain repo_url field.
Coordinator deploy validates repo_url for public visibility.
Shell displays clickable repo link on Browse cards and
BrowsedProject sidebar.

Rust workspace:           363 → 366+ (+3 serde + compat tests)
Python coordinator:       96+1 → 99+1 (+3 deploy validation)
Vitest unit:              180+ → 182+ (+2 repo link tests)

Scope cuts honoured:
- NOT GitHub API verification (Sprint 14, trust repo_url)
- NOT repo_url on existing apps retroactively
```

---

## 5. Phase C — postMessage bridge MVP

### 5.1 Message protocol

**Nouveau fichier** : `web/src/bridge/protocol.ts` (~80 LOC)

Schema Zod des messages bridge :

```typescript
// Direction : iframe → host
const BridgeRequestSchema = z.object({
  type: z.literal("sbfb-bridge-request"),
  id: z.string().uuid(),       // correlation ID
  method: z.enum([
    "task_submit",
    "storage_get",
    "storage_set",
  ]),
  payload: z.record(z.unknown()),
});

// Direction : host → iframe
const BridgeResponseSchema = z.object({
  type: z.literal("sbfb-bridge-response"),
  id: z.string().uuid(),       // meme correlation ID
  success: z.boolean(),
  data: z.unknown().optional(),
  error: z.string().optional(),
});
```

Export types inferes + helpers `createRequest()`, `createResponse()`,
`createError()`.

### 5.2 Host bridge listener

**Nouveau fichier** : `web/src/bridge/useBridge.ts` (~120 LOC)

Hook React qui :
1. `useEffect` → `window.addEventListener("message", handler)`
2. `handler` : parse le message avec `BridgeRequestSchema`
3. Ignore les messages qui ne matchent pas (autres extensions, etc.)
4. Verifie que `event.source` est un iframe connu (ref tracking)
5. Dispatch par `method` :
   - `task_submit` → `POST /app/{appName}/tasks/submit` via coordinator client
   - `storage_get` → `GET /app/{appName}/state/{key}` via coordinator client
   - `storage_set` → `PUT /app/{appName}/state/{key}` via coordinator client
6. Envoie la reponse via `event.source.postMessage(response, "*")`

Le hook prend en parametre le `appName` courant (pour scoper les
appels coordinator).

**Integration** : monter `useBridge` dans `BrowsedProject.tsx` quand
une app locale est affichee en iframe.

### 5.3 SDK bridge client

**Nouveau fichier** : `web/public/sbfb-bridge.js` (~150 LOC)

Fichier JS standalone (pas de bundler) que les apps incluent :

```html
<script src="/sbfb-bridge.js"></script>
<script>
  const bridge = new SBFBBridge();
  const result = await bridge.submitTask({
    task_type: "llm",
    prompt: "Hello world",
  });
</script>
```

API :
- `new SBFBBridge(options?)` : init, ecoute les reponses
- `bridge.submitTask(payload)` → Promise<{task_id}>
- `bridge.getStorage(key)` → Promise<value>
- `bridge.setStorage(key, value)` → Promise<void>

Chaque methode :
1. Genere un UUID correlation ID
2. Envoie `parent.postMessage(request, "*")`
3. Ecoute `window.addEventListener("message")` pour la reponse matchant le correlation ID
4. Timeout 10s → reject avec erreur

### 5.4 Tests

**Nouveau fichier** : `web/src/bridge/__tests__/protocol.test.ts` (~40 LOC)
- Validation schema request/response
- Rejection messages malformes

**Nouveau fichier** : `web/src/bridge/__tests__/useBridge.test.ts` (~80 LOC)
- Mock postMessage events
- Dispatch task_submit → mock coordinator call → response envoyee
- Timeout sur requete sans reponse
- Message ignore quand type ne matche pas

### 5.5 Critere d'acceptation Phase C

- `npm run test:unit` : +5 tests bridge minimum
- `npx tsc --noEmit` : exit 0 (typage complet)
- `npm run build` : exit 0 (sbfb-bridge.js copie dans dist)
- `npm run size` : 7/7 green
- Le fichier `sbfb-bridge.js` est servable depuis `/sbfb-bridge.js`
- Integration smoke : `useBridge` monte dans BrowsedProject sans
  erreur console

### 5.6 Commit cible

```
feat(bridge): Sprint 13 Phase C — postMessage bridge MVP with task submit + storage

Apps in iframes can now communicate with the SBFB network via
a postMessage bridge. The host shell listens for typed requests
(task_submit, storage_get, storage_set), forwards them to the
coordinator API, and sends back responses with correlation IDs.

New files:
- web/src/bridge/protocol.ts — Zod message schemas
- web/src/bridge/useBridge.ts — host listener hook
- web/public/sbfb-bridge.js — SDK client for iframe apps

Vitest unit:              182+ → 187+ (+5 bridge tests)

Scope cuts honoured:
- NOT bidirectional push (host → iframe events, Sprint 14)
- NOT CPU watchdog via heartbeat (Sprint 14, D6)
- NOT bridge auth (loopback-only, same as coordinator)
```

---

## 6. Phase D — Rust launcher minimal

### 6.1 Nouveau crate

**Nouveau dossier** : `crates/nexus-launcher/`

```
crates/nexus-launcher/
├── Cargo.toml
└── src/
    └── main.rs
```

**Cargo.toml** :
```toml
[package]
name = "nexus-launcher"
version = "1.0.0"
edition = "2021"
license = "AGPL-3.0-only"

[dependencies]
open = "5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

Ajouter `"crates/nexus-launcher"` dans le workspace members du
`Cargo.toml` racine.

### 6.2 main.rs (~200 LOC)

Structure :

```rust
/// Minimal SBFB launcher — spawns daemon, opens browser, waits for Ctrl+C.

#[derive(Deserialize)]
struct RunningInfo {
    host: String,
    port: u16,
    pid: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Check if daemon already running (read running.json)
    // 2. If not, spawn `nexus-shell-daemon start` as child
    // 3. Poll running.json until it appears (max 10s)
    // 4. Read host:port from running.json
    // 5. open::that(format!("http://{}:{}", host, port))
    // 6. Wait for Ctrl+C (tokio::signal)
    // 7. If we spawned the daemon, send SIGTERM / kill child
    // 8. Wait for child exit (timeout 5s)
    // 9. Exit 0
}
```

Fonctions auxiliaires :
- `find_running_json() -> PathBuf` : cherche dans les paths connus
- `read_running_info(path) -> Option<RunningInfo>` : parse JSON
- `wait_for_running(path, timeout) -> Result<RunningInfo>` : poll
- `spawn_daemon() -> Child` : lance le daemon

### 6.3 Tests

**Test integration** dans `main.rs` ou fichier separe :
- `test_read_running_info()` : parse un JSON fixture
- `test_read_running_info_missing()` : retourne None
- `test_find_running_json_path()` : verifie le path attendu
~50 LOC tests.

### 6.4 SPDX

Ajouter le header SPDX sur `Cargo.toml` et `main.rs`.

### 6.5 Critere d'acceptation Phase D

- `cargo build -p nexus-launcher` : exit 0
- `cargo test -p nexus-launcher` : 3+ tests verts
- `cargo fmt --all --check` : exit 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0
- `cargo test --workspace --locked` : 369+ (366 + 3 launcher)
- SPDX : 217+ (2 nouveaux fichiers)
- Le binary existe et affiche un message d'aide avec `--help`

### 6.6 Commit cible

```
feat(launcher): Sprint 13 Phase D — minimal Rust launcher with browser open

New crate nexus-launcher: spawns nexus-shell-daemon as child
process, reads running.json for bound port, opens default
browser via `open` crate, waits for Ctrl+C, graceful shutdown.

No Tauri, no native window — the browser IS the client.

New files:
- crates/nexus-launcher/Cargo.toml
- crates/nexus-launcher/src/main.rs (~200 LOC)

Rust workspace:           366 → 369+ (+3 launcher tests)
SPDX:                     215 → 217+ (+2 files)

Scope cuts honoured:
- NOT system tray icon (Sprint 14+)
- NOT auto-update mechanism (Sprint 14+)
- NOT Windows installer/MSI (Sprint 14+)
```

---

## 7. Phase E — Docs (verification + audit plan)

### 7.1 sprint13_verification.md

Rejouer la checklist fail-fast (§8) et remplir la colonne Observed.
Documenter le commit stack, les metriques delta, la surface nouvelle
livree, les scope cuts respectes.

### 7.2 sprint13_audit_plan.md

Ecrire le plan d'audit pour Sprint 14 Phase 0 avec les tracks :

- Track A : Bridge securite (origin validation, message injection)
- Track B : Open source enforcement (validation contournable ?)
- Track C : Launcher robustesse (daemon deja running, crash recovery)
- Track D : UI glassmorphism accessibilite (contraste, screen reader)
- Track E : Backward compat BrowseEntry v3 (repo_url)
- Track F : Tests et couverture
- Track G : Tech debt T37-T40 reellement fermes

### 7.3 PATTERNS.md update

Marquer T37-T40 CLOSED avec SHA du commit Phase A.
Ajouter les nouveaux patterns :
- P24 : postMessage bridge protocol (type + correlation ID)
- P25 : open source enforcement (public = repo_url required)
- P26 : launcher pattern (spawn daemon + poll running.json)

### 7.4 Commit cible

```
docs(sprint13): verification + audit plan for Sprint 14
```

---

## 8. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | cargo test | `cargo test --workspace --locked` | >= 369 | |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | >= 183 | |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 99+1 | |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 10 | eslint | `npm run lint` | 0 errors | |
| 11 | vitest | `npm run test:unit` | >= 187 | |
| 12 | build | `npm run build` | exit 0 | |
| 13 | size-limit | `npm run size` | 7/7 green | |
| 14 | playwright | `npx playwright test` | >= 30 | |
| 15 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | |
| 16 | SPDX | `bash scripts/check-spdx.sh` | >= 217 | |
| 17 | T37-T40 CLOSED | `grep CLOSED docs/shell/PATTERNS.md \| grep -c 'T3[789]\|T40'` | 4 | |
| 18 | repo_url serde | `cargo test repo_url` | >= 2 green | |
| 19 | bridge tests | `npm run test:unit -- bridge` | >= 5 green | |
| 20 | launcher build | `cargo build -p nexus-launcher` | exit 0 | |
| 21 | public deploy no repo | `uv run pytest -k test_deploy_public_without_repo` | 1 pass | |
| 22 | private deploy no repo | `uv run pytest -k test_deploy_private_without_repo` | 1 pass | |

---

## 9. Git plan

| # | Commit | Phase |
|---|---|---|
| 0 | `docs(sprint13): kickoff + plan detaille` | Planning |
| 1 | `feat(web): Sprint 13 Phase A — UI Netflix glassmorphism + T37-T40` | A |
| 2 | `feat(p2p): Sprint 13 Phase B — open source enforcement for public apps` | B |
| 3 | `feat(bridge): Sprint 13 Phase C — postMessage bridge MVP with task submit + storage` | C |
| 4 | `feat(launcher): Sprint 13 Phase D — minimal Rust launcher with browser open` | D |
| 5 | `docs(sprint13): verification + audit plan for Sprint 14` | E |

---

## 10. Scope cuts

- CPU watchdog iframe → Sprint 14 (D6)
- Branding SBFB (nom, logo, favicon) → Sprint 14
- Runtime templates → Sprint 14
- Re-publish auto → Sprint 14
- Origin separee subdomain → Sprint 14+
- GitHub API verification → Sprint 14 (Sprint 13 trust repo_url)
- Bidirectional push (host → iframe events) → Sprint 14
- Bridge auth → non necessaire (loopback-only)
- System tray icon → Sprint 14+
- Windows installer → Sprint 14+
- Multi-writer iroh-docs → v1.1+

---

## 11. Risks

| # | Risque | Mitigation |
|---|---|---|
| R1 | postMessage origin opaque complique le filtrage | Tracker les iframe refs, ignorer messages sans source matchante |
| R2 | Le launcher ne peut pas trouver le binary daemon | Chercher dans PATH + meme dossier que le launcher + configurable |
| R3 | Les changements UI non commites creent des conflits | Phase A les formalise en premier, avant tout autre changement |
| R4 | `sbfb-bridge.js` charge par une app malveillante | Le bridge est read-only sauf task_submit scope a l'app courante |
| R5 | crate `open` ne marche pas sur tous les OS | Windows 11 = cible primaire, tester `open::that()` en local |

---

## 12. Checkpoint de cloture

Sprint 13 est ferme quand :
1. Checklist 22/22 verte (§8)
2. 6 commits atomiques landed sur master (0 planning + 1-4 phases + 5 docs)
3. `sprint13_verification.md` + `sprint13_audit_plan.md` ecrits
4. PATTERNS.md a jour (T37-T40 CLOSED, P24-P26)
5. Une app dans une iframe peut soumettre une tache via le bridge
6. Un publish public sans `repo_url` est rejete
7. Le launcher ouvre le navigateur et arrete proprement
8. Toutes les pages ont le design glassmorphism
