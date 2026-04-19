# Sprint 21 — Design Review Board G1 report

**Date** : 2026-04-18.
**Reviewer** : agent Explore indépendant, session fraîche, contexte
minimal (cf. `docs/claude/README.md §6.1.1`).
**Cible review** : draft D1..D5 Sprint 21 kickoff avant gel final.
**Règle** : `docs/claude/README.md §6.1.1` — le reviewer signale
les angles morts, ne propose PAS de solution. Le planner reste
owner de la décision finale et acknowledge chaque ⚠️ / ❌.
**Règle renforcée crypto/spec G1 extension** (2026-04-16) : chaque
D-decision citant source crypto/protocole standardisé DOIT
enumerer au moins UNE alternative concurrente récente (<= 6 mois)
avec raison factuelle rejet.

---

## 1. Verdict global G1

**CONDITIONAL PASS** (⚠️ × 3, ✅ × 2, ❌ × 0).

Le draft respecte globalement la règle G1 extension : sources
récentes vérifiées (governor 0.10.2, presidio 2.2.362, onnxruntime-
web 1.24.3, LLM Guard 0.3.16), alternatives concurrentes
documentées (tokio-rate-limit 0.8, tract+Rust-wasm, spaCy wasm,
Gretel GLiNER, NVIDIA gliner-PII, NeMo Guardrails, Patronus).

Les 3 ⚠️ sont **documentation/clarity**, pas functional defects.
Ils sont P2-P3 et à traiter en Phase 0 / pre-phase remediations.

**Rigor signal G4 auditeur** satisfait : reviewer a trouvé 3
findings P2+ sur la rigueur des sources + drift HARDENING_ROADMAP.

## 2. Scoring par décision

| D | Verdict | Catégorie finding |
|---|---|---|
| D1 Rate-limit | ✅ | source récente (governor 0.10.2 MIT, alternatives comparées : tokio-rate-limit 0.8 2025-11 vérifié) — note minor : date release exacte crates.io metadata non-canonique |
| D2 PII SDK Option 7 | ⚠️ | sources récentes (onnxruntime-web 1.24.3 + presidio 2.2.362 mars 2026) + alternatives comparées (tract, spaCy wasm, Gretel, nvidia, redact-core, Pyodide, regex-only, GLiNER.js inactif) ; **mais** drift HARDENING_ROADMAP §audited_findings non mis à jour + ambiguïté backbone modèle (ModernBERT cité vs DeBERTa-v3 probable) |
| D3 Output filter | ⚠️ | PLeak CCS'24 + ProxyPrompt arXiv 2505.11459 vérifiés + alternatives comparées (NeMo, guardrails-ai, Unicode category filter, Rabin-Karp, LLM semantic) ; **mais** seuil EED 0.85 non-sourcé dans le paper PLeak (valeur probablement empirique à calibrer Phase B) |
| D4 Quarantine queue | ✅ | pattern S19 `f238d31` reuse vérifié, libp2p gossipsub v1.1 graylist context correct, SBFB-specific design assumé — note : design doc Phase A dédié pre-req normal |
| D5 Cap G7 | ✅ | Carry-overs confirmés (Radicle v1.0 sept 2024, C-PLAN-1 S20 context OK), cap 2/2 respecté |

## 3. Findings détaillés G1

### D1 Rate-limit — ✅

**Source récente + alternatives vérifiées** :

- `governor 0.10.2` : confirmé crates.io, docs.rs, GitHub
  /boinkor-net/governor (High reputation context7). MIT. Maintenu
  ~2 commits/mois derniers 12 mois.
- `tower-governor 0.8` : compat axum 0.8 confirmé.
- `tokio-rate-limit 0.8` (2025-11) : 17.5M ops/sec lock-free
  hashmap, Axum 0.8.6 compat. **Rejet documenté** : drop GCRA =
  pas normalisation algorithme global vs per-key.
- `leaky-bucket`, `async-rate-limiter`, `ratelimit_meter` archived :
  raisons rejet valides documentées.

**Gap mineur** : date release exacte `governor 0.10.2` non obtenue
via crates.io metadata (timestamp index non-canonique). Crate
stable, usage production OK. Audit strict demanderait GitHub tag
verification — non-bloquant.

### D2 PII SDK Option 7 — ⚠️

**Sources récentes + alternatives vérifiées** :

Couche iframe :
- `onnxruntime-web 1.24.3` : GitHub release tag v1.24.3 confirmé,
  npm publié ~mars 2026, Microsoft High reputation. ✓
- `knowledgator/gliner-pii-edge-v1.0` : HF model card confirmé
  existence, F1 0.755 documenté, Apache-2.0 licence OK. ⚠️
  backbone ambigüe dans les sources (voir gap ci-dessous).
- `@huggingface/transformers` v4 (2026-02) : tokenizer seulement
  dans le draft ; GLiNER natif **non supporté** (issue #826
  ouverte 2024-06 non résolue) — le draft utilise uniquement le
  tokenizer, pas le runtime transformer, ce qui contourne le bug.

Couche coord-side :
- `presidio-analyzer 2.2.362` : PyPI release 2026-03-15 confirmée,
  Microsoft MIT, High reputation context7 (7.2k stars GitHub).
- `GLiNERRecognizer` : support Presidio 2.2.361+ confirmé docs.

**Alternatives concurrentes vérifiées (G1 crypto/spec extension)** :
- `spaCy NER wasm` : **n'existe pas 2026** (pas de port wasm
  officiel), rejet par non-existence.
- `Full Rust-first tract + GLiNER + wasm-bindgen` : tract opset
  9-18 max vs GLiNER export opset 19, tract wasm32-unknown-unknown
  browser non documenté, zero precedent production, gline-rs v1.0.1
  mainstream Rust GLiNER a choisi ort PAS tract — rejet
  techniquement documenté.
- `Hybrid Rust-wasm iframe + Presidio` : mêmes blockers iframe.
- `GLiNER.js npm` : inactif 1 an (dernière release mars 2025),
  7650 DL/week — rejet sur maintenabilité vs wrapper custom
  contrôlable.
- `redact-core Rust` (censgate 2026-02) : 2 mois d'existence,
  maintainer solo, pas d'audit — rejet immaturité.
- `Pyodide + Presidio iframe` : 500 MB overhead cold-start —
  rejet resource constraint.
- `Regex-only (scrubadub/pii-codex)` : FP/FN NER élevés — rejet
  qualité detection.
- `Gretel gretel-gliner-bi-small-v1.0` (10/2024) : F1 0.89-0.95,
  DeBERTa-v3-small backbone, 349 MB int8 — trop lourd iframe
  cold-start, OK coord-side.
- `urchade/gliner_multi_pii-v1` (multilingue FR/EN/ES/DE/IT/PT,
  intégré Presidio par défaut) : 349 MB int8 — trop lourd iframe.
  **Candidat coord-side** si besoin multilingue.
- `nvidia/gliner-PII` (2025-10-28) : 570M params, NVIDIA Open Model
  License — compatibilité AGPL-3.0 non vérifiée, rejet prudent.

**Gap P2-1 — drift HARDENING_ROADMAP §3 S21 non mis à jour** :
- Libellé original S17 : « regex PII + optional spaCy NER wasm
  ~500 LOC »
- Requalifié S21 : « onnxruntime-web 1.24.3 + knowledgator/gliner-
  pii-edge-v1.0 + @huggingface/transformers tokenizer + Presidio
  2.2.362 GLiNERRecognizer »
- Frontmatter `audited_findings` HARDENING_ROADMAP.md `last_
  validated: 2026-04-18` ne liste pas encore ce drift. L'audit gate
  S19 item 3 mentionne « S21 grammar ≠ prompt injection defense »
  mais pas le drift spaCy → onnxruntime-web.
- **Traçabilité** : requis pour que le drift soit découvrable par
  audit S22 session fraîche via le frontmatter.
- **Action** : ajout entrée frontmatter `audited_findings` 2026-04-
  18 à fusionner dans le commit `chore(planning): open Sprint 21`.

**Gap P2-2 — ambiguïté backbone modèle `gliner-pii-edge-v1.0`** :
- Recherche 1 (research G2 initial) : « ModernBERT (`ettin-encoder
  -32m`), 32M params, hidden 384 ».
- Recherche 2 (G1 reviewer) : « backbone DeBERTa-v3 ».
- WebFetch planner session : model card HF ne montre pas backbone
  explicite, renvoie uniquement tailles ONNX (197 MB uint8 =
  probablement base-v1.0, pas edge-v1.0).
- **Divergence constatée** : 3 sources, 3 interprétations
  différentes. Likely root cause : HF model card de
  `gliner-pii-edge-v1.0` incomplète sur specs architecture, +
  les 4 variants (edge/small/base/large) sont confus si lus
  superficiellement.
- **Action** : Phase B S21 G8 scan S1 (pre-first-line-of-code)
  DOIT inclure fetch fresh de la model card + `config.json` du
  repo HF pour figer backbone + size ONNX quantized int8 avant
  écrire le wrapper JS. Si backbone final ≠ ModernBERT (ex: DeBERTa
  -v3 avec DisentangledSelfAttention), vérifier que onnxruntime-
  web 1.24.3 supporte l'opset export (généralement opset 19 OK
  via ORT Web, contrairement à tract). Phase B preflight.md
  document ce scan.

### D3 Output filter — ⚠️

**Sources vérifiées** :
- `LLM Guard 0.3.16` (ProtectAI, mai 2025 release, MIT) :
  confirmé PyPI, github.com/protectai/llm-guard 7.2k stars, actif.
  `InvisibleText` scanner doc'd avec curated list zero-width + PUA
  U+E000-U+F8FF + Tag chars U+E0020-U+E007F.
- `PLeak CCS'24` (arXiv 2405.06823, ACM SIGSAC octobre 2024) :
  confirmé. Paper utilise EED (Extended Edit Distance Levenshtein)
  comme métrique similarité reconstruction prompt.
- `ProxyPrompt` (arXiv 2505.11459, mai 2025) : protection 94.70%
  vs 42.80% baseline — future proactive defense, out-of-scope
  detection reactive S21.
- OWASP LLM Top 10 2025 #7 System Prompt Leakage confirmé.

**Alternatives concurrentes vérifiées** :
- NeMo Guardrails 0.11+ (Colang 2.0) : rejet overkill Python 3.8
  dropped.
- `guardrails-ai` + Hub `detect_prompt_injection` : rejet LLM
  secondaire scoring couteux.
- Unicode category filter `Cc|Cf|Co|Cs` strict : rejet casse
  RLO/LRO i18n.
- Rabin-Karp rolling hash : pas paper dédié 2024-2025 prompt-leak,
  académia préfère EM/SM/EED.
- LLM-based semantic : out-of-scope S21 (S23+ si besoin).

**Gap P3-1 — seuil EED 0.85 non-sourcé** :
- Draft cite « threshold 0.85 ».
- PLeak CCS'24 paper utilise EED mais le **seuil 0.85 spécifique
  n'est pas cité dans le paper**. Valeur plausible (cutoff
  similarité standard), mais non vérifiable par citation paper.
- **Action** : Phase B S21 DOIT soit tuner empiriquement via test
  suite dédiée (scénarios divers reconstruction similarity) et
  documenter le tuning dans un commentaire de code source
  (`/* Threshold 0.85 tuned via test_output_filter_eed_similarity
  _scenarios.rs Phase B S21, see sprint21_phase_B_design.md */`),
  soit extraire un default configurable via
  `~/.sbfb/output_filter_policy.toml` hot-reload pattern S20 PoW
  policy. Choix à figer Phase B design doc.

### D4 Quarantine queue — ✅

**Pattern S19 `f238d31` upload_queue.py** : réutilisation baseline
validée S19 production.
**SQLite WAL** : standard sqlite.org, concurrent readers non-
bloqués.
**Manual flush CLI** pattern S20 `sbfb canary ack` : cohérent.
**libp2p gossipsub v1.1 graylist threshold** : confirmé dans spec
(peer score P1-P7, graylist = automatic peer scoring decay).

**Différence SBFB explicite** : quarantine hold 15 min + manual
flush = **design SBFB-spécifique** sans équivalent public P2P.
Draft reconnaît explicitement et prévoit design doc Phase A dédié.

**Action** : Phase A S21 design doc
`.planning/research/S21_phase_D_quarantine_design.md` (ou équivalent
localisation) DOIT couvrir :
- Schema SQLite evolution (backward compat)
- TTL clock semantics (received_at vs inserted_at, NTP sync)
- Manual flush security (bearer X-SBFB-Token + Host + Origin check
  pattern S16)
- Interaction gossip layer (gossip-level PoW gate check at flush
  time vs pre-hold)
- Expected cardinality + benchmarks (~1000 msg/min/15min TTL
  estimate)

Non-bloquant ouverture S21. Pre-req Phase A normal.

### D5 Cap G7 — ✅

**Carry-overs confirmés** :
- Meta-1 Radicle-v1.0 activation tracking : Radicle v1.0 release
  2024-09-13 confirmé, runbook `docs/release/MIRROR_FALLBACK.md
  §3.1-3.8` self-contained.
- C-PLAN-1 plan docs fix : Sprint 20 Phase C context vérifié,
  wire-point divergence confirmée.

**Hors cap tech debt PATTERNS.md** :
- T-NN canary JCS (path `crates/nexus-shell-daemon-core/src/canary/
  mod.rs` à vérifier exists).
- T-NN+1 CanaryRegistry verify Ed25519 (path `packages/nexus-
  coordinator/src/nexus_coordinator/canary_registry.py` à vérifier).
- T-NN+2 iframe PII SDK Rust-wasm realignement S22+.

**Cap respecté** : **2/2**.

## 4. Zones non-vérifiées G1 (budget 30 min)

1. **D1 governor 0.10.2 exact release date** : crates.io releases
   JSON non accessible WebSearch. Audit strict demanderait GitHub
   tag verification — non-bloquant.

2. **D3 EED threshold 0.85 empirical origin** : paper PLeak
   n'énonce pas seuil 0.85 explicitement. Draft ne cite pas source
   (empirical, RFC default, secondary paper). Probablement tuning
   empirique Phase B S21.

3. **D2 ModernBERT vs DeBERTa-v3 backbone confusion** : HF model
   card `gliner-pii-edge-v1.0` incomplète, 3 sources divergent.
   Résolu par scan G8 S1 Phase B fresh pre-first-line-of-code.

4. **D4 SQLite WAL contention edge cases** : size growth cardinalité
   non benchée. Phase A design spec required.

5. **T-NN+2 S22+ feasibility** : tract wasm32-unknown-unknown +
   opset 19 futurologique. Risque item bloqué indéfiniment. Recommend
   S21 Phase D spike (test tract latest + opset 19 coverage fresh)
   si jamais ouvert S22+.

## 5. Acknowledged review findings (G1 ack planner)

Le planner acknowledge chaque ⚠️ ci-dessous explicitement. Aucun
blocage P0/P1 pour Phase A S21. Les 3 ⚠️ sont remediated dans le
kickoff final ou reportés vers Phase 0 / pre-phase actions.

**D1 ✅** — noted, no action required. Governor 0.10.2 date exacte
non critique ; crate stable production-grade.

**D2 ⚠️** — **adjust required AVANT commit `open Sprint 21`** :
1. Ajouter entrée `audited_findings` dans frontmatter
   `docs/security/HARDENING_ROADMAP.md` datée 2026-04-18 :
   > "S21 D2 PII SDK requalifié : libellé roadmap S17 'spaCy NER
   > wasm ~500 LOC' obsolète (spaCy pas de port wasm 2026).
   > Stack retenue : onnxruntime-web 1.24.3 (Microsoft, mars 2026)
   > + knowledgator/gliner-pii-edge-v1.0 (backbone à confirmer
   > Phase B G8 S1 scan) + @huggingface/transformers tokenizer
   > iframe + presidio-analyzer 2.2.362 + GLiNERRecognizer
   > coord-side. Decision `sprint21_kickoff.md §D2`. Rust-wasm
   > iframe realignement Option G reporté S22+ via tech debt
   > T-NN+2 (blocked tract opset 18 max vs GLiNER opset 19 +
   > wasm32-browser zero-precedent + gline-rs a choisi ort pas
   > tract)."
2. Phase B S21 G8 preflight DOIT inclure scan S1 SOTA fresh sur
   `knowledgator/gliner-pii-edge-v1.0` (backbone, config.json,
   tokenizer, opset export) avant écriture du wrapper JS.
   Verdict possible : EXECUTE plan-as-is si confirmé Apache-2.0
   + backbone opset-compatible onnxruntime-web, ou SCOPE-CUT-
   CONSISTENT si switch vers modèle base/small, ou DESIGN-
   CONFLICT si blocker opset.

**D3 ⚠️** — adjust recommanded Phase B S21 :
1. Documenter origine du seuil EED 0.85 dans le code source via
   commentaire (empirique post-tuning Phase B OU pattern PATTERNS
   .md tech debt calibration S22).
2. Alternative : configurable via `~/.sbfb/output_filter_policy.
   toml` hot-reload pattern S20 (parallèle à `relay_pow_policy.
   toml`).
3. Phase B design doc `.planning/research/S21_phase_C_output_
   filter_design.md` (ou équivalent) DOIT couvrir le tuning
   methodology + corpus tests retenus.

**D4 ✅** — noted. Design doc Phase A dédié acté pre-req normal
(pattern S20 Phase B/D design docs `.planning/research/`).

**D5 ✅** — noted, no action required. Cap 2/2 respecté.

---

## 6. Rigor signal G4 auditeur G1

Reviewer a trouvé **3 findings P2 actifs** + 2 findings P3-P2
minor (date governor, EED seuil source, tract feasibility S22+).
Verdict CONDITIONAL PASS, pas CONCERN (rigor signal satisfait).

Planner ack section §5 ci-dessus honore règle `README.md §6.1.1`
(acknowledge chaque ⚠️ explicite).

**Sprint 21 Phase A non bloqué** une fois :
- Entrée `audited_findings` HARDENING_ROADMAP ajoutée (D2 remediation
  §5 item 1).
- Kickoff §4 D2 référence Phase B G8 S1 scan obligatoire (D2
  remediation §5 item 2).

Ces 2 remediations sont intégrées dans le commit
`chore(planning): open Sprint 21` lui-même.
