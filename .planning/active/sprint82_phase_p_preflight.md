# Sprint 82 Phase P — Preflight G8 (split domaine frost → `frost_api.rs`, discipline étendue PO-10)

## Contexte + méthode (Workflow multi-agents, 2026-07-16)

- HEAD au preflight : `542254b` (Phase O DONE). Phases A→O DONE (16/24).
- Orchestration : Workflow ultracode `wf_5b916de9-12d` — **53 agents** (5 scans G8, réfuteur
  adversarial par claim, critic final ; run repris de cache après interruption de process,
  0 agent en erreur).
- Mission : split du domaine frost de http.rs (9518 l) vers `frost_api.rs`, discipline
  étendue PO-10 (les 8 tests du domaine co-migrent via `crate::test_support`).

## Faits corrigés (vérifiés disque)

1. **La route `/api/canary/frost/trusted-dealer` EXISTE** (http.rs:389-392, `.route(` éclaté
   multi-ligne par rustfmt — faux négatif du grep mono-ligne du scout). Les 4 handlers sont
   route-wired ; AUCUN n'est CLI-only ; ne pas chercher de 5ᵉ handler (claim « miroir 1:1
   CLI » RÉFUTÉ : miroir 4-de-5, `build_signing_package` est CLI-only via primitives core).
2. **Bornes plan `3559-3722` doublement stales** (les 2 bouts tombent dans le mod tests au
   HEAD). Réelles par NOM : **PROD = :2208-2370** (bannière 3 l incluse), **TESTS =
   :5782-6192** (bannière incluse). « 4 handlers » du plan : CORRECT.
3. **0 couplage cross-module — le split le plus PROPRE de S82** : cli.rs = enum clap
   `FrostCommand` (:364) ; main.rs `handle_frost` (:809) appelle DIRECTEMENT
   `nexus_shell_daemon_core::canary::*` (generate_dkg :824, ceremony_round1 :861,
   ceremony_round2 :958, ceremony_aggregate :998) — JAMAIS la couche http. Les hits
   test_support (:472/:484) = noms de fixtures golden (data, pas des appels). 0 re-pointage
   hors build_router. frost prod ne consomme RIEN de crate::http (0 helper carry, contraste
   seed_api).
4. **frost-ed25519 = "3"** (Cargo.toml:402, upgradé S34-A) — le « 2.1 » d'un scan venait de
   SPRINT_LOG:313 (état S20 historique, piège doc-stale classe leçon-N ; immatériel au move).

## Move-set FINAL

**Production → frost_api.rs (verbatim :2208-2370, bannière :2208-2210 SUBSUMÉE par le `//!`
du module — pas de doublon) :**
- pub(crate) : FrostTrustedDealerRequest (:2213), frost_trusted_dealer (:2224),
  FrostRound1Request (:2243), frost_round1 (:2254), FrostRound2Request (:2292),
  frost_round2 (:2299), FrostAggregateRequest (:2331), frost_aggregate (:2337) —
  4 handlers + 4 Request DTOs (types dans les signatures `Json<T>`, lint
  private_interfaces sous -D warnings).
- **PRIVÉS** : FrostTrustedDealerResponse (:2219), FrostRound1Response (:2249) —
  body-only derrière `impl IntoResponse`, jamais dans une signature (précision vs un
  naïf « 6 DTO pub(crate) », auto-réfuté par S4).
- **CONTRAINTE LOAD-BEARING** : préserver l'ORDRE DES CHAMPS (`k` premier dans
  TrustedDealerRequest, `participant` premier dans Round1Request) — le golden asserte le
  texte serde EXACT « missing field \`k\` » / « missing field \`participant\` »
  (test_support.rs:492/:480).
- Imports prod = EXACTEMENT 3 lignes : `axum::http::StatusCode` ;
  `axum::response::{IntoResponse, Json}` ; `serde::{Deserialize, Serialize}`.
  Tout le reste reste full-path dans les corps (nexus_shell_daemon_core::canary::*).

**Tests → frost_api.rs::tests (verbatim :5782-6192, 8 tests)** :
frost_http_trusted_dealer_returns_shares_and_pubkey (:5787), round1_returns_commitment
(:5829), round2_returns_signature_share (:5886), aggregate_returns_valid_signature (:5970),
invalid_threshold_k_gt_n (:6087), malformed_json_body (:6108), round1_invalid_key_package
(:6129), aggregate_invalid_pubkey (:6152). Bloc d'imports du mod tests (6 lignes —
DÉVIATION vs gabarit seed_api) : `use super::*;` + **`use std::sync::Arc;` (L'AJOUT —
frost prod est stateless, Arc ne transite pas via super::*)** + `axum::body::to_bytes` +
`axum::http::{Method, Request}` + `tower::ServiceExt` + `use crate::test_support::*;` ;
**DROP `nexus_core_rs::{KeyPair, create_node}`** (0 usage dans les 8 tests, unused_imports
sinon).

**Restent** : golden_http_frost_domain (test_support.rs:467-497, URI-driven, famille 9/9
atomique) ; les 4 routes dans build_router re-pointées full-path (URL byte-identiques,
commentaire :386-388 intact) ; cli.rs/main.rs handle_frost/test_support INTOUCHÉS.
main.rs : `mod frost_api;` entre `mod files;` (:40) et `mod health_api;` (:41) — mod
NORMAL (pas cfg(test), handlers de production).

## S2/S3/S4 — synthèse

- **S2** : couche http frost = glue Json→primitives core (S31/S32 warrant canary FROST DKG) ;
  séparation frost/canary CONFIRMÉE (canary_observed/network_health/freshness → S4 ; les
  routes /api/canary/frost/* re-pointent frost_api en gardant leurs paths).
- **S3** : sécurité-neutre — routes dans authed_routes (tier loopback T0-admin), 0 gate
  duress (stateless crypto, rien à gater), 0 secret persisté par la couche http ; les 4
  tests négatifs migrent avec sémantique location-indépendante.
- **S4 docs** : 0 ref file:symbol GATING sur les symboles frost ; hits docs = prose
  historique immuable (SPRINT_LOG) ou primitives core (non touchées). ASYMÉTRIE DE FILET
  consignée (dette pré-existante, PAS scope P) : golden couvre empty-body 422 pour
  round1 + trusted-dealer seulement ; round2 n'a AUCUN test error-path ; aggregate teste
  un full-body 400. Phase P = move ±0 test — consigné honnêtement, pas clos.

## Oracle T1 précis

- fmt --check 0 ; clippy --workspace --all-targets -D warnings 0 (= l'oracle
  private_interfaces qui PROUVE la répartition 4-pub(crate)/2-privés).
- nextest workspace **== 2108 Win EXACT** ; Docker /workspace **== 2112**, 0 flake attendu.
- 8 tests sous `frost_api::tests::` ; goldens 9/9 dont golden_http_frost_domain (les 2
  messages 422 byte-identiques) ; 0 résidu frost sous http::tests::.
- http.rs attendu ~8942 l (−576). 0 delta Cargo ; 0 route path ; frost n'a AUCUN T1
  web/Playwright (T0 loopback+CLI) : golden + 8 tests + count EXACT = l'oracle complet.
  **[Réconciliation post-code (review F1)** : −576 = retrait BRUT des 2 blocs ; les 4
  re-points de routes en multi-ligne rustfmt ajoutent +9 → net réel **−567, http.rs
  final 8951 l** ; frost_api.rs = **598 l** post-fmt (pas 596).]
- T2 = N-A.

## Verdict: EXECUTE

Aucun fait du plan n'exige d'adaptation d'approche : « 4 handlers » exact, domaine contigu,
0 couplage, 0 helper, 0 doc gating — seules les coords étaient stales (re-dérivées par NOM,
règle standing du sprint) et le scout avait un faux négatif de grep (route multi-ligne),
corrigés ci-dessus sans changer l'approche. Le code suit le move-set ci-dessus. (1ᵉʳ
EXECUTE des splits S82 — N/N2/O étaient PLAN-ADAPT.)
