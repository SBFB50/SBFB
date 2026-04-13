# {{PROJECT_NAME}}

App SBFB minimale (template `html`). Scaffolde par `sbfb init html`.

## Structure

- `index.html` — l'entree de l'app. Charge `sbfb-bridge.js` depuis
  le shell hote et demontre l'API (submitTask + onEvent).
- `SBFB.json` — le manifeste SBFB. Le champ `node_id` doit matcher
  le node_id du daemon qui publiera l'app (preuve de propriete
  via pattern Keyoxide). `sbfb init` rempli ce champ automatiquement
  depuis le daemon local.

## Tester en local

L'app peut se tester sans daemon — le bridge postMessage echoue
silencieusement quand il n'y a pas d'hote. Ouvrir simplement :

```
python -m http.server 8080
```

puis visiter `http://localhost:8080/` dans le browser.

Pour tester le bridge complet, deployer via un daemon SBFB qui
ouvre le shell React en tant qu'hote.

## Publier

### Via repo Git (recommande, public, verifie)

1. `git init && git add . && git commit -m "initial commit"`
2. Pousser vers un repo public (GitHub / GitLab / Codeberg / Gitea)
3. Appeler `POST /project/deploy-from-repo` sur le coordinateur
   avec `{"repo_url": "https://github.com/user/repo"}`. Le
   coordinateur cloiclone, verifie SBFB.json, signe provenance,
   deploie.

### Via upload zip (prive)

1. `zip -r my-app.zip index.html SBFB.json README.md`
2. `POST /project/deploy` avec le zip en multipart

## Ressources

- [SBFB Bridge SDK](https://github.com/veffix/nexus-grid/blob/main/web/public/sbfb-bridge.js)
- Bridge methods : `submitTask`, `getStorage`, `setStorage`, `onEvent`
