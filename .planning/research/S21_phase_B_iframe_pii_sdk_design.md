---
sprint: 21
phase: B
topic: iframe_pii_sdk_design
date: 2026-04-19
agent: Opus 4.7 (session fraîche post-Phase-A commit `63afe4e`)
word_count: ~1900
archive_rationale: §6.11 README — design doc pre-req Phase B (pattern S19 Phase B/C/D/E design docs, pattern S20 Phase B/D design docs)
---

# Sprint 21 Phase B — Design doc pre-Phase-B : iframe PII redaction SDK

## 1. Architecture décision (host shell vs iframe-embedded)

**Décision retenue** : le SDK PII vit **côté host shell React**
(pas embedded dans chaque iframe user). Exposé aux apps iframe
via une 4e méthode whitelist `pii_redact` du bridge postMessage
(pattern S13).

### 1.1 Rationale

Trois options étaient possibles architecturalement :

| Option | Descriptif | Verdict |
|---|---|---|
| **(a) SDK embedded dans chaque iframe user** | l'app publisher intègre onnxruntime-web + GLiNER model dans sa propre build, charge localement avant postMessage | **REJETÉ** : explosion bundle size côté publisher (45.8 MB + wasm ORT + tokenizer), N×load du même modèle pour N iframes actives, pas de cohérence cross-app, CSP iframe user peut bloquer `wasm-unsafe-eval` selon contexte coord |
| **(b) SDK dans host shell React, exposé via bridge** | 1 load du modèle dans host, apps appellent `bridge.piiRedact(text)` via postMessage | **RETENU** : load unique shared, CSP du host shell (pas user iframe), defense uniforme cross-app, cohérent pattern S13 bridge whitelist additive (`task_submit`, `storage_get`, `storage_set` → + `pii_redact`) |
| **(c) SDK coord-side uniquement (Phase C seule)** | pas de layer 1 client, coord fait tout | **REJETÉ** : pas de defense-in-depth, un bypass bridge = fuite PII network-wide, contredit kickoff §D2 Couche 1 |

**Option (b) retenue par alignement kickoff §D2 + pattern bridge
S13 additif**. Kickoff §D2 disait « Integration : exposé via
`sbfb-bridge.js` postMessage bridge (pattern S13) — nouvelle
méthode whitelist `pii_redact(text, policy)` dans le bridge coord
side ». Le mot « coord side » dans kickoff = ambigu : le bridge
côté host shell dispatch vers coord POUR `task_submit` et
`storage_*`, mais pour `pii_redact` le dispatch est **local au
host shell** (pas coord round-trip) car Phase C coord-side layer
est un 2e filet independant et chaque layer fait son propre
detect-redact.

### 1.2 Flow séquentiel

```
┌──────────────────┐   postMessage       ┌──────────────────────┐
│  iframe user app │ ──────────────────> │  host shell (React)  │
│  (app publisher) │   "pii_redact" req  │  useBridge listener  │
└──────────────────┘                     └──────────────────────┘
                                                     │
                                                     │  local dispatch
                                                     │  (no coord call)
                                                     ▼
                                         ┌──────────────────────┐
                                         │ SDK PII (web/src/    │
                                         │ sdk/pii/)            │
                                         │ ┌──────────────────┐ │
                                         │ │ wrapper.ts       │ │
                                         │ │  (ONNX + tokenizr│ │
                                         │ │   OR fallback)   │ │
                                         │ └──────────────────┘ │
                                         │ ┌──────────────────┐ │
                                         │ │ fallback.ts      │ │
                                         │ │  (regex curated) │ │
                                         │ └──────────────────┘ │
                                         └──────────────────────┘
                                                     │
                                                     │  redacted text
                                                     ▼
┌──────────────────┐   postMessage       ┌──────────────────────┐
│  iframe user app │ <────────────────── │  host shell (React)  │
│                  │   response (id)     │                      │
└──────────────────┘                     └──────────────────────┘
```

App iframe utilise ensuite le `redactedText` pour `bridge.submitTask({ prompt: redactedText })` → coord → worker. Le worker
ne voit jamais les PII originales. Layer 2 coord-side (Phase C)
re-redact en defense-in-depth si layer 1 a manqué quelque chose
ou a été bypassé par une app malveillante qui n'appelle pas
`pii_redact`.

## 2. Backbone modèle confirmé (scan S1 closed)

Source primary : `.planning/research/S21_research_backbone_
resolution.md` (archive 2026-04-18) + préflight G8
`sprint21_phase_B_preflight.md` (2026-04-19 HEAD `63afe4e`).

| Property | Value |
|---|---|
| Modèle | `knowledgator/gliner-pii-edge-v1.0` |
| Licence | Apache-2.0 |
| Backbone | **ModernBERT** (`jhu-clsp/ettin-encoder-32m`, 32M params, 10 layers, hidden 384, context 8192) |
| Tokenizer | BPE byte-level OLMo-style (`PreTrainedTokenizerFast`), vocab 50 370 |
| ONNX quantized | `model_quint8.onnx` = **45.8 MB** |
| F1 benchmark | 75.50 % sur `synthetic-multi-pii-ner-v1` |
| Special tokens | `[CLS]=50281`, `[SEP]=50282`, `[PAD]=50283`, `[MASK]=50284`, `<<ENT>>`, `<<SEP>>`, `|||EMAIL_ADDRESS|||`, `|||PHONE_NUMBER|||`, `|||IP_ADDRESS|||` |
| Opset ONNX | compat ORT Web 1.24.3 (exports `optimum` émettent opset 14-18, ORT supporte ≤ 21 depuis 1.20) |

**Fallback model** (si perf iframe insuffisant bench runtime) :
`onnx-community/gliner_multi_pii-v1` (349 MB, multilingue) via
lazy-load switch dans `wrapper.ts` (config policy). Non-livré
Phase B S21 — option ouverte pour future session.

## 3. Architecture code (arborescence)

```
web/src/sdk/pii/
├── index.ts                — public exports (detect, redact, configure, types)
├── wrapper.ts              — GlinerPiiDetector (ONNX inference + tokenizer)
├── fallback.ts             — regex curated detector (email, phone, CC, SSN, IBAN)
├── policy.ts               — PiiPolicy type + default policy + entity list
└── __tests__/
    ├── fallback.test.ts    — 6 tests (regex detect + redact)
    ├── policy.test.ts      — 3 tests (default, threshold, disabled)
    └── wrapper.test.ts     — 2 tests (fallback-on-error, policy-respect)

web/public/
└── models/                 — gitignored, model_quint8.onnx downloadé on-demand
    └── .gitkeep            — placeholder pour déférer download runtime
```

### 3.1 `policy.ts` — types et defaults

```typescript
export interface PiiPolicy {
  enabled: boolean;                // disable = pass-through
  entities: PiiEntity[];           // whitelist des entités à redact
  replacement: string;             // placeholder, tokenized `{ENTITY}`
  confidence_threshold: number;    // 0..1, findings < threshold skippés
  use_model: boolean;              // false = fallback regex only
}

export type PiiEntity =
  | "PERSON"
  | "EMAIL_ADDRESS"
  | "PHONE_NUMBER"
  | "CREDIT_CARD"
  | "SSN"
  | "IBAN"
  | "IP_ADDRESS"
  | "MEDICAL_LICENSE"
  | "US_PASSPORT"
  | "URL";

export const DEFAULT_POLICY: PiiPolicy = {
  enabled: true,
  entities: ["PERSON", "EMAIL_ADDRESS", "PHONE_NUMBER",
             "CREDIT_CARD", "SSN", "IBAN"],
  replacement: "[REDACTED:{ENTITY}]",
  confidence_threshold: 0.5,
  use_model: true,
};
```

### 3.2 `fallback.ts` — regex curated

5 regex curated coverant les entités haute-fréquence **sans
context NER requis** :

| Entité | Regex base | Post-validation |
|---|---|---|
| EMAIL_ADDRESS | RFC-5322 subset `[\w.+-]+@[\w-]+\.[\w.-]+` | TLD ≥ 2 chars |
| PHONE_NUMBER | E.164 `\+[1-9]\d{1,14}` + USA `\(\d{3}\) \d{3}-\d{4}` | min 10 digits |
| CREDIT_CARD | `\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b` | **Luhn check** (filter false positives) |
| SSN | `\b\d{3}-\d{2}-\d{4}\b` | skip `000-xx-xxxx` / `666-xx-xxxx` invalides |
| IBAN | `\b[A-Z]{2}\d{2}[\s]?[A-Z0-9\s]{11,30}\b` | optional MOD 97 check (non-livré Phase B, follow-up) |

Chaque regex produit `PiiFinding { entity, start, end, confidence = 1.0 }`. `redact()` applique `policy.replacement` avec tokenization `{ENTITY}` remplacement.

Fallback trigger automatique dans 3 cas :
1. `policy.use_model === false` (opérateur force regex-only)
2. Model load échec runtime (network error, CORS, file 404)
3. Model inference throw (corrupt ONNX, WASM OOM)

### 3.3 `wrapper.ts` — GLiNER ONNX runtime

```typescript
export class GlinerPiiDetector {
  private session: InferenceSession | null = null;
  private tokenizer: PreTrainedTokenizer | null = null;
  private modelLoadError: Error | null = null;

  async ensureLoaded(modelUrl: string): Promise<boolean> {
    if (this.session && this.tokenizer) return true;
    if (this.modelLoadError) return false;
    try {
      const [ort, tf] = await Promise.all([
        import("onnxruntime-web"),
        import("@huggingface/transformers"),
      ]);
      this.session = await ort.InferenceSession.create(modelUrl);
      this.tokenizer = await tf.AutoTokenizer.from_pretrained(
        "knowledgator/gliner-pii-edge-v1.0"
      );
      return true;
    } catch (err) {
      this.modelLoadError = err as Error;
      return false;
    }
  }

  async detect(text: string, policy: PiiPolicy): Promise<PiiFinding[]> {
    const loaded = await this.ensureLoaded(DEFAULT_MODEL_URL);
    if (!loaded) return fallbackDetect(text, policy);
    // tokenize → ONNX run → span classifier → filter by threshold
    // ... (implementation detail inline wrapper.ts)
  }
}
```

Le `GlinerPiiDetector` est **singleton** au host shell (1 load
pour toutes les apps). Alors que les regex fallback sont pure
functions stateless.

**Lazy-load strict** : `import("onnxruntime-web")` + `import
("@huggingface/transformers")` **dynamic imports** → le main bundle
React n'embarque rien. Vite émet un chunk séparé `PiiDetector-
*.js` (non-budget size-limit).

**Modèle location** : `DEFAULT_MODEL_URL = "/models/gliner-pii-
edge-v1.0.onnx"` (servi statique depuis `web/public/models/`).
Gitignored car 45.8 MB — téléchargé on-demand par script ou
manual fetch runtime (cf. §5 Loading strategy).

### 3.4 `index.ts` — exports publics

```typescript
export type { PiiPolicy, PiiEntity, PiiFinding } from "./policy";
export { DEFAULT_POLICY } from "./policy";
export { GlinerPiiDetector } from "./wrapper";
export { fallbackDetect, redact } from "./fallback";
export { detectAndRedact } from "./index";  // high-level helper
```

## 4. Integration bridge (protocol extension)

### 4.1 `web/src/bridge/protocol.ts`

```typescript
export const BridgeMethodSchema = z.enum([
  "task_submit",
  "storage_get",
  "storage_set",
  "pii_redact",      // Sprint 21 Phase B
]);
```

Payload schema additionnel :

```typescript
export const PiiRedactPayloadSchema = z.object({
  text: z.string().max(50_000),    // hard cap 50KB text
  policy: z.object({/* partial override PiiPolicy */}).optional(),
});
```

### 4.2 `web/src/bridge/useBridge.ts`

Nouveau case dans le switch dispatch :

```typescript
case "pii_redact": {
  const { text, policy } = PiiRedactPayloadSchema.parse(req.payload);
  const effective = { ...DEFAULT_POLICY, ...(policy ?? {}) };
  const redacted = await detectAndRedact(text, effective);
  return { redacted_text: redacted.text, findings_count: redacted.findings.length };
}
```

**Local dispatch** (pas de coord call) : `detectAndRedact` lance
le singleton detector local au host shell. Le coord (Phase C)
reçoit le `redacted_text` via `task_submit` downstream (defense-
in-depth layer 2).

### 4.3 `web/public/sbfb-bridge.js`

```javascript
/**
 * Redact PII from text via the host SDK. Sprint 21 Phase B.
 * @param {string} text — input text to scan
 * @param {Object} [policy] — partial policy override
 * @returns {Promise<{ redacted_text: string, findings_count: number }>}
 */
piiRedact(text, policy) {
  return this._call("pii_redact", { text: text, policy: policy });
}
```

## 5. Loading strategy (bundle + model)

### 5.1 Bundle

`onnxruntime-web` + `@huggingface/transformers` pesent ~5-8 MB
combinés (minified). Stratégie **lazy-load strict** :

- Import dynamic `import()` dans `wrapper.ts::ensureLoaded()`.
- Vite splitting auto → chunk `PiiDetector-*.js` séparé.
- Main bundle React `assets/index-*.js` **inchangé** (budget ≤ 50 KB préservé).
- Chunks ORT/transformers **non listés dans `.size-limit.json`**
  (out-of-budget intentionnel — consumers opt-in par call bridge).

### 5.2 Modèle ONNX (45.8 MB)

**Non embarqué dans le repo git** (gitignore pattern `web/public/
models/`). Phase B livre :
- `web/public/models/.gitkeep` placeholder dir
- Un `scripts/download-pii-model.sh` (optionnel — pas livré cette
  phase, tech debt follow-up si besoin runtime prod)
- README dev note : pour activer le modèle en dev/runtime,
  télécharger `model_quint8.onnx` depuis
  `https://huggingface.co/knowledgator/gliner-pii-edge-v1.0/
  resolve/main/onnx/model_quint8.onnx` vers `web/public/models/
  gliner-pii-edge-v1.0.onnx`.

**Fallback transparent** : en dev + CI sans modèle, `wrapper.ts
::ensureLoaded()` retourne `false` → redact bascule automatiquement
sur `fallback.ts` regex. **Les tests Phase B tournent sur ce
path** (pas de download modèle en CI).

### 5.3 CSP iframe considerations

Plan §5.2 mentionnait « CSP iframe requirements (`wasm-unsafe-
eval` directive, SAB cross-origin isolation si threads requis) ».

**Résolution architecture retenue** : puisque ORT tourne **dans
le host shell** (origin `http://localhost:5173/` dev, futur
`app://` tauri-like), pas dans l'iframe user :

- **CSP iframe user** : pas de modification requise. Les apps
  publiées gardent `sandbox="allow-scripts"` + CSP `connect-src
  'none'` existante Sprint 12. Aucune relaxation du sandbox user.
- **CSP host shell** : Vite dev serveur n'impose pas de CSP par
  défaut. Production build : aucune restriction sur `script-src`
  ni `wasm-unsafe-eval` dans le bundler actuel. Si CSP ajouté
  futur, inclure `wasm-unsafe-eval` pour ORT + `worker-src
  'self'` pour les web workers ORT internes.
- **SAB cross-origin isolation** : `SharedArrayBuffer` requis
  uniquement si `onnxruntime-web` threads activés. Phase B désactive
  threads par défaut (single-thread WASM, plus lent mais sans
  contrainte SAB → pas besoin de `Cross-Origin-Opener-Policy:
  same-origin` + `Cross-Origin-Embedder-Policy: require-corp`
  headers serveur). Follow-up tech debt si perf insuffisant : activer
  threads + ajuster headers.

### 5.4 Timeout + error propagation

`bridge.piiRedact()` hérite du timeout par défaut 10 s (constructor
`SBFBBridge`). Si `ensureLoaded()` prend > 10 s (network modèle),
l'iframe reçoit un error response `"bridge timeout after 10000ms"`.
L'app iframe peut retry avec `policy.use_model: false` pour forcer
le fallback regex (rapide, <50 ms).

Sur error model inference : fallback automatique (cf. §3.2). L'app
iframe reçoit un response success avec `findings_count` reflétant
les findings regex uniquement. Silent degradation volontaire
(éviter break les apps en prod si modèle fail — le layer 2
coord-side rattrape).

## 6. Tests Phase B

### 6.1 Vitest (`web/src/sdk/pii/__tests__/`) — 10+ tests

1. `fallback.test.ts::detects_email` — `user@domain.com` → 1 finding EMAIL_ADDRESS
2. `fallback.test.ts::detects_phone_e164` — `+33123456789` → 1 finding PHONE_NUMBER
3. `fallback.test.ts::detects_credit_card_with_luhn_filter` — valide Luhn accepté, invalid rejeté
4. `fallback.test.ts::detects_ssn_us_with_invalid_filter` — `123-45-6789` accepté, `000-12-3456` rejeté
5. `fallback.test.ts::detects_iban` — `FR7630001007941234567890185` finding IBAN
6. `fallback.test.ts::redact_applies_replacement_token` — `{ENTITY}` token remplacé
7. `policy.test.ts::default_policy_includes_standard_entities` — DEFAULT_POLICY check
8. `policy.test.ts::threshold_filters_low_confidence_findings` — confidence < threshold skip
9. `policy.test.ts::disabled_policy_returns_pass_through` — `enabled: false` = original text
10. `wrapper.test.ts::fallback_engaged_on_model_load_error` — inject load fail → fallback trigger
11. `wrapper.test.ts::respects_entity_whitelist_in_policy` — policy.entities filter findings

### 6.2 Extension `protocol.test.ts` + `useBridge.test.ts` — 2 tests

- `protocol.test.ts::accepts_pii_redact_method` — `BridgeMethodSchema` accepte `pii_redact`
- `useBridge.test.ts::dispatches_pii_redact_locally_without_coord_call` — mock `fetch` non-appelé

### 6.3 Playwright (`web/tests/bridge-pii-redact.spec.ts`) — 5 tests

1. `pii_redact_method_is_registered` — iframe call `bridge.piiRedact("hello")` → response success (no-op on clean text)
2. `pii_redact_redacts_email_fallback` — `"contact me at foo@bar.com"` → response contains `[REDACTED:EMAIL_ADDRESS]`
3. `pii_redact_disabled_policy_passes_through` — `{ enabled: false }` policy → text inchangé
4. `pii_redact_rejects_non_string_payload` — `bridge.piiRedact(42)` → error response
5. `pii_redact_correlation_id_preserved` — request `id` retourné dans response

**Total Phase B tests** : **16** (11 Vitest SDK + 2 Vitest bridge
+ 5 Playwright) — plan §5.4 projetait 10+5=15, delta +1 actuel
acceptable (léger up).

## 7. Scope cuts Phase B (PAS dans cette phase)

- **Download script modèle ONNX** : gitkeep placeholder livré, script bash optionnel reporté S+1 ops si besoin runtime prod
- **Threading ORT WASM** : single-thread par défaut, multi-thread follow-up si bench iframe insuffisant
- **Modèle fallback `gliner_multi_pii-v1` multilingue** : pas wired Phase B (option policy ouverte mais unused)
- **IBAN MOD 97 validation** : regex simple uniquement, false positive rate acceptable layer 1 (layer 2 Presidio coord Phase C fait validation stricte)
- **Policy hot-reload côté host shell** : DEFAULT_POLICY statique en Phase B, hot-reload runtime tech debt follow-up si besoin opérateur
- **Prometheus metrics PII findings** : pas de metric exposée Phase B, observability follow-up
- **Test perf bench cold-start iframe** : bench modèle ONNX download + load + inference latency — non-livré Phase B (requires modèle CI fixture, hors budget)

## 8. Risques résiduels (R-PII-1..R-PII-N)

- **R-PII-1** : modèle fallback regex a false positive rate
  élevé sur texte bruyant (ex: numéro version semver `1.2.3.4`
  détecté comme IBAN partial). **Mitigation** : policy
  `confidence_threshold` + Luhn check CC + whitelist TLD. Layer
  2 coord-side Phase C corrige residuels.
- **R-PII-2** : onnxruntime-web bundle ~5 MB → first load iframe
  lent (cold-start). **Mitigation** : lazy-load strict + chunk
  separation Vite auto + documenté fallback transparent.
- **R-PII-3** : modèle ONNX absent en CI → tests Playwright
  tournent fallback-only. Couverture model path testée via
  Vitest unit (mock injected). **Mitigation** : accepté,
  documenté `test-plan` + fail-fast row verification.
- **R-PII-4** : app malveillante bypass bridge, envoie PII directement
  via fetch coord. **Mitigation** : defense-in-depth layer 2 coord
  Phase C rattrape via `PiiRedactor` pre-dispatch worker.

## 9. Approbation

Design doc consommé par implementation Phase B suivante.
Pas de question ouverte bloquante. Phase B EXECUTE plan-as-is
(G8 préflight verdict `sprint21_phase_B_preflight.md`
2026-04-19).
