# Vision Frontend v2 — Design depuis le protocole

**Date** : 2026-04-27
**Principe** : Page blanche. Aucune référence à la structure actuelle.
On part de ce que le protocole PERMET, pas de ce qui EXISTE en UI.
**Inspiration primaire** : Spacedrive (tokens, layout, glassmorphism ciblé)
**Docs source** : THREAT_MODEL.md, RUNTIME_ISOLATION.md, bridge/protocol.ts,
frontend_ux_protocol_analysis.md (surface API complète)

---

## 0. Ce qu'est SBFB vu par l'utilisateur

SBFB n'est pas un dashboard. C'est un **environnement local** :

- Tu lances SBFB. Tu ES un noeud du réseau. Ton identité existe.
- Tu découvres des apps que d'autres ont publiées. Tu choisis qui tu fais confiance (curators).
- Tu ouvres une app — elle tourne dans un sandbox isolé DANS ton shell.
- Si tu veux, tu partages ton GPU pour aider le réseau (consent explicite).
- Tu peux publier ta propre app depuis ton repo Git.
- Tout est vérifiable. Tout est cryptographique. Zéro compte, zéro login, zéro cloud.

Le shell est un **OS pour apps P2P**. Pas un site web avec des pages.

---

## 1. Philosophie design

### 1.1 Principes

**P1 — Tu es un noeud, pas un visiteur.**
L'UI montre ton identité, ton état, tes connexions dès le premier pixel.
Pas de "page d'accueil" vide. Tu es DANS le réseau.

**P2 — La confiance est un acte conscient.**
Chaque app affiche sa provenance. Chaque curator affiche ses claims.
Rien n'est "trusted by default". L'utilisateur CHOISIT, explicitement.

**P3 — Les apps sont des citoyens, pas des liens.**
Une app SBFB n'est pas un lien vers un site. C'est un programme
qui tourne localement dans un sandbox. L'UI doit montrer qu'elle
VIT dans le shell (pas "redirige" quelque part).

**P4 — Le réseau est vivant.**
Les peers arrivent et partent. Les tâches s'exécutent. Les curators
publient. L'UI montre cette activité comme un flux, pas comme des
tableaux statiques qu'on rafraîchit.

**P5 — La sécurité est visible, pas cachée.**
Le consent GPU a 4 niveaux. Le panic wipe existe. La provenance est
vérifiable. L'UI rend ces mécanismes tangibles — pas enfouis dans
des settings.

### 1.2 Anti-principes (ce qu'on refuse)

- Pas de gamification (leaderboards, XP, badges tier)
- Pas de notifications push intrusives
- Pas de metrics vanity (downloads count, "trending")
- Pas de dark patterns (consent pre-coché, difficult to revoke)
- Pas de "admin panel" vibe (tables CRUD partout)
- Pas de light mode (dark-only, cohérent avec l'identité technique)

---

## 2. Architecture d'information — 4 espaces

Au lieu de "pages" dans une navigation plate, le shell a **4 espaces**
qui correspondent aux 4 activités fondamentales du protocole :

```
┌─────────────────────────────────────────────────┐
│                    SHELL SBFB                    │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ DISCOVER │  │   RUN    │  │  NODE    │      │
│  │          │  │          │  │          │      │
│  │ trouver  │  │ utiliser │  │ mon      │      │
│  │ des apps │  ��� une app  │  │ noeud    │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│                                                  │
│  ┌──────────┐                                    │
│  │ PUBLISH  │                                    │
│  │          │                                    │
│  │ deployer │                                    │
│  │ mon app  │                                    │
│  └──────────┘                                    │
└─────────────────────────────────────────────────┘
```

### Espace 1 : DISCOVER — trouver des apps sur le réseau

**Ce que le protocole expose** :
- `GET /daemon/browse` → BrowseEntry[] (project_id, name, description, category, status, is_open_source, repo_url, provenance_hash, archive_hash, curator_pubkey, source)
- `GET /daemon/curators` → curator lists avec entries + subscribed set
- `GET /daemon/default-curators` → curators par défaut
- `POST /daemon/curators/subscribe` / `DELETE /daemon/curators/{pubkey}`
- `GET /api/contributor/project/{pid}` → contributeurs attestés
- `GET /api/contributor/envelope/{pid}/{nid}` → in-toto SLSA

**Concept UX** :

Discover n'est PAS un catalogue plat. C'est une exploration guidée par
la confiance. L'utilisateur voit les apps **à travers les yeux de ses
curators** — pas une liste brute du réseau entier.

**Sections** :

**a) Feed curators** — la vue par défaut
Chaque curator suivi = un "canal" d'apps approuvées. L'UI montre :
```
┌─────────────────────────────────────────────┐
│ Curator : Alice (8a3f...c2d1)   ★ 1.2k kudos│
│ 14 apps approuvées · rev 23 · il y a 2h     │
│ ─────────────────────────────────────────── │
│ [icon] DataViz    [icon] LLM Chat  [icon]...│
│ ★ verified        ★ verified       ◆ comm.  │
│                                              │
│ Curator : Bob (7d2e...a9f0)     ★ 890 kudos │
│ 8 apps approuvées · rev 11 · il y a 1j      │
│ ─────────────────────────────────────────── │
│ [icon] Notebook   [icon] Analyzer  [icon]...│
└─────────────────────────────────────────────┘
```
Pas de grille Netflix anonyme. Chaque app est DANS le contexte de
qui l'a approuvée. L'utilisateur sait POURQUOI il voit cette app.

**b) Recherche globale** — barre de recherche en haut
Cherche dans TOUTES les browse entries, pas uniquement les curators suivis.
Résultats groupés par source : "Curators suivis" → "Réseau global".
Filtres : catégorie, verified only, reachable only, GPU required.

**c) Explorer un curator** — page profil curator
Clic sur un curator → profil complet :
- Pubkey Ed25519 (copiable)
- Identicon déterministe
- Nombre d'apps approuvées, revision, date dernière update
- Claims Keyoxide (si disponibles via le futur protocole)
- Liste complète de ses apps avec provenance
- Bouton subscribe / unsubscribe

**d) Page app** — avant de lancer
Clic sur une app → page de CONFIANCE (pas juste un splash screen) :
```
┌──────────────────────────────────────────────┐
│ [← Retour]                                    │
│                                               │
│ ┌─��────────────────────────────────────────┐ │
│ │                                          │ │
│ │    Preview (screenshot ou icon large)    │ │
│ │                                          │ │
│ └──────────────────────────────────────────┘ │
│                                               │
│ DataViz v2.1.0                    [Lancer ▶] │
│ "Visualisation interactive de datasets"       │
│                                               │
│ ── Provenance ──────────────────────────────  │
│ Source   github.com/alice/dataviz             │
│ Commit   a3f2c8d (2026-04-25)                │
│ Hash     7c8d...ef12 (BLAKE3)                │
│ Signé    Ed25519 ✓ par node:8a3f...          │
│ SLSA     Level 1 ✓                           │
│ Archive  1.2 MB · 42 noeuds la servent       │
│                                               │
│ ── Confiance ───────────────────────────────  │
│ Approuvée par 3 curators que tu suis :        │
│   Alice (★ 1.2k) · Bob (★ 890) · Carol (★ 340)│
│                                               │
│ ── Sandbox ─────────────────────────────────  │
│ Cette app peut :                              │
│   ✓ Soumettre des tâches compute              │
│   ✓ Lire/écrire son storage local             │
│   ✓ Redacter du PII localement                │
│   ✗ Aucun accès réseau direct (sandboxée)     │
│   ✗ Aucun accès à tes données shell           │
│                                               │
│ ── Contributeurs attestés ──────────────────  │
│ alice (first deploy 2026-03-15) · bob (2026-04)│
│                                               │
│ [Voir le code source ↗] [Signaler]           │
└──────────────────────────────────────────────┘
```

**Données clés** : Provenance, confiance (curators), sandbox permissions,
contributeurs. L'utilisateur a TOUT pour décider avant de lancer.

### Espace 2 : RUN — utiliser une app

**Ce que le protocole expose** :
- `GET /blob-serve/{hash}/{path}` → fichiers zip décompressés dans iframe sandbox
- Bridge postMessage : `task_submit`, `storage_get`, `storage_set`, `pii_redact`
- Heartbeat watchdog (5s stall detection)
- `POST /consent/whitelist/add` → contribuer GPU à cette app

**Concept UX** :

L'app lancée prend TOUT l'écran. Le shell disparaît — sauf une
**barre flottante** en haut qui apparaît au hover souris (ou gesture).

```
┌──────────────────────────────────────────────┐
│ ◀ │ DataViz │ ★ verified │ 🔗 bridge OK │ ⋯ │  ← barre flottante auto-hide
├──────────────────────────────────────────────┤
│                                              │
│                                              │
│           [IFRAME APP FULL SCREEN]           │
│                                              │
│     Tout le contenu de l'app sandboxée       │
│     s'affiche ici. Le shell n'interfère      │
│     pas avec le rendu.                        │
│                                              │
│                                              │
└──────────────────────────────────────────────┘
```

**Barre flottante** (auto-hide après 3s d'inactivité souris) :
- Bouton retour (← Discover)
- Nom de l'app + version
- Badge provenance (verified / community)
- Indicateur bridge (connected / stalled)
- Menu contextuel (⋯) : infos app, contribuer GPU, voir source, signaler, fermer

**État stalled** (watchdog 5s sans heartbeat) :
```
┌──────────────────────────────────────────────┐
│                                              │
│         L'application ne répond plus.         │
│                                              │
│     [Recharger]        [Fermer]              │
│                                              │
└──────────────────────────────────────────────┘
```

Overlay semi-transparent sur l'iframe gelée. Deux actions claires.

### Espace 3 : NODE — mon noeud SBFB

**Ce que le protocole expose** :
- Identité : Ed25519 keypair (node_id), daemon version, uptime
- GPU : WorkerStateV1 (GPU name, VRAM, utilization, temperature, power, tasks)
- Consent : ConsentConfig (level L1-L4, caps, whitelist)
- Réseau : neighborhood peers, curators subscribed, browse entries count
- Sécurité : security events (14 types JSONL), canary status, quarantine
- Fairness : Gini, top 5% share, churn rate, worker count
- Kudos : ledger hash-chain, entries par worker
- Invites : create, list, revoke

**Concept UX** :

NODE est le **cockpit** de ton noeud. Pas un dashboard plat avec 15
cards — un espace organisé en **sections dépliables** qui montre
l'essentiel par défaut et le détail à la demande.

```
┌──────────────────────────────────────────────┐
│ MON NOEUD                                     │
│                                               │
│ ┌─ Identité ──────────────────────────────┐  │
│ │ [identicon]  nexus:8a3f...c2d1          │  │
│ │ Daemon v0.34.0 · uptime 14h23m          │  │
│ │ [Copier node_id]                        │  │
│ └─────────────────────────────────────────┘  │
│                                               │
│ ┌─ GPU ───────────────────────────────────┐  │
│ │ RTX 5080                                │  │
│ │ ████████░░ 78%  VRAM 12/16 GB           │  │
│ │ 72°C · 280W · Consent L2 (open source)  │  │
│ │ [Modifier le consent]                   │  │
│ │                                         │  │
│ │ Tâche en cours : llm-inference-3a8f     │  │
│ │ ████████████░░░ 73%                     │  │
│ │ Soumise par dataviz (curator: Alice)    │  │
│ │                                         │  │
│ │ Kudos aujourd'hui : +42                 │  │
│ └─────────────────────────────────────────┘  │
│                                               │
│ ▶ Réseau (3 peers · 2 curators · 14 apps)    │
│ ▶ Sécurité (0 événements récents · canary OK) │
│ ▶ Fairness (Gini 0.34 · 12 workers actifs)   │
│ ▶ Invites (2 actifs · 1 utilisé)              │
│ ▶ Quarantaine (0 messages en attente)         │
│                                               │
└──────────────────────────────────────────────┘
```

Les sections ▶ se déplient au clic pour montrer le détail :

**Réseau déplié** :
- Peers connectés (node_id court, type, latence)
- Curators suivis (avec count apps par curator)
- Statistique : apps servies, bandwidth P2P

**Sécurité déplié** :
- Timeline des 20 derniers events (SecurityEvent enum)
- Canary : date, headline, next_update, signature status
- Panic wipe rappel du keybind

**Fairness déplié** :
- Gauge Gini (0-1, seuil warning 0.70)
- Top 5% share bar
- Worker count, churn rate

**Invites déplié** :
- Table invites (wire, scope, expires, uses)
- Bouton créer invite (worker/observer, durée, max uses)

**Consent Dialog** (modal depuis le bouton "Modifier le consent") :
```
┌──────────────────────────────────────────────┐
│ Consent GPU                                   │
│                                               │
│ Quel niveau de partage acceptes-tu ?          │
│                                               │
│ ○ L1 — Mes projets uniquement (défaut)        │
│   Seules tes propres apps utilisent ton GPU.   │
│                                               │
│ ○ L2 — Open source vérifiées                  │
│   Apps déployées depuis un repo public,        │
│   provenance SLSA L1 vérifiée.                 │
│                                               │
│ ○ L3 — Whitelist manuelle                     │
│   Tu choisis individuellement chaque app.      │
│   [input node_id] [+ Ajouter]                 │
│   - dataviz (8a3f...) [× Retirer]             │
│                                               │
│ ○ L4 — Tous les projets publics               │
│   ⚠ Risque maximum. N'importe quelle app     │
│   du réseau peut utiliser ton GPU.             │
│                                               │
│ Limites :                                     │
│ Puissance max   [▬▬▬▬▬▬▬░░░] 400W            │
│ VRAM max        [▬▬▬▬▬▬▬▬░░] 16 GB           │
│ Heures/jour     [▬▬▬▬▬▬░░░��] 12h             │
│                                               │
│ [Annuler]                    [Enregistrer]    │
└──────────────────────────────────────────────┘
```

### Espace 4 : PUBLISH — déployer mon app

**Ce que le protocole expose** :
- `POST /project/deploy-from-repo` (public, SLSA L1)
- `POST /project/deploy` (private, upload zip)
- `POST /publish-blob` → store blob → `POST /publish` → gossip announce

**Concept UX** :

Un wizard en 4 étapes, pas une page avec un formulaire.

**Étape 1 — Source**
```
Comment veux-tu publier ?

[Depuis un repo Git]     [Upload un zip]
  (public, vérifié)       (privé, non vérifié)
```

**Étape 2 — Vérification** (repo Git)
```
URL du repo : [https://github.com/alice/dataviz    ]
Commit (optionnel) : [a3f2c8d...                    ]

[Vérifier →]

✓ Repo accessible
✓ SBFB.json trouvé
✓ node_id correspond à ton noeud
✓ index.html présent
  Clone en cours... 42%
```

**Étape 3 — Provenance**
```
Provenance générée :

Source    github.com/alice/dataviz
Commit    a3f2c8d (HEAD)
Hash      7c8d...ef12
Signé     Ed25519 par ton noeud (8a3f...c2d1)
SLSA      Level 1

L'archive inclut provenance.json.

[← Retour]               [Publier sur le réseau →]
```

**Étape 4 — Publication**
```
✓ Archive stockée (hash: 7c8d...ef12)
✓ Annonce broadcast via gossip
✓ 1 noeud la sert (toi)

Ton app est maintenant visible par les curators du réseau.
Pour qu'elle apparaisse dans Discover, un curator doit
l'ajouter à sa liste.

[Copier le lien] [Voir dans Discover] [Terminé]
```

---

## 3. Layout shell — structure globale

```
┌──────────────────────────────────────────────────────┐
│ [TopBar — 48px, flottante, blur]                      │
│  ┌──────┐                    ┌─────��────────────────┐│
│  �� Logo │  [search.........] │ [node status] [⚙]   ││
│  └──────┘                    └──────────────────────┘│
├──────────┬───────────────────────────────────────────┤
│ Sidebar  │  Content area                             │
│ ~200px   │                                           │
│          │  Contenu de l'espace actif                 │
│ DISCOVER │  (Discover / Run / Node / Publish)        │
│ ────── │                                           │
│ [Store]  │                                           │
│ [Curators]│                                          │
│          │                                           │
│ MON NOEUD│                                           │
│ ────── │                                           │
│ [GPU]    │                                           │
│ [Réseau] │                                           │
│ [Sécurité]│                                          │
│          │                                           │
│ MES APPS │                                           │
│ ────── │                                           │
│ [Publier]│                                           │
│ [app 1]  │                                           │
│ [app 2]  │                                           │
│          │                                           │
│──────────│                                           │
│ [status] │                                           │
│ 3 peers  │                                           │
│ daemon ● │                                           │
└──────────┴───────────────────────────────────────────┘
```

### Sidebar (~200px) — Spacedrive-inspired

**3 groupes** (titres uppercase `text-[11px] tracking-wider`) :

**DISCOVER**
- Store — le feed curators + recherche globale
- Curators — gérer ses abonnements

**MON NOEUD**
- GPU — consent + métriques worker
- Réseau — peers, neighborhood, stats
- Sécurité — events, canary, panic

**MES APPS** (dynamique)
- Publier — wizard deploy
- [app installée 1] — clic = lance dans l'espace RUN
- [app installée 2]
- ...

**Bottom bar** (ancrée en bas de la sidebar) :
- Daemon status dot (● vert connecté, ● rouge offline)
- Peers count
- Settings gear icon

### TopBar (48px, flottante)

**Pattern portal** (Spacedrive) : chaque espace injecte ses propres
contrôles dans la TopBar.

- **Discover** : barre de recherche centrée, filtres à droite
- **Run** : auto-hide, nom app + badge + bridge status
- **Node** : titre "Mon noeud" + dernière activité
- **Publish** : étape courante du wizard

**Toujours visible** (injection globale) :
- Node status résumé (identicon mini + peers count)
- Command palette trigger (Cmd+K)

### RUN mode — full screen

Quand une app est lancée, le layout change :
- Sidebar **se collapse** (0px, animation 0.3s)
- TopBar devient la **barre flottante** auto-hide
- L'iframe prend 100% de l'espace
- Le content area n'a plus de padding

Retour au shell = clic bouton ← ou Escape.

---

## 4. Flux utilisateur — les 7 parcours fondamentaux

### F1 : Premier lancement (onboarding)

```
1. Launcher crée keypair Ed25519 + bearer token (~0.1s)
2. Shell s'ouvre → aucun curator, aucune app
3. Message de bienvenue :
   "Bienvenue sur SBFB. Tu es le noeud nexus:8a3f...c2d1."
   [identicon large]
   "Pour découvrir des apps, abonne-toi à un curator."
   [Ajouter les curators par défaut]  [Ajouter manuellement]
4. Clic "défaut" → subscribe aux curators du réseau
5. Feed se peuple → l'utilisateur explore
```

Pas de wizard 7 étapes. Une action, un résultat.

### F2 : Découvrir et lancer une app

```
1. Sidebar > Store → feed curators
2. Scroll → trouve "DataViz" dans le canal d'Alice
3. Clic → page confiance (provenance, sandbox, curators)
4. Lit la provenance, vérifie que c'est open source ✓
5. Clic [Lancer ▶]
6. Transition : sidebar collapse, iframe full screen
7. App tourne, bridge connecté, watchdog actif
8. Utilisateur travaille dans l'app
9. Escape ou ← → retour au shell
```

### F3 : Contribuer mon GPU

```
1. Sidebar > GPU → voit son RTX 5080 idle
2. Consent actuel : L1 (mes projets uniquement)
3. Clic [Modifier le consent]
4. Choisit L2 (open source vérifiées)
5. Ajuste caps : 300W max, 12 GB VRAM, 8h/jour
6. [Enregistrer]
7. Worker commence à accepter des tâches open source
8. La card GPU montre la tâche en cours + kudos accumulés
```

### F4 : Publier une app

```
1. Sidebar > Publier
2. Choisit "Depuis un repo Git"
3. Colle l'URL → vérification automatique
4. Revoit la provenance générée
5. [Publier sur le réseau]
6. Reçoit confirmation + hash
7. Partage le lien avec un curator pour inclusion
```

### F5 : Gérer ses curators

```
1. Sidebar > Curators
2. Voit la liste des curators suivis
3. Clic sur un curator → profil complet (apps, revision, claims)
4. [+ Ajouter un curator] → input pubkey hex 64 chars
5. Ou [Ajouter les défauts] pour les curators réseau
6. Unsubscribe via bouton × sur chaque curator
```

### F6 : Réagir à un événement sécurité

```
1. Sidebar > Sécurité → déplie
2. Timeline montre : "ConsentChange il y a 2h" + "TokenRotation il y a 1j"
3. Canary status : "Frais (publié il y a 12j, prochain dans 33j)"
4. Si canary expiré → banner rouge "⚠ Canary expiré — le mainteneur
   n'a pas publié depuis 45+ jours"
```

### F7 : Panic wipe (urgence)

```
1. N'importe quand : Ctrl+Shift+Alt+W × 5 en 3s
2. Aucun feedback visuel (deniability)
3. Keypair détruite, state wipé, daemon exit
4. Au relancement : nouvelle identité, zéro historique
```

---

## 5. Composants design system — inventaire

### 5.1 Tokens sémantiques (Spacedrive-inspired, hue SBFB)

On adopte le système Spacedrive (app/sidebar/ink/accent/menu) avec
notre propre hue. Proposition : **hue 260** (violet, cohérent avec
l'accent actuel #7c3aed).

```css
/* Surfaces */
--app:             hsl(260, 15%, 13%);
--app-box:         hsl(260, 15%, 18%);
--app-line:        hsl(260, 15%, 23%);
--app-hover:       hsl(260, 15%, 19%);
--app-selected:    hsl(260, 15%, 24%);

/* Sidebar */
--sidebar:         hsl(260, 15%, 7%);
--sidebar-box:     hsl(260, 15%, 16%);
--sidebar-line:    hsl(260, 15%, 23%);
--sidebar-selected: hsl(260, 15%, 24%);

/* Text */
--ink:             hsl(260, 35%, 92%);
--ink-dull:        hsl(260, 10%, 70%);
--ink-faint:       hsl(260, 10%, 55%);

/* Accent */
--accent:          hsl(260, 100%, 65%);  /* violet vif */
--accent-faint:    hsl(260, 100%, 72%);
--accent-deep:     hsl(260, 100%, 55%);

/* Menu */
--menu:            hsl(260, 15%, 10%);
--menu-hover:      hsl(260, 15%, 30%);
--menu-ink:        hsl(260, 25%, 92%);

/* Status */
--status-ok:       hsl(142, 70%, 45%);   /* vert */
--status-warn:     hsl(38, 92%, 50%);    /* ambre */
--status-error:    hsl(0, 72%, 51%);     /* rouge */
--status-info:     hsl(208, 100%, 57%);  /* bleu */
```

### 5.2 Composants de base

| Composant | Rôle | Détails |
|-----------|------|---------|
| **SidebarGroup** | Section sidebar | Titre uppercase + items |
| **SidebarItem** | Lien sidebar | Icon + label, active/inactive states |
| **SidebarStatus** | Bottom bar | Daemon dot + peers count |
| **TopBarPortal** | Injection per-page | left/center/right slots |
| **SearchBar** | Recherche globale | Input avec icône, fuzzy match |
| **NodeBadge** | Identité mini | Identicon + node_id court |

### 5.3 Composants Discover

| Composant | Rôle | Détails |
|-----------|------|---------|
| **CuratorFeed** | Canal d'apps par curator | Header curator + grille apps |
| **CuratorCard** | Profil curator compact | Identicon + pubkey + kudos + count apps |
| **AppCard** | App dans la grille | Icon + nom + trust badge + curator endorsements |
| **TrustBadge** | Niveau confiance | Verified (vert) / Community (gris) / Official (violet) |
| **ProvenanceChain** | Chaîne visuelle | repo → commit → hash → sign (étapes cliquables) |
| **SandboxPermissions** | Permissions app | Liste ✓/✗ des capabilities |
| **ContributorList** | Attestations | Liste contributeurs avec dates |
| **AppPage** | Page confiance complète | Composition des composants ci-dessus |

### 5.4 Composants Run

| Composant | Rôle | Détails |
|-----------|------|---------|
| **AppFrame** | Iframe sandbox | Full-screen, sandbox="allow-scripts" |
| **FloatingBar** | Barre auto-hide | Nom + badge + bridge status + menu |
| **BridgeIndicator** | État bridge | Connected (vert) / Stalled (rouge) / Starting (ambre) |
| **StalledOverlay** | App gelée | Message + Recharger/Fermer |

### 5.5 Composants Node

| Composant | Rôle | Détails |
|-----------|------|---------|
| **IdentityCard** | Keypair Ed25519 | Identicon large + node_id + copy + daemon version |
| **GpuCard** | Métriques GPU | Model + VRAM bar + util% + temp + power + task courante |
| **ConsentBadge** | Niveau consent | L1/L2/L3/L4 avec couleur |
| **ConsentDialog** | Modal consent | 4 radio + caps sliders + whitelist |
| **CollapsibleSection** | Section dépliable | Titre + chevron + contenu animé |
| **SecurityTimeline** | Events log | Timeline verticale, icônes par type, timestamp relatif |
| **CanaryStatus** | Warrant canary | Date + headline + freshness badge |
| **FairnessGauge** | Gini coefficient | Barre 0-1 avec seuil warning 0.70 |
| **InviteTable** | Invites CRUD | Table + actions (create, revoke) |
| **PeerList** | Peers connectés | Table avec node_id, type, latence |
| **KudosCounter** | Kudos journaliers | Number ticker animé |

### 5.6 Composants Publish

| Composant | Rôle | Détails |
|-----------|------|---------|
| **PublishWizard** | Flow 4 étapes | Stepper + content per step |
| **SourcePicker** | Choix source | Git repo / Upload zip |
| **RepoVerifier** | Vérification repo | Checklist animée (✓ repo, ✓ SBFB.json, ✓ index.html) |
| **ProvenancePreview** | Revue provenance | Résumé avant publication |
| **PublishResult** | Confirmation | Hash + lien + prochaines étapes |

### 5.7 Composants transversaux

| Composant | Rôle | Détails |
|-----------|------|---------|
| **CommandPalette** | Cmd+K | Recherche apps, navigation, actions rapides |
| **Identicon** | Avatar déterministe | Généré depuis Ed25519 pubkey (jdenticon) |
| **NumberTicker** | Métrique animée | Compteur avec transition spring (Magic UI) |
| **StatusDot** | Indicateur santé | ● vert/ambre/rouge, pulse animation optionnelle |
| **Toast** | Notification non-bloquante | Apparaît en bas à droite, disparaît après 5s |
| **GlassPanel** | Panel glassmorphic | `bg-surface/65 backdrop-blur-2xl rounded-2xl` |

---

## 6. Ce qui est UNIQUE à SBFB (pas copiable d'ailleurs)

### 6.1 Feed curators comme vue par défaut du store

Aucun app store ne fait ça. Les stores affichent des grilles plates
(Umbrel, CasaOS, Chrome) ou des catégories (F-Droid, Steam).
SBFB montre les apps **à travers les curators** — parce que dans un
réseau sans modération centrale, la confiance vient des PERSONNES
que tu suis, pas d'un algorithme.

### 6.2 Page de confiance avant lancement

Aucune plateforme P2P ne montre autant d'information de provenance
AVANT de lancer une app. La chaîne visuelle repo → commit → hash →
signature → curators est unique à SBFB.

### 6.3 Consent GPU comme acte design first-class

vast.ai/RunPod sont des marketplaces (tu vends ton GPU).
Salad/Golem sont des pools (tu donnes ton GPU et tu es payé).
SBFB est un système de **consent volontaire à 4 niveaux** sans
contrepartie monétaire. Le design doit montrer que c'est un CHOIX
éthique, pas une transaction.

### 6.4 Identité = keypair, zéro login

Pas de formulaire login. Pas de "forgot password". Pas de OAuth.
Tu ES ton node_id. L'identicon + le node_id court sont partout
dans l'UI comme un rappel constant de qui tu es sur le réseau.

### 6.5 Panic wipe intégré au shell

Aucune app grand public n'a un bouton de destruction totale
accessible par keybind. C'est un design de sécurité activiste
(journalistes, dissidents, chercheurs sous surveillance).

---

## 7. Prochaines étapes

Ce document est un **brief de design**, pas un plan de sprint.

Pour transformer ça en code :
1. Figer les tokens (hue, palette complète) → `web/src/index.css`
2. Implémenter le layout shell (sidebar + topbar + content) → 1 phase
3. Implémenter Discover (feed curators + app page) → 1-2 phases
4. Implémenter Node (identity + GPU + sections dépliables) → 1-2 phases
5. Implémenter Run (iframe + floating bar + watchdog) → 1 phase
6. Implémenter Publish (wizard 4 étapes) → 1 phase

Estimation : 1 sprint complet (6 phases A-F) pour le core.
Les enrichissements (security timeline, fairness gauge, canary,
quarantine) = sprint suivant.

---

## 8. Ce document n'est PAS

- Un plan de sprint (pas de D1-D5, pas de commits, pas de fail-fast)
- Du code (aucun fichier touché)
- Une copie de Spacedrive (on prend les TOKENS, pas le produit)

C'est une vision produit pour un shell P2P qui n'existe nulle part
ailleurs. Les décisions Day 0 seront gelées dans le kickoff du sprint
qui implémente cette vision.
