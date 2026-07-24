# Sprint 83 — Vue Product Owner : chantier « Workflow Agents »

> **Statut du document** : vue de lecture pour le PO, générée le
> 2026-07-23. Elle **ne fait pas autorité** — le plan technique canonique
> reste `sprint83_plan.md` (43 phases) et les décisions gelées vivent dans
> `sprint83_kickoff.md`. Si un détail diverge, ce sont eux qui ont raison.

---

## 1. Pourquoi ce sprint

Aujourd'hui, le « process Factory » (kickoff → plan → code → review →
audit) vit dans des documents et dans la discipline d'un agent. Il marche,
mais il n'est **ni portable, ni vérifiable par une machine, ni partageable**.

Le sprint 83 transforme ce process en **produit** :

- **Un format** : un « pack » signé qui décrit un workflow d'agents (qui
  fait quoi, dans quel ordre, avec quels garde-fous) — publiable et
  installable sur le réseau SBFB comme n'importe quelle app.
- **Un moteur** : un runtime local qui exécute ces workflows de façon
  **fiable** (un crash ne perd rien de ce qui est confirmé, une action
  confirmée n'est jamais refaite) et **honnête** (quand le résultat est
  inconnaissable, il le dit — il n'invente jamais un succès ; une action
  au sort inconnu peut être re-tentée sous la même clé, jamais dupliquée
  en silence côté cible qui déduplique).
- **Une app** : le « Workflow Hub », l'interface où l'on découvre, lit,
  lance et suit ces workflows.
- **Une preuve** : à la fin, le process Factory lui-même tourne DANS ce
  système (dogfood), et devient le premier workflow publié.

La règle de sécurité cardinale du chantier : **le graphe signé EST le
plan**. Une IA peut travailler à l'intérieur d'une étape, mais elle ne
peut jamais étendre le plan, s'accorder un droit, changer une destination
ou prononcer un verdict à la place du process.

---

## 2. Où on en est (2026-07-23)

**14 phases jouées sur 43. Le moteur est construit ; l'app reste à faire.**

| Lot | Phases | État |
|---|---|---|
| Fondations & dette héritée | 0 · A · A1 | ✅ Fait |
| Socle transverse (accessibilité, i18n, sécurité du pont apps) | B · C | ✅ Fait |
| Le moteur d'exécution | D · E · F · G · H · I · I1 · J · **K** | ✅ Fait — **K en clôture de vérification** (code fini, tests verts, dernier contrôle croisé en cours) |
| Sécurité d'exécution | L · L1 · M | À venir (prochaines) |
| Connexions IA | N · O · P · P1 | À venir |
| L'app Workflow Hub | Q · R · S · T · U · V · W · W1 | À venir |
| Écosystème réseau | X · Y | À venir |
| La preuve par l'usage (dogfood) | Z · AA · AB · AC · AD · AE · AF · AG | À venir |
| Blindage & fermeture | AH · AI · AI1 · AI2 · AJ · AJ1 | À venir |

Deux jalons majeurs sont **derrière nous** :

- **Phase E (GO/NO-GO)** : la preuve que le moteur survit à un crash sans
  jamais refaire une action déjà **confirmée** — et quand il ne peut pas
  savoir, il le déclare au lieu d'inventer. C'était le verrou du sprint — GO.
- **Phase G (gel du langage)** : le vocabulaire des workflows (18 types de
  nœuds) est gelé, avec 4 exemples complets et réalistes qui servent
  d'étalon depuis.

---

## 3. Ce que chaque lot apporte (langage produit)

### Lot 1 — Fondations (0, A, A1) ✅
L'audit du sprint précédent est purgé, les contrats de preuve du sprint
sont posés (comment on saura que c'est fini et que ça marche), et un
paquet de dettes héritées (CI rouge, licences, tests instables) est réglé
pour partir propre.

### Lot 2 — Socle transverse (B, C) ✅
- **B** : l'accessibilité et le français/anglais/arabe sont des fondations,
  pas des rustines — avec une garantie forte : les besoins d'accessibilité
  d'un utilisateur (donnée de santé, donc sensible) **ne quittent jamais
  sa machine**.
- **C** : le « pont » par lequel une app parle au réseau est verrouillé :
  une app n'obtient que ce que sa fiche déclare, ce que l'utilisateur
  accorde, et ce que la politique locale autorise — l'intersection des
  trois, jamais plus.

### Lot 3 — Le moteur (D → K) ✅
C'est le cœur livré ce mois-ci :

- **D** : le vrai process Factory est extrait en un contrat lisible par
  machine — la matière première du premier workflow.
- **E** : la preuve de fiabilité (crash-test) — GO.
- **F** : le format de pack **signé** : personne ne peut altérer un
  workflow publié sans que ça se voie.
- **G** : le langage des workflows (étapes, décisions, jonctions,
  validations humaines…) — gelé sur 4 exemples réels.
- **H** : le compilateur qui refuse tout plan malformé ou dangereux
  **avant** toute exécution (20 classes de refus).
- **I + I1** : la mémoire durable des exécutions (chaque run laisse un
  journal complet, consultable, sauvegardable) + sa consultation en
  lecture seule depuis le daemon.
- **J** : le chef d'orchestre : il déroule le plan de façon déterministe —
  rejouer le même journal redonne toujours le même état, une annulation
  est honnête et absorbante.
- **K (en clôture)** : les « bras » du moteur — les actions réelles
  (fichiers, processus) avec retries bornés, timeouts, nettoyage des
  processus enfants (Windows inclus), et la règle d'or : **un résultat
  inconnaissable est déclaré inconnaissable**, jamais converti en succès ;
  seule une preuve récupérée par l'opérateur peut débloquer, et c'est
  tracé.

### Lot 4 — Sécurité d'exécution (L, L1, M)
Qui a le droit de faire quoi : les permissions ne peuvent que se
**réduire** en descendant les couches (jamais s'élargir) ; les secrets
vivent dans un coffre local et les approbations humaines sont signées ;
chaque artefact produit garde sa provenance.

### Lot 5 — Connexions IA (N, O, P, P1)
Le moteur parle aux modèles : locaux (Ollama), cloud, et surtout **Claude
Code et Codex** comme exécutants d'étapes — avec reçus d'exécution
attestés. P/P1 ouvrent le calcul distribué SBFB et un usage MCP borné.

### Lot 6 — L'app Workflow Hub (Q → W1)
L'interface : accueil et bibliothèque de workflows (R), registre d'agents
(S), **canvas visuel** avec plan et mode guidé (T), inspecteurs et
vérifications avant lancement (U), **Centre des Runs** (V — suivre,
annuler, relancer, réconcilier un run), évaluations comparées (W), et un
**premier panel d'utilisateurs réels** (W1).

### Lot 7 — Écosystème (X, Y)
Publier, découvrir et installer des packs sur le réseau — avec les mêmes
garanties de source vérifiable que les apps ; opérations entre apps.

### Lot 8 — La preuve par l'usage (Z → AG)
Le dogfood : le process Factory complet (kickoff → plan → préflight →
implémentation → review → verdict) tourne **dans** le système, et devient
le **premier workflow publié** (AC). Puis trois autres packs concrets
(Ideas, Conception, Babel) et l'assistance IA à la création (AG).

### Lot 9 — Blindage & fermeture (AH → AJ1)
Attaques simulées et chaîne d'approvisionnement (AH), tenue en charge
(AI), accessibilité multi-outils + panel round 2 (AI1), **acceptance
agrégée** (AI2 — c'est là que les paliers de preuve passent de « pas
encore mesuré » à un verdict machine), bascule contrôlée avec retour
arrière possible (AJ), et clôture documentaire (AJ1).

---

## 4. Garde-fous produit (tenus à chaque phase)

- **Aucune rupture réseau** : tous les formats publiés restent en v1 ;
  rien de ce sprint ne casse un nœud existant.
- **Aucune dépendance runtime ajoutée** au daemon.
- **Le mur reste un mur** : l'interface ne calcule jamais un verdict —
  elle restitue ce que le moteur a prouvé.
- **Pas de monétisation** : les kudos restent une réputation, jamais une
  monnaie ; aucun mécanisme de paiement/stake dans les packs.
- **Pilote fermé** tant que l'audit sécurité du transport (R-iroh) n'est
  pas fait — pas d'ouverture publique dans ce sprint.

---

## 5. Décisions PO — jouées et à venir

**Déjà arbitrées pendant le sprint** (échantillon) :
- Le reçu d'exécution de la phase K reste local et non-signé (la
  signature riche arrive avec les adaptateurs Claude/Codex en O) — évite
  d'élargir le périmètre crypto gelé.
- La publication d'une idée est toujours une **décision humaine**, jamais
  une sortie de modèle (trouvaille de la phase G, corrigée dans le
  langage lui-même).
- Les écrans d'administration des runs (restaurer, purger, forcer un
  writer) sont rattachés au **Centre des Runs (V)** — pas de porte
  réseau avant qu'il existe.

**Encore ouvertes** (kickoff §11, à trancher au fil de l'eau) :
- Q3–Q7 : détails d'exécution des décisions D1..D5 (arriveront avec les
  phases concernées).
- Q10–Q15 : dont **Q12** — faut-il committer les 2 Research Packs dans le
  dépôt ou les garder hors-dépôt ? (aujourd'hui non stagés).
- Q9-b : recrutement du **panel accessibilité** (long délai — à lancer en
  parallèle, indépendamment du code).

---

## 6. Prochaine étape immédiate

1. **Clore la phase K** : dernier contrôle croisé (Codex round 2) en
   cours → commit.
2. **Phase L** : permissions et atténuation (7 couches, 17 classes de
   capacités, testées par propriétés).
3. En parallèle PO : lancer le recrutement du panel a11y (Q9-b) et
   trancher Q12.
