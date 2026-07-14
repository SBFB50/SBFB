# Sprint 82 Phase D — Review (Workflow)

Réparation/requalification de chaque test RED du set nextest
`binary(multi_daemon)` (baseline S81 A3 sous iroh 0.98 : 4/10 verts), en
distinguant le test-rot d'infra du vrai signal-produit. Review ultracode =
Workflow 10 agents (7 dimensions en parallèle + vérification adversariale
des findings P0/P1/P2 + synthèse). Le diff final est **test-only sur 4
fichiers** (3 code + 1 commentaire CI ; 0 bump wire, 0 dep, invariant
`heberger != publier` intact, garde d'auth S65 byte-identique). Le Workflow
a rendu **CONCERN** (0 P0/P1 ; 2 P2 doc-honesty tenus par l'adversarial) ;
les 2 P2 + 3 affinages de niveau note ont été **fixés in-phase** puis le
gate §7.4 complet a été re-joué vert — d'où le verdict ci-dessous.

## Verdict: PASS

Review Workflow OK (2 P2 doc-honesty fixés in-phase, re-gate §7.4 vert)
ET Codex réconcilié (cf. §Codex reconciliation) : le commit atomique
Phase D peut procéder. Aucun P0/P1 soulevé ni tenu à aucun stade. Les
P3/notes restants sont routés (commit body + wrap-up), listés plus bas.

## Codex reconciliation

Codex GPT-5.6 Sol (`model_reasoning_effort=max`), bundle auto-contenu
diff-inline (précédent Phase B), **round 1 : PASS — CONFORME/CLEAN sur
les 5 livrables D1..D5, 0 P0/P1, 0 P2/P3** (artefact brut
`sprint82_phase_d_codex_review.md`, output `codex exec -o` non réécrit).
Verdicts par livrable avec évidence ligne-diff : D1 zip STORED réel +
rename honnête + assert octets exacts, frontière S12 intacte ; D2 header
+ provenance aux 4 call-sites, `feed_sync.rs` absent du diff (garde 403
intact) ; D3 test hermétique un-gated 403/503, positive-control ancré
mk_state, usage `oneshot` correct (clone puis consume) ; D4 doc-module
conserve le disclaim relay, enregistre 4/4 + attribution iroh 1.0.1 sans
isoler de feature, 0 changement du corps gossip ; D5 diff limité aux 2
fichiers de test + module `#[cfg(test)]` + commentaires yml (0 manifest,
0 dep, 0 wire, 0 version, 0 garde production). Comptabilité jugée
cohérente (6 réparations / 0 requalification ; 10/10 sous
`SBFB_INTEGRATION=1`). Aucun GAP à corriger ni documenter : boucle Codex
close au round 1 (critère d'arrêt « CLEAN »), suites non re-lancées
(aucune correction post-Codex).

## Fixes in-phase (post-verdict Workflow CONCERN)

1. **P2-1 (doc-honesty, tenu par adversarial)** — le nom
   `test_cross_daemon_blob_transfer` revendiquait une couverture
   cross-daemon-over-iroh qui n'existe nulle part et n'a jamais existé
   (`spawn(1)`, publish + GET sur le même nœud ; la branche integration
   pré-Phase-D fetchait déjà le même unique daemon). Le de-gate Phase D
   rendait le misnomer load-bearing (vert à chaque push default). **FIXÉ** :
   renommé `test_blob_serve_local_zip_roundtrip` + commentaire de
   provenance du rename. Renommage vérifié safe par l'adversarial (aucune
   réf .yml/.sh/.toml/.md au nom ; `integration-nightly.yml` filtre par
   `binary(multi_daemon)`, pas par nom).
2. **P2-2 (doc-honesty, tenu par adversarial)** —
   `.github/workflows/integration-nightly.yml:16-18` affirmait encore un
   backlog S81-A3 « repairs tracked by the S81→S82 audit carry » que cette
   phase clôt : contradiction d'arbre post-commit entre deux descriptions
   du même class. **FIXÉ** : commentaire réécrit (« repaired in S82
   Phase D — the class ran 10/10 … guards that repaired state against
   regression »). Delta strictement comment-only (vérifié : toutes les
   lignes changées commencent par `#`) ; actionlint indisponible localement
   sur cette session (introuvable PATH/emplacements connus) — non requis
   pour un delta commentaire, à re-jouer au gate push Phase T.
3. **Notes (affinage, même passe)** : doc-module harness — « four feed
   tests (three in this file, one in `nexus-coordinator-rs`) » ;
   attribution de la convergence gossip adoucie (« attributed to the S81
   transport delta (iroh 1.0.1); the loopback measurement does not isolate
   which S81 feature closed it » — E3/Topologie B ne sont plus présentés
   comme cause opérante établie) ; blob explicitement exclu du self-skip
   class dans la doc-module ; commentaire du positive-control 503 ancré aux
   internals `mk_state{feed_sync_state:None}` (« re-anchor, don't
   delete »).

## Dimensions (verdicts Workflow sur le diff pré-fixes)

### Dimension 1 — Correctness ligne-à-ligne : CLEAN
`make_zip` (harness) utilise l'API zip correcte, miroir du helper
`http.rs:7944`, dep `zip` pré-existante (0 nouvelle dep, 0 change
Cargo.lock). Le round-trip blob exerce le vrai contrat S12 (publish zip →
200+hash → GET fichier interne → 200 + octets exacts) ; le de-gate
n'ajoute qu'un GET loopback local. Les 4 sites feed reçoivent le header
`x-sbfb-feed-internal:1` et seulement eux (publish-blob intact). Le test
hermétique atteint bien la branche 503 via `mk_state{feed_sync_state:None}`
(http.rs:4762), franchie seulement après le garde 403. Provenance ace05b0
(S65 Phase A) confirmée.

### Dimension 2 — Suites de vérification via artefacts : CLEAN
fmt rc=0, clippy `--workspace --all-targets -D warnings` rc=0, doctests
rc=0, release rc=0. Post-fix : test hermétique PASS ; `SBFB_INTEGRATION=1`
10/10 PASS ; default 10/10 PASS. Un unique FAIL au run #1
(`nexus-launcher token_rotation::rotates_after_interval`) : crate NON
touchée par le diff (dernier touche 94cccb2, Sprint 18), flake
horloge-murale documenté (poll 20 ms sur interval 80 ms pouvant observer
l'état après DEUX rotations sous charge), **disparu au retry full-green**
(2099 pass, 0 skip, rc=0). Compte workspace 2098 → 2099 (+1).

### Dimension 3 — Couverture sémantique des tests : CONCERN → résolu
P2-1 (nom blob) tenu → fixé in-phase (cf. supra). Le test négatif du garde
S65 épingle correctement l'auth ET atteint le vrai sujet : sans header →
403 (garde) ; avec header → 503 (`feed_sync_state: None`). Retirer le
garde ferait échouer l'assertion #1 ⇒ défend réellement le scénario
« régression silencieuse pré-S65 ». Les tests feed/gossip relay-gated
gardent leur early-return vert silencieux sur default CI : pattern S81-K
standing, disclaim par la doc-module, désormais backstoppé sur default CI
par le test hermétique 403.

### Dimension 4 — Scope cuts + conformité plan : CLEAN
Les 3 adaptations PLAN-ADAPT sont présentes : (1) test négatif hermétique ;
(2) blob vrai zip + de-gate ; (3) split 6/0 re-dérivé des runs frais.
**Zéro test requalifié** — conforme (requalifier un test-rot réparable =
scope-cut déguisé, interdit par le preflight). De-gate S1a direct-addr
correctement laissé hors-scope. Aucun creep : ajout http.rs entièrement
sous `#[cfg(test)]`, 0 wire, 0 dep.

### Dimension 5 — Sécurité (deep) : CLEAN
Garde d'auth S65 (`feed_sync.rs:597-619`) **byte-identique** (absent de
`--name-only`). Le diff n'ajoute que le header aux call-sites loopback
(chemin interne sanctionné, le harness détient le bearer du daemon) + un
test négatif réel = vraie couverture default-CI du chemin 403. Le
positive-control 503 échoue **visiblement** si `mk_state` change (jamais
vert silencieux dans la direction dangereuse). CSP blob-serve non assertée
dans le round-trip mais couverte de façon redondante par
`blob_serve_coep.rs` (un-gated). Header = defense-in-depth documentée,
commentaires clairs « internal-only ». Zéro surface production nouvelle.

### Dimension 6 — Doc-honnêteté + docs-contract (§6.12 test-acteur) : CLEAN
Comptage 4/4 convergence gossip exact (fresh + determinism, HEAD 2931b82) ;
la doc disclaim correctement la couverture relay et ne fait aucune claim
WAN/30 s. Précision d'attribution appliquée in-phase (cf. Fixes §3).
Frontières §6.12 : phase test-only, aucun acteur externe ne lit une
primitive nouvelle → N/A (aucune étiquette due). Aucun commentaire ne
promet de travail futur (anti STALE-PHASE-K respecté : provenance vers le
passé immuable seulement).

### Dimension 7 — Conventions + patterns : CONCERN → résolu
P2-2 (yml nightly) tenu → fixé in-phase. P3 routés : `.config/nextest.toml:77-81`
(K-R-13) décrit encore le class comme « boots real iroh nodes over the
network » alors qu'un membre (blob) est désormais local — clarification
optionnelle routée hors ce commit ; `CLAUDE.md:408-410` compte encore le
test blob parmi les 6 iroh-networked env-instables → réconciliation au
wrap-up S82 (avec le passage 2098→2099). Le test hermétique est
idiom-correct (`mk_state` + `build_test_router` + `oneshot`). Duplication
`make_zip` harness/http.rs acceptée (idiome test-helper local du repo,
pas de crate test-util partagée).

## Vérification adversariale des findings

| Finding | Tenu | Sévérité finale | Disposition |
|---|---|---|---|
| Nom `test_cross_daemon_blob_transfer` sans couverture cross-daemon (spawn(1) local, de-gate vert à chaque push) | Oui | P2 | **Fixé in-phase** (rename `test_blob_serve_local_zip_roundtrip`) |
| `integration-nightly.yml:16-18` affirme un backlog S81-A3 clos par Phase D | Oui | P2 | **Fixé in-phase** (commentaire réécrit, delta comment-only) |

Aucun P0/P1 soulevé ni tenu.

## P2/P3 à documenter dans le commit body
1. **Split final + note env (livrable plan)** : 6 réparations (5 test-rot
   code : blob + 4 feed-headers ; gossip_exchange constaté convergent sous
   iroh 1.0.1, 0 changement code) / 0 signal-produit ; delta 2098 → 2099
   (+1 test hermétique) ; note env Docker-on-Windows explicite.
2. **token_rotation flake** : FAIL run #1 pré-existant sans rapport
   (Sprint 18), disparu au retry full-green 2099/0 ; solo 4/4 PASS.
3. **P3 prose stale routés** : `.config/nextest.toml:77-81` (clarification
   optionnelle) ; `CLAUDE.md:408-410` (réconciliation wrap-up S82).
4. **actionlint indisponible localement** cette session (delta yml
   comment-only ; à re-jouer au gate push Phase T).

## Conformité plan + preflight PLAN-ADAPT
Les 3 adaptations PLAN-ADAPT sont honorées : (1) test négatif hermétique
du garde S65 en default-CI no-relay ; (2) blob réparé avec vrai zip
in-memory (`make_zip`, STORED) + de-gate 100 % local déterministe ;
(3) split 5/1 re-dérivé des runs frais → **6 réparations / 0
signal-produit**. Livrables plan : chaque red réparé (product-fix test-side,
zéro requalification puisque tous réparables), T1 non-régression +
multi_daemon ciblé vert atteint (10/10 default + 10/10 integration),
T2 = N-A respecté. Compte final + note env : dans le commit body.

## Suites (artefacts scratchpad session)
- `phase_d_fresh_run.txt` : run frais pré-fix HEAD 2931b82 (5 PASS/5 FAIL,
  gossip PASS).
- `phase_d_determinism.txt` : 3× repeat pré-fix (stable 5/5, gossip 4/4
  cumulé).
- `phase_d_verify.txt` : post-fix — hermétique PASS ; `SBFB_INTEGRATION=1`
  10/10 ; default 10/10.
- `phase_d_fullcheck{,2,3}.txt` : fmt/clippy/doctests/release rc=0 ;
  nextest workspace run #1 = 1 flake env token_rotation (crate non
  touchée) ; retry **full-green 2099/2099, 0 skip, rc=0**.
- `phase_d_final_gate.txt` (post-fixes review) : fmt 0, clippy 0,
  multi_daemon default 10/10 + integration 10/10, workspace **2099/2099**,
  doctests 0, release 0.
- Docker canonique dual-platform : gate de push groupé Phase T (pattern
  S82 ; env Docker-on-Windows instable documenté CLAUDE.md).
