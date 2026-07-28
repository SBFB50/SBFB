# Chapitre 13 — Ce que Nexus refuse

Une plateforme se définit autant par ce qu’elle rend possible que par ce
qu’elle refuse de devenir. Les limites de Nexus V1 ne sont pas toutes des
fonctionnalités en attente. Certaines sont des choix de conception destinés à
préserver l’autonomie des utilisateurs.

Elles peuvent être discutées et améliorées, mais elles ne doivent pas être
contournées sous prétexte qu’un modèle plus puissant, une communauté plus
nombreuse ou une situation urgente promettrait de « mieux savoir » à la place
des personnes.

Ces refus ne font pas disparaître le risque. Ils empêchent surtout Nexus
d’accumuler certains pouvoirs dont il ne pourrait plus garantir l’usage.

## Nexus ne produit pas de vérité globale

RRV recherche des sources, relie des objets, expose des signatures et permet
de suivre des curateurs. Il ne maintient pas un registre central du vrai et du
faux.

Une telle vérité globale demanderait une autorité finale : une institution,
une majorité, un algorithme ou une IA capable de trancher pour tous. Même bien
intentionnée, cette autorité serait vulnérable aux erreurs partagées, aux
changements de contexte et à la capture.

La V1 préfère représenter plusieurs niveaux : fait déclaré, document source,
interprétation, contestation, provenance et degré de vérification. Une
affirmation peut être solidement documentée sans devenir incontestable. Une
source peut être authentique tout en étant incomplète. Une communauté peut
publier une conclusion et une autre conserver un désaccord argumenté.

Une signature prouve une attribution ou l’intégrité d’un objet, pas la vérité
de son contenu. Une Proof Card explique des signaux de preuve ; elle n’est ni
un certificat absolu ni une note morale. Nexus aide à construire un jugement ;
il ne remplace pas le jugement.

## Nexus ne construit pas une infrastructure de surveillance

Le fonctionnement d’une communauté produit de nombreuses traces : recherches,
lectures, soutiens, contributions, exécutions, relations entre personnes et
lieux de connexion. Agrégées, ces traces peuvent révéler bien davantage que ce
que chacun pensait partager.

La V1 refuse donc le graphe comportemental universel. Elle ne cherche pas à
connaître l’ensemble des lectures, des relations ou des déplacements d’un
utilisateur pour personnaliser son expérience. Les journaux techniques sont
limités à une finalité précise, conservés pour une durée maîtrisée et placés
sous le contrôle du contexte qui les produit.

Les preuves d’exécution ne deviennent pas un dossier général sur la personne.
Les statistiques utiles sont agrégées lorsque cela suffit. Les espaces privés
ne publient pas leurs listes de membres. Une identité peut prouver un droit
sans exposer toutes les communautés auxquelles elle participe.

Le pair-à-pair n’est pas une excuse pour diffuser davantage. Au contraire,
puisque la réplication complique le retrait, la décision de publier doit être
plus stricte.

## Nexus ne fait pas de microciblage politique ou psychologique

Il serait techniquement tentant d’utiliser l’Atelier des communs et RRV pour
déduire les convictions, les fragilités ou les sujets capables de persuader
chaque personne. Workflow Hub pourrait ensuite adapter automatiquement les
messages, et Factory déployer les interfaces correspondantes.

Nexus refuse cette chaîne.

Les préférences exprimées pour participer à un projet ne servent pas à
fabriquer un profil psychologique secret. Les données d’une communauté ne sont
pas croisées avec celles d’une autre pour deviner des opinions. Un message
collectif ne change pas silencieusement d’argument selon les vulnérabilités
supposées de son destinataire.

Cette limite n’interdit pas l’accessibilité, la traduction, le choix d’une
langue ou l’adaptation volontaire d’une interface. Elle distingue l’aide
demandée par l’utilisateur de la persuasion cachée exercée sur lui.

Une communication peut être adressée à un groupe qui a choisi de la recevoir.
Sa source, sa finalité et son responsable restent visibles. L’optimisation
opaque de l’influence individuelle n’est pas une capacité de la V1.

## L’IA n’est souveraine sur aucun effet

Une IA peut produire une réponse impressionnante et néanmoins se tromper sur
un nom, une destination, une règle ou le sens d’une demande. Elle peut aussi
être influencée par un contenu hostile présent dans les documents qu’elle lit.

La V1 ne lui permet donc pas :

- de s’accorder une nouvelle capacité ;
- d’étendre seule le graphe d’un workflow validé ;
- de choisir une destination sensible non prévue ;
- d’augmenter un budget ;
- de résoudre ou révéler un secret hors du contexte autorisé ;
- d’approuver sa propre action ;
- de déclarer qu’un effet a réussi sans preuve suffisante.

Le modèle reste un composant de composition et d’assistance. Les contrats
typés, le runtime Rust et le consentement humain constituent l’autorité. Même
un modèle local, exécuté sur la machine de l’utilisateur, ne devient pas
souverain : la souveraineté locale appartient à la personne, pas au logiciel
qui tourne près d’elle.

## Un workflow ne se réécrit pas pendant qu’il agit

La capacité d’un agent à adapter son plan semble attractive. Elle devient
dangereuse lorsqu’elle permet de modifier en cours d’exécution les étapes, les
destinataires ou les permissions sur lesquelles l’utilisateur avait fondé son
consentement.

Nexus fige donc le plan compilé d’un run. Les données et certains choix prévus
peuvent varier ; la structure autorisée du workflow ne se transforme pas
silencieusement.

Si une meilleure stratégie apparaît, elle produit un nouveau brouillon.
Celui-ci peut être comparé, validé et promu comme nouvelle version pour une
exécution suivante. En cas d’urgence, le run peut être arrêté, puis remplacé
explicitement. Il n’existe pas de raccourci où l’agent devient son propre
auteur, son propre contrôleur et son propre approbateur.

Factory évolue de la même manière : fork, modification, test, approbation,
nouvelle version. Sa liberté vient de sa capacité à évoluer entre les cycles,
non d’une mutation incontrôlée au milieu d’un travail.

## Les données sensibles ne vont pas sur le réseau public

Le chiffrement est indispensable, mais il ne rend pas toute publication
raisonnable. Un objet chiffré et largement répliqué peut rester disponible
pendant des années. Ses métadonnées, sa taille, ses moments d’accès ou ses
relations avec d’autres objets peuvent déjà révéler des informations.

La V1 refuse donc de placer sur le réseau public les secrets, données
d’adhésion, carnets d’adresses, dossiers personnels, conversations
confidentielles, prompts sensibles ou artefacts permettant d’identifier des
personnes vulnérables.

Le réseau public transporte des objets destinés à être publics :
applications, packs, documents publiés, preuves minimisées et annonces
nécessaires à leur découverte. Les espaces privés utilisent des canaux, des
capacités et des politiques distincts. Lorsque seule une preuve d’existence
est nécessaire, Nexus publie une empreinte ou un engagement approprié plutôt
que la donnée elle-même.

La règle reste compréhensible sans expertise cryptographique : si la diffusion
durable de l’objet pourrait nuire à une personne, cet objet n’appartient pas au
réseau public.

## Une limite n’est pas une promesse magique

Ces refus réduisent le pouvoir du système ; ils ne rendent pas les personnes
infaillibles. Un auteur peut publier une information fausse sous son vrai nom.
Une communauté peut choisir de mauvais curateurs. Un administrateur peut
abuser de ses droits locaux. Un utilisateur peut accorder une capacité qu’il
n’a pas comprise. Un logiciel peut contenir une faille.

La V1 répond à ces risques par plusieurs couches : minimisation des pouvoirs,
provenance, isolation, versions, reçus, possibilité d’appel, pluralité des
sources et droit de partir. Aucune couche n’est parfaite. Leur combinaison
cherche à éviter qu’une seule erreur suffise à tout compromettre.

Cette honnêteté est peut-être le refus le plus important. Nexus ne promet ni
une société sans conflit, ni une confiance entièrement automatisée, ni une
intelligence artificielle capable de gouverner à notre place.

Il propose une infrastructure où l’on peut coopérer à grande échelle tout en
continuant à demander : qui parle, sur quelle base, avec quel pouvoir, sous
quelle version, et comment puis-je dire non ?

Ces refus ne prennent leur sens que lorsqu’une société réelle éprouve le
système : projets urgents, communautés nombreuses, appartenances fortes et
besoins contradictoires. La dernière partie quitte donc l’architecture pour
observer ce que Nexus pourrait permettre — et ce que son succès pourrait
menacer.
