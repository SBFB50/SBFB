# Sprint 82 — Phase B — Preflight G8 (benchmarks standards sharding)

## Verdict: PLAN-ADAPT

Les deux signaux dominants — WIRE (S4) et DEPS (S1b) — **TIENNENT** : aucune
metrique standard n'exige un bump wire SBFB, aucune dependance runtime n'est
ajoutee. Il n'y a donc **pas de DESIGN-CONFLICT**. Mais le signal OSS (S1a) fait
basculer vers **PLAN-ADAPT** : la premisse du plan « llama-bench dans le fork
vendored » est **factuellement refutee sur disque**. Le code doit suivre une
approche corrigee (binaires standards construits depuis un checkout upstream
epingle + metriques fines mesurees nativement cote-hote) sans toucher aucune
Day-0 figee ni aucun invariant dur.

Repartition des scans : **3× PLAN-ADAPT** (S1b, S2, S4) + **1× EXECUTE** (S3).
Verification adversariale : le S3=EXECUTE est **valide mais borne a la surface
securite** (0 nouveau endpoint / flux d'un KIND nouveau / relation de confiance)
— il ne leve pas l'axe OSS/build, et il porte lui-meme 2 cautions (fuite de
chemin FS dans un artefact committe + acquisition supply-chain de llama-bench).
Il ne suffit donc pas a remonter l'agregat au-dessus de PLAN-ADAPT.

Ce PLAN-ADAPT est **borne et pilote par une refutation factuelle** (un premisse
d'outil faux), pas une derive de conception. Ce n'est pas un signal-meta de
design flou.

## S1a — OSS prior-art (llama-bench / perplexity / TTFT-TPOT-ITL / schema artefact)

Aucun scan S1a dedie n'a ete fourni ; les faits OSS sont reconstitues et
verifies ici (disque + sources).

- **llama-bench** est l'outil de benchmark officiel livre AVEC llama.cpp
  (upstream `tools/llama-bench`). Il emet deux debits standards :
  - `pp512` = prompt-processing 512 tokens (throughput single-forward-pass,
    **compute-bound**, tokens/s = (B×512)/median_time) ;
  - `tg128` = text-generation 128 tokens un-par-un via KV-cache
    (**memory/latency-bound**, tokens/s = (B×128)/median_decode_time).
  Source : [knightli.com — llama.cpp GPU benchmark pp512/tg128](https://knightli.com/en/2026/04/23/llama-cpp-gpu-benchmark-cuda-rocm-vulkan-scoreboard/),
  [blog.majid.info — Llama-bench](https://blog.majid.info/llama-bench/).
- **perplexity** est l'outil officiel llama.cpp (`tools/perplexity`) qui calcule
  la **PPL** sur un dataset ; **wikitext-2** est le corpus de reference standard
  pour la parite.
- **Vocabulaire TTFT / TPOT / ITL** (vLLM / MLPerf inference-serving) :
  - **TTFT** (Time To First Token) = delai jusqu'au 1er token (prefill).
  - **TPOT** (Time Per Output Token) = moyenne des inter-token apres le 1er.
  - **ITL** (Inter-Token Latency) = ecart par-token en decode streaming ;
    la moyenne des ITL d'une requete **egale** son TPOT. TPOT est request-weighted
    (latence par requete) ; ITL/p95 est le budget SLO fin.
  Sources : [Anyscale — LLM latency & throughput metrics](https://docs.anyscale.com/llm/serving/benchmarking/metrics),
  [Spheron — TTFT/ITL/P99 latency budgets 2026](https://www.spheron.network/blog/llm-inference-slo-ttft-itl-latency-budget-guide-2026/),
  [vllm#6531 — ITL vs TPOT](https://github.com/vllm-project/vllm/issues/6531).
- **REFUTATION DECISIVE (on-disk)** : ces outils standards ne sont PAS dans le
  fork vendore. `vendor/llama-cpp-sys-2/llama.cpp/tools/` = **`mtmd` seul** ;
  `examples/` **absent** (l'`ls` sort en exit 2) ; `build.rs:531-534` fixe
  `LLAMA_BUILD_TOOLS=OFF` et le commentaire `:847-849` avertit que
  `LLAMA_BUILD_TOOLS=ON` tirerait « all tools (batched-bench, quantize, etc.)
  and their CMakeLists.txt files, which are not included in the crate package ».
  Le build mtmd passe par `cc::Build`, pas par un cmake-tools complet. => **les
  binaires llama-bench/perplexity doivent venir d'AILLEURS.** C'est ce qui
  determine EXECUTE-vs-PLAN-ADAPT : non-buildables tels quels ⇒ PLAN-ADAPT.
- **Schema d'artefact** : precedent local direct = `sprint82_t2_bootseed.json`
  (frere de Phase A) et `sprint81_t2_j_shard_inference.json` — fichiers JSON de
  mesure committes a la main, verdicts fermes. Le schema versionne du benchmark
  porte sur l'artefact **hors-wire**, jamais sur un format canonical.

## S1b — Deps / build (invariant 0 dep)

Signal scan : **PLAN-ADAPT**. Invariant 0-dep **TENU**, verifie.

- `serde_json` est deja workspace-dependency dans les 12 crates
  (`crates/nexus-worker-core/Cargo.toml:68`). L'artefact JSON n'ajoute AUCUNE
  dep : il suit le pattern `b3_shard_pipeline.sh` (curl + python3 + fallback
  bash), **0 code/dep Rust** (`scripts/acceptance/b3_shard_pipeline.sh:128-190`).
- La collecte de metriques shard reutilise les routes loopback existantes
  `/api/daemon/shard-session/{id}/{generate,result}` — aucun crate ni dep
  nouvelle.
- **Cout de build llama-bench = rig-only** : CI ne build JAMAIS les features
  `llm_llama_cpp*` (`Cargo.toml:498` « CI never builds with these features » ;
  grep `.woodpecker*`/`.github` = 0). Un checkout upstream separe est un
  **build-tool**, pas une entree `[dependencies]` : **0 churn Cargo.lock**,
  `[patch.crates-io]` inchange (`Cargo.toml:501-503`).
- Un helper Rust de collecte host-side (si ajoute) n'utiliserait que
  tokio/serde/serde_json deja au lockfile.

## S2 — Decisions historiques traversees

Signal scan : **PLAN-ADAPT**. Aucune decision gelee rouverte.

- Le design sharding est GELE sur le **mecanisme** (pipeline-parallel exclusif,
  ALPN `sbfb/shard/1`, N0-N4 — `sharding_design_addendum_sota_2026-05-30.md`)
  mais **pas** sur le vocabulaire de mesure : l'addendum n'emploie NULLE PART
  `llama-bench`/`perplexity`/`wikitext`/`tg128`/`pp512`/`TPOT`/`ITL`/`MLPerf`/
  `vLLM` (grep = 0). PO-2 (benchmarks IN S82) **ajoute une couche de mesure**,
  ne re-concoit rien ⇒ extension coherente, pas de re-debat.
- **Amendement canon T3** : les repercussions memory pts 2/3 (Plan S81 Phase K,
  Roadmap v5 Phase L) sont **STALE** — S81 a ferme SANS Phase L, les benchmarks
  sont passes en S82-B. L'action canonique se reduit a **README §4** (table
  actuelle T0/T1/T2 a `README:622-626`, aucun T3). NE PAS toucher roadmap-v5/S81.
- Le harness sharding a etendre est **UNIQUE** (`b3_shard_pipeline.sh`, verdicts
  fermes PASS/BLOCK/RIG-ABSENT + artefact JSON) ; le plan Phase B **remplace
  RIG-ABSENT par BLOCK{rig}** (rig engage pour A) — ecart voulu a documenter.

## S3 — Threat model / surface securite

Signal scan : **EXECUTE** (borne securite) — retenu comme **valide dans son
perimetre** mais non-decisif sur l'axe OSS/build.

- **0 nouvelle surface d'attaque** : les routes shard-session pilotees par le
  harness sont deja duress-gatees et testees (`http.rs:6706-6795` :
  `shard_session_routes_noop_in_duress` — group/mount/generate no-op sous
  Duress) ; le data-plane `sbfb/shard/1` est deja threat-modele (THREAT_MODEL
  §16, admission `is_member` Ed25519, cap frame 256 MiB, catalogue SI-1..SI-5).
  Mesurer ce chemin ne cree ni endpoint ni flux d'un KIND nouveau ; les
  activations circulent deja en clair (SI-1 residuel ASSUME).
- **CAUTION 1 (confirmee disque)** : un artefact T2 shard committe fuit deja le
  **chemin FS + username Windows** : `sprint81_t2_j_shard_inference.json:15`
  (`"model": "C:/Users/FlowUP/spike_fork/codellama-34b.gguf"`) + `:22`
  (diagnosis). Le check `artifact_hygiene` (`:43`) ne verifie QUE le prefixe
  pubkey-membre, **pas le chemin**. => le schema de
  `sprint82_t2_benchmarks.json` doit porter **model NAME + blake3**, jamais le
  chemin absolu, et le check d'hygiene doit etre etendu.
- **CAUTION 2** : l'acquisition de llama-bench/perplexity est un **choix a
  securiser** — preferer un build depuis source pinnee a un prebuilt telecharge
  (surface supply-chain). Croise l'invariant 0-dep (build-surface, pas dep
  runtime, mais a documenter).
- **Garde-fou conception** : la perplexite doit rester calculee **tail-side** et
  emise en **scalaire** ; ne PAS router de vecteurs logprob par-token
  cross-machine (elargirait SI-1/SI-3).

## S4 — Wire format / canonical (0 bump)

Signal scan : **PLAN-ADAPT**. Invariant **0-wire SBFB TENU**, verifie sur disque.

- `RunMetrics` (`shard_plan.rs:383-403`) est all-integer, partie du `RunProof`
  signe JCS (`DOMAIN_RUN_PROOF_V1`, « must round-trip bit-identically »
  `:379-380`) : porte deja `ttft_ms` (:385), `decode_milli_tokens_per_sec`
  (:390), `p95_token_latency_ms` (:393). **TTFT = ttft_ms ; TPOT = 1/decode_rate
  derivable ; ITL-moyen present** ⇒ 0 champ nouveau requis sur le wire.
  **PIEGE A PROSCRIRE** : ajouter un percentile/ITL a `RunMetrics` change les
  canonical bytes ⇒ bumperait `RUN_PROOF_FORMAT_VERSION` = **violation D4**.
- Le RunProof est **node-local « never wire, never on-disk »**
  (`shard_session.rs:79-82`) ; il n'est expose a la loopback que comme **hash
  hex** de signature (`schemas/shard.rs:149-151`).
- La vue loopback NON-SIGNEE `ShardSessionResultView` (`schemas/shard.rs:127-165`)
  est trop grossiere (`ttft_s` en **secondes entieres** :136, `toks_per_s`
  entier :143). **Precedent additif 0-bump verifie** : `rtt_frontier_ms` ajoute
  S81-I (`schemas/shard.rs:80-88` — « Additive on the inner view … 0-bump » +
  refresh snapshot drift-gate `:476-507`). => surfacer TTFT_ms/TPOT/ITL fins =
  **champ additif** sur ce DTO (0-bump) et/ou ecriture artefact local.
- **Mesure host-side possible** : le decode sharde est un round-trip
  token-par-token pilote par l'hote (`drive_decode_loop`,
  `shard_session.rs:1297-1574`) ; chaque `ShardStepReply` = 1 token
  (`shard.rs:468-486`). L'ITL par-token derive de deltas `Instant` host-side.
  Le `p95_token_latency_ms` actuel est une **MOYENNE** mal-etiquetee
  (`shard_session.rs:1546` = `decode_ms/tokens`) — a instrumenter reellement
  dans la struct `Outcome` interne NON-WIRE (remplissage, pas changement de
  forme).
- Tous les `*_FORMAT_VERSION` = 1 et inchanges (`FEED_FORMAT_VERSION=1`,
  `SHARD_PLAN_FORMAT_VERSION=1`, `RUN_PROOF_FORMAT_VERSION=1`,
  `COMPUTE_GROUP_FORMAT_VERSION=1`, `SHARD_STEP_PAYLOAD_V=1`) ; les 4 types
  invariants (Task/ProjectAnnouncement/CuratorList/FeedEntry) ne sont que
  traverses en lecture ; l'ALPN `sbfb/shard/1` reste byte-identique. L'artefact
  `sprint82_t2_benchmarks.json` est un **fichier local committe**, jamais
  transporte (`sprint82_plan.md:94-99`, miroir `sprint82_t2_bootseed.json`).

## Scope confirme (ce qui EST livrable dans Phase B)

Tout verifie sur disque, a 0-wire / 0-dep :

1. **Harness benchmark versionne DETERMINISTE RUNNABLE** — chemin shard = extension
   de `b3_shard_pipeline.sh` (routes loopback existantes) ; chemin single-machine =
   script frere enveloppant llama-bench/perplexity **construits separement**
   (rig-only build-tool).
2. **Artefact `sprint82_t2_benchmarks.json` committe** — fichier local hors-wire,
   schema versionne : model NAME + blake3 + quant + split + TTFT_ms/TPOT/ITL/
   throughput + pp512/tg128 + PPL(entier)/PPL(sharde). Model NAME + blake3, jamais
   le chemin absolu.
3. **README §4 amende (tier T3 + track audit + invariant kickoff)** — pur docs,
   **independant du rig**, ratifiable meme si la mesure est BLOCK{rig}.
4. **Note perplexity-parity entier-vs-sharde** — epingle le commit llama.cpp exact.

**Rig-gate (pas un scope-cut)** : la MESURE LIVE (llama-bench single-machine +
perplexity-parity shardee 2-machines : Mac Metal + GGUF ~20 Go + worker
`--features llm_llama_cpp_metal`, chemin GGUF-direct pas Ollama) ⇒ **BLOCK{rig}**
si froid, **JAMAIS RIG-ABSENT** (rig engage pour Phase A, `sprint82_plan.md:100-101`).
Le harness/schema/note/amendement restent livrables ; T2 **PASS** quand le rig tourne.

## Approche retenue (PLAN-ADAPT : approche corrigee concrete)

**Piste 1 — outils standards construits SEPAREMENT.** Builder llama-bench +
perplexity comme **build-tools rig-only** depuis un checkout **upstream** de
llama.cpp epingle au MEME snapshot que le backend shard vendore. Ancre de
provenance = `THIRD-PARTY-NOTICES.md:13` → `utilityai/llama-cpp-rs
4afdaf0782ef7f3254a186a7ff67a1c7491c6dce` (`.cargo_vcs_info.json`) ; resoudre le
sha `ggml-org/llama.cpp` bundle dans ce commit et le checkouter dehors de
l'arbre Cargo, builde avec le MEME backend (CUDA/Metal). => 0 churn Cargo.lock,
0 dep runtime. **Rejeter** : (b) flip `LLAMA_BUILD_TOOLS=ON` (build.rs
`cc::Build` mtmd-seul, CMakeLists tools absents, fragile) ; (c) prebuilt
telecharge (supply-chain, S3).

**Piste 2 — metriques fines shardees mesurees NATIVEMENT host-side.**
Instrumenter de vrais timestamps par-token dans `drive_decode_loop`
(`shard_session.rs`), calcul dans `Outcome` NON-WIRE ; remplacer le p95 moyenne
par une distribution reelle (carry J7-2). Surfacer via **champ additif** sur
`ShardSessionResultView` (precedent `rtt_frontier_ms`, 0-bump + refresh
snapshot) et/ou l'artefact local. **JAMAIS** `RunMetrics`/`RunProof`.

**Perplexite shardee** : calcul **tail-side**, scalaire via `/result` existant ;
zero vecteur logprob cross-machine.

**Canon T3** : 3 gestes **README §4 uniquement** (ligne T3 + Track audit miroir
Track J + invariant kickoff miroir #16). Memory pts 2/3 STALE, ne pas toucher
roadmap-v5/S81.

**Determinisme** : epingler blake3 modele + hash wikitext-2 + seed + n_threads +
quant + commit llama.cpp exact.

## Questions ouvertes tranchables au code (avec defaut recommande)

1. **Provenance llama-bench/perplexity** — DEFAUT (a) checkout upstream
   ggml-org/llama.cpp SEPARE, epingle au sha bundle dans utilityai@4afdaf0,
   builde rig-only meme backend. Rejeter prebuilt + flip TOOLS=ON.
2. **Routage metriques fines** — DEFAUT champ additif sur
   `ShardSessionResultView` (0-bump + refresh snapshot) + artefact local ; jamais
   `RunMetrics`/`RunProof`.
3. **ITL/p95 reel vs moyenne** — DEFAUT instrumenter timestamps par-token
   host-side dans `drive_decode_loop`, struct `Outcome` NON-WIRE ; remplacer
   `p95_token_latency_ms = decode_ms/tokens`. Carry J7-2, 0 bump.
4. **Champ modele de l'artefact** — DEFAUT model NAME + blake3, jamais chemin FS
   absolu (fuit username, cf. `sprint81_t2_j_shard_inference.json:15,22`) ;
   etendre le check `artifact_hygiene` pour rejeter tout `C:/Users`/`/Users/`.
5. **Verdict rig** — DEFAUT BLOCK{rig} si binaires/GGUF/worker-metal/SSH/session
   absents ; PASS sinon ; JAMAIS RIG-ABSENT.
6. **Surface perplexite shardee** — DEFAUT tail-side, scalaire via `/result` ;
   zero vecteur logprob cross-machine (garde-fou S3).
7. **Forme du harness** — DEFAUT etendre `b3_shard_pipeline.sh` (chemin shard) +
   script frere enveloppant les binaires (single-machine) ; cout de build rig
   documente dans `rig.local.env`, jamais CI.
8. **Amendement canon T3** — DEFAUT README §4 UNIQUEMENT (ligne T3 + Track audit
   + invariant kickoff) ; ne pas toucher roadmap-v5/S81 (memory pts 2/3 STALE).

## Evidence (file:line + sources OSS)

- **0-wire** : `crates/nexus-core-rs/src/shard_plan.rs:383-403` (RunMetrics
  all-integer + ttft_ms:385 + decode_milli_tokens_per_sec:390 +
  p95_token_latency_ms:393) ; `:379-380` (« round-trip bit-identically ») ;
  `:412-459` (RunProof.metrics) ; `crates/nexus-core-rs/src/schemas/shard.rs:127-165`
  (ShardSessionResultView : ttft_s:136 secondes, toks_per_s:143) ; `:80-88`
  (precedent additif rtt_frontier_ms 0-bump) ; `:476-507` (drift snapshot).
- **Mesure host-side** : `crates/nexus-shell-daemon/src/shard_session.rs:1297-1574`
  (drive_decode_loop) ; `:79-82` (registre in-memory node-local) ; `:1546` (p95 =
  moyenne mal-etiquetee) ; `crates/nexus-core-rs/src/schemas/shard.rs` ShardStepReply.
- **llama-bench absent du fork** : `vendor/llama-cpp-sys-2/llama.cpp/tools/` =
  `mtmd` seul (ls) ; `examples/` absent (ls exit 2) ;
  `vendor/llama-cpp-sys-2/build.rs:531-534` (LLAMA_BUILD_TOOLS=OFF) + `:847-849`
  (commentaire).
- **Provenance / determinisme** : `THIRD-PARTY-NOTICES.md:13`
  (utilityai/llama-cpp-rs 4afdaf0782ef...) ; `.cargo_vcs_info.json` ;
  `vendor/llama-cpp-sys-2/llama.cpp/cmake/build-info.cmake:1` (BUILD_NUMBER 0,
  BUILD_COMMIT "unknown") ; `patches/llama-cpp-shard.patch`.
- **Fuite chemin FS** : `.planning/archive/v2.1/sprint81_t2_j_shard_inference.json:15,22`
  (`C:/Users/FlowUP/spike_fork/...`) + `:43` (artifact_hygiene ne couvre que le
  prefixe pubkey).
- **Threat / duress** : `crates/nexus-shell-daemon/src/http.rs:6706-6795`
  (shard_session_routes_noop_in_duress) ; `docs/security/THREAT_MODEL.md:1241-1296`
  (§16 + SI-1..SI-5).
- **Deps** : `crates/nexus-worker-core/Cargo.toml:68` (serde_json workspace) ;
  `scripts/acceptance/b3_shard_pipeline.sh:128-190` (emit artefact 0-dep) ;
  `Cargo.toml:498,501-503` (CI never builds llm_llama_cpp*, patch inchange).
- **Plan / canon** : `.planning/active/sprint82_plan.md:84-104` (Goal/Livrables/
  Testabilite Phase B) ; `docs/claude/README.md:622-626` (table T0/T1/T2, aucun
  T3) + `:643-650` (Track J) + `:648-650` (invariant kickoff #16).
- **Design gele** : `.planning/research/sharding_design_addendum_sota_2026-05-30.md`
  (0 hit perplexity/llama-bench/tg128/pp512/TPOT/ITL/MLPerf/vLLM/wikitext).
- **Sources OSS** :
  [knightli.com pp512/tg128](https://knightli.com/en/2026/04/23/llama-cpp-gpu-benchmark-cuda-rocm-vulkan-scoreboard/) ,
  [blog.majid.info llama-bench](https://blog.majid.info/llama-bench/) ,
  [Anyscale metrics](https://docs.anyscale.com/llm/serving/benchmarking/metrics) ,
  [Spheron TTFT/ITL 2026](https://www.spheron.network/blog/llm-inference-slo-ttft-itl-latency-budget-guide-2026/) ,
  [vllm#6531 ITL vs TPOT](https://github.com/vllm-project/vllm/issues/6531) .
