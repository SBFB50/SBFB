# Sprint 73 Phase B Preflight

Date: 2026-06-04
HEAD: `5361fd8`
Verdict: **EXECUTE**

## Verdict: EXECUTE

Phase B (dette non-convertible, 7 items P2) est research-aligned sur les 5
scans. Aucun blocage S1b/S2/S3/S4 (donc pas de DESIGN-CONFLICT) ; aucun
APPROACH-NAIVE/LIB-EXISTS S1a (donc pas de PLAN-ADAPT). Le fix D6
`multi_thread` est confirme **root-cause cross-platform**, pas un masque, par
lecture du code (acteur iroh-docs spawn + boucle pump engine) et des sources
amont (tokio #2499/#7049, iroh-docs 0.99 CHANGELOG). Quatre notes
non-bloquantes (carry-over) sont consignees pour le sprint d'implementation
(elargissement du scope multi_thread, test `chat_stream_uses_opus_model` a
preserver, EventSource mock, serial_test = nouvelle dev-dep).

## Evidence Rules
- Claim policy : chaque claim cite un chemin repo, une sortie de commande, une
  URL/date, ou une hypothese explicite.
- Local sources read :
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint73_kickoff.md` (D1..D6, §4 D6 worker-pump)
  - `.planning/active/sprint73_plan.md` (§Phase B B.1-B.5, fail-fast §5)
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (tests :92, :146)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (pump :659-729 ; tests
    :1497, :1413/1479/1586/1725/1789/1847/1971)
  - `crates/nexus-core-rs/src/docs.rs` (:291 get_many_by_prefix → actor query)
  - `crates/nexus-core-rs/src/node.rs` (:326-345 Gossip/Docs/Router spawn)
  - `crates/sbfb-factory/src/provider_router.rs` (:260-468 network_stream,
    resolve_daemon :273, poll loop diagnostic-loss :403/:406/:384)
  - `crates/sbfb-factory/src/daemon_client.rs` (:30,:42 std::fs::read_to_string)
  - `crates/sbfb-factory/src/operator_server.rs` (:311 default_model, :648-654
    session create, :755-788 ChatSendRequest, :922-928 model default at dispatch)
  - `crates/sbfb-factory/tests/operator_server.rs` (:255-284
    chat_stream_uses_opus_model, :797 operator_sprint_history_endpoint, client
    timeouts :70/:80/:91)
  - `crates/sbfb-factory/tests/process_cli.rs` (:472-487 audit_commit zombie)
  - `tools/factory-operator/package.json` (pas de vitest), `src/lib/executionChat.ts`
  - `web/vitest.config.ts` + `web/package.json` (template Vitest v4 jsdom)
  - `docs/rust/PATTERNS.md` (§P54 :2838-2885, §P53/§P55)
  - memory `feedback_model_46.md` (regle modele Claude, scope)
- Commands run (sorties pertinentes citees en ligne) :
  - `git rev-parse --short HEAD` → `5361fd8`
  - `git log --oneline -- crates/nexus-shell-daemon/src/dispatch_loop.rs`
    (introduction E2E `2f9238d` S71 Phase A ; pas de reversion)
  - `grep name=iroh-docs/iroh/tokio/rusqlite/redb Cargo.lock` → iroh 0.98.2,
    iroh-docs 0.98.0, tokio 1.52.3, rusqlite 0.36.0, libsqlite3-sys 0.34.0,
    redb 2.6.3 ; **serial_test absent du lock** (nouvelle dev-dep si introduite)
  - `cargo nextest run -p nexus-shell-daemon -E 'test(dispatched_task_is_claimed_and_executed_by_worker_engine)'`
    → **PASS 3.4s** (isole, process-per-test)
  - `cargo nextest run -p nexus-worker-core -E 'test(engine_claims_and_executes_tasks_on_registered_doc)'`
    → **PASS 2.1s** (isole)
  - `grep TASK_FORMAT_VERSION dispatch_loop.rs runtime.rs` → reference fixture
    seulement (:70, :1085), aucune definition/bump

## Scope
- Plan source : `.planning/active/sprint73_plan.md` §Phase B (B.1-B.5), §5
  fail-fast rows 12-18.
- Target files (8 zones) :
  - `crates/nexus-shell-daemon/src/dispatch_loop.rs` (test :146 attribut)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (test miroir pump)
  - `docs/rust/PATTERNS.md` (§P54 statut P2-A-1)
  - `crates/sbfb-factory/tests/process_cli.rs` (:472-487 de-hardcode SHA)
  - `crates/sbfb-factory/tests/operator_server.rs` (serialisation / timeout)
  - `tools/factory-operator/` (infra Vitest NEW + 1-3 tests)
  - `crates/sbfb-factory/src/provider_router.rs` (:383-407 last_err ; :273,321
    sync-fs) + `daemon_client.rs` (:30,42 sync-fs)
  - `crates/sbfb-factory/src/operator_server.rs` (:312,924-927 model picker) +
    `tools/factory-operator/src/lib/executionChat.ts` + `pages/ExecutionChat.tsx`
- Deps/APIs/specs :
  - **NEW dev-dep candidate** : `serial_test` 3.4.0 (P2-OPERATOR-TIMEOUT, option A)
  - **NEW dev-dep set** : `vitest` + `jsdom` + `@testing-library/*` cote
    `factory-operator` (P2-OPERATOR-NO-TEST-RUNNER) — miroir `web/`
  - `tokio` 1.52.3 (attribut `multi_thread, worker_threads=2`) — deja dispo
    (feature `full`), aucun bump
  - `tokio::fs` / `spawn_blocking` (P2-SYNC-FS-ASYNC) — deja dispo
- Security/protocol surfaces : aucune nouvelle. Le gate SENSITIVE_ACTIONS
  (operator_server.rs :896-910) reste **AVANT** le dispatch provider (:934) —
  le model-picker n'y touche pas. T0 loopback (X-SBFB-Token + Host) preserve.
- Tests expected (plan B.3) : 8 — worker-pump x2 (multi_thread, Windows+Linux),
  zombie fixture, operator serialise, factory-operator Vitest >=1, last_err
  surfacing, sync-fs revue, model-picker non-Claude.

## S1a OSS Prior Art
- Domain(s) : (1) tokio runtime flavor pour test pilotant un acteur async sur
  thread dedie ; (2) serialisation de tests d'integration liant des ports ;
  (3) mock EventSource/SSE en Vitest ; (4) diagnostic d'erreur sur boucle de
  poll HTTP ; (5) per-provider model defaulting.

- **(1) Worker-pump multi_thread (D6)** :
  - tokio docs (attr.test / runtime) : sur `current_thread`, une tache `spawn`
    n'avance que quand le futur principal yield ; un acteur de fond continu
    exige `multi_thread`. Mecanisme du hang.
  - **Mecanisme confirme dans le repo** : `node.rs:336-339`
    `Docs::builder().spawn(...)` cree l'**acteur iroh-docs** ; `docs.rs:291-307`
    `get_many_by_prefix` envoie une `Query` a cet acteur et draine un stream.
    Le pump engine (`runtime.rs:680-714`) boucle en `tokio::select!` sur
    `shutdown_rx` + `sleep(poll)` puis `tick()` → `get_many_by_prefix` →
    aller-retour vers l'acteur. Le test (`dispatch_loop.rs:224`,
    `runtime.rs:1586`) `tokio::spawn` le pump et poll en parallele dans un
    `tokio::time::timeout(10s)`. Sur `current_thread`, ces deux taches +
    l'acteur se disputent un seul worker — la liveness depend du flavor.
  - **Fix prouve in-repo en multi_thread** :
    `crates/nexus-core-rs/examples/two_nodes_docs_sync.rs:101`
    `#[tokio::main(flavor = "multi_thread", worker_threads = 4)]` (seul sync
    2-noeuds qui marche) ; `crates/nexus-launcher/src/token_rotation.rs:124`
    `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`. Les 2 tests
    E2E pump sont les **seuls outliers** `current_thread`.
  - Finding : **APPROACH-ALIGNED**. Passer les tests pump a `multi_thread`
    matche le runtime de prod (binaire worker, daemon) et le seul exemple sync
    fonctionnel. Ce n'est pas un masque : c'est aligner le test sur la maniere
    dont le code tourne partout ailleurs.

- **(2) serial_test (P2-OPERATOR-TIMEOUT, option A)** :
  - crates.io/serial_test, docs.rs/serial_test : **3.4.0** (2026-02-21),
    mature, `#[serial]` (intra-process) et `file_serial` (cross-process, file
    locking) — exactement le cas d'un test-group liant des ports + spawn git.
    Licence MIT (compatible AGPL projet). Source : crates.io / GitHub palfrey.
  - Finding : **APPROACH-ALIGNED** (lib mature dispo). Note : alternative
    sans nouvelle dep = timeout client deja present (`operator_server.rs`
    tests :70/:80/:91 = 5s) rendu configurable. Les deux options du plan sont
    valides ; serial_test est la plus propre pour un test-group qui se DoS
    lui-meme sous charge Windows.

- **(3) Vitest mock EventSource (P2-OPERATOR-NO-TEST-RUNNER)** :
  - jsdom n'expose pas EventSource nativement. Patterns matures : MSW (`sse`
    namespace, API post-nov-2024) ou une classe stub `EventSource`
    hand-rolled. Sources : mswjs.io/docs/sse, github binarymist/mocksse.
  - `web/vitest.config.ts` (Vitest v4, jsdom, `setupFiles` `src/test/setup.ts`)
    + `web/package.json` devDeps (`@testing-library/*`, jsdom 29, vitest 4) =
    template in-repo a mirrorer pour `factory-operator`.
  - Finding : **APPROACH-ALIGNED**. Pour 1-3 tests de logique (mapping
    StreamChunk, gate sensitive, no-reconnect), un stub EventSource leger
    (miroir `web/src/test/setup.ts`) suffit ; MSW serait sur-dimensionne.

- **(4) Poll diagnostic preservation (P2-POLL-DIAGNOSTIC-LOSS)** : pattern
  standard « keep last transient error, surface it on deadline » (cf. reqwest
  retry loops, hyper). `provider_router.rs:403/:406` jettent l'erreur via
  `continue` ; :384-389 emet un « timed out » generique. Finding :
  **APPROACH-ALIGNED** avec le plan (memoriser `last_err`, l'inclure au
  timeout).

- **(5) Per-provider model defaulting (P2-OLLAMA-MODEL-PICKER)** : pattern
  trivial (defaut par provider). Finding : **APPROACH-ALIGNED**.

- **Impact S1a global** : aucun. Aucun APPROACH-NAIVE, aucun LIB-EXISTS
  bloquant (serial_test/vitest sont des outils de test additifs, pas un
  remplacement d'une primitive maison). Pas de PLAN-ADAPT.

## S1b Dependencies, CVEs, Release Notes
- Scanned : iroh 0.98.2, iroh-docs 0.98.0, tokio 1.52.3, rusqlite 0.36.0,
  libsqlite3-sys 0.34.0, redb 2.6.3, + dev-deps candidates serial_test 3.4.0
  et vitest/jsdom.
- Commands/sources :
  - `Cargo.lock` (grep name=…) : versions exactes ci-dessus (P2-PREFLIGHT-
    TRANSITIVE-DEPTH applique — versions resolues du lock, pas la contrainte
    `Cargo.toml` `tokio = "1.40"` qui resout a 1.52.3).
  - iroh-docs releases (github.com/n0-computer/iroh-docs/releases) :
    **0.99.0 = 2026-05-08**, contient « Drain Actor::tasks JoinSet in
    run_async » (fix teardown — pertinent au hang Windows) MAIS **breaking**
    (« Update to latest 1.0.0-rc.0 deps and redb@4 ») → **non adoptable** sous
    le pin gele iroh 0.98 (R-iroh-audit P0). La claim D6 du kickoff est exacte.
  - tokio #2499 (github) : « Investigate threaded runtime with single thread
    shutdown on Windows » — Windows-only, **open**. Concerne le scheduler
    threaded-1-thread (pas current_thread stricto sensu) ; classe de hang
    teardown Windows.
  - tokio #7049 (github, 2024-12-24) : « cargo test will hang on windows, it
    may cause by thread_local drop? » — **current_thread runtime**,
    Windows-only, **open** ; hang APRES sortie nominale, lie au drop du runtime
    / thread_local. C'est la classe exacte du worker-pump.
- Finding : **clean (non-bloquant)**.
  - Nuance evidence : la memory/kickoff resume « tokio #2499/#7049 deadlock
    Windows current_thread » ; en realite #2499 vise le threaded-1-thread et
    #7049 le current_thread. Les deux sont des hangs teardown Windows-only ;
    `multi_thread, worker_threads=2` evite les deux classes. Imprecision
    cosmetique, n'invalide pas le fix. A refleter dans `PATTERNS §P54`.
  - Aucune CVE crypto/wire/network/sandbox/signing introduite. serial_test et
    vitest sont dev-only (jamais dans le binaire de prod). tokio multi_thread
    est deja dans la feature `full` deja activee — aucun bump, aucun risque
    supply-chain nouveau cote prod.

## S2 Historical Decisions
- Commands :
  - `git log --all --oneline -- crates/nexus-shell-daemon/src/dispatch_loop.rs`
    → E2E pump introduit `2f9238d` (S71 Phase A B-3) ; pas de commit qui
    revert ou rejette une variante `multi_thread`.
  - `grep -rn "multi_thread|current_thread|P2-A-1|worker-pump" .planning/archive/v2.1
    | grep reject|band-aid|do not|never` → **aucun** resultat (aucune decision
    historique n'interdit le fix multi_thread).
- Decisions crossed :
  - **P2-A-1 worker-pump** : carry documente §P54 (`docs/rust/PATTERNS.md`
    :2871-2882) « Windows-native caveat … environment artefact … verify via
    Docker/CI before push ». Reporte S71→S72→S73, escalade 3/3. Reverse-commit
    check : pas de reversion ; le fix `multi_thread` est une avancee, jamais
    rejetee. **Confirmed non-blocking** (carry legitime, plan le ferme).
  - **Regle modele Claude** (`feedback_model_46.md`) : « toujours
    claude-opus-4-8[1m] partout, jamais alias ». Scope = **agents/invocations
    Claude**, pas backend Ollama. Reverse check : le model-picker garde Claude
    = `claude-opus-4-8[1m]` (operator_server.rs :311) et n'attribue un autre id
    qu'aux providers Ollama/Network (qui ne sont pas des invocations Claude).
    **Pas de contradiction** — la regle n'exige pas qu'Ollama recoive un id
    Claude ; au contraire, lui passer `claude-opus-4-8[1m]` est le bug
    (P2-OLLAMA-MODEL-PICKER). Non-blocking.
  - **PO-14 streaming WAN** (gele S72) : le model-picker et last_err ne
    rouvrent pas le streaming token-par-token reseau (network reste
    submit→poll→un seul Done, provider_router.rs :297-307). Preserve.
- Finding : **clean**. Aucun DESIGN-CONFLICT S2 (pas de fix deja rejete, pas de
  claim figee contredite).

## S3 Local Patterns And Threat Model
- Threats/contracts checked :
  - **T0 loopback trust** : le model-picker passe par `/api/chat/{id}/send`
    (operator_server.rs) deja sous X-SBFB-Token + Host (tier Operator :3001
    formalise S73 Phase A, LOOPBACK §2.1/§8.1). Inchange.
  - **Gate SENSITIVE_ACTIONS** : reste AVANT le dispatch provider
    (operator_server.rs :896-910 puis :934). Le model-picker s'insere APRES le
    gate (:922-928), provider-independant — aucun bypass introduit.
  - **Info leak diagnostic (P2-POLL-DIAGNOSTIC-LOSS)** : surfacer `last_err`
    au timeout ameliore l'observabilite. Risque = fuite d'une URL interne /
    detail HTTP dans le message StreamChunk::Error. Surface = loopback
    Operator local (meme utilisateur), pas reseau → **acceptable**.
    Recommandation non-bloquante : ne pas inclure le token ni l'URL complete
    avec query secrets dans le message (le code actuel ne logge pas le token —
    a preserver).
  - **Model injection (P2-OLLAMA-MODEL-PICKER)** : le `model` est un string
    deja accepte aujourd'hui (ChatSendRequest.model serde default
    operator_server.rs :759). Ajouter un selecteur front ne change pas la
    surface de confiance — le modele est choisi par l'utilisateur local, passe
    a son propre Ollama/daemon. Pas de nouvelle surface (T0-only).
- HARDENING_ROADMAP status : Phase B ne touche aucun pre-requirement
  HARDENING_ROADMAP du sprint (le re-cadrage §3 a ete fait Phase A). Aucun
  pre-requirement manquant pour B.
- Finding : **clean** (S3 full execute — pas de nouveau composant securite,
  pas de nouveau wire ; les changements sont test/durcissement interne sur des
  chemins T0 deja durcis). Aucune regression T0-T5.

## S4 Protocol And Wire Invariants
- Wire/security files checked (target files Phase B) :
  - `dispatch_loop.rs`, `runtime.rs` : `TASK_FORMAT_VERSION` n'apparait qu'en
    **reference fixture** (`:70` `version: TASK_FORMAT_VERSION`, `:1085`
    idem) — aucune definition, aucun bump. Le changement Phase B = attribut
    `#[tokio::test]` uniquement.
  - `provider_router.rs`, `daemon_client.rs`, `operator_server.rs`,
    `process_cli.rs` (tests) : **aucun** symbole `*_VERSION`/`DOMAIN_`/
    `canonical_bytes` (grep `NONE`).
- VERSION/domain/canonical status : inchanges.
  `FEED_FORMAT_VERSION=1`, `TASK_FORMAT_VERSION` inchange,
  `*_ANNOUNCEMENT_VERSION=1`. Aucun decoder tolerant multi-version ajoute.
  `#[serde(default)]` existants (operator_server.rs :759 model,
  provider_router.rs :78) = tolerance runtime pre-launch legitime, pas de
  drift wire. Le model-picker ne change pas le shape wire de ChatSendRequest
  (le champ `model` existe deja).
- Day 0 status : **preserved**. D6 (multi_thread cross-platform, fallback
  exemption formelle) suivi a la lettre. Aucune des decisions Day-0 gelees
  (kickoff §4 D1..D6 ; CLAUDE.md decisions gelees) n'est rebattue.
- Finding : **clean**. Phase B est wire-neutre. Aucun blocage S4, aucun
  DESIGN-CONFLICT.

## Plan Adaptation
Non requis (verdict EXECUTE, pas PLAN-ADAPT).

## Risks And Scope Cuts
- Blocking risks : **none**.
- Non-blocking risks / carry-over (a tracer dans le commit body Phase B) :
  1. **Scope multi_thread plus large que les 2 tests nommes.** Le plan nomme
     l'E2E `dispatch_loop.rs:146` + le miroir `runtime.rs:1497`. Mais
     `runtime.rs` contient ~6 autres tests pump (`:1413,:1479,:1725,:1789,
     :1847,:1971`) avec le meme pattern `tokio::spawn(run_until_shutdown) +
     get_many_by_prefix` sous `#[tokio::test]`, plus `dispatch_loop.rs:92`
     (`dispatch_loop_writes_to_doc`, create_node + spawn, lecture seule).
     **Recommandation** : passer en `multi_thread` TOUS les tests pump qui
     spawn un engine/dispatch concurremment avec un poll d'acteur iroh-docs
     (pas seulement les 2 nommes), pour eviter qu'un `cargo test`
     shared-process Windows hang sur un test non corrige. Non-bloquant car la
     verification canonique reste CI Linux (nextest process-per-test) ; mais
     c'est l'esprit de §P54 « tout test pilotant le pump … DOIT etre
     multi_thread ». A documenter dans §P54.
  2. **Test `chat_stream_uses_opus_model` a preserver vert.**
     `operator_server.rs` test :255-284 (session `provider:"claude"`) assert
     `body.contains("claude-opus-4-8[1m]")`. Le model-picker per-provider DOIT
     garder le defaut Claude = `claude-opus-4-8[1m]` (operator_server.rs :311)
     et ne changer le defaut QUE pour Ollama/Network. Si le fix touche le
     defaut global, ce test casse. Contrainte de compat, non-bloquante si
     respectee.
  3. **EventSource absent de jsdom.** Les tests `factory-operator` qui
     touchent `openStream` (executionChat.ts :77 `new EventSource(...)`)
     exigent un stub EventSource (miroir `web/src/test/setup.ts`). Borner les
     1-3 tests a la logique pure (mapping StreamChunk, gate, no-reconnect) +
     un stub leger, pas un harness SSE complet.
  4. **serial_test = nouvelle dev-dep (si option A retenue).** Absent du
     Cargo.lock ; l'ajouter touche le lock (et le residu Cargo.lock inerte P3
     deja note). Alternative sans dep : timeout client configurable (deja 5s
     en place). Le plan autorise les deux ; choisir et tracer.
  5. **front-only `web/` non touche par Phase B** : le model-picker Operator
     vit dans `tools/factory-operator/` (exemption Rust-first, comme `web/`).
     `pages/ExecutionChat.tsx` est dans `factory-operator`, pas `web/src` —
     verifier le bon arbre avant edition (le plan B.2 cible bien
     `tools/factory-operator/src/.../ExecutionChat.tsx`).
- Scope cuts still honored (kickoff §7) :
  - #11 rate-limit per-client search → S74+ (Phase B ne touche pas search).
  - #13 streaming token-par-token WAN → jamais (PO-14) : last_err + model-picker
    ne rouvrent pas le streaming (network reste submit→poll→un Done).
  - Phase B reste **non-convertible en feature** : aucun item ne deborde vers
    une nouvelle capacite produit (durcissement + dette test uniquement).

## Action
- **EXECUTE** : implementer Phase B comme planifie (B.1-B.5), en integrant les
  5 notes carry-over ci-dessus.
- Le fix D6 `multi_thread` est le chemin primaire confirme root-cause ; le
  fallback exemption formelle (`#[cfg_attr(windows, ignore)]` + §P54 trigger
  iroh 1.0) reste l'issue de secours si un residu teardown (lie au pin 0.98
  pre-0.99 « Drain Actor::tasks JoinSet ») persiste apres multi_thread.
  **Verifier Windows natif + Docker Linux** (`feedback_wsl_before_push`) avant
  de declarer P2-A-1 CLOSED.
- Le commit body Phase B doit citer ce preflight (G8 traceability) et les
  notes carry-over 1-5.
- Ce preflight n'autorise pas le commit a lui seul : gate Codex + review
  `## Verdict: PASS` + body 9 sections restent requis.
