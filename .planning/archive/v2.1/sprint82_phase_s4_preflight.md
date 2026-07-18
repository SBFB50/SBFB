# Sprint 82 — Phase S4 — Préflight G8 (synthèse)

- **Phase** : S4 — sweep final : dispatch des singles (`canary_*`, `diagnostic_neighborhood`, `panic_wipe`, `blob_serve` + middleware CSP + cluster pull) + assertion critère machine PO-10
- **Date** : 2026-07-17
- **HEAD** : `0a32ffa` — `http.rs` = 4322 l (re-vérifié `wc -l` + `git rev-parse` par le synthétiseur ; lecture seule)
- **Cas** : B (pre-code)
- **Verdict** : **PLAN-ADAPT** (le move-set nommé du plan tient avec toutes ses ancres EXACTES sur disque, MAIS 4 adaptations matérielles : [1] `mint_blob_ticket`+`archive_hash_from_ticket` **STAY** http.rs contre le défaut plan ; [2] middleware CSP **MOVE** — contradiction interne Goal/état-final tranchée côté Goal avec evidence ; [3] cluster pull **MOVE** — re-arbitrage S2 inversé par le fait nouveau « blob_serve part » ; [4] co-migration ÉTENDUE des familles de tests router-driven pré-série — sans elle le critère machine PO-10 `wc -l ~≤2500 TOTAL` ÉCHOUE arithmétiquement [~3140–3260])

Préflight = dossier 6 paires scan+critic [moveset, tests, couplage, history, threat, docsgates]. Les critics PRIMENT sur les scans quand ils apportent une evidence disque ; le synthétiseur a ré-ancré lui-même les faits load-bearing sur disque HEAD `0a32ffa` (tip + 4322 l + 89 routes + doc-contrat publish_api.rs:20-21 verbatim). Convergence FORTE du dossier : les 6 scans + 6 critics sont d'accord sur les ancres, les 3 arbitrages et le dépassement PO-10 ; les critics n'ont produit que des compléments (re-points manqués, doubles-comptes ~15 l, nits off-by-one).

---

## 1. Contexte

Phase S4 = 10ᵉ et DERNIÈRE phase refacto de la série N→S4 avant clôture T. Le plan (`sprint82_plan.md` §S4, l.440-452) nomme le move-set par NOM ; toutes les ancres re-dérivées sur disque sont EXACTES (`canary_observed` :1347, `canary_network_health` :1377, `canary_freshness` :1398, `diagnostic_neighborhood` :1331, `panic_wipe` :1112, `blob_serve` :1145, `mint_blob_ticket` :1306, middleware :655, cluster :923/:929/:935/:960/:996). **0 borne stale** — mais le plan a DEUX omissions (`find_directory_app_by_hash` :935 et `archive_hash_from_ticket` :1299, tranchées §2/§4), une contradiction interne (middleware CSP, §3), un Goal partiellement stale (`default_curators`/`publish_blob` DÉJÀ dispatchés R/S — seuls restes vérifiés : champ état :95 + routes déjà en `crate::` :378/:386, 0 action), et un critère machine inatteignable au scope nommé seul (§8).

Région production 1-1425 ; `mod tests` 1427-4322 (2896 l) ; le `#[cfg(test)]` inline :773 = `BrowseListResponse` (DTO test-only, verdict S2 STAY reconfirmé §6.4). Bannière orpheline `// -- Sprint 74 Phase D: keep-online local pin --` :4321 confirmée (suivie du seul `}` :4322 — legs Phase O) → retrait.

---

## 2. ARBITRAGE 1 — cluster « Directory-only pull resolution » : **MOVE → blob_serve_http.rs** (TRANCHÉ)

Question routée par S2 (« le cluster suit-il blob_serve ? », sprint82_phase_s2_preflight.md §3). **OUI.**

**Evidence décisive** : le verdict STAY de S2 (unanime 7 scans + 3 critics à l'époque) reposait sur UN prédicat — « 0 caller dans le move-set » (blob_serve restait dans http.rs). Ce prédicat est **INVERSÉ** en S4 : `blob_serve` (le caller le plus lourd : :1202 `find_directory_app_by_hash`, :1215 `directory_pull_providers`, :1228 `DIRECTORY_PULL_TIMEOUT_SECS`) devient MOVER. Comparaison des coûts (vérifiée par les paires couplage+history+threat) :

| Option | Bumps | Churn | Cohérence |
|---|---|---|---|
| **STAY http.rs** | **1 bump SHARED NOUVEAU** (`find_directory_app_by_hash` :935 privée→pub(crate), compiler-forcé : son unique caller prod quitte le module) | 3 arêtes blob_serve_http→http | tests cluster :1521-1665 stranded dans http.rs sans symbole testé restant |
| **MOVE blob_serve_http.rs** | **0 bump nouveau** (`by_hash` + `PULL_PROVIDER_CAP` :923 restent PRIVÉS co-localisés, leurs tests co-migrent) | 1 édit du bloc `use` seed_api.rs:22-25 (split compiler-forcé, re-point ROUTINE) + 2 re-points docs | « les 5 symboles existent pour les tiers pull de blob_serve, seed_api réutilise » |

Le garde-fou « STOP si edit seed_api » des checklists S2/S3 était l'ORACLE d'un arbitrage STAY, pas un invariant permanent (history §1.6) — l'édit seed_api devient un re-point d'import PLANIFIÉ. Le cluster reste **ATOMIQUE** (jamais éclaté — figé O+S2 ×2) : bannière :914-916 + CAP :918-923 + TIMEOUT :925-929 + by_hash :931-947 + by_project :949-976 + providers :978-1024 (nested `push_unique` :1005-1014) partent EN BLOC. `find_directory_app_by_hash` (omis de l'énumération du plan) est membre de fait et co-migre PRIVÉ. Les doc-comments porteurs de menace voyagent VERBATIM (:918-922 Sybil dial-chain, :925-929 budget mur, :978-995 ordering=fallback + SYBIL-SEEDER-TAIL « carried to the S76 audit »).

---

## 3. ARBITRAGE 2 — middleware `blob_serve_csp_middleware` : **MOVE → blob_serve_http.rs** (TRANCHÉ)

Contradiction interne du plan : Goal :446 « (+ middleware CSP si déplaçable proprement) » VS état final :448-449 « middlewares auth/CORS/CSP … » restant dans http.rs (même liste PO-10 (b) :340-341 et memory PO).

**Evidence qui tranche côté Goal** (convergence moveset+couplage+threat+docsgates, avec la clause d'ouverture « si déplaçable proprement » écrite dans le Goal lui-même — SATISFAITE) :
1. Consommateur UNIQUE = `build_router` :256, layer scoped au nest `/blob-serve` :254-256 — **PAS un middleware routeur global** : auth = `daemon-core::auth_required` appliqué :606 (n'a JAMAIS vécu dans http.rs), CORS = `cors_layer` :685-700 appliqué globalement :613 (STAY). L'énumération « auth/CORS/CSP » de l'état final est une liste par analogie, pas une contrainte structurelle.
2. Cohésion 100 % domaine blob : nom, contrat T37, toutes constantes via `nexus_shell_daemon_core::blob_serve` (re-exports S79 de `nexus_core_rs::csp` — la SOURCE du contrat ne bouge pas, figé S79-E).
3. Coût MOVE = 1 bump ROUTINE + re-point full-path :256 + 3 re-points docs. Coût STAY = fn mono-domaine orpheline dans http.rs pendant que le handler qu'elle enveloppe part.
4. **Filet mécanique double** : tests témoins :2260 (`blob_serve_error_responses_have_csp`) + :2305 (`blob_serve_csp_header_byte_exact_matches_contract`) co-migrent et traversent le VRAI `build_router` prod (test_support.rs:75) ; + golden `golden_http_public_tier` (test_support.rs:351) fige BYTE-EXACT CSP+nosniff+COOP+COEP+CORP sur `/blob-serve/not-a-hash/` — un dé-câblage silencieux du layer casse les deux.

**État final REFORMULÉ** (adaptation n°3) : « middlewares **auth/CORS** + helpers origin restent dans http.rs ; le middleware CSP suit le nest blob-serve dans `blob_serve_http.rs` ». L'invariant sécurité INTOUCHABLE : layer toujours câblé au nest `/blob-serve` de `public_routes` SANS bearer (S76-B8 THREAT-BLOBSERVE-BEARER, THREAT_MODEL:1014 — public PAR CONSTRUCTION).

---

## 4. ARBITRAGE 3 — destinations `panic_wipe` / `mint_blob_ticket` / `archive_hash_from_ticket`

### 4.1 `panic_wipe` → **blob_serve_http.rs** (défaut plan RATIFIÉ)
Thin-dispatch (35 l) sur `crate::panic::PanicWipeService` (panic.rs, NE BOUGE PAS — toute la logique destructive + ses tests unitaires y vivent). Aucun module d'accueil plus naturel n'existe (créer un `panic_api.rs` pour 35 l = surcoût sans gain ; `panic.rs` est un module SERVICE, pas API — y coller un handler axum mélangerait les couches). Cohabitation « singles résiduels du chemin daemon-critique dans le module neuf », pattern assumé (précédent S3 : « deux singletons résiduels S68 dans un fichier »). AUCUN test router-driven `/panic/wipe` n'existe (grep exhaustif confirmé 2 scans) → 0 test à co-migrer. Invariants VERBATIM §9.

### 4.2 `mint_blob_ticket` + `archive_hash_from_ticket` : **STAY http.rs** (DÉVIATION du défaut plan — adaptation n°1)
Le plan liste `mint_blob_ticket` dans le move-set blob_serve_http.rs par défaut. **Evidence contre, unanime history+couplage+threat** :
- **blob_serve ne l'appelle PAS** (grep : unique occurrence http.rs = sa définition :1306). blob_serve = côté CONSOMMATEUR (find_archive_ticket_by_hash/fetch_ticket/fetch_hash_multi) ; mint = côté PRODUCTEUR (publish/seed/deploy). Le défaut plan regroupait deux affinités OPPOSÉES.
- 3 consommateurs multi-domaines : deploy.rs:682, publish_api.rs:34+:375, seed_api.rs:24+:956 — classe hub EXACTE de `wrap_payload_with_pow`/`truncate_on_char_boundary`/`ErrorResponse` qui restent.
- **Doc-contrat Phase S** (publish_api.rs:20-21, vérifié verbatim par le synthétiseur) : « `mint_blob_ticket` and `wrap_payload_with_pow` **stay in `http.rs` (multi-domain consumers)** ». MOVE falsifierait un doc-contrat écrit il y a une phase ; STAY le garde VRAI (0 réécriture).
- STAY économise : réécriture publish_api:20-21 + 3 re-points `use` + re-point doc-link `runtime.rs:2521` `` [`crate::http::mint_blob_ticket`] `` (angle mort classe S2-F2, invisible cargo doc).
`archive_hash_from_ticket` :1299 (absent du plan, sibling « ticket helpers ») : consommateur prod unique = runtime.rs:2282 — **pas** blob_serve. Règle « pas de panaché » (couplage §5) : les deux STAY ensemble. Coût résiduel accepté : son test :2195-2214 reste dans http.rs → hazard import `BlobsClient` (§7.2).

---

## 5. MOVE-SET FINAL PRODUCTION (par NOM, bornes disque re-dérivées HEAD `0a32ffa`)

Bannières de section appartiennent au symbole qui suit (co-migrent).

### 5.1 → `blob_serve_http.rs` (module NEUF ; `mod blob_serve_http;` dans main.rs, ordre alpha entre `browse_api` :32 et `canary_api` :33)

| # | Symbole | Kind | Lignes (doc+attrs inclus) | Vis. → cible | Bump |
|---|---|---|---|---|---|
| 1 | `blob_serve_csp_middleware` | async fn middleware | 653-681 (fn :655) | privée → **pub(crate)** | ROUTINE |
| — | Bannière « Directory-only pull resolution » | — | 914-916 | co-migre | — |
| 2 | `PULL_PROVIDER_CAP` | const | 918-923 (const :923) | privée → **reste privée** | — |
| 3 | `DIRECTORY_PULL_TIMEOUT_SECS` | const | 925-929 (const :929) | pub(crate) → pub(crate) | — |
| 4 | `find_directory_app_by_hash` | fn | 931-947 (fn :935) | privée → **reste privée** (caller = blob_serve co-mover + tests co-movers) | — |
| 5 | `find_directory_app_by_project` | fn | 949-976 (fn :960) | pub(crate) → pub(crate) | — |
| 6 | `directory_pull_providers` (+ nested `push_unique` :1005-1014) | fn | 978-1024 (fn :996) | pub(crate) → pub(crate) | — |
| 7 | `panic_wipe` | async fn handler | 1104-1138 (fn :1112) | privée → **pub(crate)** | ROUTINE |
| 8 | `blob_serve` | async fn handler | 1140-1290 (fn :1145) | privée → **pub(crate)** | ROUTINE |

### 5.2 → `diagnostic_api.rs` (EXISTANT — header :2-5 fairness-only à ÉLARGIR « Diagnostic endpoints », 2 familles de routes)

| # | Symbole | Lignes | Vis. → cible | Bump |
|---|---|---|---|---|
| 9 | `NeighborhoodResponse` | 780-786 (struct :783) | `pub` → `pub` (co-migre ; consommateurs = handler :1336 + test :2383 SEULS, vérifié grep repo) | — |
| 10 | `diagnostic_neighborhood` | 1323-1341 (fn :1331 ; doc :1323-1330 = rationale iroh re-certifié S81-C, co-migre VERBATIM) | privée → **pub(crate)** | ROUTINE |

### 5.3 → `canary_api.rs` (EXISTANT — header :2-5 « Completes the 3 routes already in http.rs » devient FAUX → réécriture OBLIGATOIRE)

| # | Symbole | Lignes | Vis. → cible | Bump |
|---|---|---|---|---|
| — | Bannière « Sprint 39 Phase C — Canary registry HTTP endpoints » | 1343-1345 | co-migre | — |
| 11 | `canary_observed` | 1347-1375 | privée → **pub(crate)** | ROUTINE |
| 12 | `canary_network_health` | 1377-1396 | privée → **pub(crate)** | ROUTINE |
| 13 | `canary_freshness` | 1398-1420 | privée → **pub(crate)** | ROUTINE |

### 5.4 STAY confirmés (production)
`DaemonHttpState` :65-201 + snapshot :203-224 ; `build_router` :226-621 ; `auth_token_public` :623-651 ; `cors_layer` :683-700 ; `is_valid_origin` :702-729 (consommateur main.rs:606) ; `is_loopback_origin` :731-757 ; `BrowseListResponse` :763-778 (cfg(test), verdict S2, §6.4) ; `ErrorResponse` :788-796 + `runtime_error_to_response` :798-824 (SHARED, panic_wipe co-mover la construit :1131 — DÉJÀ pub(crate) champ compris, 0 bump) ; `health`/`info`/`project_info` :830-865 ; `wrap_payload_with_pow` :867-896 ; `truncate_on_char_boundary` :898-912 ; `trustworthy_open_source` :1026-1047 + `index_browse_entry` :1049-1102 (doc soudé :1026-1031) ; **`archive_hash_from_ticket` :1292-1303 + `mint_blob_ticket` :1305-1321 (§4.2)** ; bannière « Tests » :1422-1424.

### 5.5 Re-points `build_router` (paths BYTE-IDENTIQUES, 89 routes invariantes — re-vérifié `grep -c` = 89, 0 `.route(` dans mod tests)
:255 → `crate::blob_serve_http::blob_serve` ; :256 → `middleware::from_fn(crate::blob_serve_http::blob_serve_csp_middleware)` ; :388 → `crate::blob_serve_http::panic_wipe` ; :407 → `crate::diagnostic_api::diagnostic_neighborhood` ; :444/:445/:446 → `crate::canary_api::{canary_observed, canary_network_health, canary_freshness}`.

### 5.6 Bump ledger CONSOLIDÉ = **7 ROUTINE, 0 SHARED, 0 classe R prédit**
Les 7 : middleware, panic_wipe, blob_serve, diagnostic_neighborhood, canary ×3. Signatures vérifiées (Path<String>/Json<Value>/State/Request-Next only — aucun type privé) → 0 DTO compiler-forcé prédit. Le bump SHARED conditionnel (`find_directory_app_by_hash`) est ÉVITÉ par l'arbitrage cluster-MOVE. Si la compile force un bump non prédit (précédents O E0425 / S3 DTOs Query) : amender in-phase, pattern documenté.

---

## 6. TESTS CO-MIGRÉS + EXTENSION PO-10 + PROMOTIONS test_support

### 6.1 Domaines S4 (movers du scope nommé) — par NOM
- **→ blob_serve_http.rs (~448 l)** : bannière :1521 + `directory_resolvers_match_hash_and_project` :1523-1614 + `fetch_provider_ordering` :1615-1665 (asserts CAP :1657/:1662 — résolvent en privé via co-localisation) ; bannière blob :1976-1979 (réécrite : elle abrite le test feed :1981-2035 qui part vers feed_api §6.2) + `blob_serve_returns_file_from_cached_zip` :2037-2132 + `remote_app_renders_via_p2p_fetch` :2133-2193 (2-nœuds iroh-networked, classe env-instable Docker-on-Windows) + `blob_serve_returns_404_for_unknown_hash` :2216-2231 + `blob_serve_rejects_path_traversal` :2232-2258 + `blob_serve_error_responses_have_csp` :2259-2303 + `blob_serve_csp_header_byte_exact_matches_contract` :2304-2360. **ÎLOT STAY** : `archive_hash_from_ticket_decodes_the_hash` :2195-2214 (suit §4.2 STAY).
- **→ diagnostic_api.rs (~129 l)** : bannière :2362-2364 + `diagnostic_neighborhood_returns_own_node_id_and_empty_peers` :2366-2389 ; **+ les 4 fairness router-driven** (handlers DÉJÀ dans diagnostic_api.rs) : :3284-3333 (`diagnostic_fairness_ok`, `diagnostic_fairness_ema_on_nonempty_ledger`) + :3430-3480 (`diagnostic_fairness_returns_500_on_corrupted_db`, `_on_poisoned_mutex`).
- **→ canary_api.rs (~128 l, LES 6)** : `canary_observed_post_ok` :2467-2498 + `canary_network_health_get_ok` :2500-2517 + bloc :2744-2820 (bannière + `canary_freshness_returns_200`, `canary_freshness_unknown_pubkey_returns_200`, **`canary_inject_rate_updates` :2780 + `canary_observed_divergence_empty` :2803** — dette orpheline pré-série : leurs handlers vivent DÉJÀ dans canary_api.rs).

### 6.2 EXTENSION PO-10 (adaptation n°2) — familles router-driven des domaines PRÉ-série (~1631 l), modules cibles TOUS EXISTANTS (vérifié disque)
Sans cette extension, http.rs post-S4 ≈ **3137-3262 l** (recompute exact 3 paires indépendantes) → le critère machine PO-10 `wc -l ~≤2500 TOTAL` (livrable plan :451-452 + memory PO « ~≤2.5k l TOTAL » + directive « 0 carry split différé ») **ÉCHOUE de ~640-760 l**. La combinaison minimale (deploy+consent) échoue aussi (2540, recompute critic) ; deploy+consent+files passe limite (~2354). La lecture cohérente de « S82 = une FIN » + discipline post-N2 « un domaine part avec TOUS ses tests, 0 test orphelin » = **sweep COMPLET** — laisser ~800+ l de tests orphelins de domaine dans http.rs serait EXACTEMENT un carry « split différé ». Census (spans vérifiés, arithmétique critic-corrigée) :

| Famille | Spans http.rs | ~l | Module cible |
|---|---|---|---|
| deploy + atelier-fork/workspace | :3482-3567 + :4077-4319 (incl. `post_workspace` :4177-4189, co-migre PRIVÉ) | 329 | deploy.rs |
| consent | :2523-2656 + :3706-3839 | 268 | consent.rs |
| files | :2658-2742 + :3841-3941 | 186 | files.rs |
| contributor (incl. `contributor_verify_rejects_non_hex_path_params` :1502-1519) | :1502-1519 + :2822-2896 + :3177-3234 | 151 | contributor_api.rs |
| tasks | :3040-3133 + :3402-3428 | 121 | tasks_api.rs |
| apps | :3569-3683 | 115 | apps.rs |
| invite | :2898-2985 | 88 | invite_api.rs |
| kudos | :3135-3175 + :3356-3400 | 86 | kudos_api.rs |
| storage | :3943-4002 | 60 | storage_api.rs |
| feed_insert (`feed_insert_rejects_without_internal_header` :1981-2035) | :1981-2035 | 55 | feed_api.rs |
| publish-cards | :4022-4075 | 54 | publish_api.rs |
| quarantine | :2987-3038 | 52 | quarantine_api.rs |
| shell_discover / coordinator_health / worker_state | :3258-3282 / :3236-3256 / :3335-3354 | 66 | shell_api.rs / health_api.rs / worker_state_api.rs |

### 6.3 Tests STAY (état final « tests core routeur », ~490 l)
Header+imports :1428-1433 ; `browse_index_rejects_open_source_without_provenance` :1435-1500 (direct-call sur helpers STAY) ; health/info/404 :1666-1754 ; origin/CORS :1756-1940 ; `project_info…` :1946-1974 ; SPA :2391-2465 (spa_fallback STAY per S2) ; `auth_token_returns_200_from_loopback` :3687-3704 ; `archive_hash_from_ticket_decodes_the_hash` :2195-2214 (§4.2).

### 6.4 Promotions test_support + fixtures
- **`browse_entries`** :4004-4020 : partagé entre famille deploy (movers → deploy.rs) et publish-cards (movers → publish_api.rs) → **promotion test_support.rs pub(crate)** (count-neutral, pattern O/Q/S3, verbatim SANS réécriture).
- **`post_workspace`** :4177-4189 : exclusif famille deploy → co-migre PRIVÉ (0 promotion). NB double-compte critic : il est DANS le span :4077-4319, ne pas le compter deux fois.
- **`BrowseListResponse`** :763-778 : STAY http.rs (foyer neutre S2 ; consommateurs browse_api+publish_api cross-module inchangés). 0 promotion.
- Famille **golden 9/9** (test_support.rs:351-669) : INTACTE, observateur externe, 0 edit — filet principal avec le count invariant.

---

## 7. HAZARDS IMPORTS (classe N2) + classe R

### 7.1 http.rs post-move — orphelins vérifiés par grep exhaustif (à confirmer sous `-D warnings`)
- :41 retirer `Path` de `axum::extract::{…}` (usages :1147 + :1400 partent tous deux, :1400 est full-path dans canary_freshness → seul :1147 compte, part).
- :43 `middleware::{self, Next}` → `middleware` (Next :655 part ; `from_fn*` reste :256/:606).
- :48 `nexus_core_rs::…BlobsClient` : usage prod unique :1175 part, MAIS le test STAY :2198 (archive_hash) l'utilise via `use super::*` → **déplacer l'import dans le bloc use du mod tests** (`use nexus_core_rs::BlobsClient;`), sinon unused en build non-test OU E0433 en test.
- :51 `blob_serve::{self, BlobServeCache}` → `BlobServeCache` seul (champ état :98 ; tous les `blob_serve::…` :660-669/:1156/:1264/:1277 partent).
- :59 `tracing::{debug, warn}` → `debug` seul (warn restants = `tracing::warn!` full-path :1074/:1096 dans index_browse_entry STAY).
- **Piège chaud `Deserialize` (:55)** : dernier user prod non-test = `NeighborhoodResponse` (mover) ; restant = `BrowseListResponse` `#[cfg(test)]` → build non-test = unused = FAIL. Fix : `use serde::Serialize;` + `#[cfg(test)] use serde::Deserialize;`.
- Après l'extension §6.2, re-balayer les imports du mod tests résiduel à la compile (population réduite à ~15 tests).

### 7.2 Modules destination
- **blob_serve_http.rs** (use-list prédite, couplage §3) : `Arc`, `SystemTime`, `axum::extract::{Path, Request, State}`, `StatusCode`, `axum::middleware::Next`, `axum::response::{IntoResponse, Json}`, `nexus_core_rs::BlobsClient`, `nexus_shell_daemon_core::blob_serve`, `tracing::{debug, warn}`, `crate::http::{DaemonHttpState, ErrorResponse}`. Full-path dans les corps verbatim : hex, serde_json, `iroh::EndpointId`, iroh_blobs, anyhow, `NodeDirectoryEntry`, `crate::seed_registry::SeedRegistry`, `crate::runtime::mint_ticket_for_hash`… non — mint STAY, cette réf est DANS mint qui reste. Tests : + `test_support::*`, `to_bytes`, `Method/Request`, `ServiceExt`, `KeyPair`/`create_node` (:2134), `DiscoveryClient`.
- **canary_api.rs** : + `use axum::response::IntoResponse;` (SEUL manquant, vérifié) ; tests + harness N2 complet (le module n'importe rien du harness aujourd'hui).
- **diagnostic_api.rs** : + `use serde::{Deserialize, Serialize};` (derives NeighborhoodResponse) ; tests idem.
- **seed_api.rs:22-25** (compiler-forcé, split) : `use crate::http::{DaemonHttpState, mint_blob_ticket};` + `use crate::blob_serve_http::{DIRECTORY_PULL_TIMEOUT_SECS, directory_pull_providers, find_directory_app_by_project};`. Le doc-link :853 `` [`DIRECTORY_PULL_TIMEOUT_SECS`] `` résout via le use re-pointé (0 action) ; doc :124 = prose backtick (0 action).
- **Familles §6.2** : chaque module cible reçoit le template harness (`use super::*;` réduit + `use crate::test_support::*;` + to_bytes/Method/Request/ServiceExt) — n'importer QUE l'utilisé.

### 7.3 Classe R (réactivée, prédiction AVANT compile)
0 bump R prédit (§5.6) — signatures des 7 movers vérifiées sans type privé. Vigilance aux call-sites re-pointés : si un `private_interfaces`/E0603 apparaît (précédent S3 : 1ʳᵉ compile non-parfaite de la série), amender le ledger in-phase AVANT commit.

---

## 8. PROJECTION `wc -l` vs CRITÈRE MACHINE PO-10

| Poste | Δ lignes |
|---|---|
| Base HEAD `0a32ffa` | 4322 |
| − prod movers §5 (447 spans − mint 17 − archive 12 = 418 ; + interstices ≈ 8) | ≈ −426 |
| − tests S4 §6.1 (cluster 144 + blob 304 [324−archive 20] + diagnostic 28 + canary 128) | ≈ −604 |
| − fairness §6.1 | −101 |
| − familles pré-série §6.2 | ≈ −1631 |
| − promotion `browse_entries` | −17 |
| − bannière orpheline :4321 | −2 |
| **http.rs post-S4 projeté** | **≈ 1540 (fourchette 1480-1610)** |

Décomposition attendue : production ≈ 995-1010 l (state+router+middlewares auth/CORS+origin+hub helpers) + tests core ≈ 490-560 l. **Critère PO-10 `~≤2500 TOTAL` : PASS avec marge.** Tension de lecture consignée (moveset §J) : kickoff:211 + design_review:97 lisent « prod < ~2500 » (déjà PASS trivialement à ~1000) mais le texte le plus récent et le plus spécifique (plan :451-452 livrable + memory PO 2026-07-15 + amendement PO-10 (b) :339-340 « ~≤2500 l TOTAL ») dit TOTAL → **TOTAL fait foi**, d'où l'adaptation n°2. Sans elle : ≈3140-3260 = FAIL.

Comptabilité invariante : count nextest **EXACT Win 2108 / Docker 2112** (moves purs, 0 test créé/supprimé, promotions count-neutral) ; web 412 / operator 201 ; goldens 9/9 ; 89 routes ; 0 wire ; 0 dep (Cargo.toml/lock INTACTS — tous les imports movers vérifiés présents).

---

## 9. INVARIANTS VERBATIM (sécurité) — conditions de move

1. **Layer CSP câblé au nest** `/blob-serve` de `public_routes` SANS bearer (risque n°1 de la phase) — filet : tests :2260/:2305 co-migrés MÊME commit + golden public_tier.
2. **panic_wipe** : `execute()` SYNCHRONE d'abord :1115 → 200 :1125 → `tokio::spawn`+sleep 100ms+`exit_only(0)` :1121-1123 ; commentaire :1117-1120 (« exit_only skips re-running execute », fix P2-B1 anti double-wipe) CO-MIGRE ; PAS de gate duress (intentionnel S20-B — ne pas « corriger »).
3. **blob_serve** : `validate_zip_path` AVANT tout accès :1156 ; ordre des 4 tiers :1176-1258 (preview → local → ticket → directory-only) ; re-lecture PAR LE HASH DEMANDÉ post-fetch :1248-1251 (BLAKE3 = vérité) ; 200 sans CSP inline :1279-1280.
4. **canary ×3** : mutex poisoned → 500 générique `{"error":"internal"}` SANS détail aux 3 sites :1366-1373/:1387-1394/:1411-1418 (anti info-leak).
5. **diagnostic_neighborhood** : peers = `subscribed_pubkeys_hex()` SEUL :1333 — ne jamais élargir ; doc :1323-1330 (2× vérifiée S23-E + S81-C) verbatim.
6. **Cluster** : cap 8 + timeout 120 + anchor-first Q5 + dedup/self-exclusion + docs menace §2 — tout dans le code, voyage tel quel.
7. Gate Factory `run_gate_csp_authoring` : INSENSIBLE (scanne le workspace d'app, jamais http.rs — vérifié gates.rs:527-580). `check-frontier-contracts.sh` ancré csp.rs.

---

## 10. RE-POINTS DOCS + GATES (liste fermée, critic-complétée)

**Gates scripts : les 3 VERTS post-S4 SANS édition** (vérifié par relecture intégrale) — `check-frontier-contracts.sh` (0 ancre http.rs), `check-sharding-docs.sh` (:90 satisfait par les routes shard qui RESTENT ; 0 token numérique `http.rs:<n>` dans les fichiers gated), `check-factory-docs.sh` (0 réf http.rs dans les 2 fichiers scannés). Hooks/CI/acceptance : 0 réf.

**Re-points à faire (honnêteté doc, leçon N/S2-F2)** :
| # | Fichier:ligne | Action | Condition |
|---|---|---|---|
| 1 | `canary_api.rs:2-5` | réécrire header (« already in http.rs » devient faux) | OBLIGATOIRE |
| 2 | `diagnostic_api.rs:2-5` | élargir charte (fairness + neighborhood) | OBLIGATOIRE |
| 3 | `docs/rust/PATTERNS.md:4159` | `http.rs:directory_pull_providers` → `blob_serve_http.rs:…` | cluster MOVE |
| 4 | `docs/rust/PATTERNS.md:4196` | test byte-exact → `blob_serve_http.rs::…` | test co-migre |
| 5 | `docs/rust/PATTERNS.md:4211` | nest (`http.rs` `blob_serve_csp_middleware`) → `blob_serve_http.rs` | middleware MOVE |
| 6 | `docs/security/THREAT_MODEL.md:19` | « handler `blob_serve_http.rs`, route registrée dans `http.rs` » | blob_serve MOVE |
| 7 | `docs/security/THREAT_MODEL.md:776` | `http.rs` → `blob_serve_http.rs` | middleware MOVE |
| 8 | `docs/factory/FACTORY_GATES.md:190` | « nextest byte-exact (`http.rs`…) » → `blob_serve_http.rs` | test co-migre |
| 9 | `web/e2e/app-authoring.spec.ts:28` | réf test → `blob_serve_http.rs::…` (:14 = prose symbole sans path, inventoriée, 0 action) | test co-migre |
| 10 | `nexus-shell-daemon-core/src/browse.rs:762-764` | « lives daemon-side (`http.rs` …) » → `blob_serve_http.rs` (cross-crate, critic-catch ×3) | cluster MOVE |
| 11 | `nexus-shell-daemon-core/src/blob_serve.rs:282` | réf `http.rs:556` DÉJÀ stale → forme symbole sans numéro | opportuniste |
| 12 | `test_support.rs:702` + `:846-848` | « staying http.rs fork/pull-resolution » + « staying http.rs deploy/… tests » deviennent faux avec §6.2 → re-word | sweep |
| 13 | `http.rs:1058` (doc index_browse_entry) | « gated at http.rs:934 » doublement stale (publish gate → publish_api Phase S) → fix opportuniste | opportuniste |
| 14 | bannière tests :1976-1979 | réécrite/scindée (blob part, feed part) | sweep |

**0 re-point (preuves négatives)** : THREAT_MODEL:1014 (`build_router` STAY) ; LOOPBACK:76/:83/:257 (paths + nom de const sans ancre) ; WIRING_SPEC sharding:144/:165 + llms.txt:37 (`authed_routes` STAY) ; publish_api.rs:20-21 (reste VRAI avec mint STAY) ; runtime.rs:2521+:2282 (mint/archive STAY) ; seed_api.rs:124/:853 (résolvent via use) ; shell PATTERNS:1215/:1249 (entrées SHA-historiques, convention série) ; web/src 0 impact (paths seuls, 0 usage canary/diagnostic).

---

## 11. Arbitrage scan-vs-critic (corrections matérielles retenues)

1. **moveset (ADJUSTED)** : + re-point runtime.rs:2521 si mint MOVE (rendu SANS OBJET par l'arbitrage STAY §4.2) ; span :1435-1519 = DEUX tests (browse_index + contributor_verify :1502-1519 → ce dernier part avec la famille contributor §6.2) ; compte seed_api « 4 symboles » présupposait mint MOVE — corrigé à 3 + mint reste `crate::http::`.
2. **tests (ADJUSTED)** : double-compte `post_workspace` 15 l corrigé (deploy = 329, sweep = 1631, pas 1646) ; **combinaison minimale deploy+consent = 2540 > 2500 ÉCHOUE** → renforce l'adaptation sweep ; re-point browse.rs:762-764 ajouté ; nits spans (project_info 29 l, auth_token 18 l, `health_returns_200_with_fixed_shape`).
3. **couplage (ADJUSTED)** : browse.rs:762-764 cite EXPLICITEMENT `http.rs` → re-point OBLIGATOIRE (pas conditionnel) ; canary_api.rs = 104 l ; tests migrants ~616 vs ~610 (immatériel).
4. **history (ADJUSTED)** : LOOPBACK:257 = scope nonce T2 CONFIRM (pas « rotation inopérante ») ; double-compte canary :2780-2824 corrigé (comptés UNE fois, au titre §6.1 canary) ; sourcing P2-B1 = body commit `c32ecb3` (le commentaire disque encode le mécanisme sans nommer P2-B1) ; nits off-by-1 (NeighborhoodResponse :783, ErrorResponse :791, middleware :655-681).
5. **threat (ADJUSTED)** : + les 2 tests canary orphelins :2780/:2803 au lot co-migrant (repris §6.1) ; runtime.rs:2282 = call-site CODE si archive MOVE (SANS OBJET, STAY) ; usages seed_api :185/:452 complétés ; :773 = BrowseListResponse consigné (§1, 0 implication sécurité).
6. **docsgates (ADJUSTED)** : :934 = fin de doc de `find_directory_app_by_hash` (pas doc CAP) — le point substantiel (réf :1058 stale) tient ; + spec.ts:14 et seed_api.rs:124 à l'inventaire (0 action).

Aucune correction ne déplace un symbole hors des arbitrages §2-4 ni ne force un bump SHARED. Convergence totale sur : ancres exactes, 7 ROUTINE/0 SHARED, dépassement PO-10 sans extension, 3 gates verts, 0 wire/0 dep.

---

## 12. Pièges ACTIVÉS (classes standing re-confirmées pour S4)

- **doc_lazy_continuation** : 0 ligne `//!`/`///` commençant par `+` dans http.rs AUJOURD'HUI (grep vide), MAIS les docs cluster :978-995 contiennent des listes ET les headers réécrits (canary_api/diagnostic_api/blob_serve_http NEUF) sont du texte NEUF → vérifier au clippy (piège S3 : ligne `//!` commençant par « + »).
- **N2 imports** : §7 — les destinations canary_api/diagnostic_api n'ont AUCUN import harness aujourd'hui ; blob_serve_http part de zéro ; chaque famille §6.2 réplique le template S2 (browse_api.rs:286-294).
- **R reachability** : §5.6/§7.3 — prédiction 0, amender AVANT commit si la compile parle.
- **S2-F2 docs STAY → symboles mouvants** : re-grep bidirectionnel post-move OBLIGATOIRE (liens ``[`Symbol`]`` en cfg(test) invisibles à cargo doc ; état disque : 0 lien dans le mod tests, 1 seul lien prod concerné = runtime.rs:2521 neutralisé par mint-STAY).
- **Bornes par NOM** : toute extraction (prod ET tests) par NOM, jamais par plage — les spans ci-dessus sont des aides de lecture, pas des ordres de découpe.
- **Faux-négatif grep multi-ligne** (leçon P) : routes multi-lignes — vérifié, les 7 re-points :255-446 sont mono-ligne, mais re-vérifier au diff.
- Standing ops : Docker `/workspace` + `MSYS_NO_PATHCONV=1` + `bash -c` ; pipefail ; gros cargo en background ; `SBFB_TEST_HTTP_TIMEOUT_SECS=120` Docker-on-Windows ; `remote_app_renders_via_p2p_fetch` env-instable → re-run solo avant de conclure régression ; codex `--sandbox read-only` ; TOKEN_IDENTICAL par tranche (exceptions DÉCLARÉES : bannière :4321 supprimée, headers modules réécrits, bannière :1976-1979 scindée, qualifieurs de visibilité).

---

## 13. Risques résiduels

1. **Ampleur** : S4 étendu = la plus grosse phase de la série (~430 l prod + ~2350 l tests vers 3+15 modules). Mitigé : pattern prouvé 9×, harness test_support partagé, count invariant EXACT comme oracle dur, goldens 9/9, extraction par NOM. Si la phase déborde, le repli ORDONNÉ est : S4-core (arbitrages §2-4) + familles par rendement (deploy 329 → consent 268 → files 186 → …) jusqu'à `wc -l ≤2500` ; MAIS l'état cible reste le sweep complet (0 test orphelin) — tout repli partiel doit être consigné comme déviation, pas silencieux.
2. **Dé-câblage silencieux du layer CSP** : neutralisé par co-migration des 2 tests témoins DANS LE MÊME COMMIT + golden.
3. **P2 pré-existants consignés** (Codex S2/S3) : 0 golden feed/search/provenance/preview/browse/nodes — inchangé par S4, à re-consigner au commit body.
4. Compile non-parfaite possible (précédent S3) : la prédiction bump/imports est grep-vérifiée mais la vérité est au compilateur — amender le préflight in-phase, discipline établie.

---

## 14. Verdict final : **PLAN-ADAPT** — adaptations numérotées

1. **`mint_blob_ticket` + `archive_hash_from_ticket` STAY http.rs** (déviation du défaut plan « blob_serve_http.rs ») — evidence : 0 consommateur co-mover (blob_serve ne les appelle pas), 3 consommateurs producteurs multi-domaines, doc-contrat publish_api.rs:20-21 Phase S resterait VRAI, symétrie classe hub `wrap_payload_with_pow`. Économise 5 re-points dont le doc-link runtime.rs:2521.
2. **Co-migration ÉTENDUE aux familles de tests router-driven pré-série** (§6.2, ~1631 l + fairness 101 l, 15 modules cibles existants) — REQUISE par le critère machine PO-10 `wc -l http.rs ~≤2500 TOTAL` (sans elle : ~3140-3260 = FAIL) et par « 0 carry split différé ». Promotion test_support : `browse_entries`.
3. **Contradiction middleware CSP tranchée côté Goal = MOVE** ; état final du plan reformulé « middlewares auth/CORS + helpers origin » (la clause « si déplaçable proprement » du Goal est satisfaite avec evidence §3 ; le layer reste câblé au nest public, filet byte-exact double).
4. **Cluster pull MOVE → blob_serve_http.rs** (réponse à la question routée par S2 : OUI, il suit blob_serve) — le prédicat du STAY S2 est inversé ; STAY créerait l'unique bump SHARED de la phase. `find_directory_app_by_hash` (omis du plan) co-migre PRIVÉ ; édit seed_api.rs:22-25 = re-point PLANIFIÉ (l'ancien garde-fou « STOP si edit seed_api » était l'oracle du verdict S2, caduc).
5. **Goal partiellement stale** : `default_curators`/`publish_blob` DÉJÀ dispatchés (R/S) — 0 action ; retrait bannière orpheline :4321 confirmé ; réécritures headers canary_api/diagnostic_api + bannières = deltas non-move DÉCLARÉS.

### Checklist compile-hazard (AVANT 1ᵉʳ build)
1. 7 bumps ROUTINE (§5.6) ; `mod blob_serve_http;` main.rs entre :32/:33 ; 7 re-points routes full-path (§5.5).
2. Split use seed_api.rs:22-25 (§7.2) — mint reste `crate::http::`.
3. Orphelins http.rs : Path/Next/BlobsClient(→mod tests)/blob_serve::self/warn + split Serialize / cfg(test) Deserialize (§7.1).
4. Promotion `browse_entries` verbatim ; `post_workspace` co-migre privé ; goldens/test_support intacts par ailleurs.
5. Re-points docs #1-14 (§10) dans la MÊME phase ; re-grep S2-F2 bidirectionnel post-move.
6. STAY à NE PAS balayer : mint/archive + hub helpers + BrowseListResponse + browse_index/origin/CORS/SPA/health/info/project_info/auth_token/archive-test.
7. Si compile réclame un bump SHARED ou un edit hors liste (seed_api/browse.rs/docs §10) : STOP, re-vérifier l'arbitrage.
8. Gates D4 : fmt/clippy `-D warnings`/nextest Win 2108/doc/release + Docker 2112 + 3 gates docs + `wc -l http.rs` ≤2500 ASSERTÉ au commit body + review Workflow + Codex Sol.

---

## 15. Amendements in-phase (implémentation, discipline précédents O/S3)

1. **`contributor_dashboard_*` (:3177-3234) → `kudos_api.rs`, PAS `contributor_api.rs`** —
   la table §6.2 les rangeait sous « contributor » par nom de ressource, mais la
   discipline de la série est tests-follow-handler : la route
   `/api/v1/contributor/{node_id}` pointe `crate::kudos_api::contributor_dashboard`
   (http.rs:565-568 pré-move, S76 Phase E D4), et dans http.rs ces 2 tests suivaient
   le bloc kudos SANS bannière propre (contiguïté du fichier). Delta comptable :
   contributor 151→93 l, kudos 86→144 l — total inchangé.
2. **`mod blob_serve_http;` dans main.rs : entre `apps` et `browse_api`** — §5.1
   disait « entre browse_api et canary_api », mais `blob` < `brow` en ordre
   alphabétique ; rustfmt (`reorder_modules` défaut) aurait déplacé la ligne.
3. **Bannières de section supprimées (deltas non-move DÉCLARÉS, complément §12)** :
   « Sprint 46 Phase A — integration tests 12 MANDATORY routes » (:2519-2521) et
   « Sprint 46 Phase B — integration tests 14 recent routes + debt » (:2898-2900)
   couvraient des familles éclatées vers 8+ modules chacune — orphelines de sens,
   retirées. La bannière « Pagination tests (debt items 2 + 4) » (:3356-3357)
   co-migre avec le test kudos qu'elle précède ; `tasks_list_with_limit` arrive
   dans tasks_api.rs sans bannière (état verbatim : il n'en avait pas en propre).
4. **2 re-points docs SUPPLÉMENTAIRES attrapés par le re-grep bidirectionnel
   post-move (§12, leçon S2-F2)** — absents de la liste fermée §10 :
   `nexus-core-rs/src/csp.rs:9` (consommateur runtime cité « http.rs » →
   `blob_serve_http.rs`) et la charte `browse_api.rs:19-23` (« the
   directory-only pull-resolution cluster … stay in http.rs » devenu FAUX
   avec l'arbitrage cluster-MOVE §2 — re-wordée : BrowseListResponse +
   chokepoint restent, le cluster a suivi blob_serve). La liste §10 passe de
   14 à 16 re-points.
5. **Post-review (P3 review Workflow)** : 3e re-word test_support manqué APPLIQUÉ
   (`make_test_submission` :821-823, « staying http.rs tasks_api tests » →
   tasks_api.rs S4) ; knowledge pack
   `examples/daisyui-animejs-showcase/knowledge/factory-integration-hardened.md`
   = **EXEMPTION EXPLICITE** (artefact de recherche historique S79-era aux refs
   déjà multi-stales pré-S4 [http.rs:551-577/:234, blob_serve.rs:286] et dont
   l'« Action » CSP a été réalisée en S79-E — classe SPRINT_LOG/entrées
   SHA-historiques, on ne re-pointe pas un document daté) ; preuve
   TOKEN_IDENTICAL persistée (`scratchpad/s4/proof/token_identical_report.txt`,
   43/43 + sha256 snapshot==HEAD) ; compta diagnostic_api corrigée : 1 test
   neighborhood + 4 fairness = 5 (pas « 3+4 »).
6. **Post-Codex (round 1 PASS WITH NOTES, 0 P0/P1)** : preuve TOKEN_IDENTICAL copiée
   REPO-VISIBLE → `.planning/active/sprint82_phase_s4_token_proof.txt` (le chemin
   scratchpad session du point 5 n'était pas lisible d'un auditeur externe) ; bannière
   « SNAPSHOT HISTORIQUE » posée EN TÊTE du knowledge pack
   `factory-integration-hardened.md` (l'exemption devient visible depuis l'artefact
   lui-même) ; re-point opportuniste dette pré-existante Phase N
   `docs/sharding/examples/observe.curl.md:14` → `shard_session_http_api.rs` ;
   ledger wording : 6 routes + 1 layer middleware = 7 re-points ; deploy reçoit 11
   tests (le prompt Codex disait 9 — le body est au 11 exact) ; P2 pré-existants
   consignés par Codex : goldens absents feed/search/provenance/preview/proof-card/
   browse/nodes + famille dispatch_loop candidate test-group nextest borné (flake
   parallélisme défaut, hors diff S4).
