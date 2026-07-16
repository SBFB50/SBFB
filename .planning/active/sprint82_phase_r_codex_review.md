Verdict global : la migration source est fidèle à `HEAD 7faa632e6b071e5a9b0990e62f650da8a3133706`. Aucun drift fonctionnel n’a été trouvé. Deux points restent partiellement démontrés : l’exécution des tests est bloquée par le sandbox en lecture seule, et le working tree global contient trois changements de recherche hors Phase R.

### Livrable 1 : Nouveau module et migration production

- Statut : CONFIRME
- Fichier(s) : [curators_api.rs:1-178](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:1)
- Evidence :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscribeCuratorRequest {
    /// Lowercase hex of the curator's Ed25519 public key (64 chars).
    pub curator_pubkey_hex: String,
```

La ligne 1 contient le SPDX et les lignes 2-15 le bandeau anglais. Les sept `use` production sont exactement aux lignes 17, 19-23 et 25. Les quatre DTO commencent aux lignes 38, 49, 56 et 73 ; les quatre handlers `pub(crate)` aux lignes 80, 92, 148 et 168.

Comparaison mécanique UTF-8, fins de ligne normalisées :

- `HEAD:http.rs:728-769` = `curators_api.rs:27-68`, SHA-256 identiques.
- `HEAD:http.rs:827-833` = `curators_api.rs:70-76`, SHA-256 identiques.
- `HEAD:http.rs:924-1009` = `curators_api.rs:78-163` après retrait des seuls quatre préfixes `pub(crate)`, SHA-256 identiques.
- `HEAD:http.rs:1932-1943` = `curators_api.rs:165-178` après retrait de `pub(crate)` et normalisation du re-wrap rustfmt de la signature. Les docs et le corps sont strictement identiques.

### Livrable 2 : Invariants de sécurité

- Statut : CONFIRME
- Fichier(s) : [curators_api.rs:92-162](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:92), [http.rs:818-843](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:818), [iroh_runtime.rs:671-675](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon-core/src/iroh_runtime.rs:671)
- Evidence :

```rust
if crate::noop_identity::curator_subscribe_in_duress(state.identity_mode)
    == crate::noop_identity::SubscribeOutcome::Noop
{
    return (
        StatusCode::OK,
```

Après le `debug!` ligne 96, la porte duress est bien le premier statement exécutable, lignes 101-103. L’early return 200 avec liste vide est aux lignes 104-110, avant `subscribe` ligne 112 et avant le hot-join lignes 128-133.

Le `send(GossipCmd::JoinPeers(...)).await` est best-effort (`let _`) dans le bras `Ok`, donc après la mutation réussie. La recherche crate-wide ne trouve qu’une construction productrice, ligne 130 ; les autres occurrences production sont la définition de l’enum dans `runtime.rs:1665` et son consommateur à `runtime.rs:2064`.

`unsubscribe_curator`, lignes 148-162, ne contient aucune porte duress. La validation reste déléguée à `CuratorRuntime::subscribe` (`iroh_runtime.rs:671-675`) ; l’erreur remonte au helper ligne 142, où `BadPubkeyHex` est mappée sur 400 dans `http.rs:826-842`.

### Livrable 3 : Dix tests router-driven co-migrés

- Statut : CONFIRME
- Fichier(s) : [curators_api.rs:180-610](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/curators_api.rs:180)
- Evidence :

```rust
assert_eq!(resp.status(), StatusCode::OK);
let body = to_bytes(resp.into_body(), 4096).await.unwrap();
let list: CuratorsListResponse = serde_json::from_slice(&body).unwrap();
assert!(list.entries.is_empty());
assert!(list.subscribed_curators.is_empty());
```

Le bloc d’imports demandé est présent aux lignes 182-193, avec `crate::test_support::*` dans un groupe séparé. Il existe exactement dix `#[tokio::test]`, aux lignes 195, 215, 281, 320, 347, 395, 431, 466, 485 et 575. La bannière Sprint 11 Phase B est aux lignes 462-464.

Comparaison mécanique :

- `HEAD:http.rs:3756-4021` = `curators_api.rs:195-460`.
- `HEAD:http.rs:4467-4488` = `curators_api.rs:462-483`.
- `HEAD:http.rs:4524-4608` = `curators_api.rs:485-569`.
- `HEAD:http.rs:5147-5186` = `curators_api.rs:571-610`.

Les quatre paires ont des SHA-256 identiques. Chaque test traverse le routeur avec `oneshot` et possède au moins une assertion utile ; nombres d’assertions observés : `3, 7, 1, 1, 3, 2, 2, 2, 2, 3`.

### Livrable 4 : Visibilités partagées dans http.rs

- Statut : CONFIRME
- Fichier(s) : [http.rs:811-844](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:811)
- Evidence :

```rust
pub(crate) struct ErrorResponse {
    error: String,
}

pub(crate) fn runtime_error_to_response(
```

`ErrorResponse` est `pub(crate)` ligne 814, tandis que son champ `error` reste privé ligne 815. Le helper est `pub(crate)` lignes 818-820. Le diff de visibilité de `http.rs` ne contient aucun autre ajout `pub(crate)`.

Les cinq constructions non-curators restantes sont aux lignes 1106, 1198, 1825, 1841 et 1877, réparties entre `publish_project`, `publish_directory`, `publish_blob` et `panic_wipe`.

### Livrable 5 : Amputation de http.rs et reroutage

- Statut : CONFIRME
- Fichier(s) : [http.rs:282-376](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:282), [http.rs:2616-2623](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2616)
- Evidence :

```rust
.route(
    "/api/daemon/curators/subscribe",
    post(crate::curators_api::subscribe_curator),
)
.route(
```

Les quatre routes conservent exactement leurs paths et pointent vers le chemin complet :

- `/api/daemon/curators` : lignes 285-288.
- `/api/daemon/curators/subscribe` : lignes 289-292.
- `/api/daemon/curators/{pubkey}` : lignes 293-296.
- `/api/daemon/default-curators` : lignes 373-376.

Recherche résiduelle dans `http.rs` : zéro définition des quatre DTO, zéro définition des quatre handlers et zéro des dix tests migrés. Les trois imports test devenus orphelins sont absents ; le bloc restant est aux lignes 2618-2623.

Les quatre tests STAY restent présents : `info_reflects_live_curator_runtime_counts` ligne 3617, `browse_returns_empty_list_when_no_curators_cached` ligne 3643, `daemon_boot_in_duress_mode_publishes_fake_curator_empty` ligne 4575 et `spa_fallback_serves_curators_as_html_document` ligne 4713. La bannière S20-B reste aux lignes 4564-4566.

### Livrable 6 : Déclaration du module

- Statut : CONFIRME
- Fichier(s) : [main.rs:31-39](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:31)
- Evidence :

```rust
mod contributor_api;
mod coordinator_api;
mod curators_api;
mod deploy;
mod diagnostic_api;
```

Le module est normal, sans `#[cfg(test)]`, et placé alphabétiquement entre `coordinator_api` et `deploy`, ligne 37. Le diff de `main.rs` ne contient que cet ajout.

### Livrable 7 : Références documentaires

- Statut : CONFIRME
- Fichier(s) : [daemon.ts:100-103](C:/Users/FlowUP/Documents/Code/nexus/web/src/api/daemon.ts:100), [docs/rust/PATTERNS.md:933-946](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:933), [docs/shell/PATTERNS.md:1172-1178](C:/Users/FlowUP/Documents/Code/nexus/docs/shell/PATTERNS.md:1172)
- Evidence :

```text
`#[serde(deny_unknown_fields)]` on `SubscribeCuratorRequest`,
`SubscriptionsResponse` and `CuratorsListResponse` (all three in
`crates/nexus-shell-daemon/src/curators_api.rs` since the Sprint 82
Phase R split), and on `BrowseListResponse` (stays in
`crates/nexus-shell-daemon/src/http.rs`).
```

Le commentaire web pointe sur `curators_api.rs` ligne 102. G-3 attribue les trois DTO à `curators_api.rs`, conserve `BrowseListResponse` dans `http.rs`, supprime les anciens numéros inline et repointe le test à `curators_api::tests::subscribe_rejects_extra_fields`, lignes 935-945. P19 pointe sur `curators_api.rs` ligne 1177.

`docs/claude/SPRINT_LOG.md` est byte-identique à `HEAD` : blob `03bf62789003ac4a883d69b879047ef51cc2ae35`.

### Livrable 8 : Golden intact et vert

- Statut : PARTIEL
- Fichier(s) : [test_support.rs:235-345](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:235), [test_support.rs:543-570](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:543)
- Evidence :

```rust
name: "curators_unsubscribe_unknown",
method: Method::DELETE,
uri: "/api/daemon/curators/deadbeef",
json_body: None,
want_status: 400,
```

`test_support.rs` est strictement intact : blob working tree et `HEAD` identiques, `80a456b49305d49279bef4b67629354ce6cab10d`. Les neuf tests golden sont toujours regroupés dans ce fichier, aux lignes 350, 393, 426, 468, 501, 543, 580, 626 et 667. Le harness vérifie réellement status, headers et corps aux lignes 308-334. Le golden curators fixe le JSON d’erreur `BadPubkeyHex` aux lignes 559-567.

Le delta statique de tests est exactement nul : `http.rs` passe de 166 à 156 attributs de test et `curators_api.rs` en apporte 10. Un binaire de test construit après les sources liste bien les 9 goldens et les 10 tests migrés. `cargo fmt --all --check` et `git diff --check` passent.

Réserve : le vert n’a pas pu être rejoué. Cargo est bloqué sur `target/debug/.cargo-lock` en lecture seule ; l’exécution directe des neuf goldens s’arrête avant leurs assertions à `test_support.rs:137`, lorsque `tempfile::tempdir()` reçoit `PermissionDenied`. Le baseline nextest Windows `2108/2108` n’est donc pas indépendamment confirmé dans cet environnement.

### Livrable 9 : Périmètre strict

- Statut : PARTIEL
- Fichier(s) : [http.rs:75-202](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:75), [daemon.ts:100-103](C:/Users/FlowUP/Documents/Code/nexus/web/src/api/daemon.ts:100), `.planning/research/**`
- Evidence :

```rust
pub gossip_cmd_tx: crate::runtime::GossipCmdTx,
/// Default curator pubkeys from `[curator]` config section.
/// Sprint 11 Phase B. Exposed via `GET /default-curators`.
pub default_curators: Vec<String>,
/// Sprint 12 Phase A: LRU cache of decompressed zip archives
```

Les cinq contraintes explicites sont confirmées :

- `Cargo.toml` et `Cargo.lock` ont les mêmes blobs que `HEAD`.
- Dans `web/`, seul `web/src/api/daemon.ts` change, uniquement sur le commentaire ligne 102.
- `build_router` ligne 246 et `authed_routes` ligne 282 gardent leurs noms.
- `DaemonHttpState`, lignes 75-202, est exactement identique à `HEAD`; `default_curators` reste ligne 96.
- Les corps migrés sont identiques à `HEAD` selon les comparaisons du livrable 1.

La réserve concerne le working tree global : il porte aussi une modification de `.planning/research/sprint82_workflow_engine/verification_blueprint.md` et deux nouveaux fichiers `workflow_*_2026-07-15.md`, sans rapport de contenu avec le split. Les artefacts Phase R les qualifient de préexistants, mais une comparaison `working tree vs HEAD` seule ne permet pas d’établir indépendamment cette provenance. L’exclusivité globale du périmètre Phase R reste donc partielle.

## Résumé final

- Total livrables : 9
- Confirmés : 7
- Gaps : 0
- Partiels : 2

Aucun fichier n’a été modifié pendant cet audit.