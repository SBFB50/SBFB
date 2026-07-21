# Sprint 83 Phase G — Preflight G8

**Sprint** : 83 — chantier « Workflow Agents » (43 phases A..AJ1).
**Phase** : G — Langage workflow, agents et rôles (4 goldens), plan canonique
`.planning/active/sprint83_plan.md:629-676` (G.1 scope :631-640, G.2 livrables :642-651,
G.3 tests :653-660, G.4 acceptation :662-668, G.5 commit :670-676).
**Date** : 2026-07-21. **HEAD** : `23fdc1dc` (Phase F committée), arbre propre.
**Méthode** : mode ultracode Workflow — **5 agents de scan** (S1a prior-art OSS profond,
S1b dépendances/CVE/graphe transitif, S2 décisions historiques traversées, S3 couverture
THREAT_MODEL, S4 invariants wire/frontières), **3 passes adversariales** de réfutation +
critique de complétude (`missed[]`), puis cette synthèse. Le synthétiseur a **rejoué à HEAD**
les mesures porteuses : census DOMAIN (== **29**), exit du gate frontier (== 0, `[2 tagged]`,
`[29 frozen]`), absence des modules `graph.rs`/`agent_definition.rs`, valeurs de
`pack::PRESENTATION_KEYS` (10 clés), présence du harnais python `jsonschema 4.26.0` dans
`.venv`, structure `FactoryProcessProfile` (9 rôles), absence du node `sequence` dans le
crate, forme réelle des étiquettes `// FRONTIER:`.
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux de scan : S1a=PLAN-ADAPT,
S1b=EXECUTE, S2=PLAN-ADAPT, S3=EXECUTE, S4=PLAN-ADAPT.

---

## 1. Rationale du verdict

**Aucune décision Day-0 (D4 GELÉE) ni aucun invariant du plan n'est en tension structurelle
avec Phase G.** Le prior art réel 2026 (MS Agent Framework, OpenAI Agents SDK, LangGraph 1.0,
n8n, Dify, A2A v1.0) **valide** les choix SBFB plutôt que les contredire :

- boucle bornée STRUCTURELLE (`repeat`/`map` à borne constante-au-format) > `recursion_limit`
  global de LangGraph (compteur d'étapes non analysable statiquement) ;
- `handoff` à payload filtré > défaut « l'agent receveur voit tout l'historique » d'OpenAI ;
- `join` à politique explicite {all, any, quorum} > barrière `AddFanInBarrier` ALL-only de MS ;
- ports nominaux par référence de schéma pinnée > `handles`-string non typés de Dify/n8n ;
- fail-closed sur node-kind inconnu (le plan signé EST l'autorité) — divergence ASSUMÉE vs la
  tolérance additive du feed raw-op.

Le verdict n'est **pas DESIGN-CONFLICT** : aucune alternative rejetée au kickoff (graphe-as-code
LangGraph, gate substring, boucles non bornées, A2A/MCP comme wire) n'est ré-ouverte, et aucun
arbitrage PO n'est requis. Le `git log --grep=DEVIATION/rejected` sur le crate est **VIDE** : pas
de déviation antérieure à rouvrir sur le langage.

Le verdict n'est **pas EXECUTE** parce que Phase G **committe 4 goldens et GÈLE le profil de
nodes** : la FORME des schémas doit encoder dès maintenant les garanties que le compilateur §10.5
refusera (Phase H), sinon il faut re-geler plus tard. Neuf points de rédaction mesurés doivent
corriger l'approche brute du plan (§8) ; deux d'entre eux touchent le **texte du plan lui-même**
(G.3 « valide contre son JSON Schema » est plus faible que le libellé si une contrainte de valeur
est déclarée ; G.5 « FRONTIER posé » est mécaniquement impossible en forme machine pour un contrat
non signé). Rien n'est retiré du périmètre — il est **précisé** — donc pas SCOPE-CUT-CONSISTENT.

**Un seul risque de sévérité haute** : si `handoff` modèle son filtre de contexte comme un champ
OPTIONNEL/defaulted (miroir de l'`input_filter` OpenAI), le défaut devient tout-le-contexte et
casse l'invariant D4 ratifié « le handoff ne transmet jamais implicitement tout l'historique ».

---

## 2. Base de départ mesurée (rejouée à HEAD)

| Fait | Mesure | Source |
|---|---|---|
| Census DOMAIN | **29**, gate clean EXIT=0 `[29 frozen]` | `scripts/check-frontier-contracts.sh:218` |
| Étiquettes FRONTIER machine | **2** (signés seulement) | `pack.rs:391`, `shard_plan.rs:188` |
| Frontière contrat LOCAL | PROSE, pas de tag | `factory_profile.rs:266`, `accessibility_profile.rs:343` |
| Stubs Graph/Agent | INLINE, vides `{}`, entrées 2/3 sur 7 | `contracts.rs:53-65`, `:104-111` |
| `graph.rs`/`agent_definition.rs` | **absents** (à créer) | glob `src/` |
| Registre | `len == 7`, `deny_unknown_fields` + `title==identité` | `contracts.rs:136-150` |
| Drift-gate schema | `schema_snapshot_stub` **STRICT** (fail-on-missing depuis F) | `contracts.rs:186-217` |
| Node `sequence` | **absent** du crate (nommé pourtant kickoff:449) | grep crate |
| `PRESENTATION_KEYS` | 10 clés réservées, strip récursif | `pack.rs:1219-1230` |
| Harnais python `jsonschema` | **4.26.0 présent** dans `.venv` | `.venv/Lib/site-packages/` |
| Deps crate cible | 6 workspace-deps, `[dev-dependencies]` **vide** | `Cargo.toml:11-30` |
| schemars | 1.2.1, draft 2020-12 | `Cargo.lock`, `contracts.rs:158-165` |

---

## 3. Scan S1a — Prior art OSS profond (verdict de scan : PLAN-ADAPT)

Prior art vérifié DATÉ/VERSIONNÉ. Aucune contradiction fondée avec D4 : le prior art VALIDE les
choix SBFB. Les findings sont des **raffinements de forme de schéma à intégrer AVANT** de committer
les goldens et de geler le profil. Tous **CONFIRMED** en adversarial.

- **S1a-F1 — `handoff` : allowlist de payload/contexte REQUISE, jamais optionnelle. [P1]**
  OpenAI Agents SDK : par DÉFAUT l'agent receveur voit TOUT l'historique, `input_filter` est
  opt-in (`openai.github.io/openai-agents-python/handoffs/`, fetch 2026-07-21). L'invariant SBFB
  (`research §10.4:892-894`, `kickoff:452-453`) est l'inverse : payload schématisé filtré par
  `context_policy`, jamais tout l'historique/mémoire/outils. *Nuance adversariale honnête* : le
  modèle SBFB est déjà payload-passing par conception (pas d'héritage de conversation façon
  OpenAI), donc le risque « importer le défaut all-history » est structurellement atténué ; le
  cœur actionnable reste correct — `payload_schema_ref`/`context_policy` REQUIS (non `Option`,
  non `#[serde(default)]`).
- **S1a-F2 — `join` : politique explicite + satisfiabilité (anti-deadlock statique). [P2]**
  Plus expressif que MS (fan-in barrière ALL-only). `research §10.5:903` refuse « join dont les
  branches et la politique sont incohérentes ». En G : champ politique enum {all, any, quorum(n)}
  + déclaration explicite des branches sources. L'analyse de satisfiabilité (une politique `all`
  interdite si une branche source est gardée par une condition mutuellement exclusive → deadlock
  dans un plan signé) = Phase H. Ajouter un golden non-`all`.
- **S1a-F3 — `map`/`repeat` : borne = constante compile-time, jamais runtime. [P2]**
  LangGraph Send API n'a AUCUNE limite de cardinalité native ; son `recursion_limit` global est
  déconseillé par sa propre doc. `research §10.5:902` refuse « map sans limite de cardinalité ».
  En G : `max_cardinality`/`max_iterations` = entier constant dans le graphe signé (ou `$ref`
  vers `maxItems` d'un schéma pinné), jamais une expression.
- **S1a-F4 — Enum 18 node-kinds : union discriminée TAGGÉE, une orthographe canonique. [P2]**
  Dify accepte deux orthographes (`if-else`/`if_else`) — anti-patron FATAL pour un format signé
  JCS (deux suites d'octets pour la même sémantique). En G : enum interne-taggée `kind` avec
  `#[serde(rename="agent.invoke")]` etc. (un seul spelling dotté), `deny_unknown_fields` par
  variante, JAMAIS `untagged`, JAMAIS d'alias. *Note d'implémentation* : les noms dottés
  (`agent.invoke`, `model.invoke`, `wait.signal`, `human.review`, `capability.call`,
  `agent.as_tool`) exigent un `#[serde(rename=...)]` explicite (kebab-case ne produit pas de point).
  *Voir aussi S7-M6/M7 (roster contesté) et S7-A ci-dessous (friction serde tag + deny_unknown).*
- **S1a-F5 — `budget_policy` : struct multi-dimensionnelle TYPÉE. [P2]**
  `research §10.2:847-850` liste déjà token_budget/cost_budget/timeout/max_turns séparés sur
  l'agent. Mirroir en struct typée dans `budget_policy` du graphe ; comparaison enfant≤parent PAR
  dimension au compilateur (H). Un budget scalaire rend la comparaison indéfinie entre unités.
- **S1a-F6 — Ports NOMINAUX par digest de schéma pinné. [P2, majoritairement scope H]**
  JSON Schema est structurel par nature ; pour du vrai nominal dans un format signé, la
  compatibilité de ports = égalité de digest de schéma pinné (ou règle de sous-typage décidée).
  En G : les ports référencent un schéma par ref pinnable (`input_schema_ref`/`output_schema_ref`,
  `research §10.1:813-814`). La RÈGLE de compatibilité = décision Phase H.
- **S1a-F7 — Intégrité référentielle des edges + unicité node_id (à consigner pour H). [P2]**
  `research §10.5:900-912` (13 refus re-lus exhaustivement) ne liste NI « endpoint d'edge →
  node_id existant » NI « unicité node_id ». *Nuance* : `nodes{}` modélisé en map
  (`§10.1:816`) rend l'unicité des clés structurellement acquise si le schéma keye par id ; mais
  « node inaccessible » ne couvre PAS un edge pointant vers un id inexistant. Consigner les deux
  règles manquantes dans `node_profile_freeze` pour Phase H.
- **S1a-F8 — Fail-closed sur node-kind/champ inconnu (divergence assumée vs feed). [P2]**
  Le graphe signé refuse un node inconnu (`deny_unknown_fields` + enum taggée → erreur de parse),
  DIVERGENCE ASSUMÉE vs le feed raw-op tolérant (`CLAUDE.md §Pre-launch`). C'est la tension AM-12
  (`kickoff:480-483`). À ÉNONCER explicitement dans `sprint83_node_profile_freeze.md` + body du
  commit pour éviter la confusion future.
- **S1a-F9 — Cancel/états terminaux ALIGNÉS avec A2A v1.0. [INFO]** `research §12.7` (cancel
  honnête) conforme à A2A (`TaskNotCancelableError`, états terminaux). Rien à changer ; A2A reste
  référence-seulement (WA-10, pas une dep).
- **S1a-F10 — Le gel pinne set ET forme (pas comptage seul). [P3]** n8n montre le risque du drift
  de forme sous nom stable (typeVersion par node). SBFB n'a pas de per-node version pré-launch ;
  `node_profile_18_frozen` doit s'appuyer sur le snapshot de schéma (forme), pas seulement l'ensemble.

---

## 4. Scan S1b — Dépendances / CVE / graphe transitif (verdict de scan : EXECUTE)

Phase G n'exige **aucune dépendance runtime ni dev neuve**. Tous **CONFIRMED**.

- **S1b-1 — 0 dep neuve. [INFO]** 6 deps du crate toutes `{ workspace = true }`
  (`Cargo.toml:11-27`), `[dev-dependencies]` VIDE (`:29-30`). Remplir les stubs + committer 4
  goldens n'utilise que serde/serde_json/schemars déjà présents. G ne touche NI `Cargo.toml` NI
  `Cargo.lock` : 0-bump/0-dep tenu par construction.
- **S1b-2 — Piège `preserve_order` ACTIF. [P2]** La feature `schemars/preserve_order` est tirée
  par `ollama-rs 0.3.4`, donc ACTIVE en build `--workspace`, INACTIVE en `-p sbfb-workflow-core`
  (mesuré : `cargo tree -e features -i schemars@1.2.1` → `preserve_order └── ollama-rs`). serde_json
  bascule sur IndexMap → l'ordre des membres JSON diverge. Une comparaison de golden/snapshot en
  STRING serait verte dans un mode, rouge dans l'autre. **Comparer TOUS les goldens et le snapshot
  de schéma en `serde_json::Value`** (pattern `contracts.rs:213`, ordre-indépendant), et jouer les
  tests sous `-p` ET `--workspace`.
- **S1b-3 — Pas de crate validateur Rust `jsonschema`. [P2]** Absent du lock (`grep` = 0). Le
  `jsonschema 4.26.0` de la review Phase A est le paquet **PyPI Python** (majeur 4.x ; le crate Rust
  est en 0.x). L'ajouter tirerait ~15 nodes transitifs neufs = violation « 0 dep neuve ». Valider
  par round-trip serde typé. *Corrigé/complété par S7-A ci-dessous : le round-trip serde n'est PAS
  équivalent à la validation JSON-Schema complète.*
- **S1b-5 — Outillage schema sain côté CVE. [INFO]** Aucun RUSTSEC actif sur schemars /
  serde_json 1.0.149 / serde_jcs ; `deny.toml` sans ignore pour ces crates. Le gate supply-chain
  rouge connu (CI-1) est hors-scope schema.
- **S1b-6 — `schemars 0.9.0` = orphelin lock bénin. [INFO]** Absent de l'arbre construit
  (`cargo tree -i schemars@0.9.0` → no match) ; tiré uniquement par `serde_with` via un chemin de
  feature inactif. Ne pas tenter de le retirer.
- **S1b-7 — Drift-gate à répliquer = `schema_snapshot_stub` STRICT. [P3]** Remplir les stubs
  rafraîchit `contracts.schema.json` dans le MÊME commit (via `UPDATE_SNAPSHOTS=1`). *Voir S7-M8
  (ambiguïté `graph_schema_matches_snapshot`).*
- **S1b-4 — schemars 1.2.1 draft 2020-12 (bump S72). [INFO]** Les nouveaux schémas graph/agent
  héritent draft 2020-12 automatiquement ; le test `every_contract_emits_nonempty_draft_2020_12_object`
  les couvrira sans code neuf.

---

## 5. Scan S2 — Décisions historiques traversées (verdict de scan : PLAN-ADAPT)

`git log --grep=DEVIATION/rejected/scope-cut` sur le crate = VIDE (aucune déviation antérieure sur
le langage). Tous **CONFIRMED** (re-mesurés verbatim en adversarial).

- **S2-G-01 — FRONTIÈRE en PROSE, PAS de tag machine. [P1]** `SbfbWorkflowGraph.v1`/
  `SbfbAgentDefinition.v1` vivent DANS le pack signé (D4 pt1), SANS domaine ni format-version
  propres. Un tag `// FRONTIER: <name> domain= version=` soit fabrique une famille census (casse
  le gel 29), soit est malformé (`check-frontier-contracts.sh:148-160` exige `const <domain>` /
  `const <version>` déclarés → échec). Précédent EXACT : `factory_profile.rs:262-271` (contrat
  LOCAL, « carries no // FRONTIER: domain/version tag »). **La formulation de la tâche « tag
  // FRONTIER: posé au commit de remplissage (exception AM-3) » = étiquette en forme PROSE** :
  commentaire de provenance simple + drift-gate schema, comme Phases B/D/E.
- **S2-G-02 — Census GELÉ à 29 en G. [P1]** Aucune nouvelle famille, aucun `const DOMAIN_*` ni
  `*_FORMAT_VERSION`. Graph/Agent ne sont pas signés indépendamment. Le body annonce « 0 bump wire,
  census 29 intact ».
- **S2-G-03 — Tests golden STRICT fail-on-missing. [P1]** `contracts.rs:186-217` : golden
  absent/vide = PANIC, jamais self-heal. `UPDATE_SNAPSHOTS` reste le seul chemin de refresh.
- **S2-G-04 — Frontière de scope G/H stricte. [P1]** G = SCHÉMAS + 4 goldens qui VALIDENT
  (conformité schemars) + gel du profil. G ne COMPILE PAS, ne fait PAS la validation statique
  §10.5, ne produit PAS de `CompiledPlan` ni de `*.compiled.golden.json` — tout cela est Phase H
  (`plan:638-640`, « aucun format d'exécution figé avant les 4 goldens »).
- **S2-G-05 — Remplir change le schema → régénérer `contracts.schema.json` sous build workspace. [P2]**
  Via `UPDATE_SNAPSHOTS=1` (preserve_order actif via ollama-rs→schemars). L'`assert_eq` porte sur
  `Value` donc le TEST ne rougit pas pour l'ordre seul ; la régénération-en-workspace importe pour
  le déterminisme byte du FICHIER committé. Vérifier Win + Docker canonique 1.94.
- **S2-G-06 — Conventions préservées. [P2]** `len == 7` (Graph=2/7, Agent=3/7),
  `deny_unknown_fields`, `title==identité`. Test `every_contract_emits_...` reste vert.
- **S2-G-07 — `handoff` ≠ `agent.as_tool` : deux primitives distinctes. [P2]** handoff = transfert
  de contrôle + payload filtré ; agent.as_tool = parent garde synthèse + guardrails
  (`research §10.4:887-894`). Test discriminant `handoff_not_agent_as_tool`.
- **S2-G-08 — Profil CANDIDAT, gelé APRÈS les goldens ; gel possible ≤18. [P2]**
  `research §10.3:880-881` : « Cette liste n'est pas gelée. Les goldens doivent montrer quels nodes
  peuvent être fusionnés ». Si fusion : documenter la justification, ne pas gonfler à 18.
- **S2-G-09 — T1 = N-A-no-frontend-change (0 miroir TS en G). [P2]** Le canvas = Phase T. Évite le
  piège parité TS des rounds Codex Phase F. 0 diff `web/`.
- **S2-G-10 — Mutation-avant-confiance + réfutation adversariale des faits Codex. [P2]**
  Phase E : 2 mutations laissaient la suite verte, un contrôle rouge était une TAUTOLOGIE ; Phase B
  loop-6 : Codex confabulait une API inexistante. Muter chaque golden (JSON malformé, node retiré,
  handoff illicite) et prouver que SEUL le test attendu rougit.
- **S2-G-11 — fmt dual-toolchain 1.94/1.95 + goldens LF. [P2]** La 1.95 locale tolère plus que la
  1.94 canonique Docker. Committer les `*.workflow.golden.json` en LF.
- **S2-G-12 — D4/D2 gelé prime sur l'argument code-mort. [P2]** Précédent Phase F B1 (Codex r3 :
  4 domaines D2 déclarés, pas 1). Déclarer les 4 policy types + `capability_requests[]`/
  `agent_refs[]`/`subworkflow_refs[]`/`tool_refs[]`/`allowed_handoffs[]`/`guardrail_refs[]` comme
  champs de schéma même si consommés en H/L.
- **S2-G-13 — G n'OWN aucune ligne de gate §23. [P2]** Les 4 goldens sont des fixtures ; ils ne
  FERMENT PAS G-VERTICALS (row 24, ouvre en Z). Exit de G = condition machine (4 goldens valides +
  artefact de gel), pas une fermeture de gate.
- **S2-G-14 — Refs pinnées déclaratives en G ; fermeture des capacités en H/L. [P2]**
  `capability_requests[]`/`tool_refs[]`/`allowed_handoffs[]` = références pinnées ; l'intersection
  7-couches (§14.1) et le refus « capacité absente du manifeste » (§10.5) = Phase H/L. Ne pas
  implémenter l'algèbre de capacités en G.

---

## 6. Scan S3 — Couverture THREAT_MODEL (verdict de scan : EXECUTE)

Le design D4 PORTE l'invariant Plan-Then-Execute/CaMeL — à condition que la RÉDACTION des schémas
bake les contraintes structurelles. Tous **CONFIRMED**.

- **S3-01 — `classifier`/`switch` : sortie bornée à un ENSEMBLE d'arêtes STATIQUE. [P1]** Le modèle
  SÉLECTIONNE un label dans un set clos ; une valeur non-matchée route vers un défaut/erreur
  déterministe, JAMAIS vers une destination neuve (`research §10.3:870`, §19.3 : l'injection ne
  change pas la topologie/les destinations). *Nuance d'ancre* : §19.3:1604 « destinations réseau »
  vise l'egress ; la garantie de ROUTAGE repose sur l'immutabilité de topologie + §10.5 accessibilité.
- **S3-02 — `capability.call` : ref pinnée du manifeste, AUCUN endpoint/url/command/code inline. [P1]**
  `research §10.5:906,910` refuse « capacité absente du manifeste » + « endpoint ou code exécutable ».
  Baseline propre : `pack.rs:418-420` (`requested_capabilities` = ref opaque). Schéma =
  `{ capability_ref ∈ requested_capabilities, args typés par schéma pinné }` + `deny_unknown_fields`.
- **S3-03 — `handoff` : `target ∈ allowed_handoffs[]` statique + `payload_schema_ref` pinné. [P1]**
  Sinon un LLM redirige le contrôle (`research §10.5:908`). Distinct de `agent.as_tool`.
- **S3-04 — `repeat`/`map` : borne entière REQUISE constante-au-format. [P1]** Jamais une référence
  à l'état de run ni une sortie LLM (`research §10.5:901-902`, §19.6). Rejeter `recursion_limit`
  global (kickoff:494-496 « repeat borné structurel strictement plus fort »).
- **S3-05 — Budgets REQUIS non-defaultable-à-illimité. [P2]** Tension AM-12 : la tolérance
  `#[serde(default)]` (robustesse runtime) NE doit PAS s'appliquer aux budgets — un budget omis qui
  désérialise à « illimité » casse §19.6. Documenter « champ sécurité, pas robustesse runtime ».
- **S3-06 — `id` de node stable+unique = précondition node_instance_id/HITL (Phase L). [P2]**
  `research §12.2:982-996` (node_instance_id encode le chemin), §14.4:1284-1293 (toute mutation
  invalide l'approbation). `id: String` REQUIS et unique par scope ; validation d'unicité dans les
  goldens.
- **S3-07 — Zéro secret dans graph/agent ; projection allow-listée par CONSTRUCTION. [P2]**
  `secret_refs` restent sur le pack root (`pack.rs:428`) ; agent référence via `secret_ref` opaque.
  `WorkflowPublicProjection.v1` = patron A9 §17 (champ ABSENT du type, pas `#[serde(skip)]` —
  `THREAT_MODEL.md:1683`).
- **S3-08 — LACUNE RÉELLE : THREAT_MODEL v21 ne porte aucun asset « graphe signé = plan ». [P2]**
  Assets A1..A9 s'arrêtent avant tout asset plan/graphe (grep `Plan-Then-Execute|A10|CompiledPlan`
  = NONE). L'invariant §19.3 ne vit que dans le Research Pack/kickoff. **Carry P2** : ajouter
  l'asset/surface à la phase d'enforcement (H où §10.5 devient exécutable, ou AH), pas skiper en
  silence. Phase G étant pur format+goldens, le report est défendable — mais TRACÉ.
- **S3-09 — `cancellation_policy` = conditions déterministes host/opérateur/budget/timeout. [INFO]**
  `wait.signal` = signal TYPÉ host-driven ; aucun node ne route une sortie LLM vers un cancel
  autoritaire (`research §12.7`). Enforcement runtime = Phase J.
- **S3-10 — Phase G n'introduit AUCUN verbe bridge. [INFO]** Livrables = types schemars + goldens +
  freeze ; aucune route daemon, aucun ajout allowlist. `capability.call` est une DÉCLARATION dans
  le plan, pas le câblage d'un verbe. Conforme à l'ordre P0 C→D→E→F (C DONE).

---

## 7. Passe adversariale + critique de complétude

**Aucun finding réfuté.** Les 3 passes adversariales ont re-mesuré chaque finding : tous **CONFIRMED**
sauf **deux ADJUSTED** (sur-affirmation resserrée), et la critique de complétude a remonté **8 items
`missed[]` fondés** que le synthétiseur a re-vérifiés à HEAD et intègre comme findings additionnels.

### 7.1 Findings ajustés (sur-affirmation resserrée, pas réfutée)

- **S4-F3 ADJUSTED — « deserialize round-trip = validation JSON-Schema ÉQUIVALENTE » est un
  SURCLAIM.** `serde` + `deny_unknown_fields` impose types + required + refus de champ inconnu,
  PAS les contraintes de VALEUR que schemars peut émettre (`range` numérique sur une borne
  `repeat`/`map`, `pattern` sur un id, `minItems`/`minLength`, `const`/`enum` au-delà des variantes).
  Un golden type-compatible mais hors-contrainte PASSERAIT le deserialize tout en violant son JSON
  Schema généré → « valide contre son JSON Schema » (G.1/G.3) serait plus faible que le libellé.
  **Équivalence conditionnelle** : vraie SEULEMENT si le schéma n'encode aucune contrainte de valeur
  au-delà de types/required/deny_unknown. *Actionnable* : voir A9 §8.2.
- **S1a-F4 ADJUSTED — le roster « 18 » est CONTESTÉ, pas réglé.** Deux sources GELÉES divergent :
  `kickoff:449` (D4 GELÉE) nomme `sequence/switch/classifier/parallel/map/join/repeat...` incluant
  **`sequence`** — ABSENT de la table `research §10.3` (18 rangs re-comptés : input/transform/
  model.invoke/agent.invoke/agent.as_tool/handoff/capability.call/knowledge.query/switch/classifier/
  parallel/map/join/repeat/human.review/subworkflow/wait.signal/output). Mesuré : `grep sequence`
  dans le crate = seulement le PK RunEvent `(run_id, sequence)`, **aucun node kind `sequence`**.
  Le set/orthographe DOIT être réconcilié DANS l'artefact de gel AVANT que `node_profile_18_frozen`
  ait un sens. *Actionnable* : voir M6/M7 + A10 §8.2.

### 7.2 Findings additionnels issus de `missed[]` (re-vérifiés fondés)

- **M1 — Cohérence `factory.workflow.golden.json` ↔ `FactoryProcessProfile.v1` (Phase D) non
  gardée. [P1]** Le plan met Factory PREMIER, mais aucun test G.3 ne cross-check le golden Factory
  contre le contrat process DÉJÀ committé. Re-vérifié : `FactoryProcessProfile` porte **9 rôles**
  (`factory_profile.rs:422-427` : driver, reviewer, auditor, codex-verifier…), invariants
  `codex_output_rewritable=false` (`:637`), no-auto-PASS. Un golden Factory nommant ses agents
  « dev/checker/ai » ou omettant l'auditor validerait schemars-vert tout en divergeant de ce que
  les hooks du repo enforce — analogue exact du P1 `sbfb-ideas` raté en Phase C. **Ajouter un test
  qui asserte que les agents/rôles du golden Factory correspondent aux rôles + invariants de
  `FactoryProcessProfile.v1`.**
- **M2 — Indépendance driver/reviewer/Codex : livrable G.1, aucun test G.3. [P1]** `plan:632-634`
  liste explicitement « contraintes d'indépendance driver/reviewer/Codex » et le gate
  G-REVIEW-INDEPENDENCE existe (`plan:209`), mais `handoff_not_agent_as_tool` ne teste PAS que les
  3 rôles sont des agents distincts. **Le golden Factory doit encoder 3 `SbfbAgentDefinition`
  distincts (`instructions_ref` distincts, aucun handoff qui les fusionne) + un test G qui l'asserte.**
- **M3 — Vacuité de `four_goldens_validate`. [P1]** Le learning Phase A « t2 vacuous attrapé par
  Codex » N'EST PAS appliqué à G, et S1b-3/S4-F3 poussent DANS le piège. Un golden `nodes:[]`/
  `entrypoints:[]` (ou `{}` si `serde(default)`) passe VACUEUSEMENT le round-trip. Re-vérifié : le
  harnais python `jsonschema 4.26.0` EXISTE (`.venv/Lib/site-packages/jsonschema`, celui des 4
  contre-tests Codex de Phase A). **Fermer la vacuité par (a) un CONTRÔLE NÉGATIF discriminant (un
  golden malformé qui DOIT rougir), (b) des assertions de richesse (chaque golden exerce ≥1 handoff
  à allowlist, ≥1 repeat borné, ≥1 capability.call), et (c) au choix : garder le schéma sans
  contrainte de valeur [documenté] OU valider les 4 goldens contre le schéma ÉMIS via le harnais
  python existant / un `validate()` Rust.** Ne PAS ajouter de crate Rust `jsonschema`.
- **M4 — Collision avec `pack::PRESENTATION_KEYS` (Phase F, committé). [P2]** Re-vérifié :
  `pack.rs:1219-1230` réserve **10** noms (layout/position/x/y/width/height/color/collapsed/zoom/
  viewport) INTERDITS « à toute profondeur » ; `graph_semantic_view` (`:1236`) les STRIP
  silencieusement. Si `SbfbWorkflowGraph.v1` (ou un sous-champ de node/agent) déclare l'un de ces
  10 comme champ sémantique, deux graphes différents s'effondrent sur le MÊME digest `graph_semantic`.
  **Asserter que les property-names RÉCURSIFS des schémas Graph/Agent sont DISJOINTS de
  `pack::PRESENTATION_KEYS`** (décision de forme à figer en G, pas à découvrir en H/T).
- **M5 — Modèle de document F↔G (`deny_unknown_fields` vs présentation/canvas) non résolu. [P2]**
  Un golden valide `SbfbWorkflowGraph.v1` (`deny_unknown_fields`) NE PEUT PAS porter x/y/layout →
  les goldens G sont sémantiques-seulement et ne peuvent exercer `pack::presentation_view`. Or
  « canvas puis outline = même digest » (WA-T1.2, gate G-A11Y-PARITY) exige un document canvas
  SUR-ENSEMBLE — décision (sidecar/enveloppe vs intra-graph) que la forme du schéma G active ou
  verrouille. **Faire passer ≥1 golden par `graph_semantic_view` (public, cheap) pour prouver que
  le format est CONSOMMABLE par la machinerie pack committée** et attraper M4.
- **M6 — Discordance `sequence` D4 vs §10.3 (voir S1a-F4 ADJUSTED). [P2]** Réconcilier `sequence`
  (kickoff:449, GELÉE) vs la table 18 (§10.3, GELÉE, invite à fusion) DANS
  `sprint83_node_profile_freeze.md` AVANT que `node_profile_18_frozen` soit fiable.
- **M7 — `node_profile_18_frozen` = test de STABILITÉ, pas de CORRECTNESS. [P2]** `enum == liste`
  ne prouve que l'accord enum↔liste committée ENSEMBLE, jamais que la liste est JUSTE (dropper
  `sequence` ou fusionner 2 nodes était-il fondé ?). Risque tautologique (learning Phase E). **La
  liste gelée doit être une DÉCISION humaine datée dans l'artefact (portant la réconciliation M6),
  et le test un SET-check discriminant (ajout/retrait d'UNE variante → rouge), PAS `count==18`.**
- **M8 — Ambiguïté du drift-gate. [P3]** `plan:659` nomme `schema_drift::tests::graph_schema_matches_snapshot`,
  mais (a) aucun module `schema_drift` n'existe et (b) `contracts::tests::schema_snapshot_stub`
  (`contracts.rs:186-217`) snapshotte DÉJÀ les 7 identités (dont Graph/Agent, entrées 2/3).
  **Décider : étendre `schema_snapshot_stub` (source unique `contracts.schema.json`) OU un snapshot
  distinct par-instance sur les 4 `*.workflow.golden.json` — éviter un test redondant OU un 2e
  fichier snapshot qui peut driver.**

### 7.3 Angles d'implémentation supplémentaires (fondés, à verrouiller)

- **S7-A — Friction `serde` : `deny_unknown_fields` × enum internally-tagged.** Le champ `tag` peut
  être traité comme inconnu (limitation serde documentée). **VÉRIFIER par mutation que l'encodage
  exact retenu** (externally-tagged, struct-wrapper `kind` + flatten, ou per-variant-struct)
  compile ET rejette réellement un champ inconnu — risque d'implémentation réel derrière S1a-F4.
- **S7-B — Overrides de `agent.invoke` : atténuation MONOTONE.** `research §10.2:853-855` : un node
  `agent.invoke` « fixe les overrides autorisés ». Un override pourrait re-élargir
  budget/capacités/contexte au-delà de la `AgentDefinition` — vecteur d'échappement à verrouiller
  dans le schéma (les overrides ne peuvent qu'ATTÉNUER), au même titre que enfant≤parent.
- **S7-C — Distinction borne (compile-time) vs cardinalité (runtime) de `map`.** `map` est
  data-dependent : seule la BORNE max est constante ; la cardinalité effective reste la taille de
  la collection au runtime. Modéliser `max_cardinality` comme DISTINCT du compte réel (éviter la
  lecture « borne = fan-out fixe »).
- **S7-D — Tension mutation-test vs 0-dep.** La discipline « muter chaque golden » (S2-G-10) reste
  faisable SANS dep (JSON malformé, node retiré, handoff illicite édités à la main) ; ne PAS
  atteindre `proptest`/`quickcheck` (= dev-dep neuve). À tracer explicitement.

---

## 8. Contraintes bindantes pour l'implémentation (le code suit CE préflight)

### 8.1 Blocants — à trancher AVANT le premier `Edit`

- **B1 — Réconcilier le roster de nodes.** Trancher `sequence` (kickoff:449) vs la table 18 de
  §10.3 (sans `sequence`) et documenter la décision datée dans `sprint83_node_profile_freeze.md`.
  Gel possible ≤18 si fusion justifiée. Le test `node_profile_18_frozen` = SET-check discriminant,
  pas `count==18` (M6, M7, S1a-F4).
- **B2 — Encodage de l'enum de nodes.** Union discriminée TAGGÉE (interne `#[serde(tag="kind")]`
  recommandé), une orthographe canonique dottée par kind via `#[serde(rename=...)]`,
  `deny_unknown_fields` par variante ; JAMAIS `untagged`, JAMAIS d'alias. **Vérifier par mutation
  que l'encodage rejette réellement un champ inconnu** (friction serde, S7-A).
- **B3 — `handoff` : filtre de contexte REQUIS.** `payload_schema_ref`/`context_policy` = champs
  REQUIS (jamais `Option`/`#[serde(default)]`). Le golden Factory exhibe un handoff à allowlist
  explicite. Distinct de `agent.as_tool` (test `handoff_not_agent_as_tool` discriminant) (S1a-F1,
  S3-03, S2-G-07).
- **B4 — FRONTIÈRE en PROSE.** Ne PAS écrire `// FRONTIER: SbfbWorkflowGraph domain=... version=...`
  (impossible, casse le gate/census). Suivre `factory_profile.rs:262-271` : commentaire de
  provenance + drift-gate schema. AUCUN `DOMAIN_*` ni `*_FORMAT_VERSION` neuf — census reste 29
  (S2-G-01, S2-G-02, S4-F1/F2).
- **B5 — Fermer la vacuité de `four_goldens_validate`.** Un contrôle NÉGATIF discriminant (golden
  malformé → rouge) + assertions de richesse (≥1 handoff à allowlist, ≥1 repeat borné, ≥1
  capability.call par golden pertinent). Décider la stratégie de conformité de valeur (schéma sans
  contrainte de valeur documenté, OU validation contre le schéma émis via harnais python
  `jsonschema 4.26.0` existant / `validate()` Rust). Ne PAS ajouter de crate Rust `jsonschema`
  (M3, S4-F3).

### 8.2 Adaptations obligatoires

- **A1 — Modules dédiés.** Créer `pub mod graph;` (ou `pack`-style) + `pub mod agent_definition;`
  dans `lib.rs`, re-exportés via `pub use` dans `contracts.rs`, remplacer les stubs inline
  `contracts.rs:53-65`. SPDX `AGPL-3.0-or-later` en tête de chaque `.rs` neuf. Garder `len == 7`
  (S4-F7).
- **A2 — Comparaisons en `Value`.** TOUS les goldens et le snapshot de schéma comparés en
  `serde_json::Value`, jamais en string (piège `preserve_order` via ollama-rs). Jouer les tests
  sous `-p` ET `--workspace` (S1b-2).
- **A3 — Refresh du golden schema.** Régénérer `contracts.schema.json` via `UPDATE_SNAPSHOTS=1`
  SOUS build workspace, dans le MÊME commit que le remplissage. Conserver `deny_unknown_fields` +
  `title==identité` sur chaque type neuf (S2-G-05, S2-G-06).
- **A4 — `join` à politique.** Champ enum {all, any, quorum(n)} + branches sources déclarées ;
  golden non-`all` ; satisfiabilité → Phase H (S1a-F2).
- **A5 — `map`/`repeat` bornés.** `max_cardinality`/`max_iterations` = entier constant REQUIS,
  distinct de la cardinalité runtime ; jamais une expression/sortie LLM (S1a-F3, S3-04, S7-C).
- **A6 — `budget_policy` typé multi-dimensionnel.** {token/cost/timeout/max_turns} séparés, unités
  explicites, REQUIS non-defaultable-à-illimité ; overrides `agent.invoke` monotones (atténuation
  seule) (S1a-F5, S3-05, S7-B). Déclarer aussi error/timeout/cancellation policies +
  capability_requests[]/agent_refs[]/subworkflow_refs[]/tool_refs[]/allowed_handoffs[]/guardrail_refs[]
  même si consommés en H/L (S2-G-12, S2-G-14).
- **A7 — Nodes porteurs de risque.** `classifier`/`switch` → set d'arêtes STATIQUE (labels→cibles
  pinnées), défaut/erreur déterministe, aucun champ destination libre. `capability.call` =
  `{ capability_ref ∈ requested_capabilities, args typés par schéma pinné }`, AUCUN
  endpoint/url/host/command/code inline. Ports par `input_schema_ref`/`output_schema_ref` pinnables
  (S3-01, S3-02, S1a-F6).
- **A8 — Zéro secret / `id` node.** Aucun secret dans graph/agent (`secret_refs` sur pack root ;
  `secret_ref` opaque) ; `WorkflowPublicProjection` allow-listée par CONSTRUCTION (patron A9 §17,
  champ absent du type). `id: String` REQUIS, stable, unique par scope (précondition
  node_instance_id/HITL) ; `cancellation_policy` = conditions déterministes host/budget/timeout,
  `wait.signal` = signal typé, aucun cancel piloté par sortie LLM (S3-06, S3-07, S3-09).
- **A9 — Disjonction PRESENTATION_KEYS.** Asserter que les property-names RÉCURSIFS de
  `SbfbWorkflowGraph.v1` + `SbfbAgentDefinition.v1` sont DISJOINTS de `pack::PRESENTATION_KEYS`
  (10 clés). Décider le modèle de document F↔G (sidecar/enveloppe vs intra-graph) ; faire passer ≥1
  golden par `graph_semantic_view` pour prouver la consommabilité (M4, M5).
- **A10 — Artefact de gel.** `sprint83_node_profile_freeze.md` DATÉ, après validation des 4
  goldens, portant la réconciliation M6, énonçant le fail-closed sur node-kind inconnu (divergence
  assumée vs feed, S1a-F8), et le partage set+forme du gel (S1a-F10). Consigner pour Phase H les 2
  règles statiques manquantes de §10.5 : intégrité référentielle des edges + unicité node_id
  (S1a-F7).
- **A11 — Cohérence Factory + indépendance des rôles.** Test assertant que les agents/rôles du
  golden Factory correspondent à `FactoryProcessProfile.v1` (9 rôles, `codex_output_rewritable=false`,
  no-auto-PASS) ; 3 `SbfbAgentDefinition` distincts pour driver/reviewer/Codex (instructions_ref
  distincts, aucun handoff fusionnant) + test d'indépendance (M1, M2).
- **A12 — Drift-gate tranché.** Décider : étendre `schema_snapshot_stub` (source unique) OU
  snapshot distinct par-instance sur les 4 goldens. Éviter redondance/2e fichier drifting (M8).
- **A13 — Carry P2 THREAT_MODEL.** Tracer l'ajout d'un asset/surface « graphe signé = plan »
  (Plan-Then-Execute §19.3) à la phase d'enforcement H (ou AH). Report EXPLICITE dans le body G
  (précédent Phase E), jamais un silence (S3-08).

### 8.3 Interdits

- Ne PAS déclarer de `DOMAIN_*` ni `*_FORMAT_VERSION` neuf (census gelé 29).
- Ne PAS poser d'étiquette `// FRONTIER:` machine sur graph/agent (contrats non signés).
- Ne PAS toucher `Cargo.toml`/`Cargo.lock` (6 deps workspace suffisent, 0 dep neuve).
- Ne PAS ajouter le crate Rust `jsonschema` ni aucune dev-dep (`proptest`/`quickcheck`).
- Ne PAS comparer goldens/snapshot en string (piège `preserve_order`).
- Ne PAS toucher `web/` (T1 = N-A-no-frontend-change ; 0 miroir TS).
- Ne PAS implémenter la compilation, la validation statique §10.5, `CompiledPlan`, ni de
  `*.compiled.golden.json` (Phase H).
- Ne PAS déclarer un champ graph/agent portant un nom de `PRESENTATION_KEYS`.
- Ne PAS geler `node_profile_18_frozen` par `count==18` (SET-check discriminant obligatoire).

### 8.4 Invariants à vérifier (commande par invariant)

| Invariant | Commande | Attendu |
|---|---|---|
| census intact | `bash scripts/check-frontier-contracts.sh` | exit 0, `[29 frozen]` |
| registre 7 | `cargo nextest run -p sbfb-workflow-core --locked` | exit 0, `len == 7` |
| 4 goldens valident + non vacueux | tests G + contrôle négatif | golden malformé → ROUGE |
| 0 dep neuve | `git diff Cargo.toml Cargo.lock` | vide |
| comparaison ordre-robuste | tests sous `-p` ET `--workspace` | verts dans les deux |
| golden schema à jour | `UPDATE_SNAPSHOTS=1` puis run sans var | pas de drift |
| SPDX | `bash scripts/check-spdx.sh` | exit 0 |
| 0 diff web/ | `git diff --stat web/ tools/factory-operator/` | vide |
| dual-platform | `cargo fmt --all --check` ; `clippy --workspace --all-targets -D warnings` ; `nextest --workspace` ; `test --doc` ; `build -p nexus-shell-daemon --release` | verts Win **ET** Docker `sbfb-ci` (1.94) |

---

## 9. Risques clés

- **R1 — `four_goldens_validate` vacueux (le plus dangereux).** Le round-trip serde
  (`deny_unknown_fields`) est plus faible que le JSON Schema émis (pas de `minItems`/`range`/`pattern`)
  ; un golden quasi-vide ou hors-contrainte passe VERT. Sans contrôle négatif + richesse, l'exit
  machine ne prouve rien — exactement la vacuité que Codex a BLOQUÉE en Phase A (M3, S4-F3).
- **R2 — Fuite tout-contexte du handoff.** Si le filtre de contexte est defaulted, le défaut devient
  tout-l'historique (patron OpenAI) et casse l'invariant D4 ratifié (S1a-F1).
- **R3 — Roster contesté + gel tautologique.** `sequence` (kickoff GELÉE) absent de §10.3 ;
  `node_profile_18_frozen` à `count==18` ou `enum==liste committée ensemble` est une tautologie
  (learning Phase E) — un mutant `handoff`→`handoff2` dans l'enum ET la liste passerait vert
  (M6, M7).
- **R4 — Collision `PRESENTATION_KEYS`.** Un champ graph/agent nommé `layout`/`position`/`x`/`y`/…
  est strippé silencieusement par `graph_semantic_view` → deux graphes différents = même digest
  `graph_semantic` (M4).
- **R5 — FRONTIÈRE machine sur contrat non signé.** Un tag `// FRONTIER: ... domain= version=` sur
  graph/agent rougit le gate (const domain/version absents) ou casse le census (S2-G-01, S4-F2).
- **R6 — `preserve_order`.** Comparer en string (pas `Value`) → vert sous `-p`, rouge sous
  `--workspace` (S1b-2).
- **R7 — Friction serde tag + `deny_unknown_fields`.** L'encodage retenu peut ne pas rejeter
  réellement les champs inconnus comme voulu → `deny_unknown` vert pour une mauvaise raison ; muter
  avant de croire (S7-A, S2-G-10).
- **R8 — Cohérence Factory / indépendance rôles « claimed-not-tested ».** G.1 liste le livrable, G.3
  n'a aucun test — un golden Factory divergent de `FactoryProcessProfile.v1` valide schemars-vert
  (M1, M2).
- **R9 — Ambiguïté drift-gate.** `graph_schema_matches_snapshot` redondant avec `schema_snapshot_stub`,
  ou 2e fichier snapshot qui peut driver, laissant les 4 instances de golden sans snapshot propre
  (M8).

---

## 10. Verdict G8

**PLAN-ADAPT.**

Justification (§4.5.7) — **aucune décision Day-0 (D4 GELÉE) ni aucun invariant du plan n'est en
tension structurelle** : data-flow à ports typés, `handoff` ≠ `agent.as_tool`, `repeat`/`map`
bornés structurellement, invariant Plan-Then-Execute, budgets dans le format signé restent
exactement ceux du kickoff `:445-499`. Le prior art réel 2026 VALIDE ces choix. Le verdict n'est
**pas DESIGN-CONFLICT** : aucune alternative rejetée n'est ré-ouverte, aucun arbitrage PO n'est
requis — les corrections sont des raffinements de FORME de schéma et de spécification de test, tous
à l'intérieur du périmètre gelé, chacun appuyé sur une evidence code/OSS rejouée à HEAD (census 29,
gate clean, `PRESENTATION_KEYS` 10 clés, absence du node `sequence`, harnais python `jsonschema`,
9 rôles `FactoryProcessProfile`, forme réelle des tags FRONTIER). Le verdict n'est **pas EXECUTE**
parce que Phase G committe des goldens et GÈLE le profil : neuf points mesurés doivent corriger
l'approche brute du plan avant le premier `Edit` (dont deux touchant le texte du plan : G.3 « valide
contre son JSON Schema » plus faible que le round-trip serde, et G.5 « FRONTIER posé » impossible en
forme machine). Il n'est **pas SCOPE-CUT-CONSISTENT** : rien n'est retiré — le périmètre est
conservé et précisé.

**Approche corrigée que le code suit** (autorité : ce préflight, pas le texte brut du plan) :

1. Réconcilier le roster `sequence`/§10.3 et geler le profil par une décision humaine datée +
   SET-check discriminant — **B1, M6, M7**.
2. Encoder l'enum de nodes en union discriminée taggée, une orthographe canonique dottée,
   `deny_unknown_fields` par variante, vérifié par mutation — **B2, S7-A**.
3. Rendre le filtre de contexte du `handoff` REQUIS ; distinguer `agent.as_tool` — **B3**.
4. Poser la frontière graph/agent en PROSE (0 `DOMAIN_*`, census 29) — **B4**.
5. Fermer la vacuité des goldens : contrôle négatif + richesse + stratégie de conformité de valeur,
   sans crate Rust `jsonschema` — **B5, M3**.
6. Créer les modules dédiés re-exportés, comparer en `Value`, refresh du golden schema sous
   workspace dans le même commit — **A1, A2, A3**.
7. Baker les garanties structurelles : `join` à politique, `map`/`repeat` bornés, `budget_policy`
   typé + overrides monotones, nodes à risque (classifier/capability.call/ports), zéro secret,
   `id` node unique — **A4-A8**.
8. Asserter la disjonction `PRESENTATION_KEYS`, la cohérence Factory + indépendance des rôles ;
   trancher le drift-gate ; tracer le carry P2 THREAT_MODEL — **A9-A13, M1, M2, M4, M5, M8, S3-08**.
9. Body 9 sections conforme à G.5, en y ajoutant : « 0 bump wire, census 29 intact, 0 dep runtime »,
   fail-closed node-kind inconnu (divergence assumée vs feed), report explicite THREAT_MODEL nommant
   la lacune « graphe signé = plan » et sa phase propriétaire (H/AH), scope cuts datés (compilation
   §10.5 / capacités fermées → H ; HITL → L ; canvas → T).
