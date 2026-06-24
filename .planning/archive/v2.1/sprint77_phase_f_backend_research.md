# Recherche alternatives au fork — Phase F backend shard

> Recherche multi-source (Workflow 3 axes + synthese) demandee par le PO AVANT
> d'engager l'option (a) fork llama.cpp. Question : Ollama (changelog/plugin/
> cross-host/hidden states) ou un systeme OSS offre-t-il un chemin plus simple
> vers le decoupage cross-machine compatible verif TOPLOC + integration Rust ?
> **Recommandation : CONFIRMER-A-FORK.**

## Reponse courte

**NON, il n'existe pas de chemin cle-en-main plus simple que forker llama.cpp** pour livrer le vrai 70B-eclate cross-machine de S77. Aucun runtime tiers (Ollama, exo, Petals, Parallax, mistral.rs...) ne tient **simultanement** les 4 contraintes SBFB : decoupage par blocs de couches entre machines + acces aux hidden states pour la verif TOPLOC + integration Rust legere + p2p mutuellement non-confiant. Le **seul** concurrent serieux au fork est **candle** (Rust pur, Apache/MIT) : il supprime le FFI et donne un controle total des couches, mais il ne supprime pas le travail — il deplace le « fork » vers *reecrire et maintenir ~300 lignes du forward du modele dans notre crate*, avec une perf quantifiee-CUDA inferieure a llama.cpp. **L'option (a) reste le bon choix** pour la fidelite/perf GGUF max et parce qu'on controle deja le backend en Rust in-process.

## Ollama

Reponse directe a « Ollama est-il un raccourci ? » : **non, sur les 4 sous-axes**.

- **Decoupage cross-machine** : pas de support officiel. Issue de reference [#4643](https://github.com/ollama/ollama/issues/4643) **OUVERTE depuis mai 2024** (0 PR mergee) ; [#9147](https://github.com/ollama/ollama/issues/9147) close « *ollama doesn't currently support distributed inference* » ; la seule tentative de layer-split ([PR #10844](https://github.com/ollama/ollama/pull/10844), ouverte) **wrappe le backend RPC de llama.cpp** — exactement ce qui a ete rejete en D2 (graphe central, workers passifs, 0 verif, LAN-only). Le « distribue » Ollama existant = **load-balancing d'instances entieres** (OLOL/SOLLOL), pas le decoupage d'un modele.
- **Plugins / backend custom** : aucun. Moteur Go monolithique, scheduler interne non expose.
- **Hidden states** : `logprobs`/`top_logprobs` en [v0.12.11](https://github.com/ollama/ollama/releases/tag/v0.12.11) (nov. 2025) = **top-N tokens de sortie** (cap 0-20), **pas le vecteur hidden state**. `/api/embed` = embedding poole+normalise, pas le dernier hidden state brut. TOPLOC N0 (LSH top-k k=128 du dernier hidden state) **inaccessible** en HTTP.
- **Forker Ollama ?** Ollama a quitte llama.cpp (mai 2025) pour **son propre moteur Go/GGML**. Forker ce moteur Go pour layer-split + extraction d'activations est **au moins aussi dur** que forker llama.cpp, alors qu'on a deja `llama-cpp-2` in-process (logits + sampler accessibles).

**Verdict : Ollama ne fait gagner aucun temps pour S77. A ecarter.**

## Systemes OSS distribues

| Systeme | Cross-machine par blocs ? | Hidden states exposes ? | Integration Rust | Fork requis ? | Plus simple que (a) ? |
|---|---|---|---|---|---|
| **llama-cpp-2** (baseline) | non | partiel — final seulement | faible (deja in-process) | oui pour le partiel | — (etat de depart) |
| **candle** (HF, Rust pur) | oui (via reecriture) | **oui** (boucle couches explicite) | moyen — FFI 0, vendorer ~300 L de forward | partiel (reimplementer le forward) | **seule alternative credible** |
| **Petals** | oui natif | **oui** | eleve — Python/torch/hivemind, subprocess | partiel | non — 0 verif Byzantine + runtime Python |
| **Parallax / Lattica** | oui natif | incertain | moyen-eleve — coeur Python, P2P Rust | oui | non — moteur SGLang + churn DHT |
| **exo** | oui natif | oui | eleve — Python/MLX, Apple-centre | oui | non — modele home-cluster confiant, GPL |
| **prima.cpp** | oui (= un fork llama.cpp) | incertain | eleve — re-binder le fork C++ | oui | non — c'est deja (a), faite par un tiers (Halda central, ZMQ) |
| **distributed-llama** | non (tensor-parallel LAN) | non | eleve — C++, TCP custom | oui | non — orchestrateur central, LAN-only |
| **Cake** (Rust/candle) | oui natif | incertain | moyen — Rust mais master central | partiel | non — master central, 0 verif |
| **GPUStack / llama-box** | oui (= RPC llama.cpp) | non | eleve | oui | non — RPC deja rejete D2 |
| **vLLM + Ray** | oui (datacenter) | non | eleve — Python + Ray central | oui | non — orchestration Ray datacenter |
| **NVIDIA Dynamo / Mooncake** | non (disagg. P/D RDMA) | non | eleve | oui | hors-scope — datacenter NVLink |

## La contrainte qui decide

Trois contraintes eliminent en cascade les raccourcis ; **une seule** suffit deja a disqualifier la plupart :

1. **Forward partiel d'un range de couches avec injection d'un hidden state amont** (contrainte #1). Le vrai verrou. `llama-cpp-2` (pin `^0.1.143`, resolu **0.1.146**) n'expose **ni eval-callback, ni layer-range, ni injection d'embedding** : `embeddings_ith` = hidden state **final** du modele entier (suffit pour TOPLOC mono-host, **pas** pour un shard). Tout systeme boite-noire generate-only (Ollama, exo subprocess) tombe ici.
2. **Acces aux hidden states pour la verif Byzantine TOPLOC** (contrainte #2). Elimine Ollama, distributed-llama, GPUStack, vLLM, Dynamo. Petals/exo cochent mais sur runtime Python.
3. **p2p mutuellement non-confiant** (contrainte #4). Elimine tous les modeles master/worker central (distributed-llama, Cake, mistral.rs NCCL, GPUStack RPC, vLLM Ray) — motif deja rejete D2. Et 0 systeme tiers n'implemente la verif N0-N4 : **le coeur du travail SBFB reste a ecrire de toute facon**.

**Le seul candidat qui survit honnetement, c'est candle.** Pesee franche contre le fork :

| Critere | Fork llama.cpp (option a) | candle (Rust pur) |
|---|---|---|
| Decoupe 70B-GGUF par blocs | **oui** (modele prima.cpp prouve, 70B/4 devices) | oui mais forward reimplemente |
| Hidden states de frontiere | oui (acces ggml interne) | **oui** (Tensor injectable/extractible, le plus propre) |
| Integration Rust | FFI/binding a maintenir | **FFI 0, Rust natif, controle total** |
| Perf quantifiee CUDA | **max** (kernels GGUF natifs) | **inferieure** (dequant->fp16, issues #1250/#1813) |
| Cout reel | maintenir un patch sur un C++ etranger | **vendorer + maintenir ~300 L de forward modele** + fields prives a contourner |
| Fidelite GGUF / ecosysteme | reference | suit, moins mature sur 70B quant |

Le « fork » ne disparait pas avec candle : il se **deplace** de « patcher du C++ etranger » vers « porter et maintenir le forward du modele en Rust », avec une **perte de perf sur la quantification CUDA** (sensible pour du 70B sur GPU grand public). On gagne le Rust pur et le controle des couches ; on perd la fidelite GGUF et la perf, et on herite d'un cout de maintenance de code modele.

## Recommandation

**Confirme l'option (a) : forker llama.cpp** (modele prima.cpp, MIT, compat AGPL). C'est la voie qui livre **a la fois** le block-split 70B-GGUF reel **et** l'acces aux activations de frontiere — les deux besoins exacts de S77 — avec la **perf quantifiee maximale**, en restant dans le backend qu'on controle deja en Rust in-process (`feature llm_llama_cpp`).

Honnetete sur le cout du fork : il faut maintenir un patch sur du C++ tiers (rebase periodique sur upstream ggml), et le fork de reference (prima.cpp) apporte un **scheduler central Halda + transport ZMQ + 0 verif Byzantine** — donc on n'emprunte que la **mecanique ggml du ring/layer-range**, pas son control-plane : le placement (Phase D/E deja code), le transport (iroh/ALPN `sbfb/shard/1`) et la verif (N0-N4) restent du code SBFB. C'est un fork **minimal et chirurgical** sur ggml, pas l'adoption d'un runtime tiers.

Deux nuances a acter explicitement avec le PO :

- **Si le 70B-eclate reel ne tient pas dans ce sprint**, le repli produit existe deja et coute zero fork : `PlacementOutcome::EndpointFederation` (`crates/nexus-coordinator-rs/src/placement.rs:159-216`) — 1 worker = 1 modele entier, hidden state final via `embeddings_ith`, verif TOPLOC N0 pleinement fonctionnelle. Ca **abandonne le livrable coeur** (un 70B sur 2+ machines) mais garde un sens produit (plusieurs modeles entiers federes). A ne declencher que si le fork derape, **pas** comme objectif par defaut.
- **candle reste le plan B no-fork-C++** si le cout de maintenance du patch ggml devient ingerable. Le garder en reserve documentee, sans l'engager : il echange un cout de maintenance C++ contre un cout de maintenance Rust + une perte de perf quant — pari moins favorable tant que la perf GGUF compte.

Aucune alternative facile (Ollama, runtime tiers en subprocess) ne change la donne : ni le forward partiel, ni les hidden states de frontiere, ni le modele non-confiant. **Le fork est le chemin le plus propre vers le 70B-eclate verifiable — confirme.**
