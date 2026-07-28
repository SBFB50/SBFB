# Chapitre 9 — Le nœud, les applications et le calcul partagé

Nexus ne commence pas par un compte hébergé sur un service central. Il commence par un nœud local. Ce nœud donne à la personne une identité technique, un espace de stockage, une interface, un cache d’applications, un runtime de workflows et, si elle le souhaite, des capacités de calcul et les outils Factory.

Le nœud constitue le centre de gravité de l’expérience. Le réseau étend ses possibilités ; il ne remplace pas son autorité locale.

## Ce qui vit sur la machine

Le daemon est le service local qui parle au réseau, conserve les métadonnées, vérifie les signatures, gère les blobs et expose une API en boucle locale. Le shell est l’interface de confiance. Il affiche les applications, les sources, les permissions, les notifications, les demandes de consentement et les états du nœud.

Les clés privées, les secrets et les décisions privilégiées restent du côté de cet hôte. Une application ouverte dans Nexus ne reçoit ni la clé du nœud, ni un accès libre au système de fichiers, ni le bearer du daemon.

Hors ligne, le nœud peut ouvrir une application déjà récupérée, consulter ses données ou préparer un brouillon. Lorsque la connexion revient, il reprend les échanges autorisés et vérifie les changements. « Local-first » signifie fonctionner sans dépendre à chaque instant d’un serveur distant ; cela ne signifie pas vivre toujours isolé.

## Des applications statiques dans un espace borné

Une application SBFB est principalement une archive web statique : HTML, CSS, JavaScript et autres ressources nécessaires. Le shell l’ouvre dans une iframe sandboxée avec une politique de contenu stricte. L’application n’établit pas directement des connexions arbitraires vers Internet. Elle dialogue avec l’hôte par un bridge typé.

Le bridge transforme « lire cette clé », « rechercher » ou « soumettre une
tâche » en opération contrôlée. Le shell présente la demande dans une surface
de confiance, résout les secrets et recueille les grants ou confirmations
humaines. Le daemon authentifie l’application et sa version, valide les entrées
typées et recalcule, à chaque tentative, l’intersection entre capacités
demandées, politique locale et grants. Avec le runtime, il exécute,
journalise et produit les reçus faisant autorité.

Ce modèle est moins permissif qu’un navigateur généraliste et moins puissant qu’un programme natif. C’est volontaire. Une application de lecture, une carte locale, un tableau de données ou un atelier d’idées n’a pas besoin d’obtenir le contrôle de la machine pour être utile.

Les opérations réellement privilégiées restent dans des outils locaux séparés. Factory Operator, par exemple, peut travailler dans un dépôt, lancer des tests et préparer un commit. Il n’est pas injecté dans l’iframe d’une application du réseau.

## Iroh : transporter sans créer une autorité centrale

Iroh fournit plusieurs primitives réseau complémentaires.

Les **blobs** transportent les contenus immuables : archives d’applications, packs, artefacts et pièces de preuve. Leur adresse dépend de leur contenu. Si un seul octet change, l’adresse change. Un nœud peut donc vérifier ce qu’il reçoit sans faire confiance à la machine qui le lui transmet.

Le **gossip** et les feeds diffusent des annonces et des événements signés : une version a été publiée, une source est devenue obsolète, un curateur recommande un projet, un artefact est disponible. Ces messages rendent les changements découvrables, mais une annonce n’est pas le contenu annoncé.

Les **docs** servent aux états partagés qui doivent évoluer, par exemple un espace applicatif répliqué entre un ensemble de nœuds. Le partage est explicite et lié à une namespace. Toutes les données de toutes les applications ne sont pas placées dans un document global.

Iroh essaie d’établir des échanges directs entre pairs et peut utiliser un relais de transport lorsque la topologie du réseau l’exige. Un relais aide les octets à circuler ; il ne devient ni l’autorité de publication ni le propriétaire logique des données.

Nexus n’est ni une blockchain à consensus mondial, ni une base distribuée unique où tous verraient instantanément le même état. Il combine des contenus immuables, des journaux signés, des espaces partagés choisis et des vues locales.

## Héberger n’est pas publier

Un nœud peut conserver et servir une archive créée par quelqu’un d’autre. Cela améliore sa disponibilité, mais ne transforme pas l’hébergeur en auteur. La publication est liée à une identité, à une signature et à une provenance ; l’hébergement signifie seulement que le nœud possède les octets et accepte de les transmettre.

Une application peut ainsi survivre à l’arrêt temporaire de son auteur sans brouiller les responsabilités. L’identité Ed25519 atteste la clé signataire, pas l’identité civile d’une personne. L’interface conserve cette nuance pour auteurs, champions et workers.

## Données locales, partagées, publiées et projetées

Pour comprendre ce qui circule, il faut distinguer quatre catégories.

1. **Les données privées locales** : préférences, secrets, brouillons, historique privé d’un run et documents non publiés. Elles restent sur le nœud, sauf export ou partage explicite.
2. **Les données partagées dans un groupe ou une application** : elles circulent dans une namespace déterminée avec des droits de lecture ou d’écriture. Leur caractère P2P ne les rend pas publiques.
3. **Les artefacts publiés** : applications, packs, versions, manifests et documents rendus accessibles par leur adresse de contenu.
4. **Les projections publiques** : résumés volontairement limités permettant la découverte et la vérification — auteur technique, version, licence, digest, provenance, statuts ou résultats sélectionnés.

RRV consomme principalement les artefacts publics et les projections autorisées. Il ne parcourt pas les bases privées des applications. Une application peut publier un schéma ou des statistiques expurgées sans publier les lignes qui les ont produites.

Une projection n’est jamais la base d’autorité originale. Elle sert à rechercher et à expliquer. Si une carte indique qu’un workflow a réussi, l’utilisateur doit pouvoir remonter, selon ses droits, au run, aux reçus et aux digests correspondants. Si ces éléments manquent, RRV indique que l’affirmation est incomplète.

## Installation, activation et évolution

Lorsqu’un nœud découvre une application ou un pack, il peut d’abord l’installer désactivé. Cette étape récupère la version et permet de l’inspecter sans lui accorder de capacités.

L’activation présente ensuite les demandes : stockage, recherche, tâche réseau, destination externe, accès à un provider ou autre effet. L’utilisateur peut accepter, refuser ou limiter. Les permissions effectives sont l’intersection entre ce que le pack demande, ce que la version déclare, la politique du nœud et le consentement accordé.

Une mise à jour possède un nouveau digest et ne remplace pas silencieusement l’ancienne. Si ses données, ses outils ou ses destinations changent, elle demande un nouveau consentement. L’utilisateur peut comparer, différer, revenir en arrière ou révoquer.

Les Proof Cards rendent ce cycle lisible. Elles réunissent les informations disponibles sur la source, la licence, le hash de l’archive, la fraîcheur, la provenance, les tests, les curations et les risques connus. Une Proof Card n’affirme pas qu’un logiciel est parfait. Elle montre ce qui a été établi et ce qui reste incertain.

Le réseau Nexus est ainsi un ensemble de vues souveraines qui peuvent coopérer. Il ne recherche pas une page d’accueil identique pour tous. Chaque nœud choisit ses sources et ses curateurs, tout en conservant la possibilité de retrouver l’origine d’un artefact et de quitter une recommandation.

## Workflow Hub écrit ; le runtime exécute

Dans Nexus, un agent n’est pas une personnalité magique placée au-dessus du système. C’est une définition versionnée et bornée : un rôle, un prompt, des schémas d’entrée et de sortie, une politique de modèle, des outils possibles, un contexte, des budgets, des timeouts et des limites d’itération.

Workflow Hub sert à écrire et versionner les workflows qui assemblent ces agents avec des opérations déterministes. L’auteur y relie des entrées, des agents, des outils, des branches, des maps, des joins, des boucles bornées, des sous-workflows et des validations humaines. L’interface peut aider à produire un diff déclaratif ; elle n’est pas l’autorité d’exécution et ne modifie pas silencieusement le workflow.

Avant le lancement, le compilateur Rust vérifie l’ensemble. Les ports doivent être compatibles. Les boucles doivent avoir une borne. Les destinations, les capacités et les budgets doivent être connus. Une sortie de modèle ne peut pas supprimer un passage humain requis.

Le résultat est un plan compilé immuable. Le runtime local en devient l’autorité d’exécution. Il journalise les événements, planifie les étapes, conserve une inbox et une outbox, crée des checkpoints et applique les politiques de retry et d’idempotence.

Le journal permet de reprendre après un crash. Une clé d’idempotence limite la répétition d’un effet externe. Si un effet non idempotent a peut-être eu lieu sans reçu fiable, le runtime déclare `outcome_unknown` au lieu d’inventer un résultat.

Le workflow peut évoluer, mais pas au milieu d’un run. Une modification produit une nouvelle version, de nouvelles évaluations et, si les demandes changent, un nouveau consentement. Le run en cours reste attaché au plan et aux digests avec lesquels il a commencé.

## Trois lieux pour le calcul

Un même workflow peut utiliser plusieurs familles de providers, à condition que sa politique l’autorise.

Le **calcul local** utilise les ressources de la machine, par exemple via Ollama ou llama.cpp. Les données ne quittent pas le nœud pour l’inférence. Le coût marginal et la confidentialité peuvent être favorables, mais la vitesse et la qualité dépendent du matériel et du modèle disponible.

Le **cloud** utilise un provider configuré par l’utilisateur. Il peut donner accès à des modèles plus puissants ou plus rapides. Il implique toutefois une sortie de données, des conditions de service, un coût et une disponibilité externes. Nexus ne transforme pas une clé API en secret contenu dans un pack publié. Le shell de confiance résout le credential au moment autorisé et n’en révèle pas la valeur au workflow ou à l’application.

Le **réseau SBFB** distribue des tâches bornées à des workers volontaires. C’est particulièrement adapté au batch : traduction de fragments publics, indexation, scan de licence, build, test, extraction de schéma ou vérification d’un artefact. La tâche décrit les entrées et les limites ; le worker produit un résultat signé accompagné des métadonnées prévues.

Le lieu vient du plan, de la politique de provider, de la classification des données, des capacités disponibles et du consentement. Un modèle peut proposer un contenu ; il ne peut pas s’accorder un provider, relever son budget ou changer de destination.

## Consentement de l’utilisateur et du fournisseur

Le consentement existe des deux côtés.

La personne qui lance un workflow voit les capacités demandées, les types de données, les destinations, le coût estimé et les étapes humaines. Une action sensible est confirmée dans le shell de confiance, pas dans une boîte de dialogue dessinée par l’application sandboxée. La confirmation est liée au digest exact de la version et à l’effet présenté.

La personne qui fournit du calcul choisit également ses limites : backend autorisé, modèles, consommation, disponibilité, types de tâches et règles de consentement. Mettre un GPU à disposition ne signifie pas accepter toute charge venue du réseau.

Les secrets constituent une catégorie séparée. Un pack peut déclarer qu’il a besoin d’un secret nommé sans contenir sa valeur. Le runtime vérifie qu’un resolver autorisé est disponible. Les logs, prompts, événements et artefacts sont expurgés selon leur politique. Une absence de secret bloque ou suspend proprement l’étape au lieu de pousser l’utilisateur à le coller dans un champ public.

## Ce que les preuves de calcul disent — et ne disent pas

Il faut distinguer deux mécanismes. Une **tâche entière** confie à un worker une
entrée bornée — par exemple un fragment public à traduire ou un artefact à
tester — puis récupère son résultat. Une **inférence shardée** répartit les
couches d’un grand modèle public entre plusieurs machines et fait circuler les
activations d’une étape à l’autre.

Une tâche peut conserver l’empreinte de ses entrées, l’identité technique du
worker, le modèle déclaré, l’empreinte de la version disponible, les
paramètres, les sorties et les reçus. Plusieurs workers peuvent être sollicités
et leurs résultats comparés. Ces éléments améliorent l’auditabilité.

Ils ne transforment pas automatiquement une réponse en vérité. Une signature prouve quel nœud a signé. Un hash prouve l’intégrité d’un contenu donné. Un test prouve qu’un comportement a été observé dans certaines conditions. Un quorum prouve que plusieurs participants ont produit des résultats compatibles selon une règle. Aucun de ces mécanismes ne garantit seul que leur interprétation du monde est juste.

Matériels et runtimes peuvent produire des divergences numériques : selon la
tâche, on compare une égalité exacte, une tolérance ou des propriétés
vérifiables. Dans une inférence shardée, le plan signé prouve seulement que
l’initiateur a composé et signé ce plan. Le `RunProof`, signé par le driver,
atteste ce que ce driver dit avoir observé ; c’est une auto-attestation non
répudiable, pas une preuve indépendante de justesse. Les niveaux de
vérification N0 à N3 augmentent graduellement l’assurance par engagements,
contrôles ponctuels, redondance ou révélation différée. Aucun ne transforme
seul le résultat en vérité.

Le sharding d’un grand modèle réduit la contrainte sur un nœud, mais augmente
latence, fragilité et exposition. En V1, les activations circulent en clair
entre les membres : une liste d’admission contrôle qui participe, pas ce qu’un
membre admis peut observer. L’inférence shardée est donc réservée aux modèles
et entrées publics ou non sensibles. Les usages sensibles restent locaux ou
dans une infrastructure de confiance non shardée.

Provider absent, worker disparu, budget atteint, source invalide ou refus humain font partie des états normaux à expliquer. Il n’existe pas de scheduler mondial déplaçant librement un run privé. L’autorité reste sur le nœud initiateur. Des feuilles de calcul peuvent être distribuées ; plan, secrets, décisions et état privé ne deviennent pas publics.

## Reconnaître sans transformer la contribution en capital

En V1, Nexus reconnaît les tâches de calcul effectivement exécutées. Il
conserve leurs reçus dans un registre public chaîné par empreintes.

Cette reconnaissance prend la forme de kudos, pas d’une monnaie. Les kudos ne
peuvent pas être achetés, revendus, mis en garantie ou convertis en droit
universel de gouverner. Ils ne donnent pas à une grande ferme de GPU une
autorité politique sur le protocole ou sur une communauté.

Le calcul des kudos compresse les écarts par des rendements logarithmiques et
fait décroître les traces anciennes par une moyenne mobile exponentielle. Le
registre restant public et lié à l’identité technique du nœud, ses
contributions peuvent être corrélées entre projets. Les kudos ne sont donc ni
un certificat moral, ni une garantie d’anonymat.

Cette retenue protège le réseau contre une trajectoire familière : commencer
par remercier les contributeurs, puis laisser l’accumulation de matériel
devenir une accumulation de pouvoir. Nexus reconnaît ce qui a été fourni ; il
ne vend pas l’autorité commune à ceux qui peuvent fournir le plus de machines.

### Au-delà de la V1 — reconnaître d’autres formes de contribution

Le stockage altruiste, le relais, la traduction, la revue d’accessibilité, la
médiation et le soin communautaire comptent eux aussi. Leur mesure expose
toutefois des risques très différents. Le système multi-familles capable de
les reconnaître sans les aplatir dans un score unique appartient à la vision
Kudos v2, après la V1. Le livre ne lui attribue donc pas les garanties du
registre de calcul déjà défini.

Enfin, l’automatisation ne résout pas le problème de la maintenance. Agents et calcul partagé abaissent le coût de certaines tâches, permettent des contrôles plus systématiques et rendent un petit groupe plus capable. Ils ne remplacent ni le porteur du besoin, ni la revue sceptique, ni les personnes concernées, ni la décision d’arrêter une solution devenue nuisible.

Cette puissance distribuée pose alors la question décisive : comment faire
confiance sans recréer un centre, et comment autoriser l’action sans abandonner
le consentement ? La partie suivante examine les mécanismes qui portent cette
charge.
