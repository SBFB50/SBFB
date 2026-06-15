# Checklist de test PO — etat post-S75 + UX-ARRIVAL

> Cible : HEAD `8dfb4f7` (S75 decouverte PULL node-centrique + ancre VPS,
> mini-cycle UX-ARRIVAL, hotfix outbox auto-purge).
> Materiel reel : PC Windows (noeud auteur), Mac (pair), VPS Hetzner (ancre).
> La plupart des tests se font en SOLO sur le PC. La section 2 marque ceux
> qui exigent 2+ machines.

---

## 0. Pre-requis : lancer le systeme

### 0.1 Build (une fois, depuis la racine du repo)

```powershell
# Daemon (obligatoire)
cargo build -p nexus-shell-daemon --release
# Launcher (spawn daemon + ouvre le navigateur + tray) — recommande
cargo build -p nexus-launcher --release
# Factory CLI (pour les tests de publication ligne de commande)
cargo build -p sbfb-factory --release
# Shell web (sert l'UI React ; le launcher cherche web/dist/)
cd web ; npm install ; npm run build ; cd ..
```

Binaires produits : `target\release\nexus-shell-daemon.exe`,
`target\release\nexus-launcher.exe`, `target\release\sbfb-factory.exe`.

### 0.2 Run (deux options)

**Option A — launcher (le plus simple, comme un utilisateur final)**

```powershell
.\target\release\nexus-launcher.exe
```

Le launcher : spawn `nexus-shell-daemon start` en enfant, attend que le
daemon ait ecrit `running.json`, ouvre le navigateur sur l'URL du shell,
affiche une icone tray (clic droit -> menu, Quit pour arreter proprement).

**Option B — daemon seul (pour piloter en curl/PowerShell)**

```powershell
.\target\release\nexus-shell-daemon.exe start
```

### 0.3 Trouver l'URL du shell + le port

Le port HTTP est **ephemere par defaut** (l'OS en choisit un libre a
chaque boot). Le port reel est ecrit dans `running.json` juste apres le
bind. Sur Windows :

```powershell
Get-Content "$env:APPDATA\nexus-grid\shell-daemon\running.json"
# -> {"api_host":"127.0.0.1","api_port":<PORT>,"schema_version":...}
```

URL du shell : `http://127.0.0.1:<PORT>` (le launcher l'ouvre tout seul).
Logs : `$env:APPDATA\nexus-grid\logs\daemon.log` (rotation journaliere).

### 0.4 Auth (obligatoire pour tout test curl/PowerShell)

Toutes les routes sauf `GET /health`, `GET /auth/token`,
`GET /blob-serve/...` exigent l'en-tete `X-SBFB-Token` (hex 64 car.) **et**
Host loopback (127.0.0.1 ou ::1) **et** Origin absent ou loopback.

Recuperer le token (route publique, checks Host+Origin seuls) :

```powershell
$base = "http://127.0.0.1:<PORT>"
$tok  = (Invoke-RestMethod "$base/auth/token").token
# Toutes les requetes authentifiees ensuite :
$H = @{ "X-SBFB-Token" = $tok }
Invoke-RestMethod "$base/api/daemon/info" -Headers $H
```

(Le token vit aussi dans `%USERPROFILE%\.sbfb\auth_token`, mais passer
par `GET /auth/token` est la voie testee par le shell.)

### 0.5 Arreter le daemon

- Via launcher : tray -> Quit, ou Ctrl+C dans la console du launcher
  (TerminateProcess de l'enfant sur Windows).
- Daemon seul : lire le PID via `running.json` puis
  `taskkill /PID <pid> /F`, ou supprimer
  `$env:APPDATA\nexus-grid\shell-daemon\running.json` (marqueur singleton).

### 0.6 Pieges connus (a lire AVANT de tester)

- **Verrouillage de l'exe release** : si un daemon tourne deja, le
  rebuild echoue (`os error 5` / fichier verrouille). Arreter le daemon
  AVANT `cargo build --release`.
- **Tuer par pattern complet** : sur Windows, fermer la fenetre du
  launcher ne tue pas toujours l'enfant ; preferer `taskkill /F` sur le
  PID de `running.json`. (Sur Mac/Linux, `pkill -f nexus-shell-daemon`
  — PAS `pkill -f 'nexus-shell-daemon start'` qui ne matche pas la
  cmdline `--config X start`, sinon le port reste pris silencieusement.)
- **`stop.bat` est PERIME** : il vise les anciens ports 8000/3002 +
  `docker compose down` (NEXUS cold-case). Ne PAS l'utiliser pour le
  daemon Rust actuel.
- **Singleton strict** : un seul daemon a la fois. Si `running.json`
  existe d'un crash precedent, le nouveau boot peut refuser de bind ;
  supprimer le marqueur orphelin.
- **Solo : ne pas confondre "vide" et "casse"** : un PC seul sans
  abonnement ne verra rien dans Browse tant qu'il n'a pas publie une app
  ou ajoute une ancre. C'est normal, pas une panne (cf. cold-start).

---

## 1. Parcours a tester (le coeur)

Format constant : Titre / Etapes / Resultat attendu / Ce que ca prouve /
Statut (DEVRAIT MARCHER, PARTIEL, CONNU CASSE).

### Categorie Baseline (fonctionne depuis longtemps, solo PC)

---

**B1. Le systeme demarre et repond**

- Etapes :
  ```powershell
  .\target\release\nexus-launcher.exe
  # puis dans une autre console :
  $base = "http://127.0.0.1:<PORT>"   # PORT depuis running.json
  Invoke-RestMethod "$base/health"
  ```
- Resultat attendu : navigateur ouvert sur le shell ; `/health` renvoie
  `{status, schema_version, daemon_version}` HTTP 200.
- Ce que ca prouve : build + boot + bind + singleton + serveur HTTP OK.
- Statut : DEVRAIT MARCHER

---

**B2. Le shell charge et redirige vers Mes projets**

- Etapes : ouvrir `http://127.0.0.1:<PORT>/` dans le navigateur.
- Resultat attendu : redirection vers `/my-projects`. Si aucun noeud
  connu -> ecran OnboardingEmpty (Step 1 "Demarre le daemon" +
  Step 2 "Se connecter a un noeud"). Le daemon same-origin s'auto-
  enregistre via `bootstrap.ts`, donc en pratique on voit la page
  Projects avec le noeud local en carte "En ligne".
- Ce que ca prouve : React shell servi (web/dist), bootstrap same-origin,
  routing.
- Statut : DEVRAIT MARCHER

---

**B3. Etat du noeud (identite + reseau)**

- Etapes : naviguer vers `/my-network` dans le shell. En parallele :
  ```powershell
  Invoke-RestMethod "$base/api/daemon/info" -Headers $H
  ```
- Resultat attendu : page My Network avec IdentityCard (node_id, version,
  uptime), GpuCard, ProjectsServedCard, LastTaskCard ; polling 2s.
  `/api/daemon/info` renvoie le DaemonStateSnapshot.
- Ce que ca prouve : identite stable, snapshot daemon, auth header.
- Statut : DEVRAIT MARCHER

---

**B4. Publier une app depuis un repo Git (deploy verifie)**

- Etapes (shell) : aller sur `/deploy`, remplir 3 champs
  (repo_url HTTPS public, project_name, description),
  cliquer "Publier sur le reseau". Repo de reference :
  `https://github.com/SBFB50/SBFB.git` (contient examples/sbfb-ideas).
  Variante curl :
  ```powershell
  $body = @{ repo_url="https://github.com/SBFB50/SBFB.git"
             project_name="ideas-test"; category="community"
             description="test"; apps=@() } | ConvertTo-Json
  Invoke-RestMethod "$base/api/v1/deploy-from-repo" -Method Post `
    -Headers $H -ContentType "application/json" -Body $body
  ```
- Resultat attendu : carte succes "App publiee et en ligne", badge signal,
  liens "Voir la fiche" + "Details techniques" (hash/provenance/commit).
  Cote pipeline : clone git -> verif manifest -> signature Ed25519 +
  BLAKE3 -> provenance.json -> ProjectAnnouncement broadcast gossip.
- Ce que ca prouve : deploy from source, provenance SLSA L1 auto-attestee,
  diffusion gossip locale.
- Statut : DEVRAIT MARCHER (exige reseau internet pour clone HTTPS)

---

**B5. L'app publiee apparait dans Browse et s'ouvre**

- Etapes : apres B4, aller sur `/browse`. L'app doit etre dans la grille
  "Tes sources" (is_own=true). Cliquer la carte -> `/browse/:projectId`.
- Resultat attendu : la fiche s'ouvre en plein ecran ; l'iframe charge
  l'app via `/blob-serve/<archive_hash>/index.html` ; status dot
  reachable (vert). Header auto-hide (revele si souris < 48px du haut).
- Ce que ca prouve : aggregator browse (source=direct), daemon blob-serve,
  rendu iframe sandboxe, content-addressing local.
- Statut : DEVRAIT MARCHER

---

**B6. Recherche plein texte (FTS5 local)**

- Etapes (shell) : sur `/browse`, taper un mot du nom/description de l'app
  publiee dans la barre de recherche. Variante curl :
  ```powershell
  Invoke-RestMethod "$base/api/daemon/search?q=ideas&limit=20" -Headers $H
  ```
- Resultat attendu : SearchResultsView avec SearchHitCard, badges
  "Source verifiable"/"P2P"/"Provenance" selon le cas ; lien "Source".
- Ce que ca prouve : index FTS5 a chaud, route search bornee (q <= 1024o,
  limit <= 100, offset <= 10000).
- Statut : DEVRAIT MARCHER (recherche LOCALE seulement, cf. section 3)

---

**B7. Les apps d'exemple chargent via bridge postMessage**

- Etapes : publier/ouvrir `examples/sbfb-explorer` (bridge :
  node_status, identity_pubkey, browse_list, provenance_verify) puis
  `examples/sbfb-ideas` (storage_get/set/list/delete, identity_pubkey).
  Dans l'app, declencher les actions UI qui appellent le bridge.
- Resultat attendu : l'Explorer affiche le statut du noeud / la pubkey ;
  Ideas Hub permet de proposer/voter une idee (stockage local).
- Ce que ca prouve : bridge postMessage whitelist, iframe <-> daemon,
  storage par app.
- Statut : DEVRAIT MARCHER (storage Ideas non partage cross-noeud, cf. §3)

---

### Categorie S75-PULL (decouverte node-centrique + ancre)

---

**P1. Lister les annuaires de noeuds (route /nodes)**

- Etapes :
  ```powershell
  Invoke-RestMethod "$base/api/daemon/nodes" -Headers $H | ConvertTo-Json -Depth 5
  ```
  Shell : aller sur `/nodes`.
- Resultat attendu : enveloppe `{nodes:[...], observed:[...]}`. `nodes` =
  publishers d'annuaire abonnes (avec node_id, revision, app_count,
  catalog). `observed` = noeuds entendus sur gossip SANS abonnement
  (node_id + last_seen seulement). Solo frais : tout vide (cold-start ->
  CTA "Ajouter une ancre"). Apres B4, ton propre annuaire peut apparaitre.
- Ce que ca prouve : route additive /nodes (S75-D), separation
  abonnes/observes (UX-ARRIVAL).
- Statut : DEVRAIT MARCHER

---

**P2. S'abonner a une ancre par cle (verrou 3 : action explicite)**

- Etapes (shell) : `/nodes` -> "Ajouter une ancre" (AddAnchorDialog) ->
  coller une identite 64 hex (celle d'un autre noeud, ex. le VPS) ->
  S'abonner. Variante curl (l'ancre reutilise le mecanisme curator) :
  ```powershell
  $b = @{ curator_pubkey_hex="<64hex>" } | ConvertTo-Json
  Invoke-RestMethod "$base/api/daemon/curators/subscribe" -Method Post `
    -Headers $H -ContentType "application/json" -Body $b
  ```
- Resultat attendu : l'abonnement est ajoute (idempotent) ; apres
  reception gossip, le catalogue de l'ancre apparait dans /nodes ->
  cliquable vers /node/:nodeId. Le champ identite n'est JAMAIS prerempli
  par defaut (verrou 3) sauf clic explicite "S'abonner" sur un noeud
  observe.
- Ce que ca prouve : abonnement explicite, ingestion d'annuaire gated,
  pas d'auto-trust.
- Statut : DEVRAIT MARCHER (le remplissage du catalogue depend du gossip,
  cf. cross-machine §2 ; en solo le catalogue restera "en attente")

---

**P3. Parcourir le catalogue d'un noeud abonne**

- Etapes (shell) : `/nodes` -> cliquer un NodeRow d'un noeud abonne ayant
  annonce un catalogue -> `/node/:nodeId`.
- Resultat attendu : grille CatalogCard (dedup par project_id+
  archive_hash). Badge "Source verifiable" UNIQUEMENT si annonce editeur
  directe + is_open_source=true (jamais sur un simple listing annuaire,
  qui hardcode is_open_source=false). CTA "Ouvrir" -> /browse/:projectId.
- Ce que ca prouve : annuaire = source de DECOUVERTE (pas autorite),
  marquage provenance honnete (verrou 4).
- Statut : DEVRAIT MARCHER (exige un noeud distant ayant un catalogue =
  cross-machine ; cf. §2)

---

**P4. Persistance de la decouverte (locator anti-rollback)**

- Etapes : apres P2/P3 avec un catalogue ingere, redemarrer le daemon
  (arret + relance). Re-verifier `/nodes`.
- Resultat attendu : au boot, `repull_directories()` re-fetch les
  annuaires des ancres abonnees (timeout 15s/ancre) via le locator
  `anchors.json` persiste ; floor anti-rollback : une revision plus
  ancienne ne remplace pas. Le catalogue revient sans re-abonnement.
- Ce que ca prouve : durabilite catalogue distant, locator persiste,
  anti-rollback.
- Statut : DEVRAIT MARCHER (cross-machine pour avoir un vrai annuaire
  distant ; cf. §2)

---

**P5. Pull multi-provider ancre-d'abord d'une app injoignable directement**

- Etapes : ouvrir une app dont le ticket direct n'est plus joignable mais
  presente dans un annuaire abonne (scenario survives-VPS-death : auteur
  hors-ligne, ancre VPS en ligne). Cliquer "Ouvrir".
- Resultat attendu : `fetch_hash_multi` telecharge le blob par hash nu,
  ordonne ancre-d'abord (cap MAX_FETCH_PROVIDERS=16 dans la primitive),
  pkarr resout l'adresse ; l'app rend (HTTP 200) meme auteur down.
- Ce que ca prouve : pull multi-provider, decouplage heberger != publier,
  content-addressing comme verite de joignabilite.
- Statut : DEVRAIT MARCHER cross-machine (prouve LIVE a l'acceptance
  S75) ; PARTIEL en solo (pas de 2e provider). Voir aussi PULL-3 §3.

---

### Categorie UX-ARRIVAL (arrivee d'un pair frais)

---

**U1. Section "Decouvert sur le reseau" separee de "Tes sources"**

- Etapes (shell) : `/browse`. Observer les deux zones.
- Resultat attendu : grille principale "Tes sources" = apps is_own OU
  curator/nodedirectory OU from_subscribed (catalog-backed). Section
  "Decouvert sur le reseau" = ambiant gossip NON sollicite, cappee a 24,
  jamais melangee a la grille principale, jamais en hero. Dedup par
  (project_id, archive_hash).
- Ce que ca prouve : decision PO C-hybride (mes sources vs ambiant),
  anti-pollution de la grille principale.
- Statut : DEVRAIT MARCHER (la section "Decouvert" se remplit seulement
  si des annonces ambiantes arrivent = cross-machine pour la voir non
  vide ; vide en solo est normal)

---

**U2. Noeuds observes (entendus sans abonnement) + CTA S'abonner**

- Etapes (shell) : `/nodes`, section ObservedSection.
  ```powershell
  (Invoke-RestMethod "$base/api/daemon/nodes" -Headers $H).observed
  ```
- Resultat attendu : liste de node_id + last_seen pour les annuaires
  entendus sur gossip sans abonnement (metadata seule, blob du catalogue
  JAMAIS fetche pour une annonce non sollicitee). CTA "S'abonner" ouvre
  AddAnchorDialog PRE-REMPLI avec ce node_id (seul cas de prefill,
  verrou 3).
- Ce que ca prouve : surface observed RAM-only bornee (cap 256 + eviction
  stalest + TTL 48h + rate-limit), NO-FETCH/NO-DIAL, self-guard (jamais
  notre propre node_id).
- Statut : DEVRAIT MARCHER cross-machine ; en solo observed reste vide
  (rien a entendre). NB : observed RAM-only -> vide apres reboot (§3).

---

**U3. Toggle "Garder en ligne" reconcilie (pas de faux ON/OFF)**

- Etapes (shell) : sur `/node/:nodeId` ou via AvailabilitySheet d'une app
  to a soi, basculer "Garder en ligne" (SupportButton / toggle isOwn).
  Re-ouvrir la fiche / rafraichir.
- Resultat attendu : l'etat du toggle reflete `self_pin_enabled` (3 etats)
  reconcilie via la query seedCount, sans faux positif apres refresh.
  Precedence echo > intent > defaut-ON.
- Ce que ca prouve : keep-online M18, reconciliation WEB-1.
- Statut : DEVRAIT MARCHER (transitoire possible si meme project_id avec
  hash differents en browse, cf. DETTE-13 §3 — logique OK, pedagogie a
  affiner)

---

### Categorie Disponibilite (panneau S74-A)

---

**D1. Ouvrir le panneau Disponibilite d'une app**

- Etapes (shell) : `/browse/:projectId` -> CTA "Disponibilite"
  (ouvre AvailabilitySheet).
- Resultat attendu : 3 sections scellees : AUTEUR (immuable, qui a signe),
  ETAT (sonde reachable/unreachable/checking, CTA "Reverifier"),
  QUI LA GARDE EN LIGNE (seeders + toggle/soutien). Les "COPIES DE
  SECOURS" sont inertes "Bientot" (NF-2, jamais faux bouton actif).
- Ce que ca prouve : panneau Disponibilite, separation auteur/etat/seeders,
  pas de host field (verrou 8).
- Statut : DEVRAIT MARCHER

---

**D2. Reverifier l'etat (sonde) d'une app**

- Etapes (shell) : dans AvailabilitySheet, section ETAT -> "Reverifier"
  (declenche browse/pull). Variante curl :
  ```powershell
  Invoke-RestMethod "$base/api/daemon/browse/pull" -Method Post -Headers $H
  ```
- Resultat attendu : le status dot passe checking puis reachable/
  unreachable selon la sonde reelle. /browse/pull broadcast un
  browse_request gossip et retourne immediatement.
- Ce que ca prouve : sonde de joignabilite live (pas un compteur fige).
- Statut : DEVRAIT MARCHER

---

**D3. Compteur de seeds "Toi + N pairs" (best-effort)**

- Etapes :
  ```powershell
  Invoke-RestMethod "$base/api/daemon/seed-count/<project_id>?archive_hash=<hash>" -Headers $H
  ```
- Resultat attendu : `{peer_count, self_seeding, self_pin_enabled}`.
  Best-effort : peut sur-estimer, ne sert jamais d'octets absents.
  En solo : peer_count=0 attendu.
- Ce que ca prouve : compteur disponibilite best-effort, version-exacte
  par (pid,hash).
- Statut : PARTIEL (le compteur cross-noeud ne converge PAS de maniere
  fiable, cf. DETTE-1 / SeedAnnounced §3 ; le champ existe et repond,
  mais ne croit pas en pratique cross-machine)

---

**D4. Soutenir une app distante (seed volontaire)**

- Etapes (shell) : sur une app PAS a soi, AvailabilitySheet ->
  "Soutenir ce projet". Variante curl :
  ```powershell
  $b = @{ project_id="<pid>"; archive_hash="<hash>" } | ConvertTo-Json
  Invoke-RestMethod "$base/api/daemon/seed" -Method Post -Headers $H `
    -ContentType "application/json" -Body $b
  ```
- Resultat attendu : le noeud acquiert l'archive (ticket direct OU
  multi-provider par hash) et la pin sous tag keep-online (skip-GC).
  Solo : marche seulement si l'app est joignable d'une source.
- Ce que ca prouve : seed volontaire communautaire, pin local.
- Statut : DEVRAIT MARCHER cross-machine (auteur en ligne ou annuaire) ;
  PARTIEL solo.

---

**D5. Inviter une ancre a seeder (invite lie au contenu)**

- Etapes :
  ```powershell
  # Minter une invite liee a l'app
  $b = @{ project_id="<pid>"; expires_in_secs=86400; max_uses=1 } | ConvertTo-Json
  $inv = Invoke-RestMethod "$base/api/daemon/seed/invite" -Method Post `
    -Headers $H -ContentType "application/json" -Body $b
  # Demander a un pair ancre de seeder (requester leg)
  $r = @{ peer_node_id="<endpoint hex>"; project_id="<pid>"
          invite_token=$inv.token } | ConvertTo-Json
  Invoke-RestMethod "$base/api/daemon/seed/request" -Method Post `
    -Headers $H -ContentType "application/json" -Body $r
  ```
- Resultat attendu : invite `{token, expires_at, max_uses}` liee a la
  paire (project_id, archive_hash) visible. La requete seed/request
  valide que le pair existe dans browse, que l'app a un archive, que
  pair != self ; invite TOUJOURS requise.
- Ce que ca prouve : protocole sbfb/seed/0 authentifie (Ed25519 + JCS),
  invite M19 liee au contenu.
- Statut : DEVRAIT MARCHER cross-machine (exige un vrai peer_node_id
  joignable) ; PARTIEL solo (pas de pair a designer).

---

### Categorie Compute (taches + worker)

---

**C1. Etat du worker / GPU**

- Etapes (shell) : `/my-network` (polling 2s) ; ou
  ```powershell
  Invoke-RestMethod "$base/api/v1/worker/state" -Headers $H
  ```
- Resultat attendu : GpuCard (VRAM, utilisation %, temp, power),
  ConsentBadge (niveaux L1-L4 GPU). Banner si stalled/offline.
- Ce que ca prouve : monitoring worker, consentement GPU 4 niveaux.
- Statut : DEVRAIT MARCHER (worker local ; valeurs GPU dependent d'Ollama/
  driver actifs)

---

**C2. Lister/soumettre une tache au coordinateur**

- Etapes :
  ```powershell
  Invoke-RestMethod "$base/api/v1/tasks?limit=20" -Headers $H
  ```
- Resultat attendu : liste des taches (vide si aucune). Submit via
  `/api/v1/tasks/submit` puis poll `/api/v1/tasks/{id}` et
  `/api/v1/tasks/{id}/result`.
- Ce que ca prouve : pipeline coordinateur dispatch/result.
- Statut : DEVRAIT MARCHER pour lister ; l'execution reelle E2E depend
  d'un worker enrole (cross-process). PARTIEL en pur solo sans worker.

---

### Categorie Fork (atelier)

---

**F1. Forker une app dans l'atelier**

- Etapes (shell) : `/browse/:projectId` d'une app ayant archive_hash OU
  repo_url HTTPS -> CTA "Forker dans l'atelier".
- Resultat attendu : la primitive fork reconstruit un workspace cible
  (forge-clone si repo, sinon blob-reconstruct depuis l'archive) avec
  garde-fous (anti git-arg-injection, anti zip-slip/bomb,
  MAX_ARCHIVE_ENTRIES=4096).
- Ce que ca prouve : atelier fork, invariant open-source => provenance.
- Statut : DEVRAIT MARCHER

---

**F2. Redeployer un fork local re-signe**

- Etapes : apres edition du workspace forke, POST le ZIP :
  ```powershell
  Invoke-RestMethod "$base/api/v1/deploy-workspace" -Method Post `
    -Headers $H -InFile workspace.zip -ContentType "application/zip"
  ```
- Resultat attendu : redeploy local re-signe (Ed25519 du noeud), nouvelle
  archive + provenance, annonce sur le reseau ; marquee "Version derivee"
  cote front si annonce editeur source=direct match exact.
- Ce que ca prouve : boucle fork->edit->redeploy, re-signature locale.
- Statut : DEVRAIT MARCHER (ZIP <= 100 MB)

---

## 2. Tests solo (PC seul) vs cross-machine (PC + Mac + VPS)

**Testables en SOLO sur le PC** (rien d'autre requis) :
- B1-B7 (boot, shell, publish from repo, browse, search, bridge apps)
- P1 (route /nodes), P2 (s'abonner — l'abonnement marche, mais le
  catalogue restera "en attente" sans pair distant)
- U1 (la grille split s'affiche ; section "Decouvert" vide est normale)
- U3 (toggle keep-online sur une app a soi)
- D1, D2, D3 (panneau, sonde, compteur=0 attendu)
- F1, F2 (fork + redeploy local)
- C1 (etat worker local)

**Exigent 2+ machines** (PC auteur + au moins une autre) :
- P3 (catalogue d'un noeud distant) — besoin d'un noeud ayant publie un
  catalogue.
- P4 (persistance du catalogue distant au reboot) — besoin d'un annuaire
  distant a re-pull.
- P5 (pull multi-provider ancre-d'abord) — scenario survives-VPS-death :
  PC publie, VPS ancre seede, PC auteur down, Mac frais pull via VPS.
  C'est le test phare S75, prouve LIVE a l'acceptance.
- U1 non-vide (section "Decouvert sur le reseau" peuplee par l'ambiant
  d'un autre noeud).
- U2 (noeuds observes) — besoin d'un autre noeud emettant un annuaire
  sans qu'on soit abonne.
- D4 cross-noeud (seed volontaire d'une app distante reellement
  acquise d'un pair).
- D5 (seed/request a une vraie ancre joignable).
- C2 E2E (execution de tache par un worker enrole, potentiellement sur
  une autre machine).

**Topologie de reference cross-machine** (acceptance S75) :
- PC Windows = noeud auteur (publie l'app).
- VPS Hetzner = ancre headless (systemd, config seed pour garder l'app
  en ligne ; `[seed]` defaut VIDE, ancre via abonnement explicite).
- Mac = pair frais (s'abonne au VPS, decouvre l'annuaire, pull par hash).
- Piege cross-machine : configurer les ancres AVANT le boot du pair frais
  (un subscribe POST-boot ne re-join PAS le gossip bootstrap fige au
  runtime ; re-demarrer le daemon apres avoir ajoute l'ancre).

---

## 3. NE PAS TESTER / connu non-livre (section honnete)

Ces comportements sont documentes comme casses, differes ou best-effort.
Le symptome decrit est ATTENDU — ce n'est pas un bug a signaler.

- **SeedAnnounced ne converge pas cross-noeud (DETTE-1)** : apres un pin
  cross-machine, `/api/daemon/info` (et donc le compteur de D3) reste a
  `peer_count:0` ~10 min, voire indefiniment, car personne ne suit le
  feed du seeder. Best-effort par design, route audit S76. Ne pas
  attendre que le compteur croisse.

- **L'annuaire d'un seeder n'annonce pas ce qu'il seede (DETTE-2)** : un
  VPS qui seede une app d'autrui publie `catalog_len:0` (un annuaire ne
  liste que les apps blob-detenues en propre, verrou 4). Consequence : un
  pair frais qui n'a QUE l'ancre-seeder comme abonnement ne peut pas
  DECOUVRIR l'app via l'annuaire (il faut l'annonce de l'editeur).
  Question design PO, non resolue.

- **Failover cross-tier absent (PULL-3, DETTE-3)** : si le ticket direct
  est mort, le fetch NE BASCULE PAS automatiquement vers
  directory -> multi-provider en plein vol ; l'app reste inaccessible
  jusqu'a intervention. La chaine est ordonnee (direct -> SeedRegistry
  -> annuaires abonnes) mais sans rebascule croisee. Symptome : timeout
  puis echec, pas de retry sur un autre tier.

- **Re-drive-on-ingest absent / fenetre morte 1er boot (DETTE-5)** : si
  `keep_online_projects=[hash]` est configure mais qu'aucun annuaire
  couvrant ce project_id n'a encore ete ingere au passage du driver
  one-shot, l'app est "skipped" ("configured app not resolvable yet").
  Remede operateur : redemarrer le daemon APRES que le gossip a ingere
  l'annuaire. Ne pas signaler le skip comme un bug.

- **Routes seed dynamiques sans gate duress (DETTE-4)** : en mode duress
  avec data-root partage, `/seed/request`, keep-online, reannounce
  s'executent sans duress-gate (lot dette coherent S76). Ne PAS tester
  les routes seed dynamiques en mode duress avec data-root partage.

- **Surface observed RAM-only (DETTE-10)** : `/nodes` `observed[...]` est
  en memoire seule ; vide apres reboot du daemon, puis se re-remplit via
  gossip. Attendu, non persiste.

- **Rate-limit observed sur identite non signee (DETTE-11)** : le node_id
  observe est un claim nu (PoW protege le topic, pas le payload). Un
  flood de fausses identites peut remplir le registre observed (cap 256)
  et evincer des hints honnetes. Classe residuelle fresh-flood assumee
  (publisher-binding route S76). Ne pas tester comme une faille.

- **sbfb-ideas n'est PAS partage cross-noeud (DETTE-12)** : seul le
  namespace sbfb-ideas est route en iroh-docs (whitelist hardcodee).
  Toute autre app (ou un fork) utilise un storage LOCAL (HashMap+SQLite),
  PAS de replication P2P. Ne pas attendre qu'un vote dans Ideas Hub
  apparaisse sur un autre noeud pour une app custom.

- **Known_entry_count double-compte (DETTE-6)** : `known_browse_entries`
  peut sur-compter (curator + nodedirectory pour la meme app). Best-effort
  accepte ; aucun chemin ne traite ce compteur comme autorite.

- **Boot multi-ancre sequentiel ~15s/ancre (DETTE-8)** : avec N ancres
  abonnees, le re-pull au boot est sequentiel (cumul N x 15s timeout).
  Borne pilote OK ; ne pas attendre un re-pull parallelise.

- **Recherche federee cross-noeud NON livree (DETTE-14, SearchManifest)** :
  la recherche est LOCALE (FTS5) seulement. Aucune query n'est propagee
  aux pairs. Differe (sign-off PO). Ne pas chercher a "trouver l'app d'un
  autre noeud" via la barre de recherche.

- **GPU partage cross-machine NON livre (DETTE-17)** : S75 ne livre QUE la
  decouverte PULL. Le partage GPU cross-machine est S76. Sharding = S77.

- **GC reaper budget disque (DETTE-7)** : pas de LRU/reaper d'eviction
  force ; seule la policy config borne le budget. Les blobs re-pulles d'un
  annuaire distant ne sont pas pin skip-GC et peuvent etre collectes.

---

## 4. Ordre de test recommande (du plus simple au plus complexe)

1. **B1, B2, B3** — boot + shell + etat noeud (sanity, 5 min).
2. **B4 -> B5** — publier une app depuis un repo, la voir dans Browse,
   l'ouvrir (la boucle produit centrale, solo).
3. **B6, B7** — recherche locale + apps d'exemple via bridge.
4. **F1 -> F2** — forker puis redeployer (solo, prouve l'atelier).
5. **D1, D2, D3** — panneau Disponibilite, sonde, compteur (=0 attendu
   solo).
6. **P1, P2** — route /nodes + s'abonner a une ancre (solo : abonnement OK,
   catalogue "en attente").
7. **U1, U3** — grille split UX-ARRIVAL + toggle keep-online (solo).
8. *(passer en cross-machine — demarrer Mac + VPS)*
9. **P3, P4** — catalogue distant + persistance au reboot.
10. **U2, U1-non-vide** — noeuds observes + section "Decouvert" peuplee.
11. **D4, D5** — seed volontaire + invite/request a une ancre.
12. **P5** — pull multi-provider ancre-d'abord (survives-VPS-death,
    le test phare : PC publie -> VPS seede -> PC down -> Mac pull via VPS
    -> render 200).
13. **C1, C2** — etat worker puis (si worker enrole) execution E2E.

Astuce : ne passer en cross-machine qu'apres avoir valide toute la colonne
solo. La majorite des regressions visibles se voient deja en solo (boot,
publish, browse, fork, panneaux, grille). Le cross-machine ne sert qu'a
valider la decouverte PULL et le seed — et c'est precisement la que vivent
les dettes best-effort de la section 3.
