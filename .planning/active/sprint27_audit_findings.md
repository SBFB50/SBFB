# Sprint 27 — Audit findings (S28 Phase 0)

**Date** : 2026-04-25
**Auditeur** : session fraîche S28 Phase 0
**Tip audité** : `64e22c5` (S27 Phase E wrap-up)
**Audit plan** : `.planning/active/sprint28_audit_plan.md`
**Verdict** : **PASS** (0 P0, 0 P1, 5 P2, 1 P3)

---

## 1. Résumé

Sprint 27 a livré 4 phases A-D + Phase E wrap-up. Les composants
unitaires sont corrects et bien testés (+26 tests, 821 Rust / 1802
total). Le principal finding est que l'infrastructure watermark
(Track B) est correctement implémentée comme bibliothèque standalone
mais n'est pas câblée dans le sampling pipeline end-to-end. Pas de
risque fonctionnel (pre-launch), mais la verification s'auto-attribue
un niveau d'intégration supérieur au réel.

---

## 2. Track A — P2 batch S26 (Phase A `f8b8e2d`)

| ID | Check | Verdict |
|---|---|---|
| STAGE-1 | `validate_stage_guard_map` wired Dispatcher.__init__ + 2 test cases | **PASS** |
| EVENT-1 | `_emit_capability_event` except → logger.debug(exc_info=True), non-bloquant | **PASS** |
| DESC-1 | TaskHandlerDescriptor.description via fn.__doc__ + manifest endpoint | **PASS** |
| ROTATE-1 | JsonFileWriter rotation 10 MiB + .1→.5 shift + max 5 enforced | **PASS** |
| RENAME-1 | grep exhaustif EtwWriter = 0 résiduel dans tout le workspace | **PASS** |

**Findings Track A** : aucun.

---

## 3. Track B — Watermark SynthID (Phase B `7bb656b`)

| ID | Check | Verdict |
|---|---|---|
| PRF-1 | `prf_score` pure function, secret = paramètre instance, déterministe | **PASS** |
| ZTEST-1 | z-test binomial, test FP rate (non-watermarked < threshold) | **PASS** |
| INJECT-1 | `compute_bias` applique +delta green-only, disabled check OK | **P2** (cf. P2-B-1) |
| CONFIG-1 | Defaults corrects (enabled=false, delta=2.0, window=4) | **P2** (cf. P2-B-2) |
| RISK-1 | Pas de test intégration watermark + llguidance | **P2** (cf. P2-B-3) |

### P2-B-1 — Watermark injection non câblée dans le sampling pipeline

**Fichier** : `crates/nexus-worker-core/src/llm/watermark.rs` (119 LOC)
**Constat** : le module `watermark.rs` expose `prf_score`, `compute_bias`
et `should_inject` — fonctions correctes et unitairement testées (4
tests Rust). Mais **aucun call site** n'existe dans `llama_cpp.rs` :
grep `compute_bias|watermark::|use.*watermark` dans `llama_cpp.rs` = 0
match. Le module est déclaré (`pub mod watermark;` dans `mod.rs` L55)
mais jamais consommé par le sampling loop.

De plus, `runtime.rs:1062` fixe `output_token_ids: vec![]` — le champ
wire format n'est jamais peuplé côté worker, donc le detector
coordinator-side ne reçoit jamais de données pour le z-test.

La verification `sprint27_verification.md` row 14 affirme "Logit bias
+delta appliqué dans sampling pipeline [x]" citant le test
`test_watermark_logit_bias_applied`. Ce test **n'existe pas** sous ce
nom — le test réel est `compute_bias_applies_delta_to_green_only`
(watermark.rs:94) qui teste la **computation** du biais, pas
l'intégration dans le pipeline.

**Impact** : aucun impact fonctionnel (pre-launch). L'architecture PRF
+ config + wire format est en place. Le câblage effectif (call site
dans `llama_cpp.rs` + population de `output_token_ids` dans runtime.rs)
est un gap estimé à ~20-30 LOC. Carry S28.

### P2-B-2 — `configs/watermark.toml.sample` absent

**Constat** : le kickoff D1 spécifiait "Nouveau
`configs/watermark.toml.sample`". Le fichier n'existe pas (`ls configs/`
→ `pow_escalation.toml.sample`, `trust_web_seeds.toml`,
`worker.toml.sample`). La config watermark est une section `[watermark]`
dans `WorkerConfig` (config.rs:434-448), mais `worker.toml.sample` ne
la mentionne pas non plus.

**Impact** : gap de documentation utilisateur. Les options watermark ne
sont documentées que dans le code source Rust. Carry S28 (phase dette
ou Phase A batch).

### P2-B-3 — Pas de test intégration watermark + llguidance

**Constat** : le risk register R-S27-4 identifiait le risque de conflit
logit bias + llguidance. Le plan prévoyait un test d'intégration. Aucun
test combinant watermark + grammar n'existe.

**Impact** : risque documenté, non matérialisé (injection non câblée de
toute façon — cf. P2-B-1). Carry S29 per audit plan RISK-1.

---

## 4. Track C — Couche 3 multi-forge (Phase C `d52ce89`)

| ID | Check | Verdict |
|---|---|---|
| PARSER-1 | Format `--format=%aI\|%GK\|%G?\|%GS` OK git ≥ 2.34 + error handling | **PASS** |
| PARSER-2 | SigType enum Gpg/Ssh only, pas de catch-all, X.509 ignoré proprement | **PASS** |
| CACHE-1 | SQLite WAL (pragma L46), PK (repo_url, fingerprint) | **PASS** |
| TRUST-1 | Score multiplicatif, decay saturating_sub(1).max(1) | **PASS** |
| SEED-1 | Fingerprint FlowUP dummy `000...` padding | **P2** (cf. P2-C-1) |
| DELEG-1 | trust_level + valid_until + scope sans bump VERSION, JCS déterministe | **PASS** |

### P2-C-1 — Fingerprint bootstrap dummy dans trust_web_seeds.toml

**Fichier** : `configs/trust_web_seeds.toml:9`
**Constat** : le fingerprint FlowUP est
`80b439cb0000000000000000000000000000000000000000000000000000abcd` —
contient du zero-padding évident. L'audit plan SEED-1 demande
explicitement "pas de dummy `000...`". Ce n'est pas une clé Ed25519
réelle (ne peut pas signer de DelegationCert).

Les entrées ONG sont correctement commentées (pas de faux claims de
partenariat).

**Impact** : le trust-web ne peut pas être exercé end-to-end avec cette
clé. Fonctionnellement anodin (pre-launch placeholder explicitement
labellisé). Remplacer par la vraie clé Ed25519 FlowUP quand le
trust-web sera exercé (S28 outreach ONG).

---

## 5. Track D — Gate 3 docs (Phase D `814e485` + `4913f7f` + `6eee5ca`)

| ID | Check | Verdict |
|---|---|---|
| ROADMAP-1 | HARDENING_ROADMAP S27 "SynthID", last_validated 2026-04-25 | **PASS** |
| THREATS-1 | COMPUTE_THREATS §4.4 SynthID-inspired + BIRA + arXiv:2509.23019 | **PASS** |
| GATE3-1 | Gate 3 checklist 14 items livrés S22-S27 + 2 restants explicites | **PASS** |
| PATTERNS-1 | P37 watermark + P38 trust-web | **P2** (cf. P2-D-1) |
| SELFDIST-1 | SELF_DISTRIBUTION.md 8 sections spec complètes | **PASS** |

### P2-D-1 — P37 chemin injector incomplet

**Fichier** : `docs/rust/PATTERNS.md:2124`
**Constat** : P37 réfère l'injector à
`crates/nexus-worker-core/src/llm/llama_cpp.rs` (L2124). Le code
watermark réel est dans `crates/nexus-worker-core/src/llm/watermark.rs`
(119 LOC, PRF + compute_bias + should_inject). Le commit body Phase B
nomme correctement `watermark.rs`.

Historique : le commit `4913f7f` avait corrigé `llama_cpp.rs →
watermark.rs` (factuellement correct), mais `6eee5ca` l'a revert ("remaining
path corrections"). Les deux commits s'annulent (net diff = 0 entre
`814e485` et `6eee5ca`). Le premier fix était juste, le second l'a
défait.

**Impact** : doc technique incorrecte pour un développeur consultant
P37. P37 devrait mentionner les deux fichiers : `watermark.rs`
(computation) et `llama_cpp.rs` (futur call site d'intégration). Carry
S28 (correction doc).

---

## 6. Meta-track — G8 traceability

| ID | Check | Verdict |
|---|---|---|
| G8-FILES | 5/5 preflight (A-E) dans archive/v1.2/ | **PASS** |
| REVIEW-FILES | 4/4 review (A-D) + E absent (attendu, wrap-up) | **PASS** |
| VERDICT-CONSISTENCY | 5 EXECUTE → 5 commits phase, 0 DESIGN-CONFLICT → 0 pivot | **PASS** |
| PATH-CORRECTIONS | 4913f7f + 6eee5ca s'annulent (P3 process noise) | **P3** (cf. P3-META-1) |

### P3-META-1 — Commits path correction à net diff nul

**Constat** : `git diff 814e485..6eee5ca -- docs/` = vide. Les deux fix
commits ajoutent du bruit dans l'historique sans effet net. Le premier
(`4913f7f`) corrigeait `llama_cpp.rs → watermark.rs` (correct), le
second (`6eee5ca`) le revert. Pas d'impact fonctionnel — observation
process.

---

## 7. Pre-launch protocol check

| Check | Verdict |
|---|---|
| `*_VERSION = 1` partout (10+ constantes) | **PASS** |
| 0 tolerant decoder multi-version | **PASS** |
| 0 test "legacy decode" zombie | **PASS** |
| `#[serde(default)]` S27 avec rationale runtime documented | **PASS** |
| `DOMAIN_DELEGATION_CERT_V1` inchangé malgré 3 champs ajoutés | **PASS** |

---

## 8. Sprint pair S28 — observation phase dette

S28 est un sprint pair → phase dette obligatoire (§6.2.1 Règle 1).
Candidats identifiés par cet audit + carry :

| ID | Description | Source | LOC estimé |
|---|---|---|---|
| P2-B-1 | Câblage watermark dans sampling pipeline llama.cpp | S27 audit | ~30 LOC |
| P2-B-2 | watermark.toml.sample documentation | S27 audit | ~20 LOC |
| P2-C-1 | Fingerprint réel seeds.toml (FlowUP Ed25519 key) | S27 audit | ~5 LOC |
| P2-D-1 | P37 path correction watermark.rs | S27 audit | ~5 LOC |
| SC-9 | Platform writers complets (journald, oslog) | S27 kickoff §7.9 | ~200 LOC |
| SC-10 | ONNX end-to-end CI fixture | S27 kickoff §7.10 | ~100 LOC |

---

## 9. G4 calibration rigor

5 P2 + 1 P3 trouvés. Le seuil G4 (≥ 1 P2+ documenté) est satisfait.
Aucun finding fictif ou inflationary — chacun est tracé à un fichier
et une ligne spécifique.

---

## 10. Verdict

**PASS** — 0 P0, 0 P1, 5 P2, 1 P3.

Aucun fix bloquant requis. P2 documentés pour absorption S28 (phase
dette sprint pair + Phase A batch si applicable).
