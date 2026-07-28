# Chapitre 5 — RRV, chercher sans fabriquer la vérité

Une réponse peut être exacte et impossible à croire. Elle peut aussi être
fausse et extraordinairement convaincante.

Les interfaces conversationnelles ont rendu cette tension quotidienne. Nous
posons une question et recevons une synthèse fluide, parfois accompagnée de
liens. La facilité de lecture masque souvent la difficulté réelle : quelles
sources ont été consultées ? La réponse reproduit-elle un texte, en déduit-elle
quelque chose ou rapporte-t-elle une affirmation externe ? Une version plus
récente existe-t-elle ? Le résultat concerne-t-il mon application locale, un
objet du réseau ou le Web en général ?

RRV — Recherche Réseau Vérifiable — est la réponse de Nexus à ce problème.
Son ambition n’est pas de tout savoir. Elle est de conserver la nature des
preuves pendant la recherche et l’explication.

## Une recherche avec un lieu

Toute requête RRV possède un périmètre visible.

`@current` désigne le contexte ouvert : l’application, l’erreur ou l’objet que
la personne regarde. `@protocole` ou `@network` couvre les applications, feeds,
provenances, curations et projections connues du nœud. `@dev` permet
d’inspecter un dépôt, des symboles, des tests, des schémas et un processus
explicitement autorisés. `@web` ouvre une recherche externe volontaire. Un
espace privé peut être interrogé seulement lorsqu’une capacité dédiée le rend
accessible.

Le périmètre n’est pas un détail réservé aux experts. Il change le sens de la
réponse.

« Cette application ne stocke aucune donnée personnelle » peut vouloir dire que
son manifeste ne déclare pas de clé sensible, que son code a été inspecté, que
ses données locales ont été examinées avec consentement, ou simplement que son
auteur l’affirme sur un site. RRV refuse d’aplatir ces quatre situations.

Lorsqu’une synthèse croise plusieurs espaces, chaque partie conserve son
origine. Une source Web ne devient pas une preuve protocolaire parce qu’elle
est citée à côté d’un hash vérifié.

## Quatre verbes pour ne pas tricher

RRV qualifie ses affirmations avec un petit vocabulaire.

**Lu** signifie que l’information apparaît directement dans un objet accessible
: fichier, manifeste, événement, archive, schéma ou source. Lire prouve la
présence d’un énoncé, pas sa justesse.

**Déduit** signifie qu’une analyse relie plusieurs éléments ou tire une
conclusion qui n’est écrite nulle part telle quelle. La déduction peut être
très solide, mais elle doit pouvoir être discutée.

**Vérifié** signifie qu’une procédure définie soutient précisément
l’affirmation : signature valide, empreinte identique, test passé, build
reproduit, propriété déterministe confirmée ou quorum prévu. La portée de la
vérification reste limitée à ce qui a réellement été testé.

**Non vérifié** désigne une hypothèse, une déclaration externe ou une
affirmation pour laquelle le lien de preuve manque.

Le système peut afficher d’autres nuances — obsolète, source seulement,
résultat local, validation humaine — mais il ne doit jamais sacrifier cette
séparation à la fluidité d’une réponse.

## De la liste de résultats au dossier vérifiable

Une recherche classique optimise souvent un ordre. RRV commence par expliquer
pourquoi un objet apparaît.

Un résultat peut montrer son auteur technique, sa version, sa licence, sa date,
son digest, ses sources, les curateurs qui le recommandent ou le déconseillent,
les capacités qu’il demande et les limites connues. Le score de pertinence ne
se confond pas avec un signal de preuve. Un document très pertinent peut être
mal attesté ; un objet parfaitement signé peut être hors sujet.

La fiche d’une application rassemble ensuite son dossier :

- l’archive et son empreinte ;
- la source et le commit auxquels la publication se relie ;
- les manifests et capacités déclarées ;
- les versions, forks et événements connus ;
- les tests ou audits publiés ;
- les curations et désaveux ;
- la provenance de fabrication lorsqu’elle existe ;
- les risques et informations manquantes.

Ce dossier n’est pas un certificat statique. Il évolue à mesure que le nœud
reçoit de nouvelles annonces, qu’une source devient obsolète, qu’un audit est
publié ou qu’une version est révoquée.

## La Proof Card comme explication

La Proof Card condense le dossier dans une surface lisible. Elle peut réunir
des facteurs de provenance, de fraîcheur, de licence, de source et de curation.
Elle possède une version de formule afin que son calcul ne change pas
silencieusement.

Son nom peut inciter à la surestimer. Une Proof Card ne dit pas « cette
application est sûre » ni « cette proposition est vraie ». Elle répond à des
questions plus petites :

- la source annoncée est-elle présente ?
- l’archive correspond-elle au digest attendu ?
- la signature disponible est-elle valide ?
- la version est-elle récente selon les informations connues ?
- quels curateurs ont publié un avis ?
- quels éléments manquent ?

Le score éventuel est moins important que les facteurs bruts et leur
explication. L’utilisateur doit pouvoir comprendre pourquoi la carte change et
désactiver une lecture qui ne correspond pas à son contexte.

## Inspecter sans entrer par effraction

RRV peut devenir très puissant lorsqu’il recherche dans le code, les schémas et
les données publiées par des applications. Cette puissance repose sur un
contrat, jamais sur l’idée que « tout est dans le réseau ».

Le frontend d’une application peut être inspecté lorsque l’archive ou le source
pack est public. Le backend peut être décrit à partir d’un dépôt, d’un schéma
d’API, de routes déclarées ou de journaux expurgés. Une base de données peut
publier sa structure, ses migrations et certaines statistiques sans publier
ses lignes.

Les secrets, messages privés, identités, brouillons, historiques personnels et
données chiffrées restent hors index par défaut. Une inspection privée demande
une capacité explicite, une portée limitée, une journalisation et une
révocation possible.

RRV n’accède donc pas à « la base Iroh ». Iroh réplique des espaces auxquels
des capacités donnent accès. RRV consomme des projections publiques ou des
espaces de lecture qui lui ont été expressément ouverts. Son index local peut
utiliser SQLite et la recherche plein texte sans transformer le réseau entier
en base relationnelle partagée.

## Rechercher les processus, pas seulement les produits

Deux applications visuellement identiques peuvent avoir des histoires très
différentes. L’une est une archive publiée sans source. L’autre relie son
besoin, sa conception, son code, ses tests, ses revues et son build.

RRV sait donc inspecter le processus lorsqu’il est publié. Pour une application
issue de Factory, il peut retrouver le plan, les décisions, les résultats de
gates, les reviews, les limites et les artefacts. Il devient possible de
demander non seulement « que fait ce logiciel ? », mais « pourquoi cette
version a-t-elle été considérée comme livrable ? ».

Cette visibilité n’accorde pas à RRV le droit de prononcer le verdict. Le
verdict appartient au processus qui l’a produit et aux personnes autorisées.
RRV le restitue, l’explique et vérifie l’intégrité des éléments disponibles.

## Une recherche qui mène à des actions bornées

Trouver n’est pas toujours suffisant. RRV peut proposer :

- ouvrir une application ou une source ;
- comparer deux versions ;
- recalculer une empreinte ;
- vérifier une provenance ;
- lancer un audit ou une évaluation autorisés ;
- installer un pack désactivé ;
- forker une application ou un workflow ;
- transmettre un composant à Conception ou Factory ;
- demander un calcul de vérification au réseau.

Chaque action décrit la capacité requise, les données envoyées, le lieu
d’exécution, le coût possible, l’effet attendu et la possibilité de retour en
arrière. RRV ne passe pas d’une phrase à une action sensible dans le silence.

Une suggestion reste une intention. Le shell ou l’outil autorisé rend le
consentement et le runtime exécute.

## Les résultats distants restent des annonces

Les nœuds peuvent publier un `SearchManifest` indiquant ce qu’ils savent
rechercher : domaines, types d’objets, langues ou capacités. Cette annonce aide
à découvrir une source de recherche sans envoyer toutes les requêtes de
l’utilisateur à un centre.

Le partage est volontaire, borné, signé et expirant. Le manifest ne contient
pas de données privées. Une réponse distante reste marquée comme résultat
annoncé jusqu’à ce que les objets, sources ou preuves puissent être vérifiés
localement.

Cette distinction protège contre une nouvelle centralisation : le nœud qui
répond le plus vite ou possède le plus grand index ne devient pas
automatiquement l’oracle du réseau.

## Un outil contre le faux consensus

RRV ne peut pas empêcher une communauté de se tromper. Plusieurs curateurs
peuvent répéter la même erreur. Des workers peuvent partager la même faiblesse.
Une source officielle peut être fausse. Un test peut ne pas couvrir le cas qui
compte.

Mais RRV peut empêcher certaines confusions de devenir invisibles. Il peut
montrer que cinq articles se citent mutuellement, qu’une synthèse vient du même
modèle, qu’un chiffre externe n’a pas de preuve locale, qu’un résultat a été
signé mais non reproduit, ou qu’une objection minoritaire existe encore.

Son potentiel ne vient donc pas d’une omniscience supérieure. Il vient d’une
discipline : préserver les désaccords, les portées et les niveaux de preuve
jusqu’à la réponse finale.

Dans un réseau humaniste, la recherche ne doit pas demander à chacun de croire
la même chose. Elle doit donner à chacun de meilleurs moyens de comprendre ce
qu’il choisit de croire, d’utiliser ou de contester.

