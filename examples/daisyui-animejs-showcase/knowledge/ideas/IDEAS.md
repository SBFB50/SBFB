# Idea Engine — shortlist pour curation

> 12 concepts retenus (32 gen-1 + 12 mutants gen-2, juges adversariaux 5-dim). Concepts SEULEMENT — aucun rendu. Tri humain attendu avant build.

**Synthese** : Cette fournee fait basculer la vitrine d'effets-temps-pur (counters, scrubs, logos, staggers) vers une famille d'animations a ETAT PERSISTANT et BOUCLE DE RETROACTION : derivees d'ordre 2-3 (jerk, courbure) filtrees par cascades de damp, integrateurs a fuite, treillis couples et lectures geometriques reelles (getPointAtLength brut, getBoundingClientRect d'un FLIP). Les meilleurs candidats — sismographe-freeze, surfeur-arclength, satellite-comete, train-couple, gare-mesuree — partagent un trait absent du corpus existant : la sortie n'est plus devinable depuis le nom des primitives, et plusieurs corrigent a la source des failles techniques reelles (engine.speed jamais anime, motion-path non-interrogeable, synchronie by-construction). La diversite est volontairement etalee sur cinq archetypes visuels distincts (instrument sismique, rail SVG vivant, mecanique d'engrenages, panneau de gare FLIP, sequenceur a groove) pour eviter trois costumes du meme moteur damp-derive. Les 6 recommandes couvrent toutes les lentilles tout en restant CSP-faisables et perceptibles.

## Tableau de tri

| # | titre | total/50 | build | reco | gen |
|---|---|:--:|:--:|:--:|:--:|
| 1 | Le sismographe — le temps se fige sur les a-coups, pas sur les beats | 42 | M | ★ | 2 |
| 2 | Surfeur a contre-courant : il rate les coudes du rail vivant | 42 | L | ★ | 2 |
| 3 | Le satellite-comete sur un rail vivant qui chauffe quand le rail le fouette | 41 | L | ★ | 2 |
| 4 | Le train d'engrenages qui transmet son couple comme une onde | 40 | M | ★ | 2 |
| 5 | Le panneau de gare pilote par la course reelle des cartes | 40 | L | ★ | 2 |
| 6 | Le panneau qui chauffe par a-coups ET vire de teinte dans les virages | 38.5 | M | ★ | 2 |
| 7 | L'engrenage avec du jeu | 37.5 | M |  | 1 |
| 8 | Le groove dont la maille se resserre quand la tete freine | 38 | M |  | 2 |
| 9 | L'odometre qui s'emballe quand le rail se tord | 38 | L |  | 2 |
| 10 | La cascade pilotee par sa propre vitesse | 37 | M |  | 1 |
| 11 | Le fond qui pulse au rythme du reclassement | 37 | M |  | 1 |
| 12 | Surfeur d'inertie sur rail vivant | 37 | M |  | 1 |

## 1. Le sismographe — le temps se fige sur les a-coups, pas sur les beats  ★

`jerk-freeze-decoupled-clock-g2` — gen 2 — **42/50** — build **M**

Une aiguille-stylet (segment SVG var(--ink)) trace une oscillation auto-pilotee (Lissajous sin/cos) sur une bande de papier defilante. Tant que le trace est lisse, tout coule a vitesse normale. Mais des qu'un VIRAGE nerveux survient (le jerk, derivee 3e de la position, depasse un seuil), TOUTE la scene plonge au ralenti — l'aiguille suspendue, le papier presque arrete, une onde de choc figee — puis repart. Le spectateur ne comprend pas pourquoi le ralenti tombe pile sur les saccades et jamais sur les courbes douces : aucun beat n'est scripte, c'est la physique du trace qui declenche le freeze.

- **Gap/base attaqué** : cross_product engine.globalControls x ease.spring x stagger (#13, unexplored) corrige par cross_product createTimer.onUpdate x utils.damp x animatable.property.getter (#1, unexplored) ; croisement coverage_gap 'engine.speed jamais ANIME' + 'damp en cascade derivative + reinjection croisee'
- **Primitives** : `engine.globalControls`, `createAnimatable`, `animatable.property.getter`, `utils.damp`, `utils.mapRange`, `utils.clamp`, `value.functionBased`, `tween.property.cssVar`
- **Leviers** : dilatation temporelle globale (engine.speed) et locale (stretch/sub-progress) · physique fake par damp/lerp derivative frame-independante · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc)
- **Scores** : surprise 9 · combo 8 · procédural 9 · vivacité 7 · anti-déjàvu 9
- **Pourquoi neuf** : Aucun exemple-banque n'anime engine.speed (seulement LU en playback) ET aucun ne couple une derivee d'ordre 3 a un effet d'orchestration. Le dejavu_corpus n'a ni bullet-time ni declencheur par grandeur derivee. La rampe est une fonction de l'etat emergent (le jerk filtre, jamais reproductible identiquement) et l'horloge est physiquement decouplee par rAF natif hors-moteur — croisement inter-lentille dilatation x physique-fake-derivative inedit.

**Mécanisme** : Une cible JS {t} auto-pilotee : x=R*cos(1.3t), y=R*sin(2.1t) injectee dans un createAnimatable({x,y}) lu chaque frame par getter (anim.x()/anim.y()). DANS UNE BOUCLE requestAnimationFrame BRUTE (pas un anime Timer) on lit performance.now() pour dt reel, on derive en cascade vx=damp(vx,(x-px)/dt,dt,0.9) puis ax=damp(ax,(vx-pvx)/dt,dt,0.8) puis jx=damp(jx,(ax-pax)/dt,dt,0.55) (factors decroissants = filtrage anti-bruit de la derivee 3e, traite la faille de finition jerk-snap). |jerk| est mappe par mapRange+clamp en une cible de freeze freezeTarget in [0,1] ; on amortit freezeNow=damp(freezeNow,freezeTarget,dt,0.4) puis engine.speed = lerp(1,0.06,freezeNow). POINT CARDINAL (corrige l'auto-recursion du candidat de base) : cette boucle est un requestAnimationFrame natif de l'iframe, qui n'est PAS un tickable du moteur — engine.speed propage child.speed a tous les tickables anime mais PAS au rAF du document, donc l'horloge de freeze lit toujours le temps reel et le profil de freeze reste fidele. Le trace lui-meme (animate du segment SVG vers les coords cibles, et le papier qui defile) est un tickable anime classique, donc IL ralentit avec engine.speed sans une ligne recodee. Le meme rAF ecrit --freeze=freezeNow sur :root via setProperty ; un calque ::after consomme opacity:calc(var(--freeze)*0.7) (desaturation composite-safe) et une grille de graduations consomme calc(1 - var(--freeze)*0.4) en scaleX. 1 source (le jerk) -> N consommateurs CSS.

**Plan CSP** : 0 reseau / 0 fetch : la cible est une pure fonction trigonometrique de performance.now() local, donnees entierement simulees. anime en UMD window.anime (utils.damp/mapRange/clamp/lerp, createAnimatable, engine). SVG peint en var(--ink)/var(--paper) via CSS (jamais fill-* Tailwind). AUCUN box-shadow anime : le 'choc' visuel = opacity statique sur ::after + scaleX, tout composite. Le rAF natif (requestAnimationFrame du document iframe) est autorise (pas un worker, pas de reseau). Strings utilisateur en francais ('Sismographe temporel', 'Le temps se cabre sur les a-coups').

**Reduced-motion** : Si prefers-reduced-motion : on n'enregistre jamais la boucle rAF de freeze, engine.speed reste fige a 1, et la cible est posee a un t median fixe (aiguille au repos sur une graduation, --freeze=0, papier immobile). Etat-final lisible, zero dilatation, zero derivee active.

---

## 2. Surfeur a contre-courant : il rate les coudes du rail vivant  ★

`living-rail-arclength-surfer-g2` — gen 2 — **42/50** — build **L**

Un point lumineux var(--spark) poursuit une cible qui glisse le long d'un rail SVG en train de morpher (vague -> boucle serree). Le point n'est JAMAIS branche sur un motion-path : a chaque frame on lit la cible par path.getPointAtLength() BRUT sur le path en cours de deformation, et le point la rattrape par double damp en cascade. Quand le morph cree un coude serre, la cible 'pivote' d'un coup mais le point lourd derape visiblement tangent au virage, projette une trainee conique etiree (var(--trail)) et s'incline, puis se recolle. Une etiquette chiffree affiche la longueur d'arc REELLEMENT couverte par la cible depuis le depart, qui 'respire' parce que le rail change de longueur sous elle.

- **Gap/base attaqué** : coverage_gap : utils.damp en cascade derivative + getPointAtLength brut sur path morphe (corrige la faille partagee des deux candidats de base, motion-path abandonne comme primitive-phare)
- **Primitives** : `createTimer`, `svg.morphTo`, `utils.damp`, `utils.mapRange`, `utils.clamp`, `utils.roundPad`, `value.functionBased`, `tween.property.cssVar`, `composition.add`, `callbacks.onUpdate`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · morph + draw + motion-path imbriques sur un meme trace SVG · composition:'add' pour decomposer un mouvement en porteuse + modulation · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · utils.roundPad / padStart pour compteurs a largeur fixe anti-jitter · modifier procedural par-frame (quantize/sin/bruit) sur n'importe quelle prop
- **Scores** : surprise 9 · combo 8 · procédural 9 · vivacité 7 · anti-déjàvu 9
- **Pourquoi neuf** : Aucun exemple-banque ni dejavu_corpus n'echantillonne getPointAtLength BRUT par frame sur un path en cours de morph. Corrige la faille technique des deux bases (createMotionPath n'est PAS un oracle de position a p) : motion-path est ABANDONNE comme primitive-phare, remplace par sampling explicite. Le derapage 2e ordre est rendu PERCEPTIBLE par le mapping pow+plancher. Distance forte au dejavu.

**Mécanisme** : Un <path id='rail'> seul. createTimer loop (autopilote, 0 pointeur) : p = (sin(0.5*currentTime)+1)/2. On lit L = rail.getTotalLength() et target = rail.getPointAtLength(p*L) DIRECTEMENT chaque frame — surface-cle car svg.createMotionPath ne donne PAS d'oracle de position interrogeable a p arbitraire (il ne produit que des TweenObjectValue {from:0,to:totalLength}), et svg.morphTo+timeline.refresh ne re-echantillonne pas en continu. Pursuite : px=utils.damp(px,target.x,deltaTime,0.6) puis on derive la vitesse lue (vx=damp(vx,(px-pxPrev)/dt,dt,0.85)) et l'acceleration laterale aLat = composante de (v-vPrev) perpendiculaire a v (produit croise normalise). aLat est mappee NON-LINEAIREMENT : t=utils.mapRange(Math.pow(Math.abs(aLat),1.6),0,AMAX,TRAIL_FLOOR,TRAIL_MAX) avec un plancher visible TRAIL_FLOOR, ecrite en setProperty('--trail',t) et en --skew (utils.clamp). Simultanement animate('#rail',{d:svg.morphTo('#rail-loop')}) loop alternate deforme le rail : comme la cible vit sur un path qui change, l'ecart cible/point explose dans les coudes => derapage PERCEPTIBLE par construction. Un 3e tween composition:'add' ajoute un micro-jitter sin haute frequence sur px/py. Le point est un <circle cx=0 cy=0> deplace par x/y absolus. La longueur d'arc parcourue = integration de |target_n - target_{n-1}| accumulee, affichee via modifier utils.roundPad(1)+padStart anti-saut-de-largeur ; elle respire car L varie au morph.

**Plan CSP** : 100% local : aucun fetch/CDN/worker, anime en UMD window.anime, chemins relatifs. Donnees simulees = autopilote sin/cos + integration locale. glow de la trainee = box-shadow STATIQUE sur ::after, SEUL --trail (opacity + scaleX du degrade conique) transitionne, jamais box-shadow. SVG peint via var(--spark)/var(--trail) en CSS (pas fill-* Tailwind). morphTo mono-trace ferme (#rail -> #rail-loop). getPointAtLength lu sur le path reel chaque frame, 0 dependance interaction. Strings UI en francais.

**Reduced-motion** : prefers-reduced-motion : pas de Timer, pas de morph. Le rail est pose a sa forme mediane, le point pose a p=0.5 par un seul getPointAtLength, --trail=TRAIL_FLOOR (trainee minimale statique), etiquette affichant la longueur d'arc de la forme mediane. Etat fixe, lisible, zero a-coup.

---

## 3. Le satellite-comete sur un rail vivant qui chauffe quand le rail le fouette  ★

`rail-vivant-courbure-jerk-comete-g2` — gen 2 — **41/50** — build **L**

Un anneau SVG ferme morphe lentement (cercle -> ellipse tordue -> haricot -> retour) en boucle. Un satellite le suit, mais en RETARD amorti : quand le rail se tord vite, le satellite se fait fouetter. On derive du suivi reel DEUX grandeurs decorrelees : son JERK (a-coups du fouet) qui l'etire en comete (skew + scaleX + trainee opacity), et la COURBURE de sa propre trajectoire (produit vectoriel des deltas) qui vire la teinte du satellite ET de l'anneau via une CSS var en conic. Un integrateur a fuite retient la chaleur : apres un fouet le satellite reste incandescent puis refroidit. Le motion-path n'est PAS vestigial : il fournit la position-CIBLE re-echantillonnee sur la geometrie morphee, et c'est l'ECART amorti cible-vs-position qui genere le jerk et la courbure.

- **Gap/base attaqué** : cross_product #8 (createMotionPath x morphTo x timeline.add) croise avec cross_product #1 (damp derivative) et lever 6 (CSS var bus). Croisement de DEUX clusters qui ne co-occurrent jamais : 'SVG morph/motion-path imbrique' et 'frame-rate decouple smoothing derivative'.
- **Primitives** : `svg.morphTo`, `svg.createMotionPath`, `svg.createDrawable`, `timeline.refresh`, `utils.damp`, `createTimer`, `callbacks.onUpdate`, `tween.property.cssVar`, `core.createTimeline`, `utils.clamp`, `utils.mapRange`
- **Leviers** : morph + draw + motion-path imbriques sur un meme trace SVG · physique fake par damp/lerp derivative frame-independante · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc)
- **Scores** : surprise 8 · combo 9 · procédural 9 · vivacité 7 · anti-déjàvu 8
- **Pourquoi neuf** : Corrige la faille 'motion-path vestigial' : ici le motion-path FOURNIT la cible re-echantillonnee sur la geometrie morphee, et c'est l'ecart amorti cible-vs-satellite qui PRODUIT le jerk et la courbure. Le cluster morph/motion-path imbrique co-occurre toujours avec un logo (dejavu) ; ici avec cascade derivative damp 3e-ordre + bus CSS decorrele, combinaison absente du corpus. Couleur (courbure) et incandescence (jerk retenu) divergent visiblement.

**Mécanisme** : createTimeline : (a) createDrawable(['#rail']) draw '0 0'->'0 1' au boot ; (b) une chaine morphTo cercle->ellipse->haricot->cercle en loop sur #rail. Un objet animatable cache parcourt createMotionPath('#rail',0.0001) (offset!=0 = wrap propre, cx=0/cy=0) pour fournir la POSITION-CIBLE le long du rail courant ; onLoop du morph appelle tl.refresh() pour re-echantillonner getPointAtLength. Un createTimer separe fait la physique : suivi MOU damp (0.18) du satellite vers la cible ; PRE-DAMP de la vitesse avant re-derivation (anti-flicker) ; jerk = hypot des deltas de vitesse pre-dampee ; courbure = |cross(v, dv)|/|v|^3 ; integrateur a fuite heat=heat*pow(0.9,dt/16)+jerk*dt*0.0012. hot pilote scaleX/skewX + opacity de la trainee ; bend pilote conic-gradient de l'anneau et du satellite. jerk et courbure restent decorrelees : fouet rectiligne chauffe sans virer, long arc lent vire sans chauffer.

**Plan CSP** : 0 reseau : geometrie SVG locale (getPointAtLength/morphTo/createMotionPath) + morph en boucle, aucun pointeur. anime UMD window.anime.svg.{morphTo,createMotionPath,createDrawable} + utils.{damp,mapRange,clamp}. PIEGES SBFB respectes : motion-path => element cible cx=0 cy=0 ; morphTo mono-trace ferme path d<->d ; SVG peint var(--spark)/var(--orbit) jamais fill-* Tailwind. AUCUN box-shadow anime : comete = scaleX/skewX + opacity trainee + opacity ::after radial statique. Strings FR. Chemins relatifs.

**Reduced-motion** : prefers-reduced-motion : pas de Timer physique ni de morph ; on seek le draw a '0 1', on pose le rail sur sa forme mediane (ellipse), satellite a offset 0.5, --comet=0 et --bend=0.3. Image fixe nette.

---

## 4. Le train d'engrenages qui transmet son couple comme une onde  ★

`train-couple-onde-amortie-g2` — gen 2 — **40/50** — build **M**

Un train de 7 engrenages SVG (var(--metal)) tourne en continu, mais le couple ne se transmet pas instantanement : quand le moteur force (impulsion auto-pilotee sur la roue motrice), une bosse de vitesse angulaire VOYAGE de roue en roue avec retard et amortissement — la 3e roue accelere quand la 2e l'a deja fait, puis le sur-regime reflue. On VOIT une chaine cinematique elastique : elle force, rattrape, transmet, et le jeu mecanique (backlash) tremble dans les phases d'a-coup. Aucune call() discrete : la propagation est emergente.

- **Gap/base attaqué** : cross_product[utils.wrap(modifier)+tween.relative+composition.add] (fresh) croise avec coverage_gap[utils.damp en cascade derivative + reinjection croisee] + coverage_gap[tween.property.cssVar comme bus central anime]
- **Primitives** : `createTimer`, `utils.damp`, `utils.wrap`, `utils.clamp`, `utils.mapRange`, `value.functionBased`, `composition.add`, `tween.property.cssVar`, `callbacks.onUpdate`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · composition:'add' pour decomposer un mouvement en porteuse + modulation · modifier procedural par-frame (quantize/sin/bruit) sur n'importe quelle prop
- **Scores** : surprise 8 · combo 8 · procédural 9 · vivacité 7 · anti-déjàvu 8
- **Pourquoi neuf** : Etat angulaire DECOUPLE du rendu (accumulateur continu vs wrap pixel-only) donc les impulsions ne sautent jamais au rebouclage, et les call() discrets sont remplaces par une onde de couple emergente (reinjection croisee damp vel[i-1]->vel[i]). Aucun cluster ne couple un treillis lateral de VITESSE a des engrenages, et le cluster wrap/carrousel n'utilise jamais wrap en rendu-seul sur etat continu. Zone vierge vs dejavu. La bosse de regime voyage le long du train — perceptible.

**Mécanisme** : Etat JS pur par roue i in [0,7) : angle[i] CONTINU non-wrappe (accumulateur), vel[i] vitesse angulaire. Un seul createTimer render-loop lit self.deltaTime. La roue motrice 0 recoit une cible de vitesse vBase + impulsions auto-pilotees (sin/cos de currentTime). Couplage lateral dans le domaine VITESSE : vel[i] = utils.damp(vel[i], targetVel[i], dt, K[i]) ou targetVel[i] depend de vel[i-1] (reinjection croisee amont->aval avec K[i] decroissant = retard de transmission). angle[i] += vel[i]*dt s'accumule SANS JAMAIS wrapper l'etat. Le wrap n'intervient QU'au rendu : chaque roue ecrit --w{i} = utils.wrap(angle[i],0,360). CORRECTION DU SEAM : pour combiner porteuse (cssVar) et backlash additif, DEUX <g> imbriques — outer pilote par var(--w{i}) en CSS, inner anime par un tween composition:'add' (sin +-2deg loop rapide, function-based par roue) — la decomposition fonctionne car les deux canaux sont sur des elements distincts, jamais ecrases. Une var --strain = mapRange(clamp(|vel[i-1]-vel[i]|)) module l'opacity d'un degrade chaleur STATIQUE.

**Plan CSP** : 0 reseau / 0 fetch / 0 worker : etat JS + Math + engine tick local. SVG <g> peint var(--metal), degrade chaleur var(--hot), aucun fill-* Tailwind. transform:rotate(calc(var(--w{i})*1deg)) = composite GPU sur le <g> outer ; backlash via composition:'add' sur le <g> inner. Glow chaleur = opacity sur degrade statique. Impulsions auto-pilotees (sin/cos) = iframe inerte OK, 0 pointeur. anime UMD window.anime. Strings FR ('Couple transmis', 'Jeu mecanique').

**Reduced-motion** : prefers-reduced-motion : createTimer ne demarre pas, on pose chaque roue a son angle de repos engrene, 0 backlash, --strain=0. Engrenages visibles, alignes, immobiles : etat-final lisible.

---

## 5. Le panneau de gare pilote par la course reelle des cartes  ★

`gare-pilotee-par-geometrie-mesuree-g2` — gen 2 — **40/50** — build **L**

Une grille facon tableau de departs se reorganise en magic-move (FLIP spring). Pendant que les cartes glissent, une boucle de retroaction MESURE frame-par-frame la distance reellement parcourue par chaque carte (getBoundingClientRect, pas un parametre seede). Cette mesure pilote DEUX choses : (a) un fond conic-gradient + un clip-path en eventail qui pulsent au rythme de la course totale, et (b) le SEUIL par carte qui declenche le re-brouillage Solari de son libelle : seules les cartes qui ont le plus voyage re-brouillent leur statut. L'intensite visuelle ET la reecriture textuelle EMERGENT de la cinematique mesuree.

- **Gap/base attaqué** : cross_product (fusion des deux survivants FLIP : bus-conic layout+spring+shuffle ET split-flap-de-donnees scramble-retarget) + coverage_gap 'layout.createLayout creatif inexploite' + coverage_gap 'utils.damp en cascade derivative' + coverage_gap 'tween.property.cssVar comme bus central'
- **Primitives** : `layout.createLayout`, `layout.AutoLayout.update`, `createTimer`, `callbacks.onUpdate`, `callbacks.onLoop`, `utils.damp`, `utils.mapRange`, `utils.clamp`, `tween.property.cssVar`, `text.scrambleText`, `ease.spring`, `utils.createSeededRandom`, `utils.shuffle`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · loop infini + onLoop:refresh() + RNG seede = generatif reproductible
- **Scores** : surprise 8 · combo 9 · procédural 7 · vivacité 8 · anti-déjàvu 8
- **Pourquoi neuf** : Aucun fingerprint de l'examples-bank ne lit la geometrie WAAPI d'un FLIP en readback. Le croisement layout-FLIP + readback-rect->bus-cssVar + scramble-threshold-par-distance n'existe nulle part. Corrige la faille 'synchronie by-construction' des deux bases : --reorg et le declenchement du scramble sont la consequence MESUREE de la cinematique reelle, pas deux fonctions-du-temps verrouillees par parametre seede.

**Mécanisme** : Ancre dans layout.createLayout (waapi.animate pour les transforms -> rects reellement mesurables frame-par-frame). layout.AutoLayout.update retourne une Timeline dont onComplete declenche le retarget. createTimer.onUpdate + getBoundingClientRect readback = la boucle de retroaction authentique. utils.damp lisse la somme des deplacements (frame-independant). tween.property.cssVar ecrit --reorg sur documentElement. text.scrambleText comme valeur de innerHTML, retarget par seed reproductible. utils.mapRange/clamp pour normaliser distance->[0,1] et rang->seuil.

**Plan CSP** : 0 reseau / 0 fetch : statuts et scores 100% tableaux locaux simules, RNG seede deterministe. anime = window.anime UMD global, scripts classiques, chemins relatifs. Strings utilisateur en FRANCAIS ('VOIE 4 - A L'HEURE', 'RETARDE 6 MIN'). Glow eventuel = box-shadow STATIQUE sur ::after, seul opacity/--reorg transitionne. iframe inerte : auto-pilotage total par Timer loop.

**Reduced-motion** : prefers-reduced-motion: update(cb,{duration:0}) -> les cartes prennent leur place sans glissement ; --reorg fige a une valeur mediane ; aucun createTimer de readback lance ; libelles cibles poses directement en textContent sans scrambleText. Etat final propre, immobile.

---

## 6. Le panneau qui chauffe par a-coups ET vire de teinte dans les virages  ★

`bus-jerk-courbure-integrateur-fuite-g2` — gen 2 — **38.5/50** — build **M**

Une cible invisible glisse en figure de Lissajous (sin/cos de currentTime, 0 pointeur). Un seul Timer la suit en amorti et derive DEUX grandeurs geometriquement decorrelees de la meme trajectoire : le JERK (a-coups, derivee 3e du suivi) et la COURBURE (produit vectoriel des deltas position x vitesse). Le jerk pilote l'etirement-comete (skew/scaleX) d'une grappe de lames SVG ; la courbure pilote l'angle d'un conic-gradient (la teinte vire). Un integrateur a fuite (heat += |jerk|*dt ; heat *= 0.92) RETIENT la chaleur des a-coups passes, si bien que le panneau reste tiede apres une saccade et refroidit lentement.

- **Gap/base attaqué** : cross_product #1 (createTimer.onUpdate x utils.damp x derivee) pousse au-dela du verbatim |accel|->scale ; coverage_gap 'utils.damp en cascade derivative et reinjection croisee' + coverage_gap 'tween.property.cssVar comme bus central'. Cross-product des deux gaps.
- **Primitives** : `createTimer`, `utils.damp`, `tween.property.cssVar`, `callbacks.onUpdate`, `utils.mapRange`, `utils.clamp`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · modifier procedural par-frame (quantize/sin/bruit) sur n'importe quelle prop
- **Scores** : surprise 6.5 · combo 8 · procédural 9 · vivacité 7 · anti-déjàvu 8
- **Pourquoi neuf** : Le cross_product #1 et la examples-bank s'arretent a UNE grandeur (|accel|->scale) et au cluster 'frame-rate decouple smoothing' (lerp/damp en suivi simple). Ici : derivee d'ordre 3 ; DEUX grandeurs geometriquement orthogonales issues du MEME suivi routees vers des consommateurs CSS de natures differentes ; integrateur a fuite = etat persistant inter-frame qui casse la pure-fonction-du-temps ; pre-damp de la vitesse AVANT re-derivation = correction explicite du flicker.

**Mécanisme** : createTimer onUpdate : cible auto ax=sin(t*0.0011)+0.5cos(t*0.0029), ay=cos(t*0.0017)+0.5sin(t*0.0037). nx=damp(px,ax,dt,0.55) ; rawV=(nx-px)/dt ; PRE-DAMP nvx=damp(vx,rawVx,dt,0.4) AVANT re-derivation (tue le flicker numerique du jerk). jerk = hypot des deltas de vitesse pre-dampee. courbure = |cross(v,dv)|/|v|^3 (decorrelee, mesure la rotation pas l'a-coup). INTEGRATEUR A FUITE (etat persistant inter-frame) : heat=heat*pow(0.92,dt/16)+jerk*dt*0.001. --comet=hot pilote scaleX(calc(1+var(--comet)*1.8)) skewX de ~30 lames SVG ; --bend=turn pilote conic-gradient(from calc(var(--bend)*220deg)) ; clip-path band epaisseur calc(2px+var(--comet)*22px). jerk et courbure geometriquement orthogonaux -> deux consommateurs vivent leur vie separement.

**Plan CSP** : 0 reseau / 0 fetch : cible auto-pilotee par sin/cos de currentTime (iframe inerte OK, aucun pointeur). anime UMD window.anime.utils.{damp,mapRange,clamp} + createTimer. SVG peint var(--ink)/var(--color-*) en CSS jamais fill-* Tailwind. Aucun box-shadow anime : 'chaud' = scaleX/skewX + opacity ::after radial statique ; conic/clip consomment vars en calc(). Strings FR ('Chaleur retenue', 'Virage'). Chemins relatifs.

**Reduced-motion** : prefers-reduced-motion : Timer ne demarre pas ; on pose une fois --comet=0.18 et --bend=0.35 (etat median fige, lames legerement etirees, teinte intermediaire). Panneau lisible et calme, sans pulsation.

---

## 7. L'engrenage avec du jeu

`engrenage-jeu-mecanique-additif` — gen 1 — **37.5/50** — build **M**

Un train d'engrenages SVG tourne en continu, mais chaque roue n'a pas un spin parfait : sur la rotation porteuse lineaire s'additionne un micro-backlash (le jeu mecanique reel des dents) et, a chaque demi-tour, une impulsion additive de couple se propage de roue en roue via des call() sur labels. On voit une machine qui a du JEU — elle force, rattrape, transmet — sans aucune physique reelle.

- **Gap/base attaqué** : cross_product: 'utils.wrap (modifier) + tween.relative + composition.add' (engrenage spin perpetuel + wobble additif) croise avec 'timeline.call + label' pour la transmission de couple
- **Primitives** : `composition.add`, `value.functionBased`, `utils.stagger`, `timeline.call`, `timeline.label`, `tween.property.cssVar`, `createTimer`
- **Leviers** : composition:'add' pour decomposer un mouvement en porteuse + modulation · valeurs/positions relatives et positions timeline symboliques · une seule CSS var animee orchestrant N consommateurs
- **Scores** : surprise 7.5 · combo 7.5 · procédural 7 · vivacité 7.5 · anti-déjàvu 8
- **Pourquoi neuf** : La banque fait du spin/wrap (carrousels) et de la composition:'add' (particules) separement, jamais un objet SVG unique en spin perpetuel MODULE par backlash additif. La transmission de couple roue-a-roue via call()+label+impulsion additive est un usage de timeline.call hors de son role canonique. Aucun match dejavu_corpus.

**Mécanisme** : Chaque engrenage SVG (peint var(--metal)) a un tween porteur rotate:'+=1turn' loop ease:linear, modifier:anime.utils.wrap(0,360) = spin infini sans derive. Par-dessus, composition:'add' : rotate keyframes sin +-3deg loop rapide = backlash idle, amplitude function-based par roue. Une timeline maitresse pose des labels 'tic0','tic1'... ; tl.call(fn,'ticN') injecte a chaque demi-tour une impulsion additive courte (animate(roueN,{rotate:'+=6', composition:'add', ease:spring})) qui se propage avec un stagger de delai = transmission de couple. Une CSS var --strain (pilotee par le sin du backlash cumule) module l'opacity d'un degrade chaleur statique sur les axes.

**Plan CSP** : 0 reseau : engrenages SVG inline, 0 fetch. SVG peint via var(--metal)/var(--strain) (fill-* Tailwind ne compile pas). composition.add/wrap/call/label/cssVar/createTimer tous usable=true. Chaleur = degrade box-shadow-free, opacity seule. anime UMD scripts classiques.

**Reduced-motion** : prefers-reduced-motion : spin porteur lineaire LENT uniquement (pas de backlash additif, pas d'impulsions), --strain fige, call() d'impulsion no-op. Engrenages tournant doucement, ou figes a une pose nette en option statique.

---

## 8. Le groove dont la maille se resserre quand la tete freine

`groove-a-maille-variable-g2` — gen 2 — **38/50** — build **M**

Une frise de 18 tuiles avec une tete de lecture auto-pilotee qui accelere et freine (faux-scroll inertiel). La cascade des tuiles ne 'cliquete' pas sur une grille figee : quand la tete FREINE, les tuiles se collent sur un groove DENSE (doubles-croches groupees, maille serree) ; quand elle ACCELERE, la maille s'etire. On VOIT le grain du rythme respirer en continu. Un odometre a largeur fixe defile sur l'integrale de la vitesse sans jamais sauter de colonne.

- **Gap/base attaqué** : cross_product : croise le faux-scroll-derivee (scrub-stagger-derive-vitesse) avec le groove syncope a snap non-uniforme (sequenceur-groove-snap-non-uniforme), en corrigeant la faille snap-vitesse-dependant identifiee dans la critique du candidat de base.
- **Primitives** : `createTimer`, `utils.damp`, `utils.mapRange`, `utils.clamp`, `utils.lerp`, `utils.snap`, `ease.steps`, `tween.property.cssVar`, `utils.roundPad`, `utils.padStart`, `callbacks.onUpdate`, `value.functionBased`
- **Leviers** : scrub d'une timeline par une source non-temporelle (faux-scroll/derivee) · physique fake par damp/lerp derivative frame-independante · modifier procedural par-frame (quantize/sin/bruit) sur n'importe quelle prop
- **Scores** : surprise 8 · combo 8 · procédural 8 · vivacité 6 · anti-déjàvu 8
- **Pourquoi neuf** : Aucun fingerprint examples-bank ne matche : le snap n'y est QUE drag/carousel, le faux-scroll-inertiel-iframe est absent du dejavu_corpus. La faille du candidat de base (tableau d'ancres statique) est corrigee : on interpole entre DEUX jeux d'ancres AVANT le snap, donc l'espacement effectif varie avec la vitesse. Detournement structurel de utils.snap (quantize a maille mouvante) + boucle de retroaction derivative continue.

**Mécanisme** : DEUX jeux d'ancres rythmiques pre-calcules : A_serre (groove syncope) et A_relache (balayage quasi-uniforme), MEME longueur que le nombre de tuiles. Un createTimer auto-pilote p(t) en boucle alternate, lissee par utils.damp (momentum sans scroll reel). Chaque frame on DERIVE v=(p-p_prev) puis t=clamp(mapRange(|v|,V_LENT,V_RAPIDE,0,1)). Cle de la correction : on NE re-evalue PAS utils.stagger (closure memoize). On interpole par-frame anchor_i = lerp(A_serre[i],A_relache[i],t), PUIS delay_i = snap(anchor_i, A_serre) : a basse vitesse l'interpolant colle sur A_serre (paires qui s'allument ensemble) ; a haute vitesse il derive vers A_relache. phase_i = clamp((p - delay_i/SCALE)) applique a scale (ease.steps(2)) et opacity via setProperty par tuile. L'odometre integre |v| et ecrit innerHTML via roundPad(1)+padStart(6,'0').

**Plan CSP** : 100% local : window.anime UMD global, createTimer + utils.{damp,mapRange,clamp,lerp,snap,roundPad,padStart} tous purs (CSP usable). Zero fetch/reseau/worker/CDN, ancres generees au boot par fonction locale seedee deterministe. Pas de pointeur ni scroll (auto-pilote Timer). Pas de box-shadow anime : claquement = scale+opacity composite-safe. DOM tuiles (pas de SVG). Strings UI en francais ('vitesse', 'maille').

**Reduced-motion** : prefers-reduced-motion : on fige t a 0.5, on pose la maille intermediaire une seule fois, toutes les tuiles a leur phase finale sans boucle Timer ni damp ; l'odometre affiche une valeur cible fixe formatee. Etat-final lisible.

---

## 9. L'odometre qui s'emballe quand le rail se tord

`breathing-odometer-rail-g2` — gen 2 — **38/50** — build **L**

Croisement du surfeur d'inertie et de la jauge respirante : deux points (var(--ink) en tete, var(--ghost) en ombre retardee) poursuivent une cible glissant sur un rail SVG qui morphe d'un arc tendu vers un arc affaisse. La cible avance a vitesse-de-parametre constante, mais comme le rail RACCOURCIT/s'allonge au morph, l'odometre affiche la longueur d'arc reellement parcourue qui s'EMBALLE quand le rail se tend et patine quand il s'affaisse. L'ombre traine d'autant plus que l'acceleration laterale est forte : on VOIT pourquoi le chiffre respire.

- **Gap/base attaqué** : cross_product : svg.morphTo + getPointAtLength brut (remplace createMotionPath) + arclength-odometer ; hybride surfeur d'inertie x jauge-anse respirante corrigeant les deux failles
- **Primitives** : `createTimer`, `svg.morphTo`, `utils.damp`, `utils.snap`, `utils.mapRange`, `utils.roundPad`, `value.functionBased`, `tween.property.cssVar`, `callbacks.onUpdate`, `utils.clamp`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · morph + draw + motion-path imbriques sur un meme trace SVG · une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · modifier procedural par-frame (quantize/sin/bruit) sur n'importe quelle prop · utils.roundPad / padStart pour compteurs a largeur fixe anti-jitter
- **Scores** : surprise 8 · combo 8 · procédural 8 · vivacité 6 · anti-déjàvu 8
- **Pourquoi neuf** : Hybride coherent corrigeant LEURS deux failles : getPointAtLength brut sur path morphe remplace createMotionPath/refresh ; la respiration du chiffre n'est plus un effet 2e ordre subtil mais la consequence DIRECTEMENT VISIBLE d'un odometre qui s'emballe pendant qu'on voit le rail se tordre. Aucun exemple-banque ne couple odometre-longueur-d'arc + double-damp leader/ombre + snap non-uniforme comme profil de progression.

**Mécanisme** : Un seul <path id='anse'>. createTimer autopilote : s avance par s += SPEED*deltaTime, modifier utils.snap sur un tableau NON-uniforme [0,0.12,0.12,0.4,0.75,0.78,1] pour marquer un palier puis chuter. Chaque frame : L=anse.getTotalLength() (re-mesure au morph), target=anse.getPointAtLength(s*L) — lecture BRUTE. px/py = damp(...,0.6) pour --ink, ghostX/Y = damp(...,0.30) (ombre retardee). delta arc reel = distance euclidienne target_n->target_{n-1} accumulee dans odo ; comme L varie au morph (animate('#anse',{d:morphTo('#anse-affaissee')}) loop alternate), un meme pas ds couvre une distance differente => l'odometre respire. odo via roundPad(1)+padStart(6,'0'). aLat (derivee par damp en cascade) pilote --ghost-stretch via mapRange(pow(|aLat|,1.4)). UNE var --p=s alimente un conic-gradient de fond.

**Plan CSP** : 0 reseau/fetch/CDN/worker, anime UMD window.anime, chemins relatifs. Donnees simulees : abscisse parametrique + integration odometre + getTotalLength/getPointAtLength sur le path reel. glow eventuel = box-shadow STATIQUE sur ::after, seul --ghost-stretch transitionne. SVG peint var(--ink)/var(--ghost)/var(--fill). morphTo un seul path #anse->#anse-affaissee. Autopilote complet. Strings francaises ('vitesse', 'distance avalee').

**Reduced-motion** : prefers-reduced-motion : aucun Timer ni morph. anse posee a forme mediane, --ink/--ghost a s=0.6, odometre fige a la longueur d'arc de la mediane, --ghost-stretch=FLOOR, --p=0.6. Lisible, immobile.

---

## 10. La cascade pilotee par sa propre vitesse

`scrub-stagger-derive-vitesse` — gen 1 — **37/50** — build **M**

Une frise de 20 tuiles dont la cascade d'apparition est pilotee par une 'tete de lecture' qui n'avance pas a vitesse constante: elle accelere et freine selon un faux-scroll auto-pilote. Quand elle FREINE, les tuiles se groupent rythmiquement (snap sur des ancres); quand elle ACCELERE, elles s'etirent. Un odometre a largeur fixe affiche la position parcourue (0042.7) qui defile sans saut.

- **Gap/base attaqué** : cross_product utils.snap(non-uniforme)+stagger(modifier)+steps croise avec lever 'scrub par source non-temporelle' et 'physique fake damp derivative' — groove rythmique pilote par une vitesse derivee, pas par le temps brut
- **Primitives** : `utils.stagger`, `utils.snap`, `utils.damp`, `utils.mapRange`, `utils.clamp`, `createTimer`, `value.functionBased`, `utils.roundPad`, `utils.padStart`, `callbacks.onUpdate`
- **Leviers** : scrub d'une timeline par une source non-temporelle (faux-scroll/derivee) · modifier procedural par-frame (quantize) sur n'importe quelle prop · physique fake par damp/lerp derivative frame-independante
- **Scores** : surprise 7 · combo 8 · procédural 8 · vivacité 6 · anti-déjàvu 8
- **Pourquoi neuf** : Le dejavu 'tl.progress=mouseX' et 'lerp/damp suivi simple' sont depasses: ici la DERIVEE de la position auto-pilotee module un snap non-uniforme de stagger, chaine d'indirection (Timer->damp->derivee->modifier->odometre) absente de tous les clusters. Pas de pointeur, pas de scroll reel. La boucle de retroaction vitesse->groove est un comportement emergent.

**Mécanisme** : Un createTimer auto-pilote une position p(t)=faux-scroll en boucle alternate, lissee par utils.damp => momentum inertiel sans scroll reel. On DERIVE la vitesse v=(p-p_prev) chaque frame. Cette vitesse pilote DEUX choses: (1) on doit mapRange(v) AVANT le snap (correction de la faille) puis re-deriver les delais par-frame dans onUpdate plutot que re-evaluer utils.stagger — a basse vitesse les delais collent sur les ancres, a haute vitesse la distribution s'etire ; (2) un onUpdate integre |v| et ecrit un odometre via roundPad(1)+padStart(6,'0'). scale des tuiles en ease:steps(2).

**Plan CSP** : Faux-scroll = Timer local alternate (aucun scroll DOM, iframe inerte OK). damp pur (Math.exp) couple au tick engine local. snap/mapRange/clamp/roundPad/padStart purs. 0 reseau/worker, donnees simulees. SVG tuiles var(--tile) via cssVar. Strings FR ('position', 'vitesse', 'distance').

**Reduced-motion** : mediaQuery reduced: faux-scroll fige a mi-course, tuiles posees a leur etat median (toutes visibles, scale 1), odometre affiche la distance totale finale figee, aucun Timer/damp actif.

---

## 11. Le fond qui pulse au rythme du reclassement

`bus-conic-pilote-par-le-flip` — gen 1 — **37/50** — build **M**

Une magie-move de cartes re-triees, mais le FLIP lui-meme PILOTE une seule variable CSS --reorg que consomment un fond conic-gradient et un clip-path : a chaque grande reorganisation le fond entier tourne sa teinte et un balayage en eventail traverse l'ecran, synchrone avec le glissement des cartes. Un compteur de 'distance totale parcourue' par les cartes pendant le FLIP est ecrit dans --reorg, et ce bus unique repeint des dizaines d'elements sans une seule timeline supplementaire.

- **Gap/base attaqué** : cross_product: tween.property.cssVar comme bus central animant conic-gradient/clip-path/calc consommes par N elements ; croise avec coverage_gap createLayout FLIP
- **Primitives** : `layout.createLayout`, `layout.AutoLayout.update`, `utils.createSeededRandom`, `utils.shuffle`, `tween.property.cssVar`, `value.functionBased`, `utils.mapRange`, `createTimer`, `callbacks.onLoop`, `ease.spring`
- **Leviers** : une seule CSS var animee orchestrant N consommateurs (conic/clip/calc) · loop infini + onLoop:refresh() + RNG seede = generatif reproductible
- **Scores** : surprise 7.5 · combo 8 · procédural 6 · vivacité 7.5 · anti-déjàvu 8
- **Pourquoi neuf** : Croise deux zones gaps : createLayout (singleton) ET cssVar-comme-bus-d-orchestration. Le pont 'la magnitude du FLIP nourrit la var qui repeint tout' n'existe nulle part. Le mecanisme rend la nouveaute PERCEPTIBLE (le fond reagit a l'intensite du reclassement). Aucun match dejavu_corpus.

**Mécanisme** : createLayout sur la grille DOM (RNG seede pour le re-tri). Au moment de update(cb), on calcule la magnitude totale du reordonnancement (somme des |delta-rang| des cartes) et on la passe a un tween cssVar PARALLELE : animate(':root',{'--reorg':[0,mapRange(magnitude,0,maxMag,0,1)]},ease:spring) qui revient ensuite a 0. Le fond a conic-gradient(from calc(var(--reorg)*1turn)) et un overlay clip-path:polygon() en calc(var(--reorg)*...). Un seul tween anime la var ; la repeinture massive est gratuite cote CSS, 0 timeline par element.

**Plan CSP** : cssVar et createLayout tous deux usable. SVG eventuel peint via var(--color-*). 0 reseau, donnees simulees. UMD anime global, FR ('Reorganisation'). Glow eventuel = box-shadow STATIQUE sur ::after avec seule opacity:var(--reorg) qui transitionne. conic-gradient/clip-path = repaint CSS pur compatible CSP scellee.

**Reduced-motion** : --reorg fige a 0 (fond teinte de repos, pas de balayage), et update({duration:0}) : les cartes prennent leur classement final immediatement, le bus CSS reste constant.

---

## 12. Surfeur d'inertie sur rail vivant

`morphing-rail-inertia-surfer` — gen 1 — **37/50** — build **M**

Un point lumineux var(--spark) surfe une trajectoire derivee : sa cible glisse le long d'un rail SVG qui morphe lentement d'une vague vers une boucle, mais le point N'ATTEINT jamais la cible — il la poursuit par double damp, et son inclinaison/longueur de trainee est dictee par l'acceleration laterale du virage. Quand le rail morphe et cree un coude serre, le point 'derape' visiblement avant de se recoller, comme une bille lourde sur une piste qui se tord.

- **Gap/base attaqué** : cross_product[svg.createMotionPath, svg.morphTo, timeline.add] croise avec coverage_gap[damp cascade derivative]
- **Primitives** : `svg.morphTo`, `svg.createMotionPath`, `createTimer`, `utils.damp`, `utils.lerp`, `utils.mapRange`, `value.functionBased`, `tween.property.cssVar`, `composition.add`
- **Leviers** : physique fake par damp/lerp derivative frame-independante · morph + draw + motion-path imbriques sur un meme trace SVG · composition:'add' pour decomposer un mouvement en porteuse + modulation
- **Scores** : surprise 8 · combo 8 · procédural 8 · vivacité 6 · anti-déjàvu 7
- **Pourquoi neuf** : Les exemples motion-path branchent l'objet DIRECTEMENT sur le chemin. Ici on insere une couche d'inertie par double damp entre le rail et l'objet : le point a sa propre physique et derape dans les transitoires de courbure. Le rail-qui-morphe-sous-le-suiveur n'existe nulle part dans la banque. Pas le logo cinematique du dejavu.

**Mécanisme** : CORRECTION de la formulation : svg.createMotionPath ne fournit pas un oracle de position interrogeable a progress p — on utilise path.getPointAtLength brut sur le path morphe (echantillonnage manuel) pour lire la cible a p auto-pilotee (p=(sin(0.5t)+1)/2). Le point reel poursuit par px=damp(px,targetX,dt,0.85). vx/ax derives par damp successifs : l'acceleration laterale (composante perpendiculaire a la vitesse) est mappee par mapRange sur rotate (inclinaison) et sur --trail (longueur de trainee). animate('#rail',{d:morphTo('#rail-loop')}) loop alternate deforme le rail ; l'ecart cible/point explose dans les coudes => derapage. Un tween composition:'add' ajoute un micro-jitter sin haute frequence.

**Plan CSP** : 0 reseau. morphTo entre deux <path> mono-trace fermes de meme type. point a cx=0/cy=0 (deplace en x/y absolus apres echantillonnage manuel). SVG peint var(--spark). Trainee = degrade STATIQUE sur ::after, seul --trail transitionne, jamais box-shadow anime. Auto-pilote integral.

**Reduced-motion** : Pas de Timer, pas de morph : rail fige sur la forme 'vague', point pose a p 0.5 sur le rail (px=targetX, ax=0), trainee --trail=0. Etat statique propre.

---

## Mentions honorables

- **Saccade — le monde passe en stop-motion quand ca s'emballe** (`jerk-quantized-stopmotion-g2`) — Meme socle jerk-derive que le sismographe mais module engine.fps en paliers stop-motion (twos/threes) au lieu de speed ; ecarte pour ne pas garder deux variantes du meme declencheur emergent.
- **Le panneau dont la violence du brouillage suit l'a-coup mesure** (`gare-jerk-derive-perturbation-g2`) — FLIP magic-move ou l'a-coup mesure par carte module continument le param perturbation du scrambleText ; recoupe trop le panneau-gare-geometrie-mesuree deja recommande.
- **Deux tetes de lecture, deux mailles qui s'interferent** (`groove-graphite-double-tete-g2`) — Generalisation a 2 sources du groove-maille-variable produisant un battement ; gain surtout '+1 source', emergence survendue (battement deterministe periodique).
- **Le manometre qui sent les a-coups** (`manometre-derivative-acceleration`) — Aiguille mesurant l'acceleration de son propre suivi mou via damp cascade vers un bus --strain a 3 consommateurs ; solide mais transcription quasi-verbatim du cross_product #1.
- **Origami de donnees (% bus -> clip-path + morph)** (`depliage-keyframes-percent-clip-morph`) — UNE var --fold en keyframes overshoot asservit un pli clip-path ET un morph SVG par seek ; idee jolie mais l'argument 'silhouette rebondit' est mecaniquement faux (seek clampe).
- **Le tableau d'affichage qui se reorganise et change ses libelles** (`split-flap-de-donnees-flip`) — FLIP + re-brouillage Solari du libelle a l'atterrissage de chaque carte ; pont inter-cluster reel mais profondeur = fonction deterministe du seed sans retroaction.
