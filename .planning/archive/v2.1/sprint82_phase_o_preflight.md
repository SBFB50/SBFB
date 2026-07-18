# Sprint 82 Phase O — Preflight G8 (split domaine seed → `seed_api.rs`, discipline étendue PO-10)

## Contexte + méthode (Workflow multi-agents, 2026-07-16)

- HEAD au preflight : `c5be6e4` (Phase N2 test_support DONE). Phases A→N2 DONE (15/24).
- Orchestration : Workflow ultracode `wf_f39c4c99-f46` — **55 agents** (5 scans G8 dont
  périmètre/cohésion dédié, réfuteur adversarial par claim, critic final). 0 agent en erreur.
- Mission : split du domaine seed de http.rs (11903 l) vers `seed_api.rs` — **1ᵉʳ split sous
  discipline ÉTENDUE PO-10** (TOUS les tests du domaine co-migrent via `crate::test_support`).
- 3 questions de périmètre instruites et TRANCHÉES (Q1/Q2/Q3 ci-dessous).

## Décisions de périmètre (tranchées avec évidence)

- **Q1 keep-online : IN (migre avec O).** `set_keep_online` :1622 + `KeepOnlineRequest` :1617
  (doc :1610) est le frère-duress de seed_voluntary (commentaire :1633 « Mirrors
  run_boot_seed_driver + seed_voluntary » ; THREAT_MODEL :1022 les groupe DURESS-FRERES-LOCAL) ;
  écrit la MÊME table keep_online (M18) + keep_online_tag que seed_voluntary
  (:2309/:2351) et run_boot_seed_driver (:1900/:2001) ; 0 consommateur cross-module du
  handler ; 0 couplage directory-publish. Auto-contenu.
- **Q2 boot-driver : MIGRE.** `run_boot_seed_driver` :1837 (déjà pub(crate)) +
  `seed_already_announced` :2035 (seul appelant prod = le driver :1995) +
  `build_seed_fetch_chain` :2404 (helper privé de seed_voluntary). Re-pointages EXACTS :
  **runtime.rs:1523 et runtime.rs:2473** (seuls call-sites prod qualifiés, grep-confirmé)
  `crate::http::` → `crate::seed_api::`. `reannounce_directory_at_boot` :1494 RESTE
  (domaine publish/directory, Phase S ; consumer runtime.rs:1516 INCHANGÉ). La propriété
  documentée « duress gate = FIRST statement » migre verbatim avec le corps.
- **Q3 nodes : OUT (→ Phase S2).** NodesResponse :2047 / ObservedNodeView :2063 /
  NodeSummary :2072 / nodes_response :2089 / list_nodes :2122 RESTENT ; route :310 inchangée
  bare-name. 0 couplage dans les deux sens.
- **Cluster « Directory-only pull resolution » (header :1691) : RESTE dans http.rs, ATOMIQUE**
  (contradiction C1 arbitrée : le cluster est structurellement partagé seed↔blob_serve —
  blob_serve consomme by_hash :3078 + pull_providers :3091 + TIMEOUT :3104). **Bump
  pub(crate) ×3** pour les symboles que seed_api référencera cross-module :
  `find_directory_app_by_project` :1736, `directory_pull_providers` :1772,
  `DIRECTORY_PULL_TIMEOUT_SECS` :1705. `find_directory_app_by_hash` :1711 (consommateur
  blob_serve-only) + `PULL_PROVIDER_CAP` :1699 (interne) restent PRIVÉS. NOTE PLAN (à ne pas
  résoudre en O) : ce cluster n'est assigné à AUCUNE phase — hétérogène (by_hash → S4
  blob_serve ; by_project/pull-providers → S2 browse) ; consigné pour arbitrage S2/S4.

## Dérives plan à consigner

1. **Bornes plan `http.rs:2489-3263` DOUBLE-STALES** (post N+N2). Région réelle par NOM =
   **2 tranches NON-contiguës** : [1610 (doc keep-online) → 2037 (close
   seed_already_announced)] — en EXCLUANT le cluster pull [~1691-1800] qui reste — +
   [2138 (doc SeedVoluntaryRequest) → 2845 (close seed_request_peer)] — en EXCLUANT
   l'intrus nodes [2039-2136]. Le début plan 2489 tombe DANS le corps de seed_count.
2. **Bornes plan Phase S `1159-1727` sur-étendues** : publish réel finit à
   next_directory_revision :1608 ; la queue 1616-1727 = keep-online (→ O) + cluster pull
   (reste). Corrigé pour éviter la collision O/S.
3. **« handler + DTO + tests co-déplacés » SOUS-SPÉCIFIE 4 adaptations porteuses** :
   (a) bump pub(crate) de 3 symboles STAY (pas un move 0-visibilité comme N) ;
   (b) promotion de **4 fixtures de test partagées** vers test_support.rs ;
   (c) 2 re-points runtime.rs ; (d) keep-online absorbé (Q1).
4. Route de `seed_invite_list` trouvée : **:304 `/api/daemon/seed/invites/{project_id}`**
   (7 routes seed au total : :291, :295, :299, :300, :301, :304, :307).

## Move-set FINAL

**Production → seed_api.rs (VERBATIM ; 7 handlers privés → pub(crate)) :**
set_keep_online (+KeepOnlineRequest, doc :1610), seed_voluntary (+SeedVoluntaryRequest :2154,
+enum SeedFetchPlan :2167), seed_count (+SeedCountQuery :2420), seed_invite_mint
(+SeedInviteMintRequest :2523), seed_invite_revoke (+SeedInviteRevokeRequest :2590),
seed_invite_list, seed_request_peer (+SeedRequestPeerRequest :2680,
+SEED_REQUEST_TIMEOUT_SECS :2653) ; run_boot_seed_driver + seed_already_announced
(pub(crate) conservé) ; build_seed_fetch_chain (privé).

**Tests → seed_api.rs::tests (~18 + 2 fixtures seed-only ; move PAR-FN, JAMAIS tranche
contiguë — health_returns_200 :6289, info_returns_full_snapshot :6311, vps_authoring :5760
s'intercalent et RESTENT) :** seed_voluntary_directory_only_app :5158,
seed_voluntary_version_discriminator_local_rejects :5256, seed_count_exposes_self_pin_intent
:5411, boot_seed_driver_pins_configured_projects :5477,
redrive_on_ingest_pins_configured_app_without_restart :5553, boot_repins_keep_online_blobs
:5698, boot_seed_driver_empty_config_is_noop :5734, request_seed_prod_caller :5825,
boot_seed_driver_noop_in_duress :5939, seed_already_announced_predicate :5973,
boot_driver_prefers_keep_online_hash_over_directory :5989, seed_request_peer_noop_in_duress
:6045, set_keep_online_noop_in_duress :6076, seed_voluntary_noop_in_duress :6119,
pull_falls_back_across_tiers_when_ticket_dead :6159, seed_request_peer_rejects_local_errors
:6203, voluntary_seed_distant_public_app_no_approval :11409, keep_online_off_removes_tag
:11504 ; + fixtures seed-only co-migrantes : has_tag :11397, ingest_remote_directory :4783.

**Fixtures partagées → test_support.rs pub(crate) (4 — contradiction C3 arbitrée, S5 avait
raison) :** own_browse_entry :4290, catalog_app :4821, make_zip :7211, deploy_workspace_app
:11089 (partagées entre tests O-migrants et tests fork/browse qui RESTENT ; résolution par
le glob `use crate::test_support::*` existant — 0 édit call-site).

> **DÉVIATION IN-PHASE (consignée post-code, compiler-forcée)** : `ingest_remote_directory`
> est la **5ᵉ fixture promue en test_support** (pas co-migrée dans seed_api::tests comme
> listé ci-dessus). Le critic la disait seed-only mais `reachable_via_seeder_status` (test
> composite qui RESTE dans http.rs, décision C2 de CE preflight) l'appelle aussi — prouvé
> par le compilateur (E0425 http.rs:3979). Cohérent avec la politique fixtures partagées ;
> 0 scope creep (test_support déjà dans le périmètre).

**Tests qui RESTENT :** directory_resolvers_match_hash_and_project :4832 (teste by_hash+
by_project, tous deux dans http.rs), fetch_provider_ordering :4924 (cluster),
nodes_response_pins_envelope_and_grouping :4975 (S2), reachable_via_seeder_status :5048
(composite 3-domaines — C2 arbitrée STAY : 2/3 assertions S2, router-driven, re-home
naturel en S2), publish_directory_* + vps_authoring_signs_own_directory :5760 (Phase S).

**Golden :** `golden_http_seed_domain` (test_support.rs:427) RESTE — routes byte-identiques,
doit rester vert. Famille 9/9 jamais fragmentée.

## S1b/S2/S3/S4 — synthèse

- **S1b** : 0 dep, 0 delta lock. Edge entrant conservé : seed_api → `crate::http::`
  {mint_blob_ticket, find_directory_app_by_project, directory_pull_providers,
  DIRECTORY_PULL_TIMEOUT_SECS, DaemonHttpState}. Edges déjà cross-module inchangés
  (deploy::keep_online_tag/decode_hash_hex, feed_sync::emit_seed_announced,
  noop_identity, seed_protocol::request_seed).
- **S2** : invariants S74 D/E/F + S75 verrous anti-recentralisation + S76 B1 duress —
  tous portés par du code qui migre VERBATIM (invite lié à (project_id,archive_hash),
  DOMAIN_SEED_REQUEST_V1 signé dans seed_protocol.rs non touché). Coords plan
  doublement stales confirmées.
- **S3** : sécurité-neutre par construction — 7 routes restent dans authed_routes (tier
  loopback T0), 4 gates duress inline migrent avec leurs corps, 0 frontière de confiance
  traversée. Filet = golden seed + 4 tests duress + count invariant.
- **S4 (leçon N appliquée)** : grep exhaustif docs → **0 ref file:symbol GATING** ; aucun
  gate CI ne résout un symbole seed. 2 refs prose non-gating à ré-honnêter in-phase
  (PO-10) : THREAT_MODEL.md:1019 (`http.rs run_boot_seed_driver` → seed_api.rs) +
  PATTERNS.md:4153 (ancre stale directory_pull_providers — le cluster RESTE mais remonte
  de ~79 l après le départ de keep-online). Consommateurs runtime des routes (web/CLI/
  scripts/e2e) tous couplés path+shape. Gap de couverture CONSIGNÉ (pré-existant, non
  bloquant) : seed_invite_* n'ont AUCUN test HTTP dédié — couverture = golden seed +
  tests DB seed_protocol.rs:467+.

## Vérification adversariale — synthèse

55 agents ; contradictions arbitrées : C1 cluster pull (STAY atomique + bump ×3 — la
cohésion du cluster prime sur la minimisation des edges), C2 reachable_via_seeder_status
(STAY, composite S2-dominant), C3 fixtures 4 pas 2 (S5 avait raison : make_zip +
deploy_workspace_app partagés avec les tests fork restants via keep_online_off_removes_tag),
C4 framing « 0 couplage comme N » corrigé (O force un bump de visibilité de symboles STAY +
promotion fixtures — couplage que N n'avait pas).

## Oracle T1 précis

- fmt --all --check 0 ; clippy --workspace --all-targets -D warnings 0.
- nextest workspace : **== 2108 Win EXACT** ; Docker sbfb-ci mount `/workspace` : **== 2112**.
- Goldens **9/9** (dont golden_http_seed_domain — routes byte-identiques) ; 4 tests duress
  seed + 14 autres sous leur nouveau path `seed_api::tests::`.
- 0 delta Cargo.lock ; 0 route path ; routes count inchangé ; gates docs verts.
- Diff-preuve : bloc production migré verbatim (seuls deltas = 7 qualifieurs de visibilité
  handlers + 3 bumps pub(crate) des symboles STAY).
- T2 = N-A.

## Verdict: PLAN-ADAPT

Approche du plan confirmée (split domaine seed, routes inchangées, golden-gardé) — 0
DESIGN-CONFLICT, sécurité-neutre unanime. Adaptations requises : bornes re-dérivées par NOM
(2 tranches non-contiguës, plan doublement stale, début plan DANS un corps de fn) ;
keep-online absorbé (Q1) ; driver migré avec 2 re-points runtime.rs (Q2) ; nodes exclu (Q3) ;
cluster pull STAY atomique + bump pub(crate) ×3 ; 4 fixtures promues test_support ; 2 refs
prose docs ré-honnêtées. Le code suit l'approche corrigée ci-dessus.
