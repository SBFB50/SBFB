Verdict global : la Phase I1 est **CONFIRMEE** sur le working tree de `master` à `e48d8280`. Les neuf livrables sont réellement implémentés ; les deux corrections du round 2 sont présentes et cohérentes avec le code. Aucun Cargo/npm n’a été lancé. Le gate frontier a été exécuté séparément et retourne `exit 0`, census `29 frozen`.

### Livrable 1 : handlers GET, DTOs et contrats d’erreur

- Statut : CONFIRME
- Fichier(s) : [workflow_api.rs:57-68](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:57), [workflow_api.rs:70-114](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:70), [workflow_api.rs:127-173](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:127), [workflow_api.rs:183-212](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:183)
- Evidence :

```rust
77:    status: &'static str,
79:    terminal_reason: Option<String>,
80:    event_count: u64,
83:    frontier_node: Option<String>,
84:    spec_digest: Option<String>,
```

Les DTOs n’exposent ni `RunEvent`, ni snapshot, ni receipt/request hash. Le mapping couvre `running/completed/terminal` aux lignes 116-121. `valid_run_id` applique réellement la borne et l’allowlist ASCII aux lignes 62-68 ; le handler répond 400 aux lignes 188-193. Un miss produit `found:false, run:null` aux lignes 131-135 puis HTTP 200 à la ligne 197. Les `StoreError` sont converties en HTTP 500 aux lignes 168-173 et 198/211.

### Livrable 2 : câblage daemon complet et authentifié

- Statut : CONFIRME
- Fichier(s) : [Cargo.toml:33-38](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/Cargo.toml:33), [main.rs:70-77](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:70), [http.rs:203-212](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:203), [http.rs:293-299](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:293), [http.rs:386-399](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:386), [http.rs:646-653](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:646), [runtime.rs:742-755](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:742)
- Evidence :

```rust
392:        .route(
393:            "/api/daemon/workflow/run/{run_id}",
394:            get(crate::workflow_api::workflow_run_status),
396:        .route(
397:            "/api/daemon/workflow/lease",
```

La dépendance est un path workspace sans feature `spike-faults`. Le module est compilé hors `cfg(test)`. Les routes sont inline dans `authed_routes`, puis le middleware `auth_required` est appliqué à la ligne 646 ; elles ne figurent ni dans `public_routes` ni dans un sous-router mergé.

Le store est ouvert une fois via `opts.paths.root.join(RUN_STORE_FILE)` aux lignes 752-755, puis déplacé dans l’état ligne 823. Aucun appel de production à `acquire_lease` n’existe : l’unique occurrence du crate daemon est ligne 368 de `workflow_api.rs`, sous `#[cfg(test)]`, uniquement pour peupler le test de projection lease.

### Livrable 3 : compte des routes 89 → 91

- Statut : CONFIRME
- Fichier(s) : [http.rs:392-399](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:392)
- Evidence :

```text
HEAD e48d8280 :       89 appels .route(
Working tree :        91 appels .route(
Nouvelles routes :    GET workflow/run/{run_id}
                      GET workflow/lease
```

Le décompte a été effectué directement sur la version HEAD puis sur le fichier courant.

### Livrable 4 : invariants durs

- Statut : CONFIRME
- Fichier(s) : [Cargo.lock:5194-5199](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:5194), [Cargo.toml engine:32-39](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/Cargo.toml:32), [check-frontier-contracts.sh:202-256](C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:202)
- Evidence :

```text
git diff engine/core : vide
Cargo.lock numstat   : +1 / -0
FRONTIER workflow    : 0 tag
wire/domain ajouté   : 0
frontier gate        : clean — DOMAIN census [29 frozen]
```

`Cargo.lock` ajoute uniquement `"sbfb-workflow-engine"` à la liste des dépendances déjà existante du daemon ; aucun package externe n’est ajouté. `git status` et `git diff` sont vides pour `crates/sbfb-workflow-engine` et `crates/sbfb-workflow-core`. Le gate retourne bien `exit 0`.

### Livrable 5 : tests et golden HTTP

- Statut : CONFIRME
- Fichier(s) : [workflow_api.rs:215-380](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:215), [workflow_api.rs:393-459](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/workflow_api.rs:393), [test_support.rs:302-338](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:302), [test_support.rs:632-686](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:632), [test_support.rs:721-764](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:721)
- Evidence :

```rust
446:        state
450:            .ensure_run("run-duress", "digest-d")
452:        let (status, json) = get_json(&app, "/api/daemon/workflow/run/run-duress").await;
453:        assert_eq!(status, 200);
455:            json["found"], true,
```

Le module contient exactement neuf tests. Ils exercent utilement les trois statuts, le miss, la lease vide/présente, la key-set allowlist, la garde, le contrat HTTP et le duress avec run réel seedé. `golden_http_workflow_domain` contient trois cas avec assertions de statut, headers et corps via `golden_check`, lignes 316-338. Les deux sondes workflow sont présentes dans `golden_families_stable`, lignes 632-644.

Les stores de test sont file-backed dans le tempdir : `mk_state` aux lignes 145-156 et l’initialisation manuelle de `curators_api` aux [lignes 497-504](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:497).

### Livrable 6 : inventaire D7

- Statut : CONFIRME
- Fichier(s) : [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:1-10](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:1), [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:63-69](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:63), [LOOPBACK_ENDPOINTS_TRUST_TIERS.md:102-105](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:102)
- Evidence :

```text
102: GET  /api/daemon/workflow/run/{run_id}         S83 I1 — T0
103: POST /api/daemon/workflow/run/{run_id}/restore FUTUR J/K — T1
104: POST /api/daemon/workflow/gc                    FUTUR J/K — T1
105: POST /api/daemon/workflow/lease/force           FUTUR J/K — T1
```

Le front-matter porte `last_validated: 2026-07-22` et annonce exactement `33 rows = 28 route-paths ... sur 91 routes`. Le recomptage direct confirme 33 rows, quatre futures au total — canary plus les trois mutations workflow — et 28 chemins présents distincts après déduplication de `/api/v1/consent/set`. La row T0 indique explicitement qu’elle couvre aussi GET lease.

### Livrable 7 : THREAT_MODEL v24

- Statut : CONFIRME
- Fichier(s) : [THREAT_MODEL.md:1763-1771](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1763), [THREAT_MODEL.md:1773-1785](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1773), [THREAT_MODEL.md:2130-2152](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2130)
- Evidence :

```text
1766: le chemin daemon ECRIVAIN (J) acquiert ; les routes I1 sont LECTURE SEULE
1767: tier D7 DELIBERE en I1 ; rows T1 pré-classées, implémentation DATEE J/K
1771: run_id serveur-validé [A-Za-z0-9._-]{1,128} → 400
1774: Phase I1 LIVREE en LECTURE SEULE
```

La première correction est effective : `T-RUN-RESTORE-ROLLBACK` ne promet plus une exposition I1 et indique clairement l’absence des routes restore/GC. `T-RUN-DOUBLE-EMIT` attribue l’acquisition au writer J. Les bornes honnêtes séparent reads I1 et mutations J/K, et le changelog v24 reprend routes, auth, DTO allowlist, duress, absence de lease I1, census et byte-intact.

### Livrable 8 : clôture docs-contrat workflow

- Statut : CONFIRME
- Fichier(s) : [docs/workflow/llms.txt:20-62](C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:20), [REFERENCE.md:19-94](C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/REFERENCE.md:19), [llms.txt:8-14](C:/Users/FlowUP/Documents/Code/nexus/llms.txt:8), [llms.txt:28-32](C:/Users/FlowUP/Documents/Code/nexus/llms.txt:28)
- Evidence :

```json
42:    "status": "running",
43:    "terminal_reason": null,
44:    "event_count": 1,
45:    "frontier_node": null,
46:    "spec_digest": "digest-a"
```

La seconde correction est effective : l’exemple est désormais un littéral conforme au DTO et non une pseudo-union JSON. Les domaines alternatifs sont expliqués séparément aux lignes 51-55. L’exemple lease correspond également aux champs réels. Les ancres `workflow_run_response`, `valid_run_id`, `workflow_lease_response` et `lease_state` existent dans les fichiers cités. Le `llms.txt` racine indexe bien workflow comme troisième sous-système.

### Livrable 9 : artefacts process

- Statut : CONFIRME
- Fichier(s) : [sprint83_phase_i1_preflight.md:13-20](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_i1_preflight.md:13), [sprint83_phase_i1_preflight.md:90-161](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_i1_preflight.md:90), [sprint83_phase_i1_review_r1.md:1-10](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i1_review_r1.md:1), [sprint83_phase_i1_review_r2.md:1-25](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i1_review_r2.md:1)
- Evidence :

```text
preflight:13  Verdict G8 : PLAN-ADAPT
preflight:92  A1. Famille mutante DIFFÉRÉE J/K
preflight:98  A2. N=2 routes READ, inline dans authed_routes
review_r2:5  Verdict r2 : PASS-PENDING
```

Le preflight contient bien A1 à A13. Le round r1 conserve son verdict historique `CONCERN` et ses findings ; le round r2 ferme les corrections et porte le verdict attendu `PASS-PENDING`.

### Recoupe transverse

Les trois seuls constructeurs directs de `DaemonHttpState` ont tous été adaptés : production [runtime.rs:756-823](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:756), harness central [test_support.rs:157-209](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:157) et test manuel [curators_api.rs:505-560](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:505). Les nombreux consommateurs de `mk_state*` passent tous par le constructeur central modifié. Aucun fichier `web/`, engine ou core n’est modifié, et aucune route mutante workflow n’est enregistrée.

## Resume final

- Total livrables : 9
- Confirmes : 9
- Gaps : 0
- Partiels : 0