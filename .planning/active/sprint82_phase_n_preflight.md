# Sprint 82 Phase N — Preflight G8 (split http.rs : domaine shard-session → `shard_session_http_api.rs`)

## Contexte + méthode (Workflow multi-agents, 2026-07-15)

- HEAD au preflight : `29a9255` (S82 Phase M — golden HTTP + dédup harness). Phases A→M DONE (13/20).
- Orchestration : Workflow ultracode `wf_09f48a9f-ce8` — **51 agents** (5 scans G8 + cartographie
  exhaustive, chaque claim vérifié par un réfuteur adversarial indépendant, critic de complétude
  final). 0 agent en erreur.
- Mission : 1ᵉʳ split de http.rs (13130 l au HEAD) — extraire le domaine shard-session http vers
  `crates/nexus-shell-daemon/src/shard_session_http_api.rs`, discipline commune N→S
  (co-déplacer handler + DTO + tests SANS orphelin ; route inchangée ; golden Phase M vert
  post-split ; 0 route path, 0 bump wire ; count nextest invariant).
- Toutes les coordonnées ci-dessous sont **re-dérivées par NOM sur le HEAD actuel** (consigne
  mémoire : le fichier a bougé de +183 l depuis le kickoff ; les coords du plan sont stales).

## Dérives plan à consigner (vérifiées disque, HEAD `29a9255`)

1. **Bornes région STALES sur les DEUX bouts** : plan §Phase N dit `http.rs:2154-2509` ; la région
   réelle = **`http.rs:2129-2453`**. Début 2129 = bordure haute `// ====` du bandeau de domaine
   (2129-2131 bandeau, 2133-2147 doc S77-L/S81-I/S82-G, 2148-2151 `use nexus_core_rs::{…}`) —
   commencer à 2154 orphelinerait le bandeau et le bloc `use`. Fin 2453 = accolade fermante de
   `shard_session_drop_shard` ; 2470-2509 appartient DÉJÀ au domaine seed (Phase O,
   `SeedVoluntaryRequest` :2470-2471) — le plan déborde de ~56 l dans le domaine voisin.
2. **« 6 handlers » → 8 fns à déplacer** : 6 handlers async (`shard_session` :2185,
   `shard_session_group` :2205, `shard_session_mount` :2254, `shard_session_generate` :2301,
   `shard_session_result` :2415, `shard_session_drop_shard` :2436) + **2 projections pures
   privées** (`shard_session_response` :2158, `shard_session_result_response` :2380) ; 0 intrus
   dans la région (aucun struct/const/impl).
3. **« DTO co-déplacés » quasi-N/A** : AUCUN DTO daemon-local n'existe dans la région. Les 6 DTO
   wire vivent déjà dans `nexus-core-rs/src/schemas/shard.rs` (depuis S77-L/S82-G) ;
   `MountSessionRequest` vit dans `crate::shard_session`. Le « co-déplacement DTO » se réduit à
   relocaliser le bloc `use nexus_core_rs::{ShardGenerateRequest, ShardGroupMintRequest,
   ShardSessionResultResponse, ShardSessionResultView, ShardSessionStatusResponse,
   ShardSessionView}` (:2148-2151).
4. **`frontier_closure: N/A` INCOMPLET — livrable MANQUÉ, CI-BLOQUANT** (catch du critic, raté
   par les 5 scans individuels) : `scripts/check-sharding-docs.sh` (câblé CI :
   `.woodpecker/ci-linux.yml:87`, `.github/workflows/ci.yml:132`, `scripts/verify.sh:95` ;
   baseline CLEAN au HEAD) fait `grep -qF <Symbol> <file>` pour chaque source_ref `path:Symbol`.
   **6 refs pointent `crates/nexus-shell-daemon/src/http.rs:shard_session_response`**
   (5 dans `docs/sharding/WIRING_SPEC.md` ~:139/:140/:155/:159/:198 + 1 dans
   `docs/sharding/llms.txt:37`). Le move sort TOUTES les occurrences littérales de
   `shard_session_response` de http.rs → gate ROUGE → CI + verify.sh step 14 ROUGES.
   Phase N DOIT re-pointer ces 6 refs vers `shard_session_http_api.rs:shard_session_response`
   (+ re-pointer par honnêteté `http.rs:shard_session` WIRING_SPEC:166, qui survivrait au grep
   par substring mais deviendrait mensonger ; laisser `http.rs:authed_routes` ×2 intact —
   `authed_routes`/`build_router` restent dans http.rs). Le check REQUIRED_ANCHORS (:214)
   continue de passer (le tail `shard_session_response` est préservé quand seul le path change).
   L'analogie « docs = 0 édit comme Phase L » est INVALIDE : L décomposait DANS le fichier
   (ancres file:symbol tenaient), N DÉPLACE les symboles vers un fichier NEUF (chaque ancre
   file:symbol casse). Canon S80 `a6b4ca4` : le test de frontière est le TEST-ACTEUR, jamais
   « 0 wire bump ».
5. **T1 du plan incomplet** : la liste de gates N→S (« golden vert + nextest invariant +
   fmt/clippy ») omet `bash scripts/check-sharding-docs.sh` — à ajouter explicitement à
   l'oracle T1 de Phase N (c'est le seul gate docs que le move casse).
6. **Visibilité non spécifiée par le plan** : les 6 handlers sont aujourd'hui `async fn` privés,
   référencés uniquement par `build_router` en bare-name → doivent devenir **`pub(crate) async
   fn`** (PAS `pub`). Les 2 projections RESTENT privées (tous leurs appelants — handlers
   :2192/:2422 ET les 2 tests unitaires — co-migrent). `DaemonHttpState` (:75) et son champ
   `shard_sessions` (:201) sont déjà `pub` : 0 autre bump.
7. Mineur, non porteur : la note de discipline du plan cite le module tests « 4546-12460
   (~7915 l) » — réel : `mod tests` ouvre à :4528 (`use super::*` :4530) et s'étend au-delà de
   12460 (golden :12827 dedans). Confirme le drift généralisé des coords pré-M.

## S1a — Modularisation (gabarit `*_api.rs` établi, crate binaire pur)

- `nexus-shell-daemon` est un **crate binaire pur** (Cargo.toml:12-14 `[[bin]]` seul, aucun
  `[lib]`, aucun lib.rs). Modules déclarés à plat dans `main.rs:31-63` = 31 déclarations de
  modules-fichiers (dont 2 cfg-gated : `named_pipe_server` :48 windows, `uds_server` :61 unix)
  + 1 mod test inline `handler_tests` :1193. Insérer `mod shard_session_http_api;` (après
  `mod shard_session;` :56) ; la résolution frère `crate::shard_session_http_api::<handler>`
  est prouvée in-vivo (`crate::shard_session::parse_pubkey_hex` à main.rs:412).
- **Phase N est le 1ᵉʳ split de http.rs, mais le PATTERN cible existe déjà : 11 modules
  `*_api.rs` frères** (tasks/storage/shell/contributor/canary/health/quarantine/worker_state/
  invite/diagnostic/kudos). Gabarit vérifié 11/11 : `use crate::http::DaemonHttpState;`
  (contributor_api.rs:13…), handlers `pub async fn`, `#[cfg(test)] mod tests` propre,
  build_router les référence en CHEMIN COMPLET sans `use` (http.rs:344
  `crate::contributor_api::verify_contributor`…). Nuance Phase N : `pub(crate)` suffit
  (cohérent avec l'usage intra-crate ; les 11 modules historiques utilisent `pub`, mais rien
  n'exige `pub` ici — voir §Approche, décision D-N1).
- Imports du nouveau module (calqués contributor_api.rs) : `std::sync::Arc`,
  `axum::extract::{Path, State}`, `axum::response::{IntoResponse, Json, Response}`,
  `axum::http::StatusCode`, `tracing::debug`, `crate::http::DaemonHttpState`, + le bloc
  `use nexus_core_rs::{…}` co-déplacé. `serde_json::json!`, `tokio::spawn`,
  `crate::shard_session::*`, `crate::noop_identity::*` restent en full-path (comme aujourd'hui).
- Tous les items `crate::shard_session` consommés sont déjà `pub` (DEFAULT_MAX_NEW_TOKENS :152,
  MountSessionRequest :228, ShardSessionStatus :269, ShardSessionRegistry :379,
  mint_compute_group :586, mount_session :729, generate_session :916, parse_pubkey_hex :1696)
  — 0 bump côté logique.

## S1b — Deps (0 dep, 0 feature, 0 delta lock)

- Move intra-crate pur : toutes les crates référencées par la région sont déjà des deps
  déclarées de nexus-shell-daemon. **0 dépendance, 0 feature Cargo, 0 delta Cargo.lock.**
- http.rs est entièrement feature-gate-free (seuls `#[cfg(test)]` :739 et :4528 ; 0
  `#[cfg(feature=…)]` sur 13130 l). Les gates `llm_llama_cpp` vivent dans nexus-worker-core,
  PAS dans shard_session.rs (claim initial réfuté, conclusion inchangée).
- `shard_session.rs` (logique) et `nexus-core-rs/schemas/shard.rs` (schémas) ne bougent PAS.

## S2 — Décisions historiques (S77-J, S81-I/J, S82-M)

- Corpus traversé : bodies `66259c6` (S77 J), S81 I/J, `29a9255` (M) + preflight/review M.
- Body Phase M : golden = **filet conçu explicitement pour les splits N→S** (commentaire
  http.rs:12662-12670) ; « golden vert 2× sur HEAD inchangé AVANT dédup » déjà honoré ;
  redaction allowlist minimale-observée ; oracle EXACT (== baseline+N, jamais >=).
- Le preflight M disait déjà « 8 fns » (plus exact que le plan) mais ses bornes 2158-2490
  étaient aussi imprécises (omettaient bandeau + use).
- Review M : P3 M-1..M-7 documentés — l'échantillonneur golden assume ~11/24 handlers sans case
  dédiée ; la couverture des 4 handlers shard non-golden est portée par le test duress + les
  2 tests de projection co-migrants (compensation compilateur + tests HTTP).
- **Oracle N hérité : delta 0 EXACT** (move pur) → Win 2108 / Docker 2112 invariants ; golden
  9/9 vert post-split ; 0 wire bump préservé verbatim.

## S3 — Threat model (move sécurité-neutre par construction)

- THREAT_MODEL (v17/v18) : les 4 propriétés portées par la couche http shard-session vivent
  HORS des handlers déplacés. La **privacy whitelist SI-3/SI-4 EST le type-shape**
  (`ShardSessionView`/`ShardSessionResultView`, nexus-core-rs/schemas/shard.rs:83-97, non
  touché) ; les projections sont la « privacy seam » (commentaire http.rs:2137-2139, test
  :5579) et déménagent verbatim.
- **Auth = niveau routeur** (bearer X-SBFB-Token + Host + Origin sur `authed_routes`, layer
  :529) — jamais dans les handlers. Le move est auth-neutre : les 6 routes restent inscrites
  dans authed_routes, seule la référence de fn change.
- **Duress = inline en tête de chaque handler mutant** (group/mount/generate) → se déplace
  VERBATIM avec la fn. Aucun chemin duress ne dépend du fichier.
- LOOPBACK_ENDPOINTS_TRUST_TIERS.md : tiers par route (T0 lectures/drive, T1-candidat
  group/mount) attachés au PATH (inchangé) — préserver les commentaires de tier verbatim dans
  le module extrait.
- Aucune logique de sécurité locale dans les handlers (mint/mount/generate/parse_pubkey_hex
  vivent dans `crate::shard_session`, non touché). `pub(crate)` n'élargit rien hors-crate.
- Filet : golden Phase M + test duress (tous deux PATH-agnostiques, pilotés routeur) + clippy.

## S4 — Wire format (0-wire PROUVÉ, frontière = DOCS pas web)

- Les 6 routes vivent dans `authed_routes` DANS `pub fn build_router` — 0 path modifié.
- **3 consommateurs runtime** tracés : web (`daemon.ts` getShardSession GET-status seul +
  ShardSessionPanel), CLI (`main.rs:468-553` shard_api_call, 6 routes reqwest loopback),
  script acceptance (`b3_shard_pipeline.sh` status/generate/result/drop-shard) + e2e
  compute-shard.spec.ts (via page /compute). **Tous couplés path+shape JSON, JAMAIS au nom du
  handler Rust** → move-safe.
- Fixtures golden : INLINE (`serde_json::json!` dans GoldenCase), pas sur disque — preuve
  mécanique embarquée dans la suite.
- 0 canonical / FEED_FORMAT_VERSION / *_ANNOUNCEMENT_VERSION dans la région. Schéma Zod web
  inchangé (lit nexus-core-rs shapes).
- frontier_closure côté web/CLI : N/A (aucune signature DTO touchée). **La SEULE frontière
  touchée est le docs-contrat sharding** (§Dérives pt 4) — consommateur = le gate
  check-sharding-docs.sh + les lecteurs de WIRING_SPEC/llms.txt.

## Vérification adversariale — synthèse

- 51 agents : chaque claim des 5 scans challengé par un réfuteur indépendant (re-dérivation,
  jamais recopie). 7 claims REFUTED, tous corrigés à la source dans ce document :
  1. « 33 mod dans main.rs » → 31 modules-fichiers (+1 inline) ;
  2. bornes début 2130 → **2129** (bordure `// ====` sinon orpheline) ;
  3. « privacy whitelist = les 2 projections » → whitelist = TYPE-SHAPE (core), projections =
     seam (conclusion sécurité inchangée) ;
  4. « projections référencées seulement par les tests » → aussi par leurs handlers
     (:2192/:2422 ; conclusion « restent privées » inchangée) ;
  5. « gates llm_llama_cpp dans shard_session.rs » → nexus-worker-core (0 feature requis,
     inchangé) ;
  6. « ~20 domaines golden » → exactement 9 `golden_http_*` (7 appellent golden_run) ;
  7. « duress test co-migre » (discipline lue trop littéralement) → il RESTE (voir décision).
- Catch décisif du CRITIC (raté par les 5 scans) : le gate `check-sharding-docs.sh` casse au
  move (§Dérives pt 4). C'est LE delta qui fait basculer le verdict en PLAN-ADAPT.

## Approche d'implémentation

### Décisions de design tranchées

- **D-N1 (visibilité)** : handlers → `pub(crate) async fn` (pas `pub` — aucun consommateur
  hors-crate ; crate binaire pur donc équivalent en pratique, `pub(crate)` documente l'intention).
  Projections → restent `fn` privées (tous les appelants co-migrent).
- **D-N2 (golden RESTE)** : `golden_http_shard_session_domain` (:12827) RESTE dans http.rs —
  unanime 5 scans + critic. C'est 1 des 9 `golden_http_*` partageant le harness privé
  cross-domaine (GOLDEN_VOLATILE_FIELDS/golden_redact/GoldenBody/GoldenCase/golden_check/
  golden_run :12677-12778), conçu Phase M comme LE filet des splits N→S ; il pilote par URI
  via build_test_router→build_router, ne nomme aucun symbole déplacé → 0 orphelin en restant.
  Le déplacer forcerait un export pub(crate)-under-cfg(test) du harness et fragmenterait la
  famille golden. « Golden Phase M vert post-split » exige un observateur SINGULIER stable
  sur les 6 phases N→S.
- **D-N3 (duress RESTE)** : `shard_session_routes_noop_in_duress` (:6717-6808) même règle —
  100 % piloté HTTP (`.uri(…)` :6730/:6771/:6792) + harness privé (mk_state_with_mode :4698,
  build_test_router :4677), ne référence AUCUNE fn déplacée → 0 orphelin en restant. La
  discipline « co-déplacer les tests » se lit sémantiquement : co-migrer ce qui référence les
  symboles déplacés (sinon orphelin), garder les tests router-driving avec leur harness.
- **D-N4 (tests qui MIGRENT)** : exactement 2 — `shard_session_response_pins_empty_envelope`
  (:5539-5575) et `shard_session_projection_hides_member_identities` (:5578-5687). Ils
  appellent DIRECTEMENT les projections privées (:5553/:5643, :5568/:5677) → s'orphelinent
  sinon. 0 dépendance harness (construction manuelle de ShardSessionRegistry) → un
  `#[cfg(test)] mod tests { use super::*; … }` suffit dans le nouveau fichier.

### Séquence

1. Créer `crates/nexus-shell-daemon/src/shard_session_http_api.rs` (imports §S1a).
2. Déplacer la région production **http.rs:2129-2453 VERBATIM** (bandeau inclus, fin avant le
   domaine seed) ; handlers en `pub(crate)`, projections privées.
3. Déplacer les 2 tests de projection dans le `mod tests` du nouveau fichier.
4. `main.rs` : ajouter `mod shard_session_http_api;` après `mod shard_session;` (:56).
5. `build_router` (:315-336) : re-pointer les 6 refs de handler vers
   `crate::shard_session_http_api::<handler>` — strings de route INTOUCHÉES ; authed_routes,
   champ `shard_sessions` (:196-201), builders test (:4778/:7928) restent.
6. **Docs-contrat** : re-pointer les 6 source_refs `http.rs:shard_session_response` →
   `shard_session_http_api.rs:shard_session_response` (5 × WIRING_SPEC.md + 1 × llms.txt:37)
   + `http.rs:shard_session` (WIRING_SPEC:166) ; laisser `http.rs:authed_routes` ×2.
7. Full fail-fast avant commit (pièges L/M : `set -o pipefail`, jamais `| tail` nu, gros jobs
   cargo en avant-plan séquentiel) : fmt + clippy -D warnings + **check-sharding-docs.sh
   VERT** + nextest count == 2108 Win (Docker 2112 au gate dual-platform) + golden vert +
   web/suites intactes.
8. Commit `refactor(daemon): Sprint 82 Phase N — split shard-session http domain to
   shard_session_http_api.rs (0 wire bump)`.

## Oracle T1 précis

- `cargo nextest run --workspace --locked` : count **== 2108 Win EXACT** (delta 0 — move pur,
  les édits docs n'ajoutent aucun test) ; Docker sbfb-ci **== 2112** au gate dual-platform.
- `golden_http_shard_session_domain` + les 8 autres golden : VERTS post-split (9/9).
- `shard_session_routes_noop_in_duress` + 2 tests projection (nouveau path
  `shard_session_http_api::tests::…`) : VERTS.
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0.
- **`bash scripts/check-sharding-docs.sh` : VERT** (les 6 source_refs re-pointés ; le gate
  échouerait à coup sûr sans l'étape 6 — c'est le critère machine ajouté par ce preflight).
- 0 delta Cargo.lock ; 0 route path ; git diff docs limité aux 7 refs re-pointés.
- T2 = N-A (plan).

## Verdict: PLAN-ADAPT

Approche du plan CONFIRMÉE (move pur intra-crate sur gabarit `*_api.rs` établi 11×, 0 décision
Day-0/PO violée, 0 wire, 0 dep) — mais le plan porte des faits périmés/faux à corriger :
bornes `2154-2509` → **2129-2453** (débordait dans le domaine seed) ; « 6 handlers » →
**8 fns + 1 bloc use + 2 tests de projection** ; « DTO co-déplacés » → aucun DTO local
(1 ligne use) ; et surtout **`frontier_closure: N/A` est FAUX** : le gate CI
`check-sharding-docs.sh` casse au move (6 source_refs file:symbol dans docs/sharding/) —
livrable docs-contrat ajouté in-phase + gate ajouté à l'oracle T1. Golden et duress RESTENT
dans http.rs (router-driven, harness-couplés) ; seuls les 2 tests de projection migrent.
Le code suit l'approche corrigée ci-dessus, pas les coordonnées du plan.
