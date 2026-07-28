# Chapitre 6 — Conception, décider avant de construire

Le moment le plus coûteux d’un projet n’est pas toujours celui où l’on écrit du
code. C’est parfois celui où une hypothèse implicite devient une architecture
et commence à produire des conséquences.

Si ce passage n’est pas visible, Factory recevra une demande ambiguë et
comblera les vides. Les agents choisiront un modèle de données, une interface,
une politique de permission ou une dépendance. Le résultat pourra être
techniquement cohérent tout en répondant au mauvais problème.

Conception existe pour empêcher la fabrication de devenir, par défaut, le lieu
de toutes les décisions.

## Un pont, pas un péage obligatoire

Conception reçoit généralement un `DecisionPack` issu de l’Atelier des
communs. Elle peut aussi partir d’une décision humaine déjà documentée. Son
objectif est de transformer cette décision en choix de produit et choix
techniques auditables.

Elle n’est pas obligatoire pour chaque modification. Une correction locale,
une migration bien comprise ou une tâche de maintenance peuvent aller
directement vers Factory. Conception devient particulièrement utile lorsqu’un
travail touche plusieurs frontières : utilisateurs, données, sécurité,
protocole, accessibilité, performance ou gouvernance.

Ce caractère optionnel est important. Sinon, un outil conçu pour réduire les
erreurs deviendrait une bureaucratie universelle.

## Quatre regards qui commencent séparément

La V1 organise au moins quatre analyses parallèles.

L’**architecture produit** décrit les personnes, les parcours, les états, les
erreurs, les non-objectifs et les effets attendus.

La **recherche UX et accessibilité** examine la compréhension, les modalités
d’interaction, les barrières, les contextes mobiles ou hors ligne et la
possibilité d’accomplir le même travail sans vision, souris ou vocabulaire
technique.

L’**analyse sécurité et confidentialité** cherche les acteurs malveillants,
les fuites, les secrets, les abus de pouvoir, les effets irréversibles et les
données qui ne doivent pas quitter leur contexte.

L’**analyse d’impact protocolaire** distingue ce qui peut rester dans une
application de ce qui modifierait un format partagé, une capacité, une
signature ou une règle commune.

Ces analyses commencent séparément. Si elles reçoivent d’abord une synthèse
unique, elles risquent de devenir quatre justifications du même choix.

## La contradiction comme méthode de convergence

Les quatre regards produisent des options et des compromis. Ils se relisent
ensuite de manière contradictoire.

L’UX peut montrer qu’une architecture correcte impose une interaction
impossible à certaines personnes. La sécurité peut démontrer qu’un parcours
simple concentre trop de données. L’analyse protocolaire peut refuser qu’une
logique métier alourdisse le daemon de tous les nœuds. Le produit peut constater
qu’une protection proposée retire l’utilité principale.

Le but n’est pas que chaque spécialiste gagne sa bataille. La cross-review rend
les tensions explicites afin qu’un humain puisse arbitrer.

Les décisions importantes deviennent des enregistrements d’architecture :
contexte, options, décision, conséquences, alternatives rejetées et condition
de réexamen. Une option écartée n’est pas effacée. Si le contexte change, elle
peut être retrouvée sans réinventer tout le débat.

## Le `ConceptionPack`

Le résultat de Conception est un artefact structuré et versionné. Il contient
au minimum :

- le problème, les personnes concernées et les travaux à accomplir ;
- le périmètre et les non-objectifs ;
- les flux, états, erreurs et modes dégradés ;
- les frontières d’autorité entre application, hôte, runtime et réseau ;
- le modèle de données et les contrats d’entrée/sortie ;
- les décisions d’architecture et leurs alternatives ;
- les menaces, la confidentialité et les règles de rétention ;
- les exigences d’accessibilité, de langue, de performance et d’adaptation aux
  tailles d’écran ;
- les capacités, destinations et consentements nécessaires ;
- les critères d’acceptation et la stratégie de preuve ;
- les migrations, inconnues et risques résiduels ;
- la provenance des sources et des contributions agentiques.

Les maquettes, prototypes ou recherches lourdes restent des artefacts référencés
par le pack. Le texte ne les avale pas jusqu’à devenir un document impossible
à maintenir.

Factory peut ainsi consommer des champs et des contrats plutôt qu’interpréter
une prose fragile.

## Ce que les agents font bien

Des agents sont particulièrement utiles pour lire de grands corpus, comparer
des options, retrouver une contrainte oubliée, générer des scénarios d’erreur,
contraster plusieurs architectures et vérifier la cohérence d’un pack.

Ils peuvent aussi simuler des perspectives : opérateur, utilisateur débutant,
auditeur, personne travaillant hors ligne ou mainteneur qui reprendra le projet
dans deux ans.

Cette simulation ne remplace pas ces personnes. Un agent ne vit pas la fatigue
d’un parcours, la crainte d’exposer une donnée ou l’usage d’un lecteur d’écran
dans un contexte réel. Le pack distingue donc ce qui vient d’une analyse
agentique, d’une source, d’un test ou d’une validation humaine.

## La décision reste humaine

Une recommandation de Conception ne devient pas une décision parce qu’elle est
majoritaire parmi quatre agents. Le système peut détecter des incohérences
déterministes, mais il ne calcule pas l’importance relative d’une valeur, d’un
risque et d’un délai.

L’arbitre humain peut retenir une option moins performante parce qu’elle est
plus réparable, refuser une fonction utile parce qu’elle collecte trop de
données, ou accepter un pilote dont les limites sont clairement déclarées.

Cette décision est conservée et reliée au digest du pack. Si le pack change,
l’approbation précédente ne couvre pas silencieusement la nouvelle version.

## Un instrument d’économie morale

Conception n’ajoute pas seulement de la rigueur technique. Elle force le projet
à dire où il place le coût.

Une interface très simple peut transférer une charge invisible aux
modérateurs. Une synchronisation totale peut transférer le risque aux personnes
dont les données sont répliquées. Une automatisation rapide peut transférer le
travail aux reviewers. Une preuve exhaustive peut devenir une surveillance.

En rendant ces déplacements visibles avant la fabrication, Conception donne
une forme pratique à la responsabilité.

Elle ne garantit pas que le bon choix sera fait. Elle garantit quelque chose de
plus réaliste : le choix ne pourra pas être attribué après coup à « la
technique » alors qu’il résultait d’un arbitrage humain.

Une conception, même excellente, ne produit pourtant aucun effet par elle-même.
Il reste à transformer ses décisions en organisation exécutable, puis cette
organisation en fabrication. C’est le passage de la partie suivante.
