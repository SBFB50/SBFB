# Sprint 21 — Audit findings (session fraîche Sprint 22 Phase 0)

**Auditeur** : session fraîche 2026-04-19 (pattern permanent depuis
Sprint 7, cf. `docs/claude/README.md §3`).
**Timebox effective** : ~1 h 15 (lecture ciblée Cas A + investigation
indépendante Track-par-Track + vérification tests en parallèle).
**Range audité** : `b34d451..7887471` (18 commits Sprint 21 + chores
tooling G4/G8 inclus).
**Tip audité** : `7887471` (chore(sprint21): Phase F — wrap-up +
verification + audit plan S22 + migrate planning).

---

## 1. Verdict global : **PASS**

0 P0 + 0 P1. **6 P2 documentés** (dont 3 carry-overs déjà tracés
dans `sprint21_audit_plan.md` + 3 nouveaux que ce rapport ajoute
au carry S22). **5 P3 cosmétiques**. **3 meta-tracks validés**.

**Rigor signal G4** : **SATISFAIT**. ≥ 1 P2+ documenté (règle
calibration). L'audit a exploré les 6 tracks A-F du plan + les
3 meta-tracks (Radicle-v1.0, G8 traceability, hook coverage
Phase D) + une dimension **review-to-audit-plan carry gap**
découverte indépendamment (lecture rétroactive des phase review
files vs contenu de `sprint21_audit_plan.md`).

**Sprint 22 Phase A non bloqué.** Aucun fix commit `fix(sprint21):
...` requis pré-S22. Ce document `sprint21_audit_findings.md`
sert de véhicule de carry consolidé vers le kickoff S22 (pattern
§3.5 README).

---

## 2. Contexte

### 2.1 Range audit (18 commits Sprint 21)

```
7887471 chore(sprint21): Phase F — wrap-up + verification + audit plan S22 + migrate planning   ← tip audit
49f0d32 feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix + PATTERNS §P34)
f830579 feat(sprint21): Phase D — quarantine queue SQLite WAL + manual flush CLI
a82e8db chore(planning): sprint21 §7 Phase D realignement coord-Python + design doc
17035c3 chore(docs): fairness vision + LT-1 Kudos-v2 commitment + S22 roadmap flag
23abb11 feat(sprint21): Phase C — coord-side PII redaction + output filter (Presidio GLiNER + local InvisibleText + EED echo)
041d8d0 chore(planning): sprint21 §6.2 pivot D3 — drop llm-guard (transitive-pin presidio-analyzer conflict) + local InvisibleText scanner
624ad7e chore(planning): sprint21 §6.2 naming-fix — dispatcher + validator hooks (drop inexistant task_response_validator.py)
d5b0035 feat(sprint21): Phase B — client-side PII redaction SDK iframe (onnxruntime-web + GLiNER PII edge)
63afe4e feat(sprint21): Phase A — rate-limit sliding-window multi-tier per-(consumer, worker, model) via governor GCRA worker-engine gate R1
57c829d chore(hook): phase-auditor-gate.sh — fix false-positive exit 2 on missing archive review
b4bda81 chore(planning): sprint21 Phase A R1 scope-cut — worker-engine gate pure Rust (drop HTTP middleware drift)
5e67ce0 chore: bump axum 0.7 → 0.8 workspace-wide
60adceb chore(planning): sprint21 Phase A — G8 pivot proposal + arbitrage Option C (axum 0.7 → 0.8 bump prereq)
f5ad2e1 chore(research): sprint21 — archive 4 research outputs (§6.11 pattern rétroactif)
8ba1d7c chore(planning): sprint21 kickoff §D2 — backbone ModernBERT confirmed + ort-wasm post-G1 re-check + multi-agent team fix acknowledgments
71de0ec chore(workflow): docs/claude/README.md — §6.1.1 G1 ext custom Rust stack + §6.2.1 carry long-term + §6.10 G9 factual-research + §6.11 archive research outputs
7e34fe6 chore(agents): nexus-phase-auditor — eliminate forced rigor signal + anti-hallucination code-reread + archive-immediately note
b34d451 chore(planning): open Sprint 21 — rate-limit + PII SDK defense-in-depth + output filter + quarantine queue
```

5 commits `feat` (phases A/B/C/D/E) + Phase F wrap-up chore +
9 chores (planning/workflow/skill/hook) + 1 chore axum bump
workspace-wide + 1 chore research archivage rétroactif.

### 2.2 Compteurs tests vérifiés (tip `7887471`)

| Suite | verification.md | Observé audit | Delta vs S20 | Match |
|---|---|---|---|---|
| Rust workspace nextest | 659 | **659 pass, 0 skip** (re-run auditeur 8.4 s) | +17 vs 642 | ✓ |
| Python coordinator | 249 + 3 skip | **249 pass + 3 skip** (re-run auditeur 157 s) | +36 vs 213+3 | ✓ |
| Python SDK | 185 | 185 (pattern-match, non re-run) | +0 | ~ |
| Python app-gov | 46 | 46 (pattern-match, non re-run) | +0 | ~ |
| Vitest unit | 256 | 256 (pattern-match, non re-run) | +15 vs 241 | ~ |
| Playwright | 38 | 38 (pattern-match, non re-run) | +0 vs 38 | ~ |
| size-limit | 7/7 | 7/7 (phase F doc-only, inchangé) | — | ~ |

**Total** : **~1436 tests** (+65 vs baseline 1371 S20 final).
Re-run auditeur confirme les 2 suites les plus lourdes (Rust +
coord-Python) — les 4 autres suites font confiance au self-report
Phase F (non-touchées par Phase F doc-only).

### 2.3 Pre-launch protocol policy — PASS

```bash
$ grep -rE '_VERSION\s*[:=]\s*[0-9]+' crates/nexus-core-rs/src/schemas/ \
    crates/nexus-shell-daemon-core/src/canary/
BLOB_VERSION = 0x01
TASK_RESPONSE_VERSION = 1
CANARY_VERSION = 1
ANNOUNCEMENT_VERSION = 1
```

Aucun `_VERSION` bumpé pendant S21. Aucun tolerant decoder
multi-version introduit. Le wire canary migré `serde_jcs` (Phase E
T-NN) préserve la signature (path signing via
`canonical_bytes(&signed, DOMAIN_WARRANT_CANARY_V1)` inchangé).
Pre-launch policy CLAUDE.md respectée.

### 2.4 G8 traceability — 5/5 phases A-E + F trivial

| Phase | G8 verdict | Document présent ? |
|---|---|---|
| A | DESIGN-CONFLICT → Option C arbitré user | `sprint21_phase_A_pivot_proposal.md` ✓ |
| B | SCOPE-CUT-CONSISTENT | `sprint21_phase_B_preflight.md` ✓ |
| C | SCOPE-CUT-CONSISTENT | `sprint21_phase_C_preflight.md` ✓ |
| D | SCOPE-CUT-CONSISTENT | `sprint21_phase_D_preflight.md` ✓ |
| E | SCOPE-CUT-CONSISTENT | `sprint21_phase_E_preflight.md` ✓ |
| F | EXECUTE (trivial) | `sprint21_phase_F_preflight.md` ✓ |

**Coverage 100%.** Premier sprint avec G8 systématique
(instauré S20 Phase E via commit `59225ee`, codifié tool
`nexus-phase-preflight` skill). Le pivot G8 Phase A → arbitrage
user Option C (axum bump) a été exécuté proprement : proposal
émise, user arbitré, plan mis à jour via `b4bda81` + chore
workspace bump `5e67ce0` AVANT code.

### 2.5 Migration PARA Phase F — PASS

- `.planning/active/` **vide** au tip `7887471` (avant écriture
  du présent document). ✓
- Tous les 17 fichiers Sprint 21 attendus présents dans
  `archive/v1.2/` **sauf** `sprint21_phase_D_review.md`
  volontairement absent (cf. §4.8 finding P2-S21-5 hook coverage
  Phase D). ✓ accepté sous réserve de la remédiation carry.

---

## 3. Track-by-track

### 3.1 Track A — Phase A rate-limit governor (PASS avec 2 P2 + 1 P3 carries)

Commit `63afe4e feat(sprint21): Phase A — rate-limit sliding-
window multi-tier per-(consumer, worker, model) via governor GCRA
worker-engine gate R1`. Précédé de `60adceb` (G8 pivot) +
`5e67ce0` (axum bump) + `b4bda81` (R1 scope-cut).

**A-1 G8 pivot Option C retrospective** : `git log --stat 5e67ce0~1..5e67ce0`
montre 4 fichiers touchés par le bump workspace (`Cargo.toml` +
`Cargo.lock` + `browse.rs` call-site + `http.rs` call-site).
`git log --oneline 5e67ce0..HEAD -- crates/ Cargo.toml` ne liste
que les 2 feat Phase A/E et **aucun fix-up tardif lié au bump**.
Arbitrage Option C effectif sans régression. ✓

**A-2 R1 scope-cut HTTP middleware** : documenté dans chore
`b4bda81`, rationale « `/task/submit` vit côté FastAPI Python
Sprint 4 Phase A, `tower-governor` axum ne middleware pas
FastAPI ». Scope-cut rétroactivement cohérent avec architecture
actuelle — test pas de drift. ✓ Scope-cut explicitement carry
S22+ sprint API sécurité Python dédié (pattern `slowapi` ou
équivalent).

**A-3 PoW bypass check sur path engine::accept** : grep
`RateLimiter::check | rate_limit.*check | engine::accept` ne
retourne **aucune** call-site production. L'engine runtime
(`crates/nexus-worker-core/src/engine/runtime.rs`) n'appelle
JAMAIS `RateLimiter::check()`. La primitive existe mais **n'est
pas consommée**. Lignes 782 + 795 montrent uniquement des appels
à `consent::should_accept_task` — aucun gate rate-limit en
amont.

**Finding principal** : cf. **P2-S21-1** ci-dessous. Le Phase A
review `sprint21_phase_A_review.md` classait P2-A-1, P2-A-2,
P3-A-1 avec promesse explicite « À tracer dans
`sprint21_audit_plan.md §Track-X` Phase F + re-carry S22 ».
**Phase F wrap-up n'a PAS carrié** ces 3 findings dans le
`audit_plan.md` Track A qui liste 3 sous-items **différents**
(G8 retrospective, R1 scope-cut doc, PoW bypass check) sans
mentionner engine-wire-up gap ni hot-reload incomplete ni
burst_multiplier silent floor.

Ce rapport `audit_findings.md` consolide la carry (cf.
findings §4 ci-dessous).

### 3.2 Track B — Phase B PII SDK iframe (PASS avec 1 P2 + 1 P3 carries)

Commit `d5b0035 feat(sprint21): Phase B — client-side PII redaction
SDK iframe (onnxruntime-web + GLiNER PII edge)`.

**B-1 G8 S1 backbone GLiNER résolution** : preflight
`sprint21_phase_B_preflight.md` documente S1 scan « backbone
ModernBERT confirmé + ONNX quint8 45.8 MB + opset compat ORT Web
1.24.3 ». Inline code `web/src/sdk/pii/wrapper.ts:45` montre
`@huggingface/transformers` import pour tokenizer v4. Verdict
preflight EXECUTE plan-as-is. ✓

**B-2 size-limit 7/7** : verification.md row 15 confirme 7/7
pass. `onnxruntime-web` + `@huggingface/transformers` dynamic
`import()` (wrapper.ts:45) produisent des chunks séparés
lazy-loaded, hors budget main bundle. ✓

**B-3 CSP `connect-src 'none'` iframe respectée** : le SDK PII
vit côté **host shell** (`web/src/sdk/pii/`), pas dans l'iframe
user. CSP iframe non-étendue. Le modèle ONNX bundled dans
`web/public/models/` (host-side, pas iframe). Pattern §P23
iframe CSP préservé. ✓

**Finding scaffold gap** : cf. **P2-S21-3** ci-dessous.
`OnnxModelHandle.detect()` ligne 107 de wrapper.ts retourne
systématiquement `[]` après avoir exécuté l'inférence ONNX
(tokenize + session.run). Le post-processing span-logits →
findings **n'est pas implémenté**. La détection PII iframe
dépend EXCLUSIVEMENT du fallback regex (5 entités : email,
phone, credit card, SSN, IBAN). Phase C coord-side couvre
defense-en-profondeur. Phase B review pré-commit a classé P2
(documentation-of-gap, pas bug) mais **non carrié** au
`audit_plan.md`.

### 3.3 Track C — Phase C coord PII + output filter (PASS clean)

Commit `23abb11 feat(sprint21): Phase C — coord-side PII
redaction + output filter (Presidio GLiNER + local InvisibleText
+ EED echo)`. Précédé de `041d8d0` (pivot D3 drop llm-guard) +
`624ad7e` (naming fix dispatcher).

**C-1 InvisibleText scanner coverage** : `output_filter.py:61-109`
(grep `PUA|U+E000|zero-width|InvisibleText`) reproduit fidèlement
llm-guard 0.3.16 `InvisibleText` scanner : zero-width U+200B
(line 81), PUA U+E000-U+F8FF + U+F0000-U+FFFFD + U+100000-U+10FFFD
(line 109), Tag chars, BOM U+FEFF. Fallback pivot D3 cohérent
et code auditable ligne par ligne vs dép opaque. ✓

**C-2 EED seuil 0.85 configurable** : `output_filter.py:56`
`DEFAULT_EED_THRESHOLD = 0.85`. `output_filter_policy.toml.sample:
71` expose `eed_threshold = 0.85` au prompt_echo section + ligne
43 documente « Seuil empirique tuné sur le... ». Hot-reload via
file watcher (pattern S20 PoW policy loader). ✓

**C-3 Presidio [gliner] extra + même modèle ONNX** :
`pii_redactor.py:48` `DEFAULT_MODEL_NAME = "knowledgator/gliner-
pii-edge-v1.0"` — **exactement le même modèle** que
`web/src/sdk/pii/wrapper.ts` DEFAULT_MODEL_URL (host-shell).
Single source of truth ONNX respecté. `pyproject.toml` extra
`[gliner]` via `presidio-analyzer[gliner]>=2.2.362`. ✓

Aucun carry Track C.

### 3.4 Track D — Phase D quarantine queue (PASS avec 1 carry scope hors-scope documenté)

Commit `f830579 feat(sprint21): Phase D — quarantine queue SQLite
WAL + manual flush CLI`. Précédé de `a82e8db` (réalignement
coord-Python G8 SCOPE-CUT-CONSISTENT).

**D-1 Paths réalignés** : verification.md rows 25+26+28+29
confirment tous les paths plan §7.2 réaligné existent (`pii_
redactor.py`, `output_filter.py`, `quarantine_queue.py`,
CLI `nexus-coordinator quarantine --help`). ✓

**D-2 Test cardinality drift 1k vs 10k** :
`test_quarantine_queue.py:166` `test_cardinality_1k_entries_
sweeps_clean` (réduit vs plan §7.3 row 5 `10k`). Docstring
ligne 173 documente « 60s default timeout. A dedicated 10k
stress harness is [deferred] ». Design doc §6.1 documente
la décision rétroactivement. Drift acceptable retrospective.
✓ **Carry S22 ou sprint ops futur** : bench harness 10k
dédié si utile pour scale production.

**D-3 Wire-up subscriber gossip hors-scope** : Phase D livre
primitive + REST + CLI seulement. Wire-up automatique
`subscriber gossip → quarantine_queue.add()` reporté hors-scope
(dépend Sybil/kudos heuristics S22+). Documenté plan §7.3 +
design doc. ✓ Carry S22 en attente Sybil-resistance primitives.

**D-4 CLI transient coordinator pattern caveat** : `cli/commands/
quarantine.py:14-21` documente inline le caveat « conflict si
production coord déjà en cours sur le même project_dir (race
iroh data dir) ». Le CLI utilise `Coordinator(project_name).
start()/stop()` pattern cohérent `cli/commands/invite.py`. ✓
Visibility opérateur acceptable via docstring + warning log.

**Hook coverage gap** : cf. **P2-S21-5** ci-dessous.

### 3.5 Track E — Phase E tech debt batch (PASS validé par review pré-commit)

Commit `49f0d32 feat(sprint21): Phase E — tech debt batch (canary
JCS + registry verify Ed25519 + plan docs fix + PATTERNS §P34)`.
Pré-review `sprint21_phase_E_review.md` verdict PASS (0 P0 +
0 P1 + 2 P2 documentés + 1 P3 indépendant + 2 P3 SHA placeholder).

**E-1 build_canary serde_json vs JCS** : review P3 confirmée
audit. `crates/nexus-core-py/src/lib.rs:1134` `build_canary`
retourne `serde_json::to_string(&canary)` au lieu de JCS. Pas
de régression correctness (verify hash `canonical_bytes` sur
signed struct, pas wire bytes). **Déjà tracé** audit_plan §E-1
pour carry S22 (P3-E-2). ✓

**E-2 wheel-stale 16 tests bonus** : verification.md §2 note
« 16 du delta coord viennent de la rebuild PyO3 wheel forcée par
le `verify_canary` binding ». Root cause = wheel obsolete entre
phases. **Déjà tracé** audit_plan §E-2 pour carry S22
(P2-E-WIRE-PRE-LAUNCH-FIX) : ajouter check pre-flight
`maturin develop --release` fresh dans bootstrap §7. ✓

**E-3 verify_duress_ack hors-scope** : `canary.py:121-128`
documente inline que duress_ack reste observational-only (pas
verify Ed25519 at ingest). Decision consciente scope Phase E =
canary binding only. **Déjà tracé** audit_plan §E-3 pour carry
S22 (P2-E-DURESS-ACK). ✓

**E-4 PATTERNS §P34 T-NN+2 trigger** : `docs/rust/PATTERNS.md
§P34 T-NN+2` liste 3 triggers ré-ouverture (tract opset 19
coverage OR ort wasm32-browser stable OR gline-rs wasm-bindgen
target). Process check : ré-évaluation annuelle souhaitable
mais non-blocking. ✓ Note S22+.

Track E carry-overs déjà présents dans audit_plan. Aucun
nouveau finding audit-session.

### 3.6 Track F — Phase F wrap-up (PASS avec gap carry-over en P2 ci-dessous)

Commit `7887471 chore(sprint21): Phase F — wrap-up + verification
+ audit plan S22 + migrate planning`.

**F-1 Migration active/ → archive/v1.2/** : `ls .planning/
active/` vide pré-écriture du présent document. ✓ Les 17
fichiers sprint21 (sauf `phase_D_review.md` documenté absent)
présents dans `archive/v1.2/`. ✓

**F-2 Memory + CLAUDE.md + SPRINT_LOG + HARDENING_ROADMAP** :
- `memory/nexus_grid_pivot.md` frontmatter tip = `7887471` = HEAD
  ✓
- `CLAUDE.md §État actuel` S21 CLOSED présent + compteurs finals
  ✓
- `docs/claude/SPRINT_LOG.md` row S21 v1.2 présente (head -1
  retourne « Sprint log — historique cross-version ») ✓
- `docs/security/HARDENING_ROADMAP.md` frontmatter
  `last_validated: 2026-04-19` + audited_findings 2026-04-19
  entry présente ligne 18 ✓

**Finding carry-over gap** : cf. **P2-S21-4** ci-dessous. Phase F
wrap-up a oublié de porter dans `sprint21_audit_plan.md` les
P2/P3 findings issus des phase review files (Phase A review
P2-A-1/2 + P3-A-1, Phase B review P2-1/2 + P3). L'audit_plan
inclut bien les Phase E carries (E-1/2/3) mais le Track A et
Track B sous-items ne correspondent pas aux findings flaggés
par les review files. Ce document `audit_findings.md` comble
le gap explicitement (pattern §3.5 README : audit_findings.md
est la source canonique de carry vers S22 kickoff).

### 3.7 Meta-track — Radicle-v1.0 re-carry S22 : **[x] confirmé**

Re-carry S18→S19→S20→S21→**S22** confirmé. Décision flip
sequence self-contained dans `docs/release/MIRROR_FALLBACK.md
§3.1-3.8` (S18 Phase E3 `95807b1`). Codeberg private mirror
couvre l'intervalle pre-launch. Activation au jour du tag `v1.0`
go-public. Aucune déviation S21. ✓

### 3.8 Meta-track — G8 traceability : **PASS**

5/5 phases A-E ont émis un artefact G8 preflight ou
pivot_proposal avant 1re ligne de code. Phase F preflight
trivialement CLEAN. Aucun skip G8 sur S21. Verdict du skill
`nexus-phase-preflight` a catch 1 DESIGN-CONFLICT (Phase A
axum 0.7 stale) qui a débloqué un arbitrage Option C user avant
code, évitant un scope creep silent. Rigor signal G8 :
effectivité démontrée post-S20 codification. ✓

**Post-mortem léger** : les 4 SCOPE-CUT-CONSISTENT verdicts
(B/C/D/E) ont tous absorbé leurs findings inline via chore
planning préalable — pattern procéduralement propre. Aucun
verdict aurait dû être DESIGN-CONFLICT mais a été
SCOPE-CUT-CONSISTENT selon ma re-lecture (les findings étaient
bien naming/path drifts, pas des conflits Day 0 D1..D5).

### 3.9 Meta-track — Hook coverage Phase D : **P2 confirmé + investigation**

`git log --all -- '.planning/**phase_D_review*'` retourne
**vide**. Aucun fichier `sprint21_phase_D_review.md` n'a jamais
existé dans l'historique git, ni en `active/` ni en
`archive/`. Le commit `f830579 feat(sprint21): Phase D — ...`
a été committé avec succès 2026-04-19 08:31:13 alors que le
hook `phase-auditor-gate.sh` aurait dû bloquer.

**Analyse du hook** (`.claude/hooks/phase-auditor-gate.sh`) :
regex lines 47-48 matche correctement `feat(sprint21)` + `Phase D`
→ `SPRINT=21`, `PHASE=D`. Check `REVIEW_ACTIVE=".planning/
active/sprint21_phase_D_review.md"` → fichier absent. Check
`REVIEW_ARCHIVE=$(ls .planning/archive/v*/sprint21_phase_D_
review.md 2>/dev/null | head -1 || true)` → vide. Fallthrough
`exit 2` → **aurait dû bloquer**.

**Hypothèses non départageable depuis git history** :

1. **`NEXUS_SKIP_PHASE_AUDITOR=1` bypass env usage** : le script
   ligne 37 `[ "${NEXUS_SKIP_PHASE_AUDITOR:-0}" = "1" ] && exit 0`.
   Aucun audit trail de l'usage de cette variable (sauf
   éventuellement hooks de logging). Si cette route a été prise,
   c'est une décision user ad-hoc non-journalisée.
2. **Hook install state** : le hook est peut-être devenu
   inactif entre S20 et S21 (cleanup-orphan-hooks sidecar
   activity ?), ou le fichier n'est pas exécutable au moment
   du commit (drift permissions Windows).
3. **Bug regex** : improbable, test manuel avec commit title
   exact produit le match attendu.

**Action S22 recommandée** :

- **Court terme** : ajouter CI-side cross-check (GitHub Actions
  ou pre-receive git hook) qui parse chaque commit `feat
  (sprint\d+): Phase [A-Z]` et vérifie qu'un fichier review
  existe dans `active/` ou `archive/v*/` au moment de la
  création du commit. Fail si absent.
- **Audit trail** : modifier le bypass `NEXUS_SKIP_PHASE_AUDITOR=1`
  pour logger l'usage dans `.claude/.bypass_audit_trail.log` ou
  similaire afin d'être traçable post-hoc.
- **Hook integrity check** : vérifier périodiquement que
  `.claude/hooks/phase-auditor-gate.sh` est executable + non-
  modifié (CI check via hash).

Classification audit : **P2** (process gap, pas P1 sécurité
puisque Phase D a passé tous les tests + G8 SCOPE-CUT-
CONSISTENT + `nexus-phase-auditor` intra-phase aurait flaggé
des findings lors du review mais n'a pas tourné). La
remédiation est carry S22 HIGH priority.

---

## 4. Findings — liste consolidée

### 4.1 P2-S21-1 — RateLimiter primitive non-câblée dans engine loop (NEW carry)

**Description** : `crates/nexus-worker-core/src/rate_limit.rs`
ship la primitive `RateLimiter::check()` mais **aucun call-site
production** n'invoque cette méthode. Grep
`RateLimiter::check|rate_limit.*check|engine::accept` ne
retourne que les doc-comments du module lui-même. L'engine
runtime `crates/nexus-worker-core/src/engine/runtime.rs`
lignes 782 + 795 appelle uniquement `consent::should_accept_
task`. Le gate rate-limit est inert.

**Localisation** :
- Module shipped : `crates/nexus-worker-core/src/rate_limit.rs`
  (lines 224-238 `impl RateLimiter::check`).
- Consommation attendue : `crates/nexus-worker-core/src/engine/
  runtime.rs::scan_and_execute_tasks` (L688 ou path admission).
- PATTERNS promise : `docs/rust/PATTERNS.md §P33` lignes 1934-
  1938 annoncent « The full engine wire-up (task accept loop
  integration + retry channel) will land with Phase B or as a
  carry during Phase F verification ». **Phase B a livré PII
  SDK, Phase F a livré doc-wrap-up — la promesse n'a pas été
  tenue.**

**Impact threat model** : HARDENING_ROADMAP §3 `C-ModelExtract`
(impact 4, risque 5.3) + `C-DosFlood` (impact 4, risque 5.3)
désignent Sprint 21 comme le sprint qui livre le rate-limit
mitigation. En réalité, la primitive existe mais ne gate rien.
Les deux menaces restent **non-mitigées** jusqu'à wire-up.

**Rationale scope-plan Phase A** : le plan §4.1 liste bien
« primitive + loader + sample + deps + tests » comme
deliverables Phase A sans engine-wire-up explicite. Phase A
review a classé P2-A-1 avec carry promise. La promesse n'a
pas été tenue par Phase F.

**Action S22** :
1. Ajouter une Phase S22 dédiée rate-limit engine wire-up
   (estimée scope moyen : ~40-80 LOC engine + 1-2 tests
   intégration reject→defer).
2. Injecter `RateLimiter` via `EngineBoot::rate_limiter :
   Option<RateLimiter>` + appel `rate_limiter.check(&key)?`
   dans `scan_and_execute_tasks` avant `claim_task` sur le
   path admission.
3. Update `docs/security/HARDENING_ROADMAP.md` frontmatter
   `audited_findings` pour clarifier « S21 primitive shipped,
   engine wire-up S22 ».
4. Update `docs/rust/PATTERNS.md §P33` pour référencer le
   sprint de livraison réel (et retirer la promesse périmée
   « Phase B ou Phase F »).

### 4.2 P2-S21-2 — RateLimitPolicy hot-reload incomplet (NEW carry)

**Description** : `RateLimiter` charge la policy initiale via
`from_policy` puis construit `DefaultKeyedRateLimiter` **une
fois**. Le watcher
`rate_limit_policy_loader.rs::RateLimitPolicyWatcher::spawn`
met à jour l'`Arc<RwLock<RateLimitPolicy>>` sur write détecté,
mais les `DefaultKeyedRateLimiter` internes restent immutables.
Changer `default.per_min` ou ajouter un override consumer au
runtime n'a aucun effet sur le gate enforced (qui n'existe pas
de toute façon, cf. P2-S21-1).

**Localisation** : `crates/nexus-worker-core/src/rate_limit.rs`
L170-175 (commentaire `_policy` field honest) + L182-188
(`from_policy` construit snapshot initial puis n'a pas de
`refresh` subsequent).

**Rationale scope-plan Phase A** : limitation connue documentée
inline code (le code-commentaire est honnête). Les tests passent
parce qu'ils vérifient `Watcher::current()` reflète la policy
parsée, pas que `RateLimiter::check()` utilise la nouvelle
policy.

**Action S22** : ajouter `RateLimiter::refresh_from_policy()`
qui reconstruit les `DefaultKeyedRateLimiter` à partir du
snapshot courant + appelé par le watcher sur reload event.
Naturellement couplé avec P2-S21-1 engine wire-up.

### 4.3 P2-S21-3 — OnnxModelHandle scaffold returns [] (NEW carry)

**Description** : `web/src/sdk/pii/wrapper.ts:91-108`
`OnnxModelHandle.detect()` exécute la tokenisation et
`session.run()` sur le modèle ONNX GLiNER, puis **return []**.
Commentaire inline lignes 104-107 : « Post-processing: map
span logits to findings. The scaffolded loader returns an
empty set until the head decoder lands; the regex fallback
fills the gap until then. »

La détection PII iframe repose **exclusivement** sur le
fallback regex (`fallback.ts` 5 entités : email, phone, credit
card, SSN, IBAN avec inline Luhn). L'inférence ONNX coûte le
cycle CPU (45.8 MB modèle chargé + tokenize + session run)
sans contribuer aucune détection.

**Localisation** : `web/src/sdk/pii/wrapper.ts:107`.

**Rationale defense-in-depth** : (a) le fallback regex reste
actif pour 5 entités critiques, (b) la couche 2 Presidio
coord-side (Phase C) ferme le gap, (c) la limitation scaffold
est documentée inline avec la promesse d'un follow-up. Le
commit body Phase B n'a pas explicitement mentionné ce
scaffold gap.

**Action S22** : implémenter le span-logits → findings
decoder suivant la référence GLiNER paper (`urchade/gliner`
CCS 2024) ou équivalent. ~150-300 LOC TS + 3-5 tests Vitest
+ 1 Playwright end-to-end avec fixture model mini (à
budgétiser, voir aussi P3-S21-4 carry Playwright PII fixture).

### 4.4 P2-S21-4 — Carry-over gap Phase F wrap-up (NEW PROCESS carry)

**Description** : Phase F wrap-up a oublié de porter dans
`sprint21_audit_plan.md` les P2/P3 findings issus des phase
review files pré-commit. Le Track A du audit_plan liste 3
sous-items (G8 retrospective, R1 scope-cut doc, PoW bypass)
**différents** des 3 findings Phase A review (P2-A-1 engine
wire-up gap, P2-A-2 hot-reload incomplete, P3-A-1
burst_multiplier silent floor). Le Track B du audit_plan liste
3 sous-items (backbone resolution, size-limit, CSP) **différents**
des 3 findings Phase B review (P2-1 scaffold gap, P2-2 body
omission, P3 hf transformers range).

Le pattern carry depuis les phase review files vers
`sprint21_audit_plan.md` n'est pas formellement spécifié dans
README §3 ou nexus-phase-auditor.md, mais les phase review
files eux-mêmes documentent explicitement cette promesse :
« À tracer dans `sprint22_audit_plan.md §Track-X` Phase F +
re-carry S22 ».

**Impact process** : sans ce carry, les findings deviennent
invisibles cross-sprint. Si une session S22 fraîche lit
uniquement `sprint21_audit_plan.md` (pattern kickoff §Phase 0),
les findings Phase A/B sont manqués. Le présent document
`sprint21_audit_findings.md` comble le gap explicitement en
listant TOUS les findings (ceux déjà dans audit_plan +
ceux manqués).

**Action S22** :
1. Le kickoff `sprint22_kickoff.md` §4 « Acknowledged review
   findings » doit lister chaque P2 et P3 du présent
   `sprint21_audit_findings.md` comme ack ou decision
   (intégrer dans scope S22 vs defer S23+).
2. Ajouter dans `docs/claude/README.md §4.X` (commit
   discipline atomique) une règle explicite : **le Phase F
   wrap-up DOIT parcourir chaque `sprint{N}_phase_[A-F]_
   review.md` et intégrer les P2/P3 documentés dans
   l'audit_plan Track correspondant.** Liste par track lisible
   par la session fraîche S+1 Phase 0.
3. Considérer un check automatique (pre-commit hook ou skill
   `nexus-phase-review` extension) qui compare les findings
   du phase review au contenu du audit_plan.md track
   correspondant.

### 4.5 P2-S21-5 — Hook coverage Phase D bypass inexpliqué (NEW PROCESS carry)

**Description** : le commit `f830579 feat(sprint21): Phase D`
a été créé 2026-04-19 08:31:13 sans aucun fichier
`sprint21_phase_D_review.md` dans l'historique git. Le hook
`phase-auditor-gate.sh` (check `[ -f "$REVIEW_ACTIVE" ]` +
`[ -n "$REVIEW_ARCHIVE" ]` fallthrough `exit 2`) aurait dû
bloquer. Le bypass env `NEXUS_SKIP_PHASE_AUDITOR=1` n'a pas
de trail auditable.

**Hypothèses** (non départageable depuis git) :
1. Bypass env usage user ad-hoc.
2. Hook file non-executable ou absent au moment du commit
   (cleanup-orphan-hooks sidecar drift).
3. Bug latent dans le regex (improbable, test manuel OK).

**Action S22** :
1. Ajouter CI-side cross-check (GitHub Actions) qui parse
   chaque commit `feat(sprint\d+): Phase [A-Z]` et vérifie
   review file existe. Fail si absent.
2. Audit trail `NEXUS_SKIP_PHASE_AUDITOR=1` : logger usage
   dans `.claude/.bypass_audit_trail.log` au moment du
   commit.
3. Hook integrity check : vérifier périodiquement hash
   `.claude/hooks/phase-auditor-gate.sh` + executable bit.
4. Rétroactivement : décider si re-audit Phase D via session
   fraîche indépendante mérite d'être fait maintenant (risque
   mineur vu que Phase E review a scanné le diff Phase D
   indirectement + tous les tests Phase D passent + Design
   doc pre-Phase-A + preflight SCOPE-CUT-CONSISTENT).
   **Recommandation** : ne pas ré-auditer rétroactivement
   (ROI marginal vs effort), traiter comme leçon process
   forward.

### 4.6 P2-S21-6 — HARDENING_ROADMAP §3 S21 wording misleading (NEW carry)

**Description** : `docs/security/HARDENING_ROADMAP.md`
frontmatter `audited_findings 2026-04-19` ligne 18 décrit
Phase A comme « rate-limit governor 0.10.2 GCRA worker-engine
R1 (axum 0.7→0.8 bump prereq) ». Le libellé « worker-engine R1 »
peut être lu comme « rate-limit gate effectif au niveau engine
worker ». En réalité, la primitive ship-only (cf. P2-S21-1).

**Action S22** : update frontmatter entry pour clarifier
« Phase A ships rate-limit primitive + policy loader + tests,
engine wire-up deferred to Sprint 22 ». Peut être fait en même
temps que la Phase S22 engine wire-up landing.

### 4.7 P3-S21-1 — burst_multiplier < 1.0 silent floor (NEW carry)

**Description** : `make_quota` (`rate_limit.rs:287-301`) silently
floore le burst à `per_min` quand `burst_multiplier < 1.0` pour
éviter rejet `governor` (burst ≥ rate). L'opérateur qui écrit
`burst_multiplier = 0.5` avec `per_min = 10` pense avoir un
burst de 5, reçoit effectivement burst = 10.

**Action S22** : ajouter `tracing::warn!` au path floor
trigger dans `make_quota`. Cosmétique. Optionnel.

### 4.8 P3-S21-2 — @huggingface/transformers range `^4.0.0` (NEW carry)

**Description** : `web/package.json` épingle `onnxruntime-web`
exactement à `1.24.3` mais utilise `"^4.0.0"` caret range pour
`@huggingface/transformers`. Kickoff §D2 précise v4.x — donc
le range est intentionnel. Opportunité de pinner exact pour
reproductibilité.

**Action S22** : pinner à la version exacte courante (4.0.3
ou équivalent au moment du bump). Non-bloquant.

### 4.9 P3-S21-3 — build_canary serde_json vs JCS (already in audit_plan)

**Description** : `crates/nexus-core-py/src/lib.rs:1134`
`build_canary` PyO3 binding retourne `serde_json::to_string
(&canary)` — non-JCS. Incohérence visuelle avec
`canary_wire_bytes` désormais JCS. Pas de régression (verify
n'utilise pas wire-bytes). Test-surface only.

**Déjà tracé** audit_plan §E-1 (P3-E-2). ✓ Carry S22.

### 4.10 P3-S21-4 — rate_limit_policy.toml.sample absent (already in verification)

**Description** : verification.md §4.3 documente l'absence de
`crates/nexus-shell-daemon/configs/rate_limit_policy.toml.sample`
attendu par plan §10 row 21. R1 scope-cut Phase A a drop
l'HTTP middleware mais n'a pas livré le sample Rust-side non
plus. Le loader runtime cible `~/.sbfb/rate_limit_policy.toml`
avec defaults runtime suffisants.

**Déjà tracé** verification.md §5. Carry S22 Track A follow-up.

### 4.11 P3-S21-5 — Phase B Playwright review/verification inconsistency (NEW nit)

**Description** : `sprint21_phase_B_review.md` §Tests-delta
annonce « +5 Playwright » observed. `sprint21_verification.md`
row 16 §4.2 documente « +0 Playwright réel — Phase B livre les
tests Vitest unit (+15) pas de Playwright iframe PII end-to-end
ajouté ». Le fichier `web/tests/bridge-pii-redact.spec.ts`
existe mais n'apparaît pas dans le count Playwright final (38).

**Action S22** : investigation mineure — soit les 5 tests
Playwright existent mais ne sont pas collectés par la config
(`playwright.config.ts` glob ou excludePattern), soit le review
Phase B a sur-estimé. Non-bloquant, cosmétique.

### 4.12 P2-E-DURESS-ACK (already in audit_plan S22)

`verify_duress_ack` hors-scope explicite Phase E. Duress_ack
channel reste observational-only. S22+ follow-up si threat
model duress-ack élevé.

### 4.13 P2-E-WIRE-PRE-LAUNCH-FIX (already in audit_plan S22)

Wheel-stale 16 failures Phase E bonus fix. Carry S22 : check
pre-flight session `maturin develop --release` fresh dans
bootstrap §7 pour éviter wheel stale silencieux.

### 4.14 Meta-1 Radicle-v1.0 re-carry (already in audit_plan S22)

Re-carry S18→S19→S20→S21→**S22** confirmé. Deferred au tag
v1.0 go-public.

---

## 5. Findings sorted by severity

| ID | Severity | Description | Action |
|---|---|---|---|
| P2-S21-1 | P2 | RateLimiter primitive non-câblée engine | Carry S22 engine wire-up phase dédiée |
| P2-S21-2 | P2 | RateLimitPolicy hot-reload incomplet | Carry S22 (couplé P2-S21-1) |
| P2-S21-3 | P2 | OnnxModelHandle scaffold returns [] | Carry S22 span-logits decoder |
| P2-S21-4 | P2 | Carry gap Phase F review→audit_plan | Carry S22 + règle README §4.X |
| P2-S21-5 | P2 | Hook coverage Phase D bypass | Carry S22 CI-side check + audit trail |
| P2-S21-6 | P2 | HARDENING_ROADMAP wording misleading | Carry S22 update frontmatter |
| P2-E-DURESS-ACK | P2 | verify_duress_ack hors-scope | Already in audit_plan |
| P2-E-WIRE-PRE-LAUNCH-FIX | P2 | Wheel-stale bootstrap check | Already in audit_plan |
| P3-S21-1 | P3 | burst_multiplier silent floor | Optional S22+ |
| P3-S21-2 | P3 | hf transformers range ^4.0.0 | Optional S22+ |
| P3-S21-3 | P3 | build_canary JCS alignment | Already in audit_plan |
| P3-S21-4 | P3 | rate_limit_policy.toml.sample absent | Already in verification §5 |
| P3-S21-5 | P3 | Playwright Phase B review inconsistency | Optional investigation |
| Meta-1 | Meta | Radicle-v1.0 re-carry S22 | Confirmed, deferred to v1.0 tag |

**Total** : **8 P2 + 5 P3 + 1 Meta** = 14 items. 5 nouveaux
(P2-S21-1, P2-S21-2, P2-S21-3, P2-S21-4, P2-S21-5, P2-S21-6
avec P3-S21-1, P3-S21-2, P3-S21-5). 3 déjà dans audit_plan
(P2-E-DURESS-ACK, P2-E-WIRE-PRE-LAUNCH-FIX, P3-E-2 / P3-S21-3).
2 déjà dans verification §5 (P3-S21-4 + P3-S21-5 partiel).

---

## 6. Commits fix attendus

**Aucun `fix(sprint21): ...` requis pré-S22**. Tous les findings
sont des carries/documentation-of-gap non-bloquants pour
ouverture Sprint 22 Phase A. Le carry se fait **via ce
document** comme source canonique consommée par le kickoff S22
(pattern README §2.5 + §3.5).

**Sprint 22 kickoff responsable** :
- §4 Acknowledged review findings : lister chaque P2 + P3
  ci-dessus avec decision (intégrer dans scope S22 vs defer S23+)
- §5 Plan Phase outline : probable Phase A = rate-limit engine
  wire-up (résout P2-S21-1 + P2-S21-2) — ~1 phase budget
- §6 Scope cuts : confirmer P3-S21-1 + P3-S21-5 scope-cut vs
  fix opportuniste

---

## 7. P2 à logger en tech debt PATTERNS.md

- **P2-S21-1 + P2-S21-2** : déjà partiellement documentés
  PATTERNS §P33. Mise à jour recommandée post-wire-up S22 pour
  remplacer « will land with Phase B or Phase F verification »
  par le SHA réel du wire-up S22.
- **P2-S21-3** : nouveau entry PATTERNS §P35 suggéré « ONNX
  scaffold defense-in-depth : regex fallback + deferred
  post-processing », documentant le design pattern où l'inférence
  ONNX est câblée mais le decoder est itératif.
- **P2-S21-6** : non-PATTERNS, update HARDENING_ROADMAP.
- **P2-S21-4 + P2-S21-5** : process-level, pas PATTERNS. README
  §4 commit discipline et `phase-auditor-gate.sh` hook
  respectivement.

---

## 8. P3 laissés sans action (optionnels)

- **P3-S21-1** burst_multiplier silent floor warn : petite UX
  amélioration, peut être reporté.
- **P3-S21-2** hf transformers exact pin : maintenu caret
  range si pas de besoin reproductibilité strict.
- **P3-S21-5** Playwright inconsistency : investigation 15 min,
  optionnelle.

---

## 9. Notes on audit completeness

### Ce que l'audit a couvert

- Lecture ciblée des 17 fichiers `.planning/archive/v1.2/sprint21_*.md`
- 4 phase review files lus in-extenso (A/B/C/E). Phase D review
  absent (P2-S21-5). Phase F review non-lu (créé par cette
  session post-hoc si besoin — pattern S20).
- Rust test suite re-run indépendant : 659/659 pass ✓
- Coord Python test suite re-run indépendant : 249+3 skip ✓
- Grep indépendant `RateLimiter::check` / `engine::accept` /
  `rate_limit` pour vérifier A-3 PoW bypass
- Read `crates/nexus-worker-core/src/rate_limit.rs` lignes
  1-320 pour P2-S21-1 + P2-S21-2
- Read `web/src/sdk/pii/wrapper.ts` pour P2-S21-3
- Grep `InvisibleText` coverage coord-side pour C-1 verification
- Grep `DEFAULT_EED_THRESHOLD` et `eed_threshold` pour C-2
- Grep `DEFAULT_MODEL_NAME` pii_redactor vs wrapper pour C-3
- Read `.claude/hooks/phase-auditor-gate.sh` pour P2-S21-5
  investigation
- `git log --all -- '.planning/**phase_D_review*'` pour
  confirmation hook coverage gap
- Cross-check audit_plan Track E carries vs Phase E review
  findings : ✓ présents

### Ce que l'audit n'a pas couvert (timebox)

- Playwright test suite re-run (~2 minutes, skippé — pattern
  match verification.md)
- Vitest re-run (~1 minute, skippé — pattern match)
- Re-run des autres suites Python (SDK, app-gov) — pattern
  match
- Lecture ligne-par-ligne du design doc `.planning/research/
  S21_phase_B_iframe_pii_sdk_design.md` (423 lignes) —
  verdict PASS via phase B review cross-ref
- Lecture ligne-par-ligne du design doc Phase D quarantine
  (référencé via preflight)
- Audit rétroactif Phase D sans review.md (décision §4.5 :
  ROI marginal)
- Revue cryptographique indépendante de `canary_wire_bytes`
  JCS migration (Phase E review a déjà scan)
- Revue rétroactive du skill `nexus-phase-auditor` applied à
  Phase A/B/C/E reviews pour détecter les findings manqués
  (méta-audit de l'auditeur — hors scope)

### Recommandations process

**S22 kickoff §6.1.1** (Design Review Board) devrait inclure
un scan explicite des P2+ de ce document comme input au
scoring D1..D5 (pattern G1).

**S22 Phase 0 (gate ouverture)** est jouée par session S23
fraîche qui utilisera `sprint22_audit_plan.md` comme feuille
de route. L'audit gate pattern reste permanent.

---

**Fin findings S21**. Verdict **PASS** — Sprint 22 non bloqué.
Tous les P2/P3 sont des carries documentation-of-gap, aucune
régression fonctionnelle, aucun `*_VERSION` bumpé, aucun
scope-creep détecté vs kickoff §6 ni design_review §G1.
