# Note au lecteur

Ce livre décrit Nexus comme si sa première version complète était devant nous,
allumée, utilisable et reliée à d’autres nœuds. On y ouvre des applications,
on y fait mûrir des idées, on y compose des équipes d’agents, on y construit
des outils, on y partage du calcul et on y inspecte les preuves laissées par
chaque étape. Ce choix du présent n’est pas une tentative de maquiller
l’avancement réel du projet. C’est une méthode : pour comprendre une
architecture aussi vaste, il faut voir ce que toutes ses pièces produisent
ensemble.

Une liste de fonctions ne suffirait pas. Elle dirait qu’il existe un moteur de
workflow, un moteur de recherche, une fabrique d’applications et un réseau
pair-à-pair. Elle ne montrerait pas le mouvement essentiel : une personne part
d’un problème vécu, rassemble des connaissances, rencontre des objections,
choisit une voie, organise le travail, construit une réponse, puis laisse
derrière elle des artefacts que d’autres peuvent vérifier, reprendre ou
contester.

Le mot *V1* a donc ici un sens précis. Il ne signifie pas « première version
commerciale » ni « minimum que l’on peut vendre ». Il désigne le premier état
dans lequel la proposition de Nexus tient debout de bout en bout. Une idée
n’est plus condamnée à rester dans une conversation. Une communauté n’est
plus obligée de louer son existence numérique à une plateforme. Une
application peut être publiée, distribuée et ouverte sans magasin central. Un
workflow peut survivre à la fermeture d’un onglet. Un résultat peut conserver
la trace de ses sources, de ses transformations et de ses validations. Une
personne qui ne voit pas l’écran peut créer et exploiter le même système sans
recevoir une version appauvrie.

Ce livre n’est pourtant pas une prophétie technologique. Nexus ne supprime pas
le conflit, l’erreur, la mauvaise foi, le manque de temps, l’épuisement des
mainteneurs ou la difficulté de faire vivre un commun pendant dix ans. Il ne
transforme pas une signature cryptographique en vérité. Il ne transforme pas
un grand nombre de participants en intelligence collective. Il ne transforme
pas un modèle de langage en arbitre impartial. Il ne rend pas inoffensive une
donnée sensible simplement parce qu’elle est chiffrée.

La V1 décrite ici est au contraire intéressante parce qu’elle garde ces
limites visibles. Elle organise la séparation des rôles. Un modèle propose ;
un validateur déterministe vérifie des invariants ; une personne consent,
arbitre ou refuse. Une application demande une capacité ; elle ne s’octroie
pas un accès. Une communauté recommande ; elle ne devient pas le ministère de
la vérité du réseau. Un auteur signe une publication ; il n’impose pas cette
publication aux autres. Un nœud conserve sa vue du monde ; il n’a pas besoin
d’adopter le classement universel d’un centre.

## Une description d’aboutissement, pas un constat de livraison

Au moment où ces pages sont écrites, le dépôt Nexus contient des fondations
réelles, des contrats, des outils, des démonstrateurs, des travaux de
recherche et des parties encore en construction. Certaines capacités sont
code-backed et testées ; d’autres sont des contrats en cours d’implémentation ;
d’autres encore appartiennent à la vision qui donne sa cohérence à
l’ensemble.

Le corps du livre ne répétera pas cette nuance à chaque paragraphe. Cela
rendrait le récit illisible. Il parle donc de la V1 accomplie. Toute utilisation
du livre comme preuve de maturité actuelle serait une erreur de catégorie. La
preuve de ce qui est effectivement livré doit toujours venir d’un état daté du
code, de tests, d’audits et d’expériences réelles — jamais de la puissance
d’une vision.

## Pourquoi « Nexus » et « SBFB » ?

Le dépôt et l’écosystème ont grandi avec deux noms. Dans ces pages, **Nexus**
est l’expérience complète : le nœud local, les applications, la recherche, la
création, les workflows, la fabrique et les communautés. **SBFB** est le
protocole commun qui permet aux nœuds de se reconnaître, d’échanger des objets
adressés par leur contenu, de publier des événements signés, de partager
certaines données et de contribuer du calcul.

On peut utiliser SBFB sans adopter toutes les applications de Nexus. On peut
forker une application sans forker le protocole. On peut aussi forker le
protocole lui-même, puisque la liberté de sortie fait partie de la protection
contre la capture. Cette distinction entre écosystème, protocole et
applications n’est pas cosmétique : elle empêche qu’une interface, une équipe
ou une organisation se présente comme propriétaire de tout le réseau.

## Le pari

Le pari de Nexus n’est pas que la technologie résoudra les problèmes humains.
Il est plus modeste et plus exigeant :

> si les personnes disposent d’une infrastructure qui préserve leur autonomie,
> rend les responsabilités visibles, organise la contradiction et abaisse le
> coût de transformer une idée en outil, alors davantage de problèmes pourront
> être pris en charge comme des communs.

Ce pari dépend encore de personnes capables de porter une idée, de faire un
choix, d’écrire une règle, de relire un résultat, de maintenir un logiciel et
de se retirer lorsqu’une structure trahit son but. Nexus cherche moins à
remplacer cette responsabilité qu’à lui donner un terrain praticable.

Le livre commence là : non pas avec une nouvelle intelligence artificielle,
mais avec une question très ancienne. De quels moyens avons-nous besoin pour
agir ensemble sans remettre tout notre pouvoir à celui qui héberge la
plateforme ?

