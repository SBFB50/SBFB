# 05 — Héberger n’est pas publier

## Résultat de la nouvelle méthode

La scène ne part plus d’un schéma Nexus recouvert d’un style ancien. Elle part d’une pratique culturelle possédant déjà la logique recherchée :

> Dans l’imprimerie parisienne de la Renaissance, une même forme produit plusieurs exemplaires d’une édition. Ces exemplaires quittent l’atelier pour être vendus, lus, transportés ou conservés, mais leur marque typographique continue d’indiquer l’atelier d’origine.

Le prompt montre donc un atelier d’imprimeur-libraire, sa presse, des feuilles séchant, un colporteur, un lecteur et un porteur de bibliothèque. Il n’introduit ni coffre conceptuel, ni gardien symbolique, ni double dépôt symétrique, ni sceau inventé.

### Phrase courte pour le livre

> Plusieurs nœuds peuvent garder la même archive disponible : la copie change de lieu, pas d’origine.

### Phrase culturelle qui précède l’explication technique

> Plusieurs exemplaires d’une édition quittent la même presse et rejoignent des lecteurs ou des bibliothèques différents ; ils conservent pourtant la marque de l’atelier qui les a mis en circulation.

## 1. Invariant Nexus exact

L’image doit aider à distinguer quatre propositions :

1. un artefact publié peut être copié à l’identique sur plusieurs nœuds ;
2. chaque copie supplémentaire peut améliorer sa disponibilité ;
3. le nœud qui conserve et sert les octets affirme seulement sa capacité de les servir ;
4. la publication, la provenance et la signature d’origine ne lui sont pas transférées.

L’invariant n’est pas « plusieurs serveurs possèdent chacun leur propre publication ». C’est :

> plusieurs fournisseurs peuvent détenir le même contenu adressé par hash, tandis que la provenance de la publication reste celle de l’auteur.

Deux frontières doivent rester visibles :

- le hash atteste l’intégrité des octets demandés, pas la vérité de leur contenu ;
- la signature technique rattache une affirmation à une clé, pas automatiquement à une identité civile.

## 2. Preuves dans le manuscrit

État inspecté le 27 juillet 2026 : `HEAD 9134af64`. Le dossier `docs/book/nexus-v1/` est actuellement non suivi par Git. Les passages ci-dessous décrivent donc le manuscrit de travail présent sur disque, pas un contrat déjà intégré à l’historique principal.

- `docs/book/nexus-v1/09_reseau_apps_compute.md:31-43` distingue blobs immuables, annonces signées, documents partagés et relais. Le relais transporte sans devenir propriétaire logique ni autorité de publication.
- `docs/book/nexus-v1/09_reseau_apps_compute.md:45-49` énonce directement « Héberger n’est pas publier » : conserver et servir l’archive d’autrui améliore sa disponibilité sans transformer l’hébergeur en auteur.
- `docs/book/nexus-v1/09_reseau_apps_compute.md:47` sépare explicitement identité, signature et provenance de la simple possession des octets.
- `docs/book/nexus-v1/09_reseau_apps_compute.md:72-74` rappelle que la Proof Card expose ce qui est établi sans transformer la conservation ou la recommandation en autorité globale.
- `docs/book/nexus-v1/00_note_au_lecteur.md:34-45` refuse de transformer une signature en vérité ou une publication signée en obligation pour tous.

## 3. Preuves dans le code vivant

### 3.1 L’invariant est nommé dans le domaine

`crates/nexus-shell-daemon/src/seed_api.rs:6-12` regroupe l’hébergement volontaire, l’épinglage `keep_online`, les invitations de seed et le pilote headless, puis inscrit explicitement :

`heberger != publier, seeder != auteur`

Ce n’est donc pas seulement une interprétation éditoriale.

### 3.2 Le seeder récupère les octets exacts sans refaire la provenance

`crates/nexus-shell-daemon/src/seed_api.rs:344-358` décrit le chemin de seed volontaire :

- le nœud récupère le blob public distant ;
- il l’épingle sous un tag local `keep_online` ;
- il persiste son intention de le conserver ;
- il ne resigne jamais la provenance de l’application ;
- la récupération peut essayer l’ancre de répertoire puis plusieurs seeders.

Le commentaire relie l’intégrité à BLAKE3 : le nœud de soutien ne peut conserver que les octets correspondant à l’adresse de contenu demandée.

### 3.3 L’affirmation de disponibilité reste séparée de l’auteur

`crates/nexus-coordinator-rs/src/public_feed.rs:87-105` définit `SeedAnnouncedPayload` avec :

- le projet ;
- l’identité technique du seeder ;
- le hash BLAKE3 de l’archive détenue.

Le seeder signe sa propre affirmation de conservation. Cette signature ne réattribue jamais l’auteur de l’application.

`crates/nexus-shell-daemon/src/feed_sync.rs:113-135` matérialise la même séparation lors de l’émission : la clé du seeder signe l’annonce de seed, jamais la provenance de l’application.

### 3.4 L’ancre joignable n’est pas l’autorité de provenance

`crates/nexus-shell-daemon-core/src/browse.rs:117-125` définit la source `NodeDirectory`. Le `node_id` de l’ancre sert à joindre un fournisseur, mais l’autorité de provenance reste la signature de l’auteur.

`crates/nexus-shell-daemon-core/src/browse.rs:737-760` précise encore que :

- `archive_hash` est l’adresse BLAKE3 provenant de l’auteur ;
- l’ancre et les seeders sont des fournisseurs, pas l’autorité ;
- la récupération essaie plusieurs fournisseurs ;
- BLAKE3 constitue le contrôle d’intégrité.

### 3.5 Le test exerce les octets réellement conservés

`crates/nexus-shell-daemon/src/seed_api.rs:1128-1158` vérifie qu’un nœud :

- récupère le blob depuis un fournisseur ;
- possède ensuite exactement le payload attendu ;
- conserve le tag d’épinglage ;
- persiste le couple `keep_online` et hash d’archive.

L’assertion compare les octets reçus à ceux de l’auteur. Elle ne crée aucune nouvelle provenance.

## 4. Historique Git pertinent

L’historique montre une construction progressive de l’invariant :

- `f5c25757` — récupération P2P des archives découvertes pour qu’elles puissent réellement être servies ;
- `4c1acc57` — épinglage local persistant et politique `keep_online` ;
- `b76a0848` — protocole authentifié de demande d’hébergement inter-nœuds ;
- `66a9409b` — annonce distincte des seeders et compteur de disponibilité ;
- `821aa8c2` — répertoires de nœuds durables avec séparation explicite entre l’ancre joignable et l’auteur ;
- `00104501` — récupération multi-fournisseurs par adresse de contenu ;
- `542254b1` — extraction du domaine seed vers `seed_api.rs` sans changement de contrat réseau ;
- `32abfabf` — poursuite du découpage du daemon avec les tests de ce domaine co-migrés.

Le chemin live est donc plus avancé qu’une simple vision du livre. Cela ne signifie toutefois pas que toute disponibilité annoncée est garantie.

## 5. Limites honnêtes de l’implémentation

### 5.1 Une annonce n’est pas une preuve de présence permanente

`crates/nexus-coordinator-rs/src/public_feed.rs:87-99` indique qu’un compteur de seeders est *best effort*. Un nœud peut annoncer qu’il détient un blob puis devenir indisponible. Seule une récupération réussie établit qu’il sert les octets au moment considéré.

### 5.2 Le compteur est un indice de fraîcheur

`crates/nexus-shell-daemon/src/seed_api.rs:639-663` décrit `peer_count` comme un nombre de seeders récemment vus. L’adresse BLAKE3 reste la vérité du contenu recherché ; le nombre affiché ne constitue pas une garantie de téléchargement.

### 5.3 L’intégrité ne prouve pas l’auteur civil ni la qualité

Recevoir les octets correspondant au hash prouve que le contenu n’a pas changé par rapport à cette adresse. Cela ne prouve pas :

- que le contenu est vrai ;
- que l’auteur technique est la personne civile qu’il prétend être ;
- que la licence déclarée est valide ;
- que l’application est sûre ;
- qu’un seeder restera en ligne.

### 5.4 L’image représente uniquement le cas sans modification

Un nœud qui modifie l’archive ne possède plus le même objet adressé par contenu. Un fork, un nouveau build ou une nouvelle publication doit produire un autre hash et une nouvelle provenance. Le prompt ne représente que la réplication à l’identique.

## 6. Recherche culturelle avant l’analogie

### 6.1 Candidats étudiés

#### Vitraux de Chartres déposés pendant les guerres — rejeté

Le déplacement et le stockage des panneaux montrent très bien qu’un dépôt ne devient pas l’auteur d’une œuvre. Ils montrent mal l’augmentation de disponibilité :

- un panneau matériel ne peut être présent simultanément dans plusieurs lieux ;
- son retrait protège l’original mais le rend temporairement moins accessible ;
- une caisse de guerre évoque la sauvegarde, pas la réplication.

Cette pratique reste pertinente pour « conserver pendant une interruption », mais pas pour le présent invariant.

#### Moulages de sculptures — intéressant mais secondaire

L’atelier de moulage du GrandPalaisRmn, actif depuis deux siècles, produit des reproductions à partir d’une collection de moules. Cette pratique rend une forme accessible dans plusieurs lieux et conserve l’attribution de l’original.

Elle ajoute néanmoins deux rôles créatifs difficiles à séparer visuellement :

- l’auteur de la sculpture originale ;
- l’atelier qui fabrique matériellement le moulage.

Le spectateur pourrait légitimement considérer le mouleur comme coauteur de la reproduction. Cette ambiguïté est plus grande que dans le cas d’octets copiés sans transformation.

Source : [Ateliers d’art du GrandPalaisRmn](https://ateliersartmuseesnationaux.fr/).

#### Chalcographie et tirages d’une plaque — candidat très fort, non retenu

La Chalcographie du Louvre conserve des milliers de cuivres et en imprime des tirages. Une matrice produit plusieurs épreuves qui conservent l’origine de la gravure.

Mais l’impression d’une plaque est un acte de fabrication : l’encrage, la pression, le papier et l’état de la plaque peuvent différencier les épreuves. Pour Nexus, les seeders ne produisent pas de nouvelle interprétation du blob. L’imprimerie typographique d’une édition reste plus immédiatement compréhensible pour un public général.

Source : [Ateliers d’art du GrandPalaisRmn — chalcographie](https://ateliersartmuseesnationaux.fr/).

#### Partition et interprétation musicale — rejeté

Plusieurs lieux peuvent jouer la même œuvre sans devenir compositeurs. Cependant chaque interprétation est volontairement différente. La métaphore convient à un workflow réinterprété, pas à un blob identique.

#### Dépôt légal et exemplaires imprimés — retenu

Le livre imprimé fournit simultanément :

- une édition issue d’un atelier identifiable ;
- plusieurs exemplaires issus de la même forme ;
- une circulation vers des lecteurs, libraires et bibliothèques ;
- une conservation dans des institutions qui ne deviennent pas titulaires du contenu ;
- une marque typographique répétée sur les exemplaires.

La BnF précise aujourd’hui que le dépôt légal sert la collecte, la conservation, le signalement et la consultation, et qu’il ne confère aucun droit de propriété sur le titre ni sur le contenu déposé.

Source : [BnF — Qu’est-ce que le dépôt légal ?](https://depotlegal.bnf.fr/afficherPageStatique.do?aliasPage=quest-ce-que-le-depot-legal).

La pratique contemporaine distingue également deux obligations lorsqu’une même structure est éditeur et imprimeur : un exemplaire rejoint la BnF comme dépôt éditeur, un autre la bibliothèque habilitée de la région comme dépôt imprimeur.

Source : [BnF — Dépôt légal imprimeur, mode d’emploi](https://www.bnf.fr/fr/centre-d-aide/depot-legal-imprimeur-mode-demploi).

### 6.2 Audit du doublon avec l’illustration RRV lyonnaise

Le second pilote comportait déjà une image consacrée à RRV dans un atelier d’imprimerie lyonnais. Le risque de répétition est donc réel :

- une grande presse verticale peut produire une silhouette presque identique ;
- papier, encre, Renaissance et vêtements d’atelier appartiennent au même voisinage visuel ;
- deux images successives pourraient être lues comme les épisodes d’un même récit plutôt que comme deux communautés distinctes ;
- Ideogram a déjà transformé la première proposition en peinture historique détaillée, malgré la référence à la gravure.

Trois alternatives ont été réévaluées spécifiquement pour éviter ce doublon.

#### Bronzes de Rodin

Le Musée Rodin diffuse dans le monde des éditions originales limitées à douze exemplaires, produites depuis les moules ou modèles légués par l’artiste. La distinction entre auteur, fondeur et lieu d’exposition est culturellement riche.

Cette piste n’est pourtant pas plus fidèle à l’invariant. Le musée est ayant droit et possède explicitement une mission d’édition : il n’est donc pas seulement un hébergeur. Le fondeur réalise matériellement chaque bronze et appose parfois sa propre marque. L’objet porte ainsi plusieurs responsabilités légitimes, contrairement au seeder qui recopie des octets sans refabriquer l’œuvre.

Sources :

- [Musée Rodin — Les éditions originales de bronze](https://www.musee-rodin.fr/musee/institution/editions-originales-de-bronze) ;
- [Musée Rodin — Ève, grand modèle](https://collections.musee-rodin.fr/en/document/eve-grand-modele/6385d6c805c32181005d0295), notice distinguant Rodin comme auteur et Alexis Rudier comme fondeur.

#### Fac-similés de Lascaux

Lascaux offre la plus forte alternative visuelle. L’original est fermé depuis 1963 ; Lascaux II, l’exposition itinérante Lascaux III et Lascaux IV donnent accès à ses peintures dans plusieurs contextes. Les panneaux de Lascaux III sont mobiles et démontables, et l’Atelier des Fac-Similés du Périgord a reproduit les parois avec relevés 3D, modelage et pigments.

Cette histoire montre remarquablement qu’un lieu d’exposition ne devient pas l’origine des figures. Elle perd toutefois deux éléments de l’invariant demandé :

- les personnes paléolithiques à l’origine des peintures ne sont pas identifiées par une signature comparable à une clé de publication ;
- les fac-similés sont des reconstructions artisanales et interprétées, pas des copies octet pour octet sous la même adresse.

Lascaux conviendrait mieux à une fonctionnalité sur la conservation, l’accès à un original indisponible ou la transmission d’une trace dont l’auteur civil est inconnu.

Sources :

- [Lascaux — histoire des fac-similés](https://lascaux.fr/fr/histoire-de-lascaux/) ;
- [Lascaux — chantier de reproduction intégrale](https://lascaux.fr/fr/lascauxiv-construction-reproduction-grotte/).

#### Dépôt d’une œuvre unique

Un dépôt du Cnap ou d’un musée sépare clairement auteur, propriétaire et lieu dépositaire. Il ne crée cependant pas plusieurs exemplaires disponibles en même temps : l’œuvre matérielle change seulement de lieu. Cette piste illustre « garder n’est pas posséder », pas la redondance de fournisseurs.

### 6.3 Décision après audit

La pratique de l’édition est conservée malgré le voisinage avec le pilote RRV, car elle est la seule candidate examinée qui réunisse naturellement :

- une matrice identifiable ;
- plusieurs exemplaires d’une même édition ;
- une circulation vers plusieurs détenteurs ;
- une marque d’origine répétée ;
- une conservation qui ne transfère pas le contenu intellectuel.

Le doublon de médium est préférable à une métaphore culturellement spectaculaire mais techniquement fausse. La série devra toutefois éviter de placer les deux images côte à côte.

La différenciation visuelle est obligatoire :

- RRV interroge et compare des sources ; son image est centrée sur l’examen documentaire ;
- cette image est centrée sur la sortie de l’atelier et la dispersion des exemplaires ;
- le rendu demandé ici est un bois gravé français à aplats limités, pas une peinture historique ;
- la répétition de l’olivier et les trois destinations dominent la narration davantage que l’architecture de la presse.

Si la refonte native de RRV adopte finalement une autre pratique, ce risque disparaîtra. Dans le cas contraire, l’édition finale du livre devra soit espacer ces images, soit réserver l’imprimerie à la fonctionnalité dont le test aveugle est le plus concluant.

## 7. Communauté et histoire visuelle retenues

### Les imprimeurs-libraires parisiens de la Renaissance

Cette communauté ne se réduit pas à un style posé sur l’image. Son activité fournit tous les éléments de la scène :

- presse à vis ;
- forme encrée ;
- feuilles humides ;
- séchage ;
- exemplaires reliés ou en cahiers ;
- imprimeur-libraire ;
- colporteur ou porteur de livres ;
- lecteur universitaire ;
- bibliothèque ;
- marque typographique d’atelier.

Robert Estienne offre un cas documenté particulièrement lisible :

- il exerce comme imprimeur-libraire à Paris de 1526 à 1551 ;
- son atelier se trouve rue Saint-Jean-de-Beauvais, en face des écoles ;
- sa marque est l’olivier ;
- il devient imprimeur du roi pour le latin et l’hébreu en 1539 puis pour le grec ;
- ses éditions sont liées à une famille de caractères et à un programme éditorial identifiable.

Source institutionnelle : [BnF — Robert I Estienne](https://reliures.bnf.fr/ark:/12148/cdt9xnx1).

Autre source de contexte : [BnF Essentiels — Le marché européen du livre au XVIe siècle](https://essentiels.bnf.fr/fr/livres-et-ecritures/histoire-du-livre-occidental/efbeed25-e194-437d-ac46-ee9254f2d014-livre-lepoque-moderne-16e-18e-siecles/article/a6d17561-a559-42af-ab61-eb3187346bc8-marche-europeen-livre-16e-siecle).

### Références visuelles

- [Wikimedia Commons — marque de Robert Estienne](https://commons.wikimedia.org/wiki/File:Estienne_2.jpg), reproduction du domaine public provenant de la bibliothèque numérique BEIC ;
- [Wikimedia Commons — catégorie Robert Estienne](https://commons.wikimedia.org/wiki/Category:Robert_Estienne), comprenant pages de titre, marque à l’olivier, caractères et représentations d’atelier ;
- [BnF Essentiels — Le roi et les arts](https://essentiels.bnf.fr/fr/histoire/temps-modernes/ccf48dc0-ff3c-4432-9b57-7e838f7a6721-francois-ier-entre-pouvoir-et-image/album/c1674bc9-b94f-4fc4-98a7-771ec5077ffb-roi-et-arts), contenant une image d’atelier d’imprimerie et une page de titre avec marque typographique ;
- [BnF — Robert I Estienne](https://reliures.bnf.fr/ark:/12148/cdt9xnx1), pour l’adresse, le métier, la chronologie et la marque à l’olivier ;
- [BnF — Le Nouveau Testament grec de 1550](https://www.bnf.fr/fr/agenda/limprimeur-et-le-roi-le-nouveau-testament-grec-1550), pour un exemple d’édition attribuée à l’imprimeur et à ses caractères.

## 8. Précaution historique essentielle

Le prompt ne doit pas être présenté comme une reconstitution littérale du dépôt légal de 1537.

La page historique de la BnF nuance le récit traditionnel : l’ordonnance de Montpellier imposait d’abord de présenter les livres avant leur mise en vente, dans un contexte comprenant aussi le contrôle des idées ; le dispositif fut peu appliqué et ne correspond pas exactement au double dépôt contemporain.

Source : [BnF Essentiels — L’ordonnance de Montpellier](https://essentiels.bnf.fr/fr/histoire/temps-modernes/ccf48dc0-ff3c-4432-9b57-7e838f7a6721-francois-ier-entre-pouvoir-et-image/article/cdbda9aa-961a-4f62-97e3-af045f10b571-ordonnance-montpellier).

La scène représente donc une **circulation plausible d’exemplaires dans le Paris des imprimeurs-libraires**, pas la procédure juridique moderne projetée en 1540. Le dépôt légal actuel sert seulement de preuve supplémentaire que conserver un exemplaire n’accorde pas la propriété du titre ou du contenu.

## 9. Correspondance avec Nexus, maintenue hors de l’image

| Pratique du livre | Fonction Nexus | Limite de l’analogie |
|---|---|---|
| Une forme imprime plusieurs exemplaires | Un blob adressé par contenu est copié sur plusieurs nœuds | Des impressions physiques peuvent varier légèrement ; les octets sous un même hash doivent être identiques |
| La marque à l’olivier identifie l’atelier | Provenance et signature rattachent la publication à une clé | Une marque imprimée peut être copiée ; une signature cryptographique est vérifiable |
| Colporteur, lecteur et bibliothèque détiennent des exemplaires | Plusieurs seeders détiennent et servent le blob | Le lecteur ne sert pas nécessairement l’exemplaire à d’autres |
| La bibliothèque conserve sans acquérir l’œuvre intellectuelle | L’hébergeur ne devient pas auteur | Le droit de propriété intellectuelle et le protocole cryptographique sont deux régimes différents |
| Davantage d’exemplaires réduisent le risque de disparition | Davantage de seeders peuvent améliorer la disponibilité | Une copie hors ligne ou perdue n’améliore pas la disponibilité réelle |

Le prompt ne tente pas de dessiner cette table. Il montre uniquement la pratique du livre. La correspondance appartient au texte du chapitre.

## 10. Pourquoi la composition est culturellement native

### Une scène unique plutôt que trois compartiments

L’atelier ouvre directement sur la rue. Presse, séchage, circulation, lecture et remise à une bibliothèque appartiennent au même environnement du livre. Les personnages ne sont pas répartis dans des cases égales et ne jouent pas les rôles abstraits d’un diagramme.

### La provenance vient d’une vraie marque d’imprimeur

L’olivier n’est pas un sceau inventé pour Nexus. Il dérive de la marque documentée de Robert Estienne. Le prompt retire la devise et tout texte afin d’éviter les pseudomots d’Ideogram ; il conserve seulement la forme végétale répétée.

### Les destinataires accomplissent de vrais gestes

- l’imprimeur actionne la presse ;
- les feuilles sèchent ;
- le colporteur emporte des volumes ;
- le lecteur consulte un exemplaire ;
- le porteur apporte des cahiers à une bibliothèque.

Personne ne garde les mains vides pour symboliser artificiellement l’absence d’autorité.

### L’origine reste un atelier, pas un souverain

La presse occupe une place importante parce qu’elle est matériellement la source des exemplaires. Aucun trône, couronne, juge ou gardien ne transforme l’origine en pouvoir politique sur les lecteurs.

## 11. Discipline culturelle

- Le prompt reprend des objets attestés de la culture matérielle du livre imprimé.
- La scène est qualifiée d’illustration inspirée, pas de document historique retrouvé.
- Aucun portrait précis de Robert Estienne n’est prétendu.
- La marque à l’olivier est simplifiée en symbole sans devise pour respecter l’interdiction de texte.
- Les rôles sociaux sont plausibles, mais leurs vêtements ne prétendent pas reconstituer un atelier particulier à un jour déterminé.
- La variété des carnations ne doit pas être présentée comme une statistique démographique du Paris de 1540 ; elle évite simplement de faire d’une pratique européenne un monde artificiellement uniforme.
- Le récit n’idéalise pas l’ordonnance de 1537, dont la dimension de contrôle est explicitement mentionnée dans cette note.
- Le livre devra créditer les sources de la BnF et distinguer le contenu historique de l’analogie technique.

## 12. Risques de génération

1. **Ideogram transforme l’image en peinture historique.**  
   Rejeter les rendus à modelé académique réaliste. Conserver le contour gravé, les aplats limités et la composition de livre illustré.

2. **Les oliviers diffèrent entre exemplaires.**  
   Rejeter une génération où la forme générale, les racines ou la bordure ovale varient fortement. L’idée dépend d’une origine visuelle commune.

3. **Un destinataire ajoute une seconde marque.**  
   Rejeter tampons, blasons, sceaux, signatures, étiquettes ou cachets supplémentaires.

4. **La bibliothèque devient le centre d’autorité.**  
   Rejeter palais monumental, trône, garde armé, fonctionnaire dominant ou lumière privilégiant l’entrée de bibliothèque.

5. **Le colporteur devient un faussaire.**  
   Il transporte des volumes achevés ; il ne doit disposer ni d’une presse miniature, ni d’une matrice, ni d’un tampon.

6. **Les copies paraissent être des livres différents.**  
   Les formats, reliures visibles et marques doivent rester cohérents. Les exemplaires peuvent se trouver à des étapes matérielles distinctes — feuille, cahier, volume relié — mais leur page d’origine doit rester reconnaissable.

7. **La presse imprime directement une couverture moderne.**  
   Rejeter jaquette, dos collé contemporain, reliure industrielle ou machine motorisée.

8. **Faux texte.**  
   Rejeter toute lettre, tout chiffre, devise latine, enseigne, légende, cartel, pagination ou pseudomot.

9. **La scène devient une publicité pour une marque.**  
   L’olivier doit rester une marque typographique historique intégrée aux pages, pas un logo géant flottant dans l’espace.

10. **Trop de causalité graphique.**  
    Rejeter flèches, lignes de connexion, rubans reliant les personnages, compartiments numérotés et pictogrammes réseau.

## 13. Test aveugle

Présenter l’image sans titre, sans phrase Nexus et sans cette note à cinq personnes.

### Questions

1. « Que se passe-t-il dans cette scène ? »
2. « Les feuilles et livres visibles semblent-ils provenir du même atelier ou de plusieurs ateliers ? Pourquoi ? »
3. « Que font les personnes éloignées de la presse ? »
4. « Qui semble avoir imprimé les exemplaires ? »
5. « Le lecteur, le porteur ou la bibliothèque vous semblent-ils avoir créé une nouvelle édition ? »
6. « Si un exemplaire restait dans la bibliothèque après la fermeture de l’atelier, que resterait-il identifiable ? »
7. « Résumez l’image en une phrase sans utiliser les mots ordinateur, réseau ou Nexus. »

### Seuil de réussite

- quatre personnes sur cinq décrivent l’impression puis la circulation de plusieurs exemplaires ;
- quatre reconnaissent la répétition de l’olivier comme lien d’origine ;
- aucune ne considère spontanément le porteur ou le lecteur comme imprimeur ;
- au moins trois comprennent qu’un exemplaire peut rester disponible loin de l’atelier ;
- au moins trois formulent une idée proche de « plusieurs copies, une même origine » ;
- aucune ne décrit une bibliothèque ou un personnage comme autorité centrale de l’ensemble.

### Échec instructif

Si les testeurs voient seulement « une journée dans une imprimerie », l’image reste culturellement juste mais l’olivier n’est pas assez comparable. Renforcer alors uniquement la répétition de la marque sur les feuilles, sans ajouter flèche, texte ou compartiment.

Si les testeurs pensent que chaque porteur possède une édition différente, réduire le nombre de formats matériels et montrer trois volumes reliés identiques.

Si les testeurs lisent la bibliothèque comme propriétaire de l’œuvre, diminuer son architecture et donner davantage de place au geste de circulation dans la rue.

## 14. Verdict

Cette proposition est plus fidèle à la nouvelle méthode que le vitrail précédent :

- la communauté fournit elle-même les objets et les gestes ;
- la scène reste plausible sans Nexus ;
- la provenance n’est pas représentée par un dispositif inventé ;
- la multiplicité vient naturellement du tirage d’une édition ;
- l’hébergement est expliqué ensuite par le texte, sans transformer l’image en diagramme.

La limite demeure explicite : une marque typographique n’offre pas la sécurité d’une signature cryptographique et des exemplaires physiques ne sont jamais aussi strictement identiques que des octets sous une même adresse BLAKE3. L’illustration transmet une intuition, pas une équivalence technique.
