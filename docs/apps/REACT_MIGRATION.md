# Migration d'un projet React standard vers SBFB

Guide pas a pas pour porter une app React existante (Vite, CRA,
Next export statique, etc.) sur la plateforme SBFB. La cible est
un bundle statique charge dans un iframe sandboxe, qui parle au
reseau uniquement via `window.postMessage`.

---

## 1. Comprendre les contraintes

L'app tournera dans un iframe avec :

| Restriction | Cause | Impact |
|---|---|---|
| `sandbox="allow-scripts"` sans `allow-same-origin` | origine opaque (`null`) | pas de cookies, localStorage, IndexedDB partages |
| CSP `default-src 'self'` | daemon blob-serve | pas de CDN (Google Fonts, jsDelivr, etc.) |
| CSP `connect-src 'none'` | meme | **aucun** fetch / XHR / WebSocket sortant |
| CSP `script-src 'self' 'unsafe-inline'` | meme | scripts externes bloques, inline OK |
| Chemins relatifs obligatoires | blob-serve sert depuis `/blob-serve/{hash}/...` | `base: "/"` casse tout |

**Regle unique** : tout ce qui n'est pas DOM local passe par le
bridge `postMessage`. Pas d'exception.

---

## 2. Checklist de compatibilite

Avant de commencer, faire l'inventaire de ce qui sortira de l'iframe :

- [ ] `fetch()`, `axios`, `XMLHttpRequest` → a migrer vers `bridge.submitTask` ou `getStorage`/`setStorage`
- [ ] `WebSocket`, `EventSource` → a migrer vers `bridge.onEvent`
- [ ] `localStorage`, `sessionStorage`, `IndexedDB` → a migrer vers `bridge.setStorage` / `getStorage`
- [ ] `cookie` → **impossible** (origine null). Tout state doit passer par le bridge
- [ ] Google Fonts / font CDN → inliner dans le bundle ou self-host
- [ ] Images CDN (Cloudinary, imgix) → pre-bundler ou passer par une task
- [ ] Analytics (GA, Plausible) → **impossible en connect-src none**. Si besoin, ajouter un event bridge dedie
- [ ] Sentry / Datadog → idem
- [ ] Service Workers → OK en theorie mais scope limite a l'origine null
- [ ] WebGL / WebGPU / WASM → OK (tout est local)
- [ ] Import dynamique `import("https://...")` → **bloque**, tout doit etre bundle

Si un point coche ne peut pas etre migre, l'app n'est pas compatible
SBFB en l'etat.

---

## 3. Preparation du build

### 3.1 Config Vite (recommande)

```ts
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  base: "./",                    // OBLIGATOIRE : chemins relatifs
  plugins: [react()],
  build: {
    outDir: "dist",
    assetsDir: "assets",
    rollupOptions: {
      output: {
        manualChunks: undefined, // bundle unique = zip plus simple
      },
    },
  },
});
```

Sans `base: "./"` le bundle charge ses assets depuis `/assets/...`
qui pointe vers le daemon blob-serve a la racine, pas vers
`/blob-serve/{hash}/assets/...`. Resultat : ecran blanc.

### 3.2 Config CRA / Webpack

Si le projet est en CRA, ejecter n'est pas necessaire : utiliser
`"homepage": "."` dans `package.json`. Webpack 5 : `publicPath: ""`
dans la config.

### 3.3 Next.js

Next export statique (`next export`, ou `output: "export"` en App
Router). Definir `basePath` et `assetPrefix` vides, puis ajuster
manuellement si besoin. **Next SSR n'est pas supporte** — pas de
serveur derriere l'iframe.

---

## 4. Brancher le bridge

### 4.1 index.html

Le shell SBFB sert `sbfb-bridge.js` a la racine blob-serve du zip
de l'app (le coordinator copie le SDK dans chaque zip au deploy).
Ajouter avant les modules app :

```html
<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Mon app SBFB</title>
</head>
<body>
  <div id="root"></div>
  <script src="./sbfb-bridge.js"></script>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
```

Note : `./sbfb-bridge.js` (relatif), pas `/sbfb-bridge.js`.

### 4.2 Declaration TypeScript

```ts
// src/sbfb.d.ts
declare global {
  interface Window {
    SBFBBridge: new (options?: {
      timeout?: number;
      heartbeatInterval?: number;
    }) => {
      submitTask: (payload: Record<string, unknown>) => Promise<unknown>;
      getStorage: (key: string) => Promise<unknown>;
      setStorage: (key: string, value: unknown) => Promise<unknown>;
      onEvent: (name: string, cb: (payload: unknown) => void) => () => void;
      destroy: () => void;
    };
  }
}
export {};
```

### 4.3 Hook React idiomatique

Centraliser le cycle de vie du bridge dans un hook unique :

```tsx
// src/hooks/useSBFBBridge.ts
import { useEffect, useRef } from "react";

export function useSBFBBridge() {
  const bridgeRef = useRef<InstanceType<typeof window.SBFBBridge> | null>(null);

  useEffect(() => {
    if (typeof window.SBFBBridge !== "function") {
      // Mode dev local : sbfb-bridge.js n'est pas charge
      return;
    }
    bridgeRef.current = new window.SBFBBridge();
    return () => {
      bridgeRef.current?.destroy();
      bridgeRef.current = null;
    };
  }, []);

  return bridgeRef;
}
```

Consommation :

```tsx
function MyComponent() {
  const bridge = useSBFBBridge();

  async function handleClick() {
    const result = await bridge.current?.submitTask({ prompt: "..." });
    console.log(result);
  }
}
```

**Ne jamais** instancier `new SBFBBridge()` dans le render : chaque
instance ajoute un listener `window.message` qui fuit au re-render.

---

## 5. Migrer le code

### 5.1 Requetes HTTP

**Avant**
```ts
const res = await fetch("/api/tasks", {
  method: "POST",
  body: JSON.stringify({ prompt: "..." }),
});
const data = await res.json();
```

**Apres**
```ts
const data = await bridge.current.submitTask({ prompt: "..." });
```

Le coordinator valide le payload et retourne `{ task_id, status }`.
Le resultat de la task arrive plus tard via un event push.

### 5.2 Persistance

**Avant**
```ts
localStorage.setItem("chat_history", JSON.stringify(messages));
const history = JSON.parse(localStorage.getItem("chat_history") || "[]");
```

**Apres**
```ts
await bridge.current.setStorage("chat_history", { messages });
const { messages } = await bridge.current.getStorage("chat_history");
```

Le storage SBFB est **par-app** et **par-user** (segmente par
node_id). Le coordinator le persiste dans iroh-docs (CRDT), donc
c'est partage entre devices du meme utilisateur sans code supp.

### 5.3 Temps reel

**Avant**
```ts
const ws = new WebSocket("wss://api.example.com/events");
ws.onmessage = (e) => handleEvent(JSON.parse(e.data));
```

**Apres**
```tsx
useEffect(() => {
  if (!bridge.current) return;
  const unsub = bridge.current.onEvent("task_result_ready", (payload) => {
    handleEvent(payload);
  });
  return unsub;
}, [bridge]);
```

Le host push les events `task_result_ready`, `storage_changed`,
etc. L'app s'abonne uniquement a ce qu'elle consomme. Pas de
reconnection logic a gerer : le canal postMessage est persistant
tant que l'iframe vit.

### 5.4 Assets externes

**Avant** (bloque)
```html
<link href="https://fonts.googleapis.com/css2?family=Inter" rel="stylesheet">
```

**Apres** : self-host via `@fontsource/inter` ou equivalent, ou
passer aux polices systeme.

```ts
// src/main.tsx
import "@fontsource/inter/400.css";
import "@fontsource/inter/600.css";
```

Les WOFF2 sont bundles par Vite dans `dist/assets/`.

### 5.5 Images

Images locales : `import logo from "./logo.png"` — Vite les hash
et bundle dans `assets/`. OK.

Images dynamiques d'une API : passer par une task qui retourne un
data URL (base64) ou un blob stocke via `setStorage`.

---

## 6. Mode dev local

`sbfb-bridge.js` n'existe pas sur le dev server Vite. Deux options :

**Option A — bridge no-op** (dev rapide, logique figee)
```ts
if (typeof window.SBFBBridge !== "function") {
  window.SBFBBridge = class {
    submitTask() { return Promise.resolve({ mock: true }); }
    getStorage() { return Promise.resolve(null); }
    setStorage() { return Promise.resolve({ ok: true }); }
    onEvent() { return () => {}; }
    destroy() {}
  } as any;
}
```

**Option B — deployer a chaque iteration** sur un daemon SBFB local
via `POST /project/deploy` et ouvrir l'iframe reelle. Plus fidele,
plus lent.

Recommande : A pour le dev UI, B avant chaque commit.

---

## 7. Fichiers obligatoires pour publier

A la racine du `dist/` final (ou du repo si build a la publication) :

### 7.1 `SBFB.json`

```json
{
  "node_id": "<votre_node_id_ed25519_hex>",
  "project_name": "mon-app",
  "template": "react",
  "version": "0.1.0"
}
```

Le `node_id` doit matcher le daemon local qui publie — c'est la
preuve de propriete (pattern Keyoxide). Recupere via
`GET /daemon/node_id`.

### 7.2 `index.html` a la racine

Verifie par le coordinator au clone (etape 4 de la verif Sprint 14).

---

## 8. Publication

```bash
npm run build                               # genere dist/
cp SBFB.json dist/                          # si pas deja fait
cd dist && git init && git add . && git commit -m "release"
# push vers un repo public (GitHub, GitLab, Codeberg, Gitea)

# cote daemon SBFB
curl -X POST http://127.0.0.1:8080/project/deploy-from-repo \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "https://github.com/user/mon-app",
    "commit_sha": "<sha_complet_40_chars>",
    "project_name": "mon-app",
    "visibility": "public"
  }'
```

Le coordinator clone, verifie SBFB.json, zip le contenu, signe
`provenance.json` (SLSA L1), et publie sur iroh-blobs. Cf.
`docs/VISION_USE_CASES.md` et `sprint14_keyoxide_decision.md` pour
les garanties.

Pour une app privee (zip direct sans verification repo) :
`POST /project/deploy` avec un multipart zip. Mais l'app n'apparait
que sur votre node, pas dans le browse global.

---

## 9. Anti-patterns

**A eviter** :

- `fetch("http://127.0.0.1:7000/coordinator/...")` — connect-src 'none' le bloque silencieusement. Toujours passer par le bridge.
- `new SBFBBridge()` dans le render d'un composant — fuite de listeners. Le mettre dans `useEffect`.
- Oublier `base: "./"` dans Vite — bundle blanc.
- Oublier `bridge.destroy()` a l'unmount — fuite heartbeat + listeners.
- Subscribe au meme event `onEvent` sans unsub — les callbacks s'empilent.
- Stocker des secrets en setStorage — le storage est sync CRDT, tout device de l'user y accede. Pas de secret par device.
- Pyodide chargee depuis CDN — bloque. Bundler `indexURL: "./pyodide/"` dans le zip.
- `window.open`, `location.href = "https://..."` — navigation hors iframe bloquee par le sandbox.

---

## 10. Verification post-migration

```bash
# 1. Build sans erreur
npm run build

# 2. Verifier que dist/index.html reference bien ./sbfb-bridge.js
grep "sbfb-bridge" dist/index.html

# 3. Verifier que tous les assets sont en chemins relatifs
grep -E 'src="/|href="/' dist/index.html  # doit etre VIDE

# 4. Deployer sur un daemon local + ouvrir dans le shell React
# 5. Ouvrir les devtools de l'iframe : aucune erreur CSP / no-origin
```

Si `connect-src 'none'` bloque quelque chose, la console affiche
un warning `Refused to connect to ...` — c'est la que se lit la
derniere dette de migration oubliee.

---

## Reference

- SDK : `web/public/sbfb-bridge.js`
- Protocole : `web/src/bridge/protocol.ts`
- Template officiel : `packages/nexus-coordinator/src/nexus_coordinator/templates/react/`
- Host iframe : `web/src/pages/BrowsedProject.tsx`
- Daemon blob-serve + CSP : `crates/nexus-shell-daemon-core/src/blob_serve.rs`
- Deploy verifie Sprint 14 : `.planning/sprint14_plan.md`
