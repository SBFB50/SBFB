# Research: SBFB Ethernet Cluster Mode R&D

**Domain:** LAN/Ethernet and remote-fiber distributed LLM compute for private SBFB compute groups
**Researched:** 2026-05-21
**Status:** R&D proposal, not S68/S69 scope
**Confidence:** HIGH for the roadmap decision, MEDIUM for backend benchmarks until local tests run

---

## 0. Verdict

Open source already has multiple engines that can distribute a single large
model across several machines. SBFB should not reimplement tensor parallelism,
pipeline parallelism, CUDA kernels, NCCL, Ray, or `ggml` RPC.

The missing SBFB-specific solution is a **control plane**:

```text
SBFB Ethernet Cluster Mode =
  private group membership
  + local machine discovery
  + signed capability registry
  + model artifact hashes
  + backend launch profiles
  + network/security guardrails
  + reproducible benchmark proofs
  + signed run evidence
```

The data plane remains a specialized backend:

```text
Phase 1 backend: llama.cpp RPC / llama-box
Phase 2 backend candidates: exo, GPUStack/vLLM, SGLang, LocalAI wrappers
```

Iroh/SBFB coordinates, proves, distributes manifests, and records evidence.
It does not carry the per-token activation/tensor traffic.

Important correction for remote individuals:

```text
People at distance sharing gaming PCs over domestic fiber is not the same
problem as 2-3 cabled machines on one LAN.
```

For remote people, the primary solution is **Remote Fiber Compute Sharing**:

```text
Remote Fiber Compute Sharing =
  Factory task decomposition
  + remote local-model workers
  + signed artifacts and patches
  + redundant review / tests / benchmarks
  + optional full-model endpoint federation
```

The long-term "one giant model split across remote homes" path should be
treated as a narrow R&D subcase, not the product default. WAN/fiber is now fast
enough for large artifacts, model shard distribution, remote endpoints, and
parallel coding agents. It is still usually the wrong substrate for synchronous
per-token tensor parallelism because every token introduces latency-sensitive
coordination barriers.

---

## 1. Product Boundary

### 1.1 What this mode is

Ethernet Cluster Mode is a private, opt-in, local-cluster feature for users who
own or control several machines on the same wired LAN and want to run a model
that does not fit on one machine.

Examples:

- 2-3 desktop PCs with NVIDIA GPUs on 2.5/10GbE.
- A home lab with one controller and several GPU workers.
- A private research group running a shared cluster for coding/research.
- A Babel or Factory private compute group that wants benchmarked large-model
  inference after batch compute is already stable.

### 1.2 What this mode is not

It is not:

- public WAN volunteer model parallelism;
- a replacement for task decomposition;
- Ollama federation;
- a trustless compute market;
- a P2P feature for unknown machines;
- a Gate 1 requirement;
- a S68/S69 feature.

### 1.3 Roadmap placement

Current repo evidence says the near-term path stays:

```text
S67-S69: Factory + RRV @protocole + Babel dogfood
S70-S72: networked/proof search and post-pilot hardening
S73+: private compute groups / app-driven compute
Post-Gate 2: Ethernet Cluster Mode R&D spike
```

### 1.4 Remote people boundary

The real "sharing power between individuals" product is not a home datacenter.
It is a private or semi-private compute group where each participant contributes
their own machine as a worker.

This gives two distinct modes:

| Mode | Network | What is shared | Good use |
|------|---------|----------------|----------|
| Remote Fiber Compute Sharing | Internet/fiber between people | tasks, patches, tests, embeddings, model endpoints | Factory/Babel/coding |
| Ethernet Cluster Mode | same LAN or very low-latency metro link | backend devices for one sharded model | R&D for a model too large for one box |

SBFB should market the first one as the power-sharing feature. Ethernet Cluster
Mode is only a specialized R&D extension when a measured network looks more
like a lab link than the open Internet.

This aligns with:

- `.planning/research/gpu_pooling_distributed_inference.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`
- `crates/nexus-core-rs/src/task.rs`
- `docs/security/COMPUTE_THREATS.md`

---

## 2. Repo Grounding

### 2.1 Current compute contract

The canonical task path is:

```text
TaskEntry -> ClaimEntry -> ResultEntry
```

That path is signed, verifiable, and naturally supports batch/task
decomposition. It remains the default for Babel chunks, repository indexing,
tests, audits, embeddings, and multi-agent coding tasks.

Ethernet Cluster Mode must not replace this queue. It should appear as a
special worker capability or private compute group mode:

```text
capability: cluster_inference
backend: llamacpp_rpc | gpustack_vllm | sglang | exo
scope: private_group_only
network: local_lan_only
```

### 2.2 Current LLM backend boundary

`crates/nexus-worker-core/src/llm/mod.rs` already isolates generation behind
`LlmBackend`. Existing backends:

- `ollama`: HTTP to local Ollama, default and easy to run.
- `llama_cpp`: in-process `llama-cpp-2` + llguidance, feature-gated.

Ethernet Cluster Mode should not be bolted into the default `ollama` path. It
should be a separate backend profile or sidecar runner because the cluster has
different lifecycle, security, and failure semantics.

### 2.3 Threat model carry-over

`docs/security/COMPUTE_THREATS.md` already names prompt leakage, malicious
workers, GPU side channels, model extraction, DoS, and no-GPU-sharing policy.

Ethernet Cluster Mode increases the blast radius:

- prompts and KV/activation data cross the LAN;
- every RPC worker can observe model traffic unless the backend protects it;
- open RPC ports become local attack surfaces;
- all nodes in a single model run become availability-critical;
- artifact hashes and launch commands become security contracts.

Therefore the R&D spike must include security gates, not only speed tests.

---

## 3. Open Source Landscape

### 3.1 llama.cpp RPC

**Source:** https://github.com/ggml-org/llama.cpp/blob/master/tools/rpc/README.md

What exists:

- `rpc-server` exposes remote `ggml` devices.
- `llama-cli` and `llama-server` can connect with `--rpc host:port`.
- `--tensor-split` can override automatic weight/KV placement.
- remote tensor cache can reduce repeated model load traffic.
- RDMA transport exists on Linux when `libibverbs` is available.

Important official warning:

- The RPC backend is marked proof-of-concept, fragile, and insecure.
- The docs explicitly say not to run RPC on an open network or sensitive
  environment.
- `SECURITY.md` says not to use RPC/`rpc-server`/`llama-server` on untrusted
  networks and to encrypt data sent over the network.
- A 2026 GitHub advisory (`GHSA-j8rj-fmpv-wcxw`, CVE-2026-34159) documents
  unauthenticated RCE against the RPC backend when an attacker can reach the
  TCP RPC port. Version pinning, network isolation, and quarantine rules are
  mandatory for any SBFB wrapper.

SBFB fit:

```text
Best first backend for Ethernet Cluster Mode.
Use it as data plane, wrapped by SBFB controls.
Do not expose it outside a private LAN or tunnel.
```

Why it fits:

- GGUF is already the local-model ecosystem SBFB can realistically support.
- It directly solves "model does not fit on one box".
- It is simple enough for a two-machine R&D benchmark.

Why it is not enough alone:

- no SBFB identity;
- no private group policy;
- no signed provenance;
- no model artifact contract;
- no secure membership;
- no user-facing cluster lifecycle;
- weak security on raw RPC ports.
- no authorization layer strong enough to distinguish "seen on LAN" from
  "allowed to execute cluster traffic".

### 3.2 GPUStack

**Sources:**

- https://docs.gpustack.ai/2.0/overview/
- https://docs.gpustack.ai/2.0/user-guide/built-in-inference-backends/
- https://docs.gpustack.ai/0.6/tutorials/performing-distributed-inference-across-workers-llama-box/

What exists:

- open-source GPU cluster manager;
- schedulers, monitoring, user/API key management;
- vLLM, SGLang, TensorRT-LLM, MindIE, VoxBox backends;
- distributed vLLM across workers via Ray;
- distributed GGUF/llama-box tutorial for models exceeding one worker's VRAM;
- GPUStack 2.0 is more enterprise/cluster oriented than a lightweight home
  protocol primitive.

SBFB fit:

```text
Excellent reference and possible external backend.
Not the SBFB primitive itself.
```

Why:

- it assumes an operator-managed cluster;
- it is centralized;
- it overlaps with but does not implement SBFB trust/provenance semantics;
- it is larger than an early local-cluster R&D spike.

Possible SBFB use:

- import ideas from scheduler/capability detection;
- benchmark GPUStack as a comparative backend;
- later allow a private group to register "external_cluster_endpoint" if the
  user already runs GPUStack.

### 3.3 LocalAI distributed modes

**Sources:**

- https://localai.io/features/distribute/
- https://localai.io/features/distributed-mode/

What exists:

- P2P/federated mode;
- federated mode routes each request to one worker;
- worker mode can split weights using llama.cpp RPC;
- non-P2P mode can run `local-ai worker llama-cpp-rpc` and set
  `LLAMACPP_GRPC_SERVERS`;
- newer distributed mode uses PostgreSQL/NATS for production management;
- vLLM multi-node data-parallel mode is operator-launched and relies on vLLM's
  own cross-rank traffic.

SBFB fit:

```text
Good proof that wrappers around llama.cpp RPC are feasible.
Not enough as the SBFB control plane.
```

Why:

- it can shard weights in the llama.cpp path;
- it also includes load-balancing modes that do not solve "one model too big";
- it uses its own network/admin assumptions rather than SBFB provenance;
- useful as an interoperability target or comparison.

### 3.4 exo

**Source:** https://github.com/exo-explore/exo

What exists:

- automatic device discovery;
- topology-aware model placement;
- tensor and pipeline parallelism;
- RDMA over Thunderbolt 5 on recent macOS;
- OpenAI/Claude/Ollama-compatible APIs;
- strong Apple Silicon/Mac Studio story;
- official README says Linux currently runs CPU-only and Linux GPU support is
  still under development.

SBFB fit:

```text
Best reference for "local devices become an AI cluster".
Not a drop-in SBFB backend for common NVIDIA/Linux/Windows PCs yet.
```

Why:

- the UX direction is close to Ethernet Cluster Mode;
- automatic discovery and topology-aware splitting are exactly the class of
  control logic SBFB should study;
- but the current strongest path is Apple/RDMA, not generic wired PCs with
  NVIDIA cards.

Possible SBFB use:

- benchmark if the user owns multiple supported Macs;
- study topology scoring and cluster UX;
- do not make it the default backend for Nexus hardware assumptions.

### 3.5 vLLM + Ray

**Sources:**

- https://docs.vllm.ai/en/v0.7.0/serving/distributed_serving.html
- https://docs.ray.io/en/latest/serve/llm/user-guides/cross-node-parallelism.html

What exists:

- tensor-parallel and pipeline-parallel serving;
- Ray for multi-node placement when multiprocessing is not enough;
- production serving orientation;
- strong throughput story, batching, OpenAI-compatible serving.
- vLLM's security docs say inter-node traffic is insecure by default,
  including PyTorch Distributed, KV-cache transfer, tensor, pipeline, and data
  parallel channels. Nodes must be isolated on a dedicated network.
- vLLM's parallelism docs warn that efficient cross-node tensor parallelism
  needs fast interconnects, preferably InfiniBand-class hardware.

SBFB fit:

```text
Good long-term backend for controlled homogeneous GPU clusters.
Poor first backend for consumer Ethernet R&D.
```

Why:

- assumes Python/Ray operational complexity;
- multi-node GPU serving usually wants homogeneous GPUs and clean networking;
- not aligned with GGUF/Ollama-style local model usage;
- useful if Ethernet Cluster Mode later targets serious private labs.
- Ray itself expects a safe network and trusted code. Ray token auth exists in
  recent releases, but Ray documents it as defense in depth, not a replacement
  for network isolation.

### 3.6 SGLang

**Sources:**

- https://docs.sglang.ai/basic_usage/deepseek_v3.html
- https://docs.sglang.ai/advanced_features/router.html

What exists:

- high-performance serving runtime;
- multi-node tensor parallelism for models too large for one node;
- router/gateway with worker lifecycle and multi-protocol routing;
- production deployment orientation.
- SGLang multi-node deployment requires multiple GPU nodes, high-speed
  interconnect such as InfiniBand/RoCE/high-bandwidth Ethernet, consistent
  topology, shared storage or synchronized model weights, and recent NCCL.

SBFB fit:

```text
Strong future backend candidate.
Not the first home-LAN GGUF path.
```

Why:

- more suited to datacenter or well-controlled clusters;
- likely overkill before SBFB has private compute groups;
- useful for post-Gate R&D once benchmark/proof plumbing exists.
- it opens the usual HPC/runtime surfaces (`dist-init-addr`, NCCL, HTTP
  server, routers) that SBFB would need to pin to private interfaces and audit.

### 3.7 Text Generation Inference (TGI)

**Sources:**

- https://huggingface.co/docs/text-generation-inference/main/en/index
- https://huggingface.co/docs/text-generation-inference/main/basic_tutorials/launcher

What exists:

- mature Hugging Face text-generation server;
- tensor parallel / sharding-oriented launch options;
- tracing, metrics, safetensors, and model-serving operational features.

SBFB fit:

```text
Useful serving backend reference.
Not an Ethernet Cluster Mode control plane.
```

Why:

- TGI is a model server, not a private LAN admission/provenance layer;
- it is less directly aligned than llama.cpp RPC for GGUF home-lab clusters;
- it remains useful as a benchmark/backend comparison if SBFB later supports
  Hugging Face safetensors clusters.

### 3.8 Petals

**Sources:**

- https://github.com/bigscience-workshop/petals
- https://arxiv.org/abs/2312.08361

What exists:

- decentralized pipeline-parallel inference;
- public swarm and private swarm options;
- designed for unreliable/geodistributed devices;
- academic/research-proven for Llama/BLOOM-class models;
- privacy caveat: public swarm processes data on other people's machines.

SBFB fit:

```text
Important research reference.
Not the first Ethernet Cluster Mode backend.
```

Why:

- it targets WAN/unreliable swarms better than most;
- but SBFB's current product need is private, local, user-controlled machines;
- model support and ecosystem differ from the GGUF path.

### 3.9 Distributed Llama

**Source:** https://github.com/b4rtaz/distributed-llama

What exists:

- open-source home-device distributed LLM inference;
- root node plus worker nodes;
- RAM usage split across nodes;
- supports API server and CLI modes;
- limitations include power-of-two node counts and custom supported
  quantizations/formats.

SBFB fit:

```text
Useful research comparison.
Not broad enough to become the SBFB default.
```

Why:

- it is close in spirit to "connect home devices";
- constraints are too specific for the first SBFB R&D path;
- keep it in the benchmark matrix if time allows.

### 3.10 Ollama

**Sources:**

- https://github.com/ollama/ollama/issues/4643
- https://github.com/ollama/ollama/issues/5983
- https://github.com/ollama/ollama/issues/9147

What exists:

- multi-GPU behavior on one host exists in some forms;
- multi-node "one large model across machines" remains issue/feature-request
  territory, not a stable user-facing Ollama feature.

SBFB fit:

```text
Keep Ollama for default single-worker task execution.
Do not wait for Ollama to solve Ethernet Cluster Mode.
```

### 3.11 Domestic fiber reality for remote people

**Sources:**

- https://www.free.fr/freebox/freebox-ultra/
- https://reseaux.orange.fr/vos-equipements/livebox-7
- https://www.sfr.fr/offre-internet/sfr-fibre-premium-bis
- https://www.bouyguestelecom.fr/offres-internet/bbox-wifi-7
- https://www.itu.int/rec/T-REC-G.9804.3/en

What exists in France-like domestic fiber markets in 2026:

- mainstream premium residential offers can advertise up to 8 Gbit/s down and
  8 Gbit/s up, usually theoretical and eligibility-dependent;
- this usually needs compatible 10G LAN equipment, cabling, NICs, and a wired
  path, not normal Wi-Fi;
- XGS-PON-class access makes symmetric multi-gigabit home links realistic;
- 50G-PON is standardized and emerging, but it should not be treated as a
  common residential baseline yet;
- residential fiber is still an Internet path, not a dedicated cluster fabric.

Engineering interpretation:

```text
8 Gbit/s symmetric ~= 1 GB/s theoretical each way.
Real usable application throughput is lower.
RTT/jitter and routing matter more than peak throughput for model parallelism.
```

For SBFB, this means domestic fiber is excellent for:

- distributing model artifacts and caches;
- shipping repo shards, build outputs, test logs, embeddings and patches;
- routing full prompts to remote full-model workers;
- running many coding/research agents in parallel;
- redundant validation of patches, tests and proofs.

It is still weak for synchronous per-token tensor parallelism across arbitrary
homes because remote inference has synchronization barriers:

- tensor parallelism needs cross-node collectives inside many transformer
  blocks;
- pipeline parallelism adds a network hop between stages for every decoded
  token;
- WAN RTT and jitter compound at token rate;
- NAT, CGNAT, bufferbloat, ISP peering, asymmetric routes, and shared PON
  contention create unstable tail latency.

Therefore:

```text
Remote people + fiber = great for asynchronous compute sharing.
Remote people + fiber = risky for one shared real-time model.
Remote people + fiber + same metro/ISP + measured low RTT = possible R&D.
```

---

## 4. Decision Matrix

| Option | Solves model too large for one node? | Good for wired consumer PCs? | Security ready? | SBFB action |
|--------|--------------------------------------|-------------------------------|-----------------|-------------|
| Remote task decomposition | No single shared model, but solves product compute | Yes, best fit for Internet/fiber | Needs SBFB proof layer | Product default |
| Remote full-model endpoint federation | No sharding, but each user hosts models | Yes | Needs admission/secrets policy | Strong Factory path |
| Remote WAN tensor/pipeline sharding | Maybe, only strict measured links | Usually no | High risk | R&D only |
| llama.cpp RPC | Yes | Yes, with manual setup | No | First backend, wrap it |
| llama-box/GPUStack | Yes | Yes, but heavier | Better ops, still centralized | Compare, maybe external backend |
| LocalAI worker/P2P | Partial/yes via llama.cpp RPC | Possible | Own assumptions | Compare wrapper design |
| exo | Yes | Strong on Apple/RDMA, weak Linux GPU | Experimental | Watch/optional backend |
| vLLM/Ray | Yes | Better for lab/datacenter | Operator-managed | Later backend |
| SGLang | Yes | Better for lab/datacenter | Operator-managed | Later backend |
| TGI | Partial / serving-shard oriented | Better for HF serving | Operator-managed | Backend comparison |
| Petals | Yes | WAN/research focus | Public swarm privacy caveat | Research only |
| Distributed Llama | Yes | Interesting but constrained | Unknown | Research comparison |
| Ollama alone | No multi-node sharding | No | N/A | Default local worker only |

Conclusion:

```text
No existing OSS project gives SBFB's exact product:
remote/private compute group + protocol identity + signed capability registry
+ model artifact provenance + reproducible benchmark proofs.

Therefore create SBFB Remote Fiber Compute Sharing as the product path, and
SBFB Ethernet Cluster Mode as a narrower control/proof layer for measured
low-latency sharded-model R&D. Neither is a new tensor engine.
```

---

## 5. Proposed New Solution

### 5.0 Two-layer solution

For remote people, the product should be designed in two layers:

```text
Layer A: Remote Fiber Compute Sharing (RFCS)
  Internet/fiber between trusted people.
  Coarse-grained Factory/Babel tasks.
  Each worker runs a complete local model, local tools, or local tests.
  Network moves prompts, repo shards, artifacts, patches, logs and proofs.

Layer B: Ethernet Cluster Mode (ECM)
  Same LAN or measured low-latency metro link.
  One backend may shard one large model across several machines.
  Network moves backend activation/tensor traffic.
  R&D-only until benchmarks prove it.
```

Layer A is the power-sharing product. Layer B is a specialized backend mode.

### 5.0.1 Why RFCS is not "less ambitious"

The strongest Factory use case is not one remote 120B model pretending to be a
single local GPU. It is a verifiable distributed coding process:

```text
planner -> implementers -> reviewers -> test runners -> security auditors
        -> benchmark runners -> patch merger -> provenance publisher
```

Each step is coarse enough that Internet latency disappears inside the task
runtime. A 20 ms, 50 ms, or even 150 ms RTT is irrelevant when a worker spends
30 seconds to 20 minutes generating, testing, compiling, or auditing.

This is where domestic fiber changes the product:

- model files can be distributed or cached quickly;
- repository shards and build artifacts can move cheaply;
- several open-source coding models can run in parallel on different machines;
- workers can specialize by GPU/VRAM/toolchain;
- the group can spend aggregate compute on redundancy and verification.

The "big model" equivalent becomes a **committee of open-source agents with
proofs**, not necessarily one neural network split at every layer.

### 5.0.2 RFCS execution model

```text
1. Factory creates a task graph from the app goal.
2. SBFB strips secrets and builds scoped work packages.
3. Iroh distributes manifests, blobs, repo shards, and model references.
4. Remote workers claim tasks according to signed capabilities.
5. Each worker runs local tools/model in a sandbox.
6. Results return as patches, logs, test output, embeddings or review notes.
7. Factory cross-checks results with redundancy and local policy.
8. Accepted artifacts are signed into provenance.
```

Good RFCS tasks:

- implement one isolated component;
- write or repair tests;
- run full build/lint/typecheck on a different OS/GPU;
- audit a patch for security or protocol invariants;
- generate multiple solution candidates;
- translate or chunk Babel content;
- index repositories and produce embeddings;
- benchmark local models or backends;
- reproduce a failure and submit logs.

Bad RFCS tasks:

- send private secrets to unknown workers;
- require millisecond-level lockstep;
- rely on a worker's unverified claim;
- merge code without local tests/review;
- promise that remote open-source agents equal a frontier agent on all tasks.

### 5.0.3 Remote sharded-model R&D profile

WAN sharding should only be attempted when the network passes a measured
profile. It is not enough to say "fiber".

Minimum profile for any experiment:

```text
sustained uplink per worker: >= 1 Gbit/s
preferred uplink per worker: >= 5-8 Gbit/s
RTT controller <-> worker: <= 5 ms preferred, <= 10 ms hard R&D ceiling
jitter p95: <= 1 ms preferred
packet loss: effectively zero during benchmark
public exposure: none
same country/metro/ISP path: strongly preferred
```

Even then, prefer pipeline-style experiments over fine tensor-parallel
collectives, because tensor-parallel all-reduce patterns are much more
latency-sensitive.

Remote sharding can be researched as:

- **metro-fiber cluster:** friends in the same city/ISP with measured low RTT;
- **batch pipeline serving:** many queued prompts to keep the pipeline full;
- **model-stage hosting:** one node hosts coarse model stages, not tiny tensor
  slices;
- **private group only:** no public swarm, no unknown GPUs, no raw RPC port.

It should not be the default way Factory shares power.

### 5.1 Name

```text
SBFB Ethernet Cluster Mode (ECM)
```

### 5.2 One-line contract

```text
ECM lets a private SBFB group temporarily combine trusted local machines
over wired LAN to run a large model through a specialized backend, while SBFB
records the cluster manifest, model hashes, capability claims, launch profile,
benchmark results, and shutdown evidence.
```

### 5.3 Architecture

```text
SBFB daemon / private group
  |
  | iroh docs/blobs/gossip control plane
  | - invite and membership
  | - node identity
  | - capability records
  | - model artifact manifests
  | - cluster launch manifest
  | - signed benchmark evidence
  v
Cluster controller node
  |
  | backend-native data plane on private LAN
  | - llama.cpp RPC TCP/RDMA, or
  | - vLLM/Ray, or
  | - SGLang, or
  | - exo
  v
Cluster worker nodes
  - expose backend devices only to approved controller
  - report health/capabilities
  - run under local sandbox/firewall policy
```

### 5.4 Control plane responsibilities

SBFB owns:

- private group invite;
- node identity and public keys;
- allowlist of cluster members;
- hardware capability claims;
- operator consent;
- model artifact hash/digest;
- backend profile selection;
- launch command template;
- network bind policy;
- benchmark plan;
- signed run report;
- shutdown/cleanup evidence;
- warnings if the backend violates security policy.

Discovery is deliberately non-authoritative:

```text
LAN discovery produces candidates.
SBFB admission produces members.
Only admitted members can appear in a cluster manifest.
```

Backend owns:

- tensor/pipeline/data parallel execution;
- GPU kernels;
- KV cache placement;
- tensor-split;
- per-token scheduling;
- backend-native serving endpoint.

### 5.5 Minimal manifests

#### Capability entry

```json
{
  "schema_version": 1,
  "node_id": "ed25519-pubkey-or-node-id",
  "scope": "private_group",
  "lan_addrs": ["192.168.1.42"],
  "accelerators": [
    {
      "vendor": "nvidia",
      "name": "RTX 4090",
      "vram_mb": 24576,
      "backend_support": ["llamacpp_rpc_cuda"]
    }
  ],
  "network": {
    "link": "ethernet",
    "speed_mbps_observed": 2500,
    "latency_ms_observed": 0.6
  },
  "consent": {
    "max_watts": 350,
    "max_minutes": 120,
    "local_lan_only": true
  },
  "signature": "..."
}
```

#### Cluster manifest

```json
{
  "schema_version": 1,
  "cluster_id": "uuid",
  "backend": "llamacpp_rpc",
  "scope": "private_group_only",
  "controller_node": "node-a",
  "worker_nodes": ["node-b", "node-c"],
  "model": {
    "name": "Qwen/Qwen2.5-72B-Instruct-GGUF",
    "files": [
      {
        "path": "qwen2.5-72b-instruct-q2_k-00001-of-00007.gguf",
        "sha256": "..."
      }
    ]
  },
  "backend_args": {
    "n_gpu_layers": 99,
    "tensor_split": "auto",
    "ctx_size": 8192,
    "rpc_ports": [50052]
  },
  "security": {
    "rpc_exposure": "lan_allowlist_only",
    "public_network_allowed": false,
    "requires_artifact_hash_match": true,
    "requires_shutdown_evidence": true
  }
}
```

#### Benchmark report

```json
{
  "schema_version": 1,
  "cluster_id": "uuid",
  "backend": "llamacpp_rpc",
  "model_digest": "sha256:...",
  "prompt_profile": "short_chat_512_in_128_out",
  "metrics": {
    "load_seconds": 0,
    "ttft_ms": 0,
    "decode_tokens_per_sec": 0,
    "total_tokens_per_sec": 0,
    "network_rx_mb": 0,
    "network_tx_mb": 0,
    "gpu_peak_vram_mb": 0
  },
  "baseline": {
    "single_node_offload_tokens_per_sec": 0,
    "single_node_fails_to_load": true
  },
  "signature": "..."
}
```

---

## 6. R&D Benchmark Plan

### 6.1 Required hardware profiles

Minimum useful test matrix:

| Profile | Machines | Network | Goal |
|---------|----------|---------|------|
| A | 2 PCs, NVIDIA GPUs | 1GbE | Check feasibility only |
| B | 2 PCs, NVIDIA GPUs | 2.5GbE | Practical home-lab baseline |
| C | 2-3 PCs, NVIDIA GPUs | 10GbE | Real Ethernet target |
| D | 2 Macs TB5/RDMA | Thunderbolt/RDMA | exo comparison |

### 6.2 Models

Use models that expose different failure modes:

| Model class | Purpose |
|-------------|---------|
| 14B-32B GGUF | baseline sanity; should fit on one strong GPU |
| 70B GGUF | first practical "needs more memory" case |
| 100B+ / MoE GGUF | stress path if available |
| small 7B | regression/control where cluster should not be slower by surprise |

### 6.3 Metrics

Record:

- model load time;
- time to first token;
- prefill tokens/sec;
- decode tokens/sec;
- end-to-end tokens/sec;
- total wall time;
- network throughput;
- CPU utilization on controller and workers;
- GPU utilization;
- VRAM peak per device;
- prompt failure modes;
- recovery after worker disconnect;
- leakage surface: open ports and bind addresses.

### 6.4 Go/no-go thresholds

Initial R&D passes only if:

- model that cannot load on one node loads on cluster;
- repeatable launch in <= 10 minutes after artifacts are present;
- no RPC port is reachable outside allowlisted LAN scope;
- artifact hash mismatches stop launch;
- shutdown removes backend processes and closes ports;
- benchmark report is signed and reproducible;
- operator warning clearly says raw backend RPC is not safe for public networks.

Speedup is not required for first pass. The first pass is about:

```text
fit larger model + reproducible proof + bounded risk
```

Speed becomes a second pass.

---

## 7. Security Requirements

### 7.1 Hard invariants

- Private group only.
- No public relay for tensor traffic.
- No unknown workers.
- For remote Factory compute, no repository secrets, tokens, private keys or
  production credentials are included in work packages.
- Remote worker output is untrusted until verified by local policy, tests, and
  provenance checks.
- A remote worker may receive scoped code/data, not ambient authority over the
  user's machine or SBFB identity.
- Discovery never grants trust. mDNS/libp2p/EXO-style discovery only finds
  candidates; SBFB admission must explicitly authorize them.
- No untrusted model artifacts without hash pinning.
- No raw `rpc-server` exposed on `0.0.0.0` without OS firewall allowlist.
- No silent fallback from secure group to public network.
- No claim that results are private if backend sends cleartext over LAN.
- No multi-tenant GPU sharing during cluster run.
- No backend version with known unauthenticated network RCE in the allowlist.

### 7.2 RPC wrapping options

Because llama.cpp RPC is officially fragile/insecure, SBFB must add one of:

1. WireGuard/Tailscale-like private interface generated by the operator.
2. SSH tunnel per worker.
3. Local firewall allowlist plus private VLAN.
4. Future SBFB QUIC tunnel if performance is proven acceptable.

For Phase 1 R&D, the simplest acceptable rule is:

```text
Only bind rpc-server to a private LAN interface and require firewall allowlist
for the controller IP.
```

But the doc must label this as R&D-only, not production security.

For any Ray/vLLM/SGLang backend, the rule is stricter:

```text
The backend runtime must run on an isolated network segment or dedicated
interface. Tokens/API keys are defense in depth, not the trust boundary.
```

### 7.3 Failure modes to test

- worker process dies mid-generation;
- controller dies and leaves worker RPC processes open;
- wrong model hash;
- wrong tensor split;
- worker advertises fake VRAM;
- controller tries to use public IP;
- another LAN machine probes RPC port;
- slowest worker bottlenecks the run;
- model loads but decode becomes slower than single-machine offload.
- vulnerable backend version is detected and quarantined before launch;
- a discovered but non-admitted LAN peer attempts to join.

---

## 8. Implementation Shape

### 8.1 Do not modify default task queue first

The current task system should stay batch-first. Ethernet Cluster Mode should
start as a sidecar R&D command or private-group operation:

```text
nexus cluster probe
nexus cluster plan --model <gguf-manifest>
nexus cluster launch --backend llamacpp-rpc
nexus cluster bench --profile short-chat
nexus cluster stop
nexus cluster report
```

### 8.2 Candidate modules

Future code surfaces:

```text
crates/nexus-cluster-rs/              # new crate, if the spike graduates
crates/nexus-worker-core/src/cluster/ # worker-side capability probe
configs/cluster.toml.sample           # operator config
docs/security/CLUSTER_THREATS.md      # dedicated threat model
```

Do not put this directly into `nexus-shell-daemon` first. Keep it optional and
R&D-gated.

### 8.3 Backend adapter trait

Possible Rust shape:

```rust
trait ClusterBackend {
    fn probe(&self) -> ClusterProbe;
    fn plan(&self, manifest: &ClusterManifest) -> ClusterPlan;
    fn launch(&self, plan: &ClusterPlan) -> ClusterRun;
    fn stop(&self, run_id: ClusterRunId) -> StopReport;
    fn benchmark(&self, run_id: ClusterRunId, profile: BenchProfile) -> BenchReport;
}
```

First implementation:

```text
LlamaCppRpcBackend
```

It should shell out to pinned binaries at first, not embed RPC in-process.
That keeps the R&D reversible and avoids coupling SBFB releases to a fragile
backend ABI.

---

## 9. Build vs Adopt Decision

### 9.1 What we adopt

Adopt existing engines:

- llama.cpp RPC for GGUF two/three-machine tests;
- GPUStack as cluster-manager comparison;
- exo as UX/topology/RDMA reference;
- vLLM/SGLang for later lab-grade backends.

### 9.2 What we build

Build the SBFB-specific layer:

- cluster manifest;
- signed capability record;
- model artifact hash policy;
- private-group membership and consent;
- launch plan generator;
- safe bind/firewall checks;
- benchmark proof report;
- failure/shutdown evidence;
- user-visible warning language.

### 9.3 What we do not build

Do not build:

- tensor parallel kernels;
- a new LLM runtime;
- a new scheduler competing with vLLM/SGLang;
- model format conversion;
- public model-parallel swarm;
- Ollama-compatible distributed backend from scratch.

---

## 10. Future Sprint Candidate

Only after Gate 2 or explicit R&D allocation:

```text
Sprint candidate: S73+ / post-Gate 2
Title: Remote Fiber Compute Sharing for Factory
Goal: let trusted remote participants contribute local open-source model
      workers to Factory tasks, with signed artifacts and verification.
```

Suggested phases:

| Phase | Scope | Output |
|-------|-------|--------|
| A | private group task manifest | scoped task package schema |
| B | worker capability proof | model/toolchain/GPU capability entry |
| C | remote sandbox runner | signed patch/log/test result |
| D | redundant verification | n-of-m review/test policy |
| E | product decision | can it replace some Claude/Codex paths? |

Then, only after RFCS works:

```text
Sprint candidate: S73+ / post-Gate 2
Title: Ethernet Cluster Mode R&D spike
Goal: produce signed, reproducible benchmark evidence for llama.cpp RPC
      over 2-3 trusted wired machines.
```

Suggested phases:

| Phase | Scope | Output |
|-------|-------|--------|
| A | OSS refresh + threat model | `CLUSTER_THREATS.md`, source list |
| B | Manifest/probe prototype | capability + cluster manifest schemas |
| C | llama.cpp RPC runner | launch/stop/bench shell wrapper |
| D | Benchmark evidence | signed report, baseline comparison |
| E | Decision | go/no-go: productize, defer, or drop |

Exit criteria:

- `llama.cpp RPC` cluster can be launched from a pinned manifest;
- model hash mismatch blocks launch;
- benchmark report is signed;
- raw RPC ports are not exposed outside allowlist;
- docs state the feature is private LAN only;
- R&D decision does not pollute S68-S69 roadmap.

---

## 11. User-Facing Framing

Short version:

```text
Ethernet Cluster Mode lets your own trusted machines act as one local AI
workbench for a model too large for one box. SBFB handles membership,
permissions, proofs, model hashes and audit trail. The actual tensor traffic
is handled by specialized inference backends such as llama.cpp RPC.
```

Do not promise:

- "all computers become one GPU";
- "internet GPU pooling";
- "faster tokens automatically";
- "secure on any network";
- "works with Ollama cluster mode".

Safe promise:

```text
First goal: run a bigger local model, reproducibly and with proof.
Second goal: benchmark whether speed is acceptable on your wired network.
```

---

## 12. Source Index

Primary external sources checked on 2026-05-21:

- llama.cpp RPC README:
  https://github.com/ggml-org/llama.cpp/blob/master/tools/rpc/README.md
- llama.cpp SECURITY:
  https://github.com/ggml-org/llama.cpp/blob/master/SECURITY.md
- llama.cpp RPC unauthenticated RCE advisory:
  https://github.com/ggml-org/llama.cpp/security/advisories/GHSA-j8rj-fmpv-wcxw
- GPUStack overview:
  https://docs.gpustack.ai/2.0/overview/
- GPUStack built-in backends:
  https://docs.gpustack.ai/2.0/user-guide/built-in-inference-backends/
- GPUStack llama-box distributed tutorial:
  https://docs.gpustack.ai/0.6/tutorials/performing-distributed-inference-across-workers-llama-box/
- LocalAI P2P/federated inference:
  https://localai.io/features/distribute/
- LocalAI distributed mode:
  https://localai.io/features/distributed-mode/
- exo:
  https://github.com/exo-explore/exo
- vLLM distributed serving:
  https://docs.vllm.ai/en/v0.7.0/serving/distributed_serving.html
- vLLM security:
  https://docs.vllm.ai/en/latest/usage/security/
- Ray cross-node LLM parallelism:
  https://docs.ray.io/en/latest/serve/llm/user-guides/cross-node-parallelism.html
- Ray security:
  https://docs.ray.io/en/latest/ray-security/index.html
- Ray token authentication:
  https://docs.ray.io/en/latest/ray-security/token-auth.html
- SGLang DeepSeek multi-node tensor parallelism:
  https://docs.sglang.ai/basic_usage/deepseek_v3.html
- SGLang router/gateway:
  https://docs.sglang.ai/advanced_features/router.html
- SGLang multi-node deployment:
  https://sgl-project-sglang-93.mintlify.app/deployment/multi-node
- Hugging Face TGI:
  https://huggingface.co/docs/text-generation-inference/main/en/index
- Petals:
  https://github.com/bigscience-workshop/petals
- Petals paper:
  https://arxiv.org/abs/2312.08361
- Distributed Llama:
  https://github.com/b4rtaz/distributed-llama
- Ollama multi-node feature requests:
  https://github.com/ollama/ollama/issues/4643
  https://github.com/ollama/ollama/issues/5983
  https://github.com/ollama/ollama/issues/9147
- Freebox Ultra 8 Gbit/s symmetric offer:
  https://www.free.fr/freebox/freebox-ultra/
- Orange Livebox 7 / Livebox Max 8 Gbit/s symmetric offer:
  https://reseaux.orange.fr/vos-equipements/livebox-7
- SFR Fibre Premium / Box 10+ 8 Gbit/s symmetric offer:
  https://www.sfr.fr/offre-internet/sfr-fibre-premium-bis
- Bouygues Bbox Wi-Fi 7 8 Gbit/s symmetric offer:
  https://www.bouyguestelecom.fr/offres-internet/bbox-wifi-7
- ITU-T 50G-PON specification:
  https://www.itu.int/rec/T-REC-G.9804.3/en

Primary repo sources:

- `.planning/research/gpu_pooling_distributed_inference.md`
- `.planning/research/rrv_scoped_search_compute_groups.md`
- `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`
- `crates/nexus-core-rs/src/task.rs`
- `crates/nexus-worker-core/src/config.rs`
- `crates/nexus-worker-core/src/llm/mod.rs`
- `crates/nexus-worker-core/src/llm/llama_cpp.rs`
- `docs/security/COMPUTE_THREATS.md`
