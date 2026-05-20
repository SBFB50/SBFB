# 04 - Domain Smoke Matrices

But : choisir le smoke reel adapte au domaine touche. Les tests unitaires ne
suffisent pas quand le bug est dans l'integration.

## Daemon-served SPA

Utiliser quand le diff touche `web/src`, `web/dist`, `http.rs`, routes daemon,
auth token, `ServeDir`, `fallback_service`, CORS ou navigation.

Checklist :

- document navigation directe `/browse`
- document navigation directe `/curators`
- F5/reload sur `/browse`
- F5/reload sur `/curators`
- API JSON avec token : `/api/daemon/browse`
- API JSON sans token : 401 attendu
- `/auth/token` depuis shell servi par daemon
- in-app navigation vers les memes pages
- reponse JSON ne recoit pas header HTML `no-cache`
- HTML SPA recoit header anti-cache si attendu

Evidence attendue :

```bash
cd web && npm run build
node <playwright-smoke>.mjs
```

## App detail et blob iframe

Utiliser quand le diff touche `BrowsedProject`, `/blob-serve`, deploy/publish,
archive zip, CSP, COEP, CORP, CORS.

Checklist :

- projet local sans archive affiche un etat clair ;
- projet avec `archive_hash` charge `/blob-serve/{hash}/index.html` ;
- iframe body non vide ;
- console browser sans erreur critique ;
- modules ES locaux chargent ;
- images/assets locaux chargent ;
- `connect-src 'none'` conserve l'isolation ;
- `Access-Control-Allow-Origin` et `Cross-Origin-Resource-Policy` justifies.

## P2P / gossip / interconnexion

Utiliser quand le diff touche `iroh`, `GossipClient`, bootstrap peers, browse
aggregator, project announcements, blob tickets.

Checklist :

- machine A publie ;
- machine B voit l'entree dans `/api/daemon/browse` ;
- machine B peut ouvrir le detail ;
- si archive distante : B a un `archive_hash` local apres fetch ticket ;
- diagnostic neighborhood n'est pas confondu avec vrais peers gossip ;
- logs contiennent `NeighborUp` ou evidence equivalente ;
- redemarrage d'un daemon ne casse pas l'identite attendue ;
- test negatif : sans bootstrap/adresse, la limite est documentee.

## Frontend UI francais

Utiliser quand le diff touche `web/src/**/*.tsx`.

Checklist :

- pas de strings anglaises nouvelles ;
- accents francais visibles : `Réseau`, `abonnés`, `connectés`, `détails` ;
- texte ne deborde pas sur mobile ;
- boutons iconiques quand pertinent ;
- pas de nouvelle landing page si l'app doit etre directement utile ;
- screenshot ou test DOM si layout modifie.

Commande :

```bash
cd web
bash scripts/scan-en-strings.sh
npm run lint
npm run test:unit
```

## Security / protocol

Utiliser quand le diff touche canonical, schema, signature, provenance, consent,
bridge, sandbox, CORS, CSP, body limit, path traversal.

Checklist :

- threat model lu ou grep cible ;
- `Security delta` present in review and in body section `## Codex verification` ;
- `Pre-launch protocol` present ;
- aucun `*_VERSION` modifie sans S4 ;
- canonical bytes inchanges ou decision explicite ;
- tests negatifs pour bypass auth/origin/path ;
- body limits justifies par threat model et UX ;
- wildcard CORS justifie par publicness + sandbox.

## Rust/Python migration

Utiliser quand le diff remplace ou contourne du Python par Rust.

Checklist :

- route Python legacy identifiee ;
- route Rust cible documentee ;
- compat ou rupture explicite ;
- tests frontend mis a jour ;
- anciens chemins ne retournent pas HTML parse comme JSON ;
- migration ne change pas le trust boundary.
