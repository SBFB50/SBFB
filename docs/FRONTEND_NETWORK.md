# FRONTEND_NETWORK.md — Page "Puissance Citoyenne" (Network)

Documentation technique de la page `/network` et de ses composants compute.

---

## 1. Vue d'ensemble

La page **Network** est accessible via la route `/network` (fichier `web/src/pages/NetworkPage.tsx`).
Elle est enregistree dans `web/src/App.tsx` :

```tsx
<Route path="/network" element={<NetworkPage />} />
```

Elle s'affiche dans le `Layout` principal (sidebar + topbar) au meme titre que les autres pages.

**Titre affiche** : "Puissance Citoyenne"
**Sous-titre** : "Reseau GPU distribue -- les citoyens alimentent l'IA politique"
**Icone** : `Cpu` (lucide-react), fond `emerald-500/10`, icone `text-emerald-400`

La page combine un bandeau header avec badges de mode d'execution, 5 cartes metriques, un bandeau self-worker avec controle pause/resume, et 6 onglets de contenu.

---

## 2. Bandeau Self-Worker

Situe entre les metriques et les onglets, ce bandeau affiche l'etat du GPU local de l'utilisateur et permet de le controler.

### Etats visuels

| Condition | Fond / Bordure | Indicateur (dot) |
|---|---|---|
| `running && !paused` | `bg-emerald-500/5 border-emerald-500/20` | `bg-emerald-500 animate-pulse` |
| `paused` | `bg-yellow-500/5 border-yellow-500/20` | `bg-yellow-500` |
| Arrete / pas de GPU | `bg-[var(--bg-card)] border-[var(--border)]` | `bg-gray-500` |

### Informations affichees

- **Ligne principale** (selon etat) :
  - Running : "Votre GPU contribue au reseau"
  - Paused : "GPU en pause"
  - Detect with no run : "GPU detecte : {selfWorker.gpu_model}"
  - No GPU : "Aucun GPU detecte"
- **Ligne secondaire** (running) : `{gpu_model} ({vram_mb / 1024} GB) -- {tasks_completed} taches -- {last_tokens_per_sec} tok/s`
- **Ligne secondaire** (arrete) : "Activez la contribution GPU pour alimenter l'analyse politique"

### Bouton Pause / Reprendre

Visible uniquement quand `selfWorker.running === true`.

| Etat | Texte | Style |
|---|---|---|
| `paused` | "Reprendre" | `bg-emerald-600 hover:bg-emerald-500 text-white` |
| Running | "Pause" | `bg-yellow-600/20 text-yellow-400 border-yellow-600/30` |

Actions API :
- Pause : `pauseSelfWorker()` -> POST `/compute/self-worker/pause`
- Resume : `resumeSelfWorker()` -> POST `/compute/self-worker/resume`
- Apres chaque action, `selfWorkerQ.refetch()` est appele.

### Champs du self-worker (depuis `useSelfWorkerStatus`)

| Champ | Type | Description |
|---|---|---|
| `running` | boolean | Worker actif |
| `paused` | boolean | Worker en pause |
| `gpu_model` | string | Nom du GPU (ex: "RTX 5080") |
| `vram_mb` | number | VRAM en megaoctets |
| `tasks_completed` | number | Nombre de taches terminees |
| `last_tokens_per_sec` | number | Derniere vitesse d'inference |

---

## 3. Badges de mode d'execution (header)

Trois badges conditionnels dans le coin superieur droit du header :

| Condition | Texte | Couleur |
|---|---|---|
| `model.execution_mode === 'petals'` | "Petals Swarm ({coverage_pct}% blocs)" | fuchsia-500 |
| `model.execution_mode === 'distributed'` | "Mode distribue (exo)" | purple-500 |
| `model.transition_state === 'transitioning'` | "Transition: {readiness_pct}%" | yellow-500 animate-pulse |

---

## 4. Les 5 cartes metriques

Grille `grid-cols-2 xl:grid-cols-5`. Composant interne `MetricCard`.

| # | Label | Champ source | Icone | Couleur |
|---|---|---|---|---|
| 1 | Contributeurs | `stats.nodes_online` | `Users` | `#22c55e` (emerald) |
| 2 | VRAM Totale | `stats.vram_total_gb` (formate "{n} GB") | `HardDrive` | `#06b6d4` (cyan) |
| 3 | Modele Actif | `stats.model_tier` ou `stats.current_model` (dernier segment apres `/`) | `Zap` | `#a855f7` (purple) |
| 4 | Tasks Aujourd'hui | `stats.tasks_today` | `Activity` | `#f59e0b` (amber) |
| 5 | Total Contributeurs | `leaderboard.total_contributors` | `Server` | `#3b82f6` (blue) |

Chaque carte utilise `bg-[var(--bg-card)]` avec `border-[var(--border)]`, label en `text-[10px] uppercase tracking-wider text-[var(--text-muted)]`, valeur en `text-xl font-bold`.

---

## 5. Les 6 onglets

Utilise `@/components/ui/tabs` (Tabs, TabsList variant="line", TabsTrigger, TabsContent).
Etat local : `const [activeTab, setActiveTab] = useState('stats')`.

### 5.1 Stats (`StatsTab`)

**Fichier** : `web/src/components/compute/StatsTab.tsx`
**Props** : `{ stats, model, hybrid, nodes }`

Contenu :
- **Carte "File de taches"** -- 4 barres de progression horizontales :
  - En attente (`tasks_pending`) -- couleur `#f59e0b`, icone `Clock`
  - En cours (`tasks_assigned`) -- couleur `#3b82f6`, icone `Loader2 animate-spin`
  - Terminees (`tasks_completed`) -- couleur `#22c55e`, icone `CheckCircle`
  - Echouees (`tasks_failed`) -- couleur `#ef4444`, icone `XCircle`
  - Chaque barre montre count/total en pourcentage.
- **Carte "Modele actif"** -- 6 rangees cle/valeur :
  - Modele : `model.target_model || stats.current_model`
  - Tier : `model.target_tier || stats.model_tier`
  - Mode : `model.execution_mode || 'local'`
  - Transition : `model.transition_state || 'stable'`
  - VRAM totale : `stats.vram_total_gb`
  - Max node : `model.max_single_node_vram_gb`
  - Barre de readiness (visible quand `readiness_pct < 100`)
- **Carte "Distribution GPU"** -- Grille `grid-cols-2 sm:grid-cols-3 xl:grid-cols-4` de mini-cartes par node :
  - Dot de statut (idle = emerald, busy = blue animate-pulse, offline = gray)
  - Nom du node, modele GPU, VRAM en GB

### 5.2 Leaderboard (`LeaderboardTab`)

**Fichier** : `web/src/components/compute/LeaderboardTab.tsx`
**Props** : `{ entries: LeaderboardEntry[], totalContributors: number }`

Interface `LeaderboardEntry` :

| Champ | Type |
|---|---|
| `rank` | number |
| `name` | string |
| `gpu_model` | string |
| `vram_mb` | number |
| `tasks_completed` | number |
| `avg_tokens_per_sec` | number |
| `trust_score` | number |
| `status` | string |

Table avec 7 colonnes :
1. **#** -- Rang, trophee pour top 3 (gold `#fbbf24`, silver `#94a3b8`, bronze `#cd7c2f`)
2. **Pseudo** -- `e.name`
3. **GPU** -- `e.gpu_model` + VRAM (hidden `md:table-cell`)
4. **Tasks** -- `e.tasks_completed` en vert emerald
5. **Vitesse** -- `e.avg_tokens_per_sec` t/s (hidden `sm:table-cell`)
6. **Confiance** -- `e.trust_score` avec icone Shield, couleur selon seuil (>=80 emerald, >=50 secondary, <50 red)
7. **Status** -- Dot colore (idle/busy/offline)

### 5.3 Nodes (`NodesTab`)

**Fichier** : `web/src/components/compute/NodesTab.tsx`
**Props** : `{ nodes: ComputeNode[] }`

Interface `ComputeNode` :

| Champ | Type |
|---|---|
| `id` | string |
| `name` | string |
| `gpu_model` | string |
| `vram_mb` | number |
| `status` | string |
| `tasks_completed` | number |
| `tasks_errored` | number |
| `avg_tokens_per_sec` | number |
| `trust_score` | number |
| `connected_at` | string | null |

Deux sections :
- **En ligne** -- Nodes filtrees par `status === 'idle' || status === 'busy'`
- **Hors ligne** -- Nodes filtrees par `status === 'offline'` (affichee uniquement si > 0)

Labels de statut :

| Statut | Label | Couleur |
|---|---|---|
| `idle` | En ligne | `#22c55e` |
| `busy` | En calcul | `#3b82f6` |
| `offline` | Hors ligne | `#6b7280` |
| `banned` | Banni | `#ef4444` |

Chaque `NodeCard` affiche : dot + nom, badge de statut, modele GPU + VRAM, tasks_completed (CheckCircle emerald), vitesse tok/s (Zap yellow), trust_score (Shield).

### 5.4 Swarm Petals (`SwarmTab`)

**Fichier** : `web/src/components/compute/SwarmTab.tsx`
**Props** : `{ swarm: Record<string, any> }`

Contenu :
- **Bandeau de sante** -- Couleur selon `swarm.health` :
  - `healthy` : `#22c55e` "Operationnel"
  - `degraded` : `#f59e0b` "Degrade"
  - `offline` : `#ef4444` "Hors ligne"
  - `unknown` : `#6b7280` "Inconnu"
  - Affiche "Pret a servir" si `swarm.is_ready === true`
- **Carte "Couverture des blocs transformer"** :
  - Barre de progression : `blocks_covered / blocks_total` (defaut 80 blocs)
  - Grille visuelle : chaque bloc = carre 2x2 (vert si couvert, `var(--bg-hover)` sinon)
  - Legende : "Chaque carre = 1 bloc transformer du modele"
- **3 cartes stats** (`grid-cols-1 md:grid-cols-3`) :
  - Noeuds Petals (`nodes_online`) -- icone Server bleu
  - Tokens/s batch (`throughput_tok_s`) -- icone Zap jaune
  - Modele distribue (`model` dernier segment) -- icone Wifi cyan
- **Carte "Comment ca fonctionne"** -- Texte explicatif sur le fonctionnement de Petals (decoupage en blocs transformer, pipeline GPU, performance attendue ~2 tok/s single / ~80 tok/s batch avec 50 contributeurs).

### 5.5 Ma contribution (`ContributeTab`)

**Fichier** : `web/src/components/compute/ContributeTab.tsx`
**Props** : `{ impact: Record<string, any> | null, loading: boolean }`

Deux etats :
- **Pas connecte** (`!impact || !impact.node_id`) : Message d'accueil avec instructions CLI :
  ```
  pip install nexus-worker
  nexus-worker register --server nexusgov.fr --name "Pseudo"
  nexus-worker start
  ```
- **Connecte** : Carte d'impact personnel :
  - Header : nom, GPU, VRAM, trust_score, total tasks
  - 4 StatCards :
    - Classement : `Top {100 - impact.percentile}%`
    - Tokens cette semaine : `impact.tokens_this_week`
    - Uptime total : `impact.uptime.total_seconds` converti en heures
    - Session en cours : `impact.uptime.current_session_seconds` converti en h/m
  - **Impact par type de tache** : barres de progression par `task_type` dans `impact.tasks_by_type[]`
  - **Message d'impact** : bandeau emerald "Votre GPU a contribue a rendre la democratie plus transparente"

Note : dans `NetworkPage.tsx`, ce composant est appele avec `impact={null}` et `loading={false}` (valeurs en dur). L'integration complete avec `useNodeImpact` n'est pas encore cablée.

### 5.6 Badges (`BadgesTab`)

**Fichier** : `web/src/components/compute/BadgesTab.tsx`
**Props** : aucune (composant autonome)

Affiche une grille de 7 badges en `grid-cols-1 sm:grid-cols-2 xl:grid-cols-3`.

---

## 6. Les 7 badges

| # | ID | Nom | Description | Condition | Icone | Couleur |
|---|---|---|---|---|---|---|
| 1 | `first_task` | Premiere tache | Completez votre premiere tache de calcul | 1 tache completee | `Star` | `#22c55e` |
| 2 | `centurion` | Centurion | 100 taches completees -- contribution significative | 100 taches | `Flame` | `#f59e0b` |
| 3 | `millionnaire` | Millionnaire | 1 000 taches -- pilier du reseau | 1 000 taches | `Crown` | `#a855f7` |
| 4 | `pilier` | Pilier | 10 000 taches -- vous etes indispensable | 10 000 taches | `Award` | `#ec4899` |
| 5 | `always_on` | 24/7 | Uptime continu de plus de 7 jours | 7 jours d'uptime continu | `Clock` | `#06b6d4` |
| 6 | `early_adopter` | Early Adopter | Parmi les 10 premiers contributeurs du reseau | Top 10 premiers inscrits | `Rocket` | `#3b82f6` |
| 7 | `power_node` | Power Node | GPU avec plus de 24 GB de VRAM | VRAM > 24 GB | `Zap` | `#eab308` |

Mapping icone via `ICON_MAP` : `{ star: Star, flame: Flame, crown: Crown, award: Award, clock: Clock, rocket: Rocket, zap: Zap }` (toutes de lucide-react).

---

## 7. Integration API

### 7.1 Fonctions API (`web/src/api/compute.ts`)

Toutes les fonctions utilisent `api` (instance Axios depuis `./client`), base URL vers le backend FastAPI (port 8000).

| Fonction | Methode | Endpoint | Parametres |
|---|---|---|---|
| `getComputeStats` | GET | `/compute/stats` | -- |
| `getComputeNodes` | GET | `/compute/nodes` | `?status=` (optionnel) |
| `getComputeLeaderboard` | GET | `/compute/leaderboard` | `?limit=` (defaut 20) |
| `getComputeModelStatus` | GET | `/compute/model/status` | -- |
| `getComputeModelAssignments` | GET | `/compute/model/assignments` | -- |
| `getComputeModelTransitions` | GET | `/compute/model/transitions` | `?limit=` (defaut 20) |
| `getComputeHybridStatus` | GET | `/compute/hybrid/status` | -- |
| `getComputeSwarmStatus` | GET | `/compute/swarm/status` | -- |
| `getSelfWorkerStatus` | GET | `/compute/self-worker/status` | -- |
| `pauseSelfWorker` | POST | `/compute/self-worker/pause` | -- |
| `resumeSelfWorker` | POST | `/compute/self-worker/resume` | -- |
| `getComputeHealth` | GET | `/compute/health` | -- |
| `getComputeUptime` | GET | `/compute/uptime` | -- |
| `getNodeImpact` | GET | `/compute/nodes/{nodeId}/impact` | path param |
| `getComputeBadges` | GET | `/compute/badges` | `?node_id=` (optionnel) |

### 7.2 Hooks React Query (`web/src/hooks/useCompute.ts`)

Tous les hooks utilisent `@tanstack/react-query` `useQuery`.

| Hook | Query Key | API Function | Refetch Interval | Params |
|---|---|---|---|---|
| `useComputeStats` | `['compute-stats']` | `getComputeStats` | 30s | -- |
| `useComputeNodes` | `['compute-nodes', status]` | `getComputeNodes(status)` | 30s | `status?: string` |
| `useComputeLeaderboard` | `['compute-leaderboard', limit]` | `getComputeLeaderboard(limit)` | 60s | `limit: number` (defaut 20) |
| `useComputeModelStatus` | `['compute-model-status']` | `getComputeModelStatus` | 30s | -- |
| `useComputeHybridStatus` | `['compute-hybrid-status']` | `getComputeHybridStatus` | 30s | -- |
| `useComputeSwarmStatus` | `['compute-swarm-status']` | `getComputeSwarmStatus` | 30s | -- |
| `useSelfWorkerStatus` | `['self-worker-status']` | `getSelfWorkerStatus` | **5s** | -- |
| `useComputeHealth` | `['compute-health']` | `getComputeHealth` | 15s | -- |
| `useComputeUptime` | `['compute-uptime']` | `getComputeUptime` | 60s | -- |
| `useNodeImpact` | `['node-impact', nodeId]` | `getNodeImpact(nodeId)` | 60s | `nodeId: string | null` (`enabled: !!nodeId`) |
| `useComputeBadges` | `['compute-badges', nodeId]` | `getComputeBadges(nodeId)` | 120s | `nodeId?: string` |

Hooks utilises dans `NetworkPage.tsx` : `useComputeStats`, `useComputeLeaderboard(20)`, `useComputeNodes`, `useComputeModelStatus`, `useComputeHybridStatus`, `useComputeSwarmStatus`, `useSelfWorkerStatus`.

Hooks disponibles mais non utilises dans la page : `useComputeHealth`, `useComputeUptime`, `useNodeImpact`, `useComputeBadges`.

---

## 8. Flux de donnees

```
NetworkPage.tsx
  |
  |-- useComputeStats()         -----> getComputeStats()      --> GET /compute/stats          --> statsQ.data
  |-- useComputeLeaderboard(20) -----> getComputeLeaderboard() --> GET /compute/leaderboard   --> leaderboardQ.data
  |-- useComputeNodes()         -----> getComputeNodes()       --> GET /compute/nodes          --> nodesQ.data
  |-- useComputeModelStatus()   -----> getComputeModelStatus() --> GET /compute/model/status   --> modelQ.data
  |-- useComputeHybridStatus()  -----> getComputeHybridStatus()--> GET /compute/hybrid/status  --> hybridQ.data
  |-- useComputeSwarmStatus()   -----> getComputeSwarmStatus() --> GET /compute/swarm/status   --> swarmQ.data
  |-- useSelfWorkerStatus()     -----> getSelfWorkerStatus()   --> GET /compute/self-worker/status --> selfWorkerQ.data
  |
  |-- [MetricCard x5]           <-- stats, leaderboard.total_contributors
  |-- [Self-worker banner]      <-- selfWorker  (+ pauseSelfWorker / resumeSelfWorker POST)
  |
  |-- <StatsTab>                <-- stats, model, hybrid, nodes
  |-- <LeaderboardTab>          <-- leaderboard.entries, leaderboard.total_contributors
  |-- <NodesTab>                <-- nodes
  |-- <SwarmTab>                <-- swarm
  |-- <ContributeTab>           <-- impact=null (hardcoded)
  |-- <BadgesTab>               <-- (no props, static badge definitions)
```

Cycle de rafraichissement :
- Self-worker : toutes les **5 secondes** (haute frequence pour feedback temps reel)
- Stats, nodes, model, hybrid, swarm : toutes les **30 secondes**
- Leaderboard : toutes les **60 secondes**
- Badges : toutes les **120 secondes** (donnees stables)

---

## 9. Sidebar : section "Puissance Citoyenne"

**Fichier** : `web/src/components/AppSidebar.tsx`

La section est le troisieme `SidebarGroup` dans la navigation. Son label est style en emerald : `text-emerald-400/70`.

```tsx
const computeItems: NavItem[] = [
  { to: '/network', icon: Cpu, label: 'Reseau GPU' },
  { to: '/network?tab=leaderboard', icon: Activity, label: 'Leaderboard' },
];
```

| # | Route | Icone | Label |
|---|---|---|---|
| 1 | `/network` | `Cpu` | Reseau GPU |
| 2 | `/network?tab=leaderboard` | `Activity` | Leaderboard |

Detection active : compare `location.pathname + location.search` avec `to`, avec cas special pour `/network` sans query string.

---

## 10. Variables CSS dark theme utilisees

Definies dans `web/src/index.css` sous `:root` (mode sombre permanent, pas de light mode).

### Variables NEXUS custom

| Variable | Valeur | Usage dans Network |
|---|---|---|
| `--bg-primary` | `#0a0a0f` | Fond des barres de progression, fond des mini-cartes nodes |
| `--bg-card` | `#1a1d35` | Fond des MetricCard, fond du bandeau self-worker (arrete) |
| `--bg-hover` | `#22263e` | Hover sur lignes du leaderboard, blocs swarm non couverts |
| `--text-primary` | `#e2e4f0` | Texte principal (valeurs metriques, noms de nodes) |
| `--text-secondary` | `#8b8fa8` | Texte secondaire (modele GPU, descriptions) |
| `--text-muted` | `#565973` | Labels, sous-titres, informations tertiaires |
| `--border` | `rgba(255,255,255,0.08)` | Bordures de toutes les cartes et du bandeau |

### Couleurs Tailwind directes

| Couleur | Hex | Usage |
|---|---|---|
| `emerald-500` | `#22c55e` | Self-worker actif, nodes idle, contributeurs, taches terminees |
| `emerald-400` | -- | Texte vert (tasks, trust score >= 80, message d'impact) |
| `blue-500` | `#3b82f6` | Nodes busy, taches en cours, total contributeurs |
| `yellow-500` | `#f59e0b` | Self-worker pause, taches en attente, degraded swarm |
| `purple-500/400` | `#a855f7` | Modele actif, mode distribue |
| `fuchsia-500/400` | -- | Badge Petals swarm |
| `red-500/400` | `#ef4444` | Taches echouees, trust score < 50, swarm offline |
| `cyan-400` | `#06b6d4` | VRAM totale, badge 24/7, modele distribue swarm |
| `gray-500` | `#6b7280` | GPU arrete, nodes offline |
| `pink-500` | `#ec4899` | Badge Pilier |

---

## 11. Arbre des composants

```
App.tsx
  QueryClientProvider
    TooltipProvider
      BrowserRouter
        Routes
          Route element={<Layout />}
            Route path="/network" element={<NetworkPage />}

Layout.tsx
  SidebarProvider
    AppSidebar                          (web/src/components/AppSidebar.tsx)
      SidebarGroup "Puissance Citoyenne"
        SidebarMenuItem "/network"       -- Reseau GPU
        SidebarMenuItem "/network?tab=leaderboard" -- Leaderboard
    SidebarInset
      TopBar
      <Outlet />  -->  NetworkPage

NetworkPage.tsx                          (web/src/pages/NetworkPage.tsx)
  MetricCard x5                          (composant interne)
  Self-worker banner                     (JSX inline)
  Tabs
    TabsList (variant="line")
      TabsTrigger "stats"                -- Statistiques
      TabsTrigger "leaderboard"          -- Leaderboard
      TabsTrigger "nodes"                -- Nodes ({count})
      TabsTrigger "swarm"                -- Swarm Petals
      TabsTrigger "contribute"           -- Ma contribution
      TabsTrigger "badges"               -- Badges
    TabsContent "stats"
      StatsTab                           (web/src/components/compute/StatsTab.tsx)
        TaskBar x4                       (composant interne)
        InfoRow x6                       (composant interne)
        NodeCard grid                    (composant interne, reutilise les nodes)
    TabsContent "leaderboard"
      LeaderboardTab                     (web/src/components/compute/LeaderboardTab.tsx)
    TabsContent "nodes"
      NodesTab                           (web/src/components/compute/NodesTab.tsx)
        NodeCard x N                     (composant interne)
    TabsContent "swarm"
      SwarmTab                           (web/src/components/compute/SwarmTab.tsx)
    TabsContent "contribute"
      ContributeTab                      (web/src/components/compute/ContributeTab.tsx)
        StatCard x4                      (composant interne)
    TabsContent "badges"
      BadgesTab                          (web/src/components/compute/BadgesTab.tsx)
```

---

## 12. Fichiers references

| Fichier | Role |
|---|---|
| `web/src/pages/NetworkPage.tsx` | Page principale, orchestrateur des hooks et onglets |
| `web/src/components/compute/StatsTab.tsx` | Onglet statistiques (file de taches, modele, distribution GPU) |
| `web/src/components/compute/LeaderboardTab.tsx` | Onglet classement contributeurs |
| `web/src/components/compute/NodesTab.tsx` | Onglet liste des nodes en ligne / hors ligne |
| `web/src/components/compute/SwarmTab.tsx` | Onglet Petals swarm (couverture blocs, sante) |
| `web/src/components/compute/ContributeTab.tsx` | Onglet contribution personnelle |
| `web/src/components/compute/BadgesTab.tsx` | Onglet badges (7 badges statiques) |
| `web/src/api/compute.ts` | 15 fonctions API (Axios vers `/compute/*`) |
| `web/src/hooks/useCompute.ts` | 11 hooks React Query avec refetch intervals |
| `web/src/components/AppSidebar.tsx` | Navigation sidebar, section "Puissance Citoyenne" |
| `web/src/App.tsx` | Routeur, route `/network` |
| `web/src/index.css` | Variables CSS dark theme |
