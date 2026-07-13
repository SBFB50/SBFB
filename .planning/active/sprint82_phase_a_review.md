# Sprint 82 — Phase A review
## Verdict: PASS

## Codex reconciliation

Codex GPT-5.6 Sol (reasoning max) — boucle complète : R1 GAP (3 P1) → fixés
root-cause ; R2 GAP (2 nouveaux P1 : rate-limit single-flight + attribution b3)
→ fixés round-3 ; **R3 = PASS, commit gate PASS, 0 P0/P1 restant**. R3 joué sur
le diff collé inline (l'exécuteur fichiers de Codex crashait `-1073741502` cette
session — contourné, pas un défaut de code). 2 gaps NON-BLOQUANTS documentés
(acceptés) : **P2** — le test coalescing prouve le None + le pin mais pas
l'espacement des passes (suggestion Codex : compteur de passes test-only) ;
**P3** — wording PATTERNS §P74 « pace only the trailing pass » à préciser
(grace-window post-passe). Suites relancées après chaque fix : fmt/clippy
--workspace verts, nextest --workspace **2097/2097**, tests daemon round-3 PASS.
Artefact brut : `sprint82_phase_A_codex_review.md` (output `codex exec -o`).

Phase A « Convergence cold-boot catch-up » (boot-SEED, carry P1
S81-G-ESC-1, PLAN-ADAPT). Synthèse + vérification adversariale des
6 dimensions, re-vérifiée au code réel.

**Résumé du gate** : aucun P0/P1 confirmé → pas de FAIL. 4 findings
P2 + 5 findings P3 confirmés au code → rigueur suffisante (pas de
CONCERN). Le T2 live n'est PAS joué (RIG-ABSENT escaladé PO-1=B) →
le verdict reste PASS-**PENDING** : review OK, Codex non encore fait,
et la seule modification comportementale runtime de la phase (l'opt-in
worker `cold_boot_aggressive()`) n'est prouvée que par le T2 live, pas
par un test hermétique (cf. finding docs-honesty ci-dessous).

Toutes les sévérités sont P2/P3 : rien ne bloque le commit ; tout est
à consigner honnêtement dans le body. Un fix trivial est disponible
pour #1 et pour #8 (recommandé, non bloquant).

---

## Findings confirmés par dimension

### ancre-correctness

**[P2 — CONFIRMED] Pré-datation de `last_redrive` neutralisée en
cold-boot (< 30 s d'uptime) → la 1re ingestion est coalescée, pas
immédiate.**
`crates/nexus-shell-daemon/src/runtime.rs:1676-1678` :
`let mut last_redrive = Instant::now().checked_sub(REDRIVE_MIN_INTERVAL).unwrap_or_else(Instant::now);`.
Vérifié : sur Linux `Instant` = `CLOCK_MONOTONIC`, époque = boot kernel.
Sur une ancre VPS systemd démarrée < 30 s après le boot machine,
`now()` < 30 s → `checked_sub(30s)` underflow → `None` → fallback
`Instant::now()` : `last_redrive` N'EST PAS pré-daté. La 1re ingestion
d'annuaire (arrive en quelques s) a alors `elapsed() < REDRIVE_MIN_INTERVAL`
→ `maybe_redrive_seed_on_ingest` retourne `None` (runtime.rs:2081-2083)
→ coalescée. Le re-drive ne se déclenche qu'à ~30 s d'uptime.
Corroboré par le test lui-même : `http.rs:6219-6223` pré-date via
`.expect("machine uptime exceeds the re-drive cooldown")` avec le
commentaire « the monotonic clock epoch is system boot, always > the
cooldown before any test runs » — factuellement FAUX sur une machine
bootée < 30 s (le `.expect(...)` paniquerait ; jamais couvert car les
runners CI ont un uptime >> 30 s). Le commentaire prod runtime.rs:1673-1675
(« re-drives immediately, closing the first-boot dead window fastest »)
est donc faux dans le cas cible le plus froid (ancre VPS lancée par
systemd au boot) — précisément celui que la Phase A vise. Auto-cicatrise
en ≤ 30 s (borné, pas de fuite), mais non couvert par aucun test et
non exercé par le rig d'acceptance (qui redémarre le PROCESSUS daemon
sur machine à uptime >> 30 s, pas une machine froide).
**Fix (trivial, recommandé)** : sentinel qui force le 1er passage —
`let mut last_redrive: Option<Instant> = None;` puis
`if last_redrive.map_or(false, |t| t.elapsed() < REDRIVE_MIN_INTERVAL) { return None; }`.
Ajouter un test simulant `last_redrive = now()` (cold boot) vérifiant
que la 1re ingestion fire tout de même.

### worker-correctness

**[P2 — CONFIRMED] `warm` ne relâche jamais si le 1er `NeighborUp`
est manqué → re-dial agressif ~1 s permanent.**
`crates/nexus-core-rs/src/doc_sync.rs:326-332` : `warm` ne bascule
`true` QUE sur observation d'un événement `NeighborUp` (arm
`Pulled::Event(DocsLiveEvent::NeighborUp)`, l.318). La subscription
iroh-docs est edge-triggered et peut manquer le `NeighborUp` initial —
l'auteur le reconnaît l.353-355 (« Covers the case where the initial
NeighborUp fired before we subscribed »). Dans ce cas `neighbors` reste
vide, `warm` reste `false`, et le backstop l.352-358 utilise
`check_interval_for(false)` = `cold_boot_check_interval` = 1 s +
`min_rejoin_for(false)` = 1 s : `rejoin` est ré-émis toutes les ~1 s
À VIE pour un doc déjà synchronisé (neighbor up, edge manqué). `warm`
est déclaré HORS de la boucle de reconnexion (l.270) donc préservé
entre reconnexions ; aucune autre voie que l'edge `NeighborUp` ne
flippe `warm`, aucun plafond wall-clock ne relâche la fenêtre cold.
Régime ~15× le backstop S77 (15 s → 1 s), permanent et par-doc. La
convergence reste correcte (`start_sync` idempotent), d'où P2. C'est
exactement le « ne relâche jamais » ciblé par la revue.
**Fix** : seconde voie de relâche indépendante de l'edge — flipper
`warm = true` dès que `neighbors` devient non-vide par tout chemin, ou
plafonner la fenêtre cold par une deadline wall-clock (relax après
~60 s de cold quel que soit l'état observé).

**[P3 — CONFIRMED] L'opt-in `cold_boot_aggressive()` côté engine
(engine/runtime.rs:752) n'est ancré par AUCUN test.** — voir dimension
docs-honesty ci-dessous (finding fusionné : la même défaillance est
signalée par worker-correctness [P3] et docs-honesty [P2] ; sévérité
retenue = P2).

### security

**[P2 — CONFIRMED] Le cooldown 30 s borne le taux de LANCEMENT mais
pas l'accumulation de passes re-drive en file sur `seed_driver_lock`
(DoS résiduel via ancre abonnée).**
`runtime.rs:2072-2096` : `*last_redrive` est fixé à `now()` AVANT le
`tokio::spawn` (mesure depuis le lancement, pas la fin, l.2084), puis
la tâche spawnée fait `lock.lock().await` avant `run_boot_seed_driver`.
`http.rs:1696` `DIRECTORY_PULL_TIMEOUT_SECS = 120` : `run_boot_seed_driver`
itère SÉQUENTIELLEMENT `for pid in configured` (http.rs:1847) ; un app
dont les octets sont infetchables (provider dialable mais ne sert pas)
consomme jusqu'à 120 s (`fetch_and_pin`/`fetch_and_pin_multi`,
http.rs:1904-1913 / 1949-1953) → N apps = jusqu'à N×120 s par passe.
Le cooldown `REDRIVE_MIN_INTERVAL = 30 s` (runtime.rs:2042) autorise
un lancement toutes les 30 s tandis que le lock ne draine qu'une passe
par N×120 s → accumulation de futures bloquées (chacune détenant un
clone `Arc<DaemonHttpState>` + `Vec<String>` configured).
Chemin d'attaque vérifié : un ancre ABONNÉ (semi-trusted, l'opérateur
l'a subscribed) annonce un directory signé pour le pid `keep_online`
exact de l'opérateur pointant vers un hash aux octets infetchables,
puis bumpe la révision (anti-rollback trivialement satisfait) toutes
les 30 s. La résolution atteint bien la branche `dir_hit`
(http.rs:1872-1876 : `direct`=None pour un app jamais publié localement
→ `keep_online_row` hash None → `dir_hit` fournit le hash ; http.rs:1932
branche `fetch_and_pin_multi` 120 s). Chaque ingest accepté relance une
passe qui timeout ; les tâches s'empilent plus vite qu'elles ne drainent.
Gated derrière un ancre abonné (limite la sévérité à P2, non-bloquant),
mais l'analyse DoS du preflight (« cooldown 30 s + accept-list
config-only + subscription-gate suffisent ») est incomplète : elle
couvre l'injection de cible et le taux de lancement, PAS la borne sur
les passes concurrentes en file.
**Fix** : remplacer le `spawn` inconditionnel par un `try_lock` (ou un
`AtomicBool` in-flight) et retourner `None` si une passe est déjà en
cours — borne les re-drives en vol à 1 et rend le cooldown suffisant.
Alternative : ne mettre à jour `last_redrive` qu'à la FIN de la passe.

**[P3 — CONFIRMED] `seed_voluntary` (POST /api/daemon/seed) reste hors
du `seed_driver_lock` — 3e émetteur de `SeedAnnounced` non sérialisé.**
`http.rs:2509` `seed_voluntary` + son émission `http.rs:2695`
`emit_seed_announced` n'acquièrent PAS `seed_driver_lock` (le lock de
Phase A ne couvre que `run_boot_seed_driver` : boot driver
runtime.rs:1195 + re-drive runtime.rs:2089). Un seed manuel opérateur
concurrent d'un re-drive pour le même pid peut lire
`was_already_announced`=false des deux côtés et double-émettre.
Pré-existant (le boot driver ne partageait déjà aucun lock avec
`seed_voluntary`) et non-fatal (émission best-effort, dédup côté ingest
via `SeedRegistry`) ; mais la doc du lock (runtime.rs:2061-2063 +
1125-1128) devrait noter explicitement le périmètre (boot driver +
re-drive uniquement) pour ne pas suggérer une exhaustivité qu'elle n'a
pas. **Fix** : documenter le périmètre exact, ou faire acquérir le lock
à `seed_voluntary` pour fermeture totale.

**[P3 — CONFIRMED] La cadence cold-boot 1 s ne se borne pas dans le
temps si aucun voisin ne se forme jamais.**
`doc_sync.rs:270` `warm` reste `false` tant qu'aucun `NeighborUp`
n'arrive ; `check_interval_for(false)` = 1 s, `min_rejoin_for(false)`
= 1 s. Si le coordinateur (peers du ticket) est mort, le worker re-dial
`start_sync` toutes les ~1 s indéfiniment — 15× le backstop 15 s — sans
retomber sur la cadence steady. Borné aux propres peers du ticket (pas
un tiers), délibéré pour la convergence, surface de sécurité nulle,
d'où P3. **Fix (optionnel)** : après un budget de warmup (~60 s sans
`NeighborUp`), retomber sur la cadence steady même en cold. Note : même
mécanisme racine que le finding P2 worker-correctness ci-dessus — un
plafond wall-clock les ferme tous deux.

### invariants-grounding

**[P3 — CONFIRMED] Claim « 0 observable change » du défaut légèrement
sur-affirmé (invariant tenu EN PRATIQUE).**
`doc_sync.rs:326-332` : le bloc de transition warm
`if !warm { warm = true; ticker = interval(config.check_interval); ... }`
s'exécute inconditionnellement pour TOUTES les configs, y compris
`default()`, alors que le doc-comment de `cold_boot_check_interval`
(l.122-125) et le test (l.520-522) affirment que le défaut garde « the
exact Sprint 77 behavior (0 observable change) ». Au 1er `NeighborUp`,
un appelant en config défaut reconstruit son ticker (réalignement de
phase du backstop) — action absente en S77.
L'invariant « refacto = 0 comportement observable ailleurs » tient
malgré tout : (a) pour le défaut `check_interval == cold_boot_check_interval`
donc cadence identique ; (b) le tick immédiat du rebuild est neutralisé
par le garde `neighbors.is_empty()` (le `NeighborUp` vient d'insérer,
neighbors non-vide → rejoin skippé) ; (c) le SEUL appelant de production
(`engine/runtime.rs:752`) utilise `cold_boot_aggressive()`, jamais le
défaut. Nuance de grounding, pas un défaut de correction. **Fix
(optionnel)** : garder le rebuild par
`if !warm && config.cold_boot_check_interval != config.check_interval`,
ou noter le réalignement de phase inerte dans le commentaire.

### docs-honesty

**[P2 — CONFIRMED] « Revert-proof structurel » sur-revendiqué :
reverter l'opt-in worker laisse tous les tests T1 verts.**
`doc_sync.rs:499-501` doc-comment du test
`cold_boot_config_accelerates_only_the_cold_window` : « Revert-proof:
switch the worker back from `cold_boot_aggressive()` to `default()` and
the strict inequalities below fail. » et l.522 « This is the revert-proof
of the worker's switch. » FAUX : le test (l.504) construit
`KeepaliveConfig::cold_boot_aggressive()` DIRECTEMENT et ne référence
jamais le call-site worker. Vérifié par grep : `engine/runtime.rs:752`
est le SEUL usage worker de `cold_boot_aggressive()` et AUCUN test de
`nexus-worker-core` n'asserte le config passé par
`Engine::run_until_shutdown` (grep `cold_boot_aggressive|KeepaliveConfig`
sur le crate = usage seul, 0 assertion). Reverter :752 →
`KeepaliveConfig::default()` ne change RIEN au test → il reste vert.
Or :752 est l'UNIQUE ligne qui change le comportement runtime pour
fermer S81-K, et elle n'est gardée par aucun test hermétique. Le test
revert-proofe seulement la sémantique du CONSTRUCTEUR (le vider →
inégalités échouent), pas le choix du worker. L'acceptance (« WORKER
cadence, revert-proof structurel » ; « Revert `cold_boot_aggressive()`
→ `default()` fait échouer les inégalités strictes ») répercute cette
attribution trompeuse. Le gate « T1 GREEN prérequis DUR du commit » ne
couvre donc PAS la seule modification comportementale de la phase.
**Fix** : reformuler doc-comment + acceptance — le test revert-proofe
la sémantique du constructeur `cold_boot_aggressive()` ; l'opt-in worker
lui-même n'est prouvé que par le T2 live (cohérent avec l'honnêteté
transport-only déjà écrite l.489-497). OU ajouter un test worker-side
assertant que l'Engine spawn la keepalive avec `cold_boot_aggressive()`
(config observable) pour garder réellement le call-site.

**[P3 — CONFIRMED] T2 artifact = super-ensemble de la shape canonique
du script (champ `note` en trop).**
`.planning/active/sprint82_t2_bootseed.json` porte les 7 champs
canoniques (status/stage/delay_s/claim_s/task_id/diagnosis/last_response)
PLUS un champ `note` ; `scripts/acceptance/b3_live_pc_vps.sh`
`emit_artifact` (l.186-194 branche python + l.200-202 branche printf)
n'émet QUE les 7 champs. Le vrai run opérateur écrasera l'artefact et
supprimera `note`. Bénin (JSON tolérant, `note` = documentation-only ;
status=RIG-ABSENT + stage=preflight correspondent exactement à
`rig_absent()` l.208-213). Pas un défaut d'honnêteté, mais la shape
n'est pas byte-identique à la sortie du script. **Fix (optionnel)** :
retirer `note` pour parité stricte, ou noter la divergence cosmétique.

---

## Vérification sémantique des tests

- **`redrive_on_ingest_pins_configured_app_without_restart`
  (http.rs, ~6180-6320)** — red→green RÉEL. CONTROL (l.6225-6242) :
  avant ingestion d'annuaire, `maybe_redrive_seed_on_ingest` retourne
  `Some` (cooldown pré-élapsé) mais la passe pinne 0 (app non
  résolvable) + `!has_tag` → la fenêtre morte reproduite. FIX
  (l.6256-6273) : après `ingest_remote_directory` du pid configuré, la
  passe pinne 1 + `has_tag` + `get_keep_online == Some((true, Some(hash)))`
  → l'app est acquise+pinnée SANS redémarrage. Revert-proofs VALIDES :
  COOLDOWN (l.6294-6306) — 2e appel avec `last_redrive` non-avancé →
  `None` (retirer le garde cooldown → `Some` → échec) ; EMPTY-config
  (l.6308-6317) — `&[]` → `None` (retirer le garde `is_empty` → `Some`
  → échec). RÉSERVE : le pré-datage du test via
  `.expect("machine uptime exceeds the re-drive cooldown")`
  (l.6221-6223) paniquerait sur une machine bootée < 30 s (cf. finding
  ancre-correctness P2) — non couvert.
- **`cold_boot_config_accelerates_only_the_cold_window`
  (doc_sync.rs:502-526)** — teste UNIQUEMENT la logique de sélection de
  cadence du constructeur (`check_interval_for` / `min_rejoin_for` sur
  `cold_boot_aggressive()` et `default()`). N'exerce NI la boucle
  keepalive NI le call-site worker. Honnêtement documenté que le
  bénéfice de convergence est transport-only et prouvé par le T2 live
  (l.489-497). SEUL le sous-claim « revert-proof of the worker's switch »
  est sur-revendiqué (cf. finding docs-honesty P2).
- **`keepalive_rejoins_doc_after_neighbor_loss`
  (doc_sync.rs:408-487)** — red→green 2-nœuds RÉEL du chemin
  `NeighborDown` : CONTROL (l.458-461) le réplica détaché ne converge
  PAS sur `k2` sans re-join (reproduit `recv:0`) ; FIX (l.477-480) la
  keepalive re-join et `k2` converge. Config `cold_boot_* == steady`
  (l.466-474) — n'exerce PAS le warmup cold-boot, honnêtement noté.
  Le littéral cfg a bien été mis à jour avec les 2 nouveaux champs.
- **`gossip_cmd_outbox_persists_to_db` (runtime.rs, défaut
  GossipTaskConfig l.3304-3306)** — les 3 nouveaux champs sont câblés
  `boot_driver_state: None` / `keep_online_projects: vec![]` /
  `seed_driver_lock: new Mutex` → un test de la tâche gossip sans état
  driver ne re-drive rien (garde `Some(ref bds)` runtime.rs:1801).
  Cohérent.

Conclusion tests : les red→green sont réels et les revert-proofs
cooldown/empty-config tiennent. La seule faille de couverture est
l'opt-in worker (`engine/runtime.rs:752`), non gardé par un test
hermétique — la doc doit cesser de prétendre le contraire.

---

## Invariants durs

- **0 bump wire SBFB** : TENU. `GossipTaskConfig` (+3 champs) et
  `KeepaliveConfig` (+2 champs) sont des structs internes, jamais
  sérialisées sur le fil. Aucune constante `*_ANNOUNCEMENT_VERSION` /
  `FEED_FORMAT_VERSION` touchée.
- **0 dep runtime** : TENU. Aucune nouvelle dépendance ; réutilise
  `tokio::sync::Mutex`, `Instant`, `start_sync` existants.
- **iroh =1.0.1** : TENU (aucune modification du pin).
- **refacto = 0 comportement observable ailleurs** : TENU EN PRATIQUE
  (cf. finding invariants-grounding P3 : réalignement de phase inerte
  du ticker pour le défaut, neutralisé par le garde `neighbors.is_empty()`
  et par le fait qu'aucun appelant de production n'utilise le défaut).
- **Duress-safe (DURESS-BOOT-LEAK)** : TENU. `run_boot_seed_driver`
  duress-gate en PREMIÈRE instruction (http.rs:1840-1844, `return 0`) ;
  `maybe_redrive_seed_on_ingest` ne lit ni `keep_online` ni pid ni
  `project_id` avant ce gate (runtime.rs:2072-2096, il passe juste
  `configured` à la primitive gatée). Un nœud decoy re-drive `0`.
- **heberger != publier, seeder != auteur** : TENU. Le re-drive ne fait
  qu'acquérir+pinner des octets content-addressés (BLAKE3), jamais
  publier/signer une authorship ; il n'itère que l'accept-list
  `configured` de l'opérateur, jamais un pid réseau.

---

## À corriger avant commit (P0/P1)

**AUCUN.** Zéro P0/P1 confirmé. Le commit n'est pas bloqué par la review.

Recommandations non-bloquantes (fixes triviaux, à l'appréciation) :
- **#1 (P2 ancre)** : sentinel `Option<Instant>` pour que la 1re
  ingestion fire même sur ancre VPS bootée < 30 s — ferme un gap dans
  l'environnement CIBLE de la phase, ~5 lignes.
- **#8 (P2 docs-honesty)** : reformuler le doc-comment/acceptance
  « revert-proof of the worker's switch » — pur texte, aligne la doc
  sur la réalité du gate T1.

## À documenter dans le body du commit (P2/P3)

Consigner les 9 findings confirmés, en particulier :
1. **[P2]** Pré-datage cold-boot < 30 s neutralisé (auto-heal ≤ 30 s,
   untested, env cible VPS).
2. **[P2]** `warm` non-relâché si `NeighborUp` initial manqué →
   re-dial 1 s permanent (convergence correcte, gaspillage durable).
3. **[P2]** DoS résiduel : cooldown borne le lancement mais pas les
   passes concurrentes en file sur `seed_driver_lock` (gated ancre
   abonnée ; l'analyse DoS du preflight est incomplète).
4. **[P2]** « Revert-proof » de l'opt-in worker sur-revendiqué : la
   seule ligne comportementale (`engine/runtime.rs:752`) n'est prouvée
   que par le T2 live (RIG-ABSENT), pas par T1.
5. **[P3]** `seed_voluntary` hors périmètre du `seed_driver_lock`
   (3e émetteur non sérialisé, pré-existant, non-fatal).
6. **[P3]** Cadence cold 1 s non bornée dans le temps si aucun voisin
   ne se forme (délibéré, borné à son propre coordinateur).
7. **[P3]** Claim « 0 observable change » du défaut tenu en pratique,
   pas littéralement (réalignement de phase inerte).
8. **[P3]** T2 artifact = super-ensemble de la shape script (champ
   `note` cosmétique, écrasé au vrai run).

Rappel gate testabilité : T2 live boot-SEED = **RIG-ABSENT** escaladé
PO-1=B (compteur S81-G-ESC-1 à 3/3) — la feature reste PROVISIONAL
jusqu'au `b3 PASS < 30 s` sur rig opérateur ; le body doit le refléter
honnêtement (pas de `DIFFERE-materiel` en prose muette).

Prochaine étape : gate Codex (BLOQUANT review→commit) sur le diff.
