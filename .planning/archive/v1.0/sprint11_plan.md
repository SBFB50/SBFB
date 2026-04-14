# Sprint 11 — Plan detaille (P2P end-to-end : publish + discovery + render)

**Ecrit** : 2026-04-12, apres kickoff valide.
**Tip master d'entree** : `4d04ac4`
**Decisions Day 0** : D1-D5 gelees (cf. `sprint11_kickoff.md` §3)

---

## 1. Etat verifie a l'entree

| Suite | Count | Status |
|---|---|---|
| Rust workspace | 312 | green |
| Python SDK | 167 | green |
| Python coordinator | 83 + 1 skipped | green |
| Python app-gov | 46 | green |
| Vitest unit | 161 | green |
| Playwright | 27 | green |
| size-limit | 7/7 | green |
| SPDX check | 204/204 | green |

**Donnees de structure existantes a etendre** :
- `BrowseEntry` (`browse.rs:117`) : project_id, project_name, category, description, curator_pubkey, curator_name — pas de champ `source` (curator vs direct)
- `ShellDaemonConfig` (`config.rs:185`) : `logging` + `network` seulement — pas de section `[curator]`
- `CuratorRuntime` (`iroh_runtime.rs`) : gossip topic `nexus-grid/curator/v1`, wire format v1 curator-only
- Coordinator `visibility` (`config.py:70`) : `"private"` par defaut, `"public"` en option — pas de publish action
- Routes web (`App.tsx:39-70`) : 6 routes dont `/browse` (grid read-only) — pas de `/browse/:projectId`

---

## 2. Decisions Day 0 (gelees — rappel synthetique)

- **D1** : apps web = blobs statiques + iframe sandboxee (mode additionnel, TabView reste)
- **D2** : self-publish via gossip + pkarr : coordinator public → daemon broadcast → autres noeuds Browse
- **D3** : default curator FlowUP hardcode dans config.rs
- **D4** : Browse → clic → `/browse/:projectId` → app plein ecran (local seulement Sprint 11, remote Sprint 12+)
- **D5** : VPS EU (`135.181.42.188`) = premier noeud live

**Clarification D4** : "plein ecran" signifie que les apps du projet
sont rendues directement dans la zone de contenu principale (pas
d'accordeon "Invoquer"). Sprint 11 implemente uniquement le rendu
d'apps **locales** (coordinator sur la meme machine). Le rendu
d'apps hebergees sur un noeud distant (cross-node P2P fetch) est
scope-cut Sprint 12+. Le scenario D5 fonctionne parce que le
visiteur de `http://135.181.42.188` utilise le shell web qui
tourne sur le meme VPS que le coordinator.

---

## 3. Research consulte

- iroh 0.97 `presets::N0` publie automatiquement le NodeAddr sur pkarr DHT au boot. Pas besoin de publish pkarr custom pour la decouverte de noeud — l'adresse est deja resolvable par node_id.
- Le gossip topic `nexus-grid/curator/v1` est le canal unique (D3 Sprint 7). Les messages sont JSON `{"v": 1, "curator": "<hex>", "ticket": "<blob_ticket>"}`. On ajoute un deuxieme type de message `{"v": 1, "type": "project", ...}` sur le meme topic.
- `BrowseAggregator` (`browse.rs`) aplatit les curator lists en `BrowseEntry`. Il faut ajouter une source "direct" pour les projets annonces via gossip (pas via curator list).
- Coordinator `visibility` est deja un champ config (`"public"` / `"private"`). Le `running.json` du coordinator l'expose deja via `registry.py`. Il manque le trigger publish.
- Le daemon HTTP (`http.rs`) a 6 endpoints. On ajoute `POST /publish` qui accept une `ProjectAnnouncement`.
- Le coordinator daemon proxy (`api/daemon.py`) forwarde 5 routes. On ajoute `POST /daemon/publish`.
- Les routes web sont lazy-loaded (Sprint 9 Phase A). On ajoute `/browse/:projectId` avec le meme pattern.

---

## 4. Phase A — Self-publish coordinator → gossip discovery

### 4.1 Daemon core : module `publish.rs`

**Nouveau fichier** `crates/nexus-shell-daemon-core/src/publish.rs` (~150 LOC) :

```rust
/// Gossip-based project announcement.
///
/// When a coordinator with visibility=public starts, it sends
/// a ProjectAnnouncement via gossip on the curator topic.
/// Other daemons receive it and add the project to the browse
/// aggregator as a "direct" entry (no curator intermediary).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnnouncement {
    /// Wire format version. Always 1 for Sprint 11.
    pub v: u32,
    /// Message type discriminator (vs curator announcements).
    #[serde(rename = "type")]
    pub msg_type: String,  // "project"
    /// Hex node_id of the announcing daemon.
    pub node_id: String,
    /// Project metadata.
    pub project_name: String,
    pub category: String,
    pub description: String,
    /// List of app names available on this project.
    pub apps: Vec<String>,
}
```

- `ProjectAnnouncement::to_gossip_bytes()` → serde_json serialize
- `ProjectAnnouncement::from_gossip_bytes()` → parse + validate v=1, type="project"
- `publish_project(gossip: &GossipClient, topic: [u8;32], ann: &ProjectAnnouncement)` → broadcast

### 4.2 Daemon core : extension `iroh_runtime.rs`

- `CuratorRuntime::process_announcement_bytes()` actuellement rejette tout message sans champ `"curator"`. Ajouter un branchement : si `"type": "project"` → deleguer a un nouveau handler `process_project_announcement()`.
- `process_project_announcement()` : valide le `ProjectAnnouncement`, passe le `BrowseEntry` resultant au `BrowseAggregator`.

### 4.3 Daemon core : extension `browse.rs`

- Ajouter un champ `source: BrowseSource` a `BrowseEntry` :
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub enum BrowseSource {
      Curator,
      Direct,
  }
  ```
- `BrowseAggregator::add_direct_entry(entry: BrowseEntry)` : ajoute un projet annonce directement (pas via curator list). Dedup par `project_id`.
- `BrowseAggregator::list()` : retourne curator + direct entries (deja triees par name).
- `BrowseEntry` pour un direct entry : `curator_pubkey = ""`, `curator_name = "Self-published"`, `source = Direct`.

### 4.4 Daemon HTTP : `POST /publish`

**Fichier** `crates/nexus-shell-daemon/src/http.rs` — ajouter :

```rust
async fn publish_project(
    State(state): State<DaemonHttpState>,
    Json(payload): Json<PublishRequest>,
) -> impl IntoResponse { ... }

#[derive(Deserialize)]
struct PublishRequest {
    project_name: String,
    category: String,
    description: String,
    apps: Vec<String>,
}
```

- Construit un `ProjectAnnouncement` avec le `node_id` du daemon
- Appelle `publish::publish_project()` pour broadcast gossip
- Appelle `BrowseAggregator::add_direct_entry()` pour l'ajouter localement aussi
- Retourne 200 `{"published": true}`

### 4.5 Coordinator : `POST /project/publish` + auto-publish

**Fichier** `packages/nexus-coordinator/src/nexus_coordinator/api/health.py` — ajouter endpoint :

```python
@router.post("/project/publish")
async def publish_project(coord: CoordinatorDep) -> dict:
    """Publish this project to the P2P network via daemon gossip."""
    ...
```

- Lit la config du coordinator (project_name, visibility, description)
- Collecte la liste des apps installees (coord.app_registry)
- Forwarde au daemon `POST /publish` via le proxy HTTP
- Retourne `{"published": true}`

**Auto-publish au demarrage** : dans `Coordinator.start()` (step 11, apres app discovery) :
- Si `self.config.network.visibility == "public"` ET daemon reachable :
  - Appeler `POST /project/publish` sur soi-meme
  - Log INFO "Published project '{name}' to P2P network"
- Si daemon unreachable : log WARNING, ne bloque pas le boot

### 4.6 Tests Phase A

**Rust** (~8 nouveaux tests) :
- `publish::test_announcement_roundtrip` — serde JSON round-trip
- `publish::test_announcement_rejects_wrong_version` — v != 1 rejete
- `publish::test_announcement_rejects_wrong_type` — type != "project" rejete
- `browse::test_add_direct_entry` — BrowseAggregator accepte une entree directe
- `browse::test_direct_and_curator_entries_coexist` — les deux sources apparaissent dans list()
- `browse::test_direct_entry_dedup_by_project_id` — pas de doublons
- `iroh_runtime::test_process_project_announcement` — message gossip → BrowseEntry
- `http::test_publish_endpoint` — integration HTTP POST /publish

**Python** (~4 nouveaux tests) :
- `test_publish_project_endpoint` — POST /project/publish retourne 200
- `test_publish_project_requires_daemon` — daemon down → 503
- `test_auto_publish_on_start_public` — coordinator public → publish called
- `test_auto_publish_on_start_private` — coordinator private → publish NOT called

### 4.7 Critere d'acceptation Phase A

- `cargo test --workspace --locked` passe avec ~320 tests (+8)
- `uv run pytest packages/nexus-coordinator/tests/ -q` passe avec ~87 (+4)
- Un coordinator avec `visibility=public` s'annonce automatiquement au boot
- Le daemon HTTP repond a `POST /publish` avec 200
- Le `GET /browse` inclut les projets self-published (source: "direct")
- `verify.sh --quick` exit 0

### 4.8 Commit cible

```
feat(p2p): Sprint 11 Phase A — self-publish coordinator projects via gossip

- crates/nexus-shell-daemon-core/src/publish.rs: ProjectAnnouncement struct
  + gossip broadcast + from/to bytes
- browse.rs: BrowseSource enum (Curator|Direct) + add_direct_entry()
- iroh_runtime.rs: process_project_announcement() handler (v1 type=project)
- http.rs: POST /publish endpoint
- coordinator api/health.py: POST /project/publish + auto-publish on start
  if visibility=public
- 8 new Rust tests + 4 new coordinator tests

Test delta:
  Rust: 312 → ~320 (+8)
  coord: 83+1 → ~87+1 (+4)
Scope cuts honoured: no blob upload, no iframe, no remote fetch
```

---

## 5. Phase B — Default curator + auto-subscription

### 5.1 Daemon core : extension `config.rs`

Ajouter une section `[curator]` a `ShellDaemonConfig` :

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuratorConfig {
    /// Ed25519 public keys (hex) to auto-subscribe at first boot.
    /// Empty by default. VPS deployments set this to FlowUP's pubkey.
    #[serde(default)]
    pub default_curators: Vec<String>,
}

pub struct ShellDaemonConfig {
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    #[serde(default)]
    pub curator: CuratorConfig,
}
```

### 5.2 Daemon boot : auto-subscribe

**Fichier** `crates/nexus-shell-daemon/src/runtime.rs` — dans `DaemonRuntime::start()` :

Apres le chargement de `CuratorRuntime` :
1. Lire `config.curator.default_curators`
2. Pour chaque pubkey dans la liste : si PAS deja dans `subscriptions.json` → appeler `CuratorRuntime::subscribe(pubkey)`
3. Log INFO "Auto-subscribed to N default curator(s)"
4. Ne fait rien si `subscriptions.json` contient deja ces pubkeys (idempotent)

### 5.3 Deploy : `create-curator-list.sh`

**Nouveau fichier** `deploy/create-curator-list.sh` (~60 LOC) :

Script bash qui :
1. Genere (ou lit) une keypair Ed25519 curator sur le VPS
2. Cree un JSON curator list avec les projets officiels
3. Signe la liste avec la cle privee
4. Publie le blob via l'API du daemon (`POST /publish` ou upload blob direct)
5. Broadcast l'annonce gossip

Pour Sprint 11, ce script est un helper manuel pour le VPS EU.
L'automatisation systemd viendra Sprint 12+.

### 5.4 Coordinator : endpoint `GET /daemon/default-curators`

**Fichier** `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py` — ajouter :

```python
@router.get("/daemon/default-curators")
async def default_curators(coord: CoordinatorDep) -> dict:
    """Return the daemon's configured default curator pubkeys."""
    ...
```

Forwarde au daemon un nouveau endpoint `GET /default-curators` qui retourne
`config.curator.default_curators`. Utile pour le shell web qui affiche
"Curators par defaut" dans la page Curators.

### 5.5 Tests Phase B

**Rust** (~5 nouveaux tests) :
- `config::test_parse_curator_section` — TOML avec `[curator]` parse OK
- `config::test_default_curator_empty` — section absente → default vide
- `config::test_auto_subscribe_at_boot` — mock CuratorRuntime, verifie subscribe appele
- `config::test_auto_subscribe_idempotent` — deja abonne → pas de double subscribe
- `http::test_default_curators_endpoint` — GET /default-curators retourne la liste

**Python** (~2 nouveaux tests) :
- `test_default_curators_proxy` — GET /daemon/default-curators forwarde
- `test_default_curators_daemon_down` — daemon down → 503

### 5.6 Critere d'acceptation Phase B

- `cargo test --workspace --locked` passe avec ~325 tests (+5)
- `uv run pytest packages/nexus-coordinator/tests/ -q` passe avec ~89 (+2)
- Un daemon lance avec `[curator] default_curators = ["<hex>"]` auto-subscribe au boot
- `GET /default-curators` retourne la liste configuree
- `verify.sh --quick` exit 0

### 5.7 Commit cible

```
feat(p2p): Sprint 11 Phase B — default FlowUP curator + auto-subscription

- config.rs: [curator] section with default_curators Vec<String>
- runtime.rs: auto-subscribe to default curators at boot (idempotent)
- http.rs: GET /default-curators endpoint
- coordinator api/daemon.py: GET /daemon/default-curators proxy
- deploy/create-curator-list.sh: curator list creation script for VPS
- 5 new Rust tests + 2 new coordinator tests

Test delta:
  Rust: ~320 → ~325 (+5)
  coord: ~87+1 → ~89+1 (+2)
Scope cuts honoured: no auto-broadcast systemd, manual VPS script
```

---

## 6. Phase C — Browse → app plein ecran UX

### 6.1 Route `/browse/:projectId`

**Fichier** `web/src/App.tsx` — ajouter entre `/browse` et `/curators` :

```tsx
{
  path: "/browse/:projectId",
  lazy: () => import("@/pages/BrowsedProject"),
},
```

### 6.2 Browse page : cards cliquables

**Fichier** `web/src/pages/Browse.tsx` — modifier `BrowseCard` :

- Wrapper le card dans un `<Link to={/browse/${entry.project_id}}>` ou utiliser `useNavigate` + onClick
- Ajouter un curseur pointer + hover effect (deja dans shadcn Card)
- Le `project_id` est le node_id hex du coordinator (champ existant de `BrowseEntry`)

### 6.3 Nouvelle page `BrowsedProject.tsx`

**Nouveau fichier** `web/src/pages/BrowsedProject.tsx` (~200 LOC) :

Layout :
```
+----------------------------------------------+
| ← Retour Browse     Nom du projet            |
+----------------------------------------------+
| Sidebar (250px)    | Zone principale          |
| - Nom projet       | - Onglets apps           |
| - Categorie        |   [App1] [App2] [App3]   |
| - Description       | - Contenu de l'onglet   |
| - Curator source   |   selectionnee           |
| - Status badge     |   (TabView rendu plein   |
| - Node ID          |    ecran)                |
+----------------------------------------------+
```

Logique :
1. Extraire `projectId` du param URL
2. Determiner si le projet est local (meme coordinator que `activeCoordinatorUrl`) :
   - Si OUI : fetch le manifest via `getAppManifest()` → lister les apps → rendre TabView
   - Si NON : afficher un message "Projet heberge sur un noeud distant. Connectez-vous directement." (Sprint 12+ pour P2P cross-node fetch)
3. Pour les apps locales : premier onglet selectionne par defaut, rendu via `TabViewRenderer`
4. Pour les apps web blobs (Sprint 12+) : placeholder "App web — disponible prochainement"

### 6.4 Composant `WebAppFrame.tsx` (skeleton)

**Nouveau fichier** `web/src/components/app/WebAppFrame.tsx` (~40 LOC) :

Composant iframe sandboxee pour le rendu futur des apps web statiques (D1).
Sprint 11 livre le composant avec un placeholder "Chargement de l'application..."
car le backend blob fetch n'est pas encore implemente.

```tsx
interface WebAppFrameProps {
  blobUrl?: string;
}

export function WebAppFrame({ blobUrl }: WebAppFrameProps) {
  if (!blobUrl) {
    return <div className="...">Application web non disponible</div>;
  }
  return (
    <iframe
      src={blobUrl}
      sandbox="allow-scripts allow-same-origin"
      className="w-full h-full border-0"
      title="Application web"
    />
  );
}
```

### 6.5 API : extensions daemon.ts

**Fichier** `web/src/api/daemon.ts` — ajouter :

- `BrowseEntry` Zod schema : ajouter le champ optionnel `source: z.enum(["Curator", "Direct"]).optional()` (backward compat)
- `listBrowse()` : inchange, le champ `source` arrive du daemon si present

**Fichier** `web/src/api/coordinator.ts` — ajouter :

- `getProjectApps(baseUrl, projectId)` : helper qui determine si le projectId est local, et si oui retourne la liste des apps disponibles via manifest

### 6.6 Tests Phase C

**Vitest** (~12 nouveaux tests) :
- `BrowsedProject.test.tsx` : rendu avec projet local (sidebar + onglets apps)
- `BrowsedProject.test.tsx` : rendu avec projet distant (message "noeud distant")
- `BrowsedProject.test.tsx` : navigation retour vers Browse
- `BrowsedProject.test.tsx` : selection d'onglet app
- `BrowsedProject.test.tsx` : projet sans apps → message vide
- `WebAppFrame.test.tsx` : rendu sans blobUrl → placeholder
- `WebAppFrame.test.tsx` : rendu avec blobUrl → iframe sandbox attrs
- `Browse.test.tsx` : card cliquable → navigation
- `Browse.test.tsx` : card affiche source badge (Curator/Direct)
- `daemon.test.ts` : BrowseEntry schema accepte source optionnel
- `daemon.test.ts` : BrowseEntry schema accepte sans source (compat)
- `coordinator.test.ts` : getProjectApps pour projet local

**Playwright** (~3 nouveaux specs) :
- `browse-click-project.spec.ts` : Browse → clic card → /browse/:id navigue
- `browse-click-project.spec.ts` : /browse/:id affiche sidebar + nom projet
- `browse-click-project.spec.ts` : /browse/:id → retour Browse fonctionne

### 6.7 Critere d'acceptation Phase C

- `npm run test:unit` passe avec ~173 tests (+12)
- `npx playwright test` passe avec ~30 tests (+3)
- Cliquer une card Browse navigue vers `/browse/:projectId`
- La page affiche sidebar info + TabView rendu si projet local
- Le composant `WebAppFrame` existe (skeleton)
- `verify.sh --quick` exit 0
- `npm run size` : 7/7 green (nouveau chunk `BrowsedProject` sous lazy)

### 6.8 Commit cible

```
feat(web): Sprint 11 Phase C — Browse full-screen app rendering

- App.tsx: new route /browse/:projectId (lazy-loaded)
- pages/BrowsedProject.tsx: sidebar + TabView full-screen for local projects,
  "remote node" message for cross-node (Sprint 12+)
- components/app/WebAppFrame.tsx: iframe sandbox skeleton for web app blobs
- pages/Browse.tsx: BrowseCard now clickable → navigate to /browse/:id
- api/daemon.ts: BrowseEntry.source optional field support
- api/coordinator.ts: getProjectApps() helper
- 12 new Vitest + 3 new Playwright specs

Test delta:
  Vitest: 161 → ~173 (+12)
  Playwright: 27 → ~30 (+3)
Scope cuts honoured: no blob fetch, no cross-node rendering, no iframe live content
```

---

## 7. Phase D — Deploy VPS EU live

### 7.1 Web build pour le VPS

**Fichier** `deploy/deploy-web.sh` (~40 LOC) :

```bash
#!/usr/bin/env bash
# Build the web shell and upload to VPS nginx root.
set -euo pipefail
HOST="${1:?Usage: deploy-web.sh <host> <ssh-key>}"
KEY="${2:?}"
cd "$(dirname "$0")/../web"
npm ci && npm run build
scp -i "$KEY" -r dist/* "nexus@$HOST:/opt/nexus-grid/web/"
ssh -i "$KEY" "nexus@$HOST" "sudo systemctl reload nginx"
```

### 7.2 Configuration nginx

**Nouveau fichier** `deploy/nginx-nexus.conf` (~30 LOC) :

```nginx
server {
    listen 80;
    server_name _;
    root /opt/nexus-grid/web;
    index index.html;

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Coordinator API proxy (loopback)
    location /api/ {
        proxy_pass http://127.0.0.1:8000/;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### 7.3 Configuration coordinator VPS

**Nouveau fichier** `deploy/coordinator.toml.example` :

```toml
[identity]
project_name = "gov-officiel"

[network]
visibility = "public"
api_port = 8000

[apps]
# nexus-app-gov loaded from packages/
```

### 7.4 Configuration daemon VPS

**Nouveau fichier** `deploy/config.toml.example` :

```toml
[logging]
level = "info"

[network]
api_host = "127.0.0.1"
api_port = 7000

[curator]
default_curators = []
# Populated after first curator list creation
```

### 7.5 Update `deploy/provision.sh`

Ajouter a la fin du script :
- `apt install -y nginx`
- Copier `nginx-nexus.conf` vers `/etc/nginx/sites-available/nexus`
- `ln -sf ... sites-enabled/nexus && rm -f sites-enabled/default`
- `mkdir -p /opt/nexus-grid/web`
- `chown nexus:nexus /opt/nexus-grid/web`

### 7.6 Update `deploy/deploy.sh`

Ajouter un role `--role web` qui :
- Upload le build web via SCP
- Reload nginx
- Smoke test : `curl -s http://localhost/ | grep -q "nexus"`

### 7.7 Deploiement interactif (hors commit, session live)

L'utilisateur fournit le SSH au VPS EU. Sequence :
1. Build local : `cd web && npm run build`
2. Upload web : `deploy/deploy-web.sh 135.181.42.188 ~/.ssh/vps_eu`
3. Init coordinator : `ssh nexus@... "cd /opt/nexus-grid/data && nexus-coordinator init gov-officiel --public"`
4. Restart services : `sudo systemctl restart nexus-daemon nexus-coordinator`
5. Creer curator list : `deploy/create-curator-list.sh 135.181.42.188`
6. Smoke test : `curl http://135.181.42.188/` → HTML du shell web

### 7.8 Critere d'acceptation Phase D

- Fichiers de config dans `deploy/` (nginx, coordinator.toml, config.toml)
- `deploy/deploy-web.sh` script fonctionnel
- `deploy/provision.sh` mis a jour avec nginx
- `verify.sh --quick` exit 0
- (Si VPS dispo) : `http://135.181.42.188/` affiche le shell web, Browse montre le projet gov

### 7.9 Commit cible

```
feat(deploy): Sprint 11 Phase D — VPS EU live with coordinator + shell web

- deploy/nginx-nexus.conf: SPA routing + API proxy to coordinator
- deploy/deploy-web.sh: build + upload web shell to VPS nginx
- deploy/coordinator.toml.example + config.toml.example: VPS config templates
- deploy/provision.sh: nginx install + site config
- deploy/deploy.sh: new --role web for shell deployment
- deploy/create-curator-list.sh: manual curator list creation for VPS

Test delta: unchanged (deploy scripts are infrastructure)
Scope cuts honoured: no US/Asia VPS, no custom domain, no HTTPS
```

---

## 8. Phase E — Verification + audit plan

### 8.1 Livrables

- `.planning/sprint11_verification.md` — checklist fail-fast remplie
- `.planning/sprint11_audit_plan.md` — plan pour Sprint 12 Phase 0

### 8.2 Mises a jour

- `docs/claude/README.md` §10 : ajouter Sprint 11 dans la table cross-ref
- `docs/shell/PATTERNS.md` : nouveaux patterns si applicable (P-self-publish, P-default-curator)
- Memory `nexus_grid_pivot.md` : update tip, compteurs, Sprint 11 summary + Sprint 12 outline

### 8.3 Commit cible

```
docs(sprint11): verification + audit plan for Sprint 12

- .planning/sprint11_verification.md: N/N fail-fast checklist
- .planning/sprint11_audit_plan.md: tracks for Sprint 12 Phase 0
- docs/claude/README.md §10: Sprint 11 added to cross-reference table
- Memory nexus_grid_pivot.md updated
```

---

## 9. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | cargo test | `cargo test --workspace --locked` | >= 325 passed | |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 167 passed | |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 89 passed + 1 skipped | |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | |
| 9 | tsc | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 10 | eslint | `npm run lint` | 0 errors | |
| 11 | vitest | `npm run test:unit` | >= 173 passed | |
| 12 | coverage | `npm run test:coverage` | lines >= 85%, branches >= 78% | |
| 13 | build | `npm run build` | exit 0 | |
| 14 | size-limit | `npm run size` | 7/7 green (+ BrowsedProject chunk) | |
| 15 | playwright | `npx playwright test` | >= 30 passed | |
| 16 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | |
| 17 | npm audit | `npm audit --audit-level=high` | 0 high/crit | |
| 18 | SPDX check | `bash scripts/check-spdx.sh` | exit 0 | |
| 19 | POST /publish | `curl -X POST localhost:PORT/publish -d '...'` | 200 {"published": true} | |
| 20 | GET /browse direct | `curl localhost:PORT/browse` | entries with source Direct | |
| 21 | default-curators | `curl localhost:PORT/default-curators` | 200 with array | |
| 22 | browse route | naviguer `/browse/<id>` dans le browser | page BrowsedProject affichee | |
| 23 | browse card clic | cliquer une card dans /browse | navigation vers /browse/:id | |
| 24 | deploy scripts | `ls deploy/nginx-nexus.conf deploy/deploy-web.sh` | existent | |
| 25 | config examples | `ls deploy/coordinator.toml.example deploy/config.toml.example` | existent | |
| 26 | SPDX nouveaux | `bash scripts/check-spdx.sh` | couvre les nouveaux .rs/.ts/.py | |
| 27 | verify.sh full | `./scripts/verify.sh` | exit 0 | |

---

## 10. Git plan

| # | Phase | Commit |
|---|---|---|
| 0 | Planning | `docs(sprint11): kickoff + plan detaille with D1-D5` |
| 1 | A | `feat(p2p): Sprint 11 Phase A — self-publish coordinator projects via gossip` |
| 2 | B | `feat(p2p): Sprint 11 Phase B — default FlowUP curator + auto-subscription` |
| 3 | C | `feat(web): Sprint 11 Phase C — Browse full-screen app rendering` |
| 4 | D | `feat(deploy): Sprint 11 Phase D — VPS EU live with coordinator + shell web` |
| 5 | E | `docs(sprint11): verification + audit plan for Sprint 12` |

---

## 11. Scope cuts (copie kickoff §5)

- Pas de upload blob via UI — Sprint 12+
- Pas de branding SBFB — Sprint 12+
- Pas de 2 VPS supplementaires (US/Asia) — Sprint 12+
- Pas de multi-writer iroh-docs — Sprint 12+
- Pas de monetisation / tokens — hors scope
- Pas de sandboxing CSP avance — basic sandbox attrs Sprint 11
- Pas de custom domain / DNS — acces par IP
- Pas de cross-node app rendering — local seulement Sprint 11

---

## 12. Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | Gossip message size limit | ProjectAnnouncement trop gros → rejete | Garder < 1 KB (pas de blobs dans le message) |
| R2 | BrowseSource breaking change serde | Daemon v1 vs v2 incompatibles | Champ `source` optionnel (#[serde(default)]) |
| R3 | CuratorRuntime refuse les messages non-curator | ProjectAnnouncement drop silencieux | Brancher AVANT le check "curator" dans process_announcement_bytes |
| R4 | nginx proxy CORS | Shell web bloque par CORS vers coordinator | proxy_pass sur loopback, pas de CORS cross-origin |
| R5 | VPS EU pas provisionnee avec nginx | Phase D partielle | Scripts commites, deploiement en session suivante |
| R6 | Lazy chunk BrowsedProject casse size-limit | Build echec | Budget genereux (< 50 KB pour la page) |
| R7 | scan-en-strings detecte des strings anglais dans BrowsedProject | Guard echec | Ecrire les strings FR des le depart |

---

## 13. Checkpoint de cloture

Le sprint est ferme quand :
1. 27/27 fail-fast checklist green
2. 6 commits atomiques landed sur master (planning + 4 phases + docs)
3. `sprint11_verification.md` + `sprint11_audit_plan.md` ecrits
4. PATTERNS.md a jour (nouveaux patterns P2P publish + default curator)
5. Memory `nexus_grid_pivot.md` mise a jour
6. (Optionnel) VPS EU live avec shell web + Browse + app gov visible
