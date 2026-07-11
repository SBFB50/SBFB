# Review S81 Phase J — Inference reelle cablee sur `sbfb/shard/1` + run live + T2

## Verdict: PASS

Verdict PASS effectif : **P1 fingerprint CORRIGÉ + Codex 2e passe le confirme fermé ;
P1 manifeste-binding ACCEPTÉ/DIFFÉRÉ Phase K par décision PO explicite (2026-07-10)** —
ce n'est donc PAS « 0 P1 survivant » mais « 0 P0/P1 bloquant : le seul P1 restant est
un carry PO acté vers Phase K ». Le P1 que la mitigation blake3 avait introduit
(`std::fs::read` 16 Go → OOM tail 8 Go, Codex 2e passe) est RÉSOLU : lecture retirée,
digest vérifié out-of-band via `b3sum`. Suites dual-platform VERTES (nextest Win
2084/2084, Docker `sbfb-ci` 2088/2088, fmt 0, clippy -D warnings 0). Voir §Codex
reconciliation + §Suites §7.4.

Aucun P0/P1 survivant sur les 9 dimensions (correctness core-rs+worker-core,
correctness daemon, tests semantiques, securite/threat-model, scope/Day-0/PO,
research grounding, livrables/patterns/frontieres, run-live-vs-evidence,
CLI/UX operateur). Le cablage de l'inference reelle (codecs applicatifs
`ShardStepRequest/Reply` dans des frames OPAQUES inchangees, `forward_hidden_sample_last`
greedy deterministe + `ShardStageForwarder` role-aware, `drive_decode_loop`
autoregressif avec SI-9 par-step + churn resume-from-cache stateless + RunProof
driver-signee) est fonctionnellement CORRECT et coherent avec les preuves live.
Invariants de phase tenus et RE-VERIFIES par plusieurs dimensions : **0 bump wire**
(`SHARD_ALPN=sbfb/shard/1`, `SHARD_PLAN/RUN_PROOF/COMPUTE_GROUP_FORMAT_VERSION`
tous a 1, `SHARD_STEP_PAYLOAD_V` = app-guard LEGITIME nomme « NOT a wire
*_FORMAT_VERSION » a l'interieur d'une frame opaque byte-identique S77, pattern
feed raw-op de la pre-launch policy) ; **0 dep externe** (diff Cargo.toml =
`[features]` cuda/metal seulement, Cargo.lock INTACT) ; whitelist exact-keys
SI-3/SI-4 re-testee avec `tokens` ; duress-gate intact sur generate ; chemin echo
`model_digest=0` reste byte-identique S77 ; hygiene artefact T2 committe conforme
S3-4 (aucun prefixe pubkey membre, run_proof = prefixe signature DRIVER public).

**PASS-PENDING = review OK, gate Codex non joue.** Le verdict passe a PASS apres
Codex GPT-5.6 Sol reasoning max CLEAN (ou P2/P3 documentes) + confirmation verte
des blocs dual-platform (nextest workspace Win + Docker `sbfb-ci`) actuellement EN
COURS cote main-thread au moment de la review.

8 P2 + 12 P3 retenus, tous NON bloquants et a documenter au commit body (dont les
5 dettes explicitement dues au wrap-up Phase K : docs-contract des 2 nouvelles
frontieres, index frontieres loopback, cold/warm ttft, correction directe non
machine-asserte). 0 fix code exige avant commit. 13 nits ergonomiques laisses en
transparence plus bas.

## Synthese executive

Phase J execute l'arbitrage PO Option B (preflight DESIGN-CONFLICT resolu Gate 0,
`sprint81_phase_j_preflight.md §Arbitrage:387-395`) : le scope-cut « 0 Rust
(acceptance live) » du plan §J gele est SUPERSEDE par un delta Rust substantiel.
Le diff = 13 fichiers tracked + 3 untracked (`sprint81_phase_j_preflight.md`,
`sprint81_phase_j_review.md`, `sprint81_t2_j_shard_inference.json` ; le
`sprint81_phase_j_codex_review.md` brut s'ajoute au commit). Le cycle de decode reel est une composition
pure des primitives S77 : marche HUB pilotee-dialer (le tenseur frontiere
retraverse le driver a chaque frontiere), fenetres de couches contigues, walk
sequentiel — pipeline-parallel exclusif + STAR/HUB figes respectes.

Preuves live 2026-07-10 (post-flip iroh 1.0.1) : session `s81-j-live`,
readiness-barrier rtt 14ms, CodeLlama-34B 16,3 Go (blake3 `67d84c04…`, sha256
byte-identique 2 machines) eclate 5080-CUDA [0,37) head + Mac-M2-8GB-Metal [37,48)
tail, generation reelle 16 tokens greedy 2 tok/s (fenetre HUB 1-3 predite S1a-2),
2 runs byte-identiques, harness officiel PASS premier run. Le run est HONNETE et
l'artefact AUTHENTIQUE (raw inaltere 11 cles ordre `emit_artifact`, `\\n`
double-echappe signature du pipeline sed->json.dumps impossible a forger a la
main, LAN explicitement label « LAN PASS » pas WAN).

Ponderation adversariale des 9 dimensions : **0 P0, 0 P1, 8 P2 CONFIRMES, 12 P3
CONFIRMES, 13 nits**. Zero finding refute, zero downgrade (tous les verdicts
adversariaux : `refuted_ids=[]`, `downgraded=[]`). Aucune dimension n'a exige de
verif adversariale supplementaire (pas de P0/P1 a challenger).

## Dimension 1a — Correctness core-rs + worker-core (diff ligne par ligne)

Cablage CORRECT et coherent avec le binding vendored `vendor/llama-cpp-2`. Points
verifies OK (non-findings) : tie-break greedy `if v > best_v` gagne le PLUS PETIT
index vocab (claim « lowest vocab index » exacte) ; `ShardBackendForwarder` (mid
pur) byte-equivalent apres extraction `le_bytes_to_f32s`/`f32s_to_le_bytes` ;
dispatch role-aware derive de la `ShardWindow` VALIDEE jamais des octets,
single-shard rejete loud des deux cotes ; `hidden_token_count` couvre
`n_embd==0`/`len==0`/non-multiple -> `n_tokens-1` ne peut pas underflow ; codecs
JSON roundtrip + `deny_unknown_fields` + version-guard `v` ; toploc =
`embeddings_ith(last)` post-norm conforme au contrat Phase G ; semantics
is_first/is_last + EOS-piece-vide corrects ; drift snapshots coherent avec
`tokens: Option<u64>`.

- **1A-1 (P2)** — Le garde `if logits.is_empty()` de `forward_hidden_sample_last`
  est du CODE MORT et son diagnostic est trompeur. `get_logits_ith` (vendored)
  retourne TOUJOURS `n_vocab` (>0), donc `.is_empty()` est inatteignable et le
  message « last shard produced no logits (lm_head missing…) » ne peut jamais
  s'afficher. Le vrai scenario « lm_head absent » serait un panic (assert) ou de
  l'UB : contrairement a `get_logits` (context.rs:239) qui `assert!(!data.is_null())`,
  `get_logits_ith` NE null-check PAS `data` avant `from_raw_parts`. Le run live
  passe (is_last => lm_head resident), donc pas un bug vivant, mais faux sentiment
  de robustesse. Evidence : `crates/nexus-worker-core/src/llm/shard.rs:560-566` ;
  `vendor/llama-cpp-2/src/context.rs:280-297`.
- **1A-2 (nit)** — Rejet fp32-mal-route par `decode` repose sur « les octets fp32
  ne parsent pas en JSON » = probabiliste. Adequatement mitige (dispatch de role
  DETERMINISTE derive de la ShardWindow validee + version-guard + deny_unknown) ;
  commentaire a nuancer. Evidence : `crates/nexus-core-rs/src/shard.rs` (decode).
- **1A-3 (nit)** — Argmax sur logits NaN : un NaN est silencieusement ignore ;
  tous-NaN renvoie token 0 sans signal. Acceptable (modele casse != input adverse),
  boucle deterministe + sure memoire. Evidence : `llm/shard.rs:567-574`.

## Dimension 1b — Correctness daemon (`nexus-shell-daemon`)

`drive_decode_loop` fonctionnellement correct : ordre replay-insert-avant-dispatch
OK (l.1278 avant l.1315), fallback consomme exactement une fois (`st.fallback.take`
l.1328), participants = executeurs reels dedupliques, EOS/max_new clamp corrects,
fingerprint hex parse defensivement, dispatch `model_digest` coherent avec la
detection `transport_only` (manifest==zeros), pas de div/0 ni overflow. Le fix
clap `mount_config` est une VRAIE collision (arg global `config` id=`config` vs
positionnel Mount id=`config`) correctement resolue.

- **J1b-1 (P2)** — Le teardown parque les liens persistants sans jamais appeler
  `send.finish()`, donc l'accept loop du worker se termine par une erreur (stream
  reset) au lieu du FIN propre que le code documente et que `drive_hop` honore
  (l.794 `send.finish().ok()`). Aucune perte de donnees (toutes les replies lues en
  step_hop avant teardown), mais chaque session de decode genere une AcceptError
  cote worker et le contrat FIN documente n'est pas tenu. Evidence :
  `shard_session.rs:1398-1402` vs `:794` + accept loop `nexus-core-rs/src/shard.rs:314-324`.
- **J1b-2 (P3)** — `RunMetrics.p95_token_latency_ms` = MOYENNE arithmetique
  (`decode_ms/tokens`), pas un 95e percentile. Comme le cout par step CROIT
  (recompute stateless quadratique F2), la vraie p95 est superieure : le champ signe
  dans le RunProof sous-estime la latence de queue. Friction avec la discipline
  honnetete. Evidence : `shard_session.rs:1419` vs chemin transport `:1165`.
- **J1b-3 (P3)** — `participants` du chemin decode n'est pas borne a
  `RUN_PROOF_MAX_PARTICIPANTS=256`. Un primaire ayant execute >=1 step PUIS son
  fallback comptent tous deux : un plan large (jusqu'a `SHARD_PLAN_MAX_ASSIGNMENTS=256`)
  avec churn generalise peut produire ~2*N > 256 -> `check_run_proof_caps` Err ->
  generation reussie jetee « run proof sign failed ». Edge extreme (fan-out realiste
  3-5), divergence latente reelle. Evidence : `shard_session.rs:1367-1369`/`:1425-1432`
  + `shard_plan.rs:584-589`/`:88`/`:92`.
- **J1b-4 (nit)** — Connexion stalled fermee (`hop-deadline`) PUIS re-parquee dans
  `used` -> double close en teardown (`done`). iroh close idempotent -> inoffensif,
  mais diverge du chemin transport. Evidence : `shard_session.rs:1322-1324` vs `:1099`.
- **J1b-5 (nit)** — `replay.insert(a.layer_start, frame.clone())` clone l'input de
  stage a CHAQUE step pour CHAQUE stage alors que le cache n'est consomme que sur le
  chemin churn (rare). Surcout d'allocation borne mais evitable. Evidence :
  `shard_session.rs:1278` consomme seulement `:1356-1359`.

## Dimension 2 — Tests semantiques (delta + non-tautologie)

VERDICT = PASS. Delta REEL = **+3 daemon + 2 core-rs + 1 gguf-ignored**,
EXACTEMENT l'annonce ; aucun autre `#[test]` ajoute ; le gguf feature-gated +
`#[ignore]` n'inflate PAS le compte nextest standard. Non-tautologie confirmee :
les 3 tests decode roulent la VRAIE boucle sur QUIC loopback (`shard_rig` boot de
noeuds iroh reels servant `SHARD_ALPN`, les fakes ne remplacent que le compute LLM
pas le transport). `reroutes_mid_decode` prouve le replay stateless EXACT
(`result_text` IDENTIQUE au run sans churn, pas juste « complete »). Le virage
`model_digest` [1u8;32]->[0u8;32] dans les tests echo est NECESSAIRE et
non-affaiblissant (0 assertion sur [1u8;32] : grep model_digest+assert = 0 hit).
gguf = double discipline (`#[ignore]` + garde runtime `gguf_path()`). Codecs
core-rs : cross-rejects couverts ; wiring `max_tokens` verifie bout-en-bout.

- **J-D2-1 (P3)** — Asymetrie de couverture des rejets croises :
  `step_reply_roundtrips_and_rejects_garbage` teste fp32->reply + v-mismatch mais
  N'assertait PAS le rejet inverse (une charge `ShardStepRequest` decodee comme
  `ShardStepReply`), alors que le test symetrique `step_request_` teste bien
  reply->request. Rejet garanti STRUCTURELLEMENT (deny_unknown_fields + champs requis
  disjoints) donc SUR mais non-atteste ; la tache demandait « couvrir les rejets
  croises ». Evidence : `crates/nexus-core-rs/src/shard.rs:561`.
- **J-D2-2 (nit)** — Les bornes du clamp `max_new_tokens.clamp(1, MAX_NEW_TOKENS_CAP)`
  ne sont pas testees (seule la valeur 4 dans les bornes) ; ecretage sup (>256->256)
  et inf (0->1) non exerces. Risque faible (clamp std). Evidence :
  `shard_session.rs:1266`.

## Dimension 3 — Securite + threat model (SI-* / DoS / injection / fuite identite)

Chemin d'inference reelle globalement propre sur les 7 points interroges : l'erreur
de `ShardStepReply::decode` cote driver EST passee par `sanitize_diagnostic`
(redaction hex>=32 + strip control-chars + cap 240) ; `tokens` = agregat u64,
whitelist re-testee ; `MAX_SHARD_FRAME_BYTES=256 MiB` applique par read_frame ;
duress-gate intact + `max_tokens` clampe a `MAX_NEW_TOKENS_CAP=256` ; `toploc_hex`
parse strict (len==64 + hex::decode + try_from, sinon zeros) ; `x-sbfb-token` vit
dans le header AUTH jamais dans l'artefact. Aucune nouvelle classe de menace ni
nouveau DOMAIN_*/ALPN.

- **D3-1 (P2)** — `result_text` accumule des `piece` attaquant-controles (tail
  byzantin admis) sans borne par-piece ni borne totale : DoS memoire sur le driver.
  La seule borne est le PRODUIT `MAX_NEW_TOKENS_CAP(256)` x taille_piece(<=256 MiB)
  ≈ 64 GiB, que le commentaire de surete presente A TORT comme suffisant. Un tail
  byzantin (adversaire SI-9 deja modelise) renvoie une reply de 256 MiB ->
  `result_text` ~256 MiB retenu + amplifie par re-clone+serialise a CHAQUE
  GET /result. Fix suggere : cap par-piece (~4-16 KiB) OU cap cumulatif nomme
  `MAX_RESULT_TEXT_BYTES` + corriger le commentaire. Pre-launch + groupe prive +
  rig operateur -> P2 defense-en-profondeur. Evidence : `shard_session.rs:1385`
  (push_str sans cap) + `shard.rs:85` + commentaire surestimant `:145-148`.
- **D3-2 (P3)** — `reply.piece` (bytes attaquant-controles) concatene dans
  `result_text` SANS normalisation/control-chars (contrairement au chemin d'erreur
  qui strippe), puis projete brut dans /result + copie dans `diagnosis`. L'echappement
  JSON serde empeche l'injection structurelle, mais un piece a guillemets/backslashes
  pourrait perturber l'extraction sed du harness (robustesse harness, pas vuln
  daemon). Evidence : `shard_session.rs:1385` vs `sanitize_diagnostic` `:1378`.
- **D3-3 (nit)** — La premiere shard etend la sequence avec `req.generated`
  (Vec<i32> driver) sans valider que les ids sont dans la plage de vocabulaire.
  `forward_tokens` errorera vraisemblablement plutot que paniquer ; driver = operateur
  authed loopback -> surface tres faible. Evidence : `llm/shard.rs:~254`/`:407`.

## Dimension 4 — Scope + Day-0 + PO

DIMENSION 4 = PASS. Aucune violation P0/P1. Verifs positives : 0 bump wire HONNETE
(`SHARD_STEP_PAYLOAD_V` = app-guard legitime, pas un bump deguise) ; 0 dep externe
(Cargo.lock 0 diff) ; arbitrage PO Option B documente au bon endroit (preflight
§Arbitrage, plan §J gele avec supersede trace = convention SBFB) ; pipeline-parallel
exclusif + STAR/HUB respectes ; pre-launch policy OK (drift snapshot regenere
intentionnellement + whitelist mise a jour + champ loopback additif `tokens`) ;
deviation « serve identity != worker enrollment » coherente Phase I. BONUS verifie :
la claim operateur « same placement inputs => same plan » est SOUND (mount_session
ET le Plan CLI passent tous deux `RttMatrix::new()` vide ; sampling_key ne depend
que de (session_id, pubkey)).

- **D4-1 (P2)** — La ligne FIGEE du plan §J « Delta tests attendu : 0 Rust
  (acceptance live) » est desormais FAUSSE — Phase J livre un delta Rust substantiel.
  Le supersede EST correctement documente dans le preflight §Arbitrage (emplacement
  canonique), donc PAS un defaut du working tree. MAIS le commit body doit posseder
  explicitement le vrai delta cumule et citer l'arbitrage PO Option B — jamais
  recopier « 0 Rust ». A verifier a la redaction du body. Evidence :
  `.planning/active/sprint81_plan.md:395` vs `sprint81_phase_j_preflight.md:387-395`.
- **D4-2 (P3)** — `drive_decode_loop` reinjecte un prefixe 16-hex (64 bits) de la
  pubkey d'un membre du groupe PRIVE dans ses chaines d'erreur de churn
  (`worker_hex`/`fb_hex`), NON sanitize (sanitize_diagnostic redige les 64-hex mais
  preserve les troncatures 16-hex). Ces Err remontent au `diagnosis` de GET /result
  et donc dans un artefact T2 sur un BLOCK. Cross-dim (deja carry R-J-7/S3-4) ;
  l'artefact T2 committe ACTUEL est un PASS et est propre. Le code NEUF de J perpetue
  le pattern. Evidence : `shard_session.rs:1327`/`:1335` (mirror pre-existant `:1102`/`:1109`).
- **D4-3 (nit)** — `SHARD_STEP_PAYLOAD_V` + `ShardStepRequest/Reply` constituent un
  NOUVEAU contrat wire applicatif versionne sur la frontiere P2P `sbfb/shard/1`
  (driver <-> head/tail = runtimes daemon distincts). Legitimement « 0 bump wire »
  mais NOUVELLE frontiere que la cloture docs-contract Phase K (Track K / DoD (d))
  doit indexer, pas seulement une frontiere loopback. Evidence :
  `crates/nexus-core-rs/src/shard.rs:379`/`:383-472`.

## Dimension 5 — Research grounding (vs preflight spec)

L'implementation honore le SPEC preflight sur l'essentiel : les 3 tells
anti-faux-vert R-J-4 cables (result_text!=prompt, tokens>=2, libelles
« verified/per-shard » remplaces par « driver-signed/HUB baseline ») ; debit
2 tok/s = PASS legitime dans la fenetre HUB 1-3 tok/s juge contre la baseline HUB
(jamais Petals/single-machine) ; determinisme S1a-8 prouve hermetiquement ET live ;
semantique churn I cablee+testee (voie live vs hermetique honnetement disjointe) ;
RunProof DRIVER-only S2-J-07 ; artefact T2 enveloppe/tracke S4-5 ; T2 committe ne
fuit AUCUN prefixe pubkey membre S3-4.

- **J-D5-1 (P2)** — Le prerequis DUR preflight S6-4/S1a-7 « le harness DOIT asserter
  un chemin DIRECT avant PASS/BLOCK » n'est PAS honore : ni le daemon ni le harness
  n'interrogent le type de connexion iroh (direct vs relais). Seul le RTT est
  echantillonne (`conn.rtt(PathId::ZERO)`) et le gate rtt<80ms passe identiquement
  pour un direct-LAN et un relais Helsinki rapide. La claim T2 « dial direct » repose
  uniquement sur rtt=14ms — superieur aux 1-5ms predits S6-3 pour du direct-LAN, et
  preleve a la readiness-barrier (fenetre transitoire relais->direct que S6-4 met en
  garde). La directness n'est PAS machine-verifiee ; le T2 sur-affirme. Fix : soit
  interroger `remote_conn_type`/conn_type et asserter Direct au readiness-barrier,
  soit adoucir la claim T2 (« type de chemin NON asserte par le harness ») + carry.
  NE PAS laisser « dial direct » non-qualifie dans l'artefact committe. Evidence :
  `crates/nexus-core-rs/src/shard.rs:179` + `shard_session.rs:648` +
  `sprint81_t2_j_shard_inference.json:30`.
- **J-D5-2 (P3)** — S1a-6 exige « cold vs warm reportes separement ». Le warm-up
  Metal a bien ete fait (2 pre-runs compilent les shaders), mais l'artefact T2
  committe ne reporte QUE le ttft warm (`ttft_s=0`) ; le ttft cold (=1 au premier run
  operateur) est absent. Le split cold/warm n'apparait pas dans le livrable committe.
  Evidence : `sprint81_t2_j_shard_inference.json:26`/`:27` + `shard_session.rs:44`.
- **J-D5-3 (nit)** — Le run PASS committe (prompt harness « In one word… ») produit
  une regurgitation repetitive du prompt, pas « Paris ». La preuve de CORRECTION
  repose entierement sur un pre-run operateur SEPARE a prompt DIFFERENT
  (« The capital of France is » -> « Paris »). Honnetement disclose dans
  `determinism_note`, mais le result_text du run PASS est une evidence de correction
  faible en standalone (passe le tell anti-echo via tokens=16, n'evidence pas une
  reponse correcte). Evidence : `sprint81_t2_j_shard_inference.json:22` vs `:27`.

## Dimension 6 — Livrables / patterns / frontieres (test-acteur §6.12)

PROPRE — aucun P0/P1. SPDX present sur les 4 fichiers .rs touches ; 0 promesse
forward-looking dans les commentaires code (grep TODO/FIXME/Phase K/will = vide) ;
franglais respecte ; 0 bump wire verifie ; la frontiere loopback /result gagne
`tokens` avec contrat-machine tenu (snapshot regenere response+view + whitelist
exact-keys + doc frontier exacte) ; harness = consommateur relibelle HONNETEMENT
(« driver-signed RunProof », tells anti-echo cables et BLOQUANT correctement le
chemin echo tokens=1 ; garde numerique TOKENS l.387 precede `-lt 2` l.389 = pas
d'erreur shell) ; hygiene artefact T2 conforme S3-4.

- **J6-1 (P2)** — Les payloads `ShardStepRequest/Reply` sont une VRAIE frontiere
  cross-processus/cross-machine (driver=daemon <-> worker `serve` sur une AUTRE
  machine, binaires build/deployes independamment) transportee dans les frames
  opaques `sbfb/shard/1`. L'etiquette INLINE est excellente (rationale 0-bump, garde
  `v`, deny_unknown_fields), MAIS `docs/protocol/SHARD_PROTOCOL_SPEC.md` et l'index
  docs-contract NE les mentionnent nulle part. Par la doctrine test-acteur §6.12
  (« une API loopback lue par un runtime distinct EST une frontiere »), a documenter.
  Non-bloquant pour CETTE phase : la cloture docs-contract est un livrable Phase K —
  a NE PAS laisser tomber. Evidence : `crates/nexus-core-rs/src/shard.rs` (defs apres
  :348) + `docs/protocol/SHARD_PROTOCOL_SPEC.md` (grep ShardStep = 0 hit).
- **J6-2 (P3)** — La frontiere loopback /result gagne `tokens` (contrat machine tenu
  a CETTE phase) ; a consigner dans la cloture docs-contract / index Phase K (nouveau
  champ sur une frontiere loopback existante). Evidence : `http.rs:2416` +
  `schemas/shard.rs:421` + snapshots `shard_session_result_*.schema.json`.
- **J6-3 (nit)** — Portee honnete du gate : le run PASS gate a produit une REPETITION
  du prompt, pas « Paris ». Les 4 gates prouvent non-echo + multi-token + RunProof
  signee + debit, PAS la correction semantique (attestee par le determinism_note =
  pre-run NON gate). Honnetement scope (le champ criterion ne revendique aucune
  correction). Evidence : `sprint81_t2_j_shard_inference.json:22` vs `:27`.
- **J6-4 (nit)** — `p95_token_latency_ms: decode_ms / tokens` = latence MOYENNE, pas
  p95 (miroir de l'echo `:1165`). Champ non expose au verdict harness ; mal-libelle
  PRE-EXISTANT propage. Evidence : `shard_session.rs:1419`.
- **J6-5 (nit)** — `shard-session plan` lit la VRAM depuis `--worker pubkey:vram`
  tandis que `mount` re-derive depuis `workers[].vram` du mount-config ; rien ne
  cross-verifie que les deux jeux coincident. Le backend/N0 attraperait un mismatch
  de couches, mais le gap ergonomique G3 n'est que partiellement ferme. Evidence :
  `main.rs` (branche Plan) + `cli.rs` (Mount JSON brut).
- **J6-6 (nit)** — `serve` clampe `n_ctx.min(MAX_SHARD_N_CTX)` silencieusement ;
  l'aide CLI ne mentionne pas le clamp. Evidence : `main.rs` + `cli.rs`.

## Dimension 7 — Claims du run live vs evidence (T2)

Run live globalement HONNETE, artefact AUTHENTIQUE ; 0 P0/P1. Reconciliations
CONFIRMEES : raw INALTERE (11 cles ordre `emit_artifact`, `\\n` double-echappe =
signature du pipeline non-forgeable) ; nombres coherents (tokens=16=MAX_TOKENS,
toks_per_s=2 UNFLOORED -> decode_ms≈8000ms -> 2 tok/s fenetre HUB, rtt=14ms LAN) ;
**ttft 0 vs 1 EXPLIQUE** (`ttft_s=ttft_ms/1000` division ENTIERE resolution 1s ;
pre-run operateur froid=warm-up Metal/CUDA -> ttft_s=1, run harness chaud ->
ttft_s=0 ; le pre-run a de facto servi de warm-up jete) ; carries EXACTS vs code
(RunProof driver-only unique, SI-9 hermetique + coupe live comptee, KV recompute
stateless 0 reutilisation) ; LAN pas WAN explicitement label ; enveloppe conforme
au frere E3.

- **J7-1 (P2)** — La preuve CORRECTION+DETERMINISME phare (« answer is CORRECT
  (Paris) », « run 1 = run 2 BYTE-IDENTICAL », « real weights, not an echo ») provient
  d'un run OPERATEUR SEPARE a prompt DIFFERENT, pas du run harness COMMITTE. Le
  result_text du PASS committe est une repetition du prompt qui ne contient AUCUNE
  reponse correcte (vraie inference base-model, passe le tell anti-echo par le \n de
  tete). Le run operateur (Paris + determinisme 2-runs) est prose-only : aucun
  artefact machine-lisible ne le corrobore, alors que l'ethos T2 est « jamais
  prose-only ». Un lecteur presse conflatera les deux. Fix : committer les 2 runs
  bruts de determinisme A COTE du T2 OU ajouter une ligne `observed` explicite que le
  run committe produit une continuation/repetition (base-model reel) et que la
  CORRECTION est portee par le run operateur separe. Evidence :
  `sprint81_t2_j_shard_inference.json:27` + `:22` + `shard_session.rs:1385`.
- **J7-2 (P3)** — `ttft_s:0` via `ttft_ms/1000` division entiere (resolution 1s) :
  l'ecart 0-vs-1 est un pur artefact de granularite + warm-up, non une mesure fine.
  « sub-second first token » est correct mais `ttft_s` a 1s de resolution est un
  indicateur faible. Fix optionnel post-commit : exposer aussi `ttft_ms`. Evidence :
  `shard_session.rs:452` + `sprint81_t2_j_shard_inference.json:18`.
- **J7-3 (P3)** — `observed` affirme « worker_drop_count incremented… » mais
  `worker_drop_count` N'EST PAS un champ de l'artefact brut committe (emit_artifact
  l'omet ; il ne vit que sur la vue /result). Seul `last_response:{found,dropped}`
  evidence la coupe dans les fichiers committes. Reformuler `observed` pour ne
  s'appuyer que sur `last_response`. Evidence :
  `sprint81_t2_j_shard_inference.json:26` + `b3_shard_pipeline.sh:37-39`/`:23`.

## Dimension 8 — CLI/UX operateur + robustesse

Cablage CLI majoritairement propre : `serve --model` sans feature bail avec message
clair (main.rs:335-342) ; `--layer-start/--layer-end` portent `requires="model"` ;
`plan` parse `pubkey:vram` via `parse_pubkey_hex` valide (courbe Ed25519, 32 octets)
+ sort `--layer-end 0` correct pour le tail ; le rename `config`->`mount_config`
(cli.rs:273) est un VRAI fix root-cause d'une collision d'id clap latente
(`Cli::command().debug_assert()` NE detecte PAS les fusions global-vs-subcommand-positional,
d'ou le ship Phase I + surface live), AUCUNE autre collision.

- **J8-1 (P2)** — `serve --model` transmet la fenetre `--layer-start/--layer-end`
  operateur directement a `ShardBackend::load` SANS pre-validation, alors que `load`
  est explicitement documente (shard.rs:256-263 « # Aborts ») pour declencher un
  `GGML_ASSERT` natif qui ABORTE le process sur fenetre hors-borne, AVANT le check
  `ShardWindow` recuperable. Le doc dit « The scheduler and the F2 claim gate MUST
  pre-validate the window » — or `serve` est un nouvel appelant qui saute cette
  pre-validation. Un typo courant (`--layer-start 40 --layer-end 20`, ou
  `--layer-end 999`) fait crasher dur (SIGABRT/STATUS_) au lieu d'un `anyhow` propre.
  Le cas trivialement rattrapable `end != 0 && start >= end` n'est pas garde. Fix :
  precondition cheap `if end != 0 && start >= end { bail! }` + idealement probe
  metadonnees-only du n_layer. Operateur authed loopback -> P2. Evidence :
  `main.rs:305` + `crates/nexus-worker-core/src/llm/shard.rs:256-263`.
- **J8-2 (P3)** — Le commentaire serve affirme « probe the model's layer count first
  so `end==0` resolves… » mais AUCUN probe n'est fait : `is_first`/`is_last` derives
  des valeurs CLI BRUTES, la resolution `end==0 => n_layer` se produit DANS
  `ShardBackend::load`. Commentaire trompeur — le probe qu'il decrit est exactement
  le remede de J8-1. Evidence : `main.rs:290-292`.
- **J8-3 (P3)** — Le fix rename est CORRECT mais il n'existe AUCUN test
  `try_parse_from` pour un sous-commande `shard-session` : le bug exact qui a mordu
  live n'a aucun garde-fou de regression. Fix : test asserer `cli.config` ET
  `mount_config` distincts. Evidence : `cli.rs:273` + `cli.rs:530-536` (seul test CLI).
- **J8-4 (P3)** — `check_stale_or_bail` identifie le pid vivant par le NOM d'exe
  (`EXPECTED_PROCESS_NAME=nexus-shell-daemon`) : un daemon HEAD lance depuis une COPIE
  RENOMMEE lit comme pid mort -> `mount/generate/status/result/drop-shard` bail. Pre-
  existant (S49) mais Phase J fait des binaires per-machine renommes/feature-buildes
  la norme operateur. `serve` non affecte (boot son propre node). Fix = ligne runbook
  (pas code). Evidence : `main.rs:173-186` + `registry.rs:366`.
- **J8-5 (nit)** — Le tell anti-echo `if [ "$TOKENS" -lt 2 ]` BLOQUE tout decode
  reel produisant exactement 1 token (ex. « Paris » puis EOS). Le run live a produit
  16 tokens donc n'a pas mordu, mais le couplage « vraie inference <=> >=2 tokens »
  est fragile. Defensible pour ce benchmark. Evidence : `b3_shard_pipeline.sh:389`.

## Refutes / downgrades (transparence adversariale)

Les 9 verdicts adversariaux retournent tous `refuted_ids=[]`, `confirmed_ids=[]`,
`downgraded=[]` avec la note « aucun P0/P1 — pas de verif adversariale requise ».
**Aucun finding refute, aucun downgrade applique** : les 33 findings (8 P2 + 12 P3 +
13 nits) sont tous CONFIRMES tels que reportes par les dimensions. Contrairement a
Phase I (1 P1 dual-reporte + boucle de correction), Phase J n'a exige AUCUNE passe
adversariale de challenge ni aucune boucle de correction pre-commit — la surface est
CLEAN cote P0/P1 des la premiere passe.

## Findings P2/P3 retenus (a documenter au commit body)

**P2 (8, defense-en-profondeur / honnetete / dette Phase K — 0 bloquant) :**
- **1A-1** — garde `logits.is_empty()` code mort + diagnostic trompeur (`get_logits_ith`
  ne null-check pas `data` avant `from_raw_parts` — invariant honnete a substituer).
- **J1b-1** — teardown sans `send.finish()` sur liens persistants -> AcceptError worker
  par session, contrat FIN documente non tenu.
- **D3-1** — `result_text` non borne (DoS memoire ~64 GiB tail byzantin) + commentaire
  de surete surestimant la borne.
- **D4-1** — commit body DOIT posseder le vrai delta Rust cumule + citer arbitrage PO
  Option B, jamais recopier « 0 Rust » du plan §J gele.
- **J-D5-1** — chemin DIRECT non machine-asserte (prerequis DUR S6-4/S1a-7) ; claim T2
  « dial direct » a qualifier ou carry.
- **J6-1** — frontiere `ShardStepRequest/Reply` sur `sbfb/shard/1` a indexer dans
  SHARD_PROTOCOL_SPEC + docs-contract Phase K (hors gouvernance *_FORMAT_VERSION).
- **J7-1** — preuve correction/determinisme (« Paris ») portee par un run operateur
  separe prose-only ; a corroborer par artefact OU expliciter dans `observed`.
- **J8-1** — `serve --model` sans pre-validation de fenetre -> abort process natif sur
  typo (garde `end!=0 && start>=end` + probe n_layer recommandes).

**P3 (12, doc-honnetete / couverture / robustesse) :**
- **J1b-2** — `p95_token_latency_ms` = moyenne, pas p95 (sous-estime la latence de queue).
- **J1b-3** — `participants` decode non borne a `RUN_PROOF_MAX_PARTICIPANTS` (edge churn
  generalise plan large -> sign Err).
- **J-D2-1** — rejet croise request-comme-reply non atteste (sur structurellement).
- **D3-2** — `reply.piece` non normalise dans `result_text` (robustesse extraction harness).
- **D4-2** — prefixe pubkey membre 16-hex non sanitize dans les Err de churn (carry R-J-7).
- **J-D5-2** — cold vs warm ttft non separes dans l'artefact T2 (S1a-6).
- **J6-2** — champ `tokens` sur frontiere /result a consigner docs-contract Phase K.
- **J7-2** — `ttft_s` division entiere (resolution 1s) indicateur faible ; exposer `ttft_ms`.
- **J7-3** — `observed` cite `worker_drop_count` absent de l'artefact brut (s'appuyer sur
  `last_response`).
- **J8-2** — commentaire serve « probe layer count » trompeur (aucun probe reel).
- **J8-3** — 0 test de regression `try_parse_from` pour le bug clap `mount_config` vecu live.
- **J8-4** — stale-check par nom d'exe -> binaire renomme vu comme pid mort (ligne runbook).

**nits (13, non a documenter obligatoirement) :** 1A-2, 1A-3, J1b-4, J1b-5, J-D2-2,
D3-3, D4-3, J-D5-3, J6-3, J6-4, J6-5, J6-6, J8-5.

## Suites §7.4

Etat honnete au moment de la review (verdicts finaux consignes au commit) :

- **Shard tests** : 66/66 + 3 nouveaux decode (EOS / max_tokens / reroute mid-decode).
- **fmt** : 0 ; **clippy -D warnings** : 0 (+ fix warning deprecated `Special` sous
  feature Metal).
- **web** : 412/412 ; **operator** : 201/201 ; **doctests** OK ; **release build** OK.
- **nextest workspace Win** : EN COURS (re-run apres crash transitoire rustc
  `STATUS_STACK_BUFFER_OVERRUN` sous contention — pas un echec de tests).
- **Docker `sbfb-ci`** : EN COURS.

RESERVE PASS-PENDING : les deux blocs dual-platform (nextest workspace Win + Docker
`sbfb-ci`) DOIVENT etre confirmes verts AVANT le commit ; le gate Codex doit etre
joue et CLEAN (ou P2/P3 documentes).

## Fixes in-phase (post-review, avant Codex)

Bien que « 0 fix code exige », 8 findings avaient un fix CHEAP root-cause —
appliques avant le gate Codex (pattern Phase H), suites §7.4 re-jouees apres :

- **J1b-1 (P2) APPLIQUE** — teardown decode `link.send.finish().ok()` avant de
  parquer la connexion : le worker termine sur le FIN propre documente, plus
  d'AcceptError par session saine.
- **D3-1 (P2) APPLIQUE** — cap cumulatif nomme `MAX_RESULT_TEXT_BYTES = 64 KiB`
  (regle §6.9) : une piece byzantine sur-dimensionnee fait echouer PROPREMENT le
  drive (jamais une troncature silencieuse) ; commentaire de surete corrige.
- **1A-1 (P2) APPLIQUE** — garde mort `logits.is_empty()` retire, remplace par
  le commentaire honnete (le binding retourne toujours n_vocab ; lm_head absent
  = assert natif du fork, pas un slice vide).
- **J8-1 (P2) APPLIQUE** — precondition cheap `end != 0 && start >= end -> bail`
  dans `serve --model` (le typo operateur ne SIGABRT plus) ; le check model-bound
  reste l'assert natif (pas de probe metadata expose par le binding).
- **J8-2 (P3) APPLIQUE** — commentaire « probe the model's layer count » trompeur
  remplace par la description reelle (resolution end==0 DANS load).
- **J8-3 (P3) APPLIQUE** — test de regression clap
  `shard_mount_positional_never_shadows_global_config` (+1 test) : le bug de
  collision vecu live a desormais son garde-fou.
- **J-D2-1 (P3) APPLIQUE** — assert croise request-comme-reply ajoute au test
  codec core-rs (symetrie complete des rejets croises).
- **J7-1 + J7-3 + J-D5-1 + J-D5-2 (P2/P3) APPLIQUES (artefact T2)** —
  `determinism_runs` machine-comparable (2 result_text verbatim + note
  correction « Paris » + cold/warm ttft S1a-6), `observed` reformule (le PASS
  gate = continuation repetitive, correction portee par determinism_runs ;
  drop evince par last_response, pas worker_drop_count), claim « dial direct »
  QUALIFIEE (path-type NON machine-asserte, carry Phase K conn_type).
- **J8-4 (P3) APPLIQUE (doc)** — note operateur stale-check-par-nom-d'exe dans
  `rig.local.env.example`.

Non appliques (routes commit body / Phase K) : J1b-2 (p95=moyenne, miroir
pre-existant), J1b-3 (cap participants, edge extreme), D3-2 (piece non
normalise — le cap D3-1 borne le risque), D4-2 (prefixe 16-hex churn, carry
R-J-7), J6-1/J6-2/D4-3 (index docs-contract Phase K), J7-2 (ttft_ms DTO,
Phase K), nits.

Delta tests post-fixes : +2 (test clap cli.rs + assert croise dans test
existant = +1 comptable nextest ; total attendu Win 2076->2082).

## Codex reconciliation

Rapport brut : `sprint81_phase_j_codex_review.md` (`codex exec -m gpt-5.6-sol
-c model_reasoning_effort=max`, output non reecrit). **L'executeur shell local de
Codex etait defaillant dans cet environnement** (git/lecture-fichier suspendus,
3 tentatives « NON EVALUABLE ») ; audit obtenu en 4e passe en COLLANT le diff +
les 3 fichiers untracked INLINE dans le prompt (read-only), contournant
l'executeur casse. Verdict Codex : **GAP — 2 P1 + P2/P3**. Reconciliation :

- **P1 Codex #2 — fingerprint final potentiellement perime : CORRIGE.** Le
  fingerprint est desormais re-assigne a CHAQUE step via `parse_toploc_hex`
  (total, zeros si vide/invalide) → le DERNIER reply decide toujours ce que la
  RunProof signe. +2 tests (`parse_toploc_hex_defaults_to_zeros...` unitaire +
  `decode_loop_signs_last_step_fingerprint_even_when_blank` integration : un
  dernier toploc VIDE signe zeros, jamais le `0xab` de l'avant-dernier step).
- **P1 Codex #1 — stage/model/window non lies au manifeste : CARRY DOCUMENTE →
  Phase K (decision PO 2026-07-10).** Un serve (primaire/fallback) choisit son
  window/role via ses args CLI, pas via le manifeste signe ; rien n'enforce a la
  readiness que le stage distant a charge {model_digest, window, role} attendu →
  un fallback mal-fenetre pourrait produire un resultat plausible-faux signe par
  le driver. Rationale du carry : (a) NON exploitable par un tiers (membres admis
  seulement = machines de l'operateur) ; (b) design PRE-EXISTANT hors-bande
  (note Phase I « le manifeste voyage OUT OF BAND, le window est la launch
  config du worker ») que J rend load-bearing mais n'introduit pas ; (c)
  n'affecte PAS le run live prouve (bons windows depuis `plan`, aucun fallback) ;
  (d) la RunProof N'A JAMAIS pretendu attester la correction (`shard_plan.rs`
  doc : « une signature valide prouve QUI a signe, pas que le calcul est
  correct ») — meme famille que la verif per-worker N0-N3 deja differee.
  **Mitigation cheap appliquee** : `serve --model` calcule et AFFICHE le blake3
  du GGUF reellement charge (+window+role dans la banniere) → l'operateur peut
  cross-verifier {model, window, role} contre le mount `model_digest` +
  `shard-session plan` AVANT de faire confiance a la RunProof.
  **Enforcement complet (handshake capability readiness liant loaded-stage ↔
  manifeste signe) = livrable Phase K** (THREAT_MODEL §16 + audit gate S82).
- **Defauts harness Codex : CORRIGES.** (1) occurrence mensongere « per-shard
  signed RunProofs » (l.338) + « per-shard RunProof signatures » (l.361) →
  « DRIVER-signed RunProof / driver RunProof signature » ; (2) `eval`+JSON non
  echappe du body generate → build via `python3 json.dumps` (fallback rejette
  quote/backslash) + `MAX_TOKENS` valide numerique early (RIG-ABSENT sinon) ;
  (3) `LAST_RESPONSE` ecrase par la reponse drop-shard → la reponse `/result`
  brute (tokens/text/proof) est preservee dans `RESULT_RESPONSE` et re-injectee.
- **P2/P3 Codex restants : DOCUMENTES** (commit body + Phase K). Notables :
  FIN cleanup seulement sur succes (P2, miroir chemin transport) ; alloc
  transitoire 256 MiB avant caps metier (P2, borne DoS existante) ; pas de
  check NaN sur logits avant argmax (P2, modele-casse != adverse) ; validation
  CLI model-bound + `n_ctx` incomplete (P2) ; frontiere `ShardStepRequest/Reply`
  absente de la doc protocole (P2 → index docs-contract Phase K) ; `plan`
  n'imprime pas les fallback windows (P2, lie au carry P1#1) ; `p95` = moyenne
  (P3) ; cap participants churn (P3) ; artefact `determinism_runs` transcrit-main
  sans /result brut par run (P3, honnetement label) ; hash byte-identique sans
  capture par-hote (P3) ; tell `tokens>=2` rejette une generation legitime a 1
  token (P3, defensible pour ce benchmark).

Suites re-jouees apres les fixes P1 : voir §Suites §7.4 (nextest workspace Win +
Docker `sbfb-ci` + fmt/clippy). Le fichier Codex brut n'est PAS reecrit.

## Prochaine etape

Verdict PASS effectif (0 P0/P1 survivant : P1#2 corrige, P1#1 carry PO acte) →
commit atomique `feat(daemon+core): Sprint 81 Phase J — inference reelle sur
sbfb/shard/1 + run live + T2` avec body figeant le delta tests reel + les P2/P3 +
le carry P1#1. Dettes Phase K re-routees : **binding loaded-stage↔manifeste
(P1#1, enforcement)**, index frontiere step-payloads (J6-1/D4-3), champ `tokens`
/result (J6-2), assertion chemin direct (J-D5-1), cold/warm ttft (J-D5-2),
corroboration correction (J7-1), sanitize prefixe pubkey 16-hex (D4-2),
`plan` fallback windows.
