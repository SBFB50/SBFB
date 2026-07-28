# 10 — Héberger n'est pas publier

## Phrase courte pour le livre

> Deux nœuds peuvent conserver la même archive et la rendre plus disponible ; ils en gardent les octets sans devenir les auteurs de la publication signée.

## Fonctionnalité Nexus représentée

L'image distingue trois rôles matériels :

1. la verrière supérieure montre l'origine : une artisane applique une marque unique à un coffre bleu fermé ;
2. les deux baies inférieures montrent la disponibilité : deux lieux égaux conservent chacun une copie strictement identique du coffre ;
3. les deux gardiens montrent l'hébergement : ils veillent sur les copies avec les mains vides, sans outil permettant de fabriquer, repeindre ou signer l'objet.

Le disque d'ambre entaillé n'est ni une récompense ni un label de qualité. Il matérialise la liaison stable entre un contenu et sa provenance. Sa forme, sa position et son défaut volontaire restent identiques sur l'original et les deux copies.

L'illustration ne prétend pas qu'un objet matériel puisse être dupliqué sans fabrication. Elle emploie cette impossibilité comme métaphore des octets adressés par contenu : deux nœuds peuvent posséder exactement le même blob sans produire deux nouvelles publications.

## Preuves dans le dépôt live

État inspecté : `HEAD 9134af64`, le 27 juillet 2026. Le dossier `docs/book/nexus-v1/` est actuellement non suivi par le dépôt principal ; les références au livre décrivent donc le manuscrit de travail présent sur disque, pas un contrat déjà commité.

### Contrat du livre

- `docs/book/nexus-v1/09_reseau_apps_compute.md:37-43` distingue les annonces, les contenus immuables, les états partagés et les relais de transport. Un relais fait circuler des octets sans devenir l'autorité de publication.
- `docs/book/nexus-v1/09_reseau_apps_compute.md:45-49` formule directement la fonction : conserver et servir l'archive d'autrui augmente sa disponibilité sans transformer l'hébergeur en auteur ; la publication reste liée à l'identité, à la signature et à la provenance.
- `docs/book/nexus-v1/03_une_journee_dans_nexus.md:136` montre plusieurs ateliers qui activent une application puis choisissent de servir son archive.

### Contrat et implémentation Rust

- `crates/nexus-shell-daemon/src/seed_protocol.rs:4-8` décrit le protocole par lequel un pair récupère, épingle et conserve une archive.
- `crates/nexus-shell-daemon/src/seed_protocol.rs:20-38` impose la vérification de la requête et du hash BLAKE3 ; le seeder réépingle les octets de l'archive signée par l'auteur sans signer de nouvelle provenance. Le même fichier distingue l'hébergement demandé de l'hébergement volontaire d'un contenu public.
- `crates/nexus-shell-daemon/src/feed_sync.rs:113-135` sépare explicitement la déclaration du seeder de la provenance de l'application : le nœud signe seulement l'affirmation qu'il conserve le blob.
- `crates/nexus-coordinator-rs/src/public_feed.rs:89-105` encode `SeedAnnounced` avec l'identité du seeder et le hash de l'archive, tout en précisant que cette opération ne réattribue jamais la paternité.
- `crates/nexus-coordinator-rs/src/db.rs:790-801` conserve les couples `(project_id, archive_hash)` épinglés localement et couvre aussi bien le premier seeder auteur que les seeders volontaires distants.
- `crates/nexus-shell-daemon-core/src/browse.rs:120-127` sépare l'ancre d'hébergement joignable de l'autorité sur la provenance, qui reste la signature de l'auteur.
- `crates/nexus-shell-daemon-core/src/browse.rs:737-760` maintient l'adresse de contenu BLAKE3 de l'auteur lors de la découverte et essaie l'ancre puis d'autres seeders sans leur conférer d'autorité.
- `crates/nexus-shell-daemon/src/blob_serve_http.rs:130-172` construit une liste bornée de fournisseurs ; un fournisseur mensonger peut provoquer un échec de connexion, mais il ne peut substituer d'autres octets au hash demandé.
- Le test `crates/nexus-shell-daemon/src/seed_api.rs:2141-2234` utilise deux nœuds Iroh réels : le second récupère et épingle un blob public distant, puis le test vérifie que les octets servis sont exactement ceux de l'auteur, sans nouvelle provenance.

### Historique Git pertinent

- `f5c25757` — 5 juin 2026 : récupération P2P des archives découvertes afin qu'elles puissent réellement être servies.
- `4c1acc57` — 7 juin 2026 : politique locale persistante `keep_online` et épinglage des blobs.
- `b76a0848` — 8 juin 2026 : protocole inter-nœuds authentifié de demande d'hébergement.
- `66a9409b` — 8 juin 2026 : annonce distincte des seeders, réémise après redémarrage, sans réattribution de la provenance.
- `821aa8c2` — 10 juin 2026 : annuaires de nœuds signés et séparation explicite entre ancre de découverte et auteur.
- `00104501` — 10 juin 2026 : récupération multi-fournisseurs par hash de contenu.

## Limites honnêtes de l'état actuel

La propriété « un seeder ne devient pas auteur » est fortement matérialisée et testée dans le code. Deux limites empêchent néanmoins de lire l'image comme une promesse absolue de disponibilité :

- `SeedAnnounced` est une affirmation signée de conservation. `crates/nexus-coordinator-rs/src/public_feed.rs:95-99` précise que le comptage peut surestimer la disponibilité ; seule une récupération réussie prouve qu'un fournisseur sert réellement les octets.
- `crates/nexus-test-harness/tests/cross_daemon_blob.rs:5-10` ne réalise pas, en environnement isolé, le téléchargement complet d'un daemon externe vers l'autre. Ce chemin dépend de tests avec connectivité de relais. Le test ciblé de `seed_api.rs` exerce toutefois deux nœuds Iroh réels dans le processus.

Une autre frontière doit rester nette : réhéberger les mêmes octets ne republie rien. En revanche, modifier l'archive ou redéployer un fork produit un nouvel objet et une nouvelle provenance locale. Le prompt ne représente que le premier cas.

## Tradition visuelle choisie

### Le vitrail narratif de Chartres et sa communauté professionnelle vivante

Le choix associe un langage historique documenté à un métier toujours transmis :

- le [Centre international du Vitrail — technique du vitrail](https://www.centre-vitrail.org/fr/musee-du-vitrail/la-technique-du-vitrail/) décrit l'assemblage traditionnel de morceaux de verre coloré par des baguettes de plomb, technique employée depuis le Moyen Âge ;
- son dossier sur les [vitraux légendaires de Chartres](https://www.centre-vitrail.org/fr/vitraux-de-chartres/) documente leur grammaire narrative en médaillons ainsi que leur état matériel et leurs transformations au fil du temps ;
- les [formations actuelles du Centre](https://www.centre-vitrail.org/fr/formations-vitrail/) attestent la transmission contemporaine du tracé, de la coupe, du sertissage, de la soudure et de la peinture sur verre par des professionnels ;
- le [manuel du ministère de la Culture](https://www.culture.gouv.fr/thematiques/monuments-sites/ressources/les-guides-guides-techniques-fascicules-et-manuels/manuel-de-conservation-restauration-et-creation-de-vitraux) définit un corpus professionnel partagé pour la conservation, la restauration et la création de verrières.

La légende éditoriale devra parler d'une « illustration conceptuelle inspirée de la narration et de la matière des vitraux médiévaux de Chartres ». Elle ne devra pas la présenter comme un vitrail historique retrouvé ni comme l'œuvre d'un atelier contemporain précis.

## Discipline culturelle appliquée

- Le prompt reprend des éléments documentés et généraux : baie en lancette, médaillons narratifs, verre coloré, plomb, palette cobalt-rubis-émeraude-ambre et tracé monumental.
- Il ne reproduit aucune verrière précise, aucun saint, aucun récit biblique et aucun blason.
- Il ne cite et n'imite aucun artiste vivant ni aucun atelier contemporain.
- La scène est volontairement séculière : une artisane, deux archives et deux gardiens. Les croix, halos, trônes et autels sont explicitement exclus.
- Le sceau entaillé est un dispositif pédagogique contemporain. Il ne prétend pas reproduire la signature historique d'un maître-verrier.
- Les personnes représentées ne sont pas désignées comme une population médiévale authentique. Le vitrail est ici un langage narratif, pas une reconstitution ethnographique.

## Pourquoi la composition est construite ainsi

### Une origine, deux lieux égaux

L'origine occupe un petit médaillon supérieur parce que l'attribution doit rester visible. Elle est toutefois plus petite que l'ensemble des deux baies inférieures et ne comporte ni halo ni trône. L'auteur est une origine vérifiable, pas un souverain du réseau.

Les deux lieux d'hébergement ont exactement les mêmes dimensions, la même lumière et le même encadrement. Aucun n'est central : leur symétrie exprime la redondance sans inventer un serveur principal.

### Une marque simple et défectueuse

Un disque parfait pourrait être recréé fortuitement ou lu comme une médaille. La petite encoche triangulaire à quatre heures agit comme un détail d'identité facile à comparer. Les trois coffres doivent conserver exactement cette anomalie.

### Des gardiens aux mains vides

Ajouter des pinceaux, matrices ou outils de scellement près des hébergeurs les transformerait visuellement en coauteurs. Leurs mains ouvertes et éloignées des coffres indiquent une garde matérielle, pas une nouvelle fabrication.

### Aucun flux dessiné

Des flèches auraient expliqué le réseau à la place de la scène. Un tuyau ou une corde reliant les trois baies aurait aussi pu faire du transporteur une nouvelle autorité centrale. La duplication identique et l'égalité des lieux portent seules l'idée.

## Risques de génération à surveiller

1. **Les coffres dérivent.** Rejeter toute génération où la couleur, la forme, la position du sceau ou l'orientation de l'encoche diffèrent entre les trois exemplaires.
2. **Un hébergeur ajoute sa propre marque.** Rejeter un second sceau, une initiale, un blason, un tampon, une signature ou une retouche sur les copies.
3. **La symétrie crée un gagnant central.** Aucun coffre inférieur ne doit être plus grand, plus haut, plus lumineux ou entouré d'un décor plus riche.
4. **La source devient une divinité.** Rejeter halo, nimbe, couronne, trône, geste de bénédiction ou personnages inférieurs agenouillés vers l'artisane.
5. **Les cabinets deviennent des autels ou reliquaires.** Ils doivent rester de simples vitrines d'archive, sans cierges, anges, ostensoirs ni architecture sacrée.
6. **Les gardiens deviennent fabricants.** Rejeter pinceaux, marteaux, matrices, pigments, palettes et mains posées sur le sceau.
7. **Le vitrail devient une copie historique.** Rejeter toute reproduction reconnaissable d'une scène religieuse chartraine ; la composition doit rester conceptuelle et séculière.
8. **Ideogram invente du texte.** Rejeter lettres, chiffres, faux latin, cartels, noms, monogrammes et pseudomots.
9. **La copie est lue comme un faux.** Si les deux coffres inférieurs paraissent clandestins ou dégradés, simplifier le décor et renforcer leur égalité matérielle avec l'original.

## Test aveugle

Présenter l'image seule, sans titre, sans phrase du livre et sans mention de Nexus à cinq personnes.

Poser seulement :

1. « Voyez-vous trois objets différents ou le même objet présent dans plusieurs lieux ? Quel détail vous le fait penser ? »
2. « Qui paraît avoir créé ou marqué l'objet ? »
3. « Que font les deux personnes du bas : fabriquent-elles, jugent-elles ou conservent-elles ? »
4. « L'un des deux lieux du bas paraît-il plus officiel ou plus propriétaire que l'autre ? »
5. « Si un des deux lieux du bas fermait, que resterait-il ? »
6. « Résumez l'image en une phrase. »

### Seuil de réussite

- au moins quatre personnes sur cinq identifient le même artefact conservé dans deux lieux égaux ;
- au moins quatre attribuent la création ou la marque à l'artisane supérieure ;
- aucune n'attribue spontanément la paternité à l'un des deux gardiens ;
- au moins quatre comprennent qu'un exemplaire resterait disponible si l'un des lieux cessait de le conserver ;
- aucune ne décrit un lieu inférieur comme le serveur principal, le propriétaire officiel ou l'autorité de publication ;
- au moins trois formulent spontanément une idée proche de « conserver une copie n'est pas l'avoir créée ».

Si les trois coffres ne sont pas reconnus comme identiques, réduire encore leur décor à la silhouette cobalt et au disque entaillé. Si les gardiens sont lus comme des auteurs, les éloigner davantage des vitrines et placer leurs mains derrière le dos. Si la scène devient religieuse, remplacer la lancette par un panneau rectangulaire de musée tout en conservant la matière verre-plomb et les trois registres.
