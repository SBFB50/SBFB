# Sprint 15 — Audit findings (Sprint 16 Phase 0 gate)

**Auditeur** : Claude Code session fraiche (Sprint 16 Phase 0)
**Date** : 2026-04-14
**Tip audite** : `4da0043` (docs Sprint 15 verification + audit plan)
**Commit stack Sprint 15** :
- `b2940b3` Phase A — bidirectional push via sbfb-bridge-event
- `3c729ba` Phase B — CPU watchdog via heartbeat + stalled overlay
- `e6644be` Phase C — sbfb init CLI with html/react/pyodide templates
- `f5aea3e` Phase D — Playwright iframe push + heartbeat E2E
- `4da0043` Phase E (docs) — verification + audit plan for Sprint 16

**Timebox observe** : ~1h15.

---

## Verdict global : **PASS**

- **P0** : 0
- **P1** : 0
- **P2** : 1 (timing-sensible Playwright, confirme vert)
- **P3** : 4 (nits, laisses sans action)

Sprint 16 Phase A peut demarrer apres la migration PARA staged
(voir §Hors-scope audit) sans fix(sprint15) bloquant.

---

## Track A — Bridge push protocol — **CLEAN**

`BridgeEventSchema` borne correctement `name` a `z.string().min(1).max(64)`
et accepte `payload: z.unknown()`. Le SDK `sbfb-bridge.js` utilise un
`Set<callback>` par name (dedup automatique, O(1) unsubscribe via le
closure retourne). `destroy()` appelle `_eventHandlers.clear()`. Les
callback errors sont swallowed avec `console.error` dans le handler
loop sans casser les autres callbacks ni le bridge. `pushEvent` dans
`useBridge` est no-op safe quand `iframeRef.current` ou `contentWindow`
est `null`.

5 tests Vitest protocol (accept valid, null payload, reject empty name,
reject >64 chars, reject wrong type) + 4 tests pushEvent (post ok, null
iframeRef no-op, null contentWindow no-op, arbitrary payload types).

### A-1 (P3) — onEvent unsubscribe function non testee directement

**Localisation** : `web/public/sbfb-bridge.js:150-154`

Le retour de `bridge.onEvent(name, cb)` est une fonction qui fait
`current.delete(callback)`. Aucun test Vitest ni Playwright n'asserte
que l'invocation de cette fonction empeche les callbacks suivants. Le
comportement est correct par construction (Set.delete) mais non couvert.

**Action** : aucune. Nit de couverture, peut etre ajoute dans un sprint
futur en meme temps qu'un audit complet de la surface bridge.

### A-2 (P3) — Multi-callback same-name non teste

**Localisation** : `web/public/sbfb-bridge.js:145-150`

`onEvent("foo", cb1); onEvent("foo", cb2)` ajoute les 2 callbacks au
Set et devrait appeler les 2 a chaque event. Aucun test direct. Le
comportement est correct par construction mais non verifie.

**Action** : aucune. Meme rationale que A-1.

### A-3 (P3) — Callback throw comportement non teste directement

**Localisation** : `web/public/sbfb-bridge.js:59-66`

Un callback qui throw est attrape par le `try/catch` du handler loop
et logge via `console.error` ; les callbacks suivants continuent. Le
code est defensif mais ce scenario n'a pas de test Vitest explicite.

**Action** : aucune. P3 pur.

---

## Track B — Watchdog state machine — **CLEAN**

State machine `unknown` / `healthy` / `stalled` implementee avec :
- `lastHeartbeatRef` ref tracking le timestamp du dernier heartbeat
- Source validation stricte (`event.source !== iframe.contentWindow`
  → return) sur le heartbeat handler
- Zod schema `z.number().positive()` rejette `ts: 0` et `ts: -1`
- `setInterval` 2s separe du message handler : detecte staleness
  meme si l'iframe ne poste plus rien
- `resetWatchdog` remet `lastHeartbeatRef = null` + state a `"unknown"`,
  ce qui empeche la transition immediate vers `"stalled"` (guard
  `if (last === null) return`)
- Overlay JSX dans `BrowsedProject.tsx:315` strictement conditionne
  sur `watchdogState === "stalled"` + `hasArchive` (pas de flash au
  mount en state `unknown`)
- `reloadIframe` guard : early return si `!frame || !daemonInfo ||
  !entry.archive_hash`

8 tests state machine (unknown at mount, healthy sur 1er heartbeat,
stalled apres timeout, recovery stalled→healthy, ignore unknown source,
reset→unknown, reset ne re-transition pas) + 3 schema heartbeat + 1
BrowsedProject overlay unknown. Couverture complete des transitions
listees dans le kickoff D2.

### B-1 (P3 nit) — setWatchdogState updater form redondante

**Localisation** : `web/src/bridge/useBridge.ts:117,162`

Pattern `setWatchdogState((prev) => (prev === "healthy" ? prev : "healthy"))`
est equivalent a `setWatchdogState("healthy")` en React 18 (bail-out
via Object.is). Le form updater est legerement plus couteux (closure).

**Action** : aucune. Code lisible et correct.

---

## Track C — CLI scaffold + templates — **CLEAN**

`_substitute` couvre exactement 2 placeholders declares (`{{NODE_ID}}`,
`{{PROJECT_NAME}}`). `_read_local_node_id` est fully-defensive :
`try/except (json.JSONDecodeError, OSError)` + verification
`isinstance(node_id, str) and node_id`. Le template pyodide charge
`./pyodide/pyodide.js` en relatif (pas de CDN) et le README explique
explicitement la contrainte CSP blob-serve `connect-src 'none'` avec
instructions de download local (`npm install pyodide@0.29.3` ou
`curl` direct GitHub releases). `sbfb_main.py` a le callback vide qui
force Typer en multi-command mode. `pyproject.toml` declare bien
`[project.scripts] sbfb = ...sbfb_main:app` et
`[tool.hatch.build.targets.wheel.force-include]` route
`src/nexus_coordinator/templates` vers `nexus_coordinator/templates`
dans le wheel.

15 tests pytest couvrent : happy paths 3 types (html 4 / react 3 /
pyodide 2), fallback malformed JSON, rejected unknown template
"svelte", rejected existing destination, placeholder integrity (aucun
`{{...}}` qui fuit quand daemon tourne).

### C-1 (P3 nit) — `.gitignore` dans `_TEXT_SUFFIXES` est code mort

**Localisation** : `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/scaffold.py:90-103`

L'entree `".gitignore"` dans le set `_TEXT_SUFFIXES` n'est jamais
matchee car `Path(".gitignore").suffix == ""` (pas `".gitignore"`).
Le fallback `item.name.startswith(".")` ligne 119 attrape bien le
fichier, donc le comportement est correct. L'entree est redondante
mais inoffensive.

**Action** : aucune. Peut etre nettoye dans un sprint futur en meme
temps qu'un refactor du set suffixes.

### C-2 (P3 nit) — TOCTOU theorique entre `path.exists()` et `path.mkdir()`

**Localisation** : `packages/nexus-coordinator/src/nexus_coordinator/cli/commands/scaffold.py:147-153`

`if path.exists(): raise ...` puis `path.mkdir(parents=True)` sans
`exist_ok`. Un autre process qui cree le dossier entre les 2 appels
ferait lever `FileExistsError` au lieu du `BadParameter` pretty.
Scenario improbable (scaffolding local user-invoked) et l'erreur
serait bruyante mais non-corruptive.

**Action** : aucune. Acceptable pour un CLI dev-tool.

---

## Track D — Playwright iframe E2E — **CLEAN**

Les 3 tests Playwright utilisent le VRAI fichier `web/public/sbfb-bridge.js`
via `readFile(resolve(process.cwd(), "public/sbfb-bridge.js"), "utf-8")`
inline dans l'iframe HTML — PAS un stub. Le replace `<\/script>` a un
commentaire explicite pointant le probleme du parser HTML qui ferme
`<script>` au premier `</script>` vu (meme dans des commentaires JS —
la JSDoc `@example` de bridge.js contient un `</script>`). Le pattern
echo (iframe postMessage vers parent avec `type: "iframe-echo"`) est
le seul moyen sans `contentDocument` (sandbox sans `allow-same-origin`
bloque l'acces). `page.route("**/bridge-test/iframe")` remplace un
webServer side-car (risque R2 elude). `heartbeatInterval: 200ms`
(push-event : 0 pour desactiver) accelere sans denaturer. Chaque test
a sa propre `page` Playwright donc isolation `window.__echoes` /
`__heartbeats` automatique.

### D-1 (P2) — Delta heartbeat asserted dans `[50, 1500]ms` — marge large

**Localisation** : `web/tests/bridge-heartbeat.spec.ts:85-87`

Le test asserte `delta > 50 && delta < 1500` pour 2 heartbeats a 200ms
d'intervalle. Marge haute (7.5x) tolerante mais pourrait masquer un
vrai regression timing. Sur CI lent (Windows ou runner partage) la
borne basse 50ms est safe. La borne haute 1500ms peut passer meme si
le timer se retarde a 1s au lieu de 200ms.

**Action** : aucune ce sprint. A envisager en Sprint 17+ si flaky
apparait. Verification 32/32 verts ce sprint donc non-urgent.

### D-2 (P3 nit) — `page.waitForTimeout(300)` magic number

**Localisation** : `web/tests/bridge-push-event.spec.ts:78,110`

Attente fixe 300ms pour que l'iframe charge + enregistre
`bridge.onEvent` handler. Plus robuste serait un `waitForFunction`
sur un marker posttraite par l'iframe au mount (ex: postMessage
`type: "iframe-ready"` que le parent collecte). MVP acceptable.

**Action** : aucune.

---

## Track E — Backward compat Sprint 13 — **CLEAN**

`git diff c32d9c7 HEAD -- web/src/bridge/protocol.ts` confirme que
`BridgeMethodSchema`, `BridgeRequestSchema`, `BridgeResponseSchema`,
`createResponse`, `createErrorResponse` sont **inchanges** depuis
Sprint 13 Phase C. Les ajouts Sprint 15 sont purement additifs
(`BridgeEventSchema`, `createEvent`, `BridgeHeartbeatSchema`). Aucun
breaking change pour les apps qui consomment le bridge Request/Response.

`useBridge` retourne maintenant `UseBridgeHandle { pushEvent,
watchdogState, resetWatchdog }` vs. void/rien Sprint 13. Changement
additif en TypeScript (un caller qui ignorait le return est toujours
valide). Le seul appelant productif est `BrowsedProject.tsx:173` qui
destructure `{ watchdogState, resetWatchdog }` — mis a jour dans le
meme commit Phase B, pas de regression.

`_startHeartbeat` du SDK a un `try/catch` autour de
`parent.postMessage` qui swallow les erreurs (`parent` absent en
file:// standalone). Comportement degrade (heartbeat envoye dans le
vide) mais pas de crash.

214 Vitest / 33 Playwright / 373 Rust / 182 SDK / 153 coord / 46 gov
tous verts au HEAD `4da0043`, incluant les tests Sprint 13 non-touches.

---

## Track F — Scope cuts respectes — **CLEAN**

Grep systematique sur les 4 commits Phase A-D :

| Check | Result |
|---|---|
| `sbfb publish` subcommand | Absent. Les matches "publish" sont des strings doc (README, help text) |
| `task_result_ready` server dispatcher | Aucun dans `packages/nexus-coordinator/src/`. Les 3 matches sont dans les templates `index.html`/`App.tsx` (cote client iframe, demo uniquement) |
| Branding / logo / favicon | 0 match dans diff Sprint 15 |
| VPS / subdomain / cross-node code | 0 match. Seules occurrences dans `docs/shell/PATTERNS.md` sont des commentaires de rationale |
| Templates Vue / Svelte / Jupyter | Absents. Ls templates/ = html, pyodide, react |
| `iframe.remove()` / kill / terminate | 0 match dans `web/src/bridge/`. Reload via `about:blank` + setTimeout, pas de kill-by-force |
| Re-publish auto / webhook | 0 match dans diff Sprint 15 |
| MIME scan | 0 match dans diff Sprint 15 |

Aucun scope creep detecte.

---

## Track G — Couverture tests — **CLEAN**

Comptage tests Sprint 15 specifiques :

| Suite | Sprint 15 additions |
|---|---|
| Vitest protocol.test.ts (BridgeEventSchema) | 5 |
| Vitest useBridge.test.ts (pushEvent) | 4 |
| Vitest watchdog.test.ts (state machine + schema) | 11 |
| Vitest BrowsedProject.test.tsx (overlay unknown) | 1 |
| Playwright bridge-heartbeat | 1 |
| Playwright bridge-push-event | 2 |
| Pytest test_cli_scaffold | 15 |
| **Total** | **39** |

Matching avec verification.md : +21 Vitest (9 Phase A + 12 Phase B)
+ 3 Playwright + 15 pytest = 39. ✓

Edge cases listes dans l'audit plan et couvrenture :

| Cas | Couvert ? |
|---|---|
| Heartbeat source mismatch | ✅ watchdog.test.ts "ignores heartbeats from unknown source" |
| resetWatchdog sans heartbeat prealable | ✅ "does not re-transition to stalled after reset" |
| Scaffold malformed running.json fallback | ✅ test_malformed_running_json_falls_back_to_placeholder |
| Scaffold unknown template "svelte" rejected | ✅ test_rejects_unknown_template |
| Playwright subscribed vs unrelated event | ✅ "iframe ignores events for non-subscribed names" |
| onEvent unsubscribe return | ❌ (cf A-1) |
| onEvent multi-callback same name | ❌ (cf A-2) |
| onEvent callback throw | ❌ (cf A-3) |

Aucun test tautologique : les assertions verifient du comportement
concret (ex: `expect(fakeWindow.postMessage).toHaveBeenCalledWith(...)`,
`expect(sbfb["node_id"]).toBe(fake_node_id)`), pas de `expect(true).toBe(true)`.

Les 3 cas non couverts sont tous P3 et touchent la meme API
(`onEvent` return + error handling). Un fichier de test
complementaire ~30 LOC couvrirait les 3. Non-bloquant pour Sprint 16.

---

## Summary

| Track | Verdict |
|---|---|
| A — Bridge push protocol | CLEAN (3 P3) |
| B — Watchdog state machine | CLEAN (1 P3 nit) |
| C — CLI scaffold + templates | CLEAN (2 P3 nit) |
| D — Playwright iframe E2E | CLEAN (1 P2, 1 P3) |
| E — Backward compat Sprint 13 | CLEAN |
| F — Scope cuts respectes | CLEAN |
| G — Couverture tests | CLEAN (3 P3 deja referencees A-1/2/3) |

**Conclusion** : Sprint 15 est un sprint serre, sans P0/P1, scope
strictement respecte, couverture solide des D1..D5. Les P2/P3
identifies sont nits de robustness ou de couverture marginale, aucun
ne merite un `fix(sprint15): ...` dedie.

---

## Hors-scope audit — migration PARA staged

Le `git status` au HEAD `4da0043` montre ~50 renames staged non commit :
`.planning/sprint*.md -> .planning/archive/v1.0/sprint*.md` (S3, S4..S13)
et `.planning/sprint1[45]*.md -> .planning/archive/v1.1/...`. Le diff
`f6015b3..4da0043` confirme qu'aucun des 5 commits Sprint 15 n'a
committe cette migration layout. C'est un cleanup PARA (cf.
nexus_grid_pivot.md "Layout planning depuis S16") prepare dans l'index
mais non-livre.

**Recommandation** : commiter separement la migration avant le premier
commit Sprint 16 Phase A, avec un titre type
`chore(planning): PARA layout — archive S0-15 under v1.0/v1.1/`.
N'entre pas dans le verdict audit (pas du code Sprint 15).

---

## Next steps

1. Commiter le present findings doc dans `.planning/active/` avec un
   `docs(sprint15): audit findings from Sprint 16 Phase 0 gate`
2. Commiter la migration PARA staged en `chore(planning): ...`
3. Demarrer Sprint 16 Phase A (bearer token loopback) selon
   `.planning/active/sprint16_kickoff.md` §4 D1 apres validation D1..D5
   par l'utilisateur
