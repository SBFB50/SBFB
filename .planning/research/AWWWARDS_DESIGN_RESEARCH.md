# Research: Awwwards-Level Design for NEXUS GOV Dashboard
**Domain:** Premium dark-theme React dashboard — shadcn/ui + Tailwind CSS 4 + Vite 8 + React 19
**Researched:** 2026-04-10
**Overall confidence:** HIGH (multiple verified sources; official docs confirmed)

---

## 1. Awwwards Design Patterns in 2025-2026

### What Award-Winning Sites Actually Do (Evidence-Based)

Awwwards-level quality in 2025-2026 is defined by **purposeful motion + information density + tactile depth**. The design language shifted away from decorative animations toward animations that carry semantic meaning: a counter that rolls up signals data freshness; a card that tilts in 3D signals interactivity; a background with subtle noise signals depth without distraction.

Key distinguishing traits observed across winning dark-theme dashboards:

**Motion architecture (not decoration)**
- Every UI state change is animated: tab switches, data loads, number updates, filter toggles
- Staggered list entrances (50–80ms delay between items) make data feel alive
- Exit animations match entrance animations (no jarring disappearances)

**Depth through layering**
- Three distinct visual planes: deep background (WebGL/shader or CSS gradient), mid-layer (cards with backdrop-blur), foreground (interactive elements)
- Subtle border gradients (`border: 1px solid transparent; background-clip: padding-box`) create premium feel
- Noise/grain textures at 3-8% opacity break the "too digital" flatness of pure dark UIs

**Typography as a design element**
- Variable font weight axes used dynamically (numbers animate weight during count-up)
- Font feature settings: `font-feature-settings: 'tnum' 1` for tabular numbers in dashboards
- Fluid type scaling with `clamp()` — no breakpoint jumps

**Microinteractions on every interactive element**
- Buttons: scale 0.97 on press (not 0.95 — too much), glow on hover
- Cards: 2-4° tilt toward cursor, subtle glow follows cursor position
- Tab indicators: sliding pill with spring physics, not CSS transition
- Sidebar links: left-border accent animates width on hover/active

---

## 2. Animation Libraries — Stack Decision

### Definitive Recommendation: Framer Motion (Motion) as the Primary Layer

**Confidence: HIGH** — verified via official docs, multiple ecosystem sources

Framer Motion (rebranded to `motion` package in v11+) is the correct choice for NEXUS GOV because:

1. It integrates natively with shadcn/ui components via `motion()` wrapper
2. It handles all four animation needs: entrance, gesture, layout, page transitions
3. React 19 compatibility confirmed — Motion works with concurrent rendering
4. The project already uses React 19.2 + Vite 8 (no conflicts)

**Bundle size reality:**
- Full `motion` import: ~34kB (non-tree-shakeable)
- `LazyMotion` + `m` components + `domAnimation`: ~19.6kB deferred
- `LazyMotion` + `m` components + `domMax` (includes drag/layout): ~25kB deferred
- For a dashboard at 71K+ LOC, this is negligible

**Installation:**
```bash
npm install motion
```

**Do NOT use `framer-motion` package directly** — it is the old package name. The current package is `motion` (v12+).

### Secondary: Lenis for Smooth Scroll

**Confidence: HIGH** — official package, darkroom.engineering

Required for the scroll-triggered sections (if any page-level scrolling exists). Provides the visceral "premium scroll" feel.

```bash
npm install lenis
```

Usage in `main.tsx` or layout wrapper:
```tsx
import { ReactLenis } from 'lenis/react'
// Wrap root: <ReactLenis root options={{ lerp: 0.1, duration: 1.2 }}>
```

**GSAP is NOT recommended** for this project for the following reasons:
- GSAP ScrollTrigger solves the same problem as `motion`'s `whileInView` but with 10x the API surface
- GSAP ScrollTrigger has React integration friction (cleanup in `useEffect`, gsap.context() required)
- GSAP's free tier does not include SplitText (typography splits require the paid Club GSAP license)
- For dashboards (not scroll-storytelling sites), Motion's `whileInView` + `viewport` is sufficient

**React Spring is NOT recommended** for this project:
- Spring physics is excellent for physics-based gestures (dragging, throwing)
- The API is significantly more verbose than Motion for standard entrance/hover animations
- No built-in layout animation support (Motion's `layoutId` is uniquely powerful for tab indicators)

### Supporting: react-countup / number-flow for Metric Cards

**number-flow** (MIT, ~6.8kB gzipped, dependency-free): Recommended for animated stat counters. Built on Web Animations API, not JS loops. Apple-quality number transitions. Supports scroll-spy trigger.

```bash
npm install @number-flow/react
```

Canonical usage for a metric card KPI:
```tsx
import NumberFlow from '@number-flow/react'
<NumberFlow value={deputies} format={{ notation: 'compact' }} />
```

**react-countup** is the alternative if number-flow's style feels too subtle. It has `enableScrollSpy` prop for viewport triggering.

---

## 3. Component Library Ecosystem for shadcn-Compatible Premium Components

### Tier 1: Copy-Paste Components (Full Code Ownership)

These follow the same philosophy as shadcn — you own the code, no runtime dependency.

**Aceternity UI** — `ui.aceternity.com`
The gold standard for motion-heavy shadcn components in 2025. 200+ components built with Tailwind + Motion. The most relevant for NEXUS GOV:

| Component | Use in NEXUS GOV |
|-----------|-----------------|
| Aurora Background | Hero/header background behind stats |
| Card Spotlight | Hover glow that follows cursor on metric cards |
| Background Beams | Empty state / loading placeholders |
| 3D Card Effect | Politician profile cards |
| Glowing Effect | Active/selected state for sidebar items |
| Moving Border | Active scan status indicator |
| Tracing Beam | Timeline visualization sidebar indicator |
| Text Generate Effect | AI analysis text reveal |
| Encrypted Text | Loading state for sensitive data |
| Grid and Dot Backgrounds | Section dividers / tab panel backgrounds |

**Magic UI** — `magicui.design`
150+ free components. Best-in-class for specific effects:

| Component | Use in NEXUS GOV |
|-----------|-----------------|
| Smooth Cursor | Custom cursor for the entire dashboard |
| Number Flow | Already covered above |
| Animated Shiny Text | Label badges / status chips |
| Shimmer Button | Primary CTA buttons (Start Scan, etc.) |
| Marquee | Scrolling news/alert ticker |
| Word Rotate | Alternating label text |
| Retro Grid | Background texture for empty states |
| Blur Fade | Section entrance animations |
| Animated List | Live event feed (SSE alerts) |
| Bento Grid | Stats overview layout |

**Motion Primitives** — `motion-primitives.com`
Focused collection of high-quality Motion-powered primitives. Best for:
- `AnimatedGroup` — staggered list entrances
- `TextEffect` — character-by-character text reveals
- `Transition` — smooth component mounting/unmounting
- `InView` — scroll-triggered reveal wrapper

**Animate UI** — `animate-ui.com`
Full animated drop-in replacements for shadcn components (Dialog, Tabs, Accordion, etc.). Key value: animated versions of the exact shadcn components NEXUS GOV already uses.

### Tier 2: Install via npm

These are actual packages with APIs, not copy-paste code:

| Package | Purpose | Bundle Impact |
|---------|---------|--------------|
| `lenis` | Smooth scroll | ~9kB |
| `@number-flow/react` | Animated numbers | ~6.8kB |
| `react-parallax-tilt` | 3D tilt for cards | ~8kB |
| `motion` | Core animation engine | ~34kB (with LazyMotion optimization: ~19kB) |

---

## 4. Specific Dashboard Techniques

### 4.1 Animated Stat Counters (Metric Cards)

Current state: Static numbers in MetricCard component.
Target: Numbers count up on first viewport entry, re-animate on data refresh.

**Pattern:**
```tsx
// Trigger on mount + data change
import NumberFlow from '@number-flow/react'
import { useInView } from 'motion/react'

function MetricCard({ value, label }) {
  const ref = useRef(null)
  const inView = useInView(ref, { once: true })
  return (
    <div ref={ref}>
      <NumberFlow value={inView ? value : 0} />
    </div>
  )
}
```

### 4.2 Card 3D Tilt + Glow Effect

The "cursor spotlight" effect where a radial glow follows the mouse position on hover is the single highest-impact visual upgrade for a card-heavy dashboard. Implementation:

```tsx
// Using Aceternity UI CardSpotlight (copy-paste)
// Or roll it with Motion:
function GlowCard({ children }) {
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const handleMouseMove = (e) => {
    const rect = e.currentTarget.getBoundingClientRect()
    setPosition({ x: e.clientX - rect.left, y: e.clientY - rect.top })
  }
  return (
    <div onMouseMove={handleMouseMove} className="relative overflow-hidden">
      <div
        className="pointer-events-none absolute opacity-0 group-hover:opacity-100 transition-opacity"
        style={{
          background: `radial-gradient(200px at ${position.x}px ${position.y}px, rgba(59,130,246,0.15), transparent)`,
        }}
      />
      {children}
    </div>
  )
}
```

For 3D tilt: `react-parallax-tilt` is the zero-friction solution. 8kB, handles perspective math, supports glare overlay:
```tsx
import Tilt from 'react-parallax-tilt'
<Tilt tiltMaxAngleDegree={4} glareEnable glareMaxOpacity={0.08}>
  <PoliticianCard />
</Tilt>
```

### 4.3 Tab Indicator Sliding Pill

Current shadcn Tabs uses CSS transitions. For Awwwards quality, the indicator needs Motion's `layoutId` for spring-physics spring:

```tsx
// Animated tab indicator using layoutId
{tabs.map(tab => (
  <button key={tab.id} onClick={() => setActive(tab.id)}>
    {active === tab.id && (
      <motion.div
        layoutId="tab-indicator"
        className="absolute inset-0 bg-primary/10 rounded-md"
        transition={{ type: 'spring', stiffness: 400, damping: 30 }}
      />
    )}
    {tab.label}
  </button>
))}
```

This is also the approach used by Vercel's navigation and Linear's sidebar.

### 4.4 Skeleton Loading with Shimmer

Current state: `<LoadingSpinner>` (rotating icon).
Target: Content-aware skeleton matching actual layout.

Tailwind 4 custom shimmer:
```css
@keyframes shimmer {
  from { background-position: -200% center; }
  to   { background-position: 200% center; }
}

.skeleton-shimmer {
  background: linear-gradient(
    90deg,
    var(--bg-card) 25%,
    rgba(255,255,255,0.04) 50%,
    var(--bg-card) 75%
  );
  background-size: 200% auto;
  animation: shimmer 1.8s ease-in-out infinite;
}
```

shadcn's `<Skeleton>` component uses `animate-pulse` by default. Replace with the shimmer above for premium feel.

### 4.5 Glassmorphism for Floating Panels

The TopBar, modals, and tooltips in NEXUS GOV are candidates for glassmorphism. The pattern requires a real background behind the element (not a flat bg-card):

```tsx
className="backdrop-blur-xl bg-white/[0.03] border border-white/[0.08] shadow-2xl"
```

**Critical:** Backdrop blur is GPU-expensive. Use only for elements that are:
1. Positioned over varied content (sidebar over main content — yes)
2. Rarely animating themselves (static/sticky elements)

Do NOT apply backdrop-blur to elements that animate their position — causes GPU layer thrashing.

### 4.6 Noise/Grain Texture Overlay

Single CSS pseudo-element on `body` or a wrapper:

```css
body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9999;
  opacity: 0.035;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
  background-repeat: repeat;
  background-size: 180px;
}
```

Opacity 0.035 is the correct value for dark UIs — visible on close inspection but not distracting.

### 4.7 Aurora / Animated Gradient Background

For the header area or empty-state panels. Aceternity's Aurora Background uses CSS keyframes animating background-position on stacked radial gradients. No WebGL required, pure CSS:

```css
@keyframes aurora {
  from { background-position: 50% 50%, 50% 50%; }
  to   { background-position: 350% 50%, 350% 50%; }
}
.aurora {
  background:
    repeating-linear-gradient(100deg, rgba(59,130,246,0.1) 10%, rgba(168,85,247,0.08) 15%, rgba(6,182,212,0.06) 20%, transparent 25%),
    repeating-linear-gradient(100deg, rgba(15,15,30,0) 0%, rgba(59,130,246,0.04) 2%, transparent 5%);
  background-size: 300%, 200%;
  animation: aurora 60s linear infinite;
  filter: blur(8px);
}
```

**NEXUS GOV application:** Use behind the top stat bar (deputies count, scan status) as a very subtle glow zone.

### 4.8 Staggered List Entrances for Data Tables

When data loads, items should appear staggered, not all at once:

```tsx
import { motion, AnimatePresence } from 'motion/react'

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06 } }
}
const item = {
  hidden: { opacity: 0, y: 12 },
  show:  { opacity: 1, y: 0, transition: { duration: 0.25, ease: 'easeOut' } }
}

<motion.ul variants={container} initial="hidden" animate="show">
  {politicians.map(p => (
    <motion.li key={p.id} variants={item}>
      <PoliticianRow politician={p} />
    </motion.li>
  ))}
</motion.ul>
```

Cap stagger at 15 items maximum (beyond that, users see the last items too late). For long lists, only animate the first 10.

### 4.9 Page/Tab Transition

NEXUS GOV uses React Router v7. The View Transitions API is now Baseline (Chrome 111+, Firefox 133+, Safari 18+). React Router v7 has native `viewTransition` support:

```tsx
// In React Router Link / navigate
<Link viewTransition to="/gov">...</Link>
```

For more control, Motion's `AnimatePresence` with the current route key:
```tsx
<AnimatePresence mode="wait">
  <motion.div
    key={location.pathname}
    initial={{ opacity: 0, y: 8 }}
    animate={{ opacity: 1, y: 0 }}
    exit={{ opacity: 0, y: -8 }}
    transition={{ duration: 0.18 }}
  >
    <Routes location={location}>...</Routes>
  </motion.div>
</AnimatePresence>
```

### 4.10 Custom Cursor (Desktop Only)

For the political dashboard aesthetic, a custom cursor adds premium feel without being distracting. Magic UI's `SmoothCursor` component provides a physics-based cursor follower.

**Important:** Always detect touch/mobile and disable:
```tsx
const isTouchDevice = 'ontouchstart' in window
if (!isTouchDevice) { /* render custom cursor */ }
```

### 4.11 3D Elements — Verdict for NEXUS GOV

React Three Fiber (R3F) for background effects: **Low ROI for this project.**

Reasons:
- RTX 5080 handles it fine, but end-users may not
- R3F adds ~250kB to bundle (Three.js + R3F + @react-three/drei)
- The dashboard already has heavy data visualization libs (d3, recharts, reagraph, nivo, g6 — already ~600kB combined)
- CSS Aurora + noise texture achieves 90% of the visual impact at 0kB

**Exception:** A GLSL noise shader as a subtle background could be done with a single `<canvas>` + tiny shader — no R3F needed. Under 20 lines of WebGL code. Medium confidence this is worthwhile.

**If 3D is desired:** R3F's `@react-three/drei` `<Float>` component for a floating 3D element (e.g., the parliament hemicycle converted to 3D) would be the targeted use case, not a background.

---

## 5. shadcn/ui Customization for Premium Look

### CSS Variable Architecture (NEXUS GOV Already Has Good Bones)

The current `index.css` is correctly structured. The upgrade path is:

**1. Richer border gradients**
Replace flat `rgba(255,255,255,0.08)` borders with subtle gradient borders on featured cards:
```css
.card-featured {
  border: 1px solid transparent;
  background:
    linear-gradient(var(--bg-card), var(--bg-card)) padding-box,
    linear-gradient(135deg, rgba(59,130,246,0.3), rgba(168,85,247,0.1)) border-box;
}
```

**2. Add glow CSS variables**
```css
:root {
  --glow-blue:   0 0 20px rgba(59,130,246,0.15), 0 0 60px rgba(59,130,246,0.05);
  --glow-purple: 0 0 20px rgba(168,85,247,0.15), 0 0 60px rgba(168,85,247,0.05);
  --glow-green:  0 0 20px rgba(34,197,94,0.15),  0 0 60px rgba(34,197,94,0.05);
}
.card:hover { box-shadow: var(--glow-blue); }
```

**3. Geist variable font — unlock font features**
NEXUS GOV already imports `@fontsource-variable/geist`. Unlock tabular numbers for all numeric displays:
```css
.metric-value {
  font-feature-settings: 'tnum' 1, 'ss01' 1;
  font-variant-numeric: tabular-nums;
}
```

**4. Override shadcn radius for sharper feel**
Current `--radius: 0.625rem`. For a more architectural/precise dashboard aesthetic, reduce:
```css
--radius: 0.375rem; /* 6px — sharper, more data-dense look */
```

**5. Active element depth illusion**
When a tab or sidebar item is active, increase its visual "height" with a subtle top-border:
```css
.tab-active {
  border-top: 1px solid var(--primary);
  box-shadow: 0 -1px 0 var(--primary);
}
```

---

## 6. Performance Architecture

### The Cardinal Rules (Verified via MDN + Google Performance Docs)

**Only animate `transform` and `opacity`** — these are composited on the GPU thread, never trigger layout or paint. Everything else (width, height, top, left, background-color) triggers reflow and kills 60fps.

```
SAFE:   transform: translateX/Y/Z, scale, rotate
        opacity
        filter (blur, brightness) — composited since Chrome 76
UNSAFE: width, height, margin, padding, top, left, border-radius (during animation)
        background-color (use opacity + overlay instead)
        box-shadow (use opacity change on a pseudo-element instead)
```

### Bundle Size Impact (Additive to Current 600kB+ DataViz Libs)

| Addition | Bundle Add | Priority |
|----------|-----------|---------|
| `motion` (full) | +34kB | Required |
| `motion` (LazyMotion domAnimation) | +19kB deferred | Recommended |
| `lenis` | +9kB | Recommended |
| `@number-flow/react` | +7kB | High ROI |
| `react-parallax-tilt` | +8kB | Medium ROI |
| Aceternity components (copy-paste) | 0kB (inline CSS) | High ROI |
| Magic UI SmoothCursor | 0kB (copy-paste) | Medium ROI |
| R3F + Three.js | +250kB | Low ROI for NEXUS GOV |
| tsParticles (slim) | +30kB | Low ROI vs CSS alternative |

**Verdict:** Add motion + lenis + number-flow. Skip R3F, skip tsParticles. Use CSS for particle/noise effects.

### Lazy Loading Animations Below the Fold

Motion's `whileInView` is already scroll-aware. For tab panels that load lazily (already implemented in GovernmentPage.tsx with `React.lazy`), combine with:
```tsx
// Stagger entrance only when tab becomes visible
<motion.div
  initial={{ opacity: 0, y: 16 }}
  animate={{ opacity: 1, y: 0 }}
  transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
>
  <TabContent />
</motion.div>
```

The easing `[0.16, 1, 0.3, 1]` is the Expo Out curve — very fast initial movement, smooth settle. This is what Linear/Vercel use.

### GPU Layer Management

Avoid creating too many GPU layers simultaneously:
- Max 5-7 `will-change: transform` elements at once
- Never apply `will-change` in CSS permanently — add it on hover, remove on leave
- `backdrop-filter` creates GPU layers — use sparingly (2-3 elements max on screen simultaneously)

### Reduced Motion Respect

Always wrap Motion animations in a prefers-reduced-motion check:
```tsx
import { useReducedMotion } from 'motion/react'

function AnimatedCard({ children }) {
  const shouldReduce = useReducedMotion()
  return (
    <motion.div
      whileHover={shouldReduce ? {} : { scale: 1.02, y: -2 }}
    >
      {children}
    </motion.div>
  )
}
```

---

## 7. Priority Upgrade Roadmap for NEXUS GOV

### Phase A — Zero-Dependency Wins (CSS Only, 1-2 days)

1. Add grain/noise overlay to `body::after` in `index.css`
2. Add glow CSS variables for `.card:hover` states
3. Add gradient border on featured cards (politician profiles)
4. Replace `animate-pulse` skeleton with shimmer animation
5. Add `font-feature-settings: 'tnum' 1` to all metric values
6. Reduce border-radius slightly (0.375rem) for more precise aesthetic
7. Add subtle `transition: all 0.15s ease` to all interactive elements globally

### Phase B — Motion Core (motion package, 1 week)

8. Install `motion` package
9. Wrap all tab switching in `AnimatePresence` with sliding pill `layoutId`
10. Add staggered list entrances to politician list, positions list, alert feed
11. Add `whileHover` scale + glow to MetricCard components
12. Add page transition with `AnimatePresence` in App.tsx routes
13. Copy Aceternity `CardSpotlight` for politician profile cards

### Phase C — Data Animation (number-flow + Lenis, 3 days)

14. Install `@number-flow/react` — replace all `<MetricCard>` static numbers
15. Install `lenis` — wrap root in `ReactLenis` for smooth scroll
16. Add Aurora background to the stats header zone
17. Copy Magic UI `AnimatedList` for SSE alert feed (real-time events look spectacular with this)

### Phase D — Premium Polish (react-parallax-tilt + effects, 1 week)

18. Install `react-parallax-tilt` for politician profile cards
19. Copy Magic UI `SmoothCursor` (desktop only)
20. Copy Aceternity `MovingBorder` for active scan status chip
21. Copy Aceternity `TextGenerateEffect` for AI analysis reveal text
22. Animated Tab transitions using React Router viewTransition API

---

## 8. Real Examples and References

### GitHub Repositories

- **awesome-shadcn-ui**: `github.com/birobirobiro/awesome-shadcn-ui` — curated list of all shadcn extensions
- **UI TripleD**: 100+ Framer Motion + shadcn components — `github.com/tripled-io/ui-tripled`
- **shadcn-examples**: `github.com/shadcn-examples/shadcn-examples` — 67 copy-paste examples

### Component Libraries (All Free Tier Available)

| Library | URL | Best For |
|---------|-----|---------|
| Aceternity UI | `ui.aceternity.com` | Background effects, card hover, text effects |
| Magic UI | `magicui.design` | Smooth cursor, animated list, shimmer buttons |
| Motion Primitives | `motion-primitives.com` | Text reveals, stagger groups, in-view triggers |
| Animate UI | `animate-ui.com` | Animated shadcn component replacements |
| Origin UI | `originui.com` | 400+ copy-paste, comprehensive coverage |

### Design Inspiration

- **Awwwards Data Visualization**: `awwwards.com/websites/data-visualization/`
- **Awwwards Dark Mode Collection**: `awwwards.com/awwwards/collections/dark-mode/`
- **Linear App**: Reference for tab animations, sidebar micro-interactions
- **Vercel Dashboard**: Reference for stat cards, gradient accents, typography
- **shadcn.io Themes**: `shadcnblocks.com/themes` — premium theme variations

---

## 9. Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| Framer Motion / Motion API | HIGH | Official docs + ecosystem verified |
| Aceternity/Magic UI components | HIGH | Verified live sites + GitHub |
| CSS techniques (glassmorphism, noise, aurora) | HIGH | MDN + multiple implementation articles |
| Performance rules (transform/opacity) | HIGH | MDN + Google Web Fundamentals |
| number-flow package | HIGH | Official npm + shadcn.io ecosystem |
| Lenis smooth scroll | HIGH | Official package + darkroom.engineering |
| View Transitions API browser support | HIGH | MDN Baseline status + Chrome blog |
| GSAP not recommended | MEDIUM | Trade-off analysis; GSAP has its champions |
| R3F low ROI for this project | MEDIUM | Bundle size analysis; actual impact subjective |

---

## Sources

- [Aceternity UI Components](https://ui.aceternity.com/components)
- [Magic UI](https://magicui.design/)
- [Motion (Framer Motion) — Reduce Bundle Size](https://motion.dev/docs/react-reduce-bundle-size)
- [Motion — LazyMotion](https://motion.dev/docs/react-lazy-motion)
- [Lenis GitHub — darkroomengineering](https://github.com/darkroomengineering/lenis)
- [NumberFlow for React](https://number-flow.barvian.me/)
- [Motion Primitives — shadcn.io registry](https://www.shadcn.io/template/ibelick-motion-primitives)
- [Animate UI](https://animate-ui.com/)
- [Awesome shadcn/ui — birobirobiro](https://github.com/birobirobiro/awesome-shadcn-ui)
- [React View Transitions — React Labs Blog](https://react.dev/blog/2025/04/23/react-labs-view-transitions-activity-and-more)
- [View Transitions API — MDN](https://developer.mozilla.org/en-US/docs/Web/API/View_Transition_API)
- [CSS GPU Animation — Smashing Magazine](https://www.smashingmagazine.com/2016/12/gpu-animation-doing-it-right/)
- [GSAP ScrollTrigger Docs](https://gsap.com/docs/v3/Plugins/ScrollTrigger/)
- [Framer Motion vs React Spring 2025 — Hooked On UI](https://hookedonui.com/animating-react-uis-in-2025-framer-motion-12-vs-react-spring-10/)
- [Best shadcn UI Libraries 2025 — DevKit.best](https://www.devkit.best/blog/mdx/shadcn-ui-libraries-comparison-2025)
- [Awwwards Data Visualization](https://www.awwwards.com/websites/data-visualization/)
- [Geist Font OpenType Features — Lexington Themes](https://lexingtonthemes.com/blog/geist-opentype-features)
- [Fluid Typography — fluid.tw](https://fluid.tw/)
- [Motion+ Cursor — Magnetic Features](https://motion.dev/blog/introducing-magnetic-cursors-in-motion-cursor)
- [Dashboard Design Patterns 2026 — Art of Styleframe](https://artofstyleframe.com/blog/dashboard-design-patterns-web-apps/)
