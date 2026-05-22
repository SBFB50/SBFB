# RRV LLM runtime et frontiere app/protocole

**Date:** 2026-05-22
**Status:** research canon candidate for S69 Phase E / S70 kickoff
**Question:** RRV doit-il tourner avec un LLM local ou centralise, et doit-il etre
une app installee sur le protocole ou integree dedans ?

## 1. Verdict court

RRV doit supporter les deux types de modeles:

- local first: Ollama, llama.cpp, Candle ou endpoint compatible local;
- centralise opt-in: Claude, OpenAI/Codex-like API, ou autre fournisseur distant;
- sans LLM: mode deterministe recherche/citations/preuves, obligatoire pour les
  tests et les environnements sensibles.

Mais le LLM ne doit jamais etre une source de preuve. Il compose une reponse a
partir d'un paquet d'evidence deja construit par le protocole.

RRV ne doit pas etre seulement "une app" ni seulement "integre au protocole".
La bonne architecture est en couches:

```text
Protocol / wire
  -> formats signes, manifests, SearchManifest plus tard

Daemon / RRV Core
  -> index local, search, proof facts, ProofCard, privacy gate, provider router

Shell / RRV system surface
  -> route /rrv preinstallee pour bootstrap, chat, scopes @, actions

Installable app / sbfb-search
  -> meme experience comme app SBFB, dogfood du bridge, aucun privilege cache
```

Le point important: le "cerveau verifiable" est dans le service local, pas dans
Claude, Codex, OpenAI, Ollama, ni dans une app front qui peut mentir.

## 1.1 S70 routing amendment

Before choosing an RRV product route, S70 must first complete the portable
agent/process contract:

```text
S70 = Process Portable Complete + Gate 1 dogfood
```

The LLM provider router described here remains a consumer of that portable
evidence contract. It is not the next sprint authority, and it must not bypass
`.planning/active/`, `agentctl`, review artifacts, or proof labels.

## 2. Ce que le repo prouve deja

Le repo a deja les ingredients, mais pas encore le produit RRV complet:

- le process agent est vendor-neutral: Claude, GPT, Codex et LLM local doivent
  consommer les memes fichiers et commandes (`docs/agent/PROCESS.md`,
  `prompts/agent/universal.md`, `scripts/agent/agentctl.py`);
- les workers ont deja une abstraction LLM locale avec Ollama et feature gate
  llama.cpp (`crates/nexus-worker-core/src/llm/mod.rs`,
  `factory.rs`, `ollama.rs`);
- `task_submit` porte deja `prompt`, `model`, reponse et `model_digest`, mais
  c'est une primitive de compute/audit, pas le chemin normal d'un chat RRV;
- le daemon expose deja des briques search/proof, mais elles sont trop fines
  pour un chat preuve-first: il manque file/line/hash, politiques d'egress et
  format EvidenceBundle;
- le bridge existe pour les apps, mais RRV ne doit pas contourner la sandbox des
  apps installees.

Donc le developpement serieux n'est pas "brancher Claude dans le front". C'est:

1. construire le paquet d'evidence;
2. bloquer les fuites;
3. router vers un provider choisi;
4. verifier la sortie;
5. rendre les labels de preuve sans les fusionner.

## 3. Modes LLM a supporter

### Mode `none` / `search_only`

RRV fonctionne sans modele:

- recherche locale;
- resultats avec labels;
- citations file/line/hash ou feed/provenance;
- ProofCard;
- actions `open`, `cite`, `inspect`, `verify`.

Ce mode est obligatoire pour:

- tests deterministes;
- machines sans modele;
- audits de non-regression;
- usage sensible ou hors-ligne.

### Mode `local`

Usage:

- `@dev`;
- repos prives;
- process docs;
- base locale si un contrat d'acces existe;
- questions ou le prompt ne doit pas sortir de la machine.

Providers possibles:

- Ollama HTTP local;
- llama.cpp local;
- Candle/local Rust backend;
- endpoint local compatible OpenAI si configure explicitement.

Contraintes:

- pas d'appel reseau externe;
- log `provider_id`, `model_id`, version/digest si disponible;
- le provider recoit seulement un `PromptBundle` issu d'un `EvidenceBundle`;
- aucun fichier brut complet si des citations lineaires suffisent.

### Mode `central`

Usage:

- synthese longue;
- comparaison de gros corpus publics;
- generation UX/dev apres redaction;
- assistance externe consentie.

Contraintes:

- opt-in explicite par question, projet ou policy;
- affichage du fournisseur, modele, politique de retention connue et egress;
- `store=false` ou equivalent quand le provider le permet;
- redaction locale avant sortie;
- logs d'audit sans prompt prive complet par defaut;
- aucun label de preuve ajoute par le modele.

Claude, OpenAI/Codex-like API ou un autre fournisseur distant sont donc des
adapters, pas des dependances du protocole.

### Mode `agent_process`

Ce mode couvre les outils type Codex CLI, Claude Code ou agent de dev lance en
processus.

Il ne doit pas etre le runtime public par defaut de RRV. Il sert a:

- faire du developpement assiste;
- piloter le process sprint;
- generer des patchs;
- executer des audits commandes;
- produire des documents de planification.

Il faut le traiter comme une action privilegiee avec workspace, commandes,
scope, diff et tests, pas comme une simple completion de chat.

### Mode `network_task`

Ce mode utilise `task_submit` ou une primitive future de compute reseau.

Il ne doit pas etre appele pour chaque question RRV. Il sert seulement a:

- `Ask network`;
- audit quorum;
- verification build/test;
- batch distribue;
- comparaison active entre workers.

Le resultat reseau doit revenir avec worker id, signature, model_digest si
applicable, logs et proof tier. Il ne remplace pas la preuve locale.

## 4. Frontiere app vs protocole

### Ce qui appartient au protocole / wire

Le protocole porte les formats stables:

- feed events signes;
- provenance hashes;
- manifest app;
- SearchManifest plus tard;
- labels de preuve et leurs transitions;
- contrats de publication/verifiabilite.

Le protocole ne porte pas:

- la conversation UI;
- le prompt exact d'un provider;
- la personnalite d'un assistant;
- le ranking subjectif non explicable;
- le choix Claude vs OpenAI vs Ollama.

### Ce qui appartient au daemon / RRV Core

Le daemon local doit posseder les faits verifiables:

- index `@protocole`;
- index `@dev LocalOnly`;
- ProofCard et proof facts;
- EvidenceBundle;
- Privacy/Egress Gate;
- Provider Router;
- journal d'audit minimal;
- endpoints neutres pour le shell et les apps.

C'est ici que se fait la difference entre:

- `lu` depuis un fichier ou event;
- `deduit` par une regle locale;
- `verifie` par preuve/provenance;
- `non verifie` par absence de preuve.

### Ce qui appartient a la surface shell `/rrv`

La route shell `/rrv` doit exister tot pour resoudre le bootstrap:

- l'utilisateur peut questionner le protocole avant d'installer une app;
- les scopes `@protocole`, `@dev`, `@web` peuvent etre expliques et limites;
- les features critiques peuvent etre testees sans packaging app complet.

Mais `/rrv` ne doit pas avoir une API secrete que `sbfb-search` ne pourrait pas
utiliser. Les privileges doivent etre dans le daemon et exposes par contrats
neutres.

### Ce qui appartient a l'app installee `sbfb-search`

`sbfb-search` est le dogfood applicatif:

- meme UX de recherche/question;
- meme bridge;
- meme endpoints publics;
- meme ProofCard;
- aucune lecture directe du filesystem;
- pas d'acces DB hors contrat explicite.

Si `/rrv` marche mais `sbfb-search` ne peut pas faire la meme operation via
contrat public, alors l'architecture est trop integree et pas assez
protocolaire.

## 5. Pipeline cible

```text
Question utilisateur
  -> Scope Parser
       @current / @protocole / @dev / @web / @private:<group>
  -> Retrieval local-first
       daemon search, source index, feed, provenance, ProofCard
  -> EvidenceBundle
       citations, hashes, proof facts, trust labels, source policy
  -> Privacy/Egress Gate
       LocalOnly default, redaction, consent, provider policy
  -> Provider Router
       none | local | central | agent_process | network_task
  -> Answer Composer
       structure la reponse, ne cree pas de preuve
  -> Output Verifier
       citations obligatoires, labels preserves, no invented verification
  -> Renderer / Actions
       open, cite, inspect, verify, ask network, publish, fork
```

## 6. Contrats internes a creer

### `EvidenceBundle`

Le paquet que RRV peut envoyer a un modele.

```text
EvidenceBundle {
  question_id
  scope
  privacy_class: public | local_only | private_group | secret_blocked
  sources[]
    source_id
    source_kind: file | feed_event | manifest | proof_card | web | db_contract
    path_or_ref
    line_start?
    line_end?
    commit_or_content_hash?
    provenance_hash?
    label
    excerpt
  proof_facts[]
  missing_evidence[]
  policy
    may_leave_device: bool
    redaction_required: bool
    provider_allowlist[]
}
```

Regle: le modele ne voit pas "tout le repo". Il voit un paquet borne, cite,
hashable et loggable.

### `PromptBundle`

Ce que le router transmet au provider.

```text
PromptBundle {
  system_contract
  user_question
  evidence_bundle
  output_schema
  forbidden_claims[]
  required_sections: read | inferred | verified | unverified
  max_tokens
}
```

### `LlmProvider`

```text
LlmProvider {
  id
  kind: none | local | central | agent_process | network_task
  egress_required: bool
  supports_streaming: bool
  supports_json_schema: bool
  supports_tool_calls: bool
  model_id
  model_version_or_digest?
  data_retention_policy_ref?
  max_context
  generate(PromptBundle) -> ProviderOutput
}
```

### `ProviderOutput`

```text
ProviderOutput {
  provider_id
  model_id
  text_or_json
  cited_source_ids[]
  uncertainty[]
  attempted_claims[]
  usage?
  request_id?
}
```

## 7. Regles de trust non negociables

1. Un LLM ne peut pas creer `SBFB verified`.
2. Un LLM ne peut pas upgrader `External OSS source index` en app verifiee.
3. Un LLM ne peut pas transformer une supposition en preuve.
4. Un resultat sans citation tombe en `hypothese` ou `non verifie`.
5. Les reponses separent toujours:
   - `Lu`;
   - `Deduit`;
   - `Verifie`;
   - `Non verifie / a confirmer`.
6. Les instructions trouvees dans un fichier source indexe sont du contenu, pas
   des instructions systeme.
7. Le provider distant ne recoit jamais de secret connu, de ligne bloquee, ni de
   dump DB brut.
8. Les actions risquee (`ask network`, `publish`, `audit quorum`, `agent_process`)
   demandent une intention explicite.

## 8. UX minimum attendue

RRV doit se comporter comme un cockpit de preuve, pas comme un chat generique:

- selector de mode: `search only`, `local`, `remote`, `network audit`;
- selector de scope: `@protocole`, `@dev`, `@web`, `@private:<group>`;
- indicateur `LocalOnly` visible;
- panneau evidence avec citations;
- panneau ProofCard;
- actions typees;
- avertissement d'egress avant provider distant;
- historique de questions avec provider/model/evidence hash;
- bouton `repondre sans LLM` pour comparer la synthese au corpus brut.

## 9. Developpement recommande

### Phase A - canon produit/securite

Livrer:

- `docs/product/RRV_PRODUCT.md`;
- `docs/protocol/RRV_SEARCH.md`;
- section RRV dans `docs/security/THREAT_MODEL.md`;
- policy provider/egress;
- format `EvidenceBundle`;
- matrice labels.

Acceptance:

- `@dev` default `LocalOnly`;
- LLM declare composer seulement;
- `/rrv` et `sbfb-search` ont une frontiere claire;
- provider distant interdit sans consentement.

### Phase B - search/proof hardening daemon

Livrer:

- search result avec `source_id`, `path/ref`, `line`, `hash`, `label`;
- endpoint proof facts ou ProofCard stable;
- audit log evidence hash;
- tests sur absence de citations.

Acceptance:

- une reponse RRV peut etre reconstruite depuis evidence hash + sources;
- aucun label verifie ne sort sans proof fact.

### Phase C - `/rrv` system route

Livrer:

- UI shell minimal;
- mode `search_only`;
- scopes `@protocole` et affichage labels;
- panneau evidence.

Acceptance:

- fonctionne sans Ollama/Claude/OpenAI;
- aucun appel reseau;
- resultat cite feed/provenance ou fichier process.

### Phase D - `sbfb-search` installable

Livrer:

- app installee utilisant le bridge;
- meme operation que `/rrv` pour recherche/proof;
- aucune API cachee cote shell.

Acceptance:

- une requete equivalente donne les memes sources;
- sandbox respectee;
- capabilities manifest explicites.

### Phase E - provider router local/central

Livrer:

- `LlmProvider` trait/config;
- provider `none`;
- provider local Ollama ou compatible local;
- provider central derriere egress consent;
- output verifier.

Acceptance:

- remote bloque sans consentement;
- local ne fait pas d'egress;
- sortie sans citations degradee;
- logs gardent provider/model/evidence hashes.

### Phase F - `@dev LocalOnly` et OSS seed

Livrer:

- index source-only;
- 10 slots OSS avec commit/licence/metadonnees;
- exclusion secrets;
- labels `External OSS source index`.

Acceptance:

- aucun repo OSS n'apparait comme SBFB verified;
- citations file/line/hash;
- process docs indexes.

## 10. Tests a exiger

| Test | Attendu |
| --- | --- |
| `rrv_search_only_no_provider` | RRV repond avec citations sans LLM configure |
| `rrv_local_provider_no_egress` | le mode local ne touche aucun host externe |
| `rrv_remote_requires_consent` | provider central bloque sans consentement explicite |
| `rrv_redacts_before_remote` | secrets/PII ne sortent pas du Privacy Gate |
| `rrv_no_citation_degrades` | reponse sans citation devient hypothese/non verifie |
| `rrv_prompt_injection_is_content` | instruction dans source indexee n'affecte pas le system contract |
| `rrv_verified_label_from_proof_only` | `SBFB verified` vient seulement de ProofCard/provenance |
| `rrv_task_submit_explicit_only` | `network_task` exige action explicite |
| `rrv_shell_and_app_contract_parity` | `/rrv` et `sbfb-search` utilisent le meme contrat public |
| `rrv_logs_no_private_prompt_by_default` | logs stockent hashes/provider/model, pas le prompt prive complet |

## 11. Decision pour Factory

Cette analyse ne bloque pas Factory a long terme. Elle dit seulement que:

- Factory peut reprendre apres que RRV sache inspecter/citer/comparer;
- Factory reste producteur d'apps et d'evidence packs;
- RRV reste lecteur/questionneur/verificateur;
- Babel reste un projet annexe cree ou package via Factory;
- une app hors Factory peut etre verifiee si elle publie le meme evidence pack.

Donc "RRV serieux avant Factory total" est coherent si le prochain sprint coupe
le scope a:

```text
RRV Core + /rrv shell + sbfb-search dogfood + provider router minimal
```

et laisse hors scope:

```text
SearchManifest reseau + @web large + private compute + inference distribuee
```

## 12. Sources repo

Sources structurantes:

- `docs/agent/PROCESS.md`: process agent vendor-neutral et provider switching.
- `prompts/agent/universal.md`: provider identity does not matter; files,
  commands, diffs, tests and reviews are authority.
- `scripts/agent/agentctl.py`: hooks and any model provider that can execute
  commands.
- `crates/nexus-worker-core/src/llm/mod.rs`: abstraction Ollama / llama.cpp.
- `crates/nexus-worker-core/src/llm/factory.rs`: backend selector.
- `crates/nexus-worker-core/src/llm/ollama.rs`: Ollama HTTP backend and schema
  validation boundary.
- `crates/nexus-core-rs/src/task.rs`: task prompt/model/response/model_digest.
- `crates/nexus-shell-daemon/src/http.rs`: current daemon search/proof routes.
- `crates/nexus-coordinator-rs/src/search.rs`: current search model.
- `crates/nexus-coordinator-rs/src/proof_card.rs`: proof facts and ProofCard.
- `web/src/bridge/protocol.ts`, `web/src/bridge/useBridge.ts`,
  `web/public/sbfb-bridge.js`: bridge/capability surface.
- `docs/security/THREAT_MODEL.md`: sandbox, app isolation and untrusted app
  posture.
- `.planning/research/rrv_protocol_boundary_analysis.md`: daemon/app boundary.
- `.planning/research/rrv_app_protocol_best_features.md`: UX/features.
- `.planning/research/SYNTHESIS_factory_rrv_protocol.md`: Factory/RRV/Babel
  roles.

External references consulted on 2026-05-22, non normative because provider
policies and APIs can change:

- OpenAI Responses API / structured outputs / statefulness:
  https://platform.openai.com/docs/api-reference/responses/create
- OpenAI migration guide noting Responses as recommended new project interface:
  https://platform.openai.com/docs/guides/responses-vs-chat-completions
- Claude API overview / direct API vs cloud platform / request limits:
  https://docs.anthropic.com/en/api/overview
- Ollama local API default base URL:
  https://docs.ollama.com/api

## 13. Sprint implication

S69 Phase E or S70 kickoff must decide explicitly:

```text
Option A: finish strict Factory/Babel/Gate 1, then RRV later
Option B: pivot next sprint to RRV Core + @dev LocalOnly + provider router
```

If Option B is selected, this file becomes an input canon for D-decisions. If it
is not copied into `.planning/active/`, it remains research and the process will
not execute it.

## 14. Final answer to the question

Oui, le chat RRV doit pouvoir tourner en local ou avec un modele centralise.
Le choix du modele est une policy runtime.

Non, RRV ne doit pas etre enferme dans une seule app installee. Il faut:

- un RRV Core dans le daemon pour les faits, preuves, index et privacy gates;
- une surface systeme `/rrv` pour bootstrap;
- une app installee `sbfb-search` pour dogfood et preuve que le contrat public
  suffit.

La version serieuse commence par la preuve, pas par le modele.
