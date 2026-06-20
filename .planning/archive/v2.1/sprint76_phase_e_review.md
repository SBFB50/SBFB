# Sprint 76 Phase E — Review (dashboard contributeur, D4)

## Verdict: PASS

**Verdict initial de la review : CONCERN (1 P1). Le P1 a ete RESOLU a la racine
(cf. `## P1 resolution`) ; review -> PASS-PENDING ; puis Codex GPT5.5 CLEAN (14
CONFIRME / 0 GAP / 0 PARTIEL) -> promu PASS (cf. `## Codex reconciliation`).**
0 P0, 0 P1 restant, 4 P2 (3 traites + 1 deja documente), 2 P3 (traites).

La review adversariale (11 agents, fan-out 5 dimensions + 5 verifications
adversariales) a trouve un seul P1 reel mais MATERIEL : le seul producteur prod de
`generation_time_ms` (worker `runtime.rs:1125`) le codait en dur a `0`, ce qui
declenchait le sanity-bound a son reglage le plus serre (`ceiling = 1000`) pour
TOUTE credit honnete >1000 tokens — degradant le signal kudos existant en prod au
lieu de seulement borner l'absurde. Les 2 risques BLOQUANTS du preflight (R1
GPU-heures, R2 route auth) et le risque sur-promesse (R4 anti-Sybil) etaient deja
tous mitiges, et toutes les decisions gelees tiennent.

## P1 resolution (fix post-review, root cause)

Le P1 etait un bug LATENT pre-existant (le champ valait toujours 0, jamais consomme
avant que le sanity-bound n'en devienne le 1er lecteur prod). Fix a la racine, PAS
un band-aid :
- **Worker mesure la vraie duree** (`engine/runtime.rs`) : `Instant` monotone capture
  AVANT `self.llm.generate(...)`, `generation_time_ms = gen_start.elapsed().as_millis()`
  apres ; `started_at`/`finished_at` bracketent le meme appel. Le `generation_time_ms: 0`
  en dur est supprime. Consequence : en prod les credits honnetes portent une duree
  reelle (centaines-milliers de ms pour une inference LLM) => ceiling >> tokens =>
  AUCUN clamp d'un honnete ; seul l'absurde (gen_ms minuscule + tokens enormes) est borne.
- **Test du chemin worker (deterministe, non-flaky)** : `StubBackend::with_delay_ms`
  (additif, defaut 0) force une latence d'inference ; le test E2E existant
  `dispatched_task_is_claimed_and_executed_by_worker_engine` (`dispatch_loop.rs`)
  utilise `with_delay_ms(5)` et asserte `result.payload.generation_time_ms >= 1`
  (+ `finished_at >= started_at`) sur le `ResultEntry` signe deserialise — une
  regression au hardcode 0 echouerait ce test.
- **Test de non-regression** au niveau borne : `sanity_bound_preserves_honest_large_credit`
  (`kudos_ledger.rs`) prouve qu'un credit honnete volumineux (5000 tokens / 30 s) passe
  UNCLAMPED et out-score le clamp plat.
- `sanity_bounded_tokens` (`.max(1)`) est CONSERVE : apres le fix worker, un honnete
  n'a jamais gen_ms=0 ; un gen_ms=0 + gros tokens = claim implausible -> clampe (voulu).
- Re-verification mecanique : `cargo nextest run --workspace` **1799 passed 0 skipped**
  (= 1789 + 10) ; fmt/clippy --all-targets/doctests verts ; front vitest 397 + coverage
  87.2/79.01/85.92/88.52. Aucun nouveau finding introduit par le fix.

## P2/P3 resolution
- **P2 raw_kudos transite-mais-mort** : RETIRE du wire (struct `ContributorSummary`,
  handler `kudos_api.rs`, Zod `ContributorSummarySchema`, mock test) — plus de champ mort.
- **P2 refetchInterval 5000** : commentaire de justification ajoute (`Network.tsx`,
  cadence credit = minutes => 5s = poll leger assume) + mentionne au commit body.
- **P2 commit body +9 vs +5** : le delta REEL est **+10 Rust / +1 Vitest** (le P1 fix
  a ajoute `sanity_bound_preserves_honest_large_credit`) ; reconcilie 1789->1799.
- **P2 median-de-groupe DOC-P2** : deja documente honnetement (THREAT_MODEL §15.3 +
  preflight) ; non implemente (verifie).
- **P3 tasks_served unicite** : note doc ajoutee dans `ContributorSummary` (unicite
  `(task_id, worker)` garantie en amont par `UNIQUE(task_id, worker_id)` sur
  `task_results`, pas par la table kudos).
- **P3 PATTERNS §P61** : ajoute (`docs/rust/PATTERNS.md`) — sanity-bound plausibility-check
  sur input de reward auto-declare hors-quorum, 3 proprietes (borne asymetrique /
  chokepoint centralise / input borne doit etre REEL end-to-end).

## Suites §7.4 (rappel mecanique)
Reportees vertes par l'implementeur (mecanique, non re-executees dans cette review) :
- `cargo fmt --all --check` OK
- `cargo clippy --workspace --all-targets --locked -- -D warnings` OK
- `cargo nextest run --workspace --locked` : 1798 passed 0 skipped (= 1789 + 9)
- `cargo test --workspace --locked --doc` OK
- front `npm run lint` 0 err ; `tsc --noEmit` OK
- `npm run test:unit` (vitest) 397 passed (= 396 + 1)
- `npm run test:coverage` 87.2 / 79.01 / 85.92 / 88.52 (>= 85/78/85/85)
- `npm run build` + `npm run size` + `scan-en-strings.sh` OK
- release build `cargo build -p nexus-shell-daemon --release` : en cours (parallele)

## Dimensions

### D1 Correctness + tests semantiques (Rust)
Verdict dimension : CONCERN (1 P1, reste OK). Mecanique du dashboard correcte et
bien testee semantiquement. Findings retenus :

- **[P1] Worker code `generation_time_ms` a 0 en dur -> sanity-bound clampe TOUTE
  credit honnete a <=1000 tokens en prod.**
  Evidence : `crates/nexus-worker-core/src/engine/runtime.rs:1125` `generation_time_ms: 0,`
  est le SEUL site de construction `ResultPayload` non-test (confirme par grep :
  les autres a `validator.rs:450/466`, `http.rs:8068/8480`, `result_sync.rs:301`,
  `validator_loop.rs:179`, `verification.rs:295`, `task.rs:697` sont `#[cfg(test)]`/helpers).
  Le worker calcule `let now = now_unix_secs();` (`runtime.rs:1119`) et pose
  `started_at: now, finished_at: now` (`runtime.rs:1128-1129`) — AUCUNE duree reelle
  n'est capturee. Les 2 sites prod credit() (`http.rs:3471`, `validator_loop.rs:115`)
  passent `entry.payload.generation_time_ms`, donc `0` en prod. Dans
  `sanity_bounded_tokens(tokens, 0)` (`kudos_ledger.rs:44-47`) : `ceiling =
  TOKENS_PER_MS_CEILING.saturating_mul(0.max(1)) = 1_000` -> `tokens.min(1000)`.
  Le test `sanity_bound_clamps_implausible_token_claims` (`kudos_ledger.rs:539`)
  PROUVE le comportement : `sanity_bounded_tokens(10_000, 0) == TOKENS_PER_MS_CEILING`
  (= 1000). Donc tout resultat honnete >1000 tokens s'ecrase sur la valeur plate
  `log_utility(1000)` ~ 10967 au lieu de sa vraie valeur (ex 4000 tokens ->
  `log_utility(4000)` ~ 12966, ~15% de sous-credit ; tous les resultats >1000 tokens
  deviennent indistinguables). Le doc-comment (`kudos_ledger.rs:32-34`) affirme que la
  borne « bounds absurdity (1e9 tokens in 5 ms), not a real rate » et est rarement
  active — en prod elle est TOUJOURS active au reglage le plus serre. Aucun test ne
  l'attrape : tous les test-calls credit() passent un `1_000` litteral, jamais le
  chemin worker reel (gen_ms=0).
  Recommandation : cabler la vraie duree cote worker (`finished_at - started_at` en ms,
  ou timer de decode de l'engine ; `now` est deja calcule a `runtime.rs:1119`).
  Alternative court-terme defendable : traiter `generation_time_ms == 0` comme
  « inconnu » = PAS de clamp (early-return `tokens` si `gen_ms == 0`) plutot que floor
  a 1000, et tracer la dette ; sinon la feature degrade le signal kudos en prod.
  Ajouter un test qui construit le payload via le chemin worker (gen_ms=0, tokens=4000)
  et asserte le comportement voulu.

- **[OK] `sanity_bounded_tokens` formule/overflow/floor corrects et testes.**
  `kudos_ledger.rs:44-47` : `saturating_mul` evite l'overflow ; `.max(1)` evite le
  collapse a 0. Test `kudos_ledger.rs:522` couvre plausible-non-clampe (500/5000->500),
  absurde-clampe (1e9/5 -> 5*CEILING, assert log_utility strictement inferieur), gen_ms=0.

- **[OK] credit() clampe AVANT log_utility, prouve par lecture DB.**
  `kudos_ledger.rs:97` `let bounded_tokens = sanity_bounded_tokens(...)` puis `:104`
  `amount: log_utility(bounded_tokens)`. Test `credit_applies_sanity_bound_to_amount`
  (`kudos_ledger.rs:542`) relit `gamed.amount` via `get_project_entries` et asserte
  `== log_utility(5*CEILING)` ET `< log_utility(1e9)`.

- **[OK] get_contributor_summary reutilise effective_score() exactement ; agregats
  coherents et deterministes.** Meme `effective_score` (EMA `KUDOS_EMA_ALPHA=0.97`,
  `kudos_ledger.rs:18`). `effective_total == sum(per_project)` (test
  `get_contributor_summary_aggregates_ema` `kudos_ledger.rs:581-582`) ;
  `tasks_served = COUNT lignes` (test `counts_tasks_served` `kudos_ledger.rs:606`,
  isolation worker-b) ; tri deterministe ; cas vide teste (`kudos_ledger.rs:618`).
  Nuance P3 doc : `tasks_served` compte des lignes ledger ; l'unicite
  `(task_id, worker)` est garantie en amont par status-guard +
  `UNIQUE(task_id, worker_id)` sur `task_results`, pas sur la table kudos (coherent
  avec leaderboard existant, pas une regression).

- **[OK] get_worker_entries : filtre correct + test EXPLAIN QUERY PLAN asserte l'index.**
  `WHERE worker_node_id = ?1` ; test `contributor_query_uses_worker_index` asserte
  `plan.contains('idx_kudos_worker')` (index pre-existant).

- **[OK] contributor_dashboard miroir fidele de leaderboard, lock+Err -> 500, tests
  via vrai router.** `kudos_api.rs` meme pattern que `leaderboard()` ; tests via
  `build_test_router(...).oneshot(...)`.

- **[OK] Tous les sites credit() ont le nouveau param ; les 2 sites prod passent la
  vraie valeur du payload** (`http.rs:3471`, `validator_loop.rs:115` = `entry.payload`,
  meme objet signe que `tokens_generated`). Aucune valeur factice cote credit() ; le
  defaut prod 0 vient du WORKER (cf. P1), pas d'un mauvais cablage credit().

### D2 Scope / PLAN-ADAPT / Decisions gelees
Verdict dimension : PASS. Les 8 adaptations PLAN-ADAPT presentes et conformes ;
0 decision gelee violee (0 P0). 0 finding bloquant.
- PLAN-ADAPT #1 : aucun CREATE INDEX/M20 ; `idx_kudos_worker` pre-existant (`db.rs:50`),
  test = EXPLAIN QUERY PLAN.
- PLAN-ADAPT #2 : median-de-groupe DOCUMENTE P2 (THREAT_MODEL §15.3), PAS implemente
  (grep `median` sur le diff Rust = 0 hit).
- PLAN-ADAPT #3/#5/#7 : credit() +`generation_time_ms` aux 2 sites prod ;
  `effective_score` reutilise verbatim (`KUDOS_EMA_ALPHA=0.97`) ; Zod `.strict()`
  miroir.
- #4 (route authed), #6 (GPU-heures locales), #8 (THREAT_MODEL §15.3) : cf. D3.

### D3 Securite + honnetete (R1 GPU-hours, R2 route auth, R4 sanity-bound)
Verdict dimension : PASS. Les 2 risques BLOQUANTS et le risque sur-promesse mitiges.
0 finding bloquant.
- **R2 mitige** : route `/api/v1/contributor/{node_id}` DANS `authed_routes`
  (`http.rs:454-457`, bloc termine par `.layer(...auth_required)` `http.rs:495`).
  `public_routes` ne contient QUE `/health` + `/blob-serve` (`http.rs:253-255`).
- **R1 mitige** : GPU-heures = `consent.hours_used_today` LOCAL (snapshot worker,
  `Network.tsx:180`), libelle « GPU-heures donnees par cette machine aujourd'hui
  (non attestees) ». La route contributor (`kudos_api.rs`) ne renvoie AUCUN champ GPU ;
  doc explicite « never aggregated server-side ». `ContributorSummary` Rust sans champ
  GPU.
- **R4 mitige** : sanity-bound documente comme plausibility-check (BOINC
  `wu.rsc_fpops_bound` analogue), PAS anti-Sybil ; residuel forge-coherente = Sybil
  multi-keypair pre-existant §15.2. `TOKENS_PER_MS_CEILING=1000` ~ genereux pour un
  debit reel. NOTE TRANSVERSALE : la borne « ne clampe jamais un honnete » n'est vraie
  qu'avec un `generation_time_ms` realiste ; cf. P1 (D1) — avec gen_ms=0 en prod elle
  clampe TOUT honnete >1000. L'honnetete du LABEL est correcte ; l'efficacite REELLE
  est compromise par le producteur worker.

### D4 Front + identite
Verdict dimension : PASS (2 P2 robustesse mineurs). Invariant identite critique tient :
`node_id == hex(worker_pubkey)` (NodeId iroh 0.98 Display = HEXLOWER des 32 octets ==
`hex::encode(public_bytes())`, meme keypair endpoint+signature) -> la query matche les
lignes kudos, vue PAS toujours-vide.
- **[OK] Identite node_id == hex(worker_pubkey)** : chaine `runtime.rs:1249` ->
  `node.rs:173` `endpoint.id().to_string()` ; endpoint du meme keypair
  (`runtime.rs:256`) ; resultats signes du meme keypair (`runtime.rs:1132` ->
  `worker_pubkey = keypair.public_bytes()`). Deja eprouve par la prod /nodes
  (`http.rs:5719` asserte `node_id_hex == hex::encode(pow_keypair.public_bytes())`).
- **[OK] Zod `.strict()` miroir EXACT** du JSON handler (5 champs top-level + 3
  per_project ; rename interne `effective_total` -> wire `effective_kudos` correctement
  aplati cote handler).
- **[OK] ContributorCard** : 3 metriques, `enabled: nodeId !== undefined`,
  isLoading/isError geres (FR), libelle GPU honnete.
- **[OK] Test 3-metriques** attend la valeur resolue (`findByText`) + asserte le
  libelle honnete (cette machine / non attestees).
- **[P2] Valeurs numeriques non bornees a l'affichage** : la carte expose uniquement
  `effective_kudos` ; `raw_kudos` transite dans le wire mais est inutilise cote UI ;
  pas de separateur milliers. Optionnel/non-bloquant — decision produit (afficher
  raw a cote de l'EMA OU retirer le champ pour eviter un champ transite-mais-mort).
- **[P2] `refetchInterval: 5000`** sur la carte contributeur : polling 5s desynchronise
  de la cadence reelle des credits kudos (ordre minute). Loopback benin (`retry:0`),
  mais a aligner (15-30s) ou indexer sur le refetch du worker snapshot ; a mentionner
  comme choix assume dans le commit body.

### D5 Research grounding + Patterns + Doc
Verdict dimension : PASS-PENDING. Grounding sain, doc honnete. 1 P2 (commit body) + 1 P3.
- **[OK] Analogie BOINC `rsc_fpops_bound` fidelitueuse, pas un overclaim** : BOINC
  CreditNew utilise ce bound comme seuil de plausibilite sur un chiffre auto-declare
  (PFC) alimentant le credit ; role partage (anti-inflation sur un input de reward
  auto-declare). Le mot « analogue » est calibre — pas de revendication d'identite de
  mecanisme.
- **[P2] Le commit body doit declarer la sur-livraison de tests : +9 Rust reels vs
  +5 plan.** Compte dans le diff : `kudos_ledger.rs` +5 `#[test]`, `db.rs` +2,
  `http.rs` +2 = 9 Rust ; `Network.test.tsx` +1 = 1 Vitest. `plan.md §8 E.5`
  annoncait « +5 Rust +1 Vitest ». Reconcilier avec le delta nextest §7.4
  1789->1798 (+9). Ne pas propager le +5 perime.
- **[OK] Coherence doc** : 0 GPU-heure ecrite/lue cote coordinator ; « validator
  INCHANGE » tient (grep `validate_quorum_pre_guardrail` = 0 hit dans le diff) ;
  median correctement NON implemente (DOC-P2). Reference `task.rs:476,481` exacte.
- **[P3] Nouvelle entree PATTERNS.md genuinement net-new** (recommandation seule) :
  §P61 « sanity-bound plausibility-check sur un input de reward auto-declare,
  hors-quorum » (propriete de borne asymetrique ; distinct de §P36/§P60.2 qui couvrent
  l'accord multi-worker, pas le clamp d'un auto-report solo).

## Verification adversariale (5 claims)
1. **Route contributor sous authed_routes (PAS public_routes)** — TIENT (non refute).
   `http.rs:454-457` dans le bloc `authed_routes` (debut `:275`, fin `.layer(auth_required)`
   `:495`) ; `public_routes` = `/health` + `/blob-serve` seulement (`:253-255`).
2. **GPU-heures LOCALES, libellees honnetement, jamais agregees ni renvoyees par la
   route** — TIENT (non refute). Route response sans champ GPU (`kudos_api.rs`) ;
   struct `ContributorSummary` sans champ GPU ; UI source = snapshot worker local
   (`Network.tsx:180`) ; libelle « cette machine / non attestees ».
3. **state.node_id == hex(worker_pubkey)** — TIENT (non refute). iroh-base 0.98
   `PublicKey::Display = HEXLOWER` (PAS z-base-32) ; meme 32 octets keypair pilotent
   endpoint ET signature ; `hex::encode` (lowercase) == `HEXLOWER` -> chaines 64-hex
   byte-identiques ; read path sans re-encode. La vue n'est PAS toujours vide.
4. **validate_quorum_pre_guardrail / validator.rs INCHANGE** — TIENT (non refute).
   `validator.rs` absent du diff ; `validate_quorum_pre_guardrail` (defini `validator.rs:219`)
   n'apparait nulle part dans le diff ; seul `validator_loop.rs:115` (event-loop daemon
   distinct) ajoute 1 ligne `entry.payload.generation_time_ms` a credit().
5. **Median-de-groupe N NON implemente, seul le sanity-bound per-entry est code** —
   TIENT (non refute). `grep median` -> uniquement THREAT_MODEL §15.3 (DEFERRED P2) ;
   aucune modif `task_results`/`insert_task_result`/schema/quorum ; `generation_time_ms`
   pre-existant (`task.rs:481`), 0 bump wire.

Les 5 refutations de l'adversaire (toutes `refuted=false`) TIENNENT a l'inspection.
AUCUN faux drapeau a rejeter. Le P1 (D1) N'EST PAS contredit par les claims
adversariales : aucune ne porte sur le producteur worker `generation_time_ms` ; il est
conserve.

## Decisions gelees — statut
- **kudos NON-MONETAIRE 0-token** : TENU. `git diff | grep -Ei
  'cost|deposit|stake|burn|refund|escrow|currency|wallet|monetary|price'` = 0 hit ;
  seul `monet` = descriptif (`Network.tsx` « Kudos non monetaires »).
- **validator (validate_quorum_pre_guardrail) INCHANGE** : TENU (cf. claim 4).
- **0 bump wire** : TENU. `grep '_VERSION|DOMAIN_|FORMAT_VERSION'` = 0 hit ;
  `tokens_generated`/`generation_time_ms` champs pre-existants (`task.rs:476,481`).
- **self-view per-node, PAS de ranking reseau-wide (EigenTrust rejete)** : TENU. Route
  keyee 1 seul `node_id` ; doc-comment + THREAT_MODEL §15.3 explicites.

## P0/P1/P2/P3 count (apres resolution)
- P0 : 0
- P1 : 0 (le P1 worker `generation_time_ms: 0` est RESOLU a la racine — cf. `## P1 resolution` ; verdict CONCERN -> PASS-PENDING)
- P2 : 0 restant (raw_kudos RETIRE ; refetchInterval documente ; commit body reconcilie +10/+1 ; median deja DOC-P2)
- P3 : 0 restant (tasks_served note doc AJOUTEE ; PATTERNS §P61 AJOUTE)

## Risques residuels / a surveiller (post-Codex)
1. **P1 prod-degrade (BLOQUANT de cette review)** : tant que le worker code
   `generation_time_ms: 0` (`runtime.rs:1125`), le sanity-bound ecrase TOUTE credit
   honnete >1000 tokens a une valeur plate, supprimant la differentiation que le
   dashboard est cense exposer. Resolution attendue AVANT commit : soit cabler la vraie
   duree worker, soit early-return `tokens` si `gen_ms == 0` (= « inconnu », pas de
   clamp) + dette tracee + test du chemin worker. Le release build en cours ne couvre
   pas ce trou (c'est un trou semantique, pas de compilation).
2. **Acceptance LIVE differee** : le comportement prod du sanity-bound (et du dashboard)
   n'est pas valide cross-machine dans cette phase ; a verifier a l'acceptance G.
3. **Residuel Sybil forge-coherente** (assume M, §15.2/§15.3) : non couvert par le
   sanity-bound, honnetement documente.
4. **Commit body** : declarer +10 Rust / +1 Vitest, reconcilier §7.4 1789->1799,
   et mentionner le choix `refetchInterval` assume. (Le P1 fix a ajoute le test
   `sanity_bound_preserves_honest_large_credit`, d'ou +10 et non +9.)

## Codex reconciliation

Codex GPT5.5 (`codex exec`, `sprint76_phase_e_codex_review.md`, output brut) lance
APRES la resolution du P1 et la re-verification des suites. Verdict : **14 CONFIRME
/ 0 GAP / 0 PARTIEL** (9 livrables + 5 invariants). Codex a verifie en particulier :
- L1 sanity_bounded_tokens + TOKENS_PER_MS_CEILING ; L2 credit() clamp avant
  log_utility cable aux 2 sites prod ; **L3 le worker mesure la vraie duree**
  (`runtime.rs:1102-1111` Instant autour de generate, `generation_time_ms: 0` en dur
  SUPPRIME) — le P1 est confirme RESOLU par Codex ;
- L4 get_contributor_summary reutilise effective_score ; L5 get_worker_entries +
  index pre-existant (0 M20) ; L6 route sous authed_routes ; L7 front Zod .strict() +
  3 metriques ; L8 THREAT_MODEL §15.3 + PATTERNS §P61 ; L9 tests (E2E
  `generation_time_ms >= 1` via `with_delay_ms`) ;
- 5 invariants CONFIRMES : kudos non-monetaire, validator INCHANGE, 0 bump wire,
  self-view per-node, sanity-bound non sur-vendu anti-Sybil.

0 GAP -> aucune correction supplementaire requise, pas de re-boucle. Review promue
PASS. Sequence respectee : preflight PLAN-ADAPT -> code -> review CONCERN (1 P1) ->
fix root-cause -> suites vertes 1799/397 -> review PASS-PENDING -> Codex 14/0/0 ->
review PASS -> commit.
