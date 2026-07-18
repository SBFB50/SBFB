# Workflow Agents - recherche de conception ultradeep

Statut: Research Pack hors sprint. Ce document ne cree aucun sprint actif,
n'emporte aucun verdict de phase et ne modifie aucun contrat protocolaire. Le
plan unifie en 36 phases A a AJ ajoute le 2026-07-17 reste une proposition a
faire verifier independamment avant kickoff, notamment par Claude.

Autorite documentaire: ce Research Pack est le successeur de conception du
blueprint historique `.planning/research/sprint82_workflow_engine/`. Le vieux
blueprint et son JSON brut restent des sources d'evidence, pas une base
d'architecture ou un dossier a activer sans revalidation contre ce document.
Pour le processus sprint, `docs/agent/PROCESS.md`,
`docs/agent/AGENT_SYSTEM.md`, les artefacts `.planning/active/` et les hooks Git
restent superieurs a ce Research Pack.

Document compagnon centre uniquement sur l'application et ses parcours produit:

- `.planning/research/workflow_hub_product_conception_2026-07-15.md`

Ce document global est l'autorite de conception et de sequencement. Le document
Workflow Hub detaille l'UX mais ne porte plus de sprint ou de sequence de phases
autonome. **Workflow Hub est l'app visible; Workflow Agents designe le chantier
global app + runtime + protocole + adaptateurs.**

Date de verification initiale: 2026-07-15.
Date de mise a jour du plan: 2026-07-17.

Objet: concevoir le chantier global **Workflow Agents**, dont l'application
visible **Workflow Hub** permet de creer, tester, executer, publier, installer
et reutiliser des workflows LLM et multi-agents connectes au protocole
Nexus/SBFB. Le workflow Factory complet est
le vertical fondateur et doit reproduire le vrai processus sprint Nexus
(Claude, Codex, reconciliation, gates, preuves, commit humain, carries et
audit). Ideas, Conception, Babel et les futures apps doivent ensuite pouvoir
s'appuyer sur cette base commune au lieu de recreer chacune leur propre moteur.

## 1. Correction de cadrage

La premiere version de cette recherche assimilait a tort le produit a un
Workflow Studio privilegie integre dans l'Operator. Ce cadrage ne correspond
pas au besoin PO precise ensuite.

L'application visible visee, Workflow Hub, est une **app SBFB de premier rang**:

- publiable et installable comme les autres apps du protocole;
- utilisable en plein ecran comme outil de conception et d'exploitation;
- capable de construire de vrais workflows LLM et agents;
- connectee aux modeles locaux, aux fournisseurs cloud autorises et au calcul
  distribue SBFB;
- reutilisable par Ideas, Conception, Factory, Babel et d'autres apps;
- sans obtenir pour autant l'autorite, les secrets ou les droits du noeud.

La correction structurante est donc:

```text
Workflow Hub = app SBFB sandboxee, plan de controle et interface auteur
Workflow Runtime = service Rust hote, plan d'execution et autorite locale
Prompt/Agent Pack = artefact portable, signable, publiable et installable
Protocole SBFB = distribution des packs, artefacts et calculs terminaux bornes
```

L'app n'est ni un simple viewer, ni un interpreteur autonome dans une iframe,
ni une route interne de l'Operator. Elle peut pre-valider un graphe, l'editer
et afficher son execution. Les decisions autoritaires sont prises par le
runtime hote a partir du meme artefact compile.

## 2. Verdict executif

### 2.1 Oui, le produit construira des workflows LLM et agents

Workflow Hub doit permettre de definir independamment:

- des `AgentDefinition` reutilisables;
- des prompts et schemas d'entree/sortie;
- des politiques de modele et de provider;
- des outils natifs, MCP ou adaptateurs metier;
- des handoffs entre agents;
- des memoires et contextes bornes;
- des budgets, timeouts, retries et limites d'iterations;
- des validations humaines;
- des evaluations et preuves de provenance.

Ces agents sont ensuite composes dans un graphe declaratif comprenant:

- sequences et branches;
- routage par regle ou classification structuree;
- parallelisme, fan-out et fan-in;
- boucles explicitement bornees;
- agent principal avec agents specialistes;
- transfert de controle entre agents;
- outils et appels de services;
- recherche dans des corpus;
- sous-workflows;
- attentes humaines ou evenementielles;
- production d'artefacts types.

Exemple de workflow Ideas:

```text
Saisie d'idee
  -> clarification
  -> grounding repo + recherche externe en parallele
  -> generation de plusieurs options
  -> agents critiques independants
  -> synthese
  -> arbitrage humain
  -> ResearchPack + DecisionPack
  -> handoff optionnel vers Conception ou Factory
```

### 2.2 Architecture recommandee

```text
+---------------------------------------------------------------------+
| Workflow Hub - app SBFB sandboxee, plein ecran                      |
|                                                                     |
| Bibliotheque | Canvas | Agents | Capacites | Runs | Evals           |
+-------------------------------+-------------------------------------+
                                | bridge type, identite app liee
                                v
+---------------------------------------------------------------------+
| Shell hote de confiance                                             |
|                                                                     |
| grants par app/pack | confirmations HITL | secrets | auth | quotas  |
+-------------------------------+-------------------------------------+
                                |
                                v
+---------------------------------------------------------------------+
| Workflow Runtime Rust                                              |
|                                                                     |
| compiler | reducer | scheduler | event store | inbox/outbox         |
| checkpoints | reprise | annulation | provenance | evaluation        |
+-----------+----------------------+----------------------+------------+
            |                      |                      |
            v                      v                      v
     modeles locaux         providers autorises    calcul SBFB borne
     Ollama/llama.cpp       Claude/OpenAI/etc.      Task/Result signes
            |
            +---- outils natifs / MCP / Factory / corpus ------------+
```

### 2.3 Pourquoi conserver l'iframe

Le fait que Workflow Hub soit un outil de developpement ne justifie pas de
faire de tout son code une partie privilegiee du shell.

| Option | Benefice | Risque | Verdict |
|---|---|---|---|
| App iframe qui execute tout | Publication simple | Secrets, outils et etat durable impossibles a proteger correctement | Refuse |
| Route privilegiee dans l'Operator | Acces facile au repo et aux providers | Grossit le TCB, reserve le produit a l'Operator, expose le shell aux dependances de l'editeur | Refuse comme produit principal |
| App sandboxee + runtime hote | App portable, moindre privilege, moteur durable et generique | Exige de vrais contrats bridge et grants | Recommande |
| UI dupliquee app + Operator | Peut couvrir tous les cas | Drift UX et semantique, double maintenance | A eviter |

Une iframe plein ecran peut porter un canvas React riche. La frontiere est
fonctionnelle, pas visuelle. Elle doit rester invisible pour l'utilisateur,
mais forte pour la securite.

Le chrome hote, et non l'iframe, doit afficher les confirmations qui concernent:

- secret ou credential;
- ecriture dans un workspace;
- commande shell ou terminal;
- acces reseau ouvert;
- donnees envoyees a un provider ou a un worker distant;
- publication, installation, mise a jour ou activation d'un pack;
- commit, push ou action destructive.

### 2.4 Principe de controle

```text
Le graphe deterministe controle le processus.
Les agents raisonnent dans des ilots agentiques bornes.
Un LLM ne peut jamais modifier le graphe compile, accorder une capacite,
changer une destination ou produire un verdict process autoritaire.
```

Cette approche reprend la convergence des frameworks modernes: workflows
explicites pour les etapes connues, agents seulement pour les problemes
ouverts, checkpoints durables et human-in-the-loop pour les effets sensibles.

### 2.5 Ambition MVP confirmee par le PO

Le MVP vise une base produit audacieuse, pas une preuve de concept reduite. Le
futur sprint porte l'objectif complet et peut comporter autant de phases que
necessaire. Il inclut des le depart:

- auteur visuel et accessible;
- moteur durable et multi-provider;
- calcul SBFB et artifacts;
- packs publies/installes/forkables;
- handoffs inter-apps;
- Ideas, Conception, Factory et Babel comme packs executables;
- securite, provenance, evals et exploitation;
- accessibilite maximale app + shell pour toutes les grandes familles de
  handicap, leurs combinaisons et les limitations temporaires.

Le statut reste pilote ferme tant que les preuves de production publique ne
sont pas acquises.

## 3. Questions auxquelles la recherche repond

1. Quelle est la frontiere exacte entre app, shell, runtime et protocole?
2. Quelles primitives permettent de couvrir Ideas, Conception, Factory et
   Babel sans construire quatre moteurs?
3. Comment representer des agents, handoffs, outils, memoires et sous-workflows
   sans lier Nexus a un framework commercial?
4. Comment reprendre un run apres crash sans doubler un effet externe?
5. Que peut distribuer le protocole actuel sans pretendre posseder un
   ordonnanceur de DAG distribue?
6. Comment publier et installer un workflow avec hash, signature, provenance,
   licences, modeles, outils et evals?
7. Quelles autorisations doivent etre appliquees avant d'exposer des methodes
   Workflow a une app sandboxee?
8. Quel scope floor audacieux prouve une base produit complete connectee au
   protocole?

## 4. Methode et niveau de preuve

La recherche croise quatre categories de preuves.

1. **Code vivant du depot**: bridge, manifest, CSP, stockage, Task/Result,
   provider router, provenance, recherche et publication.
2. **Documents Nexus structurants**: Prompt/Agent Packs, workflow Ideas,
   architecture Ideas/Factory, Babel, RRV et blueprint Workflow supersede.
3. **Prototype local**: pack inspector `sbfb-agent-packs-lab` et workflow Idea
   Hub construit dans le laboratoire Flowise local.
4. **Sources primaires externes**: LangGraph, OpenAI Agents SDK, Microsoft
   Agent Framework, Google ADK, Flowise, MCP, A2A, JSON/JCS, DSSE, in-toto,
   OpenTelemetry, OWASP et WCAG.

Niveaux utilises dans ce document:

- **EXISTANT**: code ou contrat present aujourd'hui;
- **CANDIDAT**: proposition de conception a valider avant gel;
- **MVP**: premiere version produit audacieuse et complete pour pilote ferme;
- **FUTUR**: ne doit pas etre vendu comme disponible au MVP.

## 5. Verite actuelle du depot

### 5.1 Aucun Workflow Runtime generique n'existe encore

Les noms `SBFBPromptAgentPack` et `SBFBWorkflow` apparaissent dans les notes de
recherche, mais pas dans un crate, un schema livre ou une commande executable.
Le blueprint historique etait deja supersede pour le slot S82. Son architecture
est desormais aussi remplacee par le present Research Pack; ses mesures et
fixtures restent des evidences a revalider au futur kickoff:

- `.planning/research/sprint82_workflow_engine/verification_blueprint.md:3-12`.

Ce Research Pack ne doit donc pas transformer des intentions documentaires en
capacites pretendument disponibles.

### 5.2 Le bridge est ferme mais ses declarations ne sont pas des grants

Le schema du host expose 16 verbes, dont `task_submit`, `task_result`, le
stockage, la recherche et la provenance:

- `web/src/bridge/protocol.ts:20-48`.

Le routeur verifie que le message provient bien de l'iframe suivie, puis
dispatch la requete avec le nom d'app fourni par le host:

- `web/src/bridge/useBridge.ts:126-164`.

Mais le manifeste ne contient qu'une liste `bridge.methods`, et le code indique
explicitement qu'elle est declarative, pas une frontiere runtime:

- `crates/sbfb-manifest/src/lib.rs:11-40`;
- `crates/sbfb-manifest/src/lib.rs:52-66`.

Le routeur ne verifie pas encore, pour chaque appel, l'intersection entre:

```text
methode demandee par l'app
intersect methode declaree par sa release installee
intersect grant d'installation/activation
intersect politique locale du noeud
intersect consentement requis pour l'effet courant
```

**Consequence P0:** il ne faut pas ajouter des verbes Workflow privilegies
avant d'avoir rendu cette intersection autoritaire dans le host.

### 5.3 `task_submit` est une feuille LLM, pas un workflow

L'app fournit actuellement un prompt, un modele et un type de tache. Le host
injecte le `project_id`; `task_result` est ensuite interroge par polling:

- `web/src/bridge/useBridge.ts:235-261`.

Le wire Rust possede une `Task` signee avec `task_id`, `task_type`, prompt,
system prompt, modele, priorite, `parent_task_id` et metadata string-only:

- `crates/nexus-core-rs/src/task.rs:153-202`.

`TaskEntry` et `ResultEntry` sont signes Ed25519:

- `crates/nexus-core-rs/src/task.rs:416-453`;
- `crates/nexus-core-rs/src/task.rs:541-570`.

Le resultat est principalement `result_text` et des metriques d'inference:

- `crates/nexus-core-rs/src/task.rs:463-537`.

`parent_task_id` donne une causalite indicative, mais iroh-docs est LWW et ne
fournit pas un journal ordonne de DAG. Il n'existe ni scheduler de graphe, ni
frontier, ni checkpoint, ni pause, ni reprise, ni cancel reseau, ni artifact
result type.

**Conclusion:** le noeud initiateur doit rester l'autorite du run. Le reseau
execute des feuilles de calcul bornees et retourne des recus signes.

### 5.4 Le stockage applicatif ne convient pas au journal des runs

Le stockage des apps est un KV persistant SQLite avec upsert, et seule l'app
`sbfb-ideas` est routee vers iroh-docs dans le MVP actuel:

- `crates/nexus-shell-daemon/src/storage_api.rs:5-7,27-32`;
- `crates/nexus-shell-daemon/src/storage_api.rs:293-339`;
- `crates/nexus-coordinator-rs/src/db.rs:140-145,947-988`.

Il est limite en ecriture et reste last-value, sans transaction multi-cles,
sequence d'evenements, outbox ou compare-and-swap. Il convient aux brouillons
de l'app, pas a une histoire d'execution autoritaire.

Le runtime doit posseder son store local explicite:

- runs et plans pinning;
- journal append-only sequence;
- snapshots/checkpoints;
- inbox/outbox transactionnels;
- effets et cles d'idempotence;
- approbations et signaux;
- artefacts par reference.

### 5.5 Le sandbox actuel est une base utile

Le contenu d'app recoit une origine opaque et ne peut ni ouvrir de connexion,
ni lancer de worker, ni imbriquer une iframe:

- `crates/nexus-core-rs/src/csp.rs:24-33`.

Cette contrainte est compatible avec Workflow Hub si:

- tout acces externe passe par un service hote type;
- les gros artefacts sont references, pas injectes dans `postMessage`;
- le canvas et l'editeur restent purement declaratifs;
- aucun `eval`, code importe, plugin arbitraire ou credential ne penetre dans
  l'iframe;
- les runs continuent meme si l'iframe est fermee ou rechargee.

### 5.6 Factory fournit des adaptateurs, pas un moteur

Le routeur Factory sait envoyer un prompt vers Claude, Ollama ou Network selon
un contrat `un prompt -> un stream`:

- `crates/sbfb-factory/src/provider_router.rs:63-109`.

Les sessions et journaux de l'Operator sont encore principalement en memoire,
et le gate sensible historique repose sur des sous-chaines de texte:

- `crates/sbfb-factory/src/operator_server.rs:59`;
- `crates/sbfb-factory/src/operator_server.rs:1436-1460`.

Ce gate reste utile pour du texte libre humain, mais ne peut pas autoriser des
nodes/outils. Les workflows exigent des capacites typees, compilees et
comparees a des grants.

### 5.7 Publication, recherche et provenance sont centrees sur les apps

Le depot sait actuellement cloner une app, valider son manifeste, construire
une archive, la hasher, la publier en blob et produire une provenance de
release. Il ne sait pas publier un `artifact_kind=prompt_agent_pack` ni indexer
les agents, workflows, schemas, modeles, outils et evals d'un pack.

L'endpoint de provenance verifie une release d'app. La provenance de run
`workflow -> plan -> nodes -> prompts -> modeles -> outils -> datasets ->
outputs` n'existe pas. Certaines etapes de feed/provenance sont best-effort et
`app_version` n'est pas dans toute l'enveloppe signee:

- `crates/nexus-coordinator-rs/src/provenance.rs:18-28,102-123`;
- `crates/nexus-shell-daemon/src/deploy.rs:355-370`.

Ces limites doivent etre fermees pour les packs avant de parler de publication
verifiable.

## 6. Lecons des frameworks actuels

### 6.1 Convergence utile

| Reference | Primitive utile | Lecon Nexus |
|---|---|---|
| LangGraph | etat type, routes, fan-out, checkpoint, interrupt, store | Separer checkpoint de run et memoire durable; effets avant reprise idempotents |
| OpenAI Agents SDK | `agent-as-tool` et `handoff` distincts | Les deux intentions ne doivent pas etre fusionnees dans un node ambigu |
| Microsoft Agent Framework | graphes types, fonctions + agents, checkpoint, HITL | Utiliser une fonction quand un agent n'est pas necessaire; jamais de serialization opaque executable |
| Google ADK | sequential, parallel, loop, graphes et sous-workflows | Meme couverture entre format, editeur et runtime |
| Flowise Agentflow V2 | UX de nodes, etat, branches, boucles, HITL | Reference produit et import limite; pas socle protocolaire |
| CrewAI | Flows deterministes autour de Crews autonomes | Bon modele d'ilots agentiques bornes |
| n8n | historique d'executions, retry et debug | L'exploitation est une fonction produit, pas un ecran secondaire |

La decision n'est pas de choisir l'un de ces frameworks pour le protocole.
Nexus doit standardiser un artefact portable et un contrat d'execution. Des
adaptateurs peuvent ensuite importer ou exporter des subsets verifies.

### 6.2 MCP et A2A ont des roles differents

MCP 2025-11-25 fournit des outils, ressources et prompts avec schemas. Les
annotations de risque d'un serveur ne sont pas une preuve de confiance. Nexus
doit appliquer ses propres classes d'effets, grants, quotas et confirmations.

A2A 1.0 fournit une interoperation future entre agents independants via Agent
Cards, Messages, Tasks et Artifacts. Ce n'est ni un format de DAG, ni un
scheduler Nexus, ni un remplacement du pack.

Position recommandee:

```text
Workflow graph Nexus = orchestration declarative locale et portable
MCP = adaptateur d'outils dans une activite autorisee
A2A = adaptateur futur vers un agent distant opaque et borne
Task/Result SBFB = calcul terminal signe du reseau Nexus actuel
```

### 6.3 Ce qu'il ne faut pas copier

- serialization native arbitraire telle que pickle;
- workflow dont l'editeur et le runtime comprennent des nodes differents;
- code JavaScript/Python importe et execute comme node;
- credentials ou IDs de prompts dependants d'un fournisseur dans le pack;
- registre central ou ranking global obligatoire;
- auto-update silencieux;
- memoire conversationnelle globale partagee par defaut;
- agent autonome qui s'accorde lui-meme des outils;
- trace presentee comme preuve de justesse du resultat.

## 7. Produit et parcours utilisateur

### 7.1 Espaces principaux

Workflow Hub doit etre un outil operationnel, pas une landing page. Les
espaces de premier niveau proposes sont:

| Espace | Fonction |
|---|---|
| Bibliotheque | workflows locaux, installes, publies, forks et templates |
| Editeur | canvas, outline et mode guide accessibles, inspecteur type et validation |
| Agents | registre d'agents reutilisables, versions, prompts et handoffs |
| Capacites | outils demandes, grants effectifs, destinations et donnees |
| Runs | executions live, timeline, checkpoints, couts et interventions |
| Evals | datasets, trajectoires attendues, regressions et comparaisons |
| Publication | preflight, diff, provenance, signature et activation |

### 7.2 Parcours de creation

1. Creer un workflow vide, partir d'un template ou forker un pack installe.
2. Definir les schemas d'entree et de sortie.
3. Selectionner ou creer les agents dans un registre separe du canvas.
4. Composer le graphe et relier les donnees par ports types.
5. Associer des outils et voir immediatement les capacites demandees.
6. Choisir une politique de placement: local, provider autorise, reseau SBFB.
7. Lancer la validation statique et corriger les erreurs accessibles depuis
   une liste, sans obligation de manipuler le canvas.
8. Executer un scenario de test avec donnees et budgets bornes.
9. Observer la timeline, les decisions, handoffs, outils et artefacts.
10. Promouvoir la version vers un pack signe, puis demander publication ou
    installation via une confirmation hote.

### 7.3 Parcours d'exploitation

L'ecran Runs doit permettre de:

- distinguer `running`, `waiting_human`, `waiting_external`, `completed`,
  `failed`, `cancel_requested`, `cancelled` et `outcome_unknown`;
- filtrer par workflow, version, app appelante, provider et statut;
- voir chaque tentative sans confondre retry et nouvelle execution;
- inspecter les entrees/sorties redigees selon sa permission;
- reprendre une interruption avec une decision explicite;
- annuler ce qui est annulable et expliquer ce qui ne l'est plus;
- forker un run depuis un checkpoint sans reecrire son historique;
- exporter un paquet de preuve sans exposer les secrets.

### 7.4 Utilisation par les autres apps

Une app ne doit pas appeler une autre iframe. Elle produit ou consomme un
artefact via le host, ou invoque une operation headless publiee par un pack.

```text
Ideas -> DecisionPack -> Conception -> ConceptionPack -> Factory
Babel -> TranslationBatch -> validation -> PublishedTranslation
App tierce -> OperationIntent -> Workflow Runtime -> ArtifactRef/Receipt
```

Une app UI-only n'est pas un service headless. Pour etre appelable, un pack
doit exporter explicitement une operation, ses schemas et une definition de
workflow compilable par l'hote. Un `CompiledPlan` target-runtime peut etre mis
en cache et atteste, mais ne remplace jamais la compilation/validation locale.

## 8. Workflows verticaux de reference

Les types de nodes ne doivent pas etre geles a partir d'une demo artificielle.
Ils doivent couvrir quatre fixtures goldens representant des contraintes tres
differentes.

### 8.1 Ideas

But: transformer une intuition en decision humaine documentee sans creer
automatiquement un sprint.

```text
IdeaInput
  -> ClarificationAgent
  -> parallel {
       RepoGroundingService
       ExternalResearchAgent
     }
  -> OptionGeneratorAgent
  -> fanout options
       -> SkepticAgent
       -> FeasibilityAgent
       -> CommonsAlignmentAgent
  -> join
  -> SynthesisAgent
  -> HumanArbitration
  -> ResearchPack
  -> DecisionPack
```

Invariants:

- l'app Ideas transmet de la donnee, jamais une autorite de controle;
- la recherche cite ses sources et distingue fait, inference et hypothese;
- les reviewers independants ne partagent pas leur brouillon par defaut;
- l'humain peut abandonner, approfondir ou promouvoir;
- `ResearchPack` n'est pas un verdict de sprint;
- le handoff Factory reste une intention typee.

Ancrage:

- `.planning/research/idea_development_workflow_provider_neutral.md:116-249`;
- `.planning/research/idea_development_workflow_provider_neutral.md:423-448`.

### 8.2 Conception

But: transformer un `DecisionPack` accepte en choix produit et techniques
auditables avant la fabrication.

```text
DecisionPack
  -> RequirementNormalizer
  -> parallel {
       ProductArchitectureAgent
       UXResearchAgent
       SecurityThreatAgent
       ProtocolImpactAgent
     }
  -> options + tradeoffs
  -> adversarial cross-review
  -> HumanDesignArbitration
  -> deterministic consistency checks
  -> ConceptionPack
```

`ConceptionPack` est un contrat candidat, pas encore implemente. Il devrait
contenir au minimum:

- probleme, utilisateurs, jobs et hors-perimetre;
- flux et etats produit;
- architecture et frontieres d'autorite;
- modele de donnees et contrats I/O;
- ADRs et alternatives rejetees;
- menace, confidentialite et data boundaries;
- exigences UX, accessibilite, i18n et performance;
- matrice d'acceptation et strategie de verification;
- impacts protocole et migrations;
- decisions humaines, inconnues et risques residuels;
- provenance de chaque source et contribution agentique.

Invariants:

- Conception ne fabrique pas silencieusement du code;
- une recommandation LLM n'est jamais une decision PO implicite;
- les options concurrentes sont conservees dans l'historique;
- les maquettes ou assets sont des artefacts par reference;
- le pack produit doit etre consommable par Factory sans prose fragile.

Il n'existe aujourd'hui ni app Conception ni schema officiel. Ce vertical doit
donc etre traite comme un nouveau contrat metier, pas comme un fait repo.

### 8.3 Factory

But: devenir le **workflow fondateur** du produit. Il consomme un
brief/conception et reproduit le vrai processus sprint Nexus de bout en bout,
sans inventer une version simplifiee de Factory. Le workflow orchestre; les
fichiers repo, hooks, gates, preuves et decisions humaines gardent l'autorite.

```text
FactorySprintWorkflow
  -> PreviousSprintAuditPhase0
  -> Kickoff + Plan + IndependentDesignReview(G1/G9)
  -> for each declared phase label {
       Preflight(S1a,S1b,S2,S3,S4)
       -> DriverImplementation
       -> ThreeVerificationBlocks
       -> IndependentReview(PASS-PENDING | CONCERN | FAIL)
       -> RawCodexVerification(CLEAN | GAP-P0..P3)
       -> ReconciliationAndBoundedRepair
       -> FinalReview("## Verdict: PASS")
       -> HumanConfirmedCommitIntent(9-section body)
     }
  -> Verification + CarryRouting(G6/G7)
  -> WrapUp + NextSprintAuditPlan
  -> OptionalHumanConfirmedPushIntent(at ratified human checkpoint)
  -> NextSprintAuditGate(Phase0, later)
```

Invariants:

- `process > RRV > Factory`; un prompt reste une entree non autoritaire;
- les 9 roles et 10 modes de `AGENT_SYSTEM.md` restent representables sans
  fusion silencieuse des responsabilites;
- le driver ne peut ni autoriser ni auto-attester `## Verdict: PASS`; il peut
  seulement materialiser la promotion depuis une reconciliation independante
  verifiee;
- la review independante reste `PASS-PENDING` jusqu'a la verification Codex;
- la sortie Codex brute n'est ni reecrite ni masquee par Claude ou le driver;
- toute correction P0/P1 reboucle par review puis Codex avant le verdict final;
- aucun commit, push, publication ou shell sensible n'est autonome;
- `commit.intent` et `push.intent` sont lies au HEAD, index, diff, gates et
  artefacts exacts, puis confirmes par un humain juste avant l'effet;
- le commit de phase conserve exactement les neuf sections imposees par le
  process, dont `## Codex verification` et la fermeture des carries;
- G1 a G10, FG0 a FG10, audit et hooks conservent exactement leur semantique
  extraite; seules les gates declarees bloquantes sont fail-closed, sans rendre
  bloquants G5 retire, confirmations/informations ou FG10 asynchrone;
- les sorties shell sont des artefacts non fiables, pas des instructions;
- un effet non idempotent dont l'issue est inconnue n'est pas relance en aveugle;
- le workflow ne remplace jamais les gates du repo et ne peut pas promouvoir
  seul une preuve ou un verdict;
- la bascule depuis la baseline legacy reelle -- Workflow Claude, driver
  main-thread, Codex CLI, hooks Git/`agentctl` et surfaces Factory concernees --
  exige shadow-runs, oracle de parite, kill switch, rollback teste et
  conservation temporaire de l'ancien chemin.

Ancrage:

- `docs/agent/AGENT_SYSTEM.md`;
- `docs/agent/PROCESS.md`;
- `docs/agent/RRV_FACTORY_CONTRACT.md`;
- `docs/factory/FACTORY_GATES.md`;
- `.planning/research/idea_hub_factory_ultracode_redesign.md:204-244`.

### 8.4 Babel

But: orchestrer un pipeline tres long, hautement parallele et fortement
provenance, melant calcul reseau, controles automatiques et travail humain.

```text
SourceCandidate
  -> RightsAndRedistributionGate
  -> SignedSourceManifest
  -> deterministic chunking
  -> fanout TranslationTask leaves on SBFB
  -> automatic validators
  -> human correction/review
  -> consensus attestation
  -> reassembly
  -> PublishedTranslation + provenance graph
```

Invariants:

- Gutenberg est le premier corpus, pas une dispense de verification des droits;
- chaque chunk conserve source, version, langue, modele et lineage;
- les workers distants ne recoivent que le chunk et le contexte minimal;
- les brouillons, validations, corrections et attestations sont distincts;
- la signature d'un worker prouve l'auteur du recu, pas la justesse linguistique;
- le workflow peut durer des jours et reprendre sans perdre le frontier;
- les gros textes vivent en blobs, pas dans le KV de l'app ni les events UI.

Ancrage:

- `.planning/research/babel_translation_protocol.md:44-135`.

### 8.5 Ce que les quatre goldens imposent

| Besoin | Ideas | Conception | Factory | Babel |
|---|:---:|:---:|:---:|:---:|
| LLM/agent local | oui | oui | oui | possible |
| Recherche/corpus | oui | oui | repo | massif |
| Parallele/fanout | oui | oui | oui | massif |
| Human-in-the-loop | decision | arbitrage | effets | qualite |
| Outils privilegies | non | limites | forts | ingestion/publication |
| Calcul reseau | option | option | option | central |
| Longue duree/reprise | utile | utile | requise | critique |
| Provenance fine | sources | decisions | diff/tests | chaque chunk |
| Artefact cross-app | DecisionPack | ConceptionPack | release/evidence | traduction |

Un vocabulaire de huit nodes, suffisant pour une ancienne demo, ne doit pas etre
gele avant que ces quatre fixtures compilent et s'executent avec les memes
semantiques.

## 9. Modele d'artefact portable

### 9.1 Conserver un seul artefact racine

La recherche precedente a deja propose `SBFBPromptAgentPack.v1`. Workflow
Agents doit etendre cette direction plutot que creer un format racine
concurrent appele `SBFBWorkflow.v1`.

Decision candidate:

```text
SBFBPromptAgentPack.v1          artefact racine publie
  workflow/workflow.json       SBFBWorkflowGraph.v1
  agents/*.json                SBFBAgentDefinition.v1
  prompts/*                    contenu versionne
  schemas/*.json               JSON Schema pinne
  capabilities/*.json          demandes, sans grants ni secrets
  evals/*                      datasets et specs
  provenance/*                 attestations et signatures
  layout/workflow.layout.json  presentation hors digest semantique
```

Le nom exact des schemas reste a ratifier. La decision importante est qu'un
workflow, ses agents, ses schemas et ses politiques soient publies ensemble et
verrouilles par digest.

### 9.2 Layout conceptuel

```text
SBFB.pack.json
PACK.lock
README.md
LICENSES/
SIGNATURES.json
provenance/
workflow/
  workflow.json
  subworkflows/
agents/
prompts/
schemas/
capabilities/
  requested.json
models/
  requirements.json
knowledge/
  manifest.json
evals/
  manifest.json
  datasets/
  scenarios/
layout/
  workflow.layout.json
```

Le pack ne contient jamais:

- secret ou token;
- endpoint prive avec credential;
- code executable non audite;
- grant accorde par un noeud;
- etat live d'un run;
- memoire personnelle d'un utilisateur;
- preuve mensongere de reproductibilite independante du modele et du provider.

### 9.3 Digests separes

Au minimum:

- `content_digest`: tous les contenus fonctionnels et locks;
- `graph_semantic_digest`: topologie et configuration executable;
- `presentation_digest`: positions, groupes et annotations visuelles;
- `compiled_plan_digest`: fermeture exacte compilee pour un runtime cible;
- `evaluation_digest`: dataset, scenarios et graders;
- `publication_digest`: enveloppe, licences et attestations publiees.

Deplacer un node sur le canvas ne doit pas changer la semantique executable.
Changer un prompt, un schema, une capacite, une borne ou un provider lock doit
la changer.

La canonicalisation inter-langages doit utiliser JCS/RFC 8785 ou le helper
canonique commun valide par fixtures Rust/TypeScript, avec domaines de
signature distincts.

### 9.4 Lockfile

`PACK.lock` doit pinner:

- digests des agents et sous-workflows;
- versions des schemas;
- prompts exacts;
- requirements de modele et compatibilites testees;
- snapshots des outils MCP, pas leur credential;
- Agent Cards A2A futures;
- corpus ou indexes de connaissance par digest;
- version du compilateur et profil d'execution;
- politiques d'evaluation.

Un changement dynamique de la liste d'outils ne devient jamais silencieusement
une capacite du plan deja compile.

## 10. Contrats de graphe et d'agent

### 10.1 `WorkflowDefinition`

Champs candidats:

```text
workflow_id
schema_version
workflow_version
input_schema_ref
output_schema_ref
entrypoints[]
nodes{}
edges[]
error_policy
timeout_policy
budget_policy
cancellation_policy
capability_requests[]
agent_refs[]
subworkflow_refs[]
artifact_outputs[]
```

Le schema doit refuser les champs inconnus dans le profil signe, valider les
references et compiler vers un `CompiledPlan` immuable.

### 10.2 `AgentDefinition`

```text
agent_id
version
display metadata
instructions_ref
input_schema_ref
output_schema_ref
model_policy
tool_refs[]
allowed_handoffs[]
context_policy
memory_policy
guardrail_refs[]
approval_policy
max_turns
token_budget
cost_budget
timeout
```

Une definition d'agent est reutilisable dans plusieurs workflows. Un node
`agent.invoke` fixe ensuite le contexte et les overrides autorises pour une
invocation precise.

### 10.3 Profil de nodes candidat

| Node | Role | Agentique? |
|---|---|---:|
| `input` | valider et introduire l'entree typee | non |
| `transform` | mapping declaratif pur | non |
| `model.invoke` | appel LLM structure sans boucle autonome | limite |
| `agent.invoke` | agent avec outils/handoffs bornes | oui |
| `agent.as_tool` | specialiste appele, controle conserve par le parent | oui |
| `handoff` | transfert explicite du controle et du payload | oui |
| `capability.call` | activite native/MCP/metier autorisee | non |
| `knowledge.query` | recherche dans corpus pinne | limite |
| `switch` | route deterministe sur donnees validees | non |
| `classifier` | route proposee par modele puis validee | limite |
| `parallel` | branches statiques concurrentes | non |
| `map` | fan-out dynamique borne sur collection | non |
| `join` | fan-in avec politique explicite | non |
| `repeat` | boucle avec condition et borne | variable |
| `human.review` | interruption et decision hote | humain |
| `subworkflow` | appel d'un plan pinne | variable |
| `wait.signal` | attente durable d'un signal type | non |
| `output` | valider et publier un artefact | non |

Cette liste n'est pas gelee. Les goldens doivent montrer quels nodes peuvent
etre fusionnes sans rendre leur risque ou leur semantique ambigus.

### 10.4 Handoff et agent-as-tool

Ces primitives repondent a des intentions differentes:

- `agent.as_tool`: le parent conserve la responsabilite de la synthese et des
  guardrails; le specialiste traite une sous-tache bornee;
- `handoff`: un agent ou une etape transfere explicitement le controle d'une
  portion du workflow a un autre agent autorise.

Le handoff contient un payload schema, filtre par la politique de contexte. Il
ne transmet jamais implicitement tout l'historique, toute la memoire ou tous
les outils du donneur.

### 10.5 Validation statique minimale

Le compilateur refuse:

- node ou sortie inaccessible;
- cycle non porte par un node de boucle borne;
- `map` sans limite de cardinalite;
- join dont les branches et la politique sont incoherentes;
- reference de schema, agent, prompt ou sous-workflow non pinnee;
- output non type;
- capacite demandee mais absente du manifeste du pack;
- provider incompatible avec le modele ou la sortie structuree;
- handoff non autorise par l'agent cible;
- secret literal;
- endpoint ou code executable interdit;
- budget enfant superieur au budget parent;
- publication sans provenance, politique HITL ou evaluation requise.

## 11. Etat, contexte et memoire

Un dictionnaire global mutable partage par tous les agents rendrait les grants,
la reprise et le debug impossibles a raisonner. Il faut separer:

1. **Input artifacts immuables**: donnees initiales par digest.
2. **Run state**: etat autoritaire du graphe et frontier courant.
3. **Node invocation context**: projection minimale pour une tentative.
4. **Agent private scratch**: travail temporaire, non partage par defaut.
5. **Conversation thread**: historique optionnel et explicitement borne.
6. **Long-term memory**: store local permissionne, separe des checkpoints.
7. **Knowledge corpus**: contenus adressables et provenance.
8. **Handoff payload**: structure explicite transmise au prochain agent.
9. **Artifacts**: sorties durables par reference et schema.

### 11.1 Regles de contexte

- minimisation par defaut;
- allowlist de champs et de classes de donnees;
- redaction avant provider ou worker distant;
- pas de memoire P2P personnelle;
- pas de partage automatique entre apps;
- retention et TTL explicites;
- provenance sur chaque source injectee;
- compaction visible et auditable, jamais reecriture silencieuse de l'histoire;
- limites de tokens et de taille avant chaque invocation.

### 11.2 Memoire et checkpoint ne sont pas synonymes

Le checkpoint permet de reprendre le calcul d'un run. La memoire conserve des
connaissances au-dela de ce run. Les deux ont des politiques de permission,
retention, suppression et export differentes.

Cette separation est confirmee par LangGraph, qui distingue checkpointers de
thread et stores cross-thread. Nexus doit conserver la distinction dans son
propre contrat plutot que copier l'API d'un framework.

## 12. Compilation et semantique d'execution

### 12.1 Une seule autorite semantique

L'iframe peut linter pour une reponse immediate, mais le resultat autoritaire
vient du compilateur Rust hote. Le meme pack doit alimenter:

- l'editeur;
- le validateur;
- le runtime;
- la publication;
- les evals;
- l'inspection d'un run historique.

Le compilateur produit un `CompiledPlan.v1` contenant:

- digest du pack source;
- digests exacts des workflows, agents, prompts, schemas et outils;
- fermeture des capacites requises;
- IDs d'occurrence uniques pour chaque node et sous-workflow;
- routes, joins et bornes normalises;
- budgets herites et attenues;
- politique de placement;
- politique de retry/timeout/cancel;
- version et profil du compilateur.

Le plan est immuable pendant le run. Modifier le brouillon cree un nouveau
plan; cela ne change jamais un run en cours.

### 12.2 Identifiants

```text
workflow_id          identite logique
pack_digest          version de contenu publiee/installee
plan_digest          plan compile exact
run_id               execution logique
node_instance_id     occurrence dans le run, y compris map/subworkflow
effect_id             effet logique stable entre retries
attempt_id            tentative physique, change a chaque retry
artifact_digest       sortie immutable
call_id               appel inter-app dedupliquable
```

`node_instance_id` doit encoder ou referencer le chemin d'execution complet,
pas seulement le `node_id` du template. Deux iterations ou deux enfants d'un
map ne doivent jamais partager la meme identite d'effet.

### 12.3 Etats de run et de node

Etats candidats de run:

```text
created
validated
queued
running
waiting_human
waiting_external
cancel_requested
completed
failed
cancelled
outcome_unknown
```

Etats candidats de node instance:

```text
pending
ready
dispatched
running
waiting
succeeded
failed_retryable
failed_terminal
skipped
cancelled
outcome_unknown
```

L'etat rendu est une reduction du journal. Il ne doit pas etre la seule source
de verite mutable.

### 12.4 Journal append-only

`RunEvent.v1` devrait contenir au minimum:

```text
run_id
sequence
event_id
event_type
recorded_at
plan_digest
node_instance_id?
effect_id?
attempt_id?
actor
input_refs[]
output_refs[]
decision_ref?
receipt_ref?
trace_context?
data_classification
```

Le payload doit rester compact. Prompts, sorties longues, corpus, diffs et
traces detaillees sont des `ArtifactRef`, recuperes a la demande selon les
permissions.

Le store doit garantir atomiquement:

```text
append event
+ update snapshot/frontier
+ enqueue outbox command
```

et, lors d'une reponse:

```text
dedupe inbox receipt
+ append result event
+ update snapshot/frontier
+ enqueue next commands
```

### 12.5 Livraison et idempotence

Le transport d'une activite est **au moins une fois**. La transition logique du
run doit etre appliquee **une fois** par `effect_id` ou receipt unique.

Regles:

- `effect_id` reste stable entre retries;
- `attempt_id` change a chaque tentative;
- un adaptateur idempotent recoit la cle d'effet;
- un resultat duplique ne produit pas deux transitions;
- une commande outbox peut etre reemise apres crash;
- un effet non idempotent sans recu apres perte de connexion devient
  `outcome_unknown`;
- `outcome_unknown` exige reconciliation ou decision humaine, pas retry auto;
- les operations avant une interruption humaine doivent etre idempotentes ou
  deja journalisees.

### 12.6 Pause, reprise et signaux

Une interruption stocke:

- le plan et le node exacts;
- le digest de la vue presentee;
- la decision attendue et son schema;
- les donnees affichees apres redaction;
- nonce, sequence et expiration;
- principal humain et politique requise.

La reprise ne recommence pas implicitement tout le node. Elle rejoue le reducer
jusqu'au checkpoint, consomme un signal mono-usage valide, puis emet les
prochaines commandes deterministes.

Fermer ou tuer l'iframe ne doit pas tuer le run. Redemarrer le runtime doit
restaurer le meme frontier sans reexecuter un effet deja confirme.

### 12.7 Cancel honnete

`cancel_requested` signifie que le runtime arrete d'emettre de nouveaux effets
et demande l'annulation aux adaptateurs qui la supportent. Une inference ou
operation deja partie peut encore retourner. Le journal conserve ce resultat
tardif mais ne le propage pas sans politique explicite.

Le protocole Task actuel n'a pas de cancel fort. Le produit doit afficher cette
limite au lieu de promettre une annulation instantanee.

## 13. Placement local, cloud et reseau SBFB

### 13.1 Autorite locale du run

Pour le MVP, un seul runtime local possede:

- le plan compile;
- le journal ordonne;
- le frontier;
- les timers;
- les decisions HITL;
- les grants et secrets;
- la resolution des effets;
- la provenance complete du run.

Le protocole transporte:

- packs et versions publies;
- artefacts content-addressed;
- annonces, recherche et curation;
- taches de calcul terminales;
- resultats et recus signes;
- plus tard, delegation a des agents distants opaques.

Il ne transporte pas au MVP:

- le journal live complet d'un run;
- les secrets;
- la memoire privee des agents;
- les grants locaux;
- un consensus global sur le frontier;
- un scheduler de DAG multi-maitre.

### 13.2 Politique de placement

Chaque node calculable peut demander une politique, jamais une destination
arbitraire:

```text
local_only
local_preferred
approved_provider
sbfb_network_allowed
sbfb_network_required
```

Le runtime resout cette politique selon:

- compatibilite du modele et du schema;
- disponibilite locale;
- consentement et classification des donnees;
- budget cout/latence/energie;
- confiance et politique du noeud;
- besoin de redondance ou validation;
- contraintes de licence et de juridiction.

### 13.3 Calcul reseau MVP

Un worker SBFB ne recoit qu'une feuille sans outil privilegie:

- prompt/system prompt minimises ou artefact public autorise;
- modele attendu;
- type de tache;
- deadline et budget compatibles avec le wire;
- identifiants de correlation opaques;
- classification deja autorisee;
- aucun secret, grant, historique complet ou capability token reutilisable.

Le runtime archive le `TaskEntry`, les claims pertinents et les `ResultEntry`
comme recus. Une signature prouve l'emetteur et l'integrite, pas la correction
semantique. Les validations et consensus restent des etapes separees.

Le wire actuel est trop etroit pour les artefacts riches. Le MVP peut mapper
des IDs de correlation de facon compatible, mais un contrat type futur doit
ajouter des references d'artefacts, schemas, deadlines, classifications et
attestations sans surcharger des metadata string-only.

### 13.4 Orchestrateur distribue futur

Repandre le frontier sur plusieurs noeuds exige au minimum:

- proprietaire/leader de run;
- epoch et fencing tokens;
- journal durable ordonne;
- leases et heartbeats;
- timers durables;
- inbox/outbox dedupliques;
- politique de partition et failover;
- migration de workers;
- revocation de delegation;
- resolution des divergences.

Ce chantier n'est pas necessaire pour prouver le chantier Workflow Agents
connecte au
protocole. Il doit rester explicitement FUTUR.

## 14. Capacites, grants et secrets

### 14.1 Calcul de l'autorite effective

```text
effective_capabilities =
    OS capability toggle
  intersect politique locale du noeud
  intersect grants accordes a la release de l'app
  intersect grants accordes a la version du pack
  intersect fermeture du plan compile
  intersect capacite necessaire a l'effet courant
  intersect decision HITL encore valide
```

Chaque couche ne peut qu'attenuer. Ni l'app, ni le pack, ni un agent, ni un
serveur MCP ne peut elargir l'ensemble.

### 14.2 Classes de capacites

Profil candidat:

- `read_public`;
- `read_private_scoped`;
- `write_draft`;
- `write_workspace`;
- `network_fixed_destination`;
- `network_open`;
- `remote_compute`;
- `model_local`;
- `model_cloud`;
- `tool_native`;
- `tool_mcp`;
- `secret_use`;
- `publish`;
- `install`;
- `commit`;
- `push`;
- `destructive`.

Chaque capability ref inclut schemas, destinations, classe d'effet, quotas,
timeout, politique de donnees et niveau d'approbation. Un nom d'outil seul ne
suffit pas.

### 14.3 Secrets

Le pack stocke uniquement une reference logique telle que
`secret_ref=github.publisher`. Le binding vers un credential reel appartient
au keystore hote et se fait juste avant l'appel autorise.

Un secret ne doit jamais apparaitre dans:

- iframe ou app KV;
- pack, lockfile ou layout;
- event log ou snapshot;
- prompt, contexte ou Task reseau;
- trace, metrique ou message d'erreur;
- ArtifactRef ou export de debug;
- dataset d'evaluation.

### 14.4 HITL autoritaire

L'app peut expliquer qu'une decision est attendue. Seul le chrome hote rend et
signe l'approbation. La decision est liee a:

- `run_id`, `plan_digest`, `node_instance_id`, `effect_id`;
- operation et destination exactes;
- resume ou digest des donnees sortantes;
- classe d'effet et cout maximal;
- principal, nonce, sequence et expiration;
- choix accorde ou refuse.

Modifier un parametre apres affichage invalide l'approbation.

### 14.5 Delegation

Le `DelegationCert` existant lie une identite SBFB a une cle de forge; ce n'est
pas un grant d'activite:

- `crates/nexus-core-rs/src/attestations/delegation.rs:2-20,108-151`.

Un futur `DelegationGrant.v1` devrait etre hote, un saut au MVP, avec sujet,
acteur, audience, operations, schemas, classes de donnees, budgets, expiration
et attenuation monotone. Il ne doit pas etre place dans les metadata publiques
d'une Task.

## 15. Contrats host et bridge candidats

### 15.1 Ne pas exposer une collection de super-pouvoirs bruts

Deux options doivent etre comparees au spike:

1. verbes bridge explicites par operation;
2. service commande/evenements unique avec envelopes typees et operation IDs.

Dans les deux cas, le host verifie l'identite de l'app, sa release installee,
ses methods declarees, ses grants et le plan cible avant dispatch.

Surface conceptuelle minimale:

```text
workflow.draft.list
workflow.draft.get
workflow.draft.put
workflow.validate
workflow.compile
workflow.run.start
workflow.run.get
workflow.run.events
workflow.run.signal_intent
workflow.run.cancel
workflow.artifact.get
workflow.pack.inspect
workflow.pack.install_intent
workflow.pack.publish_intent
```

Les suffixes `intent` signifient que l'appel ne vaut jamais approbation. Le
host peut afficher une confirmation, refuser ou exiger un grant supplementaire.

### 15.2 Push, replay et backpressure

Le polling de `task_result` ne suffit pas a une timeline de run. Le runtime a
besoin d'un abonnement live authentifie et borne, avec:

- curseur/sequence de reprise;
- catch-up par page;
- evenements compacts;
- limite de debit et coalescence des deltas LLM;
- unsubscribe a la fermeture;
- filtrage par app et grants;
- recuperation d'artefacts separee;
- aucun token ou secret dans l'URL.

Le transport exact peut etre SSE hote ou channel bridge gere par le shell. Il
ne doit pas modifier la source de verite, qui reste le journal local.

### 15.3 Appels inter-apps

Contrat candidat `AppCallEnvelope.v1`:

```text
call_id
caller_app_release
service_pack_digest
operation_id
input_artifact_refs[]
requested_budget
delegation_grant_ref
trace_context
depth
deadline
```

Le runtime cree un child-run idempotent et retourne un
`AppCallReceipt/ArtifactRef`. Il impose:

- schemas d'entree/sortie;
- allowlist d'operation;
- profondeur maximale;
- detection de cycle;
- budgets attenues;
- pas d'appel iframe-to-iframe;
- pas de transit de secret;
- provenance caller -> child run -> artefact.

## 16. Publication, installation et decouverte

### 16.1 Cycle de vie

```text
draft
  -> validated
  -> evaluated
  -> signed
  -> published
  -> indexed
  -> locally curated
  -> installed_disabled
  -> activated_with_grants
  -> forked / deprecated / revoked locally
```

Publication, installation et activation sont trois decisions distinctes.
Installer ne donne aucun droit. Activer cree un receipt local avec la version
exacte et les grants. Une mise a jour est une nouvelle version et repasse par
diff, evaluation et consentement.

### 16.2 Catalogue

La decouverte doit rester anti-capture:

- recherche par nom, domaine, schema, capacite, provider, langue et licence;
- curation locale ou communautaire explicite;
- preuve de provenance et evals visibles;
- aucune autorite centrale obligatoire;
- aucun ranking global presente comme verite;
- pas de token, pay-to-rank ou pouvoir par GPU;
- forks et continuations visibles;
- version et digest toujours affiches.

### 16.3 Publication minimum connectee au protocole

Pour meriter le terme connecte au protocole, un vertical MVP doit prouver:

1. build d'un pack immutable;
2. hash et signature de l'auteur/publisher;
3. stockage content-addressed;
4. annonce/indexation et recherche sur un autre noeud ou fixture reseau;
5. inspection avant installation;
6. installation desactivee;
7. activation avec grants locaux;
8. execution locale plus une feuille de calcul SBFB;
9. archivage des Task/Result receipts;
10. output type consommable par une autre app.

Sans ces dix points, le produit serait seulement un builder local avec un
bouton reseau, pas une base de workflows IA du protocole.

## 17. Provenance et attestations

### 17.1 Ne pas confondre les roles de signature

Il faut separer au moins:

1. signature de l'auteur du pack;
2. attestation du publisher reliant source et digest publie;
3. attestation du compilateur reliant pack et `CompiledPlan`;
4. receipt local d'installation et de grants;
5. attestation de run reliant plan, inputs, events et outputs;
6. receipts d'activites locales, providers, MCP et Factory;
7. `TaskEntry`, claim et `ResultEntry` des workers reseau;
8. attestations d'evaluation et de revue humaine.

Une cle autorisee a publier une app n'est pas automatiquement autorisee a
attester un resultat de modele ou une decision humaine.

### 17.2 Graphe de provenance d'un run

```text
Source pack
  -> PublishedPack digest
  -> InstalledPack + local grants receipt
  -> CompiledPlan
  -> Run input artifacts
  -> Node/Activity receipts
  -> Human decisions
  -> Output artifacts
  -> Eval/verification attestations
```

Chaque lien contient le type, les digests d'entree/sortie, l'acteur, le temps,
la politique et les limites connues.

### 17.3 Formats

Le contenu signe doit etre canonicalise et domaine-separe. Une enveloppe DSSE
et des statements in-toto sont de bons formats d'export interoperables, mais
ils ne remplacent pas le modele de confiance Nexus.

Le code actuel de `nexus-trace-core` ne doit pas etre repris comme primitive
canonique sans correction: certaines maps sont serialisees via `serde_json`
sans garantie JCS et l'integration OTel ouvre de nouvelles racines au lieu de
propager necessairement le parent:

- `crates/nexus-trace-core/src/lib.rs:31-41`;
- `crates/nexus-trace-core/src/signed.rs:48-65`;
- `crates/nexus-trace-core/src/otel.rs:22-31`.

### 17.4 Ce que la provenance ne prouve pas

- qu'un modele a raisonne correctement;
- que le texte est vrai;
- que la traduction est bonne;
- que l'outil distant etait honnete;
- que l'evaluation generalise hors dataset;
- que le provider a execute les poids annonces, sauf preuve independante;
- qu'une decision humaine est optimale.

Elle prouve qui declare avoir fait quoi, sur quelles entrees, avec quels
digests et quelles validations. La qualite exige des controles distincts.

## 18. Evaluation et observabilite

### 18.1 `EvaluationSpec`

Champs candidats:

```text
evaluation_id
dataset_digest
scenario_schema
initial_state
expected_output_constraints
expected_route_constraints
expected_handoffs
expected_tool_trajectory
forbidden_effects
deterministic_graders[]
llm_graders[]
human_review_policy
cost_budget
latency_budget
quality_thresholds
```

Les resultats sont toujours scopes par:

- pack et plan digest;
- provider et modele/version;
- tool/capability snapshot;
- dataset et date;
- runtime/compiler version;
- politique de sampling;
- environnement local/reseau.

Il n'existe pas de label global "meilleur agent" valable hors contexte.

### 18.2 Ordre des graders

1. schema et invariants deterministes;
2. route, handoff et trajectoire d'outils;
3. effets interdits et budget;
4. comparaison a une fixture ou oracle lorsque possible;
5. revue humaine;
6. grader LLM clairement identifie et non autoritaire.

Un LLM grader ne peut pas valider la securite, les droits Babel, un gate de
process ou sa propre conformite.

### 18.3 Observabilite

L'UI et les exports doivent relier:

- trace -> run -> node instance -> attempt -> effect;
- tokens, cout, latence et energie lorsque disponibles;
- queue, provider et worker;
- prompts et outputs par references proteges;
- tool calls, handoffs et guardrails;
- checkpoints, retries et decisions humaines;
- artefacts et digests de provenance.

OpenTelemetry GenAI peut fournir des noms d'attributs interoperables. Nexus
doit toutefois appliquer redaction, taille maximale et opt-in avant d'exporter
du contenu.

## 19. Securite - threat model

### 19.1 Adversaires

- pack importe malveillant;
- workflow legitime compromis par une mise a jour;
- prompt injection dans donnees repo, web, RAG ou app;
- serveur MCP ou agent A2A hostile;
- app appelante qui usurpe une operation;
- worker reseau qui ment ou rejoue un resultat;
- provider qui conserve des donnees;
- dependance frontend compromise;
- utilisateur presse qui approuve une vue trompeuse;
- crash entre effet externe et journalisation;
- artefact volumineux visant memoire, disque ou UI.

### 19.2 Import hostile

Avant tout rendu ou resolution:

- limite de taille, profondeur, cardinalite et longueur des chaines;
- parse structure, `deny_unknown_fields` pour le profil signe;
- aucune interpolation de chemin hors racine;
- aucun `eval`, script, template executable ou deserialization native;
- verification des digests et signatures avant import actif;
- scan des secrets et endpoints suspects;
- schemas refs locales ou pinnees;
- rapport `mapped/approximated/refused` pour import Flowise/autre;
- un import reste brouillon non signe jusqu'a revue et republication.

### 19.3 Prompt injection

Les prompts et donnees non fiables peuvent influencer une sortie de modele,
mais pas:

- la topologie compilee;
- la liste des outils;
- les destinations reseau;
- les grants;
- la classification des donnees;
- les schemas de sortie;
- la politique HITL;
- les budgets ou bornes de boucle;
- l'identite d'un publisher.

Le runtime valide tout tool call contre le node, le plan et les grants. Les
sorties d'outil sont revalidees avant reinjection dans un modele.

### 19.4 Donnees et calcul distant

Classification candidate:

```text
public
internal
private
secret
regulated_or_rights_restricted
```

Le dispatch distant applique une politique explicite par classe, destination,
juridiction et retention. `secret` ne part jamais. Les donnees privees ou
restreintes exigent redaction ou consentement precis. Les corpus Babel gardent
leur policy de droits independamment de la capacite technique a les envoyer.

### 19.5 SSRF, MCP et destinations

- destinations fixes pinnees lorsque possible;
- DNS/IP et redirects revalides par l'adaptateur hote;
- loopback, link-local, metadata cloud et reseaux prives bloques par defaut;
- pas de token passthrough;
- scopes minimaux et audience liee;
- output schema et taille verifies;
- tool annotations considerees non fiables;
- confirmation montrant destination, donnees et effet reels.

### 19.6 Ressources

Chaque plan et node porte des limites:

- tokens entree/sortie;
- cout et duree;
- iterations et profondeur de sous-workflow;
- cardinalite map/fanout;
- concurrence locale et distante;
- taille d'artifact/event;
- stockage et retention;
- appels outil/provider;
- retries et backoff.

Le runtime reserve les budgets enfants dans le budget parent pour eviter que
des branches concurrentes depassent collectivement la limite.

## 20. UX et accessibilite maximale, i18n et performance

Decision PO du 2026-07-15: l'accessibilite maximale fait partie du MVP
audacieux. Elle n'est ni une amelioration post-MVP, ni une phase cosmetique de
fin de sprint. Chaque phase qui ajoute une surface ou un parcours doit fermer
ses exigences d'accessibilite avant son commit de phase.

Le terme "pour tout handicap" est traduit en objectif testable et honnete:

- couvrir toutes les grandes familles de barrieres visuelles, auditives,
  motrices, de parole, cognitives, de lecture/apprentissage, neurologiques,
  vestibulaires, photosensibles, sensorielles et les combinaisons de besoins;
- couvrir aussi le vieillissement, les limitations temporaires et les
  situations d'usage contraignantes;
- ne demander aucun diagnostic et permettre de cumuler les preferences;
- viser l'autonomie complete sur les parcours, pas seulement une page qui
  passe un scanner automatique;
- tester avec des technologies d'assistance et avec des personnes handicapees;
- publier les limites residuelles au lieu de promettre une perfection
  universelle que WCAG lui-meme ne garantit pas, meme au niveau AAA.

Socle de conformite recommande:

- **WCAG 2.2 niveau AA bloquant** pour l'app, le shell hote et les processus
  complets, pas seulement des composants isoles;
- evaluation et fermeture de chaque critere **AAA applicable**, avec toute
  exception documentee, justifiee et arbitree;
- **ATAG 2.0 Part A niveau AA** comme objectif pour l'outil auteur lui-meme,
  applique avec la version courante de WCAG;
- **ATAG Part B niveau AA lorsque applicable** aux templates, formulaires,
  validations humaines et contenus visibles produits par Workflow Hub;
- guidance **COGA** obligatoire pour comprehension, memoire, attention,
  prevention/correction d'erreurs et personnalisation;
- **EN 301 549** comme matrice ICT europeenne complementaire: figer au kickoff
  la version publiee/harmonisee applicable et evaluer toute revision approuvee,
  sans confondre conformite technique et accessibilite reelle;
- patterns **WAI-ARIA APG** comme guidance informative pour les widgets riches,
  sans remplacer HTML natif ni les tests reels.

### 20.1 Canvas, outline et mode guide a parite

Le canvas ne peut pas etre l'unique moyen d'editer un workflow. Il faut une
vue outline/table accessible et un mode guide lineaire capables de:

- creer, renommer, configurer et supprimer un node;
- definir les connexions par selects types;
- ordonner et grouper les branches;
- afficher les erreurs et naviguer vers leur source;
- utiliser le clavier sans drag-and-drop;
- annoncer les changements via live regions sobres;
- conserver le focus apres validation ou mutation.

Les trois projections modifient le meme modele et appellent les memes commandes
typees, jamais des representations qui driftent.

### 20.2 Structure de l'editeur

```text
barre application: workflow/version/save/validate/run/publish
rail gauche: nodes, agents et sous-workflows
centre: canvas, outline ou mode guide lineaire
inspecteur droit: configuration typee, I/O, modele, outils, budgets
tiroir bas: erreurs, timeline, console et artifacts
```

Pas de cartes imbriquees decoratives. Les panneaux sont denses, redimensionnables
et adaptes a un outil de travail repete.

### 20.3 Signaux de confiance visibles

Chaque node affiche sans jargon inutile:

- local, cloud autorise ou reseau SBFB;
- donnees qu'il recoit;
- outils et classe d'effet;
- validation humaine requise;
- budget et timeout;
- version/digest de l'agent;
- statut live et nombre de tentatives.

La vue Publication montre le diff semantique separe du diff de layout.

### 20.4 i18n

- textes utilisateur en francais dans le produit initial;
- IDs, schemas et protocoles stables en anglais technique;
- aucune chaine runtime construite par concatenation fragile;
- messages d'erreur avec code stable et rendu localise;
- support RTL dans canvas, outline et inspecteur;
- noms fournis par un pack traites comme contenu, pas comme traduction UI;
- statuts descriptifs, jamais vocabulaire de verdict process reserve.

### 20.5 Performance

Budgets a mesurer au spike, pas a inventer:

- canvas charge en chunk lazy;
- virtualisation des bibliotheques et timelines longues;
- events LLM coalesces, pas un render par token;
- artefacts charges a la demande;
- layout calcule dans la limite permise par le CSP, sans worker dans l'app
  actuelle;
- graphe large avec niveaux de detail;
- index local des packs et runs;
- taille du vendor canvas mesuree puis gatee;
- aucune dependance runtime lourde dans le bundle initial du shell.

Un full-screen iframe ne doit pas provoquer de double scroll, rupture de focus
ou tailles instables. Ces points doivent etre verifies par Playwright desktop
et mobile, pas seulement par snapshots DOM.

### 20.6 Couverture par familles de barrieres

| Besoin/barriere | Exigences MVP |
|---|---|
| Cecite et braille | landmarks, titres, noms/roles/valeurs, ordre logique, descriptions textuelles du graphe, tableaux de donnees, statut non visuel, aucun contenu canvas-only |
| Basse vision | zoom 400 %, reflow a 320 CSS px, texte redimensionnable, focus fort, contrastes renforces, aucune perte sous magnification |
| Daltonisme/sans couleur | etats par texte + forme + icone, palettes color-safe et monochrome, forced colors, aucun sens porte par la couleur seule |
| Surdite/malentendance | aucun signal sonore seul, captions, transcription et description textuelle de tout media ou evenement audio |
| Moteur/dexterite | clavier complet, switch-compatible, commande vocale, single-pointer sans drag, grandes cibles, undo, pas de geste complexe obligatoire |
| Parole | aucune commande vocale obligatoire; toutes les fonctions restent clavier, pointer et assistive-tech |
| Cognitif/intellectuel | langage simple, structure stable, progressive disclosure, resume avant detail, aide contextuelle, actions reversibles, erreurs corrigibles |
| Dyslexie/lecture | choix de fonte lisible, espacement, largeur de ligne bornee, lecture assistee, pas de texte justifie ou tronque indispensable |
| Attention/neurodivergence | mode focus, densite reduite, pas d'autoplay, notifications groupees, reprise de contexte, controle des interruptions |
| Memoire/fatigue | autosave, historique, breadcrumbs, valeurs conservees, aucune obligation de memoriser un code/etat entre etapes |
| Vestibulaire | reduced motion reel, aucune animation de deplacement/zoom indispensable, transitions desactivables |
| Photosensibilite | aucune sequence au-dela des seuils de flash, pas de clignotement decoratif, mode statique |
| Sensoriel/migraine | themes calmes, transparence et effets retires, densite et contraste reglables, aucun fond anime |
| Handicaps combines | preferences empilables; la valeur la plus protectrice gagne par axe sans effacer les autres besoins |
| Temporaire/situationnel | responsive, touch, luminosite forte, une main, faible bande passante, reprise apres interruption |

Cette matrice s'applique a toutes les surfaces:

- catalogue, editeur, canvas, outline et mode guide;
- registre des agents et formulaires de schemas;
- timeline, logs, artefacts et evals;
- installation, activation et publication;
- dialogues du shell hote pour permissions, secrets, couts et donnees;
- erreurs, offline, reprise apres crash et etats vides;
- workflows embarques Ideas, Conception, Factory et Babel.

### 20.7 Reutiliser le systeme d'accessibilite du repo sans couplage Operator

Le repo possede deja un contrat livre dans Factory Operator:

- `tools/factory-operator/src/preferences/accessibility.ts:6-37` couvre basse
  vision, lecteur d'ecran, perception des couleurs, vestibulaire,
  photosensibilite, moteur, cognitif, dyslexie, attention, auditif, parole et
  sensoriel;
- `tools/factory-operator/scripts/check-accessibility-system.sh:1-72` gate le
  preboot, forced colors, motion, reflow, labels non-couleur et surfaces;
- `docs/factory/OPERATOR_ACCESSIBILITY_DESIGN_SYSTEM.md` definit des besoins
  empilables et la resolution vers l'etat le plus protecteur.

Workflow Hub doit extraire un contrat partage ou reproduire les memes
semantiques dans un module commun teste; il ne doit pas importer l'Operator.

Contrat hote -> app candidat:

```text
ResolvedAccessibilityProfile.v1
  theme
  contrast
  color_vision
  pointer_size
  text_spacing
  font_mode
  motion
  transparency
  density
  reading_assist
  focus_strength
  scale
  target_size
  shortcuts
  live_verbosity
  captions
```

Regles de confidentialite:

- le host transmet des **preferences de rendu resolues**, jamais un diagnostic;
- `needs[]` reste local et n'est ni envoye a l'iframe, ni publie, ni trace;
- aucun libelle medical ou identifiant de besoin n'apparait dans le DOM, les
  logs, prompts, runs, exports, telemetrie ou messages bridge;
- aucune preference d'accessibilite ne part vers un provider ou worker SBFB;
- la semantique accessible est active par defaut; aucun "mode lecteur d'ecran"
  et aucune detection d'une technologie d'assistance ne sont requis;
- les preferences systeme, hote et app sont empilees deterministiquement;
- l'utilisateur peut surcharger une preference sans devoir declarer un
  handicap;
- le profil s'applique avant l'affichage interactif pour eviter flash,
  mouvement ou theme dangereux au demarrage;
- les changements du panneau hote sont propages a l'app sans reload et sans
  perte de focus;
- synchronisation et export des seules preferences fonctionnelles resolues sont
  des opt-in explicites, revocables et effacables; `needs[]` et diagnostics ne
  sont jamais exportables ni synchronisables.

L'iframe garde une option locale accessible, mais le chrome hote reste la
surface de secours si l'app ne demarre pas.

### 20.8 Parite fonctionnelle sans drag, vision ou pointer precis

Le canvas est une projection visuelle. L'outline semantique et le mode guide
lineaire sont deux interfaces completes de premiere classe, pas des fallbacks
degrades.

Toutes les actions semantiques ont cinq chemins lorsque pertinents:

1. drag/pointer pour les utilisateurs qui le souhaitent;
2. single-pointer sans drag;
3. clavier/switch avec focus previsible;
4. outline/formulaires compatibles lecteur d'ecran et commande vocale;
5. parcours guide lineaire a une etape visible a la fois.

Concretement:

- ajouter un node par bouton/menu rechercheable;
- connecter deux ports par `Source` + `Destination` et bouton `Connecter`;
- deplacer/reordonner par boutons et menus, pas seulement drag;
- decrire chaque node, port, branche, join et erreur en texte;
- offrir vue arborescente, liste d'aretes et resume du chemin actif;
- annoncer les mutations structurelles, mais jamais chaque token LLM;
- permettre pause/reprise des annonces live et consulter un journal groupe;
- conserver/restaurer le focus apres ajout, suppression, undo ou dialogue;
- rendre tout raccourci remappable/desactivable et eviter les touches simples
  actives par defaut;
- faire contenir le libelle visible dans le nom accessible de toute commande
  pour la reconnaissance vocale;
- garantir une alternative single-pointer distincte du seul clavier, conformement
  a WCAG 2.5.7;
- viser 44 x 44 CSS px pour les controles principaux et ne jamais descendre
  sous le minimum/espacement WCAG 2.2.

Les graphiques, timelines, couts, diffs et preuves ont toujours une table, une
liste structuree ou un resume textuel equivalent exportable. Le canvas,
l'outline et le mode guide produisent le meme digest semantique; seules les
metadonnees de layout peuvent differer.

### 20.9 Accessibilite cognitive et controle de l'IA

Les workflows IA ajoutent des barrieres specifiques: incertitude, sorties
longues, changements d'etat et couts difficiles a anticiper.

Exigences:

- expliquer en langage simple ce que va faire le run avant lancement;
- afficher et lire une etape a la fois en mode focus;
- separer clairement demande utilisateur, proposition IA, preuve et decision;
- fournir resume, detail et donnees brutes comme niveaux distincts;
- ne jamais changer de vue ou lancer un effet sur simple focus/input;
- demander seulement les informations necessaires et reutiliser celles deja
  fournies;
- permettre brouillon, pause, retour, undo et correction sans tout recommencer;
- expliquer une erreur avec cause, impact et prochaine action;
- conserver l'orientation apres reconnexion ou reprise de checkpoint;
- ne jamais imposer un timer court a une approbation;
- afficher cout, donnees sortantes, destination et risque sous une forme courte
  puis extensible;
- offrir un contact/aide et un moyen simple de signaler une barriere;
- tester les textes et parcours avec des personnes ayant des handicaps
  cognitifs et d'apprentissage, pas seulement avec une formule de lisibilite.

### 20.10 Shell hote et frontiere iframe accessibles

L'accessibilite de l'app ne suffit pas si une approbation privilegiee est
inutilisable.

Le shell doit:

- donner a l'iframe un titre et un nom accessibles;
- offrir une navigation claire entre chrome hote et app;
- rendre l'iframe inactive pendant une modale hote, placer le focus dans la
  modale puis le restaurer exactement;
- ne jamais creer de keyboard trap et offrir annulation/retour accessibles;
- rendre action, permissions, donnees, destination, effet, cout, reversibilite
  et alternative `Refuser` en texte simple et structure;
- accepter password managers et collage pour l'authentification;
- ne pas utiliser CAPTCHA cognitif, memorisation ou geste comme unique moyen;
- ne pas imposer type-to-confirm, calcul, code memorise ou delai court pour une
  approbation;
- heriter du meme profil contraste/motion/echelle que l'app;
- fonctionner a 400 % de zoom, forced colors et lecteur d'ecran;
- proposer une confirmation non temporelle et un receipt consultable;
- garder la file d'approbations utilisable si l'iframe est fermee ou en panne;
- rester utilisable si le canvas ou JavaScript de l'app echoue.

### 20.11 Matrice de verification

**Automatique a chaque phase frontend:**

- lint semantique et composants natifs en priorite;
- tests de roles, noms, descriptions, erreurs et focus;
- axe-core sur chaque vue, dialogue, theme, statut et viewport principal;
- Playwright clavier-only sur les parcours modifies;
- emulation reduced motion, forced colors, contraste, zoom/reflow et RTL;
- tests de parite canvas/outline/mode guide sur le meme modele;
- scan anti-flash, anti-couleur-seule et anti-raccourci non protege;
- gate du profil d'accessibilite et absence de `needs[]` dans traces/reseau.

**Manuel avant chaque jalon produit:**

| Plate-forme | Combinaisons bloquantes |
|---|---|
| Windows 11 | NVDA + Firefox ESR/Chrome; JAWS + Edge; afficheur braille; Forced Colors; Magnifier 200/400 % |
| Windows interaction | Voice Access et Dragon; clavier seul; switch representatif; touch/coarse pointer sans drag |
| macOS | VoiceOver + Safari; Zoom; Voice Control; Switch Control |
| iOS/iPadOS | VoiceOver + Safari; Zoom; texte agrandi; orientation; Switch Control |
| Android | TalkBack + Chrome; taille texte/affichage maximale; clavier externe et switch |
| Ubuntu LTS | Orca + Firefox ESR; clavier seul; contraste eleve |

Chaque combinaison couvre aussi reduced motion, monochrome, captions et
l'absence d'information transmise seulement par son, couleur ou mouvement.
Les versions courante et precedente, ou ESR, sont figees au kickoff afin que
les preuves restent reproductibles.

Une matrice plate-forme peut etre separee entre CI, laboratoire et acceptance
humaine. Une combinaison non disponible localement doit etre executee sur une
machine/participant adapte avant closure, pas simplement marquee "non testee".

**Tests avec utilisateurs:**

- participants handicapes remuneres et impliques des prototypes au release
  candidate;
- au moins douze participants distincts en deux rounds, couvrant cecite/braille,
  basse vision, moteur/voix/switch, audition, cognition/apprentissage,
  vestibulaire/sensoriel et besoins combines;
- consentement et compte rendu accessibles, pauses, interprete/accompagnant et
  technologie personnelle pris en charge selon le besoin;
- scenarios complets decouverte -> installation/activation -> auteur -> run ->
  HITL -> reprise -> publication;
- collecte de taux de completion, erreurs, temps, fatigue, comprehension et
  sentiment de controle;
- barriere bloquante corrigee puis retestee avant closure;
- retour accessible par plusieurs canaux.

### 20.12 Definition de done accessibilite

Le MVP ne ferme pas tant que:

- zero echec connu WCAG 2.2 A/AA sur toute surface, composant, etat ou processus
  complet inclus dans le MVP;
- chaque critere AAA applicable a une decision et une preuve;
- la matrice EN 301 549 applicable, versionnee au kickoff, ne contient aucun
  ecart P0/P1 non resolu et archive chaque clause non applicable;
- zero violation axe critique/serieuse non resolue;
- mille sequences de mutations generees conservent un digest semantique
  identique entre canvas, outline et mode guide;
- toutes les actions canvas ont une alternative outline, mode guide, clavier et
  single-pointer;
- WA-T1/WA-T2/WA-T3 et le parcours accessibilite WA-T4 passent dans l'app et le shell;
- aucune information ou action ne depend uniquement de couleur, position,
  son, mouvement, drag, hover ou memorisation;
- 400 % zoom, 320 CSS px, text spacing, forced colors, RTL et reduced motion
  ne font perdre ni contenu ni commande;
- les tests lecteurs d'ecran, braille, switch, commande vocale, touch et clavier
  sont archives avec OS, navigateur, AT et version;
- les tests avec utilisateurs handicapes n'ont plus de barriere P0/P1 sur un
  parcours critique;
- un rapport de conformite/accessibilite versionne documente preuves,
  exceptions, limites connues et plan de correction;
- les preferences restent privees et fonctionnent avant, pendant et apres le
  chargement de l'iframe; seules des valeurs fonctionnelles resolues peuvent
  etre exportees/synchronisees apres opt-in explicite.

## 21. Architecture de code candidate

### 21.1 `crates/sbfb-workflow-core`

Crate pur, sans Tokio, reseau, DB ou provider:

- schemas pack/graphe/agent;
- canonicalisation et digests;
- validation statique;
- compilation vers plan normalise;
- import/export de subsets declaratifs;
- fixtures et golden traces;
- generation de JSON Schema.

### 21.2 `crates/sbfb-workflow-engine`

Runtime hote-neutre:

- reducer et machine d'etat;
- scheduler local;
- retries, timers, joins et loops;
- inbox/outbox et idempotence;
- checkpoints et replay;
- traits store, clock, activity, artifact, signal, secret et policy;
- events et receipts;
- aucun couplage UI.

### 21.3 Integration hote

Le daemon ou un companion service de confiance doit fournir:

- SQLite/migrations du run store;
- auth app/release/grants;
- bridge commande/evenement;
- blob/artifact store;
- publication/indexation des packs;
- adaptateur compute SBFB;
- HITL et secret store;
- appels inter-apps.

Le placement exact daemon vs service compagnon est une decision de kickoff.
Le daemon ne doit pas dependre de toute la Factory.

### 21.4 Adaptateurs

- `ModelAdapter`: Ollama, Claude, OpenAI et autres providers autorises;
- `NetworkComputeAdapter`: Task/Result Nexus;
- `FactoryAdapter`: repo, diff, tests, PTY et operations sensibles;
- `McpToolAdapter`: tools/resources/prompts apres policy;
- `KnowledgeAdapter`: corpus et recherche;
- `AppOperationAdapter`: child-runs host-medies;
- `A2aAdapter`: futur, agents distants opaques.

### 21.5 App Workflow Hub

App React sandboxee:

- editeur de brouillons;
- bibliotheque et catalogue;
- registre d'agents;
- validation locale non autoritaire;
- suivi live via bridge;
- evals et comparaison;
- diff et intentions publication/installation;
- aucune cle, DB de run ou execution d'outil.

## 22. Perimetre du MVP audacieux et complet

### 22.1 Definition PO

Decision PO du 2026-07-15:

```text
Un seul sprint porte la totalite de l'objectif Workflow Agents.
Le nombre de phases n'est pas plafonne.
Une phase ordonne et prouve le travail; elle ne sert pas a reduire arbitrairement
le produit a un canvas ou a repousser les fonctions centrales en S+1.
```

Ici, MVP signifie **premiere version produit complete utilisable en pilote
ferme**, pas demo technique minimale. Elle doit etre assez audacieuse pour
devenir reellement la base commune des workflows IA du protocole.

Le cycle a livrer entierement est:

```text
discover/fork -> author agents + graph -> validate -> compile -> evaluate
  -> inspect grants/data/cost -> run local/cloud/network
  -> pause -> kill/restart -> resume -> human decisions
  -> typed artifacts -> cross-app handoff
  -> sign -> publish -> index -> install disabled -> activate with grants
  -> observe -> compare -> deprecate/fork without silent update
```

L'audace porte sur la completude et la qualite, pas sur une promesse fausse de
scheduler mondial. L'autorite du run reste locale au MVP; les workers SBFB
executent des feuilles bornees et signees.

### 22.2 Onze piliers non negociables

#### Pilier A - Produit auteur complet

- app SBFB sandboxee plein ecran;
- bibliotheque, recherche, templates, fork et versions;
- canvas visuel riche;
- outline/table/formulaire et mode guide lineaire a parite fonctionnelle;
- inspecteurs types, ports I/O et schemas;
- undo/redo, autosave, diff semantique/layout et validation continue;
- import declaratif borne de Flowise apres validation hostile;
- export du pack source lisible et auditable.

#### Pilier B - Agents et orchestration

- registre d'agents versionnes separe du canvas;
- prompts, model policy, I/O, outils, contexte, memoire et budgets;
- `model.invoke`, `agent.invoke`, `agent.as_tool` et `handoff` distincts;
- sequence, switch, classifier, parallel, map, join et repeat borne;
- human review, wait/signal, subworkflow et artifact output;
- contexte minimal et handoff payload explicite;
- aucun LLM autorise a modifier le plan ou ses capacites.

#### Pilier C - Runtime durable

- compilateur/reducer Rust autoritaire;
- SQLite avec event log, checkpoints, inbox/outbox et migrations;
- IDs run/node/effect/attempt stables;
- retries, timeouts, budgets, timers et backpressure;
- kill/restart/resume sans double effet;
- cancel honnete et `outcome_unknown` pour les effets ambigus;
- runs independants de la vie de l'iframe;
- timeline live et replay par sequence.

#### Pilier D - Modeles et outils

- Ollama/local comme chemin autonome;
- au moins un provider cloud reel autorise par secret ref pour l'acceptance,
  derriere un adaptateur provider-neutral et un double hermetique sans
  credential dans CI;
- compute SBFB pour les feuilles LLM distantes;
- outil hote borne et schema;
- MCP comme adaptateur permissionne pour un subset utile, sans token
  passthrough;
- `AgentSessionAdapter` pour les sessions de travail et adaptateurs explicites
  Claude Code et Codex CLI, avec version, modele, prompt/input/output digests,
  exit code et sortie brute dans leurs receipts;
- independance driver/reviewer/Codex enforcee par roles, contexte et famille de
  provider, avec tout fallback degrade rendu visible et arbitre;
- structured outputs et validation de schemas;
- selection local/cloud/reseau visible avant lancement.

#### Pilier E - Securite et autorisation

- enforcement runtime des methods declarees par release d'app;
- grants distincts app/pack/plan/effet;
- capabilities attenuees et testees par proprietes;
- secrets par refs opaques et keystore hote;
- data classification et consentement d'egress;
- HITL rendu par le chrome hote;
- import hostile, anti-injection et anti-SSRF;
- aucune action commit/push/publish/destructive autonome.

#### Pilier F - Protocole et artefacts

- `SBFBPromptAgentPack.v1` racine et graph/agents schemas;
- ArtifactRef content-addressed pour les gros outputs;
- Task/Result signes archives dans le lineage;
- publication et recuperation multi-noeuds;
- recherche/indexation des packs, workflows, agents et versions;
- provenance pack -> plan -> run -> effects -> outputs;
- limites courantes du wire affichees sans surpromesse.

#### Pilier G - Cycle de vie du pack

- draft, validation, eval, signature et publication;
- inspection avant installation;
- installation desactivee;
- activation avec grants explicites;
- version pinning, diff, rollback et fork;
- update elargissant les droits bloque sans nouveau consentement;
- depreciation et revocation locale; une version revoquee reste auditable mais
  ne peut plus demarrer un nouveau run;
- aucune mise a jour silencieuse;
- licences et sources visibles.

#### Pilier H - Inter-apps

- operations headless publiees par schema;
- child-runs idempotents et dedupliques;
- handoffs par ArtifactRef, jamais iframe-to-iframe;
- budgets, depth, cycles et caller identity verifies;
- chaine fonctionnelle Ideas -> Conception -> Factory;
- Babel consommateur du meme runtime sans moteur parallele.

#### Pilier I - Quatre packs metier utilisables

- **Ideas**: intake, clarification, grounding, recherche, options, reviewers,
  synthese, arbitrage humain, ResearchPack et DecisionPack;
- **Conception**: architecture, UX, securite, protocol impact, cross-review,
  arbitrage et ConceptionPack;
- **Factory**: workflow fondateur reproduisant audit Phase 0, kickoff, plan,
  design review G1/G9, preflight S1a-S4, implementation, trois blocs de
  verification, review independante `PASS-PENDING`, Codex brut,
  reconciliation/reparation, verdict final `PASS`, intentions commit/push
  confirmees humainement, body neuf sections, carries G6/G7, verification et
  audit/wrap-up, dans un workspace isole;
- **Babel**: rights gate, source manifest, chunking, fanout compute,
  validation/correction, crash/reprise, reassembly et provenance; acceptance CI
  sur au moins 100 chunks hermetiques et dogfood borne sur deux workers;

Ce sont des reference packs reellement executables et forkables, pas quatre
captures d'ecran. Factory est le premier golden et fournit les contraintes du
moteur avant le gel du langage. Leur taille de donnees reste bornee pour
l'acceptance, mais leur pipeline et leurs frontieres sont complets.

#### Pilier J - Evaluation, provenance et exploitation

- datasets/scenarios versionnes;
- sorties, routes, handoffs et trajectoires d'outils evalues;
- graders deterministes avant LLM graders;
- comparaison version/model/provider/cout/latence;
- timeline, retries, decisions et artifacts inspectables;
- export de preuve redige et signe;
- mutation d'un digest, receipt ou lien de lineage rejetee;
- retention, suppression et limites documentees;
- aucun badge de qualite global hors contexte.

#### Pilier K - Accessibilite maximale et i18n

- exigences completes du chapitre 20 sur app **et** shell hote;
- WCAG 2.2 AA bloquant, AAA applicable evalue, ATAG et COGA;
- parite canvas/outline/mode guide, keyboard, switch, voice et single-pointer;
- profils empilables sans diagnostic partage;
- technologies d'assistance et tests avec utilisateurs handicapes;
- francais complet, architecture i18n, RTL et textes dynamiques;
- aucun parcours central reporte pour cause d'accessibilite.

### 22.3 Scope floor produit

Le sprint n'est pas considere complet si l'un de ces resultats manque:

- un utilisateur non technique peut creer et comprendre un workflow sans
  modifier du JSON;
- un utilisateur expert peut inspecter schemas, digests, capabilities et
  traces sans perdre la vue produit;
- le meme contrat fonctionne avec Ollama local, un provider cloud reel autorise
  et une feuille SBFB, sans permettre a `local_only` de sortir du noeud;
- un run survit a un crash et ne double pas ses effets;
- une validation humaine sensible est accessible et liee au digest exact;
- un output type passe d'une app a une autre;
- un pack est publie, trouve, inspecte, installe puis active sur un autre noeud;
- fork, update avec re-consentement, depreciation et revocation locale sont
  prouves sans mise a jour silencieuse;
- les quatre reference packs passent chacun un E2E complet et pas seulement une
  compilation de fixture;
- les preuves de securite, provenance, evaluation et accessibilite sont
  archivees;
- le produit reste utilisable sans vision, sans souris, sans drag, sans son et
  avec les modes cognitifs/motion/contraste combines.

### 22.4 Hors MVP explicite

Ces exclusions preservent une frontiere de confiance ou un chantier distribue
distinct; elles ne sont pas des coupes de confort:

- scheduler de DAG distribue et failover multi-leader;
- replication live P2P du run state prive;
- co-edition temps reel du canvas;
- execution de code importe ou plugin frontend tiers;
- marketplace centrale, ranking global ou token governance;
- auto-update;
- Agent Cards A2A comme autorite de confiance;
- memoire personnelle P2P;
- garantie de cancel fort d'une Task reseau que le wire ne sait pas annuler;
- import total et executable de Flowise/n8n/LangGraph;
- optimisation automatique de prompts sans dataset et consentement;
- agents qui commit/push/publient sans decision humaine;
- promesse de production publique avant les preuves de pilote ferme.

### 22.5 Criteres d'acceptation flagship

Le prefixe `WA-` separe ces acceptations produit des tiers de testabilite
`T0..T3` du processus repo et des sous-acceptations `HUB-T*` de l'app.

**WA-T1 - Auteur et runtime durable local**

1. creer plusieurs agents et un workflow depuis l'app sans JSON;
2. utiliser canvas puis outline sur le meme graphe et obtenir le meme digest;
3. valider une erreur de schema, boucle et capability puis les corriger;
4. lancer avec Ollama ou backend hermetique;
5. appeler un outil hote schema avec un effet idempotent;
6. atteindre `waiting_human`;
7. fermer l'iframe, tuer et redemarrer le runtime;
8. retrouver le meme run/frontier et approuver dans le chrome hote;
9. terminer sans reexecuter l'effet precedent;
10. comparer deux versions du run et verifier artifacts/events/provenance;
11. verifier l'absence du secret canari partout.

**WA-T2 - Protocole multi-noeuds**

1. signer et publier un pack depuis le noeud A;
2. le rechercher, verifier et installer desactive sur le noeud B;
3. afficher le diff de capabilities puis activer avec grants minimaux;
4. executer le meme contrat avec Ollama reel, un provider cloud reel autorise
   et une Task SBFB distante, tout en prouvant que `local_only` ne sort jamais;
5. archiver Task/claims/Result signes et signaler leurs limites de preuve;
6. joindre les sorties dans un ArtifactRef type;
7. consommer cet artefact depuis une autre app;
8. verifier provenance source -> pack -> plan -> run -> worker -> output;
9. publier une nouvelle version sans auto-updater le noeud B;
10. bloquer son activation si les droits augmentent jusqu'au nouveau
    consentement explicite;
11. forker, deprecier puis revoquer localement la version precedente sans
    casser son audit historique ni autoriser un nouveau run;
12. effectuer un rollback explicite vers une version epinglee encore autorisee,
    avec diff, grants valides et historique intact.

**WA-T3 - Chaine metier cross-app**

1. Ideas execute recherche sourcee, avis independants et arbitrage humain pour
   produire ResearchPack puis DecisionPack;
2. Conception execute quatre analyses paralleles, cross-review, arbitrage et
   checks deterministes avant de produire un ConceptionPack complet;
3. Factory consomme ce pack dans un workspace jetable isole et execute le
   contrat sprint complet: kickoff/plan/G1-G10, preflight, implementation,
   trois blocs de verification, review independante, Codex brut,
   reconciliation/reparation, verdict final, commit humain, carries et audit;
4. la chaine Ideas -> Conception -> Factory passe par operations headless et
   child-runs; un appel duplique est deduplique, un cycle, une depth ou un budget
   excessif sont refuses;
5. Babel execute un corpus Gutenberg aux droits verifies sur au moins 100 chunks
   hermetiques, distribue un dogfood borne sur deux workers, reprend apres crash,
   reassemble et conserve rights/provenance;
6. chaque pack peut etre ouvert, compris, forke, reevalue et invoque par schema
   depuis Workflow Hub ou une app autorisee.

**WA-T4 - Accessibilite maximale end-to-end**

1. creer, connecter, valider, lancer et reprendre le meme workflow sans souris
   et sans drag;
2. refaire le parcours avec lecteur d'ecran et comprendre graphe, erreurs,
   timeline, HITL et resultat;
3. executer l'alternative single-pointer sur touch/coarse pointer;
4. utiliser commande vocale avec labels visibles correspondant aux noms
   accessibles;
5. passer a 400 %/320 CSS px, forced colors, monochrome, text spacing, RTL et
   reduced motion sans perte de contenu ou fonction;
6. activer plusieurs preferences combinees avant le chargement de l'iframe;
7. ouvrir une approbation hote, comprendre operation/donnees/cout, refuser ou
   accepter puis retrouver le focus;
8. verifier captions/transcriptions et absence de son/mouvement/couleur seuls;
9. prouver que `needs[]`/diagnostics ne quittent jamais le noeud et que les
   preferences resolues ne sont exportees/synchronisees qu'apres opt-in, jamais
   vers un provider ou worker;
10. verifier la parite semantique canvas/outline/mode guide sur des mutations
    generees;
11. faire completer le parcours par le panel handicape en deux rounds et fermer
    toute barriere P0/P1;
12. decouvrir, inspecter, installer desactive, activer et publier avec clavier,
    lecteur d'ecran et mode guide;
13. archiver la matrice EN 301 549 applicable sans ecart P0/P1 non resolu.

**WA-T5 - Attaque et exploitation**

1. importer un pack malveillant sans executer son contenu;
2. refuser usurpation d'app, grant expire, capability non declaree et data
   privee vers reseau;
3. injecter crashs entre journal/outbox/reponse sans perte ni double effet;
4. simuler timeout, resultat duplique, resultat tardif et `outcome_unknown`;
5. verifier quotas, backpressure, retention et suppression;
6. muter chaque classe de digest/receipt/lineage et verifier le rejet;
7. mesurer graphe large, timeline longue et Babel multi-chunks contre les
   budgets chiffres ratifies au kickoff;
8. exporter un rapport de preuve sans secret ni donnee d'accessibilite;
9. conserver l'app operable et explicable dans tous les etats d'erreur.

**WA-T6 - Parite, migration et bascule Factory**

1. extraire en contrat machine-lisible les 9 roles, 10 modes, G1 a G10,
   FG0 a FG10, artefacts, statuts, trois blocs de verification, neuf sections
   de commit, carries et audit du processus courant;
2. executer la baseline legacy reelle -- Workflow Claude, driver main-thread,
   Codex CLI, hooks Git/`agentctl` et surfaces Factory concernees -- puis le
   nouveau workflow dans deux worktrees/state dirs isoles issus du meme snapshot
   immuable HEAD/index/profil et sur le meme intake; comparer transitions,
   artefacts, decisions et refus;
3. executer au moins une phase puis un sprint complet en shadow mode sans
   donner au nouveau runtime l'autorite d'un commit ou push autonome;
4. prouver l'independance driver/reviewer/Codex, conserver la sortie Codex brute
   et bloquer toute promotion de `PASS-PENDING` sans reconciliation;
5. emettre `commit.intent` puis `push.intent` seulement apres confirmation
   humaine liee au HEAD/index/diff et aux gates exacts;
6. laisser les hooks Git bloquer un verdict, un artefact ou un body incomplet,
   meme si le workflow declare localement un succes;
7. tester kill switch, snapshot, rollback vers l'ancien chemin et recuperation
   apres crash pendant la bascule;
8. ne deprecier l'ancien chemin qu'apres un sprint complet vert, sans ecart de
   parite ouvert et avec une procedure de retour documentee.

## 23. Gates bloquants proposes

| Gate | Preuve attendue |
|---|---|
| `G-SCOPE-FLOOR` | onze piliers livres; WA-T1 a WA-T6 complets; aucune coupe centrale masquee en carry |
| `G-PHASE-GRAMMAR` | labels A a AJ reconnus par preflight/review/Codex/auditor; phase numerique refusee |
| `G-AUTHORITY` | replay deterministe; tuer l'iframe ne tue pas le run |
| `G-BRIDGE-GRANT` | methode non declaree/non accordee refusee malgre message iframe valide |
| `G-EFFECT` | doubles commandes/resultats sans double transition ni effet |
| `G-UNKNOWN` | effet non idempotent perdu devient `outcome_unknown` |
| `G-FS-PROCESS-EFFECT` | HEAD/index/worktree preconditions, racine filesystem et descendants processus verifies en K/Q/AA |
| `G-CAPABILITY` | attenuation, expiration, audience et profondeur testees par proprietes |
| `G-SECRET` | canari absent app KV, DB, logs, blobs, prompts, traces et exports |
| `G-SECRET-VAULT` | credentials providers lies/rotates/supprimes dans un coffre hote, jamais dans le run ou l'app |
| `G-HITL` | iframe incapable d'approuver; mutation des parametres invalide la decision |
| `G-STORE` | fault injection journal/outbox sans perte ni double emission |
| `G-UPGRADE-REPLAY` | ancien run rejoue apres upgrade; events/upcasters, runtime et adapters pinning prouves |
| `G-SINGLE-WRITER` | lease/fencing empeche deux runtimes d'emettre la meme outbox |
| `G-NETWORK` | Task partagee public-only sans envelope chiffre; aucun grant/secret; ensemble reel des lecteurs prouve |
| `G-WIRE-COMPAT` | Task/Result/ArtifactRef vNext interopere ou refuse proprement les versions mixtes |
| `G-PLACEMENT` | meme contrat sur Ollama reel, cloud reel autorise et SBFB; `local_only` ne sort jamais |
| `G-MCP-SSRF` | schemas/outils pinne, DNS/IP/redirect revalides, destinations privees bloquees, output borne |
| `G-CLAUDE-CODEX-RECEIPT` | provider/modele/CLI/version/digests/exit/output brut et independance prouves en O/AB |
| `G-APPCALL` | child-run deduplique, schemas et cycles/depth refuses |
| `G-LIFECYCLE` | publish/install/activate/fork/update/re-consent/rollback/deprecate/revoke sans auto-update |
| `G-PUBLISH-RECOVERY` | publication partielle reparable et republish idempotent apres fault injection |
| `G-RRV-PROJECTION` | projection publique allowlistee, Proof Card, tombstone/reindex; RRV ne lance ni ne modifie |
| `G-VERTICALS` | E2E complet Ideas, Conception, Factory et Babel; pas seulement des fixtures compilables |
| `G-PROVENANCE` | mutation de chaque digest rejetee, roles non interchangeables |
| `G-ARTIFACT-ACL` | connaitre le digest ne permet pas de lire un artefact prive; audience/chiffrement/retention testes |
| `G-EVAL` | routes, tools, budgets et sorties verifies sur goldens |
| `G-FACTORY-CONTRACT` | 9 roles, 10 modes, G1-G10, FG0-FG10 et hooks separes; enforcement reel marque; drifts de verdict resolus |
| `G-REVIEW-INDEPENDENCE` | driver, reviewer et Codex distincts; sortie Codex brute; aucun auto-PASS |
| `G-HOOK-FINALITY` | hooks refusent PROFILE_STALE/source mismatch, PASS-PENDING, Codex absent, mismatch phase/HEAD et body incomplet |
| `G-FACTORY-CUTOVER` | shadow diff vert, source de verite explicite, kill switch et rollback testes avant deprecation |
| `G-HOST-ISOLATION` | companion separe, IPC authentifie/capability-scoped et crash sans fuite prouves en Q |
| `G-WORKFLOW-CSP` | profil CSP Hub strict; toute exception compensee et XSS incapable d'obtenir un effet bridge |
| `G-AUTHORING-AI-DIFF` | diff declaratif source-only, provenance, inspection et acceptation; aucune action autonome |
| `G-IMPORT-HOSTILE` | parser borne avant rendu; code/secrets/outils arbitraires refuses; brouillon non signe |
| `G-SUPPLY-CHAIN` | locks/licences/advisories/SBOM/build offline et corpus hostile verifies |
| `G-A11Y-WCAG` | zero echec connu A/AA sur toute surface/etat/processus; matrice AAA applicable archivee |
| `G-A11Y-EN301549` | version applicable figee; matrice clause par clause sans ecart P0/P1 non resolu |
| `G-A11Y-AUTHORING` | canvas, outline et mode guide partagent commandes/digest; templates et reparations suivent ATAG |
| `G-A11Y-PARITY` | 1 000 mutations generees a digest identique; chaque action disponible sans drag |
| `G-A11Y-AT` | matrice Windows/macOS/iOS/Android/Linux, lecteurs d'ecran, braille, voice et switch archivee |
| `G-A11Y-COGA` | comprehension, memoire, attention, erreurs et reprise testees avec utilisateurs concernes |
| `G-A11Y-PREFS` | axes fonctionnels empilables preboot; aucun diagnostic/need dans DOM, app, trace ou reseau |
| `G-A11Y-HOST` | grants, secrets, HITL, publication et erreurs du chrome hote au meme niveau |
| `G-A11Y-USERS` | deux rounds/panel remunere; zero barriere P0/P1 sur parcours critiques |
| `G-I18N` | nouveaux messages extraits et parite locale gatee |
| `G-PERF` | budgets chiffres graphe/timeline/Babel et size-limit verrouilles |

## 24. Plan canonique d'un futur sprint - 36 phases A a AJ

Ce plan remplace les anciennes listes indicatives de 38 phases dans ce document
et de 32 phases dans le compagnon Workflow Hub. Il n'active aucun numero de
sprint et ne cree aucun fichier sous `.planning/active/`.

### Prealable d'activation - Phase 0, hors des 36 phases

Avant Phase A, le sprint actif precedent doit etre ferme puis audite selon le
process repo. Le futur kickoff rebaseline le HEAD, produit kickoff, plan,
verification, audit plan et design review independante G1/G9; il ratifie scope,
decisions WA/HUB, menaces, WA-T1 a WA-T6, budgets et matrice de preuves. Il reserve
aussi provider cloud, deux workers, machines/licences AT et panel remunere avec
owner, disponibilite, fallback et exit condition. Un P0/P1, un
`DESIGN-CONFLICT`, une decision non gelee, une ressource obligatoire non
reservee ou un S82 encore ouvert bloque l'activation. Ce prealable n'est pas une
phase d'implementation et n'entre pas dans le compte A a AJ.

Les labels A a Z puis AA a AJ sont volontaires: ils restent compatibles avec la
grammaire de phase actuelle. Chaque phase doit avoir preflight, review
independante, Codex, reconciliation, verdict final, commit atomique et les trois
blocs Rust/Python/frontend, ou une exemption docs-only explicite conforme au
process. Accessibilite, securite, i18n et tests sont transversaux des le premier
commit concerne; AI consolide les preuves, elle ne repare pas tardivement le
produit.

### Phase A - Cadrage, contrats de preuve et ressources

Materialiser les decisions deja ratifiees en contrats versionnes et verifier
mecaniquement la matrice `objectif -> acceptance -> gate -> phase -> commande
-> artefact`, les budgets, owners et reservations. Prouver aussi que les labels
A a AJ declenchent les gates repo et qu'aucune dependance Day-0 n'est seulement
supposee. Phase A ne re-ouvre pas les decisions gelees au kickoff. Gates
principaux: `G-SCOPE-FLOOR`, `G-PHASE-GRAMMAR` et G1/G9.

### Phase B - Fondation accessibilite, i18n et preferences privees

Installer avant toute surface le preboot, les design tokens, le mode guide, le
profil de preferences resolues sans diagnostic, les harness WCAG/ATAG/COGA,
l'architecture i18n/RTL et la matrice handicap/AT/OS avec premier contact panel.

### Phase C - Autorite du bridge existant

Faire respecter a chaque appel et a chaque effet l'intersection
caller/app/release/methode/grant, y compris offline et local. Tester usurpation,
expiration, revocation per-call sur harness synthetique et branches speciales
avant d'ajouter un nouveau pouvoir Workflow. L, V et AI ferment ensuite la
revocation pendant un vrai run, juste avant dispatch et avant sortie d'outbox.
Gate initial: `G-BRIDGE-GRANT`.

### Phase D - Extraction du vrai contrat Factory

Transformer le processus actuel en `FactoryProcessProfile.v1` machine-lisible,
versionne et lie aux digests de ses sources: Truth Stack, 9 roles, 10 modes,
G1-G10, FG0-FG10, preflight S1a-S4, artefacts, statuts,
`PASS-PENDING`, Codex brut, reconciliation, trois blocs de verification, body
neuf sections, carries, audit et wrap-up. Ce golden impose le vocabulaire du
moteur avant tout gel de schema. Separer clairement les gates process G1-G10,
les gates de publication d'app FG0-FG10 et les hooks Git de finalite; marquer
pour chaque regle si elle est seulement documentee ou mecaniquement enforcee.
Resoudre les drifts de nombre de tracks/dimensions, verdict audit et exemption
Codex contre le canon avant compilation. `gate-check`
reste un diagnostic optionnel GO/BLOCK, jamais une nouvelle autorite ni un
teammate permanent. D ferme `G-FACTORY-CONTRACT` et definit seulement le contrat
candidat de `G-REVIEW-INDEPENDENCE`; ce dernier n'est ferme au runtime qu'en AB
puis consolide en AI.

Le profil lie les digests de toutes ses sources et les revalide au demarrage,
avant preflight, review et commit. Tout mismatch produit `PROFILE_STALE`, bloque
l'effet, impose re-extraction/migration puis nouvelle review du profil.

### Phase E - Spike durable bloquant

Prouver kill/restart/replay, journal/outbox atomiques, frontier identique et
absence de double effet a chaque frontiere de crash. L'echec produit un GO/NO-GO
explicite et bloque F: aucun format n'est gele avant cette preuve.

### Phase F - Format portable et signe

Definir `SBFBPromptAgentPack.v1`, layout, lockfile, licences, limites,
canonicalisation, domaines de signature et corpus hostile. Aucun code importe,
secret, grant ou etat live dans le pack; les digests Rust/TypeScript concordent.

### Phase G - Langage workflow, agents et roles

Definir graphes, agents, I/O, schemas, sequence, branche, parallelisme, map/join,
boucles bornees, sous-workflows, handoffs, memoires, budgets et contraintes
d'independance driver/reviewer/Codex. Compiler conceptuellement les quatre
goldens, Factory en premier.

### Phase H - Compilateur Rust autoritaire

Implementer normalisation, validation statique, fermeture des capabilities,
digests semantiques et plan immuable. Prouver anti-drift Rust/TypeScript et que
les quatre goldens, dont Factory complet, compilent sans pouvoir acquis par un
LLM.

### Phase I - Run store durable et compatible upgrades

Implementer SQLite, migrations, event log versionne, upcasters, snapshots,
inbox/outbox, transactions, retention, backup/restore, GC et fencing single
writer. Rejouer un ancien run apres upgrade et refuser un downgrade dangereux.
Gates: `G-STORE`, `G-UPGRADE-REPLAY`, `G-SINGLE-WRITER`.

### Phase J - Orchestrateur local

Implementer reducer/scheduler deterministe, branches, parallelisme, map/join,
boucles, timers, wait/signal, sous-workflows, pause, reprise, backpressure et
cancel honnete. L'autorite du run reste sur le noeud initiateur.

### Phase K - Effets, retries et idempotence

Definir `effect_id`, `attempt_id`, receipts, timeouts, reconciliation et
`outcome_unknown`. Lier les effets filesystem/processus au HEAD, index,
worktree et preconditions attendus; nettoyer aussi les processus descendants
apres cancel/crash, notamment sous Windows. Gate: `G-FS-PROCESS-EFFECT`.

### Phase L - Permissions, coffre de secrets et HITL hote

Implementer attenuation, audience, profondeur, expiration, revalidation juste
avant dispatch, classification/egress, coffre generique de credentials avec
rotation/suppression/zeroize et confirmations accessibles dans le chrome hote.
Gates: `G-CAPABILITY`, `G-SECRET-VAULT`, `G-SECRET`, `G-HITL`.

### Phase M - Artefacts, ACL et provenance

Definir `ArtifactRef` type, facade blob, classes public/prive/secret/reseau,
audience, chiffrement, retention, suppression, lineage et receipts. Connaitre un
digest ne donne aucun droit de lecture. Gates: `G-ARTIFACT-ACL` et
`G-PROVENANCE`.

### Phase N - Modeles locaux et cloud

Livrer adaptateur provider-neutral, Ollama reel, au moins un provider cloud reel
autorise par `secret_ref`, structured outputs, streaming borne, couts/quotas,
redaction et doubles hermetiques CI sans credential.

### Phase O - Adaptateurs Claude Code et Codex

Livrer `AgentSessionAdapter`, `ClaudeCodeAdapter` et `CodexExecAdapter`. Chaque
receipt conserve provider, modele, CLI/version, digests, exit code et sortie
brute. Les contextes et roles restent independants; un fallback meme-provider
est visible, degrade et jamais assimile silencieusement a une verification
independante. Gate: `G-CLAUDE-CODEX-RECEIPT`.

### Phase P - Wire, compute SBFB et MCP borne

Faire evoluer Task/Result/ArtifactRef avec schema, classification, audience et
deadline en compatibilite mixed-version; executer seulement des feuilles SBFB
bornees et signees. Ajouter MCP par outils/schemas pinne, sans token passthrough,
avec limites, validation et protections SSRF/DNS/redirect. **Plancher MVP:** tant
qu'un envelope chiffre recipient-scoped n'est pas livre et gate, seules les
donnees classees `public` peuvent entrer dans une Task partagee. La preuve doit
afficher et verifier l'ensemble reel des lecteurs du tasks-doc. Gates:
`G-WIRE-COMPAT`, `G-NETWORK`, `G-PLACEMENT` et `G-MCP-SSRF`.

### Phase Q - Hote de confiance et companion Factory

Exposer commandes/events/replay/quotas par bridge type. Placer repo, git, shell,
PTY et adaptateurs privilegies dans un companion local separe, avec IPC
authentifie et capability-scoped, sans elargir le TCB reseau du daemon. Ajouter
un profil CSP Hub strict et les intentions `commit.intent`/`push.intent` sans
execution autonome. Gates: `G-HOST-ISOLATION` et `G-WORKFLOW-CSP`.

### Phase R - Accueil et bibliotheque Workflow Hub

Livrer app sandboxee, boot/offline/reconnexion, navigation, recherche, recents,
templates, versions, conflits, etats vides/erreurs et responsive. La
bibliotheque operationnelle est l'accueil, pas une landing page decorative.

### Phase S - Registre d'agents et brouillons

Livrer CRUD/version/diff des agents, prompts, policies, outils, contexte,
memoire, budgets et dependances, plus revisions de brouillon, autosave,
undo/redo, recovery et conflits.

### Phase T - Canvas, outline et mode guide

Construire trois projections de premiere classe sur un modele de commandes
unique. Creation, connexion, reordre, erreurs et edition complete fonctionnent
sans drag, a la voix, au clavier et au lecteur d'ecran, avec digest semantique
identique.

### Phase U - Inspecteurs, capacites et preflight

Livrer formulaires types, schemas, providers, outils, data/cout/destination,
budgets, demandes/effectifs/refus/expiration de grants, diagnostics continus et
preflight comprehensible. Toute mutation invalide le consentement lie a
l'ancien digest.

### Phase V - Centre des Runs

Livrer liste, statuts, timeline bornee, child-runs, artifacts, HITL, pause,
cancel, retry, reconnect, replay, checkpoint et `outcome_unknown`, avec
alternatives textuelles et sans faux succes lorsque l'hote refuse.

### Phase W - Evaluations et comparaison

Livrer datasets/scenarios versionnes, graders deterministes avant LLM, routes,
handoffs, trajectoires d'outils, cout/latence, regressions, comparaison et export
de preuve. Aucun grader LLM ne peut accorder securite, droits, PASS ou commit.

### Phase X - Publication, decouverte, installation et RRV

Livrer signature, publication recuperable, catalogue decentralise, inspection,
installation desactivee, activation avec grants, fork, update/re-consent,
rollback, depreciation et revocation sans auto-update. Definir
`WorkflowPublicProjection.v1`, Proof Card, allowlist publique, tombstones et
reindex; RRV recherche et inspecte mais n'execute ni ne modifie. Gates:
`G-PUBLISH-RECOVERY`, `G-RRV-PROJECTION`, `G-LIFECYCLE`.

### Phase Y - Operations inter-apps

Livrer operations headless, child-runs idempotents, schemas, caller release,
depth, cycles, budgets et handoffs par ArtifactRef sous ACL. Aucun appel
iframe-to-iframe ni transit de secret. Gate: `G-APPCALL`.

### Phase Z - Factory kickoff, plan et preflight

Rejouer dans un workspace et namespace `dogfood_factory` distincts le cycle
d'entree Factory sur un cas reel:
audit Phase 0 de son sprint precedent, kickoff, plan, design review independante
et preflight S1a-S4. Un `DESIGN-CONFLICT`, `PROFILE_STALE` ou un artefact
manquant bloque avant
implementation. Audit Phase 0 `FAIL`, design review `FAIL` et `CONCERN` sans
acceptation humaine tracee bloquent egalement; aucune levee implicite n'existe.

### Phase AA - Factory implementation et tests

Executer le travail borne dans un worktree isole, avec driver, effets lies au
HEAD/index/worktree, tests cibles puis trois blocs de verification complets.
Prompt injection, repo hostile et sortie shell ne peuvent modifier plan,
capabilities ou autorite.

### Phase AB - Factory review, Codex et reconciliation

Enchainer review independante `PASS-PENDING`, output Codex brut,
CLEAN/GAP-P0..P3, reconciliation et boucle de reparation bornee. Toute correction
P0/P1 repasse review puis Codex; le driver ne peut jamais s'auto-attribuer PASS.
La machine d'etat conserve explicitement review, Codex, reconciliation,
repair/verifications et nouveau round au lieu d'ecraser un resultat precedent.
Les receipts prouvent provider, modele, CLI/version, prompt/input/diff digests,
exit code et output brut. Cette phase ferme `G-REVIEW-INDEPENDENCE`; AI le
reteste end-to-end. Si le budget de reparation est epuise, le run finit en
`BLOCKED_REPAIR_BUDGET`/`FAIL` et attend arbitrage ou re-scope humain: jamais
PASS, commit ou carry d'un P0/P1.

### Phase AC - Factory verdict, commit, carries et closure

Promouvoir seulement apres reconciliation vers `## Verdict: PASS`. Emettre un
`commit.intent` confirme humainement et lie au diff/gates exacts; les hooks
refusent `PROFILE_STALE`, mismatch, Codex absent ou body neuf sections incomplet.
Router G6/G7,
produire verification, audit plan et wrap-up, puis ne proposer `push.intent`
qu'apres ces preuves pour le dogfood. Cette position du push est une politique candidate a
ratifier au kickoff, pas une verite extraite des 10 modes actuels; tout autre
checkpoint exige une decision humaine explicite. Gates: `G-HOOK-FINALITY` et
fermeture des carries.

Apres wrap-up, conserver `IMPLEMENTED_AUDIT_PENDING`; le sprint ne devient
`AUDITED_CLOSED` qu'apres l'audit retrospectif du sprint suivant. Ces deux
statuts ne sont jamais fusionnes en un faux succes immediat.

Ces artefacts et statuts appartiennent au sprint Factory interne dogfood. Ils ne
ferment pas le sprint global A a AJ, dont verification, audit plan, wrap-up et
go/no-go restent exclusivement en AJ.

### Phase AD - Pack Ideas

Livrer intake, clarification, grounding, recherche sourcee, options, critiques
independantes, synthese, arbitrage humain, ResearchPack et DecisionPack
executables, forkables et evaluables.

### Phase AE - Pack Conception

Livrer architecture, UX, securite et protocole en parallele, cross-review,
arbitrage, ADRs et ConceptionPack type consommable par Factory, sans produire du
code ni une decision PO implicite.

### Phase AF - Pack Babel

Livrer rights gate, corpus Gutenberg legal borne, manifest source, chunking,
100 chunks hermetiques CI, dogfood deux workers, fanout SBFB, validation,
correction, crash/reprise, reassembly et provenance.

### Phase AG - Entrees declaratives: assistance IA et import Flowise

Transformer une demande en francais en diff declaratif source-only, avec
provenance, inspection et acceptation humaine explicite avant application.
L'assistant ne peut ni editer silencieusement, ni accorder un grant, ni lancer,
signer ou publier. Importer aussi un subset Flowise data-only allowliste apres
le format autoritaire: parser avant rendu, borner tailles/profondeur/cardinalite,
refuser code, credentials et outils arbitraires, produire rapport
mapped/approximated/refused et seulement un brouillon non signe sans grant ni
effet. Gates: `G-AUTHORING-AI-DIFF` et `G-IMPORT-HOSTILE`.

### Phase AH - Securite adversariale et supply chain

Fuzzer pack, compiler, bridge et wire; attaquer injection, SSRF, revocation,
secrets canari, CSP et packs malveillants. Verifier locks, licences, advisories,
SBOM, build offline/reproductible et corpus zip/path/casse/Unicode/duplicate
JSON/SVG/Markdown hostile. Gate: `G-SUPPLY-CHAIN`.

### Phase AI - Preuve produit complete

Executer WA-T1 a WA-T5 et la partie pre-cutover de WA-T6, les quatre packs,
trois placements, deux state dirs/deux daemons puis un dogfood sur deux machines
physiques avec partition/reconnexion.
Consolider fault injection, mixed-version, scale, performance, bundle/storage,
francais/RTL, cinq OS/AT, WCAG/AAA/EN 301 549, ATAG/COGA et deux rounds
utilisateurs remuneres. Aucun P0/P1 ne peut devenir un carry de confort.

### Phase AJ - Shadow Factory, cutover, rollback et documentation

Faire tourner la baseline legacy reelle (Workflow Claude, driver main-thread,
Codex CLI, hooks Git/`agentctl` et surfaces Factory concernees) et le nouveau
workflow sur le meme intake, d'abord avec `authority=legacy` et le nouveau
moteur execute reellement mais avec effets externes simules/refuses, sur une phase
puis un sprint complet. Comparer transitions, artefacts,
gates, receipts, decisions et refus par oracle differentiel: champs
deterministes exactement, sorties LLM semantiquement par auditeur independant.
Les deux executions utilisent worktrees/state dirs isoles issus du meme snapshot
HEAD/index/profil. Definir source de verite, coexistence, migration/archivage,
kill switch, snapshot et rollback;
garder les hooks Git comme autorite fail-closed et l'ancien chemin jusqu'a un
sprint complet vert. `authority=new` exige zero P0/P1 et aucun P2/P3 de parite
ouvert sans acceptation humaine tracee, puis une decision humaine explicite;
aucun rollback n'efface un commit ni ne force-push automatiquement.
Rejouer ensuite un smoke puis l'acceptance complete post-bascule avant de fermer
WA-T6 et HUB-T10. Fermer avec contrats, guides, threat model, preuves,
limites, audit plan et go/no-go pilote ferme. Gate: `G-FACTORY-CUTOVER`.

Aucun point de coupe arbitraire dans cette liste ne satisfait l'objectif. Si
une dependance externe rend un pilier impossible, le sprint doit revenir au PO
pour arbitrer explicitement l'objectif; il ne peut pas declarer le MVP complet
en cachant la fonction dans un carry.

## 25. Risques et mitigations

| Risque | Priorite | Mitigation |
|---|---:|---|
| App manifest confondu avec autorisation | P0 | enforcement host par release/grant avant bridge Workflow |
| Double effet apres crash | P0 | journal + outbox atomiques, `effect_id`, reconciliation unknown |
| Secret dans contexte ou trace | P0 | refs opaques, scanner canari, redaction et egress policy |
| Prompt injection accorde un outil | P0 | plan/capacites hors controle du modele |
| Workflow publie contient du code | P0 | format declaratif strict, aucun eval/deserialization executable |
| Reseau presente comme scheduler | P0 produit | autorite locale et feuilles seulement au MVP |
| Provenance confondue avec justesse | P0 confiance | roles/preuves explicites et evals separees |
| Pack update elargit les droits | P0 | version/digest nouveau, install disabled, re-consent |
| Scope audacieux recoupe en demo | P0 produit | G-SCOPE-FLOOR, WA-T1 a WA-T6 et arbitrage PO obligatoire avant toute coupe |
| UX canvas inaccessible ou alternatives degradees | P0 accessibilite | modele unique, parite fonctionnelle et golden digest canvas/outline/mode guide |
| HITL hote inaccessible | P0 accessibilite/securite | G-A11Y-HOST sur grants, secrets, publication et erreurs |
| Profil expose un diagnostic | P0 privacy | transmettre seulement preferences resolues; `needs[]` reste local et non trace |
| Scanner automatique donne une fausse confiance | P0 accessibilite | multi-AT, tests manuels et participants handicapes remuneres |
| Accessibilite repoussee en fin de sprint | P0 process | fondation Phase B, gates a chaque phase, Phase AI seulement consolidante |
| Factory reduite a implementation/test | P0 produit/process | golden complet Phase D, workflow Z-AC et shadow/cutover AJ |
| Codex integre comme simple appel LLM | P0 process | adaptateur dedie O, sortie brute, independance et reconciliation bloquante AB |
| Cutover rend les hooks secondaires | P0 process | `G-HOOK-FINALITY`, source de verite explicite et rollback AJ |
| Etat global partage fuit entre agents | P1 | projections de contexte et stores separes |
| Artifact/result wire trop etroit | P1 protocole | ArtifactRef type et evolution wire planifiee |
| KV app utilise comme event log | P1 fiabilite | store runtime dedie |
| Bundle shell grossit | P1 perf | app separee, chunks lazy, budgets mesures |
| Import framework promet trop | P1 | subset allowliste et rapport de pertes/refus |
| Conception reste prose non consommable | P1 produit | schema ConceptionPack + fixture Factory |
| Babel contourne les droits | P0 legal/trust | source-policy gate avant fanout |

## 26. Decisions PO

### 26.1 Decisions deja confirmees par le cadrage

- Le chantier global s'appelle conceptuellement **Workflow Agents** et son app
  visible **Workflow Hub**.
- Il sert a construire des workflows LLM et multi-agents.
- Il constitue la base commune d'Ideas, Conception, Factory, Babel et futures
  apps connectees au protocole.
- Workflow Hub est l'app visible de conception/exploitation; Workflow Agents
  designe le chantier global incluant app, runtime, protocole et adaptateurs.
- Le workflow Factory complet est le vertical fondateur: il reprend le vrai
  systeme de sprint, Claude, Codex, reconciliation, gates, preuves, commit
  humain, carries et audit, puis sert de base forkable aux futures Factory.
- Il doit etre une app utilisable sur les noeuds, pas seulement une fonction
  interne de Factory.
- Le MVP doit etre audacieux et livrer l'objectif complet dans un seul sprint.
- Le sprint peut contenir autant de phases que necessaire; aucune limite de
  phases ne justifie une coupe du scope floor.
- L'accessibilite maximale pour toutes les grandes familles de handicap et
  leurs combinaisons fait partie du MVP, sur l'app et le shell hote.
- La recherche est hors sprint; aucun fichier `.planning/active` ne doit etre
  cree ou modifie par ce chantier.

### 26.2 Decisions a ratifier au kickoff

| ID | Decision | Recommandation |
|---|---|---|
| WA-0 | Placement UI | app SBFB sandboxee plein ecran; approbations dans le chrome hote |
| WA-1 | Nom schema workflow | `SBFBWorkflowGraph.v1` dans `SBFBPromptAgentPack.v1` |
| WA-2 | Placement runtime | service Rust hote generique, pas iframe ni dependance Factory complete |
| WA-3 | Autorite run MVP | noeud initiateur unique |
| WA-4 | Surface bridge | commandes/evenements types avec enforcement release/grants |
| WA-5 | Store | SQLite journal/outbox dedie |
| WA-6 | Appels cross-app | operations headless par child-run, jamais iframe-to-iframe |
| WA-7 | Interfaces auteur | app plein ecran + canvas/outline/mode guide a parite |
| WA-8 | Workflow phare | Ideas -> Conception -> Factory avec local + network leaf |
| WA-9 | Fixture scale | Babel sur corpus legal borne reel, multi-chunks et provenance |
| WA-10 | MCP/A2A | MCP adapter MVP si borne; A2A futur |
| WA-11 | Publication | cycle pack complet dans l'objectif MVP |
| WA-12 | Import Flowise | subset data-only, apres format autoritaire |
| WA-13 | Standard accessibilite | WCAG 2.2 AA bloquant, AAA applicable evalue, ATAG + COGA |
| WA-14 | Profil accessibilite | preferences resolues host->app; aucun diagnostic ou `needs[]` partage |
| WA-15 | Acceptance accessibilite | multi-AT + participants handicapes remuneres avant closure |
| WA-16 | Matrice europeenne | version EN 301 549 publiee/harmonisee applicable figee au kickoff; revision approuvee evaluee |
| WA-17 | Contrat Factory source | `PROCESS.md`, `AGENT_SYSTEM.md`, hooks et artefacts repo prevalents; golden extrait avant langage |
| WA-18 | Independance agents | roles/contextes separes; Claude Code et Codex adapters explicites; fallback degrade visible |
| WA-19 | Isolation privilegiee | FactoryAdapter dans un companion local IPC capability-scoped, pas dans le TCB reseau complet |
| WA-20 | Wire workflow | evolution backward-compatible Task/Result/ArtifactRef; reseau public-only sans envelope chiffre recipient-scoped gate |
| WA-21 | Secrets providers | coffre hote generique avec binding, rotation, suppression, zeroize et audit d'usage |
| WA-22 | Compatibilite runtime | events versionnes/upcasters, adapter pinning, fencing, backup/restore et GC referentiel |
| WA-23 | Commit et push | intentions seulement, confirmation humaine liee au HEAD/index/diff/gates; hooks autoritaires |
| WA-24 | Migration Factory | shadow old/new, oracle de parite, source de verite, kill switch, rollback puis deprecation tardive |

## 27. Questions techniques ouvertes

- Le runtime vit-il dans `nexus-shell-daemon`, un companion local ou une
  couche partagee avec `sbfb-factory`?
- Quel schema machine-lisible represente exactement les 9 roles, 10 modes,
  G1-G10, FG0-FG10 et artefacts Factory sans dupliquer leur autorite?
- Quelle frontiere IPC minimale separe le runtime generique du companion
  privilegie Factory (repo, git, shell, PTY, Claude Code et Codex)?
- Quel contrat de migration et retention pour le run store?
- Quelles versions d'events, upcasters, leases/fencing et regles de downgrade
  permettent replay et rollback apres upgrade?
- Quel transport live est le plus simple a securiser avec l'iframe actuelle?
- Les ArtifactRef reutilisent-ils directement iroh-blobs ou une facade hote?
- Quel subset de JSON Schema est supporte identiquement Rust/TypeScript?
- Comment modeliser `map/join` sans explosion de checkpoint?
- Quelle politique de reconciliation pour chaque adaptateur non idempotent?
- Comment representer cout/energie quand un worker ne fournit pas de metrique
  attestable?
- Quelle attestation relie proprement une curation locale a un pack publie?
- Quel schema exact pour `ConceptionPack` et son intake Factory?
- Quelle evolution backward-compatible de Task/Result ajoute ArtifactRef et
  classification sans casser les workers actuels?
- Quel backend de coffre de secrets est retenu sur Windows/macOS/Linux et en CI
  hermetique, avec rotation et suppression verifiables?
- Phase P livre-t-elle un envelope chiffre recipient-scoped au-dela du plancher
  `public`-only, ou ce support reste-t-il explicitement futur?
- Combien de shadow-runs et quel oracle differentiel ferment
  `G-FACTORY-CUTOVER` avant deprecier l'ancien chemin?
- Quelle limite de taille et de complexite pour un pack avant installation?
- Le profil d'accessibilite partage devient-il un contrat du shell ou un
  contrat general SBFB, et comment migre-t-il sans exposer les besoins?
- Quelles machines et licences sont reservees pour JAWS, VoiceOver, TalkBack,
  Dragon/Voice Access, switch et Braille?
- Comment recruter et remunerer un panel de participants avec besoins varies et
  combines tout au long du sprint?
- Quels criteres AAA sont applicables a chaque surface et quelles exceptions
  eventuelles exigent une decision explicite?

## 28. Definition de done de la recherche

Cette recherche est suffisante pour preparer un kickoff lorsque:

- la frontiere app/runtime/protocole est explicite;
- la question iframe est tranchee par moindre privilege;
- la capacite de construire des workflows LLM/agents est detaillee;
- les quatre verticals imposent le vocabulaire candidat;
- les faits du repo sont separes des propositions;
- le protocole actuel n'est pas presente comme scheduler de DAG;
- le format racine reste coherent avec Prompt/Agent Packs;
- autorisation, secrets, reprise, idempotence et provenance sont traites;
- le MVP audacieux possede onze piliers, un scope floor et WA-T1 a WA-T6;
- le sprint full-objective sans limite de phases est explicite;
- le futur sprint possede un seul DAG canonique de 36 phases A a AJ, avec la
  Phase 0 d'audit hors decompte et aucun plan Hub concurrent;
- Factory est specifiee comme workflow fondateur complet, avec migration,
  shadow, cutover et rollback, pas comme une simple boucle de generation;
- l'accessibilite maximale couvre chaque famille de barrieres, l'app, le host,
  les preferences privees, les AT et les tests utilisateurs;
- les non-objectifs retires sont des frontieres de confiance/distribution, pas
  des coupes de confort;
- les gates et decisions de kickoff sont actionnables.

Elle ne constitue pas encore:

- un schema ratifie;
- une estimation fiable sans rebaseline;
- une preuve d'implementation;
- un engagement de sprint;
- un verdict de securite ou de readiness publique.

## 29. Sources Nexus principales

- `.planning/research/prompt_agent_packs_protocol_research.md`
- `.planning/research/idea_development_workflow_provider_neutral.md`
- `.planning/research/idea_development_science_ultradeep_2026-06-30.md`
- `.planning/research/idea_hub_factory_ultracode_redesign.md`
- `.planning/research/babel_translation_protocol.md`
- `.planning/research/s73_s75_factory_babel_research.md`
- `.planning/research/sprint82_workflow_engine/verification_blueprint.md`
  (source historique de mesures/fixtures; architecture remplacee ici)
- `docs/agent/RRV_FACTORY_CONTRACT.md`
- `web/src/bridge/protocol.ts`
- `web/src/bridge/useBridge.ts`
- `crates/sbfb-manifest/src/lib.rs`
- `crates/nexus-core-rs/src/csp.rs`
- `crates/nexus-core-rs/src/task.rs`
- `crates/nexus-shell-daemon/src/storage_api.rs`
- `crates/nexus-shell-daemon/src/deploy.rs`
- `crates/nexus-coordinator-rs/src/provenance.rs`
- `crates/sbfb-factory/src/provider_router.rs`
- `crates/sbfb-factory/src/operator_server.rs`
- `tools/factory-operator/src/preferences/accessibility.ts`
- `tools/factory-operator/scripts/check-accessibility-system.sh`
- `docs/factory/OPERATOR_ACCESSIBILITY_DESIGN_SYSTEM.md`

Prototype inspecte, hors repo Nexus:

- `C:/Users/FlowUP/Documents/Code/sbfb-agent-packs-lab`
- `C:/Users/FlowUP/Documents/Code/flowise-local/.flowise/create_idea_hub_complete_workflow.js`

## 30. Sources externes primaires

Orchestration et duree:

- LangGraph, workflows and agents:
  https://docs.langchain.com/oss/python/langgraph/workflows-agents
- LangGraph, persistence:
  https://docs.langchain.com/oss/python/langgraph/persistence
- LangGraph, interrupts:
  https://docs.langchain.com/oss/python/langgraph/interrupts
- OpenAI Agents SDK, orchestration:
  https://openai.github.io/openai-agents-python/multi_agent/
- OpenAI Agents SDK, handoffs:
  https://openai.github.io/openai-agents-python/handoffs/
- Microsoft Agent Framework overview:
  https://learn.microsoft.com/en-us/agent-framework/overview/
- Microsoft Agent Framework checkpoints:
  https://learn.microsoft.com/en-us/agent-framework/workflows/checkpoints
- Google ADK workflow agents:
  https://adk.dev/agents/workflow-agents/
- Flowise Agentflow V2:
  https://docs.flowiseai.com/using-flowise/agentflowv2
- CrewAI Flows:
  https://docs.crewai.com/en/concepts/flows
- n8n executions:
  https://docs.n8n.io/workflows/executions/all-executions/

Interoperabilite et securite:

- MCP tools, version 2025-11-25:
  https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- MCP authorization:
  https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization
- A2A key concepts:
  https://a2a-protocol.org/latest/topics/key-concepts/
- A2A discovery:
  https://a2a-protocol.org/latest/topics/agent-discovery/
- RFC 8785, JSON Canonicalization Scheme:
  https://www.rfc-editor.org/rfc/rfc8785.html
- DSSE:
  https://github.com/secure-systems-lab/dsse
- in-toto Statement v1:
  https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md
- OpenTelemetry GenAI semantic conventions:
  https://opentelemetry.io/docs/specs/semconv/registry/attributes/gen-ai/
- OWASP LLM Top 10:
  https://genai.owasp.org/llm-top-10/

Accessibilite:

- WCAG 2.2 Recommendation:
  https://www.w3.org/TR/WCAG22/
- ATAG overview, authoring tools:
  https://www.w3.org/WAI/standards-guidelines/atag/
- COGA, Making Content Usable:
  https://www.w3.org/TR/coga-usable/
- WAI-ARIA APG, keyboard interface:
  https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/
- WAI-ARIA 1.2:
  https://www.w3.org/TR/wai-aria-1.2/
- WAI-ARIA APG, treegrid pattern:
  https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/
- WAI, diverse abilities and barriers:
  https://www.w3.org/WAI/people-use-web/abilities-barriers/
- WAI, involving users in accessibility evaluation:
  https://www.w3.org/WAI/test-evaluate/involving-users/
- WAI, selecting accessibility evaluation tools:
  https://www.w3.org/WAI/test-evaluate/tools/selecting/
- ETSI, EN 301 549 V3.2.1 publiee:
  https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf
- ETSI Human Factors, chantier de revision:
  https://www.etsi.org/technical-groups/hf/
- WCAG 2.5.7, alternatives to dragging:
  https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html
- WCAG 2.5.8, target size minimum:
  https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- WCAG 2.3.3, animation from interactions:
  https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html

## 31. Handoff au futur kickoff

Le futur kickoff doit commencer par:

1. terminer puis auditer le sprint precedent en Phase 0 avant d'activer le
   nouveau sprint;
2. revalider toutes les ancres contre le HEAD et confirmer que les labels A a
   AJ activent bien tous les gates actuels;
3. ratifier WA-0 a WA-24 et la relation Workflow Hub app visible / Workflow
   Agents chantier global;
4. transformer WA-T1 a WA-T6, les onze piliers et les gates en matrice executable
   `objectif -> acceptance -> gate -> phase -> commande -> artefact`;
5. reserver provider cloud, deux workers, machines/licences AT et panel avec
   owner, disponibilite, fallback et exit condition;
6. corriger l'enforcement des grants bridge avant d'ajouter des pouvoirs;
7. extraire le golden Factory complet avant de figer le langage ou les roles;
8. livrer la fondation accessibilite et son gate avant les premieres surfaces;
9. prouver crash/replay/outbox avant de geler tout le format;
10. compiler les quatre goldens avant de figer le vocabulaire des nodes;
11. conserver `SBFBPromptAgentPack.v1` comme racine portable;
12. garder l'autorite du run locale et le calcul reseau terminal au MVP;
13. traiter l'app iframe comme UI non fiable, meme si elle est first-party;
14. conserver Claude Code et Codex comme adaptateurs distincts avec sortie
    Codex brute, reconciliation et independance verifiable;
15. appliquer les gates accessibilite, securite et i18n a chaque phase; AI ne
    fait que consolider les preuves;
16. faire de Z a AC une execution reelle du processus Factory, puis fermer AJ
    par shadow-run, cutover/rollback et autorite finale des hooks;
17. refuser toute coupe qui laisserait de cote accessibilite, publication,
    installation, quatre packs, Factory complete ou handoff cross-app tout en
    appelant le resultat MVP Workflow Agents complet.

### Checklist de revue independante avant activation

- confirmer qu'il existe exactement 36 labels de Phase A a Phase AJ;
- confirmer qu'aucune liste 1-38 ou 1-32 ne reste activable;
- verifier l'ordre C -> D -> E -> F pour bridge, Factory, durable et format;
- verifier chaque exigence Factory contre `PROCESS.md`, `AGENT_SYSTEM.md`,
  `RRV_FACTORY_CONTRACT.md`, `FACTORY_GATES.md`, hooks et code courant;
- verifier que chaque gate a une phase proprietaire, une commande, une fixture
  et un artefact machine-lisible dans le futur kickoff;
- verifier qu'aucune dependance externe non reservee n'est presentee comme
  disponible;
- rendre un verdict `GO`, `PLAN-ADAPT` ou `NO-GO` sans modifier l'autorite des
  fichiers repo.
