# 09 — `outcome_unknown` : ne pas répéter un effet incertain

## Phrase courte pour le livre

> Si un effet non idempotent a peut-être eu lieu mais qu’aucun reçu fiable
> n’est revenu, le runtime déclare le résultat inconnu et bloque toute nouvelle
> tentative automatique.

## Fonctionnalité Nexus représentée

L’image vise une frontière très précise du runtime :

1. l’intention a été enregistrée ;
2. la première tentative est partie vers une cible externe ;
3. l’effet a peut-être été appliqué ;
4. le résultat n’a pas été journalisé sous la forme d’un reçu fiable ;
5. la cible ne garantit pas qu’une répétition serait sans effet supplémentaire ;
6. le runtime ne relance donc pas automatiquement l’opération et ne la déclare
   ni réussie ni échouée.

Le mot `outcome_unknown` ne signifie pas « erreur inconnue » au sens vague. Il
nomme une situation épistémique bornée : le moteur sait que la tentative a
commencé, mais ne possède pas la preuve qui lui permettrait de conclure sur son
effet externe.

Le code prévoit ensuite deux postures configurées par l’hôte : fermer le run
dans cet état terminal ou parquer le nœud afin qu’un opérateur local applique
une procédure de réconciliation. Dans les deux cas, aucune seconde exécution
automatique n’est autorisée.

## Ancrage dans le livre

- [`07_workflow_hub.md`](../../07_workflow_hub.md), lignes 206–210 : après un
  crash, le journal permet de reprendre à une frontière sûre ; sans reçu
  confirmant un effet externe possible, le run entre dans `outcome_unknown`.
- [`09_reseau_apps_compute.md`](../../09_reseau_apps_compute.md), lignes 84–88 :
  le runtime local journalise, applique les politiques de retry et
  d’idempotence, puis refuse d’inventer le résultat d’un effet non idempotent
  resté sans reçu fiable.
- [`10_confiance_et_capacites.md`](../../10_confiance_et_capacites.md), lignes
  158–169 : répéter peut produire l’effet deux fois, tandis que déclarer
  l’échec peut être faux ; le moteur conserve donc explicitement l’incertitude.
- [`03_une_journee_dans_nexus.md`](../../03_une_journee_dans_nexus.md), ligne
  114 : le récit montre une tâche sans reçu qui n’est pas renvoyée
  aveuglément ; une nouvelle tentative exige une décision ultérieure et une
  nouvelle clé.

## Preuves dans le dépôt live

État inspecté : `HEAD 9134af64`, le 27 juillet 2026.

- [`lib.rs`](../../../../../crates/sbfb-workflow-engine/src/lib.rs), lignes
  35–46 : l’intention et l’outbox partagent une transaction ; contre une cible
  non idempotente, une reprise peut produire `outcome_unknown` sans rappeler la
  cible.
- Le même fichier, lignes 87–95, définit la frontière irréductible B2 : l’effet
  a pu être appliqué avant que son résultat soit journalisé. Une cible
  idempotente peut être rappelée sous la même clé ; une cible qui ne déclare pas
  cette propriété ne l’est pas.
- [`engine.rs`](../../../../../crates/sbfb-workflow-engine/src/engine.rs),
  lignes 377–443 : le moteur retrouve l’effet pending, vérifie le hash, la
  déclaration d’idempotence et le scope de déduplication journalisés ; si la
  cible n’est pas idempotente, il enregistre `TerminalReason::OutcomeUnknown`
  puis retourne `EngineError::OutcomeUnknown`, sans appeler la cible.
- [`orchestrator.rs`](../../../../../crates/sbfb-workflow-engine/src/orchestrator.rs),
  lignes 1464–1505 : à l’expiration d’une tentative, la branche idempotente peut
  entrer dans la politique de retry ; la branche non idempotente passe par
  `unknowable_outcome`.
- Le même fichier, lignes 1506–1538, borne cette fonction à deux issues :
  parking pour réconciliation ou fermeture terminale — « never an automatic
  retry, never an invented success ».
- [`store.rs`](../../../../../crates/sbfb-workflow-engine/src/store.rs), lignes
  2602–2635 : le parking `node_outcome_unknown` et la fermeture de la tentative
  sont journalisés dans une même transaction ; un run terminal ne peut pas être
  requalifié après coup.
- [`store_spike.rs`](../../../../../crates/sbfb-workflow-engine/tests/store_spike.rs),
  lignes 544–593 : un crash est injecté après l’effet mais avant le résultat ;
  la sortie externe reste comptée une seule fois après reprise et le run devient
  terminal `outcome_unknown`.
- [`orchestrator.rs`](../../../../../crates/sbfb-workflow-engine/tests/orchestrator.rs),
  lignes 1894–1956 : le test du parcours de graphe vérifie qu’un pending
  non-idempotent produit l’état terminal et que le reducer l’expose comme
  `RunState::OutcomeUnknown`.

### Limite honnête

Le runtime ne peut pas rendre l’effet « exactement une fois » par sa seule
volonté. Une réémission sûre dépend d’une cible qui déduplique durablement la
même clé dans le même backend. Quand cette propriété n’existe pas, suspendre et
exposer l’incertitude est la garantie réelle.

La réconciliation n’autorise pas non plus un opérateur à transformer une
supposition en succès. Le chemin `result_recovered` exige un reçu structuré et
relié à l’effet ; à défaut, la décision reste un abandon honnête ou une
vérification propre à l’activité.

## Historique Git vérifié

- `6de49b12` — 20 juillet 2026,
  `feat(workflow-engine): Sprint 83 Phase E — spike durable bloquant
  (journal/outbox atomiques, echo-target)` : introduit le journal/outbox
  atomique, la frontière B2 et le test discriminant prouvant qu’un effet
  non-idempotent incertain n’est pas relancé.
- `a23510e8` — 23 juillet 2026,
  `feat(workflow-engine): Sprint 83 Phase J — orchestrateur local déterministe` :
  transporte l’état terminal dans l’orchestrateur et son reducer.
- `90921211` — 24 juillet 2026,
  `feat(workflow-engine): Sprint 83 Phase K — effets, retries et idempotence` :
  ajoute les tentatives non bloquantes, timeouts, retries bornés et la
  réconciliation optionnelle sans conversion arbitraire en succès.

Le dossier `docs/book/nexus-v1/` est non suivi dans le worktree observé. Les
commits ci-dessus prouvent donc l’évolution du moteur live, pas un historique
Git déjà établi du manuscrit.

## Tradition visuelle choisie

### La communauté horlogère bisontine et ses planches techniques

La scène s’appuie sur le milieu historique de la « Fabrique » horlogère de
Besançon, de ses ateliers et de son école municipale puis nationale. Il ne
s’agit pas d’une communauté homogène disparue : cette histoire industrielle et
artisanale possède des continuités contemporaines dans les microtechniques et
l’horlogerie franc-comtoise. Le prompt mobilise son patrimoine documentaire,
pas l’identité supposée de ses descendants.

Sources institutionnelles :

- Le [Musée du Temps de Besançon — collections](https://www.mdt.besancon.fr/les-collections/)
  conserve 1 500 montres, ébauches et boîtes, des horloges, des outils et
  machines, ainsi qu’environ 2 000 gravures liées à l’horlogerie du XVIe au
  XIXe siècle. Son parcours documente les mécanismes, le processus de
  fabrication et le rôle de l’école d’horlogerie.
- Le [Service Inventaire et Patrimoine de Bourgogne-Franche-Comté](https://patrimoine.bourgognefranchecomte.fr/publications/lecole-dhorlogerie-de-besancon-le-lycee-jules-haag)
  retrace la manufacture nationale amorcée en 1793, l’ouverture de l’école
  municipale en 1862 et l’enseignement horloger maintenu jusqu’en 1988 avant la
  spécialisation dans les microtechniques.
- Le [dossier d’inventaire de l’École nationale d’Horlogerie](https://patrimoine.bourgognefranchecomte.fr/dossiers-inventaire/ecole-professionnelle-dite-ecole-nationale-dhorlogerie-puis-lycee-polyvalent-jules-haag-ia25000690)
  relie l’ouverture de l’école à la demande de la Fabrique bisontine et donne
  accès à des plans, coupes, dessins sur calque et documents d’atelier.
- Le [réseau Musées Bourgogne-Franche-Comté](https://musees-bfc.fr/musee-du-temps)
  signale que le Musée du Temps conserve aussi le matériel pédagogique de
  l’École nationale d’Horlogerie.

La grammaire retenue est celle d’une planche technique historique reconstruite :
vue frontale, mécanisme en coupe, pièces nettement séparées, hachures de cuivre
et lavis limités. L’indigo et l’ocre sont un choix éditorial destiné à séparer
les fonctions ; ils ne sont pas présentés comme une palette canonique de toute
l’horlogerie bisontine. Aucun artiste vivant, brevet identifiable, marque
commerciale, cadran historique ou planche précise n’est imité.

## Logique visuelle

La scène doit pouvoir être lue comme un seul instant après la perte du retour :

1. **Tentative partie** — le percuteur est détendu, la coupelle de départ est
   vide et la première capsule se trouve entièrement de l’autre côté du
   portillon à sens unique.
2. **Effet extérieur opaque** — la capsule entre dans une armoire distante
   close, sans fenêtre et sans résultat visible.
3. **Reçu absent** — un conduit indépendant ressort de l’armoire mais se
   termine au-dessus d’une coupelle de porcelaine vide.
4. **Aucune conclusion inventée** — l’armoire ne porte ni voyant, ni balance,
   ni couleur de succès ou d’échec.
5. **Retry bloqué** — une deuxième capsule existe, mais une cage à rochet
   solidaire du mécanisme détendu lui interdit physiquement d’atteindre le
   départ.
6. **Décision suspendue** — l’horlogère observe le retour vide, mains retirées
   de la manivelle et de l’armoire ; elle n’accomplit ni geste de victoire, ni
   geste d’alarme.

La composition ne tente pas de représenter une timeline entière. Elle montre
la frontière matérielle entre ce que le runtime sait — départ et absence de
reçu — et ce qu’il refuse de prétendre savoir — l’effet dans la cible.

## Risques de génération

1. **La capsule reste avant le portillon.** Le rendu ferait alors croire que
   l’opération n’est jamais partie. Elle doit être entièrement du côté distant.
2. **Un reçu apparaît dans la coupelle.** Toute pièce, lettre ou lumière dans
   le retour détruit l’incertitude.
3. **L’armoire révèle son intérieur.** Rejeter une porte ouverte, une fenêtre,
   un produit terminé, un voyant vert ou rouge, une cloche en action ou une
   fumée interprétable.
4. **La seconde capsule peut être envoyée.** La cage doit couper physiquement
   son accès au lancement et être engagée dans le mécanisme déjà détendu.
5. **Le tube devient une flèche.** Aucun conduit ne doit avoir de pointe, de
   chevron ou de marque de direction.
6. **De faux cadrans apparaissent.** Rejeter tout chiffre romain, nombre,
   inscription, étiquette, pseudo-texte ou symbole de statut.
7. **La scène devient une réparation générique.** Les trois preuves visuelles
   — capsule passée, retour vide, duplicata bloqué — doivent dominer les détails
   décoratifs de l’atelier.
8. **L’horlogère devient l’autorité qui sait.** Son expression doit rester
   attentive et neutre ; elle ne doit ni ouvrir la cible, ni désigner une
   conclusion, ni tenir la clé du cabinet.

## Test aveugle

Présenter l’image seule à cinq personnes, sans le titre, la phrase du livre ni
le mot `outcome_unknown`.

Demander :

1. « Une première opération est-elle déjà partie ? Qu’est-ce qui vous le fait
   penser ? »
2. « Peut-on savoir ce qui s’est passé dans l’armoire fermée ? »
3. « Quel objet attendu manque dans la scène ? »
4. « La machine peut-elle envoyer immédiatement la seconde capsule ? »
5. « L’image affirme-t-elle un succès, un échec, ou ni l’un ni l’autre ? »

### Seuil de réussite

- au moins quatre personnes sur cinq identifient une première tentative déjà
  partie ;
- au moins quatre repèrent le retour vide comme une confirmation manquante ;
- au moins quatre comprennent que la seconde capsule est physiquement bloquée ;
- aucune majorité ne conclut que l’opération a sûrement réussi ou sûrement
  échoué ;
- au moins trois résument spontanément la scène par une formulation proche de
  « on ne sait pas ce qui s’est passé, donc on ne recommence pas ».

Échec si le public voit seulement un atelier d’horlogerie, croit que le
mécanisme est en panne avant tout départ, ou interprète la capsule bloquée comme
une simple réserve de pièces. Dans ce cas, agrandir le portillon franchi, la
coupelle vide et le rochet, puis réduire les détails secondaires.
