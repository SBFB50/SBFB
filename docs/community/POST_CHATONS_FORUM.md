# Post forum CHATONS

**Categorie** : Cafe du commerce
**Titre** : SBFB — distribuer des apps web en P2P, sans serveur central, code verifiable (AGPL-3.0)

---

Salut a toutes et tous,

Je m'appelle Theophile, dev solo francophone. Je bosse sur un protocole
libre (AGPL-3.0) qui touche directement a ce que font les CHATONS, et
je pense que c'est le bon endroit pour en parler.

## L'idee en bref

SBFB est un protocole P2P ou n'importe qui publie une app web — du
simple HTML a du React, Python dans le navigateur, notebook Jupyter,
peu importe. Le reseau la distribue automatiquement entre les noeuds.
Les utilisateurs la voient dans un navigateur, dans un iframe
sandboxe. Pas de serveur central, pas de store, pas de compte.

Concretement : vous installez un petit daemon Rust (~21 Mo, un seul
binaire, zero config obligatoire), vous publiez votre app en une
commande, et elle se propage. Si votre machine s'eteint, les gens qui
l'ont deja vue continuent de la servir depuis leur noeud.

## Comment le reseau fonctionne

Le daemon fait tourner un noeud P2P base sur iroh (protocole QUIC,
chiffre de bout en bout avec ChaCha20-Poly1305). Trois mecanismes
principaux :

**La propagation des apps** : quand vous publiez une app, le daemon
cree une archive zip de votre code, calcule son hash BLAKE3, et
l'annonce sur le reseau via gossip — un systeme de diffusion pair-a-
pair ou les messages se propagent de voisin en voisin. Les autres
noeuds qui voient l'annonce telecharge l'archive automatiquement et
la mettent en cache. C'est le meme principe que BitTorrent, applique
a des apps web.

**Le stockage des donnees** : les apps peuvent stocker des donnees
(votes, commentaires, preferences) dans un espace replique entre les
noeuds. C'est base sur un CRDT (un type de structure de donnees qui
sait fusionner sans conflit). Ca fonctionne meme hors-ligne : chaque
noeud continue de fonctionner independamment, et les donnees se
synchronisent automatiquement a la reconnexion. Pas de base de
donnees centrale a maintenir.

**La decouverte** : les noeuds se trouvent via une table de hachage
distribuee (DHT), avec un fallback DNS et un relais WebSocket. Ca
fonctionne derriere un NAT, pas besoin d'IP publique ni de port
forwarding.

## La provenance — savoir d'ou vient le code

Chaque app publiee est accompagnee d'une preuve cryptographique :

- Le daemon clone votre depot Git
- Il verifie le commit que vous avez declare
- Il construit l'archive zip
- Il signe le tout avec sa cle Ed25519

Le resultat c'est un fichier `provenance.json` qui dit : "cette
archive correspond a ce commit de ce repo, signe par ce noeud a
cette date". N'importe qui sur le reseau peut verifier la chaine
complete. C'est ce qu'on appelle SLSA Level 1 — une auto-attestation
verifiable.

Ca veut dire qu'un utilisateur peut regarder une app sur le reseau et
verifier que le code qu'il execute correspond bien au code source
publie. Pas de binaire opaque, pas de "faites-moi confiance".

## Niveaux de confiance — savoir a quel point on peut faire confiance

Le protocole definit 6 niveaux de confiance cumulatifs pour chaque
app :

- **N0** : l'app a ete uploadee sur le reseau (zip brut, rien
  verifie)
- **N1** : l'app declare un repo source lisible par tous
- **N2** : la provenance a ete auto-attestee (Ed25519 + BLAKE3)
- **N3** : la signature de provenance est verifiee en direct par le
  daemon
- **N4** : le build est reproductible par un tiers independant
  (futur)
- **N5** : l'app est inscrite dans le feed public verifie (chaine de
  hash immuable)

L'utilisateur voit directement le niveau de confiance de chaque app.
Un **Proof Card** (score de 0 a 100) resume 7 facteurs de risque :
est-ce que la provenance est signee, est-ce qu'un curator l'a
recommandee, est-ce que le repo est accessible, est-ce que le code
est recent. Le score est decomposable — on peut voir chaque couche
de preuve individuellement.

## Les curators — la confiance sans moderation

Pas de moderateur central qui decide quoi publier ou quoi supprimer.
A la place, un systeme de listes de recommandation :

- N'importe qui peut creer une liste de curator : c'est juste une
  cle Ed25519 et une liste d'apps que vous recommandez
- Vous signez votre liste, elle se propage sur le reseau par gossip
- Les utilisateurs choisissent a quels curators ils font confiance
  et s'abonnent a leurs cles publiques
- Un curator peut dire "je recommande cette app" ou "je la
  deconseille" — mais il ne peut pas la supprimer du reseau

L'idee c'est que chaque communaute peut etre son propre curator.
Vous recommandez les apps que vous avez verifiees pour vos
utilisateurs. Un autre hebergeur recommande les siennes. Les
utilisateurs composent leur vision du reseau en choisissant
leurs curators.

Les listes ont un compteur de revision monotone (impossible de
revenir en arriere a une version precedente), et le systeme gere
la rotation de cle si vous devez changer votre cle Ed25519 (fenetre
de transition de 14 jours max, l'ancienne et la nouvelle cle sont
acceptees pendant la transition, puis l'ancienne est revoquee).

## Le feed public — un journal immuable

Chaque evenement important sur le reseau (nouvelle app publiee,
curator qui endorse une app, source devenue obsolete) est inscrit
dans un **feed public append-only**. Chaque entree est signee
Ed25519 et chainee par hash BLAKE3 a la precedente — impossible
d'effacer ou modifier retroactivement.

Ca veut dire que n'importe qui peut telecharger l'historique complet
du reseau et verifier que personne n'a triche. C'est la meme idee
qu'une blockchain mais sans la lourdeur — pas de consensus distribue,
pas de proof-of-work global, juste des chaines de hash par auteur
qui se verifient independamment.

## La recherche

Un moteur de recherche local (SQLite FTS5) indexe toutes les apps
par nom, description et categorie. La recherche tourne localement
sur votre noeud — pas de serveur central qui sait ce que vous
cherchez. Les requetes sont nettoyees contre les injections.

## Le sandbox — comment les apps sont isolees

Les apps tournent dans un iframe du navigateur avec cinq couches de
protection :

L'iframe n'a pas le droit d'acceder a la page parente. Il ne peut
pas faire de requetes reseau vers l'exterieur (le navigateur bloque
ca via une Content Security Policy stricte). Il ne peut pas lire les
cookies ou le stockage local du shell. Il ne peut pas acceder aux
donnees d'une autre app.

La seule facon pour une app de communiquer avec le reseau, c'est via
un **bridge** : un SDK JavaScript qui envoie des messages au shell
via `postMessage`. Chaque message a un identifiant unique, un timeout
de 10 secondes, et le shell verifie que le message vient bien de
l'iframe attendu. Un heartbeat toutes les secondes detecte si une
app est figee (watchdog CPU).

Les methodes disponibles sont whitelistees — on ne peut que :
stocker/lire des donnees, soumettre une tache de calcul, lire l'etat
du reseau, verifier une provenance, chercher des apps, lire son
identite publique. Rien d'autre.

Cote daemon, tout passe par loopback (127.0.0.1) avec un token
d'authentification de 256 bits, plus une verification de l'origine
des requetes, plus une verification des credentials du processus
appelant (SO_PEERCRED sur Linux, Named Pipe DACL sur Windows).
Aucun port n'est expose sur le reseau.

## Factory — creer et publier des apps

`sbfb-factory` est un outil en ligne de commande (Rust) qui guide
toute la chaine :

**Creer** : `sbfb-factory create --template static --name mon-outil`
genere un squelette avec un manifeste `SBFB.json`, un `index.html`,
le SDK bridge, et un README. Deux templates de base : un statique
simple et un avec le bridge integre pour les apps collaboratives.

**Valider** : `sbfb-factory validate` verifie que le manifeste est
correct, que les methodes bridge demandees sont dans la liste
autorisee, qu'il n'y a pas de fichiers sensibles.

**Tester** : `sbfb-factory preview` charge votre app dans le daemon
en mode ephemere (30 minutes, max 10 previews simultanees). Vous
voyez exactement ce que les utilisateurs verront, dans le meme
sandbox.

**Publier** : `sbfb-factory publish` passe par 11 verifications
automatiques (les "gates") :
- Le manifeste est valide et complet
- Les methodes bridge demandees sont autorisees
- Il n'y a pas de secrets dans le code (scan regex : cles AWS,
  tokens API, certificats PEM)
- Il n'y a pas de symlinks qui sortent du repertoire (protection
  path traversal)
- Le daemon local est accessible
- La provenance est signee Ed25519
- L'archive est diffusee sur le reseau P2P

Toutes ces verifications sauf la derniere tournent en local, hors-
ligne. Si une verification bloquante echoue, la publication s'arrete
et vous dit pourquoi.

Le manifeste de chaque app (`SBFB.json`) declare explicitement ce
dont l'app a besoin :

```json
{
  "name": "mon-outil-de-vote",
  "display_name": "Outil de vote communautaire",
  "description": "Proposez et votez sur des idees pour votre asso",
  "category": "social",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get", "storage_set", "identity_pubkey"]
  }
}
```

Si une app ne declare pas `storage_set`, elle ne peut pas ecrire de
donnees. C'est le principe du moindre privilege — l'utilisateur voit
ce que l'app peut faire avant de l'ouvrir.

## Factory Operator — l'outil graphique

En plus du CLI, un outil graphique local (React, dark theme) connecte
au daemon donne une vue d'ensemble :

- L'etat du sprint de developpement en cours
- L'historique complet de toutes les modifications avec le diff
  inline fichier par fichier (on voit le code qui a change, ligne
  par ligne, colore en vert/rouge)
- Le journal de toutes les actions Factory
- Un chat integre pour poser des questions sur le projet
- La validation des artefacts de planification
- L'audit des commit bodies
- La generation de context-packs pour transferer le contexte entre
  outils

Le tout avec navigation par sprint (on peut remonter sur les 67
sprints passes et voir le detail de chacun).

## Le partage de puissance GPU

Un des aspects les plus differenciants : SBFB permet de partager
du calcul GPU entre les noeuds du reseau. L'idee, c'est que si vous
avez un GPU (meme modeste) et que vous acceptez de le partager, les
apps du reseau peuvent l'utiliser pour faire de l'inference IA
(traduction, resume, analyse de texte, etc.) via Ollama.

### Comment ca fonctionne

Un utilisateur soumet une tache via une app ("resume ce texte en
200 mots"). Le coordinateur local route la tache vers un noeud qui
a un GPU disponible et qui a accepte de le partager. Le worker
execute la tache et renvoie le resultat. Le coordinateur verifie
le resultat et credite le worker en kudos.

### Le consentement explicite

La premiere fois que vous lancez le daemon, un dialogue vous demande
votre niveau de partage :

- **Niveau 1** : je ne partage mon GPU que pour mes propres projets
- **Niveau 2** : pour les projets open source dont le code est
  verifie
- **Niveau 3** : pour une liste de projets que je choisis
  manuellement (whitelist)
- **Niveau 4** : pour tous les projets du reseau

Et des plafonds configurables : watts max, VRAM max, heures max par
jour. Le daemon refuse automatiquement les taches qui depassent vos
limites. Le consentement est rechargeable a chaud (modification du
fichier de config, prise en compte en 50 millisecondes).

### Le monitoring GPU

Le daemon surveille en temps reel l'utilisation de votre GPU (VRAM
utilisee, pourcentage d'utilisation, temperature, consommation en
watts). Ces informations sont affichees dans la page "Mon reseau"
du shell. Si votre GPU surchauffe ou depasse vos limites, les
taches sont refusees automatiquement.

### La validation des resultats

Les resultats ne sont pas acceptes aveuglement :

- **Quorum** : si plusieurs workers traitent la meme tache, leurs
  resultats sont compares (hash SHA256). Si les resultats divergent,
  les outliers sont rejetes.
- **Filtrage PII** : avant d'envoyer une tache au worker, le
  coordinateur scanne le prompt pour detecter et masquer les donnees
  personnelles (emails, telephones, cartes bancaires, IBAN). Le scan
  utilise des regex + un modele ML leger (GLiNER).
- **Filtrage de sortie** : les resultats suspects (contenu toxique,
  patterns d'exfiltration) sont filtres avant de revenir a l'app.
- **Quarantaine** : les taches douteuses sont mises en quarantaine
  dans une file d'attente. Un validateur humain peut les revoir et
  decider de les accepter ou les rejeter.

### Les kudos — la reputation

Quand un noeud contribue du calcul, il recoit des points de
reputation appeles kudos. Quelques points importants :

- C'est **pas une monnaie**. On ne peut pas transferer ses kudos,
  les vendre, les echanger. C'est un score lie a votre identite.
- La formule utilise des **rendements decroissants** (logarithme) :
  contribuer beaucoup donne plus de kudos, mais pas
  proportionnellement. Ca empeche les gros GPU de concentrer toute
  la reputation.
- Les kudos **decroissent dans le temps** (demi-vie ~23 jours) :
  les contributions recentes comptent plus que les anciennes.
- Chaque entree kudos est **signee et chainee** par hash BLAKE3 :
  n'importe qui peut telecharger le ledger complet et verifier que
  personne n'a triche.
- Un **coefficient de Gini** est calcule en temps reel pour detecter
  si la reputation se concentre chez quelques noeuds. Si ca depasse
  un seuil, c'est un signal d'alerte visible par tous.
- Un **leaderboard** par projet et des **metriques de fairness**
  (top-K contributors, taux de renouvellement) sont accessibles
  via des endpoints dedies.

## Le vote dans les apps

L'app Ideas Hub montre comment faire du vote decentralise :

Chaque vote est stocke comme une cle dans le stockage P2P :
`votes/{idee}/{cle_publique_du_votant}`. Comme la cle publique
Ed25519 de chaque noeud est unique, c'est impossible de voter deux
fois. Re-cliquer retire le vote. Les votes se synchronisent entre
tous les noeuds automatiquement.

Ce pattern est generique — n'importe quelle app SBFB peut l'utiliser
pour du budget participatif, des sondages, du peer-review, ou
n'importe quel mecanisme de decision collective.

## Les invitations — groupes prives

Un developpeur peut creer des invitations signees pour enroler des
workers dans un projet specifique. L'invitation a un scope (quelles
actions sont autorisees), une date d'expiration, et un nombre
maximum d'utilisations. Ca permet de former un groupe prive de
testeurs ou de workers de confiance avant de rendre un projet
public.

## La securite en profondeur

### Ce qu'on garantit aujourd'hui

- Identite Ed25519 protegee par permissions fichier (0600)
- Loopback authentifie (token 256 bits + verification de l'origine
  + verification des credentials du processus appelant)
- Sandbox iframe 5 couches
- Feed public append-only (chaine de hash, impossible d'effacer)
- Kudos verifiables (chaine BLAKE3 + Ed25519)
- Anti-spam Hashcash 16 bits sur le gossip (un noeud doit resoudre
  un puzzle cryptographique avant de publier une annonce — ca coute
  quelques secondes de calcul, assez pour decourager le spam)
- Resistance Sybil 3 couches : un nouveau noeud doit exister 7
  jours minimum avant de participer au gossip (couche 1), des
  attestations de contribution sont enregistrees (couche 2), et un
  systeme de delegation de reputation est prevu (couche 3)
- Rotation de cle Ed25519 avec fenetre de transition
- Capacites desactivees par defaut (features sensibles comme
  l'acces MCP ou le calcul GPU illimite sont gate-off par defaut
  et necessitent une activation explicite avec privilege admin OS)

### Le warrant canary

Chaque mois, le mainteneur signe une declaration cryptographique
attestant que le projet n'a recu aucun ordre secret de modification
du code, de surveillance des utilisateurs, ou d'insertion de
backdoor. Si la signature cesse de paraitre, c'est un signal
d'alerte (dead-man switch). Le framework FROST (signature a seuil
K-de-N entre plusieurs mainteneurs) est pret pour distribuer la
responsabilite entre plusieurs personnes quand le projet grandira.

### Le mode sous contrainte

Si quelqu'un vous force physiquement a reveler votre cle, un code
PIN secret active un mode degrade qui detruit la keypair locale et
envoie un signal canari discret. La cle n'existe plus — impossible
de cooperer meme sous contrainte.

### Ce qu'on ne garantit pas encore (et on le dit)

- La cle privee n'est pas chiffree au repos (prevu : Keychain
  macOS / DPAPI Windows / libsecret Linux)
- Pas d'audit de securite formel sur iroh (notre couche reseau)
- Rate limiting incomplet sur certains endpoints

On prefere dire clairement ce qui manque plutot que de faire croire
que tout est parfait.

## Les apps qui tournent

Trois apps exemples, toutes en HTML/JS pur sans aucune dependance :

**Protocol Explorer** : 6 sections de documentation interactive sur
le protocole. Un panneau live affiche l'etat du reseau en temps reel
(pairs connectes, apps disponibles). Un bouton verifie la provenance
d'une app en un clic — le daemon verifie la signature Ed25519 et
affiche le resultat.

**Ideas Hub** : proposer et voter sur des idees. Les donnees se
synchronisent entre les noeuds automatiquement. 1 identite = 1 vote.
On peut trier par nombre de votes ou par date. On peut supprimer ses
propres idees.

**Factory Viewer** : affiche les apps du reseau avec leur Proof Card
(score de qualite 0-100 decomposable en 7 facteurs de risque).

## L'interface utilisateur

Le shell React (l'interface que voit l'utilisateur) propose :

- **Browse** : une grille d'apps du reseau avec statut
  reachable/unreachable, badges de verification, et filtrage par
  curator
- **Vue immersive** : quand on ouvre une app, elle prend tout l'ecran
  avec une barre qui se masque automatiquement. La barre affiche la
  provenance et le Proof Card.
- **Curators** : gestion des abonnements aux listes de curators
- **Mon reseau** : etat live du noeud (GPU, pairs connectes, taches
  en cours, kudos gagnes, consent applique)
- **Deploy** : formulaire pour publier directement depuis un repo Git
- **Recherche** : recherche full-text locale dans toutes les apps

Le daemon a aussi une icone dans la barre systeme
(Windows/macOS/Linux) qui affiche l'etat et les erreurs sans avoir
a ouvrir un terminal.

## Installation

Un binaire Rust (~21 Mo), zero config obligatoire :

```bash
./nexus-shell-daemon
# -> cree une identite Ed25519 + token d'auth dans ~/.sbfb/
# -> ouvre le navigateur
# -> ecoute sur 127.0.0.1 uniquement
# -> consomme ~150 Mo RAM en idle
```

Pour un service systemd sur un serveur :

```bash
sudo useradd -m -s /bin/false sbfb
sudo cp nexus-shell-daemon /usr/local/bin/
sudo systemctl enable sbfb-daemon
sudo systemctl start sbfb-daemon
```

Installeurs disponibles : Windows (NSIS), Linux (.deb), macOS (.dmg).
Aucun port public expose. Pas de certificat TLS a renouveler.

## Etat du projet

- ~1800 tests (1486 Rust, 279 JavaScript), tous verts
- Installeurs Windows, Linux .deb, macOS .dmg
- P2P teste en LAN (Windows <-> Mac) et en WAN (dev <-> VPS Helsinki)
- Licence AGPL-3.0
- Solo maintainer, pas de startup, pas de fondation, pas de token
- Pilote ferme — pas encore de noeuds tiers en production

## Ce que je cherche

Je ne cherche pas de financement. Je cherche un ou deux hebergeurs
alternatifs motives pour tester le protocole en conditions reelles :

1. Installer un noeud SBFB a cote de vos services existants
2. Publier 2-3 petites apps utiles a votre communaute
3. Se mettre en curator l'un pour l'autre
4. Voir ce qui marche et ce qui casse

Le resultat attendu : "3 apps publiees, 2 noeuds qui se voient, le
P2P tient" — ou "ca marche pas pour telle raison et voila ce qu'il
faudrait changer". Les deux me sont utiles.

Le code est sur [lien repo]. Dispo pour une demo en visio de
15 minutes ou pour repondre a vos questions ici.

Bonne journee
