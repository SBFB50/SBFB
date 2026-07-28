# Fonction 7 — Projection publique minimale

## Phrase courte du livre

> Les données privées restent locales ; seule une projection volontairement
> limitée est partagée ou publiée.

La scène ne représente ni le chiffrement ni une interdiction générale de
partager. Elle représente une décision positive et bornée : l'ensemble riche
reste dans l'atelier, tandis qu'une personne produit et expose une seconde
pièce qui ne reprend qu'un motif nécessaire.

## Tradition visuelle retenue

**La tapisserie d'Aubusson, pratiquée à Aubusson, Felletin et dans le sud de la
Creuse.**

Ce choix désigne un savoir-faire vivant, pas une iconographie communautaire
figée. L'UNESCO décrit une image tissée manuellement, réalisée sur un métier de
basse lisse à partir d'un carton, avec des fils teints. La Cité internationale
de la tapisserie documente aujourd'hui la formation des lissières et lissiers,
les ateliers du territoire, la recherche et la création contemporaine.

Sources vérifiables :

- UNESCO, inscription de la tapisserie d'Aubusson au patrimoine culturel
  immatériel en 2009 :
  <https://ich.unesco.org/fr/RL/la-tapisserie-d-aubusson-00250>
- Cité internationale de la tapisserie, tissage manuel, travail sur l'envers,
  rôle du carton et relation entre artiste et lissier :
  <https://www.cite-tapisserie.fr/les-savoir-faire/les-savoir-faire-de-la-tapisserie-d-aubusson/tissage-et-finitions>
- Cité internationale de la tapisserie, formation qualifiante contemporaine :
  <https://www.cite-tapisserie.fr/les-missions/transmettre-les-savoir-faire>
- Cité internationale de la tapisserie, annuaire des ateliers et communauté
  textile actuelle d'Aubusson-Felletin :
  <https://www.cite-tapisserie.fr/les-savoir-faire/l-ecosysteme-art-tisse>

Le prompt ne copie aucune tapisserie historique ou contemporaine déterminée,
aucune signature d'artiste et aucun motif présenté comme sacré. Les branches,
oiseaux, outils et récipients sont des formes ordinaires inventées pour
l'expérience. La lissière porte des vêtements de travail actuels, pas un
costume folklorique. La palette de laine et les formes imbriquées servent à
évoquer le médium ; elles ne prétendent pas résumer six siècles de création
aubussonnaise.

## Ancrage dans le livre

- `../../09_reseau_apps_compute.md:51-62` sépare données privées locales,
  espaces partagés bornés, artefacts publiés et projections publiques. Le
  chapitre précise qu'une projection peut exposer un schéma ou une statistique
  expurgée sans publier les lignes sources.
- `../../05_rrv.md:116-135` place les secrets, messages privés, identités,
  brouillons et historiques hors index par défaut. RRV consomme des projections
  publiques ou des espaces expressément ouverts.
- `../../13_ce_que_nexus_refuse.md:126-147` refuse la diffusion publique des
  données sensibles et préfère une preuve minimisée lorsque la donnée complète
  n'est pas nécessaire.
- `../../91_carte_de_la_v1.md:49-59` rappelle qu'une donnée ne passe jamais
  automatiquement du privé au partagé puis au public.

Ces documents appartiennent actuellement au manuscrit non suivi par Git du
worktree. Ils décrivent la V1 visée et ne constituent donc pas, seuls, une
preuve que le parcours public complet est déjà livré.

## Preuves dans le code et limites d'implémentation

Le dépôt possède déjà des précédents mécaniques solides de projection par
liste blanche :

- `../../../../../crates/nexus-shell-daemon/src/shard_session_http_api.rs:24-30`
  définit la forme du DTO comme frontière de confidentialité : seuls les
  agrégats sortent, jamais les identités des workers ou de l'initiateur.
- Le test
  `shard_session_projection_hides_member_identities` dans ce même fichier
  vérifie qu'un groupe privé de deux workers devient un simple
  `member_count`, sans sérialiser leurs clés publiques.
- `../../../../../crates/nexus-shell-daemon/src/workflow_api.rs:27-31`
  limite les réponses locales à des progrès agrégés et exclut événements bruts,
  snapshots, reçus et hashes de requête.
- Le test `run_response_key_set_is_the_allowlist` verrouille l'ensemble exact
  des champs sérialisés de cette vue locale.

La limite importante est explicite :

- `../../../../../crates/sbfb-workflow-core/src/contracts.rs:96-103`
  réserve `WorkflowPublicProjection.v1`, mais le type est encore un **stub
  vide** dont les champs doivent être écrits dans une phase ultérieure de
  publication/RRV.
- La projection de `workflow_api.rs` est authentifiée et locale au daemon ;
  elle ne constitue pas encore le parcours volontaire de publication publique
  illustré ici.

Le statut honnête de la fonctionnalité est donc : **principe V1 documenté,
précédents d'allowlist implémentés et testés, contrat public réservé, publication
publique volontaire de bout en bout encore à construire.**

## Ce que montre l'historique Git

- `66259c63` — `feat(web): Sprint 77 Phase J — shard session panel + hermetic
  E2E` — introduit une projection agrégée de session privée : le compteur des
  membres est visible, les identités ne le sont pas.
- `f4b257c0` — `feat(workflow-core): Sprint 83 Phase A — cadrage, matrice de
  gates et grammaire de phase` — réserve
  `WorkflowPublicProjection.v1` sous forme de contrat vide, sans prétendre
  livrer la publication.
- `03c14d84` — `feat(daemon): Sprint 83 Phase I1 — run store loopback read
  routes` — livre la vue locale agrégée des runs et documente explicitement que
  la projection publique de workflow appartient à une phase ultérieure.

L'historique confirme donc une progression vers des vues minimisées, mais il
interdit de présenter l'image comme la photographie d'une fonctionnalité
publique déjà complète.

## Construction logique de l'image

1. **La grande tapisserie intérieure** représente l'ensemble source : nombreux
   motifs, relations et couleurs, tous conservés dans l'atelier.
2. **La cloison pleine** représente une frontière locale ordinaire. Il n'y a ni
   barreau, cage, cadenas, gardien, caméra ni métaphore punitive.
3. **La lissière** accomplit le geste de sélection. La projection n'apparaît pas
   spontanément et n'est pas choisie par un réseau abstrait.
4. **Le petit carré séparé** reprend exactement un seul rameau tricolore de la
   grande pièce. Il est une nouvelle représentation limitée, pas un morceau
   arraché à la source.
5. **Le cadre orienté vers l'extérieur** matérialise la publication.
6. **La personne dans l'espace public** ne voit que le petit carré ; son regard
   ne traverse pas la cloison vers la source.

La répétition contrôlée du rameau est le seul lien visuel entre source et
projection. Elle remplace les flèches, les légendes et les écrans remplis de
texte.

## Risques de génération à contrôler

- Ideogram peut ouvrir la cloison et rendre la grande tapisserie visible depuis
  l'extérieur, ce qui détruirait la frontière.
- Il peut découper physiquement le rameau dans la grande œuvre. Le petit carré
  doit rester une pièce séparée et la grande tapisserie doit demeurer intacte.
- Il peut dupliquer toute la tapisserie dans le cadre public au lieu de réduire
  son contenu à un seul rameau.
- Il peut transformer l'atelier en cachot, coffre-fort ou scène de surveillance
  malgré les exclusions. Toute présence de barreaux, chaînes, cadenas ou
  gardes invalide le rendu.
- Il peut produire un tapis oriental générique, une enluminure médiévale ou un
  costume régional fantasmé. Le rendu recherché est une image textile
  contemporaine, avec une artisane actuelle.
- La proximité matérielle avec le pilote canut doit rester limitée au textile :
  les Canuts illustraient le transport entre métiers ; ici aucune navette ni
  circulation inter-nœuds n'apparaît, seulement une source et sa vue réduite.

## Test aveugle

Présenter l'image seule à cinq personnes, sans le titre, la phrase du livre ni
le mot « projection ». Poser séparément :

1. Où se trouve l'ensemble contenant le plus d'informations ?
2. Que peut réellement voir la personne située dehors ?
3. La grande œuvre a-t-elle quitté l'atelier ou été découpée ?
4. Qui a choisi ce qui devient visible ?
5. Le petit carré est-il l'œuvre entière ou une représentation limitée ?

Le rendu passe si :

- au moins quatre personnes sur cinq disent que la grande œuvre reste dans
  l'atelier ;
- au moins quatre identifient que le carré extérieur ne contient qu'un motif
  correspondant ;
- au moins trois attribuent la sélection à la lissière, et non au visiteur, au
  cadre ou à un mécanisme automatique ;
- aucune ne décrit une prison, une confiscation, une censure ou la destruction
  de l'original ;
- aucune n'a besoin d'imaginer une flèche ou un texte absent pour expliquer la
  relation.

Si les personnes comprennent seulement « une artisane expose une miniature »,
l'esthétique fonctionne mais l'explication Nexus échoue : il faudra accentuer
la différence quantitative entre la source riche et le carré minimal, sans
ajouter de symbole abstrait.
