# Pilote France — cinq fonctions, cinq communautés visuelles

Ce lot teste une direction éditoriale, pas une charte graphique. Chaque image
emploie une tradition visuelle différente pour expliquer une seule propriété
de Nexus. Aucune communauté n'est présentée comme propriétaire, caution ou
incarnation officielle du protocole.

Les prompts sont destinés à Ideogram 4 en `768:1376`. Ils ne contiennent aucun
élément textuel : la phrase française reste dans le livre, tandis que l'image
doit porter l'idée par la scène, les positions, les barrières et les gestes.

| Fichier | Fonction Nexus | Phrase courte du livre | Communauté ou tradition visuelle | Pourquoi l'association fonctionne |
|---|---|---|---|---|
| `01_epinal_bridge_local.json` | Bridge typé et autorisation locale | L'application demande une opération précise ; le nœud vérifie puis accepte ou refuse son exécution. | Imagiers populaires d'Épinal, Vosges | Une planche populaire frontale permet de rendre le seuil, la requête unique et le geste de décision immédiatement lisibles. |
| `02_bretagne_noeud_local_first.json` | Nœud local-first | Même coupé du réseau, le nœud ouvre les applications déjà reçues, consulte ses données et prépare le travail localement. | Graveuses et graveurs sur bois de Bretagne | Le contraste noir, les masses simples et la scène de tempête rendent visible la coupure extérieure sans rendre l'atelier intérieur inactif. |
| `03_bayeux_workflow_immuable_v2.json` | Workflow compilé immuable | Une exécution suit le plan validé jusqu'au bout ; toute modification devient une nouvelle version pour un prochain départ. | Atelier historique de broderie narrative de Bayeux, Normandie | Le plan bleu reste visible mais enfermé dans un guide scellé ; l'opérateur n'atteint que la manivelle et la version rouge attend dans un berceau déconnecté. |
| `04_canuts_iroh_transport.json` | Iroh transporte sans gouverner | Iroh fait circuler les contenus entre des nœuds autonomes ; le relais ne devient ni auteur, ni propriétaire, ni autorité de publication. | Canuts et tisseurs de la Croix-Rousse, Lyon | Les fils, métiers indépendants et navettes matérialisent le transport sans créer de métier central dominant. |
| `05_compagnons_factory_preuves.json` | Factory, preuves et décision humaine | Factory fabrique et rassemble les preuves ; une revue indépendante contrôle le résultat et une personne habilitée garde la décision finale. | Compagnons des métiers du bois et art du trait | L'épure, l'assemblage, la mesure contradictoire et la dernière cheville tenue par une autre personne séparent visuellement fabrication, revue et autorisation. |

## Ancrage dans le dépôt

Les cinq images n'ont pas toutes le même statut d'implémentation. Le livre
décrit une V1 complète ; le dépôt en construit les couches progressivement.

- Le bridge possède déjà un schéma de requête typé et une autorité effective
  calculée appel par appel par intersection entre déclaration, grant et
  politique locale :
  [`web/src/bridge/protocol.ts`](../../../../../web/src/bridge/protocol.ts) et
  [`web/src/bridge/authority.ts`](../../../../../web/src/bridge/authority.ts).
- Le cache d'archives servi localement existe dans
  [`crates/nexus-shell-daemon/src/blob_serve_http.rs`](../../../../../crates/nexus-shell-daemon/src/blob_serve_http.rs).
  L'expérience local-first complète de la phrase reste une cible V1 plus large
  que ce seul cache.
- `CompiledPlan.v1` est défini comme un plan immuable produit par le
  compilateur Rust dans
  [`crates/sbfb-workflow-core/src/compile.rs`](../../../../../crates/sbfb-workflow-core/src/compile.rs).
- Les échanges Iroh par blobs et gossip sont présents dans
  [`crates/nexus-shell-daemon-core/src/iroh_runtime.rs`](../../../../../crates/nexus-shell-daemon-core/src/iroh_runtime.rs).
  La séparation entre transport, auteur et autorité est un contrat
  d'architecture, pas une propriété créée par le relais lui-même.
- Les gates Factory sont décrites et reliées au code dans
  [`docs/factory/FACTORY_GATES.md`](../../../../factory/FACTORY_GATES.md).
  La revue indépendante et la confirmation humaine sont aussi des exigences de
  processus ; elles ne doivent pas être présentées comme un simple test
  automatique déjà suffisant.

## Ressources de vérification

### 1. Imagiers populaires d'Épinal

- Musée de l'Image, présentation de l'imagerie populaire :
  <https://museedelimage.fr/ressources/les-editions-du-musee>
- Galerie d'images sur Wikimedia Commons :
  <https://commons.wikimedia.org/wiki/Category:Images_d%27%C3%89pinal>

À vérifier dans le rendu : contours francs, couleurs en aplats, composition
frontale et populaire. Éviter les légendes moralisantes, les uniformes
napoléoniens et l'imitation d'une planche historique précise.

### 2. Graveuses et graveurs sur bois de Bretagne

- Musée des Beaux-Arts de Brest, exposition *Bois Bretagne* :
  <https://musee.brest.fr/bois-bretagne-la-gravure-sur-bois-en-bretagne-1850-1950>

À vérifier dans le rendu : taille franche, noirs structurants, formes
synthétiques et nombre limité de couleurs. Le prompt ne reprend aucun signe
religieux, nationaliste ou motif présenté comme l'essence de la Bretagne.

### 3. Broderie narrative de Bayeux

- Panorama officiel haute définition, Ville de Bayeux, DRAC Normandie,
  Université de Caen et CNRS :
  <https://www.bayeuxmuseum.com/la-tapisserie-de-bayeux/decouvrir-la-tapisserie-de-bayeux/explorer-la-tapisserie-de-bayeux-en-ligne/>
- Présentation de sa narration en scènes successives :
  <https://www.bayeuxmuseum.com/la-tapisserie-de-bayeux/la-tapisserie-au-fil-de-son-histoire/la-tapisserie-de-bayeux-une-source-dinspiration-inepuisable/>

À vérifier dans le rendu : toile de lin, fil de laine, profils plats, bordures
et narration continue. Ne pas copier une scène, un personnage ou une
inscription de l'œuvre.

### 4. Canuts de la Croix-Rousse

- Maison des Canuts, histoire sociale et démonstration du métier Jacquard :
  <https://maisondescanuts.fr/>

À vérifier dans le rendu : structure réelle du métier, chaîne, trame, cartes
perforées et atelier haut de plafond. Ne pas réduire les canuts à un décor
luxueux : leur histoire est aussi ouvrière, technique et sociale.

### 5. Compagnons des métiers du bois

- Musée du Compagnonnage de Tours, collections consacrées au bois :
  <https://www.museecompagnonnage.fr/le-compagnonnage/la-main/nos-collections/le-bois>

À vérifier dans le rendu : épure grandeur nature, géométrie appliquée,
assemblages lisibles, outils plausibles et rôles distincts. Ne pas utiliser de
symboles initiatiques ni présenter le compagnonnage comme une autorité secrète.

## Protocole de test

Pour chaque génération, masquer d'abord la phrase du livre et demander à trois
personnes : « Que se passe-t-il ? Qui peut agir ? Qu'est-ce qui reste séparé ? »
L'image passe seulement si deux personnes sur trois décrivent le bon rapport
de pouvoir sans avoir besoin de connaître Nexus.

Évaluer séparément :

1. compréhension de la mécanique ;
2. fidélité culturelle générale ;
3. absence de caricature ;
4. absence de texte ou de flèche produits par le modèle ;
5. absence d'un personnage ou d'un objet qui semble devenir l'autorité de
   tout le système.
