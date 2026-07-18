Base auditée : `master`, HEAD `c5be6e4c31590c435bd608ffeb5e4015632be3bb`, diff de phase non commitée. Les fichiers `.planning/research/*` ont été ignorés. Les constats ci-dessous proviennent du code et des commandes exécutées sur le checkout actuel.

### Livrable 1 : Nouveau module `seed_api.rs`

- Statut : CONFIRME
- Fichier(s) : [seed_api.rs:27](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/seed_api.rs:27), [http.rs:1629](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1629)
- Evidence :
  - `seed_api.rs` contient exactement 2325 lignes.
  - Les tranches HEAD `http.rs:1610-1688`, `1802-2037` et `2138-2845` correspondent à `seed_api.rs:27-105`, `107-342` et `344-1051`.
  - Comparaison conservant littéraux et texte des commentaires : 18/18 items identiques après les seules normalisations autorisées — visibilité, whitespace/rustfmt.
  - Sept handlers deviennent `pub(crate)` : lignes 39, 380, 664, 738, 801, 821 et 900.
  - Six DTO deviennent `pub(crate)` avec zéro champ public : lignes 34, 360, 626, 730, 797 et 887.
  - `SeedFetchPlan`, `build_seed_fetch_chain` et `SEED_REQUEST_TIMEOUT_SECS` restent privés : lignes 373, 610 et 859.
  - Les quatre blocs duress sont byte-identiques à HEAD. Le premier statement du driver est bien le garde des lignes 154-158.

```rust
#[derive(Debug, serde::Deserialize)]
pub(crate) struct KeepOnlineRequest {
    project_id: String,
    enabled: bool,
}
```

```rust
if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
    == crate::noop_identity::PublishOutcome::Noop
{
    return 0;
}
```

  - Le cluster Directory-only reste entier dans `http.rs:1629-1735`; `PULL_PROVIDER_CAP` et `find_directory_app_by_hash` restent privés.
  - Le cluster nodes reste entier dans `http.rs:1737-1834`.

### Livrable 2 : Tests co-migrés

- Statut : CONFIRME
- Fichier(s) : [seed_api.rs:1053](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/seed_api.rs:1053), [http.rs:3200](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:3200)
- Evidence :
  - Le module utilise bien le harness partagé.

```rust
use crate::test_support::*;

#[tokio::test(flavor = "multi_thread")]
async fn seed_voluntary_directory_only_app() {
```

  - Les 18 tests demandés commencent aux lignes 1064, 1162, 1317, 1383, 1459, 1604, 1640, 1666, 1780, 1814, 1830, 1886, 1917, 1960, 2000, 2044, 2142 et 2237.
  - `has_tag` est co-migré aux lignes 2129-2139.
  - Comparaison par fonction contre HEAD : 19/19 identiques, attributs de test compris.
  - Chaque test contient des assertions utiles : minimum 2, maximum 14 assertions inspectées par fonction.
  - Aucun des 18 noms migrés ne subsiste sous `http::tests`.
  - Les tests explicitement conservés existent : `publish_directory_*` lignes 3397-3650, `directory_resolvers_match_hash_and_project:3749`, `fetch_provider_ordering:3841`, `nodes_response_pins_envelope_and_grouping:3892`, `reachable_via_seeder_status:3965`, `vps_authoring_signs_own_directory:4075`.

### Livrable 3 : Fixtures partagées promues

- Statut : CONFIRME
- Fichier(s) : [test_support.rs:704](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:704), [seed_api.rs:1083](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/seed_api.rs:1083), [http.rs:3979](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:3979)
- Evidence :
  - Définitions `pub(crate)` uniques :
    - `own_browse_entry:704`
    - `catalog_app:724`
    - `make_zip:739`
    - `deploy_workspace_app:755`
    - `ingest_remote_directory:781`
  - Recherche sur tout `crates/nexus-shell-daemon/src/**/*.rs` : exactement une définition par symbole.
  - Comparaison contre HEAD : 5/5 identiques après `pub(crate)`, dédent et re-wrap rustfmt. `catalog_app` reçoit uniquement la virgule terminale imposée par son passage en signature multiligne.
  - Elles sont consommées par les deux modules : par exemple `seed_api.rs:1083-1087`, `2244-2248`; `http.rs:3979-3983`, `8931-8933`.

```rust
pub(crate) fn catalog_app(
    project_id: &str,
    archive_hash: &str,
    name: &str,
) -> nexus_core_rs::CatalogApp {
```

### Livrable 4 : Purge `http.rs`, routes et visibilités STAY

- Statut : CONFIRME
- Fichier(s) : [http.rs:282](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:282), [http.rs:1634](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1634)
- Evidence :
  - `http.rs` : 9518 lignes.
  - `git diff --numstat HEAD` : `1476 / 3861`, soit −2385 lignes nettes.
  - Les sept paths restent byte-identiques et sont re-pointés vers `crate::seed_api::*` aux lignes 291-321.

```rust
.route(
    "/api/daemon/seed/request",
    post(crate::seed_api::seed_request_peer),
)
```

```rust
.route(
    "/api/daemon/seed-count/{project_id}",
    get(crate::seed_api::seed_count),
)
.route("/api/daemon/nodes", get(list_nodes))
```

  - Ils sont toujours dans `authed_routes`, créé ligne 282, protégé par `auth_required` ligne 553 puis fusionné ligne 558.
  - Comparaison de toutes les déclarations communes entre HEAD et le `http.rs` courant : exactement trois changements de visibilité :
    - `DIRECTORY_PULL_TIMEOUT_SECS:1640`
    - `find_directory_app_by_project:1671`
    - `directory_pull_providers:1707`
  - `PULL_PROVIDER_CAP:1634` et `find_directory_app_by_hash:1646` restent privés.
  - Aucun delta `Cargo.toml`/`Cargo.lock`; aucune définition `*_VERSION` modifiée.

### Livrable 5 : Re-points cross-module et documentation

- Statut : CONFIRME
- Fichier(s) : [main.rs:53](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:53), [runtime.rs:1516](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:1516), [THREAT_MODEL.md:1019](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1019), [PATTERNS.md:4153](C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:4153)
- Evidence :

```rust
mod runtime;
mod seed_api;
mod seed_protocol;
```

  - `runtime.rs` contient exactement deux appels `crate::seed_api::run_boot_seed_driver`, lignes 1523 et 2473, et zéro ancien appel `crate::http::run_boot_seed_driver`.
  - `reannounce_directory_at_boot` reste qualifié `crate::http::` ligne 1516.

```rust
let pinned = {
    let _guard = seed_driver_lock.lock().await;
    crate::seed_api::run_boot_seed_driver(&boot_driver_state, &configured).await
};
```

  - `deploy.rs` est inchangé (`git diff --quiet` = 0).
  - `THREAT_MODEL.md:1019` désigne `seed_api.rs run_boot_seed_driver`.
  - `PATTERNS.md:4153` utilise l’ancre stable `http.rs:directory_pull_providers`.

### Livrable 6 : Oracles de compte et gates

- Statut : CONFIRME
- Fichier(s) : [test_support.rs:427](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:427), [seed_api.rs:1053](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/seed_api.rs:1053)
- Evidence :

```rust
GoldenCase {
    name: "seed_invites_list",
    method: Method::GET,
    uri: "/api/daemon/seed/invites/proj-golden",
    json_body: None,
```

  - `cargo nextest run --workspace --locked` : **2108/2108 passés, 0 skipped**.
  - `cargo nextest list -p nexus-shell-daemon --locked` : **466 tests** — 453 binaires, 6 E2E, 7 loopback.
  - Exactement **18** sous `seed_api::tests::`, **9** `golden_http_*` sous `test_support::`, **0** des 18 tests migrés sous `http::tests::`.
  - Attributs de test dans les conteneurs concernés : HEAD `205 + 9 = 214`; courant `187 + 9 + 18 = 214`. Delta exact : zéro.
  - `cargo fmt --all --check` : vert.
  - `cargo clippy --workspace --all-targets --locked -- -D warnings` : vert.
  - `scripts/check-sharding-docs.sh` : clean.
  - `scripts/check-frontier-contracts.sh` : clean.
  - `git diff --check` : vert.
  - Zéro delta `Cargo.toml`/`Cargo.lock`; zéro définition `*_VERSION` changée. La référence `SEED_FORMAT_VERSION` à `seed_api.rs:974` est seulement co-déplacée verbatim.

## Résumé final

- Total livrables : 6
- Confirmés : 6
- Gaps : 0
- Partiels : 0

Le `PASS-PENDING` de l’artefact de review reste conforme au séquencement annoncé et n’est pas traité comme une preuve : le verdict ci-dessus repose sur le code et les gates rejoués. HEAD est resté `c5be6e4` pendant tout l’audit.

