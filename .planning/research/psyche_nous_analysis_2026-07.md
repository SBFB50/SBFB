# Analyse Psyche Network + Nous Research — prior-art pour le sharding SBFB

- **Date** : 2026-07-10
- **Question posée** : « Analyse Psyche Network + Nous Research » — qui sont-ils, quelle
  architecture, et qu'est-ce que SBFB peut apprendre / réutiliser / doit refuser, sans
  complaisance.
- **Méthodologie** : 4 lecteurs indépendants ont collecté des faits sourcés
  (1. docs officielles `docs.psyche.network` ; 2. Nous Research corp/funding/portefeuille ;
  3. **lecture directe du code** `PsycheFoundation/psyche@main` ; 4. papiers DisTrO/DeMo +
  paysage concurrent). Chaque fait a ensuite été **contre-vérifié** claim-par-claim contre
  ses sources primaires (verdicts CONFIRMED / REFUTED / nuances). Ce rapport ne garde que
  le vérifié et signale explicitement le réfuté/non-vérifié (§5).

> Note de cadrage : Psyche fait de l'**entraînement** distribué (pre-training, gradients).
> SBFB fait de l'**inférence** shardée (pipeline-parallel, activations). Ce sont deux
> problèmes de communication **différents** (bande passante des gradients vs latence des
> activations). Tout le rapport tient cette distinction ; ne jamais la confondre.

---

## §1 — Nous Research (qui, quoi, financement, portefeuille)

Laboratoire d'IA open-source basé à New York, né informellement en 2022 comme collectif
Discord/Twitter de chercheurs, formalisé en 2023. Positionnement : casser le monopole des
big-tech sur l'entraînement frontier en utilisant des GPU sous-utilisés / grand public sur
l'internet ouvert, pour produire des modèles open-weight « human-centric », neutres,
résistants à la censure. [CONFIRMED, corroboré multi-sources]

- **Fondateurs** : Jeffrey Quesnelle (CEO, auteur principal du papier long-contexte YaRN) ;
  Karan Malhotra (Head of Behavior, visage public Hermes/WorldSim/DisTrO) ; Teknium (Head of
  Post Training, pseudonyme, développeur original de la lignée Hermes) ; Shivani Mitra. [CONFIRMED]
- **Financement** : seed ~5,2 M$ (janv. 2024, Distributed Global + OSS Capital) puis ~15 M$
  (juin 2024) = ~20 M$ seed ; **Série A 50 M$ (avril 2025) menée quasi-intégralement par
  Paradigm (crypto-VC), valorisation 1 Md$ exprimée contre un token pas-encore-émis (structure
  SAFT)** → ~65 M$ cumulés. Backers : Delphi, North Island, CEO de Together AI, co-fondateur
  Solana Raj Gokal. [CONFIRMED — Fortune 25/04/2025, SiliconANGLE, The Block]
- **Token** : **aucun token officiel émis au 13/04/2026** malgré la valorisation SAFT 1 Md$ ;
  les paires « NOUS »/« Psyche » sur DEX Solana sont des impostures. [CONFIRMED — Gate.com]

**Portefeuille technique** :
- **Hermes** (modèles open-weight phares). Hermes 4 (26/08/2025) = famille 14B/70B/405B
  post-entraînée sur checkpoints Meta Llama 3.1 ; raisonnement hybride `<think>` togglable,
  function-calling, 131K contexte, SOTA RefusalBench (censure réduite, alignement neutre).
  [CONFIRMED — arXiv 2508.18255]
- **Hermes 4.3 (36B)** = **premier modèle de prod post-entraîné ENTIÈREMENT sur le réseau
  Psyche** (base ByteDance Seed-36B, contexte 512K, débit moyen **144 000 tok/s sur 24 nœuds
  Psyche**), qui aurait **battu son jumeau entraîné en cluster centralisé**. [CONFIRMED —
  blog Nous ; date ~déc. 2025]
- **Forge Reasoning API** (bêta, nov. 2024) : moteur d'inference-time-scaling combinant
  plusieurs LLM open+closed + MCTS + Chain-of-Code ; a hissé Hermes 70B à parité avec des
  modèles bien plus gros, surpassant o1-preview sur AIME. [CONFIRMED]
- **DisTrO** (optimiseur) et **Psyche** (réseau) — cf. §2/§3.

---

## §2 — Psyche Network (architecture complète vérifiée sur le code)

Système d'**entraînement distribué** (pre-training) de LLM transformer entre parties
mutuellement non-confiantes, sur internet. Entraîne des architectures Llama/Deepseek chargées
via HuggingFace. Repo `PsycheFoundation/psyche`, **Apache-2.0**, ~78% Rust, v0.2.0 (janv. 2026),
~2560 commits, développement actif (derniers commits vus mars 2026). [CONFIRMED]

**Trois acteurs** : (1) **Coordinator** = source de vérité unique de l'état global d'un run ;
(2) **Clients** = nœuds GPU qui entraînent + soumettent + agissent comme **témoins** ;
(3) **Data Provider** = fournit les données (HTTP / TCP / fichiers locaux / bucket GCP). [CONFIRMED]

**Transport P2P = iroh — VÉRITÉ VÉRIFIÉE sur le code** (même famille QUIC que SBFB). Le
`Cargo.toml` racine pinne **iroh 0.97.0 / iroh-relay 0.97.0 / iroh-blobs 0.99 / iroh-gossip
0.97.0**, `ed25519-dalek 3.0.0-pre.1`, **aucun libp2p, aucun quinn direct** (QUIC = quinn
interne d'iroh). [CONFIRMED — lecture directe des manifestes]. Point important : le lecteur
« corp/presse » (source 2) n'avait **pas** pu confirmer iroh depuis les blogs/articles, mais
la **lecture du code (source 3) l'a confirmé directement** — c'est le fait porteur, il est
solide.

**Data-plane à 3 transports SÉPARÉS par type de payload** (pattern instructif) :
1. **Coordination** → `iroh-gossip` sur `gossip_topic(run_id)`, messages signés ed25519
   (`SignedMessage::sign_and_encode`).
2. **Résultats d'entraînement DisTrO** → `iroh-blobs` avec `MemStore` éphémère
   (`add_downloadable` → `BlobTicket` → `Downloader` **trié par latence** `latency_sorted.rs` ;
   GC par tags `remove_staled_tags`).
3. **Paramètres/checkpoints du modèle** → **streams QUIC bidirectionnels directs**
   (`ModelSharing`/`SharableModel`, `save_to_stream` PyTorch, **bf16 depuis mars 2026** pour
   réduire la bande passante). [CONFIRMED — `shared/network/src/lib.rs`, `p2p_model_sharing.rs`]

**Coordinator = machine à états.** Docs : 5 phases (WaitingForMembers → Warmup → RoundTrain →
RoundWitness → Cooldown). Code : **enum `RunState` à 8 états** (Uninitialized, WaitingForMembers,
Warmup, RoundTrain, RoundWitness, Cooldown, Finished, Paused). Un Round bufferise jusqu'à
`NUM_STORED_ROUNDS=4` (pipeline), max `SOLANA_MAX_NUM_CLIENTS=256` clients/epoch,
`SOLANA_MAX_NUM_WITNESSES=32`. Un Epoch = plusieurs Rounds finissant en Cooldown ; si les
participants actifs tombent sous `min_clients`, retour à WaitingForMembers. [CONFIRMED — `coordinator.rs`]

**Deux back-ends de coordination, MÊME crate.** Le state-machine partagé `psyche-coordinator`
tourne soit **ON-chain** (`architectures/decentralized/` — programme **Solana Anchor**,
12 instructions : `init_coordinator`, `tick` (n'importe qui peut faire avancer l'état),
`join_run`, `witness`, `health_check`, `checkpoint`, `set_future_epoch_rates`…), soit
**OFF-chain** (`architectures/centralized/` — `psyche-centralized-server` TCP, tokio+clap).
**Le même code de coordination est découplé du transport de consensus** — prouvé par deux
back-ends interchangeables. [CONFIRMED]

**Assignation déterministe du travail.** Le coordinateur émet une graine aléatoire ; chaque
client dérive **indépendamment** ses indices de batch depuis `(seed, round_index, epoch_index)`.
Aucune donnée n'est poussée par client — juste une graine — et aucun batch n'est entraîné deux
fois. [CONFIRMED]

**Vérification anti-triche = attestation par témoins (OPTIMISTE), PAS de recompute, PAS de ZK.**
- **Commitment** = `{ data_hash: [u8;32], signature: [u8;64] }` (96 octets), hash **SHA-256**
  des résultats d'un Batch, diffusé en P2P.
- **Witnesses** élus aléatoirement (même graine que l'assignation). Chaque témoin construit
  un **filtre de Bloom** (hachage FNV, cible faux-positifs **~1%**) : `participant_bloom`
  (qui a participé) + `broadcast_bloom` (quels résultats ont circulé) + une **racine Merkle**
  sur les broadcasts du round précédent.
- Le coordinateur retient le commitment consensuel par **quorum de témoins 2/3**
  (`WITNESS_QUORUM_RATIO = 2/3`, `select_consensus_commitment_by_witnesses`).
- Faiblesse assumée : les Bloom attestent « j'ai **reçu** un résultat », pas « le calcul est
  **correct** ». Dans les premiers runs, `verification_percent=0` (vérification **désactivée**).
  [CONFIRMED — `commitment.rs`, `bloom.rs`, `witness.rs`, docs]

**Churn / tolérance aux pannes.** États client `Healthy / Dropped / Withdrawn / Ejected`
(Ejected → slashing possible). Health-check on-chain + `trainer_healthy_score_by_witnesses`
(le score monte quand les données d'un client apparaissent dans les Bloom des pairs).
`exited_clients` tracké à part pour les lookups d'index des rounds précédents. [CONFIRMED]

**Partage du modèle pour les joiners** : mode **HubRepo** (des « checkpointer » uploadent sur
HuggingFace après chaque epoch) ou **P2P** (`Checkpoint::P2P` — le joiner **assemble le modèle
en demandant chaque couche à un pair différent**) ; après Cooldown, reset à `Checkpoint::P2P`.
[CONFIRMED]

**Incentives = MONÉTAIRES, on-chain (l'inverse exact de l'invariant SBFB).** Contributions
comptées en « points » partagés à égalité entre clients finissant l'epoch ; un programme
Solana **Treasurer** échange chaque point contre un token SPL (`--earning-rate-total-shared`,
`--slashing-rate-per-client`) ; un **Mining Pool** délègue des fonds pour acheter du compute.
**Chaque client doit détenir un wallet Solana financé et payer des frais SOL** même pour un
run sans récompense. Permissioning par clés Solana (délégués éphémères par machine). [CONFIRMED]

**État réel des runs.** Testnet ~mai 2025 (≈500 K$ de GPU donnés en 44 min). Run phare =
**Consilience 40B** (DeepSeek-v3 + MLA dense sans MoE, **20 000 milliards de tokens** : FineWeb
14T + FineWeb-2 4T + Stack V2 upsampled 1T ; dual-licence CC0/MIT ; checkpoints tous les 500
steps) — décrit comme **le plus grand pre-training décentralisé sur internet publié**, avec une
cohorte cloud hétérogène (Oracle, Lambda, Crusoe, Northern Data) et >11 000 steps avec
ajout/retrait dynamique de nœuds. Successeur annoncé : **Covenant-72B** (arXiv 2603.08163,
~mars 2026). [CONFIRMED — model card HF, presse]

---

## §3 — DisTrO / DeMo (la technique, les chiffres, les limites)

**DeMo (Decoupled Momentum Optimization)** = la formalisation technique de l'idée DisTrO.
Papier **arXiv:2411.19870** (29/11/2024). Auteurs : Bowen Peng, Lizhang Chen, Baiyu Su,
Jeffrey Quesnelle (Nous) + **Diederik P. Kingma** (co-inventeur d'Adam) + Qiang Liu. [CONFIRMED]
⚠️ **« peer-reviewed » est imprécis** : c'est un **preprint arXiv**, pas une publi acceptée en
venue à comité. [cf. §5]

**Algorithme (3 étapes)** : (i) **découpler** la mise à jour de momentum locale par worker
(pas d'all-reduce du gradient complet) ; (ii) **transformée orthonormale rapide (DCT)** sur le
momentum puis **top-k sparsification** (garder les k plus gros coefficients fréquentiels =
composantes « rapides », empiriquement peu nombreuses et de faible rang effectif) ; (iii)
**error feedback** : la composante extraite est **soustraite** du momentum local, donc les
composantes « lentes » sont accumulées et synchronisées **paresseusement**, jamais perdues →
convergence comparable à AdamW+All-Reduce. [CONFIRMED — abstract verbatim + Epoch.ai]

**Chiffres vérifiés (à garder honnêtes)** :
- **DeMo (mesuré, preprint)** : **jusqu'à 85× moins de données/GPU** que AdamW-DDP, comm/step
  réduite jusqu'à ~2 ordres de grandeur, sur modèles **300M et 1B**. [CONFIRMED — abstract]
- **DisTrO (rapport préliminaire août 2024)** : **857×** (transfert par step **74,4 Go → 86,8 Mo**)
  sur **32× H100 en DDP où chaque GPU tenait le modèle ENTIER en VRAM**. [CONFIRMED — aibase]
- **1000×–10000×** = chiffres **préliminaires/marketing** (1000–3000× pré-entraînement,
  jusqu'à 10000× post-training/fine-tuning). [CONFIRMED comme *projections*, pas comme mesures]

**LIMITE FONDAMENTALE (porteuse pour SBFB)** : DeMo/DisTrO/DiLoCo sont **DATA-PARALLEL
uniquement**. Chaque nœud tient le **modèle COMPLET** en VRAM et fait forward+backward local ;
la méthode **compresse la synchro des gradients entre répliques complètes**. Elle **ne permet
PAS** d'entraîner un modèle trop gros pour un nœud — ça, c'est du **model parallelism**
(pipeline/tensor), une autre classe de problème. [CONFIRMED — Epoch.ai]

**Plafond** : l'entraînement décentralisé sur internet reste **~1000× sous le frontier**
(runs 6e22–6e23 FLOP ; réseau actif le plus rapide ~9e17 FLOP/s, ~300× sous les datacenters
frontier). [CONFIRMED]

**Le vrai cousin technique de SBFB n'est PAS DisTrO** : c'est **Petals** (Yandex/BigScience —
inférence pipeline-parallel, passage d'**activations** par couche, hivemind/libp2p) et
**Pluralis** (Protocol Models — **model-parallel over-internet**, compression subspace des
activations forward+backward jusqu'à 100× sur liens 80 Mbps). Filiation vérification : **TOPLOC**
(Prime Intellect, INTELLECT-2) = même famille que les checks activations N0-N3 + RunProof de
SBFB. [CONFIRMED]

---

## §4 — Analyse comparative SBFB (sans complaisance)

### (a) Recouvrement réel
Les deux = **compute P2P en Rust sur hardware hétérogène, sur le transport iroh, entre pairs
à confiance imparfaite, avec besoin de tolérance au churn et de détection des malhonnêtes**.
Mais le problème est **différent** : Psyche = entraînement data-parallel (goulot = bande
passante des **gradients**, 74,4 Go/step) ; SBFB = inférence pipeline-parallel (goulot = latence
des **activations** par hop sur `sbfb/shard/1`). Le recouvrement est donc **au niveau
transport/orchestration/vérification**, PAS au niveau de l'algorithme de compute. Psyche est
même **la preuve externe la plus forte** qu'iroh passe à l'échelle pour du GPU-sur-internet
(40B / 20T tokens, cohorte cloud) — bien au-delà de la cible SBFB (2–5 machines de confiance).

### (b) Ce que SBFB peut APPRENDRE / RÉUTILISER (concret)
1. **Découplage state-machine / transport** : Psyche fait tourner **le même crate coordinator**
   on-chain (Solana) **ou** off-chain (centralized-server). Leçon directe pour l'**orchestrateur
   de session sharding S81 Phase I** : garder la logique (états round/epoch, assignation, quorum)
   dans un **crate pur testable hermétiquement**, découplé du gossip/daemon.
2. **Data-plane à 3 transports par payload** : gossip signé (coordination) / iroh-blobs
   MemStore + tags-GC (résultats éphémères, download **latency-sorted**) / QUIC direct (params
   lourds, bf16). Le `latency_sorted.rs` et `remove_staled_tags` sont des primitives **à copier
   plutôt que réinventer** si SBFB distribue des artefacts/checkpoints volumineux.
3. **Assignation déterministe par graine unique** (le client dérive ses propres indices depuis
   seed+round+epoch) : pattern élégant pour couper le trafic de coordination dans l'assignation
   de shards/tâches.
4. **Attestation optimiste Bloom+Merkle+quorum 2/3** : couche de participation **bon-marché**
   (qui a participé / quels résultats ont circulé) qui **ne recalcule rien** — plus légère que
   le recompute d'activations. SBFB pourrait l'étudier comme **complément** à TOPLOC pour la
   participation (N1/N2), en gardant TOPLOC comme filet de correction, **sans** le volet slashing.
5. **Taxonomie de churn** `Healthy/Dropped/Withdrawn/Ejected` + probes de santé pilotées par
   témoins + score `trainer_healthy_score_by_witnesses` : grille concrète à confronter au
   routing+churn S77 Phase E et au self-heal `runtime.rs`.
6. **Mode P2P de partage modèle** (joiner assemble le modèle en demandant chaque couche à un
   pair **différent**) = prior-art direct pour la distribution de poids pipeline-parallel.

### (c) Ce qui est INCOMPATIBLE avec les invariants SBFB
- **Coordinateur ON-chain Solana** = exactement la centralisation/autorité que SBFB refuse
  (« no central server, no admin »). SBFB reste serverless : orchestrateur in-vivo sur gossip,
  groupe privé **admission-first Ed25519**.
- **Incentives monétaires** (points → token SPL Treasurer, slashing-rate-per-client, mining
  pools, **frais SOL obligatoires**) = **violent l'invariant PO non-monétaire** (kudos =
  réputation non-transférable, jamais de stake/burn/monnaie). **NE JAMAIS proposer d'adopter le
  modèle token.** À citer comme **repoussoir argumenté** : Psyche résout la confiance par
  l'économie on-chain ; SBFB la résout autrement (admission-first + TOPLOC).
- **Stack ML PyTorch/libtorch** (`tch`, fork Nous, dépendance CUDA lourde, clusters homogènes) vs
  **fork llama.cpp GGUF bit-exact CPU/CUDA/Metal** de SBFB — confirme que le choix SBFB est plus
  léger/portable pour un rig hétérogène (RTX 5080 + Mac M2 Metal).

### (d) Menaces / opportunités de positionnement
- **Menace** : Psyche est **matériellement en avance opérationnellement** (run 40B live mai 2025,
  cohorte cloud réelle, v0.2.0, Covenant-72B en préparation) et **surfinancé** (65 M$, Paradigm).
  Le récit « crypto-AI décentralisée » capte l'attention et les GPU.
- **Opportunité** : Psyche est **le contre-modèle parfait** pour affirmer l'identité SBFB
  « communs humanistes » vs « marché d'incitation crypto ». **Training vs inference sont
  complémentaires, pas concurrents** — les artefacts open-weight de Psyche (Consilience 40B,
  Hermes 4.3, CC0/MIT) sont précisément le genre de modèles que **l'inférence shardée de SBFB
  pourrait SERVIR**. Le récit « source verifiable » de SBFB rime avec le « publicly-verifiable
  pretraining run » de Consilience : la vérifiabilité-comme-produit résonne dans l'espace.

### (e) 3–6 actions concrètes candidates
1. **Avant S81 Phase I (orchestrateur)** : lire `shared/coordinator/src/coordinator.rs`
   (enum `RunState`, `NUM_STORED_ROUNDS`, `select_consensus_commitment_by_witnesses`) et la
   séparation crate-pur / back-end — valider le découpage logique/transport de l'orchestrateur SBFB.
2. **Avant toute distribution de blobs volumineux (S81 J / S82)** : lire
   `shared/network/src/latency_sorted.rs` + `lib.rs` (iroh-blobs MemStore, `add_downloadable`,
   Downloader latency-sorted, `remove_staled_tags`). ⚠️ **Skew de version** : Psyche = iroh 0.97/0.99,
   SBFB = **iroh 1.0.1** → les API ont changé, **les patterns transfèrent, pas le copier-coller**.
3. **Pour la conception witness N1/N2** : lire `shared/core/src/bloom.rs` + `commitment.rs` +
   `shared/client/src/state/witness.rs` ; évaluer une couche d'attestation Bloom+Merkle de
   participation en **complément** de TOPLOC, **sans** slashing économique.
4. **Assignation de shards** : envisager une assignation déterministe **seed → indices** dérivée
   côté worker (comme `assign_data_for_state`) pour réduire le trafic de coordination.
5. **Doctrine SBFB** : verser Psyche comme **référence « ce qu'on refuse »** dans THREAT_MODEL /
   vision (coordinateur Solana + Treasurer + slashing = anti-pattern non-monétaire assumé).
6. **Veille** : archiver DisTrO/DeMo comme référence **hors-scope tant que la cible reste
   l'inférence** ; ne devient pertinent que si un axe fine-tuning distribué émerge — et seulement
   en data-parallel réplique-complète, ce qui **casse** le pari matériel SBFB.

---

## §5 — Claims RÉFUTÉS / NON-VÉRIFIÉS (honnêteté)

- **RÉFUTÉ** — « Hermes 4 405B post-entraîné sur un dataset de **60 Md tokens** » : le tech report
  (arXiv 2508.18255) dit **~19 Md tokens / ~5M échantillons**. Les 60 Md appartiennent à **Hermes
  4.3**, pas à Hermes 4 (erreur ~3×). Le reste du claim (base Llama-3.1-405B, mode hybride, poids
  HF, licence Llama) est correct.
- **INCOHÉRENCE dans les matériaux de Nous** : le blog Hermes 4.3 cite une baseline « 1,2 Md
  tokens » pour Hermes 4 qui contredit les 19 Md du tech report. Non résolu côté Nous ; à ne pas
  trancher.
- **IMPRÉCIS** — DeMo qualifié de « peer-reviewed » : c'est un **preprint arXiv** (v1 2024-11-29,
  v2 révisé 2026-02-06), pas de venue à comité trouvée. Dire « papier technique formel ».
- **NON-VÉRIFIÉ depuis la presse (mais résolu par le code)** — le lecteur corp (source 2) **n'a
  pas** pu confirmer la lib P2P du data-plane depuis blogs/articles. **La lecture du code (source
  3) confirme iroh** directement (Cargo.toml). La coordination/consensus = **Solana** ; le
  data-plane = **iroh**. Aucune contradiction : ce sont deux couches distinctes.
- **NON-VERBATIM** — le mot « gossip » n'apparaît pas verbatim dans les docs (elles disent
  « data-sharing »/« broadcast ») ; le **code** confirme `iroh-gossip` — le fait « transport =
  iroh » est exact, seul le descripteur « gossip » était inféré côté docs.
- **SOFT SPOTS (n'affectent aucun verdict)** : macOS explicitement « single-GPU » (docs confirment
  seulement « dev-only Metal ») ; la phrase exacte « `max_round_train_time` bas exclut les GPU
  lents » (le paramètre existe, la causalité précise non surfacée) ; signature exacte
  `Client::new(id)` et type de retour `BTreeMap<BatchId, Identity>` (présence de fonction
  confirmée, type non cité verbatim).
- **DÉTAILS D'IMPLÉM MINEURS** : le hash SHA-256 de `TransmittableDistroResult` est une **méthode
  calculée**, pas un champ stocké ; `remove_staled_tags` vit dans `lib.rs` (pas `serialized_distro.rs`).
  Substance intacte.
- **DATES** : Hermes 4 = fin août 2025 (arXiv 25/08, blogs 26–27/08) — « 26/08 » est défendable,
  pas canonique. Hermes 4.3 = ~déc. 2025 (des caches montrent août 25, le récit Psyche pointe déc.).
- **TOKEN** : aucun token officiel Nous au 13/04/2026 (le « proprio vs SOL » est daté 2025 ; une
  source BSCNews faible évoque « Q2 2026 » sans émission constatée).

**Sources primaires principales** : `docs.psyche.network` (intro/glossary/general-workflow/
data-provider/model-sharing/client-faq/join-run + print.html) ; `github.com/PsycheFoundation/psyche`
(Cargo.toml, `shared/{core,coordinator,network,client}`, `architectures/{centralized,decentralized}`) ;
arXiv **2411.19870** (DeMo), **2508.18255** (Hermes 4) ; model card HF `PsycheFoundation/consilience-40b-CqX3FUm4` ;
`github.com/NousResearch/DisTrO` ; `epoch.ai/gradient-updates/...` ; Petals arXiv 2209.01188/2312.08361 ;
Pluralis arXiv 2506.01260 ; Fortune/SiliconANGLE/The Block/Gate.com (funding/token).
