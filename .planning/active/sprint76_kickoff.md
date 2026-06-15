# Sprint 76 — Kickoff : GPU partagé volontaire, prouvé cross-machine

> Arc 3.5 Factory Complete Vision **6/6** (clôture de l'arc). La roadmap v5
> amendée par S75 décale le GPU partagé de S75 → **S76**, et le sharding pipeline
> → S77. S76 réutilise le moteur consent GPU 4-niveaux + caps + GPU monitor +
> wire Task/Claim/Result Ed25519+JCS + pompe worker + validator/quorum DB déjà
> matures, et les **expose/câble** cross-machine. Le vrai trou levé est **B-3** :
> aucune preuve cross-machine du chemin compute n'existe aujourd'hui.

**Écrit** : 2026-06-15 (post-audit gate S75 **CONDITIONAL PASS** `73831c0`,
P1 `DURESS-BOOT-LEAK` fixé `23a08c9` — condition de blocage Phase A **LEVÉE**).
**Type** : **sprint PAIR** — une **phase dette est réservée (Phase B), NON
convertible en feature** (Règle 1 G7). Aucun item 3/3 MANDATORY à l'entrée ;
mais **4 carries sont à 2 reports** (3 traités en Phase B pour casser l'escalade).
**Tip master d'entrée** : `23a08c9` (audit findings S75 : **0 P0, 1 P1 FIXÉ
`23a08c9`, 14 P2, 6 P3**).
**Phase 0 audit Sprint 75** : **DÉJÀ JOUÉ** — `73831c0` CONDITIONAL PASS, P1
levé par `23a08c9` (HEAD).
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV (OPEN).
**Roadmap source** : `.planning/roadmap_v5_factory_complete_vision.md`. Sprint
6/6 de l'Arc 3.5 (Factory Complete Vision), dernier sprint de l'arc.

---

## Sources context7 + WebSearch consultées (pre-gel)

Recherche factuelle (G9 + G2 + G10) AVANT gel des D1-D5. Toutes accédées
**2026-06-15** sauf indication contraire. Le code SBFB ancré est listé dans
chaque bloc §4 « Implications code ».

### D1 — Surface « offrir ma puissance » (consent + caps + monitor)
- **BOINC** `global_prefs` (client v8) — WebFetch 2026-06-15,
  `github.com/BOINC/boinc/wiki/Preferences` : modèle granulaire `Use at most N%
  CPU`, `Suspend GPU when computer in use`, `Compute only between HH:MM`,
  day-of-week ; persistance multi-niveau (manager local → website → account
  manager). Cap déclaratif auto-appliqué client-side.
- **Folding@home** power-slider — WebSearch 2026-06-15,
  `foldingathome.org/faqs/.../folding-power-slider` + `FoldingAtHome/fah-control`
  CHANGELOG (FAH v7) : 3 crans Light/Medium/Full + toggle `Only when idle` ;
  forum t=26083 (réglage idle-vs-working séparé jamais livré = signal non-MVP).
- **Petals** v2.2.0 `--public_name` — WebSearch 2026-06-15,
  `bigscience-workshop/petals` README + `health.petals.dev` : enrôlement = un
  process lancé, attribution = paramètre explicite opt-in, visibilité = dashboard
  agrégé (≥10 blocs servis). Release tag v2.0.0.post1 vérifié.
- **Salad / vast.ai** — WebSearch 2026-06-15, `docs.salad.com/.../faqs` +
  `docs.vast.ai/host/how-to-self-test` : opt-in « share GPU when not in use » +
  earnings dashboard ; vast.ai impose un self-test avant « verified » (rejeté).

### D2 — E2E cross-machine task-routing compute (lève B-3)
- **iroh-docs / iroh-blobs** (transport réel SBFB) — `docs.rs/iroh-docs` latest +
  `github.com/n0-computer/iroh-docs`, lu 2026-06-15 : « each entry's value is the
  32-byte BLAKE3 hash … the content itself is not stored or transferred through a
  replica » (les entries `task:`/`result:` portent un hash, le payload voyage via
  iroh-blobs) ; « Docs is a meta protocol that relies on iroh-blobs and
  iroh-gossip ». iroh 0.98.0 (2026-04-17), 0.98.2 latest, **1.0.0-rc.1**
  (2026-05-27, RC avant 1.0 — upgrade = décision Gate-1/PO, PAS S76).
- **BOINC** JobReplication — WebSearch 2026-06-15,
  `github.com/BOINC/boinc/wiki/CreditNew` : `min_quorum`/`target_nresults`,
  consensus canonical (modèle redondance retenu conceptuellement).
- **Petals** v2.0.0 (2023-08, arxiv 2209.01188), **Parallax** (arxiv 2509.26182,
  v0.0.1 2025-09-29), **Ray Serve** 2.55.1 (`docs.ray.io`), **GPUStack/llama-box**
  0.6 (`docs.gpustack.ai`) — tous lus 2026-06-15, transports rejetés (§4 D2).

### D3 — Quorum redundancy>1 sur sorties DÉTERMINISTES
- **TOPLOC** (Prime Intellect) — WebFetch blog + **arXiv:2501.16007** (2025-01) :
  hash top-k du last hidden state, comparaison exponent/mantissa séparés,
  « works reliably across non-deterministic GPU hardware », 100% détection ;
  prod dans INTELLECT-2 (arXiv:2505.07291, 2025-05).
- **Thinking Machines Lab**, *Defeating Nondeterminism in LLM Inference* —
  WebFetch blog 2025-09 : 80 complétions uniques/1000 @ temp=0 par défaut →
  1000/1000 bit-identiques après kernels batch-invariants, **scope = même
  hardware**, aucune garantie cross-GPU.
- **LMSYS/SGLang**, *Towards Deterministic Inference* — WebFetch 2025-09-22
  (SGLang ≥0.5.3) : déterminisme intra-serveur, +34% latence, TP1/TP2 only.
- **Ingonyama**, *Reproducibility in DL/LLMs* — WebFetch 2024-09-22 : « unable to
  achieve reproducibility across different machines » avec llama.cpp/GGUF stock ;
  réécriture kernels GEMM CUDA → prefill effondré 302→43 tok/s.
- **zkLLM** arXiv:2404.16109 (CCS 2024) : 803s proving + 986s commit / forward
  pass LLaMA-2-13B (rejeté). **Parallax** vérification « coming next » (non livré).
- Code OSS `ollama/ollama#10751` + `llama-cpp-python#972` (WebSearch 2026-06) :
  temp=0 seed-fixe → sorties divergentes Ollama-vs-llama.cpp même GGUF.

### D4 — Dashboard contributeur (kudos non-monétaire per-task)
- **BOINC** CreditNew/CreditOptions — WebFetch 2026-06-15,
  `github.com/BOINC/boinc/wiki/CreditNew` : « granted credit = claimed credit of
  canonical instance … discard high/low, average the rest » ; host normalization
  + scale probation anti-cherry-picking ; sanity check `wu.fpops_bound`.
- **Folding@home** QRB — WebSearch 2026-06-15, `foldingathome.org/faqs/points` :
  `base * max(1, sqrt(k·deadline/elapsed))`, k=0.75, conditionné passkey + ≥10 WU
  + ≥80% taux retour (anti-Sybil retenu comme garde-fou d'affichage, formule
  rejetée).
- **Gridcoin** RAC→magnitude→GRC — WebSearch 2026-06-15, `gridcoin.us/wiki/
  magnitude` : convertit RAC en jeton monétaire (REJETÉ = viole décision gelée).
- **EigenTrust** — WebFetch 2026-06-15, `nlp.stanford.edu/pubs/eigentrust.pdf`
  (Kamvar/Schlosser 2003) : trust global power-iteration, faiblesse Sybil
  explicite (REJETÉ = ranking global + Sybil + mauvais modèle).

### D5 — Quantization 4-bit documentée
- **bartowski/Llama-3.3-70B-Instruct-GGUF** — HuggingFace, fetched 2026-06-15 :
  Q4_K_M 42.52 GB, Q4_K_S 40.35, IQ4_XS 37.90, Q3_K_M 34.27, Q2_K 26.38.
- **« Which Quantization Should I Use? »** — **arXiv 2601.14277v1** (2026-01-11,
  <90j) : F16 ppl 7.32, Q4_K_M 7.56, Q4_K_S 7.62, Q4_0 7.74 ; Q4_K_S meilleur
  tradeoff 4-bit, Q4_K_M défaut défendable (diff dans le bruit).
- **bartowski/Qwen2.5-14B-Instruct-GGUF** — fetched 2026-06-15 : 14B Q4_K_M
  ~8.5 GB → tient 1×16GB ; 32B Q4_K_M ~22 GB → NE tient PAS sur 16GB.
- **llama-cpp-2** `LlamaModelParams` — `docs.rs/llama-cpp-2` latest, fetched
  2026-06-15 : `with_n_gpu_layers`, `with_split_mode`, `with_devices` (≤16),
  `fit_params` (0.1.146, 2026-04-30). Pin repo `0.1.143`, latest 0.1.147 —
  bump optionnel (hygiène + `fit_params` multi-carte, non bloquant).
- AWQ/GPTQ/EXL2/bitsandbytes — `oobabooga.github.io/blog/posts/gptq-awq-exl2-
  llamacpp`, spheron 2026, ahmadosman.com (tous lus 2026-06-15, tous rejetés =
  GPU-only Python hors-stack Rust in-process).

### G2 Triggers + fraîcheur deps (scan thème GPU/compute)
- 5 fichiers `triggers_revalidate` scannés (`HARDENING_ROADMAP` last_validated
  2026-06-03, `GUARDRAILS` 2026-04-20, `CAPABILITY_TOGGLES` 2026-04-20, `LOOPBACK`
  2026-06-03, `WARRANT_CANARY` 2026-04-18) : **0 revalidation due** (NVIDIA CCM
  H100 = TEE hors-scope RTX 5080 ; wasmtime non utilisé).
- **ollama-rs 0.3.4** (2026-02-12, latest, 0 CVE) ; **nvml-wrapper 0.12.1**
  (0 CVE) ; **iroh 0.98** pin gelé ; **llama-cpp-2 0.1.143→0.1.146/147**
  (bump optionnel).
- **CVE-2026-34159** (llama.cpp RPC RCE, 9.8, patch b8492, 2026-04-02) — **NON
  APPLICABLE** : SBFB utilise llama.cpp in-process (`llama_cpp.rs`), pas
  `rpc-server`. **CVE-2026-2069** (GBNF overflow, 4.8 local, patch #18993) —
  **NON APPLICABLE** : sampling via llguidance Rust-side, pas GBNF natif.
  Sources : `sentinelone.com/vulnerability-database/cve-2026-34159` +
  `cve-2026-2069`, vérifiées 2026-06-15.
- Quantization 4-bit / 70B / 16GB — `sitepoint.com/vram-requirements-70b-models-
  16gb-gpu-2026`, `blog.easecloud.io/ai-cloud/run-70b-models-on-consumer-gpus`,
  `github.com/ggml-org/llama.cpp/blob/master/docs/multi-gpu.md`, vérifiés
  2026-06-15.

---

## §1 Constat d'entrée

### §1.1 D'où on part

S75 a livré le **pivot découverte PULL node-centrique + ancre VPS** (Arc 3.5
5/6), clos par `8b53c38` puis durci par un mini-cycle hors-sprint UX-ARRIVAL
(`e980d7e`) et 3 hotfixes Cas D. L'audit gate S75 (`73831c0`) a rendu un
**CONDITIONAL PASS** : le cœur PULL est sain (0 P0, 0 bump wire, 0 delta dep,
lock-3 tripwire vierge, verrou-4 marquage éditeur-seul, acceptance
survives-VPS-death LIVE tracée) mais a confirmé un **P1 `DURESS-BOOT-LEAK`** —
deux chemins de boot (`reannounce_seeds_at_boot` S74 + republication feed S66)
émettaient des données du vrai data root signées sous la clé leurre. Ce P1 est
**fixé** par `23a08c9` (HEAD), tests duress 11/11 + clippy clean. **La condition
de blocage de S76 Phase A est donc LEVÉE.**

S76 ouvre le **dernier sprint de l'Arc 3.5** : le **GPU partagé volontaire,
prouvé cross-machine**. Le constat factuel (G9, code lu) est que la mécanique
compute est **déjà mûre et largement câblée** — ce qui manque n'est pas du moteur
mais de l'**exposition** (panneau front « offrir ma puissance ») et de la
**preuve cross-machine** (B-3 : aucune exécution compute sur deux hôtes physiques
n'a jamais été démontrée ; le commentaire `dispatch_loop.rs:166` dit explicitement
« Cross-machine/cross-node sync is S75 »). Le moteur consent GPU 4-niveaux + caps
W/VRAM/h + UsageTracker (`consent.rs:84-435`, 25 tests), la pompe worker pull
(`engine/runtime.rs:847-919`), le validator quorum exact-match
(`validator.rs:202-338`), le ledger kudos per-task EMA (`kudos_ledger.rs:51-163`),
et le bridge result-sync (`result_sync.rs:142`) **existent tous**. S76 est un
sprint d'**exposition + wiring + preuve LIVE**, pas de construction de primitive.

### §1.2 Ancrage roadmap v2.1 — Arc 3.5 Factory Complete Vision 6/6

Roadmap v5 (CANON, `roadmap_v5_factory_complete_vision.md`), amendée par S75 :
l'arc 6 sprints S71-S76 se referme ici. S71 (assainir compute+sécurité), S72
(provider routing), S73 (recherche réseau), S74 (atelier fork), S75 (découverte
PULL — pulled-forward), **S76 (GPU partagé cross-machine — ce sprint)**. Le
sharding pipeline (phare) est **S77**, sprint distinct (directive PO
`feedback_ultra_complete_sprints` : S77 sharding = feature distincte, PAS un
defer du GPU S76). Dépendances aval : S77 sharding réutilisera le dial-set
multi-provider et la cohorte homogène posés ici ; le tensor-split mono-machine
multi-GPU (D5) est explicitement S77.

### §1.3 Compteurs tests entrée (tip `23a08c9`)

Documentés (non re-runnés au kickoff — la fail-fast Win+Docker sera re-jouée
comme gate AVANT push). Le P1 fix `23a08c9` ajoute ~+2 tests duress (gate 2
chemins) au-dessus de la baseline UX-ARRIVAL.

| Suite | Count | Source |
|---|---|---|
| Rust nextest (Windows natif) | ~1761 (+2 duress) | MEMORY tip + verification.md §5 |
| Rust nextest (Docker Linux canonique) | ~1765 (+2 duress) | MEMORY tip + verification.md §5 |
| Vitest `web/` | 379 | MEMORY tip |
| Vitest factory-operator | 7 | verification.md §5 |
| size-limit | 6/6 | verification.md §5 |
| coverage web | 87.17/79.01/85.92/88.5 (≥ 85/85/78/85) | verification.md §5 |
| **Total** | **~2154** | (Rust + Vitest + factory + size) |

### §1.4 Pre-launch protocol policy (rappel)

Le réseau n'a aucun déploiement live tiers. Conséquences cardinales pour S76 :
- **`*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent à 1.** Aucun champ
  ajouté à `Task`/`ResultPayload`/capability advert ne bump une version : ce sont
  des additions sur la v1 courante. Si on touche le canonical, on **redéfinit la
  v1**, on ne la bump pas.
- **Feed extensible via raw-op** : aucun nouvel op compute ne bump
  `FEED_FORMAT_VERSION`.
- **`#[serde(default)]` reste légitime** pour la robustesse runtime (un client
  qui envoie un `Task` minimal → champs omis à zéro/false plutôt que 422). Le
  rationale est écrit dans le doc-comment du champ (pas « legacy compat »).
- **Pas de tolerant decoder multi-version, pas de test « legacy decode ».**
- **`ResultPayload.model_digest`/`logprobs_hash` existent déjà** (`task.rs:374,
  383`) dans le canonical signé v1 — S76 les **utilise/corrige**, ne les ajoute
  pas (cf. D3, finding G1).

---

## §2 Goal

S76 transforme le compute SBFB de « mûr mais non prouvé cross-machine » en
**contribution GPU volontaire démontrée de bout en bout sur des hôtes physiques
distincts**. Il (1) expose une surface front « offrir ma puissance » réutilisant
le consent 4-niveaux + caps + GPU monitor existants ; (2) **lève B-3** par une
acceptance LIVE scriptée (PC↔VPS, puis +Mac) du chemin task→claim→execute→result
signé cross-process/cross-réseau ; (3) prouve le **quorum redundancy>1 sur
sorties déterministes** via une cohorte de workers homogènes (exact-match
inchangé) ; (4) livre un **dashboard contributeur** (kudos non-monétaire per-task,
zéro token crypto — décision gelée) ; (5) **documente la quantization 4-bit**
(GGUF Q4_K_M/IQ4_XS par taille de carte, cible single-card honnête ≤14B, 70B =
S77). La Phase B (dette réservée, non convertible) ferme le lot duress restant,
les carries à 2 reports (anti-escalade) et les trous de couverture test/doc.

**Critère SMART : toutes les rows fail-fast vertes au verification.md**, plus
l'acceptance LIVE cross-machine B-3 démontrée (PC/VPS/Mac via SSH, trace dans
verification.md). Mesure binaire au Phase G wrap-up.

---

## §3 Phase 0 — Audit gate Sprint 75 (DONE = CONDITIONAL PASS)

Joué cette session (Cas A, workflow 18-agents anti-anchoring + skeptics +
synthèse). **CONDITIONAL PASS** (`73831c0`) : **0 P0, 1 P1, 14 P2, 6 P3**. Le
cœur du pivot PULL est SAIN (machinerie crypto/wire correcte, 0 bump wire, 0
delta dépendance, lock-3 tripwire vierge, verrou-4 éditeur-seul, acceptance
survives-VPS-death tracée). 3/4 candidats P1 ont été **RÉFUTÉS** (faux-marquage
NodeCatalog, hardcode lock-3, trace acceptance).

Le seul P1 **CONFIRMÉ** (skeptic #3, deux chemins) = **`DURESS-BOOT-LEAK`** :
le mode duress échange l'identité (keypair leurre) mais partage le data root réel ;
`reannounce_seeds_at_boot` (S74 Phase F) + republication feed S66 (`runtime.rs:
769-854`) émettaient automatiquement au boot des données du vrai data root signées
sous la clé leurre. **Fix landed : `23a08c9` (HEAD)** — short-circuit duress en
tête des 2 chemins (miroir de `run_boot_seed_driver` / `boot_seed_driver_noop_in_
duress`), tests duress 11/11 + clippy clean. **Condition de blocage S76 Phase A
LEVÉE.**

Note process : les agents `nexus-audit-gate` + `supervisor` ne sont pas
enregistrés → fallback workflow 18-agents + hooks (backstop mécanique D17). Les
14 P2 sont routés §6 (`sprint76_audit_plan.md` 13 tracks + surfaces UX-ARRIVAL).

---

## §4 Décisions Day 0 (D1-D5 gelées)

Recherche ultra-profonde §Sources. Scoring G1 (détail `sprint76_design_review.md`):
**D1 ⚠️ D2 ⚠️ D3 ⚠️ D4 ✅ D5 ✅** — 3 ⚠️ sur 5, tous **adjust** (corrections
inline appliquées ci-dessous au bloc « Acknowledged review findings »).

### D1 — Surface « offrir ma puissance » : reuse consent 4-niveaux + caps + GPU monitor (net-new MINCE)

**Sources consultées** : BOINC `global_prefs` (WebFetch 2026-06-15) ; Folding@home
power-slider 3-crans + idle-toggle (WebSearch 2026-06-15, forum t=26083) ; Petals
v2.2.0 `--public_name` opt-in (WebSearch 2026-06-15) ; Salad/vast.ai earnings +
self-test (WebSearch 2026-06-15). Code local : `consent.rs:84-435` (ConsentLevel
+ Caps + UsageTracker + ConsentWatcher), `engine/runtime.rs:929-982` (gate consent
fail-closed dans la pompe), `engine/state_writer.rs:60-204` (snapshot SANS niveau
ni usage-vs-cap), `local_worker.rs:259-313` (worker co-localisé câblé
`Whitelist[own_doc]` hardcodé `:306-308`), `GpuConsentDialog.tsx`, `consent.ts`,
`Network.tsx:65-435`, `http.rs:423-430` (routes daemon `/api/v1/consent*`),
`worker_state_api.rs:21-31` (proxy lit `<root>/worker/state.json`).

**Retenu** (3 paragraphes). REUSE intégral du moteur consent/caps/monitor + une
surface « offrir ma puissance » net-new **mince** : front + 2 champs additifs au
snapshot + une bascule d'enrôlement at-large. Les 4 piliers (les niveaux de
consentement, caps W/VRAM/h, UsageTracker, ConsentWatcher fail-closed) sont
matures (25 tests), déjà câblés dans la pompe. Le travail S76 est de
l'exposition + wiring, pas de la logique.

Composant 1 — **l'enrôlement worker volontaire at-large** (la vraie bascule).
Aujourd'hui le seul worker at-large est le `local_worker.rs` on-demand, câblé en
dur sur le niveau least-privilege limité à son propre doc (`:306-308`) — il
**ignore** le `consent.json` que l'utilisateur édite via le dialog. Le panneau
doit (a) écrire le niveau choisi dans `consent.json` (route existante), et (b)
faire que le worker co-localisé **lise ce `consent.json` utilisateur** quand le
partage public est activé. Le modèle d'activation = le **niveau lui-même** (à la
BOINC : le réglage EST l'état), pas un flag `enabled` séparé. Pause = retomber au
niveau least-privilege (réversible instantané via ConsentWatcher). *Nomenclature
réelle* : l'enum est `OwnProjects / OpenSource / Whitelist / All` (`consent.rs:
391-432`), PAS « L1-L4 ». Le least-privilege actuel du worker co-localisé est
`Whitelist[own_doc]`. La sémantique d'enrôlement est tranchée au bloc G1 ci-dessous.

Composant 2 — **affichage temps-réel des caps consommés** : ajouter un bloc
additif optionnel `consent: Option<ConsentSnapshot { level, max_hours_day,
hours_used_today, max_watts, max_vram_mb }>` à `WorkerStateSnapshot`. **Additif ⇒
PAS de bump `SCHEMA_VERSION`** (le doc-comment `state_writer.rs:23-29` autorise les
champs optionnels additifs sur la même version). La pompe a déjà `self.usage` +
`self.consent.current()`. Composant 3 — **page front** réutilisant
`GpuConsentDialog` + `GpuCard` + une jauge caps-consommés ; CTA principal exprime
une **intention** (« Offrir ma puissance au réseau »), pas `consent/set` ni
`kind/provider` (directive UX gelée). Reconnaissance contributeur (modèle Petals
`--public_name`) = opt-in, hors-MVP, relève de D4.

**Rejeté** (≥3) :
- **Net-new scheduler BOINC `global_prefs`** (max-CPU-%, day-of-week,
  compute-only-between-HH:MM, suspend-on-idle) — REJETÉ. Source : BOINC wiki
  (WebFetch 2026-06-15). (1) le moteur niveaux + caps W/VRAM/h couvre déjà le
  besoin volontaire MVP (25 tests) ; (2) la détection d'idle est OS-spécifique
  (X11/Win32/Quartz idle APIs) = surface multi-plateforme lourde hors scope ;
  (3) F@h lui-même n'a livré que Light/Medium/Full + idle-toggle (forum t=26083
  prouve que le raffinement idle-vs-working n'a jamais été jugé nécessaire).
- **Flag `worker_enabled: bool` séparé du niveau** (modèle toggle Salad) —
  REJETÉ. Dédouble la source de vérité (`enabled=true`+`level=least` = sert rien ;
  `enabled=false`+`level=All` = contradiction). `consent.rs:397-413` fait déjà du
  niveau l'unique porte d'admission, fail-closed (`runtime.rs:974-979`).
- **Endpoint HTTP sur le binaire worker** (modèle vast.ai daemon, axum dans
  `nexus-worker`) — REJETÉ. Décision gelée Sprint 5 D3 (`state_writer.rs:16-20`):
  worker CLI-only, snapshot fichier, daemon = seul point HTTP. Un serveur worker
  ré-introduit une surface loopback déjà durcie côté daemon et casse le split.
- **Self-test/benchmark d'enrôlement bloquant** (modèle vast.ai « verified ») —
  REJETÉ. Source : `docs.vast.ai/host/how-to-self-test`. (1) réseau non-monétaire
  (pas de SLA) ; (2) friction contraire au budget « pair frais < 1 min » ; (3) la
  qualité worker est gérée a posteriori par quorum/validator/quarantine.

**Implications code** : `consent.rs:84-435` REUSE intégral 0 changement logique ;
`state_writer.rs:60-204` + champ additif `consent` (pas de bump SCHEMA_VERSION) ;
`engine/runtime.rs:929-982` passer niveau+usage au flush snapshot ; **point de
décision clé** `local_worker.rs:259-313` (worker co-localisé lit `consent.json`
utilisateur si partage activé, vs `Whitelist[own_doc]` hardcodé `:306-308`) ;
`http.rs:423-430` routes daemon réutilisées ; front `GpuConsentDialog.tsx`,
`consent.ts`, `Network.tsx`, `coordinator.ts` (type `WorkerStateV1` + `consent?`).

### D2 — E2E cross-machine task-routing compute (lève B-3) = acceptance LIVE sur iroh-docs/blobs (transport déjà tranché, forcé par iroh 0.98 gelé)

**Sources consultées** : iroh-docs/iroh-blobs (`docs.rs` + `n0-computer/iroh-docs`,
lu 2026-06-15) ; BOINC JobReplication (wiki, 2026-06-15) ; Petals v2.0.0 /
Parallax arXiv:2509.26182 / Ray Serve 2.55.1 / GPUStack 0.6 (tous 2026-06-15,
contre-exemples). Code local : `runtime.rs:3627-3776`
(`e2e_network_execute_gate_real_http_no_frontier_mock` = gate in-process 2 nœuds
iroh), `dispatch_loop.rs:23-60` (sole-writer `task:`) + `:155-303` (commentaire
`:166` « Cross-machine/cross-node sync is S75 »), `result_sync.rs:142`
(`spawn_result_subscribe`) câblé `runtime.rs:692`, `engine/runtime.rs:847-919`
(pompe pull : `get_many_by_prefix(b"task:")` → blob par content-hash → verify →
consent), `validator.rs:219-338` (quorum), `task.rs:319-435` (sign/verify).

**Retenu** (3 paragraphes). Le transport est **déjà tranché et c'est iroh-docs +
iroh-blobs sur QUIC (relais-assisté NAT)** — exactement le stack que S75 a prouvé
cross-machine (PC↔Mac↔VPS). Aucun nouveau transport n'est introduit. **Cette
décision n'est PAS « pull node-centrique = SOTA » — elle est forcée par la
contrainte gelée iroh 0.98 + le modèle S75 déjà prouvé** (finding G1 D2). Le
« trou B-3 » n'est pas une mécanique réseau manquante (le chemin compte tient
déjà debout in-process sur deux nœuds iroh distincts) — c'est l'absence
d'exécution sur **deux processus OS sur deux hôtes physiques**. Le livrable est
donc une **acceptance LIVE scriptée**, identique en forme à la chaîne S75
survives-VPS-death, pas un nouveau protocole ni un nouveau test unitaire.

Le modèle de claim retenu = **pull node-centrique déjà implémenté** : le worker
distant scanne `task:` sur le doc projet répliqué, récupère le blob par
content-hash via iroh-blobs, vérifie la signature Ed25519, passe le filtre
consent, claim en écrivant `claim:{id}` (anti-double-claim
`task_already_handled_on_doc`). Le doc iroh-docs **EST** la file de travail
répliquée — pas de scheduler central. Le result revient signé par le chemin
existant complet : worker signe `ResultEntry` (Ed25519+JCS) → `result:{id}` →
iroh-docs réplique → `spawn_result_subscribe` forwarde au validator (guardrail
AVANT persist, D5/S73) → quorum + signature → `result_text` (M16) → kudos →
`GET /result`. La vérification de signature du result est **déjà testée**
(`dispatch_loop.rs:285-299`, P2-A-2).

Topologie de preuve, **2 paliers** : *Palier 1 (lève B-3)* VPS (coordinateur/
ancre) ↔ PC RTX 5080 (worker réel Ollama, redundancy=1) — le PC mint un invite
worker-scope, démarre `nexus-worker` (binaire OS), submit HTTP au VPS → claim+
exécution GPU réel → result signé WAN → `GET /result`. *Palier 2 (lève quorum
déterministe)* VPS + PC + Mac, `redundancy_factor=2`, tâche `verifiable=true` —
deux workers homogènes produisent le même `result_text`, le validator voit deux
sorties identiques → consensus accepté (le `min_quorum=2` de BOINC, mais
content-addressed). **Premier critère d'acceptance falsifiable** (finding G1 D2) :
mesurer le délai de réplication `result:` PC→VPS sur WAN réel ; **si > timeout du
gate (150×200ms=30s) c'est un BLOCK à diagnostiquer, PAS un timeout à rallonger**
— en référence explicite au constat S75 `SeedAnnounced peer_count:0 ~10 min`
(chemin DOC distinct du chemin gossip de feed, hypothèse à falsifier en premier).

**Rejeté** (5) :
- **Push-scheduler central type Ray Serve / GCS head node** (`docs.ray.io`
  2.55.1, 2026-06-15) — REJETÉ : tout l'état de routing dans le GCS sur le head
  node, Controller crée/détruit les replicas, proxy HTTP forwarde activement =
  point central, viole « zero coordination centrale » + le modèle pull S75.
- **RPC synchrone worker-to-worker type GPUStack/llama-box** (`docs.gpustack.ai`
  0.6, 2026-06-15) — REJETÉ : RPC server pipeline-parallel + registration
  centralisée, suppose lien basse-latence stable, incompatible batch/async WAN
  + NAT/relais. SBFB fait du task-parallel (chaque worker = tâche entière).
- **DHT custom + beam-search type Petals/Parallax** (Petals v2.0.0 2023-08 ;
  Parallax arXiv:2509.26182 + Hivemind DHT) — REJETÉ : (a) conçu pour
  model-parallel sharding (routing token-par-token sur latence), SBFB fait du
  task-parallel ; (b) DHT Hivemind/Lattica dupliquerait iroh-gossip (pin gelé)
  = wire+dép concurrents.
- **Route HTTP loopback proxifiée (worker POST son result via HTTP/Tor)** —
  REJETÉ : canal parallèle au stack iroh éprouvé, exige un port worker exposé
  (NAT hostile), contourne la réplication content-addressed. Le loopback durci
  est intra-nœud par conception ; l'étendre cross-host re-pose toute l'auth déjà
  résolue par tickets+invites M19.
- **BOINC server stack (scheduler+feeder+validator+assimilator)** (wiki
  2026-06-15) — le *modèle* (redondance/min_quorum/consensus) est RETENU
  conceptuellement (= `validate_quorum_pre_guardrail`), mais l'*implémentation*
  centralisée (shared-memory feeder, MySQL, scheduler RPC push) est REJETÉE :
  aucun P2P, serveur unique. On garde la leçon, pas le code.

**Implications code** : **zéro changement de mécanique** sur `dispatch_loop.rs:
23-60`, `engine/runtime.rs:847-919`, `result_sync.rs:142`, `validator.rs:219-338`,
`task.rs:319-435`. Le livrable B-3 est une **acceptance LIVE scriptée** (trace
SSH PC↔VPS puis +Mac dans verification.md), le test in-process `runtime.rs:3627`
reste le gate anti-régression. Point d'attention non-bloquant : la pompe vérifie
consent contre les `estimated_*` *déclarés* du `Task` (`engine/runtime.rs:929-935`,
zéro = cap inerte), jamais contre `gpu.snapshot()` réel — câblage VRAM-live
net-new hors scope strict B-3, noté au plan.

### D3 — Quorum redundancy>1 sur sorties DÉTERMINISTES : cohorte homogène exact-match (étage 1, prouvé S76) + sémantique TOPLOC en réserve documentée (étage 2)

**Sources consultées** : TOPLOC arXiv:2501.16007 (2025-01) ; Thinking Machines
(2025-09) ; SGLang (2025-09-22) ; Ingonyama (2024-09-22) ; zkLLM arXiv:2404.16109
(CCS 2024) ; Parallax (vérif « coming next ») ; ollama#10751 + llama-cpp-python#972
(2026-06). Code local : `validator.rs:202-338` (quorum exact-match sur
`result_text` brut, B.2 early-reject `:248-282`), `engine/runtime.rs:1260-1285` +
`llm/mod.rs:253-263` (`verifiable ⇒ deterministic(seed)`, temp=0 + `seed =
blake3(task_id)[..4]` — **seed déjà cross-worker-stable**), `task.rs:74-213`
(Task), **`task.rs:349-435` (`ResultPayload.model_digest` `:374` + `logprobs_hash`
`:383` EXISTENT)**, `runtime.rs:1082` (`model_digest = blake3(task.model)` = hash
du NOM, pas du fichier GGUF), `capability_store.rs` + `dispatcher.rs:37-133`.

**Retenu** (3 paragraphes). Modèle à **deux étages**, exact-match strict comme
primitive, sémantique tolérante en réserve documentée. **La décision S76 est de
NE PAS toucher le validator et de rendre le quorum *exploitable* en contraignant
la COHORTE, pas le hardware.** Les sources établissent factuellement que
l'exact-match cross-GPU hétérogène **n'est pas garanti** (Ingonyama : même
quant+même GGUF échouent ; Thinking Machines/SGLang : déterminisme scopé
même-hardware). La conclusion d'ingénierie n'est donc PAS « rendre le LLM
déterministe cross-GPU » (Ingonyama : 7× le prefill + réécriture kernels =
hors-scope, hors iroh 0.98, hors Ollama).

Étage 1 (S76, prouvé) — **exact-match sur cohorte homogène**. Définir une « sortie
déterministe exploitable » = (modèle pinné par digest + quantization pinnée +
même famille de runtime + greedy temp=0 + seed=blake3(task_id)), et router une
tâche `verifiable` redundancy>1 vers une cohorte de workers homogènes sur ce
tuple. **Correction G1 D3 (code vérifié)** : `ResultPayload.model_digest`
(`task.rs:374`) et `logprobs_hash` (`task.rs:383`) **EXISTENT DÉJÀ** comme couches
2/3 de vérification — ce n'est PAS un « ajout additif de champ ». `model_digest`
est actuellement `blake3(model_name)` (`runtime.rs:1082`), PAS le digest du
fichier GGUF (discordance avec son doc-comment qui dit « exact model file »). Le
routing cohorte-homogène S76 doit donc (i) **décider si on durcit `model_digest`
vers un hash de fichier GGUF** [P1 dans la phase, ou doc-note si hors-scope], et
(ii) advertir le tuple (model_digest, quant, runtime_family) dans la capability
worker (`capability_store.rs`). Le validator `validate_quorum_pre_guardrail`
reste **INCHANGÉ**. La preuve = deux workers homogènes (même image/CPU StubBackend
hermétique, ou deux Ollama même quant) exécutant la même tâche `verifiable`
redundancy=2/3 → `best_count > threshold`, `result_text` byte-identique
cross-process. Résultat **attendu honnête écrit dans l'acceptance** (anti faux-vert
T1) : exact-match tient en cohorte homogène, **diverge** sur GPU hétérogène (que
le validator rejette correctement — outlier logging déjà là).

Étage 2 (réserve, NON codé S76 — design note + threat-model row) — **vérification
sémantique tolérante type TOPLOC** : commitment top-k du dernier hidden state,
comparé exponent/mantissa, « works reliably across non-deterministic GPU
hardware ». **Correction G1 D3** : le slot existe déjà et s'appelle
**`logprobs_hash`** (`task.rs:383`, « layer 3 »), pas `result_hash`. Travail futur
(Ollama n'expose pas les hidden states ; `LlamaCppBackend` C-API oui), donc lié au
backend feature-gated `llm_llama_cpp`. **Aucun bump wire** : champs déjà présents
dans le v1 signé.

**Rejeté** (5) :
- **Rendre le LLM déterministe cross-GPU (batch-invariant kernels / Ingonyama
  GEMM rewrite)** — REJETÉ. Ingonyama (2024-09-22) : prefill 302→43 tok/s,
  réécriture kernels + ban Tensor cores ; SGLang (2025-09) : +34% latence, TP1/TP2
  only. Backends Ollama+llama.cpp stock (gelé), iroh 0.98 pinné. Hors d'échelle +
  inutile si cohorte homogène.
- **Match sémantique/tolérant texte (Levenshtein, embeddings, LLM-judge)** —
  REJETÉ. (a) Non déterministe (un judge LLM ré-introduit le problème) ; (b)
  surface d'attaque (deux réponses « proches » dont une malveillante passeraient) ;
  (c) TOPLOC montre que la tolérance correcte se fait sur les *activations*, pas
  le texte. Le validator exact-match (`validator.rs:284-337`) est plus sûr et testé.
- **zkLLM / preuve ZK d'inférence** — REJETÉ. arXiv:2404.16109/CCS 2024 : 803s
  proving + 986s commit / forward pass LLaMA-2-13B, ~18-23 jours pour 2000 tokens.
  « Prohibitively expensive » (auteurs).
- **Pas de redondance — confiance simple (Petals/Parallax)** — REJETÉ. Petals
  n'a aucune vérification ; Parallax « coming next » (non livré 2026). Supprimer
  le quorum = abandonner l'objectif ; canary/quarantine suppose un quorum amont.
- **Hash des logits complets (au lieu top-k/texte)** — REJETÉ. Non-associativité
  FP fait diverger les logits de petite magnitude (Thinking Machines = la source
  du problème). TOPLOC ne hash que les top-k précisément car « small values are
  more susceptible to rounding ». Pire des deux mondes.

**Implications code** : `validator.rs:202-338` quorum exact-match **INCHANGÉ**
(verrou) ; `engine/runtime.rs:1260-1285` + `llm/mod.rs:253-263` `deterministic(seed)`
déjà câblé, seed cross-worker-stable (rien à changer) ; **`task.rs:374`
`model_digest` à corriger (blake3 nom → blake3 fichier GGUF) OU doc-note** +
`task.rs:383` `logprobs_hash` = slot TOPLOC étage 2 (additif, déjà v1) ;
`capability_store.rs` + `dispatcher.rs:37-133` = routing réplicas vers workers
homogènes (net-new principal) ; `dispatch_loop.rs:155-303` (`multi_thread`
MANDATORY P2-A-1 déjà imposé) base du test cross-process redundancy>1.

### D4 — Dashboard contributeur : comptabilité kudos non-monétaire per-task (vue d'agrégation contributeur sur le ledger existant)

**Sources consultées** : BOINC CreditNew/CreditOptions (claimed vs granted,
WebFetch 2026-06-15) ; F@h QRB + passkey/≥10 WU (2026-06-15) ; Gridcoin RAC→GRC
(2026-06-15, rejeté) ; EigenTrust (Stanford 2003, rejeté). Code local :
`kudos_ledger.rs:51-163` (`credit()` = `log_utility(tokens_generated)`, chaîne
BLAKE3 **per-project** `:64`, EMA `alpha=0.97` `:124-132`, agrégation per-project
`:134-163`), `validator_loop.rs:70-120` (`credit()` UNIQUEMENT après `Accepted`+
guardrail, `worker_id = hex(entry.worker_pubkey)` = clé qui a SIGNÉ), `task.rs:
349-435` (`tokens_generated` `:363` dans le payload signé mais **hors quorum** —
le validator ne compare que `result_text`), `db.rs:1025-1068` (`list_kudos_entries`
+ `worker_contributions` sans EMA), `consent.rs:229-328` (`UsageTracker`/`usage.json`
local worker-side, jamais répliqué au coordinator).

**Retenu** (3 paragraphes). Adopter la séparation BOINC **« claimed vs granted »**,
appliquée per-task et keyée sur la clé Ed25519 du résultat validé. L'invariant clé
de BOINC n'est pas la formule (notre `log_utility` joue ce rôle) mais *granted =
fonction de l'instance canonique/consensus, jamais du claim brut*. Notre code
respecte déjà la moitié : `credit()` n'est appelé qu'après `Accepted`
(`validator_loop.rs:70`), et sous redundancy>1 ce `Accepted` exige un quorum strict
sur `result_text` ; le worker crédité est la clé qui a **signé** (`entry.worker_
pubkey`), pas une auto-désignation = notre « canonical instance ».

**Per-task est déjà la granularité native** : le ledger stocke une ligne
`KudosEntry` *par task*. Le « per-project » d'aujourd'hui n'est qu'une **vue
d'agrégation** (`get_project_kudos`). Le dashboard contributeur = une **deuxième
vue keyée sur `worker_node_id`** sur les mêmes lignes, réutilisant exactement
`effective_score()` (EMA `alpha=0.97`). Fonction `get_contributor_kudos(db,
worker_node_id, now)` miroir de `get_project_kudos`, agrégeant kudos effectifs
(EMA), tâches servies (= lignes validées), per-project breakdown. Affichage = 3
métriques honnêtes : kudos effectifs (EMA), tâches servies, **GPU-heures données
LOCALES** (lues depuis `usage.json` du nœud — honnête « heures que cette machine a
données », non-attestées ; les GPU-heures ne sont PAS dans le ledger).

**Anti-gaming** : l'attribution est saine (quorum-gated credit + clé signataire).
**Le trou réel** : `amount = log_utility(tokens_generated)` et `tokens_generated`
est self-déclaré dans le payload signé (`task.rs:363`) **mais n'est pas dans la
comparaison de quorum** — deux workers honnêtes produisent le même `result_text`
(passent le quorum) mais peuvent déclarer des `tokens_generated` divergents → un
worker malhonnête gonfle son `tokens_generated` sans échouer le quorum. C'est le
problème « claimed credit » que BOINC résout par averaging des instances
canoniques. Décision PO au plan (Q D4) : (a) **durcir** `amount =
log_utility(median(tokens_generated))` du groupe d'accord (BOINC discard-high/low)
+ sanity-bound `tokens ≤ f(generation_time_ms)`, OU (b) **documenter le trou en
P2** (la `log_utility` compresse déjà <10×, test `log_utility_compression`).

**Rejeté** (5) :
- **Gridcoin Proof-of-Research RAC→magnitude→GRC** (`gridcoin.us/wiki/magnitude`,
  2026-06-15) — REJETÉ, **viole décision gelée** : convertit la RAC en jeton
  monétaire transférable. Anti-pattern direct de « kudos non-monétaire, zéro
  token crypto ». On réutilise l'insight RAC=EMA, on rejette le pont monétaire.
- **EigenTrust trust global power-iteration** (Stanford 2003, 2026-06-15) —
  REJETÉ : (a) ranking global normalisé (SBFB rejette par principe) ; (b)
  faiblesse Sybil explicite (papier) résolue « by imposing a cost » que SBFB met
  ailleurs (PoW, invite M19) ; (c) exige un graphe de transactions pair-à-pair
  qu'on n'a pas (crédits émis par coordinator post-quorum).
- **F@h QRB `base*sqrt(k·deadline/elapsed)`** (2026-06-15) — REJETÉ comme formule
  (prime la vitesse → pousse hardware cher/watts, tension avec volontaire+caps),
  **partiellement retenu** comme garde-fou d'affichage (passkey + ≥10 WU = ne pas
  afficher un contributeur avant N tâches validées).
- **Per-project uniquement (statu quo)** — REJETÉ : ne couvre pas le scope S76
  (vue duale contributeur). Les lignes portent déjà `worker_node_id` ; gap
  d'agrégation/route/UI, pas de manque de données.
- **GPU-heures comme champ wire signé** — REJETÉ pour S76 : self-déclaré donc
  gameable (même classe que `tokens_generated`), ouvre une surface wire
  pré-launch alors que `started_at`/`finished_at`/`generation_time_ms` existent
  déjà (`task.rs:368,386,389`) et que le panneau honnête lit `usage.json` local.

**Implications code** : `kudos_ledger.rs:124-163` `get_contributor_kudos` miroir
de `get_project_kudos` ; `db.rs:1025-1068` query `WHERE worker_node_id=?1` +
**index SQLite sur `worker_node_id`** ; `kudos_api.rs:44-144` handler
`contributor_dashboard(Path(node_id))` miroir `leaderboard()` ; `validator_loop.rs:
108-120` + `http.rs:3342-3351` = point de décision anti-gaming `tokens_generated` ;
`consent.rs:229-328` `usage.json` = source GPU-heures locales ; front page
contributeur réutilisant `Network.tsx` (GpuCard + ProjectsServedCard).

### D5 — Quantization 4-bit DOCUMENTÉE (GGUF doc-only ; runtime quant déjà présent ; cible 1 GPU/contributeur ≤14B modèle entier ; gros modèles = sharding cross-machine S77)

**Sources consultées** : bartowski Llama-3.3-70B-GGUF + Qwen2.5-14B-GGUF
(HuggingFace, 2026-06-15) ; arXiv 2601.14277v1 (2026-01-11, <90j) ; llama-cpp-2
`LlamaModelParams` (`docs.rs`, 2026-06-15) ; oobabooga/spheron/ahmadosman
(AWQ/GPTQ/EXL2, 2026-06-15). Code local : `llm/llama_cpp.rs:143-164` (`ensure_model`
ne pose QUE `with_n_gpu_layers`, mono-GPU), `config.rs:331-372` (`LlamaCppConfig`
sans champ format-quant ni tensor-split), `llm/mod.rs:173-197` (tag exemples
`qwen2.5-7b-instruct-q4_k_m`, trait `LlmBackend` `:315`), `gpu/mod.rs:147-151`
(`vram_budget_remaining_bytes`), `consent.rs:417-432` (cap VRAM).

**Retenu** (3 paragraphes). **S76 LIVRE de la DOCUMENTATION, pas un nouveau runtime
de quantification.** Le runtime quantifié existe déjà entièrement : `LlamaCppBackend`
charge n'importe quel GGUF pré-quantifié via `load_from_file`, offload GPU via
`n_gpu_layers`. Le format 4-bit (Q4_K_M, IQ4_XS, Q4_0) est **baked dans le fichier
`.gguf`** — aucun « chemin de quantification » à coder côté worker : l'opérateur
télécharge un GGUF déjà au bon format et le pointe via `model_path`. Le livrable
S76 = (1) un doc opérateur recommandant le format par taille de carte, (2) la
table d'empreintes VRAM, (3) le branchement honnête des caps VRAM existants.

Format recommandé : **Q4_K_M par défaut** (défaut « recommended » bartowski +
meilleure ppl 4-bit 7.56 vs Q4_0 7.74, arXiv 2601.14277), **IQ4_XS** quand on
serre la VRAM, **Q4_K_S** si l'arXiv prime (diff dans le bruit).
**Cadrage produit (arbitrage PO Checkpoint §11)** : un contributeur a **UNE** carte
(16GB), pas deux — le mono-machine 2-GPU n'est pas un vrai déploiement. La **cible
honnête single-GPU = ≤14B** (Qwen2.5-14B Q4_K_M ~8.5 GB tient 1×16GB, **modèle
ENTIER** servi par un worker). Les gros modèles (32B/70B) ne tiennent PAS sur une
carte 16GB (70B Q4_K_M = 42.5 GB, IQ4_XS = 37.9 GB, même Q2_K = 26.4 GB) ; le
chemin vers ces tailles n'est PAS « ajouter une 2e carte » mais **éclater le modèle
sur 2+ machines à 1 GPU chacune = sharding cross-machine = S77**. L'offload CPU
mono-machine reste documenté comme palliatif lent (2-5 tok/s, batch/async), pas
comme la voie principale. S76 prouve d'abord le task-routing d'un modèle ENTIER
cross-machine ; S77 l'éclate.

Table d'empreintes à documenter : 7B/8B Q4_K_M ~4.6 GB (1×16GB, modèle entier,
40-80 tok/s) ; **14B Q4_K_M ~8.5 GB (1×16GB, cible honnête single-GPU, 25-50)** ;
32B Q4_K_M ~22 GB (1 carte 24GB, PAS 16GB — hors cible contributeur) ; 70B Q2_K
~26.4 GB / IQ4_XS ~37.9 GB / Q4_K_M ~42.5 GB (**ne tient sur AUCUNE carte 16GB ;
voie = sharding cross-machine 2+ machines × 1 GPU = S77** ; l'offload CPU
mono-machine 2-5 tok/s n'est qu'un palliatif documenté). **Pré-condition quorum**
(lien D3) : deux workers DOIVENT utiliser le MÊME GGUF (même quant, même build)
pour un exact-match — la doc l'impose.

**Rejeté** (6) :
- **AWQ** (oobabooga/spheron 2026-06-15) — REJETÉ : (a) GPU-only, pas d'offload
  CPU → inutilisable pour le cas carte 16GB single-GPU ; (b) runtime vLLM/Python
  incompatible abstraction `LlmBackend` Rust in-process ; (c) nouveau backend
  entier hors scope.
- **GPTQ** (oobabooga 2026-06-15) — REJETÉ : ~3 pts MMLU sous AWQ (dominé), même
  contrainte GPU-only Python, aucun avantage qualité sur GGUF Q4_K_M ni intégration.
- **ExLlamaV2/EXL2** (ahmadosman 2026-06-15) — REJETÉ malgré 40-70% plus rapide :
  GPU-only strict zéro offload, runtime Python, pas de binding Rust. La vitesse
  n'est pas limitante (batch/async 1-3 tok/s).
- **bitsandbytes 4-bit (NF4/load_in_4bit)** (oobabooga 2026-06-15) — REJETÉ :
  quant on-the-fly HF/PyTorch, exige Transformers/Python, incompatible worker
  Rust in-process, plus lent + moins bonne qualité que GGUF k-quants.
- **Q2_K pour rabougrir un 70B** — REJETÉ comme défaut : 26.4 GB ne tient toujours
  pas sur une carte 16GB (et le mono-machine 2-GPU n'est pas une cible, PO) ;
  qualité « very low » (bartowski) ; option documentée extrême, pas recommandation.
  Le bon chemin 70B = **sharding cross-machine S77** (2 machines × 1 GPU).
- **Câbler le tensor-split mono-machine multi-GPU DANS S76**
  (`with_split_mode`+`with_devices` vérifiés docs.rs) — **REJETÉ FERMEMENT
  (arbitrage PO Checkpoint §11, NON ré-évaluable)** : un contributeur a **UNE**
  carte — le mono-machine 2-GPU n'est pas un vrai déploiement, donc le câbler n'a
  pas de cible (« personne n'a 2 GPU »). Le SEUL chemin multi-GPU réaliste est
  **cross-machine** (2 machines × 1 GPU) = le **sharding pipeline S77** (le travail
  original = validation des activations entre pairs non-confiants). S76 prouve le
  task-routing d'un modèle ENTIER cross-machine AVANT de l'éclater (séquencement
  roadmap « ne pas empiler »).

**Implications code** : `llm/llama_cpp.rs:143-164` câble UNIQUEMENT
`with_n_gpu_layers` — **inchangé si doc-only** (tensor-split = S77) ;
`config.rs:331-356` `LlamaCppConfig` inchangé (ajout `tensor_split`/`devices`
SEULEMENT si S77) ; `gpu/mod.rs:147-151` `vram_budget_remaining_bytes` +
`consent.rs:417-432` réutilisables tels quels (gate admission par budget) ; doc
cible `docs/operators/QUANTIZATION.md` + lien depuis le panneau D1.

---

**Acknowledged review findings (G1)** — `sprint76_design_review.md` : **3 ⚠️
(D1, D2, D3), tous adjust, corrections appliquées INLINE ci-dessus.**

- **D1 ⚠️ adjust APPLIQUÉ** : les deux « risques » sont promus en décisions.
  (a) Le préfixe `/api/v1` vs `/consent/set` (`consent.ts` POST `/consent/set` vs
  daemon `http.rs:423 /api/v1/consent/set`) est un **pré-requis bloquant de la
  phase D1**, pas un risque — première tâche de la phase « offrir ma puissance »
  (vérifier `web/vite.config.ts` + réconcilier ; critère : POST consent depuis le
  front packagé écrit `consent.json` ; fix(sprint76) légitime si trou prod). (b)
  La **sémantique d'enrôlement worker co-localisé est TRANCHÉE** :
  `OwnProjects`/`Whitelist`(least-priv) = OFF (le worker co-localisé garde son
  `Whitelist[own_doc]` actuel `local_worker.rs:307-308`) ; `OpenSource`/`All` = le
  worker co-localisé **lit le `consent.json` utilisateur** ; `All` reste un opt-in
  **double-confirmé** (cohérent `threatNote` « risque maximum »). Noms d'enum
  réels (`consent.rs:391-432`), pas « L1-L4 ».
- **D2 ⚠️ adjust APPLIQUÉ** : (a) la justification « pull vs push » est réécrite
  comme **forced-by-frozen-iroh-0.98 + modèle S75 prouvé**, PAS « SOTA pull »
  (aucune source <90j n'étaye le point de décision ; la décision est de toute
  façon contrainte). (b) La **convergence `result:` cross-machine WAN est le 1er
  critère d'acceptance falsifiable** de la phase B-3, en référence explicite au
  constat S75 `SeedAnnounced peer_count:0 ~10 min` : délai > timeout du gate
  (150×200ms) = BLOCK à diagnostiquer, pas timeout à rallonger (anti faux-vert).
- **D3 ⚠️ adjust APPLIQUÉ** : (a) recency marquée honnêtement (faits physiques FP
  stables 2024-2025, pas de publi <90j ; la seule <90j [arXiv 2601.14277] relève
  de D5). (b) **Implication code réécrite** : `ResultPayload.model_digest`
  (`task.rs:374`) et `logprobs_hash` (`task.rs:383`) **EXISTENT** ; `model_digest`
  = `blake3(nom)` (`runtime.rs:1082`), PAS le fichier GGUF ; le routing
  cohorte-homogène S76 doit (i) durcir `model_digest` → hash fichier GGUF [P1 ou
  doc-note] + (ii) advertir le tuple en capability ; **`logprobs_hash`, pas
  `result_hash`, est le slot TOPLOC de l'étage 2**. (c) Résultat attendu honnête
  (exact-match en cohorte homogène, diverge GPU hétérogène) écrit comme critère
  d'acceptance.
- **D4 ✅ / D5 ✅** : 0 ⚠️ G1. **Arbitrages PO Checkpoint §11 (TRANCHÉS, gelés)** :
  **D1** = `OpenSource`+`All` ouvrent le partage (least-priv `OwnProjects`/
  `Whitelist` = OFF). **D2** = convergence `result:` WAN = 1er critère falsifiable
  (confirmé). **D3** = durcir `model_digest` (nom→fichier GGUF) en **P1 dans la
  phase compute**. **D4** = **durcir maintenant** `amount=log_utility(median(
  tokens_generated))` du groupe d'accord quorum + sanity-bound (Phase E modifie
  `credit()`). **D5** = **doc-only, mono-machine 2-GPU ENTERRÉ** (« personne n'a 2
  GPU ») : cible single-GPU ≤14B modèle entier ; le chemin gros-modèle = **sharding
  cross-machine 2 machines × 1 GPU = S77**, pas le tensor-split mono-machine ; S76
  prouve d'abord le task-routing cross-machine.

---

## §5 Plan Phase outline A-G

Sprint PAIR : **Phase B est la phase dette réservée, NON convertible en feature
(Règle 1 G7)**. Le P1 étant déjà fixé (`23a08c9`), Phase B est purement
dette/refacto/tests/doc. Critical path features = A (panneau) → puis le compute
B-3/quorum (C/D) ; B (dette) est insérée tôt pour fermer le lot duress restant et
les carries à 2 reports avant que les phases compute ne touchent les mêmes
fichiers.

- **Phase 0** — Audit gate S75 (DONE = CONDITIONAL PASS `73831c0`, P1 levé
  `23a08c9`).
- **Phase A** — **Panneau « offrir ma puissance » + enrôlement worker co-localisé**
  (D1). Réconcilier le préfixe route `/api/v1` (pré-requis bloquant) ; champ
  additif `consent` au snapshot (0 bump SCHEMA_VERSION) ; worker co-localisé lit
  `consent.json` utilisateur si partage activé (`All` double-confirmé) ; front
  intention « Offrir ma puissance » + jauge caps consommés. *Critère* : POST
  consent depuis front packagé écrit `consent.json` + jauge h/h affichée + tests
  enrôlement par niveau.
- **Phase B** — **DETTE RÉSERVÉE (non convertible)**. Lot duress freres local-only
  (DURESS-FRERES-LOCAL + publisher-binding observed) ; CARRY-3-AGGREGATOR-SANITIZE
  (2 reports, anti-escalade) ; PULL-3 cross-tier failover + investigation
  SeedAnnounced (2 reports) ; LOOPBACK-TIERS-STALE (2 reports, anti-escalade) ;
  T6-OUTBOX-DIRECT ; WS-3/PD-5 hoisting ; DISCRIMINATEUR-CURATOR-ANCRE ;
  THREAT-BLOBSERVE-BEARER ; lot test front (5 pages 0-test + CI-PLAYWRIGHT-NOOP +
  shell T6/T7) ; BRIDGE-ALLOWLIST-DRIFT ; UX-ARRIVAL-PLAN-INSCRIPTION. *Critère* :
  3 carries à 2 reports fermés + lot duress no-op testé + coverage front ≥ seuils
  + 0 bump wire.
- **Phase C** — **E2E cross-machine compute B-3 (palier 1) + cohorte homogène**
  (D2 + D3 étage 1). Acceptance LIVE scriptée PC↔VPS (worker réel Ollama,
  redundancy=1) ; corriger `model_digest` (blake3 nom → fichier GGUF) [P1 ou
  doc-note] + advertir tuple (model_digest, quant, runtime_family) en capability ;
  routing réplicas vers cohorte homogène. *Critère* : trace SSH PC→VPS result
  signé rendu + 1er critère falsifiable convergence `result:` WAN < timeout +
  capability tuple testée.
- **Phase D** — **Quorum redundancy>1 prouvé déterministe (palier 2)** (D3 étage 1
  suite). Acceptance LIVE VPS+PC+Mac `redundancy_factor=2` `verifiable=true` →
  `result_text` byte-identique → consensus ; test cross-process StubBackend
  redundancy>1 hermétique ; design note TOPLOC (`logprobs_hash` slot étage 2,
  feature `llm_llama_cpp`). *Critère* : quorum=2 exact-match LIVE + test
  cross-process déterministe + résultat hétérogène-diverge documenté (anti
  faux-vert).
- **Phase E** — **Dashboard contributeur** (D4). `get_contributor_kudos` miroir
  `get_project_kudos` + index SQLite `worker_node_id` ; route
  `contributor_dashboard` ; front page contributeur (kudos effectifs / tâches
  servies / GPU-heures locales `usage.json`) ; décision anti-gaming
  `tokens_generated` (durcir median vs P2 documenté). *Critère* : route + page +
  3 métriques honnêtes + tests agrégation EMA.
- **Phase F** — **Quantization 4-bit documentée** (D5). `docs/operators/
  QUANTIZATION.md` (table empreintes + reco format par carte + pré-condition
  quorum même-GGUF) ; **cadrage PO : 1 GPU/contributeur, gros modèles = sharding
  cross-machine S77** (mono-machine 2-GPU enterré, « personne n'a 2 GPU ») ; lien
  depuis panneau D1 (« ta carte 16GB → modèles ≤14B Q4_K_M, modèle entier ») ;
  gate admission par cap VRAM existant (design note). *Critère* : doc présente +
  table + cible single-GPU ≤14B honnête + 70B = sharding cross-machine S77
  explicite (PAS mono-machine 2-GPU).
- **Phase G** — **Wrap-up + acceptance + clôture arc**. verification.md fail-fast
  (Win + Docker canonique gate AVANT push) ; `sprint77_audit_plan.md` ; carries
  reconduits S77 ; THREAT_MODEL rows compute/duress ; PATTERNS rust+shell ;
  SPRINT_LOG row ; CLAUDE.md état ; roadmap v5 (Arc 3.5 6/6 clos, S77 sharding
  ouvert). *Critère* : N/N fail-fast verts + acceptance LIVE B-3+quorum tracée +
  artefacts écrits.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 76)

**Aucun item 3/3 MANDATORY à l'entrée S76.** Mais **4 carries sont à 2 reports**
(un 3e report = MANDATORY S77, Règle 2) — 3 sont traités en Phase B pour casser
l'escalade.

| Item | Reports | Phase S76 | Exit condition |
|---|---|---|---|
| (aucun 3/3 à l'entrée) | — | — | — |

### Carry absorbés S76 (Phase B dette + phases features)

| Item | Reports | Phase S76 | Exit condition |
|---|---|---|---|
| DURESS-FRERES-LOCAL | 1/3 | B | `seed_voluntary`+`set_keep_online` no-op en duress (3 tests ZÉRO mutation data root) + THREAT_MODEL row |
| CARRY-3-AGGREGATOR-SANITIZE | **2/3** | B | `trustworthy_open_source` downgrade à l'INGRESS `runtime.rs:2231` (chokepoint) + test ingress + THREAT_MODEL doc |
| PULL-3 cross-tier failover | **2/3** | B | fallback ordonné ticket-mort→directory→multi-provider câblé driver E + test multi-tier + note investigation SeedAnnounced |
| LOOPBACK-TIERS-STALE | **2/3** | B | 7 routes S74+S75 inscrites §3 (T0 ; T1 candidats /directory/publish + /seed/request) + phrase fausse plan d'audit corrigée |
| T6-OUTBOX-DIRECT | 1/3 | B | test 2-nœuds `GossipCmd::Outbox` neighbor_count>0 (pattern hijack-guard A) |
| WS-3/PD-5 hoisting | 1/3 | B | `my_endpoint_addr()` hoisté once-per-pass (`runtime.rs:1655-1850`) + nextest inchangé vert |
| OBSERVED-FORGED-IDS / Publisher-binding | 1/3 | B | capture observed liée à l'identité PoW publisher (borne la forge) |
| DISCRIMINATEUR-CURATOR-ANCRE | 1/3 | B | `listCurators().entries` distingue curator-pur vs ancre (0 wire) + copy /nodes honnête |
| THREAT-BLOBSERVE-BEARER | 1/3 | B | cellule mitigation corrigée (route publique ; amplification bornée subscribed-only+cap+timeout) |
| FRONTEND-COVERAGE-GAP + CI-PLAYWRIGHT-NOOP + shell T6/T7 | 1/3 | B | smoke render 5 pages 0-test + ≥1 spec Playwright réel OU étape [10] retirée + coverage ≥ 85/85/78/85 |
| BRIDGE-ALLOWLIST-DRIFT | 1/3 | B | allowlist Rust(10)↔TS(15) alignée OU test parité + doc `manifest.methods` déclaratif |
| UX-ARRIVAL-PLAN-INSCRIPTION | 1/3 | B (doc) | surface UX-ARRIVAL inscrite au sprint76_audit_plan comme track couvert |
| PULL-2 multi-provider (→ devient socle D2 dial-set) | clos S75 D | C | rappel : `fetch_hash_multi` câblé S75-D ; PULL-3 = le résidu failover |

### Carries reconduits S77 (compteur incrémenté + justification renouvelée)

| Item | Reports | Justification RENOUVELÉE |
|---|---|---|
| SYBIL-SEEDER-TAIL | 2/3 → 3/3 si non fait | Résiduel **availability-only** non-sécuritaire (ancre slot-0 non-crowdable, verrou tient) ; **dépendance interne** : S77 sharding touche le dial-set/topology → regroupement naturel du sampling avec le sharding, pas un fix isolé. Exemption nommée, sinon MANDATORY S77. |
| REVISION-HOME-DURABILITY | 1/3 → 2/3 | Mitigé par systemd `SBFB_HOME` épinglé (vérifié LOOPBACK header S75) ; surveiller au S76 si un mode déploiement sans home pinné apparaît. Pas exploitable pré-launch. |
| KNOWN-ENTRY-OVERCOUNT | 1/3 → 2/3 | Superset **HONNÊTE** (curator-list + annuaire) ; dedup (pid,hash) requis SEULEMENT si une UX future affiche « N apps découvrables ». Pas de consommateur UI → pas de bug aujourd'hui. |
| seeder catalog_len:0 (constat live G) | 1/3 → 2/3 | **Question DESIGN PO** (section « seeded » distincte non-autoritaire vs verrou-4 + F-Droid). Bloque sur arbitrage PO, pas sur code. |
| RE-DRIVE-ON-INGEST (P3) | 1/3 → 2/3 | Fenêtre morte 1er boot, remède opérateur documenté (restart). P3 conception ; lié SeedAnnounced/PULL-3 — si B3 résout la convergence, peut se fermer en cascade. |
| T-NN+3 canonical_bytes dup | open S70 | Absorbable au prochain sprint touchant JCS crypto ; pas forcé par scope S76 GPU. |

### Attention 3/3 S77 (signal PO)

| Item | Reports actuels | Si non traité Phase B S76 |
|---|---|---|
| CARRY-3-AGGREGATOR-SANITIZE | 2/3 | → 3/3 MANDATORY S77 (**traité en B ce sprint** pour éviter) |
| LOOPBACK-TIERS-STALE | 2/3 | → 3/3 MANDATORY S77 + escalade G7 (**traité en B**) |
| PULL-3 | 2/3 | → 3/3 MANDATORY S77 (**traité en B**, lié au cœur quorum) |
| SYBIL-SEEDER-TAIL | 2/3 | → 3/3 MANDATORY S77 sauf exemption « dépendance interne sharding » nommée (reconduit avec cette exemption) |

### LT items (ROADMAP_COMMITMENTS, Règle 3)

- **LT-2 Radicle flip** : trigger ARMÉ, dry-run privé FAIT (rad 1.9.1, RID
  `rad:z2v4pg…`, visibility=private). Flip publie le repo = **action IRRÉVERSIBLE,
  décision PO HORS-SPRINT**. Ne PAS embarquer dans S76.
- **LT-5 Redundancy persistence** : la primitive Python `RedundancyDispatcher` est
  **superseded** (retirée S71-B faute de consommateur) ; le quorum redundancy>1
  S76 réalise l'intention via le **quorum DB-backed** (`validator.rs`). Le kickoff
  acte LT-5 résorbé par le pivot Rust — **ne PAS re-coder le dispatcher Python**.
- **LT-7 Self-hosted build** : gate satisfait (Tier 1+2 S55, Tier 3 S60) ;
  `execute_build` **dormant**. S76 ramène le compute (B-3 levé) = moment naturel
  de **décider** : câbler `execute_build` (router `task_type=="build"`) OU le
  marquer dormant-jusqu'à-S77. **Décision de scope kickoff** : reste dormant si le
  scope GPU ne le permet pas (S76 déjà chargé) ; ré-évaluable Phase C preflight.
- LT-1 (clos pre-v1.0), LT-3/LT-4 (latents, conditions non remplies), LT-6
  (RESOLVED S32) : aucune action.

### Externes inchangés (exemptions)

P2-A-1 rand (exemption upstream) ; P2-AUDIT-2 iroh (pin 0.98 gelé) ; T-NN+2 iframe
Rust-wasm (§P34, blocker toolchain confirmé) ; P3-OS-1 operator_server (Docker-sur-
Windows bind-mount non fidèle, canonique = CI Linux natif).

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale (factuel) |
|---|---|---|---|
| 1 | **Sharding pipeline** (modèle 70B éclaté sur 2+ machines × 1 GPU) | S77 | Feature distincte (directive PO `feedback_ultra_complete_sprints`), pas un defer du GPU. 70B (42.5 GB) ne tient sur AUCUNE carte 16GB ; **arbitrage PO « personne n'a 2 GPU »** → le seul chemin = sharding **cross-machine** (pas mono-machine 2-GPU). S76 prouve d'abord le task-routing cross-machine. |
| 2 | **Tensor-split mono-machine multi-GPU** (`with_split_mode`+`with_devices`) | rejeté (renvoie #1 S77) | **Arbitrage PO Checkpoint §11 : « personne n'a 2 GPU »** — le mono-machine 2-GPU n'est pas un vrai déploiement, rien à câbler. Le multi-GPU réaliste = cross-machine (2 machines × 1 GPU) = sharding pipeline (#1, S77). NON ré-évaluable. |
| 3 | **Câblage VRAM-live à l'admission** (cap vs `gpu.snapshot()` réel) | S77 | Aujourd'hui le cap vérifie l'`estimated_*` déclaré du Task (`engine/runtime.rs:929-935`, zéro=inerte). Net-new hors scope strict B-3 ; jauge heures (mesurée) suffit pour le panneau honnête. |
| 4 | **Durcir `tokens_generated` hors-quorum** (median du groupe d'accord) | S76-D ou P2 | Décision PO au plan : durcir (modifie signature `credit()`) OU documenter P2 (la `log_utility` compresse déjà <10×). Pas un défaut bloquant. |
| 5 | **TOPLOC étage 2 implémenté** (commitment top-k hidden state) | post-S77 | Ollama n'expose pas les hidden states ; lié `LlamaCppBackend` C-API feature-gated. Design note + slot `logprobs_hash` posé S76, implémentation future. |
| 6 | **Quorum cross-GPU hétérogène** (exact-match sur hardware différent) | post-S77 (TOPLOC) | Factuellement impossible en stock (Ingonyama : même GGUF échoue) sans réécriture kernels. S76 prouve la cohorte homogène ; l'hétérogène attend TOPLOC. |
| 7 | **`execute_build` câblé (LT-7)** | S77 (ré-éval C) | S76 déjà chargé (B-3 + quorum + dashboard + panneau) ; décision dormant-jusqu'à-S77 sauf trivialité au preflight. |
| 8 | **Reconnaissance contributeur publique** (modèle Petals `--public_name`) | post-launch | Opt-in non-MVP ; le dashboard contributeur D4 livre le cœur, l'attribution publique réseau-wide est cosmétique. |
| 9 | **Self-test/benchmark d'enrôlement** (modèle vast.ai « verified ») | jamais (rejeté) | Réseau non-monétaire (pas de SLA) ; friction contraire au budget pair frais < 1 min ; qualité gérée a posteriori par quorum/quarantine. |
| 10 | **Scheduler horaire/idle BOINC `global_prefs`** (day-of-week, idle-detect) | post-launch | Détection idle OS-spécifique (X11/Win32/Quartz) = surface multi-plateforme lourde ; le moteur niveaux + caps couvre le besoin volontaire MVP. |
| 11 | **AWQ/GPTQ/EXL2/bitsandbytes** (runtimes quant alternatifs) | jamais (rejeté) | Tous GPU-only Python hors-stack Rust in-process ; changer de format = changer de runtime = casse `LlmBackend`. SBFB = GGUF/llama.cpp. |
| 12 | **GPU-heures attestées réseau-wide** (champ wire signé) | post-launch | Self-déclaré = gameable (classe `tokens_generated`) ; ouvre surface wire pré-launch ; `usage.json` local honnête suffit pour le panneau. |
| 13 | **Upgrade iroh 1.0** (1.0.0-rc.1 disponible 2026-05-27) | Gate-1/PO | Décision gelée iroh 0.98 pinné ; upgrade = décision PO/Gate-1, pas un sprint feature. |
| 14 | **Bump `llama-cpp-2` 0.1.143→0.1.146/147** | opportun (preflight) | Hygiène + `fit_params` multi-carte ; CVE 2026 non applicables (in-process + llguidance) ; bump optionnel non bloquant, à faire si une phase touche le backend. |

---

## §8 Traçabilité scope (chaque scope-cut/What's-NOT S75 → traitement S76)

| Item S75 "What's NOT" / scope-cut | Sprint + Phase S76 |
|---|---|
| SearchManifest (DEFER s73) | Reconduit DIFFÉRÉ post-launch (#1 S75 ; inchangé, non touché par GPU) |
| Tantivy (gate >50K docs) | Reconduit gelé (inchangé) |
| GC reaper / budget disque enforced | Reconduit post-launch (non touché ; ancre VPS S75 bornée par policy) |
| Recherche cross-nœud fédérée | Reconduit (= SearchManifest, post-launch) |
| Approbation pair pour seed distant | Reconduit (seed volontaire/invite S74, inchangé) |
| Mobile/Electron client | Reconduit (front = shell React ; panneau D1 = page React, pas nouveau client) |
| Migration wire post-tag | Reconduit (S76 = 0 bump wire ; tout additif, `*_VERSION` à 1) |
| **GPU partagé cross-machine** (décalé S76 par amendement S75) | **TRAITÉ S76 (cœur : Phases A/C/D/E/F)** |
| **Sharding pipeline** (décalé S77) | Reconduit S77 (#1 §7 ; feature distincte) |
| Kudos-threshold tuning empirique | Reconduit post-launch (D4 pose le dashboard ; calibration seuil = post-launch) |
| Multi-ancre UX avancée (priorité/fallback chains) | Reconduit post-S76 (PULL-3 failover Phase B = le résidu mécanique ; UX priorisation = post-launch) |
| Bloom/Merkle digest | Reconduit (= SearchManifest rejeté) |
| Carries S75 audit (14 P2) | Routés §6 : 11 absorbés Phase B (dont 3 anti-escalade), 5 reconduits S77, 4 externes |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | **Convergence `result:` cross-machine WAN échoue** (constat S75 `SeedAnnounced peer_count:0 ~10 min` ; chemin DOC ≠ gossip feed mais non prouvé) | Medium | High | 1er critère falsifiable Phase C (D2 adjust) : mesurer délai PC→VPS ; >timeout=BLOCK à diagnostiquer, pas timeout rallongé. Chemin DOC est le S75-prouvé pour apps. |
| R2 | **Déterminisme cross-hardware casse l'exact-match** (Ingonyama : même GGUF diverge cross-GPU) | High | Medium | D3 cohorte homogène (même digest+quant+runtime) ; résultat hétérogène-diverge écrit comme attendu (anti faux-vert T1) ; StubBackend hermétique pour le test cross-process. |
| R3 | **Bug wiring `/api/v1` vs `/consent/set`** rend le panneau inerte en prod packagée | Medium | High | Pré-requis bloquant Phase A (D1 adjust) : vérifier `vite.config.ts` + réconcilier ; critère POST consent front-packagé écrit `consent.json` ; fix(sprint76) si trou prod. |
| R4 | **Trou anti-gaming `tokens_generated`** (self-déclaré hors-quorum → kudos gonflés) | Medium | Medium | Décision PO D4-Q : durcir `amount=log_utility(median)` du groupe d'accord + sanity-bound, OU P2 documenté (log_utility compresse <10×). |
| R5 | **Phase B dette débordée** (lot duress + 3 anti-escalade + tests) sacrifie une fermeture | Medium | Medium | Priorité B1/B2/B7 (les 2-reports + duress restant) ; B9 (tests) non sacrifiable (Règle 1 : phase dette inclut tests manquants, non convertible). |
| R6 | **Escalade G7** : CARRY-3 / LOOPBACK-TIERS / PULL-3 passent 3/3 si non traités | Low (si B exécutée) | Medium | Les 3 sont des livrables Phase B avec exit condition binaire ; SYBIL-SEEDER-TAIL reconduit avec exemption « dépendance sharding » nommée. |
| R7 | **`model_digest` durci (nom→GGUF) casse des tests existants** (couche 2 vérif) | Low | Medium | D3 adjust : durcir EST P1 OU doc-note ; si durci, miroir des tests existants + capability advert testée ; sinon doc la discordance et garder le name-hash pour S76. |

---

## §10 Audit gate pattern — rappel

- **Phase 0 JOUÉE** : audit gate S75 = CONDITIONAL PASS (`73831c0`), P1
  `DURESS-BOOT-LEAK` levé par `23a08c9` (HEAD), condition de blocage Phase A levée.
- **La dernière phase (G)** produira en sortie :
  - `sprint76_verification.md` (self-report fail-fast, colonne Observed remplie ;
    Win + Docker canonique = gate AVANT push, `feedback_wsl_before_push`) ;
  - `sprint77_audit_plan.md` (plan pour Phase 0 S77) ;
  - mise à jour `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` (nouveaux
    patterns compute/quorum/quantization + tech debt) ;
  - mise à jour `THREAT_MODEL.md` (rows surface compute cross-machine + duress
    frères fermés) ;
  - `SPRINT_LOG.md` row S76 + `CLAUDE.md` état + `roadmap_v5` (Arc 3.5 6/6 clos,
    S77 sharding ouvert).

---

## §11 Checkpoint de validation (arbitrage PO AVANT plan détaillé/code)

5 questions — 1 par D-choice, dernier moment pour pivoter sans coût :

1. **D1 — sémantique d'enrôlement worker co-localisé** : OK pour que
   `OwnProjects`/`Whitelist`(least-priv) = OFF (le worker garde son
   `Whitelist[own_doc]`), `OpenSource`/`All` = le worker lit le `consent.json`
   utilisateur, et `All` = opt-in **double-confirmé** ? Ou préfères-tu un autre
   découpage (ex. `OpenSource` reste least-priv, seul `All` ouvre) ? *(Le bug
   `/api/v1` est de toute façon un pré-requis bloquant, pas une option.)*

2. **D2 — convergence `result:` WAN comme 1er critère falsifiable** : OK pour que
   l'acceptance B-3 ouvre par la mesure du délai de réplication `result:` PC→VPS,
   et qu'un délai > timeout du gate (30s) soit un **BLOCK à diagnostiquer** (pas un
   timeout à rallonger), en référence au constat S75 `SeedAnnounced peer_count:0` ?

3. **D3 — `model_digest` fichier-GGUF vs nom + frontière S76/S77** : OK pour
   durcir `model_digest` de `blake3(nom)` → hash du fichier GGUF en **P1 dans la
   phase compute** (le champ existe déjà `task.rs:374`), et garder TOPLOC
   (`logprobs_hash` étage 2) en design note pour post-S77 ? Ou préfères-tu une
   doc-note S76 (garder le name-hash) et durcir au S77 ?

4. **D4 — kudos `tokens_generated` : durcir vs documenter** : OK pour trancher
   **maintenant** entre (a) durcir `amount=log_utility(median(tokens_generated))`
   du groupe d'accord + sanity-bound (modifie la signature de `credit()`), ou (b)
   accepter le trou comme **P2 documenté** (la `log_utility` compresse déjà
   l'incitatif <10×) ? Quel niveau de durcissement veux-tu sur ce sprint ?

5. **D5 — doc-only vs câbler le tensor-split mono-machine** : OK pour que S76 reste
   **doc-only** (table empreintes + reco format + caps VRAM existants, cible
   single-card ≤14B, 70B=S77), et que le câblage tensor-split multi-GPU
   (`with_split_mode`+`with_devices`, binding dispo) soit S77 ? Ou veux-tu câbler
   le mono-machine 2-GPU dès S76 si le delta est trivial au preflight ?

### Résolution PO (Checkpoint joué — `2026-06-15`)

Décisions tranchées par le PO, gelées dans §4 :
- **D1** = `OpenSource`+`All` ouvrent le partage (least-priv `OwnProjects`/
  `Whitelist` = OFF) ; le bug `/api/v1` est un pré-requis bloquant Phase A.
- **D2** = convergence `result:` WAN = 1er critère falsifiable (confirmé).
- **D3** = durcir `model_digest` (nom→fichier GGUF) en **P1 dans la phase compute**
  (champ existant) ; TOPLOC (`logprobs_hash`) en design note post-S77.
- **D4** = **durcir maintenant** : `amount=log_utility(median(tokens_generated))`
  du groupe d'accord quorum + sanity-bound (Phase E modifie `credit()`).
- **D5** = **doc-only ; mono-machine 2-GPU ENTERRÉ** — arbitrage PO littéral :
  « l'objectif c'est 2 GPU de 2 machines, personne n'a 2 GPU ». La cible S76 =
  modèle ENTIER single-GPU ≤14B prouvé cross-machine (task-routing) ; le chemin
  gros-modèle = **sharding cross-machine 2 machines × 1 GPU = S77** (pas le
  tensor-split mono-machine, qui n'a pas de cible). Séquencement roadmap respecté :
  prouver le compute cross-machine AVANT de l'éclater.

---

*Audit gate pattern rappel : S76 produira en sortie `sprint76_verification.md` +
`sprint77_audit_plan.md` (Phase G). Phase 0 déjà jouée (CONDITIONAL PASS, P1
levé). Arc 3.5 se referme à ce sprint ; S77 = sharding pipeline, feature
distincte.*
