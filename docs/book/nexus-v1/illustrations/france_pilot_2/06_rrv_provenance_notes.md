# 06 — RRV : retrouver une source sans fabriquer la vérité

## Phrase courte du livre

> Retrouver ou lire une source ne la rend pas vraie : RRV conserve son origine
> et son niveau de preuve, sans élever automatiquement son affirmation au
> statut vérifié.

## Fonction précise

Cette illustration cible une seule séparation du chapitre RRV :

- une source peut être **retrouvée** et **lue** ;
- sa provenance peut être conservée ;
- un autre objet peut apporter un premier élément de corroboration ;
- le statut **vérifié** reste néanmoins distinct tant que la procédure de
  preuve attendue n'est pas satisfaite.

L'image ne prétend pas que trois sceaux constituent la règle de vérification de
Nexus. Ils sont une traduction matérielle de niveaux de preuve distincts, pas
une spécification du protocole.

## Ancrage dans le livre

- [`05_rrv.md`](../../05_rrv.md), lignes 14–16 : RRV conserve la nature des
  preuves pendant la recherche et l'explication.
- [`05_rrv.md`](../../05_rrv.md), lignes 38–40 : une synthèse conserve
  l'origine de chaque partie ; une source Web citée à côté d'un hash ne devient
  pas une preuve protocolaire.
- [`05_rrv.md`](../../05_rrv.md), lignes 42–64 : le livre sépare explicitement
  **lu**, **déduit**, **vérifié** et **non vérifié** ; lire prouve la présence
  d'un énoncé, pas sa justesse.
- [`05_rrv.md`](../../05_rrv.md), lignes 149–151 : RRV restitue et explique un
  verdict produit ailleurs ; il ne s'accorde pas le droit de le prononcer.
- [`05_rrv.md`](../../05_rrv.md), lignes 192–204 : source officielle,
  répétition collective et signature peuvent rester erronées ou insuffisantes ;
  RRV préserve les portées et les niveaux de preuve.

## Preuves dans le dépôt live

Le code présent soutient une partie de cette direction, mais pas encore toute
la sémantique d'affirmations décrite par le livre.

- [`proof_card.rs`](../../../../../crates/nexus-coordinator-rs/src/proof_card.rs),
  lignes 2–8 : la `ProofCard` est un calcul local de complétude des preuves,
  non un format signé ni une vérité protocolaire.
- Le même fichier, lignes 20–32 et 39–115, conserve séparément provenance,
  dépôt, commit, hashes, licence, fraîcheur, risques et curations au lieu de les
  réduire à un unique verdict.
- Le même fichier, lignes 145–218, garde des facteurs explicites
  `no_provenance`, `unverified_deploy`, `stale_source` et `old_release`, puis
  calcule un score versionné. Ce score mesure une complétude documentaire ; il
  ne vérifie pas la vérité générale d'une affirmation.
- [`browse.rs`](../../../../../crates/nexus-shell-daemon-core/src/browse.rs),
  lignes 113–127, distingue la provenance de découverte `Curator`, `Direct` et
  `NodeDirectory`; le commentaire précise que l'ancre d'hébergement ne devient
  jamais l'autorité sur la provenance de l'auteur.
- Le même fichier, lignes 232–255, expose séparément hash d'archive, URL de
  source, hash de provenance et déclaration open source.
- [`apps.rs`](../../../../../crates/nexus-shell-daemon/src/apps.rs), lignes
  20–55 et 78–108, projette séparément `source`, `status`,
  `has_provenance`/`provenance_hash`, curateur et date du dernier probe.

### Limite honnête

Je n'ai pas trouvé dans le code live un moteur RRV général implémentant déjà
les quatre statuts d'affirmation du manuscrit — **lu**, **déduit**,
**vérifié**, **non vérifié** — sur toutes les sources. Le dépôt possède des
briques réelles de découverte, provenance, Proof Card et qualification
d'applications. L'illustration représente donc le contrat éditorial de la V1
décrite, appuyé par ces briques, et non une preuve que le produit complet est
déjà livré.

## Historique Git vérifié

- `328ef157` — `feat(p2p): Sprint 14 Phase B — ProjectAnnouncement v4 with
  provenance hash` : introduction du hash de provenance dans l'annonce et le
  browse.
- `f9d722e6` — `feat(proof-card): Sprint 68 Phase A — ProofCard computation +
  daemon endpoint` : ajout du calcul local de complétude et de ses facteurs
  séparés.
- `f6637d38` et `821aa8c2` — ajout puis ingestion durable du
  `NodeDirectoryEntry` : l'hébergement sert à la découverte sans réattribuer
  l'autorité de provenance.

Le répertoire `docs/book/nexus-v1/` est actuellement non suivi par Git dans le
worktree observé. Il ne possède donc pas encore d'historique Git propre à
citer ; cette absence est signalée plutôt que remplacée par une fausse preuve.

## Tradition visuelle retenue

### Imprimeurs-libraires et tailleurs d'histoires de la Renaissance lyonnaise

Le choix porte sur le milieu historique des imprimeurs-libraires, graveurs sur
bois et illustrateurs actifs à Lyon entre la fin du XVe et le XVIe siècle. Ce
milieu professionnel a disparu sous cette forme sociale et technique, tandis
que ses livres, matrices et méthodes d'impression subsistent dans les
collections.

Sources institutionnelles et collections :

- Musée de l'Imprimerie et de la Communication graphique de Lyon, présentation
  du musée et rôle majeur de Lyon dans l'innovation et la diffusion imprimée :
  <https://www.imprimerie.lyon.fr/fr/edito/presentation_musee>
- Parcours du MICG : imprimeurs-libraires de la Renaissance, collaboration de
  Jean de Tournes avec l'illustrateur Bernard Salomon, et place de la gravure
  sur bois :
  <https://www.imprimerie.lyon.fr/fr/edito/visite-guidee>
- Collection du MICG de 200 bois gravés anciens et 13 bois dessinés employés
  pour une Bible lyonnaise du XVIe siècle :
  <https://noscollections.imprimerie.lyon.fr/index.php/Detail/objects/4065>
- Bibliothèque municipale de Lyon : les vignettes lyonnaises gravées sur bois
  circulaient comme réservoir de scènes et de motifs auprès d'autres artisans :
  <https://collections.bm-lyon.fr/presseXIX/PER0044ae180811d905>
- MICG, conférence sur Barthélémy Buyer et Guillaume Le Roy : premiers ateliers
  lyonnais des années 1470 et constitution d'un centre européen majeur :
  <https://www.imprimerie.lyon.fr/en/node/503>

Le prompt ne copie aucune planche historique précise, aucun monogramme réel et
aucun sujet religieux. Il reprend une grammaire générale : tailles noires,
figures compactes, espace frontal, palette manuelle limitée et objets
d'imprimerie matériellement identifiables.

## Logique visuelle

La scène est un atelier unique, pas un diagramme :

1. **Origine conservée** — le sceau bleu du folio reprend la rosette de la
   presse vide située en amont.
2. **Lecture visible** — le folio est ouvert dans un simple berceau de
   consultation.
3. **Examen distinct** — l'archiviste compare le folio à une estampe produite
   par une autre technique et portant un autre sceau.
4. **Preuve encore incomplète** — un seul témoin indépendant est présent.
5. **Statut non promu** — le folio consulté demeure physiquement hors de
   l'armoire verrouillée.
6. **État vérifié représenté sans badge** — un autre dossier déjà placé dans
   l'armoire porte plusieurs sceaux d'origines différentes.

Le pont effondré donne aux deux documents un contenu concret à comparer. Il
évite les lignes de code, badges, jauges et pictogrammes abstraits.

## Risques de génération

- Ideogram peut transformer les cordons ou le regard en flèches : rejeter le
  rendu si une direction graphique apparaît.
- Les pages peuvent recevoir du faux texte : rejeter toute inscription,
  lettre, chiffre ou pseudo-latin.
- Les deux vues du pont peuvent devenir deux scènes sans rapport : elles
  doivent conserver le même pont et la même arche centrale effondrée.
- Le folio bleu peut être dupliqué dans l'armoire : il doit rester uniquement
  dans le berceau ouvert.
- L'armoire peut devenir un autel ou un reliquaire religieux : conserver une
  armoire d'archives séculière en fer et bois.
- Trois sceaux peuvent être lus comme un vote majoritaire. La légende du livre
  doit rappeler qu'ils représentent plusieurs preuves indépendantes, pas une
  règle de majorité ni un seuil normatif de Nexus.
- La gravure peut devenir une imitation exacte de Bernard Salomon : conserver
  la référence au milieu collectif lyonnais et refuser toute copie d'une
  vignette identifiable.

## Test aveugle

Masquer la phrase et demander séparément à au moins trois personnes :

1. « D'où vient le livre ouvert ? »
2. « Que fait la personne au centre ? »
3. « Le livre ouvert est-il déjà rangé avec les dossiers les plus établis ? »
4. « Qu'est-ce qui distingue le livre consulté du dossier enfermé ? »
5. « Le fait qu'il soit lisible suffit-il pour ouvrir l'armoire ? »

Le rendu passe si au moins deux personnes sur trois comprennent simultanément :

- que l'origine reste attachée au document ;
- que lire et examiner ne sont pas la même chose que valider ;
- que le document ouvert n'a pas automatiquement reçu le statut du dossier
  enfermé.

Échec si le public ne voit qu'une scène générique de bibliothèque, interprète
l'armoire comme une censure, ou pense que l'archiviste cache volontairement une
source déjà prouvée.
