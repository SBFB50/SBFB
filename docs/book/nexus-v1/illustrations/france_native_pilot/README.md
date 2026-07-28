# Pilote culturel natif — cinq fonctionnalités Nexus

Ce lot applique une méthode différente des deux pilotes précédents :

> La culture ne fournit pas un style pour habiller un diagramme. Elle fournit
> une pratique réelle dont les objets, les gestes et le récit éclairent ensuite
> une fonctionnalité Nexus.

Chaque scène doit donc rester plausible et compréhensible comme activité
culturelle ou professionnelle même si toute référence à Nexus est retirée.
La correspondance technique appartient à la légende et aux notes de recherche.

## Les cinq scènes

| Prompt | Pratique culturelle racontée | Fonctionnalité éclairée | Pourquoi la relation est native |
|---|---|---|---|
| [`01_rrv_source_pas_verite.json`](01_rrv_source_pas_verite.json) | Un correcteur lyonnais du XVIe siècle compare la première épreuve d'un bois gravé avec le modèle fourni par l'artiste. | RRV conserve origine et différences ; trouver ou lire une source ne garantit pas sa justesse. | La presse, le modèle, le bois, l'épreuve et les états successifs appartiennent réellement au travail d'impression. Aucun meuble de statut ou seuil de sceaux n'est ajouté. |
| [`02_projection_minimale.json`](02_projection_minimale.json) | Une osmothécaire de Versailles conserve le flacon original et prépare pour l'extérieur une unique mouillette parfumée protégée par une pochette. | L'ensemble riche reste local ; seule une projection volontairement limitée sort. | La charte réelle de l'Osmothèque interdit la consultation ou le prélèvement des formules et permet l'envoi d'une mouillette sous cellophane. |
| [`03_curation_plurielle.json`](03_curation_plurielle.json) | Une marchande de faïence déballe trois séries françaises d'assiettes historiées, issues de manufactures et d'époques distinctes, qui sélectionnent différentes Fables de La Fontaine. | Plusieurs sélections attribuées peuvent coexister sans former un classement mondial. | Les séries Creil, Creil-Montereau et Digoin sont documentées ; leur coexistence dans une boutique de 1970 est plausible et aucune grille de comparaison n'est inventée. |
| [`04_outcome_unknown.json`](04_outcome_unknown.json) | Un garde de block ferroviaire attend la reddition du train engagé dans un canton noyé de brouillard et maintient le sémaphore fermé devant le train suivant. | Sans confirmation fiable d'un effet potentiellement accompli, le runtime conserve l'incertitude et bloque la répétition automatique. | Le cantonnement manuel exige réellement de constater l'arrivée complète avant de rendre la voie libre. Le signal fermé est une pratique de sécurité, pas une icône de statut inventée. |
| [`05_heberger_pas_publier.json`](05_heberger_pas_publier.json) | Un atelier parisien de la Renaissance tire une édition dont des exemplaires portant la même marque à l'olivier partent vers lecteur, colporteur et bibliothèque. | Plusieurs nœuds peuvent conserver et servir les mêmes octets sans devenir les auteurs de la publication. | La forme, le tirage, la marque d'imprimeur, la circulation des livres et le dépôt sont natifs de la culture imprimée. Les destinataires n'ont pas besoin d'être transformés en gardiens symboliques. |

## Recherches ultra-deep

- [`01_rrv_source_pas_verite_research.md`](01_rrv_source_pas_verite_research.md)
- [`02_projection_minimale_research.md`](02_projection_minimale_research.md)
- [`03_curation_plurielle_research.md`](03_curation_plurielle_research.md)
- [`04_outcome_unknown_research.md`](04_outcome_unknown_research.md)
- [`05_heberger_pas_publier_research.md`](05_heberger_pas_publier_research.md)

Chaque dossier contient :

- la pratique culturelle avant toute analogie ;
- les sources institutionnelles et références visuelles ;
- la plausibilité de chaque objet et geste ;
- le mapping Nexus dans une section séparée ;
- les preuves du livre, du code live et de Git ;
- les différences entre V1 décrite et implémentation actuelle ;
- les risques Ideogram ;
- un test aveugle et ses seuils d'échec.

## Décisions issues de la recherche

La recherche avait le droit de rejeter le premier appariement :

- Aubusson a été écarté pour la projection minimale : l'échantillon textile
  documente surtout la création et le choix d'atelier. L'Osmothèque possède une
  règle réelle beaucoup plus exacte.
- Nevers seule a été écartée pour la curation : les attributions anciennes
  sont parfois trop incertaines. Les séries documentées des Fables fournissent
  une pluralité vérifiable.
- L'horlogerie a été écartée pour `outcome_unknown` : la machine à capsules du
  pilote précédent était un diagramme inventé. Le block ferroviaire contient
  naturellement attente, absence de confirmation et suspension.
- Le vitrail symétrique a été écarté pour l'hébergement : les deux dépôts
  identiques étaient artificiels.

## Doublon assumé : deux communautés de l'imprimé

Les prompts 01 et 05 utilisent tous deux l'imprimerie de la Renaissance, mais
pas le même geste ni le même milieu :

- Lyon : correction d'une épreuve avant nouveau tirage ;
- Paris : production et circulation de plusieurs exemplaires d'une édition.

Les recherches ont étudié Lascaux, les moulages, le vitrail, la chalcographie,
la partition et le dépôt d'œuvres. Aucun candidat ne conservait aussi
directement la distinction entre origine, exemplaires identifiables et
disponibilité. Le doublon est donc maintenu plutôt que de forcer une cinquième
culture. Dans le livre, ces deux images ne devront pas apparaître côte à côte.

## Ordre de test recommandé

1. **02 — Osmothèque** : pratique la plus immédiatement lisible et la moins
   schématique.
2. **04 — block ferroviaire** : meilleur test de l'incertitude honnête et de la
   suspension.
3. **01 — correction d'épreuve** : comparer si Ideogram rend vraiment les deux
   différences du pont.
4. **05 — circulation d'une édition** : vérifier que la marque à l'olivier est
   identique sur chaque exemplaire sans devenir un logo moderne.
5. **03 — assiettes historiées** : scène culturellement juste, mais la relation
   avec plusieurs curations dépendra davantage de la légende du livre.

## Rejets immédiats

Rejeter un rendu si :

- un objet n'appartenant pas à la pratique culturelle devient nécessaire pour
  comprendre la scène ;
- Ideogram ajoute des badges, écrans, flèches, classements ou textes ;
- le correcteur ressemble à un juge de vérité ;
- la mouillette contient un flacon ou une formule complète ;
- une série de faïence domine les autres comme gagnante ;
- le sémaphore s'ouvre malgré l'absence de reddition ;
- le lecteur, le colporteur ou la bibliothèque ajoutent leur propre marque
  d'auteur aux exemplaires ;
- la reconstitution esthétique masque le geste réel qui porte l'analogie.
