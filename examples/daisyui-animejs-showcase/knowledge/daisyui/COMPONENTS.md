# Composants daisyUI v5.5.23 — catalogue annoté CSP

> 68 entrées, ancrées dans `node_modules/daisyui/components/*.css` (source) + le `llms.txt` officiel. Colonne **CSP** = utilisable dans l'iframe scellée SBFB.

## Index

| # | composant | classe | CSP | modifiers |
|---|---|---|:--:|---|
| 1 | alert | `alert` | ✅ | alert-info alert-success alert-warning alert-error alert-outline alert-dash… |
| 2 | avatar | `avatar` | ✅ | avatar-group avatar-online avatar-offline avatar-placeholder |
| 3 | badge | `badge` | ✅ | badge-outline badge-dash badge-soft badge-ghost badge-neutral badge-primary… |
| 4 | breadcrumbs | `breadcrumbs` | ✅ | sm:breadcrumbs md:breadcrumbs lg:breadcrumbs xl:breadcrumbs |
| 5 | button | `btn` | ✅ | btn-neutral btn-primary btn-secondary btn-accent btn-info btn-success… |
| 6 | calendar | `cally` | ✅ |  |
| 7 | card | `card` | ✅ | card-border card-dash card-side image-full card-xs card-sm… |
| 8 | carousel | `carousel` | ✅ | carousel-vertical carousel-horizontal carousel-start carousel-center carousel-end |
| 9 | chat | `chat` | ✅ | chat-start chat-end chat-bubble-primary chat-bubble-secondary chat-bubble-accent chat-bubble-neutral… |
| 10 | checkbox | `checkbox` | ✅ | checkbox-primary checkbox-secondary checkbox-accent checkbox-neutral checkbox-info checkbox-success… |
| 11 | collapse | `collapse` | ✅ | collapse-arrow collapse-plus collapse-open collapse-close sm:collapse md:collapse… |
| 12 | countdown | `countdown` | ✅ |  |
| 13 | diff | `diff` | ✅ | sm:diff md:diff lg:diff xl:diff 2xl:diff |
| 14 | divider | `divider` | ✅ | divider-neutral divider-primary divider-secondary divider-accent divider-success divider-warning… |
| 15 | dock | `dock` | ✅ | dock-active dock-xs dock-sm dock-md dock-lg dock-xl… |
| 16 | drawer | `drawer` | ✅ | drawer-end drawer-open sm:drawer md:drawer lg:drawer xl:drawer… |
| 17 | dropdown | `dropdown` | ✅ | dropdown-start dropdown-center dropdown-end dropdown-top dropdown-bottom dropdown-left… |
| 18 | fab | `fab` | ✅ | fab-flower sm:fab md:fab lg:fab xl:fab 2xl:fab |
| 19 | fieldset | `fieldset` | ✅ | sm:fieldset md:fieldset lg:fieldset xl:fieldset 2xl:fieldset |
| 20 | fileinput | `file-input` | ✅ | file-input-ghost file-input-neutral file-input-primary file-input-secondary file-input-accent file-input-info… |
| 21 | filter | `filter` | ✅ |  |
| 22 | footer | `footer` | ✅ | footer-center footer-horizontal footer-vertical sm:footer sm:footer-horizontal md:footer-horizontal… |
| 23 | hero | `hero` | ✅ | sm:hero md:hero lg:hero xl:hero 2xl:hero |
| 24 | hover3d | `hover-3d` | ✅ |  |
| 25 | hovergallery | `hover-gallery` | ✅ | sm:hover-gallery md:hover-gallery lg:hover-gallery xl:hover-gallery 2xl:hover-gallery |
| 26 | indicator | `indicator` | ✅ | indicator-start indicator-center indicator-end indicator-top indicator-middle indicator-bottom… |
| 27 | input | `input` | ✅ | input-ghost input-neutral input-primary input-secondary input-accent input-info… |
| 28 | kbd | `kbd` | ✅ | kbd-xs kbd-sm kbd-md kbd-lg kbd-xl sm:kbd… |
| 29 | label | `label` | ✅ |  |
| 30 | link | `link` | ✅ | link-hover link-neutral link-primary link-secondary link-accent link-success… |
| 31 | list | `list` | ✅ | list-col-wrap list-col-grow sm:list md:list lg:list xl:list… |
| 32 | loading | `loading` | ✅ | loading-spinner loading-dots loading-ring loading-ball loading-bars loading-infinity… |
| 33 | mask | `mask` | ✅ | mask-squircle mask-heart mask-hexagon mask-hexagon-2 mask-decagon mask-pentagon… |
| 34 | menu | `menu` | ✅ | menu-disabled menu-active menu-focus menu-dropdown-show menu-xs menu-sm… |
| 35 | mockup-browser | `mockup-browser` | ✅ |  |
| 36 | mockup-code | `mockup-code` | ✅ |  |
| 37 | mockup-phone | `mockup-phone` | ✅ |  |
| 38 | mockup-window | `mockup-window` | ✅ |  |
| 39 | modal | `modal` | ✅ | modal-open modal-top modal-middle modal-bottom modal-start modal-end |
| 40 | navbar | `navbar` | ✅ | sm:navbar md:navbar lg:navbar xl:navbar 2xl:navbar |
| 41 | progress | `progress` | ✅ | progress-neutral progress-primary progress-secondary progress-accent progress-info progress-success… |
| 42 | radial-progress | `radial-progress` | ✅ |  |
| 43 | radio | `radio` | ✅ | radio-neutral radio-primary radio-secondary radio-accent radio-success radio-warning… |
| 44 | range | `range` | ✅ | range-neutral range-primary range-secondary range-accent range-success range-warning… |
| 45 | rating | `rating` | ✅ | rating-half rating-xs rating-sm rating-md rating-lg rating-xl |
| 46 | select | `select` | ✅ | select-ghost select-neutral select-primary select-secondary select-accent select-info… |
| 47 | skeleton | `skeleton` | ✅ | skeleton-text |
| 48 | stack | `stack` | ✅ | stack-top stack-bottom stack-start stack-end |
| 49 | stat | `stats` | ✅ | stats-horizontal stats-vertical |
| 50 | status | `status` | ✅ | status-primary status-secondary status-accent status-neutral status-info status-success… |
| 51 | steps | `steps` | ✅ | step-neutral step-primary step-secondary step-accent step-info step-success… |
| 52 | swap | `swap` | ✅ | swap-active swap-rotate swap-flip |
| 53 | tab | `tabs` | ✅ | tabs-box tabs-border tabs-lift tab-active tab-disabled tabs-top… |
| 54 | table | `table` | ✅ | table-zebra table-pin-rows table-pin-cols table-xs table-sm table-md… |
| 55 | textarea | `textarea` | ✅ | textarea-ghost textarea-neutral textarea-primary textarea-secondary textarea-accent textarea-info… |
| 56 | textrotate | `text-rotate` | ✅ | duration-{value} (utilitaire Tailwind, ms, ex duration-12000) classes de texte Tailwind libres (text-7xl, font-title, leading-[2], couleurs bg-*/text-* sur chaque ligne) |
| 57 | timeline | `timeline` | ✅ | timeline-vertical timeline-horizontal timeline-compact timeline-snap-icon timeline-box sm:timeline… |
| 58 | toast | `toast` | ✅ | toast-start toast-center toast-end toast-top toast-middle toast-bottom… |
| 59 | toggle | `toggle` | ✅ | toggle-primary toggle-secondary toggle-accent toggle-neutral toggle-success toggle-warning… |
| 60 | tooltip | `tooltip` | ✅ | tooltip-open tooltip-top tooltip-bottom tooltip-left tooltip-right tooltip-primary… |
| 61 | validator | `validator` | ✅ |  |
| 62 | glass | `glass` | ✅ | --glass-blur --glass-opacity --glass-reflect-degree --glass-reflect-opacity --glass-border-opacity --glass-text-shadow-opacity |
| 63 | join | `join` | ✅ | join-vertical join-horizontal sm:join md:join lg:join xl:join… |
| 64 | radius (rounded-*) | `rounded-box` | ✅ | rounded-box rounded-field rounded-selector rounded-t-box rounded-b-box rounded-l-box… |
| 65 | prose (typography) | `prose` | ✅ | --tw-prose-body --tw-prose-headings --tw-prose-links --tw-prose-code --tw-prose-pre-bg --tw-prose-pre-code… |
| 66 | mask | `mask` | ✅ | mask-squircle mask-heart mask-hexagon mask-hexagon-2 mask-decagon mask-pentagon… |
| 67 | svg noise/icons (base — svg.css) | `--fx-noise` | ✅ | --fx-noise |
| 68 | root color + reset (base) | `:root` | ✅ | --root-bg --page-scroll-bg --color-base-100 --color-base-content --default-font-family |

## `alert` — alert

Boite de notification en grid (grid-auto-flow:column) qui aligne icone + texte + actions; passe a 2 colonnes des qu'il y a un 2e enfant (:has(:nth-child(2))). Couleurs via variables --alert-color/--alert-border-color, variantes soft/outline/dash, orientation vertical/horizontal.

- **Modifiers** : `alert-info`, `alert-success`, `alert-warning`, `alert-error`, `alert-outline`, `alert-dash`, `alert-soft`, `alert-vertical`, `alert-horizontal`, `sm:alert-horizontal`
- **Mécanismes CSS** : display:grid · grid-auto-flow:column · :has(:nth-child(2)) layout switch · color-mix(in oklab,...) · box-shadow depth (oklch + var(--depth)) · background-image:var(--fx-noise) (texture noise, data-URI/var generee build-time) · border-radius:var(--radius-box) · CSS custom properties --alert-color/--alert-border-color
- **CSP SBFB** : ✅ usable — CSS pur, aucune url() distante: le seul background-image est var(--fx-noise) (genere par le theme daisyUI au build, inline data: ou none). color-mix/oklch/grid/:has sont du CSS natif compose dans l'iframe. role=alert est statique. Aucune dependance JS, fetch, font distante ou form.
  - ⚠️ var(--fx-noise) doit etre fourni par le theme daisyUI vendore (deja inline au build); si une app pointait noise vers une URL http externe, COEP require-corp + connect-src 'none' la bloquerait, mais le defaut daisyUI est inline donc OK
  - ⚠️ fermeture/dismiss de l'alerte = comportement a coder localement (bouton + handler JS), le CSS ne gere pas la disparition

```html
<div role="alert" class="alert {MODIFIER}">{CONTENT}</div>
```

---

## `avatar` — avatar

Conteneur inline-flex pour une vignette: l'enfant <div> impose aspect-ratio:1 + overflow:hidden, l'<img> est object-fit:cover 100%. avatar-group empile en flex avec bordure base-100. avatar-online/offline ajoutent une pastille via ::before (background-color success/base-300). avatar-placeholder centre un contenu textuel (initiales).

- **Modifiers** : `avatar-group`, `avatar-online`, `avatar-offline`, `avatar-placeholder`
- **Mécanismes CSS** : display:inline-flex · aspect-ratio:1 · object-fit:cover · ::before pastille status (position:absolute, outline) · border-radius:3.40282e38px (cercle) · background-color:var(--color-success/base-300) · border:4px solid var(--color-base-100) pour avatar-group
- **CSP SBFB** : ✅ usable — Structure et styles 100% CSS, aucune url()/mask/backdrop-filter. La seule reserve est l'<img src> dans l'exemple: une URL http distante serait bloquee par connect-src 'none' + COEP. En iframe scellee, utiliser des images same-origin (data:, blob:, ou fichiers packages dans l'archive) au lieu d'URLs distantes.
  - ⚠️ <img src> vers URL distante http(s) interdit par connect-src 'none' + COEP require-corp: servir l'image en data:/blob: ou depuis l'archive same-origin
  - ⚠️ les mask classes suggerees par la doc (mask-squircle/mask-hexagon) reposent sur des mask-image data-URI Tailwind/daisyUI: OK si compiles build-time inline, a verifier (pas dans avatar.css lui-meme)

```html
<div class="avatar {MODIFIER}">
  <div>
    <img src="{image-url}" />
  </div>
</div>
```

---

## `badge` — badge

Pastille inline-flex de hauteur var(--size) pilotee par --size-selector, fond/bordure via --badge-color/--badge-bg/--badge-fg. Variantes outline/dash (fond transparent, bordure currentColor), soft (color-mix 8%/10%), ghost (base-200), 8 couleurs semantiques, 5 tailles.

- **Modifiers** : `badge-outline`, `badge-dash`, `badge-soft`, `badge-ghost`, `badge-neutral`, `badge-primary`, `badge-secondary`, `badge-accent`, `badge-info`, `badge-success`, `badge-warning`, `badge-error`, `badge-xs`, `badge-sm`, `badge-md`, `badge-lg`, `badge-xl`
- **Mécanismes CSS** : display:inline-flex · color-mix(in oklab,...) pour soft · border-radius:var(--radius-selector) · CSS vars --badge-color/--badge-bg/--badge-fg/--size · background-image:none,var(--fx-noise) · border:var(--border) solid · height:var(--size) calc(var(--size-selector)*N)
- **CSP SBFB** : ✅ usable — Pur CSS (color-mix, custom props, fx-noise inline). Aucune url() distante, mask, backdrop-filter, JS, fetch ou form. Un <span> texte simple, parfaitement scellable.
  - ⚠️ var(--fx-noise) doit etre la valeur inline du theme vendore (defaut daisyUI = OK)

```html
<span class="badge {MODIFIER}">Badge</span>
```

---

## `breadcrumbs` — breadcrumbs

Fil d'ariane: wrappe un <ul>/<ol>/<menu> en flex nowrap avec overflow-x:auto (scroll si depasse). Chaque <li> apres le premier recoit un separateur chevron via &+:before (bordure top+right 1px tournee 45deg). Liens cliquables avec underline au hover (media hover:hover) et focus-visible outline.

- **Modifiers** : `sm:breadcrumbs`, `md:breadcrumbs`, `lg:breadcrumbs`, `xl:breadcrumbs`
- **Mécanismes CSS** : display:flex · overflow-x:auto · &+:before separateur chevron (border-top+border-right 1px, rotate:45deg) · [dir=rtl] rotate:-135deg · @media (hover:hover) text-decoration underline · :focus-visible outline · white-space:nowrap
- **CSP SBFB** : ✅ usable — Le separateur est dessine en pur CSS (bordures + rotate), pas d'url()/mask/SVG distant. Pas de JS ni form. Les <a> sont des liens de navigation: dans une iframe scellee a origine opaque, eviter href externes (navigation interceptee/inutile) et utiliser des ancres internes ou button+handler.
  - ⚠️ <a href> vers une page distante n'a pas de sens en iframe scellee (origine opaque, frame-src 'none' pour sous-iframes); preferer ancres internes # ou handlers JS locaux pour la navigation in-app

```html
<div class="breadcrumbs">
  <ul><li><a>Link</a></li></ul>
</div>
```

---

## `btn` — button

Bouton inline-flex tout-en-CSS-vars (--btn-bg/--btn-fg/--btn-color/--btn-border/--btn-shadow/--btn-p/--size). Hover/active assombrissent via color-mix + translate. Supporte input checkbox/radio (appearance:none, aria-label rendu via :after content attr), etat checked->primary, join (radius via --join-*). Variantes ghost/link/outline/dash/soft + 8 couleurs + 5 tailles + formes square/circle/wide/block.

- **Modifiers** : `btn-neutral`, `btn-primary`, `btn-secondary`, `btn-accent`, `btn-info`, `btn-success`, `btn-warning`, `btn-error`, `btn-outline`, `btn-dash`, `btn-soft`, `btn-ghost`, `btn-link`, `btn-active`, `btn-disabled`, `btn-xs`, `btn-sm`, `btn-md`, `btn-lg`, `btn-xl`, `btn-wide`, `btn-block`, `btn-square`, `btn-circle`
- **Mécanismes CSS** : display:inline-flex · color-mix(in oklab,...) hover/active · box-shadow depth (oklch+var(--depth)) · text-shadow · translate:0 .5px sur :active · transition color/bg/border/shadow · :focus-visible / :has(:focus-visible) outline+isolation · input[type=checkbox/radio] appearance:none + :after content:attr(aria-label) · background-image:none,var(--btn-noise) · border-radius via --join-* / --radius-field
- **CSP SBFB** : ✅ usable — 100% CSS pur, pas d'url() distante (btn-noise = fx-noise inline), pas de mask/backdrop-filter/fetch/font. L'action du bouton se code en JS local (onclick). Idiome SBFB recommande: div/button + handler, jamais <form> submit. btn-link rend juste un soulignement; eviter <a href> externe.
  - ⚠️ si utilise comme <a class="btn" href>, meme reserve que breadcrumbs: navigation externe inutile en iframe scellee
  - ⚠️ ne pas mettre btn dans un <form> qui submit (form-action 'none' + sandbox bloquent): utiliser button type=button + handler

```html
<button class="btn {MODIFIER}">Button</button>
```

---

## `cally` — calendar

Skins CSS pour 3 librairies calendrier tierces: Cally (web component, style via ::part()), React Day Picker (classes rdp-*) et Pikaday (classes pika-*). Stylise jours, navigation, selection, plages (range), today via les variables de theme (--color-primary, --color-base-content, --radius-field). Le calendrier lui-meme (logique de dates, navigation) vient de la lib JS, daisyUI ne fournit que l'habillage.

- **Sous-parties** : `react-day-picker`, `pika-single`, `rdp-day`, `rdp-day_button`, `rdp-nav`, `rdp-chevron`, `rdp-month_grid`, `rdp-selected`, `rdp-range_start`, `rdp-range_middle`, `rdp-range_end`, `pika-lendar`, `pika-title`, `pika-button`, `pika-prev`, `pika-next`
- **Mécanismes CSS** : ::part() styling (Cally web component, Shadow DOM) · calendar-month custom element · border-collapse table grid (rdp/pika) · fill:var(--color-base-content) sur .rdp-chevron (SVG peint via CSS fill, pas un utilitaire Tailwind fill-*) · rotate/transform RTL · :hover/:disabled states · border-radius:var(--radius-field) · --tw-content pour chevrons pika (content:'‹'/'›')
- **CSP SBFB** : ✅ usable — L'habillage CSS est utilisable (couleurs/parts/tables). MAIS le composant est intrinsequement pilote par une lib JS tierce (Cally <calendar-date> custom element, react-day-picker React, Pikaday): structure CSS OK, comportement (rendu des jours, navigation, selection) a coder/embarquer localement sans reseau. Cally requiert le script web-component du package vendore same-origin (pas de CDN). Le .rdp-chevron utilise fill:var(--color-base-content) directement en CSS (pas l'utilitaire Tailwind fill-* non compile), donc le SVG est peint correctement.
  - ⚠️ depend d'une lib JS tierce qui doit etre vendore dans l'archive (Cally web component / react-day-picker / Pikaday) — jamais charger via CDN (connect-src 'none')
  - ⚠️ Cally repose sur Shadow DOM + ::part(): allow-scripts suffit, mais le script du custom-element doit etre inline ou same-origin
  - ⚠️ fill:var(--color-base-content) est en CSS source donc OK; ne PAS compter sur des utilitaires Tailwind fill-*/stroke-* qui ne compilent pas dans l'iframe
  - ⚠️ les chevrons SVG internes (rdp-chevron) viennent de la lib JS: s'assurer qu'ils sont des SVG inline, pas des <img> distants

```html
<calendar-date class="cally">{CONTENT}</calendar-date>
```

---

## `card` — card

Conteneur flex-column avec radius var(--radius-box). card-body porte le padding (--card-p) et la taille de police; card-title (font-weight 600) et card-actions (flex-wrap). figure:first/last-child herite des radius coins. image-full superpose body+image en grid (img brightness 28%). card-side passe en row. Selectable: :has(>input) -> cursor pointer, :has(>:checked) -> outline 2px.

- **Sous-parties** : `card-body`, `card-title`, `card-actions`
- **Modifiers** : `card-border`, `card-dash`, `card-side`, `image-full`, `card-xs`, `card-sm`, `card-md`, `card-lg`, `card-xl`, `sm:card-side`
- **Mécanismes CSS** : display:flex/grid (image-full) · border-radius:var(--radius-box) + inherit coins figure · :has(>input)/:has(>:checked) outline selection · filter:brightness(28%) sur image-full img · object-fit:cover · transition:outline · CSS vars --card-p/--card-fs/--cardtitle-fs · :focus-visible outline
- **CSP SBFB** : ✅ usable — Pur CSS (flex/grid, :has, filter:brightness, custom props). Aucune url() distante, mask, backdrop-filter, JS, fetch ou form. Le seul point: l'<img> dans <figure> doit etre same-origin (data:/blob:/archive), pas une URL http distante (connect-src 'none' + COEP).
  - ⚠️ <img src> distant interdit (connect-src 'none' + COEP require-corp): images en data:/blob:/same-origin
  - ⚠️ le pattern carte-selectable :has(>:checked) suppose un input checkbox/radio dans la carte: OK, pur CSS, pas de form submit requis

```html
<div class="card {MODIFIER}">
  <figure><img src="{image-url}" alt="{alt-text}" /></figure>
  <div class="card-body">
    <h2 class="card-title">{title}</h2>
    <p>{CONTENT}</p>
    <div class="card-actions">{actions}</div>
  </div>
</div>
```

---

## `carousel` — carousel

Zone scrollable avec scroll-snap-type x/y mandatory et scroll-behavior:smooth (snap par item). carousel-item = flex:none + scroll-snap-align (start/center/end selon le modifier). Scrollbar masquee (scrollbar-width:none + ::-webkit-scrollbar display:none). Navigation = scroll natif ou ancres #id; pas d'autoplay CSS.

- **Sous-parties** : `carousel-item`
- **Modifiers** : `carousel-vertical`, `carousel-horizontal`, `carousel-start`, `carousel-center`, `carousel-end`
- **Mécanismes CSS** : scroll-snap-type:x/y mandatory · scroll-snap-align:start/center/end · scroll-behavior:smooth (sous @media prefers-reduced-motion:no-preference) · overflow-x/y:scroll · scrollbar-width:none + ::-webkit-scrollbar{display:none} · display:inline-flex / flex-direction column · flex:none sur item
- **CSP SBFB** : ✅ usable — Le scroll-snap et le masquage de scrollbar sont 100% CSS natif, aucune url()/fetch/font. La navigation par scroll/swipe fonctionne sans JS. Structure CSS OK, mais boutons prev/next ou autoplay = comportement a coder localement sans reseau (scrollIntoView/scrollTo en JS local, ou ancres internes #id). Images des items doivent etre same-origin.
  - ⚠️ autoplay/boutons de navigation non fournis par le CSS: a coder en JS local (scrollTo/scrollIntoView) ou via ancres #id internes — aucun reseau requis
  - ⚠️ <img> des carousel-item doivent etre same-origin/data:/blob: (connect-src 'none' + COEP)
  - ⚠️ les liens-ancres prev/next type <a href="#item2"> fonctionnent en iframe (navigation interne au document)

```html
<div class="carousel {MODIFIER}">{CONTENT}</div>
```

---

## `chat` — chat

Disposition grid d'une ligne de conversation: chat-image (span 2 lignes), chat-header (ligne 1), chat-bubble (lignes 1-3), chat-footer (ligne 3). chat-start/chat-end positionnent a gauche/droite (grid-template-columns + place-items). La bulle a une queue dessinee via ::before avec mask-image (SVG en data-URI inline dans --mask-chat) qui prend la couleur du fond (background-color:inherit). 8 couleurs de bulle.

- **Sous-parties** : `chat-image`, `chat-header`, `chat-footer`, `chat-bubble`
- **Modifiers** : `chat-start`, `chat-end`, `chat-bubble-primary`, `chat-bubble-secondary`, `chat-bubble-accent`, `chat-bubble-neutral`, `chat-bubble-info`, `chat-bubble-success`, `chat-bubble-warning`, `chat-bubble-error`
- **Mécanismes CSS** : display:grid + grid-template-columns/grid-row · mask-image:var(--mask-chat) avec SVG en data:image/svg+xml INLINE (pas d'URL distante) · mask-repeat/mask-position/mask-size · background-color:inherit (la queue prend la couleur de la bulle) · transform:rotateY pour orientation start/end + [dir=rtl] · place-items · border-radius:var(--radius-field)
- **CSP SBFB** : ✅ usable — La queue de bulle utilise mask-image mais le SVG est un data:image/svg+xml INLINE dans la variable --mask-chat (aucune URL http distante), donc compatible CSP iframe scellee (data: autorise par default-src data:). mask-image est compose et fonctionne. Pas de JS/fetch/font/form. chat-image avatar suit la regle avatar (image same-origin).
  - ⚠️ mask-image data: inline = OK (data: dans default-src); aucune URL distante donc rien a corriger
  - ⚠️ si chat-image contient un avatar <img>, image same-origin/data:/blob: requise
  - ⚠️ le rendu temps-reel d'une conversation (ajout de bulles) = JS local, le CSS ne gere que la presentation statique

```html
<div class="chat {PLACEMENT}">
  <div class="chat-image"></div>
  <div class="chat-header"></div>
  <div class="chat-bubble {COLOR}">Message text</div>
  <div class="chat-footer"></div>
</div>
```

---

## `checkbox` — checkbox

Style un <input type=checkbox> natif (appearance:none) en case carree var(--size). La coche est dessinee via ::before avec clip-path:polygon anime (transition clip-path/opacity/rotate/translate) — pas d'image. Etat :checked applique fond --input-color + revele la coche; :indeterminate dessine une barre; forced-colors/print fallback content:'✔︎'. 8 couleurs (via --input-color) + 5 tailles + :disabled opacity .2.

- **Modifiers** : `checkbox-primary`, `checkbox-secondary`, `checkbox-accent`, `checkbox-neutral`, `checkbox-info`, `checkbox-success`, `checkbox-warning`, `checkbox-error`, `checkbox-xs`, `checkbox-sm`, `checkbox-md`, `checkbox-lg`, `checkbox-xl`
- **Mécanismes CSS** : appearance:none · ::before clip-path:polygon(...) pour dessiner la coche (anime) · transition clip-path/opacity/rotate/translate · rotate:45deg · color-mix(in oklab,...) pour la bordure · :checked/[aria-checked=true]/:indeterminate states · box-shadow inset depth (oklch+var(--depth)) · @media forced-colors/print content:'✔︎' · CSS vars --input-color/--size
- **CSP SBFB** : ✅ usable — 100% CSS natif: la coche est en clip-path (aucun SVG/url()/mask distant), couleurs en color-mix/custom props. Aucun JS/fetch/font/form requis — un input checkbox fonctionne en iframe scellee. Sa valeur se lit en JS local; ne pas dependre d'un <form> submit (form-action 'none').
  - ⚠️ si dans un <form> qui submit, le submit est bloque (form-action 'none' + sandbox): lire l'etat via JS local (input.checked) + handler, pas de soumission reseau

```html
<input type="checkbox" class="checkbox {MODIFIER}" />
```

---

## `collapse` — collapse

Conteneur expansible (accordeon) qui montre/cache du contenu. L'ouverture est pilotee en CSS pur via un <input type=checkbox|radio> enfant (:checked) ou l'attribut [open]/[tabindex]:focus-within sur l'element, ou via <details>/<summary>. Les variantes collapse-arrow/collapse-plus ajoutent un indicateur visuel (chevron rotatif / signe +/-).

- **Sous-parties** : `collapse-title`, `collapse-content`
- **Modifiers** : `collapse-arrow`, `collapse-plus`, `collapse-open`, `collapse-close`, `sm:collapse`, `md:collapse`, `lg:collapse`, `xl:collapse`, `2xl:collapse`
- **Mécanismes CSS** : display:grid + grid-template-rows max-content 0fr->1fr (animation de hauteur) · transition grid-template-rows .2s (sous prefers-reduced-motion) · appearance:none + opacity:0 sur l'input checkbox/radio enfant · selecteurs :checked ~ .collapse-content et :has(>input:checked) pour l'etat ouvert (toggle 100% CSS) · content-visibility:hidden/visible + transition allow-discrete · support <details>::details-content avec interpolate-size:allow-keywords + height auto · collapse-arrow:after = chevron en box-shadow + transform rotate(45deg)->rotate(225deg) · collapse-plus:after = pseudo-content --tw-content '+'/'-' via var() · outline focus-visible color var(--color-base-content)
- **CSP SBFB** : ✅ usable — 100% CSS pur, aucun url() distant, aucun @font-face, aucun backdrop-filter, aucun mask-image. Le toggle ouvert/ferme fonctionne sans JS via input:checked ou [open] sur <details>. Aucune dependance reseau ni form submit.
  - ⚠️ Le contenu pseudo-element +/- du collapse-plus depend de --tw-content (caractere texte), sans risque CSP
  - ⚠️ prefers-reduced-motion desactive les transitions mais pas la fonctionnalite — OK
  - ⚠️ Si on veut un toggle programmatique au-dela de l'input/details natif, coder le handler localement (pas de reseau)

```html
<div tabindex="0" class="collapse {MODIFIER}">
  <div class="collapse-title">{title}</div>
  <div class="collapse-content">{CONTENT}</div>
</div>
```

---

## `countdown` — countdown

Affiche un nombre 0-999 avec un effet de transition de defilement (style afficheur mecanique) quand la valeur change. La valeur est portee par la CSS var --value sur des <span> enfants ; le rendu visuel des chiffres est genere par CSS via un pseudo-element contenant la liste 00..99.

- **Mécanismes CSS** : display:inline-flex ; enfants inline-block avec overflow-y:clip · calcul mathematique CSS pur : mod(), round(to-zero,...), clamp(), max() sur var(--value) pour extraire centaines/dizaines/unites · pseudo-elements :before/:after avec --tw-content listant '00\a 01\a ... 99\a' (chiffres pre-rendus en texte) · font-variant-numeric:tabular-nums + white-space:pre · transition top (translation verticale) cubic-bezier(1,0,0,1) 1s pour l'effet de defilement · visibility:hidden sur l'enfant, visible sur les pseudo-elements
- **CSP SBFB** : ✅ usable — Structure et animation 100% CSS (math CSS moderne + pseudo-content). Aucun url(), font distante, backdrop-filter ni mask. Le rendu fonctionne uniquement avec la var --value en style inline.
  - ⚠️ La doc precise explicitement qu'il faut changer le texte du span ET la var --value via JS pour animer un veritable compte a rebours : comportement (incrementation/decrementation) a coder localement sans reseau (setInterval pur)
  - ⚠️ Necessite un navigateur supportant mod()/round()/clamp() CSS (Chromium recent — OK dans l'iframe Chromium)
  - ⚠️ Accessibilite : ajouter aria-live=polite + aria-label={number} (cf. regle doc)

```html
<span class="countdown">
  <span style="--value:{number};">number</span>
</span>
```

---

## `diff` — diff

Comparaison cote-a-cote de deux elements (images ou contenu) avec une poignee de redimensionnement glissable qui revele plus ou moins de chaque cote. Le slider est realise via resize:horizontal natif sur .diff-resizer (aucun JS).

- **Sous-parties** : `diff-item-1`, `diff-item-2`, `diff-resizer`
- **Modifiers** : `sm:diff`, `md:diff`, `lg:diff`, `xl:diff`, `2xl:diff`
- **Mécanismes CSS** : display:grid (grid-template-rows 1fr 1.8rem 1fr / columns auto 1fr) avec superposition des items dans la meme cellule · container-type:inline-size + unites cqi (container query) pour les largeurs · resize:horizontal sur .diff-resizer = poignee glissable native (pas de JS) · clip-path:inset(...) + transform scaleY/translate pour la zone de saisie du resizer · color-mix(in oklab, var(--color-base-100) ...) pour le voile du :after de diff-item-2 · outline focus-visible + selecteurs :focus / :has(.diff-item-1:focus-visible) qui modifient min/max-width du resizer (interaction clavier CSS) · object-fit:cover sur les enfants media · user-select:none ; pointer-events:none sur les contenus
- **CSP SBFB** : ✅ usable — Le mecanisme de comparaison/slider est 100% CSS (resize natif + container queries + :focus). Aucun url() distant, font, backdrop-filter ou mask dans la feuille. Les images placees a l'interieur doivent etre same-origin (data:/blob:/relatives), pas de http distant.
  - ⚠️ Les <img> internes doivent venir de sources autorisees par la CSP (self/data:/blob:) ; une image http distante serait bloquee (mais c'est le contenu de l'app, pas le composant)
  - ⚠️ COEP require-corp : toute image cross-origin sans CORP serait bloquee — utiliser des assets locaux inline
  - ⚠️ Aucun risque fill-*/stroke-* (composant non-SVG)

```html
<figure class="diff">
  <div class="diff-item-1">{item1}</div>
  <div class="diff-item-2">{item2}</div>
  <div class="diff-resizer"></div>
</figure>
```

---

## `divider` — divider

Separateur visuel horizontal ou vertical, avec texte optionnel au centre. Les lignes sont dessinees par les pseudo-elements :before/:after en flex.

- **Modifiers** : `divider-neutral`, `divider-primary`, `divider-secondary`, `divider-accent`, `divider-success`, `divider-warning`, `divider-info`, `divider-error`, `divider-vertical`, `divider-horizontal`, `divider-start`, `divider-end`, `sm:divider`, `md:divider`, `lg:divider`, `xl:divider`, `2xl:divider`
- **Mécanismes CSS** : display:flex + pseudo :before/:after flex-grow:1 formant les deux segments de ligne · --divider-color:color-mix(in oklab, var(--color-base-content) 10%, transparent) · modifiers couleur surchargent background-color des :before/:after via var(--color-*) · divider-horizontal -> flex-direction:column (ligne verticale) ; divider-vertical -> row · divider-start:before{display:none} / divider-end:after{display:none} pour aligner le texte · @media print : border .5px solid sur les segments · gap:1rem si non vide
- **CSP SBFB** : ✅ usable — CSS pur (flex + pseudo-elements + color-mix + var). Aucune dependance reseau, font, backdrop-filter, mask ni url(). Totalement statique.
  - ⚠️ Aucun risque CSP identifie

```html
<div class="divider {MODIFIER}">{text}</div>
```

---

## `dock` — dock

Barre de navigation fixee en bas de l'ecran (bottom navigation / tab bar mobile). Contient une liste de boutons avec icone + dock-label. dock-active marque l'onglet courant via un indicateur :after.

- **Sous-parties** : `dock-label`
- **Modifiers** : `dock-active`, `dock-xs`, `dock-sm`, `dock-md`, `dock-lg`, `dock-xl`, `sm:dock`, `md:dock`, `lg:dock`, `xl:dock`, `2xl:dock`
- **Mécanismes CSS** : position:fixed bottom:0 left:0 right:0 ; display:flex justify-content:space-around · height:calc(4rem + env(safe-area-inset-bottom)) + padding-bottom:env(safe-area-inset-bottom) (safe-area iOS) · border-top color-mix(in oklab, var(--color-base-content) 5%, transparent) · enfants flex-direction:column ; transition opacity hover · dock-active:after = indicateur (background-color:currentColor, width 2.5rem) ; transition width/background-color · etats [aria-disabled=true]/[disabled] : pointer-events:none + couleur attenuee via color-mix · tailles dock-xs..dock-xl ajustent height + font-size du dock-label
- **CSP SBFB** : ✅ usable — Structure et style 100% CSS (flex fixe, currentColor, color-mix, env() safe-area). Aucun url(), font distante, backdrop-filter ni mask. La navigation active est appliquee via la classe dock-active.
  - ⚠️ L'icone est typiquement un <svg> inline : le peindre via currentColor / var(--color-*) ; les utilitaires Tailwind fill-*/stroke-* ne compilent pas dans l'iframe scellee — utiliser fill="currentColor" sur le SVG
  - ⚠️ Le changement d'onglet actif (toggle dock-active) suppose du JS : coder le handler localement, aucun reseau requis
  - ⚠️ env(safe-area-inset-bottom) sans effet hors contexte mobile mais inoffensif

```html
<button>
    <svg>{icon}</svg>
    <span class="dock-label">Text</span>
</button>
```

---

## `drawer` — drawer

Layout en grille avec une barre laterale (sidebar) coulissante a gauche (ou a droite via drawer-end) qui peut s'ouvrir/se fermer. Le toggle est un checkbox cache (.drawer-toggle) actionne par un <label for>. drawer-open garde la sidebar visible (ex. lg:drawer-open).

- **Sous-parties** : `drawer-toggle`, `drawer-content`, `drawer-side`, `drawer-overlay`
- **Modifiers** : `drawer-end`, `drawer-open`, `sm:drawer`, `md:drawer`, `lg:drawer`, `xl:drawer`, `2xl:drawer`, `is-drawer-open:`, `is-drawer-close:`
- **Mécanismes CSS** : display:grid grid-auto-columns:max-content auto · checkbox .drawer-toggle cache (opacity:0,width/height:0) ; etat via :checked ~ .drawer-side · translate:-100% -> 0% sur .drawer-side > :not(.drawer-overlay) (glissement) + transition translate .3s · drawer-overlay = voile cliquable background-color:oklch(0% 0 0/.4) (couleur RGBA, PAS backdrop-filter) · visibility/opacity + transition allow-discrete pour l'apparition · support [dir=rtl] (translate:100%) · scroll-driven animation : animation set-page-has-scroll scroll() + var --page-* sur :root:has(.drawer-toggle:checked) pour figer le scroll de page · drawer-end -> grid-auto-columns:auto max-content + colonne inversee · drawer-open -> sidebar position:sticky visible
- **CSP SBFB** : ✅ usable — Toggle 100% CSS via checkbox:checked + label[for], aucun JS requis pour ouvrir/fermer. Le voile (drawer-overlay) est une couleur unie oklch translucide, PAS un backdrop-filter — aucun cout de compositing special. Aucun url(), font distante ni mask.
  - ⚠️ Le label[for] qui pilote le checkbox fonctionne dans la sandbox allow-scripts sans allow-forms (ce n'est pas un form submit) — OK
  - ⚠️ Les variantes is-drawer-open:/is-drawer-close: sont des variants Tailwind/daisyUI a la compilation : verifier qu'elles sont incluses dans le build purge si utilisees
  - ⚠️ La scroll-driven animation (animation-timeline:scroll()) est progressive-enhancement ; sans support, le drawer reste fonctionnel

```html
<div class="drawer {MODIFIER}">
  <input id="my-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content">{CONTENT}</div>
  <div class="drawer-side">{SIDEBAR}</div>
</div>
```

---

## `dropdown` — dropdown

Affiche un menu (.dropdown-content) au clic/focus/hover sur un bouton. Trois mecanismes : <details>/<summary>, l'API popover native ([popover] + popovertarget + position-anchor), ou le focus CSS (tabindex + :focus-within).

- **Sous-parties** : `dropdown-content`
- **Modifiers** : `dropdown-start`, `dropdown-center`, `dropdown-end`, `dropdown-top`, `dropdown-bottom`, `dropdown-left`, `dropdown-right`, `dropdown-hover`, `dropdown-open`, `dropdown-close`, `sm:dropdown`, `md:dropdown`, `lg:dropdown`, `xl:dropdown`, `2xl:dropdown`
- **Mécanismes CSS** : position-area / anchor positioning (var --anchor-v/--anchor-h, position-anchor) pour le placement · display:none + opacity:0 + scale:95% a l'etat ferme ; opacity:1 scale:100% via :focus-within / .dropdown-open / .dropdown-hover:hover / :popover-open · @starting-style + transition allow-discrete (opacity, scale, display, overlay) + @keyframes dropdown · support [popover] natif : ::backdrop background-color oklab(... /.3) (couleur, pas filter) ; @supports not(position-area) fallback margin:auto · masquage du marqueur ::-webkit-details-marker · z-index:999
- **CSP SBFB** : ✅ usable — Ouverture sans JS possible via <details>/<summary>, focus CSS (:focus-within) ou popover natif. Aucun url(), font distante, backdrop-filter ni mask. ::backdrop du popover est une simple couleur translucide.
  - ⚠️ L'API popover (popovertarget) et l'anchor positioning (position-area) requierent un Chromium recent — disponible dans l'iframe Chromium ; sinon fallback CSS prevu
  - ⚠️ Le placement par anchor-name/position-anchor passe par des styles inline (style="anchor-name:--x") — pas de risque CSP
  - ⚠️ Fermeture au clic exterieur fiable avec popover/details natifs ; sinon coder localement

```html
<details class="dropdown">
  <summary>Button</summary>
  <ul class="dropdown-content">{CONTENT}</ul>
</details>
```

---

## `fab` — fab

Floating Action Button fixe dans le coin bas. Au focus, revele des boutons d'action secondaires (speed dial) en colonne, ou en eventail quart-de-cercle avec fab-flower. fab-close/fab-main-action remplacent le bouton original quand ouvert.

- **Sous-parties** : `fab-close`, `fab-main-action`
- **Modifiers** : `fab-flower`, `sm:fab`, `md:fab`, `lg:fab`, `xl:fab`, `2xl:fab`
- **Mécanismes CSS** : position:fixed bottom:1rem inset-inline-end:1rem z-index:999 · ouverture via :focus-within (CSS pur, pas de JS) : enfants n+2 passent de visibility:hidden/opacity:0/scale:80% a visible/opacity:1/scale:100% · transition opacity/scale/visibility + transition-delay echelonne (nth-child 3..6) · rotate:90deg sur le bouton original quand fab-close/fab-main-action present · fab-flower : disposition radiale via transform translateX(cos(var(--degree))*var(--position)) translateY(sin(...)) — trigonometrie CSS (cos/sin) · var --degree/--position calcules selon :has(:nth-child(n)) ; support [dir=rtl] (--flip-degree)
- **CSP SBFB** : ✅ usable — Ouverture/fermeture 100% CSS via :focus-within ; disposition flower via fonctions trigonometriques CSS (cos/sin). Aucun url(), font distante, backdrop-filter ni mask. S'appuie sur le composant btn (lui-meme CSS pur).
  - ⚠️ L'ouverture repose sur :focus-within : sur certains environnements tactiles le focus peut etre capricieux — un handler JS local peut completer (sans reseau)
  - ⚠️ Les icones SVG internes doivent etre peintes via currentColor/var(--color-*) ; fill-*/stroke-* Tailwind ne compilent pas dans l'iframe
  - ⚠️ cos()/sin() CSS requierent un navigateur recent (Chromium OK) ; sinon la version verticale (sans fab-flower) reste fonctionnelle

```html
<div class="fab">
  <div tabindex="0" role="button" class="btn btn-lg btn-circle btn-primary">{IconOriginal}</div>
  <button class="btn btn-lg btn-circle">{Icon1}</button>
  <button class="btn btn-lg btn-circle">{Icon2}</button>
  <button class="btn btn-lg btn-circle">{Icon3}</button>
</div>
```

---

## `fieldset` — fieldset

Conteneur en grille pour grouper des elements de formulaire apparentes. fieldset-legend sert de titre, et un .label (ou .fieldset-label dans la CSS) sert de description/texte d'aide. Note : la doc liste 'label' comme classe de description, la CSS source definit en plus .fieldset-label.

- **Sous-parties** : `fieldset-legend`, `fieldset-label`
- **Modifiers** : `sm:fieldset`, `md:fieldset`, `lg:fieldset`, `xl:fieldset`, `2xl:fieldset`
- **Mécanismes CSS** : display:grid grid-template-columns:1fr grid-auto-rows:max-content gap:.375rem · fieldset-legend : display:flex justify-content:space-between font-weight:600 color var(--color-base-content) · fieldset-label : color color-mix(in oklab, var(--color-base-content) 60%, transparent) ; cursor:pointer si :has(input) · padding-block + font-size .75rem
- **CSP SBFB** : ✅ usable — Pur layout/typographie CSS (grid + flex + color-mix + var). Aucun url(), font distante, backdrop-filter ni mask. Element semantique <fieldset>/<legend> standard.
  - ⚠️ fieldset est typiquement utilise dans un <form> : sous sandbox allow-scripts sans allow-forms et form-action 'none', NE PAS compter sur un submit natif — utiliser div/button + handler JS local pour traiter les donnees sans reseau
  - ⚠️ Le composant lui-meme n'impose aucun <form> ; il peut etre utilise hors form sans risque

```html
<fieldset class="fieldset">
  <legend class="fieldset-legend">{title}</legend>
  {CONTENT}
  <p class="label">{description}</p>
</fieldset>
```

---

## `file-input` — fileinput

Champ de televersement de fichier stylise. Style le <input type=file> natif et son pseudo-element ::file-selector-button (qui herite du look des btn daisyUI : couleur, ombre, bruit).

- **Sous-parties** : `::file-selector-button (pseudo-element, pas une classe)`
- **Modifiers** : `file-input-ghost`, `file-input-neutral`, `file-input-primary`, `file-input-secondary`, `file-input-accent`, `file-input-info`, `file-input-success`, `file-input-warning`, `file-input-error`, `file-input-xs`, `file-input-sm`, `file-input-md`, `file-input-lg`, `file-input-xl`, `sm:file-input`, `md:file-input`, `lg:file-input`, `xl:file-input`, `2xl:file-input`
- **Mécanismes CSS** : height:var(--size) (--size = var(--size-field)*N selon la taille) ; border var(--border) · box-shadow inset depth via color-mix + var(--depth) ; --input-color color-mix(in oklab, var(--color-base-content) 20%) · border-radius via var(--radius-field) et var(--join-*) (compat join) · ::file-selector-button stylise comme un bouton : --btn-bg/--btn-fg/--btn-border, box-shadow, text-shadow oklch · background-image:var(--btn-noise) = var(--fx-noise) (texture de bruit, definie ailleurs dans le theme) · modifiers couleur surchargent --btn-color et --input-color via var(--color-*-content) / var(--color-*) · etats :focus (outline 2px) et :disabled (color-mix attenue, --btn-noise:none)
- **CSP SBFB** : ✅ usable — Style 100% CSS via variables de theme et color-mix ; aucun url() distant ni font distante ; le ::file-selector-button est un pseudo-element natif, pas de JS pour l'apparence. var(--fx-noise) est une texture de bruit definie par le theme daisyUI vendore (souvent un data:URI SVG inline same-origin, pas une URL http distante).
  - ⚠️ Verifier la valeur de --fx-noise dans le theme vendore : si c'est un url() data: inline => OK ; si jamais un url() http distant => bloque (controler au build). En pratique daisyUI 5 utilise un SVG inline data:
  - ⚠️ La selection de fichier ouvre le picker natif du navigateur ; en iframe sandbox allow-scripts (sans allow-same-origin) le picker fonctionne, mais l'upload reseau est impossible (connect-src 'none') — lire le fichier en local via FileReader/Blob, aucun envoi reseau
  - ⚠️ Si place dans un <form>, le submit est bloque (form-action 'none' + sandbox) : traiter via JS local (input.files)

```html
<input type="file" class="file-input {MODIFIER}" />
```

---

## `filter` — filter

Groupe de boutons radio agissant comme un filtre exclusif : quand un radio est coche (sauf le reset), tous les autres se replient (opacity 0, width 0, border 0, scale 0) via transitions, ne laissant que l'option selectionnee + le bouton reset. Le reset (filter-reset / input[type=reset]) reapparait des qu'une option est cochee.

- **Sous-parties** : `filter-reset`
- **Mécanismes CSS** : flex-wrap + display:flex · selecteur :has(input:checked:not(.filter-reset)) pour piloter le repli · negation :not(:has(...)) pour cacher le reset quand rien n'est coche · transition margin/opacity/padding/border-width/scale · scale + aspect-ratio sur filter-reset · --tw-content + content pour injecter le glyphe × via :after · input[type=radio] / input[type=reset] state-driven, 100% CSS
- **CSP SBFB** : ✅ usable — Pur CSS state-driven via :checked/:has, aucune url() distante, aucun mask/backdrop, aucun fetch. Fonctionne dans l'iframe scellee. ATTENTION : la doc montre un <form>. Le submit d'un <form> est bloque par form-action 'none' + sandbox sans allow-forms, MAIS ici le form ne fait que conteneur de radios filtrants (pas de submit) — le repli est purement CSS sur :checked, donc l'effet visuel marche. Le bouton type=reset reinitialise les radios cote DOM (geste natif autorise, pas une navigation). Si une vraie soumission est voulue, utiliser div+button+handler JS local.
  - ⚠️ la doc privilegie <form> ; tout submit reseau est bloque (form-action 'none' + sandbox), mais le filtrage CSS n'en depend pas
  - ⚠️ content:var(--tw-content) avec "×" : OK, contenu CSS local
  - ⚠️ si on attend une action sur changement, il faut un handler JS local (connect-src 'none')

```html
<form class="filter">
  <input class="btn btn-square" type="reset" value="×"/>
  <input class="btn" type="radio" name="{NAME}" aria-label="Tab 1 title"/>
  <input class="btn" type="radio" name="{NAME}" aria-label="Tab 2 title"/>
</form>
```

---

## `footer` — footer

Layout de pied de page en CSS grid : grid-auto-flow row par defaut, gap 2.5rem/1rem, place-items start. footer-center centre le texte et passe en flow column dense. footer-horizontal force grid-auto-flow:column, footer-vertical force row. footer-title met le titre en uppercase, opacity .6, semi-bold.

- **Sous-parties** : `footer-title`
- **Modifiers** : `footer-center`, `footer-horizontal`, `footer-vertical`, `sm:footer`, `sm:footer-horizontal`, `md:footer-horizontal`, `lg:footer-horizontal`, `xl:footer-horizontal`, `2xl:footer-horizontal`, `sm:footer-vertical`, `footer-title (part)`
- **Mécanismes CSS** : display:grid + grid-auto-flow row/column dense · place-items:start / center · gap (2.5rem 1rem) · text-transform:uppercase + opacity sur footer-title · variantes responsive media (width>=640/768/1024/1280/1536) compilees comme classes sm:/md:/lg:/xl:/2xl:
- **CSP SBFB** : ✅ usable — Pur layout grid + typographie, aucune ressource distante, aucun JS. 100% CSS pur, totalement sur dans l'iframe scellee.
  - ⚠️ la suggestion doc 'utiliser base-200 pour le fond' = simple classe Tailwind locale, aucun risque
  - ⚠️ les liens internes du footer (<a>) : navigation interne OK ; liens externes bloques par sandbox/navigation, sans danger pour le rendu

```html
<footer class="footer {MODIFIER}">{CONTENT}</footer>
```

---

## `hero` — hero

Conteneur grand format centre : CSS grid avec tous les enfants empiles sur grid-row/column 1 (superposition). background-size:cover + background-position:50% pour image de fond. hero-overlay pose une couche couleur semi-transparente (color-mix neutral 50%) par-dessus l'image. hero-content centre le contenu (flex, gap, max-width 80rem, isolation:isolate).

- **Sous-parties** : `hero-content`, `hero-overlay`
- **Modifiers** : `sm:hero`, `md:hero`, `lg:hero`, `xl:hero`, `2xl:hero`
- **Mécanismes CSS** : display:grid avec enfants tous en grid-row/column-start:1 (overlay stacking) · background-size:cover + background-position:50% · color-mix(in oklab, var(--color-neutral) 50%, transparent) pour l'overlay · isolation:isolate sur hero-content · flex + place-items:center
- **CSP SBFB** : ✅ usable — Structure CSS pure (grid + color-mix), totalement OK dans l'iframe scellee. ATTENTION : hero utilise tres souvent background-image. Si l'image est servie via une URL http distante (style inline background-image:url('https://...')), elle est BLOQUEE par CSP (connect-src 'none' / origine opaque). Servir l'image en local (same-origin dans l'archive : url('./hero.webp')), en data: URI, ou via blob:.
  - ⚠️ background-image:url(distant) interdit — utiliser asset local same-origin, data: ou blob:
  - ⚠️ les exemples vitrine daisyUI pointent vers img.daisyui.com : a remplacer par asset local
  - ⚠️ color-mix oklab : supporte navigateurs modernes, OK composite

```html
<div class="hero {MODIFIER}">{CONTENT}</div>
```

---

## `hover-3d` — hover3d

Wrapper a 9 enfants exacts : le 1er est le contenu visible (image/carte), les 8 suivants sont des zones de survol invisibles disposees en grille 3x3. Au survol d'une zone, :has(>:nth-child(N):hover) recalcule des custom properties (--transform, --shine, --shadow) qui font pivoter le 1er enfant en rotate3d, deplacent un reflet radial (::before) et l'ombre (drop-shadow), creant un tilt 3D. perspective:75rem, easing custom (linear() avec keyframes).

- **Mécanismes CSS** : perspective:75rem + transform:rotate3d() pilote par var(--transform) · custom properties --transform/--shine/--shadow recalculees via :has(>:nth-child(N):hover) · filter:drop-shadow() empile (4 couches) anime par --shadow · ::before reflet radial-gradient(circle, #fff3, #0000) deplace via translate var(--shine) + blur · easing linear(...) custom (cubic-like spring) · grid-area 3x3 pour les 8 zones de hover · scale au hover, isolation/z-index · 100% CSS, effet pilote au pointeur (hover) sans JS
- **CSP SBFB** : ✅ usable — Effet entierement CSS (perspective, rotate3d, drop-shadow, radial-gradient inline, :has) — aucun JS requis, aucune url() distante dans le CSS du composant (le reflet est un gradient genere). Sur dans l'iframe scellee. SEUL piege : l'<img> de l'exemple pointe vers img.daisyui.com (distant) — interdit. Mettre l'image en local same-origin / data: / blob:. Le gradient #fff3 est inline-generated, pas une ressource.
  - ⚠️ l'img d'exemple est distante (img.daisyui.com) — remplacer par asset local
  - ⚠️ necessite exactement 9 enfants directs ; contenu non-interactif (la doc l'exige)
  - ⚠️ drop-shadow filters multiples : cout GPU mais composite OK
  - ⚠️ aucun mask-image distant, aucun fetch

```html
<div class="hover-3d my-12 mx-2">
  <figure class="max-w-100 rounded-2xl">
    <img src="https://img.daisyui.com/images/stock/creditcard.webp" alt="Tailwind CSS 3D card" />
  </figure>
  <div></div>
  <div></div>
  <div></div>
  <div></div>
  <div></div>
  <div></div>
  <div></div>
  <div></div>
</div>
```

---

## `hover-gallery` — hovergallery

Galerie d'images (jusqu'a 10) en grille : la 1ere image occupe toute la largeur (grid-column 1/-1, opacity 1), les autres sont a opacity 0 reparties en colonnes. Le nombre de colonnes (--items) s'auto-ajuste via :has(>:nth-child(N)). Au survol horizontal d'une zone (:hover sur un enfant), cette image passe opacity 1 et prend toute la largeur, et la 1ere image se masque (:has(:hover) > :first-child{display:none}). Au-dela de 10 enfants : display:none.

- **Modifiers** : `sm:hover-gallery`, `md:hover-gallery`, `lg:hover-gallery`, `xl:hover-gallery`, `2xl:hover-gallery`
- **Mécanismes CSS** : display:inline-grid + grid-template-columns:repeat(var(--items),1fr) · --items auto-calcule via chaine de :has(>:nth-child(N)) · opacity 0/1 + grid-column 1/-1 au :hover de chaque enfant · object-fit:cover · overflow:hidden, gap 1px · variantes responsive sm:/md:/lg:/xl:/2xl: · 100% CSS hover-driven, sans JS ni carousel auto
- **CSP SBFB** : ✅ usable — Mecanique de bascule purement CSS (:hover + :has + grid), aucun timer/JS (pas de carousel auto). Structure CSS OK dans l'iframe. SEUL piege majeur : ce composant EST une galerie d'images — les <img> doivent etre servies en local same-origin (assets de l'archive), data: ou blob:. Les URLs img.daisyui.com de l'exemple sont distantes et BLOQUEES par CSP. Les images doivent avoir la meme dimension (doc).
  - ⚠️ src d'images distants (img.daisyui.com) interdits — utiliser assets locaux same-origin/data:/blob:
  - ⚠️ COEP require-corp : les images doivent etre CORP-compatibles ; same-origin dans l'archive = OK
  - ⚠️ necessite un max-width (sinon remplit le conteneur)
  - ⚠️ comportement = hover CSS uniquement ; pas d'auto-rotation (aucun JS requis)

```html
<figure class="hover-gallery max-w-60">
  <img src="https://img.daisyui.com/images/stock/daisyui-hat-1.webp" />
  <img src="https://img.daisyui.com/images/stock/daisyui-hat-2.webp" />
  <img src="https://img.daisyui.com/images/stock/daisyui-hat-3.webp" />
  <img src="https://img.daisyui.com/images/stock/daisyui-hat-4.webp" />
</figure>
```

---

## `indicator` — indicator

Positionne un element (badge/point) sur le coin d'un autre. Le conteneur .indicator est inline-flex position:relative ; .indicator-item est position:absolute, place par defaut en haut-droite (translate 50%,-50%). Les classes placement modifient des custom properties --indicator-t/b/s/e/x/y pour deplacer l'item (start/center/end horizontal, top/middle/bottom vertical), avec support RTL.

- **Sous-parties** : `indicator-item`
- **Modifiers** : `indicator-start`, `indicator-center`, `indicator-end`, `indicator-top`, `indicator-middle`, `indicator-bottom`, `sm:indicator`, `sm:indicator-top`, `md:indicator-end`, `lg:indicator-bottom`
- **Mécanismes CSS** : position:relative conteneur + position:absolute sur indicator-item · custom properties --indicator-t/-b/-s/-e/-x/-y pour le placement · translate var(--indicator-x) var(--indicator-y) · z-index:1, white-space:nowrap · support [dir=rtl] inversant start/end · variantes responsive compilees · 100% CSS pur
- **CSP SBFB** : ✅ usable — Pur positionnement CSS via custom properties et translate, aucune ressource distante, aucun JS. Totalement sur dans l'iframe scellee.

```html
<div class="indicator">
  <span class="indicator-item">{indicator content}</span>
  <div>{main content}</div>
</div>
```

---

## `input` — input

Champ de saisie texte stylise (peut aussi etre un conteneur inline-flex enveloppant un <input> nu + icones/labels). Hauteur var(--size), bordure var(--border), box-shadow depth (inset), border-radius var(--radius-field) (avec hooks join-ss/se/ee/es). Au focus/focus-within : --input-color passe a base-content + outline 2px. Couleurs via --input-color (color-mix). Tailles via --size. Gere disabled, input[type=number] spin-button, date picker indicator.

- **Modifiers** : `input-ghost`, `input-neutral`, `input-primary`, `input-secondary`, `input-accent`, `input-info`, `input-success`, `input-warning`, `input-error`, `input-xs`, `input-sm`, `input-md`, `input-lg`, `input-xl`, `sm:input`, `md:input`
- **Mécanismes CSS** : border + box-shadow inset pilotes par --depth et --input-color (color-mix oklab) · --size = var(--size-field)*N pour les tailles · appearance:none + reset interne sur :where(input) · focus/focus-within : outline 2px + isolation · color-mix(in oklab, var(--color-*) ...) pour couleurs et etats disabled · ::-webkit-calendar-picker-indicator / ::-webkit-inner-spin-button (number/date) · @media (forced-colors:active) et (pointer:coarse) ajustements · border-radius via var(--join-*) pour integration join · 100% CSS
- **CSP SBFB** : ✅ usable — Style purement CSS, aucune url() distante, aucune dependance reseau. Le composant <input> lui-meme fonctionne (saisie locale). Sur dans l'iframe. ATTENTION CSP : un <input> a l'interieur d'un <form> dont l'action enverrait des donnees est bloque (form-action 'none' + sandbox sans allow-forms) — toute soumission/validation doit etre geree par un handler JS local (div+button+click), connect-src 'none' empeche tout envoi reseau.
  - ⚠️ si place dans un <form> avec submit : submit bloque par form-action 'none' + sandbox — gerer via handler JS local
  - ⚠️ aucune validation/recherche reseau possible (connect-src 'none')
  - ⚠️ pseudo-elements webkit (date/number) dependants du navigateur, pas un risque CSP

```html
<input type="{type}" placeholder="Type here" class="input {MODIFIER}" />
```

---

## `kbd` — kbd

Affiche une touche clavier : pastille inline-flex, border-radius var(--radius-field), fond base-200, bordure color-mix (base-content 20%) avec un bord inferieur plus epais (border + 1px) pour l'effet 'touche'. Taille via --size (var(--size-selector)*N), min-width=height pour les touches carrees, padding-inline .5em.

- **Modifiers** : `kbd-xs`, `kbd-sm`, `kbd-md`, `kbd-lg`, `kbd-xl`, `sm:kbd`, `md:kbd`, `lg:kbd`, `xl:kbd`, `2xl:kbd`
- **Mécanismes CSS** : border + border-bottom plus epais (calc(var(--border)+1px)) pour relief de touche · color-mix(in srgb, var(--color-base-content) 20%, transparent) pour les bordures · --size = var(--size-selector)*N (height + min-width) · background-color:var(--color-base-200) · box-shadow:none, display:inline-flex · variantes responsive sm:/md:/lg:/xl:/2xl: · 100% CSS pur
- **CSP SBFB** : ✅ usable — Purement decoratif en CSS (border, color-mix, var), aucune ressource distante ni JS. Totalement sur dans l'iframe scellee.

```html
<kbd class="kbd {MODIFIER}">K</kbd>
```

---

## `label` — label

label : conteneur inline-flex pour texte de label + champ ; color attenuee (currentcolor 60%), cursor:pointer si contient un input. Comme enfant d'un .input/.select il devient un add-on lateral avec bordure separatrice. floating-label : parent positionne d'un input + un <span> qui flotte au-dessus du champ ; le span est cache (opacity 0) tant que le placeholder est visible, et remonte (translate + scale .75) au focus-within ou quand le champ est rempli (:not(:has(input:placeholder-shown))). Tailles du span s'adaptent a input-xs..xl.

- **Sous-parties** : `floating-label`
- **Mécanismes CSS** : color-mix(in oklab, currentcolor 60%, transparent) pour la teinte · floating-label : transition top/translate/scale/opacity sur le span + ::placeholder · selecteurs :focus-within et :not(:has(input:placeholder-shown)) pour declencher le flottement · scale:.75 + translate pour la position flottante · :has(.input-xs/.select-xs/...) span pour adapter la taille · cursor:pointer via :has(input) · border-inline separatrice quand label est add-on de .input/.select · 100% CSS pur (state-driven placeholder-shown)
- **CSP SBFB** : ✅ usable — Effet flottant purement CSS via :placeholder-shown / :focus-within, aucune ressource distante, aucun JS. Sur dans l'iframe. Note : <label> wrappe un <input> (saisie locale OK) ; aucune soumission reseau implicite.
  - ⚠️ si l'input enveloppe est dans un <form> a submit : envoi bloque (form-action 'none' + sandbox), gerer en JS local
  - ⚠️ aucun risque CSS/ressource

```html
<label class="floating-label">
  <input type="text" placeholder="Type here" class="input" />
  <span>{label text}</span>
</label>
```

---

## `link` — link

Ajoute le soulignement manquant aux liens (text-decoration-line:underline) + cursor:pointer, gestion focus/focus-visible (outline 2px). link-hover n'affiche le soulignement qu'au survol. Les modifiers couleur appliquent var(--color-*) avec un assombrissement au hover (color-mix 80% + #000), tous sous @media (hover:hover).

- **Modifiers** : `link-hover`, `link-neutral`, `link-primary`, `link-secondary`, `link-accent`, `link-success`, `link-info`, `link-warning`, `link-error`, `sm:link`, `md:link`, `lg:link`
- **Mécanismes CSS** : text-decoration-line:underline · cursor:pointer · focus-visible outline 2px + @media (forced-colors:active) fallback · color:var(--color-*) par modifier · @media (hover:hover) :hover color-mix(in oklab, var(--color-*) 80%, #000) · link-hover : underline seulement au hover · variantes responsive sm:/md:/lg: · 100% CSS pur
- **CSP SBFB** : ✅ usable — Pur style typographique (underline + couleur), aucune ressource distante, aucun JS. Le style fonctionne dans l'iframe. Note comportementale : un <a href> vers une URL externe est neutralise par la sandbox/origine opaque (la navigation/ouverture est bloquee) ; le style reste correct. Pour une action, utiliser un handler JS local. Aucun risque CSP cote CSS.
  - ⚠️ href externe non navigable depuis l'iframe scellee (sandbox/origine opaque) — le style s'applique mais le clic ne navigue pas hors iframe ; utiliser handler JS local pour une action interne
  - ⚠️ aucun risque ressource/url() distante

```html
<a class="link {MODIFIER}">Click me</a>
```

---

## `list` — list

Conteneur vertical (flex column, font-size .875rem) dont chaque enfant .list-row est une grille CSS en colonnes. Le 2e enfant remplit l'espace par defaut ; list-col-grow deplace ce 1fr via :has() ; list-col-wrap force un enfant sur la 2e ligne (grid-row-start:2). Separateurs entre lignes via :after border-bottom.

- **Sous-parties** : `list-row`, `list-col-wrap`, `list-col-grow`
- **Modifiers** : `list-col-wrap`, `list-col-grow`, `sm:list`, `md:list`, `lg:list`, `xl:list`, `2xl:list`
- **Mécanismes CSS** : display:flex / flex-direction:column · display:grid sur .list-row · grid-template-columns:var(--list-grid-cols) avec minmax(0,auto) · grid-auto-flow:column · selecteur :has(.list-col-grow:nth-child(n)) pour recalculer les colonnes · pseudo :after border-bottom comme separateur · color-mix(in oklab, var(--color-base-content) 5%, transparent) · var(--radius-box), var(--border) · word-break:break-word
- **CSP SBFB** : ✅ usable — 100% layout CSS pur (flex/grid, :has(), color-mix, custom props). Aucune url() distante, aucun mask-image, aucun JS requis, pas de form. Compile build-time en CSS same-origin.
  - ⚠️ Aucun risque CSP
  - ⚠️ color-mix et :has() requierent un moteur recent (Chromium/WebKit modernes) mais sont composites/locaux

```html
<ul class="list">
  <li class="list-row">{CONTENT}</li>
</ul>
```

---

## `loading` — loading

Indicateur de chargement anime : un element inline-block dont la couleur vient de currentColor (background-color:currentColor) et dont la forme/animation est portee par un mask-image SVG anime (animateTransform/animate dans le SVG). Les modifiers de style changent le SVG du mask, les tailles changent la width (multiple de --size-selector).

- **Modifiers** : `loading-spinner`, `loading-dots`, `loading-ring`, `loading-ball`, `loading-bars`, `loading-infinity`, `loading-xs`, `loading-sm`, `loading-md`, `loading-lg`, `loading-xl`
- **Mécanismes CSS** : mask-image: url("data:image/svg+xml,...") (SVG INLINE data:, pas distant) · background-color:currentColor (peint la forme du mask) · animation SVG native via <animateTransform>/<animate> embarques dans le data: SVG (SMIL) · aspect-ratio:1 · mask-position/mask-size/mask-repeat · width calc(var(--size-selector,.25rem)*n) · pointer-events:none
- **CSP SBFB** : ✅ usable — Le mask-image utilise une data:image/svg+xml INLINE (autorisee par data: dans la CSP et same-origin une fois dans app.css). L'animation est portee par le SVG lui-meme (SMIL), pas de JS ni de fetch. Aucune URL http distante.
  - ⚠️ mask-image present mais data: inline donc OK sous la CSP (data: autorise)
  - ⚠️ L'animation SMIL du SVG dans un mask : supportee par Chromium/WebKit ; purement locale, aucun reseau
  - ⚠️ Aucune url() distante, pas de fill-*/stroke-* Tailwind (le SVG porte ses propres attributs stroke/fill)

```html
<span class="loading loading-spinner loading-lg"></span>
```

---

## `mask` — mask

Recadre le contenu d'un element (souvent une <img>) selon une forme. La forme est appliquee via mask-image avec un SVG inline data: (chemin de la forme). mask-half-1/2 affichent une moitie via mask-position + mask-size:200% (avec variantes RTL).

- **Modifiers** : `mask-squircle`, `mask-heart`, `mask-hexagon`, `mask-hexagon-2`, `mask-decagon`, `mask-pentagon`, `mask-diamond`, `mask-square`, `mask-circle`, `mask-star`, `mask-star-2`, `mask-triangle`, `mask-triangle-2`, `mask-triangle-3`, `mask-triangle-4`, `mask-half-1`, `mask-half-2`
- **Mécanismes CSS** : mask-image: url("data:image/svg+xml,...") (SVG INLINE data: par forme) · mask-size:contain / mask-repeat:no-repeat / mask-position:50% · mask-half-1/2 : mask-size:200% + mask-position 0/100% + variante :dir(rtl) · display:inline-block / vertical-align:middle
- **CSP SBFB** : ✅ usable — Toutes les formes sont des data:image/svg+xml INLINE, autorisees par data: dans la CSP et same-origin apres build. Pas de JS, pas de reseau. Attention separee : le src de l'<img> masquee doit etre local/data:/blob: ; une URL http externe serait bloquee par la CSP.
  - ⚠️ mask-image OK car data: inline (jamais url http distante dans le .css)
  - ⚠️ Le contenu masque (src de l'img) doit etre 'self'/data:/blob: ; une image http distante serait bloquee par default-src
  - ⚠️ COEP require-corp : une image cross-origin sans CORP serait bloquee — utiliser des assets locaux

```html
<img class="mask mask-squircle" src="{image-url}" />
```

---

## `menu` — menu

Liste de liens/boutons verticale ou horizontale. Items en grille (grid-auto-flow:column) avec etats hover/focus/active coloriees via color-mix et var(--menu-active-bg/fg). Sous-menus via <details>/<summary> (CSS pur, transition de ::details-content + interpolate-size) OU via menu-dropdown/menu-dropdown-toggle (necessite JS pour ajouter menu-dropdown-show). Fleches de submenu dessinees en box-shadow inset + rotate.

- **Sous-parties** : `menu-title`, `menu-dropdown`, `menu-dropdown-toggle`
- **Modifiers** : `menu-disabled`, `menu-active`, `menu-focus`, `menu-dropdown-show`, `menu-xs`, `menu-sm`, `menu-md`, `menu-lg`, `menu-xl`, `menu-vertical`, `menu-horizontal`, `sm:menu-horizontal`, `lg:menu-horizontal`
- **Mécanismes CSS** : display:flex / flex-flow:column wrap (menu-horizontal -> row inline-flex) · display:grid sur les items, grid-auto-flow:column · color-mix(in oklab, var(--color-base-content) 10%, transparent) pour hover/focus · var(--menu-active-bg/fg), var(--radius-field), var(--border) · <details>::details-content transition (block-size 0 -> auto) + interpolate-size:allow-keywords · @starting-style + transition-behavior:allow-discrete pour l'apparition du sous-menu horizontal · box-shadow inset pour les fleches (pseudo :after) + rotate/translate · background-image:var(--fx-noise) sur l'etat actif · @keyframes menu (opacity) · forced-colors / prefers-reduced-motion media queries
- **CSP SBFB** : ✅ usable — Structure et styles 100% CSS. Les sous-menus <details>/<summary> sont collapsibles SANS JS (CSS natif). Pas d'url() distante, pas de mask-image, pas de form, pas de fetch.
  - ⚠️ Le mode dropdown via menu-dropdown/menu-dropdown-toggle suppose du JS pour basculer la classe menu-dropdown-show : structure CSS OK, comportement a coder localement (toggle de classe) sans reseau
  - ⚠️ Si on veut des sous-menus, preferer <details> (zero JS) plutot que menu-dropdown
  - ⚠️ var(--fx-noise) est une custom prop locale du theme (pas une url distante)

```html
<ul class="menu bg-base-200 rounded-box w-56">
  <li><a>Item 1</a></li>
  <li><a>Item 2</a></li>
  <li><a>Item 3</a></li>
</ul>
```

---

## `mockup-browser` — mockup-browser

Cadre decoratif imitant une fenetre de navigateur : 3 pastilles (box-shadow sur :before) + une barre d'outils mockup-browser-toolbar pouvant contenir un faux champ d'URL (.input) prefixe d'une icone loupe (mask SVG inline). Purement visuel.

- **Sous-parties** : `mockup-browser-toolbar`
- **Mécanismes CSS** : pseudo :before avec box-shadow:1.4em 0,2.8em 0,4.2em 0 (les 3 pastilles) · border-radius:var(--radius-box) · overflow:auto hidden · mask:url("data:image/svg+xml,...") sur .input:before (icone loupe, SVG INLINE) · background-color:currentColor pour peindre l'icone du mask · variante :dir(rtl) flex-direction:row-reverse · var(--color-base-200) pour le fond du faux input
- **CSP SBFB** : ✅ usable — Decor CSS pur. L'icone loupe du champ URL est un mask data: inline (OK). Pas de reseau, pas de JS, pas de form. Le texte d'URL est purement decoratif (un div, pas un vrai lien).
  - ⚠️ mask sur .input:before = data: inline, conforme
  - ⚠️ Aucune url() http distante

```html
<div class="mockup-browser border border-base-300 w-full">
  <div class="mockup-browser-toolbar">
    <div class="input">https://daisyui.com</div>
  </div>
  <div class="grid place-content-center border-t border-base-300 h-80">Hello!</div>
</div>
```

---

## `mockup-code` — mockup-code

Bloc imitant un editeur/terminal de code : fond neutral, 3 pastilles (box-shadow sur :before), et chaque <pre data-prefix> affiche un prefixe (ex '$', '1') via content:attr(data-prefix) dans :before. La coloration syntaxique du <code> n'est PAS fournie (requiert une lib).

- **Mécanismes CSS** : pseudo :before box-shadow (3 pastilles) · content:var(--tw-content) avec --tw-content:attr(data-prefix) · var(--color-neutral) / var(--color-neutral-content) · var(--radius-box) · overflow:auto hidden · direction:ltr
- **CSP SBFB** : ✅ usable — Pur CSS + attributs data-prefix. Aucune url(), aucun mask, aucun JS, aucun reseau. La coloration syntaxique optionnelle devrait etre faite par une lib LOCALE bundlee (pas de CDN).
  - ⚠️ La coloration syntaxique du <code> n'est pas incluse : si voulue, bundler la lib localement (jamais un CDN)
  - ⚠️ Aucun risque CSP intrinseque

```html
<div class="mockup-code">
  <pre data-prefix="$"><code>npm i daisyui</code></pre>
</div>
```

---

## `mockup-phone` — mockup-phone

Mockup d'un telephone (iPhone) : cadre noir avec bordure grise et coins tres arrondis (corner-shape superellipse si supporte), une encoche/camera (mockup-phone-camera) et une zone d'affichage (mockup-phone-display) qui recadre son contenu (overflow:hidden) ; une <img> enfant fait object-fit:cover.

- **Sous-parties** : `mockup-phone-camera`, `mockup-phone-display`
- **Mécanismes CSS** : display:inline-grid / grid-area:1/1/1/1 (camera et display superposes) · aspect-ratio:462/978 · border:5px solid #6b6b6b + border-radius:65px · @supports (corner-shape:superellipse(...)) progressive enhancement · overflow:hidden + object-fit:cover sur l'img du display · background:#000
- **CSP SBFB** : ✅ usable — Cadre CSS pur (grid, aspect-ratio, border-radius). Aucune url() distante, aucun mask, aucun JS. Le contenu de mockup-phone-display est libre — appliquer les regles CSP sur ce contenu (img locale/data:/blob:).
  - ⚠️ corner-shape:superellipse n'est applique que sous @supports (degradation gracieuse en border-radius classique sinon) — non bloquant
  - ⚠️ Si une <img> distante est mise dans le display, elle suit la CSP (default-src 'self' data: blob:) + COEP : utiliser des assets locaux

```html
<div class="mockup-phone">
  <div class="mockup-phone-camera"></div>
  <div class="mockup-phone-display">{CONTENT}</div>
</div>
```

---

## `mockup-window` — mockup-window

Cadre imitant une fenetre d'OS : barre superieure decorative avec 3 pastilles (box-shadow sur :before, aligne flex-start, flex-end en RTL) ; le contenu suit dessous. Supporte aussi <pre data-prefix> pour prefixer des lignes.

- **Mécanismes CSS** : display:flex / flex-direction:column · pseudo :before box-shadow:1.4em 0,2.8em 0,4.2em 0 (3 pastilles) · aspect-ratio:1 + border-radius:3.40282e38px (cercle) sur la pastille · var(--radius-box) · overflow:auto hidden · content:attr(data-prefix) pour pre[data-prefix] · variante [dir=rtl] align-self:flex-end
- **CSP SBFB** : ✅ usable — Decor CSS pur, aucune ressource distante, aucun mask, aucun JS, pas de form. Conforme.
  - ⚠️ Aucun risque CSP
  - ⚠️ Contenu interne libre — appliquer la CSP a ce qu'on y met

```html
<div class="mockup-window border border-base-300 w-full">
  <div class="grid place-content-center border-t border-base-300 h-80">Hello!</div>
</div>
```

---

## `modal` — modal

Boite de dialogue centree (grid place-items:center, position:fixed inset:0, z-index:999). Visible quand .modal-open / [open] / :target / .modal-toggle:checked+.modal. Le fond passe a oklch(0% 0 0/.4), la modal-box anime opacity/translate/scale. Trois modes d'ouverture : <dialog> natif (showModal() = JS), checkbox (CSS pur via :checked), ancre :target (CSS pur). Placements modal-top/bottom/start/end via translate.

- **Sous-parties** : `modal-box`, `modal-action`, `modal-backdrop`, `modal-toggle`
- **Modifiers** : `modal-open`, `modal-top`, `modal-middle`, `modal-bottom`, `modal-start`, `modal-end`
- **Mécanismes CSS** : position:fixed / display:grid / place-items:center / z-index:999 · transition visibility/background-color/opacity + @starting-style + transition-behavior:allow-discrete · selecteurs d'etat .modal-open / [open] / :target / .modal-toggle:checked+.modal · translate + scale pour l'animation de la modal-box et les placements · custom props --modal-tl/tr/bl/br pour les rayons selon placement · animation:set-page-has-scroll scroll() + :root:has(&) (scroll-driven animation) pour figer le scroll de page · box-shadow sur modal-box · background-color:oklch(0% 0 0/.4) (overlay simple, PAS backdrop-filter)
- **CSP SBFB** : ✅ usable — L'overlay/backdrop est une simple background-color (PAS un backdrop-filter), donc zero souci composite. Les variantes checkbox (modal-toggle:checked) et ancre (:target) sont 100% CSS, sans JS. Aucune url() distante, aucun mask.
  - ⚠️ Le pattern <dialog> + showModal() repose sur JS (onclick) ; en iframe scellee le JS est autorise (unsafe-inline/unsafe-eval) mais c'est du comportement a coder localement — sinon preferer le mode checkbox/:target (CSS pur)
  - ⚠️ <form method="dialog"> pour fermer : sous sandbox allow-scripts sans allow-forms la soumission de formulaire est bloquee, et form-action 'none' bloque tout POST reseau -> preferer <button type="button"> + handler JS close(), ou les modes checkbox/:target
  - ⚠️ scroll() animation timeline (scroll-driven) : progressive, non bloquant

```html
<button onclick="my_modal.showModal()">Open modal</button>
<dialog id="my_modal" class="modal">
  <div class="modal-box">{CONTENT}</div>
  <form method="dialog" class="modal-backdrop"><button>close</button></form>
</dialog>
```

---

## `navbar` — navbar

Barre de navigation horizontale (flex, width:100%, min-height:4rem, padding .5rem, position:relative). navbar-start (50%, debut), navbar-center (centre, flex-shrink:0), navbar-end (50%, fin) positionnent le contenu.

- **Sous-parties** : `navbar-start`, `navbar-center`, `navbar-end`
- **Modifiers** : `sm:navbar`, `md:navbar`, `lg:navbar`, `xl:navbar`, `2xl:navbar`
- **Mécanismes CSS** : display:flex / align-items:center · width:50% sur navbar-start et navbar-end (display:inline-flex) · justify-content:flex-start/flex-end · flex-shrink:0 sur navbar-center · min-height:4rem · position:relative
- **CSP SBFB** : ✅ usable — Pur layout flex. Aucune ressource distante, aucun mask, aucun JS, pas de form. Le fond (bg-base-100/200) est un utilitaire Tailwind local.
  - ⚠️ Aucun risque CSP
  - ⚠️ Un menu deroulant mobile (dropdown) ajoute dans la navbar peut supposer du JS ou utiliser le composant dropdown CSS — a evaluer separement

```html
<div class="navbar bg-base-100 shadow-sm">
  <a class="btn btn-ghost text-xl">daisyUI</a>
</div>
```

---

## `progress` — progress

Barre de progression basee sur l'element natif <progress> (appearance:none). Le remplissage est colore via currentColor (::-webkit-progress-value / ::-moz-progress-bar), la piste via color-mix de currentColor 20%. Sans attribut value -> etat :indeterminate avec gradient anime (repeating-linear-gradient + keyframes progress). Couleurs via les modifiers (color:var(--color-*)).

- **Modifiers** : `progress-neutral`, `progress-primary`, `progress-secondary`, `progress-accent`, `progress-info`, `progress-success`, `progress-warning`, `progress-error`
- **Mécanismes CSS** : appearance:none sur <progress> · ::-webkit-progress-bar / ::-webkit-progress-value / ::-moz-progress-bar pseudo-elements · color-mix(in oklab, currentcolor 20%, transparent) pour la piste · background-color:currentColor pour la valeur · :indeterminate + repeating-linear-gradient + @keyframes progress · @supports (-moz-appearance) / (-webkit-appearance) · var(--radius-box) · prefers-reduced-motion media query · color:var(--color-primary\|...) selon le modifier
- **CSP SBFB** : ✅ usable — Pur CSS sur element <progress> natif. L'animation indeterminee est un gradient CSS + keyframes, aucune ressource distante, aucun mask, aucun JS. La doc demande value/max comme attributs HTML statiques.
  - ⚠️ Aucun risque CSP
  - ⚠️ Pour faire AVANCER la barre dynamiquement il faut changer l'attribut value en JS local (comportement a coder localement, sans reseau) ; l'affichage statique et l'animation indeterminee sont 100% CSS

```html
<progress class="progress w-56" value="0" max="100"></progress>
```

---

## `radial-progress` — radial-progress

Indicateur circulaire de progression. La valeur est passee par la custom prop --value (0-100) en style inline. L'anneau est dessine par un conic-gradient (currentColor jusqu'a --radialprogress) combine a un radial-gradient, masque en anneau via mask radial-gradient. Un :after rond marque la tete de la jauge (rotate + translate). --size et --thickness configurables.

- **Mécanismes CSS** : conic-gradient(currentColor var(--radialprogress), #0000 0) · radial-gradient(farthest-side, currentColor 98%, #0000) pour la tete · mask / -webkit-mask: radial-gradient(...) pour creuser l'anneau (mask GENERE par gradient, pas une url) · pseudo :after avec transform:rotate(calc(var(--value)*3.6deg - 90deg)) translate(...) · custom props --value, --size, --thickness, --radialprogress · transition de --radialprogress (typed/registered) + transition transform · display:inline-grid / place-content:center · border-radius:3.40282e38px (cercle)
- **CSP SBFB** : ✅ usable — Le mask est un radial-gradient GENERE par CSS (pas une url() ni un SVG distant), donc conforme. Tout est custom props + gradients + transform. Aucun reseau, aucun JS pour l'affichage statique.
  - ⚠️ mask: radial-gradient(...) present mais c'est un mask par gradient (aucune url distante) -> OK composite local
  - ⚠️ La transition fluide de --radialprogress suppose @property/registered custom property (progressive) ; non bloquant
  - ⚠️ Pour animer la valeur dynamiquement, modifier --value en JS local (comportement a coder localement, sans reseau)

```html
<div class="radial-progress" style="--value:70;" aria-valuenow="70" role="progressbar">70%</div>
```

---

## `radio` — radio

Bouton radio stylise (input[type=radio], appearance:none) : cercle borde, etat :checked/[aria-checked=true] remplit le centre via :before background-color:currentColor + box-shadow inset (anneau interieur) et une petite animation de padding (@keyframes radio). Couleurs via --input-color, tailles via --size.

- **Modifiers** : `radio-neutral`, `radio-primary`, `radio-secondary`, `radio-accent`, `radio-success`, `radio-warning`, `radio-info`, `radio-error`, `radio-xs`, `radio-sm`, `radio-md`, `radio-lg`, `radio-xl`
- **Mécanismes CSS** : appearance:none · border:var(--border) solid var(--input-color, color-mix(in srgb, currentColor 20%, #0000)) · :checked / [aria-checked=true] -> :before background-color:currentColor + box-shadow inset (point central) · @keyframes radio (animation de padding au check) + prefers-reduced-motion · --input-color (modifiers couleur) / --size (modifiers taille, multiples de --size-selector) · background-image:var(--fx-noise) sur :before · border-radius:3.40282e38px (cercle) · box-shadow inset (profondeur var(--depth)) · forced-colors / print media queries
- **CSP SBFB** : ✅ usable — Pur CSS sur input natif. Aucune url() distante, aucun mask-image, aucun JS requis (le groupement par name= et la selection sont natifs au navigateur). var(--fx-noise) est une custom prop locale du theme.
  - ⚠️ Aucun risque CSP
  - ⚠️ Etant un input radio natif, son groupement par 'name' fonctionne sans reseau ; aucun <form> requis (et une soumission de form serait bloquee par form-action 'none' + sandbox)

```html
<input type="radio" name="radio-1" class="radio" checked="checked" />
```

---

## `range` — range

Style un <input type=range> natif : piste, thumb circulaire et remplissage de progression. La couleur du thumb est pilotee par --range-thumb (couleur-content) et la couleur de progression par currentColor (color via les modifiers range-*). La taille du thumb par --range-thumb-size.

- **Modifiers** : `range-neutral`, `range-primary`, `range-secondary`, `range-accent`, `range-success`, `range-warning`, `range-info`, `range-error`, `range-xs`, `range-sm`, `range-md`, `range-lg`, `range-xl`
- **Mécanismes CSS** : -webkit-appearance:none / appearance:none · ::-webkit-slider-runnable-track + ::-webkit-slider-thumb + ::-moz-range-track + ::-moz-range-thumb · box-shadow multi-couches (inset + huge spread 0 0 0 2rem / 100cqw) pour peindre le fill de progression · container query units cqw pour le remplissage · color-mix(in oklab,...) pour --range-bg · oklch() avec var(--depth) pour ombrages · border-radius via var(--radius-selector) · [dir=rtl] --range-dir:-1 · @media forced-colors:active fallback border
- **CSP SBFB** : ✅ usable — 100% CSS pur sur un input natif : aucun url() distant, aucun mask-image, aucun backdrop-filter, aucune @font-face. Le remplissage est fait via box-shadow/cqw, donc compile dans le app.css vendore same-origin. L'interactivite (drag du thumb) est native HTML, sans reseau. Lire la valeur cote app se fait via JS local (oninput) sans fetch.
  - ⚠️ Les container query units (cqw) supposent un contexte de container ; rendu correct mais visuellement sensible a la largeur du parent.
  - ⚠️ Pour reagir a la valeur il faut un handler JS local (input.value) ; aucun reseau requis, mais ce n'est pas du pur CSS.

```html
<input type="range" min="0" max="100" value="40" class="range {MODIFIER}" />
```

---

## `rating` — rating

Groupe de radios (ou inputs) representant une note. Chaque enfant est peint en aplat via background-color:var(--color-base-content) avec opacity .2, passant a opacity 1 quand il est :checked / [aria-checked=true] / [aria-current=true] OU s'il a un voisin coche (selecteur :has(~:checked)). Donne l'effet etoiles remplies jusqu'a la selection. L'exemple officiel forme les etoiles via les utilitaires Tailwind mask mask-star.

- **Sous-parties** : `rating-hidden`
- **Modifiers** : `rating-half`, `rating-xs`, `rating-sm`, `rating-md`, `rating-lg`, `rating-xl`
- **Mécanismes CSS** : background-color:var(--color-base-content) + opacity 0.2->1 pour l'etat rempli · selecteur :has(~:checked,~[aria-checked=true],~[aria-current=true]) pour remplir les precedents · input{appearance:none;border:none} · @keyframes rating (filter:brightness/contrast + scale) sous prefers-reduced-motion:no-preference · scale:1.1 sur :focus-visible et :active:focus · rating-hidden = background-color transparent + width .5rem (radio de reset) · rating-half reduit la largeur des enfants pour demi-etoiles
- **CSP SBFB** : ✅ usable — Le CSS daisyUI de .rating est pur (background-color/opacity/scale/keyframes), sans url() distant. L'etat est pilote par CSS :checked (radios natifs), donc fonctionne sans JS ni reseau. ATTENTION : l'exemple officiel utilise les utilitaires Tailwind mask + mask-star ; mask-star de daisyUI/Tailwind v4 emet un mask-image en data: URI (forme d'etoile inline), same-origin/data: => autorise par la CSP (data: present dans default-src). Aucune URL http distante.
  - ⚠️ mask-image via mask-star : verifier au build que Tailwind genere bien un data: URI (cas standard daisyUI) et non une reference a un fichier externe ; data: est autorise par la CSP, une URL http ne le serait pas.
  - ⚠️ Sans mask, les enfants sont de simples carres colores : il faut fournir une forme (mask, clip-path local, ou SVG inline peint via background-color/currentColor) pour l'aspect etoile.
  - ⚠️ Lire/soumettre la note (radios) hors d'un <form> : pas de submit (form-action 'none') ; lire input:checked en JS local.

```html
<div class="rating {MODIFIER}">
  <input type="radio" name="rating-1" class="mask mask-star" />
</div>
```

---

## `select` — select

Style un <select> natif (ou wrapper avec select interne) : hauteur via --size, bordure/ombre via --input-color, et la fleche de dropdown dessinee en CSS pur via deux linear-gradient. Gere [multiple], :disabled, focus (outline 2px). Supporte le futur appearance:base-select + ::picker(select) pour styler la liste deroulante customisable.

- **Modifiers** : `select-ghost`, `select-neutral`, `select-primary`, `select-secondary`, `select-accent`, `select-info`, `select-success`, `select-warning`, `select-error`, `select-xs`, `select-sm`, `select-md`, `select-lg`, `select-xl`
- **Mécanismes CSS** : appearance:none + fleche via background-image:linear-gradient(45deg,...) ,linear-gradient(135deg,...) (PAS d'url, PAS de SVG distant) · background-position/size calcules pour placer la fleche · color-mix(in oklab,...) pour --input-color et etats disabled · box-shadow inset avec var(--depth) · border-*-radius via var(--join-ss/se/ee/es,var(--radius-field)) (integration join) · @supports(appearance:base-select) + ::picker(select) + ::picker-icon{display:none} (customizable select moderne) · transitions sur option hover/active · [dir=rtl] repositionne la fleche
- **CSP SBFB** : ✅ usable — Element <select> natif entierement stylable en CSS pur : la fleche est faite avec des linear-gradient (aucune url() ni SVG distant), color-mix et box-shadow compilent dans le app.css vendore same-origin. Pas de mask-image, pas de backdrop-filter, pas de font distante. Le dropdown natif et la selection fonctionnent sans JS ni reseau.
  - ⚠️ Le bloc ::picker(select)/appearance:base-select est gate par @supports : sur navigateurs sans support, fallback au select natif (aucun probleme CSP, juste cosmetique).
  - ⚠️ Pour reagir au changement de valeur cote app : handler JS local (onchange) ; aucun reseau requis. Ne pas envelopper dans un <form> qui submit (form-action 'none').

```html
<select class="select {MODIFIER}">
  <option>Option</option>
</select>
```

---

## `skeleton` — skeleton

Bloc placeholder de chargement : fond base-300 + un gradient diagonal anime (effet shimmer) qui balaie de gauche a droite en boucle. skeleton-text applique le meme shimmer sur du texte via background-clip:text (le texte devient le masque du gradient).

- **Modifiers** : `skeleton-text`
- **Mécanismes CSS** : background-image:linear-gradient(105deg,...) anime · @keyframes skeleton (background-position 150% -> -50%) avec animation 1.8s infinite · will-change:background-position · background-size:200% · @media prefers-reduced-motion:reduce -> transition-duration:15s (ralenti) ; no-preference -> animation active · skeleton-text : color:transparent + -webkit-background-clip:text/background-clip:text + gradient color-mix(base-content) · border-radius via var(--radius-box)
- **CSP SBFB** : ✅ usable — 100% CSS pur : linear-gradient anime, background-clip:text, keyframes. Aucune url() distante, aucun mask-image, aucun font distant. Compile entierement dans le app.css vendore. Pas de JS ni reseau (la dimension se regle avec les utilitaires h-*/w-* Tailwind, eux aussi locaux).
  - ⚠️ Aucun risque CSP. Le shimmer respecte prefers-reduced-motion (ralenti plutot que coupe). Penser a fixer h-*/w-* sinon le bloc n'a pas de dimension visible.

```html
<div class="skeleton"></div>
```

---

## `stack` — stack

Empile visuellement les enfants les uns sur les autres dans une meme cellule de grille, avec un leger decalage/echelle et opacite degressive (premier enfant net z-index 3, suivants opacity .9/.7) pour un effet de pile de cartes. Les modifiers controlent vers quel bord la pile se decale.

- **Modifiers** : `stack-top`, `stack-bottom`, `stack-start`, `stack-end`
- **Mécanismes CSS** : display:inline-grid avec grid-template-rows/columns 3px 4px 1fr 4px 3px · grid-area superposees par nth-child / first-child pour decaler chaque couche · z-index + opacity degressive (1er=3 net, 2e=.9, suivants=.7) · stack-top/bottom/start/end reassignent les grid-area pour changer la direction du decalage
- **CSP SBFB** : ✅ usable — Pur layout CSS Grid (grid-area, z-index, opacity). Aucune url(), aucun mask, aucun backdrop-filter, aucun font, aucun JS. Compile trivialement dans le app.css vendore et fonctionne sans reseau.
  - ⚠️ Aucun risque CSP. Pour que les couches aient la meme taille, fixer w-*/h-* sur le .stack comme indique par la doc.

```html
<div class="stack {MODIFIER}">{CONTENT}</div>
```

---

## `stats` — stat

Conteneur (stats) en grille qui aligne des blocs .stat separes par une bordure pointillee. Chaque stat contient un titre (stat-title, attenue), une grande valeur (stat-value, 2rem bold 800), une description (stat-desc), une figure/icone (stat-figure, alignee a droite sur 3 rangs) et des actions (stat-actions). Horizontal par defaut, vertical via stats-vertical.

- **Sous-parties** : `stat`, `stat-title`, `stat-value`, `stat-desc`, `stat-figure`, `stat-actions`
- **Modifiers** : `stats-horizontal`, `stats-vertical`
- **Mécanismes CSS** : display:inline-grid + grid-auto-flow:column (ou row pour vertical) · overflow-x:auto / overflow-y:auto · border-inline-end / border-block-end dashed via var(--border) sur :not(:last-child) (separateurs) · color-mix(in oklab,var(--color-base-content) 60%,transparent) pour title/desc attenues · grid-template-columns/grid-row span pour positionner figure et textes · border-radius via var(--radius-box)
- **CSP SBFB** : ✅ usable — Pur CSS Grid + color-mix + bordures. Aucune url() distante, aucun mask-image, aucun backdrop-filter, aucun font distant, aucun JS. La structure est statique et compile dans le app.css vendore same-origin.
  - ⚠️ Aucun risque CSP propre au composant. Une icone placee dans stat-figure via un SVG inline doit etre peinte par CSS (currentColor/var(--color-*)) car les utilitaires Tailwind fill-*/stroke-* ne sont pas garantis de compiler dans l'iframe scellee.

```html
<div class="stats {MODIFIER}">
  <div class="stat">{CONTENT}</div>
</div>
```

---

## `status` — status

Petit pastille/point d'etat (inline-block, aspect-ratio 1, ~.5rem) montrant un statut (online/offline/erreur). Couleur via les modifiers status-*, leger reflet via un radial-gradient en haut-gauche et une ombre portee. Ne rend aucun contenu textuel.

- **Modifiers** : `status-primary`, `status-secondary`, `status-accent`, `status-neutral`, `status-info`, `status-success`, `status-warning`, `status-error`, `status-xs`, `status-sm`, `status-md`, `status-lg`, `status-xl`
- **Mécanismes CSS** : aspect-ratio:1 + width/height fixes (tailles xs..xl) · background-image:radial-gradient(circle at 35% 30%,oklch(1 0 0/calc(var(--depth)*.5)),#0000) pour le reflet (PAS d'url) · background-color via color-mix / var(--color-*) par modifier · box-shadow color-mix avec var(--depth) · border-radius via var(--radius-selector) · display:inline-block
- **CSP SBFB** : ✅ usable — 100% CSS pur : le reflet est un radial-gradient genere en CSS (aucune url() distante), couleurs via color-mix/oklch, ombre via box-shadow. Aucun mask, aucun font, aucun JS. Compile dans le app.css vendore.
  - ⚠️ Aucun risque CSP. Pour un point clignotant (ex. live), ajouter une animation locale ou l'utilitaire animate-* Tailwind (local) ; rien de reseau.

```html
<span class="status {MODIFIER}"></span>
```

---

## `steps` — steps

Affiche une progression a etapes (<ul class=steps>/<li class=step>). Chaque step montre un cercle numerote auto (compteur CSS) relie par une barre. Les modifiers step-* colorent l'etape ET la barre la reliant a l'etape suivante de meme couleur, indiquant la progression. step-icon remplace le numero par une icone ; data-content sur le <li> remplace le numero par une chaine arbitraire.

- **Sous-parties** : `step`, `step-icon`
- **Modifiers** : `step-neutral`, `step-primary`, `step-secondary`, `step-accent`, `step-info`, `step-success`, `step-warning`, `step-error`, `steps-vertical`, `steps-horizontal`
- **Mécanismes CSS** : counter-reset:step + counter-increment + content:counter(step) pour la numerotation auto · content:var(--tw-content) via ::after (numero) ou attr(data-content) si [data-content] · ::before = barre de liaison (background-color:var(--step-bg), margin-inline-start:-100%) · selecteur d'adjacence .step-primary + .step-primary:before pour colorer la barre entre deux etapes de meme couleur · border-radius:3.40282e38px (cercle, valeur ~FLT_MAX) · display:grid + grid-template-rows/columns (horizontal vs vertical), overflow auto hidden · variables --step-bg/--step-fg pilotees par les modifiers couleur
- **CSP SBFB** : ✅ usable — Pur CSS : compteurs, content/attr(), pseudo-elements, grid, color-mix/var. Aucune url() distante, aucun mask-image, aucun backdrop-filter, aucun font distant, aucun JS. Compile dans le app.css vendore et fonctionne sans reseau. data-content lit un attribut HTML local.
  - ⚠️ Aucun risque CSP. Si on met une icone via step-icon, fournir un SVG inline peint en currentColor/var(--step-fg) (pas de fill-*/stroke-* Tailwind ni d'image distante). L'etat actif est purement declaratif (ajout des classes step-* en HTML ou via JS local).

```html
<ul class="steps {MODIFIER}">
  <li class="step">{step content}</li>
</ul>
```

---

## `swap` — swap

Bascule la visibilite de deux (ou trois) elements empiles. Pilote soit par une checkbox cachee interne (:checked/:indeterminate fait apparaitre swap-on/swap-indeterminate et masque swap-off), soit par la classe swap-active ajoutee en JS. swap-rotate ajoute une rotation 45deg/-45deg, swap-flip un retournement 3D (rotateY 180deg avec perspective).

- **Sous-parties** : `swap-on`, `swap-off`, `swap-indeterminate`
- **Modifiers** : `swap-active`, `swap-rotate`, `swap-flip`
- **Mécanismes CSS** : display:inline-grid avec enfants en grid-row/column-start:1 (superposes) · input{appearance:none} cache le controle natif · opacity 0<->1 pilotee par input:checked~.swap-on / input:indeterminate~.swap-indeterminate · swap-active : classe alternative (sans checkbox) pour forcer l'etat · swap-rotate : rotate:45deg / -45deg sur on/off · swap-flip : transform-style:preserve-3d + perspective:20rem + rotateY(180deg) + backface-visibility:hidden · transitions transform/rotate/opacity sous prefers-reduced-motion:no-preference
- **CSP SBFB** : ✅ usable — 100% CSS pur : grid, transforms 3D, opacity, etats via checkbox native :checked/:indeterminate. Aucune url() distante, aucun mask, aucun backdrop-filter, aucun font distant. Le mode <label>+checkbox fonctionne sans JS ni reseau. Compile dans le app.css vendore.
  - ⚠️ Le mode swap-active (sans checkbox) suppose un toggle de classe en JS local (input non utilise) : structure CSS OK, comportement a coder localement sans reseau.
  - ⚠️ :indeterminate ne peut etre mis qu'en JS (input.indeterminate=true) ; sans cela swap-indeterminate ne s'affiche jamais. Aucun reseau requis dans les deux cas.

```html
<label class="swap {MODIFIER}">
  <input type="checkbox" />
  <div class="swap-on">{content when active}</div>
  <div class="swap-off">{content when inactive}</div>
</label>
```

---

## `tabs` — tab

Barre d'onglets (role=tablist) contenant des .tab (boutons, radios input[type=radio] avec aria-label, ou labels). L'onglet selectionne (:checked / label:has(:checked) / .tab-active / [aria-selected=true] / [aria-current]) affiche son .tab-content adjacent (display:block). Styles : tabs-box (fond base-200, onglet actif sureleve), tabs-border (soulignement 3px), tabs-lift (onglets en languettes avec coins arrondis dessines par gradients), placements tabs-top/tabs-bottom.

- **Sous-parties** : `tab`, `tab-content`
- **Modifiers** : `tabs-box`, `tabs-border`, `tabs-lift`, `tab-active`, `tab-disabled`, `tabs-top`, `tabs-bottom`, `tabs-xs`, `tabs-sm`, `tabs-md`, `tabs-lg`, `tabs-xl`
- **Mécanismes CSS** : display:flex + flex-direction var(--tabs-direction) · input[type=radio].tab:after{content:attr(aria-label)} pour afficher le libelle du radio · selecteur d'etat :checked / label:has(:checked) / [aria-selected=true] / [aria-current] + & + .tab-content{display:block} pour reveler le contenu · tabs-lift : coins arrondis dessines via radial-gradient (var(--radius-start/end)) en pseudo-element ::before (PAS d'url) · tabs-border : ::before barre 3px + border-top 3px sur l'actif · tabs-box : background base-200 + box-shadow inset/depth sur l'actif · input{appearance:none;opacity:0} pour radios/labels caches · nombreuses variables --tab-* / --tab-radius-grad / --tabcontent-radius-* ; outline focus-visible
- **CSP SBFB** : ✅ usable — Pur CSS : flex, radios/labels natifs avec selecteurs :checked/:has, content:attr(aria-label), radial-gradient pour les coins (aucune url() distante), color-mix/box-shadow. Aucun mask-image, aucun backdrop-filter, aucun font distant. Le mecanisme radio (name partage) bascule le contenu SANS JS ni reseau. Compile dans le app.css vendore.
  - ⚠️ La variante boutons (role=tab) sans radios ne bascule pas le contenu en CSS seul (la doc precise que les radio inputs sont necessaires) : pour des onglets a base de <button>, gerer .tab-active + affichage du contenu en JS local, sans reseau.
  - ⚠️ Aucun url()/font distant. :has() requis pour le mode label:has(:checked) (largement supporte).

```html
<div role="tablist" class="tabs tabs-box">
  <input type="radio" name="my_tabs" class="tab" aria-label="Tab" />
</div>
```

---

## `table` — table

Style une <table> HTML native : largeur 100%, border-collapse:separate, padding cellules, separateurs de lignes, en-tetes thead/tfoot attenues. table-zebra alterne le fond des lignes paires (:nth-child(2n)). table-pin-rows/cols rend thead/tfoot/th sticky. row-hover surligne au survol. Tailles xs..xl modulent font-size et padding.

- **Sous-parties** : `thead`, `tbody`, `tfoot`, `tr`, `th`, `td`
- **Modifiers** : `table-zebra`, `table-pin-rows`, `table-pin-cols`, `table-xs`, `table-sm`, `table-md`, `table-lg`, `table-xl`, `row-hover`, `sm:table`, `md:table`, `lg:table`, `xl:table`, `2xl:table`
- **Mécanismes CSS** : border-collapse:separate + border-spacing via --tw-border-spacing-x/y · border-radius:var(--radius-box) · color-mix(in oklab/oklch, var(--color-base-content) X%, transparent) pour bordures et thead attenue · position:sticky (top/bottom/left) pour table-pin-rows/cols · background-color:var(--color-base-200/300) sur :nth-child(2n) pour zebra · @media (hover:hover) pour row-hover · var(--border) solid pour separateurs · logical props :dir(rtl) text-align:right
- **CSP SBFB** : ✅ usable — 100% CSS pur sur balises HTML natives (table/thead/tbody/tr/th/td). Aucune url() distante, aucun mask, aucun JS requis. color-mix/oklch/sticky/border-spacing sont du CSS moderne compose offline. overflow-x-auto est un utilitaire Tailwind compile build-time. Rendu identique dans l'iframe scellee.
  - ⚠️ Aucun risque CSP
  - ⚠️ Le surlignage row-hover depend de @media (hover:hover) : inerte sur ecrans tactiles (comportement CSS natif, pas un bug)
  - ⚠️ Le tri/pagination des donnees, si voulu, est a coder en JS local sans reseau (la table daisyUI est purement presentationnelle)

```html
<div class="overflow-x-auto">
  <table class="table {MODIFIER}">
    <thead>
      <tr>
        <th></th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <th></th>
      </tr>
    </tbody>
  </table>
</div>
```

---

## `textarea` — textarea

Style un champ <textarea> multi-ligne : bordure via var(--input-color), radius-field, fond base-100, min-height 5rem, largeur clamp(3rem,20rem,100%), ombre interne pilotee par --depth. Focus passe --input-color a base-content + outline 2px. Les modifiers couleur reaffectent --input-color ; ghost retire fond/ombre/bordure. Gere etat disabled (fond base-200, curseur not-allowed, placeholder attenue).

- **Sous-parties** : `textarea (element <textarea> direct ou wrapper avec >textarea enfant)`
- **Modifiers** : `textarea-ghost`, `textarea-neutral`, `textarea-primary`, `textarea-secondary`, `textarea-accent`, `textarea-info`, `textarea-success`, `textarea-warning`, `textarea-error`, `textarea-xs`, `textarea-sm`, `textarea-md`, `textarea-lg`, `textarea-xl`, `sm:textarea`, `md:textarea`, `lg:textarea`, `xl:textarea`, `2xl:textarea`
- **Mécanismes CSS** : var(--input-color) pilote la couleur de bordure/focus · color-mix(in oklab, ...) pour bordure semi-transparente et placeholder disabled · box-shadow inset pilote par calc(var(--depth)*X%) · width:clamp(3rem,20rem,100%) · font-size:max(var(--font-size,...),...) (anti-zoom mobile) · &:has(>textarea[disabled]) selecteur :has() · @media (pointer:coarse)+@supports(-webkit-touch-callout) bump font a 1rem · outline 2px var(--input-color) au focus, isolation:isolate · appearance:none
- **CSP SBFB** : ✅ usable — CSS pur sur element form natif. Aucune url() distante, aucun mask, aucun backdrop-filter. Le composant n'envoie rien par lui-meme : la saisie reste locale. clamp/color-mix/:has()/@supports sont du CSS compose offline.
  - ⚠️ Aucun risque CSP intrinseque
  - ⚠️ Ne JAMAIS encapsuler dans un <form> avec submit reseau : form-action 'none' + sandbox bloque le submit. Lire la valeur via JS local (textarea.value) et un button+handler
  - ⚠️ La soumission/persistance des donnees est a coder localement sans fetch reseau

```html
<textarea class="textarea {MODIFIER}" placeholder="Bio"></textarea>
```

---

## `text-rotate` — textrotate

Affiche jusqu'a 6 lignes de texte une a la fois en boucle infinie verticale (defilement). Detecte le nombre de lignes via :has(:nth-child(2..6)) et choisit la keyframe rotator adaptee. Duree 10s par defaut (--duration:var(--tw-duration)), surchargeable par duration-{ms}. L'animation se met en pause au survol (animation-play-state:paused).

- **Sous-parties** : `span/div enfant unique conteneur`, `spans/divs internes (2 a 6 = les lignes de texte)`
- **Modifiers** : `duration-{value} (utilitaire Tailwind, ms, ex duration-12000)`, `classes de texte Tailwind libres (text-7xl, font-title, leading-[2], couleurs bg-*/text-* sur chaque ligne)`
- **Mécanismes CSS** : @keyframes rotator + animation:rotator var(--duration) linear(...) infinite (easing par fonction linear() multi-stops) · --items compte les lignes via :has(:nth-child(N)) · translate:0 -100% et --first-item-position pour le defilement vertical · clip-path:inset(.5px 0) pour masquer les debordements · height:1lh / display:grid / overflow:hidden · animation-play-state:paused au :hover · transition-property:none
- **CSP SBFB** : ✅ usable — Animation 100% CSS (keyframes + :has()), aucune dependance JS, aucune url() distante. linear() easing, clip-path, translate sont du CSS pur compose offline. Fonctionne dans l'iframe scellee. unsafe-inline autorise les styles mais ici tout est dans app.css vendore.
  - ⚠️ Aucun risque CSP
  - ⚠️ L'animation est continue (infinite) : sera coupee sous prefers-reduced-motion uniquement si l'app le gere (le composant ne le fait PAS nativement) — a considerer pour l'accessibilite
  - ⚠️ clip-path et linear() easing : support navigateur recent requis (OK Chromium/WebKit modernes)

```html
<span class="text-rotate max-md:text-3xl text-7xl font-title">
  <span class="justify-items-center">
    <span>DESIGN</span>
    <span>DEVELOP</span>
    <span>DEPLOY</span>
    <span>SCALE</span>
    <span>MAINTAIN</span>
    <span>REPEAT</span>
  </span>
</span>
```

---

## `timeline` — timeline

Affiche une liste d'evenements chronologiques sur une <ul>/<li>. Chaque li est une grille 3x3 placant timeline-start / timeline-middle (l'icone) / timeline-end. Les <hr> entre items forment la ligne de connexion (fond base-300, epaisseur .25rem) avec coins arrondis adaptatifs selon presence de timeline-middle. timeline-vertical (defaut) empile verticalement, timeline-horizontal en ligne, timeline-compact force tout d'un cote, timeline-snap-icon colle l'icone au debut. timeline-box encadre le contenu (bordure + radius-box + ombre).

- **Sous-parties** : `timeline-start`, `timeline-middle`, `timeline-end`, `timeline-box`, `li (chaque evenement)`, `hr (connecteurs entre items)`
- **Modifiers** : `timeline-vertical`, `timeline-horizontal`, `timeline-compact`, `timeline-snap-icon`, `timeline-box`, `sm:timeline`, `md:timeline`, `lg:timeline`, `xl:timeline`, `2xl:timeline (+ variantes responsive de chaque part/modifier)`
- **Mécanismes CSS** : display:grid avec grid-template-rows/columns pilotees par --timeline-row-start/col-start (minmax(0,1fr) auto minmax(0,1fr)) · grid-area + place-self pour positionner start/middle/end · <hr> stylises (background-color:var(--color-base-300), height/width .25rem) comme connecteurs · border-radius logique (border-start-start-radius etc.) conditionnee par :has(.timeline-middle) · var(--radius-selector)/var(--radius-box)/var(--border) · @media print bordures sur hr · flex-direction column/row pour vertical/horizontal
- **CSP SBFB** : ✅ usable — Structure purement CSS (grid + hr + place-self). Aucune url() distante, aucun mask, aucun JS. Les icones dans timeline-middle sont fournies par l'auteur (souvent un <svg> inline ou caractere) — a peindre via currentColor/var(--color-*), pas via fill-* Tailwind. color-mix/oklch/logical-props composes offline.
  - ⚠️ Aucun risque CSP structurel
  - ⚠️ Si l'icone middle est un SVG : utiliser un <svg> inline avec fill="currentColor" ou var(--color-*) — les utilitaires Tailwind fill-*/stroke-* ne compilent PAS de maniere fiable dans l'iframe scellee
  - ⚠️ Pas d'image bitmap via url() distant pour les icones (interdit) : inliner en data: ou SVG

```html
<ul class="timeline {MODIFIER}">
  <li>
    <div class="timeline-start">{start}</div>
    <div class="timeline-middle">{icon}</div>
    <div class="timeline-end">{end}</div>
  </li>
</ul>
```

---

## `toast` — toast

Conteneur position:fixed qui empile ses enfants (flex column, gap .5rem) dans un coin de la fenetre. Par defaut en bas a droite (bottom:1rem; inset-inline:auto 1rem). Les modifiers placement modulent --toast-x/--toast-y et les ancres top/bottom/inset-inline pour positionner le bloc (start/center/end horizontalement, top/middle/bottom verticalement). Chaque enfant joue une courte animation d'apparition (keyframes toast : opacity+scale).

- **Sous-parties** : `enfants directs du toast (chaque alerte/notification empilee)`
- **Modifiers** : `toast-start`, `toast-center`, `toast-end`, `toast-top`, `toast-middle`, `toast-bottom`, `sm:toast`, `md:toast`, `lg:toast`, `xl:toast`, `2xl:toast (+ variantes responsive de chaque placement)`
- **Mécanismes CSS** : position:fixed + inset-inline / top / bottom pour l'ancrage · translate:var(--toast-x,0) var(--toast-y,0) pour centrage (-50%) · @keyframes toast (opacity 0->1, scale .9->1) · @media (prefers-reduced-motion:no-preference) gate l'animation · display:flex; flex-direction:column; gap · width:max-content; max-width:calc(100vw - 2rem) · background-color:#0000 (conteneur transparent)
- **CSP SBFB** : ✅ usable — CSS pur (position:fixed + translate + keyframes). Aucune url() distante, aucun mask, aucun backdrop-filter. L'animation respecte prefers-reduced-motion nativement. Compose offline dans l'iframe.
  - ⚠️ Aucun risque CSP
  - ⚠️ position:fixed est relatif au viewport de l'iframe (pas a la page hote) — comportement attendu et sain pour une app scellee
  - ⚠️ L'apparition/disparition programmatique des toasts (timer, dismiss) est a coder en JS LOCAL sans reseau (ajout/retrait de l'enfant dans le DOM) ; le CSS ne gere que l'animation d'entree

```html
<div class="toast {MODIFIER}">{CONTENT}</div>
```

---

## `toggle` — toggle

Transforme un <input type=checkbox> en interrupteur. Utilise display:inline-grid 3 colonnes (0fr 1fr 1fr) qui basculent en (1fr 1fr 0fr) a l'etat :checked, ce qui fait glisser le thumb ::before de gauche a droite via transition de grid-template-columns. Couleur pilotee par --input-color (modifiers couleur sur :checked). Tailles via --size (multiple de --size-selector). Gere :indeterminate (.5fr 1fr .5fr), :disabled (opacity .3), focus-visible. Les enfants nth-child(2)/(3) permettent des icones on/off avec rotation/opacity.

- **Sous-parties** : `pseudo-element ::before (le bouton coulissant/thumb)`, `enfants optionnels >* (icones on/off via nth-child(2)/(3))`
- **Modifiers** : `toggle-primary`, `toggle-secondary`, `toggle-accent`, `toggle-neutral`, `toggle-success`, `toggle-warning`, `toggle-info`, `toggle-error`, `toggle-xs`, `toggle-sm`, `toggle-md`, `toggle-lg`, `toggle-xl`, `sm:toggle..2xl:toggle (+ variantes responsive)`
- **Mécanismes CSS** : appearance:none sur input natif · display:inline-grid + grid-template-columns animees (0fr/1fr) pour le glissement · ::before content:var(--tw-content) = le thumb (background-color:currentColor, box-shadow inset) · transition grid-template-columns + translate + inset-inline-start · var(--input-color)/--size/--toggle-p/--border calc() · background-image:var(--fx-noise) sur ::before (texture, variable theme locale) · @starting-style pour fade-in du thumb · rotate/opacity transitions sur enfants nth-child · forced-colors / print media queries
- **CSP SBFB** : ✅ usable — CSS pur sur input checkbox natif. L'etat est gere par :checked (pas de JS requis pour l'apparence). --fx-noise est une variable de theme daisyUI resolue localement (data: SVG inline ou none selon theme), pas une url() distante http. Aucun mask, aucun backdrop-filter. Compose offline.
  - ⚠️ Aucun risque CSP : --fx-noise pointe vers une ressource interne au theme (souvent data: ou none), jamais une URL reseau
  - ⚠️ La lecture/persistance de l'etat (checked) cote app est a coder en JS LOCAL sans reseau si besoin
  - ⚠️ Ne pas dependre d'un submit de <form> (bloque) : lire input.checked via handler

```html
<input type="checkbox" class="toggle {MODIFIER}" />
```

---

## `tooltip` — tooltip

Affiche une infobulle au survol/focus d'un element. Le texte vient soit de l'attribut data-tip (rendu via ::before content:attr(data-tip)) soit d'un enfant .tooltip-content. ::after dessine la petite fleche. Apparait (opacity 0->1) au :hover, :has(:focus-visible) ou via la classe tooltip-open (forcage). Placement top(defaut)/bottom/left/right par transform+inset. Couleur de fond via --tt-bg, texte via --color-*-content.

- **Sous-parties** : `tooltip-content (alternative riche a data-tip)`, `::before (bulle texte via attr(data-tip))`, `::after (la fleche/tail)`
- **Modifiers** : `tooltip-open`, `tooltip-top`, `tooltip-bottom`, `tooltip-left`, `tooltip-right`, `tooltip-primary`, `tooltip-secondary`, `tooltip-accent`, `tooltip-info`, `tooltip-success`, `tooltip-warning`, `tooltip-error`, `sm:tooltip..2xl:tooltip (+ variantes responsive)`
- **Mécanismes CSS** : content:var(--tw-content) avec --tw-content:attr(data-tip) sur ::before · ::after avec mask-image:var(--mask-tooltip) = SVG INLINE en data:image/svg+xml (la fleche) · mask-position/mask-repeat · position:absolute + inset + transform translateX/Y + rotate selon placement · opacity transition cubic-bezier; :hover / :has(:focus-visible) / .tooltip-open declenchent · var(--tt-bg)/--tt-off/--tt-tail/--tt-pos · pointer-events:none sur la bulle
- **CSP SBFB** : ✅ usable — CSS pur declenche par :hover/:focus-visible — aucun JS requis. La fleche utilise mask-image avec un SVG INLINE en data:image/svg+xml (PAS une URL http distante) : data: est autorise par la CSP (default-src ... data:) et le mask est compose offline. Aucune ressource reseau.
  - ⚠️ mask-image present mais sur data: URI inline (autorise) — pas d'url() distante : OK
  - ⚠️ COEP require-corp n'affecte pas un mask data: inline
  - ⚠️ tooltip-open (forcage programmatique) necessite d'ajouter/retirer la classe en JS LOCAL si on veut un controle imperatif
  - ⚠️ Le texte data-tip est insere via CSS content : non selectionnable et lu differemment par les lecteurs d'ecran (preferer tooltip-content + aria pour l'accessibilite)

```html
<div class="tooltip {MODIFIER}" data-tip="Tooltip text">
  <button class="btn">Hover me</button>
</div>
```

---

## `validator` — validator

Change la couleur d'un controle de formulaire (input/select/textarea) en success ou error selon la validation native du navigateur via les pseudo-classes :user-valid / :user-invalid (et aria-invalid). Affecte --input-color (success/error). Quand le champ est invalide, le frere .validator-hint devient visible (visibility:visible + display:revert-layer) et passe en couleur error. validator-hint est cache par defaut (visibility:hidden).

- **Sous-parties** : `validator-hint (message d'erreur, frere du champ)`
- **Mécanismes CSS** : :user-valid / :user-invalid (validation form native CSS) · &:has(:user-valid) / :has(:user-invalid) pour wrappers · [aria-invalid]:not([aria-invalid=false]) selecteur attribut · --input-color reaffecte a var(--color-success)/var(--color-error) · ~ .validator-hint (selecteur de frere general) bascule visibility + display:revert-layer · visibility:hidden par defaut sur validator-hint
- **CSP SBFB** : ✅ usable — 100% CSS s'appuyant sur la validation NATIVE du navigateur (:user-valid/:user-invalid, attribut required/pattern/type). Aucune url(), aucun mask, aucun JS, aucun reseau. Fonctionne dans l'iframe scellee.
  - ⚠️ Aucun risque CSP
  - ⚠️ La validation est native (HTML constraint validation) : ne PAS compter sur un submit de <form> pour declencher l'envoi (form-action 'none' + sandbox bloque). :user-invalid se base sur l'interaction utilisateur, pas sur un submit
  - ⚠️ Pour une validation custom (pattern complexe, async), coder en JS LOCAL et poser aria-invalid sur l'element — le CSS suit aria-invalid
  - ⚠️ Aucun appel reseau pour valider (interdit par connect-src 'none') : toute verification serveur est impossible dans l'iframe

```html
<input type="{type}" class="input validator" required />
<p class="validator-hint">Error message</p>
```

---

## `glass` — glass

Effet verre depoli (glassmorphism). Applique un flou d'arriere-plan via backdrop-filter, un double linear-gradient de reflets (oklch blanc/noir transparent), un box-shadow inset de bordure et un text-shadow. Tout est parametre par des CSS custom properties (valeurs par defaut inline : blur 40px, opacity 30%, reflect 100deg/5%, border 20%).

- **Modifiers** : `--glass-blur`, `--glass-opacity`, `--glass-reflect-degree`, `--glass-reflect-opacity`, `--glass-border-opacity`, `--glass-text-shadow-opacity`
- **Mécanismes CSS** : backdrop-filter:blur() · background-image:linear-gradient() (double, reflets) · oklch() avec alpha · box-shadow inset (bordure simulee) · text-shadow · CSS custom properties (--glass-*)
- **CSP SBFB** : ✅ usable — CSS pur, aucune url() distante ni @font-face. backdrop-filter:blur() est un effet composite GPU autorise dans l'iframe scellee (default-src 'self' n'affecte pas les filtres CSS). Les gradients utilisent oklch() inline, pas de ressource externe. Une fois Tailwind/daisyUI purge dans app.css vendore same-origin, .glass est entierement statique.
  - ⚠️ backdrop-filter = effet composite (note perf) : floute ce qui est DERRIERE l'element dans le MEME document iframe ; ne traverse pas la frontiere de l'iframe scellee (origine opaque) donc ne floute jamais le shell hote — comportement attendu et sur
  - ⚠️ Necessite un fond non-opaque/un empilement derriere l'element pour etre visible ; sur fond plat l'effet parait nul
  - ⚠️ backdrop-filter peut etre desactive par certains navigateurs/parametres prefers-reduced-* mais degrade proprement (fond transparent)

```html
<div class="glass">...</div>
```

---

## `join` — join

Conteneur flex (inline-flex) qui groupe visuellement des enfants (boutons, inputs, selects) en une seule unite : arrondit uniquement les coins externes du premier/dernier .join-item via 4 custom properties de rayon (--join-ss/se/es/ee) pilotees par :first-child/:last-child/:only-child, et chevauche les bordures internes par margin-inline-start negatif (-var(--border)). join-vertical/horizontal change l'axe et bascule l'arrondi sur block-start/end. Variantes responsive par breakpoint.

- **Sous-parties** : `join-item`
- **Modifiers** : `join-vertical`, `join-horizontal`, `sm:join`, `md:join`, `lg:join`, `xl:join`, `2xl:join`, `sm:join-vertical`, `lg:join-horizontal`
- **Mécanismes CSS** : display:inline-flex · CSS custom properties --join-ss/se/es/ee (rayons logiques) · border-start/end-start/end-radius (proprietes logiques RTL-aware) · :first-child / :last-child / :only-child · margin-inline-start:calc(var(--border,1px)*-1) (chevauchement bordures) · z-index/isolation au :focus et .btn:hover · @media (width>=) breakpoints responsive · var(--radius-field) (rayon partage)
- **CSP SBFB** : ✅ usable — Pure mise en page CSS (flexbox + bordures groupees + variables de rayon). Aucune url(), font ni JS. Statique apres purge dans app.css same-origin. Les coins/bordures se calculent par selecteurs structurels — fonctionne dans l'iframe scellee sans aucun reseau.
  - ⚠️ Le chevauchement repose sur var(--border) et var(--radius-field) fournis par le theme daisyUI : ces tokens DOIVENT etre presents dans le app.css vendore (sinon fallback 1px / 0 rayon) — verifier que le theme racine est compile
  - ⚠️ RTL gere via proprietes logiques ; OK
  - ⚠️ Aucun comportement JS requis (groupement purement visuel)

```html
<div class="join">
  <button class="btn join-item">Button</button>
  <button class="btn join-item">Button</button>
  <button class="btn join-item">Button</button>
</div>
```

---

## `rounded-box` — radius (rounded-*)

Utilitaires d'arrondi semantiques alignes sur les 3 tokens de rayon du theme daisyUI : --radius-box (cartes/modales), --radius-field (boutons/inputs), --radius-selector (checkbox/toggle/petits controles). Couvre rayon global, par cote (t/b/l/r) et par coin (tl/tr/br/bl), avec declinaisons responsive a chaque breakpoint. Permet d'arrondir des elements custom de maniere coherente avec les composants daisyUI.

- **Modifiers** : `rounded-box`, `rounded-field`, `rounded-selector`, `rounded-t-box`, `rounded-b-box`, `rounded-l-box`, `rounded-r-box`, `rounded-tl-box`, `rounded-tr-box`, `rounded-br-box`, `rounded-bl-box`, `rounded-t-field`, `rounded-b-field`, `rounded-l-field`, `rounded-r-field`, `rounded-tl-field`, `rounded-tr-field`, `rounded-br-field`, `rounded-bl-field`, `rounded-t-selector`, `rounded-b-selector`, `rounded-l-selector`, `rounded-r-selector`, `rounded-tl-selector`, `rounded-tr-selector`, `rounded-br-selector`, `rounded-bl-selector`, `sm:/md:/lg:/xl:/2xl: variantes`
- **Mécanismes CSS** : border-radius / border-*-*-radius · CSS custom properties --radius-box / --radius-field / --radius-selector · @media (width>=) breakpoints responsive
- **CSP SBFB** : ✅ usable — Pur border-radius reference a des CSS variables de theme. Zero url/font/JS/reseau. Entierement statique apres compilation dans app.css same-origin.
  - ⚠️ Depend que les tokens --radius-box/field/selector soient definis par le theme racine compile dans le app.css vendore ; sans theme, valeur indefinie (pas d'arrondi)

```html
<div class="rounded-box bg-base-200 p-4">Carte arrondie token-coherente</div>
```

---

## `prose` — prose (typography)

Style typographique pour contenu HTML libre (markdown rendu, articles). daisyUI ne fournit que le PONT de theme : il remappe toutes les variables --tw-prose-* (body, headings, links, bold, quotes, code, pre-bg, borders...) sur les couleurs du theme daisyUI (var(--color-base-content), color-mix oklab pour les opacites, var(--color-neutral) pour les blocs pre). Style aussi le code inline (background base-200, bordure base-300, radius-selector, padding) en retirant les guillemets ::before/::after de Tailwind Typography. Le rendu typographique de base (marges, tailles titres) vient du plugin @tailwindcss/typography, pas de ce fichier.

- **Sous-parties** : `prose code`
- **Modifiers** : `--tw-prose-body`, `--tw-prose-headings`, `--tw-prose-links`, `--tw-prose-code`, `--tw-prose-pre-bg`, `--tw-prose-pre-code`, `prose-sm/base/lg/xl/2xl (plugin Tailwind typography)`
- **Mécanismes CSS** : CSS custom properties --tw-prose-* (theme bridge) · color-mix(in oklab, var(--color-base-content) X%, transparent) · var(--color-base-content/neutral/base-200/base-300) · var(--radius-selector) / var(--border) · :where(code):not(pre>code) (specificite 0) · ::before/::after display:none (retrait guillemets code)
- **CSP SBFB** : ✅ usable — Le pont de theme daisyUI est du CSS pur (variables + color-mix). Aucune url() ni @font-face. Compile dans app.css same-origin. Le contenu prose lui-meme reste statique.
  - ⚠️ Requiert le plugin @tailwindcss/typography pour le rendu reel (marges/tailles) : verifier qu'il est compile build-time dans app.css vendore
  - ⚠️ Liens <a href> internes (ancres #) OK ; tout href http externe = navigation hors-iframe geree par sandbox (pas allow-top-navigation) — du contenu prose ne doit pas dependre d'une navigation externe
  - ⚠️ Si le contenu prose contient des <img src=http://...> distants, ils sont bloques par CSP (img-src herite de default-src 'self') + COEP require-corp : n'inclure que des images same-origin/data:
  - ⚠️ Pas de comportement JS ; statique

```html
<article class="prose">
  <h1>Titre</h1>
  <p>Paragraphe avec <code>code inline</code> et un <a href="#section">lien interne</a>.</p>
</article>
```

---

## `mask` — mask

Recadre n'importe quel element (souvent <img>) a une forme geometrique. .mask pose mask-position:50%/mask-size:contain/no-repeat ; chaque forme applique un mask-image:url("data:image/svg+xml,...") INLINE (data URI, le SVG du contour est embarque dans le CSS, pas une ressource distante). mask-half-1/2 coupe la forme en deux via mask-size:200% + position 0/100% (RTL-aware). Toutes les formes sont des SVG data URIs same-origin compiles dans app.css.

- **Modifiers** : `mask-squircle`, `mask-heart`, `mask-hexagon`, `mask-hexagon-2`, `mask-decagon`, `mask-pentagon`, `mask-diamond`, `mask-square`, `mask-circle`, `mask-star`, `mask-star-2`, `mask-triangle`, `mask-triangle-2`, `mask-triangle-3`, `mask-triangle-4`, `mask-half-1`, `mask-half-2`
- **Mécanismes CSS** : mask-image:url("data:image/svg+xml,...") (data URI inline) · mask-position / mask-size:contain / mask-repeat:no-repeat · mask-size:200% (half) · :dir(rtl) / [dir=rtl] (mirroring) · @layer daisyui.l1.l2.l3 (cascade layers)
- **CSP SBFB** : ✅ usable — PIEGE EVITE : les masques daisyUI utilisent mask-image avec des SVG en DATA URI inline (data: autorise par default-src ... data:), PAS d'url() http distante. Donc sur dans l'iframe scellee une fois compile dans app.css same-origin. La forme est purement decorative/CSS.
  - ⚠️ PIEGE SVG/peinture : .mask masque la FORME mais ne PEINT pas un SVG ; pour un picto SVG colore via CSS, ne PAS compter sur fill-*/stroke-* Tailwind (ils ne compilent pas dans l'iframe) — peindre via mask-image + background-color:var(--color-*) ou color-mix, JAMAIS un <img src=http>
  - ⚠️ L'image source du <img class=mask> doit etre same-origin ou data: (img-src herite 'self' + COEP require-corp) ; un src http externe est bloque
  - ⚠️ Pas de JS ; statique. Le contenu sous le masque doit exister localement

```html
<img class="mask mask-squircle" src="avatar.webp" alt="avatar" />
```

---

## `--fx-noise` — svg noise/icons (base — svg.css)

Couche base daisyUI qui definit la variable racine --fx-noise : une texture de bruit (feTurbulence fractalNoise) encodee en SVG DATA URI inline, utilisable comme background-image decorative (ex. grain de texture sur fond). C'est la facon canonique dont daisyUI embarque des SVG : data URI inline, jamais une URL de fichier distant.

- **Modifiers** : `--fx-noise`
- **Mécanismes CSS** : url("data:image/svg+xml,...") (SVG inline, feTurbulence) · CSS custom property :root --fx-noise · filter feTurbulence/fractalNoise (dans le SVG embarque)
- **CSP SBFB** : ✅ usable — SVG embarque en DATA URI (data: autorise par default-src ... data:). Aucune ressource reseau. Sert de modele : toute icone/texture SVG dans une app scellee doit etre inline (data: ou balise <svg> dans le DOM), jamais url(http://). Statique apres compilation.
  - ⚠️ LECON GENERALE pour le pack : reproduire ce pattern (SVG data URI inline / <svg> DOM) pour TOUTES les icones de l'app ; bannir <img src=http://...svg> et url(http) en mask/background
  - ⚠️ Si une app definit un --fx-* avec une url() distante, c'est bloque (default-src 'self') — toujours inline

```html
<div style="background-image:var(--fx-noise)" class="opacity-20">grain</div>
```

---

## `:root` — root color + reset (base)

Socle base daisyUI : rootcolor.css fixe le background/color de :root et [data-theme] sur var(--root-bg)=var(--color-base-100) et var(--color-base-content) (le theme peint la page). reset.css = preflight Tailwind/daisyUI (box-sizing:border-box, marges 0, border 0 solid, normalisation form controls, img/svg display:block, ::placeholder via color-mix). scrollbar.css colore la scrollbar via color-mix. properties.css declare @property --radialprogress. Ces couches posent les tokens et la normalisation dont dependent tous les composants.

- **Sous-parties** : `[data-theme]`
- **Modifiers** : `--root-bg`, `--page-scroll-bg`, `--color-base-100`, `--color-base-content`, `--default-font-family`
- **Mécanismes CSS** : CSS custom properties :root/[data-theme] (--root-bg, --color-base-*) · color-mix(in oklch, currentColor X%, transparent) (placeholder, scrollbar) · @property --radialprogress (registered custom property, animation %) · preflight reset (box-sizing, border 0 solid, appearance:button) · font-family:var(--default-font-family, ui-sans-serif, system-ui...)
- **CSP SBFB** : ✅ usable — CSS pur (tokens, reset, color-mix, @property). Aucune url ni @font-face distante. La font-family par defaut est une PILE SYSTEME (ui-sans-serif, system-ui, sans-serif, emoji) — pas de Google Fonts, donc compatible CSP (aucune requete reseau de police). Compile dans app.css same-origin.
  - ⚠️ PIEGE FONT EVITE : --default-font-family pointe sur des polices SYSTEME, pas un @font-face distant ; si l'app veut une police custom, l'embarquer en @font-face avec src:url(data:) ou un fichier same-origin vendore, JAMAIS Google Fonts (bloque par CSP + COEP)
  - ⚠️ Le theme racine (data-theme + tokens --color-*) DOIT etre compile dans app.css pour que tous les composants/glass/join/prose aient leurs variables ; un app.css sans theme = couleurs/rayons indefinis
  - ⚠️ @property --radialprogress sert le composant radial-progress (animation de pourcentage) — statique, pas de JS requis pour la declaration

```html
<html data-theme="dark"><body class="bg-base-100 text-base-content">...</body></html>
```

---

