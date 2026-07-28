# Pilote culturel mondial — vingt communautés, vingt histoires visuelles

Ce lot prolonge le pilote culturel natif en dehors du seul territoire français.
Il ne cherche ni une charte graphique universelle, ni dix « costumes » posés
sur le même diagramme. Chaque illustration part d'une pratique documentée,
puis choisit une fonctionnalité Nexus que cette pratique permet d'éclairer
sans devenir l'autorité symbolique du protocole.

Les prompts sont destinés à Ideogram 4 en `768:1376`. Ils sont entièrement
wordless : aucune légende, lettre, flèche, interface ou diagramme ne doit être
généré dans l'image. La phrase Nexus reste dans le livre.

## Règle non négociable — illustration, jamais photographie

La communauté ne doit jamais être représentée par une photographie, une
fausse photographie historique, une peinture photoréaliste ou un décor
cinématographique auquel quelques motifs culturels auraient été ajoutés.

Le langage graphique propre à la pratique construit **toute l'image** :

- les personnes deviennent des figures composées avec les aplats, contours et
  proportions du médium ;
- les objets suivent sa géométrie et sa manière d'organiser l'espace ;
- le support matériel devient le champ complet de la composition ;
- profondeur réaliste, lumière photographique, bokeh, modelé 3D, perspective
  cinématographique et texture de photographie sont interdits ;
- aucun motif culturel ne sert de simple bordure décorative autour d'une scène
  réaliste.

Ainsi, chaque prompt doit être reconnaissable comme une illustration produite
dans son médium propre : textile, estampe, écorce peinte, mosaïque, théâtre
d'ombres ou gravure. Ce sont des illustrations explicatives produites **dans**
vingt langages graphiques, pas des reportages sur vingt communautés.

## Les vingt scènes

| Prompt | Fonctionnalité Nexus | Phrase courte pour le livre | Communauté et pratique | Langage graphique |
|---|---|---|---|---|
| [`01_yoruba_capacites_bornees.json`](01_yoruba_capacites_bornees.json) | Capacités exprimées et bornées avant l'action | Une demande ne donne pas un pouvoir général : l'effet reste limité par ce qui a été déclaré, accordé et autorisé localement. | Femmes yorùbá productrices d'àdìrẹ ẹlẹ́kọ à Abeokuta et Ibadan | Indigo profond et réserves claires produites par la pâte de manioc, composition en champs géométriques |
| [`02_kinngait_double_consentement.json`](02_kinngait_double_consentement.json) | Consentement de la personne qui demande et du worker qui fournit le calcul | Une tâche ne part que si la personne accepte son effet et si le fournisseur accepte cette charge précise dans ses propres limites. | Artistes et maîtres imprimeurs inuit de Kinngait Studios | Estampe stonecut de Kinngait, grands aplats, contours noirs, papier crème et palette restreinte |
| [`03_tonga_inference_shardee.json`](03_tonga_inference_shardee.json) | Inférence shardée entre membres admis | Un plan ordonné répartit des sections contiguës entre plusieurs workers admis, puis les rassemble en un seul résultat. | Kokaʻanga, groupes de femmes tongiennes fabriquant collectivement le ngatu | Écorce crème, frottages de kupesi, pigments koka brun-rouge et détails tongo noirs |
| [`04_ajrakh_adresse_contenu.json`](04_ajrakh_adresse_contenu.json) | Blobs adressés par leur contenu | L'adresse désigne le contenu exact : modifier un seul détail produit un nouvel objet sans remplacer silencieusement l'ancien. | Imprimeurs khatri d'Ajrakhpur et de Dhamadka | Répétitions ajrakh en indigo, garance, noir et réserve crème, blocs de teck gravés |
| [`05_devia_parite_accessible.json`](05_devia_parite_accessible.json) | Même puissance fonctionnelle sous plusieurs formes accessibles | Canvas, structure, parcours guidé et conversation modifient le même modèle ; aucun mode n'est une version appauvrie. | Mouvement artistique sourd Deaf View/Image Art, ou De’VIA | Sérigraphie contemporaine à contraste fort, centralité des mains et du regard, rouge, cobalt, noir et crème |
| [`06_dongho_pow_session_gossip.json`](06_dongho_pow_session_gossip.json) | Preuve de travail Hashcash par session de gossip | Pour ouvrir une courte session sur un sujet, l'émetteur calcule une preuve fraîche liée à sa clé et à ce sujet ; sa vérification est rapide, mais multiplier les identités et les sujets impose de refaire le travail. | Communauté d'artisans de l'estampe de Đông Hồ, Bắc Ninh | Xylographie sur papier điệp, pigments naturels superposés en aplats et contours noirs |
| [`07_ebru_vram_ephemere.json`](07_ebru_vram_ephemere.json) | Nettoyage de la VRAM entre deux tâches | Quand la protection CUDA optionnelle est activée, une passe recouvre de zéros presque toute la VRAM alors libre après chaque tâche, afin que la suivante ne retrouve pas ses résidus. | Maîtres, apprentis et praticiens de l'ebru en Türkiye | Marbrure sur eau transférée au papier, palettes minérales et composition de livre |
| [`08_chitrakathi_quorum_deterministe.json`](08_chitrakathi_quorum_deterministe.json) | Quorum exact entre inférences déterministes | Une tâche vérifiable fixe l'échantillonnage ; plusieurs workers comparables calculent séparément, et seul un texte strictement majoritaire et identique peut être retenu. | Peintres-conteurs de la communauté Thakar de Pinguli | Cartes Chitrakathi en aplats minéraux, profils expressifs et contours noirs |
| [`09_isekatagami_filtre_avant_persistance.json`](09_isekatagami_filtre_avant_persistance.json) | Filtre de caractères invisibles avant persistance | Avant d'enregistrer un résultat ou d'attribuer des kudos, le nœud rejette toute sortie contenant des caractères invisibles ; aucun texte de cette sortie n'est conservé. | Horishi, dessinateurs et teinturiers de l'Ise-katagami, préfecture de Mie | Pochoirs de washi au tanin de kaki, komon minutieux et teinture indigo à réserve |
| [`10_khipu_kudos_non_monetaires.json`](10_khipu_kudos_non_monetaires.json) | Kudos non monétaires à rendement logarithmique et vieillissement | Un résultat accepté ajoute des kudos liés au worker ; doubler les tokens n'ajoute qu'un palier fixe, puis le poids visible des anciennes contributions décroît avec le temps. | Khipukamayuq historiques de l'empire inka et communautés andines gardiennes des khipus | Khipu matériel dans une illustration andine à l'encre et aux aplats de terres colorées |
| [`11_ukiyoe_contrats_agents_types.json`](11_ukiyoe_contrats_agents_types.json) | Contrats d'agents typés et passages de relais | Chaque spécialiste reçoit un artefact du type attendu et produit celui qu'attend l'étape suivante ; une liaison incompatible est refusée avant le run. | Réseau historique des dessinateurs, graveurs, imprimeurs et éditeurs d'ukiyo-e d'Edo | Estampe ukiyo-e en aplats, contours gravés, washi et composition diagonale d'atelier |
| [`12_huixcolotla_map_bornee.json`](12_huixcolotla_map_bornee.json) | `map` bornée utilisant un sous-workflow épinglé | Le même traitement s'applique à chaque élément de la collection, mais le run refuse d'étendre la collection au-delà de la limite inscrite dans le plan. | Artisanes et artisans du papel picado de San Salvador Huixcolotla, Puebla | Illustration entièrement découpée dans des couches de papier de soie coloré |
| [`13_ewe_parallel_join.json`](13_ewe_parallel_join.json) | Branches parallèles puis `join` déterministe | Les branches déclarées avancent séparément ; leur réunion n'utilise que les sources prévues et conserve la cohorte exacte qui a déclenché le résultat. | Tisserands ewe du Ghana et du Togo, notamment les communautés de la région Volta | Kente ewe en bandes étroites, blocs de trame supplémentaire et figures entièrement tissées |
| [`14_nuusavi_journal_reprise.json`](14_nuusavi_journal_reprise.json) | Journal durable et reprise sans rejouer les étapes terminées | Après une interruption, l'état est reconstruit depuis les événements conservés et l'exécution reprend à la première frontière sûre sans refaire les effets confirmés. | Peintres-scribes historiques ñuu savi et communautés descendantes de la Mixteca | Manuscrit pictural en accordéon sur peau préparée, aplats rouges, noirs, turquoise et ocre |
| [`15_mua_roi_nuoc_signal_durable.json`](15_mua_roi_nuoc_signal_durable.json) | Attente durable d'un signal externe typé | Le run peut rester garé sans boucler ; seul le signal attendu le réveille, et la même livraison ne peut pas provoquer deux reprises. | Guildes de marionnettes sur eau du delta du Fleuve Rouge, Viêt Nam | Illustration laquée spatiale : pavillon, bassin, musiciens et marionnettes de bois en action |
| [`16_subak_switch_ferme.json`](16_subak_switch_ferme.json) | `switch` déterministe à routes fermées et repli fixe | Une valeur choisit seulement une voie déjà construite ; si elle ne correspond à aucune branche, elle rejoint le repli prévu sans créer de destination. | Communautés agricoles des subak de Bali | Paysage hydraulique narratif en aplats minéraux, terrasses, canaux, vannages et eau |
| [`17_onggi_repeat_bornee.json`](17_onggi_repeat_bornee.json) | `repeat` bornée avec arrêt anticipé déterministe | Le corps épinglé recommence jusqu'au signal de fin ou jusqu'à sa limite structurelle ; il ne peut jamais prolonger lui-même la boucle. | Onggijang, artisanes et artisans coréens de l'onggi | Illustration à l'encre et aux terres colorées, jarre volumique, tour, outils réels et butée Nexus ajoutée |
| [`18_murano_classifieur_ferme.json`](18_murano_classifieur_ferme.json) | Classifieur à labels fermés et branche `default` | Le modèle propose parmi les destinations déclarées ; une réponse inconnue va au repli et ne fabrique jamais une quatrième catégorie. | Artisanes et artisans des perles et murrines de verre de Murano | Illustration entièrement construite en plages de verre coloré et contours sombres |
| [`19_jura_reessais_bornes.json`](19_jura_reessais_bornes.json) | Réessais bornés, temporisés, puis échec explicite | Un effet réessayable dispose d'un nombre fini de nouvelles tentatives ; après épuisement, il s'arrête ou emprunte la sortie d'erreur déclarée. | Communautés horlogères et de mécanique d'art de l'Arc jurassien franco-suisse | Gravure émaillée d'un automate à mécanisme ouvert, frein et berceau de réparation |
| [`20_samba_plan_compile_immuable.json`](20_samba_plan_compile_immuable.json) | `CompiledPlan.v1` immuable pendant le run | Une fois le plan compilé et lancé, ses modules, routes et capacités ne changent plus ; toute modification produit une autre version. | Communautés des écoles de samba et ateliers de barracão de Rio de Janeiro | Illustration spatiale de char allégorique en papier mâché, métal peint, costumes et avenue |

## 1 — Àdìrẹ ẹlẹ́kọ yorùbá

### Histoire retenue

L'àdìrẹ désigne plusieurs procédés de teinture à réserve développés et portés
principalement par des femmes yorùbá. Abeokuta devient un centre majeur au
XIXe siècle ; l'àdìrẹ ẹlẹ́kọ à pâte de manioc se développe au début du
XXe siècle et distingue notamment la dessinatrice de réserve et la teinturière.
La pâte protège matériellement certaines zones avant l'immersion : le bain
d'indigo ne décide pas après coup où il agit.

Cette contrainte physique éclaire les capacités Nexus sans transformer la
teinturière en juge : l'effet n'atteint que la surface laissée disponible avant
l'action.

Sources :

- [V&A — Àdìrẹ, tied and dyed indigo textiles](https://www.vam.ac.uk/articles/adire-tied-and-dyed-indigo-textiles/)
- [Smarthistory — Ibadandun woman’s wrapper](https://smarthistory.org/ibadandun-womans-wrapper-unrecorded-yoruba-artist/)
- [Princeton University Art Museum — Àdìrẹ ẹlẹ́kọ wrapper](https://artmuseum.princeton.edu/art/collections/objects/135166)

Rejeter l'image si la pâte est appliquée après la teinture, si tout le tissu
devient uniformément bleu, si une femme domine l'autre comme autorité générale,
ou si les motifs deviennent des lettres inventées.

## 2 — Kinngait Studios

### Histoire retenue

Le programme d'estampe de Kinngait commence à la fin des années 1950 et la
coopérative est fondée en 1959. Une caractéristique essentielle est la
collaboration entre la personne qui produit le dessin et le maître imprimeur
qui le transpose en stonecut, stencil, lithographie ou gravure. Le travail du
printmaker est une contribution créative et technique identifiable ; il ne
fusionne pas pour autant tous les rôles.

La scène ne prétend pas reproduire une cérémonie d'approbation historique.
Elle montre le moment ordinaire où un dessin précis est proposé et accepté
comme travail, tandis que les autres dessins ne deviennent pas une autorisation
illimitée.

Sources :

- [West Baffin Cooperative — histoire de la coopérative](https://www.westbaffin.com/history-wbec)
- [Inuit Art Foundation — évolution de l'estampe à Kinngait](https://www.inuitartfoundation.org/lite/inuit-art-quarterly/read/how-has-printmaking-evolved-at-kinngait-studios-in-the-last-sixty-years)
- [Canadian Museum of History — Cape Dorset prints in the 1960s](https://www.historymuseum.ca/capedorsetprints/history/1960s.php)

Rejeter l'image si le printmaker saisit tous les dessins, si l'artiste paraît
contrainte, si un sceau devient une médaille d'autorité, ou si le rendu copie
une œuvre reconnaissable d'un artiste de Kinngait.

## 3 — Kokaʻanga et ngatu tongien

### Histoire retenue

Le ngatu est un tissu d'écorce tongien produit par les femmes. Une partie du
travail peut commencer individuellement, mais la formation et la décoration de
grandes pièces mobilisent des groupes de travail, les kokaʻanga. Autour d'un
long papa, les femmes appliquent et frottent les motifs de plusieurs kupesi
placés sous un même tissu continu, puis renforcent certains détails à la
peinture.

Le prompt ne réduit pas le ngatu à une métaphore informatique : il montre
d'abord une production collective réelle. La correspondance Nexus vient du
travail contigu, ordonné et réuni en une seule surface.

Sources :

- [Te Papa — Decorating ngatu](https://collections.tepapa.govt.nz/topic/1114)
- [National Museum of Australia — barkcloth ngatu ʻuli](https://www.nma.gov.au/explore/features/cook_forster/objects/barkcloth_ngatu_uli_oz631)
- [University of Glasgow — Tongan barkcloth](https://tapa.gla.ac.uk/tonga/)

Rejeter l'image si les femmes produisent six tissus indépendants, si le ngatu
est découpé en boîtes de diagramme, si une participante devient cheffe du
réseau, ou si des armoiries et symboles nationaux sont inventés.

## 4 — Imprimeurs khatri et ajrakh

### Histoire retenue

L'ajrakh est une tradition de réserve, mordançage et impression au bloc
pratiquée dans le Sindh et le Kachchh. Des familles khatri se sont établies au
Kachchh il y a plusieurs siècles. Après le séisme de 2001 et les problèmes
d'eau à Dhamadka, des artisans ont fondé Ajrakhpur. La pratique reste vivante,
transmise et capable d'intégrer de nouveaux dessins.

Plusieurs blocs complémentaires construisent une répétition précise. Le prompt
représente une inspection de preuves naturelle dans l'atelier : un bloc modifié
produit un motif différent, tandis que les impressions antérieures restent
physiquement présentes.

Sources :

- [Khamir — Art of Ajrakh](https://exhibitions-khamir.org/ajrakh/art-of-ajrakh)
- [Khamir — Sufiyan Ismail Khatri](https://www.khamir.org/artisans/sufiyan-ismail-khatri-0)
- [Government of India — Kutch Ajrakh](https://handicrafts.nic.in/crafts/All_Crafts/Craft_Categories/Textile/Hand_Block_Printing/Kutch_Ajrakh/KutchAjrakhwebpage.html)
- [British Museum — carved Ajrakh printing block](https://www.britishmuseum.org/collection/object/A_2017-3063-3)

Rejeter l'image si les trois impressions deviennent identiques, si l'ancien
motif disparaît, si les blocs ressemblent à des sceaux de certification, ou si
la scène présente un motif religieux ou figuratif inventé comme « authentique ».

## 5 — De’VIA

### Histoire retenue

Le terme Deaf View/Image Art est formulé en 1989 par neuf artistes sourds avant
Deaf Way I. De’VIA ne désigne pas tout art produit par une personne sourde :
il désigne une intention de représenter l'expérience sourde. Le manifeste
mentionne notamment les contrastes forts, la focalisation centrale et
l'attention aux yeux, aux mains, au visage et aux textures.

La scène utilise ce langage pour parler directement de l'égalité d'accès. Elle
évite les oreilles brisées, appareils médicaux, gestes de « guérison » et toute
imitation d'un signe lexical précis. Les quatre représentations doivent offrir
une prise réelle sur une seule construction commune.

Sources :

- [Deaf Art — 1989 and the De’VIA Manifesto](https://deaf-art.org/history/1989/)
- [Deaf Arts Handbook — Deaf View/Image Art](https://deaf-art.org/wp-content/uploads/2021/01/DeafArtsHandbook_Volume1_FINAL2020_Dec3.pdf)

Rejeter l'image si un mode devient purement décoratif, si les quatre structures
ne correspondent pas, si les personnages sont médicalisés, si les mains sont
illisibles, ou si le style est présenté comme celui de toutes les personnes
sourdes.

## Audit technique du second lot

Les cinq premières images étaient déjà réparties entre capacités, consentement,
sharding, adressage de contenu et accessibilité. Le premier brouillon des images
6 à 10 répétait ensuite bridge, immutabilité, projection, provenance et
curation, soit des familles déjà traitées dans les pilotes français.

Le second lot a donc été reconstruit à partir des appels de production, des
recherches de sprint et de l'historique Git. Deux candidates ont été écartées :

- la sortie JSON contrainte est implémentée dans les backends LLM, mais le
  constructeur `build_generate_params` utilisé par le worker ne lui transmet
  actuellement aucun schéma ;
- la rotation de clés possède primitive, cache et restauration SQLite, mais
  `handle_rotation_message` n'a pas de caller dans la boucle gossip actuelle.

Cette discipline évite d'illustrer comme promesse opérationnelle une primitive
isolée ou un design de recherche ancien.

## 6 — Artisans de l'estampe de Đông Hồ

### Histoire retenue

L'estampe de Đông Hồ est une pratique villageoise de Bắc Ninh. Des blocs de
bois distincts déposent successivement les couleurs puis le contour noir sur un
papier điệp couvert de poudre de coquillage. Ses sujets comprennent la vie
quotidienne, l'histoire et les paysages. La pratique a été inscrite en 2025 sur
la Liste du patrimoine culturel immatériel nécessitant une sauvegarde urgente.

La scène n'utilise pas un sceau comme vérité ou autorité. Un bloc dense demande
un effort initial ; son même petit motif accompagne ensuite plusieurs images
différentes suspendues pendant une courte session.

Sources culturelles :

- [UNESCO — Craft of making Đông Hồ folk woodblock printings](https://ich.unesco.org/en/USL/craft-of-making-ong-h-folk-woodblock-printings-01737)
- [Vietnam National Authority of Tourism — ordre des blocs et pigments](https://seagames31.vietnamtourism.gov.vn/dong-ho-folk-painting-village/)

### Preuve Nexus et limite

`pow.rs` lie le défi Hashcash à la clé publique, au sujet et à un instant ;
`pow_gossip.rs` réutilise la preuve dans un cache de session de quinze minutes ;
la boucle de réception de `runtime.rs` rejette l'enveloppe avant de traiter son
payload. La preuve renchérit le spam multi-identité et multi-sujet, mais ne
signe pas le contenu : l'intégrité du payload reste l'affaire des signatures
des couches supérieures.

Rejeter l'image si chaque feuille exige un nouveau grand bloc, si le motif
devient certificat de vérité, si la feuille non marquée est détruite, si le
rendu devient photographie ou si de faux caractères vietnamiens apparaissent.

## 7 — Communauté des praticiens de l'ebru

### Histoire retenue

L'ebru crée des motifs en faisant flotter puis travailler des pigments sur une
préparation liquide avant de les transférer au papier. Le savoir se transmet
notamment par relation maître-apprenti et reste pratiqué sans barrière d'âge,
de genre ou d'origine selon le dossier UNESCO.

La scène exploite une propriété matérielle immédiatement lisible : l'œuvre
achevée est ailleurs, les restes du bain sont retirés, et la feuille suivante
reste vierge tant que la surface n'est pas redevenue propre. Elle n'affirme pas
que tous les ateliers d'ebru réalisent exactement ce nettoyage entre chaque
feuille.

Source culturelle :

- [UNESCO — Ebru, Turkish art of marbling](https://ich.unesco.org/en/RL/ebru-turkish-art-of-marbling-00644)

### Preuve Nexus et limite

Après chaque tâche, `engine/runtime.rs` fait avancer
`EphemeralLifecycle`, appelle `wipe_vram`, puis demande un arrêt gracieux quand
le nombre maximal de tâches est atteint. Le vrai nettoyage CUDA est
**optionnel** : avec la feature `gpu-ephemeral`, `ephemeral.rs` alloue à zéro
90 % de la VRAM alors libre ; sans cette feature, l'appel est explicitement un
no-op. L'image explique donc le mode renforcé, pas le comportement garanti de
tous les workers.

Rejeter l'image si le pigment de la première feuille passe sur la seconde, si
la feuille terminée est détruite, si le résidu rejoint le papier vierge, si
l'ebru n'est qu'une bordure autour d'un atelier réaliste ou si de la calligraphie
inventée apparaît.

## 8 — Peintres-conteurs Thakar de Pinguli

### Histoire retenue

Le Chitrakathi de Pinguli, dans le Maharashtra, est porté par la communauté
Thakar. Des images peintes sur papier accompagnent une narration orale ; les
figures de profil, les contours fermes et les expressions accentuées assurent
une lecture immédiate à distance.

Le prompt emploie une scène forestière séculière entièrement nouvelle. Il ne
met en image ni divinité ni épisode d'épopée et ne prétend pas que le quorum
serait une pratique Thakar : il emprunte au Chitrakathi sa comparaison directe
de cartes narratives.

Sources culturelles :

- [Development Commissioner Handicrafts, Inde — Chitrakathi painting](https://handicrafts.gov.in/crafts/All_Crafts/Craft_Categories/Miscellaneous/Folk_Painting/Chitrakathi_painting/ChitrakathiWebPage.html)
- [National Cultural Audiovisual Archives of India — Chitrakathi Painting](https://ncaa.gov.in/repository/search/displaySearchRecordPreview/CVI-BETA_1235-BC/Chitrakathi-Painting)

### Preuve Nexus et limite

Pour une `Task::verifiable`, `build_generate_params` impose température zéro
et une graine dérivée de l'identifiant de tâche. Le validateur conserve une
seule réponse par worker, compte les textes strictement identiques et n'accepte
qu'une majorité stricte. Ce quorum exact suppose des workers au runtime
homogène ; il ne prouve pas que la réponse majoritaire est vraie.

Rejeter l'image si les deux cartes retenues diffèrent d'un détail, si les
artistes se copient, si un personnage devient juge souverain, si la carte
divergente est punie ou si des figures religieuses sont inventées.

## 9 — Communauté artisanale de l'Ise-katagami

### Histoire retenue

L'Ise-katagami de la préfecture de Mie emploie plusieurs feuilles de washi
laminées au tanin de kaki. Des horishi y découpent des motifs extrêmement fins
qui servent ensuite à déposer une réserve régulière sur le tissu avant
teinture. Dessinateur, découpeur et teinturier restent des compétences
distinctes.

Le prompt fait du pochoir un instrument de comparaison, pas une décoration :
une étoffe irrégulière reste entière et dehors, tandis qu'une autre étoffe
propre entre dans le coffre. Il ne prétend pas reproduire un komon historique
identifiable.

Sources culturelles :

- [Gouvernement du Japon — The Ise-Katagami Patterns that Fascinated Europeans](https://www.gov-online.go.jp/eng/publicity/book/hlj/html/202212/202212_08_en.html)
- [Gouvernement du Japon — Cutting-Edge Kimono et division des métiers](https://www.gov-online.go.jp/eng/publicity/book/hlj/html/201701/201701_11_en.html)

### Preuve Nexus et limite

`output_filter.rs` détecte plusieurs familles de caractères invisibles.
`coordinator_api.rs` et `validator_loop.rs` exécutent ce garde-fou avant
`result_text`, puis n'attribuent aucun kudos en cas de tripwire. Les détecteurs
de fuite du prompt système existent aussi dans le module, mais le chemin HTTP
actuel leur transmet un prompt système vide ; cette image se limite donc
volontairement au garde réellement alimenté.

Rejeter l'image si le tissu irrégulier est partiellement archivé, si le filtre
répare silencieusement son contenu, si le coffre est placé avant le contrôle,
si le pochoir devient sceau officiel ou si l'ensemble ressemble à une photo de
kimono.

## 10 — Khipukamayuq historiques et gardiens andins des khipus

### Histoire retenue

Le khipu est un système andin de cordes et de nœuds employé notamment par
l'empire inka pour conserver et transmettre des informations. Les spécialistes
chargés de ces registres sont généralement appelés khipukamayuq. Des centaines
de khipus subsistent dans les collections, mais leurs conventions complètes ne
sont pas toutes déchiffrées et quelques communautés andines en ont conservé la
garde.

Le prompt ne donne donc aucune signification historique inventée aux couleurs
ou aux nœuds. Il utilise seulement la propriété documentée d'un registre
matériel continu, auquel de nouvelles cordes peuvent être ajoutées sans
transformer le registre en monnaie.

Sources culturelles :

- [UNESCO Memory of the World — The Khipu Database](https://www.unesco.org/en/memory-world/lac/khipu-database-khipu-archives)
- [Smithsonian National Museum of the American Indian — Quipu](https://www.si.edu/collections/snapshot/quipu)
- [NIST Museum — khipukamayuq et précision des khipus](https://www.nist.gov/nist-museum/standardizing-empire)

### Preuve Nexus et limite

Après validation et passage du garde-fou, le daemon appelle `credit`. Le ledger
applique `1000 × log2(1 + tokens bornés)`, relie les entrées d'un projet par une
chaîne BLAKE3 et calcule un score effectif avec une décroissance EMA de 0,97 par
jour. Les métriques Gini et part du haut servent à l'observation, pas à une
redistribution automatique. En V1, ces kudos décrivent surtout le calcul
accepté ; ils ne sont ni monnaie, ni actif transférable, ni droit de vote.

Rejeter l'image si les cordes deviennent argent, impôt, médaille ou suffrage,
si un worker devient physiquement géant, si un khipukamayuq est représenté
comme souverain omniscient, si d'anciens cordons disparaissent ou si des
significations historiques précises sont attribuées aux couleurs.

## Audit technique du lot Workflow Hub

Ce troisième lot ne reprend pas les promesses générales de la recherche produit
comme si toute l'interface était déjà livrée. Il part des primitives commitées
dans `sbfb-workflow-core` et `sbfb-workflow-engine` :

- le langage fermé de `graph.rs` comprend notamment `agent.invoke`,
  `agent.as_tool`, `handoff`, `parallel`, `map`, `join` et `subworkflow` ;
- `compile.rs` transforme le graphe résolu en `CompiledPlan.v1`, vérifie les
  références, ports et schémas épinglés, les handoffs, les joins et les bornes ;
- `orchestrator.rs` et `reducer.rs` reconstruisent les décisions depuis le
  journal, ordonnent le ready-set de manière canonique et journalisent les
  fan-outs et cohortes de join avant de s'y fier ;
- `store.rs` conserve journal, frontière, waits et signaux ; une consommation
  atomique dépense définitivement le nonce qui réveille une attente.

Limite produit : `docs/workflow/REFERENCE.md` confirme les routes de lecture du
run store, mais reporte encore la famille mutante de contrôle des runs. Les
illustrations expliquent donc le compilateur et le runtime réellement présents
qui doivent soutenir Workflow Hub, pas un auteur graphique, un catalogue ou un
centre de contrôle supposés entièrement disponibles aujourd'hui. Le travail
non commité en cours sur HITL, secrets et coffre n'a pas été utilisé pour ce
lot.

### Correction après le test Ideogram du prompt 15

Le premier prompt 15, consacré à l'idempotence dans une frise d'Uruk, a été
retiré de la série active après génération. Malgré une culture distincte, il
répétait la même famille formelle que plusieurs images précédentes : information
inscrite sur un rectangle plat — tissu, estampe, livre, tablette. Ideogram a en
outre transformé les bullae en grands boucliers et le relief en faux artefact
archéologique, sans rendre le doublon intelligible.

Le JSON reste dans `retired/` comme contre-exemple. La nouvelle règle de
sélection porte autant sur la **forme de l'image** que sur la communauté :

- au maximum deux supports plats dominants par groupe de cinq ;
- au moins une performance corporelle ;
- au moins une scène spatiale ou architecturale ;
- au moins un mécanisme, paysage ou objet volumique ;
- aucune nouvelle culture ne compte comme diversité si elle reproduit la même
  grammaire « panneau qui contient de l'information ».

## 11 — Atelier collaboratif de l'ukiyo-e d'Edo

### Histoire retenue

Une estampe ukiyo-e n'était pas la production solitaire du dessinateur célèbre.
Sa fabrication commerciale associait quatre expertises : dessinateur, graveur,
imprimeur et éditeur. Le dessin était transféré sur un bloc de cerisier ; le
graveur préparait le relief, puis l'imprimeur alignait le papier et les blocs de
couleur avant que l'éditeur ne diffuse la feuille.

La scène conserve cette chaîne matérielle sans transformer l'éditeur en
souverain du groupe. Elle invente une image séculière de deux oiseaux de rivière
et n'imite aucune estampe reconnaissable.

Source culturelle :

- [The Metropolitan Museum of Art — Woodblock Prints in the Ukiyo-e Style](https://www.metmuseum.org/essays/woodblock-prints-in-the-ukiyo-e-style)

### Preuve Nexus et limite

`agent_definition.rs` lie chaque agent à un rôle, des schémas d'entrée et de
sortie, une politique de modèle, des outils et des bornes. `graph.rs` distingue
l'appel simple, l'agent utilisé comme outil et le handoff ; ce dernier exige un
schéma de payload et une allowlist de contexte. `compile.rs` refuse un agent non
déclaré, un handoff non autorisé et un port dont le digest de schéma ne
correspond pas.

Rejeter l'image si les quatre personnes font le même métier, si le dessin saute
directement à la feuille finale, si l'éditeur devient géant ou trône, si des
cartouches japonais inventés apparaissent ou si le rendu ressemble à une photo
d'atelier.

## 12 — Papel picado de San Salvador Huixcolotla

### Histoire retenue

À San Salvador Huixcolotla, le papel picado est produit en frappant des ciseaux,
poinçons et demi-lunes métalliques sur des blocs superposés de papier de soie
posés sur une base. Un motif préparé peut ainsi traverser plusieurs feuilles.
La pratique constitue un métier familial et une activité économique majeure de
la commune.

La butée rigide du prompt est une métaphore graphique de la borne Nexus, pas un
outil historique revendiqué comme universel dans les ateliers de Huixcolotla.
Le motif d'agave et d'hirondelles est original, séculier et dépourvu de texte.

Sources culturelles :

- [Diario Oficial de la Federación — demande d'indication géographique « Papel Picado de San Salvador Huixcolotla, Puebla »](https://sidof.segob.gob.mx/notas/5792806)
- [Museo Nacional de Culturas Populares — Perforando papel picado en San Salvador Huixcolotla](https://mncp.cultura.gob.mx/obras-2/perforando-papel-picado-en-san-salvador-huixcolotla/)

### Preuve Nexus et limite

`MapNode` porte un `max_cardinality` non nul et un sous-workflow épinglé. Au
runtime, `orchestrator.rs` refuse une collection trop grande avant de
journaliser le fan-out ; `reducer.rs` refuse également un journal corrompu dont
la cardinalité dépasserait la borne signée. Toutes les feuilles admises suivent
le même enfant, mais la collection ne peut pas agrandir le plan.

Rejeter l'image si la feuille excédentaire est forcée sous la butée, si les
bannières terminées portent des motifs différents, si le papel picado devient
une bordure autour d'une scène réaliste, ou si des crânes, saints, lettres et
symboles nationaux sont ajoutés.

## 13 — Tissage kente ewe en bandes étroites

### Histoire retenue

Les tissus de prestige kente ewe sont construits à partir de longues bandes
étroites tissées séparément. Elles sont coupées à longueur puis cousues
selvedge contre selvedge pour former une étoffe beaucoup plus large. Les
collections documentent à la fois la régularité technique et une grande
souplesse d'invention dans les motifs de trame supplémentaire.

La scène ne prétend pas qu'un atelier ewe aurait historiquement modélisé un
ordonnanceur. Elle rend visible la propriété matérielle documentée : des bandes
autonomes restent reconnaissables après leur réunion.

Sources culturelles :

- [The Metropolitan Museum of Art — Man's kente, Ewe artist](https://www.metmuseum.org/art/collection/search/320677)
- [British Museum — Ewe kente composé de dix-neuf bandes](https://www.britishmuseum.org/collection/object/E_Af1980-30-3)
- [British Museum — tisserand ewe de Kpetoe](https://www.britishmuseum.org/collection/object/E_Af2006-15-174)

### Preuve Nexus et limite

`ParallelNode` active toutes ses branches statiques. `JoinNode` déclare sa
politique et la liste exacte de ses sources. Le reducer observe les arrivées
réelles ; l'orchestrateur journalise `join_fired` avec la cohorte retenue afin
qu'une arrivée tardive ne puisse ensuite élargir le quorum ni remplacer le
gagnant d'un join `any`.

Rejeter l'image si une bande disparaît dans la grande étoffe, si les bandes se
fusionnent avant d'être terminées, si une quatrième bande cachée entre dans le
join, si l'assembleuse devient cheffe souveraine ou si la scène est une photo
habillée d'une bordure kente.

## 14 — Manuscrits picturaux ñuu savi

### Histoire retenue

Le Codex Tonindeye est un des rares livres picturaux mésoaméricains
précoloniaux conservés. Peint sur peau et plié en accordéon, il porte deux
récits sur ses faces. Sa lecture contemporaine demande la participation des
chercheuses, chercheurs et locuteurs autochtones descendants ; il ne s'agit
donc pas d'une réserve de glyphes disponibles pour un décor générique.

Le prompt invente une histoire séculière de poterie et interdit les noms
calendaires, divinités, lignées et signes reconnaissables. Le médium fournit
seulement la continuité matérielle d'un récit conservé après interruption.

Sources culturelles :

- [British Museum — An Indigenous reframing of the fall of the Aztec empire](https://www.britishmuseum.org/blog/indigenous-reframing-fall-aztec-empire)
- [Smarthistory et British Museum — Codex Tonindeye](https://smarthistory.org/codex-zouche-nuttall/)

### Preuve Nexus et limite

`reducer.rs` définit l'état comme une fonction pure de `CompiledPlan` et du
journal : pending, ready, waiting et succès structurels sont reconstruits à
chaque réduction. `engine.rs` précise que replay n'est pas réexécution : une
étape achevée est servie depuis son reçu, indexé par `node_instance_id`, sans
retoucher la cible. L'onglet ou le processus peut donc disparaître sans que
l'état dépende d'un cache mémoire.

Rejeter l'image si les quatre scènes terminées sont repeintes, si l'interruption
efface une feuille, si le peintre saute plusieurs feuilles, si des glyphes
historiques sont inventés, ou si le manuscrit devient un flowchart muni de
flèches.

## 15 — Guildes de marionnettes sur eau du delta du Fleuve Rouge

### Histoire retenue

Le múa rối nước s'est développé dans les communautés agricoles du delta du
Fleuve Rouge. Le bassin forme la scène ; les marionnettistes restent derrière
le rideau du pavillon et commandent sous l'eau des figures de bois peintes et
laquées. Les guildes emploient notamment tambours, cymbales et cloches de bois.
La musique règle le rythme et accompagne les mouvements des marionnettes.

L'association exacte entre les trois motifs, les trois instruments et les trois
marionnettes est une métaphore originale du signal typé ; elle n'est pas
présentée comme une convention historique des guildes. La scène reste
entièrement spatiale : eau, pavillon, corps, instruments et mouvement, sans
support documentaire.

Sources culturelles :

- [Viet Nam National Authority of Tourism — Water puppetry, musique et manipulation](https://vietnamtourism.gov.vn/en/post/10599)
- [Vietnam National Puppetry Theatre — Unique Features of Traditional Water Puppetry](https://nhahatmuaroivietnam.vn/en/node/986)
- [Viet Nam Government News — Water puppetry, instruments et guildes](https://en.baochinhphu.vn/water-puppetry-a-unique-folk-art-form-of-viet-nam-11122080210112769.htm)

### Preuve Nexus et limite

`WaitSignalNode` porte un nom de signal statique et un schéma de payload
épinglé. Lorsqu'il atteint ce nœud, l'orchestrateur ouvre une attente durable
avec un nonce dérivé par le moteur. Le host dépose le payload ; le store
consomme le dépôt, journalise la reprise, ferme l'attente et dépense le nonce
dans une seule transaction. Rejouer la même livraison ne peut donc pas
réveiller le run une seconde fois.

Rejeter l'image si le buffle ou le canard bougent avec la grenouille, si le
musicien frappe plusieurs instruments, si les motifs correspondants ne sont pas
lisibles, si des opérateurs apparaissent devant le rideau, si le bassin devient
un diagramme, ou si le rendu ressemble à une photographie de spectacle.

## 16 — Communautés agricoles des subak de Bali

### Histoire et lecture de l'image

Le subak est un système coopératif de gestion de l'eau par des canaux, tunnels,
barrages et rizières en terrasses. L'illustration conserve cette infrastructure
réelle : un canal amont rencontre trois vannages construits ; un seul est levé,
une seule rizière reçoit l'eau et les deux autres voies restent sèches. Le bassin
central constitue le repli prévu.

Lecture Nexus : les canaux sont les branches déclarées, le vannage choisi est la
valeur du discriminant et l'eau est le payload. Aucun courant ne creuse une
quatrième voie pendant l'exécution.

Source culturelle :

- [UNESCO — Cultural Landscape of Bali Province: the Subak System](https://whc.unesco.org/en/list/1194)

Rendu GPT :

- [`renders/16_subak_switch_ferme_gpt.png`](renders/16_subak_switch_ferme_gpt.png)

### Preuve Nexus et limite

`SwitchNode` porte un champ discriminant et une liste statique de labels.
L'orchestrateur lit une chaîne déjà validée, choisit le port correspondant ou
le port réservé `default`, puis journalise cette décision. Il ne construit
jamais un nouveau port.

Le paysage hydraulique est réel ; l'équivalence entre ses voies et un `switch`
est l'analogie. Le rendu actuel rend très bien la branche choisie, mais le bassin
de repli reste moins distinct que les trois vannages. Rejeter une future version
si les canaux ressemblent à des flèches, si plusieurs branches sont mouillées
ou si le vannier devient une autorité religieuse.

## 17 — Onggijang et fabrication coréenne de l'onggi

### Histoire et lecture de l'image

La pratique réelle emploie de l'argile, un tour, une palette extérieure
`surae`, une enclume intérieure `dogae`, des outils de lissage et une finition
humide du col avant séchage et cuisson. Ces outils restent visibles dans la
version retenue.

Le cadre en bois, sa butée supérieure et la feuille mobile ne sont pas
présentés comme des outils traditionnels : ce sont des ajouts Nexus assumés. La
jarre terminée soulève la feuille avant d'atteindre le plafond ; le potier ouvre
les mains et une bande d'argile demeure inutilisée.

Lecture Nexus : les passages successifs sur la paroi sont les itérations, la
feuille levée est `until_output_flag=true`, la traverse supérieure est
`max_iterations` et l'argile restante montre l'arrêt anticipé.

Source culturelle :

- [Korea Heritage Service — outils et étapes de fabrication de l'onggi](https://kh.or.kr/brd/board/696/L/menu/314?bbIdx=110616&brdType=R&searchField=&searchText=&thisPage=1)
- [Korea Heritage Service — présentation patrimoniale de l'onggi](https://english.cha.go.kr/cop/bbs/selectBoardArticle.do?bbsId=BBSMSTR_1200&mn=EN_01_01&nttId=49716&pageUnit=10)

Rendu GPT :

- [`renders/17_onggi_repeat_bornee_gpt.png`](renders/17_onggi_repeat_bornee_gpt.png)

### Preuve Nexus et limite

`RepeatNode` épingle son sous-workflow, son nombre maximal d'itérations et le
nom d'un booléen de sortie. L'orchestrateur arrête la boucle lorsque ce booléen
vaut vrai ou lorsque la borne signée est atteinte ; le modèle ne peut étendre
la borne.

Cette image choisit consciemment la compréhension technique au prix d'un objet
inventé. Le livre doit donc toujours préciser que le cadre est métaphorique.
Rejeter l'image si le cadre est décrit comme historique, si les vrais outils
disparaissent, si une nouvelle argile apparaît indéfiniment ou si plusieurs
jarres transforment la scène en séquence de panneaux.

## 18 — Artisanes et artisans des murrines de Murano

### Histoire et lecture de l'image

La murrine est obtenue en assemblant des cannes de verre colorées autour d'un
noyau, puis en coupant la canne pour révéler son motif. Le four, les cannes et
les tranches de verre appartiennent à cette pratique ; le carrousel à trois
berceaux et sa coupelle centrale sont une construction Nexus.

Lecture Nexus : fleur rouge, spirale verte et étoile bleue sont les trois labels
fermés. La tranche asymétrique tenue par la pince ne correspond à aucun ; elle
rejoint la coupelle neutre. Aucun quatrième berceau n'est fabriqué.

Source culturelle :

- [UNESCO — The art of glass beads](https://ich.unesco.org/en/RL/the-art-of-glass-beads-01591)

Rendu GPT :

- [`renders/18_murano_classifieur_ferme_gpt.png`](renders/18_murano_classifieur_ferme_gpt.png)

### Preuve Nexus et limite

`ClassifierNode` déclare une liste fermée. Après l'appel modèle, le runtime
accepte seulement un label de cette liste ; toute proposition hors ensemble
rejoint `default`. Le compilateur refuse en plus qu'une classification issue
du modèle autorise directement un effet en aval lorsque sa provenance n'est
pas déterministe ou humaine.

Le rendu emprunte au verre coloré mais n'est pas un artefact historique de
Murano. Rejeter l'image si une quatrième coupelle apparaît, si la tranche
inconnue entre dans un berceau existant, si les motifs deviennent des lettres
ou si le four donne l'impression que le classement déclenche déjà une action
sensible.

## 19 — Horlogerie et mécanique d'art de l'Arc jurassien

### Histoire et lecture de l'image

Les communautés horlogères franco-suisses de l'Arc jurassien fabriquent des
montres, horloges, automates, oiseaux chanteurs et objets de mécanique d'art.
Leurs mécanismes cachés peuvent aussi être rendus visibles comme partie de
l'esthétique de l'objet.

Lecture Nexus : les trois billes de la crémaillère constituent le budget
d'essais ; deux sont déjà tombées. La dernière serre le frein rouge autour du
ressort et ouvre le berceau de réparation. L'oiseau bloqué ne peut plus
recommencer.

Source culturelle :

- [UNESCO — Craftsmanship of mechanical watchmaking and art mechanics](https://ich.unesco.org/en/RL/craftsmanship-of-mechanical-watchmaking-and-art-mechanics-01560)

Rendu GPT :

- [`renders/19_jura_reessais_bornes_gpt.png`](renders/19_jura_reessais_bornes_gpt.png)

### Preuve Nexus et limite

`ErrorPolicy` fixe `max_retries_per_node` et l'action après épuisement.
L'orchestrateur journalise chaque échec, programme un délai de backoff si une
tentative reste disponible, puis applique `fail-workflow` ou le port `error`
déclaré. Sans sortie d'erreur câblée, il échoue fermé.

Le mécanisme précis de trois billes et du berceau est inventé ; l'automate et
la mécanique visible sont culturellement fondés. Une image fixe ne montre pas
la durée du backoff. Rejeter si une quatrième bille apparaît, si le ressort
continue après le frein ou si le berceau renvoie directement l'oiseau dans la
même boucle.

## 20 — Écoles de samba et barracões de Rio de Janeiro

### Histoire et lecture de l'image

Les écoles de samba sont des communautés durables et leurs ateliers produisent
collectivement costumes et chars allégoriques. Le samba-enredo organise le
défilé ; les allégories matérialisent son récit sur des véhicules poussés sur
l'avenue.

Lecture Nexus : le barracão est l'espace d'édition, le portail est la
compilation et l'avenue est le run. Le char qui l'a franchi conserve ses pièces
fixées. La tête de poisson, la nageoire et l'arche restées sur leurs chariots
sont des modifications possibles pour une autre version, pas pour ce run.

Sources culturelles :

- [IPHAN — Matrizes do samba no Rio de Janeiro](https://www.gov.br/iphan/pt-br/superintendencias/rio-de-janeiro/patrimonio-imaterial)
- [Biblioteca Nacional Digital — Carnaval, écoles de samba et chars allégoriques](https://bndigital.bn.gov.br/carnaval/)

Rendu GPT :

- [`renders/20_samba_plan_compile_immuable_gpt.png`](renders/20_samba_plan_compile_immuable_gpt.png)

### Preuve Nexus et limite

Le compilateur Rust produit un `CompiledPlan.v1` immuable contenant les digests
d'artefacts, occurrences de nœuds, câblage, capacités fermées, budgets et
politiques. Modifier un brouillon produit un nouveau plan ; cela ne modifie
jamais le plan d'un run en cours.

Le portail étroit est une invention métaphorique, pas une règle des barracões.
Rejeter l'image si une pièce quitte l'atelier et se fixe au char après le
portail, si un deuxième char remplace silencieusement le premier, si le
portail devient un poste d'autorité ou si la scène devient une photographie de
Carnaval.

## Discipline commune

Ces analogies ont des limites :

- elles n'affirment pas que les communautés auraient « anticipé » Nexus ;
- elles ne donnent aucun privilège protocolaire à une communauté ;
- elles ne transforment pas une pratique vivante en banque gratuite de motifs ;
- elles ne permettent pas de générer une cérémonie, une croyance ou un symbole
  sacré non documenté ;
- elles demandent une revue culturelle humaine avant publication, surtout pour
  les traditions autochtones et les mouvements d'auto-représentation.

Pour le test aveugle, montrer d'abord l'image sans la phrase Nexus et demander :
« Que font les personnes ? Qu'est-ce qui peut agir ? Qu'est-ce qui reste
limité, séparé ou commun ? » Ajouter ensuite la phrase et vérifier que l'image
clarifie le mécanisme au lieu de le contredire.
