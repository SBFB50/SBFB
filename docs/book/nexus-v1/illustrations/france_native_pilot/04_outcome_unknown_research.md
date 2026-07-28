# 04 — `outcome_unknown` par une pratique ferroviaire native

## Verdict de recherche

La pratique la plus honnête trouvée sur le territoire français est le
**cantonnement ferroviaire en block manuel**, tel qu’il est assuré par les
gardes des postes de cantonnement.

La scène ne transforme pas une fonctionnalité informatique en machine
imaginaire. Elle montre une situation ordinaire de cette pratique :

1. un premier train a déjà été expédié dans un canton ;
2. le poste amont attend que le poste aval ait observé son arrivée complète ;
3. tant que cette reddition de voie libre n’est pas revenue, le canton reste
   traité comme occupé ;
4. le sémaphore reste fermé et le train suivant attend.

Cette pratique porte naturellement la relation essentielle de
`outcome_unknown` : **une action a quitté le poste local, la preuve attendue de
son achèvement manque, et l’opérateur suspend l’action suivante dangereuse au
lieu de conclure ou de recommencer**.

L’homologie n’est toutefois pas parfaite. Le train suivant n’est pas une
seconde invocation du même `effect_id`, et la reddition ferroviaire n’est pas
un reçu cryptographique. Cette différence est conservée explicitement dans
les limites ci-dessous ; elle ne doit pas être effacée par la légende du livre.

## Comparaison des pratiques candidates

La sélection a été faite à partir des pratiques, avant de dessiner une scène.
Quatre relations étaient recherchées :

1. une action déjà engagée et difficile ou dangereuse à répéter ;
2. une confirmation attendue après cette action ;
3. l’absence possible de cette confirmation ;
4. un opérateur qui suspend au lieu de conclure.

| Pratique française examinée | Relations natives trouvées | Écart décisif | Décision |
|---|---|---|---|
| Télégraphe optique Chappe | Un poste forme un signal, le poste suivant le reproduit, le premier conserve le dernier signal lors d’une suspension | Réexposer le même signal est précisément le mécanisme de transmission ; la répétition n’est pas un second effet dangereux | Écarté |
| Tir de mine et traitement d’un raté | Mise à feu irréversible, délai d’attente, zone maintenue interdite, reconnaissance par le boutefeu, procédure particulière en cas de raté | L’explosion ou le silence n’est pas un reçu indépendant retourné par une cible ; une image risquerait de confondre « résultat inconnu » et « explosif forcément intact » | Écarté malgré une forte proximité |
| Grand feu de la Manufacture de Sèvres | Cuisson irréversible, surveillance par montres fusibles, refroidissement prolongé avant ouverture | La source institutionnelle documente les contrôles pendant la cuisson, pas une règle de suspension née de la perte de leur retour ; la « montre absente » aurait été inventée | Écarté |
| Écluse automatisée des Voies navigables de France | Une manœuvre séquentielle dangereuse, attentes obligatoires, sortie seulement après ouverture complète | Les feux et messages donnent surtout des préconditions avant l’action ; la source ne documente pas un effet déjà parti dont le résultat demeure inconnu | Écarté |
| Cantonnement ferroviaire en block manuel | Train expédié, arrivée complète observée en aval, reddition attendue, sémaphore amont maintenu fermé, train suivant suspendu | Le suivant n’est pas la répétition littérale du même train et la reddition n’est pas cryptographique | **Retenu comme homologie la plus honnête** |

### Pourquoi le télégraphe Chappe n’a pas été retenu

Le Conservatoire numérique des arts et métiers décrit une véritable logique de
répétition poste à poste. Le stationnaire aval prépare le signal qu’il observe,
ne l’« assure » qu’après le poste transmetteur et le reproduit pour le relais
suivant. Un répétiteur intérieur conserve en outre le dernier signal pendant
une suspension :

<https://cnum.cnam.fr/pgi/redir.php?ident=8VI101&onglet=c>

Cette pratique aurait donné une image culturellement très forte : deux
stationnaires, deux machines Chappe et un signal immobilisé par le brouillard.
Mais elle aurait renversé l’invariant Nexus. Dans le système Chappe, conserver
ou reproduire le signal est la méthode normale permettant son acheminement ;
la duplication visuelle ne constitue pas un effet externe non idempotent.
L’utiliser pour expliquer « ne pas répéter » aurait été séduisant mais faux.

### Pourquoi le tir de mine n’a pas été retenu

Le règlement français des industries extractives impose, après un tir, un
délai d’attente pendant lequel la zone dangereuse reste interdite, puis une
reconnaissance du chantier par un boutefeu. Les ratés font l’objet d’un
traitement distinct :

<https://www.legifrance.gouv.fr/codes/section_lc/JORFTEXT000000520737/LEGISCTA000006169383/2008-08-31>

La source INERIS consacrée aux explosifs rassemble les prescriptions de
transport, mise en œuvre, contrôle et traitement :

<https://sstie.ineris.fr/thematiques-rgie/explosifs>

La proximité avec l’irréversibilité et la suspension est excellente. Le défaut
est épistémique : une détonation entendue, une fumée ou l’absence de bruit
n’est pas un accusé de réception séparé et fiable. De plus, une illustration
sans texte pourrait pousser le lecteur à conclure soit « l’explosion a eu
lieu », soit « la charge est intacte », alors que l’objectif est précisément de
ne conclure ni l’un ni l’autre. La scène ferroviaire possède une confirmation
aval institutionnellement distincte.

### Pourquoi le four de Sèvres n’a pas été retenu

La Manufacture de Sèvres documente une cuisson de grand feu suivie par des
montres fusibles étalonnées et un refroidissement d’au moins trois semaines
avant défournement :

<https://www.sevresciteceramique.fr/media/dp-four-a-bois-jo2024-7-compressed.pdf>

Le geste est irréversible et la prudence est culturellement native. Cependant,
les montres fournissent un contrôle pendant le processus ; la source ne décrit
pas la perte d’une confirmation après l’engagement ni l’interdiction d’une
seconde cuisson dans cet état. Inventer une montre manquante, un four opaque et
un second lot bloqué aurait reproduit l’erreur méthodologique du précédent
prompt horloger.

### Pourquoi l’écluse n’a pas été retenue

Voies navigables de France décrit une suite réelle : annonce de l’arrivée,
entrée autorisée, bassinée, attente de l’ouverture complète, puis sortie :

<https://www.vnf.fr/vnf/services/apprendre-a-franchir-une-ecluse-automatisee/>

Cette pratique montre très bien une machine qui refuse de sauter une étape.
Mais elle illustre surtout une **autorisation préalable**. Elle conviendrait
mieux au bridge typé et aux préconditions locales qu’à un effet déjà lancé
dont aucun reçu fiable n’est revenu.

## Phrase courte pour le livre

> Quand le poste aval n’a pas confirmé l’arrivée complète du premier train, le
> garde maintient le canton fermé : l’absence de confirmation ne devient ni
> une arrivée inventée, ni l’autorisation d’envoyer le suivant.

## 1. Pratique native, avant toute analogie avec Nexus

### 1.1 Le canton et les deux postes

Le block manuel découpe la ligne en cantons délimités par des postes. Un garde
manœuvre les appareils de block et les signaux depuis chaque poste. Le poste
amont annonce l’expédition d’un train au poste aval. Après le passage du train,
le sémaphore se ferme et reste bloqué.

Le poste aval ne rend pas la voie libre parce qu’une durée théorique s’est
écoulée. Son garde doit observer directement la signalisation d’arrière et
constater ainsi que le train est arrivé **complet**. Il peut alors indiquer au
poste amont que le canton est libre. C’est seulement après ce retour que le
poste amont peut rouvrir le sémaphore et expédier le train suivant.

Le document pédagogique de l’Établissement public de sécurité ferroviaire
(EPSF) formule toute la chaîne :

- les postes successifs sont reliés par téléphone et appareils de block ;
- l’appareil permet d’annoncer l’expédition du train ;
- le poste aval indique que la voie est libre après l’arrivée complète ;
- cette complétude est vérifiée par l’observation de la signalisation
  d’arrière ;
- le block interdit l’ouverture du signal tant que le canton est occupé.

Ce n’est donc pas un coffre métaphorique, un voyant inventé ou une machine à
capsules. Le garde, le téléphone, l’appareil de block, le sémaphore et le train
retenu sont les acteurs et objets réels de la pratique.

### 1.2 Ce que signifie une confirmation absente

Le poste amont sait que le train a été expédié. Il ne sait pas encore, par la
procédure de block, que le train est arrivé complet au poste aval. Plusieurs
réalités restent compatibles avec cette absence de reddition : le train est
encore en marche, il est arrêté dans le canton, il a subi une anomalie, le
poste aval n’a pas encore vérifié sa queue, ou la communication est dérangée.

Le garde ne choisit pas arbitrairement une de ces explications. Il conserve la
conséquence de sécurité commune : la section n’est pas rendue libre. Le
sémaphore fermé ne dit pas « le train a échoué » ; il dit seulement « la
condition autorisant le prochain engagement n’est pas acquise ».

Le document de SNCF Réseau sur la conception horaire décrit le même invariant :
le block manuel s’effectue de poste à poste et exige la vérification de la
complétude du train par le garde aval **avant** de rendre la voie libre au
poste amont et de permettre l’expédition du suivant.

### 1.3 Le sémaphore n’est pas une icône ajoutée

Le sémaphore est le dispositif historique propre au cantonnement. La notice
patrimoniale des sémaphores d’Oyonnax explique qu’une seule circulation est
admise dans un canton et que l’aile horizontale commande l’arrêt. Elle décrit
une aile de tôle rouge bordée de blanc, un mât, une échelle, un câble, un
contrepoids et un moteur Mors. Les appareils observés reprennent le modèle du
réseau Paris–Lyon–Méditerranée et appartiennent au patrimoine ferroviaire de
l’Ain et du Jura.

Le prompt conserve l’aile mécanique et élimine ses lampes rouge et verte, les
plaques et les inscriptions afin d’éviter qu’Ideogram transforme la scène en
interface de statut moderne. L’aile reste néanmoins un objet réel de la
pratique, pas un pictogramme surimposé.

### 1.4 Une communauté professionnelle et une mémoire graphique

La communauté mobilisée ici est celle des **gardes de block, agents de
circulation, sémaphoristes et autres cheminots chargés de l’espacement des
trains**. Elle n’est ni un peuple uniforme ni une identité folklorique. C’est
une communauté professionnelle constituée par des objets, des gestes, une
responsabilité partagée entre postes et une culture de sécurité.

Le langage visuel vient aussi de cette communauté. Les fonds SNCF conservent
des affiches de sécurité destinées aux cheminots. Une affiche de 1940 présente
un geste professionnel dangereux sous la forme d’une scène peinte claire et
directe. Le prompt reprend cette grammaire — gouache aplatie, contours fermes,
action centrale, environnement de travail reconnaissable — mais retire tout
texte. Il ne copie ni l’affiche ni son accident ; il emploie la forme
documentaire créée pour transmettre des règles dans le monde ferroviaire.

## 2. Sources institutionnelles et références visuelles

### Règle d’exploitation

1. **EPSF — Les signaux, les régimes d’exploitation des lignes, les systèmes
   d’espacement des trains**, pages 29 à 31 :  
   <https://www.securite-ferroviaire.fr/sites/default/files/reglementations/pdf/2023-03/document-pedagogique-signaux-regimes-exploitation-v1.pdf>

   Passages déterminants :

   - page 29 : le garde aval vérifie que le train est arrivé entier grâce à sa
     signalisation d’arrière, puis avise le garde amont que la voie est libre ;
   - page 30 : les appareils relient les postes, annoncent l’expédition et
     indiquent la libération après arrivée complète ;
   - page 31 : les liaisons électriques interdisent l’ouverture du sémaphore
     tant que le canton est occupé.

2. **SNCF Réseau — Règles de mise en compatibilité des sillons en conception
   horaire**, section 6.2.2, page 18 :  
   <https://www.sncf-reseau.com/medias-publics/2023-12/ar02384-regles_mise_compatibilite_sillons.pdf>

   La section précise que le block manuel est effectué de poste à poste et
   exige une vérification de la complétude du train à l’arrivée avant la
   reddition de voie libre et l’expédition du train suivant.

### Objets et implantation

3. **POP, Plateforme ouverte du patrimoine — deux sémaphores mécaniques
   d’Oyonnax** :  
   <https://pop.culture.gouv.fr/notice/palissy/IM01000937>

   La notice localise les deux sémaphores aux extrémités de la gare, décrit
   leur aile, leur câble, leur contrepoids et leur position d’arrêt, puis
   rattache le type unifié SNCF au modèle PLM.

4. **Photographie institutionnelle directe — sémaphore côté
   Andelot-en-Montagne** :  
   <https://pop-perf-assets.s3.gra.io.cloud.ovh.net/memoire/IVR43_20070100001NUCA/ivr43_20070100001nuca.jpg>

   Cette image sert seulement à contrôler la silhouette, l’échelle, le mât,
   le contrepoids et le rapport entre le signal et la voie. Le prompt retire
   les inscriptions et la signalisation lumineuse visibles sur les appareils
   patrimoniaux plus récents.

5. **Conservatoire numérique des arts et métiers — catalogue des transports
   sur rails**, entrée sur l’électro-sémaphore Lartigue de 1874 :  
   <https://cnum.cnam.fr/pgi/redir.php?ident=M6774&onglet=c>

   L’entrée décrit le passage d’un train dans un canton, la mise à l’arrêt de
   la grande aile, l’annonce électrique au sémaphore suivant et le déblocage
   renvoyé au premier poste après le franchissement du second.

### Langage graphique de la communauté ferroviaire

6. **Archives SNCF — affiche de sécurité au travail de 1940** :  
   <https://www.archives.sncf.com/pendant-le-travail-ne-soyez-pas-distrait-ne-laissez-pas-le-protecteur-leve/>

   La référence justifie le choix d’une illustration de sécurité narrative,
   centrée sur un agent et son geste. Elle ne justifie pas de copier ses
   personnages, sa machine ou son texte.

   Image numérisée en haute définition :  
   <https://www.archives.sncf.com/wp-content/uploads/mediatheque/tr_sardo_1672.jpg>

7. **Archives SNCF — description des fonds historiques** :  
   <https://www.archives.sncf.com/que-sont-nos-fonds/>

   Les fonds regroupent textes réglementaires, plans de matériel et de gares,
   photographies, lithographies, revues internes, objets et affiches retraçant
   les métiers et la culture visuelle des cheminots.

8. **Archives SNCF — fiches documentaires**, présence de dossiers « Block
   manuel » et « Block Lartigue » :  
   <https://www.archives.sncf.com/fiches-documentaires-5/>

### Référence secondaire pour explorer les images

9. **Wikimedia Commons — signaux ferroviaires en France** :  
   <https://commons.wikimedia.org/wiki/Category:Railway_signals_in_France>

   Cette galerie n’établit aucune règle d’exploitation. Elle sert uniquement
   à comparer les silhouettes régionales et à repérer les anachronismes
   possibles avant validation par les sources institutionnelles.

## 3. Plausibilité de chaque geste et de chaque objet

| Élément du prompt | Statut dans la pratique | Preuve ou prudence |
|---|---|---|
| Ligne divisée en canton entre deux postes | Attesté | EPSF, pages 29–31 |
| Premier train déjà expédié | Attesté comme événement annoncé au poste aval | EPSF, page 30 |
| Téléphone entre postes | Attesté | EPSF, pages 29–30 |
| Garde qui attend la reddition | Attesté par la séquence opérationnelle | EPSF et SNCF Réseau |
| Vérification de l’arrivée complète | Attestée par observation de la signalisation d’arrière | EPSF, page 30 |
| Sémaphore maintenu fermé | Attesté ; le système interdit sa réouverture tant que le canton est occupé | EPSF, page 31 |
| Train suivant retenu | Conséquence directe attestée | SNCF Réseau, section 6.2.2 |
| Appareil de block en fonte, cloche et levier | Famille d’objets attestée ; forme volontairement générique | EPSF illustre plusieurs appareils, CNAM documente Lartigue |
| Garde écoutant le téléphone | Geste plausible en cantonnement téléphonique ou situation dégradée | EPSF indique le téléphone et le retour au cantonnement téléphonique |
| Brouillard cachant le poste aval | Condition narrative plausible, non nécessaire à la règle | Il matérialise seulement l’impossibilité de voir le résultat à distance |
| Reste de fumée du train parti | Détail narratif plausible pour une traction vapeur | Il n’est pas présenté comme une preuve réglementaire |
| Jura vers 1940 | Reconstruction éditoriale cohérente avec le patrimoine PLM et l’affiche SNCF | Ce n’est pas la reconstitution d’une gare et d’une date précises |

### Détails volontairement exclus

- aucun jeton, capsule, tuyau de retour ou armoire opaque ;
- aucun voyant rouge ou vert ;
- aucun écran, badge, statut ou symbole « succès/échec » ;
- aucune flèche montrant le sens du train ;
- aucun registre lisible ou formulaire pseudo-historique ;
- aucune horloge servant artificiellement de délai ;
- aucun deuxième opérateur présenté comme autorité distante omnisciente ;
- aucune barrière inventée en travers de la voie.

Le seul arrêt visible est celui produit par l’objet culturel réel : le
sémaphore mécanique.

## 4. Mapping séparé vers Nexus

La scène ferroviaire doit d’abord rester autonome. Le mapping ci-dessous
appartient à la légende et au dossier de conception, pas à l’image.

| Pratique ferroviaire | Invariant Nexus |
|---|---|
| Le poste amont a expédié le premier train | L’intention a été enregistrée et la tentative a commencé |
| Le train disparaît dans un canton que le garde amont ne peut inspecter | L’effet externe peut avoir eu lieu hors de la frontière locale |
| Le poste aval doit constater l’arrivée complète | Une preuve liée à l’effet est nécessaire pour conclure |
| La reddition de voie libre ne revient pas | Aucun reçu fiable n’est disponible |
| Le canton reste traité comme occupé | L’incertitude est conservée explicitement |
| Le sémaphore demeure fermé | Le moteur ne relance pas automatiquement l’effet non idempotent |
| Le train suivant attend | Une opération potentiellement duplicative ou dangereuse reste suspendue |
| Une procédure ferroviaire distincte traite le dérangement | Nexus peut parquer pour réconciliation ou fermer en `outcome_unknown` |

### Ce que la métaphore ne doit pas prétendre

1. **Le train suivant n’est pas le même train.**  
   Dans Nexus, le risque central est de rappeler la même cible pour le même
   effet non idempotent. Dans le block, l’action interdite est l’engagement
   d’une autre circulation dans la même ressource exclusive. L’homologie porte
   sur la suspension conservatrice, pas sur l’identité de la commande.

2. **La voie est une ressource exclusive physique.**  
   Nexus traite aussi des effets sans ressource spatiale partagée : paiement,
   envoi, publication, commande distante. Le canton explique le risque de
   double action, mais il ne modélise pas tous les effets.

3. **La reddition n’est pas un reçu cryptographique.**  
   Elle repose sur une procédure, une observation humaine et des appareils de
   block. Un reçu Nexus est structuré et lié à un `effect_id`. L’image ne doit
   donc pas suggérer que tout appel téléphonique serait une preuve suffisante.

4. **Le block adopte un état opérationnel conservateur, pas une connaissance
   du monde.**  
   Le garde n’affirme pas que le premier train est encore dans le canton ; il
   refuse seulement d’établir la voie libre sans la condition requise. C’est
   précisément la partie la plus féconde de l’analogie.

## 5. Ancrage dans le manuscrit Nexus V1

- [`07_workflow_hub.md`](../../07_workflow_hub.md), lignes 206–210 : après un
  crash, la reprise se fait à une frontière sûre ; si un effet externe a
  peut-être eu lieu sans reçu, le run entre dans `outcome_unknown`.
- [`09_reseau_apps_compute.md`](../../09_reseau_apps_compute.md), lignes
  84–88 : le runtime conserve inbox, outbox et journal, applique les politiques
  d’idempotence et refuse d’inventer le résultat d’un effet non idempotent.
- [`10_confiance_et_capacites.md`](../../10_confiance_et_capacites.md), lignes
  155–169 : une requête peut avoir quitté la machine avant la rupture de la
  connexion ; la répéter peut doubler l’effet, tandis que déclarer l’échec peut
  être faux.
- [`03_une_journee_dans_nexus.md`](../../03_une_journee_dans_nexus.md), lignes
  112–116 : une tâche sans reçu n’est pas renvoyée aveuglément ; une décision
  ultérieure est requise.

Le manuscrit décrit ici un contrat V1. Le dossier `docs/book/nexus-v1/` est
non suivi dans l’état du worktree observé ; son contenu ne constitue donc pas
à lui seul une preuve de livraison.

## 6. Code live et comportement effectivement testé

État inspecté : `HEAD 9134af64`, le 27 juillet 2026.

### Reprise après une tentative non confirmée

- [`engine.rs`](../../../../../crates/sbfb-workflow-engine/src/engine.rs),
  lignes 377–443 : à la reprise, le moteur retrouve un effet pending. Il
  vérifie le hash, la déclaration d’idempotence journalisée et le scope de
  déduplication. Si la cible n’était pas idempotente, il enregistre
  `TerminalReason::OutcomeUnknown` et retourne sans appliquer la cible.

- [`store_spike.rs`](../../../../../crates/sbfb-workflow-engine/tests/store_spike.rs),
  lignes 544–594 : le test interrompt le processus après l’effet externe mais
  avant le résultat. Après reprise, la sortie externe reste unique et le run
  devient terminal `outcome_unknown`; le test interdit explicitement le retry.

### Expiration sans reçu

- [`orchestrator.rs`](../../../../../crates/sbfb-workflow-engine/src/orchestrator.rs),
  lignes 1464–1500 : lorsque la deadline est franchie, une cible idempotente
  peut suivre la politique de retry ; une cible non idempotente passe par
  `unknowable_outcome`.

- Le même fichier, lignes 1502–1538 : l’issue inconnue est soit parquée pour
  réconciliation, soit fermée terminalement, « never an automatic retry,
  never an invented success ».

- [`effects.rs`](../../../../../crates/sbfb-workflow-engine/tests/effects.rs),
  lignes 334–392 : `non_idempotent_without_receipt_becomes_unknown` vérifie un
  unique démarrage, aucun événement de tentative échouée et la raison
  terminale `outcome_unknown`.

### Réconciliation sans blanchiment de l’incertitude

- [`effects.rs`](../../../../../crates/sbfb-workflow-engine/tests/effects.rs),
  lignes 1106–1208 : un run configuré pour la réconciliation peut rester
  parqué ; un résultat récupéré doit fournir le reçu réel avant la poursuite.

- Le même fichier, lignes 1228–1323 : un abandon produit un état terminal
  absorbant qu’un reçu tardif ne transforme pas après coup en succès.

- Le même fichier, lignes 1326–1365 :
  `reconcile_recovered_requires_receipt` refuse une récupération sans preuve.

### Limite live

Le runtime ne crée pas une garantie « exactement une fois » contre une cible
qui ne déduplique pas durablement. Sa garantie réelle est plus modeste et plus
importante : il **ne réapplique pas automatiquement** une opération
non-idempotente dont l’effet reste non confirmé.

## 7. Historique Git

Trois commits structurent la fonctionnalité observée :

- `6de49b12` — 20 juillet 2026 —  
  `feat(workflow-engine): Sprint 83 Phase E — spike durable bloquant
  (journal/outbox atomiques, echo-target)`  
  Introduit la frontière crash après effet/avant résultat et le test
  discriminant du non-retry.

- `a23510e8` — 23 juillet 2026 —  
  `feat(workflow-engine): Sprint 83 Phase J — orchestrateur local
  déterministe`  
  Transporte l’état `outcome_unknown` dans le parcours du graphe et son
  reducer.

- `90921211` — 24 juillet 2026 —  
  `feat(workflow-engine): Sprint 83 Phase K — effets, retries et idempotence`  
  Ajoute deadlines, retries bornés, distinction idempotente/non idempotente et
  réconciliation fondée sur un reçu.

Ces commits prouvent l’évolution du moteur live. Ils ne prouvent pas que le
livre ou cette illustration ont déjà été publiés.

## 8. Construction de l’image

### Un seul instant culturellement plausible

L’image se situe au poste amont, après le départ d’un premier train et avant la
reddition de voie libre :

- la courbe et le brouillard cachent le poste aval ;
- un reste de fumée donne une trace du départ sans montrer son résultat ;
- le garde écoute le téléphone et observe la ligne ;
- sa main libre reste éloignée du levier ;
- le sémaphore mécanique demeure horizontal ;
- la locomotive suivante attend avant le signal.

La scène n’essaie pas de montrer simultanément les deux gares, l’historique
complet, le train arrivé et le train encore en ligne. Ce serait redevenir un
diagramme. Le hors-champ est constitutif de la pratique et de l’incertitude.

### Pourquoi cinq objets

1. **Le garde** porte l’attente et la suspension humaine.
2. **Le téléphone** matérialise le canal réel par lequel la reddition peut
   revenir.
3. **L’appareil de block** ancre la procédure dans le poste.
4. **Le sémaphore** matérialise la fermeture conservatrice.
5. **La locomotive suivante** rend visible l’action qui n’est pas autorisée.

Le premier train et le poste aval restent hors champ. Les montrer donnerait au
lecteur une information que le garde ne possède pas et surchargerait le
modèle.

## 9. Risques de génération

1. **Ideogram montre le premier train arrêté dans le brouillard.**  
   Cela transforme l’incertitude en panne visible. Rejeter si une locomotive
   apparaît au-delà du sémaphore.

2. **Le téléphone produit une lettre, une bande papier ou une bulle.**  
   Rejeter toute forme de reçu visible, texte, parole ou message.

3. **Le garde actionne le levier.**  
   Son geste doit rester suspendu : une main tient l’écouteur, l’autre demeure
   retirée.

4. **Le sémaphore devient un feu tricolore.**  
   Rejeter les têtes lumineuses, feux rouge/vert et panneaux modernes. L’aile
   mécanique horizontale suffit.

5. **La locomotive franchit le signal.**  
   Elle doit rester plusieurs mètres en amont, roues immobiles et freins
   appliqués.

6. **Le modèle invente un cadenas ou une barrière sur la voie.**  
   Aucun obstacle physique supplémentaire n’est nécessaire. Le signal et la
   discipline d’exploitation portent la règle.

7. **L’appareil ressemble à une interface moderne.**  
   Rejeter écran, jauge colorée, statut, clavier, numéro lisible ou voyant.

8. **Le brouillard devient une métaphore fantastique.**  
   Il doit rester une météo ordinaire de vallée, sans silhouette, visage,
   symbole ou lumière surnaturelle.

9. **La scène devient une affiche touristique nostalgique.**  
   Le garde au téléphone, le levier retenu, le sémaphore fermé et la seconde
   locomotive doivent rester les points dominants.

10. **Faux texte ferroviaire.**  
    Rejeter numéros de locomotive, initiales de compagnie, plaque de gare,
    horaires, affiche murale, cadran, code ou pseudo-inscription.

## 10. Test aveugle

Présenter l’image seule à cinq personnes, sans titre, légende, mention de
Nexus ni explication du block.

### Questions ouvertes

1. « Que s’est-il passé juste avant cet instant ? »
2. « Qu’attend l’homme au téléphone ? »
3. « Pourquoi la locomotive ne repart-elle pas ? »
4. « L’image permet-elle de savoir si le premier train est arrivé ? »
5. « Que ferait selon vous le garde si aucune nouvelle n’arrivait ? »

Ne pas demander « voyez-vous un reçu manquant ? » ou « comprend-on
`outcome_unknown` ? » : ces formulations fourniraient la réponse.

### Seuil de réussite

- au moins quatre personnes sur cinq comprennent qu’un premier train est déjà
  parti ;
- au moins quatre identifient une confirmation ou nouvelle attendue ;
- au moins quatre comprennent que la locomotive suivante est volontairement
  retenue ;
- aucune majorité ne conclut que le premier train a sûrement réussi son trajet
  ou sûrement échoué ;
- au moins trois formulent spontanément une idée proche de « on ne sait pas,
  donc on attend avant d’envoyer le suivant ».

### Échec et décision

Le test échoue si la majorité voit seulement :

- un chef de gare qui téléphone pour annoncer un retard ;
- une locomotive ordinairement arrêtée en gare ;
- un appareil cassé en réparation ;
- un accident déjà certain dans le brouillard ;
- un feu dont l’homme connaît secrètement l’état.

Dans ce cas, ne pas ajouter d’icône ou de flèche. Réviser d’abord la pratique
visible : rapprocher le sémaphore de la locomotive, dégager la main retirée du
levier, renforcer le reste de fumée au-delà de la courbe et cadrer le téléphone
avec le visage du garde.

## Conclusion

Cette illustration ne prétend pas que le block manuel est un protocole
informatique avant l’heure. Elle montre une discipline française réelle où
l’absence d’un retour attendu conserve une section fermée et suspend une
nouvelle action risquée.

Son apport pédagogique tient à une règle simple, entièrement portée par la
pratique :

> **ne pas avoir reçu la preuve de libération n’autorise pas à agir comme si
> elle avait été reçue.**
