# Research — Contribution family Sybil matrix (ton analyse + 4 agents findings + Option F consolidée)

Date : 2026-04-20
Auteur : FlowUP (analyse utilisateur) + synthèse nexus-phase orchestrator + 4 agents parallèles indépendants
Contexte : Sprint 22 en cours (Phase A livrée `0bc499f`). Réflexion hors-scope déclenchée post-commit Phase A sur l'asymétrie de coût de production entre les familles de contribution au kudos ledger.
Statut : **Research capture pour référence future**. Engagement long-terme formalisé via `LT-3` dans `docs/release/ROADMAP_COMMITMENTS.md`. Zéro code pré-v1.0, zéro amendement workflow pré-v1.0. Implémentation conditionnelle post-Gate-3 (S30+) sur triggers empiriques.

---

## 1. Problem statement original (user)

Le système kudos SBFB actuel (`packages/nexus-coordinator/src/nexus_coordinator/kudos.py:52-68`) mesure **uniquement la contribution compute** (`kudos = tokens × quality_factor × trust_multiplier`). La Sybil defense naturelle du compute repose sur le **coût physique** : GPU-seconds + électricité. Un bot qui génère 1000 tâches a brûlé 1000 tâches d'électricité réelle.

Le problème exposé : si on étend le ledger kudos à **8 familles** (compute, funding, code, docs, review, moderation, design, accessibility), le coût réel par kudos devient **radicalement inégal** :

| Famille | Coût réel par kudos | Sybil defense naturelle |
|---|---|---|
| Compute | GPU + électricité + temps | **Forte** (coût physique) |
| Funding | Argent réel transféré | **Forte** (coût monétaire) |
| Accessibility | Test réel + handicap + outils spécifiques | **Modérée** (coût d'expertise) |
| Code | 1 PR bien écrite par LLM agent | Faible (marginal ~$0.10) |
| Docs | 200 pages/heure LLM à 200 pages/heure | **Quasi-nulle** (pire cas) |
| Review | LLM fine-tuné sur le projet | Faible |
| Moderation | LLM avec contexte social | Faible |
| Design | Midjourney / Figma AI / v0 | Faible |

**Résumé factuel** : sur 8 familles, 2 ont une défense naturelle forte (compute, funding), 1 modérée (accessibility), et **5 sont vulnérables au mass-farming LLM à coût marginal quasi-nul**. Le design initial « pas besoin de vérifier si c'est un humain » tient pour compute mais s'effondre pour les 5 vulnérables.

## 2. Option D initialement proposée (user)

Proposition : **ship in lockstep** — règle workflow inscrite dans `docs/claude/README.md` stipulant qu'aucune famille de kudos ne s'ouvre sans sa défense dans le même sprint. Défenses à 3 composants par famille :

1. **Quality-gate** : validation qualité par pairs crédibles (merge par reviewer trust≥0.8, citation 3× en 30j pour docs, attestation utilité pour review, outcome-based pour moderation, adoption mesurée pour design, rapport + a11y-reviewer pour accessibility).
2. **Rate cap hebdomadaire** par node_id par famille (Code=50, Docs=20, Review=20, Mod=5, Design=10, A11y=10 proposés).
3. **Coefficient voting-weight** réduit (Compute/Funding=1.0×, A11y=1.2×, Code/Mod=0.8×, Design=0.6×, Review=0.5×, Docs=0.4×) — **votables par gouvernance Type C**.

## 3. Investigation 4 agents parallèles (2026-04-20)

Lancée pour trianguler objectivement Option D contre l'état de l'art + contraintes codebase. Chaque agent missionné indépendamment, **instruction explicite de challenger D plutôt que de la valider**.

### 3.1 Agent 1 — Sprint planning + LT-1 audit

**Verdict Option D** : BLOCKER CRITIQUE — rang **5/5** (dernier). Option **E** (déférer) rank 1, Option **C** rank 2, **B** rank 3, **A** rank 4, **D** rank 5 (2/10).

Blockers identifiés :
- **B1 Type-C governance inexistant** : aucun mécanisme « paramètre de gouvernance votable » dans le codebase ou le roadmap S18-S30. Option D s'appuie sur une primitive non-livrée.
- **B2 Window déjà ouverte S22** : `ContributorAttestation` Couche 2 (S22 Phase C `sprint22_kickoff.md:53-68`) introduit **factuellement** une famille implicite « code contributor » sans quality-gate anti-farming. Ship lockstep ne peut pas sécuriser rétroactivement.
- **B3 LT-1 devient obsolète** : les triggers `Gini > 0.70 OR top-5% > 50%` (`ROADMAP_COMMITMENTS.md:65-86`) sont définis agnostiques de taxonomie multi-famille. Option D force leur recalcul **par famille** — LT-1 doit être remplacé, pas complété.
- **B4 Dépendance circulaire governance** : coefficients votables jour 0 impossible sans vote mechanism.

### 3.2 Agent 2 — OSS precedents research (15 systèmes étudiés)

**Verdict Option D** : 4.8/10. Aucun précédent de « ship lockstep rule » dans les 15 systèmes OSS étudiés.

Findings clés :
- **SourceCred** (Protocol Labs, MakerDAO) : coefficients réglables par mainteneur → **capture documentée**, MakerDAO arrêt 2022-10.
- **RetroPGF (Optimism R4+)** : coefficients votables par badgeholders, controverses VC-backed projects.
- **Protocol Guild** ($100M+ cumulés 2022-2025, zéro capture rapportée) : **coefficients non-votables**, time-weighted `sqrt(months)`, invitation seulement. **Pattern dominant qui marche = non-votable + mécanique simple**.
- **StackOverflow** : seul cap hebdo formel observé (200 rep/jour global depuis 2008), pas par catégorie. Adapté 15+ ans.
- **Apache/Debian/Linux** : **refusent la réputation numérique multi-famille**, préfèrent l'invitation PMC/mentor. Pattern **Option E** en pratique.
- **Gitcoin Passport (R11)** : 5.3% flagged Sybil, best-estimate 6.4% actual (3.6-9.3%), 83% détection post-hoc.
- **LayerZero 2024** : 803,273 wallets (59%) removed, cluster unique 60,995 sybils (Nansen).
- **BrightID + AntiSybil** : dormant oct 2021, Passport poids 0.202.

### 3.3 Agent 3 — Recherche académique LLM-farming

**Verdict Option D** : « Composition de patterns connus, combinaison inédite mais **non-avisée** ». Ré-invention du problème que Protocol Guild résout par invitation + Gitcoin par clawback.

Findings empiriques :
- **ICLR 2024** : ≥15.8% reviews AI-assisted.
- **ICLR 2026** : scandal ~21% reviews prompt-injected, policy treats prompt injection comme collusion.
- **Stack Overflow CHI 2024** (Kabir et al.) : 52% réponses GPT-4 incorrectes, 77% verbose, 35% préférées par humains malgré l'incorrection.
- **Coût bypass reviewer-gate** : ~$0.02/review × 40/sem × 100 nodes = **$80/semaine pour bypass complet** (papers refs 3-6).
- **Steem takeover 2020** (Jeong SSRN 4686738) : token-vote governance capturable par exchanges en **72h**.
- **DAO governance empirical** (a16z) : 78% tokens DAO chez top 20%, ENS 1% contrôle 62.4%.
- **Distribution commit-frequency OSS** (Kolassa/Riehle arXiv 1408.4978) : **log-normal**, pas pure power-law. Linux kernel top contributors >100 commits/sem (Intel/Google chacun >12% d'un cycle ~2068 contributeurs).
- **CoderAbbit 2025** : PRs/author +20% YoY grâce à IA, incidents/PR +23.5%.
- **Gitcoin fraud detection coût** (GR13) : $17k humain pour 12k évaluations.

Attaques non-couvertes par Option D (chiffrées) :
1. **Reviewer-farming récursif** : 100 sybils mutualisent reviewer role → trust_multiplier artificiel ≥ 0.8 → validation mutuelle sans limite. Coût ~$80/sem pour bypass complet.
2. **Coefficient capture** : après 6 mois avec 728 voting-weight/node × 100 nodes, l'adversaire vote lui-même les coefficients de sa famille préférée × 2, transformant son stock en 145600 voting-weight. **Self-amplifying**.
3. **Log-normal tail** : caps 50 PR/sem cassent les mainteneurs lourds humains (Linux kernel); adversaires lissent 40/sem × 50 nodes = 2000/sem invisible.

### 3.4 Agent 4 — Faisabilité technique codebase

**Verdict Option D** : INFEASIBLE pré-v1.0.

Coûts chiffrés :
- **ContributionType schema** : ~550 LOC Rust+Python + bump `DOMAIN_KUDOS_V1 → V2` → **viole pre-launch protocol policy** (`docs/claude/README.md §6.1.4`). P0 gate.
- **Quality gate peer-review** : ~250 LOC mais dépend de trust assignment mechanism (governance) → loop P1.
- **Rate cap hebdo refactor** : ~550 LOC (RateKey restructure + week bucketing).
- **Total D** : ~2500 LOC pré-v1.0, 2.5-3 sprints consumés, G7 cap exhausted S23-S24, **Gate 2 jeopardy** (+6 semaines).

Comparaison :

| Option | Pre-v1.0 LOC | Sprints | Gate 2 | v1.0 timeline |
|---|---|---|---|---|
| **D full** | ~2500 | 2.5-3 | +6 semaines | JEOPARDY |
| **E defer** | 150 (planning) | 0 | None | On track |
| **F split gov** | ~800 | 1 | +2 semaines | Tight shippable |

## 4. Blockers convergents (4/4 agents)

1. **Governance Type C n'existe pas** (Agents 1, 4) — primitive manquante.
2. **Window déjà ouverte par S22 Couche 2** (Agent 1) — `ContributorAttestation` crée famille implicite sans défense.
3. **Wire version bump** (Agent 4) — viole `docs/claude/README.md §6.1.4` pre-launch policy.
4. **Peer-review récursivement farmable** (Agent 3) — ICLR 2024 15.8%, ICLR 2026 21%, bypass $80/sem.
5. **Coefficients votables empiriquement capturés** (Agents 2, 3) — Steem 72h, SourceCred arrêt MakerDAO, ENS 1% contrôle 62.4%. Protocol Guild stable car **non-votable**.
6. **Caps 50/20 non-justifiés** (Agent 3) — log-normal kernel Linux, 50 PR/sem casse mainteneurs humains lourds.
7. **Gate 2 jeopardy** (Agent 4) — 2.5-3 sprints coût, viole cap G7.

## 5. Option F consolidée — composition asymétrique de patterns validés

Émerge de la convergence des 4 agents (3/4 proposent F, 1/4 propose E avec F post-v1.0). Principe : **ne pas inventer une règle workflow inédite, copier 3 patterns empiriquement validés sur 15-30 ans chacun, en les composant par classe d'observabilité de la famille**.

### 5.1 Couche A — Objective-quantifiable (compute + funding)

- **Mécanisme** : coût physique/monétaire intrinsèque = Sybil defense naturelle. Rate-cap global StackOverflow-style (200 rep/jour adapté) en complément cosmétique.
- **Précédent empirique** : SBFB S21 Phase A `63afe4e` (governor GCRA live), StackOverflow 2008-2026 (15 ans stable).
- **Changement requis** : **aucun** — le système actuel fait déjà ça.

### 5.2 Couche B — Subjective-judgeable post-hoc (code + docs)

- **Mécanisme** : evaluator committee rotatif + kudos conditionnels gelés 30 jours (clawback RetroPGF-style). Le committee évalue a posteriori la valeur réelle de la contribution avant de dégeler le kudos.
- **Précédent empirique** : Optimism RetroPGF R1-R5 (2022-2026), Gitcoin Grants R11-14 ($17k humain / 12k évaluations = $1.4/évaluation, amortissable).
- **Rationale** : rate caps a priori = optimal pour familles objectivement mesurables (compute = oui, docs = non). Le jugement post-hoc par humains évite la fragilité du peer-review automatique (récursivement farmable).

### 5.3 Couche C — Social-only (review + moderation + design + accessibility)

- **Mécanisme** : **invitation Protocol-Guild-style** time-weighted par membres existants. Pas de score numérique, pas de vote coefficient, pas de rate-cap.
- **Précédent empirique** : Protocol Guild ($100M+ cumulés 2022-2025, zéro capture), Linux kernel MAINTAINERS (30+ ans), Debian Developer ladder, Apache PMC.
- **Rationale** : les familles dont la valeur est *intrinsèquement sociale* ne peuvent pas être quantifiées proprement. L'invitation non-votable est le seul pattern stable >3 ans observé.

### 5.4 Ce qui est éliminé de l'Option D

- Coefficients votables par gouvernance Type C → **capture risk** (Steem, SourceCred).
- Rate caps par famille 50/20/5 → **chiffres non-justifiés** (log-normal kernel).
- Taxonomie `ContributionType` monolithique pré-v1.0 → **wire bump P0** (pre-launch policy).
- Règle workflow « ship lockstep » → **sans précédent vérifiable** (agent 2, 15 systèmes scannés).

## 6. Plan d'implémentation

### 6.1 Maintenant (hors-sprint chore planning — zéro code)

1. **Research doc capture** (ce fichier) — référence future.
2. **LT-3 entry** dans `docs/release/ROADMAP_COMMITMENTS.md` — engagement formel conditionnel.
3. **Sprint stub** `.planning/reserved/S31_contribution_families_kickoff.md` — kickoff pré-rempli activable.
4. **Amendement HARDENING §3 S23** — ajout 2 items (design doc + observability endpoint).
5. **Memory update** `nexus_grid_pivot.md` — frontmatter reflète LT-3 + S23 amendé.

### 6.2 S23 (lors du kickoff S23 — design + observability foundation)

- **Phase C** : ajouter « Contribution families design doc + KUDOS_V2 wire spec » (~400 LOC docs, design-only). Co-localisé avec « Couche 3 design doc finalisation » déjà planifié S23.
- **Phase D** : ajouter « Fairness observability endpoint `/diagnostic/fairness` » (~80 LOC Python + 40 LOC tests). Calcule Gini + top-5% + churn-rate-vs-hardware du ledger compute existant. **Zéro wire impact, zéro schema change**. Rend triggers LT-1/LT-3 factuellement mesurables dès Gate 2 activation (fin S22).

### 6.3 Post-v1.0 (horizon S31-S32, implémentation conditionnelle)

Déclencheurs (au moins UN) :
- Gate 2 ou Gate 3 activé avec ≥3 contributeurs non-compute réels actifs pour ≥1 app
- `/diagnostic/fairness` reporte Gini > 0.70 OR top-5% > 50% sur ledger compute
- Audit externe Cure53/ToB S29 signale vulnérabilité contribution-family concrète

Si activé → sprint dédié (~1500 LOC Rust+Python + docs) implémentant les 3 couches asymétriques calibrées sur data empirique.

Si jamais activé → **kudos reste compute-only indéfiniment**. Familles non-compute restent socialement reconnues (AUTHORS, `ContributorAttestation` S22 Couche 2 binaire) **sans score**. C'est le pattern Apache/Debian/Linux, validé 30+ ans.

## 7. Sources

### Agent 1 — Sprint planning + codebase
- `packages/nexus-coordinator/src/nexus_coordinator/kudos.py:52-94` — schema actuel
- `docs/release/ROADMAP_COMMITMENTS.md` — LT-1 fairness reform
- `docs/FAIRNESS_VISION.md §7-8` — horizon post-v1.0
- `docs/security/HARDENING_ROADMAP.md §3 S22-S30`
- `.planning/active/sprint22_kickoff.md §4` — Day 0 S22

### Agent 2 — OSS précédents (URLs datées 2026-04-20)
- StackOverflow Daily Dose Reputation Cap (blog 2008-12)
- Gitcoin R11 Anti-Fraud Evaluation (BlockScience)
- Gitcoin Sybil Resistance in QF 2024
- Holonym acquires Gitcoin Passport
- RetroPGF R4/R5 Optimism Docs (2024-2026)
- Lessons Learned 2 Years RetroPGF (Optimism governance)
- Protocol Guild Compensation Insights 2025
- Protocol Guild docs (membership + time_weight)
- MakerDAO End SourceCred Funding poll Oct 2022
- SourceCred Trial Final Report MakerDAO
- Zargham Exploring Subjectivity SourceCred
- Colony reputation decay whitepaper
- Coordinape Gift Circle docs
- Apache Community New Committer
- Debian New Members Corner
- Linux Kernel MAINTAINERS + Contribution Maturity Model
- Hypercerts Foundation docs
- TheNewStack AI PR Crisis 2026 (Geerling)
- CoderAbbit State of AI vs Human Code 2025

### Agent 3 — Recherche académique 2024-2026
- Kumar et al. IEEE S&P 2024 — GossipSub formal analysis (arXiv 2212.05197)
- Kabir et al. CHI 2024 — "Is Stack Overflow Obsolete?" (dl.acm.org/doi/10.1145/3613904.3642596)
- "Detecting LLM-Generated Peer Reviews" arXiv 2503.15772
- "When Your Reviewer is an LLM" arXiv 2509.09912
- "Breaking the Reviewer" arXiv 2506.11113
- ICLR 2026 LLM policy (iclr.cc 2025-08-26)
- Lalley & Weyl — Quadratic Voting collusion formel
- Buterin-Hitzig-Weyl — Flexible Public Goods arXiv 1809.06421
- BlockScience Gitcoin GR11 (block.science.com)
- LayerZero + Nansen 2024 airdrop fairness
- Jeong — Steem Takeover SSRN 4686738
- Kolassa/Riehle — OSS Commit Frequency arXiv 1408.4978
- Apache Inequality PLOS ONE
- Benkler 2017 — Peer Production
- a16z DAO Governance Attacks
- Protocol Guild Membership (protocol-guild.readthedocs.io)
- Optimism RetroPGF Origin (Buterin 2021)

### Agent 4 — Codebase technique
- `packages/nexus-coordinator/src/nexus_coordinator/kudos.py:52-94`
- `crates/nexus-core-rs/src/canonical.rs:50-138`
- `crates/nexus-worker-core/src/rate_limit.rs` (S21 primitive)
- `packages/nexus-coordinator/src/nexus_coordinator/provenance.py:35-45`
- `docs/claude/README.md §6.1.4` pre-launch protocol policy

---

## 8. Conclusion

Option D initiale user était **intention correcte, design prématuré**. La valeur de l'analyse : l'asymétrie coût-production entre 8 familles est un problème réel que ni LT-1 ni le kickoff S22 ne couvraient explicitement.

La meilleure réponse objective pour un projet open-source qui veut être solide dès v1.0 = **Option F composition de 3 patterns validés 15-30 ans chacun**, implémentée post-v1.0 sur data empirique via `LT-3`. Capture préservée dès aujourd'hui via research doc + LT-3 + sprint stub + observability endpoint S23.

Pattern dominant observé dans les 4 agents : **ne pas inventer, composer des briques éprouvées**. SourceCred/BrightID/Steem capturés montrent que la sophistication custom échoue ; Protocol Guild/Apache/Debian/StackOverflow stables montrent que la simplicité + non-votabilité + invitation tient.
