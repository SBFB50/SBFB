# Sprint 12 — Kickoff (Rendu universel : archive zip + daemon blob-serve + iframe isolee)

**Ecrit** : 2026-04-13
**Tip master d'entree** : `31479fa` (Sprint 11 audit T28-T36 tech debt log)
**Phase 0 audit** : DONE. Sprint 11 audit CONDITIONAL PASS leve
dans `f2c94e3` (3 P1 + 1 P1 fixes). Gate verte.

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-11 **CLOSED**. v1.0.0 released.
- Le flow P2P **publication** fonctionne (gossip + curator lists)
- Le flow P2P **consommation locale** fonctionne (TabView plein ecran)
- Le flow P2P **consommation distante** ne fonctionne pas :
  placeholder "Projet distant" pour tout projet non-local
- SBFB est une plateforme ouverte : n'importe qui publie,
  le contenu est **untrusted par defaut** (zero moderation
  centrale, decision Day 0 gelee)
- Infrastructure existante sous-exploitee :
  - `WebAppFrame.tsx` (iframe sandbox, 35 LOC) — fonctionnel
  - CAS file upload (711 LOC SDK + 324 LOC API) — fonctionnel
  - `BlobsClient` Rust (add/get/fetch) — fonctionnel
  - `GET /app/{name}/files/{sha256}` (bytes bruts) — fonctionnel
- 25 items de tech debt ouverts (T1 + T6-T7 + T13-T36)

### 1.2 Compteurs de tests a l'entree

| Suite | Count |
|---|---|
| Rust workspace | 331 |
| Python SDK | 167 |
| Python coordinator | 89 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 173 |
| Playwright | 30 |
| size-limit | 7/7 |
| SPDX | 209/209 |

### 1.3 Le probleme

SBFB promet "n'importe qui publie une app, n'importe qui
l'utilise". Aujourd'hui :
1. Alice publie une app React (ou Python, ou HTML, ou notebook)
2. Bob la decouvre dans Browse
3. Bob clique → **dead end** "Projet distant"

La plateforme ne tient pas sa promesse. Le rendu cross-node
n'existe pas, et le modele de rendu est couple au shell React
(TabViewRenderer) au lieu d'etre universel.

### 1.4 Vision plateforme

SBFB est une **plateforme d'hebergement P2P universelle**.
Chaque projet publie une **archive web** (zip avec index.html).
Le reseau distribue l'archive via iroh-blobs. Le daemon la
decompresse et la sert via HTTP. Les clients la rendent dans
un iframe isolee. Toute techno qui produit du HTML est
supportee (React, Vue, Python/Pyodide, WASM, notebooks, HTML
pur, markdown).

---

## 2. Goal en une phrase

**N'importe qui publie un zip avec un index.html, n'importe
qui clique dans Browse et voit l'app tourner dans une iframe
isolee — quelle que soit la techno, avec isolation de securite
maximale pour le contenu untrusted.**

---

## 3. Decisions Day 0 (D1..D7 gelees)

### D1 — Archive zip = format universel de publication

**Retenu** : tout projet publie sur SBFB est un zip contenant
un `index.html` comme point d'entree + tout fichier necessaire
(JS, CSS, images, WASM, .py, etc.). Le zip est stocke comme
un blob iroh unique.

**Rejete** :
- Fichier HTML unique (limite aux apps single-file)
- Repertoire de blobs individuels (N fetches P2P, complexe)
- Docker container (overkill, runtime lourd)

**Implications** :
- Un blob = un fetch P2P, meme pour des apps multi-fichiers
- Le dev fait `npm run build` puis zip son `dist/`
- Le coordinator peut pre-rendre les TabView en HTML et les
  zipper automatiquement

### D2 — Le daemon sert les blobs directement (origin separee)

**Retenu** : le daemon Rust expose `GET /blob-serve/{hash}/{path}`
qui decompresse le zip et sert le fichier demande. L'iframe
pointe directement vers le daemon (port 7000), pas vers le
coordinator (port 8000). Port different = origin differente =
isolation iframe maximale.

**Rejete** :
- Service Worker gateway (requiert `allow-same-origin` pour
  intercepter les sub-resources → annule le sandbox avec du
  contenu untrusted)
- Coordinator proxy (meme origin que le shell → pas d'isolation)
- Blob URL (`URL.createObjectURL`) — single-file uniquement

**Implications** :
- Le daemon gagne un endpoint HTTP + un cache LRU des zips
  decompresses
- Le frontend construit le `src` iframe directement vers le
  daemon
- Aucun Service Worker necessaire
- Marche aussi pour des clients non-browser (mobile, CLI, Electron)

### D3 — Sandbox iframe maximal pour contenu untrusted

**Retenu** : `sandbox="allow-scripts"` SANS `allow-same-origin`.
L'iframe a une origin opaque — zero acces au shell, zero acces
aux cookies/storage du shell. Le JS tourne a 100% dans l'iframe
(animations, React, WASM, tout marche).

Le daemon injecte des headers CSP sur chaque reponse :
```
Content-Security-Policy: connect-src 'none'; frame-ancestors *
X-Content-Type-Options: nosniff
```

`connect-src 'none'` bloque tout `fetch()`/XHR sortant depuis
l'iframe → bloque crypto mining (pas de pool), bloque
exfiltration de fingerprint, bloque DDoS relay.

**Rejete** :
- `allow-same-origin` (contenu untrusted peut echapper au
  sandbox et acceder au shell)
- Pas de CSP (laisse les requetes sortantes ouvertes)

**Implications** :
- Les apps dans l'iframe ne peuvent pas faire de requetes
  reseau (pas de fetch API, pas de CDN externe)
- Tout doit etre dans le zip (JS, CSS, fonts, images)
- Pyodide doit etre bundle dans le zip (pas de CDN load)

### D4 — ProjectAnnouncement v2 avec archive_ticket

**Retenu** : `ProjectAnnouncement` v2 backward-compatible.
Nouveau champ `archive_ticket: Option<String>` = le BlobTicket
du zip archive. `BrowseEntry` propage le champ. Les daemons
v1 ignorent v2 (deja le cas).

**Rejete** :
- Manifest JSON separe du zip (double blob, double fetch)
- Pas de ticket dans l'announcement (force un lookup supplementaire)

### D5 — TabView pre-rendu en HTML statique par le coordinator

**Retenu** : au moment du publish, le coordinator :
1. Collecte les descriptors de toutes les apps TabView
2. Genere un HTML statique auto-contenu (~460 LOC Python,
   zero deps) avec inline CSS (dark theme, ~140 lignes)
3. Zippe le HTML → blob → announce

Les apps TabView existantes (gov, hello-world) passent par le
meme chemin que tout le reste : zip → daemon → iframe.

**Rejete** :
- Garder le TabViewRenderer React pour le cross-node (couple
  le rendu au shell, bloque les clients non-React)

**Implications** :
- Le shell React garde son TabViewRenderer pour les pages
  locales (ProjectDetail) — retrocompat
- Le cross-node est 100% iframe, techno-agnostique
- Les charts SVG sont portes (meme algo de coordonnees,
  pas de lib chart)
- Les blocks interactifs (button, file_upload) sont rendus
  en `<form>` ou placeholder statique

### D6 — Banniere "contenu tiers" dans le shell

**Retenu** : le shell affiche une banniere visible au-dessus
de l'iframe pour tout contenu distant : "Contenu publie par
un tiers — non verifie par SBFB". Mitigation anti-phishing.

### D7 — Tech debt batch T28-T36

**Retenu** : Phase E ferme les 9 items T28-T36 issus de l'audit
Sprint 11.

---

## 4. Plan Phase outline A..F

### Phase A — Daemon blob-serve endpoint + zip decompression (2j)

- Dep Rust : crate `zip` pour decompression
- Endpoint `GET /blob-serve/{hash}/{path}` dans le daemon HTTP
- Decompression zip depuis le blob store iroh
- Cache LRU en memoire des zips decompresses
- Content-Type detection (extension + magic bytes)
- Headers CSP `connect-src 'none'` sur chaque reponse
- Pour les blobs distants : `BlobsClient::fetch_ticket()` avant
  decompression
- ProjectAnnouncement v2 : `archive_ticket: Option<String>`
- BrowseEntry : `archive_ticket: Option<String>`
- Tests Rust
- **Commit** : `feat(p2p): Sprint 12 Phase A — daemon blob-serve
  endpoint with zip decompression + CSP isolation`

### Phase B — Pipeline publish (coordinator zip + announce) (1-2j)

- Coordinator `POST /project/deploy` : accepte un zip upload
  multipart, stocke comme blob via daemon, publie announcement v2
- TabView pre-render `render_tabview_to_html()` : ~460 LOC Python,
  zero deps, genere HTML auto-contenu avec inline CSS/SVG
- Au publish des apps SDK : le coordinator pre-rend les TabView,
  genere un zip, publie comme n'importe quelle app web
- Extension `POST /project/publish` existant pour inclure
  `archive_ticket`
- Ajouter `text/html` a `ALLOWED_MAGIC_BYTES` dans le CAS SDK
- Tests Python
- **Commit** : `feat(p2p): Sprint 12 Phase B — publish pipeline
  with TabView pre-render + universal zip deploy`

### Phase C — Frontend cross-node rendering (1-2j)

- `BrowsedProject.tsx` : remplacer `RemoteProjectPlaceholder`
  par une iframe pointant vers le daemon blob-serve
- Construire l'URL : `http://{daemon_host}:{daemon_port}/blob-serve/{hash}/index.html`
- `sandbox="allow-scripts"` sans `allow-same-origin`
- Banniere "contenu tiers" au-dessus de l'iframe
- Loading state pendant le fetch blob P2P
- Erreur propre si noeud offline ou zip invalide
- Garder le placeholder si `archive_ticket` absent (compat v1)
- Zod schema : `archive_ticket: z.string().optional()` dans
  BrowseEntrySchema
- Tests Vitest + Playwright
- **Commit** : `feat(web): Sprint 12 Phase C — cross-node iframe
  rendering with untrusted content isolation`

### Phase D — Local publish integration + smoke test (1j)

- Le coordinator local (gov) se publie automatiquement avec
  TabView pre-rendu au boot si `visibility=public`
- Smoke test : un noeud distant voit le projet gov dans Browse,
  clique, et voit l'app rendue dans l'iframe (HTML statique
  genere depuis les 19 tabs)
- Verifier que le VPS EU sert le meme flow
- Tests integration
- **Commit** : `feat(p2p): Sprint 12 Phase D — local publish
  integration + cross-node smoke test`

### Phase E — Tech debt batch T28-T36 (1j)

- T28-T31 : validations Rust + tests Python
- T32-T33 : deploy scripts (DRY nginx, HTTPS prep)
- T34-T36 : coverage frontend + nginx
- **Commit** : `fix(tech-debt): Sprint 12 Phase E — close T28-T36
  from Sprint 11 audit`

### Phase F — Verification + audit plan (0.5j)

- `.planning/sprint12_verification.md`
- `.planning/sprint12_audit_plan.md`
- Update memory + docs + PATTERNS.md
- **Commit** : `docs(sprint12): verification + audit plan for
  Sprint 13`

---

## 5. Scope cuts

- **Pas de branding SBFB** — Sprint 13
- **Pas de 2 VPS supplementaires** (US/Asia) — Sprint 13
- **Pas de runtime templates** (`sbfb publish --type python`) —
  Sprint 13. Sprint 12 le dev prepare son zip lui-meme.
- **Pas de re-publish automatique** — Sprint 13
- **Pas de origin separee par subdomain** (`{hash}.app.sbfb.local`)
  — Sprint 13. Sprint 12 utilise le port daemon comme origin.
- **Pas de multi-writer iroh-docs** — v1.1+
- **Pas de custom domain / DNS** — Sprint 13+
- **Pas de HTTPS live** — T33 prep seulement

---

## 6. Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | Zip bomb (zip compresse a 1000:1) | OOM daemon | Limiter taille decompresse a 100MB |
| R2 | BlobTicket stale apres restart | Fetch echoue | iroh pkarr resolve via relay N0 |
| R3 | TabView pre-render diverge du React renderer | UX inconsistante | Tester visuellement les 19 tabs gov |
| R4 | CSP `connect-src 'none'` casse des apps legit | Apps qui fetch des APIs externes | Documenter la contrainte, relaxer en Sprint 13 avec allow-list |
| R5 | Pyodide sans CDN = zip de 40MB+ | Latence premier fetch | Acceptable, cache apres premier fetch |
| R6 | `zip` crate Rust ajoute du build time | CI plus lent | Crate mature, compile rapide |
| R7 | Port daemon pas expose sur VPS nginx | iframe 404 | Ajouter location /blob-serve/ dans nginx |

---

## 7. Audit gate pattern — rappel

- Phase 0 audit Sprint 11 jouee et fermee (tip `f2c94e3`)
- Phase F produira `sprint12_audit_plan.md` pour Sprint 13 Phase 0
- Convention permanente respectee

---

## 8. Checkpoint de validation

L'utilisateur confirme :
1. **D1** : zip = format universel
2. **D2** : daemon sert directement (origin separee)
3. **D3** : `sandbox="allow-scripts"` + CSP `connect-src 'none'`
4. **D4** : ProjectAnnouncement v2 avec archive_ticket
5. **D5** : TabView pre-rendu en HTML par le coordinator
6. **D6** : banniere "contenu tiers"
7. **D7** : tech debt T28-T36
8. **Goal** : n'importe quelle app, n'importe quelle techno,
   rendue en iframe isolee via P2P
