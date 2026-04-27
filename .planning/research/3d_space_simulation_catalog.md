# Catalogue simulations 3D spatiales réalistes — Rendu scientifique pour SBFB

**Date** : 2026-04-27
**Objectif** : Trouver des libs/projets pour un rendu UNIQUE — pas des particules
décoratives mais des simulations physiquement correctes d'artefacts cosmiques.
**Sources** : Context7 (Three.js, R3F, tsparticles) + recherche web/GitHub

---

## 1. Trous noirs — gravitational lensing

### Tier S — Référence scientifique

**[ebruneton/black_hole_shader](https://github.com/ebruneton/black_hole_shader)** — 189 stars, BSD-3
- Trou noir Schwarzschild (non-rotatif) avec disque d'accrétions et étoiles de fond
- Maths : Beam tracing avec tables précalculées, intersections rayon courbé en **temps constant par pixel**
- Effets : Doppler relativiste, beaming, anneau d'Einstein, sphère de photons
- Paper scientifique : ebruneton.github.io/black_hole_shader/paper.pdf
- Tech : WebGL 2.0, GLSL. Shaders directement portables en R3F via `shaderMaterial`

**[oseiskar/black-hole](https://github.com/oseiskar/black-hole)** — 268 stars
- Intégration ODE des géodésiques de Schwarzschild **directement en GLSL GPU**
- Three.js natif. 30+ FPS à 1920x1080 sur GPU mid-range
- Unités normalisées (rayon de Schwarzschild = 1)

### Tier A — Kerr (trous noirs ROTATIFS, physique complète)

**[steeltroops-ai/blackhole-simulation](https://github.com/steeltroops-ai/blackhole-simulation)**
- Le plus ambitieux techniquement. Ray-marching relativiste pour **Kerr à spin quasi-extremal (a=0.999)**
- Maths : Métrique de Kerr-Schild régularisée, intégrateur symplectique de Yoshida d'ordre 6, conservation du Hamiltonien, ISCO calculé explicitement, photon ring
- Performance : **2M+ géodésiques par frame à 120Hz**, TAA avec reprojection, variance clipping YCoCg
- Tech : Next.js 14 + WebGPU/WebGL 2.0 + **Rust/WASM** (maths physiques)

**[SushantGagneja/Black-Hole-simulation](https://github.com/SushantGagneja/Black-Hole-simulation)**
- Kerr en Three.js + GLSL. Frame dragging (Lense-Thirring), accrétions volumétriques procédurales
- Demo live : black-hole-simulation-psi.vercel.app

**[chrismatgit/black-hole-simulation](https://github.com/chrismatgit/black-hole-simulation)** — MIT
- **React + TypeScript + Three.js + Vite** — le plus directement portable en R3F
- Schwarzschild, Runge-Kutta 4, 50-2000 rayons configurables, redshift gravitationnel

### Tier B — Artistique / spectaculaire

**[MisterPrada/singularity](https://github.com/MisterPrada/singularity)** — 274 stars
- TSL (Three.js Shading Language) + WebGPU/WebGL dual
- Ray marching + bloom. Visuellement spectaculaire
- Demo : singularity.misterprada.com

### Shadertoy références

- [Black hole with accretion disk](https://www.shadertoy.com/view/tsBXW3) — le plus populaire
- [Interstellar-style](https://www.shadertoy.com/view/MctGWj)
- [Gravitational Lensing (Zippy)](https://www.shadertoy.com/view/Wcc3R2)

---

## 2. Nébulae volumétriques — ray marching

**[Erichlof/THREE.js-PathTracing-Renderer](https://github.com/erichlof/THREE.js-PathTracing-Renderer)** — 2200 stars, CC0 (domaine public)
- Path tracer temps réel sur Three.js
- Volumes : nuages, brouillard, caustiques, shafts de lumière, planètes avec atmosphère physique
- Maths : Monte Carlo integration, Beer-Lambert atténuation, Rayleigh scattering, FBM terrain
- 30-60 FPS navigateur, y compris mobile
- Shaders directement utilisables en R3F

**Maxime Heckel — Volumetric Raymarching** (blog + code)
- blog.maximeheckel.com/posts/real-time-cloudscapes-with-volumetric-raymarching/
- Tutorial complet pour rendu volumétrique **dans React Three Fiber directement**
- Applicable aux nébulae en changeant les paramètres de densité/émission

**Shadertoy références nébulae**
- [Star Nest (Kali)](https://www.shadertoy.com/view/XlfGRj) — fractal 3D kaliset avec volumétrique, **le shader spatial le plus iconique de Shadertoy**
- Shaders de Duke : Dusty Nebula, Supernova Remnant, Cat's Eye, Crab Nebula — CC-BY-NC-SA 3.0
- Article technique : tonisagrista.com/blog/2024/rendering-aurorae-nebulae/

---

## 3. Simulations N-body gravitationnelles — formation de galaxies

**[dgreenheck/webgpu-galaxy](https://github.com/dgreenheck/webgpu-galaxy)** — 61 stars, MIT
- Galaxie spirale interactive, **TSL + WebGPU + Three.js**
- Jusqu'à **750,000 particules** en compute shaders GPU
- Génération procédurale des bras spiraux, nuages de poussière, bloom HDR
- **Directement portable en R3F** (Three.js + TSL natif)

**[DrA1ex/JS_ParticleSystem](https://github.com/DrA1ex/JS_ParticleSystem)**
- "Galaxy Birth" en temps réel. N-Body + 1-Body, GPGPU dans un worker
- Maths : Arbre spatial hiérarchique (Barnes-Hut) O(N log N)
- **100,000 particules** avec ~500k opérations au lieu de 10 milliards

**[andrewdcampbell/galaxy-sim](https://github.com/andrewdcampbell/galaxy-sim)**
- Formation de galaxies N-body temps réel WebGL + Three.js
- Gaz coalescant en étoiles autour d'un trou noir central
- Origine : CS184 Berkeley (qualité académique)

**[Harmony-of-the-Spheres](https://github.com/TheHappyKoala/Harmony-of-the-Spheres)** — 134 stars
- Simulateur N-body Newtonien. **React + Redux + Three.js**
- De la chorégraphie 3-corps à New Horizons
- Demo : gravitysimulator.org

---

## 4. Wormholes et distorsions spatiales

**[sirxemic/wormhole](https://github.com/sirxemic/wormhole)** — 74 stars, MIT
- Simulation wormhole Ellis avec ray tracing temps réel
- Maths : Métrique d'Ellis, projection par symétrie sphérique
- Tech : TypeScript + WebGL + Three.js + WebVR

**[TracingGeodesics (Mykhailo Moroz)](https://michaelmoroz.github.io/TracingGeodesics/)**
- L'approche la plus complète. Visualisation de **MULTIPLES métriques** :
  Schwarzschild, Kerr, Kerr-Newman, **Alcubierre (warp drive)**, wormholes
- Innovation majeure : description Hamiltonienne des géodésiques,
  **4 gradients au lieu de 64 dérivées du tenseur métrique en 4D**
- ~100 lignes de code core pour **n'importe quelle métrique**
- Portage Unity/HLSL existe, portage GLSL faisable

**[portsmouth/gravy](https://github.com/portsmouth/gravy)** — MIT
- Lensing gravitationnel générique en WebGL
- Potentiel gravitationnel 3D défini en GLSL, lumière déviée par le gradient local

---

## 5. Étoiles réalistes — corps noir, atmosphère

**[wwwtyro/glsl-atmosphere](https://github.com/wwwtyro/glsl-atmosphere)** — 630 stars, Unlicense (domaine public)
- Module GLSL pour atmosphères réalistes (Rayleigh + Mie scattering)
- npm installable via glslify
- Ray marching atmosphérique, coefficients physiques

**[ebruneton/precomputed_atmospheric_scattering](https://github.com/ebruneton/precomputed_atmospheric_scattering)** — BSD-3
- Implémentation de référence du paper EGSR 2008
- Précomputation de textures atmosphériques, couche d'ozone, profils de densité custom
- Demo WebGL2

**[THRASTRO/thrastro-shaders](https://github.com/THRASTRO/thrastro-shaders)**
- Shaders THREE.js spécialisés astronomie : FirmamentShader, SkyShader, PlanetShader, OrbitalShader
- Maths : Rayleigh + Mie (GPU Gems 2 Ch.16), éclipses fragment-level, coordonnées 64-bit, Kepler

**[bpodgursky/uncharted](https://github.com/bpodgursky/uncharted)**
- Voisinage solaire (75 années-lumière) en Three.js
- Température → couleur via spectre de **corps noir (Planck)**, texture 1D 800K-30000K
- Catalogue HYG réel (Hipparcos + Yale + Gliese)

---

## 6. Fluid dynamics / SPH — gaz interstellaire

**[PavelDoGreat/WebGL-Fluid-Simulation](https://github.com/PavelDoGreat/WebGL-Fluid-Simulation)** — 16k+ stars, MIT
- LA référence en simulation fluide WebGL. Navier-Stokes GPU
- Advection, divergence, curl, vorticity confinement, pressure solving
- Port React : github.com/x8BitRain/react-webgl-fluid

**[jeantimex/fluid](https://github.com/jeantimex/fluid)** — WebGPU
- SPH + PIC/FLIP en WebGPU compute shaders
- Dizaines de milliers de particules à 60 FPS

**[LinzhouLi/WebGPU-Fluid-Simulation](https://github.com/LinzhouLi/WebGPU-Fluid-Simulation)**
- Position Based Fluid (PBF) en WebGPU, tension de surface, vorticity

---

## 7. Ray marching frameworks

**[danielesteban/three-raymarcher](https://github.com/danielesteban/three-raymarcher)**
- Abstraction ray marching SDF pour Three.js. **Support R3F**
- npm installable, shapes (boxes, capsules), raycaster intersection

**[MelonCode/r3f-raymarching](https://github.com/MelonCode/r3f-raymarching)**
- Fork R3F-natif. API déclarative React pour ray marching

**[Inigo Quilez — SDF functions](https://iquilezles.org/articles/distfunctions/)**
- La bible des Signed Distance Functions. Co-fondateur de Shadertoy
- Toutes les primitives SDF, opérations booléennes, smooth union, répétition

---

## 8. Projets de référence spectaculaires

**[100,000 Stars](https://stars.chromeexperiments.com/)** — Google Chrome Experiment
- THREE.js + CSS3D. 119,617 étoiles du catalogue HYG réel
- 87 étoiles individuellement identifiées
- Case study : web.dev/case-studies/100000stars

**[jsOrrery](https://github.com/mgvez/jsorrery)** — 425 stars, MIT
- Simulateur de mécanique orbitale du système solaire, positions précises par date
- Éléments orbitaux Kepler de NASA JPL, théorie ELP2000-85 (Lune), VSOP87 (Terre)

**[Three.js Roadmap — Black Hole WebGPU Tutorial](https://threejsroadmap.com/blog/raytracing-a-black-hole-with-webgpu)**
- Tutorial step-by-step : trou noir en TSL + WebGPU
- Lensing, disque d'accrétion avec coloration corps noir, Doppler beaming

**[Three.js Roadmap — Galaxy Simulation](https://threejsroadmap.com/blog/galaxy-simulation-webgpu-compute-shaders)**
- 1M+ particules, génération procédurale, nuages de poussière, interactivité

---

## 9. Infrastructure R3F + post-processing

| Lib | Usage | Pertinence |
|-----|-------|-----------|
| `@react-three/fiber` | Renderer React pour Three.js | Base de tout. `frameloop="demand"` pour 0 GPU au repos |
| `@react-three/drei` | Helpers : Stars, Float, Sparkles, shaderMaterial, Html | Stars pour fond, shaderMaterial pour custom GLSL |
| `@react-three/postprocessing` | Bloom, God Rays, ChromaticAberration, DepthOfField | Bloom sélectif sur les peers actifs, god rays sur les artefacts |
| `@react-three/gpu-pathtracer` | Path tracing GI dans R3F | Rendu photoréaliste pour screenshots/previews |
| `three-raymarcher` | Ray marching SDF déclaratif | Nébulae, trous noirs via SDF |
| `pmndrs/postprocessing` | EffectComposer, UnrealBloom, SSAO | Post-processing pipeline complet |

### Performance — R3F en background

```jsx
// Le background 3D ne consomme 0 GPU quand rien ne bouge
<Canvas frameloop="demand" dpr={[1, 1.5]} performance={{ min: 0.5 }}>
  <Suspense fallback={null}>
    <SpaceScene />
  </Suspense>
</Canvas>
```

- `frameloop="demand"` : rend uniquement sur `invalidate()` (mouvement souris, événement réseau)
- `performance={{ min: 0.5 }}` : DPR adaptatif, réduit la résolution si le device rame
- Quand une iframe app est active → suspendre le rendu 3D complètement
- RTX 5080 = 0 problème de budget GPU pour un background spatial

---

## 10. Vision — ce qu'on peut construire pour SBFB

### Concept : le réseau P2P comme phénomène cosmique

Le shell SBFB ne montre pas un "graphe de réseau" classique (noeuds + lignes).
Il montre le réseau comme un **phénomène astrophysique vivant** :

**Background ambient** (toujours visible derrière l'UI) :
- Nébula volumétrique subtile (ray marching FBM, opacité 10-20%)
- Star field procédural (Points géometry, corps noir Planck pour couleur)
- Lente rotation/drift (0.001 rad/s) pour donner de la vie

**Espace Discover — le Store** :
- Chaque curator = un **amas stellaire** (cluster de points lumineux)
- Chaque app = une **étoile** dans l'amas, taille proportionnelle aux endorsements
- Les curators que tu suis = amas au premier plan (focus)
- Les curators du réseau = amas en arrière-plan (flous, depth of field)
- Zoom sur un amas = les étoiles individuelles deviennent des app cards

**Espace Node — ton noeud** :
- Ton noeud = une **étoile centrale** avec corona (bloom sélectif)
- Les peers connectés = étoiles satellites avec lignes de connexion lumineuses
- Les tâches en transit = **flux de particules** entre les étoiles (comme un disque d'accrétion)
- Le trou noir au centre = métaphore du compute (les tâches "tombent" dedans et des résultats "émergent")

**Espace Run — app lancée** :
- Transition : l'étoile de l'app se rapproche en **zoom cosmique** (fly-through)
- L'iframe s'ouvre comme un "portail" dans l'espace
- Background spatial toujours visible autour des bords (pas noir plat)

**Événements réseau en temps réel** :
- Nouveau peer = flash lumineux (nova)
- Tâche complétée = pulse de lumière entre deux étoiles
- Peer déconnecté = étoile qui s'éteint (fade out 2s)
- Panic wipe = **supernova** visuelle (burst blanc puis noir)

### Architecture technique recommandée

```
Layer 0 : Canvas R3F (background, z-index: -1)
  ├── SpaceBackground (nebula FBM + star field)
  ├── NetworkConstellation (peers = étoiles, connexions = lignes)
  └── PostProcessing (bloom sélectif, depth of field)

Layer 1 : UI React (foreground, glassmorphic)
  ├── Sidebar (~200px, bg-sidebar/65 backdrop-blur)
  ├── TopBar (48px, blur)
  └── Content (pages Discover/Node/Publish)

Layer 2 : Iframe apps (RUN mode, z-index: 10)
  └── AppFrame (full-screen sandbox)
```

Le Canvas R3F est un **fond permanent** derrière l'UI glassmorphic.
Le `backdrop-blur` de la sidebar laisse entrevoir le space background.
Quand une app tourne (RUN), le Canvas passe en `frameloop="never"` → 0 GPU.

### Ce qui est faisable maintenant vs futur

**Sprint frontend v2 (immédiat)** :
- Star field procédural (drei `<Stars>` ou Points custom) — 1h
- Nébula volumétrique subtile (shader FBM) — 4h
- Bloom sélectif sur éléments actifs — 2h
- Background spatial derrière le glassmorphism — 1h

**Sprint suivant (enrichissement)** :
- Peers comme constellation interactive (react-force-graph-3d ou custom) — 8h
- Flux de particules entre peers pour les tâches — 4h
- Transitions cosmiques entre espaces (fly-through) — 4h

**Post-v1.0 (spectaculaire)** :
- Trou noir Kerr comme métaphore du compute (disque d'accrétion = task queue) — 16h
- Wormhole comme transition entre app pages — 8h
- Formation de galaxie N-body comme animation d'onboarding — 8h
- Atmosphère planétaire (Rayleigh+Mie) pour les pages d'identité — 4h

---

## 11. Ce document n'est PAS

- Un plan de sprint
- Du code
- Une décision figée

C'est un catalogue de ce qui EXISTE en simulation spatiale réaliste pour le browser,
avec une vision de comment l'appliquer au shell SBFB. Les choix techniques seront
gelés dans un kickoff.
