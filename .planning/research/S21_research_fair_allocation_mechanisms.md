---
sprint: 21
topic: fair_allocation_anti_whale_mechanisms
date: 2026-04-19
agent: general-purpose (WebSearch + WebFetch)
prompt_source: transcript session orchestrateur 2026-04-19 user turn "cherche des projet open source, documente toi avec context7, recherche deep"
word_count: 2500
---

## Prompt donne a l'agent

Recherche deep-dive : **mécanismes anti-concentration de richesse / anti-whale / allocation équitable** documentés dans la littérature CS / crypto / économie publique, applicables à un système de scoring pour réseau P2P compute.

Contexte : nexus-grid (réseau P2P compute LLM). La formule actuelle `kudos = tokens × quality × trust` amplifie l'écart entre matériel haut de gamme et bas de gamme. On cherche les mécanismes théoriques + implémentations open source qui aplatissent cet écart sans tuer l'incentive à contribuer.

Axes de recherche (utilise WebSearch et WebFetch, papers académiques + code open source) :

1. **Quadratic funding** (Vitalik Buterin, Glen Weyl, Zoë Hitzig — "Liberal Radicalism" 2018)
   - Formule exacte (somme des √contributions puis mise au carré)
   - Implémentation Gitcoin Grants (QF + passport sybil)
   - Applicabilité à un scoring de workers (pas juste de donateurs)
   - Papers : arxiv 1809.06421

2. **Quadratic voting** (Posner & Weyl "Radical Markets")
   - Différence avec QF, applicabilité

3. **Sybil resistance** (car QF sans anti-sybil est cassé)
   - Gitcoin Passport
   - Proof of Humanity (Kleros)
   - World ID (Worldcoin) — controversé mais technique intéressante
   - BrightID
   - Idena

4. **Fair allocation / scheduling** (littérature networking & OS)
   - Max-min fairness (Bertsekas)
   - Proportional fairness (Kelly 1997, utilisé dans 4G/5G scheduling)
   - Weighted fair queuing
   - Dominant Resource Fairness (Ghodsi et al NSDI 2011)
   - Applicabilité : au lieu d'allouer bande passante, allouer tâches/kudos

5. **Shapley value** en allocation coopérative
   - Complexité vs approximations (ex: sampling)
   - Usages en ML (contribution evaluation, federated learning)

6. **Log-utility** / diminishing returns
   - Kelly criterion
   - Log scaling en economics (utilité marginale décroissante)
   - Papers sur credit assignment avec log scaling

7. **Universal Basic Income / floor participation**
   - UBI crypto (Circles UBI, ImpactMarket)
   - GoodDollar
   - Principe du "floor" pour petits contributeurs

8. **Differential rewards in volunteer computing** (papers académiques)
   - Chercher sur arxiv / Google Scholar : "fairness volunteer computing", "credit assignment BOINC fairness", "heterogeneous reward distributed computing"

9. **Matthew effect en systèmes réputation** (science des réseaux)
   - Papers sur rich-get-richer dynamics
   - Mitigations connues

10. **Dominant Resource Fairness** — plus spécifique, très cité pour ressources hétérogènes (CPU vs GPU vs mémoire)

Pour chaque mécanisme, réponds :
- Principe en 2-3 phrases
- Formule mathématique si applicable
- Une implémentation OSS concrète (lien github/gitlab)
- Pertinence pour scoring P2P compute (1-5)
- Tradeoffs / failure modes connus
- URL sources

**Livrable** : rapport markdown (≤ 2000 mots) structuré par famille de mécanismes, avec une section finale **"combo recommandé pour nexus-grid"** — 2-3 mécanismes compatibles entre eux, qui adressent Matthew effect sans sacrifier l'incentive hardware.

Tone : technique, concret, avec formules et liens. Pas de bullshit crypto marketing. Academic & open source first.

## Rapport recu

# Mécanismes anti-concentration pour scoring P2P compute — rapport deep-dive

**Contexte** : nexus-grid utilise `kudos = tokens × quality × trust`, qui amplifie l'écart matériel. L'objectif est d'aplatir la distribution sans tuer l'incentive à contribuer du hardware haut de gamme. Ce rapport couvre 10 familles de mécanismes, avec formules, implémentations OSS, et un combo recommandé.

---

## 1. Quadratic Funding (QF) et Quadratic Voting (QV)

### 1.1 QF — Buterin, Hitzig, Weyl 2018 (arxiv 1809.06421)

**Principe** : allouer un pool de matching proportionnel au **carré de la somme des racines carrées des contributions**. Petites contributions sur-subventionnées (anti-free-riding), grosses contributions faiblement subventionnées (alignement sur biens privés).

**Formule** :
```
F_p = (Σ_i √c_i,p)²
```
où `c_i,p` = contribution du contributeur `i` au projet `p`. Le matching = `F_p − Σ c_i,p`.

**OSS** : [gitcoinco/quadratic-funding](https://github.com/gitcoinco/quadratic-funding) (legacy), [dcgtc/dgrants](https://github.com/dcgtc/dgrants) (current protocol), Allo Protocol / Grants Stack.

**Pertinence P2P compute** : **4/5**. Transposition : au lieu de `donateurs → projet`, lire `workers distincts → tâche/projet`. Un projet où 100 petits workers contribuent chacun 10 tokens score plus qu'un seul worker qui en produit 1000. Directement anti-whale.

**Failure modes** : cassé sans anti-sybil (Buterin 2020). COCM (§1.3) est la mitigation SOTA.

### 1.2 QV — Posner & Weyl

**Principe** : coût quadratique d'achat de votes. 1 vote = 1 crédit, 2 votes = 4, n votes = n². Incite à dépenser sur ce qui compte vraiment.

**Formule** : `cost(v) = v²`.

**Pertinence** : **2/5** pour le scoring (pas la même primitive : QV est allocation de préférences, pas de ressources produites). Plus pertinent pour le **curation / voting** des projets.

**Sources** : [Wikipedia QV](https://en.wikipedia.org/wiki/Quadratic_voting), [Lalley & Weyl 2015](https://www.ias.edu/sites/default/files/sss/pdfs/Rodrik/workshop%2014-15/Weyl-Quadratic_Voting.pdf).

### 1.3 COCM (Connection-Oriented Cluster Matching)

**Principe** : extension QF qui détecte des clusters de donateurs via le graphe social et pondère le match par diversité. Projets soutenus par des cercles sociaux diffus > projets soutenus par clique.

**Formule** (simplifiée) : le matching d'une paire de donateurs `(i,j)` sur projet `p` est pondéré par `(1 − s(i,j))` où `s(i,j)` est un score de similarité social (clustering sur graphe).

**OSS** : [Jmiller4/plural-qf](https://github.com/Jmiller4/plural-qf) (Python).

**Pertinence** : **4/5** — pour nexus-grid, transpose en : workers sur même AS/subnet/IP range = même cluster → matching réduit. Anti-Sybil structurel sans KYC.

**Sources** : [Gitcoin COCM](https://www.gitcoin.co/blog/leveling-the-field-how-connection-oriented-cluster-matching-strengthens-quadratic-funding).

---

## 2. Sybil resistance (prérequis critique QF)

| Protocole | Primitive | OSS | Pertinence nexus |
|---|---|---|---|
| **Proof of Humanity** (Kleros) | Vidéo + vouching + court dispute | [Proof-of-Humanity/contracts](https://github.com/Proof-Of-Humanity) | 2/5 (UX lourde, KYC soft) |
| **BrightID** | Social graph + SybilRank | [BrightID/BrightID](https://github.com/BrightID/BrightID) | 3/5 (pas de KYC, graphe) |
| **Idena** | Reverse Turing tests synchrones bi-hebdo | [idena-network/idena-go](https://github.com/idena-network/idena-go) | 3/5 (décentralisé mais ceremony-lock) |
| **World ID** (Worldcoin) | Iris scan via Orb + ZKP | [worldcoin/world-id-contracts](https://github.com/worldcoin/world-id-contracts) | 1/5 (hardware spécifique, controverse privacy) |
| **Gitcoin Passport** | Agrégation stamps (GH/Discord/ENS) + scoring | [gitcoinco/passport](https://github.com/gitcoinco/passport) | 3/5 (stamps GH déjà compatibles avec écosystème dev) |

**Principe commun** : produire un identifiant unique-par-humain pour pondérer `√c_i,p` par `is_human(i)`.

**Pertinence globale** : **3/5**. Pour nexus-grid, l'approche la plus compatible est **Passport-style multi-signal** (GH account ancien + SBFB.json Keyoxide déjà présent + BrightID optionnel). Pas de KYC, pas d'Orb.

**Sources** : [Frontiers PoP review](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2020.590171/full), [Gitcoin sybil 2024](https://gitcoin.co/research/quadratic-funding-sybil-resistance).

---

## 3. Fair allocation / scheduling (OS & networking)

### 3.1 Max-min fairness (Bertsekas & Gallager 1992)

**Principe** : maximiser l'allocation du user le plus mal loti, puis le second, etc. (leximin). Ne peut augmenter la part de A qu'en baissant celle de B ≤ A.

**OSS** : implémenté dans Linux CFS, HTB, WFQ ; [Apache YARN FairScheduler](https://github.com/apache/hadoop).

**Pertinence** : **3/5** — trop agressif si seul (les petits workers dominent), mais brique de base.

### 3.2 Proportional fairness (Kelly 1997) & α-fairness (Mo & Walrand 2000)

**Principe** : maximiser `Σ log(x_i)` sous contraintes. Concavité du log = utilité marginale décroissante = petites allocations valorisées davantage. α-fairness généralise : α=0 → utilitarisme, α=1 → proportional, α→∞ → max-min.

**Formule** :
```
max Σ_i (x_i^(1-α)) / (1-α)     pour α ≠ 1
max Σ_i log(x_i)                 pour α = 1
```

**OSS** : scheduler 4G/5G (`PF scheduler` dans open-source RAN : [srsRAN_Project](https://github.com/srsran/srsRAN_Project)).

**Pertinence** : **5/5** — **LE pivot théorique recommandé**. Remplacer `kudos ∝ tokens × q × trust` par `kudos ∝ log(1 + tokens × q × trust)` capte directement "rendement décroissant" et aplatit la queue sans la couper.

**Sources** : [Stanford lecture notes](https://web.stanford.edu/~ashishg/network-algorithms/notes/lecture10.pdf), [An Axiomatic Theory of Fairness (Princeton)](https://www.princeton.edu/~chiangm/fairness.pdf).

### 3.3 Dominant Resource Fairness (Ghodsi et al., NSDI 2011)

**Principe** : pour ressources hétérogènes (CPU, RAM, GPU, VRAM), chaque user a une **ressource dominante** (celle dont il consomme la plus grande fraction). DRF égalise les parts de ressource dominante.

**Formule** : allocation `a_i` tq `max min_i (s_i)` où `s_i = max_r (u_i,r / R_r)` (share dominante).

**Propriétés prouvées** : strategy-proof, envy-free, Pareto-efficient, sharing-incentive.

**OSS** : [apache/mesos](https://github.com/apache/mesos) (implémentation canonique), YARN Capacity Scheduler.

**Pertinence** : **5/5** — **directement applicable**. Pour nexus-grid : les ressources sont VRAM, tokens/s, context window, bande passante. Un worker RTX 5080 (gros VRAM) et un worker CPU-only (grosse RAM) voient leur ressource dominante différente → share équilibré sur la bonne dimension.

**Sources** : [NSDI'11 paper PDF](https://amplab.cs.berkeley.edu/wp-content/uploads/2011/06/Dominant-Resource-Fairness-Fair-Allocation-of-Multiple-Resource-Types.pdf).

---

## 4. Shapley value (allocation coopérative)

**Principe** : allocation équitable dans un jeu coopératif. Chaque joueur reçoit sa contribution marginale moyennée sur toutes les permutations de coalitions.

**Formule** :
```
φ_i(v) = Σ_{S ⊆ N\{i}} (|S|! · (n−|S|−1)!) / n! · [v(S ∪ {i}) − v(S)]
```

**Complexité** : exponentielle O(2^n) exacte → approximations indispensables (Monte Carlo, Owen sampling, truncated).

**OSS** :
- [akassharjun/ShapleyValueFL](https://github.com/akassharjun/ShapleyValueFL) (pip lib, federated learning)
- GTG-Shapley (ACM TIST 2022) — guided MC + truncation
- FedOwen (arXiv 2508.21261) — Owen sampling

**Pertinence** : **3/5** — théoriquement élégant mais coûteux. Utilisable offline pour **recalibrer périodiquement les poids `trust`** (ex : weekly batch Shapley sur tasks récents), pas online par tâche.

**Sources** : [ShapleyValueFL](https://github.com/akassharjun/ShapleyValueFL), [GTG-Shapley ACM](https://dl.acm.org/doi/10.1145/3501811).

---

## 5. Log-utility / Kelly / diminishing returns

Voir §3.2 (proportional fairness ≡ log-utility maximization). Kelly criterion utilise `max E[log(wealth)]` pour sizing sequential bets — même math.

**Applicabilité scoring** : transformation monotone `kudos_effective = log(1 + kudos_raw)` ou `kudos_raw^α` avec α ∈ (0,1) (sqrt-scaling si α=0.5).

**Pertinence** : **5/5** — plus simple que QF, sans anti-sybil requis (pas de matching pool), aplatit mécaniquement.

**Sources** : [Kelly criterion wiki](https://en.wikipedia.org/wiki/Kelly_criterion).

---

## 6. UBI crypto / floor participation

### Circles UBI
**Principe** : chaque user mint son propre token (CRC) à 1/heure. Trust par web of trust (A trust B → tokens échangeables). Non-accumulable par design.

**OSS** : [CirclesUBI](https://github.com/CirclesUBI) (62 repos, Gnosis Chain).

### GoodDollar
**Principe** : daily claim G$ financé par yield farming sur pool crypto. UBI externalement financé.

**OSS** : [GoodDollar/GoodProtocol](https://github.com/GoodDollar).

**Pertinence P2P compute** : **3/5** pour le **floor**. Exemple : chaque worker prouvé unique touche un kudos-floor indépendant de son hardware (proto : 1 kudos/heure online + availability proof). Évite qu'un Raspberry Pi ne touche 0 à vie.

**Sources** : [Frontiers Circles UBI 2024](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2024.1362939/full).

---

## 7. Differential rewards in volunteer computing

### BOINC CreditNew
**Principe** : credit normalized par device-benchmark, cross-project comparable. Historique d'échecs (EON, Asteroids@Home) dû au gaming des benchmarks.

**OSS** : [BOINC/boinc](https://github.com/BOINC/boinc) (CreditNew dans `sched/`).

**Leçons** :
- benchmark-based = gaming facile
- utility-based = bon mais complexité cross-projet
- normalisation inter-projet nécessaire pour éviter "race to bottom"

**Papers arxiv récents** :
- [Credit Fairness: Online Fairness in Shared Resource Pools (arxiv 2601.17944)](https://arxiv.org/abs/2601.17944) — mécanisme LendRecoup credit-fair + Pareto
- [Incentive-based VC using Blockchain (arxiv 2009.11901)](https://arxiv.org/pdf/2009.11901)
- [Proof of Team Sprint (arxiv 2503.19301)](https://arxiv.org/html/2503.19301) — distribution collaborative équitable
- [CYCle: Collaborative Fairness Decentralized Learning (arxiv 2501.12344)](https://arxiv.org/abs/2501.12344)

**Pertinence** : **4/5** — LendRecoup directement importable (workers qui "prêtent" compute en période creuse récupèrent en période chargée).

---

## 8. Matthew effect & mitigation (network science)

**Constat** : systèmes reputation suivent preferential attachment `P(new_link → i) ∝ k_i` → distribution power-law. Le top-1% accumule.

**Mitigations documentées** :
1. **Bianconi-Barabási fitness model** : `P ∝ η_i × k_i` — fitness latente casse le monopole des early movers (fit-gets-richer). Transposable : pondérer kudos par qualité intrinsèque (quality-score récent) plutôt que score cumulé.
2. **Aging** : `P ∝ k_i × (t − t_i)^{−β}` — decay temporel sur vieux kudos. Un worker qui ne contribue plus perd en ranking.
3. **Cap / log-scaling** : plafond absolu ou compression log (§3.2 et §5).

**OSS** : [networkx](https://github.com/networkx/networkx) implémente BA + BB models (samplers, pas scoring prod).

**Pertinence** : **4/5** — **aging + fitness = 2 leviers faciles et peu risqués** à ajouter à la formule existante.

**Sources** : [Royal Society Interface 2014](https://royalsocietypublishing.org/doi/10.1098/rsif.2014.0378), [Bianconi-Barabási wiki](https://en.wikipedia.org/wiki/Bianconi%E2%80%93Barab%C3%A1si_model).

---

## 9. Synthèse ranking

| # | Mécanisme | Pertinence | Implém. coût |
|---|---|---|---|
| 1 | α-fairness (log-utility) | 5/5 | Faible (1 fonction) |
| 2 | DRF (ressources multi-dim) | 5/5 | Moyen (tracker VRAM/CPU/BW) |
| 3 | QF + COCM | 4/5 | Élevé (matching pool + sybil) |
| 4 | BB fitness + aging | 4/5 | Faible |
| 5 | LendRecoup credit fairness | 4/5 | Moyen |
| 6 | Passport-style sybil multi-signal | 3/5 | Moyen |
| 7 | UBI floor | 3/5 | Faible (cron job) |
| 8 | Shapley offline batch | 3/5 | Élevé (approx MC) |
| 9 | BOINC CreditNew | 2/5 | (leçons, pas code) |
| 10 | QV / World ID | 1-2/5 | Hors-scope |

---

## 10. Combo recommandé pour nexus-grid

### Composition en 3 couches compatibles

**Couche A — Re-scaling log-utility (α-fair α=1)**
Remplacement direct de la formule actuelle :
```
kudos_raw   = tokens × quality × trust
kudos_award = K · log(1 + kudos_raw / K₀)
```
- `K₀` = bruit/seuil (ex: 1 token) évite `log(0)`
- `K` = échelle (ex: 100) normalise la distribution
- Effet : RTX 5080 qui produit 100× plus qu'un Pi 4 ne touche que ~`log(100) ≈ 4.6×` plus. Incentive hardware préservée (plus c'est rapide, plus on fait de tâches à l'heure → volume > compression unitaire), mais queue coupée.

**Pourquoi pas QF pur** : QF requiert anti-sybil robuste + matching pool externe, overkill pour v1.0. Log-scaling a les mêmes propriétés anti-concentration avec 1 ligne de code.

**Couche B — DRF pour la dimension `task assignment` (pas scoring)**
Le **dispatcher** (coord-side) assigne les tâches en respectant DRF sur `{VRAM, context_window, tokens/s}`. Un Pi 4 et un RTX 5080 ne se disputent jamais la même tâche (dominant resource différente) → pas de "prix Pareto" où le Pi 4 perd systématiquement. Cela garantit que **les petits workers reçoivent du travail**, condition nécessaire pour que la Couche A ait un sens (sans tâches, log(0) = −∞).

OSS de référence : [Mesos](https://github.com/apache/mesos) pour le code DRF canonique. Port Rust dans `nexus-coordinator` dispatcher, fichier nouveau `drf.rs`.

**Couche C — Fitness + aging (Bianconi-Barabási-inspired)**
Le `trust` actuel devient un EMA (exponential moving average) avec decay :
```
trust(t+1) = α · trust(t) + (1 − α) · quality_last_task
```
avec `α ≈ 0.95` (half-life ~14 jours à 1 tâche/jour). Évite qu'un worker qui a cumulé trust en S1 rente dessus ad vitam. Le "fitness latent" = `quality_last_window` (7 jours glissants), pondéré dans l'assignation.

### Ce que le combo NE fait PAS (assumé)
- Pas de matching pool externe (pas de tokenomics, pas de trésorerie)
- Pas de KYC / biometric sybil (incompatible threat model nexus-grid S17)
- Pas de Shapley online (trop coûteux, garder pour audit batch hebdo si besoin)
- Pas de UBI floor en v1.0 (si présent, simple `availability_kudos = 1/hour online` additif après sybil resistance light Passport-style SBFB.json Keyoxide)

### Propriétés garanties par le combo
- **Anti-whale** : log compression + DRF + aging
- **Incentive hardware préservée** : volume × log reste monotone croissant en hardware
- **Strategy-proof** (DRF) + **envy-free** (DRF) + **Pareto-efficient** (DRF)
- **Sybil-resistant soft** : pas de pool à farmer (pas de QF), donc attaquant Sybil gagne juste du log(n) pour n machines fake → faible ROI
- **Implémentation** : 3 PR indépendantes, Couche A en 1 phase (trivial), Couche B en 2-3 phases (DRF dispatcher), Couche C en 1 phase (EMA trust)

### Références papers clés à citer dans plan d'implémentation
1. Mo & Walrand, "Fair end-to-end window-based congestion control", IEEE/ACM ToN 2000 (α-fairness canonical)
2. Ghodsi et al., "Dominant Resource Fairness", NSDI 2011
3. Bianconi & Barabási, "Competition and multiscaling in evolving networks", Europhys. Lett. 2001
4. Kelly, Maulloo & Tan, "Rate control for communication networks: shadow prices, proportional fairness and stability", J. Oper. Res. Soc. 1998

---

**Sources** :
- [Buterin, Hitzig, Weyl — Liberal Radicalism (arxiv 1809.06421)](https://arxiv.org/abs/1809.06421)
- [gitcoinco/quadratic-funding (GitHub)](https://github.com/gitcoinco/quadratic-funding)
- [Gitcoin — Sybil Resistance in QF 2024](https://gitcoin.co/research/quadratic-funding-sybil-resistance)
- [Gitcoin — COCM explanation](https://www.gitcoin.co/blog/leveling-the-field-how-connection-oriented-cluster-matching-strengthens-quadratic-funding)
- [Jmiller4/plural-qf (GitHub)](https://github.com/Jmiller4/plural-qf)
- [Ghodsi et al. — DRF NSDI 2011 PDF](https://amplab.cs.berkeley.edu/wp-content/uploads/2011/06/Dominant-Resource-Fairness-Fair-Allocation-of-Multiple-Resource-Types.pdf)
- [DRF Wikipedia](https://en.wikipedia.org/wiki/Dominant_resource_fairness)
- [Proportional fairness — Stanford lecture](https://web.stanford.edu/~ashishg/network-algorithms/notes/lecture10.pdf)
- [An Axiomatic Theory of Fairness — Princeton](https://www.princeton.edu/~chiangm/fairness.pdf)
- [akassharjun/ShapleyValueFL (GitHub)](https://github.com/akassharjun/ShapleyValueFL)
- [GTG-Shapley ACM TIST](https://dl.acm.org/doi/10.1145/3501811)
- [Proof of Humanity Kleros docs](https://docs.kleros.io/products/proof-of-humanity)
- [BrightID / Idena / PoH comparative — Humanode blog](https://blog.humanode.io/comparative-analysis-of-different-proof-of-personhood-pop-protocols/)
- [World ID by World](https://world.org/world-id)
- [Circles UBI — Frontiers Blockchain 2024](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2024.1362939/full)
- [CirclesUBI GitHub](https://github.com/CirclesUBI)
- [GoodDollar — cryptoforinnovation profile](https://cryptoforinnovation.org/community-development-gooddollar/)
- [BOINC Credit System wiki](https://en.wikipedia.org/wiki/BOINC_Credit_System)
- [BOINC incentive paper — Berkeley](https://boinc.berkeley.edu/boinc_papers/credit/text.php)
- [Matthew Effect in empirical data — Royal Society Interface 2014](https://royalsocietypublishing.org/doi/10.1098/rsif.2014.0378)
- [Bianconi-Barabási model — Wikipedia](https://en.wikipedia.org/wiki/Bianconi%E2%80%93Barab%C3%A1si_model)
- [Credit Fairness in Shared Resource Pools (arxiv 2601.17944)](https://arxiv.org/abs/2601.17944)
- [Incentive-Based VC using Blockchain (arxiv 2009.11901)](https://arxiv.org/pdf/2009.11901)
- [Proof of Team Sprint (arxiv 2503.19301)](https://arxiv.org/html/2503.19301)
- [Kelly criterion — Wikipedia](https://en.wikipedia.org/wiki/Kelly_criterion)
- [Quadratic voting — Wikipedia](https://en.wikipedia.org/wiki/Quadratic_voting)
- [Lalley & Weyl — Quadratic Voting PDF](https://www.ias.edu/sites/default/files/sss/pdfs/Rodrik/workshop%2014-15/Weyl-Quadratic_Voting.pdf)
- [Max-min fairness — Wikipedia](https://en.wikipedia.org/wiki/Max-min_fairness)
- [Weighted fair queueing — Wikipedia](https://en.wikipedia.org/wiki/Weighted_fair_queueing)
- [α-proportional fairness IEEE](https://ieeexplore.ieee.org/document/5062017/)
- [Gitcoin Passport GitHub](https://github.com/gitcoinco/passport)

## Decision downstream

Cette recherche est la source factuelle primaire qui alimente `docs/FAIRNESS_VISION.md` (vision produit reforme kudos). Le combo A+B+C (log-utility + DRF + EMA fitness-aging) identifie §10 est la proposition technique retenue pour Kudos v2 (long-term commitment LT-1 dans `docs/release/ROADMAP_COMMITMENTS.md`, cible post-v1.0/v2.0). G9 factual-research-gate satisfait pour toute future D-decision S22+ touchant la formule kudos.
