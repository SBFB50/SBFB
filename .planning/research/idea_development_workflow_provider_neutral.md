# Idea Development Workflow Provider-Neutral

Date: 2026-06-28
Status: NOTE DE RECHERCHE. Hors sprint. Aucun code, aucun wire, aucun commit
de phase, aucun verdict process. Ce document decrit le systeme de workflow a
creer pour maturer les idees, avant l'application Ideas Hub faite plus tard
avec Factory.

Update cadrage 2026-06-28: ce document decrit la mecanique
provider-neutral. Le cadrage produit correct est
`idea_commons_workflow_anti_capture.md`: Atelier des communs, communs
verifiables, anti-capture, humanitaire/culture libre, pas funnel SaaS.
Les termes `Product`, `PO`, `funnel`, `value` doivent etre lus comme des
raccourcis historiques et remplaces dans les futurs artefacts par
`commun`, `steward d'usage`, `chemin de maturation`, `utilite collective`.

## 1. Intention

Le besoin n'est pas seulement une app "Ideas Hub". Le besoin est un workflow
portable, comparable au role de `CLAUDE.md`, mais centre sur le cycle de vie
d'une idee de commun/protocole/service d'entraide:

1. capturer une idee brute;
2. la clarifier en probleme, utilisateur, valeur, risques;
3. chercher l'etat de l'art et les contraintes repo;
4. produire plusieurs options;
5. faire une critique sceptique multi-agent/multi-modele;
6. transformer l'option retenue en dossier de recherche;
7. seulement ensuite, si l'humain tranche, produire un kickoff de sprint ou un
   brief Factory app.

Le workflow doit fonctionner avec Claude, OpenAI/GPT, Gemini, Ollama/local et
SBFB Network. Les modeles changent; les artefacts, schemas, gates et preuves
restent stables.

## 2. Relation avec la note Ideas Hub

La note `.planning/research/idea_hub_factory_ultracode_redesign.md` traite le
substrat commun futur: claim signe, convergence feed/storage, gouvernance,
handoff sandbox -> Operator par donnee, jamais par controle.

Le present document est la couche au-dessus:

- Ideas Hub app = capture, discussion, proposition, prise en charge,
  registre local de maturation visible;
- Atelier des communs = cadre social: besoin situe, commun verifiable,
  anti-capture, porteur responsable, curation locale;
- Workflow Provider-Neutral = machine de maturation et de decision;
- Operator = espace privilegie local qui execute le workflow, lit le repo, cree
  des artefacts, et lance de vraies sessions agent;
- Factory = outil qui transformera plus tard une idee mature en app SBFB, sans
  donner a l'app sandboxee le pouvoir de signer, lancer ou commit.

Regle cardinale: le hub PROPOSE; l'Operator DEVELOPPE; l'humain ARBITRE; les
gates VERIFIENT. Aucun LLM ne fabrique une autorite finale.

## 3. Entites

### 3.1 Idea

Objet brut, non autoritaire. Une idee peut venir du chat, de l'app Ideas Hub, du
repo, d'un forum, d'un bug ou d'une intuition mainteneur.

Champs minimum:

```json
{
  "idea_id": "blake3(canonical idea seed)",
  "title": "...",
  "one_liner": "...",
  "source": "chat|ideas-hub|repo|external|operator",
  "tags": ["factory", "workflow", "llm"],
  "created_by": "node_id or local operator",
  "created_at": "timestamp outside identity preimage"
}
```

### 3.2 Development Brief

Premier artefact utile. Il force l'idee a devenir testable sans la transformer
en sprint.

Sections:

- Problem: quel probleme reel ou dommage evite;
- Personnes concernees: pour qui, dans quel contexte local;
- Why SBFB: pourquoi le protocole est le bon terrain;
- Commons purpose: quel commun est augmente, protege ou rendu accessible;
- Existing repo anchors: fichiers/commits/docs concernes;
- Non-goals: ce qu'on ne promet pas;
- Risk class: protocol, security, process, UX, compute, governance;
- Anti-capture: donnees, dependance fournisseur, rente, leaderboard,
  gouvernance par hardware ou kudos;
- Evidence needed: recherches a faire avant decision.

### 3.3 Research Pack

Document repo-visible sous `.planning/research/`. Il contient les options,
sources, objections, decision candidate, et les questions PO ouvertes.

Il n'est pas un kickoff. Il ne contient pas de verdict `PASS`. Il peut contenir
`Recommended`, `Concern`, `Blocked`, `Provisional`, mais jamais un gate final.

### 3.4 Decision Pack

Pont vers le sprint ou Factory. Produit seulement quand l'humain veut agir.

Sorties possibles:

- `sprint_candidate`: transformable en kickoff Cas C;
- `factory_app_candidate`: brief d'app a construire avec Factory;
- `research_only`: garder en backlog;
- `reject`: rejet documente;
- `needs_external_evidence`: blocage par source, materiel, modele ou test.

## 4. Pipeline

### Stage 0 - Intake

But: capturer sans juger.

Sortie: `Idea`.

Regles:

- ne pas demander au LLM de trancher;
- ne pas deduire une roadmap;
- ne pas creer de sprint;
- accepter les idees faibles, mais les marquer faibles.

### Stage 1 - Clarification

But: transformer l'idee en brief.

Questions obligatoires:

1. Quelle valeur utilisateur concrete?
   Traduction future: quelle utilite collective, quel dommage evite, quelle
   autonomie gagnee?
2. Quel lien avec SBFB/Factory/RRV/compute/gouvernance?
3. Quelle preuve rendrait l'idee fausse?
4. Quelle surface sensible touchee?
5. Quelle version minimale utile?

Sortie: `Development Brief`.

### Stage 2 - Repo Grounding

But: verifier que l'idee ne flotte pas dans le chat.

Actions:

- `rg` sur les mots-cles;
- lire fichiers source/protocole/process proches;
- detecter decisions gelees;
- detecter gaps deja documentes;
- distinguer code-backed, doc-only, future work, stale.

Sortie: liste `file:line` + classification.

### Stage 3 - External Research

But: exploiter les modeles cloud quand ils apportent vraiment quelque chose:
web actuel, docs fournisseur, papiers, OSS, API recentes.

Capability requise, pas fournisseur fixe:

- `web_research` pour actualite, API cloud, OSS vivant;
- `source_citations` pour preuves;
- `long_context` pour gros docs;
- `structured_output` pour tableaux d'options;
- `tool_use` pour tests de schema ou extraction;
- `code_execution` seulement pour calculs/prototypes non destructifs;
- `computer_use` uniquement en sandbox et jamais pour repo writes.

Sortie: `Research Notes` avec sources et impact.

### Stage 4 - Option Generation

But: produire 2-4 options reelles, pas une seule solution maquillee.

Format:

```text
Option A - Conservative
Option B - Protocol-native
Option C - Commons / solidarity / local-governance first
Option D - Reject / do nothing
```

Chaque option doit inclure:

- ce qu'elle reutilise;
- ce qu'elle ajoute;
- ce qu'elle casse ou risque de casser;
- testabilite;
- cout de maintenance;
- compatibilite avec closed pilot.

### Stage 5 - Skeptical Review

But: forcer la contradiction avant le sprint.

Roles:

- Steward d'usage: besoin situe, utilite collective, non-extraction,
  appropriation volontaire;
- Protocol Reviewer: wire, convergence, P2P, invariants;
- Security Reviewer: trust boundary, signing, sandbox, PII;
- Process Reviewer: gates, atomicite, evidence;
- Maintainer Reviewer: bus-factor, complexite, dette;
- Cloud LLM Reviewer: ce qui est possible avec les outils cloud actuels.

Chaque role rend:

```text
VERDICT: ACCEPT | NEEDS-CORRECTION | REJECT | BLOCKED
Top finding:
Required correction:
Evidence:
```

### Stage 6 - Synthesis

But: integrer les corrections, pas les empiler.

Sortie: `Research Pack` final:

- Executive summary;
- Recommended option;
- Corrections integrated;
- Open PO questions;
- Future sprint mapping;
- Hard non-goals;
- Provider/capability assumptions.

### Stage 7 - Human Arbitration

But: l'humain decide.

Actions possibles:

- `promote_to_kickoff`;
- `promote_to_factory_brief`;
- `keep_research`;
- `reject`;
- `ask_more_research`.

Le systeme ne doit jamais auto-promote une idee en sprint.

## 5. Provider-Neutral Capability Model

Ne pas coder contre `claude`, `gpt`, `gemini`, `ollama` comme concepts
produit. Coder contre des capacites.

```json
{
  "provider": "openai|anthropic|gemini|ollama|network",
  "model": "provider-specific-id",
  "capabilities": {
    "long_context": true,
    "structured_output": true,
    "tool_use": true,
    "web_research": true,
    "file_search": true,
    "code_execution": false,
    "computer_use": false,
    "prompt_cache": true,
    "thinking_budget": true,
    "citations": true,
    "mcp": false,
    "local_only": false
  },
  "limits": {
    "max_input_tokens": null,
    "max_output_tokens": null,
    "cost_class": "free|low|medium|high",
    "data_boundary": "local|cloud|network"
  }
}
```

Selection policy:

- local/Ollama pour ideation privee, reformulation, extraction simple;
- cloud cheap/fast pour triage et summaries;
- cloud frontier pour critique profonde, architecture, security review;
- cloud avec web/search pour faits actuels;
- cloud avec structured output pour schemas et matrices;
- network provider pour tester le dogfood compute, jamais comme autorite;
- humain pour arbitrage et toute action irreversible.

## 6. Cloud LLM Feature Map

Sources consultees le 2026-06-28:

- OpenAI docs: Responses API, built-in tools, structured outputs.
  <https://developers.openai.com/api/docs/guides/deployment-checklist>
  <https://developers.openai.com/api/docs/guides/migrate-to-responses>
  <https://developers.openai.com/api/docs/guides/responses-vs-chat-completions>
- Anthropic docs: Messages API, tool use, prompt caching, extended thinking,
  MCP servers, model capabilities endpoint.
  <https://platform.claude.com/docs/en/api/python/beta>
  <https://platform.claude.com/docs/en/api/python/beta/messages>
- Gemini docs: function calling, structured output with tools, Google Search,
  URL context, code execution, file search.
  <https://ai.google.dev/gemini-api/docs/get-started>
  <https://ai.google.dev/gemini-api/docs/structured-output>
  <https://ai.google.dev/gemini-api/docs/file-search>

Portable conclusions:

- `structured_output` is a first-class requirement. Every workflow stage should
  have a JSON schema, then render Markdown from that schema.
- `tool_use` is common enough to be part of the core abstraction, but tools must
  be host-executed and allowlisted.
- `web_research` is provider-specific but strategically important for G9-style
  factual research.
- `prompt_cache`/context caching should be an optimization layer, never a
  correctness assumption.
- `thinking`/reasoning controls are provider-specific. Store only the public
  result, not hidden reasoning, as the artifact.
- `computer_use` is dangerous for this repo. It belongs in an isolated research
  sandbox, not in the privileged Operator path, unless a future security design
  proves otherwise.
- `MCP` can be useful as a tool transport, but SBFB should still keep its own
  capability registry and gate model. MCP is an adapter, not the authority.

## 7. Workflow Prompts

The system needs a new prompt family, separate from sprint prompts.

Proposed files:

```text
prompts/idea/base.md
prompts/idea/intake.md
prompts/idea/clarify.md
prompts/idea/repo-grounding.md
prompts/idea/external-research.md
prompts/idea/options.md
prompts/idea/skeptical-review.md
prompts/idea/synthesis.md
prompts/idea/decision-pack.md
```

These prompts should obey the same truth stack:

```text
repo files > .planning/research > commits > prompts > chat
```

But they must not invoke sprint gates unless the human promotes the idea.

## 8. Artifact Schemas

Minimum schemas to add later:

```text
Idea
DevelopmentBrief
RepoGroundingReport
ExternalResearchReport
OptionSet
SkepticalReview
ResearchPack
DecisionPack
```

These schemas should live in Factory/process code only after design review.
Before that, Markdown templates are enough.

Important: `ResearchPack` is not a verdict artifact. It should never contain
`## Verdict: PASS`.

## 9. Operator UX

The Operator should expose intentions, not commands:

- "Developper une idee de commun"
- "Ancrer dans le repo"
- "Chercher l'etat de l'art"
- "Comparer les options"
- "Faire critiquer par plusieurs modeles"
- "Transformer en brief Factory"
- "Preparer un kickoff"

It should show:

- current stage;
- inputs used;
- provider/capabilities selected;
- cost/time estimate;
- generated artifact path;
- unresolved stewardship / usage questions;
- clear boundary: "research, not sprint".

It should not show:

- raw `agentctl` commands as primary CTA;
- model jargon as the main decision surface;
- fake PASS/GO labels;
- auto-commit or auto-push affordances.

## 10. Multi-Model Orchestration Pattern

Recommended pattern:

1. One driver model creates the structured draft.
2. Two or more reviewer models critique from distinct roles.
3. A synthesis model integrates corrections.
4. A deterministic validator checks schema, required sections, and forbidden
   claims.
5. Human arbitrates.

Never use "majority vote" as truth. Use disagreement as signal:

- if models disagree on facts, re-check sources;
- if models disagree on architecture, record options;
- if models disagree on risk, pick the stricter risk until disproved;
- if local and cloud disagree, trust repo evidence over both.

## 11. Boundary With Ideas Hub App

The future Ideas Hub app should only:

- publish ideas;
- display discussion and local maturation state;
- let a node claim/champion with host confirmation;
- show public research links and resulting apps;
- offer a passive hint to open Operator.

It should never:

- launch Operator;
- pass bearer tokens;
- commit files;
- execute agents;
- decide a sprint;
- calculate final process verdicts.

The handoff key is data:

```text
idea_id -> ResearchPack -> DecisionPack -> kickoff/frontmatter or Factory brief
```

No control-flow handoff from sandbox to privileged local process.

## 12. Sprint Candidate Later

If promoted, this should probably be its own sprint before the Ideas Hub app
implementation:

Phase A: write `docs/idea-workflow/` doctrine and templates.
Phase B: add Markdown templates + prompt family.
Phase C: add schema validator and `sbfb-factory idea` commands.
Phase D: Operator read-only UI for idea workflow stages.
Phase E: provider capability registry and selection policy.
Phase F: cloud adapters behind capability flags.
Phase G: multi-model skeptical review orchestration.
Phase H: research-pack export and promote-to-kickoff/factory-brief.
Phase I: docs-contract closure + llms.txt.

Keep the future Ideas Hub app separate. The app can consume this workflow later,
but the workflow must exist first so the app does not invent process authority.

## 13. Non-Goals

- No new feed op in this workflow document.
- No Ideas Hub implementation.
- No bridge method.
- No cloud API key storage design.
- No automatic commit/push.
- No replacement for `docs/agent/PROCESS.md`.
- No provider-specific lock-in.
- No ranking global of ideas.

## 14. Open Questions

1. Should this become `docs/idea-workflow/` first, or stay in research until the
   Ideas Hub convergence blocker is resolved?
2. Should the first implementation target Operator only, with no daemon/app
   surface?
3. Which providers are allowed in closed pilot: Claude CLI, OpenAI API, Gemini
   API, Ollama, SBFB Network?
4. Where do cloud API keys live: environment only, OS keychain, or Operator
   config encrypted at rest?
5. Should cloud web search be allowed to see private repo snippets, or only
   sanitized questions plus public docs?
6. What is the minimum JSON schema set needed before UI work?
7. How do we measure success without SaaS drift: clarified social needs,
   ResearchPacks with RRV labels, risks of extraction avoided, commons
   published with source/provenance/license, maintenance, forks of rescue,
   rejected ideas with clear rationale?
