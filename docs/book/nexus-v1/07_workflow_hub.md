# Chapitre 7 — Workflow Hub, rendre l’organisation programmable

Les organisations reposent sur des processus que presque personne ne voit en
entier.

Une demande arrive par courriel. Une personne la reformule. Une autre cherche
des sources. Un tableur tient lieu de file d’attente. Un spécialiste intervient
si quelqu’un pense à le prévenir. Une décision importante se prend dans une
conversation dont le contexte disparaîtra. Lorsque le responsable est absent,
le processus ralentit parce qu’une partie de la règle vivait dans sa mémoire.

Workflow Hub transforme ces habitudes en objets que l’on peut comprendre,
versionner, tester et transmettre.

Il ne cherche pas à rendre toute relation humaine algorithmique. Il rend
explicite ce qui l’est déjà : les entrées, les rôles, les passages de relais,
les conditions, les outils, les limites, les décisions et les sorties.

## Une bibliothèque de manières de faire

La première page du Hub n’est pas une promesse marketing. C’est une
bibliothèque de travail.

On y retrouve les workflows récents, les brouillons, les modèles, les packs
installés, les versions publiées et les forks. Chaque objet montre son auteur,
sa licence, son digest, son statut et les capacités qu’il demande. La personne
peut reprendre un brouillon, comparer deux versions, ouvrir un run ou partir
d’un modèle.

Le catalogue plus large permet de découvrir les workflows publiés par
d’autres. Il ne les range pas selon un podium mondial. La recherche utilise des
critères explicites : domaine, langue, schéma d’entrée, licence, capacité,
source ou curateur choisi.

Un modèle n’est pas une règle imposée. Il est une hypothèse réutilisable. Une
association peut installer un workflow de traduction, l’exécuter sur des
exemples, puis le forker pour adapter ses validations. La lignée reste visible
sans empêcher la divergence.

## Décrire un workflow sans écrire du JSON

L’auteur peut commencer de quatre manières : un workflow vide, un modèle, le
fork d’une version existante ou un import déclaratif borné. Il précise ensuite
les entrées et les sorties, choisit ou crée les agents, ajoute des opérations
et relie les données.

Le langage du Hub permet d’exprimer :

- des étapes déterministes ;
- des appels de modèle ;
- des outils autorisés ;
- des branches et des décisions ;
- des travaux parallèles et leur réunion ;
- des traitements répétés avec une borne ;
- des sous-workflows ;
- des passages de relais entre agents ;
- des validations humaines ;
- des artefacts produits pour une autre application.

L’auteur ne manipule pas obligatoirement la structure brute. Les formulaires,
les commandes et les diagnostics conservent les types et les conséquences
visibles.

Cette exigence ne simplifie pas le système au point de cacher les risques. Un
débutant peut travailler en langage clair, mais il voit toujours les données,
destinations, coûts et pouvoirs que son workflow implique.

## Quatre modes pour un même sens

Le même workflow peut être créé par plusieurs chemins.

Le **canvas** offre une représentation spatiale. Les étapes deviennent des
nœuds, les données circulent par des ports et les groupes rendent les parties
du processus visibles.

L’**outline** présente le graphe comme une structure textuelle complète. Il
permet de naviguer, connecter, configurer et corriger sans déplacement
graphique.

Le **mode guidé** traite une étape ou une décision à la fois. Il explique les
conséquences, conserve le contexte et évite d’exiger le vocabulaire d’un
diagramme.

Le **mode conversationnel** permet enfin de créer et de modifier le workflow en
dialoguant : « après les deux recherches, ajoute une revue indépendante ; ne
transmets au cloud que les documents publics ; demande-moi avant toute
publication ». Le système répond par une proposition structurée, explique ce
qu’elle change et montre les nouveaux pouvoirs demandés.

Ces modes ne sont pas quatre produits. Ils appliquent les mêmes commandes
typées au même modèle. À sémantique égale, ils produisent le même digest. Le
canvas peut conserver une mise en page différente sans créer un workflow
différent.

Cette parité change le statut de l’accessibilité. Une personne aveugle qui
utilise la conversation ou l’outline n’accède pas à une traduction pauvre du
canvas. Elle possède une voie de création de première classe. Une personne
fatiguée peut passer du canvas au mode guidé. Une équipe peut relire
collectivement le même processus dans la représentation qui lui convient.

## L’assistant propose un diff

L’intelligence artificielle peut accélérer la composition. Elle peut suggérer
un agent, signaler une étape oubliée, transformer une description en brouillon,
proposer une correction ou expliquer pourquoi une connexion ne fonctionne pas.

Sa sortie reste un **diff déclaratif**.

L’utilisateur voit les nœuds ajoutés, les connexions modifiées, les outils
demandés, les budgets changés et les données concernées. Il peut accepter une
partie, corriger ou refuser. L’assistant ne reçoit pas le droit de :

- modifier silencieusement le workflow ;
- accorder une capacité ;
- ajouter une destination cachée ;
- lancer un run ;
- signer ou publier une version ;
- changer le plan d’une exécution déjà commencée.

La conversation n’est donc pas une délégation de souveraineté. Elle est une
autre manière d’écrire une proposition formelle.

## Des agents réutilisables, mais jamais indéfinis

Le registre d’agents sépare les rôles du dessin du workflow. Chaque agent
possède une version, un rôle, un prompt, des schémas d’entrée et de sortie, une
politique de modèle, des outils possibles, un contexte, une mémoire éventuelle,
des budgets, un timeout et une limite d’itération.

Le Hub distingue trois relations souvent confondues :

- un **appel simple** demande une sortie à un agent ;
- un **handoff** lui transmet la responsabilité d’une partie du travail ;
- un **agent-as-tool** permet à un autre agent de l’invoquer comme capacité
  bornée.

Ces différences influencent le contexte, la provenance, le budget et la
responsabilité. Elles restent visibles dans la timeline.

Modifier un agent partagé crée une nouvelle version. Les workflows qui
référencent l’ancienne ne basculent pas silencieusement. L’auteur peut comparer
les prompts, capacités, évaluations et usages avant de migrer.

## Le préflight : comprendre avant de lancer

Avant chaque run, Workflow Hub présente une lecture complète :

1. les erreurs qui empêchent la compilation ;
2. les avertissements et limites connues ;
3. les versions et digests du workflow et des agents ;
4. le lieu prévu pour chaque étape : local, fournisseur autorisé ou réseau ;
5. les données qui quittent le nœud et leur destination ;
6. les outils, leurs effets et leur réversibilité ;
7. les capacités accordées, absentes ou expirées ;
8. le coût, les budgets, les timeouts et les bornes ;
9. les décisions humaines attendues ;
10. une explication simple du résultat visé.

Cette page n’est pas une avalanche de conditions d’utilisation. Elle permet
de relier un choix à un effet. Une personne peut réduire la portée d’une
capacité, choisir un modèle local ou renoncer.

Le bouton *Lancer* transmet une intention au runtime. Il ne fabrique pas un
succès local si l’hôte refuse.

## Un plan déterminé avant l’action

Le compilateur Rust transforme la définition en `CompiledPlan`. Il vérifie les
types, les références, les branches, les réunions, les boucles, les
destinations, les budgets et les points de validation humaine.

À partir de ce moment, le plan de l’exécution est immuable.

Un modèle peut produire une donnée qui déterminera une branche déjà prévue. Il
ne peut pas inventer une nouvelle branche, ajouter un outil ou augmenter son
autorité. Une nouvelle version du workflow peut être créée pendant que le run
continue, mais elle ne remplace pas son plan.

Cette règle est la charnière entre un assistant agentique et un système
gouvernable. L’agent peut être créatif dans les espaces définis. Il ne redéfinit
pas les règles au moment où elles devraient le limiter.

## Le Centre des Runs

Une exécution ne disparaît pas lorsque l’onglet se ferme. Le runtime possède un
journal durable, des checkpoints et une machine d’état.

Le Centre des Runs distingue notamment :

- en file ;
- en cours ;
- en attente d’une personne ;
- en attente d’un système externe ;
- en pause ;
- terminé ;
- échoué ;
- annulation demandée ;
- annulé ;
- résultat inconnu.

La timeline montre les étapes, tentatives, retries, passages de relais, outils,
artefacts, décisions, coûts et providers. Les événements de modèle sont
regroupés par transition utile ; le lecteur d’écran n’est pas bombardé par
chaque token.

Après un crash, le runtime reconstruit l’état depuis le journal et reprend à la
frontière sûre. Une clé d’idempotence évite de répéter un effet déjà reçu.
Lorsqu’un effet externe a peut-être eu lieu mais qu’aucun reçu ne le confirme,
le run entre dans l’état `outcome_unknown`. L’interface explique l’incertitude
et les moyens de la résoudre.

Une annulation reste elle aussi honnête. Demander l’arrêt d’un travail local
n’annule pas magiquement un message déjà envoyé ou une tâche distante déjà
prise. Le Hub distingue ce qui peut encore être interrompu de ce qui doit être
vérifié.

## L’intervention humaine hors de l’iframe

Les approbations sensibles apparaissent dans le shell de confiance. Cette
frontière empêche une application de dessiner une fausse boîte de dialogue qui
imiterait l’autorité du système.

La demande indique l’action, les données, la destination, le coût, l’effet et
le digest exact. Le focus se déplace vers la confirmation puis revient à
l’endroit logique. Aucune épreuve de mémoire, aucun compte à rebours et aucun
texte à recopier ne bloque une personne handicapée.

La file d’approbations survit à la fermeture du Hub. L’humain n’est pas sommé
de rester devant une animation pendant qu’un workflow long attend sa décision.

## Évaluer avant de publier

Le Hub permet de créer des scénarios et des datasets versionnés. Une évaluation
peut vérifier une sortie, un budget, une trajectoire, une capacité ou
l’interdiction d’un effet.

Les graders déterministes passent avant les juges fondés sur un modèle. Un test
de schéma, une empreinte, une règle de confidentialité ou une limite de coût ne
doivent pas dépendre de l’opinion d’un LLM. Les évaluations humaines et
agentiques restent séparées.

L’auteur peut comparer deux versions, deux modèles ou deux providers. Il peut
inspecter chaque cas avant de lire une moyenne. Aucun score unique ne devient
la qualité universelle d’un agent.

## Publier, installer, activer, reprendre

Le cycle d’un pack sépare des décisions que les plateformes confondent :

```text
brouillon → validé → évalué → signé → publié → indexé
  → inspecté → installé désactivé → activé avec capacités
  → fork / mise à jour avec nouveau consentement / retour / révocation
```

Publier ne force personne à installer. Installer n’accorde aucun droit.
Activer lie des capacités à une version exacte. Une mise à jour qui change les
données ou les destinations exige un nouveau consentement. Une version
révoquée reste consultable pour l’audit mais ne lance plus de nouveaux runs.

La personne peut forker, comparer et revenir en arrière. Il n’existe pas de
mise à jour silencieuse sous prétexte que l’auteur original sait mieux.

## Relier les applications sans les fusionner

Les applications de Nexus communiquent par artefacts et opérations typées, pas
en prenant le contrôle de leurs iframes respectives.

Ideas peut produire un `DecisionPack`. Conception le transforme en
`ConceptionPack`. Factory consomme ce pack et produit un ensemble de code, de
preuves et de release. Babel peut exposer une opération de traduction qui
produit un `TranslationBatch`.

Le Hub montre l’application appelante, la version invoquée, les entrées, le
budget, la profondeur restante, le child-run et le résultat. Il refuse les
cycles, profondeurs ou capacités non autorisés.

Cette composition permet à l’écosystème de grandir sans fabriquer un
super-programme qui contient toute la logique.

## Programmer la coopération

Workflow Hub ne remplace pas une équipe. Il offre une langue commune entre les
personnes, les agents, les outils et les preuves.

Son potentiel est considérable : une communauté peut publier non seulement un
résultat, mais une manière de le produire. Une autre peut l’étudier, l’adapter
et l’améliorer. Une personne isolée peut utiliser des rôles et des reviews qui
étaient autrefois réservés à une grande organisation.

Mais ce potentiel dépend du maintien de la frontière fondamentale. Le workflow
organise le pouvoir ; il ne doit jamais devenir une excuse pour le rendre
invisible.

