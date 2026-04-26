# Split Inference Design — Research Document

**Sprint 30 Phase D livrable** (2026-04-26). Document de recherche
sur les patterns de verification et de confidentialite pour
l'inference distribuee dans le contexte SBFB. Pas de code —
findings pour un sprint dedie post-Gate 4.

**Scope** : inference LLM repartie sur N workers non-trusted (pas
de serveur central, pas d'admin). Le modele est heberge worker-
side. Le consumer soumet un prompt, recoit un resultat, ne
controle pas le worker. Le coordinator distribue les taches et
agrege les resultats.

---

## 1. Contexte SBFB

### 1.1 Pourquoi le split inference est pertinent

SBFB distribue du compute LLM sur des workers volontaires. Le
threat model (`THREAT_MODEL.md §5 STRIDE`) identifie deux classes
de menaces directement liees a l'inference distribuee :

- **C-PromptLeak** (tier T5, impact 5/5) : un worker malveillant
  voit le prompt en clair et peut l'exfiltrer. Mitigation actuelle
  = ephemeral workers + VRAM wipe (S23). Mitigation maximale = TEE
  attestation (LT-4) + split inference (ce document).
- **C-ResultSpoof** (tier T5, impact 5/5) : un worker renvoie un
  resultat forge. Mitigation actuelle = redundancy voting 3-worker
  majority (S23) + watermark canary-input spot-check (S22/S28).

Le split inference promet de partitionner le modele pour que
**aucun worker individuel ne voie l'integralite du prompt ou des
activations**, mais au prix d'une complexite reseau et d'une
surface d'attaque supplementaire sur les activations intermediaires.

### 1.2 Contraintes specifiques SBFB

- **Zero serveur central** : pas de trusted third party pour
  orchestrer le partitionnement. Le coordinator est un processus
  local au consumer.
- **Workers heterogenes** : GPU differents (RTX 3060 → H100),
  bande passante variable, pas de datacenter uniforme.
- **Latence sensible** : inference LLM interactive (streaming
  tokens), chaque hop supplementaire ajoute de la latence.
- **Pre-launch** : zero deploiement live, design doc = le livrable,
  pas de code.

---

## 2. Patterns existants

### 2.1 BOINC — Verification par redundance deterministe

**Reference** : Anderson, D.P. (2019). "BOINC: A Platform for
Volunteer Computing". arXiv:1903.01699.

**Mecanisme** : BOINC distribue des taches identiques a N workers
independants (redundancy factor configurable). Un validateur
compare les outputs et cherche un **quorum de resultats
equivalents**. Pour les applications a sortie deterministe (ex:
simulation physique), le validateur compare byte-par-byte. Pour
les sorties non-deterministes, un validateur specifique a
l'application decide de l'equivalence (tolerances numeriques).

**Applicabilite SBFB** :

| Dimension | Applicable ? | Commentaire |
|---|---|---|
| Redundance N-worker | **Oui** | SBFB l'implemente deja (S23 `Task.redundancy_factor` 3-worker majority) |
| Validation byte-identique | **Non** | LLM inference est stochastique (temperature, sampling top-k/top-p, seed GPU non-deterministe). Deux runs du meme prompt sur le meme modele produisent des outputs differents |
| Validateur application-specifique | **Partiel** | Possible via watermark canary-input (S22/S28) : le consumer glisse des prompts known-answer et verifie la coherence. Mais ca ne valide pas chaque token |
| Quorum iteratif | **Oui** | Si les resultats divergent, generer une instance supplementaire — pattern replicable |

**Lecon BOINC** : la verification par hash deterministe est
inapplicable aux LLMs stochastiques. La verification par
redundance (N-worker majority) reste le mecanisme principal, mais
ne couvre pas la confidentialite du prompt (chaque worker voit le
prompt en clair).

### 2.2 Truebit — Verification interactive par jeu

**Reference** : Teutsch, J. & Reitwiessner, C. (2017). "A
scalable verification solution for blockchains".
https://people.cs.uchicago.edu/~teutsch/papers/truebit.pdf

**Mecanisme** : Truebit est un protocole cryptoeconomique de
verification off-chain. Un **solver** execute la tache, un
**challenger** peut contester le resultat. Le protocole joue un
**jeu de verification interactive** : par recherche binaire sur
les etapes de calcul, les deux parties isolent l'etape exacte ou
les Merkle proofs de l'etat memoire divergent. L'arbitrage on-
chain verifie uniquement cette etape isolee (cout constant).

**Applicabilite SBFB** :

| Dimension | Applicable ? | Commentaire |
|---|---|---|
| Verification par jeu | **Partiel** | Le principe est sound : isoler l'etape de calcul fautive. Mais l'inference LLM est un forward pass continu (pas de steps discrets checkpointables sans overhead) |
| Recherche binaire sur etapes | **Non** | Le forward pass d'un transformer est un pipeline de layers. Chaque layer produit un tenseur d'activations. Le checkpoint de chaque layer est possible mais coute ~O(model_size) en memoire/bande passante par checkpoint |
| Cout constant arbitrage | **Oui** | Si l'on peut isoler le layer fautif, un re-compute d'un seul layer est O(1) vs le modele complet |
| Incentives economiques | **Non** | SBFB n'a pas de token. Les kudos sont non-transferables, non-monetaires. L'incentive a challenger est la reputation curator, pas un gain financier |

**Lecon Truebit** : l'idee de verification interactive par
isolation d'etape est transposable au partitionnement par layer
d'un transformer. Un consumer pourrait re-verifier un layer
specifique en re-computant les activations d'entree → sortie de
ce layer. Mais le cout de transfert des activations intermediaires
est le bottleneck (cf. §3).

### 2.3 Golem — Task markets + reputation

**Reference** : Golem Network. https://www.golem.network/

**Mecanisme** : Golem est un marketplace de compute distribue.
Les requestors publient des taches, les providers les executent,
et un systeme de reputation + escrow tokenise gere la confiance.
Les taches sont executees dans des environnements isolees (Docker/
VM). La verification repose sur la reputation du provider +
la capacite du requestor a re-executer la tache (costly).

**Applicabilite SBFB** :

| Dimension | Applicable ? | Commentaire |
|---|---|---|
| Task markets | **Oui** | SBFB a deja ce pattern : `ProjectAnnouncement` + `ClaimEntry` + dispatcher |
| Reputation scoring | **Oui** | Kudos per-project (S0+), ContributorAttestation (S22), trust-web multi-forge (S27) |
| Isolation compute | **Partiel** | SBFB a ephemeral workers (S23) + VRAM wipe + process isolation (S29). Pas de VM/Docker formelle |
| Re-execution verification | **Oui** | SBFB re-run sampling 1-5% (S24) + watermark canary-input (S22/S28) |

**Lecon Golem** : le pattern reputation + task markets est deja
le modele SBFB. Golem ne resout pas le probleme de confidentialite
du prompt — le provider voit la tache en clair. L'isolation
compute Golem (Docker) est plus forte que l'isolation SBFB
actuelle (ephemeral workers + VRAM wipe) mais moins forte que TEE.

### 2.4 Split Learning — Partitionnement de modele

**References** :

- Gupta, O. & Raskar, R. (2018). "Distributed learning of deep
  neural network over multiple agents". arXiv:1810.06060.
- "Advancements and challenges in privacy-preserving split
  learning: experimental findings and future directions".
  International Journal of Information Security, Springer (2025).
  https://link.springer.com/article/10.1007/s10207-025-01045-9
- "Private and Secure Distributed Deep Learning: A Survey". ACM
  Computing Surveys (2024).
  https://dl.acm.org/doi/10.1145/3703452

**Mecanisme** : le modele neural est coupe en segments. Le
client execute les premieres couches (+ optionnellement les
dernieres dans le schema "U-shaped"), le serveur execute les
couches intermediaires. Seules les **activations intermediaires**
(smashed data) sont transmises entre client et serveur — pas le
raw input ni le raw output.

**Variantes** :

- **Vanilla split** : client → cut layer activations → server →
  output. Le serveur voit les activations mais pas l'input brut.
- **U-shaped split** : client (debut + fin) → server (milieu).
  Le serveur ne voit ni l'input ni l'output, seulement les
  activations intermediaires des couches centrales.
- **PPSL (Privacy-Preserving Split Learning)** : ajout de
  Differential Privacy (DP) ou Homomorphic Encryption (HE) sur
  les activations pour empecher la reconstruction de l'input
  depuis les smashed data.

**Applicabilite SBFB** :

| Dimension | Applicable ? | Commentaire |
|---|---|---|
| Partitionnement modele | **Partiel** | Applicable si le consumer peut executer les premieres/dernieres couches localement. Mais : (a) les modeles LLM 7B+ ne tiennent pas sur le GPU consumer moyen, (b) le partitionnement layer-by-layer d'un transformer est specifique a l'architecture (attention heads, KV-cache cross-layer) |
| U-shaped split | **Desirable** | Le consumer garde le prompt et le resultat, le worker ne voit que les activations intermediaires. Pattern ideal pour C-PromptLeak |
| Smashed data transfer | **Bottleneck** | Les activations intermediaires d'un LLM 7B sont de l'ordre de ~128 MB par token (batch_size × seq_len × hidden_dim × sizeof(float16)). Pour un prompt de 1000 tokens, le transfert est ~128 GB — inacceptable en latence et bande passante |
| DP sur activations | **Degradation** | La Differential Privacy sur les activations reduit significativement la qualite du texte genere (les activations sont des features denses, pas des sparse embeddings). Les budgets epsilon eleves requis pour maintenir la qualite rendent la protection faible |
| HE sur activations | **Inacceptable** | L'inference HE sur un transformer complet est 100-1000x plus lente que le plaintext (benchmarks TFHE). Incompatible avec le streaming interactif |

**Lecon split learning** : le U-shaped split est le pattern le
plus prometteur pour la confidentialite du prompt, mais les couts
de transfert et de compute sont prohibitifs pour les LLMs actuels
(7B+). Le split learning a ete demontre sur des modeles CNN/
ResNet plus petits. L'applicabilite aux transformers larges est
un probleme ouvert.

---

## 3. Implications threat model

### 3.1 Nouvelles surfaces d'attaque

Le split inference introduit des vecteurs non couverts par le
threat model actuel (`THREAT_MODEL.md §5`) :

| Vecteur | Description | Severity |
|---|---|---|
| **SI-1 Activation reconstruction** | Un worker recevant les activations intermediaires peut tenter de reconstruire l'input original via un modele inverse (decoder) entraine sur le meme modele. Attaques demontrees sur ResNet/VGG (Vepakomma et al. 2020). Applicabilite aux transformers : plus difficile (attention patterns), mais pas impossible. | High |
| **SI-2 Layer gradient leakage** | Si le protocole inclut un backward pass (fine-tuning distribue), les gradients des activations leakent de l'information sur l'input (Zhu et al. 2019 DLG). SBFB = inference-only (pas de training distribue), donc **non applicable** en mode inference pure. | N/A (inference-only) |
| **SI-3 Activation fingerprinting** | Meme sans reconstruire l'input, les patterns statistiques des activations intermediaires peuvent identifier le **type** de prompt (classification vs generation, langue, domaine). Correlation possible avec des prompts connus. | Medium |
| **SI-4 Collusion inter-workers** | Si le split est sur 2+ workers, la collusion de tous les workers reconstruit le pipeline complet. La confidentialite ne tient que si au moins 1 worker est honnete (modele honest-but-curious). | High |
| **SI-5 Latence side-channel** | Le temps de compute d'un layer revele la complexite du prompt (longueur, nombre d'attention heads actifs). Mitigation : padding constant-rate (cf. `maybenot` VALIDATED_BLUEPRINT.md). | Low |

### 3.2 Interactions avec mitigations existantes

| Mitigation SBFB existante | Interaction split inference |
|---|---|
| Redundancy voting 3-worker (S23) | **Compatible** : chaque segment peut etre re-execute sur N workers independants |
| Watermark canary-input (S22/S28) | **Compatible** : le consumer insere le canary avant le split, verifie apres le merge |
| Ephemeral workers + VRAM wipe (S23) | **Renforce** : les activations intermediaires sont purgees a chaque restart |
| TEE attestation (LT-4) | **Complementaire** : le TEE protege les activations en memoire, le split reduit ce qui est expose a chaque worker |
| Rate-limit GCRA (S21/S22) | **Orthogonal** : pas d'interaction directe |
| COOP/COEP blob-serve (S30) | **Orthogonal** : isolation iframe, pas compute-side |

### 3.3 Impact sur THREAT_MODEL.md §9

Si le split inference est implemente dans un sprint futur, les
sections suivantes de `THREAT_MODEL.md §9` devront etre etendues :

- §9.5 ajout d'une sous-section "Split inference residual risks"
  (SI-1 through SI-5)
- §9.1 consent GPU : nouveau mode "split-participant" (le worker
  consent a executer un segment, pas le modele complet)
- §9.4 rate-limit : les activations intermediaires consomment de
  la bande passante, budget a integerr dans le rate-limit

---

## 4. Recommendations

### 4.1 Approche recommandee : Truebit-style layer verification adaptee

Combiner les lecons des 4 patterns :

1. **Redundance N-worker** (BOINC, deja SBFB S23) comme
   mecanisme de verification primaire. Le split ne remplace pas la
   redundance — il la complete.

2. **Verification interactive par layer** (Truebit adapte) : le
   consumer peut demander a un worker independant de re-compute un
   layer specifique (cut point verification). Cout O(1 layer) au
   lieu de O(model). Applicable si le consumer fournit les
   activations d'entree du layer (necessites archivees
   temporairement).

3. **U-shaped split** (split learning) comme objectif long-terme
   pour confidentialite maximale C-PromptLeak. Prerequis :
   - Modeles suffisamment petits pour que le consumer execute
     debut+fin (ou edge GPU suffisamment puissant)
   - Protocole de transfert d'activations compresse (quantized
     activations, pruning intermediaire)
   - Budget latence acceptable (< 2x vs monolithique)

4. **Reputation + spot-check** (Golem, deja SBFB S22+) comme
   filet de securite continu. Le split ne remplace pas la
   verification probabiliste.

### 4.2 Ce que SBFB ne devrait PAS faire

- **Ne pas implementer de split inference avant Gate 4**. Les
  prerequis (TEE, recrutement, audit) sont plus urgents que le
  split pour les threat levels Gate 1-3.
- **Ne pas implementer HE sur activations**. Le cout 100-1000x
  est prohibitif pour l'inference interactive. Les approches PPSL
  avec HE sont reservees au training/fine-tuning offline.
- **Ne pas forker les runtimes LLM** (llama.cpp, Ollama) pour
  injecter des cut points. Preferer une approche wrapper qui
  intercepte les activations aux frontieres de couches sans
  modifier le runtime.
- **Ne pas supposer que le split resout C-PromptLeak seul**. Sans
  DP ou TEE, les activations intermediaires leakent de
  l'information (SI-1). Le split est un element de defense-in-
  depth, pas une solution complete.

### 4.3 Sprint dedie — scope suggere

Un sprint dedie post-Gate 4 pourrait livrer :

- **Phase A** : design doc formelle SPLIT_INFERENCE_PROTOCOL.md
  (wire format cut points, activation serialization, compression)
- **Phase B** : prototype cut-point verification sur un modele
  toy (7B, 1 layer split) — mesure latence et bande passante
  reelles
- **Phase C** : integration threat model SI-1..SI-5 dans
  THREAT_MODEL.md
- **Phase D** : U-shaped split prototype si budget latence OK

---

## 5. References

### Academiques

1. Anderson, D.P. (2019). "BOINC: A Platform for Volunteer
   Computing". arXiv:1903.01699.
   https://arxiv.org/pdf/1903.01699

2. Teutsch, J. & Reitwiessner, C. (2017). "A scalable verification
   solution for blockchains" (Truebit whitepaper).
   https://people.cs.uchicago.edu/~teutsch/papers/truebit.pdf

3. Gupta, O. & Raskar, R. (2018). "Distributed learning of deep
   neural network over multiple agents". arXiv:1810.06060.

4. "Advancements and challenges in privacy-preserving split
   learning: experimental findings and future directions".
   International Journal of Information Security, Springer (2025).
   https://link.springer.com/article/10.1007/s10207-025-01045-9

5. "Private and Secure Distributed Deep Learning: A Survey". ACM
   Computing Surveys (2024).
   https://dl.acm.org/doi/10.1145/3703452

6. Vepakomma, P. et al. (2020). "NoPeek: Information leakage
   reduction to share activations in distributed deep learning".
   arXiv:2008.09161.

7. Zhu, L. et al. (2019). "Deep Leakage from Gradients". NeurIPS
   2019.

### Projets OSS

8. BOINC — https://boinc.berkeley.edu/
   Validation redundante, quorum N-worker, 20+ ans de production.

9. Truebit — https://truebit.io/
   Verification interactive, jeu binaire challenge/response.

10. Golem Network — https://www.golem.network/
    Task markets distribues, reputation, isolation Docker/VM.

### Documents SBFB connexes

- [`THREAT_MODEL.md`](THREAT_MODEL.md) §5 STRIDE, §9 residual
  risks per-configuration
- [`COMPUTE_THREATS.md`](COMPUTE_THREATS.md) — C-PromptLeak,
  C-ResultSpoof, C-ComputeTheft
- [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) §3 S30 (split
  inference research)
- [`VALIDATED_BLUEPRINT.md`](VALIDATED_BLUEPRINT.md) Couche 6
  (TEE), Couche 12 (research track split inference)
- [`PROCESS_ARCHITECTURE.md`](PROCESS_ARCHITECTURE.md) — broker/
  executor IPC (foundation pour activation transfer protocol)
