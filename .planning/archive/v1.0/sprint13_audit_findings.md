# Sprint 13 — Audit findings (Phase 0 Sprint 14)

**Auditeur** : session fraiche 2026-04-13
**Tip audite** : `f5ea8d0` (master, post-Sprint 13 docs)
**Commit stack audite** : `b44f40c..08853ff` (planning + Phase A-D + docs)
**Timebox observe** : ~1h30

---

## Verdict global : CONDITIONAL PASS

- **P0** : 0
- **P1** : 1 (stale running.json dans launcher)
- **P2** : 3
- **P3** : 5

Le P1 doit etre fixe en commit `fix(sprint13): ...` avant le premier
commit Sprint 14 Phase A.

---

## Track A — Bridge securite

**Verdict : PASS**

L'implementation du bridge postMessage est correctement securisee :

- **Whitelist stricte** : `BridgeMethodSchema = z.enum(["task_submit",
  "storage_get", "storage_set"])` dans `protocol.ts`. Toute methode
  non-whitelist est rejetee au parsing Zod avant le dispatch.
- **Source validation** : `event.source === iframe.contentWindow`
  (reference exacte, non contournable cross-origin) dans
  `useBridge.ts:49`.
- **Double validation payload** : Zod au niveau protocol, puis
  re-validation par methode dans dispatch (ex: `SubmitAppTaskBodySchema`
  pour `task_submit`).
- **Sandbox iframe** : `sandbox="allow-scripts"` sans
  `allow-same-origin` (origin opaque, pas d'acces DOM/storage host).
- **URL encoding** : `encodeURIComponent()` sur appName et key dans
  les URLs coordinator.

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| A-1 | P3 | `reply(target, response)` utilise `postMessage(response, "*")` au lieu de l'origin cible. Impact faible (UUID correlation, pas de secrets dans la reponse) mais un `event.origin` serait plus strict. |

---

## Track B — Open source enforcement

**Verdict : PASS (avec notes pour Sprint 14)**

La validation `repo_url` est bien en place cote coordinateur
(`deploy.py:128`) et non contournable cote client :
- Public sans `repo_url` → 400 (teste par `test_deploy_public_without_repo_url_rejected`)
- Public avec `repo_url` → 200
- Prive sans `repo_url` → 200
- Propagation correcte : Rust `ProjectAnnouncement` v3 → gossip →
  `BrowseEntry` → Zod `.optional()` → UI lien cliquable.

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| B-1 | P2 | Aucune validation de format URL. `repo_url` accepte toute string non-vide incluant `javascript:alert(1)`. Le frontend rend `<a href={entry.repo_url}>` (Browse.tsx:252, BrowsedProject.tsx:260) — XSS possible au clic. Sprint 14 remplace le systeme par deploy verifie from source, donc non-bloquant pour la gate mais a noter. |

Note : les `rel="noopener noreferrer"` et `target="_blank"` sont
correctement presents sur les liens repo (pas d'acces window.opener).

---

## Track C — Launcher robustesse

**Verdict : CONCERN (1 P1)**

Le launcher gere correctement :
- Daemon deja running : reutilise sans spawner un second (main.rs:110-115)
- Daemon introuvable : message d'erreur clair + exit 1 (main.rs:123-129)
- Ctrl+C : SIGTERM Unix / kill Windows + timeout 5s (main.rs:170-198)

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| C-1 | **P1** | **Stale running.json non detecte.** `read_running_info()` lit le PID mais ne verifie jamais si le process est vivant (`pid` a `#[allow(dead_code)]`). Si le daemon crash, le launcher lit le fichier stale, ouvre le browser sur un daemon mort, et l'utilisateur est bloque (doit supprimer manuellement `~/.nexus-grid/shell-daemon/running.json`). Le daemon-core a deja `check_stale_or_bail()` dans `registry.rs:291` qui fait exactement cette verification via sysinfo — le launcher devrait reutiliser cette logique ou au minimum faire un health check HTTP. |
| C-2 | P3 | `libc` utilise pour `SIGTERM` sur Unix (main.rs:175) mais pas declare dans `Cargo.toml` — dependance transitive via tokio "full". Fragile si les features tokio changent. |

---

## Track D — UI glassmorphism accessibilite

**Verdict : PASS**

Le redesign glassmorphism est visuellement coherent et globalement
accessible. Navigation clavier fonctionnelle, liens externes avec
`rel="noopener noreferrer"`, `scan-en-strings.sh` propre, aria-labels
presents sur les elements critiques.

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| D-1 | P2 | 2 instances `text-white/30` sur texte 11px : ProjectDetail.tsx:131 (URL coordinateur) et BrowsedProject.tsx:271 (label "sandbox"). Ratio ~3.3:1 sous le seuil WCAG AA 4.5:1 pour texte normal. Recommandation : monter a `text-white/40` (ratio ~4.4:1). |
| D-2 | P3 | ~12 instances `text-white/40` sur labels secondaires (10-12px). Ratio ~4.4:1 — borderline WCAG AA pour texte normal (seuil 4.5:1). Acceptable pour du texte secondaire/metadata. |

---

## Track E — Backward compat BrowseEntry v3

**Verdict : PASS**

Tous les criteres de backward compat sont satisfaits :
- `ProjectAnnouncement` : `#[serde(default, skip_serializing_if = "Option::is_none")]` sur `repo_url` (publish.rs:52)
- `BrowseEntry` : idem (browse.rs:175)
- Pas de `#[serde(deny_unknown_fields)]` sur les types wire
- Zod : `repo_url: z.string().optional()` (daemon.ts:206)
- Tests : `v3_announcement_with_repo_url_round_trips()`, `v2_announcement_parses_without_repo_url()`, `v3_announcement_without_repo_url_omits_field()` — tous verts
- Version check accepte v1-v3 (publish.rs:117)

Aucun finding.

---

## Track F — Tests et couverture

**Verdict : PASS**

Compteurs verifies independamment (tous verts) :

| Suite | Plan | Observe | Delta |
|-------|------|---------|-------|
| Rust workspace | 369 | 369 | = |
| Python SDK | 183 | 183 | = |
| Python coord | 99+1 | 99+1 | = |
| Python gov | 46 | 46 | = |
| Vitest | 191 | 191 | = |
| Playwright | 30 | non rejoue (env CI) | — |

Le self-report est **exact** sur tous les compteurs verifiables.

Deploy tests : 3/3 cas couverts (public sans → 400, public avec → 200,
prive sans → 200).

Bridge tests : 9 tests (depasse le plan de 5+). Protocol validation
+ source check + error handling couverts.

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| F-1 | P3 | `storage_get` et `storage_set` ont des tests de validation schema mais pas de tests d'integration dispatch dans `useBridge.test.ts` (seul `task_submit` est teste en dispatch mock). |

---

## Track G — Tech debt T37-T40

**Verdict : PASS (1 P2 sur T38)**

| Item | Status | Verification |
|------|--------|-------------|
| T37 CSP middleware | CLOSED | Middleware `blob_serve_csp_middleware` monte sur routes blob-serve (http.rs:127,147). Test `blob_serve_error_responses_have_csp` verifie 404 avec headers CSP. |
| T38 SVG dimensions | CLOSED (P2) | Constantes H=120, PAD_L=32, PAD_T/B corrects. Mais **PAD_R=16 diverge de React PAD_X=32** (symmetrique). Plot widths : Python=352px vs React=336px. |
| T39 file_upload test | CLOSED | Test `test_render_file_upload()` (test_html_render.py:322) existe et passe. |
| T40 nginx X-Real-IP | CLOSED | `proxy_set_header X-Real-IP $remote_addr;` present dans les 3 location blocks de nginx-nexus.conf. |

Findings :

| ID | Sev | Finding |
|----|-----|---------|
| G-1 | P2 | T38 PAD_R=16 ne matche pas React PAD_X=32 (cote droit). Le plan Sprint 13 §3.4 listait "Cible (React) PAD_R=16" mais la valeur React reelle est 32 pour les deux cotes. Le plan research etait incorrect. |
| G-2 | P3 | T37-T40 marques CLOSED dans PATTERNS.md sans SHA de commit (contrairement a P24-P26 qui ont des SHAs). |

---

## Findings list sorted by severity

| ID | Sev | Track | Finding |
|----|-----|-------|---------|
| C-1 | **P1** | Launcher | Stale running.json : PID lu mais jamais verifie. Daemon crash → launcher bloque. |
| B-1 | P2 | Open source | repo_url accepte toute string (javascript: XSS possible au clic). Sprint 14 remplace. |
| D-1 | P2 | UI | text-white/30 sur texte 11px (2 instances, ratio 3.3:1 < WCAG AA). |
| G-1 | P2 | Tech debt | T38 PAD_R=16 ≠ React PAD_X=32. Charts Python et React divergent de 16px. |
| A-1 | P3 | Bridge | reply postMessage avec wildcard "*" au lieu de origin cible. |
| C-2 | P3 | Launcher | libc non declare en dep explicite (transitive via tokio). |
| D-2 | P3 | UI | ~12 instances text-white/40 borderline WCAG AA. |
| F-1 | P3 | Tests | storage_get/set pas testes en dispatch integration. |
| G-2 | P3 | Tech debt | T37-T40 sans SHA dans PATTERNS.md. |

---

## Commits fix attendus

### P1 — Blocking Sprint 14

**C-1 : Stale running.json detection**

Le fix minimal : apres `read_running_info()`, verifier que le PID
est vivant avant de declarer le daemon "already running". Options :
- (a) HTTP health check `GET /health` sur le port lu → le plus fiable
- (b) Verifier le PID via `sysinfo` (deja dans le workspace)
- (c) Tenter de supprimer le fichier stale et spawner

Approche recommandee : (a) HTTP health check — simple, portable,
et prouve que le daemon repond. Si le check echoue, supprimer le
fichier stale et spawner normalement.

Commit cible : `fix(sprint13): detect stale running.json in launcher via health check`

---

## P2 a logger en tech debt

- **B-1** : Sprint 14 remplace le systeme repo_url par deploy verifie.
  Si repo_url subsiste pour le prive, ajouter une validation de format
  URL (`z.string().url()` + whitelist protocoles http/https/git).
- **D-1** : Monter `text-white/30` → `text-white/40` sur les 2
  instances identifiees. Peut etre fait en Sprint 14 Phase A.
- **G-1** : Corriger `_SVG_PAD_R` de 16 a 32 pour aligner sur React.
  Ou documenter la divergence comme un choix de design. ~1 LOC.

---

## P3 laisses sans action

- A-1 : wildcard reply. Risque faible, a considerer en Sprint 14.
- C-2 : libc implicite. Corrigeable par `libc = "0.2"` en Cargo.toml.
- D-2 : text-white/40 borderline. Choix de design valide pour du
  texte secondaire.
- F-1 : storage dispatch tests. A couvrir quand le bridge gagne des
  features.
- G-2 : SHA manquant dans PATTERNS.md. Nit de tracabilite.

---

## Notes on audit completeness

- Playwright non rejoue (necessite env CI avec navigateur). Le
  self-report indique 30 passes — accepte sur la base du CI vert.
- Les fichiers non trackes (cc.json, site/, docs/apps/) sont hors
  scope per audit plan.
- La decision Sprint 14 (deploy verifie from source) n'a pas ete
  auditee (c'est un Day 0 de Sprint 14, pas un finding Sprint 13).
