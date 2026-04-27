# Catalogue d'inspiration frontend — Projets open source + patterns UX

**Date** : 2026-04-27
**Méthode** : 3 agents parallèles (P2P UI, dashboards React modernes, design patterns local-first)
**Objectif** : Identifier les projets GitHub et patterns UX pour élever le front SBFB.
**Complément de** : `frontend_ux_protocol_analysis.md` (inventaire gaps)

---

## 1. Projets de référence — Top 10

### Tier S — Même stack, directement applicable

| Projet | Stars | Stack | Licence | Pertinence |
|--------|-------|-------|---------|------------|
| **[Spacedrive](https://github.com/spacedriveapp/spacedrive)** | ~35k | React + TS + Tailwind + Radix | AGPL-3.0 | Design reference absolue. Glassmorphic dark, file explorer distribué, job queue viz, sidebar+content layout. Le plus beau frontend P2P open source. |
| **[shadcn-admin](https://github.com/satnaing/shadcn-admin)** | ~11k | React + Vite + TS + shadcn/ui + Tailwind | MIT | **Même stack exacte.** 10+ pages, sidebar collapsible, command palette, dark mode. Architecture de référence directe. |
| **[Akash Console](https://github.com/akash-network/console)** | ~200 | Next.js + TS + shadcn/ui + Tailwind | Apache-2.0 | Compute marketplace. Deploy wizard, GPU provider cards, deployment logs. Composants shadcn customisés réutilisables. |
| **[Magic UI](https://github.com/magicuidesign/magicui)** | ~19k | React + TS + Tailwind + Motion | MIT | Companion shadcn natif. 150+ composants animés copy-paste : animated beams (connexions P2P), globe (network), dock, number tickers. |

### Tier A — Patterns UX clés à étudier

| Projet | Stars | Stack | Licence | Pertinence |
|--------|-------|-------|---------|------------|
| **[Umbrel](https://github.com/getumbrel/umbrel)** | ~8k | Vue 3 + TS + Tailwind | PolyForm NC | App store UI parfait : grid cards + catégories + 1-click install + dashboard widgets. Design glassmorphic. |
| **[Coolify](https://github.com/coollabsio/coolify)** | ~38k | Nuxt 3 + Tailwind | Apache-2.0 | Deploy from Git, service status cards, build logs streaming. Analogie directe avec notre verified deploy. |
| **[CasaOS](https://github.com/IceWhaleTech/CasaOS-UI)** | ~28k | Vue 3 + TS + Vite | Apache-2.0 | Consumer-friendly. System monitor temps réel, drag-drop widgets, badge "Official" vs "Community". |
| **[Tremor](https://github.com/tremorlabs/tremor)** | ~3k | React + Tailwind + Recharts + Radix | Apache-2.0 | 35+ composants data dashboard : tracker (uptime), spark charts (GPU), KPI cards (network stats). Composable avec shadcn. |
| **[Dub.co](https://github.com/dubinc/dub)** | ~22k | Next.js + Tailwind + SWR | AGPL-3.0 | Design system production (@dub/ui). Analytics dashboard. Référence qualité finition. AGPL comme nous. |
| **[React Bits](https://github.com/DavidHDev/react-bits)** | ~32k | React + TS + Tailwind | MIT | 110+ composants animés. Text animations, backgrounds dynamiques. Installable via shadcn CLI. |

### Tier B — Librairies spécialisées

| Librairie | Stars | Usage SBFB |
|-----------|-------|-----------|
| **[Motion](https://github.com/motiondivision/motion)** | ~31k | Animations layout, page transitions, spring physics. `motion/react`. |
| **[react-force-graph](https://github.com/vasturiano/react-force-graph)** | ~2k | Graphe réseau P2P interactif (2D/3D). Peers = noeuds, gossip = liens. |
| **[Reagraph](https://github.com/reaviz/reagraph)** | ~1k | Alternative WebGL au graphe. Clustering auto, API React-native. |
| **[cmdk](https://github.com/pacocoursey/cmdk)** | ~10k | Command bar Cmd+K. Déjà dans notre stack (cmdk 1.1.1). |
| **[sonner](https://github.com/emilkowalski/sonner)** | ~10k+ | Toast notifications élégantes. |
| **[jdenticon](https://github.com/nickkwas/jdenticon)** | ~1k+ | Identicons déterministes depuis Ed25519 pubkey. |

---

## 2. Patterns UX par domaine — applicables à SBFB

### 2.1 App Store / Browse

**Sources** : Umbrel, Steam, VS Code Marketplace, Chrome Web Store, Docker Hub, F-Droid

| Pattern | Source | Application SBFB |
|---------|--------|-----------------|
| Grid cards + catégories horizontales | Umbrel | Page `/browse` : filtres par category scrollables en haut |
| Tri-level trust badges | Docker Hub | "Official" (apps SBFB) / "Source-verified" (SLSA) / "Community" (non vérifié) |
| Anti-features badges | F-Droid | Badges warning orange : "Large download", "Requires GPU", "Unverified source" |
| Permissions display pre-install | Chrome | "This app can: submit tasks, read/write storage, no network access (sandboxed)" |
| Verified Publisher badge | VS Code | Checkmark bleu si deploy-from-repo + Keyoxide proof |
| Curator collections | Steam | "Curator X recommends 12 apps for data science" — page curatée |
| Screenshots carousel | Steam/Umbrel | Hero section Browse avec preview de l'app dans l'iframe |
| Grid/List toggle + keyboard nav | Chonky | Browse switcher vue grille / vue liste compact |

**Wireframe recommandé — Browse card** :
```
┌─────────────────┐
│  [app icon]      │
│  App Name        │
│  ★ Source-verified│  ← badge vert SLSA
│  by 3 curators   │  ← endorsements count
│  "Description..." │
│  1.2 MB · 42 nodes│  ← archive size + serving count
└─────────────────┘
```

### 2.2 Network / Peers

**Sources** : Syncthing, IPFS Desktop, Linear, Spacedrive

| Pattern | Source | Application SBFB |
|---------|--------|-----------------|
| Peer list avec health indicators | Syncthing | Table : node_id, type, reachability, bandwidth, latence |
| Status bar ambient (jamais bloquant) | Linear | Micro-indicateur en bas : peers count + sync dot |
| Job queue visualization | Spacedrive | File d'attente tâches : pending → running → done |
| Force-directed graph interactif | react-force-graph | Vue optionnelle topologie P2P (noeuds + liens gossip) |
| Device cards | Syncthing | Chaque peer = card avec identité, type, métriques |

**Anti-pattern à éviter** : Worldmap globe (impressionnant en démo, inutile en pratique).

### 2.3 Worker / GPU Compute

**Sources** : NVIDIA DCGM, vast.ai, RunPod, Salad.com, Akash

| Pattern | Source | Application SBFB |
|---------|--------|-----------------|
| GPU utilization bar + temp gauge | DCGM/Grafana | Card worker : VRAM bar, utilization %, temperature °C |
| "Earning while you sleep" hero metric | Salad | "Kudos earned today: +42" comme métrique principale |
| Provider cards avec specs filtrables | Akash/vast.ai | Liste workers : GPU model, VRAM, uptime, tasks completed |
| Sparkline mini-charts | Tremor | 30 min historique à côté de chaque métrique temps réel |
| Task current + progress | RunPod | Card dernière tâche : hash, status, progress bar, prompt preview |

**Wireframe recommandé — Worker card** :
```
┌──────────────────────────────────────┐
│ Worker Status: ACTIVE    Kudos: +42/j│
├──────────────────────────────────────┤
│ RTX 5080 │ ████████░░ 78% │ 12/16 GB│
│ 72°C     │ 280W          │ L2 consent│
├──────────────────────────────────────┤
│ Current: llm-inference-3a8f (73%)    │
│ By: curator:nexus-ai (★ 1.2k kudos) │
└──────────────────────────────────────┘
```

### 2.4 Identity / Local-First

**Sources** : AnyType, Quiet, Keyoxide, Element

| Pattern | Source | Application SBFB |
|---------|--------|-----------------|
| Identity = keypair locale, pas de login | AnyType | Premier lancement : "Creating your network identity..." + identicon |
| Identicon déterministe depuis clé | jdenticon | Avatar unique généré depuis node_id Ed25519 |
| QR code pour pairing cross-device | Element | Futur : scanner QR pour lier un second device |
| Claim verification badges | Keyoxide | Profil curator : liste claims vérifiés (GitHub, forge, DNS) |
| "You own your identity" messaging | AnyType | Onboarding : node_id = identité, pas de reset possible |

### 2.5 Trust / Provenance

**Sources** : Sigstore, npm provenance, F-Droid, Docker Hub

| Pattern | Source | Application SBFB |
|---------|--------|-----------------|
| Provenance chain visual | Sigstore | repo → build → sign → réseau (étapes cliquables) |
| Verified badge bleu | VS Code/npm | Badge "Source-verified" sur Browse cards |
| Vulnerability scan display | Docker Hub | Futur : audit automatisé archives, compteurs severity |
| "Report" button discret | Chrome Store | Flag communautaire sur BrowsedProject |

**Wireframe recommandé — Provenance modal** :
```
┌──────────────────────────────────────┐
│ Provenance — App Name                │
├──────────────────────────────────────┤
│ [git] → [build] → [sign] → [network]│
│                                      │
│ Source: github.com/foo/bar           │
│ Commit: a3f2c8d (2026-04-25)        │
│ Hash:   7c8d...ef12 (BLAKE3)        │
│ Signed: Ed25519 by node:8a3f...     │
│ SLSA:   Level 1 ✓                   │
│ Nodes:  42 serving this archive     │
│                                      │
│ [View source ↗] [Copy hash] [Close] │
└──────────────────────────────────────┘
```

### 2.6 Esthétique / Design System

**Sources** : Warp, Vercel, Raycast, Hack The Box, cool-retro-term

| Élément | Recommandation | Source |
|---------|---------------|--------|
| Background | Gradient subtil `#0a0a0f → #0f0f1a` (pas noir plat) | Warp |
| Borders | `1px #1a1a2e` (visible mais discret) | Vercel |
| Accent primary | Violet vif `#7c3aed` (tailwind violet-600) | Warp/Tailwind |
| Accent glow hover | `box-shadow: 0 0 20px rgba(accent, 0.15)` | cool-retro-term |
| Text primary | `#e4e4e7` (pas blanc pur `#fff`) | Vercel |
| Text secondary | `#71717a` (tailwind zinc-500) | Tailwind |
| Monospace data | JetBrains Mono / Geist Mono pour hashes, IDs | Warp |
| Cards | `backdrop-blur-xl + bg-white/5` (glassmorphic) | Umbrel |
| Hover | lighten 5%, active lighten 10% | Vercel |
| Transitions | Spring physics 200-300ms (Motion) | Raycast |

---

## 3. Librairies à intégrer — plan d'action

### Déjà dans la stack (à mieux exploiter)

| Lib | Version actuelle | Usage actuel | Potentiel inexploité |
|-----|-----------------|-------------|---------------------|
| cmdk | 1.1.1 | CommandPalette | Recherche apps, navigation peers, actions rapides |
| recharts | (via TabView) | ChartLineBlock, ChartBarBlock | GPU metrics sparklines, network stats |
| lucide-react | 1.7.0 | Icônes | Cohérent, rien à changer |
| @base-ui/react | 1.3.0 | Headless components | Continuer la migration depuis Radix |

### À ajouter (compatible stack)

| Lib | Raison | Intégration |
|-----|--------|------------|
| **Magic UI** (composants sélectifs) | Animated beams, globe, number ticker, dock | Copy-paste via shadcn CLI, MIT |
| **Tremor** (composants sélectifs) | Tracker, SparkChart, KPI cards pour monitoring | Import direct, même base Radix+Tailwind |
| **Motion** (framer-motion successor) | Layout animations, page transitions, presence | `npm i motion`, import `motion/react` |
| **jdenticon** | Identicons depuis Ed25519 pubkey | ~10 KB, zéro dep |
| **sonner** | Toast notifications (remplace potentiel existant) | ~5 KB, shadcn-compatible |
| **react-force-graph** | Graphe réseau P2P page Network | ~50 KB, Canvas/WebGL |

### À NE PAS ajouter

| Lib | Raison du rejet |
|-----|----------------|
| D3.js complet | Trop lourd, react-force-graph suffit pour le graphe |
| Three.js | Overkill sauf si 3D mandatoire |
| Framer Motion legacy | Migré vers `motion/react`, utiliser le nouveau nom |
| MUI/Ant Design | Conflit avec shadcn/ui, philosophie différente |
| Chart.js | Recharts déjà dans la stack, pas de raison de changer |

---

## 4. Synthèse — ce qu'on retient

### 3 projets à cloner et étudier en priorité

1. **shadcn-admin** — Architecture shell identique (Vite+React+shadcn). Étudier le layout, la sidebar, le command palette, le routing.
2. **Spacedrive** — Le gold standard du frontend P2P. Étudier le glassmorphic, la sidebar, les job queues, les device cards.
3. **Akash Console** — Compute marketplace shadcn. Étudier les GPU cards, le deploy wizard, les provider filters.

### 5 composants Magic UI à intégrer en premier

1. **Animated Beam** — Visualiser les connexions P2P entre daemon et peers
2. **Number Ticker** — Métriques temps réel (peers count, kudos, tasks)
3. **Globe** — Hero section page Network (optionnel, spectaculaire)
4. **Dock** — Navigation apps installées (alternative à la sidebar)
5. **Marquee** — Défilement apps trending sur la page Browse

### 3 composants Tremor pour le monitoring

1. **Tracker** — Uptime workers sur 30 jours (barres vert/rouge)
2. **SparkChart** — Mini-graphes GPU util / VRAM / temperature inline
3. **BarList** — Top workers par kudos, top apps par usage

### Anti-patterns stricts

- **Pas de worldmap globe pour les peers** (joli mais inutile)
- **Pas de leaderboard** (Matthew effect, cf. fairness_vision.md)
- **Pas de gamification** (streaks, levels, badges bronze/silver/gold)
- **Pas de modales bloquantes pour le sync** (toujours ambient)
- **Pas de fond noir plat #000** (toujours gradient subtil)
- **Pas de light mode** (dark-only, cohérent avec l'identité)

---

## 5. Ce document n'est PAS

- Un plan de sprint (pas de phases, pas de commits)
- Du code (aucun fichier touché)
- Une décision D1-D5 (les choix de libs restent à geler dans un kickoff)

C'est un catalogue d'inspiration pour alimenter les décisions Day 0 d'un futur sprint frontend.
Les wireframes sont indicatifs — le design final sera itéré dans le sprint.
