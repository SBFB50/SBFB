# Partie — Confiance, pouvoir et limites

Un réseau décentralisé ne devient pas digne de confiance simplement parce qu’il n’a pas de serveur central. Il peut déplacer le pouvoir sans le rendre visible. Il peut remplacer une administration identifiable par quelques mainteneurs, quelques clés cryptographiques ou un algorithme de classement que personne ne comprend. Il peut aussi donner une apparence d’objectivité à des décisions profondément humaines.

Nexus V1 part donc d’une question plus exigeante : non pas « comment supprimer la confiance ? », mais « comment savoir à qui, à quoi et jusqu’où nous faisons confiance ? »

La réponse tient dans trois choix. Le premier consiste à produire des preuves modestes mais vérifiables : l’origine d’un objet, son intégrité, sa version, les pouvoirs demandés et les effets observés. Le deuxième consiste à répartir l’autorité entre plusieurs acteurs et à préserver la possibilité de choisir, de contester, d’exporter ou de bifurquer. Le troisième consiste à refuser certaines fonctions, même lorsqu’elles seraient techniquement séduisantes.

Ces choix ne font pas disparaître les erreurs, les abus ou les conflits. Ils cherchent à empêcher qu’une erreur locale devienne silencieusement une autorité globale.

## Chapitre 1 — Faire confiance sans fermer les yeux

Dans la vie courante, le mot « confiance » recouvre des choses très différentes. On peut faire confiance à une personne pour tenir une promesse, à un logiciel pour ne pas modifier un fichier, à une organisation pour appliquer ses propres règles ou à une source pour décrire correctement un événement. Confondre ces formes de confiance produit beaucoup de malentendus.

Nexus V1 les sépare.

Une signature cryptographique peut montrer qu’un document a été publié par une certaine identité. Une empreinte peut montrer que le document téléchargé est identique à celui qui a été annoncé. Un reçu d’exécution peut montrer qu’une activité a été demandée, autorisée puis observée. Aucun de ces éléments ne prouve, à lui seul, que le contenu est vrai, que son auteur est bienveillant ou que le résultat est utile.

Cette distinction est le point de départ de l’architecture de confiance.

### Une chaîne de provenance, pas un certificat de vérité

Lorsqu’une application, une idée documentée ou un workflow est publié dans Nexus, il reçoit une identité liée à son contenu. Si un seul caractère change, cette identité change également. L’auteur signe l’objet publié et la version annoncée. Il devient alors possible de répondre à plusieurs questions simples :

- qui a signé cet objet ;
- quel contenu exact a été signé ;
- quelle version a été installée ou exécutée ;
- si le contenu reçu a été modifié depuis sa publication ;
- quels objets antérieurs ou quelles sources sont cités.

RRV rend ces éléments lisibles. Il ne se contente pas d’afficher un badge abstrait. Il présente une carte de preuve : provenance déclarée, signatures disponibles, liens vers les sources, historique des versions et limites connues. L’utilisateur peut ainsi distinguer un document anonyme, une publication signée par une identité nouvelle, un travail repris par plusieurs curateurs ou une version dont la construction a été vérifiée par des tiers.

La progression de la confiance reste volontairement graduelle. Une identité stable vaut mieux qu’un contenu sans origine, mais elle ne vaut pas une vérification indépendante. Une source publiée vaut mieux qu’une affirmation sans référence, mais elle ne garantit pas la qualité de l’interprétation. Plusieurs attestations concordantes renforcent un dossier, mais elles peuvent encore partager la même erreur.

Nexus ne transforme donc jamais la provenance en vérité automatique. Il rend seulement le chemin de l’affirmation plus visible.

Cette modestie est essentielle dans Idea Hub. Une proposition populaire n’y devient pas vraie parce qu’elle recueille beaucoup de soutien. Elle devient visible comme proposition soutenue par certaines personnes, accompagnée de certains arguments et reliée à certaines sources. Les objections, variantes et incertitudes peuvent rester attachées au même dossier. RRV permet ensuite de rechercher cet ensemble sans réduire le débat à un score unique.

### Des pouvoirs exprimés avant l’action

La provenance répond à la question « d’où vient cet objet ? ». Les capacités répondent à une autre question : « qu’a-t-il le droit de faire ici ? »

Une application ou un workflow n’obtient pas de pouvoirs simplement parce qu’il a été installé. Il déclare les capacités dont il a besoin : lire un dossier déterminé, appeler un service précis, publier un artefact, utiliser un modèle local, envoyer un message vers une destination connue. Cette déclaration n’est qu’une demande.

Le pouvoir réellement disponible résulte de l’intersection de plusieurs limites :

1. ce que le pack a déclaré ;
2. ce que l’utilisateur ou l’organisation a accordé ;
3. ce que la politique locale autorise ;
4. ce que le runtime sait exécuter en sécurité ;
5. les contraintes propres à l’activité, comme une destination, un budget ou une durée.

Si l’une de ces limites manque, l’effet n’est pas autorisé. Une formulation générale comme « accès au réseau » ne donne pas un droit illimité de contacter n’importe quel service. Une capacité est typée et bornée. Elle décrit autant que possible le verbe, la ressource, la destination et le contexte.

Imaginons un workflow destiné à organiser une collecte solidaire. Il peut avoir besoin de lire un répertoire contenant des listes de matériel, de produire une synthèse et de publier cette synthèse dans l’espace du projet. Cela ne lui donne pas le droit de lire les carnets d’adresses personnels, d’envoyer des messages aux participants ni de publier sur le réseau public. Chacun de ces effets nécessite une capacité distincte.

Le workflow peut être parfaitement légitime et néanmoins demander trop de pouvoirs. L’interface doit alors permettre de comprendre le désaccord : « cette version demande l’accès à telle destination, mais votre politique locale ne l’accorde pas ». Le refus n’est pas un plantage mystérieux ; il fait partie du fonctionnement normal.

### Trois autorités qui ne se remplacent pas

Dans Workflow Hub, l’intelligence artificielle peut aider à décrire une intention, proposer un graphe d’activités, reformuler une consigne ou signaler une incohérence. Elle n’est pas l’autorité d’exécution.

La V1 répartit les rôles :

- le modèle compose et explique ;
- le compilateur Rust vérifie les contrats et les invariants ;
- l’humain consent lorsque l’effet engage une ressource, une personne ou une responsabilité sensible.

Cette séparation évite de demander au même système d’imaginer une action, de s’accorder le droit de l’accomplir et de déclarer ensuite qu’elle s’est bien passée. Un modèle peut proposer l’envoi d’un message ; il ne peut ni ajouter silencieusement cette capacité au workflow, ni choisir seul une nouvelle liste de destinataires, ni approuver sa propre demande.

Le compilateur transforme le graphe édité dans Workflow Hub en plan déterminé et versionné. Il vérifie notamment que les références existent, que les types sont compatibles, que les capacités demandées sont exprimables et que les points de consentement sont placés avant les effets concernés. Le moteur exécute ensuite ce plan. Il ne suit pas une improvisation textuelle permanente.

L’humain, enfin, ne doit pas être réduit à un bouton décoratif. Une demande d’approbation utile indique l’action prévue, la destination, les données concernées, le coût ou le budget, ainsi que les conséquences possibles d’un refus. Le consentement porte sur un effet compréhensible, pas sur une promesse générale de « laisser l’IA travailler ».

### Des secrets absents des objets partagés

Un pack portable doit pouvoir circuler sans transporter les secrets de son auteur. Les clés d’API, jetons, mots de passe et identifiants privés ne sont donc pas intégrés au workflow publié. Le pack déclare seulement qu’un secret d’un certain type sera nécessaire au moment de l’exécution.

Le secret est résolu localement, dans le contexte de la personne ou de l’organisation qui lance le workflow. Il n’est fourni qu’à l’activité autorisée, pour la durée nécessaire. Le journal peut indiquer qu’un secret a été utilisé sans en révéler la valeur.

Cette séparation permet à deux communautés d’exécuter le même workflow avec leurs propres comptes et leurs propres politiques. Elle évite aussi qu’un fork ou une simple inspection du pack expose des identifiants qui n’auraient jamais dû voyager.

### Des reçus honnêtes, y compris lorsque le résultat est inconnu

Une exécution distribuée ne peut pas toujours savoir ce qui s’est produit. Une requête peut avoir quitté la machine, puis la connexion être interrompue avant la réponse. Relancer automatiquement l’activité pourrait produire l’effet deux fois ; la déclarer échouée pourrait être faux.

Nexus V1 conserve donc un état explicite : le résultat est inconnu.

Ce choix paraît moins confortable qu’un voyant vert ou rouge, mais il est plus sûr. Le moteur enregistre ce qu’il sait : l’autorisation a été accordée, la tentative a commencé, une réponse n’a pas été reçue. Il peut ensuite demander une vérification humaine, utiliser une clé d’idempotence ou appliquer une procédure propre à l’activité. Il ne fabrique pas une certitude pour rendre l’interface plus rassurante.

Les reçus, événements et artefacts forment ainsi une mémoire vérifiable de l’exécution. Ils ne sont pas une surveillance générale de l’utilisateur. Ils sont limités au run, à ses effets et à ses besoins d’explication.

L’architecture de confiance de Nexus repose finalement sur une idée simple : une bonne preuve répond précisément à une question limitée. Dès qu’elle prétend répondre à toutes les questions, elle devient une nouvelle forme d’autorité opaque.

## Chapitre 2 — Gouverner sans fabriquer un nouveau centre

La décentralisation technique ne suffit pas à produire une gouvernance pluraliste. Un réseau peut être distribué entre des milliers de machines et rester contrôlé par une seule équipe qui possède la marque, les clés de publication, le classement par défaut et la capacité de décider quelles versions sont « officielles ».

Nexus V1 cherche donc à distribuer non seulement les fichiers, mais aussi les chemins par lesquels une décision acquiert de l’autorité.

### Plusieurs espaces, plusieurs légitimités

Une communauté peut adopter une charte, choisir des curateurs, maintenir une collection d’applications et définir ses propres règles de publication. Une autre communauté peut utiliser le même protocole avec des choix différents. Aucun de ces espaces n’est automatiquement « le réseau tout entier ».

Cette distinction protège le pluralisme. Elle permet à un collectif de maintenir une exigence éditoriale forte sans imposer cette exigence à tous les autres. Elle permet aussi aux utilisateurs de quitter une gouvernance devenue insatisfaisante sans perdre leurs objets, leur historique ou la possibilité de continuer le travail ailleurs.

RRV reflète cette pluralité. Il n’existe pas un classement universel qui déterminerait ce que chacun doit voir. Les résultats peuvent être organisés selon des sources choisies, des curateurs suivis, des critères explicites ou des espaces communautaires. L’utilisateur doit pouvoir savoir pourquoi un résultat apparaît et changer de perspective.

Cela ne signifie pas que tous les classements se valent. Certains seront plus documentés, plus ouverts ou plus résistants aux manipulations. Mais leur autorité vient de règles visibles et d’une confiance accordée, pas du simple fait qu’ils sont le réglage caché de la plateforme.

### Idea Hub n’est pas une machine à fabriquer une majorité

Idea Hub facilite l’émergence et le travail des propositions. Il peut recueillir des soutiens, des objections, des besoins, des compétences et des expériences de terrain. Il ne doit pas transformer chaque question complexe en concours de popularité.

Une mobilisation nombreuse peut légitimement montrer qu’un sujet compte pour beaucoup de personnes. Elle ne doit pas effacer les minorités concernées, les contraintes matérielles ou les connaissances contradictoires. La gouvernance d’une idée peut donc combiner plusieurs signaux : soutien, diversité des contributeurs, qualité des sources, faisabilité, effets sur les personnes concernées et existence d’alternatives.

Les protections contre la capture ne consistent pas seulement à bloquer des robots. Elles ralentissent aussi les emballements humains : comptes nouveaux aux pouvoirs progressifs, limites de rythme, historique des modifications, séparation entre proposition et décision, période de discussion, possibilité de faire appel et conservation des désaccords.

Une idée rejetée ne disparaît pas nécessairement. Elle peut être archivée, forkée ou reprise dans un autre espace. Une idée acceptée n’obtient pas pour autant tous les moyens d’agir. Elle doit encore passer par les capacités, les budgets et les consentements du contexte où elle sera réalisée.

### La version comme outil de gouvernance

Dans Nexus, une version n’est pas seulement un numéro technique. Elle matérialise un accord.

Workflow Hub permet de travailler librement sur un brouillon, de tester des variantes et de discuter une modification. Lorsque le workflow est prêt, une version est compilée, examinée puis approuvée. Les exécutions qui la citent savent alors exactement quel plan elles utilisent.

Factory suit la même logique. Il peut évoluer, améliorer ses méthodes ou intégrer de nouvelles étapes entre deux cycles. Une équipe peut forker son processus, le modifier, le tester et proposer une nouvelle version. Elle ne remplace pas silencieusement le processus qui est déjà en train de produire une application.

Cette discipline donne une forme concrète à la responsabilité. Si une décision produit un mauvais résultat, il est possible de retrouver la version, les règles et les approbations correspondantes. Si une amélioration est proposée, elle peut être comparée à l’état précédent. Le désaccord ne porte pas sur un système mouvant dont personne ne peut reconstituer le comportement.

### Répartir les points de pouvoir

La V1 considère plusieurs formes de pouvoir séparément :

- écrire le protocole ;
- maintenir une implémentation ;
- signer une release ;
- distribuer une application ;
- recommander un contenu ;
- administrer un espace ;
- modérer une publication ;
- accorder une capacité locale ;
- approuver un effet sensible.

Réunir tous ces pouvoirs dans les mêmes mains serait pratique, mais fragile. Nexus les rend donc distincts autant que possible.

Une équipe peut maintenir l’implémentation de référence sans contrôler toutes les collections distribuées. Un curateur peut recommander une application sans pouvoir lui accorder des droits sur la machine de l’utilisateur. Un administrateur communautaire peut modérer son espace sans effacer l’objet des autres espaces. Une release peut être signée et vérifiée par plusieurs responsables sans donner à chacun un accès quotidien aux secrets de production.

La gouvernance du projet lui-même suit cette logique. Les changements importants sont documentés ; les conflits d’intérêts sont déclarés ; les décisions qui modifient les frontières de sécurité ou les formats durables demandent davantage de revue qu’un ajustement ordinaire. Les clés critiques ne reposent pas sur une seule personne. Les règles de compatibilité et de migration sont publiques.

Il reste toujours des centres de gravité. L’implémentation la plus utilisée influence le protocole. Une interface populaire façonne les habitudes. Un financeur important peut peser sur les priorités même sans droit formel. La gouvernance pluraliste ne nie pas ces forces ; elle cherche à les rendre discutables et réversibles.

### Le droit pratique de partir

Une gouvernance n’est réellement limitée que si l’on peut la quitter.

Dans Nexus V1, cette possibilité passe par des mécanismes concrets : export des données, formats documentés, objets adressés par leur contenu, historique portable, possibilité de changer de curateur et droit de forker un workflow ou une application selon sa licence. Une communauté peut migrer sans demander la permission à l’organisation qu’elle quitte.

La sortie n’efface pas les obligations prises, les droits des autres auteurs ni les limites de confidentialité. Elle empêche simplement qu’un désaccord politique, éditorial ou technique se transforme en captivité.

Cette propriété modifie aussi la qualité du pouvoir. Une équipe qui sait que les utilisateurs peuvent réellement partir doit convaincre davantage et imposer moins. Le fork n’est pas souhaitable pour chaque désaccord, mais sa possibilité protège la négociation.

### Une infrastructure ouverte aux personnes, pas appropriée par une appartenance

Nexus peut accueillir des personnes venues d’associations, de mouvements, de métiers ou de traditions très différentes. Leur expérience et leur capacité d’engagement sont précieuses. Elles participent comme personnes ou comme communautés identifiées, selon les mêmes règles publiques que les autres.

Aucune appartenance extérieure ne donne un droit particulier sur le protocole. Inversement, elle ne constitue pas une raison de soupçon ou d’exclusion. Ce qui compte dans l’espace commun est le comportement observable : respect des capacités, transparence des sources, absence d’usurpation, possibilité de contradiction et conformité aux règles de l’espace choisi.

Cette neutralité n’est pas une absence de valeurs. Nexus V1 privilégie la dignité des personnes, leur autonomie, l’accessibilité, la coopération volontaire et la possibilité de contester le pouvoir. Mais ces valeurs sont inscrites dans des garanties communes, pas confondues avec l’autorité d’une organisation particulière.

La gouvernance pluraliste demande finalement plus de travail qu’un centre unique. Elle produit des désaccords visibles, des procédures et parfois des bifurcations. C’est son coût, mais aussi son rôle : empêcher l’efficacité immédiate de devenir une domination durable.

## Chapitre 3 — Les refus qui protègent le projet

Une plateforme se définit autant par ce qu’elle rend possible que par ce qu’elle refuse de devenir. Les limites de Nexus V1 ne sont pas des fonctionnalités en attente. Certaines constituent des choix de conception destinés à préserver l’autonomie des utilisateurs.

Elles pourront être discutées et améliorées, mais elles ne doivent pas être contournées sous prétexte qu’un modèle plus puissant, une communauté plus nombreuse ou une situation urgente promettrait de « mieux savoir » à la place des personnes.

### Nexus ne produit pas de vérité globale

RRV recherche des sources, relie des objets, expose des signatures et permet de suivre des curateurs. Il ne maintient pas un registre central du vrai et du faux.

Une telle vérité globale demanderait une autorité finale : une institution, une majorité, un algorithme ou une IA capable de trancher pour tous. Même bien intentionnée, cette autorité serait vulnérable aux erreurs partagées, aux changements de contexte et à la capture.

La V1 préfère représenter plusieurs niveaux : fait déclaré, document source, interprétation, contestation, provenance et degré de vérification. Une affirmation peut être solidement documentée sans devenir incontestable. Une source peut être authentique tout en étant incomplète. Une communauté peut publier une conclusion et une autre conserver un désaccord argumenté.

Nexus aide à construire un jugement ; il ne remplace pas le jugement.

### Nexus ne construit pas une infrastructure de surveillance

Le fonctionnement d’une communauté produit de nombreuses traces : recherches, lectures, soutiens, contributions, exécutions, relations entre personnes et lieux de connexion. Agrégées, ces traces peuvent révéler bien davantage que ce que chacun pensait partager.

La V1 refuse donc le graphe comportemental universel. Elle ne cherche pas à connaître l’ensemble des lectures, des relations ou des déplacements d’un utilisateur pour personnaliser son expérience. Les journaux techniques sont limités à une finalité précise, conservés pour une durée maîtrisée et placés sous le contrôle du contexte qui les produit.

Les preuves d’exécution ne deviennent pas un dossier général sur la personne. Les statistiques utiles sont agrégées lorsque cela suffit. Les espaces privés ne publient pas leurs listes de membres. Une identité peut prouver un droit sans exposer toutes les communautés auxquelles elle participe.

Le P2P n’est pas une excuse pour diffuser davantage. Au contraire, puisque la réplication complique le retrait, la décision de publier doit être plus stricte.

### Nexus ne fait pas de microciblage politique ou psychologique

Il peut être tentant d’utiliser Idea Hub et RRV pour déduire les convictions, les fragilités ou les sujets capables de persuader chaque personne. Workflow Hub pourrait ensuite adapter automatiquement les messages, et Factory déployer les interfaces correspondantes.

Nexus V1 refuse cette chaîne.

Les préférences exprimées pour participer à un projet ne servent pas à fabriquer un profil psychologique secret. Les données d’une communauté ne sont pas croisées avec celles d’une autre pour deviner des opinions. Un message collectif ne change pas silencieusement d’argument selon les vulnérabilités supposées de son destinataire.

Cette limite n’interdit pas l’accessibilité, la traduction, le choix d’une langue ou l’adaptation volontaire d’une interface. Elle distingue l’aide demandée par l’utilisateur de la persuasion cachée exercée sur lui.

Une communication peut être adressée à un groupe qui a choisi de la recevoir. Sa source, sa finalité et son responsable restent visibles. L’optimisation opaque de l’influence individuelle n’est pas une capacité de la V1.

### L’IA n’est souveraine sur aucun effet

Une IA peut produire une réponse impressionnante et néanmoins se tromper sur un nom, une destination, une règle ou le sens d’une demande. Elle peut aussi être influencée par un contenu hostile présent dans les documents qu’elle lit.

La V1 ne lui permet donc pas :

- de s’accorder une nouvelle capacité ;
- d’étendre seule le graphe d’un workflow validé ;
- de choisir une destination sensible non prévue ;
- d’augmenter un budget ;
- de résoudre ou révéler un secret hors du contexte autorisé ;
- d’approuver sa propre action ;
- de déclarer qu’un effet a réussi sans preuve suffisante.

Le modèle reste un composant de composition et d’assistance. Les contrats typés, le runtime et le consentement humain constituent l’autorité. Même un modèle local, exécuté sur la machine de l’utilisateur, ne devient pas souverain : la souveraineté locale appartient à la personne, pas au logiciel qui tourne près d’elle.

### Un workflow ne se réécrit pas pendant qu’il agit

La capacité d’un agent à adapter son plan semble attractive. Elle devient dangereuse lorsqu’elle permet de modifier en cours d’exécution les étapes, les destinataires ou les permissions sur lesquelles l’utilisateur avait fondé son consentement.

Nexus V1 fige donc le plan compilé d’un run. Les données et certains choix prévus peuvent varier ; la structure autorisée du workflow ne se transforme pas silencieusement.

Si une meilleure stratégie apparaît, elle produit un nouveau brouillon. Celui-ci peut être comparé, validé et promu comme nouvelle version pour une exécution suivante. En cas d’urgence, le run peut être arrêté, puis remplacé explicitement. Il n’existe pas de raccourci où l’agent devient son propre auteur, son propre contrôleur et son propre approbateur.

Factory évolue de la même manière : fork, modification, test, approbation, nouvelle version. Sa liberté vient de sa capacité à évoluer entre les cycles, non d’une mutation incontrôlée au milieu d’un travail.

### Les données sensibles ne vont pas sur le réseau public

Le chiffrement est indispensable, mais il ne rend pas toute publication raisonnable. Un objet chiffré et largement répliqué peut rester disponible pendant des années. Ses métadonnées, sa taille, ses moments d’accès ou ses relations avec d’autres objets peuvent déjà révéler des informations.

La V1 refuse donc de placer sur le réseau public les secrets, données d’adhésion, carnets d’adresses, dossiers personnels, conversations confidentielles, prompts sensibles ou artefacts permettant d’identifier des personnes vulnérables.

Le réseau public transporte des objets destinés à être publics : applications, packs, documents publiés, preuves minimisées et annonces nécessaires à leur découverte. Les espaces privés utilisent des canaux, des capacités et des politiques distincts. Lorsque seule une preuve d’existence est nécessaire, Nexus publie une empreinte ou un engagement approprié plutôt que la donnée elle-même.

La règle reste compréhensible sans expertise cryptographique : si la diffusion durable de l’objet pourrait nuire à une personne, cet objet n’appartient pas au réseau public.

### Une limite n’est pas une promesse magique

Ces refus réduisent le pouvoir du système ; ils ne rendent pas les personnes infaillibles. Un auteur peut publier une information fausse sous son vrai nom. Une communauté peut choisir de mauvais curateurs. Un administrateur peut abuser de ses droits locaux. Un utilisateur peut accorder une capacité qu’il n’a pas comprise. Un logiciel peut contenir une faille.

La V1 répond à ces risques par plusieurs couches : minimisation des pouvoirs, provenance, isolation, versions, reçus, possibilité d’appel, pluralité des sources et droit de partir. Aucune couche n’est parfaite. Leur combinaison cherche à éviter qu’une seule erreur suffise à tout compromettre.

Cette honnêteté est peut-être le refus le plus important. Nexus ne promet ni une société sans conflit, ni une confiance entièrement automatisée, ni une intelligence artificielle capable de gouverner à notre place.

Il propose une infrastructure où l’on peut coopérer à grande échelle tout en continuant à demander : qui parle, sur quelle base, avec quel pouvoir, sous quelle version, et comment puis-je dire non ?
