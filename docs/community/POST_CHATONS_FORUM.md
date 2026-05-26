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

Le daemon fait tourner un noeud P2P base sur iroh (QUIC, chiffre de
bout en bout avec ChaCha20-Poly1305). Trois mecanismes :

**La propagation des apps** : quand vous publiez une app, le daemon
cree une archive zip de votre code, calcule son hash BLAKE3, et
l'annonce sur le reseau via gossip (un systeme de diffusion
pair-a-pair ou les messages se propagent de voisin en voisin).
Les autres noeuds qui voient l'annonce telecharge l'archive
automatiquement et la mettent en cache.

**Le stockage des donnees** : les apps peuvent stocker des donnees
(votes, commentaires, preferences) dans un espace de stockage
replique entre les noeuds. C'est base sur un CRDT (Conflict-free
Replicated Data Type) — en gros, si deux personnes modifient la
meme donnee en meme temps, le systeme sait fusionner sans conflit.
Ca fonctionne meme hors-ligne : chaque noeud continue de
fonctionner independamment, et les donnees se synchronisent
automatiquement a la reconnexion.

**La decouverte** : les noeuds se trouvent via une table de hachage
distribuee (DHT), avec un fallback DNS et un relais WebSocket. Ca
fonctionne derriere un NAT, pas besoin d'IP publique ni de port
forwarding.

## La provenance — savoir d'ou vient le code

C'est le point qui differencie le plus SBFB d'un simple hebergement
de fichiers. Chaque app publiee est accompagnee d'une preuve
cryptographique :

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

## Les curators — la confiance sans moderation

Pas de moderateur central qui decide quoi publier ou quoi supprimer.
A la place, un systeme de **listes de recommandation** :

- N'importe qui peut creer une liste de curator : c'est juste une
  cle Ed25519 et une liste d'apps que vous recommandez
- Vous signez votre liste, elle se propage sur le reseau par gossip
- Les utilisateurs choisissent a quels curators ils font confiance
  et s'abonnent a leurs cles publiques
- Un curator peut dire "je recommande cette app" ou "je la
  deconseille" — mais il ne peut pas la supprimer du reseau

L'idee c'est que chaque communaute (chaque CHATONS par exemple) peut
etre son propre curator. Vous recommandez les apps que vous avez
verifiees pour vos utilisateurs. Un autre CHATONS recommande les
siennes. Les utilisateurs composent leur vision du reseau en choisissant
leurs curators.

Les listes ont un compteur de revision monotone (impossible de revenir
en arriere), et le systeme gere la rotation de cle si vous devez
changer votre cle Ed25519 (fenetre de transition de 14 jours max).

## Le sandbox — comment les apps sont isolees

Les apps tournent dans un iframe du navigateur avec plusieurs couches
de protection :

L'iframe n'a pas le droit d'acceder au DOM de la page parente. Il ne
peut pas faire de requetes reseau vers l'exterieur (le navigateur
bloque ca via une Content Security Policy stricte). Il ne peut pas
lire les cookies ou le stockage local du shell. Il ne peut pas acceder
aux donnees d'une autre app.

La seule facon pour une app de communiquer avec le reseau, c'est via
un **bridge** : un SDK JavaScript (`sbfb-bridge.js`) qui envoie des
messages au shell via `postMessage`. Chaque message a un identifiant
unique pour eviter les interferences, un timeout de 10 secondes, et
le shell verifie que le message vient bien de l'iframe attendu.

Les methodes disponibles sont whitelistees : stocker/lire des donnees,
soumettre une tache de calcul, lire l'etat du reseau, verifier une
provenance, chercher des apps. Rien d'autre.

Cote daemon, tout passe par loopback (127.0.0.1) avec un token
d'authentification de 256 bits. Aucun port n'est expose sur le reseau.

## Factory — l'outil de creation et publication

`sbfb-factory` est un outil en ligne de commande (Rust) qui guide
la creation et la publication d'apps :

**Creer une app** : `sbfb-factory create --template static --name mon-outil`
genere un squelette avec un manifeste `SBFB.json`, un `index.html`,
le SDK bridge, et un README. Il y a deux templates de base : un
statique simple et un avec le bridge integre pour les apps qui ont
besoin de stocker des donnees.

**Valider** : `sbfb-factory validate` verifie que le manifeste est
correct, que les methodes bridge demandees sont dans la liste
autorisee, qu'il n'y a pas de fichiers sensibles (.env, cles privees).

**Tester localement** : `sbfb-factory preview` charge votre app dans
le daemon local en mode ephemere (30 minutes). Vous voyez exactement
ce que les utilisateurs verront, dans le meme sandbox.

**Publier** : `sbfb-factory publish` passe par 11 verifications
automatiques (les "gates") avant de diffuser sur le reseau :
- Validation du manifeste et du format
- Scan de secrets (detection de cles API, tokens AWS, certificats)
- Verification sandbox (pas de symlinks qui sortent du repertoire,
  pas de path traversal)
- Test de connectivite avec le daemon
- Signature de provenance Ed25519
- Diffusion sur le reseau P2P

Toutes ces verifications sauf la derniere tournent en local, hors-
ligne. Si une verification bloquante echoue, la publication s'arrete
et vous dit pourquoi.

Factory inclut aussi un **outil graphique local** (Factory Operator)
qui donne une vue d'ensemble : etat du sprint de developpement,
historique complet des modifications avec diff inline fichier par
fichier, journal des actions, et un chat integre pour poser des
questions sur le projet.

## Le manifeste SBFB.json

Chaque app a un petit fichier JSON qui la decrit :

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

Le champ `bridge.methods` declare explicitement ce dont l'app a
besoin. Si une app ne declare pas `storage_set`, elle ne peut pas
ecrire de donnees. C'est le principe du moindre privilege.

## La reputation — les kudos

Quand un noeud contribue du calcul (par exemple de l'inference IA
avec un GPU) au reseau, il recoit des points de reputation appeles
kudos. Quelques points importants :

- C'est **pas une monnaie**. On ne peut pas transferer ses kudos a
  quelqu'un, les vendre, les echanger. C'est un score lie a votre
  identite de noeud.
- La formule utilise des **rendements decroissants** (logarithme) :
  contribuer 1000 tokens ne donne pas 1000 fois plus de kudos que
  contribuer 1 token. Ca empeche les gros de concentrer toute la
  reputation.
- Les kudos **decroissent dans le temps** (demi-vie ~23 jours) :
  les contributions recentes comptent plus que les anciennes. Un
  noeud inactif perd progressivement du rang.
- Chaque entree kudos est **signee et chainee** par hash BLAKE3 :
  n'importe qui peut telecharger le ledger complet d'un projet et
  verifier que personne n'a triche.
- Un **coefficient de Gini** est calcule en temps reel pour detecter
  si la reputation se concentre chez quelques noeuds. Si le Gini
  depasse 0.70, c'est un signal d'alerte.

## Le vote dans les apps

L'app Ideas Hub montre comment faire du vote decentralise :

Chaque vote est stocke comme une cle dans le stockage P2P :
`votes/{ideaId}/{pubkey_du_votant}`. Comme la cle publique Ed25519
de chaque noeud est unique, c'est impossible de voter deux fois.
Re-cliquer retire le vote (supprime la cle). Les votes se
synchronisent entre tous les noeuds qui ont rejoint l'app.

Ce pattern est generique — n'importe quelle app SBFB peut l'utiliser
pour du budget participatif, des sondages, du peer-review, ou
n'importe quel mecanisme de decision collective.

## Le compute distribue

Un aspect optionnel mais interessant : SBFB permet de partager du
calcul GPU entre les noeuds du reseau. Un utilisateur soumet une
tache (par exemple "resume ce texte"), le reseau la route vers un
noeud qui a un GPU disponible, le resultat revient.

Le consentement est **explicite a 4 niveaux** :
- Niveau 1 : je ne partage mon GPU que pour mes propres projets
- Niveau 2 : pour les projets open source verifies
- Niveau 3 : pour une liste de projets que je choisis
- Niveau 4 : pour tout le monde

Avec des plafonds configurables : watts max, VRAM max, heures max
par jour. Le daemon refuse automatiquement les taches qui depassent
vos limites.

Les resultats sont valides (quorum de verification si plusieurs
workers traitent la meme tache), et les taches suspectes vont en
quarantaine.

## Les apps qui tournent

Trois apps exemples, toutes en HTML/JS pur sans aucune dependance :

**Protocol Explorer** : 6 sections de documentation interactive sur
le protocole. Un panneau live affiche l'etat du reseau en temps reel
(pairs connectes, apps disponibles). Un bouton permet de verifier la
provenance d'une app en un clic — le daemon verifie la signature
Ed25519 et affiche le resultat.

**Ideas Hub** : proposer et voter sur des idees. Les donnees se
synchronisent entre les noeuds automatiquement. Chaque identite ne
peut voter qu'une fois. On peut trier par nombre de votes ou par
date. On peut supprimer ses propres idees.

**Factory Viewer** : affiche les apps du reseau avec leur score de
qualite (Proof Card, 0-100, base sur 7 facteurs de risque : est-ce
que le code est verifie, est-ce qu'un curator l'a recommande, est-ce
que le repo est accessible, etc).

## La securite — ce qu'on garantit et ce qu'on dit clairement

**Garanti aujourd'hui** :
- Identite Ed25519 protegee par permissions fichier (0600)
- Loopback authentifie (token 256 bits + verification de l'origine)
- Sandbox iframe 5 couches (CSP, origin separee, bridge controle)
- Feed public append-only (chaine de hash, impossible d'effacer)
- Kudos verifiables (chaine BLAKE3 + Ed25519)
- Anti-spam Hashcash sur le gossip
- Resistance Sybil couche 1 (un noeud doit exister 7 jours avant
  de participer au gossip)

**Pas encore garanti (et on le dit)** :
- La cle privee n'est pas chiffree au repos (prevu : Keychain macOS
  / DPAPI Windows / libsecret Linux)
- Pas d'audit de securite formel sur iroh (notre couche reseau)
- Rate limiting incomplet sur certains endpoints

On prefere dire clairement ce qui manque plutot que de faire croire
que tout est parfait.

## Etat du projet

- ~1800 tests (1486 Rust, 279 JavaScript), tous verts
- Installeurs Windows, Linux .deb, macOS .dmg
- P2P teste en LAN (Windows <-> Mac) et en WAN (dev <-> VPS Helsinki)
- Licence AGPL-3.0
- Solo maintainer, pas de startup, pas de fondation, pas de token
- Pilote ferme — pas encore de noeuds tiers en production

## Ce que je cherche

Je ne cherche pas de financement. Je cherche un ou deux CHATONS
motives pour tester le protocole en conditions reelles :

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
