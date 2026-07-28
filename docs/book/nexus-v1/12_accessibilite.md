# Chapitre 12 — L’accessibilité comme architecture

Une plateforme qui prétend augmenter la capacité d’agir ne peut pas réserver
ses fonctions les plus puissantes à ceux qui voient un grand écran, déplacent
précisément une souris, entendent un signal, mémorisent une longue séquence ou
lisent vite un langage technique.

L’accessibilité de Nexus ne commence donc pas par une liste d’attributs ajoutés
à une interface terminée. Elle commence par une question d’architecture :
combien de chemins sémantiques permettent d’accomplir la même action ?

Si le workflow n’existe que sous la forme d’un canvas, le lecteur d’écran
recevra au mieux une description de ce que les autres peuvent modifier. Si
l’approbation sensible apparaît dans une modale inaccessible, le runtime le
plus sûr devient inutilisable. Si une carte n’a pas d’équivalent textuel, la
donnée publique reste fermée à une partie du public.

La V1 traite ces situations comme des défauts fonctionnels.

## La même puissance, plusieurs formes

Workflow Hub illustre le principe le plus ambitieux. Canvas, outline, mode
guidé et conversation modifient le même modèle. Une personne peut :

- créer une étape ;
- configurer un agent ;
- connecter une sortie à une entrée ;
- définir une branche ;
- limiter une boucle ;
- inspecter les capacités ;
- corriger une erreur ;
- lancer une évaluation ;
- publier une version ;

sans dépendre d’un geste de drag, d’une position visuelle ou d’une
représentation graphique.

Le système ne se contente pas de produire une description automatique du
graphe. Il expose ses objets, relations, erreurs et commandes. L’outline est
éditable. Le mode guidé conserve la même profondeur. La conversation produit
des diffs structurés que l’utilisateur peut parcourir et accepter élément par
élément.

Cette parité est vérifiable : les chemins qui expriment le même workflow
produisent le même digest sémantique.

## Une interaction conversationnelle de première classe

Pour une personne aveugle ou malvoyante, la conversation peut devenir une
nouvelle façon principale d’interagir avec le site et le protocole :

> « Explique-moi les étapes qui envoient des données hors de mon nœud. »
>
> « Ajoute une revue humaine après la traduction et avant la publication. »
>
> « Compare cette version à celle qui a fonctionné hier. »
>
> « Lis les trois erreurs bloquantes, puis ouvre la première dans le mode
> guidé. »

Le modèle aide à formuler, naviguer et corriger. Il n’est pas l’autorité.
Chaque commande devient une intention typée. Le runtime valide les invariants.
Les consentements sensibles restent humains.

Cette séparation protège l’accessibilité elle-même. Une interface
conversationnelle entièrement dépendante d’un modèle distant pourrait devenir
inutilisable hors ligne, en cas de panne ou lorsqu’un document doit rester
privé. Les mêmes actions restent donc disponibles par commandes déterministes,
formulaires, raccourcis et structure sémantique.

La voix n’est jamais obligatoire. Une personne qui ne parle pas, ne souhaite
pas parler ou se trouve dans un espace public peut utiliser le texte, le
clavier, un switch ou une technologie d’assistance.

## Ne jamais déduire le handicap

Nexus n’a pas besoin de savoir qu’une personne est aveugle, dyslexique,
daltonienne, épileptique ou atteinte d’une maladie précise. Il a besoin de
connaître des préférences fonctionnelles :

- contraste renforcé ;
- échelle de texte ;
- réduction du mouvement ;
- densité ;
- largeur de ligne ;
- espacement ;
- annonces regroupées ;
- mode de focus ;
- délais protecteurs ;
- présentation guidée.

Le shell résout ces axes et transmet le minimum nécessaire à l’application. Il
ne place pas un diagnostic dans le DOM, un prompt, un journal, une télémétrie
ou le réseau. Il ne détecte pas la présence d’un lecteur d’écran.

Lorsque plusieurs besoins se combinent, les préférences s’empilent et la
valeur la plus protectrice l’emporte sur chaque axe. Une grande police ne doit
pas annuler la réduction de mouvement. Un contraste élevé ne doit pas forcer
une densité cognitive excessive.

La synchronisation éventuelle de ces préférences reste volontaire et
révocable. Le profil d’accessibilité n’est pas un identifiant public.

## L’hôte et l’application forment un seul parcours

Une application SBFB s’exécute dans une iframe sandboxée. Les capacités,
secrets et approbations sensibles appartiennent au shell. Cette séparation
technique peut produire une fracture d’usage si le focus, les noms et les
annonces ne traversent pas proprement la frontière.

La V1 traite donc le shell et l’application comme un seul processus
accessible :

- l’iframe possède un titre clair ;
- le passage entre hôte et application est annoncé ;
- une demande sensible déplace le focus vers le dialogue de confiance ;
- l’application inactive ne piège pas le clavier derrière la confirmation ;
- après la décision, le focus revient à l’endroit logique ;
- les couleurs, l’échelle, le mouvement et la densité restent cohérents ;
- la file d’approbations peut être ouverte même si l’app est fermée.

Une confirmation ne repose pas sur un texte à recopier, un calcul mental, un
CAPTCHA, une limite très courte ou un bouton qui se déplace. Elle explique
l’action, la donnée, la destination, le coût et les conséquences.

La sécurité n’a pas besoin de fabriquer une barrière cognitive.

## Concevoir pour des besoins qui se croisent

Les catégories de handicap aident à vérifier les risques, mais une personne ne
vit pas dans une seule colonne.

La cécité demande des rôles, noms, états, landmarks, tableaux, transcripts et
descriptions du graphe. La basse vision exige un reflow réel à fort zoom, un
focus visible et des contenus qui ne disparaissent pas derrière des panneaux.
Le daltonisme interdit qu’une couleur soit le seul signal. La surdité exige que
le son ne porte aucune information exclusive.

Les limitations motrices demandent des cibles suffisantes, l’usage complet au
clavier, au tactile, à la voix ou au switch, ainsi qu’une alternative à tout
drag. Les difficultés cognitives demandent un langage simple, une aide stable,
des conséquences visibles, une récupération d’erreur et la possibilité de
réduire la densité. La dyslexie peut demander des réglages d’espacement et de
largeur. La fatigue et les troubles de mémoire rendent l’autosave, l’historique
et la reprise indispensables. Les sensibilités vestibulaires ou
photosensibles imposent la réduction du mouvement et l’absence de flash
dangereux.

Un système réellement accessible n’additionne pas des « modes handicap ». Il
construit des primitives capables de se composer.

## L’accessibilité des résultats

Un outil auteur accessible ne suffit pas si les applications qu’il produit
restent inaccessibles.

Factory intègre donc des critères d’accessibilité aux templates, aux gates et
aux revues. Le manifeste peut déclarer les langues et les modalités. La
prévisualisation permet de tester :

- navigation clavier ;
- ordre et restauration du focus ;
- structure sémantique ;
- noms, rôles et états ;
- zoom, reflow et espacement ;
- contrastes et couleurs forcées ;
- mouvement réduit ;
- alternatives textuelles ;
- erreurs et récupération ;
- parcours mobile et tactile.

Les composants réutilisables conservent leurs preuves et leurs limites. Une
application dérivée ne peut pas revendiquer automatiquement l’accessibilité de
son template après avoir modifié son interface.

RRV peut indexer les rapports, tests et retours publiés. Une Proof Card peut
montrer qu’une matrice a été exécutée. Elle ne transforme pas un audit en
garantie éternelle pour tous les usages.

## Prouver avec des personnes

L’automatisation détecte beaucoup : absence de nom, contraste insuffisant,
focus perdu, structure invalide, contenu qui déborde, préférence ignorée. Elle
ne ressent pas la désorientation, la fatigue, le langage ambigu ou la
complexité d’un vrai parcours.

La preuve d’accessibilité combine donc :

- tests composants et statiques ;
- scénarios navigateur au clavier et avec différents réglages ;
- lecteurs d’écran sur plusieurs systèmes ;
- braille, contrôle vocal, switch et tactile ;
- audits manuels ;
- panels rémunérés de personnes handicapées ;
- correction des barrières critiques avant publication.

La V1 prend WCAG 2.2 AA comme plancher bloquant et examine les critères plus
ambitieux applicables, l’accessibilité des outils auteurs, les besoins
cognitifs et les exigences de technologies d’assistance. Elle ne promet pas une
perfection universelle. Elle publie les limites restantes.

## Une source d’innovation, pas seulement une obligation

Le mode guidé aide aussi la personne débutante. Les annonces regroupées
réduisent le bruit pour tous. Les commandes conversationnelles rendent un
graphe complexe plus compréhensible. Le reflow améliore l’usage sur un petit
écran. L’autosave et la reprise protègent chacun contre les interruptions.

L’accessibilité transforme ainsi le produit. Elle oblige Nexus à séparer le
sens de sa représentation, à rendre ses états nommables, à exposer ses
commandes et à minimiser les informations implicites.

Ce sont précisément les qualités dont un système multi-agents a besoin pour
rester explicable.

L’accessibilité n’est donc pas seulement compatible avec l’ambition de Nexus.
Elle est l’une des disciplines qui empêchent cette ambition de devenir une
interface brillante contrôlée par ceux qui savent déjà la lire.
