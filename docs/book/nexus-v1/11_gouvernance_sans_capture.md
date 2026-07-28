# Chapitre 11 — Gouverner sans capturer

La décentralisation technique ne suffit pas à produire une gouvernance
pluraliste. Un réseau peut être distribué entre des milliers de machines et
rester contrôlé par une seule équipe qui possède la marque, les clés de
publication, le classement par défaut et la capacité de décider quelles
versions sont « officielles ».

Nexus distribue donc non seulement les fichiers, mais aussi plusieurs chemins
par lesquels une décision peut acquérir de l’autorité. Il ne prétend pas
supprimer le pouvoir. Il cherche à le rendre visible, limité et contestable.

Deux couches doivent cependant rester distinctes. Le **protocole** garantit
des mécanismes : objets signés, capacités locales, pluralité des curateurs,
absence de classement mondial imposé, possibilité de forker et de partir. Une
**charte institutionnelle** organise des responsabilités humaines : organes de
décision, représentation, conflits d’intérêts, recours, garde des clés et
procédure de modification. SBFB rend ces chartes publiables et vérifiables ; il
ne les écrit pas et ne les applique pas magiquement à toutes les communautés.

## Plusieurs espaces, plusieurs légitimités

Une communauté peut adopter une charte, choisir des curateurs, maintenir une
collection d’applications et définir ses propres règles de publication. Une
autre communauté peut utiliser le même protocole avec des choix différents.
Aucun de ces espaces n’est automatiquement « le réseau tout entier ».

Cette distinction protège le pluralisme. Elle permet à un collectif de
maintenir une exigence éditoriale forte sans imposer cette exigence à tous les
autres. Elle permet aussi aux utilisateurs de quitter une gouvernance devenue
insatisfaisante sans perdre leurs objets, leur historique ou la possibilité de
continuer le travail ailleurs.

RRV reflète cette pluralité. Il n’existe pas un classement universel qui
détermine ce que chacun doit voir. Les résultats peuvent être organisés selon
des sources choisies, des curateurs suivis, des critères explicites ou des
espaces communautaires. L’utilisateur peut savoir pourquoi un résultat
apparaît et changer de perspective.

Cela ne signifie pas que tous les classements se valent. Certains sont plus
documentés, plus ouverts ou plus résistants aux manipulations. Mais leur
autorité vient de règles visibles et d’une confiance accordée, pas du simple
fait qu’ils sont le réglage caché de la plateforme.

## L’Atelier des communs n’est pas une machine à fabriquer une majorité

L’Atelier des communs facilite l’émergence et le travail des propositions. Il
recueille des soutiens, des objections, des besoins, des compétences et des
expériences de terrain. Il ne transforme pas chaque question complexe en
concours de popularité.

Une mobilisation nombreuse peut légitimement montrer qu’un sujet compte pour
beaucoup de personnes. Elle ne doit pas effacer les minorités concernées, les
contraintes matérielles ou les connaissances contradictoires. La gouvernance
d’une idée combine donc plusieurs signaux : soutien, diversité des
contributeurs, qualité des sources, faisabilité, effets sur les personnes
concernées et existence d’alternatives.

Les protections contre la capture ne consistent pas seulement à bloquer des
robots. Une communauté peut inscrire dans sa charte des pouvoirs progressifs
pour les nouveaux comptes, des limites de rythme, une séparation entre
proposition et décision, une période de discussion, un recours et la
conservation des désaccords. Le protocole peut horodater, signer et versionner
ces règles ; leur légitimité et leur application restent une responsabilité
institutionnelle.

Une idée rejetée ne disparaît pas nécessairement. Elle peut être archivée,
forkée ou reprise dans un autre espace. Une idée acceptée n’obtient pas pour
autant tous les moyens d’agir. Elle doit encore passer par les capacités, les
budgets et les consentements du contexte où elle sera réalisée.

## La version comme outil de gouvernance

Dans Nexus, une version n’est pas seulement un numéro technique. Elle
matérialise un accord.

Workflow Hub permet de travailler librement sur un brouillon, de tester des
variantes et de discuter une modification. Lorsque le workflow est prêt, une
version est compilée, examinée puis approuvée. Les exécutions qui la citent
savent alors exactement quel plan elles utilisent.

Le processus de Factory suit la même logique. Il peut évoluer, améliorer ses
méthodes ou intégrer de nouvelles étapes entre deux cycles. Une équipe peut le
forker, le modifier, le tester et proposer une nouvelle version. Elle ne
remplace pas silencieusement le processus qui est déjà en train de produire
une application.

Cette discipline donne une forme concrète à la responsabilité. Si une décision
produit un mauvais résultat, il est possible de retrouver la version, les
règles et les approbations correspondantes. Si une amélioration est proposée,
elle peut être comparée à l’état précédent. Le désaccord ne porte pas sur un
système mouvant dont personne ne peut reconstituer le comportement.

## Répartir les points de pouvoir

La V1 considère plusieurs formes de pouvoir séparément :

- écrire le protocole ;
- maintenir une implémentation ;
- signer une release ;
- distribuer une application ;
- recommander un contenu ;
- administrer un espace ;
- modérer une publication ;
- accorder une capacité locale ;
- approuver un effet sensible.

Réunir tous ces pouvoirs dans les mêmes mains serait pratique, mais fragile.
Nexus les rend donc distincts autant que possible.

Une équipe peut maintenir l’implémentation de référence sans contrôler toutes
les collections distribuées. Un curateur peut recommander une application sans
pouvoir lui accorder des droits sur la machine de l’utilisateur. Un
administrateur communautaire peut modérer son espace sans effacer l’objet des
autres espaces. Une release peut être signée et vérifiée par plusieurs
responsables sans donner à chacun un accès quotidien aux secrets de
production.

L’organisation qui maintient l’implémentation de référence doit ajouter sa
propre charte à cette séparation technique. Dans la V1 décrite ici, elle
documente les changements importants, exige la déclaration des conflits
d’intérêts pertinents, renforce la revue des frontières de sécurité et des
formats durables, répartit la garde des clés critiques et publie les règles de
compatibilité et de migration. Ce sont des engagements institutionnels
auditables, pas des propriétés que le protocole pourrait garantir à lui seul.
Une autre communauté peut adopter une charte différente sans obtenir pour
autant davantage d’autorité sur les autres nœuds.

Il reste toujours des centres de gravité. L’implémentation la plus utilisée
influence le protocole. Une interface populaire façonne les habitudes. Un
financeur important peut peser sur les priorités même sans droit formel. La
gouvernance pluraliste ne nie pas ces forces ; elle cherche à les rendre
discutables et réversibles.

## Le droit pratique de partir

Une gouvernance n’est réellement limitée que si l’on peut la quitter.

Dans Nexus, cette possibilité passe par des mécanismes concrets : export des
données, formats documentés, objets adressés par leur contenu, historique
portable, possibilité de changer de curateur et droit de forker un workflow ou
une application selon sa licence. Une communauté peut migrer sans demander la
permission à l’organisation qu’elle quitte.

La sortie n’efface pas les obligations prises, les droits des autres auteurs
ni les limites de confidentialité. Elle empêche simplement qu’un désaccord
politique, éditorial ou technique se transforme en captivité.

Cette propriété modifie aussi la qualité du pouvoir. Une équipe qui sait que
les utilisateurs peuvent réellement partir doit convaincre davantage et
imposer moins. Le fork n’est pas souhaitable pour chaque désaccord, mais sa
possibilité protège la négociation.

## Une infrastructure ouverte aux personnes, pas appropriée par une appartenance

Nexus accueille des personnes venues d’associations, de mouvements, de
métiers ou de traditions très différentes. Leur expérience et leur capacité
d’engagement sont précieuses. Elles participent comme personnes ou comme
communautés identifiées, selon les mêmes règles publiques que les autres.

Aucune appartenance extérieure ne donne un droit particulier sur le
protocole. Inversement, elle ne constitue pas une raison de soupçon ou
d’exclusion. Ce qui compte dans l’espace commun est le comportement
observable : respect des capacités, transparence des sources, absence
d’usurpation, possibilité de contradiction et respect des règles de l’espace
choisi.

Cette ouverture n’est pas une absence de valeurs. La vision Nexus privilégie
la dignité des personnes, leur autonomie, l’accessibilité, la coopération
volontaire et la possibilité de contester le pouvoir. Certaines conditions
sont protégées techniquement ; les autres doivent être assumées dans des
chartes, des pratiques et des décisions humaines explicites. Aucune
organisation particulière ne peut présenter ses propres choix comme une
garantie naturelle du protocole.

La gouvernance pluraliste demande finalement plus de travail qu’un centre
unique. Elle produit des désaccords visibles, des procédures et parfois des
bifurcations. C’est son coût, mais aussi son rôle : empêcher l’efficacité
immédiate de devenir une domination durable. Gouverner sans capturer ne
signifie pas obtenir l’unanimité ; cela signifie préserver, jusque dans le
désaccord, la capacité de comprendre, de contester et de reprendre autrement.
