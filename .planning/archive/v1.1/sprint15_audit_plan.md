# Sprint 15 — Audit plan pour Sprint 16 Phase 0

**Ecrit** : 2026-04-14
**Tip a auditer** : `f5aea3e` (Phase D) + commit Phase E docs
**Commit stack** : 4 commits Phase A-D + 1 docs

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre : memory -> git log Sprint 15 -> kickoff -> plan
   -> verification -> cet audit plan
2. **NE PAS lire** `docs/shell/PATTERNS.md` avant d'avoir forme une
   opinion track par track
3. Timebox suggere : 2-3h
4. Delivrable : `.planning/sprint15_audit_findings.md`
5. Les commits fix eventuels (P0/P1) doivent atterrir avant le
   premier commit Sprint 16 Phase A

---

## Track A — Bridge push protocol (D1)

**Question** : le nouveau canal `sbfb-bridge-event` est-il
correctement fire-and-forget, sans leak de coupling inutile ?

**Methodes** :
- Lire `web/src/bridge/protocol.ts` : `BridgeEventSchema` verifie
  `name` string 1..64 chars, `payload: z.unknown()`. Pas de `id`
  ni `correlation_id`.
- Lire `web/public/sbfb-bridge.js` : `_onMessage` gere 2 types
  (response + event) sans conflit. `onEvent(name, cb)` gere Set de
  callbacks. `destroy()` nettoie `_eventHandlers`.
- Lire `web/src/bridge/useBridge.ts` : `pushEvent` ne fait PAS de
  source validation (sortant, pas besoin). Return UseBridgeHandle
  incluant `pushEvent`.
- Tester manuellement : `bridge.onEvent("same", cb1); bridge.onEvent
  ("same", cb2)` -> les 2 cb appelees ? (Set gere).
- Verifier callback errors swallow : `cb = () => { throw "x" }` ne
  casse pas le bridge.
- Verifier `BridgeEventSchema` rejette name "" et name > 64 chars.

**Signal** :
- P0 : memory leak callbacks (destroy n'enleve pas tout)
- P1 : onEvent crash quand callback throw
- P2 : protocol accepte name arbitrairement long (pas de cap) →
  non, on a z.string().max(64)
- P3 : nit sur le pattern Set vs Array

---

## Track B — CPU watchdog state machine (D2)

**Question** : le watchdog detecte-t-il correctement les transitions
healthy / stalled / unknown sans faux positifs ?

**Methodes** :
- Lire `web/src/bridge/useBridge.ts` : state machine avec
  `lastHeartbeatRef` + setInterval 2s. `STALL_THRESHOLD_MS = 5000`
  exporte.
- Verifier : `resetWatchdog` ramene bien a "unknown" (pas "healthy"
  faux-default).
- Verifier : heartbeat source validation (event.source ===
  iframe.contentWindow) present.
- Verifier : schema Zod rejette `ts: 0` et `ts: -1` (z.number().positive()).
- Lire `BrowsedProject.tsx` : `reloadIframe` check `daemonInfo` +
  `entry.archive_hash` avant de reset. Guard present.
- Verifier : overlay JSX conditionnel STRICTEMENT sur `watchdogState
  === "stalled"`, pas sur `!== "healthy"` (sinon flash au mount).
- Tester Vitest avec fake timers : unknown -> healthy, healthy ->
  stalled apres 5s, stalled -> healthy sur nouveau heartbeat,
  resetWatchdog -> unknown sans re-transition immediate.

**Signal** :
- P0 : race condition entre heartbeat receiver et stall checker
- P1 : `resetWatchdog` ne reset pas vraiment (revient en stalled
  tout de suite)
- P1 : `reloadIframe` crash quand daemonInfo null
- P2 : faux positif au cold-start (overlay flash avant premier
  heartbeat)
- P3 : valeurs 5000/2000 hard-codees (pas configurables)

---

## Track C — CLI scaffold + templates (D3)

**Question** : `sbfb init` est-il robuste (rejections propres,
substitutions complete, pas de leak de placeholders) ?

**Methodes** :
- Lire `scaffold.py::_substitute` : remplace `{{NODE_ID}}` et
  `{{PROJECT_NAME}}`. Aucun autre placeholder support.
- Lire `scaffold.py::_read_local_node_id` : try/except robuste,
  retourne None sur JSON invalide ou OSError.
- Verifier : template pyodide `index.html` charge `./pyodide/
  pyodide.js` relatif, PAS un CDN. Le README explique explicitement
  la CSP blob-serve (connect-src 'none').
- Lire `sbfb_main.py` : callback vide present pour forcer multi-
  command mode (sinon Typer avale "init" comme arg).
- Lire `pyproject.toml` : entry point `sbfb = ...sbfb_main:app` +
  `[tool.hatch.build.targets.wheel.force-include]` avec
  `templates/` -> `nexus_coordinator/templates`.
- Tester : `uv run sbfb init html /tmp/x` dans repo local.
- Verifier : aucun test ne laisse echapper `{{NODE_ID}}` dans un
  fichier genere si daemon running.
- Rechercher d'autres occurrences de `{{` dans les templates
  (ne devrait rien trouver apart les placeholders connus).

**Signal** :
- P0 : shell injection dans `_copy_template` (path traversal via
  ref name malicieux) - improbable mais verifier
- P1 : template pyodide charge Pyodide depuis CDN (contournement
  CSP sil passait)
- P1 : un placeholder utilise dans les templates mais non-substitue
  dans `_substitute`
- P2 : wheel build ne contient pas les templates (force-include mal
  configure)
- P3 : messages Rich avec emojis (vs convention projet)

---

## Track D — Playwright iframe E2E (D4)

**Question** : les tests Playwright exercent-ils vraiment le flow
cross-origin avec le SDK reel ?

**Methodes** :
- Lire `web/tests/bridge-heartbeat.spec.ts` : verifie que le test
  lit `public/sbfb-bridge.js` avec `readFile`. Le fichier bridge est
  inline via route.fulfill, donc le SDK EXACT que les apps utilisent.
- Verifier le replace `<\/script>` : commentaire explicite le why.
- Lire `bridge-push-event.spec.ts` : le pattern echo (iframe
  postMessage retour) est le seul moyen de verifier le callback
  fired sans `contentDocument` (sandbox sans same-origin bloque).
- Verifier : les 3 tests peuvent tourner en isolement (pas de
  shared state via window ou localStorage cross-test).
- Verifier : heartbeatInterval: 200ms au lieu du default 1000ms —
  accelere les tests, legitime.
- Tester robustness : rerun 3x, aucun flake ?

**Signal** :
- P1 : les tests utilisent un stub de bridge.js (pas le vrai)
- P2 : flaky sur les timeouts (200ms trop court)
- P3 : pas de cleanup route entre tests

---

## Track E — Backward compat Sprint 13

**Question** : le Sprint 15 casse-t-il le bridge Sprint 13 ?

**Methodes** :
- `git log --oneline Sprint 13 Phase C` : identifier commit
  `c32d9c7` (bridge initial). Les tests Sprint 13 sont-ils toujours
  verts apres Sprint 15 ?
- `grep useBridge(` dans web/src : compter les appelants.
  Sprint 13 n'utilisait pas le return ; Sprint 15 ajoute un return.
  TypeScript accepte — l'ancien code fait `useBridge(...)` sans
  destructurer, ce qui est valide (on ignore le return).
- Verifier que `BridgeRequestSchema` et `BridgeResponseSchema`
  n'ont pas ete touches (breaking pour les apps existantes).
- Lire `useBridge.ts::handler` : heartbeat checked avant request.
  Si un ancien daemon envoie un request sans heartbeat jamais — le
  request path est atteint. Aucun lock-out.
- Le constructor SBFBBridge auto-demarre le heartbeat. Une app
  Sprint 13 sans host qui ecoute recoit des erreurs dans la
  console (postMessage sur parent null). `try/catch` autour du
  `parent.postMessage` dans `_startHeartbeat` — verifier.

**Signal** :
- P0 : BridgeRequestSchema ou BridgeResponseSchema ont change
  (breaking pour apps Sprint 13)
- P1 : heartbeat auto-start casse les apps standalone (ouvertes en
  file:// sans host)
- P2 : console pollution avec warnings heartbeat quand pas d'host
- P3 : nit sur la doc

---

## Track F — Scope cuts respectes

**Question** : aucune ligne de code n'a fuit dans les zones Sprint 16+ ?

**Methodes** :
- Grep dans les 4 commits Phase A-D :
  - `grep -r "publish" packages/nexus-coordinator/src/nexus_coordinator/cli/` → pas de `sbfb publish` sous-commande
  - `grep "task_result_ready" packages/nexus-coordinator/` → pas de dispatcher server-side qui produit des events
  - `grep -i "brand\|logo\|favicon" web/src` → pas de branding dans les 4 commits
  - `grep -i "vps\|subdomain\|cross-node" web/src crates/` → pas d'infra ops
  - `grep "Vue\|Svelte\|Jupyter" packages/nexus-coordinator/src/nexus_coordinator/templates/` → pas de templates additionnels
  - `grep "iframe.remove\|kill\|terminate" web/src/bridge/` → pas de kill-by-force
- Verifier que les ~30 items "differe" listes dans le kickoff §6
  sont bien absents.

**Signal** :
- P1 : scope creep confirme (un item Sprint 16+ a fuit dans le code)
- P2 : scope creep "tant qu'on y est" subtil (refactor hors scope)

---

## Track G — Couverture tests + corrects

**Question** : les tests Vitest + pytest + Playwright couvrent-ils
les cas critiques des D1..D5 ?

**Methodes** :
- Compter :
  - Vitest bridge/ : protocol (4+5=9), useBridge (3+4=7), watchdog
    (8+3=11), BrowsedProject watchdog (1) = 28 tests bridge-specific
  - Pytest scaffold : 15 tests
  - Playwright : 3 tests (heartbeat + push + ignore)
- Check cas manques :
  - Heartbeat source validation unit test (event.source != iframe
    contentWindow)
  - resetWatchdog sans heartbeat prealable (unknown -> reset ->
    unknown)
  - Scaffold : malformed running.json fallback vers placeholder
  - Scaffold : unknown type "svelte" rejete avec exit != 0
  - Playwright : subscribed_event vs unrelated_event (filtrage par
    name cote iframe)
- Verifier que les tests ne sont pas tautologiques (passent
  automatiquement car l'assertion est triviale).

**Signal** :
- P1 : un D1..D5 sans aucun test associe
- P2 : tests mais assertions faibles
- P3 : un edge case non couvert

---

## Verdict global attendu

- **PASS** : 0 P0, 0 P1 -> Sprint 16 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables -> Sprint 16 bloque tant
  que les `fix(sprint15): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 -> re-conception partielle

---

## Out of scope pour l'audit

- Les D1..D5 gelees (cf. kickoff §4) — ne pas les rebattre
- Les scope cuts (re-publish, branding, VPS, dispatcher server-side,
  etc.) - listes dans kickoff §6
- Les pins de deps (iroh 0.97, axum 0.7, pyodide 0.29.3)
- Le test SDK flaky Windows (pre-existant Sprint 14 et avant)
- Les P2 Sprint 14 (T44..T51) loggees en Phase E - non re-ouvertes
- Les fichiers non trackes (cc.json, site/, docs/apps/, etc.)
- La decision CLI `sbfb` separe vs `nexus-coordinator` - Day 0

---

## Livrable final attendu

```
.planning/sprint15_audit_findings.md
```

Format : verdict global + une section par track (A-G) avec
findings ventiles P0 / P1 / P2 / P3 + commits fix pour les P0/P1
eventuels.
