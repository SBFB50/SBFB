# Rapport entre les 12 concepts (IDEAS.md) et SBFB

> Analyse adversariale (2 angles indépendants + synthèse honnête anti-survente). Distingue lien PLATEFORME (conformité bac à sable) et lien SÉMANTIQUE (sens métier SBFB).

## Verdict global

Le lien PLATEFORME est fort et uniforme : les 12 concepts sont 0-reseau / 0-fetch / donnees simulees, donc tous conformes au bac-a-sable et tous re-themables en visualisation de signaux SIMULES (demo de pattern, jamais live), exactement comme #sbfb-vivant. Mais le lien SEMANTIQUE est globalement faible : l'Idea Engine a optimise la nouveaute anime.js (jerk, backlash, FLIP, odometres, rails morphants), pas le sens metier SBFB. Le sens n'emerge qu'apres coup, par re-mapping, et il converge presque toujours sur le MEME petit territoire : sharding/routing/churn (shard_plan.rs, routing.rs, sp-pipeline). Constat decisif partage par les deux angles : AUCUN des 12 ne touche kudos/reputation ni provenance Ed25519 (deja couverts par proof_card et Babel) — donc re-thematiser ces idees ne DIVERSIFIE pas l'arc metier, il DENSIFIE 2-3 cartes existantes (relais, route, cockpit GPU). Honnetement : 4 remaps sont naturels (le mecanisme EST la metaphore du signal), 3 plausibles (mecanique colle mais costume plaque), 5 forces ou redondants (effets-temps purs ou gen-1 subsumes par un meilleur frere). La convergence quasi-parfaite des deux angles independants sur ce classement renforce le verdict plutot qu'elle ne le gonfle.

## Par concept

| # | concept | sens SBFB aujourd'hui | objet SBFB cible | force du remap |
|---|---|:--:|---|:--:|
| 1 | Le sismographe — le temps se fige sur les a-coups, pas sur les beats | fort | churn / sante de shard : derivee de la latence inter-noeuds declenchant le re-route Petals (replace_failed_server, routing.rs) | **naturel** |
| 2 | Surfeur a contre-courant : il rate les coudes du rail vivant | fort | routing.rs route_min_latency vs churn ACTIF (assign_fallback_nodes, replace_failed_server) | **naturel** |
| 3 | Le satellite-comete sur un rail vivant qui chauffe quand le rail le fouette | fort | sp-pipeline (token sur sp-path, deja present) + charge cumulee de shard + niveau N0-N3 | **naturel** |
| 4 | Le train d'engrenages qui transmet son couple comme une onde | fort | shard_plan.rs / data-plane sbfb/shard/1 : propagation d'activation couche-a-couche | **naturel** |
| 5 | Le panneau de gare pilote par la course reelle des cartes | moyen | browse.rs / NodeDirectory : re-classement + joignabilite tri-etat sous churn | **plausible** |
| 6 | Le panneau qui chauffe par a-coups ET vire de teinte dans les virages | moyen | consent.ts gc-cockpit : double signal charge GPU vs niveau de consentement | **plausible** |
| 7 | L'engrenage avec du jeu (gen 1) | moyen | Identique au rang 4 (propagation d'activation) mais transmission discrete au lieu de continue | **force** |
| 8 | Le groove dont la maille se resserre quand la tete freine | faible | Debit tok/s du pipeline vs occupation (sp-tps deja affiche en clair) | **force** |
| 9 | L'odometre qui s'emballe quand le rail se tord | moyen | Debit utile (tokens valides par RunProof) vs travail nominal sous churn | **force** |
| 10 | La cascade pilotee par sa propre vitesse (gen 1) | faible | Cadence tok/s, integralement subsume par le rang 8 | **force** |
| 11 | Le fond qui pulse au rythme du reclassement (gen 1) | moyen | Intensite de churn / volume d'ingest gossip (NodeDirectory) | **force** |
| 12 | Surfeur d'inertie sur rail vivant (gen 1) | faible | routing/churn, integralement subsume par 2 (et 9) | **force** |

### 1. Le sismographe — le temps se fige sur les a-coups, pas sur les beats

- **Sens SBFB aujourd'hui** : fort
- **Objet SBFB cible** : churn / sante de shard : derivee de la latence inter-noeuds declenchant le re-route Petals (replace_failed_server, routing.rs)
- **Re-mapping** : Stylet = RTT/cadence inter-token mesure d'un shard ; jerk de RTT > seuil = signe avant-coureur de churn qui fige TOUTE la timeline du pipeline (fenetre de re-route, shard reassigne au fallback_node, RunProof suspendu), pendant que l'horloge de detection reste en temps reel.
- **Force** : naturel — Tient : le decouplage rAF-natif / engine.speed est isomorphe au design 'le coordinator detecte le churn par derivee, pas par horloge, meme quand le data-plane gele'. Angle 1 le note semantique fort, angle 2 le degrade a faible en le voyant comme extension sp-pipeline — mais le remap churn-par-jerk est reel, pas plaque. Naturel confirme.

### 2. Surfeur a contre-courant : il rate les coudes du rail vivant

- **Sens SBFB aujourd'hui** : fort
- **Objet SBFB cible** : routing.rs route_min_latency vs churn ACTIF (assign_fallback_nodes, replace_failed_server)
- **Re-mapping** : Rail morphant = topologie vivante (matrice RTT) ; cible glissante = route optimale instantanee ; point lourd = route effectivement servie qui rattrape par inertie (re-signature revision+1). Quand un noeud tombe, la route servie DERAPE visiblement = la sur-latence cross-tier failover (carry PULL-3) rendue perceptible. Odometre d'arc reel = re-assignations cumulees qui 'respirent' avec la taille de la topologie.
- **Force** : naturel — Le plus utile des 4 naturels : il rend lisible PULL-3, un carry reel et non encore visualise. Le derapage-dans-le-coude EST la sur-latence transitoire d'un fallback non digere. Aucun forcage.

### 3. Le satellite-comete sur un rail vivant qui chauffe quand le rail le fouette

- **Sens SBFB aujourd'hui** : fort
- **Objet SBFB cible** : sp-pipeline (token sur sp-path, deja present) + charge cumulee de shard + niveau N0-N3
- **Re-mapping** : Deux signaux geometriquement ORTHOGONAUX sur un seul token du relais : jerk -> etirement comete = pic de charge instantanee ; integrateur a fuite (heat *= 0.9^dt) = reputation/kudos OU charge VRAM cumulee qui reste chaude apres un pic puis refroidit ; courbure -> teinte = niveau de verification N0->N3 independant de la charge.
- **Force** : naturel — Naturel mais ATTENTION a la double lecture : angle 1 mappe la chaleur sur kudos/reputation (decay non-monetaire), angle 2 sur VRAM/charge GPU. La version VRAM est plus honnete (le decay-de-reputation est une metaphore plus libre que le refroidissement thermique). Le decouplage charge-vs-confiance est reel des deux cotes. Naturel, en privilegiant la lecture charge/verification.

### 4. Le train d'engrenages qui transmet son couple comme une onde

- **Sens SBFB aujourd'hui** : fort
- **Objet SBFB cible** : shard_plan.rs / data-plane sbfb/shard/1 : propagation d'activation couche-a-couche
- **Re-mapping** : Quasi-isomorphe : chaque engrenage = un shard (couches [0..11), [11..22)...) ; bosse de couple qui voyage avec retard amorti = activation forward-pass qui traverse les shards stage par stage (TTFT = temps pour atteindre le dernier engrenage) ; backlash = jitter de synchro inter-shard ; --strain = ecart de vitesse entre shards consecutifs = bottleneck de debit (stage le plus lent borne le tok/s) ; couplage sans call discret = pipeline emergent sans chef d'orchestre.
- **Force** : naturel — Le remap le PLUS direct du sharding de toute la liste, et le sharding est la feature phare S77. La propagation emergente (pas de call discret) est fidele au data-plane continu/streame. Naturel sans reserve — meilleure metaphore que le token-sur-rail du rang 3 pour le pipeline lui-meme.

### 5. Le panneau de gare pilote par la course reelle des cartes

- **Sens SBFB aujourd'hui** : moyen
- **Objet SBFB cible** : browse.rs / NodeDirectory : re-classement + joignabilite tri-etat sous churn
- **Re-mapping** : FLIP = re-classement Browse/annuaire quand une vague d'ingest gossip arrive ; distance MESUREE par carte (readback getBoundingClientRect) = ampleur du changement de statut (Unknown->Reachable, gain/perte de seeders) ; seules les cartes qui ont le plus voyage re-brouillent leur libelle = seules les apps dont la joignabilite a vraiment change re-affichent 'Toi + N pairs' (modele honnete du best-effort).
- **Force** : plausible — La mecanique FLIP+readback colle bien et le principe 'consequence mesuree, pas seed' est fidele a l'esprit des cartes vivantes. MAIS le 'tableau de departs de gare' reste un costume plaque, et c'est une carte A CREER, pas une extension d'une carte existante (angle 2). Plausible honnete — a garder en reserve si l'arc annuaire/churn merite une carte dediee.

### 6. Le panneau qui chauffe par a-coups ET vire de teinte dans les virages

- **Sens SBFB aujourd'hui** : moyen
- **Objet SBFB cible** : consent.ts gc-cockpit : double signal charge GPU vs niveau de consentement
- **Re-mapping** : Reskin du cockpit GPU (consent.ts) : jerk = pics de charge (rafales de taches), integrateur a fuite = watts/VRAM moyens qui restent eleves apres une rafale puis refroidissent, courbure -> teinte = niveau de consentement L1->L4 ou criticite des taches.
- **Force** : plausible — Les deux angles le disent : c'est MECANIQUEMENT le rang 3 sans le rail morphant — donc plus pauvre (pas de topologie, pas de pipeline) et redondant si 3 est retenu. Plausible seul, force si 3 est pris. Recommandation : ne le retenir QUE si on abandonne 3, ce qui n'est pas le cas.

### 7. L'engrenage avec du jeu (gen 1)

- **Sens SBFB aujourd'hui** : moyen
- **Objet SBFB cible** : Identique au rang 4 (propagation d'activation) mais transmission discrete au lieu de continue
- **Re-mapping** : Memes engrenages = memes shards, mais transmission par call()+label = modele PAR ETAGE DISCRET du forward-pass (chaque appel = un hop d'activation au shard suivant, stagger = RTT relais).
- **Force** : force — Angle 2 est plus genereux (plausible, 'relais pas-a-pas pedagogique') mais angle 1 tranche juste : la transmission discrete par call/label modele MAL le data-plane qui est continu/streame. Le rang 4 fait tout ce que fait 7 en mieux ET de facon emergente fidele. Force / a ecarter au profit du 4. Le 'plausible' de l'angle 2 est une concession pedagogique, pas un vrai gain de sens.

### 8. Le groove dont la maille se resserre quand la tete freine

- **Sens SBFB aujourd'hui** : faible
- **Objet SBFB cible** : Debit tok/s du pipeline vs occupation (sp-tps deja affiche en clair)
- **Re-mapping** : Tete de lecture = front de generation ; vitesse derivee = debit tok/s instantane ; maille qui se resserre au freinage = tokens qui s'accumulent en attente (cadence inter-token densifiee) ; odometre = compteur total de tokens generes.
- **Force** : force — Desaccord entre angles : angle 1 dit plausible ('cadence qui respire avec le debit est reelle en SBFB'), angle 2 dit force ('sp-pipeline affiche DEJA tok/s, le costume groove n'apporte aucune semantique nouvelle'). Angle 2 a raison : le signal est deja couvert, le groove musical n'est qu'une metaphore d'ambiance. Force. Technique pure.

### 9. L'odometre qui s'emballe quand le rail se tord

- **Sens SBFB aujourd'hui** : moyen
- **Objet SBFB cible** : Debit utile (tokens valides par RunProof) vs travail nominal sous churn
- **Re-mapping** : Rail morphant = topologie/RTT ; cible a cadence de dispatch constante (parametre) mais odometre = DEBIT UTILE REEL (tokens confirmes par RunProof) ; point-tete = front de generation, ombre retardee = quorum/RunProof qui confirme avec retard. L'ecart cadence-nominale vs debit-reel = la PROVISIONAL-ite du benchmark cross-machine rendue litterale.
- **Force** : force — Desaccord net : angle 1 le dit naturel (l'idee 'travail abouti diverge du travail nominal selon la sante reseau' = exactement la PROVISIONAL-ite du benchmark, et l'ombre-retardee = quorum qui suit la production), angle 2 le dit force (doublon du rang 2 avec une jauge en plus, l'ombre n'a pas d'analogue clair). Verdict honnete : l'idee ombre=quorum-retarde est SEDUISANTE mais le concept reste le rang 2 + un odometre ; le rang 2 porte deja routing/churn plus proprement. Je tranche FORCE (concede que l'angle 1 a trouve le meilleur habillage des forces, d'ou semantic_tie moyen, mais subsume par 2).

### 10. La cascade pilotee par sa propre vitesse (gen 1)

- **Sens SBFB aujourd'hui** : faible
- **Objet SBFB cible** : Cadence tok/s, integralement subsume par le rang 8
- **Re-mapping** : Aucun convaincant — au mieux 'cascade d'apparition de tuiles = arrivee des tokens', strictement le rang 8 (deja force) en moins abouti (snap sur ancres statiques, pas de maille interpolee).
- **Force** : force — Les deux angles concordent : doublon d'un doublon, aucun signal SBFB propre. A ecarter. Technique pure.

### 11. Le fond qui pulse au rythme du reclassement (gen 1)

- **Sens SBFB aujourd'hui** : moyen
- **Objet SBFB cible** : Intensite de churn / volume d'ingest gossip (NodeDirectory)
- **Re-mapping** : Magnitude totale du reordonnancement (somme |delta-rang|) = intensite de churn / volume d'ingest gossip ; --reorg pilote un fond qui pulse fort quand le reseau se reorganise beaucoup.
- **Force** : force — Angle 1 dit plausible (jauge d'ambiance 'le reseau bouge'), angle 2 tranche plausible-vers-force avec un argument decisif : ici la magnitude vient d'un SEED, ce qui TRAHIT le principe 'jamais synchronie by-construction / consequence mesuree' des composants vivants, alors que le rang 5 fait emerger la mesure de la cinematique reelle. Donc sous-ensemble pauvre du rang 5 ET infidele a l'esprit. Force. Garder 5, laisser 11 en technique.

### 12. Surfeur d'inertie sur rail vivant (gen 1)

- **Sens SBFB aujourd'hui** : faible
- **Objet SBFB cible** : routing/churn, integralement subsume par 2 (et 9)
- **Re-mapping** : Aucun convaincant — 'point lourd = route qui derape dans un churn brutal', strictement les rangs 2 et 9 en gen-1, sans odometre d'arc, sans ombre retardee, sans la non-linearite corrigee.
- **Force** : force — Les deux angles concordent : le plus faible du lot, aucun signal SBFB distinct. Le rang 2 capte toute la semantique routing/churn avec en plus une grandeur mesurable affichee. A ecarter. Technique pure.

## Recommandation

RE-THEMATISER SBFB (4 remaps naturels, par ordre de priorite) : (4) Train d'engrenages -> propagation d'activation couche-a-couche dans shard_plan.rs : la metaphore la plus directe et la plus juste du sharding, feature phare S77 ; a privilegier comme extension de sp-pipeline. (2) Surfeur/arclength -> derapage de routing sous churn : le plus UTILE car il rend lisible PULL-3 (carry reel non encore visualise). (1) Sismographe-jerk -> detection de churn par derivee : isomorphe au design coordinator. (3) Satellite-comete -> charge-vs-verification orthogonales sur un token (privilegier la lecture VRAM/charge, pas la lecture kudos-decay qui est plus libre). ATTENTION : ces 4 ne DIVERSIFIENT PAS l'arc metier — ils densifient tous les memes 2 cartes (relais shard + routing/churn). Si l'objectif est d'elargir #sbfb-vivant vers kudos/reputation/provenance, AUCUN de ces 12 ne sert (deja couverts par proof_card/Babel) ; il faudra des concepts neufs. RESERVE (3 plausibles, seulement si l'arc churn/routing/annuaire merite une carte dediee, et non redondante) : (5) gare-FLIP -> annuaire vivant mesure par readback = le meilleur des 3, fidele a 'consequence mesuree pas seed', mais carte A CREER. (6) panneau jerk+courbure -> cockpit GPU double-signal, MAIS redondant avec 3, donc seulement si 3 n'est pas pris. (7) engrenage-gen1 -> relais discret pedagogique, MAIS subsume par 4. TECHNIQUE PURE / LAISSER TOMBER (5 concepts) : (8) groove, (9) odometre-rail, (10) cascade-gen1, (11) fond-FLIP-gen1, (12) surfeur-gen1 — tous des doublons mecaniques ou des costumes forces dont SBFB ne tire qu'un compteur tok/s ou une jauge debit deja couverts par sp-pipeline. Les garder dans l'arc 'technique' legitime de la vitrine (demo anime.js), sans aucune etiquette SBFB.

---

## Annexe — les 2 angles bruts

### Angle : mecanisme-vers-signal 

mecanisme-vers-signal : pour chaque concept j ai isole le MECANISME PHYSIQUE de l animation (ce qui declenche/propage/mesure) puis cherche le signal SBFB qui se comporte selon la MEME loi. Le re-mapping le plus fort est celui ou le mecanisme EST deja la metaphore du signal (ex : une bosse de couple qui voyage de roue en roue amortie = une activation de shard qui se propage couche par couche ; un seuil de jerk qui fige la scene = un seuil de jerk-de-latence qui declenche le re-route de churn ; un point lourd qui rate le coude d un rail qui morphe = un routeur qui rate l adaptation a un churn brutal). Reference de calibration = la section sbfb-vivant de index.html (verification.rs N0-N3, placement.rs couverture de shard, browse.rs tri-etat, proof_card.rs, daemon.ts redondance, consent.ts GPU, shard_plan.rs/routing.rs relais). Honnetete : 5 concepts portent deja un sens SBFB fort par leur mecanique meme (1,2,3,4,9), 3 sont plausibles avec un re-theme leger (5,8,11), 4 sont des effets-temps purs qui demandent un costume force (6,7,10,12). Conformite bac-a-sable forte partout : tous sont 0-reseau/0-fetch a donnees simulees, donc un panneau SBFB en serait une visualisation de signaux SIMULES (demo de pattern, pas live), exactement comme sbfb-vivant.

### Angle : famille-composants-vivants 

famille-composants-vivants : prolonger l'arc metier deja en place dans #sbfb-vivant. Chaque concept d'animation est juge non pas dans l'abstrait mais comme une EXTENSION possible d'une carte existante (verification.rs N0-N3, placement.rs couverture de shard, browse.rs joignabilite tri-etat, proof_card.rs score additif, daemon.ts redondance « Toi + N pairs », consent.ts cockpit GPU, shard_plan.rs/routing.rs relais d'inference). Verdict d'ensemble : 4 concepts ont un fit NATUREL (le relais sismographe-jerk -> sante d'un shard ; le surfeur/arclength -> derapage de routing sous churn ; le satellite-comete sur rail vivant -> token sur le relais qui chauffe ; le train d'engrenages -> propagation d'activation couche-a-couche). 3 ont un fit PLAUSIBLE (gare-FLIP -> re-routing mesure sous churn ; panneau jerk+courbure -> double-signal RTT/charge ; bus-FLIP -> reclassement de l'annuaire de noeuds). Les 5 restants (groove a maille, odometres, cascade-vitesse, variantes surfeur gen-1) sont des doublons mecaniques ou des costumes forces : SBFB n'y gagne qu'un compteur tok/s ou une jauge debit deja couverte par sp-pipeline. Le grand absent du remap : aucun de ces 12 ne touche kudos/reputation ni provenance Ed25519 (deja couverts par proof_card et Babel) — donc re-thematiser ces idees ne diversifie PAS l'arc metier, il densifie surtout les cartes shard/routing existantes. Recommandation PO : re-thematiser SBFB uniquement les rangs 1-4 (fit naturel, prolongent une carte vivante avec un VRAI signal) ; garder 5-7 en reserve si l'arc churn/routing merite une carte dediee ; laisser 8-12 en technique pure.

