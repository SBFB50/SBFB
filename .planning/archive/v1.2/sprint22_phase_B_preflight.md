# Sprint 22 Phase B — preflight G8

Date : 2026-04-19
HEAD : `88eee23`
Verdict : **SCOPE-CUT-CONSISTENT** (1 finding S1 non-bloquant, adaptation
inline pseudocode plan vs canonical GLiNER decoder — pas de carry S+1)

---

## Contexte

Phase B = GLiNER span-logits decoder iframe SDK (P2-S21-3 wire-up
debt carry). Scope plan §5 :

- `web/src/sdk/pii/wrapper.ts` (modifié lignes 82-108) : remplacer
  `return []` scaffold par appel décoder.
- `web/src/sdk/pii/decoder.ts` (nouveau ~250 LOC TS) : module pur
  `decodeSpans()` + `greedyDedup()` + `toFinding()`.
- `web/src/sdk/pii/__tests__/decoder.test.ts` (nouveau, +5 Vitest).
- `web/src/sdk/pii/__tests__/wrapper.test.ts` (modifié, +1 Vitest
  fixture integration).
- Delta projeté : **+6 Vitest** (256 → 262).

Aucun wire format Rust touché. Aucune dépendance ajoutée (libs déjà
chargées S21 Phase B `d5b0035`).

---

## Scans

### S1 — SOTA 2026 vs design

**Libs scannées** :

| Lib | Version | Source | Finding |
|---|---|---|---|
| `onnxruntime-web` | 1.24.3 | `web/package.json` | **clean** — aucune CVE 2026 |
| `@huggingface/transformers` | `^4.0.0` | `web/package.json` | **non-bloquant** : v5.0.0 breaks GLiNER (Issue urchade/GLiNER #324, uniform low scores via dtype change). Range caret `^4.0.0` = `>=4.0.0 <5.0.0` défensif par construction, v5 exclue par résolution semver. Rationale confirme P3-S21-2 carry audit trail. |
| `@huggingface/transformers.js` | déjà chargé S21 Phase B | `d5b0035` | **clean** — pas de regression Issue #826 résolu |
| GLiNER.js npm | référence algo (pas dep) | plan §5.1 | **clean** — release mars 2025 dernière, bus factor 1, algo référence pas dep runtime |

**Canonical decoder algo** (context7
`/websites/urchade_github_io_gliner`, query `span logits tensor
output format decoder sigmoid threshold greedy dedup algorithm`,
2026-04-19 fresh) :

```python
# Canonical GLiNER span decoder (urchade/GLiNER/decoding/decoder.py)
probs = torch.sigmoid(model_output)  # (B, L, K, C)
b_idx, s_idx, k_idx, c_idx = torch.where(probs > threshold)
valid = (s_idx + k_idx + 1) <= num_tokens[b_idx]
# ... vectorized extract scores + build Span tuples ...
spans = greedy_search(spans, flat_ner=True, multi_label=False)
```

**Output tensor shape = single `(B, L, K, C)`** (batch × length × max
span width × classes).

**Finding S1 non-bloquant** : Plan §5.2 pseudocode suppose 3-tensor
destructure `[start_logits, end_logits, span_logits] = outputs` —
c'est une imprécision plan vs canonical upstream (single tensor
`(B, L, K, C)`). L'implémentation Phase B adapte inline à la
signature ONNX runtime de `knowledgator/gliner-pii-edge-v1.0`
(vérifiée via `session.outputNames` au bootstrap du SDK). Plan §3
Research consulté mentionne explicitement "sigmoid + greedy dedup"
qui est conforme canonical → imprécision localisée §5.2 pseudocode
seulement, pas de design conflict.

**Classification** : non-bloquant (plan pseudocode détail, algo
target conforme upstream). Pas de carry S+1 requis, résolu
in-phase par adaptation signature.

**WebSearch CVE check** :
- `onnxruntime-web 1.24.3 CVE 2026 security advisory` : aucun
  résultat spécifique, clean.
- `GLiNER.js npm 2026 release breaking change decoder output
  format` : seule issue upstream = transformers v5.0.0 breaking
  (déjà mitigée par range caret `^4.0.0`).

**Verdict S1** : **1 finding non-bloquant** (pseudocode adaptation
inline).

### S2 — Décisions historiques traversées

**git log scan** :
```
git log --all --grep="DEVIATION\|rejected\|scope-cut\|deliberate\|threat-model" \
  --oneline -- web/src/sdk/pii/
→ d5b0035 feat(sprint21): Phase B — client-side PII redaction SDK iframe
  (seul commit touchant la zone, pas de décision rejetée)
```

**Archive scan** : `sprint21_audit_findings.md §4.3 P2-S21-3`
explicite carry pour S22 Phase B avec description précise du gap
scaffold → decoder. Phase B S22 = **continuation planifiée** du
scaffold, pas re-hash d'une décision rejetée.

**Memory feedback scan** : aucune règle "ne jamais faire X" touchant
PII decoder / span logits / ONNX output.

**Reverse-commit check** : non applicable (aucun finding S2 à
reverter).

**Verdict S2** : **clean**.

### S3 — Threat model coverage

**Threats mappés T0-T5** :

| Threat | Cover Phase B ? | Notes |
|---|---|---|
| **A4 User consent PII** (`THREAT_MODEL.md:62`) | partiel (defense-in-depth iframe side) | coord-side Phase C S21 déjà live (`23abb11`), iframe side = couche 1 défense |
| **C-ModelExtract** rate-limit per-consumer | non-scope Phase B | livré S21 Phase A, câblé S22 Phase A |
| **B-Sybil / B-Eclipse / B-GossipPoison** | non-scope Phase B | couverts Phase C S22 |

**HARDENING_ROADMAP §3 S22** : items 1-5 listés ; Phase B = résolution
wire-up debt P2-S21-3, pas nouvel item hardening. Pas de
pre-requirement non-livré.

**Regression flags** : aucune. Phase B renforce la primitive PII
existante (fallback regex → decoder GLiNER), pas de regression sur
threat déjà couvert.

**Verdict S3** : **clean**.

### S4 — Wire format / pre-launch invariants

**Files Phase B** : uniquement `web/src/sdk/pii/*` (pure TS iframe).

**Wire format Rust touché** : **aucun**.

**`_VERSION` fields** :
- `BLOB_VERSION = 0x01` : inchangé
- `TASK_RESPONSE_VERSION = 1` : inchangé
- `CANARY_VERSION = 1` : inchangé
- `ANNOUNCEMENT_VERSION = 1` : inchangé
- `PROVENANCE_VERSION = 1` : inchangé
- `CURATOR_LIST_VERSION = 1` : inchangé

**canonical.rs touché** : non.

**`#[serde(default)]` ajouté** : non (TS, pas Rust).

**Day 0 préservés** :
- D1 Sybil composition 3 couches : non touché Phase B.
- D2 scope γ hybride 6 phases : **Phase B = item B** (GLiNER
  span-logits decoder) conforme plan.
- D3 NVML baseline : non touché.
- D4 Watermark canari-input : non touché.
- D5 Cap G7 1/2 : non touché.

**Decisions actées `nexus_grid_pivot.md`** : non contredites.

**Pre-launch protocol policy** : respectée (aucun wire format
modifié).

**Verdict S4** : **clean**.

---

## Finding unique (SCOPE-CUT-CONSISTENT)

**S1-B-1** : Plan §5.2 pseudocode `const [start_logits, end_logits,
span_logits] = outputs` vs canonical GLiNER decoder single-tensor
output `(B, L, K, C)`.

- **Nature** : imprécision pseudocode plan, pas design conflict.
- **Action inline** : decoder Phase B adapte signature à
  `session.outputNames` runtime ONNX de `knowledgator/gliner-pii-
  edge-v1.0`. API decoder = `decodeSpans(modelOutput, tokens,
  threshold)` avec `modelOutput: Float32Array` + metadata shape
  lu du tensor ORT.
- **Carry S+1 requis** : **non**. Résolu in-phase par
  adaptation signature, aligné canonical upstream doc Phase B §3
  research.
- **Audit trail** : documenté dans le commit body Phase B
  (section "G8 preflight finding S1-B-1").

---

## Garde-fous G8 (§6.9 README)

- [x] **1. Evidence-based** : canonical decoder cité context7
  `/websites/urchade_github_io_gliner` (query 2026-04-19) ;
  Issue upstream urchade/GLiNER #324 transformers v5 break ;
  WebSearch CVE clean.
- [x] **2. Day 0 respect** : D2 Phase B GLiNER decoder confirmé,
  pas de pivot.
- [x] **3. Wire format** : zero `*_VERSION` touché.
- [x] **4. Test budget** : +6 Vitest projeté plan §5.3 = conforme
  (<< 2.5x).
- [x] **5. Theme sprint** : PII SDK defense-in-depth = theme S22
  §3 roadmap item P2 carry.
- [x] **6. Pas YAGNI** : decoder consommé immédiatement par
  wrapper.ts lignes 82-108.
- [x] **7. Retrospective trackée** : preflight.md + commit body
  → audit_plan S23 Track B (drift Playwright end-to-end déjà
  tracé plan §5.3 + §12).

---

## Action

**Procéder implémentation Phase B**. Adaptation pseudocode §5.2
inline documentée dans commit body section "G8 preflight finding
S1-B-1". Pas de carry S23 requis (résolu in-phase).

Fail-fast checklist Phase B (plan §5.4) :
- `cd web && npm run test:unit` vert ≥ 262 (+6 Vitest)
- `npm run lint` + `npx tsc --noEmit -p tsconfig.app.json` 0 erreur
- `npm run build` + `npm run size` 7/7 pass
- `bash scripts/scan-en-strings.sh` 0 unexpected EN
- `OnnxModelHandle.detect()` retourne spans non-vides sur fixture
