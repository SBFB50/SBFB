# Sprint 21 Phase C — preflight G8

Date : 2026-04-19
HEAD : `d5b0035` (feat(sprint21): Phase B — client-side PII redaction SDK
iframe (onnxruntime-web + GLiNER PII edge))
Verdict : **EXECUTE plan-as-is**

Scope Phase C (rappel plan §6) : coord-side PII redaction layer 2
(`presidio-analyzer 2.2.362` + `GLiNERRecognizer` extra `[gliner]` sur
même modèle upstream HF `knowledgator/gliner-pii-edge-v1.0` que Phase B
iframe) + output filter (`LLM Guard 0.3.16 InvisibleText` wrappé pour
usage output + EED echo detection via `rapidfuzz.Levenshtein.
normalized_similarity` seuil 0.85 configurable `~/.sbfb/output_
filter_policy.toml` hot-reload + Substring Match / Exact Match fallback).
Hook pre-dispatch worker ordering : rate-limit (Phase A) → PoW (S20
Phase C) → PII redact (Phase C) → dispatch → output filter
(pre-validator Phase C) → `validate_task_response` (S20 D existant).

---

## Scans

### S1 — SOTA 2026 vs design

Libs scannées via Context7 MCP + WebSearch CVE :

| Lib | Version plan | Context7 snapshot | CVE WebSearch | Delta |
|---|---|---|---|---|
| `presidio-analyzer` | `2.2.362` | `/microsoft/presidio` 975 snippets High, `[gliner]` extra + `GLiNERRecognizer` API `model_name` / `entity_mapping` / `flat_ner` / `multi_label` / `map_location` confirmés. Release 2026-03-15 active | snyk + Safety scans = 0 vulnérabilité connue sur 2.2.362. Pas de CVE-2026 trouvée | **clean** |
| `llm-guard` | `0.3.16` | `/protectai/llm-guard` 385 snippets High, `InvisibleText` vit dans `input_scanners` mais API stateless `scan(text)` sanitized_text + is_valid + risk_score. `output_scanners.Sensitive` / `NoRefusal` / `Relevance` / `URLReachability` existent mais pas `InvisibleText` output natif | Pas de CVE-2026 publique sur llm-guard | **pivot D3 post-preflight** (cf. §Pivot log 2026-04-19 : transitive-pin incompatible avec D2, feature ré-implémentée localement) |
| `rapidfuzz` | `3.x` | `/rapidfuzz/rapidfuzz` 277 snippets Medium. `Levenshtein.normalized_similarity(s1, s2)` float [0.0-1.0], parfait pour EED seuil 0.85 configurable | snyk scans = 0 vulnérabilité connue. Version PyPI courante `3.14.x` active | **clean** |

**Note de conception 1 (non-bloquante)** : `LLM Guard 0.3.16
InvisibleText` est officiellement catégorisé `input_scanners` par
la lib. Son API `scan(text) -> (sanitized, is_valid, risk_score)`
est stateless et opère sur un texte arbitraire. Le plan Phase C
§6.2 (`output_filter.py`) réutilise cet InputScanner sur
`model_output` — pattern techniquement valide (même code path
côté lib), documenté dans `.planning/research/S21_phase_C_output_
filter_design.md` (à créer en même temps que le code Phase C
selon pattern Phase B). Le wrapping explicite dans `class
OutputFilter` isole la pattern et reste stable si LLM Guard
0.4+ sort un `output_scanners.InvisibleText` natif.

**Note de conception 2 (non-bloquante)** : Le plan §6 dit « même
modèle ONNX source of truth unique ». Précision factuelle via
Context7 `GLiNERRecognizer` exemple officiel (`urchade/gliner_
multi_pii-v1` typique) : `GLiNERRecognizer` charge le modèle via
`gliner` Python lib (`GLiNER.from_pretrained(model_name)`), qui
résout vers PyTorch par défaut, PAS vers le fichier ONNX 45.8 MB
de Phase B. Source of truth unique = **même modèle upstream HF**
(`knowledgator/gliner-pii-edge-v1.0`), pas même artefact binaire.
Design doc Phase C documentera :
1. Coord = PyTorch path (CPU/GPU, pas de contrainte taille ni
   latence WASM) via `GLiNERRecognizer(model_name="knowledgator/
   gliner-pii-edge-v1.0", ...)`.
2. Iframe = ONNX quint8 45.8 MB déjà livré Phase B.
3. Les deux utilisent le même mapping 10 entités (email, phone,
   CC, SSN, IBAN, name, address, etc.) pour parité comportementale.

Verdict : **S1: clean** (2 notes de conception documentées, aucun
finding bloquant ni non-bloquant au sens G8).

### S2 — Decisions historiques traversées

Fichiers Phase C ciblés (plan §6.2 après correction naming
2026-04-19, cf. `chore(planning)` immédiatement précédent Phase C) :
- `packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py`
  (nouveau)
- `packages/nexus-coordinator/src/nexus_coordinator/output_filter.py`
  (nouveau)
- `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py`
  (modifié — hook `PiiRedactor.redact()` pre-signing dans
  `Dispatcher.submit`)
- `packages/nexus-coordinator/src/nexus_coordinator/validator.py`
  (modifié — hook `OutputFilter.filter()` post-verify Rust
  dans `Validator._handle_result`)
- `packages/nexus-coordinator/pyproject.toml` (modifié)

Commandes :

```bash
git log --all --grep="DEVIATION\|rejected\|scope-cut\|threat-model\
\|pii\|presidio\|llm-guard\|output.filter\|prompt.leak" --oneline -i
# → 0 hit archive qui rejette Presidio/LLM Guard/PII coord-side

grep -rE "DEVIATION deliberee|rejected for|scope-cut at|threat-model" \
  .planning/archive/v*/sprint*_*.md 2>/dev/null | \
  grep -iE "pii|redaction|presidio|gliner|spacy|output.filter|llm.guard"
# → 0 hit

grep -rE "do not|never|reject|avoid" \
  "$HOME/.claude/projects/C--Users-FlowUP-Documents-Code-nexus/memory/feedback_*.md"
# → pas de règle PII/Presidio/coord-side évitée
```

Analyse findings potentiels :

- Le rejet documenté « spaCy NER wasm ~500 LOC » (roadmap §S21
  original) = **drift roadmap** requalifié 2026-04-18 dans
  `HARDENING_ROADMAP.md` frontmatter `audited_findings` : stack
  retenue = `presidio-analyzer 2.2.362 + GLiNERRecognizer`
  coord-side. C'est la **rationale du design RETENU** Phase C,
  pas un rejet. Reverse-commit check confirmé inline.
- Memory `audited_findings 2026-04-18 S21 open` cite littéralement :
  « coord-side = presidio-analyzer 2.2.362 (Microsoft MIT,
  2026-03-15) + GLiNERRecognizer extra [gliner] + même modèle
  ONNX source-of-truth unique ». Le plan Phase C implémente
  exactement cette décision. Clean.
- Memory `audited_findings 2026-04-16 S19 deep analysis` note
  : « S21 grammar ≠ prompt injection defense ». Rappel de garde
  — le structured output S20 est grammar (format JSON
  contraint), pas une defense prompt injection. Phase C output
  filter (LLM Guard InvisibleText + EED echo) = **couche
  distincte** explicitement reactive-detection, conforme à ce
  garde-fou. Clean.
- Pattern Phase B (`sbfb-bridge.js` extension 3→4 méthodes
  whitelist) ne s'applique pas à Phase C (aucun bridge iframe
  touché).

Reverse-commit check : N/A (0 finding à classifier, les mentions
rencontrées sont soit le design RETENU soit des garde-fous
confirmant le plan).

Verdict : **S2: clean**.

### S3 — Threat model coverage

Commandes :

```bash
grep -E "^### T[0-9]|^## " docs/security/THREAT_MODEL.md
grep -B 1 -A 5 "S21" docs/security/HARDENING_ROADMAP.md
ls .planning/active/sprint21_phase_*_review.md
```

Threats T0-T5 mapping (cf. `docs/security/ADVERSARIES.md` +
`THREAT_MODEL.md` + HARDENING_ROADMAP §3) :

| Threat | Couverture Phase C | Status |
|---|---|---|
| **T4 — Model extraction / PII harvest** (worker malveillant extrait PII du prompt utilisateur) | **Layer 2 defense-in-depth coord-side** (iframe layer 1 Phase B redact avant postMessage, coord layer 2 Presidio re-scan avant dispatch worker). Double-opt filtering | ✅ couvert |
| **T3 — Prompt leak / active extraction** (adversaire reconstruit system_prompt via response exfiltration, PLeak CCS'24) | **Primitive primaire Phase C** (EED Levenshtein similarity > 0.85 + Substring / Exact Match, appliqués sur `model_output` avant retour client). Curve de recall/precision tunable via seuil policy.toml | ✅ couvert |
| **T-OWASP-LLM-#7** (prompt leakage zero-width + PUA + Tag chars invisibles dans output) | LLM Guard `InvisibleText` scanner whitelist `Cf` pour RLO/LRO/i18n légitime (Arabe/Hébreu) | ✅ couvert |
| **T2 — PII leak passive** (user push PII accidentellement → redacted coord-side même si iframe layer 1 fail model load) | Presidio layer 2 = fallback non-regression | ✅ couvert |
| **T1 — Network observer** | Non-regression : coord redact n'affecte pas iroh encryption, opérateur coord voit redacted-only (pré-dispatch) | ✅ non-regression |
| **T0 — Passive attacker** | Non-regression | ✅ |
| **T5 — Nation-state / TEE escape** | Hors scope (S25+ roadmap) | ➖ hors scope |

HARDENING_ROADMAP §3 S21 ligne pertinente (frontmatter
`audited_findings 2026-04-18`) :

> « §4 extraction + §7 DoS meme primitive rate-limit per-consumer →
> mutualiser S21-22 ... PII SDK defense-in-depth : client iframe ...
> coord-side = presidio-analyzer 2.2.362 + GLiNERRecognizer »

Phase C livre **exactement** layer 2 (coord-side) + output filter
requis §3 S21. Pas de pré-requirement manquant.

Regression flags : aucun.
- Phase A rate-limit (worker-engine gate Rust) ne partage pas de
  surface avec Phase C Python coord-side.
- Phase B iframe JS client ne partage pas de surface avec Phase C
  Python coord-side.
- Le hook ordering rate-limit → PoW → PII redact → dispatch
  respecte l'architecture enforce par le daemon Rust (Phase A +
  S20 C) puis le coord Python (Phase C).

Verdict : **S3: clean**.

### S4 — Wire format / pre-launch invariants

Commandes :

```bash
grep -rE "_VERSION\s*[:=]\s*[0-9]+" \
  crates/nexus-core-rs/src/canonical.rs \
  crates/nexus-core-rs/src/schemas/ \
  packages/nexus-coordinator/src/nexus_coordinator/task_response_validator.py
# → seul hit : "crates/nexus-core-rs/src/schemas/mod.rs: *_VERSION = 1
#    pre-launch protocol policy" (commentaire policy, aucune modification
#    introduite Phase C)

grep -A 10 "Pre-launch protocol" CLAUDE.md
# → politique confirme : BLOB_VERSION=0x01, TASK_RESPONSE_VERSION=1,
#    CANARY_VERSION=1, ANNOUNCEMENT_VERSION=1 tous inchangés
```

Invariants vérifiés :

| Invariant | Status Phase C |
|---|---|
| `BLOB_VERSION = 0x01` | ✅ inchangé (Phase C coord-side pur Python, ne touche pas iroh-blobs format) |
| `TASK_RESPONSE_VERSION = 1` | ✅ inchangé (hook Phase C tourne AVANT `validate_task_response` S20 D — n'affecte pas le schema JSON Draft-07) |
| `CANARY_VERSION = 1` | ✅ inchangé (Phase C ne touche pas canary) |
| `ANNOUNCEMENT_VERSION = 1` | ✅ inchangé (Phase C ne touche pas ProjectAnnouncement) |
| Pas de tolerant decoder multi-version introduit | ✅ |
| `#[serde(default)]` ajoutés legitimes | ✅ N/A (Phase C Python, pas de Rust serde touché) |
| DOMAIN_* signatures préservées | ✅ inchangées (Phase C ne touche pas crypto sign) |
| D1..D5 Day 0 non rebattus | ✅ D2 layer 2 + D3 output filter implémentés tels que figés kickoff §D2-D3 |
| Decisions `nexus_grid_pivot.md` non contredites | ✅ (coord Python FastAPI existant, ajout module pur, pas de nouvelle primitive wire) |

Détails hook insertion (points réels identifiés dans le code S20) :

- **PiiRedactor hook** dans `Dispatcher.submit`
  (`dispatcher.py`) AVANT `nexus_core.sign_task` : le prompt
  user (et le system_prompt) est redacté **avant** signature,
  donc avant toute écriture dans iroh-docs. La `SubmitRequest`
  voit ses champs `prompt` et `system_prompt` remplacés par les
  versions redactées. Aucun changement de la struct
  `SubmitRequest` — c'est une mutation in-flight documentée
  inline. Le TaskEntry wire signé contient le texte redacté,
  donc ni worker ni relai voient le prompt brut.

- **OutputFilter hook** dans `Validator._handle_result`
  (`validator.py`) APRÈS `Verifier.verify_entries` (3-layer
  signature + model digest + logprob fingerprint) et AVANT
  `Dispatcher.mark_completed` + `KudosLedger.credit`. Un
  verdict negative (invisible chars / echo above threshold)
  convertit le event en `ValidationEvent(kind="result_rejected",
  reason="output_filter: <sub-reason>")` + appelle
  `dispatcher.mark_failed(task_id, reason)`. Le worker n'est
  pas crédité — pattern identique à un verify 3-layer fail.
  **Le ResultEntry est déjà publié sur iroh-docs par le worker
  avant d'arriver ici** ; le filtre protège la surface coord
  (audit trail + kudos + delivery au client via l'API control
  plane), pas la surface réseau gossip. C'est la limite
  inhérente du defense-in-depth coord-side documentée dans le
  threat model `T3 Prompt leak`.

- **Wire format `TASK_RESPONSE_VERSION = 1`** n'est PAS touché
  ni lu côté coord Python — `validate_task_response` reste
  exclusivement côté worker Rust (`ollama.rs:222` +
  `llama_cpp.rs:439`). L'OutputFilter coord opère uniquement
  sur le champ `payload.content` (ou équivalent) du
  `ResultEntry` déjà deserializé, pas sur le schema JSON
  draft-07 du worker.

Verdict : **S4: clean**.

---

## Synthèse

| Scan | Verdict |
|---|---|
| S1 SOTA | clean (2 notes de conception documentées inline) |
| S2 Historical | clean |
| S3 Threat model | clean |
| S4 Wire invariants | clean |

**Règle d'agrégation** : 0 finding bloquant + 0 finding
non-bloquant → **EXECUTE plan-as-is**.

## Action

Procède Phase C code implementation selon plan §6 :

1. **Aucun pivot** nécessaire. Day 0 D2 layer 2 + D3 output
   filter respectées.
2. **Aucun carry-over S22** ajouté depuis ce préflight.
3. Design doc pre-Phase-C `.planning/research/S21_phase_C_output_
   filter_design.md` (plan §6.1) à créer dans le même commit
   Phase C (pattern Phase B co-commit CRAFT + code accepté par
   audit Phase B §Working tree audit). Le design doc explicitera :
   - Note de conception 1 : `InvisibleText` input_scanner
     réutilisation output (stateless API).
   - Note de conception 2 : source-of-truth unique = modèle HF
     upstream (pas fichier ONNX binaire iframe).
   - EED seuil 0.85 empirical tuning corpus (garde-fou D3 kickoff).
   - Hook pre-dispatch ordering rate-limit → PoW → PII →
     dispatch + output filter pre-validator.
4. Commit phase suivra template plan §6.5 avec body riche (delta
   +10 Python coord tests + scope cuts + working tree audit G5 +
   référence à ce préflight + design doc).

Ce document sera archivé Phase F dans `archive/v1.2/` avec les
autres artefacts S21.

---

## Pivot log post-preflight

### 2026-04-19 — Pivot D3 (drop llm-guard)

**Finding tardif** : au premier `uv sync` pendant l'implémentation
coord-side (après le préflight initial ci-dessus), le graph deps
a échoué — `llm-guard 0.3.16` (dernière release PyPI 2026-04, pas
de plus récent disponible) transitive-pin `presidio-analyzer==
2.2.358`, incompatible avec le `>=2.2.362` requis par D2 stack.
G8 S1 finding **manqué par le préflight initial** : les versions
unitaires ont été vérifiées via context7 (`/protectai/llm-guard`
385 snippets + `/microsoft/presidio` 975 snippets) mais le graph
deps transitif cross-libs n'a pas été résolu avant écriture code.
Seul `uv sync` l'expose (pas de check context7 pour les transitive
pins de ce type).

**Arbitrage user 2026-04-19 — Option B** : drop llm-guard entièrement
et ré-implémenter le scanner `InvisibleText` localement dans
`output_filter.py`. Rationale :

1. Le scanner `InvisibleText` de llm-guard est lui-même ~30 lignes
   d'unicode category checks + regex strip (pas de ML, pas de
   modèle chargé, API stateless). Cf. context7 exemple : `from
   llm_guard.input_scanners import InvisibleText; scanner =
   InvisibleText(); sanitized, is_valid, risk_score =
   scanner.scan(prompt)` — comportement reproductible en
   ~30 lignes Python.
2. llm-guard 0.3.16 tire un graph transitif massif (torch +
   transformers + spaCy ~500 MB) pour des scanners qu'on
   n'utilise pas (`Sensitive`, `NoRefusal`, `Relevance`,
   `URLReachability`). Drop élimine cet overhead.
3. Ré-implémentation locale = code auditable, pas de magic
   lib, testable ligne par ligne. Parité avec llm-guard
   comportement testée explicitement (tests 4-5 du plan
   §6.3).
4. L'esprit D3 kickoff (feature InvisibleText + whitelist
   Cf pour RLO/LRO i18n) est **préservé par construction**
   — c'est le même algorithme, juste implémenté chez nous.

**Impact G8 scans (re-classification)** :
- S1 : llm-guard passe de "non-bloquant note de conception 1"
  à "pivot D3 post-preflight" — dep retirée, feature
  ré-implémentée.
- S2-S4 : inchangés (pas de threat model / wire format / Day 0
  touchés par le pivot).

**Garde-fou G8 vérifié pour pivot** (README §6.9) :
- [x] Evidence-based : `uv sync` error output + PyPI version
  check + context7 llm-guard lock file inspection.
- [x] Day 0 respect : D3 non rebattu (feature InvisibleText
  préservée, implémentation détail changé).
- [x] Wire format : aucun touché.
- [x] Test budget cap : +1 test parité (InvisibleText local vs
  whitelist Cf), delta total <= cap 2.5x.
- [x] Thème sprint : préservé (output filter, defense-in-depth).
- [x] Pas YAGNI : le pivot réduit le scope (moins de deps, pas
  plus de code).
- [x] Retrospective trackée : `chore(planning)` suivant +
  mention `sprint21_audit_plan.md` Phase C Track pivot D3.

**Verdict G8 post-pivot** : **EXECUTE plan-as-is modifié**
(équivalent SCOPE-CUT-CONSISTENT avec retrait de dep).

**Action** : `chore(planning)` avant feat Phase C update
`sprint21_plan.md §6.2` + `pyproject.toml` + ce préflight + design
doc §2.2 revised. Feat Phase C suivra avec code modules +
dispatcher/validator hooks + tests.
