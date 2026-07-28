# Pilote France 2 — cinq autres fonctionnalités Nexus

Ce lot prolonge le premier pilote avec cinq fonctionnalités différentes. Chaque
prompt a été confié à une analyse indépendante : contrat du livre, preuves dans
le dépôt live, historique Git, recherche culturelle, construction spatiale,
risques Ideogram et test aveugle.

Les images restent entièrement wordless. Les phrases explicatives appartiennent
au livre, jamais à l'illustration.

| Prompt | Fonctionnalité | Phrase courte | Tradition visuelle | Situation temporelle | Mécanisme visuel |
|---|---|---|---|---|---|
| [`06_rrv_provenance.json`](06_rrv_provenance.json) | RRV et niveaux de preuve | Retrouver ou lire une source ne la rend pas vraie ; RRV conserve son origine sans la déclarer automatiquement vérifiée. | Imprimeurs-libraires et graveurs de la Renaissance lyonnaise | Milieu professionnel historique disparu sous cette forme | Un folio traçable à sa presse est lu et comparé, mais reste hors de l'armoire contenant les dossiers corroborés. |
| [`07_projection_minimale.json`](07_projection_minimale.json) | Projection publique minimale | Les données privées restent locales ; seule une projection volontairement limitée est partagée ou publiée. | Tapisserie d'Aubusson | Savoir-faire vivant | Une grande œuvre riche reste intacte dans l'atelier ; la lissière expose dehors un carré séparé ne reprenant qu'un motif. |
| [`08_curation_plurielle.json`](08_curation_plurielle.json) | Curations plurielles | Plusieurs curateurs publient leurs listes signées ; aucune ne devient le classement mondial du réseau. | Faïence historique de Nevers | Tradition locale née de circulations culturelles et toujours vivante | Trois plats égaux colorent des sélections différentes du même corpus et portent trois marques d'origine équivalentes. |
| [`09_outcome_unknown.json`](09_outcome_unknown.json) | Effet non idempotent sans reçu | Si un effet a peut-être eu lieu sans reçu fiable, le runtime déclare l'incertitude et ne recommence pas aveuglément. | Communauté horlogère bisontine et planche technique historique | Communauté technique historique avec transmission patrimoniale | Une première capsule a franchi le portillon, le retour est vide et le mécanisme bloque physiquement le duplicata. |
| [`10_heberger_pas_publier.json`](10_heberger_pas_publier.json) | Héberger n'est pas publier | Deux nœuds peuvent servir la même archive sans devenir les auteurs de la publication signée. | Vitrail narratif de Chartres et métier de maître-verrier | Langage historique transmis par une communauté professionnelle vivante | Une origine marque un coffre ; deux dépôts égaux conservent des copies portant exactement la même marque, sans outil de signature. |

## Statut honnête dans le dépôt

- **06 — RRV** : Browse, provenance et Proof Cards existent, mais le moteur RRV
  général appliquant tous les statuts d'affirmation du manuscrit n'est pas
  encore livré.
- **07 — projection publique** : des projections locales minimisées et leurs
  allowlists sont testées ; `WorkflowPublicProjection.v1` reste un contrat
  public réservé mais vide.
- **08 — curation** : les listes signées et leur attribution existent dans le
  backend. L'interface Browse actuelle ne juxtapose pas encore parfaitement
  plusieurs curateurs pour le même projet.
- **09 — outcome unknown** : l'invariant est directement exercé dans le moteur
  et ses tests : un effet non idempotent sans reçu devient inconnu, sans retry
  automatique.
- **10 — hébergement** : les chemins de réplication, de seeding, de provenance
  et la séparation `seeder != author` possèdent des preuves concrètes. L'image
  reste une explication conceptuelle, pas une capture de l'interface.

Les preuves exactes, commits et limites se trouvent dans les notes :

- [`06_rrv_provenance_notes.md`](06_rrv_provenance_notes.md)
- [`07_projection_minimale_notes.md`](07_projection_minimale_notes.md)
- [`08_curation_plurielle_notes.md`](08_curation_plurielle_notes.md)
- [`09_outcome_unknown_notes.md`](09_outcome_unknown_notes.md)
- [`10_heberger_pas_publier_notes.md`](10_heberger_pas_publier_notes.md)

## Ordre de génération recommandé

1. **07 — projection minimale** : relation source/projection la plus simple.
2. **09 — outcome unknown** : mécanique précise et fortement ancrée dans le code.
3. **10 — héberger n'est pas publier** : tester si le sceau reste vraiment
   identique dans les trois emplacements.
4. **06 — provenance RRV** : vérifier que l'armoire n'est pas interprétée comme
   une censure.
5. **08 — curation plurielle** : génération la plus exigeante, car les six
   objets doivent rester reconnaissables et aux mêmes positions sur trois plats.

## Rejets immédiats

Rejeter une génération si :

- Ideogram ajoute un texte, une flèche, un badge ou un classement ;
- un sceau devient une médaille de qualité plutôt qu'une marque d'origine ;
- la source privée est découpée ou visible depuis l'espace public ;
- le duplicata de `outcome_unknown` franchit le mécanisme ;
- un hébergeur ajoute sa propre marque au coffre répliqué ;
- une tradition devient un collage de costumes, drapeaux ou monuments ;
- la beauté du médium rend impossible l'explication du rapport de pouvoir.
