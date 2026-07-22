Le verdict global est **GAP** : l’ossature de `CompiledPlan.v1` est solide, mais plusieurs garanties annoncées comme fermées restent contournables, notamment le binding document↔digest, la provenance à travers les sous-workflows et la satisfiabilité des joins imbriqués.

Les sections inline 2–4 n’étant pas numérotées, les références y utilisent `fichier:fonction` suivies de la ligne exacte, sans inventer de numéro.

### 1. `compile(pack) -> CompiledPlan.v1` — PARTIEL

Les neuf familles sont structurellement présentes dans `CompiledPlan` : pack, artefacts, capacités, instances, câblage, budgets, placement, politiques d’exécution et profil compilateur (`crates/sbfb-workflow-core/src/compile.rs:CompiledPlan`). Les identités RFC 6901 sont correctement injectives : `segment.replace('~', "~0").replace('/', "~1")`, puis `format!("{scope}/{}", escape_segment(node_id))` (`compile.rs:escape_segment`, `compile.rs:instance_id`).

Les deux digests passent bien par `digest_for`; aucun `serde_json::to_vec` n’est utilisé pour digérer (`compile.rs:compiled_plan_digest`, `compile.rs:compile`). Le tri des edges et des `human_gates` est explicite.

Mais la fidélité autoritaire n’est pas acquise :

- Le compilateur ne lie jamais le contenu de `ProvidedDocument.document` à `ProvidedDocument.blake3`. Le contrat reconnaît explicitement : « computing a file's byte digest is the materialiser's job », puis `load_workflow_document` contrôle seulement format du pin, appartenance au lock et égalité avec `expected_pin`; aucun hash du document n’est calculé (`compile.rs:ProvidedDocument`, `compile.rs:load_workflow_document`). `synth_input` aggrave l’écart avec la ligne exacte `let root_pin = pin_for("doc:root");`, indépendante du contenu (`compile.rs:synth_input`). Ainsi, le même `PACK.lock` et le même `pack_digest` peuvent produire des plans sémantiquement différents si le caller change la `Value`.

- Le cache agents court-circuite le binding :  
  `if let Some(parsed) = self.agents_parsed.get(&agent_ref.agent_id) { return Ok(parsed.clone()); }`  
  est exécuté avant `check_pin` et avant la comparaison avec le document fourni (`compile.rs:load_agent`). Deux scopes déclarant le même `agent_id` sous deux digests différents font donc ignorer le second pin.

- `record_digest` termine par `.insert(id.to_string(), digest.to_string());` sans refuser un même ID sous deux digests, sauf pour les schémas qui ont une garde séparée (`compile.rs:record_digest`). Prompts, agents, tools, corpora, guardrails et workflows peuvent être écrasés dans `artefact_digests`.

- Même problème pour les workflows : `self.semantic_views.insert(graph.workflow_id.clone(), view);` écrase silencieusement un workflow homonyme. Un root et un enfant de même `workflow_id`, mais de contenus différents, laissent un seul semantic view et un seul digest dans le plan (`compile.rs:load_workflow_document`).

- Les `subworkflow_refs` déclarés mais inutilisés sont seulement enregistrés par `self.record_digest("subworkflows", ...)`; aucun `check_pin` n’est effectué tant qu’aucun node ne les développe (`compile.rs:expand_workflow`).

- Les joins ne sont pas complètement normalisés : le `WorkflowNode::Join` embarqué conserve ses `source_node_ids` template. La review interne l’admet explicitement : « restent des ids-template […] résolution per-scope = Phase J » (`.planning/active/rounds/sprint83_phase_h_review_r1.md:D-3b`), alors que le preflight demande les routes/joins normalisés.

- La phrase documentaire « every collection is a `BTreeMap` or an explicitly sorted `Vec` » est fausse littéralement : `entrypoints` est collecté sans tri et les vecteurs de `capability_closure` sont alimentés par `push` sans tri final (`compile.rs:expand_workflow`). Le résultat reste déterministe avec les parcours BTree actuels, mais la justification annoncée est inexacte.

### 2. Les 20 classes de refus — GAP

| # | Verdict | Confrontation code/test |
|---|---|---|
| 1 | CONFIRME | `validate_reachability` refuse chaque node non atteint; test variant-précis `static_validation_rejects_unreachable_node`. |
| 2 | PARTIEL | Tout cycle est effectivement refusé par Kahn, mais cette classe partage `CycleDetected` et le même test avec #16. |
| 3 | GAP | Aucun variant dédié `UnboundedMap` : le commentaire dit que `InvalidDocument` « covers […] unbounded map », et le test attend exactement `CompileError::InvalidDocument` (`compile.rs:CompileError`, `static_validation_rejects_unbounded_map`). |
| 4 | GAP | Le cas simple branches exclusives est testé, mais `check_join_policy` n’est pas correct pour les topologies de joins imbriquées, voir ci-dessous. |
| 5 | GAP | `UnpinnedReference` existe, mais le contenu du document n’est pas hashé, le cache agent ignore certains pins et les sous-workflows déclarés mais inutilisés ne sont pas vérifiés. |
| 6 | CONFIRME | Binding bidirectionnel output-node↔`artifact_outputs`, doublons et schémas divergents couverts par `validate_artifact_outputs` et trois bras rouges. |
| 7 | CONFIRME | Manifeste pack et requête workflow contrôlés séparément par `CapabilityNotInManifest` et `CapabilityNotRequested`. |
| 8 | PARTIEL | `check_model_policy` ne lit que le préfixe du `model_id`. La moitié « sortie structurée » n’est ni décidée ni testée; le commentaire la délègue aux adapters N/O. |
| 9 | GAP | Seuls les `AgentInvoke` **directement** entrants sont contrôlés : `for edge ... e.to == node_id` puis `if let Some(WorkflowNode::AgentInvoke(giver))`. Un transform intermédiaire, ou aucun giver agent, laisse passer le handoff sans allowlist (`compile.rs:expand_workflow`, bras `Handoff`). |
| 10 | PARTIEL | Variant et test existent, mais seulement pour les formes reconnues par `looks_like_materialised_secret`; le caractère best-effort est documenté. |
| 11 | PARTIEL | Le code reconnaît uniquement `://`, `#!` et `$(` (`compile.rs:scan_document_strings`). Cela n’implémente pas littéralement toute la classe « endpoint ou code exécutable ». |
| 12 | PARTIEL | Les quatre dimensions sont correctement parcourues dans `budget_dimensions`, avec comparaison inclusive `if child_v > parent_v`. Les bras rouges couvrent tokens, wall-clock et turns, mais aucun refus ne tue l’omission de `max_cost_micro_usd`. |
| 13 | GAP | Seul le HITL nominal est réellement contrôlé. `synth_input` met `evaluation: blake3_hex(b"ev")` sans artefact d’évaluation lié, et `publishing(true)` compile néanmoins (`compile.rs:synth_input`, `static_validation_rejects_publication_without_gate`). Provenance/évaluation ne sont que présentes par forme. |
| 14 | CONFIRME | `DanglingEdge` et `DuplicateEdge`, avec contrôle vert et deux bras discriminants. |
| 15 | GAP | La bijection des ports sortants est bien appliquée, mais la satisfiabilité générale des joins ne l’est pas. |
| 16 | PARTIEL | Tout cycle arbitraire est fonctionnellement refusé, mais aucune variante ni aucun test distinct de #2, contrairement à l’exigence explicite du preflight §9. |
| 17 | CONFIRME | Node IDs et clés `field_map` contrôlés contre `PRESENTATION_KEYS`, deux bras rouges. |
| 18 | CONFIRME | Listes vides, doublons et collision `default` correctement refusés. |
| 19 | GAP | La traversée testée fonctionne, mais il reste des chemins de blanchiment réels et des bypasses inter-scope. |
| 20 | PARTIEL | Même schema ID/digests différents est refusé. Toutefois `validate_port_types` s’exécute avant le chargement des agents, et le cache `load_agent` permet d’ignorer un second pin du même agent. |

Le claim « une variante dédiée et un test discriminant pour chacune des 20 classes » est donc réfuté par les lignes exactes :

- `InvalidDocument` est explicitement utilisé pour le `map` non borné.
- Un seul `CycleDetected` représente #2 et #16.
- Un seul `static_validation_rejects_cycle` porte les deux.
- Les dimensions structurées-output, provenance/évaluation et plusieurs cas de pinning ne disposent d’aucun bras rouge matériel.

L’ordre des validations est bien : manifeste → structure → ports/joins → reachability → DAG → outputs → types → HITL → classification → schémas/agents/nodes (`compile.rs:expand_workflow`). Il entraîne notamment deux problèmes :

- `validate_port_types(graph)` reçoit `agents_parsed` avant le chargement des `agent_refs`; les schémas des `AgentInvoke` sont donc absents lors de la passe edge-level.
- Les tests provider, handoff et budget mutent les documents sans repinner les bytes. Dans un compilateur liant réellement document et digest, ils seraient interceptés comme `UnpinnedReference`; ils atteignent le variant attendu uniquement parce que ce binding manque.

Le join présente un faux pass concret. `reachable_from_entrypoints` traverse un join dès qu’un seul feeder est atteignable, via la ligne inconditionnelle `queue.push(edge.to.as_str());`; il ne tient pas compte de `JoinPolicy::All`. Avec deux choosers indépendants X/Y, quatre joins `all` représentant X0Y0, X0Y1, X1Y0 et X1Y1, puis un join final `quorum(2)`, un seul join combinatoire peut réellement ouvrir. Pourtant, retirer X0 laisse chaque join « atteignable » par son feeder Y, donc aucune domination n’est détectée et `quorum(2)` est accepté. Les tests actuels ne couvrent que la séparation directe derrière un chooser (`compile.rs:check_join_policy`, `compile.rs:reachable_from_entrypoints`).

La provenance connaît deux bypasses supplémentaires :

- `WorkflowNode::CapabilityCall(_) | WorkflowNode::KnowledgeQuery(_) => false` blanchit sans justification un champ provenant d’un modèle (`compile.rs:field_taints_from_model`). Un chemin modèle → capability/knowledge → switch(`visibility`) → effet passe.
- `effect_depends_on` ne parcourt que `graph.nodes` et `graph.edges` du scope courant. Un switch de classification parent routant vers un sous-workflow contenant l’effet, ou un switch enfant commandant un effet parent, échappe à la règle (`compile.rs:effect_depends_on`).

### 3. Anti-vacuité — PARTIEL

Les quatre branches demandées de `field_taints_from_model` sont effectivement exercées avec contrôles verts et bras rouges : rename, champ non mappé, sortie `map` opaque et join clean+model (`compile.rs:static_validation_taint_traversal_branches`).

Les bornes inclusives sont également confirmées :

- `depth > MAX_PLAN_DEPTH`, avec exact compile et `+1` refusé.
- `child_v > parent_v`, avec budget exactement égal accepté (`compile.rs:static_validation_rejects_plan_too_deep`, `budget_equality_is_within_bounds`).

En revanche, `plan_then_execute_invariant` n’établit pas les neuf garanties annoncées :

- Le bras « classification » modifie les `decision_labels` de `/po-ratification`, ligne exacte `"classification (human decision labels)"`. Ce sont des labels approve/reject de HITL, pas un discriminant ou label de classification; la famille HITL est donc testée deux fois tandis que la classification réelle ne l’est pas.

- Le bras publisher ne modifie que `pack_name`. Ni la clé/signature du publisher ni `pack.provenance` ne sont dans `CompiledPlan`; deux publishers signant un même lock avec les mêmes nom/révision donnent le même plan.

- Les neuf mutations sont appliquées **après compilation** sur une `serde_json::Value`. Elles prouvent qu’un hash change quand ses bytes changent et que les champs existent, pas que le compilateur dérive fidèlement chaque famille depuis les artefacts pinnés.

- Le seul bras pack-sensibilité est `bumped.pack.pack_revision = 2`. La phrase « ANY pack change moves the digest » est fausse : modifier `description`, `provenance` ou plusieurs claims de `pack.digests` ne change pas le plan.

- Surtout, le compilateur est pur de `CompileInput`, pas du pack seul : la signature est `pub fn compile(input: &CompileInput)`. Le même `input.pack` avec une autre `root_workflow.document` et le même pin peut produire un autre plan.

### 4. Posture frontière — CONFIRME

La posture Phase H est respectée dans le diff fourni :

- `CompiledPlan` est déplacé dans `compile.rs` puis ré-exporté par `pub use crate::compile::CompiledPlan;` (`contracts.rs:78-91`).
- `pub mod compile;` est ajouté, tandis que `golden_store` reste `#[cfg(test)]` (`lib.rs:106-130`).
- Aucun nouveau `DOMAIN_*`, `FORMAT_VERSION`, producteur signé ou appel `canonical_bytes(DOMAIN_COMPILED_PLAN_V1)` n’apparaît.
- Aucun tag machine `// FRONTIER:` n’est attaché au type; la mention backticked dans la prose n’est pas un tag.
- La NAME LIST n’est pas changée; seule la nuance « identité remplie / signing tag toujours réservé » est ajoutée (`scripts/check-frontier-contracts.sh:209-215`).
- `contracts.schema.json` est bien modifié dans le même diff (+1130/−2), et le test strict de snapshot ainsi que le registre `len == 7` sont couverts par le run 97/97 fourni.

### 5. Quatre compiled goldens — CONFIRME

La cohérence avec le compilateur actuel est démontrée par un test robuste : chaque golden est recompilé, sérialisé en `serde_json::Value`, relu puis comparé par `assert_eq!(value, committed)` (`compile.rs:golden_compiled_plans_stable`). Le 97/97 fourni implique que les quatre fichiers complets correspondent au code courant.

Le factory visible est cohérent :

- occurrences RFC 6901 `/codex-rounds/codex-verify`;
- `process.git_commit -> ["/commit-phase"]`;
- `process.run_tests -> ["/run-suites"]`;
- `human_gates == ["/po-ratification"]`;
- root et enfant présents dans `entrypoints`;
- budgets et politiques par scope;
- `four_goldens_compile` compare `plan.pack_digest` à `content_digest_of_lock(&input.pack.lock)`.

Les trois autres fichiers ne sont pas inspectables manuellement au-delà de leurs extraits, mais le test complet, comparé en `Value` et annoncé vert après les corrections, ferme le risque de fixtures générées avant un fix de production.

### 6. `THREAT_MODEL.md` — GAP

La structure demandée est présente : nouvelle §18, asset A10, cinq `T-PLAN-*`, ancienne revue renumérotée §19 et historique v22 (`docs/security/THREAT_MODEL.md:1699-1744`, `2041+`). Les mentions « moitié statique », version compilateur fixe, limites nominales et scan `://`-only sont bien écrites.

Mais plusieurs sur-affirmations restent :

- Claim réfuté : « `compile()` est une fonction PURE du pack ». La signature réelle est `compile(input: &CompileInput)` et les documents sont des entrées indépendantes non hashées par le compilateur (`THREAT_MODEL.md:§18/T-PLAN-LLM-MUTATION`; `compile.rs:compile`, `ProvidedDocument`).

- Claim réfuté : « 20 classes […] une variante `CompileError` + un test discriminant chacune ». `map` utilise `InvalidDocument`, et #2/#16 partagent `CycleDetected` et un test (`THREAT_MODEL.md:§18/T-PLAN-STATIC-BYPASS`; `compile.rs:CompileError`).

- Claim réfuté : `T-PLAN-CLASSIFICATION-LAUNDER` « fermée (nominal v1) ». Les caveats ne couvrent que synonymes et marqueurs, pas le blanchiment par `CapabilityCall`/`KnowledgeQuery` ni le bypass aux frontières de sous-workflow (`THREAT_MODEL.md:§18/T-PLAN-CLASSIFICATION-LAUNDER`; `compile.rs:field_taints_from_model`, `effect_depends_on`).

- Claim réfuté : « UN digest par `schema_id` à l’échelle du plan » est vrai pour les schémas ordinaires, mais peut être contourné par le retour anticipé du cache agent. Plus largement, les autres catégories d’artefacts écrasent silencieusement les IDs homonymes (`THREAT_MODEL.md:§18/T-PLAN-DIGEST-MALLEABILITY`; `compile.rs:load_agent`, `record_digest`).

- « le graphe signé EST le plan […] enforceable pour la première fois » est prématuré : la fin de la même section reconnaît que la réconciliation `pack.digests.compiled_plan`↔artefact est ultérieure, et le compilateur ne vérifie pas document↔pin.

- La documentation de `CompiledNodeInstance` affirme que le plan est self-contained et que l’engine n’exécute jamais depuis les documents source. Pourtant le golden factory montre qu’un `agent.invoke` compilé ne contient que `agent_id` et `budget`; model policy, instructions, tools et guardrails restent dans l’agent externe (`factory.compiled.golden.json:node_instances./drive-implementation`).

Le caveat T-PLAN-SCAN-EVASION est, lui, honnête et précisément borné.

### 7. Refactor test-support — CONFIRME

Le diff `graph.rs` ne touche que `#[cfg(test)] mod tests` à partir de `graph.rs:663`. Les helpers retirés — constantes, chemins, `ArtifactStore`, parseurs et `write_pretty` — sont reproduits dans `golden_store.rs`; aucune assertion ni aucun struct de production n’est supprimé.

`pack.rs` contient exactement deux changements fonctionnels, tous deux de visibilité :

- `looks_like_materialised_secret` → `pub(crate)` (`pack.rs:1039+`);
- `is_lower_hex_digest` → `pub(crate)` (`pack.rs:1089+`).

### 8. Interdits du preflight — CONFIRME

Dans le bundle fourni :

- aucun ajout `thiserror`, `anyhow`, `indexmap`, `petgraph` ou crate `jsonschema`;
- aucun changement `Cargo.toml`/`Cargo.lock`, donc aucune dev-dep;
- aucun champ graph/agent ajouté : le diff `graph.rs` est test-only et `agent_definition.rs` est absent du diff;
- aucun changement engine/web/operator;
- comparaison des compiled goldens en `serde_json::Value`;
- le `serde_json::to_string` restant appartient uniquement au test intra-profil de déterminisme, pas aux goldens ni à un digest;
- tous les digests de H passent par `digest_for`;
- aucun appel à `canonical_bytes(DOMAIN_COMPILED_PLAN_V1)`;
- aucun nouveau domaine, format wire ou tag frontier machine.