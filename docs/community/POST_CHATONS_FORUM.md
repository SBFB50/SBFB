# Post forum CHATONS

**Categorie recommandee** : Cafe du commerce
**Titre recommande** : SBFB - vers une couche applicative communautaire, verifiable et P2P
**Statut** : version prete a poster + annexe de reponses pour le fil
**A remplacer avant publication** : `[lien demo]`, `[canal pilote]`

---

## Version a poster

Salut a toutes et tous,

Je m'appelle Theophile. Je developpe en solo un protocole libre sous AGPL-3.0 appele SBFB. Je viens ici parce que le sujet touche directement a l'auto-hebergement, aux communs numeriques et a la confiance dans le logiciel libre.

Je ne cherche pas de financement, ni des "utilisateurs" au sens startup. Je cherche plutot un ou deux hebergeurs alternatifs prets a regarder un pilote ferme, a me dire ce qui tient, ce qui casse, et ce qui serait inacceptable pour une communaute comme CHATONS.

Point important : ce n'est pas pret pour une mise en production CHATONS. Il n'y a pas encore de noeuds tiers en production ni d'audit de securite formel de toute la pile. Le pilote serait un premier essai externe encadre, pas un service exploitable.

## L'idee

SBFB sert a publier une application web comme un paquet verifiable, puis a la diffuser entre noeuds P2P. L'utilisateur l'ouvre dans son navigateur, dans une iframe sandboxee. Il n'y a pas de serveur central qui detient le catalogue, pas de compte cloud, pas de store applicatif.

Le deuxieme axe, aussi important dans ma vision, est le partage de puissance IA/GPU. Une app SBFB peut rester legere et demander, avec consentement explicite, des taches de traduction, d'analyse, de verification, d'indexation ou de generation a des workers GPU/CPU volontaires. L'objectif n'est pas seulement de decentraliser l'hebergement d'apps, mais aussi de decentraliser la puissance de calcul IA pour qu'elle ne depende pas uniquement de quelques plateformes cloud ou de ceux qui possedent les plus gros moyens.

En pratique :

1. une personne publie une app web depuis un depot source ;
2. le noeud construit une archive, calcule son hash et signe une provenance ;
3. l'annonce se propage en P2P ;
4. les autres noeuds peuvent telecharger, mettre en cache et verifier l'archive qu'ils ouvrent.

Schema court du chemin d'une app :

```text
Depot source + commit
        |
        v
[Factory / daemon local]
        |
        | archive + hash + provenance signee
        v
[Reseau P2P : annonce + blob]
        |
        v
[Noeud lecteur : cache + verification locale]
        |
        v
[App ouverte en iframe sandbox]
        |
        v
[Avis curator choisi par l'utilisateur]
```

Ce n'est pas un remplacant de YunoHost, PeerTube, F-Droid, Sandstorm, Solid ou IPFS. Je le vois plutot comme une brique complementaire qui croise plusieurs idees : distribution d'apps libres, contenu adresse, donnees controlables par les communautes, sandbox navigateur, curation locale, preuve de provenance et mutualisation de puissance IA/GPU opt-in.

## Pourquoi je pense que ca peut parler aux CHATONS

Les CHATONS portent deja une culture de services libres, transparents, decentralises et respectueux des donnees. SBFB essaie d'appliquer cette logique a la distribution d'apps web :

- **alternatif** : pas de plateforme SaaS centrale a qui deleguer la publication ;
- **transparent** : une app peut etre reliee a son depot source et a un commit precis ;
- **ouvert** : le protocole et le code sont sous AGPL-3.0 ;
- **neutre** : le reseau ne decide pas seul ce qui est "bon" ou "mauvais" ;
- **mutualiste** : le calcul IA/GPU peut etre partage volontairement, avec consentement, limites et allowlist ;
- **anti-capture IA** : la puissance de generation, traduction, analyse ou verification ne doit pas etre reservee aux grandes plateformes ;
- **solidaire** : chaque communaute peut recommander ce qu'elle a teste.

Le point cle, pour moi, c'est la curation. Un CHATONS, une association ou un collectif local pourrait tenir sa propre liste signee d'apps recommandees ou deconseillees. Les utilisateurs choisissent les listes auxquelles ils font confiance. Personne ne devient moderateur central du reseau ; chaque communaute garde sa propre politique de recommandation.

| Acteur | Peut faire | Ne peut pas faire |
|---|---|---|
| Publieur | Publier une app et sa provenance | Imposer la confiance |
| Curator CHATONS | Recommander ou deconseiller une app | Supprimer une app du reseau |
| Utilisateur | Choisir les listes qu'il suit | Etre force par un curator global |
| Daemon local | Verifier hash, signature, feed et sandbox | Decider qu'une app est moralement bonne |
| Worker GPU/CPU | Executer des taches opt-in pour des projets autorises | Voir plus que la tache recue ou prendre le pouvoir |

## Ce qui est verifiable

Chaque app peut etre accompagnee d'une provenance :

- depot source public ;
- commit annonce ;
- hash de l'archive ;
- signature Ed25519 du noeud qui publie ;
- verification locale possible par le daemon.

C'est une auto-attestation verifiable, pas un audit externe. Autrement dit : ca ne prouve pas que l'app est "bonne" ou sans faille, mais ca evite le simple "faites-moi confiance". On peut verifier que l'archive recue correspond au hash signe dans une attestation qui annonce un depot et un commit. Ce n'est pas encore une preuve independante que ce commit reconstruit exactement l'archive ; ce niveau dependra de builds reproductibles ou de quorums tiers.

La verification du projet lui-meme a aussi une couche hors GitHub : Woodpecker tourne sur `ci.sbfb.world` comme CI self-hosted. Aujourd'hui, il sert de preuve d'infrastructure et de second chemin de verification avec GitHub Actions. Le reseau qui se recompile vraiment par plusieurs workers independants reste la trajectoire suivante : les fondations build task + quorum SHA256 existent, mais le worker quorum E2E public n'est pas encore un acquis du pilote.

| Niveau affiche | Ce que ca affirme | Ce que ca n'affirme pas |
|---|---|---|
| Upload direct | Une archive existe sur le reseau | Origine du code inconnue |
| Source lisible | Un depot public est declare | L'archive n'est pas encore prouvee par build tiers |
| Provenance signee | Commit + hash + signature du noeud sont lies | Auto-attestation, pas audit externe |
| Signature verifiee | Le daemon a verifie la signature et le hash | Pas encore build reproductible tiers |
| CI self-hosted | `ci.sbfb.world` verifie le repo hors GitHub Actions | Pas encore quorum public multi-worker |
| Build reproductible | Un tiers reconstruit le meme hash | Futur, pas acquis pour le pilote |
| Feed verifie | L'historique observe est integre par hash-chain | Ne prouve pas que l'app est bonne ou sure |

Le reseau vise aussi a garder une trace verifiable des publications, recommandations, changements de source/provenance et, cote Factory, des actions locales journalisees. Ce n'est pas encore un journal global de toutes les executions utilisateur.

## Les garde-fous deja en place

La securite n'est pas renvoyee a "plus tard". Les couches deja posees combinent sandbox navigateur, CSP stricte, bridge `postMessage` avec methodes autorisees, daemon local en loopback avec token et controles Host/Origin, provenance signee, hash d'archive, feed append-only et Factory Operator avec actions allowlistees et logs.

Je ne presente pas ca comme "securite parfaite". C'est une defense en profondeur deja implementee en partie, encore a auditer formellement, et justement exposee au pilote pour etre critiquee.

## La vision a terme

La vision longue n'est pas "heberger une page HTML en P2P". J'aimerais arriver a un reseau ou une communaute peut :

- creer une application avec un outil local, puis la publier comme paquet verifiable ;
- distribuer cette app entre noeuds, sans store central ;
- garder des donnees applicatives simples synchronisees entre les noeuds qui participent ;
- retrouver les apps et leurs preuves via une recherche locale et verifiable ;
- choisir ses curators, donc sa propre couche sociale de confiance ;
- mutualiser du calcul IA/GPU opt-in pour des usages utiles : traduction, indexation, verification, analyse de documents, generation assistee, aide a la publication.

L'objectif final est une couche applicative communautaire : permettre a une association, un CHATONS, une cooperative, une ecole, un tiers-lieu ou un collectif local de creer, publier, maintenir, auditer, recommander, retrouver et executer ses propres outils web verifiables. Pas seulement des micro-outils : des apps de coordination, de decision collective, de documentation, de traduction, de publication, d'archivage, de recherche, de collaboration ou de calcul mutualise.

L'idee forte est que le pouvoir ne soit pas concentre dans un SaaS, un store, une forge, un hebergeur central ou un fournisseur IA. Les communautes doivent pouvoir choisir leurs curators, leurs regles d'adoption, leurs niveaux de preuve et, a terme, les ressources de calcul qu'elles acceptent de mutualiser.

Le partage de puissance IA/GPU est donc un sujet central, pas une option decorative. Dans les briques actuelles, il existe deja un worker GPU/CPU, un dispatch de taches, un consentement GPU opt-in par niveau de partage, des limites watts/VRAM/heures et un ledger kudos compute/task avec rendement logarithmique, vieillissement EMA et metriques fairness. Mais je ne veux pas d'un modele ou seuls les gens avec de gros GPU accumulent toute l'influence : le GPU doit compter quand il rend un service reel, sans devenir le seul pouvoir du reseau. La reconnaissance du stockage, relais, revue, docs, traduction, curation et validation humaine est une direction de gouvernance a construire, pas une regle de vote du pilote.

La base technique testee maintenant est le socle de cette vision : distribution P2P, provenance, sandbox navigateur, recherche locale, curators et apps de demo. Le pilote sert a verifier si ce socle tient avant de brancher les briques plus ambitieuses dessus.

Je ne veux pas non plus invisibiliser Factory et le chat local. Factory existe aujourd'hui comme CLI, Operator local et Viewer de preuves pour apps simples et templates statiques ; l'atelier non-dev complet et les templates riches sont l'arc suivant. Un premier Operator peut demarrer une session depuis un context-pack et journaliser les actions. Les alias type `@security`, `@audit`, `@dev` decrivent le modele cible de routage vers les preuves ; pour le pilote, cela doit rester de la lecture et de l'explication assistee, sans autorite et sans promesse de RRV complet.

## Ce que j'aimerais tester

Je cherche un pilote ferme avec une ou deux personnes, pas un lancement public large.

| Dans le pilote ferme | Hors pilote / plus tard |
|---|---|
| Installer 2 ou 3 noeuds | Mise en production CHATONS |
| Publier 2 ou 3 apps simples | Apps critiques ou donnees sensibles |
| Verifier P2P, sandbox et provenance | Audit securite complet |
| Tester vouch/disendorse curator | Gouvernance communautaire finale |
| Documenter les No-Go | Lancement public large |

Un echec m'interesse autant qu'une reussite. Si le P2P ne tient pas, si l'UX est incomprehensible, si la securite n'est pas assez lisible, ou si le modele curator pose probleme, c'est exactement le retour que je viens chercher.

Ce pilote ne teste pas toute la vision future. Il teste le commencement de la couche applicative : est-ce qu'une app peut etre creee, publiee, retrouvee, ouverte, verifiee, puis recommandee ou non par une communaute ?

## Etat honnete du projet

Ce qui est deja testable : daemon local, interface web, sandbox, bridge, publication depuis source, provenance signee, feed verifiable, recherche locale, Proof Cards, Factory CLI/Operator/Viewer pour apps simples, worker GPU/CPU avec consentement opt-in, dispatch de taches, ledger kudos compute/task, Ideas Hub, premieres briques curator, CI self-hosted `ci.sbfb.world` et fondations build task/quorum SHA256.

Ce qui est prepare pour le pilote : protocole de test Gate 1, chemin Babel via `static-reader`, migration d'app web classique, vouch ou disendorse curator.

Ce qui reste trajectoire : SearchManifest P2P, RRV reseau complet, page Factory non-dev, Babel publique finale, gouvernance communautaire, quorum de build tiers, worker quorum E2E, partage IA/GPU multi-noeuds a grande echelle et apps avancees type capteurs, Alexandria ou surveillance de foret.

Je ne veux pas faire passer le projet pour plus mature qu'il ne l'est. Aujourd'hui, c'est une experimentation avancee, libre, testable, mais encore en pilote ferme.

## Exemple concret

Une app de budget participatif local pourrait etre publiee comme app SBFB :

- le code est dans un depot public ;
- le noeud publie l'app avec une provenance signee ;
- les votes sont stockes dans un espace replique entre noeuds ;
- chaque identite locale vote une fois ;
- un CHATONS peut recommander cette app dans sa propre liste curator ;
- un autre collectif peut choisir de ne pas la recommander.

L'objectif n'est pas de remplacer un service heberge classique quand il fonctionne tres bien. L'objectif est de tester un autre mode de distribution : une app web verifiable, partagee entre noeuds, avec des recommandations communautaires plutot qu'un store central.

## Ce que je demande

Si une ou deux personnes ici veulent regarder le projet avec un oeil d'hebergeur, donnez-moi votre username Codeberg : je peux vous partager le depot prive en lecture seule, faire une demo courte, ou repondre publiquement aux questions dans ce fil.

Le depot Codeberg n'est pas encore public ; l'acces pilote se fait en read-only le temps de garder un cadre de test ferme.

Demo ou discussion : `[lien demo]` / `[canal pilote]`

Merci d'avance pour les critiques, surtout les critiques dures. C'est le meilleur moment pour les recevoir.

Bonne journee a toutes et tous.

---

## Annexe pour repondre dans le fil

Cette annexe n'est pas a poster d'un bloc dans le premier message. Elle sert a repondre proprement aux questions techniques sans improviser.

### SBFB en une phrase

SBFB est un protocole de distribution d'apps web en P2P : une app est empaquetee, signee, diffusee entre noeuds, puis ouverte localement dans un iframe sandboxe avec un bridge limite.

### Difference avec YunoHost

YunoHost installe et administre des services sur un serveur. SBFB ne cherche pas a remplacer cette couche. SBFB vise plutot une couche de distribution et de verification d'apps web entre noeuds, avec provenance et cache P2P. Un CHATONS pourrait tres bien heberger des services classiques avec YunoHost et tester SBFB a cote pour des apps communautaires experimentales.

### Difference avec IPFS

IPFS adresse tres bien le contenu. SBFB ajoute un cadre applicatif : manifeste, provenance, sandbox navigateur, bridge autorise, feed public, curators et donnees applicatives repliquees. L'objet n'est pas seulement "un fichier a telecharger", mais "une app web ouvrable et verifiable".

### Difference avec F-Droid

F-Droid repose sur un depot et une chaine de publication Android. SBFB reprend l'intuition "catalogue d'apps libres et verifiables", mais pour des apps web et sans autorite unique de catalogue. Les curators sont multiples : chaque communaute peut signer ses propres recommandations.

### Difference avec une blockchain

SBFB n'a pas de token, pas de minage, pas de consensus global et pas de monnaie. Le feed est append-only et verifiable par hash-chain pour l'historique observe par un noeud ou synchronise entre pairs, mais il ne cherche pas a resoudre une verite globale ni un consensus economique mondial. Les preuves servent a verifier une histoire de publication, pas a creer un marche.

### Securite du sandbox

Le modele client repose sur plusieurs couches :

- iframe sandbox sans acces direct a la page parente ;
- politique CSP stricte pour les apps non fiables ;
- bridge `postMessage` avec methodes limitees ;
- daemon expose en `127.0.0.1` avec token ;
- separation entre app sandboxee et API locale.

Formulation prudente a garder : "defense en profondeur", pas "securite parfaite". Une app malveillante reste un risque, et le pilote doit justement servir a verifier les angles morts.

### Securite plus large que le sandbox

Le sandbox navigateur n'est qu'une couche. La securite du projet repose aussi sur :

- daemon expose en loopback, avec authentification locale par token ;
- separation entre UI locale, app sandboxee et API protocole ;
- manifests et provenance signes ;
- hash-chain pour le feed append-only ;
- curators separes de la publication brute ;
- Factory Operator avec actions allowlistees et logs d'audit ;
- systeme de sprint/phase/review pour garder les decisions et les gates visibles dans le depot.

La formulation juste est donc : SBFB met en place une defense en profondeur et une tracabilite verifiable, mais ne pretend pas remplacer un audit de securite formel.

### Provenance et limites

La provenance SBFB est une auto-attestation :

- elle annonce un depot et un commit ;
- elle lie l'archive recue a un hash et a une signature locale ;
- elle peut etre verifiee localement ;
- elle ne remplace pas un audit humain ;
- elle ne prouve pas encore un build reproductible par un tiers.

Le bon wording est donc : "source-verifiable" ou "provenance verifiable", pas "logiciel sur a executer".

### Curators

Un curator ne supprime pas une app du reseau. Il signe une liste de recommandations ou de deconseils. Les utilisateurs choisissent les curators qu'ils suivent. C'est une couche sociale de confiance, pas une police centrale du reseau.

Etat prudent a expliquer : la curation existe aujourd'hui au niveau primitives/listes et operations feed selon l'etat sprint recent. L'UI de gouvernance complete, le quorum social, le dissent/timeline et la delegation restent post-pilote. Certaines specs protocolaires plus anciennes doivent etre relues comme historiques.

### Donnees personnelles

Le daemon n'a pas besoin de compte cloud et ne centralise pas les requetes. Pour un pilote CHATONS, il faut eviter les donnees sensibles : apps de demonstration, vote non critique, documentation, formulaires de test. Les questions RGPD serieuses doivent etre traitees avant tout deploiement avec de vraies donnees personnelles.

### Kudos, anti-capture et vote

Le partage IA/GPU n'est pas seulement une idee future : il existe deja comme brique de la pile avec worker GPU/CPU, taches, consentement opt-in et limites d'usage. Ce qui reste a durcir, c'est l'usage multi-noeuds a grande echelle, la gouvernance autour de ces contributions et la facon de ne pas transformer le GPU en source unique de pouvoir. Sinon, le protocole reproduirait le probleme qu'il essaie d'eviter : ceux qui possedent le materiel le plus cher finissent par peser le plus.

Le cadrage a garder :

- les kudos ne sont pas une monnaie, pas un token, pas un stake, pas un actif transferable ;
- les kudos servent a rendre visibles des contributions verifiables ;
- le compute GPU est une contribution, mais pas la seule ;
- les contributions doivent pouvoir etre separees par projet, pour reconnaitre les personnes qui ont vraiment aide ce projet precis ;
- le vote ou le poids social eventuel doit rester plafonne, auditable et resistant aux Sybil.

Modele vise a terme :

- **compute** : execution de taches utiles, avec rendement decroissant pour eviter que 100 fois plus de GPU donne 100 fois plus de pouvoir ;
- **stockage** : heberger et servir des apps ou donnees utiles au reseau ;
- **relais** : aider les pairs a se trouver, relayer du gossip, garder de la disponibilite ;
- **projet** : code, docs, traduction, correction, design, accessibilite, revue, moderation, validation humaine ;
- **curation** : recommander, deconseiller, expliquer les risques, tenir une liste utile pour une communaute.

Ce qui existe deja dans le code est plus limite : ledger kudos surtout lie aux contributions compute/task, hash-chain, rendement logarithmique, vieillissement EMA et metriques fairness. Le modele multi-familles et le vote pondere par contribution restent une trajectoire a traiter prudemment, pas un acquis du pilote CHATONS.

Phrase utile :

> Le but n'est pas de recompenser ceux qui ont le plus gros GPU, mais de rendre visible une diversite de contributions au reseau et aux projets, sans transformer cette reputation en argent ou en pouvoir capturable.

### Factory, RRV et Babel

Factory est l'atelier local : creer, valider, previsualiser, publier. RRV est la couche de recherche verifiable : retrouver les apps, leurs preuves et plus tard leurs sources. Babel est le premier dogfood Factory prevu cote roadmap, pas le protocole lui-meme. Le bon cadrage public est :

- **aujourd'hui** : Factory existe comme CLI, Operator local et Viewer de preuves pour apps simples et templates statiques ; publication, provenance et recherche locale commencent a exister ;
- **pilote** : prouver qu'une app simple circule et se verifie entre noeuds ;
- **apres pilote** : enrichir RRV, durcir Factory vers un atelier non-dev complet, tester une app canari plus ambitieuse comme Babel.

Formulation plus ambitieuse mais exacte :

> Factory est l'atelier qui doit permettre de transformer une idee en app SBFB verifiable : template, manifeste, preview sandboxee, scans, provenance, publication, Proof Card et trace de revue.

### Matrice de maturite des briques

Cette matrice sert a eviter deux erreurs : minimiser la vision, ou presenter comme acquis ce qui est encore research.

**1. Testable aujourd'hui / code-backed**

- daemon local loopback, shell web, iframe sandboxee, bridge `postMessage` allowliste ;
- publication depuis source, `SBFB.json`, archive hashee, provenance signee Ed25519 ;
- Browse, blob-serve, deploy-from-repo, feed local append-only, materialisation et verification hash-chain ;
- Curator lists et operations feed `CuratorVouched` / `CuratorDisendorsed` cote protocole, avec UI complete encore a durcir ;
- recherche locale FTS5 `@protocole` et endpoint local de recherche ;
- Proof Cards locales : score de completude de preuve, facteurs bruts, endpoint daemon, composant UI ;
- Factory CLI : create, validate, diff, scan-secrets, preview, publish ;
- templates Factory `static` et `static-reader` ;
- Factory provenance, preview ephemere, publish path vers daemon ;
- Factory Viewer, Factory Operator et `sbfb-factory process` ;
- context-pack et prototype de chat qui lit le process/protocole sans devenir autorite ;
- Ideas Hub : proposition, vote, retrait, stockage applicatif replique ;
- kudos ledger, hash-chain, log-utility, EMA et metriques fairness.
- CI self-hosted Woodpecker sur `ci.sbfb.world`, en complement de GitHub Actions ;
- fondations LT-7 : task `build`, build executor MVP et quorum SHA256 DB-persistent pour comparer des artefacts.

**2. Chemins de pilote prepares**

- Gate 1 : 9 tests manuels pour installation, P2P, publication, Babel via Factory, feed sync, restart, stabilite 24 h, recherche et Proof Card ;
- Babel via `static-reader` : chemin cree/valide/previsualise/publie, mais pas encore l'app vitrine finale ;
- migration d'une app React classique vers SBFB avec contraintes sandbox, bridge, assets relatifs et verification post-deploiement ;
- review communautaire : un curator peut recommander/deconseiller, mais la gouvernance complete demande encore UI, timeline et dissent.

**3. Prochain arc S71+ / durcissement**

- SearchManifest opt-in : partager une partie de l'index local entre noeuds, signe, verifiable, rate-limite, sans discovery publique par defaut ;
- RRV Core : mieux croiser resultats, preuves, feed, provenance, Proof Cards et plus tard sources/citations ;
- gouvernance UI : timeline curator, dissent, vouch/disendorse visibles et explicables ;
- Factory hardening : templates plus riches, meilleur diff, tests adversariaux, packaging plus accessible aux non-devs ;
- Proof Card comme evenement/feed op ou element exportable dans un proof pack, selon le prochain design ;
- build tiers et worker quorum E2E : passer de l'auto-attestation et de la CI self-hosted vers une preuve plus forte, avec plusieurs builders independants.

**4. Research / vision long terme**

- kudos multi-familles : compute, stockage, relais, projet, curation, avec poids ajustables, Gini par famille et protections anti-capture ;
- vote declencheur de taches : seuil de votes, budgets compute, anti-spam, generation par workers, review humaine avant publication ;
- compute GPU : court terme = decomposition en taches independantes ; temps reel / model parallelism WAN = long terme, pas pilote CHATONS ;
- Babel complet : corpus, source-policy, traductions opt-in, validation humaine, roles de contribution visibles ;
- surveillance de foret : pipeline tuiles publiques, inference vision, aggregation CRDT, dashboard verifiable ;
- Alexandria, D&D, render farm, registre vivant, apps de crise et capteurs citoyens : bons showcases pour tester stockage, compute, curation et Factory, mais pas promesses du premier pilote ;
- inspirations p2panda/public feed : operations append-only, replay, offline catch-up et sync durable comme patterns possibles, sans creer un deuxieme statut "open source" ni remplacer la provenance SBFB.

### Comment un projet plus gros passe par Factory

Le chemin cible pour un projet plus riche n'est pas "coder une app dans un coin puis l'uploader". Il ressemble plutot a :

1. choisir un template ou un domain pack ;
2. declarer les donnees, les capacites bridge et les limites dans `SBFB.json` ;
3. generer l'app avec Factory ;
4. previsualiser l'app dans le meme sandbox que les utilisateurs ;
5. lancer les gates : manifeste, sandbox, secrets, dependances, provenance, publish ;
6. publier via le daemon ;
7. afficher les preuves dans Browse, RRV et les Proof Cards ;
8. laisser le chat/Operator aider a expliquer l'etat sans devenir l'autorite finale.

Cette chaine est importante pour les projets ambitieux : elle evite que Factory devienne juste un generateur de code opaque. Chaque app produite doit laisser une trace verifiable de son template, de ses capacites, de son commit, de son archive, de ses preuves et de ses revues.

### Exemples de projets avances via Factory

**Babel Reader**

Statut public a garder : premier dogfood Factory prevu, pas le protocole lui-meme. Babel sert a prouver qu'une app reelle peut etre creee, publiee, retrouvee et verifiee via SBFB.

Ce que Babel peut montrer :

- corpus de textes sous domaine public ou licence compatible ;
- source-policy : droit, redistribution P2P, attribution, opt-out ;
- lecture offline et bibliotheque locale ;
- provenance des textes et des versions ;
- plus tard : traduction par workers opt-in, corrections humaines, validations et roles de contribution visibles.

**Surveillance de foret**

Statut public a garder : showcase avance / design, pas livrable pilote CHATONS. L'idee est une app de cartographie et d'alerte sur images satellite publiques.

Ce que ca montrerait :

- images Sentinel-2, Landsat ou FIRMS decoupees en tuiles ;
- taches d'inference vision envoyees a des workers GPU volontaires ;
- resultats signes ou au minimum rattaches a leurs taches ;
- aggregation en CRDT : carte de chaleur, zones a verifier, alertes ;
- dashboard web publie comme app SBFB, verifiable et curationnable.

**Capteurs citoyens**

Des Raspberry Pi de quartier publient des mesures de qualite de l'air ou d'humidite. Une app dashboard Factory lit les donnees, affiche les tendances et peut demander du calcul opt-in pour de la prediction locale. Pour un CHATONS, c'est probablement plus proche d'un pilote associatif realiste que la vision satellite/GPU complete.

**Alexandria / corpus de connaissance**

Une bibliotheque locale de contenus publics ou redistribuables, servie par blobs P2P, cachee par les noeuds et interrogeable localement. Le point fort est la distribution de donnees et la consultation offline, pas forcement le calcul GPU.

**Donjon & Dragon / jeu collaboratif IA**

Exemple moins CHATONS mais utile pour montrer un autre axe : etat partage CRDT + generation LLM par GPU volontaires + app publiee comme paquet SBFB. A garder plutot en annexe ou demo technique, pas dans le premier message communautaire.

### Chat, alias et acces au protocole

Le chat fait partie de la trajectoire Factory/Operator, mais il faut le cadrer prudemment. Le premier niveau est un Operator local qui peut partir d'un context-pack au lieu d'une conversation vide et journaliser les actions. Le context-pack donne un acces en lecture a l'etat du protocole, aux apps publiees, aux preuves, aux Proof Cards, au feed, aux curators, aux actions Factory et aux artefacts de sprint/phase.

Les alias publics a expliquer simplement :

- `@research` : comprendre les choix techniques et les sources ;
- `@dev` : lire une app, un manifeste, un commit, une implementation ;
- `@audit` : verifier les preuves, les gates et les regressions ;
- `@security` : regarder sandbox, loopback, tokens, bridge, crypto ;
- `@product` : resumer le statut, le pilote, les risques et la roadmap.

Limite importante : ces alias ne donnent pas de privilege automatique. Ils orientent le chat vers les bons corpus et les bonnes preuves. Les actions sensibles restent controlees par l'Operator, allowlistees et journalisees. Le chat peut aider a comprendre et preparer une action ; il ne remplace pas les revues, les signatures, les gates ni les commits. Pour le pilote, il faut le presenter comme lecture/explication assistee, pas comme RRV complet ni autorite de verification.

Phrase utile si quelqu'un demande si c'est "juste une IA" :

> L'objectif n'est pas de mettre une IA au-dessus du protocole, mais de donner au mainteneur local un assistant qui lit les preuves du protocole, les artefacts Factory et les gates de sprint, sans devenir lui-meme la source de verite.

### Installation

Formulation prudente :

- "des installeurs ont deja ete produits et valides en interne" ;
- "le pilote fournira les binaires et les hash a verifier" ;
- "ne pas redistribuer les binaires du pilote hors du groupe de test".

Eviter de promettre une installation publique tant que le pilote ferme et la stabilite 24 h ne sont pas passes.

### Questions probables

**Est-ce pret pour un CHATONS en production ?**

Non. C'est trop tot. Je cherche un pilote ferme, pas une mise en production.

**Pourquoi poster ici si ce n'est pas pret ?**

Parce que les mauvais choix d'architecture se corrigent mieux avant le lancement public. Les hebergeurs alternatifs verront vite les risques que je rate seul.

**Qui decide ce qui est visible ?**

Chaque utilisateur choisit ses curators. Un curator recommande ou deconseille ; il ne controle pas tout le reseau.

**Que se passe-t-il si une app est dangereuse ?**

Le protocole peut afficher son niveau de preuve, le sandbox limite ses capacites, et les curators peuvent la deconseiller. Mais il ne faut pas pretendre qu'une preuve de provenance rend automatiquement une app sure.

**Est-ce que ceux qui ont des GPU auront plus de pouvoir ?**

Pas automatiquement, et ce serait un mauvais objectif. Le GPU peut compter comme contribution quand il rend un vrai service, mais la vision kudos est multi-contribution : stockage, relais, revue, code, docs, traduction, curation, validation humaine. Le vote pondere par kudos, s'il arrive, doit etre par projet, plafonne et auditable. Sinon on remplace un monopole de plateforme par un monopole de materiel.

**Est-ce que SBFB remplace l'hebergement web ?**

Non. C'est une autre maniere de distribuer certaines apps web. Les services classiques restent mieux adaptes pour beaucoup d'usages.

**Pourquoi AGPL ?**

Le coeur SBFB est sous AGPL-3.0-or-later pour empecher qu'un acteur privatise l'infrastructure commune : toute modification du protocole ou du daemon exploitee comme service reseau doit rester redistribuable avec son code source.

Les apps publiees sur SBFB ne deviennent pas automatiquement AGPL par le simple fait d'utiliser le protocole. En revanche, pour etre recommandees comme apps verifiees dans le pilote, elles doivent publier leur source, declarer une licence libre/open source, fournir une provenance verifiable et rester auditables par les noeuds et les curators. Une app qui modifie, embarque ou derive du code AGPL de SBFB doit respecter l'AGPL.

## Checklist avant publication

- Remplacer `[lien demo]` et `[canal pilote]`.
- Verifier que le depot Codeberg partage aux pilotes en lecture seule correspond bien au commit annonce.
- Preparer les hash des binaires du pilote.
- Poster seulement la section "Version a poster" en premier message.
- Garder l'annexe pour les reponses dans le fil.
- Ne pas promettre "production", "audit complet" ou "open source verified" sans expliquer le niveau exact de preuve.
