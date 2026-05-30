# Addendum SOTA — Design du sharding LLM distribue (S76)

**Date** : 2026-05-30.
**Statut** : R&D, extension de `remote_user_sharded_llm_rnd.md` (2026-05-21).
**Etend aussi** : `docs/security/SPLIT_INFERENCE_DESIGN.md`,
`gpu_pooling_distributed_inference.md`, `DISTRIBUTED_GPU_RESEARCH.md`,
`ethernet_cluster_mode_rnd.md`, `chat_ia_reseau_recherche_reseau_rnd.md`.
**Source** : workflow de recherche 4 axes (Petals, Parallax/SOTA, verif
peers non-confiants, transport iroh) + relecture du R&D repo existant.
**Pour** : le sprint S76 (feature phare sharding) de
`roadmap_v5_factory_complete_vision.md`. Ce doc NE remplace PAS la R&D
2026-05-21 — il la met a jour avec l'etat de l'art 2025-2026 et fige des
decisions de design concretes.

---

## 0. Ce que cet addendum ajoute a la R&D 2026-05-21

La R&D existante avait deja : verdict (faisable, groupe prive), framing
« couche protocolaire neutre » (pas de kernel), primitives wire
(ComputeGroup/WorkerCapability/ShardPlan/ShardAssignment/
ShardedSessionManifest/RunProof §10), sizing VRAM (§6), contraintes reseau
NAT/CGNAT (§7), threat model groupe prive (§8), plan R&D (§11). Le NOUVEAU :

1. **Schema de verification gradue N0-N4** ancre sur la SOTA 2025 (TOPLOC,
   DiFR, VeriLLM, opML, SENTINEL) — comble le « Byzantine non implemente »
   de Petals et le « confidence_score field » de SPLIT_INFERENCE.
2. **Algorithme de scheduler concret** (Parallax placement + Petals routing
   + churn) portable en Rust.
3. **Mapping iroh data-plane** precis (ALPN custom `sbfb/shard/1`).
4. **Enveloppe de performance chiffree et honnete** (Petals reel vs
   Parallax datacenter vs WAN residentiel SBFB).

---

## 1. Architecture (decisions figees)

- **Pipeline / layer-split EXCLUSIVEMENT** (Petals-like block serving),
  jamais tensor-parallel. Le tensor-parallel (exo, distributed-llama,
  prima.cpp) exige des all-reduce par couche → LAN/NVLink only, exclu hors
  datacenter. Justification verrouillee par le finding empirique commun
  Petals/Parallax : **« performance degrades with higher latency, not
  bandwidth »** (Llama-2-70B identique a 1Gbit et 100Mbit, mais -31 % a
  100ms RTT). → on optimise le **RTT et le nombre de hops**, pas le debit.
- 1 shard = bloc contigu de N couches Transformer / worker, KV-cache
  `local_ephemeral` (deja dans ShardAssignment §10.3).
- Cas cible : **70B Q4 sur 3-5 PC gamers 16-24 GB VRAM**. Decode interactif
  2-5 tok/s (No-go < 2 tok/s), TTFT < 15 s. Batch=1 sequentiel pour le
  tour-par-tour ; batching parallele = mode « pre-generation » optionnel
  cote app, pas le hot-path.
- **SBFB n'ecrit aucun kernel CUDA/ggml** : admission + manifests +
  transport + proofs + verification + fallback. Le backend (llama.cpp RPC
  wrappe ou Petals-like) calcule.
- **INVARIANT NON-NEGOCIABLE** : mode **groupe prive explicite** (allowlist
  Ed25519, zero worker anonyme). Le sharding est en tension directe avec
  « peers anonymes / zero admin » → jamais le mode public par defaut. R-iroh
  -audit P0 → pilote ferme uniquement.

## 2. Scheduler (hybride Parallax + Petals, porte Rust sur iroh)

PAS de reutilisation du code hivemind/Python (torch/go-libp2p anciens,
ecosysteme en sommeil depuis sept 2023) — design porte from-scratch.

- **Phase 1 placement (montage de session)** : DP + contrainte VRAM/worker
  (water-filling sur VRAM **libre mesuree** via gpu_probe, pas declaree) +
  contrainte de lien. Grouper les couches consecutives entre peers a faible
  RTT mutuel. SBFB n'a pas de geoIP central → **clustering empirique
  k-medoids sur matrice RTT pairwise mesuree** (repond a la question
  ouverte « regions sans autorite geo »).
- **Phase 2 routing (par requete)** : DAG layer-indexe + 1 sweep DP G→D
  (relaxation Parallax : `dp2(l+1,g') = min(..., dp2(l,g) + rho<g,g'> +
  tau<g',l+1>)`, rho = RTT one-way/paire, tau = latence profilee
  couche/GPU). O(L·R²) — negligeable a 3-5 peers.
- **Churn** : re-equilibrage **ACTIF de Petals** (heap de fallback/stage
  ordonne par latence + cache client-side d'activations + `replace_failed
  _server` O(t)), PAS le modele « cle DHT expire » de Parallax (ne re-route
  jamais mid-inference — faille churn). Perf-map (rho, tau) republiee toutes
  1-2 s dans **iroh-docs** (equivalent du DHT Kademlia, deja en place).

## 3. Verification sur peers non-confiants (N0-N4) — le travail original

Aucun zkML sur le chemin par defaut (ezkl GPT-2 ~159 s/token, zkLLM 13B
15 min/forward — prohibitif). Etend `verification.rs` (4 couches),
`redundancy.rs` (majority vote), `rerun.rs` (sampling 1 %).

| Niveau | Quoi | Cout | Quand |
|--------|------|------|-------|
| **N0** baseline | Ed25519 + fingerprint **TOPLOC** (LSH top-k dernier hidden state, 258 B/32 tok, detection 100 % swap modele/precision, validation 100x plus rapide). **Remplace le Layer3 logprob fragile.** | ~0 + 258 B/32 tok | systematique |
| **N1** spot-check | Verifieur tire par **VRF Ed25519** : prefill-only **VeriLLM** (~1 %) OU **Activation-DiFR** (AUC>0.999 en 2 tokens). One-honest-verifier. **Doit randomiser temperature ET seed** (faille DiFR). | ~1 % | 1-5 %, VRF |
| **N2** redondance | M-of-N, comparaison **fingerprint TOLERANT** (pas hash byte — casse sous non-determinisme GPU) | 100-300 % | haute-criticite |
| **N3** contestation | Bissection **opML-style** sur commitments d'activations par-frontiere (ancres iroh-docs + commit-reveal Ed25519, pas de smart-contract) ; **SENTINEL** (EMA inter-stages) localise le stage corrompu | O(1 bloc) | sur litige |
| **N4** zkML | DeepProve/NANOZK, mode « high-assurance » | 50-15000x | opt-in Gate 4+ |

**Sweet-spot SBFB** = N0 systematique + N1 1-5 % + N2 haute-criticite + N3
sur contestation. Mapping criticite-tache → niveau a formaliser (ex.
LibanLive = N2 obligatoire, DnD Forge = N0 seul).

**Limite d'implementation honnete (gap repo confirme)** : TOPLOC/DiFR
exigent l'acces au dernier hidden state / top-k logits. Le backend
**Ollama (HTTP localhost:11434) ne l'expose PAS** sans fork (interdit). MAIS
le backend **`llama_cpp`** (`crates/nexus-worker-core/src/llm/llama_cpp.rs:
340,434,452`) embarque `llama-cpp-2` **in-process** et manipule deja les
logits → extraction top-k faisable **sans fork**. → N0/N1 reserves au
backend `llama_cpp`, degrades a signature+redondance sur Ollama.

## 4. Transport iroh

- **Control plane (reutilise)** : iroh-docs LWW (perf-map, manifests,
  claims, RunProof, fingerprints) ; iroh-blobs BLAKE3 (distribution des
  POIDS hash-pinnes, Downloader resumable) ; gossip (annonces) ; pkarr +
  hole-punching + relais N0 (NAT/CGNAT).
- **Data plane activations (NOUVEAU)** : ALPN custom **`sbfb/shard/1`** au
  point d'insertion `node.rs:341-345`
  (`Router::builder().accept(b"sbfb/shard/1", ShardProtocol).spawn()`).
  Aucun ALPN custom n'existe aujourd'hui. 1 Connection QUIC **persistante**
  par paire de shards consecutifs + `open_bi` long-vecu **reutilise pour
  tous les tokens** (jamais re-connect/token). Framing longueur-prefixe.
  Pas via blobs (hashing = latence morte inadaptee au live) ni docs/gossip
  (inadaptes au point-a-point).
- **Observabilite** : `conn.stats()` (RTT path, jitter) alimente la perf-map
  sans ping applicatif (a verifier : fiabilite RTT par-connexion sur liens
  residentiels reels).
- **Contrainte dure (repo §7.1)** : hot-path DIRECT obligatoire (UDP/IPv6 ou
  hole-punch). **NO-GO si relais N0 dans le hot path.** iroh ~70 % direct,
  ~30 % relais → pour ces 30 %, sharding **refuse**, pas degrade
  silencieusement.
- **Securite transport ≠ securite calcul** (caveat fondamental) : QUIC/TLS
  protege du reseau, PAS du worker qui voit les activations en clair
  (inversion/split-learning, SI-1). Pas de TEE GPU consumer 2026 → **aucun
  claim de confidentialite face aux workers**, groupe prive obligatoire,
  pas de secret app dans les prompts. Limite physique, pas gap iroh.

## 5. Quantization

- **Poids (sizing)** : Q4 (~0.5 B/param). 5×16 GB = 70B Q4 confortable ;
  5×24 GB = 70B + 120B plausible ; 5×32 GB = 235B R&D. **Seuil sharding** :
  ne sharder QUE si `VRAM_modele_quantifie > VRAM_max_d_un_worker` (70B Q4
  ~40 GB > 24 GB 4090 → shard). Sinon endpoint federation (1 worker = 1
  modele, plus simple). Le sharding est un COUT (latence somme des
  frontieres + fragilite worker le plus lent + churn), pas un objectif.
- **Activations (bande passante)** : quant blockwise dynamique (Petals)
  halve la BP sans perte notable. Marginal en decode (~16 KB/token/
  frontiere), significatif en **prefill** (64-256 MB/frontiere). Le
  fingerprint TOPLOC doit etre coherent avec la precision d'activation.

## 6. Enveloppe de performance (honnete, chiffree)

- **Petals reel (seule preuve WAN residentielle)** : Llama-2-70B 2.29
  steps/s a <5ms → **1.57 a 100ms RTT (-31 %)**. BLOOM-176B geo-distribue
  14 serveurs Europe-NA : 0.83 steps/s (-28 % par la latence seule).
  Batch-64 : 253 tok/s.
- **Parallax = datacenter 10ms RTT** (borne SUPERIEURE optimiste, AUCUN test
  NAT/residentiel — ne pas extrapoler a SBFB).
- **WAN residentiel SBFB (pessimiste honnete)** : RTT 30-150ms, upload
  ASYMETRIQUE (le seuil Petals « 25 Mbit/s » est limite par l'**upload**).
  Extrapolation : ~3 s/token a 100ms multi-hop → **streaming token-par-token
  interactif inutilisable a 100-150ms**. Plancher acceptable si RTT/
  frontiere ≤ 20-40ms ET hops minimises. Prefill sature l'upload → TTFT
  domine par l'upload du prefill (placement Phase 1 modelise up/down
  **separement**).
- **Verdict produit** : D&D tour-par-tour VIABLE a 2-5 tok/s si l'app masque
  la latence (TTFT < 15 s OK). Chat/coding ultra-reactif **NON VIABLE** sur
  WAN residentiel multi-hop — **a ne pas promettre**. RTT > 80ms ou relais
  hot-path = NO-GO produit.
- Confidence : HIGH faisabilite, MEDIUM perfs reelles avant benchmark
  multi-machines SBFB (chiffres = Petals/Parallax, pas SBFB-mesures).

## 7. Phasage (prerequis dur)

**PREREQUIS BLOQUANT** : la **preuve cross-machine du task-routing (S75)**
doit etre verte AVANT tout code de sharding S76. Le sharding est strictement
plus exigeant et reutilise l'infra de routing cross-machine.

- **Phase 0 design** (ce doc + formalisation) : figer wire formats
  (schema_version=1, raw-op extensible pre-v1.0), ALPN `sbfb/shard/1`,
  schema N0-N3, mapping iroh. Etendre THREAT_MODEL (SI-1..SI-5). Trancher
  incentive-a-verifier + mapping criticite→niveau.
- **Phase A** network_probe + placement (RTT/jitter/loss/up/down/NAT/relay
  via conn.stats, k-medoids, scheduler 2-phases). Gate reseau : GO si p95
  RTT/frontiere ≤ 20-30ms, direct, pas de relais.
- **Phase B** spike toy (1B/7B sur 2-3 machines, ALPN, open_bi long-vecu,
  RunProof, TOPLOC N0 via llama_cpp).
- **Phase C** verification N1/N2/N3.
- **Phase D** benchmark 70B private group (3-5 PC, mesures reelles TTFT/
  tok-s/jitter/drop, test worker-drop) — **le gate produit**.
- **Phase E** GO/NO-GO (productiser / R&D / abandon).

Garde-fou : commencer 2-3 machines puis 5 (fragilite ∝ nombre de shards).

## 8. Questions ouvertes critiques (a porter au kickoff S76)

1. **Incentive a verifier sans token NON RESOLU** : un lazy verifier
   rationnel ne verifie pas ; VeriLLM resout par reward on-chain que SBFB
   ne peut pas repliquer (kudos non-monetaire interdit stake). Piste :
   kudos-pour-spot-check via curator reputation — a valider.
2. **Collusion worker+verifieur sous VRF** si Anti-Sybil non resolu →
   taille de pool honnete minimale + parametre VRF.
3. **Batch-invariance** imposable a Ollama/llama.cpp ? Sinon dependance
   totale aux fingerprints tolerants.
4. **TOPLOC sur Ollama impossible** (HTTP) → imposer `llama_cpp` pour le
   mode sharding, ou degrader la verif ?
5. Spot-check a temp>0 sans seed reproductible cross-GPU.
6. Bissection opML sans VM deterministe (commitments par-frontiere
   reproductibles ?).
7. Cache d'activations O(t) explose sur gros contexte de campagne.
8. Endpoint iroh dedie data-plane vs partage control-plane (benchmark).
9. API publique `TransportConfig` iroh 0.98 (congestion, datagram
   unreliable, taille max) — verifier `cargo doc` avant de promettre du
   tuning.
10. Retention/GC des fingerprints TOPLOC apres fenetre de contestation.

## 9. Sources cles

Petals (arXiv 2312.08361, 2209.01188, MIT) ; Parallax (arXiv 2509.26182,
Apache-2.0) ; Lattica (arXiv 2510.00183, Rust+QUIC, jumeau iroh) ; TOPLOC
(arXiv 2501.16007) ; DiFR (arXiv 2511.20621) ; VeriLLM (arXiv 2509.24257) ;
opML (arXiv 2401.17555) ; SENTINEL (arXiv 2603.03592) ; Thinking Machines
batch-invariance (arXiv 2506.09501). Repo : `remote_user_sharded_llm_rnd.md`,
`docs/security/SPLIT_INFERENCE_DESIGN.md`, `verification.rs`, `node.rs`
(154/262/341), `llama_cpp.rs` (340/434/452). Context7 `/n0-computer/iroh`
(2026-05-30, MIT/Apache compat AGPL).
