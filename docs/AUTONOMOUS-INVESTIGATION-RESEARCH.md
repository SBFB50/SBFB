# Recherche ULTRA DEEP : Projets Open Source pour Investigation Autonome

**Date :** 2026-04-05
**Objectif :** Identifier les meilleurs projets et patterns pour transformer NEXUS en systeme d'investigation AUTONOME CONTINU 24/7.

---

## Table des matieres

1. [Top 10 Projets a Integrer / S'inspirer](#1-top-10-projets)
2. [Architecture Recommandee pour la Boucle Autonome](#2-architecture-boucle-autonome)
3. [Ce qui Existe vs Ce qu'il Faut Construire](#3-existe-vs-construire)
4. [Plan d'Action Concret](#4-plan-daction)
5. [Sources](#5-sources)

---

## 1. Top 10 Projets

### 1.1 CRIS — Criminal Reasoning Intelligence System

**Ce que c'est :** Systeme multi-agents specialises pour l'investigation criminelle, avec Neo4j + ChromaDB + Streamlit. Exactement le meme stack que NEXUS.

**Pourquoi c'est critique pour NEXUS :**
- 6 agents specialises : Link Agent (traversee de graphe), Profiler Agent (profils psychologiques), Geo-Intel Agent (analyse spatiale/Rossmo), Witness Agent (credibilite des temoignages), Predictor Agent (simulations Monte Carlo), OSINT Agent (empreinte numerique)
- Orchestrateur central qui delegue aux agents specialises
- Modele POLE (Person-Object-Location-Event) dans Neo4j — exactement ce que NEXUS utilise deja
- Audit trail complet pour chaque decision (tracabilite legale)
- Scores de confiance sur chaque prediction

**Ce qu'on prend :**
- L'architecture multi-agents specialises (pas un seul LLM pour tout)
- Le pattern Orchestrateur -> Agents specialises
- Le Witness Agent (analyse de credibilite des temoignages)
- Le Predictor Agent (simulations Monte Carlo sur les hypotheses)
- La formule de Rossmo pour l'analyse geospatiale

**Adaptation NEXUS :** Remplacer Google ADK/Gemini par notre stack Ollama multi-modeles. L'orchestrateur utilise gemma4:e4b pour le routage, nexus:26b pour l'analyse profonde, deepseek-r1 pour la verification.

**Ref :** https://medium.com/@francotesei/building-a-multi-agent-criminal-intelligence-system-how-ai-agents-neo4j-and-network-analysis-a9fff194dc12

---

### 1.2 POPPER — Automated Hypothesis Validation (Stanford)

**Ce que c'est :** Framework agentique pour la validation rigoureuse d'hypotheses par falsification sequentielle (principe de Karl Popper). Publie a ICML 2025.

**Pourquoi c'est critique pour NEXUS :**
- Exactement le pattern dont NEXUS a besoin : au lieu de chercher a CONFIRMER les hypotheses (biais de confirmation), POPPER cherche a les FALSIFIER
- Controle strict de l'erreur Type-I (faux positifs) via tests sequentiels
- Deux agents : Experiment Design Agent (concoit les tests) + Experiment Execution Agent (les execute)
- Support de LLMs locaux via OpenAI-compatible APIs (Ollama fonctionne)
- Mode ReAct agent pour un raisonnement ameliore
- 10x plus rapide que les scientifiques humains pour la validation

**Ce qu'on prend :**
- Le paradigme de FALSIFICATION au lieu de confirmation
- La boucle : extraire implications mesurables -> concevoir test -> executer -> aggreger
- Le controle statistique (E-values) pour eviter les faux positifs
- L'integration directe (Python, compatible Ollama)

**Adaptation NEXUS :** Integrer comme module `nexus/core/hypothesis_falsifier.py`. Chaque hypothese passe par POPPER avant d'etre scoree. deepseek-r1 concoit les tests de falsification, nexus:26b les execute contre les preuves.

**Ref :** https://github.com/snap-stanford/POPPER

---

### 1.3 LangGraph — Orchestration de Graphes d'Agents

**Ce que c'est :** Framework de LangChain pour construire des agents comme des graphes cycliques avec etat, points d'arret, et boucles de raisonnement.

**Pourquoi c'est critique pour NEXUS :**
- Modele de graphe dirige CYCLIQUE (pas lineaire) — essentiel pour la boucle d'investigation continue
- Etat persistant entre les cycles (checkpoints)
- Points d'interruption humain-dans-la-boucle (l'investigateur peut intervenir)
- Support natif du pattern ReAct (Reasoning + Acting)
- Branchement conditionnel (si nouvelle preuve -> re-evaluer, sinon -> monitoring)
- Self-correction : si un outil retourne un resultat ambigu, l'agent essaie une autre strategie

**Ce qu'on prend :**
- L'architecture de graphe cyclique pour la boucle OODA
- Le modele d'etat partage entre agents
- Les checkpoints persistants (resume apres crash/restart)
- Le pattern de self-correction

**Adaptation NEXUS :** LangGraph devient l'orchestrateur central de la boucle autonome. Chaque noeud du graphe = une etape de l'investigation (observe/orient/decide/act). Compatible Ollama via LangChain.

**Ref :** https://github.com/langchain-ai/langgraph

---

### 1.4 SpiderFoot — Automatisation OSINT (200+ modules)

**Ce que c'est :** Plateforme d'automatisation OSINT avec 200+ modules, modele publisher/subscriber, et moteur de correlation YAML.

**Pourquoi c'est critique pour NEXUS :**
- 200+ modules d'OSINT (DNS, WHOIS, breach checks, social media, dark web, etc.)
- Architecture publisher/subscriber : les decouvertes d'un module alimentent les autres automatiquement
- Correlation engine (v4.0) : 37 regles YAML pre-definies + custom
- Cible tous les types d'entites : IP, domaine, email, telephone, username, nom, Bitcoin
- Export CSV/JSON/GEXF + backend SQLite

**Ce qu'on prend :**
- Le modele publisher/subscriber pour le pipeline de decouverte
- Les modules de reconnaissance (DNS, WHOIS, breach, social media)
- Le moteur de correlation YAML
- L'architecture modulaire pour ajouter de nouvelles sources

**Adaptation NEXUS :** Ne pas integrer SpiderFoot en entier (trop gros, focus securite). Extraire les modules pertinents pour les cold cases (breach lookups, social media, historique web) et les adapter en modules NEXUS. Utiliser le meme pattern publisher/subscriber.

**Ref :** https://github.com/smicallef/spiderfoot

---

### 1.5 GPT-Researcher — Agent de Recherche Profonde Autonome

**Ce que c'est :** Agent autonome qui conduit des recherches approfondies sur n'importe quel sujet en combinant recherche web, scraping, et LLMs.

**Pourquoi c'est critique pour NEXUS :**
- Architecture multi-agents : Planner (genere les questions) + Execution (collecte) + Publisher (rapport)
- Deep Research : exploration recursive en arbre, dive dans les sous-sujets
- Aggregation statistique pour eviter les biais (20+ sources)
- Compatible avec n'importe quel provider LLM
- Rapports de 5-6 pages avec citations
- Support de documents locaux (PDF, CSV, Excel, Word, Markdown)
- Observabilite via LangSmith

**Ce qu'on prend :**
- Le pattern Planner/Execution/Publisher pour les sessions de recherche
- L'exploration recursive en arbre pour les sujets complexes
- L'aggregation multi-sources pour la fiabilite
- Le mode recherche locale (sur les documents du dossier)

**Adaptation NEXUS :** Adapter comme module `nexus/core/deep_researcher.py`. Le Planner utilise nexus:26b pour generer les questions d'investigation, l'execution utilise SearXNG/Robin pour la collecte, gemma4:e4b pour le filtrage.

**Ref :** https://github.com/assafelovic/gpt-researcher

---

### 1.6 Reflexion — Agents Auto-Correcteurs (NeurIPS 2023)

**Ce que c'est :** Framework ou les agents apprennent de leurs erreurs via reflexion verbale et memoire episodique, SANS mise a jour des poids.

**Pourquoi c'est critique pour NEXUS :**
- Memoire a court terme (trajectoire actuelle) + memoire a long terme (reflexions distillees)
- L'agent identifie ses propres erreurs et s'auto-suggere des lecons
- Pas de fine-tuning necessaire — fonctionne avec n'importe quel LLM
- Amelioration significative sur les taches de raisonnement (HotPotQA)
- Extension Multi-Agent Reflexion (MAR) : plusieurs critiques avec des perspectives differentes

**Ce qu'on prend :**
- Le pattern de reflexion episodique : apres chaque cycle d'investigation, l'agent reflechit sur ce qu'il a bien/mal fait
- La memoire a long terme des lecons apprises
- Le pattern MAR pour la pensee adversariale (Advocate + Devil's Advocate)

**Adaptation NEXUS :** Integrer comme pattern dans la boucle OODA. Apres chaque cycle (6h clearweb, 24h dark web), le systeme reflechit : "Qu'ai-je appris ? Quelles hypotheses sont renforcees/affaiblies ? Quelles erreurs ai-je commises ?" Stocke dans SQLite `reflection_log`.

**Ref :** https://github.com/noahshinn/reflexion

---

### 1.7 Mem0 — Couche de Memoire Persistante pour Agents IA

**Ce que c'est :** Couche de memoire open source qui s'intercale entre l'application et le LLM. Extrait, stocke, et recupere automatiquement les informations pertinentes.

**Pourquoi c'est critique pour NEXUS :**
- Extraction automatique d'information pertinente des conversations/analyses
- Reduction de 90% des tokens par requete (ne charge que le contexte pertinent)
- Compatible Ollama pour les LLMs locaux
- Fonctionne avec LangGraph, CrewAI, et d'autres frameworks
- Memoire par entite/utilisateur/session

**Ce qu'on prend :**
- Le pattern de memoire contextuelle automatique
- La reduction intelligente du contexte (crucial avec 16GB VRAM)
- L'integration Ollama native

**Adaptation NEXUS :** Utiliser pour la memoire de l'investigateur virtuel. Chaque analyse enrichit la memoire du systeme. Quand une nouvelle preuve arrive, Mem0 recupere automatiquement le contexte pertinent des analyses precedentes.

**Ref :** https://github.com/mem0ai/mem0

---

### 1.8 Taranis AI — OSINT Continu avec NLP

**Ce que c'est :** Outil OSINT avance utilisant l'IA et le NLP pour la collecte continue d'information et l'analyse situationnelle.

**Pourquoi c'est critique pour NEXUS :**
- Collecte CONTINUE de multiples sources (web, RSS, Twitter, email, Slack)
- Workers asynchrones via Celery pour le traitement des articles
- NLP automatique pour enrichir le contenu collecte
- Systeme collaboratif de ranking et de notation
- Generation automatique de rapports et summaries
- Stack : Vue.js + Flask + Celery + PostgreSQL

**Ce qu'on prend :**
- Le pattern de collecte continue multi-sources avec workers
- Le systeme de ranking collaboratif
- L'architecture asynchrone (Celery) pour le traitement en background

**Adaptation NEXUS :** S'inspirer de l'architecture de workers pour le monitoring continu. Remplacer Celery par APScheduler (plus leger, in-process) vu qu'on n'a pas besoin de distribution multi-machines.

**Ref :** https://github.com/taranis-ai/taranis-ai

---

### 1.9 SAGE — Agents Auto-Evolutifs avec Reflexion et Memoire

**Ce que c'est :** Framework d'agents qui evoluent via feedback iteratif, reflexion, et un systeme de memoire base sur la courbe d'oubli d'Ebbinghaus.

**Pourquoi c'est critique pour NEXUS :**
- Systeme de memoire dual : STM (Short-Term Memory) + LTM (Long-Term Memory)
- Courbe d'oubli d'Ebbinghaus : priorise dynamiquement l'information de haute valeur, elague les donnees triviales
- 3 agents : User, Assistant, Checker (pattern de verification)
- Amelioration de 57-100% sur les modeles open source
- Feedback iteratif + reflexion = amelioration continue

**Ce qu'on prend :**
- Le systeme de memoire dual STM/LTM avec priorisation dynamique
- La courbe d'oubli pour le nettoyage automatique de la memoire
- Le pattern Checker (agent de verification)

**Adaptation NEXUS :** Le LTM stocke les patterns d'investigation appris au fil du temps. Le STM contient le contexte de l'investigation en cours. La courbe d'oubli determine quand les anciennes pistes non-confirmees perdent leur priorite.

**Ref :** https://arxiv.org/html/2409.00872v2

---

### 1.10 Neo4j POLE + GraphRAG + GDS

**Ce que c'est :** Combinaison du modele POLE (Person-Object-Location-Event), de GraphRAG (Microsoft), et des algorithmes Neo4j Graph Data Science pour l'investigation criminelle.

**Pourquoi c'est critique pour NEXUS :**
- **POLE** : Modele de donnees standard pour les investigations policieres, deja aligne avec notre schema Neo4j
- **GraphRAG** : Extraction automatique d'entites + relations via LLM, construction de knowledge graph, detection de communautes via Leiden, resumes en langage naturel
- **Neo4j GDS** : Louvain (detection de communautes), PageRank (noeuds influents), Betweenness Centrality (connecteurs), Label Propagation, Triangle Count
- Le tout est open source et deja dans notre stack

**Ce qu'on prend :**
- GraphRAG pour l'extraction automatique d'entites et relations des preuves
- Algorithmes GDS pour l'analyse de reseau criminel (communautes, centralite, chemins)
- Le contexte provider Neo4j pour l'Agent Framework
- Les requetes Cypher pour la detection de patterns

**Adaptation NEXUS :** Notre schema Neo4j est deja POLE-compatible. Ajouter neo4j-graphrag pour l'extraction automatique. Utiliser GDS pour detecter les communautes dans les reseaux de suspects.

**Refs :**
- https://github.com/neo4j-graph-examples/pole
- https://neo4j.com/blog/developer/microsoft-graphrag-neo4j/
- https://github.com/neo4j-contrib/ms-graphrag-neo4j

---

## 2. Architecture Boucle Autonome

### 2.1 Pattern OODA (Observe-Orient-Decide-Act) pour NEXUS

L'architecture de daemon d'investigation continue repose sur le pattern OODA, adapte aux cold cases :

```
                    +--------------------------------------------------+
                    |              BOUCLE OODA PRINCIPALE               |
                    |              (tourne 24/7 en daemon)              |
                    +--------------------------------------------------+
                                          |
              +-----------+-----------+-----------+-----------+
              |           |           |           |           |
              v           v           v           v           v
         +---------+ +---------+ +---------+ +---------+ +----------+
         | OBSERVE | | ORIENT  | | DECIDE  | |   ACT   | | REFLECT  |
         +---------+ +---------+ +---------+ +---------+ +----------+
              |           |           |           |           |
              v           v           v           v           v
         Collecte    Analyse &    Prioriser   Executer    Reflechir
         passive     Correler     les actions  les         sur le
         de donnees  nouvelles    (quelle      actions     cycle
         (SearXNG,   donnees     hypothese    (recherche,  (lecons,
         Robin,      avec        re-evaluer   rapport,     erreurs,
         RSS)        existantes  en premier?) alerte)      pivots)
```

### 2.2 Architecture Detaillee du Daemon

```
+================================================================+
|                    NEXUS AUTONOMOUS DAEMON                       |
|                    (nexus/autonomous/daemon.py)                  |
+================================================================+
|                                                                  |
|  +----------------------------------------------------------+   |
|  |  SCHEDULER (APScheduler BackgroundScheduler)              |   |
|  |                                                           |   |
|  |  Cron Jobs:                                               |   |
|  |  - every 6h  : clearweb_sweep (SearXNG)                  |   |
|  |  - every 24h : darkweb_sweep (Robin/Tor)                 |   |
|  |  - every 12h : hypothesis_reeval_cycle                   |   |
|  |  - every 1h  : new_evidence_check                        |   |
|  |  - every 48h : deep_research_session                     |   |
|  |  - every 7d  : full_adversarial_review                   |   |
|  +----------------------------------------------------------+   |
|                             |                                    |
|                             v                                    |
|  +----------------------------------------------------------+   |
|  |  ORCHESTRATOR (LangGraph StateGraph)                      |   |
|  |                                                           |   |
|  |  Nodes:                                                   |   |
|  |  [observe] -> [orient] -> [decide] -> [act] -> [reflect] |   |
|  |       ^                                            |      |   |
|  |       +--------------------------------------------+      |   |
|  |                    (boucle continue)                       |   |
|  |                                                           |   |
|  |  State:                                                   |   |
|  |  - active_cases: List[CaseState]                          |   |
|  |  - pending_actions: PriorityQueue                         |   |
|  |  - hypothesis_scores: Dict[str, float]                    |   |
|  |  - last_findings: List[Finding]                           |   |
|  |  - reflection_log: List[Reflection]                       |   |
|  +----------------------------------------------------------+   |
|                             |                                    |
|                             v                                    |
|  +----------------------------------------------------------+   |
|  |  AGENT POOL (Agents Specialises)                          |   |
|  |                                                           |   |
|  |  [OSINT Agent]        gemma4:e4b  -> SearXNG/Robin        |   |
|  |  [Entity Agent]       gemma4:e4b  -> Neo4j/ChromaDB       |   |
|  |  [Hypothesis Agent]   nexus:26b   -> SQLite               |   |
|  |  [Falsification Agent] deepseek-r1 -> POPPER              |   |
|  |  [Link Agent]         nexus:26b   -> Neo4j GDS            |   |
|  |  [Witness Agent]      deepseek-r1 -> Analyse credibilite  |   |
|  |  [Adversarial Agent]  deepseek-r1 -> Devil's Advocate     |   |
|  |  [Research Agent]     nexus:26b   -> Deep Research         |   |
|  +----------------------------------------------------------+   |
|                             |                                    |
|                             v                                    |
|  +----------------------------------------------------------+   |
|  |  MEMORY LAYER                                             |   |
|  |                                                           |   |
|  |  STM (Short-Term): contexte du cycle actuel               |   |
|  |  LTM (Long-Term): lecons apprises, patterns identifies    |   |
|  |  Episodic: reflexions apres chaque cycle                  |   |
|  |  Semantic: ChromaDB embeddings pour recherche similaire   |   |
|  |                                                           |   |
|  |  Backend: SQLite (structured) + ChromaDB (vectoriel)      |   |
|  +----------------------------------------------------------+   |
|                             |                                    |
|                             v                                    |
|  +----------------------------------------------------------+   |
|  |  VRAM SERIALIZER (nexus/llm/vram_manager.py)              |   |
|  |                                                           |   |
|  |  Queue de taches par taille de modele :                   |   |
|  |  1. Batch gemma4:e4b  (~3GB)  -- toutes taches legeres   |   |
|  |  2. nexus:26b         (~15GB) -- analyse profonde         |   |
|  |  3. deepseek-r1:14b   (~9GB)  -- verification             |   |
|  |  4. nomic-embed-text  (~1GB)  -- embeddings               |   |
|  |                                                           |   |
|  |  Mutex global : jamais deux gros modeles en parallele     |   |
|  +----------------------------------------------------------+   |
+================================================================+
```

### 2.3 Detail de Chaque Phase OODA

#### OBSERVE (Collecte passive)

```python
# Declenchee par : scheduler (6h clearweb, 24h darkweb)
# Modele utilise : gemma4:e4b (reformulation + filtrage)

async def observe(state: InvestigationState) -> InvestigationState:
    """
    Pour chaque case active :
    1. Generer des requetes de recherche (gemma4:e4b)
       - Basees sur les entites connues
       - Basees sur les hypotheses en cours
       - Variantes linguistiques / alias
    2. Executer les recherches (SearXNG clearweb / Robin darkweb)
    3. Filtrer les resultats (gemma4:e4b)
       - Deduplication semantique via ChromaDB
       - Scoring de pertinence
    4. Stocker les resultats bruts dans SQLite + ChromaDB
    """
```

#### ORIENT (Analyse & Correlation)

```python
# Declenchee par : nouvelles donnees dans OBSERVE
# Modeles utilises : gemma4:e4b (extraction) + nexus:26b (analyse)

async def orient(state: InvestigationState) -> InvestigationState:
    """
    Pour chaque nouveau resultat pertinent :
    1. Extraire les entites (gemma4:e4b) -> Neo4j
    2. Generer embeddings (nomic-embed-text) -> ChromaDB
    3. Detecter les connexions avec le graphe existant (Neo4j GDS)
       - Nouvelles aretes entre noeuds existants ?
       - Communautes modifiees ?
       - Centralite changee ?
    4. Analyse profonde (nexus:26b)
       - Qu'est-ce que cette information change ?
       - Quelles hypotheses sont affectees ?
       - Y a-t-il de nouvelles pistes ?
    """
```

#### DECIDE (Priorisation)

```python
# Declenchee par : ORIENT terminee
# Modele utilise : nexus:26b (raisonnement complexe)

async def decide(state: InvestigationState) -> InvestigationState:
    """
    Decisions a prendre :
    1. Quelles hypotheses re-evaluer en priorite ?
       - Celles avec le plus grand delta de score potentiel
       - Celles avec de nouvelles preuves contradictoires
    2. Quelles recherches supplementaires lancer ?
       - Entites non-explorees
       - Pistes ouvertes non-investiguees
    3. Faut-il generer un rapport / alerte ?
       - Score > seuil => alerte
       - Changement significatif => snapshot
    4. File de priorite (PriorityQueue)
       - Score = urgence * impact * fraicheur
    """
```

#### ACT (Execution)

```python
# Declenchee par : DECIDE a rempli la file de priorite
# Modeles utilises : selon l'action

async def act(state: InvestigationState) -> InvestigationState:
    """
    Executer les actions de la file :
    1. Re-evaluation d'hypothese (nexus:26b)
       - Nouveau score + justification
       - Snapshot dans SQLite
    2. Falsification (deepseek-r1 via POPPER)
       - Test de chaque hypothese contre les preuves
       - Mise a jour du score si falsifie
    3. Recherche approfondie (GPT-Researcher pattern)
       - Planner genere les questions
       - Execution collecte les infos
       - Publisher synthetise
    4. Generation de rapport / alerte
       - Si changement significatif
    """
```

#### REFLECT (Meta-cognition)

```python
# Declenchee par : fin d'un cycle complet OODA
# Modele utilise : nexus:26b (reflexion profonde)

async def reflect(state: InvestigationState) -> InvestigationState:
    """
    Auto-evaluation du cycle (inspire de Reflexion + SAGE) :
    1. Qu'ai-je decouvert de nouveau ?
    2. Quelles hypotheses ont change ? Pourquoi ?
    3. Ai-je commis des erreurs de raisonnement ?
    4. Y a-t-il un biais de confirmation dans mes analyses ?
    5. Quelles pistes devrais-je explorer differemment ?
    6. La memoire a long terme doit-elle etre mise a jour ?

    Stockage : SQLite reflection_log + ChromaDB (embedding pour
    recherche semantique des reflexions passees)

    Pattern Devil's Advocate :
    - deepseek-r1 joue le role d'avocat du diable
    - Questionne les conclusions de nexus:26b
    - Force la consideration d'alternatives
    """
```

### 2.4 Gestion des Priorites entre Cases

```python
class CasePriority:
    """
    Score de priorite pour chaque case active.
    Recalcule apres chaque cycle OODA.
    """
    def calculate(self, case: Case) -> float:
        score = 0.0

        # Facteur 1 : Fraicheur des donnees
        # Plus il y a longtemps qu'on n'a rien trouve, plus on intensifie
        hours_since_last_finding = (now - case.last_finding_date).hours
        if hours_since_last_finding > 168:  # 7 jours
            score += 30  # Boost pour relancer l'investigation
        elif hours_since_last_finding < 24:
            score += 50  # Donnees fraiches, capitaliser

        # Facteur 2 : Volatilite des hypotheses
        # Si les scores bougent beaucoup, il y a de l'action
        hypothesis_volatility = std_dev(case.recent_score_changes)
        score += hypothesis_volatility * 20

        # Facteur 3 : Nombre de pistes ouvertes non-explorees
        open_leads = count(case.leads.where(status='open'))
        score += min(open_leads * 5, 30)

        # Facteur 4 : Priorite manuelle de l'investigateur
        score += case.manual_priority * 10

        return score
```

### 2.5 Anti-Biais de Confirmation

NEXUS integre 4 mecanismes anti-biais :

1. **Falsification systematique (POPPER)** : Chaque hypothese est soumise a des tentatives de falsification AVANT d'etre scoree positivement

2. **Devil's Advocate (deepseek-r1)** : Un agent dedie joue l'avocat du diable sur chaque conclusion. Il est OBLIGE de trouver des contre-arguments

3. **Pensee adversariale structuree (ASPIC+)** : Les arguments sont modelises comme des structures avec des regles strictes et defeasibles. Un argument peut "attaquer" un autre argument

4. **Multi-perspective (MAR - Multi-Agent Reflexion)** : Plusieurs "personnalites" de critique evaluent chaque hypothese avec des perspectives differentes :
   - Le Sceptique : doute de tout
   - Le Procureur : cherche a inculper
   - L'Avocat de la defense : cherche a innocenter
   - L'Analyste froid : ne regarde que les chiffres

---

## 3. Ce qui Existe vs Ce qu'il Faut Construire

### DEJA DISPONIBLE (a integrer)

| Composant | Projet | Effort d'integration | Priorite |
|-----------|--------|---------------------|----------|
| Falsification d'hypotheses | POPPER (Stanford) | ~8h (adapter au stack Ollama) | CRITIQUE |
| Orchestration de graphe d'agents | LangGraph | ~12h (graphe OODA + agents) | CRITIQUE |
| Memoire persistante intelligente | Mem0 | ~6h (installer + configurer Ollama) | HAUTE |
| Reconnaissance OSINT modulaire | SpiderFoot (modules) | ~10h (extraire modules pertinents) | HAUTE |
| Recherche profonde autonome | GPT-Researcher | ~8h (adapter au stack local) | HAUTE |
| Reflexion episodique | Reflexion (pattern) | ~4h (implementer le pattern) | MOYENNE |
| Knowledge graph auto-construction | GraphRAG + Neo4j | ~10h (integrer ms-graphrag-neo4j) | HAUTE |
| Analyse de graphe criminel | Neo4j GDS | ~6h (algorithmes communaute/centralite) | MOYENNE |
| Collecte clearweb continue | SearXNG API | ~4h (wrapper Python + scheduler) | DEJA PREVU |
| Collecte dark web continue | Robin API | ~4h (wrapper Python + scheduler) | DEJA PREVU |
| Reconnaissance faciale | CompreFace | ~4h (Docker + API REST) | BASSE |

### A CONSTRUIRE (custom NEXUS)

| Composant | Description | Effort | Priorite |
|-----------|-------------|--------|----------|
| Daemon autonome OODA | Boucle principale observe/orient/decide/act/reflect | ~20h | CRITIQUE |
| VRAM Serializer | Gestionnaire de queue GPU pour serialiser les modeles | ~6h | CRITIQUE |
| Agent Orchestrator | Routage intelligent vers agents specialises | ~12h | CRITIQUE |
| Adversarial Agent | Agent avocat du diable (deepseek-r1) | ~8h | HAUTE |
| Witness Credibility Agent | Analyse de credibilite des temoignages | ~8h | HAUTE |
| Geo-Intel Agent | Analyse spatiale (formule de Rossmo) | ~6h | MOYENNE |
| Predictor Agent | Simulations Monte Carlo sur hypotheses | ~8h | MOYENNE |
| Case Priority Manager | Priorisation dynamique entre cases | ~4h | HAUTE |
| Reflection Logger | Stockage + indexation des reflexions | ~4h | MOYENNE |
| Anti-bias Engine | 4 mecanismes de mitigation des biais | ~12h | HAUTE |
| Investigation Timeline | Timeline auto-generee des evenements | ~6h | MOYENNE |
| Alert Escalation | Systeme d'alertes graduees | ~4h | HAUTE |

### TOTAL ESTIME pour le systeme autonome

| Categorie | Heures |
|-----------|--------|
| Integration de projets existants | ~76h |
| Construction custom | ~98h |
| Tests + integration | ~30h |
| **TOTAL** | **~204h** |

Ce total s'ajoute aux ~207h du plan de developpement existant (6 phases).
**Mais** il y a un recouvrement significatif avec les phases 3 (monitoring) et 4 (hypotheses) du plan existant, estimable a ~40h.

**Total reel supplementaire : ~164h (~20 jours de travail)**

---

## 4. Plan d'Action

### Ordre d'integration recommande

**Phase A — Fondations Autonomes (~40h)**
1. VRAM Serializer (prerequis pour tout)
2. LangGraph : graphe OODA de base
3. Daemon APScheduler
4. Agent Orchestrator minimal

**Phase B — Intelligence (~36h)**
5. POPPER : falsification d'hypotheses
6. Adversarial Agent (Devil's Advocate)
7. Anti-bias Engine
8. Mem0 : memoire persistante

**Phase C — Collection (~28h)**
9. SearXNG wrapper + scheduler
10. Robin wrapper + scheduler
11. SpiderFoot modules (breach, social, historique)
12. GPT-Researcher pattern pour les recherches profondes

**Phase D — Analyse Avancee (~30h)**
13. GraphRAG + Neo4j : extraction automatique
14. Neo4j GDS : communautes + centralite
15. Witness Credibility Agent
16. Geo-Intel Agent (Rossmo)
17. Predictor Agent (Monte Carlo)

**Phase E — Meta-cognition (~20h)**
18. Reflexion episodique (pattern Reflexion)
19. SAGE : memoire STM/LTM avec courbe d'oubli
20. Case Priority Manager
21. Reflection Logger

**Phase F — Polish (~10h)**
22. Alert Escalation
23. Investigation Timeline
24. Integration tests
25. Documentation

---

## 5. Sources

### Projets GitHub

- [CRIS - Criminal Reasoning Intelligence System](https://medium.com/@francotesei/building-a-multi-agent-criminal-intelligence-system-how-ai-agents-neo4j-and-network-analysis-a9fff194dc12)
- [POPPER - Stanford Hypothesis Falsification](https://github.com/snap-stanford/POPPER)
- [LangGraph - Agent Graph Orchestration](https://github.com/langchain-ai/langgraph)
- [SpiderFoot - OSINT Automation](https://github.com/smicallef/spiderfoot)
- [GPT-Researcher - Autonomous Research Agent](https://github.com/assafelovic/gpt-researcher)
- [Reflexion - Verbal Reinforcement Learning](https://github.com/noahshinn/reflexion)
- [Mem0 - Universal Memory Layer](https://github.com/mem0ai/mem0)
- [Taranis AI - Continuous OSINT](https://github.com/taranis-ai/taranis-ai)
- [SAGE - Self-evolving Agents](https://arxiv.org/html/2409.00872v2)
- [Neo4j POLE Crime Investigation](https://github.com/neo4j-graph-examples/pole)
- [Microsoft GraphRAG + Neo4j](https://github.com/neo4j-contrib/ms-graphrag-neo4j)
- [CrewAI - Multi-Agent Framework](https://github.com/crewAIInc/crewAI)
- [Microsoft Agent Framework (AutoGen + Semantic Kernel)](https://github.com/microsoft/autogen)
- [Robin - Dark Web OSINT](https://github.com/apurvsinghgautam/robin)
- [OSINT-with-LLM](https://github.com/mouna23/OSINT-with-LLM)
- [OSINT AI Agent (LangGraph)](https://github.com/dazzyddos/OSINT_AI_Agent)
- [theHarvester](https://github.com/laramies/theHarvester)
- [recon-ng](https://github.com/lanmaster53/recon-ng)
- [CompreFace - Facial Recognition](https://github.com/exadel-inc/CompreFace)

### Articles et Documentation

- [OODA Loop Pattern for Autonomous AI Agents](https://dev.to/yedanyagamiaicmd/the-ooda-loop-pattern-for-autonomous-ai-agents-how-i-built-a-self-improving-system-2ap3)
- [Harnessing OODA Loop for Agentic AI](https://www.sogeti.com/featured-articles/harnessing-the-ooda-loop-for-agentic-ai/)
- [NVIDIA LLo11yPop OODA Architecture](https://developer.nvidia.com/blog/optimizing-data-center-performance-with-ai-agents-and-the-ooda-loop-strategy/)
- [Devil's Advocate Architecture for Multi-Agent AI](https://medium.com/@jsmith0475/the-devils-advocate-architecture-how-multi-agent-ai-systems-mirror-human-decision-making-9c9e6beb09da)
- [ACRD Protocol - Adversarial Claim Robustness](https://www.mdpi.com/2673-2688/6/7/147)
- [ReAct Pattern with LangGraph](https://machinelearningmastery.com/building-react-agents-with-langgraph-a-beginners-guide/)
- [Top Agentic AI Frameworks 2026](https://www.alphamatch.ai/blog/top-agentic-ai-frameworks-2026)
- [CrewAI Framework 2025 Review](https://latenode.com/blog/ai-frameworks-technical-infrastructure/crewai-framework/crewai-framework-2025-complete-review-of-the-open-source-multi-agent-ai-platform)
- [Neo4j POLE Data Model Guide](https://guides.neo4j.com/sandbox/pole/index.html)
- [Neo4j GDS Community Detection](https://neo4j.com/docs/graph-data-science/current/algorithms/community/)
- [GraphRAG Integration Neo4j](https://neo4j.com/blog/developer/microsoft-graphrag-neo4j/)
- [SearXNG Search API](https://docs.searxng.org/dev/search_api.html)
- [Mem0 Research - 26% Accuracy Boost](https://mem0.ai/research)
- [OSINT Framework Guide 2026](https://www.penligent.ai/hackinglabs/osint-framework-a-comprehensive-guide-to-open-source-intelligence-in-2026/)
- [Best OSINT Tools 2026](https://shadowdragon.io/blog/best-osint-tools/)
- [Trace Labs - Crowdsourced OSINT](https://www.tracelabs.org/)
- [OSINT-BIBLE GitHub](https://github.com/frangelbarrera/OSINT-BIBLE)
- [Awesome AI OSINT](https://github.com/ubikron/Awesome-AI-OSINT)
