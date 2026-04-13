# {{PROJECT_NAME}}

App SBFB avec Pyodide (template `pyodide`). Scaffolde par
`sbfb init pyodide`.

## Important : bundle Pyodide requis

Le daemon blob-serve injecte un CSP strict
(`default-src 'self' ...; connect-src 'none'`) qui **bloque les
fetches externes**. Pyodide ne peut donc PAS etre charge depuis un
CDN — il doit etre present dans le zip deploye.

### Telecharger Pyodide 0.29.3

Avant la premiere publication :

```bash
# Option 1 : via npm (recommande)
npm install pyodide@0.29.3
mkdir -p pyodide
cp -r node_modules/pyodide/* pyodide/

# Option 2 : telechargement direct
mkdir -p pyodide
curl -L https://github.com/pyodide/pyodide/releases/download/0.29.3/pyodide-0.29.3.tar.bz2 \
  | tar -xj --strip-components=1 -C pyodide
```

Apres cette etape, `./pyodide/` contient `pyodide.js`,
`pyodide.asm.wasm`, `python_stdlib.zip`, etc. Taille totale
~40 MB. Ajoute `pyodide/` a git si tu veux que le repo soit
deploy-ready ; sinon committe juste un script `setup.sh` qui
re-telecharge.

## Tester en local

```bash
python -m http.server 8080
```

Visite `http://localhost:8080/`. La page chargera Pyodide depuis
`./pyodide/` et permettra d'executer du Python.

## Publier

Voir `packages/nexus-coordinator/templates/html/README.md`. Le flux
est identique : `POST /project/deploy-from-repo` pour public ou
`POST /project/deploy` pour prive.

## Limitations

- Pas de `fetch()` sortant (CSP `connect-src 'none'`) — les
  packages Python doivent etre dans le bundle Pyodide
- Pas de SharedArrayBuffer (origin blob-serve n'a pas
  `COOP/COEP`) — certains optimizations Pyodide sont indisponibles
- Taille du zip ~40 MB min (Pyodide seul). Max actuel = 100 MB

## Le bridge

Identique au template html : `bridge.submitTask`, `bridge.onEvent`,
etc. Le Python peut appeler le bridge via
`pyodide.runPython('from js import bridge; bridge.submitTask(...)')`.
