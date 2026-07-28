# Glossaire

## Agent

Définition versionnée d’un rôle borné : prompt, entrées, sorties, politique de
modèle, outils possibles, contexte, budget, timeout et limites. Un agent n’est
ni une personne juridique ni une autorité autonome.

## Application SBFB

Archive web distribuable et adressée par son contenu, ouverte dans une iframe
sandboxée. Elle accède aux capacités du nœud uniquement par un bridge contrôlé.

## Artefact

Objet produit ou consommé par un workflow : document, dataset, rapport,
archive, preuve, pack, diff ou résultat typé. Les gros contenus vivent en
blobs ; les événements du run y font référence.

## Atelier des communs

Processus de maturation qui part d’une graine de besoin, la clarifie, recherche
l’existant, produit plusieurs options, organise la contradiction et aboutit à
un `ResearchPack` puis éventuellement à un `DecisionPack`. Ideas Hub en est la
surface d’entrée.

## BLAKE3

Fonction de hachage utilisée pour adresser et vérifier des contenus. Une
empreinte identique soutient l’intégrité des octets comparés ; elle ne prouve
pas la vérité de leur sens.

## Blob

Objet immuable adressé par son contenu : archive d’application, pack, gros
artefact ou pièce de preuve. Plusieurs nœuds peuvent le conserver et le servir.

## Bridge

Canal typé entre une application sandboxée et l’hôte de confiance. Il expose
des opérations bornées plutôt qu’un accès général au réseau ou à la machine.

## Capacité

Droit typé et limité d’accomplir une opération : lire une ressource, écrire
dans un espace, utiliser un outil, contacter une destination ou publier un
objet. Une capacité demandée n’est pas une capacité accordée.

## CompiledPlan

Plan immuable produit par le compilateur Rust à partir d’un workflow validé. Il
fixe la topologie, les outils, les destinations, les budgets et les points de
consentement d’une exécution.

## Conception

Workflow qui transforme une décision acceptée en choix de produit et choix
techniques
auditables. Il produit un `ConceptionPack` consommable par Factory.

## ConceptionPack

Artefact structuré contenant problème, périmètre, flux, architecture, données,
capacités, décisions, menaces, accessibilité, critères d’acceptation, inconnues
et provenance.

## Curation

Recommandation, mise en garde ou sélection publiée par une personne ou une
communauté. L’utilisateur choisit les curateurs qu’il suit. La curation locale
ne supprime pas un objet pour le réseau entier.

## DecisionPack

Artefact d’arbitrage humain produit après la maturation d’une idée. Il indique
la décision, son périmètre, ses raisons, ses inconnues et la suite autorisée.

## Digest

Empreinte stable d’un objet canonique ou de sa sémantique. Elle permet de lier
une approbation, une exécution ou une preuve à une version exacte.

## Ed25519

Système de signature numérique utilisé pour attribuer et vérifier des objets.
Une signature valide relie l’objet à une clé ; elle n’établit ni l’identité
civile ni la justesse de son contenu.

## Factory

Processus et outils de fabrication qui transforment un brief ou un pack en
artefact livré avec plan, isolation, implémentation, tests, reviews, preuves et
confirmation humaine.

## Fork

Création d’une lignée indépendante à partir d’un objet existant. Le fork permet
d’expérimenter ou de poursuivre un projet sans effacer son origine.

## Grant

Autorisation effectivement accordée à une application, un pack ou une version
pour utiliser une capacité. Le grant peut être réduit, expiré ou révoqué.

## HITL

*Human in the loop*. Intervention humaine requise pour arbitrer, revoir ou
consentir. Dans Nexus, le shell de confiance présente les demandes sensibles
et recueille une décision liée au digest exact ; le daemon/runtime l’intègre à
son calcul d’autorité.

## Idempotence

Propriété qui permet de répéter une opération sans répéter son effet. Le
runtime utilise des clés d’idempotence et des reçus pour reprendre après une
interruption.

## Iroh

Ensemble de primitives réseau utilisées par SBFB pour transporter des blobs,
des annonces et des documents répliqués entre pairs. Iroh n’est pas une base
SQL globale ni une autorité de gouvernance.

## Kudos

Registre V1 non monétaire et non transférable des tâches de calcul exécutées,
avec rendements logarithmiques et décroissance temporelle. Il est public et
peut relier les contributions d’une même identité technique entre plusieurs
projets. Les kudos ne sont ni un token, ni un droit de vote universel, ni une
preuve de vérité. La reconnaissance séparée du stockage, du relais et d’autres
formes de contribution appartient à Kudos v2.

## Nœud

Environnement local d’une personne ou d’une organisation : identité technique,
daemon, shell, stockage, cache, runtime et, selon le consentement, calcul ou
outils de fabrication.

## Outcome unknown

État honnête indiquant qu’un effet externe a peut-être eu lieu mais qu’aucun
reçu ne permet de le confirmer. Le runtime n’invente ni succès ni échec.

## Pack

Artefact portable et versionné regroupant un workflow, des agents, prompts,
schémas, capacités demandées, évaluations et provenance.

## Projection publique

Vue limitée par une liste de champs autorisés, dérivée d’un objet privé ou
faisant autorité, puis publiée pour la recherche, la découverte ou l’audit.
RRV consomme des projections ; il ne reçoit pas automatiquement la base source.

## Proof Card

Surface explicative réunissant les signaux disponibles sur une version :
source, signature, digest, licence, fraîcheur, provenance, tests, curations et
limites. Ce n’est pas un certificat absolu.

## Provenance

Chaîne qui relie un objet à ses sources, versions, transformations, auteurs
techniques et attestations. Elle explique l’origine ; elle ne remplace pas le
jugement sur le contenu.

## ResearchPack

Dossier de recherche issu de l’Atelier des communs. Il rassemble preuves,
options, critiques, inconnues et recommandations sans prononcer un verdict de
fabrication.

## RRV

Recherche Réseau Vérifiable. Application et surfaces de recherche qui trouvent
des objets, expliquent leur origine, distinguent ce qui est lu, déduit,
vérifié ou non vérifié et proposent des actions bornées.

## Run

Exécution durable d’un `CompiledPlan`, identifiée, journalisée et liée à des
versions exactes. Un run peut être repris, mis en attente, annulé dans la
limite des effets déjà partis ou forké depuis un checkpoint.

## Runtime

Moteur Rust faisant autorité pour l’exécution : il compile, planifie,
journalise et reprend les workflows. Workflow Hub l’observe et lui transmet des
intentions ; il ne le remplace pas.

## SBFB

Protocole de Nexus pour publier et distribuer des applications, transporter
des objets et événements signés, partager certains états et contribuer du
calcul sans serveur central obligatoire.

## Shell de confiance

Interface hôte qui présente les demandes privilégiées, recueille grants et
confirmations humaines, et sert d’intermédiaire pour la résolution des secrets.
Il sépare les applications sandboxées de l’autorité d’exécution du
daemon/runtime ; il ne calcule pas seul les capacités effectives et ne produit
pas les reçus faisant autorité.

## Workflow

Définition versionnée d’un processus : entrées, sorties, opérations, agents,
branches, parallélisme, outils, validations, budgets et conditions d’arrêt.

## Workflow Hub

Application de création, versionnement, évaluation, exploitation et
publication des workflows et agents. Elle offre canvas, outline, mode guidé et
interaction conversationnelle sur le même modèle sémantique.
