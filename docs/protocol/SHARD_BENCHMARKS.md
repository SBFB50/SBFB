<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Benchmarks standards du sharding (Sprint 82 Phase B)

> Note de conception. Mesurer les perfs du sharding pipeline-parallel avec
> des **outils et un vocabulaire reconnus**, pour que les optimisations
> futures (réutilisation KV F2, quantisation, topologie A/B, 3 machines) se
> décident sur des baselines **versionnées et comparables dans le temps** —
> pas sur une impression (« ~2 tok/s contre une fenêtre prédite »).
>
> Canon : tier **T3 — Benchmark de référence** (`docs/claude/README.md §4`),
> opt-in par-sprint. Harness : `scripts/acceptance/benchmarks_standards.sh`
> (+ `b3_shard_pipeline.sh` pour le chemin shardé). Artefact versionné :
> `.planning/active/sprint82_t2_benchmarks.json` (`schema_version: 1`).

## 1. Ce qu'on mesure (et pourquoi ces outils)

| Métrique | Outil / source | Ce qu'elle capture |
|---|---|---|
| **pp512** (tok/s) | `llama-bench` (llama.cpp officiel) | prompt-processing 512 tokens — débit d'un forward pass, **compute-bound** |
| **tg128** (tok/s) | `llama-bench` | text-generation 128 tokens un-par-un via KV-cache — **memory/latency-bound** |
| **PPL** (perplexité) | `perplexity` (llama.cpp officiel), corpus **wikitext-2-raw** | qualité du modèle sur un corpus standard — base de la **parité entier-vs-shardé** |
| **TTFT** (ms) | mesuré host-side (chemin shardé) | time-to-first-token (prefill) — vocabulaire vLLM/MLPerf |
| **TPOT** (ms) | host-side | time-per-output-token = **moyenne** des écarts inter-token post-1er token |
| **ITL p50/p95** (ms) | host-side | inter-token-latency, **distribution réelle** (nearest-rank), pas une moyenne |
| **débit** (milli-tok/s) | host-side | `decode_milli_tokens_per_sec` (2 300 = 2,3 tok/s), résolution sub-entière |

`llama-bench` et `perplexity` sont les outils **officiels** livrés avec
llama.cpp ; `pp512`/`tg128` et la PPL wikitext-2 sont le langage commun des
comparatifs GPU llama.cpp. TTFT/TPOT/ITL sont les métriques de latence de
service standard (vLLM, MLPerf Inference) : on emprunte leurs **définitions**,
on ne prétend pas rejouer leur harness complet (hors gabarit rig 2 machines).

## 2. Provenance — build des outils standards (PLAN-ADAPT preflight G8)

**Les binaires `llama-bench`/`perplexity` ne sont PAS dans le fork vendoré.**
`vendor/llama-cpp-sys-2/build.rs` fixe `LLAMA_BUILD_TOOLS=OFF` (le seul tool
compilé est `mtmd`, via `cc::Build` ; `tools/CMakeLists.txt` n'est pas inclus
dans le package du crate). Ils doivent donc être construits **séparément**,
depuis un checkout **upstream** de llama.cpp, épinglé au **même snapshot** que
le backend shard vendoré — sinon la parité PPL compare des kernels GGML
différents et la preuve de non-dégradation est invalide.

**Chaîne de provenance à résoudre (rig-side, une fois) :**

1. Le crate vendoré est snapshotté par `utilityai/llama-cpp-rs` au commit
   `4afdaf0782ef7f3254a186a7ff67a1c7491c6dce`
   (`vendor/llama-cpp-sys-2/.cargo_vcs_info.json` + `THIRD-PARTY-NOTICES.md`).
2. Le fork lui-même porte `BUILD_COMMIT "unknown"`
   (`vendor/llama-cpp-sys-2/llama.cpp/cmake/build-info.cmake`) — l'exact sha
   `ggml-org/llama.cpp` **n'est pas gravé dans l'arbre**.
3. Résoudre ce sha = le commit du **sous-module `llama.cpp`** de
   `utilityai/llama-cpp-rs@4afdaf0`. Checkouter ce sha **hors** de l'arbre
   Cargo, appliquer (si l'on veut le chemin shardé identique) le delta
   `patches/llama-cpp-shard.patch`, et builder avec le **même backend** :
   CUDA sur la RTX 5080, Metal sur le Mac M2.

C'est un **build-tool**, jamais une entrée `[dependencies]` : **0 churn
`Cargo.lock`, 0 dep runtime**, la CI ne build jamais les features
`llm_llama_cpp*`. On renseigne `LLAMACPP_COMMIT` (le sha ggml-org résolu) dans
`rig.local.env` pour le graver dans l'artefact. Alternatives **rejetées** :
flip `LLAMA_BUILD_TOOLS=ON` (build-surface fragile, CMakeLists tools absents) ;
binaire prebuilt téléchargé (surface supply-chain nouvelle).

## 3. Parité de perplexité — entier vs shardé

- **PPL(entier)** : `perplexity -m <gguf> -f wikitext-2-raw/wiki.test.raw` sur
  **une** machine (le modèle entier), seed pinné. C'est la référence.
- **PPL(shardé)** : **PAS câblée en Phase B.** Aucun producteur n'émet
  aujourd'hui une PPL shardée (ni `b3_shard_pipeline.sh`, ni la vue Rust
  `/result`), donc `ppl_sharded` et `delta` restent **`null`**, même sur rig
  chaud. Le **design visé** (travail futur) : la PPL serait calculée
  **tail-side** par le dernier shard (qui fait déjà
  `lm_head -> logits -> argmax -> detokenize`) et émise en **scalaire** via la
  route `/result` — **jamais** de vecteurs logprob par-token cross-machine
  (cela élargirait la surface d'activations en clair, THREAT_MODEL §16,
  SI-1/SI-3 ; garde-fou de conception). Tant que ce chemin n'est pas
  implémenté, la parité se limite honnêtement à `ppl_whole`.
- **Parité** : `delta = PPL(shardé) − PPL(entier)`. Le split pipeline-parallel
  est mathématiquement équivalent au forward entier (mêmes poids, même ordre de
  couches) ; un `delta` non-nul signale une divergence de kernel (backend
  hétérogène, précision de la frontière fp32-LE) à investiguer, pas un défaut
  de conception.

## 4. Métriques fines host-side (0 bump wire)

Le décode shardé est un round-trip **token-par-token piloté par l'hôte**
(`drive_decode_loop`, `shard_session.rs`) : chaque `ShardStepReply` = 1 token,
donc l'hôte voit l'instant d'arrivée de chaque token. On dérive :

- **TTFT** = instant du 1er reply (prefill) ;
- **écarts inter-token** = deltas entre replies consécutifs (le 1er token est le
  TTFT, exclu des écarts) ;
- **TPOT** = moyenne des écarts ; **ITL p50/p95** = percentiles nearest-rank de
  la distribution des écarts.

Ces valeurs remontent au harness via des **champs additifs sur la vue loopback
NON-SIGNÉE** `ShardSessionResultView` (`ttft_ms`, `tpot_ms`, `itl_p50_ms`,
`itl_p95_ms`, `decode_milli_tokens_per_sec`) — précédent exact `rtt_frontier_ms`
(S81 Phase I, additif 0-bump + refresh du snapshot drift-gaté). **Jamais** dans
`RunMetrics`/`RunProof` : ce payload JCS signé doit round-trip byte-identique,
y ajouter un champ bumperait `RUN_PROOF_FORMAT_VERSION` (violation de
l'invariant refacto D4). Note d'honnêteté : le champ signé
`RunMetrics::p95_token_latency_ms` est en réalité une **moyenne**
(`decode_ms / tokens`), pas un vrai p95 — il reste byte-stable ; la vraie
distribution p50/p95 vit sur la vue non-signée + l'artefact.

## 5. Déterminisme + hygiène de l'artefact

**Déterminisme** — l'artefact épingle tout ce qui rend deux runs comparables :
modèle (NAME + blake3), quant, split (`n_shards`), commit llama.cpp,
`bench_params` (pp/tg/threads/repetitions), seed de perplexité, et le hash du
corpus wikitext-2 (`corpus_blake3`, calculé **quand `perplexity` tourne**). Une
régression future se détecte en comparant à la baseline committée sous ces mêmes
clés. Le harness **refuse un `PASS`** sans les pins load-bearing : `LLAMACPP_COMMIT`
doit être résolu (pas `unknown`) et le blake3 du modèle disponible (`b3sum`) —
sinon la baseline n'est pas reproductible et le verdict tombe à `BLOCK{rig}`.

**Hygiène** — l'artefact committé porte le modèle en **NAME + blake3**, **jamais
un chemin de fichier** (un artefact shard antérieur fuitait
`C:/Users/<user>/spike_fork/...` = layout FS + username). `redact_model` réduit
tout chemin à son basename ; `assert_no_fs_path` échoue la run si un chemin
survit ; les diagnostics `BLOCK` référencent des **noms de variables**, jamais
des valeurs de chemin.

## 6. Schéma de l'artefact (`schema_version: 1`)

```json
{
  "schema_version": 1,
  "status": "PASS | BLOCK",
  "diagnosis": "texte (jamais un chemin FS)",
  "model": "codellama-34b.gguf",
  "model_blake3": "<64 hex | null>",
  "quant": "Q4_K_M",
  "n_shards": 2,
  "llamacpp_commit": "<sha ggml-org résolu | unknown>",
  "bench_params": {"pp": 512, "tg": 128, "threads": 8, "repetitions": 3},
  "single_machine": [
    {"machine": "head", "backend": "cuda",  "pp_tok_s": 0.0, "tg_tok_s": 0.0}
  ],
  "sharded": {
    "n_shards": 2, "ttft_ms": 0, "tpot_ms": 0,
    "itl_p50_ms": 0, "itl_p95_ms": 0,
    "decode_milli_tokens_per_sec": 0, "tokens": 0
  },
  "perplexity_parity": {
    "corpus": "wikitext-2-raw", "corpus_blake3": "<64 hex | null>", "seed": 1234,
    "ppl_whole": 0.0, "ppl_sharded": null, "delta": null
  }
}
```

`sharded` et `perplexity_parity` valent `null` tant que la mesure
correspondante n'a pas tourné ; `ppl_sharded`/`delta` restent `null` en
Phase B (chemin `/result` non câblé, cf. §3). `corpus_blake3` épingle le
contenu exact du corpus wikitext-2 (calculé quand `perplexity` tourne ;
`perplexity` ne gate PAS le `PASS`, la parité étant incomplète).

**Rig-gate / anti-faux-vert (durci, Codex Phase B)** — un `PASS` exige que le
**sharding ait réellement été mesuré** avec une provenance comparable :
(1) baselines single-machine `llama-bench` pp/tg ; (2) un jeu de métriques
shardées **VALIDÉ** — le harness rejette un artefact `b3` sauf si `status=PASS`,
le **NAME ET le blake3** du modèle correspondent, `n_shards` correspond, **les
cinq métriques fines + `tokens` sont des entiers**, et le fichier est **récent**
(`mtime < B3_MAX_AGE_MIN`, défaut 120 min — proxy de fraîcheur, pas un run-id
cryptographique : un fichier délibérément « touché » le contourne, mais un vieil
artefact accidentellement réutilisé est rejeté). Jamais fusionner une baseline
courante avec un vieux run shardé sans lien ; (3) les pins de provenance —
`LLAMACPP_COMMIT` doit être un **sha hex** (pas `unknown`) et le blake3 du modèle
un **64-hex valide** (pas vide/`unavailable`). Tout requis manquant/malformé
(binaires, GGUF ~20 Go, session shardée non montée/pilotée, métriques
incomplètes, pins absents) ⇒ **`BLOCK{rig}`** (jamais `RIG-ABSENT` — le rig est
engagé pour la Phase A boot-SEED). Une écriture d'artefact qui échoue est
**FATAL (exit 2)**, jamais un exit-0 creux. Le harness, le schéma, cette note et l'amendement canon
T3 restent livrables ; l'artefact passe à `PASS` quand le rig tourne avec ces
requis satisfaits.
