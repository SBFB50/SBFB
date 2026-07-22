Verdict global : pas encore de fermeture finale sur ces seuls extraits. Trois points sont fermés ; deux restent PARTIELS à cause de tests qui ne prouvent pas entièrement l’invariant annoncé. Les 106/106 et les gates vertes confirment la santé de la suite, pas le caractère discriminant de ces tests.

1. **CONFIRME-FERME — autorité du workflow racine.**  
   `CompileInput::root_workflow` statue explicitement que le choix appartient à l’hôte lanceur, puisque F ne désigne aucune racine. Le triplet `root_workflow_id` + digest exact dans `artefact_digests.workflows` + `pack_digest` rend le choix traçable sans inventer une autorité du signataire absente du format. La limite est correctement formulée : auditabilité du choix, pas désignation cryptographique d’une racine.

2. **CONFIRME-FERME — `knowledge.query` ne blanchit plus.**  
   `KnowledgeQuery` rejoint `CapabilityCall` dans la branche récursive et conserve donc la provenance du champ. Le test `model.invoke → knowledge.query → switch` est discriminant : l’ancien blanchiment aurait accepté le graphe, tandis que le code corrigé doit produire `ClassificationFromModel` sur `sw`.

3. **PARTIEL — contrat du donneur `agent.as_tool`.**  
   Le code actuel est conforme : seul `AgentInvoke` est collecté puis arrête le parcours ; `AgentAsTool` traverse la branche générique. En revanche, le test fourni ne contient aucun `agent.invoke`. Il prouve seulement que le spécialiste `agent.as_tool` n’est pas donneur ; il ne prouve ni la traversée jusqu’au parent orchestrateur, ni l’arrêt au premier `agent.invoke`.  
   Fermeture attendue : un graphe `agent.invoke(parent) → agent.as_tool(worker) → handoff`, avec succès lorsque le parent autorise la cible et refus lorsqu’il ne l’autorise pas.

4. **PARTIEL — bras rouge “sous-workflow = effet opaque”.**  
   Le test laisse `deeper → do → done`, et le type de `do` n’est pas redéfini dans l’extrait. S’il s’agit du `capability.call` habituel de `mini_graph`, le test peut réussir simplement grâce à cet effet aval, même si `Subworkflow` cesse d’être lui-même considéré comme opaque/effectful. Le corps de `effect_depends_on` n’est par ailleurs pas montré.  
   Il faut un cas isolé `switch → subworkflow → terminal`, sans autre effet aval dans le graphe parent, qui produise toujours `ClassificationFromModel`.

5. **CONFIRME-FERME — formulations “ANY pack change”.**  
   Le commentaire limite désormais correctement la sensibilité aux champs projetés dans le plan et distingue les champs non projetés, protégés par la signature du pack. Le message d’assertion annoncé — « a plan-projected pack change » — est cohérent, et l’extrait du `THREAT_MODEL` ne réintroduit pas la généralisation erronée.

Conclusion : aucun nouveau GAP de conception démontré, mais S83 reste non fermé formellement tant que les preuves de régression des points 3 et 4 ne sont pas rendues discriminantes.