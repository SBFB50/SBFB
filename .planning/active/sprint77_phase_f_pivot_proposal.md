# Sprint 77 Phase F — Pivot proposal (DESIGN-CONFLICT)

> Document de decision PO. Genere par le preflight G8 (Workflow 6 agents,
> verdict DESIGN-CONFLICT confirme par 2 scans concordants + refutation
> adversariale echouee). Detail factuel complet : `sprint77_phase_f_preflight.md`.

## Le conflit en une phrase

Le livrable **coeur** de la Phase F (§9.2 #1 du plan) — exécuter un **forward
partiel** d'un bloc de couches `layer_start..layer_end`, en **injectant** un
hidden state amont en entrée et en **extrayant** le hidden state intermédiaire de
frontière — est **infaisable via le wrapper safe `llama-cpp-2` 0.1.146** pinné au
workspace. C'est exactement le mécanisme du « 70B éclaté cross-machine », la
feature phare de S77 (scope MAXIMAL D3).

## Pourquoi c'est bloqué (factuel, réfuté adversarialement)

| Capacité | Statut | Source |
|---|---|---|
| Extraire le hidden state **FINAL** + top-k (TOPLOC N0, dernier shard) | **FAISABLE** sans fork — `LlamaContext::embeddings_ith()` | docs.rs/llama-cpp-2 ; discussions/3643 |
| **Forward partiel** d'un range de couches + injection d'entrée + sortie intermédiaire | **INFAISABLE** via l'API safe — aucun `cb_eval`, aucun layer-range, aucune injection d'embedding | docs.rs LlamaModel/LlamaContextParams |
| eval-callback (seule voie no-fork plausible) | **N'aide pas** — observe pendant un forward complet, n'injecte pas, ne démarre pas à une couche arbitraire ; vit dans `-sys-2` unsafe | docs.rs/llama-cpp-sys-2 |
| Preuve par l'exemple | **prima.cpp** (seul SOTA 70B-GGUF pipeline par blocs) est un **FORK** de llama.cpp | discussions/12852 ; arxiv 2504.08791 |

Le fallback R2 prévu au plan (§9.4 « dégrader N2 ») ne couvre **que** l'extraction
finale (déjà faisable) — il **ne traite pas** la dimension infaisable. D'où le
DESIGN-CONFLICT (et non un SCOPE-CUT-CONSISTENT, qui préserverait l'objectif).

## Tension Day-0 sous-jacente

Deux engagements gelés se contredisent ici :
- **D3 / scope MAXIMAL** : « S77 vise le 70B COMPLET éclaté cross-machine, 0 defer du cœur ».
- **Addendum design figé** : « SBFB n'écrit aucun kernel ggml/CUDA » + transport/archi pipeline figés, implicitement « sans fork ».

Le préflight ne tranche pas un Day-0 — il **remonte** le conflit. Note : un
fork/patch de l'**orchestration C++** (graph-split + hook d'éval) **réutilise** les
kernels ggml existants ; il n'écrit pas de kernel CUDA. La lettre de « aucun kernel
ggml » n'est donc pas forcément violée par (a) — mais « sans fork » l'est.

## Les trois options

### (a) Fork/patch llama.cpp — livre le cœur, brise « sans fork »
Exposer un `llama_decode` partiel + injection d'embeddings via graph-split C++
(modèle prima.cpp). **Pour** : réalise réellement le 70B-éclaté, honore 0-defer.
**Contre** : effort C++ significatif, dépendance à un fork maintenu (`llama-cpp-2`
ne suffit plus), surface build native élargie, en tension avec « sans fork ».
Risque calendaire : peut déborder une seule phase.

### (b) Forward complet in-process + sous-livrables + spike — defer le cœur
Le worker tient le modèle **entier** + extrait le hidden state final + primitives
shard [déjà Phase C] + claim ComputeGroup/VRAM + spike toy multi-process. Le
**forward partiel réel est différé** (carry P1 avec rig de convergence).
**Pour** : 100% faisable via l'API safe, livre N0/wire/transport/claim, débloque
G-K sur le chemin d'extraction finale. **Contre** : **SCOPE-CUT explicite contre
0-defer** — le 70B éclaté cross-machine n'existe pas réellement ce sprint
(chaque worker chargerait tout le modèle, pas un bloc).

### (c) Backend ggml custom from-scratch — effort maximal
Réécrire un chemin d'exécution par bloc sur ggml bas-niveau. Même tension que (a),
effort le plus élevé, le moins de réutilisation. Généralement dominé par (a).

## Sous-livrables exécutables QUEL QUE SOIT l'arbitrage

Indépendants du backend de forward (valides intégralement pour (b), pré-requis pour (a)/(c)) :
1. Claim `ShardAssignment` (lecture `layer_start/layer_end/role/...`) — 0-bump-wire.
2. Filtre groupe privé au claim (réutilise admission Phase B) + **vérif signature
   `ShardedSessionManifest` côté dialer** (garde P1 à ajouter, S3-F-1).
3. **Cap VRAM fail-closed au claim** (S3-F-2 P1) — VRAM déjà mesurée, **pas** de
   pompe live (scope cut #7 respecté).
4. Extraction hidden state final + top-k extractible (prérequis N0, slot RunProof
   reste `[0u8;32]` jusqu'à Phase G).
5. Build `--features llm_llama_cpp` tôt (R2, double test).
6. **Section surface shard dans THREAT_MODEL** (S3-F-5 P1, SI-1 + SI-4 + caveat).

## Recommandation

À l'utilisateur (PO) de trancher. Cadrage honnête :
- Si l'objectif non-négociable de S77 est **prouver le 70B réellement éclaté**, seul
  **(a)** le livre — au prix d'un fork assumé (et probablement d'un re-scope de la
  Phase F sur plusieurs phases : fork+build, puis claim+exec).
- Si l'on accepte de **prouver d'abord la chaîne complète (wire/transport/verif/claim)
  sur du forward complet** et de différer le forward partiel réel à une phase dédiée
  S77 (pas un autre sprint), **(b)** est le chemin pragmatique et 100% faisable — mais
  c'est un defer du cœur, à assumer explicitement contre la directive « sprints
  ultra-complets ».
- **(c)** n'est retenu que si un fork upstream est jugé inacceptable mais qu'on veut
  quand même le forward partiel — coûteux, rarement préférable à (a).

**Aucun code du forward partiel n'est écrit tant que le PO n'a pas choisi (a)/(b)/(c).**
