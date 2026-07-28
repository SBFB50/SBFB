# Chapitre 16 — De la difficulté locale au commun réutilisable

Tout commence par une remarque ordinaire.

Au cours d’une semaine de forte chaleur, Samira accompagne son père à un
rendez-vous. Sur le chemin du retour, ils cherchent un endroit frais où
attendre le bus. La médiathèque est ouverte, mais rien ne l’indique depuis la
rue. Une fontaine publique se trouve à quelques centaines de mètres ; son
accès est barré par des travaux. Le commerce qui acceptait autrefois
d’accueillir les personnes fragiles a changé de propriétaire.

Samira connaît plusieurs habitants qui possèdent chacun un fragment
d’information, mais aucun lieu où les réunir sans exposer la vie des personnes
concernées.

Le chapitre 3 suivait une fabrication concentrée en une journée. Ici, le temps
est différent. Le commun va se former pendant plusieurs mois, au rythme des
disponibilités, des erreurs du terrain, des saisons et de sa reprise par
d’autres.

## Semaine 1 — Commencer par la difficulté, pas par l’application

Le soir, elle ouvre l’Atelier des communs. Elle ne propose pas encore une
application. Elle décrit un problème : « Dans le quartier, il est difficile de
savoir où trouver gratuitement un lieu frais, de l’eau et une assise
accessible pendant les épisodes de chaleur. »

Le système l’aide à distinguer l’observation, les hypothèses et la solution
imaginée. Samira peut dicter son texte. Elle choisit de publier son prénom, mais
elle aurait pu contribuer sous une identité communautaire. Elle ne saisit ni
le nom de son père ni son état de santé : ces informations ne sont pas
nécessaires.

Avant de créer une nouvelle idée, l’Atelier interroge RRV. Trois initiatives
proches apparaissent. L’une est un simple document produit dans une autre
ville. La deuxième est une carte ancienne dont plusieurs horaires ne sont plus
valides. La troisième décrit un workflow de vérification saisonnière. RRV ne
proclame pas que ces documents sont vrais. Il montre leur provenance, leur
date, les versions disponibles, les sources citées et les objections
enregistrées.

Samira relie son idée à ces travaux au lieu de les copier silencieusement.
Quelques habitants la rejoignent : un bibliothécaire, une personne utilisant
un fauteuil roulant, un étudiant, une infirmière retraitée et deux membres de
collectifs différents. Certains ont une appartenance politique, d’autres
aucune. Le projet ne leur demande pas de la déclarer.

Dans l’Atelier, le groupe transforme l’intuition en proposition examinable. Le
service devra recenser uniquement des lieux dont l’ouverture peut être
publiée. Il indiquera l’accessibilité, les horaires, la présence d’eau et la
source de chaque information. Il ne suivra pas les déplacements des
utilisateurs. Il ne conservera pas de liste des personnes âgées, malades ou
isolées.

Une fonctionnalité d’accompagnement individuel, un moment envisagée, est
retirée de cette première version : elle demanderait un autre niveau de
protection et de responsabilité.

Les désaccords restent visibles. Certains souhaitent que toute information
puisse être ajoutée immédiatement. D’autres craignent qu’une adresse erronée
mette une personne en difficulté. Le compromis n’est pas masqué par une
synthèse artificiellement unanime : une contribution peut être proposée
librement, mais elle porte le statut « à vérifier » jusqu’à confirmation par
une source identifiée.

## Semaines 2 à 4 — Transformer la proposition en chemin gouverné

Lorsque la proposition devient suffisamment précise, le groupe ouvre dans
Workflow Hub un workflow intitulé « Relais de fraîcheur local ». Le Hub
n’invente pas une autorité. Il aide les auteurs à versionner un parcours :

1. recevoir une proposition de lieu ;
2. vérifier que l’adresse est destinée à être publique ;
3. demander une source pour les horaires ;
4. effectuer une revue d’accessibilité ;
5. obtenir l’accord du responsable du lieu ;
6. publier ou maintenir la proposition en attente ;
7. programmer une nouvelle vérification ;
8. enregistrer une correction ou un retrait.

Le modèle de langage suggère des étapes oubliées et reformule les consignes en
langage clair. Le compilateur Rust refuse cependant une version où la
publication pourrait s’effectuer avant l’accord requis. Il signale aussi qu’un
connecteur de messagerie demande une capacité absente. Le groupe peut modifier
le workflow ou demander explicitement cette capacité ; le modèle ne peut pas
se l’accorder lui-même.

Avant approbation, plusieurs revues sont routées. La revue de confidentialité
confirme que le service n’a pas besoin de données de santé. La revue
d’accessibilité exige une navigation complète au clavier, des libellés
compréhensibles, une vue texte équivalente à la carte et la possibilité de
signaler une erreur sans pointer avec précision sur un écran. Une revue
opérationnelle demande qui répondra lorsqu’un lieu fermera.

Ces vérifications ne garantissent pas que le projet sera utile. Elles rendent
ses hypothèses et ses responsabilités plus difficiles à ignorer.

## Mois 2 — Fabriquer, puis contredire le premier résultat

Factory reçoit ensuite le pack approuvé. À partir du cahier des charges et des
composants autorisés, elle produit une première application. Celle-ci
fonctionne sur téléphone et ordinateur, mais aussi sous la forme d’une liste
légère. Elle peut être distribuée entre les nœuds qui choisissent de
l’héberger.

Le bridge n’accorde que les opérations nécessaires. L’application peut lire le
jeu de lieux publié et proposer une correction ; elle n’accède ni au carnet
d’adresses, ni à une base militante, ni à la position permanente de
l’utilisateur.

La version issue de Factory n’est pas publiée automatiquement parce que son
build réussit. Des humains la testent. La personne en fauteuil constate qu’une
catégorie « accessible » est trop vague. Le groupe sépare l’accès sans marche,
la largeur de passage, les toilettes accessibles et la disponibilité d’une
assise. Une personne aveugle découvre que l’ordre de lecture de la carte est
incohérent ; la vue est corrigée.

Le workflow et l’application reçoivent chacun une nouvelle version gouvernée.
Factory elle-même pourra évoluer lors d’un cycle ultérieur approuvé, mais elle
ne se réécrit pas librement pendant qu’elle fabrique cette release.

La publication produit plusieurs artefacts : l’application, le manifeste de
permissions, le workflow, les résultats des revues, les traductions et une
Proof Card. Les auteurs et responsables signent ce qu’ils déclarent avoir
vérifié. Cela permet de savoir qui a approuvé quelle version. Cela ne prouve
pas qu’un lieu sera effectivement ouvert demain.

L’interface l’exprime sans ambiguïté : « horaire confirmé par la médiathèque à
telle date » est une provenance ; « ce lieu sera disponible lorsque vous
arriverez » serait une promesse que le système ne peut pas garantir.

## Mois 3 — Laisser le réel corriger le projet

Le service est installé par quelques nœuds communautaires. Aucun n’est « le
nœud » d’un mouvement politique. Une maison de quartier, une association et
deux habitants hébergent les contenus publics. Les groupes qui ont contribué
peuvent faire connaître l’outil, mais ils ne reçoivent pas la liste de ceux qui
le consultent.

Les statistiques nécessaires sont agrégées et minimisées : nombre de fiches
corrigées, délai moyen de vérification, proportion de lieux revus récemment,
problèmes d’accessibilité signalés. Le projet refuse de mesurer l’opinion ou
l’appartenance des utilisateurs.

Le réel commence alors son travail de contradiction. Un local associatif ferme
pendant deux semaines sans mettre à jour son annonce. Une personne signale
l’erreur. Le workflow suspend la fiche, demande une confirmation et publie la
correction. Ailleurs, une fontaine est inaccessible malgré une donnée
municipale qui la disait disponible.

Les deux informations sont conservées avec leur origine et leur date. RRV
permet de constater la divergence ; il ne choisit pas magiquement laquelle est
vraie. Une vérification humaine tranche pour la version opérationnelle, et la
raison de ce choix reste consultable.

## Mois 6 — Transmettre des capacités plutôt que des fichiers de personnes

Quelques mois plus tard, un groupe d’une autre région souhaite reprendre le
projet. Il ne reçoit pas le fichier des utilisateurs du premier quartier,
puisqu’un tel fichier n’existe pas. Il installe un ensemble plus sobre et plus
précieux : le workflow versionné, le schéma des lieux, les composants
accessibles, les tests, les traductions, les règles de minimisation, les
incidents déjà rencontrés et l’historique des décisions.

Ce nouveau groupe ajoute une information sur les transports adaptés et traduit
l’interface. Il découvre que le mot « fraîcheur » ne convient pas à tous les
contextes et propose un vocabulaire paramétrable. Il forke la version,
expérimente, puis soumet les améliorations générales au projet d’origine. Les
données locales restent locales ; les apprentissages deviennent communs.

C’est ainsi qu’une idée devient un commun dans Nexus. Non parce qu’elle est
centralisée, déclarée universelle ou signée par une autorité prestigieuse. Elle
devient un commun parce que d’autres peuvent la comprendre, retrouver ses
hypothèses, la réutiliser sans demander une permission arbitraire, l’adapter
sans effacer sa filiation et rendre à leur tour leurs améliorations
disponibles.

Le commun n’est pas l’uniformité. Une ville peut exiger une validation
institutionnelle ; un village peut s’appuyer sur deux responsables identifiés ;
une communauté peut préférer ne publier que des sources officielles ; une
autre peut accepter les observations citoyennes en les étiquetant clairement.
Le protocole n’abolit pas ces choix. Il leur donne une forme explicite et
portable.

À la fin, Samira n’a pas créé une « solution nationale ». Elle a contribué à
résoudre un problème situé, avec assez de soin pour que d’autres n’aient pas à
repartir de zéro. Nexus n’a remplacé ni les lieux d’accueil, ni les personnes
qui répondent, ni les institutions responsables de l’espace public. Il a rendu
leur coopération plus transmissible.

Une infrastructure humaniste ne mesure donc pas sa réussite au nombre de
personnes qu’elle enferme dans son réseau. Elle la mesure à la quantité de
capacités qu’elle rend aux communautés : comprendre, décider, fabriquer,
vérifier, contester, transmettre et partir avec ce qu’elles ont légitimement
construit.
