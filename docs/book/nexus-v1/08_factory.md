# Chapitre 8 — Factory, fabriquer avec des preuves

L’intelligence artificielle a rendu spectaculaire la naissance d’un logiciel.
Une phrase peut devenir une interface. Un modèle peut produire en quelques
minutes une quantité de code qui demandait autrefois plusieurs jours.

Cette accélération déplace le goulot. Écrire n’est plus toujours la partie la
plus difficile. Comprendre ce qui a été demandé, limiter le changement,
vérifier les effets, corriger sans dériver, conserver la provenance et savoir
quand le travail est réellement terminé deviennent les tâches décisives.

Factory est la réponse de Nexus à ce déplacement.

Elle ne promet pas « une application en un clic ». Elle transforme une
intention approuvée en travail isolé, revu, prouvé et livrable.

## Plusieurs portes d’entrée

Factory peut recevoir :

- une demande humaine directe et bornée ;
- un brief issu de l’Atelier des communs ;
- un `DecisionPack` ;
- un `ConceptionPack` ;
- une correction ou une migration définie dans un dépôt ;
- le fork d’une application ou d’un composant trouvé par RRV.

Conception n’est donc pas un passage obligatoire. En revanche, l’entrée doit
indiquer ce que l’on veut changer, ce que l’on ne veut pas changer et comment
le résultat sera évalué.

Lorsque ces éléments manquent, Factory ne doit pas inventer silencieusement le
produit. Elle peut proposer des questions, demander une conception ou limiter
le travail à une exploration réversible.

## Le vrai produit : une chaîne de responsabilité

Le résultat visible de Factory peut être une application, une nouvelle
version, un composant, une migration, un pack ou une publication. Son produit
profond est la chaîne qui relie ce résultat à son intention.

Une fabrication complète contient :

1. l’audit de l’état d’entrée ;
2. le périmètre et le plan ;
3. une revue indépendante de la conception lorsque le risque l’exige ;
4. le préflight de chaque phase ;
5. une implémentation dans un espace isolé ;
6. des vérifications techniques ;
7. une revue indépendante ;
8. une vérification contradictoire supplémentaire ;
9. une réconciliation et, si nécessaire, une réparation bornée ;
10. un verdict final rendu selon les gates ;
11. une intention de commit ou de publication confirmée par une personne ;
12. les carries, limites et preuves de fermeture.

Ces étapes ne sont pas toutes des agents. Certaines sont des commandes
déterministes, d’autres des fichiers, des hooks, des reviews ou des décisions
humaines. Le workflow les orchestre sans leur retirer leur autorité propre.

## Planifier sans confondre plan et vérité

Le plan décrit les changements attendus, les fichiers ou surfaces concernés,
les tests, les risques et les dépendances. Il relie chaque exigence à une
preuve future.

Un agent peut aider à le produire. RRV peut retrouver les contrats du dépôt et
les incidents antérieurs. Conception peut fournir les critères. Le plan reste
un artefact à approuver, pas une prophétie.

Le préflight cherche ce qui rendrait le travail dangereux ou incohérent avant
la première modification : état du dépôt, changements non liés, frontières de
sécurité, contrats de protocole, données, tests nécessaires et conditions de
retour en arrière.

Cette discipline est particulièrement importante dans un monde d’agents
rapides. Plus la capacité d’écrire est grande, plus le coût d’une mauvaise
direction peut se multiplier vite.

## Un espace de travail isolé

Factory ne traite pas le dépôt entier comme une feuille blanche. Le travail
prend place dans un espace contrôlé, souvent un worktree ou un environnement
équivalent. Le périmètre autorisé est connu. Les fichiers appartenant à
l’utilisateur ou à un autre chantier ne deviennent pas une matière à
réorganiser.

Les sorties de commandes sont considérées comme des données non fiables. Un
texte présent dans un fichier, un log ou une dépendance ne devient pas une
instruction pour l’agent. Les secrets sont recherchés et exclus. Les
dépendances, licences et chemins sont vérifiés selon la classe du projet.

Pour une application SBFB, Factory prépare aussi le manifeste, les capacités,
la sandbox, la politique de contenu, le bridge et la provenance de source.

## Des rôles qui ne s’auto-valident pas

Une Factory crédible sépare la personne ou l’agent qui implémente de ceux qui
évaluent le résultat.

Le **driver** porte le travail et rassemble les éléments. Les **agents
d’implémentation** écrivent ou modifient. Les **vérificateurs** exécutent les
tests et inspectent les propriétés déterministes. Le **reviewer indépendant**
cherche les écarts par rapport au plan, aux contrats et aux risques. Une revue
contradictoire supplémentaire peut rechercher des manques que le premier
cadre n’a pas vus.

Le driver ne peut pas s’attribuer seul le verdict final. Une revue favorable
peut rester `PASS-PENDING` tant que la vérification indépendante manque. Une
sortie brute contradictoire n’est pas réécrite pour paraître compatible.

S’il existe un écart sérieux, Factory réconcilie les analyses, produit une
réparation bornée, puis repasse par les contrôles requis. Le processus ne
transforme pas automatiquement un échec en avertissement pour avancer.

## Trois familles de vérification

Les détails varient selon le projet, mais Factory distingue utilement trois
blocs.

La **vérification ciblée** couvre le changement : tests unitaires, contrats,
types, comportement et cas d’erreur directement concernés.

La **vérification d’intégration** examine les frontières : shell, daemon,
bridge, runtime, réseau, base, navigateur réel ou plusieurs nœuds lorsque
l’affirmation l’exige.

La **vérification globale** cherche les régressions dans l’ensemble du dépôt :
formatage, lint, suites, build, sécurité de dépendances, taille ou autres gates
du projet.

Un voyant vert ne vaut que pour la commande réellement exécutée dans
l’environnement réellement observé. Un test hermétique ne prouve pas une
interaction réseau. Un test backend ne prouve pas l’accessibilité d’un
parcours navigateur. Une réussite locale ne prouve pas la disponibilité
publique d’une release.

Factory conserve cette portée au lieu d’additionner tous les verts dans un
mot unique.

## Fabriquer une application, pas seulement son code

Lorsqu’elle construit une app, Factory prend en charge un ensemble plus large
que le bundle.

Elle crée ou adapte le projet, maintient le manifeste, vérifie les licences,
prépare le bridge, limite les capacités, produit les chaînes traduisibles,
teste les états d’erreur et génère une prévisualisation dans la sandbox réelle.
Elle relie les composants réutilisés à leur source et à leur version.

Les critères d’accessibilité font partie de la fabrication : structure
sémantique, clavier, focus, zoom, reflow, contraste, mouvement, lecteurs
d’écran et tests avec personnes lorsqu’ils sont requis. Une application qui
fonctionne uniquement dans la prévisualisation visuelle du développeur n’est
pas terminée.

La publication relie ensuite l’archive à un repo, un commit, des digests et des
preuves disponibles. Elle n’affirme pas qu’un tiers indépendant a reproduit le
build si ce travail n’a pas eu lieu.

## Le contrôle humain au bord de l’effet

Un agent peut préparer un commit, un push, un déploiement ou une publication.
Il ne les effectue pas simplement parce que son plan les mentionne.

Factory produit une **intention** liée à l’état exact : HEAD, index, diff,
gates, artefacts, destination et version. Le shell ou l’outil de confiance
présente ces éléments à la personne juste avant l’effet. Si l’état change,
l’intention précédente ne couvre plus le nouvel état.

Cette confirmation humaine n’est pas une méfiance vague envers l’IA. Elle répond à
une propriété simple : certains effets déplacent la responsabilité hors de
l’espace de simulation. Un commit devient une trace partagée ; un push modifie
un dépôt distant ; une publication rend un objet découvrable ; un déploiement
peut toucher des utilisateurs.

Le modèle peut recommander. La personne autorisée engage.

## Fermer sans effacer les limites

La fermeture d’un travail rassemble le verdict, les preuves, les décisions et
les carries.

Un carry est une limite ou une suite explicitement routée. Il ne doit pas
servir à cacher une exigence bloquante. Factory distingue ce qui empêche la
livraison, ce qui limite la portée, ce qui mérite une amélioration future et
ce qui dépend d’une preuve externe.

Le résultat final peut donc être :

- livré dans le périmètre annoncé ;
- partiellement démontré avec une limite claire ;
- bloqué ;
- abandonné ;
- ou conservé comme expérimentation.

Cette pluralité est plus utile qu’une culture où tout finit par être appelé
« terminé ».

## Factory comme workflow fondateur

Factory n’est pas seulement un outil qui exécute des workflows. Son propre
processus est exprimable dans Workflow Hub.

On peut y voir l’audit d’entrée, le plan, les phases, les revues, les gates, les
intentions humaines et la fermeture. Cette capacité est une forme de dogfood :
le système de workflow doit pouvoir porter la fabrication du système lui-même.

Elle introduit aussi une question délicate. Si Factory peut modifier Factory,
comment empêcher une auto-évolution incontrôlée ?

La réponse est le gouvernement par versions.

Une communauté forke `Factory v1`, modifie son profil, évalue les différences,
exécute des runs en parallèle ou en mode shadow, examine les écarts, vérifie un
kill switch et un rollback, puis approuve éventuellement `Factory v2`. Les
nouveaux travaux peuvent utiliser cette version. Les travaux déjà engagés sur
`v1` restent liés à leur plan.

La Factory est donc libre et évolutive, mais elle ne modifie pas rétroactivement
les règles qui devaient la juger.

## Ce que Factory change pour un petit groupe

Une grande organisation peut distribuer les rôles entre chefs de produit,
architectes, développeurs, QA, sécurité, accessibilité, release managers et
auditeurs. Une petite association ou un mainteneur seul ne possède pas cette
équipe.

Factory ne crée pas magiquement les compétences absentes. Elle permet toutefois
de matérialiser leurs fonctions : demander une analyse distincte, exécuter les
contrôles systématiquement, garder une trace, solliciter une revue humaine au
bon moment et réutiliser les processus établis par d’autres.

Le résultat est un **mainteneur augmenté**, pas un mainteneur remplacé.

Cette différence est au cœur du potentiel de Nexus. Les communs échouent
rarement par manque d’idées. Ils échouent parce que la charge de transformer,
vérifier, documenter et maintenir repose sur trop peu de personnes. Factory
abaisse une partie de cette charge et la rend transmissible.

Elle ne garantit pas qu’un commun vivra. Elle lui donne une meilleure chance de
ne pas dépendre entièrement de la mémoire et de l’épuisement de celui qui l’a
commencé.
