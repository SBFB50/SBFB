# Sprint 22 — Audit plan pour Sprint 23 Phase 0

**Écrit** : 2026-04-20 (Phase F wrap-up S22).
**Cible** : session fraîche Sprint 23 Phase 0 qui joue l'audit gate
avant ouverture kickoff S23.
**Range audit** : commits Sprint 22 `87b0891..<Phase F tip>`.
**Verdict à produire** : `.planning/active/sprint22_audit_findings.md`
(PASS / CONDITIONAL PASS / FAIL — rigor signal G4 : ≥ 1 P2+
documenté exigé, sinon CONCERN pas PASS).

Ce plan guide l'auditeur sur **9 tracks** (A-F + meta-track Radicle-
v1.0 + meta-track G8 traceability + meta-track hook coverage
Phase D S21 closeout + meta-track agents_sudo hors-sprint
absorption). Pattern permanent depuis Sprint 7. Calibration G4 et
Working tree audit G5 dans `.claude/agents/nexus-phase-auditor.md`
et `.claude/skills/nexus-phase-review/SKILL.md`.

---

## Contexte d'entrée S23

- Tip S22 Phase F wrap-up (résolu post-commit dans
  `sprint22_verification.md §3` row 1).
- Compteurs tests finals : **Rust 710** / 185 SDK / **263+3
  skipped** coord / 46 gov / **264 Vitest** / 38 Playwright / 7/7
  size / 246+ SPDX (delta S22 vs baseline 1436 : **+73**).
- Audit gate S21 levé via `96a953b` verdict PASS (0 P0 + 0 P1 +
  findings P2/P3 carry S22 tous absorbés phases A-F — cf. Track
  A meta-absorption).
- Carries G7 cap **1/2 slots** consommés S22 : T-NN+2 iframe
  Rust-wasm hors cap formel PATTERNS §P34. Slot 2 libre ou post-
  Phase F findings.
- **LT-2 Meta-1 Radicle-v1.0 RECLASSIFICATION régularisée** kickoff
  S22 §4 D5 (sortie cap G7 formel, trigger unique tag v1.0
  go-live, cf. `docs/release/ROADMAP_COMMITMENTS.md §LT-2`).
- **LT-3 Contribution family Sybil matrix** créée hors-sprint
  chore `88eee23` + recherche `dbc4ceb` (réservation S31 stub).
- **LT-4 OS biometric gate cross-platform** créée hors-sprint chore
  `9676bd9` agents_sudo integration (`docs/release/ROADMAP_
  COMMITMENTS.md §LT-4`).
- **Pre-launch protocol policy codifiée CLAUDE.md** : aucun
  `*_VERSION` bumpé pendant S22, aucun tolerant decoder multi-
  version introduit. Nouveaux domain tags `DOMAIN_AGE_WITNESS_V1`
  + `DOMAIN_CONTRIBUTOR_ATTESTATION_V1` + `DOMAIN_DELEGATION_CERT
  _V1` (design-only) en pre-launch stable. Confirmation grep
  `_VERSION` Track-D audit.

## Commits S22 à auditer

```
<HEAD>     chore(sprint22): Phase F — wrap-up + verification + audit plan S23 + process fixes (P2-S21-4 + P2-S21-5) + migrate planning
690fab3    feat(sprint22): Phase E — watermark canari-input spot-check consumer 1/N primitive
4b7d38e    chore(claude): README §4.1.1 commit body file-based convention (fix heredoc fragility Windows Git Bash)
8146db7    chore(hook,agent): phase-auditor-gate C5 shell math fix + rouge-ligne C9 threat docs + agent prompt focus post-code optimisation
e621a92    chore(planning): sprint22 Phase E — G8 preflight verdict EXECUTE (watermark canari-input primitive)
9c8805b    chore(hook): C3 regex extend .rs|.py — couvrir crypto Python coord-side
34dacdc    chore(hook): phase-auditor-gate amendement criteres conditional run + lightweight pre-commit checks
b4926cf    chore(planning): open amendement proposal phase-auditor-gate criteres conditional run
56211f2    feat(sprint22): Phase D — NVML util+duree profile log-only baseline foundation S24
6f7601e    chore(planning): sprint22 Phase D — G8 preflight verdict SCOPE-CUT-CONSISTENT (NVML baseline log-only foundation S24)
df8a7a1    chore(claude): workflow docs 4.7 migration + MRCR self-test + effort/model discipline
dfd6222    chore(repo): remove accidentally committed commit message file (cf3918c staging artifact)
cf3918c    feat(sprint22): Phase C — Sybil-resistance composition 3 couches (age witness + contributor attestation + daemon proxy + Couche 3 RFC)
fb16a50    chore(planning): sprint22 Phase C — G8 preflight verdict EXECUTE (Sybil-resistance composition 3 couches)
9676bd9    chore(planning): agents_sudo integration S22 Phase F to S29 + LT-4 (18 features mapped + 3 design docs + research matrix)
e9530c2    feat(sprint22): Phase B — GLiNER span-logits decoder iframe SDK
eaa8d4f    chore(planning): sprint22 Phase B — G8 preflight verdict SCOPE-CUT-CONSISTENT (GLiNER span-logits decoder)
88eee23    chore(planning): open LT-3 contribution family Sybil matrix + reserve S31 stub + amend S23 hardening
dbc4ceb    chore(research): capture contribution family Sybil matrix analysis (S22 hors-sprint)
0bc499f    feat(sprint22): Phase A — rate-limit engine wire-up + hot-reload + policy sample smoke
9547ff3    chore(planning): sprint22 Phase A — G8 preflight verdict EXECUTE (rate-limit engine wire-up)
87b0891    chore(planning): open Sprint 22 — Sybil composition 3 couches + rate-limit engine wire + GLiNER span-decoder + NVML baseline + watermark canari + LT-2 Radicle reclassification
```

Plus le commit Phase F lui-même + un éventuel chore fix-up post-F.

---

## Track A — Phase A (rate-limit engine wire-up)

**Commit** : `0bc499f feat(sprint22): Phase A — rate-limit engine
wire-up + hot-reload + policy sample smoke`. Précédé de `9547ff3`
G8 preflight verdict EXECUTE.

**Pré-revue** : `sprint22_phase_A_review.md` (verdict PASS, 3 P2
+ 2 P3).

### A-1 Absorption wire-up debts S21

Phase A absorbe P2-S21-1 (RateLimiter primitive non-câblée engine),
P2-S21-2 (RateLimitPolicy hot-reload incomplet), P2-S21-6 (HARDENING
§3 S21 wording fix, chore planning opening), P3-S21-4 (`rate_limit
_policy.toml.sample` absent). **Audit check** : les 4 fixes sont-
ils effectifs dans le code livré ? Tests intégration engine
passent-ils (`engine::runtime::tests::rate_limit_gate_rejects_
saturated_tuple` etc.) ?

### A-2 P2-S22A-1 `dashmap` dep unused (carry S23 cleanup)

Review Phase A flag `dashmap = { workspace = true }` déclaré en
dépendance directe dans `crates/nexus-worker-core/Cargo.toml`
ligne 163, mais aucun `use dashmap` ni `dashmap::` dans le code
source après refacto governor. **Audit check S23** : confirmer
dépendance inutile (grep strict) et retirer du Cargo.toml (ou
justifier usage transitif). Severity P2 (bloat dependency).

### A-3 P2-S22A-2 `sprint21_verification.md` row 21 chemin obsolete

Review Phase A flag `sprint21_verification.md` row 21 pointe
`shell-daemon/configs/rate_limit_policy.toml.sample` (chemin
obsolete) alors que le fichier existe en `worker-core/configs/
rate_limit_policy.toml.sample`. **Audit check S23** : fichier
déjà archivé `.planning/archive/v1.2/sprint21_verification.md` —
peut être amend via footnote correction ou accepté as-is (audit
closed post-migration).

### A-4 P2-S22A-3 PATTERNS.md §P33 structure obsolète

`docs/rust/PATTERNS.md §P33` décrit encore l'ancienne structure
`RateLimiter` avec `Arc<DefaultKeyedRateLimiter>` + `Arc<DashMap
<...>>`. Post-refacto Phase A wire-up l'architecture réelle est
différente. **Audit check S23** : update §P33 wording ou ajouter
paragraphe « Post-S22 Phase A wire-up : ... ». Severity P2
(documentation drift).

### A-5 P3-S22A-1 estimations LOC prospectives plans S22

Review Phase A (et Phase B duplicate P3-B-1) flag que
`sprint22_plan.md §5-8` et `sprint22_kickoff.md` contiennent des
estimations LOC prospectives (`~250 LOC`, `~300 LOC`) contraires
à `docs/claude/README.md §6.7` (banni pour plans, seule borne de
scoping autorisée sans métrique succès). **Audit check S23** :
proposer amendement §6.7 README pour clarifier « estimations LOC
= borne scoping acceptable **si** documentée comme telle, pas
métrique succès ». Ou purger plans S22 retroactivement (migration).
Severity P3 (hygiène workflow).

---

## Track B — Phase B (GLiNER span-logits decoder iframe SDK)

**Commit** : `e9530c2 feat(sprint22): Phase B — GLiNER span-logits
decoder iframe SDK`. Précédé de `eaa8d4f` G8 preflight SCOPE-CUT-
CONSISTENT.

**Pré-revue** : `sprint22_phase_B_review.md` (verdict PASS, 2 P2
+ 3 P3).

### B-1 Absorption P2-S21-3 scaffold replace

Phase B absorbe P2-S21-3 (OnnxModelHandle scaffold returns []).
**Audit check** : `web/src/sdk/pii/wrapper.ts:82-108` appelle bien
le decoder `decodeSpans(start_logits, end_logits, span_logits,
tokens, threshold)` + `greedyDedup(spans)` + `toFinding()` au
lieu de `return []`.

### B-2 P2-B-1 End-to-end ONNX non exercé CI

Review flag : `jsdom` ne peut pas charger `onnxruntime-web` WASM
ni le modèle 45 MB. Le chemin `OnnxModelHandle.detect()` →
`decodeSpans()` → `toFinding()` n'est pas testé end-to-end en CI
(Vitest stubs). **Audit check S23** : fixture model mini dédiée
(trouver ONNX PII ≤ 10 MB ou distil GLiNER) + Playwright iframe
end-to-end. Carry confirmé Track B-drift S22 plan §11.

### B-3 P2-B-2 fallbackDetect déclenché sur 0 entités

Review flag : quand le modèle retourne 0 entités (texte sans PII
légitime), `wrapper.ts:308-311` `filtered.length === 0` déclenche
systématiquement `fallbackDetect()` — comportement résiduel
scaffold post-decoder. **Audit check S23** : décider sémantique
explicite — « 0 findings = pas de PII » vs « 0 findings = fallback
regex pour redondance ». Fix in-code + tests unitaires ajustés.
Severity P2 (faux négatif potentiel mais fallback sûr).

### B-4 P3-B-2/B-3 toFloat32Array branches défensives non exercées

Review flag : `wrapper.ts:175-179` `toFloat32Array` contient
branches `BigInt64Array`/`BigUint64Array` qui sont passées à
`Float32Array.from(..)` sans conversion explicite — commentaire
L177-178 documente correctement que ces types « ne supportent
pas le numeric indexing » mais branches non-couvertes par tests
unitaires (stubs `wrapper.test.ts` retournent directement
`PiiFinding[]`). **Audit check S23** : ajouter 2 tests dédiés ou
supprimer branches mortes. Severity P3 (couverture).

---

## Track C — Phase C (Sybil-resistance composition 3 couches)

**Commit** : `cf3918c feat(sprint22): Phase C — Sybil-resistance
composition 3 couches (age witness + contributor attestation +
daemon proxy + Couche 3 RFC)`. Précédé de `fb16a50` G8 preflight
EXECUTE. Suivi de `dfd6222` chore repo cleanup staging artifact.

**Pré-revue** : `sprint22_phase_C_review.md` (verdict PASS, 1 P2
delta tests recalibration + 1 P3 DOMAIN re-export).

### C-1 Absorption P0-G1-1 bootstrap allowlist

Kickoff §4 D1 + G1 review ack : `crates/nexus-shell-daemon-core/
src/bootstrap_allowlist.rs` créé (~100 LOC) + `~/.sbfb/bootstrap
_allowlist.toml` format + hot-reload pattern `pow_policy_loader.rs`
S20 + expires `v1.0`. **Audit check** : module wire au
`join_topic_with_age_witness` gossip (bootstrap self-witness pre-
v1.0), tests `bootstrap_allowlist::load_toml_schema` +
`is_bootstrap_node` + `rejects_expired` passent.

### C-2 Absorption P0-G1-2 design doc PREDICATE

Kickoff §4 D1 + G1 review ack : `docs/security/CONTRIBUTOR_
ATTESTATION_PREDICATE.md` créé AVANT le code Phase C (P0-G1-2 ack
obligatoire) — in-toto v1.0 spec + predicateType URI stable + JSON
schema draft-07 + fields + envelope + verification offline +
exemples + limitations (LT-1 Kudos-v2 horizon). **Audit check** :
`predicateType = "https://nexus-grid.org/contributor-attestation/v1"`
cité dans `contributor.rs` impl exact match doc.

### C-3 Absorption P2-G1-3 Matthew-effect TODO inline

Kickoff §4 D1 + G1 review ack : commentaire inline obligatoire
dans `curator::verify_with_contributor_registry()` et
`ContributorAttestation::build()` pointant `docs/FAIRNESS_VISION
.md §7` + `docs/release/ROADMAP_COMMITMENTS.md §LT-1`. **Audit
check** : grep `Matthew effect one layer deeper` dans `crates/
nexus-core-rs/src/attestations/contributor.rs` + `curator.rs`.

### C-4 P2 delta tests commit body recalibration

Review Phase C P2 : le body commit doit afficher le delta correct
post-nextest workspace run. Si body publie `+49` alors que real
`+30` ou autre, inflate. **Audit check S23** : grep commit body
`cf3918c`, vérifier nombre correspond à la réalité S22 verification
§2 table (delta Phase C = +30 vers le total +51 Rust).

### C-5 P3 DOMAIN_PROVENANCE_V1 + DOMAIN_WARRANT_CANARY_V1 non re-exportes

Review Phase C P3 cosmétique pré-existant hors-scope : `DOMAIN
_PROVENANCE_V1` + `DOMAIN_WARRANT_CANARY_V1` pas re-exportés en
`crates/nexus-core-rs/src/lib.rs`. **Audit check S23** : ajouter
re-export ou documenter raison exclusion (usage interne crates
uniquement). Severity P3 (API surface hygiene).

### C-6 Couche 3 RFC deferred implem S23-S27

`docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` design-only S22
(`SBFB.json::contributions[]` extension + `DelegationCert` Ed25519
+ parser `git log --show-signature` + multi-forge cross-validate +
Amnesty trust-web S27). **Audit check S23** : le RFC reste
design-only ; aucune implem code Rust/Python S22 sur Couche 3
(sauf trait delegation_cert squelette si présent — vérifier
`DELEGATION_CERT_VERSION = 1` design-only, pas code live).

---

## Track D — Phase D (NVML baseline log-only foundation S24)

**Commit** : `56211f2 feat(sprint22): Phase D — NVML util+duree
profile log-only baseline foundation S24`. Précédé de `6f7601e`
G8 preflight SCOPE-CUT-CONSISTENT.

**Pré-revue** : `sprint22_phase_D_review.md` (verdict PASS après
retrospective iteration 2 P1 resolution, 2 P2 + 1 P3 résolus inline).

### D-1 Resolution iteration 2 P1 profile.rs non-staged

Review Phase D documente P1 initial « `profile.rs` non-staged dans
commit atomique » — `git status --short` montrait `?? crates/
nexus-worker-core/src/gpu/profile.rs` malgré `gpu/mod.rs` déclarant
`pub mod profile;` dans le diff. Résolu iteration 2 : `git add
profile.rs` + amend commit (S22 Phase D pattern exception, non
répétable). **Audit check S23** : `git log --follow profile.rs`
confirme fichier tracké au tip ; vérifier aucune récurrence
pattern similaire S23+.

### D-2 P2 body commit ref THREAT_MODEL corrigée

Review Phase D : body commit mentionnait `THREAT_MODEL §7 ligne 85`
inexistant. Corrigé retrospective à `HARDENING_ROADMAP §3 ligne 85`
+ `THREAT_MODEL §7` sans ligne (ne pas citer ligne si pas présente).
Pattern à éviter : citer numéros de ligne dans docs référence qui
peuvent drift. **Audit check S23** : proposer convention README §6
pour body commits « cite section + sous-section, pas numéro ligne
doc externe ». Severity P2 (convention hygiène).

### D-3 P2 deviation LOC 643 vs ~250 documentée

Review Phase D : `sprint22_plan.md §7.2` estimait `~250 LOC` pour
NVML profile ; livré 643 LOC (2.5x). Body commit Phase D section
« Code organization deviation Option A » chiffre l'écart (tests
+150 + doc +120 + helpers +60). **Audit check S23** : pattern
identique P2-E-2 Phase E (plan estimation ~250 vs 520 livré) +
P3-S22A-1 — proposer S23 chore planning purge estimations LOC
prospectives dans plans à venir. Severity P2 (convention).

### D-4 P3 comment `last_seen_timestamp` corrigé

Review Phase D P3 résolu : docstring struct `NvmlComputeProcess`
corrigée inline `gpu/profile.rs:113-119` clarifiant wall-clock
vs future-proof shape S24. **Audit check** : vérifier docstring
actuel match review iteration 2 (pas de régression).

### D-5 S24 foundation carry

Phase D livre stats-only baseline. S24 consumera via random re-run
sampling (HARDENING §3 S24 dep, ligne 311). **Audit check S23** :
confirmer dependency mapping HARDENING §3 S24 ligne update « S22
NVML baseline livré → S24 anomaly detection consumer ».

---

## Track E — Phase E (watermark canari-input primitive)

**Commit** : `690fab3 feat(sprint22): Phase E — watermark canari-
input spot-check consumer 1/N primitive`. Précédé de `e621a92` G8
preflight verdict EXECUTE.

**Pré-revue** : `sprint22_phase_E_review.md` (verdict PASS LIGHT-
AUDIT, 0 P0 + 0 P1 + 2 P2 + 2 P3).

### E-1 P2-E-1 `_reload_policy_locked` suffix trompeur

Review Phase E P2 : `CanaryInputManager.__init__` ligne 504 appelle
`_reload_policy_locked` (suffixe `_locked` mais le helper n'acquiert
pas le lock lui-même, s'attend à être appelé sous lock). Appel
depuis `__init__` techniquement sûr (single-threaded par construction)
mais trompeur futur contributeur. **Audit check S23** : ajouter
commentaire inline `# safe: single-threaded init, lock not yet
needed` OU renommer `_reload_policy_inner` + ajouter wrapper
public. Non-bloquant. Severity P2 (naming convention drift).

### E-2 P2-E-2 pattern estimation LOC prospective

Review Phase E P2 : `plan §8.2` estimait `~250 LOC` pour `canary
_input.py` ; livré 520 LOC (2x). Deviation documentée body commit.
Pattern à éviter plans S23+. **Audit check S23** : confirmé
duplicate P3-S22A-1 (Phase A) + P2 Phase D D-3 = **trois
occurrences S22 du même pattern** — carry fort S23 chore planning
README §6.7 amendement explicite.

### E-3 P3-E-1 `/api/canary/observed-divergence` expose expected_answer

Review Phase E P3 : endpoint loopback bearer retourne `expected_
answer` + `observed_answer` dans chaque `DivergenceRecord`.
Acceptable loopback bearer single-user. **Carry S23 B1 Guardrails**
si alerting externe : surfacer filtre `?include_answers=false`
default-off ou sanitize avant export monitoring. `DEFAULT_SEED
_PROMPTS` docstring déjà avertit `canary_input.py:695-702`.
Severity P3 (design alerting durable).

### E-4 P3-E-2 CLI terminologie `canary-rotate` vs `canary rotate`

Review Phase E P3 : preflight §Action annonce `canary-rotate` (avec
tiret) alors que code enregistre `name="canary"` + sous-cmd
`rotate`. Invocation correcte `nexus-coordinator canary rotate`.
Ecart terminologie preflight uniquement, aucun impact fonctionnel.
**Audit check S23** : corriger preflight archivé (cosmétique) ou
accepter post-migration. Severity P3 (documentation).

---

## Track F — Phase F (wrap-up docs only — ce commit)

**Commit** : `<HEAD>` — chore(sprint22): Phase F — wrap-up +
verification + audit plan S23 + process fixes (P2-S21-4 + P2-S21-5)
+ migrate planning.

### F-1 Process fix P2-S21-4 README §4.X règle parse phase_review

`docs/claude/README.md §4.X` (modifié Phase F) : ajoute règle
explicite que wrap-up Phase F doit parser chaque
`sprint{N}_phase_[A-F]_review.md` et intégrer leurs P2/P3 dans
`sprint{N}_audit_plan.md` Track correspondant. **Audit check S23** :
verifier ce plan S22 applique la règle (chaque Track A-E a-t-il
sous-track Px-review correspondant ?) → si oui, règle effective.

### F-2 Process fix P2-S21-5 GHA phase-review cross-check

`.github/workflows/phase-review-cross-check.yml` (nouveau Phase F) :
GHA parse `git log --format='%s' master..HEAD | grep 'feat(sprint
\d+): Phase [A-F]'` et fail si fichier review.md correspondant
absent. `.claude/.bypass_audit_trail.log` (nouveau) trace chaque
usage `NEXUS_SKIP_PHASE_AUDITOR=1`. **Audit check S23** : workflow
green sur dry-run master ; si S22 Phase D review fut retrospectif
(cf. Meta-track hook coverage S21 closeout), le workflow l'aurait
catch.

### F-3 Migration `active/` → `archive/v1.2/`

Vérifier que `.planning/active/` est **vide** post-commit et que
les fichiers sprint22 sont tous présents dans `archive/v1.2/` :

- sprint22_kickoff.md
- sprint22_plan.md
- sprint22_design_review.md
- sprint22_carry_summary.md
- sprint22_phase_A_preflight.md + sprint22_phase_A_review.md
- sprint22_phase_B_preflight.md + sprint22_phase_B_review.md
- sprint22_phase_C_preflight.md + sprint22_phase_C_review.md
- sprint22_phase_D_preflight.md + sprint22_phase_D_review.md
- sprint22_phase_E_preflight.md + sprint22_phase_E_review.md
- sprint22_phase_F_preflight.md (+ sprint22_phase_F_review.md créé
  par session fraîche S23 Phase 0 pattern S18/S19/S20/S21)
- sprint22_verification.md
- sprint22_audit_plan.md (ce document)

### F-4 Memory + CLAUDE.md + SPRINT_LOG.md updated

Audit check :

- `memory/nexus_grid_pivot.md` frontmatter tip sync avec HEAD
  Phase F.
- `memory/MEMORY.md` row SBFB pivot résumé S22.
- `CLAUDE.md §État actuel` ajout Sprint 22 CLOSED + compteurs
  tests finaux + commits + carry LT-2/LT-3/LT-4.
- `docs/claude/SPRINT_LOG.md` row S22 finale sous v1.2.
- `docs/security/HARDENING_ROADMAP.md` `last_validated: 2026-04-
  20` (déjà bumped Phase B) + `audited_findings` entry S22
  CLOSED résumé.

---

## Meta-track — Radicle-v1.0 reclassification LT-2 (sortie cap G7)

**Reclassification** régularisée kickoff S22 §4 D5 (rattrapage règle
§6.2.1 après 3 carry-overs consécutifs S19/S20/S21). Sortie cap G7
formel vers `docs/release/ROADMAP_COMMITMENTS.md §LT-2`.

Trigger unique d'activation : tag `v1.0` go-live sur master →
réouvrir Meta-1 comme carry actif dans le sprint qui pose le tag
(réintégration cap G7). `docs/release/MIRROR_FALLBACK.md §3 "Flip
sequence Codeberg → Radicle"` reste la runbook.

**Audit check S23** : confirmer **[deferred] → LT-2 trigger v1.0
only** (pas de re-carry G7 S23, pas d'action requise).

---

## Meta-track — G8 traceability (Sprint 22 = 6/6 phases A-F)

**Deuxième sprint avec G8 systématique** (premier = S21 5/5 A-E).
Couverture S22 :

| Phase | G8 verdict | Document | Action user |
|---|---|---|---|
| A | EXECUTE | `sprint22_phase_A_preflight.md` | — (plan clean) |
| B | SCOPE-CUT-CONSISTENT | `sprint22_phase_B_preflight.md` | carry S23 (Playwright fixture) |
| C | EXECUTE | `sprint22_phase_C_preflight.md` | — (design G1 robuste) |
| D | SCOPE-CUT-CONSISTENT | `sprint22_phase_D_preflight.md` | NVML scope-cut stats-only → S24 anomaly |
| E | EXECUTE | `sprint22_phase_E_preflight.md` | — (primitive simple) |
| F | EXECUTE | `sprint22_phase_F_preflight.md` | — (wrap doc-only) |

**0 DESIGN-CONFLICT** déclenché S22 (vs 1 en S21 Phase A axum
bump). Lecture : kickoff §4 D1..D5 post-G1 Design Review Board
robuste ; factuel-research pre-gel a fermé les angles morts.

**Audit check S23** : le zero DESIGN-CONFLICT S22 prouve-t-il la
qualité du G1 pre-gel (et non un sous-déclenchement G8 trop
permissif) ? Vérifier au moins 1 finding post-mortem où G8 aurait
dû DESIGN-CONFLICT mais n'a fait que SCOPE-CUT-CONSISTENT. Si
trouvé, calibrer rules d'agrégation Step 6 skill preflight.

---

## Meta-track — Hook coverage Phase D S21 closeout

**Carry S21 audit** : Phase D S21 sans `sprint21_phase_D_review.md`
(hook `phase-auditor-gate.sh` `57c829d` avait un gap post-Phase A).
S22 livre 3 chore corrections hook :

- `9c8805b` C3 regex extend `.rs|.py` (couvre crypto Python coord-side)
- `34dacdc` amendement critères conditional run + lightweight
  pre-commit checks
- `b4926cf` proposal amendement
- `8146db7` C5 shell math fix + rouge-ligne C9 threat docs + agent
  prompt focus post-code

**Audit check S23** :

1. Hook coverage a-t-il bien catché toutes les phases S22 B/C/D/E ?
   Confirmer présence de `sprint22_phase_{B,C,D,E}_review.md` dans
   active/ au moment de chaque commit respectif.
2. Le gap Phase D S21 est-il totalement fermé post-S22 process
   fix F-2 (GHA workflow) ? Ou le workflow ne catch que sur PR et
   pas sur local commit ? Documenter limitation.
3. `.bypass_audit_trail.log` a-t-il enregistré 0 ou N usages
   `NEXUS_SKIP_PHASE_AUDITOR=1` S22 ? Si N > 0, vérifier
   justifications inline.

Severité : P2 (process gap partiellement fermé).

---

## Meta-track — agents_sudo hors-sprint absorption S22

**Commit hors-sprint** `9676bd9` (2026-04-20) : analyse deep
`openai-agents-python` + `microsoft/sudo` → 18 features produit
mappées sprint-by-sprint S22 Phase F → S29 + LT-4 via `.planning/
research/S23_to_S29_agents_sudo_integration_matrix.md` + 3 design
docs (`docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` + `docs/
security/CAPABILITY_TOGGLES.md` + `docs/security/GUARDRAILS_
ARCHITECTURE.md`) + `docs/release/ROADMAP_COMMITMENTS.md §LT-4`.

**S22 Phase F absorption** : D1 three-mode trade-off doc
`docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (189 LOC) 3
tiers AUTO / CONFIRM_PROMPT / BIOMETRIC_GATE + `consent.json`
schema `level_threat_note` + `residual_threats_acknowledged`.
**Implementation** : T1 CONFIRM_PROMPT = S25 co-landing D5 ;
T2 BIOMETRIC_GATE = LT-4 post-v1.0.

**Audit check S23** :

1. Aucun code Rust/Python introduit S22 sur ce item (pur doc
   absorption). Grep diff S22 `consent.json` / `loopback_endpoints`
   → 0 match attendu hors docs.
2. HARDENING §3 ligne S22 + ligne S25 `audited_findings` entry
   « 2026-04-20 S22 hors-sprint agents_sudo integration » déjà
   posée au moment de la Phase F (voir frontmatter).
3. Mapping S23+ amendements documenté research matrix : B1
   S23 Guardrails refactor pipeline déclaratif + A1 S24 hooks
   TaskDispatchHooks + A3/B2/C2/C5/D5 S25 + A4/C1 S26 + C4/D2/D3
   S28 + A2/B4 S29. **Kickoff S23 doit arbitrer B1 timing** (dédié
   / distribué / défer).

---

## Meta-track — LT-3 Contribution family Sybil matrix hors-sprint

**Commit hors-sprint** `88eee23` + `dbc4ceb` : création LT-3
Contribution family Sybil matrix (`.planning/research/S22_
contribution_family_sybil_matrix.md` archive) + réservation S31
stub + amendement S23 hardening.

**Audit check S23** :

1. `docs/release/ROADMAP_COMMITMENTS.md §LT-3` exists with
   trigger + runbook.
2. `HARDENING_ROADMAP §3 S23` bump ligne amendement : Couche 3
   RFC implem preview S23 + LT-3 decision boundary pattern.
3. S31 stub réservé mais pas ouvert (design-only).

Severity : P3 (roadmap documentation).

---

## Critères verdict S23 Phase 0

- **PASS** : 0 P0 + 0 P1 + ≥ 1 P2+ documenté dans
  `sprint22_audit_findings.md`. Si 0 P2+ trouvé, le verdict est
  **CONCERN** (pas PASS) — re-auditer dimension manquée :
  research-grounding, horizon long-terme, working-tree audit,
  G8 traceability, hook coverage, pattern LOC estimations, etc.
- **CONDITIONAL PASS** : ≥ 1 P1 résolu inline dans la session
  d'audit (commits `fix(sprint22): ...`).
- **FAIL** : ≥ 1 P0 ou ≥ 1 P1 non résolu après la session
  d'audit. Re-conception requise sur la zone touchée.

Range commits attendu pour fix-ups éventuels :
`<HEAD>..<post-audit-fix tip>`. Inclure tout fix dans le range
audit-finishing avant ouverture du kickoff S23.

---

## Pour Sprint 23 kickoff (post-audit)

- Confirmer **LT-2 Radicle-v1.0** reste `[deferred]` (trigger tag
  v1.0 unique). Pas de re-carry G7.
- Confirmer **LT-3 Contribution family Sybil matrix** reste
  design-only S31.
- Confirmer **LT-4 OS biometric gate** reste post-v1.0.
- **Scope S23** : B1 Guardrails refactor pipeline déclaratif
  (agents_sudo absorption) + Couche 3 multi-forge cross-validate
  preview + ephemeral workers + escalating PoW + honeypot +
  redundancy voting 3-worker (carry S22 co-deferrer) + P2-B-1/
  P2-B-2 fixes ONNX end-to-end + carries P2 cleanup (dashmap
  dep + PATTERNS §P33 + LOC estimations README §6.7 amend)
- G1 Design Review Board agent Explore indépendant (cf. README
  §6.1.1) pour scoring D1..D5
- G2 trigger revalidate scan sur HARDENING_ROADMAP +
  `openai-agents-python release > 0.7.0` (cf. triggers frontmatter)
  avant gel D1..D5
