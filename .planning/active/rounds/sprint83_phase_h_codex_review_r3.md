Le verdict global reste **GAP**. La majorité des corrections R2 sont réellement présentes et cohérentes, mais quatre chemins ne sont pas fermés par l’arbre fourni : le rôle du workflow racine n’est pas lié à une référence signée précise, `knowledge.query` blanchit encore une influence du modèle, le handoff transitif ignore `agent.as-tool`, et la claim « tout changement du pack change le plan » subsiste malgré la réconciliation annoncée. Les suites vertes ne couvrent aucun de ces contre-exemples.

1. Binding document ↔ digest — **PARTIEL.**  
   `ProvidedDocument` ne contient plus qu’un `Vec<u8>`, `pin()` calcule réellement BLAKE3, et `bind_provided` impose appartenance au lock puis égalité avec `expected_pin`. C’est fermé pour les agents et sous-workflows, chargés avec `Some(ref.blake3)`.  
   Mais la racine est chargée par `load_workflow_document(..., None)`. Comme `lock_digests` ne conserve que l’ensemble des digests, sans chemin ni rôle, n’importe quel workflow JSON présent dans le lock peut être fourni comme racine. Un même pack signé peut donc produire plusieurs racines autoritaires selon le choix du caller. Il faut lier la racine à une référence ou un chemin signé déterminé, pas seulement à « un digest quelconque du lock ».

2. Cache agents court-circuitant le binding — **CONFIRME-FERME.**  
   Le cache contient `(digest, SbfbAgentDefinition)` et chaque hit compare `cached_digest` à `agent_ref.blake3` avant de retourner le document. Le test cross-scope tue bien l’ancien comportement first-wins.

3. Écrasement silencieux dans `record_digest` — **CONFIRME-FERME.**  
   `record_digest` retourne désormais un `Result`, refuse un même identifiant associé à deux digests et tous les call sites propagent `?`. L’insertion préalable dans `semantic_views` ne crée pas de sortie corrompue : le conflit suivant termine la compilation.

4. `subworkflow_refs` déclarés mais non développés — **CONFIRME-FERME.**  
   Chaque référence déclarée passe maintenant par `check_pin` puis `record_digest`, indépendamment de son expansion. La correction ferme exactement le GAP R2. Les bytes d’un enfant inutilisé ne sont toutefois pas exigés ; seule sa présence cryptographique dans le lock l’est.

5. Agents chargés après le contrôle des ports — **CONFIRME-FERME.**  
   Tous les `agent_refs` sont chargés avant `validate_port_types`. `declared_input_schema` et `declared_output_schema` voient donc les schémas des agents. Le test agent-output → transform est discriminant et atteint bien le pass edge-level.

6. Handoff limité aux donneurs directs — **PARTIEL.**  
   La remontée transitive à travers un `transform`, join ou autre node non-agent est effective, et le test fourni ferme le contournement R2 pour `agent.invoke`.  
   En revanche, `upstream_agent_invokers` ne reconnaît que `WorkflowNode::AgentInvoke`. Un chemin `agent.as-tool(worker) → transform → handoff(receiver)` ne considère jamais `worker` comme donneur et n’examine donc pas son `allowed_handoffs`. La documentation dit que la marche « stoppe à l’agent », mais le code ne stoppe pas sur `AgentAsTool`. Soit cette variante doit être explicitement exclue par le contrat, soit elle doit être contrôlée et testée.

7. `capability.call` blanchissant le taint — **GAP.**  
   La branche exacte R2 est corrigée : `CapabilityCall` recurse vers ses entrées, et le test model → capability → switch est valide.  
   Mais `KnowledgeQuery` retourne encore inconditionnellement `false`. Le raisonnement « corpus pinné donc résultat propre » est incorrect : le résultat d’une recherche dépend de la requête. Le chemin suivant est accepté par le code :

   `model.invoke → knowledge.query → switch(visibility) → capability.call`

   Le modèle peut contrôler la sélection dans le corpus, donc influencer le discriminant. Le corpus pinné fixe les données disponibles, pas le résultat sélectionné. Aucun test fourni ne couvre cette branche.

8. `effect_depends_on` limité au scope courant — **CONFIRME-FERME dans le code.**  
   Un `map`, `repeat` ou `subworkflow` aval est désormais traité comme effet, ce qui ferme le cas précis « switch parent → enfant potentiellement effectful ». Le sur-refus est explicite et fail-closed. Il manque néanmoins un test rouge ciblé : aucun des six nouveaux tests n’empêche une régression retirant ces trois variantes du `matches!`.

9. Famille « classification » du plan testée seulement via HITL — **CONFIRME-FERME.**  
   `plan_pins_switch_discriminants` mute le `discriminant_field` du switch Babel réellement embarqué dans `node_instances` et vérifie le changement de digest. Cela complète effectivement l’ancien test des `decision_labels`.

10. Dimension coût sans bras rouge — **CONFIRME-FERME.**  
    `static_validation_rejects_cost_budget_overflow` cible `max_cost_micro_usd` et exige cette dimension précise dans `BudgetOverflow`.

11. Claims de module et THREAT_MODEL — **PARTIEL.**  
    Les reformulations sur l’entrée résolue, les bytes liés par hash, le caractère « self-contained à résolution de refs près », l’ordre du document et les joins single-level sont nettement plus honnêtes.  
    Deux surclaims persistent toutefois :

    - `plan_then_execute_invariant` dit toujours « ANY pack change moves the digest » puis « a changed pack must change the plan » ;
    - `T-PLAN-CLASSIFICATION-LAUNDER` est déclaré « fermée » alors que `knowledge.query` fournit le contournement ci-dessus.

12. Joins imbriqués — **PARTIEL, honnêtement borné.**  
    Le code reste single-level et peut toujours accepter une composition de joins globalement impossible. La borne est maintenant clairement décrite comme un risque de liveness, avec revalidation des arrivées reportée à J. Cette limitation n’est pas fermée, mais elle n’est plus dissimulée comme une garantie statique complète.

13. « ANY pack change moves the digest » — **GAP.**  
    La correction annoncée n’est pas complètement appliquée. Le commentaire du test et le THREAT_MODEL conservent la claim générale, alors que le test ne mute que `pack_revision`. Une modification de `pack.description`, de `provenance.builder_id` ou d’une claim de digest non consommée produit le même `CompiledPlan` et donc le même `compiled_plan_digest`. La signature du pack peut changer, mais ce n’est pas ce que prétend le test. La formulation doit être limitée aux champs réellement projetés dans le plan.

14. Identité publisher limitée à nom + révision — **PARTIEL, borne externe.**  
    Le plan fixe bien `pack_name`, `pack_revision` et le digest du lock : une sortie de modèle ne peut pas les réécrire après compilation. Mais il ne contient aucune identité de clé signataire, et `CompileInput` accepte un `SbfbPromptAgentPack` brut plutôt qu’un type attestant sa vérification. La provenance authentifiée dépend donc entièrement d’un prérequis externe `verify_signed_pack`, dont le bundle ne permet pas de vérifier le call graph. Ce n’est pas une identité publisher autonome portée par le plan.

15. Provenance et évaluation « par forme seulement » — **PARTIEL, honnêtement borné.**  
    Le compilateur ne consomme ni ne valide sémantiquement la provenance ou l’évaluation ; leur présence vient du type/parse. Le document le dit désormais. Le contrôle HITL est actif, mais les deux autres moitiés restent bien des préconditions de forme.

16. Trois compiled goldens prétendument non inspectables — **CONFIRME-FERME.**  
    `golden_compiled_plans_stable` parcourt tout `GOLDEN_NAMES` et compare les quatre plans comme valeurs structurées. La stabilité byte-à-byte dépend du résultat de suite fourni, car le test montré compare des `Value`, pas les bytes sérialisés.

17. Structured output et scan best-effort — **PARTIEL, honnêtement borné.**  
    Le compilateur vérifie seulement les préfixes provider/modèle ; l’exécution effective du structured output reste N/O. Le scan reste contournable par encodage, fragmentation et schémas sans `://`. Ces limites sont maintenant explicitement documentées et ne sont plus présentées comme fermées.

18. `graph_semantic_digest_order_independent` — **CONFIRME-FERME.**  
    Le test reconnaît correctement que le réordonnancement modifie les bytes, donc le lock et `pack_digest`, tout en exigeant le même digest sémantique et le même plan modulo les deux bindings dérivés des bytes.

La promotion à PASS demande au minimum : lier cryptographiquement le rôle de la racine, faire tainter `knowledge.query` par ses entrées, statuer et tester `AgentAsTool` comme donneur de handoff, ajouter un bras rouge parent-switch → sous-workflow, et supprimer toutes les formulations résiduelles « tout changement du pack ».