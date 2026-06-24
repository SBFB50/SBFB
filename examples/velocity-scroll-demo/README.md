# velocity-scroll-demo — app SBFB de démonstration

Petite app **auto-portante** qui reproduit l'effet **velocity-scroll** (la pile
s'incline selon la vitesse de défilement) en **JavaScript vanilla**, pour montrer
qu'on peut faire de belles animations dans une app SBFB **sans framework, sans CDN,
sans réseau** — et que ça tourne tel quel sous la CSP du bac à sable.

## Contenu (2 fichiers)

```
velocity-scroll-demo/
├── index.html   # tout-en-un : HTML + CSS inline + JS inline (vanilla)
└── SBFB.json    # manifeste v2 (bridge.methods = [] : aucune capacité demandée)
```

## La technique (en clair)

- Boucle `requestAnimationFrame` (esprit WAAPI), **0 dépendance**.
- Vélocité = `(scrollY - scrollYPrécédent) / Δtemps` (px/ms, signée).
- Lissage par un ressort minimal (`smooth += (v - smooth) * 0.12`).
- Le JS écrit une variable CSS `--vel` sur `:root` ; le CSS applique
  `transform: skewY(var(--vel))`. Séparation propre, **accéléré GPU**.
- Un bandeau (marquee) dont la vitesse **« penche »** avec la vélocité.
- **`prefers-reduced-motion` respecté** (exigence du design system SBFB Reflect).

## Pourquoi ça passe la CSP SBFB

L'app est servie en iframe sandbox sous :

```
default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none';
base-uri 'none'; form-action 'none'; sandbox allow-scripts
```

- `sandbox allow-scripts` + `'unsafe-inline' 'unsafe-eval'` → le **CSS et le JS inline tournent**.
- `connect-src 'none'` → l'animation **n'a pas besoin du réseau**, donc aucun blocage.
- **0 ressource externe** : pas de `<link>`/`<script>` distant, pas de Google Fonts
  (polices **système**), pas d'image externe (dégradés CSS). Une vraie app embarquerait
  ses polices en **woff2 dans l'archive** (jamais via CDN).

## Comment la voir tourner

À publier/prévisualiser puis ouvrir **via le shell Browse → iframe sandbox**
(jamais ouvrir l'URL blob-serve directement dans un onglet).

```
sbfb-factory preview      # zippe + POST /api/v1/preview/load → URL blob-serve
```

## Ce que ça prouve

Partir sur du **HTML/CSS/JS vanilla** pour les apps SBFB **n'est pas une limite**
côté animation : le natif (CSS moderne + WAAPI) couvre l'essentiel, et pour aller
plus loin on **vendorise** une petite lib (Motion One ~2–18 Ko, anime.js…) **dans
l'archive**. Le bac à sable scellé n'empêche **rien** d'animé — seulement d'aller
chercher des choses dehors.
