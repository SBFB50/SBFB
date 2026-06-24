# Documentation officielle anime.js v4 — scrape complet

> 419 pages de `animejs.com/documentation`, scrapees verbatim (prose + signatures + exemples de code). Entites HTML decodees.

## Sommaire

- **documentation** (1 pages)
- **getting-started** (5 pages)
- **timer** (20 pages)
- **animation** (57 pages)
- **timeline** (49 pages)
- **animatable** (11 pages)
- **draggable** (46 pages)
- **layout** (29 pages)
- **scope** (14 pages)
- **events** (16 pages)
- **svg** (4 pages)
- **text** (21 pages)
- **utilities** (43 pages)
- **easings** (7 pages)
- **web-animation-api** (17 pages)
- **engine** (13 pages)
- **adapters** (4 pages)
- **timer/timer-methods** (11 pages)
- **timer/timer-properties** (1 pages)
- **animation-callbacks** (2 pages)
- **animation-methods** (11 pages)
- **events / onScroll / ScrollObserver settings** (5 pages)
- **events / onScroll / ScrollObserver thresholds** (5 pages)
- **events / onScroll / ScrollObserver synchronisation modes** (3 pages)
- **events / onScroll / ScrollObserver Methods** (3 pages)
- **events / onScroll / ScrollObserver Properties** (1 pages)
- **text / splitText** (2 pages)
- **text / splitText / TextSplitter Settings** (3 pages)
- **text / scrambleText** (2 pages)
- **text / scrambleText / parameters** (10 pages)
- **adapters/threejs-adapter** (3 pages)


## documentation

### (racine)

`https://animejs.com/documentation`

> Page d'accueil de la documentation d'Anime.js v4 listant l'ensemble des grandes categories d'API du moteur d'animation JavaScript de Julian Garnier.

Documentation racine d'Anime.js v4.0.0. Le site organise l'API en grandes categories : Getting started (Installation, module imports, vanilla JS, React), Timer (playback settings, callbacks, methods, properties), Animation (targets, animatable properties, tween value types, keyframes, callbacks, methods), Timeline (adding timers/animations, syncing, time positioning, callbacks, methods), Animatable (settings, getters/setters, revert), Draggable (axes parameters, settings, callbacks, methods), Layout (usage patterns, settings, states, methods), Scope (constructor functions, method registration, parameters), Events (onScroll observer avec thresholds, synchronization modes, callbacks), SVG (morphTo, createDrawable, createMotionPath), Text NEW (splitText et scrambleText), Utilities (stagger(), fonctions mathematiques, utilitaires chainables), Easings (built-in eases, Bezier, linear, steps, spring), WAAPI (integration Web Animation API), Engine (configuration globale), Adapters NEW (adaptateur Three.js pour animations 3D). La navigation fournit des liens vers des exemples CodePen, le depot GitHub et les options de sponsoring. Un guide de migration v3 vers v4 est disponible sur GitHub.

**Faits clés**

- Version documentee : Anime.js v4.0.0
- Auteur : Julian Garnier
- Categories : Getting started, Timer, Animation, Timeline, Animatable, Draggable, Layout, Scope, Events, SVG, Text (NEW), Utilities, Easings, WAAPI, Engine, Adapters (NEW)
- Text et Adapters (Three.js) sont marquees NEW
- Guide de migration v3->v4 disponible sur GitHub


## getting-started

### getting-started

`https://animejs.com/documentation/getting-started`

> Section d'introduction couvrant le telechargement, l'installation et l'import d'Anime.js dans un projet, avec quatre sous-pages.

La section Getting Started introduit le telechargement, l'installation et l'import d'Anime.js dans un projet. Elle contient quatre sous-pages : Installation, Module imports, Using with vanilla JS, Using with React. Un guide de migration de v3 vers v4 est disponible sur GitHub pour les utilisateurs existants.

**Faits clés**

- Sous-pages : Installation, Module imports, Using with vanilla JS, Using with React
- Guide de migration v3->v4 disponible sur GitHub

### getting-started/installation

`https://animejs.com/documentation/getting-started/installation`

> Methodes d'installation d'Anime.js : NPM, CDN (ES modules et UMD global), et telechargement direct depuis GitHub avec 6 fichiers de distribution.

Anime.js v4.0.0+ s'installe de trois facons. (1) NPM : 'npm install animejs', puis import via ES modules ('import { animate } from "animejs"') ou CommonJS ('const { animate } = require("animejs")'). (2) CDN : ES modules depuis esm.sh ('import { animate } from "https://esm.sh/animejs"') ; UMD global depuis JsDelivr via une balise script qui cree un objet global 'anime'. (3) Telechargement direct depuis GitHub : six fichiers de distribution disponibles (points d'entree ES modules .js et CommonJS .cjs, plus les bundles ESM et UMD minifies et non-minifies).

**Faits clés**

- Trois methodes : NPM, CDN, telechargement direct GitHub
- NPM : npm install animejs
- Import ES module : import { animate } from 'animejs'
- Import CommonJS : const { animate } = require('animejs')
- CDN ESM : import { animate } from 'https://esm.sh/animejs'
- CDN UMD : https://cdn.jsdelivr.net/npm/animejs/dist/bundles/anime.umd.min.js cree un global 'anime'
- 6 fichiers de distribution GitHub : dist/modules/index.js (ESM entry), dist/modules/index.cjs (CJS entry), dist/bundles/anime.esm.js, dist/bundles/anime.esm.min.js, dist/bundles/anime.umd.js, dist/bundles/anime.umd.min.js

```js
npm install animejs
```

```js
import { animate } from 'animejs';
```

```js
const { animate } = require('animejs');
```

```js
import { animate } from 'https://esm.sh/animejs';
```

```js
<script src="https://cdn.jsdelivr.net/npm/animejs/dist/bundles/anime.umd.min.js"></script>
<script>
  const { animate } = anime;
</script>
```

### getting-started/module-imports

`https://animejs.com/documentation/getting-started/module-imports`

> Strategies d'import d'Anime.js : module principal, sous-chemins (subpaths) pour eviter de charger du code inutile, et chargement natif via importmap sans bundler.

Anime.js propose plusieurs strategies d'import. (1) Import depuis le module principal 'animejs' : tous les modules sont accessibles directement, pratique pour les projets avec bundler grace au tree-shaking. (2) Import depuis des sous-chemins (subpaths) : import granulaire qui empeche de charger du code inutile, utile quand le bundler ne peut pas optimiser ou en environnement non-bundle. La liste complete des subpaths inclut animejs/animation, animejs/timer, animejs/timeline, animejs/animatable, animejs/draggable, animejs/layout, animejs/scope, animejs/engine, animejs/events, animejs/easings, animejs/utils, animejs/svg, animejs/text, animejs/waapi. (3) ES Module imports sans bundler : via une 'importmap' qui mappe les specificateurs de modules aux chemins de fichiers reels dans le document HTML, permettant le chargement de modules sans outil de build.

**Faits clés**

- Import principal depuis 'animejs' (tree-shaking avec bundler)
- Subpaths granulaires evitent de charger du code inutile
- Subpaths disponibles : animejs/animation, /timer, /timeline, /animatable, /draggable, /layout, /scope, /engine, /events, /easings, /utils, /svg, /text, /waapi
- Import sans bundler possible via <script type="importmap">
- Chemins de fichiers : /node_modules/animejs/dist/modules/<module>/index.js

```js
import { animate, splitText, stagger, random, globals } from 'animejs';

const split = splitText('p');

animate(split.words, {
  opacity: () => random(0, 1, 2),
  delay: stagger(50),
});
```

```js
import { animate } from 'animejs/animation';
import { splitText } from 'animejs/text';
import { stagger, random } from 'animejs/utils';

const split = splitText('p');

animate(split.words, {
  opacity: () => random(0, 1, 2),
  delay: stagger(50),
});
```

```js
import { animate } from 'animejs/animation';
import { createTimer } from 'animejs/timer';
import { createTimeline } from 'animejs/timeline';
import { createAnimatable } from 'animejs/animatable';
import { createDraggable } from 'animejs/draggable';
import { createLayout } from 'animejs/layout';
import { createScope } from 'animejs/scope';
import { engine } from 'animejs/engine';
import * as events from 'animejs/events';
import * as easings from 'animejs/easings';
import * as utils from 'animejs/utils';
import * as svg from 'animejs/svg';
import * as text from 'animejs/text';
import * as waapi from 'animejs/waapi';
```

```js
<script type="importmap">
{
  "imports": {
    "animejs": "/node_modules/animejs/dist/modules/index.js",
    "animejs/animation": "/node_modules/animejs/dist/modules/animation/index.js",
    "animejs/timer": "/node_modules/animejs/dist/modules/timer/index.js",
    "animejs/timeline": "/node_modules/animejs/dist/modules/timeline/index.js",
    "animejs/animatable": "/node_modules/animejs/dist/modules/animatable/index.js",
    "animejs/draggable": "/node_modules/animejs/dist/modules/draggable/index.js",
    "animejs/layout": "/node_modules/animejs/dist/modules/layout/index.js",
    "animejs/scope": "/node_modules/animejs/dist/modules/scope/index.js",
    "animejs/engine": "/node_modules/animejs/dist/modules/engine/index.js",
    "animejs/events": "/node_modules/animejs/dist/modules/events/index.js",
    "animejs/easings": "/node_modules/animejs/dist/modules/easings/index.js",
    "animejs/utils": "/node_modules/animejs/dist/modules/utils/index.js",
    "animejs/svg": "/node_modules/animejs/dist/modules/svg/index.js",
    "animejs/text": "/node_modules/animejs/dist/modules/text/index.js",
    "animejs/waapi": "/node_modules/animejs/dist/modules/waapi/index.js"
  }
}
</script>

<script type="module">
  import { animate } from 'animejs/animation';
  import { splitText } from 'animejs/text';
  import { stagger, random } from 'animejs/utils';

  const split = splitText('p');

  animate(split.words, {
    opacity: () => random(0, 1, 2),
    delay: stagger(50),
  });
</script>
```

### getting-started/using-with-vanilla-js

`https://animejs.com/documentation/getting-started/using-with-vanilla-js`

> Utilisation d'Anime.js en JavaScript vanilla : importer les modules necessaires et commencer a animer directement, avec un exemple complet (logo anime, draggable, rotation au clic).

Utiliser Anime.js en JavaScript vanilla est simple : il suffit d'importer les modules necessaires et de commencer a animer. L'exemple importe animate, utils, createDraggable et spring. Il selectionne des elements via utils.$, cree une animation de rebond en boucle sur '.logo.js' (keyframes scale avec ease 'inOut(3)' puis spring bounce), rend le logo draggable autour de son centre avec releaseEase spring, et anime la rotation du logo a chaque clic sur le bouton (rotate = rotations * 360, ease 'out(4)', duration 1500).

**Faits clés**

- Pattern : importer les modules necessaires puis animer directement
- utils.$('.selector') retourne un tableau (destructuration [ $el ])
- animate() accepte des keyframes pour scale (tableau d'objets { to, ease, duration })
- spring({ bounce: .7 }) utilisable comme ease et comme releaseEase
- createDraggable avec container: [0,0,0,0]
- Eases dans l'exemple : 'inOut(3)', 'out(4)', spring({ bounce: .7 })
- Previous: Module imports ; Next: Using with React

```js
import { animate, utils, createDraggable, spring } from 'animejs';

const [ $logo ] = utils.$('.logo.js');
const [ $button ] = utils.$('button');
let rotations = 0;

// Created a bounce animation loop
animate('.logo.js', {
  scale: [
    { to: 1.25, ease: 'inOut(3)', duration: 200 },
    { to: 1, ease: spring({ bounce: .7 }) }
  ],
  loop: true,
  loopDelay: 250,
});

// Make the logo draggable around its center
createDraggable('.logo.js', {
  container: [0, 0, 0, 0],
  releaseEase: spring({ bounce: .7 })
});

// Animate logo rotation on click
const rotateLogo = () => {
  rotations++;
  $button.innerText = `rotations: ${rotations}`;
  animate($logo, {
    rotate: rotations * 360,
    ease: 'out(4)',
    duration: 1500,
  });
}

$button.addEventListener('click', rotateLogo);
```

### getting-started/using-with-react

`https://animejs.com/documentation/getting-started/using-with-react`

> Integration d'Anime.js avec React en combinant useEffect() et createScope() ; les instances declarees dans le scope sont nettoyees via scope.current.revert() et les methodes exposees via self.add().

Anime.js s'utilise avec React en combinant useEffect() de React et createScope() d'Anime.js. Dans useEffect, createScope({ root }).add(self => {...}) scope toutes les instances Anime.js au <div ref={root}>. A l'interieur : une animation de rebond en boucle sur '.logo', un createDraggable autour du centre, et l'enregistrement d'une methode self.add('rotateLogo', (i) => {...}) appelable hors de useEffect. Le nettoyage de toutes les instances se fait dans le return de useEffect via scope.current.revert(). Le clic sur le bouton appelle scope.current.methods.rotateLogo(newRotations) pour animer la rotation. Composants cles du pattern : useRef() pour les references DOM, createScope() pour namespacer les animations a un conteneur, scope.current.revert() pour le cleanup, self.add() pour enregistrer des methodes appelables a l'exterieur.

**Faits clés**

- Pattern : combiner React useEffect() et Anime.js createScope()
- createScope({ root }).add(self => {...}) scope les instances au <div ref={root}>
- Cleanup : return () => scope.current.revert() dans useEffect
- self.add('nom', fn) enregistre une methode appelable hors useEffect
- Appel externe : scope.current.methods.rotateLogo(newRotations)
- useRef(null) pour root et scope ; useState pour l'etat (rotations)
- useEffect avec tableau de deps vide [] (montage unique)

```js
import { animate, createScope, spring, createDraggable } from 'animejs';
import { useEffect, useRef, useState } from 'react';
import reactLogo from './assets/react.svg';
import './App.css';

function App() {
  const root = useRef(null);
  const scope = useRef(null);
  const [ rotations, setRotations ] = useState(0);

  useEffect(() => {
  
    scope.current = createScope({ root }).add( self => {
    
      // Every Anime.js instance declared here is now scoped to <div ref={root}>

      // Created a bounce animation loop
      animate('.logo', {
        scale: [
          { to: 1.25, ease: 'inOut(3)', duration: 200 },
          { to: 1, ease: spring({ bounce: .7 }) }
        ],
        loop: true,
        loopDelay: 250,
      });
      
      // Make the logo draggable around its center
      createDraggable('.logo', {
        container: [0, 0, 0, 0],
        releaseEase: spring({ bounce: .7 })
      });

      // Register function methods to be used outside the useEffect
      self.add('rotateLogo', (i) => {
        animate('.logo', {
          rotate: i * 360,
          ease: 'out(4)',
          duration: 1500,
        });
      });

    });

    // Properly cleanup all Anime.js instances declared inside the scope
    return () => scope.current.revert()

  }, []);

  const handleClick = () => {
    setRotations(prev => {
      const newRotations = prev + 1;
      // Animate logo rotation on click using the method declared inside the scope
      scope.current.methods.rotateLogo(newRotations);
      return newRotations;
    });
  };

  return (
    <div ref={root}>
      <div className="large centered row">
        <img src={reactLogo} className="logo react" alt="React logo" />
      </div>
      <div className="medium row">
        <fieldset className="controls">
        <button onClick={handleClick}>rotations: {rotations}</button>
        </fieldset>
      </div>
    </div>
  )
}

export default App;
```

```js

.logo.react {
  width: 150%;
  height: 150%;
}
```


## timer

### timer

`https://animejs.com/documentation/timer`

> Le Timer cree et gere des callbacks temporises, une alternative amelioree aux fonctions de timing natives du navigateur, synchronisee avec les animations. Cree via createTimer().

Le Timer d'Anime.js cree et gere des callbacks temporises, servant d'alternative amelioree aux fonctions de timing natives du navigateur tout en restant synchronise avec les animations. Les timers sont instancies via createTimer(), disponible depuis 'animejs' ou le module autonome 'animejs/timer'. Sections de la page : Playback settings, Callbacks, Methods, Properties.

**Faits clés**

- Cree via createTimer(parameters) ; retourne une instance Timer
- Imports : import { createTimer } from 'animejs' OU 'animejs/timer'
- parameters (optionnel, Object) combine Timer playback settings et Timer callbacks
- Sections : Playback settings, Callbacks, Methods, Properties
- Proprietes utilisees dans l'exemple : self.currentTime, self._currentIteration
- frameRate accepte un nombre (ex. 30)

```js
import { createTimer } from 'animejs';
const timer = createTimer(parameters);
```

```js
import { createTimer } from 'animejs/timer';
```

```js
import { createTimer } from 'animejs';

const [ $time, $count ] = utils.$('.value');

createTimer({
  duration: 1000,
  loop: true,
  frameRate: 30,
  onUpdate: self => $time.innerHTML = self.currentTime,
  onLoop: self => $count.innerHTML = self._currentIteration
});
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="value lcd">0</span>
    </pre>
  </div>
  <div class="half col">
    <pre class="large log row">
      <span class="label">callback fired</span>
      <span class="value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings

`https://animejs.com/documentation/timer/timer-playback-settings`

> Vue d'ensemble des reglages de lecture (playback settings) de createTimer() ; neuf parametres controlent les timings et comportements du timer.

Les Timer playback settings sont des proprietes de configuration de createTimer() qui controlent les timings et comportements d'un timer ; elles sont definies directement dans l'objet parameters. La page liste neuf parametres : delay, duration, loop, loopDelay, alternate, reversed, autoplay, frameRate, playbackRate. La page d'ensemble ne fournit pas les specifications detaillees (valeurs par defaut, types acceptes, plages) de chaque parametre ; ces details sont sur les pages dediees accessibles via la navigation. Disponible depuis la 4.0.0.

**Faits clés**

- Neuf parametres : delay, duration, loop, loopDelay, alternate, reversed, autoplay, frameRate, playbackRate
- Definis directement dans l'objet parameters de createTimer()
- Disponible depuis 4.0.0
- Les valeurs par defaut/types detailles sont sur les pages dediees

```js
createTimer({
  duration: 1000,
  frameRate: true,
  loop: true,
  // Additional callbacks follow
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### timer/timer-playback-settings/delay

`https://animejs.com/documentation/timer/timer-playback-settings/delay`

> Parametre delay : nombre >= 0 (defaut 0) definissant le temps en millisecondes avant que le timer ne demarre.

Le parametre delay (Number, valeur >= 0, defaut 0) definit le temps en millisecondes avant que le timer ne demarre. Il controle combien de temps le systeme attend avant de lancer l'execution du timer. La valeur par defaut globale peut etre modifiee via engine.defaults.delay. L'exemple configure un timer avec une pause initiale de 2000 ms avant le debut de l'execution, avec affichage en temps reel du temps ecoule.

**Faits clés**

- Nom : delay
- Type : Number
- Valeurs acceptees : >= 0
- Defaut : 0
- Override global : engine.defaults.delay = 500
- Unite : millisecondes

```js
import { engine } from 'animejs';
engine.defaults.delay = 500;
```

```js
import { createTimer, utils } from 'animejs';

const [ $time ] = utils.$('.time');

createTimer({
  delay: 2000,
  onUpdate: self => $time.innerHTML = self.currentTime
});
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings/duration

`https://animejs.com/documentation/timer/timer-playback-settings/duration`

> Parametre duration : nombre >= 0 (defaut Infinity) definissant la duree du timer en millisecondes ; les valeurs > 1e12 sont clampees a 1e12 (~32 ans).

Le parametre duration (Number >= 0, defaut Infinity) definit combien de temps le timer s'execute, en millisecondes. Une duration de 0 fait que le timer se termine immediatement au lancement. Les valeurs superieures a 1e12 sont clampees en interne a 1e12, ce qui represente environ 32 ans, pour eviter que des durees inattendument longues ne cassent le systeme.

**Faits clés**

- Nom : duration
- Type : Number
- Valeurs acceptees : >= 0
- Defaut : Infinity
- duration: 0 => le timer se termine immediatement au playback
- Gotcha : valeurs > 1e12 clampees en interne a 1e12 (~32 ans)
- Unite : millisecondes

```js
import { createTimer, utils } from 'animejs';

const [ $time ] = utils.$('.time');

createTimer({
  duration: 2000,
  onUpdate: self => $time.innerHTML = self.currentTime
});
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings/loop

`https://animejs.com/documentation/timer/timer-playback-settings/loop`

> Parametre loop : Number, Infinity, true ou -1 (defaut 0) definissant combien de fois un timer se repete ; true et -1 equivalent a Infinity.

Le parametre loop (accepte Number, Infinity, true ou -1 ; defaut 0) definit combien de fois un timer se repete, dans la plage [0, Infinity]. Les valeurs true et -1 fonctionnent toutes deux comme Infinity, activant une repetition sans fin. Reference des valeurs : Number = repetitions dans [0, Infinity] ; Infinity = boucle perpetuelle ; true = equivalent a Infinity ; -1 = equivalent a Infinity. La valeur par defaut globale peut etre modifiee via engine.defaults.loop. L'exemple affiche le nombre de boucles (onLoop) et le temps d'iteration courant (self.iterationCurrentTime via onUpdate).

**Faits clés**

- Nom : loop
- Types acceptes : Number, Infinity, true, -1
- Defaut : 0
- Plage : [0, Infinity]
- true === Infinity ; -1 === Infinity
- Override global : engine.defaults.loop = true
- onLoop se declenche a chaque boucle ; self.iterationCurrentTime = temps de l'iteration courante

```js
import { engine } from 'animejs';
engine.defaults.loop = true;
```

```js
import { createTimer, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');
const [ $time ] = utils.$('.time');

let loops = 0;

createTimer({
  loop: true,
  duration: 1000,
  onLoop: () => $loops.innerHTML = ++loops,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime
});
```

```js
<div class="large centered row">
  <div class="col">
    <pre class="large log row">
      <span class="label">loops count</span>
      <span class="loops value">0</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings/playback-loopdelay

`https://animejs.com/documentation/timer/timer-playback-settings/playback-loopdelay`

> Parametre loopDelay : nombre >= 0 (defaut 0) definissant le delai en millisecondes entre les boucles.

Le parametre loopDelay (Number >= 0, defaut 0) definit le delai en millisecondes entre les boucles. Il s'applique specifiquement aux reglages de lecture des timers. La valeur par defaut globale peut etre modifiee via engine.defaults.loopDelay. L'exemple combine loop: true, loopDelay: 750 et duration: 250, affiche le nombre de boucles (onLoop) et le temps d'iteration borne via utils.clamp(self.iterationCurrentTime, 0, 250). Disponible depuis la 4.0.0.

**Faits clés**

- Nom : loopDelay
- Type : Number
- Valeurs acceptees : >= 0
- Defaut : 0
- Override global : engine.defaults.loopDelay = 500
- Unite : millisecondes (delai entre les boucles)
- Disponible depuis 4.0.0

```js
import { engine } from 'animejs';
engine.defaults.loopDelay = 500;
```

```js
import { createTimer, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');
const [ $time ] = utils.$('.time');

let loops = 0;

createTimer({
  loop: true,
  loopDelay: 750,
  duration: 250,
  onLoop: () => $loops.innerHTML = ++loops,
  onUpdate: self => $time.innerHTML = utils.clamp(self.iterationCurrentTime, 0, 250)
});
```

```js
<div class="large centered row">
  <div class="col">
    <pre class="large log row">
      <span class="label">loops count</span>
      <span class="loops value">0</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings/alternate

`https://animejs.com/documentation/timer/timer-playback-settings/alternate`

> Parametre alternate : Boolean (defaut false) qui inverse la direction du timer a chaque iteration (avant sur iterations impaires, arriere sur iterations paires) ; necessite loop true ou > 1.

Le parametre alternate (Boolean, defaut false) controle si la direction du timer s'inverse a chaque iteration. Quand il est active avec loop: true ou un loop count > 1, le timer joue en avant sur les iterations impaires et en arriere sur les iterations paires. La valeur par defaut globale peut etre modifiee via engine.defaults.alternate. Note : ce parametre necessite que loop soit true ou regle sur une valeur > 1 pour prendre effet.

**Faits clés**

- Nom : alternate
- Type : Boolean
- Defaut : false
- Override global : engine.defaults.alternate = true
- Avant sur iterations impaires, arriere sur iterations paires
- Gotcha : necessite loop: true ou loop > 1 pour prendre effet
- Note verbatim de l'exemple : l'import affiche 'import { animate } from "animejs"' alors que le code utilise createTimer/utils (incoherence presente dans la doc source)

```js
import { engine } from 'animejs';
engine.defaults.alternate = true;
```

```js
import { animate } from 'animejs';

const [ $loops ] = utils.$('.loops');
const [ $time ] = utils.$('.time');

let loops = 0;

createTimer({
  loop: true,
  duration: 1000,
  alternate: true,
  onLoop: () => $loops.innerHTML = ++loops,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime
});
```

```js
<div class="large centered row">
  <div class="col">
    <pre class="large log row">
      <span class="label">loops count</span>
      <span class="loops value">0</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-playback-settings/reversed

`https://animejs.com/documentation/timer/timer-playback-settings/reversed`

> Le parametre reversed (Boolean, defaut false) etablit la direction de lecture initiale d'un timer : a true, la premiere iteration se joue en sens inverse pour iterationTime.

reversed (Boolean = false) etablit la direction de lecture initiale d'un timer. Le currentTime du timer avance toujours de 0 vers duration, mais mettre ce parametre a true fait que la propriete iterationTime (iterationCurrentTime) tourne en sens inverse pendant la premiere iteration. Valeurs acceptees : true (la premiere iteration tourne a l'envers), false (premiere iteration normale, defaut). Modifiable globalement via engine.defaults.reversed = true.

**Faits clés**

- Signature: reversed: Boolean = false
- Defaut: false
- true => la premiere iteration tourne a l'envers (sur iterationTime)
- currentTime avance toujours de 0 a duration independamment de reversed
- Override global: engine.defaults.reversed = true
- Related: alternate, autoplay

```js
import { engine } from 'animejs';
engine.defaults.reversed = true;
```

```js
import { animate } from 'animejs';

const [ $iterationTime ] = utils.$('.iteration-time');
const [ $currentTime ] = utils.$('.current-time');

createTimer({
  duration: 10000,
  reversed: true,
  onUpdate: self => {
    $iterationTime.innerHTML = self.iterationCurrentTime;
    $currentTime.innerHTML = self.currentTime;
  }
});
```

### timer/timer-playback-settings/autoplay

`https://animejs.com/documentation/timer/timer-playback-settings/autoplay`

> autoplay (Boolean | onScroll(), defaut true) controle si un timer demarre automatiquement ; false impose un appel manuel a .play() ; ignore (force a false) si le timer est ajoute a une timeline.

autoplay (type Boolean | onScroll(), defaut true) gouverne si un timer commence a jouer automatiquement. A true, la lecture demarre immediatement. A false, il faut declencher manuellement via .play(). On peut aussi passer la fonction onScroll() pour declencher la lecture quand les conditions de seuil de scroll sont satisfaites. Ce parametre est ignore lorsque le timer est ajoute a une timeline, et est alors force a false. Modifiable globalement via engine.defaults.autoplay = false.

**Faits clés**

- Signature: autoplay: Boolean | onScroll()
- Defaut: true
- false => demarrage manuel via .play()
- Accepte la fonction onScroll() pour declenchement au scroll
- Ignore et force a false quand le timer est ajoute a une timeline
- Override global: engine.defaults.autoplay = false

```js
const [ $time ] = utils.$('.time');
const [ $playButton ] = utils.$('.play');

const timer = createTimer({
  autoplay: false,
  onUpdate: self => $time.innerHTML = self.currentTime
});

const playTimer = () => timer.play();

$playButton.addEventListener('click', playTimer);
```

```js
import { engine } from 'animejs';
engine.defaults.autoplay = false;
```

### timer/timer-playback-settings/framerate

`https://animejs.com/documentation/timer/timer-playback-settings/framerate`

> frameRate (Number, defaut 240) determine les frames par seconde (fps) auxquels un timer tourne ; toute valeur > 0, plafonnee au taux de rafraichissement du moniteur/navigateur ; modifiable via timer.fps.

frameRate (Number, defaut 240) determine les images par seconde (fps) auxquelles un timer tourne. Il accepte n'importe quel nombre superieur a 0, bien que le taux reel soit plafonne au taux de rafraichissement du moniteur ou, dans certains cas, par le navigateur lui-meme. La valeur peut etre modifiee apres la creation du timer via timer.fps = value. Modifiable globalement via engine.defaults.frameRate = 30.

**Faits clés**

- Signature: frameRate: Number (default 240)
- Defaut: 240
- Accepte tout nombre > 0
- Plafonne au taux de rafraichissement du moniteur ou du navigateur
- Modifiable a chaud via timer.fps = value
- Override global: engine.defaults.frameRate = 30

```js
import { engine } from 'animejs';
engine.defaults.frameRate = 30;
```

```js
import { createTimer, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $fps ] = utils.$('.fps');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  frameRate: 60,
  onUpdate: self => $time.innerHTML = self.currentTime,
});

const updateFps = () => {
  const { value } = $range;
  $fps.innerHTML = value;
  timer.fps = value;
}

$range.addEventListener('input', updateFps);
```

### timer/timer-playback-settings/playbackrate

`https://animejs.com/documentation/timer/timer-playback-settings/playbackrate`

> playbackRate (Number, defaut 1, min 0) definit un multiplicateur de vitesse pour accelerer ou ralentir la lecture d'un timer ; 0 stoppe ; modifiable via timer.speed.

playbackRate (Number, defaut 1, valeur minimale 0) definit un multiplicateur de vitesse pour accelerer ou ralentir la lecture d'un timer (1.0 = vitesse normale). Une valeur de 0 stoppe entierement la lecture. Cette propriete peut etre modifiee apres creation via timer.speed = value. Modifiable globalement via engine.defaults.playbackRate = .75.

**Faits clés**

- Signature: playbackRate: Number
- Defaut: 1
- Valeur minimale: 0 (0 stoppe la lecture)
- 1.0 = vitesse normale
- Modifiable a chaud via timer.speed = value
- Override global: engine.defaults.playbackRate = .75
- Disponible depuis la version 4.0.0

```js
import { engine } from 'animejs';
engine.defaults.playbackRate = .75;
```

```js
import { createTimer, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $speed ] = utils.$('.speed');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  playbackRate: 2,
  onUpdate: self => $time.innerHTML = utils.round(self.currentTime, 0),
});

const updateSpeed = () => {
  const speed = utils.roundPad(+$range.value, 1);
  $speed.innerHTML = speed;
  utils.sync(() => timer.speed = speed);
}

$range.addEventListener('input', updateSpeed);
```

### timer/timer-callbacks

`https://animejs.com/documentation/timer/timer-callbacks`

> Page d'apercu listant les six callbacks de timer (onBegin, onComplete, onUpdate, onLoop, onPause, then()) specifies directement dans l'objet parametres de createTimer().

Les callbacks executent des fonctions a des points specifiques durant la lecture d'un timer. Les callbacks (Function) sont specifies directement dans l'objet de parametres de createTimer(). La documentation liste six methodes de callback : onBegin (s'execute quand le timer commence), onComplete (quand le timer se termine), onUpdate (durant les mises a jour de lecture), onLoop (a chaque iteration de boucle), onPause (quand le timer est mis en pause), et then() (mecanisme de callback base sur Promise). Les signatures, parametres et exemples detailles de chaque callback figurent sur leurs pages respectives.

**Faits clés**

- Six callbacks: onBegin, onComplete, onUpdate, onLoop, onPause, then()
- Callbacks de type Function specifies dans l'objet parametres de createTimer()
- onBegin = au demarrage
- onComplete = a la fin
- onUpdate = pendant les mises a jour
- onLoop = a chaque iteration de boucle
- onPause = a la pause
- then() = callback base sur Promise

```js
createTimer({
  duration: 1000,
  frameRate: true,
  loop: true,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### timer/timer-callbacks/onbegin

`https://animejs.com/documentation/timer/timer-callbacks/onbegin`

> onBegin (Function, defaut noop) s'execute quand un timer demarre et recoit l'instance du timer en premier argument.

Le callback onBegin (Function, defaut noop) s'execute quand un timer commence. Il recoit l'instance du timer en premier argument (self). Modifiable globalement via engine.defaults.onBegin. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: onBegin (Function), defaut noop
- Recoit l'instance du timer (self) en premier argument
- S'execute quand le timer demarre (apres delay le cas echeant)
- Override global: engine.defaults.onBegin = self => console.log(self.id)
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $status ] = utils.$('.status');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  delay: 2000,
  duration: 2000,
  onBegin: self => $status.innerHTML = 'true'
});

const logTimer = createTimer({
  duration: 4000,
  onUpdate: self => $time.innerHTML = timer.currentTime
});
```

```js
import { engine } from 'animejs';
engine.defaults.onBegin = self => console.log(self.id);
```

### timer/timer-callbacks/oncomplete

`https://animejs.com/documentation/timer/timer-callbacks/oncomplete`

> onComplete (Function, defaut noop) s'execute quand toutes les iterations d'un timer sont terminees ; recoit l'instance du timer en premier argument.

Le callback onComplete (Function, defaut noop) s'execute quand toutes les iterations d'un timer ont fini de jouer. Il recoit l'instance du timer en premier argument (self). Modifiable globalement via engine.defaults.onComplete.

**Faits clés**

- Signature: onComplete (Function), defaut noop
- Recoit l'instance du timer (self) en premier argument
- S'execute quand toutes les iterations sont terminees
- Override global: engine.defaults.onComplete = self => console.log(self.id)

```js
import { engine } from 'animejs';
engine.defaults.onComplete = self => console.log(self.id);
```

```js
import { createTimer, utils } from 'animejs';

const [ $status ] = utils.$('.status');
const [ $time ] = utils.$('.time');

createTimer({
  duration: 2000,
  onComplete: self => $status.innerHTML = 'true',
  onUpdate: self => $time.innerHTML = self.currentTime
});
```

```js
<div class="large row">
  <div class="col">
    <pre class="large log row">
      <span class="label">completed</span>
      <span class="status value">false</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-callbacks/onupdate

`https://animejs.com/documentation/timer/timer-callbacks/onupdate`

> onUpdate (Function, defaut noop) s'execute a chaque frame d'un timer en cours, declenche au frameRate specifie ; recoit l'instance du timer en premier argument.

Le callback onUpdate ((self) => void, Function, defaut noop) s'execute a chaque frame d'un timer en cours d'execution, declenche au frameRate specifie. Le callback recoit l'instance du timer en premier argument (self), permettant d'acceder aux proprietes comme currentTime. Modifiable globalement via engine.defaults.onUpdate.

**Faits clés**

- Signature: onUpdate: (self) => void (Function), defaut noop
- S'execute a chaque frame, au frameRate specifie
- Recoit l'instance du timer (self) en premier argument
- Acces a self.currentTime
- Override global: engine.defaults.onUpdate = self => console.log(self.id)

```js
import { engine } from 'animejs';
engine.defaults.onUpdate = self => console.log(self.id);
```

```js
import { createTimer, utils } from 'animejs';

const [ $updates ] = utils.$('.updates');
const [ $time ] = utils.$('.time');

let updates = 0;

createTimer({
  onUpdate: self => {
    $updates.innerHTML = ++updates;
    $time.innerHTML = self.currentTime;
  }
});
```

```js
<div class="large row">
  <div class="col">
    <pre class="large log row">
      <span class="label">updates</span>
      <span class="updates value">0</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-callbacks/onloop

`https://animejs.com/documentation/timer/timer-callbacks/onloop`

> onLoop (Function, defaut noop) s'execute a chaque fois qu'une iteration de timer se termine ; recoit l'instance du timer en parametre.

Le callback onLoop (Function, defaut noop) s'execute a chaque fois qu'une iteration de timer se termine. Il recoit l'instance du timer en parametre (self), permettant d'acceder aux proprietes du timer durant l'execution de la boucle. Modifiable globalement via engine.defaults.onLoop. Disponible depuis la v4.0.0.

**Faits clés**

- Signature: onLoop (Function callback), defaut noop
- S'execute a chaque fois qu'une iteration se termine
- Recoit l'instance du timer (self) en parametre
- Override global: engine.defaults.onLoop = self => console.log(self.id)
- Disponible depuis la v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onLoop = self => console.log(self.id);
```

```js
import { createTimer, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');
const [ $time ] = utils.$('.time');

let loops = 0;

createTimer({
  loop: true,
  duration: 1000,
  onLoop: self => $loops.innerHTML = ++loops,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime,
});
```

```js
<div class="large row">
  <div class="col">
    <pre class="large log row">
      <span class="label">loops</span>
      <span class="loops value">0</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-callbacks/onpause

`https://animejs.com/documentation/timer/timer-callbacks/onpause`

> onPause (Function, defaut noop) s'execute quand un timer en cours est mis en pause ; recoit l'instance du timer en premier argument.

Le callback onPause ((self) => void, Function, defaut noop) s'execute quand un timer en cours d'execution est mis en pause. Il recoit l'instance du timer en premier argument (self), permettant d'acceder aux proprietes et methodes du timer durant l'evenement de pause. Personnalisable par instance ou globalement via engine.defaults.onPause. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: onPause: (self) => void (Function), defaut noop
- S'execute quand un timer en cours est mis en pause
- Recoit l'instance du timer (self) en premier argument
- Override global: engine.defaults.onPause = self => console.log(self.id)
- Disponible depuis la version 4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onPause = self => console.log(self.id);
```

```js
import { createTimer, utils } from 'animejs';

const [ $resumeButton, $pauseButton ] = utils.$('.button');
const [ $paused ] = utils.$('.paused');
const [ $time ] = utils.$('.time');

let paused = 0;

const timer = createTimer({
  onPause: () => $paused.innerHTML = ++paused,
  onUpdate: self => $time.innerHTML = self.currentTime
});

const pauseTimer = () => timer.pause();
const resumeTimer = () => timer.resume();

$resumeButton.addEventListener('click', resumeTimer);
$pauseButton.addEventListener('click', pauseTimer);
```

### timer/timer-callbacks/then

`https://animejs.com/documentation/timer/timer-callbacks/then`

> then(callback) retourne une Promise qui se resout et execute un callback (recevant l'instance du timer) quand le timer se termine ; permet le pattern async/await.

La methode then(callback: Function): Promise retourne une Promise qui se resout et execute un callback quand le timer se termine. Le callback recoit l'instance du timer en premier argument. Elle s'utilise en invocation directe (createTimer({duration: 500}).then(callback)) ou dans un contexte async/await. Permet le controle de timer base sur Promise et le chainage sequentiel d'animations. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: then(callback: Function): Promise
- Retourne une Promise resolue a la fin du timer
- Le callback recoit l'instance du timer en premier argument
- Compatible async/await
- Permet le chainage sequentiel d'animations
- Disponible depuis la version 4.0.0

```js
createTimer({duration: 500}).then(callback);
```

```js
async function waitForTimerToComplete() {
  return createTimer({ duration: 250 })
}

const asyncTimer = await waitForTimerToComplete();
```

```js
import { createTimer, utils } from 'animejs';

const [ $status ] = utils.$('.status');
const [ $time ] = utils.$('.time');

createTimer({
  duration: 2000,
  onUpdate: self => $time.innerHTML = self.currentTime,
})
.then(() => $status.innerHTML = 'fulfilled');
```

```js
<div class="large row">
  <div class="col">
    <pre class="large log row">
      <span class="label">promise status</span>
      <span class="status value">pending</span>
    </pre>
  </div>
  <div class="col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
```

### timer/timer-methods

`https://animejs.com/documentation/timer/timer-methods`

> Page d'apercu listant les 12 methodes disponibles sur l'instance Timer retournee par createTimer(), donnant le controle du timing, du comportement et de la progression d'un timer.

Methodes disponibles sur l'instance Timer retournee par une fonction createTimer(), offrant le controle du timing, du comportement et de la progression d'un timer. La documentation liste 12 methodes : play(), reverse(), pause(), restart(), alternate(), resume(), complete(), reset(), cancel(), revert(), seek(), stretch(). Les signatures et exemples detailles de chaque methode figurent sur leurs pages individuelles.

**Faits clés**

- 12 methodes: play(), reverse(), pause(), restart(), alternate(), resume(), complete(), reset(), cancel(), revert(), seek(), stretch()
- Disponibles sur l'instance Timer retournee par createTimer()
- Controlent timing, comportement et progression du timer

### timer/timer-methods/play

`https://animejs.com/documentation/timer/timer-methods/play`

> play() force le timer a jouer vers l'avant et retourne l'instance du timer (chainable).

La methode play(): Timer force le timer a jouer vers l'avant. Elle retourne l'instance du timer, permettant le chainage de methodes avec d'autres operations de timer (reverse(), pause(), etc.). Pour demontrer son effet, le timer doit etre cree avec autoplay: false. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: play(): Timer
- Force le timer a jouer vers l'avant
- Retourne l'instance du timer (chainable)
- Necessite autoplay: false pour observer l'effet
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $playButton ] = utils.$('.play');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  duration: 2000,
  autoplay: false,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime,
});

const playTimer = () => timer.play();

$playButton.addEventListener('click', playTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button play">Play</button>
  </fieldset>
</div>
```


## animation

### animation

`https://animejs.com/documentation/animation`

> Page d'introduction a l'animation et a la fonction animate(targets, parameters) qui retourne une JSAnimation.

Les animations dans Anime.js modifient les valeurs de proprietes des elements cibles avec des parametres, callbacks et methodes de controle configurables. On specifie quoi animer et comment cela doit se comporter dans le temps. La fonction principale est animate(targets, parameters), importee depuis 'animejs'. Le parametre targets accepte des selecteurs CSS, des elements DOM, des objets JavaScript ou des tableaux. Le parametre parameters est un objet contenant les proprietes animables, les parametres de tween, les reglages de lecture (playback settings) et les callbacks. animate() retourne une JSAnimation. Une variante legere de 3KB basee sur la Web Animation API est disponible via waapi.animate(targets, parameters). Sous-sections de la documentation animation : Targets, Animatable properties, Tween value types, Tween parameters, Keyframes, Playback settings, Callbacks, Methods, Properties.

**Faits clés**

- Signature: animate(targets, parameters) — importe depuis 'animejs'
- targets: selecteurs CSS, elements DOM, objets JS, ou tableaux
- parameters: objet de proprietes animables + tween parameters + playback settings + callbacks
- Retourne: JSAnimation
- Variante WAAPI legere 3KB: waapi.animate(targets, parameters)
- Sous-sections: Targets, Animatable properties, Tween value types, Tween parameters, Keyframes, Playback settings, Callbacks, Methods, Properties

```js
import { animate } from 'animejs';

const animation = animate(targets, parameters);
```

```js
import { animate, stagger, splitText } from 'animejs';

const { chars } = splitText('h2', { words: false, chars: true });

animate(chars, {
  y: [
    { to: '-2.75rem', ease: 'outExpo', duration: 600 },
    { to: 0, ease: 'outBounce', duration: 800, delay: 100 }
  ],
  rotate: {
    from: '-1turn',
    delay: 0
  },
  delay: stagger(50),
  ease: 'inOutCirc',
  loopDelay: 1000,
  loop: true
});
```

```js
import { waapi } from 'animejs';

const animation = waapi.animate(targets, parameters);
```

### animation/targets

`https://animejs.com/documentation/animation/targets`

> Les targets sont le premier argument de animate() et definissent les elements auxquels les changements de valeurs de proprietes sont appliques.

Les targets sont specifies comme premier argument de la fonction animate(). Ils definissent les elements auxquels les changements de valeurs de proprietes sont appliques. Quatre categories de targets sont supportees: CSS Selector (selection par chaine), DOM Elements (references directes), JavaScript Objects (objets JS simples avec proprietes animables), Array of targets (plusieurs targets en un seul appel). Chaque type de target dispose d'une page dediee.

**Faits clés**

- Target = premier argument de animate()
- 4 types de targets: CSS Selector, DOM Elements, JavaScript Objects, Array of targets
- Chaque type a une page dediee

```js
animate(
  '.square',  // Target
  {
    translateX: 100,
    scale: 2,
    opacity: .5,
    duration: 400,
    delay: 250,
    ease: 'out(3)',
    loop: 3,
    alternate: true,
    autoplay: false,
    onBegin: () => {},
    onLoop: () => {},
    onUpdate: () => {},
  }
);
```

### animation/targets/css-selector

`https://animejs.com/documentation/animation/targets/css-selector`

> Permet d'animer un ou plusieurs elements DOM via une chaine de selecteur CSS standard acceptee par document.querySelectorAll().

Type: String. Accepte toute valeur de chaine acceptee par document.querySelectorAll(). Parametre requis (pas de valeur par defaut). Le target CSS Selector permet d'animer un ou plusieurs elements DOM via la syntaxe de selecteur CSS standard. Cette approche interroge le DOM avec la chaine fournie et applique l'animation a tous les elements correspondants simultanement. Supporte les selecteurs de classe (.classname), d'ID (#id), et les selecteurs complexes avec pseudo-classes (:nth-child()). Disponible depuis la version 1.0.0.

**Faits clés**

- Type: String
- Accepte toute valeur que document.querySelectorAll() accepte
- Cible tous les elements correspondants simultanement
- Depuis version 1.0.0

```js
import { animate } from 'animejs';

animate('.square', { x: '17rem' });
animate('#css-selector-id', { rotate: '1turn' });
animate('.row:nth-child(3) .square', { scale: [1, .5, 1] });
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div id="css-selector-id" class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
```

### animation/targets/dom-elements

`https://animejs.com/documentation/animation/targets/dom-elements`

> Anime des elements DOM directement en passant des references d'elements ou des collections (NodeList) a animate().

La signature est animate(target, properties) ou target accepte: HTMLElement, SVGElement, SVGGeometryElement, NodeList. Cette methode de ciblage anime un ou plusieurs elements DOM directement en passant des references d'elements ou des collections de noeuds a la fonction animate. Fonctionne avec des elements uniques et des collections (NodeList), et supporte les elements SVG en plus des elements HTML standards. Disponible depuis la version 1.0.0.

**Faits clés**

- Accepte: HTMLElement, SVGElement, SVGGeometryElement, NodeList
- Fonctionne avec elements uniques et collections (NodeList)
- Supporte les elements SVG
- Depuis version 1.0.0

```js
import { animate } from 'animejs';

const $demo = document.querySelector('#selector-demo');
const $squares = $demo.querySelectorAll('.square');

animate($demo, { scale: .75 });
animate($squares, { x: '23rem' });
```

```js
<div id="selector-demo">
  <div class="medium row">
    <div class="square"></div>
  </div>
  <div class="medium row">
    <div class="square"></div>
  </div>
  <div class="medium row">
    <div class="square"></div>
  </div>
</div>
```

### animation/targets/javascript-objects

`https://animejs.com/documentation/animation/targets/javascript-objects`

> Permet d'animer les proprietes d'instances d'Object JavaScript et d'instances de classe.

Anime.js permet d'animer les instances d'Object JavaScript et les instances de classe en ciblant leurs proprietes. Accepte: Object, Instance of Class. L'exemple anime un vecteur 2D de {x: 0, y: 0} vers {x: 100, y: 150}. L'utilitaire modifier (utils.round(0)) arrondit les valeurs en entiers, tandis que le callback onUpdate logge l'etat courant en JSON pour observer la progression de l'animation en temps reel. Permet d'animer n'importe quelle propriete numerique d'objet, utiliser des callbacks pour reagir aux changements, compatible avec les instances de classe personnalisees, supporte les modifiers pour transformer les valeurs animees.

**Faits clés**

- Accepte: Object, Instance of Class
- Anime les proprietes numeriques des objets
- Compatible avec instances de classe personnalisees
- Supporte les modifiers

```js
import { animate, utils } from 'animejs';

const [ $log ] = utils.$('code');

const vector2D = { x: 0, y: 0 };

animate(vector2D, {
  x: 100,
  y: 150,
  modifier: utils.round(0),
  onUpdate: () => $log.textContent = JSON.stringify(vector2D),
});
```

### animation/targets/array-of-targets

`https://animejs.com/documentation/animation/targets/array-of-targets`

> Cible plusieurs targets valides simultanement en les regroupant dans un Array; tous types de targets peuvent etre melanges.

Signature: animate(Array<Target>, animationProperties). Cible plusieurs targets valides simultanement en les regroupant dans un Array. Tous types de targets peuvent etre regroupes ensemble (heterogenes). Permet d'animer differents types de targets (selecteurs CSS, elements DOM, objets JavaScript, etc.) en un seul appel animate en les passant comme elements du tableau. Tous les types de targets documentes separement sont compatibles. Utile pour coordonner des animations a travers differents types d'elements et d'objets simultanement.

**Faits clés**

- Signature: animate(Array<Target>, animationProperties)
- Accepte des types de targets heterogenes dans un seul tableau
- Tous les types de targets documentes sont compatibles

```js
import { animate, utils } from 'animejs';

const [ $log ] = utils.$('code');

const vector2D = { x: 0, y: 0 };

animate([vector2D, '.square'], {
  x: '17rem',
  modifier: utils.roundPad(2).padStart(5, '0'),
  onRender: () => $log.textContent = JSON.stringify(vector2D),
});
```

### animation/animatable-properties

`https://animejs.com/documentation/animation/animatable-properties`

> Les proprietes animables sont definies dans l'objet de parametres de animate() et determinent quels aspects des targets peuvent etre animes.

Les proprietes animables sont definies dans l'Object de parametres de la fonction animate() et determinent quels aspects des elements targets peuvent etre animes. Six categories principales: CSS Properties (attributs de style CSS standards), CSS Transforms (proprietes de transformation comme translateX, scale), CSS Variables (proprietes CSS personnalisees), JavaScript Object Properties (proprietes numeriques d'objets JS), HTML Attributes (attributs d'elements HTML standards), SVG Attributes (attributs specifiques aux SVG). Les proprietes animables (translateX, scale, opacity) sont placees aux cotes des parametres de controle de l'animation (duration, delay, callbacks) dans l'objet de configuration. Disponible depuis 1.0.0.

**Faits clés**

- 6 categories: CSS Properties, CSS Transforms, CSS Variables, JavaScript Object Properties, HTML Attributes, SVG Attributes
- Definies dans l'Object de parametres de animate()
- Melangees avec les parametres de controle (duration, delay, callbacks)
- Depuis version 1.0.0

```js
animate('.square', {
  translateX: 100,
  scale: 2,
  opacity: .5,
  duration: 400,
  delay: 250,
  ease: 'out(3)',
  loop: 3,
  alternate: true,
  autoplay: false,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### animation/animatable-properties/css-properties

`https://animejs.com/documentation/animation/animatable-properties/css-properties`

> Toute propriete CSS numerique ou de couleur peut etre animee; les noms avec tirets doivent etre en camelCase ou en chaine.

Toute propriete CSS numerique et de couleur peut etre animee. Les proprietes avec tirets (comme background-color) doivent etre converties en camelCase (backgroundColor) ou ecrites comme des chaines. Note de performance: la plupart des proprietes CSS peuvent declencher des changements de layout ou des repaints, resultant en animations saccadees. La documentation insiste: toujours prioriser opacity et les CSS transforms autant que possible pour des resultats plus fluides.

**Faits clés**

- Valeurs numeriques et de couleur supportees
- Noms avec tirets: camelCase ou notation chaine requise
- Gotcha perf: prioriser opacity et CSS transforms pour eviter layout/repaint saccades

```js
import { animate } from 'animejs';

animate('.square', {
  left: 'calc(7.75rem * 2)',
  borderRadius: 64,
  'background-color': '#F9F640',
  filter: 'blur(5px)',
});
```

```js
<div class="large row">
  <div class="square"></div>
</div>
```

### animation/animatable-properties/css-transforms

`https://animejs.com/documentation/animation/animatable-properties/css-transforms`

> Les proprietes de transform individuelles peuvent etre animees via animate() et waapi.animate(); ordre de rendu fixe perspective->translate->rotate->scale->skew.

Les proprietes de transform CSS individuelles peuvent etre animees directement avec la methode JS animate() et la methode WAAPI waapi.animate(), offrant plus de controle que les animations CSS natives. Contrainte cle: les transforms suivent un ordre de rendu fixe: perspective -> translate -> rotate -> scale -> skew, quel que soit l'ordre dans lequel elles sont definies dans les parametres d'animation. Proprietes valides (avec raccourci, defaut, unite): translateX (x, '0px', 'px'); translateY (y, '0px', 'px'); translateZ (z, '0px', 'px'); rotate ('0deg', 'deg'); rotateX ('0deg', 'deg'); rotateY ('0deg', 'deg'); rotateZ ('0deg', 'deg'); scale ('1'); scaleX ('1'); scaleY ('1'); scaleZ ('1'); skew ('0deg', 'deg'); skewX ('0deg', 'deg'); skewY ('0deg', 'deg'); perspective ('0px', 'px'). Les proprietes d'axes adjacents se regroupent automatiquement en raccourci CSS (translateX + translateY -> translate(x, y)). La methode JS ne parse pas les transforms depuis les declarations CSS; utiliser utils.set() pour definir des valeurs de transform inline avant l'animation. Les transforms individuelles WAAPI requierent le support navigateur de CSS.registerProperty() et retombent sur aucune animation si non supporte.

**Faits clés**

- Ordre de rendu fixe: perspective -> translate -> rotate -> scale -> skew (independant de l'ordre defini)
- Defauts: translate* '0px', rotate*/skew* '0deg', scale* '1', perspective '0px'
- Raccourcis: translateX=x, translateY=y, translateZ=z
- Axes adjacents regroupes en raccourci CSS automatiquement
- JS ne parse pas les transforms depuis CSS; utiliser utils.set() pour valeurs inline
- Transforms individuelles WAAPI requierent CSS.registerProperty(), sinon fallback aucune animation

```js
import { animate, waapi } from 'animejs';

animate('.square', {
  x: '15rem', // TranslateX shorthand
  scale: 1.25,
  skew: -45,
  rotate: '1turn',
});

// WAAPI version recommended for animating transform property directly
waapi.animate('.square', {
  transform: 'translateX(15rem) scale(1.25) skew(-45deg) rotate(1turn)',
});
```

### animation/animatable-properties/css-variables

`https://animejs.com/documentation/animation/animatable-properties/css-variables`

> Les variables CSS a valeur numerique ou couleur peuvent etre animees en passant le nom de la variable comme chaine; permet d'animer des pseudo-elements.

Les variables CSS a valeur numerique ou de couleur peuvent etre animees en passant le nom de la variable comme chaine aux parametres d'animation. Cela permet d'animer des proprietes sur des pseudo-elements comme ::after et ::before. Important: pour animer des variables CSS avec la methode waapi.animate() basee sur WAAPI, il faut d'abord utiliser CSS.registerProperty(propertyDefinition); sinon cela retombe sur aucune animation. Dans l'exemple, l'utilisation d'une fonction (() => 'var(--radius)') empeche la conversion des variables.

**Faits clés**

- Nom de variable CSS passe comme chaine aux parametres
- Permet d'animer pseudo-elements ::after / ::before
- Gotcha WAAPI: CSS.registerProperty(propertyDefinition) requis sinon aucune animation
- Utiliser une fonction (() => 'var(--radius)') empeche la conversion des variables

```js
import { animate, utils } from 'animejs';

// Assign the CSS variables to the properties of the animated elements
utils.set('.square', {
  '--radius': '4px',
  '--x': '0rem',
  '--pseudo-el-after-scale': '1', // applied to the pseudo element "::after"
  // Using a function prevents the variables from being converted
  borderRadius: () => 'var(--radius)',
  translateX: () => 'var(--x)',
});

// Animate the values of the CSS variables
animate('.square', {
  '--radius': '20px',
  '--x': '16.5rem',
  '--pseudo-el-after-scale': '1.55' // Animates the ":after" pseudo element
});
```

```js
<div class="medium row">
  <div class="css-variables square"></div>
</div>
```

```js
.demo .css-variables.square:after {
  position: absolute;
  opacity: .5;
  top: 0;
  left: 0;
  content: "";
  display: block;
  width: 100%;
  height: 100%;
  background: currentColor;
  border-radius: inherit;
  transform: scale(var(--pseudo-el-after-scale));
}
```

### animation/animatable-properties/javascript-object-properties

`https://animejs.com/documentation/animation/animatable-properties/javascript-object-properties`

> Les proprietes numeriques et de couleur d'un Object JavaScript peuvent etre passees directement aux parametres d'animation.

Les proprietes numeriques et de couleur d'un Object JavaScript peuvent etre passees directement aux parametres d'animation pour animer des valeurs d'objet personnalisees. L'exemple anime des proprietes d'objet JS simples - valeurs numeriques et chaines avec unites. Le callback onRender met a jour l'affichage avec l'etat courant de l'objet a mesure que l'animation progresse. L'utilitaire modifier (utils.round(0)) arrondit les valeurs a l'entier le plus proche durant l'animation.

**Faits clés**

- Proprietes numeriques et de couleur d'un Object passees directement
- Supporte valeurs numeriques et chaines avec unites (ex '42%' -> '100%')
- onRender met a jour l'affichage durant l'animation
- modifier (utils.round(0)) arrondit les valeurs

```js
import { animate, utils } from 'animejs';

const myObject = {
  number: 1337,
  unit: '42%',
}

const [ $log ] = utils.$('code');

animate(myObject, {
  number: 50,
  unit: '100%',
  modifier: utils.round(0),
  onRender: function() {
    $log.innerHTML = JSON.stringify(myObject);
  }
});
```

```js
<pre class="row large centered">
  <code>{"number":1337,"unit":"42%"}</code>
</pre>
```

### animation/animatable-properties/html-attributes

`https://animejs.com/documentation/animation/animatable-properties/html-attributes`

> Les attributs HTML numeriques et de couleur peuvent etre animes en les passant directement comme parametres d'animation.

Anime.js permet d'animer les attributs HTML numeriques et de couleur en les passant directement comme parametres d'animation. Noms de propriete: tout attribut HTML valide (ex value, attributs data-*). Type: valeurs numeriques ou de couleur. Defaut: depend de l'etat initial de l'attribut. Les attributs HTML contenant des donnees numeriques ou de couleur peuvent etre animes avec la meme syntaxe que les proprietes CSS. La librairie detecte et interpole automatiquement ces valeurs d'attribut sur la duree de l'animation. Peut etre combine avec d'autres parametres comme alternate, loop et modifier. Disponible depuis la version 1.0.0.

**Faits clés**

- Attributs HTML numeriques et de couleur passes directement
- Defaut depend de l'etat initial de l'attribut
- Fonctionne avec value, data-* etc.
- Combinable avec alternate, loop, modifier
- Depuis version 1.0.0

```js
import { animate, utils } from 'animejs';

animate('input', {
  value: 1000, // animate the input "value" attribute
  alternate: true,
  loop: true,
  modifier: utils.round(0),
});
```

```js
<pre class="row large centered">
  <input type="range" value="0" min="0" max="1000" />
  <input type="text" value="0" size="5"/>
</pre>
```

### animation/animatable-properties/svg-attributes

`https://animejs.com/documentation/animation/animatable-properties/svg-attributes`

> Les attributs SVG numeriques et de couleur peuvent etre animes en les passant directement aux parametres d'animation.

Les attributs SVG numeriques et de couleur peuvent etre animes en les passant directement aux parametres d'animation. L'exemple anime baseFrequency et scale sur feTurbulence/feDisplacementMap, et points sur un polygon. Pour des animations SVG plus avancees, la documentation recommande d'utiliser les methodes utilitaires SVG integrees (morphTo, createDrawable, createMotionPath).

**Faits clés**

- Attributs SVG numeriques et de couleur passes directement
- Anime des attributs comme baseFrequency, scale, points
- Pour animations avancees: utiliser morphTo, createDrawable, createMotionPath

```js
import { animate } from 'animejs';

animate(['feTurbulence', 'feDisplacementMap'], {
  baseFrequency: .05,
  scale: 15,
  alternate: true,
  loop: true
});

animate('polygon', {
  points: '64 68.64 8.574 100 63.446 67.68 64 4 64.554 67.68 119.426 100',
  alternate: true,
  loop: true
});
```

```js
<svg width="128" height="128" viewBox="0 0 128 128">
  <filter id="displacementFilter">
    <feTurbulence type="turbulence" numOctaves="2" baseFrequency="0" result="turbulence"/>
    <feDisplacementMap in2="turbulence" in="SourceGraphic" scale="1" xChannelSelector="R" yChannelSelector="G"/>
  </filter>
  <polygon points="64 128 8.574 96 8.574 32 64 0 119.426 32 119.426 96" fill="currentColor"/>
</svg>
```

```js
.demo polygon {
  filter: url(#displacementFilter)
}
```

### animation/tween-value-types

`https://animejs.com/documentation/animation/tween-value-types`

> Specifie les valeurs de depart et de fin definissant l'animation des proprietes animables; sept categories de types de valeurs.

Cette page documente comment specifier les valeurs de depart (start) et de fin (end) qui definissent l'animation des proprietes animables. Les valeurs d'animation sont assignees aux proprietes animables et supportent plusieurs syntaxes. Sept categories de types de valeurs de tween sont listees: Numerical value, Unit conversion value, Relative value, Color value, Color function value, CSS variable, Function based value. Les specifications detaillees de chaque type sont documentees sur des pages liees separees. Introduite en version 1.0.0.

**Faits clés**

- Definit les valeurs start et end des proprietes animables
- 7 types: Numerical value, Unit conversion value, Relative value, Color value, Color function value, CSS variable, Function based value
- Chaque type documente sur une page separee
- Depuis version 1.0.0

```js
animate('.square', {
  x: '6rem',
  y: $el => $el.dataset.y,
  scale: '+=.25',
  opacity: {
    from: .4,
  },
});
```

### animation/tween-value-types/numerical-value

`https://animejs.com/documentation/animation/tween-value-types/numerical-value`

> Type de valeur de tween acceptant un Number brut ou une String contenant au moins un nombre pour animer les proprietes numeriques.

Une valeur numerique anime les proprietes numeriques en acceptant soit un Number brut, soit une String contenant au moins un nombre. Quand aucune unite n'est specifiee pour des proprietes qui attendent une unite (comme width), le navigateur applique son unite par defaut (typiquement les pixels). La methode JS animate() herite des unites precedemment definies sur la meme propriete de la meme cible : si l'on anime width: '50%' puis plus tard width: 75, le second appel devient '75%'. La methode WAAPI animate() applique automatiquement 'px' par defaut uniquement pour certaines proprietes : proprietes de position/dimension comme x, translateX, width, height, margin, padding, borderWidth, borderRadius, fontSize, etc.

**Faits clés**

- Signature : animate(target, { propertyName: Number | String })
- L'heritage d'unite ne s'applique qu'a la methode JS, pas a WAAPI
- WAAPI possede une liste predefinie de proprietes qui passent par defaut en pixels (x, translateX, width, height, margin, padding, borderWidth, borderRadius, fontSize, etc.)
- Omettre l'unite sur des proprietes qui ne defaultent pas peut produire des resultats inattendus

```js
import { waapi } from 'animejs';

waapi.animate('.square', {
  x: 240, //  -> 240px
  width: 75, // -> 75px
  rotate: '.75turn',
});
```

```js
<div class="large row">
  <div class="square"></div>
</div>
```

### animation/tween-value-types/unit-conversion-value

`https://animejs.com/documentation/animation/tween-value-types/unit-conversion-value`

> Type de valeur de tween (String) permettant d'animer vers une unite differente de l'unite par defaut ou de l'unite actuellement appliquee.

Ce type de valeur de tween permet d'animer vers des valeurs cibles dans des unites differentes de l'unite par defaut ou de l'unite actuellement appliquee. Il est particulierement utile pour animer des proprietes CSS vers des unites comme les pourcentages, rem ou turn qui different de l'etat d'unite initial de l'element. Type : String.

**Faits clés**

- Type : String, aucune valeur par defaut specifiee
- Caveat : avec la methode JS animate(), les conversions d'unite peuvent parfois produire des resultats inattendus selon le type d'unite et les proprietes animees
- Pratique recommandee : definir les unites hors animation via utils.set() puis animer vers l'unite courante pour des resultats plus previsibles
- Alternative : utiliser la methode WAAPI animate() pour une gestion plus fiable des conversions d'unite

```js
import { animate, utils } from 'animejs';

animate('.square', {
  width: '25%', // from '48px' to '25%',
  x: '15rem', // from '0px' to '15rem',
  rotate: '.75turn', // from `0deg` to '.75turn',
});
```

### animation/tween-value-types/relative-value

`https://animejs.com/documentation/animation/tween-value-types/relative-value`

> Valeurs relatives qui modifient la valeur actuelle de la cible par addition, soustraction ou multiplication via des prefixes de string.

Les valeurs relatives modifient la valeur actuelle de la cible par addition, soustraction ou multiplication en utilisant des prefixes de string. Prefixe '+=' pour l'addition (ex. '+=45', '+=45px'), '-=' pour la soustraction (ex. '-=45', '-=45deg'), '*=' pour la multiplication (ex. '*=.5'). Fonctionne avec des valeurs unitless comme avec des valeurs unitees.

**Faits clés**

- Prefixe '+=' = addition (ex. '+=45', '+=45px')
- Prefixe '-=' = soustraction (ex. '-=45', '-=45deg')
- Prefixe '*=' = multiplication (ex. '*=.5')
- Disponible depuis la version 2.0.0
- Fonctionne avec valeurs unitless et unitees

```js
import { animate, utils } from 'animejs';

const [ $clock ] = utils.$('.clock');
const [ $add ] = utils.$('.add');
const [ $sub ] = utils.$('.sub');
const [ $mul ] = utils.$('.mul');

const add = () => animate($clock, { rotate: '+=90' });
const sub = () => animate($clock, { rotate: '-=90' });
const mul = () => animate($clock, { rotate: '*=.5' });

$add.addEventListener('click', add);
$sub.addEventListener('click', sub);
$mul.addEventListener('click', mul);
```

```js
<div class="large centered row">
  <div class="clock"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button add">+ 90°</button>
    <button class="button sub">- 90°</button>
    <button class="button mul">× .5</button>
  </fieldset>
</div>
```

### animation/tween-value-types/color-value

`https://animejs.com/documentation/animation/tween-value-types/color-value`

> Anime.js supporte l'animation des proprietes de couleur via plusieurs formats CSS standard (HEX, HEXA, RGB, RGBA, HSL, HSLA, couleurs nommees).

Anime.js supporte l'animation des proprietes de couleur en utilisant plusieurs formats standard. Ces valeurs peuvent etre parsees et animees sur toute propriete de couleur animable. Formats supportes : HEX ('#F44' ou '#FF4444'), HEXA ('#F443' ou '#FF444433'), RGB ('rgb(255, 168, 40)'), RGBA ('rgba(255, 168, 40, .2)'), HSL ('hsl(255, 168, 40)'), HSLA ('hsla(255, 168, 40, .2)'), et couleurs nommees ('red' ou 'aqua', support WAAPI). Fonctionne avec n'importe quelle propriete de couleur animable (background, color, border-color, etc.). Tous les formats de couleur CSS standard sont parses automatiquement.

**Faits clés**

- Formats : HEX, HEXA, RGB, RGBA, HSL, HSLA, couleurs nommees (support WAAPI)
- Disponible depuis la version 1.0.0
- Fonctionne avec toute propriete de couleur animable (background, color, border-color, etc.)
- Tous les formats de couleur CSS standard sont parses automatiquement

```js
import { animate } from 'animejs';

animate('.hex',  {
  background: '#FF4B4B',
});

animate('.hexa', {
  background: '#FF4B4B33',
});
```

```js
animate('.rgb',  {
  background: 'rgb(255, 168, 40)',
});

animate('.rgba', {
  background: 'rgba(255, 168, 40, .2)',
});
```

```js
animate('.hsl',  {
  background: 'hsl(44, 100%, 59%)',
});

animate('.hsla', {
  background: 'hsla(44, 100%, 59%, .2)',
});
```

### animation/tween-value-types/color-function-value

`https://animejs.com/documentation/animation/tween-value-types/color-function-value`

> La fonction CSS color() peut etre animee via la methode WAAPI animate(), supportant toute syntaxe d'espace de couleur CSS valide.

La fonction CSS color() peut etre animee en utilisant la methode WAAPI animate() d'Anime.js, supportant toute syntaxe d'espace de couleur CSS valide. Cette fonctionnalite permet d'animer des couleurs specifiees avec la notation de la fonction CSS color(). Elle accepte toute syntaxe d'espace de couleur CSS valide telle que definie par la specification CSS, permettant de travailler avec des espaces de couleur avances comme display-p3. Fonctionne exclusivement avec la methode WAAPI animate().

**Faits clés**

- Signature : waapi.animate(target, { backgroundColor: 'color(display-p3 1.0 0.267 0.267 / 1.0)' })
- Disponible depuis la version 4.0.0
- Fonctionne exclusivement avec la methode WAAPI animate() (marque indicateur WAAPI)
- Accepte toute syntaxe d'espace de couleur CSS valide (ex. display-p3)

```js
import { waapi } from 'animejs';

waapi.animate('.circle',  {
  backgroundColor: 'color(display-p3 1.0 0.267 0.267 / 1.0)',
});
```

### animation/tween-value-types/css-variable

`https://animejs.com/documentation/animation/tween-value-types/css-variable`

> Les variables CSS peuvent etre animees en passant le nom de la variable via la syntaxe 'var(--my-value)'.

Les variables CSS peuvent etre animees en passant le nom de la variable via la syntaxe 'var(--my-value)'. Le parametre accepte la syntaxe standard des custom properties CSS via la fonction var(). Quand on utilise la fonction JavaScript animate() (et non WAAPI), la librairie calcule la valeur courante de la variable au demarrage de l'animation. Si la variable est mise a jour de l'exterieur durant l'animation, il faut appeler .refresh() pour recalculer et animer vers la nouvelle valeur.

**Faits clés**

- Parametre : var(--variable-name), String CSS variable
- Syntaxe : 'var(--my-value)'
- En JS animate(), la valeur de la variable est calculee au demarrage de l'animation
- Pour animer une variable CSS mise a jour en JS, appeler .refresh() (apres .restart() si besoin) pour recalculer la nouvelle valeur

```js
import { waapi, animate, stagger } from 'animejs';

waapi.animate('.square',  {
  rotate: 'var(--rotation)',
  borderColor: ['var(--hex-orange-1)', 'var(--hex-red-1)'],
  duration: 500,
  delay: stagger(100),
  loop: true,
});

animate('.square',  {
  scale: 'var(--scale)',
  background: ['var(--hex-red-1)', 'var(--hex-orange-1)'],
  duration: 500,
  delay: stagger(100),
  loop: true,
  alternate: true,
});
```

```js
target.style.setProperty('--x', '100px');
const anim = animate(target, { x: 'var(--x)' });
target.style.setProperty('--x', '200px');
anim.restart().refresh();
```

### animation/tween-value-types/function-based

`https://animejs.com/documentation/animation/tween-value-types/function-based`

> Valeurs basees sur une fonction evaluee par cible permettant des valeurs d'animation differentes pour chaque cible dans les animations multi-cibles.

Les valeurs function-based permettent des valeurs d'animation differentes pour chaque cible dans les animations multi-cibles en acceptant une fonction evaluee par cible. Signature : (target, index, targets, prevTween) => value. Parametres : target (l'element/objet anime courant), index (position zero-based dans le tableau de cibles), targets (Array de toutes les cibles animees), prevTween (valeur de fin calculee du tween precedent pour la meme cible/propriete). La valeur de retour doit etre soit une Tween value, soit des Tween parameters. Les valeurs peuvent etre recalculees dynamiquement via animation.refresh().

**Faits clés**

- Signature : (target, index, targets, prevTween) => value
- target = cible animee courante ; index = position zero-based ; targets = Array de toutes les cibles ; prevTween = valeur de fin calculee du tween precedent (meme cible/propriete)
- La valeur de retour doit etre une Tween value ou des Tween parameters
- Migration v4.4.0+ : le 3e parametre est passe de total (Number) a targets (Array) ; remplacer total par targets.length
- Les valeurs peuvent etre recalculees via animation.refresh()

```js
import { animate, utils } from 'animejs';

animate('.square', {
  x: $el => $el.getAttribute('data-x'),
  y: (_, i) => 50 + (-50 * i),
  scale: (_, i, t) => (t.length - i) * .75,
  rotate: () => utils.random(-360, 360),
  borderRadius: () => `+=${utils.random(0, 8)}`,
  duration: () => utils.random(1200, 1800),
  delay: () => utils.random(0, 400),
  ease: 'outElastic(1, .5)',
});
```

### animation/tween-parameters

`https://animejs.com/documentation/animation/tween-parameters`

> Les tween parameters configurent les valeurs animees, le timing et les comportements, definissables globalement ou localement par propriete.

Les tween parameters configurent les valeurs de propriete animees, le timing et les comportements. Ils peuvent etre definis globalement (s'appliquant a toutes les proprietes) ou localement (pour des proprietes specifiques via un objet). Local tween parameters : appliques a des proprietes individuelles via la syntaxe objet. Global tween parameters : appliques a toutes les proprietes de la config d'animation. Concept cle : toutes les proprietes animables heritent des parametres globaux, qui peuvent etre surchargees localement pour un tween specifique. Parametres disponibles (menu) : to, from, delay, duration, ease, composition (feature JS), modifier (feature JS).

**Faits clés**

- Parametres : to (valeur cible), from (valeur de depart), delay, duration, ease, composition (JS), modifier (JS)
- Toutes les proprietes animables heritent des parametres globaux, surchargeables localement pour un tween specifique
- Definition locale via syntaxe objet par propriete ; definition globale au niveau de la config d'animation

```js
animate('.square', {
  x: {
    to: 100,
    delay: 0,
    ease: 'inOut(4)'
  },
  scale: 1,
  opacity: .5,
  duration: 400,
  delay: 250,
  ease: 'out(3)',
  loop: 3,
  alternate: true,
});
```

### animation/tween-parameters/to

`https://animejs.com/documentation/animation/tween-parameters/to`

> Parametre tween specifiant la valeur de fin de l'animation, depuis la valeur courante de la cible vers la valeur 'to'.

Le parametre to specifie la valeur de fin d'une animation. L'animation progresse depuis la valeur courante de la cible vers la valeur to specifiee. Il doit etre place dans un objet de parametres de tween local. Le parametre supporte une syntaxe en tableau permettant de definir a la fois le point de depart et le point de fin sous la forme [fromValue, toValue]. Type : toute Tween value type valide ou Array de deux keyframes. Requis sauf si la propriete from est definie ; valeur par defaut = valeur courante de la cible (si from est defini).

**Faits clés**

- Nom : to ; Type : toute Tween value type valide ou Array de deux keyframes
- Default : valeur courante de la cible (si from est defini)
- Requis : oui, sauf si la propriete from est definie
- Doit etre place dans un objet de parametres de tween local
- Quand seul to est fourni, l'animation part de la valeur courante
- Peut etre couple avec from pour un controle explicite de la plage

```js
import { animate } from 'animejs';

animate('.square', {
  x: {
    to: '16rem', // From 0px to 16rem
    ease: 'outCubic',
  },
  rotate: {
    to: '.75turn', // From 0turn to .75turn
    ease: 'inOutQuad'
  },
});
```

```js
<div class="large row">
  <div class="square"></div>
</div>
```

### animation/tween-parameters/from

`https://animejs.com/documentation/animation/tween-parameters/from`

> Parametre tween animant DEPUIS une valeur specifiee VERS la valeur courante de la cible.

Le parametre from anime depuis une valeur specifiee vers la valeur courante de la cible. Il doit etre defini dans un objet de parametres de tween local et n'est requis que si aucune propriete to n'est definie. Comportement par defaut : la valeur courante de la cible est utilisee si seule une propriete to est definie. Accepte toute Tween value type valide (numerical, unit conversion, relative, color, CSS variable, function-based, etc.). Le parametre from inverse la direction d'animation typique : au lieu d'animer vers une valeur, il anime depuis le point de depart specifie vers l'etat courant/par defaut de l'element.

**Faits clés**

- Nom : from ; Type : toute Tween value type valide
- Signature : from: <Tween value types>
- Requis uniquement si aucune propriete to n'est definie
- Default : la valeur courante de la cible est utilisee si seule la propriete to est definie
- Doit etre defini dans un objet de parametres de tween local
- Inverse la direction : anime depuis le point de depart vers l'etat courant/par defaut

```js
import { animate } from 'animejs';

animate('.square', {
  opacity: { from: .5 }, // Animate from .5 opacity to 1 opacity
  translateX: { from: '16rem' }, // From 16rem to 0rem
  rotate: {
    from: '-.75turn', // From -.75turn to 0turn
    ease: 'inOutQuad',
  },
});
```

```js
<div class="large row">
  <div class="square"></div>
</div>
```

### animation/tween-parameters/delay

`https://animejs.com/documentation/animation/tween-parameters/delay`

> Parametre definissant le delai en millisecondes au debut de toutes les proprietes animees, ou localement a une propriete specifique.

Le parametre delay definit le delai en millisecondes au debut de toutes les proprietes animees, ou localement a une propriete specifique. Applique globalement, il affecte toutes les proprietes de l'animation. Applique a des proprietes individuelles, il ne retarde que l'animation de cette propriete specifique. Type : Number (>= 0) ou fonction retournant un Number (>= 0). Default : 0 (ou personnalise via engine.defaults.delay).

**Faits clés**

- Nom : delay ; Signature : delay: Number | Function
- Type : Number (>= 0) ou fonction retournant Number (>= 0)
- Default : 0 (personnalisable via engine.defaults.delay)
- Applicable globalement (toutes proprietes) ou localement (propriete specifique)
- Depuis 1.0.0

```js
import { animate } from 'animejs';

const animation = animate('.square', {
  x: '17rem',
  rotate: {
    to: 360,
    delay: 1000, // Local delay applied only to rotate property
  },
  delay: 500,  // Global delay applied to all properties
  loop: true,
  alternate: true
});
```

```js
import { engine } from 'animejs';
engine.defaults.delay = 500;
```

### animation/tween-parameters/duration

`https://animejs.com/documentation/animation/tween-parameters/duration`

> Parametre specifiant la duree d'une animation en millisecondes, applicable globalement ou localement par propriete.

Le parametre duration specifie combien de temps une animation s'execute en millisecondes. Il peut etre applique globalement a toutes les proprietes ou localement a des proprietes individuelles. Type : Number ou Function. Default : 1000 (millisecondes). Valeurs acceptees : un nombre >= 0, ou une valeur function-based retournant un nombre >= 0. Les valeurs de duree superieures a 1e12 ou egales a Infinity sont clampees en interne a 1e12 (environ 32 ans).

**Faits clés**

- Nom : duration ; Type : Number ou Function
- Default : 1000 (millisecondes)
- Valeurs acceptees : nombre >= 0 ou fonction retournant un nombre >= 0
- Les durees > 1e12 ou = Infinity sont clampees en interne a 1e12 (environ 32 ans)
- Configurable globalement via engine.defaults.duration

```js
import { animate } from 'animejs';

const animation = animate('.square', {
  x: '17rem',
  rotate: {
    to: 360,
    duration: 1500, // Local duration only applied to rotate property
  },
  duration: 3000,  // Global duration applied to all properties
  loop: true,
  alternate: true
});
```

```js
import { engine } from 'animejs';
engine.defaults.duration = 500;
```

### animation/tween-parameters/ease

`https://animejs.com/documentation/animation/tween-parameters/ease`

> Parametre specifiant la fonction d'easing controlant l'evolution des valeurs de propriete animees dans le temps (acceleration/deceleration).

Le parametre ease specifie la fonction d'easing controlant comment les valeurs de propriete animees changent dans le temps. Il determine l'acceleration et la deceleration de l'animation tout au long de la lecture. On peut l'appliquer globalement a toutes les proprietes ou cibler des proprietes individuelles. Type : Easing Function | String | function-based value. Default : 'out(2)'. Formats d'entree supportes : noms de fonctions d'easing integrees sous forme de strings, fonctions d'easing depuis l'objet eases, et valeurs function-based retournant des fonctions d'easing ou des noms en string.

**Faits clés**

- Nom : ease ; Type : Easing Function | String | function-based value
- Default : 'out(2)'
- Formats d'entree : noms d'easing en string, fonctions de l'objet eases, valeurs function-based retournant fonctions ou noms d'easing
- Applicable globalement ou localement par propriete
- Configurable globalement via engine.defaults.ease

```js
import { animate, waapi, eases, spring } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'inQuad',
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  ease: eases.outQuad,
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: {
    to: 360,
    ease: 'out(6)',
  },
  ease: spring({ stiffness: 70 }),
});
```

```js
import { engine } from 'animejs';
engine.defaults.ease = 'outElastic(1, .5)';
```

### animation/tween-parameters/composition

`https://animejs.com/documentation/animation/tween-parameters/composition`

> Le parametre composition controle comment plusieurs animations ciblant la meme propriete sur le meme element interagissent (remplacer, preserver, melanger).

Le parametre `composition` determine comment des animations simultanees interagissent lorsque plusieurs animations ciblent la meme propriete sur le meme element. Il decide si une nouvelle animation remplace, preserve, ou melange avec une animation existante. Trois modes : 'replace' (valeur 0) annule et remplace l'animation en cours ; 'none' (valeur 1) preserve l'animation en cours, la nouvelle ne la remplace pas ; 'blend' (valeur 2) cree une animation additive qui melange les valeurs ensemble. La valeur par defaut est 'replace' (en dessous de 1000 cibles) et 'none' (a partir de 1000 cibles, version JS). On peut configurer le defaut global via engine.defaults.composition. Le mode 'blend' ne fonctionne qu'en jouant vers l'avant (forward) deux animations ou plus avec composition 'blend' simultanement, et est incompatible avec : keyframes multiples, valeurs de couleur, methode reverse(), parametre loop, parametre reversed, parametre alternate.

**Faits clés**

- Parametre: composition
- Type: String ou Number
- Defaut: 'replace' (< 1000 cibles) ; 'none' (>= 1000 cibles, version JS)
- Modes: 'replace' = 0 (annule et remplace), 'none' = 1 (preserve, ne remplace pas), 'blend' = 2 (additif, melange les valeurs)
- Defaut global: engine.defaults.composition
- Gotcha: 'blend' ne marche qu'en jouant FORWARD 2+ animations 'blend' simultanement
- Gotcha: 'blend' incompatible avec keyframes multiples, valeurs de couleur, reverse(), loop, reversed, alternate

```js
import { animate, utils } from 'animejs';

const squares = utils.$('.square');
const [ $none, $replace, $blend ] = squares;

// Animate each square with a different composition mode
squares.forEach($square => {
  const mode = $square.classList[1];
  animate($square, {
    scale: [.5, 1],
    alternate: true,
    loop: true,
    duration: 750,
    composition: mode,
  });
});

// Common animation parameters
const enter = { scale: 1.5, duration: 350 };
const leave = { scale: 1.0, duration: 250 };

// Composition 'none' animations
const enterNone = () => animate($none, {
  composition: 'none', ...enter
});

const leaveNone = () => animate($none, {
  composition: 'none', ...leave
});

$none.addEventListener('mouseenter', enterNone);
$none.addEventListener('mouseleave', leaveNone);

// Composition 'replace' animations
const enterReplace = () => animate($replace, {
  composition: 'replace', ...enter
});

const leaveReplace = () => animate($replace, {
  composition: 'replace', ...leave
});

$replace.addEventListener('mouseenter', enterReplace);
$replace.addEventListener('mouseleave', leaveReplace);

// Composition 'blend' animations
const enterBlend = () => animate($blend, {
  composition: 'blend', ...enter
});

const leaveBlend = () => animate($blend, {
  composition: 'blend', ...leave
});

$blend.addEventListener('mouseenter', enterBlend);
$blend.addEventListener('mouseleave', leaveBlend);
```

```js
import { engine } from 'animejs';
engine.defaults.composition = 'blend';
```

### animation/tween-parameters/modifier

`https://animejs.com/documentation/animation/tween-parameters/modifier`

> Le parametre modifier accepte une fonction qui transforme les valeurs numeriques animees pendant l'animation, globalement ou par propriete.

Le parametre `modifier` accepte une fonction qui transforme les valeurs numeriques animees pendant l'animation. Il peut etre applique globalement a toutes les proprietes ou localement a une propriete specifique. Lorsque la valeur finale comporte des unites (comme '100px'), la portion chaine est automatiquement ajoutee apres modification. La plupart des fonctions utilitaires de la bibliotheque fonctionnent comme modifiers. La signature de la fonction est (value: Number) => Number : value est la valeur numerique animee courante, et la fonction retourne une valeur numerique modifiee. On peut configurer le defaut global via engine.defaults.modifier.

**Faits clés**

- Parametre: modifier
- Type: Function | null
- Defaut: null
- Signature fonction: (value: Number) => Number
- Si la valeur finale a des unites (ex '100px'), la chaine d'unite est ajoutee apres modification
- La plupart des utils fonctionnent comme modifier (ex utils.round(0))
- Applicable globalement ou par propriete (ex y: { to:'70rem', modifier: ... })
- Defaut global: engine.defaults.modifier

```js
import { animate, utils } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  modifier: utils.round(0),
  duration: 4000,
});
```

```js
animate('.row:nth-child(2) .square', {
  x: '85rem',
  modifier: v => v % 17,
  duration: 4000,
});
```

```js
animate('.row:nth-child(3) .square', {
  x: '17rem',
  y: {
    to: '70rem',
    modifier: v => Math.cos(v) / 2,
  },
  duration: 4000,
});
```

```js
import { engine } from 'animejs';
engine.defaults.modifier = customFunction;
```

### animation/keyframes

`https://animejs.com/documentation/animation/keyframes`

> Les keyframes permettent de creer des sequences d'animation sur une meme propriete animable, via valeurs de tween, parametres de tween, ou keyframes au niveau animation (duration-based / percentage-based).

Les keyframes permettent de creer des sequences d'animation sur la meme propriete animable. Deux approches principales : (1) Property value keyframes - appliquees directement aux valeurs de propriete (tableau de valeurs de tween, ou tableau de parametres de tween) ; (2) Animation keyframes - definies au niveau de l'animation, animant plusieurs proprietes par keyframe (duration-based via tableau d'objets, ou percentage-based via objet a cles pourcentage). La documentation pointe vers les pages detaillees : Tween values keyframes, Tween parameters keyframes, Duration-based keyframes, Percentage-based keyframes.

**Faits clés**

- Deux familles: Property value keyframes (valeurs/parametres de tween par propriete) et Animation keyframes (au niveau animation)
- Property value - Tween Values Array: ex x: [0, 100, 200]
- Property value - Tween Parameters Array: ex x: [{to: 100}, {to: 200}]
- Animation keyframes - Duration based: keyframes: [ {...}, {...} ] (tableau d'objets)
- Animation keyframes - Percentage based: keyframes: { '0%':{...}, '50%':{...}, '100%':{...} }
- Pages liees: tween-values-keyframes, tween-parameters-keyframes, duration-based-keyframes, percentage-based-keyframes

```js
animate('.square', {
  x: [0, 100, 200],
  y: [0, 100, 200],
  duration: 3000,
})
```

```js
animate('.square', {
  x: [{to: 100}, {to: 200}],
  y: [{to: 100}, {to: 200}],
  duration: 3000,
})
```

```js
animate('.square', {
  keyframes: [
    { x: 100, y: 100 },
    { x: 200, y: 200 },
  ],
  duration: 3000,
})
```

```js
animate('.square', {
  keyframes: {
    '0%'  : { x: 0,   y: 0   },
    '50%' : { x: 100, y: 100 },
    '100%': { x: 200, y: 200 },
  },
  duration: 3000,
})
```

### animation/keyframes/tween-values-keyframes

`https://animejs.com/documentation/animation/keyframes/tween-values-keyframes`

> Sequence plusieurs valeurs de tween specifiques a une propriete animable via un Array ; le premier element est la valeur 'from'.

Permet de sequencer plusieurs valeurs de Tween specifiques a une propriete animable en utilisant un Array. La duree entre keyframes est egale a la duree totale de l'animation divisee par le nombre de transitions. Le premier element du tableau definit la valeur 'from' (point de depart de l'animation). Le type est un Array de valeurs de Tween valides. Syntaxe raccourcie pour definir la valeur initiale : animate(target: { x: [-100, 100] }) anime x de -100 a 100. L'easing se distribue uniformement entre les keyframes sauf surcharge par keyframe ; utiliser playbackEase pour appliquer un easing a travers toute la sequence de keyframes. Disponible depuis la version 4.0.0.

**Faits clés**

- Type: Array de valeurs de Tween valides
- Premier element du tableau = valeur 'from' (point de depart)
- Duree entre keyframes = duree totale / nombre de transitions
- Raccourci valeur initiale: x: [-100, 100] anime x de -100 a 100
- ease applique entre chaque keyframe si aucun ease defini par keyframe
- playbackEase applique l'easing a travers tous les keyframes
- Disponible depuis v4.0.0

```js
import { animate } from 'animejs';

animate('.square', {
  translateX: ['0rem', 0, 17, 17, 0, 0],
  translateY: ['0rem', -2.5, -2.5, 2.5, 2.5, 0],
  scale: [1, 1, .5, .5, 1, 1],
  rotate: { to: 360, ease: 'linear' },
  duration: 3000,
  ease: 'inOut', // ease applied between each keyframes if no ease defined
  playbackEase: 'ouIn(5)', // ease applied accross all keyframes
  loop: true,
});
```

```js
animate(target: { property: [value1, value2, value3, ...] });
```

```js
animate(target: { x: [-100, 100] }); // Animate x from -100 to 100
```

### animation/keyframes/tween-parameters-keyframes

`https://animejs.com/documentation/animation/keyframes/tween-parameters-keyframes`

> Sequence plusieurs parametres de tween (ease, delay, duration, modifier) specifiques a une propriete animable via un Array d'objets parametres.

Permet de sequencer plusieurs parametres de Tween specifiques a une propriete animable. Cette fonctionnalite offre un controle granulaire en exposant les reglages ease, delay, duration et modifier pour des keyframes individuels. Lorsqu'aucune duree explicite n'est definie, chaque keyframe recoit une portion egale de la duree totale de l'animation. Le type est un Array de parametres de Tween. Chaque objet keyframe accepte des parametres de timing individuels. Le parametre ease s'applique entre les keyframes lorsqu'il n'est pas defini explicitement par keyframe ; utiliser playbackEase pour appliquer un easing collectivement a travers tous les keyframes.

**Faits clés**

- Type: Array de parametres de Tween
- Parametres exposes par keyframe: ease, delay, duration, modifier
- Si aucune duration explicite: chaque keyframe recoit une part egale de la duree totale
- ease applique entre keyframes si non defini par keyframe
- playbackEase applique a travers tous les keyframes collectivement

```js
import { animate } from 'animejs';

animate('.square', {
  x: [
    { to: '17rem', duration: 700, delay: 400 },
    { to: 0, duration: 700, delay: 800 },
  ],
  y: [
    { to: '-2.5rem', ease: 'out', duration: 400 },
    { to: '2.5rem', duration: 800, delay: 700 },
    { to: 0, ease: 'in', duration: 400, delay: 700 },
  ],
  scale: [
    { to: .5, duration: 700, delay: 400 },
    { to: 1, duration: 700, delay: 800 },
  ],
  rotate: { to: 360, ease: 'linear' },
  duration: 3000,
  ease: 'inOut',
  playbackEase: 'ouIn(5)',
  loop: true,
});
```

### animation/keyframes/duration-based-keyframes

`https://animejs.com/documentation/animation/keyframes/duration-based-keyframes`

> keyframes en Array<Object> ou chaque objet contient des proprietes animables + parametres de tween ; les durees non specifiees valent duree totale / nombre de keyframes.

Sequence plusieurs proprietes animables l'une apres l'autre, offrant un controle granulaire du timing. Le parametre keyframes est un Array<Object> ou chaque objet contient une (ou plusieurs) propriete(s) animable(s) et des parametres de tween (ease, delay, duration, modifier). Lorsqu'on ne specifie pas de duree pour un keyframe individuel, elle est calculee automatiquement comme : duree totale de l'animation / nombre de keyframes. Le parametre ease s'applique entre les keyframes sauf surcharge par keyframe ; playbackEase s'applique a travers tous les keyframes comme une fonction d'easing maitre. Les durees de keyframe non specifiees repartissent le temps restant proportionnellement.

**Faits clés**

- Signature: keyframes: Array<Object>
- Chaque objet = propriete(s) animable(s) + parametres de tween (ease, delay, duration, modifier)
- Duree d'un keyframe non specifiee = duree totale / nombre de keyframes (ex 3000/5 = 600ms)
- ease applique entre chaque keyframe si aucun ease defini
- playbackEase = easing maitre a travers tous les keyframes

```js
import { animate } from 'animejs';

animate('.square', {
  keyframes: [
    { y: '-2.5rem', ease: 'out', duration: 400 },
    { x: '17rem', scale: .5, duration: 800 },
    { y: '2.5rem' }, // The duration here is 3000 / 5 = 600ms
    { x: 0, scale: 1, duration: 800 },
    { y: 0, ease: 'in', duration: 400 }
  ],
  rotate: { to: 360, ease: 'linear' },
  duration: 3000,
  ease: 'inOut', // ease applied between each keyframes if no ease defined
  playbackEase: 'ouIn(5)', // ease applied accross all keyframes
  loop: true,
});
```

### animation/keyframes/percentage-based-keyframes

`https://animejs.com/documentation/animation/keyframes/percentage-based-keyframes`

> keyframes en Object dont les cles sont des pourcentages ('0%','50%',...) ; syntaxe miroir de CSS @keyframes ; seul ease est configurable par keyframe.

Sequence plusieurs proprietes animables avec des positions definies par des pourcentages de la duree totale de l'animation. Le parametre keyframes est un Object ou les cles sont des chaines de pourcentage ('0%', '25%', etc.) et les valeurs sont des objets contenant des proprietes animables plus un parametre ease optionnel. La syntaxe reflete les @keyframes CSS. Le premier keyframe etablit la valeur de depart. Seul le parametre ease est configurable par keyframe ; les autres controles de tween ne sont pas disponibles dans ce format. Le parametre ease s'applique entre les keyframes lorsqu'il est specifie individuellement ; un ease de repli s'applique entre les keyframes qui n'ont pas d'easing explicite ; playbackEase s'applique a travers tous les keyframes globalement.

**Faits clés**

- Signature: keyframes: Object
- Cles = chaines de pourcentage ('0%','25%',...) ; valeurs = objets de proprietes animables + ease optionnel
- Syntaxe miroir de CSS @keyframes
- Premier keyframe etablit la valeur de depart
- Seul ease est configurable par keyframe ; autres controles de tween indisponibles dans ce format
- playbackEase applique globalement a travers tous les keyframes

```js
import { animate } from 'animejs';

animate('.square', {
  keyframes: {
    '0%'  : { x: '0rem', y: '0rem', ease: 'out' },
    '13%' : { x: '0rem', y: '-2.5rem', },
    '37%' : { x: '17rem', y: '-2.5rem', scale: .5 },
    '63%' : { x: '17rem', y: '2.5rem', scale: .5 },
    '87%' : { x: '0rem', y: '2.5rem', scale: 1 },
    '100%': { y: '0rem', ease: 'in' }
  },
  rotate: { to: 360, ease: 'linear' },
  duration: 3000,
  ease: 'inOut',
  playbackEase: 'ouIn(5)',
  loop: true,
});
```

### animation/animation-playback-settings

`https://animejs.com/documentation/animation/animation-playback-settings`

> Page d'introduction listant les reglages de lecture (timings et comportements) definis directement dans les parametres de animate().

Les reglages de lecture (Playback settings) specifient les timings et comportements d'une animation. Les proprietes de reglage de lecture sont definies directement dans l'objet de parametres de animate(). La documentation liste ces reglages : delay (attente initiale avant le demarrage), duration (longueur de l'animation), loop (nombre de repetitions), loopDelay (pause entre iterations de boucle), alternate (inverse la direction a chaque boucle), reversed (joue l'animation a l'envers), autoplay (demarrage automatique ou declenchement manuel), frameRate (frequence des images), playbackRate (multiplicateur de vitesse), playbackEase (easing applique a la vitesse de lecture), persist (reglage de persistance lie a WAAPI). Chaque reglage a sa propre page de documentation detaillee liee depuis cette section.

**Faits clés**

- Les playback settings sont definis directement dans l'objet de parametres de animate()
- Reglages listes: delay, duration, loop, loopDelay, alternate, reversed, autoplay, frameRate, playbackRate, playbackEase, persist
- persist est lie a WAAPI
- Chaque reglage a sa page de documentation detaillee

```js
animate('.square', {
  translateX: 100,
  scale: 2,
  opacity: .5,
  duration: 400,
  delay: 250,
  ease: 'out(3)',
  loop: 3,
  alternate: true,
  autoplay: false,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### animation/animation-playback-settings/delay

`https://animejs.com/documentation/animation/animation-playback-settings/delay`

> delay: Number | Function, defaut 0 ; delai initial en ms avant le demarrage, applique globalement a tous les tweens.

Definit le delai par defaut en millisecondes des tweens de l'animation. Ce parametre etablit combien de temps l'animation attend avant de commencer. Le delai s'applique globalement a toutes les proprietes de l'animation. Type Number (>= 0) ou valeur basee sur une fonction (retournant un nombre >= 0). Defaut 0. On peut modifier le defaut global via engine.defaults.delay.

**Faits clés**

- Parametre: delay
- Type: Number (>= 0) | Function retournant >= 0
- Defaut: 0
- Delai initial en ms avant demarrage, applique globalement a tous les tweens/proprietes
- Defaut global: engine.defaults.delay

```js
import { engine } from 'animejs';
engine.defaults.delay = 500;
```

```js
import { animate } from 'animejs';

const playbackDelay = animate('.delay', {
  x: '16rem',
  scale: 1.8,
  delay: 500, // Global delay applied to all properties
  loop: true,
  alternate: true
});
```

```js
<div class="medium row">
  <div class="circle delay"></div>
</div>
```

### animation/animation-playback-settings/duration

`https://animejs.com/documentation/animation/animation-playback-settings/duration`

> duration: Number | Function, defaut 1000 ; longueur en ms de tous les tweens ; 0 = completion instantanee ; clamp a 1e12.

Etablit la longueur par defaut de l'animation en millisecondes pour tous les tweens. Type Number | Function, defaut 1000. Definir duration a 0 complete instantanement les animations au lancement. Les valeurs depassant 1e12 sont internement plafonnees (clamped) a cette limite (~32 ans). Valeurs acceptees : un nombre >= 0, ou une valeur basee sur une fonction retournant un nombre >= 0. On peut modifier le defaut global via engine.defaults.duration.

**Faits clés**

- Parametre: duration
- Type: Number | Function
- Defaut: 1000
- duration: 0 = completion instantanee au lancement
- Valeurs > 1e12 plafonnees a 1e12 (~32 ans)
- Valeurs acceptees: nombre >= 0 ou fonction retournant >= 0
- Defaut global: engine.defaults.duration

```js
import { engine } from 'animejs';
engine.defaults.duration = 500;
```

```js
import { animate } from 'animejs';

animate('.dur-0', {
  x: '17rem',
  duration: 0,
});

animate('.dur-500', {
  x: '17rem',
  duration: 500,
});

animate('.dur-2000', {
  x: '17rem',
  duration: 2000
});
```

```js
<div class="medium row">
  <div class="circle dur-0"></div>
  <div class="padded label">duration: 0</div>
</div>
<div class="medium row">
  <div class="circle dur-500"></div>
  <div class="padded label">duration: 500</div>
</div>
<div class="medium row">
  <div class="circle dur-2000"></div>
  <div class="padded label">duration: 2000</div>
</div>
```

### animation/animation-playback-settings/loop

`https://animejs.com/documentation/animation/animation-playback-settings/loop`

> loop: Number | boolean | Infinity, defaut 0 ; nombre de repetitions ; true et -1 equivalent a Infinity.

Le parametre loop definit combien de fois une animation se repete. Il accepte des valeurs dans l'intervalle [0, Infinity], avec une gestion speciale pour les valeurs booleennes et infinies. Number = nombre specifique de boucles (0 a Infinity) ; Infinity = repetition indefinie ; true equivaut a Infinity ; -1 equivaut a Infinity. Defaut 0. On peut modifier le defaut global via engine.defaults.loop.

**Faits clés**

- Parametre: loop
- Type: Number | boolean | Infinity
- Defaut: 0
- Intervalle [0, Infinity]
- true = Infinity ; -1 = Infinity ; Infinity = repetition indefinie
- Defaut global: engine.defaults.loop

```js
import { engine } from 'animejs';
engine.defaults.loop = true;
```

```js
import { animate } from 'animejs';

animate('.loop', {
  x: '17.5rem',
  loop: 3,
});
```

```js
animate('.loop-alternate', {
  x: '17.5rem',
  loop: 3,
  alternate: true,
});
```

```js
animate('.loop-reverse', {
  x: '17.5rem',
  loop: 3,
  reversed: true,
});
```

```js
animate('.loop-infinity', {
  x: '17.5rem',
  loop: true, // Or Infinity
});
```

### animation/animation-playback-settings/playback-loopdelay

`https://animejs.com/documentation/animation/animation-playback-settings/playback-loopdelay`

> loopDelay: Number, defaut 0 ; delai en ms entre les boucles.

Le parametre loopDelay specifie la duree du delai en millisecondes qui se produit entre des boucles d'animation consecutives. Il definit le delai en millisecondes entre les boucles. Type Number, defaut 0, valeurs acceptees tout nombre >= 0. On peut modifier le defaut global via engine.defaults.loopDelay. Parametres lies : loop (active le cyclage continu) et alternate (inverse la direction sur les boucles successives).

**Faits clés**

- Parametre: loopDelay
- Type: Number
- Defaut: 0
- Valeurs acceptees: tout nombre >= 0
- Delai en ms entre les boucles consecutives
- Defaut global: engine.defaults.loopDelay
- Parametres lies: loop, alternate

```js
import { engine } from 'animejs';
engine.defaults.loopDelay = 500;
```

```js
import { animate } from 'animejs';

const loopDelayAnimation = animate('.circle', {
  x: '16rem',
  scale: {
    to: 1.8,
    delay: 500,
    duration: 500,
  },
  loopDelay: 1000,
  loop: true,
  alternate: true,
});
```

### animation/animation-playback-settings/alternate

`https://animejs.com/documentation/animation/animation-playback-settings/alternate`

> alternate: Boolean, defaut false ; inverse la direction de l'animation a chaque iteration quand loop est actif.

Le parametre alternate determine si la direction de l'animation s'inverse a chaque iteration quand loop est active (defini a true ou superieur a 1). Lorsqu'il est active, l'animation joue en avant, puis en arriere, puis en avant a nouveau pour chaque cycle de boucle. Type Boolean, defaut false. On peut modifier le defaut global via engine.defaults.alternate. Necessite que le parametre loop soit a true ou superieur a 1 pour observer le comportement d'alternance ; fonctionne en combinaison avec le parametre reversed pour controler la direction initiale.

**Faits clés**

- Parametre: alternate
- Type: Boolean
- Defaut: false
- Inverse la direction a chaque iteration (forward, backward, forward...)
- Necessite loop = true ou > 1 pour observer l'alternance
- Combinable avec reversed pour controler la direction initiale
- Defaut global: engine.defaults.alternate

```js
import { engine } from 'animejs';
engine.defaults.alternate = true;
```

```js
import { animate } from 'animejs';

animate('.dir-normal', {
  x: '17rem',
  alternate: false, // Default
  loop: 1,
});

animate('.dir-alternate', {
  x: '17rem',
  alternate: true,
  loop: 1, // Required to see the second iteration
});

animate('.dir-alternate-reverse', {
  x: '17rem',
  alternate: true,
  reversed: true,
  loop: 1,
});
```

### animation/animation-playback-settings/reversed

`https://animejs.com/documentation/animation/animation-playback-settings/reversed`

> Le parametre de playback `reversed` (Boolean, defaut false) determine si une animation progresse vers l'avant ou vers l'arriere depuis son debut.

`reversed` est un reglage de playback de type Boolean avec une valeur par defaut de `false`. Il etablit si une animation progresse vers l'avant ou vers l'arriere depuis son point de depart. Quand `true`, l'animation joue en sens inverse ; quand `false`, elle joue vers l'avant normalement. Le defaut peut etre modifie globalement via `engine.defaults.reversed`.

**Faits clés**

- Nom: reversed
- Type: Boolean
- Defaut: false
- true = lecture en sens inverse ; false = lecture vers l'avant (comportement par defaut)
- Modifiable globalement via engine.defaults.reversed = true
- Disponible depuis v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.reversed = true;
```

```js
import { animate } from 'animejs';

animate('.dir-normal', {
  x: '17rem',
  reversed: false, // Default behaviour
  loop: true
});

animate('.dir-reverse', {
  x: '17rem',
  reversed: true,
  loop: true
});
```

```js
<div class="medium row">
  <div class="circle dir-normal"></div>
  <div class="padded label">reversed: false</div>
</div>
<div class="medium row">
  <div class="circle dir-reverse"></div>
  <div class="padded label">reversed: true</div>
</div>
```

### animation/animation-playback-settings/autoplay

`https://animejs.com/documentation/animation/animation-playback-settings/autoplay`

> Le parametre `autoplay` (Boolean | onScroll(), defaut true) controle si une animation demarre automatiquement a sa creation.

`autoplay` est de type Boolean ou une configuration `onScroll()`, avec une valeur par defaut de `true`. Il controle si une animation commence a jouer automatiquement des sa creation. A `true`, la lecture demarre immediatement ; a `false`, un controle manuel de la lecture est requis via des methodes comme `play()`. On peut aussi passer une configuration `onScroll()` pour declencher la lecture quand les conditions de seuil de scroll sont satisfaites. Contrainte importante : ce parametre n'a aucun effet quand l'animation est ajoutee a une timeline et sera force a `false`. Le defaut peut etre modifie globalement via `engine.defaults.autoplay`.

**Faits clés**

- Nom: autoplay
- Type: Boolean | onScroll()
- Defaut: true
- false requiert un controle manuel via play()
- Accepte une config onScroll() pour declenchement au scroll
- Gotcha: aucun effet quand l'animation est dans une timeline -> force a false
- Modifiable globalement via engine.defaults.autoplay = false

```js
import { engine } from 'animejs';
engine.defaults.autoplay = false;
```

```js
animate('.autoplay-true', {
  x: '17rem',
  autoplay: true, // Default
});

animate('.autoplay-false', {
  x: '17rem',
  autoplay: false,
});
```

```js
<div class="medium row">
  <div class="circle autoplay-true"></div>
  <div class="padded label">autoplay: true</div>
</div>
<div class="medium row">
  <div class="circle autoplay-false"></div>
  <div class="padded label">autoplay: false</div>
</div>
```

### animation/animation-playback-settings/framerate

`https://animejs.com/documentation/animation/animation-playback-settings/framerate`

> Le parametre `frameRate` (Number, defaut 240) determine le nombre de frames par seconde (fps) auquel une animation est jouee.

`frameRate` est de type Number avec une valeur par defaut de `240` ; il accepte un nombre superieur a 0. Il determine le nombre de frames par seconde (fps) auquel une animation est jouee. Le frame rate est plafonne par le taux de rafraichissement du moniteur ou par les limitations du navigateur. Cette propriete peut etre ajustee dynamiquement apres la creation de l'animation en reassignant `animation.fps = value`. Le defaut peut etre modifie globalement via `engine.defaults.frameRate`.

**Faits clés**

- Nom: frameRate
- Type: Number
- Defaut: 240
- Accepte un nombre > 0
- Plafonne par le refresh rate du moniteur ou les limitations du navigateur
- Modifiable apres creation via animation.fps = value
- Modifiable globalement via engine.defaults.frameRate = 30

```js
import { engine } from 'animejs';
engine.defaults.frameRate = 30;
```

```js
import { animate } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $fps ] = utils.$('.fps');

const animation = animate('.circle', {
  x: '16rem',
  loop: true,
  alternate: true,
  frameRate: 60,
});

const updateFps = () => {
  const { value } = $range;
  $fps.innerHTML = value;
  animation.fps = value;
}

$range.addEventListener('input', updateFps);
```

### animation/animation-playback-settings/playbackrate

`https://animejs.com/documentation/animation/animation-playback-settings/playbackrate`

> Le parametre `playbackRate` (Number, defaut 1, >= 0) etablit un multiplicateur de vitesse pour accelerer ou ralentir une animation.

`playbackRate` est de type Number avec une valeur par defaut de `1` ; il doit etre >= 0. Il etablit un multiplicateur de vitesse pour accelerer ou ralentir une animation. A `0`, l'animation reste immobile. La valeur peut etre ajustee apres la creation via la propriete `.speed`. Le defaut peut etre modifie globalement via `engine.defaults.playbackRate`. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: playbackRate
- Type: Number
- Defaut: 1
- Contrainte: >= 0
- 0 = animation immobile
- Modifiable apres creation via animation.speed = value
- Modifiable globalement via engine.defaults.playbackRate = .75
- Disponible depuis v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.playbackRate = .75;
```

```js
import { animate, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $speed ] = utils.$('.speed');

const animation = animate('.circle', {
  x: '16rem',
  loop: true,
  alternate: true,
  playbackRate: 1,
});

const updateSpeed = () => {
  const { value } = $range;
  $speed.innerHTML = utils.roundPad(+value, 2);
  utils.sync(() => animation.speed = value);
}

$range.addEventListener('input', updateSpeed);
```

### animation/animation-playback-settings/playbackease

`https://animejs.com/documentation/animation/animation-playback-settings/playbackease`

> Le parametre `playbackEase` (ease function, defaut null) applique une fonction d'easing a l'ensemble de la lecture de l'animation.

`playbackEase` est de type ease function avec une valeur par defaut de `null`. Il applique une fonction d'easing globalement sur toute la timeline de lecture de l'animation, plutot qu'entre les keyframes individuels. Distinction cle avec le parametre tween `ease` : l'easing de tween s'applique entre chaque transition de keyframe de propriete, tandis que `playbackEase` controle la progression globale de l'animation du debut a la fin. Le defaut peut etre modifie globalement via `engine.defaults.playbackEase`.

**Faits clés**

- Nom: playbackEase
- Type: ease function
- Defaut: null
- Applique un easing a TOUTE la lecture de l'animation (pas entre keyframes individuels)
- Difference avec le tween ease : tween ease = entre chaque keyframe ; playbackEase = progression globale
- Modifiable globalement via engine.defaults.playbackEase = 'inOut'

```js
import { animate } from 'animejs';

animate('.square', {
  keyframes: [
    { y: '-2.5rem', duration: 400 },
    { x: '17rem', rotate: 180, scale: .5 },
    { y: '2.5rem' },
    { x: 0, rotate: 360, scale: 1 },
    { y: 0, duration: 400 }
  ],
  duration: 4000,
  playbackEase: 'inOut(3)', // this ease is applied accross all keyframes
  loop: true,
});
```

```js
import { engine } from 'animejs';
engine.defaults.playbackEase = 'inOut';
```

### animation/animation-playback-settings/persist

`https://animejs.com/documentation/animation/animation-playback-settings/persist`

> Le parametre `persist` (Boolean, defaut false) empeche l'annulation/liberation automatique des animations WAAPI une fois terminees.

`persist` est de type Boolean avec une valeur par defaut de `false`. Par defaut, les animations WAAPI sont automatiquement annulees et liberees de la memoire quand elles se terminent. `persist` empeche ce nettoyage automatique, gardant les animations terminees actives afin que des methodes puissent toujours etre appelees sur elles. Pour les animations WAAPI controlees par scroll, `persist` est automatiquement active. Le defaut peut etre modifie globalement via `engine.defaults.persist`.

**Faits clés**

- Nom: persist
- Type: Boolean
- Defaut: false
- Specifique aux animations WAAPI (waapi.animate)
- Par defaut les animations WAAPI sont annulees et liberees de la memoire a la fin
- persist: true empeche ce nettoyage pour pouvoir appeler des methodes apres completion
- Automatiquement active pour les animations WAAPI controlees par scroll
- Modifiable globalement via engine.defaults.persist = true

```js
import { engine } from 'animejs';
engine.defaults.persist = true;
```

```js
import { waapi, utils } from 'animejs';

const [ $button ] = utils.$('.button');

const animationA = waapi.animate('.square-a', {
  x: '17rem',
  persist: false, // default
});

const animationB = waapi.animate('.square-b', {
  x: '17rem',
  persist: true,
});

const alaternateAnimations = () => {
  animationA.alternate().resume();
  animationB.alternate().resume();
};

$button.addEventListener('click', alaternateAnimations);
```

### animation/animation-callbacks

`https://animejs.com/documentation/animation/animation-callbacks`

> Page d'introduction listant les callbacks d'animation, executes a des points specifiques de la lecture et specifies directement dans les parametres de animate().

Les callbacks d'animation permettent d'executer des fonctions a des points specifiques de la lecture de l'animation. Les callbacks de type Function sont specifies directement dans l'objet de parametres de `animate()`. Les callbacks disponibles sont : onBegin, onComplete, onBeforeUpdate, onUpdate, onRender, onLoop, onPause et then(). Chaque callback est documente individuellement dans ses sous-sections. Fonctionnalite disponible depuis la version 1.0.0, faisant partie de la section Animation.

**Faits clés**

- Callbacks listes: onBegin, onComplete, onBeforeUpdate, onUpdate, onRender, onLoop, onPause, then()
- Type des callbacks: Function
- Specifies directement dans l'objet de parametres de animate()
- Disponible depuis v1.0.0

```js
animate('.square', {
  translateX: 100,
  scale: 2,
  opacity: .5,
  duration: 400,
  delay: 250,
  ease: 'out(3)',
  loop: 3,
  alternate: true,
  autoplay: false,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### animation/animation-callbacks/onbegin

`https://animejs.com/documentation/animation/animation-callbacks/onbegin`

> Le callback `onBegin` (Function, defaut noop) s'execute quand une animation commence sa lecture, recevant l'instance d'animation en premier argument.

`onBegin` est de type Function avec une valeur par defaut de `noop`. Le callback s'execute quand une animation commence sa lecture, apres toute periode de `delay`. Il recoit l'instance d'animation (`self`) comme premier parametre, donnant acces a l'etat et aux methodes de l'animation, par exemple la propriete `began`. Le defaut peut etre modifie globalement via `engine.defaults.onBegin`. Disponible depuis v4.0.0.

**Faits clés**

- Nom: onBegin
- Type: Function
- Defaut: noop
- S'execute quand la lecture commence, APRES toute periode de delay
- Recoit l'instance d'animation (self) en premier argument (ex: self.began)
- Modifiable globalement via engine.defaults.onBegin
- Disponible depuis v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onBegin = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const animation = animate('.circle', {
  x: '16rem',
  delay: 1000, // Delays the onBegin() callback by 1000ms
  onBegin: self => $value.textContent = self.began
});
```

```js
<div class="large row">
  <div class="circle"></div>
  <pre class="large log row">
    <span class="label">began</span>
    <span class="value">false</span>
  </pre>
</div>
```

### animation/animation-callbacks/oncomplete

`https://animejs.com/documentation/animation/animation-callbacks/oncomplete`

> Le callback `onComplete` (Function, defaut noop) s'execute quand toutes les iterations (loops) d'une animation sont terminees.

`onComplete` est de type Function avec une valeur par defaut de `noop`. Il execute une fonction quand toutes les iterations (loops) d'une animation ont fini de jouer — uniquement apres que TOUS les loops soient termines, pas apres chaque loop individuel (pour cela utiliser `onLoop`). Le callback recoit l'instance d'animation comme unique parametre (ex: `self.completed`). Le defaut peut etre modifie globalement via `engine.defaults.onComplete`. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: onComplete
- Type: Function
- Defaut: noop
- Se declenche seulement apres que TOUS les loops soient finis (pas apres chaque loop -> utiliser onLoop)
- Recoit l'instance d'animation comme unique parametre (ex: self.completed)
- Modifiable globalement via engine.defaults.onComplete
- Disponible depuis v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onComplete = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const animation = animate('.circle', {
  x: '16rem',
  delay: 500,
  loop: 2,
  alternate: true,
  onComplete: self => $value.textContent = self.completed
});
```

```js
<div class="large row">
  <div class="circle"></div>
  <pre class="large log row">
    <span class="label">completed</span>
    <span class="value">false</span>
  </pre>
</div>
```

### animation/animation-callbacks/onbeforeupdate

`https://animejs.com/documentation/animation/animation-callbacks/onbeforeupdate`

> Le callback `onBeforeUpdate` (Function, defaut noop) s'execute avant la mise a jour des valeurs de tween, a chaque frame, au frameRate specifie.

`onBeforeUpdate` est de type Function avec une valeur par defaut de `noop`. Il s'execute avant la mise a jour des valeurs de tween a chaque frame d'une animation en cours, au `frameRate` specifie. Il recoit l'instance d'animation comme premier argument. Cela permet d'executer une logique custom qui affecte le calcul des valeurs — utile pour modifier dynamiquement le comportement de l'animation selon sa progression (ex: ajuster un multiplicateur juste avant la mise a jour des tweens via `self.iterationProgress`). Le defaut peut etre modifie globalement via `engine.defaults.onBeforeUpdate`. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: onBeforeUpdate
- Type: Function
- Defaut: noop
- S'execute AVANT la mise a jour des valeurs de tween, a chaque frame, au frameRate specifie
- Recoit l'instance d'animation en premier argument (ex: self.iterationProgress)
- Utile pour modifier le calcul des valeurs avant l'update des tweens
- Modifiable globalement via engine.defaults.onBeforeUpdate
- Disponible depuis v4.0.0

```js
import { animate, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let mult = 1;
let updates = 0;

const animation = animate('.circle', {
  x: '16rem',
  loopDelay: 1500,
  modifier: v => mult * v,
  loop: true,
  alternate: true,
  onBeforeUpdate: self => {
    $value.textContent = ++updates;
    // Update the mult value just before updating the tweens
    mult = 1 - self.iterationProgress;
  }
});
```

```js
import { engine } from 'animejs';
engine.defaults.onBeforeUpdate = self => console.log(self.id);
```

### animation/animation-callbacks/onupdate

`https://animejs.com/documentation/animation/animation-callbacks/onupdate`

> Le callback `onUpdate` (Function, defaut noop) s'execute a chaque frame d'une animation en cours, a la frequence du frameRate.

`onUpdate` est de type Function avec une valeur par defaut de `noop`, de signature `onUpdate: (self) => void`. Il s'execute a chaque frame d'une animation en cours, declenche a la frequence specifiee par le reglage `frameRate` de l'animation. Le callback recoit l'instance d'animation comme premier argument. Dans la sequence des callbacks, il est precede de `onBeforeUpdate` et suivi de `onRender`. Le defaut peut etre modifie globalement via `engine.defaults.onUpdate`.

**Faits clés**

- Nom: onUpdate
- Type: Function
- Defaut: noop
- Signature: onUpdate: (self) => void
- S'execute a chaque frame, a la frequence du frameRate
- Recoit l'instance d'animation en premier argument
- Sequence des callbacks: precede par onBeforeUpdate, suivi par onRender
- Modifiable globalement via engine.defaults.onUpdate

```js
onUpdate: (self) => void
```

```js
import { engine } from 'animejs';
engine.defaults.onUpdate = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let updates = 0;

const animation = animate('.circle', {
  x: '16rem',
  loopDelay: 1500,
  loop: true,
  alternate: true,
  onUpdate: self => $value.textContent = ++updates
});
```

```js
<div class="large row">
  <div class="circle"></div>
  <pre class="large log row">
    <span class="label">updates</span>
    <span class="value">0</span>
  </pre>
</div>
```

### animation/animation-callbacks/onrender

`https://animejs.com/documentation/animation/animation-callbacks/onrender`

> Le callback `onRender` (Function, defaut noop) s'execute chaque fois qu'une animation effectue un rendu a l'ecran ; il ne se declenche pas pendant les periodes delay/loopDelay.

`onRender` est de type Function avec une valeur par defaut de `noop`, de signature `onRender: (self) => void`. Il s'execute chaque fois qu'une animation effectue un rendu a l'ecran. Il ne se declenche PAS pendant les periodes de `delay` ou `loopDelay` ou aucun rendu n'a lieu. Le callback recoit l'instance d'animation comme unique parametre, permettant d'acceder aux proprietes et methodes de l'animation durant le cycle de rendu. Le defaut peut etre modifie globalement via `engine.defaults.onRender`.

**Faits clés**

- Nom: onRender
- Type: Function
- Defaut: noop
- Signature: onRender: (self) => void
- S'execute a chaque rendu de l'animation a l'ecran
- Gotcha: ne se declenche PAS pendant les periodes delay ou loopDelay (pas de rendu)
- Recoit l'instance d'animation comme unique parametre
- Modifiable globalement via engine.defaults.onRender

```js
onRender: (self) => void
```

```js
import { engine } from 'animejs';
engine.defaults.onRender = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $rendersLog ] = utils.$('.value');

let renders = 0;

const animation = animate('.circle', {
  x: '16rem',
  loopDelay: 1500,
  loop: true,
  alternate: true,
  onRender: self => $rendersLog.textContent = ++renders
});
```

### animation/animation-callbacks/onloop

`https://animejs.com/documentation/animation/animation-callbacks/onloop`

> Le callback `onLoop` (Function, defaut noop) s'execute chaque fois qu'une iteration (loop) d'animation se termine.

`onLoop` est de type Function avec une valeur par defaut de `noop`. Il execute une fonction chaque fois qu'une iteration (loop) d'animation se termine. Le callback recoit l'instance d'animation comme premier argument. Le defaut peut etre modifie globalement via `engine.defaults.onLoop`. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: onLoop
- Type: Function
- Defaut: noop
- S'execute chaque fois qu'une iteration (loop) se termine (a la difference de onComplete qui attend tous les loops)
- Recoit l'instance d'animation en premier argument
- Modifiable globalement via engine.defaults.onLoop
- Disponible depuis v4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onLoop = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let loops = 0;

const animation = animate('.circle', {
  x: '16rem',
  loopDelay: 1500,
  loop: true,
  alternate: true,
  onLoop: self => $value.textContent = ++loops
});
```

```js
<div class="large row">
  <div class="circle"></div>
  <pre class="large log row">
    <span class="label">loops</span>
    <span class="value">0</span>
  </pre>
</div>
```

### animation/animation-methods/seek

`https://animejs.com/documentation/animation/animation-methods/seek`

> La methode seek() met a jour le currentTime d'une animation et l'avance a une position temporelle precise.

seek() met a jour le currentTime de l'animation et l'avance a une position temporelle specifique. Elle accepte un parametre time (Number) = le nouveau currentTime en ms de l'animation, et un parametre optionnel muteCallbacks (Boolean, defaut false) qui, si true, empeche le declenchement des callbacks. La methode retourne l'animation elle-meme, permettant le chainage avec d'autres methodes d'animation.

**Faits clés**

- Signature: animation.seek(time, muteCallbacks);
- time: Number — nouveau currentTime en ms de l'animation
- muteCallbacks: Boolean optionnel, defaut false — si true empeche les callbacks de se declencher
- Retourne l'animation elle-meme (chainable)

```js
animation.seek(time, muteCallbacks);
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $playPauseButton ] = utils.$('.play-pause');

const updateButtonLabel = animation => {
  $playPauseButton.textContent = animation.paused ? 'Play' : 'Pause';
}

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  duration: 1750,
  delay: stagger(250),
  autoplay: false,
  onUpdate: self => {
    $range.value = self.currentTime;
    updateButtonLabel(self);
  },
  onComplete: updateButtonLabel,
});

const seekAnimation = () => animation.seek(+$range.value);

const playPauseAnimation = () => {
  if (animation.paused) {
    animation.play();
  } else {
    animation.pause();
    updateButtonLabel(animation);
  }
}

$range.addEventListener('input', seekAnimation);
$playPauseButton.addEventListener('click', playPauseAnimation);
```

### animation/animation-methods/stretch

`https://animejs.com/documentation/animation/animation-methods/stretch`

> La methode stretch() ajuste la duree totale d'une animation a une valeur cible en recalculant proportionnellement durees et tweens.

stretch() ajuste la duree globale d'une animation pour correspondre a une duree cible specifiee. Elle recalcule a la fois la duree de l'animation et celle de ses tweens proportionnellement. La duree totale equivaut a la duree d'iteration multipliee par le nombre d'iterations — ainsi une animation de 1000ms qui boucle deux fois a une duree totale de 3000ms (3 iterations x 1000ms). Le parametre duration (Number) est la nouvelle duree totale en millisecondes. La methode retourne l'animation elle-meme, permettant le chainage.

**Faits clés**

- Signature: animation.stretch(duration);
- duration: Number — nouvelle duree totale en millisecondes de l'animation
- Recalcule duree de l'animation ET durees des tweens proportionnellement
- Duree totale = duree d'iteration x nombre d'iterations (1000ms x 2 loops = 3000ms total, soit 3 iterations)
- Gotcha: regler duration a 0 rend tous les tweens de meme longueur, affectant les appels stretch() ulterieurs
- Retourne l'animation elle-meme (chainable)

```js
animation.stretch(duration);
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $totalDuration ] = utils.$('.value');

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  delay: stagger(200),
});

const stretchAnimation = () => {
  const newDuration = +$range.value;
  $totalDuration.textContent = newDuration;
  animation.stretch(newDuration).restart();
}

stretchAnimation();
$range.addEventListener('input', stretchAnimation);
```

### animation/animation-methods/refresh

`https://animejs.com/documentation/animation/animation-methods/refresh`

> La methode refresh() recalcule les valeurs des proprietes animees definies par fonction en mettant a jour les valeurs from et to.

refresh() re-calcule les valeurs des proprietes animees definies avec une valeur basee sur une fonction (Function based value) en mettant a jour les valeurs 'from' vers les valeurs cibles actuelles, et les valeurs 'to' vers les valeurs nouvellement calculees. Seules les valeurs de proprietes animables sont recalculees ; la duration et le delay ne peuvent pas etre rafraichis. La methode ne prend aucun parametre et retourne l'instance d'animation, permettant le chainage. Particulierement utile lors de la combinaison de valeurs basees sur fonction avec des boucles, ou lorsqu'on doit recalculer dynamiquement les cibles des proprietes. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: refresh() — aucun parametre
- Met a jour les valeurs from vers les valeurs cibles actuelles et les valeurs to vers les valeurs nouvellement calculees
- Seules les valeurs de proprietes animables sont recalculees ; duration et delay ne peuvent pas etre rafraichis
- Disponible depuis la version 4.0.0
- Retourne l'instance d'animation (chainable)

```js
refresh()
```

```js
import { animate } from 'animejs';

const [ $refreshButton ] = utils.$('.refresh');

const animation = animate('.square', {
  x: () => utils.random(0, 17) + 'rem',
  y: () => utils.random(-1, 1) + 'rem',
  rotate: () => utils.random(-360, 360, 1),
  scale: () => utils.random(.1, 1.5, 2),
  duration: 750,
  loop: true,
  onLoop: self => self.refresh()
});

const refreshAnimation = () => animation.refresh().restart();

$refreshButton.addEventListener('click', refreshAnimation);
```

### animation/animation-properties

`https://animejs.com/documentation/animation/animation-properties`

> Liste des proprietes accessibles sur les instances Animation retournees par animate() et waapi.animate().

Les instances Animation exposent un ensemble de proprietes en lecture/ecriture. Certaines proprietes sont exclusives a la version JavaScript d'animate() (marquees 'JS only') et ne sont pas disponibles dans les variantes WAAPI. id (String | Number, JS only): gets and sets the ID of the animation. targets (Array): gets the current animation targets. currentTime (Number): gets and sets the global current time in ms of the animation. iterationCurrentTime (Number, JS only): gets and sets the current iteration time in ms. deltaTime (Number, JS only): gets the time in ms elapsed between the current and previous frame. progress (Number): gets and sets the overall progress of the animation from 0 to 1. iterationProgress (Number, JS only): gets and sets the progress of the current iteration from 0 to 1. currentIteration (Number, JS only): gets and sets the current iteration count. duration (Number): gets the total duration in ms of the animation. speed (Number): gets and sets the speed multiplier of the animation. fps (Number, JS only): gets and sets the fps of the animation. paused (Boolean): gets and sets whether the animation is paused. began (Boolean, JS only): gets and sets whether the animation has started. completed (Boolean): gets and sets whether the animation has completed. reversed (Boolean, JS only): gets and sets whether the animation is reversed. backwards (Boolean, JS only): gets whether the animation is currently playing backwards.

**Faits clés**

- id: String | Number (JS only) — gets and sets the ID of the animation
- targets: Array — gets the current animation targets
- currentTime: Number — gets and sets the global current time in ms
- iterationCurrentTime: Number (JS only) — gets and sets the current iteration time in ms
- deltaTime: Number (JS only) — gets the time in ms elapsed between current and previous frame
- progress: Number — gets and sets the overall progress from 0 to 1
- iterationProgress: Number (JS only) — gets and sets the progress of the current iteration from 0 to 1
- currentIteration: Number (JS only) — gets and sets the current iteration count
- duration: Number — gets the total duration in ms
- speed: Number — gets and sets the speed multiplier
- fps: Number (JS only) — gets and sets the fps
- paused: Boolean — gets and sets whether the animation is paused
- began: Boolean (JS only) — gets and sets whether the animation has started
- completed: Boolean — gets and sets whether the animation has completed
- reversed: Boolean (JS only) — gets and sets whether the animation is reversed
- backwards: Boolean (JS only) — gets whether the animation is currently playing backwards
- Les proprietes marquees 'JS only' ne sont pas disponibles dans les variantes WAAPI


## timeline

### timeline

`https://animejs.com/documentation/timeline`

> createTimeline() cree une instance Timeline pour composer et coordonner des animations, timers et autres timelines.

createTimeline(parameters) cree une instance Timeline avec des methodes pour composer des animations coordonnees. Le parametre parameters (Object, optionnel) accepte les Timeline playback settings et les Timeline callbacks. L'import peut se faire via 'animejs' ou 'animejs/timeline'. La timeline retournee expose les methodes principales: add (pour ajouter un timer ou une animation avec un target/parametres/position), sync (pour synchroniser une timeline/animation/timer existant), call (pour executer un callback a une position), label (pour definir un label de position). On peut utiliser le labeling, l'ajout de plusieurs animations a differentes positions temporelles, et la syntaxe de timing relatif (ex: '<-=500').

**Faits clés**

- Signature: createTimeline(parameters)
- parameters: Object optionnel — accepte Timeline playback settings et Timeline callbacks
- Import: import { createTimeline } from 'animejs'; ou from 'animejs/timeline';
- Retourne une instance Timeline
- Methodes principales: add(), sync(), call(), label()

```js
import { createTimeline } from 'animejs';
// or
import { createTimeline } from 'animejs/timeline';
```

```js
timeline.add(target, animationParameters, position);
timeline.add(timerParameters, position);
timeline.sync(timelineB, position);
timeline.call(callbackFunction, position);
timeline.label(labelName, position);
```

```js
import { createTimeline } from 'animejs';

const tl = createTimeline({ defaults: { duration: 750 } });

tl.label('start')
  .add('.square', { x: '15rem' }, 500)
  .add('.circle', { x: '15rem' }, 'start')
  .add('.triangle', { x: '15rem', rotate: '1turn' }, '<-=500');
```

### timeline/add-timers

`https://animejs.com/documentation/timeline/add-timers`

> Les timers peuvent etre integres a une timeline via add() (creation) ou sync() (synchronisation d'un timer existant).

Les timers peuvent etre integres a une timeline via la methode add() (qui cree de nouveaux timers) ou la methode sync() (qui synchronise des timers existants). Pour la creation: timeline.add(parameters, position) ou parameters (Object) = timer playback settings et callbacks, et position (optionnel) = time position pour le placement dans la timeline. Pour la synchronisation: timeline.sync(timer, position) ou timer = une instance de Timer existante, et position (optionnel) = time position. Les deux retournent la timeline elle-meme (chainable).

**Faits clés**

- Creation timer: timeline.add(parameters, position); — parameters: Object (timer playback settings + callbacks), position optionnel: time position
- Synchronisation timer: timeline.sync(timer, position); — timer: instance Timer existante, position optionnel: time position
- Les deux methodes retournent la timeline elle-meme (chainable)

```js
timeline.add(parameters, position);
```

```js
timeline.sync(timer, position);
```

```js
import { createTimeline, createTimer, utils } from 'animejs';

const [ $timer01, $timer02, $timer03 ] = utils.$('.timer');

const timer1 = createTimer({
  duration: 1500,
  onUpdate: self => $timer01.innerHTML = self.currentTime,
});

const tl = createTimeline()
.sync(timer1)
.add({
  duration: 500,
  onUpdate: self => $timer02.innerHTML = self.currentTime,
})
.add({
  onUpdate: self => $timer03.innerHTML = self.currentTime,
  duration: 1000
});
```

### timeline/add-animations

`https://animejs.com/documentation/timeline/add-animations`

> add() cree et ajoute une animation a une timeline (composition de tweens avec les enfants existants) ; sync() integre une animation existante sans affecter les enfants.

La methode add() cree et ajoute une animation directement a une timeline, permettant la composition des valeurs de tweens avec les enfants existants de la timeline. Signature: timeline.add(targets, parameters, position) ou targets = elements DOM, selecteurs CSS, objets JS ou tableaux de ceux-ci ; parameters (Object) = proprietes animables, parametres de tween, playback settings et callbacks ; position (optionnel) = time position. La methode sync() incorpore une animation existante dans une timeline. Contrairement a add(), la composition des tweens se fait au moment de la creation de l'animation, laissant les enfants de la timeline inchanges. Signature: timeline.sync(animation, position) ou animation = une instance d'Animation pre-creee, et position (optionnel) = time position. sync() retourne la timeline elle-meme (chainable).

**Faits clés**

- add(): timeline.add(targets, parameters, position); — composition des tweens avec les enfants existants de la timeline
- add() targets: DOM elements, CSS selectors, JS objects ou tableaux
- add() parameters: Object — proprietes animables, parametres tween, playback settings, callbacks
- sync(): timeline.sync(animation, position); — composition des tweens a la creation de l'animation, enfants timeline inchanges
- sync() animation: instance Animation pre-creee
- position optionnel pour les deux: time position
- sync() retourne la timeline (chainable)

```js
timeline.add(targets, parameters, position);
```

```js
timeline.sync(animation, position);
```

```js
import { createTimeline, animate } from 'animejs';

const circleAnimation = animate('.circle', {
  x: '15rem'
});

const tl = createTimeline()
.sync(circleAnimation)
.add('.triangle', {
  x: '15rem',
  rotate: '1turn',
  duration: 500,
  alternate: true,
  loop: 2,
})
.add('.square', {
  x: '15rem',
});
```

### timeline/sync-waapi-animations

`https://animejs.com/documentation/timeline/sync-waapi-animations`

> sync() permet de coordonner des animations Web Animation API (WAAPI) au sein d'une timeline.

Cette fonctionnalite permet aux animations Web Animation API (WAAPI) d'etre coordonnees au sein d'une structure de timeline. En synchronisant des animations WAAPI a differentes positions, on peut orchestrer des sequences multi-elements complexes avec un controle de timing precis. Signature: timeline.sync(animation, position). Le parametre 'synced' accepte une Animation, un Timer ou une Timeline = l'animation, timer ou timeline a synchroniser. position (optionnel) = ou placer l'element synchronise dans la timeline (time position). La methode retourne la timeline elle-meme, permettant le chainage.

**Faits clés**

- Signature: timeline.sync(animation, position);
- synced: Animation | Timer | Timeline — l'element a synchroniser
- position optionnel: time position — ou placer l'element synchronise
- Retourne la timeline elle-meme (chainable)
- Import via waapi: import { createTimeline, waapi } from 'animejs';

```js
timeline.sync(animation, position);
```

```js
import { createTimeline, waapi } from 'animejs';

const circle = waapi.animate('.circle', {
  x: '15rem',
});

const triangle = waapi.animate('.triangle', {
  x: '15rem',
  y: [0, '-1.5rem', 0],
  ease: 'out(4)',
  duration: 750,
});

const square = waapi.animate('.square', {
  x: '15rem',
  rotateZ: 360,
});

const tl = createTimeline()
  .sync(circle, 0)
  .sync(triangle, 350)
  .sync(square, 250);
```

### timeline/sync-timelines

`https://animejs.com/documentation/timeline/sync-timelines`

> sync() permet de synchroniser une timeline avec d'autres timelines, animations ou timers pour une lecture coordonnee.

La methode sync() permet de synchroniser des timelines avec d'autres timelines, animations ou timers, permettant une lecture coordonnee a travers plusieurs instances de timeline. Signature: timelineA.sync(timelineB, position). Le parametre 'synced' accepte une Animation, un Timer ou une Timeline = la timeline, animation ou timer a synchroniser avec la timeline courante. position (optionnel) = time position specifiant ou dans la timeline courante l'element synchronise doit etre positionne. La methode retourne la timeline elle-meme, supportant le chainage. L'exemple montre la synchronisation d'une animation autonome dans la timeline A, puis la synchronisation de deux timelines distinctes (A et B) dans une timeline principale, avec B decalee de 2000 millisecondes ('-=2000').

**Faits clés**

- Signature: timelineA.sync(timelineB, position);
- synced: Animation | Timer | Timeline — element a synchroniser avec la timeline courante
- position optionnel: time position — ou positionner l'element synchronise dans la timeline courante
- Retourne la timeline elle-meme (chainable)

```js
timelineA.sync(timelineB, position);
```

```js
import { createTimeline, animate } from 'animejs';

const circleAnimation = animate('.circle', {
  x: '15rem'
});

const tlA = createTimeline()
.sync(circleAnimation)
.add('.triangle', {
  x: '15rem',
  duration: 2000,
})
.add('.square', {
  x: '15rem',
});

const tlB = createTimeline({ defaults: { duration: 2000 } })
.add(['.triangle', '.square'], {
  rotate: 360,
}, 0)
.add('.circle', {
  scale: [1, 1.5, 1],
}, 0);

const tlMain = createTimeline()
.sync(tlA)
.sync(tlB, '-=2000');
```

### timeline/call-functions

`https://animejs.com/documentation/timeline/call-functions`

> La methode call() execute une fonction arbitraire a une position temporelle precise dans une timeline.

La methode call() permet d'executer des fonctions arbitraires a des moments specifiques au sein d'une timeline. Utile pour declencher des effets de bord, mettre a jour des elements DOM, ou coordonner des evenements non-animation avec la sequence d'animation. Signature: timeline.call(callback, position). callback (Function) = la fonction a executer a la position temporelle specifiee. position (optionnel) = time position indiquant quand la fonction doit etre appelee dans la timeline. La methode retourne la timeline elle-meme, permettant le chainage. L'exemple execute trois fonctions distinctes a 0ms, 800ms et 1200ms respectivement.

**Faits clés**

- Signature: timeline.call(callback, position)
- callback: Function — la fonction a executer a la position specifiee
- position optionnel: time position — quand la fonction doit etre appelee
- Retourne la timeline elle-meme (chainable)

```js
timeline.call(callback, position)
```

```js
import { createTimeline, utils } from 'animejs';

const [ $functionA ] = utils.$('.function-A');
const [ $functionB ] = utils.$('.function-B');
const [ $functionC ] = utils.$('.function-C');

const tl = createTimeline()
.call(() => $functionA.innerHTML = 'A', 0)
.call(() => $functionB.innerHTML = 'B', 800)
.call(() => $functionC.innerHTML = 'C', 1200);
```

### timeline/time-position

`https://animejs.com/documentation/timeline/time-position`

> Le parametre position specifie quand un enfant de timeline est insere ; il accepte plusieurs formats (absolu, relatif, label, stagger).

La time position specifie quand un enfant de timeline est insere. Si undefined, l'enfant est positionne a la fin de la timeline. Le parametre position est utilise dans add(), call(), sync() et label(), et accepte divers formats pour controler la flexibilite du timing. Types: Absolute (ex: 100) = placement exact a 100ms ; Addition ('+=100') = 100ms apres le dernier element ; Subtraction ('-=100') = 100ms avant la fin du dernier element ; Multiplier ('*=.5') = la moitie de la duree totale de l'element ; Previous end ('<') = a la fin de l'element precedent ; Previous start ('<<') = au debut de l'element precedent ; Combined ('<<+=250') = 250ms apres le debut de l'element precedent ; Label ('My Label') = a la position du label specifie ; Stagger (stagger(10)) = positionnement decale de 10ms.

**Faits clés**

- Si position undefined, l'enfant se place a la fin de la timeline
- Absolute: 100 — placement exact a 100ms
- Addition: '+=100' — 100ms apres le dernier element
- Subtraction: '-=100' — 100ms avant la fin du dernier element
- Multiplier: '*=.5' — la moitie de la duree totale de l'element
- Previous end: '<' — a la fin de l'element precedent
- Previous start: '<<' — au debut de l'element precedent
- Combined: '<<+=250' — 250ms apres le debut de l'element precedent
- Label: 'My Label' — a la position du label
- Stagger: stagger(10) — positionnement decale de 10ms

```js
timeline.add(target, animationParameters, position);
timeline.add(timerParameters, position);
timeline.call(callbackFunction, position);
timeline.sync(labelName, position);
timeline.label(labelName, position);
```

```js
import { createTimeline } from 'animejs';

const tl = createTimeline()
.label('start', 0)
.add('.square', {
  x: '15rem',
  duration: 500,
}, 500)
.add('.circle', {
  x: '15rem',
  duration: 500,
}, 'start')
.add('.triangle', {
  x: '15rem',
  rotate: '1turn',
  duration: 500,
}, '<-=250');
```

### timeline/timeline-playback-settings

`https://animejs.com/documentation/timeline/timeline-playback-settings`

> Les playback settings configurent le timing et le comportement d'une timeline, passes dans l'objet parameters de createTimeline().

Les Timeline playback settings configurent les proprietes de timing et de comportement d'une timeline creee avec createTimeline(). Les reglages sont passes directement dans l'objet parameters de createTimeline(). Reglages disponibles: defaults (proprietes d'animation par defaut appliquees a toutes les animations contenues), delay (temps d'attente initial avant le demarrage de la timeline), loop (nombre de repetitions), loopDelay (duree de pause entre les cycles de boucle), alternate (inverse le sens aux iterations alternees), reversed (demarre la lecture en sens inverse), autoplay (demarre automatiquement la lecture a la creation), frameRate (controle la frequence de rendu), playbackRate (ajuste la vitesse de lecture), playbackEase (fonction d'easing appliquee a la progression de la timeline). Chaque reglage dispose d'une page de documentation dediee.

**Faits clés**

- Reglages passes dans l'objet parameters de createTimeline()
- defaults — proprietes d'animation par defaut appliquees aux animations contenues
- delay — temps d'attente initial avant le demarrage
- loop — nombre de repetitions
- loopDelay — pause entre cycles de boucle
- alternate — inverse le sens aux iterations alternees
- reversed — demarre la lecture en sens inverse
- autoplay — demarre automatiquement la lecture a la creation
- frameRate — controle la frequence de rendu
- playbackRate — ajuste la vitesse de lecture
- playbackEase — easing applique a la progression de la timeline

```js
createTimeline({
  defaults: {
    ease: 'out(3)',
    duration: 500,
  },
  loop: 3,
  alternate: true,
  autoplay: false,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### timeline/timeline-playback-settings/defaults

`https://animejs.com/documentation/timeline/timeline-playback-settings/defaults`

> Le parametre defaults definit des reglages partages appliques a toutes les animations enfants d'une timeline.

Le parametre defaults (type Object, valeur par defaut undefined) etablit des reglages de configuration partages appliques a toutes les animations enfants d'une timeline. Il accepte les tween parameters (a l'exception de from et to), les playback settings et les callbacks. Chaque animation enfant herite des reglages par defaut sauf surcharge explicite. Le parametre from ne peut pas etre utilise dans defaults ; seules les cibles d'animation peuvent specifier from individuellement. Disponible depuis la version 2.0.0.

**Faits clés**

- Nom du parametre: defaults
- Type: Object
- Valeur par defaut: undefined
- Accepte: tween parameters (sauf from et to), playback settings, callbacks
- Chaque animation enfant herite des defaults sauf surcharge explicite
- Gotcha: from ne peut pas etre utilise dans defaults ; seules les cibles peuvent specifier from individuellement
- Disponible depuis la version 2.0.0

```js
import { createTimeline } from 'animejs';

const tl = createTimeline({
  defaults: {
    ease: 'inOutExpo',
    duration: 500,
    loop: 2,
    reversed: true,
    alternate: true,
  }
})
.add('.square', { x: '15rem' })
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' });
```

### timeline/timeline-playback-settings/delay

`https://animejs.com/documentation/timeline/timeline-playback-settings/delay`

> Le parametre delay definit le nombre de millisecondes avant que la timeline commence son execution.

delay (Number, defaut 0) definit le nombre de millisecondes avant le demarrage de l'execution de la timeline. Il accepte tout nombre superieur ou egal a 0. La valeur par defaut peut etre modifiee globalement via engine.defaults.delay. Disponible depuis la version 2.0.0.

**Faits clés**

- Nom: delay
- Type: Number
- Defaut: 0
- Accepte tout nombre >= 0
- Defaut global modifiable via engine.defaults.delay
- Disponible depuis la version 2.0.0

```js
import { createTimeline, createTimer, utils } from 'animejs';

const tl = createTimeline({
  delay: 2000,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');

const [ $time ] = utils.$('.time');

createTimer({
  duration: 2000 + tl.duration,
  onUpdate: self => $time.innerHTML = (2000 - self.currentTime) * -1,
});
```

```js
import { engine } from 'animejs';
engine.defaults.delay = 500;
```

### timeline/timeline-playback-settings/loop

`https://animejs.com/documentation/timeline/timeline-playback-settings/loop`

> Le parametre loop definit combien de fois une timeline se repete.

loop (defaut 0) definit le nombre de repetitions d'une timeline. Il accepte des valeurs numeriques entre 0 et Infinity. Valeurs acceptees : Number (compteur de repetitions dans l'intervalle [0, Infinity]), Infinity (repete indefiniment), true (equivaut a Infinity), -1 (equivaut a Infinity). Le defaut peut etre modifie globalement via engine.defaults.loop. Disponible depuis la v2.0.0.

**Faits clés**

- Nom: loop
- Type: Number | Infinity | true | -1
- Defaut: 0
- Number = compteur dans [0, Infinity]
- Infinity / true / -1 = repete indefiniment
- Defaut global via engine.defaults.loop
- Disponible depuis la v2.0.0

```js
import { engine } from 'animejs';
engine.defaults.loop = true;
```

```js
import { createTimeline, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');

let loops = 0;

const tl = createTimeline({
  loop: true,
  onLoop: self => $loops.innerHTML = ++loops,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');
```

### timeline/timeline-playback-settings/playback-loopdelay

`https://animejs.com/documentation/timeline/timeline-playback-settings/playback-loopdelay`

> loopDelay specifie le delai (ms) entre les iterations successives de la boucle d'une timeline.

loopDelay (Number, defaut 0, valeur >= 0) specifie la duree du delai (en millisecondes) qui se produit entre les iterations successives de la boucle d'une animation timeline. Le defaut peut etre modifie globalement via engine.defaults.loopDelay.

**Faits clés**

- Nom: loopDelay
- Type: Number
- Defaut: 0
- Valeur valide: >= 0
- Delai en ms entre iterations de boucle
- Defaut global via engine.defaults.loopDelay

```js
import { engine } from 'animejs';
engine.defaults.loopDelay = 500;
```

```js
import { createTimeline, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');

const tl = createTimeline({
  loopDelay: 500,
  loop: true,
  onLoop: self => $loops.innerHTML = self._currentIteration,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
  <pre class="large log row">
    <span class="label">loops</span>
    <span class="loops value">0</span>
  </pre>
</div>
```

### timeline/timeline-playback-settings/alternate

`https://animejs.com/documentation/timeline/timeline-playback-settings/alternate`

> alternate inverse le sens de lecture de la timeline a chaque iteration de boucle (effet ping-pong).

alternate (Boolean, defaut false) determine si le sens de lecture d'une timeline s'inverse a chaque iteration de boucle. Cela ne prend effet que lorsque loop vaut true ou une valeur superieure a 1. Lorsqu'active, la timeline joue en avant a la premiere iteration, puis en arriere a la deuxieme, et continue d'alterner les directions, creant un effet ping-pong plutot que de redemarrer du debut a chaque cycle. Le defaut peut etre change globalement via engine.defaults.alternate.

**Faits clés**

- Nom: alternate
- Type: Boolean
- Defaut: false
- Ne prend effet que si loop = true ou > 1
- Effet ping-pong (avant/arriere alterne)
- Defaut global via engine.defaults.alternate

```js
import { createTimeline, utils } from 'animejs';

const [ $loops ] = utils.$('.loops');

let loops = 0;

const tl = createTimeline({
  loop: true,
  alternate: true,
  onLoop: self => $loops.innerHTML = ++loops,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');
```

```js
import { engine } from 'animejs';
engine.defaults.alternate = true;
```

### timeline/timeline-playback-settings/reversed

`https://animejs.com/documentation/timeline/timeline-playback-settings/reversed`

> reversed controle le sens initial de lecture de la timeline ; true demarre la lecture en arriere.

reversed (Boolean, defaut false) controle le sens de lecture initial d'une timeline. A true, la timeline demarre en arriere ; a false, elle joue en avant. Ce reglage n'affecte que le sens initial ; la lecture peut etre inversee a l'execution via la methode reverse(). Le defaut peut etre change globalement via engine.defaults.reversed. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: reversed
- Type: Boolean
- Defaut: false
- true = demarre en arriere
- N'affecte que le sens initial; reverse() pour inverser a l'execution
- Defaut global via engine.defaults.reversed
- Disponible depuis la version 4.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $time ] = utils.$('.time');

const tl = createTimeline({
  reversed: true,
  onUpdate: self => $time.innerHTML = self.currentTime
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');
```

```js
import { engine } from 'animejs';
engine.defaults.reversed = true;
```

### timeline/timeline-playback-settings/autoplay

`https://animejs.com/documentation/timeline/timeline-playback-settings/autoplay`

> autoplay controle si la timeline demarre automatiquement a la creation; accepte aussi onScroll().

autoplay (Boolean | onScroll(), defaut true) controle si une timeline commence a jouer automatiquement a la creation. A true, la lecture commence immediatement. A false, un declenchement manuel est requis via des methodes comme play() ou restart(). On peut alternativement passer onScroll() pour synchroniser la lecture de la timeline avec la position de scroll selon des conditions de seuil. Le defaut peut etre modifie globalement via engine.defaults.autoplay. Disponible depuis la version 2.0.0.

**Faits clés**

- Nom: autoplay
- Type: Boolean | onScroll()
- Defaut: true
- false = declenchement manuel via play()/restart()
- onScroll() = synchronise la lecture avec le scroll
- Defaut global via engine.defaults.autoplay
- Disponible depuis la version 2.0.0

```js
import { engine } from 'animejs';
engine.defaults.autoplay = false;
```

```js
import { createTimeline, utils } from 'animejs';

const [ $paused ] = utils.$('.paused');
const [ $play ] = utils.$('.play');

const tl = createTimeline({
  autoplay: false,
  onUpdate: self => $paused.innerHTML = !!self.paused,
  onComplete: self => $paused.innerHTML = !!self.paused
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');

const playTl = () => tl.paused ? tl.restart() : tl.play();

$play.addEventListener('click', playTl);
```

### timeline/timeline-playback-settings/framerate

`https://animejs.com/documentation/timeline/timeline-playback-settings/framerate`

> frameRate controle le taux de rafraichissement (fps) de la lecture de la timeline.

frameRate (Number > 0, defaut 240) controle le frame rate de lecture (fps) d'une timeline. Le taux reel est plafonne par le taux de rafraichissement du moniteur ou les limitations du navigateur. Cette valeur peut etre modifiee a l'execution via timeline.fps = value. Le defaut peut etre modifie globalement via engine.defaults.frameRate. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: frameRate
- Type: Number (> 0)
- Defaut: 240
- Plafonne par le moniteur / navigateur
- Modifiable a l'execution via timeline.fps = value
- Defaut global via engine.defaults.frameRate
- Disponible depuis la version 4.0.0

```js
import { engine } from 'animejs';
engine.defaults.frameRate = 30;
```

```js
import { createTimeline, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $fps ] = utils.$('.fps');

const tl = createTimeline({
  frameRate: 60,
  loop: true,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');

const updateFps = () => {
  const { value } = $range;
  $fps.innerHTML = value;
  tl.fps = value;
}

$range.addEventListener('input', updateFps);
```

### timeline/timeline-playback-settings/playbackrate

`https://animejs.com/documentation/timeline/timeline-playback-settings/playbackrate`

> playbackRate definit un multiplicateur de vitesse pour la lecture de la timeline.

playbackRate (Number, defaut 1, contrainte >= 0) etablit un multiplicateur de vitesse pour la lecture de la timeline. Les valeurs superieures a 1 accelerent l'execution, celles inferieures a 1 la ralentissent. Une valeur de 0 empeche la lecture. La propriete peut etre ajustee dynamiquement apres creation via timeline.speed = value. Le defaut peut etre modifie globalement via engine.defaults.playbackRate.

**Faits clés**

- Nom: playbackRate
- Type: Number
- Defaut: 1
- Contrainte: >= 0
- > 1 accelere, < 1 ralentit, 0 empeche la lecture
- Ajustable a l'execution via timeline.speed = value
- Defaut global via engine.defaults.playbackRate

```js
import { createTimeline, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $speed ] = utils.$('.speed');

const tl = createTimeline({
  playbackRate: 2,
  loop: true,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '-=500')
.add('.square', { x: '15rem' }, '-=500');

const updateSpeed = () => {
  const speed = utils.roundPad(+$range.value, 1);
  $speed.innerHTML = speed;
  utils.sync(() => tl.speed = speed);
}

$range.addEventListener('input', updateSpeed);
```

```js
import { engine } from 'animejs';
engine.defaults.playbackRate = .75;
```

### timeline/timeline-playback-settings/playbackease

`https://animejs.com/documentation/timeline/timeline-playback-settings/playbackease`

> playbackEase applique une fonction d'easing sur la progression globale de la lecture de la timeline.

playbackEase (ease function, defaut null) applique une fonction d'easing sur l'ensemble de la progression de lecture de la timeline, affectant la facon dont les animations enfants progressent dans le temps plutot que leurs animations individuelles. Le parametre module la courbe de vitesse de lecture de toute la timeline. Alors que chaque animation enfant possede son propre easing, playbackEase enveloppe la progression globale d'une couche d'easing supplementaire, faisant accelerer, ralentir ou suivre une courbe personnalisee a toute la sequence du debut a la fin. Accepte toute valeur valide du parametre ease. Le defaut peut etre modifie globalement via engine.defaults.playbackEase.

**Faits clés**

- Nom: playbackEase
- Type: ease function
- Defaut: null
- Applique un easing sur la progression globale de la timeline
- Accepte toute valeur valide du parametre ease
- Defaut global via engine.defaults.playbackEase

```js
import { createTimeline } from 'animejs';

const tl = createTimeline({
  playbackEase: 'inOut(3)', // this ease is applied across all children
})
.add('.circle', { x: '15rem', ease: 'out(1)' })
.add('.triangle', { x: '15rem', ease: 'out(2)' })
.add('.square', { x: '15rem', ease: 'out(3)' });
```

```js
import { engine } from 'animejs';
engine.defaults.playbackEase = 'inOut';
```

### timeline/timeline-callbacks

`https://animejs.com/documentation/timeline/timeline-callbacks`

> Section listant les callbacks de timeline executes a des moments precis de la lecture, definis dans createTimeline().

Section 'Timeline callbacks'. Permet d'executer des fonctions a des moments specifiques durant la lecture d'une timeline. Ces fonctions callback sont definies directement dans l'objet de parametres de createTimeline(). Callbacks disponibles : onBegin, onComplete, onBeforeUpdate, onUpdate, onRender, onLoop, onPause, then().

**Faits clés**

- Callbacks definis dans l'objet de parametres de createTimeline()
- Liste: onBegin, onComplete, onBeforeUpdate, onUpdate, onRender, onLoop, onPause, then()

```js
createTimeline({
  defaults: {
    ease: 'out(3)',
    duration: 500,
  },
  loop: 3,
  alternate: true,
  autoplay: false,
  onBegin: () => {},
  onLoop: () => {},
  onUpdate: () => {},
});
```

### timeline/timeline-callbacks/onbegin

`https://animejs.com/documentation/timeline/timeline-callbacks/onbegin`

> onBegin s'execute quand une timeline commence sa lecture et recoit l'instance timeline.

onBegin (Function, defaut noop) s'execute quand une timeline commence sa lecture. Il recoit l'instance de timeline comme premier argument. Le callback est retarde de toute valeur delay specifiee avant son execution. Le parametre self donne acces a l'objet timeline et a ses proprietes (comme self.began). Le defaut peut etre modifie globalement via engine.defaults.onBegin.

**Faits clés**

- Nom: onBegin
- Type: Function
- Defaut: noop
- Recoit l'instance timeline (self) comme premier argument
- Retarde par la valeur delay avant execution
- self.began accessible
- Defaut global via engine.defaults.onBegin

```js
import { engine } from 'animejs';
engine.defaults.onBegin = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const tl = createTimeline({
  delay: 1000, // Delays the onBegin() callback by 1000ms
  onBegin: self => $value.textContent = self.began
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' })
.add('.square', { x: '15rem' });
```

### timeline/timeline-callbacks/oncomplete

`https://animejs.com/documentation/timeline/timeline-callbacks/oncomplete`

> onComplete s'execute quand toutes les iterations (boucles) d'une timeline ont fini de jouer.

onComplete (Function, defaut noop) s'execute quand toutes les iterations (boucles) d'une timeline ont fini de jouer. Il recoit l'instance de timeline comme premier argument. Le defaut peut etre modifie globalement via engine.defaults.onComplete. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom: onComplete
- Type: Function
- Defaut: noop
- S'execute quand toutes les iterations/boucles sont finies
- Recoit l'instance timeline (self) comme premier argument
- self.completed accessible
- Defaut global via engine.defaults.onComplete
- Disponible depuis la version 4.0.0

```js
import { engine } from 'animejs';
engine.defaults.onComplete = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const tl = createTimeline({
  defaults: { duration: 500 },
  loop: 1,
  onComplete: self => $value.textContent = self.completed
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' })
.add('.square', { x: '15rem' });
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
  <pre class="large log row">
    <span class="label">completed</span>
    <span class="value">false</span>
  </pre>
</div>
```

### timeline/timeline-callbacks/onbeforeupdate

`https://animejs.com/documentation/timeline/timeline-callbacks/onbeforeupdate`

> onBeforeUpdate s'execute avant la mise a jour des valeurs des animations enfants a chaque frame.

onBeforeUpdate (Function, defaut noop) s'execute avant que les valeurs des animations enfants ne soient mises a jour a chaque frame d'une timeline en cours, se declenchant au frameRate specifie. Il donne acces a l'objet timeline lui-meme via son premier parametre (self). Le defaut peut etre modifie globalement (la doc montre engine.defaults.onUpdate dans l'exemple de defaut global).

**Faits clés**

- Nom: onBeforeUpdate
- Type: Function
- Defaut: noop
- S'execute avant la mise a jour des valeurs des animations enfants a chaque frame
- Se declenche au frameRate specifie
- Recoit l'instance timeline (self) comme premier argument
- L'exemple de defaut global de la doc montre engine.defaults.onUpdate

```js
import { engine } from 'animejs';
engine.defaults.onUpdate = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let updates = 0;

const tl = createTimeline({
  defaults: { duration: 500 },
  loopDelay: 250,
  loop: true,
  onBeforeUpdate: self => $value.textContent = ++updates
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '+=250')
.add('.square', { x: '15rem' }, '+=250');
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
  <pre class="large log row">
    <span class="label">updates</span>
    <span class="value">0</span>
  </pre>
</div>
```

### timeline/timeline-callbacks/onupdate

`https://animejs.com/documentation/timeline/timeline-callbacks/onupdate`

> Callback de timeline execute a chaque frame d'une timeline en cours, selon le frameRate, recevant l'instance de timeline.

onUpdate est un callback de timeline (type Function, valeur par defaut noop) qui execute une fonction a chaque frame d'une timeline en cours, au frameRate specifie. Le callback recoit l'instance de la timeline (self) comme premier argument. La valeur par defaut peut etre modifiee globalement via engine.defaults.onUpdate.

**Faits clés**

- Signature: onUpdate: (self) => void
- Type: Function
- Default: noop
- Le callback recoit l'instance de timeline (self) comme premier argument
- Execute a chaque frame au frameRate specifie
- Modifiable globalement via engine.defaults.onUpdate

```js
onUpdate: (self) => void
```

```js
import { engine } from 'animejs';
engine.defaults.onUpdate = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let updates = 0;

const tl = createTimeline({
  defaults: { duration: 500 },
  loopDelay: 250,
  loop: true,
  onUpdate: self => $value.textContent = ++updates
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '+=250')
.add('.square', { x: '15rem' }, '+=250');
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
  <pre class="large log row">
    <span class="label">updates</span>
    <span class="value">0</span>
  </pre>
</div>
```

### timeline/timeline-callbacks/onrender

`https://animejs.com/documentation/timeline/timeline-callbacks/onrender`

> Callback de timeline execute a chaque rendu de contenu a l'ecran ; ne se declenche pas pendant les delais, loopDelay ou quand aucun enfant ne rend.

onRender est un callback de timeline (type Function, valeur par defaut noop) qui s'execute chaque fois qu'une timeline rend du contenu a l'ecran. Il ne se declenche PAS pendant les periodes de delay, les intervalles loopDelay, ni quand aucun element enfant n'est activement en rendu. Contrairement a onUpdate qui peut s'executer independamment de l'etat de rendu, onRender cible specifiquement les frames ou des pixels sont dessines a l'ecran. Le callback recoit l'instance de timeline (self). Modifiable globalement via engine.defaults.onRender.

**Faits clés**

- Signature: onRender(self: Timeline): void
- Type: Function
- Default: noop
- Ne se declenche pas pendant delay, loopDelay, ni sans enfant en rendu
- Differe de onUpdate : onRender cible les frames ou des pixels sont dessines
- Modifiable globalement via engine.defaults.onRender

```js
onRender(self: Timeline): void
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let renders = 0;

const tl = createTimeline({
  defaults: { duration: 500 },
  loopDelay: 250,
  loop: true,
  onRender: self => $value.textContent = ++renders
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' }, '+=250')
.add('.square', { x: '15rem' }, '+=250');
```

```js
import { engine } from 'animejs';
engine.defaults.onRender = self => console.log(self.id);
```

### timeline/timeline-callbacks/onloop

`https://animejs.com/documentation/timeline/timeline-callbacks/onloop`

> Callback de timeline execute a chaque fois qu'une timeline termine une iteration de boucle.

onLoop est un callback de timeline (type Function, valeur par defaut noop) qui execute une fonction chaque fois qu'une timeline termine un cycle d'iteration. Le callback recoit l'instance de timeline (self) comme premier parametre, permettant l'acces aux proprietes et methodes de la timeline lors des evenements de boucle. Modifiable globalement via engine.defaults.onLoop.

**Faits clés**

- Signature: onLoop: (self: Timeline) => void
- Default: noop
- Execute a chaque iteration de boucle terminee
- Le callback recoit l'instance de timeline (self) comme premier parametre
- Modifiable globalement via engine.defaults.onLoop

```js
onLoop: (self: Timeline) => void
```

```js
import { engine } from 'animejs';
engine.defaults.onLoop = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let loops = 0;

const tl = createTimeline({
  defaults: { duration: 500 },
  loopDelay: 500,
  loop: true,
  onLoop: self => $value.textContent = ++loops
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' })
.add('.square', { x: '15rem' });
```

### timeline/timeline-callbacks/onpause

`https://animejs.com/documentation/timeline/timeline-callbacks/onpause`

> Callback de timeline execute quand une timeline en cours se met en pause, manuellement ou automatiquement.

onPause est un callback de timeline (type Function, valeur par defaut noop) qui s'execute quand une timeline en cours se met en pause, via des methodes manuelles ou des declencheurs automatiques. Le callback recoit l'instance de timeline (self) comme premier argument. Une timeline se met en pause quand : .pause() est appele ; .cancel() est appele ; .revert() est appele ; tous les enfants sont chevauches par une autre timeline/animation avec composition: 'replace' ; tous les targets des animations enfants sont supprimes sans autres timers actifs. Modifiable globalement via engine.defaults.onPause.

**Faits clés**

- Signature: onPause(self: Timeline): void
- Type: Function
- Default: noop
- Declenche par .pause(), .cancel(), .revert()
- Declenche aussi quand tous les enfants sont chevauches par composition: 'replace'
- Declenche quand tous les targets sont supprimes sans autres timers actifs
- Le callback recoit l'instance de timeline (self) comme premier argument
- Modifiable globalement via engine.defaults.onPause

```js
onPause(self: Timeline): void
```

```js
import { engine } from 'animejs';
engine.defaults.onPause = self => console.log(self.id);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $animateButton, $pauseButton, $removeButton ] = utils.$('.button');
const [ $value ] = utils.$('.value');
const shapes = utils.$('.shape');
const [ $triangle, $square, $circle ] = shapes;

let paused = 0;
let alternate = 0;
let tl;

const animateShapes = () => {
  alternate = !alternate;
  const x = (alternate ? 15 : 0) + 'rem';
  const rotate = (alternate ? 360 : -360);
  tl = createTimeline({
    defaults: { duration: 2000 },
    onPause: () => $value.textContent = ++paused
  })
  .add($circle, { x }, 0)
  .add($triangle, { x }, 0)
  .add($square, { x }, 0)
  .add(shapes, { rotate }, 0);
}

const pauseTL = () => {
  if (tl) tl.pause();
}

const removeTargets = () => {
  utils.remove(shapes);
}

animateShapes();
$animateButton.addEventListener('click', animateShapes);
$pauseButton.addEventListener('click', pauseTL);
$removeButton.addEventListener('click', removeTargets);
```

### timeline/timeline-callbacks/then

`https://animejs.com/documentation/timeline/timeline-callbacks/then`

> Methode retournant une Promise qui se resout et execute un callback quand la timeline se termine ; utilisable en chainage ou avec async/await.

then(callback) prend un parametre callback de type Function, execute quand la timeline se termine, recevant la timeline comme premier argument. La methode retourne une Promise qui se resout a la fin de la timeline. Elle peut etre chainee directement ou utilisee avec la syntaxe async/await pour gerer la fin de la timeline. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: then(callback)
- callback: Function, execute a la fin de la timeline, recoit la timeline comme premier argument
- Retourne une Promise resolue a la fin de la timeline
- Utilisable en chainage direct ou avec async/await
- Disponible depuis la version 4.0.0

```js
then(callback)
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const tl = createTimeline({
  defaults: { duration: 500 },
  loop: 1,
})
.add('.circle', { x: '15rem' })
.add('.triangle', { x: '15rem' })
.add('.square', { x: '15rem' });

tl.then(() => $value.textContent = 'fulfilled');
```

```js
async function waitForTimelineToComplete() {
  return createTimeline()
  .add('.square', { x: 100 })
  .add('.square', { y: 100 });
}

const asyncTimeline = await waitForTimelineToComplete();
```

### timeline/timeline-methods

`https://animejs.com/documentation/timeline/timeline-methods`

> Page d'index listant les methodes disponibles sur l'instance Timeline retournee par createTimeline().

Cette page d'index liste les methodes disponibles sur l'instance Timeline retournee par la fonction createTimeline(). La documentation enumere les methodes (add, set, sync, label, remove, call, init, play, reset, reverse, pause, restart, alternate, resume, complete, cancel, revert, seek, stretch, refresh). Ces methodes permettent de controler le timing, le comportement et la progression d'une timeline : composition (ajouter/synchroniser du contenu), controle de lecture (play/pause/restart), navigation temporelle (seek), et gestion d'etat (revert/refresh). La page d'index ne fournit que la liste ; les details (parametres, exemples) sont sur les pages individuelles de chaque methode.

**Faits clés**

- Methodes disponibles sur l'instance Timeline retournee par createTimeline()
- Liste de methodes incluant : add(), set(), sync(), label(), remove(), call(), init()
- Egalement methodes de controle heritees : play(), reset(), reverse(), pause(), restart(), alternate(), resume(), complete(), cancel(), revert(), seek(), stretch(), refresh()
- La page d'index ne contient pas d'exemples de code ; details sur pages individuelles

### timeline/timeline-methods/add

`https://animejs.com/documentation/timeline/timeline-methods/add`

> Cree et ajoute des animations et timers a une timeline a une position specifiee ; le type d'element ajoute depend des parametres passes.

add() cree et ajoute des animations et timers a une timeline. Le type d'element ajoute a la timeline depend des parametres passes a add(). Deux formes : timeline.add(targets, parameters, position) pour des animations, ou timeline.add(timerParameters, position) pour des timers seuls. targets (Targets, requis) = elements DOM, selecteurs CSS, ou objets JS a animer. parameters (requis) = parametres d'animation/timer (proprietes a animer et configuration de lecture). position (Time position, optionnel) = point d'insertion dans la timeline. timerParameters (requis) = parametres de timer pour les ajouts de timer seul. La methode retourne la timeline elle-meme, permettant le chainage.

**Faits clés**

- Signature: timeline.add(targets, parameters, position) OU timeline.add(timerParameters, position)
- targets: Targets (requis) - elements DOM, selecteurs CSS, objets JS
- parameters: parametres animation/timer (requis)
- position: Time position (optionnel) - point d'insertion
- timerParameters: parametres de timer pour ajout de timer seul
- Le type d'element ajoute depend des parametres passes
- Retourne la timeline (chainable)

```js
timeline.add(targets, parameters, position);
// OR
timeline.add(timerParameters, position);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const tl = createTimeline()
  .label('start timer 1', 0)
  .label('animate circle', 1000)
  .label('start timer 2', 2000)
  .add({
    duration: 1000,
    onUpdate: self => $value.innerHTML = self.currentTime,
  }, 'start timer 1')
  .add('.circle', {
    duration: 2000,
    x: '16rem',
  }, 'animate circle')
  .add({
    duration: 1000,
    onUpdate: self => $value.innerHTML = self.currentTime,
  }, 'start timer 2');
```

### timeline/timeline-methods/set

`https://animejs.com/documentation/timeline/timeline-methods/set`

> Definit instantanement les valeurs de proprietes des targets a un moment specifique de la timeline, sans duree d'animation.

set() definit instantanement les valeurs de proprietes des targets a un moment specifique de la timeline. Signature : timeline.set(targets, parameters, position). targets (Targets, requis) = elements DOM ou objets. parameters (Animatable properties, requis) = proprietes animables a assigner. position (Time position, optionnel) = moment de la timeline ou positionner les valeurs. Permet d'assigner des valeurs sans duree d'animation. La methode supporte le chainage et retourne l'instance de timeline. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: timeline.set(targets, parameters, position)
- targets: Targets (requis)
- parameters: Animatable properties (requis)
- position: Time position (optionnel)
- Definit les valeurs instantanement, sans duree d'animation
- Retourne la timeline (chainable)
- Disponible depuis la version 4.0.0

```js
timeline.set(targets, parameters, position);
```

```js
import { createTimeline } from 'animejs';

const tl = createTimeline()
.set('.circle', { x: '15rem' })
.set('.triangle', { x: '15rem' }, 500)
.set('.square', { x: '15rem' }, 1000);
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
</div>
```

### timeline/timeline-methods/sync

`https://animejs.com/documentation/timeline/timeline-methods/sync`

> Synchronise une JSAnimation, un Timer, une Timeline ou une WAAPIAnimation a une timeline a une position donnee.

sync(synced, position?) synchronise une animation JavaScript, une WAAPI Animation, un timer, une timeline, ou une WAAPI Animation native a une timeline. synced (type JSAnimation | Timer | Timeline | Anime.js WAAPIAnimation | WAAPIAnimation) = l'animation ou timeline a synchroniser avec la timeline courante. position (Time position, optionnel) = ou placer le contenu synchronise dans la timeline. La composition de la valeur du tween est determinee a la creation de la timeline et n'altere pas les enfants existants quand elle est ajoutee. Retourne la timeline elle-meme (chainable). Disponible depuis la v4.0.0.

**Faits clés**

- Signature: sync(synced, position?)
- synced: JSAnimation | Timer | Timeline | Anime.js WAAPIAnimation | WAAPIAnimation
- position: Time position (optionnel)
- La composition de la valeur du tween est determinee a la creation de la timeline et n'altere pas les enfants existants
- Retourne la timeline (chainable)
- Disponible depuis la v4.0.0

```js
sync(synced, position?)
```

```js
import { createTimeline, animate, waapi } from 'animejs';

const circleAnimation = waapi.animate('.circle', {
  x: '15rem'
});

const tlA = createTimeline()
.sync(circleAnimation)
.add('.triangle', {
  x: '15rem',
  duration: 2000,
})
.add('.square', {
  x: '15rem',
});

const tlB = createTimeline({ defaults: { duration: 2000 } })
.add(['.triangle', '.square'], {
  rotate: 360,
}, 0)
.add('.circle', {
  scale: [1, 1.5, 1],
}, 0);

const tlMain = createTimeline()
.sync(tlA)
.sync(tlB, '-=2000');
```

### timeline/timeline-methods/label

`https://animejs.com/documentation/timeline/timeline-methods/label`

> Associe une position temporelle a un nom de label pour une reference facile dans la timeline.

label() associe des positions temporelles specifiques a des noms de label pour une reference facile dans la timeline. Signature : timeline.label(labelName, position). labelName (String, requis) = identifiant du label. position (Time position, optionnel ; defaut = position courante de la timeline). Une fois cree, les labels fonctionnent comme references de position temporelle dans les operations de timeline (par ex. dans add()). Retourne l'instance de timeline (chainable). Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: timeline.label(labelName, position)
- labelName: String (requis)
- position: Time position (optionnel, defaut = position courante de la timeline)
- Les labels sont references comme positions temporelles dans d'autres methodes comme add()
- Retourne la timeline (chainable)
- Disponible depuis la version 4.0.0

```js
timeline.label(labelName, position);
```

```js
import { createTimeline } from 'animejs';

const tl = createTimeline()
.label('circle', 0)
.label('square', 500)
.label('triangle', 1000)
.add('.square', {
  x: '17rem',
  duration: 500,
}, 'square')
.add('.circle', {
  x: '13rem',
  duration: 1000,
}, 'circle')
.add('.triangle', {
  x: '15rem',
  rotate: '1turn',
  duration: 500,
}, 'triangle');
```

### timeline/timeline-methods/remove

`https://animejs.com/documentation/timeline/timeline-methods/remove`

> Supprime des animations, timers, timelines, targets ou proprietes de tween specifiques d'une timeline ; la suppression n'affecte pas la duree.

remove() supprime des animations, timers, timelines, targets ou proprietes de tween specifiques de la timeline. Plusieurs formes : timeline.remove([animation, timer, timeline]) pour supprimer des animations/timers/timelines individuels ; timeline.remove(targets) pour supprimer par selecteur CSS/element DOM/objet ; timeline.remove(targets, propertyName) pour supprimer une propriete animable specifique. Parametres : object (optionnel, Animation | Timer | Timeline) ; targets (Targets) ; propertyName (String, optionnel) ; position (Time position, optionnel). La timeline se met automatiquement en pause quand tous les items sont supprimes. Important : supprimer des items d'une timeline n'affecte pas sa duree. Retourne la timeline (chainable).

**Faits clés**

- Signatures: remove([animation, timer, timeline]) / remove(targets) / remove(targets, propertyName)
- object (optionnel): Animation | Timer | Timeline
- targets: Targets ; propertyName: String (optionnel) ; position: Time position (optionnel)
- La timeline se met automatiquement en pause quand tous les items sont supprimes
- Supprimer des items n'affecte PAS la duree de la timeline
- Retourne la timeline (chainable)

```js
timeline.remove([animation, timer, timeline])
timeline.remove(targets)
timeline.remove(targets, propertyName)
```

```js
import { animate, createTimeline, utils } from 'animejs';

const [ $removeA, $removeB, $removeC ] = utils.$('.button');

const animation = animate('.circle', { x: '15rem', scale: [1, .5, 1] });

const tl = createTimeline({ loop: true, alternate: true })
  .sync(animation)
  .add('.triangle', { x: '15rem', rotate: 360 }, 100)
  .add('.square',   { x: '15rem' }, 200);

const removeAnimation = () => tl.remove(animation);
const removeTarget = () => tl.remove('.square');
const removeRotate = () => tl.remove('.triangle', 'rotate');

$removeA.addEventListener('click', removeAnimation);
$removeB.addEventListener('click', removeTarget);
$removeC.addEventListener('click', removeRotate);
```

### timeline/timeline-methods/call

`https://animejs.com/documentation/timeline/timeline-methods/call`

> Appelle la fonction callback passee a la position temporelle specifiee dans la timeline.

call() appelle la fonction callback passee a la position temporelle specifiee. Signature : timeline.call(callback, position). callback (Function, requis) = la fonction a executer. position (Time position, optionnel) = quand le callback doit se declencher. Cette methode permet d'executer une logique personnalisee a des moments precis de la lecture de la timeline, utile pour declencher des effets de bord ou coordonner des mises a jour non-animees. Retourne l'instance de timeline (chainable).

**Faits clés**

- Signature: timeline.call(callback, position)
- callback: Function (requis)
- position: Time position (optionnel)
- Execute une logique a un moment precis de la lecture de la timeline
- Retourne la timeline (chainable)

```js
timeline.call(callback, position);
```

```js
import { createTimeline, utils } from 'animejs';

const [ $functionA ] = utils.$('.function-A');
const [ $functionB ] = utils.$('.function-B');
const [ $functionC ] = utils.$('.function-C');

const tl = createTimeline()
.call(() => $functionA.innerHTML = 'A', 0)
.call(() => $functionB.innerHTML = 'B', 800)
.call(() => $functionC.innerHTML = 'C', 1200);
```

### timeline/timeline-methods/init

`https://animejs.com/documentation/timeline/timeline-methods/init`

> Initialise les valeurs de depart de tous les enfants d'une timeline, forcant un rendu immediat de leur etat initial.

init() initialise les valeurs de depart de tous les elements enfants d'une timeline. Aucun parametre requis. Contrairement aux appels animate() standard, les animations avec des valeurs from explicites ajoutees a une timeline ne rendent pas automatiquement leur etat initial ; elles ne rendent que lorsque la tete de lecture de la timeline les atteint. Cette methode force un rendu immediat des etats initiaux de tous les enfants. Particulierement utile quand on veut que les animations avec valeurs from aient leur etat initial immediatement visible plutot que d'attendre que la timeline progresse jusqu'a chaque element. Retourne l'instance de timeline (chainable).

**Faits clés**

- Signature: init() — aucun parametre
- Initialise les valeurs de depart de tous les enfants de la timeline
- Les animations avec valeurs from ne rendent pas leur etat initial avant que la tete de lecture les atteigne ; init() force ce rendu immediatement
- Retourne la timeline (chainable)

```js
init()
```

```js
import { createTimeline } from 'animejs';

const tl = createTimeline()
.add('.square',   { x: { from: '15rem' } })
.add('.triangle', { x: { from: '15rem' } }, 500)
.add('.circle',   { x: { from: '15rem' } }, 1000)
.init();
```

### timeline/timeline-methods/play

`https://animejs.com/documentation/timeline/timeline-methods/play`

> La methode play() force la timeline a jouer en avant depuis sa position courante.

play() force la timeline a jouer vers l'avant (forward). Elle reprend/lance la lecture depuis la position courante dans le sens forward. Aucun parametre. Retourne la timeline elle-meme, ce qui permit le chainage avec d'autres methodes de timeline. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: play()
- Aucun parametre
- Retourne la timeline elle-meme (chainable)
- Force la timeline a jouer forward depuis la position courante
- Disponible depuis la version 2.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $playButton ] = utils.$('.play');

const tl = createTimeline({
  autoplay: false
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const playTimeline = () => tl.play();

$playButton.addEventListener('click', playTimeline);
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button play">Play</button>
  </fieldset>
</div>
```

### timeline/timeline-methods/reset

`https://animejs.com/documentation/timeline/timeline-methods/reset`

> reset() met la timeline en pause et restaure ses proprietes internes a leur etat initial, avec une option de soft reset.

reset(softReset) met la timeline en pause et restaure les proprietes currentTime, progress, reversed, began et completed a leur etat initial. Le parametre optionnel softReset (Boolean, defaut false) : si true, ne reinitialise que les valeurs internes sans provoquer de rendu visuel. Retourne l'instance de la timeline pour permettre le chainage de methodes.

**Faits clés**

- Signature: timeline.reset(softReset)
- Parametre softReset: Boolean, optionnel, defaut false — si true, reinitialise seulement les valeurs internes sans rendu visuel
- Met la timeline en pause et restaure currentTime, progress, reversed, began, completed a leur etat initial
- Retourne l'instance de la timeline (chainable)

```js
import { createTimer, utils } from 'animejs';

const [ $reset ] = utils.$('.button');

const tl = createTimeline({
  loop: true,
  alternate: true
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const resetTimeline = () => {
  tl.reset();
  $time.innerHTML = timer.currentTime;
}

$reset.addEventListener('click', resetTimeline);
```

### timeline/timeline-methods/reverse

`https://animejs.com/documentation/timeline/timeline-methods/reverse`

> reverse() force la timeline a jouer en arriere (backward).

reverse() force la timeline a jouer vers l'arriere (backward). Aucun parametre. Retourne la timeline elle-meme, chainable avec d'autres methodes de timeline. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: reverse()
- Aucun parametre
- Force la timeline a jouer backward
- Retourne la timeline elle-meme (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $reverseButton ] = utils.$('.reverse');

const tl = createTimeline()
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const reverseTimeline = () => tl.reverse();

$reverseButton.addEventListener('click', reverseTimeline);
```

```js
<div class="large row">
  <div class="medium pyramid">
    <div class="triangle"></div>
    <div class="square"></div>
    <div class="circle"></div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button reverse">Reverse</button>
  </fieldset>
</div>
```

### timeline/timeline-methods/pause

`https://animejs.com/documentation/timeline/timeline-methods/pause`

> pause() arrete la lecture d'une timeline active en conservant sa position courante.

pause() stoppe la lecture d'une timeline active. Elle suspend toutes les animations de la timeline dans leur etat courant sans les reinitialiser ni les completer. La timeline conserve sa position de lecture courante et peut etre reprise avec resume(). Aucun parametre. Retourne la timeline elle-meme, permettant le chainage de methodes. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: pause()
- Aucun parametre
- Suspend les animations dans leur etat courant sans reset ni complete
- La position de lecture est conservee; reprise possible via resume()
- Retourne la timeline elle-meme (chainable)
- Disponible depuis la version 2.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $pauseButton ] = utils.$('.pause');

const tl = createTimeline({
  loop: true,
  alternate: true,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const pauseTimeline = () => tl.pause();

$pauseButton.addEventListener('click', pauseTimeline);
```

### timeline/timeline-methods/restart

`https://animejs.com/documentation/timeline/timeline-methods/restart`

> restart() remet currentTime a 0 et restaure les proprietes animees a leur etat initial, relancant la lecture si autoplay est actif.

restart() remet le currentTime de la timeline a 0 et restaure toutes les proprietes animees des elements a leur etat initial. Si la timeline a autoplay active, elle relance automatiquement la lecture apres le restart. Aucun parametre. Retourne l'instance de la timeline, chainable. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: restart()
- Aucun parametre
- Met currentTime a 0 et restaure les proprietes animees a leur etat initial
- Relance la lecture automatiquement si autoplay est active
- Retourne l'instance de la timeline (chainable)
- Disponible depuis la version 2.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $restartButton ] = utils.$('.restart');

const tl = createTimeline({
  loop: true,
  alternate: true,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const restartTimeline = () => tl.restart();

$restartButton.addEventListener('click', restartTimeline);
```

### timeline/timeline-methods/alternate

`https://animejs.com/documentation/timeline/timeline-methods/alternate`

> alternate() bascule le sens de lecture en ajustant le currentTime pour refleter la nouvelle progression temporelle.

alternate() bascule (toggle) le sens de lecture tout en ajustant la position currentTime pour refleter la nouvelle progression temporelle. Cette methode inverse le sens de lecture de la timeline et maintient un positionnement temporel approprie. Aucun parametre. Retourne l'instance de la timeline, chainable. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: alternate()
- Aucun parametre
- Toggle le sens de lecture en ajustant currentTime pour refleter la nouvelle progression
- Retourne l'instance de la timeline (chainable)
- Disponible depuis la version 2.0.0

```js
import { creatTimeline, utils } from 'animejs';

const [ $alternateButton ] = utils.$('.button');

const tl = createTimeline({ loop: true })
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const pauseTimeline = () => tl.pause();
const playTimeline = () => tl.play();
const alternateTimeline = () => tl.alternate();

$alternateButton.addEventListener('click', alternateTimeline);
```

### timeline/timeline-methods/resume

`https://animejs.com/documentation/timeline/timeline-methods/resume`

> resume() reprend la lecture d'une timeline en pause dans son sens de lecture courant.

resume() reprend la lecture d'une timeline en pause dans son sens (direction) courant. Une timeline precedemment mise en pause continue d'animer la ou elle s'etait arretee, en conservant le sens (forward ou reverse) actif avant la pause. Aucun parametre. Retourne l'instance de la timeline, chainable. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: resume()
- Aucun parametre
- Reprend la lecture d'une timeline en pause dans son sens courant
- Conserve la direction (forward/reverse) active avant la pause
- Retourne l'instance de la timeline (chainable)
- Disponible depuis la version 2.0.0

```js
import { creatTimeline, utils } from 'animejs';

const [ $pauseButton, $alternateButton, $resumeButton ] = utils.$('.button');

const tl = createTimeline({ loop: true })
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const pauseTimeline = () => tl.pause();
const alternateTimeline = () => tl.alternate();
const resumeTimeline = () => tl.resume();

$pauseButton.addEventListener('click', pauseTimeline);
$alternateButton.addEventListener('click', alternateTimeline);
$resumeButton.addEventListener('click', resumeTimeline);
```

### timeline/timeline-methods/complete

`https://animejs.com/documentation/timeline/timeline-methods/complete`

> complete() complete la timeline instantanement en avancant a son etat final.

complete() complete la timeline instantanement : elle avance immediatement la timeline a son etat final, executant toutes les animations restantes jusqu'a leur point de fin sans delai supplementaire. Aucun parametre. Retourne l'instance de la timeline, chainable. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: complete() -> Timeline
- Aucun parametre
- Complete la timeline instantanement (avance a l'etat final)
- Retourne l'instance de la timeline (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $completeButton ] = utils.$('.complete');

const tl = createTimeline({
  loop: true,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const completeTimeline = () => tl.complete();

$completeButton.addEventListener('click', completeTimeline);
```

### timeline/timeline-methods/cancel

`https://animejs.com/documentation/timeline/timeline-methods/cancel`

> cancel() met la timeline en pause, la retire de la boucle principale du moteur et libere la memoire associee.

cancel() met la timeline en pause, la retire de la boucle principale (main loop) du moteur et libere les ressources memoire associees. Aucun parametre. Retourne la timeline elle-meme, permettant le chainage de methodes. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: cancel()
- Aucun parametre
- Met en pause, retire de la main loop du moteur, libere la memoire
- Retourne la timeline elle-meme (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $cancelButton ] = utils.$('.cancel');
const [ $playButton ] = utils.$('.play');

const tl = createTimeline({
  loop: true,
  alternate: true,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const cancelTimeline = () => tl.cancel();
const playTimeline = () => tl.play();

$cancelButton.addEventListener('click', cancelTimeline);
$playButton.addEventListener('click', playTimeline);
```

### timeline/timeline-methods/revert

`https://animejs.com/documentation/timeline/timeline-methods/revert`

> revert() termine completement la timeline : annule la lecture, restaure les valeurs originales, retire les styles inline et deconnecte les instances onScroll() liees.

revert() termine completement une timeline. Elle annule la lecture, restaure toutes les valeurs de proprietes animees a leur etat original, retire les styles CSS inline et deconnecte toute instance onScroll() liee. C'est le choix approprie quand on veut arreter et detruire entierement une timeline plutot que simplement la mettre en pause. Aucun parametre. Retourne l'instance de la timeline, chainable.

**Faits clés**

- Signature: revert()
- Aucun parametre
- Annule la lecture, restaure les valeurs originales, retire les styles CSS inline, deconnecte les onScroll() lies
- Detruit completement la timeline (a distinguer de pause)
- Retourne l'instance de la timeline (chainable)

```js
import { createTimeline, utils } from 'animejs';

const [ $revertButton ] = utils.$('.revert');
const [ $restartButton ] = utils.$('.restart');

// Set an initial x value
utils.set(['.circle', '.triangle', '.square'], { x: '15rem' });

const tl = createTimeline({
  loop: true,
  alternate: true,
})
.add('.circle',   { x: 0 })
.add('.triangle', { x: 0 }, 500)
.add('.square',   { x: 0 }, 1000);

const revertTimeline = () => tl.revert();
const restartTimeline = () => tl.restart();

$revertButton.addEventListener('click', revertTimeline);
$restartButton.addEventListener('click', restartTimeline);
```

### timeline/timeline-methods/seek

`https://animejs.com/documentation/timeline/timeline-methods/seek`

> seek() positionne la lecture de la timeline a un temps precis, avec option pour museler les callbacks.

seek(time, muteCallbacks) met a jour la position de lecture courante de la timeline a une valeur de temps specifique, permettant un seeking manuel dans le contenu. Le parametre time (Number) est le nouveau currentTime en millisecondes de la timeline. Le parametre muteCallbacks (Boolean, defaut false) : si true, empeche les callbacks de se declencher. Retourne la timeline elle-meme, chainable. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: timeline.seek(time, muteCallbacks)
- Parametre time: Number — nouveau currentTime en millisecondes
- Parametre muteCallbacks: Boolean, defaut false — si true, empeche les callbacks de se declencher
- Met a jour la position de lecture a un temps precis (seeking manuel)
- Retourne la timeline elle-meme (chainable)
- Disponible depuis la version 2.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $playPauseButton ] = utils.$('.play-pause');

const updateButtonLabel = tl => {
  $playPauseButton.textContent = tl.paused ? 'Play' : 'Pause';
}

const tl = createTimeline({
  autoplay: false,
  onUpdate: self => {
    $range.value = self.currentTime;
    updateButtonLabel(self);
  },
  onComplete: updateButtonLabel,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const seekTimeline = () => tl.seek(+$range.value);

const playPauseTimeline = () => {
  if (tl.paused) {
    tl.play();
  } else {
    tl.pause();
    updateButtonLabel(tl);
  }
}

$range.addEventListener('input', seekTimeline);
$playPauseButton.addEventListener('click', playPauseTimeline);
```

### timeline/timeline-methods/stretch

`https://animejs.com/documentation/timeline/timeline-methods/stretch`

> stretch() change la duree totale d'une timeline et de ses enfants pour l'ajuster a un temps specifique.

stretch(duration) change la duree totale d'une timeline et de ses enfants pour qu'elle s'ajuste a un temps specifique. Le parametre duration (Number) est la nouvelle duree totale en millisecondes de la timeline. La duree totale equivaut a la duree d'une iteration multipliee par le nombre total d'iterations. Par exemple, une timeline de 1000ms qui boucle deux fois (3 iterations au total) a une duree totale de 3000ms. Retourne la timeline elle-meme, chainable.

**Faits clés**

- Signature: timeline.stretch(duration)
- Parametre duration: Number — nouvelle duree totale en millisecondes
- Change la duree totale de la timeline ET de ses enfants
- Duree totale = duree d'iteration x nombre total d'iterations (ex: 1000ms loop:1 / 3 iterations = 3000ms)
- Retourne la timeline elle-meme (chainable)

```js
import { createTimeline, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $totalDuration ] = utils.$('.value');

const tl = createTimeline({
  loop: 1,
  alternate: true,
})
.add('.circle',   { x: '15rem' })
.add('.triangle', { x: '15rem' }, 500)
.add('.square',   { x: '15rem' }, 1000);

const stretchTimeline = () => {
  const newDuration = +$range.value;
  $totalDuration.textContent = newDuration;
  tl.stretch(newDuration).restart();
}

stretchTimeline();
$range.addEventListener('input', stretchTimeline);
```

### timeline/timeline-methods/refresh

`https://animejs.com/documentation/timeline/timeline-methods/refresh`

> refresh() recalcule les valeurs animees des enfants definies par des fonctions, en mettant a jour leurs valeurs from et to.

refresh() recalcule les valeurs animees des enfants de la timeline definies par des valeurs basees sur des fonctions (function-based values), en mettant a jour leurs valeurs 'from' aux valeurs cibles courantes et leurs valeurs 'to' aux valeurs nouvellement calculees. Seules les valeurs de proprietes animables sont recalculees ; la duree (duration) et le delai (delay) ne peuvent pas etre rafraichis. Aucun parametre. Retourne la timeline elle-meme, chainable. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: refresh()
- Aucun parametre
- Recalcule les valeurs des enfants definies par des function-based values (met a jour from -> valeurs cibles courantes, to -> valeurs recalculees)
- Seules les valeurs de proprietes animables sont recalculees; duration et delay ne peuvent pas etre rafraichis
- Retourne la timeline elle-meme (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimeline, utils } from 'animejs';

const [ $refreshButton ] = utils.$('.refresh');

const tl = createTimeline({
  loop: true,
  onLoop: self => self.refresh()
})
.add('.circle',   { x: () => utils.random(0, 15) + 'rem' }, 0)
.add('.triangle', { x: () => utils.random(0, 15) + 'rem' }, 0)
.add('.square',   { x: () => utils.random(0, 15) + 'rem' }, 0);

const refreshTimeline = () => tl.refresh().restart();

$refreshButton.addEventListener('click', refreshTimeline);
```

### timeline/timeline-properties

`https://animejs.com/documentation/timeline/timeline-properties`

> Liste des proprietes accessibles sur une instance Timeline retournee par createTimeline(), avec leurs types et comportement get/set ou read-only.

Les instances Timeline retournees par createTimeline() exposent un ensemble de proprietes permettant de lire et/ou modifier l'etat de la timeline. La plupart sont a la fois lisibles et modifiables (get/set), quelques-unes sont en lecture seule. id (String | Number) lit et definit l'identifiant de la timeline. labels (Object) lit et definit la map des labels de position temporelle. currentTime (Number) lit et definit le temps courant global en millisecondes. iterationCurrentTime (Number) lit et definit le temps courant de l'iteration en cours en millisecondes. deltaTime (Number) lit le temps ecoule en ms entre la frame courante et la precedente (lecture seule). progress (Number) lit et definit la progression globale de 0 a 1. iterationProgress (Number) lit et definit la progression de l'iteration en cours de 0 a 1. currentIteration (Number) lit et definit le compteur d'iteration courant. duration (Number) lit la duree totale en millisecondes (lecture seule). speed (Number) lit et definit le multiplicateur de vitesse. fps (Number) lit et definit les frames par seconde. paused (Boolean) lit et definit l'etat de pause. began (Boolean) lit et definit si l'animation a commence. completed (Boolean) lit et definit l'etat de completion. reversed (Boolean) lit et definit l'etat de lecture inversee. backwards (Boolean) lit si la lecture se fait actuellement vers l'arriere (lecture seule).

**Faits clés**

- id: String | Number — get/set identifiant
- labels: Object — get/set map des labels de position temporelle
- currentTime: Number — get/set temps courant global (ms)
- iterationCurrentTime: Number — get/set temps courant de l'iteration (ms)
- deltaTime: Number — read-only, temps ecoule (ms) entre frame courante et precedente
- progress: Number — get/set progression globale 0 a 1
- iterationProgress: Number — get/set progression iteration courante 0 a 1
- currentIteration: Number — get/set compteur d'iteration
- duration: Number — read-only, duree totale (ms)
- speed: Number — get/set multiplicateur de vitesse
- fps: Number — get/set frames par seconde
- paused: Boolean — get/set etat de pause
- began: Boolean — get/set animation demarree
- completed: Boolean — get/set etat de completion
- reversed: Boolean — get/set lecture inversee
- backwards: Boolean — read-only, lecture vers l'arriere en cours
- Aucun exemple de code ni valeur par defaut fourni pour les proprietes Timeline specifiquement


## animatable

### animatable

`https://animejs.com/documentation/animatable`

> createAnimatable() cree des instances animant efficacement les proprietes de cibles, remplacant animate() et utils.set() pour les mises a jour de valeurs frequentes.

createAnimatable() cree des instances qui animent efficacement les proprietes des cibles. C'est un remplacement performant de animate() et utils.set() dans les scenarios impliquant des mises a jour de valeurs frequentes (evenements de curseur, boucles d'animation). On l'importe via import { createAnimatable } from 'animejs'; ou en module standalone via import { createAnimatable } from 'animejs/animatable';. Signature : const animatable = createAnimatable(targets, parameters); ou targets sont les Targets (selecteurs CSS, elements DOM, objets ou tableaux) et parameters un objet contenant les Animatable settings. Retourne une instance Animatable. Une fois creee, l'instance expose des fonctions de propriete : appelee avec une valeur elle declenche une animation (animatable.propertyName(value, duration, ease)), appelee sans argument elle retourne la valeur courante (animatable.propertyName()). Contrainte importante : seules des valeurs Number ou Array<Number> sont acceptees par les fonctions de propriete.

**Faits clés**

- Signature: const animatable = createAnimatable(targets, parameters)
- targets: CSS selectors, DOM elements, objects, or arrays
- parameters: objet d'Animatable settings
- Retourne une instance Animatable
- Remplacement performant de animate() et utils.set() pour mises a jour frequentes
- Property functions: propertyName(value, duration, ease) anime; propertyName() retourne la valeur courante
- Contrainte: seules valeurs Number ou Array<Number> acceptees par les property functions
- Import standalone: 'animejs/animatable'

```js
import { createAnimatable } from 'animejs';
```

```js
import { createAnimatable } from 'animejs/animatable';
```

```js
const animatable = createAnimatable(targets, parameters);
```

```js
animatable.propertyName(value, duration, ease); // Triggers animation
animatable.propertyName();                       // Returns current value
```

```js
import { createAnimatable, utils } from 'animejs';

const animatableSquare = createAnimatable('.square', {
  x: 500, // x animation duration: 500ms
  y: 500, // y animation duration: 500ms
  ease: 'out(3)',
});

const onMouseMove = e => {
  const x = utils.clamp(e.clientX - left - hw, -hw, hw);
  const y = utils.clamp(e.clientY - top - hh, -hh, hh);
  animatableSquare.x(x); // Animates x in 500ms
  animatableSquare.y(y); // Animates y in 500ms
}

window.addEventListener('mousemove', onMouseMove);
```

### animatable/animatable-settings

`https://animejs.com/documentation/animatable/animatable-settings`

> Les settings d'une instance Animatable peuvent etre appliques globalement a toutes les proprietes ou specifiquement a une propriete via un objet ; quatre options : unit, duration, ease, modifier.

Les Animatable settings (Anime.js 4.0.0+) configurent les proprietes animees. Ils peuvent etre appliques de deux manieres : globalement (appliques a toutes les proprietes sur l'objet parameters) ou par propriete (appliques specifiquement a une propriete en passant un objet). Quatre options de configuration existent : unit (specifie l'unite de mesure des valeurs), duration (controle le timing de l'animation), ease (definit les fonctions d'easing) et modifier (applique des transformations de valeur). Cette section fait partie de la categorie Animatable, qui inclut aussi les methods (getters, setters, revert) et la documentation des properties.

**Faits clés**

- Settings applicables globalement (sur l'objet parameters) ou par propriete (en passant un objet)
- Quatre settings: unit, duration, ease, modifier
- unit: unite de mesure des valeurs
- duration: timing de l'animation
- ease: fonction d'easing
- modifier: transformation de valeur
- Anime.js 4.0.0+

```js
createAnimatable(targets, {
  x: {
    unit: 'rem',
    duration: 400,
    ease: 'out(4)'
  },
  y: 200,
  rotate: 1000,
  ease: 'out(2)' // global setting
});
```

### animatable/animatable-settings/unit

`https://animejs.com/documentation/animatable/animatable-settings/unit`

> Le setting unit definit l'unite de mesure CSS appliquee a une valeur de propriete animee dans une instance Animatable.

Le setting unit (type String) definit l'unite de mesure appliquee a la valeur d'une propriete animee dans une instance Animatable. Il accepte une chaine d'unite CSS valide. Cela permet de travailler avec differents systemes d'unites — par exemple specifier les rotations en radians ('rad') plutot qu'en degres. La valeur par defaut n'est pas precisee dans la documentation. Aucun gotcha ni note supplementaire n'est documente pour ce parametre.

**Faits clés**

- Parametre: unit
- Type: String
- Accepte: une chaine d'unite CSS valide
- Valeur par defaut: non precisee dans la documentation
- Permet ex. rotations en radians ('rad') au lieu de degres
- Peut etre defini par propriete: rotate: { unit: 'rad' }

```js
import { createAnimatable, utils } from 'animejs';

const $demos = document.querySelector('#docs-demos');
const [ $clock ] = utils.$('.clock');
let bounds = $clock.getBoundingClientRect();
const refreshBounds = () => bounds = $clock.getBoundingClientRect();

const clock = createAnimatable($clock, {
  rotate: { unit: 'rad' }, // Set the unit to 'rad'
  duration: 400,
});

const { PI } = Math;
let lastAngle = 0
let angle = PI / 2;

const onMouseMove = e => {
  const { width, height, left, top } = bounds;
  const x = e.clientX - left - width / 2;
  const y = e.clientY - top - height / 2;
  const currentAngle = Math.atan2(y, x);
  const diff = currentAngle - lastAngle;
  angle += diff > PI ? diff - 2 * PI : diff < -PI ? diff + 2 * PI : diff;
  lastAngle = currentAngle;
  clock.rotate(angle); // Pass the new angle value in rad
}

window.addEventListener('mousemove', onMouseMove);
$demos.addEventListener('scroll', refreshBounds);
```

### animatable/animatable-settings/duration

`https://animejs.com/documentation/animatable/animatable-settings/duration`

> Le setting duration definit la duree de l'animation (ms) pour la transition vers les valeurs de propriete animees ; defaut 1000.

Le setting duration definit la duree de l'animation en millisecondes pour la transition vers les valeurs de propriete animees. Type Number ou Function, valeur par defaut 1000. Il accepte un Number egal ou superieur a 0, ou une valeur basee sur une fonction retournant un Number egal ou superieur a 0.

**Faits clés**

- Parametre: duration
- Type: Number ou Function
- Valeur par defaut: 1000
- Accepte: un Number >= 0
- Accepte: une valeur basee sur une fonction retournant un Number >= 0
- Duree exprimee en millisecondes

```js
import { createAnimatable, utils, stagger } from 'animejs';

const $demos = document.querySelector('#docs-demos');
const $demo = document.querySelector('.docs-demo.is-active');
let bounds = $demo.getBoundingClientRect();
const refreshBounds = () => bounds = $demo.getBoundingClientRect();

const circles = createAnimatable('.circle', {
  x: 0,
  y: stagger(200, { from: 'center', start: 200 }),
  ease: 'out(4)',
});

const onMouseMove = e => {
  const { width, height, left, top } = bounds;
  const hw = width / 2;
  const hh = height / 2;
  const x = utils.clamp(e.clientX - left - hw, -hw, hw);
  const y = utils.clamp(e.clientY - top - hh, -hh, hh);
  circles.x(x).y(y);
}

window.addEventListener('mousemove', onMouseMove);
$demos.addEventListener('scroll', refreshBounds);
```

### animatable/animatable-settings/ease

`https://animejs.com/documentation/animatable/animatable-settings/ease`

> Le setting ease determine la fonction d'easing controlant la transition vers une valeur de propriete animee ; defaut 'out(2)'.

Le setting ease (type string, valeur par defaut 'out(2)') determine quelle fonction d'easing controle la transition vers une valeur de propriete animee. Il accepte les memes valeurs d'easing que le parametre ease des animations standards. La documentation recommande d'utiliser des fonctions d'easing de type 'out' pour des resultats remarquables et interessants ; les fonctions de type 'in' produisent des changements souvent trop subtils pour etre observes. L'exemple montre deux horloges avec des easings differents : l'une en linear, l'autre en outElastic pour un mouvement plus dynamique.

**Faits clés**

- Parametre: ease
- Type: string
- Valeur par defaut: 'out(2)'
- Accepte les memes valeurs d'easing que le parametre ease des animations standards
- Recommandation: utiliser des easing de type 'out' (les 'in' sont trop subtils)

```js
import { createAnimatable } from 'animejs';

const clock1 = createAnimatable('.clock-1', {
  rotate: { unit: 'rad' },
  ease: 'linear',
});

const clock2 = createAnimatable('.clock-2', {
  rotate: { unit: 'rad' },
  ease: 'outElastic',
});
```

### animatable/animatable-settings/modifier

`https://animejs.com/documentation/animatable/animatable-settings/modifier`

> Le setting modifier definit une fonction qui modifie les valeurs numeriques animees d'une instance Animatable ; defaut noop.

Le setting modifier (type Modifier function, valeur par defaut noop, introduit en 4.0.0) definit une fonction qui modifie ou altere le comportement des valeurs numeriques animees dans une instance Animatable. L'exemple illustre deux cas d'usage : le snapping de valeurs a des intervalles (via utils.snap(PI / 10)) et l'inversion de valeurs via une fonction modifier custom (v => -v).

**Faits clés**

- Parametre: modifier
- Type: Modifier function
- Valeur par defaut: noop
- Since: 4.0.0
- Modifie/altere les valeurs numeriques animees
- Cas d'usage: snapping (utils.snap) et inversion (v => -v)

```js
import { createAnimatable, utils, stagger } from 'animejs';

const PI = Math.PI;

const clock1 = createAnimatable('.clock-1', {
  rotate: { unit: 'rad' },
  modifier: utils.snap(PI / 10),
  duration: 0,
});

const clock2 = createAnimatable('.clock-2', {
  rotate: { unit: 'rad' },
  modifier: v => -v,
  duration: 0,
});

const rotateClock = (animatable) => {
  return e => {
    const [ $clock ] = animatable.targets;
    const { width, height, left, top } = $clock.getBoundingClientRect();
    const x = e.clientX - left - width / 2;
    const y = e.clientY - top - height / 2;
    animatable.rotate(Math.atan2(y, x) + PI / 2);
  }
}

const rotateClock1 = rotateClock(clock1);
const rotateClock2 = rotateClock(clock2);

const onMouseMove = e => {
  rotateClock1(e);
  rotateClock2(e);
}

window.addEventListener('mousemove', onMouseMove);
```

### animatable/animatable-methods

`https://animejs.com/documentation/animatable/animatable-methods`

> Vue d'ensemble des methodes d'une instance Animatable retournee par createAnimatable() : getters, setters et revert().

Les methodes sont disponibles sur l'instance Animatable retournee par createAnimatable(). Trois sections de methodes sont decrites : Getters (pour recuperer les valeurs des proprietes animables), Setters (pour assigner des valeurs aux proprietes animables) et revert() (pour restaurer l'etat d'origine). Ce sont des sous-sections dans la documentation Animatable plus large, avec du contenu lie incluant les Animatable settings (unit, duration, ease, modifier) et les Animatable properties.

**Faits clés**

- Methodes sur l'instance Animatable retournee par createAnimatable()
- Trois sections: Getters, Setters, revert()
- Getters: recuperer les valeurs des proprietes animables
- Setters: assigner des valeurs aux proprietes animables
- revert(): restaurer l'etat d'origine

```js
const animatable = createAnimatable(target, parameters);
animatable.x(100)
animatable.y(50, 500, 'out(2)')
animatable.revert()
```

### animatable/animatable-methods/getters

`https://animejs.com/documentation/animatable/animatable-methods/getters`

> Les methodes de propriete appelees sans argument agissent comme des getters retournant la valeur courante animee (Number ou Array<Number>).

Les methodes sans arguments agissent comme des getters pour les proprietes animables. Chaque propriete animable definie dans les parametres de l'animatable devient une methode accessible sur l'objet animatable. Appelees sans arguments, ces methodes recuperent la valeur courante de cette propriete. Valeurs retournees : un Number pour les proprietes a valeur unique, ou un Array de Number pour les proprietes multi-valeurs (par exemple les valeurs RGB d'une couleur). Appeler une methode getter recupere la valeur animee courante de la propriete, utile pour le monitoring ou pour utiliser ces valeurs dans des callbacks et gestionnaires d'evenements.

**Faits clés**

- Methodes sans arguments = getters
- Chaque propriete animable definie devient une methode sur l'objet animatable
- Retourne un Number pour proprietes a valeur unique
- Retourne un Array de Number pour proprietes multi-valeurs (ex. RGB)
- Utile pour monitoring, callbacks et gestionnaires d'evenements

```js
import { createAnimatable, utils } from 'animejs';

const circle = createAnimatable('.circle', {
  x: 500,
  y: 500,
  ease: 'out(2)',
});

// Gets and logs current x and y values
circle.animations.x.onRender = () => {
  $x.innerHTML = utils.roundPad(circle.x(), 2);
  $y.innerHTML = utils.roundPad(circle.y(), 2);
}
```

### animatable/animatable-methods/setters

`https://animejs.com/documentation/animatable/animatable-methods/setters`

> Les methodes de propriete appelees avec des arguments agissent comme des setters chainables animant value vers la cible (signature value, duration, easing) et retournent l'objet animatable.

Les setters sont des methodes generees automatiquement a partir des proprietes animables. Appelees avec des arguments, elles agissent comme des setters chainables qui mettent a jour et animent les valeurs de propriete. Signature : animatable.property(value, duration, easing). Parametres : value (Number | Array<Number>) la valeur cible vers laquelle animer ; duration (optionnel, Number) la duree de transition en millisecondes ; easing (optionnel, ease) la fonction d'easing de l'animation. Valeur de retour : l'objet animatable lui-meme, permettant le chainage de plusieurs appels de setters de propriete (ex. animatable.x(100).y(200) anime x vers 100 et y vers 200 en 500ms).

**Faits clés**

- Signature: animatable.property(value, duration, easing)
- value: Number | Array<Number> — valeur cible
- duration: optionnel, Number — duree de transition en ms
- easing: optionnel, ease — fonction d'easing
- Retourne l'objet animatable lui-meme (chainable)
- Chainage: animatable.x(100).y(200)

```js
animatable.property(value, duration, easing);
```

```js
animatable.x(100).y(200); // Animate x to 100 and y to 200 in 500ms
```

```js
import { createAnimatable, utils } from 'animejs';

const circle = createAnimatable('.circle', {
  x: 0,
  y: 0,
  backgroundColor: 0,
  ease: 'outExpo',
});

const rgb = [164, 255, 79];

// Sets new durations and easings
circle.x(0, 500, 'out(2)');
circle.y(0, 500, 'out(3)');
circle.backgroundColor(rgb, 250);

const onMouseMove = e => {
  const { width, height, left, top } = bounds;
  const hw = width / 2;
  const hh = height / 2;
  const x = utils.clamp(e.clientX - left - hw, -hw, hw);
  const y = utils.clamp(e.clientY - top - hh, -hh, hh);
  rgb[0] = utils.mapRange(x, -hw, hw, 0, 164);
  rgb[2] = utils.mapRange(x, -hw, hw, 79, 255);
  circle.x(x).y(y).backgroundColor(rgb); // Update values
}

window.addEventListener('mousemove', onMouseMove);
```

### animatable/animatable-methods/revert

`https://animejs.com/documentation/animatable/animatable-methods/revert`

> revert() arrete et detruit completement une instance animatable en restaurant les valeurs d'origine et nettoyant les styles inline CSS ; retourne l'animatable (chainable).

La methode revert() (aucun parametre requis) arrete et detruit completement une instance animatable en restaurant toutes les proprietes animables a leurs valeurs d'origine et en nettoyant tout style inline CSS qui avait ete applique. Elle retourne l'animatable lui-meme, permettant le chainage avec d'autres methodes animatable. A utiliser quand on veut completement arreter et detruire un animatable.

**Faits clés**

- Methode: revert()
- Aucun parametre requis
- Arrete et detruit completement l'animatable
- Restaure toutes les proprietes animables a leurs valeurs d'origine
- Nettoie les styles inline CSS appliques
- Retourne l'animatable lui-meme (chainable)

```js
revert()
```

```js
import { createAnimatable, utils, stagger } from 'animejs';

const $demos = document.querySelector('#docs-demos');
const $demo = $demos.querySelector('.docs-demo.is-active');
const [ $revertButton ] = utils.$('.revert');
let bounds = $demo.getBoundingClientRect();
const refreshBounds = () => bounds = $demo.getBoundingClientRect();

const circles = createAnimatable('.circle', {
  x: stagger(50, { from: 'center', start: 100 }),
  y: stagger(200, { from: 'center', start: 200 }),
  ease: 'out(4)',
});

const onMouseMove = e => {
  const { width, height, left, top } = bounds;
  const hw = width / 2;
  const hh = height / 2;
  const x = utils.clamp(e.clientX - left - hw, -hw, hw);
  const y = utils.clamp(e.clientY - top - hh, -hh, hh);
  circles.x(x).y(y);
}

const revertAnimatable = () => {
  window.removeEventListener('mousemove', onMouseMove);
  circles.revert();
}

$revertButton.addEventListener('click', revertAnimatable);
window.addEventListener('mousemove', onMouseMove);
$demos.addEventListener('scroll', refreshBounds);
```

### animatable/animatable-properties

`https://animejs.com/documentation/animatable/animatable-properties`

> L'instance Animatable expose deux proprietes en lecture: targets (Array) et animations (Object), introduites en 4.0.0.

L'instance Animatable retournee par createAnimatable() expose deux proprietes principales : targets (Array) recupere les Targets animables, et animations (Object) recupere toutes les Animations associees. Usage : animatable.targets accede au tableau des cibles, animatable.animations accede a l'objet des animations. Cette fonctionnalite a ete introduite en version 4.0.0. Ce sont des proprietes getter offrant un acces en lecture a l'etat interne de l'instance animatable apres sa creation.

**Faits clés**

- targets: Array — recupere les Targets animables
- animations: Object — recupere toutes les Animations associees
- Proprietes getter (acces en lecture a l'etat interne)
- Introduit en version 4.0.0

```js
const animatable = createAnimatable(targets, parameters);

animatable.targets    // Access targets array
animatable.animations // Access animations object
```


## draggable

### draggable

`https://animejs.com/documentation/draggable`

> createDraggable() active le drag sur des elements DOM ; signature createDraggable(target, parameters), retourne une instance Draggable avec parametres d'axes, settings, callbacks et methodes.

La fonction createDraggable() active la fonctionnalite de glisser-deposer (drag) sur des elements DOM. Import depuis 'animejs' ou via le sous-chemin standalone 'animejs/draggable'. Signature : const draggable = createDraggable(target, parameters); ou target est un selecteur CSS ou un element DOM, et parameters (optionnel) un objet contenant axes parameters, settings et callbacks. Retourne une instance Draggable. L'objet parameters accepte trois types de configurations : (1) Axes parameters — controlent le mouvement x/y, le snapping, les modifiers et le mapping de valeurs ; (2) Settings — configurent triggers, containers, friction, velocity, cursors et thresholds ; (3) Callbacks — repondent aux evenements de drag (onGrab, onDrag, onUpdate, onRelease, onSnap, onSettle, onResize, onAfterResize). Methodes disponibles : disable() / enable(), setX() / setY(), animateInView() / scrollInView(), stop() / reset() / revert() / refresh().

**Faits clés**

- Signature: const draggable = createDraggable(target, parameters)
- target: CSS Selector ou DOM Element
- parameters: optionnel, objet (axes parameters, settings, callbacks)
- Retourne une instance Draggable
- Import standalone: 'animejs/draggable'
- Axes parameters: mouvement x/y, snapping, modifiers, mapping de valeurs
- Settings: triggers, containers, friction, velocity, cursors, thresholds
- Callbacks: onGrab, onDrag, onUpdate, onRelease, onSnap, onSettle, onResize, onAfterResize
- Methodes: disable()/enable(), setX()/setY(), animateInView()/scrollInView(), stop()/reset()/revert()/refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square');
```

```js
<div class="large row centered">
  <div class="square draggable"></div>
</div>
```

```js
const draggable = createDraggable(target, parameters);
```

### draggable/draggable-axes-parameters

`https://animejs.com/documentation/draggable/draggable-axes-parameters`

> Page d'index des parametres specifiques aux axes d'un draggable cree avec createDraggable(), applicables globalement ou par axe.

Les axes parameters configurent le comportement de dragging par axe pour les elements crees avec createDraggable(). Ils peuvent etre specifies globalement (s'appliquent aux deux axes x et y) ou individuellement par axe en passant un objet sous la cle x ou y. Ces parametres sont passes dans l'objet de configuration aux cotes des autres reglages draggable (settings) et des callbacks. La page liste cinq parametres d'axe : x, y, snap, modifier, mapTo. Chacun peut etre applique globalement ou par axe ; les signatures detaillees, valeurs par defaut et explications completes sont documentees dans les sous-pages dediees.

**Faits clés**

- 5 parametres d'axe : x, y, snap, modifier, mapTo
- Peuvent etre globaux (les deux axes) ou par axe (objet sous x ou y)
- Passes dans l'objet de config aux cotes des settings et callbacks

```js
createDraggable('.square', {
  x: { snap: 100 },
  y: { snap: 50 },
  modifier: utils.wrap(-200, 0),
  containerPadding: 10,
  releaseStiffness: 40,
  releaseEase: 'out(3)',
  onGrab: () => {},
  onDrag: () => {},
  onRelease: () => {},
});
```

### draggable/draggable-axes-parameters/x

`https://animejs.com/documentation/draggable/draggable-axes-parameters/x`

> Active, desactive ou configure le dragging sur l'axe horizontal (x).

Le parametre x controle le comportement de dragging sur l'axe horizontal. Passer true active le mouvement sur l'axe x, false le desactive entierement (empeche tout mouvement horizontal), ou bien un objet contenant les parametres d'axe detailles (snap, modifier, mapTo) permet de configurer finement cet axe.

**Faits clés**

- Type : Boolean | Draggable axes parameters Object
- Defaut : true
- x: false empeche tout mouvement horizontal
- Disponible depuis v4.0.0

```js
import { createDraggable } from 'animejs';

createDraggable('.square.enabled', {
  x: true
});

createDraggable('.square.disabled', {
  x: false
});
```

```js
<div class="large spaced-evenly row">
  <div class="square enabled draggable"></div>
  <div class="square disabled draggable"></div>
</div>
<div class="large spaced-evenly row">
  <div class="label">x enabled</div>
  <div class="label">x disabled</div>
</div>
```

### draggable/draggable-axes-parameters/y

`https://animejs.com/documentation/draggable/draggable-axes-parameters/y`

> Active, desactive ou configure le dragging sur l'axe vertical (y).

Le parametre y controle le comportement de dragging sur l'axe vertical. Passer true active le mouvement vertical, false le desactive, ou un objet contenant les parametres d'axe additionnels (snap, modifier, mapTo) configure finement cet axe.

**Faits clés**

- Type : Boolean | Draggable axes parameters Object
- Defaut : true
- y: false desactive le dragging vertical
- Objet : voir snap, modifier, mapTo

```js
import { createDraggable } from 'animejs';

createDraggable('.square.enabled', {
  y: true
});

createDraggable('.square.disabled', {
  y: false
});
```

```js
<div class="large spaced-evenly row">
  <div class="square enabled draggable"></div>
  <div class="square disabled draggable"></div>
</div>
<div class="large spaced-evenly row">
  <div class="label">y enabled</div>
  <div class="label">y disabled</div>
</div>
```

### draggable/draggable-axes-parameters/snap

`https://animejs.com/documentation/draggable/draggable-axes-parameters/snap`

> Arrondit la valeur finale d'un ou des deux axes au plus proche increment specifie, ou selectionne la valeur la plus proche dans un tableau.

Le parametre snap arrondit la valeur finale de l'un ou des deux axes au plus proche increment specifie. Lorsqu'un tableau est fourni, il selectionne la valeur la plus proche dans ce tableau au lieu d'un increment. Defini comme fonction, la valeur se rafraichit automatiquement au redimensionnement du container ou de la cible, ou manuellement via la methode refresh(). Peut etre global aux deux axes ou specifique a un axe (ex. x: { snap: ... }).

**Faits clés**

- Type : Number | Array<Number> | Function
- Defaut : 0
- Array = selectionne la valeur la plus proche du tableau
- Function : refresh auto au resize ou via refresh()
- Disponible depuis v4.0.0

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  snap: 56, // Global to both x and y
  x: { snap: [0, 200] }, // Specific to x 
});
```

```js
<div class="large grid square-grid">
  <div class="square draggable"></div>
</div>
```

```js
#draggable-draggable-axes-parameters-snap .demo {
  width: 300px;
}

#draggable-draggable-axes-parameters-snap .grid::before {
  background-size: 100px 56px;
}

#draggable-draggable-axes-parameters-snap .square {
  width: 100px;
  height: 56px;
}
```

### draggable/draggable-axes-parameters/modifier

`https://animejs.com/documentation/draggable/draggable-axes-parameters/modifier`

> Fonction modificatrice qui transforme ou contraint les valeurs d'un ou des deux axes draggable.

Le parametre modifier accepte une fonction modificatrice (ModifierFunction) qui transforme ou contraint les valeurs des axes draggable. Il peut etre applique globalement aux deux axes ou cibler un axe specifique individuellement. Le modifier global s'applique aux deux axes a moins d'etre surcharge ; les modifiers specifiques a un axe (ex. x: { modifier: ... }) ont priorite pour cet axe. Fonctionne avec des fonctions utilitaires comme utils.wrap() ainsi que des fonctions modificatrices personnalisees.

**Faits clés**

- Signature : modifier: ModifierFunction
- Defaut : noop
- Disponible depuis v4.0.0
- Modifier par axe a priorite sur le modifier global
- Compatible avec utils.wrap() et fonctions custom

```js
import { createDraggable, utils } from 'animejs';

createDraggable('.square', {
  modifier: utils.wrap(-32, 32), // Global to both x and y
  x: { modifier: utils.wrap(-128, 128) }, // Specific to x 
});
```

```js
<div class="large grid centered square-grid">
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-axes-parameters/mapto

`https://animejs.com/documentation/draggable/draggable-axes-parameters/mapto`

> Mappe la valeur d'un axe draggable vers une autre propriete animable de l'element.

Le parametre mapTo permet de mapper la valeur d'un axe draggable vers une propriete animable differente de l'element. Au lieu de deplacer l'element le long de son axe par defaut (translation 2D), le mouvement est redirige pour transformer ou modifier n'importe quelle autre propriete. Il accepte des chaines de caracteres representant des proprietes animables valides. Dans l'exemple, le drag horizontal est mappe sur le transform rotateY et le drag vertical sur la propriete z, creant un modele d'interaction 3D plutot qu'une translation 2D standard.

**Faits clés**

- Type : String
- Defaut : null
- Accepte le nom d'une propriete animable valide
- Exemple : x->rotateY, y->z pour une interaction 3D

```js
import { createDraggable, utils } from 'animejs';

utils.set('.square', { z: 100 });

createDraggable('.square', {
  x: { mapTo: 'rotateY' },
  y: { mapTo: 'z' },
});
```

```js
<div class="large grid centered perspective square-grid">
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-settings

`https://animejs.com/documentation/draggable/draggable-settings`

> Page d'index des reglages (settings) configurant la physique, la sensibilite et le comportement d'un draggable.

Les draggable settings configurent le comportement des elements crees avec createDraggable(). Ils sont passes dans un objet de configuration aux cotes des parametres d'axe et des callbacks. La documentation liste 16 reglages configurables qui modifient la physique du dragging, la sensibilite d'interaction ou le retour visuel : trigger (element declenchant le drag), container (contrainte de limites), containerPadding (espacement dans les limites), containerFriction (resistance pendant le drag dans le container), releaseContainerFriction (friction post-release), releaseMass (inertie au relachement), releaseStiffness (tension du ressort apres release), releaseDamping (amortissement des oscillations du ressort), velocityMultiplier (mise a l'echelle de la velocite), minVelocity (seuil min de velocite), maxVelocity (plafond de velocite), releaseEase (easing de l'animation post-release), dragSpeed (multiplicateur de reactivite du drag), dragThreshold (distance min avant declenchement), scrollThreshold (proximite declenchant le scroll), scrollSpeed (velocite de scroll pendant le drag).

**Faits clés**

- 16 settings : trigger, container, containerPadding, containerFriction, releaseContainerFriction, releaseMass, releaseStiffness, releaseDamping, velocityMultiplier, minVelocity, maxVelocity, releaseEase, dragSpeed, dragThreshold, scrollThreshold, scrollSpeed
- Passes dans l'objet de config aux cotes des axes parameters et callbacks

### draggable/draggable-settings/trigger

`https://animejs.com/documentation/draggable/draggable-settings/trigger`

> Designe un element alternatif servant de poignee pour declencher le drag a la place de la cible.

Le parametre trigger designe un element alternatif pour initier le comportement de drag, plutot que la cible elle-meme. Utile lorsqu'on veut qu'un element controle le dragging d'un autre. Il accepte un selecteur CSS (string) ou un element du DOM (HTMLElement). Dans l'exemple, l'element .circle sert de poignee de drag tandis que le container .row repond a l'interaction de drag.

**Faits clés**

- Signature : trigger: string | HTMLElement
- Accepte un selecteur CSS (string) ou un HTMLElement
- L'element trigger sert de poignee ; la cible est deplacee

```js
import { createDraggable } from 'animejs';

createDraggable('.row', {
  trigger: '.circle',
});
```

```js
<div class="large centered row">
  <div class="square"></div>
  <div class="circle draggable"></div>
  <div class="square"></div>
</div>
```

### draggable/draggable-settings/container

`https://animejs.com/documentation/draggable/draggable-settings/container`

> Definit les limites a l'interieur desquelles l'element draggable peut se deplacer.

Le setting container definit les limites a l'interieur desquelles un element draggable peut se deplacer, empechant l'element d'etre traine au-dela des limites specifiees. Il accepte plusieurs types : un selecteur CSS (String) ciblant un HTMLElement servant de limites de container, un HTMLElement (reference DOM directe), un tableau de nombres [top, right, bottom, left] (valeurs de padding), ou une fonction retournant un tableau [top, right, bottom, left] qui se rafraichit automatiquement au redimensionnement de la fenetre/element ou via un appel manuel a refresh().

**Faits clés**

- Type : null | String | HTMLElement | Array<Number> | Function
- Defaut : null
- Array : [top, right, bottom, left]
- Function : refresh auto au resize fenetre/element ou via refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
});

createDraggable('.circle', {
  container: [-16, 80, 16, 0],
});
```

```js
<div class="large centered grid square-grid array-container">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

```js
.grid.array-container::after {
  opacity: 1;
  top: calc(1rem);
  right: calc(1rem - 1px);
  bottom: calc(1rem - 1px);
  left: calc(10rem);
  border: 1px dotted currentColor;
  box-shadow: none;
}
```

### draggable/draggable-settings/containerpadding

`https://animejs.com/documentation/draggable/draggable-settings/containerpadding`

> Definit l'espacement (en pixels) autour des limites du container.

Le parametre containerPadding definit l'espacement autour des limites du container en pixels. Lorsque l'element draggable approche ces bords paddes, il respecte la distance definie. Types acceptes : Number (valeur unique appliquee uniformement), Array<Number> au format [top, right, bottom, left] (notation type CSS), ou Function retournant un Array<Number> au format [top, right, bottom, left] avec rafraichissement automatique au redimensionnement du container ou de la cible (ou via refresh()).

**Faits clés**

- Type : Number | Array<Number> | Function
- Defaut : 0
- Array/Function : format [top, right, bottom, left]
- Function : refresh auto au resize container/cible ou via refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  containerPadding: [ 16, 32, -16, 64], // top, right, bottom, left
  scrollThreshold: 0,
});
```

```js
<div class="large centered padded show-bounds grid square-grid">
  <div class="square draggable"></div>
</div>
```

```js
.grid.padded.show-bounds::after {
  opacity: 1;
  top: calc(1rem);
  right: calc(2rem - 1px);
  bottom: calc(-1rem - 1px);
  left: calc(4rem);
  border: 1px dashed currentColor;
  box-shadow: none;
}
```

### draggable/draggable-settings/containerfriction

`https://animejs.com/documentation/draggable/draggable-settings/containerfriction`

> Controle la resistance appliquee quand un element traine depasse les limites du container.

Le parametre containerFriction controle la resistance appliquee lorsqu'un element traine se deplace au-dela des limites de son container. Une valeur de 0 autorise un mouvement non restreint, tandis que 1 empeche completement l'element de depasser les limites. Utilise comme fonction, la valeur se met a jour automatiquement au redimensionnement du container/cible, ou peut etre rafraichie manuellement via la methode refresh().

**Faits clés**

- Type : Number (0 a 1) | Function retournant Number (0 a 1)
- Defaut : 0.8
- 0 = mouvement libre, 1 = empeche tout depassement
- Function : refresh auto au resize ou via refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  containerFriction: 0,
});

createDraggable('.circle', {
  container: '.grid',
  containerFriction: 1,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/releasecontainerfriction

`https://animejs.com/documentation/draggable/draggable-settings/releasecontainerfriction`

> Surcharge la friction appliquee a l'element lance hors limites au moment du relachement.

Le setting releaseContainerFriction surcharge la friction appliquee a un element traine quand il est projete hors des limites au moment du release. Une valeur de 0 signifie aucune friction (l'element bouge librement), tandis que 1 empeche l'element de depasser les limites du container. Valeurs acceptees : un nombre entre 0 et 1 inclus, ou une fonction retournant un nombre entre 0 et 1. Implemente comme fonction, la valeur se rafraichit automatiquement au redimensionnement du container ou de la cible, ou manuellement via refresh().

**Faits clés**

- Type : Number | Function
- Defaut : la valeur de containerFriction
- Plage : 0 a 1 inclus (0 = libre, 1 = empeche depassement)
- Function : refresh auto au resize ou via refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  releaseContainerFriction: 0,
});

createDraggable('.circle', {
  container: '.grid',
  releaseContainerFriction: 1,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/releasemass

`https://animejs.com/documentation/draggable/draggable-settings/releasemass`

> Controle la masse appliquee a l'element apres relachement, influencant vitesse, distance et rebond.

Le setting releaseMass controle la masse appliquee a un element traine apres que l'utilisateur le relache. La valeur de masse influence la rapidite de deplacement post-release, la distance parcourue et le comportement de rebond (bounciness). Diminuer la masse accelere le mouvement, l'augmenter le ralentit. Caveat important : ce parametre n'a aucun effet lorsqu'une fonction d'easing spring est passee au setting releaseEase, car la valeur de masse propre au ressort prend le dessus. Dans l'exemple, le square avec releaseMass: .1 bouge plus vite apres release, le circle avec releaseMass: 10 bouge plus lentement.

**Faits clés**

- Type : Number
- Plage : 0 a 10000
- Defaut : 1
- Masse basse = mouvement plus rapide ; masse haute = plus lent
- Gotcha : sans effet si releaseEase est un spring (la masse du spring prime)

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  releaseMass: .1,
});

createDraggable('.circle', {
  container: '.grid',
  releaseMass: 10,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/releasestiffness

`https://animejs.com/documentation/draggable/draggable-settings/releasestiffness`

> Parametre Draggable controlant la rigidite du mouvement de l'element relache apres un drag.

releaseStiffness est un Number (defaut 80, plage 0 a 10000) qui controle la rigidite (stiffness) du mouvement de l'element draggable apres relachement. Des valeurs basses produisent un mouvement plus lent et progressif, des valeurs hautes un mouvement plus vif/rebondissant. Ce reglage est ecrase si une easing spring est passee a releaseEase, car celle-ci utilise sa propre configuration de stiffness interne.

**Faits clés**

- Type: Number
- Defaut: 80
- Plage: 0 a 10000
- Valeurs basses = mouvement plus lent/progressif ; valeurs hautes = plus vif
- Ecrase si une spring easing est passee a releaseEase (la spring utilise sa propre stiffness)

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  releaseStiffness: 20,
});

createDraggable('.circle', {
  container: '.grid',
  releaseStiffness: 300,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/releasedamping

`https://animejs.com/documentation/draggable/draggable-settings/releasedamping`

> Parametre Draggable controlant l'amortissement (damping) applique a l'element relache apres un drag.

releaseDamping est un Number (defaut 10, plage 0 a 10000) qui controle l'amortissement (dampening) applique aux elements relaches apres un drag, affectant la velocite, la distance parcourue et le comportement de rebond. Des valeurs basses augmentent le rebond au contact des limites du conteneur. Ce parametre n'a aucun effet quand une spring easing est fournie via releaseEase, car la valeur de damping de la spring a la priorite.

**Faits clés**

- Type: Number
- Defaut: 10
- Plage: 0 a 10000
- Affecte velocite, distance parcourue et rebond
- Valeurs basses = plus de rebond au contact des limites du conteneur
- Sans effet si une spring easing est fournie via releaseEase (le damping de la spring prime)

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  releaseDamping: 5,
});

createDraggable('.circle', {
  container: '.grid',
  releaseStiffness: 30,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/velocitymultiplier

`https://animejs.com/documentation/draggable/draggable-settings/velocitymultiplier`

> Parametre Draggable multipliant la velocite appliquee a l'element relache apres un drag.

velocityMultiplier est un Number >= 0 ou une Function retournant un Number >= 0 (defaut 1). Il modifie la velocite appliquee a l'element draggable apres relachement : 0 elimine toute velocite, 1 represente la velocite normale, 2 double la velocite. Les valeurs basees sur une fonction se rafraichissent automatiquement au redimensionnement du conteneur/cible et peuvent etre rafraichies manuellement via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 1
- Valeur minimale: 0
- 0 = pas de velocite, 1 = velocite normale, 2 = velocite doublee
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  velocityMultiplier: 0,
});

createDraggable('.circle', {
  container: '.grid',
  velocityMultiplier: 5,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/minvelocity

`https://animejs.com/documentation/draggable/draggable-settings/minvelocity`

> Parametre Draggable definissant la velocite minimale appliquee a l'element relache apres un drag.

minVelocity est un Number >= 0 ou une Function retournant un Number >= 0 (defaut 0). Il specifie la velocite minimale a appliquer a l'element draggable apres relachement. Avec une valeur basee sur une fonction, elle se rafraichit automatiquement au redimensionnement du conteneur/cible et peut etre rafraichie manuellement via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 0
- Valeur minimale: 0
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  minVelocity: 0,
});

createDraggable('.circle', {
  container: '.grid',
  minVelocity: 10,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/maxvelocity

`https://animejs.com/documentation/draggable/draggable-settings/maxvelocity`

> Parametre Draggable definissant la limite superieure de velocite appliquee a l'element relache apres un drag.

maxVelocity est un Number >= 0 ou une Function retournant un Number >= 0 (defaut 50). Il definit la limite superieure de la velocite appliquee aux elements draggable lors du relachement. Quand une fonction est utilisee, la valeur se rafraichit automatiquement au redimensionnement du conteneur/cible ou via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 50
- Valeur minimale: 0
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  maxVelocity: 0,
});

createDraggable('.circle', {
  container: '.grid',
  maxVelocity: 100,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/releaseease

`https://animejs.com/documentation/draggable/draggable-settings/releaseease`

> Parametre Draggable definissant l'easing applique a l'element apres relachement, snap ou repositionnement hors-limites.

releaseEase definit un easing personnalise (type ease, defaut eases.outQuint, depuis 4.0.0) applique a l'element draggable apres relachement, un evenement snap, ou un repositionnement quand l'element est tire hors des limites. Quand on utilise spring() comme fonction d'easing, celle-ci ecrase les parametres releaseMass, releaseStiffness et releaseDamping. Le parametre velocity de la fonction spring est ignore et remplace par la velocite reelle de l'element draggable.

**Faits clés**

- Type: ease (fonction d'easing)
- Defaut: eases.outQuint
- Depuis: 4.0.0
- Applique apres relachement / snap / repositionnement hors-limites
- spring() ecrase releaseMass, releaseStiffness et releaseDamping
- Le parametre velocity de spring est ignore et remplace par la velocite reelle de l'element

```js
import { createDraggable, spring } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  releaseEase: 'outElastic',
});

createDraggable('.circle', {
  container: '.grid',
  releaseEase: spring({
    stiffness: 150,
    damping: 15,
  })
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/dragspeed

`https://animejs.com/documentation/draggable/draggable-settings/dragspeed`

> Parametre Draggable controlant la vitesse de glissement de l'element.

dragSpeed est un Number ou une Function (defaut 1) qui controle la vitesse de glissement (dragging) d'un element. Des valeurs plus elevees augmentent la vitesse de deplacement ; 0 empeche le drag ; des valeurs negatives inversent la direction du drag. Implementee en fonction, la valeur se rafraichit automatiquement au redimensionnement du conteneur/cible et peut etre mise a jour manuellement via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 1
- 0 empeche le drag
- Valeurs negatives inversent la direction du drag
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  dragSpeed: 2,
});

createDraggable('.circle', {
  container: '.grid',
  dragSpeed: .5,
});
```

```js
<div class="large centered grid square-grid">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-settings/dragthreshold

`https://animejs.com/documentation/draggable/draggable-settings/dragthreshold`

> Parametre Draggable definissant la distance en pixels necessaire pour declencher un drag.

dragThreshold est un Number, ou un objet { mouse: Number, touch: Number }, ou une Function (defaut { mouse: 3, touch: 7 }). Il etablit la distance en pixels necessaire pour declencher un drag. On peut appliquer un seuil uniforme (un seul nombre) ou differencier selon la methode d'entree via un objet avec proprietes mouse et touch. Implementee en fonction, la valeur se rafraichit automatiquement au redimensionnement du conteneur ou de la cible, ou manuellement via la methode refresh(). Disponible depuis la version 4.2.1.

**Faits clés**

- Type: Number | { mouse: Number, touch: Number } | Function
- Defaut: { mouse: 3, touch: 7 }
- Distance en pixels pour declencher un drag
- Seuil uniforme (nombre) ou specifique par input (objet mouse/touch)
- Depuis: 4.2.1
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
dragThreshold: 20
```

```js
dragThreshold: { mouse: 10, touch: 15 }
```

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.grid',
  dragThreshold: 20,
});

createDraggable('.circle', {
  container: '.grid',
  dragThreshold: { mouse: 10, touch: 15 },
});
```

### draggable/draggable-settings/scrollthreshold

`https://animejs.com/documentation/draggable/draggable-settings/scrollthreshold`

> Parametre Draggable definissant la distance en pixels au-dela des limites du conteneur avant activation du scroll automatique.

scrollThreshold est un Number ou une Function (defaut 20) qui determine combien de pixels l'element draggable doit parcourir au-dela des limites du conteneur avant que le defilement automatique (auto-scroll) ne s'active. Implementee en fonction, la valeur se rafraichit automatiquement au redimensionnement du conteneur ou de la cible, ou manuellement via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 20
- Pixels au-dela des limites du conteneur avant activation de l'auto-scroll
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.scroll-container',
  scrollThreshold: 12,
});
```

```js
<div class="full-container scroll-container scroll-x scroll-y">
  <div class="scroll-content">
    <div class="large padded grid square-grid">
      <div class="square draggable"></div>
    </div>
  </div>
</div>
```

```js
#draggable-draggable-settings-scrollthreshold .draggable {
  background-color: rgba(var(--rgb-current), .25);
  border: solid 1px currentColor;
}

#draggable-draggable-settings-scrollthreshold .draggable::after {
  content: "";
  display: block;
  position: absolute;
  top: 12px;
  left: 12px;
  right: 12px;
  bottom: 12px;
  background-color: currentColor;
  border-radius: 2px;
}
```

### draggable/draggable-settings/scrollspeed

`https://animejs.com/documentation/draggable/draggable-settings/scrollspeed`

> Parametre Draggable controlant la vitesse de defilement automatique du conteneur pendant un drag.

scrollSpeed est un Number ou une Function retournant un Number (defaut 1.5, depuis 4.0.0). Il controle la velocite du defilement automatique du conteneur pendant les operations de drag. Des valeurs plus elevees augmentent la vitesse de scroll ; 0 desactive entierement le defilement du conteneur. Implementee en fonction, la valeur se rafraichit automatiquement au redimensionnement du conteneur ou de l'element cible, ou peut etre mise a jour manuellement via la methode refresh().

**Faits clés**

- Type: Number | Function
- Defaut: 1.5
- Depuis: 4.0.0
- 0 desactive entierement le scroll du conteneur
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  container: '.scroll-container',
  scrollSpeed: 2,
});
```

```js
<div class="full-container scroll-container scroll-x scroll-y">
  <div class="scroll-content">
    <div class="large padded grid square-grid">
      <div class="square draggable"></div>
    </div>
  </div>
</div>
```

### draggable/draggable-settings/cursor

`https://animejs.com/documentation/draggable/draggable-settings/cursor`

> Parametre Draggable personnalisant le style de curseur CSS pour les etats hover et grab.

cursor est un Boolean, ou un objet { onHover: string, onGrab: string }, ou une Function (defaut { onHover: 'grab', onGrab: 'grabbing' }). Il personnalise le style de curseur CSS pour les etats hover et grabbed sur les appareils correspondant a '(pointer:fine)'. Mettre false desactive entierement le style de curseur personnalise. Il ne s'applique que sur les appareils a pointeur fin (exclut le tactile). Implemente en fonction, la valeur se rafraichit automatiquement au redimensionnement du conteneur/cible et peut etre mise a jour manuellement via la methode refresh().

**Faits clés**

- Type: Boolean | { onHover: string, onGrab: string } | Function
- Defaut: { onHover: 'grab', onGrab: 'grabbing' }
- false desactive le style de curseur personnalise
- Ne s'applique que sur appareils a pointeur fin '(pointer:fine)' (exclut le tactile)
- Valeurs fonction: refresh auto sur resize + via methode refresh()

```js
import { createDraggable } from 'animejs';

createDraggable('.square', {
  cursor: false
});
```

```js
createDraggable('.circle', {
  cursor: {
    onHover: 'move',
    onGrab: 'wait'
  }
});
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
  <div class="circle draggable"></div>
</div>
```

### draggable/draggable-callbacks

`https://animejs.com/documentation/draggable/draggable-callbacks`

> Page d'introduction listant les callbacks Draggable executes a des moments precis du cycle de drag.

Les Draggable callbacks executent des fonctions a des points specifiques pendant le drag d'un element. Les fonctions de callback sont specifiees directement dans les parametres Object de createDraggable(). Elles repondent chacune a une phase d'interaction precise, du grab initial au settlement final. Les huit callbacks disponibles sont: onGrab, onDrag, onUpdate, onRelease, onSnap, onSettle, onResize, onAfterResize. Disponible depuis 4.0.0. Des pages individuelles documentent chaque callback en detail.

**Faits clés**

- Callbacks definis directement dans l'Object de parametres de createDraggable()
- 8 callbacks: onGrab, onDrag, onUpdate, onRelease, onSnap, onSettle, onResize, onAfterResize
- Depuis: 4.0.0

```js
createDraggable('.square', {
  x: { snap: 100 },
  y: { snap: 50 },
  modifier: utils.wrap(-200, 0),
  containerPadding: 10,
  containerStiffness: 40,
  containerEase: 'out(3)',
  onGrab: () => {},
  onDrag: () => {},
  onRelease: () => {},
});
```

### draggable/draggable-callbacks/ongrab

`https://animejs.com/documentation/draggable/draggable-callbacks/ongrab`

> Callback Draggable execute quand un utilisateur saisit (grab) un element draggable.

onGrab est un callback de signature (draggable) => void (defaut noop) execute quand un utilisateur saisit (grab / initie l'interaction avec) un element draggable. Il recoit l'instance draggable comme premier argument. Disponible depuis 4.0.0.

**Faits clés**

- Signature: onGrab: (draggable) => void
- Defaut: noop
- Recoit l'instance draggable comme premier argument
- Execute au grab (initiation de l'interaction)
- Depuis: 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let grabs = 0;

createDraggable('.square', {
  container: '.grid',
  onGrab: () => $value.textContent = ++grabs
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">grabs</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/ondrag

`https://animejs.com/documentation/draggable/draggable-callbacks/ondrag`

> Callback de draggable execute pendant que l'element est traine.

onDrag est un callback de createDraggable execute lorsque l'element est en cours de glissement (drag). Il se declenche de maniere repetee tant que l'utilisateur traine activement l'element, distinct de onGrab (debut du drag) et onRelease (fin du drag). Le callback recoit l'instance draggable comme premier argument. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onDrag
- Type: Function
- Default: noop
- Le callback recoit l'instance draggable comme premier argument
- Disponible depuis 4.0.0
- Se declenche en continu pendant le drag (distinct de onGrab/onRelease)

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let drags = 0;

createDraggable('.square', {
  container: '.grid',
  onDrag: () => $value.textContent = ++drags
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">drags</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/onupdate

`https://animejs.com/documentation/draggable/draggable-callbacks/onupdate`

> Callback de draggable execute chaque fois que la position de l'element traine change.

onUpdate est un callback de createDraggable execute chaque fois que la position de l'element traine change. Il se declenche en continu pendant le drag, permettant de suivre ou de reagir aux mises a jour de position en temps reel. Le callback recoit l'instance draggable comme premier argument. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onUpdate
- Type: Function
- Default: noop
- Le callback recoit l'instance draggable comme premier argument
- Disponible depuis 4.0.0
- Se declenche chaque fois que la position change

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let updates = 0;

createDraggable('.square', {
  container: '.grid',
  onUpdate: () => $value.textContent = ++updates
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">updates</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/onrelease

`https://animejs.com/documentation/draggable/draggable-callbacks/onrelease`

> Callback de draggable execute quand l'utilisateur relache l'element apres l'avoir traine.

onRelease declenche une fonction lorsque l'utilisateur relache l'element apres l'avoir traine. Il fait partie du cycle d'interaction grab/drag/release et se declenche immediatement au relachement. Le callback recoit l'instance draggable comme premier parametre. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onRelease
- Type: Function (signature: (draggable) => void)
- Default: noop
- Le callback recoit l'instance draggable comme premier parametre
- Disponible depuis 4.0.0
- Suit le cycle d'interaction grab/drag/release

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let releases = 0;

createDraggable('.square', {
  container: '.grid',
  onRelease: () => $value.textContent = ++releases
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">releases</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/onsnap

`https://animejs.com/documentation/draggable/draggable-callbacks/onsnap`

> Callback de draggable execute lorsqu'un evenement de snap survient pendant le drag.

onSnap est un callback execute chaque fois qu'un evenement de snap survient durant le glissement de l'element. Il recoit l'instance draggable comme premier argument. L'exemple utilise l'option snap pour definir l'increment de snap et modifier: utils.snap(16) pour aussi faire snapper l'element pendant le drag. Callbacks lies: precedent onRelease, suivant onSettle. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onSnap
- Type: Function
- Default: noop
- Le callback recoit l'instance draggable comme premier argument
- Disponible depuis 4.0.0
- S'utilise avec l'option snap (ex: snap: 16) et modifier: utils.snap(16)
- Callbacks lies: onRelease (precedent), onSettle (suivant)

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let snaps = 0;

createDraggable('.square', {
  container: '.grid',
  snap: 16,
  modifier: utils.snap(16), // also snap the element while draggin
  onSnap: () => $value.textContent = ++snaps
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">snaps</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/onsettle

`https://animejs.com/documentation/draggable/draggable-callbacks/onsettle`

> Callback de draggable execute quand la cible traine s'arrete completement de bouger apres relachement.

onSettle est un callback execute lorsque la cible traine s'arrete completement de bouger apres avoir ete relachee. Il recoit l'instance draggable comme premier argument. A distinguer de onRelease (qui se declenche immediatement au relachement) : onSettle attend que tout mouvement (inertie/snap) soit termine. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onSettle
- Type: Function
- Default: noop
- Le callback recoit l'instance draggable comme premier argument
- Disponible depuis 4.0.0
- Se declenche quand la cible s'arrete completement (apres inertie/snap), distinct de onRelease (immediat)

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let stops = 0;

createDraggable('.square', {
  container: '.grid',
  onSettle: () => $value.textContent = ++stops
});
```

```js
<div class="large padded grid square-grid">
  <pre class="large log row">
    <span class="label">stops</span>
    <span class="value">0</span>
  </pre>
  <div class="square draggable"></div>
</div>
```

### draggable/draggable-callbacks/onresize

`https://animejs.com/documentation/draggable/draggable-callbacks/onresize`

> Callback de draggable execute quand le container ou la cible traine change de taille.

onResize est un callback execute chaque fois que le container ou l'element cible traine change de taille. La fonction recoit l'instance draggable comme premier parametre, permettant de reagir aux changements de dimensions. Callback lie: onAfterResize s'execute apres la fin du resize. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onResize
- Type: Function (signature: (self: Draggable) => void)
- Default: noop
- Le callback recoit l'instance draggable (self) comme premier parametre
- Disponible depuis 4.0.0
- Se declenche quand le container ou la cible change de taille
- Callback lie: onAfterResize (apres completion du resize)

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let resizes = 0;

createDraggable('.square', {
  container: '.grid',
  onResize: self => {
    $value.textContent = ++resizes;
  }
});
```

```js
<div class="iframe-content resizable">
  <div class="large padded grid square-grid">
    <pre class="large log row">
      <span class="label">resizes</span>
      <span class="value">0</span>
    </pre>
    <div class="square draggable"></div>
  </div>
</div>
```

### draggable/draggable-callbacks/onafterresize

`https://animejs.com/documentation/draggable/draggable-callbacks/onafterresize`

> Callback de draggable execute apres qu'un resize du container ou de la cible soit termine et les valeurs draggable mises a jour.

onAfterResize execute une fonction apres que le container ou la cible traine ait change de taille ET que les valeurs draggable aient ete mises a jour. Le callback recoit l'instance draggable (self) comme premier argument. Utile pour repositionner les elements lors de changements de dimensions du viewport. L'exemple appelle self.animateInView(1000, 30) dans le callback. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: onAfterResize
- Type: Function
- Default: noop
- Le callback recoit l'instance draggable (self) comme premier argument
- Disponible depuis 4.0.0
- Se declenche apres que le resize soit complete ET les valeurs draggable mises a jour (distinct de onResize)
- Utile pour repositionner via self.animateInView()

```js
import { createDraggable, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let resizes = 0;

const draggable = createDraggable('.square', {
  container: '.grid',
  onAfterResize: self => {
    $value.textContent = ++resizes;
    self.animateInView(1000, 30);
  }
});
```

```js
<div class="iframe-content resizable">
  <div class="large padded grid square-grid">
    <pre class="large log row">
      <span class="label">resizes</span>
      <span class="value">0</span>
    </pre>
    <div class="square draggable"></div>
  </div>
</div>
```

### draggable/draggable-methods

`https://animejs.com/documentation/draggable/draggable-methods`

> Page d'index listant les methodes disponibles sur une instance draggable.

Page d'index/apercu des methodes Draggable disponibles sur les instances retournees par createDraggable(). Elle liste les noms de methodes sous forme de liens vers leurs pages individuelles, sans signatures detaillees ni exemples sur la page d'index elle-meme. Methodes listees: disable(), enable(), setX(), setY(), animateInView(), scrollInView(), stop(), reset(), revert(), refresh().

**Faits clés**

- Page d'index (overview) des methodes Draggable
- Methodes listees: disable(), enable(), setX(), setY(), animateInView(), scrollInView(), stop(), reset(), revert(), refresh()
- Pas de signatures detaillees ni d'exemples sur la page d'index (voir pages individuelles)

### draggable/draggable-methods/disable

`https://animejs.com/documentation/draggable/draggable-methods/disable`

> Methode qui desactive une instance draggable, la rendant inerte et non-interactive.

La methode disable() desactive une instance draggable, la rendant inerte et non-interactive. Elle retourne l'objet draggable lui-meme pour permettre le chainage de methodes. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: disable(): Draggable
- Aucun parametre
- Retourne l'instance draggable elle-meme (chainage)
- Rend l'element inerte/non-interactif
- Disponible depuis 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $disableButton ] = utils.$('.disable');

const draggable = createDraggable('.square');

const disableDraggable = () => draggable.disable();

$disableButton.addEventListener('click', disableDraggable);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button disable">Disable</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/enable

`https://animejs.com/documentation/draggable/draggable-methods/enable`

> Methode qui reactive un element draggable precedemment desactive.

La methode enable() reactive un element draggable precedemment desactive, restaurant sa fonctionnalite interactive. Elle retourne l'instance draggable elle-meme pour permettre le chainage de methodes. Complemente disable() pour basculer l'etat draggable. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: enable(): Draggable
- Aucun parametre
- Retourne l'instance draggable elle-meme (chainage)
- Reactive un draggable precedemment desactive
- Complemente disable()
- Disponible depuis 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $enableButton ] = utils.$('.enable');

const draggable = createDraggable('.square');

draggable.disable();

const enableDraggable = () => draggable.enable();

$enableButton.addEventListener('click', enableDraggable);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button enable">Enable</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/setx

`https://animejs.com/documentation/draggable/draggable-methods/setx`

> Methode qui definit manuellement la position horizontale x de la cible draggable.

La methode setX() definit programmatiquement la position horizontale de l'element cible draggable. Elle est equivalente a la mise a jour directe de la propriete draggable.x lorsque la suppression de callback n'est pas specifiee. Parametres: x (Number) la nouvelle coordonnee horizontale; muteCallback (Boolean, optionnel, defaut false) qui empeche le callback onUpdate de se declencher quand true. Retourne l'instance draggable pour le chainage.

**Faits clés**

- Signature: setX(x: Number, muteCallback?: Boolean): Draggable
- Parametre x: Number (nouvelle coordonnee horizontale)
- Parametre muteCallback: Boolean optionnel, defaut false (empeche onUpdate si true)
- Equivalent a draggable.x sans suppression de callback
- Retourne l'instance draggable (chainage)

```js
import { createDraggable, utils } from 'animejs';

const [ $setButton ] = utils.$('.set');

const draggable = createDraggable('.square');

const setRandomX = () => draggable.setX(utils.random(-100, 100));

$setButton.addEventListener('click', setRandomX);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button set">Set random x</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/sety

`https://animejs.com/documentation/draggable/draggable-methods/sety`

> Methode qui definit manuellement la position verticale y de la cible draggable.

La methode setY() met a jour manuellement la position verticale de l'element cible draggable. Elle est fonctionnellement equivalente a l'assignation directe de draggable.y lorsque le parametre optionnel de callback n'est pas specifie. Parametres: y (Number) la nouvelle valeur verticale; muteCallback (Boolean, optionnel, defaut false) qui supprime le declenchement du callback onUpdate quand true. Retourne l'instance draggable pour le chainage. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: setY(y: Number, muteCallback?: Boolean): Draggable
- Parametre y: Number (nouvelle valeur verticale)
- Parametre muteCallback: Boolean optionnel, defaut false (supprime onUpdate si true)
- Equivalent a draggable.y sans suppression de callback
- Retourne l'instance draggable (chainage)
- Disponible depuis 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $setButton ] = utils.$('.set');

const draggable = createDraggable('.square');

const setRandomY = () => draggable.setY(utils.random(-40, 40));

$setButton.addEventListener('click', setRandomY);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button set">Set random y</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/animateinview

`https://animejs.com/documentation/draggable/draggable-methods/animateinview`

> Methode qui anime un element draggable dans la vue lorsqu'il est positionne hors des limites du container.

La methode animateInView() anime un element draggable dans la vue lorsqu'il est positionne hors des limites du container. Elle calcule automatiquement la position correcte et transitionne en douceur le draggable dans le viewport. Parametres optionnels: duration (Number, defaut 350) la duree d'animation en millisecondes; gap (Boolean) distance supplementaire depuis les bords du container (en pixels); ease (ease, defaut InOutQuad) la fonction d'easing appliquee. Retourne l'instance draggable elle-meme.

**Faits clés**

- Signature: animateInView(duration?, gap?, ease?): Draggable
- Parametre duration: Number optionnel, defaut 350 (ms)
- Parametre gap: Boolean optionnel (distance supplementaire des bords en pixels)
- Parametre ease: ease optionnel, defaut InOutQuad
- Retourne l'instance draggable elle-meme
- Anime l'element dans la vue s'il est hors des limites du container

```js
import { createDraggable, utils } from 'animejs';

const [ $animateInView ] = utils.$('.animate-button');

const draggable = createDraggable('.square', {
  container: '.grid',
});

const animateInView = () => {
  draggable.animateInView(400, 16);
}

// Set the draggable position outside the container
draggable.x = -24;
draggable.y = 72;

$animateInView.addEventListener('click', animateInView);
```

### draggable/draggable-methods/scrollinview

`https://animejs.com/documentation/draggable/draggable-methods/scrollinview`

> Methode du Draggable qui declenche une animation de scroll du conteneur pour ramener le draggable dans la zone visible quand sa position depasse les bornes du seuil de scroll.

scrollInView(duration?, gap?, ease?) declenche une animation de scroll du conteneur lorsque la position de l'element draggable sort des bornes du seuil de scroll etabli. Elle repositionne automatiquement le viewport pour ramener le draggable dans la vue. Parametres : duration (Number, optionnel) = duree de l'animation en millisecondes, defaut 350 ; gap (Boolean, optionnel) = distance supplementaire depuis les bords du conteneur vers laquelle le draggable s'anime ; ease (ease, optionnel) = fonction d'easing de l'animation, defaut InOutQuad. Retourne l'instance draggable (supporte le chainage de methodes).

**Faits clés**

- Signature: scrollInView(duration?, gap?, ease?)
- duration (Number, optionnel): duree en ms, defaut 350
- gap (Boolean, optionnel): distance supplementaire depuis les bords du conteneur
- ease (ease, optionnel): fonction d'easing, defaut InOutQuad
- Retourne l'instance draggable (chainable)
- Disponible depuis la version 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $scrollInView ] = utils.$('.button');

const draggable = createDraggable('.square', {
  container: '.scroll-container',
});

const scrollInView = () => {
  draggable.scrollInView(400, 100);
}

// Set the draggable position outside the scroll viewport
draggable.x = 120;
draggable.y = 200;

$scrollInView.addEventListener('click', scrollInView);
```

```js
<div class="full-container scroll-container scroll-x scroll-y">
  <div class="scroll-content">
    <div class="large padded grid square-grid">
      <div class="square draggable"></div>
    </div>
  </div>
</div>
<fieldset class="absolute controls">
  <button class="button">Scroll in view</button>
</fieldset>
```

### draggable/draggable-methods/stop

`https://animejs.com/documentation/draggable/draggable-methods/stop`

> Methode du Draggable qui arrete toutes les animations en cours ciblant le draggable, le scroll du conteneur et l'animation de release.

stop() arrete toutes les animations en cours ciblant le draggable, l'animation de scroll du conteneur et l'animation de release du draggable. Cela inclut les animations de mouvement, le scroll du conteneur et les animations d'inertie au relachement. Retourne l'instance draggable elle-meme, permettant le chainage de methodes.

**Faits clés**

- Signature: stop()
- Arrete toutes les animations du draggable + scroll conteneur + animation de release
- Retourne l'instance draggable (chainable)
- Disponible depuis la version 4.0.0

```js
import { createDraggable, animate, utils } from 'animejs';

const [ $stopButton ] = utils.$('.stop');

const draggable = createDraggable('.square');

animate(draggable, {
  x: [-100, 100],
  alternate: true,
  loop: true
});

const stopDraggable = () => draggable.stop();

$stopButton.addEventListener('click', stopDraggable);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button stop">Stop</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/reset

`https://animejs.com/documentation/draggable/draggable-methods/reset`

> Methode du Draggable qui restaure l'element draggable a sa position initiale.

reset() restaure l'element draggable a sa position initiale. Retourne l'instance draggable elle-meme, permettant le chainage avec d'autres methodes du draggable.

**Faits clés**

- Signature: reset()
- Restaure l'element draggable a sa position initiale
- Retourne l'instance draggable (chainable)
- Aucun parametre requis

```js
import { createDraggable, utils } from 'animejs';

const [ $resetButton ] = utils.$('.reset');

const draggable = createDraggable('.square');

const resetDraggable = () => draggable.reset();

$resetButton.addEventListener('click', resetDraggable);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button reset">Reset</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/revert

`https://animejs.com/documentation/draggable/draggable-methods/revert`

> Methode du Draggable qui restaure l'element a son etat initial et le desactive.

revert() restaure l'element draggable a son etat initial et le desactive. Cette methode annule toutes les transformations de drag et ramene l'element a son etat d'avant le drag. Retourne l'instance draggable elle-meme (chainable). Aucun parametre requis.

**Faits clés**

- Signature: revert()
- Restaure l'element a son etat initial ET desactive le draggable
- Retourne l'instance draggable (chainable)
- Aucun parametre requis
- Disponible depuis la version 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $revertButton ] = utils.$('.revert');

const draggable = createDraggable('.square');

function revertDraggable() {
  draggable.revert();
  $revertButton.disabled = true;
}

$revertButton.addEventListener('click', revertDraggable);
```

```js
<div class="large centered row">
  <div class="square draggable"></div>
</div>
<div class="large row">
  <fieldset class="controls">
    <button class="button revert">Revert</button>
  </fieldset>
</div>
```

### draggable/draggable-methods/refresh

`https://animejs.com/documentation/draggable/draggable-methods/refresh`

> Methode du Draggable qui re-calcule tous les parametres definis via une fonction et recalcule toutes les valeurs internes.

refresh() re-calcule chaque parametre defini en utilisant une fonction et recalcule toutes les valeurs internes du draggable, mettant a jour l'etat interne de l'instance. Les parametres rafraichissables sont : snap, container, containerPadding, containerFriction, dragSpeed, scrollSpeed, scrollThreshold, minVelocity, maxVelocity, velocityMultiplier. Retourne l'instance draggable elle-meme, permettant le chainage de methodes.

**Faits clés**

- Signature: refresh(): Draggable
- Recalcule les parametres definis comme fonctions et les valeurs internes
- Parametres rafraichissables: snap, container, containerPadding, containerFriction, dragSpeed, scrollSpeed, scrollThreshold, minVelocity, maxVelocity, velocityMultiplier
- Retourne l'instance draggable (chainable)
- Disponible depuis la version 4.0.0

```js
import { createDraggable, utils } from 'animejs';

const [ $refreshButton ] = utils.$('.refresh');

const draggable = createDraggable('.square', {
  snap: () => utils.random(0, 32, 0),
  dragSpeed: () => utils.random(.5, 1.5, 1),
});

const refreshDraggable = () => draggable.refresh();

$refreshButton.addEventListener('click', refreshDraggable);
```

### draggable/draggable-properties

`https://animejs.com/documentation/draggable/draggable-properties`

> Liste complete des proprietes exposees par l'instance Draggable retournee par createDraggable(target, parameters), pour lire et modifier le comportement du draggable.

L'instance Draggable retournee par createDraggable(target, parameters) expose de nombreuses proprietes pour lire et definir le comportement du draggable (toutes Since 4.0.0). Proprietes en lecture/ecriture : snapX (Number|Array<Number>), snapY (Number|Array<Number>), scrollSpeed (Number), scrollThreshold (Number), dragSpeed (Number), maxVelocity (Number), minVelocity (Number), velocityMultiplier (Number), releaseEase (Function), containerPadding (Array<Number> [top,right,bottom,left]), containerFriction (Number), $container (HTMLElement), $target (HTMLElement), x (Number), y (Number), progressX (Number 0-1), progressY (Number 0-1), cursor (Boolean|DraggableCursorParams), isFinePointer (Boolean), et les callbacks onGrab, onDrag, onRelease, onUpdate, onSettle, onSnap, onResize, onAfterResize (Function). Proprietes en lecture seule : releaseSpring (Spring), containerBounds (Array<Number>), containerArray (Array<HTMLElement>|null), $trigger (HTMLElement), $scrollContainer (Window|HTMLElement), velocity (Number), angle (Number radians), xProp (String), yProp (String), destX (Number), destY (Number), deltaX (Number), deltaY (Number), enabled (Boolean), grabbed (Boolean), dragged (Boolean), disabled (Array<Number>), fixed (Boolean), useWin (Boolean), initialized (Boolean), canScroll (Boolean), contained (Boolean), manual (Boolean), released (Boolean), updated (Boolean), scroll (Object {x,y}), coords (Array<Number> [x,y,prevX,prevY]), snapped (Array<Number>), pointer (Array<Number> [x,y,prevX,prevY]), scrollView (Array<Number> [width,height]), dragArea (Array<Number> [x,y,width,height]), scrollBounds (Array<Number> [top,right,bottom,left]), targetBounds (Array<Number>), window (Array<Number> [width,height]), pointerVelocity (Number), pointerAngle (Number radians), activeProp (String). Les proprietes en lecture seule ne peuvent pas etre assignees ; toute tentative d'assignation n'affectera pas le comportement.

**Faits clés**

- snapX / snapY: Number|Array<Number> — valeur de snap par axe (R/W)
- scrollSpeed: Number — vitesse d'auto-scroll (R/W)
- scrollThreshold: Number — distance seuil pour auto-scroll (R/W)
- dragSpeed: Number (R/W)
- maxVelocity / minVelocity / velocityMultiplier: Number (R/W)
- releaseEase: Function — easing des animations (R/W)
- releaseSpring: Spring — objet spring interne (lecture seule)
- containerPadding: Array<Number> [top,right,bottom,left] (R/W)
- containerFriction: Number (R/W)
- containerBounds: Array<Number> (lecture seule)
- containerArray: Array<HTMLElement>|null — conteneurs multiples
- $container: HTMLElement (R/W) ; $target: HTMLElement (R/W) ; $trigger: HTMLElement (lecture seule) ; $scrollContainer: Window|HTMLElement (lecture seule)
- x / y: Number — position (R/W)
- progressX / progressY: Number (0-1) (R/W)
- velocity: Number (lecture seule) ; angle: Number radians (lecture seule)
- xProp / yProp: String — nom de propriete mappee (lecture seule)
- destX / destY: Number (lecture seule) ; deltaX / deltaY: Number (lecture seule)
- enabled / grabbed / dragged: Boolean (lecture seule)
- cursor: Boolean|DraggableCursorParams (R/W)
- disabled: Array<Number> par axe (lecture seule)
- fixed / useWin / initialized / canScroll / contained / manual / released / updated: Boolean (lecture seule)
- isFinePointer: Boolean — detection pointeur fin (R/W)
- scroll: Object {x,y} (lecture seule)
- coords: Array<Number> [x,y,prevX,prevY] (lecture seule)
- snapped: Array<Number> par axe (lecture seule)
- pointer: Array<Number> [x,y,prevX,prevY] (lecture seule)
- scrollView: Array<Number> [width,height] (lecture seule)
- dragArea: Array<Number> [x,y,width,height] (lecture seule)
- scrollBounds: Array<Number> [top,right,bottom,left] (lecture seule)
- targetBounds / window: Array<Number> (lecture seule)
- pointerVelocity: Number (lecture seule) ; pointerAngle: Number radians (lecture seule)
- activeProp: String — propriete animee active (lecture seule)
- Callbacks R/W: onGrab, onDrag, onRelease, onUpdate, onSettle, onSnap, onResize, onAfterResize (Function)
- Toutes les proprietes Since 4.0.0 ; les proprietes lecture seule ne sont pas assignables
- Ordres d'arrays coherents: [x,y,prevX,prevY] ou [top,right,bottom,left]


## layout

### layout

`https://animejs.com/documentation/layout`

> Le module Layout permet d'animer automatiquement la transition entre deux etats de layout HTML, y compris des proprietes normalement difficiles a animer (CSS display, flex direction, grid, ordre DOM).

Layout permet d'animer automatiquement la transition entre deux etats de layout HTML, facilitant l'animation de proprietes normalement impossibles ou difficiles a animer comme CSS display, flex direction, parametres de grille et ordre DOM. Les instances Layout sont creees via la methode createLayout() depuis le point d'entree principal 'animejs' ou importees comme module autonome depuis 'animejs/layout'. La methode accepte un selecteur CSS ou un element DOM comme root, plus des parametres optionnels pour les settings et states. Signature : createLayout(root, parameters?) ou root = selecteur CSS ou element DOM et parameters (optionnel) = objet contenant les parametres de settings et states ; retourne une instance AutoLayout. Les animations de layout se declenchent via deux approches : appeler layout.record() puis layout.animate() avant et apres la mise a jour du layout, ou utiliser layout.update(cb) avec les changements de layout a l'interieur de la fonction callback.

**Faits clés**

- Signature: createLayout(root, parameters?)
- root: selecteur CSS ou element DOM (requis)
- parameters (optionnel): objet contenant settings et states
- Retourne une instance AutoLayout
- Import: 'animejs' ou module autonome 'animejs/layout'
- Deux declenchements: record()/animate() ou update(cb)
- Anime des proprietes difficiles: CSS display, flex direction, grid, ordre DOM

```js
import { createLayout, utils, stagger } from 'animejs';

const layout = createLayout('.layout-container');

let i = 0;

function animateLayout() {
  return layout.update(({ root }) => {
    root.dataset.grid = (++i % 4) + 1;
  }, {
    duration: 1000,
    delay: stagger(150),
    onComplete: () => animateLayout()
  });
}

const layoutAnimation = animateLayout();
```

### layout/usage

`https://animejs.com/documentation/layout/usage`

> Usage du module Layout (depuis v4.3.0) : enregistrer l'etat initial du DOM puis animer vers le nouvel etat apres changement de layout, via deux approches.

La fonctionnalite Layout (disponible depuis la v4.3.0) permet d'animer automatiquement entre des etats DOM en enregistrant les positions initiales et en animant vers les nouvelles apres des changements de layout. Deux approches existent. Methode 1 - Record et Animate separement : enregistrer l'etat actuel du layout (layout.record()), modifier le DOM (classes CSS, ajout d'elements, etc.), puis declencher l'animation (layout.animate()). Methode 2 - Update dans un callback : effectuer les modifications DOM dans un callback update() ; un seul appel enregistre l'etat initial, execute les changements et anime la difference. La documentation couvre huit cas d'usage specifiques : specifier un element root, l'animation de la propriete CSS display, les layouts staggered, le reordonnancement DOM, les animations d'entree/sortie, l'echange de parent, et les dialogues modaux.

**Faits clés**

- Disponible depuis la v4.3.0
- Methode 1: createLayout() -> record() -> modifier DOM -> animate()
- Methode 2: update(cb) enregistre, execute les changements et anime en un seul appel
- Couvre 8 cas d'usage: root, CSS display, stagger, reordonnancement DOM, enter/exit, swap parent, modales

```js
const layout = createLayout(rootEl);
layout.record();
```

```js
rootEl.classList.toggle('row');
layout.animate();
```

```js
const layout = createLayout(rootEl);
layout.update(() => rootEl.classList.toggle('row'));
```

### layout/usage/specifying-a-root

`https://animejs.com/documentation/layout/usage/specifying-a-root`

> Le parametre root (requis) definit l'element racine que le layout mesure et anime ; par defaut tous ses enfants sont animes.

Le parametre root (requis ; Type : chaine selecteur CSS | element DOM ; pas de defaut, parametre obligatoire) definit l'element racine que le layout mesure et anime. Il sert de conteneur dont les descendants seront cibles pour l'animation. Par defaut, tous les enfants de l'element root sont animes. Le root etablit aussi une borne pour les requetes d'elements enfants, bien que des elements en dehors du root puissent etre cibles manuellement en assignant des attributs data layout id. Le root est le seul parametre obligatoire pour creer un layout ; les requetes d'enfants sont limitees aux descendants du root specifie ; les elements externes peuvent etre cibles via des attributs layout id manuels.

**Faits clés**

- root (requis): selecteur CSS string | element DOM ; pas de defaut
- Definit l'element racine mesure et anime
- Par defaut tous les enfants du root sont animes
- Les requetes d'enfants sont limitees aux descendants du root
- Les elements externes au root sont ciblables via attributs data layout id manuels

```js
import { createLayout, utils } from 'animejs';

const [ $rootA, $rootB ] = utils.$('.layout-container');
const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layoutA = createLayout($rootA);
const layoutB = createLayout($rootB);

function animateLayoutA() {
  layoutA.update(({ root }) => root.classList.toggle('row'));
}

function animateLayoutB() {
  layoutB.update(({ root }) => root.classList.toggle('row'));
}

$buttonA.addEventListener('click', animateLayoutA);
$buttonB.addEventListener('click', animateLayoutB);
```

```js
<div class="large layout centered row">
  <div class="layout-container col grid-layout row">
    <div class="item col">A 1</div>
    <div class="item col">A 2</div>
  </div>
  <div class="layout-container col grid-layout row">
    <div class="item col">B 1</div>
    <div class="item col">B 2</div>
  </div>
</div>
```

### layout/usage/css-display-property-animation

`https://animejs.com/documentation/layout/usage/css-display-property-animation`

> Un AutoLayout peut animer automatiquement les transitions entre proprietes CSS display (flex, grid, none), avec des etats d'entree/sortie personnalises pour les enfants masques.

CSS display property animation (Since 4.3.0) : un AutoLayout peut animer automatiquement les transitions entre proprietes CSS display comme flex, grid ou none. La librairie supporte des etats d'entree et de sortie personnalises pour les enfants qui deviennent masques via display: none ou visibility: hidden. Dans l'exemple, le parametre leaveTo definit l'etat d'animation personnalise des elements quittant le layout avec display: none (transform: scale(0), delay: stagger(75)).

**Faits clés**

- Since 4.3.0
- Anime automatiquement les transitions entre display: flex, grid, none
- Supporte etats entree/sortie personnalises pour enfants masques (display:none / visibility:hidden)
- Parametre leaveTo: etat d'animation des elements quittant le layout (ex: transform scale(0), delay stagger(75))

```js
import { createLayout, utils, stagger } from 'animejs';

const [ $button ] = utils.$('.controls button');
const items = utils.$('.item');

const displayClasses = [
  'flex-row',
  'grid-1',
  'flex-col',
  'none',
  'grid-2',
  'flex-row-reverse',
];

const layout = createLayout('.layout-container', {
  // Custom animation state for elements leaving the layout with display: none
  leaveTo: {
    transform: 'scale(0)',
    delay: stagger(75),
  },
});

let index = 0;

function animateLayout() {
  layout.update(({ root }) => {
    root.classList.remove(displayClasses[index]);
    index++;
    if (index > displayClasses.length - 1) index = 0;
    root.classList.add(displayClasses[index]);
    $button.innerText = displayClasses[index];
  });
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container flex-row">
    <div class="item col">Item A</div>
    <div class="item col">Item B</div>
    <div class="item col">Item C</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">flex-row</button>
  </fieldset>
</div>
```

```js
#layout-usage-css-display-property-animation .grid-1 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  
  & > :last-child:nth-child(odd) {
    grid-column: span 2;
  }
}

#layout-usage-css-display-property-animation .grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  
  & > :first-child {
    grid-column: span 2;
  }
}

#layout-usage-css-display-property-animation .none .item {
  display: none;
}

#layout-usage-css-display-property-animation .flex-row {
  display: flex;
  flex-direction: row;
}

#layout-usage-css-display-property-animation .flex-col {
  display: flex;
  flex-direction: column;
}

#layout-usage-css-display-property-animation .flex-row-reverse {
  display: flex;
  flex-direction: row-reverse;
}
```

### layout/usage/staggered-layout-animation

`https://animejs.com/documentation/layout/usage/staggered-layout-animation`

> La propriete delay d'un AutoLayout accepte la fonction stagger() pour creer des animations decalees quand les positions des enfants changent.

Staggered layout animation (disponible depuis la version 4.3.0) : la propriete delay de la fonctionnalite AutoLayout accepte la fonction utilitaire stagger() pour creer des animations staggered (decalees) lorsque les positions des enfants du layout changent. Cela permet a differents elements de s'animer a des moments differents selon leur position dans le layout. Dans l'exemple, le parametre from du stagger varie selon l'etat du layout (last si la classe row est presente, sinon first).

**Faits clés**

- Disponible depuis la version 4.3.0
- La propriete delay accepte la fonction stagger()
- Cree des animations decalees quand les positions des enfants changent
- Le parametre from de stagger peut varier selon l'etat du layout (ex: 'last' / 'first')

```js
import { createLayout, utils, stagger } from 'animejs';

const [ $button ] = utils.$('.controls button');
const [ $root ] = utils.$('.layout-container');
const items = utils.$('.item');

const layout = createLayout($root, { ease: 'outExpo' });

function animateLayout() {
  layout.update(() => {
    $root.classList.toggle('row');
  }, {
    // Different stagger "from" param depending on the layout state
    delay: stagger(50, { from: $root.classList.contains('row') ? 'last' : 'first' })
  });
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container">
    <div class="item col">Item A</div>
    <div class="item col">Item B</div>
    <div class="item col">Item C</div>
    <div class="item col">Item D</div>
    <div class="item col">Item E</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Stagger animation</button>
  </fieldset>
</div>
```

```js
#layout-usage-staggered-layout-animation .layout-container {
  overflow: scroll;
  flex-wrap: nowrap;
}

#layout-usage-staggered-layout-animation .layout-container .item {
  min-height: 2rem;
}
```

### layout/usage/dom-order-change-animation

`https://animejs.com/documentation/layout/usage/dom-order-change-animation`

> Anime automatiquement les changements d'ordre DOM : quand des elements sont reordonnes dans un conteneur, le layout enregistre leurs positions precedentes et les anime vers leurs nouvelles positions.

DOM order change animation (disponible depuis la v4.3.0) : cette fonctionnalite anime automatiquement les changements d'ordre DOM des elements. Quand des elements sont reordonnes dans un conteneur, le systeme de layout enregistre leurs positions precedentes et les anime de maniere fluide vers leurs nouvelles positions. La methode layout.update() accepte un callback qui modifie le DOM, declenchant l'animation basee sur les changements de position. Aucun gotcha specifique mentionne pour cette fonctionnalite.

**Faits clés**

- Disponible depuis la v4.3.0
- Anime automatiquement les changements d'ordre DOM des elements
- Enregistre les positions precedentes et anime vers les nouvelles
- layout.update() accepte un callback qui modifie le DOM
- Exemple utilise utils.shuffle() puis appendChild pour reordonner

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container');

function animateLayout() {
  layout.update(({ root }) => {
    const items = [...root.querySelectorAll('.item')];
    utils.shuffle(items).forEach($el => root.appendChild($el))
  });
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container row">
    <div class="item col">A</div>
    <div class="item col">B</div>
    <div class="item col">C</div>
    <div class="item col">D</div>
    <div class="item col">E</div>
    <div class="item col">F</div>
    <div class="item col">G</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Shuffle</button>
  </fieldset>
</div>
```

### layout/usage/enter-layout-animation

`https://animejs.com/documentation/layout/usage/enter-layout-animation`

> Anime automatiquement les elements quand ils entrent dans un layout, personnalisable via le parametre d'etat enterFrom (opacity 0 par defaut).

Enter layout animation (disponible depuis la version 4.3.0) : cette fonctionnalite anime automatiquement les elements lorsqu'ils entrent dans un layout. Les developpeurs peuvent personnaliser les proprietes initiales et le timing via le parametre d'etat enterFrom, qui par defaut met opacity a 0. Cela permet aux elements nouvellement ajoutes de s'animer en douceur a l'apparition avec des transforms, changements d'opacite, duration et fonctions d'easing specifies. Dans l'exemple, enterFrom definit transform: 'translateY(100px) scale(.25)', opacity: 0, duration: 350 et ease: 'out(3)' (duration et ease appliques aux elements entrant dans le layout).

**Faits clés**

- Disponible depuis la version 4.3.0
- Anime les elements qui entrent dans un layout
- Parametre d'etat enterFrom: opacity 0 par defaut
- enterFrom accepte transform, opacity, duration, ease (duration et ease appliques aux elements entrant)

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  duration: 250,
  ease: 'outQuad',
  enterFrom: {
    transform: 'translateY(100px) scale(.25)',
    opacity: 0,
    duration: 350, // Applied to the elements entering the layout
    ease: 'out(3)' // Applied to the elements entering the layout
  }
});

let count = 0;

function addItem() {
  layout.update(({ root }) => {
    const $item = document.createElement('div');
    $item.classList.add('item', 'col');
    $item.innerHTML = ++count;
    if (count > 15) return $button.disabled = true;
    root.appendChild($item);
  });
}

$button.addEventListener('click', addItem);
```

```js
<div class="large layout centered row">
  <div class="layout-container col grid-layout row">

  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Add item</button>
  </fieldset>
</div>
```

### layout/usage/exit-layout-animation

`https://animejs.com/documentation/layout/usage/exit-layout-animation`

> Demontre comment animer les elements quittant le layout via le parametre d'etat leaveTo, en specifiant leurs proprietes finales et timings.

La fonctionnalite d'animation de sortie de layout anime automatiquement les elements quittant le layout. On specifie leurs proprietes finales et timings via le parametre d'etat leaveTo, dont la valeur par defaut est { opacity: 0 }. leaveTo accepte des proprietes CSS/transform (ex. transform, opacity) ainsi que des timings d'animation (duration et ease) qui s'appliquent uniquement aux elements quittant le layout. Pratique : les elements doivent etre caches (via display: none, par ex. en ajoutant une classe is-hidden) AVANT d'appeler layout.update(). L'animation s'execute sur les elements caches ; on peut y acceder via la propriete layout.leaving pour les retirer du DOM apres la fin de l'animation (dans le .then()).

**Faits clés**

- Parametre d'etat: leaveTo (Object), defaut { opacity: 0 }
- leaveTo accepte des proprietes CSS/transform + duration + ease
- duration et ease dans leaveTo s'appliquent uniquement aux elements quittant le layout
- Les elements doivent etre caches via display: none AVANT layout.update()
- layout.leaving = tableau des elements qui quittent, accessible pour les retirer du DOM apres l'animation
- layout.update() retourne un thenable (.then())

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  duration: 250,
  ease: 'outQuad',
  leaveTo: {
    transform: 'translateY(-100px) scale(.25)',
    opacity: 0,
    duration: 350, // Applied to the elements leaving the layout
    ease: 'out(3)' // Applied to the elements leaving the layout
  }
});

function removeItem() {
  layout.update(({ root }) => {
    const items = root.querySelectorAll('.item:not(.is-hidden)');
    if (!items.length) return $button.disabled = true;
    items[0].classList.add('is-hidden'); // temporarily hide the element using `display: none`
  }).then(() => {
    // Remove the elements from the DOM when the animation finishes
    layout.leaving.forEach($el => $el.remove());
  });
}

$button.addEventListener('click', removeItem);
```

### layout/usage/swap-parent-animation

`https://animejs.com/documentation/layout/usage/swap-parent-animation`

> Anime automatiquement le deplacement d'un element enfant d'un parent vers un autre conteneur lors d'un changement de structure DOM.

La fonctionnalite swap parent animation anime automatiquement les elements enfants deplaces d'un conteneur parent vers un autre. Disponible depuis la version 4.3.0. Elle combine createLayout() avec la methode update() pour orchestrer l'animation lors des changements de structure DOM. layout.update() enregistre les positions actuelles des elements avant modification, applique les changements DOM (deplacement de l'enfant vers un parent different via appendChild), puis anime la transition visuelle vers les nouvelles positions de layout. Dans l'exemple, on gere les z-index des conteneurs pour controler la superposition pendant le swap.

**Faits clés**

- Disponible depuis v4.3.0
- Utilise createLayout() + update()
- update() enregistre les positions avant modif DOM, applique le changement (appendChild vers nouveau parent), puis anime la transition
- Gestion des z-index des conteneurs pour la superposition pendant le swap

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout');

function animateLayout() {
  layout.update(({ root }) => {
    const $child = root.querySelector('.item');
    const $parent = $child.parentElement;
    const $nextParent = $parent.nextElementSibling || $parent.previousElementSibling;
    $parent.style.zIndex = '0';
    $nextParent.style.zIndex = '1';
    $nextParent.appendChild($child);
  })
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container container-a col grid-layout row">
    <div class="item col">Item A</div>
  </div>
  <div class="layout-container container-b col grid-layout row">
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Swap parent</button>
  </fieldset>
</div>
```

```js
#layout-usage-swap-parent-animation .container-b {
  flex: 2;
}
```

### layout/usage/animate-modal-dialog

`https://animejs.com/documentation/layout/usage/animate-modal-dialog`

> Cree des transitions fluides entre un element cliquable et sa version agrandie dans une boite de dialogue modale, en combinant les layout ids et le parametre children.

La fonctionnalite d'animation de boite de dialogue modale (disponible depuis v4.3.0) permet des transitions fluides entre un element cliquable et sa version agrandie a l'interieur d'un dialog modal. On peut animer des elements situes en dehors du root specifie en combinant les layout ids (attribut data-layout-id) et le parametre children. Dans l'implementation : on cree un element dialog ajoute au body, puis un modalLayout via createLayout($dialog, { children: [...], properties: [...] }). A l'ouverture, on clone l'element source, on le place dans le dialog, on appelle modalLayout.update() qui affiche le modal (showModal()) et cache l'original (classe is-open). A la fermeture, update() ferme le dialog, retire la classe is-open et remet le focus. children specifie quels elements animer dans le root du modal (requis pour animer des elements hors du conteneur root). properties liste les proprietes/variables CSS a animer en plus du layout (ex. ['--overlay-alpha'] pour l'opacite de fond). data-layout-id identifie les elements correspondants entre layouts. Notes : cloner les elements avant de les ajouter au modal pour preserver l'original ; utiliser .classList.add('is-open') pour cacher le declencheur original ; appeler .focus() a la fermeture pour la navigation clavier ; display: none cache le contenu jusqu'a l'ouverture du modal. La duration peut etre definie par element declencheur via un data-attribute (data-duration).

**Faits clés**

- Disponible depuis v4.3.0
- Combine layout ids (data-layout-id) + parametre children pour animer des elements hors du root
- children: tableau de selecteurs des elements a animer dans le modal
- properties: tableau de proprietes/variables CSS a animer (ex. ['--overlay-alpha'])
- duration peut etre passee par element via data-duration (e.g. $item.dataset.duration dans les options de update())
- Cloner l'element avant appendChild pour preserver l'original
- classList.add('is-open') cache le declencheur original pendant l'animation
- Appeler .focus() a la fermeture pour la navigation clavier

```js
import { createLayout, utils } from 'animejs';

const buttons = utils.$('button');

// Create demo dialog
const $dialog = document.createElement('dialog');
$dialog.id = 'layout-dialog';
document.body.appendChild($dialog);

// Create modal layout with children specification
const modalLayout = createLayout($dialog, {
  children: ['.item', 'h2', 'h3', 'p'],
  properties: ['--overlay-alpha'],
});

const openModal = e => {
  const $target = e.target;
  const $item = $target.closest('.item');
  const $clone = $item.cloneNode(true);
  $dialog.innerHTML = '';
  $dialog.appendChild($clone);
  modalLayout.update(() => {
    $dialog.showModal();
    $item.classList.add('is-open');
  }, {
    duration: $item.dataset.duration
  });
}

const closeModal = (e) => {
  let $item;
  modalLayout.update(({ root }) => {
    $dialog.close();
    $item = buttons.find(item => item.classList.contains('is-open'));
    $item.classList.remove('is-open');
    $item.focus();
  });
};

buttons.forEach($button => $button.addEventListener('click', openModal));
$dialog.addEventListener('cancel', closeModal);
```

```js
<button data-layout-id="A" data-duration="500" class="item">
  <h2 data-layout-id="A-title">Item A</h2>
  <p>Hidden content</p>
</button>
```

### layout/layout-settings

`https://animejs.com/documentation/layout/layout-settings`

> Vue d'ensemble des parametres de configuration passes directement a createLayout().

Les layout settings sont des parametres passes directement a createLayout() dans son objet de configuration. Les reglages disponibles incluent : children (cible les elements a animer), delay (decalage de timing initial), duration (duree de l'animation en ms), ease (fonction d'easing, ex. 'inOut(3.5)'), properties (tableau de proprietes CSS a animer, ex. ['boxShadow']). Des parametres d'etat (enterFrom, leaveTo, swapAt) et des callbacks (onBegin, onUpdate, onComplete) sont egalement configurables, chacun documente sur sa propre page.

**Faits clés**

- Les settings sont passes dans l'objet de config de createLayout()
- Settings: children, delay, duration, ease, properties
- Parametres d'etat: enterFrom, leaveTo, swapAt
- Callbacks: onBegin, onUpdate, onComplete
- createLayout() est thenable (.then())

```js
import { createLayout } from 'animejs';

createLayout('.layout-container', {
  children: '.item',
  duration: 350,
  delay: 0,
  ease: 'inOut(3.5)',
  properties: ['boxShadow'],
  enterFrom: { opacity: 0 },
  leaveTo: { opacity: 0 },
  swapAt: { opacity: 0 },
  onBegin: () => {},
  onUpdate: () => {},
  onComplete: () => {},
}).then(() => {});
```

### layout/layout-settings/children

`https://animejs.com/documentation/layout/layout-settings/children`

> Specifie quels elements du root de layout doivent voir leurs positions, dimensions et proprietes animees.

Le parametre children specifie quels elements a l'interieur du root de layout doivent voir leurs positions, dimensions et proprietes animees. Type: selecteur CSS | element DOM | NodeList | Array<DOMTargetSelector>. Defaut: '*'. Les elements non explicitement cibles sont traites comme 'frozen' (geles) : ils basculent (swap) entre etats au point 50% de la transition de leur parent plutot que d'animer en continu. Cette approche evite le bruit visuel du re-flow de texte et permet de cibler les elements nouvellement ajoutes via l'attribut data-layout-id.

**Faits clés**

- Type: CSS selector | DOM element | NodeList | Array<DOMTargetSelector>
- Defaut: '*'
- Les elements non cibles sont 'frozen' : ils swap entre etats a 50% de la transition du parent au lieu d'animer en continu
- Evite le bruit visuel du re-flow de texte
- Permet de cibler les elements nouvellement ajoutes via data-layout-id

```js
import { createLayout, utils } from 'animejs';

const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  children: '.item',
  duration: 1000,
});

function animateLayout(swapAt) {
  layout.update(({ root }) => root.classList.toggle('row'), { swapAt });
}

const animateWithoutFade = () => animateLayout({ opacity: 1 });
const animateWithFade = () => animateLayout({ opacity: 0 });

$buttonA.addEventListener('click', animateWithoutFade);
$buttonB.addEventListener('click', animateWithFade);
```

```js
<div class="large layout centered row">
  <div class="layout-container col grid-layout row">
    <div class="item col"><p>These p tags are not targeted</p></div>
    <div class="item col"><p>So they simply swap between states</p></div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Animate without fade</button>
    <button class="button">Animate with fade</button>
  </fieldset>
</div>
```

### layout/layout-settings/delay

`https://animejs.com/documentation/layout/layout-settings/delay`

> Definit le delai par defaut (en ms) pour toutes les transitions de layout animees.

Le parametre delay definit le delai par defaut en millisecondes pour toutes les transitions de layout animees. Type: Number | Function. Defaut: 0. Accepte une valeur numerique (ms) ou une valeur basee sur une fonction retournant un nombre >= 0. Supporte l'utilitaire stagger() pour distribuer les delais sur plusieurs elements. Le delay peut etre surcharge par appel dans le parametre options de la methode update().

**Faits clés**

- Type: Number | Function
- Defaut: 0
- Valeur en millisecondes ou fonction retournant un nombre >= 0
- Supporte l'utilitaire stagger()
- Surchargeable par appel via le parametre options de update()

```js
import { createLayout, utils, stagger } from 'animejs';

const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  delay: 500 // Delays the transition by 500ms
});

function animateLayout(delay) {
  layout.update(({ root }) => root.classList.toggle('row'), { delay });
}

const animateWith500MsDelay = () => animateLayout();
const animateWithStaggerDelay = () => animateLayout(stagger(150));

$buttonA.addEventListener('click', animateWith500MsDelay);
$buttonB.addEventListener('click', animateWithStaggerDelay);
```

### layout/layout-settings/duration

`https://animejs.com/documentation/layout/layout-settings/duration`

> Definit la duree d'animation (en ms) pour tous les elements du layout.

Le parametre duration definit la duree d'animation en millisecondes pour tous les elements du layout. Type: Number | Function. Defaut: 350. Accepte des valeurs numeriques >= 0, ou une valeur basee sur une fonction retournant un tel nombre. Compatible avec l'utilitaire stagger().

**Faits clés**

- Type: Number | Function
- Defaut: 350
- Valeur en millisecondes >= 0 ou fonction retournant un tel nombre
- Compatible avec l'utilitaire stagger()

```js
import { createLayout, utils } from 'animejs';

const [ $rootA, $rootB ] = utils.$('.layout-container');
const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layoutA = createLayout($rootA);
const layoutB = createLayout($rootB, { duration: 1000 });

function animateLayoutA() {
  layoutA.update(({ root }) => root.classList.toggle('row'));
}

function animateLayoutB() {
  layoutB.update(({ root }) => root.classList.toggle('row'));
}

$buttonA.addEventListener('click', animateLayoutA);
$buttonB.addEventListener('click', animateLayoutB);
```

```js
<div class="large layout centered row">
  <div class="layout-container col grid-layout row">
    <div class="item col">default</div>
    <div class="item col">duration</div>
  </div>
  <div class="layout-container col grid-layout row">
    <div class="item col">1000ms</div>
    <div class="item col">duration</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Animate default</button>
    <button class="button">Animate 1000ms</button>
  </fieldset>
</div>
```

### layout/layout-settings/ease

`https://animejs.com/documentation/layout/layout-settings/ease`

> Definit la courbe d'easing ou le spring par defaut applique a toute la transition de layout.

Le parametre ease etablit la courbe d'easing ou l'animation spring par defaut appliquee a l'ensemble d'une transition de layout. Type: Easing Function | built-in ease String | Function-based value. Defaut: 'inOut(3.5)'. Disponible depuis 4.3.0. Valeurs acceptees : une fonction d'easing, un identifiant String d'easing integre, ou une 'Function based value' retournant soit une fonction d'easing soit un identifiant String integre. Il influence la progression temporelle de toutes les animations durant le changement de layout. On peut surcharger l'ease par defaut du layout dans les parametres de la methode update().

**Faits clés**

- Type: Easing Function | built-in ease String | Function-based value
- Defaut: 'inOut(3.5)'
- Disponible depuis 4.3.0
- Supporte les springs (ex. spring())
- Surchargeable par appel via les parametres de update()

```js
import { createLayout, utils, spring } from 'animejs';

const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  ease: 'outExpo'
});

function animateLayout(ease) {
  // You can override the layout default ease in the update() method parameters
  layout.update(({ root }) => root.classList.toggle('row'), { ease });
}

const animateWith500MsDelay = () => animateLayout();
const animateWithStaggerDelay = () => animateLayout(spring());

$buttonA.addEventListener('click', animateWith500MsDelay);
$buttonB.addEventListener('click', animateWithStaggerDelay);
```

### layout/layout-settings/properties

`https://animejs.com/documentation/layout/layout-settings/properties`

> Etend la liste des proprietes CSS automatiquement mesurees et animees pendant les transitions de layout.

Le parametre properties etend la liste des proprietes CSS qui sont automatiquement mesurees et animees durant les transitions de layout. Type: Array de noms de propriete CSS (String). La position et les dimensions sont toujours gerees en interne. Utiliser ce parametre pour inclure des proprietes personnalisees comme des variables CSS ou d'autres proprietes CSS non couvertes par la liste par defaut. Valeur par defaut : ['opacity', 'fontSize', 'color', 'backgroundColor', 'borderRadius', 'border', 'filter', 'clipPath'].

**Faits clés**

- Type: Array de noms de propriete CSS (String)
- Defaut: ['opacity', 'fontSize', 'color', 'backgroundColor', 'borderRadius', 'border', 'filter', 'clipPath']
- Position et dimensions toujours gerees en interne (pas besoin de les lister)
- Sert a inclure des variables CSS ou proprietes hors liste par defaut (ex. ['boxShadow'])

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  duration: 800,
  properties: ['boxShadow']
});

function animateLayout() {
  layout.update(({ root }) => root.classList.toggle('row'));
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container grid-layout row">
    <div class="item col">animate</div>
    <div class="item col">box-shadow</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Animate</button>
  </fieldset>
</div>
```

```js
#layout-layout-settings-properties .grid-layout .item {
  color: var(--hex-current-1);
  background-color: var(--hex-current-4);
  box-shadow: inset 0px 0px 6px var(--hex-current-1);
}

#layout-layout-settings-properties .grid-layout.row .item {
  box-shadow: inset 0px 0px 20px var(--hex-current-1);
}
```

### layout/states-parameters

`https://animejs.com/documentation/layout/states-parameters`

> Vue d'ensemble des parametres d'etat (enterFrom, leaveTo, swapAt) qui controlent l'apparence des elements selon la phase de transition.

Les states parameters definissent les proprietes appliquees aux elements durant des phases de transition specifiques avec createLayout(). Ils controlent l'apparence des elements lorsqu'ils entrent, sortent ou changent de position (swap) dans le layout. Trois parametres d'etat principaux : enterFrom (proprietes appliquees aux elements apparaissant dans le layout), leaveTo (proprietes appliquees aux elements disparaissant du layout), swapAt (proprietes appliquees aux enfants non animes pendant les transitions). Chaque parametre d'etat accepte un objet contenant des proprietes CSS a animer plus des overrides optionnels delay, duration et ease. Apres avoir appele layout.update() ou layout.animate(), on peut acceder a : layout.entering (elements apparus), layout.leaving (elements disparus), layout.swapping (enfants non animes). Ces tableaux sont vides et repeuples a chaque appel .animate().

**Faits clés**

- 3 parametres d'etat: enterFrom, leaveTo, swapAt
- enterFrom = elements entrants ; leaveTo = elements sortants ; swapAt = enfants non animes
- Chaque etat = objet de proprietes CSS + overrides optionnels delay/duration/ease
- Tableaux accessibles apres update()/animate(): layout.entering, layout.leaving, layout.swapping
- Ces tableaux sont vides puis repeuples a chaque appel .animate()

```js
import { createLayout } from 'animejs';

createLayout('.layout-container', {
  children: '.item',
  duration: 350,
  delay: 0,
  ease: 'inOut(3.5)',
  properties: ['boxShadow'],
  enterFrom: { opacity: 0 },
  leaveTo: { opacity: 0 },
  swapAt: { opacity: 0 },
  onBegin: () => {},
  onUpdate: () => {},
  onComplete: () => {},
}).then(() => {});
```

### layout/states-parameters/enterFrom

`https://animejs.com/documentation/layout/states-parameters/enterFrom`

> Definit les proprietes initiales et timings de transition appliques aux elements entrant dans le layout.

Le parametre enterFrom definit les proprietes initiales et les timings de transition appliques aux elements entrant dans le layout. Type: Object. Defaut: { opacity: 0 }. Un element 'entre' lorsqu'il devient visible depuis display: none ou visibility: hidden, ou lorsqu'il est nouvellement ajoute au DOM. Le parametre accepte n'importe quelle propriete CSS valide (Number|String|Function), plus delay (Number|Function), duration (Number|Function) et ease (String|Function). Limitation importante : les animations d'etat ne supportent pas encore les raccourcis (shorthands) de transform CSS. Utiliser les proprietes transform individuelles a la place.

**Faits clés**

- Type: Object
- Defaut: { opacity: 0 }
- Un element entre quand il passe de display:none/visibility:hidden a visible, ou est ajoute au DOM
- Accepte proprietes CSS (Number|String|Function) + delay + duration + ease
- Gotcha: les animations d'etat ne supportent pas les shorthands transform (x/y) ; utiliser transform: 'translate(...)' 

```js
// Invalid
enterFrom: { x: 100, y: 200 }

// Valid
enterFrom: { transform: 'translate(100px, 200px)' }
```

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  duration: 250,
  ease: 'outQuad',
  enterFrom: {
    transform: 'translateY(100px) scale(.25)',
    opacity: 0,
    duration: 350,
    ease: 'out(3)'
  }
});

let count = 0;

function addItem() {
  layout.update(({ root }) => {
    const $item = document.createElement('div');
    $item.classList.add('item', 'col');
    $item.innerHTML = ++count;
    if (count > 15) return $button.disabled = true;
    root.appendChild($item);
  });
}

$button.addEventListener('click', addItem);
```

### layout/states-parameters/leaveTo

`https://animejs.com/documentation/layout/states-parameters/leaveTo`

> Definit les proprietes CSS finales et timings de transition pour les elements quittant le layout.

Le parametre leaveTo definit les proprietes CSS finales et les timings de transition pour les elements sortant du layout. Type: Object. Defaut: { opacity: 0 }. Un element est considere comme 'leaving' lorsqu'il devient cache via display: none ou visibility: hidden. Accepte des noms de propriete CSS prenant des valeurs Number, String ou Function, plus delay (Number|Function), duration (Number|Function) et ease (String|Function). Limitation importante : les animations d'etat ne supportent pas encore les shorthands de transform CSS. Utiliser les proprietes transform individuelles a la place.

**Faits clés**

- Type: Object
- Defaut: { opacity: 0 }
- Un element 'leaving' = cache via display:none ou visibility:hidden
- Accepte proprietes CSS (Number|String|Function) + delay + duration + ease
- Gotcha: pas de shorthands transform (x/y) ; utiliser transform: 'translate(...)' 
- layout.leaving accessible apres l'animation pour retirer les elements du DOM

```js
// Invalid
leaveTo: { x: 100, y: 200 }

// Valid
leaveTo: { transform: 'translate(100px, 200px)' }
```

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  duration: 250,
  ease: 'outQuad',
  leaveTo: {
    transform: 'translateY(-100px) scale(.25)',
    opacity: 0,
    duration: 350,
    ease: 'out(3)'
  }
});

function removeItem() {
  layout.update(({ root }) => {
    const items = root.querySelectorAll('.item:not(.is-hidden)');
    if (!items.length) return $button.disabled = true;
    items[0].classList.add('is-hidden');
  }).then(() => {
    layout.leaving.forEach($el => $el.remove());
  });
}

$button.addEventListener('click', removeItem);
```

### layout/states-parameters/swapAt

`https://animejs.com/documentation/layout/states-parameters/swapAt`

> Definit les proprietes a mi-transition appliquees aux enfants non animes des elements animes.

Le parametre swapAt definit les proprietes a mi-transition appliquees aux enfants non animes des elements animes. Type: Object. Defaut: { opacity: 0, ease: 'inOut(1.75)' }. L'animation interpole vers ces valeurs specifiees a 50% de la progression, puis revient a l'etat calcule de l'element. Accepte n'importe quelle propriete CSS valide : un objet avec des noms de propriete CSS comme cles (valeurs Number|String|Function), plus delay (Number|Function), duration (Number|Function) et ease (String|Function). Limitation importante : les animations d'etat ne supportent pas encore les shorthands de transform CSS ; utiliser des declarations transform completes au lieu de proprietes raccourcies comme scale.

**Faits clés**

- Type: Object
- Defaut: { opacity: 0, ease: 'inOut(1.75)' }
- S'applique aux enfants NON animes des elements animes
- L'animation interpole vers ces valeurs a 50% de progression puis revient a l'etat calcule
- Accepte proprietes CSS (Number|String|Function) + delay + duration + ease
- Gotcha: pas de shorthands transform (ex. scale) ; utiliser transform complet
- Surchargeable par appel via le parametre options de update()

```js
import { createLayout, utils } from 'animejs';

const [ $buttonA, $buttonB ] = utils.$('.controls button');

const layout = createLayout('.layout-container', {
  children: '.item',
  duration: 1000,
});

function animateLayout(swapAt) {
  layout.update(({ root }) => root.classList.toggle('row'), { swapAt });
}

const animateWithFade = () => animateLayout({ opacity: 0, filter: 'blur(3px)' });
const animateWithoutFade = () => animateLayout({ opacity: 1 });

$buttonA.addEventListener('click', animateWithFade);
$buttonB.addEventListener('click', animateWithoutFade);
```

### layout/layout-methods

`https://animejs.com/documentation/layout/layout-methods`

> Page d'index listant les methodes disponibles sur l'instance AutoLayout retournee par createLayout(): record(), animate(), update(), revert().

Page d'index/passerelle de la section Layout methods. Seul texte substantiel: "Methods available on the AutoLayout instance returned by createLayout(), to record, animate, and revert layout states." La page liste quatre methodes (record(), animate(), update(), revert()) et fournit des liens de navigation vers les pages de documentation individuelles de chaque methode. Aucun exemple de code n'est present sur cette page d'index; la documentation detaillee de chaque methode se trouve sur des pages separees.

**Faits clés**

- Methodes listees: record(), animate(), update(), revert()
- Ces methodes sont disponibles sur l'instance AutoLayout retournee par createLayout()
- Page d'index sans exemple de code; lien Next pointe vers record()

### layout/layout-methods/record

`https://animejs.com/documentation/layout/layout-methods/record`

> record() capture un instantane (snapshot) de la disposition courante qui servira d'etat initial de la prochaine animation creee avec animate().

Signature: record(): AutoLayout. Description officielle: "Record a layout snapshot that will be used as the initial state of the next animation created with animate()." La methode capture l'etat de layout DOM courant avant modifications, etablissant une base pour les transitions animees vers de nouvelles positions. Retourne une instance AutoLayout, permettant le chainage de methodes. Workflow typique: 1) appeler record() pour capturer le layout courant, 2) modifier le DOM (reordonner, ajouter ou retirer des elements), 3) appeler animate() pour effectuer la transition de l'etat enregistre vers le nouvel etat.

**Faits clés**

- Signature: record(): AutoLayout
- Retourne une instance AutoLayout (chainable)
- Capture l'etat initial pour la prochaine animation animate()
- Methodes liees: animate(), update(), revert()

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container');

function animateLayout() {
  // Record the current state of the layout
  layout.record();
  // Update the layout state
  const first = layout.root.firstElementChild;
  if (first) layout.root.append(first);
  // Animate to the new state
  layout.animate();
}

$button.addEventListener('click', animateLayout);
```

### layout/layout-methods/animate

`https://animejs.com/documentation/layout/layout-methods/animate`

> animate(parameters?) compare le dernier snapshot record() avec les mesures actuelles et retourne un Timeline qui anime automatiquement chaque propriete modifiee entre les deux etats.

Signature: animate(parameters?: Object): Timeline. Description officielle: la methode animate() "compares the last record() snapshot with the latest measurements and returns a Timeline that automatically animates every changed property between the two states." Elle accepte des parametres d'animation optionnels (parameters, Object) pour personnaliser le timing et l'easing de la transition de layout, surchargeant les defauts pour cette transition specifique. Retourne un objet Timeline gerant les changements de layout. Pattern d'usage: appeler record() avant les changements de layout, modifier le DOM, puis invoquer animate() pour effectuer automatiquement la transition entre les deux etats avec des proprietes d'animation personnalisables.

**Faits clés**

- Signature: animate(parameters?: Object): Timeline
- Parametre optionnel parameters (Object): config d'animation surchargeant les defauts
- Retourne un Timeline
- Compare le dernier snapshot record() aux mesures actuelles et anime chaque propriete modifiee

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container');

function animateLayout() {
  // Record the current state of the layout
  layout.record();
  // Update the layout state
  const first = layout.root.firstElementChild;
  if (first) layout.root.append(first);
  // Animate to the new state
  layout.animate({
    duration: 750,
    ease: 'out(4)',
  });
}

$button.addEventListener('click', animateLayout);
```

### layout/layout-methods/update

`https://animejs.com/documentation/layout/layout-methods/update`

> update(callback, overrides?) est un helper en un seul appel combinant record(), l'execution d'un callback de mutation du DOM, puis animate().

Signature: update(callback: Function, overrides?: Object): Timeline. Parametres: callback (Function) - fonction executant les mutations DOM pour mettre a jour le layout; overrides (Object, optionnel) - parametres d'animation pour surcharger le timing/easing par defaut du layout pour cette transition specifique. La methode update() est un helper unique qui combine trois operations: elle invoque record() pour capturer l'etat initial, execute le callback fourni pour modifier le DOM, puis appelle animate() avec les parametres optionnels overrides pour effectuer la transition entre l'ancien et le nouveau layout. Retourne un Timeline.

**Faits clés**

- Signature: update(callback: Function, overrides?: Object): Timeline
- callback (Function): mutations DOM; overrides (Object, optionnel): parametres d'animation
- Combine record() + callback + animate()
- Retourne un Timeline
- GOTCHA: "This might not work in some frameworks (I haven't tested all of them). Use the manual record() / .animate() combo if the animation doesn't work with .update()."

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');

const layout = createLayout('.layout-container');

function animateLayout() {
  // Triggers both layout.record() and layout.animate()
  layout.update(() => {
    const first = layout.root.firstElementChild;
    if (first) layout.root.append(first);  
  }, {
    duration: 750,
    ease: 'out(4)',  
  });
}

$button.addEventListener('click', animateLayout);
```

### layout/layout-methods/revert

`https://animejs.com/documentation/layout/layout-methods/revert`

> revert() termine toutes les animations de layout en cours, ramenant le DOM a son etat actuel reel.

Signature: revert(): AutoLayout. Description officielle: la methode "completes all currently running layout animations, reverting the DOM to its actual current state." Elle stoppe les transitions de layout actives et ramene le DOM pour refleter le layout reel courant sans animation. Retourne une instance AutoLayout, permettant le chainage de methodes. Disponible depuis la version v4.3.0.

**Faits clés**

- Signature: revert(): AutoLayout
- Retourne une instance AutoLayout (chainable)
- Termine toutes les animations de layout en cours et ramene le DOM a son etat reel courant
- Disponible depuis v4.3.0
- createLayout accepte un objet de params en 2e argument (ex: { duration: 5000, ease: 'out(3)' })

```js
import { createLayout, utils } from 'animejs';

const [ $animate, $revert ] = utils.$('.controls button');

const layout = createLayout('.layout-container', { duration: 5000, ease: 'out(3)' });

function animateLayout() {
  layout.update(() => {
    const first = layout.root.firstElementChild;
    if (first) layout.root.append(first);
    $revert.disabled = false;
  }).then(() => $revert.disabled = true);
}

function revertLayout() {
  layout.revert();
}

$animate.addEventListener('click', animateLayout);
$revert.addEventListener('click', revertLayout);
```

### layout/layout-id-attribute

`https://animejs.com/documentation/layout/layout-id-attribute`

> L'attribut data-layout-id permet d'animer automatiquement entre deux elements situes a des emplacements DOM differents, sans clonage ni deplacement.

Les Layout IDs permettent des animations automatiques entre deux elements a des emplacements DOM differents sans cloner ni deplacer les elements. Les IDs sont assignes automatiquement ou definis via l'attribut data-layout-id (accessible en JS via element.dataset.layoutId). Comportement: quand deux elements partagent le meme layout id et que l'un est masque (display: none ou visibility: hidden) tandis que l'autre est visible, le layout anime automatiquement entre eux lors des changements d'etat. Utile pour animer entre des etats DOM alternatifs sans repositionnement d'element. Introduit en version 4.3.0.

**Faits clés**

- Attribut: data-layout-id (JS: element.dataset.layoutId)
- IDs assignes automatiquement ou definis manuellement
- Anime entre deux elements de meme layout id quand l'un est masque (display:none / visibility:hidden) et l'autre visible
- Pas de clonage ni de deplacement d'element
- Introduit en v4.3.0

```js
import { createLayout, utils } from 'animejs';

const [ $button ] = utils.$('.controls button');
const [ $itemA1, $itemA2 ] = utils.$('.item');

// Manually set the same layout id to both items
$itemA1.dataset.layoutId = "item-A";
$itemA2.dataset.layoutId = "item-A";

// Hide item 2
$itemA2.classList.add('is-hidden');

const layout = createLayout('.layout');

function animateLayout() {
  layout.update(({ root }) => {
    // Toggle the visibility and alternate between the two items
    $itemA1.classList.toggle('is-hidden');
    $itemA2.classList.toggle('is-hidden');
  });
}

$button.addEventListener('click', animateLayout);
```

```js
<div class="large layout centered row">
  <div class="layout-container container-a col grid-layout row">
    <div class="item col">Item A</div>
  </div>
  <div class="layout-container container-b col grid-layout row">
    <div class="item col">Item A</div>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Toggle visibility</button>
  </fieldset>
</div>
```

```js
#layout-layout-id-attribute .item.is-hidden {
  display: none;
}
```

### layout/layout-callbacks

`https://animejs.com/documentation/layout/layout-callbacks`

> Les animations de layout heritent de tous les callbacks de Timeline (onBegin, onUpdate, onComplete), executes a des points specifiques de lecture.

Les animations de layout heritent de tous les callbacks de Timeline, permettant d'executer des fonctions a des points specifiques de la lecture, comme pour un Timeline normal. Callbacks supportes (herites de Timeline): onBegin (au demarrage de l'animation), onUpdate (pendant la lecture), onComplete (a la fin de l'animation). Note: la methode .then() est disponible sur l'objet Timeline retourne par .update() et .animate(), permettant un chainage de callbacks base sur les promesses apres la fin des animations de layout.

**Faits clés**

- Heritent de tous les callbacks de Timeline
- Callbacks: onBegin, onUpdate, onComplete
- La methode .then() est disponible sur le Timeline retourne par .update() et .animate() (chainage promesse)
- Les callbacks sont passes dans l'objet de params de createLayout()

```js
import { createLayout } from 'animejs';

createLayout(root, {
  children: '.item',
  duration: 350,
  delay: 0,
  ease: 'inOut(3.5)',
  properties: ['boxShadow'],
  enterFrom: { opacity: 0 },
  leaveTo: { opacity: 0 },
  swapAt: { opacity: 0 },
  onBegin: () => {},
  onUpdate: () => {},
  onComplete: () => {},
}).then(() => {});
```

### layout/layout-properties

`https://animejs.com/documentation/layout/layout-properties`

> Proprietes exposees sur l'instance AutoLayout retournee par createLayout() (params, root, children, etats, timeline, tableaux runtime, id, etc.).

L'instance AutoLayout retournee par createLayout() expose les proprietes suivantes. params (Object): configuration passee a createLayout(). root (HTMLElement): element racine resolu pour les mesures. children (String | Array<String>): selecteur(s) des elements suivis. enterFromParams (Object): parametres d'animation pour les noeuds entrants. leaveToParams (Object): parametres d'animation pour les noeuds sortants. swapAtParams (Object): parametres d'animation pour les noeuds echanges (swap). properties (Set<String>): noms des proprietes CSS interpolees lors des changements de valeur. oldState (LayoutSnapshot): mesures precedentes. newState (LayoutSnapshot): dernieres mesures. timeline (Timeline | null): timeline du dernier .animate() ou .update(). animating (Array<DOMTarget>): noeuds animes lors du dernier appel. swapping (Array<DOMTarget>): noeuds echanges lors du dernier appel. entering (Array<DOMTarget>): noeuds entrants lors du dernier appel. leaving (Array<DOMTarget>): noeuds sortants lors du dernier appel. id (String | Number): identifiant de layout (get/set). Notes: les LayoutSnapshot (oldState/newState) fournissent des methodes d'inspection comme .getNode(element) et .getComputedValue(element, property); les tableaux entering/leaving/swapping se repeuplent a chaque appel .animate() et doivent etre lus immediatement apres .update() pour coordonner des etats personnalises.

**Faits clés**

- params (Object): config passee a createLayout()
- root (HTMLElement): element racine resolu
- children (String | Array<String>): selecteur(s) suivis
- enterFromParams / leaveToParams / swapAtParams (Object): params anim entree/sortie/swap
- properties (Set<String>): proprietes CSS interpolees
- oldState / newState (LayoutSnapshot): mesures precedentes / dernieres
- timeline (Timeline | null): timeline du dernier animate()/update()
- animating / swapping / entering / leaving (Array<DOMTarget>): noeuds du dernier appel
- id (String | Number): identifiant get/set
- LayoutSnapshot expose .getNode(element) et .getComputedValue(element, property)
- entering/leaving/swapping se repeuplent a chaque .animate(); lire immediatement apres .update()

### layout/common-auto-layout-gotchas

`https://animejs.com/documentation/layout/common-auto-layout-gotchas`

> Liste des pieges courants du layout automatique et leurs contournements (fade inattendu, saut du root, saut de texte, elements inline non animes, transform shorthand, SVG).

Six pieges courants sont documentes. 1) Elements fading out unexpectedly: des descendants non cibles passent a opacity 0 puis reviennent a 1 quand la taille de leur parent change sans qu'ils soient explicitement selectionnes; contournement: les ajouter aux selecteurs children ou ajuster swapAt (ex: swapAt: { opacity: 1 }). 2) Root element position jumps: la position de l'element racine ne peut pas s'animer car cela affecterait le positionnement des freres hors du layout; contournement: utiliser l'element parent comme nouvelle racine. 3) Text jumping during transition: animer fontSize avec width/height provoque un reflow du texte, surtout dans Firefox a cause des ratios width/font-size intermediaires; contournement: appliquer white-space: nowrap quand le retour a la ligne n'est pas necessaire. 4) Inlined elements not moving: les elements adjacents a des noeuds texte sont exclus des animations de position pour preserver le flux du texte; contournement: envelopper le texte dans des balises span. 5) Transform shorthands not working: les parametres enterFrom, leaveTo et swapAt ne supportent pas les transform en raccourci; contournement: utiliser des chaines transform completes (ex: transform: 'scale(0)'). 6) SVG elements not animated: les elements SVG et leurs descendants sont exclus; seuls les elements HTML sont suivis (aucun contournement fourni dans la doc).

**Faits clés**

- 6 gotchas: fade out inattendu, saut de position du root, saut de texte, elements inline non deplaces, transform shorthand non supporte, SVG non anime
- Fade out: ajouter aux children ou ajuster swapAt
- Root jump: utiliser le parent comme racine
- Text jumping: white-space: nowrap (notamment Firefox)
- Inline: envelopper le texte dans des span
- enterFrom/leaveTo/swapAt ne supportent pas les transform shorthand -> utiliser des chaines transform completes
- SVG et descendants exclus; seuls les elements HTML suivis (pas de contournement)

```js
const layout = createLayout('#root', {
  children: '.card',
  swapAt: { opacity: 1 }
});
```

```js
// Instead of:
const layout = createLayout('#container .grid');

// Use:
const layout = createLayout('#container');
```

```js
<!-- Instead of: -->
<p>Some text <span class="highlight">inline element</span> more text</p>

<!-- Use: -->
<p><span>Some text</span> <span class="highlight">inline element</span> <span>more text</span></p>
```

```js
createLayout('#root', {
  enterFrom: { transform: 'scale(0)' }
});
```


## scope

### scope

`https://animejs.com/documentation/scope`

> createScope(parameters) cree un Scope dont les instances anime.js peuvent reagir aux media queries, utiliser des roots personnalises, partager des params par defaut et etre reverties en lot.

Import: import { createScope } from 'animejs'; (ou import { createScope } from 'animejs/scope';). Signature: const scope = createScope(parameters); Description officielle: "Anime.js instances declared inside a Scope can react to media queries, use custom root elements, share default parameters, and be reverted in batch, streamlining work in responsive and component-based environments." Accepte un objet Scope parameters optionnel avec des options comme mediaQueries, root et defaults. Retourne une instance Scope. Sections liees: Scope parameters (root, defaults, mediaQueries), Scope methods (add, addOnce, keepTime, revert, refresh), Scope properties.

**Faits clés**

- Signature: createScope(parameters) -> Scope
- Import: 'animejs' ou 'animejs/scope'
- Parametres: mediaQueries, root, defaults (objet optionnel)
- Les instances dans un Scope reagissent aux media queries, roots personnalises, params partages, revert en lot
- Methodes du Scope: add, addOnce, keepTime, revert, refresh
- self.matches expose l'etat des media queries nommees

```js
import { animate, utils, createScope } from 'animejs';

createScope({
  mediaQueries: {
    isSmall: '(max-width: 200px)',
    reduceMotion: '(prefers-reduced-motion)',
  }
})
.add(self => {

  const { isSmall, reduceMotion } = self.matches;
  
  if (isSmall) {
    utils.set('.square', { scale: .5 });
  }
    
  animate('.square', {
    x: isSmall ? 0 : ['-35vw', '35vw'],
    y: isSmall ? ['-40vh', '40vh'] : 0,
    loop: true,
    alternate: true,
    duration: reduceMotion ? 0 : isSmall ? 750 : 1250
  });

});
```

### scope/add-constructor-function

`https://animejs.com/documentation/scope/add-constructor-function`

> scope.add(constructor) / scope.addOnce(constructor) execute une fonction constructeur dans le contexte du Scope; elle recoit self et peut retourner une fonction de nettoyage.

Signatures: scope.add(constructor) et scope.addOnce(constructorFunction). Le constructeur recoit un argument self (l'instance Scope courante). Il peut retourner (optionnellement) une fonction de nettoyage (cleanup) invoquee quand le Scope se reverte ou quand les media queries changent. La fonction constructeur s'execute immediatement dans le contexte du Scope apres avoir ete passee a add() ou addOnce(). Le Scope "registers and keeps track of all animations, timers, timelines, animatables, draggables, onScrolls, and even other scopes declared inside the constructor function." Gotcha: les methodes/animations declarees a l'interieur du scope accedent a self (le contexte du scope), permettant un comportement dynamique base sur des proprietes comme matches.

**Faits clés**

- Signatures: scope.add(constructor) et scope.addOnce(constructorFunction)
- Le constructeur recoit self (instance Scope courante)
- Retour optionnel: fonction cleanup invoquee au revert du Scope ou changement de media query
- S'execute immediatement dans le contexte du Scope
- Le Scope enregistre et suit toutes les animations, timers, timelines, animatables, draggables, onScrolls et autres scopes declares dans le constructeur
- self.matches expose l'etat des media queries; utils.$() est scope

```js
import { utils, animate, createScope, createDraggable } from 'animejs';

createScope({
  mediaQueries: { isSmall: '(max-width: 200px)' },
  defaults: { ease: 'linear' },
})
.add(self => {

  /* Media queries state are accessible on the matches property */
  const { isSmall } = self.matches;
  /* The $() utility method is also scoped */
  const [ $square ] = utils.$('.square');

  if (self.matches.isSmall) {
    /* Only animate the square when the iframe is small */
    animate($square, {
      rotate: 360,
      loop: true,
    });
  } else {
    /* Only create the draggable when the iframe is large enough */
    $square.classList.add('draggable');
    createDraggable($square, {
      container: document.body,
    });
  }
  
  return () => {
    /* Removes the class 'draggable' when the scope reverts itself */
    $square.classList.remove('draggable');
  }

});
```

### scope/register-method-function

`https://animejs.com/documentation/scope/register-method-function`

> self.add('methodName', methodFunction) enregistre une methode dans le Scope, ensuite accessible via scope.methods.methodName en conservant le contexte du Scope.

Signature: scope.add('methodName', methodFunction) (utilise sous la forme self.add(name, fn) a l'interieur du constructeur). Parametres: name (String) - identifiant de la methode; methodFunction (Function) - la fonction a enregistrer, recevant ...args de type quelconque. Les methodes enregistrees dans un Scope deviennent accessibles via l'objet methods de l'instance Scope: "Once registered, the method becomes available on the Scope instance's methods object." Cela permet une invocation externe (ex: comme handler d'evenement) tout en maintenant le contexte d'execution dans le Scope. Gotcha: les methodes enregistrees dans le scope accedent a self (le contexte du scope), permettant un comportement dynamique base sur des proprietes comme matches.

**Faits clés**

- Signature: scope.add(name, methodFunction) (via self.add() dans le constructeur)
- name (String): identifiant de la methode; methodFunction (Function): recoit ...args
- La methode enregistree devient disponible sur scope.methods (ex: scope.methods.onClick)
- Permet l'invocation externe tout en gardant le contexte du Scope
- Les methodes accedent a self (ex: self.matches)

```js
import { utils, animate, createScope } from 'animejs';

const scope = createScope({
  mediaQueries: { isSmall: '(max-width: 200px)' },
})
.add(self => {
  
  self.add('onClick', (e) => {
    const { clientX, clientY } = e;
    const { isSmall } = self.matches;
    
    animate('.square', {
      rotate: isSmall ? '+=360' : 0,
      x: isSmall ? 0 : clientX - (window.innerWidth / 2),
      y: isSmall ? 0 : clientY - (window.innerHeight / 2),
      duration: isSmall ? 750 : 400,
    });
  });
  
  utils.set(document.body, {
    cursor: self.matches.isSmall ? 'alias' : 'crosshair'
  });
});

document.addEventListener('click', scope.methods.onClick);
```

### scope/scope-parameters

`https://animejs.com/documentation/scope/scope-parameters`

> createScope() accepte trois parametres: root (selecteur/element racine), defaults (params d'animation par defaut) et mediaQueries (objet de media queries nommees).

createScope() accepte trois parametres. root: selecteur CSS (String) ou element definissant la portee racine pour les selections et mesures. defaults (Object): parametres par defaut partages pour les animations declarees dans le scope (ex: duration, ease). mediaQueries (Object): objet associant des noms de query a des chaines de media query (ex: { mobile: '(max-width: 640px)' }). Le contexte passe au constructeur (self/ctx) expose un objet matches contenant des booleens pour chaque media query definie (ex: ctx.matches.mobile, ctx.matches.reducedMotion), facilitant les animations adaptatives. Note: la page liste ces trois parametres mais ne fournit pas de tableau explicite de types ni de valeurs par defaut detaillees, renvoyant vers les pages individuelles.

**Faits clés**

- 3 parametres: root, defaults, mediaQueries
- root: selecteur CSS String ou element (portee racine)
- defaults: Object de params d'animation par defaut (ex: duration, ease)
- mediaQueries: Object {nom: 'media query string'} (ex: { mobile: '(max-width: 640px)' })
- Le contexte expose matches (booleens par media query, ex: ctx.matches.mobile)
- La page ne donne pas de tableau explicite de types/defauts (renvoie aux pages individuelles)

```js
import { createScope, animate } from 'animejs';

createScope({
  root: '.section',
  defaults: {
    duration: 250,
    ease: 'out(4)',
  },
  mediaQueries: {
    mobile: '(max-width: 640px)',
    reducedMotion: '(prefers-reduced-motion)',
  }
})
.add( ctx => {
  const isMobile = ctx.matches.mobile;
  const reduceMotion = ctx.matches.reducedMotion;
  animate(targets, {
    x: isMobile ? 0 : '100vw',
    y: isMobile ? '100vh' : 0,
    duration: reduceMotion ? 0 : 750
  });
});
```

### scope/scope-parameters/root

`https://animejs.com/documentation/scope/scope-parameters/root`

> Le parametre root limite toutes les requetes DOM d'un Scope aux descendants de l'element specifie.

Le parametre `root` (CSS Selector | DOM Element) definit un element racine qui limite toutes les requetes DOM a l'interieur d'un Scope aux descendants du HTMLElement specifie. Particulierement utile pour creer des environnements d'animation isoles dans des architectures basees composants, comme les applications React. Quand un `root` est specifie, tous les selecteurs (ex. `.square`) ne se resolvent qu'a l'interieur des descendants de cet element racine, creant des contextes d'animation auto-contenus. Disponible depuis la version 4.0.0.

**Faits clés**

- Nom du parametre: root
- Type: CSS Selector | DOM Element
- S'applique a createScope()
- Disponible depuis 4.0.0
- Limite les requetes DOM aux descendants de l'element racine
- Utile pour architectures composants (React)

```js
import { createScope, animate } from 'animejs';

createScope({ root: '.row:nth-child(2)' })
.add(() => {
  animate('.square', {
    x: '17rem',
    loop: true,
    alternate: true
  });
});
```

```js
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">outside scope</div>
</div>
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">inside scope</div>
</div>
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">outside scope</div>
</div>
```

### scope/scope-parameters/defaults

`https://animejs.com/documentation/scope/scope-parameters/defaults`

> Le parametre defaults definit des proprietes par defaut pour toutes les instances Timer, Animation et Timeline creees dans un scope.

Le parametre `defaults` (Object, optionnel) etablit des proprietes preconfiguree pour toutes les instances Timer, Animation et Timeline creees a l'interieur d'un scope donne. Ces valeurs par defaut servent de valeurs de repli sauf si elles sont surchargees au niveau de l'animation individuelle. S'applique a createScope(). Proprietes acceptees: playbackEase (Easing name String | Function), playbackRate (Number), frameRate (Number), loop (Number | Boolean), reversed (Boolean), alternate (Boolean), autoplay (Boolean), duration (Number | Function), delay (Number | Function), composition (String | Function), ease (Easing name String | Function), loopDelay (Number), modifier (Function), onBegin (Callback Function), onUpdate (Callback Function), onRender (Callback Function), onLoop (Callback Function), onComplete (Callback Function).

**Faits clés**

- Nom du parametre: defaults
- Type: Object (optionnel)
- S'applique a createScope()
- Valeurs de repli surchageables au niveau animation individuelle
- Proprietes: playbackEase, playbackRate, frameRate, loop, reversed, alternate, autoplay, duration, delay, composition, ease, loopDelay, modifier, onBegin, onUpdate, onRender, onLoop, onComplete

```js
import { createScope, animate } from 'animejs';

const rows = utils.$('.row');

rows.forEach(($row, i) => {
  createScope({
    root: $row,
    defaults: { ease: `out(${1 + i})` }
  })
  .add(() => {
    animate('.square', {
      x: '17rem',
      loop: true,
      alternate: true
    });
  });
});
```

```js
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">scope 1</div>
</div>
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">scope 2</div>
</div>
<div class="medium row">
  <div class="square"></div>
  <div class="padded label">scope 3</div>
</div>
```

### scope/scope-parameters/mediaqueries

`https://animejs.com/documentation/scope/scope-parameters/mediaqueries`

> Le parametre mediaQueries definit des media queries pour rafraichir conditionnellement un Scope quand leurs etats de correspondance changent.

Le parametre `mediaQueries` (Object) definit des media queries pour rafraichir conditionnellement un Scope lorsque leurs etats de correspondance (match states) changent. Les etats de correspondance deviennent accessibles via la propriete `matches` du scope. L'objet accepte des cles (noms de chaine arbitraires pour chaque media query) et des valeurs (chaines de definition de media query). Les etats correspondants sont accessibles par destructuration depuis `self.matches` a l'interieur de la fonction de callback du scope, permettant une logique d'animation responsive basee sur la taille du viewport et les preferences utilisateur. Disponible depuis 4.0.0.

**Faits clés**

- Nom du parametre: mediaQueries
- Type: Object
- Cles: noms de chaine arbitraires; Valeurs: chaines de definition de media query
- Etats de correspondance accessibles via self.matches
- S'applique a createScope()
- Disponible depuis 4.0.0
- Rafraichit conditionnellement le Scope au changement d'etat de match

```js
import { createScope, animate } from 'animejs';

createScope({
  mediaQueries: {
    isSmall: '(max-width: 100px)',
    isMedium: '(min-width: 101px) and (max-width: 200px)',
    isLarge: '(min-width: 201px)',
    reduceMotion: '(prefers-reduced-motion)',
  }
})
.add(self => {

  const { isSmall, isMedium, isLarge, reduceMotion } = self.matches;
    
  utils.set('.square', { scale: isMedium ? .75 : isLarge ? 1 : .5 });
    
  animate('.square', {
    x: isSmall ? 0 : ['-35vw', '35vw'],
    y: isSmall ? ['-40vh', '40vh'] : 0,
    rotate: 360,
    loop: true,
    alternate: true,
    duration: reduceMotion ? 0 : isSmall ? 750 : 1250
  });

});
```

### scope/scope-methods

`https://animejs.com/documentation/scope/scope-methods`

> Vue d'ensemble des methodes disponibles sur une instance Scope retournee par createScope().

Les Scope methods sont des fonctions disponibles sur les instances retournees par `createScope()`. La documentation indique: 'Methods available on the Scope instance returned by a createScope() function.' Cinq methodes sont listees: add() (ajoute animations/timers ou enregistre une methode nommee), addOnce() (ajoute animations/timers executes une seule fois), keepTime() (maintient la synchronisation temporelle entre rafraichissements), revert() (annule les changements du scope), refresh() (rafraichit l'etat du scope). Chaque methode possede sa propre page de documentation dediee. Disponible depuis 4.0.0.

**Faits clés**

- 5 methodes: add(), addOnce(), keepTime(), revert(), refresh()
- Disponibles sur l'instance Scope retournee par createScope()
- Disponible depuis 4.0.0

```js
const scope = createScope(parameters);
scope.add()
scope.refresh()
scope.revert()
```

### scope/scope-methods/add

`https://animejs.com/documentation/scope/scope-methods/add`

> add() ajoute une fonction constructeur ou enregistre une fonction methode nommee a une instance Scope.

La methode `add()` a un double role: elle peut ajouter une fonction constructeur OU enregistrer une fonction methode nommee a une instance Scope, permettant une liaison de fonctionnalite dynamique dans les animations scopees. Signatures: `scope.add(constructor)` et `scope.add(name, method)`. Pour ajouter un constructeur: `constructor` (Function) recoit l'instance Scope comme argument. Pour enregistrer une methode: `name` (String) identifiant pour stocker/acceder la methode, `method` (Function) fonction methode a enregistrer. Retourne l'instance Scope elle-meme, permettant le chainage de methodes. Les methodes nommees enregistrees sont accessibles via `scope.methods`.

**Faits clés**

- Signatures: scope.add(constructor) et scope.add(name, method)
- constructor (Function) recoit l'instance Scope comme argument
- name (String) + method (Function) pour enregistrer une methode nommee
- Retourne l'instance Scope (chainable)
- Methodes nommees accessibles via scope.methods

```js
import { createScope, createAnimatable, createDraggable } from 'animejs';

const scope = createScope({
  mediaQueries: {
    isSmall: '(max-width: 200px)',
  }
})
.add(self => {

  const [ $circle ] = utils.$('.circle');
    
  if (self.matches.isSmall) {
    $circle.classList.add('draggable');
    self.circle = createDraggable($circle, {
      container: document.body,
    });
  } else {
    $circle.classList.remove('draggable');
    self.circle = createAnimatable($circle, {
      x: 500,
      y: 500,
      ease: 'out(3)'
    });
  }
  
  let win = { w: window.innerWidth, h: window.innerHeight };
  
  self.add('refreshBounds', () => {
    win.w = window.innerWidth;
    win.h = window.innerHeight;
  });
      
  self.add('onMouseMove', e => {
    if (self.matches.isSmall) return;
    const { w, h } = win;
    const hw = w / 2;
    const hh = h / 2;
    const x = utils.clamp(e.clientX - hw, -hw, hw);
    const y = utils.clamp(e.clientY - hh, -hh, hh);
    if (self.circle.x) {
      self.circle.x(x);
      self.circle.y(y);
    }
  });
  
  self.add('onPointerDown', e => {
    const { isSmall } = self.matches;
    animate($circle, {
      scale: [
        { to: isSmall ? 1.25 : .25, duration: isSmall ? 50 : 150 },
        { to: 1, duration: isSmall ? 250 : 500 },
      ]
    });
  });
  
});

window.addEventListener('resize', scope.methods.refreshBounds);
window.addEventListener('mousemove', scope.methods.onMouseMove);
document.addEventListener('pointerdown', scope.methods.onPointerDown);
```

### scope/scope-methods/addonce

`https://animejs.com/documentation/scope/scope-methods/addonce`

> addOnce() ajoute un constructeur a un Scope qui s'execute une seule fois, non reverti entre changements de media query.

La methode `addOnce()` ajoute un constructeur a un Scope qui s'execute une seule fois. Cela permet d'executer du code une fois et d'ajouter des animations scopees qui ne seront pas reverties entre les changements de media query. Signature: `scope.addOnce(constructor)`. Parametre: `constructor` (Function). Retourne le Scope lui-meme. Contrainte cle: le code de addOnce() ne peut pas etre conditionnel, car cela en annule le but et perturbe le suivi des callbacks deja executes ou non. Disponible depuis 4.1.0.

**Faits clés**

- Signature: scope.addOnce(constructor)
- constructor (Function)
- Retourne le Scope
- S'execute une seule fois; animations non reverties entre changements de media query
- GOTCHA: ne peut PAS etre conditionnel (perturbe le suivi des callbacks executes)
- Disponible depuis 4.1.0

```js
if (scope.matches.small) {
  scope.addOnce(() => { animate(target, params) });
}
```

```js
scope.addOnce(() => { animate(target, params) });
```

```js
import { createScope, createTimeline, utils, stagger } from 'animejs';

const scope = createScope({
  mediaQueries: {
    isSmall: '(max-width: 200px)',
  }
})
.add(self => {
  self.addOnce(() => {
    /* Animations declared here won't be reverted between mediaqueries changes */
    createTimeline().add('.circle', {
      backgroundColor: [
        $el => utils.get($el, `--hex-red-1`),
        $el => utils.get($el, `--hex-citrus-1`),
      ],
      loop: true,
      alternate: true,
      duration: 2000,
    }, stagger(100));
  });
  
  self.add(() => {
    createTimeline().add('.circle', {
      x: self.matches.isSmall ? [-30, 30] : [-70, 70],
      scale: [.5, 1.1],
      loop: true,
      alternate: true,
    }, stagger(100)).init();
  });
});
```

### scope/scope-methods/keeptime

`https://animejs.com/documentation/scope/scope-methods/keeptime`

> keepTime() recree un Timer, Animation ou Timeline entre changements de media query tout en conservant son temps courant.

La methode `keepTime()` accepte une fonction constructeur qui retourne un Timer, Animation ou Timeline. Elle recree un Timer, Animation ou Timeline entre les changements de media query tout en gardant la trace de son temps courant, permettant de mettre a jour de maniere transparente les parametres d'une animation sans casser l'etat de lecture. Signature: `scope.keepTime(constructor: Function): Timer | Animation | Timeline`. Parametre: constructor (fonction retournant une instance Timer/Animation/Timeline). Retourne l'objet Timer/Animation/Timeline cree. Contrainte: les appels keepTime() ne peuvent pas etre conditionnels, car cela en annule le but et perturbe le suivi des callbacks deja executes ou non. Permet aux animations responsive d'adapter leurs parametres entre breakpoints sans perturber l'etat de lecture.

**Faits clés**

- Signature: scope.keepTime(constructor: Function): Timer | Animation | Timeline
- Constructor retourne un Timer/Animation/Timeline
- Conserve le temps courant entre changements de media query
- Permet mise a jour transparente des parametres sans casser la lecture
- GOTCHA: ne peut PAS etre conditionnel

```js
// Don't do this
if (scope.matches.small) {
  scope.keepTime(() => animate(target, params));
}
```

```js
// Do this
scope.keepTime(() => animate(target, params));
```

```js
import { createScope, createTimeline, utils, stagger } from 'animejs';

const scope = createScope({
  mediaQueries: {
    isSmall: '(max-width: 200px)',
  }
})
.add(self => {
  self.addOnce(() => {
    createTimeline().add('.circle', {
      backgroundColor: [
        $el => utils.get($el, `--hex-red-1`),
        $el => utils.get($el, `--hex-citrus-1`),
      ],
      loop: true,
      alternate: true,
      duration: 2000,
    }, stagger(100));
  });
  
  self.keepTime(() => createTimeline().add('.circle', {
    x: self.matches.isSmall ? [-30, 30] : [-70, 70],
    scale: [.5, 1.1],
    loop: true,
    alternate: true,
  }, stagger(100)).init());
});
```

### scope/scope-methods/revert

`https://animejs.com/documentation/scope/scope-methods/revert`

> revert() annule tous les objets Anime.js declares dans un Scope et execute les fonctions de nettoyage enregistrees.

La methode `revert()` annule tous les objets Anime.js declares a l'interieur d'une instance Scope et execute toute fonction de nettoyage (cleanup) enregistree par les constructeurs. Elle retourne le Scope lui-meme pour le chainage. Signature: `revert(): Scope`. Comportement: annule automatiquement toutes les animations creees dans le scope, execute la fonction de nettoyage retournee par le constructeur du scope (utile pour retirer des ecouteurs d'evenements et autre configuration manuelle).

**Faits clés**

- Signature: revert(): Scope
- Annule tous les objets Anime.js declares dans le scope
- Execute la fonction de cleanup retournee par le constructeur
- Retourne le Scope (chainable)
- Utile pour retirer les event listeners

```js
import { utils, stagger, createScope, createTimeline } from 'animejs';

const [ $button1, $button2 ] = utils.$('.revert');

function onMouseEnter() { animate(this, { scale: 2, duration: 250 }) }
function onMouseLeave() { animate(this, { scale: 1, duration: 750 }) }

const scopeConstructor = scope => {
  const circles = utils.$('.circle');
    
  circles.forEach(($circle, i) => {
    animate($circle, {
      opacity: .25,
      loop: true,
      alternate: true,
      duration: 500,
      delay: i * 100,
      ease: 'inOut(3)',
    });
    $circle.addEventListener('mouseenter', onMouseEnter);
    $circle.addEventListener('mouseleave', onMouseLeave);
  });
  
  // Cleanup function to remove event listeners on revert
  return () => {
    circles.forEach($circle => {
      $circle.removeEventListener('mouseenter', onMouseEnter);
      $circle.removeEventListener('mouseleave', onMouseLeave);
    });
  }
}

const scope1 = createScope({ root: '.row-1' }).add(scopeConstructor);
const scope2 = createScope({ root: '.row-2' }).add(scopeConstructor);

$button1.addEventListener('click', () => scope1.revert());
$button2.addEventListener('click', () => scope2.revert());
```

### scope/scope-methods/refresh

`https://animejs.com/documentation/scope/scope-methods/refresh`

> refresh() annule le Scope et le reconstruit en re-executant toutes les fonctions constructeur enregistrees.

La methode `refresh()` annule (reverts) le Scope et le reconstruit en executant toutes les fonctions constructeur enregistrees. En interne, refresh() est appele a chaque changement d'etat d'une media query. Signature: `refresh(): Scope`. Retourne l'instance Scope elle-meme, permettant le chainage de methodes.

**Faits clés**

- Signature: refresh(): Scope
- Annule le Scope et re-execute tous les constructeurs enregistres
- Appele en interne a chaque changement d'etat de media query
- Retourne le Scope (chainable)

```js
import { utils, stagger, createScope, createTimeline } from 'animejs';

const [ $button1, $button2 ] = utils.$('.refresh');

const scopeConstructor = scope => {
  const circles = utils.$('.circle');
  if (scope.i === undefined || scope.i > circles.length - 1) scope.i = 0;
  const i = scope.i++;
  
  utils.set(circles, {
    opacity: stagger([1, .25], { from: i, ease: 'out(3)' }),
  });
  
  createTimeline()
  .add(circles, {
    scale: [{ to: [.5, 1], duration: 250 }, { to: .5, duration: 750 }],
    duration: 750,
    loop: true,
  }, stagger(50, { from: i }))
  .seek(750)
}

const scope1 = createScope({ root: '.row-1' }).add(scopeConstructor);
const scope2 = createScope({ root: '.row-2' }).add(scopeConstructor);

const refreshScope1 = () => scope1.refresh();
const refreshScope2 = () => scope2.refresh();

$button1.addEventListener('click', refreshScope1);
$button2.addEventListener('click', refreshScope2);
```

### scope/scope-properties

`https://animejs.com/documentation/scope/scope-properties`

> Proprietes accessibles (lecture seule) sur une instance Scope retournee par createScope().

Proprietes en lecture seule disponibles sur une instance Scope retournee par createScope(): `data` (Object) - objet pour stocker des variables associees au scope; chaque propriete ajoutee est effacee quand le scope est reverti. `defaults` (Object) - recupere les parametres par defaut configures pour ce scope. `root` (Document | HTMLElement) - obtient l'element racine pour les operations DOM dans ce scope. `constructors` (Array<Function>) - recupere la collection de fonctions constructeur enregistrees a ce scope. `revertConstructors` (Array<Function>) - recupere la collection de fonctions constructeur de revert. `revertibles` (Array<Tickable|Animatable|Draggable|ScrollObserver|Scope>) - obtient le tableau des objets revertibles crees dans ce scope. `methods` (Object) - obtient l'objet contenant les methodes ajoutees a ce scope. `matches` (Object) - obtient l'objet contenant les resultats de correspondance des media queries courants. `mediaQueryLists` (Object) - obtient l'objet contenant les objets MediaQueryList pour ce scope.

**Faits clés**

- data: Object (variables associees au scope, effacees au revert)
- defaults: Object (parametres par defaut du scope)
- root: Document | HTMLElement (element racine des operations DOM)
- constructors: Array<Function>
- revertConstructors: Array<Function>
- revertibles: Array<Tickable|Animatable|Draggable|ScrollObserver|Scope>
- methods: Object (methodes ajoutees au scope)
- matches: Object (resultats de correspondance media queries courants)
- mediaQueryLists: Object (objets MediaQueryList du scope)
- Toutes les proprietes sont des accesseurs en lecture seule


## events

### events

`https://animejs.com/documentation/events`

> Collection de methodes utilitaires d'ecouteurs d'evenements pour declencher et controler des animations.

Le module Events fournit des methodes utilitaires d'ecouteurs d'evenements pour declencher et controler des animations: 'A collection of event listener utility methods to trigger and control animations.' Les fonctions Events sont disponibles via trois methodes d'import: depuis l'objet `events` (import { events } from 'animejs'; puis events.onScroll()), en import direct depuis le module principal (import { onScroll } from 'animejs';), ou via import de sous-chemin (import { onScroll } from 'animejs/events';). L'evenement principal documente est `onScroll` - un observateur d'evenements base sur le defilement avec de nombreuses options de configuration (settings, types de seuil, modes de synchronisation, callbacks, methodes et proprietes pour controler les animations declenchees par la position de scroll).

**Faits clés**

- Module Events = methodes utilitaires d'ecouteurs d'evenements
- 3 imports: { events } from 'animejs' / { onScroll } from 'animejs' / { onScroll } from 'animejs/events'
- Evenement principal documente: onScroll
- onScroll inclut settings, thresholds, sync modes, callbacks, methods, properties

```js
import { events } from 'animejs';
events.onScroll();
```

```js
import { onScroll } from 'animejs';
```

```js
import { onScroll } from 'animejs/events';
```

### events/onscroll

`https://animejs.com/documentation/events/onscroll`

> onScroll() cree un ScrollObserver qui declenche et synchronise des instances Timer, Animation et Timeline au scroll.

La fonction `onScroll()` cree un ScrollObserver qui declenche et synchronise des instances Timer, Animation et Timeline lors du defilement. Signature: `onScroll(parameters: Object): ScrollObserver`. Elle active les animations pilotees par le scroll en acceptant une configuration via settings, thresholds, modes de synchronisation et callbacks. Elle peut etre declaree directement dans le parametre `autoplay`. Retourne une instance ScrollObserver. Imports: depuis le module principal (import { onScroll, animate } from 'animejs';) ou en sous-chemin (import { onScroll } from 'animejs/events';). Supporte une configuration imbriquee pour settings, thresholds, sync modes et callbacks. Disponible depuis v4.0.0.

**Faits clés**

- Signature: onScroll(parameters: Object): ScrollObserver
- Declenche et synchronise Timer, Animation, Timeline au scroll
- Peut etre declare directement dans le parametre autoplay
- Retourne une instance ScrollObserver
- Disponible depuis v4.0.0
- Supporte settings, thresholds, sync modes, callbacks

```js
import { onScroll, animate } from 'animejs';
```

```js
import { onScroll } from 'animejs/events';
```

```js
import { animate, createTimer, createTimeline, utils, onScroll } from 'animejs';

const [ container ] = utils.$('.scroll-container');
const debug = true;

// Animation with onScroll
animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  autoplay: onScroll({ container, debug })
});

// Timer with onScroll
createTimer({
  duration: 2000,
  alternate: true,
  loop: true,
  onUpdate: self => {
    $timer.innerHTML = self.iterationCurrentTime
  },
  autoplay: onScroll({
    target: $timer.parentNode,
    container,
    debug
  })
});

// Timeline with onScroll
createTimeline({
  alternate: true,
  loop: true,
  autoplay: onScroll({
    target: circles[0],
    container,
    debug
  })
})
.add(circles[2], { x: '9rem' })
.add(circles[1], { x: '9rem' })
.add(circles[0], { x: '9rem' });
```

### events/onscroll/scrollobserver-settings

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings`

> Proprietes de configuration passees dans l'objet de parametres de onScroll() pour le ScrollObserver.

Les ScrollObserver settings sont des proprietes de configuration passees directement dans l'objet de parametres de onScroll() (Anime.js v4.0.0+). La structure de navigation indique cinq settings principaux: container (specifie l'element conteneur de scroll), target (definit quel element observer), debug (active la visualisation de debogage), axis (definit l'axe de scroll, 'x' ou 'y'), repeat (controle si les evenements de scroll se repetent). Chaque setting possede sa propre page de documentation detaillee. NOTE: les informations precises de type, valeurs par defaut et explications detaillees de chaque setting individuel ne sont pas presentes dans l'extrait recupere et se trouvent sur les sous-pages dediees a chaque setting.

**Faits clés**

- 5 settings principaux: container, target, debug, axis, repeat
- container: element conteneur de scroll
- target: element a observer
- debug: active la visualisation de debogage
- axis: axe de scroll ('x' ou 'y')
- repeat: controle la repetition des evenements de scroll
- Settings passes directement dans l'objet de parametres de onScroll()
- Anime.js v4.0.0+
- GOTCHA fetch: types/defauts detailles non presents dans l'extrait (sur sous-pages dediees)

```js
animate('.square', {
  x: 100,
  autoplay: onScroll({
    container: '.container',
    target: '.section',
    axis: 'y',
    enter: 'bottom top',
    leave: 'top bottom',
    sync: true,
    onEnter: () => {},
    onLeave: () => {},
    onUpdate: () => {},
  })
});
```

### events/onscroll/scrollobserver-synchronisation-modes/smooth-scroll

`https://animejs.com/documentation/events/onscroll/scrollobserver-synchronisation-modes/smooth-scroll`

> Le mode de synchronisation smooth scroll anime la progression de lecture d'une animation liee pour qu'elle rattrape la position de scroll avec un lag, via un parametre sync numerique entre 0 et 1.

Ce mode de synchronisation (sync: Number entre 0 et 1 inclus) anime la progression de lecture (playback progress) d'une animation liee pour qu'elle corresponde a la position de scroll. Plus la valeur est basse, plus la duree necessaire a l'animation pour rattraper la position de scroll courante augmente, creant un effet plus doux et decale (smoother, lagged effect).

**Faits clés**

- Parametre: sync: Number (0 a 1 inclus)
- Valeurs basses = duree de rattrapage plus longue = effet plus doux/decale
- Exemple utilise sync: .25

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: .25,
    debug: true,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-synchronisation-modes/eased-scroll

`https://animejs.com/documentation/events/onscroll/scrollobserver-synchronisation-modes/eased-scroll`

> Le mode eased scroll applique une fonction d'easing a la progression de lecture d'une animation liee relativement a la position de scroll, via sync prenant une chaine d'easing.

Le mode de synchronisation eased scroll applique une fonction d'easing a la progression de lecture (playback progress) d'une animation liee relativement a la position de scroll. Cela permet a la progression de l'animation de suivre une trajectoire d'easing courbe plutot que de correspondre lineairement au scroll. Le parametre sync accepte une chaine de fonction d'easing (ex: 'inOutCirc'). Voir la documentation du parametre ease pour les fonctions d'easing disponibles.

**Faits clés**

- Parametre: sync (string, accepte une fonction d'easing)
- Exemple utilise sync: 'inOutCirc'
- Voir documentation du parametre ease pour les easings disponibles

```js
import { animate, stagger, onScroll } from 'animejs';

animate('.square', {
  x: '12rem',
  rotate: '1turn',
  ease: 'linear',
  delay: stagger(100, { from: 'last' }),
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: 'inOutCirc',
    debug: true,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks`

> Vue d'ensemble des fonctions de callback de ScrollObserver, declenchees a des points specifiques du scroll et definies directement dans l'objet de parametres de onScroll().

Les callbacks de ScrollObserver declenchent des fonctions a des points specifiques durant le scroll. Les fonctions de callback ScrollObserver sont definies directement dans l'objet de parametres de onScroll(). Neuf fonctions de callback sont disponibles: onEnter (cible franchit le seuil d'entree), onEnterForward (entree en scrollant vers l'avant), onEnterBackward (entree en scrollant vers l'arriere), onLeave (cible quitte le seuil), onLeaveForward (sortie vers l'avant), onLeaveBackward (sortie vers l'arriere), onUpdate (pendant les mises a jour de scroll), onSyncComplete (quand la synchronisation est terminee), onResize (lors des evenements de redimensionnement). Chaque callback accepte une fonction executee a l'evenement de scroll specifie; des pages de documentation individuelles detaillent chaque callback.

**Faits clés**

- 9 callbacks: onEnter, onEnterForward, onEnterBackward, onLeave, onLeaveForward, onLeaveBackward, onUpdate, onSyncComplete, onResize
- Callbacks definis directement dans l'objet de parametres de onScroll()

```js
animate('.square', {
  x: 100,
  autoplay: onScroll({
    container: '.container',
    target: '.section',
    axis: 'y',
    enter: 'bottom top',
    leave: 'top bottom',
    sync: true,
    onEnter: () => {},
    onLeave: () => {},
    onUpdate: () => {}
  })
});
```

### events/onscroll/scrollobserver-callbacks/onenter

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onenter`

> Callback declenche chaque fois que le seuil enter est franchi; recoit l'instance ScrollObserver comme premier argument.

onEnter (Function, defaut noop) declenche une fonction chaque fois que le seuil enter est atteint. Le callback recoit l'instance ScrollObserver comme premier argument. Il se declenche quel que soit le sens du scroll, permettant d'executer une logique personnalisee a cette position de scroll.

**Faits clés**

- Signature: onEnter: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire au franchissement du seuil enter, sans distinction de direction

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let entered = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onEnter: () => $value.textContent = ++entered,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded sticky">
      <div class="large row">
        <pre class="large log row">
          <span class="label">entered</span>
          <span class="value">0</span>
        </pre>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks/onenterforward

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onenterforward`

> Callback declenche quand le seuil enter est franchi en scrollant vers l'avant; recoit l'instance ScrollObserver comme premier argument.

onEnterForward (Function, defaut noop) s'execute chaque fois que le seuil enter est franchi en scrollant dans le sens avant (forward). La fonction recoit l'instance ScrollObserver comme premier argument. Il se distingue de onEnter qui se declenche quel que soit le sens du scroll. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: onEnterForward: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire uniquement en scrollant vers l'avant
- Disponible depuis 4.0.0

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let entered = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onEnterForward: () => $value.textContent = ++entered,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded sticky">
      <div class="large row">
        <pre class="large log row">
          <span class="label">entered</span>
          <span class="value">0</span>
        </pre>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks/onenterbackward

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onenterbackward`

> Callback declenche quand le seuil enter est franchi en scrollant vers l'arriere; recoit l'instance ScrollObserver comme premier argument.

onEnterBackward (Function, defaut noop) s'execute chaque fois que le seuil enter est franchi en scrollant vers l'arriere (backward). La fonction recoit l'instance ScrollObserver comme premier argument.

**Faits clés**

- Signature: onEnterBackward: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire uniquement en scrollant vers l'arriere

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let entered = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onEnterBackward: () => $value.textContent = ++entered,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded sticky">
      <div class="large row">
        <pre class="large log row">
          <span class="label">entered</span>
          <span class="value">0</span>
        </pre>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks/onleave

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onleave`

> Callback declenche chaque fois que le seuil leave est franchi durant l'observation du scroll; recoit l'instance ScrollObserver comme premier argument.

onLeave (Function, defaut noop) se declenche chaque fois que le seuil leave est franchi durant l'observation du scroll. Il recoit l'instance ScrollObserver comme premier argument. L'exemple demontre le comptage des sorties: a chaque fois que l'element quitte la region de seuil definie, le compteur s'incremente.

**Faits clés**

- Signature: onLeave: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire au franchissement du seuil leave, sans distinction de direction

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let exits = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onLeave: () => $value.textContent = ++exits,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded sticky">
      <div class="large row">
        <pre class="large log row">
          <span class="label">exits</span>
          <span class="value">0</span>
        </pre>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks/onleaveforward

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onleaveforward`

> Callback declenche quand le seuil leave est franchi en scrollant vers l'avant; recoit l'instance ScrollObserver comme premier argument.

onLeaveForward (Function, defaut noop) se declenche chaque fois que le seuil leave est franchi en scrollant dans le sens avant (forward). Il recoit l'instance ScrollObserver comme premier argument. Pages liees: onLeave, onLeaveBackward, et documentation des thresholds.

**Faits clés**

- Signature: onLeaveForward: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire uniquement en scrollant vers l'avant

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let exits = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onLeaveForward: () => $value.textContent = ++exits,
  })
});
```

### events/onscroll/scrollobserver-callbacks/onleavebackward

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onleavebackward`

> Callback declenche quand le seuil leave est franchi en scrollant vers l'arriere; recoit l'instance ScrollObserver comme premier argument.

onLeaveBackward (Function, defaut noop) se declenche chaque fois que le seuil leave est franchi en scrollant dans le sens arriere (backward). Il recoit l'instance ScrollObserver comme premier argument. Le callback incremente un compteur a chaque fois que l'element quitte la region observee en scrollant vers l'arriere au-dela du seuil leave defini.

**Faits clés**

- Signature: onLeaveBackward: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire uniquement en scrollant vers l'arriere

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let exits = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
    onLeaveBackward: () => $value.textContent = ++exits,
  })
});
```

### events/onscroll/scrollobserver-callbacks/onupdate

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onupdate`

> Callback declenche chaque fois que la progression de l'objet lie se met a jour durant la synchronisation du scroll; recoit l'instance ScrollObserver comme premier argument.

onUpdate (Function, defaut noop) se declenche chaque fois que la progression (progress) de l'objet lie se met a jour durant la synchronisation du scroll. Il recoit l'instance ScrollObserver comme premier argument. Cela permet de reagir aux changements continus de la progression de l'animation pendant que l'utilisateur scrolle. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: onUpdate: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire a chaque mise a jour de la progression de l'objet lie durant la sync
- Disponible depuis 4.0.0

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let updates = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: .5,
    debug: true,
    onUpdate: () => $value.textContent = ++updates,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded sticky">
      <div class="large row">
        <pre class="large log row">
          <span class="label">updates</span>
          <span class="value">0</span>
        </pre>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-callbacks/onsynccomplete

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onsynccomplete`

> Callback execute quand la synchronisation de l'objet lie est terminee; recoit l'instance ScrollObserver comme premier argument.

onSyncComplete (Function, defaut noop) s'execute quand la synchronisation (synchronisation) de l'objet lie est terminee. Il se declenche durant l'animation basee sur le scroll lorsque l'animation liee finit de se synchroniser avec la position de scroll. Il recoit l'instance ScrollObserver comme premier argument. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: onSyncComplete: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire quand l'objet lie finit de se synchroniser avec la position de scroll
- Disponible depuis 4.0.0

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let completions = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom top',
    leave: 'center bottom',
    sync: .5,
    debug: true,
    onSyncComplete: () => $value.textContent = ++completions,
  })
});
```

### events/onscroll/scrollobserver-callbacks/onresize

`https://animejs.com/documentation/events/onscroll/scrollobserver-callbacks/onresize`

> Callback declenche quand le container d'un ScrollObserver est redimensionne; recoit l'instance ScrollObserver comme premier argument.

onResize (Function, defaut noop) declenche une fonction quand le container d'un ScrollObserver est redimensionne. Il recoit l'instance ScrollObserver comme premier argument. Utile pour suivre ou reagir aux changements de mise en page (layout shifts) durant l'observation du scroll. Disponible depuis la version 4.3.3.

**Faits clés**

- Signature: onResize: Function
- Defaut: noop
- Recoit l'instance ScrollObserver comme premier argument
- Fire quand les dimensions du container du ScrollObserver changent
- Disponible depuis 4.3.3

```js
import { animate, onScroll, utils } from 'animejs';

const [ $value ] = utils.$('.value');

let resizes = 0;

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom top',
    leave: 'center bottom',
    sync: .5,
    onResize: () => $value.textContent = ++resizes,
  })
});
```

### events/onscroll/scrollobserver-methods

`https://animejs.com/documentation/events/onscroll/scrollobserver-methods`

> Vue d'ensemble des methodes de l'instance ScrollObserver (link, refresh, revert) pour gerer le cycle de vie et le comportement des animations basees sur le scroll.

L'instance ScrollObserver fournit trois methodes: link() (lie des animations ou timelines a la progression du scroll), refresh() (rafraichit les calculs et mesures de l'observer) et revert() (supprime le scroll observer et annule les changements). Ces methodes gerent le cycle de vie et le comportement des animations basees sur le scroll creees via la fonction onScroll(). Les details specifiques des parametres et exemples de code complets de chaque methode sont disponibles sur leurs pages de documentation individuelles (ex: /documentation/events/onscroll/scrollobserver-methods/link).

**Faits clés**

- 3 methodes: link(), refresh(), revert()
- link() lie des animations/timelines a la progression du scroll
- refresh() rafraichit les calculs/mesures de l'observer
- revert() supprime le scroll observer et annule les changements
- Details complets par methode sur pages individuelles

```js
const scrollObserver = onScroll(parameters);
scrollObserver.link();
scrollObserver.refresh();
scrollObserver.revert();
```


## svg

### svg

`https://animejs.com/documentation/svg`

> Module svg : collection de fonctions utilitaires pour le morphing SVG, le dessin de lignes et les animations de motion path.

Le module svg fournit une collection de fonctions utilitaires pour aider au morphing SVG, au dessin de lignes (line drawing) et aux animations de motion path. Trois methodes d'import sont disponibles : (1) via l'objet svg depuis le module principal, (2) imports nommes directs, (3) import sous-chemin via 'animejs/svg'. Trois fonctions utilitaires principales sont identifiees : morphTo (morphing de forme SVG), createDrawable (cree des objets path SVG dessinables pour effets de dessin de ligne), createMotionPath (genere des animations de motion path le long de chemins SVG). Chaque fonction dispose de sa propre page de documentation dans la section SVG.

**Faits clés**

- Module svg = collection d'utilitaires pour morphing SVG, line drawing et motion path
- 3 fonctions: morphTo, createDrawable, createMotionPath
- Import 1: import { svg } from 'animejs'; puis svg.morphTo()
- Import 2: import { morphTo, createMotionPath, createDrawable } from 'animejs';
- Import 3 (subpath): import { morphTo, createMotionPath, createDrawable } from 'animejs/svg';

```js
import { svg } from 'animejs';
svg.morphTo();
svg.createMotionPath();
svg.createDrawable();
```

```js
import { morphTo, createMotionPath, createDrawable } from 'animejs';
```

```js
import { morphTo, createMotionPath, createDrawable } from 'animejs/svg';
```

### svg/morphto

`https://animejs.com/documentation/svg/morphto`

> Fonction svg.morphTo() qui anime la transition de morphing entre deux formes SVG via les attributs d ou points.

svg.morphTo(shapeTarget, precision) anime la transition de morphing entre deux formes SVG en modifiant l'attribut d des elements path ou l'attribut points des elements polyline/polygon. Le parametre shapeTarget (requis) est la forme SVG cible vers laquelle morpher (selecteur CSS, SVGPathElement, SVGPolylineElement ou SVGPolygonElement). Le parametre precision est un Number entre 0 et 1 (defaut 0.33) qui controle la densite de generation de points pour le morphing ; 0 desactive l'extrapolation. La fonction retourne un tableau contenant les valeurs string de depart et finale de la forme.

**Faits clés**

- Signature: svg.morphTo(shapeTarget, precision)
- shapeTarget (requis): CSS selector | SVGPathElement | SVGPolylineElement | SVGPolygonElement
- precision: Number 0-1, defaut 0.33 ; controle la densite de generation de points ; 0 desactive l'extrapolation
- Retour: tableau contenant les valeurs string de depart et finale de la forme
- Modifie l'attribut d (path) ou points (polyline/polygon)

```js
svg.morphTo(shapeTarget, precision);
```

```js
import { animate, svg, utils } from 'animejs';

const [ $path1, $path2 ] = utils.$('polygon');

function animateRandomPoints() {
  utils.set($path2, { points: generatePoints() });
  animate($path1, {
    points: svg.morphTo($path2),
    ease: 'inOutCirc',
    duration: 500,
    onComplete: animateRandomPoints
  });
}

animateRandomPoints();

function generatePoints() {
  const total = utils.random(4, 64);
  const r1 = utils.random(4, 56);
  const r2 = 56;
  const isOdd = n => n % 2;
  let points = '';
  for (let i = 0, l = isOdd(total) ? total + 1 : total; i < l; i++) {
    const r = isOdd(i) ? r1 : r2;
    const a = (2 * Math.PI * i / l) - Math.PI / 2;
    const x = 152 + utils.round(r * Math.cos(a), 0);
    const y = 56 + utils.round(r * Math.sin(a), 0);
    points += `${x},${y} `;
  }
  return points;
}
```

### svg/createdrawable

`https://animejs.com/documentation/svg/createdrawable`

> Fonction svg.createDrawable() qui cree un proxy SVG exposant une propriete draw pour animer le dessin de ligne.

svg.createDrawable(target) cree un objet proxy qui expose une propriete draw controlant la portion visible d'une ligne SVG. Le parametre target peut etre un selecteur CSS, un SVGLineElement, SVGPathElement, SVGPolylineElement ou SVGRectElement. La fonction retourne un tableau contenant un Proxy de l'SVGElement avec une propriete draw additionnelle. La propriete draw accepte une chaine de deux valeurs separees par un espace (debut et fin) allant de 0 a 1, definissant quelle portion du chemin est affichee. Note de performance : l'utilisation de l'attribut vector-effect: non-scaling-stroke peut impacter la performance car les facteurs d'echelle doivent etre recalcules a chaque frame d'animation.

**Faits clés**

- Signature: const [ drawable ] = svg.createDrawable(target);
- target: CSS selector | SVGLineElement | SVGPathElement | SVGPolylineElement | SVGRectElement
- Retour: tableau contenant un Proxy de l'SVGElement avec propriete draw additionnelle
- Propriete draw: chaine de 2 valeurs separees par espace (start end) de 0 a 1
- Gotcha performance: vector-effect: non-scaling-stroke recalcule les facteurs d'echelle a chaque frame

```js
const [ drawable ] = svg.createDrawable(target);
```

```js
drawable.draw = '0 1';      // Full line visible
drawable.draw = '0 .5';     // First half visible
drawable.draw = '.25 .75';  // Middle section visible
drawable.draw = '.5 1';     // Second half visible
drawable.draw = '1 1';      // No line visible (endpoint only)
```

```js
import { animate, svg, stagger } from 'animejs';

animate(svg.createDrawable('.line'), {
  draw: ['0 0', '0 1', '1 1'],
  ease: 'inOutQuad',
  duration: 2000,
  delay: stagger(100),
  loop: true
});
```

### svg/createmotionpath

`https://animejs.com/documentation/svg/createmotionpath`

> Fonction svg.createMotionPath() qui cree des objets de parametres Tween predefinis animant le long des coordonnees et de l'inclinaison d'un SVGPathElement.

svg.createMotionPath(path, offset) cree des objets de parametres Tween predefinis qui animent le long des coordonnees et de l'inclinaison d'un SVGPathElement : 'Creates pre-defined Tween parameter objects that animate along an SVGPathElement's coordinates and inclination.' Le parametre path (selecteur CSS ou SVGPathElement) est l'element path SVG a suivre ; offset (Number optionnel, defaut 0) est une valeur entre 0 et 1 pour le decalage du chemin. La fonction retourne un objet contenant trois proprietes de parametres tween : translateX (mappe a la coordonnee x du chemin), translateY (mappe a la coordonnee y du chemin), rotate (mappe a l'angle/inclinaison du chemin). Ces proprietes peuvent etre etalees (spread) directement dans les objets de configuration d'animation.

**Faits clés**

- Signature: const { translateX, translateY, rotate } = svg.createMotionPath(path, offset);
- path: CSS selector | SVGPathElement - l'element path a suivre
- offset: Number optionnel, defaut 0, valeur entre 0 et 1 pour le decalage du chemin
- Retour: objet { translateX, translateY, rotate } - parametres tween
- translateX -> coordonnee x, translateY -> coordonnee y, rotate -> angle/inclinaison du chemin
- Les proprietes peuvent etre spreadees directement dans la config d'animation

```js
const { translateX, translateY, rotate } = svg.createMotionPath(path, offset);
```

```js
import { animate, svg } from 'animejs';

const carAnimation = animate('.car', {
  ease: 'linear',
  duration: 5000,
  loop: true,
  ...svg.createMotionPath('path')
});

animate(svg.createDrawable('path'), {
  draw: '0 1',
  ease: 'linear',
  duration: 5000,
  loop: true,
});
```


## text

### text

`https://animejs.com/documentation/text`

> Module text : fonctions utilitaires pour les animations de texte (splitText, scrambleText).

Le module text fournit des fonctions utilitaires pour les animations de texte dans Anime.js. Il est disponible via plusieurs methodes d'import : import depuis le module principal (objet text), imports nommes directs, ou import sous-chemin via 'animejs/text'. Le module inclut deux utilitaires principaux d'animation de texte : splitText ('A collection of utility functions to help with text animations') et scrambleText (fonction d'animation de brouillage de texte). Chaque fonction a sa propre section de documentation detaillee couvrant settings, parametres, methodes, callbacks et proprietes.

**Faits clés**

- Module text = utilitaires d'animation de texte
- 2 fonctions: splitText, scrambleText
- Import 1: import { text } from 'animejs'; puis text.splitText()
- Import 2: import { splitText, scrambleText } from 'animejs';
- Import 3 (subpath): import { splitText, scrambleText } from 'animejs/text';

```js
import { text } from 'animejs';
text.splitText();
text.scrambleText();
```

```js
import { splitText, scrambleText } from 'animejs';
```

```js
import { splitText, scrambleText } from 'animejs/text';
```

### text/splittext/textsplitter-settings/chars

`https://animejs.com/documentation/text/splittext/textsplitter-settings/chars`

> Le reglage chars de splitText() controle si et comment chaque caractere est decoupe en element separe (span inline-block avec data-line/data-word/data-char).

Le reglage `chars` determine si et comment les caracteres individuels sont decoupes en elements separes. Quand active, chaque caractere est enveloppe dans un span avec `display: inline-block` et des attributs data suivant sa position de ligne, mot et caractere. Structure de wrapper par defaut : `<span style="display: inline-block;" data-line="0" data-word="0" data-char="0">H</span>`. Options de configuration : Boolean (`true` = comportement par defaut) ; Object (passer des split parameters pour personnaliser le wrapper, ex. `{ wrap: 'clip' }`) ; String (template HTML pour un markup de wrapper custom). Gotcha critique sur la continuite d'animation : quand on decoupe par lignes en plus des caracteres, les decoupes de lignes ecrasent les elements caracteres existants, ce qui stoppe les animations de caracteres en cours. Pour maintenir la lecture continue lors des resize, enregistrer l'animation via addEffect ; split.revert() reverte aussi les animations issues de addEffect.

**Faits clés**

- Parametre: chars
- Type: Boolean | Object | String
- Defaut: false
- Boolean true = comportement de wrapping par defaut
- Object = split parameters (ex. { wrap: 'clip' })
- String = template HTML pour wrapper custom
- Wrapper par defaut: span display:inline-block + data-line/data-word/data-char
- Gotcha: le split par lignes ecrase les elements caracteres et stoppe les animations en cours; utiliser addEffect pour continuite; split.revert() reverte aussi les animations addEffect

```js
<span style="display: inline-block;" data-line="0" data-word="0" data-char="0">H</span>
```

```js
const split = splitText(target, params);

split.addEffect(({ lines, words, chars }) => animate([lines, words, chars], {
  opacity: { from: 0 },
}));

split.revert(); // Also reverts animations from addEffect
```

```js
import { animate, splitText, stagger } from 'animejs';

const { chars } = splitText('p', {
  chars: { wrap: 'clip' },
});

animate(chars, {
  y: [
    { to: ['100%', '0%'] },
    { to: '-100%', delay: 750, ease: 'in(3)' }
  ],
  duration: 750,
  ease: 'out(3)',
  delay: stagger(50),
  loop: true,
});
```

### text/splittext/textsplitter-settings/debug

`https://animejs.com/documentation/text/splittext/textsplitter-settings/debug`

> Le parametre debug active un style CSS visuel sur les elements decoupes: contours verts (lignes), rouges (mots), bleus (caracteres).

Le parametre `debug` active un style CSS visuel sur les elements de texte decoupe a des fins de debogage. Quand active, il applique des contours colores : vert pour les lignes, rouge pour les mots, bleu pour les caracteres. Cela aide a visualiser la structure de wrapper creee par l'operation de decoupage. Usage : `splitText(target, { debug: true });`. Valeur par defaut : false.

**Faits clés**

- Parametre: debug
- Type: Boolean
- Defaut: false
- Contours colores: vert=lignes, rouge=mots, bleu=caracteres

```js
splitText(target, { debug: true });
```

```js
import { animate, splitText, stagger, utils } from 'animejs';

const [ $button ] = utils.$('button');
const [ $p ] = utils.$('p');

let debug = false;
let split;

const toggleDebug = () => {
  if (split) split.revert();
  debug = !debug;
  split = splitText($p, {
    lines: true,
    chars: true,
    words: true,
    debug: debug,
  });
}

toggleDebug();

$button.addEventListener('click', toggleDebug);
```

### text/splittext/textsplitter-settings/includespaces

`https://animejs.com/documentation/text/splittext/textsplitter-settings/includespaces`

> Le parametre includeSpaces controle si les caracteres d'espace sont enveloppes dans les elements de decoupage de splitText().

Le parametre `includeSpaces` controle si les caracteres d'espace blanc sont enveloppes dans les elements de decoupage lors de l'utilisation de splitText(). Quand active, les espaces entre mots ou lignes sont preserves comme des elements separes ; quand desactive, les espaces sont exclus du markup de decoupage. Note : ce reglage a ete introduit en version 4.1.0 et s'applique specifiquement aux operations de decoupage au niveau des mots ou la gestion des espaces devient pertinente. Valeur par defaut : false.

**Faits clés**

- Parametre: includeSpaces
- Type: Boolean
- Defaut: false
- Introduit en v4.1.0
- Active = les espaces sont preserves comme elements separes

```js
import { animate, splitText, stagger, utils } from 'animejs';

const [ $button ] = utils.$('button');
const [ $p ] = utils.$('p');

let includeSpaces = true;
let split;

const toggleSpaces = () => {
  if (split) split.revert();
  includeSpaces = !includeSpaces;
  split = splitText($p, {
    debug: true,
    includeSpaces: includeSpaces,
  });
}

toggleSpaces();

$button.addEventListener('click', toggleSpaces);
```

### text/splittext/textsplitter-settings/accessible

`https://animejs.com/documentation/text/splittext/textsplitter-settings/accessible`

> Le parametre accessible cree un element clone qui preserve la structure de l'element original decoupe pour maintenir l'accessibilite.

Le parametre `accessible` cree un element clone qui preserve la structure de l'element original decoupe, garantissant que l'accessibilite est maintenue quand le texte est decoupe en parties individuelles pour l'animation. Valeur par defaut : true. Dans l'exemple, le clone accessible est accede via `split.$target.firstChild`.

**Faits clés**

- Parametre: accessible
- Type: Boolean
- Defaut: true
- Cree un element clone preservant la structure originale pour l'accessibilite
- Clone accessible accessible via split.$target.firstChild

```js
import { createTimeline, splitText, stagger, utils } from 'animejs';

const [ $button ] = utils.$('button');
const split = splitText('p', { debug: true });
const $accessible = split.$target.firstChild;

$accessible.style.cssText = `
  opacity: 0;
  position: absolute;
  color: var(--hex-green-1);
  width: 100%;
  height: 100%;
  left: 0;
  top: 0;
  outline: currentColor dotted 1px;
`;

const showAccessibleClone = createTimeline({
  defaults: { ease: 'inOutQuad' },
})
.add($accessible, {
  opacity: 1,
  z: '-2rem',
}, 0)
.add('p', {
  rotateX: 0,
  rotateY: 60
}, 0)
.add(split.words, {
  z: '6rem',
  opacity: .75,
  outlineColor: { from: '#FFF0' },
  duration: 750,
  delay: stagger(40, { from: 'random' })
}, 0)
.init();

const toggleAccessibleClone = () => {
  showAccessibleClone.alternate().resume();
}
 
$button.addEventListener('click', toggleAccessibleClone);
```

### text/splittext/split-parameters

`https://animejs.com/documentation/text/splittext/split-parameters`

> Les split parameters (class, wrap, clone) se configurent en passant un objet aux proprietes lines, words et chars de splitText().

Les split parameters se configurent en passant un objet aux proprietes `lines`, `words` et `chars` dans splitText(). Ces parametres definissent la classe CSS, le comportement de wrap, ou le type de clone d'un decoupage. Trois split parameters sont disponibles : `class` (controle l'assignation de classe CSS aux elements decoupes), `wrap` (determine le comportement de wrap, ex. 'clip'), et `clone` (drapeau pour la gestion du type de clone). Disponible depuis 4.1.0.

**Faits clés**

- Split parameters se passent en objet a lines/words/chars
- Trois parametres: class, wrap, clone
- class = classe CSS des elements decoupes
- wrap = comportement de wrap (ex. 'clip')
- clone = type de clone
- Depuis v4.1.0

```js
splitText(target, {
  lines: true,
  words: {
    wrap: 'clip',
    class: 'split-word',
    clone: true
  },
  includeSpaces: true,
  debug: true,
});
```

### text/splittext/split-parameters/class

`https://animejs.com/documentation/text/splittext/split-parameters/class`

> Le split parameter class applique une classe CSS custom a tous les elements decoupes generes par le text splitter.

Applique une classe CSS custom a tous les elements decoupes generes par le text splitter. Cela permet de styliser les composants individuels (caracteres, mots ou lignes) via des regles CSS externes. Type : String | null, defaut : null. Disponible depuis v4.1.0. Exemple de sortie HTML : un span de classe custom enveloppe un span inline-block.

**Faits clés**

- Parametre: class
- Type: String | null
- Defaut: null
- Applique une classe CSS custom a tous les elements decoupes
- Depuis v4.1.0

```js
<span class="my-custom-class" style="display: inline-block;">
  <span style="display: inline-block;">word</span>
</span>
```

```js
import { animate, stagger, splitText } from 'animejs';

splitText('p', {
  chars: { class: 'split-char' },
});

animate('.split-char', {
  y: ['0rem', '-1rem', '0rem'],
  loop: true,
  delay: stagger(100)
});
```

```js
<div class="large centered row">
  <p class="text-xl">Custom CSS class.</p>
</div>
<div class="small row"></div>
```

```js
.split-char {
  color: var(--hex-current-1);
  background-color: var(--hex-current-3);
  outline: 1px solid var(--hex-current-2);
  border-radius: .25rem;
}
```

### text/splittext/split-parameters/wrap

`https://animejs.com/documentation/text/splittext/split-parameters/wrap`

> Le split parameter wrap ajoute un element wrapper supplementaire avec une propriete CSS overflow specifiee a tous les elements decoupes; true = 'clip'.

Ce parametre ajoute un element wrapper supplementaire avec une propriete CSS `overflow` specifiee a tous les elements decoupes. Quand mis a `true`, il vaut par defaut `'clip'`. Type : 'hidden' | 'clip' | 'visible' | 'scroll' | 'auto' | Boolean | null, defaut : null. Valeurs acceptees : chaines ('hidden', 'clip', 'visible', 'scroll', 'auto'), Boolean (true = 'clip'), ou null (pas de wrapping). Disponible depuis v4.1.0.

**Faits clés**

- Parametre: wrap
- Type: 'hidden' | 'clip' | 'visible' | 'scroll' | 'auto' | Boolean | null
- Defaut: null
- true equivaut a 'clip'
- Ajoute un wrapper avec overflow CSS specifie
- Depuis v4.1.0

```js
<span style="overflow: clip; display: inline-block;">
  <span style="display: inline-block;">word</span>
</span>
```

```js
import { animate, stagger, splitText } from 'animejs';

const { chars } = splitText('p', {
  chars: { wrap: true },
});

animate(chars, {
  y: ['75%', '0%'],
  duration: 750,
  ease: 'out(3)',
  delay: stagger(50),
  loop: true,
  alternate: true,
});
```

```js
<div class="large centered row">
  <p class="text-xl">Split and wrap text.</p>
</div>
```

### text/splittext/split-parameters/clone

`https://animejs.com/documentation/text/splittext/split-parameters/clone`

> Le split parameter clone duplique les elements decoupes dans une direction specifiee via positionnement absolu; true = 'center'.

Le parametre `clone` duplique les elements decoupes dans une direction specifiee en enveloppant lignes, mots ou caracteres. Quand active, les elements sont positionnes via positionnement absolu avec ajustement des proprietes CSS `top` et `left`. Passer `true` vaut par defaut `'center'`. Type : 'left' | 'top' | 'right' | 'bottom' | 'center' | Boolean | null, defaut : null. Structure de sortie : un span relatif inline-block contenant le span original plus un span absolu duplique (white-space: nowrap).

**Faits clés**

- Parametre: clone
- Type: 'left' | 'top' | 'right' | 'bottom' | 'center' | Boolean | null
- Defaut: null
- true equivaut a 'center'
- Duplique les elements via positionnement absolu (top/left ajustes)

```js
<span style="position: relative; display: inline-block;">
  <span style="display: inline-block;">word</span>
  <span style="position: absolute; top: 100%; left: 0px; white-space: nowrap; display: inline-block;">word</span>
</span>
```

```js
import { createTimeline, stagger, splitText } from 'animejs';

const { chars } = splitText('p', {
  chars: {
    wrap: 'clip',
    clone: 'bottom'
  },
});

createTimeline()
.add(chars, {
  y: '-100%',
  loop: true,
  loopDelay: 350,
  duration: 750,
  ease: 'inOut(2)',
}, stagger(150, { from: 'center' }));
```

```js
<div class="large centered row">
  <p class="text-xl">Split and clone text.</p>
</div>
<div class="small row"></div>
```

### text/splittext/html-template

`https://animejs.com/documentation/text/splittext/html-template`

> Un template HTML (String) enveloppe les elements decoupes; doit contenir '{value}' (requis) et peut utiliser '{i}' pour l'index.

Les templates HTML personnalises enveloppent les elements de texte decoupe (lignes, mots ou caracteres). Type : String. Doit contenir au moins une reference de variable `'{value}'` (requis) ; supporte optionnellement `'{i}'` pour l'index courant du decoupage. Le placeholder `'{value}'` est remplace par le contenu reel decoupe, tandis que `'{i}'` est remplace par l'index base zero de chaque element. La librairie applique automatiquement les styles d'affichage necessaires comme `'display: inline-block;'` sans necessiter de definition manuelle dans le template. Le template s'applique uniquement aux proprietes lines, words et chars. Omettre '{value}' cause un comportement indefini ; eviter les declarations CSS redondantes (styles d'affichage injectes automatiquement).

**Faits clés**

- Type: String
- Doit contenir '{value}' (requis)
- Supporte optionnellement '{i}' (index base zero)
- Styles d'affichage (display: inline-block) injectes automatiquement
- S'applique a lines, words et chars
- Omettre {value} = comportement indefini

```js
splitText('p', { chars: '<em class="char-{i}">{value}</em>' });
```

```js
<p>
  <em class="char-0" style="display: inline-block;">H</em>
  <em class="char-1" style="display: inline-block;">E</em>
  <em class="char-2" style="display: inline-block;">L</em>
  <em class="char-3" style="display: inline-block;">L</em>
  <em class="char-4" style="display: inline-block;">O</em>
</p>
```

```js
splitText('p', {
  chars: `<span class="char-3d word-{i}">
    <em class="face face-top">{value}</em>
    <em class="face-front">{value}</em>
    <em class="face face-bottom">{value}</em>
  </span>`,
});
```

### text/splittext/textsplitter-methods

`https://animejs.com/documentation/text/splittext/textsplitter-methods`

> L'instance TextSplitter retournee par splitText() expose trois methodes: addEffect(), revert() et refresh().

L'interface TextSplitter expose trois methodes d'instance sur les objets retournes par splitText() : addEffect() (applique des effets d'animation aux elements de texte decoupe), revert() (restaure la structure de texte originale, annule le decoupage), et refresh() (met a jour l'instance, utile apres modifications du DOM). Chaque methode opere sur l'instance TextSplitter pour modifier, restaurer ou maintenir l'etat des elements decoupes.

**Faits clés**

- Trois methodes: addEffect(), revert(), refresh()
- addEffect() = applique des effets d'animation
- revert() = annule le decoupage / restaure le texte original
- refresh() = met a jour / re-decoupe l'instance

```js
const split = splitText(target, parameters);

split.addEffect()    // Apply effects to split content
split.revert()       // Undo the text splitting
split.refresh()      // Refresh the splitter instance
```

### text/splittext/textsplitter-methods/addeffect

`https://animejs.com/documentation/text/splittext/textsplitter-methods/addeffect`

> addEffect(callback) preserve l'etat des animations/callbacks entre les decoupes (split par lignes) et permet de tout reverter via split.revert().

Signature : `addEffect(callback: Function): SplitText`. La fonction callback recoit l'instance SplitText comme premier argument et peut retourner une Animation, Timeline, Timer, ou une fonction de cleanup optionnelle. La methode preserve l'etat des animations et callbacks entre les decoupes lors d'un split par lignes, et permet de reverter toutes les animations du decoupage en une fois via split.revert(). Elle permet un enregistrement d'animation sur du texte decoupe, se rafraichissant automatiquement quand : les polices du document sont chargees (si split par lignes), ou la largeur de l'element cible change (lors d'un split par lignes). Notes : les effets s'executent en securite apres le chargement des polices et les mises a jour du DOM ; les fonctions de cleanup (retours optionnels) s'executent avant les recalculs ; appeler split.revert() supprime a la fois le decoupage de texte et les animations enregistrees.

**Faits clés**

- Signature: addEffect(callback: Function): SplitText
- Le callback recoit l'instance SplitText comme 1er argument
- Peut retourner Animation, Timeline, Timer, ou fonction de cleanup optionnelle
- Preserve l'etat animations/callbacks entre splits (split par lignes)
- Rafraichissement auto au chargement des polices et au changement de largeur (split par lignes)
- Cleanup s'execute avant chaque recalcul / entre chaque split
- split.revert() supprime le decoupage ET les animations enregistrees

```js
import { animate, utils, stagger, splitText } from 'animejs';

const colors = [];

splitText('p', {
  lines: true,
})
/* Registering an animation to the split */
.addEffect(({ lines }) => animate(lines, {
  y: ['50%', '-50%'],
  loop: true,
  alternate: true,
  delay: stagger(400),
  ease: 'inOutQuad',
}))
/* Registering a callback to the split */
.addEffect(split => {
  split.words.forEach(($el, i) => {
    const color = colors[i];
    if (color) utils.set($el, { color });
    $el.addEventListener('pointerenter', () => {
      animate($el, {
        color: utils.randomPick(['#FF4B4B', '#FFCC2A', '#B7FF54', '#57F695']),
        duration: 250,
      })
    });
  });
  return () => {
    /* Called between each split */
    split.words.forEach((w, i) => colors[i] = utils.get(w, 'color'));
  }
});
```

### text/splittext/textsplitter-methods/revert

`https://animejs.com/documentation/text/splittext/textsplitter-methods/revert`

> revert() restaure l'element cible a son etat HTML original, retire le style debug et annule toutes les animations ajoutees via addEffect().

Signature : `revert(): TextSplitter`. Cette methode restaure l'element cible a son etat HTML original. Elle retire tout style de debug et annule toutes les animations qui ont ete ajoutees via split.addEffect(). Retourne l'instance TextSplitter pour le chainage de methodes. Disponible depuis la version 4.1.0.

**Faits clés**

- Signature: revert(): TextSplitter
- Restaure l'element a son HTML original
- Retire le style debug
- Annule les animations ajoutees via addEffect()
- Retourne l'instance pour chainage
- Depuis v4.1.0

```js
import { animate, stagger, splitText, utils } from 'animejs';

const [ $button ] = utils.$('button');
const [ $p ] = utils.$('p');

const split = splitText('p', {
  words: { wrap: 'clip' },
  debug: true,
});

split.addEffect((self) => animate(self.words, {
  y: ['100%', '0%'],
  duration: 1250,
  ease: 'out(3)',
  delay: stagger(100),
  loop: true,
  alternate: true,
}));

const revertSplit = () => {
  split.revert();
  $button.setAttribute('disabled', 'true');
}

$button.addEventListener('click', revertSplit);
```

### text/splittext/textsplitter-methods/refresh

`https://animejs.com/documentation/text/splittext/textsplitter-methods/refresh`

> refresh() re-decoupe manuellement le texte en appliquant les changements de proprietes faits sur l'instance TextSplitter.

Signature : `refresh(): TextSplitter`. Re-decoupe manuellement le contenu texte en appliquant tout changement de parametre fait sur l'instance TextSplitter. Cela permet des mises a jour dynamiques du contenu HTML ou des options de configuration sans creer de nouveau splitter. Proprietes modifiables avant d'appeler refresh() : $target (HTMLElement, element cible), html (String, contenu HTML a decouper), debug (Boolean, visibilite du style debug), includeSpaces (Boolean, envelopper les espaces), accessible (Boolean, creer un clone accessible), lineTemplate (String, template HTML des lignes), wordTemplate (String, template HTML des mots), charTemplate (String, template HTML des caracteres). Retourne l'instance TextSplitter mise a jour pour le chainage.

**Faits clés**

- Signature: refresh(): TextSplitter
- Re-decoupe manuellement en appliquant les changements de parametres
- Proprietes modifiables: $target, html, debug, includeSpaces, accessible, lineTemplate, wordTemplate, charTemplate
- Permet mises a jour dynamiques sans nouveau splitter
- Retourne l'instance mise a jour pour chainage

```js
import { animate, stagger, splitText, utils } from 'animejs';

const [ $add, $remove ] = utils.$('button');
const [ $p ] = utils.$('p');

const split = splitText('p', {
  lines: { wrap: 'clip' },
  debug: true,
});

split.addEffect((self) => animate(self.words, {
  y: ['0%', '75%'],
  loop: true,
  alternate: true,
  ease: 'inOutQuad',
  delay: stagger(150)
}));

const words = ['sit', 'amet', 'consectetur', 'adipiscing', 'elit', 'tortor', 'lectus', 'aliquet'];

const addRandomWord = () => {
  split.html += ' ' + utils.randomPick(words);
  split.refresh();
}

const removeRandomWord = () => {
  const words = split.words.map(w => w.innerHTML);
  split.html = (words.splice(utils.random(0, words.length - 1), 1), words).join(' ');
  split.refresh();
}

$add.addEventListener('click', addRandomWord);
$remove.addEventListener('click', removeRandomWord);
```

### text/scrambletext/scrambletext-parameters/duration

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/duration`

> Le parametre duration de scrambleText permet de surcharger la duree d'animation calculee automatiquement, en millisecondes.

Signature: duration: Number | Function(target, index, targets) → Number. Valeur par defaut: auto-calculee. Le parametre duration permet de surcharger la duree d'animation calculee automatiquement. Quand il n'est pas defini ou mis a 0, la librairie calcule la duree a partir de la longueur du texte, du revealRate et du settleDuration. Fournir une valeur explicite (en millisecondes) donne un controle precis sur le timing de l'animation de scramble.

**Faits clés**

- Signature: duration: Number | Function(target, index, targets) → Number
- Defaut: auto-calculee
- Quand unset ou 0, duree calculee a partir de longueur du texte, revealRate et settleDuration
- Valeur explicite en millisecondes

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [500, 2000, 5000];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ duration: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Override the auto-computed animation duration with a specific value in milliseconds for precise control.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>500</button>
    <button>2000</button>
    <button>5000</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/perturbation

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/perturbation`

> Le parametre perturbation (0 a 1, defaut 0) randomise le timing de reveal de chaque caractere pour un effet plus organique.

Signature: perturbation: Number (0 to 1). Defaut: 0. Ce parametre randomise le timing de reveal de chaque caractere pour creer un effet plus organique et moins uniforme. A 0, les caracteres se revelent a intervalles regulierement espaces. Des valeurs plus elevees introduisent des decalages aleatoires sur les temps de debut et de fin de chaque caractere. A 1, le decalage peut atteindre la duree complete de settle, permettant aux caracteres de se chevaucher et de se stabiliser dans le desordre.

**Faits clés**

- Signature: perturbation: Number (0 to 1)
- Defaut: 0
- A 0 = reveal a intervalles reguliers; valeurs plus elevees = decalages aleatoires
- A 1 = decalage peut atteindre la settle duration complete, chevauchement possible

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [0, 0.5, 1];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ perturbation: values[i], cursor: '_________' }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Add random timing offsets to each character for a more organic and less uniform reveal effect.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>0</button>
    <button>0.5</button>
    <button>1</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/from

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/from`

> Le parametre from determine le point de depart de la vague de reveal du texte durant le scramble (defaut 'auto').

Signature: from: 'auto' | 'left' | 'center' | 'right' | 'random' | Number. Defaut: 'auto'. Le parametre from determine le point de depart de la vague de reveal du texte durant l'animation de scrambling. A 'auto', la direction s'adapte au contexte: reveal depuis la gauche lors d'une expansion, depuis la droite lors d'une contraction. On peut aussi specifier une direction concrete ou un index de caractere precis. Valeurs acceptees: 'auto' (resout en 'left' quand le texte grandit, 'right' quand il retrecit), 'left' (depuis la gauche), 'center' (depuis le centre), 'right' (depuis la droite), 'random' (caracteres dans un ordre aleatoire), Number (depuis un index de caractere specifique).

**Faits clés**

- Signature: from: 'auto' | 'left' | 'center' | 'right' | 'random' | Number
- Defaut: 'auto'
- 'auto' = 'left' quand le texte grandit, 'right' quand il retrecit
- Number = reveal depuis un index de caractere specifique
- 'random' = ordre aleatoire des caracteres

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const froms = ['right', 'center', 'random'];

function play(from) {
  animate($p, {
    innerHTML: scrambleText({ from }),
  });
}

play(froms[0]);

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(froms[i])));
```

```js
<div class="large row">
  <p class="text-s text-mono">Control where the reveal wave starts from, whether left, center, right, or a specific character index.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>right</button>
    <button>center</button>
    <button>random</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/reversed

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/reversed`

> Le parametre reversed (Boolean, defaut false) inverse la direction de reveal du scrambleText relativement a la valeur from.

Signature: reversed: Boolean = false. Le parametre reversed inverse la direction de reveal pour scrambleText. Quand active, il inverse le flux d'animation relativement a la valeur from. Par exemple, from: 'center' avec reversed: true revele depuis les bords vers l'interieur au lieu de depuis le centre vers l'exterieur.

**Faits clés**

- Signature: reversed: Boolean = false
- Defaut: false
- Inverse le flux d'animation relativement a la valeur from
- Exemple: from:'center' + reversed:true = reveal des bords vers l'interieur

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const [ $button ] = utils.$('button');

let reversed = false;

function play() {
  reversed = !reversed;
  $button.textContent = `reversed: ${reversed}`;
  animate($p, {
    innerHTML: scrambleText({ from: 'center', reversed }),
  });
}

play();

$button.addEventListener('click', play);
```

```js
<div class="large row">
  <p class="text-s text-mono">The animation flows in the opposite direction of the specified from value.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>reversed: false</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/seed

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/seed`

> Le parametre seed (Number, defaut 0) controle la reproductibilite de l'animation de scramble via un generateur aleatoire seede.

Signature: seed: Number. Defaut: 0 (non seede). Le parametre seed controle la reproductibilite de l'animation de scramble. Mis a une valeur non nulle, il initialise un generateur de nombres aleatoires seede qui produit la sequence de caracteres identique a chaque relecture. Mis a 0, il utilise un generateur non seede, creant des patterns de scramble differents a chaque fois.

**Faits clés**

- Type: Number
- Defaut: 0 (non seede)
- Valeur non nulle = generateur seede, sequence identique a chaque replay
- 0 = generateur non seede, pattern different a chaque fois

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [0, 42, 99];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ seed: values[i], revealRate: 2, settleRate: 2 }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Set a seed value for the random number generator.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>0</button>
    <button>42</button>
    <button>99</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-callbacks

`https://animejs.com/documentation/text/scrambletext/scrambletext-callbacks`

> Page d'apercu des callbacks de scrambleText: un seul callback disponible, onChange, specifie directement dans les parametres de scrambleText().

La page identifie un seul callback disponible pour scrambleText(): onChange, qui s'execute durant l'animation de scramble text. Les callbacks sont specifies directement dans l'Object de parametres de scrambleText(). Les details d'implementation, la signature de la fonction et les exemples de code de onChange sont disponibles sur la sous-page dediee onChange (lien 'Next' dans la navigation). Ajoutee depuis la version 4.4.0 de la librairie.

**Faits clés**

- Un seul callback: onChange
- Les callbacks sont specifies directement dans l'Object de parametres de scrambleText()
- Disponible depuis 4.4.0

```js
animate(target, {
  innerHTML: scrambleText({
    text: 'Hello World',
    chars: 'uppercase',
    onChange: () => {}
  }),
});
```

### text/scrambletext/scrambletext-callbacks/onchange

`https://animejs.com/documentation/text/scrambletext/scrambletext-callbacks/onchange`

> Le callback onChange s'execute a chaque mise a jour de caractere du scramble, recevant la chaine courante et la progression (0-1).

Nom du callback: onChange. Signature: onChange(scrambledString: String, progress: Number): void. Ce callback s'execute a chaque mise a jour de caractere du texte scramble durant toute l'animation. Il recoit la chaine scramble courante et la progression de l'animation (0-1) en parametres. Valeur par defaut: noop (no operation). Disponible depuis la version 4.4.0.

**Faits clés**

- Signature: onChange(scrambledString: String, progress: Number): void
- S'execute a chaque mise a jour de caractere scramble durant l'animation
- Recoit la chaine scramble courante et la progression (0-1)
- Defaut: noop
- Disponible depuis 4.4.0

```js
import { animate, scrambleText, utils } from 'animejs';

const [ $p ] = utils.$('p');
const [ $btn ] = utils.$('button');

const audioCtx = new AudioContext();
let soundEnabled = false;

const tickSound = () => {
  if (!soundEnabled) return;
  const t = audioCtx.currentTime;
  const o = audioCtx.createOscillator();
  const g = audioCtx.createGain();
  o.type = 'sine';
  o.frequency.setValueAtTime(4000 + Math.random() * 400, t);
  g.gain.setValueAtTime(0.001, t);
  g.gain.linearRampToValueAtTime(0.035, t + 0.001);
  g.gain.exponentialRampToValueAtTime(0.001, t + 0.003);
  o.connect(g).connect(audioCtx.destination);
  o.start(t);
  o.stop(t + 0.003);
};

$btn.addEventListener('click', () => {
  soundEnabled = !soundEnabled;
  if (soundEnabled) audioCtx.resume();
  $btn.textContent = soundEnabled ? 'Sound ON' : 'Sound OFF';
});

animate($p, {
  innerHTML: scrambleText({ onChange: tickSound }),
  loop: true,
  loopDelay: 1500,
});
```


## utilities

### utilities

`https://animejs.com/documentation/utilities`

> Le module utilities fournit une collection de fonctions utilitaires pour les taches d'animation courantes, utilisables aussi comme fonctions modificatrices.

Le module utilities fournit une collection de fonctions utilitaires pour les taches d'animation courantes, qui peuvent aussi servir de fonctions modificatrices (modifier functions). Utilitaires disponibles: stagger(), $(), get(), set(), cleanInlineStyles(), remove(), sync(), keepTime(), random(), createSeededRandom(), randomPick(), shuffle(), round(), clamp(), snap(), wrap(), mapRange(), lerp(), damp(), roundPad(), padStart(), padEnd(), degToRad(), radToDeg(), ainsi que des utilitaires chainables (Chain-able utilities). Methodes d'import: depuis le module principal via l'objet utils, par imports directs depuis 'animejs', ou depuis le sous-chemin 'animejs/utils'. La documentation detaillee de chaque fonction utilitaire est disponible sur des pages individuelles.

**Faits clés**

- Collection de fonctions utilitaires utilisables aussi comme modifier functions
- Utilitaires: stagger, $, get, set, cleanInlineStyles, remove, sync, keepTime, random, createSeededRandom, randomPick, shuffle, round, clamp, snap, wrap, mapRange, lerp, damp, roundPad, padStart, padEnd, degToRad, radToDeg + chain-able utilities
- Import via utils.X depuis 'animejs', import direct depuis 'animejs', ou depuis 'animejs/utils'

```js
import { utils } from 'animejs';

utils.stagger();
utils.$();
utils.get();
utils.set();
// Other functions
```

```js
import {
  stagger,
  $,
  get,
  set,
  // Other functions
} from 'animejs';
```

```js
import {
  stagger,
  $,
  get,
  set,
  // Other functions
} from 'animejs/utils';
```

### utilities/stagger

`https://animejs.com/documentation/utilities/stagger`

> stagger() cree des effets d'animation sequentiels en distribuant progressivement des valeurs sur plusieurs cibles, retournant une valeur basee fonction.

Signature: const functionValue = stagger(value, parameters). Le premier argument value est une valeur de stagger (numerique ou de type plage/range); le second parameters (optionnel) est un objet de configuration du comportement de stagger. Retourne une valeur basee sur une fonction (function-based value) utilisable dans les animations. L'utilitaire stagger() cree des effets d'animation sequentiels en distribuant progressivement des valeurs sur plusieurs cibles. Il genere des valeurs basees fonction qui permettent des animations coordonnees avec delais progressifs ou interpolation de valeurs. Trois cas d'usage principaux: Time staggering (distribution de delais sur les cibles), Values staggering (interpolation progressive de valeurs d'animation), Timeline staggering (sequencement d'animations dans des timelines). Options de configuration supplementaires: stagger base sur grille (grid), controles directionnels, fonctions d'easing, et modifiers pour des patterns de sequencement avances.

**Faits clés**

- Signature: stagger(value, parameters)
- value = stagger value (numerique ou range); parameters (optionnel) = objet de config
- Retourne une function-based value
- 3 cas d'usage: time staggering, values staggering, timeline staggering
- Options: grid, directional controls, easing, modifiers

```js
import { stagger } from 'animejs';

const functionValue = stagger(value, parameters);
```

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  x: '17rem',
  scale: stagger([1, .1]),
  delay: stagger(100),
});
```

```js
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
```

### utilities/stagger/time-staggering

`https://animejs.com/documentation/utilities/stagger/time-staggering`

> Le time staggering applique stagger() aux proprietes de timing (delay, duration) pour creer un effet de cascade entre cibles multiples.

Signature: stagger(value, options). Le time staggering applique la fonction stagger() aux proprietes de timing comme delay et duration dans les animations multi-cibles. Chaque cible successive recoit des valeurs de timing incrementees progressivement, creant un effet de cascade ou les animations commencent et progressent a intervalles differents. Dans l'exemple, chaque delay augmente de 100ms et chaque duration augmente de 200ms (en partant de 500ms via start: 500). Resultat: 1ere cible delay 0ms/duration 500ms; 2e delay 100ms/duration 700ms; 3e delay 200ms/duration 900ms; 4e delay 300ms/duration 1100ms.

**Faits clés**

- Signature: stagger(value, options)
- S'applique aux proprietes de timing (delay, duration)
- Chaque cible successive recoit des valeurs incrementees
- Option start: valeur de depart (ex: start: 500 => duration debute a 500ms)
- Resultat exemple: delay 0/100/200/300ms, duration 500/700/900/1100ms

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  x: '17rem',
  delay: stagger(100),
  duration: stagger(200, { start: 500 }),
  loop: true,
  alternate: true
});
```

### utilities/stagger/values-staggering

`https://animejs.com/documentation/utilities/stagger/values-staggering`

> Le values staggering donne a chaque cible une valeur de propriete incrementee differemment, creant un effet de cascade sur les valeurs.

Le values staggering permet a chaque cible d'une animation multi-cibles de recevoir des valeurs de propriete progressivement differentes. La fonction stagger() retourne une valeur basee fonction utilisable dans n'importe quelle propriete animable, creant un effet de cascade sur plusieurs elements. Syntaxe: stagger(value) ou stagger([startValue, endValue]), ou value peut etre une valeur numerique, une plage, ou autres types de valeurs stagger supportes. Toutes les proprietes animables des tweens acceptent des function-based values, permettant l'usage de la fonction retournee par stagger() dans les animations multi-cibles. Chaque cible suivante recoit une valeur incrementee, creant un effet d'animation distribue. Dans l'exemple, chaque element .square recoit une position y et une valeur rotate differentes, distribuees sur les plages specifiees.

**Faits clés**

- Syntaxe: stagger(value) ou stagger([startValue, endValue])
- value peut etre numerique, range, ou autres types stagger
- Toutes les proprietes animables des tweens acceptent function-based values
- Chaque cible suivante recoit une valeur incrementee
- Utilisable avec { from: stagger(...) } sur une propriete (ex: rotate)

```js
stagger(value)
stagger([startValue, endValue])
```

```js
import { animate, stagger } from 'animejs';

const animation = animate('.square', {
  y: stagger(['-2.75rem', '2.75rem']),
  rotate: { from: stagger('-.125turn') },
  loop: true,
  alternate: true
});
```

```js
<div class="small justified row">
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
</div>
```

### utilities/stagger/timeline-positions-staggering

`https://animejs.com/documentation/utilities/stagger/timeline-positions-staggering`

> stagger() s'integre au positionnement add() d'une timeline pour creer des animations staggered, avec callbacks egalement staggered par cible.

La fonction stagger() s'integre au positionnement add() d'une timeline pour creer des animations staggered sur plusieurs cibles. Chaque cible recoit sa propre animation positionnee a intervalles determines par la configuration de stagger. Syntaxe: stagger(interval, { start: position }). La propriete start definit la position de stagger initiale et accepte les memes valeurs que les arguments de position de add() d'une timeline. Caracteristiques cles: positionnement base fonction (l'argument de position de add() accepte la fonction retournee par stagger()); animations par cible (chaque cible cree sa propre animation a une position staggered dans la timeline); callbacks staggered (les callbacks definis sur l'animation staggered se declenchent pour chaque cible a leurs temps staggered respectifs). L'exemple stagger deux elements de 500ms chacun, en partant du label 'circle completes' moins 500ms.

**Faits clés**

- Syntaxe: stagger(interval, { start: position })
- start accepte les memes valeurs que les arguments de position de add()
- L'argument position de add() accepte la fonction retournee par stagger()
- Chaque cible cree sa propre animation a position staggered
- Les callbacks (ex: onComplete) sont egalement staggered par cible
- start supporte les references de label avec offset (ex: 'circle completes-=500')

```js
stagger(interval, { start: position })
```

```js
import { createTimeline, stagger, utils } from 'animejs';

const tl = createTimeline();

const onComplete = ({ targets }) => {
  utils.set(targets, { color: 'var(--hex-red)' });
}

tl
  .add('.circle', { x: '15rem', onComplete })
  .label('circle completes')
  .add(['.triangle', '.square'], {
    x: '15rem',
    onComplete, // Callbacks are also staggered
  }, stagger(500, { start: 'circle completes-=500' }));
```

### utilities/stagger/stagger-value-types

`https://animejs.com/documentation/utilities/stagger/stagger-value-types`

> stagger() accepte deux types de valeurs (numerical et range) qui determinent comment espacer les animations sur plusieurs elements.

Selon la documentation, deux types de valeurs sont acceptes par stagger(): Numerical (valeurs numeriques) et Range (valeurs de plage). La fonction stagger() accepte une 'Stagger Value' qui determine comment espacer les animations sur plusieurs elements. Cette valeur peut etre exprimee de deux facons differentes selon le cas d'usage. La page 'Stagger value types' fait partie de la section Utilities et precede les pages detaillees: Numerical value (details sur les valeurs numeriques) et Range value (details sur les valeurs de plage). L'exemple illustre la signature avec un objet de parametres incluant start, from, reversed, ease et grid.

**Faits clés**

- Deux types de valeurs: Numerical (numerique) et Range (plage)
- La 'Stagger Value' determine comment espacer les animations sur plusieurs elements
- Sous-pages detaillees: Numerical value et Range value
- Options de parametres illustrees: start, from, reversed, ease, grid

```js
stagger(
  '1rem',
  {
    start: 100,
    from: 2,
    reversed: false,
    ease: 'outQuad',
    grid: [8, 8],
  }
);
```

### utilities/stagger/stagger-value-types/numerical-value

`https://animejs.com/documentation/utilities/stagger/stagger-value-types/numerical-value`

> Une valeur numerique passee a stagger() definit l'increment ajoute a chaque element successif.

Le type de valeur numerique pour stagger() accepte un Number ou une String contenant au moins un Number. Cette valeur definit le montant d'increment applique a chaque element echelonne : chaque element successif augmente la valeur de ce montant. La valeur peut etre un nombre simple ou inclure une unite sous forme de chaine, permettant un echelonnement flexible aussi bien sur les proprietes CSS que sur les parametres temporels (ex : delay).

**Faits clés**

- Type accepte : Number | String contenant au moins un Number
- Definit le montant d'increment applique a chaque element successif
- Peut inclure une unite via une String (ex '5.75rem')

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  // Increase translateX by 5.75rem for each elements
  x: stagger('5.75rem'),
  // Increase delay by 100ms for each elements
  delay: stagger(100)
});
```

```js
<div class="small row">
  <div class="square"></div>
  <div class="padded label">x: 0rem      delay: 0ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="padded label">x: 5.75rem   delay: 100ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="padded label">x: 11.5rem   delay: 200ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="padded label">x: 17.25rem  delay: 300ms</div>
</div>
```

### utilities/stagger/stagger-value-types/range-value

`https://animejs.com/documentation/utilities/stagger/stagger-value-types/range-value`

> Une valeur de plage [start, end] passee a stagger() distribue les valeurs uniformement entre deux bornes numeriques.

La valeur de plage (range value) accepte un tableau de deux bornes — numeriques ou chaines avec unite — sous la forme stagger([Number|String, Number|String]). Anime.js distribue les valeurs uniformement entre ces deux valeurs numeriques et genere automatiquement des valeurs interpolees pour chaque element anime. Cette approche distribue les valeurs sequentiellement a travers les elements cibles, utile pour creer des animations en cascade avec des intensites ou des timings variables.

**Faits clés**

- Signature : stagger([Number|String, Number|String])
- Distribue les valeurs uniformement entre les deux bornes
- Accepte des nombres ou des chaines avec unite

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  y: stagger(['2.75rem', '-2.75rem']),
  delay: stagger([0, 500]),
});
```

```js
<div class="small justified row">
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
</div>
```

### utilities/stagger/stagger-parameters

`https://animejs.com/documentation/utilities/stagger/stagger-parameters`

> Le second argument de stagger() est un objet de parametres qui controle comment l'echelonnement est applique.

La fonction stagger() accepte un second argument contenant un objet de parametres de configuration qui controle la facon dont l'echelonnement est applique aux animations. Les parametres disponibles sont : start (decalage de timing initial), from (point d'origine des calculs de stagger), reversed (booleen pour inverser la direction), ease (applique une fonction d'easing au timing du stagger), grid (active l'echelonnement 2D base sur une grille), axis (direction du stagger de grille horizontal/vertical), modifier (transforme les valeurs de stagger), use (selectionne quelles valeurs de stagger appliquer), total (definit la longueur totale du stagger) et jitter (introduit de l'aleatoire dans le stagger, marque NEW). Chaque parametre dispose de sa propre page de documentation dediee.

**Faits clés**

- stagger() accepte un 2e argument : objet de parametres
- Parametres : start, from, reversed, ease, grid, axis, modifier, use, total, jitter
- jitter est marque NEW

```js
stagger(
  '1rem',
  {
    start: 100,
    from: 2,
    reversed: false,
    ease: 'outQuad',
    grid: [8, 8],
  }
);
```

### utilities/stagger/stagger-parameters/stagger-start

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-start`

> start ajoute un decalage de base fixe a chaque valeur echelonnee, decalant toute la sequence.

Le parametre start (Number, defaut 0 ; peut etre une position temporelle de timeline dans un contexte timeline) etablit un decalage de base applique a tous les calculs echelonnes. Il ajoute une valeur fixe au resultat calcule de chaque element, decalant toute la sequence de stagger vers l'avant. Ainsi toutes les valeurs echelonnees commencent a partir de ce point defini plutot que de zero.

**Faits clés**

- Type : Number (ou position temporelle de timeline en contexte timeline)
- Defaut : 0
- Ajoute une valeur fixe a chaque valeur echelonnee
- Exemple : 1er x:14rem delay:500ms ; 2e x:15rem delay:600ms ; 3e x:16rem delay:700ms ; 4e x:17rem delay:800ms

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  x: stagger('1rem', { start: 14 }), // adds 14 to the staggered value
  delay: stagger(100, { start: 500 }), // adds 500 to the staggered value
});
```

### utilities/stagger/stagger-parameters/stagger-from

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-from`

> from definit la position de depart de l'effet d'echelonnement parmi les elements cibles.

Le parametre from (Number | String | [x, y] | [x, y, z], defaut 0) definit la position de depart de l'effet de stagger parmi les elements cibles. Valeurs acceptees : un Number = index de depart de l'effet ; 'first' = equivalent a l'index 0 ; 'center' = demarre l'effet depuis le centre ; 'last' = demarre depuis le dernier element ; 'random' = randomise l'ordre des valeurs echelonnees ; [x, y] = tableau de coordonnees normalisees (0 a 1) pour controler l'origine d'une grille 2D ; [x, y, z] = coordonnees normalisees (0 a 1) pour l'origine d'une grille 3D.

**Faits clés**

- Type : Number | String | [x, y] | [x, y, z]
- Defaut : 0
- Valeurs String : 'first', 'center', 'last', 'random'
- Number = index de depart ; [x,y] / [x,y,z] = coordonnees normalisees 0-1 pour origine de grille

```js
import { createTimeline, stagger } from 'animejs';

const tl = createTimeline({
  loop: true,
  defaults: { duration: 500 },
  delay: 500,
  loopDelay: 500
})
.add('.row:nth-child(1) .square:nth-child(8)', { color: '#FFF', scale: 1.2 })
.add('.row:nth-child(1) .square', {
  scale: 0,
  delay: stagger(25, { from: 7 }),
}, '<')
.add('.row:nth-child(2) .square:first-child', { color: '#FFF', scale: 1.2 })
.add('.row:nth-child(2) .square', {
  scale: 0,
  delay: stagger(25, { from: 'first' }),
}, '<')
.add('.row:nth-child(3) .square:nth-child(6)', { color: '#FFF', scale: 1.2 })
.add('.row:nth-child(3) .square', {
  scale: 0,
  delay: stagger(25, { from: 'center' }),
}, '<')
```

### utilities/stagger/stagger-parameters/stagger-reversed

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-reversed`

> reversed applique la sequence d'echelonnement en ordre inverse a travers les elements cibles.

Le parametre reversed (Boolean, defaut false, disponible depuis v2.0.0) controle si la sequence de stagger doit etre appliquee en ordre inverse. Lorsqu'il est active (reversed: true), les calculs de delai operent a rebours a travers les elements cibles : le dernier element commence en premier avec un delai nul, tandis que le premier element subit le delai accumule maximal, inversant la progression normale du stagger.

**Faits clés**

- Type : Boolean
- Defaut : false
- Disponible depuis v2.0.0
- true : le dernier element demarre en premier (delai 0), le premier a le delai max

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  translateX: '17rem',
  delay: stagger(100, { reversed: true }),
});
```

```js
<div class="small row">
  <div class="square"></div>
  <div class="label padded">delay: 300ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="label padded">delay: 200ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="label padded">delay: 100ms</div>
</div>
<div class="small row">
  <div class="square"></div>
  <div class="label padded">delay: 0ms</div>
</div>
```

### utilities/stagger/stagger-parameters/stagger-ease

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-ease`

> ease applique une courbe d'easing a la distribution des valeurs echelonnees a travers les elements.

Le parametre ease (fonction d'easing, defaut 'linear') applique une courbe d'easing a la facon dont les valeurs echelonnees sont distribuees a travers les elements animes. Il controle le motif d'acceleration et de deceleration de l'effet de stagger lui-meme, plutot que l'animation de chaque element individuel. Toute fonction d'easing valide supportee par le parametre ease de la librairie est acceptee (cubic-bezier, spring, steps, etc.).

**Faits clés**

- Type : fonction d'easing
- Defaut : 'linear'
- Affecte la distribution des valeurs de stagger, pas l'animation de chaque element
- Accepte cubic-bezier, spring, steps, etc.

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  y: stagger(['2.75rem', '-2.75rem'], { ease: 'inOut(3)' }),
  delay: stagger([0, 500], { ease: 'inOut(3)', from: 'center' }),
  ease: 'inOutQuad',
  duration: 500,
  loop: true,
  alternate: true,
});
```

```js
<div class="small justified row">
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
</div>
```

### utilities/stagger/stagger-parameters/stagger-grid

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-grid`

> grid distribue les valeurs echelonnees a travers une grille 2D ou 3D.

Le parametre grid (type [Number, Number] | [Number, Number, Number] | boolean, defaut null) distribue les valeurs de stagger a travers une disposition en grille 2D ou 3D. Il accepte des dimensions explicites sous forme de tableau ou active un calcul automatique de la grille a partir des positions des elements cibles. Valeurs acceptees : [columns, rows] = grille 2D explicite avec dimensions specifiees ; [columns, rows, depth] = grille 3D explicite ; true = mode auto-grille calculant les dimensions depuis les positions des elements (2D pour des objets {x, y}, 3D quand z est present). Notes de version : v4.4.0 introduit le mode auto-grille (grid: true) qui calcule les dimensions depuis les positions des elements au lieu de les exiger explicitement ; v4.5.0 supporte les dimensions 3D explicites et l'auto-grille bascule en 3D quand les cibles exposent des valeurs z numeriques (utile avec l'adaptateur Three.js).

**Faits clés**

- Type : [Number, Number] | [Number, Number, Number] | boolean
- Defaut : null
- [columns, rows] = grille 2D ; [columns, rows, depth] = grille 3D ; true = auto-grille
- v4.4.0 : auto-grille (grid: true)
- v4.5.0 : 3D explicite + auto-grille 3D quand z numerique present (Three.js adapter)

```js
import { animate, stagger } from 'animejs';

const $squares = utils.$('.square');

function animateGrid() {
  animate($squares, {
    scale: [
      { to: [0, 1.25] },
      { to: 0 }
    ],
    boxShadow: [
      { to: '0 0 1rem 0 currentColor' },
      { to: '0 0 0rem 0 currentColor' }
    ],
    delay: stagger(100, {
      grid: [11, 4],
      from: utils.random(0, 11 * 4)
    }),
    onComplete: animateGrid
  });
}

animateGrid();
```

### utilities/stagger/stagger-parameters/stagger-grid-axis

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-grid-axis`

> axis restreint la direction des animations de grille echelonnees a un axe specifique (x, y ou z).

Le parametre axis ('x' | 'y' | 'z') restreint la direction des animations de grille echelonnees a un axe specifique. 'x' contraint le mouvement au positionnement horizontal, 'y' au positionnement vertical, et 'z' s'applique exclusivement aux grilles 3D lors de l'utilisation de l'adaptateur Three.js. Note : l'axe 'z' s'applique uniquement aux grilles 3D, utile pour animer dans l'espace 3D avec l'adaptateur Three.js.

**Faits clés**

- Type : 'x' | 'y' | 'z'
- 'x' = horizontal, 'y' = vertical, 'z' = grilles 3D uniquement (Three.js adapter)
- S'utilise avec le parametre grid

```js
import { animate, stagger, utils } from 'animejs';

const grid = [11, 4];
const $squares = utils.$('.square');

function animateGrid() {
  const from = utils.random(0, 11 * 4);
  animate($squares, {
    translateX: [
      { to: stagger('-.75rem', { grid, from, axis: 'x' }) },
      { to: 0, ease: 'inOutQuad', },
    ],
    translateY: [
      { to: stagger('-.75rem', { grid, from, axis: 'y' }) },
      { to: 0, ease: 'inOutQuad' },
    ],
    opacity: [
      { to: .5 },
      { to: 1 }
    ],
    delay: stagger(85, { grid, from }),
    onComplete: animateGrid
  });
}

animateGrid();
```

### utilities/stagger/stagger-parameters/stagger-modifier

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-modifier`

> modifier est une fonction qui transforme la valeur echelonnee retournee par stagger().

Le parametre modifier (Function) accepte une fonction qui transforme la valeur echelonnee retournee par stagger(). Cette fonction recoit la valeur numerique animee courante (value, Number) et doit retourner un Number ou une String pour modifier le comportement de l'animation echelonnee. Dans l'exemple, le modifier transforme les valeurs numeriques de stagger en valeurs CSS de type boxShadow en multipliant la valeur et en l'incorporant dans la syntaxe de l'ombre.

**Faits clés**

- Type : Function
- Recoit value (Number) = valeur numerique animee courante
- Retourne Number | String
- Permet de transformer des valeurs numeriques en chaines CSS (ex boxShadow)

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  boxShadow: [
    { to: stagger([1, .25], {
        modifier: v => `0 0 ${v * 30}px ${v * 20}px currentColor`,
        from: 'center'
      })
    },
    { to: 0 },
  ],
  delay: stagger(100, { from: 'center' }),
  loop: true
});
```

### utilities/stagger/stagger-parameters/stagger-use

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-use`

> use definit un ordre d'echelonnement personnalise base sur une propriete/attribut des elements cibles.

Le parametre use (String, defaut null) definit un ordre de stagger personnalise en referencant une propriete ou un attribut des elements cibles, au lieu de suivre leur ordre naturel dans le DOM. La propriete/attribut reference doit contenir des nombres sequentiels commencant a 0. Gotcha : lorsque l'on utilise use avec les parametres from, reversed ou ease, il faut explicitement definir une valeur total si l'index personnalise le plus eleve est inferieur au nombre reel de cibles echelonnees.

**Faits clés**

- Type : String
- Defaut : null
- Reference une propriete/attribut contenant des nombres sequentiels commencant a 0
- Avec from/reversed/ease : definir total si l'index max custom < nombre reel de cibles

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  x: '17rem',
  rotate: 90,
  delay: stagger(250, { use: 'data-index' }),
});
```

```js
<div class="small row">
  <div class="square" data-index="2"></div>
  <div class="padded label">data-index="2"</div>
</div>
<div class="small row">
  <div class="square" data-index="0"></div>
  <div class="padded label">data-index="0"</div>
</div>
<div class="small row">
  <div class="square" data-index="3"></div>
  <div class="padded label">data-index="3"</div>
</div>
<div class="small row">
  <div class="square" data-index="1"></div>
  <div class="padded label">data-index="1"</div>
</div>
```

### utilities/stagger/stagger-parameters/stagger-total

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-total`

> total fixe une longueur d'echelonnement personnalisee au lieu du nombre reel de cibles.

Le parametre total (Number, defaut null) etablit une longueur d'echelonnement personnalisee au lieu de s'appuyer sur le nombre reel de cibles echelonnees. Il est particulierement utile lorsque la valeur maximale d'un ordre personnalise (defini via le parametre use) est inferieure au nombre reel de cibles echelonnees, surtout en utilisant les parametres from, reversed ou ease. Parametres associes : use, from, reversed, ease.

**Faits clés**

- Type : Number
- Defaut : null
- Fixe une longueur de stagger personnalisee au lieu du nombre reel de cibles
- Utile quand l'index max d'un ordre custom (use) < nombre de cibles, avec from/reversed/ease
- Parametres associes : use, from, reversed, ease

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  x: '17rem',
  rotate: 90,
  delay: stagger(250, { use: 'data-index', total: 2, reversed: true }),
});
```

```js
<div class="small row">
  <div class="square" data-index="0"></div>
  <div class="padded label">data-index="0"</div>
</div>
<div class="small row">
  <div class="square" data-index="0"></div>
  <div class="padded label">data-index="0"</div>
</div>
<div class="small row">
  <div class="square" data-index="1"></div>
  <div class="padded label">data-index="1"</div>
</div>
<div class="small row">
  <div class="square" data-index="1"></div>
  <div class="padded label">data-index="1"</div>
</div>
```

### utilities/stagger/stagger-parameters/stagger-jitter

`https://animejs.com/documentation/utilities/stagger/stagger-parameters/stagger-jitter`

> jitter ajoute un decalage aleatoire a chaque valeur echelonnee.

Le parametre jitter (Number | [Number, Number] | null, defaut null) ajoute un decalage aleatoire a chaque valeur echelonnee. Le decalage peut etre un nombre unique, applique symetriquement comme [-value, +value], ou un tableau de plage qui interpole de la premiere valeur (cible la plus proche) a la seconde (cible la plus eloignee). Reproductibilite : passer un parametre seed rend le jitter reproductible ; le meme seed affecte aussi from: 'random'. Valeurs de seed : false = decalages aleatoires varient a chaque execution (defaut) ; true = seed avec 0 ; <Number> = utilise le nombre directement comme seed.

**Faits clés**

- Type : Number | [Number, Number] | null
- Defaut : null
- Number = decalage aleatoire dans [-value, +value]
- [Number, Number] = rampe du premier (cible la plus proche) au second (la plus eloignee)
- seed : false (defaut, varie), true (seed 0), Number (seed direct) ; meme seed affecte from:'random'

```js
import { animate, stagger } from 'animejs';

animate('.square', {
  y: ['-2.75rem', 2.75],
  duration: 500,
  delay: stagger(100, { jitter: 100 }),
  ease: 'inOutQuad',
  alternate: true,
  loop: true,
});
```

### utilities/dollar-sign

`https://animejs.com/documentation/utilities/dollar-sign`

> utils.$() converts a CSS selector or DOM Elements into an array of elements, scope-aware alternative to document.querySelectorAll().

Le `$()` (utils.$()) convertit un parametre `targets` en un tableau d'elements, comme alternative a `document.querySelectorAll()`. Lorsqu'il est utilise dans un Scope (createScope), il interroge a l'interieur de l'element racine (`root`) du Scope plutot que dans le document entier : il appelle effectivement `root.querySelectorAll()` au lieu de la requete globale, respectant ainsi la frontiere du Scope. Le parametre `targets` accepte un selecteur CSS ou des DOM Elements. Il retourne un Array d'HTMLElement, SVGElement ou SVGGeometryElement.

**Faits clés**

- Signature: const targetsArray = utils.$(targets);
- Parametre targets: accepte un selecteur CSS ou des DOM Elements
- Retourne un Array d'HTMLElement, SVGElement ou SVGGeometryElement
- Dans un Scope, interroge le root du Scope (root.querySelectorAll()) au lieu du document global

```js
const targetsArray = utils.$(targets);
```

```js
import { utils, createScope } from 'animejs';

// Targets all the '.square' elements
utils.$('.square').forEach($square => {
  utils.set($square, { scale: .5 });
});

createScope({ root: '.row:nth-child(2)' }).add(() => {
  // Limits the selection to '.row:nth-child(2) .square'
  utils.$('.square').forEach($square => {
    utils.set($square, { rotate: 45 });
  });
});
```

### utilities/get

`https://animejs.com/documentation/utilities/get`

> utils.get() retourne la valeur courante d'une propriete d'une cible, avec conversion ou suppression d'unite optionnelle.

`utils.get(target, property, unit)` retourne la valeur courante d'une propriete d'une cible, avec conversion ou suppression d'unite optionnelle. `target` (Targets) est l'element cible ; `property` (String) est un nom de propriete valide de la cible ; `unit` (optionnel, String | Boolean) : si `false`, l'unite est supprimee ; si une chaine est fournie, la valeur est convertie vers cette unite. La valeur de retour est une String quand la cible est un HTMLElement/SVGElement et que `unit` n'est ni `false` ni une chaine d'unite valide, et un Number quand `unit` vaut `false`. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: const value = utils.get(target, property, unit);
- target (Targets): l'element cible
- property (String): un nom de propriete valide de la cible
- unit (optionnel, String | Boolean): false supprime l'unite ; une chaine convertit vers cette unite
- Retourne une String par defaut, un Number quand unit vaut false
- Disponible depuis la version 2.0.0

```js
const value = utils.get(target, property, unit);
```

```js
import { animate, utils } from 'animejs';

const [ $raw, $rem, $num ] = utils.$('.value');
const [ $sq1, $sq2, $sq3 ] = utils.$('.square');

const getValues = () => {
  // Return the raw parsed value (string with px)
  $raw.textContent = utils.get($sq1, 'x');
  // Return the converted value with unit (string with rem)
  $rem.textContent = utils.get($sq2, 'x', 'rem');
  // Return the raw value with its unit removed (number)
  $num.textContent = utils.get($sq3, 'x', false);
}

animate('.square', {
  x: 270,
  loop: true,
  alternate: true,
  onUpdate: getValues
});
```

### utilities/set

`https://animejs.com/documentation/utilities/set`

> utils.set() applique immediatement (sans animation) une ou plusieurs valeurs de proprietes a une ou plusieurs cibles et retourne un objet Animation revertable.

`utils.set(targets, properties)` definit immediatement une ou plusieurs valeurs de proprietes sur une ou plusieurs cibles, applique les changements de style/propriete sans animation, avec effet instantane. `targets` (Targets) sont la/les cible(s) ; `properties` (Object) est un objet de proprietes et valeurs valides de la cible. Retourne un objet Animation qui inclut une methode `.revert()` pour annuler les changements. Notes : pour mettre a jour repetitivement les memes proprietes sur les memes cibles, un Animatable est recommande pour de meilleures performances ; `utils.set()` ne fonctionnera pas si on essaie de definir un attribut sur un element DOM ou SVG qui n'est pas deja defini sur l'element.

**Faits clés**

- Signature: const setter = utils.set(targets, properties);
- targets (Targets): la/les cible(s)
- properties (Object): objet de proprietes et valeurs valides de la cible
- Retourne un objet Animation avec une methode .revert()
- Performance: pour mises a jour repetees, preferer un Animatable
- Ne fonctionne pas pour definir un attribut DOM/SVG non deja defini sur l'element

```js
const setter = utils.set(targets, properties);
```

```js
import { utils, stagger } from 'animejs';

const setter = utils.set(squares, {
  borderRadius: '50%',
  y: () => utils.random(-1, 1) + 'rem',
  scale: stagger(.1, { start: .25, ease: 'out' }),
  color: () => `var(--hex-${utils.randomPick(colors)})`
});

// Later: revert changes
setter.revert();
```

### utilities/clean-inline-styles

`https://animejs.com/documentation/utilities/clean-inline-styles`

> utils.cleanInlineStyles() retire tous les styles CSS inline ajoutes par une instance Animation/Timeline donnee, en preservant les autres styles.

`utils.cleanInlineStyles(instance)` retire tous les styles CSS inline qui ont ete ajoutes par une instance Animation ou Timeline specifiee. Particulierement utile comme callback `onComplete()` pour nettoyer les styles apres la fin d'une animation. Important : il ne retire que les styles appliques par cette instance particuliere — les autres styles restent intacts (par exemple ceux poses par `utils.set()`). Le parametre `instance` (Animation | Timeline) est l'instance dont les styles inline doivent etre retires. Retourne l'instance Animation ou Timeline passee.

**Faits clés**

- Signature: const cleanedInstance = utils.cleanInlineStyles(instance);
- instance (Animation | Timeline): l'instance dont les styles inline sont retires
- Retourne l'instance Animation ou Timeline passee
- Ne retire que les styles inline appliques par cette instance precise, preserve les autres
- Usage typique: callback onComplete()

```js
const cleanedInstance = utils.cleanInlineStyles(instance);
```

```js
import { animate, utils } from 'animejs';

utils.set('.square', { scale: .75 });

animate('.keep-styles', {
  x: '17rem',
  borderRadius: '50%',
});

animate('.clean-styles', {
  x: '17rem',
  borderRadius: '50%',
  // This removes the translateX and borderRadius inline styles
  // But keeps the scale previously added outside of this animation
  onComplete: utils.cleanInlineStyles
});
```

### utilities/remove

`https://animejs.com/documentation/utilities/remove`

> utils.remove() retire une ou plusieurs cibles des animations actives, annulant si necessaire les Animation/Timeline qui les referencent ; peut cibler une instance et/ou propriete specifiques.

`utils.remove(targets, instance, propertyName)` retire une ou plusieurs cibles des animations actives. Quand des cibles sont retirees, toute Animation ou Timeline les referencant est annulee si necessaire. `targets` (Targets, requis) sont les elements a retirer des animations ; `instance` (Animation | Timeline, optionnel) est une instance specifique d'animation ou de timeline ; `propertyName` (String, optionnel) est un nom de propriete animable specifique. On peut retirer des cibles globalement ou cibler des instances et proprietes specifiques pour un controle plus fin du nettoyage. Retourne un Array des elements cibles retires. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: const removed = utils.remove(targets, instance, propertyName);
- targets (Targets, requis): elements a retirer des animations
- instance (Animation | Timeline, optionnel): instance specifique
- propertyName (String, optionnel): nom de propriete animable specifique
- Annule les Animation/Timeline referencant les cibles si necessaire
- Retourne un Array des elements cibles retires
- Disponible depuis la version 2.0.0

```js
const removed = utils.remove(targets, instance, propertyName);
```

```js
import { animate, utils } from 'animejs';

let updates = 0;

const [ $removeFirstButton ] = utils.$('.remove-1');
const [ $removeSecondButton ] = utils.$('.remove-2');
const [ $updates ] = utils.$('.value');

const animation = animate('.square', {
  x: '17rem',
  rotate: 360,
  alternate: true,
  loop: true,
  onUpdate: () => {
    $updates.textContent = updates++;
  }
});

$removeFirstButton.onclick = () => {
  utils.remove('.row:nth-child(1) .square');
}

$removeSecondButton.onclick = () => {
  utils.remove('.row:nth-child(2) .square', animation, 'x');
}
```

### utilities/sync

`https://animejs.com/documentation/utilities/sync`

> utils.sync() execute une fonction callback en synchronisation avec la boucle du moteur (engine loop) et retourne un Timer.

`utils.sync(callback)` execute une fonction callback en synchronisation avec la boucle du moteur (engine loop). Permet d'executer du code au meme timing que le cycle de mise a jour interne d'Anime.js, garantissant que les operations restent synchronisees avec les animations actives. Le parametre `callback` (Function) est une fonction a executer synchronisee avec la boucle du moteur. Retourne un Timer. L'exemple demontre la synchronisation des mises a jour de vitesse d'animation avec la boucle du moteur, assurant des changements de propriete fluides sans desalignement de timing.

**Faits clés**

- Signature: utils.sync(function)
- callback (Function): fonction a executer synchronisee avec la boucle du moteur
- Retourne un Timer
- Execute le code au meme timing que le cycle de mise a jour interne d'Anime.js

```js
utils.sync(function)
```

```js
import { animate, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $speed ] = utils.$('.speed');

const animation = animate('.circle', {
  x: '16rem',
  loop: true,
  alternate: true,
  playbackRate: 1,
});

const updateSpeed = () => {
  const { value } = $range;
  $speed.innerHTML = utils.roundPad(+value, 2);
  utils.sync(() => animation.speed = value);
}

$range.addEventListener('input', updateSpeed);
```

### utilities/createtimekeeper

`https://animejs.com/documentation/utilities/createtimekeeper`

> utils.keepTime() retourne une fonction qui recree un Timer/Animation/Timeline en preservant son temps de lecture courant, permettant de mettre a jour les parametres ou changer de cible sans repartir de zero.

`utils.keepTime(constructor)` retourne une fonction qui recree un Timer, une Animation ou une Timeline tout en preservant son temps de lecture courant. Cela permet des mises a jour de parametres sans interrompre l'etat de l'animation. Le parametre `constructor` (Function) est un callback qui retourne une instance Timer, Animation ou Timeline. Retourne une fonction qui invoque le constructor et retourne l'instance suivie. Cas d'usage cle : permet de changer de cible d'animation ou de mettre a jour des parametres tout en maintenant la continuite de lecture — appeler la fonction retournee recree l'animation a sa position courante plutot que de redemarrer depuis le debut.

**Faits clés**

- Signature: keepTime(constructor: Function): Function
- constructor (Function): callback retournant un Timer, Animation ou Timeline
- Retourne une fonction qui invoque le constructor et retourne l'instance suivie
- Recree l'instance a sa position de lecture courante (pas de redemarrage depuis le debut)
- Note: la page utilise le nom keepTime (URL createtimekeeper)

```js
keepTime(constructor: Function): Function
```

```js
import { animate, utils } from 'animejs';

const [ $button ] = utils.$('button');
const clocks = utils.$('.clock');
let targetIndex = 0;

const animateNextTarget = utils.keepTime(() => {
  if (targetIndex > clocks.length - 1) targetIndex = 0;
  return animate(clocks[targetIndex++], {
    color: ['#B7FF54', '#FF4B4B'],
    rotate: 360,
    ease: 'linear',
    duration: 8000,
    loop: true,
  })
});

animateNextTarget();

$button.addEventListener('click', animateNextTarget);
```

### utilities/random

`https://animejs.com/documentation/utilities/random`

> utils.random() genere un nombre aleatoire entre min et max, avec un controle optionnel de precision via le nombre de decimales.

`utils.random(min, max, decimalLength)` genere un nombre aleatoire entre deux bornes, avec un controle optionnel de precision via le nombre de decimales. `min` (Number) est la valeur minimale de l'intervalle ; `max` (Number) la valeur maximale ; `decimalLength` (Number, defaut 0) le nombre optionnel de decimales dans le resultat. Retourne un Number, une valeur aleatoire dans l'intervalle specifie. Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: const randomValue = utils.random(min, max, decimalLength);
- min (Number): valeur minimale de l'intervalle
- max (Number): valeur maximale de l'intervalle
- decimalLength (Number, defaut 0): nombre optionnel de decimales
- Retourne un Number
- Disponible depuis la version 2.0.0

```js
const randomValue = utils.random(min, max, decimalLength);
```

```js
import { utils } from 'animejs';

utils.set('.square', {
  x: () => utils.random(2, 18, 2) + 'rem',
  rotate: () => utils.random(0, 180),
  scale: () => utils.random(.25, 1.5, 3),
});
```

```js
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
<div class="small row">
  <div class="square"></div>
</div>
```

### utilities/createseededrandom

`https://animejs.com/documentation/utilities/createseededrandom`

> utils.createSeededRandom() retourne une fonction pseudo-aleatoire pre-amorcee (seeded) qui renvoie toujours la meme suite de nombres dans un intervalle donne, de maniere deterministe.

`utils.createSeededRandom(seed, seededMin, seededMax, seededDecimalLength)` retourne une fonction pseudo-aleatoire pre-amorcee qui renvoie toujours la meme suite de Number dans un intervalle specifie, avec un troisieme parametre optionnel determinant le nombre de decimales. La fonction retournee genere des valeurs aleatoires deterministes — des valeurs de seed identiques produisent des sequences identiques. Parametres : `seed` (Number, defaut 0) valeur de seed pour la generation pseudo-aleatoire ; `seededMin` (Number, defaut 0) valeur minimale de l'intervalle ; `seededMax` (Number, defaut 1) valeur maximale ; `seededDecimalLength` (Number, defaut 0) nombre de decimales en sortie. Retourne une fonction random() pre-amorcee acceptant les parametres (min, max, decimalLength). Disponible depuis la version 2.0.0.

**Faits clés**

- Signature: const seededRandom = utils.createSeededRandom(seed, seededMin, seededMax, seededDecimalLength);
- seed (Number, defaut 0): valeur de seed pour la generation pseudo-aleatoire
- seededMin (Number, defaut 0): valeur minimale de l'intervalle
- seededMax (Number, defaut 1): valeur maximale de l'intervalle
- seededDecimalLength (Number, defaut 0): nombre de decimales en sortie
- Retourne une fonction random() pre-amorcee acceptant (min, max, decimalLength)
- Deterministe: meme seed produit la meme sequence
- Disponible depuis la version 2.0.0

```js
const seededRandom = utils.createSeededRandom(seed, seededMin, seededMax, seededDecimalLength);
```

```js
import { utils } from 'animejs';

const seededRandom = utils.createSeededRandom(12345);

utils.set('.square', {
  x: () => seededRandom(2, 18, 2) + 'rem',
  rotate: () => seededRandom(0, 180),
  scale: () => seededRandom(.25, 1.5, 3),
});
```

### utilities/random-pick

`https://animejs.com/documentation/utilities/random-pick`

> utils.randomPick() selectionne et retourne un element aleatoire d'une collection (Array, NodeList ou String).

`utils.randomPick(collection)` selectionne et retourne un element arbitraire de n'importe quel type de collection. Le parametre `collection` accepte un Array, une NodeList ou une String. Retourne un element choisi aleatoirement dans la collection fournie. Utile pour introduire de l'aleatoire dans les animations et les assignations de proprietes. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: const randomElement = utils.randomPick(collection);
- collection: accepte Array | NodeList | String
- Retourne un element choisi aleatoirement dans la collection
- Disponible depuis la version 4.0.0

```js
const randomElement = utils.randomPick(collection);
```

```js
import { utils } from 'animejs';

utils.set('.letter', {
  x: () => utils.randomPick([5, 9, 13, 17]) + 'rem',
  scale: () => utils.randomPick([1, 1.25, 1.5, 1.75]),
  color: () => `var(--hex-${utils.randomPick(['red', 'orange', 'yellow'])}-1)`,
  innerHTML: () => utils.randomPick('ABCD'),
});
```

```js
<div class="small row">
  <div class="letter">A</div>
</div>
<div class="small row">
  <div class="letter">B</div>
</div>
<div class="small row">
  <div class="letter">C</div>
</div>
<div class="small row">
  <div class="letter">D</div>
</div>
```

### utilities/shuffle

`https://animejs.com/documentation/utilities/shuffle`

> utils.shuffle() mute un tableau en randomisant l'ordre de ses elements, avec un generateur aleatoire (seeded) optionnel.

`utils.shuffle(array, rnd)` mute un tableau en randomisant l'ordre de ses elements. Il modifie le tableau original plutot que d'en creer une copie, et accepte optionnellement un generateur de nombres aleatoires amorce (seeded) pour des melanges reproductibles. `array` (Array) est le tableau a melanger (mute en place) ; `rnd` (Function, optionnel) est un generateur de nombres aleatoires correspondant a la signature de `random()`, defaut `random`. Retourne le tableau Array mute avec l'ordre des elements randomise. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: const shuffledArray = utils.shuffle(array); ou utils.shuffle(array, rnd)
- array (Array): le tableau a melanger, mute en place
- rnd (Function, optionnel): generateur aleatoire correspondant a la signature de random(), defaut random
- Retourne le tableau Array mute (ordre randomise)
- Mutation in-place: modifie le tableau original, pas de copie
- Disponible depuis la version 4.0.0

```js
const shuffledArray = utils.shuffle(array);
```

```js
const seededShuffle = utils.shuffle(array, utils.createSeededRandom(42));
```

```js
import { utils, animate, stagger } from 'animejs';

const [ $shuffle ] = utils.$('button');
const squares = utils.$('.square');
const x = stagger('3.2rem');

// Initial squares x position
utils.set(squares, { x });

const shuffle = () => animate(utils.shuffle(squares), { x });

$shuffle.addEventListener('click', shuffle);
```

### utilities/round

`https://animejs.com/documentation/utilities/round`

> utils.round() arrondit une valeur a un nombre de decimales donne, ou retourne une fonction d'arrondi reutilisable et chainable si appele avec le seul decimalLength.

`utils.round(value, decimalLength)` arrondit une valeur numerique a un nombre specifie de decimales. Quand appele avec les deux parametres, il retourne le resultat arrondi immediatement (Number). Quand appele avec seulement `decimalLength`, il retourne une fonction reutilisable qui arrondit les valeurs a cette precision — permettant le chainage avec d'autres utilitaires. Parametres : `value` (Number, optionnel) ; `decimalLength` (Number, requis). Valeur de retour : avec parametre value, retourne un Number ; sans parametre value, retourne une Function utilitaire chainable pour arrondir avec une longueur de decimales pre-definie.

**Faits clés**

- Signatures: const roundedValue = utils.round(value, decimalLength); / const roundingFunction = utils.round(decimalLength);
- value (Number, optionnel)
- decimalLength (Number, requis)
- Avec value: retourne un Number ; sans value: retourne une Function chainable
- Chainable avec clamp(): utils.clamp(0, 100).round(2)
- Utilisable comme modifier d'animation: modifier: utils.round(1)

```js
const roundedValue = utils.round(value, decimalLength);
```

```js
const roundingFunction = utils.round(decimalLength);
```

```js
const roundedValue = utils.round(72.7523, 2); // 72.75
```

```js
const clampAndRound = utils.clamp(0, 100).round(2);
clampAndRound(72.7523);  // 72.75
clampAndRound(120.2514); // 100
```

```js
import { animate, utils } from 'animejs';

animate('.rounded', {
  rotate: '1turn',
  modifier: utils.round(1), // Used as a function
  duration: 3000,
  loop: true,
});
```

### utilities/clamp

`https://animejs.com/documentation/utilities/clamp`

> utils.clamp() restreint un Number entre min et max, ou cree une fonction de clamping reutilisable et chainable si appele avec seulement min et max.

`utils.clamp(value, min, max)` restreint un Number entre les valeurs min et max specifiees, ou cree une fonction de clamping avec des parametres min et max pre-definis. Quand appele avec trois arguments, il retourne immediatement un nombre borne. Quand appele avec deux arguments (min, max), il retourne une fonction reutilisable et chainable. Parametres : `value` (Number, optionnel) ; `min` (Number, requis) ; `max` (Number, requis).

**Faits clés**

- Signatures: const clampedValue = utils.clamp(value, min, max); / const clamperFunction = utils.clamp(min, max);
- value (Number, optionnel)
- min (Number, requis)
- max (Number, requis)
- Avec 3 arguments: retourne un Number borne ; avec 2 arguments: retourne une Function chainable
- Chainable: utils.clamp(0, 100).round(2)
- Utilisable comme modifier d'animation: modifier: utils.clamp(.25, .75)

```js
const clampedValue = utils.clamp(value, min, max);
```

```js
const clamperFunction = utils.clamp(min, max);
```

```js
const clampBetween0and100 = utils.clamp(0, 100);
clampBetween0and100(90);  // 90
clampBetween0and100(120); // 100
clampBetween0and100(-15); // 0
```

```js
const clampAndRound = utils.clamp(0, 100).round(2);
clampAndRound(72.7523); // 72.75
clampAndRound(120.2514); // 100
```

```js
animate('.clamped', {
  rotate: '1turn',
  modifier: utils.clamp(.25, .75),
  duration: 3000,
  loop: true,
  ease: 'inOut',
});
```

### utilities/snap

`https://animejs.com/documentation/utilities/snap`

> utils.snap() rounds a Number to the nearest specified increment, or to the closest value in an array, or creates a chainable snapping function.

utils.snap() arrondit un Number a l'increment specifie le plus proche, ou cree une fonction de snapping avec un parametre increment predefini. Si l'increment est un Array, l'utilitaire selectionne la valeur du tableau la plus proche de l'entree. Appele avec une valeur, il retourne un resultat numerique ; appele sans valeur (increment seul), il retourne une fonction utilitaire chainable. Fonctionne tres bien avec le parametre de tween 'modifier'.

**Faits clés**

- Signature: const snappedValue = utils.snap(value, increment); / const snapperFunction = utils.snap(increment);
- Parametre value (optional): Number
- Parametre increment: Number | Array<Number>
- Retour: Number si value fournie, sinon Function chainable
- Si increment est un Array, choisit la valeur du tableau la plus proche
- Chainable avec clamp() et autres utilitaires

```js
const snapTo10 = utils.snap(10);
snapTo10(94);  // 90
snapTo10(-17); // -20

const snapToArray = utils.snap([0, 50, 100]);
snapToArray(30);  // 50
snapToArray(75);  // 100
snapToArray(-10); // 0

const clampAndSnap = utils.clamp(0, 100).snap(30);
clampAndSnap(72.7523); // 60
clampAndSnap(120.2514); // 90
```

```js
import { animate, utils } from 'animejs';

animate('.normal', {
  rotate: '1turn',
  duration: 3000,
  loop: true,
  ease: 'inOut',
});

animate('.snapped', {
  rotate: '1turn',
  modifier: utils.snap(.25),
  duration: 3000,
  loop: true,
  ease: 'inOut',
});
```

### utilities/wrap

`https://animejs.com/documentation/utilities/wrap`

> utils.wrap() enveloppe (boucle) un Number entre une plage min/max, ou cree une fonction wrapping chainable avec min/max predefinis.

utils.wrap() enveloppe un Number entre une plage definie par les valeurs min et max, ou cree une fonction de wrapping avec des parametres min et max predefinis. Appele avec une valeur, retourne le nombre enveloppe ; appele uniquement avec min/max, produit une fonction chainable pour des operations de wrapping repetees. Les valeurs depassant max repartent depuis min (et inversement).

**Faits clés**

- Signature: const wrappedValue = utils.wrap(value, min, max); / const wrapperFunction = utils.wrap(min, max);
- Parametre value (optional): Number
- Parametre min: Number (requis)
- Parametre max: Number (requis)
- Retour: Number si value fournie, sinon Function chainable
- Chainable avec round() et autres utilitaires

```js
const wrapBetween0and100 = utils.wrap(0, 100);
wrapBetween0and100(105); // 5
wrapBetween0and100(220); // 20
wrapBetween0and100(-15); // 85
```

```js
const wrapAndRound = utils.wrap(0, 100).round(2);
wrapAndRound(105.7523); // 5.75
wrapAndRound(220.2514); // 20.25
```

```js
animate('.wrapped', {
  rotate: '1turn',
  modifier: utils.wrap(-.25, .25),
  duration: 3000,
  loop: true,
  ease: 'inOut',
});
```

### utilities/map-range

`https://animejs.com/documentation/utilities/map-range`

> utils.mapRange() remappe un Number d'une plage source vers une plage cible, ou cree une fonction de mapping chainable avec plages predefinies.

utils.mapRange() mappe un Number d'une plage a une autre, ou cree une fonction de mapping avec des parametres de plages predefinis. Appele avec une valeur, retourne le nombre mappe ; sans valeur, retourne une fonction reutilisable chainable. Les bornes source sont fromLow/fromHigh et les bornes cibles toLow/toHigh.

**Faits clés**

- Signature: const mappedValue = utils.mapRange(value, fromLow, fromHigh, toLow, toHigh); / const mapperFunction = utils.mapRange(fromLow, fromHigh, toLow, toHigh);
- Parametre value (optional): Number
- Parametres fromLow, fromHigh, toLow, toHigh: Number
- Retour: Number si value fournie, sinon Function chainable
- Disponible depuis la version 4.0.0
- Chainable avec clamp() et autres utilitaires

```js
const mappedValue = utils.mapRange(45, 0, 100, 0, 200); // 90
```

```js
const mapFrom0and100to0and200 = utils.mapRange(0, 100, 0, 200);
mapFrom0and100to0and200(45);  // 90
mapFrom0and100to0and200(120); // 240
mapFrom0and100to0and200(-15); // -30
```

```js
const normalizeAndClamp = utils.mapRange(-100, 100, 0, 1).clamp(0, 1);
normalizeAndClamp(50);  // 0.75
normalizeAndClamp(120); // 1
```

```js
animate('.mapped', {
  rotate: '12turn',
  modifier: utils.mapRange(0, 12, 0, 1),
  duration: 12000,
});
```

### utilities/lerp

`https://animejs.com/documentation/utilities/lerp`

> utils.lerp() effectue une interpolation lineaire entre start et end selon progress (0-1), ou cree une fonction d'interpolation chainable.

utils.lerp() effectue une interpolation lineaire. Avec les trois parametres (start, end, progress), retourne un seul nombre interpole. Sans progress, retourne une fonction chainable qui accepte ensuite des valeurs de progression. progress est une valeur entre 0 et 1.

**Faits clés**

- Signature: const interpolatedValue = utils.lerp(start, end, progress); / const interpolatorFunction = utils.lerp(start, end);
- Parametre start: Number
- Parametre end: Number
- Parametre progress (optional): Number entre 0 et 1
- Retour: Number si progress fourni, sinon Function chainable
- Alias: interpolate()
- Chainable avec round() et autres utilitaires

```js
utils.lerp(0, 100, 0.5);  // Returns 50
```

```js
const interpolateBetween0and100 = utils.lerp(0, 100);
interpolateBetween0and100(0.5);  // 50
interpolateBetween0and100(0.75); // 75
```

```js
const interpolateAndRound = utils.lerp(0, 100).round(2);
interpolateAndRound(0.677523); // 67.75
```

```js
animate('.interpolated', {
  rotate: '1turn',
  modifier: utils.lerp(0, 12),
  duration: 3000,
});
```

### utilities/damp

`https://animejs.com/documentation/utilities/damp`

> utils.damp() est une version d'interpolation lineaire independante du frame rate, utilisant deltaTime et un facteur amount (0-1).

utils.damp() est une version de utils.lerp() independante du frame rate. Plus amount approche de 1, plus le resultat approche de la valeur end. deltaTime est exprime en millisecondes. Utile dans des boucles (createTimer) pour des animations stables quel que soit le frame rate.

**Faits clés**

- Signature: utils.damp(start, end, deltaTime, amount)
- Parametre start: Number
- Parametre end: Number
- Parametre deltaTime: Number (millisecondes)
- Parametre amount: Number [0-1] (facteur d'interpolation)
- Retour: Number
- Version frame-rate-independante de utils.lerp()
- Disponible depuis la version 4.0.0

```js
utils.damp(0, 100, 8, 0);    // 0
utils.damp(0, 100, 8, 0.5);  // 50
utils.damp(0, 100, 8, 1);    // 100
```

```js
import { animate, createTimer, utils } from 'animejs';

const [ $input ] = utils.$('.input');
const [ $lerped15fps ] = utils.$('.lerped-15');

const dampedLoop = createTimer({
  frameRate: 15,
  onUpdate: clock => {
    const sourceRotate = utils.get($input, 'rotate', false);
    const lerpedRotate = utils.get($lerped15fps, 'rotate', false);
    utils.set($lerped15fps, {
      rotate: utils.damp(lerpedRotate, sourceRotate, clock.deltaTime, .075) + 'turn'
    });
  }
});
```

### utilities/round-pad

`https://animejs.com/documentation/utilities/round-pad`

> utils.roundPad() arrondit un nombre a un nombre de decimales donne en completant avec des zeros, retournant une String, ou cree une fonction chainable.

utils.roundPad() effectue un arrondi numerique a une decimale specifiee tout en completant avec des zeros si necessaire. Appele avec les deux arguments, retourne une String. Appele avec seulement decimalLength, produit une fonction reutilisable chainable. value peut etre un Number ou une String.

**Faits clés**

- Signature: const roundedPaddedValue = utils.roundPad(value, decimalLength); / const roundPadderFunction = utils.roundPad(decimalLength);
- Parametre value (optional): Number | String
- Parametre decimalLength: Number
- Retour: String si value fournie, sinon Function chainable
- Complete avec des zeros pour atteindre la longueur decimale
- Chainable avec snap() et autres utilitaires

```js
const roundPadTo2Decimals = utils.roundPad(2);
roundPadTo2Decimals(90.12345);  // '90.12'
roundPadTo2Decimals(120);       // '120.00'
roundPadTo2Decimals(15.9);      // '15.90'
```

```js
const snapAndRoundPad = utils.snap(50).roundPad(2);
snapAndRoundPad(123.456); // '100.00'
snapAndRoundPad(175.789); // '200.00'
```

```js
import { animate, utils } from 'animejs';

animate('.value', {
  innerHTML: '8.1',
  modifier: utils.roundPad(3),
  duration: 10000,
  ease: 'linear',
});
```

### utilities/pad-start

`https://animejs.com/documentation/utilities/pad-start`

> utils.padStart() complete un nombre depuis le debut avec une chaine jusqu'a une longueur donnee, retournant une String, ou cree une fonction de padding chainable.

utils.padStart() complete un Number depuis le debut avec une chaine jusqu'a ce que le resultat atteigne une longueur donnee, ou cree une fonction de padding avec des parametres totalLength et padString predefinis. Retourne une String si une valeur est fournie, sinon une fonction utilitaire chainable.

**Faits clés**

- Signature: const paddedValue = utils.padStart(value, totalLength, padString); / const padderFunction = utils.padStart(totalLength, padString);
- Parametre value (optional): String | Number
- Parametre totalLength: Number
- Parametre padString: String
- Retour: String si value fournie, sinon Function chainable
- Padding applique depuis le debut (start)
- Disponible depuis la version 4.0.0

```js
const padTo5WithZeros = utils.padStart(5, '0');
padTo5WithZeros('123');  // '00123'
padTo5WithZeros(78);     // '00078'
padTo5WithZeros('1234'); // '01234'

const roundAndPad = utils.round(2).padStart(5, '0');
roundAndPad(12.345);  // '12.35'
roundAndPad(7.8);     // '07.80'
```

```js
import { animate, utils } from 'animejs';

animate('.value', {
  innerHTML: 10000,
  modifier: utils.round(0).padStart(6, '-'),
  duration: 100000,
  ease: 'linear',
});
```

### utilities/pad-end

`https://animejs.com/documentation/utilities/pad-end`

> utils.padEnd() complete un nombre depuis la fin avec une chaine jusqu'a une longueur donnee, retournant une String, ou cree une fonction de padding chainable.

utils.padEnd() complete un Number depuis la fin avec une chaine jusqu'a ce que le resultat atteigne une longueur donnee, ou cree une fonction de padding reutilisable quand appele sans valeur. Avec une valeur fournie : retourne une String paddee. Sans valeur : retourne une fonction utilitaire chainable acceptant un nombre/chaine a padder.

**Faits clés**

- Signature: const paddedValue = utils.padEnd(value, totalLength, padString); / const padderFunction = utils.padEnd(totalLength, padString);
- Parametre value (optional): String | Number
- Parametre totalLength: Number
- Parametre padString: String
- Retour: String si value fournie, sinon Function chainable
- Padding applique depuis la fin (end)
- Disponible depuis la version 4.0.0

```js
const padTo5WithZeros = utils.padEnd(5, '0');
padTo5WithZeros('123');  // '12300'
padTo5WithZeros(78);     // '78000'
padTo5WithZeros('1234'); // '12340'
```

```js
const roundAndPadEnd = utils.round(0).padEnd(5, '0');
roundAndPadEnd(123.456); // '12300'
roundAndPadEnd(7.8);     // '80000'
```

```js
animate('.value', {
  innerHTML: 1,
  modifier: utils.round(3).padEnd(6, '-'),
  duration: 100000,
  ease: 'linear',
});
```

### utilities/deg-to-rad

`https://animejs.com/documentation/utilities/deg-to-rad`

> utils.degToRad() convertit des degres en radians, ou retourne une fonction chainable de conversion.

utils.degToRad() convertit des mesures en degres en leurs equivalents en radians. Appele sans arguments, retourne une fonction utilitaire chainable pour convertir des degres en radians. Retourne un Number si degrees est fourni.

**Faits clés**

- Signature: const radians = utils.degToRad(degrees);
- Parametre degrees (optional): Number
- Retour: Number si degrees fourni, sinon Function chainable
- utils.degToRad(360) === 6.283185307179586
- Chainable avec round() et autres utilitaires

```js
const radians = utils.degToRad(360); // 6.283185307179586
```

```js
const degToRad = utils.degToRad();
degToRad(360); // 6.283185307179586

const roundDegToRad = utils.degToRad().round(2);
roundDegToRad(180); // 3.14
roundDegToRad(90);  // 1.57
```

```js
import { animate, createAnimatable, utils } from 'animejs';

const radAnimatable = createAnimatable('.rad', {
  rotate: { unit: 'rad', duration: 0 },
});

const [ $deg ] = utils.$('.deg');

const degAnimation = animate($deg, {
  rotate: '360deg',
  ease: 'linear',
  loop: true,
  onUpdate: () => {
    const degrees = utils.get($deg, 'rotate', false);
    radAnimatable.rotate(utils.degToRad(degrees));
  }
});
```

### utilities/rad-to-deg

`https://animejs.com/documentation/utilities/rad-to-deg`

> utils.radToDeg() convertit des radians en degres, ou retourne une fonction chainable de conversion.

utils.radToDeg() convertit des mesures d'angle en radians en leurs equivalents en degres. Appele sans arguments, produit une fonction reutilisable chainable. Retourne un Number quand radians est fourni.

**Faits clés**

- Signature: const degrees = utils.radToDeg(radians);
- Parametre radians (optional): Number
- Retour: Number si radians fourni, sinon Function chainable
- utils.radToDeg(Math.PI) === 180
- Chainable avec round() et autres utilitaires

```js
utils.radToDeg(1.7453292519943295); // 100
utils.radToDeg(Math.PI);            // 180
```

```js
const roundRadToDeg = utils.radToDeg().round(2);
roundRadToDeg(Math.PI / 7);  // 25.71
```

```js
import { animate, createAnimatable, utils } from 'animejs';

const degAnimatable = createAnimatable('.deg', {
  rotate: { unit: 'deg', duration: 0 }
});

const [ $rad ] = utils.$('.rad');

const degAnimation = animate($rad, {
  rotate: (Math.PI * 2) + 'rad',
  ease: 'linear',
  loop: true,
  onUpdate: () => {
    const radians = utils.get($rad, 'rotate', false);
    degAnimatable.rotate(utils.radToDeg(radians));
  }
});
```

### utilities/chain-able-utility-functions

`https://animejs.com/documentation/utilities/chain-able-utility-functions`

> Les fonctions utilitaires chainables permettent de combiner plusieurs operations en une seule expression en appelant l'utilitaire sans son parametre value optionnel.

Les fonctions utilitaires chainables (depuis v4.0.0) permettent de combiner plusieurs fonctions dans une seule expression pour creer des operations complexes par enchainement. Elles s'activent en appelant une fonction utilitaire SANS son parametre de valeur optionnel : elles retournent alors une fonction reutilisable qui peut etre enchainee avec d'autres fonctions chainables. Fonctions supportees : round(), clamp(), snap(), wrap(), mapRange(), interpolate() (alias lerp()), roundPad(), padStart(), padEnd(), degToRad(), radToDeg(). Fonctionnent tres bien en combinaison avec le parametre de tween 'modifier' des animations.

**Faits clés**

- Disponible depuis v4.0.0
- Fonctions chainables: round(), clamp(), snap(), wrap(), mapRange(), interpolate() (alias lerp()), roundPad(), padStart(), padEnd(), degToRad(), radToDeg()
- Activation: appeler l'utilitaire SANS son parametre value optionnel -> retourne une fonction reutilisable
- Combinables avec le tween parameter 'modifier'

```js
const chainableClamp = utils.clamp(0, 100);
const result = chainableClamp(150); // 100
```

```js
const normalizeAndRound = utils.mapRange(0, 255, 0, 1).round(1);
normalizeAndRound(128); // '0.5'
normalizeAndRound(64);  // '0.3'
```

```js
const clampRoundPad = utils.clamp(0, 100).round(2).padStart(6, '0');
clampRoundPad(125)   // '000100'
clampRoundPad(75.25) // '075.25'
```

```js
import { animate, utils } from 'animejs';

animate('.value', {
  innerHTML: 1000,
  modifier: utils.wrap(0, 10).roundPad(3).padStart(6, '0'),
  duration: 100000,
  alternate: true,
  loop: true,
  ease: 'linear',
});
```


## easings

### easings

`https://animejs.com/documentation/easings`

> Vue d'ensemble des easings d'anime.js : categories (built-in, cubic Bezier, linear, steps, irregular, spring), accessibles via l'objet easings ou importes depuis 'animejs'/'animejs/easings'.

Les easing functions controlent la progression d'une animation. Elles sont accessibles via l'objet easings ou importees directement depuis le module 'animejs' ou le sous-chemin 'animejs/easings'. Elles acceptent des parametres numeriques et peuvent etre passees au parametre 'ease' des configurations d'animation. Categories presentees : Built-in eases (fonctions predefinies), Cubic Bezier (courbe personnalisee), Linear (progression uniforme), Steps (transitions discretes par paliers), Irregular (easing personnalise non-standard), Spring (animations ressort basees sur la physique). Les eases peuvent etre referencees par String (ex. 'out(3)', 'inOut(3)') ou par fonction programmatique. Un editeur interactif de fonctions d'easing est disponible pour visualisation et personnalisation.

**Faits clés**

- Categories: Built-in eases, Cubic Bezier, Linear, Steps, Irregular, Spring
- Accessibles via l'objet easings ou imports depuis 'animejs' / 'animejs/easings'
- Passees au parametre 'ease' des animations
- Eases referencables par String (ex. 'out(3)', 'inOut(3)') ou par fonction
- cubicBezier prend 4 parametres numeriques
- spring accepte un objet de config (ex. { bounce: .35 })
- Editeur interactif de fonctions d'easing disponible

```js
import { easings } from 'animejs';
easings.eases.inOut(3);
easings.cubicBezier(.7, .1, .5, .9);
easings.spring({ bounce: .35 });
```

```js
import { eases, cubicBezier, spring } from 'animejs';
eases.inOut(3);
cubicBezier(.7, .1, .5, .9);
spring({ bounce: .35 });
```

```js
import { animate, waapi, cubicBezier, spring } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'out(3)',
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  ease: cubicBezier(.7, .1, .5, .9),
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  ease: spring({ bounce: .35 }),
});
```

### easings/built-in-eases

`https://animejs.com/documentation/easings/built-in-eases`

> Liste des easings predefinis d'anime.js (Linear, Power, Quad, Cubic, Quart, Quint, Sine, Expo, Circ, Bounce, Back, Elastic) referencables par nom dans le parametre ease ou via l'objet eases.

Anime.js fournit une collection de fonctions d'easing predefinies, specifiables par nom dans le parametre 'ease' des animations et accessibles via l'objet importe 'eases'. Chaque famille (sauf linear) propose les variantes in / out / inOut / outIn. Certaines familles acceptent des parametres personnalises : Power (power = 1.675), Back (overshoot = 1.70158), Elastic (amplitude = 1, period = .3). Les fonctions parametriques supportent des arguments personnalises (ex. 'outElastic(.8, 1.2)') pour affiner le comportement. La famille Power utilise les noms generiques 'in', 'out', 'inOut', 'outIn'.

**Faits clés**

- Linear: 'linear'
- Power (power = 1.675): 'in', 'out', 'inOut', 'outIn'
- Quad: 'inQuad', 'outQuad', 'inOutQuad', 'outInQuad'
- Cubic: 'inCubic', 'outCubic', 'inOutCubic', 'outInCubic'
- Quart: 'inQuart', 'outQuart', 'inOutQuart', 'outInQuart'
- Quint: 'inQuint', 'outQuint', 'inOutQuint', 'outInQuint'
- Sine: 'inSine', 'outSine', 'inOutSine', 'outInSine'
- Exponential: 'inExpo', 'outExpo', 'inOutExpo', 'outInExpo'
- Circular: 'inCirc', 'outCirc', 'inOutCirc', 'outInCirc'
- Bounce: 'inBounce', 'outBounce', 'inOutBounce', 'outInBounce'
- Back (overshoot = 1.70158): 'inBack', 'outBack', 'inOutBack', 'outInBack'
- Elastic (amplitude = 1, period = .3): 'inElastic', 'outElastic', 'inOutElastic', 'outInElastic'
- Eases parametriques: ex. 'inOut(3)', 'outElastic(.8, 1.2)'
- Accessibles aussi via l'objet importe 'eases'

```js
import { animate, waapi } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'inOut',
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'inOut(3)',
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'inOutExpo',
});
```

### easings/cubic-bezier-easing

`https://animejs.com/documentation/easings/cubic-bezier-easing`

> Définit le rythme d'une animation via une courbe de Bézier cubique à quatre points de contrôle.

Un easing cubic bezier définit le rythme d'une animation à l'aide d'une courbe de Bézier. En JavaScript on utilise la fonction cubicBezier(x1, y1, x2, y2) ; en WAAPI on passe une chaîne 'cubic-bezier(x1, y1, x2, y2)' ou 'cubicBezier(x1, y1, x2, y2)'. x1 et x2 sont les coordonnées X des deux points de contrôle et doivent être dans l'intervalle 0–1. y1 et y2 sont les coordonnées Y et acceptent n'importe quelle valeur : une valeur négative crée de l'anticipation, une valeur >1 crée un dépassement (overshoot).

**Faits clés**

- Signature JS : cubicBezier(x1, y1, x2, y2)
- Signature WAAPI : 'cubic-bezier(x1, y1, x2, y2)' ou 'cubicBezier(x1, y1, x2, y2)'
- x1, x2 : Number, doivent être 0–1 (coordonnées X des points de contrôle)
- y1, y2 : Number, n'importe quelle valeur ; négatif = anticipation, >1 = overshoot

```js
import { animate, cubicBezier } from 'animejs';

animate(target, { x: 100, ease: cubicBezier(0, 0, 0.58, 1) });
```

```js
import { waapi } from 'animejs';

waapi.animate(target, { x: 100, ease: 'cubic-bezier(0, 0, 0.58, 1)' });
waapi.animate(target, { x: 100, ease: 'cubicBezier(0, 0, 0.58, 1)' });
```

```js
import { animate, waapi, cubicBezier } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  ease: cubicBezier(0.5, 0, 0.9, 0.3)
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  ease: cubicBezier(0.1, 0.7, 0.5, 1)
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'cubicBezier(0.7, 0.1, 0.5, 0.9)'
});
```

### easings/linear-easing

`https://animejs.com/documentation/easings/linear-easing`

> Définit le rythme via interpolation linéaire entre des points (stops) spécifiés, avec positions temporelles optionnelles.

Le linear easing définit le rythme d'animation par interpolation linéaire entre des points (stops) spécifiés. Disponible pour JavaScript (linear(stop1, stop2, ...stopN)) et pour WAAPI (chaîne 'linear(stop1, stop2, ...stopN)'). Chaque stop est un Number : valeur de sortie où 0 = début et 1 = fin ; au minimum deux stops sont requis ; des valeurs hors de 0-1 créent un dépassement (overshoot). On peut optionnellement fournir une position temporelle sous forme de chaîne 'valeur pourcentage' (ex. '0.5 50%') ; si on l'omet, les stops sont répartis uniformément. Le pourcentage ne peut pas s'appliquer au premier ni au dernier stop.

**Faits clés**

- Signature JS : linear(stop1, stop2, ...stopN)
- Signature WAAPI : 'linear(stop1, stop2, ...stopN)'
- stop : Number, 0 = début / 1 = fin, minimum deux requis, hors 0-1 = overshoot
- percentage : String optionnel au format 'valeur pourcentage' ; omis = répartition uniforme ; ne peut pas s'appliquer aux premier/dernier stops

```js
import { animate, linear } from 'animejs';

animate(target, { x: 100, ease: linear(0, '0.5 50%', '0.3 75%', 1) });
```

```js
import { waapi } from 'animejs';

waapi.animate(target, { x: 100, ease: 'linear(0, 0.5 50%, 0.3 75%, 1)' });
```

```js
animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: linear(0, 0, 0.5, 0.5, 1, 1)
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: linear(0, '1 25%', 0, 1)
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: 'linear(1, 0 25%, 1, 0)'
});
```

### easings/steps-easing

`https://animejs.com/documentation/easings/steps-easing`

> Crée une animation par paliers qui saute entre les valeurs à intervalles discrets au lieu d'une transition fluide.

Le steps easing crée une animation par paliers qui saute entre les valeurs à intervalles discrets plutôt qu'avec une transition fluide. Signature JS : steps(n, fromStart). Signature WAAPI : chaîne 'steps(n, position)'. Le paramètre n (Number) représente le nombre de paliers égaux ; il doit être un entier positif. Le paramètre optionnel fromStart (Boolean) : si true, le changement se produit au début du palier ; si false, à la fin du palier (défaut : false). Pour WAAPI, on utilise la chaîne 'start' ou 'end' à la place du booléen.

**Faits clés**

- Signature JS : steps(n, fromStart)
- Signature WAAPI : 'steps(n, position)'
- n : Number, entier positif, nombre de paliers égaux
- fromStart : Boolean optionnel, défaut false ; true = changement au début du palier, false = à la fin
- WAAPI utilise 'start' / 'end' au lieu du booléen fromStart

```js
import { animate, steps } from 'animejs';

animate(target, { x: 100, ease: steps(5) });
animate(target, { x: 100, ease: steps(5, true) });
```

```js
import { waapi } from 'animejs';

waapi.animate(target, { x: 100, ease: 'steps(5)' });
waapi.animate(target, { x: 100, ease: 'steps(5, start)' });
```

```js
import { animate, waapi } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  ease: steps(4)
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  ease: steps(4, true)
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  ease: 'steps(8, end)'
});
```

### easings/irregular-easing

`https://animejs.com/documentation/easings/irregular-easing`

> Établit un rythme d'animation via interpolation linéaire entre des points aléatoires, créant une progression non uniforme.

Le irregular easing établit le rythme d'animation par interpolation linéaire à travers des waypoints randomisés, créant une progression imprévisible et non uniforme plutôt que des courbes lisses. Signature : irregular(steps, randomness). Le paramètre steps (Number) représente le nombre de paliers aléatoires à générer et doit être un entier positif. Le paramètre optionnel randomness (Number, défaut 1) contrôle l'amplitude des variations aléatoires : des valeurs plus élevées créent des sauts plus prononcés entre les paliers. Fonctionnalité disponible en version 4.0.0 ou ultérieure.

**Faits clés**

- Signature : irregular(steps, randomness)
- steps : Number, entier positif, nombre de paliers aléatoires
- randomness : Number optionnel, défaut 1 ; amplitude des variations (plus élevé = sauts plus dramatiques)
- Disponible à partir de la version 4.0.0

```js
import { animate, waapi, irregular } from 'animejs';

animate('.row:nth-child(1) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: irregular(10, .5)
});

animate('.row:nth-child(2) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: irregular(10, 1)
});

waapi.animate('.row:nth-child(3) .square', {
  x: '17rem',
  rotate: 360,
  duration: 2000,
  ease: irregular(10, 2)
});
```

### easings/spring

`https://animejs.com/documentation/easings/spring`

> Génère une fonction d'easing basée sur la physique produisant des courbes avec comportement de stabilisation ; retourne aussi une durée calculée.

La méthode spring() génère une fonction d'easing basée sur la physique qui produit des courbes avec comportement de stabilisation. Elle retourne à la fois une fonction d'easing et une durée calculée. Deux jeux de paramètres existent. Les paramètres perçus (contrôle intuitif, axés sur le ressenti visuel) : bounce (Number, plage -1 à 1, défaut 0.5) contrôle le rebond — valeurs 0-1 créent des courbes rebondissantes, valeurs négatives créent des courbes suramorties ; plage -.5 à .5 recommandée ; duration (Number, plage 10 à 10000, défaut 628) durée perçue en ms quand l'animation semble terminée. Les paramètres physiques (contrôle direct) : mass (Number, 1 à 10000, défaut 1) masse de l'objet, valeurs élevées = mouvement plus lent et lourd ; stiffness (Number, 0 à 10000, défaut 100) coefficient de ressort, valeurs élevées = réponse plus tendue ; damping (Number, 0 à 10000, défaut 10) opposition au mouvement, valeurs élevées = moins de rebond ; velocity (Number, -10000 à 10000, défaut 0) vélocité initiale donnant un élan de départ. Un callback onComplete perçu (version JavaScript uniquement) se déclenche lorsque la durée perçue est atteinte (et non la durée de stabilisation). Note clé : le paramètre duration de l'animation est remplacé par la durée de stabilisation calculée du spring.

**Faits clés**

- spring() retourne une fonction d'easing ET une durée calculée
- Paramètres perçus : bounce (Number, -1 à 1, défaut 0.5 ; recommandé -.5 à .5), duration (Number, 10 à 10000, défaut 628 ms)
- Paramètres physiques : mass (1 à 10000, défaut 1), stiffness (0 à 10000, défaut 100), damping (0 à 10000, défaut 10), velocity (-10000 à 10000, défaut 0)
- onComplete perçu : Function, JavaScript uniquement, déclenché à la durée perçue (pas de stabilisation)
- Gotcha : le duration de l'animation est remplacé par la durée de stabilisation calculée du spring

```js
animate(target, { x: 100, ease: spring({ bounce: .5, duration: 350 }) });
```

```js
animate(target, { x: 100, ease: spring({ stiffness: 95, damping: 13 }) });
```

```js
animate(target, {
  x: 100,
  onComplete: () => console.log('settling duration complete'),
  ease: spring({ 
    bounce: .25,
    duration: 350,
    onComplete: () => console.log('perceived duration complete'),
  })
});
```

```js
import { animate, spring, utils } from 'animejs';

const [ $square1, $square2, $square3 ] = utils.$('.square');

utils.set('.square', { color: 'var(--hex-red-1)' })

animate($square1, {
  x: '17rem',
  rotate: 360,
  onComplete: () => utils.set($square1, { color: 'var(--hex-green-1)' }),
  ease: spring({
    bounce: .15,
    duration: 500,
    onComplete: () => utils.set($square1, { color: 'var(--hex-yellow-1)' }),
  })
});

animate($square2, {
  x: '17rem',
  rotate: 360,
  onComplete: () => utils.set($square2, { color: 'var(--hex-green-1)' }),
  ease: spring({
    bounce: .3,
    duration: 500,
    onComplete: () => utils.set($square2, { color: 'var(--hex-yellow-1)' }),
  })
});

animate($square3, {
  x: '17rem',
  rotate: 360,
  onComplete: () => utils.set($square3, { color: 'var(--hex-green-1)' }),
  ease: spring({
    stiffness: 90,
    damping: 14,
    onComplete: () => utils.set($square3, { color: 'var(--hex-yellow-1)' }),
  })
});
```


## web-animation-api

### web-animation-api

`https://animejs.com/documentation/web-animation-api`

> waapi.animate() : alternative légère (3KB vs 10KB) basée sur l'API native Element.animate() pour des animations accélérées matériellement.

Anime.js fournit une alternative WAAPI légère (3KB contre 10KB pour la version JavaScript) utilisant l'API native Element.animate(). Import : import { waapi } from 'animejs'; ou en module autonome import { waapi } from 'animejs/waapi';. Signature : waapi.animate(targets, parameters). targets (type Targets) : éléments DOM, sélecteurs CSS ou objets à animer. parameters (Object) : objet de configuration supportant les propriétés animables, paramètres de tween, réglages de lecture et callbacks. La valeur de retour est un objet WAAPIAnimation permettant de contrôler l'animation. Caractéristiques : utilise l'API Web Animation native sous le capot, supporte les mêmes propriétés d'animation que la méthode standard animate(), adapté aux animations accélérées matériellement, plus performant pour certains cas d'usage que la version complète d'Anime.js.

**Faits clés**

- Signature : waapi.animate(targets, parameters)
- Retourne un objet WAAPIAnimation
- Taille : 3KB (WAAPI) vs 10KB (version JS)
- Import : 'animejs' ou module autonome 'animejs/waapi'
- targets : DOM elements, sélecteurs CSS, ou objets ; parameters : Object de configuration

```js
import { waapi } from 'animejs';
```

```js
import { waapi } from 'animejs/waapi';
```

```js
import { waapi, stagger, splitText } from 'animejs';

const { chars } = splitText('h2', { words: false, chars: true });

waapi.animate(chars, {
  translate: `0 -2rem`,
  delay: stagger(100),
  duration: 600,
  loop: true,
  alternate: true,
  ease: 'inOut(2)',
});
```

### web-animation-api/when-to-use-waapi

`https://animejs.com/documentation/web-animation-api/when-to-use-waapi`

> Compare waapi.animate() et animate() (RAF) ; aucun n'est universellement supérieur, le choix dépend du type et du contexte d'animation.

La Web Animations API (WAAPI) et le requestAnimationFrame JavaScript (RAF) ont chacun des avantages distincts ; aucun n'est universellement supérieur, le choix dépend du type et du contexte d'animation. Privilégier waapi.animate() quand : (1) charge CPU/réseau — animation sous forte charge CPU ou réseau (bénéficie de l'accélération matérielle) ; (2) la taille du bundle compte — temps de chargement initial critique (3KB gzip vs 10KB pour la version JavaScript) ; (3) valeurs CSS complexes — animation de matrices de transformation CSS ou de fonctions de couleur CSS non gérées correctement par la version JavaScript. Utiliser animate() quand : (1) gros volume de cibles — plus de 500 cibles ; (2) animation non-DOM — objets JavaScript, canvas, WebGL ou WebGPU ; (3) types de cibles étendus — SVG, attributs DOM, ou propriétés CSS non supportées par WAAPI ; (4) séquences complexes — timelines et keyframes sophistiquées ; (5) contrôle avancé — méthodes de contrôle et callbacks étendus au-delà des capacités de WAAPI.

**Faits clés**

- Privilégier waapi.animate() : charge CPU/réseau (accélération matérielle), taille bundle critique (3KB vs 10KB), valeurs CSS complexes (matrices/couleurs)
- Utiliser animate() : >500 cibles, animation non-DOM (objets JS/canvas/WebGL/WebGPU), SVG/attributs DOM/props CSS non supportées par WAAPI, timelines/keyframes complexes, contrôle/callbacks avancés
- Aucun n'est universellement supérieur — dépend du type et contexte

```js
import { animate, waapi, utils } from 'animejs';

// WAAPI Animation
waapi.animate('.waapi.square', {
  x: '17rem',
  rotate: 180,
  loop: 3,
  alternate: true,
});

// JS Animation
const data = { x: '0rem', rotate: '0deg' }
const [ $log ] = utils.$('code');

animate(data, {
  x: 17,
  rotate: 180,
  modifier: utils.round(0),
  loop: 3,
  alternate: true,
  onRender: () => $log.innerHTML = JSON.stringify(data)
});
```

### web-animation-api/hardware-accelerated-animations

`https://animejs.com/documentation/web-animation-api/hardware-accelerated-animations`

> Les animations WAAPI s'exécutent hors du thread principal pour de meilleures performances ; limitation Safari sur les easings 'linear()'.

Les animations accélérées matériellement exploitent la Web Animation API (WAAPI) pour s'exécuter hors du thread principal, offrant des performances plus fluides quand les ressources CPU sont limitées et réduisant la consommation d'énergie. Propriétés accélérées matériellement universellement supportées dans les principaux navigateurs : opacity, transform, translate, scale, rotate. Support spécifique selon le navigateur : clip-path, filter. Limitation critique Safari : Safari (desktop et mobile) désactive l'accélération matérielle lorsque les animations utilisent des fonctions d'easing custom 'linear()'. Cela inclut les power eases comme 'out(3)', 'in(3)', 'inOut(3)', et tout easing JavaScript passé à waapi.animate(), même si la propriété elle-même supporte l'accélération.

**Faits clés**

- Propriétés accélérées universellement : opacity, transform, translate, scale, rotate
- Support spécifique navigateur : clip-path, filter
- Limitation Safari (desktop + mobile) : désactive l'accélération matérielle avec les easings custom 'linear()'
- Concerne 'out(3)', 'in(3)', 'inOut(3)' et tout easing JS passé à waapi.animate()
- Exécution hors thread principal = performances plus fluides + consommation réduite

```js
import { animate, waapi, createTimer, utils, cubicBezier } from 'animejs';

const [ $block ] = utils.$('.button');

const waapiAnim = waapi.animate('.waapi.square', {
  translate: 270,
  rotate: 180,
  alternate: true,
  loop: true,
  ease: 'cubic-bezier(0, 0, .58, 1)',
});

const jsAnim = animate('.js.square', {
  x: 270,
  rotate: 180,
  ease: cubicBezier(0, 0, .58, 1),
  alternate: true,
  loop: true,
});
```

### web-animation-api/improvements-to-the-web-animation-api

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api`

> Liste les améliorations apportées par waapi.animate() par rapport à la WAAPI native (intégration scroll, scope/media queries, et 7 catégories d'améliorations).

La page décrit les améliorations apportées par la méthode waapi.animate() d'Anime.js pour améliorer l'expérience développeur avec la Web Animation API. Améliorations clés : intégration ScrollObserver — les animations WAAPI peuvent être liées aux événements de scroll via autoplay: onScroll() ; Scope avec media queries — gestion facile des animations responsives et du nettoyage de composant via createScope({ mediaQueries: ... }). La documentation décrit sept catégories d'améliorations spécifiques, chacune ayant sa page dédiée : sensible defaults, multi-targets animation, default units, function-based values, individual CSS transforms, individual property parameters, et spring and custom easings. La page référence aussi les différences d'API avec la WAAPI native et les utilitaires de conversion comme waapi.convertEase().

**Faits clés**

- Intégration ScrollObserver via autoplay: onScroll()
- Scope avec media queries via createScope({ mediaQueries: {...} })
- Sept catégories d'améliorations : sensible defaults, multi-targets animation, default units, function-based values, individual CSS transforms, individual property parameters, spring and custom easings
- Utilitaire de conversion référencé : waapi.convertEase()

```js
waapi.animate('.square', {
  translate: '100px',
  autoplay: onScroll()
});
```

```js
createScope({
  mediaQueries: { reduceMotion: '(prefers-reduced-motion)' }
})
.add(({ matches }) => {
  const { reduceMotion } = matches;
  waapi.animate('.square', {
    transform: reduceMotion ? ['100px', '100px'] : '100px',
    opacity: [0, 1],
  });
});
```

### web-animation-api/improvements-to-the-web-animation-api/sensible-defaults

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/sensible-defaults`

> Anime.js applique des valeurs par défaut (durée/délai/easing) et persiste automatiquement la valeur finale, contrairement à la WAAPI native.

Anime.js améliore la Web Animation API native en fournissant des valeurs par défaut sensées qui manquent à la WAAPI native. La bibliothèque préserve automatiquement l'état de l'animation après son achèvement et applique des valeurs standard de durée/délai/easing sans intervention manuelle. Limitation de la WAAPI native : elle n'a aucun easing appliqué et, plus gênant, ne persiste pas sa valeur finale, laissant l'utilisateur définir manuellement les styles finaux après la fin de l'animation. Solution Anime.js : persiste automatiquement les valeurs finales d'animation et applique des easings par défaut cohérents, en accord avec la méthode standard JS animate(), éliminant le code boilerplate de persistance d'état et de configuration d'easing.

**Faits clés**

- Anime.js persiste automatiquement les valeurs finales et applique des easings par défaut
- WAAPI native : aucun easing par défaut + ne persiste pas la valeur finale (styles finaux à définir manuellement)
- Les defaults correspondent à la méthode standard JS animate()
- Élimine le boilerplate de persistance d'état et de config d'easing

```js
import { waapi } from 'animejs';

waapi.animate('.circle', { translate: '16rem' });
```

```js
const $el = document.querySelector('.circle');

$el.animate({ translate: '100px' }, {
  duration: 1000,
  easing: 'ease-out',
}).finished.then(() => {
  $el.style.translate = '100px';
});
```

### web-animation-api/improvements-to-the-web-animation-api/multi-targets-animation

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/multi-targets-animation`

> waapi.animate() accepte des sélecteurs CSS ciblant plusieurs éléments, avec support du stagger en un seul appel.

La méthode waapi.animate() supporte l'animation de plusieurs éléments DOM via des chaînes de sélecteur CSS, permettant des animations par lot avec support du stagger en un seul appel. Signature : waapi.animate(selector, properties, options). selector (String) : toute chaîne acceptée par document.querySelectorAll(). properties (Object) : propriétés d'animation (ex. translate, rotate). options (Object) : configuration d'animation. Fonctionnalités clés : accepte des chaînes de sélecteur CSS ciblant plusieurs éléments ; s'intègre avec l'utilitaire stagger() pour un timing séquentiel ; supporte les options d'animation standard delay, duration, loop, alternate. Comparée à la WAAPI native qui exige une sélection et une itération manuelle des éléments, Anime.js consolide tout en un seul appel déclaratif avec des valeurs par défaut sensées.

**Faits clés**

- Signature : waapi.animate(selector, properties, options)
- selector : String acceptée par document.querySelectorAll()
- Intégration avec stagger() pour timing séquentiel
- Supporte delay, duration, loop, alternate
- WAAPI native exige sélection + itération manuelle ; Anime.js = un seul appel déclaratif

```js
waapi.animate(selector, properties, options)
```

```js
import { waapi, stagger } from 'animejs';

waapi.animate('.circle', {
  translate: '17rem',
  delay: stagger(100),
  loop: true,
  alternate: true,
});
```

```js
document.querySelectorAll('.circle').forEach(($el, i) => {
  $el.animate({
    translate: '100px',
  }, {
    duration: 1000,
    delay: i * 100,
    easing: 'ease-out',
  }).finished.then(() => {
    $el.style.translate = '100px';
  })
});
```

### web-animation-api/improvements-to-the-web-animation-api/default-units

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/default-units`

> Anime.js applique automatiquement des unités par défaut (px, deg) aux propriétés qui en requièrent si aucune n'est spécifiée.

Lors de l'animation de propriétés qui nécessitent des unités (comme width, height, transforms), Anime.js applique automatiquement des unités par défaut sensées si aucune n'est spécifiée, ce qui simplifie la syntaxe par rapport à la WAAPI native. Propriétés et unités par défaut : x, y, z → 'px' ; translateX, translateY, translateZ → 'px' ; rotate, rotateX, rotateY, rotateZ → 'deg' ; skew, skewX, skewY → 'deg' ; perspective → 'px' ; width, height → 'px' ; margin, padding → 'px' ; top, right, bottom, left → 'px' ; borderWidth, fontSize, borderRadius → 'px'. Cette fonctionnalité élimine le besoin d'ajouter manuellement les unités, réduisant le boilerplate et améliorant la lisibilité pour les propriétés d'animation courantes.

**Faits clés**

- x, y, z et translateX/Y/Z → 'px'
- rotate, rotateX/Y/Z et skew, skewX/Y → 'deg'
- perspective, width, height, margin, padding, top/right/bottom/left → 'px'
- borderWidth, fontSize, borderRadius → 'px'
- Unités par défaut appliquées automatiquement si non spécifiées

```js
import { waapi } from 'animejs';

waapi.animate('.square', {
  opacity: .5,
  x: 250,
  rotate: 45,
  width: 40,
  height: 40,
});
```

```js
const $el = document.querySelector('.circle');

$el.animate({
  translate: '100px 50px',
  width: '150px',
  height: '80px',
}, {
  duration: 1000,
  easing: 'ease-out',
}).finished.then(() => {
  $el.style.translate = '100px';
});
```

### web-animation-api/improvements-to-the-web-animation-api/function-based-values

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/function-based-values`

> Permet des valeurs basées sur des fonctions (element, index) => value dans les propriétés WAAPI, pour une personnalisation par cible (v4.0.0+).

Cette fonctionnalité (Anime.js v4.0.0+) étend le support de la Web Animation API en permettant des valeurs basées sur des fonctions dans les définitions de propriétés, autorisant une personnalisation par cible sans itération manuelle des éléments. Signature des valeurs de propriété : (element, index) => value, où element est l'élément DOM animé, index est la position (base zéro) dans la collection de cibles, et le retour est la valeur calculée pour cette cible. Au lieu d'itérer manuellement sur les éléments et de calculer des valeurs individuelles, on passe des fonctions directement aux propriétés d'animation ; la bibliothèque les évalue pour chaque cible, calculant des valeurs uniques basées sur les données de l'élément ou la position d'index. Cela fonctionne pour les propriétés CSS comme pour les réglages d'animation tels que duration et delay. Comparé à la WAAPI native qui exige une itération manuelle, Anime.js élimine le boilerplate en gérant l'évaluation des fonctions en interne pour chaque cible.

**Faits clés**

- Signature : (element, index) => value
- element = élément DOM animé ; index = position base zéro dans la collection ; retour = valeur calculée pour la cible
- Fonctionne pour les propriétés CSS et pour duration / delay
- Disponible à partir de la version 4.0.0
- Élimine l'itération manuelle ; évaluation gérée en interne par cible

```js
(element, index) => value
```

```js
import { waapi, utils, stagger } from 'animejs';

waapi.animate('.square', {
  translate: () => `${utils.random(10, 17)}rem`,
  rotate: () => utils.random(-180, 180),
  scale: (_, i) => .25 + (i * .25),
  duration: $el => $el.dataset.duration,
  delay: stagger(100)
});
```

```js
<div class="small row">
  <div data-duration="400" class="square"></div>
</div>
<div class="small row">
  <div data-duration="600" class="square"></div>
</div>
```

### web-animation-api/improvements-to-the-web-animation-api/individual-css-transforms

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/individual-css-transforms`

> Anime.js WAAPI permet d'animer individuellement les proprietes CSS transform (x, y, rotate, scale, skew, etc.), contrairement au CSS standard ou au WAAPI natif.

Contrairement aux animations CSS standard ou au WAAPI natif, Anime.js permet d'animer separement les proprietes CSS individuelles de transform, offrant un controle plus fin. Les proprietes valides ont des raccourcis (shorthand) et des valeurs/unites par defaut. Limitations importantes: les transforms individuelles avec WAAPI ne fonctionnent que pour les navigateurs supportant CSS.registerProperty(propertyDefinition), avec un fallback sur aucune animation; et les transforms individuelles ne peuvent pas etre accelerees materiellement (hardware-accelerated).

**Faits clés**

- Proprietes valides (Property | Shorthand | Default | Unit): translateX | x | '0px' | 'px'
- translateY | y | '0px' | 'px'
- translateZ | z | '0px' | 'px'
- rotate | — | '0deg' | 'deg'
- rotateX | — | '0deg' | 'deg'
- rotateY | — | '0deg' | 'deg'
- rotateZ | — | '0deg' | 'deg'
- scale | — | '1' | —
- scaleX | — | '1' | —
- scaleY | — | '1' | —
- scaleZ | — | '1' | —
- skew | — | '0deg' | 'deg'
- skewX | — | '0deg' | 'deg'
- skewY | — | '0deg' | 'deg'
- Gotcha: ne fonctionne que si le navigateur supporte CSS.registerProperty(propertyDefinition), sinon fallback (aucune animation)
- Gotcha: les transforms individuelles ne peuvent pas etre hardware-accelerated

```js
import { waapi, utils } from 'animejs';

const $squares = utils.$('.square');

const animateSquares = () => {
  waapi.animate($squares, {
    x: () => utils.random(0, 17) + 'rem',
    y: () => utils.random(-1, 1) + 'rem',
    rotateX: () => utils.random(-90, 90),
    rotateY: () => utils.random(-90, 90),
    onComplete: () => animateSquares()
  });
}

animateSquares();
```

### web-animation-api/improvements-to-the-web-animation-api/individual-property-parameters

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/individual-property-parameters`

> Chaque propriete peut avoir ses propres parametres delay, duration et ease en passant un Object contenant au moins une propriete 'to' ou 'from'.

Chaque propriete peut avoir des parametres specifiques delay, duration et ease en passant un Object avec au moins une propriete 'to' ou 'from' comme valeur. Cette fonctionnalite (disponible depuis la version 4.0.0) permet un controle fin du timing et de l'easing par propriete avec le Web Animation API. Les parametres au niveau de la propriete surchargent les reglages globaux de l'animation; les valeurs peuvent etre des nombres simples ou des tableaux pour des sequences de keyframes.

**Faits clés**

- Signature: passer un Object avec au moins une propriete 'to' ou 'from' comme valeur d'une propriete animee
- Parametres par propriete acceptes: delay, duration, ease
- Les parametres au niveau propriete surchargent les reglages globaux
- Disponible depuis la version 4.0.0
- Les valeurs 'to'/'from' peuvent etre des nombres ou des tableaux (keyframes)

```js
import { waapi, utils, stagger } from 'animejs';

waapi.animate('.square', {
  y: {
    to: [0, -30, 0],
    ease: 'out(4)',
    duration: 1000,
  },
  rotate: { from: -180, to: 0, ease: 'out(3)' },
  scale: { to: [.65, 1, .65], ease: 'inOut(3)' },
  duration: 500,
  delay: stagger(75),
  loop: true,
});
```

```js
<div class="large row">
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
  <div class="square"></div>
</div>
```

### web-animation-api/improvements-to-the-web-animation-api/spring-and-custom-easings

`https://animejs.com/documentation/web-animation-api/improvements-to-the-web-animation-api/spring-and-custom-easings`

> Utilisation de la physique spring et des fonctions d'easing JS personnalisees avec le WAAPI Anime.js; les eases sont importes via 'eases' et 'spring'.

La documentation couvre l'utilisation de la physique spring et de fonctions d'easing JavaScript personnalisees avec le Web Animation API d'Anime.js (disponible depuis la version 4.0.0). Les fonctions d'easing integrees s'importent via 'eases' (ex: linear, outExpo, cubicBezier), et la fonction spring via 'spring'. L'easing par defaut est 'out(2)'. Les fonctions d'easing acceptent une syntaxe string ('linear', 'steps(10)', 'cubicBezier(.5,0,.5,1)', 'in(1.675)', 'out(1.675)', 'inOut(1.675)') ou une fonction.

**Faits clés**

- Import easings: import { eases } from 'animejs'; const { linear, outExpo, cubicBezier } = eases;
- Import spring: import { spring } from 'animejs';
- Easing par defaut: 'out(2)'
- Syntaxes/fonctions integrees: 'linear' / 'linear(0, .5 75%, 1)' -> linear() params (0, '.5 75%', 1)
- 'steps' / 'steps(10)' -> steps() steps=10
- 'cubicBezier' / 'cubicBezier(.5,0,.5,1)' -> cubicBezier() x1=.5 y1=0 x2=.5 y2=1
- 'in' / 'in(1.675)' -> in() power=1.675
- 'out' / 'out(1.675)' -> out() power=1.675
- 'inOut' / 'inOut(1.675)' -> inOut() power=1.675
- Disponible depuis la version 4.0.0

```js
import { eases } from 'animejs';

const { linear, outExpo, cubicBezier } = eases;
```

```js
import { spring } from 'animejs';
```

```js
import { waapi, utils, stagger, spring } from 'animejs';

waapi.animate('.circle', {
  y: [0, -30, 0],
  ease: spring({ stiffness: 150, damping: 5 }),
  delay: stagger(75),
  loop: true,
});
```

```js
<div class="large row">
  <div class="circle"></div>
  <div class="circle"></div>
  <div class="circle"></div>
  <div class="circle"></div>
  <div class="circle"></div>
  <div class="circle"></div>
</div>
```

### web-animation-api/api-differences-with-native-waapi

`https://animejs.com/documentation/web-animation-api/api-differences-with-native-waapi`

> Vue d'ensemble des differences majeures entre la syntaxe native element.animate() du WAAPI et la syntaxe waapi.animate() d'Anime.js.

Cette section couvre toutes les differences majeures entre la syntaxe native du Web Animation API element.animate() et la syntaxe Anime.js waapi.animate(element). Anime.js fusionne les valeurs de keyframes et les reglages de lecture (playback settings) dans un seul objet, alors que le WAAPI natif les separe en deux objets. Quatre differences majeures sont documentees dans les sous-sections: iterations (Anime.js utilise 'loop' au lieu de 'iterations'), direction (Anime.js utilise 'alternate'/'reversed' au lieu de 'direction'), easing (formats de valeurs d'easing differents), et finished (gestion des Promise differente).

**Faits clés**

- Anime.js fusionne Keyframes Values et Playback Settings dans un seul objet; le WAAPI natif les separe en deux arguments
- 4 differences majeures (sous-sections): iterations, direction, easing, finished
- iterations -> Anime.js utilise 'loop'
- direction -> Anime.js utilise 'alternate' (et 'reversed')
- easing -> formats de valeurs d'easing differents
- finished -> gestion de Promise differente

```js
waapi.animate(
  '.square',           // Targets
  {
    x: 100,            // Keyframes Values
    y: 50,
    opacity: .5,
    loop: 3,           // Playback Settings
    alternate: true,
    ease: 'out',
  }
);
```

```js
const $square = document.querySelector('.square');
$square.animate({
  translate: '100px 50px',  // Keyframes Values
  opacity: .5,
}, {
  iterations: 4,            // Playback Settings
  direction: 'alternate',
  easing: 'ease-out',
});
```

### web-animation-api/api-differences-with-native-waapi/iterations

`https://animejs.com/documentation/web-animation-api/api-differences-with-native-waapi/iterations`

> Anime.js remplace 'iterations' du WAAPI natif par 'loop' (Number | Boolean, defaut 0), avec une semantique de comptage differente.

Le parametre 'loop' determine combien de fois une animation se repete, remplacant la propriete native WAAPI 'iterations'. La conversion differe: ou WAAPI iterations: 3 signifie 3 lectures totales, Anime.js loop: 2 signifie 2 repetitions apres la lecture initiale. Type Number ou Boolean, plage [0, Infinity], defaut 0.

**Faits clés**

- Parametre: loop (remplace 'iterations' du WAAPI natif)
- Type: Number | Boolean
- Plage: [0, Infinity]
- Defaut: 0
- Mapping: 0 -> WAAPI 1 (pas de repetition)
- 2 -> WAAPI 3 (repete deux fois)
- Infinity / true / -1 -> WAAPI Infinity (indefiniment)
- false -> WAAPI 1 (pas de loop)
- Gotcha: loop compte les repetitions APRES la lecture initiale, alors que iterations compte les lectures totales

```js
import { waapi, stagger } from 'animejs';

waapi.animate('.square', {
  translate: '17rem',
  loop: 3,
  alternate: true,
  delay: stagger(100)
});
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
```

### web-animation-api/api-differences-with-native-waapi/direction

`https://animejs.com/documentation/web-animation-api/api-differences-with-native-waapi/direction`

> Anime.js remplace 'direction' du WAAPI natif par deux booleens separes 'reversed' et 'alternate' (defaut false chacun).

Anime.js remplace le parametre natif WAAPI 'direction' par deux parametres booleens separes pour un controle plus clair. Au lieu d'utiliser des strings comme 'forward', 'reverse', 'alternate' ou 'alternate-reverse', on utilise 'reversed' (controle le sens de lecture: avant vs arriere) et 'alternate' (controle le comportement de boucle: normal vs alternant). Cette combinaison fournit les memes quatre modes directionnels avec une API plus claire. Defaut false pour chacun.

**Faits clés**

- Parametres: reversed et alternate (remplacent 'direction')
- Type: Boolean
- Defaut: false pour chacun
- Mapping: reversed:false, alternate:false -> direction:'forward'
- reversed:true, alternate:false -> direction:'reverse'
- reversed:false, alternate:true -> direction:'alternate'
- reversed:true, alternate:true -> direction:'alternate-reverse'
- reversed = sens de lecture; alternate = comportement de boucle

```js
import { waapi, stagger } from 'animejs';

waapi.animate('.square', {
  translate: '17rem',
  reversed: true,
  delay: stagger(100)
});
```

```js
waapi.animate('.square', {
  x: 100,
  reversed: true,
  alternate: true,
  loop: 3
});
```

### web-animation-api/api-differences-with-native-waapi/easing

`https://animejs.com/documentation/web-animation-api/api-differences-with-native-waapi/easing`

> Anime.js remplace 'easing' du WAAPI natif par 'ease' (defaut 'out(2)' au lieu de 'linear'), acceptant noms/fonctions Anime.js ou noms d'easing WAAPI natifs.

Anime.js remplace le parametre natif WAAPI 'easing' par 'ease', qui accepte a la fois des fonctions d'easing personnalisees et des noms de fonctions d'easing WAAPI natifs. La bibliotheque convertit les expressions d'easing personnalisees en approximations linear() compatibles avec les specifications WAAPI. La valeur par defaut est 'out(2)' au lieu du 'linear' natif du WAAPI.

**Faits clés**

- Parametre: ease (remplace 'easing')
- Type: String (nom de fonction d'easing) | Function
- Defaut: 'out(2)' (au lieu du 'linear' natif WAAPI)
- Valeurs acceptees: tout nom/fonction d'easing Anime.js valide (/documentation/easings)
- Valeurs acceptees: tout nom de fonction d'easing WAAPI natif valide
- Anime.js convertit les easings personnalises en approximations linear() compatibles WAAPI

```js
waapi.animate('.square', {
  x: 100,
  ease: 'outElastic(1.25, .1)'
});
```

```js
import { waapi, stagger } from 'animejs';

waapi.animate('.square', {
  translate: '17rem',
  ease: 'inOut(6)',
  delay: stagger(100)
});
```

### web-animation-api/api-differences-with-native-waapi/finished

`https://animejs.com/documentation/web-animation-api/api-differences-with-native-waapi/finished`

> animation.then(callback) remplace la propriete native WAAPI animation.finished; renvoie une Promise executant un callback a la fin de l'animation.

animation.then() remplace la propriete native WAAPI animation.finished. Il renvoie une Promise qui execute un callback quand l'animation se termine. Le callback recoit l'instance d'animation comme premier argument. La methode fournit une gestion de completion basee sur les Promise, utilisable en inline ou dans des contextes async/await, ce qui est plus simple que de gerer manuellement la propriete native 'finished' du WAAPI.

**Faits clés**

- Methode: animation.then(callback) (remplace animation.finished du WAAPI natif)
- Parametre: callback (Function) — recoit l'instance d'animation comme premier argument
- Retourne: Promise
- Utilisable inline ou avec async/await

```js
animation.then(callback)
```

```js
waapi.animate(target, {
  translate: '100px',
  duration: 500,
}).then(callback);
```

```js
async function waitForAnimationToComplete() {
  return animate(target, {
    translate: '100px',
    duration: 500,
  });
}

const asyncAnimation = await waitForAnimationToComplete();
```

```js
import { waapi, utils } from 'animejs';

const [ $value ] = utils.$('.value');

const animation = waapi.animate('.circle', {
  translate: '16rem',
  loop: 2,
  alternate: true,
});

animation.then(() => $value.textContent = 'fulfilled');
```

### web-animation-api/waapi-convertease

`https://animejs.com/documentation/web-animation-api/waapi-convertease`

> waapi.convertEase(easingFunction) convertit n'importe quelle fonction d'easing JavaScript en un easing linear compatible WAAPI.

Convertit n'importe quelle fonction d'easing JavaScript en un easing linear compatible WAAPI, pour usage avec le Web Animation API. Cet utilitaire transforme des fonctions d'easing personnalisees (comme les spring eases ou les fonctions cubic-bezier) en un format que le Web Animation API natif peut consommer. La conversion produit une string d'easing linear compatible avec la methode animate() du WAAPI. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: waapi.convertEase(easingFunction)
- Parametre: easingFunction — fonction d'easing JavaScript a convertir
- Retourne: string d'easing linear compatible WAAPI
- Disponible depuis la version 4.0.0
- Usage typique: convertir spring.ease pour le passer a element.animate({...}, { easing })

```js
waapi.convertEase(easingFunction)
```

```js
import { waapi, spring } from 'animejs';

const spring = spring({ stiffness: 12 });
const linearEasing = waapi.convertEase(spring.ease);
```

```js
import { waapi, spring } from 'animejs';

const springs = [
  spring({ stiffness: 100 }),
  spring({ stiffness: 150 }),
  spring({ stiffness: 200 })
]

document.querySelectorAll('#web-animation-api-waapi-convertease .demo .square').forEach(($el, i) => {
  $el.animate({
    translate: '17rem',
    rotate: '1turn',
  }, {
    easing: waapi.convertEase(springs[i].ease),
    delay: i * 250,
    duration: springs[i].duration,
    fill: 'forwards'
  });
});
```


## engine

### engine

`https://animejs.com/documentation/engine`

> Le Engine pilote et synchronise toutes les instances Animation, Timer et Timeline d'Anime.js; ordre d'execution controlable via le parametre 'priority'.

Le Engine pilote et synchronise toutes les instances Animation, Timer et Timeline dans Anime.js. Il s'importe via 'import { engine } from 'animejs';'. Les instances s'executent dans l'ordre ou elles sont ajoutees; on controle la sequence d'execution avec le parametre 'priority' (les valeurs basses s'executent en premier; defaut 1). La documentation du Engine couvre: les Parametres (timeUnit, speed, fps, precision, pauseOnDocumentHidden), les Methodes (update(), pause(), resume()), les Proprietes (etat et configuration du moteur) et les Engine defaults (reglages globaux par defaut).

**Faits clés**

- Import: import { engine } from 'animejs';
- Le Engine pilote/synchronise toutes les instances Animation, Timer et Timeline
- Parametre priority: les valeurs basses s'executent en premier; defaut 1
- Parametres engine: timeUnit, speed, fps, precision, pauseOnDocumentHidden
- Methodes: update(), pause(), resume()
- Sections: Parameters, Methods, Properties, Engine defaults

```js
import { engine } from 'animejs';
```

```js
animate(targets, { x: 100, priority: 0 }); // Runs first
animate(targets, { y: 100, priority: 2 }); // Runs last
animate(targets, { z: 100 });              // Default priority: 1
```

### engine/engine-parameters

`https://animejs.com/documentation/engine/engine-parameters`

> Page d'index listant les cinq parametres du moteur Anime.js, accessibles via l'objet 'engine' importe de la bibliotheque.

La section Engine parameters documente les options de configuration du moteur d'animation d'Anime.js. Cinq parametres sont listes: timeUnit (seconds/milliseconds), speed, fps, precision, et pauseOnDocumentHidden. Les parametres s'accedent via l'objet 'engine' importe de la bibliotheque. Chaque parametre a une page de documentation dediee avec explications detaillees. La page d'index elle-meme contient les liens de navigation et titres de sections mais pas les details (signatures de types, valeurs par defaut, exemples) qui se trouvent sur les pages dediees a chaque parametre.

**Faits clés**

- Cinq parametres engine: timeUnit (seconds/milliseconds), speed, fps, precision, pauseOnDocumentHidden
- Acces via l'objet engine importe: import { engine } from 'animejs';
- Exemples d'acces: engine.speed, engine.fps, engine.precision
- Documentation associee a la version 4.0.0+
- La page d'index ne contient pas les details par parametre (sur pages dediees)

```js
import { engine } from 'animejs';
engine.speed
engine.fps
engine.precision
```

### engine/engine-parameters/timeunit-seconds-milliseconds

`https://animejs.com/documentation/engine/engine-parameters/timeunit-seconds-milliseconds`

> engine.timeUnit definit si les valeurs de temps (duration, delay) sont en secondes ('s') ou millisecondes ('ms'); defaut 'ms'.

Le parametre engine.timeUnit etablit si les valeurs liees au temps comme duration et delay sont interpretees en secondes ou millisecondes. Valeurs acceptees: 's' (secondes) ou 'ms' (millisecondes). Defaut 'ms'. Quand on change l'unite de temps, la duree par defaut actuellement definie est automatiquement ajustee a la nouvelle unite de temps specifiee.

**Faits clés**

- Signature: engine.timeUnit = 's' | 'ms'
- Valeurs acceptees: 's' (secondes), 'ms' (millisecondes)
- Defaut: 'ms'
- Affecte l'interpretation de duration et delay
- Gotcha: changer timeUnit ajuste automatiquement la duree par defaut courante a la nouvelle unite

```js
engine.timeUnit = 's' | 'ms'
```

```js
import { engine, animate, utils } from 'animejs';

const [ $timeS ] = utils.$('.time-s');
const [ $timeMs ] = utils.$('.time-ms');
const [ $ms, $s ] = utils.$('.toggle');

const secondsTimer = createTimer({
  duration: 1,
  loop: true,
  onUpdate: self => $timeS.innerHTML = utils.roundPad(self.iterationCurrentTime, 2)
});

const millisecondsTimer = createTimer({
  duration: 1000,
  loop: true,
  onUpdate: self => $timeMs.innerHTML = utils.roundPad(self.iterationCurrentTime, 2)
});

const toggleSetting = () => {
  const isUsingSeconds = engine.timeUnit === 's';
  engine.timeUnit = isUsingSeconds ? 'ms' : 's';
  $ms.disabled = isUsingSeconds;
  $s.disabled = !isUsingSeconds;
}

$ms.addEventListener('click', toggleSetting);
$s.addEventListener('click', toggleSetting);
```

### engine/engine-parameters/speed

`https://animejs.com/documentation/engine/engine-parameters/speed`

> engine.speed (Number >= 0, defaut 1) controle le taux de lecture global de toutes les animations gerees par le moteur.

Le parametre engine.speed controle le taux de lecture global (global playback rate) pour toutes les animations gerees par le moteur. Type Number (>= 0), defaut 1. Les valeurs superieures a 1 accelerent les animations; les valeurs entre 0 et 1 les ralentissent. Cela permet des effets simultanes de ralenti (slow-motion) ou d'avance rapide (fast-forward) sur l'ensemble de la suite d'animations.

**Faits clés**

- Signature: engine.speed = <Number>
- Type: Number (>= 0)
- Defaut: 1
- Valeurs > 1 accelerent; valeurs entre 0 et 1 ralentissent
- Affecte globalement toutes les animations gerees par le moteur

```js
engine.speed = <Number>
```

```js
import { engine, animate, utils } from 'animejs';

const [ $container ] = utils.$('.container');
const [ $range ] = utils.$('.range');

for (let i = 0; i < 150; i++) {
  const $particle = document.createElement('div');
  $particle.classList.add('particle');
  $container.appendChild($particle);
  animate($particle, {
    x: utils.random(-10, 10, 2) + 'rem',
    y: utils.random(-3, 3, 2) + 'rem',
    scale: [{ from: 0, to: 1 }, { to: 0 }],
    delay: utils.random(0, 1000),
    loop: true,
  });  
}

function onInput() {
  utils.sync(() => engine.speed = this.value);
}

$range.addEventListener('input', onInput);
```

```js
engine.speed = 0.5; // Run all animations at half speed
```

### engine/engine-parameters/fps

`https://animejs.com/documentation/engine/engine-parameters/fps`

> Le parametre engine.fps controle le frame rate global d'update/rendu des animations, defaut 240.

engine.fps est un Number (> 0) avec valeur par defaut 240. Il controle le frame rate global auquel toutes les animations sont mises a jour et rendues. Modifier ce reglage permet d'optimiser les performances sur des appareils peu puissants ou quand de nombreuses animations concurrentes sont gerees, mais peut impacter la fluidite. Un frame rate plus bas reduit la charge de calcul mais peut compromettre la fluidite visuelle ; le defaut 240 fps offre un rendu optimal dans la plupart des cas.

**Faits clés**

- Signature : engine.fps = number
- Type : Number (> 0)
- Valeur par defaut : 240
- Optimise les performances sur appareils faibles / nombreuses animations
- Frame rate bas = moins de charge mais moins de fluidite

```js
import { engine, animate, utils } from 'animejs';

const [ $container ] = utils.$('.container');
const [ $range ] = utils.$('.range');

for (let i = 0; i < 150; i++) {
  const $particle = document.createElement('div');
  $particle.classList.add('particle');
  $container.appendChild($particle);
  animate($particle, {
    x: utils.random(-10, 10, 2) + 'rem',
    y: utils.random(-3, 3, 2) + 'rem',
    scale: [{ from: 0, to: 1 }, { to: 0 }],
    delay: utils.random(0, 1000),
    loop: true,
  });  
}

function onInput() {
  engine.fps = this.value;
}

$range.addEventListener('input', onInput);
```

### engine/engine-parameters/precision

`https://animejs.com/documentation/engine/engine-parameters/precision`

> engine.precision controle l'arrondi des decimales pour les valeurs string durant les animations, defaut 4.

engine.precision est un Number, valeur par defaut 4. Il accepte un Number >= 0 pour definir le nombre de decimales d'arrondi, ou un Number < 0 pour ne pas arrondir du tout. Il s'applique exclusivement aux proprietes CSS, attributs SVG et attributs DOM (ex. '120.725px', '1.523'). L'arrondi n'a lieu que pendant les frames d'animation ; les premiere et derniere frames conservent la precision complete. Plus on ajoute de decimales, plus les animations sont precises ; en pratique une precision superieure a 4 est inutile (differences imperceptibles). Des valeurs plus basses ameliorent les performances quand on anime beaucoup d'elements, mais peuvent reduire la fluidite.

**Faits clés**

- Signature : engine.precision = Number
- Valeur par defaut : 4
- Number >= 0 = nombre de decimales d'arrondi ; Number < 0 = pas d'arrondi
- S'applique uniquement aux valeurs string (CSS, attributs SVG, attributs DOM)
- Arrondi seulement pendant les frames ; first/last frame gardent la precision pleine
- Precision > 4 = differences imperceptibles

```js
import { engine, animate, utils } from 'animejs';

const [ $container ] = utils.$('.container');
const [ $range ] = utils.$('.range');

for (let i = 0; i < 150; i++) {
  const $particle = document.createElement('div');
  $particle.classList.add('particle');
  $container.appendChild($particle);
  animate($particle, {
    x: utils.random(-10, 10, 2) + 'rem',
    y: utils.random(-3, 3, 2) + 'rem',
    scale: [{ from: 0, to: 1 }, { to: 0 }],
    delay: utils.random(0, 1000),
    loop: true,
  });  
}

function onInput() {
  engine.precision = this.value;
}

$range.addEventListener('input', onInput);
```

### engine/engine-parameters/pauseondocumenthidden

`https://animejs.com/documentation/engine/engine-parameters/pauseondocumenthidden`

> engine.pauseOnDocumentHidden (Boolean, defaut true) controle si les animations se mettent en pause quand l'onglet perd le focus.

engine.pauseOnDocumentHidden est un Boolean avec valeur par defaut true. Il controle le comportement des animations quand l'onglet du navigateur perd le focus. Active (defaut), les animations se mettent automatiquement en pause. Desactive, les animations ajustent leur currentTime pour compenser le temps passe en arriere-plan, donnant l'illusion d'une lecture continue. Note : changer d'onglet permet d'observer la difference entre les etats active et desactive.

**Faits clés**

- Signature : engine.pauseOnDocumentHidden = Boolean
- Valeur par defaut : true
- true = pause auto quand l'onglet est masque
- false = ajuste currentTime pour compenser le temps en arriere-plan (lecture continue illusoire)

```js
import { engine, utils, createTimer } from 'animejs';

const [ $globalTime ] = utils.$('.global-time');
const [ $engineTime ] = utils.$('.engine-time');
const [ $toggle ] = utils.$('.toggle');

const startTime = Date.now();

const globalTimer = setInterval(() => {
  $globalTime.innerHTML = Date.now() - startTime;
}, 16);

const engineTimer = createTimer({
  onUpdate: self => $engineTime.innerHTML = self.currentTime
});

const toggleSetting = () => {
  const isPauseWhenHidden = engine.pauseOnDocumentHidden;
  if (isPauseWhenHidden) {
    engine.pauseOnDocumentHidden = false;
    $toggle.innerHTML = '○ Disabled (Switch tab to see the effect)';
  } else {
    engine.pauseOnDocumentHidden = true;
    $toggle.innerHTML = '● Enabled (Switch tab to see the effect)';
  }
}

$toggle.addEventListener('click', toggleSetting);
```

### engine/engine-methods

`https://animejs.com/documentation/engine/engine-methods`

> Vue d'ensemble des trois methodes du moteur : update(), pause(), resume().

Le module Engine fournit trois methodes principales pour controler le fonctionnement du moteur d'animation : update(), pause() et resume(). Elles sont accessibles via l'objet engine importe depuis 'animejs'. La page index ne donne que les noms et liens ; les details (signatures, exemples) figurent sur les pages individuelles update(), pause() et resume().

**Faits clés**

- Trois methodes : update(), pause(), resume()
- Importees via : import { engine } from 'animejs'
- Navigation : precedent = pauseOnDocumentHidden ; suivant = update()

```js
import { engine } from 'animejs';

engine.update()
engine.pause()
engine.resume()
```

### engine/engine-methods/update

`https://animejs.com/documentation/engine/engine-methods/update`

> engine.update() avance manuellement le moteur d'une frame ; essentiel quand engine.useDefaultMainLoop est desactive.

engine.update() retourne l'instance Engine (chainable). La methode avance manuellement le moteur Anime.js d'une frame. Elle est essentielle quand engine.useDefaultMainLoop est desactive, ce qui permet de l'integrer dans des boucles d'animation externes comme Three.js ou des game engines. Dans l'exemple, on met engine.useDefaultMainLoop = false puis on appelle engine.update() depuis la boucle de rendu Three.js (renderer.setAnimationLoop).

**Faits clés**

- Signature : Engine.update(): Engine
- Retourne l'instance Engine (chainable)
- Avance le moteur d'une frame manuellement
- Necessaire quand engine.useDefaultMainLoop = false
- Permet l'integration dans boucles externes (Three.js, game engines)

```js
import { engine, animate, utils } from 'animejs';
import * as THREE from 'three';
import 'animejs/adapters/three';

// Prevents Anime.js from using its own loop
engine.useDefaultMainLoop = false;

const [ $container ] = utils.$('.full-container');
const color = utils.get($container, 'color');
const { width, height } = $container.getBoundingClientRect();

// Three.js setup
const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, preserveDrawingBuffer: true });
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(65, width / height, 0.1, 20);
const geometry = new THREE.BoxGeometry(1, 1, 1);
const material = new THREE.MeshBasicMaterial({ color, wireframe: true });

renderer.setSize(width, height);
renderer.setPixelRatio(window.devicePixelRatio);
$container.appendChild(renderer.domElement);
camera.position.z = 5;

function createAnimatedCube() {
  const cube = new THREE.Mesh(geometry, material);
  const r = () => utils.random(-720, 720, 1);
  scene.add(cube);
  animate(cube, {
    x: utils.random(-10, 10, 2),
    y: utils.random(-5, 5, 2),
    z: [-10, 7],
    rotateX: r,
    rotateY: r,
    rotateZ: r,
    delay: utils.random(0, 4000),
    duration: 4000,
    loop: true,
    ease: 'inSine',
  });
}

for (let i = 0; i < 40; i++) {
  createAnimatedCube();
}

function render() {
  engine.update(); // Manually update Anime.js engine
  renderer.render(scene, camera); // Render Three.js scene
}

renderer.setAnimationLoop(render);
```

### engine/engine-methods/pause

`https://animejs.com/documentation/engine/engine-methods/pause`

> engine.pause() stoppe la boucle principale du moteur et toutes les instances Timer/Animation/Timeline actives.

engine.pause() retourne l'instance Engine. Elle stoppe la boucle principale du moteur, mettant en pause toutes les instances Timer, Animation et Timeline actives. Les animations restent en pause jusqu'a ce que engine.resume() soit appele pour reprendre a la position de pause. De nouvelles instances Timer, Animation ou Timeline peuvent etre ajoutees pendant la pause mais ne s'executeront pas tant que le moteur n'a pas repris.

**Faits clés**

- Signature : Engine pause() (retourne instance Engine)
- Stoppe la boucle principale + toutes les instances Timer/Animation/Timeline actives
- Reprise via engine.resume() a la position de pause
- Nouvelles instances ajoutees pendant la pause ne s'executent qu'apres resume

```js
import { engine, animate, utils } from 'animejs';

const [ $container ] = utils.$('.container');
const [ $add, $pause ] = utils.$('button');

function addAnimation() {
  const $particle = document.createElement('div');
  $particle.classList.add('particle');
  $container.appendChild($particle);
  animate($particle, {
    x: utils.random(-10, 10, 2) + 'rem',
    y: utils.random(-3, 3, 2) + 'rem',
    scale: [{ from: 0, to: 1 }, { to: 0 }],
    loop: true,
  });
}

let timeout = 3;
let interval;

function pauseEngine() {
  engine.pause();
  $pause.setAttribute('disabled', 'true');
  $pause.innerHTML = `Resume in ${timeout--} seconds`;
  interval = setInterval(() => {
    if (timeout <= 0) {
      clearInterval(interval);
      engine.resume();
      $pause.removeAttribute('disabled');
      $pause.innerHTML = 'Pause for 3 seconds';
      timeout = 3;    
    } else {
      $pause.innerHTML = `Resume in ${timeout--} seconds`;
    }
  }, 1000);
}

$add.addEventListener('click', addAnimation);
$pause.addEventListener('click', pauseEngine);
```

### engine/engine-methods/resume

`https://animejs.com/documentation/engine/engine-methods/resume`

> engine.resume() relance le moteur apres une pause via engine.pause().

engine.resume() retourne une instance Engine. La methode relance le moteur d'animation apres qu'il a ete mis en pause via engine.pause(). Cette operation reprend toutes les animations qui tournaient quand le moteur a ete mis en pause. Pattern d'usage de base : engine.pause() met en pause le moteur et toutes les animations, engine.resume() les reprend.

**Faits clés**

- Signature : engine.resume(): Engine
- Retourne une instance Engine
- Relance le moteur apres engine.pause()
- Reprend toutes les animations qui tournaient a la pause

```js
import { engine, animate, utils } from 'animejs';

const [ $container ] = utils.$('.container');
const [ $pause, $resume ] = utils.$('button');

function addAnimation() {
  const $particle = document.createElement('div');
  $particle.classList.add('particle');
  $container.appendChild($particle);
  animate($particle, {
    x: utils.random(-10, 10, 2) + 'rem',
    y: utils.random(-3, 3, 2) + 'rem',
    scale: [{ from: 0, to: 1 }, { to: 0 }],
    loop: true,
    delay: utils.random(0, 1000)
  });
}

for (let i = 0; i < 150; i++) addAnimation();

const resumeEngine = () => engine.resume();
const pauseEngine = () => engine.pause();

$pause.addEventListener('click', pauseEngine);
$resume.addEventListener('click', resumeEngine);
```

```js
engine.pause();  // Pauses the engine and all animations
engine.resume(); // Resumes the engine and all animations
```

### engine/engine-properties

`https://animejs.com/documentation/engine/engine-properties`

> Liste des proprietes globales du moteur exposees via l'objet engine.

Le moteur d'Anime.js expose des proprietes pour controler son comportement global. Proprietes documentees : timeUnit ('ms' | 's', unite de temps pour duration/delay), currentTime (Number, temps actuel du moteur), deltaTime (Number, temps ecoule depuis la derniere frame), precision (Number, decimales arrondies pour valeurs texte), speed (Number, vitesse globale de lecture), fps (Number, frame rate global), useDefaultMainLoop (Boolean, utilise la boucle principale par defaut), pauseOnDocumentHidden (Boolean, pause quand l'onglet est masque). Aucune gotcha specifique documentee pour ces proprietes.

**Faits clés**

- timeUnit : 'ms' | 's' (unite de temps pour duration/delay)
- currentTime : Number (temps actuel du moteur)
- deltaTime : Number (temps ecoule depuis la derniere frame)
- precision : Number (decimales arrondies pour valeurs texte)
- speed : Number (vitesse globale de lecture)
- fps : Number (frame rate global)
- useDefaultMainLoop : Boolean (boucle principale par defaut)
- pauseOnDocumentHidden : Boolean (pause si onglet masque)

```js
import { engine } from 'animejs';

engine.timeUnit        // Acces a l'unite de temps
engine.currentTime     // Lecture du temps actuel
engine.deltaTime       // Lecture du temps ecoule
engine.precision       // Acces a la precision
engine.speed           // Acces a la vitesse globale
engine.fps             // Acces a la frequence d'images
engine.useDefaultMainLoop      // Acces au mode boucle
engine.pauseOnDocumentHidden   // Acces au parametre de pause
```

### engine/engine-defaults

`https://animejs.com/documentation/engine/engine-defaults`

> engine.defaults definit les proprietes globales par defaut heritees par toutes les instances Timer/Animation/Timeline.

Les engine defaults definissent les proprietes globales utilisees par toutes les instances de Timer, Animation et Timeline. Ces parametres peuvent etre modifies via l'objet defaults de engine. Parametres acceptes (avec leur type accepte) : playbackEase (Easing name String | Easing Function), playbackRate (Number), frameRate (Number), loop (Number | Boolean), reversed (Boolean), alternate (Boolean), autoplay (Boolean), duration (Number | Function), delay (Number | Function), composition (Composition types String | Function), ease (Easing name String | Easing Function), loopDelay (Number), modifier (Modifier Function), onBegin (Callback Function), onUpdate (Callback Function), onRender (Callback Function), onLoop (Callback Function), onComplete (Callback Function), onPause (Callback Function). Note : la page (telle que recuperee) ne precise pas les valeurs par defaut numeriques/booleennes initiales exactes pour chaque parametre.

**Faits clés**

- Acces via l'objet defaults de engine
- Heritees par toutes les instances Timer/Animation/Timeline
- Parametres : playbackEase, playbackRate, frameRate, loop, reversed, alternate, autoplay, duration, delay, composition, ease, loopDelay, modifier, onBegin, onUpdate, onRender, onLoop, onComplete, onPause
- Types : playbackEase/ease = Easing name String|Function ; playbackRate/frameRate/loopDelay = Number ; loop = Number|Boolean ; reversed/alternate/autoplay = Boolean ; duration/delay = Number|Function ; composition = Composition String|Function ; modifier = Modifier Function ; on* = Callback Function
- ATTENTION : l'exemple recupere montre 'engine.engine.defaults.duration = 500;' (probable artefact de fetch ; le pattern documente usuel est engine.defaults.duration = 500)
- Valeurs par defaut initiales exactes non precisees dans le contenu recupere

```js
import { engine } from 'animejs';

engine.engine.defaults.duration = 500;
```


## adapters

### adapters

`https://animejs.com/documentation/adapters`

> Les adapters permettent d'animer des objets sans proprietes directement exposees (ex. mesh Three.js, contexte canvas) avec l'API animate()/createTimeline()/utils.set().

Les adapters permettent d'animer des objets qui n'exposent pas directement leurs proprietes, comme une mesh Three.js ou un contexte canvas. Une fois enregistre, un adapter rend possible l'animation de ces objets avec l'API standard animate(), createTimeline() et utils.set(). Adapter integre : Three.js (seul adapter fourni actuellement), charge via le subpath 'animejs/adapters/three'. Adapters personnalises : creables pour tout type de cible via registerAdapter() importe depuis 'animejs/adapters'. Un Target Adapter declare un type de cible avec un ensemble fixe de noms de propriete (registerTargetAdapter + registerProperty(name, getter, setter, gate?)). Un Property Resolver gere les noms dynamiques ou multiples cibles (registerPropertyResolver retournant {get, set} ou null). Ordre de resolution : pour chaque nom anime, Anime.js verifie chaque adapter enregistre, d'abord les target adapters (premiere correspondance gagne), puis les property resolvers (premier resultat non-null). Les noms non reclames sont definis directement via target[name] = value.

**Faits clés**

- But : animer des objets sans proprietes directement exposees (mesh Three.js, contexte canvas)
- Compatible avec animate(), createTimeline(), utils.set()
- Adapter integre unique : Three.js via import 'animejs/adapters/three'
- registerAdapter() importe depuis 'animejs/adapters'
- registerTargetAdapter(predicate) + registerProperty(name, getter, setter, gate?)
- registerProperty params : name (String), getter (=> valeur), setter ((target,value,tween)=>), gate (optionnel, => Boolean)
- registerPropertyResolver((target,name) => {get,set} | null) pour noms dynamiques/multi-cibles
- Ordre de resolution : target adapters (1ere correspondance) puis property resolvers (1er non-null) puis target[name]=value

```js
import 'animejs/adapters/three';
```

```js
import { registerAdapter } from 'animejs/adapters';

const myAdapter = registerAdapter();
```

```js
const widget = myAdapter.registerTargetAdapter(target => target instanceof MyClass);

widget.registerProperty('foo',
  target => target.getFoo(),
  (target, value, tween) => target.setFoo(value),
  target => target.fooEnabled,
);
```

```js
myAdapter.registerPropertyResolver((target, name) => {
  if (target instanceof MyClass && name.startsWith('foo_')) {
    const key = name.slice(4);
    return {
      get: t => t.getFoo(key),
      set: (t, value, tween) => t.setFoo(key, value),
    };
  }
  return null;
});
```

### adapters/threejs-adapter

`https://animejs.com/documentation/adapters/threejs-adapter`

> L'adapter Three.js (import side-effect 'animejs/adapters/three') aplatit les hierarchies d'objets Three.js pour animer mesh, materiaux, textures et uniforms directement.

L'adapter Three.js se charge en import side-effect : import 'animejs/adapters/three'. Il fonctionne avec Three.js 0.150.0 et plus (peer dependency optionnelle). Il aplatit les hierarchies imbriquees d'objets Three.js, permettant d'animer directement les proprietes d'une mesh sans cibler manuellement mesh.position, mesh.rotation, mesh.material et les uniforms separement. Il convertit automatiquement les angles en degres et parse les valeurs de couleur CSS. Support des couleurs : valeurs CSS acceptees sur les couleurs de materiaux, lumieres, fog et fonds de scene. Gestion des vecteurs : les champs Vector2/3/4 sont decoupes en proprietes par-axe avec les suffixes X, Y, Z, W. Uniforms de shader : les uniforms de ShaderMaterial et les slots TSL UniformNode sont exposes par nom. Types de cibles supportes : Object3D et sous-classes (Mesh, lumieres, cameras, Sprite, Points), Material, Texture, Fog/FogExp2, TSL UniformNode, et instances Color/Vector2-4 nues. Pour les meshes batchees/instanced, recuperer les instances via getInstances(mesh) importe depuis 'animejs/adapters/three'.

**Faits clés**

- Import side-effect : import 'animejs/adapters/three'
- Three.js >= 0.150.0 (peer dependency optionnelle)
- Aplatit les hierarchies imbriquees (position/rotation/material/uniforms accessibles a plat)
- Conversion automatique angles -> degres ; parsing des couleurs CSS
- Vector2/3/4 decoupes par-axe via suffixes X, Y, Z, W
- Uniforms ShaderMaterial et slots TSL UniformNode exposes par nom
- Cibles : Object3D + sous-classes, Material, Texture, Fog/FogExp2, TSL UniformNode, Color/Vector2-4
- getInstances(mesh) (import depuis 'animejs/adapters/three') pour meshes batched/instanced

```js
import 'animejs/adapters/three';
```

```js
createTimeline({
  defaults: {
    duration: 500,
    ease: 'inOutSine',
  }
})
.add(mesh.position, { x: 100, y: 50 }, 0)
.add(mesh.rotation, {
  x: utils.degToRad(30),
  y: utils.degToRad(60),
}, 0)
.add(mesh.material, { opacity: 0.5 }, 0)
.add(mesh.material.uniforms.uTint.value, {
  r: 0, g: 0.5, b: 1,
}, 0);
```

```js
animate(mesh, {
  x: 100,           // mesh.position.x
  y: 50,            // mesh.position.y
  rotateX: 30,      // mesh.rotation.x (degrees)
  rotateY: 60,      // mesh.rotation.y (degrees)
  opacity: 0.5,     // mesh.material.opacity
  uTint: '#0080ff', // mesh.material.uniforms.uTint.value
  duration: 500,
  ease: 'inOutSine',
});
```

```js
animate(material, { color: '#ff8800', emissive: '#0ff' });
animate(scene, { background: 'rgb(20, 30, 40)' });
animate(light, { color: 'hsl(200, 80%, 50%)' });
animate(material, { color: 'var(--accent)' });
```

```js
animate(material, { normalScaleX: 0.5, normalScaleY: 1 });
animate(sprite, { centerX: 0.25 });
animate(texture, { offsetX: 1, offsetY: 0.5 });
```

```js
animate(shaderMaterial, { uTime: 1, uTint: '#0ff', uOffsetY: 0.5 });
animate(mesh, { uTime: 1, uTint: '#0ff' });
```

```js
import { getInstances } from 'animejs/adapters/three';
const instances = getInstances(mesh);
```

### adapters/threejs-adapter/threejs-object-property-adapter

`https://animejs.com/documentation/adapters/threejs-adapter/threejs-object-property-adapter`

> Mappings de proprietes Three.js (v4.5.0+) pour animer les champs/methodes integres a travers Object3D, Mesh, lumieres, cameras, scene, audio, fog, texture et uniforms.

L'adapter Three.js (v4.5.0+) fournit des mappings de proprietes pour animer les champs et methodes integres de Three.js. Object3D (toutes sous-classes : Mesh, Group, Sprite, Points, lumieres, cameras) : x/y/z -> target.position.[axis] ; rotateX/Y/Z -> target.rotation.[axis] (degres) ; scaleX/Y/Z -> target.scale.[axis] ; scale -> target.scale uniforme ; skewX/Y/Z -> shear sur axe (degres) ; transformOriginX/Y/Z -> decalage de pivot ; transformOrigin -> raccourci 'x y z'. Mesh : opacity -> target.material.opacity (bascule la visibilite a 0) ; color -> target.material.color (accepte couleurs CSS). Lumieres : color (toutes) ; groundColor (HemisphereLight). PerspectiveCamera : fov, aspect, focalLength, near, far, zoom (appelle automatiquement updateProjectionMatrix()). OrthographicCamera : left, right, top, bottom, near, far, zoom avec mises a jour de projection automatiques. Scene/Audio/Fog/Texture/UniformNode : couleur de background, controles de volume audio, couleur de fog, transforms de texture (offsetX/Y, repeatX/Y, centerX/Y, rotation), valeurs d'uniform TSL. Notes : les champs nommes par angle (rotation, angle) convertissent automatiquement degres<->radians ; les proprietes non listees passent en auto-detection pour Color, Vector, scalaire et booleen ; la rotation de Texture s'anime autour de target.center ; les mises a jour de matrice de projection sont automatiques pour les animations de camera.

**Faits clés**

- Disponible v4.5.0+
- Object3D : x/y/z (position), rotateX/Y/Z (rotation, degres), scaleX/Y/Z + scale (scale), skewX/Y/Z (shear, degres), transformOriginX/Y/Z + transformOrigin (pivot)
- Mesh : opacity (material.opacity, bascule visibilite a 0), color (material.color, couleurs CSS)
- Lumieres : color (toutes), groundColor (HemisphereLight)
- PerspectiveCamera : fov, aspect, focalLength, near, far, zoom -> updateProjectionMatrix() auto
- OrthographicCamera : left, right, top, bottom, near, far, zoom -> projection auto
- Scene/Audio/Fog/Texture/UniformNode : background, volume, fog color, texture offsetX/Y repeatX/Y centerX/Y rotation, uniforms TSL
- Champs angulaires (rotation, angle) : conversion degres<->radians automatique
- Proprietes non listees : auto-detection Color/Vector/scalaire/booleen
- Texture rotation s'anime autour de target.center

```js
import { createTimeline, createTimer, utils } from 'animejs';
import * as THREE from 'three';
import 'animejs/adapters/three';

const scene = new THREE.Scene();
const cube = new THREE.Mesh(
  new THREE.BoxGeometry(1, 1, 1),
  new THREE.MeshLambertMaterial({ color: 0xffffff })
);
scene.add(cube);

createTimeline({ defaults: { duration: 5000, ease: 'linear' } })
  .add(cube, {
    color: 'var(--hex-orange-1)',
    x: [-4, 0, 4],
    rotateZ: [360, 0, -360],
    alternate: true,
  }, 0);

createTimer({ onUpdate: () => renderer.render(scene, camera) });
```

### adapters/threejs-adapter/threejs-transforms-adapter

`https://animejs.com/documentation/adapters/threejs-adapter/threejs-transforms-adapter`

> Adapter de transforms etendus (v4.5.0+) : API de transforms type-CSS pour objets 3D Three.js et instanced meshes (position, rotation, scale, skew, transformOrigin par-axe).

L'adapter de transforms etendus Three.js (depuis v4.5.0) fournit une API de transforms type-CSS pour les objets 3D Three.js et les instanced meshes. Il permet l'animation par-axe de position, rotation et scale, en plus de proprietes additionnelles comme skew et transformOrigin. Proprietes supportees : x/y/z -> mesh.position ; rotateX/Y/Z -> mesh.rotation (degres) ; scaleX/Y/Z -> mesh.scale ; scale -> mesh.scale uniforme ; skewX/Y/Z -> shears angulaires (degres) ; transformOriginX/Y/Z -> decalages de pivot (unites de geometrie) ; transformOrigin -> raccourci string 3-tokens (x y z). Comportement de rotation : les animations supposent l'ordre Euler par defaut 'XYZ' ; changer rotation.order ne correspondra plus aux noms de propriete. Application du skew : applique apres les transformations position/rotation/scale ; ignore quand la valeur vaut zero. Bascule de visibilite automatique : mettre n'importe quel axe de scale ou opacity a 0 bascule target.visible a false ; restaurer des valeurs non-nulles restaure la visibilite. Cela ne se produit qu'a travers animate() ou utils.set() — les mutations directes contournent ce comportement.

**Faits clés**

- Disponible v4.5.0+ ; API de transforms type-CSS pour objets 3D et instanced meshes
- x/y/z -> mesh.position ; rotateX/Y/Z -> mesh.rotation (degres) ; scaleX/Y/Z + scale -> mesh.scale
- skewX/Y/Z -> shears angulaires (degres) ; transformOriginX/Y/Z -> pivot (unites de geometrie) ; transformOrigin -> string 3-tokens 'x y z'
- Rotation : suppose ordre Euler 'XYZ' par defaut ; changer rotation.order casse la correspondance des noms
- Skew applique APRES position/rotation/scale ; ignore si valeur = zero
- Visibilite auto : scale axis ou opacity a 0 -> target.visible = false ; restauration non-nulle -> visible
- Bascule visibilite uniquement via animate() / utils.set() ; mutations directes la contournent

```js
import { animate, createTimer, utils } from 'animejs';
import * as THREE from 'three';
import 'animejs/adapters/three';

// Three.js setup
const [ $container ] = utils.$('.full-container');
const color = utils.get($container, 'color');
const { width, height } = $container.getBoundingClientRect();

const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, preserveDrawingBuffer: true });
renderer.shadowMap.enabled = true;
renderer.setSize(width, height);
renderer.setPixelRatio(window.devicePixelRatio);
$container.appendChild(renderer.domElement);

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(35, width / height, 0.1, 100);
camera.position.set(0, 3, 8);
camera.lookAt(0, 0.2, 0);
const cameraRig = new THREE.Group().add(camera);
scene.add(cameraRig);

scene.add(new THREE.AmbientLight(0xffffff, 0.3));
const spot = new THREE.SpotLight(0xffffff, 100, 12, Math.PI / 5, 0.4);
spot.position.set(0, 5, 0);
spot.castShadow = true;
scene.add(spot);

const dirLight = new THREE.DirectionalLight(0xffffff, 0.5);
dirLight.position.set(2, 3, 4);
scene.add(dirLight);

const groundGeometry = new THREE.PlaneGeometry(12, 12);
const groundMaterial = new THREE.MeshLambertMaterial({ color });
const ground = new THREE.Mesh(groundGeometry, groundMaterial);
ground.rotation.x = -Math.PI / 2;
ground.receiveShadow = true;
scene.add(ground);

const gridColorA = utils.get($container, '--hex-current-1');
const gridColorB = utils.get($container, '--hex-current-3');
const grid = new THREE.GridHelper(12, 24, gridColorA, gridColorB);
grid.position.y = 0.001;
scene.add(grid);

const geometry = new THREE.BoxGeometry(1, 1, 1);
const cubes = [-2, 0, 2].map(x => {
  const cube = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial());
  cube.position.set(x, 0.5, 0);
  cube.castShadow = cube.receiveShadow = true;
  scene.add(cube);
  return cube;
});

// Animation with Three.js adapter
animate(cameraRig, {
  rotateY: 360,
  duration: 20000,
  loop: true,
  ease: 'linear',
});

utils.set(cubes[0], { color: 'var(--hex-red-1)' });
utils.set(cubes[1], { color: 'var(--hex-citrus-1)', transformOriginY: -0.5 });
utils.set(cubes[2], { color: 'var(--hex-green-1)', transformOrigin: '-.5 -.5 .5' });

// Cube 1: small up and down with rotation
animate(cubes[0], {
  y: [0.5, 1, 0.5],
  rotateY: 180,
  duration: 2000,
  loop: true,
  ease: 'inOutSine',
});

// Cube 2: skew left to right pivoting on its base
animate(cubes[1], {
  skewX: [-30, 30, 0, 0],
  skewZ: [0, 0, -30, 30],
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutSine',
});

// Cube 3: scale with the origin sliding from one corner to the opposite
animate(cubes[2], {
  scale: [1, .25, 1],
  transformOrigin: ['-.5 -.5 .5', '.5 .5 -.5'],
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutSine',
});

createTimer({ onUpdate: () => renderer.render(scene, camera) });
```


## timer/timer-methods

### timer/timer-methods/reverse

`https://animejs.com/documentation/timer/timer-methods/reverse`

> Force le timer a jouer en arriere et retourne l'instance pour le chainage.

reverse() force le timer a jouer en sens inverse (backward). La methode ne prend aucun parametre et retourne l'instance du timer, ce qui permet le chainage avec d'autres methodes de timer. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: reverse()
- Aucun parametre
- Retourne l'instance du timer (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $reverseButton ] = utils.$('.reverse');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  duration: 2000,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime,
});

const reverseTimer = () => timer.reverse();

$reverseButton.addEventListener('click', reverseTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">iteration time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button reverse">Reverse</button>
  </fieldset>
</div>
```

### timer/timer-methods/pause

`https://animejs.com/documentation/timer/timer-methods/pause`

> Met en pause un timer en cours sans reinitialiser son etat.

pause() met en pause un timer en cours d'execution. La methode arrete l'execution d'un timer actif sans reinitialiser son etat, preservant la position de temps actuelle. Aucun parametre. Retourne l'instance du timer, permettant le chainage avec d'autres methodes (ex. reverse(), restart()). Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: pause()
- Aucun parametre
- Retourne l'instance du timer (chainable)
- Ne reinitialise pas l'etat, preserve currentTime
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $pauseButton ] = utils.$('.pause');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  onUpdate: self => $time.innerHTML = self.currentTime
});

const pauseTimer = () => timer.pause();

$pauseButton.addEventListener('click', pauseTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button pause">Pause</button>
  </fieldset>
</div>
```

### timer/timer-methods/restart

`https://animejs.com/documentation/timer/timer-methods/restart`

> Reinitialise toutes les proprietes du timer et remet currentTime a 0.

restart() reinitialise toutes les proprietes du timer et remet currentTime a 0. Si autoplay est configure a true, le timer recommence automatiquement a jouer au redemarrage. Aucun parametre. Retourne l'instance du timer pour le chainage. Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: restart()
- Aucun parametre
- Remet currentTime a 0 et reinitialise toutes les proprietes
- Si autoplay=true, le timer rejoue automatiquement
- Retourne l'instance du timer (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $restartButton ] = utils.$('.restart');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  onUpdate: self => $time.innerHTML = self.currentTime
});

const restartTimer = () => timer.restart();

$restartButton.addEventListener('click', restartTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button restart">Restart</button>
  </fieldset>
</div>
```

### timer/timer-methods/alternate

`https://animejs.com/documentation/timer/timer-methods/alternate`

> Bascule la direction de lecture en ajustant currentTime pour refleter la nouvelle progression.

alternate() bascule la direction de lecture tout en ajustant la position currentTime pour refleter la nouvelle progression temporelle. Note de la documentation : seul iterationTime est reellement joue en sens inverse, car currentTime commence toujours a 0 et se termine a duration. Aucun parametre. Retourne l'instance du timer (signature alternate(): Timer). Utile pour inverser la direction d'animation en cours de lecture tout en maintenant un positionnement temporel correct dans les boucles.

**Faits clés**

- Signature: alternate(): Timer
- Aucun parametre
- Bascule la direction et ajuste currentTime
- Seul iterationTime est joue reellement en arriere (currentTime va toujours 0 -> duration)
- Retourne l'instance du timer (chainable)

```js
import { createTimer, utils } from 'animejs';

const [ $alternateButton ] = utils.$('.button');
const [ $iterationTime ] = utils.$('.iteration-time');

const timer = createTimer({
  duration: 10000,
  loop: true,
  onUpdate: self => {
    $iterationTime.innerHTML = self.iterationCurrentTime;
  }
});

const alternateTimer = () => timer.alternate();

$alternateButton.addEventListener('click', alternateTimer);
```

### timer/timer-methods/resume

`https://animejs.com/documentation/timer/timer-methods/resume`

> Reprend la lecture d'un timer en pause dans sa direction actuelle.

resume() reprend la lecture d'un timer mis en pause, dans sa direction actuelle. La methode continue un timer precedemment mis en pause sans reinitialiser son etat ni changer sa direction de lecture. Aucun parametre. Retourne l'instance du timer pour le chainage. Fonctionne de pair avec pause(). Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: resume()
- Aucun parametre
- Reprend dans la direction de lecture actuelle
- Fonctionne avec pause()
- Retourne l'instance du timer (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $resumeButton, $pauseButton, $alternateButton ] = utils.$('.button');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  duration: 2000,
  onUpdate: self => $time.innerHTML = self.iterationCurrentTime,
  loop: true,
});

const resumeTimer = () => timer.resume();
const pauseTimer = () => timer.pause();
const alternateTimer = () => timer.alternate();

$resumeButton.addEventListener('click', resumeTimer);
$pauseButton.addEventListener('click', pauseTimer);
$alternateButton.addEventListener('click', alternateTimer);
```

### timer/timer-methods/complete

`https://animejs.com/documentation/timer/timer-methods/complete`

> Termine instantanement le timer.

complete() termine un timer instantanement. La methode finit immediatement l'execution du timer et retourne l'instance, permettant le chainage avec d'autres methodes. Aucun parametre (signature complete(): Timer). Disponible depuis la version 4.0.0.

**Faits clés**

- Signature: complete(): Timer
- Aucun parametre
- Termine le timer instantanement
- Retourne l'instance du timer (chainable)
- Disponible depuis la version 4.0.0

```js
import { createTimer, utils } from 'animejs';

const [ $completeButton ] = utils.$('.complete');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  duration: 100000,
  onUpdate: self => $time.innerHTML = self.currentTime
});

const completeTimer = () => timer.complete();

$completeButton.addEventListener('click', completeTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button complete">Complete</button>
  </fieldset>
</div>
```

### timer/timer-methods/reset

`https://animejs.com/documentation/timer/timer-methods/reset`

> Met en pause le timer et reinitialise ses proprietes a leur etat initial; un softReset optionnel evite le rendu visuel.

reset(softReset) met en pause le timer et restaure ses proprietes a leur etat initial. Plus precisement, il reinitialise currentTime, progress, reversed, began et completed a leurs valeurs par defaut. Le parametre optionnel softReset (Boolean, defaut false) : si true, ne reinitialise que les valeurs internes sans declencher de rendu visuel. Retourne l'instance du timer pour le chainage.

**Faits clés**

- Signature: timer.reset(softReset)
- Parametre softReset: Boolean optionnel, defaut false — si true, reinitialise les valeurs internes sans rendu visuel
- Met le timer en pause et reinitialise currentTime, progress, reversed, began, completed
- Retourne l'instance du timer (chainable)

```js
import { createTimer, utils } from 'animejs';

const [ $time ] = utils.$('.time');
const [ $reset ] = utils.$('.button');

const timer = createTimer({
  onUpdate: self => $time.innerHTML = self.currentTime,
});

const resetTimer = () => {
  timer.reset();
  $time.innerHTML = timer.currentTime;
}

$reset.addEventListener('click', resetTimer);
```

```js
<div class="large centered row">
  <div class="half col">
    <pre class="large log row">
      <span class="label">current time</span>
      <span class="time value lcd">0</span>
    </pre>
  </div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Reset</button>
  </fieldset>
</div>
```

### timer/timer-methods/cancel

`https://animejs.com/documentation/timer/timer-methods/cancel`

> Met en pause le timer, le retire de la boucle du moteur et libere la memoire.

cancel() met en pause le timer, le retire de la boucle principale du moteur (engine's main loop) et libere la memoire. Aucun parametre (signature cancel(): Timer). Retourne l'instance du timer, permettant le chainage avec d'autres methodes comme play(), pause() ou reset().

**Faits clés**

- Signature: cancel(): Timer
- Aucun parametre
- Met en pause, retire de la boucle moteur et libere la memoire
- Retourne l'instance du timer (chainable)

```js
import { createTimer, utils } from 'animejs';

const [ $playButton ] = utils.$('.play');
const [ $cancelButton ] = utils.$('.cancel');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  onUpdate: self => $time.innerHTML = self.currentTime
});

const playTimer = () => timer.play();
const cancelTimer = () => timer.cancel();

$playButton.addEventListener('click', playTimer);
$cancelButton.addEventListener('click', cancelTimer);
```

### timer/timer-methods/revert

`https://animejs.com/documentation/timer/timer-methods/revert`

> Arrete l'execution du timer et detruit toute instance onScroll() associee.

revert() arrete l'execution du timer et detruit toute instance onScroll() (ScrollObserver) associee. A utiliser pour stopper et demanteler completement un timer ainsi que son ScrollObserver attache. Aucun parametre (signature revert(): Timer). Retourne l'instance du timer pour le chainage. Differe de cancel() en ce qu'il nettoie aussi les observateurs lies. Disponible depuis la version 4.0.0+.

**Faits clés**

- Signature: revert(): Timer
- Aucun parametre
- Arrete le timer et detruit l'instance onScroll() associee
- Differe de cancel() : nettoie aussi les observateurs (ScrollObserver) lies
- Retourne l'instance du timer (chainable)
- Disponible depuis la version 4.0.0+

```js
import { createTimer, utils } from 'animejs';

const [ $revertButton ] = utils.$('.revert');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  onUpdate: self => $time.innerHTML = self.currentTime
});

const revertTimer = () => {
  timer.revert();
  $time.innerHTML = timer.currentTime
}

$revertButton.addEventListener('click', revertTimer);
```

### timer/timer-methods/seek

`https://animejs.com/documentation/timer/timer-methods/seek`

> Met a jour currentTime et avance le timer a un instant precis; muteCallbacks optionnel pour ne pas declencher les callbacks.

seek(time, muteCallbacks) met a jour le currentTime du timer et l'avance a un instant precis, permettant une navigation directe vers n'importe quel point de la duree. Parametre time (Number) : le nouveau currentTime en millisecondes. Parametre optionnel muteCallbacks (Boolean, defaut false) : si true, empeche le declenchement des callbacks. Retourne l'instance du timer pour le chainage.

**Faits clés**

- Signature: timer.seek(time, muteCallbacks)
- time: Number — nouveau currentTime en millisecondes
- muteCallbacks: Boolean optionnel, defaut false — si true, empeche le declenchement des callbacks
- Retourne l'instance du timer (chainable)

```js
import { createTimer, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $playPauseButton ] = utils.$('.play-pause');
const [ $time ] = utils.$('.time');

const updateButtonLabel = timer => {
  $playPauseButton.textContent = timer.paused ? 'Play' : 'Pause';
}

const timer = createTimer({
  duration: 2000,
  autoplay: false,
  onUpdate: self => {
    $range.value = self.currentTime;
    $time.innerHTML = self.currentTime;
    updateButtonLabel(self);
  },
  onComplete: updateButtonLabel,
});

const seekTimer = () => timer.seek(+$range.value);

const playPauseTimer = () => {
  if (timer.paused) {
    timer.play();
  } else {
    timer.pause();
    updateButtonLabel(timer);
  }
}

$range.addEventListener('input', seekTimer);
$playPauseButton.addEventListener('click', playPauseTimer);
```

### timer/timer-methods/stretch

`https://animejs.com/documentation/timer/timer-methods/stretch`

> Recalcule la duree totale du timer pour correspondre a une duree specifiee, redistribuee sur les iterations.

stretch(duration) recalcule la duree totale d'un timer pour correspondre a un intervalle specifie. Parametre duration (Number) : la nouvelle duree totale en millisecondes. La duree totale est egale a la duree d'une iteration multipliee par le nombre total d'iterations. Par exemple, un timer de 1000ms par iteration bouclant deux fois a une duree totale de 3000ms ; appeler stretch() redistribue cette duree sur les iterations sans changer le nombre d'iterations. Retourne l'instance du timer pour le chainage.

**Faits clés**

- Signature: timer.stretch(duration)
- duration: Number — nouvelle duree totale en millisecondes
- La duree totale = duree d'une iteration x nombre total d'iterations
- Redistribue la duree sur les iterations sans changer leur nombre
- Retourne l'instance du timer (chainable)

```js
import { animate, utils } from 'animejs';

const [ $range ] = utils.$('.range');
const [ $duration ] = utils.$('.duration');
const [ $time ] = utils.$('.time');

const timer = createTimer({
  duration: 2000,
  onUpdate: self => $time.innerHTML = self.currentTime
});

const stretchTimer = () => {
  timer.stretch(+$range.value);
  $duration.innerHTML = timer.duration;
  timer.restart();
}

$range.addEventListener('input', stretchTimer);
```


## timer/timer-properties

### timer/timer-properties

`https://animejs.com/documentation/timer/timer-properties`

> Liste exhaustive des proprietes accessibles (get/set) sur une instance Timer retournee par createTimer().

Cette section liste toutes les proprietes disponibles sur une instance Timer retournee par createTimer(). Chaque propriete est en lecture/ecriture (get/set) sauf deltaTime et backwards qui sont en lecture seule. id (String | Number) : obtient et definit l'identifiant du timer. deltaTime (Number) : obtient le temps ecoule en ms entre l'image actuelle et la precedente. currentTime (Number) : obtient et definit le temps global actuel en ms du timer. iterationCurrentTime (Number) : obtient et definit le temps de l'iteration actuelle en ms. progress (Number) : obtient et definit la progression globale du timer de 0 a 1. iterationProgress (Number) : obtient et definit la progression de l'iteration actuelle de 0 a 1. currentIteration (Number) : obtient et definit le compteur d'iteration actuel. speed (Number) : obtient et definit le multiplicateur playbackRate du timer. fps (Number) : obtient et definit le frameRate du timer. paused (Boolean) : obtient et definit si le timer est en pause. began (Boolean) : obtient et definit si le timer a demarre. completed (Boolean) : obtient et definit si le timer est termine. reversed (Boolean) : obtient et definit si le timer est inverse. backwards (Boolean) : obtient si le timer joue actuellement en arriere. Aucun exemple de code n'est fourni dans cette section.

**Faits clés**

- id: String | Number — get/set identifiant du timer
- deltaTime: Number — get temps ecoule en ms entre image actuelle et precedente (lecture seule)
- currentTime: Number — get/set temps global actuel en ms
- iterationCurrentTime: Number — get/set temps de l'iteration actuelle en ms
- progress: Number — get/set progression globale 0 a 1
- iterationProgress: Number — get/set progression de l'iteration actuelle 0 a 1
- currentIteration: Number — get/set compteur d'iteration actuel
- speed: Number — get/set multiplicateur playbackRate
- fps: Number — get/set frameRate du timer
- paused: Boolean — get/set si en pause
- began: Boolean — get/set si demarre
- completed: Boolean — get/set si termine
- reversed: Boolean — get/set si inverse
- backwards: Boolean — get si joue actuellement en arriere (lecture seule)


## animation-callbacks

### animation/animation-callbacks/onpause

`https://animejs.com/documentation/animation/animation-callbacks/onpause`

> Callback executed when a running animation pauses, manually or automatically. Receives the animation instance.

Le callback onPause (type Function, defaut noop) s'execute quand une animation en cours se met en pause, que ce soit declenche manuellement ou automatiquement. La mise en pause se produit lorsque : la methode .pause() est invoquee ; la methode .cancel() est invoquee ; la methode .revert() est invoquee ; tous les tweens de l'animation sont remplaces par une autre animation avec composition: 'replace' ; toutes les cibles de l'animation sont supprimees. Le callback recoit l'instance d'animation comme premier argument. Peut etre defini globalement via engine.defaults.onPause.

**Faits clés**

- Type: Function
- Default: noop
- Signature: onPause: (animation) => void
- Recoit l'instance d'animation comme premier argument
- Declenche par .pause(), .cancel(), .revert(), composition:'replace' sur tous les tweens, ou suppression de toutes les cibles
- Defaut global: engine.defaults.onPause
- Available Since: v4.0.0

```js
onPause: (animation) => void
```

```js
import { engine } from 'animejs';
engine.defaults.onPause = self => console.log(self.id);
```

```js
import { animate, utils } from 'animejs';

const [ $circle ] = utils.$('.circle');
let paused = 0;

const animation = animate($circle, {
  x: '16rem',
  duration: 2000,
  onPause: () => console.log(++paused),
});

animation.pause();
```

### animation/animation-callbacks/then

`https://animejs.com/documentation/animation/animation-callbacks/then`

> Returns a Promise resolved on animation completion, enabling chaining or async/await.

La methode then(callback) retourne une Promise qui se resout a la fin de l'animation, permettant l'execution chainee d'un callback ou des patterns async/await. Le parametre callback (Function) est execute quand l'animation se termine et recoit l'instance d'animation comme premier argument. S'integre avec l'API Promise native de JavaScript et facilite l'orchestration sequentielle d'animations.

**Faits clés**

- Signature: animation.then(callback)
- Parametre callback (Function): execute a la fin de l'animation, recoit l'instance d'animation comme premier argument
- Returns: Promise
- Compatible avec async/await
- Available since: v4.0.0

```js
animation.then(callback)
```

```js
animate(target, {x: 100, duration: 500}).then(callback);
```

```js
async function waitForAnimationToComplete() {
  return animate(target, {
    x: 100,
    duration: 500,
  });
}

const asyncAnimation = await waitForAnimationToComplete();
```

```js
import { animate } from 'animejs';

const [ $value ] = utils.$('.value');

const animation = animate('.circle', {
  x: '16rem',
  delay: 500,
});

animation.then(() => $value.textContent = 'fulfilled');
```


## animation-methods

### animation/animation-methods

`https://animejs.com/documentation/animation/animation-methods`

> Overview page listing the control methods available on an Animation instance returned by animate().

Cette page d'apercu documente les methodes de controle disponibles sur l'instance Animation retournee par une fonction animate(), fournissant le controle sur le timing, le comportement et la progression d'une animation. Elle liste les methodes suivantes : play(), reverse(), pause(), restart(), alternate(), resume(), complete(), cancel(), revert(), reset(), seek(), stretch(), refresh(). La page fournit des liens de navigation vers les pages de documentation individuelles de chaque methode (signatures detaillees, parametres, exemples de code sur les pages dediees).

**Faits clés**

- Methodes disponibles sur l'instance Animation retournee par animate()
- Liste: play(), reverse(), pause(), restart(), alternate(), resume(), complete(), cancel(), revert(), reset(), seek(), stretch(), refresh()
- Controle du timing, du comportement et de la progression de l'animation
- Page d'apercu sans signatures/exemples detailles (renvoie aux pages individuelles)

### animation/animation-methods/play

`https://animejs.com/documentation/animation/animation-methods/play`

> Forces the animation to play forward; returns the animation (chainable).

La methode play() force l'animation a jouer en avant (forward). Elle reprend ou demarre la lecture en avant d'une animation potentiellement en pause ou arretee. Retourne l'animation elle-meme (chainable avec d'autres methodes d'animation).

**Faits clés**

- Signature: play()
- Force l'animation a jouer en avant
- Returns: l'animation elle-meme (chainable)
- Available since: v1.0.0

```js
play()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $playButton ] = utils.$('.play');

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  delay: stagger(100),
  autoplay: false, // The animation is paused by default
});

const playAnimation = () => animation.play();

$playButton.addEventListener('click', playAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button play">Play</button>
  </fieldset>
</div>
```

### animation/animation-methods/reverse

`https://animejs.com/documentation/animation/animation-methods/reverse`

> Forces the animation to play backward; returns the animation (chainable).

La methode reverse() force une animation a jouer a l'envers (backward). Elle retourne l'animation elle-meme, permettant le chainage de methodes avec d'autres operations d'animation comme play() et pause().

**Faits clés**

- Signature: reverse()
- Force l'animation a jouer a l'envers (backward)
- Returns: l'animation elle-meme (chainable)
- Available since: v1.0.0

```js
reverse()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $reverseButton ] = utils.$('.reverse');

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  delay: stagger(100),
});

const reverseAnimation = () => animation.reverse();

$reverseButton.addEventListener('click', reverseAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button reverse">Reverse</button>
  </fieldset>
</div>
```

### animation/animation-methods/pause

`https://animejs.com/documentation/animation/animation-methods/pause`

> Halts a currently executing animation; returns the animation (chainable).

La methode pause() arrete une animation en cours d'execution. Aucun parametre requis. Elle retourne l'instance d'animation elle-meme, permettant le chainage de methodes avec d'autres operations d'animation.

**Faits clés**

- Signature: pause()
- Aucun parametre requis
- Arrete une animation en cours d'execution
- Returns: l'instance d'animation (chainable)
- Available since: v1.0.0

```js
pause()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $pauseButton ] = utils.$('.pause');

const animation = animate('.square', {
  x: '17rem',
  alternate: true,
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const pauseAnimation = () => animation.pause();

$pauseButton.addEventListener('click', pauseAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button pause">Pause</button>
  </fieldset>
</div>
```

### animation/animation-methods/restart

`https://animejs.com/documentation/animation/animation-methods/restart`

> Resets all animation properties and sets currentTime to 0; plays if autoplay is enabled.

La methode restart() reinitialise toutes les proprietes de l'animation et place currentTime a 0. Si autoplay est active, l'animation demarre automatiquement la lecture. Aucun parametre requis. Elle retourne l'instance d'animation, permettant le chainage de methodes. La methode respecte le reglage autoplay configure lors de la creation de l'animation.

**Faits clés**

- Signature: restart()
- Aucun parametre requis
- Reinitialise toutes les proprietes de l'animation et place currentTime a 0
- Si autoplay est active, l'animation joue automatiquement
- Returns: l'instance d'animation (chainable)
- Respecte le reglage autoplay defini a la creation
- Available since: v1.0.0

```js
restart()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $restartButton ] = utils.$('.restart');

const animation = animate('.square', {
  x: '17rem',
  direction: 'alternate',
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100)
});

const restartAnimation = () => animation.restart();

$restartButton.addEventListener('click', restartAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button restart">Restart</button>
  </fieldset>
</div>
```

### animation/animation-methods/alternate

`https://animejs.com/documentation/animation/animation-methods/alternate`

> Toggles playback direction while adjusting currentTime to reflect the new time progress.

La methode alternate() bascule la direction de lecture tout en ajustant la position currentTime pour refleter la nouvelle progression temporelle. Cela permet d'inverser la direction de l'animation en cours de lecture tout en maintenant une progression fluide. Aucun parametre requis. Retourne l'instance d'animation (chainable). Fonctionne avec les animations en boucle pour inverser dynamiquement la direction.

**Faits clés**

- Signature: alternate()
- Aucun parametre requis
- Bascule la direction de lecture en ajustant currentTime pour refleter la nouvelle progression
- Returns: l'instance d'animation (chainable)
- Fonctionne avec les animations en boucle (loop) pour inverser dynamiquement la direction
- Available since: v1.0.0

```js
alternate()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $alternateButton ] = utils.$('.button');

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const alternateAnimation = () => animation.alternate();

$alternateButton.addEventListener('click', alternateAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button">Alternate</button>
  </fieldset>
</div>
```

### animation/animation-methods/resume

`https://animejs.com/documentation/animation/animation-methods/resume`

> Resumes playback of a paused animation in its current direction; returns the animation (chainable).

La methode resume() reprend la lecture d'une animation en pause dans sa direction courante. Elle permet de continuer une animation precedemment mise en pause a partir de l'endroit ou elle s'etait arretee, en conservant la direction de lecture d'origine. Retourne l'instance d'animation (chainable).

**Faits clés**

- Signature: resume()
- Reprend la lecture d'une animation en pause dans sa direction courante
- Conserve la direction de lecture d'origine
- Returns: l'instance d'animation (chainable)

```js
resume()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $pauseButton, $alternateButton, $resumeButton ] = utils.$('.button');

const animation = animate('.square', {
  x: '17rem',
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const pauseAnimation = () => animation.pause();
const alternateAnimation = () => animation.alternate();
const resumeAnimation = () => animation.resume();

$pauseButton.addEventListener('click', pauseAnimation);
$alternateButton.addEventListener('click', alternateAnimation);
$resumeButton.addEventListener('click', resumeAnimation);
```

### animation/animation-methods/complete

`https://animejs.com/documentation/animation/animation-methods/complete`

> Completes the animation instantly; returns the animation (chainable).

La methode complete() termine l'animation instantanement. Elle fait avancer immediatement l'animation a son etat final sans jouer la duree restante. Aucun parametre requis. Retourne l'objet animation (chainable avec d'autres methodes d'animation).

**Faits clés**

- Signature: complete()
- Aucun parametre requis
- Termine l'animation instantanement (avance a l'etat final sans jouer la duree restante)
- Returns: l'objet animation (chainable)

```js
complete()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $completeButton ] = utils.$('.complete');

const animation = animate('.square', {
  x: '17rem',
  alternate: true,
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const completeAnimation = () => animation.complete();

$completeButton.addEventListener('click', completeAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button complete">Complete</button>
  </fieldset>
</div>
```

### animation/animation-methods/cancel

`https://animejs.com/documentation/animation/animation-methods/cancel`

> Pauses the animation, removes it from the engine's main loop, and frees up memory.

La methode cancel() met l'animation en pause, la retire de la boucle principale du moteur (engine's main loop) et libere de la memoire. Retourne l'animation elle-meme (chainable). Elle differe de complete() ou pause() car elle libere la memoire et retire l'animation de la boucle de traitement du moteur.

**Faits clés**

- Signature: cancel()
- Met l'animation en pause, la retire de la boucle principale du moteur, et libere la memoire
- Returns: l'animation elle-meme (chainable)
- Differe de complete()/pause() car libere la memoire et retire de la boucle de traitement
- Available since: v4.0.0
- Une animation annulee peut etre relancee via play()

```js
cancel()
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $cancelButton ] = utils.$('.cancel');
const [ $playButton ] = utils.$('.play');

const animation = animate('.square', {
  x: '17rem',
  alternate: true,
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const cancelAnimation = () => animation.cancel();
const playAnimation = () => animation.play();

$cancelButton.addEventListener('click', cancelAnimation);
$playButton.addEventListener('click', playAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button cancel">Cancel</button>
    <button class="button play">Play</button>
  </fieldset>
</div>
```

### animation/animation-methods/revert

`https://animejs.com/documentation/animation/animation-methods/revert`

> Stops the animation entirely and restores all animated values to their original state.

La methode revert() arrete entierement une animation et restaure toutes les valeurs animees a leur etat d'origine. Elle supprime egalement tout style CSS inline applique et nettoie les instances onScroll() liees le cas echeant. A utiliser quand on a besoin de terminer et detruire completement une animation plutot que de simplement la mettre en pause. Aucun parametre requis. Retourne l'instance d'animation (chainable).

**Faits clés**

- Signature: revert() → Animation
- Aucun parametre requis
- Arrete entierement l'animation et restaure toutes les valeurs animees a leur etat d'origine
- Supprime les styles CSS inline appliques
- Nettoie les instances onScroll() liees
- Returns: l'instance d'animation (chainable)

```js
revert() → Animation
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $revertButton ] = utils.$('.revert');
const [ $restartButton ] = utils.$('.restart');

// Set an initial translateX value
utils.set('.square', { x: '17rem' });

const animation = animate('.square', {
  x: 0,
  alternate: true,
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const revertAnimation = () => animation.revert();
const restartAnimation = () => animation.restart();

$revertButton.addEventListener('click', revertAnimation);
$restartButton.addEventListener('click', restartAnimation);
```

```js
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <div class="square"></div>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button class="button revert">Revert</button>
    <button class="button restart">Restart</button>
  </fieldset>
</div>
```

### animation/animation-methods/reset

`https://animejs.com/documentation/animation/animation-methods/reset`

> Pauses the animation and resets currentTime, progress, reversed, began and completed to initial state; optional softReset.

La methode reset(softReset) met l'animation en pause et restaure les proprietes currentTime, progress, reversed, began et completed a leur etat initial. Le parametre softReset (Boolean, optionnel, defaut false) : si true, ne reinitialise que les valeurs internes sans declencher de rendu visuel. Un hard reset (defaut) declenche une mise a jour visuelle, tandis qu'un soft reset ne modifie que les valeurs internes. Retourne l'objet animation (chainable).

**Faits clés**

- Signature: animation.reset(softReset)
- Parametre softReset: Boolean, optionnel, defaut false — si true, ne reinitialise que les valeurs internes sans rendu visuel
- Met l'animation en pause et restaure currentTime, progress, reversed, began, completed a leur etat initial
- Hard reset (defaut) declenche une mise a jour visuelle ; soft reset modifie uniquement les valeurs internes
- Returns: l'objet animation (chainable)
- Available since: v3.0.0

```js
animation.reset(softReset);
```

```js
import { animate, utils, stagger } from 'animejs';

const [ $hardReset, $softReset ] = utils.$('.button');

const animation = animate('.square', {
  x: '17rem',
  alternate: true,
  ease: 'inOutSine',
  loop: true,
  delay: stagger(100),
});

const hardReset = () => animation.reset();
const softReset = () => animation.reset(true);

$hardReset.addEventListener('click', hardReset);
$softReset.addEventListener('click', softReset);
```


## events / onScroll / ScrollObserver settings

### events/onscroll/scrollobserver-settings/container

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings/container`

> Le parametre container definit l'HTMLElement (selecteur CSS ou element DOM) qui recoit le listener de scroll pour onScroll().

container specifie quel HTMLElement recoit l'event listener de scroll. Quand il est defini, le declencheur autoplay de l'animation repond aux events de scroll a l'interieur de ce conteneur specifique plutot que de la fenetre. Il accepte soit une chaine de selecteur CSS, soit une reference directe a un element DOM. Disponible depuis la version 4.0.0.

**Faits clés**

- Parametre: container
- Type: CSS Selector | DOM Element
- Default: null
- Disponible depuis 4.0.0
- Accepte un selecteur CSS string ou une reference directe a un element DOM

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container'
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-settings/target

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings/target`

> Le parametre target definit l'element HTML qui declenche l'observer de scroll; sa position relative au container determine quand les callbacks s'executent.

target specifie quel element HTML declenche l'observer d'event de scroll. La position de cet element relative au conteneur de scroll determine quand les callbacks bases sur le scroll s'executent. Il accepte un selecteur CSS (chaine, ex. '.timer') ou une reference directe a un objet HTMLElement. Valeur par defaut: s'il est defini sur une animation, le premier HTMLElement cible de l'animation; null s'il est defini en dehors d'une animation.

**Faits clés**

- Parametre: target
- Type: HTMLElement | string | null
- Default: premier HTMLElement cible de l'animation si defini sur une animation; null sinon
- Accepte un selecteur CSS string ou une reference directe a un HTMLElement

```js
import { createTimer, utils, onScroll } from 'animejs';

const [ $timer ] = utils.$('.timer');

createTimer({
  duration: 2000,
  alternate: true,
  loop: true,
  onUpdate: self => {
    $timer.innerHTML = self.iterationCurrentTime
  },
  autoplay: onScroll({
    target: $timer,
    container: '.scroll-container',
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large centered row">
        <pre class="large log row">
          <span class="label">timer</span>
          <span class="timer value lcd">0</span>
        </pre>
      </div>
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-settings/debug

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings/debug`

> debug (Boolean, default false) affiche des marqueurs visuels (regle coloree) montrant les seuils enter/leave de chaque ScrollObserver.

debug active des marqueurs visuels qui aident a comprendre ou sont positionnes les seuils enter et leave de l'animation de scroll. Quand active, chaque instance de ScrollObserver affiche sa propre regle codee par couleur. Le cote gauche indique le seuil du conteneur, tandis que le cote droit montre les valeurs de seuil de la cible (target).

**Faits clés**

- Parametre: debug
- Type: Boolean
- Default: false
- Cote gauche de la regle = seuil container; cote droit = seuil target
- Chaque ScrollObserver affiche sa propre regle codee par couleur

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container',
    debug: true,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll up</div>
      </div>
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-settings/axis

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings/axis`

> axis ('x' | 'y', default 'y') determine la direction de scroll surveillee par le container du ScrollObserver.

axis determine quelle direction de scroll le conteneur du ScrollObserver surveille. Le regler a 'x' observe le scroll horizontal, tandis que 'y' observe le scroll vertical (comportement standard).

**Faits clés**

- Parametre: axis
- Type: 'x' | 'y'
- Default: 'y'
- 'x' = scroll horizontal, 'y' = scroll vertical (standard)

```js
import { animate, utils, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container',
    axis: 'x',
  })
});
```

```js
<div class="full-container scroll-container scroll-x">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll right →</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-settings/repeat

`https://animejs.com/documentation/events/onscroll/scrollobserver-settings/repeat`

> repeat (Boolean, default true) determine si la synchronisation au scroll persiste apres la fin de l'animation liee; false revert l'instance scrollContainer.

repeat determine si la synchronisation au scroll persiste apres que l'animation liee se termine. Quand il est regle a false, l'instance scrollContainer sera revertie (revoked/reverted). repeat: true maintient la synchronisation au scroll a travers plusieurs cycles de declenchement, tandis que repeat: false desactive le comportement de repetition.

**Faits clés**

- Parametre: repeat
- Type: Boolean
- Default: true
- false => l'instance scrollContainer est revertie
- true maintient la synchronisation a travers plusieurs cycles de declenchement

```js
import { createTimer, onScroll, utils } from 'animejs';

const [ $repeat ] = utils.$('.repeat .value');
const [ $noRepeat ] = utils.$('.no-repeat .value');

let repeatUpdates = 0;
let noRepeatUpdates = 0;

createTimer({
  duration: 1000,
  autoplay: onScroll({
    container: '.scroll-container',
    target: '.repeat',
    enter: 'bottom-=40 top',
    leave: 'top+=60 bottom',
    onUpdate: () => $repeat.innerHTML = repeatUpdates++,
    repeat: true,
    debug: true,
  })
});

createTimer({
  duration: 1000,
  autoplay: onScroll({
    container: '.scroll-container',
    target: '.no-repeat',
    enter: 'bottom-=40 top',
    leave: 'top+=60 bottom',
    onUpdate: () => $noRepeat.innerHTML = noRepeatUpdates++,
    repeat: false,
    debug: true,
  })
});
```


## events / onScroll / ScrollObserver thresholds

### events/onscroll/scrollobserver-thresholds

`https://animejs.com/documentation/events/onscroll/scrollobserver-thresholds`

> Les thresholds (enter / leave) determinent quand les actions de scroll se declenchent en comparant les positions target et container; trois syntaxes acceptees.

Les thresholds determinent quand les actions basees sur le scroll se declenchent en comparant les positions de la cible (target) et du conteneur (container). Ils sont definis via les proprietes enter et leave dans les parametres d'onScroll(). enter et leave acceptent trois variantes de syntaxe: (1) Syntaxe objet { target: 'top'|'bottom'|'start'|'end', container: 'top'|'bottom'|'start'|'end' }; (2) Chaine container seule (shorthand) ou target prend une valeur par defaut (enter: 'bottom' => target defaut 'start'; leave: 'top' => target defaut 'end'); (3) Chaine container + target (full shorthand, ex. enter: 'bottom top', leave: 'top bottom'). Valeurs par defaut: enter = 'end start', leave = 'start end'. La documentation montre quatre points de bornes: Container Start, Container End, Target Start, Target End. Sous-sections: numeric values, position shorthands, relative position values, min/max thresholds.

**Faits clés**

- Proprietes: enter et leave
- Default enter: 'end start'
- Default leave: 'start end'
- 3 syntaxes: objet {target, container}, container string (shorthand), container+target string (full shorthand)
- Shorthand: enter container seul => target defaut 'start'; leave container seul => target defaut 'end'
- 4 bornes: Container Start, Container End, Target Start, Target End

```js
{
  target: 'top' | 'bottom' | 'start' | 'end',
  container: 'top' | 'bottom' | 'start' | 'end'
}
```

```js
enter: 'bottom'  // target defaults to 'start'
leave: 'top'     // target defaults to 'end'
```

```js
enter: 'bottom top'
leave: 'top bottom'
```

```js
animate('.square', {
  x: 100,
  autoplay: onScroll({
    container: '.container',
    target: '.section',
    axis: 'y',
    enter: 'bottom top',
    leave: 'top bottom',
    sync: true,
    onEnter: () => {},
    onLeave: () => {},
    onUpdate: () => {},
  })
});
```

### events/onscroll/scrollobserver-thresholds/numeric-values

`https://animejs.com/documentation/events/onscroll/scrollobserver-thresholds/numeric-values`

> Les valeurs numeriques definissent un offset depuis le haut de la target et du container; sans unite = pixels, accepte aussi unites relatives et pourcentages.

Les valeurs numeriques definissent un offset depuis le haut de la target et du container. Quand on passe des valeurs numeriques sans specification d'unite, elles sont interpretees comme des pixels. On peut aussi utiliser des unites relatives (comme rem) ou des pourcentages de la hauteur de la target/container. Type: Number | string (avec unite ou pourcentage). Unite par defaut: px. Formats acceptes: Number (ex. 100 = 100 pixels depuis le haut), Unit string (ex. '1rem' = 1rem depuis le haut), Percentage (ex. '10%' = 10% de la hauteur depuis le haut).

**Faits clés**

- Type: Number | string (avec unite ou pourcentage)
- Unite par defaut: px
- Number => pixels depuis le haut (ex. 100 = 100px)
- Unit string => ex. '1rem' = 1rem depuis le haut
- Percentage => ex. '10%' = 10% de la hauteur depuis le haut
- Offset depuis le haut de la target et du container

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container',
    // 80% from the top of the container, -50px from the top of the target 
    enter: '80% 20%',
    // 50px from the top of the container, 100px from the top of the target
    leave: '50 -25',
    debug: true
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```

### events/onscroll/scrollobserver-thresholds/positions-shorthands

`https://animejs.com/documentation/events/onscroll/scrollobserver-thresholds/positions-shorthands`

> Les shorthands de position fournissent des alias semantiques (top, bottom, left, right, center, start, end) qui resolvent en coordonnees selon l'axe.

Les positions shorthands fournissent des alias semantiques pour specifier les points d'alignement lors de la definition des seuils de scroll. Au lieu de valeurs numeriques en pixels, on passe des noms de position intuitifs qui se resolvent en coordonnees sur l'axe vertical ou horizontal selon le contexte. Valeurs acceptees: 'top' (coordonnee y du haut), 'bottom' (coordonnee y du bas), 'left' (coordonnee x gauche), 'right' (coordonnee x droite), 'center' (coordonnee x ou y centrale), 'start' (haut/gauche selon l'axe), 'end' (bas/droite selon l'axe).

**Faits clés**

- 'top' => coordonnee y du haut
- 'bottom' => coordonnee y du bas
- 'left' => coordonnee x gauche
- 'right' => coordonnee x droite
- 'center' => coordonnee x ou y centrale
- 'start' => haut/gauche selon l'axe
- 'end' => bas/droite selon l'axe

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'center top',
    leave: 'center bottom',
    debug: true
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-thresholds/relative-position-values

`https://animejs.com/documentation/events/onscroll/scrollobserver-thresholds/relative-position-values`

> Les valeurs de position relatives utilisent les operateurs prefixes += , -= , *= combines aux ancres de position pour ajuster dynamiquement les seuils.

Les valeurs de position relatives definissent les positions de seuil relativement aux coordonnees de la target et du container en utilisant la syntaxe de valeur relative (operateurs prefixes). Prefixes supportes: += (addition, ex. +=45), -= (soustraction, ex. -=50%), *= (multiplication, ex. *=.5). Cette syntaxe permet l'ajustement dynamique des seuils en combinant les ancres de position avec des operateurs mathematiques.

**Faits clés**

- += => addition (ex. +=45)
- -= => soustraction (ex. -=50%)
- *= => multiplication (ex. *=.5)
- Combine ancres de position et operateurs mathematiques pour ajustement dynamique
- Exemple: enter: 'center+=1em top-=100%', leave: 'center-=1em bottom+=100%'

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  alternate: true,
  loop: true,
  ease: 'inOutQuad',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'center+=1em top-=100%',
    leave: 'center-=1em bottom+=100%',
    debug: true
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large centered row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-thresholds/min-max

`https://animejs.com/documentation/events/onscroll/scrollobserver-thresholds/min-max`

> 'min' / 'max' definissent un seuil base sur l'espace scrollable minimal/maximal disponible, utile quand les positions initiales des elements ne declenchent pas naturellement enter/leave.

'min' | 'max' definit un seuil en utilisant l'espace scrollable minimum ou maximum disponible. Cette approche est utile quand les positions initiales des elements cibles sont trop petites ou trop grandes pour declencher naturellement les conditions enter/leave. 'min' = la plus petite valeur possible satisfaisant la condition enter ou leave; 'max' = la plus grande valeur possible satisfaisant la condition enter ou leave.

**Faits clés**

- Valeurs: 'min' | 'max'
- 'min' => la plus petite valeur possible satisfaisant la condition enter/leave
- 'max' => la plus grande valeur possible satisfaisant la condition enter/leave
- Utile quand les positions initiales sont trop petites/grandes pour declencher naturellement enter/leave
- Exemple: enter: 'max bottom', leave: 'min top'

```js
import { animate, onScroll, utils } from 'animejs';

utils.$('.square').forEach($square => {
  animate($square, {
    x: '15rem',
    rotate: '1turn',
    duration: 2000,
    alternate: true,
    ease: 'inOutQuad',
    autoplay: onScroll({
      container: '.scroll-container',
      sync: 1,
      enter: 'max bottom',
      leave: 'min top',
      debug: true
    })
  });
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
  </div>
</div>
```


## events / onScroll / ScrollObserver synchronisation modes

### events/onscroll/scrollobserver-synchronisation-modes

`https://animejs.com/documentation/events/onscroll/scrollobserver-synchronisation-modes`

> La propriete sync determine comment l'animation est synchronisee par rapport au progres du scroll ou par le franchissement de seuils; quatre categories de modes.

La propriete sync determine comment une animation est synchronisee relativement au progres du scroll ou par le franchissement de certains seuils (thresholds). Introduit pour onScroll() dans Anime.js v4.0.0+. La documentation liste quatre categories de modes de synchronisation: (1) Method names — utiliser des methodes de callback nommees; (2) Playback progress — synchroniser au progres du scroll; (3) Smooth scroll — synchronisation d'animation lisse; (4) Eased scroll — synchronisation d'animation avec easing. Chaque mode a sa propre sous-section dediee avec parametres et exemples specifiques.

**Faits clés**

- Propriete: sync
- Introduit pour onScroll() dans Anime.js v4.0.0+
- 4 categories de modes: Method names, Playback progress, Smooth scroll, Eased scroll
- sync determine la synchronisation au progres du scroll ou par franchissement de seuils

```js
animate('.square', {
  x: 100,
  autoplay: onScroll({
    container: '.container',
    target: '.section',
    axis: 'y',
    enter: 'bottom top',
    leave: 'top bottom',
    sync: true,  // Synchronisation Mode
    onEnter: () => {},
    onLeave: () => {},
    onUpdate: () => {},
  })
});
```

### events/onscroll/scrollobserver-synchronisation-modes/method-names

`https://animejs.com/documentation/events/onscroll/scrollobserver-synchronisation-modes/method-names`

> sync accepte des noms de methodes (string, default 'play pause') separes par espaces, depuis les API Animation/Timer/Timeline, declenches sur les callbacks de scroll.

Ce mode specifie des noms de methodes d'animation a declencher sur les callbacks de scroll. Il accepte des noms de methodes separes par des espaces, issus des API Animation, Timer ou Timeline. Type: string (default: 'play pause'). Options d'ordre de definition: callback unique { sync: 'play' }; seuils enter et leave { sync: 'play pause' }; callbacks directionnels { sync: 'play pause reverse reset' }.

**Faits clés**

- Type: string
- Default: 'play pause'
- Noms de methodes separes par espaces depuis Animation/Timer/Timeline
- 1 callback: { sync: 'play' }
- enter+leave: { sync: 'play pause' }
- directionnels: { sync: 'play pause reverse reset' }
- Exemple utilise sync: 'resume pause reverse reset'

```js
{ sync: 'play' }
```

```js
{ sync: 'play pause' }
```

```js
{ sync: 'play pause reverse reset' }
```

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  duration: 2000,
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: 'resume pause reverse reset',
    debug: true
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section"></div>
  </div>
</div>
```

### events/onscroll/scrollobserver-synchronisation-modes/playback-progress

`https://animejs.com/documentation/events/onscroll/scrollobserver-synchronisation-modes/playback-progress`

> sync: true (ou 1) aligne parfaitement le progres de lecture de l'animation avec la position de scroll dans la region definie.

Ce mode de synchronisation aligne parfaitement le progres de lecture (playback progress) de l'objet anime avec la position de scroll. Quand active, la progression de l'animation reflete directement le mouvement de scroll de l'utilisateur a l'interieur de la region de scroll definie. Signature: sync: true ou sync: 1. Type: boolean | number. Valeurs acceptees: 1 (equivalent numerique) et true (booleen).

**Faits clés**

- Signature: sync: true ou sync: 1
- Type: boolean | number
- Valeurs acceptees: 1 (numerique) et true (booleen)
- Aligne le playback progress de l'animation avec la position de scroll
- Default: non specifie dans la documentation
- L'exemple utilise ease: 'linear'

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: true,
    debug: true,
  })
});
```

```js
<div class="full-container scroll-container scroll-y">
  <div class="scroll-content grid square-grid">
    <div class="scroll-section padded">
      <div class="large row">
        <div class="label">scroll down</div>
      </div>
    </div>
    <div class="scroll-section padded">
      <div class="large row">
        <div class="square"></div>
      </div>
    </div>
    <div class="scroll-section">
    </div>
  </div>
</div>
```


## events / onScroll / ScrollObserver Methods

### events/onscroll/scrollobserver-methods/link

`https://animejs.com/documentation/events/onscroll/scrollobserver-methods/link`

> Methode link() qui connecte une Animation, un Timer ou une Timeline a un ScrollObserver pour synchroniser la lecture avec le scroll.

link() connecte une Animation, un Timer ou une Timeline a une instance de ScrollObserver, etablissant une synchronisation entre les evenements de scroll et la lecture de l'objet lie. Un seul objet peut etre lie a la fois ; chaque nouvel appel remplace le precedent. La methode retourne l'instance ScrollObserver elle-meme (permet le chainage). Cette methode est equivalente a passer une instance onScroll() dans le parametre autoplay d'un objet.

**Faits clés**

- Signature: link(Animation | Timer | Timeline)
- Parametre: Animation | Timer | Timeline - l'objet a synchroniser avec les evenements de scroll
- Retour: l'instance ScrollObserver elle-meme (chainage de methodes)
- Un seul objet peut etre lie a la fois ; chaque appel remplace le lien precedent
- Equivalent a passer une instance onScroll() dans le parametre autoplay d'un objet

```js
link(Animation | Timer | Timeline)
```

```js
import { animate, onScroll } from 'animejs';

const animation = animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
});

const scrollObserver = onScroll({
  container: '.scroll-container',
  enter: 'bottom-=50 top',
  leave: 'top+=60 bottom',
  sync: true,
  debug: true,
});

scrollObserver.link(animation);
```

### events/onscroll/scrollobserver-methods/refresh

`https://animejs.com/documentation/events/onscroll/scrollobserver-methods/refresh`

> Methode refresh() qui met a jour les valeurs de bornes et recalcule les valeurs basees sur des fonctions d'un ScrollObserver.

refresh() met a jour les valeurs de bornes (bounding values) et recalcule les valeurs basees sur des fonctions d'une instance ScrollObserver : 'Updates the bounding values, and re-compute the Function based value of a ScrollObserver instance.' Les parametres pouvant etre rafraichis lorsqu'ils sont definis comme valeurs basees sur des fonctions sont : repeat, axis, enter, leave. Un refresh manuel est inutile lorsque les dimensions du conteneur changent : cela se produit automatiquement en interne. La methode retourne l'instance ScrollObserver elle-meme.

**Faits clés**

- Signature: refresh()
- Retour: l'instance ScrollObserver elle-meme
- Parametres rafraichissables (si valeurs basees sur fonctions): repeat, axis, enter, leave
- Gotcha: refresh manuel inutile au changement de dimensions du conteneur (automatique en interne)

```js
import { animate, onScroll, utils } from 'animejs';

const scrollSettings = {
  enter: 20,
  leave: 60,
}

const animation = animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: () => `bottom-=${scrollSettings.enter} top`,
    leave: () => `top+=${scrollSettings.leave} bottom`,
    sync: .5,
    debug: true,
  })
});

animate(scrollSettings, {
  enter: 90,
  leave: 100,
  loop: true,
  alternate: true,
  modifier: utils.round(0),
  onUpdate: () => animation._autoplay.refresh()
});
```

### events/onscroll/scrollobserver-methods/revert

`https://animejs.com/documentation/events/onscroll/scrollobserver-methods/revert`

> Methode revert() qui desactive le ScrollObserver, retire tous les ecouteurs d'evenements et nettoie l'element HTML de debug.

revert() desactive l'instance ScrollObserver, retire tous les ecouteurs d'evenements (event listeners) et nettoie l'element HTML de debug s'il en a ete cree un. Retourne l'instance ScrollObserver elle-meme, permettant le chainage de methodes si necessaire.

**Faits clés**

- Signature: revert(): ScrollObserver
- Desactive l'instance, retire tous les event listeners, nettoie l'element HTML de debug
- Retour: l'instance ScrollObserver elle-meme (chainage possible)

```js
revert(): ScrollObserver
```

```js
import { animate, onScroll } from 'animejs';

animate('.square', {
  x: '15rem',
  rotate: '1turn',
  ease: 'linear',
  autoplay: onScroll({
    container: '.scroll-container',
    enter: 'bottom-=50 top',
    leave: 'top+=60 bottom',
    sync: 1,
    debug: true,
    onSyncComplete: self => self.revert()
  })
});
```


## events / onScroll / ScrollObserver Properties

### events/onscroll/scrollobserver-properties

`https://animejs.com/documentation/events/onscroll/scrollobserver-properties`

> Liste des proprietes exposees par les instances ScrollObserver retournees par onScroll().

Les instances ScrollObserver (retournees par les appels onScroll()) exposent un ensemble de proprietes accessibles : id (Number, identifiant unique de l'instance), container (ScrollContainer, conteneur de scroll associe), target (HTMLElement, element cible observe), linked (Animation | Timer | Timeline, objet lie pour la synchronisation), repeat (Boolean, si l'observation se repete), horizontal (Boolean, direction de scroll horizontale), enter (String | Number, valeur de seuil d'entree), leave (String | Number, valeur de seuil de sortie, gettable/settable), sync (Boolean, etat de synchronisation activee), velocity (Number, velocite de scroll courante), backward (Boolean, direction de scroll arriere), scroll (Number, position de scroll courante), progress (Number, progression courante de l'element observe de 0 a 1), completed (Boolean, observation terminee), began (Boolean, observation commencee), isInView (Boolean, element observe actuellement visible), offset (Number, decalage de l'element observe), offsetStart (Number, valeur de decalage de debut), offsetEnd (Number, valeur de decalage de fin), distance (Number, distance de scroll pour l'element observe).

**Faits clés**

- id: Number - identifiant unique de l'instance ScrollObserver
- container: ScrollContainer - conteneur de scroll associe
- target: HTMLElement - element cible observe
- linked: Animation | Timer | Timeline - objet lie pour synchronisation
- repeat: Boolean - si l'observation se repete
- horizontal: Boolean - direction de scroll horizontale
- enter: String | Number - valeur de seuil d'entree
- leave: String | Number - valeur de seuil de sortie (gettable/settable)
- sync: Boolean - etat de synchronisation activee
- velocity: Number - velocite de scroll courante
- backward: Boolean - direction de scroll arriere
- scroll: Number - position de scroll courante
- progress: Number - progression courante de l'element observe (0 a 1)
- completed: Boolean - observation terminee
- began: Boolean - observation commencee
- isInView: Boolean - element observe actuellement visible
- offset: Number - decalage de l'element observe
- offsetStart: Number - valeur de decalage de debut
- offsetEnd: Number - valeur de decalage de fin
- distance: Number - distance de scroll pour l'element observe


## text / splitText

### text/splittext

`https://animejs.com/documentation/text/splittext`

> Fonction splitText() : utilitaire leger, responsive et accessible pour decouper, cloner et envelopper lignes, mots et caracteres d'un texte.

splitText(target, parameters) decoupe le texte d'un element. Le parametre target est un selecteur CSS String ou un HTMLElement dont le texte sera decoupe ; parameters (optionnel) est un Object de configuration avec les settings TextSplitter. La fonction retourne un objet TextSplitter contenant les elements de texte decoupes. Decrit comme 'a lightweight, responsive and accessible text utility function to split, clone and wrap lines, words and characters'. Disponible depuis la v4.1.0. Depuis la v4.2.0, peut etre importe independamment : import { splitText } from 'animejs/text';.

**Faits clés**

- Signature: const split = splitText(target, parameters);
- target: CSS selector String | HTMLElement
- parameters (optionnel): Object avec settings TextSplitter
- Retour: objet TextSplitter contenant les elements de texte decoupes
- Disponible depuis v4.1.0
- Depuis v4.2.0: import independant via import { splitText } from 'animejs/text';

```js
const split = splitText(target, parameters);
```

```js
import { createTimeline, stagger, utils, splitText } from 'animejs';

const { words, chars } = splitText('p', {
  words: { wrap: 'clip' },
  chars: true,
});

createTimeline({
  loop: true,
  defaults: { ease: 'inOut(3)', duration: 650 }
})
.add(words, {
  y: [$el => +$el.dataset.line % 2 ? '100%' : '-100%', '0%'],
}, stagger(125))
.add(chars, {
  y: $el => +$el.dataset.line % 2 ? '100%' : '-100%',
}, stagger(10, { from: 'random' }))
.init();
```

### text/splittext/textsplitter-properties

`https://animejs.com/documentation/text/splittext/textsplitter-properties`

> Liste les proprietes en lecture seule exposees par une instance TextSplitter retournee par splitText().

Une instance TextSplitter (retournee par splitText(target, parameters)) expose des proprietes en lecture seule donnant acces aux resultats et a la configuration de l'operation de decoupage de texte. Proprietes documentees : $target (HTMLElement) — l'element racine decoupe ; html (String) — le html a decouper ; debug (Boolean) — si les styles de debug sont visibles ; includeSpaces (Boolean) — si les espaces doivent etre enveloppes dans le texte ; accessible (Boolean) — si l'element clone accessible doit etre cree ; lines (Array<HTMLElement>) — les elements de lignes ; words (Array<HTMLElement>) — les elements de mots ; chars (Array<HTMLElement>) — les elements de caracteres. Egalement lineTemplate, wordTemplate, charTemplate (String) — les structures de template HTML pour ligne/mot/caractere. Ces proprietes fournissent un acces en lecture seule aux resultats du decoupage. Disponible a partir de la version 4.1.0+.

**Faits clés**

- $target: HTMLElement — gets the split root element
- html: String — gets the html to split
- debug: Boolean — gets if the debug styles are visible or not
- includeSpaces: Boolean — gets if the spaces should be wrapped within the text
- accessible: Boolean — gets if the accessible clone element should be created
- lines: Array<HTMLElement> — gets the lines elements
- words: Array<HTMLElement> — gets the words elements
- chars: Array<HTMLElement> — gets the chars elements
- lineTemplate / wordTemplate / charTemplate: String — HTML template structures
- Proprietes en lecture seule ; disponible depuis v4.1.0+

```js
const split = splitText(target, parameters);
split.lines    // Access split lines
split.words    // Access split words
split.chars    // Access split characters
```


## text / splitText / TextSplitter Settings

### text/splittext/textsplitter-settings

`https://animejs.com/documentation/text/splittext/textsplitter-settings`

> Objet de settings TextSplitter configurant comment decouper le texte d'un element (lines, words, chars, debug, includeSpaces, accessible).

Les settings TextSplitter configurent comment decouper le texte des elements HTML cibles. L'objet de settings accepte les proprietes suivantes : lines (Boolean controlant si le texte est decoupe en lignes individuelles), words (Object ou boolean pour le decoupage par mot, supportant les proprietes imbriquees wrap, class, clone), chars (Object ou boolean pour le decoupage par caractere), debug (Boolean pour activer la sortie de debug visuel), includeSpaces (Boolean determinant si les caracteres d'espacement sont inclus dans le decoupage), accessible (Boolean pour les considerations d'accessibilite durant le decoupage). La documentation des settings renvoie vers des pages detaillees individuelles pour : lines, words, chars, debug, includeSpaces, accessible.

**Faits clés**

- lines: Boolean - decoupe le texte en lignes individuelles
- words: Object | boolean - decoupage par mot (proprietes imbriquees wrap, class, clone)
- chars: Object | boolean - decoupage par caractere
- debug: Boolean - active la sortie de debug visuel
- includeSpaces: Boolean - inclut ou non les caracteres d'espacement dans le decoupage
- accessible: Boolean - considerations d'accessibilite durant le decoupage
- Pages detaillees individuelles: lines, words, chars, debug, includeSpaces, accessible

```js
splitText(target, {
  lines: true,
  words: {
    wrap: 'clip',
    class: 'split-word',
    clone: true
  },
  includeSpaces: true,
  debug: true
});
```

### text/splittext/textsplitter-settings/lines

`https://animejs.com/documentation/text/splittext/textsplitter-settings/lines`

> Parametre lines (Boolean | Object | String, defaut false) controlant si et comment le texte est decoupe en lignes individuelles.

Le parametre lines controle si et comment le contenu texte doit etre decoupe en elements de ligne individuels. Type: Boolean | Object | String ; defaut: false. Quand active (lines: true), la librairie enveloppe automatiquement chaque ligne dans un span avec un style display: block et un attribut data-line pour le ciblage. Comme Object: passer des Split parameters pour personnaliser le comportement du wrapper. Comme String: fournir un template HTML personnalise pour envelopper les lignes. Comportements cles : les lignes sont recalculees apres le chargement des polices (via document.fonts.ready) et re-decoupees automatiquement quand l'element cible est redimensionne, ce qui peut interrompre les animations. Pour les elements imbriques (comme <a> ou <em>), ces elements sont dupliques a travers les limites de ligne au besoin pour maintenir la structure. Pour la persistance d'animation, utiliser split.addEffect() pour declarer des animations qui survivent au recalcul ; ces animations sont automatiquement revertees lors de l'appel a split.revert().

**Faits clés**

- Type: Boolean | Object | String ; defaut: false
- lines: true -> chaque ligne enveloppee dans un span display: block avec attribut data-line
- Comme Object: Split parameters pour personnaliser le wrapper
- Comme String: template HTML personnalise
- Lignes recalculees apres document.fonts.ready et re-decoupees au resize (peut interrompre animations)
- Elements imbriques (<a>, <em>) dupliques a travers les limites de ligne
- split.addEffect() declare des animations survivant au recalcul ; revertees par split.revert()

```js
<span style="display: block;" data-line="0">This is the first line</span>
<span style="display: block;" data-line="1">This is the second line</span>
```

```js
import { animate, splitText, stagger } from 'animejs';

splitText('p', {
  lines: { wrap: 'clip' },
})
.addEffect(({ lines }) => animate(lines, {
  y: [
    { to: ['100%', '0%'] },
    { to: '-100%', delay: 750, ease: 'in(3)' }
  ],
  duration: 750,
  ease: 'out(3)',
  delay: stagger(200),
  loop: true,
  loopDelay: 500,
}));
```

### text/splittext/textsplitter-settings/words

`https://animejs.com/documentation/text/splittext/textsplitter-settings/words`

> Parametre words (Boolean | Object | String, defaut true) controlant si et comment le texte est decoupe en mots individuels.

Le parametre words controle si et comment le texte doit etre decoupe en elements de mot individuels. Type: Boolean | Object | String ; defaut: true. Quand active, chaque mot recoit un wrapping avec un style display inline-block et des attributs data pour le suivi. L'implementation s'appuie sur l'API native Intl.Segmenter quand disponible, permettant une segmentation correcte des mots pour les langues sans delimiteur d'espace (japonais, chinois, thai, lao, khmer, birman). Les navigateurs plus anciens reviennent a String.prototype.split(). Options de configuration : Boolean (activer/desactiver avec wrapping par defaut), Object (passer des Split parameters pour personnalisation), String (fournir un template HTML personnalise). Note importante : quand on combine decoupage par mot et par ligne, le decoupage de ligne recree les elements de mot, interrompant les animations de mot ; utiliser split.addEffect() pour garantir la lecture continue a travers les resizes. split.revert() reverte a la fois le DOM et l'animation.

**Faits clés**

- Type: Boolean | Object | String ; defaut: true
- Chaque mot enveloppe avec display: inline-block et attributs data (data-line, data-word)
- Utilise Intl.Segmenter quand disponible (langues sans delimiteur: japonais, chinois, thai, lao, khmer, birman)
- Fallback navigateurs anciens: String.prototype.split()
- Options: Boolean / Object (Split parameters) / String (template HTML)
- Gotcha: decoupage par ligne recree les elements de mot et interrompt les animations de mot -> utiliser split.addEffect()
- split.revert() reverte le DOM et l'animation

```js
<span style="display: inline-block;" data-line="0" data-word="0">Split</span>
```

```js
import { animate, splitText, stagger } from 'animejs';

const { words } = splitText('p', {
  words: { wrap: 'clip' },
})

animate(words, {
  y: [
    { to: ['100%', '0%'] },
    { to: '-100%', delay: 750, ease: 'in(3)' }
  ],
  duration: 750,
  ease: 'out(3)',
  delay: stagger(100),
  loop: true,
});
```

```js
const split = splitText(target, params);

split.addEffect(({ lines, words, chars }) => animate([lines, words, chars], {
  opacity: { from: 0 },
}));

split.revert(); // Reverts both the DOM and the animation
```


## text / scrambleText

### text/scrambletext

`https://animejs.com/documentation/text/scrambletext`

> Fonction d'animation de texte qui revele le contenu via un effet de brouillage (scramble) caractere par caractere.

scrambleText() est une fonction d'animation de texte qui revele le contenu via un effet de brouillage aleatoire caractere par caractere, avec une transition fluide. Disponible depuis la version 4.4.0. Elle s'utilise comme valeur de propriete dans animate(). Import : import { scrambleText } from 'animejs'; ou import { scrambleText } from 'animejs/text';. Application : animate(target, { innerHTML: scrambleText(parameters) });. Important : appliquer scrambleText() a la propriete innerHTML plutot qu'a textContent. Signature : nom scrambleText, parametre parameters (optionnel) — un objet contenant les options de configuration ; retourne une valeur de tween basee sur une fonction, compatible avec animate().

**Faits clés**

- Disponible depuis la version 4.4.0
- S'applique a innerHTML, PAS a textContent
- Import depuis 'animejs' ou 'animejs/text'
- Parametre parameters optionnel (objet de configuration)
- Retourne une valeur de tween basee sur fonction, compatible animate()

```js
import { scrambleText } from 'animejs';
// or
import { scrambleText } from 'animejs/text';
```

```js
animate(target, { innerHTML: scrambleText(parameters) });
```

```js
import { animate, scrambleText } from 'animejs';

animate('p', {
  innerHTML: scrambleText(),
  loop: true,
  loopDelay: 1000,
});
```

```js
<div class="large row">
  <p class="text-s text-mono">scrambleText() allows you to reveal a text via a smooth randomized character scramble transition effect.</p>
</div>
```

### text/scrambletext/scrambletext-parameters

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters`

> Page index listant les parametres de configuration de scrambleText() utilisables dans animate({ innerHTML: scrambleText({...}) }).

La fonction scrambleText configure le comportement de l'animation de brouillage de texte, pour utilisation dans la propriete innerHTML d'un appel animate(). Les parametres listes (chacun avec sa propre page de detail) sont : text (texte cible a animer), chars (jeu de caracteres utilise pour le brouillage), override (comportement d'override avant l'animation), ease (fonction d'easing), cursor (caractere de curseur affiche), revealRate (vitesse de revelation des caracteres), revealDelay (delai avant la revelation), settleRate (vitesse de la phase de stabilisation finale), settleDuration (duree de l'animation de stabilisation), delay (delai initial de l'animation), duration (duree totale), perturbation (niveau d'aleatoire/chaos), from (reference de position de depart), reversed (sens inverse), seed (graine aleatoire pour reproductibilite). Les pages individuelles fournissent les explications detaillees.

**Faits clés**

- Parametres listes : text, chars, override, ease, cursor, revealRate, revealDelay, settleRate, settleDuration, delay, duration, perturbation, from, reversed, seed
- S'utilise dans la propriete innerHTML d'animate()

```js
animate(target, {
  innerHTML: scrambleText({
    text: 'Hello World',
    chars: 'uppercase',
    from: 'center',
    cursor: '_',
    settleDuration: 500,
    revealRate: 60,
    settleRate: 30,
  }),
});
```


## text / scrambleText / parameters

### text/scrambletext/scrambletext-parameters/text

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/text`

> Specifie le texte de destination vers lequel l'animation de brouillage converge ; par defaut le texte existant de l'element.

Le parametre text specifie le texte de destination vers lequel l'animation tend. Type : String | Function(target, index, targets) -> String. Valeur par defaut : le contenu textuel original de l'element cible. Quand il est omis, la fonction utilise le contenu textuel existant de l'element comme cible. Types acceptes : String (valeur de texte directe) ou Function (callback recevant (target, index, targets) et retournant une chaine).

**Faits clés**

- Type : String | Function(target, index, targets) -> String
- Defaut : contenu textuel original de l'element cible
- Si omis, utilise le textContent existant de l'element comme cible

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const [ $button ] = utils.$('button');

const texts = ['Transition between different text.', 'Hello World!', 'Anime.js 4.4 scrambleText()'];
let i = 0;

function play() {
  i = (i + 1) % texts.length;
  animate($p, {
    innerHTML: scrambleText({ text: texts[i] }),
  });
}

play();

$button.addEventListener('click', play);
```

```js
<div class="large row centered">
  <p class="text-l text-mono">Transition between different text.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>Change text</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/chars

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/chars`

> Definit le jeu de caracteres affiches pendant le brouillage ; supporte chaines litterales, syntaxe de plage par tiret, et presets nommes.

Le parametre chars specifie quels caracteres s'affichent pendant l'animation de brouillage. Type : String | Function(target, index, targets). Valeur par defaut : 'a-zA-Z0-9!%#_'. Supporte des chaines de caracteres litterales, une syntaxe de plage Unicode utilisant des tirets (ex. 'a-d' s'etend en 'abcd'), ou des combinaisons. Des presets nommes fournissent des raccourcis : 'lowercase' (a-z), 'uppercase' (A-Z), 'numbers' (0-9), 'symbols' (!%#_|*+=), 'braille' (⠀-⣿), 'blocks' (▀-▟), 'shades' (░-▓). Gotcha : pour inclure un tiret litteral, le placer en debut ou fin de chaine (ex. '-a-z' ou 'a-z-') ; sinon il est interprete comme operateur de plage.

**Faits clés**

- Type : String | Function(target, index, targets)
- Defaut : 'a-zA-Z0-9!%#_'
- Presets : lowercase (a-z), uppercase (A-Z), numbers (0-9), symbols (!%#_|*+=), braille (⠀-⣿), blocks (▀-▟), shades (░-▓)
- Syntaxe de plage par tiret (ex. 'a-d' -> 'abcd')
- Tiret litteral : le placer en debut ou fin de chaine sinon interprete comme plage

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const charSets = ['braille', 'blocks', 'numbers'];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ chars: charSets[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

### text/scrambletext/scrambletext-parameters/override

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/override`

> Determine comment le texte apparait avant le debut de l'animation de brouillage.

Le parametre override determine comment le texte apparait avant le debut de l'animation de brouillage. Signature : override: true | false | '' | ' ' | String. Valeur par defaut : true. Quand true, il brouille le texte original en utilisant le jeu de caracteres defini par chars. false affiche le texte original inchange. '' demarre depuis du vide (blank). ' ' (espace) remplace tous les caracteres par des espaces. Une chaine personnalisee utilise ce jeu de caracteres custom. Valeurs : true (brouille avec le jeu chars), false (montre le texte original tel quel), '' (demarre vide), ' ' (remplace par des espaces), String (jeu de caracteres custom).

**Faits clés**

- Signature : true | false | '' | ' ' | String
- Defaut : true
- true = brouille avec le jeu chars ; false = texte original inchange
- '' = demarre vide ; ' ' = remplace par espaces ; String = jeu custom

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const overrides = [false, 'uppercase', '_'];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ override: overrides[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

### text/scrambletext/scrambletext-parameters/ease

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/ease`

> Easing applique a l'animation de brouillage, controlant l'acceleration/deceleration de la vague de revelation.

Le parametre ease definit l'easing applique a l'animation de brouillage. Type : valeur d'easing. Valeur par defaut : 'linear'. Il controle comment la vague de revelation accelere et decelere a travers le texte pendant l'effet de brouillage. Valeurs acceptees : toute fonction d'easing valide du systeme d'easing d'Anime.js, y compris les courbes integrees comme 'linear', 'inOut(3)', 'outExpo', et des fonctions d'easing custom.

**Faits clés**

- Type : valeur d'easing
- Defaut : 'linear'
- Controle l'acceleration/deceleration de la vague de revelation
- Accepte toute fonction d'easing Anime.js (ex. 'linear', 'inOut(3)', 'outExpo', custom)

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');
const easeCurves = ['linear', 'inOut(3)', 'outExpo'];

function play(ease) {
  animate($p, { innerHTML: scrambleText({ ease, override: false }) });
}

play(easeCurves[0]);

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(easeCurves[i])));
```

```js
<div class="large row">
  <p class="text-s text-mono">Apply easing functions to control the acceleration and deceleration of the scramble reveal wave.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>linear</button>
    <button>inOut</button>
    <button>outExpo</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/cursor

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/cursor`

> Controle les caracteres affiches au bord avant de la vague de revelation a mesure que l'animation progresse.

Le parametre cursor controle les caracteres affiches au bord avant (leading edge) de la vague de revelation a mesure que l'animation scrambleText progresse dans le texte. Signature : cursor: true | Number | String. Valeur par defaut : '' (pas de curseur). Valeurs acceptees : true utilise '_' comme caractere de curseur ; Number est un code de caractere (ex. 124 pour '|') ; String utilise la chaine fournie directement comme caracteres de curseur.

**Faits clés**

- Signature : true | Number | String
- Defaut : '' (pas de curseur)
- true = utilise '_' ; Number = code de caractere (ex. 124 -> '|') ; String = chaine utilisee directement
- Affiche au bord avant de la vague de revelation

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const cursors = ['_____', '░▒▓█', '😀'];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ cursor: cursors[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Display a cursor character at the leading edge of the reveal wave as it moves through each character.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>_____</button>
    <button>░▒▓█</button>
    <button>😀</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/revealrate

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/revealrate`

> Controle la vitesse de revelation des caracteres ; les valeurs plus elevees accelerent la vague de revelation.

Le parametre revealRate controle la vitesse a laquelle les caracteres sont reveles pendant l'animation de brouillage. Signature : revealRate: Number = 60. Des valeurs numeriques plus elevees accelerent le deplacement de la vague de revelation. Ce parametre fonctionne conjointement avec settleDuration pour determiner la duree totale de l'animation lorsqu'aucune duration explicite n'est specifiee.

**Faits clés**

- Type : Number
- Defaut : 60
- Valeurs plus elevees = vague de revelation plus rapide
- Travaille avec settleDuration pour la duree totale si aucune duration explicite

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [20, 60, 120];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ revealRate: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Higher values make the reveal wave move faster.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>20</button>
    <button>60</button>
    <button>120</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/revealdelay

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/revealdelay`

> Delai en ms avant le demarrage de la vague de revelation dans l'animation de brouillage.

Le parametre revealDelay controle le delai en millisecondes avant que la vague de revelation ne commence dans l'animation de brouillage. Type : Number | Function(target, index, targets). Valeur par defaut : 0. Il determine combien de temps l'animation attend avant de commencer a devoiler le texte cible pendant l'effet de scramble. Le parametre accepte soit une valeur statique en millisecondes, soit une fonction retournant des millisecondes dynamiquement selon l'index de la cible.

**Faits clés**

- Type : Number | Function(target, index, targets)
- Defaut : 0
- Delai en ms avant le demarrage de la vague de revelation
- Accepte valeur statique ou fonction dynamique selon l'index de la cible

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [0, 500, 2000];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ revealDelay: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Add a delay in milliseconds before the reveal wave starts within the scramble animation.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>0</button>
    <button>500</button>
    <button>2000</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/settlerate

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/settlerate`

> Nombre de fois par seconde que chaque caractere cycle a travers des glyphes aleatoires ; valeurs elevees = scintillement plus rapide.

Le parametre settleRate definit combien de fois par seconde chaque caractere cycle a travers des glyphes aleatoires. Signature : settleRate: Number = 30. Des valeurs plus elevees creent un scintillement (flickering) plus rapide. Il controle la frequence de scintillement des caracteres pendant la phase de brouillage. Disponible depuis la version 4.4.0.

**Faits clés**

- Type : Number
- Defaut : 30
- Nombre de cycles de glyphes aleatoires par seconde par caractere
- Valeurs plus elevees = scintillement plus rapide
- Disponible depuis v4.4.0

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [5, 30, 60];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ settleRate: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Control how many times per second characters cycle through random values during the scramble phase.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>5</button>
    <button>30</button>
    <button>60</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/settleduration

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/settleduration`

> Duree en ms pendant laquelle chaque caractere se brouille avant de se stabiliser sur son glyphe final.

Le parametre settleDuration controle la duree en millisecondes pendant laquelle chaque caractere passe a se brouiller avant de se stabiliser sur son glyphe final, lors de l'utilisation de scrambleText(). Type : Number. Valeur par defaut : 300 (millisecondes). Parametres lies : settleRate (controle la vitesse de brouillage durant la stabilisation), revealRate et revealDelay (controlent la phase de revelation initiale).

**Faits clés**

- Type : Number
- Defaut : 300 (ms)
- Duree de brouillage de chaque caractere avant stabilisation sur son glyphe final
- Lie a settleRate, revealRate, revealDelay

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [100, 300, 1000];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ settleDuration: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Define how long each character spends scrambling before settling into its final settled value.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>100</button>
    <button>300</button>
    <button>1000</button>
  </fieldset>
</div>
```

### text/scrambletext/scrambletext-parameters/delay

`https://animejs.com/documentation/text/scrambletext/scrambletext-parameters/delay`

> Temps en ms avant le debut de l'animation de brouillage de texte.

Le parametre delay specifie le temps en millisecondes avant le debut de l'animation de brouillage de texte. Signature : delay: Number | Function(target, index, targets) -> Number. Valeur par defaut : 0. Il peut etre une valeur statique ou une fonction qui calcule dynamiquement le delai en fonction de l'element cible, de son index, et de l'ensemble des cibles.

**Faits clés**

- Type : Number | Function(target, index, targets) -> Number
- Defaut : 0
- Temps en ms avant le debut de l'animation de brouillage
- Accepte valeur statique ou fonction dynamique

```js
import { animate, scrambleText } from 'animejs';

const [ $p ] = utils.$('p');
const buttons = utils.$('button');

const values = [0, 500, 1500];

function play(i) {
  animate($p, {
    innerHTML: scrambleText({ delay: values[i] }),
  });
}

buttons.forEach(($btn, i) => $btn.addEventListener('click', () => play(i)));
```

```js
<div class="large row">
  <p class="text-s text-mono">Add a delay in milliseconds before the scramble animation starts.</p>
</div>
<div class="medium row">
  <fieldset class="controls">
    <button>0</button>
    <button>500</button>
    <button>1500</button>
  </fieldset>
</div>
```


## adapters/threejs-adapter

### adapters/threejs-adapter/materials-and-uniforms

`https://animejs.com/documentation/adapters/threejs-adapter/materials-and-uniforms`

> Decrit comment animer directement les materiaux Three.js, les uniforms de shaders et les slots TSL via animate() et utils.set(), en ciblant le materiau ou le Mesh parent.

Anime.js permet d'animer directement les materiaux Three.js, les uniforms de shaders et les slots TSL via animate() et utils.set(), en ciblant le materiau lui-meme ou son Mesh parent.

Champs de materiau animables: les types scalaires/booleens (metalness, roughness, opacity, wireframe) s'animent par leur nom direct; les couleurs (color, emissive, specular, sheenColor) s'animent par leur nom direct; les axes de vecteur (normalScale, clearcoatNormalScale) s'animent par nom per-axe (normalScaleX / normalScaleY).

Uniforms de ShaderMaterial: les uniforms personnalises s'animent par leur nom d'uniform. Les uniforms scalaires et Color utilisent le nom direct; les uniforms Vector2/3/4 exposent des noms per-axe.

Slots TSL NodeMaterial: les slots assignes a un UniformNode issu de la factory uniform() s'animent par le nom du slot sur le materiau.

UniformNode nu: un UniformNode peut etre anime directement. On utilise value pour les scalaires, color pour les couleurs, et x / y / z / w pour les vecteurs.

Mapping via le Mesh parent: les champs de materiau, les uniforms de ShaderMaterial et les slots TSL sont atteignables a travers le Mesh parent (animate(mesh, ...) equivaut a animate(mesh.material, ...)).

**Faits clés**

- Champs scalaires/booleens (metalness, roughness, opacity, wireframe): nom direct
- Champs couleur (color, emissive, specular, sheenColor): nom direct
- Axes de vecteur (normalScale, clearcoatNormalScale): noms per-axe normalScaleX / normalScaleY
- Uniforms ShaderMaterial: scalaires et Color par nom direct; Vector2/3/4 exposent des noms per-axe
- Slots TSL NodeMaterial assignes via uniform() s'animent par le nom du slot sur le materiau
- UniformNode nu: value pour scalaires, color pour couleurs, x/y/z/w pour vecteurs
- animate(mesh, {...}) equivaut a animate(mesh.material, {...})
- Gotcha: les ecritures vont directement sur mesh.material; un materiau partage entre meshes signifie qu'une animation les met tous a jour
- Gotcha: cloner le materiau par mesh pour scoper les animations individuellement
- Gotcha: definir material.transparent = true pour les materiaux dont l'opacity est animee
- Gotcha: les uniforms avec valeurs Texture, Matrix3, Matrix4, UniformArrayNode ou BufferNode ne sont pas geres

```js
const shader = new ShaderMaterial({
  uniforms: {
    tint:       { value: new Color('#f00') },
    intensity:  { value: 0.5 },
    resolution: { value: new Vector2(1024, 768) },
  },
});

animate(shader, { tint: '#0ff', intensity: 1, resolutionX: 2048 });
```

```js
import { uniform } from 'three/tsl';

material.colorNode  = uniform(new Color('#f00'));
material.offsetNode = uniform(new Vector3());

animate(material, { colorNode: '#0f0', offsetNodeY: 0.5 });
```

```js
animate(uniform(0),                { value: 1 });
animate(uniform(new Color()),      { color: '#0f0' });
animate(uniform(new Vector3()),    { x: 1, y: 0.5 });
```

```js
animate(mesh, { metalness: 1, emissive: '#0ff', uTime: 1 });
// equivalent to: animate(mesh.material, { metalness: 1, emissive: '#0ff', uTime: 1 });
```

### adapters/threejs-adapter/threejs-instanced-and-batched-meshes

`https://animejs.com/documentation/adapters/threejs-adapter/threejs-instanced-and-batched-meshes`

> Documente getInstances(mesh) et commitChanges(mesh) pour animer individuellement les instances d'un InstancedMesh ou BatchedMesh, ainsi que les proprietes per-instance supportees.

getInstances(mesh) retourne un tableau de proxies par slot (un proxy par slot) qui acceptent les memes noms de proprietes qu'un mesh regulier, permettant l'animation individuelle des instances au sein d'un seul mesh. Parametre mesh de type InstancedMesh | BatchedMesh. Les slots BatchedMesh supprimes sont null. La reference du tableau persiste a travers les changements de mesh.count, les appels addInstance() et deleteInstance().

commitChanges(mesh) vide (flush) les ecritures de matrices en attente vers le mesh. L'adaptateur l'invoque automatiquement avant le rendu, mais il faut l'appeler manuellement si on lit mesh.instanceMatrix entre les ticks d'animation et le cycle de rendu suivant. Parametre mesh: InstancedMesh | BatchedMesh precedemment passe a getInstances().

Proprietes per-instance supportees par chaque proxy: x/y/z (position), rotateX/Y/Z (rotation en degres), scaleX/Y/Z/scale (echelle), skewX/Y/Z (skew en degres), transformOriginX/Y/Z (decalage d'origine), transformOrigin ('x y z' shorthand), color (mesh.setColorAt(id, color)), visible (mesh.setVisibleAt(id, value), BatchedMesh seulement).

**Faits clés**

- getInstances(mesh) — parametre mesh: InstancedMesh | BatchedMesh; retourne un tableau de proxies par slot
- Slots BatchedMesh supprimes = null
- La reference du tableau retourne par getInstances persiste a travers mesh.count, addInstance() et deleteInstance()
- commitChanges(mesh) — flush les ecritures de matrices en attente; invoque automatiquement avant le rendu
- Appeler commitChanges manuellement si on lit mesh.instanceMatrix entre ticks d'animation et le rendu suivant
- Proprietes per-instance: x/y/z, rotateX/Y/Z (degres), scaleX/Y/Z/scale, skewX/Y/Z (degres), transformOriginX/Y/Z, transformOrigin ('x y z' shorthand), color -> setColorAt, visible -> setVisibleAt (BatchedMesh seulement)
- Note: opacity ecrit le materiau partage et affecte toutes les instances; pour un fade per-instance, construire un canal alpha dans le shader et animer color
- Note: visible est non-fonctionnel sur InstancedMesh; utiliser scale = 0 a la place
- Import: getInstances depuis 'animejs/adapters/three'

```js
getInstances(mesh)
```

```js
import { animate, stagger } from 'animejs';
import { getInstances } from 'animejs/adapters/three';

const instances = getInstances(mesh);

animate(instances, {
  x: 100,
  scale: 2,
  delay: stagger(20),
});
```

```js
commitChanges(mesh)
```

### adapters/threejs-adapter/threejs-adapter-common-gotchas

`https://animejs.com/documentation/adapters/threejs-adapter/threejs-adapter-common-gotchas`

> Liste les cas limites a considerer lors de l'animation d'objets Three.js: ordre de rotation, flag transparent, flip de visibilite, materiaux partages, fades de groupes, instance unique, types d'uniforms hors-scope et caveats des proxies d'instances.

Ordre de rotation: l'adaptateur ecrit rotateX, rotateY, rotateZ directement sur les angles d'Euler. Three.js applique les rotations dans son ordre par defaut 'XYZ'. Si target.rotation.order a ete change, les resultats ne correspondront pas aux noms de proprietes; utiliser un Quaternion en dehors de l'adaptateur pour d'autres ordres de rotation.

Flag transparent pour les fades d'opacity: les materiaux doivent avoir leur flag transparent a true avant d'animer opacity. Sans cela, Three.js rend le materiau totalement opaque quelle que soit la valeur d'opacity.

Flip de visibilite et mutation directe: definir opacity = 0, scale = 0, ou n'importe quel axe de scale a 0 bascule automatiquement target.visible a false, mais uniquement via animate() ou utils.set(). Les ecritures directes comme mesh.scale.x = 0 ne declenchent pas ce flip de visibilite.

Materiaux partages et shorthand mesh: les proprietes de materiau resident sur le materiau, pas sur le mesh. Les animer met a jour tous les meshes partageant cette instance de materiau. Cloner les materiaux par mesh pour scoper les animations. Le shorthand mesh ne route vers les materiaux que lorsque le mesh n'a pas son propre champ de ce nom.

Fades de groupes: les Groups ne possedent pas de materiaux, donc animer opacity ou color sur un group n'affecte pas les enfants. Passer un tableau de meshes descendants comme targets pour fader des sous-arbres entiers.

Instance Three.js unique: Three.js doit se resoudre a une seule instance de bundler. Des copies dupliquees dans des dependances imbriquees font echouer les verifications instanceof.

Types d'uniforms hors-scope: les uniforms avec valeurs Texture, Matrix3, Matrix4, UniformArrayNode ou BufferNode ne sont pas supportes; animer directement les champs numeriques sous-jacents.

Caveats des proxies d'instances: sur les proxies per-instance, opacity affecte le materiau partage globalement, et visible est un no-op sur InstancedMesh. Apres l'execution de getInstances(), mesh.onBeforeRender ne retournera pas exactement la fonction assignee.

**Faits clés**

- Rotation: rotateX/Y/Z ecrits sur les angles d'Euler; Three.js applique l'ordre 'XYZ' par defaut; si rotation.order change, utiliser un Quaternion hors adaptateur
- opacity: definir material.transparent = true avant d'animer opacity, sinon rendu totalement opaque
- opacity = 0 / scale = 0 / un axe de scale a 0 bascule target.visible a false, mais seulement via animate() ou utils.set() (pas via mutation directe comme mesh.scale.x = 0)
- Proprietes de materiau sur le materiau (pas le mesh); les animer met a jour tous les meshes partageant le materiau; cloner par mesh pour scoper
- Le shorthand mesh ne route vers les materiaux que si le mesh n'a pas son propre champ de ce nom
- Groups ne possedent pas de materiaux: animer opacity/color sur un group n'affecte pas les enfants; passer un tableau de meshes descendants
- Three.js doit se resoudre a une seule instance de bundler; copies dupliquees font echouer instanceof
- Uniforms Texture, Matrix3, Matrix4, UniformArrayNode, BufferNode non supportes; animer les champs numeriques sous-jacents
- Proxies per-instance: opacity affecte le materiau partage globalement; visible est un no-op sur InstancedMesh
- Apres getInstances(), mesh.onBeforeRender ne retourne pas exactement la fonction assignee

