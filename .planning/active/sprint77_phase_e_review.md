# Sprint 77 — Review Phase E

Routing DAG sweep (DP G->D) + Petals active churn (`replace_failed_server` +
heap fallback + `ActivationReplayCache` + `PerfMap` raw-op `(rho,tau)`) —
fichier net-new `crates/nexus-coordinator-rs/src/routing.rs` + 1 ligne additive
`pub mod routing;` dans `lib.rs`.

## Verdict: PASS

Aucun BLOCKER. Le seul CONCERN confirme (doc-honnetete SI-3 sur le chemin de
churn) a ete **resolu en-phase** par l'option (b) : `stage_tau` retire de la cle
d'ordonnancement du churn (nouveau `fallback_link_cost` rho-mesure-seul), `tau`
exclu, doc rendue litteralement exacte, et le test renforce pour PROUVER
l'exclusion (cf. `## Concern resolution`). Codex GPT-5.5 (verif cross-model) a
ensuite rendu **CLEAN au sens P0/P1** (cf. `## Codex reconciliation`). Les cinq
dimensions sont desormais PASS.

> Historique : ce fichier a ete ecrit en PASS-PENDING avec le CONCERN SEC-SI3
> ouvert ; il est promu PASS apres correction in-phase + Codex reconcilie.

## Tableau executif des 5 dimensions

| # | Dimension | Verdict | Findings | Note |
|---|---|---|---|---|
| 1 | Algorithmic correctness | PASS | 2 INFO | DP forward + backpointer sans off-by-one, saturating_add partout, PerfMap round-trip lossless |
| 2 | Scope-cut / Day-0 / preflight binding | PASS | 7 INFO | 6 contraintes liantes HONOREES, 0 creep Phase D/F, 0 bump wire |
| 3 | Test coverage semantics | PASS | 5 INFO | 18 `#[test]`, 4 noms d'invariants mandates presents et non-tautologiques |
| 4 | Security / threat-model (SI-3/SI-4/DoS/determinisme) | PASS (CONCERN resolu) | 1 CONCERN→FIXED + 6 INFO | churn passe a rho-mesure-seul (`fallback_link_cost`), `tau` exclu, doc exacte |
| 5 | Patterns / named consts / wire / commit-readiness | PASS | 3 INFO | miroir exact du house-style Phase D, gates verts localement |

## Dimension 1 — Algorithmic correctness (PASS)

Le DP forward de `route_min_latency` est un plus-court-chemin mono-passe valide
sur le DAG topologiquement ordonne par index de couche ; la reconstruction par
backpointers est correcte (pas d'off-by-one) et le chemin mono-stage (back vide,
boucle jamais executee) est correct. Tie-breaks entierement deterministes,
`saturating_add` garde chaque accumulation (pire cas ~7.9e12 << u64::MAX). Churn
(`replace_failed_server`, `assign_fallback_nodes`) min-pop le
`BinaryHeap<Reverse<(cost,pubkey)>>` correctement, exclut le noeud failed + les
non-membres, retourne un NOUVEAU `ShardPlan` sans mutation. Eviction
`ActivationReplayCache` oldest-first, refresh garde l'age (early-return).
`PerfMap` From/TryFrom round-trip u64->u64 lossless, `Eq` independant de l'ordre
(egalite de contenu `BTreeMap`). 18 tests verts.

- `[INFO] ALG-1 @ routing.rs:379-435` — CONFIRMED (INFO non-bloquant). Backpointer
  reconstruction tracee correcte ; tie-break lexicographique par-stage
  deterministe (meme `(req,perf)` -> meme chaine), claim load-bearing de la
  docstring tenu. Math `3 tau + 2 rho = 5000` verifiee.
- `[INFO] ALG-2 @ routing.rs:179-202` — CONFIRMED (INFO non-bloquant). Le cap
  `PERF_MAP_MAX_ENTRIES` s'execute APRES que serde a materialise le `Vec` wire
  complet ; nuance anti-DoS (1 alloc Vec avant rejet), pas un bug de correction,
  miroir du pattern `SHARD_PLAN_MAX_ASSIGNMENTS` existant.

## Dimension 2 — Scope-cut / Day-0 / preflight binding (PASS)

Les 6 contraintes liantes du preflight PLAN-ADAPT Phase E sont honorees, 0
scope-creep Phase D/F. PerfMap = raw-op genuinement non signe (serde pur, 0
`DOMAIN_*`/`canonical_bytes`/Ed25519 sur le type) ; aucun import iroh /
iroh-docs / `nexus_core_rs::docs` ; rho/tau strictement u64 micros, 0 f64, 0
rand ; tau advisory ; cache borne par const nommee ; `assign_fallback_nodes`
prend `&ShardPlan` et retourne un `ShardPlan::new(out)` frais. Wire 0-bump
confirme ; `placement.rs` et `nexus-core-rs` intouches.

- `[INFO] E-SCOPE-1 @ routing.rs:122-264` — CONFIRMED. Contrainte 1 (raw-op non
  signe) HONOREE.
- `[INFO] E-SCOPE-2 @ routing.rs:71-79,1021-1046` — CONFIRMED. Contrainte 2
  (split frontiere de crate, pas d'iroh) HONOREE ; `perf_map_republished_to_doc`
  = round-trip serialize/deserialize, pas un doc.set() networke.
- `[INFO] E-SCOPE-3 @ routing.rs:127,130,398,420,494,552` — CONFIRMED. Contrainte
  3 (u64 micros, deterministe, no rand) HONOREE.
- `[INFO] E-SCOPE-4 @ routing.rs:328-343,458-507,879-918` — CONFIRMED (cote
  allowlist). Contrainte 4 (allowlist-only SI-4) HONOREE ; voir SEC-SI3 pour la
  sous-clause « ordered by measured rho » qui, elle, n'est PAS litteralement
  tenue cote ordonnancement.
- `[INFO] E-SCOPE-5 @ routing.rs:95-100,564-635,1080-1109` — CONFIRMED.
  Contrainte 5 (cache churn borne, const nommee) HONOREE.
- `[INFO] E-SCOPE-6 @ routing.rs:524-562,996-1010` — CONFIRMED. Contrainte 6
  (fallback plan-time, re-sign nouveau manifest revision+1, pas de mutation
  in-place) HONOREE.
- `[INFO] E-SCOPE-7 @ routing.rs (whole) + git status` — CONFIRMED. Wire 0-bump
  + 0 creep cross-phase ; consent.rs intouche (scope cut #7).

## Dimension 3 — Test coverage semantics (PASS)

18 `#[test]`, 0 doctest => delta nextest = +18. Les 4 noms d'invariants de
fermeture mandates par le plan existent et exercent un comportement reel :
`routing_dag_sweep_selects_min_latency_chain` (678),
`churn_replaces_failed_server_oturn` (817), `perf_map_republished_to_doc` (1021,
correctement reformule en round-trip — honore la contrainte 2),
`routing_recomputed_on_perf_map_update` (786). Determinisme assere sur le chemin
DP (route twice -> equal, lignes 705/782). Propriete anti-SI-3/SI-4 testee. Aucun
test vacuous.

- `[INFO] E-TEST-01 @ routing.rs:466` — CONFIRMED (INFO). Garde
  chain-length-mismatch non couverte (son jumeau rho l'est).
- `[INFO] E-TEST-02 @ routing.rs:188` — CONFIRMED (INFO). Cap cote tau non
  exerce (logique byte-identique au cap rho teste).
- `[INFO] E-TEST-03 @ routing.rs:841` — CONFIRMED (INFO). Determinisme non
  assere par route-twice sur le chemin churn (pin par pubkey exacte a la place).
- `[INFO] E-TEST-04 @ routing.rs:582` — CONFIRMED (INFO). Branche
  `with_capacity(0)`->1 (`cap.max(1)`) documentee non testee directement.
- `[INFO] E-TEST-05 @ routing.rs:677` — CONFIRMED. Confirmation du compte : 18
  fns de test, 0 doctest, toutes non-vacuous. Le body de commit doit annoncer
  +18 cas nextest dans nexus-coordinator-rs.

## Dimension 4 — Security / threat-model (CONCERN)

Posture de fond saine : PerfMap raw-op non signe (0 nouveau DOMAIN_*), pas de dep
iroh donc pas de doc.set() possible, rho/tau u64 Eq-stable, DP/heaps
deterministes (BTreeMap/BTreeSet/BinaryHeap, 0 rand/time/HashMap), cap DoS
applique AVANT construction des maps, cache borne, admission SI-4 jamais relaxee
(gate `.contains` lignes 490/548), fallback = NOUVEAU ShardPlan re-signe
revision+1. Autorite d'integrite correctement placee en aval (RunProof, lignes
66-69). UN ecart d'exactitude documentaire.

- `[CONCERN] SEC-SI3-CHURN-DOC-OVERSTATES @ routing.rs:62-66, 325-343` —
  **CONFIRMED (real=true, verifie contre le code)**. La cle d'ordonnancement du
  churn INCLUT le tau self-reported, ce qui contredit la propriete de securite
  affichee. Trace concrete : `replace_failed_server` (routing.rs:493) et
  `assign_fallback_nodes` (routing.rs:551) ordonnent leur heap par
  `hop_marginal_cost` ; `hop_marginal_cost` (routing.rs:335) commence par
  `let mut cost = stage_tau(perf, worker, stage)`, et `stage_tau` lit
  `perf.get_tau(...)`, documente lignes 60/129-130 comme « self-reported
  (advisory) ». Donc tau est le terme MENEUR de la cle de churn, pas exclu. Trois
  endroits surestiment :
  - `routing.rs:63-66` (module doc) : « orders candidates by **measured rho** ».
  - `routing.rs:326-327` (`hop_marginal_cost` doc) : « ordered by **measured**
    cost ».
  - `routing.rs:456-457` (`replace_failed_server` doc) : « Ordering by measured
    rho (not self-reported tau) keeps the choice un-gameable » — contredit
    directement le code.

  Le test `replace_failed_server_orders_by_measured_rho_not_self_tau`
  (routing.rs:879-918) prouve seulement que l'honnete gagne quand l'ecart de rho
  mesure (79000us) ecrase l'ecart de tau (4999us) ; il ne prouve PAS que tau est
  exclu. Un membre malveillant peut encore biaiser le fallback de churn vers
  lui-meme en sous-declarant tau, borne uniquement par la marge de rho mesure.
  Divergence aussi avec la contrainte liante #4 et la prescription preflight
  S3-E4/S2-F (« ordonner par RTT MESURE (rho) ... PAS par tau self-reported »).

  **Severite CONCERN, pas BLOCKER** : le modele de confiance n'est PAS casse — le
  biais tau ne perturbe que l'optimisation de latence ; l'integrite est imposee
  en aval par RunProof + le gate allowlist (les deux corrects). Le probleme est la
  doc-honnetete : le module affirme une propriete « un-gameable / measured-rho
  only » qu'il ne livre pas. Vu la discipline doc-honnetete du projet, a corriger
  avant commit. Deux fixes valides : (a) correction de doc — dire que la cle de
  churn est tau+rho, qu'une sous-declaration de tau peut encore biaiser le churn
  dans la marge de rho mesure, et que l'autorite d'integrite reste le RunProof
  aval ; (b) durcir la cle d'ordonnancement du churn a rho mesure seul (retirer
  `stage_tau` du `hop_marginal_cost` de churn, en le gardant dans le DP de routage
  ou tau est intrinsequement requis et deja reconnu lignes 61-62), ce qui rendrait
  la doc litteralement vraie et honorerait la prescription preflight. Le DP de
  routage (`route_min_latency`) utilise legitimement tau et est correctement
  cadre — ce finding est scope au churn seul.
- `[INFO] SEC-SI3-RESIDUAL-NOT-IN-COMMIT-SOURCE @ routing.rs:57-69` — CONFIRMED.
  Le body de commit (## G8 traceability / ## Pre-launch protocol) est la source
  prescrite (preflight S3-E6) pour la note THREAT_MODEL §16 SI-3/SI-4 qui
  atterrit en Phase K. Aligner cette source sur la vraie cle tau+rho du churn
  pour que Phase K n'herite pas d'une sur-affirmation.
- `[INFO] SEC-SI4-ALLOWLIST-GATE-OK @ routing.rs:462,490,524-528,548` — CONFIRMED.
  Gate de collusion SI-4 correct et verifie (allowlist fournie par l'appelant,
  `.contains` sur chaque candidat, test negatif 862-876).
- `[INFO] SEC-DOS-CAPS-OK @ routing.rs:181-201,100,606-617` — CONFIRMED. Posture
  DoS saine sur chaque surface networke ; RoutingRequest/RoutingStage non
  Serde-derives (construits localement, deja capes en amont).
- `[INFO] SEC-DETERMINISM-OK @ routing.rs:71-72,398,418-420,488,546` — CONFIRMED.
  Determinisme (propriete de securite pour scheduling verifiable) pleinement
  satisfait.
- `[INFO] SEC-NO-PANIC-ON-NET-INPUT @ routing.rs:360-369,405,427` — CONFIRMED.
  Aucun panic drivable par l'attaquant ; les `expect()` sont des invariants
  internes post-validation.

## Dimension 5 — Patterns / named consts / wire / commit-readiness (PASS)

Miroir propre du house-style placement.rs (Phase D). Wire correct : PerfMap
raw-op non signe, 0 nouveau DOMAIN_*, 0 bump *_FORMAT_VERSION (seules mentions =
doc comments affirmant l'absence). rho/tau u64, DP+heap deterministes
`(cost, worker_pubkey)`, 0 rand. Les 5 magic numbers sont des const nommees
(`PERF_MAP_REPUBLISH_INTERVAL`, `PERF_MAP_MAX_ENTRIES`,
`ACTIVATION_REPLAY_CACHE_MAX`, `MISSING_TAU_PENALTY_MICROS`,
`MISSING_RHO_PENALTY_MICROS`). fallback_node peuple au PLAN time dans un NOUVEAU
ShardPlan, resign revision+1. lib.rs = +1 ligne additive en ordre alpha.

- `[INFO] E-PATTERNS-1 @ routing.rs:57-69 + commit body` — CONFIRMED.
  Commit-body readiness (preflight S3-E6) : la posture SI-3/SI-4 doit etre dans
  ## G8 traceability / ## Pre-launch protocol comme source pour THREAT_MODEL §16
  Phase K. Verifier le titre exact `feat(core): Sprint 77 Phase E — Parallax
  routing DAG + Petals active churn` avec les 9 sections de body (armement
  hooks lightcheck/Codex). **Note croisee SEC-SI3** : la posture a inscrire doit
  refleter la cle reelle tau+rho du churn, pas le « measured rho only ».
- `[INFO] E-PATTERNS-2 @ routing.rs (whole) + lib.rs:34` — CONFIRMED. Fidelite
  house-style exacte (PerfMap miroir RttMatrix, MISSING_*_PENALTY_MICROS miroir
  MISSING_RTT_PENALTY_MICROS, saturating_add, tie-break (cost,pubkey)) ; lints
  new_without_default / len_without_is_empty satisfaits.
- `[INFO] E-PATTERNS-3 @ routing.rs:122-264 + 509-562` — CONFIRMED. Wire/
  pre-launch : 0 bump, 0 nouveau DOMAIN_*, raw-op only ; cap anti-DoS dans
  TryFrom avant build des maps ; fallback additif dans ShardPlan::new(out).

## Binding constraints check

| # | Contrainte liante (preflight Phase E) | Statut | routing.rs |
|---|---|---|---|
| 1 | PerfMap = raw-op NON signe (pas de DOMAIN_*/canonical_bytes/Ed25519) | HONORE | `to_raw_op` serde pur :256 ; module doc :28-37 |
| 2 | Split frontiere de crate : pas d'import iroh/iroh-docs/docs/node | HONORE | imports :71-79 ; round-trip test :1021-1046 |
| 3 | rho/tau = u64 micros, deterministe, 0 rand/f64 | HONORE | :127,130 ; tie-break :398,420,494,552 |
| 4a | tau advisory, fallback tire UNIQUEMENT de l'allowlist (SI-4) | HONORE | gate `.contains` :490,548 ; test :862 |
| 4b | fallback churn « ordered by measured rho (pas tau self-reported) » | **HONORE (fix in-phase)** | `fallback_link_cost` rho-seul (n'appelle PAS `stage_tau`) ; doc exacte ; test `replace_failed_server_orders_by_measured_rho_not_self_tau` prouve l'exclusion (cas ou inclure tau inverserait le choix) |
| 5 | cache churn borne par const nommee, eviction oldest-first | HONORE | `ACTIVATION_REPLAY_CACHE_MAX=64` :100 ; eviction :612-616 |
| 6 | fallback plan-time, re-sign nouveau manifest revision+1, pas de mutation | HONORE | `&ShardPlan`->`ShardPlan::new(out)` :556-561 ; test :996-1010 |

Note : la contrainte 4 se decompose en deux clauses. La clause allowlist (4a) est
pleinement honoree. La clause d'ordonnancement (4b) est l'objet du CONCERN
SEC-SI3 : le code ordonne par tau+rho, pas par rho mesure seul, et la doc
l'affirme a tort. Fix avant commit (doc-correction OU retrait de `stage_tau` du
`hop_marginal_cost` de churn).

## Test delta

**+20** fonctions `#[test]` net-new dans `crates/nexus-coordinator-rs/src/routing.rs`,
**0 doctest** => delta nextest = **+20** (base nexus-coordinator-rs 305 -> 325 ;
nextest -p observe 325/325 puis 326/326 apres les 2 tests ajoutes en
reconciliation Codex). Workspace Windows nextest 1863 -> **1883**. Le body de
commit annonce ce delta `base+20`. Toutes non-vacuous.

> La review initiale comptait +18 ; +2 tests ont ete ajoutes en reconciliation
> Codex (`perf_map_raw_op_rejects_oversized_tau` round-1,
> `replace_failed_server_rejects_chain_length_mismatch` round-2). Le test
> renforce `replace_failed_server_orders_by_measured_rho_not_self_tau` (preuve
> d'exclusion de tau) fait partie du fix SI-3, pas un net-new.

## Residual concerns

1. **SEC-SI3-CHURN-DOC-OVERSTATES (CONCERN, confirme reel)** — la doc churn
   (lignes 62-66, 326-327, 456-457) affirme un ordonnancement « measured rho
   only / un-gameable » alors que la cle reelle est `stage_tau`(self-reported)+rho
   (ligne 335). A resoudre AVANT commit : soit correction de doc honnete (cle =
   tau+rho, biais SI-3 attenue-pas-elimine, integrite = RunProof aval), soit
   durcissement (retirer `stage_tau` de la cle de churn). Le modele de confiance
   n'est pas casse (allowlist + RunProof tiennent) — c'est de la doc-honnetete.
2. **Alignement commit-body / Phase K (INFO)** — la posture SI-3/SI-4 inscrite
   dans ## G8 traceability / ## Pre-launch protocol doit refleter la cle reelle
   tau+rho (source pour THREAT_MODEL §16 Phase K), pas l'over-claim.
3. **Couverture de branches mineures (INFO, non-bloquant)** — garde
   chain-length-mismatch (:466), cap cote tau (:188), `with_capacity(0)`->1
   (:582), determinisme route-twice sur le chemin churn : tous des INFO dont les
   jumeaux sont testes. Add optionnel, pas bloquant.

## Concern resolution

**SEC-SI3-CHURN-DOC-OVERSTATES — RESOLU en-phase (option b, durcissement code).**
La review a confirme que la cle d'ordonnancement du churn incluait `stage_tau`
(self-reported), contredisant la doc « measured rho only » et la contrainte
liante #4b. Correction appliquee :
- Nouveau `fallback_link_cost(perf, worker, prev, next)` = **rho mesure seul**
  (`link_rho` du/vers les voisins), **n'appelle PAS** `stage_tau`. Les deux
  appelants churn (`replace_failed_server`, `assign_fallback_nodes`) l'utilisent.
- Le routing DP (`route_min_latency`) garde `tau` (il minimise la latence ; tau y
  est advisory et l'autorite d'integrite reste le RunProof aval) — distinction
  desormais explicite dans le module doc « Advisory tau, measured rho ».
- Doc rendue litteralement exacte (module doc + `replace_failed_server` +
  `fallback_link_cost`).
- Test `replace_failed_server_orders_by_measured_rho_not_self_tau` renforce :
  valeurs choisies pour qu'inclure `tau` choisirait le menteur, mais rho-seul
  choisit l'honnete → PROUVE l'exclusion (pas seulement la domination).
Re-valide : clippy -p + nextest routing verts.

## Codex reconciliation

Codex GPT-5.5 (`codex exec`, cross-model, raw output dans
`sprint77_phase_e_codex_review.md`). **2 rounds.**
- **Round 1 — GAPS** : (P1) `routing.rs` untracked [artefact pre-commit, resolu au
  `git add`] ; (P1→reclasse P2) cap DoS « avant allocation » imprecis [reframe :
  cap par-collection AVANT build des BTreeMap = defense-in-depth ; taille brute
  bornee en amont par l'enveloppe FeedEntry ; doc corrigee] ; (P2) doc
  `assign_fallback_nodes` None [doc clarifiee] ; (P2) token `f64` en commentaire
  [reformule] ; (P2) branche tau-oversized non testee [test
  `perf_map_raw_op_rejects_oversized_tau` ajoute].
- **Round 2 — CLEAN au sens P0/P1** : livrables 1-7 CONFIRME sauf 2 PARTIEL
  documentaires, tous traites :
  - L4 PARTIEL « churn pas strictement O(t) (BinaryHeap) » → doc clarifiee :
    `O(t)` = remplacement d'**un** stage independant de la longueur L du
    pipeline ; selection per-stage `O(R)` sur les R replicas du stage via le heap
    fallback (le heap honore le libelle plan « heap fallback ordonne par
    latence »).
  - L7 PARTIEL « branche chain-length-mismatch non testee » → test
    `replace_failed_server_rejects_chain_length_mismatch` ajoute.
  - P2 « routing.rs untracked » → resolu au commit (git add).
  Verdict final reconcilie : **CLEAN P0/P1, P2 fermes (sauf untracked = commit)**.

## Suite results

- `cargo fmt --all --check` : **vert** (Win + Docker canonique).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : **vert**.
- `cargo nextest run --workspace --locked` (Windows) : **1883** (1863 + 20),
  0 skipped.
- `cargo test --workspace --locked --doc` : vert (0 doctest net-new).
- `cargo build -p nexus-shell-daemon --release` : vert.
- Docker canonique `sbfb-ci` rust:1.94 (`-p nexus-core-rs -p nexus-coordinator-rs`) :
  **723/723** 0 skipped sur le code final 20-tests (dual-platform confirme).
- `cargo nextest run -p nexus-coordinator-rs -E 'test(routing)'` : **20/20 PASS**.
