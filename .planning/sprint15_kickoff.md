# Sprint 15 — Kickoff (Bridge push + CPU watchdog + runtime templates)

**Ecrit** : 2026-04-14
**Tip master d'entree** : `f6015b3` (Sprint 14 audit findings + A-1 fix landed)
**Phase 0 audit** : DONE. Sprint 14 audit CONDITIONAL PASS leve dans
`542479f` (A-1 commit_sha SHA pinning). 8 P2 a logger tech debt dans
ce sprint Phase E. Gate verte.

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-14 **CLOSED** + gate fermee.
- Sprint 14 a livre deploy verifie (Keyoxide + SLSA L1 + PA v4 + badge
  Verifie). Le flow public end-to-end est operationnel : publisher
  pointe vers repo → coordinateur clone + signe provenance → reseau
  distribue → consommateur voit badge Verifie.
- **Manque produit** : les apps dans les iframes ne communiquent qu'en
  mode request/response (iframe demande, coordinateur repond). Il
  n'existe pas de canal host → iframe push, donc impossible pour le
  coordinateur de notifier l'app ("ton task est pret", "un peer a
  update ton storage").
- **Manque UX** : une app malformee ou avec un bug d'infinite-loop
  bloque son iframe. L'utilisateur voit une page figee sans indication
  ni moyen de recharger. Besoin d'un watchdog cote shell.
- **Manque onboarding developpeur** : publier une app requiert
  aujourd'hui (a) forker le `hello-world-app` (b) le modifier (c)
  creer un repo Git (d) appeler deploy-from-repo. Etape (a) + (b)
  n'ont pas d'outil dedie.

### 1.2 Compteurs de tests a l'entree (tip `f6015b3`)

| Suite | Count |
|---|---|
| Rust workspace | 373 |
| Python SDK | 183 (1 flaky Windows pre-existant) |
| Python coordinator | 138 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 193 |
| Playwright | 30 |
| size-limit | 7/7 |
| SPDX | 224/224 |

Total : ~949 tests.

### 1.3 Le probleme

Le bridge Sprint 13 Phase C est **unidirectionnel** (iframe → host).
Le canal inverse manque, ce qui fait que :
- Les tasks async ne peuvent pas notifier l'app quand le resultat
  arrive — l'app doit poller le coordinateur
- Le storage distant ne peut pas notifier de changements — pareil,
  poll
- Aucun signal "tu es en train de freeze" pour l'app

Symetriquement, le shell n'a **aucune visibilite** sur la sante de
l'iframe. Une app qui entre en infinite loop (scripting bug, Pyodide
qui prend 10s, fetch bloque...) laisse l'UI figee sans recours.

Et cote developpeur : le seul exemple est `examples/hello-world-app/`
(2 fichiers). Pas de CLI de scaffold, pas de template React, pas de
Pyodide. L'onboarding est "regarde hello-world et devine".

### 1.4 Vision sprint

Sprint 15 livre le **bridge bidirectionnel** (push events host → iframe),
un **watchdog CPU** (heartbeat-based, dechaine un overlay "recharger/
fermer" quand l'iframe stalle), et le **CLI `sbfb init`** qui scaffold
une app pret-a-publier en une commande. Ces trois briques ferment la
boucle developpeur → publication → runtime sante.

---

## 2. Goal en une phrase

**Le shell peut pousser des events vers les iframes, detecte les apps
stalled via heartbeat, et le CLI `sbfb init` genere un squelette
pret-a-publier en 3 types (html, react, pyodide).**

---

## 3. Phase 0 — Audit Sprint 14

DONE. Verdict CONDITIONAL PASS, 1 P1 fixe (A-1 commit_sha SHA pinning
correctement implemente, commit `542479f`). 8 P2 a logger en Phase E
comme T44..T51. Gate verte. Cf. `sprint14_audit_findings.md`.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Bridge push via event type `sbfb-bridge-event` (fire-and-forget)

**Retenu** : nouveau type de message `sbfb-bridge-event` distinct de
`sbfb-bridge-request` et `sbfb-bridge-response`. Le host pousse via
`iframe.contentWindow.postMessage(event, "*")`. L'iframe SDK expose
`bridge.onEvent(callback)` qui abonne le consommateur. Pas de
correlation ID (fire-and-forget) ; pas d'ACK cote iframe. Le host ne
sait pas si l'event a ete recu — cote UX c'est acceptable (events
sont idempotents ou resynchronisables).

**Rejete** : reutiliser `sbfb-bridge-response` avec un flag
`unsolicited: true` (deviendrait ambigu avec le flow reponse-a-
request existant). Aussi rejete : WebSocket en parallele du bridge
(deuxieme transport = split-brain, complication pour zero gain —
postMessage suffit en intra-browser).

**Implications** :
- `sbfb-bridge.js` +20 LOC : methode `onEvent(cb)`, listener
  supplementaire filtrant `type === "sbfb-bridge-event"`
- `web/src/bridge/protocol.ts` +15 LOC : `BridgeEventSchema` Zod avec
  `{ type: "sbfb-bridge-event", name: string, payload: unknown }`
- `web/src/bridge/useBridge.ts` +15 LOC : expose helper
  `pushEvent(iframe, name, payload)` ; import dans BrowsedProject
- Events MVP livres ce sprint : `heartbeat_timeout_warning` (D2),
  `task_result_ready` (placeholder, pas implemente cote dispatcher),
  `storage_changed` (placeholder). Le schema accepte n'importe quel
  event name — la whitelist n'est pas la priorite du sprint (les
  apps decident quoi consommer via callback)

### D2 — CPU watchdog via heartbeat iframe (1s) + timeout host (5s)

**Retenu** : chaque iframe qui inclut `sbfb-bridge.js` demarre
automatiquement un heartbeat au mount (`bridge._startHeartbeat(1000)`
appele dans le constructor, intervalle 1s). Le heartbeat est un
postMessage `{ type: "sbfb-bridge-heartbeat", ts: Date.now() }`. Cote
host, `useBridge` trace le dernier heartbeat recu via un ref. Un
timer de fond check toutes les 2s : si `now - lastHeartbeat > 5000`,
state `stalled`. Le composant parent (BrowsedProject) affiche un
overlay conditionnel "Application ne repond plus" avec 2 boutons :
"Recharger" (reset `iframe.src` au URL original) et "Fermer" (retour
au Browse).

**Rejete** : monitorer CPU cote host (impossible cross-origin) ;
tuer l'iframe par force (pas d'API browser sans `iframe.remove()`,
qui necessite recreer le DOM node et perdre le state) ; CSP avec
`worker-src` pour cloisonner la charge (hors scope, CSP est deja
strict). Aussi rejete : timeout plus court (2-3s) qui provoquerait
des faux positifs sur un device mobile lent.

**Implications** :
- `sbfb-bridge.js` +30 LOC : `_startHeartbeat(intervalMs)`,
  `_stopHeartbeat()`, appele dans constructor / destroy
- `web/src/bridge/protocol.ts` +10 LOC : `BridgeHeartbeatSchema`
- `web/src/bridge/useBridge.ts` +50 LOC : state machine
  `{ healthy | stalled | unknown }`, timer 2s, retourne un hook state
- `web/src/pages/BrowsedProject.tsx` +40 LOC : overlay conditionnel
  + 2 boutons, fonction `reloadIframe()` qui reset src
- Valeurs par defaut : HEARTBEAT_INTERVAL = 1000 ms,
  STALL_THRESHOLD = 5000 ms, CHECK_INTERVAL = 2000 ms. Configurables
  via env mais pas expose dans l'UI sprint 15

### D3 — CLI `sbfb init <type> <path>` via typer + 3 templates

**Retenu** : nouveau CLI Python `sbfb` avec sous-commande `init`.
Structure : `sbfb init <type> <path>` avec `<type>` = `html` | `react`
| `pyodide` et `<path>` le repertoire cible. Le CLI copie le template
depuis `packages/nexus-coordinator/templates/<type>/` vers `<path>`,
remplace les placeholders `{{NODE_ID}}` (recupere depuis le daemon
running.json local, ou laisse le placeholder si pas de daemon) et
`{{PROJECT_NAME}}` (dernier segment du path). Chaque template contient
: `index.html`, `SBFB.json` (avec node_id), `README.md` (instructions),
`.gitignore`.

Template `html` : 1 page statique avec sbfb-bridge.js CDN, bouton
"submit task" qui demo les 3 APIs.
Template `react` : structure vite minimale (package.json, vite.config,
src/App.tsx avec sbfb-bridge.js integration), build produit `dist/`.
Template `pyodide` : page HTML qui charge Pyodide depuis CDN,
execute du Python inline, exemple qui utilise `bridge.submitTask` pour
offload du compute.

**Rejete** : scaffold via UI frontend (pas d'acces filesystem dans le
browser, necessiterait une API cote coordinateur — hors scope).
Aussi rejete : fetch templates depuis GitHub en live (ajoute une
dependance reseau au simple `sbfb init`, plus lent, casse si offline).

**Implications** :
- Nouveau dossier `packages/nexus-coordinator/templates/`
  contenant 3 sous-dossiers `html/`, `react/`, `pyodide/` avec les
  fichiers de chaque template
- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/cli/init.py`
  (~150 LOC) : logique typer, copie arborescence, substitution
  placeholders
- `packages/nexus-coordinator/src/nexus_coordinator/cli/__init__.py`
  avec entry point `main` qui expose les sous-commandes
- `pyproject.toml` : ajouter `[project.scripts] sbfb =
  "nexus_coordinator.cli:main"`
- Les templates sont inclus dans le wheel via `package_data`
- Tests pytest : 8+ tests validant chaque type + placeholder
  substitution + erreur-paths (path existe, type inconnu)

### D4 — Tests Playwright real iframe + watchdog

**Retenu** : 2 nouveaux tests Playwright dans `web/tests/` :
- `bridge-push.spec.ts` : charge une iframe depuis une fixture
  locale qui ecoute `bridge.onEvent` et affiche le payload ; le test
  push un event via le hook ; asserte le payload apparait dans le DOM
  de l'iframe
- `watchdog-stalled.spec.ts` : charge une iframe qui envoie des
  heartbeats, puis le script stoppe l'interval (`clearInterval`) pour
  simuler un freeze ; le test attend 6s ; asserte l'overlay "ne repond
  plus" est visible et clique "Recharger" ; asserte que l'overlay
  disparait apres reload

Fixtures : repertoire `web/tests/fixtures/bridge-sample/` contenant un
`index.html` avec `sbfb-bridge.js`. Pour servir l'iframe, on monte la
fixture dans le blob-serve daemon en prepublish (si daemon tourne)
**OU** on sert la fixture via un petit HTTP server side-car dans le
test setup — la 2e option est preferee car independante du daemon.

**Rejete** : tester via un mock de `window.postMessage` (ne couvre
pas le cas cross-origin reel entre shell et blob-serve) ; depender
du daemon tournant en CI (fragile — le daemon nexus-shell-daemon n'est
pas encore dans le CI matrix du projet).

**Implications** :
- `web/tests/fixtures/bridge-sample/` (3 fichiers : index.html avec
  heartbeat, event listener, bridge.js stub)
- `web/tests/bridge-push.spec.ts` (~80 LOC)
- `web/tests/watchdog-stalled.spec.ts` (~100 LOC)
- Playwright config : ajouter un side-car HTTP server dans
  `webServer` ou utiliser `page.route` pour servir les fixtures

### D5 — P2 Sprint 14 audit → tech debt PATTERNS.md

**Retenu** : logger les 8 P2 identifies dans `sprint14_audit_findings.md`
(A-2 a G-2) en section "Sprint 14 audit tech debt" dans
`docs/shell/PATTERNS.md`, numerotes T44..T51. Pas de code fix dans
ce sprint (les 8 items sont tous non-bloquants ; certains necessitent
design sprint-level que Sprint 15 n'embarque pas).

**Rejete** : attaquer les P2 dans ce sprint. Le scope CPU watchdog +
bridge push + templates est deja dense ; diluer sur 8 items low-value
retarderait les features produit.

**Implications** :
- `docs/shell/PATTERNS.md` +30 LOC : 8 items T44..T51 avec titre,
  localisation, rationale, fix suggere

---

## 5. Plan Phase outline

### Phase A — Bridge push bidirectionnel (event channel)

**Scope** :
- `web/public/sbfb-bridge.js` : methode `onEvent(cb)`, listener
  filtrant `type === "sbfb-bridge-event"`
- `web/src/bridge/protocol.ts` : `BridgeEventSchema` Zod
- `web/src/bridge/useBridge.ts` : helper `pushEvent(iframe, name,
  payload)` expose via return du hook
- Tests Vitest : bridge receives event + host push triggers callback

**Critere** : appel `pushEvent(iframe, "foo", {x: 1})` declenche le
callback `onEvent` dans l'iframe avec payload correct. Zod rejette
les events mal formes.

**Commit** : `feat(bridge): Sprint 15 Phase A — bidirectional push via sbfb-bridge-event`

### Phase B — CPU watchdog iframe (heartbeat + overlay stalled)

**Scope** :
- `web/public/sbfb-bridge.js` : `_startHeartbeat(intervalMs)`,
  `_stopHeartbeat()`, auto-demarrage au constructor
- `web/src/bridge/protocol.ts` : `BridgeHeartbeatSchema`
- `web/src/bridge/useBridge.ts` : state machine avec
  `lastHeartbeatRef`, timer 2s, return `{ state: "healthy"|"stalled"|
  "unknown" }`
- `web/src/pages/BrowsedProject.tsx` : overlay conditionnel + 2 boutons
  + fonction `reloadIframe()` qui force `iframe.src = blobServeUrl(...)`
- Tests Vitest : useBridge transitions state after stall ; overlay
  render conditional

**Critere** : une iframe qui stoppe ses heartbeats voit l'overlay
"ne repond plus" apparaitre en 5-7s. Clic "Recharger" reset l'iframe
et l'overlay disparait.

**Commit** : `feat(watchdog): Sprint 15 Phase B — CPU watchdog via heartbeat + stalled overlay`

### Phase C — Runtime templates CLI `sbfb init`

**Scope** :
- `packages/nexus-coordinator/templates/html/` : index.html + SBFB.json
  + README + .gitignore
- `packages/nexus-coordinator/templates/react/` : package.json +
  vite.config.ts + src/App.tsx + index.html + SBFB.json + README
- `packages/nexus-coordinator/templates/pyodide/` : index.html avec
  Pyodide CDN + SBFB.json + README
- `packages/nexus-coordinator/src/nexus_coordinator/cli/__init__.py` :
  typer main app
- `packages/nexus-coordinator/src/nexus_coordinator/cli/init.py` :
  sous-commande `init` avec logique copy + substitution
- `pyproject.toml` : entry point `[project.scripts] sbfb`
- `MANIFEST.in` ou setup-tools include_package_data : inclure
  `templates/**` dans le wheel
- Tests pytest `test_cli_init.py` : 3 types + path exists + unknown
  type + placeholder substitution + SBFB.json valid

**Critere** : `sbfb init html /tmp/my-app` cree `/tmp/my-app/` avec
les 4 fichiers, SBFB.json contient `node_id` correct (depuis daemon
running.json local OU placeholder si daemon absent), README lisible.
Idem react + pyodide.

**Commit** : `feat(cli): Sprint 15 Phase C — sbfb init CLI with html/react/pyodide templates`

### Phase D — Tests Playwright iframe reel

**Scope** :
- `web/tests/fixtures/bridge-sample/` : fixture HTML + bridge.js
  embarque pour le test
- `web/tests/bridge-push.spec.ts` : test push event E2E
- `web/tests/watchdog-stalled.spec.ts` : test watchdog E2E
- Playwright `webServer` ajustement pour servir fixtures
  (probable : `playwright.config.ts` ajoute un static server
  side-car)

**Critere** : les 2 specs verts en local. Skip si fixtures/daemon
indisponibles.

**Commit** : `test(bridge): Sprint 15 Phase D — Playwright iframe push + watchdog stalled`

### Phase E — Docs (verification + audit plan + PATTERNS T44-T51)

**Scope** :
- `.planning/sprint15_verification.md` : fail-fast checklist remplie
- `.planning/sprint15_audit_plan.md` : plan d'audit pour Sprint 16
  Phase 0
- `docs/shell/PATTERNS.md` : section "Sprint 14 audit tech debt"
  avec T44..T51
- `docs/rust/PATTERNS.md` : si applicable (Phase A/B touchent
  frontend uniquement, probable pas de Rust update)

**Commit** : `docs(sprint15): verification + audit plan for Sprint 16`

---

## 6. Scope cuts (PAS dans ce sprint)

- Re-publish automatique sur repo update → Sprint 16 (webhook ou
  polling, necessite design de gestion Etag/SHA)
- Branding SBFB (nom produit, logo, favicon) → Sprint 16 (design
  produit autonome)
- Origin separee par subdomain blob-serve → Sprint 16+ (necessite
  DNS wildcard, ops work)
- 2 VPS supplementaires (US/Asia) → Sprint 16 (infra ops)
- MIME scan executables dans le zip → Sprint 16 (P2 Sprint 14, la
  sandbox CSP suffit pour MVP)
- Builds reproductibles (hash comparison cross-nodes) → v1.2+
- Multi-writer iroh-docs → v1.1+
- Custom domain / DNS → v1.2+
- Dispatcher de notifications push server-side (events en provenance
  du reseau P2P propages via gossip vers le host React puis vers
  iframe) → Sprint 16+. Ce sprint livre le **canal** (host → iframe).
  Cote "qui produit les events" reste manuel (developer triggers).
- Whitelist stricte des event names cote host → Sprint 16+. MVP
  accepte n'importe quel name ; les apps filtrent via `onEvent`
  callback
- Templates additionnels (Vue, Svelte, Jupyter) → Sprint 16+
  (3 templates MVP suffisent pour demontrer le pattern)
- `sbfb publish --type <X>` integre au CLI → Sprint 16+ (init
  scaffold suffit, publish via API coord existant ou deploy-from-repo)
- Watchdog kill-by-force iframe (sans reload) → Sprint 16+ (browser
  API ne supporte pas, on reste sur reload)

## 7. Tracabilite scope (items differes des sprints precedents)

| Item | Origine | Sprint 15 |
|---|---|---|
| CPU watchdog iframe | Sprint 13 D6, Sprint 14 scope cut | **Phase B** |
| Bridge push bidirectionnel | Sprint 13 roadmap, Sprint 14 scope cut | **Phase A** |
| Runtime templates `sbfb init` | Sprint 12/13 scope cut, Sprint 14 scope cut | **Phase C** |
| Re-publish auto | Sprint 12/13/14 scope cut | Differe Sprint 16 |
| Branding SBFB | Sprint 10/12/13/14 scope cut | Differe Sprint 16 |
| Origin subdomain | Sprint 12/13/14 scope cut | Differe Sprint 16+ |
| VPS US/Asia | Sprint 12/13/14 scope cut | Differe Sprint 16 |
| MIME scan | Sprint 14 scope cut | Differe Sprint 16 |
| T44..T51 P2 Sprint 14 | Sprint 14 audit findings | **Phase E** (log only) |

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 14 jouee et fermee (CONDITIONAL PASS leve dans
`542479f`). Phase E de ce sprint produira `sprint15_audit_plan.md`
pour que Sprint 16 Phase 0 audite independamment.

---

## 9. Estimations LOC

| Phase | LOC estimee | Repartition |
|---|---|---|
| A — Bridge push | ~250 | 20 bridge.js + 15 protocol + 15 useBridge + 200 tests |
| B — CPU watchdog | ~400 | 30 bridge.js + 10 protocol + 50 useBridge + 40 BrowsedProject + 270 tests |
| C — Templates CLI | ~700 | 150 cli/init.py + 300 templates (3 types) + 250 tests |
| D — Playwright | ~250 | 50 fixtures + 100 bridge-push + 100 watchdog-stalled |
| E — Docs | ~400 | verification + audit plan + PATTERNS T44-T51 |
| **Total** | **~2000** | |

---

## 10. Checkpoint de validation

Avant de passer au plan detaille, confirmer :

1. D1 (bridge push via event type `sbfb-bridge-event`, fire-and-forget) est valide
2. D2 (watchdog par heartbeat 1s / timeout 5s + overlay reload) est valide
3. D3 (CLI `sbfb init <type>` avec 3 templates html/react/pyodide) est valide
4. D4 (2 tests Playwright E2E avec fixtures locales) est valide
5. D5 (P2 Sprint 14 logges T44..T51, pas de code fix ce sprint) est valide
6. L'ordre des phases (A bridge → B watchdog → C templates → D Playwright → E docs) est OK
7. Les scope cuts (re-publish, branding, VPS, dispatcher events → Sprint 16+) sont acceptes
