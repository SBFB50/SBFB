# Recherche ultra-deep - developpement d'idee vers workflow optimal

Date: 2026-06-30
Statut: note de recherche. Aucun code, wire, node Flowise ou gate process n'est engage par ce document.

## 1. Verdict court

Le workflow optimal ne doit pas commencer par "comment construire l'app". Il doit d'abord traiter l'idee comme une hypothese vivante:

```text
intuition brute
  -> probleme reel
  -> preuves et limites
  -> divergence d'options
  -> incubation
  -> critique sceptique
  -> champion responsable
  -> decision humaine
  -> seulement ensuite brief Factory / app / sprint
```

Pour Nexus/SBFB, cela donne une separation stricte:

- Idea Hub: maturer, comparer, documenter, faire apparaitre un champion, garder la decision humaine.
- Factory: traduire l'idee mure en artefact SBFB concret: template, SBFB.json, bridge, stockage, compute, sandbox, preview, provenance, publish.

## 2. Corpus scientifique utilise

Cette recherche ne pretend pas lire "toute" la litterature mondiale. Elle extrait les invariants les plus robustes de plusieurs familles de travaux:

- Creativite organisationnelle: Amabile / componential theory, motivation, expertise, environnement social.
- Creative problem solving: separation problem finding, idea generation, idea evaluation, implementation.
- Brainstorming et ideation collective: pertes de productivite, production blocking, evaluation apprehension, interet de l'ideation individuelle/asynchrone.
- Incubation: pauses utiles apres preparation cognitive.
- Design fixation: les exemples et solutions precoces enferment l'espace de conception.
- Design thinking / human-centered design: cadrage probleme, empathie, prototypage et reduction des biais.
- Petites equipes vs grandes equipes: petits noyaux plus disruptifs, grands groupes meilleurs pour developper/industrialiser.
- IA generative: augmente l'individu, mais peut reduire la diversite collective si tout le monde s'appuie sur le meme modele.
- Decision: premortem/prospective hindsight, Delphi/feedback structure, dissent explicite.

Sources pivots:

- Amabile, T. M. - componential theory / creativity and innovation: https://www.hbs.edu/ris/Publication%20Files/12-096.pdf
- Sio & Ormerod - incubation meta-analysis: https://doi.org/10.1037/a0014212
- Diehl & Stroebe - productivity loss in brainstorming groups: https://doi.org/10.1037/0022-3514.53.3.497
- Paulus & Yang - group idea generation with exchange: https://doi.org/10.1006/obhd.2000.2888
- Jansson & Smith - design fixation: https://doi.org/10.1016/0142-694X(91)90003-F
- Dorst - core of design thinking / frame creation: https://doi.org/10.1016/j.destud.2011.07.006
- Liedtka - design thinking and cognitive bias reduction: https://doi.org/10.1111/jpim.12163
- Wu, Wang & Evans - small teams disrupt, large teams develop: https://doi.org/10.1038/s41586-019-0941-9
- Mitchell, Russo & Pennington - prospective hindsight / premortem: https://doi.org/10.1016/0749-5978(89)90010-3
- Doshi & Hauser - generative AI improves individual creativity but reduces collective diversity: https://doi.org/10.1126/sciadv.adn5290

## 3. Invariants scientifiques pour le workflow

### 3.1 Une idee n'est pas bonne parce qu'elle est nouvelle

Une idee creative doit etre nouvelle ET appropriee. Donc le workflow doit noter separement:

- nouveaute;
- utilite;
- faisabilite;
- benefice pour les personnes;
- dommage evite;
- capacite a vivre en open source/local/P2P.

### 3.2 Le probleme doit preceder la solution

Les workflows qui partent trop vite vers la solution confondent intuition, desir technique et besoin reel. Il faut obliger une etape "probleme actuel sans l'app".

Question cle:

```text
Que fait la personne aujourd'hui sans cette app, et qu'est-ce qui echoue vraiment?
```

### 3.3 Divergence et convergence doivent etre separees

Melanger generation et jugement tue les options fragiles. Le bon pattern:

1. generer plusieurs cadrages sans juger;
2. laisser incuber;
3. evaluer ensuite avec criteres explicites;
4. garder l'option "ne pas construire".

### 3.4 Ideation individuelle d'abord, groupe ensuite

Les groupes perdent des idees par blocage de production, peur du jugement et alignement premature. Donc:

- d'abord capture individuelle/asynchrone;
- ensuite regroupement et synthese;
- ensuite review contradictoire;
- jamais vote global comme gate.

### 3.5 L'incubation est une vraie etape

Une idee doit pouvoir rester "en maturation". Pour un workflow local:

- micro-pause: quelques minutes avant convergence dans une session;
- pause longue: 24-72 h pour une idee structurante;
- exception: bug critique ou besoin utilisateur deja prouve.

### 3.6 Eviter la fixation par les exemples et par SBFB trop tot

Montrer trop vite un template, un workflow Flowise, une app existante ou un pattern SBFB peut enfermer l'idee. Donc Idea Hub doit rester sobre au debut. SBFB arrive comme contrainte de traduction apres maturation, pas comme moule initial.

### 3.7 Le role du LLM est de composer, pas de prouver

Un LLM peut:

- reformuler;
- generer des options;
- chercher des objections;
- rendre lisible;
- proposer une prochaine action.

Il ne doit pas:

- valider qu'une idee est "bonne";
- produire un verdict final;
- masquer l'incertitude;
- remplacer un champion humain;
- uniformiser toutes les idees vers le meme style.

Mitigation: demander plusieurs options incompatibles, un "do nothing", un reviewer sceptique, et garder les sources/limites visibles.

### 3.8 Un champion compte plus qu'un vote

Les votes globaux favorisent popularite et ranking. Pour un commun open source, le signal fort est:

```text
je prends cette idee + prochaine action + date + critere d'abandon
```

Un claim sans plan d'action reste faible.

### 3.9 Petits noyaux pour la rupture, plus grands groupes pour developper

Le bon seuil pour Idea Hub:

- 1 a 3 personnes: cadrage, rupture, decision de porter;
- 3 a 7: recherche, objections, prototype;
- plus large: usage, maintenance, curation, documentation.

### 3.10 La critique doit etre structuree, pas sociale

Chaque idee a besoin d'un "pourquoi ne pas construire":

- risque de capture;
- donnees sensibles;
- dependance centrale;
- cout de maintenance;
- accessibilite;
- securite;
- alternative non-app;
- alternative locale plus simple;
- effet pervers si l'idee reussit.

## 4. Workflow optimal recommande

### Phase A - Capture sobre

But: capturer sans bloquer.

Sortie: `IdeaSeed`

Champs:

- titre provisoire;
- intuition brute;
- pour qui;
- situation actuelle;
- douleur ou limite;
- pourquoi maintenant;
- niveau de flou: clair / moyen / tres flou.

Regle: pas encore de solution SBFB detaillee.

### Phase B - Clarification du besoin

But: transformer l'intuition en probleme.

Sortie: `ProblemFrame`

Questions:

- Qui vit ce probleme?
- Que fait cette personne aujourd'hui?
- Qu'est-ce qui echoue, coute trop, centralise trop, ou exclut quelqu'un?
- Qu'est-ce qui doit rester prive/local?
- Quelle preuve montrerait que le probleme existe?

### Phase C - Reframing divergent

But: eviter la fixation.

Sortie: `OptionSet`

Produire au minimum:

- option 0: ne rien construire / mieux documenter / organiser autrement;
- option 1: outil local minimal;
- option 2: commun open source partageable;
- option 3: app locale/P2P;
- option 4: app avec compute local ou partage GPU si vraiment utile;
- option radicale: autre cadrage du probleme.

Regle: pas de selection pendant cette phase.

### Phase D - Incubation

But: reduire l'ancrage.

Sortie: `IncubationNote`

Modes:

- rapide: pause courte + relecture;
- normale: garder en recherche;
- urgente: justifier pourquoi on saute l'incubation.

### Phase E - Evidence pack

But: distinguer savoir, hypothese et opinion.

Sortie: `ResearchPack`

Chaque claim recoit un label:

- Lu: information collectee mais pas verifiee;
- Deduit: inference;
- Verifie: preuve fichier/source/test;
- Non verifie: a chercher.

Le pack doit inclure:

- sources;
- limites;
- hypotheses;
- risques;
- points inconnus;
- preuve minimale a construire.

### Phase F - Review sceptique

But: chercher pourquoi il ne faut pas construire.

Sortie: `SkepticReview`

Angles obligatoires:

- besoin faux ou trop faible;
- alternative plus simple;
- capture ou ranking;
- centralisation cachee;
- donnees sensibles;
- accessibilite et handicaps cumules;
- maintenance solo impossible;
- cout compute/GPU;
- securite/provenance.

### Phase G - Champion et engagement

But: remplacer "j'aime" par "je porte".

Sortie: `ChampionClaim`

Champs:

- champion;
- prochaine action;
- date de relecture;
- definition d'un succes minimal;
- critere d'abandon;
- aide demandee;
- statut: explorer / prototyper / preparer Factory / garder / rejeter.

### Phase H - Decision humaine

But: ne pas donner l'autorite au LLM.

Sortie: `DecisionPack`

Decisions possibles:

- garder en recherche;
- demander preuves supplementaires;
- fusionner avec une idee existante;
- chercher un champion;
- preparer un brief Factory;
- ouvrir un sprint;
- rejeter pour l'instant.

### Phase I - Passage Factory

But: traduire l'idee mure en build concret.

Sortie: `FactoryIntakeBrief`

Seulement ici, on introduit:

- type d'app;
- ecrans MVP;
- donnees locales;
- donnees partagees P2P;
- besoins compute;
- bridge methods;
- sandbox/CSP;
- SBFB.json;
- preview;
- provenance;
- gates;
- tests.

## 5. Forme de workflow Flowise recommandee

Ne pas faire un seul gros agent qui repond tout d'un coup. Utiliser un workflow interactif en etapes, mais sans feedback force a chaque message.

Noeuds:

1. `Start - Idee brute + langue + contexte`
2. `LLM - Capture sobre`
3. `LLM - Clarification du probleme`
4. `LLM - Reframing divergent`
5. `LLM - Evidence pack`
6. `LLM - Review sceptique`
7. `LLM - Synthese courte`
8. `Human decision - continuer / garder / preparer Factory`
9. `LLM - FactoryIntakeBrief` seulement si l'humain choisit Factory
10. `Direct reply - DecisionPack final`

Comportement:

- chaque etape renvoie une reponse lisible;
- l'utilisateur peut discuter avant la prochaine etape;
- 3 suites IA maximum;
- aucune validation obligatoire apres chaque retour;
- le LLM conserve les messages precedents;
- les decisions finales restent humaines;
- les claims restent des donnees, pas des commandes.

## 6. Prompt systeme court pour Idea Hub

```text
Tu aides a maturer une idee avant toute construction.
Tu ne vends pas l'idee et tu ne cherches pas a la transformer trop vite en app.
Tu separes: probleme, beneficiaires, preuves, options, risques, decision humaine.
Tu proposes toujours plusieurs cadrages, dont "ne pas construire".
Tu ne declares jamais qu'une idee est validee definitivement.
Tu peux mentionner local/P2P/open source si pertinent, mais tu ne forces pas SBFB avant la phase Factory.
Tu utilises des labels: Lu, Deduit, Verifie, Non verifie.
Tu gardes le LLM comme aide a la composition, jamais comme preuve ou verdict final.
```

## 7. Prompt systeme court pour Factory Intake

```text
Tu traduis une idee deja muree en brief de construction Factory.
Tu dois rendre concret: type d'app, MVP en 3 ecrans, donnees locales, donnees partagees, bridge methods, permissions, sandbox/CSP, provenance, preview, tests et gates.
Si compute est utile, precise: local, partage GPU/CPU S76, sharding S77 avance, donnees envoyees, pii_redact, task_submit, task_result, consentement, caps, fallback et preuves.
Si compute est inutile, dis-le pour simplifier l'app.
Tu ne changes pas la decision humaine et tu ne produis aucun verdict PASS.
```

## 8. Gates de qualite du workflow

- NR0: l'idee est traitee comme hypothese, pas comme promesse.
- NR1: probleme et solution sont separes.
- NR2: au moins 3 options + "ne pas construire".
- NR3: une phase d'incubation existe ou son absence est justifiee.
- NR4: chaque claim important a un label de preuve.
- NR5: review sceptique obligatoire.
- NR6: pas de ranking global comme gate.
- NR7: le champion a une prochaine action et un critere d'abandon.
- NR8: le LLM ne produit pas de verdict final.
- NR9: passage Factory seulement apres decision humaine.
- NR10: SBFB technique arrive en Factory Intake, pas en capture initiale.

## 9. Decision de recherche

La meilleure voie est de maintenir deux workflows connectes:

1. `Idea Hub / Atelier des communs`: workflow scientifique de maturation d'idee, sobre, anti-fixation, anti-ranking, humain-arbitre.
2. `Factory Intake`: workflow technique qui compile le `DecisionPack` en brief SBFB concret.

Le workflow Flowise actuel charge avec beaucoup de contexte SBFB doit etre classe comme `Factory Intake / SBFB adaptation`. Il faut creer un deuxieme workflow plus neutre pour `Idea Hub`.
