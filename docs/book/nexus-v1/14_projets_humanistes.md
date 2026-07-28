# Chapitre 14 — Des projets qui augmentent la capacité d’agir

Une infrastructure devient humaniste non parce qu’elle emploie le mot
« humain », mais parce qu’elle laisse aux personnes davantage de prise sur ce
qui les concerne. Elle ne leur demande pas de confier leur vie à une
organisation lointaine pour avoir le droit de créer, de s’entraider ou de
transmettre. Elle rend visibles les responsabilités, limite les pouvoirs
implicites et préserve la possibilité de partir, de corriger et de recommencer
autrement.

La V1 achevée de Nexus ne résout pas à elle seule la solitude, la pauvreté, le
dérèglement climatique ou l’inégalité d’accès au savoir. Un protocole ne
remplace ni une relation de confiance, ni un service public, ni une association
de quartier. Il peut cependant réduire une difficulté moins spectaculaire et
pourtant décisive : la distance entre une intention collective et sa
réalisation.

Beaucoup de projets utiles meurent dans cet intervalle. Une idée est formulée,
puis se perd dans une messagerie. Des documents sont produits, mais nul ne sait
quelle version fait foi. Une application est commencée, sans parvenir jusqu’aux
personnes qui en auraient besoin. Les mêmes erreurs sont ensuite répétées
ailleurs, faute de mémoire partageable.

## Relier ce qui reste habituellement séparé

Nexus relie quatre fonctions qui, séparées, restent souvent stériles.
L’**Atelier des communs (Ideas Hub)** accueille les problèmes, les propositions
et leurs transformations. Workflow Hub permet d’en décrire le chemin de
réalisation, de le versionner et d’en gouverner les effets. Factory transforme
un besoin suffisamment défini en application ou en outil exécutable, sans
laisser son évolution devenir une mutation incontrôlée en cours d’usage. RRV,
enfin, rend les productions, leurs sources et leurs preuves retrouvables.

RRV ne décide pas à la place des personnes. Il leur donne les moyens de savoir
ce qui existe, d’où cela vient et sur quoi repose une affirmation. Cette
continuité rend possibles des projets modestes par leur taille, mais importants
par leurs conséquences.

Une communauté rurale peut publier un annuaire hors ligne des ressources
essentielles : permanences administratives, transports solidaires, lieux de
soin, ateliers de réparation, producteurs, bibliothèques et espaces où trouver
une connexion. Les informations publiques circulent entre les nœuds
volontaires. Les coordonnées privées d’une personne vulnérable, elles, ne sont
jamais placées dans un contenu répliqué. Lorsque les horaires changent, une
nouvelle version est proposée, vérifiée par un responsable local, puis publiée
avec l’historique de la correction.

Un collectif d’entraide peut construire un parcours d’accueil accessible. Une
personne aveugle peut créer ou adapter le workflow par conversation, au clavier
et avec un lecteur d’écran. Une personne peu à l’aise avec l’écrit peut suivre
un dialogue guidé plutôt qu’un graphe visuel. Le modèle de langage aide à
formuler et à structurer ; le compilateur vérifie les invariants ; l’humain
confirme les effets sensibles. L’accessibilité n’est alors plus une page
ajoutée à la fin du projet. Elle devient une manière normale d’entrer dans le
système.

Une bibliothèque, une école populaire ou un atelier associatif peut publier
des parcours pédagogiques dont les ressources demeurent consultables sans
compte cloud obligatoire. Les participants peuvent forker un parcours, le
traduire, ajouter une activité, documenter ce qui a fonctionné, puis proposer
leur version à la communauté d’origine. L’auteur initial ne perd pas son
travail ; le groupe local n’est pas condamné à obéir à une forme unique. La
filiation des versions permet de coopérer sans effacer les désaccords.

Des habitants peuvent documenter un problème écologique concret : température
excessive dans une cour, disparition d’un passage piéton, pollution lumineuse,
difficulté d’accès à l’eau, déchets autour d’un cours d’eau. Ils peuvent
associer observations, photographies publiables, sources officielles,
propositions et comptes rendus de décisions. RRV rend ces éléments
retrouvables. L’Atelier des communs évite que dix formulations proches restent
isolées. Workflow Hub conduit la proposition vers une vérification de
faisabilité, une revue de confidentialité et une décision humaine. Factory
permet de produire un outil de collecte ou de suivi lorsque le simple document
ne suffit plus.

Un réseau d’associations peut aussi mutualiser des tâches moins visibles :
traduire un document, rendre un formulaire accessible, vérifier des liens,
produire une version facile à lire, convertir un jeu de données public ou
préparer un compte rendu. Certaines tâches peuvent utiliser du calcul
volontaire, avec des plafonds et un consentement explicite. Le fait qu’une
machine soit disponible ne donne toutefois aucun droit sur son propriétaire,
et le fait qu’une tâche soit utile ne justifie pas de lui confier des données
sensibles. La capacité de calcul reste une capacité accordée, bornée et
révocable.

## Babel : rendre un patrimoine réellement accessible

Babel est l’un des exemples les plus complets de cette ambition. Son objectif
n’est pas seulement de produire du texte traduit. Il transforme un corpus
redistribuable en bibliothèque multilingue dont chaque fragment conserve son
origine.

Le workflow commence par une vérification des droits. Un livre du domaine
public, une œuvre sous licence compatible et un texte simplement disponible
sur le Web ne possèdent pas le même statut. La source acceptée reçoit un
manifest signé et une version précise.

Le corpus est ensuite découpé de manière déterministe. Chaque fragment garde le
lien vers l’œuvre, l’édition, la langue, la position et le digest d’entrée. Les
tâches publiques peuvent être distribuées à des workers volontaires avec le
contexte minimal nécessaire. Cette fragmentation rend possible une
contribution large sans confier tout le corpus de travail ni l’état privé du
run à chaque machine.

Les sorties traversent plusieurs étapes distinctes :

1. brouillon de traduction ;
2. contrôles automatiques de structure, de termes et de fragments manquants ;
3. comparaison de variantes lorsque plusieurs workers contribuent ;
4. correction et revue humaines ;
5. attestation de la version retenue ;
6. réassemblage ;
7. publication avec le graphe de provenance.

La signature d’un worker prouve qui a signé sa sortie. Elle ne prouve pas la
qualité littéraire ou la fidélité. Un accord entre plusieurs modèles peut
révéler une convergence, pas un sens parfait. L’humain reste nécessaire pour
les ambiguïtés, le style, le contexte culturel et la responsabilité
éditoriale.

Le résultat peut être lu hors ligne, adapté à plusieurs supports, rendu
accessible et repris par une autre communauté. Une correction ne réécrit pas
silencieusement les anciennes éditions : elle crée une nouvelle version dont
la filiation reste consultable.

Babel montre aussi pourquoi les différentes briques de Nexus ont besoin les
unes des autres. L’Atelier des communs choisit les corpus et les publics. RRV
retrouve les sources, licences et traductions existantes. Conception fixe le
modèle éditorial et les critères. Workflow Hub rend le pipeline versionnable.
Le réseau apporte du calcul. Factory construit les outils de lecture et de
publication. Les curateurs aident enfin chaque nœud à trouver les collections
qu’il souhaite suivre.

Une bibliothèque commune ne naît donc pas d’un modèle qui traduit tout. Elle
naît d’une chaîne où les droits, les contributions, les choix et les
corrections restent transmissibles.

## Quatre propriétés, pas une promesse magique

Dans chacun de ces projets, la V1 apporte quatre propriétés.

La première est la **portabilité**. Une application, un workflow ou un ensemble
de documents ne dépend pas entièrement d’un serveur central et d’un compte
détenu par un intermédiaire. Une communauté peut conserver ce qu’elle a
produit, le réinstaller et le transmettre selon les capacités qui lui ont été
accordées.

La deuxième est la **gouvernance des effets**. Un workflow n’obtient pas une
autorité générale parce qu’il porte une bonne intention. Lire un document
public, envoyer un message, dépenser un budget, accéder à un secret ou publier
une version sont des effets différents. Ils doivent être accordés séparément.
Les effets les plus sensibles peuvent exiger une approbation humaine
explicite. Si l’issue d’une action externe ne peut pas être connue avec
certitude, le moteur conserve cette incertitude au lieu d’inventer un succès
rassurant.

La troisième est la **mémoire vérifiable**. Une version peut être reliée à son
auteur, à ses validations, aux transformations qui l’ont précédée et aux
artefacts qu’elle a produits. Cette mémoire rend les responsabilités plus
lisibles et les apprentissages plus facilement transmissibles.

La quatrième est le **droit à la pluralité**. Une communauté peut adopter une
version, la refuser, la forker ou en proposer une autre. Le protocole établit
les relations entre les objets ; il ne décrète pas qu’une seule communauté
détient l’interprétation légitime d’un problème.

## La provenance n’est pas la vérité

Il faut ici poser une limite essentielle. Une signature ne transforme pas une
déclaration en vérité. Elle permet d’établir, dans les limites du système
cryptographique et de la garde des clés, qu’une identité a signé un contenu
déterminé. Une chaîne de provenance peut montrer qu’un résultat est issu de
telle version, passée par telles étapes, avec tels reçus. Elle ne démontre pas
que l’observation initiale était exacte, que l’expert était compétent, que le
protocole d’enquête était impartial ou que la conclusion est juste.

RRV doit donc présenter plusieurs questions distinctes : qui affirme ? sur
quelle source ? à quelle date ? à partir de quelle version ? quelles
vérifications ont été réalisées ? quelles objections subsistent ? Une réponse
peut être bien attribuée et pourtant fausse. Elle peut être sincère et pourtant
incomplète. Elle peut être largement approuvée et pourtant injuste. La
provenance améliore la possibilité d’examiner une affirmation ; elle ne
dispense jamais de l’examen.

L’ambition humaniste de Nexus tient précisément dans cette retenue. Il ne
promet pas une machine qui saura le bien à notre place. Il propose une
infrastructure dans laquelle des personnes peuvent mieux exercer leur
jugement, rendre leurs décisions contestables, préserver les traces utiles et
limiter l’autorité qu’elles délèguent.
