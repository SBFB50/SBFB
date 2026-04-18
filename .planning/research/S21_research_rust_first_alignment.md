---
sprint: 21
topic: rust_first_alignment_analysis_d2_pii
date: 2026-04-18
agent: general-purpose (Explore deep research, ~60 min, context7 MCP + WebFetch + WebSearch)
prompt_source: transcript session orchestrateur 2026-04-18 turn 12 (user challenge "en toute objectivité, utilise context7")
word_count: ~2300
archive_rationale: §6.11 README — research output > 2000 mots fondant reject Rust-first iframe pour D2
---

## Prompt donné à l'agent

Analyse factuelle approfondie pour arbitrer un choix d'architecture SDK PII redaction dans nexus-grid (Rust+Python hybride Option G). 3 options :

1. Full Rust-first (custom `nexus-pii-rs` + tract + GLiNER ONNX + wasm-bindgen + PyO3)
2. Hybrid Rust-wasm iframe + Presidio Python coord-side
3. Pragmatique JS+Python (GLiNER.js + Presidio)

Questions factuelles : tract status 2026, ONNX Runtime Web 2026, GLiNER modèles PII, HF tokenizers Rust, wasm-bindgen maturity, Presidio status, GLiNER.js status, projets similaires Rust-first wasm NER, candle vs tract vs burn, effort estimatif.

Garde-fous : versions précises, dates absolues, sources primaires, pas d'opinion — faits. Format rapport ~3000-4500 mots.

## Rapport reçu

### 1. Executive summary factuel

- **`gline-rs` existe déjà** (crates.io, v1.0.1 10/01/2026, Apache-2.0, production-ready) mais utilise `ort` (wrapper ONNX Runtime Microsoft) et **aucun support wasm32 documenté** — Cargo.toml a 0 configuration wasm.
- **tract (Sonos) version courante 0.23.0-dev.3** (20/03/2026, stable 0.22.1 23/02/2026 / 0.21.15 09/03/2026) teste ONNX **opset 9 → 18 uniquement**. GLiNER-PII exporte typiquement opset 19. Pas de liste production-users publique hors Sonos/Snips/BlindAI.
- **GLiNER models candidates** : `knowledgator/gliner-pii-edge-v1.0` = 45.8 MB quint8 (backbone ModernBERT `ettin-encoder-32m`), `gretel-gliner-bi-small-v1.0` = DeBERTa-v3-small (pas de ONNX publié), `urchade/gliner_multi_pii-v1` onnx-community = 349 MB int8 (mDeBERTa-v3-base), `nvidia/gliner-PII` = 570M params NVIDIA Open License.
- **ONNX Runtime Web 1.24.3 (mars 2026)** WASM binary ~20 MB default build. WebGPU/WebNN experimental.
- **Presidio 2.2.362 (18/03/2026)** MIT, 7.2k stars, intègre GLiNER via `[gliner]` + `GLiNERRecognizer` recommandant `urchade/gliner_multi_pii-v1`. Paper IBM arXiv 2501.12456 chiffre Presidio ~12% derrière OneShield sur certaines entités.
- **Gretel GLiNER fine-tuned F1 = 0.94-0.95** vs Knowledgator base F1 = 0.81 et Edge F1 = 0.755.
- Pas de projet production OSS combinant `tract + wasm + GLiNER` trouvé. Le précédent le plus proche : BlindAI (Mithril Security) porté tract à SGX Rust ; tokenizers-wasm (Mithril) porté HuggingFace tokenizers à WASM.
- **`tokenizers` crate v0.22.2 (02/12/2025), feature `unstable_wasm` explicitement marquée « expérimentale »**.

### 2. Q1-Q10 détaillé

#### Q1 — tract (Sonos) status 2026

| Fait | Valeur | Source |
|---|---|---|
| Version dev | 0.23.0-dev.3 (20/03/2026) | lib.rs/crates/tract |
| Version stable | 0.22.1 (23/02/2026), 0.21.15 (09/03/2026) | crates.io/crates/tract |
| Licence | Apache-2.0 / MIT dual | lib.rs |
| Maintainer | Mathieu Poumeyrol (kali) + 73 contrib | lib.rs |
| ONNX opset | **1.4.1 → 1.13.0 (opset 9-18)** | github.com/sonos/tract README |
| Wasm target | **wasm32-wasi** via wasmtime | examples/onnx-mobilenet-v2/README |
| wasm32-unknown-unknown (browser) | **Non documenté explicitement** | Recherche exhaustive |
| SIMD | Linalg ARM NEON/AVX, pas SIMD128 wasm | Repo README |
| Production users | Sonos, Snips (wake word RPi Zero), ARM ML-KWS, BlindAI (2022-2023), Beckhoff TwinCAT 3 | Sonos Tech Blog + Mithril blog |
| DeBERTa-v3 | **Pas de test publié, pas d'exemple officiel** | Recherche Q1 + Q3 |

#### Q2 — ONNX Runtime Web status 2026

| Fait | Valeur | Source |
|---|---|---|
| Version courante | 1.24.3 (mars 2026) | npm onnxruntime-web |
| Version ORT parente | 1.24.4 (17/03/2026) | GitHub releases |
| EP WebAssembly (CPU) | Stable, SIMD+threaded par défaut depuis v1.19 | issue #25666 |
| EP WebGPU | **Experimental** | onnxruntime.ai WebGPU tutorial |
| EP WebNN | **Experimental** | onnxruntime.ai WebNN tutorial |
| Bundle wasm full | ~20 MB avec WebGPU default build | discussion #24161 |
| Bundle wasm minimal | Significativement plus petit, ORT-format-only | build docs |
| Cross-browser WebGPU 2026 | Chrome/Edge stable, Firefox progressif, Safari 18+ | Non confirmé par source primaire |
| Perf NER ~40 MB int8 | **Non trouvé benchmark chiffré public 2024-2026** | « non vérifié » |

#### Q3 — GLiNER modèles PII candidates

| Modèle | Backbone | Taille ONNX (quantisé) | Tokenizer | Licence | Date | F1 | Entités |
|---|---|---|---|---|---|---|---|
| `knowledgator/gliner-pii-edge-v1.0` | **ModernBERT** (`jhu-clsp/ettin-encoder-32m`), 32M params, hidden 384, 10 layers | 181 MB fp32 / 90.8 MB fp16 / **45.8 MB quint8** | Non listé séparément (ModernBERT WordPiece) | Apache-2.0 | 29/01/2024 | **0.755** | 60+ (name, email, phone, SSN, credit card, passport, medical...) |
| `knowledgator/gliner-pii-base-v1.0` | N/A dans config (probable DeBERTa-v3-base d'après pattern) | 197 MB uint8 / 330 MB fp16 | N/A | Apache-2.0 | — | **0.810** | 60+ |
| `knowledgator/gliner-pii-large-v1.0` | N/A | N/A | N/A | Apache-2.0 | — | **0.833** | 60+ |
| `gretelai/gretel-gliner-bi-small-v1.0` | **`microsoft/deberta-v3-small`** (768 hidden, 6 layers, 12 heads) + labels encoder MiniLM-L6-v2 | **Pas de ONNX publié sur la HF model card** | SentencePiece (DeBERTa) | Apache-2.0 | 10/2024 | **0.94** | 41 (PHI inclus) |
| `gretelai/gretel-gliner-bi-base-v1.0` | DeBERTa-v3-base probable | Pas de ONNX publié | SentencePiece | Apache-2.0 | 10/2024 | **0.95** | 41 |
| `urchade/gliner_multi_pii-v1` | **`microsoft/mdeberta-v3-base`** multilingue | 1.16 GB fp32 / 580 MB fp16 / **349 MB int8** / 349 MB quantized / 472 MB q4f16 | SentencePiece mDeBERTa | Apache-2.0 | 14/11/2023 | Non chiffré direct | 50+ (multilingue FR/EN/ES/DE/IT/PT) |
| `nvidia/gliner-PII` | Base `urchade/gliner_large-v2.1` | **Pas de ONNX mentionné** | SentencePiece | NVIDIA Open Model License (**pas Apache/MIT**) | 28/10/2025 | Argilla 0.70 / AI4Privacy 0.64 / Nemotron 0.87 | 55+ |

**Corollaire** : les modèles « DeBERTa-v3 » (gretel, multi-pii) supposent support tract opset 19. Modèle `gliner-pii-edge` évite DeBERTa via ModernBERT.

#### Q4 — HuggingFace `tokenizers` Rust crate 2026

| Fait | Valeur | Source |
|---|---|---|
| Version courante | 0.22.2 (02/12/2025) | lib.rs/crates/tokenizers |
| Licence | Apache-2.0 | lib.rs |
| Maintainer | HuggingFace officiel | lib.rs |
| Downloads mensuels | ~1.6M, 851 downstream crates | lib.rs |
| Feature wasm | **`unstable_wasm` — explicitement « experimental »** | lib.rs |
| Port WASM officiel | `tokenizers-wasm` (Mithril), wrapper npm `huggingface-tokenizers-bindings` | Mithril blog |
| Approach Mithril | Désactive filesystem, remplace regex C par Rust alternatives, wasm-bindgen contourne traits/generics | Mithril blog |
| Bundle size final | **Non publié dans le post** | « non vérifié » |
| SentencePiece (DeBERTa) | Supporté natif crate | docs.rs tokenizers |
| Perf | <20s par GB sur CPU serveur | Mithril blog |

#### Q5 — wasm-bindgen maturity 2026

| Fait | Valeur | Source |
|---|---|---|
| Version courante | 0.2.118 (10/04/2026) | crates.io history |
| Releases 2026 | 0.2.107 → 0.2.118, ~1 release/2 semaines | Même source |
| Maintainer | rustwasm org (semi-officiel Rust) | rustwasm.github.io |
| Memory64 browser | **Disponible 2026**, accès jusqu'à 16 GB | dev.to Rust & WASM in 2026 |
| Threads | `wasm-bindgen-rayon` via Web Workers + SharedArrayBuffer | GoogleChromeLabs/wasm-bindgen-rayon |
| Threads require | COOP + COEP = cross-origin isolation | v8.dev 4gb-wasm-memory |
| CSP iframe sandbox | `wasm-unsafe-eval` directive requise | MDN CSP |

#### Q6 — Presidio analyzer 2.2.362 status

| Fait | Valeur | Source |
|---|---|---|
| Version | 2.2.362 (18/03/2026) | PyPI |
| Python | 3.10 → 3.13 | PyPI |
| Licence | MIT | PyPI |
| Stars GitHub | 7.2k (fev 2026) | GitHub microsoft/presidio |
| Commits | ~1490, 47 releases | GitHub |
| Extras | ahds, azure-ai-language, gliner, langextract, server, stanza, transformers | PyPI |
| NER engines | spaCy obligatoire, Stanza, Transformers (HF), GLiNER (depuis 2.2.361), LangExtract (depuis 2.2.361) | Installation docs |
| Modèle GLiNER recommandé | **`urchade/gliner_multi_pii-v1`** | GLiNER sample |
| Benchmark Presidio vs OneShield IBM | Person 0.91 / Date 0.72 / Email 0.85 / Phone 0.77 / Location 0.81 vs OneShield D1 : 0.98, 0.96, 0.94, 0.89, 0.91 | arXiv 2501.12456 |
| Benchmark Gretel | F1 = 0.94-0.95 (bi-small/base/large) | Gretel blog |

#### Q7 — GLiNER.js npm package status

| Fait | Valeur | Source |
|---|---|---|
| Package npm | `gliner` | npm gliner via Socket |
| Dernière version | ~mars 2025, inactif | Socket + WebSearch |
| Weekly downloads | **7650** | Socket |
| Execution providers | cpu, wasm, webgpu, webgl | Socket |
| Modèle par défaut | `onnx-community/gliner_small-v2` | Socket |

#### Q8 — Projets similaires pari Rust-first wasm NER ML inference

| Projet | Pattern | Retour d'expérience | Source |
|---|---|---|---|
| **BlindAI (Mithril Security)** | Rust custom + **tract** + ONNX + Intel SGX server-side | « Pure Rust backend leveraging Rust SGX SDK ; ran Wav2vec2, ResNet, BERT » | Mithril blindai post 2022 |
| **Mithril tokenizers-wasm** | Rust HF tokenizers → wasm via wasm-bindgen → npm | Issues : traits/generics impossibles direct, wrapper JS nécessaire | Mithril porting tokenizers post |
| **parry-guard** (vaporif) | Rust CLI, DeBERTa-v3 prompt injection via Candle OR ONNX | « Candle 5-6x plus lent que ONNX » — non-wasm, daemon natif | github vaporif/parry-guard |
| **gline-rs** (fbilhaut) | Rust custom + **ort** (pas tract) + tokenizers + ndarray, server/CLI | « 4x faster than Python GLiNER on CPU, 248 seq/s sur RTX 4080 » | github fbilhaut/gline-rs |
| **Transformers.js (HF)** | JS + ONNX Runtime Web + WebGPU runtime C++ v4 | « 60 tok/s Llama 3.2 3B en browser ; data never leaves device » | HF transformersjs-v4 blog |
| **Cloudflare Infire** | Rust custom LLM serving, serveur edge | « 20% latency reduction from Lua → Rust » | Cloudflare blog |
| **wonnx** (webonnx) | Rust + WebGPU pure | **Archived 07/05/2025** — pas attention / LSTM / GRU / dynamic dims | github webonnx/wonnx |

**Finding majeur** : **aucun projet production open source ne combine `tract + GLiNER + wasm browser`**. Le seul précédent similaire en threat model (privacy-first) est BlindAI mais côté serveur SGX, pas browser. `tokenizers-wasm` Mithril est le précédent le plus pertinent pour l'étape tokenizer.

#### Q9 — candle vs tract vs burn

| Dimension | tract | candle | burn |
|---|---|---|---|
| Version | 0.23.0-dev.3 / 0.22.1 (02/2026) | Active | 0.15.0 (2026) |
| Maintainer | Sonos | HuggingFace officiel | Tracel AI |
| Wasm target | wasm32-wasi documenté (pas UUU explicite) | wasm32-unknown-unknown avec SIMD128 explicite (Qwen3) | wasm32 via Candle / NdArray / WGPU |
| ONNX import | Natif, opset 9-18 testé | `candle-onnx` subset opérateurs | `burn-onnx` convertit ONNX → code Rust, 26 modèles validés |
| SIMD | ARM NEON/AVX, pas wasm SIMD128 | Wasm SIMD128 activé explicitement | WGPU (WebGPU) backend |
| Perf vs PyTorch | Optimisé inference | 3-4x slower GPU vs PyTorch, 5-6x slower ONNX Runtime natif | 98% PyTorch CPU, 92% GPU |
| Use cases publics | Snips/Sonos, ARM ML-KWS, BlindAI, Beckhoff | HF Candle Wasm Examples, parry-guard, Qwen3 browser | burn-onnx 26 models CV/NLP/ASR/GenAI |

#### Q10 — Effort / LOC rétrospectif sur projets comparables

Aucune LOC exacte publique trouvée. Mesure a posteriori seulement possible par tokei après implémentation.

### 3. Matrice de faits pour/contre chaque option

#### Option 1 — Full Rust-first (`nexus-pii-rs` via tract + GLiNER ONNX + wasm-bindgen + PyO3)

**Faits qui la soutiennent** :
- Cohérence Option G (projet a déjà crates Rust + PyO3 + wasm-ready pattern via loopback daemon)
- `tract` Apache/MIT + pure Rust = zero native dep = clean wasm32 path
- `tokenizers` crate HF officiel avec `unstable_wasm` feature + port Mithril existant → précédent
- GLiNER-PII-edge 45.8 MB quint8 (ModernBERT backbone) potentiellement compatible tract opset 18
- wasm-bindgen 0.2.118 + memory64 2026 → jusqu'à 16 GB
- Modèle Apache-2.0 compatible AGPL-3.0 nexus-grid
- Aucun dependency Python/Microsoft runtime sur iframe → surface d'attaque minimale

**Faits qui la contredisent** :
- `tract` teste opset 9-18 seulement, **GLiNER exporte typiquement opset 19**
- **Wasm32-unknown-unknown (browser) pas documenté** par tract officiellement
- Aucun projet OSS précédent n'a combiné tract + GLiNER + wasm browser
- `tokenizers` feature `unstable_wasm` explicitement « expérimentale »
- LOC effort ouvert — pas de gline-rs-wasm précédent pour forker
- Bench tract vs ORT sur modèles NER transformer ~40 MB non publié
- Candle (alternative Rust) bench 5-6x plus lent que ONNX natif (parry-guard)
- iframe sandbox CSP : `wasm-unsafe-eval` requis

#### Option 2 — Hybrid Rust-wasm iframe + Presidio Python coord-side

**Faits qui la soutiennent** :
- Client-side : mêmes avantages Option 1 pour iframe
- Coord-side : Presidio 2.2.362 MIT mature (7.2k stars, actif 03/2026, 47 releases)
- Presidio déjà intégré GLiNERRecognizer + recommande modèle `urchade/gliner_multi_pii-v1` + ONNX Runtime backend (depuis 2.2.361 2/2025)
- Benchmark Gretel GLiNER (F1 0.94-0.95) exploitable coord-side
- Coordinator FastAPI Python déjà présent → intégration drop-in
- Permet double redaction (belt-and-suspenders)

**Faits qui la contredisent** :
- Double effort : client-side Rust/wasm + coord-side Presidio
- Surface d'attaque augmentée : dépendance Python Presidio + spaCy + transformers
- Paper IBM OneShield chiffre Presidio en retard +12% gap, Presidio False Positives documentés
- Presidio spaCy obligatoire même en mode transformers → overhead mémoire coord
- Python coord-side taille env typique importante (spaCy + transformers + torch)

#### Option 3 — Pragmatique JS + Python (GLiNER.js + Presidio)

**Faits qui la soutiennent** :
- `onnx-community/gliner_multi_pii-v1` ONNX déjà publié pour Transformers.js
- `GLiNER.js` npm package existe, 7650 weekly DL
- Transformers.js v4 (02/2026) momentum production-privacy-first
- Presidio coord-side : voir arguments Option 2
- ONNX Runtime Web 1.24.3 SIMD+threaded par défaut
- Chemin le mieux documenté avec precedents multi-producteurs

**Faits qui la contredisent** :
- **Déroge Option G**
- GLiNER.js last publish ~mars 2025 (1 an d'inactivité)
- ONNX Runtime Web wasm bundle ~20 MB default
- Dépendance JS supply-chain (onnxruntime-web, gliner, Transformers.js)
- WebGPU et WebNN EP still experimental
- Modèle 349 MB int8 multi-PII lourd pour iframe cold-start
- Iframe sandbox : GLiNER.js non testé spécifiquement sous sandbox
- Pas d'économie côté Python : Presidio coord-side = mêmes contre que Option 2

### 4. Zones non-vérifiables / « non trouvé »

1. Bench perf tract vs ONNX Runtime Web sur modèles transformer NER ~40-200 MB int8 en 2024-2026.
2. Support tract pour DisentangledSelfAttention de DeBERTa-v2/v3.
3. Support tract pour wasm32-unknown-unknown (browser).
4. Bundle size final tokenizers-wasm Mithril.
5. Bench F1 direct Presidio standalone vs Gretel GLiNER sur dataset commun 2025-2026.
6. Taille environnement Python complet Presidio + GLiNER en MB en 2026.
7. nvidia/gliner-PII ONNX format disponibilité.
8. Taille `gretelai/gretel-gliner-bi-small-v1.0` en MB et disponibilité ONNX publique.
9. NVIDIA Open Model License compatibilité avec AGPL-3.0 nexus-grid.
10. CSP `sandbox="allow-scripts"` sans `allow-same-origin` + chargement modèle 45 MB dans IndexedDB.

## Decision downstream

Le planner (orchestrateur) a émergé d'abord « Option 2 Hybrid Rust-first » comme recommandation puis, sur challenge user « en toute objectivité, utilise context7 », ce rapport a révélé **les blockers factuels** :
1. tract opset 9-18 vs GLiNER opset 19
2. tract wasm32-unknown-unknown non documenté officiellement
3. Zero precedent production tract+GLiNER+wasm browser
4. gline-rs v1.0.1 mainstream Rust GLiNER a choisi ort pas tract

Decision finale : Option 3 (Option 7 dans kickoff numbering) — JS iframe + Presidio coord. Tech debt T-NN+2 S22+ pour re-alignement Rust-wasm quand blockers levés. Cf. `sprint21_kickoff.md §D2` + `sprint21_design_review.md §D2` + archive complémentaire `S21_research_ort_wasm_alternatives.md` pour re-check post-G1.
