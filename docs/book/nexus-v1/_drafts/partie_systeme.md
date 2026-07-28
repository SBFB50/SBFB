# Partie — Nexus V1, un système pour transformer des besoins en communs numériques

## Chapitre 1 — De l’intuition humaine à une solution vérifiable

Nexus V1 peut être compris comme une chaîne de transformation. À une extrémité se trouve un besoin humain, souvent imprécis : une difficulté rencontrée dans un quartier, une information difficile à retrouver, une tâche répétitive, un service devenu trop dépendant d’une plateforme privée. À l’autre extrémité se trouve une application ou un workflow utilisable, distribué entre plusieurs nœuds, accompagné de son origine, de ses limites et des preuves produites pendant sa fabrication.

Entre les deux, Nexus sépare plusieurs responsabilités. Ideas Hub mûrit les besoins. RRV recherche l’existant et qualifie les preuves. Conception spécifie. Workflow Hub écrit et versionne les workflows. Factory réalise et ferme leur fabrication. Le protocole distribue les versions ; résultats, incidents et retours alimentent un nouveau cycle. Chaque outil aide, mais aucun ne décide seul de ce qui est vrai, utile ou légitime.

### Ideas Hub : commencer par le besoin, pas par la popularité

Ideas Hub est la porte d’entrée des besoins. Une personne peut y déposer une « graine » : le problème observé, les personnes concernées, le contexte local, le dommage actuel, l’autonomie recherchée et, si nécessaire, le degré de confidentialité. Une graine n’est ni un cahier des charges ni une promesse de développement.

Le Hub évite de réduire la sélection à un classement mondial de votes. Un grand nombre de clics indique une attention momentanée ; il ne prouve ni qu’une proposition est bien comprise, ni qu’elle peut être maintenue. Les appuis restent donc des signaux sociaux faibles. La vue dépend des sources et des listes de curation choisies par chaque nœud. Deux personnes peuvent voir des sélections différentes sans que l’une soit présentée comme la vérité officielle du réseau.

La maturation clarifie qui serait aidé, les données à garder locales, la plus petite intervention utile, les non-objectifs et la charge de maintenance. Une idée devient sérieuse lorsqu’une personne ou un petit groupe accepte de la porter. Le porteur annonce une prochaine action, une échéance, un critère d’abandon et une manière de transmettre. Une revue sceptique et un premier dossier de preuves comptent davantage qu’un total de votes.

### RRV : chercher dans le réel et montrer la nature des preuves

RRV, pour Recherche Réseau Vérifiable, intervient avant qu’un projet reparte de zéro. Il recherche des applications, des bibliothèques, des documents, des jeux de données, des modèles, des workflows, des licences, des incidents connus et des preuves de provenance. Il peut travailler dans plusieurs périmètres clairement affichés : l’application ouverte, les éléments connus localement, le réseau SBFB, le code d’un projet autorisé ou, si l’utilisateur le demande, des sources web externes.

RRV ne mélange pas ces périmètres. Une information lue dans un dépôt, une inférence produite localement, une signature vérifiée et une affirmation trouvée sur le Web ne reçoivent pas le même statut. Une réponse distingue au minimum :

- ce qui a été **lu** dans un fichier, un manifeste, un feed ou une archive ;
- ce qui a été **déduit**, avec le raisonnement correspondant ;
- ce qui a été **vérifié** par un hash, une signature, un test, un build ou une provenance ;
- ce qui reste **non vérifié**.

RRV est donc un consommateur de projections et de preuves. Il les indexe, les recherche, les rapproche et explique leurs limites. Il ne transforme pas une hypothèse en fait, ne lance pas silencieusement une fabrication et ne rend pas le verdict final d’un processus. Lorsqu’il propose une action — ouvrir une source, comparer deux versions, lancer un audit ou préparer un fork — cette action conserve une portée, un coût et un consentement visibles.

RRV permet ainsi au réseau d’accumuler des briques réutilisables. Composants, schémas, workflows, tests et enseignements d’une application deviennent des matériaux pour la suivante. La valeur vient de leur qualité et de leur capacité de reprise, pas du nombre brut de fichiers.

### Conception : décider ce que l’on va construire

Conception reçoit un besoin clarifié et un dossier RRV. Son rôle est de spécifier une solution, pas de l’implémenter. Elle explore plusieurs angles, compare des variantes, vérifie la faisabilité, rend visibles les désaccords et produit un `ConceptionPack`.

Ce pack contient notamment le problème retenu, les bénéficiaires, les non-objectifs, les contraintes de données, les scénarios d’usage, l’architecture proposée, les alternatives écartées, les risques, les critères d’acceptation et les décisions de conception. Il peut également demander un prototype très limité ou une vérification technique, mais il ne présente jamais une maquette comme une application terminée.

La distinction entre Conception et Factory est fondamentale. Conception répond à la question : « Que faut-il réaliser, dans quelles limites, et comment saurons-nous que le besoin est satisfait ? » Factory répond ensuite : « Comment le produire, le tester, le revoir et le publier proprement ? »

Cette séparation empêche le code généré de devenir la définition du besoin. Elle permet aussi de conclure qu’il vaut mieux réparer, forker, poursuivre la recherche ou ne rien construire.

### Workflow Hub : écrire et versionner les manières de travailler

Workflow Hub sert à créer, comprendre, tester et versionner des workflows. Un workflow décrit une suite bornée d’étapes : entrées typées, agents, outils, branches, opérations parallèles, validations humaines, budgets, sorties et conditions d’arrêt.

Le même workflow peut être présenté sous forme de canevas, de liste structurée ou de parcours guidé. Ces vues n’abritent pas trois logiques différentes : elles modifient le même modèle. Une personne qui n’utilise ni souris ni représentation visuelle peut donc accomplir les mêmes opérations qu’une personne qui préfère manipuler un graphe.

Le Hub est un auteur et un gestionnaire de versions. Il ne devient pas le moteur d’autorité. Avant une exécution, le compilateur Rust valide les schémas, les connexions, les boucles, les capacités, les destinations et les budgets, puis produit un plan compilé immuable. Pendant le run, ce plan reste figé. Ni une suggestion du modèle, ni une modification de l’interface, ni l’arrivée d’une nouvelle version ne changent la topologie ou les permissions d’une exécution déjà commencée.

Si l’on veut améliorer le workflow, on crée une nouvelle version : fork ou copie, modifications, validation, évaluations, revue, publication et consentement renouvelé. Les runs existants restent attachés à leur plan et à leurs digests. Cette règle protège la reproductibilité et évite qu’un processus sensible change de nature en cours de route.

### Factory : réaliser et fermer

Factory consomme un `ConceptionPack` approuvé et les briques trouvées par RRV. Elle crée le projet, applique les templates, prépare l’environnement, implémente, teste, analyse les secrets et les dépendances, construit une prévisualisation sandboxée, organise les revues et prépare la publication.

Factory conserve le lien entre l’intention, les décisions, le dépôt, les tests, les revues et l’artefact final. Son processus inclut préparation, implémentation bornée, revue indépendante, réconciliation, gates techniques et décision humaine avant commit ou publication.

« Fermer » signifie qu’un résultat ne peut pas se déclarer terminé tout seul. Un agent qui a écrit le code ne s’attribue pas son propre verdict. Un écran vert ne remplace pas les preuves. Une publication conserve la version du processus, les entrées importantes, les tests exécutés, le commit, le hash de l’archive et les limites connues.

Factory elle-même peut évoluer, mais selon la même règle que les autres workflows. Une version de son processus est utilisée telle quelle pendant un travail. Pour la modifier, on la forke, on édite son contrat, on la teste en parallèle, puis on approuve une nouvelle version. On ne change pas les gates d’une fabrication déjà engagée.

### Publication, usage et retour

Une version terminée est empaquetée, signée et adressée par son contenu. Elle peut être annoncée sur le réseau, retrouvée par RRV et installée par d’autres nœuds. L’installation ne vaut pas activation automatique : l’utilisateur inspecte la version et ses demandes de capacités, puis accorde ou refuse les permissions.

L’usage produit erreurs, mesures, retours d’accessibilité, évaluations et nouveaux risques. Les informations privées restent locales ; les éléments destinés au commun deviennent des projections publiques limitées.

Le cycle peut alors recommencer :

```text
besoin humain
  → Ideas Hub : clarification et prise en charge
  → RRV : recherche, preuves et limites
  → Conception : choix et spécification
  → Workflow Hub : workflow versionné
  → Factory : réalisation, revues et fermeture
  → SBFB : publication et distribution
  → usage : résultats, incidents et retours
  → RRV et Ideas Hub : nouvelles connaissances, forks ou nouveaux besoins
```

Ce mouvement forme un cercle vertueux seulement si les retours sont honnêtes et les projets maintenus. Davantage d’utilisateurs produisent davantage de besoins et de tests ; ils ne produisent pas automatiquement davantage de mainteneurs. Nexus rend la coopération plus accessible et la reprise moins coûteuse. Il ne supprime pas le besoin de personnes responsables.

## Chapitre 2 — Le nœud local, les applications P2P et les données partagées

Nexus ne commence pas par un compte hébergé sur un service central. Il commence par un nœud local. Ce nœud donne à la personne une identité technique, un espace de stockage, une interface, un cache d’applications, un moteur de workflows et, si elle le souhaite, des capacités de calcul et les outils Factory.

Le nœud constitue le centre de gravité de l’expérience. Le réseau étend ses possibilités ; il ne remplace pas son autorité locale.

### Ce qui vit sur la machine

Le daemon est le service local qui parle au réseau, conserve les métadonnées, vérifie les signatures, gère les blobs et expose une API en boucle locale. Le shell est l’interface de confiance. Il affiche les applications, les sources, les permissions, les notifications, les demandes de consentement et les états du nœud.

Les clés privées, les secrets et les décisions privilégiées restent du côté de cet hôte. Une application ouverte dans Nexus ne reçoit ni la clé du nœud, ni un accès libre au système de fichiers, ni le bearer du daemon.

Hors ligne, le nœud peut ouvrir une application déjà récupérée, consulter ses données ou préparer un brouillon. Lorsque la connexion revient, il reprend les échanges autorisés et vérifie les changements. « Local-first » signifie fonctionner sans dépendre à chaque instant d’un serveur distant.

### Des applications statiques dans un espace borné

Une application SBFB est principalement une archive web statique : HTML, CSS, JavaScript et autres ressources nécessaires. Le shell l’ouvre dans une iframe sandboxée, avec une politique de contenu stricte. L’application n’établit pas directement des connexions arbitraires vers Internet. Elle dialogue avec l’hôte par un bridge typé.

Le bridge transforme « lire cette clé », « rechercher » ou « soumettre une tâche » en opération contrôlée. Le shell vérifie application, version, capacités déclarées et grants accordés ; le daemon valide les entrées.

Ce modèle est moins permissif qu’un navigateur généraliste et moins puissant qu’un programme natif. C’est volontaire. Une application de lecture, une carte locale, un tableau de données ou un atelier d’idées n’a pas besoin d’obtenir le contrôle de la machine pour être utile.

Les opérations réellement privilégiées restent dans des outils locaux séparés. Factory Operator, par exemple, peut travailler dans un dépôt, lancer des tests et préparer un commit. Il n’est pas injecté dans l’iframe d’une application du réseau.

### Iroh : transporter sans créer une autorité centrale

Iroh fournit plusieurs primitives réseau complémentaires.

Les **blobs** transportent les contenus immuables : archives d’applications, packs, artefacts et pièces de preuve. Leur adresse dépend de leur contenu. Si un seul octet change, l’adresse change. Un nœud peut donc vérifier ce qu’il reçoit sans faire confiance au serveur qui le lui transmet.

Le **gossip** et les feeds diffusent des annonces et des événements signés : une version a été publiée, une source est devenue obsolète, un curateur recommande un projet, un artefact est disponible. Ces messages rendent les changements découvrables, mais une annonce n’est pas elle-même le contenu annoncé.

Les **docs** servent aux états partagés qui doivent évoluer, par exemple un espace applicatif répliqué entre un ensemble de nœuds. Le partage est explicite et lié à une namespace. Toutes les données de toutes les applications ne sont pas placées dans un document global.

Iroh essaie d’établir des échanges directs entre pairs et peut utiliser un relais de transport lorsque la topologie du réseau l’exige. Un relais aide les octets à circuler ; il ne devient pas pour autant l’autorité de la publication ni le propriétaire logique des données.

Nexus n’est ni une blockchain à consensus mondial, ni une base distribuée unique où tous verraient instantanément le même état. Il combine contenus immuables, journaux signés, espaces partagés choisis et vues locales.

### Héberger n’est pas publier

Un nœud peut conserver et servir une archive créée par quelqu’un d’autre. Cela améliore sa disponibilité, mais ne transforme pas l’hébergeur en auteur. La publication est liée à une identité, à une signature et à une provenance ; l’hébergement signifie seulement que le nœud possède les octets et accepte de les transmettre.

Une application peut ainsi survivre à l’arrêt temporaire de son auteur sans brouiller les responsabilités. L’identité Ed25519 atteste la clé signataire, pas l’identité civile d’une personne : l’interface conserve cette nuance pour auteurs, champions et workers.

### Quatre catégories de données

Pour comprendre Nexus, il est utile de distinguer quatre catégories.

1. **Les données privées locales** : préférences, secrets, brouillons, historique privé d’un run, documents non publiés. Elles restent sur le nœud, sauf export ou partage explicite.
2. **Les données partagées dans un groupe ou une application** : elles circulent dans une namespace déterminée avec des droits de lecture ou d’écriture. Leur caractère P2P ne les rend pas publiques.
3. **Les artefacts publiés** : applications, packs, versions, manifests et documents rendus accessibles par leur adresse de contenu.
4. **Les projections publiques** : résumés volontairement limités permettant la découverte et la vérification — auteur technique, version, licence, digest, provenance, statuts ou résultats sélectionnés.

RRV consomme principalement les artefacts publics et les projections autorisées. Il ne parcourt pas les bases privées des applications. Une application peut publier un schéma ou des statistiques expurgées sans publier les lignes qui les ont produites.

Une projection n’est jamais la base d’autorité originale. Elle sert à rechercher et à expliquer. Si une carte indique qu’un workflow a réussi, l’utilisateur doit pouvoir remonter au run, aux reçus et aux digests auxquels il a accès. Si ces éléments manquent, RRV indique que l’affirmation est incomplète.

### Installation, activation et évolution

Lorsqu’un nœud découvre une application ou un pack, il peut d’abord l’installer désactivé. Cette étape récupère la version et permet de l’inspecter sans lui accorder de capacités.

L’activation présente ensuite les demandes : stockage, recherche, tâche réseau, destination externe, accès à un provider ou autre effet. L’utilisateur peut accepter, refuser ou limiter. Les permissions effectives sont l’intersection entre ce que le pack demande, ce que la version déclare, la politique du nœud et le consentement accordé.

Une mise à jour possède un nouveau digest et ne remplace pas silencieusement l’ancienne. Si données, outils ou destinations changent, elle demande un nouveau consentement. L’utilisateur peut comparer, différer, revenir en arrière ou révoquer.

Les Proof Cards rendent ce cycle lisible. Elles réunissent les informations disponibles sur la source, la licence, le hash de l’archive, la fraîcheur, la provenance, les tests, les curations et les risques connus. Une Proof Card n’affirme pas qu’un logiciel est parfait. Elle montre ce qui a été établi et ce qui reste incertain.

Le réseau Nexus est ainsi un ensemble de vues souveraines qui peuvent coopérer. Il ne recherche pas une page d’accueil identique pour tous. Chaque nœud choisit ses sources et ses curateurs, tout en conservant la possibilité de retrouver l’origine d’un artefact et de quitter une recommandation.

## Chapitre 3 — Agents et calcul : local, cloud et réseau sous consentement

Dans Nexus, un agent n’est pas une personnalité magique placée au-dessus du système. C’est une définition versionnée et bornée : un rôle, un prompt, des schémas d’entrée et de sortie, une politique de modèle, des outils possibles, un contexte, des budgets, des timeouts et des limites d’itération.

Un workflow assemble ces agents avec des opérations déterministes. Cette combinaison permet d’automatiser des recherches, des transformations, des vérifications ou des fabrications sans abandonner la maîtrise du plan.

### Du graphe au plan compilé

L’auteur travaille dans Workflow Hub. Il relie des entrées, des agents, des outils, des branches, des maps, des joins, des boucles bornées, des sous-workflows et des validations humaines. L’interface peut aider à produire un diff déclaratif, mais l’assistant ne modifie pas silencieusement le workflow.

Avant le lancement, le compilateur Rust vérifie l’ensemble. Les ports doivent être compatibles. Les boucles doivent avoir une borne. Les destinations, les capacités et les budgets doivent être connues. Un passage humain requis ne peut pas être supprimé par une sortie de modèle.

Le résultat est un plan compilé immuable. Le runtime local en devient l’autorité d’exécution. Il journalise les événements, planifie les étapes, conserve une inbox et une outbox, crée des checkpoints et applique les politiques de retry et d’idempotence.

Le journal permet de reprendre après un crash. Une clé d’idempotence limite la répétition d’un effet externe. Si un effet non idempotent a peut-être eu lieu sans reçu fiable, le runtime déclare `outcome_unknown` au lieu d’inventer un résultat.

### Trois lieux d’exécution des modèles

Un même workflow peut utiliser plusieurs familles de providers, à condition que sa politique l’autorise.

Le **calcul local** utilise les ressources de la machine, par exemple via Ollama ou llama.cpp. Les données ne quittent pas le nœud pour l’inférence. Le coût marginal et la confidentialité peuvent être favorables, mais la vitesse et la qualité dépendent du matériel et du modèle disponible.

Le **cloud** utilise un provider configuré par l’utilisateur. Il peut donner accès à des modèles plus puissants ou plus rapides. Il implique toutefois une sortie de données, des conditions de service, un coût et une disponibilité externes. Nexus ne transforme pas une clé API en secret contenu dans un pack publié. Le shell de confiance résout le credential au moment autorisé et n’en révèle pas la valeur au workflow ou à l’application.

Le **réseau SBFB** distribue des tâches bornées à des workers volontaires. C’est particulièrement adapté au batch : traduction de morceaux publics, indexation, scan de licence, build, test, extraction de schéma ou vérification d’un artefact. La tâche décrit les entrées et les limites ; le worker produit un résultat signé accompagné des métadonnées prévues.

Le lieu vient du plan, de la politique de provider, de la classification des données, des capacités et du consentement. Un modèle peut proposer un contenu ; il ne peut pas s’accorder un provider, relever son budget ou changer de destination.

### Consentement de l’utilisateur et consentement du fournisseur

Le consentement existe des deux côtés.

La personne qui lance un workflow voit les capacités demandées, les types de données, les destinations, le coût estimé et les étapes humaines. Une action sensible est confirmée dans le shell de confiance, pas dans une fausse boîte de dialogue dessinée par l’application sandboxée. La confirmation est liée au digest exact de la version et à l’effet présenté.

La personne qui fournit du calcul choisit également ses limites : backend autorisé, modèles, consommation, disponibilité, types de tâches et règles de consentement. Mettre un GPU à disposition ne signifie pas accepter toute charge venue du réseau.

Les secrets constituent une catégorie séparée. Un pack peut déclarer qu’il a besoin d’un secret nommé, sans contenir sa valeur. Le runtime vérifie qu’un resolver autorisé est disponible. Les logs, prompts, événements et artefacts sont expurgés selon leur politique. Une absence de secret bloque ou suspend proprement l’étape au lieu de pousser l’utilisateur à le coller dans un champ public.

### Ce que les preuves de calcul disent — et ne disent pas

Une tâche peut conserver le hash de ses entrées, l’identité technique du worker, le modèle déclaré, le digest disponible, les paramètres, les sorties et les reçus. Plusieurs workers peuvent être sollicités et leurs résultats comparés. Ces éléments améliorent l’auditabilité.

Ils ne transforment pas automatiquement une réponse en vérité. Une signature prouve quel nœud a signé. Un hash prouve l’intégrité d’un contenu donné. Un test prouve qu’un comportement a été observé dans certaines conditions. Un quorum prouve que plusieurs participants ont produit des résultats compatibles selon une règle. Aucun de ces mécanismes ne garantit seul que l’interprétation du monde est juste.

Matériels et runtimes peuvent produire des divergences numériques : selon la tâche, on compare une égalité exacte, une tolérance ou des propriétés vérifiables. Le sharding d’un grand modèle réduit la contrainte sur un nœud, mais augmente latence, fragilité et exposition des activations. Il n’est pas confidentiel sur un réseau public ; les usages sensibles restent locaux ou dans un groupe privé approuvé.

### Les limites restent partie du produit

Provider absent, worker disparu, budget atteint, source invalide ou refus humain font partie des états normaux à expliquer.

Il n’existe pas de scheduler mondial déplaçant librement un run privé. L’autorité reste sur le nœud initiateur. Des feuilles de calcul peuvent être distribuées ; plan, secrets, décisions et état privé ne deviennent pas publics.

Enfin, l’automatisation ne résout pas le problème de la maintenance. Agents et compute abaissent le coût de certaines tâches, permettent des contrôles plus systématiques et rendent un petit groupe plus capable. Ils ne remplacent ni le porteur du besoin, ni la revue sceptique, ni les personnes concernées, ni la décision d’arrêter une solution devenue nuisible.

## Chapitre 4 — Une journée avec Nexus

À 8 h 10, Camille ouvre son nœud. Elle participe à une association de quartier qui aide les personnes isolées pendant les épisodes de forte chaleur. La semaine précédente, plusieurs habitants ont expliqué qu’ils ne savaient pas quels lieux frais étaient réellement ouverts, accessibles en fauteuil, joignables sans voiture et disponibles hors ligne.

Camille ne commence pas par demander à un agent de « créer une application canicule ». Dans Ideas Hub, elle ajoute une graine de besoin. Elle décrit le problème, les personnes concernées, la nécessité d’un mode imprimable et le risque principal : ne jamais publier l’adresse ou l’état de santé d’une personne vulnérable.

Le Hub lui montre deux besoins proches provenant de ses sources choisies. L’un concerne une carte de fontaines ; l’autre, un annuaire hors ligne de lieux d’accueil. Aucun n’apparaît comme « gagnant du réseau ». Camille lit leurs sources et contacte leurs porteurs. Ils décident de travailler ensemble. Camille signe une prise en charge limitée : produire un pilote local, avec une revue sceptique menée par Samir et un abandon prévu si les lieux ne peuvent pas être vérifiés.

À 8 h 45, ils ouvrent RRV depuis la fiche du besoin. La recherche est d’abord limitée au réseau et aux dépôts qu’ils ont autorisés. RRV retrouve une application cartographique publiée par un autre collectif, un composant de stockage hors ligne, un workflow de validation d’adresses et plusieurs schémas d’accessibilité.

Chaque résultat indique sa nature. Le composant de carte possède une archive signée et des tests, mais sa source date de neuf mois. Le workflow de validation a une provenance complète et une licence compatible. Une affirmation sur la fraîcheur des données vient seulement d’un README : RRV la marque « lue », pas « vérifiée ».

Camille élargit volontairement la recherche au Web pour consulter les jeux de données municipaux. Les liens externes restent étiquetés comme tels. RRV ne les élève pas au rang de preuve SBFB. Le groupe découvre surtout qu’il n’a pas besoin de construire un moteur cartographique : un fork de l’application existante suffira.

À 9 h 30, Conception reçoit la graine clarifiée, le dossier RRV et les contraintes. Plusieurs variantes sont examinées : une carte interactive complète, une liste très légère avec export papier, et un mode mixte. Une vérification de faisabilité montre que certaines communes ne fournissent pas d’API stable. Le `ConceptionPack` retient donc une application statique capable d’importer un fichier signé par un collectif local, de conserver la dernière version hors ligne et de produire une liste imprimable.

Le pack précise aussi ce qui n’est pas construit : aucun profil d’habitant, aucune géolocalisation permanente, aucun classement des personnes, aucune prédiction de vulnérabilité. Une adresse de lieu ne devient publique qu’après validation humaine. L’application doit expliquer la date de dernière vérification au lieu de promettre une fraîcheur qu’elle ne peut garantir.

À 10 h 20, le groupe ouvre Workflow Hub. Il part d’un workflow existant et le forke. Le nouveau workflow comporte six étapes : importer les données publiques, normaliser les champs, détecter les incohérences, demander une vérification locale, produire les formats web et papier, puis demander une approbation avant publication.

Camille préfère le mode guidé ; Samir utilise l’outline au clavier ; une troisième participante vérifie le canevas. Ils travaillent sur le même modèle. Ils ajoutent un agent local chargé de suggérer les incohérences d’adresse, mais sa sortie n’a aucune autorité : elle alimente une liste à revoir. Le workflow impose un schéma pour les lieux, un budget de calcul et une boucle de correction limitée à deux passages.

La traduction des descriptions publiques peut être envoyée à des workers volontaires. Les notes internes de vérification restent locales. La capacité `network.compute` ne s’applique qu’au champ public prévu. La publication exige une intervention humaine dans le shell.

Workflow Hub exécute les évaluations. Un cas vérifie que le workflow refuse une adresse sans commune. Un autre vérifie qu’un champ libre contenant un numéro de téléphone privé n’entre jamais dans l’artefact public. Un troisième simule la disparition d’un worker de traduction. Le run reprend avec un autre worker sans doubler l’étape déjà reçue.

À 11 h 15, le compilateur valide la version 1.0 du workflow et produit son plan. À partir de cet instant, le run lancé par Camille est épinglé à ce plan. Samir remarque qu’un export en gros caractères serait utile. Il prépare la version 1.1 dans un brouillon séparé ; la version 1.0 en cours ne change pas.

Après le déjeuner, Factory ouvre le `ConceptionPack`. Elle crée un projet à partir d’un template statique accessible, importe les composants retenus par RRV et conserve leurs licences et digests. L’agent d’implémentation travaille dans un espace borné. Il ne reçoit pas le droit de publier.

Une première prévisualisation apparaît dans la sandbox. Les tests trouvent un problème : le statut « vérifié » n’est distingué que par la couleur. Factory corrige le composant, ajoute un texte et une icône, puis relance les tests. Le scan de secrets confirme qu’aucune note privée n’a été embarquée. Le build produit une archive, mais le processus n’est pas encore fermé.

Samir mène la revue indépendante. Il constate que la date de fraîcheur est calculée à partir de l’import et non de la vérification humaine. L’agent propose un correctif. Factory réconcilie la remarque avec le `ConceptionPack`, modifie le schéma et ajoute un test de non-régression. Une seconde revue passe.

À 15 h 40, Camille voit le diff, les tests exécutés, les limites connues et la Proof Card provisoire. Elle confirme le commit. Factory ne prend pas cette décision à sa place. Le commit relie le besoin, le pack de conception, la version du workflow, les sources réutilisées et les résultats de validation.

À 16 h 10, le collectif lance le premier run réel. L’import municipal répond, mais le service d’une commune coupe la connexion pendant la récupération. Le runtime conserve le checkpoint et reprend l’étape. Plus tard, l’envoi d’une demande de vérification reçoit une réponse ambiguë du service externe. Comme aucun reçu fiable n’est disponible, le journal affiche `outcome_unknown`. Il ne prétend pas que le message est parti. Camille appelle directement le lieu et clôt l’incertitude par une validation humaine.

Le workflow produit enfin deux artefacts : la petite base publique des lieux vérifiés et le rapport interne des éléments écartés. Seul le premier peut entrer dans l’application publiée.

À 17 h, Camille approuve la publication dans le shell. Le daemon signe la version et annonce son adresse de contenu. Deux autres nœuds de l’association la récupèrent. Ils l’installent d’abord désactivée, lisent sa licence, sa provenance, ses capacités et sa date de vérification. L’application ne demande qu’un stockage local limité et la lecture de l’artefact public. Ils l’activent puis acceptent de la conserver pour la servir à leur tour.

Une personne ouvre l’application sur un ordinateur portable, puis coupe le réseau. La carte et la liste restent disponibles. Elle augmente le zoom, active les contrastes renforcés et imprime la version simplifiée. Aucune préférence liée à son handicap n’entre dans l’application ou dans le feed : le shell les a traduites en effets d’affichage sans révéler leur cause.

À 18 h 20, un collectif d’une autre ville découvre le projet dans RRV. Il voit la version, les sources, les tests et les limites. Il ne peut pas confondre « servi par trois nœuds » avec « vérifié par trois autorités ». Il choisit de forker l’application et le workflow pour les adapter aux lieux d’accueil hivernaux. La lignée entre les versions reste visible.

Avant de fermer son nœud, Camille consulte le tableau des retours. Un utilisateur a signalé un texte trop technique. Une commune a publié une nouvelle source. Un test réel confirme que l’impression fonctionne sur une ancienne imprimante. Les retours publics peuvent enrichir la projection RRV ; les messages contenant des informations personnelles restent dans l’espace local du collectif.

La journée n’a pas transformé une phrase en produit par magie. Elle a relié des gestes distincts : formuler un besoin, chercher l’existant, choisir une solution, écrire un workflow, compiler un plan, réaliser une application, revoir le résultat, consentir aux effets, publier une version et apprendre de son usage.

C’est précisément ce que Nexus V1 cherche à rendre possible. Le réseau ne remplace pas les personnes. Il leur donne un cadre où l’intelligence artificielle, le calcul distribué et le pair-à-pair augmentent leur capacité sans effacer les responsabilités, les preuves ni le droit de reprendre autrement.
