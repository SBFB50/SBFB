# Banque d'exemples — anime.js v4.5

> 52 démos distillées (examples/ + tests/playground/). Chaque entrée = « le truc » + primitives + empreinte de nouveauté + rejouabilité SBFB.

| # | nom | SBFB | empreinte | primitives |
|---|---|:--:|---|---|
| 1 | additive-creature | ✅ | prims{animate,composition:blend,createTimeline,createTimer(frameRate)… | animate, createTimeline, createTimer, stagger, utils.set, u… |
| 2 | additive-fireflies | ✅ | prims{animate,composition:blend,createTimer(frameRate:4),utils.random… | animate, createTimer, utils.set, utils.random, utils.mapRan… |
| 3 | advanced-grid-staggering | ✅ | prims{createTimeline(onComplete:self),stagger(grid,from:dynamicIndex,… | createTimeline, stagger, utils.get, utils.set, utils.random… |
| 4 | animatable-follow-cursor | ✅ | prims{createAnimatable,stagger(grid,from:center)-as-duration,utils.ma… | createAnimatable, utils.$, utils.get, utils.mapRange, stagg… |
| 5 | layered-css-transforms | ✅ | prims{createTimeline(onComplete:self),createSpring()-as-ease-in-pool,… | createTimeline, createSpring, utils.random, utils.randomPic… |
| 6 | stagger | ✅ | prims{createTimeline(composition:false,loop),stagger([min,max],grid:t… | createTimeline, stagger, utils.set, utils.random, from:'+='… |
| 7 | text/scramble | ✅ | prims{animate(innerHTML:scrambleText),scrambleText(from,chars,cursor,… | animate, scrambleText, createTimer, createTimeline, $ (util… |
| 8 | text/scramble-tl | ✅ | prims{createTimeline,scrambleText(reversed,override,from,cursor,pertu… | createTimeline, scrambleText, stagger, tl.label-relative po… |
| 9 | text/split-effects | ✅ | prims{splitText(lines,debug),split.addEffect(return-cleanup-fn),split… | splitText({lines,debug}), split.addEffect, split.revert/ref… |
| 10 | text/split-playground | ✅ | prims{splitText(lines+words+chars,wrap:clip,clone:dir,includeSpaces,a… | splitText({lines/words/chars: {wrap,clone,accessible,includ… |
| 11 | text/hover-effects | ✅ | prims{createScope(root,methods),splitText(chars/words template,clone,… | createScope({root,defaults}), scope.add('method',fn)+scope.… |
| 12 | animejs-v4-logo-animation | ✅ | prims{createTimeline(labels),svg.morphTo(chain),svg.createDrawable+dr… | createTimeline, stagger, svg.morphTo, svg.createDrawable, d… |
| 13 | timeline-50K-stars | ✅ | prims{animate,createTimeline,cubicBezier,steps,utils.random,utils.rou… | createTimeline, animate, utils.$, utils.set, utils.random, … |
| 14 | timeline-refresh-starlings | ✅ | prims{createTimeline,stagger,utils.random,utils.set,modifier,onLoop-r… | createTimeline, stagger, utils.set, utils.random, onLoop:se… |
| 15 | timeline-seamless-loop | ✅ | prims{createTimeline,stagger,utils.set,utils.degToRad,loopDelay,loop:… | createTimeline, stagger, stagger([0,1],{ease,reversed,from:… |
| 16 | timeline-stress-test | ✅ | prims{createTimeline,stagger,utils.mapRange,utils.round,playbackEase,… | createTimeline, stagger, stagger([0,duration]), utils.mapRa… |
| 17 | svg-graph | ✅ | prims{createTimeline,svg,utils.round,innerHTML-from,modifier-locale}\|… | createTimeline, animate(implicite-via-add), svg(import), st… |
| 18 | svg-line-drawing | ✅ | prims{svg.createDrawable,createTimeline,stagger,utils.random,draw-key… | svg.createDrawable, createTimeline, stagger, stagger([0,800… |
| 19 | onscroll-responsive-scope | ✅ | prims{createScope,onScroll-link,animate,stagger,scope.matches}\|struct… | createScope, createScope({mediaQueries}), scope.matches.lan… |
| 20 | onscroll-sticky | ✅ | prims{createTimeline,onScroll-autoplay,composition:blend,stagger,util… | createTimeline, onScroll-as-autoplay, autoplay:onScroll({en… |
| 21 | irregular-playback-typewriter | ✅ | prims{createTimeline,easings.irregular,easings.steps,stagger,animate-… | createTimeline, easings.irregular(steps,randomness), easing… |
| 22 | clock-playback-controls | ✅ | prims{createTimeline,masterTL.sync,animate(tl.currentTime),animate(tl… | createTimeline, masterTL.sync(childTL,0), animate(masterTL,… |
| 23 | canvas-2d | ✅ | prims{animate,createTimer,utils.random,utils.randomPick,onComplete-re… | animate, createTimer, createTimer({onUpdate}), animate(plai… |
| 24 | draggable-infinite-auto-carousel | ✅ | prims{createDraggable,createAnimatable,createTimer,utils.wrap,utils.l… | createDraggable, createAnimatable, createAnimatable({x,modi… |
| 25 | draggable-mouse-scroll-snap-carousel | ✅ | prims{createDraggable,createTimer,utils.snap,utils.lerp,utils.get-css… | createDraggable, createDraggable({snap,container,onAfterRes… |
| 26 | draggable-playground | ✅ | prims{createDraggable,createAnimatable,createTimeline,createSpring,ea… | createDraggable, createAnimatable, createTimer, createTimel… |
| 27 | additive-animations-2 (particle spheres, composition:'blend') | ✅ | animate+composition:blend+createTimer(loop,onLoop)+stagger \| N concur… | animate, createTimer, createTimeline, stagger, utils.set, u… |
| 28 | advanced-staggering-demos (catalogue from/grid/axis/range) | ✅ | createTimeline.alternate.loop + stagger(from\|grid\|axis\|range\|reversed… | createTimeline, stagger, stagger(value,{from}), stagger([a,… |
| 29 | stagger-grid-demo (24x24 grid choreography) | ✅ | createTimeline.loop + 576 generated divs + stagger(grid,axis,from:cen… | createTimeline, stagger({grid,from,axis}), utils.random (fu… |
| 30 | sprite-animation (steps() sprite-sheet) | ✅ | animate + ease:steps(N) + backgroundPosition loop \| discrete frame st… | animate, steps(9), backgroundPosition, loop:true |
| 31 | svg-path-animation (createDrawable line-draw map) | ✅ | createTimeline + svg.createDrawable(selector-all) + draw:['0 0','0 1'… | createTimeline, svg.createDrawable('path'), draw:['0 0','0 … |
| 32 | svg-morph-timeline (svg.morphTo chains, path & polygon) | ✅ | createTimeline.loop + svg.morphTo (d for path, points for polygon) ch… | createTimeline, svg.morphTo('#shape'), d: morphTo, points: … |
| 33 | svg-motion-path (createMotionPath responsive, spread params) | ✅ | animate + ...svg.createMotionPath(selector) spread on mixed DOM+SVG t… | animate, svg.createMotionPath('#path'), spread ...svg.creat… |
| 34 | animejs-v3-logo (orchestrated morph + motion-path + spring) | ✅ | createTimeline + svg.createMotionPath driving .dot + d:el=>dataset.d2… | createTimeline, svg.createMotionPath('.bounce path'), bounc… |
| 35 | animejs-v2-logo (mass createDrawable staggered line-draw) | ✅ | createTimeline.alternate + svg.createDrawable(group) + draw '0 1'->'1… | createTimeline, svg.createDrawable('.fill'), draw:{to:'0 1'… |
| 36 | animejs-mgs-logo (classList-branching function values + attr morph) | ✅ | createTimeline.alternate.loop + translateX/Y=target=>classList branch… | createTimeline.alternate.loop, translateX/Y: target => bran… |
| 37 | color-trail-canvas (plain-object targets + sibling color chaining) | ✅ | createTimer.onUpdate canvas draw + animate(plainObject{x,y,color}) + … | createTimeline, createTimer({onUpdate}), animate plain-obje… |
| 38 | color-conversion (cross-format color interpolation) | ✅ | animate + backgroundColor:[anyFormat, anyFormat] alternate \| interpol… | animate, backgroundColor: [fromStr, toStr], alternate loop,… |
| 39 | scramble (scrambleText param matrix) | ✅ | animate + innerHTML:scrambleText({from\|chars\|cursor\|perturbation\|seed… | animate, innerHTML: scrambleText(params), scrambleText({fro… |
| 40 | lerp (frame-decoupled follow via utils.lerp / utils.damp) | ✅ | createTimer.onUpdate + utils.get readback + utils.lerp vs utils.damp(… | createTimer({frameRate,onUpdate}), utils.lerp(a,b,t), utils… |
| 41 | timekeeper (createScope mediaQueries + keepTime) | ✅ | createScope({mediaQueries}) + self.keepTime(()=>createTimeline) + sco… | createScope({mediaQueries}), scope.add(self => ...), self.k… |
| 42 | scope (createScope root + addOnce + keepTime + revert) | ✅ | createScope({root,mediaQueries}) + addOnce + keepTime + return cleanu… | createScope({mediaQueries,defaults,root}), self.addOnce(fn)… |
| 43 | playback (engine + animation playback control surface) | ✅ | createTimeline + engine.fps/speed + animation.fps/speed/reversed/iter… | createTimeline({onUpdate,onLoop}), engine.fps, engine.speed… |
| 44 | timeline-nested (tl.sync of timelines + timers) | ✅ | createTimeline.sync(animate) + .sync(timeline) + .sync(timer) nested … | animate (autonomous), createTimeline.sync(animation, positi… |
| 45 | tl-seek-test (mouse-scrubbed 2000-element timeline) | ✅ | createTimeline(autoplay:false) + 2000 adds + tl.progress = mouseX/wid… | createTimeline({autoplay:false}), tl.add per element, tl.pr… |
| 46 | onscroll-sync-modes (scroll-linked sync mode catalogue) | ✅ | animate + autoplay:onScroll({sync: number\|'play pause'\|'play alternat… | animate, autoplay: onScroll({...}), onScroll sync:.5 (smoot… |
| 47 | onscroll-sticky-snap (scroll-scrubbed 3D card-stack) | ✅ | createTimeline + autoplay:onScroll({sync:1,target}) + per-card rotate… | createTimeline, autoplay: onScroll({target,sync:1,enter:'to… |
| 48 | waapi-composition (3 engines side-by-side: waapi vs js) | ✅ | waapi.animate(translate shorthand) vs waapi.animate(x,y) vs js animat… | waapi.animate, animate (js), translate shorthand 'Xpx Ypx',… |
| 49 | sandbox (timeline.sync of WAAPI animations) | ✅ | waapi.animate(autoplay:false) + createTimeline.sync(waapiAnim, positi… | waapi.animate({autoplay:false}), createTimeline.sync(waapiA… |
| 50 | keyframes (percent-keyframe object syntax vs WAAPI vs CSS) | ✅ | animate({keyframes:{'0%':{},'30%':{...,ease}}}) + native el.animate(o… | animate({keyframes:{'0%':{...},'30%':{...,ease}}}), per-key… |
| 51 | layout (createLayout FLIP auto-animate on DOM change) | ✅ | createLayout(selector) + layout.update(mutateFn,{duration}) + data-la… | createLayout('.container'), layout.update(({root})=>{...}, … |
| 52 | draggables-callbacks (createDraggable snap + lifecycle + animate target) | ✅ | createDraggable({container,snap:self=>w,velocityMultiplier}) + full o… | createDraggable('#el',{container,snap,velocityMultiplier}),… |

## additive-creature

- **Source** : `examples/additive-creature/index.js`
- **Idée** : Une grille 13x13 de blobs en mix-blend-mode:plus-lighter devient une creature organique : un createTimer a frameRate:15 relance EN CONTINU une animate avec composition:'blend', donc chaque pas de cible se fond additivement avec l'inertie de la position precedente au lieu de l'ecraser. Le curseur est pilote par une timeline auto (Math.sin/cos sur currentTime via modifier) tant que la souris ne bouge pas, sinon un createTimer de 1.5s rebascule en auto.
- **Tags** : `grid-stagger`, `additive-blend`, `framerate-throttled-retarget`, `procedural-modifier`, `auto-vs-manual-pointer-handoff`, `stagger-as-static-value`
- **Primitives** : `animate`, `createTimeline`, `createTimer`, `stagger`, `utils.set`, `utils.round`, `composition:'blend'`, `ease:'inOutExpo'`, `ease:'inQuad'`, `modifier`
- **Empreinte** : prims{animate,composition:blend,createTimeline,createTimer(frameRate),stagger(grid,from:center),utils.set}+struct{grid NxN de divs blend-mode, timer-retarget periodique vers cursor partage, dual sin/cos-modifier autopilot, pulse keyframes onLoop}
- **SBFB** : ✅ — Aucun reseau/fetch/worker, pur DOM+anime UMD. mix-blend-mode:plus-lighter et composition:'blend' marchent dans l'iframe. Adaptations: importer window.anime au lieu de l'ESM; boxShadow est seulement POSE via utils.set/stagger (jamais transitionne) donc OK; honorer prefers-reduced-motion en figeant la timeline auto (etat-final via utils.set scaleStagger/opacityStagger sans pulse).

```js
const mainLoop = createTimer({
  frameRate: 15, // retarget toward cursor every ~250ms
  onUpdate: () => {
    animate(particuleEls, {
      x: cursor.x,
      y: cursor.y,
      delay: stagger(40, { grid, from }),
      duration: stagger(120, { start: 750, ease: 'inQuad', grid, from }),
      ease: 'inOut',
      composition: 'blend', // overlapping animations blend instead of overwrite
    });
  }
});
const autoMove = createTimeline()
.add(cursor, {
  x: [-viewport.w * .45, viewport.w * .45],
  modifier: x => x + Math.sin(mainLoop.currentTime * .0007) * viewport.w * .5,
  duration: 3000, ease: 'inOutExpo', alternate: true, loop: true,
  onBegin: pulse, onLoop: pulse,
}, 0);
```

---

## additive-fireflies

- **Source** : `examples/additive-fireflies/index.js`
- **Idée** : 225 particules, chacune possede SON PROPRE createTimer a frameRate:4 qui, a chaque tick, choisit un angle aleatoire et anime la particule vers un point sur un cercle (cos/sin * rayon) autour du pointeur, avec duration/ease randomises per-axe et composition:'blend'. Le glow du pointeur n'anime QUE scale/opacity/filter (jamais box-shadow, qui est statique sur ::before/::after) — exactement la regle composite-safe.
- **Tags** : `additive-blend`, `per-element-independent-timer`, `radial-scatter`, `composite-safe-glow`, `random-per-axis-duration`, `pointer-attractor`
- **Primitives** : `animate`, `createTimer`, `utils.set`, `utils.random`, `utils.mapRange`, `composition:'blend'`, `ease:`inOut(${n})``, `per-particle createTimer(frameRate:4)`
- **Empreinte** : prims{animate,composition:blend,createTimer(frameRate:4),utils.random,utils.set}+struct{N timers independants un-par-particule, retarget radial cos/sin*radius autour pointeur, ease inOut(rand 1-5), glow box-shadow statique sur pseudo-el opacity-only anime}
- **SBFB** : ✅ — Pas de reseau. Le pattern 'un createTimer par element' est lourd (225 timers) mais reste local. Le glow respecte deja la regle SBFB (box-shadow statique sur :before/:after, seuls opacity/scale/filter sont animes au mousedown). Adaptations: window.anime UMD; sous prefers-reduced-motion, ne pas demarrer les timers et poser une disposition statique.

```js
function animateParticule($el) {
  createTimer({
    frameRate: 4,
    onUpdate: () => {
      const angle = Math.random() * Math.PI * 2;
      const radius = pointer.isDown ? activeRadius : baseRadius;
      animate($el, {
        x: { to: (Math.cos(angle) * radius) + pointer.x, duration: () => utils.random(1000, 2000) },
        y: { to: (Math.sin(angle) * radius) + pointer.y, duration: () => utils.random(1000, 2000) },
        scale: .5 + utils.random(.1, 1, 2),
        ease: `inOut(${utils.random(1, 5)})`,
        composition: 'blend'
      });
    }
  })
}
```

---

## advanced-grid-staggering

- **Source** : `examples/advanced-grid-staggering/index.js`
- **Idée** : Un seul curseur saute d'index aleatoire en index aleatoire sur une grille 41x41 de points ; le SECRET est que stagger est calcule deux fois par axe avec from:index (origine = l'emplacement courant du curseur) puis re-cible avec from:nextIndex pour le deplacement, et la ripple des points utilise {from, to} stagger PAR AXE pour creer un ressac qui converge vers l'origine mobile. Le curseur chevauche la ripple via une position negative '-=1500' dans la timeline.
- **Tags** : `grid-stagger`, `moving-stagger-origin`, `axis-split-stagger`, `from-to-stagger`, `self-chaining-timeline`, `negative-timeline-offset`
- **Primitives** : `createTimeline`, `stagger`, `utils.get`, `utils.set`, `utils.random`, `keyframes`, `axis:'x'/'y'`, `from:index`, `negative-time-position '-=1500'`
- **Empreinte** : prims{createTimeline(onComplete:self),stagger(grid,from:dynamicIndex,axis),keyframes,utils.random,utils.get(css-var)}+struct{origine de stagger = index courant variable (pas 'center'/'first'), stagger x et y separes par axis, from->to stagger entre index et nextIndex, overlap negatif '-=1500'}
- **SBFB** : ✅ — 100% local, donnees simulees (positions aleatoires), aucun reseau. Le --rows lu via utils.get(document.body,'--rows') marche. Adaptations: window.anime UMD; sous prefers-reduced-motion, ne pas relancer onComplete:animateGrid (poser le curseur a un index fixe). Aucun piege SBFB touche (pas de SVG, pas de box-shadow).

```js
animation = createTimeline({ defaults: { ease: 'inOutQuad' }, onComplete: animateGrid })
  .add('.dot', {
    keyframes: [
      { x: stagger('-.175rem', {grid, from: index, axis: 'x'}), y: stagger('-.175rem', {grid, from: index, axis: 'y'}), duration: 200 },
      { x: stagger('.125rem', {grid, from: index, axis: 'x'}), y: stagger('.125rem', {grid, from: index, axis: 'y'}), scale: 2, duration: 500 },
      { x: 0, y: 0, scale: 1, duration: 600 }
    ],
    delay: stagger(50, {grid, from: index}),
  }, 0)
  .add('.cursor', {
    x: { from: stagger('-1rem', {grid, from: index, axis: 'x'}), to: stagger('-1rem', {grid, from: nextIndex, axis: 'x'}), duration: utils.random(800, 1200) },
    y: { from: stagger('-1rem', {grid, from: index, axis: 'y'}), to: stagger('-1rem', {grid, from: nextIndex, axis: 'y'}), duration: utils.random(800, 1200) },
    ease: 'outCirc'
  }, '-=1500')
index = nextIndex;
```

---

## animatable-follow-cursor

- **Source** : `examples/animatable-follow-cursor/index.js`
- **Idée** : createAnimatable enregistre x/y/rotate comme setters appelables a chaque pointermove ; le truc est que la DUREE de chaque setter est elle-meme un stagger grid (from:'center') — donc une seule valeur pointeur poussee a 441 elements produit un retard radial sans aucune boucle ni timer. rotate a duration:0 (setter instantane) tandis que x/y trainent avec outElastic, et atan2 oriente toute la grille vers le curseur.
- **Tags** : `createAnimatable`, `stagger-as-duration`, `pointer-driven-setter`, `radial-lag`, `elastic-trail`, `no-loop-no-timer`
- **Primitives** : `createAnimatable`, `utils.$`, `utils.get`, `utils.mapRange`, `stagger`, `ease:'outElastic(.3, 1.4)'`, `rotate unit:'rad' duration:0`, `per-prop duration`
- **Empreinte** : prims{createAnimatable,stagger(grid,from:center)-as-duration,utils.mapRange,Math.atan2,ease:outElastic}+struct{setters x/y/rotate appeles dans onpointermove, duration=stagger radial donc lag par distance, rotate duration:0 instantane, zero timeline/timer}
- **SBFB** : ✅ — Pleinement rejouable : DOM + pointermove + anime UMD, aucun reseau. mix-blend-mode:plus-lighter et box-shadow STATIQUE dans le CSS (pas anime) sont conformes. Adaptations: window.anime UMD; sous prefers-reduced-motion, ne pas brancher onpointermove (ou mapper sans easing). C'est le pattern le plus econome (zero loop) — ideal en iframe.

```js
const duration = stagger(50, { ease: 'in(1)', from: 'center', grid: [rows, rows] });
const particles = createAnimatable('.particles div', {
  x: { duration }, // staggered per-element duration => radial lag
  y: { duration },
  rotate: { unit: 'rad', duration: 0 }, // instant setter, no easing
  ease: 'outElastic(.3, 1.4)',
});
window.onpointermove = e => {
  const { clientX, clientY } = e;
  particles.x(utils.mapRange(clientX, 0, w, -hw, hw));
  particles.y(utils.mapRange(clientY, 0, h, -hh, hh));
  particles.rotate(-Math.atan2(hw - clientX, hh - clientY));
}
```

---

## layered-css-transforms

- **Source** : `examples/layered-css-transforms/index.js`
- **Idée** : Genere proceduralement 100 keyframes par propriete, chacune avec un ease tire au sort dans un pool {inOutQuad,inOutCirc,inOutSine,createSpring()} et une duree aleatoire 300-1600ms — un mouvement perpetuel non-repetitif. Anime SIMULTANEMENT le transform du SVG host ET les attributs geometriques internes (circle r, rect width/height, polygon points re-scales depuis les points d'origine parses), le tout reboucle par onComplete recursif.
- **Tags** : `procedural-keyframes`, `random-ease-pool`, `svg-attribute-morph`, `spring-ease-in-pool`, `recursive-self-restart`, `non-repeating-loop`
- **Primitives** : `createTimeline`, `createSpring`, `utils.random`, `utils.randomPick`, `animation.init()`, `keyframes-array`, `function-based value`, `SVG attr animation (r, width, height, points)`
- **Empreinte** : prims{createTimeline(onComplete:self),createSpring()-as-ease-in-pool,utils.randomPick(eases),utils.random,keyframes(100x),svg-attrs:r/width/height/points}+struct{100 keyframes generes par prop, ease randomise par frame, SVG host transform + geometrie interne animes ensemble, polygon points recalcules*scale}
- **SBFB** : ✅ — Local, pas de reseau. ATTENTION piege #6 : createSpring() est DEPRECIE en v4.5 -> remplacer par spring() (valide aussi comme ease:). SVG ici utilise stroke/fill via CSS currentColor/.color-red, pas les utilitaires Tailwind fill-*/stroke-* (donc conforme piege #3 si on garde la peinture CSS var). Adaptations: window.anime UMD + spring(); sous prefers-reduced-motion, init() une seule passe sans onComplete recursif.

```js
const eases = ['inOutQuad', 'inOutCirc', 'inOutSine', createSpring()]; // v4.5: use spring()
function createKeyframes(value) {
  var keyframes = [];
  for (let i = 0; i < 100; i++) {
    keyframes.push({ to: value, ease: utils.randomPick(eases), duration: utils.random(300, 1600) });
  }
  return keyframes;
}
animation.add(polyEl, {
  points: createKeyframes(() => {
    const s = utils.random(.9, 1.6, 3);
    return `${points[0]*s} ${points[1]*s} ${points[2]*s} ${points[3]*s} ${points[4]*s} ${points[5]*s}`;
  }),
}, 0);
animation.init();
```

---

## stagger

- **Source** : `examples/stagger/index.js`
- **Idée** : 1000 points disperses aleatoirement, animes par une SEULE timeline ; le truc est le stagger comme POSITION de timeline (3e arg de .add) avec une plage [0,2000]ms, grid:true (grille inferee automatiquement) et axis:'x' from:'center' — produit une vague horizontale qui se propage du centre. Les keyframes utilisent des valeurs RELATIVES ('-=1'/'+=2', '-=180'/'+=180') donc l'animation part de l'etat aleatoire courant de chaque point sans le connaitre.
- **Tags** : `grid-stagger`, `stagger-as-timeline-position`, `inferred-grid`, `relative-keyframe-values`, `axis-wave`, `composition:false`
- **Primitives** : `createTimeline`, `stagger`, `utils.set`, `utils.random`, `from:'+=' relative values`, `stagger([0,2000] grid axis:'x')`, `composition:false`, `loop:true`
- **Empreinte** : prims{createTimeline(composition:false,loop),stagger([min,max],grid:true,from:center,axis:x)-as-position,utils.set(random),relative-values(+=/-=)}+struct{1000 elts dispersion aleatoire, stagger range en 3e arg de add = vague axiale, keyframes relatifs etat-courant-agnostiques, grid auto-inferee}
- **SBFB** : ✅ — Trivialement rejouable : DOM + anime UMD, donnees aleatoires locales, aucun reseau. Couleurs via var(--color-N) dataset, conforme. Adaptations: window.anime UMD; 1000 elts est OK mais sous prefers-reduced-motion poser un etat statique sans loop. Aucun SVG/box-shadow/morph en jeu.

```js
createTimeline({ composition: false })
  .add(dots, {
    scale: [{ from: '-=1', to: '+=2' }],   // relative to each dot's random current state
    rotate: [{ from: '-=180', to: '+=180' }],
    background: [{ from: '#FFF' }],
    duration: 1000,
    ease: 'inOut(3)',
    loop: true,
  }, stagger([0, 2000], { grid: true, from: 'center', axis: 'x' })) // stagger as timeline position
  .init();
```

---

## text/scramble

- **Source** : `examples/text/scramble/index.js`
- **Idée** : scrambleText() retourne une valeur de tween basee-fonction passee a animate($el,{innerHTML: scrambleText({...})}) : chaque element capture son texte courant et calcule une reveal-timeline per-caractere (override, cursor '░▒▓█', perturbation aleatoire de timing, settleDuration). Bonus: un createTimer frameRate:30 + WebAudio cree un 'tick' sonore synchronise via le callback onChange du scramble.
- **Tags** : `scramble-text`, `innerHTML-tween`, `function-based-tween-value`, `per-char-reveal`, `cursor-sweep`, `onChange-audio-tick`, `intro-timeline-overlap`
- **Primitives** : `animate`, `scrambleText`, `createTimer`, `createTimeline`, `$ (utils.$)`, `innerHTML tween`, `scrambleText opts: from/chars/cursor/perturbation/settleDuration/revealRate/override`
- **Empreinte** : prims{animate(innerHTML:scrambleText),scrambleText(from,chars,cursor,perturbation,settleDuration,revealRate,override),createTimer(frameRate:30),createTimeline}+struct{scrambleText=function-tween sur innerHTML, override+cursor+perturbation, hover/pointerdown=replay, onChange->WebAudio tick}
- **SBFB** : ✅ — Le coeur (scrambleText sur innerHTML) est 100% local et conforme au piege #5 (animate(el,{innerHTML:...}), texte cible pose d'abord). MAIS retirer tout le bloc 'tweaks' GUI (import 'tweaks' via importmap ESM = non-classique, dev-only). WebAudio AudioContext fonctionne dans l'iframe (pas de reseau) mais doit etre gate par geste utilisateur. Adaptations: window.anime UMD; supprimer importmap+tweaks; strings FR; honorer prefers-reduced-motion en posant le texte final sans animer.

```js
$('.scramble').forEach($el => {
  const replay = () => animate($el, { innerHTML: scrambleText({ ...scrambleParams, text: scrambleParams.text || undefined, onChange: tickSound }) });
  intro.add($el, {
    innerHTML: scrambleText({
      override: '',
      duration: 750,
      settleDuration: 250,
      perturbation: .2,
      cursor: '░▒▓█',
    }),
  }, '-=620');
  $el.addEventListener('pointerenter', replay);
  $el.addEventListener('pointerdown', replay);
});
intro.init();
```

---

## text/scramble-tl

- **Source** : `examples/text/scramble-tl/index.js`
- **Idée** : Une bande-annonce auto-jouee : une seule timeline enchaine ~25 scrambleText sur 3 slides, orchestree entierement par les positions relatives anime ('<<' = avec le precedent, '<<+=50', '<-=600', '<+=750') au lieu de delais absolus. Le scramble est utilise dans LES DEUX sens : reveal (text:'Anime.js...') et masquage inverse (text:'', reversed:true) pour faire disparaitre. stagger([0,1000],{grid:true,from:'center'}) cadence des grilles de mots.
- **Tags** : `scramble-text`, `relative-timeline-positions`, `scrubbed-trailer`, `reverse-scramble-hide`, `grid-stagger-words`, `background-color-tween`, `multi-slide-sequence`
- **Primitives** : `createTimeline`, `scrambleText`, `stagger`, `tl.label-relative positions '<<' '<<+=50' '<-=600'`, `innerHTML tween x12`, `stagger([min,max] grid:true from:center reversed)`, `background var() tween`
- **Empreinte** : prims{createTimeline,scrambleText(reversed,override,from,cursor,perturbation),stagger([min,max],grid:true,from:center,reversed),relative-pos('<<','<+=','<-=')}+struct{trailer mono-timeline 3 slides, ~25 scramble sequences par positions relatives, scramble bidirectionnel reveal+hide(text:'' reversed), background var() anime entre slides}
- **SBFB** : ✅ — Coeur rejouable et tres demonstratif. MAIS retirer import 'animatorkit/editor/gui' showGUI() (dev-only, chemin hors-repo) + l'importmap. Le pattern de positions relatives + scramble bidirectionnel est ideal pour une intro scellee. Adaptations: window.anime UMD; supprimer showGUI; remplacer les libelles anglais ('Introducing','Custom Easing'...) par du FR; sous prefers-reduced-motion, seek a la fin ou poser le dernier slide statique.

```js
tl.add('.slide:nth-child(1) p:not(.center)', {
  scale: { from: .75 }, color: { to: 'var(--red-1)' },
  innerHTML: scrambleText({ override: ' ', from: 'center', duration: 500, revealDelay: 250, cursor: '░▒▓', perturbation: .25 }),
}, stagger([250, 750], { grid: true, from: 'center', ease: 'out(3)', start: '<<' }));
tl.add('.slide:nth-child(1) p:not(.center)', {
  innerHTML: scrambleText({ text: '', override: false, from: 'center', ease: 'outQuad', reversed: true, duration: 800, cursor: '░▒▓' })
}, '<+=150');
tl.add('body', { background: 'var(--black-1)' }, '<-=600');
```

---

## text/split-effects

- **Source** : `examples/text/split-effects/index.js`
- **Idée** : splitText avec lines:true expose split.lines ET split.words ; le pattern cle est split.addEffect(cb) : le callback recoit l'objet split et retourne soit une animation, soit une FONCTION de cleanup executee au revert/refresh. Ici on combine deux effets — une vague de lignes (stagger 'data-line') + un mode drag par mot qui PERSISTE ses coordonnees (le cleanup capture utils.get(w,'x/y') dans coords[] pour les restaurer apres re-split).
- **Tags** : `split-text`, `split-addEffect`, `effect-cleanup-persistence`, `data-line-stagger`, `line-word-dual-split`, `draggable-words-reorder`, `revert-refresh-lifecycle`
- **Primitives** : `splitText({lines,debug})`, `split.addEffect`, `split.revert/refresh`, `createTimeline`, `animate`, `stagger(use:'data-line')`, `utils.set/get`, `split.lines/words`
- **Empreinte** : prims{splitText(lines,debug),split.addEffect(return-cleanup-fn),split.lines/words,stagger(use:data-line),animate,utils.get/set}+struct{addEffect retourne animation OU cleanup-fn, coords[] persistes via utils.get au revert et restaures au re-split, dual line-wave + per-word drag/tidy}
- **SBFB** : ✅ — Le coeur splitText+addEffect+persistance est rejouable et conforme au piege #5 (splitText enveloppe le contenu courant). MAIS: retirer le bloc tweaks GUI + l'importmap ESM, ET surtout supprimer le Google Fonts (preconnect fonts.googleapis.com = CDN INTERDIT) -> embarquer une police locale en data:/relative ou utiliser une font systeme. Adaptations: window.anime UMD; texte FR; reduced-motion = split sans boucle (etat final).

```js
split = splitText('p', { lines: true, debug });
split.addEffect(split => {
  return createTimeline({ defaults: { alternate: true, loop: true, loopDelay: 75, duration: 1500, ease: 'inOutQuad' } })
  .add(split.lines, { color: { from: 'var(--sega-1)' }, y: -10, scale: 1.1 }, stagger(100, { start: 0 }))
  .add(split.words, { scale: [.98, 1.04] }, stagger(100, { use: 'data-line', start: 0 }))
  .init()
});
split.addEffect(split => {
  split.words.forEach(($el, i) => { const c = coords[i]; if (c) utils.set($el, { x: c.x, y: c.y }); });
  return () => { split.words.forEach((w, i) => coords[i] = { x: utils.get(w, 'x'), y: utils.get(w, 'y') }); }; // cleanup persists coords
});
```

---

## text/split-playground

- **Source** : `examples/text/split-playground/index.js`
- **Idée** : Demontre TOUTE la surface de splitText : split simultane lines+words+chars avec options wrap:'clip'/'visible', clone:'top\|bottom\|left\|right' (duplique le glyphe pour un effet de defilement masque par overflow), includeSpaces, accessible (aria). Le clone:dir choisit la direction d'animation (x/y +/-100%) ; un re-split est declenche par diff JSON dans une boucle rAF, et tout attend document.fonts.ready avant le 1er split (le metrics de split depend des fonts chargees).
- **Tags** : `split-text`, `split-clone-direction`, `wrap-clip-overflow`, `fonts-ready-gate`, `raf-dirty-check-resplit`, `scrolling-glyph`, `accessible-split`
- **Primitives** : `splitText({lines/words/chars: {wrap,clone,accessible,includeSpaces}})`, `split.addEffect`, `split.revert`, `animate(loop,alternate)`, `stagger(from:'random')`, `document.fonts.ready`, `rAF dirty-check re-split`
- **Empreinte** : prims{splitText(lines+words+chars,wrap:clip,clone:dir,includeSpaces,accessible),split.addEffect,split.revert,animate(loop,alternate),stagger(from:random),document.fonts.ready,rAF}+struct{triple-niveau split, clone:dir=glyphe duplique scroll sous overflow clip, gate fonts.ready avant 1er split, re-split sur diff-JSON en rAF}
- **SBFB** : ✅ — Le coeur splitText (wrap/clone/clip) est rejouable et conforme piege #5. document.fonts.ready marche en iframe. MAIS retirer le bloc tweaks GUI + importmap ESM (c'est un playground de debug). Si on utilise une police custom, l'embarquer en local (pas Google Fonts). Adaptations: window.anime UMD; texte FR; gate prefers-reduced-motion (split statique sans loop). La technique clone:dir+wrap:clip = scroll de glyphe sans box-shadow ni canvas, parfaitement scellable.

```js
const animateSplit = (targets, opts) => {
  const dir = opts.clone;
  return animate(targets, {
    x: dir === 'left' ? '100%' : dir === 'right' ? '-100%' : 0,
    y: dir === 'top' ? '100%' : dir === 'bottom' ? '-100%' : !dir ? '-100%' : 0,
    loop: true, alternate: true, duration: opts.duration,
    delay: stagger(opts.stagger==='random' ? 10 : +opts.stagger, { from: opts.stagger==='random' ? 'random' : 0 }),
  });
};
document.fonts.ready.then(() => {           // split metrics depend on loaded fonts
  split = splitText('article', { lines, words, chars, includeSpaces, accessible, debug });
  // ... clone:'top'/'clip' duplicates each glyph, overflow-clipped for scroll effect
});
```

---

## text/hover-effects

- **Source** : `examples/text/hover-effects/index.js`
- **Idée** : Six effets hover empaquetes chacun dans createScope (root scopant les selecteurs + methods nommees onEnter/onLeave reliees aux pointerenter/leave). Truc structurel cle : on cree une timeline autoplay:false UNE fois puis on l'avance/recule par animate(tl,{progress:1/0}) au survol (scrub d'une timeline par une autre animation). Le split clone:'left'/'top'+wrap:'clip' fait glisser un glyphe duplique sous overflow clip ; le 3D-word utilise un template splitText multi-faces (rotateX -180 + opacities par face).
- **Tags** : `createScope`, `scoped-methods`, `split-text`, `scrub-timeline-via-progress`, `spring-ease`, `clone-clip-glyph-slide`, `3d-word-faces-template`, `data-attr-stagger`, `additive-blend-hover`
- **Primitives** : `createScope({root,defaults})`, `scope.add('method',fn)+scope.methods`, `splitText(chars:{class,clone:'left/top',wrap:'clip'})`, `splitText(words:'<span>{value}</span>' template)`, `createSpring({stiffness,damping})`, `createTimeline(autoplay:false)`, `animate(tl,{progress})`, `stagger(use:'data-char'/'data-word', from:'random')`, `composition:'blend'`, `seek(1000)`
- **Empreinte** : prims{createScope(root,methods),splitText(chars/words template,clone,wrap:clip),createSpring,createTimeline(autoplay:false),animate(tl,{progress}),stagger(use:data-char,from:random),composition:blend,seek}+struct{6 effets scopes, scrub timeline via animate-progress 0<->1 au hover, glyphe clone+clip slide, mots 3D template multi-face rotateX, spring(stiffness,damping)}
- **SBFB** : ✅ — createScope/scope.methods/scrub-via-progress/3D-faces sont 100% locaux et tres reutilisables. PIEGE #6: createSpring({...}) deprecie v4.5 -> spring({stiffness,damping}). RETIRER le Google Fonts (preconnect+css2 = CDN INTERDIT) : utiliser une police systeme ou locale (le glyphe JP '3Dで単語' demande une police a glyphes CJK -> remplacer par texte FR pour rester local). Adaptations: window.anime UMD; spring(); strings FR; reduced-motion = pas de hover-anim (etat final).

```js
createScope({ root: '#horizontal-split', defaults: { ease: 'outQuad', duration: 500 } }).add((scope) => {
  const { root, methods } = scope;
  splitText('h2', { chars: { class: 'char', clone: 'left', wrap: 'clip' } });
  const rotateAnim = createTimeline({ autoplay: false, defaults: { ease: 'inOutQuad', duration: 400 } })
    .add('.char > span', { x: '100%' }, stagger(5, { use: 'data-char' }));
  scope.add('onEnter', () => animate(rotateAnim, { progress: 1 })); // scrub timeline via progress
  scope.add('onLeave', () => animate(rotateAnim, { progress: 0 }));
  root.addEventListener('pointerenter', methods.onEnter);
  root.addEventListener('pointerleave', methods.onLeave);
});
```

---

## animejs-v4-logo-animation

- **Source** : `examples/animejs-v4-logo-animation/index.js`
- **Idée** : Animation logo cinematique entierement SVG : morph en chaine svg.morphTo entre paliers ('#line-0' -> line-0-1..6 -> line-1..5) pour un liquide qui tombe et rebondit, svg.createDrawable+draw:'0 1' pour tracer les traits j/s, animation de filtre feGaussianBlur (stdDeviation '15,15'->'0,0') pour un focus-in, et un effet 'machine a sous' textuel ou textContent est tween via un INDEX dans une chaine de chars (to:[0,chars.indexOf(c)] + modifier qui mappe la valeur arrondie au glyphe). Onion-skin par cloneNode anime avec opacity stagger.
- **Tags** : `svg-morph-chain`, `motion-blur-fegaussianblur`, `svg-draw-stroke`, `textContent-charset-scrub`, `onion-skin-clones`, `label-relative-choreography`, `custom-cubicbezier-ease`, `splash-keyframes`
- **Primitives** : `createTimeline`, `stagger`, `svg.morphTo`, `svg.createDrawable`, `draw:'0 1'`, `cubicBezier`, `eases.outElastic`, `feGaussianBlur stdDeviation tween`, `textContent tween (charset index)`, `tl.label + relative positions '<<'/'<+='/'-='`, `cloneNode onion-skin`
- **Empreinte** : prims{createTimeline(labels),svg.morphTo(chain),svg.createDrawable+draw,feGaussianBlur:stdDeviation-tween,textContent:to[0,idx]+modifier,cubicBezier,eases.outElastic,stagger(from:center),cloneNode}+struct{morph multi-palier liquide, blur-focus-in via filter, draw stroke j/s, slot-machine textContent index-mapped-to-charset, onion-skin clones opacity-stagger, choreographie par labels relatifs}
- **SBFB** : ✅ — Tout est local-SVG, aucun reseau. Conforme piege #4 : svg.morphTo paire bien path-d<->path-d (#a-1->#a-2) et polygon-points<->polygon (#i-1->#i-2) — meme type d'element. feGaussianBlur est anime sur un FILTRE SVG (pas box-shadow CSS) donc OK vs piege #2. ATTENTION: l'effet 'slot-machine' tween textContent (pas innerHTML) directement, le texte est wrappe en <span> AVANT (conforme #5). Adaptations: window.anime UMD (svg.*/eases/cubicBezier exposes sous window.anime); le SVG inline doit peindre via CSS var(--color-*) pas fill-*/stroke-* Tailwind (piege #3); reduced-motion = tl.seek(tl.duration) etat final.

```js
tl.add('#line-0', {
  d: [
    { to: svg.morphTo('#line-0-1', 0), delay: 320, duration: 60, ease: 'inQuad' },
    { to: svg.morphTo('#line-0-2', 0), duration: 80 },
    { to: svg.morphTo('#line-0-3', 0), duration: 90 },
  ],
});
// slot-machine: tween textContent through a charset index
function wrapInSpan(sel){ const t=document.querySelector(sel); t.innerHTML=[...t.textContent].map(c=>`<span>${c===' '?'&nbsp;':c}</span>`).join(''); }
wrapInSpan('#sub-text');
tl.add('#sub-text span', {
  textContent: {
    to: $el => [0, chars.indexOf($el.textContent)],
    modifier: v => { const c = chars[utils.round(v, 0)]; return c ? c : ' ' },
  },
  delay: stagger(30, { from: 'center', ease: 'inOut(2)' }),
}, 'TEXT')
.add('#blur feGaussianBlur', { stdDeviation: ['15,15', '0,0'], ease: 'out(2)', duration: 1000 }, '<<');
```

---

## timeline-50K-stars

- **Source** : `examples/timeline-50K-stars/index.js`
- **Idée** : Une timeline maitre 'scenarise' un faux compteur GitHub qui grimpe de 1 a 50000 : elle pilote la PROGRESSION (progress:1) d'une sous-timeline de clic en boucle ET un objet de donnees `data.mult` keyframe pour moduler l'intensite, pendant qu'un compteur innerHTML est anime via des courbes cubicBezier extremes (1,0,1,1 / 0,1,0,1) qui simulent l'acceleration/saturation virale.
- **Tags** : `timeline-as-director`, `animate-child-progress`, `data-object-keyframes`, `innerHTML-counter`, `cubic-bezier-extreme`, `label-positioning`, `particle-spawn`, `loop-refresh`
- **Primitives** : `createTimeline`, `animate`, `utils.$`, `utils.set`, `utils.random`, `utils.round`, `steps`, `cubicBezier`, `tl.add(animation,{progress:1})`, `tl.add(data,{...modifier})`, `.call`, `.label`, `.set`, `loop`, `onLoop:self.refresh()`
- **Empreinte** : prims{animate,createTimeline,cubicBezier,steps,utils.random,utils.round}\|struct:master-tl-drives-childTL-progress+data-obj-modifier-keyframes+innerHTML-count-rampe-via-bezier-saturant+spawn-particules-clones
- **SBFB** : ✅ — 100% DOM/transform/opacity/color + innerHTML compteur, 0 reseau (donnees deja simulees: faux compteur scenarise). Charger via window.anime UMD; destructurer anime.createTimeline/animate/utils/steps/cubicBezier. Le sprite curseur (cursor.png) est un asset local relatif: OK. Aucune box-shadow animee. Le compteur 'views' est exactement le genre de donnee SIMULEE attendu en iframe.

```js
.add(clickAnimation, {
  progress: 1,
  duration: 10000,
  ease: cubicBezier(.65,0,0,1),
}, 'CLICK START')
.add($count, {
  innerHTML: ['5', '40000'],
  modifier: utils.round(0),
  ease: cubicBezier(1,0,1,1),
  duration: 5000
}, 'CLICK START+=800')
.add($count, {
  innerHTML: '49999',
  modifier: utils.round(0),
  ease: cubicBezier(0,1,0,1),
  duration: 4250
}, '<')
.add(data, {
  mult: [0, 0, 1.5, .25, 0, 0],
  duration: 10000,
  ease: cubicBezier(1,0,1,1),
}, 'CLICK START')
```

---

## timeline-refresh-starlings

- **Source** : `examples/timeline-refresh-starlings/index.js`
- **Idée** : Murmuration de 2500 elements: chaque div porte ses params polaires (theta,radius) dans des cles Symbol() pour ne pas polluer le dataset; les positions sont des FONCTIONS lues par element, et onLoop re-randomise theta/radius PUIS appelle self.refresh() pour relire les valeurs-fonctions — donc une boucle qui ne se repete jamais. Une cible-attracteur (objet `target`) est animee en parallele avec un modifier sinusoidal sur tl.currentTime pour un mouvement de groupe organique.
- **Tags** : `per-element-function-values`, `loop-refresh-rerandomize`, `symbol-prop-storage`, `attractor-object`, `currentTime-modifier-sine`, `polar-positioning`, `stagger-temporal`, `boids-illusion`
- **Primitives** : `createTimeline`, `stagger`, `utils.set`, `utils.random`, `onLoop:self.refresh()`, `loop:true`, `modifier`, `tl.currentTime`, `function-values($el=>...)`, `Symbol()-as-prop-key`
- **Empreinte** : prims{createTimeline,stagger,utils.random,utils.set,modifier,onLoop-refresh}\|struct:per-el-fn-values-polaires+onLoop-rerandomize-puis-refresh+attractor-obj-anime-en-parallele-modifier-sin(currentTime)
- **SBFB** : ✅ — Pur DOM (2500 divs translate x/y) + objet JS attracteur, 0 reseau. 2500 elements est lourd mais reste raisonnable en iframe; reduire le count si besoin. Aucun piege touche. La cle = function-values + onLoop refresh, parfaitement rejouable. Honorer prefers-reduced-motion en posant l'etat final via seek() sans loop.

```js
tl.add('div', {
  x: $el => target.x + ($el[radius] * cos($el[theta])),
  y: $el => target.y + ($el[radius] * sin($el[theta])),
  duration: () => duration + utils.random(-100, 100),
  ease: 'inOut(1.5)',
  onLoop: self => {
    const t = self.targets[0];
    t[theta] = random() * PI * 2;
    t[radius] = target.r * sqrt(random());
    self.refresh();
  },
}, stagger((duration / count) * 1.125))
.add(target, {
  x: () => utils.random(-win.w, win.w),
  modifier: x => x + sin(tl.currentTime * .0007) * (win.w * .65),
  duration: 2800,
}, 0)
```

---

## timeline-seamless-loop

- **Source** : `examples/timeline-seamless-loop/index.js`
- **Idée** : Boucle parfaitement seamless de 500 elements disposes en cercle: le truc est d'echantillonner un stagger([0,1]) UNE fois comme 'fonction de force' par index (strengthFn), puis de baker rest/peak positions, hues, scales a partir de cette force. Chaque element anime rest->peak->rest (keyframes a 2 segments) avec un loopDelay = loopDuration - animDuration, decale par stagger, donnant une onde qui circule sans coupure.
- **Tags** : `seamless-loop`, `stagger-as-strength-function`, `baked-rest-peak-positions`, `loopDelay-gap`, `two-segment-keyframes`, `radial-layout`, `wave-propagation`, `vh-units`
- **Primitives** : `createTimeline`, `stagger`, `stagger([0,1],{ease,reversed,from:'center'})-as-strength-fn`, `utils.set`, `utils.round`, `utils.degToRad`, `loopDelay`, `loop:-1`, `keyframe-segments [{to},{to}]`, `tl.seek(0)`
- **Empreinte** : prims{createTimeline,stagger,utils.set,utils.degToRad,loopDelay,loop:-1,keyframe-2seg}\|struct:stagger([0,1])-echantillonne-en-strengthFn+bake-rest/peak+loopDelay=loopDur-animDur+onde-circulaire-seamless
- **SBFB** : ✅ — Pur DOM transform (translateX/Y en vh, rotate, scale, backgroundColor hsl), 0 reseau. Le mecanisme seamless-loop via loopDelay+stagger est exactement reutilisable. Aucun piege. Les unites vh fonctionnent dans l'iframe (viewport = taille iframe). prefers-reduced-motion: poser le rest layout via utils.set et ne pas demarrer la tl.

```js
const strengthFn = stagger([0, 1], { ease: 'inOutSine', reversed: true, from: 'center' });
const strengths = els.map((el, i) => utils.round(+strengthFn(el, i, els), 100));
const restPositions = angles.map(a => posOnCircle(a, RADIUS_VH));
const peakPositions = angles.map((a, i) => posOnCircle(a + 10 * strengths[i], RADIUS_VH * 1.1));
// ...
const tl = createTimeline({
  defaults: { ease: 'inOut(2)', loopDelay: loopDuration - animDuration, duration: animDuration },
})
.add(els, {
  translateX: [ { to: (_, i) => peakPositions[i].x + 'vh' }, { to: (_, i) => restPositions[i].x + 'vh' } ],
  translateY: [ { to: (_, i) => peakPositions[i].y + 'vh' }, { to: (_, i) => restPositions[i].y + 'vh' } ],
  loop: -1,
}, stagger(delay, { start: 0 }));
```

---

## timeline-stress-test

- **Source** : `examples/timeline-stress-test/index.js`
- **Idée** : Stress-test de 2024 elements en spirale: une seule timeline avec un stagger TEMPOREL etale sur toute la duree (stagger([0,duration])) — chaque element demarre a un instant different — combinee a utils.mapRange pour transformer l'index en angle de spirale (0..PI*100). Le `playbackEase` global re-time toute la lecture. Demontre que 2000+ animations sur une timeline restent fluides via le moteur unifie.
- **Tags** : `stress-test`, `stagger-over-full-duration`, `mapRange-index-to-angle`, `spiral-layout`, `playbackEase-global`, `scale-keyframe-array`, `hsl-rainbow-by-index`
- **Primitives** : `createTimeline`, `stagger`, `stagger([0,duration])`, `utils.mapRange`, `utils.round`, `utils.set`, `playbackEase`, `loop:true`, `keyframe-array scale:[0,.4,.2,.9,0]`, `.init`, `.seek`
- **Empreinte** : prims{createTimeline,stagger,utils.mapRange,utils.round,playbackEase,loop:true}\|struct:2000+el-une-seule-tl+stagger([0,duration])-temporel-total+mapRange(i->angle-spirale)+playbackEase-global
- **SBFB** : ✅ — Pur DOM (x/y en rem, scale, hsl background), 0 reseau. 2024 elements peut etre lourd — ajuster le count pour iframe modeste. Le pattern stagger([0,duration])+mapRange+playbackEase est directement rejouable via UMD. Aucun piege. Reduced-motion: seek a un etat stable, pas de loop.

```js
const angle = utils.mapRange(0, count, 0, Math.PI * 100);
// ...
createTimeline()
.add('div', {
  x: (_, i) => `${Math.sin(angle(i)) * distance}rem`,
  y: (_, i) => `${Math.cos(angle(i)) * distance}rem`,
  scale: [0, .4, .2, .9, 0],
  playbackEase: 'inOutSine',
  loop: true,
  duration,
}, stagger([0, duration]))
.init()
.seek(10000);
```

---

## svg-graph

- **Source** : `examples/svg-graph/index.js`
- **Idée** : Graphe d'analytics: une barre-masque (rect #b) balaye de gauche a droite (x:0->900, width:900->0) pour 'reveler' la courbe area sous un masque SVG, synchronisee a un compteur innerHTML qui s'incremente depuis 0 avec modifier toLocaleString() pour le formatage milliers. La position '<<' verrouille le compteur sur le debut du reveal — un wipe de revelation propre.
- **Tags** : `mask-wipe-reveal`, `rect-sweep`, `innerHTML-counter-locale`, `modifier-toLocaleString`, `timeline-position-sync`, `svg-area-chart`, `previous-start-position`
- **Primitives** : `createTimeline`, `animate(implicite-via-add)`, `svg(import)`, `stagger(import-inutilise)`, `utils.round`, `innerHTML:{from:0}`, `modifier:v=>round.toLocaleString()`, `x/width-rect-animation`, `loop:true`, `position '<<' '+=500'`, `.init`
- **Empreinte** : prims{createTimeline,svg,utils.round,innerHTML-from,modifier-locale}\|struct:rect-mask-sweep(x+width)-revele-area-chart+compteur-innerHTML-toLocaleString-sync-via-'<<'
- **SBFB** : ✅ — SVG inline + animation de rect (x/width) + innerHTML, 0 reseau (donnees graphe en dur = simulees). ATTENTION piege #3: ici les couleurs SVG sont en attributs (stroke=#B7FF54, fill=url(#c)) PAS via classes Tailwind fill-/stroke-, donc deja conforme; si on refactor en CSS, utiliser var(--color-*) sur fill/stroke. Le path area est statique, seul le masque bouge. Rejouable via UMD.

```js
.add('#b', {
  x: [0, 0],
  width: [0, 900],
}, 0)
.add('#count', {
  innerHTML: { from:  0 },
  modifier: v => utils.round(v, 0).toLocaleString(),
}, '<<')
.add('#b', {
  x: 900,
  width: 0,
  duration: 1500,
}, '+=500')
.add('#views', {
  opacity: 0,
  duration: 1500,
}, '<<')
```

---

## svg-line-drawing

- **Source** : `examples/svg-line-drawing/index.js`
- **Idée** : 100 lignes + 50 cercles concentriques generes en string SVG puis injectes; l'animation `draw` (attribut anime de stroke-dashoffset/array via createDrawable) prend des KEYFRAMES de fractions de trace ('.5 .5' = invisible au centre -> '.05..0.95' = revele -> retour), avec des bornes randomisees par fonction a chaque keyframe. Le stagger([0,8000]) etale le dessin dans le temps pour une vague de trace qui s'ouvre et se referme.
- **Tags** : `svg-draw-keyframes`, `createDrawable`, `partial-stroke-reveal`, `random-draw-bounds`, `stagger-temporal-spread`, `procedural-svg-injection`, `concentric-circles`, `vertical-lines-grid`
- **Primitives** : `svg.createDrawable`, `createTimeline`, `stagger`, `stagger([0,8000],{start,from:'first'})`, `utils.random`, `draw:['.5 .5', fn, '0.5 0.5']`, `loop:true`, `string-template-svg-injection`, `.init`
- **Empreinte** : prims{svg.createDrawable,createTimeline,stagger,utils.random,draw-keyframes,loop:true}\|struct:SVG-genere-en-string+draw-keyframes-fractions('.5 .5'->random->'.5 .5')+stagger([0,8000])-vague-de-trace
- **SBFB** : ✅ — createDrawable + draw keyframes = pur SVG stroke-dash, 0 reseau. ATTENTION piege #3: les stroke sont poses en attribut stroke=#A4FF4F (conforme); si CSS, var(--color-*). L'injection via innerHTML += de strings SVG fonctionne en iframe (unsafe-inline OK pour le CSS, mais l'injection est du SVG markup, pas de script). createDrawable est expose sur anime.svg en UMD. Rejouable.

```js
.add(svg.createDrawable('.line-v'), {
  draw: [
    '.5 .5',
    () => { const l = utils.random(.05, .45, 2); return `${.5 - l} ${.5 + l}` },
    '0.5 0.5',
  ],
  stroke: '#FF4B4B',
}, stagger([0, 8000], { start: 0, from: 'first' }))
.add(svg.createDrawable('.circle'), {
  draw: [
    () => { const v = utils.random(-1, -.5, 2); return `${v} ${v}`},
    () => `${utils.random(0, .25, 2)} ${utils.random(.5, .75, 2)}`,
    () => { const v = utils.random(1, 1.5, 2); return `${v} ${v}`},
  ],
  stroke: '#FF4B4B',
}, stagger([0, 8000], { start: 0 }))
```

---

## onscroll-responsive-scope

- **Source** : `examples/onscroll-responsive-scope/index.js`
- **Idée** : Le MEME effet scroll-lie a deux chorEographies differentes selon l'orientation: createScope avec mediaQueries.landscape branche conditionnellement l'animation (eventail rotatif en paysage vs cascade verticale en portrait), et onScroll(...).link() attache la PROGRESSION de l'animation choisie au scroll (sync:.1 = smoothing). Le scope re-evalue automatiquement au changement de media query.
- **Tags** : `responsive-scope`, `media-query-branch`, `scroll-linked-progress`, `onScroll-link`, `scroll-sync-smoothing`, `stagger-from-center`, `stagger-delay-cascade`, `vw-vh-from-values`
- **Primitives** : `createScope`, `createScope({mediaQueries})`, `scope.matches.landscape`, `onScroll`, `onScroll({enter,leave,sync}).link(anim)`, `animate`, `stagger`, `stagger(['-40vh','40vh'],{from:'center'})`, `stagger-as-delay`, `from-value-function`
- **Empreinte** : prims{createScope,onScroll-link,animate,stagger,scope.matches}\|struct:createScope-mediaQueries-branche-2-choreos+onScroll(enter/leave/sync).link(anim-conditionnel)+responsive-auto-reeval
- **SBFB** : ✅ — Le scroll EST local au document de l'iframe (sticky-container scrollable), 0 reseau, cartes = assets SVG relatifs. createScope+onScroll exposes sur window.anime UMD. ATTENTION: l'iframe doit etre assez haute/scrollable pour declencher enter/leave; sinon proposer une variante autoplay ou un faux scroll programmatique. mediaQueries fonctionnent sur la taille de l'iframe. Reduced-motion: scope honore prefers-reduced-motion via etat final.

```js
createScope({
  mediaQueries: { landscape: '(orientation: landscape)' },
  defaults: { ease: 'out(3)', duration: 500 },
}).add((scope) => {
  let cardsAnimation;
  if (scope.matches.landscape) {
    cardsAnimation = animate('.card', {
      y: { from: stagger(['-40vh','40vh'], {from: 'center'}) },
      rotate: { to: stagger([-30, 30]), delay: stagger([0, 950], { from: 'last', start: 200 }) },
      x: ['-60vw', stagger(['-20%', '20%'])],
    });
  } else {
    cardsAnimation = animate('.card', { y: ['150vh', stagger(['20%', '-20%'])] });
  }
  onScroll({ target: '.sticky-container', enter: 'top', leave: 'bottom', sync: .1 }).link(cardsAnimation)
});
```

---

## onscroll-sticky

- **Source** : `examples/onscroll-sticky/index.js`
- **Idée** : Un deck de 52 cartes 3D 'eventaille' au scroll: chaque carte est enveloppee runtime dans un .spinner (transformOrigin pivot bas), et onScroll est passe DIRECTEMENT comme autoplay de la timeline (le scroll devient l'horloge). composition:'blend' (additive) permet aux animations hover de s'AJOUTER a la pose scroll sans la casser. stagger modifier applique un brightness() degrade recto/verso pour la profondeur.
- **Tags** : `scroll-as-autoplay-clock`, `additive-blend-composition`, `3d-card-spinner`, `transform-origin-pivot`, `stagger-rotate-fan`, `filter-brightness-stagger`, `hover-blend-on-scroll`, `runtime-wrap-element`
- **Primitives** : `createTimeline`, `onScroll-as-autoplay`, `autoplay:onScroll({enter,leave,sync})`, `composition:'blend'`, `stagger`, `stagger([0,-360],{from:'last'})`, `utils.set`, `utils.$`, `utils.random`, `filter:brightness-via-stagger-modifier`, `animate-onmouseenter`
- **Empreinte** : prims{createTimeline,onScroll-autoplay,composition:blend,stagger,utils.set,animate-hover}\|struct:onScroll-passe-comme-autoplay(scroll=horloge)+composition:blend-hover-s-ajoute-a-pose-scroll+spinner-wrap-3D+stagger-rotate-fan
- **SBFB** : ✅ — Scroll local, transforms 3D (rotateY/rotateZ/translateY), filter brightness, 0 reseau, cartes = SVG relatifs. ATTENTION piege #2: la .card a une box-shadow STATIQUE en CSS (jamais animee) — conforme. composition:'blend' = additive blend natif, rejouable via UMD. onScroll-as-autoplay fonctionne tant que l'iframe scrolle (sticky-container 400lvh). 52 cartes OK. Reduced-motion: poser etat final, ne pas lier au scroll.

```js
createTimeline({
  defaults: { ease: 'linear', duration: 500, composition: 'blend' },
  autoplay: onScroll({ target: '.sticky-container', enter: 'top top', leave: 'bottom bottom', sync: .5 }),
})
.add('.spinner', {
  rotate: 0,
  rotateZ: { to: stagger([0, -360], { from: 'last' }), ease: 'inOut(2)' },
  transformOrigin: ['50% 100%', '50% 50%'],
  delay: stagger(1, { from: 'first' }),
}, 0)
// hover s'ajoute via blend, sans casser la pose scroll:
$card.onmouseenter = () => animate($card, { y: '-70%', duration: 350, composition: 'blend' });
```

---

## irregular-playback-typewriter

- **Source** : `examples/irregular-playback-typewriter/index.js`
- **Idée** : Effet machine a ecrire au rythme HUMAIN imparfait: chaque lettre (span) apparait via .set opacity avec stagger temporel regulier, MAIS le playbackEase:easings.irregular(n,2) re-time toute la lecture de facon saccadee/imprevisible (la frappe accelere et hesite). Un curseur separe avance en easings.steps(n) (saut discret par caractere) et clignote en boucle alternee.
- **Tags** : `irregular-ease`, `playbackEase-retiming`, `typewriter`, `stagger-reveal-letters`, `steps-ease-discrete-cursor`, `blink-loop-alternate`, `human-rhythm-simulation`
- **Primitives** : `createTimeline`, `easings.irregular(steps,randomness)`, `easings.steps`, `playbackEase:irregular`, `stagger`, `.set($spans,{opacity:[0,1]},stagger(interval))`, `animate`, `loop:true,alternate:true`
- **Empreinte** : prims{createTimeline,easings.irregular,easings.steps,stagger,animate-loop-alternate}\|struct:stagger-regulier-lettres+playbackEase:irregular(n,2)-retime-tout-en-saccade+curseur-steps(n)+blink-alternate
- **SBFB** : ✅ — Pur opacity/left sur spans pre-existants, 0 reseau. easings.irregular et easings.steps exposes sur anime.easings (ou anime.eases) en UMD. Texte en dur dans les <span> = facile a passer en FRANCAIS. Aucun piege (pas de splitText ici, les spans sont deja dans le HTML — si on voulait splitText, appliquer piege #5: reposer textContent avant re-split). Rejouable.

```js
const keystrokesSteps = $spans.length - 1;
const keystrokesInterval = 125;
createTimeline({
  playbackEase: easings.irregular(keystrokesSteps, 2),
})
.set($spans, { opacity: [0, 1] }, stagger(keystrokesInterval))
.add($cursor, {
  left: '100%',
  duration: keystrokesSteps * keystrokesInterval,
  ease: easings.steps(keystrokesSteps),
}, 0)
.init();

animate($cursor, { opacity: 0, duration: 750, ease: 'inIn(2)', loop: true, alternate: true });
```

---

## clock-playback-controls

- **Source** : `examples/clock-playback-controls/index.js`
- **Idée** : Horloge digitale 3D ou chaque chiffre est un cylindre de 10 faces (rotateX par face); une masterTL d'une JOURNEE entiere (86.4M ms) synchronise (masterTL.sync) des sous-timelines par roue de chiffres avec des durees/eases differentes selon le chiffre (les dizaines d'heures tournent en cubicBezier avec arret, les secondes en linear). On ANIME masterTL.currentTime et masterTL.speed eux-memes pour seek/slowmo/speedup — la timeline est une cible animable.
- **Tags** : `animate-the-timeline`, `currentTime-as-animatable`, `speed-as-animatable`, `timeline-sync-nested`, `3d-digit-cylinder`, `rotateX-faces`, `onUpdate-bind-controls`, `utils.sync-flush`, `day-length-timeline`
- **Primitives** : `createTimeline`, `masterTL.sync(childTL,0)`, `animate(masterTL,{currentTime})`, `animate(masterTL,{speed})`, `masterTL.onUpdate`, `masterTL.currentTime=`, `masterTL.duration/iterationDuration`, `utils.sync`, `utils.set`, `utils.degToRad`, `cubicBezier`, `utils.$`
- **Empreinte** : prims{createTimeline,masterTL.sync,animate(tl.currentTime),animate(tl.speed),onUpdate,utils.sync,cubicBezier}\|struct:masterTL-1-journee+sync(childTL-par-roue)+animate-currentTime/speed-de-la-timeline-elle-meme+chiffres-cylindre-3D-rotateX
- **SBFB** : ✅ — Pur DOM 3D (rotateX/translateY/z en ch), Date locale (pas de reseau), controles UI sliders/boutons. La cle = animate(masterTL,{currentTime/speed}) + masterTL.sync, tout expose via UMD. new Date() est local, OK en iframe. Labels boutons a passer en FRANCAIS. Aucun piege (transforms only). Rejouable; reduced-motion: figer currentTime sans loop.

```js
const numTL = createTimeline({ defaults: { ease }, loop: true });
// ...add rotateX per face...
masterTL.sync(numTL, 0);
// ...
masterTL.duration = oneday;
masterTL.iterationDuration = oneday;
masterTL.currentTime = getNow();
masterTL.onUpdate = ({currentTime, speed}) => { /* bind to UI */ };
// The timeline itself is the animation target:
animate(masterTL, { currentTime: getNow(), ease: 'inOut(3)', duration: 1500 });
animate(masterTL, { speed: .1, ease: 'out(3)', duration: 1500 });
animate(masterTL, { speed: 5, ease: 'out(3)', duration: 1500 });
```

---

## canvas-2d

- **Source** : `examples/canvas-2d/index.js`
- **Idée** : 4000 particules animees SANS toucher le DOM: anime.js anime des OBJETS JS purs (p.x, p.y, p.radius) avec des durees independantes par axe (proportionnelles a la distance) et un onComplete qui se re-anime a l'infini vers une nouvelle cible. Un createTimer separe est le 'render loop' qui peint le canvas a chaque frame (fade trail via fillRect alpha .1 + globalCompositeOperation 'screen'). anime = moteur de tweening, canvas = sortie.
- **Tags** : `animate-plain-objects`, `createTimer-render-loop`, `canvas-2d-main-thread`, `tween-engine-decoupled-from-render`, `self-perpetuating-onComplete`, `per-axis-duration`, `composite-screen-trail`, `alpha-fade-trail`
- **Primitives** : `animate`, `createTimer`, `createTimer({onUpdate})`, `animate(plain-object,{x,y,radius})`, `onComplete:re-animate`, `utils.random`, `utils.randomPick`, `per-property-duration`
- **Empreinte** : prims{animate,createTimer,utils.random,utils.randomPick,onComplete-recurse}\|struct:anime-tween-objets-JS-purs(x,y,radius)+createTimer-render-loop-canvas2D+onComplete-self-reanime+screen-blend-trail
- **SBFB** : ✅ — Canvas 2D sur le MAIN THREAD (pas de worker), getContext('2d'), 0 reseau, 0 CDN. C'est le pattern le plus important du cluster: anime = tween engine sur objets purs, createTimer = boucle de rendu. ATTENTION: contrainte SBFB interdit 'canvas via worker' — ici c'est main-thread, donc CONFORME. 4000 particules ajustables. Rejouable via window.anime UMD (animate/createTimer/utils).

```js
function animateParticule(p, i) {
  const newX = utils.random(0, viewport.width);
  const durX = Math.abs((newX - p.x) * 20);
  animate(p, {
    x: { to: newX, duration: durX },
    y: { to: utils.random(0, viewport.height), duration: /*durY*/ },
    radius: utils.random(2, 6),
    ease: 'out(1)',
    onComplete: () => { animateParticule(p, i); }
  });
}
createTimer({
  onUpdate: self => {
    ctx.globalAlpha = .1; ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, viewport.width, viewport.height);
    ctx.globalAlpha = 1; ctx.globalCompositeOperation = 'screen';
    for (let i = 0; i < maxParticules; i++) drawParticule(particules[i]);
  },
})
```

---

## draggable-infinite-auto-carousel

- **Source** : `examples/draggable-infinite-auto-carousel/index.js`
- **Idée** : Carrousel infini auto-defilant: le contenu est duplique (innerHTML+=innerHTML) et createAnimatable applique un modifier utils.wrap(v, -width/2, 0) qui boucle la position en continu (illusion d'infini). Un createTimer combine 3 sources de vitesse dans une seule equation: vitesse auto constante + draggable.deltaX (drag) + delta molette lerpe — toutes additionnees par frame. onGrab/onRelease animent la vitesse auto vers 0 et retour.
- **Tags** : `infinite-carousel`, `utils.wrap-modifier`, `createAnimatable`, `multi-source-velocity-sum`, `createTimer-drive`, `content-duplication-illusion`, `wheel-lerp-smoothing`, `drag-delta-injection`, `spring-release`
- **Primitives** : `createDraggable`, `createAnimatable`, `createAnimatable({x,modifier:utils.wrap})`, `createTimer`, `animate`, `utils.wrap`, `utils.lerp`, `draggable.deltaX`, `onGrab/onRelease`, `releaseStiffness`, `velocityMultiplier`
- **Empreinte** : prims{createDraggable,createAnimatable,createTimer,utils.wrap,utils.lerp,animate}\|struct:innerHTML-double+createAnimatable(modifier:wrap)-boucle-infinie+timer-somme-vitesses(auto+drag.deltaX+wheel.lerp)/frame
- **SBFB** : ✅ — Pointer drag + wheel events sont locaux a l'iframe (allow-scripts suffit, pas besoin de reseau), pur transform x. 0 reseau. createDraggable/createAnimatable/createTimer/utils.wrap tous exposes UMD. Items du carrousel = contenu local (texte FR/images relatives). Aucun piege. Rejouable. Note: e.preventDefault sur wheel avec {passive:false} OK en iframe.

```js
$carousel.innerHTML += $carousel.innerHTML; // illusion d'infini
const animatable = createAnimatable($carousel, {
  x: 0, modifier: v => utils.wrap(v, -carousel.width / 2, 0)
});
const { x } = animatable;
const draggable = createDraggable(carousel, {
  trigger: '#infinite-carousel', y: false,
  onGrab: () => animate(carousel, { speedX: 0, duration: 500 }),
  onRelease: () => animate(carousel, { speedX: 2, duration: 500 }),
  releaseStiffness: 20, velocityMultiplier: 1.5
});
createTimer({
  onUpdate: () => { x(x() - carousel.speedX + draggable.deltaX - carousel.wheelX - carousel.wheelY); }
});
```

---

## draggable-mouse-scroll-snap-carousel

- **Source** : `examples/draggable-mouse-scroll-snap-carousel/index.js`
- **Idée** : Carrousel bornE avec snap, pilote a la fois par drag ET molette: deux createTimer (un debounce 30fps pour detecter l'etat 'wheeling', un autre pour injecter la velocite molette dans le draggable). Le truc avance: il manipule directement l'INTERNE du draggable (draggable.pointer[], computeVelocity, handleUp) pour traduire un geste molette en geste de drag synthetique, avec friction hors-bornes calculee a la main.
- **Tags** : `snap-carousel`, `wheel-to-drag-synthesis`, `draggable-internals-poke`, `computeVelocity`, `debounce-timer`, `out-of-bounds-friction`, `utils.snap`, `css-var-spacing-read`, `frameRate-timer`
- **Primitives** : `createDraggable`, `createDraggable({snap,container,onAfterResize})`, `createTimer({frameRate,duration,autoplay:false})`, `utils.snap`, `utils.lerp`, `utils.get('--spacing')`, `draggable.computeVelocity`, `draggable.pointer[]`, `draggable.handleUp`, `containerFriction`
- **Empreinte** : prims{createDraggable,createTimer,utils.snap,utils.lerp,utils.get-cssvar,draggable.computeVelocity/pointer/handleUp}\|struct:2-timers(debounce-wheeling+velocity-inject)+molette-traduite-en-drag-synthetique-via-internals+snap+friction-hors-bornes-manuelle
- **SBFB** : ✅ — Drag + wheel locaux iframe, transform x, 0 reseau. createDraggable + utils.snap/lerp/get exposes UMD. ATTENTION: ce demo touche des champs INTERNES (draggable.pointer[2..4], computeVelocity, handleUp, coords) potentiellement instables entre versions — fonctionne en v4.5 mais a verifier au pin. utils.get('--spacing') lit une CSS var locale, OK. Items = contenu FR local. Rejouable avec prudence sur l'API interne.

```js
const draggable = createDraggable($carousel, {
  trigger: document.body,
  container: () => [0, 0, 0, -carousel.totalWidth + $carousel.offsetWidth - carousel.spacing],
  x: { snap: () => carousel.itemWidth },
  y: false,
  onAfterResize: self => self.setX(utils.snap(self.x, self.snapX)),
  releaseStiffness: 100, velocityMultiplier: 1.5, containerFriction: .5,
});
const wheelVelocityTimer = createTimer({ duration: 500, autoplay: false, onUpdate: () => {
  const x = /* clamp + friction hors-bornes */;
  draggable.pointer[0] = x;
  draggable.computeVelocity(x - draggable.coords[2], 0);
}});
```

---

## draggable-playground

- **Source** : `examples/draggable-playground/index.js`
- **Idée** : Banc d'essai exhaustif de createDraggable: drawer pilotant la PROGRESS d'une timeline via self.progressY, carrousel 3D ou x est mappe sur rotateY (x:{mapTo:'rotateY'}), liste reordonnable via onSnap qui splice un tableau et anime les voisins en outElastic, ranges qui se rejouent en boucle quand relaches, conteneurs dynamiques dont le padding d'un draggable depend de la position d'un autre. Demonstration que le draggable expose progress/snap/mapTo comme valeurs animables.
- **Tags** : `draggable-progress-drives-timeline`, `x-mapTo-rotateY`, `snap-reorder-list`, `outElastic-neighbors`, `dynamic-container-padding`, `interdependent-draggables`, `engine-timeUnit`, `onSnap-callback`, `animatable-drag-props`, `drawer-physics`
- **Primitives** : `createDraggable`, `createAnimatable`, `createTimer`, `createTimeline`, `animate`, `createSpring(DEPRECIE)`, `eases.outElastic`, `stagger`, `utils.wrap`, `utils.snap`, `utils.set`, `utils.round().clamp()`, `engine.timeUnit`, `x:{mapTo:'rotateY'}`, `x:{snap}`, `container:[]`, `onSnap/onGrab/onRelease/onSettle`, `progressX/progressY`, `draggable.animateInView`
- **Empreinte** : prims{createDraggable,createAnimatable,createTimeline,createSpring,eases.outElastic,utils.wrap/snap,x:mapTo:rotateY,progressX/Y,onSnap}\|struct:playground-draggable+progressY-drive-tl+x-mapTo-rotateY-3D+onSnap-splice-reorder-outElastic+containerPadding-interdependant
- **SBFB** : ✅ — Tout est pointer/transform local, 0 reseau. PIEGE #6 CONFIRME: ce demo utilise createSpring() (DEPRECIE v4.5) en releaseEase — a remplacer par spring({mass,stiffness,damping}) pour SBFB. blockScrolling manipule document.scrollingElement/overflow: fonctionne dans l'iframe mais le hack 'sticky' Safari-mobile est superflu en bac a sable. engine.timeUnit='ms' OK. Le reste (mapTo:'rotateY', onSnap-reorder, progress-drives-timeline) est directement rejouable via UMD.

```js
// PIEGE #6: createSpring DEPRECIE -> remplacer par spring(...)
releaseEase: createSpring({ mass: 1, stiffness: 400, damping: 30 }),
// x mappe sur rotateY pour carrousel 3D:
const carousel = createDraggable('#map-props .carousel', {
  x: { mapTo: 'rotateY' }, y: false, snap: itemAngle, dragSpeed: .4,
});
// drawer dont le drag pilote la progress d'une timeline:
const drawer = createDraggable($drawer, {
  y: { snap: ({ $target }) => $target.offsetHeight }, x: false,
  onUpdate: (self) => { drawerOpenAnim.progress = self.progressY; }
});
// onSnap reordonne une liste + anime les voisins en outElastic:
onSnap: self => {
  const toIndex = utils.round(0).clamp(0, list.length - 1)(self.destY / snap);
  list.forEach((item, i) => { if (i !== toIndex) animate(item, { y: i * snap, ease: eases.outElastic(.8, 1) }); });
}
```

---

## additive-animations-2 (particle spheres, composition:'blend')

- **Source** : `tests/playground/additive-animations-2.html`
- **Idée** : 200 particles/sphere ou chaque particule recoit PLUSIEURS animate() concurrents sur x/y avec composition:'blend' : les tweens additifs se somment au lieu de s'ecraser, donnant un mouvement organique de nuee. Un createTimer loop par particule re-tire une cible toutes les 250-1000ms (handoff entre spheres).
- **Tags** : `additive-blend`, `particle-system`, `per-element-timer`, `random-walk`, `composition-blend`, `pointer-spawn`
- **Primitives** : `animate`, `createTimer`, `createTimeline`, `stagger`, `utils.set`, `utils.random`, `utils.randomPick`, `utils.round`, `composition:'blend'`, `ease:'outElastic'`, `ease:'inOut(2.25)'`
- **Empreinte** : animate+composition:blend+createTimer(loop,onLoop)+stagger \| N concurrent additive tweens per element summed, per-particle re-target timer, pointer spawns new emitter
- **SBFB** : ✅ — Pur DOM+transform, 0 reseau, composition:'blend' est cote moteur anime — rejouable. Swap import ESM->UMD (window.anime.animate/createTimer/stagger/utils). PIEGE 2 a corriger : .ball:before utilise box-shadow STATIQUE pour le glow (deja le cas ici, seul opacity bouge) — conforme. 200 particules*3 spheres peut etre lourd dans l'iframe : reduire maxParticles ~60. prefers-reduced-motion a brancher (etat final statique).

```js
animate(this.$el, {
  x: Math.cos(a) * r + x,
  y: Math.sin(a) * r + y,
  duration: 1100 * speedScale,
  ease: 'inOut(2.25)',
  composition: 'blend'
});
// ...
this.loop = createTimer({
  duration: utils.random(250, 1000) * speedScale,
  loop: true,
  onLoop: () => { this.updatePosition() },
});
```

---

## advanced-staggering-demos (catalogue from/grid/axis/range)

- **Source** : `tests/playground/advanced-staggering-demos.html`
- **Idée** : Banque de reference VISUELLE des modes de stagger cote a cote : from 'first'/'last'/'center'/index numerique, reversed, range [-2em,2em], grid [10,5] avec from numerique=38, axis 'x'/'y' separes, et ease INTERNE au stagger (stagger(...,{ease:'easeOutExpo'}) distribue non-lineairement les delais). Toutes les pistes demarrent a 0 dans une meme timeline alternate+loop.
- **Tags** : `grid-stagger`, `stagger-from`, `stagger-range`, `stagger-axis`, `stagger-ease`, `reversed-stagger`, `reference-catalogue`
- **Primitives** : `createTimeline`, `stagger`, `stagger(value,{from})`, `stagger([a,b])`, `stagger({grid,axis})`, `stagger({reversed})`, `stagger({ease})`, `timeline.add(target,params,0)`
- **Empreinte** : createTimeline.alternate.loop + stagger(from\|grid\|axis\|range\|reversed\|ease) all at position 0 \| side-by-side stagger-mode visual reference matrix
- **SBFB** : ✅ — 100% DOM transform+color, 0 reseau. Le seul ajustement : import ESM->UMD. color:red litteral OK. C'est le meilleur exemple pour comprendre l'espace des parametres stagger — base de gout pour toute grille SBFB.

```js
.add('.grid span', {
  scale: stagger([1, 0], {grid: [10, 5], from: 38}),
  color: red,
  delay: stagger(100, {grid: [10, 5], from: 38})
}, 0)
.add('.grid-axis span', {
  translateX: stagger('.5rem', {grid: [10, 5], from: 38, axis: 'x'}),
  translateY: stagger('.5rem', {grid: [10, 5], from: 38, axis: 'y'}),
  delay: stagger(100, {grid: [10, 5], from: 38})
}, 0)
.add('.ease span', {
  translateY: stagger(['2rem', '-2rem'], {from: 'center', ease: 'easeOutQuad'}),
  delay: stagger([0, 600], {from: 'center', ease: 'easeOutExpo'})
}, 0)
```

---

## stagger-grid-demo (24x24 grid choreography)

- **Source** : `tests/playground/stagger-grid-demo.html`
- **Idée** : 576 divs generes en JS, choreographie multi-segments sur grille 24x24 ou chaque segment combine stagger axial (translateX/Y avec axis:'x'/'y' from 'center') ET des valeurs function-based () => utils.random(-10,10) pour casser la regularite. Mix de tween-keyframe array sur translateX ([{to:stagger(-.1rem)},{to:stagger(.1rem)}]) pour un aller-retour radial avant dispersion aleatoire.
- **Tags** : `grid-stagger`, `grid-choreography`, `function-based-values`, `tween-keyframes`, `procedural-generation`, `axis-stagger`
- **Primitives** : `createTimeline`, `stagger({grid,from,axis})`, `utils.random (function-based value)`, `tween keyframe array [{to},{to}]`, `backgroundColor:{from}`, `init()`
- **Empreinte** : createTimeline.loop + 576 generated divs + stagger(grid,axis,from:center) + ()=>utils.random function-values + per-prop keyframe-array \| large grid multi-segment radial-then-scatter choreography
- **SBFB** : ✅ — Pur DOM, generation procedurale en JS (documentFragment), 0 reseau. 576 elements anime — coute cher mais tient dans l'iframe sur desktop. UMD swap requis. Brancher prefers-reduced-motion sur l'etat repos.

```js
.add('.stagger-visualizer div', {
  translateX: [
    {to: stagger('-.1rem', {grid, from: 'center', axis: 'x'}) },
    {to: stagger('.1rem', {grid, from: 'center', axis: 'x'}) }
  ],
  translateY: [
    {to: stagger('-.1rem', {grid, from: 'center', axis: 'y'}) },
    {to: stagger('.1rem', {grid, from: 'center', axis: 'y'}) }
  ],
  backgroundColor: { from: '#FFF' },
  duration: 1000, scale: .5,
  delay: stagger(100, {grid, from: 'center'})
})
.add('.stagger-visualizer div', {
  translateX: () => utils.random(-10, 10),
  translateY: () => utils.random(-10, 10),
  delay: stagger(8, {from: 'last'})
})
```

---

## sprite-animation (steps() sprite-sheet)

- **Source** : `tests/playground/sprite-animation.html`
- **Idée** : Animation de sprite-sheet pixel-art en animant backgroundPosition de '0px' a '100% 0px' avec ease:steps(9) : le step-easing fige la position sur 9 paliers discrets = 9 frames d'un personnage (Ryu) au lieu d'un glissement continu. La cle est steps() comme courbe d'easing, pas un timer manuel.
- **Tags** : `sprite-sheet`, `step-easing`, `background-position-anim`, `pixel-art`, `frame-stepping`
- **Primitives** : `animate`, `steps(9)`, `backgroundPosition`, `loop:true`
- **Empreinte** : animate + ease:steps(N) + backgroundPosition loop \| discrete frame stepping of a sprite-sheet via step-ease, no manual frame timer
- **SBFB** : ✅ — Trivialement rejouable : 1 element, 1 image locale (chemin relatif assets/media/ryu.png), 0 reseau. Pour SBFB packager le PNG dans l'archive. backgroundPosition est composite-safe. UMD : window.anime.animate + window.anime.steps. Idees produit : indicateur d'etat anime, loader retro.

```js
const ryu = animate('.ryu', {
  backgroundPosition: '100% 0px',
  ease: steps(9),
  duration: 450,
  loop: true
});
```

---

## svg-path-animation (createDrawable line-draw map)

- **Source** : `tests/playground/svg-path-animation.html`
- **Idée** : Trace d'une carte SVG entiere (dizaines de paths colores) : svg.createDrawable('path') cible TOUS les paths, draw passe par keyframes ['0 0','0 1','1 1'] = la ligne se dessine (0->1) puis se 'remplit/efface' (1 1) ; duration randomisee par path + stagger(100) donne un trace organique non synchronise.
- **Tags** : `svg-line-draw`, `createDrawable`, `draw-keyframes`, `stroke-dash-anim`, `stagger`, `random-duration`
- **Primitives** : `createTimeline`, `svg.createDrawable('path')`, `draw:['0 0','0 1','1 1']`, `strokeWidth keyframes`, `stroke:{from}`, `duration:()=>utils.random`, `stagger(100)`
- **Empreinte** : createTimeline + svg.createDrawable(selector-all) + draw:['0 0','0 1','1 1'] + ()=>random duration + stagger \| mass SVG stroke-draw with 3-stage draw keyframes and per-path random timing
- **SBFB** : ✅ — SVG inline, 0 reseau. PIEGE 3 a respecter : ici stroke est pose par attribut SVG (stroke="#31B495"), pas par classe Tailwind — conforme ; pour SBFB peindre via var(--color-*) en CSS si on veut theming, jamais fill-*/stroke-* Tailwind. createDrawable manipule stroke-dasharray/offset (composite-friendly). UMD swap.

```js
createTimeline()
.add(svg.createDrawable('path'), {
  draw: ['0 0', '0 1', '1 1'],
  strokeWidth: [4, 2],
  stroke: { from: '#FFF', duration: 1000 },
  duration: () => utils.random(2000, 4000),
  loop: 2,
  ease: 'inOutSine'
}, stagger(100));
```

---

## svg-morph-timeline (svg.morphTo chains, path & polygon)

- **Source** : `tests/playground/svg-morph-timeline/index.js`
- **Idée** : Chaines de morph A->B->C->D et A->B->C->A (boucle) en sequencant des add() qui chacun fait d: svg.morphTo('#shape-x'). Demontre morph entre MEME type d'element (path d<->path d, polygon points<->polygon points) et le retour au shape initial pour boucler proprement.
- **Tags** : `svg-morph`, `morphTo`, `shape-interpolation`, `sequential-timeline`, `seamless-loop`, `polygon-morph`
- **Primitives** : `createTimeline`, `svg.morphTo('#shape')`, `d: morphTo`, `points: morphTo (polygon)`, `timeline sequential add`, `loop:true`
- **Empreinte** : createTimeline.loop + svg.morphTo (d for path, points for polygon) chained sequential adds back-to-origin \| same-element-type shape morph chain with loop-close
- **SBFB** : ✅ — Shapes en <defs>, 0 reseau. PIEGE 4 respecte : morphTo path->path et polygon->polygon (jamais path<->polygon), sous-traces fermees uniques. Pour boucler, dernier add revient au shape A. UMD : window.anime.svg.morphTo. Cas d'usage SBFB : icone d'etat qui se transforme (check/croix/sablier).

```js
const tlLoop = createTimeline({ loop: true, defaults: { duration, ease: 'inOutQuad' } });
tlLoop.add('#morph-loop', { d: svg.morphTo('#shape-b') });
tlLoop.add('#morph-loop', { d: svg.morphTo('#shape-c') });
tlLoop.add('#morph-loop', { d: svg.morphTo('#shape-a') }); // back to origin = clean loop

const tlPoly = createTimeline({ loop: true, defaults: { duration, ease: 'inOutQuad' } });
tlPoly.add('#morph-poly', { points: svg.morphTo('#poly-b') });
tlPoly.add('#morph-poly', { points: svg.morphTo('#poly-c') });
```

---

## svg-motion-path (createMotionPath responsive, spread params)

- **Source** : `tests/playground/svg-motion-path/index.js`
- **Idée** : Un meme createMotionPath('#path') est SPREAD (...) dans les params pour piloter SIMULTANEMENT un element DOM absolu ET un <rect> SVG le long du chemin, en testant 3 contextes de mise a l'echelle (no width / specified width / preserveAspectRatio). createMotionPath retourne {translateX, translateY, rotate} pretes a etaler.
- **Tags** : `motion-path`, `createMotionPath`, `param-spread`, `responsive-svg`, `multi-target`, `follow-path`
- **Primitives** : `animate`, `svg.createMotionPath('#path')`, `spread ...svg.createMotionPath`, `multi-target animate (DOM el + SVG rect)`, `ease:'linear' loop`
- **Empreinte** : animate + ...svg.createMotionPath(selector) spread on mixed DOM+SVG targets \| single motion-path driving heterogeneous targets, responsive viewBox scaling
- **SBFB** : ✅ — 0 reseau. PIEGE 1 CRITIQUE : tout element deplace par createMotionPath doit avoir cx="0" cy="0" (le translate de la motion-path s'ajoute a la geometrie) — ici le <rect> est pose a x=-10 y=-10 centre sur 0, et le .dom-el est top:-1rem left:-1rem (offset negatif = meme intention). UMD : window.anime.svg.createMotionPath.

```js
animate(['.specified-width .dom-el', '.specified-width .rect-el'], {
  duration: 3000,
  loop: true,
  ease: 'linear',
  ...svg.createMotionPath('#specifiedWidth')
});
```

---

## animejs-v3-logo (orchestrated morph + motion-path + spring)

- **Source** : `tests/playground/animejs-v3-logo-animation.html`
- **Idée** : Animation logo cinematographique : une balle (.dot) suit une motion-path (bouncePath.translateX/Y) tandis que les lettres morphent leur attribut d via data-d2/data-d3 et rebondissent. Usage avance des LABELS de position timeline ('<<'=debut du precedent, '<'=fin du precedent, '-=290'=overlap relatif) pour tisser une douzaine de pistes. fitElementToParent recalcule scale au resize.
- **Tags** : `timeline-position-labels`, `motion-path`, `path-morph-attr`, `spring-ease`, `responsive-fit`, `logo-animation`, `relative-offsets`
- **Primitives** : `createTimeline`, `svg.createMotionPath('.bounce path')`, `bouncePath.translateX/translateY`, `d: el => el.dataset.d2 (path morph via attr)`, `createSpring({velocity})`, `stagger({from})`, `position labels '<<' '<' '-=N'`, `onBegin/onRender removeAttribute`, `init()`
- **Empreinte** : createTimeline + svg.createMotionPath driving .dot + d:el=>dataset.d2 morph + position-labels(<<,<,-=N) + spring + stagger(from) \| multi-track logo choreography woven via relative timeline labels
- **SBFB** : ✅ — SVG+DOM, 0 reseau. PIEGE 6 IMPORTANT : ce demo importe createSpring (DEPRECIE en v4.5) — pour SBFB remplacer par spring() (utilise dans layout/index.js : import {spring}). PIEGE 1 : .dot est pose par utils.set translateX/Y avant la motion-path. Sinon UMD swap. Grosse demo de reference pour orchestration timeline serieuse.

```js
.add('.dot', {
  translateX: bouncePath.translateX,
  translateY: bouncePath.translateY,
  rotate: { to: '1turn', duration: 790 },
  ease: 'cubicBezier(0, .74, 1, .255)',
  duration: 800
}, '<<')
.add('.letter-m .line', {
  d: el => el.dataset.d3,
  ease: createSpring({velocity:10}),  // SBFB: remplacer par spring({velocity:10})
}, '-=680')
```

---

## animejs-v2-logo (mass createDrawable staggered line-draw)

- **Source** : `tests/playground/animejs-v2-logo-animation.html`
- **Idée** : Logo dessine au trait : createDrawable cible des groupes de paths ('.fill', '.line.out', '.icon-line') et anime draw '0 1' (apparait) puis '1 1' (se referme), avec stagger({start:700}) pour cascader. Astuce : la couleur de stroke cible est resolue par fonction depuis le parent (el => utils.get(el.parentNode, 'stroke')) — chaque lettre garde sa propre couleur.
- **Tags** : `svg-line-draw`, `createDrawable`, `draw-keyframes`, `stagger-start-offset`, `function-stroke-from-parent`, `logo-animation`
- **Primitives** : `createTimeline`, `svg.createDrawable('.fill')`, `draw:{to:'0 1'} / draw:'1 1'`, `stroke:{to:[...,el=>utils.get(el.parentNode,'stroke')]}`, `strokeWidth keyframes`, `stagger({start})`, `alternate`, `init()`
- **Empreinte** : createTimeline.alternate + svg.createDrawable(group) + draw '0 1'->'1 1' + stagger({start}) + stroke:to=el=>utils.get(parent,'stroke') \| grouped stroke-draw logo, per-element color resolved from DOM parent
- **SBFB** : ✅ — SVG inline, 0 reseau, 0 createSpring. utils.get lit une valeur CSS/attr de l'element — local. PIEGE 3 : stroke pose en attribut SVG (stroke="#5E89FB") sur les <g>, lu par utils.get — pour SBFB on peut basculer en var(--color-*) lus de la meme facon. UMD swap. Excellent patron pour reveal de logo/illustration au trait.

```js
.add(fillDraggable, {
  draw: { to: '0 1', duration: 600, ease: 'outQuart' },
  stroke: {
    to: ['#FFF', el => utils.get(el.parentNode, 'stroke')],
    duration: 350, ease: 'inOutQuart',
  },
}, stagger(60, { start: 700 }))
.add(fillDraggable, {
  draw: { to: '1 1', duration: 800, ease: 'inQuart' }
}, stagger(80, { start: 2000 }))
```

---

## animejs-mgs-logo (classList-branching function values + attr morph)

- **Source** : `tests/playground/animejs-mgs-logo-animation.html`
- **Idée** : Construction facon HUD Metal Gear : chaque trait SVG (.line) recoit translateX/translateY via une FONCTION qui branche sur classList (hori/vert/diag-left/diag-right) pour decider sa direction et amplitude d'entree (utils.random(0,1)?x:-x). Les katakana morphent leur d depuis data-d. fill anime avec alpha-zero '#F9C10000' pour fondu de teinte.
- **Tags** : `function-based-values`, `classlist-branching`, `directional-entrance`, `path-morph-attr`, `alpha-hex-fade`, `stagger-start`, `hud-assembly`
- **Primitives** : `createTimeline.alternate.loop`, `translateX/Y: target => branch on target.classList`, `d: $el => $el.getAttribute('data-d') (morph via attr)`, `fill keyframes ['#F9C10000','#F9C100']`, `stagger({start})`, `delay:()=>utils.random`
- **Empreinte** : createTimeline.alternate.loop + translateX/Y=target=>classList branch direction + d=$el=>getAttribute(data-d) + fill ['#xxxx0000','#xxxx'] \| per-target direction chosen by class, attr-driven morph, alpha-hex color fade
- **SBFB** : ✅ — SVG inline, 0 reseau, 0 createSpring. La technique classList->direction est purement locale et tres reutilisable pour des entrees directionnelles theming-aware. fill par attribut ici ; SBFB peut peindre via var(--color-*). UMD swap.

```js
.add('.line', {
  translateX: (target) => {
    let x = 1200, translate;
    if (target.classList.contains('hori')) translate = utils.random(0, 1) ? x : -x;
    if (target.classList.contains('diag-right') || target.classList.contains('diag-left')) translate = x / 3;
    return [translate, 0];
  },
  stroke: {to: ['#FFF', '#F9C100'], duration: 1200, ease: 'linear'},
  opacity: { to: [0, 1], duration: 10 },
  delay: stagger(25), duration: 500, ease: 'outSine',
}, 0)
```

---

## color-trail-canvas (plain-object targets + sibling color chaining)

- **Source** : `tests/playground/color-trail-canvas.html`
- **Idée** : anime anime des OBJETS JS plain ({x,y,color}) — pas le DOM — et un createTimer({onUpdate}) lit state.x/state.color a chaque frame pour peindre un canvas avec trail (fillRect alpha quasi-nul = fade exponentiel). Truc cle : la couleur de DEPART de chaque add() vient du add precedent via sibling-chaining (prevSibling._value), donc seul le 'to' est specifie ; overlap '-=500' fait que chaque tween prend la main a mi-course.
- **Tags** : `plain-object-target`, `canvas-render`, `onUpdate-draw-loop`, `color-sibling-chaining`, `trail-fade`, `timeline-overlap`, `value-interpolation`
- **Primitives** : `createTimeline`, `createTimer({onUpdate})`, `animate plain-object target {x,y,color}`, `color interpolation on JS object`, `timeline.add overlap '-=500'`, `sibling from-color chaining (prevSibling._value)`, `canvas 2d manual draw`
- **Empreinte** : createTimer.onUpdate canvas draw + animate(plainObject{x,y,color}) + timeline.add overlap '-=N' + implicit from-color via prevSibling \| anime drives JS object state, manual canvas paints it, color chains across sibling tweens
- **SBFB** : ✅ — Canvas 2d DIRECT (PAS via worker), 0 reseau — conforme. Le pattern 'anime pilote un objet, je peins' contourne le DOM et marche pour tout rendu custom dans l'iframe. UMD swap. Note: canvas via worker est INTERDIT (contrainte SBFB), mais ici c'est du main-thread getContext('2d') — OK.

```js
const state = { x: corners[0].x, y: corners[0].y, color: colors[0] };
for (let i = 1; i <= 4; i++) {
  const isLast = i === 4;
  // overshoot so midpoint lands on corner; -=500 hands off at that midpoint
  tl.add(state, {
    x: targetX, y: targetY,
    color: colors[i % colors.length],
    duration: isLast ? 500 : 1000,
  }, i === 1 ? 0 : '-=500');
}
createTimer({ autoplay: true, onUpdate: () => {
  ctx.fillStyle = `rgba(17,17,17,${trailFadeAlpha})`;
  ctx.fillRect(0, 0, canvas.clientWidth, canvas.clientHeight); // trail fade
  drawBall(state);
}});
```

---

## color-conversion (cross-format color interpolation)

- **Source** : `tests/playground/color-conversion.html`
- **Idée** : Verifie qu'anime interpole entre TOUS les formats de couleur melanges : hex court (#F47), hex6, hex8 (#FF447799 avec alpha), rgb/rgba, hsl/hsla. Chaque cellule lit ses deux couleurs depuis le DOM (split sur '<br>▾<br>') et anime backgroundColor de l'une a l'autre — preuve que le type COLOR d'anime normalise les espaces avant lerp.
- **Tags** : `color-interpolation`, `cross-format-color`, `hex8-alpha`, `hsl-rgb-mix`, `dom-driven-config`, `alternate-loop`
- **Primitives** : `animate`, `backgroundColor: [fromStr, toStr]`, `alternate loop`, `DOM-read color strings`
- **Empreinte** : animate + backgroundColor:[anyFormat, anyFormat] alternate \| interpolate across hex/hex8/rgb/rgba/hsl/hsla mixed, config parsed from DOM text
- **SBFB** : ✅ — Pur DOM, 0 reseau. backgroundColor est composite-acceptable. Lire la config depuis le DOM est un patron data-SIMULEE ideal pour SBFB. UMD swap. Bon pour des transitions de theme/etat.

```js
const testValues = el.innerHTML.split('<br>▾<br>');
const animation = animate(colorEl, {
  backgroundColor: [testValues[0], testValues[1]], // e.g. '#FF447799' -> 'hsla(213,100%,70%,.15)'
  ease: 'inOut', duration: 4000, loop: true, alternate: true,
});
```

---

## scramble (scrambleText param matrix)

- **Source** : `tests/playground/scramble/index.js`
- **Idée** : Catalogue exhaustif de scrambleText : from 'right'/'center'/'random'/index, chars:'01' (data-stream), cursor:true ou pattern '░▒▓█', perturbation, seed:42 (reproductible), settleRate/revealRate, et surtout text:'...' pour SCRAMBLER VERS un autre texte (croissance/reduction de longueur) avec override:' ' ou false. S'applique via innerHTML: scrambleText(params).
- **Tags** : `scramble-text`, `text-effect`, `seeded-random`, `cursor-reveal`, `text-retarget`, `param-matrix`
- **Primitives** : `animate`, `innerHTML: scrambleText(params)`, `scrambleText({from,chars,cursor,perturbation,seed,settleRate,revealRate,settleDuration,override,text,reversed})`, `createTimeline +=offset`, `anim.restart()`
- **Empreinte** : animate + innerHTML:scrambleText({from\|chars\|cursor\|perturbation\|seed\|text\|override}) \| character-scramble reveal with directional origin, cursor front, seeded reproducibility, retarget text
- **SBFB** : ✅ — Pur texte DOM, 0 reseau. PIEGE 5 : scrambleText = animate(el,{innerHTML: scrambleText(...)}) ; poser le texte cible d'abord. Strings utilisateur en FRANCAIS pour SBFB. UMD : window.anime.scrambleText. seed:42 garantit le meme rendu a chaque replay = utile pour tests hermetiques. Excellent pour titres/labels d'app.

```js
const anim = animate($els, {
  innerHTML: scrambleText(demo.params), // ex: { from: 'center', cursor: '░▒▓█', perturbation: 0.5 }
  duration: 1500,
  ...demo.animParams, // ex: { delay: stagger(200) }
});
$test.addEventListener('pointerenter', () => anim.restart());
// retarget: scrambleText({ text: 'Nouveau texte', override: ' ' })
```

---

## lerp (frame-decoupled follow via utils.lerp / utils.damp)

- **Source** : `tests/playground/lerp/index.js`
- **Idée** : Smoothing 'follow the leader' decouple du framerate : un createTimer({frameRate:10}) lit chaque frame la position SOURCE et la position COURANTE via utils.get, puis ecrit utils.lerp(courant, source, .075) (geometrique, frame-dependant) versus utils.damp(..., clock.deltaTime, ...) (correct en temps reel, independant du fps). modifier:utils.snap(100) sur la source montre l'easing implicite du suivi.
- **Tags** : `lerp-follow`, `frame-rate-decoupled`, `damp-vs-lerp`, `read-back-value`, `procedural-modifier`, `seeded-random`
- **Primitives** : `createTimer({frameRate,onUpdate})`, `utils.lerp(a,b,t)`, `utils.damp(a,b,deltaTime,t)`, `utils.get(el,'x',false)`, `utils.set`, `utils.snap (modifier)`, `utils.createSeededRandom`
- **Empreinte** : createTimer.onUpdate + utils.get readback + utils.lerp vs utils.damp(deltaTime) + modifier:utils.snap \| manual per-frame smoothing of one target toward another, fps-independent via damp
- **SBFB** : ✅ — Pur DOM transform, 0 reseau. Retirer les console.log de debug. utils.damp est la bonne primitive pour un suivi fluide robuste au fps. UMD : window.anime.utils.lerp/damp/get/set. Cas SBFB : curseur/indicateur qui suit doucement une valeur simulee.

```js
createTimer({
  frameRate: 10,
  onUpdate: clock => {
    const sourceX = utils.get($input, 'x', false);
    const dampedX = utils.get($damped, 'x', false);
    utils.set($damped, {
      x: utils.damp(dampedX, sourceX, clock.deltaTime, .075) // fps-independent
    });
  }
});
// vs utils.lerp(lerpedX, sourceX, .075) // fps-dependent
```

---

## timekeeper (createScope mediaQueries + keepTime)

- **Source** : `tests/playground/timekeeper/index.js`
- **Idée** : Anime responsive : createScope avec une mediaQuery nommee (minM); self.keepTime(fn) RECONSTRUIT la timeline a chaque changement de breakpoint MAIS preserve la position de lecture courante (le timing ne saute pas). Les params dependent de scope.matches.minM (axe x vs y selon la largeur). Le scope toggle aussi une classe body.
- **Tags** : `scope`, `media-query-anim`, `keepTime`, `responsive-rebuild`, `time-preserving-rebuild`
- **Primitives** : `createScope({mediaQueries})`, `scope.add(self => ...)`, `self.keepTime(scope => createTimeline(...))`, `scope.matches.minM`, `responsive params inside scope`
- **Empreinte** : createScope({mediaQueries}) + self.keepTime(()=>createTimeline) + scope.matches.X branching \| rebuild animation on breakpoint change while preserving current playback time
- **SBFB** : ✅ — Pur DOM, 0 reseau. mediaQueries fonctionnent dans l'iframe (responsive a la taille de l'iframe). keepTime evite les sauts visuels au resize. UMD : window.anime.createScope. Tres utile pour des apps SBFB qui doivent rester propres a tout viewport.

```js
createScope({
  mediaQueries: { minM: '(min-width: 800px)' }
}).add(self => {
  self.keepTime(scope => {
    const isMinM = scope.matches.minM;
    document.body.classList.toggle('is-min-m', isMinM);
    return createTimeline().add('.square', {
      x: isMinM ? 0 : [-50, 50],
      y: isMinM ? [-50, 50] : 0,
      rotate: -90, scale: .75, alternate: true, loop: true, ease: 'inOutQuad',
    });
  });
});
```

---

## scope (createScope root + addOnce + keepTime + revert)

- **Source** : `tests/playground/scope/index.js`
- **Idée** : Demonstration complete du cycle de vie createScope : root limite les selecteurs ('.scoped'), addOnce() execute une fois et n'est PAS reverte aux changements de mediaQuery, keepTime() reconstruit en gardant le temps, le scope retourne une fonction de cleanup (remove listeners), et scope.revert() defait tout au click. background lit une CSS var aleatoire via utils.get($el, '--cyan-1').
- **Tags** : `scope`, `scope-root`, `addOnce`, `keepTime`, `scope-revert`, `scope-cleanup`, `css-var-read`
- **Primitives** : `createScope({mediaQueries,defaults,root})`, `self.addOnce(fn)`, `self.keepTime(fn)`, `scope.revert()`, `scope cleanup return fn`, `background:($el)=>utils.get(--var) randomPick`, `pointer handlers inside scope`
- **Empreinte** : createScope({root,mediaQueries}) + addOnce + keepTime + return cleanup + scope.revert + background=$el=>utils.get('--var') \| full scope lifecycle: rooted selectors, one-shot, time-preserving rebuild, teardown
- **SBFB** : ✅ — Pur DOM, 0 reseau. createScope({root}) est ideal pour composants SBFB isoles. La fonction de cleanup + revert() est le bon patron pour monter/demonter sans fuite. utils.get('--cyan-1') lit une CSS var locale (theming SBFB via var(--color-*)). UMD swap. Retirer console.log.

```js
const scope = createScope({ mediaQueries:{isSmall:'(max-width:800px)'}, defaults:{ease:'linear'}, root:'.scoped' })
.add(self => {
  self.addOnce(() => animate('.square', { y:[0,-50,0,50,0], loop:true, ease:'inOut(2)', duration:2500 }));
  self.keepTime(() => animate('.square', { rotate:360, duration:2000, loop:true, alternate:true }));
  self.keepTime(() => animate('.square', {
    background: ($el) => utils.get($el, utils.randomPick(['--cyan-1','--lavender-1','--pink-1'])),
    loop:true, alternate:true, duration:2000,
  }));
  return () => { /* remove listeners */ };
});
document.body.addEventListener('click', () => scope.revert());
```

---

## playback (engine + animation playback control surface)

- **Source** : `tests/playground/playback/index.js`
- **Idée** : Banc de controle exhaustif de la lecture : engine.fps/engine.speed (global), animation.fps/animation.speed (local), set reversed=true/false au lieu de reverse(), iterationProgress comme setter pour scrubber, monitoring de la 'time drift' (currentTime vs Date.now elapsed) toutes les secondes. onUpdate(self) ecrit iterationProgress/currentTime/reversed dans des inputs.
- **Tags** : `playback-control`, `engine-fps`, `engine-speed`, `scrub-progress`, `reversed-setter`, `time-drift-monitor`, `onUpdate-readout`
- **Primitives** : `createTimeline({onUpdate,onLoop})`, `engine.fps`, `engine.speed`, `animation.fps`, `animation.speed`, `animation.reversed`, `animation.alternate()`, `animation.pause/resume`, `animation.iterationProgress`, `self.currentTime/iterationProgress/reversed/currentIteration`
- **Empreinte** : createTimeline + engine.fps/speed + animation.fps/speed/reversed/iterationProgress setter + onUpdate(self) readout \| full playback-rate/frame-rate control surface + scrub + drift monitoring
- **SBFB** : ✅ — Pur DOM + inputs, 0 reseau. engine.fps/speed sont globaux (cote moteur) — pratique pour ralentir tout l'iframe. iterationProgress en setter = scrubber. UMD : window.anime.engine. Les <input type=button> marchent (sandbox allow-scripts ne bloque que les <form> submit, PAS les click handlers — cf. piege iframe-sandbox-forms : ici ce sont des boutons+handlers, conforme).

```js
$enginePlaybackrate.oninput = () => { engine.speed = +$enginePlaybackrate.value; };
$animPlaybackrate.oninput = () => { animation.speed = +$animPlaybackrate.value; };
$animReverse.onclick = () => { animation.reversed = true; };
$animToggle.onclick = () => { animation.alternate(); };
$animProgress.oninput = v => { animation.iterationProgress = v.target.value; }; // scrub
// onUpdate: self => { $animProgress.value = `${self.iterationProgress}`; }
```

---

## timeline-nested (tl.sync of timelines + timers)

- **Source** : `tests/playground/timeline/nested/index.js`
- **Idée** : Composition par SYNC : on cree des animate() autonomes (A,B,C,rotateAll), puis une timeline les .sync() ensemble avec overlap ('-=500'), puis une timeline PARENT .sync(cette timeline) ET .sync(un createTimer) — donc timelines et timers s'imbriquent comme citoyens de premiere classe sous un parent unique loop.
- **Tags** : `timeline-sync`, `nested-timeline`, `timer-in-timeline`, `animation-composition`, `timeline-overlap`
- **Primitives** : `animate (autonomous)`, `createTimeline.sync(animation, position)`, `createTimeline.sync(otherTimeline)`, `createTimeline.sync(timer,0)`, `createTimer`, `'-=500' overlap`
- **Empreinte** : createTimeline.sync(animate) + .sync(timeline) + .sync(timer) nested under loop parent \| compose standalone animations, child timelines and timers into one parent via sync
- **SBFB** : ✅ — Pur DOM, 0 reseau. .sync() (vs .add()) reutilise des animations DEJA construites — bon pour modulariser de gros sequencages SBFB. UMD swap. Retirer console.log.

```js
const A = animate('.square:nth-child(1)', { x: 200 });
const B = animate('.square:nth-child(2)', { x: 200 });
const rotateAll = animate('.square', { rotate: 360 });
const TL = createTimeline({ loop: true, alternate: true })
  .sync(A).sync(B, '-=500').sync(C, '-=500').sync(rotateAll, 0);
const timer = createTimer({ onUpdate: self => console.log(self.currentTime) });
createTimeline({ loop: true }).sync(TL).sync(timer, 0); // timelines+timers nest
```

---

## tl-seek-test (mouse-scrubbed 2000-element timeline)

- **Source** : `tests/playground/tl-seek-test/index.js`
- **Idée** : Scrubbing manuel : une timeline de 2000 elements (chacun un add() opacity/scale 100ms, teinte hsl repartie sur 360deg) avec autoplay:false ; window.onmousemove ecrit tl.progress = clientX/innerWidth — la position X de la souris devient la tete de lecture de toute la timeline. Stress-test + patron scrubber generique.
- **Tags** : `scrubbed-timeline`, `progress-setter`, `mouse-scrub`, `stress-test`, `procedural-hsl`, `manual-playhead`
- **Primitives** : `createTimeline({autoplay:false})`, `tl.add per element`, `tl.progress = value (setter scrub)`, `window.onmousemove`, `procedural hsl generation`
- **Empreinte** : createTimeline(autoplay:false) + 2000 adds + tl.progress = mouseX/width \| pointer position directly drives timeline playhead over many elements
- **SBFB** : ✅ — Pur DOM, 0 reseau. tl.progress en setter est LE patron de scrubber (utilisable avec une valeur SIMULEE, un slider, un scroll). 2000 elements = lourd ; reduire a quelques centaines dans l'iframe. UMD swap.

```js
const tl = createTimeline({ autoplay: false });
for (let i = 0; i < count; i++) {
  const $el = document.createElement('div');
  $el.style.backgroundColor = `hsl(${Math.round(360/count*i)}, 60%, 60%)`;
  document.body.appendChild($el);
  tl.add($el, { opacity: 0, scale: 2, duration: 100 });
}
window.onmousemove = (e) => { tl.progress = e.clientX / window.innerWidth; };
```

---

## onscroll-sync-modes (scroll-linked sync mode catalogue)

- **Source** : `tests/playground/onscroll/assets/sync-modes.js`
- **Idée** : Catalogue des modes de synchronisation onScroll passes a autoplay: sync:.5 (scrub LISSE avec smoothing), sync:'play pause' (declenche/suspend en entree/sortie), sync:'play alternate reverse reset' (chaine de commandes de lecture par evenement de scroll), sync:'inOutExpo' (scrub easee par une courbe). enter/leave avec offsets 'max-=100 top' et 8 callbacks directionnels.
- **Tags** : `scroll-linked`, `onScroll`, `scrubbed-scroll`, `scroll-sync-modes`, `directional-callbacks`, `enter-leave-thresholds`
- **Primitives** : `animate`, `autoplay: onScroll({...})`, `onScroll sync:.5 (smoothed scrub)`, `sync:'play pause'`, `sync:'play alternate reverse reset'`, `sync:'inOutExpo' (eased scrub)`, `enter/leave thresholds`, `onEnterForward/onLeaveBackward etc`, `self.linked.progress`
- **Empreinte** : animate + autoplay:onScroll({sync: number\|'play pause'\|'play alternate reverse reset'\|easeString, enter,leave, onEnterForward...}) \| scroll-driven playback with smoothed/eased/command scrub modes and directional hooks
- **SBFB** : ✅ — Scroll DANS l'iframe (l'iframe a son propre scroll), 0 reseau. onScroll observe le scroll local — fonctionne. debug:true dessine des marqueurs (a retirer en prod). UMD : window.anime.onScroll. Idees SBFB : reveal au scroll d'une longue page d'app. Retirer les helpers de log.

```js
animate('#section-01 .card', {
  rotate: [stagger(utils.random(-1,1,2)), stagger(15)],
  transformOrigin: ['75% 75%', '75% 75%'],
  ease: 'inOut(2)',
  autoplay: onScroll({
    enter: 'max-=100 top', leave: 'min+=100 bottom',
    sync: .5, // smoothed scrub; try 'play pause' | 'play alternate reverse reset' | 'inOutExpo'
    onEnterForward: () => {}, onLeaveBackward: () => {},
    onUpdate: self => self.linked.progress,
  }),
});
```

---

## onscroll-sticky-snap (scroll-scrubbed 3D card-stack)

- **Source** : `tests/playground/onscroll/assets/sticky-snap.js`
- **Idée** : Pile de cartes 3D revelee par le scroll : onScroll({target:'.sticky-container', sync:1}) lie 1:1 la progression de scroll a une timeline ; chaque carte entre depuis -100vh/50vh (alterne i%2), rotateX -180->0 (flip), avec rotateY/Z tires aleatoirement par carte pour un desempilement naturel. scroll-snap CSS + position:sticky cadrent la scene.
- **Tags** : `scroll-linked`, `scrubbed-scroll`, `3d-transform`, `card-stack-reveal`, `per-element-random`, `sticky-scroll`, `flip-rotateX`
- **Primitives** : `createTimeline`, `autoplay: onScroll({target,sync:1,enter:'top',leave:'bottom'})`, `rotateX/rotateY/rotateZ 3D`, `z translate`, `per-card random tilt utils.random`, `tl.add per card`, `tl.init()`
- **Empreinte** : createTimeline + autoplay:onScroll({sync:1,target}) + per-card rotateX[-180,0]+rotateY/Z=random + z stagger \| scroll-scrubbed 3D card-stack flip-in with per-card random tilt
- **SBFB** : ✅ — Scroll + transform 3D dans l'iframe, 0 reseau. transform/rotate/opacity composite-safe (perspective en CSS sur .sticky). sync:1 = scrub strict. UMD : window.anime.onScroll. Retirer debug:true. Bon patron d'entree spectaculaire pour un onboarding SBFB.

```js
const tl = createTimeline({ defaults:{ease:'inOut(1)'},
  autoplay: onScroll({ target:'.sticky-container', sync:1, enter:'top', leave:'bottom' }) });
utils.$('.card').forEach(($card, i) => {
  tl.add($card, {
    z: [40, i],
    y: [i % 2 ? '-100vh' : '50vh', `${-i * 3}px`],
    opacity: { to: [0, 1], duration: 50 },
    rotateX: [-180, 0],
    rotateY: [utils.random(-30, 30), 0],
    rotateZ: [utils.random(-30, 30), 0],
  });
});
tl.init();
```

---

## waapi-composition (3 engines side-by-side: waapi vs js)

- **Source** : `tests/playground/waapi/composition/index.js`
- **Idée** : Compare 3 moteurs sur la meme grille 10x10 au click : waapi.animate avec translate shorthand, waapi.animate avec x/y, et animate() JS — chacun avec stagger grille from center et des valeurs/durees function-based () => utils.random(...) RE-EVALUEES par element. Montre que waapi (composite-only) et le moteur JS partagent la meme API de params.
- **Tags** : `waapi`, `engine-comparison`, `grid-stagger`, `function-based-values`, `per-element-random`, `translate-shorthand`
- **Primitives** : `waapi.animate`, `animate (js)`, `translate shorthand 'Xpx Ypx'`, `x/y separate`, `stagger({grid,from:'center'})`, `to:()=>utils.random per-element`, `duration:()=>utils.random per-element`
- **Empreinte** : waapi.animate(translate shorthand) vs waapi.animate(x,y) vs js animate + stagger(grid,from:center) + to/duration=()=>random \| same params across WAAPI and JS engines, per-element random targets
- **SBFB** : ✅ — Pur DOM, 0 reseau. waapi.animate delegue a la Web Animations API (composite, perf GPU) — utile pour grosses grilles dans l'iframe. translate/x/y/opacity/scale composite-safe. UMD : window.anime.waapi.animate. Decommenter createSpring/box-shadow EVITE (createSpring deprecie + box-shadow non-composite — piege 2/6).

```js
waapi.animate('.container-A .square', {
  translate: `${X}px ${Y}px`,
  opacity: { to: () => utils.random(.1, 1, 3), duration: () => utils.random(500, 4000, 0) },
  scale:   { to: () => utils.random(.1, 2, 3), duration: () => utils.random(500, 4000, 0) },
  duration, ease: 'out',
  delay: stagger(24, { grid: [10, 10], from: 'center' }),
});
// vs js: const jsAnimation = animate('.container-C .square', { x: X, y: Y, ... });
```

---

## sandbox (timeline.sync of WAAPI animations)

- **Source** : `tests/playground/sandbox/index.js`
- **Idée** : Le plus court patron utile : creer des waapi.animate(autoplay:false) puis les .sync() dans une timeline anime a des positions explicites (0, 500). Prouve qu'une timeline JS anime peut piloter/sequencer des animations WAAPI natives.
- **Tags** : `waapi`, `timeline-sync`, `waapi-in-timeline`, `autoplay-false-then-sync`
- **Primitives** : `waapi.animate({autoplay:false})`, `createTimeline.sync(waapiAnim, position)`
- **Empreinte** : waapi.animate(autoplay:false) + createTimeline.sync(waapiAnim, position) \| sequence native WAAPI animations from an anime timeline
- **SBFB** : ✅ — Pur DOM, 0 reseau. Combine la perf WAAPI composite avec le sequencage timeline anime. UMD : window.anime.waapi + window.anime.createTimeline. Minimal et directement copiable.

```js
const { animate } = waapi;
const red = animate('.red', { x: '15rem', autoplay: false });
const blue = animate('.blue', { x: '15rem', autoplay: false });
const tl = createTimeline({loop: 1})
  .sync(red, 0)
  .sync(blue, 500);
```

---

## keyframes (percent-keyframe object syntax vs WAAPI vs CSS)

- **Source** : `tests/playground/keyframes/index.js`
- **Idée** : Trois implementations equivalentes d'une meme trajectoire cote a cote : (1) anime keyframes en objet pourcentage {'0%':{x,y}, '30%':{x,y,rotate,ease:easeOut}} ou la duration est divisee par le nb de keyframes et l'ease peut etre par-keyframe, (2) Element.animate WAAPI avec offsets, (3) CSS @keyframes pur. Pedagogie d'equivalence.
- **Tags** : `percent-keyframes`, `per-keyframe-ease`, `waapi`, `css-keyframes-equivalent`, `engine-comparison`
- **Primitives** : `animate({keyframes:{'0%':{...},'30%':{...,ease}}})`, `per-keyframe ease`, `element.animate (native WAAPI offset array)`, `CSS @keyframes (class toggle)`
- **Empreinte** : animate({keyframes:{'0%':{},'30%':{...,ease}}}) + native el.animate(offset[]) + CSS @keyframes \| percent-keyframe object with per-keyframe ease, equivalence across anime/WAAPI/CSS
- **SBFB** : ✅ — Pur DOM transform, 0 reseau. La syntaxe keyframes objet-pourcentage est tres lisible pour des trajectoires complexes. x/y/rotate composite-safe. UMD swap. La duration totale / nb keyframes est un piege documente (commentaire dans le code).

```js
animate('.anime', {
  keyframes: {
    '0%'  : { x: '0rem', y: '0rem' },
    '30%' : { x: '0rem', y: '-2.5rem', rotate: 45, ease: easeOut },
    '40%' : { x: '17rem', y: '-2.5rem' },
    '50%' : { x: '17rem', y: '2.5rem', rotate: 90 },
    '100%': { x: '0rem', y: '0rem', rotate: 180, ease: easeOut }
  },
  duration: 4000, // divided by number of keyframes
  ease: 'linear', loop: true,
});
```

---

## layout (createLayout FLIP auto-animate on DOM change)

- **Source** : `tests/playground/layout/index.js`
- **Idée** : Animation de LAYOUT facon FLIP : createLayout('.container') capture la geometrie, puis layout.update(fn,{duration}) execute une mutation DOM (toggle classe 'vertical', swap d'elements via data-layout-id, display:none, ajout/retrait) DANS le callback et anime automatiquement la transition first->last. Chaque test est encapsule dans createScope({root}) pour isoler les selecteurs. Importe spring (l'ease correct v4.5, pas createSpring).
- **Tags** : `layout-animation`, `FLIP`, `createLayout`, `layout-update`, `scope-root`, `layout-id-swap`, `spring-ease`
- **Primitives** : `createLayout('.container')`, `layout.update(({root})=>{...}, {duration})`, `createScope({root})`, `spring (ease, v4.5 correct)`, `stagger`, `layout.revert()`
- **Empreinte** : createLayout(selector) + layout.update(mutateFn,{duration}) + data-layout-id swap + createScope(root) + spring \| FLIP-style auto layout animation on DOM mutation, scoped, id-based element swap
- **SBFB** : ✅ — Pur DOM, 0 reseau. createLayout anime des changements de flux/position (couteux a la main) automatiquement. data-layout-id permet de morpher entre deux elements. PIEGE 6 respecte ici : ce demo utilise spring() (correct v4.5), PAS createSpring. UMD : window.anime.createLayout. Excellent pour reorganisations d'UI d'app SBFB (tri, filtre, expand).

```js
createScope({ root: '#simple-fixed-root' }).add(({ data }) => {
  data.layout = createLayout('.container');
  data.$button.addEventListener('click', () => {
    data.layout.update(({ root }) => {
      root.classList.toggle('vertical'); // mutate DOM inside update -> auto FLIP
    }, { duration });
  });
});
```

---

## draggables-callbacks (createDraggable snap + lifecycle + animate target)

- **Source** : `tests/playground/draggables/callbacks/index.js`
- **Idée** : createDraggable avec snap fonction (snap a la largeur de la cible), container de contrainte, velocityMultiplier:0 (pas d'inertie) et tous les callbacks de cycle (onGrab/onDrag/onUpdate/onRelease/onSettle/onSnap/onResize/onAfterResize). Truc avance : on peut passer l'INSTANCE draggable a animate() pour deplacer programmatiquement sa position (x:(draggable)=>...), anime et drag cohabitent sur la meme cible.
- **Tags** : `draggable`, `createDraggable`, `snap-grid`, `drag-lifecycle-callbacks`, `container-constraint`, `animate-draggable-instance`
- **Primitives** : `createDraggable('#el',{container,snap,velocityMultiplier})`, `onGrab/onDrag/onRelease/onSettle/onSnap/onResize callbacks`, `snap: self => self.$target.offsetWidth`, `animate(draggableInstance, {x:fn,y:fn})`
- **Empreinte** : createDraggable({container,snap:self=>w,velocityMultiplier}) + full on* callbacks + animate(draggableInstance,{x:fn,y:fn}) \| constrained draggable with function-snap and programmatic position animation of the draggable
- **SBFB** : ✅ — Pointer events DANS l'iframe, 0 reseau. createDraggable gere pointerdown/move/up localement — conforme. snap-as-function et container-as-selector sont locaux. UMD : window.anime.createDraggable. Retirer les log() helpers. Cas SBFB : sliders, poignees, reordonnancement tactile. Strings FR.

```js
const manualDraggable = createDraggable('#manual', {
  velocityMultiplier: 0,
  container: '#container',
  snap: self => self.$target.offsetWidth,
  onSnap: () => log($log1, 'A onSnap'),
  onSettle: () => log($log1, 'A onSettle'),
});
animate(animatedDraggable, {
  x: (draggable) => $container.offsetWidth - draggable.$target.offsetWidth,
  y: (draggable) => $container.offsetHeight - draggable.$target.offsetHeight * 2,
}); // anime drives the draggable's own position
```

---

