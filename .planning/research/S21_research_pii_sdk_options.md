---
sprint: 21
topic: pii_sdk_options_pre_research_g2
date: 2026-04-18
agent: general-purpose (multi-turn, ~35 min research, context7 MCP + WebSearch)
prompt_source: transcript session orchestrateur 2026-04-18 turn 7 (pre-research G2 avant gel D1..D5)
word_count: ~2800
archive_rationale: §6.11 README — research output > 2000 mots fondant D2 PII SDK decision
---

## Prompt donné à l'agent

Pre-research G2 pour Sprint 21 nexus-grid. Mission : rassembler des faits vérifiables 2025-2026 via context7 MCP + WebSearch avant de figer D1..D5 Day-0 du sprint. **Ne propose pas de solution** — tu produis uniquement un rapport factuel avec sources (URL + version + date). Le planner tranchera.

Couvrir D1 rate-limit (governor vs tokio-rate-limit vs leaky-bucket vs async-rate-limiter vs axum_gcra vs tokio-rate-limit vs custom), D2 PII redaction SDK (Presidio vs scrubadub vs GLiNER vs llm-guard vs NeMo Guardrails vs redact-core vs Transformers.js vs Rust-native via tract/candle/burn/gliner-rs), D3 output filter (LLM Guard vs NeMo vs guardrails-ai + prompt echo algorithms EM/SM/EED vs Rabin-Karp), D4 quarantine queue (SQLite WAL vs BTreeMap vs gossip-wide + libp2p gossipsub v1.1 graylist precedent), D5 cap G7 vérification.

Garde-fous : sources primaires obligatoires (context7 + WebSearch), pas de supposition, dates absolues, versions précises (`governor = "0.6.3"` pas « governor latest »), alternatives concurrentes pour chaque décision. Format rapport ~2000-3000 mots structuré.

## Rapport reçu

### Flag roadmap S17 drift (items cités mais à reconsidérer)

La roadmap §3 S21 (écrite S17, 2026-04 circa) mentionne « spaCy NER wasm — ~500 LOC » pour la rédaction SDK client-side. Constat 2026-04 :

- spaCy n'a **pas** de distribution wasm officielle maintenue. Les tentatives communautaires (praeclarum/transformers-js, 2022) sont abandonnées. HuggingFace Transformers.js v4 (mars 2026, source: GitHub releases) **ne supporte pas GLiNER nativement** (issue #826 ouverte 2024-06-28, non résolue).
- L'approche moderne 2025-2026 pour NER browser : **GLiNER.js** comme package séparé via ONNX Runtime Web (execution providers `wasm`, `webgpu`, `webgl`), **pas** spaCy.
- Alternative Rust-native : `redact-core` + `redact-ner` (docs 2026-02) ciblent wasm + ONNX quantized int8.

**Conséquence planner** : le libellé « spaCy NER wasm » est factuellement obsolète. À requalifier avant code.

Autre flag : la roadmap ligne 222 « Dependencies: S19 PoW (sinon rate-limit contourne) » — confirmé par les patterns libp2p gossipsub v1.1 (P6 application-specific + P4 invalid msg penalty), mais le PoW runtime wire a été livré en S20 Phase C (`16b94ba`), donc le prérequis est **satisfait** au moment d'entrée S21.

### D1 — Rate-limit sliding-window implementation (Rust)

| Nom | Version | Date release | Licence | Algorithme | Audit public | Maintenu | Notes |
|---|---|---|---|---|---|---|---|
| `governor` | 0.10.2 | 2025-11-13 | MIT | GCRA (Generic Cell Rate Algorithm) | Non (RustSec : aucune advisory trouvée 2025) | Oui (0.10.0 2025-03-27, 0.10.1 2025-08-06, 0.10.2 2025-11-13) | Fork de ratelimit_meter. State 64 bits atomic CAS. `DefaultKeyedRateLimiter` via DashMap. `retain_recent()` + `shrink_to_fit()` pour éviction. |
| `tower_governor` | 0.8.0 | — (date non confirmée, axum 0.8 compat) | MIT | Wrapper governor (GCRA) | Non (reputation « High » sur context7) | Oui | 3 key extractors : PeerIp / SmartIp / Global. Custom key extractor possible. |
| `ratelimit_meter` | archivé | — | — | Leaky bucket + GCRA | — | **Non** — officiellement redirigé vers governor (« f.k.a. ratelimit_meter » dans tagline) | Migration guide fournie. |
| `leaky-bucket` | docs.rs live | 2025+ | MIT/Apache | Leaky bucket pur | Non | Oui | Pas de background task (waiting task assume coordinator role). Pas de keyed par défaut. |
| `async-rate-limiter` | 1.x (mindeng) | — | MIT | Token bucket | Non | Oui | Pure async/await. Pas de keyed. `try_acquire()` retourne duration to wait. |
| `tokio-rate-limit` | 0.8.0 | 2025-11 | MIT | Pluggable (token bucket + leaky bucket) | Non | Oui | **Per-key niche ciblée** explicitement vs governor (global). Zero allocation hot path. 17-20M ops/sec single-thread. Axum 0.8.6 middleware. |
| `rate_limiters` | crates.io | — | MIT | Token / Leaky / Sliding Window / Fixed Window | Non | — | Mentionné sans stars / maintainership visible. |
| `axum_gcra` | docs.rs live | — | MIT | GCRA | Non | — | Focus Axum uniquement. |
| `axum-governor` (canmi21) | docs.rs live | — | MIT | via lazy-limit | Non | Oui | **Non lié à tower-governor** (explicite dans README). GC memory management. |
| `limitr` (Arteiii) | GitHub | — | MIT | Token / Leaky / plusieurs | Non | Mineur | Faible visibilité. |

**Observations factuelles D1** :
- Aucun crate Rust de rate-limiting n'a d'audit tiers public (Cure53 / ToB / NCC) trouvé via WebSearch 2025-2026. Aucune advisory RUSTSEC concernant governor/leaky-bucket/tokio-rate-limit identifiée (best-effort search).
- GCRA implémenté uniquement par governor, tower_governor (wrap governor) et axum_gcra. Sliding-window pur seulement dans rate_limiters.
- Cardinalité per-key : governor utilise DashMap sans TTL automatique — éviction manuelle via `retain_recent()` + `shrink_to_fit()`. Pour 10k+ buckets, pas de limite structurelle mais nettoyage explicite nécessaire.
- Thread-safety / async : governor supporte sync + async + stream/sink combinators.
- Axum/tower ready : tower_governor (0.8) et axum_gcra + tokio-rate-limit Axum middleware natif.

### D2 — PII redaction SDK (Python ± Rust-native wasm)

| Nom | Version | Date release | Licence | Approche | Maintenu | PII supportés | Notes |
|---|---|---|---|---|---|---|---|
| `presidio-analyzer` | 2.2.362 | 2026-03-15 (via PyPI) | MIT | NER (spaCy/Stanza/transformers) + regex recognizers | Oui (releases continues, 2.2.358 → 2.2.362) | Email, phone, CC, SSN, IBAN, US MBI (Medicare Beneficiary ID récent), GLiNER extension | Requires spaCy pour capabilities non-NER même avec transformers engine. Pas de port Rust/wasm officiel. Memory ~300 MB. Python >=3.10 <3.14. |
| `presidio-anonymizer` | 2.2.x | aligné analyzer | MIT | Rule-based redaction/mask/hash/encrypt | Oui | — | Complément de analyzer. |
| `scrubadub` | 2.0.1 | — (docs RTD 2.0.0) | Apache-2.0 | Regex + optionnels (spacy, stanford, address) | Oui (LeapBeyond fork active) | Names, emails, phones, SSN, CC | Python >=3.6. Extras via `scrubadub_spacy`, `scrubadub_stanford`, `scrubadub_address`. FP/FN benchmarks non publiés dans search results. |
| `pii-codex` | (PyPI live) | — | Apache-2.0 | Wrapper Presidio + severity scoring 1-3 (Schwartz/Solove) | Actif (JOSS paper ref) | Tout Presidio + catégorisation | Research package. Ajoute scoring non-identifiable / semi / identifiable. |
| `GLiNER` (urchade) | 0.2.5+ | PyPI + huggingface models (knowledgator/gretel) 2024-2025 | Apache-2.0 | Bidirectional transformer zero-shot NER | Oui (GLiNER2 dev) | 60+ PII categories (gliner_multi_pii-v1, gretel variants small/base/large, knowledgator edge/base/large) | S'intègre Presidio via `GLiNERRecognizer` (extra `pip install 'presidio-analyzer[gliner]'`). |
| `redact-core` / `redact-ner` (censgate) | crates.io live (docs 2026-02) | 2026-02-02 à 2026-02-07 | Apache-2.0/MIT | Regex + ONNX quantized int8 NER | Oui (nouveau) | 36 pattern-based entity types + transformer NER | Rust-natif. Annonce « replacement for Microsoft Presidio », 20-50 MB vs 300 MB Presidio, native + **wasm** + CLI, inférence 2-10 ms/text. |
| `pii-vault` | lib.rs live | — | — | Regex + zero-dépendances | — | — | Presidio-compatible. Reversible tokenization. Zero deps beyond regex + json. |
| `Transformers.js` v4 | 4.0.0 | 2025-03 (dev start) → NPM 2026 | Apache-2.0 | ONNX Runtime Web (wasm/webgpu/webgl) | Oui (HF) | Via modèles HF | **Ne supporte pas GLiNER nativement** (issue #826 open). |
| `GLiNER.js` | npm (socket.dev ref) | — | Apache-2.0 | ONNX direct browser (wasm/webgpu/webgl) | — | GLiNER zero-shot | Alternative browser-native à Transformers.js pour GLiNER. |
| `wonnx` (webonnx) | @webonnx/wonnx-wasm | — | MIT | WebGPU inference Rust + wasm | — | Any ONNX | Pas PII-spécifique. Rust-written, wasm target. ARCHIVÉ 2025-05-07. |
| `tract` (Sonos) | docs.rs live | — | MIT/Apache | ONNX + TF inference pure Rust | Oui | — | Pas de dépendances C++, wasm-friendly. Opset 9-18 testé. |
| `candle` (HuggingFace) | crates.io live | 2024-2025 | Apache-2.0 | ML framework Rust minimaliste | Oui (HF) | — | wasm inference path validé. Ecosystème HF. `candle-onnx` manque ops Attention/LayerNorm. |
| `burn` (tracel-ai) | crates.io live | 2024-2025 | Apache-2.0 | ML framework Rust (NdArray/WGPU/CUDA/WASM) | Oui | — | `burn-onnx` crate pour import ONNX, 218 ops dont Attention. Active development v0.21. |

**Observations factuelles D2** :
- **Presidio = standard de facto** (stars en milliers, releases mensuelles 2025-2026). Poids lourd (300 MB memory, spaCy obligatoire même avec transformers engine).
- **spaCy wasm direct** : pas disponible comme distribution officielle. Les chemins wasm passent par ONNX Runtime Web + modèles quantized, pas par spaCy lui-même.
- **GLiNER = alternative zero-shot** : bidirectional transformer, modèles PII fine-tunés disponibles (gretel, knowledgator, NVIDIA). Presidio intègre via `GLiNERRecognizer`.
- Benchmarks FP/FN publiés : OneShield IBM arXiv 2501.12456 janvier 2025 annonce F1 95 % multilingual dépassant Presidio « up to 12 % » — non indépendant.
- **Rust-native 2026** : redact-core (censgate) annonce remplacement Presidio avec 36 entity types + NER + wasm. Écrit récemment (docs 2026-02). Pas d'audit public.
- Licences : majoritairement MIT/Apache-2.0 — pas de blocker AGPL pour SDK.
- Audit externe : aucun des libs PII identifiés n'a d'audit tiers publié.

### D3 — Output filter (prompt echo + beacon chars)

| Nom | Version | Date release | Licence | Approche | Maintenu | Notes |
|---|---|---|---|---|---|---|
| `llm-guard` | 0.3.16 | — (PyPI) | MIT | 15 input scanners + 20 output scanners | Oui (docs timestamp mai 2025) | **`InvisibleText` scanner** explicite pour zero-width + PUA. Sensitive scanner utilise Presidio. |
| `NeMo Guardrails` | 0.11.0, 0.12.0 à venir | 0.11 = 2025 (Langchain 3 upgrade, Python 3.8 dropped). Releases page 2026-03-12. | Apache-2.0 | Input/dialog/retrieval/execution/output rails + IORails parallel engine | Oui (NVIDIA) | Colang 2.0 devient default en 0.12. |
| `guardrails-ai` (framework + Hub) | — | — | Apache-2.0 | Composable Guard pipeline de validators | Oui | Hub = 60+ validators : `detect_prompt_injection` (Rebuff, 2024-02-15), `unusual_prompt`, LLM-based injection detection. |
| `OpenAI Guardrails Python` | — | — | — | Prompt injection detection check | — | Documenté openai.github.io/openai-guardrails-python. |

**Prompt echo detection — état de l'art 2024-2025** :
- **PLeak** (Hui et al., CCS'24, oct 2024, ACM DOI 10.1145/3658644.3670370) définit 3 métriques d'évaluation : Exact Match (EM), Substring Match (SM), Extended Edit Distance (EED, Levenshtein-based).
- **ProxyPrompt** (mai 2025, arXiv 2505.11459) : défense atteignant 94.70 % protection vs 42.80 % baseline.
- **System Prompt Extraction Attacks and Defenses** (mai 2025, arXiv 2505.23817) : survey récent.
- **OWASP LLM Top 10 2025** : System Prompt Leakage = rang #7.
- **Rabin-Karp rolling hash** : pas de paper dédié prompt-leak 2024-2025 trouvé.

**Beacon / invisible chars — état de l'art 2024-2025** :
- Unicode categories dangereuses : `Cc` (Control), `Cf` (Format), `Co` (Private Use), `Cs` (Surrogate).
- Caractères steganography courants : U+200B-U+200D, U+202C, U+FEFF, U+2800, U+3164, Tag chars U+E0020-U+E007F.
- PUA U+E000-U+F8FF exploité par « invisible prompt » attacks.
- arXiv 2512.13325 (décembre 2025) analyse 10 méthodes watermarking Unicode.
- LLM Guard `InvisibleText` scanner — explicite, maintenu.

### D4 — Quarantine queue design

| Option | Prérequis | Pattern existant SBFB | Persistance | Notes |
|---|---|---|---|---|
| SQLite WAL local | `rusqlite`/`sqlx` déjà dans workspace | **Oui — S19 Phase D delayed upload queue (`f238d31`)** | Oui (disque) | Pattern validé in-house. Multi-writer sérialisé, multi-reader concurrent. |
| In-memory BTreeMap TTL | Aucun | Non | Non (volatile) | Lock-protected. Perte au crash. |
| Gossip-wide diffusion | Quorum-based | Non | Via consensus | Complexité + nouveau DoS vector. |

**Patterns P2P établis** :
- libp2p gossipsub v1.1 définit 3 actions validation : `Accept` / `Reject` (P4 penalty) / `Ignore`. Pas de « quarantine hold 15 min » explicite. Le plus proche est peer score threshold system (graylist automatic).
- Tor DoS prevention : cooldown per-client address.
- Filecoin / ETH2.0 gossipsub (Vyzovitis et al. 2020) : attack-resilient mesh, pas de manual flush.
- AztecProtocol issue #10347 : override topicValidators pattern application-level validator.

**Conclusion factuelle** : le pattern « quarantine queue hold 15 min + manual flush » n'a **pas d'équivalent exact** dans libp2p/Tor/Freenet/Filecoin publiés. Design SBFB-spécifique.

### D5 — Cap G7 carry-overs (vérification, non-research)

Avec 2 carries retenus (Meta-1 + C-PLAN-1), cap G7 = 2/2 respecté. Tech debt E-1/E-2 hors cap (PATTERNS.md).

### Récap drifts identifiés entre roadmap S17 (§3 S21) et état 2026-04

| Élément roadmap | Formulation écrite S17 | Factuel 2026-04 |
|---|---|---|
| « spaCy NER wasm » | Cité comme candidate tech pour redaction SDK client | spaCy n'a pas de port wasm maintenu. Alternatives : GLiNER.js via ONNX Runtime Web, ou redact-core/redact-ner Rust→wasm. |
| « S19 PoW dependency » | « Sinon rate-limit contourné » | Satisfait : runtime wire livré S20 Phase C `16b94ba`. |
| « quarantine queue gossip 15 min manual flush » | Pattern cité comme si établi | Pas d'équivalent public P2P établi. Design SBFB-spécifique. |

### Zones non-vérifiables / « non trouvé »

- Audits tiers publics : aucun des crates rate-limit ni des libs PII n'a d'audit Cure53/ToB/NCC.
- Benchmarks FP/FN indépendants des libs PII : ceux publiés sont internes (OneShield = IBM, Gretel = Gretel).
- Release date `tower_governor` 0.8 : non confirmée précisément.
- Stars GitHub exacts : « thousands » pour Presidio / LLM Guard sans chiffre précis.
- CVE rate-limiting crates 2025 : best-effort search, pas exhaustif.

## Decision downstream

Rapport consommé par `.planning/active/sprint21_kickoff.md §D1-D5` (research list + alternatives comparées + rejets documentés). Decisions finales gelées post-arbitrage user multi-tours :
- D1 : governor 0.10.2 + tower-governor 0.8
- D2 : Option 7 (custom JS iframe + Presidio coord-side) — voir aussi `S21_research_rust_first_alignment.md` pour analyse Rust-first complémentaire et `S21_research_ort_wasm_alternatives.md` pour re-check post-G1
- D3 : LLM Guard 0.3.16 InvisibleText + EED prompt echo
- D4 : SQLite WAL pattern S19 reuse + CLI manual flush
- D5 : cap G7 2/2 confirmé (Meta-1 + C-PLAN-1)
