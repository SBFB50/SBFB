# Sprint 81 Phase C — Review (Workflow ultracode + agent de synthèse)

> Phase C « iroh-docs deep » (`sprint81_plan.md:147-163`, supersédée par
> le préflight PLAN-ADAPT `sprint81_phase_c_preflight.md` — la lettre du
> plan décrivait un re-typage `iroh-base 0.100` que le bump Phase B
> (`c899d54`) a déjà absorbé ; le VRAI périmètre code se réduit à
> **3 items** : [CŒUR] fix P2-SIBLING-SYNC-SET, [BLOQUANT] gate duress
> des 3 docs, [DOC-ONLY] recalibrations 0.98→0.101). Arbre SALE, HEAD
> `c899d54`, diff NON committé (8 fichiers modifiés + 1 untracked
> planning, `git status --short` = 9 lignes). 7 dimensions de review +
> 7 vérifications adversariales.

## Verdict: PASS

> **Diff Phase C substantivement CONFORME au contrat préflight qui
> supersede la lettre.** 0 P0 / 0 P1 sur les 7 dimensions après
> réconciliation adversariale. Le fix est point-unique et correct : un
> chokepoint `start_sync(Vec::new())` fail-fast placé au VRAI point de
> fusion des bras de chaque boot fn — storage (`runtime.rs:2634`), feed
> (`:2752`), project A4 réconcilié (`:2082`) — après le `let (doc,
> ticket_str) = match existing {…}` et AVANT le `Ok(…State {…})`,
> idempotent sur les bras create déjà armés par `share_write()`. Le gate
> duress (`noop_identity::sync_set_entry_in_duress`, enum
> `SyncSetOutcome::{Enter,Skip}` — jamais un bool) est appliqué aux
> 3 docs via un prédicat unique câblé à `opts.identity_mode` réel :
> sous duress la clé est un leurre mais le store/DB sont les VRAIS,
> donc un `start_sync` inconditionnel re-dialerait ≤5 pairs persistés
> par doc + servirait le vrai replica sous la clé leurre — le skip ferme
> cette régression DURESS-BOOT-LEAK (§15.1) et **ferme en prime la fuite
> latente du project doc A4 ungated depuis `fdb8ad7`** (amélioration
> sécu nette). Le discriminateur A2 (`NotFound → recreate loud / autre
> Err → fail-fast`) est préservé, le CONTROL A4 (tripwire :557) reste
> byte-intact et non-convergent, aucun `db.lock()` n'est tenu à travers
> un `await`. Le sweep doc-only est honnête (http.rs `remote_info_iter`
> reformulé sur le FOND — vérifié TOUJOURS absent en 1.0.1, pas sed'd ;
> doc_sync.rs « mechanism unchanged » ; Cargo.toml comment-only). Les
> 9 tests neufs (6 sibling CONTROL/GREEN/duress via
> `run_sibling_reopen_scenario` + 2 DocTicket round-trip/hostile core +
> 1 noop_identity unit) sont logiquement sains et red-before-green.
>
> **PASS-PENDING = review OK, Codex PAS ENCORE JOUÉ** (gate bloquante
> review → Codex → commit). En sus de Codex, **2 P2 consolidés**, aucun
> ne bloquant le commit SI le body les honore :
> **(1)** la branche Skip du **project doc** sous duress
> (`runtime.rs:2082`) — le doc le plus sensible (task/result) et le
> vecteur A4 explicitement désigné par le préflight §5.2 — n'est
> exercée par AUCUN test d'intégration (seul `IdentityMode::Normal` est
> piloté à `dispatch_loop.rs:694`) ; sa correction est prouvée 3× par
> identité structurelle avec les chokepoints storage/feed testés en
> duress + l'unit test, mais la branche sécurité n'a pas de tripwire
> propre → documenter au body OU ajouter un test project-doc-duress.
> **(2)** les 2 tests négatifs duress affirment `!converged` avec une
> borne **8s** alors que le GREEN positif exige jusqu'à **45s** sous
> charge (même chemin `start_sync`) → une régression future du CÂBLAGE
> duress convergerait dans la fenêtre 8-45s et le test PASSERAIT à tort
> (false-GREEN masquant DURESS-BOOT-LEAK) ; le code courant est correct
> et l'unit test garde le prédicat pur, mais le tripwire d'intégration
> est timing-faible → carry harness/K + documenter au body.
>
> Les 8 P3 sont documentables au body (doc-comment résiduel create-arms,
> reformulation commentaire 8s, budget terminate-after, mutation M8
> recreate duress, log littéral « duress », branches create/ticket-None
> non exercées, delta réel +9 vs estimation §7, pattern nextest §P7x→K).
> 1 finding **REFUTED** écarté avec trace (nextest slow-timeout « masque
> une régression »). Séquence : honorer (1)-(2) au body → Codex →
> réconciliation → promotion PASS → commit `fix(daemon)`.

## Portée du diff (constats croisés des 7 dimensions)

`git diff --stat` = **8 fichiers modifiés + 1 untracked planning**,
572 insertions / 50 suppressions, 0 fichier parasite (exactement les
8 attendus par le préflight §12) :

- `crates/nexus-shell-daemon/src/runtime.rs` (+173/−) — **CŒUR** :
  chokepoint `start_sync(Vec::new())` fail-fast au point de fusion de
  `boot_storage_namespace` (`:2634`) et `boot_feed_namespace` (`:2752`),
  gate duress des 3 docs (project `open_project_doc_for_dispatch:2082`
  réconcilié + storage + feed) via `sync_set_entry_in_duress`, signatures
  boot fns passées `pub(crate)` + 1 param `identity_mode` (Voie 1 §7),
  4 call-sites A2 tests passés `IdentityMode::Normal`, doc-comments
  recalibrés 0.101 + note résiduelle ticket-None.
- `crates/nexus-shell-daemon/src/noop_identity.rs` (+49) — helper
  `sync_set_entry_in_duress(mode) -> SyncSetOutcome::{Enter,Skip}` +
  doc module + unit test `duress_mode_skips_sync_set_entry` +
  assert `Enter` ajouté au test Normal existant.
- `crates/nexus-shell-daemon/src/dispatch_loop.rs` (+299/−) — 6 tests
  sibling via `run_sibling_reopen_scenario(kind, reopen)` (Storage/Feed
  × CONTROL `OpenDocDirect` / GREEN `BootFnNormal` / duress
  `BootFnDuress`), re-dial keepalive prod-fidèle (loop `doc_b.start_sync`
  toutes les ~5s jusqu'au deadline), CONTROL A4 :557 comment re-daté
  0.101 **corps intact**, convergence #5 :694 param Normal ajouté.
- `crates/nexus-core-rs/src/docs.rs` (+50) — 2 tests DocTicket :
  round-trip `mint→to_string→parse` (NamespaceId préservé + idempotence)
  + hostile ×3 → `Err`.
- `.config/nextest.toml` (+21) — test-group `two-node-convergence`
  max-threads=2 + override filter `test(/(convergence_|without_start_sync|reenters_sync_set|duress_skips_sync_set)/)`
  slow-timeout 60s×3.
- `crates/nexus-core-rs/src/doc_sync.rs` (+14/−) — note doc-only
  « mechanism unchanged » 0.101.
- `crates/nexus-shell-daemon/src/http.rs` (+12/−) — reformulation de
  fond `diagnostic_neighborhood` (`remote_info_iter` TOUJOURS absent 1.0.1).
- `crates/nexus-core-rs/Cargo.toml` (+4/−) — commentaire pkarr recalibré
  3-arg 1.0.1, **0 dep**.
- `?? .planning/active/sprint81_phase_c_preflight.md` (planning, relu).

**INTOUCHÉS prouvés par absence du diff** : `canonical.rs`, `node.rs`,
tous les `DOMAIN_*_V1` / `*_FORMAT_VERSION` (grep du diff entier = 0
hit), les matchers A2 `contains("Replica not found")`, le corps du
CONTROL A4, la string DocTicket servie au front (`storage_api.rs` /
`feed_sync.rs`), les endpoints de parse JOIN, `web/`, `tools/`.

## Dimensions et findings (7 review + 7 vérifications adversariales)

| Dimension | Verdict review | Verdict adversarial | Findings survivants |
|---|---|---|---|
| **diff-correctness** (diff ligne par ligne) | CLEAN | 2× CONFIRMED P3 | P3-1 (doc-comment create-arms), P3-2 (commentaire 8s imprécis) |
| **branch-coverage** (chokepoints × Enter/Skip, helper, ticket) | CONCERNS | 1× CONFIRMED **P2** + 1× CONFIRMED P3 | **P2-A** (project doc duress non testé), P3-6 (recreate/first-boot/ticket-None non exercés) |
| **harness-quality** (`run_sibling_reopen_scenario` + re-dial + nextest) | CLEAN | 2× CONFIRMED P3 | P3-2 (commentaire 8s), P3-3 (budget 180s serré) |
| **security-duress** (duress + fail-fast + DoS-at-boot) | CLEAN | 2× CONFIRMED P3 | P3-4 (recreate mute M8 duress), P3-5 (log littéral « duress ») |
| **research-grounding** (préflight ↔ code) | CLEAN | 1× CONFIRMED P3 | P3-1 (résidu share_write create-arms) |
| **scope-cuts** (scope + livrables + carries) | CONCERNS | 1× CONFIRMED **P2** + 1× **REFUTED** + 1× CONFIRMED P3 | **P2-B** (8s vs 45s false-green), P3-7 (delta +9 body) ; REFUTED : nextest slow-timeout « masque régression » |
| **patterns-docs** (patterns + docs-contract §6.12) | CLEAN | 1× CONFIRMED P3 | P3-8 (pattern nextest §P7x → K) |

**Recouvrements inter-dimensions dé-dupliqués** : le sujet « 8s vs 45s »
apparaît en diff-correctness P3, harness-quality P3 et scope-cuts P2 —
consolidé en **P2-B** (substance : gap de robustesse du tripwire) +
**P3-2** (facette : commentaire imprécis). Le sujet « share_write
inconditionnel des bras create/recreate sous duress » apparaît en
diff-correctness, research-grounding, security-duress et branch-coverage
— consolidé en trois axes distincts : **P3-1** (complétude du
doc-comment), **P3-4** (mutation d'état M8), **P3-6** (couverture de
test). Tous partagent une racine bénigne : les bras create mintent un
namespace FRAIS (0 pair persisté → 0 dial, 0 contenu réel → 0 serve),
donc entrer LEUR sync-set n'est pas une fuite ; le seul vecteur sur le
VRAI replica est le sous-arm ticket-None, prod-inatteignable (tout write
M8 depuis S58 persiste `Some(ticket)`).

## Findings retenus (après réconciliation adversariale)

### P0 — aucun

### P1 — aucun

### P2 (2, à documenter au body ; ni l'un ni l'autre ne bloque si le body les honore)

**P2-A — Branche Skip du project doc sous duress non exercée par un test
d'intégration** (branch-coverage, adversarial CONFIRMED).
`runtime.rs:2082` gagne le bras `Skip` (duress → pas de `start_sync`),
mais AUCUN test ne pilote `open_project_doc_for_dispatch(Duress)` : les
6 scénarios `run_sibling_reopen_scenario` ne couvrent que storage/feed,
et convergence #5 (`dispatch_loop.rs:694`) ne pilote que le bras `Enter`
(Normal) du project doc. Or c'est le doc le plus sensible (task/result)
et §5.2 du préflight le désigne comme la fuite « probablement déjà
livrée en A4 » ; le gate 503 (`task_dispatch_in_duress`) bloque le
dispatch neuf mais PAS le re-dial/serve du replica task/result persisté
sous la clé leurre — exactement ce que le skip ferme. **Failure** : si
le bras project doc régressait un jour vers `Enter` sous duress, aucun
test ne le rougirait — DURESS-BOOT-LEAK ré-ouvert silencieusement.
**Pourquoi P2 et non P1** : la correction est prouvée 3× (identité
byte-structurelle avec les chokepoints storage `:2636` / feed `:2752`
qui SONT testés en duress via `boot_{storage,feed}_namespace_duress_skips_sync_set_entry`
+ unit test `noop_identity::duress_mode_skips_sync_set_entry`) — ce
n'est pas un bug vivant. **Pourquoi P2 et non P3** : prod-exécuté
(`runtime.rs:648` passe l'`identity_mode` réel) + sécurité, pas
cosmétique. **Disposition** : documenter la lacune au body OU ajouter
`open_project_doc_for_dispatch_duress_skips_sync_set` (le plus propre —
même harness, `IdentityMode::Duress`).

**P2-B — Test négatif duress = borne 8s vs enveloppe positive 45s →
false-green sous charge CI** (scope-cuts + harness, adversarial
CONFIRMED). `dispatch_loop.rs:920/924` : `BootFnNormal => 45s`, `_ => 8s`
(le `_` couvre CONTROL `OpenDocDirect` ET duress `BootFnDuress`). Le
CONTROL est structurellement non-convergent (`open_doc` n'entre jamais
le sync-set → rejet catégorique `NotFound` à chaque re-dial,
indépendant du timing). MAIS le chemin duress exécute le MÊME boot fn
gaté que le GREEN : une régression future du câblage (miswire du `match`
dans `boot_storage/feed_namespace`, NON couvert par l'unit test pur)
ferait `Enter` sous duress → convergence exactement comme le GREEN, qui
demande démonstrablement jusqu'à 45s sous charge (l'historique de la
tâche : « single-dial 20s/45s runs each flaked once »). Convergence dans
la fenêtre 8-45s → `loop` break `false` → `assert!(!converged)` PASSE →
régression manquée. Le commentaire `:921-923` (« Load only makes
convergence SLOWER, so a short bound stays conservative ») est correct
UNIQUEMENT pour le false-RED, muet sur le false-GREEN. La fenêtre 8-45s
est le régime de charge ORDINAIRE documenté, pas « extrême ». **Pourquoi
P2 et non P1** : le code de prod est correct, le prédicat est
indépendamment et déterministiquement unit-testé, le test d'intégration
attrape toujours la régression sur run idle/solo (~2s < 8s), et aucun
fix trivial de membership-query déterministe n'est exposé par le wrapper
`DocsClient`. **Disposition** : documenter au body + carry harness/K
(fix trivial si ça mord un jour : `terminate-after=4` n'y change rien —
c'est la borne in-test 8s qu'il faudrait élargir à ~45s pour le sens
duress, au prix du temps de suite ; à trancher en K). NB : l'ancre
initiale du finding (`:369-379`) était le deadline pré-existant #4 ;
l'évidence réelle est `:915-925`.

### P3 (8, documentables au body, aucun ne bloque)

**P3-1 — Doc-comment résiduel n'énumère que le sous-bras ticket-None**
(diff-correctness + research-grounding). `runtime.rs:2543-2547` documente
comme « unreachable » le seul sous-arm ticket-None, mais les bras
create/recreate/first-boot appellent aussi `doc.share_write()`
(`api/actor.rs:410` = `start_sync(vec![])` inconditionnel), contournant
le chokepoint duress-gaté. Inoffensif : `create_doc` forge un namespace
FRAIS (0 pair persisté → 0 dial, 0 contenu réel → 0 serve) ; le seul
vecteur sur le VRAI replica est ticket-None, correctement singularisé.
Complétude cosmétique de commentaire, pas un contresens (la phrase
`:2529` reconnaît déjà « the create arms only entered it via the fragile
share_write() side-effect »). Précision à ajouter au body.

**P3-2 — Commentaire « conservative under load » du deadline négatif 8s
imprécis** (diff-correctness + harness-quality ; facette de P2-B). Pour
une assertion NÉGATIVE, un bound COURT est la direction dangereuse, pas
conservatrice. La sûreté réelle du CONTROL vient du rejet catégorique
`AbortReason::NotFound` (`state.rs:90-99`, vérifié vendored 0.101), pas
du timing — le test est sain. Reformuler le commentaire au body plutôt
que corriger du code.

**P3-3 — Marge terminate-after 180s serrée face au cumul des caps
internes** (harness-quality). Pire cas GREEN = `await_neighbor(60s)` +
baseline `await_exact_key(30s)` + boucle re-dial ~50s ≈ 140s + 3 boots
QUIC + shutdowns → ~150-170s ; budget nextest = 60s×3 = 180s → headroom
~15-40s. Empirie solide (2×2037 verts + Docker), false-RED faible proba
(requiert 3 awaits séquentiels frôlant leur max simultanément sur un
test qui converge correctement). Défensif ; noter le calcul de budget au
body. Fix trivial si ça mord : `terminate-after=4`.

**P3-4 — Bras recreate mute la ligne M8 réelle sous duress**
(security-duress ; pré-existant A2, hors gate). Sous Duress + replica
NotFound, le bras recreate (`runtime.rs:2611` create_doc → `:2615`
set_storage_namespace ; feed `:2730/:2734`) écrase la ligne M8 réelle
avec l'id du namespace leurre-frais. Atteignable UNIQUEMENT si M8=Some
ET open_doc=None (store reset / DB transportée) — sous duress le
data-dir est le VRAI (`runtime.rs:344` indépendant de `identity_mode`),
donc normalement open_doc=Some et ce bras n'est pas atteint. Bénin pour
la FUITE (namespace frais = 0 pair, 0 contenu réel) mais = mutation
d'état persistant réel sous clé leurre. Pré-existant A2, PAS
introduit/aggravé par C (le gate ne couvre que le `start_sync` explicite
du chokepoint, pas les `share_write` des bras create). Corner
corruption-only. Route note THREAT_MODEL §15.1 / Phase G.

**P3-5 — Branche Skip journalise littéralement le mot « duress »**
(security-duress). Les 3 branches Skip émettent
`debug!("duress identity: … skipped")` (`:2090` project, `:2648`
storage, `:2764` feed). Mitigation VÉRIFIÉE et si tout est sous-estimé
par le finding : le logger fichier JSON persistant (`daemon.log`) est
filtré à un niveau `info` HARDCODÉ (`logging.rs:118-120`, ne lit pas
`RUST_LOG`) → `debug!` n'atteint JAMAIS l'artefact forensique sur disque
sous aucune config ; le tell ne surface que sur stdout transitoire si
l'opérateur a explicitement lancé `RUST_LOG=debug`/`-vv`. Rompt la
convention de silence duress (le republish frère `:771-783` présente
`None` sans logger « duress » ; `/publish-blob` duress = 503 générique).
Propriété de déni plausible = network-observer-scoped, pas local-log —
hors périmètre DURESS-BOOT-LEAK. Cosmétique/cohérence de verbosité,
harmoniser au plus.

**P3-6 — Bras recreate/first-boot sous duress + sous-arm ticket-None non
exercés par test** (branch-coverage ; documentés inatteignables /
structurellement couverts). Le chokepoint duress-gaté (`:2636`) est un
point de fusion UNIQUE après le `let (doc, ticket_str) = match {…}` :
la combinaison duress×create est byte-identique au chemin
duress×ticket-Some-reopen qui EST testé (`BootFnDuress` traverse le bras
ticket-Some car boot #1 persiste `Some(ticket)`). Le sous-arm ticket-None
est prod-inatteignable (3 writes M8 persistent tous `Some`). Carry/doc.

**P3-7 — Delta réel +9 dépasse l'estimation préflight §7 « +6..8 » ;
body doit acter +9** (scope-cuts). Compté exactement : 6 `#[tokio::test]`
sibling (`run_sibling_reopen_scenario` est un helper, pas un test) +
2 DocTicket + 1 `duress_mode_skips_sync_set_entry` (l'unit noop_identity
anticipé §4.3, hors du tableau §7 et du commit-shape §12) = 9, cohérent
2028→2037. Le body doit indiquer **+9** (compter l'unit), pas le
placeholder « +6..8 ». Précision de body, 0 code.

**P3-8 — Pattern nextest test-group `two-node-convergence` candidat §P7x,
à router wrap-up K** (patterns-docs). Le pattern harness (test-group
max-threads=2 + slow-timeout élargi pour tests 2-nœuds CPU-bound) n'est
pas capturé dans PATTERNS.md (seul P2-A-1 multi_thread existe,
orthogonal). Per cadence §6.12/§P70 (GUIDE/PATTERNS en clôture, pas
per-phase) → entrée §P7x en K. Sans elle, un futur auteur de test
2-nœuds re-découvre le flake ou nomme son test hors des 4 tokens du
filtre → non-cappé → flaky silencieux. Cohérent note mémoire « §P73
candidate → K ».

### REFUTED (écarté avec trace)

**Nextest slow-timeout élargi « peut masquer une régression future de
perf de convergence »** (scope-cuts P3, adversarial **REFUTED**). Le
mécanisme est mal attribué. L'override nextest slow-timeout est 60s
warn / 180s kill (`.config/nextest.toml:44`), strictement PLUS GRAND que
et en aval du deadline fonctionnel in-test 45s (`dispatch_loop.rs:920`) :
une régression de perf poussant la convergence au-delà de 45s fait
échouer le `loop` in-test avec `converged=false` et le GREEN assert
échoue VITE à ~45s, bien avant que le kill 180s de nextest ne se
déclenche — le timeout élargi ne masque rien que la borne in-test ne
gate déjà. Le slow-timeout nextest ne compte que pour un vrai HANG (que
les deadlines in-test préviennent déjà) : c'est du pur slack harness. Le
résidu de vérité (une régression perf < 45s est silencieuse) est une
propriété de la borne in-test délibérément généreuse, inchangée par ce
config. La config est fortement auto-documentée (`nextest.toml:17-26` +
`:36-44`), rien n'est coupé silencieusement, et la reco de tracking
duplique déjà préflight §10 (WS-3/PD-5 → K). Écarté.

## Suites §7.4 (résultats fournis, audités en cohérence)

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  **2037/2037 0-skip ×2 runs consécutifs** (2028→2037 = **+9**, cohérent
  avec 6 sibling + 2 DocTicket + 1 noop_identity — le body doit acter +9,
  cf. P3-7) ; doctests 6/6 ; release build vert 6m24s.
- **Docker sbfb-ci** : fmt 0 ; clippy 0 ; nextest 2035/2041, **6 fails =
  classe env Docker-on-Windows DOCUMENTÉE** (memory `feedback_dual_platform` /
  TEST-ISOLATION-SBFB-HOME : 4 `multi_daemon` + `cross_daemon_blob` +
  `blob_serve_coep`, « timeout waiting for auth_token » au spawn daemon ;
  verts Win natif + CI Linux réel) ; doctests 6/6. Arithmétique cohérente
  avec la baseline B (2030/2032 pré-fix, +cfg(unix)).
- **web** : lint 0 err (5 warnings préexistants) ; tsc 0 ; unit 411/411 ;
  coverage 87.27/79.01/86.02/88.59 ; build ; size 6/6 ; scan-en clean.
  0 fichier `web/` au diff — attesté contexte, non contredit.
- **operator** : lint ; unit 201/201 ; gates 6/6 ; size ; E2E Playwright
  10/10. 0 fichier `tools/` au diff — attesté contexte, non contredit.
- **Codex** : joué APRÈS les dispositions post-review (cf. « Codex
  reconciliation » en fin de fichier) — round 1 : 10/10 CONFIRMÉ, 0 GAP.

Le durcissement harness (test-group nextest + re-dial keepalive
prod-fidèle) est justifié par 3 flakys observés (GREEN feed, duress feed
×2, GREEN storage), tous verts solo ~2s ; après le re-dial : 2037/2037
×2 runs. La couverture du filtre nextest est EXACTE (11 tests 2-nœuds de
dispatch_loop, 0 match parasite).

## Invariants tenus

- **0 bump wire SBFB** : grep du diff entier = 0 hit
  `DOMAIN_*_V1` / `*_FORMAT_VERSION` / `ANNOUNCEMENT_VERSION` / JCS /
  ALPN. `start_sync` / entrée sync-set = état interne moteur iroh-docs.
  La string DocTicket est celle d'iroh-docs (compat-upgrade), pas un wire
  SBFB — stabilité prouvée empiriquement par le round-trip `docs.rs`.
- **iroh strictement seul (D7)** : `git diff **/Cargo.toml` = 0 ligne de
  dep ajoutée (Cargo.toml core = commentaire pkarr recalibré seul).
- **Pas de rétro-amendement Phase B** : les pins B (=1.0.1 / =0.101.0 /
  =0.103.0 + rust-version 1.91) sont intouchés ; seul le commentaire
  pkarr est recalibré (in-scope §5.1). Bisectabilité préservée (le fix
  fonctionnel appartient EXCLUSIVEMENT à C).
- **CONTROL A4 préservé** : `dispatch_loop.rs:557` corps byte-intact,
  seul le commentaire re-daté 0.101 ; tripwire re-vérifié non-convergent
  en B, non touché par C.
- **Day-0 D1..D8 intouchés** ; **pre-launch policy** respectée (aucun
  zombie legacy-decode ajouté, N=0 confirmé ; ticket round-trip sous le
  lock courant, pas de fixture 0.98 forgée).
- **Test-acteur docs-contract §6.12 : N-A-no-new-frontier CONFIRMÉ** — le
  fix ne crée aucune frontière lue par un acteur distinct :
  `SyncSetOutcome` / `sync_set_entry_in_duress` / boot fns `pub(crate)`
  sont internes au crate (lus par `runtime.rs` + le module test du même
  crate) ; la string DocTicket servie au front est byte-identique
  (`storage_api.rs` / `feed_sync.rs` intouchés). `check-frontier-contracts.sh`
  exit 0. Aucune étiquette `// FRONTIER:` requise — consigner
  N-A-no-new-frontier au body / clôture K.
- **P2-PROJECT-DOC-SELECTOR reste dette/K** (non élargi) :
  `list_docs().first()` non-déterministe affecte
  `open_project_doc_for_dispatch` mais PAS les siblings (sélection par
  `namespace_id` M8 déterministe). C ne touche pas la persistance du
  namespace id du project doc.

## Carries sortants

- **AU BODY (P2-A)** : documenter la lacune de test project-doc-duress
  (branche sécurité prod-exécutée sans tripwire d'intégration ;
  correction prouvée par identité structurelle + unit) OU ajouter le
  test `open_project_doc_for_dispatch_duress_skips_sync_set` avant commit.
- **AU BODY (P2-B)** : nommer l'asymétrie 8s/45s des tests négatifs
  duress (false-GREEN possible sous charge sur régression du câblage) +
  carry harness/K.
- **AU BODY (P3, précisions)** : delta réel **+9** (compter l'unit
  noop_identity, pas « +6..8 ») ; doc-comment résiduel create-arms ;
  reformulation commentaire 8s ; budget terminate-after 180s ; log
  littéral « duress » (harmonisation verbosité) ; N-A-no-new-frontier
  §6.12 explicite.
- **Phase G (THREAT_MODEL.md)** : (a) ligne §15.x « re-dial boot des
  pairs persistés » couvrant les 3 docs (≤5/doc, `PEERS_PER_DOC_CACHE_SIZE`) ;
  (b) ligne miroir DURESS-BOOT-LEAK pour la mitigation duress codée en C ;
  (c) mutation M8 recreate sous duress (P3-4) ; (d) delta DNS pkarr déjà
  routé carry B. Le CODE atterrit en C, la DOC en G (DoD (d) closure).
- **Phase F (NOTE, non-bloquante)** : worker re-parse ticket persisté au
  boot NON-FATAL (Err → warn + continue) — surveiller le diagnostic si
  l'acceptance F boote un worker sur un store/allowlist reporté d'un
  build 0.98.
- **Wrap-up K** : P3-8 pattern nextest `two-node-convergence` → §P7x ;
  P2-B carry harness (élargir la borne in-test négative duress à ~45s si
  jamais un false-GREEN est observé) ; WS-3/PD-5 hoisting (Voie 1 prise,
  reste dette/K) ; P2-PROJECT-DOC-SELECTOR (inchangé, hors C).
- **Veille continue** : re-check crates.io (1.0.2 ?) + RustSEC avant le
  push live.

## Codex reconciliation

**Dispositions post-review appliquées AVANT Codex** (directive PO
qualité — l'option « la plus propre » de chaque finding) :
- **P2-A FERMÉ PAR TEST** : `open_project_doc_for_dispatch_duress_skips_sync_set`
  ajouté (`dispatch_loop.rs:1031`, même shape que convergence #5 mais
  `IdentityMode::Duress` au restart : même doc id, keepalive rejeté,
  write post-restart non convergent 8s). La branche Skip du project doc
  a désormais son tripwire d'intégration. Delta tests passe à **+10**
  (2028→2038), ce qui régularise aussi P3-7.
- **P2-B DOCUMENTÉ + carry K** : commentaire du deadline négatif
  reformulé (`dispatch_loop.rs` — la sûreté du CONTROL = rejet
  catégorique `AbortReason::NotFound`, pas le timing ; résidu duress
  false-GREEN 8-45s nommé, attrapé sur tout run idle/solo) — couvre P3-2.
- **P3-1 appliqué** : doc-comment `boot_storage_namespace` complété
  (bras create = namespace FRAIS, bénin sous duress ; ticket-None =
  seul side-effect touchant le VRAI replica).
- **P3-5 appliqué** : les 3 `debug!("duress …")` supprimés — branches
  Skip silencieuses (miroir du republish feed), commentaire in-code.
- **P3-3/P3-4/P3-6/P3-8 routés** : body (budget 180s, branches
  inatteignables) + Phase G (mutation M8 recreate duress) + wrap-up K
  (pattern nextest §P7x).

**Suites re-jouées sur le diff FINAL** : Win fmt 0 / clippy 0 / nextest
**2038/2038 0-skip** / doctests 6/6 / release 6m48s ; Docker sbfb-ci
fmt 0 / clippy 0 / nextest 2036/2042 (6 fails = classe env
Docker-on-Windows documentée, inchangée) / doctests 6/6.

**Codex GPT 5.5 round 1** (`sprint81_phase_c_codex_review.md`, output
brut `codex exec -o`, non réécrit) : **10/10 livrables CONFIRMÉ, 0 GAP,
0 PARTIEL** — chokepoint + discriminateur A2 intacts (L1), gate duress
3 docs + call-sites réels + silence duress (L2), signatures/tests mis à
jour (L3), 6 sibling + project-duress + DocTicket + unit (L4-7),
test-group nextest (L8), recalibrations doc-only factuelle
`remote_info_iter` vérifiée contre la source locale iroh-1.0.1 (L9),
invariants transverses dont `git diff -G'DOMAIN_|_FORMAT_VERSION'` = 0
hunk et CONTROL A4 corps intact (L10). Critère d'arrêt « CLEAN » atteint
au round 1 — pas de boucle. Verdict promu **PASS** ; commit
`fix(daemon): Sprint 81 Phase C — …` (shape préflight §12, delta +10).
