# Prompt/Agent Packs pour SBFB

Statut: note de recherche, hors sprint, sans verdict de phase.

Date: 2026-06-28.

Objet: analyser les projets open source existants pour creer des systemes de
prompts/agents sur mesure, puis proposer un modele SBFB pour que chaque noeud
puisse installer ces systemes et les utiliser depuis Ideas Hub, Factory ou RRV.

## Verdict court

Oui, il existe deja beaucoup de briques open source pour creer des systemes de
prompts, agents, workflows, RAG, evals et runtimes locaux. Mais il n'existe pas
un projet qui couvre exactement le besoin SBFB:

- artefact publiable sur un protocole P2P;
- verifiable par hash, signature, provenance et licence;
- installable localement par chaque noeud;
- utilisable par Ideas Hub et Factory sans donner d'autorite a l'app sandboxee;
- compatible Claude, GPT, Ollama, llama.cpp, LocalAI et reseau SBFB;
- anti-capture: pas de marketplace centrale, pas de ranking global, pas de
  token, pas de pouvoir par GPU, pas de cloud obligatoire.

Le bon chemin n'est donc pas de "mettre Dify/Flowise/LangGraph dans le
protocole". Le bon chemin est de definir un artefact SBFB portable:

```text
SBFBPromptAgentPack.v1
```

Ce pack decrit les prompts, roles, workflows, outils demandes, schemas de
sortie, jeux d'evaluation, sources RAG, contraintes de modele, licences,
provenance et preuves RRV. Le noeud local decide ensuite comment l'executer:
runner natif SBFB, PydanticAI, LangGraph, Haystack, LlamaIndex, DSPy, CrewAI,
Ollama, llama.cpp, LocalAI, ou autre adapter local.

Principe central:

```text
Le pack demande des capacites.
Le noeud local accorde seulement l'intersection:

pack requests
  intersect node policy
  intersect user consent
  intersect Operator allowlist
```

Une app SBFB, Ideas Hub ou Factory Viewer ne doit jamais installer, signer,
executer ou autoriser un agent directement. Elle propose, affiche, annote et
produit un handoff. L'Operator local reste l'executeur privilegie.

## Ancrage dans le repo actuel

Le repo a deja plusieurs briques qui vont dans ce sens.

- `prompts/agent/` contient deja des prompts portables:
  `base`, `universal`, `handoff`, `preflight`, `phase-review`,
  `phase-auditor`, `audit-gate`, `commit-body`, `app-authoring`.
- `crates/sbfb-factory/src/process.rs` contient deja un registre de prompt
  kinds et une adaptation par provider: `claude`, `codex`, `gpt`, `local`,
  `human`.
- `crates/sbfb-factory/src/provider_router.rs` distingue deja le provider qui
  lit le prompt du runtime qui execute: Claude cloud, Ollama local, reseau
  SBFB.
- `crates/sbfb-factory/src/operator_server.rs` expose deja `/api/prompt/{kind}`,
  `/api/context-pack`, `/api/providers` et bloque les actions sensibles avant
  dispatch.
- `docs/agent/RRV_FACTORY_CONTRACT.md` fixe deja la hierarchie saine:
  process > RRV > Factory > prompt.
- `docs/protocol/SBFB_JSON_V2.md` indique que `SBFB.json` reste un manifeste
  d'application; un artefact source-only/corpus/pack doit avoir son contrat
  explicite.
- `COMMONS.md` et les notes de recherche recentes imposent le cadre commun:
  AGPL pour le protocole, pas de token governance, pas de CLA obligatoire,
  fork/reuse acceptes, curation humaine locale.

Conclusion repo: SBFB a deja un mini-systeme de prompt registry local pour le
process agent. Ce qui manque est le contrat reseau et local d'un pack
prompt/agent installable par noeud.

## Paysage open source

### Orchestration agentique

| Projet | Role utile | Position SBFB |
|---|---|---|
| LangGraph | Graphes d'agents stateful, workflows longs, human-in-loop, checkpoints | Excellent adapter runtime, surtout pour workflows durables |
| PydanticAI | Agents types, schemas, tools valides, outputs structures, evals | Tres bon fit pour contrats forts et manifests verifiables |
| CrewAI | Agents roles/taches en YAML/code, rapide a authorer | Utile pour packs simples, a encadrer fortement |
| Semantic Kernel | Kernel/plugins/functions, multi-langages, enterprise | Bon en contexte .NET/enterprise, pas coeur protocolaire |
| AutoGen | Multi-agent historique, patterns utiles | A lire comme inspiration; ne pas choisir comme socle neuf si maintenance mode |

### RAG, memoire, corpus et recherche

| Projet | Role utile | Position SBFB |
|---|---|---|
| Haystack | Pipelines RAG explicites, composants, retrievers, evals | Tres bon pour workflows auditables, licence Apache |
| LlamaIndex | Connecteurs, ingestion, indexes, query engines, agents | Tres bon pour corpus, Research Packs et RAG avance |
| AnythingLLM | Workspaces RAG/agents local-first, UX mature | Bon atelier local, pas format protocolaire direct |
| Open WebUI | Console chat/RAG/tools self-host | Option locale si licence custom acceptee; pas coeur anti-capture |

### Prompt systems, typage et evals

| Projet | Role utile | Position SBFB |
|---|---|---|
| DSPy | Signatures, modules, optimisation de prompts par metrics/evals | Tres bon "compiler de prompts" si dataset/eval/model sont figes |
| BAML | Prompts types et outputs structures | Bon modele pour contrats et parsing robuste |
| promptfoo | Evals regression, red-team, injection, comparisons modeles | Bon outil de gate local et CI |
| Langfuse / TruLens | Observabilite, traces, evals | Utile si auto-heberge, jamais autorite finale |

### Runtimes par noeud

| Projet | Role utile | Position SBFB |
|---|---|---|
| Ollama | Runtime local simple, API locale, structured output/tools selon modeles | Default local pour un noeud personnel |
| llama.cpp | Runtime bas niveau, GGUF, controle fort, reproductibilite | Base locale robuste pour workers et model hash |
| LocalAI | Gateway locale OpenAI-compatible avec auth/quotas/RBAC selon config | Option pour noeud collectif ou institution |
| Jan | Poste personnel local, API loopback, MCP | Bon outil individuel, pas serveur collectif principal |
| LM Studio | App locale utile pour tester | Hors coeur SBFB car pas un composant open source redistribuable au sens protocole |
| MCP | Protocole de tools entre clients et serveurs | Adapter utile, jamais autorite ni politique de securite |

### Plateformes visuelles

| Projet | Role utile | Position SBFB |
|---|---|---|
| Dify | Produit complet workflows/RAG/agents/API | Bonne reference UX; licence et runtime trop platformes pour coeur SBFB |
| Flowise | Builder visuel de chatflows/agents | Bon outil auteur; exporter/transformer vers spec SBFB |
| PromptFlow | Modele DAG + evals + CI | Inspiration de format, pas dependance long terme |

## Decision d'architecture

Le protocole SBFB ne doit pas standardiser un framework agentique precis. Il
doit standardiser un artefact de commun:

```text
Prompt/Agent Pack = contenu verifiable + politiques + schemas + evals
Runtime Adapter = facon locale de l'executer
Operator = seul composant qui peut accorder des capacites
RRV = composant qui expose preuves et limites
Ideas Hub / Factory = interfaces qui proposent et consomment, sans autorite
```

Le pack est un paquet de fichiers source, pas une promesse magique. Il peut
etre lu, forke, audite, signe, deprecie, repris par une communaute ou installe
localement. Il ne gagne jamais le droit d'agir par le simple fait d'etre publie.

## Format propose: SBFBPromptAgentPack.v1

Layout conceptuel:

```text
SBFB.pack.json
PACK.lock
pack.provenance.json
SIGNATURES.json
LICENSES/
README.md
prompts/
agents/
workflow/
schemas/
tools.allowlist.json
models.requirements.json
rag/manifest.json
evals/
rrv.proofcards.jsonl
factory.recipe.json
ideas.card.json
install.plan.json
```

### SBFB.pack.json

Exemple minimal:

```json
{
  "schema_version": 1,
  "artifact_kind": "prompt_agent_pack",
  "pack_id": "org.commons.atelier-builder",
  "name": "atelier-commons-builder",
  "version": "0.1.0",
  "display_name": "Atelier des communs - Builder",
  "description": "Pack de maturation d'idees en communs verifiables",
  "license": "AGPL-3.0-or-later",
  "publisher": {
    "node_id": "ed25519:...",
    "maintainer": "..."
  },
  "source": {
    "repo_url": "https://example.org/commons/atelier-builder",
    "commit_sha": "...",
    "path": "packs/atelier-builder"
  },
  "commons_policy": {
    "no_global_ranking": true,
    "no_token_gate": true,
    "no_gpu_power_gate": true,
    "forkable": true,
    "data_local_first": true
  },
  "capabilities_requested": [
    "repo_read",
    "rrv_claim_labeling",
    "brief_de_commun_draft",
    "factory_recipe_draft"
  ],
  "model_requirements": {
    "structured_output": true,
    "citations": true,
    "tool_use": true,
    "min_context_tokens": 32000,
    "allowed_boundaries": ["local", "network", "cloud_optional"]
  },
  "tools_policy_ref": "tools.allowlist.json",
  "rrv_proofcards_ref": "rrv.proofcards.jsonl",
  "evals_ref": "evals/manifest.json"
}
```

### PACK.lock

Role: rendre le pack content-addressable et detecter toute derive.

Contenu:

- chemin canonique;
- taille;
- hash BLAKE3;
- type attendu;
- licence si differente;
- generated/non-generated;
- reference de source si import externe.

### pack.provenance.json

Role: relier le pack a une origine verifiable.

Contenu:

- repo URL;
- commit;
- tree hash;
- build/export command;
- timestamp;
- node id;
- maintainer key;
- artifact hash;
- signature Ed25519.

### tools.allowlist.json

Role: declarer ce que le pack demande, pas ce qu'il obtient.

Classes recommandees:

```text
read_only
write_draft
write_workspace
destructive
network
publish
secret_access
remote_compute
commit
push
```

Par defaut, seuls `read_only` et `write_draft` peuvent etre proposes sans
danger majeur. Tout le reste demande confirmation humaine explicite et journal
local.

### models.requirements.json

Role: decrire les besoins par capacite, pas par marque SaaS.

Exemples:

- `structured_output`;
- `json_schema`;
- `tool_use`;
- `citations`;
- `vision`;
- `long_context`;
- `local_only`;
- `no_network`;
- `min_context_tokens`;
- `tested_models`;
- `not_evidenced_models`.

Une validation de pack vaut seulement pour:

```text
pack version + provider + model + date/version + toolset + dataset/evals
```

Si le modele change, le statut redevient "non verifie" jusqu'a nouveau run
d'evals.

## RRV Proof Cards pour packs

RRV ne doit jamais dire "cet agent est bon". RRV peut dire:

- le pack existe a tel hash;
- les fichiers annonces sont presents;
- les signatures sont verifiables;
- la licence est declaree;
- tel outil est ou n'est pas demande;
- tel eval a ete execute avec tel modele;
- telle limite reste non prouvee.

Exemple:

```json
{
  "subject": "pack:org.commons.atelier-builder@0.1.0",
  "claim": "Le pack ne demande aucun outil d'ecriture workspace",
  "label": "Verifie",
  "evidence": [
    {
      "kind": "hash",
      "ref": "PACK.lock#tools.allowlist.json"
    }
  ],
  "limits": [
    "Ne prouve pas la qualite des prompts",
    "Ne prouve pas la robustesse contre injection indirecte"
  ],
  "formula_version": "prompt-agent-pack-proof-v1"
}
```

Labels obligatoires:

```text
Lu
Deduit
Verifie
Non verifie
```

Ne pas fusionner preuve cryptographique, hypothese LLM, avis curator et resultat
d'eval dans un score global.

## Lifecycle

```text
draft
preflight
signed
published
indexed
curated
installed
activated
forked
deprecated_or_revoked
```

Details:

- `draft`: pack local, non publie, aucune autorite.
- `preflight`: schema, lockfile, secrets, licences, tools, model requirements.
- `signed`: manifest + merkle root signes.
- `published`: blob/feed avec hash permanent.
- `indexed`: RRV lit, indexe et expose des proof cards.
- `curated`: listes locales de curators, sans blocage global.
- `installed`: noeud local installe en lecture seule, desactive par defaut.
- `activated`: l'utilisateur accorde des capacites limitees.
- `forked`: provenance de lignee explicite.
- `deprecated_or_revoked`: nouvelle attestation; pas de suppression centrale.

Pas d'auto-update silencieux. Chaque version est un nouveau hash.

## Utilisation par Ideas Hub

Ideas Hub doit etre l'Atelier des communs, pas un store d'agents.

Fonctions autorisees:

- explorer les packs par recherche locale;
- afficher provenance, licence, preuves RRV, limites et forks;
- aider une personne ou un collectif a choisir un pack;
- produire une intention signee ou un handoff vers l'Operator local;
- annoter localement les resultats;
- proposer un fork ou une reprise.

Fonctions interdites:

- installer un pack directement;
- donner un secret a un pack;
- lancer un outil a la place de l'utilisateur;
- faire un ranking global;
- cacher la licence ou les limites;
- confondre curator vouch et preuve technique.

Sorties typiques:

```text
GraineDeBesoin
BriefDeCommun
ResearchPack
PromptAgentPackInstallIntent
DecisionLocale
ForkProposal
```

## Utilisation par Factory

Factory peut consommer un pack comme assistance a la creation, jamais comme
autorite de verdict.

Commandes conceptuelles:

```text
sbfb-factory pack inspect <hash-or-id>
sbfb-factory pack preflight <path>
sbfb-factory pack install --dry-run <hash-or-id>
sbfb-factory pack install <hash-or-id>
sbfb-factory pack activate <id>@<version> --grant read_only --grant write_draft
sbfb-factory pack run <id>@<version> --task draft-brief
sbfb-factory pack fork <id>@<version> --reason ...
```

Factory doit garder:

- context-pack hashe;
- `chat_history_authoritative=false`;
- actions sensibles gatees;
- prompts et sorties non autoritaires;
- sortie en draft/diff lisible;
- absence de `PASS` produit par LLM.

Premier pack interne possible:

```text
org.sbfb.factory.app-authoring@0.1.0
```

Il emballerait les prompts existants `prompts/agent/app-authoring.md`,
`base.md`, `universal.md`, les connaissances authoring, les schemas de sortie
et les evals correspondantes. Ce serait une conversion locale d'abord, pas une
publication reseau immediate.

## Utilisation par RRV

RRV doit traiter un Prompt/Agent Pack comme une source verifiable et non comme
un oracle.

RRV peut:

- indexer le pack;
- exposer hash, provenance, licence, model requirements;
- lier des proof cards a des claims precis;
- comparer des versions;
- afficher des limites;
- garder les labels separes.

RRV ne doit pas:

- lancer l'agent avec des privileges;
- signer a la place du mainteneur;
- attribuer un score global de "meilleur agent";
- transformer une sortie LLM en preuve;
- accepter une eval sans dataset, modele, date et toolset.

## Modele par noeud

Pile recommandee anti-capture:

```text
Base personnelle:
  Ollama + llama.cpp

Noeud collectif:
  LocalAI derriere auth/quotas/RBAC + llama.cpp/Ollama

Atelier RAG local:
  AnythingLLM ou Haystack/LlamaIndex

Tools:
  MCP via Operator, jamais direct depuis l'app sandboxee

Console optionnelle:
  Open WebUI si licence acceptee, Jan pour poste individuel
```

Contraintes:

- bind strict sur `127.0.0.1` par defaut;
- cloud opt-in seulement;
- pas de vector DB cloud par defaut;
- pas de web search externe par defaut;
- secrets hors prompts/RAG/logs;
- modele documente avec licence, source, hash, quantization;
- poids de modeles: ne pas dire "open source" si ce sont seulement des
  "open weights".

## Securite

Un pack prompt/agent distribue en P2P est une dependance non fiable plus une
demande d'autorite. Les menaces principales:

- prompt injection directe;
- prompt injection indirecte via RAG, docs, pages web, diffs ou issues;
- exfiltration par tool call, URL, logs, output ou prompt cache;
- supply-chain de prompts;
- tools trop larges;
- secret leakage;
- model drift;
- evals trompeuses;
- licence incompatible;
- capture par marketplace ou ranking global.

Mitigations minimales:

- toutes les donnees P2P/RAG/curator/app sont `untrusted data`;
- les donnees non fiables ne modifient jamais system prompt, policies, tools ou
  destinations reseau;
- tools par broker local, jamais par bearer daemon partage;
- token local par agent/run/tool/duree/scope si une capacite est accordee;
- dry-run et diff visible avant ecriture;
- confirmation humaine pour shell, commit, push, publish, network, secrets;
- egress ferme par defaut;
- lockfile BLAKE3;
- signature Ed25519;
- licences SPDX;
- journaux locaux;
- evals par pack/model/toolset;
- pas d'auto-update.

## Anti-capture et communs

Regles non negociables pour rester aligne avec SBFB:

- pas de ranking global d'agents;
- pas de token gate;
- pas de stake/vote economique;
- pas de kudos gate qui devient pouvoir;
- pas de rente GPU;
- pas de cloud obligatoire;
- pas de dependance a une plateforme SaaS;
- forks visibles et encourages;
- curation locale et reversible;
- stewardship documente;
- usage humanitaire et entraide avant performance commerciale.

La question a poser a chaque pack:

```text
Quelle autonomie collective ce pack augmente-t-il,
quel dommage evite-t-il,
et quelles dependances nouvelles introduit-il ?
```

## Stack recommandee pour un premier prototype

Prototype minimal:

```text
Manifest/schema:
  JSON Schema + Rust structs dans un crate separe

Prompt/agent type:
  PydanticAI comme reference d'adapter

RAG:
  Haystack ou LlamaIndex, avec manifests de corpus hashes

Prompt optimisation:
  DSPy pour compiler/optimiser, mais artefacts figes dans le pack

Runtime local:
  Ollama + llama.cpp

Gateway collective:
  LocalAI optionnel

Tools:
  MCP via Operator

Evals:
  promptfoo ou Pydantic Evals, avec rapports signes
```

LangGraph devient prioritaire si le besoin principal est un workflow long avec
etat durable, interruptions humaines et reprises.

## Plan de recherche suivant

R0. Note de recherche actuelle.

R1. Rediger `docs/protocol/SBFB_PROMPT_AGENT_PACK_V1.md` en spec non codee.

R2. Creer un schema JSON experimental hors wire stable.

R3. Convertir les prompts repo existants en pack interne local:
`org.sbfb.factory.app-authoring@0.1.0`.

R4. Ajouter une commande dry-run:
`sbfb-factory pack preflight <path>`.

R5. Ajouter des proof cards RRV pour pack local:
hash, licence, tools demandes, model requirements, eval status.

R6. Ajouter une surface Ideas Hub qui affiche les packs mais ne les execute pas.

R7. Ajouter une activation Operator locale avec grants limites.

R8. Tester un adapter PydanticAI ou LangGraph sans l'imposer au protocole.

## Non-objectifs

- Pas de marketplace centrale.
- Pas de "best agent" global.
- Pas d'agent qui tourne dans l'iframe app.
- Pas de secrets dans les packs.
- Pas d'installation automatique depuis un clic app.
- Pas de cloud par defaut.
- Pas de provider unique.
- Pas de verdict produit par LLM.
- Pas de `SBFB.json` app v2 detourne en manifeste agent.

## Sources externes consultees

- LangGraph: https://github.com/langchain-ai/langgraph et
  https://docs.langchain.com/oss/python/langgraph/workflows-agents
- LlamaIndex: https://github.com/run-llama/llama_index et
  https://developers.llamaindex.ai/
- PydanticAI: https://github.com/pydantic/pydantic-ai et
  https://ai.pydantic.dev/
- Haystack: https://github.com/deepset-ai/haystack et
  https://docs.haystack.deepset.ai/
- DSPy: https://github.com/stanfordnlp/dspy et https://dspy.ai/
- CrewAI: https://github.com/crewAIInc/crewAI et https://docs.crewai.com/
- Ollama: https://github.com/ollama/ollama et https://docs.ollama.com/
- llama.cpp: https://github.com/ggml-org/llama.cpp
- LocalAI: https://github.com/mudler/LocalAI et https://localai.io/
- AnythingLLM: https://github.com/Mintplex-Labs/anything-llm
- Open WebUI: https://github.com/open-webui/open-webui
- Model Context Protocol: https://modelcontextprotocol.io/ et
  https://github.com/modelcontextprotocol
- promptfoo: https://github.com/promptfoo/promptfoo et
  https://www.promptfoo.dev/
- BAML: https://github.com/BoundaryML/baml et
  https://docs.boundaryml.com/
- OWASP Top 10 for LLM Applications:
  https://owasp.org/www-project-top-10-for-large-language-model-applications/
- OWASP Agentic AI Security:
  https://owasp.org/www-project-agentic-ai-security/
- Indirect prompt injection, Greshake et al.:
  https://arxiv.org/abs/2302.12173
- AgentDojo:
  https://arxiv.org/abs/2406.13352
