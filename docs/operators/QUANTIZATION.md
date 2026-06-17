# Quantization 4-bit — guide opérateur (GGUF)

Ce document s'adresse à l'**opérateur d'un nœud SBFB** qui offre sa
puissance GPU au réseau (panneau « offrir ma puissance » →
`GpuConsentDialog`). Il explique quel format de modèle quantifié choisir
selon la carte, ce qui tient honnêtement sur une carte grand public
16 Go, et pourquoi les très gros modèles relèvent du **sharding
cross-machine (S77)**, pas du multi-GPU mono-machine.

> **Doc-only.** SBFB ne quantifie rien lui-même. Le 4-bit est *baked*
> dans le fichier `.gguf` au moment où tu le télécharges (bartowski,
> ggml-org, etc.). Le worker charge le GGUF pré-quantifié tel quel via
> `LlamaCppBackend` (`llama_cpp.rs::ensure_model` →
> `LlamaModel::load_from_file` + `with_n_gpu_layers`). Il n'existe
> **aucun** paramètre runtime de quantification dans SBFB.

---

## 1. C'est quoi la quantization, en une phrase

Réduire la précision des poids du modèle (FP16 → entiers 4 bits) pour
diviser l'empreinte mémoire par ~4, au prix d'une perte de qualité
faible et mesurable. Le format **GGUF** (llama.cpp) encode le niveau de
quantization dans le nom du fichier : `…-Q4_K_M.gguf`, `…-IQ4_XS.gguf`,
`…-Q2_K.gguf`.

---

## 2. Quel format choisir (recommandation par taille de carte)

| Situation | Format recommandé | Pourquoi |
|---|---|---|
| **Cas par défaut** (tu as la VRAM) | **Q4_K_M** | Meilleur équilibre qualité/taille pour la production. Perte de perplexité dans le bruit (cf. §6). |
| **VRAM serrée** (le modèle déborde de quelques Go) | **IQ4_XS** | i-quant ~10 % plus compact que Q4_K_M, à qualité voisine. (Équivalence de qualité = propriété générale des i-quants ; IQ4_XS n'est pas tabulé dans la mesure de perplexité du §6.) |
| **Tu serres la taille au plus juste** | **Q4_K_S** | Plus petit que Q4_K_M, perplexité très proche : 7.62 vs 7.56 sur l'arXiv 2601.14277 (§6), écart dans le bruit. |
| **Dernier recours, gros modèle** | Q3_K_M / Q2_K | Très compressé mais qualité dégradée nettement. Q2_K **n'est pas** un défaut acceptable — uniquement si rien d'autre ne rentre. |

**Règle simple** : commence en **Q4_K_M**. Si ça ne rentre pas, passe en
**IQ4_XS** avant de descendre la taille du modèle. Ne descends en
Q3/Q2 qu'en tout dernier recours.

---

## 3. Table d'empreintes VRAM (taille fichier GGUF)

Chiffres réels (HuggingFace `bartowski`, vérifiés 2026-06-15). La VRAM
*au runtime* = taille fichier **+ 1 à 2 Go de KV-cache** selon la
longueur de contexte (déjà borné par les caps, cf. §5).

| Modèle | Q4_K_M | IQ4_XS | Q2_K | Tient sur 1×16 Go ? |
|---|---|---|---|---|
| **7B** | ~4.4 Go | ~3.7 Go | ~2.8 Go | ✅ largement (modèle entier) |
| **8B** | ~4.9 Go | ~4.2 Go | ~3.2 Go | ✅ largement (modèle entier) |
| **14B** | **~8.5 Go** | ~7.6 Go | ~5.8 Go | ✅ **cible single-GPU honnête (modèle entier)** |
| **32B** | ~22 Go | ~17.8 Go | ~12.8 Go | ❌ déborde d'une 16 Go (Q4_K_M ~22 Go) ; tiendrait sur 24 Go, hors-cible single-GPU SBFB |
| **70B** | **42.52 Go** | **37.90 Go** | **26.38 Go** | ❌ **ne tient sur AUCUNE carte 16 Go** |

(70B = `bartowski/Llama-3.3-70B-Instruct-GGUF` : Q4_K_M 42.52, Q4_K_S
40.35, IQ4_XS 37.90, Q3_K_M 34.27, Q2_K 26.38 Go — **tailles de fichier
citées/vérifiées**. 14B Q4_K_M ~8.5 Go et 32B Q4_K_M ~22 Go =
`bartowski/Qwen2.5-*-Instruct-GGUF` mesurés ; les colonnes **IQ4_XS /
Q2_K des modèles < 70B sont extrapolées** du ratio mesuré sur le 70B,
ordres de grandeur seulement.)

---

## 4. La cible honnête sur une carte grand public

**Une carte 16 Go (RTX 4080/5080…) fait tourner un modèle ENTIER
jusqu'à ~14B en Q4_K_M.** C'est la cible single-GPU de SBFB :

> **Ta carte 16 Go → modèles ≤14B Q4_K_M, chargés entièrement sur le
> GPU.** Qwen2.5-14B-Instruct Q4_K_M (~8.5 Go) laisse de la marge pour
> le KV-cache. C'est ce que pointe le panneau « offrir ma puissance ».

### Les gros modèles (32B / 70B) : sharding cross-machine = S77

Le 70B **ne tient sur aucune carte 16 Go** : même sa plus petite
quantization (Q2_K = 26.38 Go) dépasse déjà 16 Go, et même 32 Go ne
suffisent pas en Q4_K_M (42.52 Go).

Le chemin vers ces tailles **n'est PAS** « ajouter une 2ᵉ carte dans la
même machine » :

- Le **tensor-split mono-machine multi-GPU** (`with_split_mode` +
  `with_devices` de `llama-cpp-2`) est **hors cible** : un contributeur
  type a **une** carte, pas deux. SBFB ne câble pas cette API.
- Le multi-GPU réaliste pour SBFB = **éclater le modèle sur 2+ machines
  à 1 GPU chacune = sharding cross-machine = Sprint 77**, où chaque
  nœud porte une tranche de couches et relaie l'activation au suivant.

**Palliatif documenté (pas la voie principale)** : l'offload CPU
mono-machine (`n_gpu_layers` partiel, le reste en RAM) fait tourner un
70B à **~2-5 tok/s** — utilisable en batch/asynchrone, inconfortable en
interactif. À réserver aux cas où la latence n'importe pas.

---

## 5. Lien avec les caps VRAM existants (design note)

Le worker dispose **déjà** de tout ce qu'il faut pour refuser une tâche
trop lourde — Phase F ne touche à rien de ce câblage :

- **Budget VRAM live** : `GpuStats::vram_budget_remaining_bytes(max_vram_fraction)`
  (`crates/nexus-worker-core/src/gpu/mod.rs:147`) calcule la VRAM
  disponible (`total × fraction` puis `saturating_sub(used)`).
- **Gate d'admission par cap** : `crates/nexus-worker-core/src/consent.rs:422-425`
  rejette toute tâche dont `task.estimated_vram_mb` dépasse
  `Caps::max_vram_mb` (`RejectReason::CapVram`). Le slider « VRAM max
  (GB) » du panneau de consentement règle directement ce cap.

> **Honnêteté technique (frontière S77)** : le cap lit
> `task.estimated_vram_mb` (`crates/nexus-core-rs/src/task.rs:258-261`,
> `#[serde(default)]`), c'est-à-dire l'**estimé déclaré par l'app qui
> soumet la tâche**, *pas* la taille réelle du fichier GGUF chargé. Si
> l'app déclare `estimated_vram_mb = 0` (valeur par défaut « inconnu »),
> le cap VRAM est **inerte** (il ne rejette que si l'estimé dépasse
> `max_vram_mb`) — le contributeur reste protégé par les caps watts /
> heures et par le niveau de consentement, mais pas encore par une
> mesure VRAM réelle. Brancher l'admission sur la VRAM réellement requise
> (taille GGUF + KV-cache mesurés) est un **câblage VRAM-live laissé à
> S77** ; ce n'est pas bloquant aujourd'hui.

---

## 6. Combien on perd vraiment en 4-bit

Source : *« Which Quantization Should I Use? »*, arXiv 2601.14277v1
(2026-01-11). Perplexité (plus bas = mieux) :

| Format | Perplexité |
|---|---|
| F16 (référence) | 7.32 |
| Q4_K_M | 7.56 |
| Q4_K_S | 7.62 |
| Q4_0 | 7.74 |

L'écart F16 → Q4_K_M est **dans le bruit** pour la plupart des usages :
le 4-bit Q4_K_M est un défaut défendable, pas un compromis dégradant.

---

## 7. Pré-condition quorum : MÊME GGUF (lien D3 / redundancy>1)

Quand une tâche `verifiable` est exécutée en **redondance > 1** (quorum
cross-machine, Phase D), les workers d'une même cohorte **DOIVENT
utiliser exactement le même fichier GGUF** (même modèle, **même
quantization**, même build de `llama.cpp`).

C'est une **condition d'exactitude / de joignabilité du quorum**, pas
une barrière de sécurité :

- Deux quants différents (ou deux builds différents) produisent des
  logits puis des tokens **divergents** ⇒ l'**exact-match** du quorum
  **ne se forme jamais**. La tâche reste sans verdict (disponibilité
  dégradée), elle n'est pas compromise.
- Un worker qui exécute un GGUF différent **s'auto-exclut** de la
  majorité : son `result_text` divergent est rejeté comme *outlier* par
  `validate_quorum_pre_guardrail` (inchangé).

> La cohorte homogène (`required_runtime`, Phase C) n'est qu'un
> **routage ADVISORY** qui co-localise les workers compatibles ; la
> vraie défense reste l'**exact-match**. Détail dans
> [`THREAT_MODEL.md`](../security/THREAT_MODEL.md) §15.2 (rows « Worker
> menteur » / « Divergence cross-GPU lue comme un bug ») et
> [`PATTERNS.md`](../rust/PATTERNS.md) §P60.2. La divergence *cross-GPU
> hétérogène* (même GGUF, GPU différents) est **attendue** (réordon-
> nancement flottant) et serait gérée à l'étage 2 TOPLOC (S77, §P60.3).

---

## 8. Sécurité — CVE llama.cpp connues (non applicables)

SBFB utilise llama.cpp **in-process** via `llama-cpp-2` (pas le
`rpc-server`, pas le GBNF natif) :

- **CVE-2026-34159** (llama.cpp RPC RCE, CVSS 9.8) — **non applicable** :
  SBFB n'expose pas `rpc-server`, l'inférence est in-process.
- **CVE-2026-2069** (overflow GBNF, 4.8 local) — **non applicable** :
  le sampling contraint passe par `llguidance` côté Rust, pas par le
  GBNF natif.

---

## 9. Sources (vérifiées 2026-06-15)

- `bartowski/Llama-3.3-70B-Instruct-GGUF`, `bartowski/Qwen2.5-14B-Instruct-GGUF`,
  `bartowski/Qwen2.5-32B-Instruct-GGUF` (HuggingFace) — tailles GGUF.
- *Which Quantization Should I Use?*, arXiv 2601.14277v1 (2026-01-11) —
  perplexité par format.
- `docs.rs/llama-cpp-2` (0.1.143, pin workspace) — `LlamaModelParams`
  (`with_n_gpu_layers` ; `with_split_mode`/`with_devices` = API
  multi-GPU réservée S77).
- `github.com/ggml-org/llama.cpp/blob/master/docs/multi-gpu.md`,
  `sitepoint.com/vram-requirements-70b-models-16gb-gpu-2026`.
- CVE : `sentinelone.com/vulnerability-database/cve-2026-34159`,
  `cve-2026-2069`.
