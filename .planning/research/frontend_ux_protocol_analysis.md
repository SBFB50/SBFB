# Analyse UX/UI Frontend — Surface protocole SBFB

**Date** : 2026-04-27
**Méthode** : 5 agents parallèles (daemon API Rust, coordinator API Python, audit frontend React, features protocole, contraintes sécurité UX)
**Objectif** : Inventaire factuel de ce qui existe, ce qui manque, et les features frontend candidates.

---

## 1. Surface API disponible

### 1.1 Daemon Rust — 13 routes HTTP (axum)

| Route | Méthode | Auth | Rôle |
|-------|---------|------|------|
| `/health` | GET | non | Liveness probe (schema_version, daemon_version) |
| `/info` | GET | oui | DaemonStateSnapshot complet (curators, lists, browse entries) |
| `/curators` | GET | oui | Curator lists cachées + subscribed_curators |
| `/curators/subscribe` | POST | oui | Ajouter curator pubkey hex à l'attention set |
| `/curators/{pubkey}` | DELETE | oui | Retirer curator + évict cache |
| `/browse` | GET | oui | BrowseEntry[] avec reachability status (reachable/unreachable/unknown) |
| `/publish` | POST | oui | Broadcast ProjectAnnouncement via gossip (PoW envelope) |
| `/publish-blob` | POST | oui | Store zip blob iroh, retourne hash BLAKE3 |
| `/default-curators` | GET | oui | Curator pubkeys depuis config `[curator]` |
| `/panic/wipe` | POST | oui | **IRREVERSIBLE** : destroy identité + state + cache, exit |
| `/api/contributor/verify/{pid}/{nid}` | GET | oui | Proxy loopback vers coordinator (attestation) |
| `/diagnostic/neighborhood` | GET | oui | node_id + subscribed curator pubkeys |
| `/blob-serve/{hash}/{path}` | GET | non | Fichiers zip décompressés, CSP/COOP/COEP, LRU 32 archives |
| `/api/canary/frost/*` | POST | oui | 4 endpoints FROST DKG (trusted-dealer, round1, round2, aggregate) |

**Zéro WebSocket/SSE côté daemon.** Le frontend poll via GET (60s TTL browse cache).

### 1.2 Coordinator Python — 48 endpoints FastAPI

| Domaine | Endpoints | Clés |
|---------|-----------|------|
| Health/Project | 3 | `/health`, `/project`, `/project/publish` |
| Tasks | 2 | `/tasks/submit` (POST), `/tasks` (GET, filtrable par state) |
| Apps SDK | 7 | manifest, descriptor, commands, invoke, state, submit task |
| Events SSE | 2 | `/app/{name}/events` (SSE stream, heartbeat 30s), `_publish` |
| Files CAS | 3 | upload multipart, manifest, stream blob |
| Daemon proxy | 7 | info, curators, browse, publish, subscribe, unsubscribe, defaults |
| Deploy | 2 | `/project/deploy` (private zip), `/project/deploy-from-repo` (public SLSA) |
| Kudos | 2 | list (filtrable worker_pubkey), verify hash-chain |
| Invites | 3 | create, list, revoke |
| Consent GPU | 4 | get, set, whitelist/add, whitelist/remove |
| Contributor | 3 | verify, project list, envelope (in-toto) |
| Canary | 4 | network-health, observed, inject-rate, observed-divergence |
| Quarantine | 3 | list (pending/flushed/dropped/all), flush, drop |
| Worker state | 1 | `/worker-state` (WorkerStateV1 + GPU snapshot, stale > 15s) |
| Shell discover | 1 | `/shell/discover` (multi-coordinator registry) |

**SSE uniquement sur** `/app/{name}/events` (heartbeat 30s, topic pattern matching).

---

## 2. État frontend actuel — inventaire factuel

### 2.1 Pages existantes (7 routes, toutes FONCTIONNELLES)

| Route | Page | État | API connectée | Polling |
|-------|------|------|---------------|---------|
| `/my-projects` | Projects | FONCTIONNEL | GET /health | 5s |
| `/project/:name` | ProjectDetail (5 tabs) | FONCTIONNEL | 7 queries (project, health, tasks, kudos, invites, apps) | 2-5s |
| `/my-network` | Network | FONCTIONNEL | GET /worker-state + /consent/get | 2s |
| `/browse` | Browse (Netflix grid) | FONCTIONNEL | GET /daemon/browse | 30s stale |
| `/browse/:projectId` | BrowsedProject (iframe) | FONCTIONNEL | bridge postMessage + watchdog 5s | - |
| `/curators` | Curators | FONCTIONNEL | GET/POST/DELETE /daemon/curators | 30s stale |
| `/app/:appName/tabs/:tabName` | AppTabPage | FONCTIONNEL | GET /app/{name}/tabs/{tab}/descriptor | 5s stale |

### 2.2 Composants globaux

| Composant | État | Rôle |
|-----------|------|------|
| AppShell | FONCTIONNEL | Left rail 68px + top bar + CoordinatorPicker + nav 4 routes |
| CommandPalette | FONCTIONNEL | Ctrl+K, lazy-loaded, commandes par app |
| GpuConsentDialog | FONCTIONNEL | L1-L4 radio, caps sliders, whitelist input |
| PanicWipeKeybind | FONCTIONNEL | Ctrl+Shift+Alt+W × 5 en 3s, silent fire-and-forget |
| AddCoordinatorDialog | FONCTIONNEL | URL input + test health + nickname |
| TabView blocks (12 types) | FONCTIONNEL | Schema-driven : heading, text, kv, metric, table, badge_list, button, chart_line, chart_bar, empty, file_upload, section |
| Bridge (useBridge) | FONCTIONNEL | iframe ↔ host RPC + watchdog heartbeat |

### 2.3 Stack technique

- React 19 + React Router v7 (lazy routes, code-split)
- Zustand (1 store : knownCoordinators, localStorage persisted)
- React Query (60+ queries/mutations, Zod-validated)
- Tailwind CSS v4 (dark-only, glassmorphic)
- 55 tests (23 Vitest + 32 Playwright)
- Size-limit enforced (7 budgets : main 50KB, vendor-react 290KB, etc.)
- **Zéro mock, zéro placeholder, zéro stub** dans le runtime

---

## 3. Gaps identifiés — features protocole sans UI

### GAP 1 : Provenance / Verified Deploy — affichage absent

**Backend** : `is_open_source` flag propagé dans BrowseEntry + ProjectAnnouncement. `repo_url`, `provenance_hash`, `archive_hash` disponibles dans le wire. Endpoint `/api/contributor/envelope/{pid}/{nid}` retourne in-toto SLSA L1 envelope.

**Frontend** : Browse cards affichent un badge verified basique. **Manque** :
- Modal "Détails de provenance" : repo_url cliquable, commit_sha tronqué + copie, signature Ed25519 status, date deploy
- Lien "Voir le code source" vers le repo Git
- Badge "Verified SLSA L1" vs "Unverified" explicite
- Liste des contributeurs attestés (`/api/contributor/project/{pid}`)
- Lien vers l'envelope in-toto (audit indépendant)

**Complexité estimée** : Faible. Données déjà dans BrowseEntry, pas de nouvel endpoint requis.

### GAP 2 : Security Events Log — zéro surface

**Backend** : 14 SecurityEvent types (ConsentChange, PanicFired, TokenRotation, DuressUnlock, QuarantineDrop, SybilAdmissionReject, PowVerifyFail, CanaryPublished, CanaryDeadMansSwitchTripped, TransportDegraded, RateLimitTierBreach, CapabilityChanged, ExecutorCrash, BrokerCrash). JSONL append-only avec rotation 10 MiB / 5 fichiers.

**Frontend** : **Aucune UI.** Aucun endpoint HTTP pour lire les events.

**Manque** :
- Endpoint coordinator ou daemon : `GET /security-log?limit=N&offset=M&type=filter`
- Page `/security-log` ou section dans `/my-network` : timeline/table des événements récents
- Filtrage par severity/type
- Badge "N événements récents" dans la sidebar

**Complexité estimée** : Moyenne. Nécessite un nouvel endpoint + page.

### GAP 3 : Network Topology / Peer Discovery — zéro visualisation

**Backend** : `/diagnostic/neighborhood` retourne node_id + peers. iroh 0.98 expose la discovery pkarr. Browse aggregator sonde la reachability.

**Frontend** : `/my-network` affiche le worker local uniquement. **Aucune vue réseau P2P globale.**

**Manque** :
- Vue topologie : node local + peers connectés + curators suivis + projets servis
- Ou au minimum : table des peers avec node_id, type (coordinator/worker), reachability, adresses
- Statistiques réseau : total peers visibles, uptime réseau, latence relay

**Complexité estimée** : Moyenne-haute. Endpoint `/diagnostic/neighborhood` existe mais limité (peers = curators seulement). Vue graphe D3.js optionnelle.

### GAP 4 : Quarantine Queue — opérateur seulement

**Backend** : 3 endpoints (`/quarantine/list`, `/quarantine/flush/{id}`, `/quarantine/drop/{id}`). SQLite queue avec TTL 15 min.

**Frontend** : **Aucune UI.** Opérateur doit utiliser curl.

**Manque** :
- Table quarantine dans `/my-network` ou section dédiée
- Actions flush/drop avec confirmation
- Badge "N messages en quarantaine" dans la sidebar

**Complexité estimée** : Faible. Endpoints existent, c'est du CRUD table.

### GAP 5 : Warrant Canary — vérification manuelle

**Backend** : Canary signé Ed25519, broadcast gossip, CANARY.txt dans repo. 4 endpoints FROST DKG. Endpoint `/api/canary/network-health` pour fraîcheur fleet.

**Frontend** : **Aucune UI.** Vérification = `verify-canary.sh` en CLI.

**Manque** :
- Section "Canary" dans page Network ou Settings : date dernière publication, headline, next_update, signature status
- Badge vert "Canary frais" / rouge "Canary expiré (> 45j)"
- Optionnel : timeline historique des canaries

**Complexité estimée** : Faible. Endpoint existe.

### GAP 6 : Fairness Metrics — endpoint existe, pas d'UI

**Backend** : `GET /diagnostic/fairness` retourne `{gini, top_5_pct_share, churn_rate, worker_count}`.

**Frontend** : **Aucune UI.**

**Manque** :
- Cards dans `/my-network` ou tab "Fairness" dans ProjectDetail
- Gini coefficient gauge (0-1), top 5% share bar, worker count, churn rate
- Seuils visuels : Gini > 0.70 = warning (conformément à fairness_vision.md)

**Complexité estimée** : Faible. Endpoint existe, c'est 4 métriques.

### GAP 7 : Deploy from UI — flow absent

**Backend** : `POST /project/deploy` (private zip) et `POST /project/deploy-from-repo` (public SLSA). Complets.

**Frontend** : **Aucune UI pour déclencher un deploy.** L'utilisateur doit utiliser curl ou CLI.

**Manque** :
- Page ou modal "Publier mon app" dans ProjectDetail :
  - Input repo URL + commit SHA optionnel
  - Bouton "Déployer depuis le repo" → POST deploy-from-repo
  - Progress : clone → verify → zip → sign → publish
  - Résultat : hash blob, provenance hash, statut publication gossip
- Alternative : upload zip pour projets privés (drag & drop)

**Complexité estimée** : Moyenne. Endpoints existent, mais le flow est multi-step (clone ~30s).

### GAP 8 : Task Submission depuis le shell — flow minimal

**Backend** : `POST /tasks/submit` complet avec SubmitRequest (task_type, prompt, model, priority, redundancy_factor, metadata).

**Frontend** : Soumission possible uniquement via :
1. Bridge postMessage depuis iframe app (indirect)
2. CommandPalette app commands (indirect)
3. ButtonBlock `kind: task_submit` dans TabView (indirect)

**Manque** :
- Form direct "Soumettre une tâche" dans ProjectDetail tab Tasks
- Champs : task_type, prompt (textarea), model (select), priority (slider 1-10)
- Suivi : état pending → claimed → result avec polling
- Affichage résultat brut

**Complexité estimée** : Faible. Endpoint + schéma existent.

---

## 4. Améliorations UX sur l'existant

### 4.1 Browse page — enrichissements

**État actuel** : Grid Netflix fonctionnel avec hero + cards + status dots.

**Améliorations candidates** :
- **Filtres** : par catégorie, par source (curator vs direct), par is_open_source, par reachability
- **Recherche** : input texte pour filtrer project_name / description
- **Tri** : par nom, par date (last_probed_at), par nombre de curators qui l'approuvent
- **Détail card étendu** : hover card avec description complète, curators list, archive size

### 4.2 Curators page — enrichissements

**État actuel** : Subscribe/unsubscribe + liste fonctionnelle.

**Améliorations candidates** :
- **Default curators** : afficher les curators par défaut du daemon (GET /default-curators) avec bouton "Subscribe all defaults"
- **Curator detail** : nombre de projets dans la list, date dernière mise à jour (revision), entries preview
- **Import batch** : coller plusieurs pubkeys d'un coup (séparées par newline)

### 4.3 Network page — enrichissements

**État actuel** : Worker state live + GPU consent dialog.

**Améliorations candidates** :
- **Section daemon** : afficher l'info daemon (GET /daemon/info) — node_id, version, uptime, curators subscribed count
- **Section peers** : neighborhood (GET /diagnostic/neighborhood) — liste peers
- **Section fairness** : métriques Gini + top 5% + churn (GET /diagnostic/fairness)
- **Section quarantine** : messages en attente (GET /quarantine/list)
- **Section canary** : statut warrant canary (GET /api/canary/network-health)

### 4.4 ProjectDetail — enrichissements

**État actuel** : 5 tabs (Overview, Tasks, Kudos, Invites, Apps).

**Améliorations candidates** :
- **Tab Deploy** : formulaire deploy-from-repo + statut dernière publication
- **Tab Contributors** : liste attestée (GET /api/contributor/project/{pid})
- **Amélioration tab Tasks** : formulaire submit + suivi résultats
- **Amélioration tab Overview** : provenance details, repo link, contributor count

### 4.5 BrowsedProject — enrichissements

**État actuel** : Iframe full-screen + auto-hide top bar + bridge + watchdog.

**Améliorations candidates** :
- **Panel latéral info** : provenance, curator approvals, contributor list, kudos count
- **Bouton "Signaler"** : flag content (futur modération communautaire)
- **Bouton "Forker"** : lien vers repo source + instructions fork (app store open source)

### 4.6 Shell global — enrichissements

**État actuel** : Left rail 4 routes + CoordinatorPicker + CommandPalette.

**Améliorations candidates** :
- **Notification badges** : quarantine count, security events count, stale canary
- **Status bar** : daemon online/offline, worker running/stopped, peers count
- **Settings page** : centraliser consent, curators defaults, token info, identity (node_id + copy)
- **Onboarding amélioré** : wizard step-by-step (lancer daemon → lancer coordinator → ajouter curator → browse → contribuer GPU)

---

## 5. Features nouvelles — candidates pour sprints futurs

### FEAT-1 : Page Settings / Identity (PRIORITÉ HAUTE)

Aucune page centralisée pour gérer son identité et ses préférences.

**Contenu** :
- Node ID (copie) + QR code optionnel
- Version daemon + coordinator + worker
- Token auth info (rotation date, overlap window)
- Consent GPU (relocaliser le dialog ici + raccourci depuis Network)
- Curator defaults
- Canary status
- Export/import identity (futur Sprint 17+)

### FEAT-2 : Dashboard opérateur unifié (PRIORITÉ MOYENNE)

Fusionner les vues éparpillées en un dashboard "état du noeud" :
- Daemon health + worker state + fairness metrics + quarantine + security events + neighborhood
- Un seul écran "Mon noeud SBFB" avec sections collapsibles

### FEAT-3 : Deploy wizard (PRIORITÉ MOYENNE)

Flow guidé pour publier une app :
1. Choix : private (upload zip) ou public (repo URL)
2. Si public : input repo URL → validation → clone progress → provenance review → publish
3. Résultat : hash, provenance, lien browse

### FEAT-4 : App Store social layer (PRIORITÉ BASSE, post-v1.0)

L'app store open source par construction (cf. CLAUDE.md) :
- Lien "Voir le code source" → repo Git
- Lien "Signaler un bug" → issues du repo
- Lien "Contribuer" → PRs du repo
- Bouton "Forker et déployer ma version"
- Reviews/ratings communautaires (nécessite nouveau protocole)

### FEAT-5 : Real-time push (PRIORITÉ BASSE)

Remplacer le polling HTTP par SSE ou WebSocket côté daemon :
- Browse entries updates (actuellement poll 30s)
- Worker state changes (actuellement poll 2s)
- Security events stream
- Réduirait la charge réseau loopback

---

## 6. Contraintes sécurité impactant le design

| Contrainte | Mécanisme | Impact UX | Résidu |
|------------|-----------|-----------|--------|
| Iframe sandbox `allow-scripts` only | CSP `connect-src 'none'`, pas de `allow-same-origin` | Apps forcées d'utiliser bridge SDK, zéro réseau direct | CPU-loop DoS → watchdog 5s |
| COOP/COEP | Headers sur blob-serve | Apps doivent bundler toutes les ressources (fonts, images) | Taille archives ↑ |
| Loopback triple check | Bearer + Host + Origin | Transparent pour l'utilisateur | Malware user-mode peut voler le token |
| GPU consent L1-L4 | Enforcement worker-side, watcher live | Dialog explicite, défaut L1 sécurisé | L4 = risque max accepté |
| Panic wipe | Ctrl+Shift+Alt+W × 5 | Geste d'urgence, destruction irréversible | Adversaire kernel non mitigé |
| Ed25519 identity | Pas de login/OAuth, keypair locale | Zéro friction boot, mais perte keypair = perte identité | Pas de recovery |
| Content trust | Curator lists + provenance SLSA L1 | Badges verified/unverified sur Browse | Provenance ≠ sécurité du code |

---

## 7. Synthèse prioritaire

### Tier 1 — Quick wins (données déjà disponibles, UI simple)

| Item | Endpoints existants | Effort |
|------|-------------------|--------|
| Provenance details modal (GAP 1) | BrowseEntry fields + /api/contributor/* | ~200 LOC |
| Fairness metrics cards (GAP 6) | GET /diagnostic/fairness | ~100 LOC |
| Quarantine table (GAP 4) | GET /quarantine/list + flush/drop | ~200 LOC |
| Canary status badge (GAP 5) | GET /api/canary/network-health | ~100 LOC |
| Browse filters/search (4.1) | Client-side sur données existantes | ~150 LOC |
| Task submit form (GAP 8) | POST /tasks/submit | ~200 LOC |

### Tier 2 — Features moyennes (nouvel endpoint ou flow multi-step)

| Item | Besoin | Effort |
|------|--------|--------|
| Security events log (GAP 2) | Nouvel endpoint + page | ~400 LOC backend + ~300 LOC front |
| Deploy wizard (GAP 7) | Endpoints existent, flow UI à créer | ~500 LOC front |
| Settings/Identity page (FEAT-1) | Agrégation d'endpoints existants | ~400 LOC front |
| Network page enrichie (4.3) | Composition d'endpoints existants | ~300 LOC front |
| Default curators + batch import (4.2) | GET /default-curators existe | ~150 LOC front |

### Tier 3 — Features lourdes (nouveau protocole ou infra)

| Item | Besoin | Effort |
|------|--------|--------|
| Network topology graph (GAP 3) | Nouvel endpoint peer discovery étendu | Backend + D3.js ~1000 LOC |
| Real-time push SSE/WS (FEAT-5) | Refactor daemon HTTP → SSE | Backend Rust significant |
| App store social layer (FEAT-4) | Nouveau protocole reviews/ratings | Post-v1.0 |
| Dashboard opérateur unifié (FEAT-2) | Composition pure, pas de nouveau backend | ~600 LOC front |

---

## 8. Ce document n'est PAS

- Un plan de sprint (pas de phases, pas de commits, pas de D1-D5)
- Un kickoff (pas de scope cuts, pas de risk register)
- Du code (aucun fichier touché)

C'est un inventaire factuel pour alimenter un futur sprint frontend-focused.
Les décisions Day 0 (quelles features prioriser, quel sprint les porte) restent à prendre.
