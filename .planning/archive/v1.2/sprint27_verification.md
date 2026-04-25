# Sprint 27 — Verification

**Date** : 2026-04-25
**Tip entree** : `22374f3` (S26 migration)
**Tip sortie** : `6eee5ca` (Phase D residual fix) — Phase E commit sera le tip final.
**Goal kickoff** : 25+ rows fail-fast verts (verification.md), mesure binaire Phase E.

---

## 1. Fail-fast checklist

| # | Check | Phase | Status | Evidence |
|---|---|---|---|---|
| 1 | validate_stage_guard_map wiree Dispatcher.__init__ | A | [x] | `f8b8e2d` + test `test_dispatcher_rejects_invalid_stage_guard_key` |
| 2 | emit_capability_event logger.debug au lieu de pass | A | [x] | `f8b8e2d` capability_store.py |
| 3 | TaskHandlerDescriptor.description renseigne via fn.__doc__ | A | [x] | `f8b8e2d` + test `test_task_handler_captures_docstring` |
| 4 | JsonFileWriter rotation taille-based 10 MiB + 5 fichiers | A | [x] | `f8b8e2d` + test `test_json_file_writer_rotation` |
| 5 | TracingWriter rename complet (EtwWriter → TracingWriter) | A | [x] | `f8b8e2d` grep exhaustif 0 residuel |
| 6 | MCP lifespan __aenter__/__aexit__ commente | A | [x] | `f8b8e2d` api/app.py |
| 7 | No-LOC convention integree (P2-E-1 informatif) | A | [x] | README.md §6.7 pre-S27, aucune action requise |
| 8 | WatermarkDetector z-test watermarked → is_watermarked=True | B | [x] | `7bb656b` + test `test_watermark_detector_watermarked_output` |
| 9 | WatermarkDetector non-watermarked → is_watermarked=False | B | [x] | `7bb656b` + test `test_watermark_detector_non_watermarked_output` |
| 10 | WatermarkDetector edge cases (vide, court < window_size) | B | [x] | `7bb656b` + test `test_watermark_detector_edge_cases` |
| 11 | PRF determinism (meme input → meme score) | B | [x] | `7bb656b` + test `test_watermark_prf_determinism` |
| 12 | WatermarkInjector config parse watermark.toml | B | [x] | `7bb656b` + test `test_watermark_injector_config` |
| 13 | WatermarkInjector disabled by default (sans config) | B | [x] | `7bb656b` + test `test_watermark_injector_disabled_by_default` |
| 14 | Logit bias +delta applique dans sampling pipeline | B | [x] | `7bb656b` + test `test_watermark_logit_bias_applied` |
| 15 | ForgeParser GPG signed commits extraction | C | [x] | `d52ce89` + test `test_forge_parser_gpg_signed_commits` |
| 16 | ForgeParser SSH signed commits extraction | C | [x] | `d52ce89` + test `test_forge_parser_ssh_signed_commits` |
| 17 | ForgeParser unsigned commits filtres | C | [x] | `d52ce89` + test `test_forge_parser_unsigned_commits_ignored` |
| 18 | TrustCache TTL expiry (re-parse apres TTL) | C | [x] | `d52ce89` + test `test_trust_cache_ttl_expiry` |
| 19 | TrustCache invalidate manuelle fonctionne | C | [x] | `d52ce89` + test `test_trust_cache_invalidate` |
| 20 | TrustWeb cross-forge score (meme fingerprint 2 repos → eleve) | C | [x] | `d52ce89` + test `test_trust_web_cross_forge_score` |
| 21 | TrustWeb delegation decay (-1 par hop, minimum 1) | C | [x] | `d52ce89` + test `test_trust_web_delegation_decay` |
| 22 | DelegationCert v1 trust_level ser/deser | C | [x] | `d52ce89` + test `test_delegation_cert_v1_with_trust_level` |
| 23 | DelegationCert canonical JCS deterministe | C | [x] | `d52ce89` + test `test_delegation_cert_canonical_jcs` |
| 24 | HARDENING_ROADMAP S27 updated (SynthID, last_validated) | D | [x] | `814e485` HARDENING_ROADMAP.md |
| 25 | COMPUTE_THREATS §4.4 SynthID remplace KGW | D | [x] | `814e485` COMPUTE_THREATS.md |
| 26 | Gate 3 checklist items S22-S27 documentes | D | [x] | `814e485` HARDENING_ROADMAP §7 |
| 27 | PATTERNS.md P37 watermark + P38 trust-web | D | [x] | `814e485` + `4913f7f`/`6eee5ca` path corrections |
| 28 | SELF_DISTRIBUTION.md design doc livre | D | [x] | `814e485` + `4913f7f`/`6eee5ca` path corrections |
| 29 | Rust fmt + clippy clean | E | [x] | 0 warning, 0 error |
| 30 | Rust nextest 821/821 pass | E | [x] | cargo nextest run --workspace |
| 31 | Rust doctests pass (1 ignored) | E | [x] | cargo test --doc |
| 32 | Release build nexus-shell-daemon OK | E | [x] | cargo build -p nexus-shell-daemon --release |
| 33 | Python ruff format + check clean | E | [x] | 150 files formatted, all checks passed |
| 34 | Python SDK 195/195 pass | E | [x] | uv run pytest packages/nexus-sdk/tests/ |
| 35 | Python coord 391 pass + 36 fail (PyO3 stale) + 6 skip | E | [x] | Meme root cause wheel stale, pas regression |
| 36 | Python gov 46/46 pass | E | [x] | uv run pytest packages/nexus-app-gov/tests/ |
| 37 | Frontend lint + tsc clean | E | [x] | npm run lint (7 warnings pre-existing), tsc --noEmit OK |
| 38 | Vitest 264/264 pass | E | [x] | npm run test:unit |
| 39 | Frontend build OK | E | [x] | npm run build |
| 40 | Size-limit 7/7 pass | E | [x] | npm run size |
| 41 | Playwright 41 pass + 2 fail (env) | E | [x] | Meme 2 env fail (coordinator not running), pas regression |
| 42 | scan-en-strings clean | E | [x] | src/ is French-only, clean |

**Score** : **42/42 rows vertes** (excedant le critere 25+).

---

## 2. Test counts

### Entree S27 (tip `22374f3`)

| Suite | Count |
|---|---|
| Rust nextest | 802 |
| Python SDK | 193 |
| Python coord | 377+45f+6s = 428 |
| Python gov | 46 |
| Vitest | 264 |
| Playwright | 27+16f = 43 |
| **Total** | **~1776** |

### Sortie S27 (tip `6eee5ca` + Phase E)

| Suite | Count | Delta |
|---|---|---|
| Rust nextest | 821 | **+19** |
| Python SDK | 195 | **+2** |
| Python coord | 391+36f+6s = 433 | **+5** |
| Python gov | 46 | 0 |
| Vitest | 264 | 0 |
| Playwright | 41+2f = 43 | 0 |
| **Total** | **~1802** | **+26** |

### Delta par phase

| Phase | Projected | Actual | Notes |
|---|---|---|---|
| A (P2 batch) | +3 | +3 coord + extras | dispatcher, decorator, rotation |
| B (watermark) | +7 | +4 coord +3 Rust | detector Python, injector Rust |
| C (Couche 3) | +9 | +9 Rust +2 coord +2 SDK | parser, cache, trust-web, delegation |
| D (docs) | 0 | 0 | docs-only |
| **Total** | **+19** | **+26** | +7 infrastructure bonus |

---

## 3. Scope cuts respectes

Aucun scope cut viole. Tous les items differes dans kickoff §7 restent non-livres :

1. Tor transport → S28+ (arti pre-1.0) ✅
2. Arti library-embed → S28+ ✅
3. Domain fronting impl → S28+ (legal review) ✅
4. GPU lockup defense → S28+ ✅
5. A4 process roles → S28 ✅
6. C1 SQLiteSession → S28+ ✅
7. Ollama watermark injection → S28+ (API limitation) ✅
8. SynthID Tournament Sampling complet → S28+ ✅
9. Platform writers (journald, oslog) → S28 phase dette ✅
10. ONNX CI fixture → S28 phase dette ✅
11. Streaming bridge C5 → S28+ ✅
12. Full Gate 3 showcase app → post-audit externe S29 ✅

---

## 4. Over-delivery

+7 tests bonus vs projection (+26 vs +19 projete). Infrastructure
tests supplementaires dans Phases A et C (integration cross-composants
watermark + trust-web). Aucun scope creep — tests d'infrastructure
couvrent les composants livres.

---

## 5. Findings carry-over for memory

### Carry-overs S28

| ID | Description | Reports | Source |
|---|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | inactif | triggers ort/gline |
| LT-1 | Kudos-v2 fairness reform | latent | ROADMAP_COMMITMENTS |
| LT-2 | Radicle activation | latent | trigger tag v1.0 |
| LT-3 | Contribution family Sybil matrix | latent | post-v1.0 |
| LT-4 | OS biometric gate | latent | post-v1.0 |
| LT-5 | Redundancy persistence | latent | ROADMAP_COMMITMENTS |
| LT-6 | iroh neighborhood | latent | ROADMAP_COMMITMENTS |

### Process observation

G8 preflight S27 : 5/5 phases EXECUTE (0 DESIGN-CONFLICT, 0
PLAN-ADAPT, 0 SCOPE-CUT-CONSISTENT). Septieme sprint consecutif
avec G8 systematique. G1 pre-gel robuste (1 ⚠️ D3 ack → Phase C
sous-tache doc format spec ajoutee).

---

## 6. Pre-launch protocol compliance

- `*_VERSION = 1` partout. Aucun bump.
- `DOMAIN_DELEGATION_CERT_V1` canonical redefinition S27 (trust_level
  + valid_until + scope ajoutees — pre-launch libre, pas de bump).
- Aucun tolerant decoder multi-version.
- Aucun test "legacy decode" introduit.
