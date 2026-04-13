# {{PROJECT_NAME}}

App SBFB en React (template `react`). Scaffolde par
`sbfb init react`.

## Structure

- `src/main.tsx`, `src/App.tsx` — entree React
- `index.html` — charge `sbfb-bridge.js` depuis le shell hote
- `vite.config.ts` — `base: "./"` pour que le build fonctionne
  depuis n'importe quel chemin blob-serve
- `SBFB.json` — le manifeste. `node_id` est rempli par `sbfb init`

## Developpement

```bash
npm install
npm run dev
```

Note : en dev local (vite dev server), `/sbfb-bridge.js` n'existe
pas cote du dev server — le bridge sera un no-op postMessage. Pour
tester le bridge, deployer via un daemon SBFB.

## Publier

1. `npm run build` — genere `dist/` avec `index.html` + assets
2. Copier `SBFB.json` dans `dist/` si le build ne l'y met pas :
   `cp SBFB.json dist/`
3. Deployer `dist/` via `POST /project/deploy-from-repo` (repo
   publique) ou `POST /project/deploy` (zip prive).

### Astuce repo Git

Committer UNIQUEMENT le code source (`src/`, `index.html`, etc.)
et laisser le build se faire a la publication via un script
post-install. Ou committer `dist/` pour simplicite MVP.

## Le bridge

```tsx
const bridge = new window.SBFBBridge();

// Request/response (async)
const result = await bridge.submitTask({ prompt: "..." });
const value  = await bridge.getStorage("key");
await bridge.setStorage("key", { foo: 1 });

// Push events du hote (fire-and-forget)
const unsub = bridge.onEvent("task_result_ready", (payload) => {
  console.log("task done:", payload);
});

// Cleanup
bridge.destroy();
```
