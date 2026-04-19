---
sprint: 21
topic: p2p_compute_scoring_systems_matthew_effect
date: 2026-04-19
agent: general-purpose (WebSearch + WebFetch)
prompt_source: transcript session orchestrateur 2026-04-19 user turn "cherche des projet open source, documente toi avec context7, recherche deep"
word_count: 2400
---

## Prompt donne a l'agent

Recherche deep-dive : comment les projets de calcul distribué / P2P compute open source notent-ils les contributeurs ? Question centrale : **est-ce que leur formule de scoring amplifie l'écart entre matériel haut de gamme (RTX 5090, datacenter) et bas de gamme (Raspberry Pi, laptop 5 ans) ?** Si oui, quels mitigations ont-ils mis en place ?

Contexte du projet appelant : nexus-grid (réseau P2P pour distribuer des tâches LLM / compute sur des workers hétérogènes). La formule kudos actuelle est linéaire : `kudos = tokens_generated × quality × trust`. Un RTX 5090 à 200 t/s écrase un Pi à 5 t/s d'un facteur ~40×, et l'effet s'aggrave (plus de kudos → plus de tâches routées → plus de kudos = Matthew effect). On cherche des précédents concrets qui ont combattu ou non ce problème.

Projets à investiguer (utilise WebSearch et WebFetch systématiquement, sources primaires de préférence : docs officielles, whitepapers, code github) :

1. **BOINC** — système de crédit Cobblestone, Recent Average Credit (RAC). Formule exacte ? Comment gère-t-il la différence GPU/CPU ? Y a-t-il une normalisation par benchmark ?
2. **Folding@home** — Points + QRB (Quick Return Bonus). Comment la QRB est-elle calculée ? Avantage-t-elle les gros GPU ?
3. **SETI@home** — historique, credit system
4. **Bittensor** — Yuma consensus, TAO rewards, validator/miner incentive. Comment le scoring pondère-t-il les mineurs de différents tiers ? Subnet 1 (text-gen) est crucial pour nous.
5. **Gensyn** — proof-of-learning, verified ML compute. Structure de rétribution ?
6. **Golem Network** — task pricing, provider reputation
7. **Akash Network** — decentralized GPU leasing, bidding
8. **io.net** — DePIN GPU, clustering
9. **Render Network** — rendering jobs + RNDR tokens
10. **Petals** — distributed inference Bloom/Llama, no token rewards mais scoring interne ?
11. **Hivemind** — Yandex/HuggingFace distributed training
12. **Together Compute** ou **Flower** (federated learning) si scoring public
13. **Ethereum staking** (optionnel, comparaison proof-of-stake vs contribution)

Pour chaque projet pertinent, réponds :
- Formule de scoring exacte (si documentée)
- Est-ce linéaire / log / quadratique / tier-segmenté ?
- Mécanisme anti-concentration éventuel (cap, tier, sybil-resistance)
- Évidence empirique que l'écart se creuse ou non (études, blog posts, github issues)
- URL source

**Livrable** : rapport markdown structuré (≤ 2000 mots) avec sections par projet + section finale "patterns récurrents" + "projets qui ont explicitement lutté contre l'écart rich/poor" + "projets qui l'assument".

Priorité : sources primaires (docs.bittensor.org, github readmes officiels, whitepapers PDF) plutôt que articles de blog crypto marketing. Recoupe quand tu peux. Cite tout avec URL.

Si tu trouves un projet qui documente explicitement le problème Matthew effect ou wealth gap dans le scoring compute, c'est GOLD — creuse.

## Rapport recu

# Rapport deep-dive : scoring des contributeurs dans les réseaux P2P compute

**Contexte** : revue de 13 projets. Question centrale = "est-ce que leur formule de scoring amplifie l'écart matériel haut/bas de gamme (Matthew effect) ?" Pertinent pour la formule kudos actuelle de nexus-grid (`tokens_generated × quality × trust`, linéaire).

---

## 1. BOINC (Cobblestone + CreditNew + RAC)

**Formule exacte** (source : [CreditNew wiki](https://github.com/BOINC/boinc/wiki/CreditNew)) :
```
PFC(J) = T * peak_flops(J)          // "peak FLOPS count"
C = F * cobblestone_scale           // cobblestone_scale = 200/86400e9
```
Recent Average Credit (RAC) = moyenne glissante exponentielle, demi-vie 1 semaine (α ≈ 0.1/jour).

**Linéaire par construction** : proportionnel à `elapsed_time × peak_flops`. Pas de log, pas de tier.

**Mécanismes anti-concentration documentés** :
- **Version normalization** : si une app a des versions CPU + GPU, BOINC prend le `min` des moyennes d'efficience → pénalise la version la plus efficace pour équilibrer. C'est un shift explicite vers "Actual FLOPs" plutôt que "Peak FLOPs".
- **Host scaling factor capped à 10** : `host_scaling_factor = min(x, 10)` dans CreditNew empêche un host de réclamer >10× le crédit moyen.
- **Cross-project GPU discount** : `S(V) = moyenne_scaling_factor_GPU_across_projects` pour éviter qu'un projet GPU-only devienne "plus généreux" que les autres.
- **Sanity cap** : `PFC(J) > wu.fpops_bound` → job annulé, crédit par défaut.

**Évidence du problème** (sources primaires) :
- BOINC wiki admet : *"GPUs typically have a higher (10–100X) peak FLOPS than CPUs. However, application efficiency is typically lower (very roughly, 10% for GPUs, 50% for CPUs)"* → ratio net ≈ 2–20×.
- [Einstein@Home forum](https://einsteinathome.org/content/why-are-cpu-task-scores-so-low-compared-gpus-boinc) : *"For the same job, a GPU can complete it in 5 minutes while a CPU core needs 1–2 hours"* → ratio empirique ≈ 12–24× sur Einstein@Home.
- [Scottish BOINC Team forum](https://tsbt.co.uk/forum/viewtopic.php?t=2448) : *"Large contributors have managed to swamp the credit system and made the smaller contributors seem… irrelevant. The community does not want to end up like Bitcoin."* → reconnaissance explicite du Matthew effect.
- Sur le ranking global : *"Users and teams commonly determine world rank by comparing total credits accumulated. This highly favors users and teams that have been around the longest."* → effet cumulatif. RAC a été introduit partiellement comme mitigation mais ne change pas le routage.

**Verdict** : linéaire, non-tier, CreditNew *"is basically flawed"* selon la communauté. Einstein@Home et d'autres projets ont **divorcé de CreditNew** pour passer à un crédit fixe par type de tâche ([Einstein@Home credits](https://einsteinathome.org/content/einsteinhome-credits)). C'est le pattern "opt-out du scoring propto FLOPS" que nexus-grid pourrait étudier.

---

## 2. Folding@home (QRB)

**Formule exacte** ([points FAQ](https://foldingathome.org/faqs/points/bonus-points/how-is-the-qrb-determined/)) :
```
base_points   = scaling_factor * time_on_i5_benchmark
final_points  = base_points * max(1, sqrt(k * deadline_length / elapsed_time))
```
où `k ≈ 0.75` par projet, `deadline_length` / `elapsed_time` mesurés en jours.

**Régime** : **super-linéaire, racine carrée** sur le ratio deadline/time. `sqrt` atténue mais amplifie toujours : un GPU 4× plus rapide = 2× plus de QRB, mais aussi plus de base_points car `base_points ∝ time_on_i5`. **Effet cumulé = linéaire ou supra-linéaire.**

**Officiel** : *"equal pay for equal work"* — benchmark i5 standard pour GPU et CPU. Mais **pas de cap** sur le bonus (seulement un floor `max(1, …)`).

**Qualifications bloquantes** : passkey + ≥10 WU bonus-eligible + ≥80 % taux de retour + retour avant timeout. Ces qualifications bloquent **les Pi / laptops lents** dont le taux de retour dans les deadlines est instable.

**Évidence empirique** ([lar.systems PPD database](https://folding.lar.systems/gpu_ppd/overall_ranks)) :
- RTX 4090 : ~26M PPD/jour
- Core i5 CPU seul : ~20–50k PPD/jour
- **Ratio top-GPU vs CPU ≈ 500×–1300×**. La `sqrt` de QRB atténue à peine.

**Verdict** : QRB **amplifie** délibérément l'écart pour "aligner avec la valeur scientifique" (retour rapide = WU génération suivante plus vite). F@H l'assume : c'est une optimisation produit (science-throughput), pas d'équité.

---

## 3. SETI@home (historique, fermé 2020)

Formule initiale : `claimed_credit = (whetstone + dhrystone) × wu_cpu_time / 1728000` ([source](http://www.pperry.f2s.com/boinc-credit.htm)).

**Pure linéaire benchmark-driven**, tricherie massive documentée (optimized clients gonflaient whetstone). C'est la raison historique du pivot vers CreditNew. Pertinence : démonstration qu'un *self-reported benchmark* est game-able → **ne jamais baser kudos sur un bench client-side**.

---

## 4. Bittensor (Yuma Consensus — GOLD pour le Matthew effect)

**Formule** ([YC docs](https://docs.learnbittensor.org/learn/yuma-consensus)) :
```
R_j = Σ_i (S_i · W̄_ij)        // R_j = reward miner j
M_j = R_j / Σ_k R_k            // part d'émission miner j
```
- `S_i` = stake du validator i
- `W̄_ij` = poids post-clipping (cap κ=0.5 du stake total)
- EMA bond : `W̃_ij = (1-β) W_ij + β W̄_ij` pénalise les validators déviants

**Mécanismes anti-concentration** :
- **Clipping κ=0.5** : plafonne les poids extrêmes.
- **Bond penalty** : pénalise la déviation du consensus → anti-collusion mais **renforce la concentration** (incite à suivre les gros).
- Split fixe : 41 % miners / 41 % validators / 18 % créateur subnet.

**Évidence empirique — papier arXiv critique** ([2507.02951](https://arxiv.org/html/2507.02951v1)) :
- **Gini stake = 0.9825** (quasi-monopole parfait)
- **Top 1 % stake = 90 % médian** (range 38–99.97 %) à travers 64 subnets
- **Top 1 % rewards = 24 % médian** (range 12–57 %)
- Corrélation **stake → reward** : 0.80–0.95 pour validators, 0.50–0.80 pour miners
- Corrélation **performance → reward** : ~0.10–0.30 pour miners (faible)
- Verdict auteurs : *"rewards are overwhelmingly driven by stake, highlighting a clear misalignment between quality and compensation"*
- *"only a handful of wallets can collude to achieve a majority control in most subnets"*

**Mitigations proposées par les auteurs** (pas encore adoptées upstream) :
- Performance-weighted emission split
- **Stake cap au 88e percentile**
- Composite scoring
- Trust-bonus multiplier

**Hardware tier** : non modélisé on-chain. Mais subnet 1 (text-gen) requiert RTX 4090 / A100 / H100 → **exclusion de facto** des petits contributeurs.

**Verdict** : Bittensor est le **cas d'école** du Matthew effect chiffré. La concentration est stake-driven, pas compute-driven — mais le résultat pratique (barrière d'entrée hardware H100 + stake TAO requis) = **rich get richer confirmé empiriquement**.

---

## 5. Gensyn (proof-of-learning)

**Modèle** ([litepaper](https://docs.gensyn.ai/litepaper)) : Submitters paient, Solvers exécutent et produisent un proof-of-learning (checkpoints + métadonnées), Verifiers re-run des fragments, Whistleblowers challengent. Truebit-style staking + slashing.

**Rémunération** = prix marché négocié par tâche, **pas de scoring implicite**. Pas de formule Matthew-amplifiante — c'est un marketplace. En revanche : **stake requis pour devenir Solver** = barrière capital d'entrée. Pas de donnée empirique sur concentration (protocole pas encore au mainnet large).

---

## 6. Golem Network

**Scoring** ([reputation docs](https://docs.golem.network/docs/reputation)) : multi-dimensionnel — task success rate, uptime, CPU single/multi-core, memory bandwidth, disk I/O, network throughput. **Weights ajustables par le requestor**.

**Pricing** : `0.1 GLM/CPU·h × threads utilisés`. Marché ouvert, pas de tier imposé.

**Mitigation Matthew** : requestor peut volontairement exiger des filtres `uptime ≥ X ∧ success_rate ≥ Y` — un petit Pi avec 99 % uptime peut gagner sur un gros serveur instable. **Mais** : dans la pratique, `AgreementSelector` recommande "top performers" → biais systémique vers le haut.

**Verdict** : architecture market-based qui **n'a pas de formule universelle** — l'amplification dépend du choix requestor. Neutre.

---

## 7. Akash Network

**Mécanisme** : reverse auction pure. Client pose spec + prix max, providers bident, le moins cher gagne.

Pas de scoring global. **Concentration de fait** : Starcluster = ~7200 GB200 NVIDIA contrôlés par "Nodekeepers vetted enterprise-grade" ([Messari Q3 2025](https://messari.io/report/state-of-akash-q3-2025)) → pivot explicite vers tier datacenter. Akash **assume** la concentration haut de gamme.

---

## 8. io.net

**Formule Device Base Score** ([docs](https://io.net/docs/guides/explorer/block-rewards-page)) = combinaison de :
- Connectivity tier
- Hardware multiplier (GPU model)
- Uptime
- Job hours completed
- Bandwidth

**Multi-tier explicit par GPU model + connectivity**. Node reputation → priorité d'assignation → plus de jobs → plus de rewards = **Matthew effect hardcodé**, non caché. Proof-of-Work horaire + Proof-of-Time-Lock pour anti-triche.

---

## 9. Render Network (RNDR / RENDER) — cas le plus intéressant pour mitigation

**3-tier pricing explicit** ([Medium post officiel](https://medium.com/render-token/rndr-tokenomics-update-multi-tier-pricing-mtp-338d5dea1d29)) :
- **Tier 1 Trusted Partners** : enterprise GPU, prix ≈ AWS, SLA élevé
- **Tier 2 Priority** : 100 OctaneBench-hour (OBh) / RENDER token
- **Tier 3 Economy** : slow mais abordable

Unité = **OctaneBench (OBh)**, benchmark propriétaire OTOY qui mappe n'importe quelle combo GPU → score unique. **1 RENDER = X OBh selon tier**.

**Mitigation Matthew** : la segmentation tier isole les petits des gros — un Pi peut vivre dans Tier 3 sans jamais être écrasé par H100 de Tier 1 car **ils ne sont pas sur la même file**. Les jobs sont routés par tier choisi par le client.

**Verdict** : modèle le plus explicitement tier-segmenté. Transposable à nexus-grid.

---

## 10. Petals (Hivemind-based)

[Paper arXiv 2209.01188](https://arxiv.org/abs/2209.01188) + [GitHub](https://github.com/bigscience-workshop/petals) : **zéro scoring, zéro reward**. Bénévolat pur. Les auteurs citent explicitement *"no incentive / reward system for participants"* comme limitation ouverte. Système d'assignation par throughput et VRAM (blocks alloués à qui peut les héberger).

**Pertinence nexus-grid** : démonstration qu'un réseau inference-distribué peut fonctionner sans monnaie/scoring, en mode bénévolat + routing capacité-based.

## 11. Hivemind (Yandex/HuggingFace)

Hétérogénéité gérée au niveau **accumulation gradient locale** + averaging décentralisé. Pas de reward system public. Même philosophie que Petals.

## 12. Flower (federated learning) — recherche académique récente

État de l'art : **Shapley value** pour mesure de contribution fair. Preuve qu'il est l'unique schéma satisfaisant 4 axiomes (efficiency, symmetry, null player, additivity) ([PMC article](https://pmc.ncbi.nlm.nih.gov/articles/PMC11314990/)).

Problème : coût computationnel exponentiel. Solutions récentes : GTG-Shapley, gradient approximation, aggregation weight adjustment pour data heterogeneity. Benchmarks : Gini 0.17–0.20 sur 50–200 clients (**très inégalitaire atténué**, 5× mieux que Bittensor 0.98).

**Pertinence forte pour nexus-grid** : si on veut un système prouvablement fair, Shapley est le gold standard académique. Coût prohibitif pour online inference scoring — plutôt utilisable en offline/batch.

## 13. Ethereum staking (contrôle)

Proof-of-Stake pur : `reward ∝ stake` linéaire. Plafond 32 ETH par validator = fractionnement forcé mais les gros pools (Lido, Coinbase) contournent. Gini actuel ~0.85–0.90. Pas applicable directement au compute.

---

## Patterns récurrents identifiés

### A. Formules linéaires en FLOPS dominent → Matthew effect structurel
BOINC, Folding@home, io.net, Akash (via market), Bittensor (via stake proxy de capital). **Tous amplifient** le hardware gap.

### B. Caps / clipping introduits post-hoc
- BOINC : `min(10, host_scaling)`, sanity `PFC > fpops_bound`
- Bittensor : clip κ=0.5 stake
- Ethereum : plafond 32 ETH

Mesure **après coup**, pas by-design.

### C. RAC / moving average = mitigation cumulative mais pas du routing
RAC permet à un newcomer de monter rapidement en "recent rank" mais ne change pas qui obtient les tâches. Le problème nexus-grid (plus de kudos → plus de tâches routées) n'est **pas** résolu par RAC seul.

### D. Fixed-credit-per-task (Einstein@Home)
Divorcer du "credit ∝ FLOPS actual" → chaque tâche vaut N crédits, le Pi qui finit touche les mêmes N qu'un 4090 pour le même travail logique. Inefficace pour débit, **équitable par construction**.

### E. Tier segmentation (Render)
Séparer les files de tâches par classe de matériel. Petits et gros ne se concurrencent **jamais** directement.

### F. Shapley-based scoring (federated learning research)
Prouvablement fair axiomatiquement. Coûteux mais applicable batch/offline.

---

## Projets qui ont **explicitement lutté** contre l'écart

1. **Einstein@Home** : abandon de CreditNew → crédit fixe par task type. [Raison documentée](https://einsteinathome.org/content/einsteinhome-credits) : *"BOINC's CreditNew is basically flawed… we award fixed amounts depending on application and task type."*
2. **Render Network** : 3-tier segmentation explicite. Les tiers ne se concurrencent pas.
3. **BOINC** (demi-mesure) : host scaling cap, version normalization, cross-project GPU discount. Échec reconnu par la communauté.
4. **Papier Bittensor arXiv** : proposent stake cap 88e percentile + performance-weighted split (non adopté).

## Projets qui **assument** le gap

1. **Folding@home** : QRB `sqrt(deadline/time)` **amplifie délibérément** pour science-throughput.
2. **Bittensor** : stake = influence, by design. Gini 0.98 documenté et non corrigé.
3. **Akash** : pivot explicite vers Nodekeepers enterprise-tier (Starcluster 7200 GB200).
4. **io.net** : hardware multiplier + reputation → Matthew hardcodé.

---

## Recommandations concrètes pour nexus-grid

Pour la formule `kudos = tokens × quality × trust` actuelle :

1. **Briser la boucle routage↔kudos** : le routeur (dispatcher) doit utiliser **capacité courante** (VRAM, t/s claimed, availability) et **non** le kudos cumulé. Sinon tu reproduis Bittensor.
2. **Tier segmentation à la Render** : classes "edge", "consumer-GPU", "datacenter" avec pools de tâches séparés selon le `min_tier` de l'app. Un Pi ne concourt jamais contre un 4090. Transposable directement à ton architecture workers hétérogènes.
3. **Fixed-kudos-per-completed-task à la Einstein@Home** comme alternative au `tokens × quality` : chaque task-class vaut N kudos constants. Élimine le biais hardware. Perd l'incitation débit.
4. **Cap à la BOINC** : `kudos_host ≤ 10 × median(kudos_host)` empêche un worker de monopoliser.
5. **Shapley batch offline** : pour audits de fairness périodiques, recalcule Shapley sur les N dernières tâches et ajuste. Gini < 0.30 comme target (benchmark FL research).
6. **Anti-barrière d'entrée** : pas de stake minimum (contrairement à Bittensor/Gensyn). Le pattern "kudos = compute pur" est déjà bon — ne le contamine pas avec du capital.

Le risque spécifique à nexus-grid avec la formule actuelle = **courbe linéaire tokens × concentration routage** → exactement la trajectoire Bittensor (Gini 0.98 en 3 ans). Le choix tier-segmenté Render + fixed-credit Einstein@Home est le plus produit-cohérent avec la vision "F-Droid des apps P2P" (inclusif par construction, pas que haut de gamme).

---

## Sources

**BOINC / Einstein@Home / SETI@home :**
- [BOINC CreditNew wiki](https://github.com/BOINC/boinc/wiki/CreditNew)
- [BOINC Computation credit](https://boinc.berkeley.edu/wiki/Computation_credit)
- [BOINC Credit System Wikipedia](https://en.wikipedia.org/wiki/BOINC_Credit_System)
- [Einstein@Home Understanding credits](https://einsteinathome.org/content/understanding-credits)
- [Einstein@Home Why CPU scores are low](https://einsteinathome.org/content/why-are-cpu-task-scores-so-low-compared-gpus-boinc)
- [Scottish BOINC Team — credit controversy](https://tsbt.co.uk/forum/viewtopic.php?t=2448&view=next)
- [BOINC incentive paper (Anderson)](https://boinc.berkeley.edu/boinc_papers/credit/text.php)
- [SETI-Linux credit formula](http://www.pperry.f2s.com/boinc-credit.htm)

**Folding@home :**
- [Points FAQ](https://foldingathome.org/faq/points/)
- [QRB formula](https://foldingathome.org/faqs/points/bonus-points/how-is-the-qrb-determined/)
- [GPU QRB update 2013](https://foldingathome.org/2013/03/14/gpu-qrb-update/)
- [lar.systems GPU PPD database](https://folding.lar.systems/gpu_ppd/overall_ranks)

**Bittensor :**
- [Yuma Consensus docs](https://docs.learnbittensor.org/learn/yuma-consensus)
- [Incentive Mechanisms](https://docs.learnbittensor.org/learn/anatomy-of-incentive-mechanism)
- [Bittensor Critical Empirical Analysis arXiv 2507.02951](https://arxiv.org/html/2507.02951v1)
- [Mining in Bittensor (hardware reqs)](https://docs.learnbittensor.org/miners)

**Autres projets :**
- [Gensyn Litepaper](https://docs.gensyn.ai/litepaper)
- [Gensyn Verifiable Compute Whitepaper v1.08 Dec 2024](https://a.storyblok.com/f/257174/x/7b61f19b05/verifiable-compute-white-paper-december-2024.pdf)
- [Golem reputation](https://docs.golem.network/docs/reputation)
- [Golem pricing](https://stats.golem.network/network/provider/pricing)
- [Akash Messari Q3 2025](https://messari.io/report/state-of-akash-q3-2025)
- [io.net block rewards](https://io.net/docs/guides/explorer/block-rewards-page)
- [Render Network Multi-Tier Pricing](https://medium.com/render-token/rndr-tokenomics-update-multi-tier-pricing-mtp-338d5dea1d29)
- [Render Network pricing kb](https://know.rendernetwork.com/basics/how-much-does-rndr-cost)
- [Petals arXiv 2209.01188](https://arxiv.org/abs/2209.01188)
- [Petals GitHub](https://github.com/bigscience-workshop/petals)
- [Hivemind GitHub](https://github.com/learning-at-home/hivemind)

**Federated learning fairness :**
- [A Fair Contribution Measurement Method for FL (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11314990/)
- [Shapley-Driven Federated Learning arXiv 2406.00569](https://arxiv.org/html/2406.00569v1)
- [Contribution Evaluation in FL (NeurIPS 2021 workshop)](https://neurips2021workshopfl.github.io/NFFL-2021/papers/2021/Passerat-Palmbach2021.pdf)
- [On the Fragility of Contribution Score Computation arXiv 2509.19921](https://arxiv.org/html/2509.19921)

## Decision downstream

Cette recherche est la source factuelle primaire qui alimente `docs/FAIRNESS_VISION.md` (vision produit reforme kudos) + ligne `docs/security/HARDENING_ROADMAP.md §Sprint 22` actuellement marquee "Kudos-weighted gossip admission" — a revisiter contre les findings 1. (Bittensor Gini 0.98 amplification) et 5. (Render tier segmentation). Kudos v2 long-term commitment enregistre dans `docs/release/ROADMAP_COMMITMENTS.md` LT-1.
