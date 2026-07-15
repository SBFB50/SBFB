# Sprint 82 Phase K — Review (Workflow)

Date : 2026-07-15. Review ultracode = Workflow 7 dimensions
(D1 diff-intégral / D2 tests-sémantique / D3 sécurité-deep /
D4 scope-cuts-invariants / D5 research-grounding / D6 livrables-
patterns-frontière / D7 suites-audit), chacune = agent review +
agent de vérification adversariale de CHAQUE finding (opus-4-8[1m] ;
D4 et D6 = 0 finding, pas de passe verify). Phase K = bump
hickory-resolver 0.24 -> 0.26 (PO-7=A / D11, carry
HICKORY-024-RUSTSEC) + remédiation yanked spin 0.9.8/0.10.0 ->
0.9.9/0.10.1 (classe S81-G, même phase).

Diff review (working tree, pré-commit, PAS encore committé) = exactement
6 fichiers de phase : `Cargo.toml`, `Cargo.lock`,
`crates/nexus-core-rs/src/dns_fallback.rs`, `deny.toml`,
`docs/security/HARDENING_ROADMAP.md`, `docs/security/THREAT_MODEL.md`
+ artefact `.planning/active/sprint82_phase_k_preflight.md` (untracked,
verdict PLAN-ADAPT:439, 7 corrections câblées). Index PROPRE (rien stagé
-> review bien pré-commit). HORS PHASE, non reviewés, non-défauts :
`verification_blueprint.md` (tracké modifié, édition PO mi-session) +
`workflow_agents_app_conception_ultradeep_2026-07-15.md` +
`workflow_hub_product_conception_2026-07-15.md` (2 untracked recherche PO).

## Verdict: PASS

Aucun P0/P1. 13 findings bruts sur 7 dimensions -> après vérification
adversariale et déduplication inter-dimensions : **1 P2 + 8 P3
distincts, tous CONFIRMED** (1 P2 rétrogradé P3 : webpki-roots ; 0 REFUTED).
Aucun défaut de code bloquant : le seul P2 est une consigne d'hygiène
pré-commit (stager par chemins explicites), pas un défaut du diff ; les
8 P3 sont cosmétiques / de couverture / informationnels (3 pré-existants,
2 phase-introduits bornés, 3 doc/record). Le code suit **à la lettre**
l'approche corrigée du preflight PLAN-ADAPT. Verdict initial PASS-PENDING
promu **PASS** après la gate Codex (GPT-5.6 Sol reasoning max, round 1 :
4 CONFIRMÉ + 2 PARTIEL triés, 0 GAP — cf. §Codex reconciliation en fin
de document).

## Dimension D1 — Diff intégral ligne par ligne (CONFORME, 1 P2 + 2 P3)

`dns_fallback.rs` re-lu en entier (586 l) ; delta `Cargo.lock` re-prouvé
indépendamment par comparaison d'ensembles (name,version) HEAD↔working
(awk + `comm`). **Port fidèle et behavior-preserving** : imports conformes
au squelette (`+Arc/+Resolver/+ConnectionConfig/+TokioRuntimeProvider/+RData` ;
`-TokioAsyncResolver/-NameServerConfigGroup/-Protocol`) ; alias
`type TokioResolver = Resolver<TokioRuntimeProvider>` + enum local
`DnsTransport{Doh,Dot}` (état type-vacant remplaçant l'enum hickory
`Protocol` supprimé upstream) ; `build_resolver` : garde
`endpoints.is_empty()` préservée, `NameServerConfig::new(ep.ip, false, vec![conn])`
+ `ConnectionConfig::https/tls` **par endpoint** (P2-E-1 `server_name: Arc<str>`
per-endpoint préservé, jamais global), `trust_negative_responses=false`
rendu EXPLICITE avec rationale de course DoH/DoT (neutralise la bascule
défaut->true de 0.26), `build().map_err(NexusError::Endpoint)` (faillibilité
0.26 mappée) ; `resolve_txt_via` : `lookup.answers()` + filtre `RData::TXT`
+ `txt.txt_data.iter()` (équivalence sémantique vérifiée — concatène tous
segments TXT en ordre wire, écarte CNAME). Course `tokio::select!`, 6
`debug!`, constantes IP/TLS/ports/timeout/suffix, trait public
`DnsFallbackResolve` : tous INCHANGÉS. `Cargo.toml` : pin `0.24->0.26`,
features `["tls-ring","https-ring","webpki-roots"]`, commentaire factuel
au passé immuable (0 PROMISE_RE). `deny.toml` : 4 ignores hickory retirés,
`ignore=[]` ne garde que les 2 quick-xml, classes DoS/authentification
séparées. Docs sécurité additives (THREAT_MODEL v18 = 26 insertions / 0
suppression). Findings :

- **D1-1 (P2, CONFIRMED)** — Scoping du commit atomique : le working tree
  contient, EN PLUS des 6 fichiers de phase + préflight, `verification_blueprint.md`
  (tracké modifié) + 2 untracked de recherche PO. `git status --porcelain`
  confirme l'index vide. Un `git commit -a` embarquerait le blueprint, un
  `git add -A` les 2 recherches -> pollution de l'invariant « un commit =
  une phase ». **ACTION pré-commit** : stager EXPLICITEMENT les 6 fichiers
  + `.planning/active/sprint82_phase_k_preflight.md`, jamais `git add -A` /
  `git commit -a`. Advisory (pas un défaut de contenu du diff).
- **D1-2 (P3, CONFIRMED)** — `Cargo.lock` churn de ré-unification
  (windows-sys/heck/indexmap/syn ; anstream/anstyle-query/derive-deftly).
  `comm -23/-13` sur les ensembles triés prouve : le SEUL delta de paquets
  = retrait net du sous-arbre hickory-0.24 (12 paquets) + swap spin yanked ;
  AUCUN windows-sys/heck/indexmap/syn entrant ou sortant -> ré-unification
  pure d'arêtes (0 crate/version/licence/source neuf). D11 tenu. Bénin.
- **D1-3 (P3, CONFIRMED)** — `HARDENING_ROADMAP.md:30` « + 6 autres »
  ambigu/off-by-one (voir D3-2, même finding).

## Dimension D2 — Tests + branch coverage sémantique (CONFORME, 3 P3)

Modifications de tests correctes, honnêtes, intention préservée ; delta
`-1` net honnête et cohérent avec les runs. Le test retiré
`build_resolver_rejects_unsupported_protocol` était **vraiment type-vacant**
(l'enum local `DnsTransport{Doh,Dot}` à 2 variantes rend « protocole non
supporté » irreprésentable ; la garde runtime `!matches!` a disparu ; le
commentaire de retrait `:573-576` est exact). `build_resolver_rejects_empty_endpoints`
adapté `Protocol::Https->DnsTransport::Doh` (`:581`) — la garde vide
s'exécute AVANT usage du transport, intention 100 % préservée. Les 13 tests
restants exercent les 2 bras `match transport` (chaque `new()` construit 2
resolvers Doh+Dot) + le bras Err endpoints-vides. Extraction TXT 0.24->0.26
**ÉQUIVALENTE** (source primaire docs.rs : 0.24 `TxtLookup::iter()` filtrait
déjà aux records TXT ; 0.26 `answers()` = section answer + `RData::TXT` filtre
pareil -> même ensemble ; sécurité-neutre car record = signal « existe »,
octets non interprétés, pkarr Ed25519 en aval). Symboles `DnsTransport` /
`build_resolver` privés (grep : 0 réf hors `dns_fallback.rs`) ; trait public
INCHANGÉ -> mock `DnsFallbackMock` (browse.rs:1670) intact. Findings (3 gaps
de couverture, non bloquants) :

- **D2-1 (P3, CONFIRMED, phase-introduit)** — bras Err de `build()`
  (`dns_fallback.rs:236-238`) non couvert : 0.24 était infaillible, le
  `.map_err(NexusError::Endpoint)` neuf n'est exercé par aucun test (chemins
  valides réussissent, empty-endpoints court-circuité à `:206` avant `build()`).
  Branche défensive quasi-inatteignable (config valide construit toujours),
  module default-off. À documenter au body.
- **D2-2 (P3, CONFIRMED, pré-existant)** — assertion P2-E-1 faible
  (`dns_fallback.rs:529-571`) : `per_endpoint_tls_name_used_doh/_dot`
  n'assertent que `.label()`, pas le nom TLS effectif par endpoint. Boucle
  per-endpoint pourtant correcte (`:213-226`, `server_name` dérivé de
  `ep.tls_name` par itération) ; 0.26 n'expose pas d'accesseur public au nom
  TLS hors handshake -> `label()` = max atteignable. Faiblesse pré-existante
  (preflight §S2 #2), pas une régression.
- **D2-3 (P3, CONFIRMED, pré-existant)** — `resolve_txt_via`/`resolve_node`
  (`dns_fallback.rs:251-334`) 0-couverture (network-dépendants), y compris
  les branches réécrites `answers()`-vide->`bail!` (`:271-273`) et filtre
  non-TXT (`:264-269`). Seul le stub `DnsFallbackMock` (browse.rs:1651)
  exerce `resolve_node`, jamais le vrai chemin. Équivalence prouvée (D2 §4) ;
  préflight §388-394 corrobore (handshakes invisibles à T1, probe live opt-in
  recommandée). Note d'honnêteté au body.

## Dimension D3 — Sécurité Deep (CONFORME, 2 P3 dont 1 ex-P2 rétrogradé)

Diff sûr et factuel. `deny.toml` : retrait chirurgical des 4 ignores hickory
(plus aucune occurrence de RUSTSEC-2026-0119/0098/0099/0104), quick-xml
0194/0195 + rand + `[bans] multiple-versions="warn"` (P2-AUDIT-2-RESIDUEL)
INTACTS, `yanked="deny"` présent. Seuils EXACTS re-vérifiés contre l'advisory-db :
hickory-proto **0.26.1** (0119 DoS), rustls-webpki **0.103.13** (0098/0099
authentification + 0104 DoS CRL, seuil dur 0.103.13) ; lock : rustls **0.23.40**
unique (arbre 0.21.12 sorti), tokio-rustls 0.26.4. **aws-lc-rs/aws-lc-sys
ABSENTS** (`grep -c 'aws.lc'` = 0), ring 0.17.14 + webpki-roots 1.0.7 présents.
« 0 dep runtime neuve » prouvé mécaniquement : `git diff HEAD -- Cargo.lock |
grep '^+name = '` VIDE ; seuls `+version` = spin 0.9.9/0.10.1. THREAT_MODEL v18
purement additif (v15/v16/v17 non réécrites ; doc-comment « DNS is not a trust
anchor » intact). HARDENING `last_validated` re-daté S82 K (S81-G préservé en
Precedent), trigger standing hickory>0.26 ajouté, entrée `audited_findings`
additive. `trust_negative_responses=false` EXPLICITE (`:225`) ; P2-E-1 préservé
(`:215`) ; DNS reste non-ancre. Findings :

- **D3-1 (P3, ex-P2 DOWNGRADED, CONFIRMED)** — câblage webpki-roots (magasin
  de racines) non exercé par un handshake réel (`Cargo.toml:458` + `dns_fallback.rs`) :
  en 0.26 tls-ring/https-ring n'activent aucun root store, webpki-roots restaure
  seul les racines Mozilla ; ni cargo deny ni nextest ne montent de TLS. RÉEL
  mais rétrogradé P2->P3 : pas une régression ; pire cas échoue-FERMÉ (erreur
  resolve_node -> browse Unreachable -> pkarr Ed25519 ancre en aval, 0
  downgrade/bypass) ; surface opt-in default-off SANS appelant de production ;
  **déjà tracké** au preflight §UNVERIFIABLE (`:388-394`) avec la même
  recommandation de probe live opt-in. Porter le caveat au body (l'invariant
  « 0 changement observable » est vérifié compile/API, pas au TLS runtime).
  Voir D5-2 (même risque).
- **D3-2 (P3, CONFIRMED)** — `HARDENING_ROADMAP.md:30` « ... h2 0.3.27, http
  0.2.12 + 6 autres » sur-compte de 1 : sous-arbre retiré = 12 paquets, 6
  line-items nommés dont « hickory-* » couvrant 2 paquets -> 5 non nommés
  (enum-as-inner, linked-hash-map, lru-cache, rustls-pemfile, sct). Devrait
  être « + 5 autres ». Cosmétique, sans effet sur les faits de clôture
  d'advisory. À corriger à la volée si une passe édite ce bloc.

## Dimension D4 — Scope cuts + invariants sémantiques (CONFORME, 0 finding)

Les 6 sous-vérifications passent. **0 bump wire** : `grep -E '_VERSION'` sur
les 6 fichiers = vide, aucun canonical Task/ProjectAnnouncement/CuratorList/
FeedEntry/JCS touché. **D11 « 0 dep runtime hors hickory »** : `grep '^+name = '`
sur le diff lock = vide (0 nouveau package), webpki-roots = activation de
feature d'un crate préexistant (2 arêtes de deps, pas un crate), spin =
version-only. **iroh =1.0.1 INTACT** : `grep -E '^[+-]name = "iroh'` = vide.
HORS SCOPE respecté : P2-AUDIT-2-RESIDUEL (`multiple-versions="warn"`) non
touché, ignores quick-xml/rand intacts, `dns_fallback.rs` sans refacto
opportuniste (édits limités aux sites imposés par l'API 0.26),
browse.rs/mocks non touchés, `load_dns_fallback_from_env` non touché
(plancher opt-in default-off préservé). Drift plan (kickoff écrit
`nexus-shell-daemon`, réel = `nexus-core-rs`) consigné au preflight `:23-26`
+ dans `audited_findings`. Invariants sémantiques préservés : P2-E-1 (TLS name
per-endpoint), P2-E-2 (2 resolvers racés), `trust_negative_responses=false`
explicite load-bearing, port configurable (`conn.port = ep.port`), `-1` test
honnête, docs au passé immuable.

## Dimension D5 — Research Grounding (CONFORME, 2 P3 informationnels)

Le code suit **fidèlement, point par point**, le preflight PLAN-ADAPT : les
7 corrections du verdict toutes câblées (fichier réel `nexus-core-rs` ;
features `tls-ring/https-ring/webpki-roots` ; webpki-roots obligatoire ; API
réécrite `builder_with_config`/`options_mut`/`build().map_err` + enum local
`DnsTransport` + `trust_negative_responses=false` explicite ; extraction
`answers()`/`RData::TXT`/champ `txt_data` ; `-1` test ; docs rafraîchies passé
immuable). Points UNVERIFIABLE levés à la compilation verte (`record.data`
champ `:265`, `txt.txt_data` champ `:266`, ports forcés par `conn.port = ep.port`).
Seuils d'advisory load-bearing prouvés au lock + `cargo deny check advisories`
VERT sans les 4 ignores. Findings (informationnels, 0 action code) :

- **D5-1 (P3, CONFIRMED)** — nuance rustls-platform-verifier : le preflight
  §S1a:114 affirme à tort « rustls-platform-verifier serait un nouveau crate
  -> à éviter ». FAUX : déjà au lock (`Cargo.lock:7376`), tiré par iroh-metrics
  (`:4056`), noq-proto/quinn-family (`:5435`), reqwest (`:7129`) — hors chaîne
  hickory, pré-existant. MAIS la justification CODE ne propage PAS l'erreur :
  `Cargo.toml:452-454` fonde webpki-roots sur la préservation des roots Mozilla
  0.24, et `HARDENING_ROADMAP` audited_findings consigne verbatim la correction
  (« pre-existant via iroh-metrics, PAS ajoute par ce bump »). Recommandation
  webpki-roots correcte, nuance déjà tracée. Zéro action code.
- **D5-2 (P3, CONFIRMED)** — risque runtime « magasin de racines vide »
  (`Cargo.toml:458` + `dns_fallback.rs:128`) : mitigation webpki-roots câblée
  et documentée 3× (Cargo.toml + HARDENING 2026-07-15 + THREAT_MODEL v18
  l.1853), mais la sonde live opt-in recommandée par le preflight (`:392-394`)
  n'est pas jouée. Bornage PROUVÉ : opt-in default-off + zéro appelant de
  production (grep exhaustif : `new`/`with_dns_fallback`/`load_dns_fallback_from_env`
  tous test-only). Suivi optionnel à noter au body. Même risque que D3-1.

## Dimension D6 — Livrables + Patterns + frontière (test-acteur §6.12) (CONFORME, 0 finding)

Tous les livrables plan §Phase K (`sprint82_plan.md:266-279`) couverts et
prouvés au lock : construction resolver réécrite ✓ ; 4 ignores `deny.toml`
retirés ✓ ; 4 RUSTSEC vivants clos (seuils au lock + `cargo deny` vert) ✓ ;
remédiation yanked spin in-phase (classe S81-G) documentée ✓. Critère T1
machine atteint : Win 2099 ≥ 2095, Docker 2103 ≥ 2099, `cargo deny` vert ;
T2 = N/A conforme. **frontier_closure N/A PROUVÉ par l'acteur** : trait
`DnsFallbackResolve` (`:82-93`) hickory-free et INTACT (git diff ne le touche
pas) ; consommateur unique cross-crate = `browse.rs` via `Arc<dyn DnsFallbackResolve>`,
ne lit que `label()`/`resolve_node()` (signatures inchangées) ; `grep
hickory/TokioResolver/txt_lookup/RData` sur `crates/` = 0 hors `dns_fallback.rs` ;
DNS absent de web/src, sbfb-factory, routes loopback ; concret
`DnsFallbackResolver::new`/`with_dns_fallback` sans appelant de production
(tous `#[cfg(test)]`). Aucun ledger PATTERNS à solder (grep hickory/HICKORY-024
sur les 2 PATTERNS.md = 0 ; le carry est tracké HARDENING+THREAT, pas PATTERNS ;
la réécriture 0.26 = migration one-off, pas un pattern réutilisable). Langue
conforme par surface. Artefact preflight présent, verdict PLAN-ADAPT exploitable.

## Dimension D7 — Audit des suites + classes env (CONFORME, 1 P3 record-nicety)

Classements « flake env » tous défendables, causalité hickory->dispatch_loop
EXCLUE, seule suite « manquante » candidate (cargo deny complet) jouée verte
par ce review. **Q1** — les 3 tests Rust = classe iroh-networked Docker-on-Windows
documentée (`sigint_triggers_graceful` + `start_writes_running_json` -> Phase I
review:123-124 ; `boot_path_reenters` + `start_headless_boots` -> Phase J
review:103-105 ; loc `dispatch_loop.rs:646`) ; les 2 timeouts Vitest = classe
`vitest_env_variance` (`GpuConsentDialog` documenté verbatim Phase I:118-121 ;
`AddAnchorDialog` structurellement identique — Radix Dialog/portal jsdom 5000ms,
re-joué solo PASS). **Q2** — causalité EXCLUE sur 3 fondements : (1) le test
`boot_path_reenters` n'exerce AUCUN DNS (résolution en mémoire
`memory_lookup().add_endpoint_info`, commentaire `:685-687` « in-process there
is no pkarr to resolve them ») ; (2) `dns_fallback.rs` sans appelant de
production, non importé par `nexus-shell-daemon` ; (3) iroh utilise sa propre
hickory 0.26.1 pré-existante, INCHANGÉE par la phase (seul l'arbre legacy 0.24.4
retiré -> chemin DNS réel byte-identique). **Q3** — E2E web correctement
non-exigé (`git diff --name-only web/` = vide, leçon S81-J-1). Finding :

- **D7-1 (P3, CONFIRMED, record-nicety)** — le critère T1 machine du preflight
  (`sprint82_phase_k_preflight.md:433`) ne nomme que `cargo deny check advisories`,
  alors que la phase MUTE le lock (-12 crates net). Lacune de complétude bornée
  (retrait net -> licenses ne peut que rétrécir, sources inchangées, bans
  multiple-versions=warn réduit les duplicats). **REMÉDIÉ PAR CE REVIEW** :
  `cargo deny check` complet (cargo-deny 0.19.2, 4 sections) -> « advisories ok,
  bans ok, licenses ok, sources ok », exit 0. Double filet : GHA
  `supply-chain.yml:70-74` lance `cargo-deny-action@v2 command: check` (les 4)
  sur push master, indépendant du `ci.yml` cassé GTK. **Action minimale** : le
  body devrait consigner `cargo deny check` COMPLET 4/4 vert, pas seulement
  `advisories`, pour l'honnêteté du record d'une phase supply-chain.

## Table des findings (déduplication inter-dimensions, verdicts adversariaux)

| ID | Sév | Titre | Fichier:ligne | Dim | Verdict | Classe |
|---|---|---|---|---|---|---|
| K-1 | P2 | Scoping commit : exclure blueprint + 2 recherches untracked | verification_blueprint.md + 2 untracked | D1 | CONFIRMED | action pré-commit |
| K-2 | P3 | Cargo.lock churn ré-unification (bénin, D11 tenu) | Cargo.lock (windows-sys/heck/indexmap/syn) | D1 | CONFIRMED | cosmétique |
| K-3 | P3 | HARDENING « + 6 autres » (réel = 5) | HARDENING_ROADMAP.md:30 | D1+D3 | CONFIRMED | doc-nit |
| K-4 | P3 | Bras Err de build() non couvert (phase-introduit) | dns_fallback.rs:236-238 | D2 | CONFIRMED | coverage-gap |
| K-5 | P3 | Assertion P2-E-1 faible (pré-existante) | dns_fallback.rs:529-571 | D2 | CONFIRMED | coverage-gap |
| K-6 | P3 | resolve_txt_via/resolve_node 0-cov (pré-existant) | dns_fallback.rs:251-334 | D2 | CONFIRMED | coverage-gap |
| K-7 | P3 | webpki-roots non exercé par handshake (ex-P2) | Cargo.toml:458 + dns_fallback.rs | D3+D5 | CONFIRMED (DOWNGRADED P2->P3) | verif-completeness |
| K-8 | P3 | Nuance rustls-platform-verifier (déjà consignée) | preflight:114 -> HARDENING audited_findings | D5 | CONFIRMED | traçabilité |
| K-9 | P3 | cargo deny full hors T1 gate (re-joué vert) | preflight:433 | D7 | CONFIRMED | record-nicety |

Total confirmés : **0 P0 / 0 P1 / 1 P2 / 8 P3**. 1 finding rétrogradé
(K-7 webpki-roots P2->P3), 0 REFUTED. Doublons inter-dimensions fondus :
K-3 (D1-3 = D3-2), K-7 (D3-1 = D5-2).

### Réfutés / rétrogradés à la vérification (traçabilité)

- **webpki-roots non exercé (ex-P2 -> P3)** — scan D3 l'avait posé P2. La
  passe verify adversariale l'a rétrogradé P3 : pas un défaut code, pire cas
  échoue-FERMÉ (0 downgrade/bypass vu pkarr Ed25519 en aval), surface opt-in
  default-off sans appelant de prod, et caveat + carry recommandé DÉJÀ consignés
  au preflight §UNVERIFIABLE (`:388-394`). C'est une note de complétude de
  vérification + un nice-to-have carry, pas un bloqueur. Aucun finding P0/P1
  n'a été réfuté (il n'y en avait pas).

## Vérification §7.4 (suites, résultats main thread audités, pas relancés)

- **Rust Windows** : fmt --check 0 diff ; clippy --all-targets -D warnings
  0 warning ; **nextest 2099/2099 PASS 0-skip** (baseline 2100, `-1` = test
  type-vacant `build_resolver_rejects_unsupported_protocol` retiré et documenté ;
  plancher plan 2095 -> **2099 ≥ 2095** ✓) ; doctests VERTS ; build release
  nexus-shell-daemon VERT ; `cargo deny check advisories` VERT sans les 4 ignores.
- **Docker canonique sbfb-ci rust:1.94** (CARGO_TARGET_DIR=target-linux) : fmt
  VERT ; clippy VERT ; **nextest --no-fail-fast 2103 exécutés / 2102 PASS + 1
  FAIL** `sigint_triggers_graceful` (classe env Docker-on-Windows, re-joué solo
  PASS) ; les 2 FAIL du 1er run fail-fast (`start_writes_running_json`,
  `boot_path_reenters`) re-joués solo PASS ×2 ET PASS au run complet ;
  `boot_path_reenters` a aussi flaké UNE fois solo (stall 21,5s) puis PASS ×2
  (6,2s) -> classe env réseau, PAS régression (causalité hickory EXCLUE, D7 Q2) ;
  doctests VERTS ; plancher plan 2099 -> **2103 ≥ 2099** ✓.
- **Web** : lint VERT ; tsc VERT ; **test:unit 412** dont 2 timeouts jsdom
  5000ms sous charge (classe `vitest_env_variance` : `GpuConsentDialog` +
  `AddAnchorDialog`) re-joués solo 22/22 PASS ; test:coverage VERT ; build VERT ;
  size VERT ; scan-en-strings VERT (delta Vitest 412 inchangé).
- **Gates docs** : check-frontier-contracts / check-factory-docs /
  check-sharding-docs = 3× VERTS.
- **Complément review** : `cargo deny check` COMPLET (4 sections) re-joué VERT
  par la passe D7 (advisories/bans/licenses/sources ok, exit 0).

Compteurs finaux : **Win 2099/2099 ; Docker 2103 (2102 PASS + 1 flake env
solo-PASS) ; Vitest 412**. Cohérents avec le `-1` test unique et les baselines
annoncées (Win 2100->2099, Docker 2104->2103, Vitest 412 inchangé).

## Prochaine étape — Codex (gate BLOQUANTE review -> commit)

Review initialement PASS-PENDING : 0 P0/P1, 1 P2 (hygiène pré-commit
non-code) + 8 P3 (cosmétique/couverture/informationnel), tous CONFIRMED,
aucun défaut de code bloquant. Étape suivante = **Codex GPT-5.6 Sol
reasoning max** (`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max`),
output brut = `sprint82_phase_k_codex_review.md` (JAMAIS réécrit). Boucle
arrêtée à « CLEAN ou P2/P3 documentés ». Rappels pour la mise au commit :
(1) stager par chemins explicites les 6 fichiers + le préflight, JAMAIS
`git add -A` / `git commit -a` (K-1) ; (2) consigner au body : `cargo deny
check` COMPLET 4/4 vert (K-9), churn lock cosmétique bénin D11-tenu (K-2),
3 gaps de couverture assumés (K-4/5/6), caveat webpki-roots
non-testé-au-handshake + carry probe live opt-in (K-7/D5-2). Correction
optionnelle à la volée : « + 6 autres » -> « + 5 autres » (K-3).

## Codex reconciliation

Codex GPT-5.6 Sol reasoning max joué (round 1, output brut
`sprint82_phase_k_codex_review.md`, non réécrit) : **4 livrables
CONFIRMÉ + 2 PARTIEL, 0 GAP dur**. Tri des 2 PARTIEL :

1. **PARTIEL Livrable 3 (tests P2-E-1)** — « `per_endpoint_tls_name_used_doh/_dot`
   n'assertent que `label()`, pas le TLS name effectif ». C'est
   EXACTEMENT le finding **K-5** de cette review (P3, PRÉ-EXISTANT :
   faiblesse identique en 0.24, documentée au preflight §S2 #2 ;
   hickory 0.26 n'expose aucun accesseur public au TLS name hors
   handshake → `label()` = max atteignable sans refactor
   d'extraction hors-scope, refactor que la dimension D4 a
   précisément validé ABSENT). Classement : P3 documenté au body,
   pas de correction (un fix exigerait un refactor opportuniste
   contraire au scope de la phase).
2. **PARTIEL Livrable 4 (deny.toml « plus aucune occurrence »)** —
   les 4 IDs RUSTSEC restent cités dans le commentaire « Removed
   S82 Phase K ». **Faux positif induit par le prompt Codex**
   (l'exigence « plus AUCUNE occurrence » écrite dans le prompt
   était sur-large) : la note de retrait au passé immuable citant
   les IDs est le STYLE CANONIQUE du fichier (précédent verbatim
   « Removed S81 Phase G: RUSTSEC-2026-0097 ... » à deny.toml:56) ;
   Codex confirme lui-même que le tableau `advisories.ignore` parsé
   ne contient plus que les 2 quick-xml. Retirer les IDs de la note
   serait une PERTE de traçabilité, contraire au canon. Aucune action.

Corrections in-phase post-review consignées : **K-3** appliqué
(« + 6 autres » → « + 5 autres », HARDENING_ROADMAP.md:30) AVANT le
run Codex. Anomalie de périmètre notée par Codex (le fichier
`sprint82_phase_k_review.md` untracked absent de sa liste) : normal,
l'artefact review est écrit APRÈS la rédaction du prompt ; il est
stagé au commit avec les autres artefacts planning. Suites NON
relancées après K-3 (édit doc-only d'un fichier .md sécurité, aucun
code touché ; les gates docs restent verts). Boucle Codex arrêtée
round 1 : critère « CLEAN ou P2/P3 documentés » atteint.
