# Research: Distributed GPU Compute Pooling for LLM Inference

**Domain:** P2P distributed GPU compute for LLM inference (code generation)
**Researched:** 2026-05-10
**Overall confidence:** MEDIUM-HIGH
**Context:** SBFB workers contribute GPU power via Ollama. Question: how to
pool multiple workers for larger tasks?

---

## Executive Summary

The distributed LLM inference ecosystem is mature enough for production use
in LAN environments and advancing rapidly for WAN/P2P. Five major
open-source projects address this directly: **Petals** (BitTorrent-style
pipeline parallelism, the academic pioneer), **Exo** (Apple Silicon clusters
with tensor parallelism), **Prima.cpp** (heterogeneous consumer hardware
with pipelined-ring parallelism), **Parallax/Gradient** (P2P pipeline
serving with geo-aware scheduling), and **GPUStack** (GPU cluster manager
orchestrating vLLM/SGLang backends).

**The critical finding for SBFB:** For a P2P network of heterogeneous
volunteer GPUs running Ollama over the internet, **task decomposition
(splitting a coding task into N independent sub-tasks, each on 1 GPU) is
dramatically more practical than model parallelism (sharding one large
model across N GPUs)**. Model parallelism over WAN suffers 30-50%
throughput degradation at 100ms latency, requires all participating nodes
to stay online for the duration of inference, and is bottlenecked by the
slowest node. Task decomposition has none of these constraints, works with
existing Ollama instances unmodified, and research shows 2-6x latency
improvements with minimal quality degradation for parallelizable tasks.

Model parallelism should only be considered for the LAN case (same
household/office, <5ms latency) and even then only when the target model
genuinely does not fit on a single worker's GPU.

---

## 1. Distributed LLM Inference Projects

### 1.1 Petals (bigscience-workshop/petals)

**What:** BitTorrent-style collaborative LLM inference. Clients form
pipeline-parallel chains through community-operated servers, each hosting
a subset of transformer blocks.

**Stars:** 10.1K | **Last release:** v2.2.0 (Sep 2023) | **Status:**
Academic project, public swarm active but server count varies.

**Supported models:** Llama 3.1 (up to 405B), Mixtral 8x22B, Falcon 40B+,
BLOOM 176B.

**How layer splitting works:**
- Each server hosts a contiguous range of transformer blocks (e.g.,
  blocks 0-15 of 80).
- Client discovers available servers via DHT, forms a chain of consecutive
  servers that together cover all blocks.
- D* Lite algorithm finds the shortest (lowest-latency) path through
  available servers.
- Hidden states (activations) flow between servers via direct TCP
  connections. Dynamic blockwise quantization halves bandwidth without
  quality loss.
- Load balancing achieves 90-100% of theoretical optimal throughput
  allocation.

**Performance numbers (from NeurIPS 2023 paper, arxiv:2312.08361):**

Sequential inference (single token generation):

| Model | Hardware | Network | Steps/sec |
|-------|----------|---------|-----------|
| Llama 2 70B | 3x T4 | 1Gbps, <5ms latency | 2.29 |
| Llama 2 70B | 3x T4 | 100Mbps, <5ms latency | 2.29 |
| Llama 2 70B | 3x T4 | 100Mbps, 100ms latency | 1.57 |
| Llama 2 70B | offloading baseline | -- | 0.139 |
| BLOOM 176B | 3x A100 | 1Gbps, <5ms | 1.71 |
| BLOOM 176B | 3x A100 | 100Mbps, <5ms | 1.66 |
| BLOOM 176B | 3x A100 | 100Mbps, 100ms | 1.23 |
| BLOOM 176B | local NVLink PP | -- | 2.46 |

Batch throughput (parallel forward pass):

| Model | Hardware | Network | Tokens/sec |
|-------|----------|---------|------------|
| BLOOM 176B | 3x A100 | 1Gbps, <5ms | 253.6 |
| BLOOM 176B | 3x A100 | 100Mbps, <5ms | 182.0 |
| BLOOM 176B | 3x A100 | 100Mbps, 100ms | 112.2 |
| BLOOM 176B | 10x RTX 3090 | 1Gbps | 131.0 |
| BLOOM 176B | 10x RTX 3090 | 100Mbps | 28.1 |
| BLOOM 176B | 10x RTX 3090 | 100Mbps, 100ms | 16.8 |

Real-world geo-distributed (14 servers, 2 continents): 0.83 steps/sec
inference, 32.6 tokens/sec batch. Up to 10x faster than offloading.

**Key insight:** Latency dominates over bandwidth. Going from 1Gbps to
100Mbps at <5ms latency barely affects performance (1.71 -> 1.66 steps/sec).
But adding 100ms roundtrip cuts throughput by 28% (1.71 -> 1.23). For SBFB
workers on residential internet (20-80ms to each other): expect 15-30%
throughput degradation compared to LAN.

**Fault tolerance (BLOOM 7.1B, 4 stages):**

| Failure rate | 128-token seq | 1024-token seq |
|--------------|---------------|----------------|
| 0% | 11.4 steps/s | 10.7 steps/s |
| 0.01% | 10.6 steps/s | 7.76 steps/s |
| 0.02% | 3.38 steps/s | 2.17 steps/s |

Even 0.02% failure rate (1 in 5000 steps) causes 3-5x throughput drop on
longer sequences. This is devastating for volunteer P2P networks where
nodes disconnect frequently.

**Confidence:** HIGH -- numbers from peer-reviewed NeurIPS paper.

**Sources:**
- [Petals GitHub](https://github.com/bigscience-workshop/petals)
- [Petals paper (NeurIPS 2023)](https://arxiv.org/abs/2312.08361)
- [Petals homepage](https://petals.dev/)

---

### 1.2 Exo (exo-explore/exo)

**What:** Run frontier AI models locally by pooling memory and compute
across multiple devices. Focus on Apple Silicon clusters.

**Stars:** 25K+ | **Status:** Active development (2026).

**How model sharding works:**
- Topology-aware auto-parallel analyzes device resources and network
  characteristics.
- Supports both pipeline parallelism (split by layers) and tensor
  parallelism (split within layers).
- Tensor parallelism delivers up to 1.8x speedup on 2 devices and 3.2x
  speedup on 4 devices.
- MLX Distributed backend for communication. RDMA over Thunderbolt 5
  for microsecond-level inter-device latency.
- Automatic device discovery on the local network. No manual config.

**Performance numbers (from official benchmarks, blog.exolabs.net):**

LLaMA 3.2 3B on M4 Pro (24GB) cluster:

| Configuration | Single-request TPS | Multi-request TPS |
|---------------|-------------------|-------------------|
| 1x M4 Pro | 49.3 | 49.3 |
| 2x M4 Pro | 44.4 | 95.7 |
| 3x M4 Pro | 39.7 | 108.8 |

Single-request: 19% degradation from 1 to 3 devices (network overhead).
Multi-request: 2.2x throughput at 3 devices (not 3x -- sub-linear scaling).

Larger models:
- DeepSeek V3 671B: 5.37 tok/s on 8x M4 Pro Mac Minis.
- Llama 3.1 70B: ~12 tok/s on 2x M3 Max.

Activation transfer between nodes: <4KB for LLaMA 3.2 3B, scaling
linearly with layer size. Network hops add 20-100ms per request on LAN.

**Limitations for SBFB:**
- **Linux GPU support missing.** Currently CPU-only on Linux. GPU support
  (CUDA) is under development. This is a dealbreaker for SBFB's target
  hardware (NVIDIA GPUs on Linux/Windows).
- RDMA requires macOS 26.2+ and Thunderbolt 5. Not applicable to P2P.
- No fault tolerance: if one node goes down mid-generation, the whole
  request fails.
- Designed for same-LAN devices. No WAN/P2P support.

**Confidence:** HIGH -- official benchmarks + GitHub repository.

**Sources:**
- [Exo GitHub](https://github.com/exo-explore/exo)
- [Exo benchmarks](https://blog.exolabs.net/day-1/)
- [Exo 2026 guide](https://toolhalla.ai/blog/exo-framework-distributed-inference-guide-2026)

---

### 1.3 Prima.cpp

**What:** Distributed on-device inference for 30-70B models on
heterogeneous consumer hardware (mixed CPUs/GPUs, insufficient RAM/VRAM,
slow disks, Wi-Fi).

**Status:** Research paper (April 2025, arxiv:2504.08791). Open source fork
exists (electron-rare/prima-cpp).

**How pipelined-ring parallelism (PRP) works:**
- Devices form a ring topology. Each processes a layer window (configurable
  contiguous layer range) per round.
- Multiple rounds per token prediction. A 36-layer model across 6 devices
  with window size 2 = 3 rounds per token.
- PRP overlaps disk I/O with compute across devices. Smaller, staggered
  layer windows prevent memory overflow from prefetch-release conflicts.
- Halda scheduler co-optimizes per-device CPU/GPU workloads under
  RAM/VRAM constraints.

**Performance numbers:**

| Model size | TPOT | Notes |
|-----------|------|-------|
| 70B | 674 ms/token | <6% memory pressure |
| 32B + speculative decoding | 26 tok/s | Best case |
| 8B | 15 ms/token | Single device sufficient |

Comparison to baselines on 4 consumer devices (Mac M1 + RTX 3070 + RTX
2080Ti + Android phone, Wi-Fi 320-610 Mbps, 3-7ms latency):
- vs llama.cpp: 17x lower TPOT, 8x lower TTFT on 70B
- vs exo/dllama: 5-8x lower TPOT, 12-24x lower TTFT, no OOM

**Relevance for SBFB:** Demonstrates that heterogeneous consumer hardware
CAN do distributed inference, but only on LAN (3-7ms latency). The 674
ms/token for 70B is 1.5 tok/s -- usable for code generation but slow. The
approach is promising if SBFB ever targets same-LAN worker pooling.

**Confidence:** MEDIUM -- academic paper, not widely deployed. Fork exists
but maturity unclear.

**Sources:**
- [Prima.cpp paper](https://arxiv.org/abs/2504.08791)
- [Prima.cpp fork](https://github.com/electron-rare/prima-cpp)

---

### 1.4 Parallax (Gradient Network)

**What:** P2P distributed inference engine. Maps pipeline stages to
individual network nodes with direct hidden state exchange.

**Status:** Active, backed by Gradient Network (decentralized compute
marketplace).

**Architecture:**
- Two-phase scheduling: (1) place model layers across GPUs to optimize
  latency/throughput under memory+bandwidth constraints, (2) stitch layers
  from different replicas into execution chains balancing load.
- Region-based heuristic constrains layer allocation within geographic
  regions to minimize cross-region transfers.
- GPU workers use modified SGLang. Apple workers use optimized Metal kernel.
- Phase 1 scheduling: 8.55ms at 256 GPUs. Phase 2 per-request: 6.63ms
  at 256 GPUs (negligible overhead).

**Performance (vs Petals, on Qwen2.5-72B-Instruct-GPTQ-Int4):**
- 3.1x reduction in end-to-end latency
- 5.3x improvement in inter-token latency
- 3.1x higher throughput
- Test environment: 7 heterogeneous GPUs, geographically separated
  datacenters, 10ms average inter-machine latency.

**Relevance for SBFB:** Most architecturally similar to what SBFB would
need for model parallelism. But it requires modifying inference backends
(custom SGLang), which conflicts with SBFB's Ollama-based worker design.

**Confidence:** MEDIUM -- backed by funded company, published paper
(arxiv:2509.26182), but ecosystem is tied to Gradient's token economy.

**Sources:**
- [Parallax paper](https://arxiv.org/abs/2509.26182)
- [Parallax GitHub](https://github.com/GradientHQ/parallax)
- [Parallax docs](https://docs.gradient.network/the-open-intelligence-stack/parallax)

---

### 1.5 GPUStack

**What:** GPU cluster manager that orchestrates inference engines (vLLM,
SGLang, TensorRT-LLM, llama-box) for AI model deployment.

**Status:** Active, v0.5+ (2025-2026). Open source.

**Key features:**
- Single-node and multi-node multi-GPU inference.
- Heterogeneous GPU support across vendors (NVIDIA, AMD, Ascend, Moore
  Threads).
- Binpack and spread placement strategies.
- OpenAI-compatible API.
- Multi-node multi-GPU via llama-box backend.

**Relevance for SBFB:** GPUStack is a centralized cluster manager. It
assumes you control all nodes. Not P2P, not volunteer-based. The placement
strategies and heterogeneous GPU support are informative for SBFB's
dispatcher design, but GPUStack itself is not suitable for a trustless P2P
network.

**Confidence:** HIGH -- well-documented, active GitHub.

**Sources:**
- [GPUStack GitHub](https://github.com/gpustack/gpustack)
- [GPUStack v0.2 announcement](https://gpustack.ai/introducing-gpustack-0-2/)

---

## 2. Task Decomposition vs Model Parallelism

### 2.1 The Core Question

For SBFB code generation: split a coding task into N independent sub-tasks
(each run on 1 GPU with a model that fits) **or** shard a large model
across N GPUs over the network?

### 2.2 Model Parallelism Over P2P: Why It's Impractical

**Problem 1: Latency sensitivity.**
Petals data shows that 100ms network latency causes 28% throughput
degradation for sequential inference and 56% for batch processing (on
RTX 3090 cluster). Residential internet between peers is typically
20-80ms within a country, 100-200ms intercontinental. The degradation
compounds with each pipeline stage.

**Problem 2: Bottleneck at slowest node.**
llama.cpp RPC documentation explicitly states: "Pipeline parallelism is
bottlenecked by the slowest stage, so mixing an RTX 3090 with a GTX 1080
Ti drops overall throughput to roughly 1080 Ti levels." In a volunteer P2P
network, hardware is maximally heterogeneous. The weakest link determines
throughput for ALL participants in the pipeline.

**Problem 3: Fault sensitivity.**
Petals data: 0.02% failure rate causes 3-5x throughput drop on 1024-token
sequences. In a volunteer network, nodes disconnect regularly (sleep,
reboot, bandwidth contention). Exo has zero fault tolerance -- one node
down kills the entire request.

**Problem 4: Requires modified inference backend.**
Petals, Exo, and Parallax all use custom inference backends. They cannot
use stock Ollama. SBFB workers run Ollama. Replacing Ollama with a custom
shard-aware runtime would require massive changes to the worker crate and
break the existing Ollama model ecosystem.

**Problem 5: Coordination overhead.**
All nodes in a pipeline must be online simultaneously for the duration of
inference. For a 70B model generating a 500-token code response at 1.5
tok/s, that's ~5 minutes of synchronized uptime across 3-4 nodes. Any
disconnection = restart.

### 2.3 Task Decomposition: Why It's the Right Approach for SBFB

**Pattern: Agentic task splitting.** An orchestrator (coordinator) breaks
a complex coding task into independent sub-tasks, dispatches each to a
separate worker running a model that fits on its GPU, then merges results.

**Evidence supporting this approach:**

1. **Amazon Science research (2025):** "Smaller LLMs, when specialized,
   can match the performance of larger, unmodified frontier LLMs on the
   same tasks." This validates using multiple 8B-32B models instead of
   one 70B+ model.

2. **Skeleton-of-Thought (2023):** Up to 2.39x speedup across 12 LLMs by
   decomposing answers into skeleton + parallel expansion. Quality
   maintained for structured outputs (code, lists, explanations).

3. **ParallelPrompt (2025):** Analysis of 37,000 real-world prompts found
   10.3% contain inherent parallelizable structures. Reading Comprehension
   achieved 5.72x speedup, Repeated Generation 4.39x, with minimal quality
   degradation.

4. **Multi-agent code generation (2025):** A hierarchical decomposition +
   bottom-up generation + multi-agent validation showed 23.79% improvement
   in Pass@1 scores on HumanEval using 8B parameter models vs single-shot.

**Why this works for code generation specifically:**

- Code tasks are naturally decomposable: generate function A, generate
  function B, generate tests, generate documentation. These have low
  contextual dependency between sub-tasks.
- Each sub-task fits on a single consumer GPU (8B-32B models handle code
  well -- DeepSeek-Coder-V2-Lite-Instruct at 16B, Qwen2.5-Coder-32B).
- Failure of one sub-task does not invalidate others. Retry is trivial.
- No need for synchronized uptime. Workers can be ephemeral.
- Works with stock Ollama. Zero changes to worker runtime.

**Limitations:**
- Not all tasks are decomposable. Highly sequential reasoning (e.g.,
  "explain step by step why this algorithm works") loses quality when
  parallelized.
- Orchestration overhead: the coordinator must understand how to split
  tasks, which requires its own LLM call (meta-reasoning).
- Merging results requires validation to ensure consistency.
- As decomposition depth grows, coordination overhead can dominate,
  potentially diminishing gains.

### 2.4 Verdict

| Criterion | Model Parallelism | Task Decomposition |
|-----------|------------------|--------------------|
| Works over WAN (>20ms) | Poorly (28-56% degradation) | Yes |
| Tolerates node failure | Poorly (request fails) | Yes (retry sub-task) |
| Works with Ollama | No (custom backend) | Yes |
| Heterogeneous hardware | Bottlenecked by slowest | Each node at own speed |
| Implementation complexity | Very high | Medium |
| Quality for code gen | Same model = same quality | Multiple smaller models, slightly lower per-task but compensated by specialization |
| Minimum nodes required | 2-4 (to fit large model) | 1 (scales to N) |

**Recommendation: Task decomposition for SBFB. Model parallelism is a
non-starter for volunteer P2P over the internet.**

The only scenario where model parallelism makes sense for SBFB is a future
"local cluster" mode where a user pools their own machines on the same LAN
(e.g., home lab with 3 GPUs). This is a post-v1.0 feature at best.

---

## 3. Ollama Federation

### 3.1 Can Multiple Ollama Instances Collaborate?

**No, not natively.** Ollama is designed for single-instance use. It does
not support:
- Cross-node model sharding / pipeline parallelism
- Distributed inference across machines
- Worker federation / clustering

This is confirmed by multiple GitHub issues:
- [Issue #4643](https://github.com/ollama/ollama/issues/4643):
  "Llama.cpp now supports distributed inference across multiple machines"
  -- requesting this for Ollama. No implementation.
- [Issue #5983](https://github.com/ollama/ollama/issues/5983):
  "Distributed Computing: Run single large model on multiple machines" --
  open, no timeline.
- [Issue #9147](https://github.com/ollama/ollama/issues/9147): "Does
  ollama support multi-node pipeline inference?" -- confirmed no.

### 3.2 Third-Party Ollama Federation Projects

**Hive (2025, published in SoftwareX journal):**
- HiveCore (central proxy) + HiveNode (worker agent on each Ollama node).
- Each HiveNode opens N socket connections to HiveCore (N = concurrent
  request capacity).
- Request routing: load balancer distributes requests to nodes. Each node
  runs the FULL model -- this is request-level distribution, NOT model
  sharding.
- Security: outbound-only connections, no public exposure needed.
- HiveCore overhead: CPU 0.108 idle -> 0.139 under load. Memory <300MB.
- **This is a load balancer, not a model parallelism system.** Each Ollama
  instance must independently fit the model in memory.

**Source:** [Hive paper (SoftwareX)](https://www.sciencedirect.com/science/article/pii/S2352711025001505)

**OLOL (K2/olol):**
- gRPC interfaces for distributed inference across multiple Ollama instances.
- Unified API endpoint, transparent request distribution.
- Same pattern as Hive: load balancing, not model sharding.

**LocalAI P2P:**
- Uses libp2p (same as IPFS) for peer discovery.
- Two modes: federated (load balancing) and worker (weight sharing via
  llama-cpp RPC).
- Worker mode supports model sharding BUT only with llama-cpp-compatible
  models, not Ollama.
- Federated mode: each node loads full model. Random routing (not
  optimized).
- Status: experimental / tech preview.

**Source:** [LocalAI P2P docs](https://localai.io/features/distribute/)

### 3.3 llama.cpp RPC (underlying Ollama runtime)

Ollama uses llama.cpp internally. llama.cpp has RPC support for distributed
inference:
- One controller + N RPC workers exposing GPU memory and compute.
- Hidden states transferred between nodes after each layer (few KB).
- **Critical limitation:** "Performance gain only applies for parallel
  inference, not single inference. Inference is strictly sequential
  regarding the order of layers." Adding more RPC nodes does NOT give more
  tok/s for a single request -- it only helps when the model does not fit
  in one machine's memory.
- Pipeline parallelism bottlenecked by slowest stage.

**AMD demo (2026):** 4-node cluster of Ryzen AI Max+ ran a 1-trillion
parameter model (Kimi K2.5) via llama.cpp RPC. This is the ceiling --
only useful when the model physically cannot fit on one machine.

**Source:**
- [llama.cpp RPC guide](https://medium.com/@soumyajit.swain/distributed-llm-inference-on-consumer-machines-with-llama-cpp-a-bare-metal-approach-55ef6ef81f35)
- [AMD Ryzen AI cluster](https://www.amd.com/en/developer/resources/technical-articles/2026/how-to-run-a-one-trillion-parameter-llm-locally-an-amd.html)
- [llama.cpp RPC discussion](https://github.com/ggml-org/llama.cpp/discussions/9136)

### 3.4 Implications for SBFB

SBFB already has the right architecture: each worker runs its own Ollama
instance, the coordinator dispatches tasks. The existing task-level
distribution (each task to one worker) IS the Hive/OLOL pattern. No
additional "federation" layer is needed.

For model parallelism via Ollama: not possible without replacing Ollama
with raw llama.cpp + RPC on workers. This contradicts SBFB's design
(workers run stock Ollama, model management via `ollama pull`).

---

## 4. Real-World Performance: LAN vs WAN

### 4.1 Pipeline Parallelism Throughput Degradation

Data synthesized from Petals, Exo, and Prima.cpp benchmarks:

| Network condition | Typical latency | Throughput vs local | Source |
|-------------------|----------------|--------------------|---------
| NVLink (same machine) | <1us | 100% (baseline) | Petals paper |
| Thunderbolt 5 RDMA | <0.1ms | ~95% | Exo claims |
| LAN Ethernet 1Gbps | <1ms | ~93% (1.71 vs 1.84 est.) | Petals paper |
| LAN Wi-Fi | 3-7ms | ~85% | Prima.cpp paper |
| WAN same continent | 20-50ms | ~75-85% | Petals extrapolation |
| WAN intercontinental | 100ms+ | ~50-70% | Petals paper (1.23 vs 1.71 = 72%) |
| Exo LAN (3 devices) | 20-100ms | 81% single-req (39.7/49.3) | Exo benchmark |

### 4.2 Key Rules of Thumb

1. **Latency matters more than bandwidth.** Going from 1Gbps to 100Mbps
   at low latency: <3% degradation. Adding 100ms latency at 100Mbps: 28%
   degradation. For SBFB's target (residential connections, 50-200Mbps,
   20-100ms between peers): latency is the bottleneck, not bandwidth.

2. **Pipeline parallelism is preferred over tensor parallelism for slow
   networks.** Tensor parallelism requires all-reduce operations within
   each layer (bandwidth-intensive, latency-sensitive). Pipeline
   parallelism only transfers activations once per stage (few KB). Rule
   from vLLM: "Use pipeline parallelism across nodes and tensor
   parallelism within nodes."

3. **Heterogeneous hardware kills pipeline throughput.** The slowest node
   in the pipeline determines throughput for all. Mixing RTX 3090 + GTX
   1080 Ti = GTX 1080 Ti speed. In a volunteer P2P network, this is the
   norm, not the exception.

4. **Fault tolerance is the P2P killer.** Even 0.02% failure rate
   (1 in 5000 steps) causes 3-5x throughput drop on long sequences. For
   code generation (500-2000 tokens), this means frequent request failures
   in a network with >10 nodes and typical volunteer reliability.

### 4.3 What This Means for SBFB

**For task-level distribution (current design):** Network latency adds
20-100ms per task dispatch, which is negligible compared to LLM inference
time (seconds to minutes). The current architecture is already optimal.

**For model parallelism (hypothetical):** Only viable on LAN (<5ms
latency) with homogeneous hardware. Not suitable for SBFB's P2P WAN
network. Would require replacing Ollama with custom inference runtime.

---

## 5. Recommendation for SBFB

### 5.1 Short-term (v1.0): Stay with Task-Level Distribution

The current SBFB architecture -- coordinator dispatches independent tasks
to individual workers running stock Ollama -- is the right pattern. It
matches what Hive and OLOL do, but P2P instead of centralized.

To improve coding task throughput:
1. **Agentic task decomposition in the coordinator.** When a user submits
   a complex coding task, the coordinator's LLM call splits it into
   independent sub-tasks (e.g., function generation, test generation,
   documentation). Each sub-task is dispatched to a separate worker.
2. **Model affinity.** Prefer dispatching coding tasks to workers that
   already have a code-specialized model loaded (DeepSeek-Coder, Qwen-Coder).
3. **Speculative execution.** For critical tasks, send the same sub-task
   to 2 workers and take the first result (redundancy for reliability,
   not validation).

### 5.2 Medium-term (post-v1.0): LAN Cluster Mode

For users who have multiple GPUs on the same LAN, add an optional
"cluster mode" where workers on the same network can pool resources for
larger models. Implementation options:
- **llama.cpp RPC:** Workers expose llama.cpp RPC servers, one coordinator
  worker acts as controller. Requires bypassing Ollama and running
  llama-server directly. Moderate complexity.
- **Exo integration:** If/when Exo supports Linux GPU (CUDA), it could
  be used as the LAN inference backend. Currently blocked by missing
  Linux GPU support.

### 5.3 Long-term (post-v1.0): Evaluate Parallax/Prima.cpp

If the network grows to hundreds of nodes and there's demand for running
models that don't fit on any single worker:
- Parallax's geo-aware scheduling and region-based heuristic are the
  most applicable architecture for SBFB's P2P topology.
- Prima.cpp's pipelined-ring parallelism is promising for heterogeneous
  hardware but unproven at scale.
- Both require replacing the Ollama inference backend with custom
  runtimes, which is a major architectural change.

### 5.4 Anti-Recommendations

**Do NOT:**
- Try to implement model parallelism over WAN for v1.0. The performance
  degradation and fault sensitivity make it unusable for volunteer P2P.
- Replace Ollama with a custom inference runtime. Ollama's model
  management (`ollama pull`, model library, quantization selection) is a
  major UX win for volunteers. Replacing it removes a key onboarding
  advantage.
- Use Exo as-is. No Linux GPU support makes it irrelevant for SBFB's
  target hardware (NVIDIA GPUs on Linux/Windows).
- Build a custom Petals-like system. The complexity is enormous (DHT,
  routing, block assignment, quantization, fault recovery) and the
  performance on volunteer WAN networks is poor.

---

## 6. Summary Table of Projects

| Project | Type | P2P? | WAN? | Ollama? | Linux GPU? | Maturity | SBFB fit |
|---------|------|------|------|---------|-----------|----------|----------|
| Petals | Pipeline parallel | Yes | Yes | No | Yes | Academic, stable | Low (WAN perf) |
| Exo | Tensor + pipeline | LAN only | No | No | No (CPU only) | Active dev | None currently |
| Prima.cpp | Ring parallel | LAN only | No | No | Yes | Research | Future LAN mode |
| Parallax | Pipeline parallel | Yes | Yes | No | Yes | Active, funded | Future reference |
| GPUStack | Cluster manager | No | No | No (vLLM/SGLang) | Yes | Production | None (centralized) |
| Hive | Load balancer | No | Yes | Yes | N/A | Published | Already done by SBFB |
| OLOL | Load balancer | No | Yes | Yes | N/A | Early | Already done by SBFB |
| LocalAI P2P | LB + sharding | Yes | Yes | Partial | Yes | Experimental | Watch |
| llama.cpp RPC | Layer offload | No | Yes* | Underlying | Yes | Stable | Future LAN mode |

*llama.cpp RPC works over WAN but performance degrades significantly.

---

## Sources

### Academic Papers
- [Petals: Distributed Inference and Fine-tuning (NeurIPS 2023)](https://arxiv.org/abs/2312.08361) -- HIGH confidence
- [Prima.cpp: Fast 30-70B LLM Inference on Heterogeneous Clusters](https://arxiv.org/abs/2504.08791) -- MEDIUM confidence
- [Parallax: Efficient LLM Inference Service over Decentralized Environment](https://arxiv.org/abs/2509.26182) -- MEDIUM confidence
- [Hive: Distributed Ollama Inference (SoftwareX 2025)](https://www.sciencedirect.com/science/article/pii/S2352711025001505) -- HIGH confidence

### Project Repositories
- [Petals GitHub](https://github.com/bigscience-workshop/petals) -- 10.1K stars
- [Exo GitHub](https://github.com/exo-explore/exo) -- 25K+ stars
- [GPUStack GitHub](https://github.com/gpustack/gpustack)
- [Parallax GitHub](https://github.com/GradientHQ/parallax)
- [Prima.cpp fork](https://github.com/electron-rare/prima-cpp)
- [LocalAI P2P docs](https://localai.io/features/distribute/)
- [OLOL GitHub](https://github.com/K2/olol)

### Performance References
- [Exo transparent benchmarks](https://blog.exolabs.net/day-1/)
- [llama.cpp RPC discussion](https://github.com/ggml-org/llama.cpp/discussions/9136)
- [Ollama distributed inference issues](https://github.com/ollama/ollama/issues/4643)
- [vLLM distributed inference blog](https://blog.vllm.ai/2025/02/17/distributed-inference.html)

### Task Decomposition Research
- [Amazon Science: Task decomposition + smaller LLMs](https://www.amazon.science/blog/how-task-decomposition-and-smaller-llms-can-make-ai-more-affordable)
- [Can Small Agents Collaborate to Beat a Single Large LLM?](https://arxiv.org/abs/2601.11327)
- [Guided Code Generation with LLMs: Multi-Agent Framework](https://arxiv.org/abs/2501.06625)
- [ParallelPrompt: Extracting Parallelism from LLM Queries](https://arxiv.org/abs/2506.18728)
