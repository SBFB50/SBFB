---
sprint: 21
topic: ort_wasm_alternatives_post_g1_rust_first_recheck
date: 2026-04-18
agent: general-purpose (deep research, ~60 min, context7 MCP + WebSearch)
prompt_source: agent team post-G1 multi-agent fixes 2026-04-18 (combler lacune "custom Rust stack alternatives" non explorée par G1)
word_count: ~2500
archive_rationale: §6.11 README — research output > 2000 mots fondant rejet final Rust-first + conditions re-evaluate S22+
---

## Prompt donné à l'agent

Mission factuelle : combler une lacune identifiée post-mortem dans G1 Design Review Board Sprint 21 — l'agent G1 a validé « Option 7 JS iframe + Presidio coord-side » mais n'a PAS explicitement évalué l'alternative « custom `nexus-pii-rs` crate utilisant `ort` (wrapper Microsoft ONNX Runtime) avec wasm32-unknown-unknown target ». Vérifie objectivement si cette option Rust-first est viable en 2026 ou reste factuellement bloquée (comme pour tract).

Questions : Q1 ort (pyke) wasm target, Q2 ort backends wasm, Q3 wasm32-wasi vs wasm32-unknown-unknown, Q4 candle-wasm / burn-wasm alternatives, Q5 simulacre nexus-pii-rs avec candle + wasm + GLiNER, Q6 benchmark public, Q7 verdict final objectif.

## Rapport reçu

### 1. Executive summary objectif

L'option « custom `nexus-pii-rs` crate Rust-first wasm32 » est **factuellement viable techniquement en 2026, mais non viable dans le budget 2-semaines Phase B S21**, pour des raisons différentes de tract.

Trois sous-options Rust-first existent et sont réelles :
- **`ort` (pyke) + backend `ort-web` v0.2.1+1.24** : wrapper qui appelle `onnxruntime-web` depuis Rust via wasm-bindgen → c'est `onnxruntime-web` déguisé en Rust, pas un runtime Rust natif, et explicitement marqué **« experimental, use at your own risk »**.
- **`candle-onnx`** : runtime Rust natif, compile wasm32, ~40-80 ops, **pas d'opérateur Attention / LayerNorm / DisentangledSelfAttention** → bloqueur factuel pour GLiNER/DeBERTa-v3.
- **`burn-onnx` v0.21** : runtime Rust natif, compile wasm32 via WGPU/NdArray, **218 opérateurs dont Attention, LayerNormalization, Einsum, GatherND** (meilleur opérateur-support Rust wasm du lot), 26 modèles validés mais **DeBERTa-v3/ModernBERT non listés explicitement dans le set validé** et backend en version 0.21 active development.

Aucune des 3 options n'a de précédent production publique pour GLiNER+DeBERTa-v3 en browser wasm. `gline-rs v1.0.1` (jan 2026) utilise **`ort` natif sans support wasm32** — confirmé par son `Cargo.toml`. L'écosystème converge vers « ORT runtime + onnxruntime-web pour le browser » et non vers un runtime pur-Rust wasm pour transformers.

Verdict Q7 : **Non-viable pour livrer en Phase B S21**. Option 7 (JS iframe + `onnxruntime-web` + Presidio coord-side) reste le bon choix court-terme. Ré-évaluation S22+ conditionnée.

### 2. Q1-Q6 détaillé

#### Q1 — `ort` (pyke) et target wasm32

- **Version courante** : `ort` v2.0.0-rc.12, publiée **5 mars 2026** (crates.io, docs.rs). Version ONNX Runtime sous-jacent : 1.24.4.
- **Target natif wasm32** : le crate `ort` lui-même **ne supporte pas `wasm32-unknown-unknown` comme cible directe** — ONNX Runtime est C++, difficultés de linking en wasm.
- **Support wasm via alternative backends** : depuis `ort` v2.0.0-rc.11, abstraction « alternative-backend » avec 3 implémentations :

| Backend | Version | Mécanique wasm | Statut |
|---|---|---|---|
| `ort-web` | 0.2.1+1.24 | Charge `onnxruntime-web` (bundle WASM Microsoft officiel) depuis `cdn.pyke.io` via wasm-bindgen | Experimental |
| `ort-candle` | 0.3.0+0.9.2 | Runtime candle pur-Rust compilé wasm32 | Experimental, operator-limited |
| `ort-tract` | Non vérifié version exacte | Runtime tract pur-Rust compilé wasm32 | Experimental, opérateurs limités aux ops tract |

Documentation officielle (`https://ort.pyke.io/backends`) : *« Alternative backends are experimental, and are constantly changing and growing -- use them at your own risk! »* Pour les 3 backends wasm.

- **Production users wasm+browser documentés** : aucun trouvé publiquement.

#### Q2 — ort backends wasm

Trois backends wasm existent (cf. Q1). Point important : **`ort-web` est littéralement une bridge Rust → JS `onnxruntime-web`** — il fetch le `onnxruntime-web.wasm` officiel Microsoft depuis `cdn.pyke.io` par défaut (self-hostable via struct `Dist`). Donc au runtime, vous exécutez **exactement le même binaire wasm que onnxruntime-web**, mais avec une API Rust au-dessus.

Limite documentée : *« two WASM contexts cannot directly share memory »* — l'onnxruntime-web wasm est un context isolé du wasm Rust généré par wasm-bindgen, donc chaque tenseur d'entrée doit être sérialisé entre les deux contextes. La doc recommande `Tensor::from_array` (au lieu de `Tensor::new`) pour éviter un double-copy.

Trois execution providers dans `ort-web` : WebGL, WebGPU, WebNN.

#### Q3 — `wasm32-unknown-unknown` vs `wasm32-wasi`

- **`wasm32-unknown-unknown`** : target wasm « browser-ready », pas de syscalls POSIX, compile-time pur — wasm-bindgen nécessaire. Targetté par tous les candle-wasm-examples et par `ort-web`.
- **`wasm32-wasip1` / `wasm32-wasip2`** (ex-`wasm32-wasi`) : wasm avec syscalls WASI, tourne dans des runtimes server-side (wasmtime, wasmer, Node.js). Pas utilisable directement dans un navigateur sans shim.

`ort-web` cible spécifiquement `wasm32-unknown-unknown`. `ort-candle` et `ort-tract` héritent de la capacité wasm du crate sous-jacent.

#### Q4 — candle-wasm et burn-wasm

**candle (HuggingFace)** :
- Support wasm32-unknown-unknown officiel + SIMD128 via `target-feature = ["+simd128"]`.
- `candle-wasm-examples/` publie 11 exemples browser : BERT, BLIP, Chat Template, Llama2-C, Moondream, Phi, Quant-Qwen3, Segment Anything, T5, Whisper, YOLO. **Aucun example DeBERTa/ModernBERT/GLiNER wasm** publié par HuggingFace.
- `candle-onnx` existe mais son eval.rs implémente **~40-80 opérateurs basiques** (Add, Mul, Conv, Gemm, MatMul, Gather, Reshape, Softmax, LSTM) — **aucun des opérateurs `Attention`, `LayerNormalization`, `ScaledDotProductAttention`, `DisentangledSelfAttention`**. Conclusion factuelle : **charger un GLiNER/DeBERTa-v3 ONNX directement dans `candle-onnx` échouera sur des ops manquantes**. Il faudrait soit (a) réimplémenter DeBERTa-v3 en code Candle natif, soit (b) décomposer les ops hautes-niveau dans l'export ONNX.

**burn (Tracel AI)** :
- `burn-onnx` v0.21, 26 modèles validés réels (image classification, object detection, depth estimation, NLP, speech, generative AI).
- **218 opérateurs supportés**, dont `Attention` ✅, `LayerNormalization` ✅, `MatMul` ✅, `Gather` ✅, `GatherND` ✅, `ScatterND` ✅, `Einsum` ✅, `Reshape` ✅, `Transpose` ✅. Couverture substantiellement meilleure que candle-onnx.
- Full opset compliance opsets 1-24. Active development (v0.21 est la current minor, 61 stars 20 forks).
- Flux attendu : GLiNER ONNX → `burn-onnx` codegen Rust → compile avec `burn` + backend NdArray (CPU wasm) ou WGPU (WebGPU browser) → wasm-bindgen bundle.
- `embed_states(true)` spécifiquement documenté pour deploy wasm single-binary.
- Backends wasm : NdArray (CPU/SIMD128) et WGPU. Pas de précédent public DeBERTa-v3.

#### Q5 — nexus-pii-rs pragmatique avec candle ou burn + GLiNER

**Avec candle-onnx** : bloqué. Sans opérateurs Attention/LayerNorm, l'import ONNX échouera au chargement. Effort S21 Phase B : **non réaliste** — port candle natif prendrait 2-4 semaines ingénieur senior.

**Avec burn-onnx** : techniquement possible. Bloqueurs potentiels :
1. DeBERTa-v3 pas dans les 26 modèles validés publiquement
2. ModernBERT backbone GLiNER a des issues ONNX export documentés (Issue #237 urchade/GLiNER 2025-03, Issue #35545 HF transformers Flash Attention 2.0)
3. burn-onnx v0.21 active development : stability risk
4. Aucun précédent public DeBERTa-v3 ou GLiNER en burn+wasm+browser

**Pattern « Candle + wasm + DeBERTa-v3 » publié 2024-2026** : aucun trouvé. Candle BERT wasm existe (radames/Candle-BERT-Semantic-Similarity-Wasm), mais BERT base classique, pas DeBERTa-v3 avec disentangled attention.

**GLiNER Rust Candle existante sur GitHub** : négatif. `gline-rs v1.0.1` (fbilhaut, janvier 2026, 127 stars) est la seule implémentation Rust GLiNER mainstream et utilise `ort = "=2.0.0-rc.9"` + `ort-sys` **sans feature wasm, sans feature alternative-backend**. Ses feature flags wrapent uniquement les execution providers natifs (CUDA, TensorRT, DirectML, CoreML, NNAPI, QNN, etc.). Conclusion : **le maintainer GLiNER Rust a explicitement choisi `ort` natif et n'a pas exploré wasm browser**.

#### Q6 — Benchmarks publics comparatifs

Aucun bench public 2025-2026 ne compare `onnxruntime-web` vs `ort-web` vs `candle-wasm` vs `burn-wasm` sur un modèle NER ~40-200 MB.

### 3. Verdict Q7

**Non-viable pour Phase B S21 (2 semaines)**. Trois raisons cumulées :

1. Pas de runtime Rust wasm suffisamment mature pour charger GLiNER/DeBERTa-v3 ONNX sans re-implémentation : candle-onnx manque les ops attention, burn-onnx est active-development 0.21 avec 0 précédent DeBERTa-v3 wasm public.
2. ort-web = onnxruntime-web déguisé : utiliser ort-web = embarquer le même wasm Microsoft que l'Option 7 JS, plus une couche Rust+wasm-bindgen qui double-copy les tenseurs entre deux wasm contexts. Pas de gain technique vs Option 7, surcoût latency, statut experimental.
3. Le seul précédent Rust GLiNER production (`gline-rs v1.0.1`, 127 stars, jan 2026) n'a pas activé wasm — validation factuelle que l'écosystème n'est pas prêt pour ce combo.

**Partiellement viable S22+ sous conditions strictes** (cf. §5).

### 5. Pourquoi Option 7 reste le bon choix S21 + conditions re-evaluate S22+

**Option 7 reste correct parce que** :
- `onnxruntime-web` est **le runtime qu'utilise `ort-web` en dessous** — le « moins Rust » des options est structurellement identique à l'option Rust la plus mature, sans l'abstraction experimental.
- Coord-side Presidio Python est le bon fallback défense-en-profondeur (deux runtimes indépendants = pas single-point-of-failure sur un seul runtime ML).
- Phase B 2 semaines = budget réaliste pour JS wrapper ~300-500 LOC + Presidio integration, pas pour exploratory Rust wasm ML engineering.

**Conditions exactes pour re-évaluer en S22+** (ordre de préférence) :

1. **`ort-web` sort du statut experimental** (annonce officielle pyke.io ou release notes v0.3+) **et** un projet tiers production publie un benchmark GLiNER+DeBERTa-v3 en ort-web+browser avec accuracy validée ≥ 98% vs reference PyTorch.
2. **`burn-onnx` atteint v1.0** (sortie de active-development 0.x) **et** tracel-ai ou une tierce partie publie un exemple officiel DeBERTa-v3 ou ModernBERT en burn-wasm-examples avec accuracy validée.
3. **Émergence d'un `gliner-candle` ou `gliner-burn` sur GitHub** avec wasm feature flag (équivalent Rust de `gline-rs` mais ciblant browser), prouvant que le pattern est maintenu par au moins une personne au-delà d'un prototype one-off.
4. **Microsoft publie un binding Rust officiel pour onnxruntime-web** (aujourd'hui seul `ort-web` pyke fait ça, non-officiel) — peu probable short-term mais changerait la donne supply-chain.

**Signaux de veille** à surveiller sans action immediate (budget ~15 min / sprint) :
- Tag release `pykeio/ort`, `pykeio/ort-web`, `tracel-ai/burn-onnx`
- Issues `urchade/GLiNER` sur ONNX export ModernBERT (Issue #237 tracking)
- Nouveau crate sur `crates.io` avec description « gliner + wasm » ou « presidio + wasm » ou « pii + rust + wasm »

**Rationale Option 7 robust à ce futur** : si l'un des signaux ci-dessus déclenche, basculer de JS wrapper vers Rust wrapper reste un changement interne au SDK `sbfb-pii-redact` côté iframe, n'affecte ni le wire format entre iframe et coordinator, ni la stack Presidio coord-side. **Option 7 n'interdit pas le pivot Rust futur** — elle le retarde au moment où l'écosystème sera prêt.

## Decision downstream

Rapport consommé par :
- `.planning/active/sprint21_carry_summary.md §2 Tech debt T-NN+2` — conditions re-evaluate Rust-wasm realignement S22+ précisées
- `.planning/active/sprint21_kickoff.md §Acknowledged review findings D2 ⚠️` — ack agent team fix 2026-04-18 comblant lacune G1 « custom Rust stack alternatives » non explorée
- `docs/claude/README.md §6.1.1 Regle renforcee custom Rust stack (G1 extension 2026-04-18)` — règle formalisée pour éviter lacune future

Verdict Option 7 confirmé factuellement sans re-ouvrir la décision. Tech debt T-NN+2 reste valide avec conditions re-evaluate précisées.
