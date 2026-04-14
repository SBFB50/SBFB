# Sprint 12 — Plan detaille (Rendu universel : archive zip + daemon blob-serve + iframe isolee)

**Ecrit** : 2026-04-13, apres kickoff valide.
**Tip master d'entree** : `31479fa`
**Decisions Day 0** : D1-D7 gelees (cf. `sprint12_kickoff.md` §3)

---

## 1. Etat verifie a l'entree

| Suite | Count | Status |
|---|---|---|
| Rust workspace | 331 | green |
| Python SDK | 167 | green |
| Python coordinator | 89 + 1 skipped | green |
| Python app-gov | 46 | green |
| Vitest unit | 173 | green |
| Playwright | 30 | green |
| size-limit | 7/7 | green |
| SPDX check | 209/209 | green |

**Infrastructure existante a etendre** :
- `BlobsClient` (`blobs.rs`) : `add_bytes()`, `get_bytes()`, `fetch_ticket()` — toutes les primitives existent
- `BlobTicket::new(addr, hash, BlobFormat::Raw)` pour minter un ticket
- `DiscoveryClient::my_endpoint_addr()` pour l'adresse du daemon
- `ProjectAnnouncement` (`publish.rs`) : v=1, pas de `archive_ticket`
- `BrowseEntry` (`browse.rs`) : pas de `archive_ticket`
- `POST /publish` (`http.rs`) : accepte metadata, pas de zip
- Daemon HTTP sur port 7000, coordinator sur port 8000
- `WebAppFrame.tsx` : iframe sandbox fonctionnel (35 LOC)
- CAS `GET /app/{name}/files/{sha256}` : sert bytes bruts
- TabView SDK : 12 block kinds, `model_dump()` retourne dict

---

## 2. Decisions Day 0 (gelees — rappel synthetique)

- **D1** : zip = format universel de publication
- **D2** : daemon sert les blobs directement (origin separee, port 7000)
- **D3** : `sandbox="allow-scripts"` + CSP `connect-src 'none'` (untrusted)
- **D4** : ProjectAnnouncement v2 avec `archive_ticket: Option<String>`
- **D5** : TabView pre-rendu en HTML par le coordinator (~460 LOC)
- **D6** : banniere "contenu tiers" dans le shell
- **D7** : tech debt T28-T36

---

## 3. Research consulte

- **Securite iframe** : `sandbox="allow-scripts"` sans `allow-same-origin` donne une origin opaque. Le SW ne peut PAS intercepter les requetes (spec W3C). Mais le daemon sert directement sur un port different → les imports relatifs marchent sans SW. CSP `connect-src 'none'` injectee par le daemon bloque toutes les requetes sortantes.
- **fflate** : lib JS pour zip en browser. NON UTILISEE — le daemon Rust decompresse cote serveur avec la crate `zip` (plus rapide, plus sur, pas de JS).
- **Crate `zip` Rust** : v2.6+, mature, read-only safe. `ZipArchive::new(reader)` → `by_name(path)` → `read_to_end()`. Pas de path traversal si on valide les chemins.
- **TabView pre-render** : 12 block kinds, tous portables en HTML pur. Charts SVG = math de coordonnees identique au React (deja inline SVG, pas de lib chart). CSS fixe ~140 lignes (tokens dark theme de `index.css`). Blocks interactifs (button, file_upload) → `<form>` ou placeholder.
- **Pyodide 0.29.3** : fonctionne dans iframe `allow-scripts` sans `allow-same-origin`. Pas besoin de SharedArrayBuffer pour le mode de base. MAIS avec CSP `connect-src 'none'`, Pyodide ne peut pas charger depuis CDN → doit etre bundle dans le zip (~40MB).

---

## 4. Phase A — Daemon blob-serve endpoint + zip decompression

### 4.1 Nouvelle dep Rust : crate `zip`

**Fichier** `crates/nexus-shell-daemon/Cargo.toml` :
```toml
[dependencies]
zip = { version = "2.6", default-features = false, features = ["deflate"] }
```

### 4.2 Module `blob_serve.rs`

**Nouveau fichier** `crates/nexus-shell-daemon-core/src/blob_serve.rs` (~120 LOC) :

```rust
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

/// Cache LRU des archives decompressees.
/// Cle = hash hex du blob, valeur = map path → bytes.
pub struct BlobServeCache {
    entries: DashMap<String, Arc<HashMap<String, Vec<u8>>>>,
    max_entries: usize,
}

impl BlobServeCache {
    pub fn new(max_entries: usize) -> Self { ... }

    /// Decompresse un zip et cache le resultat.
    /// Retourne Err si le zip est invalide ou si la taille
    /// decompresse depasse `max_decompressed_bytes` (100MB).
    pub fn load(&self, hash: &str, zip_bytes: &[u8],
                max_decompressed_bytes: usize) -> Result<()> { ... }

    /// Retourne les bytes d'un fichier dans l'archive.
    pub fn get_file(&self, hash: &str, path: &str)
        -> Option<Arc<Vec<u8>>> { ... }

    /// Evicte les entrees les plus anciennes si > max_entries.
    fn evict_if_needed(&self) { ... }
}

/// Detecte le Content-Type depuis l'extension + magic bytes.
pub fn detect_content_type(filename: &str, data: &[u8]) -> &'static str { ... }

/// Valide qu'un path extrait du zip ne fait pas de path traversal.
pub fn validate_zip_path(path: &str) -> bool {
    !path.contains("..") && !path.starts_with('/')
}
```

### 4.3 Daemon HTTP : endpoint `GET /blob-serve/{hash}/{path}`

**Fichier** `crates/nexus-shell-daemon/src/http.rs` — ajouter :

```rust
async fn blob_serve(
    State(state): State<DaemonHttpState>,
    Path((hash, path)): Path<(String, String)>,
) -> impl IntoResponse {
    // 1. Valider le path (pas de traversal)
    if !blob_serve::validate_zip_path(&path) {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }

    // 2. Charger le zip dans le cache si pas deja present
    if !state.blob_serve_cache.has(&hash) {
        // Essayer de lire depuis le blob store local
        let blob_bytes = match state.blobs_client.get_bytes_hex(&hash).await {
            Ok(b) => b,
            Err(_) => {
                // Essayer de fetcher via ticket depuis BrowseEntry
                match state.browse_aggregator.get_archive_ticket(&hash) {
                    Some(ticket) => {
                        state.blobs_client
                            .fetch_ticket(&state.endpoint, &state.memory_lookup, &ticket)
                            .await?;
                        state.blobs_client.get_bytes_hex(&hash).await?
                    }
                    None => return (StatusCode::NOT_FOUND, "blob not found").into_response(),
                }
            }
        };
        state.blob_serve_cache
            .load(&hash, &blob_bytes, 100 * 1024 * 1024)?;  // 100MB max
    }

    // 3. Servir le fichier demande
    let file_bytes = match state.blob_serve_cache.get_file(&hash, &path) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "file not found in archive").into_response(),
    };

    let content_type = blob_serve::detect_content_type(&path, &file_bytes);

    // 4. Headers de securite
    let headers = [
        ("Content-Type", content_type),
        ("Content-Security-Policy", "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none'; frame-ancestors *"),
        ("X-Content-Type-Options", "nosniff"),
        ("Cache-Control", "public, max-age=3600, immutable"),
    ];

    (StatusCode::OK, headers, file_bytes.to_vec()).into_response()
}
```

Route : `.route("/blob-serve/:hash/*path", get(blob_serve))`

Default index : si `path` est vide ou `/`, servir `index.html`.

### 4.4 ProjectAnnouncement v2

**Fichier** `publish.rs` — etendre :

```rust
pub struct ProjectAnnouncement {
    pub v: u32,                          // 1 ou 2
    #[serde(rename = "type")]
    pub msg_type: String,
    pub node_id: String,
    pub project_name: String,
    pub category: String,
    pub description: String,
    pub apps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_ticket: Option<String>,  // NOUVEAU v2
}
```

- `from_gossip_bytes()` accepte v=1 ET v=2
- v=1 : `archive_ticket = None`
- v=2 : `archive_ticket = Some(ticket)`

### 4.5 BrowseEntry extension

**Fichier** `browse.rs` :
```rust
pub struct BrowseEntry {
    // ... existants ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_ticket: Option<String>,
}
```

### 4.6 Extension `POST /publish`

**Fichier** `http.rs` — etendre `PublishRequest` :

```rust
#[derive(Deserialize)]
struct PublishRequest {
    project_name: String,
    category: String,
    description: String,
    apps: Vec<String>,
    #[serde(default)]
    archive_hash: Option<String>,  // hash hex du zip blob deja stocke
}
```

Si `archive_hash` present :
1. Lire le blob depuis le store
2. `DiscoveryClient::my_endpoint_addr()` → addr
3. `BlobTicket::new(addr, hash, BlobFormat::Raw)` → ticket
4. `archive_ticket = Some(ticket.to_string())`

### 4.7 Tests Phase A

**Rust** (~12 nouveaux tests) :
- `blob_serve::test_load_valid_zip` — zip valide decompresse OK
- `blob_serve::test_load_invalid_zip` — bytes random → Err
- `blob_serve::test_zip_bomb_rejected` — zip > 100MB decompresse → Err
- `blob_serve::test_path_traversal_rejected` — `../etc/passwd` → false
- `blob_serve::test_detect_content_type_html` — `.html` → text/html
- `blob_serve::test_detect_content_type_js` — `.js` → text/javascript
- `blob_serve::test_detect_content_type_magic_png` — magic bytes PNG
- `blob_serve::test_cache_eviction` — > max_entries evicte
- `http::test_blob_serve_returns_file` — GET /blob-serve/{hash}/index.html → 200
- `http::test_blob_serve_not_found` — hash inconnu → 404
- `http::test_blob_serve_csp_headers` — reponse contient CSP connect-src none
- `publish::test_announcement_v2_with_archive_ticket` — serde roundtrip

### 4.8 Critere d'acceptation Phase A

- `cargo test --workspace --locked` >= 343 (+12)
- `GET /blob-serve/{hash}/index.html` retourne le HTML avec CSP
- `GET /blob-serve/{hash}/assets/main.js` retourne le JS
- Path traversal bloque
- Zip bomb bloque (> 100MB)
- ProjectAnnouncement v2 serde OK, v1 backward compat

### 4.9 Commit cible

```
feat(p2p): Sprint 12 Phase A — daemon blob-serve endpoint with zip decompression + CSP isolation

- blob_serve.rs: BlobServeCache (LRU, zip decompression, path validation,
  content-type detection, 100MB decompressed limit)
- http.rs: GET /blob-serve/{hash}/{path} with CSP connect-src 'none'
  + X-Content-Type-Options nosniff + Cache-Control immutable
- publish.rs: ProjectAnnouncement v2 with archive_ticket field
- browse.rs: BrowseEntry gains archive_ticket: Option<String>
- http.rs: POST /publish extended with archive_hash for ticket minting
- dep: zip 2.6 (deflate feature only)
- 12 new Rust tests

Test delta:
  Rust: 331 → ~343 (+12)
Scope cuts honoured: no SW, no subdomain origin, no runtime templates
```

---

## 5. Phase B — Pipeline publish (coordinator zip + TabView pre-render)

### 5.1 Endpoint `POST /project/deploy`

**Nouveau fichier** `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` (~80 LOC) :

```python
@router.post("/project/deploy")
async def deploy_project(
    archive: UploadFile,
    coord: CoordinatorDep,
) -> dict:
    """Upload a zip archive and publish to the P2P network."""
    # 1. Lire le zip upload
    zip_bytes = await archive.read()
    # 2. Valider que c'est un zip valide avec index.html
    _validate_zip(zip_bytes)
    # 3. Stocker comme blob via daemon POST /publish-blob
    hash_hex = await _store_blob(coord, zip_bytes)
    # 4. Publier l'announcement avec archive_hash
    await _publish_with_archive(coord, hash_hex)
    return {"deployed": True, "hash": hash_hex}
```

### 5.2 Endpoint daemon `POST /publish-blob`

**Fichier** `http.rs` — ajouter :

```rust
async fn publish_blob(
    State(state): State<DaemonHttpState>,
    body: Bytes,
) -> impl IntoResponse {
    let hash = state.blobs_client.add_bytes(body.to_vec()).await?;
    let hash_hex = hex::encode(hash);
    Json(json!({"hash": hash_hex}))
}
```

Route : `.route("/publish-blob", post(publish_blob))`

### 5.3 TabView pre-render : `render_tabview_to_html()`

**Nouveau fichier** `packages/nexus-sdk/src/nexus_sdk/html_render.py` (~460 LOC) :

Structure :
```python
# Inline CSS block (~140 lignes, tokens de web/src/index.css)
_INLINE_CSS = """
:root { ... }
body { background: #0a0a0f; color: #e2e4f0; font-family: ... }
.heading-1 { font-size: 1.5rem; font-weight: 600; }
...
"""

def render_tabview_to_html(
    descriptor: dict,
    *,
    title: str = "SBFB App",
) -> str:
    """Render a TabView descriptor dict to self-contained HTML."""
    blocks_html = _render_blocks(descriptor.get("blocks", []))
    return f"""<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{html.escape(title)}</title>
  <style>{_INLINE_CSS}</style>
</head>
<body>{blocks_html}</body>
</html>"""

def _render_blocks(blocks: list[dict]) -> str: ...
def _render_block(block: dict) -> str: ...  # dispatch par kind
def _render_heading(block: dict) -> str: ...
def _render_text(block: dict) -> str: ...
def _render_kv(block: dict) -> str: ...
def _render_metric(block: dict) -> str: ...
def _render_table(block: dict) -> str: ...
def _render_badge_list(block: dict) -> str: ...
def _render_button(block: dict) -> str: ...
def _render_chart_line(block: dict) -> str: ...  # SVG inline
def _render_chart_bar(block: dict) -> str: ...   # SVG inline
def _render_empty(block: dict) -> str: ...
def _render_section(block: dict) -> str: ...     # recursif
def _render_file_upload(block: dict) -> str: ...  # placeholder
```

### 5.4 Auto-deploy TabView apps au publish

**Fichier** `coordinator.py` — etendre `_auto_publish()` :

```python
async def _auto_publish(self):
    # ... existant : metadata publish ...

    # NOUVEAU : pre-rendre les TabView et deployer comme zip
    import zipfile, io
    from nexus_sdk.html_render import render_tabview_to_html

    zip_buf = io.BytesIO()
    with zipfile.ZipFile(zip_buf, 'w', zipfile.ZIP_DEFLATED) as zf:
        for app_name, app in self.apps.items():
            for tab in app.manifest.tabs:
                descriptor = await _maybe_call(tab.fn, app)
                if hasattr(descriptor, "model_dump"):
                    html = render_tabview_to_html(
                        descriptor.model_dump(mode="json"),
                        title=f"{app_name} — {tab.name}",
                    )
                    zf.writestr(f"{app_name}/{tab.name}.html", html)
            # index.html pour l'app = premier tab
            if app.manifest.tabs:
                first_tab = app.manifest.tabs[0].name
                zf.writestr(f"{app_name}/index.html",
                    f'<meta http-equiv="refresh" content="0;url={first_tab}.html">')
        # index.html racine = premier app
        if self.apps:
            first_app = next(iter(self.apps))
            zf.writestr("index.html",
                f'<meta http-equiv="refresh" content="0;url={first_app}/index.html">')

    zip_bytes = zip_buf.getvalue()
    # Stocker et publier
    hash_hex = await _store_blob(self, zip_bytes)
    await _publish_with_archive(self, hash_hex)
```

### 5.5 Ajouter `text/html` aux MIME autorises

**Fichier** `packages/nexus-sdk/src/nexus_sdk/files.py` :

```python
ALLOWED_MAGIC_BYTES["text/html"] = _check_html_magic

def _check_html_magic(data: bytes) -> bool:
    header = data[:50].lower().lstrip()
    return header.startswith(b"<!doctype") or header.startswith(b"<html")
```

### 5.6 Tests Phase B

**Python** (~12 nouveaux tests) :
- `test_render_heading` — h2/h3/h4 selon level
- `test_render_text_muted` — texte avec classe muted
- `test_render_kv_items` — items label/value
- `test_render_metric_tones` — 4 tones avec couleurs
- `test_render_table` — colonnes + rows + empty_text
- `test_render_chart_line_svg` — SVG genere avec path + circles
- `test_render_chart_bar_svg` — SVG genere avec rects
- `test_render_section_recursive` — section contenant des blocks
- `test_render_full_tabview` — descriptor complet → HTML valide
- `test_deploy_endpoint_valid_zip` — POST /project/deploy → 200
- `test_deploy_endpoint_invalid_zip` — pas un zip → 400
- `test_auto_deploy_tabview_generates_zip` — publish genere un zip avec HTML

### 5.7 Critere d'acceptation Phase B

- `uv run pytest packages/nexus-sdk/tests/ -q` >= 179 (+12)
- `uv run pytest packages/nexus-coordinator/tests/ -q` >= 92 (+3)
- `POST /project/deploy` accepte un zip, retourne hash
- Le coordinator genere un zip HTML depuis les TabView au publish
- Le HTML genere contient les bons SVG charts, tables, metrics
- `text/html` accepte par le CAS

### 5.8 Commit cible

```
feat(p2p): Sprint 12 Phase B — publish pipeline with TabView pre-render + universal zip deploy

- nexus_sdk/html_render.py: render_tabview_to_html() ~460 LOC Python,
  12 block kinds, inline CSS dark theme, SVG charts
- coordinator api/deploy.py: POST /project/deploy (zip upload + blob store)
- coordinator.py: auto-deploy pre-rendered TabView HTML at publish time
- daemon http.rs: POST /publish-blob (store raw bytes as blob)
- nexus_sdk/files.py: text/html added to ALLOWED_MAGIC_BYTES
- 12 new SDK tests + 3 new coordinator tests

Test delta:
  SDK: 167 → ~179 (+12)
  coord: 89+1 → ~92+1 (+3)
Scope cuts honoured: no runtime templates, no CLI sbfb publish
```

---

## 6. Phase C — Frontend cross-node rendering

### 6.1 API : extension `daemon.ts`

Ajouter `archive_ticket` au `BrowseEntrySchema` :
```typescript
archive_ticket: z.string().optional(),
```

Nouvelle fonction pour construire l'URL blob-serve :
```typescript
export function blobServeUrl(
  daemonBaseUrl: string, hash: string, path: string = "index.html"
): string {
  return `${daemonBaseUrl}/blob-serve/${hash}/${path}`;
}
```

Nouvelle fonction pour extraire le hash depuis le daemon info :
```typescript
export async function getDaemonBaseUrl(coordUrl: string): Promise<string> {
  // Retourne http://127.0.0.1:7000 (ou l'adresse du daemon)
  const info = await getDaemonInfo(coordUrl);
  return `http://${info.data.api_host}:${info.data.api_port}`;
}
```

### 6.2 `BrowsedProject.tsx` : iframe universelle

Remplacer `RemoteProjectPlaceholder` par :

```tsx
function RemoteProjectFrame({ entry, daemonBaseUrl }: Props) {
  if (!entry.archive_ticket) {
    return <RemoteProjectPlaceholder />;  // v1 sans archive
  }

  const hash = extractHashFromTicket(entry.archive_ticket);

  return (
    <div className="flex-1 flex flex-col">
      {/* Banniere contenu tiers (D6) */}
      <div className="bg-amber-900/30 border-b border-amber-500/20 px-4 py-2 text-sm text-amber-200">
        Contenu publie par un tiers — non verifie par SBFB
      </div>
      {/* iframe isolee */}
      <iframe
        src={blobServeUrl(daemonBaseUrl, hash)}
        sandbox="allow-scripts"
        className="flex-1 w-full border-0"
        title={entry.project_name}
      />
    </div>
  );
}
```

### 6.3 Logique de routage mise a jour

```tsx
if (isLocal) {
  return <LocalProjectApps ... />;  // TabView React (retrocompat)
} else {
  return <RemoteProjectFrame entry={entry} daemonBaseUrl={daemonUrl} />;
}
```

### 6.4 Tests Phase C

**Vitest** (~8 nouveaux tests) :
- `BrowsedProject.test.tsx` : remote avec archive_ticket → iframe rendu
- `BrowsedProject.test.tsx` : remote sans archive_ticket → placeholder
- `BrowsedProject.test.tsx` : banniere "contenu tiers" visible
- `BrowsedProject.test.tsx` : iframe sandbox attrs corrects
- `BrowsedProject.test.tsx` : iframe src construit correctement
- `daemon.test.ts` : BrowseEntrySchema avec archive_ticket
- `daemon.test.ts` : blobServeUrl() construit l'URL correcte
- `daemon.test.ts` : getDaemonBaseUrl() parse daemon info

**Playwright** (~2 nouveaux specs) :
- `browse-remote-iframe.spec.ts` : Browse → clic distant → iframe affichee
- `browse-remote-iframe.spec.ts` : banniere "contenu tiers" visible

### 6.5 Critere d'acceptation Phase C

- `npm run test:unit` >= 181 (+8)
- `npx playwright test` >= 32 (+2)
- Cliquer un projet distant avec archive → iframe rendue
- Cliquer un projet distant sans archive → placeholder
- Banniere "contenu tiers" visible au-dessus de l'iframe
- `sandbox="allow-scripts"` sans `allow-same-origin`
- `npm run size` : tous budgets verts
- `bash scripts/scan-en-strings.sh` : exit 0

### 6.6 Commit cible

```
feat(web): Sprint 12 Phase C — cross-node iframe rendering with untrusted content isolation

- api/daemon.ts: archive_ticket in BrowseEntrySchema, blobServeUrl(),
  getDaemonBaseUrl()
- pages/BrowsedProject.tsx: RemoteProjectFrame with sandboxed iframe
  pointing to daemon blob-serve, "contenu tiers" warning banner
- sandbox="allow-scripts" without allow-same-origin (max isolation)
- 8 new Vitest + 2 new Playwright specs

Test delta:
  Vitest: 173 → ~181 (+8)
  Playwright: 30 → ~32 (+2)
Scope cuts honoured: no subdomain origin, no SW, no runtime templates
```

---

## 7. Phase D — Local publish integration + smoke test

### 7.1 Auto-publish avec archive au boot coordinator

Verifier que le coordinator `visibility=public` :
1. Pre-rend les TabView → HTML
2. Zip le tout
3. Stocke comme blob
4. Publie l'announcement v2 avec `archive_ticket`
5. Le daemon local a le zip dans son blob store

### 7.2 Smoke test cross-node

Test end-to-end :
1. Demarrer daemon + coordinator avec app gov
2. Verifier `GET /blob-serve/{hash}/index.html` retourne le HTML
3. Verifier que l'HTML contient les tabs rendus
4. Simuler un noeud distant qui clique → iframe affiche le contenu

### 7.3 VPS EU nginx update

**Fichier** `deploy/nginx-nexus.conf` — ajouter :

```nginx
# Blob serving (separate origin for iframe isolation)
location /blob-serve/ {
    proxy_pass http://127.0.0.1:7000/blob-serve/;
    proxy_set_header Host $host;
    # Pas de CORS — l'iframe est sur le meme hostname
    # mais port different via proxy_pass
}
```

Note : sur le VPS, nginx sert tout sur port 80. Le daemon est
sur 127.0.0.1:7000 (pas expose publiquement). Le proxy `/blob-serve/`
rend le daemon accessible mais c'est le meme origin que le shell.
Pour l'isolation, le daemon est accessible directement via port 7000
en local mais via proxy en production. L'isolation par port fonctionne
en local ; sur VPS, le CSP `connect-src 'none'` est la protection
principale.

### 7.4 Tests Phase D

**Python** (~3 tests) :
- `test_auto_publish_generates_archive` — coordinator public genere zip + ticket
- `test_blob_serve_gov_tabs` — les 19 tabs gov sont dans le zip

**Rust** (~2 tests) :
- `http::test_blob_serve_after_publish` — publish → blob-serve retourne le contenu
- `http::test_publish_blob_stores_and_returns_hash` — POST /publish-blob → hash hex

### 7.5 Commit cible

```
feat(p2p): Sprint 12 Phase D — local publish integration + cross-node smoke test

- coordinator.py: auto-publish generates zip archive with pre-rendered
  TabView HTML + stores blob + announces v2 with archive_ticket
- deploy/nginx-nexus.conf: /blob-serve/ proxy to daemon
- Integration test: publish → blob-serve → HTML content verified
- 3 new Python + 2 new Rust tests

Test delta:
  Rust: ~343 → ~345 (+2)
  coord: ~92+1 → ~95+1 (+3)
```

---

## 8. Phase E — Tech debt batch T28-T36

(Identique au plan precedent §7.1-7.12 — contenu inchange)

### 8.1 Items

- T28 : validate node_id hex dans from_gossip_bytes() — 1 test Rust
- T29 : test truncated gossip message — 1 test Rust
- T30 : tests coordinator daemon 500 + auto-publish private — 2 tests Python
- T31 : validate hex default_curators at config load — 1 test Rust
- T32 : DRY nginx config provision.sh → `cp nginx-nexus.conf`
- T33 : deploy/provision-tls.sh script certbot prep (~30 LOC)
- T34 : BrowsedProject.tsx dans vitest coverage.include
- T35 : rewrite test browse aggregate_flattens non-creux — 1 test Rust
- T36 : X-Forwarded-Proto dans /daemon/ nginx

Total : ~4 Rust + 2 Python tests

### 8.2 Commit cible

```
fix(tech-debt): Sprint 12 Phase E — close T28-T36 from Sprint 11 audit

Test delta:
  Rust: ~345 → ~349 (+4)
  coord: ~95+1 → ~97+1 (+2)
Tech debt: T28-T36 all CLOSED in PATTERNS.md
```

---

## 9. Phase F — Verification + audit plan

### 9.1 Livrables

- `.planning/sprint12_verification.md`
- `.planning/sprint12_audit_plan.md`

### 9.2 Mises a jour

- `docs/claude/README.md` §10 : Sprint 12 dans la table
- `docs/shell/PATTERNS.md` : P21-P23 (blob-serve, TabView pre-render, untrusted iframe) + T28-T36 CLOSED
- Memory `nexus_grid_pivot.md` : update tip, compteurs, Sprint 12 summary

### 9.3 Commit cible

```
docs(sprint12): verification + audit plan for Sprint 13
```

---

## 10. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | cargo test | `cargo test --workspace --locked` | >= 349 passed | |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | >= 179 passed | |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 97 passed + 1 skipped | |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | |
| 9 | tsc | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 10 | eslint | `npm run lint` | 0 errors | |
| 11 | vitest | `npm run test:unit` | >= 181 passed | |
| 12 | coverage | `npm run test:coverage` | lines >= 85%, branches >= 78% | |
| 13 | build | `npm run build` | exit 0 | |
| 14 | size-limit | `npm run size` | 7/7 green | |
| 15 | playwright | `npx playwright test` | >= 32 passed | |
| 16 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | |
| 17 | npm audit | `npm audit --audit-level=high` | 0 high/crit | |
| 18 | SPDX check | `bash scripts/check-spdx.sh` | exit 0 | |
| 19 | blob-serve local | `curl localhost:7000/blob-serve/{hash}/index.html` | 200 + HTML | |
| 20 | blob-serve CSP | reponse headers | `connect-src 'none'` present | |
| 21 | blob-serve sub-resource | `curl .../blob-serve/{hash}/assets/main.js` | 200 + JS | |
| 22 | deploy endpoint | `POST /project/deploy` avec zip | 200 + hash | |
| 23 | auto-publish zip | coordinator boot public | genere zip + announce v2 | |
| 24 | browse archive_ticket | `GET /browse` | entries avec archive_ticket | |
| 25 | iframe render | `/browse/:id` dans le browser | iframe visible, banniere "tiers" | |
| 26 | backward compat v1 | daemon v1 announcement | parse OK, pas d'archive | |
| 27 | T28-T36 CLOSED | `grep CLOSED docs/shell/PATTERNS.md` | 9 items marques | |
| 28 | SPDX nouveaux | `bash scripts/check-spdx.sh` | couvre les nouveaux fichiers | |
| 29 | verify.sh | `./scripts/verify.sh` | exit 0 | |

---

## 11. Git plan

| # | Phase | Commit |
|---|---|---|
| 0 | Planning | `docs(sprint12): kickoff + plan detaille with D1-D7` |
| 1 | A | `feat(p2p): Sprint 12 Phase A — daemon blob-serve endpoint with zip decompression + CSP isolation` |
| 2 | B | `feat(p2p): Sprint 12 Phase B — publish pipeline with TabView pre-render + universal zip deploy` |
| 3 | C | `feat(web): Sprint 12 Phase C — cross-node iframe rendering with untrusted content isolation` |
| 4 | D | `feat(p2p): Sprint 12 Phase D — local publish integration + cross-node smoke test` |
| 5 | E | `fix(tech-debt): Sprint 12 Phase E — close T28-T36 from Sprint 11 audit` |
| 6 | F | `docs(sprint12): verification + audit plan for Sprint 13` |

---

## 12. Scope cuts (copie kickoff §5)

- Pas de branding SBFB — Sprint 13
- Pas de 2 VPS supplementaires (US/Asia) — Sprint 13
- Pas de runtime templates (`sbfb publish --type python`) — Sprint 13
- Pas de re-publish automatique — Sprint 13
- Pas de origin separee par subdomain — Sprint 13
- Pas de multi-writer iroh-docs — v1.1+
- Pas de custom domain / DNS — Sprint 13+
- Pas de HTTPS live — T33 prep seulement

---

## 13. Checkpoint de cloture

Le sprint est ferme quand :
1. 29/29 fail-fast checklist green
2. 7 commits atomiques landed sur master
3. `sprint12_verification.md` + `sprint12_audit_plan.md` ecrits
4. PATTERNS.md a jour (P21-P23 + T28-T36 CLOSED)
5. Memory `nexus_grid_pivot.md` mise a jour
6. N'importe quel projet avec un zip+index.html se rend en iframe
   isolee pour un utilisateur distant
