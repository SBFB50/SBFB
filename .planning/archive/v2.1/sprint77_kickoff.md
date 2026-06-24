# Sprint 77 — Kickoff : Sharding pipeline (modèle 70B éclaté cross-machine)

> **Feature phare** de la roadmap v5 (PO-6, scope cut #1 de S76). S77 fait passer
> SBFB du **task-routing du modèle ENTIER cross-machine** (prouvé S76 : une carte
> sert un modèle complet) au **modèle éclaté** : un 70B (~42.5 GB Q4) trop gros
> pour une seule carte 16 GB est découpé en blocs de couches sur **2+ machines ×
> 1 GPU**, en pipeline-parallel sur iroh-QUIC. Le mono-machine 2-GPU est ENTERRÉ
> (arbitrage PO « personne n'a 2 GPU », scope cut #2 de S76 — ne PAS ressusciter).
> Le design est **déjà figé** : `.planning/research/sharding_design_addendum_sota_2026-05-30.md`
> + `remote_user_sharded_llm_rnd.md` §10 + `docs/security/SPLIT_INFERENCE_DESIGN.md`.
> S77 EXÉCUTE ce design, ne le re-conçoit pas.
>
> **RÉVISION 2026-06-20 (arbitrage PO Checkpoint §11)** : le PO a tranché le **scope
> MAXIMAL** sur D3 et D4 (directive « sprints ultra-complets, 0 defer du cœur »). D1
> accepté (fork iroh tel quel), D2/D5 confirmés. **D3 vise désormais le 70B COMPLET**
> (scheduler Parallax 2-phases complet + benchmark 70B sur 3-5 machines = livrables, pas
> post-S77). **D4 livre la vérif COMPLÈTE N0+N1+N2+N3** + l'incentive-à-vérifier
> curator-reputation (N4 zkML reste post-S77). Le spike toy 1B/7B reste comme étape de
> bring-up intermédiaire, pas comme cible finale.

**Écrit** : 2026-06-20 (post-audit gate S76 **CONDITIONAL PASS**, motif intégrité-doc
levé par `52e70d1` — condition de blocage Phase A **LEVÉE**). **Révisé 2026-06-20**
(arbitrage PO scope maximal D3/D4).
**Type** : **sprint IMPAIR** — pas de phase dette obligatoire (Règle 1 G7 ne
s'applique qu'aux sprints pairs). Les carries absorbables sont intégrés dans les
phases fonctionnelles (la convergence + la topology scheduler + la vérif touchent leurs
zones). **Sprint TRÈS LARGE assumé PO** (scope ultra-complet) : ~11 phases A-K, le
budget de phases est ouvert (README §4, jamais plafonné).
**Aucun item 3/3 MANDATORY à l'entrée** ; mais **6 carries sont à 2 reports** dont
SYBIL-SEEDER-TAIL avec exemption nommée « dépendance interne sharding » (cf. §6).
**Tip master d'entrée** : `4da9800` (poussé sur origin/master ; audit findings S76 :
**0 P0, 0 P1, 6 P2, 11 P3** — `sprint76_audit_findings.md`).
**Phase 0 audit Sprint 76** : **DÉJÀ JOUÉ et CLOS** — CONDITIONAL PASS, fix
`52e70d1` (repointage des ancres de tests fantômes, 0 code). **Phase 0 N'EST PAS
rejouée** ; le plan S77 démarre à Phase A.
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV (OPEN, S77 continue
l'arc compute, pas de nouvelle version).
**Roadmap source** : `.planning/roadmap_v5_factory_complete_vision.md`. Sprint 7/7
de l'arc compute réel (S71-S77 — l'amendement S75-G a inséré la découverte PULL,
décalant le sharding de S76 vers S77). C'est le **dernier sprint de l'Arc 3.5
Factory Complete Vision** au sens compute distribué.

---

## Sources context7 + WebSearch consultées (pre-gel)

Recherche factuelle (G9 + G2 + G10) AVANT gel des D1-D5. Le sharding est un domaine
crypto/wire-format/réseau pointu : le seuil de recherche est élevé (5+ sources OSS,
5+ alternatives, lecture code OSS pas README). Toutes accédées **2026-06-20** sauf
indication. Le code SBFB ancré est listé dans chaque bloc §4 « Implications code ».

### Recherche pré-existante (R&D figée, fondation)
- **Addendum SOTA sharding** `.planning/research/sharding_design_addendum_sota_2026-05-30.md`
  (2026-05-30, <90j) — pipeline-parallel exclusif, ALPN `sbfb/shard/1`, schéma N0-N4,
  scheduler 2-phases (placement DP + k-medoids RTT + routing DAG + churn Petals),
  enveloppe perf chiffrée, threat model SI-1..SI-5. **Décisions de design figées.**
- **R&D antérieure** `remote_user_sharded_llm_rnd.md` (2026-05-21) §10 — wire primitives
  (`ComputeGroup`, `WorkerCapability`, `ShardPlan`, `ShardAssignment`,
  `ShardedSessionManifest`, `RunProof`), sizing VRAM §6, contraintes NAT/CGNAT §7.
- **`docs/security/SPLIT_INFERENCE_DESIGN.md`** (2026-04-27) — patterns BOINC/Truebit/
  Golem/Split-Learning, surfaces SI-1 (reconstruction activations, High) .. SI-5
  (latence side-channel, Low).

### D1 — Convergence delivery WAN (prérequis dur)
- **iroh-docs / iroh-gossip** — context7 `/websites/rs_iroh_0_95_1_iroh` + docs.rs
  iroh-gossip 0.96, lus 2026-06-20 : iroh-gossip = « epidemic broadcast trees to
  disseminate messages among a swarm of peers interested in a topic » ; iroh-docs =
  « eventually consistent key-value storage with built-in sync protocol » s'appuyant
  sur gossip + blobs. La propagation live d'une entrée dépend du **gossip neighborhood**
  formé sur le doc.
- **iroh 1.0 stable** — WebSearch 2026-06-20 (byteiota « Iroh 1.0: Dial Keys, Not
  IPs ») : iroh 1.0 publié juin 2026. **G2 trigger** : signal d'upgrade, mais le projet
  reste pin **0.98** (Day-0 gelé, upgrade = Gate-1/PO, PAS S77). Évalué : INCHANGÉ pour
  S77 (la convergence se résout dans le câblage SBFB ou se révèle être un BLOCK PO).
- **Bug live S76** — `sprint76_verification.md` §5.1 (2026-06-19) : `dispatch_loop.rs`
  écrit `task:{id}` via `doc.set` ; l'entrée incrémentale post-subscribe n'atteint pas
  la réplique distante (`recv:0`, gossip neighborhood non formé), reproduit **LAN +
  WAN**. Germe de test documenté `process_evolution_commit2_handoff.md` l.58-60.

### D2 — Data plane ALPN `sbfb/shard/1`
- **iroh Router / ALPN** — context7 `/websites/rs_iroh_0_95_1_iroh` 2026-06-20 :
  `Router::builder(endpoint).accept(alpn, handler).spawn()` ; `ProtocolHandler::accept(&self,
  connection: Connection)` ; `Connection::open_bi`/`accept_bi` — « streams very cheap
  to create … long-lived streams » ; « the initiator of [a bi] stream has to send data
  before the peer will be aware ». Point d'insertion SBFB confirmé : `node.rs:385`
  `Router::builder(...).accept(BLOBS_ALPN,...).accept(GOSSIP_ALPN,...).accept(DOCS_ALPN,...)`
  + `extra_protocols` (déjà utilisé par `SEED_ALPN = "sbfb/seed/0"` `node.rs:68`).
- **llama.cpp RPC backend** — WebSearch + README ggml-org master 2026-06-20 : `rpc-server`
  + `-DGGML_RPC=ON`, « exposing ggml devices on a remote host », « TCP-based communication
  protocol », `ggml_backend_rpc_start_server`. **Aucune vérification Byzantine, blind-trust,
  orienté LAN/Jetson** (Seeed Studio Jetson wiki, gpustack/llama-box). REJETÉ comme data
  plane (cf. §4 D2).
- **Lattica** (arXiv 2510.00183, Rust+QUIC) — « jumeau iroh » mais dépendance dupliquée
  (re-pose le transport). REJETÉ comme dép.

### D3 — Pipeline-parallel + scheduler latency-aware
- **Petals** (arXiv 2209.01188, 2312.08361, MIT) — WebSearch 2026-06-20 : « BLOOM-176B…
  1.71 steps/s single-batch, 253.6 tok/s parallel forward at 1Gbit/<5ms RTT ; inference
  does NOT depend much on bandwidth, degrades with higher latency ». Churn = `replace_failed_server`
  re-balancing actif.
- **Parallax** (arXiv 2509.26182, Apache-2.0) — WebSearch 2026-06-20 : « 3.1× lower
  end-to-end latency, 5.3× better inter-token latency, 3.1× higher throughput vs Petals…
  pipeline parallelism base strategy… two-phase scheduler jointly optimizing model
  placement and runtime chain selection ». Datacenter 10ms RTT (borne sup optimiste, pas
  de test NAT résidentiel).
- **Privacy-Aware Split Inference over WAN** (arXiv 2602.16760, <90j) + **Trust-Aware
  Routing at the Edge** (arXiv 2603.28622, <90j) — WebSearch 2026-06-20 : confirment le
  layer-split WAN + le routing trust-aware comme axes de recherche actifs 2026.
- **Code local** : `gpu/mod.rs:107` `GpuInfo.vram_free_bytes` (mesuré réel via NVML), `gpu/nvml.rs`,
  `gpu/profile.rs` — water-filling sur VRAM libre faisable.

### D4 — Vérification shard N0-N3
- **TOPLOC** (arXiv 2501.16007 + github PrimeIntellect-ai/toploc) — WebSearch 2026-06-20 :
  « locality-sensitive hash over top-k (k=128) final hidden layer activations, 258 B/32
  tok, 100% detection of model/precision swap, ~100x faster than re-running, millions×
  lower memory than zkLLM ».
- **DiFR** (arXiv 2511.20621, <90j) — « Inference Verification Despite Nondeterminism,
  AUC>0.999 en 2 tokens » ; faille = doit randomiser temperature ET seed.
- **VeriLLM** (arXiv 2509.24257) — spot-check prefill-only ~1%, one-honest-verifier ;
  reward on-chain (que SBFB ne peut pas répliquer — kudos non-monétaire).
- **opML** (arXiv 2401.17555) + **SENTINEL** (arXiv 2603.03592, <90j) — bissection sur
  commitments d'activations + EMA inter-stages pour localiser le stage corrompu.
- **TensorCommitments** (arXiv 2602.12630, <90j) + **EigenAI** (arXiv 2602.00182, <90j)
  — alternatives concurrentes fraîches (lightweight verifiable inference / deterministic
  verifiable results).
- **Code local** : `verification.rs` (3-layer : signature L1 / model_digest L2 / logprobs
  L3) — `logprobs_hash` L3 existant mais inerte (32 zéros) ; `llama_cpp.rs:41-52` backend
  feature-gated `llm_llama_cpp`, `#[ignore]`-gated, **jamais en CI** (gap factuel D4) ;
  `llama-cpp-2 = "0.1.143"` (Cargo.toml:362) in-process expose les logits ; `rerun.rs`
  (sampling 1%), `redundancy.rs` (majority vote).

### D5 — Mode groupe privé Ed25519
- **invite M19** — `invite.rs` (paire `(project_id, archive_hash)` liée, Ed25519+JCS,
  `INVITE_FORMAT_VERSION`) + `DOMAIN_SEED_REQUEST_V1` pattern (`canonical.rs:214`).
- **TEE GPU consumer** — addendum §4 + WebSearch : inexistant en 2026 → aucun claim de
  confidentialité face aux workers (SI-1/SI-4). REJETÉ comme mitigation.
- **R-iroh-audit P0** (CLAUDE.md zones rouges, inchangé) → pilote fermé, jamais public.

---

## §1 Constat d'entrée

### §1.1 D'où on part

S76 a clos l'Arc 3.5 Factory Complete Vision 6/6 en prouvant le **task-routing du
modèle ENTIER cross-machine** : panneau « offrir ma puissance », worker co-localisé,
E2E cross-machine compute (B-3), cohorte homogène `RuntimeTuple` advisory, quorum
redundancy>1 déterministe + fix bridge result-sync, dashboard contributeur,
quantization 4-bit doc-only. L'audit gate S76 (joué en S77 Phase 0) verdict
CONDITIONAL PASS — 0 P0/P1, 6 P2/11 P3, motif intégrité-doc levé par `52e70d1`.

S77 ouvre la **dernière pièce** du puzzle compute : le modèle qui ne tient pas sur une
seule carte. Un 70B Q4 ≈ 42.5 GB > 16 GB (RTX 5080) → il faut l'**éclater** en blocs de
couches Transformer servis par des machines distinctes, chaînées en pipeline. C'est le
livrable R&D le plus exigeant de l'arc, mais le design est figé (addendum SOTA) et
S76 a livré l'infra de routing cross-machine réutilisable. **Sur arbitrage PO scope
maximal (2026-06-20), S77 vise le 70B COMPLET** (scheduler Parallax 2-phases + vérif
N0-N3 + benchmark 3-5 machines), pas seulement un spike toy.

**Le constat factuel honnête** : S77 hérite d'un **prérequis bloquant non résolu** —
la convergence delivery WAN. Le live attempt S76 (`verification.md` §5.1, 2026-06-19,
matériel présent) a montré que les tâches `task:` créées **après** la souscription du
worker distant ne se propagent pas à sa réplique (sync initial bulk-only). Sans cette
convergence, aucune sous-tâche de shard ne peut atteindre N workers distants en live.
**S77 Phase A commence donc par diagnostiquer puis fixer ce bug**, pas par coder du
shard.

### §1.2 Ancrage roadmap v5

Roadmap v5 PO-6 : « GPU distribué / mini data centers — Sharding WAN prioritaire
(feature phare) : un gros modèle découpé entre cartes de nœuds distants, en assumant
1-3 tok/s batch/async + la vérif des shards. » Position : arc compute S71-S77, sprint
7/7. Dépendances **amont satisfaites** : S75 découverte PULL node-centrique + S76
task-routing cross-machine + quorum + cohorte homogène. Dépendances **aval** : aucune
(dernier sprint de l'arc ; post-S77 = N4 zkML opt-in Gate-4+ + durcissement si le
benchmark 70B révèle des gaps, ou GO/NO-GO produit selon les mesures réelles).

### §1.3 Compteurs tests entrée (tip `4da9800`)

| Suite | Count |
|---|---|
| Rust nextest (Windows natif) | 1805 |
| Rust nextest (Docker canonique +`cfg(unix)`) | 1809 |
| Vitest (`web/`) | 402 |
| Vitest factory-operator | 7 |
| size-limit | 6/6 |
| E2E Playwright hermétique (`web/e2e/`) | 39 PASS (13 specs) |
| **Total ~** | **~2260** |

> Note : compteurs post-S76 Phase H (`1d2fb36` : +1 Rust `project_info` → 1805 ; +4
> Vitest fixture B10 → 402) + E2E vague 2 (`4da9800` : 39 specs). Le tip `4da9800` est
> poussé sur origin. Les rows fail-fast (§ plan) ré-établiront la baseline mesurée.

### §1.4 Pre-launch protocol policy (rappel)

S77 ajoute des wire formats (ALPN `sbfb/shard/1`, `ComputeGroup`, `ShardPlan`,
`ShardAssignment`, `ShardedSessionManifest`, `RunProof`, commitments N3). La pre-launch
policy s'applique :
- **Nouvelles primitives = `schema_version: 1`** (pas de bump, ce sont des net-new
  pré-launch). Les `DOMAIN_*` nouveaux (`DOMAIN_COMPUTE_GROUP_V1`, `DOMAIN_SHARD_PLAN_V1`,
  `DOMAIN_RUN_PROOF_V1`, `DOMAIN_ACTIVATION_COMMIT_V1`) suivent le pattern additif
  `DOMAIN_SEED_REQUEST_V1` (S74).
- **`FEED_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` / `SCHEMA_VERSION` restent à 1.**
  Un manifest de session shard / une perf-map est une opération raw-op extensible si
  propagée par le feed/doc ; sinon c'est un wire point-à-point ALPN (hors feed).
- **`#[serde(default)]`** légitime pour la robustesse runtime des nouveaux champs ;
  documenter le rationale runtime-tolerance (pas historical-compat).
- **Pas de tolerant decoder multi-version** : on édite le canonical librement
  jusqu'au tag v1.0.
- **iroh 0.98 reste pinné** : iroh 1.0 stable (juin 2026) est un signal G2 d'upgrade,
  mais l'upgrade est une décision Gate-1/PO hors-S77. Si la convergence WAN ne se résout
  qu'avec une primitive iroh 1.0, c'est un BLOCK à arbitrer (cf. D1).

---

## §2 Goal

Livrer le **sharding pipeline cross-machine COMPLET** : un modèle 70B Q4 (~42.5 GB)
découpé en blocs de couches sur **3-5 machines × 1 GPU**, exécuté en pipeline-parallel
sur iroh-QUIC (`sbfb/shard/1`), avec un **scheduler Parallax 2-phases** (placement DP
water-filling VRAM mesurée + k-medoids RTT + routing DAG layer-indexé + churn
re-balancing actif Petals), chaque shard **vérifié** par le schéma gradué **N0+N1+N2+N3**
(TOPLOC fingerprint + spot-check VRF + redondance tolérante + bissection opML/SENTINEL)
adossé à un **incentive-à-vérifier curator-reputation** (kudos réputationnel non-monétaire,
pas de stake), le tout restreint à un **groupe privé explicite** (`ComputeGroup` allowlist
Ed25519). Le spike toy 1B/7B sur 2-3 machines est l'étape de **bring-up intermédiaire** ;
la **cible mesurée finale = le 70B Q4 sur 3-5 machines** (gate produit). Verdict perf
honnête écrit : 1-3 tok/s WAN batch/async, **jamais chat live**, NO-GO si RTT/frontière
> 80ms ou relais hot-path. N4 zkML reste hors-scope (opt-in Gate-4+, prohibitif).

**Critère SMART : toutes les rows fail-fast vertes au verification.md, mesure binaire à
la phase wrap-up** (cf. `sprint77_plan.md §Fail-fast checklist`, ~40 rows exécutables —
c'est LA source of truth mesurable du goal, pas une liste de KPIs inventés). La row T1
E2E Playwright hermétique (`web/e2e/compute-shard.spec.ts`) et la row T2 acceptance
(`b3_shard_pipeline.sh`, étendu 70B + RunProof N0-N3) sont bloquantes au wrap-up (gate de
testabilité README §4). La feature cross-machine sans **test de convergence vert + `b3`
au minimum au stage `claim`** reste PROVISIONAL + carry P1 ; l'acceptance 70B LIVE sera
vraisemblablement PROVISIONAL-matériel (5 PC + GPU) → verdict JSON honnête, jamais un
DIFFERE en prose.

---

## §3 Phase 0 — Audit gate Sprint 76

**DÉJÀ JOUÉ et CLOS.** Verdict **CONDITIONAL PASS** (`sprint76_audit_findings.md`,
orchestration Workflow ultracode 19 agents anti-anchoring) : 0 P0, 0 P1 (4 candidats
P1 du plan TOUS réfutés code-first), 6 P2, 11 P3. Invariants headline tous tenus
(snapshot consent 0-bump, fix bridge quorum rouge-avant-vert, validator inchangé, P1
`generation_time_ms` root-fixé, backend quant inchangé, 0 bump wire, fmt 0 sous les 2
toolchains). Seul motif CONDITIONAL = intégrité-doc (3 noms de tests fantômes dans
`verification.md` §6, propagés) → **CORRIGÉ `fix(sprint76)` `52e70d1`** (repointage vers
fonctions réelles, 0 code). **Condition levée, S77 Phase A débloqué.** Le plan S77
démarre à Phase A — Phase 0 N'EST PAS rejouée. Les 6 P2 sont routés au §6 (carries) ; le
nouvel invariant de clôture (« tout nom de test cité grep-résout à une fn `#[test]` »)
est porté au plan.

---

## §4 Decisions Day 0 (D1..D5 gelées)

### D1 — Convergence delivery WAN d'abord : diagnostiquer puis fixer le live-sync `task:` AVANT tout code de shard

**Sources consultées** :
- context7 `/websites/rs_iroh_0_95_1_iroh` queried 2026-06-20 : iroh-docs s'appuie sur
  gossip (epidemic broadcast) ; la propagation live d'une entrée dépend du neighborhood
  formé sur le doc.
- WebSearch 2026-06-20 « iroh 1.0 » (byteiota) : iroh 1.0 stable juin 2026 (G2 trigger,
  pin 0.98 maintenu).
- `sprint76_verification.md` §5.1 (2026-06-19) : symptôme précis du bug live.
- Code OSS germe `process_evolution_commit2_handoff.md` l.58-60 : test 2-nœuds iroh.
- Code local : `dispatch_loop.rs:23-60` (`doc.set(author, key, value)` sole writer),
  `result_sync.rs:159` (`spawn_result_subscribe` `DocsLiveEvent::InsertRemote` + boot
  catch-up), `docs.rs:248` (`set`), `docs.rs:365` (`subscribe`).

**Retenu** (D1 accepté tel quel par le PO, fork iroh inclus) : la convergence delivery
WAN est le **prérequis dur** de tout le sharding — sans propagation live des sous-tâches
de shard aux N workers distants, le pipeline ne peut pas se monter. Phase A ouvre par un
**diagnostic rouge-d'abord** : un test d'intégration Rust 2-nœuds (vrai discovery iroh,
pas un handshake in-process pré-partagé) qui écrit une entrée `task:` **incrémentale après
subscribe** et asserte qu'elle se propage à la réplique distante. Ce test reproduit le
bug observé (le germe documenté). PUIS, selon la cause-racine :
- si le trou est dans le câblage SBFB (download policy manquante, re-subscribe absent,
  auteur/permission, gossip topic non joint sur le doc projet) → `fix(sprint77)` dans la
  phase, test rouge→vert ;
- si la cause est un comportement intrinsèque d'iroh-docs 0.98 nécessitant une primitive
  absente → **BLOCK à arbitrer PO** (upgrade iroh 1.0 hors-Day-0, ou re-architecture du
  delivery — ne PAS masquer ce fork).

**Implication code** : `crates/nexus-shell-daemon/src/dispatch_loop.rs`,
`result_sync.rs`, `crates/nexus-core-rs/src/docs.rs` (subscribe/sync/download policy),
`crates/nexus-core-rs/src/node.rs` (gossip join sur le doc projet). Test net-new
`convergence_*` à côté de `dispatch_loop.rs`/`result_sync.rs`.

**Rejeté** :
- **Poll périodique au lieu de subscribe** (le worker re-scanne `task:` toutes N s) :
  REJETÉ — masque le bug réel (le live-sync iroh-docs cassé) au lieu de le corriger,
  ajoute de la latence morte, et le sharding exige une propagation sub-seconde des
  activations (les tâches de shard sont fréquentes). Band-aid contraire au principe
  no-band-aid (`feedback_approach`).
- **Relais N0 dans le hot-path delivery** : REJETÉ — l'addendum §4 verrouille le NO-GO
  relais hot-path (latence morte) ; le delivery doit converger en direct (UDP/hole-punch).
- **Canal HTTP push parallèle** (le coordinateur POST les tâches au worker) : REJETÉ —
  re-pose toute l'auth cross-host déjà résolue par invites M19, exige un port worker
  exposé (NAT hostile), contourne la réplication content-addressed iroh-docs.

### D2 — Data plane des activations = ALPN custom `sbfb/shard/1` sur iroh-QUIC `open_bi` long-vécu

**Sources consultées** :
- context7 `/websites/rs_iroh_0_95_1_iroh` queried 2026-06-20 : `Router::builder().accept(alpn,
  handler).spawn()`, `ProtocolHandler::accept(&self, connection)`, `Connection::open_bi`/`accept_bi`
  long-lived streams, « initiator must send data before peer is aware ».
- WebSearch + README ggml-org/llama.cpp master 2026-06-20 : RPC backend = TCP,
  `rpc-server` expose ggml devices bruts, blind-trust, LAN/Jetson.
- addendum SOTA §4 (figé) : data plane `sbfb/shard/1`, 1 Connection QUIC persistante par
  paire de shards consécutifs, `open_bi` long-vécu réutilisé pour tous les tokens,
  framing longueur-préfixe ; `conn.stats()` (RTT path, jitter) alimente la perf-map.
- Code local : `node.rs:68` `SEED_ALPN = "sbfb/seed/0"`, `node.rs:385`
  `Router::builder(...).accept(...).accept(...).accept(...)` + `node.rs:395` boucle
  `extra_protocols` (point d'insertion exact).

**Retenu** (D2 confirmé tel quel par le PO) : un **ALPN custom `sbfb/shard/1`** enregistré
via le mécanisme `extra_protocols` existant (miroir exact de `SEED_ALPN` S74). Les
activations intermédiaires (hidden states) voyagent sur une **Connection QUIC persistante**
entre shards consécutifs, via un `open_bi` long-vécu réutilisé pour tous les tokens
(jamais re-connect/token), framing longueur-préfixe. PAS via iroh-blobs (le hashing BLAKE3
= latence morte inadaptée au live) ni iroh-docs/gossip (inadaptés au point-à-point
haute-fréquence). Le control plane (manifests, claims, RunProof, fingerprints, perf-map)
reste sur iroh-docs/blobs/gossip réutilisés. `conn.stats()` alimente la perf-map du
scheduler (D3) sans ping applicatif.

**Implication code** : `crates/nexus-core-rs/src/node.rs` (registration ALPN +
`ShardProtocol` handler `extra_protocols`), nouveau module `crates/nexus-core-rs/src/shard.rs`
(framing, primitive de connexion), `crates/nexus-worker-core/src/` (côté worker shard).
0 dép nouvelle (réutilise l'endpoint iroh 0.98).

**Rejeté** :
- **llama.cpp RPC backend (`-DGGML_RPC=ON`, `rpc-server` TCP)** (README ggml-org master
  2026-06-20) : REJETÉ — protocole TCP en **blind-trust** (le `rpc-server` expose les
  ggml devices bruts, **aucune vérification Byzantine**, conçu pour LAN/Jetson confiant) ;
  incompatible avec « peers non-confiants » (le travail original SBFB) ; duplique le
  transport iroh-QUIC (gelé) ; pas de NAT/hole-punch.
- **Tensor-parallel (all-reduce par couche)** (exo/distributed-llama/prima.cpp) : REJETÉ
  — exige des all-reduce par couche → LAN/NVLink only, exclu hors datacenter (addendum
  §1, finding « degrades with latency »).
- **Data plane sur iroh-blobs** : REJETÉ — content-addressing BLAKE3 ajoute un hashing
  par message = latence morte inadaptée au flux d'activations live.
- **DHT Hivemind/Lattica** (Lattica arXiv 2510.00183) : REJETÉ — dupliquerait
  iroh-gossip (pin gelé) = wire + dép concurrents ; écosystème hivemind en sommeil
  (sept 2023).

### D3 — Pipeline/layer-split EXCLUSIVEMENT + scheduler Parallax 2-phases COMPLET + benchmark 70B sur 3-5 machines (SCOPE MAXIMAL, arbitrage PO)

> **AMENDEMENT PO 2026-06-21 — le benchmark 70B sur 3-5 machines est ABANDONNÉ.** Le rig
> réel disponible = **RTX 5080 (16 Go CUDA) + Mac M2 (8 Go Metal)** ; un 70B (~40 Go) est
> hors de portée. Nouvelle cible d'acceptation (Phase K) : un modèle **~20 Go arch-llama**
> éclaté sur ces **2 machines hétérogènes**, chacune ne chargeant QUE ses couches. Le
> **mécanisme reste inchangé** (pipeline/layer-split, scheduler Phase D/E déjà codé) ; seule
> l'échelle de démo change. Conséquence dure : le **chargement partiel des couches (P-D)
> devient OBLIGATOIRE** en Phase F (un 20 Go ne tient sur aucune machine seule). Le backend
> d'exécution du bloc passe du wrapper safe (infaisable, DESIGN-CONFLICT préflight F) au
> **fork llama.cpp prouvé** (spike GO : bit-exact CPU/CUDA/Metal + cross-backend cosine 0.999
> sur Mistral-7B Q4). Cf. `sprint77_phase_f_spike.md` + `sprint77_phase_f_preflight.md` §Résolution.

**Sources consultées** :
- WebSearch 2026-06-20 : Petals (arXiv 2209.01188) 1.71 steps/s à <5ms, « degrades with
  higher latency, not bandwidth » ; Parallax (arXiv 2509.26182) « 3.1× lower latency,
  two-phase scheduler placement+chain » ; Privacy-Aware Split Inference over WAN (arXiv
  2602.16760, <90j) ; Trust-Aware Routing at the Edge (arXiv 2603.28622, <90j).
- addendum SOTA §2/§5/§6 (figé) : scheduler hybride Parallax placement + Petals churn
  (DP water-filling VRAM libre, k-medoids matrice RTT pairwise, routing DAG layer-indexé
  sweep DP G→D `dp2(l+1,g') = min(..., dp2(l,g) + rho<g,g'> + tau<g',l+1>)`, churn
  `replace_failed_server` O(t) + cache activations client-side + heap fallback, perf-map
  republiée 1-2s iroh-docs), seuil sharding (`VRAM_modèle > VRAM_max_worker`), enveloppe
  perf (1-3 tok/s WAN, NO-GO RTT>80ms ou relais hot-path).
- Code local : `gpu/mod.rs:107` `GpuInfo.vram_free_bytes` (NVML mesuré), `gpu/nvml.rs`,
  `gpu/profile.rs`, `node.rs` `conn.stats()`.

**Retenu (SCOPE MAXIMAL — arbitrage PO Checkpoint §11, override du finding G1 D3)** :
pipeline/layer-split EXCLUSIVEMENT (1 shard = bloc contigu de N couches Transformer/
worker, KV-cache local). On optimise le **RTT et le nombre de hops**, pas le débit
(finding empirique verrouillé). **Le scheduler Parallax 2-phases COMPLET entre en S77** :
- **Phase 1 placement (montage de session)** : DP + contrainte VRAM/worker (water-filling
  sur VRAM **libre mesurée** `GpuInfo.vram_free_bytes`, pas déclarée) + contrainte de
  lien ; grouper les couches consécutives entre peers à faible RTT mutuel via **clustering
  empirique k-medoids sur matrice RTT pairwise mesurée** (`conn.stats()`, pas de géoIP
  central).
- **Phase 2 routing (par requête)** : DAG layer-indexé + 1 sweep DP G→D (relaxation
  Parallax), O(L·R²) négligeable à 3-5 peers.
- **Churn** : re-équilibrage **ACTIF de Petals** (heap de fallback ordonné par latence +
  cache client-side d'activations + `replace_failed_server` O(t)), PAS le « clé DHT
  expire » de Parallax (faille churn). Perf-map (rho, tau) republiée toutes 1-2s dans
  iroh-docs.

Le **benchmark 70B private group sur 3-5 PC** est le **gate produit** S77 (TTFT/tok-s/
jitter/drop + test worker-drop). Séquencement INTERNE : bring-up spike toy 1B/7B sur 2-3
machines (étape intermédiaire, prouve l'ALPN + open_bi + RunProof), PUIS 70B Q4 sur 3
machines, PUIS 5 machines (garde-fou addendum §7 « commencer 2-3 puis 5 » = séquencement
de phases, PAS scope cut). Seuil sharding : ne sharder QUE si `VRAM_modèle > VRAM_max_worker`
(sinon endpoint federation, plus simple). Gate réseau honnête : GO si p95 RTT/frontière ≤
20-30ms, direct, pas de relais ; **NO-GO si RTT>80ms ou relais hot-path** (sharding refusé,
pas dégradé silencieusement). Verdict produit écrit : 1-3 tok/s batch/async, **jamais chat
live**.

**Implication code** : nouveau module scheduler (`crates/nexus-coordinator-rs/` ou
`nexus-shell-daemon-core/`) : placement water-filling + k-medoids RTT + routing DAG + churn
actif + perf-map iroh-docs ; `crates/nexus-worker-core/src/gpu/` (`vram_free_bytes`) ;
`node.rs` `conn.stats()` (RTT/jitter) ; primitives `ShardPlan`/`ShardAssignment`/
`ShardedSessionManifest` (`remote_user_sharded_llm_rnd.md` §10.3) dans `nexus-core-rs`.

**Rejeté** :
- **Tensor-parallel** : REJETÉ (cf. D2, all-reduce LAN-only).
- **k-medoids géoIP central** : REJETÉ — SBFB n'a pas d'autorité géo ; clustering
  empirique sur RTT pairwise mesuré seulement (la version centrale serait une
  recentralisation).
- **Churn « clé DHT expire » de Parallax** : REJETÉ (addendum §2) — ne re-route jamais
  mid-inference (faille churn) ; on prend le `replace_failed_server` actif de Petals.
- **Endpoint federation pour le cas cible** : REJETÉ pour le 70B (c'est déjà S76 pour les
  modèles qui tiennent sur une carte) — mais conservé comme **fallback** : ne sharder QUE
  si `VRAM_modèle > VRAM_max_worker` (addendum §5 seuil), sinon endpoint federation. Le
  sharding est un coût (latence somme des frontières + fragilité worker le plus lent +
  churn), pas un objectif gratuit.
- **Streaming token-par-token interactif (chat live)** : REJETÉ (NO-GO produit) — 1-3
  tok/s WAN multi-hop = chat live non viable ; batch/async uniquement.

### D4 — Vérification shard COMPLÈTE N0+N1+N2+N3 + incentive-à-vérifier curator-reputation (SCOPE MAXIMAL, arbitrage PO) — N4 zkML reste post-S77

**Sources consultées** :
- WebSearch 2026-06-20 : TOPLOC (arXiv 2501.16007 + github PrimeIntellect-ai/toploc) ;
  DiFR (arXiv 2511.20621, <90j) ; VeriLLM (arXiv 2509.24257) ; opML (arXiv 2401.17555) ;
  SENTINEL (arXiv 2603.03592, <90j) ; TensorCommitments (arXiv 2602.12630, <90j) ; EigenAI
  (arXiv 2602.00182, <90j) ; zkLLM (prohibitif).
- addendum SOTA §3 (figé) : schéma N0-N4, sweet-spot N0 systématique + N1 1-5% + N2
  haute-criticité + N3 sur contestation + N4 opt-in Gate-4+ ; incentive §8 Q1 non résolu.
- Code local : `verification.rs` (3-layer signature/digest/logprobs ; L3 inerte 32 zéros) ;
  `llama_cpp.rs:41-52` (backend feature-gated `llm_llama_cpp`, `#[ignore]`-gated, **jamais
  en CI**) ; `llama-cpp-2 = "0.1.143"` (Cargo.toml:362) in-process expose les logits ;
  `task.rs:374` `model_digest` = blake3(name) ; `task.rs:383` `logprobs_hash` slot L3 ;
  `rerun.rs` (sampling 1%), `redundancy.rs` (majority vote).

**Retenu (SCOPE MAXIMAL — arbitrage PO Checkpoint §11, override du finding G1 D4)** :
schéma de vérification COMPLET **N0+N1+N2+N3 en S77** (N4 zkML reste post-S77).
- **N0 = TOPLOC fingerprint systématique** : LSH top-k (k=128) du dernier hidden state,
  258 B/32 tok, détection 100% du swap modèle/précision — **remplace le Layer3 logprob
  fragile** de `verification.rs`. `task.rs:383` `logprobs_hash` devient le slot TOPLOC réel.
- **N1 = spot-check VRF Ed25519** : vérifieur tiré par VRF (one-honest-verifier),
  prefill-only VeriLLM ~1% OU Activation-DiFR (AUC>0.999 en 2 tokens) ; **doit randomiser
  temperature ET seed** (faille DiFR). Étend `rerun.rs` (sampling 1%).
- **N2 = redondance tolérante** : M-of-N, comparaison fingerprint TOLÉRANT (pas hash byte,
  qui casse sous non-déterminisme GPU). Chemin ADDITIF ; le quorum result_text exact
  existant (`validate_quorum_pre_guardrail`) reste INCHANGÉ. Étend `redundancy.rs`.
- **N3 = bissection opML-style sur contestation** : commitments d'activations par-frontière
  (ancres iroh-docs + commit-reveal Ed25519, `DOMAIN_ACTIVATION_COMMIT_V1`, PAS de
  smart-contract) ; **SENTINEL** (EMA inter-stages) localise le stage corrompu. O(1 bloc).

**Mapping criticité-tâche → niveau de vérif** (addendum §3) : haute-criticité = N2
obligatoire (ex. tâche verifiable redundancy>1) ; faible-criticité = N0 seul ; N1 par
échantillonnage VRF 1-5% systématique ; N3 sur litige uniquement.

**Incentive-à-vérifier curator-reputation (livrable de design, addendum §8 Q1)** : un
lazy verifier rationnel ne vérifie pas, et le kudos non-monétaire interdit le stake (PO-12
gelé, vision `vision_model`). Piste retenue : le vérifieur tiré par VRF gagne du **kudos
réputationnel** (non-monétaire, non-transférable) pour un spot-check honnête, attribué via
le mécanisme curator/reputation existant. **Note honnête écrite** : c'est une mitigation
**réputationnelle**, PAS une garantie économique — un adversaire qui ne valorise pas sa
réputation peut toujours ne pas vérifier ; le pilote fermé (groupe privé D5) + l'anti-Sybil
amont (PoW/AgeWitness/invite M19) bornent le risque. Documenté THREAT_MODEL §16 sev M.

**Dépendance backend explicite** : N0/N1 exigent le hidden state → le mode sharding
**impose `llm_llama_cpp`** (Ollama HTTP ne l'expose pas sans fork interdit ; dégradé à
signature+redondance N2 sur Ollama). Comme ce backend n'a jamais tourné en CI, S77 livre
**deux niveaux de test** par niveau de vérif : (i) une primitive **hermétique** (encodage/
comparaison de fingerprints + bissection sur activations fixtures) qui **tourne en CI** ;
(ii) un test d'intégration `#[ignore]`-gated (GGUF requis) exercé localement.

**Implication code** : `crates/nexus-core-rs/src/verification.rs` (N0 remplace L3 +
primitive TOPLOC) ; `crates/nexus-worker-core/src/llm/llama_cpp.rs` (extraction top-k
hidden state in-process via `llama-cpp-2`) ; `task.rs:383` `logprobs_hash` slot TOPLOC réel ;
`rerun.rs` (N1 VRF + DiFR) ; `redundancy.rs` (N2 tolérant) ; nouveau module N3
(commit-reveal activations + SENTINEL EMA) + `canonical.rs` (`DOMAIN_ACTIVATION_COMMIT_V1`) ;
`validator.rs` (N2 additif, quorum exact INCHANGÉ) ; mécanisme curator-reputation
(incentive VRF).

**Rejeté** :
- **N4 zkML / preuve ZK d'inférence** (zkLLM arXiv 2404.16109, DeepProve/NANOZK) : REJETÉ
  pour S77 (le PO a demandé N1/N3, PAS N4) — 803s proving / forward pass 13B,
  « prohibitively expensive » (auteurs), 50-15000× overhead ; classé « opt-in Gate-4+ »
  (addendum §3). Post-S77.
- **Hash byte-exact cross-GPU** (le quorum S76) : REJETÉ pour les shards — l'exact-match
  cross-GPU hétérogène n'est PAS garanti (Ingonyama, Thinking Machines) ; un pipeline de
  shards traverse forcément des cartes différentes → fingerprint tolérant obligatoire (N2).
- **Petals no-verif (confiance simple)** : REJETÉ — Petals n'a aucune vérification
  Byzantine ; supprimer la vérif = abandonner l'objectif « peers non-confiants ».
- **LLM-judge / match sémantique** : REJETÉ — non déterministe (un judge LLM ré-introduit
  le problème), surface d'attaque (deux réponses « proches » dont une malveillante).
- **Incentive par stake monétaire (VeriLLM reward on-chain)** : REJETÉ — viole la décision
  gelée kudos non-monétaire (PO-12, `vision_model`) ; on prend la mécanique VRF + spot-check
  de VeriLLM mais l'incentive est réputationnel, pas économique.

### D5 — Mode groupe privé explicite : `ComputeGroup` allowlist Ed25519, zéro worker anonyme (livrable net-new, pas posture acquise)

**Sources consultées** :
- addendum SOTA §1 (figé, invariant non-négociable) : groupe privé explicite, allowlist
  Ed25519, R-iroh-audit P0 → pilote fermé.
- addendum SOTA §4 (caveat) : pas de TEE GPU consumer 2026 → aucun claim de
  confidentialité face aux workers (SI-1/SI-4).
- `remote_user_sharded_llm_rnd.md` §10 : `ComputeGroup` wire primitive.
- Code local : `invite.rs` (M19 paire-liée Ed25519+JCS), `canonical.rs:214`
  `DOMAIN_SEED_REQUEST_V1` pattern, `node_directory.rs` (allowlist signée),
  `runtime.rs:1046` claim-gate advisory (pas de notion de groupe).

**Retenu (D5 confirmé tel quel par le PO ; finding G1 D5 inchangé)** : le mode groupe
privé est un **livrable net-new**, pas une posture déjà tenue — rien dans le code actuel
ne contraint un worker shard à appartenir à une allowlist. S77 livre une primitive
`ComputeGroup` : une **allowlist Ed25519 des `worker_pubkey` autorisés**, signée par
l'initiateur de session (`DOMAIN_COMPUTE_GROUP_V1` sur le pattern M19). Un worker
non-allowlisté qui tente d'ouvrir une connexion `sbfb/shard/1` est **rejeté au handshake
ALPN** (avant tout calcul d'activation). **Verrou anti-recentralisation** : l'initiateur
signe l'allowlist mais n'est PAS une autorité réseau — c'est un groupe ad-hoc privé entre
pairs qui se connaissent (modèle pilote fermé), pas un registre central. **Caveat
confidentialité écrit** : les workers voient les activations en clair (SI-1/SI-4, pas de
TEE) → aucun secret app dans les prompts ; à inscrire THREAT_MODEL §16.

**Implication code** : `crates/nexus-core-rs/src/compute_group.rs` (net-new primitive
Ed25519+JCS), `canonical.rs` (`DOMAIN_COMPUTE_GROUP_V1` additif), `node.rs` (le handler
`ShardProtocol` vérifie l'appartenance allowlist au handshake), réutilise `invite.rs`
crypto.

**Rejeté** :
- **Mode public par défaut** : REJETÉ — viole R-iroh-audit P0 (audit iroh non fait → pas
  de surface publique) ; le sharding en tension directe avec « peers anonymes ».
- **PoW seul comme admission** : REJETÉ — le PoW limite le coût Sybil mais n'empêche pas
  la collusion worker+vérifieur d'un groupe ouvert ; l'allowlist explicite est le verrou
  anti-collusion pour le pilote fermé.
- **TEE GPU pour confidentialité** : REJETÉ — inexistant consumer 2026 (addendum §4) ;
  aucun claim de confidentialité possible, on l'assume (groupe privé = limite physique,
  pas gap iroh).
- **Réutiliser M19 tel quel comme admission groupe** : REJETÉ partiellement — M19 lie
  `(project_id, archive_hash)` (admission seed), pas l'appartenance à un groupe compute ;
  on réutilise la **crypto** (Ed25519+JCS+DOMAIN pattern) mais le `ComputeGroup` est un
  type distinct.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ⚠️. Rigor signal G4 satisfait (4 ⚠️ sur 5 —
au-dessus du gold 1-2, cohérent avec un sprint phare R&D où la frontière de scope est le
risque #1). Détail complet : `sprint77_design_review.md`.

**Arbitrage PO override (Checkpoint §11, 2026-06-20)** : le board avait recommandé le
**scope minimal** pour D3 (scheduler placement minimal + spike toy) et D4 (N0+N2 seulement),
PRÉCISÉMENT parce que « le scheduler complet est un sprint à lui seul » (G1 D3) et que la
pile de vérif complète est lourde (G1 D4). **Le PO a tranché le scope MAXIMAL** (directive
« sprints ultra-complets, 0 defer du cœur ») : D3 = 70B complet + Parallax 2-phases, D4 =
N0+N1+N2+N3 + incentive. **Les ⚠️ D3/D4 ne sont donc PAS levés — ils deviennent des risques
de TAILLE de sprint assumés par le PO** (cf. §9 R4 reformulé + R8 nouveau), pas des défauts
de décision. Le board confirme que la faisabilité TECHNIQUE est établie (design figé
addendum) ; le risque résiduel est l'AMPLEUR (≥11 phases) et l'incentive-vérif non-garanti
économiquement (R8).

- **D1 ⚠️** : convergence WAN non root-causée. Décision : adjust — reformulé en
  « diagnostic-puis-fix » (test rouge-d'abord + fork de décision explicite si BLOCK iroh).
  Appliqué §4 D1. **Accepté tel quel par le PO** (fork iroh inclus).
- **D3 ⚠️** : le board recommandait minimal. Décision : **PO override scope maximal** —
  scheduler Parallax complet + benchmark 70B = livrables (scope cuts #1/#3 retirés). Le ⚠️
  devient le risque R4 (taille de sprint) assumé. Appliqué §4 D3.
- **D4 ⚠️** : le board recommandait N0+N2. Décision : **PO override scope maximal** —
  N0+N1+N2+N3 + incentive curator-reputation = livrables (scope cut #2 retiré ; N4 reste).
  Le ⚠️ devient le risque R8 (incentive non-garanti économiquement). Le backend
  `llm_llama_cpp` non-CI reste un risque (R2). Appliqué §4 D4.
- **D5 ⚠️** : `ComputeGroup` livrable net-new (allowlist Ed25519, rejet handshake). Décision :
  adjust — inscrit comme livrable. **Confirmé tel quel par le PO.** Appliqué §4 D5.
- **D2 ✅** : aucun finding. **Confirmé tel quel par le PO.** À porter au plan : vérifier
  `cargo doc` `TransportConfig` iroh 0.98 avant de promettre du tuning ; `conn.stats()` RTT
  à valider sur liens résidentiels (alimente la perf-map D3).

---

## §5 Plan Phase outline A..K

> Phases ILLIMITÉES (regex `Phase [A-Z]+[0-9]?`, README §4). Le scope maximal PO impose
> ~11 phases (le board G1 avait averti « le scheduler complet est un sprint à lui seul » ;
> le PO l'assume). `Phase 0` = audit gate S76 DÉJÀ JOUÉ et CLOS. Détail exécutable :
> `sprint77_plan.md`.

### Phase A — Convergence delivery WAN (prérequis dur, diagnostic-puis-fix) — D1
Test 2-nœuds rouge-d'abord (entrée `task:` incrémentale post-subscribe). Diagnostic
cause-racine. Fix câblage SBFB (download policy / re-subscribe / gossip join doc) OU
BLOCK PO si iroh-intrinsèque. Commit `feat(daemon): Sprint 77 Phase A — WAN task delivery
convergence`. Critère : test convergence vert.

### Phase B — Data plane ALPN `sbfb/shard/1` + ComputeGroup admission — D2/D5
ALPN `ShardProtocol` enregistré via `extra_protocols` (miroir SEED_ALPN). `open_bi`
long-vécu + framing longueur-préfixe + `conn.stats()` exposé. `ComputeGroup` allowlist
Ed25519 + rejet handshake. Absorbe carry P3-D-3 (result-sync send-failure si touché).
Commit `feat(core): Sprint 77 Phase B — shard data plane ALPN + private compute group`.
Critère : 2 nœuds échangent un frame d'activation sur `sbfb/shard/1`, worker non-allowlisté
rejeté.

### Phase C — Primitives wire shard — D3
`ShardPlan`/`ShardAssignment`/`ShardedSessionManifest` (`remote_user_rnd.md` §10),
`DOMAIN_SHARD_PLAN_V1`/`DOMAIN_RUN_PROOF_V1`. Commit `feat(core): Sprint 77 Phase C —
shard wire primitives + run proof`. Critère : canonical-bytes round-trip stable.

### Phase D — Scheduler placement (water-filling VRAM + k-medoids RTT) — D3
Placement DP water-filling `GpuInfo.vram_free_bytes` mesuré + clustering k-medoids sur
matrice RTT pairwise (`conn.stats()`) + seuil sharding. Absorbe SYBIL-SEEDER-TAIL (sampling
du dial-set). Commit `feat(core): Sprint 77 Phase D — Parallax placement scheduler
(water-filling + k-medoids)`. Critère : plan placement valide pour 3-5 shards homogène VRAM.

### Phase E — Scheduler routing DAG + churn re-balancing actif — D3
Routing DAG layer-indexé (sweep DP G→D relaxation Parallax) + churn `replace_failed_server`
O(t) + heap fallback + cache activations client-side + perf-map (rho, tau) republiée 1-2s
iroh-docs. Commit `feat(core): Sprint 77 Phase E — DAG routing + active churn rebalancing`.
Critère : routing recalculé sur worker-drop, perf-map propagée.

### Phase F — Backend shard `llm_llama_cpp` (bloc de couches, 70B layer-subset) — D3/D4
Exécution d'un sous-ensemble de couches Transformer in-process via `llama-cpp-2` (charge
layer_start..layer_end, forward partiel, hidden state de sortie + extraction top-k pour
TOPLOC). Worker shard claim une `ShardAssignment`. Commit `feat(worker): Sprint 77 Phase F
— sharded layer-block execution backend`. Critère : bloc de couches sur GGUF local +
primitive hermétique.

### Phase G — Vérification N0 TOPLOC fingerprint — D4
TOPLOC (LSH top-k hidden state) remplace L3 inerte dans `verification.rs` ; `logprobs_hash`
devient le slot réel. Commit `feat(core): Sprint 77 Phase G — N0 TOPLOC fingerprint`.
Critère : primitive TOPLOC hermétique en CI détecte un swap modèle.

### Phase H — Vérification N1 spot-check VRF + incentive curator-reputation — D4
Vérifieur tiré par VRF Ed25519 (prefill-only VeriLLM/DiFR, randomise temp+seed), étend
`rerun.rs`. Incentive curator-reputation (kudos réputationnel non-monétaire pour spot-check
honnête) + note honnête (mitigation réputationnelle, pas garantie économique) + mapping
criticité→niveau. Commit `feat(core): Sprint 77 Phase H — N1 VRF spot-check + reputation
incentive`. Critère : VRF tire un vérifieur déterministe, incentive câblé, mapping documenté.

### Phase I — Vérification N2 redondance tolérante + N3 bissection opML/SENTINEL — D4
N2 fingerprint tolérant (chemin additif, quorum exact INCHANGÉ), étend `redundancy.rs`.
N3 commit-reveal activations par-frontière (`DOMAIN_ACTIVATION_COMMIT_V1`) + SENTINEL EMA
inter-stages localise le stage corrompu. Commit `feat(core): Sprint 77 Phase I — N2 tolerant
redundancy + N3 opML bisection + SENTINEL`. Critère : N2 accept/reject ; N3 localise un
stage corrompu sur fixture.

### Phase J — Front session shard + UX intentions — D5
Panneau session shard (groupe privé, membres, statut pipeline, niveau de vérif) ; UX
intentions (« Rejoindre un groupe de calcul », « Lancer un gros modèle en réseau »), pas de
jargon. Spec T1 `web/e2e/compute-shard.spec.ts`. Commit `feat(web): Sprint 77 Phase J —
shard session panel + hermetic E2E`. Critère : E2E hermétique vert, scan-en-strings clean.

### Phase K — Benchmark 70B 3→5 machines + acceptance + wrap-up — D1..D5
Bring-up spike 1B/7B (2-3 machines) PUIS benchmark 70B Q4 (3 puis 5 machines), harness
`b3_shard_pipeline.sh` (artefact JSON + RunProof N0-N3 + gate réseau GO/NO-GO + test
worker-drop). `verification.md` fail-fast + `sprint78_audit_plan.md` (Track Testabilité) +
THREAT_MODEL §16 (SI-1..SI-5 sharding + incentive) + PATTERNS + SPRINT_LOG + CLAUDE.md.
Absorbe l'invariant clôture. Commit `feat(daemon): Sprint 77 Phase K — 70B shard benchmark
acceptance + wrap-up`. Critère : fail-fast vert, 70B mesuré (PASS ou PROVISIONAL+carry si
DIFFERE-matériel honnête).

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 77)

**Aucun item à 3/3 MANDATORY à l'entrée.** SYBIL-SEEDER-TAIL est à **2/3** avec exemption
nommée « dépendance interne sharding » ; sans l'exemption il deviendrait 3/3 — l'exemption
est valide car S77 touche le dial-set/topology (le sampling anti-Sybil du tail seeder se
regroupe naturellement avec le scheduler de placement Phase D), donc il est **absorbé en
Phase D**, pas reconduit.

| Item | Reports | Phase S77 | Exit condition |
|---|---|---|---|
| (aucun 3/3 MANDATORY) | — | — | — |

### Carry absorbés S77

| Item | Reports | Phase S77 | Exit condition |
|---|---|---|---|
| SYBIL-SEEDER-TAIL | 2/3 (exemption levée par absorption) | Phase D | Sampling anti-Sybil du tail seeder traité avec le dial-set/topology du scheduler de placement (k-medoids) ; test couvrant le sampling ou doc-note honnête si availability-only |
| P3-D-3 (send-failure un-mark `seen.remove`) | 1/3 | Phase B | Si le pipeline shard ajoute un chemin result-sync, test ciblant la branche récepteur-droppé ; sinon doc-note (chemin couvert par lecture) |
| Invariant clôture noms de tests | nouveau (audit S76) | Phase K | Tout nom de test cité dans `verification.md` §6 grep-résout à une fn `#[test]` (re-check au wrap-up) |

### Carries reconduits S78

| Item | Reports | Justification |
|---|---|---|
| REVISION-HOME-DURABILITY | 2/3 → 3/3 ⚠️ | Pas d'exemption ; mitigé systemd `SBFB_HOME` épinglé. S77 (compute distribué) ne touche pas le mode déploiement home-less. **Atteint 3/3 en S78** — devra être traité au plan S78 (ou exemption « blocker externe » re-justifiée si un mode home-less n'a pas encore émergé). Pas exploitable pre-launch. |
| KNOWN-ENTRY-OVERCOUNT | 2/3 → 3/3 ⚠️ | **Exemption « dépendance séquentielle » renouvelée** : dedup `(pid,hash)` requis SEULEMENT si une UX future affiche « N apps découvrables » — aucun consommateur UI en S77 (le front S77 = session shard, pas browse) ni planifié S78. Superset honnête (curator-list + annuaire) ; pas de bug aujourd'hui. **Atteint 3/3 en S78** : exemption à re-justifier ou traiter selon l'UX browse S78. |
| seeder `catalog_len:0` | 2/3 → 3/3 ⚠️ | Pas d'exemption — **bloqué sur arbitrage PO design** (pas code) : section « seeded » non-autoritaire dans `NodeDirectoryEntry` vs verrou-4 (seeder ≠ éditeur) + modèle F-Droid. **Atteint 3/3 en S78** — devra être tranché PO au kickoff S78 (cf. Attention 3/3). |
| RE-DRIVE-ON-INGEST | 2/3 → 3/3 ⚠️ | Lié à la convergence delivery (D1). **Si D1 résout le live-sync, peut se fermer en cascade en S77** ; sinon reconduit 3/3 — à ré-évaluer au wrap-up Phase K selon le résultat D1. Remède opérateur documenté (restart). |
| T-NN+3 (canonical_bytes dup JCS) | open S70 | Pas d'exemption ; absorbable au prochain sprint touchant JCS crypto. S77 ajoute beaucoup de `DOMAIN_*` (JCS : compute_group, shard_plan, run_proof, activation_commit) — **candidat d'absorption opportuniste Phase C/I** si le code JCS est touché ; sinon reconduit. |

Exemptions valides (avec justification factuelle) :
- **SYBIL-SEEDER-TAIL** : dépendance interne sharding — le sampling se regroupe avec le
  dial-set/topology du scheduler k-medoids (Phase D). **Levée par absorption** (entre dans
  le plan, pas reconduit).
- **Externes inchangés** (< 3 reports, exemptions tenues) : P2-A-1 rand (exemption
  upstream), P2-AUDIT-2 iroh pre-release transitives (pin 0.98), T-NN+2 iframe Rust-wasm
  (PATTERNS §P34), P3-OS-1 `operator_server` OR dupliqué. LT-2 Radicle ARME (flip publié =
  décision PO hors-sprint). LT-5 résorbé.

### P2 audit S76 routés (G6 — à fusionner par le thread principal)

Les 6 P2 de `sprint76_audit_findings.md` §5/§7 (absorbables opportunément par une phase
S77 touchant le fichier) :
- **OWN-DOC-FLOOR-L2L4** (Track B) : décision PO doc-comment vs OR-floor. Hors-zone S77
  (consent worker co-localisé) → reconduit, log-debt.
- **B10-PARITE-FIXTURE** (Track C) : fixture partagée allowlist bridge. **S77 touche le
  bridge ? Non en principe** (le shard est ALPN, pas bridge postMessage) → reconduit.
- **DIRECTORY-EAGER-HAPPY-PATH** (Track C) : résolution paresseuse tier directory. Hors-zone
  S77 → reconduit, log-debt.
- **SANITY-BOUND-ASYMETRIQUE / MEDIAN-DE-GROUPE** (Track F) : durcir anti-gaming. **Lié à
  la vérif S77** — l'incentive curator-reputation (Phase H) et N2 (Phase I) touchent le
  scoring → **candidat d'absorption Phase H/I** (le median-de-groupe est exactement le type
  de durcissement que N2 fingerprint-tolérant + incentive rend naturel). Sinon reconduit.
- **P3-D-3 SEND-FAILURE-UNMARK** (Track E) : absorbé Phase B (cf. supra).

### Attention 3/3 S78

**Signal PO fort** : 4 carries atteignent 3/3 en S78 (sauf fermeture en cascade ou
absorption en S77). Le kickoff S78 devra les traiter comme phases ou re-justifier une
exemption « blocker externe / dépendance séquentielle » (Règle 2 G7).

| Item | Reports → S78 | Signal PO |
|---|---|---|
| seeder `catalog_len:0` | 2/3 → 3/3 | **Devra être résolu (arbitrage PO design) au plan S78**, pas reporté. Question : section « seeded » non-autoritaire vs verrou-4. |
| RE-DRIVE-ON-INGEST | 2/3 → 3/3 | Possiblement fermé en cascade par D1 (S77) ; sinon MANDATORY S78. Re-check au wrap-up Phase K. |
| REVISION-HOME-DURABILITY | 2/3 → 3/3 | MANDATORY S78 sauf exemption « blocker externe » re-justifiée (aucun mode home-less émergé). |
| KNOWN-ENTRY-OVERCOUNT | 2/3 → 3/3 | Exemption « dépendance séquentielle » à re-justifier S78 (pas de consommateur UI browse) ou traiter avec l'UX browse S78. |

### LT items (ROADMAP_COMMITMENTS — Règle 3)

Tous évalués, **aucune condition de déclenchement remplie pour S77** :
- LT-1 (Kudos-v2) : réclassifié pre-v1.0 S50 (fait). Latent.
- LT-2 (Radicle flip) : trigger = tag v1.0 (non posé). ARME, dry-run privé fait. Hors-S77.
- LT-3 (Sybil matrix), LT-4 (biometric gate), LT-5 (redundancy persistence) : latent post-v1.0.
- LT-6 (iroh neighborhood) : resolved S32.
- LT-7 (self-hosted build) : gate satisfait S55/S60.

---

## §7 Scope cuts

Chaque item re-évalué contre le code actuel (Step 3 G9). **Sur arbitrage PO scope maximal,
les ex-scope-cuts #1 (scheduler Parallax complet), #2 (N1 VRF + N3 opML) et #3 (benchmark
70B/5 machines) sont devenus des LIVRABLES** — ils ne figurent plus ici. Ce qui reste hors
S77 :

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | N4 zkML (DeepProve/NANOZK preuve ZK d'inférence) | post-S77 (Gate-4+) | Le PO a demandé N1/N3, PAS N4 ; 803s proving/forward 13B, 50-15000× overhead « prohibitively expensive » (addendum §3 « opt-in Gate-4+ ») |
| 2 | Tensor-parallel mono-machine 2-GPU | ENTERRÉ | Arbitrage PO S76 « personne n'a 2 GPU » + all-reduce LAN/NVLink only (addendum §1) ; scope cut #2 de S76, ne PAS ressusciter |
| 3 | Streaming token-par-token interactif (chat live sur shard) | jamais (NO-GO produit) | Enveloppe perf addendum §6 : 1-3 tok/s WAN résidentiel multi-hop = chat live NON VIABLE ; batch/async uniquement ; ne PAS promettre |
| 4 | Confidentialité face aux workers (chiffrement des activations en calcul) | jamais (limite physique) | Addendum §4 : pas de TEE GPU consumer 2026 ; les workers voient les activations en clair (SI-1/SI-4) ; groupe privé + pas de secret app dans les prompts, pas de claim de confidentialité |
| 5 | KV-cache distribué partagé / activation cache O(t) sur gros contexte de campagne | post-S77 | Addendum §8 Q7 : le cache d'activations explose sur gros contexte ; S77 KV-cache local par shard + cache churn client-side borné, pas de cache distribué persistant |
| 6 | Quantization blockwise dynamique des activations (Petals, halve la BP) | post-S77 | Addendum §5 : marginal en decode (~16 KB/tok/frontière), significatif en prefill ; optimisation BP post-preuve du benchmark 70B |
| 7 | VRAM-live admission runtime (pompe `gpu.snapshot()` réel vs `estimated_*` déclaré) | post-S77 | Carry S76 scope cut #3 ; le scheduler S77 lit `vram_free_bytes` au PLACEMENT (montage session) mais la pompe runtime garde le check sur `estimated_*` déclaré ; le câblage VRAM-live runtime est orthogonal au shard |
| 8 | Mode public / découverte ouverte de groupes de calcul | jamais (R-iroh-audit P0) | Invariant non-négociable addendum §1 : groupe privé explicite, zéro worker anonyme ; pilote fermé tant que R-iroh-audit P0 |
| 9 | Upgrade iroh 0.98 → 1.0 (stable juin 2026) | Gate-1/PO | G2 trigger détecté (iroh 1.0 stable) mais Day-0 gelé ; sauf si D1 révèle que la convergence WAN exige une primitive 1.0 (alors BLOCK PO) |
| 10 | `execute_build` LT-7 câblage (le réseau compile le réseau) | post-S77 | Carry S76 scope cut #7 ; orthogonal au sharding inférence ; LT-7 gate satisfait S55/S60 |
| 11 | Garantie économique de l'incentive-à-vérifier (stake/token) | jamais (décision gelée) | Kudos non-monétaire PO-12 interdit le stake ; l'incentive S77 est réputationnel (curator-reputation), une mitigation pas une garantie — la garantie économique exigerait un token crypto (vision_model NO) |
| 12 | Reconnaissance contributeur publique des shards | post-launch | Orthogonal au shard ; reconnaissance publique = post R-iroh-audit |
| 13 | AWQ/GPTQ/EXL2 (formats quant alternatifs) | rejeté | GGUF Q4 retenu pour le sizing shard (addendum §5) ; les autres formats sont des dép/runtime concurrents |
| 14 | Push live origin/master (lot 6 ahead `4da9800` déjà poussé) | décision PO hors-sprint | Le push est une décision opérateur (LT-2/Radicle hors-sprint) |

---

## §8 Traçabilité scope

Chaque item « What's NOT » de S76 (kickoff §7, 11 scope cuts) mappé sur son traitement S77 :

| Item S76 "What's NOT" | Sprint + Phase S77 |
|---|---|
| #1 Sharding pipeline | **C'EST S77 (tout le sprint, scope maximal)** — Phases A-K |
| #2 Tensor-split mono-machine 2-GPU | Supprimé (ENTERRÉ, scope cut #2 S77) |
| #3 VRAM-live admission runtime | Reconduit post-S77 (scope cut #7 S77) — partiellement touché (scheduler lit `vram_free_bytes` au placement Phase D) |
| #4 Median-de-groupe anti-gaming | **Candidat d'absorption Phase H/I** (l'incentive + N2 touchent le scoring) ; sinon reconduit (carry MEDIAN-DE-GROUPE) |
| #5 TOPLOC étage 2 | **Livré S77 Phase G** — N0 TOPLOC fingerprint (le slot `logprobs_hash` devient réel) |
| #6 Quorum cross-GPU hétérogène | **Livré S77 Phase I** — N2 fingerprint tolérant (complet, pas partiel ; le PO a tranché la vérif complète) |
| #7 `execute_build` LT-7 | Reconduit post-S77 (scope cut #10 S77) |
| #8 Reconnaissance contributeur publique | Reconduit post-launch (scope cut #12 S77) |
| #9 Self-test enrôlement | Rejeté (inchangé) |
| #10 Scheduler idle BOINC | Reconduit post-launch (orthogonal) |
| #11 AWQ/GPTQ/EXL2 | Rejeté (scope cut #13 S77 ; GGUF Q4 retenu) |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Convergence delivery WAN (D1) s'avère un comportement intrinsèque iroh-docs 0.98 non fixable sans upgrade 1.0 → tout le sharding BLOQUÉ | Medium | High | Phase A diagnostic rouge-d'abord + fork de décision explicite ; si BLOCK iroh, arbitrage PO (upgrade 1.0 hors-Day-0 OU re-architecture delivery) ; le reste du sprint (data plane, primitives, scheduler, vérif) avance en parallèle sur l'infra hermétique |
| R2 | Backend `llm_llama_cpp` (jamais en CI, `#[ignore]`-gated) ne build pas / n'extrait pas les hidden states comme attendu → N0/N1 TOPLOC/DiFR infaisables | Medium | High | Double test (primitive hermétique en CI sur fixtures + intégration GGUF `#[ignore]` locale) ; valider le build `--features llm_llama_cpp` tôt (Phase F) ; si extraction logits impossible sans fork, dégrader à N2 redondance + doc-note honnête |
| R3 | Performance réelle du benchmark 70B < 2 tok/s même sur 3-5 machines proches → NO-GO produit | Medium | Medium | Gate réseau explicite (RTT/frontière ≤ 20-30ms, NO-GO RTT>80ms) ; verdict produit écrit « 1-3 tok/s batch/async jamais chat live » ; bring-up séquencé (toy 1B/7B → 70B/3 → 70B/5) ; PROVISIONAL+carry si DIFFERE-matériel |
| R4 | **Sprint TRÈS LARGE assumé PO (≥11 phases, scope ultra-complet)** → risque de déborder le cycle / qualité par phase | High | Medium | Scope maximal assumé PO (override G1) ; budget de phases OUVERT (README §4) ; phases découplées (A indépendante de B-I hermétiques) ; chaque phase = 1 commit atomique avec gate dual-platform ; si une phase déborde, elle se sous-découpe (L, M…) plutôt que de bâcler |
| R5 | `ComputeGroup` + 4 `DOMAIN_*` net-new (D5/D3/D4) ouvrent une large surface wire/crypto Ed25519+JCS mal vérifiée | Medium | High | Réutilise la crypto M19 éprouvée (pattern `DOMAIN_SEED_REQUEST_V1`) ; rejet au handshake ALPN AVANT calcul ; tests signature/verify + canonical-bytes round-trip par primitive ; Codex gate sur chaque nouvelle primitive ; T-NN+3 absorbé (factoriser le JCS dup) |
| R6 | iroh `TransportConfig` 0.98 ne permet pas le tuning streams persistants promis (D2) | Low | Medium | Vérifier `cargo doc` `TransportConfig` AVANT de promettre du tuning (addendum §8 Q9) ; l'`open_bi` long-vécu de base est confirmé context7 ; le tuning fin est optionnel ; `conn.stats()` RTT à valider sur liens résidentiels |
| R7 | Acceptance LIVE benchmark 70B DIFFERE-matériel (besoin 3-5 machines + GPU 16+ GB chacune) comme S76 | High | Low | Harness `b3_shard_pipeline.sh` runnable (artefact JSON `PASS`/`BLOCK`/`RIG-ABSENT`) ; PROVISIONAL+carry P1 honnête si DIFFERE ; le test convergence Rust + les primitives TOPLOC/N1/N2/N3 hermétiques + le spike toy couvrent en CI ; précédent S76 accepté |
| R8 | **Incentive-à-vérifier (D4) non-garanti économiquement** : un lazy verifier rationnel ne vérifie pas ; le kudos réputationnel ne le force pas | Medium | Medium | Mitigation réputationnelle (kudos curator-reputation VRF) + note honnête écrite (pas une garantie économique) ; le pilote fermé (D5 groupe privé) + l'anti-Sybil amont (PoW/AgeWitness/invite M19) bornent le risque ; documenté THREAT_MODEL §16 sev M ; N4 zkML (garantie cryptographique) reste post-S77 |

---

## §10 Audit gate pattern — rappel

**Phase 0 a été jouée** : audit gate S76 CONDITIONAL PASS, condition levée par `52e70d1`.

La **Phase K (wrap-up)** du sprint devra produire :
- `sprint77_verification.md` (self-report fail-fast) AVEC une section `## Acceptance`
  portant les verdicts du gate de testabilité (README §4) : **T1** `GREEN`/`RED`/
  `N-A-no-frontend-change` (spec `web/e2e/compute-shard.spec.ts`) et **T2** `PASS`/
  `BLOCK{diagnosis}`/`RIG-ABSENT`/`N-A-no-cross-machine-feature` (vocabulaire fermé, JAMAIS
  un `DIFFERE-materiel` en prose ; l'artefact JSON de `b3_shard_pipeline.sh` porte le
  verdict, étendu 70B + RunProof N0-N3).
- `sprint78_audit_plan.md` (plan pour Phase 0 S78) qui DOIT inscrire une **track
  Testabilité standing** (miroir Track J) exigeant que l'audit gate S78 vérifie la
  création/statut CI de la spec T1 `web/e2e/compute-shard.spec.ts` + l'artefact JSON T2.
  La phase wrap-up ÉCRIT cette track ; ce kickoff en SPÉCIFIE l'exigence ici.
- Mise à jour `docs/security/THREAT_MODEL.md` **§16 (nouvelle section sharding** : SI-1
  reconstruction activations / SI-3 fingerprinting / SI-4 collusion / SI-5 latence
  side-channel, mapping sur le mode groupe privé + caveat confidentialité + incentive-vérif
  réputationnel sev M).
- Mise à jour `docs/rust/PATTERNS.md` (ALPN custom shard, TOPLOC/N1/N2/N3 primitives,
  scheduler Parallax, perf-map) et `docs/shell/PATTERNS.md` si nouveaux patterns ou tech debt.
- Re-check de l'invariant clôture audit S76 (tout nom de test cité dans `verification.md`
  §6 grep-résout à une fn `#[test]`).

---

## §11 Checkpoint de validation

> **Arbitrage PO DÉJÀ RENDU (2026-06-20)** : D1 accepté (fork iroh), D2/D5 confirmés,
> **D3/D4 pivotés vers le scope MAXIMAL**. Ce Checkpoint est conservé pour traçabilité et
> re-confirmation finale avant l'attaque du plan détaillé.

1. **D1 (convergence WAN)** — Phase A ouvre par un **diagnostic rouge-d'abord** avec un
   **fork explicite** : si la cause est intrinsèque à iroh-docs 0.98, ça devient un BLOCK
   à arbitrer (upgrade 1.0 hors-Day-0). **PO : ACCEPTÉ tel quel.**

2. **D2 (data plane)** — ALPN custom `sbfb/shard/1` sur iroh-QUIC, PAS le RPC llama.cpp
   (TCP/blind-trust/LAN). **PO : CONFIRMÉ tel quel.**

3. **D3 (scope maximal)** — S77 vise le **70B COMPLET** : scheduler Parallax 2-phases
   complet (placement water-filling + k-medoids RTT + routing DAG + churn actif) + benchmark
   70B sur 3-5 machines = livrables. Le spike toy 1B/7B est une étape de bring-up. **PO :
   SCOPE MAXIMAL tranché** (override du board G1 qui recommandait minimal). Re-confirmer que
   l'ampleur (≥11 phases, sprint très large) est assumée.

4. **D4 (vérif complète)** — La vérif des shards = **N0+N1+N2+N3** + incentive
   curator-reputation (kudos réputationnel non-monétaire, mitigation pas garantie). N4 zkML
   reste post-S77. Le mode vérifié impose `llm_llama_cpp` (Ollama dégradé). **PO : SCOPE
   MAXIMAL tranché.** Re-confirmer l'acceptation de l'incentive non-garanti économiquement
   (R8) et du backend `llm_llama_cpp` non-CI (R2).

5. **D5 (groupe privé)** — `ComputeGroup` allowlist Ed25519, rejet au handshake ALPN, zéro
   worker anonyme, pilote fermé. Aucun claim de confidentialité face aux workers (pas de TEE
   GPU 2026). **PO : CONFIRMÉ tel quel.**
