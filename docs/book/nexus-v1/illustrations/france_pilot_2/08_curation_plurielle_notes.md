# 08 — Curation plurielle

## Phrase courte pour le livre

> Plusieurs curateurs publient leurs propres listes signées ; une recommandation reste une perspective attribuée, jamais le classement mondial du réseau.

## Fonctionnalité Nexus représentée

Le lecteur voit un même ensemble de six objets dans la vitrine. Les trois plats reprennent exactement ces six objets dans le même ordre, mais chacun en colore trois selon une sélection différente. Les plats ont la même taille, la même hauteur, le même encadrement et la même quantité de couleur. Aucun n'est présenté comme le résultat officiel.

Chaque plat possède en revanche un sceau distinct. Le sceau signifie seulement : « cette sélection provient de cette source ». Il ne certifie ni la vérité, ni la qualité universelle des objets sélectionnés.

L'illustration sépare ainsi quatre notions :

1. le corpus commun, placé dans la vitrine ;
2. trois sélections autonomes, représentées par les plats ;
3. l'attribution de chaque sélection, représentée par les sceaux ;
4. l'absence de palmarès global, rendue par une stricte égalité de taille, de hauteur et de traitement.

## Preuves dans le dépôt live

État inspecté : `HEAD 9134af64`, le 27 juillet 2026.

### Contrat du livre

- `docs/book/nexus-v1/MANUSCRIT.md:247-259` distingue la curation de la censure : une personne peut suivre plusieurs curateurs, comparer leurs choix ou cesser de les suivre ; leurs sélections peuvent diverger sans devenir une vérité officielle.
- `docs/book/nexus-v1/MANUSCRIT.md:852-866` décrit une visibilité locale sans podium universel. Les signaux restent attribués et le réseau ne fabrique pas une opinion officielle unique.
- `docs/book/nexus-v1/MANUSCRIT.md:951-958` sépare la pertinence, la preuve et les curateurs qui recommandent ou déconseillent un objet.

### Contrat et implémentation Rust

- `crates/nexus-core-rs/src/curator.rs:1-29` décrit des listes signées diffusées sans registre central.
- `crates/nexus-core-rs/src/curator.rs:101-131` définit chaque `CuratorList` avec sa clé publique, son nom d'affichage, sa révision et ses propres entrées.
- `crates/nexus-core-rs/src/curator.rs:177-274` lie cryptographiquement une liste à la clé de son curateur et vérifie l'attribution Ed25519.
- `crates/nexus-shell-daemon/src/curators_api.rs:53-90` expose séparément les listes vérifiées mises en cache et l'ensemble d'attention choisi localement.
- `crates/nexus-shell-daemon-core/src/browse.rs:169-177` affirme qu'un même projet recommandé par plusieurs curateurs produit une entrée par paire `(project_id, curator_pubkey)`.
- `crates/nexus-shell-daemon-core/src/browse.rs:632-686` aplatit les listes en conservant le nom et la clé du curateur sur chaque résultat, puis `:808-812` applique uniquement un ordre stable par identifiants, pas un classement d'opinion.

### Historique Git pertinent

- `f4ae22db` — 11 avril 2026 : primitives de listes de curateurs et signatures Ed25519.
- `818429d8` — 11 avril 2026 : souscription gossip et récupération des listes dans le daemon.
- `6f328933` — 11 avril 2026 : surfaces Browse et Curators dans le shell.
- `4f52beaa` — 10 juin 2026 : Browse devient centré sur les sources choisies par le nœud.
- `f7d42bce` — 16 juillet 2026 : extraction du domaine curateurs vers `curators_api.rs` sans changement wire.

## Limite honnête de l'état actuel

L'image représente correctement le contrat du livre et la structure du backend, mais pas encore une vue comparative déjà aboutie dans l'interface.

Le daemon conserve bien une ligne par projet et par curateur. Cependant `web/src/pages/Browse.tsx:216-268` déduplique l'affichage par `(project_id, archive_hash)`. Pour les anciennes entrées de curation sans `archive_hash`, plusieurs recommandations du même projet peuvent donc être réduites à une seule carte. De plus, `web/src/pages/Browse.tsx:581-678` n'affiche pas actuellement le nom du curateur sur la carte standard.

La scène montre donc la propriété visée et les données déjà attribuées, mais elle ne doit pas être citée comme preuve que l'interface actuelle sait déjà juxtaposer trois perspectives.

## Tradition visuelle choisie

### Faïence de Nevers : une tradition locale née de circulations

Le choix ne consiste pas à inventer un mélange « italien + oriental + français » avec l'IA. Il s'appuie sur une tradition nivernaise constituée historiquement par plusieurs circulations :

- la [Ville de Nevers](https://www.nevers.fr/decouvrir-nevers/la-faience/histoire-de-la-faience) situe le développement de la faïence à la fin du XVIe siècle avec les frères Conrade venus de Mantoue ; elle documente la palette de grand feu — bleu, vert, jaune, violet — et l'évolution vers des inspirations orientales ;
- le [Musée de la Faïence et des Beaux-Arts de Nevers](https://culture.nevers.fr/musee-de-la-faience-et-des-beaux-arts/) présente une production arrivée avec des artisans italiens, puis nourrie d'influences d'Orient et d'Occident ;
- la [Frick Collection](https://www.frick.org/exhibitions/faience/nevers_golden_age) documente les liens successifs avec la majolique italienne, la céramique persane, Delft et la porcelaine chinoise, puis la formation d'un caractère propre à Nevers ;
- ce [plat de Nevers du Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/194290), daté vers 1675 et placé dans le domaine public, fournit une référence visuelle accessible : émail blanc, bleu de cobalt, oiseaux et bouquets inspirés de céramiques turques.

La faïence de Nevers reste une tradition vivante. Elle appartient ici à la catégorie des traditions de rencontre, pas à celle d'une communauté disparue.

## Discipline culturelle appliquée

- Le prompt reprend un matériau, une palette, des formes de plats et une logique de décor attestés.
- Il ne copie aucun objet historique précis.
- Il n'imite aucun artiste vivant et ne cite aucun atelier contemporain.
- Il n'utilise ni scène religieuse, ni armoirie, ni personnage « oriental » générique.
- Les six motifs sont des objets séculiers inventés pour expliquer Nexus. Ils ne sont pas présentés comme des motifs authentiques du XVIIe siècle.
- Les trois sceaux sont une convention pédagogique contemporaine. Ils évoquent l'attribution sans prétendre reproduire les marques historiques des manufactures.

La légende éditoriale devra donc employer « illustration conceptuelle dans le langage matériel de la faïence historique de Nevers » et non « faïence ancienne authentique ».

## Pourquoi la composition est construite ainsi

### Trois plats, pas trois personnes

Des personnages auraient introduit des différences involontaires de prestige par l'âge, le genre, le vêtement, la posture ou la position sociale. Trois objets strictement égaux permettent de tester directement la pluralité des listes.

### Le même ordre sur chaque plat

Si les objets changeaient de position, le spectateur pourrait croire que chaque plat montre un corpus différent. Le même agencement deux-par-trois rend visible que seule la sélection change.

### Trois objets colorés sur chaque plat

Chaque perspective reçoit la même quantité de couleur. Le plat médian ne doit donc pas devenir automatiquement le « bon compromis » ou la recommandation de la plateforme.

### La vitrine commune

La vitrine réunit le corpus sans choisir à la place des curateurs. Les objets y sont de même hauteur et sans piédestal central. Elle est basse et secondaire pour ne pas devenir une autorité supérieure.

## Risques de génération à surveiller

1. **Le plat du milieu devient principal.** Ideogram peut l'agrandir, l'éclairer ou lui ajouter une bordure. Rejeter toute image où l'un des trois plats domine.
2. **Les six motifs dérivent entre les plats.** Si les formes ou leurs positions ne correspondent plus, l'idée de perspectives sur un même corpus disparaît.
3. **Les sceaux deviennent des médailles de qualité.** Ils doivent être petits, différents et équivalents, jamais dorés ni surmontés d'un ruban honorifique.
4. **La vitrine devient un podium.** Aucun objet ne doit être plus haut, placé au centre sur un socle, couronné ou éclairé séparément.
5. **Le style devient une céramique générique.** Vérifier l'émail blanc, le bleu cobalt, le vert de cuivre, le jaune d'antimoine et le violet de manganèse, avec un dessin de grand feu net.
6. **De faux textes apparaissent.** Rejeter lettres, chiffres, cartels de musée, signatures manuscrites et pseudomots.
7. **Le décor fusionne artificiellement des cultures.** Rejeter toute accumulation de monuments, costumes ou symboles nationaux. La rencontre est portée par l'histoire de la technique nivernaise, pas par un collage de clichés.

## Test aveugle

Présenter l'image sans titre, sans phrase du livre et sans mention de Nexus à cinq personnes.

Poser seulement :

1. « Ces trois plats parlent-ils de trois ensembles différents ou du même ensemble ? Pourquoi ? »
2. « Qu'est-ce qui change entre eux ? »
3. « Est-ce qu'un plat paraît officiel, meilleur ou supérieur aux autres ? »
4. « À quoi peuvent servir les petits sceaux ? »
5. « Si vous deviez résumer l'image en une phrase, que diriez-vous ? »

### Seuil de réussite

- au moins quatre personnes sur cinq identifient le même corpus avec trois sélections différentes ;
- au moins quatre comprennent que chaque sceau attribue une sélection à une source distincte ;
- aucune ne décrit le plat central comme le classement officiel, le gagnant ou le choix du système ;
- au moins trois formulent spontanément une idée proche de « plusieurs recommandations possibles ».

Si les objets répétés ne sont pas reconnus, agrandir les plats et réduire les détails décoratifs. Si le sceau est lu comme une récompense, le remplacer par une petite marque d'atelier directement peinte sur le bord inférieur.
