# Chapitre 3 — Une journée dans Nexus

À 7 h 45, Nora pousse la porte métallique de l’atelier de réparation solidaire. Le local ouvre au public le samedi, mais une petite équipe se retrouve aujourd’hui pour résoudre un problème récurrent. Les bénévoles savent réparer beaucoup d’objets ; retrouver la bonne information leur prend parfois plus de temps que la réparation elle-même.

Les manuels sont éparpillés entre un disque partagé, des classeurs et des pages de fabricants. Certains sont en langues étrangères, les scans sont peu lisibles et le réseau passe mal dans la cave. Une bénévole utilise un lecteur d’écran ; plusieurs visiteurs lisent plus facilement l’arabe.

« Mettre tous les manuels dans une application » cache donc des décisions. Tous ne sont pas librement redistribuables, une instruction mal attribuée peut être dangereuse et une traduction fluide techniquement fausse. Un outil dépendant du cloud serait presque inutile sur place.

## 8 h 05 — Formuler le besoin dans Ideas Hub

Nora ouvre Ideas Hub sur son nœud. Elle ne demande pas immédiatement à un agent de générer une application. Elle crée une graine intitulée « Guide de réparation hors ligne, accessible et multilingue ».

Elle décrit les personnes concernées et le dommage actuel : temps perdu, réparations abandonnées, risque d’utiliser la procédure du mauvais modèle. Elle ajoute les contraintes certaines :

- aucune donnée personnelle ;
- aucune republication sans droit établi ;
- source identifiable pour chaque consigne ;
- avertissements avant la procédure ;
- fonctionnement hors ligne ;
- recherche, lecture et impression sans souris.

Ideas Hub lui montre un besoin d’index de pièces et une méthode de numérisation publiés par des ateliers voisins. Les appuis sont des signaux locaux ; aucun compteur ne décide quoi construire.

Nora se porte responsable pendant six semaines ; Yanis devient reviewer sceptique. Leur cible : vingt procédures, cinq familles d’appareils, deux langues, un mode imprimable et un test sur place. Si licences ou consignes ne peuvent pas être validées, le projet restera un index de liens.

Cette prise en charge compte davantage que cent votes anonymes. Le réseau sait désormais qu’un petit groupe accepte de porter la suite, selon quelles limites et avec quel critère d’abandon.

## 8 h 40 — Chercher avant de construire avec RRV

Nora ouvre RRV sur le seul dossier autorisé par l’atelier. Il y retrouve 312 fichiers : doublons, scans sans texte et versions contradictoires.

Chaque résultat conserve son origine. Un hash ne rend pas un PDF exact. Un guide Creative Commons possède licence et provenance vérifiables ; un scan sans auteur ni date reste « non vérifié ».

Sur le réseau SBFB, RRV retrouve :

1. une bibliothèque hors ligne testée au clavier ;
2. un workflow d’OCR conservant les pages ;
3. un schéma reliant chaque fragment à son document et à son digest ;
4. un composant multilingue droite-gauche.

Pertinence et preuve restent séparées. La bibliothèque est adaptée mais ancienne ; l’OCR possède des évaluations sur scans dégradés ; le support arabe du composant est déclaré, pas encore testé avec des utilisateurs.

Nora active volontairement le Web. RRV cite les pages de fabricants comme sources externes sans supposer que publication signifie droit de reprise. Trois documents sont réutilisables ; les autres ne pourront être que référencés.

À 9 h 25, le dossier RRV apporte déjà une conclusion importante : le projet ne doit pas être un miroir universel de manuels. Il doit être un guide construit à partir de contenus réutilisables, complété par un index de références externes lorsque les droits restent incertains.

RRV verse ces résultats, leurs sources et leurs incertitudes dans un
`ResearchPack`. Nora, Yanis et Leïla les examinent. Ils pourraient encore
arrêter le projet, demander une nouvelle recherche ou retenir une autre voie.
Ils choisissent humainement de construire un guide sourcé, hors ligne et
réparable, puis consignent ce choix, les variantes écartées et les contraintes
non négociables dans un `DecisionPack`.

## 10 h — Conception transforme la décision en architecture

Conception reçoit le `DecisionPack`. Ses analyses produit, technique,
accessibilité et sécurité comparent alors trois architectures possibles pour
réaliser la décision, sans décider à la place du groupe si le projet mérite
d’exister.

Un moteur dans les PDF serait rapide mais peu accessible. Des instructions
entièrement générées perdraient le lien précis aux pages. La troisième
architecture sépare corpus autorisé et procédures relues : chaque étape cite
sa source, les contenus non redistribuables ne sont pas embarqués.

Nora, Yanis et Leïla arbitrent à nouveau, cette fois sur la conception. Après
leur accord, le `ConceptionPack` formalise la troisième architecture. Il
définit une application statique hors ligne comprenant une recherche locale,
des fiches par famille et modèle, une vue étape par étape, une liste d’outils,
des avertissements, des citations page par page, deux langues et un export
imprimable.

L’application ne diagnostique pas automatiquement, ne collecte ni identité ni photo, et ne remplace pas la qualification nécessaire aux interventions dangereuses.

Les critères exigent notamment : aucune phrase technique sans source, aucune valeur traduite sans revue, clavier et lecteur d’écran, zoom 400 %, fonctionnement hors ligne, licence visible et distinction entre OCR, proposition et procédure validée.

Conception a spécifié ce qui doit exister et les limites à respecter. Elle n’a écrit ni l’application ni le workflow de production.

## 10 h 35 — Workflow Hub organise la chaîne documentaire

Dans Workflow Hub, Nora forke l’OCR trouvé par RRV. Elle utilise le mode guidé, Yanis l’outline et Leïla le canevas : les trois vues modifient le même modèle.

Ils construisent une chaîne bornée :

1. importer un document et calculer son digest ;
2. identifier sa version, sa langue et sa licence ;
3. refuser la publication si le droit de reprise n’est pas établi ;
4. exécuter l’OCR en conservant pages et zones ;
5. découper le texte en fragments citables ;
6. proposer une procédure à partir des seuls fragments autorisés ;
7. vérifier les valeurs, outils et avertissements ;
8. faire relire la procédure par un humain compétent ;
9. traduire uniquement la version approuvée ;
10. comparer la traduction aux contraintes techniques ;
11. demander une seconde validation humaine ;
12. produire les artefacts destinés à Factory.

Les agents restent bornés : classement des fragments, rédaction sans étape non citée, traduction du seul texte public approuvé. Un validateur déterministe bloque les valeurs dont source et unité ne correspondent pas.

L’OCR reste local. Les procédures publiques pourront être traduites par des workers volontaires. Le shell affiche destination, volume et budget ; Nora autorise ce seul périmètre. Les workers ont eux-mêmes choisi tâches, modèles et limite de consommation.

Le compilateur Rust vérifie schémas, refus, validations humaines et reprises, puis fige le plan 1.0. L’espagnol demandé ensuite part dans un brouillon 1.1 pour une prochaine exécution.

## 11 h 30 — Le run produit des artefacts, pas une illusion de certitude

Le premier run traite six manuels librement réutilisables. L’OCR local retrouve les titres et numéros de page. Deux tableaux très dégradés sont placés en attente au lieu d’être devinés.

Sur vingt-trois procédures proposées, sept sont rejetées : valeur sans unité, modèles mélangés ou avertissement absent.

Yanis examine les seize restantes. Il corrige une confusion entre « couper l’alimentation » et « mettre l’appareil en veille ». La différence paraît stylistique au modèle ; elle est essentielle dans un atelier. La correction devient un cas d’évaluation versionné.

Trois workers traduisent les textes approuvés et signent leurs résultats. Deux divergent sur un verbe technique. Le système expose conflit, modèles et sources au lieu de voter. Un modèle local propose une nouvelle version, validée par un bénévole arabophone.

Une autre tâche reste sans reçu après la disparition d’un worker. Le runtime inscrit `outcome_unknown`. Il ne renvoie pas aveuglément la même demande en prétendant que la première n’a jamais été exécutée. Nora attend l’expiration prévue, puis autorise une nouvelle tentative avec une clé distincte.

À 13 h 10, le run produit un corpus structuré, les citations, quatorze procédures validées en français, douze en arabe, un rapport des refus et les reçus de calcul. Les deux traductions incomplètes restent clairement signalées ; elles ne sont pas publiées sous une apparence de succès.

## 14 h — Factory réalise et prépare la fermeture

Factory reçoit le `ConceptionPack` et les artefacts, puis crée le projet depuis le template retrouvé par RRV en conservant licence, digest et lignée.

L’agent d’implémentation ajoute l’index local, les fiches de procédure, les citations, le changement de langue, les styles d’impression et le cache hors ligne. L’application reste statique. Sa politique de contenu interdit les connexions directes ; les mises à jour passeront par une nouvelle version SBFB.

Une première prévisualisation révèle plusieurs défauts. Les longues références débordent à fort zoom. Le changement de langue ne restitue pas correctement le focus. En arabe, une icône directionnelle conserve le sens gauche-droite. Factory ouvre des corrections bornées et ajoute les tests correspondants.

Les gates vérifient manifeste, licences, secrets, clavier, accessibilité, impression, taille et mode hors ligne. Chaque procédure doit avoir une citation résoluble ; aucun document « référence externe seulement » ne doit entrer dans le build.

Leïla mène la revue d’accessibilité sur la vraie prévisualisation. Yanis relit les avertissements. Une revue indépendante du diff constate qu’un badge « vérifié » pouvait être compris comme une certification générale de la réparation. Il devient « relu par l’atelier pour ce modèle et cette version », avec la date et les limites.

Factory restitue alors le dossier de fermeture : commit proposé, sources, tests, licences, rapport de revue, archive et risques résiduels. Elle ne s’accorde pas son propre verdict. Nora compare le résultat au `ConceptionPack`, puis confirme humainement le commit et la publication.

## 16 h 20 — Publication SBFB et premier usage hors ligne

Le daemon adresse et signe l’archive. La Proof Card donne commit, hash, licences, workflow, tests, sources et limites. Les reçus indiquent quels nœuds ont signé les traductions, sans les déclarer parfaites.

Trois ateliers découvrent le guide dans RRV et l’installent désactivé. Après inspection des capacités, ils l’activent ; deux choisissent aussi de servir l’archive.

À 17 h, le test réel commence dans la cave. Le réseau est coupé. Un nouveau bénévole cherche le modèle d’un grille-pain, suit la procédure française et ouvre la page source correspondante. Leïla parcourt une autre fiche avec son lecteur d’écran. Un visiteur consulte la version arabe, puis imprime une procédure en gros caractères pour l’apporter à l’établi.

Une variante est mal classée. Le formulaire ne demande aucun nom ; Nora publie seulement une projection expurgée avec modèle, version et résultat attendu.

RRV indexe ce retour comme incident observé, pas comme vérité universelle. Ideas Hub rattache la correction au besoin initial. Le workflow 1.1 reçoit un nouveau cas de test. La version 1.0 déjà installée ne se transforme pas en silence : Factory préparera une 1.0.1, les nœuds compareront les digests et décideront de la mise à jour.

À 18 h 15, un atelier consacré aux vélos découvre le projet. Il n’a presque aucun manuel en commun avec Nora, mais il peut réutiliser l’application, le schéma de citation, le workflow de traduction et les tests d’accessibilité. Il crée un fork dont la lignée reste visible.

La journée se termine avec quatorze procédures, pas avec une bibliothèque universelle. Pourtant, le résultat est plus grand qu’une petite application. Le besoin, les choix, le workflow, les preuves, les refus et les moyens de reprise forment désormais un commun réutilisable.

Nexus n’a pas remplacé l’expérience des réparateurs. Il a évité de la dissoudre dans des fichiers introuvables ou des réponses de modèle sans source. Il a permis à des humains, des agents et des workers de coopérer tout en conservant les responsabilités : Ideas Hub a accueilli le besoin, RRV a retrouvé et qualifié les briques, Conception a spécifié, Workflow Hub a versionné la chaîne, Factory a réalisé et fermé, SBFB a distribué, et l’usage a rouvert le cycle avec une preuve nouvelle.

Nous avons vu toute la boucle tenir dans une journée. Pour comprendre ce qui
l’empêche de devenir une simple chaîne d’automatisation, il faut maintenant
revenir à son commencement : le lieu où une intuition apprend à résister à
l’enthousiasme, à rencontrer des objections et parfois à renoncer.
