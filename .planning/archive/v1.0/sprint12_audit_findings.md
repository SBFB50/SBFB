# Sprint 12 — Audit findings (Phase 0 de Sprint 13)

**Auditeur** : session fraiche Claude Opus 4.6, 2026-04-13
**Tip audite** : `48449f9` (docs commit Phase F, code tip `bf3f009`)
**Timebox** : ~45 min (8 agents paralleles + verification manuelle)
**Methode** : 8 tracks (A-I) du `sprint12_audit_plan.md`, joues en
parallele par des agents independants, findings consolides et
reverifies manuellement par l'auditeur principal.

---

## Verdict global : CONDITIONAL PASS

- **0 P0** (2 signales par les agents, tous deux invalides apres
  verification manuelle)
- **1 P1** (deploy endpoint sans limite de taille)
- **4 P2** (defense-in-depth CSP, chart dimensions, test manquant,
  nginx header)
- **0 P3**

Le P1 doit etre fixe en `fix(sprint12): ...` avant que Sprint 13
Phase A puisse demarrer. Les P2 sont logges en tech debt.

---

## Track A — Securite blob-serve : PASS

Analyse complete de `blob_serve.rs` (270 LOC) et du handler HTTP
`blob_serve()` dans `http.rs:515-575`.

**Path traversal** : protege. `validate_zip_path()` bloque `..`,
`/`, `\`, chemins vides (`blob_serve.rs:174-188`). Les variantes
URL-encodees (`%2e%2e%2f`) sont decodees par axum avant validation.
Test `blob_serve_rejects_path_traversal()` (`http.rs:1245-1266`).

**Zip bomb** : protege. `DEFAULT_MAX_DECOMPRESSED_BYTES = 100MB`
(`blob_serve.rs:32`), enforce cumulativement pendant la
decompression (`blob_serve.rs:118-122`). Test
`load_rejects_oversized_archive()` (`blob_serve.rs:408`).

**Content-Type** : detection par extension + magic bytes
(`blob_serve.rs:199-254`). `X-Content-Type-Options: nosniff` sur
la reponse 200 (`http.rs:569`).

**CSP** : `connect-src 'none'; frame-ancestors *` injecte sur
les reponses 200 (`http.rs:568`). Voir P2-01 pour les reponses
d'erreur.

**Unsafe** : zero bloc `unsafe` dans blob_serve.rs et le handler.

**Iframe sandbox** : `sandbox="allow-scripts"` SANS
`allow-same-origin` dans `BrowsedProject.tsx:442`. Correct.

### Finding P2-01 — CSP absent sur les reponses d'erreur blob-serve

**Fichier** : `crates/nexus-shell-daemon/src/http.rs:528, 536, 546, 551, 559`

Les 5 chemins d'erreur du handler `blob_serve()` retournent
`(StatusCode, &str).into_response()` sans les headers CSP.

**Risque pratique : negligeable.** Les reponses sont en
`text/plain` (default axum pour `(StatusCode, &str)`) avec du
contenu hardcode (pas de reflection d'input utilisateur). Le seul
champ dynamique est `format!("invalid archive: {e}")` (ligne 546)
ou `e` provient du crate `zip` Rust. Un navigateur n'execute pas
de JS depuis `text/plain`.

**Recommandation** : ajouter les headers CSP sur toutes les
reponses blob-serve comme defense-in-depth. Peut se faire via
un middleware axum `tower::ServiceBuilder::layer()` sur le groupe
de routes `/blob-serve/*`.

---

## Track B — TabView pre-render fidelite : PASS avec P2

**Block coverage** : les 12 block kinds sont implementes dans
`html_render.py:462-475` (heading, text, kv, metric, table,
badge_list, button, chart_line, chart_bar, empty, section,
file_upload). Les kinds inconnus sont silencieusement ignores
(`html_render.py:483`). Correct.

**XSS prevention** : toutes les valeurs utilisateur passent par
`_esc()` (`html.escape()`, `html_render.py:185`). 24 appels
d'echappement verifies. Le label SVG chart est echappe via
`_esc(label)` a la ligne 231, incluant `y_unit`. Pas de vecteur
XSS.

**CSS dark theme** : coherent avec `web/src/index.css`
(`html_render.py:34-162`). PASS.

### Finding P2-02 — Dimensions SVG charts divergent du React shell

**Fichier** : `packages/nexus-sdk/src/nexus_sdk/html_render.py:192-197`
vs `web/src/components/app/tabview/blocks/ChartLineBlock.tsx:4-7`
et `ChartBarBlock.tsx:4-7`.

| Parametre | html_render.py | React Line | React Bar |
|-----------|---------------|------------|-----------|
| W | 400 | 400 | 400 |
| H | **180** | **120** | **120** |
| PAD_L | **45** | **32** | **32** |
| PAD_R | **10** | **16** | **16** |
| PAD_T | **10** | **16** | **24** |
| PAD_B | **30** | **16** | **24** |

Les charts pre-rendus sont plus hauts et avec un padding different.
Pas de HTML casse, mais difference visuelle significative.

**Recommandation** : aligner les constantes sur les valeurs React.

### Finding P2-03 — Pas de test file_upload dans test_html_render.py

**Fichier** : `packages/nexus-sdk/tests/test_html_render.py`

11/12 block kinds sont testes. Le block `file_upload` est
implemente dans le renderer mais n'a pas de test dedie.

**Recommandation** : ajouter `test_render_file_upload()`.

---

## Track C — Deploy endpoint securite : CONDITIONAL PASS (1 P1)

**Validation zip** : correcte. `_validate_zip()` verifie format +
presence de `index.html` (`deploy.py:34-46`). Tests pour valid,
invalid et missing index.html (`test_deploy.py:118-179`).

**Auth** : loopback-only (CORS 127.0.0.1/localhost). Acceptable.

**Race conditions** : aucune. Chaque deploy est idempotent
(content-addressed blob store). Pas d'etat partage mutable.

**Cleanup** : pas de fichiers temporaires. Tout en memoire via
`await archive.read()` et `io.BytesIO()`. Correct.

### Finding P1-01 — Absence de limite de taille sur POST /project/deploy

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:110`

`zip_bytes = await archive.read()` lit l'upload complet en memoire
sans aucune verification de taille. Le kickoff Risk R1 specifiait
explicitement "Limiter taille decompresse a 100MB".

Le pattern existe deja dans le CAS : `files.py:59-80` implemente
`_upload_chunks()` avec `max_size_bytes` et reponse HTTP 413.

**Impact** : un upload malveillant ou accidentel de plusieurs GB
peut OOM le coordinator.

**Fix requis** : ajouter `max_size_bytes` (100MB) sur le deploy
endpoint avec reponse 413 si depasse. Ajouter
`test_deploy_oversized_zip()`.

---

## Track D — Auto-publish integration : PASS

**Flow auto-publish** : correct. Le coordinator public pre-rend
les TabView, zippe, stocke comme blob, publie avec
`archive_hash` (`coordinator.py:384-440`).

**archive_hash dans payload** : present conditionnellement
(`coordinator.py:421-422`). Correct.

**Guard private** : `if self.config.network.visibility == "public"`
(`coordinator.py:387`). Un coordinator prive ne publie pas.
Test `test_auto_publish_skipped_when_private()` confirme.

**Coordinator sans apps** : gere proprement. `if self.apps:`
(ligne 504) empeche la creation du root `index.html` quand il
n'y a pas d'apps. `if file_count == 0: return None` (ligne 512)
skip l'upload. Pas de crash.

**Index.html redirects** : per-app et root, via `<meta refresh>`.
Correct.

---

## Track E — Frontend cross-node rendering : PASS

**Iframe sandbox** : `sandbox="allow-scripts"` sans
`allow-same-origin` (`BrowsedProject.tsx:442`). Correct.

**Banniere "contenu tiers"** : presente, visible, texte FR
"Contenu publie par un tiers — non verifie par SBFB"
(`BrowsedProject.tsx:437-439`). Correct.

**Fallback sans archive** : `if (!entry.archive_hash || !daemonInfo)`
(`BrowsedProject.tsx:417`) → placeholder card au lieu d'iframe
cassee. Correct.

**URL construction** : `blobServeUrl()` construit
`${daemonBaseUrl}/blob-serve/${hash}/${path}`
(`daemon.ts:380-386`). Teste (`daemon.test.ts:587-595`).

**Zod schema** : `archive_ticket: z.string().optional()` et
`archive_hash: z.string().optional()` (`daemon.ts:204-205`).
Correct.

**Coverage** : `BrowsedProject.tsx` dans vitest coverage includes
(`vitest.config.ts:45`). Tests couvrent : local, remote avec
archive, remote sans archive, banniere, sidebar metadata.

---

## Track F — Deploy infrastructure : PASS avec P2

**T32 nginx** : `provision.sh` utilise `cp` pour nginx config
(ligne 45). Les heredocs restants (lignes 78, 99, 118) sont pour
les systemd units et le script de demarrage — pas le scope de T32.
Correct.

**X-Forwarded-Proto** : present sur `/api/` (ligne 28), `/daemon/`
(ligne 38), `/blob-serve/` (ligne 49). 3/3.

**provision-tls.sh** : existe. `--non-interactive` (ligne 35),
`--redirect` (ligne 37) pour HTTP→HTTPS. Correct.

**Securite** : pas de credentials hardcodees.

### Finding P2-04 — X-Real-IP manquant sur /blob-serve/ nginx

**Fichier** : `deploy/nginx-nexus.conf:48`

Le header `X-Real-IP` est present sur les blocs `/api/` et
`/daemon/` mais absent sur `/blob-serve/`. Inconsistant.
Impact mineur (loopback), mais corrige l'asymetrie.

---

## Track G — Tech debt T28-T36 : PASS

Les 9 items sont verifies CLOSED dans `docs/shell/PATTERNS.md` :

| Item | Verification | Statut |
|------|-------------|--------|
| T28 | `InvalidNodeId` dans publish.rs:64,117 | CLOSED |
| T29 | Test truncated gossip dans publish.rs | CLOSED |
| T30 | Test daemon 500 dans test_daemon_proxy.py:683 | CLOSED |
| T31 | `retain()` hex validation dans config.rs:307 | CLOSED |
| T32 | `cp nginx-nexus.conf` dans provision.sh:45 | CLOSED |
| T33 | provision-tls.sh existe | CLOSED |
| T34 | BrowsedProject dans vitest.config.ts:45 | CLOSED |
| T35 | Test aggregate_flattens dans browse.rs:582 | CLOSED |
| T36 | X-Forwarded-Proto 3x dans nginx-nexus.conf | CLOSED |

---

## Track I — BrowseEntry backward compat : PASS

**Serde Rust** : `archive_ticket` et `archive_hash` ont
`#[serde(default, skip_serializing_if = "Option::is_none")]`
dans `BrowseEntry` (`browse.rs:164-171`) et
`ProjectAnnouncement` (`publish.rs:46-47`). Correct.

**Zod** : `.optional()` sur les deux champs (`daemon.ts:204-205`).
Correct.

**Tests** : roundtrip v2 (`browse.rs:479-499`), omission v1
(`browse.rs:502-521`), `v1_announcement_parses_without_archive_ticket`
(`publish.rs:241-256`). Frontend : placeholder quand undefined
(`BrowsedProject.test.tsx:196-212`). Coverage complete.

---

## Findings tries par severite

| ID | Severite | Track | Description | Fichier |
|----|----------|-------|-------------|---------|
| P1-01 | P1 | C | Absence de limite de taille sur POST /project/deploy | deploy.py:110 |
| P2-01 | P2 | A | CSP headers absents sur reponses d'erreur blob-serve | http.rs:528-559 |
| P2-02 | P2 | B | Dimensions SVG charts divergent du React shell | html_render.py:192-197 |
| P2-03 | P2 | B | Pas de test file_upload dans test_html_render.py | test_html_render.py |
| P2-04 | P2 | F | X-Real-IP manquant sur /blob-serve/ nginx | nginx-nexus.conf:48 |

## Commits fix attendus

1. `fix(sprint12): add 100MB upload size limit to deploy endpoint`
   — Ajouter `max_size_bytes=100*1024*1024` dans `deploy_project()`,
   reponse HTTP 413 si depasse. Ajouter
   `test_deploy_oversized_zip()`. Pattern a copier de
   `files.py:59-80`.

## P2 a logger en tech debt

- **T37** : CSP middleware pour toutes les reponses blob-serve
  (http.rs, defense-in-depth)
- **T38** : Aligner dimensions SVG charts html_render.py sur React
  (H=120, PAD matching)
- **T39** : Test file_upload block dans test_html_render.py
- **T40** : X-Real-IP header dans bloc nginx /blob-serve/

## Faux positifs signales par les agents et invalides

1. **Track A "P0 XSS CSP bypass"** : les reponses d'erreur sont
   `text/plain` avec contenu hardcode, pas de vecteur d'execution.
   Downgrade a P2-01.

2. **Track B "P0 y_unit XSS"** : `_esc(label)` a la ligne 231
   echappe le label complet incluant y_unit. `html.escape()` couvre
   `<`, `>`, `&`, `"`, `'`. Pas de vecteur.

3. **Track D "P1 empty coordinator"** : `if self.apps:` (ligne 504)
   empeche le root index.html quand pas d'apps, `file_count == 0`
   retourne None. Pas de crash.

4. **Track G "P1 T32 heredocs"** : T32 concerne le nginx config
   specifiquement (bien en `cp` ligne 45). Les heredocs restants
   sont pour systemd units, hors scope T32.

## Notes on audit completeness

- Timebox respecte (~45 min avec agents paralleles)
- 8 tracks (A-I) joues integralement
- Tous les tests rejoues et verts :
  - Rust 362, SDK 182, coord 95+1, gov 46, Vitest 180, Playwright
    30, size-limit 7/7
- PATTERNS.md NON lu avant formation des opinions (consigne §2
  du audit_plan respectee)
- Pas de Playwright e2e remote iframe (hors scope per audit_plan)
