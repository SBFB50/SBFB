# Codex review — Sprint 83 Phase G (round 2)

## Verdicts par livrable

État vérifié sur disque : HEAD `23fdc1dc4c6e5b60db1626a8d1da1b9c43549ae8`; arbre non committé. Aucun fichier modifié pendant cette revue.

### GAP1 — CONFIRME (corrigé)

La publication Ideas dépend désormais d’un véritable `human.review`, avec les deux décisions statiques `publish` et `keep-private` ([ideas.workflow.golden.json:91](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:91)). Les seules sorties sont correctement câblées vers `store-idea` et `private-note` ([ideas.workflow.golden.json:293](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:293), [ideas.workflow.golden.json:299](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:299), [ideas.workflow.golden.json:305](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:305)). `store-idea` reste bien le `capability.call storage.set` ([ideas.workflow.golden.json:137](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:137)).

Le grep récursif actuel donne zéro occurrence de `visibility` et de l’ancien identifiant `publish-gate` dans le golden et le store Ideas. Les cinq schémas corrigés sont fermés et n’exposent que les champs annoncés : [idea-submission.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/schemas/idea-submission.schema.json:5), [idea-note.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/schemas/idea-note.schema.json:5), [champion-verdict.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/schemas/champion-verdict.schema.json:5), [refine-input.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/schemas/refine-input.schema.json:5), [refine-report.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/schemas/refine-report.schema.json:5). Le prompt interdit explicitement au champion de décider la publication ([champion-instructions.md:3](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/ideas/prompts/champion-instructions.md:3)).

Le test contrôle le type humain, un chemin positif vers l’effet et la domination par exclusion BFS ([graph.rs:1503](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1503)). Une edge de bypass rendrait bien l’assertion rouge. Les branches `bug` et `vision` restent câblées vers `repro-check` et `vision-note` ([ideas.workflow.golden.json:227](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:227), [ideas.workflow.golden.json:233](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/ideas.workflow.golden.json:233)) puis convergent vers le gate.

### GAP2 — CONFIRME (corrigé)

`angle-list.angles.items` est l’objet fermé `{angle: string}` ([angle-list.schema.json:6](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/angle-list.schema.json:6)), structurellement identique à `design-angle` ([design-angle.schema.json:4](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/design-angle.schema.json:4)).

Le graphe relie bien la sortie `angle-list` de `expand` au map utilisant le subworkflow `conception-variant` ([conception.workflow.golden.json:88](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/conception.workflow.golden.json:88), [conception.workflow.golden.json:178](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/conception.workflow.golden.json:178)), dont l’entrée est `design-angle` ([conception-variant.workflow.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/subworkflows/conception-variant.workflow.json:5)). `shape_of` compare réellement `type`, le sous-arbre complet `properties`, `required` et `additionalProperties` ([graph.rs:1541](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1541)); le test correspondant est discriminant ([graph.rs:1554](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1554)).

### GAP3 — CONFIRME (corrigé)

`judge-input` attend désormais `variants: [{proposal, angle}]` et `checks: {docs_ok, feasible}` ([judge-input.schema.json:6](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/judge-input.schema.json:6), [judge-input.schema.json:25](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/judge-input.schema.json:25)). Ces shapes correspondent exactement à [variant.schema.json:4](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/variant.schema.json:4) et [checks-report.schema.json:4](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/schemas/checks-report.schema.json:4).

Le body du map produit bien `variant` ([conception-variant.workflow.json:9](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/subworkflows/conception-variant.workflow.json:9)), les checks produisent `checks-report` ([conception.workflow.golden.json:49](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/conception.workflow.golden.json:49), [conception.workflow.golden.json:114](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/conception.workflow.golden.json:114)), et l’agent judge référence `judge-input` ([conception-judge.agent.json:10](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/workflow_golden_artifacts/conception/agents/conception-judge.agent.json:10)). Les deux comparaisons structurelles du test sont donc reliées aux artefacts réellement utilisés, et non à des shapes inutilisées ([graph.rs:1571](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1571)).

### GAP4 — CONFIRME (corrigé)

Les trois tests sont non tautologiques :

- bypass du gate ou node non humain → échec de l’assertion de domination/type ;
- `angle-list.items → string` → échec de `is_object()` puis de `shape_of` ;
- `judge-input.checks → string` → mismatch avec `checks-report`.

Les trois tests ont été rejoués sur le disque actuel et passent. La vérification `visibility` du test est limitée aux propriétés racine, mais cela ne laisse aucun résidu actuel : le grep récursif est vide et les schémas concernés sont fermés. Cette limite de généralisation future n’annule pas le pouvoir discriminant demandé pour la Phase G.

### Régressions transverses — CONFIRME

- Les 10 graphes top/subworkflow ont `0` endpoint invalide et `0` node inaccessible.
- Les stores comptent toujours `84` fichiers pour `84` références uniques : `0` manquant, `0` orphelin.
- Les 18 kinds gelés restent exercés, conformément au SET-check ([graph.rs:1230](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1230)). Babel conserve le `switch` ([babel.workflow.golden.json:79](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:79)).
- Chaque golden conserve au moins un handoff autorisé, un repeat borné et un capability call ([graph.rs:1236](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:1236)).
- `four_goldens_validate` parcourt les quatre goldens et leurs agents/subworkflows ([graph.rs:881](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:881)); `golden_pins_resolve` recalcule chaque référence depuis les octets actuels et compare au pin enregistré ([graph.rs:924](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:924)). Les deux tests passent sur le disque actuel.

### Freeze règle 6 — CONFIRME (tenu)

La règle est correctement consignée comme futur refus Phase H, et non comme enforcement Phase G ([sprint83_node_profile_freeze.md:127](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_node_profile_freeze.md:127)). Elle vise explicitement les discriminants de classification public/privé ou de confidentialité issus d’un modèle ([sprint83_node_profile_freeze.md:145](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_node_profile_freeze.md:145)), conformément à l’interdit §19.3 ([workflow_agents_app_conception_ultradeep_2026-07-15.md:1597](/C:/Users/FlowUP/Documents/Code/nexus/.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md:1597)).

Babel ne constitue pas un faux positif : `quality` est un critère de qualité, et les branches `ship`, `polish` et `default` convergent toutes vers `await-community` ([babel.workflow.golden.json:145](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:145), [babel.workflow.golden.json:151](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:151), [babel.workflow.golden.json:169](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:169)). Le `wait.signal` communautaire domine ensuite `network.publish` ([babel.workflow.golden.json:17](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:17), [babel.workflow.golden.json:175](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/babel.workflow.golden.json:175)).

### Invariants a–f

- **(a) CONFIRME** — census gelé `29`, live `29`, `0` manque, `0` excédent. La liste normative et son calcul sont à [check-frontier-contracts.sh:202](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202) et [check-frontier-contracts.sh:247](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:247). Aucun nouveau domaine, format ou tag machine Phase G.
- **(b) CONFIRME** — `git diff -- Cargo.toml Cargo.lock` vide et aucun manifeste untracked.
- **(c) CONFIRME** — `git diff -- web tools/factory-operator` vide et aucun fichier untracked sous ces deux surfaces.
- **(d) CONFIRME** — sept insertions exactes dans le registre ([contracts.rs:101](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:101)); `RunEvent`, `CompiledPlan` et `WorkflowPublicProjection` restent des structs vides ([contracts.rs:70](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:70)).
- **(e) CONFIRME** — aucun compilateur ou validateur de production introduit ; le module déclare explicitement cette frontière Phase H ([graph.rs:65](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/graph.rs:65)).
- **(f) CONFIRME** — aucun `Option` ni `serde(default)` effectif. Le snapshot ferme et requiert les 16/16 champs Graph ([contracts.schema.json:308](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:308)) et les 15/15 champs Agent ([contracts.schema.json:1339](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1339)). Le balayage récursif donne également `0` mismatch `properties`/`required` sur les 35 shapes objet Graph et les 8 shapes Agent.

## GAPs

## Verdict global

**CONFIRME — 0 gap.** Les quatre GAPs du round 1 sont corrigés, la règle 6 est correctement bornée, et aucune régression ni dérive des invariants a–f n’a été trouvée.