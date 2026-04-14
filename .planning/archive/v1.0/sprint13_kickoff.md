# Sprint 13 — Kickoff (Bridge postMessage + open source enforcement + UI Netflix + launcher)

**Ecrit** : 2026-04-13
**Tip master d'entree** : `53a9e32` (Sprint 12 P1 fix deploy size limit)
**Phase 0 audit** : DONE. Sprint 12 audit CONDITIONAL PASS leve
dans `53a9e32` (1 P1 fixe : MAX_DEPLOY_BYTES=100MB). Gate verte.

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-12 **CLOSED**. v1.0.0 released + rendu universel cross-node.
- Le flow P2P **publication → distribution → rendu** fonctionne de
  bout en bout : zip + gossip + blob-serve + iframe isolee
- Le shell React est un **iframe host** agnostique de la techno des apps
- **Manque critique** : les apps dans les iframes n'ont aucun moyen
  de communiquer avec le reseau (pas de bridge, pas d'API). Elles
  sont isolees et statiques.
- **Manque produit** : aucune contrainte d'ouverture sur les apps
  publiques — le reseau pourrait etre squatte par du closed-source
- **Manque UX** : l'UI reste fonctionnelle mais generique (shadcn
  stock), pas de branding visuel fort
- **Manque desktop** : pas de launcher — l'utilisateur doit demarrer
  le daemon manuellement en CLI
- 4 items tech debt OPEN : T37-T40 (Sprint 12 audit)
- Changements non commites : UI Netflix glassmorphism partiellement
  appliquee (AppShell + Browse + BrowsedProject + index.css),
  7 Vitest BrowsedProject cassent (structure HTML modifiee)

### 1.2 Compteurs de tests a l'entree (tip `53a9e32`)

| Suite | Count |
|---|---|
| Rust workspace | 362 |
| Python SDK | 182 |
| Python coordinator | 96 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 180 (7 FAIL sur uncommitted) |
| Playwright | 30 |
| size-limit | 7/7 |
| SPDX | 215/215 |

### 1.3 Le probleme

1. **Les apps sont muettes** : une app React dans l'iframe ne peut
   pas soumettre de taches, lire des donnees, ou interagir avec le
   reseau. C'est une page statique dans un cadre — pas une app P2P.
2. **Aucune garantie d'ouverture** : n'importe qui publie du
   closed-source sur le reseau public. Les utilisateurs executent du
   code non-auditable dans leur navigateur.
3. **Pas de launcher** : demarrer SBFB necessite un terminal + CLI.
   Barriere d'entree trop haute pour un utilisateur non-dev.
4. **UI stock** : l'identite visuelle n'existe pas encore.

### 1.4 Vision sprint

Sprint 13 transforme SBFB d'une plateforme de rendu passif en une
plateforme d'apps P2P interactives avec garantie d'ouverture sur
le reseau public.

---

## 2. Goal en une phrase

**Les apps dans les iframes communiquent avec le reseau via un
bridge postMessage securise, les apps publiques sont obligatoirement
open source, l'UI adopte le design Netflix glassmorphism, et un
launcher Rust permet de demarrer SBFB en double-clic.**

---

## 3. Phase 0 — Audit Sprint 12

DONE. Verdict CONDITIONAL PASS, 1 P1 fixe (MAX_DEPLOY_BYTES=100MB),
4 P2 logges T37-T40. Gate verte. Cf. `sprint12_audit_findings.md`.

---

## 4. Decisions Day 0 (D1..D6 gelees)

### D1 — Public = open source, prive = libre

**Retenu** : toute app publiee avec visibilite `public` doit fournir
un `repo_url` (lien vers un depot de code source public). C'est une
regle du protocole de publication, pas de la moderation. Une app
privee (invitation only) n'a aucune contrainte.

**Raison** : dans un reseau P2P, le code est execute par des inconnus
dans leur navigateur. Le droit d'auditer ce qu'on execute est un
principe de securite fondamental, pas un choix ideologique. Ca empeche
la capture du reseau par des entites qui distribueraient du
closed-source gratuit.

**Rejete** : tout permettre sans contrainte. Risque de capture
corporate du reseau public (modele Amazon/Linux). Aussi rejete :
obligation GitHub specifique — `repo_url` accepte tout depot public
(GitHub, GitLab, Codeberg, self-hosted).

**Implications** :
- `ProjectAnnouncement` Rust gagne `repo_url: Option<String>`
- Le coordinator valide `repo_url` present quand visibility=public
  au moment du publish
- `BrowseEntry` Rust + Zod TypeScript gagnent `repo_url`
- Le shell affiche un lien cliquable vers le repo sur chaque entry
  publique

### D2 — UI Netflix glassmorphism dark-first

**Retenu** : le design glassmorphism (backdrop-blur, bg-opacity,
glass-card, glass-pill) deja applique dans les changements non
commites sur AppShell/Browse/BrowsedProject est la base. Sprint 13
l'etend aux pages restantes (Projects, ProjectDetail, Network,
Curators) et corrige les tests casses.

**Rejete** : garder l'UI shadcn stock. Le reseau a besoin d'une
identite visuelle forte pour la demo et le lancement.

**Implications** :
- Commit des changements non commites comme base de Phase A
- Fix des 7 Vitest BrowsedProject
- Extension glassmorphism aux 4 pages restantes
- Pas de light mode (toujours dark)

### D3 — postMessage bridge MVP (request/response + correlation IDs)

**Retenu** : le bridge utilise `window.postMessage()` entre l'iframe
et le host shell. Protocole request/response avec correlation IDs
(UUID) pour supporter les appels async. Le host forward les requetes
vers le coordinator via les endpoints REST existants.

**Rejete** : Service Worker interceptor (impossible avec
`sandbox="allow-scripts"` sans `allow-same-origin`, spec W3C).
Rejete aussi : WebSocket bidirectionnel (overengineered pour le MVP,
ajoute un serveur WS).

**Compatibilite sandbox** : `postMessage` fonctionne avec
`sandbox="allow-scripts"` sans `allow-same-origin`. Le CSP
`connect-src 'none'` ne bloque pas postMessage (spec CSP : connect-src
controle fetch/XHR/WS/EventSource, pas le HTML Messaging API).

**Implications** :
- Host listener dans le shell React (useEffect + window.addEventListener)
- Schema Zod pour les messages bridge (type, correlation_id, payload)
- SDK bridge client (fichier JS que les apps importent pour
  communiquer avec le host)
- Forward `task_submit` et `storage_read/write` vers coordinator API
- Reponses async avec timeout (10s default)

### D4 — Launcher Rust minimal (pas Tauri)

**Retenu** : un binary Rust minimaliste (`nexus-launcher`) qui :
1. Spawn `nexus-shell-daemon start` comme child process
2. Lit `running.json` pour recuperer le port
3. Ouvre le navigateur par defaut via la crate `open`
4. Attend Ctrl+C, forward le signal au daemon

**Rejete** : Tauri. Le modele webview Tauri est un "weak link" pour
le contenu untrusted (CVE-2024-35222, pas de sandbox a la hauteur
d'un navigateur). Decision prise en session 2026-04-13.

**Implications** :
- Nouveau crate `crates/nexus-launcher/` (binary, ~300 LOC)
- Dep : `open` (crate cross-platform pour ouvrir un URL)
- Pas de fenetre native — le navigateur est le client
- Graceful shutdown : Ctrl+C → SIGTERM child → wait → exit

### D5 — Tech debt T37-T40 fermes

**Retenu** : les 4 items P2 logges par l'audit Sprint 12 sont
fermes dans ce sprint.

- T37 : CSP middleware tower pour toutes les reponses blob-serve (~25 LOC)
- T38 : Aligner dimensions SVG charts html_render.py sur React (~35 LOC)
- T39 : Test file_upload block dans test_html_render.py (~20 LOC)
- T40 : X-Real-IP header dans /blob-serve/ nginx (~1 LOC)

### D6 — CPU watchdog differe Sprint 14

**Retenu** : le CPU watchdog pour les iframes est differe a Sprint 14.
La combinaison `sandbox="allow-scripts"` + CSP `connect-src 'none'`
+ open source obligatoire sur le public offre une securite suffisante
pour le MVP. Le bridge postMessage (D3) ouvre la voie au watchdog
en Sprint 14 (heartbeat via bridge).

**Rejete pour ce sprint** : implementer le watchdog maintenant. Le
bridge n'est pas encore mature, et le pattern heartbeat necessite
du retour d'experience utilisateur.

---

## 5. Plan Phase outline

### Phase A — UI Netflix glassmorphism + tech debt T37-T40

**Scope** :
- Commit les changements non commites (AppShell + Browse +
  BrowsedProject + index.css glassmorphism)
- Fix les 7 tests Vitest BrowsedProject casses
- Etendre glassmorphism a Projects, ProjectDetail, Network, Curators
- Fermer T37 (CSP middleware blob-serve)
- Fermer T38 (SVG chart dimensions)
- Fermer T39 (test file_upload)
- Fermer T40 (nginx X-Real-IP)

**Critere** : 180+ Vitest verts (0 fail), T37-T40 CLOSED, toutes
les pages ont le design glassmorphism.

**Commit** : `feat(web): Sprint 13 Phase A — UI Netflix glassmorphism + T37-T40`

### Phase B — Open source enforcement

**Scope** :
- `ProjectAnnouncement` : ajouter `repo_url: Option<String>` (Rust)
- `BrowseEntry` : ajouter `repo_url: Option<String>` (Rust)
- Coordinator : valider `repo_url` requis quand visibility=public
  dans `POST /publish` et `POST /project/deploy`
- Frontend : lien cliquable vers le repo dans Browse cards +
  BrowsedProject sidebar
- Zod schema update + backward compat (.optional())
- Tests Rust + Python + Vitest

**Critere** : un publish public sans `repo_url` → erreur 400. Un
publish prive sans `repo_url` → OK. Lien visible dans le shell.

**Commit** : `feat(p2p): Sprint 13 Phase B — open source enforcement for public apps`

### Phase C — postMessage bridge MVP

**Scope** :
- Host bridge listener dans le shell (`useBridge` hook)
- Schema Zod des messages bridge (request/response/error)
- Forward `task_submit` : iframe → host → `POST /app/{name}/tasks/submit`
- Forward `storage_get/set` : iframe → host → coordinator storage API
- SDK bridge client (`sbfb-bridge.js`, ~150 LOC, importable par
  les apps)
- Correlation IDs + timeout 10s + error handling
- Tests unitaires host listener + SDK client

**Critere** : une app dans l'iframe peut soumettre une tache via
`bridge.submitTask({...})` et recevoir la reponse.

**Commit** : `feat(bridge): Sprint 13 Phase C — postMessage bridge MVP with task submit + storage`

### Phase D — Rust launcher minimal

**Scope** :
- Nouveau crate `crates/nexus-launcher/` (binary)
- `main.rs` : parse args, spawn daemon, read running.json, open
  browser, wait Ctrl+C, graceful shutdown
- Dep `open` pour ouvrir le navigateur
- SPDX header
- Tests : process spawn + running.json read

**Critere** : `cargo run -p nexus-launcher` demarre le daemon et
ouvre le navigateur. Ctrl+C arrete proprement.

**Commit** : `feat(launcher): Sprint 13 Phase D — minimal Rust launcher with browser open`

### Phase E — Docs (verification + audit plan)

**Scope** :
- `sprint13_verification.md` avec checklist fail-fast remplie
- `sprint13_audit_plan.md` pour Sprint 14 Phase 0
- Update `docs/shell/PATTERNS.md` (T37-T40 CLOSED, nouveaux patterns)

**Commit** : `docs(sprint13): verification + audit plan for Sprint 14`

---

## 6. Scope cuts (PAS dans ce sprint)

- CPU watchdog iframe → Sprint 14 (D6, apres retour experience bridge)
- Branding SBFB (nom, logo, favicon) → Sprint 14
- Runtime templates (`sbfb publish --type python`) → Sprint 14
- Re-publish automatique → Sprint 14
- Origin separee par subdomain blob-serve → Sprint 14+
- Multi-writer iroh-docs → v1.1+
- Custom domain / DNS → Sprint 14+
- Verification automatique que le repo est public (API GitHub) →
  Sprint 14 (Sprint 13 fait confiance au `repo_url` fourni)
- 2 VPS supplementaires (US/Asia) → Sprint 14

## 7. Tracabilite scope (items differes des sprints precedents)

| Item | Origine | Sprint 13 |
|---|---|---|
| Branding SBFB | Sprint 10, 12 scope cut | Differe Sprint 14 |
| Runtime templates | Sprint 12 scope cut | Differe Sprint 14 |
| Re-publish auto | Sprint 12 scope cut | Differe Sprint 14 |
| Origin subdomain | Sprint 12 scope cut | Differe Sprint 14+ |
| VPS US/Asia | Sprint 12 scope cut | Differe Sprint 14 |
| T37-T40 tech debt | Sprint 12 audit | **Phase A** |
| Bridge postMessage | Roadmap Sprint 13 | **Phase C** |
| Launcher Rust | Roadmap Sprint 13 | **Phase D** |
| UI Netflix | Session 2026-04-13 | **Phase A** |
| Open source enforcement | Decision 2026-04-13 | **Phase B** |

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 12 jouee et fermee (CONDITIONAL PASS leve).
Phase E de ce sprint produira `sprint13_audit_plan.md` pour que
Sprint 14 Phase 0 audite independamment.

---

## 9. Estimations LOC

| Phase | LOC estimee | Repartition |
|---|---|---|
| A — UI + T37-T40 | ~400 | 250 uncommitted + 80 extension + 70 tech debt |
| B — Open source | ~200 | 60 Rust + 40 Python + 50 TS + 50 tests |
| C — Bridge | ~550 | 150 host + 100 schema + 150 SDK + 150 tests |
| D — Launcher | ~300 | 200 main + 50 tests + 50 infra |
| E — Docs | ~300 | verification + audit plan + PATTERNS |
| **Total** | **~1750** | |

---

## 10. Checkpoint de validation

Avant de passer au plan detaille, confirmer :

1. D1 (public = open source, `repo_url` obligatoire) est valide
2. D2 (glassmorphism base = les changements non commites) est valide
3. D3 (postMessage bridge, pas Service Worker) est valide
4. D4 (launcher Rust minimal, pas Tauri) est valide
5. D5 (T37-T40 en Phase A) est valide
6. D6 (CPU watchdog differe Sprint 14) est accepte
7. L'ordre des phases (A UI → B open source → C bridge → D launcher → E docs) est OK
