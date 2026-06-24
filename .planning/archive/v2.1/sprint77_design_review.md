# Sprint 77 — Design Review Board (G1)

**Date** : 2026-06-20
**Sprint** : 77 — Sharding pipeline (modèle 70B éclaté cross-machine ; feature phare roadmap v5 PO-6, scope cut #1 de S76)
**Reviewer** : self-review profond (auto-challenge systématique des 5 décisions Day-0 S77)

> Contexte review : S77 est le sprint le mieux pré-armé de l'arc — le design
> du sharding est **déjà figé** dans `.planning/research/sharding_design_addendum_sota_2026-05-30.md`
> (3 semaines, <90j) + R&D antérieure `remote_user_sharded_llm_rnd.md` §10
> (wire primitives) + `docs/security/SPLIT_INFERENCE_DESIGN.md` (SI-1..SI-5).
> Le rôle du board n'est donc PAS de re-concevoir mais de challenger la
> **traçabilité au code réel**, la **fraîcheur** des sources, et surtout la
> **frontière de scope** (le risque #1 d'un sprint phare R&D = promettre trop).


---

## Arbitrage PO override (Checkpoint §11, 2026-06-20)

> **Ce review a été rendu AVANT l'arbitrage PO.** Le board avait recommandé le **scope
> MINIMAL** pour D3 (scheduler placement minimal + spike toy 1B/7B) et D4 (N0+N2 seulement),
> précisément parce que « le scheduler complet est un sprint à lui seul » (finding D3) et
> que la pile de vérif complète est lourde (finding D4). **Le PO a tranché le scope MAXIMAL**
> au Checkpoint §11 (directive « sprints ultra-complets, 0 defer du cœur ») : D3 = 70B complet
> + scheduler Parallax 2-phases + benchmark 3-5 machines ; D4 = N0+N1+N2+N3 + incentive
> curator-reputation (N4 zkML reste post-S77).
>
> **Conséquence sur les ⚠️** : les ⚠️ D3 et D4 **NE sont PAS levés** — ils sont
> **TRANSFORMÉS en risques de TAILLE de sprint assumés par le PO**, pas en défauts de
> décision. Le board confirme que la **faisabilité TECHNIQUE** des deux est établie (design
> figé addendum SOTA : scheduler §2, vérif §3). Le risque résiduel devient :
> - **R4 (kickoff §9)** — sprint très large (11 phases A-K), risque de débordement/qualité
>   par phase. Mitigé par budget de phases ouvert + phases découplées + 1 commit atomique/phase.
> - **R8 (kickoff §9, NOUVEAU)** — l'incentive-à-vérifier N1/N3 (que D4 maximal exige) est
>   **non-garanti économiquement** (kudos non-monétaire interdit le stake ; un lazy verifier
>   rationnel ne vérifie pas). Mitigation réputationnelle (curator-reputation) + note honnête,
>   pas une garantie ; N4 zkML (garantie crypto) reste post-S77.
>
> Les findings D1 et D5 restent inchangés (D1 accepté tel quel par le PO, fork iroh inclus ;
> D5 confirmé). D2 ✅ inchangé. **La table de scoring ci-dessous reflète le review pré-override** ;
> lire les findings D3/D4 comme « risque assumé », pas « à corriger en réduisant le scope ».

---

## Scoring

| D# | Titre | Source récente (<90j) | Alternative rejetée (source) | [DETER] Crypto/spec | [DETER] Rust-first | Code vérifié | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Convergence delivery WAN d'abord (prereq dur Phase A) — fix `task:` live-sync avant tout code shard | ok (iroh 1.0 stable juin 2026 + iroh-gossip 0.96 docs.rs 2026-06-20 ; bug live S76 daté 2026-06-19) | ok (poll-vs-subscribe rejeté ; relais hot-path rejeté ; HTTP push rejeté — sourcés) | N/A | ok (iroh 0.98 pinné réutilisé ; pas de dép nouvelle) | ⚠️ | ⚠️ |
| D2 | Data plane = ALPN custom `sbfb/shard/1` sur iroh-QUIC `open_bi` long-vécu (PAS llama.cpp RPC TCP) | ok (iroh Router/ALPN context7 `/websites/rs_iroh_0_95_1_iroh` 2026-06-20 ; llama.cpp RPC README ggml-org master 2026-06-20) | ok (llama.cpp RPC TCP blind-trust rejeté ; tensor-parallel all-reduce rejeté ; blobs/gossip data-plane rejetés ; Hivemind DHT rejeté — 4 rejets sourcés) | N/A | ok (point d'insertion `node.rs:385` `extra_protocols` = miroir `SEED_ALPN` existant) | ok | ✅ |
| D3 | Pipeline/layer-split EXCLUSIVEMENT + scheduler 2-phases latency-aware (Parallax placement + Petals churn) | ok (Parallax arXiv 2509.26182 + « Privacy-Aware Split Inference over WAN » arXiv 2602.16760 <90j + Trust-Aware Routing arXiv 2603.28622 <90j) | ok (tensor-parallel rejeté avec finding empirique ; Parallax DHT-expire churn rejeté ; geoIP central rejeté ; endpoint-federation no-shard rejeté — sourcés) | N/A | ok (`GpuInfo.vram_free_bytes` `gpu/mod.rs:107` mesuré réel, water-filling faisable Rust) | ⚠️ | ⚠️ |
| D4 | Vérification shard N0-N2 graduée — N0 TOPLOC fingerprint systématique (remplace Layer3 logprob), N1 spot-check VRF, N2 redondance tolérante | ok (TOPLOC arXiv 2501.16007 + DiFR arXiv 2511.20621 <90j + TensorCommitments arXiv 2602.12630 <90j + EigenAI arXiv 2602.00182 <90j) | ok (zkLLM prohibitif rejeté ; hash byte-exact cross-GPU rejeté ; Petals no-verif rejeté ; LLM-judge rejeté — 4 rejets sourcés datés) | ok (TOPLOC <6 mois alt concurrente DiFR/TensorCommitments <90j ; `verification.rs` Layer1/2/3 existant à étendre) | ok (TOPLOC exige hidden state → backend `llm_llama_cpp` `llama_cpp.rs` ; Ollama HTTP ne l'expose pas, gap factuel documenté) | ⚠️ | ⚠️ |
| D5 | Mode groupe privé explicite (allowlist Ed25519, zéro worker anonyme) — pilote fermé, R-iroh-audit P0 | ok (pin invite M19 réutilisé ; R-iroh-audit P0 inchangé) | ok (mode public par défaut rejeté = R-iroh-audit ; PoW seul rejeté = pas anti-collusion ; TEE GPU rejeté = inexistant consumer 2026 — sourcés) | N/A | ok (`invite.rs` M19 paire-liée réutilisé ; `node_directory` allowlist) | ✅ | ⚠️ |

**Résumé** : D1 ⚠️, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ⚠️ — Rigor signal G4 satisfait (4 ⚠️ sur 5).

Le gold standard est 1-2 ⚠️ ; ici **4 ⚠️ sur 5** — au-dessus de la zone gold,
ce qui est cohérent avec un sprint phare R&D où la frontière de scope est le
risque principal. Aucune des 5 décisions n'est invalidée — toutes restent les
bons choix architecturaux (et la plupart sont déjà figées par l'addendum SOTA).
Les ⚠️ portent sur : (D1) la convergence WAN est présentée comme « fix » alors
qu'elle est un **prérequis bloquant non encore root-causé** — sa difficulté réelle
est inconnue, c'est le vrai risque du sprint ; (D3) sources scheduler récentes
mais le **scheduler complet Parallax 2-phases est trop pour un seul sprint** —
la frontière S77 vs post-S77 doit être tranchée ; (D4) le N0 TOPLOC dépend du
backend `llm_llama_cpp` qui n'a **jamais tourné en CI** (`#[ignore]`-gated, pas de
GGUF) — la faisabilité d'implémentation est conditionnée à un fait non vérifié ;
(D5) le mode groupe privé est correct mais le board doit signaler que **rien dans
le code actuel ne contraint un worker shard à être dans une allowlist** — c'est un
verrou à construire, pas un acquis.

---

## Findings

### D1 ⚠️ — La convergence delivery WAN est présentée comme « fix Phase A » alors qu'elle est un prérequis bloquant NON root-causé : sa difficulté réelle est inconnue et c'est le vrai risque du sprint

**Detail** :

D1 est solide sur le rejet des alternatives (poll-vs-subscribe, relais hot-path,
HTTP push — sourcés) et sur le Rust-first (iroh 0.98 réutilisé, 0 dép). Le ⚠️
porte sur l'item « code vérifié / changement faisable » et sur l'honnêteté du
framing :

1. **Le bug est observé mais pas root-causé.** La mémoire + `sprint76_verification.md`
   §5.1 documentent le symptôme avec précision : `dispatch_loop.rs` écrit `task:{id}`
   via `doc.set(author, ...)`, mais l'entrée incrémentale écrite **après** que le
   worker distant a souscrit n'atteint jamais sa réplique (`recv:0`, gossip
   neighborhood non formé) — reproduit en **LAN aussi**, pas seulement WAN. Le sync
   **initial bulk** livre les vieilles entrées ; les nouvelles ne se propagent pas
   en live. Mais **la cause exacte n'est pas établie** : est-ce (a) le gossip
   neighborhood iroh-docs qui ne se forme pas faute de peer-discovery actif sur ce
   doc, (b) un `set_download_policy` manquant, (c) un problème d'auteur/permission
   d'écriture cross-replica, ou (d) un comportement attendu d'iroh-docs nécessitant
   un re-subscribe/re-sync explicite ? Présenter ça comme un « fix » à appliquer
   sous-estime que la **première tâche est un diagnostic**, pas une correction connue.

2. **C'est le chemin critique de tout le sprint.** Le sharding exige que des
   sous-tâches de shard se propagent à N workers distants en live (pas seulement au
   boot). Si la convergence WAN ne se résout pas, **aucune ligne de code shard ne
   peut être validée cross-machine**. C'est donc le vrai risque produit du sprint,
   pas un détail d'infra.

**Decision** : adjust. Au kickoff §4 D1 :
- (a) **Reformuler en « diagnostic-puis-fix »** : Phase A ouvre par un test
  d'intégration Rust 2-nœuds (le germe documenté `process_evolution_commit2_handoff.md`
  l.58-60) qui **reproduit** le bug en hermétique (vrai discovery iroh, entrée `task:`
  incrémentale post-subscribe), AVANT toute tentative de fix. Le test rouge-d'abord
  est le livrable de diagnostic.
- (b) **Inscrire un fork de décision explicite** : si la cause est dans le câblage
  SBFB (download policy, re-subscribe, author) → `fix(sprint77)` dans la phase ; si
  la cause est un comportement intrinsèque iroh-docs 0.98 nécessitant une primitive
  absente → c'est un **BLOCK à arbitrer** (upgrade iroh 1.0 hors-Day-0 = décision PO,
  ou re-architecture du delivery). Ne PAS masquer ce fork.
- (c) **Critère d'acceptance falsifiable** : le test convergence Rust passe vert
  (entrée incrémentale propagée < budget) ET l'artefact `b3` cross-machine atteint
  au minimum le stage `claim` (le worker distant voit la tâche live). Sans ça, tout
  le sharding reste PROVISIONAL.

---

### D3 ⚠️ — Sources scheduler récentes, mais le scheduler 2-phases Parallax complet est trop pour un seul sprint : la frontière S77 vs post-S77 doit être tranchée explicitement

**Detail** :

D3 est rigoureux sur le fond — le finding empirique « performance degrades with
higher latency, not bandwidth » (Petals/Parallax, confirmé WebSearch 2026-06-20 :
Petals 1.71 steps/s à <5ms, dégrade avec la latence) verrouille correctement le
choix pipeline-parallel exclusif vs tensor-parallel. Les sources sont fraîches
(Parallax 2509.26182 ; « Privacy-Aware Split Inference over WAN » arXiv 2602.16760
et « Trust-Aware Routing at the Edge » arXiv 2603.28622, toutes deux <90j). Le ⚠️
porte sur la **faisabilité d'implémentation dans le périmètre d'un sprint** :

1. **Le scheduler Parallax 2-phases complet (placement DP water-filling + routing
   DAG layer-indexé + k-medoids sur matrice RTT pairwise + churn re-balancing Petals)
   est un sprint à lui seul.** L'addendum §2 décrit un algorithme from-scratch
   substantiel. Le tenter en entier au même sprint que la convergence WAN (D1, déjà
   risquée) + le data plane (D2) + la vérification (D4) garantit soit un sprint qui
   déborde, soit un scheduler bâclé. L'item « code vérifié » est ⚠️ car la faisabilité
   du scheduler complet n'est pas établie pour un seul sprint.

2. **Le spike toy (1B/7B sur 2-3 machines) est le vrai livrable falsifiable de S77**,
   pas le scheduler 70B optimal. L'addendum §7 phase B le dit : « spike toy (1B/7B
   sur 2-3 machines, ALPN, open_bi long-vécu, RunProof, TOPLOC N0) ». Le board doit
   promouvoir ce phasage au rang de décision Day-0, pas le laisser implicite.

**Decision** : adjust. Au kickoff §4 D3 :
- (a) **Trancher la frontière** : S77 livre (i) le **scheduler de placement MINIMAL**
  (water-filling VRAM sur 2-3 shards via `GpuInfo.vram_free_bytes` mesuré, RTT
  pairwise simple, PAS k-medoids ni routing DAG optimal), suffisant pour un pipeline
  fixe 2-3 hops ; (ii) le **spike toy 1B/7B** comme preuve falsifiable. Le scheduler
  Parallax 2-phases complet (k-medoids, routing DP, churn actif) → **post-S77**
  (scope cut explicite, déclenché par les mesures réelles du spike).
- (b) **Gate réseau honnête** : inscrire au critère d'acceptance le NO-GO de
  l'addendum §6 (RTT/frontière > 80ms ou relais hot-path = sharding refusé, pas
  dégradé silencieusement) — vérifiable via `conn.stats()` iroh.
- (c) Conserver le verdict produit « 1-3 tok/s batch/async, jamais chat live » comme
  attente écrite (anti faux-vert : ne pas mesurer le succès en latence interactive).

> **OVERRIDE PO 2026-06-20** : le PO a rejeté l'option (a) frontière minimale et tranché le **scope maximal** (scheduler Parallax COMPLET + benchmark 70B 3-5 machines = livrables S77). Le ⚠️ D3 devient le risque R4 (taille de sprint) assumé. (b) gate réseau NO-GO et (c) verdict perf restent appliqués.

---

### D4 ⚠️ — Le N0 TOPLOC dépend du backend `llm_llama_cpp` qui n'a JAMAIS tourné en CI (`#[ignore]`-gated, pas de GGUF) : la faisabilité d'implémentation est conditionnée à un fait non vérifié

**Detail** :

D4 est le bloc le plus solide sur la fraîcheur crypto — TOPLOC (2501.16007) +
DiFR (2511.20621, <90j) + TensorCommitments (2602.12630, <90j) + EigenAI
(2602.00182, <90j) sont toutes citables et l'alternative concurrente <6 mois est
satisfaite ([DETER] crypto ok). Les rejets (zkLLM prohibitif, hash byte-exact
cross-GPU, no-verif Petals, LLM-judge) sont sourcés et datés. Le ⚠️ porte sur
l'item « code vérifié » :

1. **TOPLOC exige le dernier hidden state / top-k logits.** Confirmé par l'addendum
   §3 et le code : le backend **Ollama (HTTP localhost:11434) ne l'expose pas** sans
   fork (interdit). Seul le backend **`llm_llama_cpp`** (`llama_cpp.rs`, `llama-cpp-2`
   in-process) peut extraire les logits. MAIS — vérifié dans le code — ce backend est
   **feature-gated `llm_llama_cpp`, `#[ignore]`-gated pour les tests E2E (pas de GGUF
   sur disque), et n'a JAMAIS tourné en CI** (`llama_cpp.rs:41-52` : « CI runs without
   the feature and never touches this module »). Le N0 TOPLOC dépend donc d'un chemin
   de code non exercé en CI. C'est un risque de faisabilité réel, pas un détail.

2. **`verification.rs` Layer3 (`logprobs`) est le slot existant, mais il est inerte.**
   Le `logprobs_hash` est `[u8;32]` doc-commenté « layer 3 » mais l'implémentation ne
   le calcule pas vraiment (32 zéros). TOPLOC n'est pas « brancher un champ existant »
   — c'est implémenter l'extraction top-k + l'encodage polynomial + le seuil de
   comparaison, le tout sur le backend feature-gated.

**Decision** : adjust. Au kickoff §4 D4 :
- (a) **Marquer la dépendance backend comme prérequis explicite** : le N0 TOPLOC
  S77 est livré **sur le backend `llm_llama_cpp` uniquement**, avec un test
  d'intégration `#[ignore]`-gated (GGUF requis) + un test **hermétique** sur la
  primitive TOPLOC pure (encodage/comparaison de fingerprints sur des activations
  fixtures, sans GGUF) qui TOURNE en CI. Le mode sharding **impose** `llm_llama_cpp`
  (Ollama dégradé à signature+redondance N2 seulement) — décision, pas option.
- (b) **Graduer le scope vérif** : S77 livre N0 (TOPLOC fingerprint) + N2 (redondance
  tolérante, réutilise le quorum existant). N1 (spot-check VRF) et N3 (bissection
  opML) → **post-S77** (scope cut). Rationale : N0+N2 couvrent le swap modèle/précision
  + la divergence ; N1/N3 sont des durcissements anti-lazy-verifier dont l'incentive
  reste non résolu (addendum §8 Q1 — kudos non-monétaire interdit le stake).
- (c) Écrire le résultat attendu honnête : le fingerprint TOPLOC **détecte** le swap
  modèle (100% selon le papier) mais **ne prouve pas** le calcul correct des tokens
  sans N1/N3 — anti-surpromesse.

> **OVERRIDE PO 2026-06-20** : le PO a rejeté l'option (b) graduation N0+N2 et tranché le **scope maximal** (N0+N1+N2+N3 + incentive curator-reputation = livrables S77 ; N4 zkML reste post-S77). Le ⚠️ D4 devient le risque R8 (incentive non-garanti économiquement) assumé. (a) backend `llm_llama_cpp` explicite + (c) anti-surpromesse restent appliqués ; l'incentive est conçu (curator-reputation réputationnel, pas stake).

---

### D5 ⚠️ — Le mode groupe privé est le bon choix, mais rien dans le code actuel ne contraint un worker shard à être dans une allowlist : c'est un verrou à construire, pas un acquis

**Detail** :

D5 est correct sur le principe (invariant non-négociable de l'addendum §1 :
allowlist Ed25519, zéro worker anonyme, R-iroh-audit P0 → pilote fermé). Les rejets
sont sourcés (mode public = R-iroh-audit ; PoW seul = pas anti-collusion ; TEE GPU
= inexistant consumer 2026, addendum §4 caveat). Le ⚠️ porte sur l'écart entre la
décision et l'état du code :

1. **Le worker compute actuel claim une tâche sur la base du consent + de la
   signature, pas d'une allowlist de groupe.** Le claim-gate (`runtime.rs:1046`
   `required_runtime`) est advisory ; le filtre consent (`should_accept_task`)
   gate sur le niveau. Il n'y a **aucun concept de « ComputeGroup allowlist » qui
   restreigne quels workers peuvent participer à un pipeline de shard**. La décision
   D5 « groupe privé » est donc un **mécanisme à construire** (le `ComputeGroup`
   primitive de l'addendum §10 n'existe pas dans le code), pas une posture déjà
   tenue par l'architecture.

2. **L'invite M19 lie une paire (project_id, archive_hash), pas une appartenance à
   un groupe de compute.** Réutiliser M19 pour l'admission au groupe shard demande
   une adaptation (le ticket d'admission au pipeline ≠ le ticket de seed). C'est
   faisable (même crypto Ed25519+JCS, même `DOMAIN_*` pattern) mais c'est du net-new.

**Decision** : adjust. Au kickoff §4 D5 :
- (a) **Inscrire le `ComputeGroup` comme livrable, pas comme posture** : S77 livre
  une primitive d'admission de groupe shard (allowlist Ed25519 des `worker_pubkey`
  autorisés, signée par l'initiateur de session, `DOMAIN_COMPUTE_GROUP_V1` sur le
  pattern M19) ; un worker non-allowlisté qui tente d'ouvrir une connexion
  `sbfb/shard/1` est rejeté au handshake ALPN (avant tout calcul d'activation).
- (b) **Verrou anti-recentralisation** : l'initiateur de session signe l'allowlist
  mais n'est PAS une autorité réseau — c'est un groupe ad-hoc privé entre pairs qui
  se connaissent (modèle pilote fermé), pas un registre central de workers.
- (c) Confirmer le caveat confidentialité (addendum §4) : aucun claim de
  confidentialité face aux workers (ils voient les activations en clair, SI-1/SI-4),
  donc pas de secret app dans les prompts — à écrire dans THREAT_MODEL §16 (nouvelle
  section sharding).

---

## Notes pour D2 (✅ — pas de finding, mais points à porter au plan)

- **D2 ✅** : bloc exemplaire. Le data plane `sbfb/shard/1` plugge sur le point
  d'insertion EXACT déjà utilisé par `SEED_ALPN` (`node.rs:385` `extra_protocols` +
  `Router::builder().accept(alpn, handler).spawn()`, confirmé context7
  `/websites/rs_iroh_0_95_1_iroh` 2026-06-20 : `RouterBuilder::accept`,
  `ProtocolHandler::accept`, `open_bi`/`accept_bi` long-lived). Le rejet de **llama.cpp
  RPC** est factuel et bien sourcé (README ggml-org master 2026-06-20 : RPC = protocole
  TCP, `rpc-server` expose les ggml devices bruts, **aucune vérification Byzantine,
  blind-trust, orienté LAN/Jetson**) — incompatible avec « peers non-confiants » +
  duplique iroh-QUIC. Le tensor-parallel (all-reduce LAN/NVLink) est rejeté avec le
  finding empirique. **À porter au plan** : vérifier `cargo doc` sur l'API
  `TransportConfig` iroh 0.98 (congestion/datagram/taille max — addendum §8 Q9) avant
  de promettre du tuning ; `conn.stats()` RTT par-connexion à valider sur liens
  résidentiels réels (addendum §4). Aucun ajustement kickoff requis.

---

**Findings résolus dans ce review** : 4 ⚠️ (D1, D3, D4, D5), tous **adjust**
(corrections inline à appliquer au kickoff §4). 0 **acknowledge**. Le board confirme
que les 5 décisions Day-0 sont les bons choix architecturaux (et figées par
l'addendum SOTA) ; les ajustements portent sur (D1) la promotion de la convergence
WAN au rang de prérequis-diagnostic non-root-causé avec fork de décision ; (D3) la
frontière scheduler minimal S77 vs Parallax complet post-S77 ; (D4) la dépendance
backend `llm_llama_cpp` non-CI + la graduation N0+N2 vs N1+N3 post-S77 ; (D5) le
`ComputeGroup` comme livrable net-new, pas comme posture acquise. **Le fil rouge des
4 ⚠️ = la frontière de scope d'un sprint phare R&D** : ne pas promettre le sharding
70B optimal complet, livrer le spike toy prouvé + l'infra (convergence + ALPN + N0
TOPLOC + groupe privé), avec la performance réelle mesurée comme gate GO/NO-GO.

---

## Checklist [DETER]

### Crypto/spec (D4 — vérification shard N0-N4 / TOPLOC)
- [x] D-choice crypto cite >=1 alternative concurrente < 6 mois (DiFR arXiv 2511.20621 + TensorCommitments arXiv 2602.12630 + EigenAI arXiv 2602.00182, toutes <90j)
- [x] Source datée < 2 ans ou revalidée (TOPLOC arXiv 2501.16007 jan 2025 ; alternatives 2025-11 → 2026-02)
- [x] Reviewer ⚠️ si alternative absente — ⚠️ posé sur le **code vérifié** (backend `llm_llama_cpp` non-CI), pas sur l'absence d'alternative

### Crypto/spec (D5 — admission groupe privé Ed25519)
- [x] Réutilise crypto existante Ed25519+JCS+`DOMAIN_*` (pattern M19/SEED, source <2 ans, revalidée S74/S75)
- [x] Alternative rejetée sourcée (TEE GPU inexistant consumer 2026, addendum §4)

### Rust-first
- [x] D1 (runtime delivery) cite iroh 0.98 Rust pinné réutilisé ; 0 dép nouvelle ; pas d'alternative non-Rust introduite
- [x] D2 (runtime transport) cite iroh-QUIC `sbfb/shard/1` ALPN Rust ; llama.cpp RPC (C++/TCP) rejeté avec gap factuel (blind-trust + duplique le transport)
- [x] D3 (runtime scheduler) cite scheduler porté from-scratch Rust ; hivemind Python/torch rejeté (écosystème en sommeil sept 2023, addendum §2)
- [x] D4 (runtime inférence/vérif) cite `llama-cpp-2` Rust binding in-process ; fork Ollama rejeté (interdit) ; gap factuel hidden-state documenté
- [x] Gap factuel documenté pour chaque alternative non-Rust rejetée
- [x] Reviewer ⚠️ si gap non documenté — aucun gap Rust-first non documenté ; les ⚠️ D1/D3/D4/D5 ne portent pas sur cet axe
- Exemptions : CI tooling, frontend UX (panneau session shard), docs (THREAT_MODEL §16), tests fixtures
