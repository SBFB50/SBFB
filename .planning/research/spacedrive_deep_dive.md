# Deep Dive Spacedrive — Patterns UX/UI pour SBFB

**Date** : 2026-04-27
**Source** : github.com/spacedriveapp/spacedrive (35k+ stars, AGPL-3.0)
**Méthode** : Clone repo + analyse code source directe
**Objectif** : Extraire les patterns techniques précis réutilisables pour le shell SBFB.

---

## 1. Stack technique exacte

| Aspect | Spacedrive | SBFB actuel | Compatible ? |
|--------|-----------|-------------|-------------|
| React | 19.1.0 | 19.2.4 | Oui |
| Routing | react-router-dom 6.20.1 | react-router-dom 7.14.0 | Oui (même API) |
| State | Zustand 5.0.8 + React Query 5.90 | Zustand 5.0.12 + React Query 5.96 | Identique |
| Styling | Tailwind CSS v4 | Tailwind CSS v4 | Identique |
| Animations | framer-motion 12.23 | Non utilisé | À ajouter (`motion/react`) |
| Icons | Phosphor Icons (`@phosphor-icons/react`) | Lucide React | À évaluer |
| UI primitives | Radix UI (via @spacedrive/primitives) | @base-ui + Radix (shadcn) | Compatible |
| Virtualisation | @tanstack/react-virtual | Non utilisé | À ajouter si Browse > 50 apps |
| Tables | @tanstack/react-table | Non utilisé | Pour vues liste |
| DnD | @dnd-kit/core + sortable | Non utilisé | Pour réordonner sidebar/widgets |
| Forms | react-hook-form | Non utilisé | Pour Settings page |
| Package mgr | Bun | npm | Non bloquant |
| Design system | @spacedrive/tokens + primitives (repo séparé spaceui) | shadcn/ui | On garde shadcn, on adopte les tokens |

---

## 2. Palette de couleurs — système sémantique (hue 235)

Spacedrive utilise un système de **tokens sémantiques** basé sur une hue unique (235 = bleu-violet) déclinée en luminosités. Chaque surface a un nom, pas une valeur brute.

### Surfaces (dark theme, hue 235)

```
app             hsl(235, 15%, 13%)   — background principal
app-box         hsl(235, 15%, 18%)   — cards / containers
app-dark-box    hsl(235, 15%, 15%)   — containers plus sombres
app-darker-box  hsl(235, 16%, 11%)   — fond le plus sombre
app-overlay     hsl(235, 15%, 17%)   — overlays
app-line        hsl(235, 15%, 23%)   — bordures
app-hover       hsl(235, 15%, 19%)   — hover state (+2 luminosité)
app-selected    hsl(235, 15%, 24%)   — sélection
app-input       hsl(235, 15%, 20%)   — inputs
app-frame       hsl(235, 15%, 25%)   — inner frame pseudo-element
```

### Sidebar (plus sombre que app)

```
sidebar         hsl(235, 15%, 7%)    — quasi-noir
sidebar-box     hsl(235, 15%, 16%)   — containers sidebar
sidebar-line    hsl(235, 15%, 23%)   — bordures sidebar
sidebar-selected hsl(235, 15%, 24%)  — item actif
```

### Texte (ink)

```
ink             hsl(235, 35%, 92%)   — primaire (quasi-blanc bleuté)
ink-dull        hsl(235, 10%, 70%)   — secondaire
ink-faint       hsl(235, 10%, 55%)   — tertiaire
```

### Accent (bleu)

```
accent          hsl(208, 100%, 57%)  — actions primaires
accent-faint    hsl(208, 100%, 64%)  — hover accent
accent-deep     hsl(208, 100%, 47%)  — pressed accent
```

### Menu (dropdowns, context menus)

```
menu            hsl(235, 15%, 10%)   — fond menu
menu-hover      hsl(235, 15%, 30%)   — item hover
menu-ink        hsl(235, 25%, 92%)   — texte menu
```

### Adaptation SBFB recommandée

Garder le **même système sémantique** (app/sidebar/ink/accent/menu) mais avec notre propre hue. Notre accent actuel est violet (#7c3aed) — on peut garder hue 235 (très proche) ou passer à hue 270 (plus violet).

---

## 3. Layout principal — ShellLayout

### Structure

```
┌──────────────────────────────────────────────────┐
│ [TopBar — 48px, position absolute, z-60]         │
├────────┬─────────────────────────────┬───────────┤
│Sidebar │  Content area (pt-12)       │ Inspector │
│ 220px  │  [TabBar]                   │   280px   │
│bg-side │  [Outlet — route content]   │ bg-side   │
│bar/65  │                             │ bar/65    │
│rounded │                             │ rounded   │
│-2xl    │                             │ -2xl      │
│        │                             │           │
│[bottom │                             │           │
│ bar]   │                             │           │
└────────┴─────────────────────────────┴───────────┘
```

### Sidebar — 220px

- Largeur : `w-[220px] min-w-[176px] max-w-[300px]`
- Background : `bg-sidebar/65` (65% opacité), `backdrop-blur-2xl` en preview mode
- Container : `rounded-2xl`, padding `p-2.5`
- Animation open/close : framer-motion, `x: -220 → 0`, `duration: 0.3`, ease `[0.25, 1, 0.5, 1]`
- Scroll fade bottom : `mask-image: linear-gradient(to bottom, black calc(100% - 40px), transparent 100%)`
- Items : `flex items-center gap-2 px-2 py-1 rounded-md text-sm font-medium`
  - Actif : `bg-sidebar-selected text-sidebar-ink`
  - Inactif : `text-sidebar-inkDull hover:text-sidebar-ink`
- Titres de groupe : `text-[11px] font-semibold text-sidebar-inkFaint uppercase tracking-wider`
- Bottom bar : boutons Sync/Jobs/Settings ancrés en bas

### TopBar — 48px

- `h-12 absolute top-0 z-[60]`
- Pas de background propre — flotte au-dessus du contenu
- Gradient fade : `h-32 bg-gradient-to-b from-app to-transparent` en dessous
- **Pattern portal** : chaque route injecte ses boutons via `<TopBarPortal left={...} right={...} />`
- Boutons : `CircleButton` ronds avec icônes Phosphor
- Blur : `backdrop-filter: saturate(120%) blur(18px)` + `border-app-line/50`

### Inspector — 280px (panneau droit)

- Animation : framer-motion `width: 0 → 280`, même ease
- Background : identique sidebar (`bg-sidebar/65`, `rounded-2xl`)
- Sections pliables avec AnimatePresence
- Variantes : FileInspector, LocationInspector, EmptyState

---

## 4. Composants clés — détails techniques

### GridView (→ notre Browse apps)

- CSS Grid : `grid-template-columns: repeat(auto-fill, minmax(${gridSize}px, 1fr))`
- `gridSize` défaut 120px, `gapSize` 16px
- **Virtualisation** toujours active (@tanstack/react-virtual)
- FileCard : thumbnail + nom tronqué + taille + tag dots
- Sélection : `bg-accent text-white` sur le nom
- DragSelect : lasso rectangle
- Keyboard nav : flèches avec calcul colonnes dynamique

### FileCard (→ notre AppCard)

```
flex flex-col items-center gap-2 p-1 rounded-lg transition-all
```
- Thumbnail : `Math.max(gridSize * 0.6, 60)` px
- Nom : `text-sm truncate px-2 py-0.5 rounded-md`
- Tag dots : `size-1.5 rounded-full` avec couleur dynamique
- Drop target : `ring-2 ring-accent ring-inset`
- Drag : opacité 40%

### StatCard / HeroStats (→ notre Overview cards)

- Carousel horizontal scrollable avec boutons flèche
- Fade gradients aux bords : `bg-gradient-to-r from-app to-transparent`
- Chaque card : icône 40px + valeur `text-3xl font-bold` + label `text-xs text-ink-dull`
- Pas de glassmorphism — flat design sur les stat cards

### JobCard (→ notre Task card)

- `rounded-xl border border-app-line/30 bg-app-box`
- Expandable : AnimatePresence `height: 0 → "auto"`, `duration: 0.15`
- Progress bar en bas
- Status indicator à gauche (spinner animé, check, X)
- Actions au hover (pause/resume/cancel)

### TagDot / TagPill (→ nos category badges / curator endorsements)

- TagDot : `size-1.5 rounded-full`, hover `scale-125 transition-transform`
- TagPill : `rounded-full`, background `${color}20` (20% opacité), texte couleur du tag

### Context Menu

- Hook `useContextMenu` déclaratif
- Style web : `bg-menu border-menu-line rounded-lg py-1 shadow-2xl`
- Items : `mx-1 px-2 py-1 rounded-md text-sm`, hover `data-[highlighted]:bg-menu-hover`
- Variant danger : `text-status-error`
- Séparateurs : `bg-menu-line mx-1 my-1 h-px`

---

## 5. Glassmorphism — usage ciblé, pas partout

Spacedrive utilise le glassmorphism de manière **chirurgicale** :

| Où | Comment | Quand |
|----|---------|-------|
| Sidebar | `bg-sidebar/65 backdrop-blur-2xl` | Toujours |
| Inspector | `bg-sidebar/65 backdrop-blur-2xl` | Toujours |
| TopBar | `saturate(120%) blur(18px)` | Toujours |
| Content gradient | `h-32 from-app to-transparent` | Toujours |
| Sidebar scroll | `mask-image gradient` fade bottom | Toujours |

**Jamais sur** : cards, buttons, inputs, tables, menus. Les cards utilisent `bg-app-box` opaque.

**Inner frame glow** (subtil) :
```css
@utility frame {
  &::before {
    content: "";
    position: absolute;
    inset: 0px;
    border-radius: inherit;
    padding: 1px;
    background: var(--color-app-frame);
    mask: linear-gradient(black, black) content-box, linear-gradient(black, black);
    mask-composite: xor;
  }
}
```

---

## 6. Animations — constantes et patterns

| Type | Duration | Easing | Usage |
|------|----------|--------|-------|
| Layout (sidebar/inspector) | 0.3s | `[0.25, 1, 0.5, 1]` (ease-out quart) | Open/close panels |
| Micro-interaction | 0.15s | même ease | Expand/collapse, menu appear |
| Startup overlay | 0.6s | `easeInOut` | Logo scale 0.8→1 |
| Pulse indicator | 2s infinite | linear | `scale: [1, 1.2, 1], opacity: [1, 0.5, 1]` |
| Hover | CSS | `transition-colors` | Tailwind utility |
| List items enter | 0.15s | ease | `opacity: 0, y: -10` → visible |
| List items exit | 0.15s | ease | `opacity: 0, x: -10` → gone |

---

## 7. Typographie

```
--font-sans: "Inter", system-ui, sans-serif
--font-mono: "JetBrains Mono", "Menlo", "Monaco", monospace
```

Échelle plus serrée que Tailwind default :
- `text-tiny: 0.7rem` (11.2px)
- `text-xs: 0.75rem` (12px)
- `text-sm: 0.8rem` (12.8px)
- `text-base: 0.875rem` (14px)

Notre stack utilise Geist — compatible, même approche (sans-serif technique).

---

## 8. Correspondances directes Spacedrive → SBFB

| Spacedrive | SBFB | Adaptation nécessaire |
|------------|------|----------------------|
| SpacesSidebar (220px) | Sidebar gauche (68px actuellement) | Élargir à ~200px, ajouter sections groupées, bottom bar |
| GridView + FileCard | Browse apps grid | Remplacer thumbnails fichier par app icons/screenshots |
| DevicePanel cards | Worker/peer cards (Network page) | Adapter : GPU model, VRAM bar, tasks count, kudos |
| JobCard expandable | Task queue display | Même pattern : card + AnimatePresence + progress |
| Inspector 280px | ProjectDetail / App detail panel | Panel droit contextuel |
| TopBar portal | TopBar SBFB | Chaque page injecte ses contrôles |
| TagDot/TagPill | Category badges, curator endorsements | Couleur dynamique, pill `rounded-full` |
| Settings grid | Future Settings/Identity page | Sections + `bg-app-box rounded-lg border` |
| Explorer viewMode toggle | Browse grid/list switch | TopBar toggle button |
| HeroStats carousel | Overview métriques | Cards horizontales scrollables |
| `mask-fade-out` | Sidebar scroll | Copy-paste utility |
| `frame` utility | Container inner glow | Copy-paste utility |
| `useContextMenu` | Right-click menus SBFB | Pattern déclaratif |

---

## 9. Ce qu'on adopte vs ce qu'on garde

### Adopter de Spacedrive

1. **Système de tokens sémantiques** : `app/app-box/app-line/app-hover/app-selected` + `sidebar/*` + `ink/ink-dull/ink-faint` + `accent/*` + `menu/*`. Remplace nos couleurs ad-hoc.
2. **Sidebar élargie ~200px** avec sections, scroll fade, bottom bar
3. **TopBar portal pattern** : chaque page injecte ses contrôles
4. **Animations framer-motion** : `duration: 0.3` layout, `0.15` micro, ease `[0.25, 1, 0.5, 1]`
5. **Glassmorphism ciblé** : sidebar + inspector seulement, pas sur les cards
6. **Utilities CSS** : `frame`, `mask-fade-out`, `top-bar-blur`, `no-scrollbar`
7. **GridView virtualisé** pour Browse si > 50 apps

### Garder de SBFB

1. **shadcn/ui** comme design system de base (Spacedrive a son propre, le nôtre est plus standard)
2. **Lucide icons** (sauf si Phosphor offre un avantage clair — à évaluer)
3. **Geist font** (Inter et Geist sont très proches)
4. **Structure routing** actuelle (lazy routes, code-split)
5. **Zod validation** API (Spacedrive n'utilise pas Zod côté frontend)
6. **Bridge postMessage** (spécifique SBFB, pas d'équivalent Spacedrive)

---

## 10. Plan d'intégration suggéré (par sprint)

### Quick wins (pas de changement d'architecture)

- Adopter les tokens sémantiques dans `web/src/index.css`
- Ajouter les utilities CSS (`frame`, `mask-fade-out`, `top-bar-blur`)
- Installer `motion/react` pour les animations layout
- Ajuster la palette : `#0a0a0f` → tokens `app` / `app-box` / `app-line`

### Évolutions moyennes (refactor composants)

- Élargir sidebar de 68px à ~200px avec sections et bottom bar
- Implémenter TopBar portal pattern
- Refaire les cards Browse en GridView avec `auto-fill` + `minmax`
- Ajouter le pattern JobCard expandable pour les tâches

### Évolutions majeures (nouvelles pages)

- Inspector panel droit (280px) pour ProjectDetail et BrowsedProject info
- Settings page avec sections `bg-app-box rounded-lg border`
- Grid/List toggle pour Browse
- Context menus (right-click) sur les cards

---

## 11. Ce document n'est PAS

- Un plan de sprint (pas de phases, pas de commits, pas de D1-D5)
- Du code (aucun fichier touché)
- Une décision figée (les choix restent à valider dans un kickoff)

C'est une extraction technique du design system Spacedrive pour informer un futur sprint frontend SBFB.
