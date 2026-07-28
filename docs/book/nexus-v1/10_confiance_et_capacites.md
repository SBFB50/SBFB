# Chapitre 10 — Prouver, autoriser, consentir

Un réseau décentralisé ne devient pas digne de confiance simplement parce
qu’il n’a pas de serveur central. Il peut déplacer le pouvoir sans le rendre
visible. Il peut remplacer une administration identifiable par quelques
mainteneurs, quelques clés cryptographiques ou un algorithme de classement
que personne ne comprend.

Nexus part donc d’une question plus exigeante : non pas « comment supprimer la
confiance ? », mais « comment savoir à qui, à quoi et jusqu’où nous faisons
confiance ? »

Dans la vie courante, ce mot recouvre des choses différentes. On peut faire
confiance à une personne pour tenir une promesse, à un logiciel pour ne pas
modifier un fichier, à une organisation pour appliquer ses règles ou à une
source pour décrire correctement un événement. Confondre ces formes de
confiance produit beaucoup de malentendus. Nexus les sépare.

Une signature cryptographique peut montrer qu’un document a été publié par
une certaine identité. Une empreinte peut montrer que le document téléchargé
est identique à celui qui a été annoncé. Un reçu d’exécution peut montrer
qu’une activité a été demandée, autorisée puis observée. Aucun de ces éléments
ne prouve, à lui seul, que le contenu est vrai, que son auteur est bienveillant
ou que le résultat est utile.

La confiance ne disparaît donc pas. Elle devient plus précise, plus limitée et
plus facile à contester.

## Une chaîne de provenance, pas un certificat de vérité

Lorsqu’une application, une idée documentée ou un workflow est publié dans
Nexus, il reçoit une identité liée à son contenu. Si un seul caractère change,
cette identité change également. L’auteur signe l’objet publié et la version
annoncée. Il devient alors possible de répondre à plusieurs questions simples :

- qui a signé cet objet ;
- quel contenu exact a été signé ;
- quelle version a été installée ou exécutée ;
- si le contenu reçu a été modifié depuis sa publication ;
- quels objets antérieurs ou quelles sources sont cités.

RRV rend ces éléments lisibles. Il ne se contente pas d’afficher un badge
abstrait. Il présente une Proof Card qui explique la provenance déclarée, les
signatures disponibles, les liens vers les sources, l’historique des versions
et les limites connues. L’utilisateur peut ainsi distinguer un document
anonyme, une publication signée par une identité nouvelle, un travail repris
par plusieurs curateurs ou une version dont la construction a été vérifiée par
des tiers.

La progression de la confiance reste volontairement graduelle. Une identité
stable vaut mieux qu’un contenu sans origine, mais elle ne vaut pas une
vérification indépendante. Une source publiée vaut mieux qu’une affirmation
sans référence, mais elle ne garantit pas la qualité de l’interprétation.
Plusieurs attestations concordantes renforcent un dossier, mais elles peuvent
encore partager la même erreur.

Nexus ne transforme donc jamais la provenance en vérité automatique. Il rend
seulement le chemin de l’affirmation plus visible.

Cette modestie est essentielle dans l’Atelier des communs (Ideas Hub). Une
proposition populaire n’y devient pas vraie parce qu’elle recueille beaucoup
de soutien. Elle devient visible comme proposition soutenue par certaines
personnes, accompagnée de certains arguments et reliée à certaines sources.
Les objections, variantes et incertitudes restent attachées au même dossier.
RRV permet ensuite de rechercher cet ensemble sans réduire le débat à un score
unique.

## Des pouvoirs exprimés avant l’action

La provenance répond à la question « d’où vient cet objet ? ». Les capacités
répondent à une autre question : « qu’a-t-il le droit de faire ici ? »

Une application ou un workflow n’obtient pas de pouvoirs simplement parce
qu’il a été installé. Il déclare les capacités dont il a besoin : lire un
dossier déterminé, appeler un service précis, publier un artefact, utiliser un
modèle local ou envoyer un message vers une destination connue. Cette
déclaration n’est qu’une demande.

Le pouvoir réellement disponible résulte de l’intersection de plusieurs
limites :

1. ce que le pack a déclaré ;
2. ce que l’utilisateur ou l’organisation a accordé ;
3. ce que la politique locale autorise ;
4. ce que le runtime sait exécuter en sécurité ;
5. les contraintes propres à l’activité, comme une destination, un budget ou
   une durée.

Si l’une de ces limites manque, l’effet n’est pas autorisé. Une formulation
générale comme « accès au réseau » ne donne pas un droit illimité de contacter
n’importe quel service. Une capacité est typée et bornée. Elle décrit autant
que possible le verbe, la ressource, la destination et le contexte.

Imaginons un workflow destiné à organiser une collecte solidaire. Il a besoin
de lire un répertoire contenant des listes de matériel, de produire une
synthèse et de publier cette synthèse dans l’espace du projet. Cela ne lui
donne pas le droit de lire les carnets d’adresses personnels, d’envoyer des
messages aux participants ni de publier sur le réseau public. Chacun de ces
effets nécessite une capacité distincte.

Le workflow peut être parfaitement légitime et néanmoins demander trop de
pouvoirs. L’interface permet alors de comprendre le désaccord : « cette
version demande l’accès à telle destination, mais votre politique locale ne
l’accorde pas ». Le refus n’est pas un plantage mystérieux ; il fait partie du
fonctionnement normal.

## Trois autorités qui ne se remplacent pas

Dans Workflow Hub, l’intelligence artificielle aide à décrire une intention,
propose un graphe d’activités, reformule une consigne ou signale une
incohérence. Elle n’est pas l’autorité d’exécution.

La V1 répartit les rôles :

- le modèle compose et explique ;
- le compilateur Rust vérifie les contrats et les invariants ;
- l’humain consent lorsque l’effet engage une ressource, une personne ou une
  responsabilité sensible.

Cette séparation évite de demander au même système d’imaginer une action, de
s’accorder le droit de l’accomplir et de déclarer ensuite qu’elle s’est bien
passée. Un modèle peut proposer l’envoi d’un message ; il ne peut ni ajouter
silencieusement cette capacité au workflow, ni choisir seul une nouvelle liste
de destinataires, ni approuver sa propre demande.

Le compilateur transforme le graphe édité dans Workflow Hub en plan déterminé
et versionné. Il vérifie notamment que les références existent, que les types
sont compatibles, que les capacités demandées sont exprimables et que les
points de consentement sont placés avant les effets concernés. Le runtime Rust
exécute ensuite ce plan. Il ne suit pas une improvisation textuelle permanente.

L’humain, enfin, ne doit pas être réduit à un bouton décoratif. Une demande
d’approbation utile indique l’action prévue, la destination, les données
concernées, le coût ou le budget, ainsi que les conséquences possibles d’un
refus. Le consentement porte sur un effet compréhensible, pas sur une promesse
générale de « laisser l’IA travailler ».

## Des secrets absents des objets partagés

Un pack portable doit pouvoir circuler sans transporter les secrets de son
auteur. Les clés d’API, jetons, mots de passe et identifiants privés ne sont
donc pas intégrés au workflow publié. Le pack déclare seulement qu’un secret
d’un certain type sera nécessaire au moment de l’exécution.

Le secret est résolu par le shell de confiance, dans le contexte de la
personne ou de l’organisation qui lance le workflow. Il n’est fourni qu’à
l’activité autorisée, pour la durée nécessaire. Le journal peut indiquer qu’un
secret a été utilisé sans en révéler la valeur.

Cette séparation permet à deux communautés d’exécuter le même workflow avec
leurs propres comptes et leurs propres politiques. Elle évite aussi qu’un fork
ou une simple inspection du pack expose des identifiants qui n’auraient jamais
dû voyager.

## Des reçus honnêtes, y compris lorsque le résultat est inconnu

Une exécution distribuée ne peut pas toujours savoir ce qui s’est produit. Une
requête peut avoir quitté la machine, puis la connexion être interrompue avant
la réponse. Relancer automatiquement l’activité pourrait produire l’effet deux
fois ; la déclarer échouée pourrait être faux.

Nexus conserve donc un état explicite : le résultat est inconnu.

Ce choix paraît moins confortable qu’un voyant vert ou rouge, mais il est plus
sûr. Le moteur enregistre ce qu’il sait : l’autorisation a été accordée, la
tentative a commencé, une réponse n’a pas été reçue. Il peut ensuite demander
une vérification humaine, utiliser une clé d’idempotence ou appliquer une
procédure propre à l’activité. Il ne fabrique pas une certitude pour rendre
l’interface plus rassurante.

Les reçus, événements et artefacts forment ainsi une mémoire vérifiable de
l’exécution. Ils ne constituent pas une surveillance générale de
l’utilisateur. Ils restent limités au run, à ses effets et à ses besoins
d’explication.

L’architecture de confiance de Nexus repose finalement sur une idée simple :
une bonne preuve répond précisément à une question limitée. Dès qu’elle
prétend répondre à toutes les questions, elle devient une nouvelle forme
d’autorité opaque. Prouver, autoriser et consentir sont trois gestes
complémentaires ; aucun ne permet de confisquer les deux autres.
