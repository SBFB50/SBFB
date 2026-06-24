# daisyui-animejs-showcase — app SBFB scellée

Vitrine **design system + animations avancées** qui prouve qu'on peut livrer une
app SBFB riche (composants **daisyUI v5**, mouvement **anime.js v4** + **CSS
moderne**) **sans CDN, sans réseau, sans Web Worker** — tout est compilé et
vendorisé dans l'archive, et ça tourne tel quel sous la CSP du bac à sable
`blob-serve`.

> Pivot par rapport à l'aperçu initial : l'`index.html` de départ chargeait
> daisyUI + Tailwind + anime.js depuis jsdelivr (aperçu de dev **non
> déployable**). Cette version compile Tailwind+daisyUI en CSS statique et
> vendorise anime.js — **0 ressource externe**.

## Contenu

### Runtime (ce qui est servi à l'iframe)
```
index.html            # 0 CDN, chemins relatifs uniquement
app.css               # Tailwind v4 + daisyUI v5 compilés (thèmes inclus)
app.js                # animations & interactions (script classique)
vendor/anime.umd.js   # anime.js v4 vendorisé (UMD → global `anime`)
SBFB.json             # manifeste v2 (bridge.methods = [] : aucune capacité)
```

### Build-time (jamais publié — voir `.gitignore`)
```
src/input.css         # source Tailwind : @import + @plugin daisyui + thèmes
scripts/vendor-anime.mjs   # copie le bundle anime hors de node_modules
scripts/check-csp.mjs      # gate de conformité CSP
package.json          # outils de build (Node éphémère)
```

## Build

Node éphémère, **runtime sans dépendance**. Licences : daisyUI **MIT**,
anime.js v4 **MIT**, Tailwind **MIT** (souveraineté OK).

```bash
npm install        # outils build-time
npm run build      # = vendor:anime + build:css
npm run check:csp  # gate de conformité (échoue si une ressource réseau apparaît)
```

- `build:css` → `tailwindcss -i src/input.css -o app.css --minify`. La source
  utilise `@import "tailwindcss" source(none)` + `@source "../index.html"` +
  `@source "../app.js"` : seules les classes réellement utilisées sont émises
  (≈ 87 Ko), thèmes daisyUI inclus.
- `vendor:anime` → copie `node_modules/animejs/dist/bundles/anime.umd.min.js`
  dans `vendor/anime.umd.js`.

## Pourquoi ça passe la CSP du bac à sable

CSP réelle injectée par `blob-serve`
(`crates/nexus-shell-daemon-core/src/blob_serve.rs`) :

```
default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none';
base-uri 'none'; form-action 'none'; frame-ancestors *; sandbox allow-scripts
+ COOP same-origin, COEP require-corp (origine opaque, sans allow-same-origin)
```

- **0 CDN / 0 fetch** : tous les assets sont dans l'archive, en **chemins
  relatifs** (`app.css`, `app.js`, `vendor/anime.umd.js`). `connect-src 'none'`
  n'est jamais sollicité (aucun `fetch`/XHR/WebSocket/EventSource/sendBeacon).
- **Scripts classiques, pas ESM** : `<script src>` se charge en mode *no-cors*
  sous `default-src 'self'`, même en **origine opaque**. Un
  `<script type="module">` serait fetché en mode CORS, qu'un document à origine
  opaque ne peut pas satisfaire pour ses propres assets — d'où le bundle **UMD**
  exposant le global `anime`. C'est exactement le schéma des apps SBFB déjà
  livrées (`sbfb-ideas`, `sbfb-explorer` : `<link>` + `<script src>`).
- **0 Web Worker** (`worker-src 'none'`), **0 iframe imbriquée**
  (`frame-src 'none'`), **aucun `<form>`** (`form-action 'none'` + sandbox sans
  `allow-forms` : les interactions passent par des `<button>` + handlers).
- **Pas de `<base href>`** (`base-uri 'none'` le neutralise).
- **Polices système** : aucune police distante (pas de Google Fonts). Choix
  assumé — souveraineté + 0 octet réseau + rendu instantané. Pour une typo de
  marque, on embarquerait des **woff2** dans l'archive (jamais via CDN).

### Note d'honnêteté sur `grep https app.css`

`app.css` contient **deux** occurrences `http(s)` qui **ne sont pas des
ressources réseau** et que le gate `check:csp` met explicitement en allowlist :

1. `http://www.w3.org/2000/svg` — le **namespace XML** des icônes SVG que
   daisyUI inline en `data:` URI (cases à cocher, radios, flèches de select).
   Un `xmlns` n'est jamais fetché par le navigateur.
2. `https://tailwindcss.com` — la **bannière de licence MIT** de Tailwind,
   conservée par obligation d'attribution.

Le gate vérifie qu'**aucun** `url(http…)`, `@import`, ni primitive réseau
n'existe, et que toute URL absolue restante est dans cette allowlist.
`index.html` et `app.js` sont, eux, **strictement zéro `http(s)`**.

## Prévisualiser (jamais en direct sur blob-serve)

À ouvrir **via le shell Browse → iframe sandbox**, jamais en ouvrant l'URL
`blob-serve` dans un onglet.

```bash
sbfb-factory preview     # zippe le dossier + charge l'aperçu, puis Browse → iframe
```

## Design system « Reflect »

Tokens **copiés dans l'app** (jamais référencés : `connect-src 'none'`), exposés
comme thème daisyUI `sbfb-reflect` (défaut, dark) + `sbfb-light` :

- Palette restreinte monochrome : canvas `#111111`, surfaces `#191919`→`#2E2E2E`,
  texte `#EBEBEB`/`#999`/`#666`, **un seul accent quasi-blanc** `#EBEBEB`
  (≤ 3×/écran). Sémantique : ok `#4ADE80`, warn `#FACC15`, danger `#F87171`.
- Grille **8 px**, rayons **2→16**, mouvement **80/120/200 ms sans overshoot**
  (le ressort `linear()` est réservé aux moments « démo » : modale, toast,
  bouton spring).
- Le sélecteur de thème embarque aussi des thèmes daisyUI intégrés (night,
  dracula, synthwave, cyberpunk…) pour montrer le système de thèmes.
- Une **section « Design system »** dédiée : palette, échelle typo, espacement,
  rayons, élévation, tokens de mouvement, + galerie de composants re-skinnés.

## Animations

**CSS natif (0 JS) :**
- Reveals pilotés par le scroll : `animation-timeline: view()` + `animation-range`.
- Parallaxe du décor : `animation-timeline: scroll(root)`.
- Entrée/sortie discrètes : `@starting-style` + `transition-behavior: allow-discrete`
  (panneau modal + toasts ; la propriété `display` est transitionnée).
- Ressort en CSS pur : easing `linear(...)` (bouton spring, modale, toast).
- Transitions entre états : **View Transitions API**
  (`document.startViewTransition`) au changement de thème et au filtrage de la
  galerie (FLIP via `view-transition-name`).

**anime.js v4 :**
- Titre hero lettre par lettre (`stagger`).
- Compteurs (count-up) déclenchés au scroll (`IntersectionObserver`).
- Vague en grille (`createTimeline` en boucle, `stagger({grid, from:'center'})`)
  + burst aléatoire + onde centrale.
- Boutons magnétiques (suivi du pointeur, retour en `createSpring`).

**Partout :** `prefers-reduced-motion` honoré (garde-fou CSS global + court-circuit
JS), et animations limitées à `transform`/`opacity` (accélérées GPU, 0 reflow).

## Souveraineté

Uniquement des dépendances **OSI** : daisyUI (MIT), anime.js v4 (MIT),
Tailwind (MIT). **Pas de GSAP** (gratuit mais non open-source). **Pas de Web
Worker**. **Pas de pi-ai/Electron/Puppeteer.** Tout est scellé dans l'archive.
