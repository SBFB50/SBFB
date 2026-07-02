# Review — Sprint 80 Phase I : Testabilité T1/T2 + re-couverture SSE single-Done + comptabilité du delta (front/harness/CI/docs, `tools/factory-operator`)

**Date :** 2026-07-02
**Périmètre :** working tree NON COMMITÉ — **16 fichiers au moment de la review initiale (11 modifiés + 5 nouveaux), 0 fichier `.rs` modifié** (front/harness/CI/docs purs). Après les fixes post-review et post-Codex, le commit final porte **18 fichiers code/harness** (les 11 + `e2e/boot.spec.ts` [P3-1] + `e2e/verify.spec.ts` [commentaire stale Codex] + 5 nouveaux) plus les artefacts process (preflight, review, codex_review). Comptabilité corrigée suite au PARTIEL livrable-12 de Codex.
Modifiés (`git diff`) : `.github/workflows/ci.yml` ; `.woodpecker/ci-linux.yml` ; `docs/rust/PATTERNS.md` (§P72 neuf) ; `tools/factory-operator/e2e/fixtures.ts` ; `e2e/serve-operator.mjs` (réécrit) ; `e2e/steer.spec.ts` (+2 sous-tests 3a/3b, sous-test 2 passé provider `local`) ; `package.json` (script `t2`, retrait `--passWithNoTests`) ; `playwright.config.ts` (webServer TABLEAU + env fixture) ; `scripts/scan-front-discipline.sh` (axe 2 anti-score + self-test anti-vacuous) ; `src/lib/streamChunk.ts` (commentaire) ; `src/lib/useTokenStream.test.ts` (+1 test 0-reconnect).
Nouveaux (untracked, lus en ENTIER) : `.planning/active/sprint80_phase_i_preflight.md` (contrat — verdict PLAN-ADAPT, 9 adaptations §6, addendum §10) ; `.planning/active/sprint80_t2_acceptance.json` (artefact T2 committé) ; `tools/factory-operator/e2e/fixture-workspace.mjs` ; `e2e/serve-fixture-daemon.mjs` ; `scripts/t2-acceptance.mjs`.
**Orchestration :** review deep pré-Codex — 5 dimensions (correctness ligne-à-ligne, livrables-vs-contrat, sécurité DEEP, sémantique des tests, patterns/docs/CI/langue) → vérification adversariale par-finding (CONFIRMED / REFUTED) → synthèse réconciliée sur le DIFF réel + grounding wire Rust (process.rs:56-67 `repo_root`, sprint_history.rs `git_cmd` cwd-based, provider_router.rs arm Ollama/Network, operator_server.rs `sse_gate`).

---

## Conclusion de la review initiale (pré-Codex) : PASS-PENDING

Le diff livre l'intégralité du contrat Phase I (8 items §4.2, 9 adaptations §6, scope cuts §7 respectés). Les invariants cardinaux tiennent tous : **0 `.rs` modifié** (donc 0 route daemon, Factory hors daemon), **0 nouvelle dep runtime** (`package.json` ne change que des scripts, les `.mjs` n'utilisent que des built-ins node), diff = vérité Rust, MUR jamais bouton, 0 verdict PASS calculé/écrit par l'UI, artefact T2 sans secret/timestamp/chemin (RIG-ABSENT impossible par construction). L'hermétisme est réel (config git 100 % locale, `core.hooksPath` neutralisé, `git rev-parse --show-toplevel` résout la fixture scellée, `SBFB_HOME` mkdtemp per-run) et le vrai repo ne fuite dans aucune assertion.

**Aucun P0, aucun P1.** Conformément au critère : PASS-PENDING (review OK, Codex pas encore passé — **jamais un verdict committable**). Deux P2 CONFIRMED de **sur-affirmation de test** (false confidence) sont recommandés en fix bon marché AVANT le gate Codex — ils ne bloquent pas (l'invariant single-Done du livrable reste couvert par les oracles exactly-one et le unit test d'accumulation), mais ils touchent la crédibilité de la phase de testabilité elle-même. Le flake de charge web/ non-lié doit re-verdir avant commit (condition mécanique, ci-dessous).

**Décompte findings (après filtrage adversarial) :** P0 = 0 · P1 = 0 · P2 = 2 (CONFIRMED) · P3 = 5 (CONFIRMED) · info = 9. **0 finding réfuté** (toutes les dimensions ont confirmé leurs findings à leur palier — aucune sur-cotation à neutraliser). Aucun viol cardinal, aucune faille, aucune régression cachée, aucun bump wire/Day-0, aucune nouvelle surface réseau prod.

---

## Résumé des 5 dimensions

| # | Dimension | Résultat |
|---|---|---|
| 1 | Correctness ligne-à-ligne (constantes fixture, routage vrai dispatch, /log, MUR, scan, t2) | OK — ports 3111/3112 et `FIXTURE_*` coïncident exactement entre `fixtures.ts`/`playwright.config.ts`/`serve-fixture-daemon.mjs` ; round-trip deltas via séparateur `|` préserve l'espace fin (`'Bonjour '`) ; `from_provider('local')→Ollama`, `('network')→Network` vérifiés ; /log push AVANT flush (0 race) ; MUR sans collision prompt bénin. **2 notes robustesse P3 + 2 info** |
| 2 | Livrables vs contrat (préflight §4.2/§6/§7 + plan) | OK — 8 items §4.2 tous présents, 9 adaptations §6 honorées, scope cuts §7 tenus (useTokenStream non ré-écrit sauf +1, sous-tests 1/2/4/5 non ré-écrits, 0 stub JS, Aperçu/Preuve non touchés). Comptabilité delta exacte. **3 info (formes de livrable adaptées, tracées PLAN-ADAPT)** |
| 3 | Sécurité DEEP (7 axes : bind loopback, artefact, MUR, hermétisme, env SBFB_*, CI, THREAT_MODEL) | OK — fixture daemon bind `127.0.0.1` only, corps drainés+jetés (0 reflection), réponses constantes ; artefact T2 = enum pur (0 secret/env/stdout/chemin) ; MUR non contourné (sous-test 4 `mur_gated_no_spawn`, streamOpened=false) ; `reuseExistingServer:false` → échec bruyant si port occupé (0 redirection silencieuse) ; **0 nouvelle surface prod → THREAT_MODEL inchangé. 2 info hygiène** |
| 4 | Sémantique des tests (PO-14 single-Done + oracles exactly-one) | CONCERN-local — cœur PO-14 correctement testé full-stack (double oracle log+/__calls, session fraîche par test, workers:1), MAIS **2 sur-affirmations (P2) : 3a ne prouve PAS le stream des deltas (satisfait par le Done seul) ; self-test anti-vacuous du scan a un angle mort sur l'axe .po-score NEUF**. **+2 P3 (titre 3b, t2 skipped→PASS) + 1 info** |
| 5 | Patterns / Docs / CI / Langue (§P72, streamChunk anchor-by-name, renumber CI, EN/FR) | OK — §P72 factuellement exact (repo_root cwd-based, cargo config discovery cwd-based, HEAD~50 fallback, find_entry_tip) ; fix `streamChunk.ts` `:1063→sse_gate` ancre-par-nom (existe operator_server.rs:1591) ; renumber CI [3]..[9] cohérent, miroir Woodpecker identique. **1 P3 (stale STALE-PHASE-K) + 1 info (label CI)** |

---

## Conformité au préflight (PLAN-ADAPT) — les 9 adaptations §6 tenues

| Adaptation figée | État | Évidence |
|---|---|---|
| SSE déterministe = fixture sur le VRAI dispatch (SBFB_DAEMON_ENDPOINT/SBFB_OLLAMA_ENDPOINT), 0 code prod Rust | CONFORME | 0 fichier `.rs` modifié ; `serve-fixture-daemon.mjs:1-22` documente le choix ; routage `from_provider('local')→Ollama`, `('network')→Network` traversé réellement |
| Backend Rust du plan (« cible ExecutionTarget echo ») résolu à 0 code prod | CONFORME (adaptation supérieure) | Le plan §Backend listait un ajout Rust « ou stub HTTP/SSE — à figer au préflight » ; le préflight §3.1/§6.3 a tranché fixture-sur-vrai-dispatch → préserve « Factory hors daemon / 0 route daemon » (LC-I-1) |
| Hermétisme = cwd binaire → workspace git fixture + bundle copié + SBFB_HOME mkdtemp per-run | CONFORME | config git 100 % LOCALE (`git -C ws config` sans `--global`), `core.hooksPath`→`.no-hooks` inexistant (hooks off), `git rev-parse --show-toplevel` résout la fixture (elle a son propre `.git`) ; 2 root-causes fixées (cargo config discovery cwd-based → cargo build ancré repo puis spawn ; spawnSync `.cmd` win32 → `shell:true`) documentées §P72 |
| T2 = projection déterministe allowlist, 0 timestamp/durée/chemin/port/secret, RIG-ABSENT impossible, exit 1 sur BLOCK | CONFORME | `sprint80_t2_acceptance.json` = `{suite,status,diagnosis:null,gates{9 enum},scenarios{10 enum}}` ; `t2-acceptance.mjs:133-139` compose `diagnosis` uniquement de clés statiques + 2 chaînes fixes (jamais stdout/env/chemin) ; `path.relative` va à la console pas au fichier |
| Fixture anchor « Sprint 0 Phase A » (find_entry_tip → range valide sur jeune repo) | CONFORME | `find_entry_tip` grep `"Sprint {prev} Phase"` (sprint_history.rs) ancré par le seed ; **gap latent HEAD~50 tracé aux DEUX endroits** (`fixture-workspace.mjs:102-107` + PATTERNS §P72) = carry, non patché depuis le harness |
| Scan anti-score axe 2 + self-test anti-vacuous | CONFORME (angle mort P2, cf. TS-2) | `scan-front-discipline.sh` axe 2 `.po` + self-test injection+restore ; passe vert Git-Bash ; **le self-test unionne .tsx+.po → non-prouvé indépendamment (P2)** |
| Gating CI Vitest (GHA step [3] + Woodpecker `factory-operator-vitest`) + retrait `--passWithNoTests` | CONFORME | `package.json:11` retire `--passWithNoTests` (échoue si 0 test) ; GHA [3] + Woodpecker miroir vitest-avant-gates ; renumber [3]..[9] cohérent |
| Comptabilité delta honnête (plancher 200/35/8 → 201/35/10) | CONFORME | +1 Vitest, +2 e2e, 0 fichier de test neuf (→ 35 inchangé) ; « −7/−8 » du plan périmé corrigé au préflight |
| Test « fetch-once-après-Done » (+1 Vitest 0-reconnect) | CONFORME | `useTokenStream.test.ts:152-162` (real timers + flush microtasks → `toHaveBeenCalledTimes(1)`) ; borné au design timerless, documenté honnêtement |

**Items T1-étendu §6.2 « VÉRIFIER (pas ré-écrire) » substantiés :** hash context-pack asserté (`ContextPackInspector.test.tsx:62 'abc12345'`) ; refus MUR + raison présents (`Mur.tsx:49`) + barrière + invariant no-bypass assertés (e2e sous-test 4 + `Mur.test.tsx`).

---

## Invariants cardinaux — tenus de bout en bout

- **0 `.rs` modifié** → 0 route daemon, Factory hors daemon, diff = vérité Rust (le harness route à travers le vrai dispatch, ne le remplace pas). Le bloc Rust 2014/2014 est INCHANGÉ vs baseline — cohérent.
- **0 nouvelle dep runtime** : `package.json` (VÉRIFIÉ) ne modifie que des scripts (`t2`, retrait `--passWithNoTests`) ; les 3 `.mjs` neufs n'importent que des built-ins node (`http`, `fs`, `child_process`, `path`, `os`). 0 `npm i`.
- **0 verdict PASS calculé/écrit par l'UI** : la phase ne touche aucun rendu de verdict ; l'artefact T2 est une projection allowlist (jamais un calcul de verdict UI). Cardinal préservé.
- **MUR jamais bouton** : couvert POSITIVEMENT par sous-test (4) (`mur_gated_no_spawn`, streamOpened=false) ; les prompts bénins de 3a/3b évitent les substrings SENSITIVE_ACTIONS (`shell/commit/push/pass`) et traversent le gate MUR (placé AVANT dispatch) sans le déclencher.
- **Total tests jamais en baisse silencieuse** : filet mécanique (CI Vitest à chaque push + retrait `--passWithNoTests`) couvre le cas catastrophique (chute à zéro) ; la baisse partielle reste couverte par la revue humaine du delta (convention README §4) — adéquat au contrat (LC-I-3).
- **Langue** : code/identifiants/commentaires/console EN ; FR uniquement dans les inputs UI de test légitimes + contenu planning fixture. Discipline tenue.

---

## Findings (P0:0 · P1:0 · P2:2 · P3:5 · info:9)

### P2 — sur-affirmation de test (recommandés AVANT Codex, bon marché, NON bloquants)

| # | Localisation | Description | Correctif |
|---|---|---|---|
| **P2-1 (TS-1)** | `steer.spec.ts:78` + `Atelier.tsx:47` + `provider_router.rs` arm Ollama | **3a ne prouve PAS que les deltas ont streamé.** `toContainText(FIXTURE_OLLAMA_TEXT)` est asserté au statut `done` (turn-status `:79` « terminé ») ; or `Atelier.tsx:47` rend `turn.result` au `done`, pas l'accumulé `turn.text`, et l'arm Ollama porte `result = std::mem::take(&mut accumulated)` (== jointure des deltas fixture). Donc `FIXTURE_OLLAMA_TEXT` apparaît via le payload du Done SEUL : une régression qui DROP tous les Delta mais conserve le Done result passerait 3a. Titre « streams tokens » + commentaire `:75-76` sur-affirment. **Backstoppé par `useTokenStream.test.ts:46` (accumulate 'Hel'+'lo'→'Hello', valeurs distinctes) → l'invariant single-Done tient ; c'est la FAUSSE CONFIANCE e2e qui est le défaut.** CONFIRMED. | Le fix « Done result ≠ jointure des deltas » n'est PAS atteignable pour l'arm Ollama (Rust dérive `result` = deltas accumulés) ; retenir l'alternative valide : **asserter sur `turn.text` PENDANT le streaming** (avant done) pour prouver que le texte vient du chemin Delta, pas du Done. À défaut : aligner titre/commentaire sur ce que le test garde réellement. |
| **P2-2 (TS-2)** | `scan-front-discipline.sh:103,105` + `scan_scores` (`:72-85`) | **Angle mort du self-test anti-vacuous sur l'axe .po-score NEUF de la phase.** `scan_scores` unionne en UNE sortie le grep `.tsx/.ts` ET le grep `^msgstr.*(...)` `.po` ; le self-test plante un probe `.tsx` (« trust-score 87 % ») ET un probe `.po` (« score de santé : 87 % ») mais n'exige que `s` non-vide. Si le grep `.po` casse silencieusement (ancre/regex), la ligne `.tsx` garde `s` non-vide → self-test PASS. L'axe `.po`-score N'EST PAS prouvé indépendamment, alors que l'en-tête affirme « A regex regression can never turn this gate silently green » et que `.po` est PRÉCISÉMENT la défense-en-profondeur AJOUTÉE en Phase I. Scan réel fonctionnel à HEAD → P2, pas P1. CONFIRMED. | Dédoubler le self-test en sondes par-surface (comme verdict/score sont déjà séparés) : `s_tsx` ET `s_po` toutes deux exigées non-vides (deux dirs ou deux variables). |

### P3 — documentés (corrigeables, un candidat à balayer dans CE commit)

- **P3-1 (DOC-1)** — `boot.spec.ts:10-11` : **forward-promise STALE-PHASE-K littéralement FAUSSE** — « The remaining sub-tests (composeur→session, SSE single-Done, MUR, diff-viewer) land in Phases C / H / I. » Or les 4 ont TOUS atterri (composeur C, MUR C, diff-viewer H, SSE single-Done **I = cette phase**). Le diff a nettoyé les jumeaux byte-quasi-identiques (`steer.spec.ts:20`, `playwright.config.ts:165-166`) MAIS a laissé `boot.spec.ts` intact. `check-frontier-contracts.sh:77` scanne `crates web/src` seulement — jamais `tools/factory-operator` → échappe au gate (gate vert malgré la fausse promesse). VÉRIFIÉ à la lecture. **Anti-pattern nommé et policé par le projet ; à balayer À LA MAIN dans le commit qui a nettoyé ses jumeaux.** CONFIRMED.
- **P3-2 (TS-3)** — `steer.spec.ts:99` : titre « zero deltas » de 3b JAMAIS asserté. Le corps (`:116-137`) n'assert que `toContainText(FIXTURE_NETWORK_RESULT)`, turn-status, exactly-one assistant, `after.submit-before.submit===1` ; un Delta parasite sur l'arm Network ne changerait ni le compte d'assistants (1 Done → 1) ni le texte rendu (`Atelier.tsx:47` montre `result`) → non détecté. Propriété néanmoins exercée par construction (arm Network émet 0 delta). Aligner : asserter l'absence de Delta OU retirer « zero deltas » du titre. CONFIRMED.
- **P3-3 (TS-4)** — `t2-acceptance.mjs:115` : `spec.ok ? 'PASS' : 'BLOCK'`. Dans le reporter JSON Playwright `spec.ok` est true pour un spec SKIPPED (skip ≠ échec) → un futur `test.skip` se projetterait PASS ; et le garde de vacuité `:122` ne couvre que la vacuité TOTALE (`length===0`) — un des 10 ids `TITLE_TO_ID` retiré/sauté disparaît sans BLOCK (l'artefact resterait PASS avec moins de scénarios). `retries:0` écarte le flaky-masking, 10/10 présents aujourd'hui. Pour un artefact d'acceptance « honnête » (but même de T2) : asserter que l'ensemble d'ids attendu est intégralement présent + traiter skipped comme BLOCK. CONFIRMED.
- **P3-4 (correctness P3-1)** — `scan-front-discipline.sh:83` : l'axe anti-score `.po` (`^msgstr.*(...)`) ne matche QUE la 1re ligne `msgstr` → un score glissé dans une continuation msgstr multi-ligne (démarrant par `"`) échappe. Défense-en-profondeur (axe primaire src/code solide) ; 0 violation réelle aujourd'hui ; exploitation = traducteur hostile futur → dette de couverture, pas régression. **Nuance :** le finding affirmait `check-i18n-verdict-cross-locale.sh:695` « also first-line-only » — FAUX (ce script accumule bien les continuations, parser PO complet) ; l'argument de confort « cohérent avec le gate existant » est donc mal-fondé, mais l'évaluation d'impact (défense-en-profondeur, future-only) tient indépendamment → sévérité P3 inchangée. CONFIRMED.
- **P3-5 (correctness P3-2)** — `steer.spec.ts:40` + `serve-fixture-daemon.mjs:94` : compteur `/__calls` cumulatif partagé + tour `local` fire-and-forget du sous-test (2) → le déterminisme du delta `after-before===1` de 3a repose sur un ordonnancement WALL-CLOCK implicite (le POST loopback de (2), émis à l'ouverture du stream en <1ms, doit atterrir AVANT le snapshot `before` de 3a, séparé par le bootstrap page ~s du beforeEach), pas sur une barrière dure/reset. Empiriquement vert (10/10) ; garde additionnelle sous-claimée : à la fermeture de page de (2), Axum drop le futur sur déconnexion client → (2) contribue 0 call. Latent-fragilité si un refactor futur supprime le bootstrap inter-tests. CONFIRMED.

### info — hygiène / traçabilité (aucune action requise)

- **INFO (livrables LC-I-1)** — Backend Rust du plan résolu à 0 code prod (fixture-sur-vrai-dispatch) : bon appel, préserve « Factory hors daemon », évite le footgun `from_provider` inconnu→Claude ; tracé PLAN-ADAPT, à ne pas lire comme backend-dep manquant.
- **INFO (livrables LC-I-2)** — Artefact T2 = snapshot ponctuel non régénéré/comparé en CI ; conforme au contrat (kickoff §T2 + prior-art WPT/Deno) ; chaque composant gaté indépendamment en CI → une vraie régression rougit quand même la CI, seul un relecteur humain d'un JSON périmé serait induit en erreur.
- **INFO (livrables LC-I-3)** — « Interdire la chute silencieuse du total » = filet qualitatif (suite exécutée + anti-zéro-test), pas un plancher numérique mécanique ; conforme (README §4 : plancher enforced au review du commit body).
- **INFO (sécurité 1)** — `serve-fixture-daemon.mjs:113` catch-all renvoie HTTP 200 pour chemins inattendus ; 0 impact (loopback-only, corps fixe non-réfléchi) ; fail-fast 4xx durcirait l'hygiène de test.
- **INFO (sécurité 2)** — `serve-operator.mjs:226` forwarde tout `process.env` au binaire Operator ; standard harness ; SBFB_* forcés par `playwright.config.ts` (précédence), artefact T2 ne capture jamais l'env ; allowlisting explicite serait plus hermétique, non requis (pré-launch, boucle locale).
- **INFO (correctness INFO-1 / tests TS-5)** — Test 0-reconnect (`useTokenStream.test.ts:162`) flush microtasks seulement → ne couvre pas un reconnect programmé par timer ; adéquat au hook actuel (transport fetch+reader, 0 setTimeout), documenté honnêtement.
- **INFO (correctness INFO-2)** — §P72 + commentaires portent des numéros de ligne Rust qui dériveront (`process.rs:56-67`, `operator_server.rs:232-236`, `provider_router.rs:282,155`) ; le fix `streamChunk.ts` (`:1063`→ancre-par-nom `sse_gate`) recrée justement la dette de drift ailleurs — cosmétique.
- **INFO (patterns CI-1)** — `ci.yml:178` label du step [4] énumère 5 des 6 gates que `npm run gates` chaîne (omet `accessibility-system`) ; pré-existant, ligne éditée cette phase (moment de compléter) ; le label sous-estime la couverture, trivialement corrigeable en appendant le 6e gate.

### Findings réfutés / neutralisés

**Aucun.** Les 5 dimensions ont confirmé chacun de leurs findings à leur palier annoncé après vérification adversariale ; aucune sur-cotation à rétrograder, aucun faux-positif à tracer. Les seuls ajustements sont deux nuances internes (P3-4 : analogie i18n-verdict mal-fondée mais impact inchangé ; P3-5 : garde Axum sous-claimée) qui ne modifient ni le verdict ni la sévérité.

---

## État des suites (§7.4) — tel que fourni par le main-thread (NON relancé)

- **Bloc Rust COMPLET vert** : `fmt --check` + `clippy -D warnings` + nextest workspace **2014/2014** (INCHANGÉ vs baseline — cohérent, 0 `.rs` modifié) + doctests + build release.
- **Front `factory-operator`** : **T2 harness PASS** (artefact : **9 gates PASS** [6 discipline + build + size-limit + vitest] + **10 scénarios e2e PASS**) ; **Vitest 201/201** (35 fichiers) ; lint + tsc verts ; e2e **10/10**.
- **Bloc web/** : baseline matin **411/411 vert** ; **re-run solo EN COURS** après 1 timeout de charge (`GpuConsentDialog` 5018ms>5000ms, suite NON touchée par la phase, variance connue) — **était vert au baseline**. → **Condition de commit (mécanique, non-liée à la phase) : web/ doit re-verdir avant le commit** ; ce flake de charge n'est pas bloquant pour la review mais l'arbre doit être propre-vert au wrap-up.

---

## Comptabilité du delta — exacte et honnête

**Plancher 200 Vitest / 35 fichiers / 8 e2e → 201 / 35 / 10.**
- **+1 Vitest** (`useTokenStream.test.ts` test 0-reconnect) ; 0 fichier de test neuf → **35 fichiers inchangé** (cohérent).
- **+2 e2e** (`steer.spec.ts` sous-tests 3a + 3b) → **8 → 10**.
- Le « −7/−8 » du plan était **périmé** (corrigé au préflight §6, addendum §10). Delta annoncé = delta réel.

---

## Actions avant commit

1. **Recommandés (bon marché, sur-affirmation de test) AVANT Codex :** corriger **P2-1 (TS-1)** (asserter `turn.text` mid-stream OU aligner titre/commentaire de 3a) et **P2-2 (TS-2)** (dédoubler le self-test en `s_tsx`/`s_po` séparés). Ce sont des défauts de false-confidence DANS la phase de testabilité elle-même — proportionnés à corriger, non bloquants pour le verdict. Re-lancer Vitest + e2e après fix.
2. **Balayer P3-1 (DOC-1)** dans CE commit : supprimer/corriger la forward-promise stale de `boot.spec.ts:10-11` (ses jumeaux ont été nettoyés dans le même diff ; le gate `check-frontier-contracts.sh` ne couvre pas `tools/factory-operator`).
3. **Condition mécanique :** faire **re-verdir web/** (flake de charge `GpuConsentDialog`, non-lié) — arbre propre-vert obligatoire au wrap-up.
4. **BLOQUANT — Gate Codex** (`codex exec`, gpt-5.5, raw dans `sprint80_phase_i_codex_review.md`) : boucler jusqu'à CLEAN ou P2/P3 documentés, puis promouvoir review→PASS.
5. **Discipline commit** : 1 commit `feat(factory-operator): Sprint 80 Phase I — <titre>` (ou `test(...)`/`ci(...)` selon dominante), body 9 sections, delta tests cumulé (+1 Vitest, +2 e2e → 201/35/10), scope cuts respectés (P3-3/P3-4/P3-5 + carry HEAD~50 → S81 documentés).
6. **Vérifications lourdes** : Vitest + tsc/build + lint + gates discipline (self-test scan) + size-limit + T1 e2e 10/10 BLOQUANT-vert + T2 artefact PASS au wrap-up (gate de testabilité par-sprint) ; **dual-platform Docker AVANT push**.

---

## Fixes post-review (re-review focalisée)

**Date :** 2026-07-02 · **Périmètre :** re-review UNIQUEMENT des 4 correctifs appliqués (P2-1/P3-2, P2-2, P3-1, P3-3) + non-régression. 0 `.rs` modifié, 0 dep runtime (les 4 fixes sont ts/sh/mjs/commentaire). Grounding wire relu dans le code courant : `operator_server.rs` (id de session, `/stream`, `Sse::new`), `provider_router.rs` (arm Network + frame `network-poll`), `llm_bridge.rs` (serde `StreamChunk`).

### Les 4 fixes tels qu'implémentés

- **P2-1 + P3-2 (oracle wire).** Note d'honnêteté : le 1er essai d'oracle sur le transcript brut via CDP (`response.text()`) a ÉCHOUÉ EN LIVE — `Protocol error « No data found »` sur un SSE déjà consommé par la page + stall du tour UI — d'où une **réécriture**. L'oracle est désormais le helper `wireTranscript()` (`steer.spec.ts:63-82`) : **session API DÉDIÉE** via request-context (POST `/api/chat/session` → POST `send` → **UN SEUL** GET `/stream` bufferisé jusqu'à EOF, `steer.spec.ts:79`), frames comptées sur le transcript SSE brut — **3a** : `"type":"delta"`×2 + `"type":"done"`×1 (`:93-94`) ; **3b** : `"type":"delta"`×0 + `"type":"done"`×1 + `"label":"network-poll"` présent (`:148-150`). Les snapshots `/__calls` sont pris **APRÈS** la session wire (`:98,154`) ; le tour UI ajoute exactement +1 (delta-based).
- **P2-2.** `scan-front-discipline.sh:91-113` — self-test à **sondes séparées** `dir_tsx`/`dir_po` ; `s_po=scan_scores(dir_po)` (`:107`) n'exerce QUE le grep `.po` (aucun `.ts/.tsx` dans `dir_po`) et le garde `|| [ -z "$s_po" ]` (`:109`) rougit dur si l'axe `.po`-score casse silencieusement → axe `.po` prouvé **indépendamment**.
- **P3-1.** `boot.spec.ts:10-11` — forward-promise périmée remplacée par une référence au présent vraie (sous-tests 2/3a/3b/4 en `steer.spec.ts`, 5 en `verify.spec.ts`). Commentaire pur, 0 changement fonctionnel.
- **P3-3.** `t2-acceptance.mjs:116-121` — **projection stricte** `spec.ok && statuses.length>0 && statuses.every(s==='expected')` : skipped/flaky/unexpected/tests-vides → BLOCK (`spec.ok` seul lisait un skip comme couvert). Cohérent avec `retries:0` (`playwright.config.ts:26`) ; titre renommé/non-mappé → **slug déterministe** (`:121`), jamais perte silencieuse.

### Résultat de la re-review — les 4 fixes FERMENT leurs findings, sans régression

Vérifications adversariales toutes CONFIRMÉES dans le code courant :

- **(a) Tour d'inférence wire supplémentaire par arm — 0 pollution des autres compteurs.** Grep : `/__calls` n'est référencé QUE par `steer.spec.ts` + `serve-fixture-daemon.mjs` (aucun autre spec n'assère un compteur absolu de fixture). Les assertions sont delta-based avec `before` snapshotté APRÈS la session wire → le tour wire est absorbé, seul le +1 du tour UI est mesuré.
- **(b) Ids de session distincts + oracle `/log` sur le BON tour.** L'id est `chat-{Rfc3339}` (`operator_server.rs:1359`) → session wire et session UI ont des ids DISTINCTS (séparés par le bootstrap page ~s) ; même en collision théorique, `HashMap::insert` (`:1400`) écrase → session UI toujours fraîche. L'oracle `/log` interroge l'id de la session **UI** (`steer.spec.ts:123,175`) → compte 1 seul assistant (le +1 de la session wire est sur une AUTRE session).
- **(c) `/stream` re-lance l'inférence à chaque appel ; wireTranscript ne l'appelle qu'UNE fois.** `handle_chat_stream` n'append un assistant que sur Done (`operator_server.rs:1692-1697`) ; `wireTranscript` ouvre le stream exactement 1× (`steer.spec.ts:79`).
- **(d) GET request-context bufferise jusqu'à EOF sans timeout problématique.** SSE sans keep-alive (`Sse::new` seul, `operator_server.rs:1704`) → EOF après Done ; poll interval fixture court → terminaison rapide, `.text()` sous le timeout par défaut.
- **(e) Assertion `network-poll` de 3b sur le BON transcript malgré `pollsByTask` monotone.** Le frame `network-poll` est émis à CHAQUE itération de poll AVANT le match de statut (`provider_router.rs:440-445`) et tous les yields pré-boucle sont `Error` (`:333-381`) ; le tour wire de 3b est le 1er tour network de la suite → son 1er poll voit `dispatched` (`serve-fixture-daemon.mjs:87`, `seen<=1`). Le tour UI qui suit voit `completed` au 1er poll, MAIS son assertion ne porte pas sur `network-poll` (uniquement sur le transcript **wire**, `:150`) → correct.
- **(f) Serde aligné.** `StreamChunk` `#[serde(tag="type")]` + renames `delta`/`done`/`debug` (`llm_bridge.rs:42-58`) ⇔ assertions `"type":"delta"` / `"type":"done"` / `"label":"network-poll"`.
- **(g) Self-test `.po` isolé réellement vert.** `scan_scores(dir_po)` matche `^msgstr.*(score de santé)` sur le probe `score de santé : 87 %` (grep `-i`) ; la ligne survit aux filtres `:\s*\*` / `:\s*//` → `s_po` non-vide.
- **(h) Mapping strict t2.** `retries:0` → un test n'est que `expected`/`unexpected` (pas de `flaky` naturel) mais la projection le BLOCK quand même (défense en profondeur) ; un titre renommé dégrade en slug (visible au diff) plutôt que perte silencieuse.

**Aucun P0/P1/P2 émergent.** Findings résiduels **info uniquement**, tous non bloquants et documentés :
- **INFO-1** — `steer.spec.ts:93` code en dur `toHaveLength(2)` au lieu de dériver `FIXTURE_OLLAMA_DELTAS.length` : couplage de maintenabilité ; un changement de fixture ferait échouer BRUYAMMENT (pas de faux vert). Aucune action Phase I.
- **INFO-2** — commentaire fixture `serve-fixture-daemon.mjs:33-34` (« ≥1 dispatched keeps the network-poll frame observable ») sur-estime le rôle du 1er poll : le frame `network-poll` est émis à CHAQUE poll → l'assertion tient indépendamment de `pollsByTask`. Nuance de commentaire seule.
- **INFO-3** — `t2-acceptance.mjs:98` `slug()` tronque à 60 car. → collision théorique entre DEUX titres non-mappés partageant le préfixe 60 car. Inatteignable pour les 10 titres actuels (tous mappés).
- **INFO/RR-1** — résidu documenté de P3-3 : le défaut cité (skip→PASS) EST corrigé, mais la 2e recommandation « asserter l'ensemble d'ids attendu intégralement présent » n'est pas implémentée — un test entièrement SUPPRIMÉ ne laisse aucune clé dans `scenarios`, `failedScenarios` ne le voit pas, le garde de vacuité ne rougit que sur `length===0` → l'artefact resterait PASS avec N<10 scénarios. Atténué par : (i) l'artefact est committé (diff 10→N relu à la main), (ii) la comptabilité du delta de tests (README §4) est un gate séparé. **Candidat carry S81** pour une défense harness-level ; non bloquant.
- **INFO/RR-2** — l'assertion `network-poll` de 3b est un oracle « ≥1 poll a eu lieu », légèrement plus faible que « branche dispatched exercée » — mais la propriété tient (le 1er poll wire renvoie bien `dispatched`) et la robustesse au `pollsByTask` monotone partagé est un PLUS. Aucun défaut.

**Findings réfutés :** aucun. Les 2 lentilles (correctness-fixes, process-closure) ont confirmé chacune leurs observations au palier info ; aucune sur-cotation à neutraliser.

### État T2 / suites après fixes

- **T2 acceptance : PASS** — 9 gates PASS + 10 scénarios PASS (`sprint80_t2_acceptance.json`), ids canoniques.
- **Steer e2e : 4/4** en isolation (561-801 ms) ; suite T1 complète 10/10.
- **Non-régression** : 0 `.rs` touché (bloc Rust 2014/2014 inchangé), 0 dep, carries P3-4/P3-5 inchangés (P3-5 plutôt atténué par le snapshot déterministe post-wire), délta tests inchangé (201/35/10).

### Conclusion

**allP1Resolved = OUI · 0 régression · 0 nouveau défaut bloquant.** Les 4 fixes ferment leurs findings ; les seuls résidus sont info (dont un candidat carry S81 sur la défense harness-level de P3-3). La phase reste **Codex-ready** — le verdict PASS-PENDING est inchangé (le passage à PASS reste conditionné au gate Codex).

---

## Codex reconciliation

Gate croisé `codex exec` GPT 5.5 (output BRUT : `sprint80_phase_i_codex_review.md`,
non réécrit) : **12 livrables — 9 CONFIRMÉ / 0 GAP / 3 PARTIEL** + 1 défaut
transverse. Disposition, jugée sur sévérité (critère d'arrêt : CLEAN ou
P2/P3 documentés) :

- **Livrable-8 PARTIEL (P2) — FIXÉ post-Codex** : un spec e2e supprimé
  aurait projeté PASS silencieusement → `t2-acceptance.mjs` exige désormais
  la présence de TOUS les ids attendus (`TITLE_TO_ID.values()`), id absent
  → BLOCK. Recoupe RR-1 de la re-review focalisée. **T2 re-run après fix :
  PASS 9 gates / 10 scénarios** (artefact régénéré, byte-stable).
- **Livrable-12 PARTIEL (P3) — FIXÉ** : comptabilité du périmètre corrigée
  en tête de ce fichier (16 fichiers à la review initiale → 18 code/harness
  au commit, artefacts process en sus).
- **Livrable-7 PARTIEL (P3) — DOCUMENTÉ carry (PO-MULTILINE-SCAN)** : les
  continuations PO multi-lignes (`msgstr ""` puis `"…"`) échappent au grep
  1-ligne de l'axe score. Défense-en-profondeur : l'axe primaire
  `.tsx`/`.ts` + la 1re ligne `msgstr` sont couverts ; un parseur PO complet
  est disproportionné pour ce gate — carry S81 avec les autres résidus.
- **Transverse — FIXÉ** : commentaire périmé `verify.spec.ts:5-8`
  (« THIS repo » → workspace fixture scellé).

Aucune correction n'a touché un chemin exercé par les suites Rust/web ;
le harness T2 modifié a été re-exécuté (PASS). Suites finales : Rust
2014/2014 + doctests + release ; web 411/411 + coverage ≥ seuils ;
factory-operator Vitest 201/201, T1 e2e 10/10 hermétique, T2 PASS.

## Verdict: PASS
