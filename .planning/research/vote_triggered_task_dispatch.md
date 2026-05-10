# Research: Vote-Triggered Autonomous Task Dispatch

**Date:** 2026-05-10
**Contexte:** SBFB Ideas Hub (sbfb-ideas) — quand une idee
communautaire atteint un seuil de votes, declencher automatiquement
une tache de generation de code AI sur les GPU workers du reseau.
**Confiance globale:** MEDIUM (mecanismes DAO bien documentes,
integration P2P+AI = territoire nouveau, pas de precedent exact)

---

## 1. DAO / Governance Automation : comment les seuils declenchent des actions

### 1.1 OpenZeppelin Governor (reference standard)

**Repo:** https://github.com/OpenZeppelin/openzeppelin-contracts/tree/master/contracts/governance

Le Governor OpenZeppelin est le standard de facto pour la
gouvernance on-chain. Son cycle de vie est directement
transposable off-chain :

```
propose() --> Voting Delay --> vote() --> Voting Period ends
         --> quorum reached? + majority? --> queue() in Timelock
         --> Timelock delay expires --> execute()
```

**Mecanismes cles transposables a SBFB :**

| Concept Governor | Equivalent SBFB |
|---|---|
| `quorumNumerator` / `quorumDenominator` | Seuil = N votes OU N% des peers actifs |
| `proposalThreshold` (min tokens pour proposer) | Kudos minimum pour poster une idee |
| `votingDelay` (blocs avant debut vote) | Delai apres publication (anti-spam reflexe) |
| `votingPeriod` (duree du vote) | Fenetre de vote (ex: 7 jours) |
| `TimelockController.execute()` | Dispatch task vers worker GPU |
| `ProposalState.Succeeded` | Seuil atteint = trigger dispatch |

**Source:** https://docs.openzeppelin.com/contracts/4.x/governance
**Confiance:** HIGH (code audite, reference industrie)

### 1.2 Compound Governor Bravo

**Repo:** https://github.com/compound-finance/compound-protocol/blob/master/contracts/Governance/GovernorBravoDelegate.sol

Compound utilise des seuils numeriques fixes :
- **Proposal threshold:** 25,000 COMP (pour creer une proposition)
- **Quorum:** 400,000 votes minimum pour qu'un vote soit valide
- **Timelock:** 2 jours entre passage et execution

Le mecanisme d'execution est permissionless : n'importe qui
peut appeler `execute()` une fois le timelock expire. C'est un
pattern cle pour SBFB — pas besoin d'un coordinateur central
pour declencher l'execution.

**Pattern transposable :** le premier noeud qui observe le seuil
atteint peut soumettre la tache. L'idempotence est geree par
un identifiant unique de l'idee.

**Source:** https://docs.compound.finance/v2/governance/
**Confiance:** HIGH

### 1.3 Aragon OSx : Governance Plugins vs Policy Plugins

**Repo:** https://github.com/aragon/osx

Aragon distingue deux types de plugins, et c'est cette distinction
qui est la plus pertinente pour SBFB :

**Governance plugins** (TokenVoting, Multisig) : collectent des
preferences et les agregent en une decision. C'est le vote
classique pour les idees. "Stakeholders evaluate and approve any
calldata prior to it becoming executable."

**Policy plugins** : executent des actions predefinies quand les
conditions sont remplies. "Deterministic, rule-bound plugins that
are governed in scope." L'execution est "permissionless to trigger
by anyone at any time." Le plugin verifie l'etat courant, et si
les conditions matchent, compose l'action et la soumet a
l'executeur.

**Application SBFB :** le seuil de votes est une **policy**.
Une fois que le reseau a decide "au-dessus de N votes, on
declenche une tache AI", c'est une regle automatique — pas une
decision humaine repetee. Seul le seuil initial est un choix
de gouvernance.

**Source:** https://blog.aragon.org/beyond-proposals-pt-i-automation-and-the-art-of-not-governing/
**Confiance:** HIGH

### 1.4 Snapshot + oSnap : Off-chain vote, on-chain execution

**Lien:** https://medium.com/uma-project/announcing-osnap-gasless-snapshot-voting-with-on-chain-execution-by-uma-7374ed729b28

oSnap resout exactement le probleme "vote off-chain, action
on-chain" :

1. Vote Snapshot se termine (off-chain, gasless)
2. N'importe qui peut soumettre les transactions correspondantes
3. Periode de challenge (optimistic oracle UMA) — si personne
   ne conteste, les transactions s'executent automatiquement
4. Si conteste : pas d'execution, re-soumission necessaire

**Mecanisme optimiste transposable a SBFB :** au lieu de verifier
le decompte des votes avant d'executer (verification synchrone),
le systeme **presume que le seuil est atteint** quand un noeud
le declare. Les autres noeuds ont une fenetre pour contester.
Pas de contestation = execution. Cela evite le besoin de consensus
synchrone pour valider le decompte.

**Source:** UMA / Snapshot
**Confiance:** MEDIUM (le pattern optimiste est prouve on-chain
mais son adaptation P2P/gossip est speculative)

### 1.5 Tally : interface de governance et execution

Tally fournit l'interface (frontend) et les outils pour deployer
des DAO avec OpenZeppelin Governor. Le pattern pertinent est
l'abstraction : Tally ne modifie pas le mecanisme de gouvernance,
il l'expose a l'utilisateur.

Pour SBFB, le parallel est que **sbfb-ideas est l'equivalent de
Tally** : une interface qui expose le mecanisme de vote et de
dispatch, sans etre elle-meme le mecanisme.

**Source:** https://docs.tally.xyz/
**Confiance:** HIGH (pour l'analogie architecturale)

---

## 2. GitOps / CI automation declenchee par des votes

### 2.1 Etat de l'art : aucun precedent direct

La recherche n'a trouve **aucun projet open source** ou des
votes communautaires declenchent directement un pipeline CI ou
de la generation de code. Les systemes existants sont :

- **GitHub Actions / GitLab CI** : declenches par des evenements
  Git (push, PR, tag, schedule) — jamais par des votes
- **ArgoCD** : synchronise un etat desire dans Git avec un cluster
  — pas de mecanisme de vote
- **Harness Triggers** : webhook, cron, manifest changes — pas de
  vote

Le concept "votes communautaires → CI pipeline" est **un
territoire vierge**. SBFB serait un pionnier ici.

**Confiance:** HIGH (pour l'assertion "ca n'existe pas encore")

### 2.2 Projets adjacents

**GitHub Copilot Coding Agent (septembre 2025) :** prend un
GitHub issue et ouvre un draft PR automatiquement. Mais le trigger
est humain (label l'issue), pas un vote communautaire.
**Source:** https://githubnext.com/projects/copilot-for-pull-requests/

**Sweep AI :** bot GitHub qui convertit les issues en PRs.
Label une issue → Sweep analyse le codebase → cree un plan →
ouvre un PR. Encore un trigger humain (label), pas un vote.
**Source:** https://github.com/sweepai/sweep

**Pattern extractible :** le mecanisme issue → plan → PR est
mure. C'est le **trigger** (vote au lieu de label) qui est
nouveau.

### 2.3 Gitcoin : votes → financement (pas code)

Gitcoin utilise le **quadratic voting** pour allouer des fonds
a des projets open source. Le vote communautaire influence la
distribution de fonds ($60M+ distribues), mais ne declenche pas
de taches de code. C'est un precedent de "vote communautaire →
action concrete" mais l'action est financiere, pas computationnelle.

**Source:** https://gitcoin.co/mechanisms/quadratic-voting
**Confiance:** HIGH

---

## 3. Threshold-based dispatch en P2P : "N sur M agreent → action"

### 3.1 Le probleme fondamental

Dans SBFB, il n'y a **pas de serveur central** qui compte les
votes et declenche l'action. Chaque noeud voit les votes arriver
via gossip. Le defi est : comment s'assurer que **tous les noeuds
s'accordent** sur le fait que le seuil est atteint, et qu'**un
seul** dispatch est effectue ?

### 3.2 Option A : CRDT counter + gossip (recommande pour SBFB)

**Mecanisme :**
- Chaque vote est un message gossip signe Ed25519
- Chaque noeud maintient un **G-Counter CRDT** (grow-only counter)
  par idee, indexe par cle publique du votant
- Le merge est idempotent : recevoir le meme vote 2 fois ne
  change rien (union des cles votantes)
- Chaque noeud peut independamment calculer le total
- Quand un noeud detecte que le seuil est atteint, il **propose**
  le dispatch

**Avantages pour SBFB :**
- Compatible avec le gossip iroh existant
- Pas besoin de consensus synchrone (Raft, Paxos)
- Tolere les retards de propagation : un noeud peut voir le seuil
  avant les autres, mais tous convergeront
- Compatible avec la philosophie "zero moderation centrale"

**Source:** https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type
+ https://github.com/JDrit/gossip-crdt
**Confiance:** HIGH (CRDTs sont prouves mathematiquement)

**Deduplication du dispatch :** le CRDT ne resout pas le
probleme "qui dispatch". Deux approches :

1. **Deterministic proposer :** hash(idea_id) mod N_peers →
   le noeud dont le rang correspond est le proposeur designe.
   Simple mais fragile si ce noeud est offline.

2. **First-come acknowledged :** le premier noeud a observer
   le seuil broadcast un `TaskProposal`. Les autres noeuds
   ignorent les proposals dupliquees pour la meme idee (par
   idea_id). Fenetre de race resolvable par ordering gossip
   (timestamp + tie-break sur la cle publique la plus basse).

### 3.3 Option B : Raft / consensus leader

Un leader elu par consensus Raft est responsable du comptage
et du dispatch. C'est plus simple algorithmiquement mais
**contradictoire** avec l'architecture SBFB :
- Necessite un cluster stable de noeuds (3, 5, 7)
- Introduit un point central (le leader)
- Complexite d'implementation significative
- Non recommande pour SBFB

**Source:** https://en.wikipedia.org/wiki/Raft_(algorithm)
**Confiance:** HIGH (pour le rejet — Raft est incompatible avec
le modele SBFB)

### 3.4 Option C : Threshold signature (FROST)

SBFB utilise deja FROST pour le warrant canary. Le pattern
serait : le dispatch n'est valide que si t-of-n curators le
signent. Le seuil de votes declenche une **ceremonie de signature
FROST** dont le resultat est la preuve cryptographique que le
quorum est atteint.

**Avantages :**
- Preuve cryptographique irrefutable
- Deja dans le codebase SBFB (`frost-ed25519 = "2.1"`)
- Lie le dispatch a l'identite des curators (pas des noeuds
  anonymes)

**Inconvenients :**
- Overhead significatif (DKG + 2 rounds de signature)
- Necessite que les curators soient en ligne simultanement
- Trop lourd pour du trigger automatique a haute frequence

**Recommandation :** FROST pour les actions a haut risque
(deployer du code sur le reseau), CRDT counter pour le
comptage des votes eux-memes.

**Source:** https://github.com/ZcashFoundation/frost
**Confiance:** HIGH (FROST est deja dans SBFB)

### 3.5 Option D : Othentic / EigenLayer AVS pattern

**Repo:** https://github.com/Othentic-Labs/avs-examples

Othentic (infrastructure AVS EigenLayer) utilise un pattern
directement pertinent :

1. **Performer** execute la tache off-chain
2. **Attesters** valident le resultat via BLS signatures
3. Si >2/3 du quorum attest "valid", la tache est acceptee
4. Le Performer broadcast le resultat via P2P
5. Les Attesters signent avec BLS et les signatures sont
   agregees

**Transposition SBFB :**
- Worker GPU = Performer (execute la generation de code)
- Curators = Attesters (valident que le code genere est sain)
- Le seuil 2/3 est configurable
- L'agregation BLS est plus legere que FROST pour de la
  validation repetee

**Source:** https://docs.othentic.xyz/main/learn/advanced-concepts/leader-election
**Confiance:** MEDIUM (le pattern est prouve dans l'ecosysteme
EigenLayer mais l'adaptation a SBFB est speculative)

### 3.6 PeerVote : precedent academique P2P

**Source:** Bocek et al. 2009, "PeerVote: A Decentralized Voting
Mechanism for P2P Collaboration Systems"
https://link.springer.com/chapter/10.1007/978-3-642-02627-0_5

PeerVote definit 6 roles (tracker, storage, user, editor,
mediator, voter). Le **mediator** est responsable d'une session
de vote : il contacte les voteurs, collecte les signatures, et
si une majorite approuve, le document modifie est publie.

**Pertinence SBFB :** le role "mediator" peut etre assume par
n'importe quel noeud (rotation par hash, comme le deterministic
proposer). La difference cle est que PeerVote vote sur des
**modifications de documents** (wiki), pas sur des idees generales.

**Confiance:** HIGH (papier academique cite, simulations validees)

---

## 4. Orchestration de generation de code AI

### 4.1 OpenHands (ex-OpenDevin) — reference open source

**Repo:** https://github.com/All-Hands-AI/OpenHands
**Score SWE-Bench:** 72% (meilleur agent open source a mai 2026)

**Architecture de decomposition :**

1. **CodeAct Agent** (agent principal) recoit une tache en langage
   naturel
2. L'agent construit un **plan pas-a-pas** depuis la spec
3. A chaque etape : execute des commandes shell, edite des
   fichiers, lance les tests
4. Si les tests echouent, l'agent itere et corrige
5. **AgentDelegateAction** permet de deleguer un sous-probleme
   a un agent specialise (ex: BrowsingAgent pour la navigation web)
6. **Micro-agents** : agents specialises via des prompts custom,
   reutilisant l'infrastructure CodeAct

**Execution sandbox :** chaque tache tourne dans un container
Docker isole avec bash + IPython + Chromium. L'API REST
OpenHands recoit les actions et retourne les observations.

**Confiance:** HIGH (code open source, papier arxiv 2407.16741)

### 4.2 SWE-Agent — decomposition en sous-agents

**Repo:** https://github.com/princeton-nlp/SWE-agent

Deux familles d'architecture identifiees :

**Workflow-based** : decompose la resolution en etapes
predefinies (localisation → edition → selection de patch).
Reduit l'horizon de planification mais necessite du human
engineering.

**General-purpose interactive** : l'agent planifie sur un
horizon long via tool calls et feedback d'execution. Plus
flexible mais plus couteux.

**SWE-Edit (2026)** : introduit des **sous-agents specialises** :
- **Viewer subagent** : recoit les fichiers complets et extrait
  le code pertinent a la demande
- **Editor subagent** : modifie le code
- Separation du contexte pour eviter la "pollution exploratoire"

**Confiance:** MEDIUM (architecture en evolution rapide)

### 4.3 Claude Code Agent Teams — coordination parallele

**Source:** https://code.claude.com/docs/en/agent-teams

Le mecanisme le plus pertinent pour SBFB car il montre comment
**decomposer et paralleliser** des taches de code :

1. **Orchestrateur** analyse l'objectif global et cree un
   **task list** (fichier sur disque)
2. Chaque agent **claim** une tache (marque "in-progress" avec
   son ID)
3. Les autres agents voient et passent a la tache suivante
4. **Dependency tracking** : les taches dependantes ne sont pas
   reclamees tant que les prerequis ne sont pas "complete"
5. Pas de messaging direct inter-agents — le fichier partage
   est le canal de coordination

**Exemple concret :** 16 agents ont construit un compilateur C
en Rust (100,000 LOC) capable de compiler le kernel Linux.
~2,000 sessions, $20,000 en API.

**Transposition SBFB :**
- L'idee votee = l'objectif global
- Le coordinateur local decompose en taches independantes
- Chaque tache est dispatchee a un worker GPU different
- Le task list est stocke dans iroh-docs (partage P2P)
- Les workers claim les taches via le protocole existant

**Confiance:** HIGH (documentation officielle Anthropic)

### 4.4 Aider — single-threaded, repo map

**Repo:** https://github.com/aider-ai/aider

Aider est single-threaded par design. Sa force est la
**repo map** : un index du codebase entier (fichiers, fonctions,
classes, relations) qui permet a l'AI de raisonner sur les
dependances cross-file.

**Non recommande** comme modele pour le dispatch multi-worker
SBFB (pas de parallelisme natif). Mais la **repo map** est un
pattern utile : le coordinateur SBFB pourrait generer un repo map
de la spec de l'idee pour informer la decomposition.

**Confiance:** HIGH

### 4.5 Decomposition des taches : patterns architecturaux

Synthese des patterns identifies dans la litterature (source:
atoms.dev/insights) :

**1. Hierarchique (UniDebugger) :**
- Niveau 1 : problemes simples
- Niveau 2 : active si Niveau 1 echoue
- Niveau 3 : agents specialises pour les cas complexes
- Escalade dynamique selon la complexite

**2. TDAG (Task Decomposition and Agent Generation) :**
- Les sous-agents sont **generes a la demande** par le LLM
- Equipes d'un "skill library" qui evolue
- Decomposition dynamique basee sur les resultats precedents

**3. DAG (Directed Acyclic Graph) :**
- Le workflow est un graphe dirige acyclique
- Chaque noeud est une sous-tache
- Parallelisme explicite : les taches sans dependances
  s'executent en parallele
- Raffinement continu basee sur la performance historique

**Recommandation pour SBFB :** le pattern **DAG** est le plus
adapte car il rend les dependances explicites et permet le
dispatch parallele vers les workers GPU.

**Confiance:** MEDIUM (synthese de multiples sources)

---

## 5. Human-in-the-loop validation apres generation AI

### 5.1 GitHub Environment Protection Rules

**Source:** https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments

GitHub fournit des mecanismes de gate natifs :
- **Required reviewers** : 1 a 6 personnes/teams doivent
  approuver avant deploiement
- **Wait timer** : delai obligatoire (0-43200 minutes)
- **Custom protection rules** : integration tierce
  (Datadog, Honeycomb, etc.)

**Transposition SBFB :** apres qu'un worker GPU genere du code,
le PR (ou son equivalent SBFB) est soumis a une revue
communautaire. Les curators jouent le role de "required reviewers".

### 5.2 pair-review : human-in-the-loop local

**Repo:** https://github.com/in-the-loop-labs/pair-review

Architecture locale (Node.js + Express) avec 3 niveaux d'analyse :
- **Level 1** : analyse les lignes modifiees uniquement
- **Level 2** : coherence dans le fichier
- **Level 3** : patterns architecturaux dans le codebase

Le reviewer humain voit les suggestions AI et les **adopte,
edite ou rejette** individuellement. Le feedback structure
(markdown avec chemins de fichiers et numeros de lignes) est
renvoye a l'agent pour iteration.

**Pattern transposable :** pour SBFB, les curators pourraient
utiliser un workflow similaire — l'AI genere, les curators
revient et approuvent, les changements approuves sont deployes.

**Confiance:** HIGH (code open source, workflow documente)

### 5.3 CodeRabbit : AI review automatise

**Source:** https://coderabbit.ai/

Bot GitHub qui revoit automatiquement les PRs avec des
commentaires inline. Gratuit pour l'open source. Ne remplace
pas la revue humaine mais la facilite.

**Pertinence SBFB :** un premier filtre automatique avant la
revue curator. Le coordinateur pourrait lancer une analyse
statique + AI review avant de proposer le code aux curators.

### 5.4 Pattern recommande pour SBFB

```
Idee atteint seuil
        |
        v
Coordinateur decompose en taches (DAG)
        |
        v
Workers GPU generent le code (sandboxe)
        |
        v
Validation automatique (tests, lint, analyse statique)
        |
        v
Review par curators (quorum Ed25519, ex: 2/3)
        |
        v
Signature FROST du code approuve
        |
        v
Verified deploy sur le reseau
```

Les curators sont les "required reviewers" du monde SBFB. Le
quorum de curators requis peut etre le meme que pour le warrant
canary (t-of-n FROST).

**Confiance:** MEDIUM (synthese de patterns existants, pas de
precedent exact)

### 5.5 Le probleme des PR AI de mauvaise qualite

**Source:** https://thenewstack.io/ai-generated-code-crisis/

Les maintainers open source sont **submerges de PRs AI de
mauvaise qualite** en 2026. Ce risque est directement pertinent
pour SBFB : si le seuil de votes est trop bas, le reseau
generera du code inutile et les curators seront surcharges.

**Mitigations :**
- Seuil de votes eleve (minimum absolu, pas relatif)
- Kudos minimum pour voter (filtre Sybil)
- Rate-limit global sur les dispatches (max N taches/jour)
- Budget compute par idee (evite les boucles infinies)

---

## 6. Synthese : architecture recommandee pour SBFB

### 6.1 Flow complet

```
+-------------------+
| 1. Idee postee    |  (storage_set via bridge)
| Ed25519 signe     |
+--------+----------+
         |
         v
+-------------------+
| 2. Votes arrive   |  (gossip + CRDT G-Counter)
| via gossip        |  chaque noeud compte independamment
+--------+----------+
         |
         v
+-------------------+
| 3. Seuil atteint  |  (policy check local)
| N votes verified  |  premier noeud a observer = proposeur
+--------+----------+
         |
         v
+-------------------+
| 4. TaskProposal   |  (broadcast gossip)
| broadcast         |  dedup par idea_id
+--------+----------+
         |
         v
+-------------------+
| 5. Coordinateur   |  (decomposition DAG)
| decompose spec    |  spec idee → sous-taches independantes
+--------+----------+
         |
         v
+-------------------+
| 6. Workers GPU    |  (dispatch parallele)
| generent code     |  sandbox Docker/WASM
+--------+----------+
         |
         v
+-------------------+
| 7. Validation     |  (automatique)
| tests + lint      |  filtre les echecs evidents
+--------+----------+
         |
         v
+-------------------+
| 8. Curator review |  (quorum t-of-n)
| Ed25519 approvals |  via bridge ou interface dediee
+--------+----------+
         |
         v
+-------------------+
| 9. Deploy         |  (verified deploy standard)
| sur le reseau     |  archive zip + provenance SLSA L1
+-------------------+
```

### 6.2 Composants a construire

| Composant | Existe deja | A construire |
|---|---|---|
| Votes via bridge | Partiellement (storage_set) | CRDT counter + gossip propagation |
| Seuil configurable | Non | Policy engine simple (seuil par tag/categorie) |
| TaskProposal gossip | Non | Nouveau type de message gossip |
| Decomposition tache | Non | LLM-based spec → DAG de sous-taches |
| Dispatch worker GPU | Oui (task_submit) | Lien entre TaskProposal et dispatch |
| Sandbox execution | Oui (worker WASM/Docker) | Rien |
| Curator review | Non | Interface de revue + collecte signatures |
| FROST signing | Oui (canary) | Reutilisation pour approval code |
| Verified deploy | Oui (S14) | Rien |

### 6.3 Seuils recommandes

| Parametre | Valeur suggeree | Rationale |
|---|---|---|
| Votes minimum pour trigger | 10 absolu | Empeche les idees obscures de consommer du compute |
| Kudos minimum pour voter | 1 (non-zero) | Filtre Sybil basique — le votant a contribue |
| Fenetre de vote | 7 jours minimum | Laisse le temps aux pairs de voir et voter |
| Delai post-seuil | 24h | Anti-rush, fenetre de contestation (inspire oSnap) |
| Max dispatches / jour | 3 | Protege les ressources GPU du reseau |
| Budget compute / tache | 30 min GPU max | Empeche les boucles infinies |
| Quorum curator review | 2/3 des curators actifs | Coherent avec Othentic/canary |

---

## 7. Risques et anti-patterns

### 7.1 CRITIQUE : Sybil gaming des votes

**Probleme :** un attaquant cree N identites Ed25519 pour
gonfler les votes et consommer du GPU gratuitement.
**Mitigation :** ponderation par Kudos (deja prevue). Un vote
d'une cle avec 0 Kudos pese 0. La reputation s'acquiert par
contribution reelle au reseau.

### 7.2 CRITIQUE : code genere malveillant

**Probleme :** le LLM genere du code avec des backdoors,
exfiltration de donnees, ou dependances compromises.
**Mitigation :** sandbox d'execution (pas d'acces reseau pour
le worker pendant la generation), curator review obligatoire,
analyse statique automatique, pas de deploy sans signature FROST.

### 7.3 MODERE : split-brain sur le comptage

**Probleme :** deux noeuds voient le seuil atteint a des moments
differents et emettent deux TaskProposals pour la meme idee.
**Mitigation :** deduplication par idea_id. Le premier
TaskProposal vu est le valide. En cas d'egalite, tie-break par
hash le plus bas de la cle publique de l'emetteur.

### 7.4 MODERE : vote fatigue

**Probleme :** trop d'idees, les utilisateurs arretent de voter,
les seuils ne sont jamais atteints.
**Mitigation :** tri par momentum (votes recents ponderes plus
que votes anciens). Alertes quand une idee est proche du seuil.

### 7.5 MINEUR : cout GPU non compense

**Probleme :** les workers depensent du GPU pour generer du code
que la communaute n'utilisera peut-etre jamais.
**Mitigation :** les workers **choisissent** quelles taches
accepter (allowlist existante). Le cout est voluntaire — coherent
avec le modele SBFB (pas de compensation monetaire, Kudos
reputationnel seulement).

---

## 8. Travaux connexes a explorer (hors-scope de cette recherche)

- **Quadratic voting pour la ponderation** (Gitcoin pattern) :
  un votant peut mettre 1 credit = 1 vote, 4 credits = 2 votes,
  9 credits = 3 votes. Empeche la tyrannie de la majorite simple.
  Complexite non triviale, post-v1.0.

- **Delegation de vote** (liquid democracy) : un pair peut
  deleguer son vote a un curator de confiance. Reduit la vote
  fatigue. Scoping review recent :
  https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1598283/full

- **Self-improving agents** (SWE-EVO benchmark) : agents qui
  s'ameliorent sur des taches de long terme. Pertinent si les
  taches generees alimentent un cycle de feedback sur la qualite
  du code.

---

## Sources

### DAO / Governance
- [OpenZeppelin Governor docs](https://docs.openzeppelin.com/contracts/4.x/governance) — HIGH
- [Compound Governor Bravo](https://docs.compound.finance/v2/governance/) — HIGH
- [Compound GovernorBravoDelegate.sol](https://github.com/compound-finance/compound-protocol/blob/master/contracts/Governance/GovernorBravoDelegate.sol) — HIGH
- [Aragon OSx Core](https://devs.aragon.org/docs/osx/how-it-works/core/) — HIGH
- [Aragon "Beyond Proposals" blog](https://blog.aragon.org/beyond-proposals-pt-i-automation-and-the-art-of-not-governing/) — HIGH
- [oSnap by UMA](https://medium.com/uma-project/announcing-osnap-gasless-snapshot-voting-with-on-chain-execution-by-uma-7374ed729b28) — MEDIUM
- [Tally docs](https://docs.tally.xyz/) — HIGH

### Threshold / P2P
- [FROST ZcashFoundation](https://github.com/ZcashFoundation/frost) — HIGH
- [FROST protocol explained](https://frost.zfnd.org/frost.html) — HIGH
- [Othentic AVS examples](https://github.com/Othentic-Labs/avs-examples) — MEDIUM
- [Othentic leader election docs](https://docs.othentic.xyz/main/learn/advanced-concepts/leader-election) — MEDIUM
- [EigenLayer AVS guide](https://avaprotocol.org/blog/a-guide-to-eigenlayer-avs-actively-validated-services-on-ethereum) — MEDIUM
- [PeerVote paper (2009)](https://link.springer.com/chapter/10.1007/978-3-642-02627-0_5) — HIGH
- [CRDT Wikipedia](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) — HIGH
- [gossip-crdt implementation](https://github.com/JDrit/gossip-crdt) — MEDIUM
- [Raft consensus](https://en.wikipedia.org/wiki/Raft_(algorithm)) — HIGH
- [BLS threshold signatures (ZetaChain)](https://www.zetachain.com/blog/threshold-bls-signature-for-decentralized-asset-control) — MEDIUM

### AI Code Generation
- [OpenHands (OpenDevin)](https://github.com/All-Hands-AI/OpenHands) — HIGH
- [OpenHands paper](https://arxiv.org/abs/2407.16741) — HIGH
- [SWE-Agent](https://github.com/princeton-nlp/SWE-agent) — HIGH
- [Claude Code Agent Teams](https://code.claude.com/docs/en/agent-teams) — HIGH
- [Anthropic C compiler case study](https://www.anthropic.com/engineering/building-c-compiler) — HIGH
- [Aider](https://github.com/aider-ai/aider) — HIGH
- [Sweep AI](https://github.com/sweepai/sweep) — MEDIUM
- [Task decomposition survey](https://atoms.dev/insights/task-decomposition-for-coding-agents-architectures-advancements-and-future-directions/a95f933f2c6541fc9e1fb352b429da15) — MEDIUM

### Human-in-the-loop
- [pair-review](https://github.com/in-the-loop-labs/pair-review) — HIGH
- [GitHub environment protection](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments) — HIGH
- [AI-generated PR crisis](https://thenewstack.io/ai-generated-code-crisis/) — MEDIUM
- [Gitcoin quadratic funding](https://gitcoin.co/mechanisms/quadratic-funding) — HIGH
- [Delegated voting scoping review](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1598283/full) — MEDIUM
