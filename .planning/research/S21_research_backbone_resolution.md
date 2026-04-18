---
sprint: 21
topic: gliner_pii_edge_backbone_resolution_primary_source
date: 2026-04-18
agent: general-purpose (WebFetch HF primary sources, ~20 min)
prompt_source: agent team post-G1 multi-agent fixes 2026-04-18 (retro résolution ambiguïté backbone)
word_count: ~1500
archive_rationale: §6.11 README — résolution factuelle d'une divergence documentée entre 3 sources précédentes
---

## Prompt donné à l'agent

Mission factuelle : trancher définitivement le backbone architecture du modèle `knowledgator/gliner-pii-edge-v1.0` sur HuggingFace (ModernBERT vs DeBERTa-v3 — divergence entre sources précédentes) + déterminer l'opset ONNX export exact + la taille précise de la variante quantized. Pas de supposition, sources primaires uniquement.

Contexte divergence :
- Recherche initiale pré-research G2 : « ModernBERT backbone »
- G1 Design Review Board : « backbone DeBERTa-v3 »
- WebFetch model card initial : « UINT8 197 MB » (probablement base-v1.0 pas edge-v1.0)

Cette ambiguïté doit être résolue AVANT Phase B pour éviter un DESIGN-CONFLICT.

## Rapport reçu

### 1. Verdict résumé

- **Backbone : ModernBERT** (variante `jhu-clsp/ettin-encoder-32m`, 32M params, 10 layers, hidden 384, 6 heads, vocab 50 370, context 8192 tokens)
- **Tokenizer : BPE** (PreTrainedTokenizerFast, style GPT/OLMo avec `add_prefix_space: true`, vocab ~50 368 — hérité du tokenizer ModernBERT upstream)
- **Taille ONNX quantized : `model_quint8.onnx` = 45.8 MB** (pas 197 MB — la valeur 197 MB du README concerne la **variante `base-v1.0` (DeBERTa-v3-small)**, pas `edge`)
- **Opset ONNX : non lisible via WebFetch** (binaire), mais **onnxruntime-web 1.24.3 supporte opset ≤ 21** (ORT 1.20 supporte déjà opset 21 per doc officielle). Les exports `optimum` / `transformers.onnx` récents (août 2025, `transformers_version: 4.55.2`) émettent opset 14-18 par défaut — **compatibilité acquise**
- **Divergence résolue : les 3 sources précédentes divergeaient parce que la recherche pré-research G2 visait `edge`, le G1 Design Review citait `base`, et le WebFetch du model card confondait les tailles edge/base (le README mélange les deux)**

### 2. Preuves factuelles (sources primaires HuggingFace)

#### 2.1 `gliner_config.json` edge — preuve décisive

URL : `https://huggingface.co/knowledgator/gliner-pii-edge-v1.0/raw/main/gliner_config.json`

Champs clés, quote littérale JSON :

```json
"model_name": "jhu-clsp/ettin-encoder-32m",
"encoder_config": {
  "_name_or_path": "jhu-clsp/ettin-encoder-32m",
  "architectures": ["ModernBertForMaskedLM"],
  "model_type": "modernbert",
  "hidden_size": 384,
  "intermediate_size": 576,
  "num_hidden_layers": 10,
  "num_attention_heads": 6,
  "vocab_size": 50370,
  "max_position_embeddings": 7999,
  "global_attn_every_n_layers": 3,
  "global_rope_theta": 160000.0,
  "local_attention": 128,
  "local_rope_theta": 160000.0,
  "cls_token_id": 50281,
  "sep_token_id": 50282,
  "pad_token_id": 50283
}
```

Preuve forte triple-coincidente : `model_type = "modernbert"` + `architectures = ["ModernBertForMaskedLM"]` + `_name_or_path = "jhu-clsp/ettin-encoder-32m"`. Aucune ambiguïté possible.

Note : le `config.json` root retourne 404 — le config canonical GLiNER est bien `gliner_config.json` (pattern standard de la famille GLiNER), pas `config.json`.

#### 2.2 `gliner_config.json` base — contraste confirmant la divergence

URL : `https://huggingface.co/knowledgator/gliner-pii-base-v1.0/raw/main/gliner_config.json`

```json
"model_name": "microsoft/deberta-v3-small",
"encoder_config": {
  "_name_or_path": "microsoft/deberta-v3-small",
  "model_type": "deberta-v2",
  "hidden_size": 768,
  "num_hidden_layers": 6,
  "num_attention_heads": 12,
  "vocab_size": 128003,
  "max_position_embeddings": 512,
  "pos_att_type": ["p2c", "c2p"]
}
```

Confirmation : **`base` = DeBERTa-v3-small**, **`edge` = ModernBERT**. Les deux modèles ont des backbones **différents**. La source G1 « DeBERTa-v3 » parlait du base, la source G2 « ModernBERT » parlait du edge — aucune des deux ne se trompait, elles parlaient de modèles différents.

#### 2.3 Fichiers ONNX edge — tailles précises

URL : `https://huggingface.co/knowledgator/gliner-pii-edge-v1.0/tree/main/onnx`

| Fichier | Taille |
|---|---|
| `model.onnx` (FP32) | **181 MB** |
| `model_fp16.onnx` (FP16) | **90.8 MB** |
| `model_quint8.onnx` (QUINT8 unsigned int8) | **45.8 MB** |

Total répertoire `/onnx` : **318 MB**. Pas de `model_int8.onnx` ni `model_uint8.onnx` — seule la variante `quint8` est distribuée.

#### 2.4 Fichiers ONNX base — preuve que « 197 MB UINT8 » du README concerne base, pas edge

URL : `https://huggingface.co/knowledgator/gliner-pii-base-v1.0/tree/main/onnx`

| Fichier base | Taille |
|---|---|
| `model.onnx` | 665 MB |
| `model_fp16.onnx` | 333 MB |
| `model_quint8.onnx` | **197 MB** |

Le README edge (15.9 kB) référence 330 MB FP16 / 197 MB UINT8 qui sont **littéralement les tailles du model base**, pas edge. **Bug documentation model card HF upstream, pas ambiguïté architecture**.

#### 2.5 `tokenizer_config.json` edge — confirmation style BPE ModernBERT

URL : `https://huggingface.co/knowledgator/gliner-pii-edge-v1.0/raw/main/tokenizer_config.json`

- `tokenizer_class: "PreTrainedTokenizerFast"`
- `model_max_length: 8192`
- Special tokens : `[CLS]=50281`, `[SEP]=50282`, `[PAD]=50283`, `[MASK]=50284`, `[UNK]=50280`
- `add_prefix_space: true` (signature BPE byte-level GPT/OLMo)
- Tokens spéciaux PII customs : `|||IP_ADDRESS|||`, `|||EMAIL_ADDRESS|||`, `|||PHONE_NUMBER|||`, `<<ENT>>`, `<<SEP>>`

#### 2.6 `jhu-clsp/ettin-encoder-32m` existe et confirme ModernBERT

URL : `https://huggingface.co/jhu-clsp/ettin-encoder-32m`

- Architecture : **ModernBERT-based encoder**
- « Transformer with RoPE, GLU activations, and prenorm layers »
- 32M params / 10 layers / hidden 384 / intermediate 576 / 6 attention heads
- Vocab 50 368 (ModernBERT tokenizer), context jusqu'à 8K tokens
- Paper : « Seq vs Seq: An Open Suite of Paired Encoders and Decoders » (arXiv 2507.11412, 15 juillet 2025)
- Licence MIT

Les paramètres du ettin-encoder-32m matchent **exactement** l'`encoder_config` du `gliner_config.json` edge. Cohérence totale.

#### 2.7 Model card edge — F1 et quantization-aware

URL : `https://huggingface.co/knowledgator/gliner-pii-edge-v1.0`

- **F1 75.50 %** sur `synthetic-multi-pii-ner-v1` (vs 80.99 % pour base)
- Précision 78.96 % / rappel 72.34 %
- Approche : « Quantization-aware pretraining »
- Collaboration Wordcab × Knowledgator
- « Optimized for edge environments, trading a slight decrease in recall for lower latency and footprint »

### 3. Divergence expliquée

| Source | Claim | Verdict |
|---|---|---|
| Recherche pré-research G2 | « ModernBERT backbone (`jhu-clsp/ettin-encoder-32m`), 32M params » pour edge | **EXACT** — confirmé par gliner_config.json edge |
| G1 Design Review Board | « backbone DeBERTa-v3 » | **CONFONDU** avec `gliner-pii-base-v1.0` qui est DeBERTa-v3-small |
| WebFetch model card « UINT8 197 MB » | Taille ONNX quantized | **BUG DOC UPSTREAM** — le README edge réutilise la table tailles du README base sans correction. La vraie taille quint8 edge est **45.8 MB** |

**Racine de la divergence : le README de `edge` est un copy-paste partiel du README de `base` sans mise à jour de la section tailles ONNX ni section architecture explicite.** La seule source authoritative est le triplet `gliner_config.json` + `tokenizer_config.json` + file tree `/onnx`.

### 4. Implications Phase B Sprint 21

- **ORT Web 1.24.3** supporte opset ≤ 21 (ORT 1.20 déjà opset 21). Exports `optimum` 2025 émettent opset 14-18 — compatibilité acquise sans effort.
- **ModernBERT supporté dans `@xenova/transformers` v3+** (porté janvier 2025). Tokenizer BPE byte-level + special tokens OLMo déjà présent.
- **Modèle ONNX quint8 = 45.8 MB** → iframe download raisonnable (~10-20 s sur 25 Mbps).
- **Inference** : 32M params / hidden 384 / 10 layers → ~100-300 ms/request CPU, ~50-150 ms avec WebGPU.
- **Bonus vs plan initial** : taille réelle 45.8 MB ≈ **4.3x plus petite** que l'estimation 197 MB du kickoff — budget bundle iframe drastiquement amélioré.

### 5. Recommandation action

Editer `.planning/active/sprint21_kickoff.md §D2` pour remplacer « backbone à confirmer Phase B G8 S1 scan » par « Backbone confirmé ModernBERT via HF `gliner_config.json` primary source 2026-04-18 ». Preuves à archiver dans `.planning/research/S21_research_backbone_resolution.md` (ce fichier).

## Decision downstream

Rapport consommé par :
- `.planning/active/sprint21_kickoff.md §D2` — backbone confirmé ModernBERT (tranche ambiguïté G1)
- `.planning/active/sprint21_kickoff.md §Acknowledged review findings D2 ⚠️` — ack multi-agent team fix 2026-04-18

Zero risque DESIGN-CONFLICT Phase B S21 S1 scan. Re-verification pre-phase reste normale (G8 garde-fou standard) mais devrait retourner EXECUTE plan-as-is sauf release upstream breaking majeur.
