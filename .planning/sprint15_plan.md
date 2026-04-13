# Sprint 15 — Plan d'execution detaille

**Ecrit** : 2026-04-14, suite a `sprint15_kickoff.md`
**Tip master entree** : `f6015b3`
**Branche** : `master`

---

## 1. Etat verifie a l'entree

| Suite | Count | Commande |
|---|---|---|
| Rust workspace | 373 pass | `cargo test --workspace --locked` |
| Python SDK | 183 pass (1 flaky Windows) | `uv run pytest packages/nexus-sdk/tests/ -q` |
| Python coordinator | 138 + 1 skipped | `uv run pytest packages/nexus-coordinator/tests/ -q` |
| Python app-gov | 46 pass | `uv run pytest packages/nexus-app-gov/tests/ -q` |
| Vitest unit | 193 pass | `npm run test:unit` (cwd web) |
| Playwright | 30 pass | `npx playwright test` |
| size-limit | 7/7 under budget | `npm run size` |
| SPDX | 224 files | `bash scripts/check-spdx.sh` |
| clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| ruff | clean | `uv run ruff format --check packages/ && uv run ruff check packages/` |
| tsc | clean | `npx tsc --noEmit -p tsconfig.app.json` |
| lint web | 0 errors, 6 warnings (T1) | `npm run lint` (cwd web) |
| scan-en-strings | clean | `bash scripts/scan-en-strings.sh` (cwd web) |

---

## 2. Decisions Day 0 (gelees, cf. kickoff §4)

- **D1** : bridge push = event type `sbfb-bridge-event`, fire-and-forget
- **D2** : watchdog = heartbeat 1s / timeout 5s + overlay reload
- **D3** : CLI `sbfb init <type>` 3 templates html/react/pyodide
- **D4** : 2 Playwright specs avec fixtures locales
- **D5** : P2 Sprint 14 -> T44..T51 PATTERNS.md (pas de code fix)

---

## 3. Research consulte

- **Bridge Sprint 13 Phase C** : lu `web/public/sbfb-bridge.js` (127 LOC),
  `web/src/bridge/useBridge.ts` (129 LOC), `web/src/bridge/protocol.ts`
  (76 LOC). Pattern bien isole, extension par ajout d'un 3e message
  type naturelle.
- **iframe CSP** : `crates/nexus-shell-daemon/src/http.rs` injecte
  `Content-Security-Policy` sur les reponses blob-serve. CSP actuel
  permet `script-src 'unsafe-inline'` — suffit pour heartbeat.
- **BrowsedProject.tsx** : iframe `sandbox="allow-scripts"` sans
  `allow-same-origin`, refs deja en place (`iframeRef`). Extension
  naturelle avec un overlay sur la div parent.
- **Typer** : deja dependance du coordinator (`typer>=0.12`). Entry
  point `[project.scripts]` standard pyproject.toml.
- **Pyodide CDN** : version 0.29.3 stable (cf. CLAUDE.md). Loader
  script standard : `<script src="https://cdn.jsdelivr.net/pyodide/
  v0.29.3/full/pyodide.js"></script>`. Le CSP blob-serve PERMET ces
  fetches (script-src wildcarded sur external scripts via
  `script-src 'unsafe-inline' https:`).
- **Playwright webServer** : `playwright.config.ts` a un bloc
  `webServer` qui lance le vite dev server. Pour servir les fixtures
  cross-origin, on peut lancer un 2e server side-car (http-server npm
  package) ou utiliser `page.route(...)` pour intercepter les requetes
  vers un hostname fictif. Choix : http-server side-car sur un port
  dedie (plus realiste, teste le path complet incluant CORS).

---

## 4. Phase A — Bridge push bidirectionnel

### Fichiers modifies

#### `web/public/sbfb-bridge.js` (+25 LOC)

```js
class SBFBBridge {
  constructor(options) {
    // ... existing fields ...
    this._eventHandlers = new Map();  // name -> Set<callback>
  }

  onEvent(name, callback) {
    if (!this._eventHandlers.has(name)) {
      this._eventHandlers.set(name, new Set());
    }
    this._eventHandlers.get(name).add(callback);
    return () => this._eventHandlers.get(name)?.delete(callback);
  }

  _onMessage = (event) => {
    const msg = event.data;
    if (!msg) return;

    if (msg.type === "sbfb-bridge-response") {
      // ... existing response handling ...
    } else if (msg.type === "sbfb-bridge-event") {
      const handlers = this._eventHandlers.get(msg.name);
      if (handlers) {
        for (const cb of handlers) {
          try { cb(msg.payload); } catch (e) { /* swallow */ }
        }
      }
    }
  };
}
```

#### `web/src/bridge/protocol.ts` (+20 LOC)

```typescript
export const BridgeEventSchema = z.object({
  type: z.literal("sbfb-bridge-event"),
  name: z.string().min(1).max(64),
  payload: z.unknown(),
});

export type BridgeEvent = z.infer<typeof BridgeEventSchema>;

export function createEvent(name: string, payload: unknown): BridgeEvent {
  return { type: "sbfb-bridge-event", name, payload };
}
```

#### `web/src/bridge/useBridge.ts` (+15 LOC)

```typescript
export function useBridge(...) {
  // ... existing ...

  const pushEvent = useCallback((name: string, payload: unknown) => {
    const iframe = iframeRef.current;
    if (!iframe || !iframe.contentWindow) return;
    iframe.contentWindow.postMessage(createEvent(name, payload), "*");
  }, [iframeRef]);

  return { pushEvent };
}
```

### Tests (Vitest, ~150 LOC)

`web/src/bridge/__tests__/push.test.ts` :

- `test_onEvent_registers_callback` : `bridge.onEvent("foo", cb)` puis
  `window.postMessage({type: "sbfb-bridge-event", name: "foo",
  payload: {x: 1}})` appelle cb avec `{x: 1}`
- `test_onEvent_ignores_other_names` : callback pour "foo" pas appele
  quand event "bar" arrive
- `test_onEvent_multiple_handlers` : 2 callbacks enregistres pour meme
  nom, les 2 appeles
- `test_onEvent_returns_unsubscribe` : fonction retournee desabonne
- `test_pushEvent_fires_postMessage_on_iframe` : appel pushEvent
  declenche `contentWindow.postMessage` avec event bien forme
- `test_pushEvent_noop_when_no_iframe` : iframeRef.current === null
  ne throw pas
- `test_BridgeEventSchema_rejects_missing_name` : Zod parse echoue
- `test_BridgeEventSchema_rejects_long_name` : name > 64 chars rejete

### Critere d'acceptation

- Tous les 8 tests verts
- `npm run test:unit` : 193 + 8 = 201 tests
- `tsc` clean
- `npm run lint` : 0 errors

### Commit

```
feat(bridge): Sprint 15 Phase A — bidirectional push via sbfb-bridge-event

Body :
- Contexte : le bridge Sprint 13 etait request/response uniquement.
  Ce sprint ajoute un canal push host -> iframe pour permettre au
  coordinateur de notifier l'app (task result ready, storage changed)
- Files : sbfb-bridge.js, protocol.ts, useBridge.ts, push.test.ts
- Delta tests : Vitest 193 -> 201 (+8)
- Scope cuts : dispatcher server-side events, whitelist event names
  (tous differes Sprint 16+)
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 5. Phase B — CPU watchdog iframe

### Fichiers modifies

#### `web/public/sbfb-bridge.js` (+30 LOC)

```js
class SBFBBridge {
  constructor(options) {
    // ...
    this._heartbeatInterval = (options && options.heartbeatInterval) || 1000;
    this._heartbeatTimer = null;
    this._startHeartbeat();
  }

  _startHeartbeat() {
    if (this._heartbeatTimer) return;
    this._heartbeatTimer = setInterval(() => {
      try {
        parent.postMessage(
          { type: "sbfb-bridge-heartbeat", ts: Date.now() },
          "*",
        );
      } catch (e) { /* swallow */ }
    }, this._heartbeatInterval);
  }

  _stopHeartbeat() {
    if (this._heartbeatTimer) {
      clearInterval(this._heartbeatTimer);
      this._heartbeatTimer = null;
    }
  }

  destroy() {
    this._stopHeartbeat();
    // ... existing destroy ...
  }
}
```

#### `web/src/bridge/protocol.ts` (+10 LOC)

```typescript
export const BridgeHeartbeatSchema = z.object({
  type: z.literal("sbfb-bridge-heartbeat"),
  ts: z.number().positive(),
});

export type BridgeHeartbeat = z.infer<typeof BridgeHeartbeatSchema>;
```

#### `web/src/bridge/useBridge.ts` (+60 LOC)

```typescript
export type WatchdogState = "unknown" | "healthy" | "stalled";

const STALL_THRESHOLD_MS = 5000;
const CHECK_INTERVAL_MS = 2000;

export function useBridge(...) {
  // ... existing ...
  const [watchdogState, setWatchdogState] = useState<WatchdogState>("unknown");
  const lastHeartbeatRef = useRef<number | null>(null);

  useEffect(() => {
    function handler(event: MessageEvent) {
      const hb = BridgeHeartbeatSchema.safeParse(event.data);
      if (hb.success) {
        lastHeartbeatRef.current = Date.now();
        if (watchdogState !== "healthy") setWatchdogState("healthy");
        return;
      }
      // ... existing request handling ...
    }
    // ... existing addEventListener/removeEventListener ...
  }, [/* deps */]);

  useEffect(() => {
    const timer = setInterval(() => {
      const last = lastHeartbeatRef.current;
      if (last === null) return;
      const age = Date.now() - last;
      if (age > STALL_THRESHOLD_MS && watchdogState !== "stalled") {
        setWatchdogState("stalled");
      }
    }, CHECK_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [watchdogState]);

  const resetWatchdog = useCallback(() => {
    lastHeartbeatRef.current = null;
    setWatchdogState("unknown");
  }, []);

  return { pushEvent, watchdogState, resetWatchdog };
}
```

#### `web/src/pages/BrowsedProject.tsx` (+50 LOC)

```typescript
const { watchdogState, resetWatchdog } = useBridge(coordUrl, appName, iframeRef);

function reloadIframe() {
  const frame = iframeRef.current;
  if (!frame || !entry.archive_hash) return;
  const url = blobServeUrl(daemonUrl, entry.archive_hash);
  frame.src = "about:blank";
  // small delay to ensure blank loaded
  setTimeout(() => {
    if (iframeRef.current) iframeRef.current.src = url;
    resetWatchdog();
  }, 50);
}

// In JSX, above the iframe:
{watchdogState === "stalled" && (
  <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md" data-testid="watchdog-overlay">
    <div className="glass-card max-w-sm p-8 text-center">
      <AlertCircle className="mx-auto mb-4 h-10 w-10 text-amber-400" />
      <h3 className="mb-2 text-lg font-bold text-white">Application ne repond plus</h3>
      <p className="mb-6 text-sm text-white/70">
        L'app n'a pas envoye de signal depuis plusieurs secondes.
      </p>
      <div className="flex gap-3 justify-center">
        <button onClick={reloadIframe} data-testid="watchdog-reload">Recharger</button>
        <button onClick={...close nav...} data-testid="watchdog-close">Fermer</button>
      </div>
    </div>
  </div>
)}
```

### Tests Vitest (~150 LOC)

`web/src/bridge/__tests__/watchdog.test.ts` :

- `test_heartbeat_transitions_to_healthy` : envoyer heartbeat, state
  passe de unknown -> healthy
- `test_no_heartbeat_stays_unknown` : pas de heartbeat, state reste
  unknown meme apres 6s (fake timers)
- `test_stale_heartbeat_transitions_to_stalled` : heartbeat recu puis
  silence 6s (fake timers), state -> stalled
- `test_resume_heartbeat_recovers_state` : stalled puis heartbeat
  arrive, state -> healthy
- `test_resetWatchdog_returns_to_unknown` : stalled puis reset, state
  -> unknown
- `test_heartbeat_schema_rejects_negative_ts` : Zod parse echoue

`web/src/pages/__tests__/BrowsedProject.test.tsx` (ajout) :

- `test_watchdog_overlay_hidden_when_healthy` : mock useBridge healthy,
  pas d'overlay
- `test_watchdog_overlay_visible_when_stalled` : mock stalled, overlay
  present avec 2 boutons
- `test_watchdog_reload_resets_iframe_src` : clic Recharger, iframe.src
  pass to about:blank puis retour a blobServeUrl

### Critere d'acceptation

- 9 tests Vitest nouveaux verts
- `npm run test:unit` : 201 + 9 = 210 tests
- `tsc` clean
- `npm run lint` clean
- `scan-en-strings.sh` clean ("Application ne repond plus" en francais)

### Commit

```
feat(watchdog): Sprint 15 Phase B — CPU watchdog via heartbeat + stalled overlay

Body :
- Contexte : une iframe avec un bug d'infinite-loop bloque l'UI sans
  recours. Ce sprint ajoute un heartbeat 1s cote client et une
  detection 5s cote host qui declenche un overlay de reload.
- Files : sbfb-bridge.js (+30), protocol.ts (+10), useBridge.ts (+60),
  BrowsedProject.tsx (+50), __tests__ watchdog + BrowsedProject
- Delta tests : Vitest 201 -> 210 (+9)
- Scope cuts : kill-by-force iframe (non supporte par browser APIs),
  monitoring CPU cote host (cross-origin impossible)
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 6. Phase C — CLI `sbfb init` + templates

### Fichiers crees

#### `packages/nexus-coordinator/templates/html/`

- `index.html` (~80 LOC) : page statique avec sbfb-bridge.js via CDN
  (ou relative path), bouton "Submit task" qui appelle `bridge.submitTask`
- `SBFB.json` (~5 LOC) : `{"node_id": "{{NODE_ID}}", "project_name":
  "{{PROJECT_NAME}}", "template": "html"}`
- `README.md` (~40 LOC) : comment tester en local, comment deployer
- `.gitignore` (~10 LOC) : dist/, node_modules/, .DS_Store

#### `packages/nexus-coordinator/templates/react/`

- `package.json` (~20 LOC) : vite, react, sbfb-bridge dep copy
- `vite.config.ts` (~15 LOC) : base '.', output dist
- `index.html` (~15 LOC) : entry avec `<div id="root">`
- `src/App.tsx` (~60 LOC) : composant qui utilise sbfb-bridge.js, demo
  submitTask + storage
- `src/main.tsx` (~10 LOC)
- `SBFB.json` (~5 LOC) avec placeholders
- `README.md` (~50 LOC) : instructions `npm install && npm run build`
- `.gitignore`

#### `packages/nexus-coordinator/templates/pyodide/`

- `index.html` (~80 LOC) : loader Pyodide 0.29.3, execute Python inline
  avec `pyodide.runPythonAsync`, demo qui appelle bridge.submitTask
- `SBFB.json` avec placeholders
- `README.md` (~40 LOC) : explications Pyodide, limitations CSP
- `.gitignore`

#### `packages/nexus-coordinator/src/nexus_coordinator/cli/__init__.py` (~40 LOC)

```python
# SPDX-License-Identifier: AGPL-3.0-or-later
"""CLI entry point for the `sbfb` command.

Sprint 15 Phase C — exposes subcommands via typer. Currently:
- `sbfb init <type> <path>` : scaffold a new app from template
"""

import typer

from nexus_coordinator.cli.init import init

app = typer.Typer(no_args_is_help=True, add_completion=False)
app.command(name="init")(init)


def main() -> None:
    app()
```

#### `packages/nexus-coordinator/src/nexus_coordinator/cli/init.py` (~150 LOC)

```python
# SPDX-License-Identifier: AGPL-3.0-or-later
"""`sbfb init` subcommand — scaffold a new app from a template.

Sprint 15 Phase C. Usage:
    sbfb init html ./my-app
    sbfb init react ./my-app
    sbfb init pyodide ./my-app
"""

import importlib.resources
import json
import shutil
from enum import Enum
from pathlib import Path

import structlog
import typer

_log = structlog.get_logger(__name__)

TEMPLATE_ROOT = importlib.resources.files("nexus_coordinator").joinpath("templates")


class TemplateType(str, Enum):
    html = "html"
    react = "react"
    pyodide = "pyodide"


def _read_running_node_id() -> str | None:
    """Read the local daemon's node_id from running.json if present."""
    path = Path.home() / "nexus-grid" / "shell-daemon" / "running.json"
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        return data.get("node_id")
    except (json.JSONDecodeError, OSError):
        return None


def _substitute_placeholders(content: str, *, node_id: str, project_name: str) -> str:
    return content.replace("{{NODE_ID}}", node_id).replace("{{PROJECT_NAME}}", project_name)


def _copy_template(template_name: str, dest: Path, *, node_id: str, project_name: str) -> None:
    src = TEMPLATE_ROOT / template_name
    if not src.is_dir():
        raise typer.BadParameter(f"template '{template_name}' not found at {src}")
    for item in src.rglob("*"):
        if item.is_file():
            rel = item.relative_to(src)
            target = dest / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            text = item.read_text(encoding="utf-8")
            substituted = _substitute_placeholders(text, node_id=node_id, project_name=project_name)
            target.write_text(substituted, encoding="utf-8")


def init(
    template_type: TemplateType = typer.Argument(..., help="Template type: html | react | pyodide"),
    path: Path = typer.Argument(..., help="Destination directory (must not exist)"),
) -> None:
    """Scaffold a new SBFB app from a template."""
    if path.exists():
        raise typer.BadParameter(f"destination {path} already exists")
    node_id = _read_running_node_id() or "{{NODE_ID}}"  # leave placeholder if no daemon
    project_name = path.name
    path.mkdir(parents=True)
    _copy_template(template_type.value, path, node_id=node_id, project_name=project_name)
    typer.echo(f"Created {template_type.value} app at {path}")
    if node_id == "{{NODE_ID}}":
        typer.echo("WARNING: daemon not running, SBFB.json contains {{NODE_ID}} placeholder")
        typer.echo("Edit SBFB.json with your node_id before publishing")
```

### Modifications

#### `packages/nexus-coordinator/pyproject.toml`

```toml
[project.scripts]
sbfb = "nexus_coordinator.cli:main"
```

`[tool.setuptools.package-data]` ou `include_package_data = true` +
MANIFEST.in pour inclure `templates/**`.

### Tests pytest (~250 LOC)

`packages/nexus-coordinator/tests/test_cli_init.py` :

- `test_init_html_creates_expected_files` : `sbfb init html /tmp/x`
  cree index.html + SBFB.json + README + .gitignore
- `test_init_react_creates_expected_files` : idem pour react
- `test_init_pyodide_creates_expected_files` : idem pour pyodide
- `test_init_substitutes_node_id_from_daemon` : mock running.json
  present, verifier SBFB.json contient le node_id reel
- `test_init_substitutes_project_name` : SBFB.json contient le dernier
  segment du path
- `test_init_leaves_placeholder_when_no_daemon` : running.json absent,
  SBFB.json contient `{{NODE_ID}}` + warning stdout
- `test_init_rejects_existing_path` : path existe, BadParameter
- `test_init_rejects_unknown_type` : typer bad param
- `test_init_sbfb_json_is_valid_json` : parse le fichier genere
- `test_init_substituted_content_has_no_unresolved_placeholders` :
  tous les `{{...}}` sont remplaces quand daemon present

Chaque test utilise CliRunner (typer testing) avec `tmp_path`.

### Critere d'acceptation

- 10 tests pytest nouveaux verts
- `uv run pytest packages/nexus-coordinator/tests/ -q` : 138 + 10 = 148
- `uv run ruff format --check && uv run ruff check` clean
- Les 3 templates sont valides comme zip SBFB (index.html + SBFB.json
  present, pas d'erreurs syntaxiques HTML/JSON)
- SPDX : les nouveaux fichiers Python ont l'en-tete. Les templates
  n'ont pas besoin (artefacts utilisateur final — ils ne sont pas
  code source du projet)

### Commit

```
feat(cli): Sprint 15 Phase C — sbfb init CLI with html/react/pyodide templates

Body :
- Contexte : publier une app SBFB ne requiert aujourd'hui qu'un zip
  avec index.html + SBFB.json, mais aucun outil de scaffold n'existe.
  Ce sprint livre `sbfb init <type> <path>` avec 3 templates MVP.
- Files : templates/{html,react,pyodide}/, cli/__init__.py, cli/init.py,
  test_cli_init.py, pyproject.toml entry point
- Delta tests : coord 138+1s -> 148+1s (+10)
- Scope cuts : publish subcommand, additional templates (Vue/Svelte/
  Jupyter) -> Sprint 16+
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 7. Phase D — Tests Playwright E2E

### Fichiers crees

#### `web/tests/fixtures/bridge-sample/index.html` (~50 LOC)

HTML + script qui :
- inclut `sbfb-bridge.js`
- instantie `const bridge = new SBFBBridge()`
- `bridge.onEvent("test_event", (payload) => { document.getElementById("received").textContent = JSON.stringify(payload); })`
- Un bouton "Stop heartbeat" qui appelle `bridge._stopHeartbeat()` pour simuler freeze

#### `web/tests/bridge-push.spec.ts` (~100 LOC)

```typescript
test("host can push events to iframe", async ({ page }) => {
  // navigate to a page that mounts useBridge with fixtures iframe
  // wait for iframe load
  // use test hook or data-testid to trigger pushEvent from host
  // assert iframe DOM shows received payload
});
```

#### `web/tests/watchdog-stalled.spec.ts` (~120 LOC)

```typescript
test("watchdog overlay appears after iframe stops heartbeat", async ({ page }) => {
  // navigate to page with iframe
  // click "Stop heartbeat" button in iframe
  // wait 6s (stall threshold 5s + grace)
  // assert watchdog-overlay visible
  // click reload
  // assert overlay disappears within 3s (heartbeat resumes)
});
```

### Adjustments

#### `web/playwright.config.ts`

Add a `fixtures` server (via http-server npm package or node http
static server) that serves `web/tests/fixtures/` on a dedicated port
(say 4210). The main `webServer` remains the vite dev server.

#### `web/src/pages/BrowsedProject.tsx`

For the Playwright test, we might need a test-only route that renders
BrowsedProject with a hardcoded fixture URL. Alternative: add a URL
param like `?test-iframe-src=http://localhost:4210/bridge-sample/`
that overrides the blob-serve URL — only active when
`import.meta.env.MODE === "test"` or behind a `data-test` guard.

### Tests coverage

- Playwright 30 -> 32 (+2)
- Pas de test tiers (TypeScript, Zod...) ; la logique est deja
  couverte par Vitest Phase A/B

### Critere d'acceptation

- `npx playwright test` : 32 pass
- Les 2 nouveaux specs sont independants (peuvent tourner seuls)

### Commit

```
test(bridge): Sprint 15 Phase D — Playwright iframe push + watchdog stalled

Body :
- Contexte : Vitest couvre la logique bridge mais pas le flow
  cross-origin reel. Ces 2 specs exercent un vrai iframe via un
  server side-car servant des fixtures locales.
- Files : web/tests/fixtures/bridge-sample/, bridge-push.spec.ts,
  watchdog-stalled.spec.ts, playwright.config.ts (side-car), eventuel
  hook test BrowsedProject.tsx
- Delta tests : Playwright 30 -> 32 (+2)
- Scope cuts : tests via vrai daemon blob-serve (ajoute en Sprint 16+
  quand le daemon sera dans la CI matrix)
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 8. Phase E — Docs (verification + audit plan + PATTERNS T44-T51)

### Fichiers crees/modifies

- `.planning/sprint15_verification.md` : fail-fast checklist ~30 rows
- `.planning/sprint15_audit_plan.md` : plan audit 7-8 tracks pour Sprint 16
- `docs/shell/PATTERNS.md` : section "Sprint 14 audit tech debt"
  avec T44..T51 (30 LOC ajoutees)

### Commit

```
docs(sprint15): verification + audit plan for Sprint 16

Body :
- Contexte : sortie du sprint, documenter ce qu'on a livre, planifier
  l'audit Sprint 16 Phase 0, logger les 8 P2 Sprint 14 en tech debt.
- Files : sprint15_verification.md, sprint15_audit_plan.md,
  docs/shell/PATTERNS.md (T44-T51)
- Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## 9. Fail-fast checklist

A remplir en Phase E `sprint15_verification.md` :

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | _ |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | _ |
| 3 | cargo test | `cargo test --workspace --locked` | 373 pass | _ |
| 4 | ruff format | `uv run ruff format --check packages/` | clean | _ |
| 5 | ruff check | `uv run ruff check packages/` | clean | _ |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 183 pass (1 flaky Win) | _ |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 138 -> 148 pass (+10) | _ |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | _ |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | _ |
| 10 | eslint | `npm run lint` | 0 errors | _ |
| 11 | vitest | `npm run test:unit` | 193 -> 210 pass (+17) | _ |
| 12 | build | `npm run build` | success | _ |
| 13 | size-limit | `npm run size` | 7/7 under budget | _ |
| 14 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | _ |
| 15 | Playwright | `npx playwright test` | 30 -> 32 pass (+2) | _ |
| 16 | SBFBBridge onEvent | vitest `push.test.ts` | 8 pass | _ |
| 17 | SBFBBridge heartbeat | vitest `watchdog.test.ts` | 6 pass | _ |
| 18 | BrowsedProject overlay | vitest `BrowsedProject.test.tsx` | 3 new pass | _ |
| 19 | Playwright push | `npx playwright test bridge-push` | pass | _ |
| 20 | Playwright watchdog | `npx playwright test watchdog-stalled` | pass | _ |
| 21 | sbfb init html | `sbfb init html /tmp/t1 && ls /tmp/t1` | 4 files | _ |
| 22 | sbfb init react | idem react | 7+ files | _ |
| 23 | sbfb init pyodide | idem pyodide | 4 files | _ |
| 24 | SBFB.json valide | `jq . /tmp/t1/SBFB.json` | ok | _ |
| 25 | placeholder substitue | grep `{{` dans tmp | 0 match si daemon UP | _ |
| 26 | SPDX new files | 2 cli .py + templates | tous ok | _ |
| 27 | bridge push schema | vitest Zod reject | pass | _ |
| 28 | heartbeat schema | vitest Zod reject | pass | _ |
| 29 | T44-T51 logged | PATTERNS.md grep | 8 items present | _ |
| 30 | bridge backward compat | requests v1 sans event still work | vitest pass | _ |

---

## 10. Git plan — ordre des commits

1. `feat(bridge): Sprint 15 Phase A — bidirectional push via sbfb-bridge-event`
2. `feat(watchdog): Sprint 15 Phase B — CPU watchdog via heartbeat + stalled overlay`
3. `feat(cli): Sprint 15 Phase C — sbfb init CLI with html/react/pyodide templates`
4. `test(bridge): Sprint 15 Phase D — Playwright iframe push + watchdog stalled`
5. `docs(sprint15): verification + audit plan for Sprint 16`

Chaque commit landed sur master apres verification complete (checklist
§9 rows pertinentes).

---

## 11. Scope cuts (copies du kickoff §6)

- Re-publish auto, branding, VPS, origin subdomain, MIME scan, multi-
  writer, custom domain -> Sprint 16+
- Dispatcher server-side events, whitelist event names -> Sprint 16+
- Templates Vue/Svelte/Jupyter, `sbfb publish` integre -> Sprint 16+
- Kill-by-force iframe -> Sprint 16+ (browser API absente)

---

## 12. Risques (R1..R6)

- **R1 — Pyodide CDN bloque par CSP blob-serve** : le CSP actuel
  autorise-t-il fetches vers cdn.jsdelivr.net ? Mitigation :
  verifier le CSP dans `crates/nexus-shell-daemon/src/http.rs` avant
  de livrer le template pyodide. Si bloque, template pyodide = CDN
  inline (Pyodide fetch bootstrap only) ou bundled. **A verifier
  Phase C debut.**
- **R2 — Playwright webServer side-car complique le setup** :
  http-server side-car peut interferer avec le vite dev server.
  Mitigation : port dedie (4210), documenter dans
  `playwright.config.ts`.
- **R3 — Bridge heartbeat alourdit le main bundle** : 30 LOC JS +
  state machine React. Size budget main = 50 KB, marge etroite.
  Mitigation : mesurer apres Phase B. Si depasse, split le watchdog
  dans un chunk lazy-loaded.
- **R4 — useBridge return shape breaking change** : Sprint 13 n'avait
  pas de return. Ajouter `{ pushEvent, watchdogState, resetWatchdog }`.
  Les appelants (BrowsedProject.tsx) n'utilisaient pas le return —
  grep `useBridge(` dans le code pour confirmer pas d'autre usage.
- **R5 — Typer entry point casse le `uv run` workflow** : ajouter
  `[project.scripts] sbfb` rend la commande dispo apres
  `pip install -e`. Dans le workflow uv, `uv run sbfb init ...` doit
  fonctionner. Mitigation : tester en Phase C.
- **R6 — Templates inclusion dans le wheel** : MANIFEST.in ou
  `package-data` doit inclure `templates/**` recursivement. Si
  oublie, `sbfb init` echouera apres install pip. Mitigation : test
  pytest qui construit le wheel et verifie le contenu (optionnel),
  sinon verification manuelle Phase C.

---

## 13. Checkpoint de cloture

1. 30/30 fail-fast checklist verts
2. 5 commits feat landed sur master (A-D + docs)
3. verification.md + audit_plan.md ecrits (commit E)
4. PATTERNS.md a jour (T44-T51)
5. Memory a mettre a jour apres fermeture (tip master + compteurs
   de tests)
