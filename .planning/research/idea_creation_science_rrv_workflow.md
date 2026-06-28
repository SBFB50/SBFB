# Idea Creation Science + RRV Workflow

Date: 2026-06-28
Status: NOTE DE RECHERCHE. Hors sprint. Aucun code, aucun wire, aucun commit
de phase, aucun verdict process. Cette note complete
`idea_development_workflow_provider_neutral.md` avec la psychologie de la
creation d'idee, la science des groupes et l'integration RRV.

Update cadrage 2026-06-28: le cadre produit correct est
`idea_commons_workflow_anti_capture.md`. Cette note parle de psychologie et
RRV; elle doit etre lue comme soutien a l'Atelier des communs, pas comme
funnel SaaS.

## 1. These

Le bon systeme n'est pas un chat a idees, ni un tableau de votes, ni un LLM
charge de trouver "la bonne idee".

Le bon systeme est un atelier de maturation des communs:

```text
idee brute
  -> clarification
  -> incubation
  -> grounding repo + RRV
  -> options divergentes
  -> critique sceptique
  -> pack de recherche
  -> arbitrage humain
  -> eventuellement brief Factory, pilote local ou kickoff sprint
```

Les idees doivent etre traitees comme des hypotheses vivantes. RRV apporte la
discipline de preuve et de limites; Factory transforme seulement une idee
deja mure en commun verifiable ou artefact/app; l'humain arbitre. Le LLM brouillonne, structure,
critique et compare, mais ne devient jamais preuve ni autorite finale.

## 2. Ce que disent les sources scientifiques

### 2.1 Creativite = nouveaute + adequation

Amabile definit la creativite comme une production a la fois nouvelle et
appropriee au but. Sa theorie componentielle combine expertise de domaine,
processus creatifs, motivation intrinseque et environnement social.

Consequence pour le commun:

- ne pas mesurer seulement la nouveaute;
- forcer aussi l'utilite collective, le dommage evite, la faisabilite et
  l'alignement SBFB;
- eviter les classements sociaux globaux qui tuent l'autonomie et la
  motivation intrinseque;
- demander un environnement de critique securise, pas une arene de votes.

### 2.2 Divergence et convergence doivent etre separees

La generation d'idees et l'evaluation d'idees n'utilisent pas les memes modes
cognitifs. Les melanger pousse vers la censure precoce, la solution unique et
la rationalisation apres coup.

Consequence pour le commun:

- `Intake` capture sans jugement;
- `OptionSet` demande plusieurs options, dont `Reject / do nothing`;
- `DecisionPack` arrive seulement apres grounding et review;
- l'UI ne doit pas afficher un bouton "go sprint" pendant la capture.

### 2.3 L'incubation est utile, mais doit etre bornee

Sio et Ormerod montrent par meta-analyse que l'incubation peut ameliorer la
resolution de probleme. Ce n'est pas une excuse pour repousser indefiniment:
c'est un mecanisme de pause, retour et relecture.

Consequence pour le commun:

- introduire un `IncubationTicket`;
- stocker les questions ouvertes;
- definir une date ou un evenement de retour;
- interdire la promotion directe idee fraiche -> sprint dans le meme flux.

### 2.4 La fixation est un risque majeur

Jansson et Smith decrivent la fixation de design: un exemple initial peut
ancrer les concepteurs, meme quand il contient des defauts. Dans un repo mature,
la fixation peut venir du style existant, d'une ancienne roadmap, d'une
suggestion LLM ou d'un vote initial.

Consequence pour le commun:

- cacher les votes/curations pendant la divergence;
- produire au moins une option "contre-style repo";
- produire une option conservative, une option protocole-native, une option
  solidarite/humanitaire/gouvernance locale et une option reject;
- lancer des reviewers independants avant la synthese.

### 2.5 La securite psychologique rend les objections exploitables

Edmondson lie la securite psychologique d'equipe aux comportements
d'apprentissage. Pour SBFB, cela signifie que l'objection doit etre un signal
de qualite, pas une punition sociale.

Consequence pour le commun:

- feedback oriente tache/preuve/risque;
- aucun jugement de personne;
- dissent visible et preservable;
- reviewer sceptique explicite dans le workflow.

### 2.6 Petits groupes pour la rupture, plus grands groupes pour developper

Wu, Wang et Evans montrent que les petites equipes tendent a produire plus de
rupture tandis que les grandes developpent davantage l'existant. Woolley et al.
montrent aussi que l'intelligence collective depend de facteurs d'interaction
comme sensibilite sociale et repartition de la parole, pas seulement du QI
individuel.

Consequence pour le commun:

- maturer une idee avec un micro-noyau de 1 a 3 humains;
- grandir seulement apres ResearchPack ou brief Factory;
- eviter les comites larges au moment de la rupture initiale;
- mesurer qui porte et qui critique, pas seulement combien votent.

### 2.7 Une intention ne livre rien sans plan d'action

Gollwitzer et Sheeran montrent que les implementation intentions, sous forme
"si situation Y, alors action X", ameliorent le passage intention -> action.

Consequence pour le commun:

- remplacer "je vote" par "je prends en charge";
- un porteur responsable doit definir engagement, date, prochaine action et critere
  d'abandon;
- un claim sans action plan reste un signal faible.

### 2.8 L'IA augmente l'individu mais peut appauvrir le collectif

Doshi et Hauser montrent que l'acces a des idees GenAI peut ameliorer la
creativite percue des productions individuelles tout en reduisant la diversite
collective des contenus.

Consequence pour le commun:

- capturer l'idee humaine brute avant suggestions LLM;
- lancer plusieurs brouillons independants;
- ne pas donner la meme synthese aux reviewers;
- mesurer la diversite des options, pas seulement la qualite d'une option;
- stocker sorties, sources et limites, pas le raisonnement cache comme
  autorite.

### 2.9 Gouvernance locale, pas oracle global

Ostrom defend les systemes polycentriques pour les communs complexes. Pour
Ideas Hub/RRV, cela pousse vers vues locales, curations revocables, dissent et
preuves, pas un score universel d'idee.

Consequence pour le commun:

- pas de leaderboard global d'idees;
- pas de trust score global;
- vues par noeud, groupe ou curator;
- export/fork des ResearchPacks;
- dissent et desaveu visibles.

## 3. Lecture repo-grounded

### 3.1 Ideas Hub actuel

`examples/sbfb-ideas/app.js` est aujourd'hui une app simple de stockage/vote:
elle liste `ideas/`, liste `votes/`, trie par votes ou recent, cree une idee et
permet de voter/supprimer ses propres idees.

Conclusion: c'est un bon canari storage/P2P, mais ce n'est pas encore une
machine de maturation d'idees. Le futur Ideas Hub ne doit pas seulement ajouter
des votes; il doit ajouter statuts, prises en charge, PaquetsRRV, reviews,
incubation, anti-capture et curation locale.

### 3.2 Workflow provider-neutral deja pose

`idea_development_workflow_provider_neutral.md` pose deja la bonne regle:

```text
hub PROPOSE; Operator DEVELOPPE; humain ARBITRE; gates VERIFIENT
```

Cette note ajoute la raison cognitive: proposition, developpement, arbitrage et
verification doivent rester separes car ils protegent contre fixation,
automation bias, censure precoce et faux consensus.

### 3.3 RRV

`rrv_app_protocol_best_features.md` pose la promesse RRV:

1. trouver un artefact;
2. expliquer son origine;
3. montrer ce qui est verifie, deduit ou seulement lu;
4. proposer une action bornee;
5. conserver les labels de confiance.

Les labels utiles pour le workflow d'idee sont:

| Label | Usage dans Ideas Workflow |
| --- | --- |
| `Lu` | present dans repo, manifest, feed, doc, archive |
| `Deduit` | rapprochement local, analyse, inference |
| `Verifie` | test, hash, signature, build, provenance, quorum |
| `Non verifie` | web, hypothese LLM, declaration non prouvee |

RRV ne decide pas. RRV classe, cite, indexe et expose les limites.

### 3.4 Factory et Operator

`docs/agent/RRV_FACTORY_CONTRACT.md` fixe l'autorite descendante:

```text
process > RRV > Factory
```

Pour Ideas Workflow:

- l'app sandboxee transmet une donnee signee ou un paquet exporte;
- l'Operator local privilegie lit le repo et prepare les artefacts;
- Factory peut produire un app brief ou une app si l'humain arbitre;
- ni Viewer, ni RRV, ni LLM ne produisent un verdict process final.

## 4. Modele de commun recommande

### 4.1 Entites

```json
{
  "GraineDeBesoin": {
    "title": "string",
    "raw_intuition": "string",
    "source": "chat|ideas_hub|repo|forum|bug|operator",
    "created_by": "node_or_local_operator",
    "tags": ["factory", "rrv"],
    "authority": "none"
  },
  "BriefDeCommun": {
    "problem": "string",
    "people_affected": "string",
    "why_sbfb": "string",
    "commons_purpose": "string",
    "non_goals": ["string"],
    "falsifying_evidence": ["string"],
    "anti_capture_risks": ["lock_in", "leaderboard", "data_capture", "gpu_rent"],
    "sensitive_surfaces": ["sandbox", "signing", "storage", "privacy"]
  },
  "IncubationTicket": {
    "open_questions": ["string"],
    "return_trigger": "date|repo_event|external_source|human_ping",
    "cannot_promote_before": "timestamp"
  },
  "RRVEvidenceBundle": {
    "claims": [
      {
        "claim": "string",
        "label": "Lu|Deduit|Verifie|Non verifie",
        "source": "file:line|url|feed|hash|test",
        "limits": "string"
      }
    ]
  },
  "OptionSet": {
    "conservative": "string",
    "protocol_native": "string",
    "solidarity_or_local_governance": "string",
    "reject_do_nothing": "string"
  },
  "DecisionPack": {
    "decision": "reject|research_only|factory_brief|sprint_candidate|needs_more_evidence",
    "human_arbitrator": "string",
    "reason": "string"
  }
}
```

### 4.2 Roles

| Role | Responsabilite | Ne doit pas faire |
| --- | --- | --- |
| Proposer | apporte probleme, intuition, contexte | decider seul |
| Porteur responsable | signe une prise en charge, porte l'idee, fixe prochaine action | remplacer les preuves |
| RRV Researcher | retrouve sources, contradictions, labels | emettre verdict final |
| Skeptic Reviewer | cherche pourquoi ne pas construire | attaquer la personne |
| Maintainer/Arbitre | promeut, garde en recherche, rejette | masquer les limites |
| Curator | atteste/desavoue dans sa vue | creer score global |
| Operator | assemble contexte, packs, prompts, traces | recevoir controle d'une app sandboxee |
| LLM | brouillonne, structure, critique, compare | devenir preuve ou arbitre |

### 4.3 Pipeline non-sprint

```text
0. Capture
   Sortie: IdeaSeed.
   Regle: aucune evaluation, aucun vote global affiche.

1. Clarification
   Sortie: ClarificationBrief.
   Regle: utilite collective, personnes concernees, non-goals, preuve
   falsifiante.

2. Incubation
   Sortie: IncubationTicket.
   Regle: pause volontaire, retour borne.

3. Repo + RRV grounding
   Sortie: RRVEvidenceBundle.
   Regle: labels Lu/Deduit/Verifie/Non verifie.

4. Options divergentes
   Sortie: OptionSet.
   Regle: au moins 3 options + reject/do nothing.

5. Reviews independantes
   Sortie: SkepticalReviewPack.
   Regle: steward d'usage, protocol, security, process, maintainer,
   cost/privacy, anti-capture.

6. Synthese
   Sortie: ResearchPack sous .planning/research/.
   Regle: integrer les objections, ne pas moyenner les avis.

7. Arbitrage humain
   Sortie: DecisionPack.
   Regle: seul l'humain promeut vers Factory ou sprint.
```

## 5. Contraintes RRV/Ideas Hub

### 5.1 Pas de ranking global

Un vote global transforme vite une hypothese en concours de popularite. Pour
SBFB, les signaux doivent rester locaux et explicites:

- `claim` par champion;
- `attestation locale reversible` par curator;
- `dissent` par reviewer;
- `watch` pour suivre sans soutenir;
- `fork` de ResearchPack;
- `abandon` explicite avec raison.

Un score public peut exister comme vue locale, jamais comme verite protocolaire.

### 5.2 Claims avant votes

Le seuil utile n'est pas "10 personnes aiment". Le seuil utile est:

```text
1 porteur responsable + 1 review sceptique + 1 evidence bundle RRV minimal
```

Un porteur responsable n'est pas une autorite. C'est une responsabilite
tracable:

- qui porte;
- jusqu'a quand;
- prochaine action;
- critere d'abandon;
- lien vers ResearchPack.

### 5.3 RRV comme couche de provenance cognitive

RRV doit empecher la fusion de statuts:

- une hypothese LLM reste `Non verifie`;
- une ligne de code lue est `Lu`;
- une inference repo est `Deduit`;
- un hash/signature/test est `Verifie`;
- une source web reste externe tant qu'elle n'est pas reliee a une preuve SBFB.

Le rendu UI doit afficher cette difference phrase par phrase, pas seulement au
niveau du document.

### 5.4 Cross-node provisional

Le registre d'idees cross-node doit rester marque `PROVISIONAL` tant qu'il
n'est pas prouve par E2E multi-noeud frais. En pilote ferme, le modele honnete
est:

```text
single-node / per-curator view first
cross-node maturation later, apres preuve de convergence
```

## 6. Utilisation des LLM cloud et locaux

### 6.1 Pattern multi-modele sain

```text
Human raw seed
  -> local/private extraction
  -> repo/RRV grounding
  -> cloud research si necessaire
  -> driver draft
  -> blind skeptical reviewers
  -> synthesis
  -> deterministic validator
  -> human arbitration
```

Les reviewers ne doivent pas tous voir la meme synthese initiale. Sinon le
systeme cree de la convergence artificielle.

### 6.2 Capacites utiles

| Capacite | Usage |
| --- | --- |
| long context | lire gros packs/repo docs |
| structured output | produire schemas exploitables |
| tool use | verifier sources, schemas, calculs |
| web research | API, OSS, papiers recents |
| file search | corpus projet, sources, prior art |
| code execution | prototypes non destructifs |
| prompt cache | repetitions de review moins couteuses |
| local_only | idees privees, brouillons sensibles |

Le produit doit enregistrer les capacites utilisees, pas seulement le nom du
provider.

### 6.3 Trace minimale

Chaque run LLM devrait laisser:

- `workflow_id`;
- stage;
- schema version;
- prompt hash;
- provider/model;
- capability flags;
- input refs;
- retrieval refs;
- tool calls;
- output hash;
- validator result;
- disagreements;
- cost/tokens si disponibles;
- data boundary;
- human decision.

## 7. Gates non-sprint proposes

Ces gates ne sont pas des gates de phase. Ils servent a verifier un ResearchPack
avant promotion.

| Gate | Question |
| --- | --- |
| NR0 Scope | Le document est-il clairement hors sprint? |
| NR1 Separation | Capture, clarification, review et decision sont-elles separees? |
| NR2 Evidence | Chaque claim a-t-il source, label RRV ou `Not evidenced`? |
| NR3 Anti-fixation | Y a-t-il plusieurs options et un reject/do nothing? |
| NR4 Incubation | L'idee a-t-elle une pause ou une raison de ne pas en avoir? |
| NR5 Human authority | La promotion reste-t-elle humaine? |
| NR6 Sandbox boundary | Aucune app sandboxee ne controle l'Operator? |
| NR7 Anti-ranking | Aucun score global n'est utilise comme gate? |
| NR8 LLM boundary | Le LLM ne sert ni de preuve ni de verdict final? |
| NR9 RRV labels | Les statuts Lu/Deduit/Verifie/Non verifie restent distincts? |
| NR10 Commons Purpose | Le besoin social, les beneficiaires et le dommage evite sont explicites? |
| NR11 License & Reuse | Licence/source/provenance/export/fork sont prevus? |
| NR12 Anti-Capture | Pas de leaderboard global, token, stake, kudos gate ou rente GPU? |
| NR13 Maintenance Reality | Porteur, prochaine action, cout et abandon propre sont documentes? |

## 8. Consequences pour la future app Ideas Hub

L'app future ne doit pas etre "Canny decentralise". Elle doit etre un Atelier
des communs lisible:

- vue `Graines`: capture rapide sans jugement;
- vue `Commun`: briefs incomplets et questions;
- vue `Incubate`: pauses, rappels, questions ouvertes;
- vue `Prises en charge`: porteurs responsables, engagements, abandons;
- vue `RRV`: preuves, labels, limites;
- vue `Options`: divergence structuree, dont ne pas construire;
- vue `Anti-capture`: rente, lock-in, donnees, GPU, leaderboard;
- vue `Reviews`: objections et dissent;
- vue `Decision locale`: arbitre humain, export Factory/ResearchPack;
- vues locales/curators: recommandations revocables, pas leaderboard global.

Les votes existants peuvent rester comme signal social faible, mais ne doivent
pas piloter la promotion. La promotion doit venir d'un pack: porteur
responsable + evidence + review + arbitrage.

## 9. Sources externes consultees

- Teresa M. Amabile, "Componential Theory of Creativity", Harvard Business
  School Working Paper 12-096, 2012:
  https://www.hbs.edu/ris/Publication%20Files/12-096.pdf
- Ut Na Sio, Thomas C. Ormerod, "Does incubation enhance problem solving? A
  meta-analytic review", Psychological Bulletin, 2009:
  https://pubmed.ncbi.nlm.nih.gov/19210055/
- David G. Jansson, Steven M. Smith, "Design fixation", Design Studies, 1991:
  https://cecas.clemson.edu/cedar/wp-content/uploads/2016/07/9-JanssonAndSmith1991.pdf
- Amy Edmondson, "Psychological Safety and Learning Behavior in Work Teams",
  Administrative Science Quarterly, 1999:
  https://journals.sagepub.com/doi/10.2307/2666999
- Anita W. Woolley et al., "Evidence for a Collective Intelligence Factor in
  the Performance of Human Groups", Science, 2010:
  https://www.science.org/doi/10.1126/science.1193147
- Lingfei Wu, Dashun Wang, James A. Evans, "Large teams develop and small teams
  disrupt science and technology", Nature, 2019:
  https://www.nature.com/articles/s41586-019-0941-9
- Peter M. Gollwitzer, Paschal Sheeran, "Implementation Intentions and Goal
  Achievement: A Meta-analysis of Effects and Processes", 2006:
  https://doi.org/10.1016/S0065-2601(06)38002-1
- Anil R. Doshi, Oliver P. Hauser, "Generative AI enhances individual
  creativity but reduces the collective diversity of novel content", Science
  Advances, 2024:
  https://www.science.org/doi/10.1126/sciadv.adn5290
- Elinor Ostrom, "Beyond Markets and States: Polycentric Governance of Complex
  Economic Systems", American Economic Review, 2010:
  https://www.aeaweb.org/articles?id=10.1257%2Faer.100.3.641

## 10. Decision de recherche

Recommandation: garder `idea_development_workflow_provider_neutral.md` comme
contrat workflow/provider, utiliser cette note comme couche "science + RRV",
et utiliser `idea_commons_workflow_anti_capture.md` comme cadrage principal.
La prochaine etape, si l'humain veut agir, n'est pas un sprint direct: c'est un
schema `ResearchPack` + prompts `prompts/idea/` + validator non-sprint qui
verifie les gates NR0-NR13.
