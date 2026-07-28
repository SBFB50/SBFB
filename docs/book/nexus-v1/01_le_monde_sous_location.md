# Chapitre 1 — Le monde numérique sous location

Le premier geste de notre vie numérique consiste rarement à créer. Il consiste
à accepter.

Accepter les conditions d’une plateforme, créer un compte, confier une adresse,
choisir un abonnement, envoyer un document sur un serveur que nous ne verrons
jamais. Nous pouvons ensuite écrire, parler, travailler, apprendre, organiser
une entraide ou publier une application. Mais presque chaque pouvoir nous est
prêté. Il dure tant que l’entreprise existe, tant que le prix reste
supportable, tant que notre usage demeure compatible avec ses règles et tant
que notre pays peut encore joindre ses serveurs.

Cette dépendance est devenue si ordinaire qu’elle ne ressemble plus à une
décision d’architecture. Elle ressemble à Internet lui-même.

Pourtant, le réseau n’exige pas qu’un document appartienne à celui qui héberge
le service. Un navigateur n’exige pas qu’une application soit distribuée par
un magasin central. Une intelligence artificielle n’exige pas que chaque
question traverse un centre de données situé à des milliers de kilomètres. Ces
choix se sont imposés parce qu’ils facilitent la croissance, la facturation, la
maintenance et le contrôle. Ils ne sont pas des lois naturelles.

Nexus commence par refuser cette confusion.

## La centralisation est aussi une relation de pouvoir

Un serveur central possède des qualités évidentes. Il permet de coordonner un
état, de corriger rapidement une panne, de modifier une règle pour tout le
monde et d’offrir une expérience homogène. Il est souvent le meilleur choix
pour un petit service bien délimité.

Le problème apparaît lorsque presque toutes les fonctions essentielles sont
réunies au même endroit. Celui qui détient le serveur détient alors aussi la
base des utilisateurs, l’historique, les clés de publication, la capacité de
fermer les comptes, le classement des contenus et parfois la seule copie
utilisable du logiciel. Une décision commerciale devient une décision
sociale. Une panne technique devient une interruption collective. Un
changement de prix devient une perte d’autonomie.

Pour une association, cela peut signifier que des années de travail vivent
dans un outil qu’elle ne peut ni réparer ni déplacer. Pour une école, que
l’apprentissage s’arrête avec la connexion. Pour une communauté locale, que
ses données de terrain remontent vers une entreprise distante. Pour un
développeur, que publier une application signifie d’abord obtenir
l’autorisation d’un intermédiaire. Pour une personne handicapée, qu’une
interface devenue inaccessible ne peut pas être adaptée sans attendre la
bonne volonté du propriétaire.

Le coût n’est donc pas seulement financier. C’est le coût de ne pas pouvoir
continuer par soi-même.

## Trois séparations artificielles

Notre écosystème numérique traite habituellement trois besoins comme des
industries distinctes.

Le premier est la **distribution du logiciel**. Une application doit être
hébergée, référencée, téléchargée et mise à jour. Cette fonction est
habituellement confiée à un magasin, à un site ou à un cloud.

Le deuxième est l’**état partagé**. Une application utile ne se réduit pas à
ses fichiers : elle contient des documents, des observations, des versions,
des décisions et des contributions. Cet état vit le plus souvent dans une
base centrale.

Le troisième est le **calcul**. Traduire un corpus, analyser une image,
rechercher dans des milliers de documents ou faire fonctionner un agent
demande du processeur, parfois un GPU. Ce calcul est généralement loué à une
infrastructure distante.

Pris séparément, chacun de ces problèmes possède de nombreuses solutions. On
peut distribuer des fichiers en pair-à-pair. On peut synchroniser des données
entre machines. On peut faire tourner un modèle en local ou partager une ferme
de calcul. Mais une communauté qui veut fabriquer une application complète
doit encore assembler ces mondes, gérer plusieurs comptes, plusieurs modèles
de confiance et plusieurs points de défaillance.

Nexus fait le pari que leur réunion change la nature de ce que l’on peut
construire.

Une application devient un objet que des pairs peuvent posséder et servir. Son
état public ou partagé peut circuler selon des capacités explicites. Le calcul
peut rester sur la machine de l’utilisateur, utiliser un fournisseur qu’il
autorise ou être confié à des travailleurs du réseau qui ont eux-mêmes
consenti. Le tout s’ouvre depuis un nœud local qui agit au nom de la personne,
pas au nom d’une plateforme distante.

## Sans compte ne veut pas dire sans identité

Le modèle centralisé associe souvent identité et compte. Exister dans le
système signifie avoir une ligne dans la base de son propriétaire. Perdre le
compte signifie perdre l’accès à son histoire.

Dans Nexus, l’identité fondamentale prend la forme d’une paire de clés détenue
localement. Elle permet de signer une publication, une contribution ou une
rotation de clé. Elle ne garantit pas que son détenteur est digne de confiance,
mais elle permet de retrouver une continuité : tel objet a été annoncé par la
même identité, telle nouvelle version succède à telle ancienne, tel reçu
correspond à telle action.

Cette identité n’a pas besoin de contenir un nom civil, une appartenance ou un
profil universel. Mais la V1 utilise une clé technique stable : les actions
publiques signées avec cette même clé restent corrélables, y compris entre
plusieurs projets. Le registre public de kudos renforce cette possibilité de
corrélation. La V1 minimise les champs révélés ; elle ne garantit pas
que les activités d’un même nœud soient impossibles à relier entre contextes.
Des identifiants de contribution rotatifs par projet relèvent d’une évolution
ultérieure.

Cette limite est décisive. Un réseau sans compte central peut encore devenir
une machine de surveillance si ses traces sont agrégées sans retenue. Nexus ne
gagne sa souveraineté qu’en combinant l’attribution utile, la minimisation des
données et une information honnête sur ce qui demeure publiquement
corrélable.

## Sans administrateur central ne veut pas dire sans règle

Une autre confusion accompagne souvent les systèmes décentralisés : si
personne ne peut supprimer un objet pour tout le réseau, alors personne ne
pourrait organiser ce qu’il voit.

Or la curation et la censure ne sont pas la même opération.

Une communauté peut publier une liste d’applications qu’elle recommande. Une
personne peut suivre plusieurs curateurs, lire leurs chartes, comparer leurs
choix ou cesser de les suivre. Un espace local peut masquer un contenu qui
enfreint ses règles. Rien de cela n’exige qu’une autorité efface l’objet chez
tous les autres.

Ce déplacement rend les désaccords plus visibles. Deux personnes peuvent voir
des sélections différentes. Une application peut être déconseillée par un
curateur et recommandée par un autre. Il n’existe plus un classement unique
dont l’origine disparaît derrière l’interface. Cette pluralité demande plus
d’effort au lecteur, mais elle lui rend aussi le choix de ses médiations.

## Le cloud n’est pas remplacé par un romantisme du local

La machine personnelle n’est pas toujours disponible, assez puissante ou bien
administrée. Un modèle local peut être trop lent. Une petite association peut
préférer un service hébergé. Un groupe de recherche peut disposer d’un cloud
souverain correctement gouverné. Nexus n’interdit pas ces choix.

Il refuse seulement qu’ils soient implicites et irréversibles.

Un workflow peut placer certaines étapes sur le nœud local, d’autres chez un
fournisseur autorisé et d’autres sur le réseau SBFB. Avant l’exécution, la
personne voit quelles données quittent sa machine, vers quelle destination et
pour quel coût. Une politique locale peut interdire le cloud pour un document
sensible tout en l’autorisant pour une recherche publique. La souveraineté ne
consiste pas à tout faire seul ; elle consiste à pouvoir choisir les
dépendances que l’on accepte.

## La première liberté : continuer

Le projet pourrait être résumé par une formule spectaculaire — applications
pair-à-pair, IA distribuée, workflows multi-agents. Mais sa promesse la plus
profonde est peut-être plus simple.

Une application importante ne devrait pas disparaître parce qu’un magasin la
retire. Une communauté ne devrait pas perdre ses outils parce qu’un abonnement
augmente. Un groupe ne devrait pas remettre ses données les plus sensibles à
un réseau public pour bénéficier d’une collaboration. Un travail ne devrait
pas devenir incompréhensible parce que l’agent qui l’a produit ne se souvient
plus de ses sources. Une personne ne devrait pas être enfermée dans une
interface qui lui refuse l’accès à cause de sa façon de voir, de lire ou
d’agir.

Nexus cherche à rendre possible une continuité : continuer à ouvrir, à
comprendre, à vérifier, à modifier, à forker et à transmettre.

Cette continuité ne naît pas d’un slogan décentralisé. Elle exige un ensemble
de mécanismes qui se tiennent mutuellement : objets adressés par leur contenu,
signatures, sandbox, capacités, formats portables, plans versionnés, preuves
lisibles, stockage local, réplication choisie, curation pluraliste et droit
effectif au fork.

Le reste du livre raconte comment ces mécanismes deviennent non pas une
collection de technologies, mais une fabrique commune.
