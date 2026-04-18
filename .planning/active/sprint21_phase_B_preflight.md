# Sprint 21 Phase B — preflight G8

Date : 2026-04-19
HEAD : `63afe4e` (feat(sprint21): Phase A — rate-limit sliding-window
multi-tier per-(consumer, worker, model) via governor GCRA
worker-engine gate R1)
Verdict : **EXECUTE plan-as-is**

Scope Phase B (rappel plan §5) : client-side PII redaction SDK iframe
— `onnxruntime-web 1.24.3` + `@huggingface/transformers` v4
tokenizer + `knowledgator/gliner-pii-edge-v1.0` (ModernBERT backbone
/ quint8 45.8 MB / opset compat ORT Web confirmé research 2026-04-18)
+ regex fallback curated + extension `sbfb-bridge.js` méthode whitelist
`pii_redact`.

---

## Scans

### S1 — SOTA 2026 vs design

Libs scannées via WebSearch + cross-check model card HF :

| Lib / artifact | Version plan | État 2026-04-19 | Delta |
|---|---|---|---|
| `onnxruntime-web` | `1.24.3` | npm latest 1.24.3, last publish ~1 mois (2026-03), patch series 1.24.x stable, no breaking documented depuis plan | **clean** |
| `@huggingface/transformers` npm | `4.x` | v4.0.0 GA (mars 2026), nouveau WebGPU Runtime C++, **CVE-2026-1839** affecte Python `transformers` (`Trainer._load_rng_state` torch.load sans weights_only) — **PAS applicable** npm package JS/WASM | **clean** |
| `knowledgator/gliner-pii-edge-v1.0` | stable | HF inchangé depuis 2024-01-29 publish + commit `de016fa4` README update (cosmétique). Backbone ModernBERT / quint8 45.8 MB / F1 75.5 confirmés sources primaires `gliner_config.json` 2026-04-18 | **clean** |
| `jhu-clsp/ettin-encoder-32m` (ModernBERT upstream) | MIT, 32M params | HF présent, arXiv 2507.11412 (paper 15 juillet 2025) public | **clean** |

WebSearch CVE :
- `CVE-2026-1839` HuggingFace Transformers RCE : **Python uniquement**
  (`src/transformers/trainer.py:3059`), fixed v5.0.0rc3. **Pas
  applicable** au npm package `@huggingface/transformers`.
- RUSTSEC / npm-advisory `onnxruntime-web` 2026 : aucun résultat
  pertinent (best-effort search).

Notes supplémentaires :
- HF discussion `#3 KeyError: 'modernbert'` (model card edge) =
  Python `transformers < 4.48` ne reconnait pas `model_type:
  modernbert`. **Pas applicable** à notre use case : on charge
  le fichier ONNX directement via `onnxruntime-web` (pas le
  loader `transformers` Python). `@huggingface/transformers`
  npm est utilisé uniquement pour le **tokenizer BPE**, pas le
  runtime transformer (issue upstream npm `#826` GLiNER loader
  natif non supporté — plan §D2 l'a anticipé en contournant
  via onnxruntime-web direct).

Verdict : **S1: clean**.

### S2 — Decisions historiques traversées

Fichiers Phase B ciblés (plan §5.3) :
- `web/src/sdk/pii/` (nouveau dir)
- `web/src/lib/sbfb-bridge.js` (modifié — ajout 4e méthode
  whitelist `pii_redact`)
- `web/package.json` (modifié — ajout deps)

Commandes :

```bash
git log --all --grep="DEVIATION\|rejected\|scope-cut\|threat-model" \
  --oneline -- web/src/sdk/ web/src/lib/sbfb-bridge.js web/package.json
# → 0 hit

grep -rE "DEVIATION deliberee|rejected for|scope-cut at|threat-model|\
PII|iframe.*SDK|Rust-first|tract" .planning/archive/v*/sprint*_*.md
# → 0 hit pertinent PII/iframe/Rust-first rejet retourné

grep -rE "do not|never|reject|avoid" memory/feedback_*.md
# → pas de rule PII iframe évitée
```

Analyse findings potentiels :
- Le rejet « Full Rust-first iframe (tract + GLiNER + wasm-
  bindgen) » documenté dans `sprint21_kickoff.md §D2` + research
  `S21_research_ort_wasm_alternatives.md` + `S21_research_rust_
  first_alignment.md` = **rationale du design RETENU** (Option 7
  defense-in-depth custom JS iframe), pas un rejet de ce qu'on
  implémente. Tech debt T-NN+2 explicit carry S22+.
- Aucun commit archive rejette l'approche onnxruntime-web +
  GLiNER + bridge postMessage pour PII redaction.
- Pattern `sbfb-bridge.js` établi S13 (3 méthodes whitelist
  `task_submit` / `storage_get` / `storage_set`) — extension à
  4 méthodes `pii_redact` = suit le pattern, pas une déviation.

Reverse-commit check : N/A (0 finding à classifier).

Verdict : **S2: clean**.

### S3 — Threat model coverage

Commandes :

```bash
grep -B 2 -A 10 "Client-side redaction\|PII SDK\|§3 S21" \
  docs/security/HARDENING_ROADMAP.md
ls .planning/active/sprint21_phase_A_review.md  # existing
```

Threats T0-T5 mapping (cf. `docs/security/ADVERSARIES.md` +
`THREAT_MODEL.md`) :

| Threat | Couverture Phase B | Status |
|---|---|---|
| **T4 — Model extraction / PII harvest** (coord/worker malveillant extrait PII du prompt client) | **Couvert layer 1** (iframe pre-redact avant postMessage → coord → worker). Layer 2 Presidio coord-side Phase C = defense-in-depth | ✅ primitive primaire Phase B |
| **T2 — PII leak passive** (utilisateur accidentellement pousse données sensibles) | **Couvert primitive GLiNER** (détection NER + 10 entités standard email/phone/CC/SSN/IBAN/...). Regex fallback si model load fail | ✅ couvert |
| **T1 — Network observer** (relayer observe prompts) | Non-regression : iframe redact AVANT iroh relay, relayer voit redacted-only | ✅ non-regression |
| **T0 — Passive attacker** | Non-regression | ✅ |
| **T3 — Active attacker** | Hors scope Phase B (runtime isolation = S17+ roadmap) | ➖ hors scope |
| **T5 — Nation-state / TEE escape** | Hors scope (S25+ Arti Tor + TEE roadmap) | ➖ hors scope |

HARDENING_ROADMAP §3 S21 ligne pertinente (frontmatter
`audited_findings 2026-04-18 S21 open`) :

> « Stack retenue defense-in-depth : client iframe = onnxruntime-web
> 1.24.3 (Microsoft, npm mars 2026) + @huggingface/transformers v4
> tokenizer + knowledgator/gliner-pii-edge-v1.0 ... »

Phase B livre **exactement** cette ligne. Requalification déjà
actée 2026-04-18. Pas de pré-requirement HARDENING manquant.

Regression flags : aucun. Phase A (rate-limit worker-engine gate)
ne partage pas de surface avec Phase B (client-side iframe JS) →
pas de cross-phase regression.

Verdict : **S3: clean**.

### S4 — Wire format / pre-launch invariants

Commandes :

```bash
grep -rE "_VERSION\s*[:=]\s*[0-9]+" crates/nexus-core-rs/src/
# → "crates/nexus-core-rs/src/schemas/mod.rs: *_VERSION = 1 pre-launch
#    protocol policy" (commentaire seul — aucun changement Phase B)
```

Invariants vérifiés :

| Invariant | Status Phase B |
|---|---|
| `BLOB_VERSION = 0x01` | ✅ inchangé |
| `TASK_RESPONSE_VERSION = 1` | ✅ inchangé |
| `CANARY_VERSION = 1` | ✅ inchangé |
| `ANNOUNCEMENT_VERSION = 1` | ✅ inchangé |
| Pas de tolerant decoder multi-version introduit | ✅ |
| `#[serde(default)]` ajoutés legitimes | ✅ N/A (Phase B = TypeScript/JavaScript, pas de Rust serde) |
| DOMAIN_* signatures préservées | ✅ inchangées (Phase B ne touche pas crypto sign) |
| D1..D5 Day 0 non rebattus | ✅ D2 implémentée telle que figée (kickoff §D2 + research backbone resolution 2026-04-18) |
| Decisions `nexus_grid_pivot.md` non contredites | ✅ (iframe host + bridge postMessage 3→4 méthodes whitelist = extension pattern S13, pas rupture architecture) |

Détails extension bridge :
- `sbfb-bridge.js` S13 ouvre 3 méthodes whitelist (`task_submit`,
  `storage_get`, `storage_set`). Phase B ajoute une 4e entrée
  `pii_redact(text, policy)` + correlation ID pattern S13
  préservé. **Pas** une nouvelle version de protocole bridge —
  c'est une extension additive explicite couverte par le
  kickoff §D2 §Implications code.

Verdict : **S4: clean**.

---

## Synthèse

| Scan | Verdict |
|---|---|
| S1 SOTA | clean |
| S2 Historical | clean |
| S3 Threat model | clean |
| S4 Wire invariants | clean |

**Règle d'agrégation** : 0 finding bloquant + 0 finding non-
bloquant → **EXECUTE plan-as-is**.

## Action

Procède Phase B code implementation selon plan §5 :

1. **Aucun pivot** nécessaire. Day 0 D2 figée respectée.
2. **Aucun carry-over S22** ajouté depuis ce préflight.
3. Design doc pre-Phase-B normal `.planning/research/S21_phase_B_
   iframe_pii_sdk_design.md` (plan §5.2) reste à créer dans le
   premier working-tree-audit de la phase si pas déjà présent.
4. Commit phase suivra template plan §5.6 avec body riche
   (delta +10 Vitest +5 Playwright + scope cuts + working tree
   audit G5 + référence ce préflight).

Ce document archivé Phase F dans `archive/v1.2/` avec les autres
artefacts S21.
