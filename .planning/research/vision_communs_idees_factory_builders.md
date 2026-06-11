# Vision — Communs, sbfb-ideas partagé, structureurs de fondations, builders

Date : 2026-06-11 (post-UX-ARRIVAL `e980d7e` + hotfix outbox `38c5578`)
Statut : **note de vision et d'orientation** — débat PO + re-examen adversarial
(workflow 5 angles indépendants, sourcés). RIEN ici n'est engagé dans un
sprint ; ce doc alimente les kickoffs futurs (S76+) et l'audit.
Origine : session de test live PC↔VPS↔Mac de l'UX d'arrivée hybride, qui a
révélé le gap sbfb-ideas et déclenché le débat produit.

---

## 1. Le constat technique déclencheur : sbfb-ideas n'est PAS partagé

Vérifié en live (2026-06-11) : chaque daemon crée SA propre namespace
iroh-docs pour une app de storage — PC `doc_id=b483d5c1…`, Mac
`doc_id=bff77536…`. Le « storage P2P » du bridge (`storage_get`/`storage_set`)
est en réalité un stockage **local par nœud**. Les idées/votes postés sur un
pair ne sont pas vus par les autres.

Pour un vrai doc partagé il faut :
1. L'AUTEUR de l'app crée la namespace et produit un **ticket** ;
2. le ticket est **distribué** avec l'app (annonce/annuaire, à côté de
   `archive_hash`) ;
3. chaque pair **rejoint** la namespace (import du ticket) au lieu d'en créer
   une — iroh-docs synchronise ;
4. le **modèle d'écriture** est à trancher (cf. §3 et §6).

Garde-fou issu du re-examen (Ostrom, précédent Usenet) : **ne PAS livrer une
namespace globale unique en écriture ouverte** (écriture permissionless sans
gouvernance = trajectoire spam Canter & Siegel 1994 → abandon). Préférer des
**namespaces imbriquées** : une racine + des forks communautaires portant
chacun sa politique d'écriture.

## 2. Décisions d'orientation issues du débat (PO, 2026-06-11)

Ces orientations sont débattues et assumées, pas encore implémentées :

- **Écriture permissionless** : tout le monde peut écrire (pas de gate
  d'écriture global — un gate global ferait du nœud un admin central, ce qui
  contredit la décision gelée « zéro modération centrale »).
- **Chaque écriture est signée Ed25519 + coûte un petit PoW** (anti-flood
  sans gardien) — voir §6.4 pour la forme corrigée (PoW dormant
  différentiel).
- **Le filtrage est une affaire de VUE, par nœud** : pubkeys suivies mises en
  avant ; le reste en pool ambiant flat. « Je valide les pairs » est légitime
  comme réglage de MA vue, jamais comme barrière d'entrée des autres.
- **Kudos = tri mou, jamais gate dure** (effet Matthew + bootstrapping d'un
  nouveau honnête ; kudos est per-projet non transférable par décision gelée).
- **REFUS du web-of-trust transitif et de tout classement global** — identifié
  par le PO lui-même : « les curators de confiance qui boostent un nouveau »
  = moteur à clusters monopolistiques. Pas de trône global ⇒ rien à
  monopoliser. La promotion se fait par **choix individuel** (le geste
  « S'abonner »), jamais par score propagé.

C'est cohérent avec ce qui vient d'être LIVRÉ (UX-ARRIVAL `e980d7e`) : grille
« Tes sources » (choix explicites, catalog-backed Ed25519) vs section
« Découvert sur le réseau » (ambiant flat, cappé, non classé) ; nœuds
observés = métadonnées seules, no-fetch ; PoW gossip + rate-limit à l'ingest.

## 3. La vision produit (langage PO)

**L'arrivée.** Camille découvre SBFB. Deux zones : « Tes sources » (vide — sa
bibliothèque, personne ne l'a remplie pour elle) et « Découvert sur le
réseau » (déjà vivant — ce que le réseau annonce, sans permission demandée à
quiconque). Personne ne valide son arrivée : pas de portier, pas d'admin. Ce
qu'on contrôle, c'est sa propre vue, pas l'accès des autres. Elle ouvre une
app de l'ambiant (téléchargée à la demande), s'abonne au nœud si ça lui
plaît → ses apps passent dans « Tes sources ». Sa bibliothèque se construit
par ses gestes.

**sbfb-ideas.** Elle poste une idée : signée (attribuée), au coût d'un petit
péage de calcul (imperceptible pour un humain, ruineux pour un robot — pas un
comité qui filtre), visible de tous — mise en avant chez ceux qui la
suivent, à plat dans l'ambiant chez les autres. Personne ne décide pour le
réseau si son idée mérite d'être vue ; elle gagne en visibilité par une somme
de choix individuels.

**Le principe en une phrase** : n'importe qui peut arriver, lire et
contribuer ; le spam est arrêté par un péage de calcul, pas par un gardien ;
personne n'occupe un trône central — chaque nœud cure sa propre vue, et on
monte en visibilité parce que des gens vous choisissent un par un.

## 4. La vision pipeline : idée → structureur de fondations → builders

**Le pipeline.** sbfb-ideas (partagé) fournit les idées → un nouveau RÔLE,
le **« structureur de fondations »**, transforme une idée brute en fondation
exploitable (roadmap, sprints, plans) via **Factory** — l'outil agentique
avec lequel ce projet lui-même est construit → des **« builders »**
communautaires, en GPU partagé (S76) + gros modèles, construisent l'app.

**Les briques existent** : Factory (réel, dogfoodé), ProviderRouter S72
(dispatch Claude/Ollama/réseau), GPU cross-machine = S76, provenance
Ed25519+SLSA, guardrails + quorum sur les résultats réseau, atelier fork S74.

**Le rôle « structureur de fondations »** est la pépite produit de la
vision : tout le monde ne code pas — certains STRUCTURENT. Prendre une idée
brute et la transformer en fondation exploitable (roadmap, sprints, critères)
est un travail réel, valorisable, et Factory est exactement son outil. Le
structureur gagne de la renommée quand ses fondations se font construire :
une voie de réputation par la valeur produite, pas par un cartel.

### 4.1 Le débat marché vs bien commun — les trois murs et leur relecture

**Trois murs initiaux (lecture marché)** : (1) qualité de la construction
autonome non résolue — sortir une roadmap est facile, construire un logiciel
correct automatiquement est le front de recherche ; (2) incitation — pourquoi
un builder dépenserait son GPU et son électricité pour l'idée d'un inconnu,
sans argent ; (3) confiance — comment savoir que le build implémente
fidèlement la roadmap ?

**Réponse du PO (lentille bien commun, assumée anti-capitaliste)** — citée
parce qu'elle est le cœur philosophique du projet :
- les projets sont des **biens communs à but d'améliorer l'humanité** —
  pas des produits à rentabiliser ;
- si un groupe prend un projet, **c'est qu'il l'intéresse** — l'intérêt
  intrinsèque EST l'incitation ;
- tout ce qui se passe est **inscrit pour toujours dans chaque nœud** — la
  renommée s'accumule sur l'historique complet, elle ne s'efface pas ;
- la confiance dans le résultat : « on s'en fout s'il y a des dérives » —
  **plusieurs équipes peuvent créer la solution**, la pluralité fait le tri ;
- les idées seront prises en main par des **communautés humanistes**.

**Ce que cette relecture change réellement (acté au débat)** :
- Le mur de l'incitation **tombe largement** : les communs de compute ont
  déjà scalé sans un centime (Folding@home ~2,4 exaFLOPS au pic COVID,
  BOINC, Wikipedia, Linux) — l'erreur de la lecture marché était de
  présupposer qu'il faut un paiement. La registre permanent par nœud est
  une réputation plus durable que n'importe quel profil de plateforme.
- Le mur de la confiance est **remplacé par la pluralité** : on ne vérifie
  pas chaque build, on laisse plusieurs versions exister (fork-and-compete) ;
  la charge se déplace vers la couche découverte/sélection — qui est
  précisément ce que l'UX-ARRIVAL vient de commencer à construire.
- Le mur de la qualité **se ramollit** : plus besoin d'autonomie totale ;
  l'humain structure et juge, l'agent exécute, la pluralité corrige —
  c'est de l'augmentation, atteignable par incréments.
- Restent deux goulots honnêtes : **réunir assez d'humains alignés** (la
  ressource rare n'est pas l'argent, c'est l'attention alignée) et rendre la
  **pluralité navigable** (la couche de sélection devient load-bearing).

→ Cette relecture est elle-même **affinée** par le re-examen adversarial
(§6) : elle était encore trop optimiste sur « les communautés » comme unité
de production (§6.1) et trop dure sur le PoW (§6.4) — mais sa structure
(intérêt intrinsèque + pluralité + augmentation) survit.

## 5. L'alliance sociétale : « le numérique en commun », l'énergie, l'orientation des usages

### 5.1 L'alignement avec le « numérique en commun » (corpus LFI 2022)

Le PO a versé au débat le livret numérique de la France insoumise — non comme
adhésion partisane du projet, mais comme **preuve qu'une très grande
communauté humaniste organisée existe en France** dont le programme converge
point par point avec ce que SBFB est déjà :

| Le programme demande | SBFB l'est / le fait |
|---|---|
| « Le numérique comme bien commun » | Bien commun assumé, AGPL, non-monétaire, vision anti-startup/anti-financiarisation (memory `vision_model`) |
| Lois de déconcentration anti-monopole | Anti-monopole **gravé dans le protocole** : pas de serveur central, pas d'admin, pas de ranking global, 5 verrous anti-recentralisation — pas une politique appliquée, une structure |
| « Clouds de confiance décentralisés, associatifs et pluriels » | C'est littéralement SBFB : des nœuds que n'importe qui héberge, pluriels, sans propriétaire |
| « Hébergements de proximité », « fabrication distribuée » (fablabs) | Chaque nœud est un hébergement de proximité ; le fork outillé + Factory = fabrication distribuée du logiciel |
| Logiciel libre, interopérabilité, anti-captivité | AGPL, apps à source vérifiable, zéro lock-in |
| Refus de la censure privée des plateformes | Zéro modération centrale ; chaque nœud cure SA vue |
| Transparence, maîtrise des données | Tout signé Ed25519, provenance SLSA, historique inscrit dans chaque nœud |
| Chiffrement, vie privée | Crypto de bout en bout du protocole, durcissement loopback, sandbox |

**Nuance structurante** : le programme porte DEUX fils — le commun garanti
par l'ÉTAT (cloud souverain public, agences) et le commun PAR LA BASE
(associatif, pluriel, fablabs). SBFB est le fil n°2 : personne ne le
« garantit » d'en haut, des pairs le font tourner. Les deux sont
complémentaires (le programme prévoit explicitement de « soutenir les
projets de clouds décentralisés associatifs ») : SBFB ne se présente pas
comme infrastructure d'État mais comme **l'outil que cette base peut
s'approprier** — cohérent avec « les idées prises en main par des
communautés humanistes ».

**Les deux conditions d'adoption par ce milieu** (les valeurs sont déjà au
rendez-vous, la crédibilité se joue ailleurs) :
1. **La sobriété** — c'est LE point que ce public scrutera : le programme
   cible « les usages énergivores des serveurs, comme le minage ». Réponse
   construite au débat puis chiffrée au re-examen (§5.2, §6.4).
2. **L'accessibilité non-technique** — le programme martèle l'illectronisme
   (13 M de Français) ; un outil de dev ne touche pas cette base. Cohérent
   avec la règle projet déjà gelée « v1.0 = prod ready non-technicien ».
3. Et la leçon des réseaux (§6.3) précise COMMENT aborder ce milieu : pas
   par l'affinité idéologique (SSB est mort en l'ayant) mais par un **outil
   utile dès la première session** — arriver avec 10-30 apps réellement
   utiles, pas avec un manifeste.

### 5.2 L'argument énergétique français

La France a un des mix électriques les plus bas-carbone du monde
(~50-60 gCO₂/kWh, nucléaire + renouvelables, contre ~400+ pour la moyenne
mondiale). Conséquence objective : une heure de GPU en France émet une
fraction de ce qu'elle émet ailleurs — le reproche « le compute IA = mauvais
pour le climat » est matériellement plus faible ici.

Mais la sobriété du programme a DEUX axes, et le nucléaire n'en règle qu'un :
- **Axe carbone** : la France gagne. Mieux : un réseau de compute
  décentralisé bas-carbone sur sol français coche « centres de calcul
  régionaux » + « souveraineté » du programme — le GPU partagé devient un
  argument de souveraineté énergético-numérique, pas une excuse.
- **Axe gaspillage de principe** (« règle verte » : ne pas prendre plus que
  ce qui se reconstitue) : propre ne veut pas dire non-gaspillé. D'où la
  distinction décisive actée au débat :
  - le **GPU qui construit des apps = compute UTILE** (l'énergie produit un
    bien commun) — défendable sur les deux axes en France ;
  - le **PoW = travail inutile par conception** — c'est le seul maillon
    attaquable, et sa défense n'est pas « c'est propre » mais « c'est micro,
    borné par l'usage, et dormant » (réhabilitation chiffrée au §6.4 :
    2-3 J/écriture, précédent Tor 2023).

### 5.3 L'orientation des usages — l'argument structurel du PO

Constat PO : l'utilisation massive actuelle de l'IA part dans la génération
de texte/image **sans réelle utilité** (slop SEO, contenu d'engagement,
images jetables). Précision actée : c'est vrai de la majorité MARCHANDE, pas
de toute génération (la traduction — Babel —, l'accessibilité, l'éducation
sont de la génération utile). Le bon cadrage : **le profit oriente
aujourd'hui le compute vers le futile ; un commun l'oriente vers ce que les
humains valorisent.**

L'argument est structurel, pas moral : le slop existe PARCE QUE la pub et
l'engagement le rémunèrent. Un commun **non-monétaire retire ce moteur** —
personne ne brûle du GPU pour du spam SEO sans modèle d'affaires. Le même
compute, débranché de l'incitation marchande et rebranché sur ce que des
communautés alignées valorisent, s'oriente mécaniquement vers l'utile :
« des applications d'intérêt public pour la planète » (PO). C'est exactement
la position du programme cité : le problème n'a jamais été l'IA, c'est **à
qui elle obéit**.

Limite honnête, assumée au débat : SBFB ne peut pas IMPOSER l'usage vertueux
par le protocole — ce serait recréer un comité, un trône, un GAFAM bis.
L'orientation est une **garantie culturelle, pas une garantie de code** :
elle vaut ce que vaudra la communauté qui s'empare de l'outil. C'est
précisément pourquoi la couche découverte/curation (§2, §6.3) et
l'accessibilité sont les pièces qui décident si la promesse devient réalité.

---

## 6. Re-examen adversarial (5 angles indépendants, sourcés)

Workflow 5 agents (2026-06-11), chacun chargé de réfuter/corriger les
conclusions ci-dessus. Synthèse de ce qui SURVIT, ce qui est CORRIGÉ, ce qui
est RÉFUTÉ.

### 6.1 « Les communautés productrices » n'existent pas — le pari réel est le mainteneur solo augmenté

- Tous les communs numériques réussis sont portés par un noyau infime,
  instable, souvent salarié : **1 % des éditeurs ont écrit 77 % de Wikipedia**
  (Matei & Britt, ~40 % de turnover du top 1 % toutes les ~5 semaines) ; le
  noyau Linux est écrit à ~85-90 % par des développeurs **payés** (Linux
  Foundation) ; MediaWiki est développé par le staff de la fondation.
- Les communs de BUILD sur le pattern exact visé (idée → vote → build
  communautaire) sont **morts** : Assembly.com (2013-2015, 4M utilisateurs,
  projets green-lit retombés au silence), Quirky (~185 M$ levés, faillite
  2015 sur le coût d'intégration). Le goulot n'est jamais l'idée ni le
  démarrage : c'est le **portage dans la durée**.
- Taux de base : **17 % de succès** sur 145 475 projets SourceForge (Schweik
  & English, MIT Press 2012) ; prédicteur n°1 = le **besoin personnel du
  leader** — donc « prendre l'idée d'un autre » part avec le facteur de
  succès principal en moins.
- **Fork-and-compete est largement un mythe à petite échelle** : les forks
  convergent presque toujours vers UN survivant (XFree86→X.org,
  OpenOffice→LibreOffice, io.js→Node re-mergé…) parce que l'attention de
  mainteneur est LA ressource rare que le fork divise. Sous la masse
  critique, deux équipes concurrentes = deux projets sous-critiques morts.
  Le framing défendable : **fork = sauvetage** (droit de reprise à coût
  quasi nul — ce que l'atelier fork S74 livre vraiment), pas marché
  concurrentiel d'idées.
- La réputation immuable **retient** le noyau actif (+60 % de productivité
  après un barnstar… mesuré sur le top 1 % — Restivo & van de Rijt 2012),
  elle ne **recrute** pas de masses.
- **Ce qui tient** : (a) les communs réussis sont conçus POUR leur 1 %, pas
  pour la foule ; (b) l'**augmentation agentique** (Factory + ProviderRouter
  + GPU) attaque exactement les conditions de Benkler (granularité, coût
  d'intégration) qui séparaient build et compute passif — c'est le **seul
  composant de la vision sans précédent négatif**, donc le seul pari
  réellement nouveau.

> **Pari falsifiable reformulé** : « UN humain augmenté (Factory + agents +
> GPU partagé) peut porter une app de l'idée à la maintenance » — testable en
> un dogfood. Le pitch honnête est « commons de mainteneurs solo augmentés »
> (cohérent avec l'ethos OpenBSD du projet), PAS « des communautés humanistes
> prendront les idées ».

### 6.2 Ostrom : SBFB n'a pas « zéro gouvernance » — il a une gouvernance de fait, non signée

- Principes structurellement **satisfaits** : P7 droit à s'organiser (curator
  lists, abonnements, fork outillé — garanti par le protocole) ; P1
  frontières (partiel : Ed25519+PoW+invites M19, mais identités gratuites) ;
  P8 imbrication (proto).
- Principes **absents** — meilleurs prédicteurs documentés d'échec des
  communs (méta-analyse Cox 2010, 91 cas) : **P3 choix collectif** (les
  règles vécues par tous sont fixées par un mainteneur solo : constitution
  BDFL non écrite), **P5 sanctions graduées** (seul outil = mute binaire
  individuel), **P6 résolution de conflits** (fork = exit sans voice ; la
  block size war Bitcoin 2015-2017 est le cas d'école du fork destructeur
  causé par l'absence de P6).
- « Zéro gouvernance » n'existe jamais empiriquement : Debian s'est doté
  d'une constitution (1998) PARCE QUE les conflits l'ont exigé ; Wikipedia a
  créé l'ArbCom dès 2004 ; sans structure formelle, des élites informelles
  non redevables remplissent le vide (Freeman 1972, « The Tyranny of
  Structurelessness »). Le rôle « structureur de fondations » concentrera le
  coût d'intégration (Benkler) → il deviendra le **centre de pouvoir de
  fait** ; sans P3/P5/P6 il sera non redevable.
- Le mainteneur solo est une **surface d'attaque** documentée : XZ utils 2024
  (burnout exploité par ingénierie sociale). Correction factuelle : OpenBSD
  n'est pas « solo sans structure » (fondation depuis 2007, noyau de
  committers).
- « Chaque nœud cure sa vue » n'est PAS de la polycentricité (choix de
  consommateur atomisé) ; la vraie primitive polycentrique de SBFB est la
  **curator list** — analogue aux instances du fediverse, qui ont fait
  émerger des sanctions graduées collectives (fediblock) sans autorité
  centrale.

> **Implications** : (1) écrire **GOVERNANCE.md** — la constitution existe
> déjà (décisions gelées + 5 verrous + interdits de vocabulaire), la publier
> avec procédure d'amendement explicite convertit un pouvoir invisible en
> pouvoir auditable — le projet signe tout SAUF sa propre gouvernance ;
> (2) promouvoir la **curator list au rang d'unité de gouvernance** :
> multi-signataires (FROST DKG déjà dans le codebase), chartes affichables,
> deny/mute-lists partageables par abonnement = sanctions graduées 100 %
> compatibles avec le refus de ranking global ; (3) traiter le **bus factor
> comme un P0** (succession des clés/domaines/ancre — LT-2 Radicle résout la
> disponibilité du code, pas la légitimité) ; (4) **institutionnaliser le
> fork** : lignée visible entre forks (B.6 open-source⇒provenance en est la
> moitié) pour que fork-and-compete soit navigable.

### 6.3 Autopsie des réseaux : le prédicteur n°1 est l'utilité égoïste de la première session

- **SSB** (quasi-mort) : tué par la friction d'arrivée (heures de sync,
  multi-device impossible) — il AVAIT la communauté alignée (solarpunk) et le
  web-of-trust ; ça n'a pas suffi. **L'alignement idéologique seul n'a sauvé
  aucun des sept cas étudiés.**
- **Nostr** (vivant, petit : ~12-17k pubkeys actives/jour) : son PoW par
  message (NIP-13) est resté **marginal** — le vrai travail anti-spam est
  fait par les relais sélectifs et le **filtrage local par graphe de
  follows** (jamais global). Centralisation de fait sur 3-4 relais.
- **Mastodon** (plateau ~1M MAU) : chaque dogme anti-découverte a été
  partiellement abandonné sous la pression de la rétention (instance par
  défaut imposée 2023, recherche réintroduite, trending ajouté). Le
  flat-sans-aucune-aide a un coût mesuré (« ghost town effect »).
- **IPFS** : content-addressing sans engagement de seed durable = liens
  morts ; usage réel via gateways HTTP centralisées. SBFB a la bonne
  contre-mesure (keep_online M18 + seed volontaire) — qui ne s'active que si
  les apps sont **désirées**.
- **BitTorrent** (LE succès durable) : demande pré-existante, incitation dans
  le protocole, et une couche découverte JAMAIS décentralisée — externalisée
  à des index web **centralisés pluriels**. **F-Droid** : curation centrale
  humaine assumée, contrat étroit et tenu. **Radicle** : trésorerie
  abondante, adoption minuscule — l'argent ne crée pas l'usage.
- Pattern gagnant historique : **transfert décentralisé + curation plurielle
  identifiable** — pas l'absence de curation, et pas la sélection
  décentralisée flat (aucun précédent de succès).

> **Implications** : (1) **sanctuariser** le budget « pair frais voit du
> contenu en < 1 min » (acquis aujourd'hui : ancre VPS + entries au boot) —
> tester chaque évolution (S76 GPU, S77 sharding) contre ce budget ;
> (2) préparer des **starter packs de curators** (listes signées importables
> en 1 clic, pattern Bluesky/Nostr) — compatible avec les 5 verrous car
> c'est un choix individuel ; (3) le pool ambiant flat cappé est viable à N
> petit, mais prévoir la suite (curation plurielle par listes) AVANT le mur
> Mastodon ; (4) cold-start : viser une communauté importable avec un
> **besoin d'outil quotidien**, pas une affinité idéologique (le pilote :
> 10-30 apps réellement utiles seedées AVANT ouverture).

### 6.4 Le PoW réhabilité — et transformé : dormant, différentiel, modèle Tor

- **Correction quantitative** du jugement « gaspillage par conception » : le
  hashcash SBFB (18 bits) coûte **~2-3 joules par écriture** — ~300-500×
  moins qu'une recherche web (~1000 J). À difficulté fixe, le coût total est
  **borné linéairement par l'usage légitime** : rien à voir avec le minage
  compétitif (enchère énergétique ouverte). L'amalgame PoW anti-spam = minage
  est économiquement faux.
- **Précédent décisif : Tor** (non-profit, jugé par le même public
  éco-militant) a adopté en 2023 le PoW EquiX pour les onion services, à
  **effort par défaut ZÉRO qui n'escalade que sous attaque** — après avoir
  conclu qu'aucune défense sans argent ni gardien ne fait mieux.
- Limite réelle (Laurie & Clayton 2004) : aucun PoW uniforme ne bloque les
  botnets (compute volé) sans surtaxer les machines faibles légitimes
  (coût régressif mobile vs RTX 5080). Réponse moderne : difficulté
  **différentielle par âge d'identité** + escalade sous attaque +
  memory-hard si besoin (Argon2id réduit l'asymétrie GPU, déjà dans
  Cargo.lock).
- **Les alternatives « propres » échouent toutes** au test
  sans-argent-sans-gardien : RLN/Waku exige membership on-chain + slashing
  financier ; preuve-de-personne = autorité hardware + échecs documentés
  (Worldcoin) ; kudos-burn violerait frontalement la décision gelée
  « kudos non-monnaie » ; proof-of-useful-work **inverse l'incitation** (si
  le travail du spammeur est utile, spammer coûte moins net) et reste de la
  recherche ouverte.
- Le PoW protège la **bande passante**, jamais l'**attention** : dans SBFB le
  pool ambiant est flat et cappé, donc le spam n'achète aucune visibilité —
  il ne peut qu'évincer par crowding → la mitigation est le **sampling par
  identité** (déjà identifié au carry S75), pas plus de difficulté.
- **SBFB a déjà les primitives du modèle Tor** : `AgeWitness` (first-seen
  attesté, gate ≥ 7 jours, Sprint 22) + `EscalatingPolicy`
  (relay_pow_policy per-topic). Transformer le hashcash toujours-actif en
  **« PoW dormant différentiel »** (≈ 0 pour les identités établies, plein
  tarif pour les fraîches, escalade sous flood) = **du câblage, pas un
  chantier** — et le wire PoW est librement redéfinissable avant le go-live.

> **Verdict anti-spam pour sbfb-ideas partagé** : PoW one-shot renforcé à la
> PREMIÈRE écriture d'une pubkey sur le doc (coût de création d'identité,
> pattern S/Kademlia), puis rate-limit par identité + micro-PoW résiduel
> dormant ; jamais de gate kudos dure ; documenter l'argumentaire énergétique
> chiffré (2-3 J vs 1000 J, coût borné, précédent Tor) dans le THREAT_MODEL
> ou un doc produit.

### 6.5 Le pipeline : le maillon manquant n'est pas la structuration — c'est le champion

- **Aucune étude mesurée n'identifie la qualité/structuration de la spec
  comme prédicteur** qu'une idée soit menée au bout. Les hackathons
  produisent exactement ce que Factory promet (prototype + plan + équipe) et
  **< 5 % des projets survivent à 5 mois** (Nolte, CSCW 2020). Decide Madrid :
  21 000+ propositions citoyennes permissionless → **2 abouties** (~0,01 %).
  GSoC : ~80-88 % de complétion DANS le programme, mais rétention prédite par
  les **liens humains mentor-contributeur**, pas par la spec ni le stipend.
- Le seul filtre au succès mesuré (NLnet/NGI Zero, ~226 projets financés) est
  une **curation experte** — exactement ce que SBFB refuse au niveau global.
  La compensation décentralisée existe déjà dans le protocole : les **curator
  lists comme comités de review pluriels et concurrents** (« la liste
  NLnet-like de tel collectif »), sans ranking global.
- BountySource est mort de son **escrow monétaire** (rachats crypto,
  confiscation, insolvabilité, fonds volés) : la décision non-monétaire de
  SBFB **évite un mode de mort réel**, elle n'est pas une naïveté.
- Prédicteurs de survie traduits en P2P : compute garanti (GPU partagé S76 =
  financement en nature) + usage réel immédiat (**un nœud qui DÉPLOIE l'app
  construite dès la semaine 1** = l'intégration opérationnelle).

> **Implications produit** : (1) faire du **« claim » le pivot du pipeline** —
> un marqueur signé Ed25519 « je prends cette idée », horodaté, public :
> c'est le champion qu'on rend visible et gratifiant ; une idée non réclamée
> est statistiquement morte, et Factory n'outille que l'étape d'APRÈS le
> claim ; (2) mesurer le funnel idée → réclamée → roadmap → app publiée → app
> maintenue, avec un base rate honnête : **2 idées abouties sur 50 = un
> succès** au regard des précédents ; (3) créer des liens sociaux structurés
> (binôme structureur-builder, canal par projet) — c'est ce qui retient, pas
> les kudos seuls.

---

## 7. Conclusions consolidées (ce qui remplace quoi)

| # | Conclusion du débat initial | État après re-examen |
|---|---|---|
| 1 | « Des communautés humanistes prendront les idées » | **Reformulé** : des micro-noyaux de 1-3 humains augmentés par l'agentique. Pari falsifiable : « un humain + Factory + GPU = une app portée jusqu'à la maintenance ». |
| 2 | Incitation résolue par précédent (Folding@home…) | **Corrigé** : le compute passif ne prouve rien pour le build (conditions de Benkler). Ce qui tient : la réputation retient le noyau ; l'augmentation agentique est le seul élément sans précédent négatif. |
| 3 | Pluralité fork-and-compete remplace la confiance | **Réfuté à petite échelle** (le fork divise l'attention rare). Reformulé : **fork = droit de sauvetage**, et lignée visible entre forks. |
| 4 | « Zéro gouvernance » | **Réfuté** : gouvernance de fait BDFL, à rendre auditable (GOVERNANCE.md) ; curator lists = unité polycentrique ; bus factor = P0. |
| 5 | « PoW = gaspillage par conception, remplaçable » | **Réfuté quantitativement** (2-3 J, coût borné). Reformulé : **PoW dormant différentiel** modèle Tor (AgeWitness + EscalatingPolicy déjà présents) ; aucune alternative compatible avec les décisions gelées. |
| 6 | Flat-sans-ranking absolu comme couche de sélection | **Corrigé** : viable à N petit, aucun précédent de succès à l'échelle. Voie : **curation plurielle par listes signées** (starter packs, comités de review concurrents) — toujours zéro ranking global. |
| 7 | La structuration (Factory) abaisse le coût = maillon manquant | **Réfuté** par tous les précédents mesurés : le maillon manquant est le **champion pré-engagé**. Le produit doit rendre le **claim signé** central ; Factory outille l'après-claim. |
| 8 | Orientation des usages par le non-monétaire | **Tient** (retirer le moteur du slop est structurel) — garantie culturelle, pas de code. |
| 9 | Alignement « numérique en commun » + énergie FR | **Tient**, avec deux conditions : sobriété défendue par les chiffres + le PoW dormant ; accessibilité non-technicien. |
| 10 | Prédicteur de survie du réseau | **Nouveau** : utilité égoïste < 1 min en première session + communauté importée par BESOIN (pas par idéologie). Budget à sanctuariser à chaque sprint. |

## 8. Candidats d'action (à router vers les kickoffs/audits, RIEN d'engagé)

1. **sbfb-ideas partagé** : namespace racine + ticket distribué avec l'app ;
   namespaces imbriquées (pas de pool global unique) ; écriture signée +
   PoW one-shot première-écriture puis dormant ; vue filtrée par nœud.
2. **PoW dormant différentiel** (câblage AgeWitness + EscalatingPolicy +
   difficulté par âge d'identité) — avant go-live, wire librement éditable.
3. **GOVERNANCE.md** : publier la constitution de fait (décisions gelées,
   5 verrous, procédure d'amendement minimale, succession/bus-factor).
4. **Curator lists comme unité de gouvernance** : multi-sig FROST, chartes,
   deny-lists partageables ; starter packs importables 1 clic.
5. **Le « claim » d'idée** : marqueur signé « je prends », funnel mesuré,
   binôme structureur-builder ; base rate de succès honnête (~2/50).
6. **Budget première session** : test permanent « pair frais voit du contenu
   en < 1 min » contre chaque évolution (S76 GPU, S77 sharding).
7. **Dogfood falsifiable du pari central** : un humain + Factory + GPU
   partagé porte UNE app de l'idée à la maintenance.
8. Déjà routés à l'audit S75/S76 : sampling anti-crowding du pool ambiant,
   publisher-binding du registre observed, TTL récepteur des annonces.

## 9. Sources principales du re-examen

Benkler « Coase's Penguin » (2002) ; Schweik & English, *Internet Success*
(MIT Press 2012, 145k projets SourceForge) ; Matei & Britt (Purdue, 250M
edits Wikipedia) ; Restivo & van de Rijt (PLoS ONE 2012) ; Robles &
González-Barahona (2012, 220 forks) ; Cox, Arnold & Villamayor-Tomas (2010,
méta-analyse 91 cas Ostrom) ; O'Mahony & Ferraro (AMJ 2007, Debian) ;
Halfaker et al. (2013, déclin Wikipedia) ; Freeman, « The Tyranny of
Structurelessness » (1972) ; De Filippi & Loveluck (2016, gouvernance
Bitcoin) ; Laurie & Clayton, « Proof-of-Work Proves Not to Work » (WEIS
2004) ; Tor Project, « Introducing Proof-of-Work Defense for Onion
Services » (2023) ; WAKU-RLN-RELAY (arXiv 2207.00117) ; Baumgart & Mies,
S/Kademlia (2007) ; Nolte et al. (CSCW 2020, hackathons) ; Silva et al.
(JSS 2020, GSoC) ; études Decide Madrid/CONSUL ; FediDB ; nostr.band ;
incident XZ utils (2024) ; Assembly.com post-mortem (HN 10555710) ; faillite
Quirky (2015) ; mort de BountySource (2023).
