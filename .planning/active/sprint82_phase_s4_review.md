# Sprint 82 — Phase S4 — Review (Workflow ultracode)

- **Phase** : S4 — sweep final du split `http.rs` (10ᵉ et dernière phase refacto
  de la série N→S4) : module NEUF `crates/nexus-shell-daemon/src/blob_serve_http.rs`
  (middleware CSP + cluster Directory-only pull resolution + panic_wipe +
  blob_serve) + arrivées canary_api.rs / diagnostic_api.rs + extension PO-10
  familles de tests router-driven vers 15 modules existants
- **Date** : 2026-07-17
- **Base** : HEAD `0a32ffa` + working tree Phase S4 (non committée)
- **Méthode** : Workflow — 8 dimensions parallèles opus-4-8[1m] + passe verify
  adversariale sur chaque finding (un finding ne compte que si is_real=true) ;
  diff lu en entier ; preuve de fidélité indépendante par multiset de lignes
  (D-only vs A-only, chaque résidu mappé à un delta DÉCLARÉ) + re-run du script
  TOKEN_IDENTICAL depuis le snapshot `http_before.rs` (vérifié byte-identique à
  `git show HEAD:…/http.rs`, sha256 `d511e73d…92e1e5`) ; invariants sécurité
  (a)-(f) relus ligne à ligne sur disque.

## Verdict: PASS

> Review Workflow : **8/8 dimensions PASS**, **0 P0 / 0 P1 / 0 P2 confirmés**.
> **5 findings P3 CONFIRMÉS** (tous documentaires/comptables, 0 défaut code) +
> 3 findings RÉFUTÉS par la passe verify (pré-existants ou déclarés, hors diff
> de phase). Les 5 P3 sont TRAITÉS avant commit (cf. § Codex reconciliation).
> Promotion PASS après round 1 Codex Sol PASS WITH NOTES 0 P0/P1.

## Dimensions (8/8 PASS)

| # | Dimension | Verdict | Findings confirmés |
|---|---|---|---|
| 1 | diff-fidelite (multiset D/A + TOKEN_IDENTICAL + invariants) | PASS | 1 P3 (partagé : compta « 3+4 ») |
| 2 | suites (sondages indépendants, pas de gros blocs relancés) | PASS | 2 P3 (compta « 3+4 » + proof/ vide) |
| 3 | tests-coverage (co-migration par NOM, balance ±0) | PASS | 0 |
| 4 | scope + STAY-set (17 items byte-identiques) | PASS | 1 P3 (compta brief 21 vs 29) |
| 5 | research grounding (préflight PLAN-ADAPT, 5 adaptations) | PASS | 1 P3 (knowledge pack S79) |
| 6 | sécurité deep — invariants verbatim (a)-(f) | PASS | 0 |
| 6bis | docs-contrat + test-acteur §6.12 | PASS | 2 P3 (test_support:823 + knowledge pack, partagé) |
| 8 | comptabilité + patterns | PASS | 1 P3 (partagé : compta « 3+4 ») |

Dédupliqués : **5 P3 distincts** (le « 3+4 » remonte par 3 dimensions, le
knowledge pack par 2).

### 1. diff-fidelite — preuve multiset PLUS FORTE que le TOKEN_IDENTICAL déclaré
Comparaison multiset de TOUTES les lignes supprimées (http.rs + 20 fichiers Rust
touchés) contre TOUTES les lignes ajoutées (module neuf 826 l lu en entier
inclus). Résidu 63 D-only / 214 A-only, **chaque ligne mappée à un delta
DÉCLARÉ** : exactement 8 préfixes `pub(crate)` (7 handlers —
blob_serve_csp_middleware, panic_wipe, blob_serve, canary_observed,
canary_network_health, canary_freshness, diagnostic_neighborhood — +
browse_entries), 2 signatures re-wrappées rustfmt (virgule trailing), 5 formes
re-wrappées de routes à paths BYTE-IDENTIQUES, 16× template harness N2, de-indent
4 browse_entries (test_support.rs:915), bannières S46-A/S46-B + keep-online :4321
supprimées (déclarées), bannière blob réécrite, headers canary/diagnostic
réécrits, doc :1058→« gated in publish_api.rs », imports nettoyés
(Path/Next/warn/blob_serve::self retirés, BlobsClient→mod tests, Serialize +
`#[cfg(test)]` Deserialize split), split use seed_api.rs:22-26,
`mod blob_serve_http;` main.rs:32 en ordre alpha réel. **AUCUN hunk non
déclaré.** Réconciliation tests 1:1 : 81 tests déplacés + 2 helpers, chaque nom
retrouvé exactement une fois ; amendement contributor_dashboard×2→kudos_api.rs
justifié (handler kudos_api.rs:159). Routes 89==89. 16 re-points docs vérifiés
un à un. Sondage : `nextest -E 'test(blob_serve_http::)'` = 8/8 PASS.

### 2. suites — sondages indépendants, baselines réconciliées
wc -l http.rs = **1513** EXACT (avant HEAD = 4322 ; critère machine PO-10
~≤2500 **PASS** avec marge) ; blob_serve_http.rs = 826 l. `.route(` = 89 avant
ET après. fmt --all --check EXIT 0 rejoué. Crate **466/466** PASS obtenu
(à --test-threads=4 ; cf. remarque non-bloquante ci-dessous). Neutralité de move
PROUVÉE arithmétiquement : 84 attributs retirés de http.rs = 73 ajoutés dans 18
modules trackés + 8 dans blob_serve_http.rs (neuf) + 3 relocalisés
intra-http.rs — balance EXACTE 0, cohérent avec le delta ±0 EXACT (Win
2108/2108, Docker 2112/2112, écart 4 = les 4 `#[cfg(unix)]` documentés). 3 gates
docs re-exécutées indépendamment = exit 0 chacune. TOKEN_IDENTICAL : re-run
indépendant du script = identité verbatim, seuls écarts = les 2 re-wraps rustfmt
DÉCLARÉS ; snapshot http_before.rs byte-identique à HEAD ; spot-checks
consent.rs OK-VERBATIM. Déviations confirmées sur disque : mint_blob_ticket
(http.rs:983) + archive_hash_from_ticket (:976) **STAY** ; seed_api.rs:22-25
split use conforme.

**Remarque non-bloquante (finding RÉFUTÉ, pré-existant hors diff)** : la famille
`dispatch_loop` boot sync-set (S81) est load-flaky à parallélisme nextest par
défaut sur cette machine (2 runs, 2 membres différents ; 3/3 en solo ; 466/466 à
--test-threads=4 ; dispatch_loop.rs ABSENT du diff S4). Classe env-variance déjà
documentée projet. Recommandation : re-run avant commit + candidat test-group
nextest à parallélisme réduit (dette routable audit plan, S81-A4).

### 3. tests-coverage — 81 co-migrés par NOM, 0 LOST, 0 SPURIOUS
Vérification par NOM (pas seulement par compte) : ensemble des 81 noms sortis de
http.rs == ensemble des 81 noms entrés, 0 doublon inter-destinations.
Destinations : +73 sur 17 fichiers trackés (deploy +11, consent +10, files +8,
canary +6, diagnostic +5, contributor +5, kudos +5, tasks +4, apps +4, invite
+4, quarantine +3, storage +2, publish +2, feed +1, shell +1, health +1,
worker_state +1) + 8 dans blob_serve_http.rs (2 cluster :374/:466 + 6 blob
:523-:769, conforme au claim cluster-2+blob-6). Doublons crate-wide : 2
homonymes seulement, TOUS deux pré-existants à HEAD avec sémantiques distinctes.
Résiduels http.rs = EXACTEMENT les 29 attendus (browse_index, health/info,
loopback/valid_origin, CORS, SPA, auth_token, archive_hash…). browse_entries :
UNE définition crate-wide (test_support.rs:915 pub(crate)). Aucun `#[ignore]`
ajouté. `nextest list` rejoué = 466 EXACT. Spot-check verbatim de 3 corps migrés
vs HEAD : token-identiques (une divergence apparente réfutée = mojibake de
l'outillage du reviewer, pas du repo).

### 4. scope + STAY-set — périmètre conforme, 17 STAY byte-identiques
STAY-set préflight §5.4 COMPLET — 17 items IDENTIQUES 0 hunk (DaemonHttpState,
auth_token_public, cors_layer, is_valid_origin, is_loopback_origin,
BrowseListResponse, ErrorResponse, runtime_error_to_response, health/info/
project_info, wrap_payload_with_pow, truncate_on_char_boundary,
trustworthy_open_source, archive_hash_from_ticket, mint_blob_ticket — déviation
STAY effective, publish_api.rs:20-21 reste vrai inchangé) ; build_router diffère
UNIQUEMENT par les 7 re-points déclarés (paths byte-identiques). Audit exhaustif
des 101 lignes ajoutées du diff http.rs : 100 % mappées. Périmètre : 29 fichiers
trackés de phase tous mappés 1:1 + 4 untracked (module neuf + préflight S4 + 2
workflow_* hors-phase) ; les 15 modules famille = additions pures DANS
`mod tests`, 0 ajout top-level (scan colonne-0). Aucun fichier hors liste.

### 5. research grounding — préflight PLAN-ADAPT honoré, ledger de bumps exact
Préflight lu EN ENTIER ; les 5 adaptations §14 vérifiées sur disque : (1) STAY
mint_blob_ticket/archive_hash_from_ticket — RIEN du défaut plan abandonné n'a
fuité ; (2) extension PO-10 15 modules + comptage statique 237==237 avant/après
sur les fichiers touchés + wc 1513 dans la fourchette §8 (1480-1610) ; (3)
middleware CSP MOVE blob_serve_http.rs:34 + layer http.rs:255-259 au nest
/blob-serve de public_routes SANS bearer + 2 tests témoins co-migrés ; (4)
cluster ATOMIQUE :62-172 avec docs menace verbatim (Sybil dial-chain, budget,
SYBIL-SEEDER-TAIL) + split use seed_api exactement §7.2 ; (5) bannières
absentes (grep 0 hit). Amendements §15 4/4 appliqués. Ledger : 7 movers TOUS
privés à HEAD → **8 bumps ROUTINE / 0 SHARED / 0 classe R** (7 handlers +
browse_entries), corroboré par grep `+pub` sur le diff complet — « compile n'a
forcé AUCUN bump supplémentaire » CONFIRMÉ. Hazards §7.1 tous appliqués.

### 6. sécurité deep — invariants (a)-(f) VERBATIM sur disque
(a) http.rs:255-264 : nest `/blob-serve` DANS public_routes sans bearer, layer
CSP `from_fn(blob_serve_http::blob_serve_csp_middleware)` câblé ;
`auth_required` uniquement sur authed_routes (:621). (b) blob_serve_http.rs:
182-208 : `execute()` SYNCHRONE → 200 → `tokio::spawn`+sleep(100 ms)+
`exit_only(0)`, commentaire anti double-wipe co-migré, PAS de gate duress
(intentionnel S20-B, non modifié) ; la route panic/wipe reste authed
(http.rs:391-394). (c) `validate_zip_path` :226 AVANT tout accès ; ordre des 4
tiers preview(:246)→local(:248)→ticket(:250)→directory-only(:271) ; relecture
PAR LE HASH DEMANDÉ post-fetch aux 2 tiers réseau (:264, :321) ; 200 sans CSP
inline (:351-359). (d) canary ×3 : mutex poisoned → 500 générique
`{"error":"internal"}`, détail en log seul. (e) diagnostic_neighborhood : peers
= `subscribed_pubkeys_hex()` SEUL (:139). (f) PULL_PROVIDER_CAP=8 privé (:71),
DIRECTORY_PULL_TIMEOUT_SECS=120 (:77), anchor-first + dedup + self-exclusion
(:144-172), docs menace verbatim. Preuve verbatim INDÉPENDANTE contre
`git show HEAD` (non-circulaire) : 13 items + 9 tests token-identiques, 0 delta
non déclaré. Témoins CSP co-migrés dans le MÊME diff, uniques dans le
workspace ; goldens : `golden_http_public_tier` INTACT, pin CSP byte-exact,
`build_test_router` passe par le VRAI `build_router`. RE-RUN CIBLÉ :
blob_serve/canary/diagnostic/golden_http_public_tier/cluster/remote_p2p =
**27/27 PASS**.

### 6bis. docs-contrat — 16 re-points appliqués et exacts, re-grep bidirectionnel
Les 16 re-points du contexte TOUS appliqués (PATTERNS :4159/:4196/:4211,
THREAT_MODEL :19/:776, FACTORY_GATES:190, app-authoring.spec.ts:28,
browse.rs:762-764 cross-crate, blob_serve.rs:282 forme symbole, csp.rs:9,
charte browse_api.rs:19-23, headers canary/diagnostic, test_support :702/:846,
bannière http.rs + doc publish gate :921). Chaque claim « stay in http.rs » d'un
doc-contrat recontrôlé sur disque (9 symboles + cors_layer + /auth/token +
ServeDir). Re-grep bidirectionnel indépendant sur crates/docs/web/scripts/
examples : exceptions légitimes confirmées ; 3 drifts remontés, tous P3 (2
confirmés de phase + 1 RÉFUTÉ pré-existant Phase N, cf. Findings). 3 gates docs
exit 0. 0 frontière neuve : Cargo.toml/lock intacts, 5 routes retirées / 5
ré-ajoutées à paths BYTE-IDENTIQUES.

### 8. comptabilité — preuve rejouée par script neuf, 43/43
Intégrité snapshot : sha256 http_before.rs == `git show HEAD:…/http.rs`.
Chiffres : http.rs 4322→1513 (numstat +101/−2910, net −2809 EXACT) ; module
neuf 826 l ; critère PO-10 PASS. Preuve TOKEN_IDENTICAL REJOUÉE (script
indépendant `s4_review_dim8_proof.py`) : **43/43 tranches OK**, chaque tranche
ws-strippée = substring du snapshot AVANT avec multiplicité 1 (locus unique) ET
d'exactement UN fichier destination conforme à l'annonce ; normalisations
nécessaires = EXACTEMENT le set déclaré (8 pub(crate) + 2 virgules trailing),
ZÉRO normalisation non déclarée ; preuve bidirectionnelle (tokens after-only →
non-circularité). Couverture exhaustive des 310 lignes supprimées hors tranches :
173 blanches + 118 bruit d'alignement git de 3 régions STAY (vérifiées
ws-verbatim présentes dans http.rs actuel) + 26 réécritures TOUTES déclarées.
Routes 89==89. Annotations test daemon src : HEAD 458 == working tree 458 →
delta ±0 confirmé au niveau crate. Structure : main.rs ordre alpha réel,
seed_api split conforme, test_support +27/−2 conforme.

## Findings confirmés (après verify adversarial)

**0 P0 / 0 P1 / 0 P2 — aucun bloquant.** 5 P3 distincts, tous documentés :

### P3-1 (CONFIRMÉ — dims 1, 2, 8) — Comptabilité « 3+4 tests » diagnostic_api : réel = 1+4 (=5)
diagnostic_api.rs HEAD = 2 tests pré-existants, working tree = 7 : delta +5 =
**1 neighborhood + 4 fairness**, pas 3+4=7. Le pré-phase http.rs contenait
exactement 1 test neighborhood + 4 fairness (vérifié sur le snapshot 4322 l).
Le préflight SUR DISQUE (:114) est CORRECT — l'erreur est confinée au résumé de
phase transmis. Aucun artefact repo faux, delta crate ±0 confirmé. **Action** :
le commit body doit dire 1+4 (=5) pour ne pas propager le chiffre.

### P3-2 (CONFIRMÉ — dim 2) — Sous-dossier proof/ du scratchpad s4 VIDE : sorties TOKEN_IDENTICAL non persistées
`scratchpad/s4/proof/` existe mais contient 0 fichier — trace de preuve
incomplète, PAS une falsification : les ENTRÉES sont persistées (slices/ 43 +
fresh/ 43), le snapshot est byte-identique à HEAD, et le re-run indépendant du
script reproduit l'identité verbatim (2 seuls [DIFF] = les 2 re-wraps rustfmt
DÉCLARÉS). **Action** : rejouer le script en redirigeant la sortie vers proof/
avant le commit (trivial).

### P3-3 (CONFIRMÉ — dim 4) — Imprécisions comptables du brief de phase (aucune violation de scope)
(1) « 21 modifiés attendus » vs réel 29 fichiers de phase (chacun mappé 1:1 à un
item déclaré — sous-compte, périmètre conforme) ; (2) « 1 untracked » omet le
préflight S4 (artefact process légitime) ; (3) le re-point doc déclaré « :1058 »
était à HEAD http.rs:**1056** (substance conforme, décalage de 2). Purement
comptable/documentaire, 0 défaut code, 0 impact invariants.

### P3-4 (CONFIRMÉ — dims 5, 6bis) — Knowledge pack exemple S79 : middleware CSP encore situé dans http.rs
`examples/daisyui-animejs-showcase/knowledge/factory-integration-hardened.md`
:65/:74/:77/:83-85 attribuent le middleware CSP à http.rs:551-577/:572-575/:558 ;
après S4 il vit dans blob_serve_http.rs:34 — le nom de FICHIER devient faux à
cette phase (seuls les numéros de ligne étaient déjà stale pré-phase). Hors
liste fermée §10 du préflight, hors preuves négatives, hors amendements §15 ;
hors périmètre des 3 gates docs (aucun gate ne rougit) ; annexe de design S79
datée, 0 ref depuis crates/. **Action** : re-point une ligne en forme symbole
(comme blob_serve.rs:282) ou classement snapshot-historique explicite.

### P3-5 (CONFIRMÉ — dim 6bis) — test_support.rs:823 : claim « staying http.rs tasks_api tests » devenu faux en S4
`make_test_submission` est consommé par coordinator_api.rs + tasks_api.rs
(:298/:350/:352) — 0 occurrence restante dans http.rs (la famille tasks a été
co-migrée en S4). Les 2 sections voisines (:702 Phase O, :846 Phase S3) ont bien
été re-wordées mais pas celle-ci (Phase Q) — 3ᵉ re-word manqué sous la propre
convention de la phase. Doc-comment test-only, 0 impact machine. **Action** :
correction 1 ligne (« moved to tasks_api.rs in S82 Phase S4 »).

## Findings RÉFUTÉS par la passe verify (is_real=false, consignés)

1. **3 fichiers hors-phase sales** (blueprint M + 2 untracked workflow_*) :
   pré-existants au snapshot de début de session ET déclarés « hors-phase PO
   intacts » — pas un défaut du diff. Rappel opérationnel conservé : **staging
   sélectif au commit, JAMAIS `git add -A`**.
2. **Flake dispatch_loop boot sync-set** à parallélisme nextest défaut :
   famille S81, dispatch_loop.rs ABSENT du diff, classe env-variance documentée
   — constat environnemental, pas une régression de phase (cf. remarque dim 2).
3. **observe.curl.md:14** pointe http.rs pour `shard_session_response` :
   pré-existant Phase N (commit `2e87eef`, avant HEAD), fichier non touché par
   S4 — dette doc résiduelle de série, à consigner (re-point one-line vers
   shard_session_http_api.rs, Phase T ou sweep ultérieur).

## Suites (§7.4) — chiffres re-vérifiés par sondage

- **Structure (rejouée lecture seule)** : http.rs **4322→1513** (wc -l EXACT,
  critère machine PO-10 ~≤2500 **PASS**) ; blob_serve_http.rs **826 l** ;
  `.route(` = **89 == 89** ; **TOKEN_IDENTICAL 43/43** (re-run indépendant,
  normalisation = EXACTEMENT le set déclaré) ; balance tests par NOM = 0 EXACT
  (81 sortis == 81 entrés) ; STAY-set 17/17 byte-identique ; snapshot
  http_before.rs byte-identique à HEAD (sha256).
- **Gates rapportés côté phase** (corroborés par sondage, non relancés en
  entier) : fmt --check Win+Docker EXIT 0 (fmt rejoué ici : clean) ; clippy
  workspace -D warnings ; nextest Win **2108/2108** / Docker sbfb-ci
  **2112/2112** 0 flake (delta **±0 EXACT** ; écart 4 = `#[cfg(unix)]`) ; crate
  **466/466** (rejoué : PASS à --test-threads=4 ; `nextest list` = 466 EXACT) ;
  doctests ; release build ; web lint 0 err + tsc + Vitest **412** + coverage
  87.27/79.01/86.02/88.59 + build + 6 size + scan-en clean ; operator Vitest
  **201/201** ; **goldens 9/9** (dans nextest, `golden_http_public_tier` pin CSP
  intact) ; 3 gates docs exit 0 (rejouées ici : exit 0 ×3) ; re-run ciblé
  sécurité 27/27 PASS + blob_serve_http::tests 8/8.

## À porter au commit body

**Corriger la comptabilité diagnostic_api : 1 neighborhood + 4 fairness (=5)**,
pas « 3+4 » (P3-1). 8 bumps ROUTINE / 0 SHARED (7 handlers + browse_entries
pub(crate)) ; 81 tests co-migrés (73 vers 17 modules trackés + 8 module neuf) +
2 helpers (post_workspace→deploy.rs, browse_entries→test_support.rs) ; déviation
STAY mint_blob_ticket + archive_hash_from_ticket (doc-contrat publish_api.rs
reste vrai) ; 16 re-points docs ; 7 routes re-pointées paths byte-identiques ;
bannières S46-A/S46-B + keep-online orpheline retirées. **Avant commit** :
staging sélectif EXCLUANT les 3 hors-phase PO (blueprint M + 2 workflow_*) ;
rejouer le script de preuve vers proof/ (P3-2) ; fixes 1-ligne recommandés
(P3-5 test_support:823 ; P3-4 knowledge pack en forme symbole ou exemption
explicite) ; consigner observe.curl.md:14 (dette pré-existante Phase N).
Comptabilité : http.rs 4322→1513 / blob_serve_http.rs 826 / 89 routes / 0 wire /
0 dep / delta tests ±0 EXACT.

## Codex reconciliation

Round 1 Codex GPT-5.6 Sol (`codex exec -m gpt-5.6-sol -c
model_reasoning_effort=max --sandbox read-only`, rapport BRUT
`sprint82_phase_s4_codex_review.md`) : **GLOBAL PASS WITH NOTES — 0 P0 / 0 P1**,
L1/L2/L3/L6/L9 CONFIRMÉS + L4/L5/L7/L8/L10 PARTIELS (tous des écarts de
comptabilité/traçabilité process, 0 défaut code, 0 boucle requise). Corroboration
indépendante clé : 81 tests retirés == 81 ajoutés mêmes noms corps
token-identiques (458 attributs crate avant==après), http.rs 4322→1513, routes
89==89 mêmes paths même ordre, STAY token-identique, invariants sécurité (a)-(f)
vérifiés sur disque, fmt + 3 gates docs rejoués exit 0.

Traitement des notes AVANT commit :
1. Ledger corrigé (body) : deploy **+11** tests (le prompt Codex portait un 9
   erroné, le body était déjà au 11 mécanique) ; « **6 routes + 1 layer
   middleware** = 7 re-points » ; **3 re-words** test_support (:702/:821/:846).
2. Preuve TOKEN_IDENTICAL rendue REPO-VISIBLE :
   `.planning/active/sprint82_phase_s4_token_proof.txt` (43/43 + sha256
   snapshot==HEAD ; le chemin scratchpad session n'était pas lisible d'un
   auditeur externe — la P3 « preuve absente » était un problème de chemin,
   pas d'existence).
3. Knowledge pack `factory-integration-hardened.md` : bannière **SNAPSHOT
   HISTORIQUE** posée en tête (l'exemption devient visible depuis l'artefact).
4. Dette pré-existante Phase N `docs/sharding/examples/observe.curl.md:14`
   re-pointée `shard_session_http_api.rs` (fix opportuniste classe :1058).
5. Review : les P3-1/P3-2/P3-4/P3-5 de cette review sont clos par les points
   ci-dessus et par les fixes pré-Codex (re-word :823, exemption, compta 1+4=5) ;
   P3-3 (imprécisions du brief) consigné au body.
6. Staging sélectif : blueprint PO modifié + 2 research untracked `workflow_*`
   EXCLUS (hors-phase PO standing).

P2 pré-existants consignés par Codex (hors diff S4, candidats S83/Phase T) :
0 golden feed/search/provenance/preview/proof-card/browse/nodes (classe S2/S3) ;
famille `dispatch_loop` candidate test-group nextest borné (flake au parallélisme
défaut, passe seule et à --test-threads=4 ; dispatch_loop.rs hors diff).
