# Sprint 21 — Kickoff (Rate-limit + client-side PII redaction SDK + output filter + quarantine queue)

**Écrit** : 2026-04-18 (session fraîche post-audit gate S20 `66a3a7c`).
**Type** : **sprint implementation** (post-Gate 2 prerequis S20,
débloquant §4 model extraction + §7 DoS flood par rate-limit
multi-tier + PII redaction defense-in-depth client/coord).
**Tip master d'entrée** : `66a3a7c` (chore(sprint20): audit gate S20
— findings verdict PASS).
**Phase 0 audit Sprint 20** : **DÉJÀ JOUÉ** — findings dans
`.planning/active/sprint20_audit_findings.md` (verdict **PASS**,
0 P0 + 0 P1 + 4 P2 carry actifs + 6 P2 résolus in-phase + 6 P3).
Migré vers `archive/v1.2/` dans ce même commit d'ouverture S21.

---

## Sources context7 + WebSearch consultées (pré-gel D1..D5)

**Deep research G2 lancée 2026-04-18** avant figer D1..D5 (trigger
`Sprint S+2=S21` activé depuis `last_validated: 2026-04-18`
HARDENING_ROADMAP) :

### Rate-limit Rust (D1)

- **`governor 0.10.2`** (crates.io, 2025-11-13, MIT, GCRA Generic
  Cell Rate Algorithm, DashMap keyed) : seul mature Rust 2026 avec
  GCRA + keyed.
- **`tower-governor 0.8`** (axum 0.8 middleware) : wrapper natif
  governor.
- **`tokio-rate-limit 0.8`** (2025-11-01, lock-free hashmap per-key
  niche) : récent mais manque GCRA, moins battle-tested.
- `ratelimit_meter` : archivé, fork officiel vers governor.
- `leaky-bucket`, `async-rate-limiter` : pas keyed native.
- Aucun crate Rust rate-limit 2026 n'a d'audit tiers public (vrai
  pour tous les candidats).

### PII redaction SDK (D2)

- **DRIFT MAJEUR roadmap S17** : libellé « spaCy NER wasm ~500 LOC »
  **obsolète 2026**. spaCy n'a pas de port wasm officiel maintenu.
  Alternatives modernes 2026 : ONNX Runtime Web + modèles GLiNER
  quantized (HF `knowledgator`, `gretelai`, `urchade/gliner_multi_
  pii`, `nvidia/gliner-PII`) OU Rust-native via tract/candle/burn.
- **`onnxruntime-web 1.24.3`** (npm, Microsoft, mars 2026) :
  standard industrie mature, WASM SIMD+threaded default.
- **`presidio-analyzer 2.2.362`** (PyPI, Microsoft MIT, 2026-03-15,
  7.2k stars, `[gliner]` extra + `GLiNERRecognizer` depuis
  2.2.361) : battle-tested coord-side.
- **`knowledgator/gliner-pii-edge-v1.0`** (HF, Apache-2.0, publié
  2024-01-29, F1 0.755, ~45.8 MB quint8 ONNX) : candidat iframe
  client-side. **Backbone à confirmer Phase B G8 S1 scan** (sources
  divergent ModernBERT vs DeBERTa-v3).
- `tract` 0.22.1 Sonos (pure Rust ONNX) : **opset 9-18 testé, GLiNER
  export opset 19** → gap probable. `wasm32-unknown-unknown`
  browser **non documenté officiellement**. Zero precedent
  production tract+GLiNER+wasm browser.
- `gline-rs v1.0.1` (2026-01, pure Rust GLiNER mainstream) a choisi
  **`ort` (wrapper ONNX Runtime Microsoft), PAS tract**. Signal
  fort opset coverage tract insuffisant.
- Gretel `gretel-gliner-bi-small-v1.0` (10/2024, DeBERTa-v3-small
  backbone, 349 MB int8, F1 0.94-0.95) : trop lourd iframe,
  candidat coord-side via Presidio.
- `urchade/gliner_multi_pii-v1` (multilingue FR/EN/ES/DE/IT/PT,
  intégré Presidio par défaut, 349 MB int8) : trop lourd iframe.
- `nvidia/gliner-PII` (2025-10-28) : NVIDIA Open Model License
  (compatibilité AGPL-3.0 non vérifiée) — rejet prudent.
- `GLiNER.js npm` (inactif ~mars 2025, 7650 DL/week) : rejet
  maintenabilité.
- `redact-core` Rust (censgate 2026-02) : crate 2 mois, solo
  maintainer, pas d'audit.

### Output filter (D3)

- **`LLM Guard 0.3.16`** (ProtectAI MIT 2025) `InvisibleText`
  scanner : curated list zero-width, PUA U+E000-U+F8FF, Tag chars
  U+E0020-U+E007F.
- **PLeak CCS'24** (arXiv 2405.06823, ACM SIGSAC octobre 2024) :
  EED (Extended Edit Distance Levenshtein) + SM (Substring Match) +
  EM (Exact Match) = métriques standard prompt leak detection.
- **ProxyPrompt** (arXiv 2505.11459, mai 2025) : défense proactive
  94.70% vs 42.80% baseline — hors scope S21 reactive detection.
- OWASP LLM Top 10 2025 #7 System Prompt Leakage confirmé.
- `NeMo Guardrails` (NVIDIA, Colang 2.0 Python 3.8 dropped) :
  overkill.
- `guardrails-ai` + Hub (LLM secondaire scoring) : couteux.

### Quarantine queue (D4)

- `libp2p gossipsub v1.1` graylist threshold (automatic peer score
  decay P1-P7) : **pas d'équivalent explicite hold 15 min + manual
  flush**.
- Tor DoS prevention (cooldown exponentiel par-client-IP) : pattern
  cooldown pas quarantine.
- Pattern S19 Phase D `f238d31` `upload_queue.py` SQLite WAL :
  validation production réutilisable.
- `libp2p gossipsub` AztecProtocol `topicValidators` issue #10347 :
  application-level `ACCEPT/REJECT/IGNORE` sans hold queue.

Frontmatter `docs/security/HARDENING_ROADMAP.md` update dans ce
commit : `last_validated: 2026-04-18` + entrée `audited_findings`
2026-04-18 requalifiant D2 SDK + §3 S21 libellé.

---

## 1. Constat d'entrée

### 1.1 D'où on part

Sprint 20 a livré les **6 big rocks Gate 2 prerequis** (encryption
at rest keypair double layer + duress PIN fake keypair noop + panic
wipe 5-tap + PoW runtime wire gossip subscribe + structured output
dual-backend LlmBackend + warrant canary federation foundations +
dual-transport WSS fallback observability) + **premier pivot G8
effectif** (Phase E federation foundations Option C arbitré user,
décision threat-model `04c9621` préservée by construction).

Audit gate S20 Phase 0 joué par session fraîche 2026-04-18,
verdict **PASS** (0 P0 + 0 P1 + 4 P2 carry actifs + 6 P2 résolus
in-phase + 6 P3).

Gate 2 prerequis effectivement fournis. **Sprint 21 attaque le
scope rate-limit multi-tier + PII redaction defense-in-depth**
pour débloquer §4 model extraction + §7 DoS flood + Gate 2+ apps
(TransLingua, FamilyScan) en sécurisant les outputs.

### 1.2 Ancrage HARDENING_ROADMAP §3 Sprint 21

La roadmap Phase D S17 spécifie Sprint 21 items :

| Item | Source roadmap | Phase S21 |
|---|---|---|
| Rate limit sliding-window per-(consumer, worker, model) worker-core | §3 S21 | A |
| Client-side redaction SDK (requalifié onnxruntime-web + GLiNER iframe + Presidio coord) | §3 S21 | B + C |
| Output filter lib SDK (system prompt echo detection + beacon chars) | §3 S21 | C |
| Quarantine queue gossip (unverified-high-rate messages hold 15 min manual flush) | §3 S21 | D |
| **C-PLAN-1 plan docs fix** wire-point divergence S20 post-audit | carry S20→S21 | 0 ou E |
| **Tech debt batch** S20 (E-1 canary JCS + E-2 registry verify Ed25519) | tech debt hors cap | E |

Plus **Meta-1 Radicle-v1.0 tracking** carry G7 (cf.
`sprint21_carry_summary.md`).

**Gate unlock** fin S21 : aucun Gate officiel HARDENING_ROADMAP §7
débloqué directement, mais **prerequisite Gate 2 consolidé**
(TransLingua/FamilyScan) par defense-in-depth PII + DoS mitigation
pour apps confidentielles.

### 1.3 Compteurs de tests à l'entrée (tip `66a3a7c`)

| Suite | Count observé entrée S21 |
|---|---|
| Rust workspace | 642 |
| Python SDK | 185 |
| Python coordinator | 213 + 3 skipped |
| Python app-gov | 46 |
| Vitest unit | 241 |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX | 246+ |
| **Total** | **~1371 tests** |

**Delta Sprint 21 attendu** : **+45 à +75** (HARDENING_ROADMAP
projection : +50). Répartition estimée par phase dans `plan.md §9`.

### 1.4 Pre-launch protocol policy (rappel)

Sprint 20 a confirmé la règle : `*_VERSION = 1` jusqu'au tag
`v1.0`, pas de tolerant decoder multi-version, `#[serde(default)]`
légitime uniquement pour runtime tolerance. Sprint 21 respecte :
aucun item ne touche un wire format existant.

Le **quarantine queue** (Phase D) produit un schema SQLite local
`~/.sbfb/quarantine.db` (format nouveau, pas de wire protocol).
Format documenté dans plan §D.

Le **rate-limit policy** produit un fichier de config local
`~/.sbfb/rate_limit_policy.toml` (pattern `relay_pow_policy.toml`
S20 + `tokens.json` S18, hot-reload file-watcher). Pas de wire.

Les **PII redaction SDK iframe + coord** sont des transformations
locales pre-send (iframe) + pre-dispatch (coord). Le worker ne
verra jamais les PII si redaction active — pas de changement wire
task protocol.

L'**output filter** fait une validation post-worker-response sur
le wire `TaskResponse v1` existant (S20 Phase D structured output)
— pas de bump de version, juste un gate supplémentaire avant
`validate_task_response`.

---

## 2. Goal en une phrase

**Le projet ajoute une défense multi-tier rate-limit per-(consumer,
worker, model) sliding-window GCRA côté worker + un SDK PII
redaction defense-in-depth (client iframe via onnxruntime-web +
coord via Presidio GLiNERRecognizer, même modèle ONNX source of
truth) + un output filter (invisible chars + prompt echo EED
detection) + une quarantine queue SQLite WAL local hold 15 min
manual flush CLI, débloquant §4 model extraction + §7 DoS flood +
consolidant Gate 2 prerequis apps confidentielles — critère SMART :
fail-fast checklist `sprint21_verification.md §Fail-fast` verte
(28+ rows exécutables) au Phase F wrap-up.**

---

## 3. Phase 0 — Audit Sprint 20 (DÉJÀ JOUÉ — verdict PASS)

**Status** : JOUÉ session 2026-04-18, commit `66a3a7c chore(sprint20):
audit gate S20 — findings (verdict PASS, no blocking fix)`. Ne pas
rejouer. Cf. `sprint20_audit_findings.md` (migré vers
`archive/v1.2/` dans ce commit d'ouverture S21).

**Commit stack du gate** :

```
66a3a7c chore(sprint20): audit gate S20 — findings (verdict PASS, no blocking fix)
131f32b chore(planning): sprint20 — migrate Phase F review overlooked in f209168
```

Aucun `fix(sprint20): ...` requis (0 P0/P1). Les 4 P2 carry actifs
sont tracés dans `sprint21_carry_summary.md` (2 carry scope =
Meta-1 + C-PLAN-1, 3 tech debt hors cap = canary JCS + registry
verify + iframe Rust-wasm S22+).

**Verdict final** : **PASS**. Sprint 21 Phase A non-bloqué.

---

## 4. Décisions Day 0 (D1..D5)

### D1 — Rate-limit sliding-window multi-tier per-(consumer, worker, model)

**Retenu** : **`governor 0.10.2`** (crates.io 2025-11-13, MIT,
GCRA Generic Cell Rate Algorithm) + **`tower-governor 0.8`** axum
middleware (compat axum 0.8).

Storage : **DashMap keyed** (`DefaultKeyedRateLimiter<K>` avec
`K = (ConsumerId, WorkerId, ModelId)` tuple). Éviction manuelle via
tokio task périodique (60s) appelant `retain_recent()` +
`shrink_to_fit()`.

Budgets multi-tier configurables via `~/.sbfb/rate_limit_policy.
toml` (pattern hot-reload file-watcher `relay_pow_policy.toml`
S20 Phase C / `tokens.json` S18 D-1) :

```toml
[default]
consumer_per_min = 100
worker_per_min = 50
model_per_min = 20
burst_multiplier = 2.0       # GCRA burst capacity = base × multiplier

[overrides]
# Opérateur peut whitelister des consumers hauts privilèges
# [[overrides.consumer]]
# pubkey_hex = "abc..."
# per_min = 500
```

Integration : middleware injecté sur HTTP loopback endpoint
`/task/submit` (et dispatch downstream) — defense-in-depth
cumulative avec PoW gossip gate existant S20 Phase C `16b94ba`.

**Rejetés** :

- **`ratelimit_meter`** : archivé depuis 2021, fork officiel
  redirige vers governor. Fin de vie.
- **`leaky-bucket`** : pas keyed native, nécessite wrapper custom
  pour cardinalité per-tuple = reinvente governor.
- **`async-rate-limiter`** : pas keyed native, même argument.
- **`tokio-rate-limit 0.8`** (crates.io 2025-11-01, lock-free
  hashmap per-key, 17-20M ops/sec single-thread, MIT) : **candidat
  sérieux récent** mais manque GCRA (token bucket + leaky bucket
  seulement). GCRA est strictement plus expressif (permet burst
  explicite + smoothing rate cross-thread cohérent). Rejet : pas
  de gain clair vs governor qui a GCRA natif + ecosystème axum
  mature `tower-governor` (pas d'axum middleware officiel côté
  tokio-rate-limit). Écart maintenabilité + battle-testing
  documentation.
- **Custom lru + atomic token bucket** : reinvente governor sans
  gain mesurable. `docs/rust/PATTERNS.md §T-reinvent-the-wheel`.
- **Global rate-limit daemon-wide** : trop grossier, défaite
  §4 (un consumer fair-share peut bloquer un autre).

**Implications code** :

- Nouveau module `crates/nexus-worker-core/src/rate_limit.rs` :
  `struct RateLimiter` wrapping `governor::DefaultKeyedRateLimiter
  <(ConsumerId, WorkerId, ModelId)>` + `Clock::quanta_with_offset
  (0)` pour déterminisme tests.
- Nouveau module `crates/nexus-shell-daemon/src/rate_limit_policy_
  loader.rs` : pattern hot-reload `~/.sbfb/rate_limit_policy.toml`
  + `notify` file-watcher (50 ms debounce, cohérent S18 D-1
  `TokenRotator` + S20 Phase C `pow_policy_loader.rs`).
- Middleware `tower-governor` branché sur route `/task/submit`
  dans `crates/nexus-shell-daemon/src/http.rs`.
- Test intégration : saturation intentionnelle tuple → 429 retour,
  éviction auto 60 s après quiet, reload live policy.toml.

**Cap règle G1 extension** (alternative concurrente récente <=6 mois
obligatoire) : `tokio-rate-limit 0.8` (nov 2025) cité et rejeté
avec rationale factuel. ✓

### D2 — PII redaction SDK : Option 7 custom JS iframe + Presidio GLiNER coord (defense-in-depth)

**Retenu — double couche defense-in-depth** :

#### Couche 1 : client-side iframe (`web/src/sdk/pii/`)

- **Runtime inference** : `onnxruntime-web 1.24.3` (npm Microsoft,
  mars 2026, WASM SIMD+threaded default). Bundle wasm ~20 MB
  default, minimal build ORT-format si nécessaire.
- **Tokenizer** : `@huggingface/transformers` v4 (npm, 2026-02)
  utilisé **uniquement pour le tokenizer** (pas le runtime
  transformer — issue #826 GLiNER natif non supporté, on contourne
  en chargeant le modèle directement via onnxruntime-web).
- **Modèle** : `knowledgator/gliner-pii-edge-v1.0` (HF, Apache-2.0,
  publication 2024-01-29, F1 0.755, ~45.8 MB quint8 ONNX cible).
  **Backbone architecture + taille exacte ONNX à confirmer Phase B
  G8 S1 scan** (sources G2 divergent ModernBERT vs DeBERTa-v3 ;
  model card HF incomplète). Fallback si backbone incompatible
  onnxruntime-web : `gliner_small-v2.1` onnx-community (plus petit,
  multilingue non-PII mais testable). Décision Phase B preflight.
- **Wrapper JS thin** ~300-500 LOC dans `web/src/sdk/pii/` :
  `detect(text) → [{ entity, start, end, confidence }]` +
  `redact(text, replacement) → redacted_text`.
- **Regex fallback curated** : si model load fail (CSP blocker,
  network error iframe), fallback scrubadub-equivalent regex pour
  email/phone/credit card minimum. Better-than-nothing defense.
- **Integration** : exposé via `sbfb-bridge.js` postMessage bridge
  (pattern S13) — nouvelle méthode whitelist `pii_redact(text,
  policy)` dans le bridge coord side.

#### Couche 2 : coord-side (`packages/nexus-coordinator`)

- `presidio-analyzer 2.2.362` (PyPI Microsoft MIT 2026-03-15,
  7.2k stars) + `presidio-anonymizer` bundled.
- Extra `[gliner]` (Presidio 2.2.361+) + `GLiNERRecognizer` avec
  **même modèle ONNX `knowledgator/gliner-pii-edge-v1.0`** que
  iframe = source of truth unique ONNX. Presidio Python charge via
  `onnxruntime` Python package (Microsoft), iframe charge via
  `onnxruntime-web` — 2 runtimes Microsoft-maintenus, même modèle.
- Wrap Python `packages/nexus-coordinator/src/nexus_coordinator/
  pii_redactor.py` : `class PiiRedactor` avec méthode `redact(text,
  language='en') → redacted_text` + `detect(text) → findings`.
- Hook pre-dispatch worker : avant d'envoyer un `Task` au worker,
  passer le prompt dans `PiiRedactor.redact()` (defense-in-depth
  layer 2 si iframe bypass).

**Defense-in-depth rationale** : iframe pre-redact avant
postMessage → coord re-redact avant dispatch worker. Si bug
iframe (ex: CSP blocker model load + fallback regex fail),
coord rattrape ; si bug coord (ex: Presidio downgrade), iframe
mitige.

**Policy configurable** : `~/.sbfb/pii_redaction_policy.toml`
coord-side (pattern hot-reload) :

```toml
[default]
enabled = true
entities = ["PERSON", "EMAIL_ADDRESS", "PHONE_NUMBER",
            "CREDIT_CARD", "SSN", "IBAN", "MEDICAL_LICENSE"]
replacement = "[REDACTED:{ENTITY}]"
confidence_threshold = 0.5

[overrides.gate2_apps]
# Apps confidentielles (TransLingua, FamilyScan) force hard-redact
enabled = true
confidence_threshold = 0.3   # plus strict
entities = ["*"]             # tout redact
```

**Rejetés** :

- **spaCy NER wasm** (libellé roadmap S17 original) : **n'existe
  pas en 2026**. spaCy n'a pas de port wasm officiel maintenu.
  Drift roadmap S17 requalifié via `audited_findings 2026-04-18`
  HARDENING_ROADMAP frontmatter.
- **Full Rust-first (tract + GLiNER + wasm-bindgen)** : `tract`
  0.22.1 Sonos teste opset ONNX 9-18 mais GLiNER exporte
  typiquement opset 19 (DisentangledSelfAttention DeBERTa-v3 non
  documenté supporté). `tract` `wasm32-unknown-unknown` (browser)
  **non documenté officiellement** — seul `wasm32-wasi`
  (wasmtime runner) démontré dans examples Sonos. Aucun precedent
  production OSS tract+GLiNER+wasm browser. `gline-rs` v1.0.1
  (2026-01, Rust GLiNER mainstream) a **choisi `ort` (wrapper
  ONNX Runtime Microsoft), PAS tract**. Défrichage non-budgetable
  S21. **Tech debt T-NN+2** re-alignment S22+ carry.
- **Hybrid Rust-wasm iframe (tract) + Presidio coord** : mêmes
  blockers iframe que Full Rust-first.
- **`GLiNER.js` npm** : dernière release ~mars 2025, inactif
  ~1 an, 7650 DL/week, maintainer solo. Maintenabilité incertaine
  vs wrapper custom contrôlable.
- **`redact-core` Rust** (censgate, 2026-02) : crate 2 mois,
  maintainer solo, pas d'audit public.
- **Pyodide + Presidio iframe** : ~500 MB Pyodide + spaCy overhead,
  cold-start iframe prohibitif.
- **Regex-only (scrubadub/pii-codex)** : FP/FN élevé sur entités
  NER (noms, adresses, organizations). Acceptable fallback, pas
  defense primaire.
- **Gretel `gretel-gliner-bi-small-v1.0`** (10/2024, DeBERTa-v3-
  small backbone, 349 MB int8, F1 0.94-0.95) : trop lourd iframe
  cold-start (349 MB vs 45.8 MB edge). **Candidat coord-side si
  besoin confidence plus élevée** (F1 0.94 vs 0.755). À réévaluer
  Phase C G8 S1 si bench edge insuffisant.
- **`urchade/gliner_multi_pii-v1`** (multilingue FR/EN/ES/DE/IT/PT,
  intégré Presidio par défaut, 349 MB int8) : trop lourd iframe.
  **Candidat coord-side si besoin multilingue** (edge est EN-only).
- **`nvidia/gliner-PII`** (2025-10-28, 570M params) : NVIDIA Open
  Model License, compatibilité AGPL-3.0 non vérifiée clause-par-
  clause — rejet prudent.

**Drift HARDENING_ROADMAP** acté dans ce commit :

Frontmatter `audited_findings` 2026-04-18 ajout :
> « S21 D2 PII SDK requalifié : libellé roadmap S17 'spaCy NER
> wasm ~500 LOC' obsolète (spaCy pas de port wasm 2026). Stack
> retenue : onnxruntime-web 1.24.3 (Microsoft, mars 2026) +
> knowledgator/gliner-pii-edge-v1.0 (backbone à confirmer Phase B
> G8 S1 scan) + @huggingface/transformers tokenizer iframe +
> presidio-analyzer 2.2.362 + GLiNERRecognizer coord-side.
> Decision `sprint21_kickoff.md §D2`. Rust-wasm iframe realignement
> Option G reporté S22+ via tech debt T-NN+2 (blocked tract opset
> 18 max vs GLiNER opset 19 + wasm32-browser zero-precedent +
> gline-rs a choisi ort pas tract). »

**Implications code** :

- Nouveau directory `web/src/sdk/pii/` : `index.ts` + `wrapper.ts`
  (tokenize → ONNX inference → span classifier) + `fallback.ts`
  (regex curated) + `policy.ts` (charge config) + tests Vitest +
  Playwright.
- Dependencies npm ajoutées `web/package.json` :
  `"onnxruntime-web": "1.24.3"`, `"@huggingface/transformers":
  "4.x"` (tokenizer only).
- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/
  pii_redactor.py` : `class PiiRedactor(presidio_analyzer +
  presidio_anonymizer + GLiNERRecognizer)`.
- Dependencies Python ajoutées `packages/nexus-coordinator/
  pyproject.toml` :
  `presidio-analyzer = {version = "2.2.362", extras = ["gliner"]}`,
  `presidio-anonymizer = "2.2.362"`.
- Config `~/.sbfb/pii_redaction_policy.toml` + hot-reload
  file-watcher (Python `watchdog` ou équivalent, cohérent pattern
  S20 Phase C `pow_policy_loader.rs` Rust).
- Bridge méthode `pii_redact` ajoutée whitelist
  `packages/nexus-sdk/src/nexus_sdk/bridge.py` + côté JS `web/src/
  lib/sbfb-bridge.js`.

**Cap règle G1 extension** : ≥ 1 alternative concurrente récente
comparée (Gretel gliner-bi-small 10/2024, urchade multi-pii,
nvidia gliner-pii 2025-10, GLiNER.js npm mars 2025, redact-core
2026-02, Full Rust tract 2026-02, Transformers.js v4 2026-02).
✓ rigor extension satisfaite.

**G8 S1 scan obligatoire Phase B** : avant 1re ligne de code
Phase B, fetch fresh model card HF `knowledgator/gliner-pii-edge-
v1.0` + `config.json` + opset export pour figer backbone + taille
quantized. Verdict possible EXECUTE plan-as-is / SCOPE-CUT-CONSISTENT
(switch modèle base/small si edge insuffisant) / DESIGN-CONFLICT
(blocker opset onnxruntime-web).

### D3 — Output filter (invisible chars + prompt echo EED detection)

**Retenu** : `LLM Guard 0.3.16` (ProtectAI MIT 2025) coord-side
Python.

Deux scanners combinés post-worker-response, avant
`validate_task_response` (S20 Phase D structured output garde-fou
existant) :

1. **Invisible chars scanner** : `InvisibleText` scanner LLM Guard
   avec curated list (zero-width U+200B-U+200D, Byte Order Mark
   U+FEFF, PUA U+E000-U+F8FF, Tag chars U+E0020-U+E007F, UTF-16
   surrogates U+DB40-U+DC7F). Strip tous les chars catégories
   `Cf`/`Co`/`Cs` + whitelist `Cf` légitime (RLO/LRO U+202A-
   U+202E nécessaires pour Arabe/Hébreu/Persan apps
   multilingues).

2. **Prompt echo detection** : EED (Extended Edit Distance
   Levenshtein) + SM (Substring Match) exact.
   - SM fallback first : si `system_prompt.as_str()` apparaît
     littéralement dans `response.output`, fail-close (prompt
     leak direct).
   - Sinon EED check : calcule `levenshtein(system_prompt,
     response_window)` glissant sur N-grammes (N=50 tokens
     default). Si `EED/len(system_prompt) < 0.15` (i.e.
     similarity > 0.85) → fail-close.
   - **Seuil 0.85 = tuning empirique Phase C S21** : documenté
     inline code commentaire + config `~/.sbfb/output_filter_
     policy.toml` hot-reload pour override par opérateur
     expérimenté. Algo justifié PLeak CCS'24 (EED métrique
     standard) mais seuil empirique, pas cité paper.

Integration : hook `packages/nexus-coordinator/src/nexus_
coordinator/output_filter.py` wire dans le path
`task_response_validator.py` (avant le `validate_task_response`
garde-fou S20 Phase D).

**Rejetés** :

- **`NeMo Guardrails` (NVIDIA)** 0.11+ Colang 2.0 : Python 3.8
  dropped 0.11, stack NVIDIA overkill pour scope SBFB (pas de
  dépendance GPU côté coord).
- **`guardrails-ai` + Hub (`detect_prompt_injection`)** : détection
  via LLM secondaire scoring (Rebuff-like) = couteux + pas pour
  output echo detection.
- **Unicode category filter strict `Cc|Cf|Co|Cs`** seul : casse
  RLO/LRO Arabe/Hébreu légitime. Nécessite whitelist RLO/LRO
  (Unicode Bidi formatting U+202A-U+202E). Plus complexe à
  whitelist manuellement vs `InvisibleText` LLM Guard
  pre-curated.
- **Rabin-Karp rolling hash** pour prompt echo : pas de paper
  dédié 2024-2025, académia préfère EM/SM/EED. Reinvente EED pour
  usage identique.
- **LLM-based semantic detection** : out-of-scope S21 (S23+
  si besoin runtime sémantique).
- **ProxyPrompt defense proactive** (arXiv 2505.11459, mai 2025,
  94.70% protection) : proactive = prompt replacement pre-inject,
  hors scope reactive detection S21.

**Alternatives concurrentes récentes comparées** : NeMo
Guardrails 0.11+ (2025), guardrails-ai (active), ProxyPrompt
(mai 2025). ✓ rigor extension satisfaite.

**Implications code** :

- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/
  output_filter.py` : `class OutputFilter(llm_guard.scanners.
  InvisibleText + custom EED detector)`.
- Dépendance Python `llm-guard = "0.3.16"` dans `pyproject.toml`.
- Config `~/.sbfb/output_filter_policy.toml` hot-reload pattern
  cohérent PII redaction.
- Tests Python coord : scénarios reconstructions prompts (PLeak-
  like), invisible chars cases (zero-width, PUA), calibration EED
  seuil.

### D4 — Quarantine queue (SQLite WAL local coord-side + CLI manual flush)

**Retenu** : SQLite WAL local coord-side schema :

```sql
CREATE TABLE quarantine_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    sender_pubkey BLOB NOT NULL,         -- Ed25519 32 bytes
    payload_bytes BLOB NOT NULL,
    received_at_epoch_s INTEGER NOT NULL,
    rate_strikes INTEGER NOT NULL,
    pow_status TEXT NOT NULL,            -- 'valid' | 'missing' | 'invalid'
    flush_status TEXT NOT NULL           -- 'pending' | 'flushed' | 'dropped'
);
CREATE INDEX idx_quarantine_received ON quarantine_messages(received_at_epoch_s);
CREATE INDEX idx_quarantine_sender ON quarantine_messages(sender_pubkey);
```

TTL **automatique 15 min** : tokio task périodique supprime les
entrées `received_at_epoch_s < now - 900`. Drop silencieux (log
info only, pas warn).

**Manual flush CLI** : `sbfb quarantine list|flush|drop` via REST
endpoint loopback `/quarantine/list`, `/quarantine/flush/{id}`,
`/quarantine/drop/{id}` avec auth bearer X-SBFB-Token + Host +
Origin check pattern S16.

Pattern réutilise **S19 Phase D `f238d31` `upload_queue.py`**
(validation production delayed upload queue).

**Rejetés** :

- **In-memory BTreeMap TTL** : volatil au crash daemon, perte
  messages quarantine. Pas de persistance = pas d'audit trail.
- **Gossip-wide diffusion** (quarantine propagé entre peers) :
  amplification DoS vector + complexité consensus + pas
  d'équivalent public P2P (libp2p gossipsub = graylist automatique,
  pas quarantine hold manual flush).
- **Redis infra externe** : ajoute dep infra hors-scope pre-launch
  (violates CLAUDE.md §Stack simple pre-launch).
- **Admin UI custom** (web/) : nouveau pattern sans précédent
  SBFB vs CLI cohérent pattern S20 `sbfb canary ack`. CLI plus
  simple, opérateur-facing.

**Alternatives P2P récentes comparées** : `libp2p gossipsub v1.1`
graylist threshold (automatic, sans manual flush) ; Tor DoS
prevention cooldown exponentiel par IP ; Filecoin/ETH2.0
gossipsub (attack-resilient mesh + peer profiles sans manual
flush) ; AztecProtocol `topicValidators` `ACCEPT/REJECT/IGNORE`
(sans hold queue). **Note** : pattern SBFB-spécifique assumé.
Documenté `plan.md §Risks` + design doc Phase A dédié
`.planning/research/S21_phase_D_quarantine_design.md`.

**Implications code** :

- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/
  quarantine_queue.py` : `class QuarantineQueue(sqlite3.connect
  WAL mode)` + CLI endpoint handlers.
- Nouveau module `crates/nexus-shell-daemon/src/api/quarantine.rs` :
  REST routes `/quarantine/*` avec auth bearer pattern S16.
- CLI sbfb-launcher : sous-commande `sbfb quarantine list|flush|
  drop` appelant endpoints loopback.
- Design doc pre-Phase-A : `.planning/research/S21_phase_D_
  quarantine_design.md` couvrant schema evolution + TTL clock
  semantics (NTP sync, received_at vs inserted_at) + security
  manual flush + cardinality expected + interaction gossip layer.

### D5 — Cap G7 carry-overs + reclassifications

**Retenu** (cf. `sprint21_carry_summary.md`) :

| # | Item | Classification S21 |
|---|---|---|
| 1 | **Meta-1 Radicle-v1.0 activation tracking** | **Carry confirmé** §Meta-track S21 (re-carry S18→S19→S20→S21) |
| 2 | **C-PLAN-1 plan docs fix** wire-point divergence S20 | **Carry confirmé** — chore Phase 0 ou integré Phase E |
| 3 | **Rate-limit per-(consumer, worker, model)** | **Scope S21 Phase A intégré** (pas carry — natif S21) |
| 4 | **Client-side redaction SDK** | **Scope S21 Phase B intégré** (pas carry — natif S21) |
| 5 | **Tech debt T-NN canary JCS** | **Hors cap** — PATTERNS.md tech debt (optionnel batch Phase E) |
| 6 | **Tech debt T-NN+1 registry verify Ed25519** | **Hors cap** — PATTERNS.md + décision maturité pre-launch |
| 7 | **Tech debt T-NN+2 iframe Rust-wasm realignement** | **Hors cap** — PATTERNS.md S22+ blocked |

**Cap G7 respecté** : **2/2** (Meta-1 + C-PLAN-1).

**Rejetés (alternatives pas rebattables)** :
- Abandonner Meta-1 (DEPRECATED.md) : casse engagement S18 Phase E3
  + runbook `MIRROR_FALLBACK.md §3` self-contained. Non-starter
  pre-v1.0.
- Livrer Rust-wasm realignement iframe dès S21 : blocked factuellement
  (cf. research G2 + design_review §D2). S22+ re-evaluate.
- Dépasser cap 2 carry : violerait `README.md §6.2.1` =
  P1 auditor S22. Cap existe pour rendre visible le glissement.

**Implications** : section §6 Items carry/dette ci-dessous + plan
§E tech debt batch optionnel.

### Acknowledged review findings (G1)

Rapport Design Review Board G1 : `.planning/active/sprint21_design_
review.md` (reviewer agent Explore indépendant, ~30 min timebox,
2026-04-18). **Scoring** : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (3 ⚠️ sur 5, pas 100% ✅ = reviewer a
trouvé findings réels).

**D1 ✅** : noted, no action required. Source governor 0.10.2
récente + alternatives comparées (tokio-rate-limit 0.8 nov 2025
vérifié, leaky-bucket, ratelimit_meter archived, async-rate-limiter
non-keyed). Date exacte release crates.io non-canonique gap mineur
acceptable.

**D2 ⚠️** : **adjust appliqué dans ce kickoff + commit open S21** :
1. Entrée `audited_findings` frontmatter `HARDENING_ROADMAP.md`
   datée 2026-04-18 ajoutée dans ce commit, requalifiant D2 SDK
   vs libellé S17 original.
2. §D2 implementations note explicite « G8 S1 scan obligatoire
   Phase B » avant 1re ligne de code (fetch fresh model card HF
   pour backbone + config.json + opset export).
3. Ambiguïté backbone `gliner-pii-edge-v1.0` (ModernBERT vs
   DeBERTa-v3) notée comme zone non-vérifiable résolue Phase B
   preflight.

**D3 ⚠️** : adjust reporté Phase B/C S21 :
1. Seuil EED 0.85 = tuning empirique documenté inline code
   commentaire Phase C + config `~/.sbfb/output_filter_policy.toml`
   hot-reload pour override opérateur.
2. Phase C design doc `.planning/research/S21_phase_C_output_
   filter_design.md` couvrira methodology tuning + corpus tests
   retenus.

**D4 ✅** : noted. Design doc Phase A dédié `.planning/research/
S21_phase_D_quarantine_design.md` acté pre-req normal (pattern
S20 Phase B/D design docs).

**D5 ✅** : noted, no action required. Cap 2/2 respecté.

**Aucun blocage P0/P1 pour Phase A S21**. Tous les ⚠️ sont points
d'amélioration redactionnelle acknowledge et adjusted inline dans
ce kickoff avant gel + remediations reportées Phase 0 / pre-phase
sans bloquer Phase A. Design Review Board mission accomplie.

---

## 5. Plan Phase outline

### Phase 0 — Audit Sprint 20 (DÉJÀ JOUÉ — verdict PASS)

`sprint20_audit_findings.md` migré vers `archive/v1.2/` dans ce
commit `chore(planning): open Sprint 21` (pattern S18/S19/S20).

### Phase A — Rate-limit sliding-window multi-tier (+15 tests)

Scope : `governor 0.10.2` + `tower-governor 0.8` middleware, DashMap
keyed `(ConsumerId, WorkerId, ModelId)`, policy file hot-reload,
éviction 60 s, tests saturation + live reload.

Design doc pre-Phase-A : `.planning/research/S21_phase_A_rate_limit_
design.md` (cap G4 pattern S20 big rocks).

Livrable commit : `feat(sprint21): Phase A — rate-limit sliding-
window multi-tier per-(consumer, worker, model) via governor GCRA`

### Phase B — Client-side PII redaction SDK iframe (+15 tests)

Scope : `web/src/sdk/pii/` wrapper JS (onnxruntime-web +
@huggingface/transformers tokenizer + modèle knowledgator/gliner-pii-
edge-v1.0 + regex fallback), integration bridge `sbfb-bridge.js`
méthode `pii_redact`, tests Vitest + Playwright iframe réels.

**G8 S1 scan obligatoire** : fetch fresh HF model card +
config.json pour backbone + opset export AVANT 1re ligne de code.
Output `sprint21_phase_B_preflight.md` verdict EXECUTE / SCOPE-CUT-
CONSISTENT / DESIGN-CONFLICT.

Livrable commit : `feat(sprint21): Phase B — client-side PII
redaction SDK iframe (onnxruntime-web + GLiNER PII edge)`

### Phase C — Coord-side PII redaction + output filter (+15 tests)

Scope : `packages/nexus-coordinator/src/nexus_coordinator/
pii_redactor.py` (Presidio 2.2.362 + GLiNERRecognizer même modèle
iframe) + `output_filter.py` (LLM Guard InvisibleText scanner +
EED prompt echo detection). Hook pre-dispatch worker. Tests Python.

Design doc pre-Phase-C : `.planning/research/S21_phase_C_output_
filter_design.md` (EED methodology + tuning corpus + seuil 0.85
rationale).

Livrable commit : `feat(sprint21): Phase C — coord-side PII
redaction + output filter (Presidio GLiNER + LLM Guard + EED echo)`

### Phase D — Quarantine queue SQLite WAL + manual flush CLI (+10 tests)

Scope : `packages/nexus-coordinator/src/nexus_coordinator/
quarantine_queue.py` SQLite WAL + tokio task TTL 15 min + REST
endpoints `/quarantine/*` + CLI `sbfb quarantine list|flush|drop`.
Tests Python + Rust daemon loopback.

Design doc pre-Phase-A (avant Phase A du code D) :
`.planning/research/S21_phase_D_quarantine_design.md` (schema +
TTL + security + cardinality bench expected).

Livrable commit : `feat(sprint21): Phase D — quarantine queue
SQLite WAL + manual flush CLI`

### Phase E — Tech debt batch S20 carry (canary JCS + registry verify + plan docs fix) (+5 tests)

Scope optionnel (peut être split chore inline S20-followup si
l'executeur préfère) :
- T-NN canary JCS : migrer `canary_wire_bytes` → `serde_jcs::to_vec`
  + test cross-language snapshot.
- T-NN+1 CanaryRegistry verify Ed25519 at ingest (décision maturité
  pre-launch tranchée Phase E).
- C-PLAN-1 plan docs fix : edit `archive/v1.2/sprint20_plan.md §6.2
  + §6.4` wire-point reference OU note en tête.
- Update tech debt PATTERNS.md entries (T-NN, T-NN+1, T-NN+2).

Livrable commit : `feat(sprint21): Phase E — tech debt batch (canary
JCS + registry verify Ed25519 + plan docs fix)`

### Phase F — Consolidation + verification + audit plan S22 (docs only)

Update CLAUDE.md + SPRINT_LOG.md row S21 + memory fusion +
`sprint21_verification.md` + `sprint21_audit_plan.md`. Migration
PARA tous `sprint21_*.md` → `archive/v1.2/`.

Livrable commit : `chore(sprint21): Phase F — wrap-up +
verification + audit plan S22 + migrate planning`

---

## 6. Items carry/dette

### Items carry confirmés S21 (cap G7 = 2/2)

- [x] **Meta-1 Radicle-v1.0 activation tracking** : re-carry
  S18→S19→S20→S21 confirmé. Owner FlowUP, deadline jour tag `v1.0`
  go-public. Runbook `docs/release/MIRROR_FALLBACK.md §3.1-3.8`
  self-contained. Phase F S21 re-carry S22 si v1.0 pas tag.
- [x] **C-PLAN-1 plan docs fix** wire-point divergence S20 : fix
  plan archivé `.planning/archive/v1.2/sprint20_plan.md §6.2 + §6.4`
  via chore Phase 0 S21 OU intégré Phase E batch tech debt. Owner
  S21 executor.

### Items reclassifiés (NON-carry — cf. `sprint21_carry_summary.md`)

- [scope] **Rate-limit per-(consumer, worker, model)** → intégré
  Phase A S21 directement.
- [scope] **Client-side PII redaction SDK** → intégré Phase B + C
  S21 directement.
- [tech-debt] **T-NN canary JCS** → PATTERNS.md + Phase E batch
  optionnel.
- [tech-debt] **T-NN+1 CanaryRegistry verify Ed25519** → PATTERNS.md
  + décision maturité Phase E.
- [tech-debt] **T-NN+2 iframe PII SDK Rust-wasm realignement
  Option G** → PATTERNS.md + S22+ blocked (tract opset / ort-wasm
  stability).

---

## 7. Traçabilité scope

Items **nouveaux Sprint 21** :
- Rate-limit sliding-window multi-tier (Phase A)
- Client-side PII redaction SDK iframe (Phase B)
- Coord-side PII redaction + output filter (Phase C)
- Quarantine queue SQLite WAL + CLI manual flush (Phase D)

Items **carry/dette** :
- Meta-1 Radicle-v1.0 (carry S18→S19→S20→S21)
- C-PLAN-1 plan docs fix (carry S20→S21)
- T-NN canary JCS (tech debt, batch Phase E optionnel)
- T-NN+1 registry verify Ed25519 (tech debt, décision Phase E)
- T-NN+2 iframe Rust-wasm realignement (tech debt S22+)

Items **différés** :
- Kudos-weighted gossip admission → S22
- Sandbox tool-calling allow-list strict → S22
- Redundancy voting `Task.redundancy_factor` → S22
- Ephemeral workers + VRAM wipe → S23
- Honeypot Eclipse detection → S23
- Re-run sampling + DNS fallback DHT → S24
- Arti Tor bridge integration → S25
- Domain fronting Snowflake-WebRTC → S25
- PQC migration ML-DSA + ML-KEM → S26+ (HNDL liability)
- Hardware keystore TPM/SE/StrongBox → S22+
- HPKE envelope peer-restore → S22+
- `actions/checkout@v4` pin SHA sweep → sprint ops futur

---

## 8. Scope cuts (PAS dans ce sprint)

Cf. §7 ci-dessus pour détail. En résumé :

- **Kudos-weighted gossip admission** : S22 (scope Sybil
  resistance)
- **Sandbox tool-calling allow-list strict** : S22
- **Redundancy voting** : S22
- **Ephemeral workers + VRAM wipe** : S23
- **Honeypot Eclipse detection** : S23
- **Re-run sampling + DNS fallback DHT** : S24
- **Arti Tor bridge integration** : S25
- **Domain fronting Snowflake-WebRTC** : S25
- **PQC migration ML-DSA + ML-KEM** : S26+
- **Hardware keystore TPM/SE/StrongBox** : S22+ (`trait KeyStore`
  abstraction livrée S20 Phase A prête)
- **HPKE envelope peer-restore** : S22+
- **Rust-wasm iframe PII SDK realignement Option G** : S22+
  (tech debt T-NN+2, blocked tract/ort-wasm)
- **LLM-based semantic output detection** : S23+
- **ProxyPrompt proactive defense** : S23+
- **spaCy NER wasm** : DEPRECATED (drift roadmap S17 requalifié
  `audited_findings 2026-04-18`)
- **`actions/checkout@v4` pin SHA sweep** : sprint ops futur

---

## 9. Audit gate pattern — rappel

Phase 0 Sprint 20 audit joué pre-S21 session 2026-04-18, verdict
PASS (commit `66a3a7c`). Phase F S21 produit `sprint21_audit_plan.
md` pour Sprint 22 Phase 0. Pattern permanent depuis Sprint 7.

Meta-1 Radicle-v1.0 tracking re-carry explicite dans
`sprint21_audit_plan.md §meta-track` (prévu Phase F).

**Rigor signal G4** : verdict audit S21 Phase 0 (session S22
fraîche) exigera ≥ 1 P2+ documenté. Si 0 finding = CONCERN pas PASS.

**Design Review Board G1** : agent Explore indépendant exécuté sur
draft D1..D5, rapport dans `sprint21_design_review.md`. Planner
acknowledge chaque ⚠️ / ❌ dans §4 « Acknowledged review findings »
(section ci-dessus, remplie après réception rapport). Verdict
CONDITIONAL PASS remediations intégrées pre-commit.

**G8 phase pre-flight** (§6.9) : avant 1re ligne de code de chaque
phase, invoquer skill `nexus-phase-preflight` pour 4 scans
factuels (S1 SOTA / S2 historical / S3 threat model / S4 wire
format). Phase B S21 DOIT scan S1 fresh modèle HF pour backbone
+ opset + config.json. Verdict EXECUTE / SCOPE-CUT-CONSISTENT /
DESIGN-CONFLICT.

---

## 10. Checkpoint de validation

Status : **draft en attente Design Review Board G1 output
acknowledged = DONE 2026-04-18**.

Points validés :

1. **D1 governor 0.10.2 GCRA** : OK — seul mature Rust 2026 avec
   GCRA + keyed, alternatives vérifiées (tokio-rate-limit 0.8
   manque GCRA).
2. **D2 Option 7 Custom JS iframe + Presidio coord** : OK
   defense-in-depth — factuel Rust-first iframe défrichage
   non-budgetable (tract opset gap + wasm32-browser zero precedent
   + gline-rs a rejeté tract). Tech debt T-NN+2 S22+ carry.
3. **D3 LLM Guard InvisibleText + EED prompt echo** : OK — algo
   academique PLeak CCS'24, seuil 0.85 tuning empirique Phase C
   documenté.
4. **D4 SQLite WAL + CLI manual flush** : OK — pattern S19 Phase D
   reuse, design doc Phase A dédié pre-req.
5. **D5 cap G7 2/2** : OK — Meta-1 + C-PLAN-1, tech debt hors cap
   dans PATTERNS.md.

**Fichiers untracked** : `sprint20_phase_F_review.md` dans active/
(duplicate NOISE auto-généré par hook/skill background pendant
audit S20, contenu factuellement incorrect sur B-1 D-1) —
supprimé dans ce commit d'ouverture S21.

---

**Note de placement** : ce kickoff est écrit directement dans
`.planning/active/` avec `sprint21_plan.md` + `sprint21_design_
review.md` (produit par agent G1) + `sprint21_carry_summary.md`.
`sprint20_audit_findings.md` migré `archive/v1.2/` dans ce même
commit `chore(planning): open Sprint 21`.
