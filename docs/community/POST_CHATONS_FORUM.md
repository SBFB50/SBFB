# Post forum CHATONS — version forum

**Categorie** : Cafe du commerce
**Titre** : Distribution d'apps web en P2P, sans serveur central — retour d'experience et proposition de pilote

---

Salut a toutes et tous,

Je m'appelle Theophile, dev solo francophone. Je bosse depuis un an et
demi sur un protocole libre (AGPL-3.0) qui touche directement a ce que
font les CHATONS, et je pense que c'est le bon endroit pour en parler.

## En deux mots

Vous connaissez F-Droid : un store d'apps Android ou le code source
est verifie, ou il n'y a pas de Google qui decide ce qui a le droit
d'exister, et ou n'importe qui peut heberger son propre depot.

J'ai construit la meme chose, mais pour les **apps web**. Ca s'appelle
SBFB. C'est un protocole P2P (ecrit en Rust, base sur iroh) ou
n'importe qui publie une app web — HTML, React, Python dans le
navigateur, notebook Jupyter, peu importe — et le reseau la distribue
automatiquement. Pas de serveur central. Pas de Vercel. Pas de GitHub
Pages. Pas de compte Google.

## Pourquoi ca pourrait interesser un CHATONS

Aujourd'hui si vous voulez distribuer une petite app web utile a votre
communaute — un outil de vote, une page d'info, un reader d'articles —
il faut un serveur web, un nom de domaine, un certificat TLS, de la
maintenance. Ou passer par un hebergeur cloud.

Avec SBFB, vous installez un daemon Rust sur votre machine (~21 Mo,
un seul binaire, zero config), vous publiez votre app en une commande
(`sbfb-factory publish`), et elle se propage sur le reseau P2P. Les
autres noeuds la telecharge automatiquement. Si votre machine est
eteinte, les gens qui l'ont deja vue continuent de la servir.

### Ce qui parle a la culture CHATONS

**Source verifiable** : chaque app publique est deployee depuis son
repo Git. Le protocole genere automatiquement une preuve
cryptographique (Ed25519 + BLAKE3) qui lie le commit source au hash
de l'archive. N'importe qui peut verifier que l'app vient bien de son
code. C'est le modele F-Droid applique au web.

**Curators au lieu de moderation** : pas de moderateur central qui
decide quoi publier. A la place, des listes de recommandation signees
Ed25519 et propagees par gossip. Chaque CHATONS pourrait etre un
curator pour sa communaute : vous signez la liste des apps que vous
recommandez, vos utilisateurs s'abonnent a votre cle publique. Si
vous deconseilleriez une app, vous signez un `CuratorDisendorsed` — mais
personne ne peut la supprimer du reseau.

**Pas de monnaie, pas de token** : quand quelqu'un contribue du calcul
GPU au reseau, il recoit des points de reputation (kudos). C'est
un score, pas une monnaie — pas de transfert, pas de marche, pas de
speculation. La formule utilise des rendements decroissants pour
empecher les gros de dominer, et un coefficient de Gini est calcule en
temps reel pour surveiller les inegalites.

**AGPL-3.0** : le code reste libre, meme si quelqu'un le deploie en
SaaS. Google interdit l'AGPL dans ses depots internes — c'est le
signal que ca marche comme barriere anti-capture.

**Fonctionne en local** : sur un reseau WiFi sans internet, les apps
se propagent entre les noeuds. Les donnees des apps (votes, contenus)
se synchronisent en P2P via un CRDT. Si le reseau se partitionne,
chaque partie continue de fonctionner, et les donnees fusionnent
automatiquement a la reconnexion.

## Comment ca marche concretement

### Pour publier une app

```bash
# Creer une app a partir d'un template
sbfb-factory create --template static --name mon-outil --output ./mon-outil

# Modifier le code (c'est juste du HTML/JS/CSS)
# ...

# Publier sur le reseau
sbfb-factory publish --repo-url https://codeberg.org/moi/mon-outil
```

La commande `publish` passe par 11 verifications automatiques avant de
diffuser : validation du manifeste, scan de secrets (regex AWS/API
keys), detection de path traversal, et signature de provenance Ed25519.
Tout ca tourne en local, hors-ligne. La publication sur le reseau P2P
est la derniere etape.

### Pour utiliser les apps

Le daemon ouvre un navigateur local. L'utilisateur voit une grille
d'apps disponibles sur le reseau (page Browse). Il clique sur une app,
elle s'ouvre dans un iframe sandbox strict — l'app ne peut pas faire de
requetes reseau, pas acceder aux fichiers, pas lire les donnees d'une
autre app. La seule communication possible passe par un bridge
postMessage avec des methodes whitelistees (stocker des donnees,
soumettre une tache, lire l'etat du reseau).

### Pour devenir curator

Vous generez une cle Ed25519, vous signez une liste des apps que
vous recommandez, et vous partagez votre cle publique. Les
utilisateurs collent la cle dans l'interface "Curators" du shell et
voient automatiquement les apps que vous recommandez. Votre liste est
propagee par gossip — pas besoin d'heberger un serveur de catalogue.

## Les apps qui tournent deja

Trois apps exemples (HTML/JS pur, zero dependance npm, 3-5 fichiers
chacune) :

- **Protocol Explorer** : documentation interactive du protocole avec
  panneau live "etat du reseau" (combien de pairs connectes, quelles
  apps disponibles, verification de provenance en un clic)

- **Ideas Hub** : proposer et voter sur des idees. Les donnees se
  synchronisent entre noeuds en P2P. 1 identite = 1 vote, pas de
  double vote possible (la cle publique Ed25519 sert d'identifiant).
  Re-cliquer retire le vote.

- **Factory Viewer** : app sandbox qui affiche les apps du reseau avec
  leur score de qualite (Proof Card 0-100 base sur 7 facteurs de
  risque)

## Ce qui est different de ce que vous connaissez

**vs Yunohost** : Yunohost exige un VPS avec IP stable. SBFB tourne
sur un laptop derriere un NAT. Le catalogue Yunohost est centralise —
si le serveur tombe, zero nouvelles installs. SBFB : la decouverte est
P2P avec 3 niveaux de fallback (DHT + DNS + WebSocket).

**vs PeerTube** : PeerTube federe la video via ActivityPub. SBFB federe
n'importe quelle app web via gossip. PeerTube a besoin d'une instance
serveur avec PostgreSQL. SBFB : un binaire, zero base de donnees a
maintenir.

**vs IPFS** : IPFS distribue des fichiers sans provenance — tu ne sais
pas d'ou vient un hash. SBFB accompagne chaque archive d'une preuve
cryptographique qui lie le commit source au hash, signee par le noeud
qui a publie. Et les apps SBFB ont un stockage collaboratif P2P integre
(CRDT), pas juste des fichiers statiques.

**vs F-Droid** : F-Droid a une equipe de review centralisee et une
seule cle de signature. SBFB : zero moderation centralisee. N curators
independants signent leurs recommandations. Une app publiee est live
immediatement.

## Etat du projet

- Un an et demi de dev, 70 sprints, ~1800 tests (1486 Rust, 279 JS)
- Installeurs Windows, Linux .deb, macOS .dmg
- P2P teste en LAN (Windows <-> Mac) et en WAN (dev <-> VPS Helsinki)
- Licence AGPL-3.0
- Solo maintainer, modele OpenBSD — pas de startup, pas de fondation,
  pas de token, pas d'investisseur
- Pilote ferme (pas encore de noeuds tiers en production)

## Ce que je cherche

Je ne cherche pas de financement. Je cherche un ou deux CHATONS
motives pour tester le protocole en conditions reelles :

1. Installer un noeud SBFB a cote de vos services existants
2. Publier 2-3 petites apps utiles a votre communaute
3. Se mettre en curator l'un pour l'autre
4. Voir ce qui marche et ce qui casse

Le resultat attendu : "3 apps publiees, 2 noeuds qui se voient, le
P2P tient" — ou "ca marche pas pour telle raison et voila ce qu'il
faudrait changer". Les deux sont utiles.

Le code est sur [lien repo]. Dispo pour une demo en visio de 15 minutes
ou pour repondre a vos questions ici. Si vous voulez les details
techniques pousses (architecture crypto, endpoints daemon, sandbox
iframe, modele de menaces), j'ai une doc technique complete que je
peux partager.

Bonne journee
