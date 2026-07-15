# Sprint 82 Phase M — Review (Workflow)

Date : 2026-07-15. Review ultracode = Workflow 12 agents sur 8 dimensions
(1 diff-intégral-ligne-par-ligne / 2 fidélité-du-golden / 3 sémantique-des-tests-golden /
4 sécurité-deep / 5 scope+covers / 6 research-grounding-préflight / 7 audit-des-suites-oracle /
8 patterns-livrables-conventions), chaque dimension = agent review + agent de
vérification adversariale de CHAQUE finding (opus-4-8[1m]) ; 3 lentilles
adversariales transverses ajoutées (SUBSTANCE/ANCRES, SÉVÉRITÉ-CALIBRATION,
RE-VÉRIF-DISQUE). Phase M = **refacto TEST-ONLY** : (1) golden de caractérisation
HTTP (9 tests `golden_http_*` en fin de `mod tests`), filet des splits N→S ;
(2) dédup des 4 constructeurs de routeur de test en UN constructeur paramétré
`build_test_router_ext` (3 axes : cors, web_root, `TestHeaders::{Inject,Raw}`)
+ 3 wrappers minces. **0 route path, 0 wire bump, 0 dep, production `build_router`
:246-543 byte-identique.** Covers `REFACTO-HTTP-TEST-HARNESS-DEDUP`,
`S82-TEST-REFACTO-NONREG`. Preflight verdict **PLAN-ADAPT**
(`sprint82_phase_m_preflight.md`, 5 corrections plan suivies).

Diff review (working tree, pré-commit, PAS encore committé) = **un seul fichier
de code** : `crates/nexus-shell-daemon/src/http.rs` (561 insertions / 44
suppressions, vérifié `git diff --stat` ; tous les hunks côté-old ≥ 4616, DANS
`mod tests` ouvert `:4528-4529`). Artefact `.planning/active/sprint82_phase_m_preflight.md`
(untracked, à stager avec la phase). HORS PHASE, non reviewés, non-défauts :
`verification_blueprint.md` (tracké modifié, édition PO mi-session, à statuer PO) +
`workflow_agents_app_conception_ultradeep_2026-07-15.md` +
`workflow_hub_product_conception_2026-07-15.md` (2 untracked recherche PO).
L'index est PROPRE (rien stagé → review bien pré-commit).

## Verdict: PASS

**PASS — gate Codex réconciliée (GPT-5.6 Sol round 1 : 6/6 livrables CONFIRMÉ,
0 GAP, 0 PARTIEL — cf. `## Codex reconciliation`).** Aucun P0/P1. Après
vérification adversariale (3 lentilles), réconciliation inter-dimensions et
re-vérification disque par le synthétiseur : **0 P2 + 7 P3 distincts, tous
CONFIRMED** (1 finding fondu = doublon inter-dimension avec ancre corrigée ;
1 lentille avait proposé un downgrade→NONE sur M-6, minoritaire → maintenu P3).
La refacto est **génuinement test-only** : les 6 hunks vivent tous dans
`mod tests`, la région production `build_router` (`:246-543`) est byte-identique
(0 hunk sous la ligne 4616), `nexus-core-rs` a un diff VIDE et les 15 constantes
`*_VERSION`/`*_FORMAT_VERSION` sont inchangées → **0 route path, 0 wire bump,
0 dep**. La dédup est **behavior-preserving** : la couche d'injection Inject
(token + Host insert-if-absent + `remove(ORIGIN)`, et surtout PAS de
`x-sbfb-feed-internal`) est BYTE-IDENTIQUE aux 2 copies supprimées, simplement
relocalisée dans le bras `TestHeaders::Inject` ; la posture Raw rend le routeur
prod nu (Origin non-strippé → les 5 tests CORS + le golden CORS pilotent un vrai
Origin) ; `build_test_router_with_web_root` conserve son retour `(Router, TempDir)`
(garde anti-GC du `ServeDir`). `build_test_router_with_cors` est PLIÉ (0 référence
résiduelle). Les 9 tests golden PASSENT (re-joués indépendamment : **9/9 PASS**,
457 skipped). Les 3 déviations documentées (a/b/c) sont TOUTES jugées saines
(cf. §Arbitrage). Verdict promu **PASS** après la gate Codex (section
`## Codex reconciliation` ci-dessous, ajoutée à la promotion).

---

## Dimension 1 — Diff intégral ligne par ligne (test-only, CONFORME, 1 P3)

Les 6 vérifications d'intégrité passent toutes. (1) La couche d'injection de
`build_test_router_ext` (`:4657-4669`) est BYTE-IDENTIQUE aux 2 corps supprimés
(ex-`build_test_router_with_cors` + ex-`build_test_router_with_web_root`), vérifié
`git diff -U20` + grep (1 SEUL bloc d'injection restant :
`contains_key(AUTH_HEADER_NAME)`→insert `TEST_TOKEN`, `contains_key(HOST)`→insert
`127.0.0.1:0`, `h.remove(ORIGIN)`, `next.run` — rien d'autre, aucun
`x-sbfb-feed-internal`). (2) Les 3 wrappers délèguent avec params EXACTS :
`build_test_router :4677 = _ext(state,&[],None,Inject)` ; `build_cors_test_router
:7693 = _ext(state,cors,None,Raw)` ; `build_test_router_with_web_root :8600 =
_ext(state,&[],Some(tmp.path()),Inject)`→`(router,tmp)`. (3) `build_test_router_with_cors`
totalement SUPPRIMÉ (grep = 0 résidu), call-site unique remplacé. (4) Aucune
ligne hors `mod tests` (:4528/4529) — 0 hunk sous 4616, `build_router` prod
`:246-543` intact, 0 route/wire/dep. (5) Imports hoistés dans `_ext`
(`HeaderValue` + `header::{HOST,ORIGIN}`), orphelin retiré de `_with_web_root`,
`clippy -D warnings` VERT Win+Docker corrobore l'absence d'import mort. (6) Ordre
des layers préservé (injection OUTERMOST ; bras Raw = routeur nu). Signature
`_ext` utilise `Option<&FsPath>` = miroir exact de `build_router` prod.

- **M-1 (P3, CONFIRMED — accepté-par-design)** — `golden_redact` applique
  l'allowlist par NOM DE CLÉ globalement sur tout l'arbre JSON (`:12684-12702`,
  match `:12688`, const `GOLDEN_VOLATILE_FIELDS :12677`). Conséquence théorique :
  si un split N→S exposait `node_id`/`revision`/`archive_hash` comme champ STABLE
  et signifiant, le golden le blanchirait. DÉFAUT ACCEPTÉ-PAR-DESIGN, pas un bug :
  (1) l'allowlist minimale-observée est documentée in-code comme le choix
  conservateur (tout AUTRE champ échoue encore) ; (2) aucune surface épinglée
  n'exerce ce cas — seul `directory_publish` (`:13047-13050`) porte ces 3 champs,
  tous légitimement volatils ; (3) le golden est un filet de détection de refacto,
  pas une spec. Aucune action. **Ce finding fond le doublon inter-dimension de la
  Dim 7 (dont l'ancre citée `:157,164` était FAUSSE — ce sont des champs de la
  struct AppState ; la vraie const est `:12677`, la fn `:12684`).**

## Dimension 2 — Fidélité du golden (CONFORME, 2 P3)

Chacune des ~18 GoldenCase correspond à une réponse RÉELLE du handler visé,
vérifié en lisant le handler : `health` (SCHEMA_VERSION=1 + `daemon_version
"0.1.0-test"`), `blob_serve` "invalid hash hex" 400 + les 6 en-têtes
CSP/COOP/COEP/CORP tirés de `csp.rs:33/36/39` via `blob_serve_csp_middleware`,
`shard_session {found:false,session:null}`, `seed_count {0,false,null}`,
`seed_invites {invites:[]}`, `kudos {project_id,total:0,contributors:[]}`,
`verify_chain {valid:true}`, `curators {entries:[],subscribed_curators:[]}`,
`unsubscribe deadbeef`→`BadPubkeyHex` 400 message exact, `directory/publish
catalog_len:0` (3 champs volatils rédigés), et les 422 extractor-reject dont le
1er champ requis du DTO est exact (group_id / project_id / participant / k /
project_name / project_id). Détection validée par assertions status+body+headers :
route débranchée→404/405 DÉTECTÉE, DTO altéré→body/erreur différente DÉTECTÉE,
middleware CSP perdu→header absent DÉTECTÉ, couche CORS supprimée→OPTIONS 405 sans
`access-control-allow-origin` DÉTECTÉ, fallback SPA supprimé→GET / 404 DÉTECTÉ.
Câblage réel via `build_test_router`→`build_router` (pas d'appel direct handler).
Couverture par domaine de split MET : N shard-session / O seed / P frost /
Q coordinator (dont verify_chain) / R curators / S publish (100 %) + CORS raw +
CSP blob-serve + SPA fallback.

- **M-2 (P3, CONFIRMED — densité du filet)** — ~11 des ~24 handlers déplacés en
  phases N→S n'ont AUCUNE GoldenCase dédiée (shard-session 4/6, seed 3, frost 2/4,
  coordinator 1, curators 1 ; publish 100 %). Le golden est un ÉCHANTILLONNEUR
  (≥1/domaine, contrat annoncé au banner `:12663-12669`), pas un filet exhaustif.
  Un re-pointage de route cassé est rattrapé par le COMPILATEUR (chemin
  `crate::<domaine>::<handler>` non résolu) → le trou résiduel réel = un drift
  SILENCIEUX (édition non-verbatim) sur un handler non-couvert. Atténuation forte :
  la discipline co-migrante déplace les tests HTTP existants (subscribe_curator,
  seed_request_peer, frost_round2/aggregate, coordinator_submit_result, shard
  mount/generate) qui passent EUX AUSSI par `build_test_router`→`build_router`.
  Non-bloquant ; recommandation optionnelle = +1 GoldenCase par handler à
  couverture unitaire fine avant le split N/O.

- **M-3 (P3, CONFIRMED — nuance de fidélité frost)** — Golden FROST
  (`:12902-12930`) = extractor-reject uniquement : les 2 cas (round1,
  trusted_dealer) postent `{}` → 422 produit par l'extracteur `axum::Json` AVANT
  le corps du handler. FROST est le SEUL domaine sans aucune GoldenCase 200
  (shard_session/seed_count/kudos/curators/publish_blob/directory exercent, eux, le
  corps). Conséquence : un drift du DTO EST capté (string d'erreur serde change),
  mais un drift de la réponse-corps frost (200 succès / 400 load_share) n'est PAS
  capté (jamais atteint). Le banner « prove the moved handlers answer identically »
  SURESTIME donc ce que le golden frost prouve : câblage, pas identité de réponse.
  Atténuation : les `frost_http_*` existants (round2:8808, aggregate:8892,
  trusted_dealer:8709) exercent la cérémonie complète et co-migrent en Phase P.
  Non-bloquant ; recommandation optionnelle = 1 GoldenCase frost à 200 (shares
  rédigées).

## Dimension 3 — Sémantique des tests golden (CONFORME, 1 P3)

Chaque test tourne sur un état frais (`mk_state` per fn, nœud iroh +
`DaemonHttpState` neuf) → indépendance cross-test OK. `golden_redact` traverse
Array (items) ET Object (récursion), nested couverts. Champs NON-volatils
assertés en clair (catalog_len:0, peer_count:0/self_seeding:false/self_pin_enabled:null,
total:0/contributors:[], entries:[]/subscribed_curators:[], health
status/schema_version:1/daemon_version) → aucun test tautologique. Les 6 cas 422
Text = messages serde EXACTS, continuation `\` correcte (aucun double-espace,
`cat -A` vérifié). Cap `to_bytes 1<<20` (1 MiB) >> plus gros body épinglé.
Comptage : +9 `#[tokio::test]` ajoutés, 0 supprimé, 0 `#[cfg(unix)]`, 0 corps de
test existant modifié. Suite golden PASS 4× (1 initial + 3 re-runs), 9/9 stable.

- **M-4 (P3, CONFIRMED — masquage de drift de TYPE)** — La rédaction
  inconditionnelle (`:12689 *val = String(GOLDEN_REDACTED)` quel que soit le type,
  appliquée à `got` `:12760` ET `want` `:12762`) masque un drift de TYPE sur les 3
  champs volatils. Scénario : si un handler déplacé sérialisait `node_id` en nombre
  JSON (12345) au lieu d'une string, les deux côtés deviennent `String("<VOLATILE>")`
  → `assert_eq` passe, le drift de type invisible. GRAVITÉ FAIBLE : (1) borné à 3
  champs d'identité intrinsèquement non-déterministes (leur rédaction est
  OBLIGATOIRE pour un golden stable) ; (2) un split behavior-preserving déplace
  l'impl Serialize verbatim → drift de type quasi-impossible dans le scope visé ;
  (3) un champ volatil ABSENT reste détecté (clés de Map divergent). Limite
  inhérente au golden-avec-champs-volatils, non corrigeable sans réintroduire
  l'instabilité. Aucune action.

## Dimension 4 — Sécurité deep (CONFORME, 0 finding)

CLEAN. Les 6 invariants requis tiennent. Posture Inject préservée (closure
byte-identique, relocalisée) — les 8 golden atteignent les handlers en 200/422/400
(pas 401/403) → token injecté authentifie encore. Posture Raw préservée (routeur
prod nu, Origin non-strippé) → `golden_http_cors_preflight_raw` PASS confirme
qu'Origin atteint la gate CORS. `_ext` n'ajoute QUE token+Host (strip Origin),
RIEN d'autre : `feed_insert_rejects_without_internal_header` (`:7973`, non touché)
asserte encore 403 sans `x-sbfb-feed-internal` → invariant intact. `AuthState::new(TEST_TOKEN)`
mono-sourcé et inchangé (dans `_ext`), partagé par les 3 postures. Les tests auth
401/403 vivent dans le crate CORE (`auth.rs:886-978`) — hors diff. Le golden CSP
reconstruit exactement `BLOB_SERVE_CSP` (`csp.rs:33`) et épingle le middleware sur
le chemin d'erreur blob-serve. Aucun secret réel dans les fixtures (le TEST_TOKEN
64-char n'est jamais échoué ; le `deadbeef` de curators = param de chemin
user-fourni, distinct du token). Re-run live 9/9 PASS.

## Dimension 5 — Scope + Covers (CONFORME, 1 P3)

Phase M génuinement 100 % test-only, les 2 covers réellement satisfaits.
`git diff` touche exactement 2 fichiers : `http.rs` (seul code) +
`verification_blueprint.md` (recherche PO hors-phase). `nexus-core-rs` = diff VIDE ;
les 15 constantes `*_VERSION`/`*_FORMAT_VERSION` inchangées vs HEAD → 0 wire bump.
`REFACTO-HTTP-TEST-HARNESS-DEDUP` : 4 constructeurs → 1 `build_test_router_ext`
(cors/web_root/`TestHeaders`) + 3 wrappers minces ; `build_test_router_with_cors`
plié (0 réf) ; `build_test_router_ext` = exactement 4 occurrences (1 def + 3
wrappers). `S82-TEST-REFACTO-NONREG` : 9 golden ajoutés, re-joués 9/9 PASS après
la dédup. Aucun scope creep (0 refacto handler, 0 move prod-code, 0 endpoint,
0 dep). Rien du plan §M abandonné.

- **M-5 (P3, CONFIRMED — hygiène de commit, non-code)** — 3 artefacts recherche
  hors-phase dans le working tree : `verification_blueprint.md` (M) +
  `workflow_agents_*.md` + `workflow_hub_*.md` (??). Ce sont des .md de recherche
  PO (« laisser intacte »), hors Phase M, sans impact sur la nature test-only.
  Scénario évitable : un `git add -A` les balaierait dans le commit test-only.
  Remède = staging sélectif `git add crates/nexus-shell-daemon/src/http.rs`
  (+ le préflight), jamais `git add -A`. Advisory de commit-hygiène assumé par le
  main thread ; 0 défaut dans le diff.

## Dimension 6 — Research grounding (préflight vs code) (CONFORME, 0 finding)

PASS. Le code implémente les 5 corrections PLAN-ADAPT de
`sprint82_phase_m_preflight.md` exactement. (1) Les 4 constructeurs re-dérivés par
NOM — le hors-glob `build_cors_test_router` traité en wrapper posture Raw,
`build_test_router_with_cors` plié (0 réf), 2 wrappers restants gardant les
178/5/4 call-sites intacts. (2) Le 3e axe `TestHeaders::{Inject,Raw}` et le retour
`(Router, TempDir)` préservés. (3) Exactement 9 golden, aucun `cfg(unix)`-gaté →
oracle EXACT 2099+9=2108 / 2103+9=2112. (4) `publish_blob` isolé (`:3261`) couvert.
(5) Chaque domaine + CORS-raw + CSP/COOP/COEP + SPA + verify_chain exercé.
L'angle-mort préflight `make_zip`/`archive_hash` est FERMÉ : `archive_hash` est
GOLDEN_REDACTED (`:13050`), NON épinglé → le non-déterminisme du last-modified du
zip ne peut pas faire flaker le golden.

## Dimension 7 — Audit des suites / oracle / non-régression dédup (CONFORME, 0 finding neuf)

`fmt --all --check` exit 0. `cargo nextest -E 'test(golden_http)'` → 9/9 PASS,
joué 2×, déterministe. Comptage exact : +9 `#[tokio::test]`, 0 supprimé, 0 corps
de test existant modifié — seuls 3 constructeurs re-câblés vers `_ext` (Inject/Raw)
+ 1 helper (`build_test_router_with_cors`) replié sans référence orpheline.
Arithmétique de l'oracle EXACTE : **Win 2099+9=2108, Docker 2103+9=2112, delta
cfg=4 préservé** (aucun `#[cfg]` ajouté). Équivalence sémantique des 3 wrappers
vérifiée ligne-à-ligne (mêmes args `build_router` + même posture layer, closure
middleware byte-identique). `git diff --stat` = 1 seul fichier code (http.rs) +
le blueprint hors-phase connu. **Le seul finding de cette dimension = doublon
substantiel de M-1 avec ancre `:157,164` ERRONÉE (réel `:12677`/`:12684`) → fondu
dans M-1.**

## Dimension 8 — Patterns, livrables et conventions (CONFORME, 2 P3)

Diff 100 % test-only, conventions du module respectées : doc-comments anglais
style `///` pour items + `//` pour banners (aligné TEST_TOKEN/mk_state), banner
`// ====` attesté partout dans `mod tests`, 0 emoji, named-constants §6.9 honoré
(`GOLDEN_VOLATILE_FIELDS`/`GOLDEN_REDACTED`), naming uniforme (`golden_http_*` × 9,
`golden_redact`/`golden_check`/`golden_run`, `TestHeaders`/`GoldenCase`/`GoldenBody`).
Les DEUX livrables §M pleinement livrés : golden verrouillant l'identité pre/post
sur 9 surfaces (6 domaines + CORS-raw + CSP + SPA) ; consolidation `_ext` (3 axes)
+ 3 wrappers minces à signature intacte + `_with_cors` plié. Aucune dette
introduite (0 defer, 0 band-aid). Anti STALE-PHASE-K RESPECTÉ (aucun « Phase N
fera X », le banner décrit un rôle enduring de filet).

- **M-6 (P3, CONFIRMED — doc différable)** — La caractérisation golden est
  génuinement NOUVELLE (aucune infra snapshot/golden préexistante) et les 6 phases
  N→S en dépendent, mais aucune entrée `docs/rust/PATTERNS.md` ne documente la
  technique (allowlist de rédaction, preuve « vert 2× sur HEAD inchangé »,
  normaliseur serde_json). VERDICT : NON requise pour M — le précédent Phase L
  (`013b611`, refacto pur) a ajouté 0 entrée PATTERNS (§P73 provient de S81).
  Documenter un §P74 golden-characterization est du scope Phase T (clôture
  docs-contrat) ou repliable dans la 1re phase N. Purement informatif ; aucune
  convention violée. *(1 lentille adversariale a proposé un downgrade→NONE au motif
  de « non-défaut auto-reconnu » ; minoritaire 2/3 → maintenu P3-record, statut
  « différé T ».)*

- **M-7 (P3, CONFIRMED — nit prospectif)** — `http.rs:12825` :
  `/// Shard-session domain (split target: 6 loopback handlers).` L'anti
  STALE-PHASE-K est RESPECTÉ (aucune phase nommée, aucune promesse de travail
  futur), mais « split target » est un descripteur présent qui deviendra
  faiblement périmé une fois Phase N réalisée (un lecteur verra « split target »
  sur un domaine déjà splitté). Le golden reste valide après le split.
  Reformulation optionnelle (décrire le domaine sans son statut pending) ; aucune
  action bloquante.

---

## Arbitrage — 3 déviations préflight jugées

- **(a) Allowlist de rédaction MINIMALE-OBSERVÉE** `[node_id, revision, archive_hash]`
  au lieu de l'allowlist large recommandée (tickets, signatures, `*_at`…) :
  **JUGÉE SAINE**. C'est le choix conservateur, empiriquement complet — j'ai
  énuméré les ~18 bodies golden : SEUL `directory_publish` (`:13047-13050`) échoue
  des données volatiles, et il en émet exactement 3 (node_id/revision/archive_hash).
  L'allowlist plus serrée est un renforcement (un drift sur TOUT autre champ échoue
  encore le golden), pas un gap. 4 runs verts sur nœuds frais, aucun leak volatil
  non-rédigé. Résidus honnêtement notés (M-1 masquage global par nom, M-4 masquage
  de type) sont bornés à ces 3 champs et sans objet pour un split behavior-preserving.

- **(b) PAS de tri récursif des clés** : **JUGÉE CORRECTE**. `serde_json::Map`
  `PartialEq` est key-order-insensitive sous les DEUX backends (BTreeMap trié /
  IndexMap set-equality) ; vérifié au lock : `serde_json 1.0.149` dépend d'`indexmap
  2.14.0` (tiré UNIQUEMENT par la feature `preserve_order`) → `IndexMap PartialEq`
  ordre-insensible. `assert_eq!` sur `Value::Object` est donc ordre-insensible ; le
  tri est sans objet. Les arrays golden restent en-ordre (corrects : tous vides `[]`).

- **(c) Hash publish-blob PINNÉ LITTÉRAL** `6e46dd10…c279` : **JUGÉE CORRECTE**.
  b3sum indépendant de `printf '{}'` (2 octets) = `6e46dd10defc9b56c29a6ec56b508
  c21f54c08192194e4df25bf36f0c9c3c279` == littéral golden. `publish_blob` lit le
  body brut et retourne `hex::encode(blobs.add_bytes(...))` = BLAKE3 → content-addressed
  déterministe, move-invariant, correctement NON-rédigé (oracle = valeur observée
  littérale, pas magic-number §6.9). Prouvé stable Win 2108 & Docker 2112.

## Table des findings (déduplication inter-dimensions, verdicts adversariaux)

| ID | Sév | Titre | Fichier:ligne | Dim | Statut | Verdict |
|---|---|---|---|---|---|---|
| M-1 | P3 | `golden_redact` allowlist GLOBALE par nom de clé — masquage latent si champ stable homonyme | http.rs:12684-12702 (const :12677) | 1+7 | documenté (accepté-par-design) | CONFIRMED (doublon Dim7 ancre corrigée) |
| M-2 | P3 | Densité filet : ~11/~24 handlers N→S sans GoldenCase dédiée (échantillonneur) | http.rs:12663-13130 | 2 | documenté (contrat ≥1/domaine tenu) | CONFIRMED |
| M-3 | P3 | Golden FROST = extractor-reject only (corps handler non exercé) | http.rs:12902-12930 | 2 | documenté (co-migrant frost_http_*) | CONFIRMED |
| M-4 | P3 | Rédaction inconditionnelle masque un drift de TYPE sur 3 champs volatils | http.rs:12689 | 3 | documenté (limite inhérente) | CONFIRMED |
| M-5 | P3 | Artefacts recherche hors-phase dans working tree — staging sélectif au commit | working tree | 5 | advisory main thread | CONFIRMED |
| M-6 | P3 | Doc pattern golden-characterization différable Phase T (§P74) | docs/rust/PATTERNS.md | 8 | différé T (précédent L) | CONFIRMED (1 downgrade minoritaire) |
| M-7 | P3 | Formulation « split target » légèrement prospective (nit) | http.rs:12825 | 8 | reformulation optionnelle | CONFIRMED |

Total confirmés : **0 P0 / 0 P1 / 0 P2 / 7 P3**. Aucun finding réfuté par ≥2
lentilles. Le finding `golden_redact global-by-key` apparaissait dans 2 dimensions
(1 et 7) → fondu en M-1, l'ancre erronée `:157,164` de la Dim 7 (champs de la
struct AppState) corrigée en `:12677`/`:12684`.

### Réconciliations / réfutations à la vérification (traçabilité)

- **Ancre M-1/Dim-7 — CORRIGÉE.** La Dim 7 citait `GOLDEN_VOLATILE_FIELDS` à
  `http.rs:157,164` : FAUX — re-vérifié disque, ces lignes sont des champs de la
  struct `AppState` (`task_dispatch_tx` `:160`). La vraie const est `:12677`, la fn
  `golden_redact` `:12684`. La SUBSTANCE (allowlist globale par nom, déviation (a)
  saine) est correcte et duplique M-1 → conservée CONFIRMED, ancre corrigée.
- **M-6 — 1 downgrade minoritaire NON retenu.** La lentille SÉVÉRITÉ-CALIBRATION a
  proposé downgrade→NONE (« non-défaut auto-reconnu, documentation légitimement
  différée »). Les 2 autres lentilles maintiennent P3-record. Majorité → **P3**,
  statut « différé Phase T » ; le précédent Phase L (0 edit PATTERNS pour un
  refacto pur) est sans ambiguïté et confirme qu'aucune entrée n'est due en M.
- **Aucun P2/P1/P0.** Les angles morts du golden (rédaction champs volatils M-1/M-4,
  échantillonnage 1/domaine M-2, frost extractor-only M-3) sont soit inhérents au
  golden-avec-champs-volatils (non fixables sans réintroduire l'instabilité), soit
  couverts par les suites HTTP co-migrantes — aucun n'atteint le seuil P2.

## Vérification §7.4 (suites, résultats main thread audités)

- **Rust Windows COMPLET VERT** : `fmt --check` OK ; `clippy --workspace
  --all-targets --locked -D warnings` OK ; **nextest workspace 2108 == 2099+9
  EXACT, 0-skip** (oracle préflight = `==`, pas `>=`) ; doctests VERTS ; release
  build OK.
- **Docker sbfb-ci COMPLET VERT** : `fmt --check` VERT + `clippy` VERT ; **nextest
  workspace 2112 == 2103+9 EXACT, 0-skip** ; doctests VERTS. Delta Docker−Win = 4
  (`#[cfg(unix)]`) préservé (aucun `#[cfg]` ajouté par la phase).
- **Web COMPLET VERT** : lint OK ; tsc OK ; **Vitest 412/412** ; coverage
  87.27 / 79.01 / 86.02 / 88.59 (≥ seuils) ; build OK ; size 129.02 / 130 kB OK ;
  `scan-en-strings` clean.
- **Factory Operator VERT** : tsc OK ; lint OK ; **Vitest 201/201**.
- **Golden ciblé** : `cargo nextest -p nexus-shell-daemon -E 'test(golden_http)'` →
  **9/9 PASS** joué 2× déterministe ; golden VERT 2× sur HEAD inchangé AVANT la
  dédup, puis re-VERT après la dédup (preuve de non-régression du filet).

Compteurs FINAUX : **Win 2108 (=2099+9) ; Docker 2112 (=2103+9) ; Vitest 412 ;
Operator 201**. Delta cumulé Rust **+9** — cohérent avec l'ajout des 9 golden,
0 test existant modifié/supprimé. Diff prod byte-identique (`build_router`
:246-543), 0 route path, 0 wire bump, 0 dep.

## Codex reconciliation

- Rapport : `sprint82_phase_m_codex_review.md` (GPT-5.6 Sol reasoning max,
  round 1, output brut non réécrit).
- Verdict : **6/6 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL — CLEAN round 1**
  (3ᵉ round-1-clean S82 après J et L). Vérification indépendante de Codex :
  SHA-256 du segment `build_router` :246-543 IDENTIQUE HEAD↔working-tree
  (`922f73eb…c628`) ; delta `.route()` / DTO serde / `#[serde]` / constantes
  `*_VERSION` / `nexus-core-rs` / `Cargo.toml`+`Cargo.lock` = 0 partout ;
  comptages re-dérivés (`#[tokio::test]` http.rs 177→186 soit +9, 0
  suppression ; `#[test]` sync 22→22 ; `#[cfg(unix)]` 0→0) ; arithmétique de
  l'oracle re-dérivée indépendamment (Win `nextest list` 2108 ; Linux 2079
  hors launcher + 33 launcher [dont l'unique test Unix `auth.rs:304`] = 2112) ;
  fmt PASS ; golden 9/9 joué 2× ; harness confirmé sans `x-sbfb-feed-internal`
  (test négatif feed-insert vert).
- Réconciliation : 0 GAP → boucle arrêtée round 1 (critère « CLEAN ou P2/P3
  documentés ») ; review promue PASS ; les 7 P3 de la review restent
  documentés tels quels (aucun contredit par Codex) ; suites non relancées
  (0 correction post-Codex).
