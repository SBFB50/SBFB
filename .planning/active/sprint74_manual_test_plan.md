# Plan de test manuel — Sprint 74 (« Disponibilité » + atelier fork) + plateforme SBFB

> **Pour qui** : toi (PO), test live à la main après reboot machine.
> **Objectif** : valider que les fonctionnalités livrées S74 (et les hotfixes
> #1–#8 qui les précèdent) marchent vraiment côté utilisateur, pas seulement en
> tests. Chaque test = **Objectif / Prérequis / Pas / Résultat attendu / [ ] verdict**.
> **Notation** : coche `[x]` PASS, `[!]` FAIL (note le détail en dessous), `[~]` partiel.
> Les blocs marqués **(2 nœuds)** exigent 2 daemons (2 machines, ou 2 `NEXUS_GRID_ROOT`
> + 2 ports sur la même machine). Les blocs **(cross-machine)** exigent 2 vraies machines.

---

## 0. Récupération environnement + build (à faire d'abord)

L'environnement de la session de dev était cassé (WSL wedgé → Docker 500 ;
réseau iroh hôte dégradé → holepunch en échec). **Un reboot machine remet tout
d'aplomb.** Ne refais PAS `wsl --shutdown` (c'est ce qui avait cassé Docker).

- [ ] **0.1 Reboot** la machine.
- [ ] **0.2 Docker** : `docker ps` répond sans erreur 500 (sinon relancer Docker Desktop, attendre l'engine).
- [ ] **0.3 Réseau iroh sain** : depuis `nexus/`,
  `cargo nextest run -p nexus-shell-daemon -E 'test(remote_seeder_reannounces_after_reboot_e2e)' --locked`
  → **passe en < quelques secondes** (s'il timeout 90s, le réseau hôte est encore
  dégradé : attendre / vérifier le pare-feu UDP, re-tester).
- [ ] **0.4 Fail-fast Rust complet** (preuve que rien n'est cassé) :
  `cargo fmt --all --check` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
  `cargo nextest run --workspace --locked` (doit être ~1675 Win, 0 fail) ·
  `cargo test --workspace --locked --doc`.
- [ ] **0.5 Build release** : `cargo build -p nexus-shell-daemon --release` + `cargo build -p nexus-launcher --release`.
- [ ] **0.6 Front** : `cd web && npm ci && npm run build` (le shell est bundlé et servi par le daemon).

### Setup nœud avec **port FIXE** (important — évite le bug localStorage)
Le shell mémorise les coordinateurs par **origine** (port inclus). Un port
éphémère qui change à chaque démarrage vide la liste → « Aucun nœud ». Pour
tester proprement, fixe le port :

```toml
# <NEXUS_GRID_ROOT>/config.toml
[network]
api_port = 8787
```

- [ ] **0.7 Nœud A** : `NEXUS_GRID_ROOT=~/.nexus-A` + `config.toml` port 8787 →
  `nexus-shell-daemon start` (ou via `nexus-launcher` qui ouvre le navigateur).
- [ ] **0.8 Shell** : ouvre `http://127.0.0.1:8787` → le daemon same-origin
  s'auto-ajoute comme nœud (hotfix `a53b9f6`), pas de « Aucun nœud ».

---

## 1. Démarrage & modèle « nœud » (Phase A — rename coordinateur→nœud)

- [ ] **1.1 Vocabulaire** : nulle part dans l'UI le mot « coordinateur » n'apparaît.
  La nav dit « **Publier** » (pas « Déployer »). Les murs vides / bannières / le
  dialogue d'ajout parlent de « **nœud** » / « **réseau** » (« Se connecter à un
  nœud », « URL du nœud », « Nœud joignable »).
  *Attendu* : 0 occurrence de « coordinateur » ; intentions claires (action = nœud, destination = réseau).
- [ ] **1.2 Connexion à un nœud** : le dialogue « Se connecter à un nœud » a comme
  URL par défaut l'origine courante (pas `8765` codé en dur).
- [ ] **1.3 Onboarding** : sur un nœud sans apps, l'écran vide explique le modèle
  daemon (pas d'ancien texte CLI Python).

---

## 2. Publier une app depuis un repo (deploy-from-repo + provenance + boot-restore)

- [ ] **2.1 Publier** : page « Publier » → déploie depuis un repo source (ex. un
  repo codeberg d'app SBFB : Protocol Explorer, Ideas Hub, ou Factory Viewer).
  *Attendu* : clone → signature Ed25519 → zip → provenance.json, l'app apparaît dans **Browse**.
- [ ] **2.2 Carte de succès « propre » (Phase A)** : après publication, la carte
  montre un **titre + ligne de vérité + pastille + avertissement**, et les hashs
  sont **repliés** sous « Détails techniques ». **AUCUN champ « hôte »/« cible »**
  (publier = acte d'identité locale signé, pas un choix d'hébergeur).
- [ ] **2.3 Rendu** : ouvre l'app depuis **Browse → la fiche → iframe** (jamais
  blob-serve en onglet direct). L'app se rend dans l'iframe sandbox.
- [ ] **2.4 Boot-restore (#7/#8)** : **redémarre le daemon** (stop/start). Browse
  ré-affiche l'app (log `gossip: restored project announcements from persisted
  outbox`). Les apps **deploy-from-repo** sont restaurées au même titre que `/publish`
  (parité outbox #8).
- [ ] **2.5 Reboot OS** : redémarre la machine, relance le nœud → l'app est
  toujours là (persistance iroh-docs + blobs + provenance + outbox).

---

## 3. Disponibilité — panneau latéral (Phase A + D + F)

> Le cœur produit S74 : **PUBLIER (auteur) est découplé de SEEDER (qui garde en
> ligne)**. Invariant : **héberger ≠ publier, seeder ≠ auteur**.

- [ ] **3.1 Ouvrir le panneau** : sur la fiche d'une app, le bouton
  « **Disponibilité** » (tri-état) ouvre le panneau latéral avec 4 sections :
  **AUTEUR scellé** / **ÉTAT** (sonde humanisée + bouton « Revérifier ») /
  **QUI-LA-GARDE** / **COPIES de secours**.
- [ ] **3.2 AUTEUR vs DISPONIBILITÉ** : l'auteur est présenté comme **immuable et
  scellé**, séparé visuellement de la disponibilité (mutable).
- [ ] **3.3 État « vu de ton nœud »** : la pastille/État emploie un libellé honnête
  type « **En ligne (vu de ton nœud)** » — pas un faux « En ligne » absolu (verrou
  anti-faux-vert NAT, D4). « Revérifier » relance la sonde.
- [ ] **3.4 Toggle « Garder en ligne » (Phase D)** : sur **ta propre app**, le
  toggle est **fonctionnel** (POST au clic, persiste). Active-le, **redémarre le
  daemon**, ré-ouvre le panneau → l'état ON **persiste** (table `keep_online` M18).
  Désactive → le tag blob skip-GC est retiré (« stockée, plus diffusée » ;
  remarque : sans GC reaper, OFF ne libère pas de disque aujourd'hui).
- [ ] **3.5 is_own correct (Phase G — KEEP-ONLINE-READ-PATH)** : le toggle
  propriétaire s'affiche bien pour une app **déployée par toi** même si son
  `project_id = blake3(nom)` ≠ `node_id` (l'ancienne heuristique le ratait). Sur
  une app **distante**, tu vois le **CTA volontaire** (« garder en ligne pour la
  communauté »), **jamais** le toggle propriétaire.
- [ ] **3.6 Compteur « Toi + N pairs » (Phase F)** : la section « Copies de secours »
  affiche « **Toi + N pairs (vus récemment)** » (best-effort). Sur une app que toi
  seul héberges : « Toi + 0 pair ».
- [ ] **3.7 CTA inertes honnêtes** : tout cran non encore câblé (ex. « Inviter un
  pair » côté UI) est un libellé « **Bientôt** » **inerte**, jamais un faux bouton actif.

---

## 4. Atelier fork (Phase B + C) — `sbfb-factory` CLI

> Forker une app du réseau dans un workspace local, l'éditer, la redéployer sous
> TON identité (provenance re-signée localement, jamais héritée de l'auteur).

- [ ] **4.1 Forker depuis un hit** : `sbfb-factory fork …` depuis le triplet de
  provenance (`repo_url` / `commit_sha` / `archive_hash`) d'une app du réseau.
  *Attendu* : workspace **distinct du repo nexus** créé, soit par **clone forge**
  `repo_url@commit_sha`, soit par **reconstruction blob** (unzip de `archive_hash`).
- [ ] **4.2 Intégrité blob (Phase C)** : le fork par blob **vérifie le hash blake3**
  (un mismatch → `ArchiveHashMismatch`, pas de fork silencieux corrompu).
- [ ] **4.3 Gardes sécurité** (contenu forge/zip = non fiable) : tente un fork avec
  un `repo_url` non-https → **rejeté** ; un `commit_sha` non-40-hex / avec option
  injectée → **rejeté** (anti git-arg-injection). Un zip avec entrée `../` ou
  symlink → **rejeté/skip** (anti zip-slip). *(Ces cas sont surtout couverts par
  les tests ; à vérifier au moins que le chemin nominal marche.)*
- [ ] **4.4 Éditer + redéployer (Phase C)** : modifie un fichier du workspace forké,
  puis `sbfb-factory redeploy …` (ou POST `/api/v1/deploy-workspace`).
  *Attendu* : nouvelle app publiée sous **TON nœud**, provenance **re-signée
  localement** (auteur = ton nœud), `is_open_source = false` (auto-attestation
  locale, jamais open-source). Elle apparaît dans Browse comme ta propre app.
- [ ] **4.5 Provenance d'origine intacte** : l'app **source** garde son auteur
  d'origine (le fork ne ré-attribue rien — R5).
- [ ] **4.6 Templates (Phase C)** : `sbfb-factory create --template react` →
  l'app générée **tourne dans l'iframe sandbox** (React 18 UMD vendored, no-build,
  compatible CSP `connect-src 'none'`). `--template pyodide` → scaffold avec
  **bannière + README honnêtes** indiquant qu'il **ne tourne pas encore** sous
  `connect-src 'none'` (expérimental). Vérifie aussi `static` et `static-reader`.

---

## 5. Seed cross-nœud (Phase E + F) — **(2 nœuds)**

> Un nœud garde en ligne l'app d'un AUTRE nœud. Deux chemins : **volontaire**
> (communautaire, sûr par content-addressing) et **invité** (auteur → pair, ALPN
> authentifié `sbfb/seed/0`).

Setup : Nœud A (port 8787, `~/.nexus-A`) publie une app ; Nœud B (port 8788,
`~/.nexus-B`) la découvre dans Browse.

- [ ] **5.1 Seed volontaire** : sur le Nœud B, ouvre la fiche de l'app de A →
  panneau Disponibilité → CTA volontaire « garder en ligne ». B **fetch + pin + tag**
  l'archive (signée par A, content-addressed). *Attendu* : B devient seeder ; le
  blob survit à un GC (tag posé — corrige R3).
- [ ] **5.2 Résilience** : **arrête le Nœud A**. Depuis un 3ᵉ point de vue (ou B),
  l'app **reste joignable** (servie par le seed de B). Le content-addressing
  garantit que B sert exactement les octets signés par A.
- [ ] **5.3 Compteur multi-seed** : sur l'app, le compteur passe à « **Toi + 1 pair** »
  (A voit B ; ou inversement selon le point de vue) après propagation `SeedAnnounced`.
- [ ] **5.4 Re-annonce au boot (Phase F)** : **redémarre le Nœud B**. Après boot, B
  ré-émet `SeedAnnounced` pour ses apps gardées-en-ligne (modèle reprovide) → le
  compteur se re-peuple sans action manuelle.
- [ ] **5.5 Seed invité (authentifié)** : si l'UI/CLI de mint d'invite est
  accessible : A émet une **invite** liée à la paire `(project_id, archive_hash)`
  → B la consomme via ALPN `sbfb/seed/0`. *Attendu* : sans invite valide → rejeté ;
  invite pour une autre app → rejetée (capability liée au contenu).
- [ ] **5.6 Anti-rejeu** : (test surtout automatisé) une requête seed rejouée est
  rejetée (nonce + fenêtre temporelle).

---

## 6. Recherche réseau (S73 + Phase G)

- [ ] **6.1 Barre de recherche** : dans Browse, le champ de recherche interroge
  `GET /api/daemon/search`. Tape un terme présent dans une app publiée → résultats
  avec provenance (repo_url / commit / hash).
- [ ] **6.2 Lien repo sûr (Phase G — B.5)** : un résultat dont le `repo_url` est
  `https://…` est cliquable ; un `repo_url` non-https s'affiche en **texte inerte**
  (pas de lien) — garde XSS.
- [ ] **6.3 Carte d'erreur (Phase G — SEARCH-VIEW)** : si la recherche échoue
  (ex. daemon arrêté en plein vol, ou drift de schéma), la vue montre une **carte
  d'erreur**, pas un **skeleton de chargement infini**. *(Pour simuler : arrête le
  daemon puis tape une recherche, ou observe un cas réseau.)*

---

## 7. Vérification de provenance & Proof Card

- [ ] **7.1 Voir la source** : sur une fiche app, le lien « Source » mène au repo
  (https uniquement — garde isHttpsUrl, Phase G B.5 sur VerificationDetail).
- [ ] **7.2 Badge de vérification** : le badge passe par ses états
  (Provenance → Vérification… → **Signature vérifiée** / Échouée).
- [ ] **7.3 Proof Card** : ouvre la Proof Card → score 0–100, couches de preuve,
  facteurs de risque, badge de risque (libellés FR, vocabulaire « source vérifiable »).

---

## 8. Apps bridge (postMessage) — Explorer / Ideas Hub / Factory Viewer

- [ ] **8.1 Protocol Explorer** : se rend dans l'iframe, les sections de démo de
  vérification fonctionnent.
- [ ] **8.2 Ideas Hub** : vote + stockage P2P (`storage_get`/`storage_set` via le bridge).
- [ ] **8.3 Factory Viewer** : consultation des preuves/aperçus/statuts (lecture seule).
- [ ] **8.4 Sandbox** : les apps tournent en `sandbox="allow-scripts"` sans
  `allow-same-origin` ; pas d'accès réseau sortant pour le contenu non fiable
  (CSP `connect-src 'none'`).

---

## 9. Exécution / compute (intentions + réseau) — #5

- [ ] **9.1 Intentions `/execute`** : la page d'exécution propose 3 intentions
  claires — « Exécuter sur Claude » / « en local (Ollama) » / « sur le réseau » —
  sans jargon `provider/kind`.
- [ ] **9.2 Claude / Ollama** : une exécution Claude streame ; une exécution Ollama
  (si un modèle local est dispo) streame aussi.
- [ ] **9.3 Réseau (auto-spawn worker, #5)** : soumets une tâche réseau. Le daemon
  **auto-spawn un worker** au 1er submit (zéro setup manuel). *Attendu* : un seul
  `Done` (PO-14), résultat ramené via le pont `result:` doc→DB.
- [ ] **9.4 Anti-orphelin** : arrête le daemon → le worker enfant meurt aussi
  (Job Object Windows / PR_SET_PDEATHSIG).
- [ ] **9.5 Gate quorum (Phase G — B.2)** : *(comportement back-end)* une tâche de
  build dont le quorum devient **mathématiquement impossible** est **rejetée
  rapidement** (statut terminal Rejected), pas laissée en attente zombie.

---

## 10. Cross-machine (validation P2P réelle) — **(cross-machine)** — optionnel

- [ ] **10.1 LAN** : 2 machines sur le même réseau (Win ↔ Mac), une publie, l'autre
  voit/rend/exécute.
- [ ] **10.2 WAN** : une machine dev ↔ un VPS (ex. Helsinki) via relay/holepunch iroh.
- [ ] **10.3 Seed cross-machine** : reprendre §5 entre 2 vraies machines.

---

## 11. Rapport de test

Pour chaque section, reporte ici :

| # | Fonctionnalité | Verdict | Note (si FAIL/partiel) |
|---|---|---|---|
| 1 | Modèle nœud / rename | [ ] | |
| 2 | Publier + boot-restore | [ ] | |
| 3 | Panneau Disponibilité (toggle, is_own, compteur) | [ ] | |
| 4 | Atelier fork + redeploy + templates | [ ] | |
| 5 | Seed cross-nœud (2 nœuds) | [ ] | |
| 6 | Recherche (lien sûr, carte d'erreur) | [ ] | |
| 7 | Provenance / Proof Card | [ ] | |
| 8 | Apps bridge | [ ] | |
| 9 | Exécution Claude/Ollama/réseau | [ ] | |
| 10 | Cross-machine (optionnel) | [ ] | |

**Bugs trouvés** (un par ligne : section, symptôme, repro, sévérité ressentie) :
- …

---

## Annexe — commandes utiles

```bash
# Nœud A (port fixe, root dédié)
NEXUS_GRID_ROOT=~/.nexus-A nexus-shell-daemon start   # config.toml [network] api_port=8787

# Nœud B (2e nœud, même machine)
NEXUS_GRID_ROOT=~/.nexus-B nexus-shell-daemon start   # config.toml [network] api_port=8788

# Publier depuis un repo (ou via la page « Publier » du shell)
curl -X POST http://127.0.0.1:8787/api/v1/deploy-from-repo \
  -H "X-SBFB-Token: <token>" -H "Content-Type: application/json" \
  -d '{"repo_url":"https://codeberg.org/…","ref":"main"}'

# Atelier fork — par clone forge (préféré) OU par reconstruction d'archive
sbfb-factory fork --dest ./mon-fork --repo-url https://… --commit-sha <sha40hex>
sbfb-factory fork --dest ./mon-fork --archive ./app.zip --archive-hash <blake3>
# Redeploy du workspace forké/édité sous l'identité du nœud local (path positionnel)
sbfb-factory redeploy ./mon-fork

# Créer une app depuis un template (static | static-reader | react | pyodide)
sbfb-factory create --template react --name mon-app --output ./mon-app

# Garder en ligne (toggle backend)
curl -X POST http://127.0.0.1:8787/api/daemon/keep-online \
  -H "X-SBFB-Token: <token>" -d '{"project_id":"<pid>","enabled":true}'

# Compteur de seeders
curl http://127.0.0.1:8787/api/daemon/seed-count/<project_id> -H "X-SBFB-Token: <token>"

# Recherche
curl "http://127.0.0.1:8787/api/daemon/search?q=<terme>" -H "X-SBFB-Token: <token>"
```

> Le `<token>` bearer est obtenu côté same-origin par le shell ; pour les appels
> curl manuels, récupère-le via le mécanisme `/auth/token` du daemon (loopback).
> Les routes/sous-commandes exactes peuvent légèrement varier — si une commande
> diffère, l'UI du shell reste le chemin de référence.
