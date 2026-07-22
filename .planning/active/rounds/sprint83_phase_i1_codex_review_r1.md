Verdict global : PARTIEL. L’implémentation, le câblage de sécurité, les tests et les invariants techniques sont présents. Deux écarts documentaires empêchent une confirmation totale : une phrase périmée dans `THREAT_MODEL.md` et un exemple JSON non conforme dans `REFERENCE.md`. Aucun `cargo`/`npm` n’a été lancé ; le contrôle frontier, lui, passe avec census 29.

### Livrable 1 : module `workflow_api.rs`

- Statut : CONFIRME
- Fichier(s) : [workflow_api.rs:62-68, 72-114, 127-212](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:62)
- Evidence : garde ASCII bornée effective ; DTOs limités aux champs autorisés ; miss transformé en `found:false` puis HTTP 200 ; `StoreError` transformé en 500 aux lignes 168-174. Aucun type `RunEvent` n’est importé.

```rust
!id.is_empty()
    && id.len() <= MAX_RUN_ID_LEN
    && id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
```

Les seules clés du run sont `run_id`, `status`, `terminal_reason`, `event_count`, `frontier_node`, `spec_digest` aux lignes 72-85. Les handlers renvoient 400 aux lignes 188-194, 200 aux lignes 197/210 et 500 via `store_error_response`.

### Livrable 2 : câblage, authentification et boot

- Statut : CONFIRME
- Fichier(s) : [Cargo.toml:33-38](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/Cargo.toml:33), [main.rs:77](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:77), [http.rs:203-212, 386-399, 646-652](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:203), [runtime.rs:742-755, 823](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:742)
- Evidence : dépendance path sans feature ; champ partagé ; deux routes ajoutées directement à `authed_routes`, avant son layer d’authentification ligne 646 ; ouverture eager unique sous `opts.paths.root`.

```rust
let run_store_path = opts.paths.root.join(crate::workflow_api::RUN_STORE_FILE);
let run_store = sbfb_workflow_engine::RunStore::open(&run_store_path)
    .map_err(|e| anyhow::anyhow!("workflow run store open failed: {e}"))?;
let run_store = Arc::new(std::sync::Mutex::new(run_store));
```

`public_routes` ne contient que `/health` et `/blob-serve` aux lignes 272-275. Le seul `acquire_lease` trouvé dans le crate daemon est à `workflow_api.rs:368`, sous `#[cfg(test)]`; aucun chemin de production ne l’appelle.

### Livrable 3 : compte des routes

- Statut : CONFIRME
- Fichier(s) : [http.rs:392-399](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:392)
- Evidence : décompte direct du même motif `.route(` : HEAD = 89, working tree = 91.

```rust
.route(
    "/api/daemon/workflow/run/{run_id}",
    get(crate::workflow_api::workflow_run_status),
)
.route(
```

La seconde route est enregistrée aux lignes 396-399. Aucun routeur imbriqué n’est utilisé pour cette famille.

### Livrable 4 : invariants durs

- Statut : CONFIRME
- Fichier(s) : [Cargo.lock:5194-5199](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:5194), [sbfb-workflow-engine/Cargo.toml:32-43](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/Cargo.toml:32), [check-frontier-contracts.sh:202-257](C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202)
- Evidence : `Cargo.lock` présente exactement une insertion, vers un package workspace déjà existant.

```text
"rand 0.8.6",
"reqwest 0.12.28",
"sbfb-manifest",
"sbfb-workflow-engine",
"serde",
```

Constats directs :

- `git diff -- crates/sbfb-workflow-engine crates/sbfb-workflow-core` : vide.
- `git diff --numstat -- Cargo.lock` : `1 0`.
- `check-frontier-contracts.sh` : exit 0, `DOMAIN census [29 frozen]`.
- Aucun nouveau `DOMAIN_*`, `FORMAT_VERSION`, `WIRE_VERSION` ou `schema_version`.
- Aucun `// FRONTIER:` sur les routes workflow.
- Aucune modification sous `web/` ou `docs/protocol/`.

### Livrable 5 : tests et non-régressions transverses

- Statut : CONFIRME
- Fichier(s) : [workflow_api.rs:215-460](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:215), [test_support.rs:145-209, 588-687, 721-764](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:145), [curators_api.rs:497-560](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:497)
- Evidence : neuf attributs de test exactement. Les trois états sont exercés, les clés sont comparées comme ensembles, les réponses HTTP sont vérifiées, et le test duress seede un vrai run.

```rust
assert_eq!(json["found"], true);
assert_eq!(json["run"]["status"], "completed");
assert_eq!(
    json["run"]["terminal_reason"],
    serde_json::Value::Null,
```

La famille golden contient bien trois cas aux lignes 725-763. Les deux sondes de stabilité sont aux lignes 632-644 et la comparaison run1/run2 est effective aux lignes 676-685. Le runner vérifie statut, en-têtes et corps aux lignes 316-338.

Le grep transversal trouve trois constructeurs littéraux de `DaemonHttpState` seulement : production, `test_support`, et le test curators. Les trois renseignent `run_store`. Les stores de test sont file-backed dans des tempdirs aux lignes `workflow_api.rs:226-229`, `test_support.rs:137-156` et `curators_api.rs:489-504`.

### Livrable 6 : inventaire D7

- Statut : CONFIRME
- Fichier(s) : [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3-5, 63-105](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3)
- Evidence : la section 3 contient exactement 33 rows. Le calcul documenté est cohérent : 33 − 1 doublon `consent/set` − 4 endpoints futurs = 28 paths actuels représentés.

```text
| GET .../workflow/run/{run_id} | S83 Phase I1 | T0 | T0 | ...
| POST .../run/{run_id}/restore | endpoint FUTUR, daté J/K | N/A | T1 | ...
| POST .../workflow/gc | endpoint FUTUR, daté J/K | N/A | T1 | ...
| POST .../workflow/lease/force | endpoint FUTUR, daté J/K | N/A | T1 | ...
```

Le front-matter indique bien `last_validated: 2026-07-22`, `33 rows = 28 route-paths` et `91 routes`. La row T0 précise explicitement qu’elle couvre aussi `GET /api/daemon/workflow/lease`. Les trois mutations sont absentes du routeur.

### Livrable 7 : `THREAT_MODEL.md` v24

- Statut : PARTIEL
- Fichier(s) : [THREAT_MODEL.md:1766-1785, 2130-2152](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1766)
- Evidence : `T-RUN-DOUBLE-EMIT` est correctement amendée ligne 1766 ; `T-RUN-ID-PII` décrit la garde livrée ligne 1771 ; les bornes et le changelog v24 décrivent les deux GET et le report J/K.

```text
**Bornes honnetes (ce que I ne ferme PAS)** : exposition loopback des
operations du store = **Phase I1 LIVREE en LECTURE SEULE** (v24 : 2 GET
`workflow_api.rs` — agregat par-run + row lease brute — inventaire D7 §3
LOOPBACK + famille golden_http ; la famille MUTANTE restore/GC/lease-force
est tier-classee T1 dans l'inventaire avec implementation DATEE J/K
```

- Écart : la row vivante `T-RUN-RESTORE-ROLLBACK`, ligne 1767, dit encore : « l’exposition loopback éventuelle = Phase I1 ». Cela contredit le code, les bornes lignes 1773-1780 et le changelog v24, qui datent la famille mutante J/K.

### Livrable 8 : clôture docs-contrat workflow

- Statut : PARTIEL
- Fichier(s) : [docs/workflow/llms.txt:20-62](C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:20), [REFERENCE.md:19-88](C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:19), [llms.txt:8-12, 28-32](C:/Users/FlowUP/Documents/Code/nexus/llms.txt:8)
- Evidence : les deux nouveaux fichiers existent ; tous leurs liens locaux résolvent ; les symboles cités existent (`valid_run_id`, `workflow_run_response`, `workflow_lease_response`, `lease_state`, les deux tests golden). La racine indexe bien le workflow comme troisième sous-système.

```json
"found": true,
"run": {
  "run_id": "run-a",
  "status": "running | completed | terminal",
  "terminal_reason": "step_hash_mismatch | outcome_unknown | null",
```

- Écart : cet exemple, lignes 37-48, n’est pas conforme au DTO. `"running | completed | terminal"` n’est aucune des trois valeurs sérialisées, et `null` est inclus dans une chaîne au lieu d’être un JSON `null`. L’exemple lease des lignes 64-68 est, lui, conforme.

### Livrable 9 : artefacts process

- Statut : CONFIRME
- Fichier(s) : [preflight:13, 90-161](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_i1_preflight.md:13), [review r1:9-60](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i1_review_r1.md:9), [review r2:5-35](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i1_review_r2.md:5)
- Evidence : le preflight porte `PLAN-ADAPT` et contient exactement A1 à A13. La r1 porte `CONCERN` et documente ses corrections ; la r2 porte le verdict final demandé.

```text
## Verdict r2 : PASS-PENDING
1. **REVIEW-1 FERMÉ** — test neuf `run_response_projects_completed`
2. **REVIEW-2 FERMÉ** — seed réel sous boot Duress
3. **REVIEW-3 FERMÉ** — 33 rows = 28 route-paths
```

## Résumé final

- Total livrables : 9
- Confirmés : 7
- Gaps : 0
- Partiels : 2