# R&D: Partage d'un gros LLM entre ordinateurs d'utilisateurs

**Domaine:** inference LLM distribuee entre ordinateurs d'utilisateurs distants
**Date:** 2026-05-21
**Statut:** recherche R&D, pas scope S67-S69
**Confidence:** HIGH sur la faisabilite technique, MEDIUM sur les performances
reelles avant benchmark multi-machines

---

## 0. Verdict

Oui, partager un gros LLM entre plusieurs ordinateurs d'utilisateurs est
techniquement possible.

Le bon verdict n'est pas binaire:

```text
possible techniquement: oui
possible entre utilisateurs distants: oui, sous contraintes
produit fiable generaliste: pas encore
bon premier spike SBFB: oui, si limite a groupe prive + benchmarks
```

Les projets open source prouvent deja plusieurs variantes:

- Petals prouve l'inference/fine-tuning distribuee sur Internet avec des
  appareils heterogenes et instables.
- llama.cpp RPC prouve l'offload de devices distants pour GGUF/ggml, mais sa
  securite brute est insuffisante.
- LocalAI prouve qu'un wrapper P2P autour de llama.cpp RPC est possible.
- exo prouve l'UX "mes machines deviennent un cluster" avec decouverte et
  placement topology-aware, surtout Apple/MLX.
- distributed-llama/prima.cpp prouvent que les home clusters heterogenes sont
  un vrai axe de recherche.
- vLLM/Ray/SGLang prouvent le multi-node serieux, mais supposent plutot un
  cluster controle, stable, rapide, et isole.

La solution SBFB a creer n'est pas un nouveau moteur tensoriel. C'est une
**couche protocolaire neutre de controle, preuve, admission et manifests** au
dessus de backends specialises:

```text
SBFB User-Sharded LLM =
  private compute group
  + signed worker capabilities
  + measured network topology
  + model artifact hashes
  + shard/session manifest
  + backend launch profile
  + secure tunnel / private data plane
  + run proofs / benchmark proofs
  + fallback and shutdown evidence
```

Ce document traite le cas precis: **un seul gros modele partage entre plusieurs
ordinateurs d'utilisateurs**. Il ne traite pas le cas plus simple ou chaque
ordinateur execute son propre modele sur une tache independante.

---

## 1. Definitions

### 1.1 Ce que veut dire "partager un gros modele"

Dans ce document, "partager un gros modele" signifie:

```text
un modele unique
dont les poids et/ou les blocs/layers sont repartis
sur plusieurs ordinateurs d'utilisateurs
et qui produit une seule reponse comme un seul LLM logique
```

Exemple D&D:

```text
5 joueurs
5 PC gamers
1 modele 70B/120B quantifie
chaque PC heberge une partie des layers ou des shards
l'app voit un seul "maitre du jeu IA"
```

### 1.2 Ce que ce n'est pas

Ce n'est pas:

- du load balancing entre plusieurs modeles complets;
- une federation d'endpoints Ollama ou vLLM;
- une queue de taches batch independantes;
- "tous les PC deviennent une carte GPU unique";
- un mode public trustless ou n'importe quel GPU inconnu rejoint;
- une garantie de confidentialite des prompts face aux workers;
- une primitive S67-S69.

Ces autres modes restent utiles, mais ils ne repondent pas a la question precise
"un gros LLM partage entre ordinateurs".

---

## 2. Synthese technique

### 2.1 Trois familles d'architectures

| Famille | Principe | WAN domestique | Verdict |
|---------|----------|----------------|---------|
| Pipeline / layer split | Chaque worker heberge des blocs/layers consecutifs | Possible | Meilleur axe utilisateurs distants |
| Tensor parallel | Chaque couche est decoupee entre GPUs, avec collectives frequentes | Mauvais hors cluster | R&D metro/datacenter seulement |
| Endpoint federation | Chaque worker heberge un modele complet et recoit des requetes | Tres bon | Pas un modele unique partage |

Pour le cas demande, la voie credible est:

```text
pipeline/layer split ou Petals-like block serving
```

Le tensor parallel fin est l'option la plus dangereuse sur Internet, car il
multiplie les synchronisations latency-sensitive a l'interieur des couches.

### 2.2 Pourquoi Internet/fibre peut marcher dans certains cas

Le reseau domestique fibre moderne peut fournir assez de debit pour:

- transferer des shards de modele;
- precharger des caches;
- envoyer des activations intermediaires;
- streamer des resultats;
- synchroniser des logs/proofs.

Mais la latence et le jitter restent les vrais ennemis:

```text
debit eleve = utile pour les poids, caches, artifacts
latence faible = indispensable pour tokens interactifs
jitter faible = indispensable pour eviter les stalls
```

Pour D&D, l'objectif n'est pas de battre un datacenter. L'objectif est une
experience acceptable:

```text
TTFT: quelques secondes acceptable
decode: 2-8 tokens/s peut etre jouable
latence par scene: 5-20 s acceptable si streaming/prefetch
```

Pour du chat ultra-reactif, le meme setup serait mauvais.

### 2.3 Cout reseau des activations

Pour un split par couches, le cout a chaque frontiere de shard peut etre estime:

```text
activation_bytes ~= tokens_contexte * hidden_size * bytes_precision
```

Ordres de grandeur:

| Modele | Hidden size approx | BF16 par token/frontiere | 4k contexte | 8k contexte |
|--------|--------------------|--------------------------|-------------|-------------|
| 70B class | 8192 | ~16 KB | ~64 MB | ~128 MB |
| 405B class | 16384 | ~32 KB | ~128 MB | ~256 MB |

Pendant le decode, le debit pur est moins impressionnant:

```text
70B, 10 tokens/s, BF16 ~= 160 KB/s/frontiere ~= 1.3 Mbit/s/frontiere
```

Donc le decode interactif est souvent limite par RTT/jitter et par le worker le
plus lent, pas par le debit fibre maximal. Le prefill et les longs contextes,
eux, peuvent consommer des centaines de MB par frontiere.

### 2.4 Pourquoi D&D est meilleur que du coding interactif

D&D accepte:

- tour par tour;
- temps de narration;
- pre-generation;
- streaming lent;
- pauses naturelles;
- gros contexte de campagne;
- usage occasionnel du "grand cerveau" pour scenes importantes.

Donc un modele partage lent peut etre utile si l'app masque la latence.

---

## 3. Etat open source

### 3.1 Petals / Hivemind

**Sources:**

- https://github.com/bigscience-workshop/petals
- https://arxiv.org/abs/2312.08361

Ce que ca prouve:

- inference distribuee sur Internet;
- appareils heterogenes;
- appareils qui rejoignent/quittent;
- partition automatique de LLMs 50B+;
- Llama 2 70B et BLOOM 176B dans les evaluations;
- performance interactive possible dans certains regimes.

Le papier indique que des LLMs 50B+ peuvent tourner efficacement sur des
appareils geodistribues avec reseau grand public, et que Petals gere deux
problemes clefs: deconnexions abruptes et partitionnement entre machines
heterogenes.

Fit SBFB:

```text
Meilleure preuve scientifique du concept "gros LLM entre utilisateurs".
Meilleur modele mental pour un backend WAN.
Pas assez integre au protocole SBFB tel quel.
```

Conclusion agent OSS:

```text
Petals/Hivemind est le seul precedent open source vraiment concu pour Internet
geodistribue et pairs instables. Les autres projets ciblent surtout LAN,
cluster prive, Thunderbolt/RDMA, ou datacenter.
```

Risques:

- confidentialite: les workers peuvent voir des etats intermediaires;
- maturite produit variable;
- modele public swarm incompatible avec groupes prives SBFB par defaut;
- incentive/reputation non aligne SBFB;
- integration model zoo / GGUF / app UX a reconstruire.

### 3.2 llama.cpp RPC

**Sources:**

- https://raw.githubusercontent.com/ggml-org/llama.cpp/master/tools/rpc/README.md
- https://raw.githubusercontent.com/ggml-org/llama.cpp/master/SECURITY.md
- https://github.com/ggml-org/llama.cpp/security/advisories/GHSA-j8rj-fmpv-wcxw

Ce que ca fournit:

- `rpc-server` expose des devices ggml distants;
- `llama-cli` / `llama-server` peuvent utiliser `--rpc host:port`;
- repartition poids + KV cache selon memoire disponible;
- override possible par `--tensor-split`;
- cache local RPC pour eviter de retransmettre de gros tenseurs;
- RDMA possible sur Linux/RoCEv2 si support `libibverbs`.

Fit SBFB:

```text
Meilleur prototype GGUF/PC gamer.
Mauvais backend brut expose a Internet.
```

Point securite bloquant:

- la doc RPC officielle marque le backend comme proof-of-concept, fragile et
  insecure;
- la security policy dit de ne pas utiliser RPC/llama-server sur reseau non
  fiable et de chiffrer les donnees reseau;
- l'advisory GHSA-j8rj-fmpv-wcxw documente une RCE non authentifiee sur le
  port RPC si un attaquant peut l'atteindre.

Donc SBFB ne doit jamais exposer `rpc-server` brut a un reseau public. Il faut:

- tunnel prive;
- allowlist d'identites;
- firewall;
- version pinning/quarantine;
- profil R&D explicite.

### 3.3 LocalAI P2P / workers

**Sources:**

- https://localai.io/features/distribute/
- https://localai.io/features/distributed-mode/index.html

Ce que ca fournit:

- federation: routage d'une requete vers un worker;
- worker mode: splitting weights via llama.cpp RPC;
- mode P2P avec token;
- mode production avec PostgreSQL/NATS;
- file transfer pour model files/configs;
- vLLM distributed mode operator-launched dans certains cas.

Fit SBFB:

```text
Bonne preuve qu'un wrapper P2P autour de llama.cpp RPC est possible.
Pas une couche de preuve/provenance SBFB.
```

LocalAI est important parce qu'il montre que "P2P + llama.cpp RPC + UX web" est
deja faisable. SBFB peut s'en inspirer, mais doit apporter:

- identities SBFB;
- manifests signes;
- hashes modeles;
- consentement worker;
- reputation/quarantine;
- preuves de benchmark.

### 3.4 exo

**Source:** https://github.com/exo-explore/exo

Ce que ca fournit:

- decouverte automatique de devices;
- placement topology-aware;
- tensor parallelism;
- MLX distributed;
- API OpenAI/Claude/Ollama compatible;
- support fort Apple Silicon / Thunderbolt / RDMA.

Fit SBFB:

```text
Meilleure reference UX locale.
Tres interessant pour "mes appareils deviennent un cluster".
Pas le backend universel PC gamer WAN.
```

exo prouve une direction importante: l'utilisateur ne devrait pas configurer
chaque shard a la main. Le systeme doit mesurer la topologie et assigner les
roles.

### 3.5 distributed-llama

**Source:** https://github.com/b4rtaz/distributed-llama

Ce que ca fournit:

- root node + workers;
- split RAM entre nodes;
- Linux/macOS/Windows;
- CPU oriented / AVX2 / ARM;
- tensor parallelism et synchronisation Ethernet;
- licence MIT.

Fit SBFB:

```text
Reference utile pour home devices.
Contraintes trop specifiques pour devenir backend par defaut.
```

Les contraintes importantes:

- topologie root/worker;
- nombre de workers contraint (`2^n - 1`);
- formats/quantizations propres;
- plutot Ethernet/local que WAN arbitraire.

### 3.6 prima.cpp

**Source:** https://arxiv.org/abs/2504.08791

Ce que ca prouve:

- 30B-70B sur home clusters heterogenes;
- CPUs/GPUs mixtes;
- RAM/VRAM insuffisante;
- liens Wi-Fi;
- OS heterogenes;
- scheduler heterogeneity-aware;
- pipeline/offload optimise.

Fit SBFB:

```text
Tres important comme reference R&D "home cluster heterogene".
Pas encore une brique SBFB directe tant que code/integration ne sont pas valides.
```

prima.cpp renforce l'idee que le sujet n'est pas fantasque: il y a un axe de
recherche actif sur les gros modeles dans des clusters domestiques imparfaits.

### 3.7 vLLM + Ray

**Sources:**

- https://docs.vllm.ai/en/v0.10.0/serving/distributed_serving.html
- https://docs.ray.io/en/latest/serve/llm/user-guides/cross-node-parallelism.html

Ce que ca fournit:

- tensor parallel;
- pipeline parallel;
- Ray pour multi-node;
- OpenAI-compatible serving;
- scheduling/placement;
- performance production.

Fit SBFB:

```text
Backend serieux pour cluster controle.
Mauvais premier choix pour utilisateurs distants heterogenes.
```

Contraintes documentees:

- vLLM recommande `VLLM_HOST_IP` sur segment reseau prive;
- trafic inter-node non chiffre;
- format inter-node exploitable si un attaquant a acces au reseau;
- tensor parallel cross-node veut InfiniBand/GDRDMA plutot que socket TCP brut;
- model weights doivent exister sur tous les nodes ou stockage partage.

### 3.8 SGLang

**Source:** https://sgl-project-sglang-93.mintlify.app/deployment/multi-node

Ce que ca fournit:

- tensor parallel multi-node;
- expert parallel;
- prefill-decode disaggregation;
- router/gateway;
- high-performance serving.

Fit SBFB:

```text
Tres bon backend laboratoire/datacenter.
R&D seulement pour utilisateurs distants.
```

Contraintes documentees:

- plusieurs nodes GPU;
- interconnect rapide: InfiniBand, RoCE, ou Ethernet haut debit;
- topologie reseau coherente;
- stockage partage ou poids synchronises;
- NCCL recent.

### 3.9 GPUStack

**Sources:**

- https://github.com/gpustack/gpustack
- https://docs.gpustack.ai/0.3/user-guide/inference-backends/

Ce que ca fournit:

- manager de clusters GPU;
- orchestration vLLM/SGLang/TensorRT-LLM/llama-box;
- UI workers;
- scheduling et optimisation;
- support backends multiples.

Fit SBFB:

```text
Bon comparateur/possible external backend.
Pas la primitive protocolaire SBFB.
```

GPUStack peut etre utilise par un groupe deja administre, mais il ne remplace
pas les primitives SBFB de groupe prive, preuve, app distribution, provenance et
consentement.

---

## 4. Architecture SBFB proposee

### 4.1 Nom

```text
SBFB User-Sharded LLM Mode (USL)
```

Nom produit possible:

```text
Party Sharded LLM
Community Sharded LLM
Private Sharded Model Session
```

Nom technique recommande:

```text
User-Sharded LLM Mode
```

### 4.2 Neutralite du protocole

Le protocole reste neutre si et seulement si il ne contient pas les regles
metier de D&D, Foret, Factory ou Babel.

Il doit porter:

```text
qui demande du compute
qui claim un shard
quel modele exact est attendu
quel artifact hash est charge
quelles ressources sont consenties
quelle session est lancee
quel resultat/proof revient
quel worker est en faute ou quarantined
```

Il ne doit pas porter:

```text
les prompts D&D
les regles de campagne
les scores de deforestation
la politique de merge Factory
la logique de traduction Babel
une economie GPU figee
un backend impose
un modele impose
```

Regle:

```text
Le protocole orchestre et prouve.
L'app interprete.
Le backend calcule.
```

### 4.3 Couche protocole neutre

Le protocole doit rester neutre. Il ne doit pas connaitre D&D, foret, Factory
ou Babel. Il doit seulement connaitre des contrats verifiables.

Primitives:

```text
ComputeGroup
WorkerCapability
NetworkProbe
ModelArtifactManifest
ShardPlan
ShardAssignment
ShardedSessionManifest
BackendLaunchProfile
ShardClaim
ShardHeartbeat
RunProof
BenchmarkProof
ShutdownProof
QuarantineEntry
```

Ces primitives servent a tout:

- D&D: maitre du jeu IA;
- foret: modele vision geospatial;
- Factory: gros code/reasoning model;
- Babel: gros modele traduction;
- recherche juridique: gros modele local confidentiel;
- rendu/generation: diffusion ou video.

### 4.4 Couche app

L'app choisit:

- le modele;
- les prompts;
- la memoire;
- les regles metier;
- la tolerance latence;
- les seuils de qualite;
- le fallback UX.

Exemple D&D:

```text
App D&D:
  - campagne
  - personnages
  - lore
  - regles
  - initiative/combat
  - voix/cartes/PNJ

Protocole SBFB:
  - 5 membres du groupe
  - model_digest
  - shard assignments
  - capabilities
  - session run proof
```

### 4.5 Couche backend

Backends candidats:

```text
R&D A: Petals-like backend
R&D B: llama.cpp RPC wrapped
R&D C: LocalAI p2p-llama-cpp-rpc adapter
R&D D: distributed-llama/prima.cpp comparison
R&D E: vLLM/SGLang only for controlled high-speed groups
```

SBFB ne doit pas reimplementer:

- CUDA kernels;
- NCCL;
- Ray;
- vLLM scheduler;
- MLX distributed;
- ggml graph execution.

SBFB doit implementer:

- admission;
- manifests;
- proofs;
- capabilities;
- model artifact distribution;
- tunnels/allowlists;
- lifecycle/fallback;
- UI consent;
- audit.

---

## 5. Data plane et control plane

### 5.1 Control plane SBFB/Iroh

Iroh/SBFB est adapte a:

- identite de noeud;
- invitations;
- tickets;
- QUIC chiffre;
- ALPN custom;
- iroh-blobs pour model artifacts;
- hashes BLAKE3 et streaming verifie;
- iroh-docs/gossip pour manifests, claims, results, proofs.

Iroh n'est pas automatiquement le meilleur transport pour chaque activation
tensorielle. Pour le data plane, on garde un backend specialise ou un tunnel
dedie.

### 5.2 Data plane backend

Le data plane transporte:

- activations;
- KV/cache metadata;
- RPC graph commands;
- tensors;
- prefill/decode messages;
- heartbeats low-level.

Options:

| Data plane | Usage | Risque |
|------------|-------|--------|
| backend raw TCP dans WireGuard/Tailscale | plus simple | depend du tunnel |
| Iroh custom ALPN | controle SBFB fort | perf a benchmarker |
| QUIC backend custom | bon WAN | gros travail |
| Ray/NCCL | cluster controle | mauvais WAN public |
| libp2p/Hivemind | Petals-like | integration lourde |

Regle:

```text
Control plane SBFB obligatoire.
Data plane backend remplacable.
```

---

## 6. Modeles et sizing

### 6.1 Ordres de grandeur

Approximation:

```text
Q4 weights ~= 0.5 byte/param + overhead
Q5/Q6 weights ~= 0.65-0.8 byte/param + overhead
KV cache augmente avec contexte, batch et architecture
MoE total != MoE active compute
```

Exemples grossiers:

| Modele | Q4 approx | Remarque |
--------|-----------|----------|
| 70B | 35-45 GB + KV | cible realiste 5 PC |
| 120B | 60-80 GB + KV | possible si 5 gros GPUs |
| 235B | 120-160 GB + KV | tres ambitieux, surtout MoE |
| 405B | 200-260 GB + KV | R&D avancee |

### 6.2 Profils 5 PC

| Groupe | VRAM brute | Cible realiste |
|--------|------------|----------------|
| 5 x 12 GB | 60 GB | 30B-70B quantifie, contexte limite |
| 5 x 16 GB | 80 GB | 70B Q4, 100B tres serre |
| 5 x 24 GB | 120 GB | 70B confortable, 120B plausible |
| 5 x 32 GB | 160 GB | 120B bon candidat, 235B R&D |

La VRAM brute n'est pas totalement disponible. Il faut garder marge pour:

- KV cache;
- buffers;
- backend runtime;
- OS/driver;
- fragmentation;
- contexte long.

### 6.3 Modeles candidats pour D&D

Le document modele dedie peut evoluer, mais pour le sharding:

```text
Premier spike: 70B quantifie
Deuxieme spike: 100B/120B quantifie
Troisieme spike: MoE 200B+ si backend adapte
```

Critere plus important que taille brute:

- bonne prose;
- instruction-following;
- contexte long;
- tool calling;
- stable en temperature faible;
- support backend sharding.

---

## 7. Contraintes reseau

### 7.1 Profil minimal R&D

Pour tenter un gros modele partage entre utilisateurs:

```text
uplink soutenu par worker: >= 1 Gbit/s
profil preferable: 5-8 Gbit/s si disponible
RTT par frontiere consecutive: <= 20 ms ideal
RTT acceptable D&D lent: <= 40 ms si pipeline coarse-grain
jitter p95: <= 10 ms hard, <= 1-3 ms preferable
packet loss: proche de zero
connexion: Ethernet filaire, pas Wi-Fi
NAT: hole punching ou tunnel stable
CGNAT: relay/tunnel obligatoire si direct impossible
```

Si RTT > 80 ms, le mode peut encore fonctionner pour scenes lentes, mais il
faut s'attendre a une experience degradee.

Go/no-go reseau plus strict:

```text
GO prototype:
  2-3 machines au debut
  p95 RTT/frontiere <= 20-30 ms
  packet loss <= 0.1%
  direct UDP/IPv6 ou NAT traversal stable
  pas de TURN/relay dans le hot path

NO-GO production:
  hot path depend d'un relais
  CGNAT massif sans IPv6/direct path
  p95 RTT/frontiere > 40 ms
  jitter > 10-20 ms
  prompts sensibles sur workers consumer non attestes
```

### 7.2 Mesures obligatoires

Avant de lancer une session:

```text
latency_probe:
  min/median/p95 RTT
  jitter p95
  packet loss
  sustained throughput
  route stability
  NAT type
  relay usage yes/no

gpu_probe:
  VRAM free
  backend support
  model shard cache
  estimated tokens/s local microbench
```

Le systeme ne doit jamais accepter "fibre" comme preuve. Il doit mesurer.

### 7.2.1 NAT, CGNAT, relais

Contraintes WAN domestique:

- IPv6 public ou UDP direct facilite fortement le mode;
- CGNAT force souvent un relais ou un tunnel;
- un relais dans le hot path ajoute latence, cout bande passante, metadata, et
  point de defaillance;
- deux utilisateurs proches geographiquement peuvent avoir une mauvaise route
  inter-ISP;
- le scheduler doit se baser sur mesures actives, pas ville/pays declares.

WebRTC/ICE/STUN/TURN sont utiles pour decouverte et NAT traversal. QUIC est
plus adapte a un data plane natif SBFB parce qu'il donne UDP, chiffrement,
streams multiplexes et migration/NAT rebinding. Aucun de ces transports ne
supprime la latence physique ni le bufferbloat domestique.

### 7.3 Pourquoi pipeline > tensor parallel en WAN

Pipeline/layer split:

- un message principal entre blocs;
- plus tolerant a la latence;
- plus simple a scheduler;
- peut streamer lentement;
- accepte des machines heterogenes.

Tensor parallel:

- collectives frequentes;
- all-reduce/all-gather;
- depend fortement bande passante + latence;
- sensible au plus lent;
- meilleur sur NVLink/InfiniBand/RoCE que sur Internet.

Donc:

```text
WAN users: pipeline/layer split
LAN/metro/fiber excellent: pipeline first, tensor only if measured
cluster lab/datacenter: tensor/pipeline via vLLM/SGLang
```

---

## 8. Securite et confiance

### 8.1 Invariants

- Groupe prive explicite.
- Aucun worker inconnu.
- Aucune exposition brute de `rpc-server`/Ray/NCCL/vLLM internode sur Internet.
- Model artifact hash-pinned.
- Backend version allowlist.
- Quarantine des versions vulnerables.
- Consentement GPU local et revocable.
- Pas de secrets app dans les prompts si workers non pleinement trusted.
- Pas de claim "confidentiel" si activations/prompts sont visibles par workers.
- Shutdown evidence obligatoire.

Transport chiffre ne suffit pas:

```text
TLS/QUIC/WireGuard protege contre ISP, relais et observateurs reseau.
Il ne protege pas contre le worker qui calcule le shard.
```

Un worker peut voir les activations dont il a besoin. Les activations
intermediaires doivent etre classees comme donnees sensibles, car les attaques
d'inversion/split learning montrent qu'elles peuvent fuir des informations sur
le prompt ou l'entree.

### 8.2 Threat model

Menaces:

- worker malveillant modifie ses activations;
- worker exfiltre prompt/campagne;
- worker ment sur sa VRAM;
- worker accepte puis disparait;
- worker lance un backend vulnerable;
- autre machine scanne le port RPC;
- host compromis capture model shards;
- participant triche sur benchmark;
- prompt injection pousse l'app a reveler secrets;
- side-channel GPU local.

### 8.3 Verification possible

Ce qu'on peut verifier:

- identite du worker;
- signature de claim/result;
- hash du modele;
- version backend;
- presence du shard;
- benchmark micro;
- result consistency sur prompts de calibration;
- logs de lancement;
- network probe;
- shutdown.

Ce qu'on ne peut pas verifier parfaitement:

- qu'un worker n'a pas lu les activations;
- qu'un worker n'a pas capture le prompt;
- que le GPU execute exactement sans instrumentation hardware;
- que la sortie creative est "correcte";
- que deux runs generatifs non deterministes devraient matcher.

Donc le mode est:

```text
trusted/private group first
public swarm later only with privacy caveats
```

Attestation:

- une vraie confidentialite forte demanderait TEE/remote attestation;
- en 2026, c'est surtout realiste sur materiel datacenter recent, pas sur GPUs
  consumer ordinaires;
- pour les PC gamers, SBFB doit parler de **groupe de confiance + preuves
  d'execution**, pas de confidential computing.

---

## 9. Fault tolerance

### 9.1 Failure modes

- worker drop pendant prefill;
- worker drop pendant decode;
- controller crash;
- shard assignment impossible;
- model hash mismatch;
- backend port deja utilise;
- NAT change;
- relay fallback trop lent;
- jitter spike;
- GPU OOM;
- thermal throttling;
- driver reset;
- participant retire consentement.

Modele de performance:

```text
latence_generation ~= somme(latences_frontieres) + temps_compute_du_plus_lent
```

Plus il y a de shards residentiels dans le chemin critique, plus l'experience
depend du plus mauvais lien et du plus mauvais worker. La premiere R&D doit
donc commencer avec 2-3 machines, puis seulement monter a 5.

### 9.2 Fallbacks

Pour une app D&D:

```text
fallback 1: pause session, reconnect same worker
fallback 2: reassign shard, reload weights, replay context
fallback 3: degrade to smaller model on fewer nodes
fallback 4: switch to endpoint federation
fallback 5: local small GM model while big model recovers
```

Pour un modele partage, le plus dur est le KV/cache et l'etat courant. Si un
worker disparait, le systeme doit souvent:

- recharger le shard ailleurs;
- reconstruire/rejouer le contexte;
- accepter une pause visible;
- ou abandonner la generation en cours.

### 9.3 Redondance

Redonder chaque shard coute cher:

```text
2x redundancy = environ 2x VRAM pour les shards redondes
```

Donc MVP:

- pas de redondance permanente;
- warm spare facultatif;
- checkpoints session;
- fallback smaller model.

---

## 10. Protocol extension proposee

### 10.1 ShardedSessionManifest

```json
{
  "schema_version": 1,
  "session_id": "uuid",
  "mode": "user_sharded_llm",
  "group_id": "private-group-id",
  "controller_node": "node-a",
  "backend": "petals_like|llamacpp_rpc|localai_p2p|distributed_llama",
  "model": {
    "name": "model-name",
    "format": "gguf|safetensors|custom",
    "artifact_hashes": ["blake3:..."],
    "tokenizer_hash": "blake3:...",
    "chat_template_hash": "blake3:..."
  },
  "network_profile": {
    "max_rtt_ms": 40,
    "max_jitter_ms": 3,
    "min_uplink_mbps": 1000,
    "relay_allowed": false
  },
  "security": {
    "private_group_only": true,
    "raw_public_ports_allowed": false,
    "artifact_hash_required": true,
    "version_allowlist_required": true
  }
}
```

### 10.2 WorkerCapability

```json
{
  "schema_version": 1,
  "node_id": "ed25519-pubkey",
  "operator_id": "member-id",
  "accelerators": [
    {
      "vendor": "nvidia",
      "name": "RTX 4090",
      "vram_mb_total": 24576,
      "vram_mb_free": 21000,
      "backend_support": ["llamacpp_rpc_cuda", "petals_cuda"]
    }
  ],
  "network": {
    "uplink_mbps_observed": 6200,
    "rtt_ms_to_controller_p50": 12.4,
    "rtt_ms_to_controller_p95": 18.1,
    "relay_used": false
  },
  "consent": {
    "max_minutes": 180,
    "max_watts": 350,
    "allow_model_shards": true,
    "allow_prompt_exposure": false
  },
  "signature": "..."
}
```

### 10.3 ShardAssignment

```json
{
  "schema_version": 1,
  "session_id": "uuid",
  "node_id": "ed25519-pubkey",
  "assignment": {
    "role": "layer_worker",
    "layers": [0, 1, 2, 3, 4, 5],
    "shard_hashes": ["blake3:..."],
    "kv_cache_policy": "local_ephemeral",
    "fallback_node": "optional-node-id"
  },
  "launch_profile_hash": "blake3:...",
  "signature": "..."
}
```

### 10.4 RunProof

```json
{
  "schema_version": 1,
  "session_id": "uuid",
  "model_digest": "blake3:...",
  "backend": "petals_like",
  "prompt_profile_hash": "blake3:...",
  "metrics": {
    "ttft_ms": 0,
    "decode_tokens_per_sec": 0,
    "p95_token_latency_ms": 0,
    "network_rx_mb": 0,
    "network_tx_mb": 0,
    "worker_drop_count": 0
  },
  "participants": ["node-a", "node-b", "node-c"],
  "signature": "..."
}
```

---

## 11. R&D plan

### Phase A - OSS refresh and threat model

Output:

- source index;
- threat model USL;
- backend shortlist;
- privacy caveats;
- protocol primitive draft.

### Phase B - Network probe prototype

Output:

- `network_probe` CLI;
- NAT/relay detection;
- RTT/jitter/loss report;
- capability entry signed.

### Phase C - Toy sharded backend

Output:

- 1B/7B toy model split across 2-3 machines;
- deterministic prompt;
- signed RunProof;
- shutdown proof.

### Phase D - 70B private group benchmark

Output:

- 3-5 PCs;
- 70B Q4 target;
- D&D-style prompt suite;
- TTFT/tokens/s/jitter/drop metrics;
- compare to single-node offload and endpoint federation.

### Phase E - Go/no-go

Decision:

```text
productize private sharded session
or keep R&D
or abandon sharding and keep endpoint/task modes
```

---

## 12. Go/no-go criteria

### Go

- 70B quantifie fonctionne sur 3-5 machines;
- TTFT acceptable pour D&D (< 15 s target R&D);
- decode >= 2 tokens/s minimum, >= 5 tokens/s target;
- un worker drop est gere sans perdre toute la campagne;
- aucun port brut expose publiquement;
- model hashes verifies;
- session proof signee;
- UX de consentement claire;
- fallback smaller model fonctionne.

### No-go

- besoin d'un reseau quasi-datacenter;
- raw RPC public impossible a securiser;
- trop de stalls/jitter;
- recovery apres drop trop lent;
- le modele partage est moins utile qu'un endpoint 70B unique;
- debug backend trop couteux pour la valeur produit;
- privacy impossible a expliquer honnetement.

---

## 13. Roadmap SBFB

Ce chantier ne doit pas entrer dans S67-S69.

Routage recommande:

```text
S67-S69: Factory + RRV @protocole + Babel dogfood
S70-S72: network/search/proof hardening
S73+: private compute groups and app-driven compute
S73+/post-Gate 2 R&D: Remote Fiber Compute Sharing
S74+/post-Gate 2 R&D: User-Sharded LLM Mode
```

Dependances:

- private compute groups;
- consent worker stable;
- artifact manifests;
- proof cards / provenance;
- worker capability registry;
- network probe;
- sandbox/fallback policy.

---

## 14. Conclusion

Le partage d'un gros LLM entre ordinateurs d'utilisateurs est objectivement
possible. Les sources open source et recherche le prouvent deja.

Mais pour SBFB, le produit ne doit pas etre:

```text
un nouveau vLLM
un nouveau llama.cpp
un nouveau Ray
un RPC brut expose sur Internet
```

Le produit doit etre:

```text
une session de modele partagee, privee, mesuree, signee et reversible
```

La valeur SBFB:

- transformer des ordinateurs d'utilisateurs en groupe compute prive;
- prouver quel modele/shard/backend a tourne;
- distribuer les artefacts par hash;
- imposer consentement et quotas;
- isoler les ports dangereux;
- enregistrer les preuves;
- permettre aux apps comme D&D d'utiliser un gros modele logique.

Le premier spike realiste:

```text
5 utilisateurs de confiance
PCs filaires/fibre
70B quantifie
backend Petals-like ou llama.cpp RPC wrapped
session D&D longue
mesure TTFT/tokens/s/drop/fallback
```

---

## 15. Source index

External sources checked 2026-05-21:

- Petals GitHub:
  https://github.com/bigscience-workshop/petals
- Petals paper:
  https://arxiv.org/abs/2312.08361
- llama.cpp RPC README:
  https://raw.githubusercontent.com/ggml-org/llama.cpp/master/tools/rpc/README.md
- llama.cpp SECURITY:
  https://raw.githubusercontent.com/ggml-org/llama.cpp/master/SECURITY.md
- llama.cpp RPC RCE advisory:
  https://github.com/ggml-org/llama.cpp/security/advisories/GHSA-j8rj-fmpv-wcxw
- LocalAI P2P/federated inference:
  https://localai.io/features/distribute/
- LocalAI distributed mode:
  https://localai.io/features/distributed-mode/index.html
- exo:
  https://github.com/exo-explore/exo
- distributed-llama:
  https://github.com/b4rtaz/distributed-llama
- prima.cpp paper:
  https://arxiv.org/abs/2504.08791
- vLLM distributed serving:
  https://docs.vllm.ai/en/v0.10.0/serving/distributed_serving.html
- Ray cross-node parallelism:
  https://docs.ray.io/en/latest/serve/llm/user-guides/cross-node-parallelism.html
- SGLang multi-node deployment:
  https://sgl-project-sglang-93.mintlify.app/deployment/multi-node
- GPUStack:
  https://github.com/gpustack/gpustack
- GPUStack inference backends:
  https://docs.gpustack.ai/0.3/user-guide/inference-backends/
- Iroh blobs:
  https://docs.iroh.computer/protocols/blobs
- Iroh protocols/ALPN:
  https://docs.iroh.computer/concepts/protocols

Repo sources:

- `.planning/research/ethernet_cluster_mode_rnd.md`
- `.planning/research/gpu_pooling_distributed_inference.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`
- `crates/nexus-core-rs/src/task.rs`
- `docs/security/COMPUTE_THREATS.md`
- `docs/security/SPLIT_INFERENCE_DESIGN.md`
- `docs/security/PROCESS_ARCHITECTURE.md`
