# Sprint 21 — Audit plan pour Sprint 22 Phase 0

**Écrit** : 2026-04-19 (Phase F wrap-up S21).
**Cible** : session fraîche Sprint 22 Phase 0 qui joue l'audit gate
avant ouverture kickoff S22.
**Range audit** : commits Sprint 21 `b34d451..<Phase F tip>`.
**Verdict à produire** : `.planning/active/sprint21_audit_findings.md`
(PASS / CONDITIONAL PASS / FAIL — rigor signal G4 : ≥ 1 P2+
documenté exigé, sinon CONCERN pas PASS).

Ce plan guide l'auditeur sur **8 tracks** (A-F + meta-track Radicle-
v1.0 + meta-track G8 traceability + meta-track hook coverage).
Pattern permanent depuis Sprint 7. Calibration G4 et Working tree
audit G5 dans `.claude/agents/nexus-phase-auditor.md` et
`.claude/skills/nexus-phase-review/SKILL.md`.

---

## Contexte d'entrée S22

- Tip S21 Phase F wrap-up (résolu post-commit dans
  `sprint21_verification.md §3` row 1).
- Compteurs tests finals : **Rust 659** / 185 SDK / **249+3
  skipped** coord / 46 gov / Vitest <résolu Phase F> /
  Playwright 38 / 7/7 size / 246+ SPDX (delta S21 vs baseline 1371
  : <résolu Phase F>).
- Audit gate S20 levé via `66a3a7c` verdict PASS (0 P0 + 0 P1 + 4
  P2 carry actifs + 6 P2 résolus in-phase + 6 P3 cosmétiques).
- Carries G7 cap 2/2 fermés Phase E (T-NN canary JCS + T-NN+1
  registry verify Ed25519). Re-carry S22 : Meta-1 Radicle-v1.0
  + T-NN+2 iframe Rust-wasm (PATTERNS.md hors cap formel).
- **Pre-launch protocol policy codifiée CLAUDE.md** : aucun
  `*_VERSION` bumpé pendant S21, aucun tolerant decoder multi-
  version introduit. Confirmation grep `_VERSION` Track-D audit.

## Commits S21 à auditer

```
<HEAD>     chore(sprint21): Phase F — wrap-up + verification + audit plan S22 + migrate planning
49f0d32    feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix + PATTERNS §P34)
f830579    feat(sprint21): Phase D — quarantine queue SQLite WAL + manual flush CLI
a82e8db    chore(planning): sprint21 §7 Phase D realignement coord-Python + design doc
17035c3    chore(docs): fairness vision + LT-1 Kudos-v2 commitment + S22 roadmap flag
23abb11    feat(sprint21): Phase C — coord-side PII redaction + output filter (Presidio GLiNER + local InvisibleText + EED echo)
041d8d0    chore(planning): sprint21 §6.2 pivot D3 — drop llm-guard (transitive-pin presidio-analyzer conflict) + local InvisibleText scanner
624ad7e    chore(planning): sprint21 §6.2 naming-fix — dispatcher + validator hooks (drop inexistant task_response_validator.py)
d5b0035    feat(sprint21): Phase B — client-side PII redaction SDK iframe (onnxruntime-web + GLiNER PII edge)
63afe4e    feat(sprint21): Phase A — rate-limit sliding-window multi-tier per-(consumer, worker, model) via governor GCRA worker-engine gate R1
57c829d    chore(hook): phase-auditor-gate.sh — fix false-positive exit 2 on missing archive review
b4bda81    chore(planning): sprint21 Phase A R1 scope-cut — worker-engine gate pure Rust (drop HTTP middleware drift)
5e67ce0    chore: bump axum 0.7 → 0.8 workspace-wide
60adceb    chore(planning): sprint21 Phase A — G8 pivot proposal + arbitrage Option C (axum 0.7 → 0.8 bump prereq)
f5ad2e1    chore(research): sprint21 — archive 4 research outputs (§6.11 pattern rétroactif)
8ba1d7c    chore(planning): sprint21 kickoff §D2 — backbone ModernBERT confirmed + ort-wasm post-G1 re-check + multi-agent team fix acknowledgments
71de0ec    chore(workflow): docs/claude/README.md — §6.1.1 G1 ext custom Rust stack + §6.2.1 carry long-term + §6.10 G9 factual-research + §6.11 archive research outputs
7e34fe6    chore(agents): nexus-phase-auditor — eliminate forced rigor signal + anti-hallucination code-reread + archive-immediately note
b34d451    chore(planning): open Sprint 21 — rate-limit + PII SDK defense-in-depth + output filter + quarantine queue
```

Plus le commit Phase F lui-même + un éventuel chore fix-up post-F.

---

## Track A — Phase A (rate-limit sliding-window multi-tier)

**Commit** : `63afe4e feat(sprint21): Phase A — rate-limit sliding-
window multi-tier per-(consumer, worker, model) via governor GCRA
worker-engine gate R1`. Précédé de `60adceb` G8 pivot proposal +
`b4bda81` scope-cut R1 + `5e67ce0` axum 0.7→0.8 bump.

**Pré-revue** : `sprint21_phase_A_review.md` (verdict du temps).
Re-scanner indépendamment.

### A-1 G8 pivot Option C arbitrage user retrospective

Phase A a déclenché un G8 pivot proposal `60adceb` qui a flag axum
0.7 stale (release > 0.7 = breaking) et user a arbitré Option C
(bump axum 0.7 → 0.8 workspace-wide en pré-requis Phase A code).

**Audit check** : verdict de l'audit pivot rétrospectif — la
décision Option C a-t-elle introduit des cassures dans d'autres
crates dépendant axum 0.7 (cf. `cargo build --workspace` post-
bump) ? `git log --oneline 5e67ce0..HEAD -- crates/` montre-t-il
des fix-up tardifs liés au bump ?

### A-2 R1 scope-cut HTTP middleware drift

Plan §4 original ciblait `tower-governor 0.8` middleware HTTP +
worker engine gate. Scope-cut R1 chore `b4bda81` a réduit à
worker-engine pure Rust (drop HTTP middleware). Audit : ce scope-
cut est-il documenté dans `sprint21_audit_plan.md` Phase A
section + carry-over correspondant pour S22 si besoin futur ?

### A-3 PoW pré-requis Phase A

Plan §4 indiquait dépendance PoW gossip wire (S20 Phase C
`16b94ba`) comme pré-requis. Vérifier que le rate-limit ne
ré-introduit pas un bypass du PoW gate (ex : un consommateur
légitime qui ne fait que des requêtes non-gossip pourrait
contourner). Audit : revisiter `crates/nexus-worker-core/src/
rate_limit.rs` pour confirmer que le gate est bien sur le path
runtime `engine::accept`.

---

## Track B — Phase B (client-side PII redaction SDK iframe)

**Commit** : `d5b0035 feat(sprint21): Phase B — client-side PII
redaction SDK iframe (onnxruntime-web + GLiNER PII edge)`.

**Pré-revue** : `sprint21_phase_B_review.md`.

### B-1 G8 S1 backbone GLiNER finding

`sprint21_phase_B_preflight.md` documente un S1 finding :
backbone GLiNER ambigu ModernBERT vs DeBERTa-v3. Memory tip dit
résolu Phase B G8 S1 scan obligatoire. Audit check : la résolution
est-elle bien documentée dans le code (commentaire inline pointant
vers le backbone retenu) + dans `sprint21_phase_B_preflight.md`
verdict ?

### B-2 onnxruntime-web 1.24.3 + bundle size

Vérifier que `web/scripts/size-limit` n'a pas été drift par le
nouveau dep onnxruntime-web (~10 MB WASM). Plan §10 row 15 attend
7/7 pass après Phase B. Si hors-cap, justifier inline + carry
audit_plan optimization S22+.

### B-3 Iframe sandbox CSP

`onnxruntime-web` fetch le modèle ONNX via réseau ou bundle local.
Vérifier que le CSP iframe `connect-src 'none'` (CLAUDE.md
§Modele de rendu) est respecté : modèle bundled dans l'archive
zip de l'app, pas fetched runtime. Sinon = régression sandbox.

---

## Track C — Phase C (coord-side PII redaction + output filter)

**Commit** : `23abb11 feat(sprint21): Phase C — coord-side PII
redaction + output filter (Presidio GLiNER + local InvisibleText
+ EED echo)`. Précédé de `041d8d0` pivot D3 drop llm-guard +
`624ad7e` naming fix.

**Pré-revue** : `sprint21_phase_C_review.md`.

### C-1 D3 pivot drop llm-guard rationale

`041d8d0` chore documente le drop `llm-guard 0.3.16` (transitive-
pin conflict avec `presidio-analyzer 2.2.362`) et le fallback
local InvisibleText scanner. Audit check : le scanner local
custom couvre-t-il les mêmes catégories (zero-width + PUA U+E000-
F8FF + Tag chars U+E0020-E007F) que llm-guard ? Test coverage
suffisante (cf. `test_output_filter.py`) ?

### C-2 EED seuil 0.85 empirique

Kickoff §D3 fige seuil Levenshtein 0.85. Audit check : le seuil
est-il configurable via `~/.sbfb/output_filter_policy.toml` hot-
reload (pattern S20 PoW policy) ? Inline comment dans
`output_filter.py` documente le tradeoff faux positifs vs faux
négatifs ?

### C-3 Presidio + GLiNERRecognizer extra `[gliner]`

Audit check : `pyproject.toml` extras `[gliner]` documenté ? Le
modèle ONNX consumé coord-side est bien le **même** que celui
chargé par l'iframe (single source of truth, kickoff §D2 figé) ?

---

## Track D — Phase D (quarantine queue SQLite WAL + CLI)

**Commit** : `f830579 feat(sprint21): Phase D — quarantine queue
SQLite WAL + manual flush CLI`. Précédé de `a82e8db` chore
réalignement coord-Python + design doc.

**Pré-revue** : **AUCUNE** dans `.planning/active/` ni archive.
Voir Meta-track « Hook coverage » ci-dessous.

### D-1 G8 SCOPE-CUT-CONSISTENT realignment

`sprint21_phase_D_preflight.md` documente 4 findings non-bloquants
(S2-D1+S2-D2 paths drift daemon Rust → coord Python +
S2-D3 cohérence pattern f238d31 + S2-D4 design doc absent).
Tous absorbés inline via `a82e8db` chore préalable. Audit check :
les paths du plan §7.2 réalignés sont bien implémentés
(`packages/nexus-coordinator/src/nexus_coordinator/quarantine_
queue.py` + `api/quarantine.py` + `cli/commands/quarantine.py`) ?

### D-2 Test cardinality 1k vs plan 10k

Plan §7.3 row 5 spécifiait `test_cardinality_10k_entries_no_panic`.
Implémentation `test_cardinality_1k_entries_sweeps_clean` (réduit
à 1k pour rester sous timeout pytest 60s, design doc §6.1
documente). Audit check : le drift est-il acceptable retrospective
ou faut-il un bench harness 10k stress séparé Phase F+ S22 ?

### D-3 Wire-up automatique subscriber gossip hors-scope

Phase D livre primitive + REST + CLI seulement. Wire-up
automatique subscriber gossip → `quarantine_queue.add()` reporté
hors-scope (carry S22+ dépend Sybil/kudos heuristics). Audit
check : carry-over explicite dans ce document Track D-3 + design
doc §7.3 Phase D documente le hors-scope ?

### D-4 CLI Coordinator transient pattern caveat

Le CLI `nexus-coordinator quarantine list/flush/drop` utilise le
pattern `Coordinator(project_name).start() / stop()` (cohérent
`cli/commands/invite.py`) qui caveat : conflict si production
coord déjà en cours sur le même project_dir (race iroh data dir).
Documenté inline `cli/commands/quarantine.py:14-21`. Audit check :
caveat est-il assez visible pour l'opérateur ? Future `--remote`
flag tracker S22+ ?

---

## Track E — Phase E (tech debt batch canary JCS + registry verify)

**Commit** : `49f0d32 feat(sprint21): Phase E — tech debt batch
(canary JCS + registry verify Ed25519 + plan docs fix + PATTERNS
§P34)`.

**Pré-revue** : `sprint21_phase_E_review.md` verdict PASS (0 P0
+ 0 P1 + 2 P2 pré-documentés + 2 P3 indépendants).

### E-1 P3-E-2 build_canary serde_json non-JCS cohérence

Audit P3-E-2 : `build_canary` PyO3 retourne `serde_json::to_string
(&canary)` non-JCS alors que `canary_wire_bytes` est désormais
JCS. Carry S22+ : aligner build_canary sur JCS pour cohérence
visuelle (zéro impact correctness — verify n'utilise pas wire-
bytes).

### E-2 P2-E-WIRE-PRE-LAUNCH-FIX wheel-stale silencieux

Audit P2 : la rebuild PyO3 wheel via `maturin develop --release`
en Phase E a accidentellement corrigé 16 failures coord pré-
existantes (provenance + dispatcher + deploy). Ces failures
étaient dues à un wheel obsolete (sign_task/verify_task_entry
bindings out-of-sync avec Rust master). **Carry S22 audit_plan** :
ajouter check pre-flight session « `maturin develop --release`
fresh ? » dans bootstrap §7 pour éviter wheel stale silencieux
en début de session fraîche.

### E-3 verify_duress_ack hors-scope explicite

Phase E plan §8.1 E-2 limitait à `verify_canary` only.
`verify_duress_ack` reste hors-scope (documenté inline `api/
canary.py:121-128` + PATTERNS §P34 « Carries closed »). Audit
check : le carry S22+ doit-il être promoted en P1 si threat model
S22 élève le coût d'un duress_ack channel observational-only ?

### E-4 PATTERNS §P34 T-NN+2 ouvert S22+

T-NN+2 iframe Rust-wasm Option G reste tracké hors cap G7 formel
(PATTERNS.md tech debt entry). Audit check : la trigger condition
(tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen) est-
elle vérifiée annuellement ou seulement au sprint qui l'envisage ?
Process check, pas blocking.

---

## Track F — Phase F (wrap-up docs only — ce commit)

**Commit** : `<HEAD>` — chore(sprint21): Phase F — wrap-up +
verification + audit plan S22 + migrate planning.

### F-1 Migration `active/` → `archive/v1.2/`

Vérifier que `.planning/active/` est **vide** post-commit et que
les fichiers sprint21 sont tous présents dans `archive/v1.2/` :

- sprint21_kickoff.md
- sprint21_plan.md
- sprint21_design_review.md
- sprint21_carry_summary.md
- sprint21_phase_A_pivot_proposal.md + sprint21_phase_A_review.md
- sprint21_phase_B_preflight.md + sprint21_phase_B_review.md
- sprint21_phase_C_preflight.md + sprint21_phase_C_review.md
- sprint21_phase_D_preflight.md (+ pas de review.md, cf. Meta-
  track hook coverage)
- sprint21_phase_E_preflight.md + sprint21_phase_E_review.md
- sprint21_phase_F_preflight.md + sprint21_phase_F_review.md
  (créé par session fraîche S22 Phase 0)
- sprint21_verification.md
- sprint21_audit_plan.md (ce document)

Phase F review sera créé par la session fraîche S22 Phase 0
(pattern S20).

### F-2 Memory + CLAUDE.md + SPRINT_LOG.md updated

Audit check :

- `memory/nexus_grid_pivot.md` frontmatter tip sync avec HEAD
  Phase F.
- `memory/MEMORY.md` row SBFB pivot résumé S21.
- `CLAUDE.md §État actuel` ajout Sprint 21 CLOSED + compteurs
  tests finaux + commits + carry Meta-1 S22 + archive path
  `archive/v1.2/` étendu S16-21.
- `docs/claude/SPRINT_LOG.md` row S21 ajoutée sous v1.2.
- `docs/security/HARDENING_ROADMAP.md` `last_validated: 2026-04-
  19` + §3 S21 résumé livré.

---

## Meta-track — Radicle-v1.0 activation tracking (re-carry S18→S19→S20→S21→S22)

**Carry confirmé** depuis S18 Phase E3 `95807b1`. Activation reste
deferred au moment du tag v1.0 go-live (Codeberg mirror disaster-
recovery couvre l'intervalle pre-launch).

Audit check : la décision flip sequence est documentée
self-contained dans `docs/release/MIRROR_FALLBACK.md §3` (S18
Phase E3 wrap-up). Le re-carry S22 reste explicit par le présent
audit_plan + memory `nexus_grid_pivot.md` § Carries G7.

**Action S22 Phase 0** : confirmer ligne par ligne `[x] Meta-1
re-carry confirmé pour S22 audit_plan` ou `[deferred] → S23`.

---

## Meta-track — G8 traceability (Sprint 21 = 5/5 phases A-E)

**Premier sprint avec G8 systématique post-S20 codification**
(`59225ee` workflow introduction). Couverture S21 :

| Phase | G8 verdict | Document | Action user |
|---|---|---|---|
| A | DESIGN-CONFLICT → Option C | `sprint21_phase_A_pivot_proposal.md` | arbitrage axum 0.7→0.8 bump |
| B | SCOPE-CUT-CONSISTENT | `sprint21_phase_B_preflight.md` | backbone GLiNER S1 finding résolu |
| C | SCOPE-CUT-CONSISTENT | `sprint21_phase_C_preflight.md` | drop llm-guard requalifié `041d8d0` |
| D | SCOPE-CUT-CONSISTENT | `sprint21_phase_D_preflight.md` | drift paths absorbé `a82e8db` |
| E | SCOPE-CUT-CONSISTENT | `sprint21_phase_E_preflight.md` | binding manquant créé inline `49f0d32` |
| F | EXECUTE plan-as-is | `sprint21_phase_F_preflight.md` | (clean trivially) |

**Audit check S22** : le coverage 5/5 prouve-t-il l'efficacité du
G8 (catch des drifts plan-vs-code que le review pre-commit ne
catche pas) ? Y a-t-il des findings post-mortem où G8 aurait dû
DESIGN-CONFLICT mais n'a fait que SCOPE-CUT-CONSISTENT ? Si oui,
calibrer les rules d'agrégation Step 6 du skill.

---

## Meta-track — Hook coverage investigation (Phase D sans review)

**Observation** : le commit Phase D `f830579` n'a pas de fichier
`sprint21_phase_D_review.md` correspondant dans `.planning/
active/` ni `archive/v1.2/`. Phase E `49f0d32` en revanche a bien
déclenché le hook `phase-auditor-gate.sh` qui a exigé le review.
Le commit `57c829d chore(hook): phase-auditor-gate.sh — fix
false-positive exit 2 on missing archive review` est intervenu
**entre** Phase A et Phase D mais semble avoir laissé un cas non
catch (Phase D).

**Audit check S22** :

1. Lire `phase-auditor-gate.sh` actuel et identifier la condition
   qui a manqué (peut-être dépend du nom de fichier `phase_D_*`
   parsing) ;
2. Confirmer que toutes les phases B/C qui ont un review.md ont
   bien été catch par le hook au moment de leur commit ;
3. Décider si on rétroactivement audit Phase D via une session
   fraîche (audit indépendant) avant de fermer S21, ou si le
   risque est acceptable post-mortem (vu que Phase D a passé tous
   les tests + G8 SCOPE-CUT-CONSISTENT + audit Phase E n'a rien
   trouvé sur Phase D).

**Severité estimée** : P2 (process gap, pas P1 sécurité). Carry
S22 priority HIGH.

---

## Critères verdict S22 Phase 0

- **PASS** : 0 P0 + 0 P1 + ≥ 1 P2+ documenté dans
  `sprint21_audit_findings.md`. Si 0 P2+ trouvé, le verdict est
  **CONCERN** (pas PASS) — re-auditer dimension manquée :
  research-grounding, horizon long-terme, working-tree audit,
  G8 traceability, hook coverage.
- **CONDITIONAL PASS** : ≥ 1 P1 résolu inline dans la session
  d'audit (commits `fix(sprint21): ...`).
- **FAIL** : ≥ 1 P0 ou ≥ 1 P1 non résolu après la session
  d'audit. Re-conception requise sur la zone touchée.

Range commits attendu pour les fix-ups éventuels :
`<HEAD>..<post-audit-fix tip>`. Inclure tout fix dans le range
audit-finishing avant ouverture du kickoff S22.

---

## Pour Sprint 22 kickoff (post-audit)

- Confirmer `Meta-1 Radicle-v1.0` re-carry [x] ou [deferred]
- Décider du scope S22 : Sybil/kudos resistance (HARDENING_ROADMAP
  §3 S22 candidat principal) + tool-calling allow-list strict +
  redundancy voting + carry P2-E-DURESS-ACK + P2-E-WIRE-PRE-
  LAUNCH-FIX + tech debt P3-E-2 build_canary JCS alignement
- G1 Design Review Board agent Explore indépendant (cf. README
  §6.1.1) pour scoring D1..D5
- G2 trigger revalidate scan sur HARDENING_ROADMAP +
  WARRANT_CANARY_HARDENING avant gel D1..D5
