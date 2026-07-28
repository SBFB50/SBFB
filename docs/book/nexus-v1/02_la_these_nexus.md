# Chapitre 2 — La thèse Nexus

Nexus n’est pas d’abord une application. C’est une chaîne de transformation.

Une personne rencontre une difficulté. Elle la formule sans devoir connaître
la solution. D’autres l’aident à distinguer le besoin, les hypothèses et les
contraintes. Des sources sont retrouvées et qualifiées. Plusieurs réponses
sont comparées. Une décision humaine choisit ou refuse une voie. Cette décision
devient une conception explicite, puis un workflow, puis un travail de
fabrication. L’application ou l’artefact obtenu est publié avec son histoire.
Les usages, les erreurs et les corrections alimentent ensuite une nouvelle
boucle.

On peut dessiner cette thèse en une ligne :

```text
besoin humain
  → Atelier des communs
  → RRV
  → Conception
  → Workflow Hub
  → Factory
  → réseau SBFB
  → usages, preuves et retours
  → nouveau besoin mieux compris
```

Cette ligne n’est pas une chaîne rigide. Une petite correction peut aller
directement vers Factory. Une recherche peut rester dans l’Atelier des communs
sans devenir un logiciel. Un workflow peut produire un rapport plutôt qu’une
application. Conception n’est pas obligatoire avant chaque travail. Ce qui
compte est la disponibilité de passages explicites entre les étapes, avec des
artefacts que les personnes et les machines peuvent comprendre.

## De l’idée au commun

Les plateformes d’idées commencent souvent par une page blanche et finissent
par un classement. Chacun propose, les autres votent, puis les idées les plus
populaires remontent. Ce mécanisme mesure une réaction. Il ne montre pas si le
problème est bien posé, si une équipe en prendra soin, si une solution existe
déjà ou si les personnes concernées ont été entendues.

L’Atelier des communs traite une idée comme une hypothèse vivante. Il la
capture avant de la juger, la clarifie, lui laisse parfois un temps
d’incubation, recherche les travaux antérieurs, produit des options
divergentes et organise une critique indépendante. Il demande ensuite une
décision humaine : abandonner, poursuivre la recherche, lancer un pilote,
préparer une conception ou fabriquer.

L’artefact important n’est pas un nombre de votes. C’est un ensemble
transmissible : le problème, les personnes concernées, les preuves, les
objections, les choix rejetés, les risques, le porteur responsable et la
prochaine action.

Une idée devient un commun non lorsqu’elle appartient à tout le monde de
manière abstraite, mais lorsque d’autres peuvent la reprendre sans perdre la
mémoire de ce qui a déjà été appris.

## De l’information à la preuve située

RRV signifie **Recherche Réseau Vérifiable**. Son rôle n’est pas de remplacer
un moteur de recherche par une IA bavarde. Il doit permettre de trouver un
objet et de comprendre ce que l’on sait réellement à son sujet.

RRV cherche dans plusieurs espaces dont les frontières restent visibles :
l’application ouverte, les objets connus du protocole, les sources et le code
publiés, un corpus privé explicitement autorisé ou, si la personne le demande,
le Web externe. Une synthèse peut croiser ces mondes, mais elle ne fusionne pas
leurs niveaux de confiance.

Chaque affirmation peut être qualifiée :

- **lue**, lorsqu’elle apparaît directement dans une source ;
- **déduite**, lorsqu’elle vient d’une analyse ou d’un rapprochement ;
- **vérifiée**, lorsqu’un test, une signature, un hash, un build ou une autre
  procédure reproductible soutient précisément l’affirmation ;
- **non vérifiée**, lorsque la preuve manque ou dépend d’une déclaration
  externe.

Cette grammaire semble modeste. Elle change pourtant la relation au résultat.
La réponse n’efface plus le chemin qui l’a produite. Le lecteur peut ouvrir la
source, inspecter une version, constater une contradiction et choisir une
action bornée : vérifier une provenance, comparer deux branches, forker un
workflow, demander une nouvelle recherche ou renoncer.

## De la décision à la conception

Entre « cette idée paraît bonne » et « construisons-la » se trouve un espace
souvent comprimé : la conception.

Conception reçoit une décision acceptée et la confronte à plusieurs regards.
L’architecture produit examine les parcours et les états. L’UX interroge les
personnes, la compréhension et l’accessibilité. La sécurité cherche les abus,
les fuites et les pouvoirs excessifs. L’analyse du protocole vérifie ce qui
peut rester applicatif et ce qui modifierait les frontières communes.

Ces analyses se déroulent en parallèle pour éviter qu’un premier récit
n’enferme toutes les autres. Elles se relisent ensuite de manière
contradictoire. Les arbitrages humains sont conservés avec les alternatives
rejetées.

Le résultat, le `ConceptionPack`, n’est pas une longue prose que Factory devra
interpréter librement. Il contient des contrats : flux, états, données,
capacités, menaces, exigences d’accessibilité, décisions d’architecture,
critères d’acceptation, inconnues et risques résiduels. Il rend le passage vers
la fabrication plus précis sans prétendre supprimer le jugement.

## De l’organisation implicite au workflow visible

Une communauté possède toujours des workflows, même lorsqu’elle ne les appelle
pas ainsi. « Quelqu’un lit les demandes, demande l’avis de Marie, envoie le
document au juriste, attend le feu vert, puis Paul publie » est déjà un
workflow. Il vit simplement dans les habitudes, les conversations et la
mémoire de quelques personnes.

Workflow Hub permet de le rendre visible, testable et versionné.

On peut partir d’un modèle, décrire l’intention en français, composer des
agents, relier des étapes, préciser les données, limiter les boucles, choisir
les outils et placer des validations humaines. Quatre voies de création
donnent accès au même modèle : un canvas spatial, un plan textuel structuré, un
mode guidé étape par étape et une conversation qui produit des modifications
déclaratives inspectables. Aucune n’est une version secondaire.

Le Hub n’est toutefois pas le moteur souverain. Il produit des intentions et
des définitions. Le compilateur Rust vérifie les invariants et fige un plan
immuable. Le shell de confiance présente les demandes, résout les secrets et
recueille les confirmations sensibles. Le daemon et le runtime recalculent les
capacités effectives, exécutent et journalisent. Cette séparation permet
d’offrir une interface riche sans transformer une application sandboxée en
administrateur de la machine.

## De la conception au logiciel vérifiable

Factory est la fabrique. Elle reçoit un besoin direct, un `DecisionPack` ou un
`ConceptionPack`, puis organise le travail nécessaire pour produire un
artefact livrable.

Le mot *fabrique* pourrait laisser croire à un bouton « génère mon app ».
Factory est presque l’inverse. Elle donne une forme durable à tout ce qui rend
une fabrication digne de confiance : plan, périmètre, environnement isolé,
implémentation, tests, revues indépendantes, réconciliation, preuves,
validation humaine des effets sensibles, provenance et possibilité de retour
en arrière.

Des agents peuvent écrire, analyser et corriger. Ils ne peuvent pas s’attribuer
le verdict final, engager un commit sensible ou publier silencieusement. Un
succès n’existe que lorsque les gates correspondantes ont réellement été
exécutées et que la décision humaine requise a été rendue.

Factory peut elle-même évoluer. Une équipe peut forker son processus dans
Workflow Hub, l’évaluer, le faire approuver et l’utiliser lors d’un cycle
suivant. Mais le processus d’une exécution en cours ne se transforme pas sous
ses propres pieds. La liberté d’évolution est assurée par les versions, pas
par une improvisation incontrôlée.

## Du produit isolé au réseau

L’artefact terminé rejoint SBFB sous une forme distribuable. Ses fichiers sont
adressés par leur contenu. Sa provenance relie la publication à une source, à
une version et à une identité. Les nœuds intéressés peuvent le récupérer et le
servir localement. L’application s’ouvre dans un bac à sable et demande ses
capacités au moyen d’un pont contrôlé.

Les données ne suivent pas toutes le même chemin. Une archive publique peut
être largement répliquée. Un corpus collaboratif peut être partagé avec une
capacité de lecture ou d’écriture précise. Une base locale et privée reste
locale. RRV reçoit des projections publiques filtrées, jamais un droit
magique de parcourir toutes les bases.

Le calcul suit la même logique. Une tâche peut rester locale, utiliser un
service autorisé ou être confiée au réseau. Le travailleur choisit les projets
qu’il accepte, les ressources qu’il consacre et les limites qu’il impose. Un
résultat signé indique qui l’a produit et sur quels objets il porte ; il ne
prouve pas à lui seul que le calcul était correct.

## Le rôle de la boucle

La véritable puissance de Nexus n’apparaît pas lors de la première
publication. Elle apparaît lorsque le résultat revient dans la boucle.

Une application produit des incidents, des usages inattendus, des corrections
et de nouveaux besoins. RRV peut retrouver leur histoire. L’Atelier des
communs peut distinguer une anomalie locale d’un problème général. Conception
peut comparer plusieurs remèdes. Workflow Hub peut faire évoluer le processus.
Factory peut produire une nouvelle version dont la provenance reste reliée à
la précédente.

Au lieu d’un produit figé livré à des utilisateurs passifs, on obtient une
capacité collective d’apprentissage.

Cette boucle n’est pas automatiquement vertueuse. Elle peut tourner à vide,
produire trop de documents, épuiser ses responsables ou amplifier une culture
dominante. Les chapitres suivants insisteront donc autant sur les porteurs, les
refus et la curation que sur les moteurs d’agents.

La thèse Nexus n’est pas « davantage d’automatisation produit davantage de
bien ». Elle est celle-ci :

> rendre le chemin de l’intention à l’action explicite, portable, contestable
> et vérifiable augmente la possibilité de construire des communs sans remettre
> leur existence à une plateforme centrale.
