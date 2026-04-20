# Sprint 22 — Audit findings (session fraîche S23 Phase 0)

**Écrit** : 2026-04-20 (session fraîche post-S22 Phase F wrap-up,
audit gate obligatoire pattern permanent depuis Sprint 7).
**HEAD audité** : `f65914e` (Phase F wrap-up).
**Range audité** : `87b0891..f65914e` (22 commits — 5 feat phases
A-E + 1 chore wrap F + 12 chore planning/hors-sprint + 1 cleanup +
1 amendement hook + 2 chore hook).
**Input plan** : `.planning/archive/v1.2/sprint22_audit_plan.md`
(9 tracks : A-F + meta-LT-2 + meta-G8 + meta-hook + meta-agents_sudo
+ meta-LT-3).
**Auditor** : session fraîche Claude Opus 4.7 (1M context).
**Calibration G4** : rigor signal satisfait (≥ 1 P2+ documenté,
8 P2 + 4 P3 carry S23 ci-dessous + re-verification inline factuelle
de chaque finding via `Read` + `Grep` sur le tip `f65914e`).

---

## Verdict : **PASS**

0 P0 + 0 P1 + **8 P2 + 4 P3** tous documentés en carry S23 ou
résolus inline durant la phase émettrice. Sprint 23 Phase A
**non-bloqué**.

Détail agrégé des critères §Critères verdict du plan :

- **P0 (blocking)** : 0. Aucune régression wire-format, aucune
  surface loopback nouvelle non-auth, aucun usage crypto hors
  nexus_core / aws-lc-rs / ed25519-dalek audités, aucune dep
  sans trace research §3 du plan S22.
- **P1 (blocking)** : 0. Toutes les wire-ups Phase A/B/C sont
  effectives runtime (greps ci-dessous), Phase D profile.rs est
  tracked git (résolu iteration 2 retrospective), Phase E CLI
  enregistre correctement `rotate`/`status` sous-commandes.
- **P2 (non-blocking)** : 8 findings, tous tracés dans des
  entrées plan `sprint22_audit_plan.md` Tracks A/B/E (carry
  S23 Phase A cleanup batch) ou résolus inline par la phase
  émettrice.
- **P3 (non-blocking)** : 4 findings cosmétiques (DOMAIN re-
  export, CLI terminologie preflight, LOC estimation advisory,
  alerting design carry B1 S23).

**CONDITIONAL PASS non déclenché** : aucun P1 à résoudre inline,
la procédure §Critères ne le demande pas. Pattern S18/S19/S21
reproduit : PASS propre, carry S23, no `fix(sprint22)` requis.

---

## 1. Résultats par track

### Track A — Phase A (rate-limit engine wire-up)

Commit `0bc499f feat(sprint22): Phase A`. Pré-revue
`sprint22_phase_A_review.md` verdict PASS (3 P2 + 2 P3).

| Check | Verdict | Evidence |
|---|---|---|
| A-1 Absorption wire-up debts S21 (P2-S21-1/2/6 + P3-S21-4) | **PASS** | `crates/nexus-worker-core/src/engine/runtime.rs` contient l'appel `RateLimiter::check` pre-`ClaimEntry` (wire effective). `rate_limit.rs` expose `swap_policy(...)` pour hot-reload Arc swap. `configs/rate_limit_policy.toml.sample` existe. Verification.md rows 19-21 ✅. |
| A-2 P2-S22A-1 dashmap dep unused | **CARRY S23** (P2) | `crates/nexus-worker-core/Cargo.toml:163` déclare `dashmap = { workspace = true }` mais grep `use dashmap\|dashmap::` sur `crates/nexus-worker-core/src/` retourne **0 match**. Dep stale post-refacto governor. Carry S23 Phase A cleanup. |
| A-3 P2-S22A-2 `sprint21_verification.md` row 21 chemin obsolète | **ACCEPT AS-IS** | `.planning/archive/v1.2/sprint21_verification.md` archivé post-migration ; amendement cosmétique non-bloquant, footnote possible S23 mais coût > bénéfice. Closed post-migration. |
| A-4 P2-S22A-3 PATTERNS.md §P33 structure obsolète | **CARRY S23** (P2) | `docs/rust/PATTERNS.md:1821-1865` : header "Sprint 21 Phase A" non update post-S22 wire ; struct snapshot cite toujours `default: Arc<DefaultKeyedRateLimiter<RateKey>>` + `overrides: Arc<DashMap<ConsumerId, ...>>` alors que le code `rate_limit.rs:167-186` a migré vers `RwLock<RateLimiterState>` groupant default + overrides. Carry S23 Phase A ou paragraphe « Post-S22 wire-up ». |
| A-5 P3-S22A-1 estimations LOC prospectives plans B-F | **CARRY S23** (P3) | Confirmé pattern sur 3 phases S22 (A advisory, D 643 LOC vs ~250 plan §7.2, E 520 LOC vs ~250 plan §8.2). Carry S23 chore planning amendement `docs/claude/README.md §6.7` pour clarifier LOC = borne scoping, pas métrique succès, ou purge plans S22 rétroactivement. |

**Track A verdict** : **PASS** avec 2 P2 + 1 P3 carry S23.

---

### Track B — Phase B (GLiNER span-logits decoder)

Commit `e9530c2 feat(sprint22): Phase B`. Pré-revue
`sprint22_phase_B_review.md` verdict PASS (3 P2 + 3 P3).

| Check | Verdict | Evidence |
|---|---|---|
| B-1 Absorption P2-S21-3 scaffold replace | **PASS** | `web/src/sdk/pii/wrapper.ts:31-33` importe `decodeSpans, greedyDedup, toFinding`. Ligne 153 `const spans = decodeSpans(...)` + ligne 160 `return greedyDedup(spans).map((s) => toFinding(s, tokenOffsets))`. Scaffold `return []` remplacé. Verification.md row 23 ✅. |
| B-2 P2-B-1 End-to-end ONNX non exercé CI | **CARRY S23** (P2) | Plan §Track B drift — fixture model mini dédiée (< 10 MB) OU Playwright iframe end-to-end requis S23. Blocage jsdom + 45 MB model non résoluble inline. |
| B-3 P2-B-2 fallbackDetect déclenché sur 0 entités | **CARRY S23** (P2) | `web/src/sdk/pii/wrapper.ts:308-313` confirmé : `if (filtered.length === 0)` → `fallbackDetect(text, policy)`. Commentaire L309 « Scaffold path returns empty » **obsolète post-Phase B** (le scaffold a été remplacé). Décision sémantique S23 : « 0 findings = pas de PII » vs « 0 findings = fallback regex redondance ». Acceptable defense-in-depth actuellement, mais commentaire induit erreur. Carry S23. |
| B-4 P3-B-2/B-3 toFloat32Array branches défensives | **CARRY S23** (P3) | `wrapper.ts:175-179` branches `BigInt64Array`/`BigUint64Array` non exercées (stubs `wrapper.test.ts` bypass). Carry S23 Track B. |

**Track B verdict** : **PASS** avec 2 P2 + 1 P3 carry S23.

---

### Track C — Phase C (Sybil-resistance composition 3 couches)

Commit `cf3918c feat(sprint22): Phase C` (+ `dfd6222` chore
cleanup). Pré-revue `sprint22_phase_C_review.md` verdict PASS
post-iteration 2 (P1 proxy http.rs levé : route + authed + guard
hex + token forward + 502 wired + 3 tests).

| Check | Verdict | Evidence |
|---|---|---|
| C-1 Absorption P0-G1-1 bootstrap allowlist | **PASS** | `crates/nexus-shell-daemon-core/src/bootstrap_allowlist.rs` existe (~553 lignes tests inclus). `crates/nexus-core-rs/src/gossip.rs:103-111` définit `AgeAdmissionPolicy` trait avec `fn is_bootstrap_node(&self, node_id: &[u8; 32]) -> bool`. `gossip.rs:147` consomme `policy.is_bootstrap_node(joining_node_id)`. `gossip.rs:289` définit `pub async fn join_topic_with_age_witness<P: AgeAdmissionPolicy>(...)` avec fallback PoW-only si witness absent (`gossip.rs:305`) + rejet witness invalide (`gossip.rs:311`). Tests `gossip.rs:492,522,544,575` couvrent sign/verify/reject. Module `BootstrapAllowlistWatcher` `bootstrap_allowlist.rs:357+` mirror exact `PowPolicyWatcher` S20 (notify + debounce 50ms + keep-last on delete + keep-last on malformed). |
| C-2 Absorption P0-G1-2 design doc PREDICATE | **PASS** | `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md:8` cite `Predicate URI : https://nexus-grid.org/contributor-attestation/v1`. `crates/nexus-core-rs/src/attestations/contributor.rs:60` définit `pub const CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE: &str = "https://nexus-grid.org/contributor-attestation/v1"`. Match exact byte-pour-byte. Schema JSON draft-07 ligne 73 + envelope in-toto v1.0 ligne 54 + verification procedure ligne 189-190 + exemples ligne 154/244. |
| C-3 Absorption P2-G1-3 Matthew-effect TODO inline | **PASS** | 3 sites grep confirmés : `contributor.rs:28-34` (module doc `//! ## Matthew-effect caveat (LT-1)`), `curator.rs:293-298` (docstring `verify_with_contributor_registry` pointant `FAIRNESS_VISION.md §7` + `ROADMAP_COMMITMENTS.md §LT-1`), `packages/nexus-coordinator/src/nexus_coordinator/contributor_registry.py:29-36` (docstring Python coord-side avec mêmes refs). |
| C-4 P2 delta tests body recalibration | **PASS** | `git log -1 --format=%B cf3918c` : body affiche `Delta reel +42 (+36 Rust + +6 Python coord ; premier body +36 Rust + 6 Py = +42 factuel)`. Recalibré correctement depuis +49 initial → +42 factuel. Cohérent avec verification.md cumul Phase C. |
| C-5 P3 DOMAIN_PROVENANCE_V1 + DOMAIN_WARRANT_CANARY_V1 non re-exportés | **CARRY S23** (P3) | `crates/nexus-core-rs/src/lib.rs:64-66` re-exporte `DOMAIN_AGE_WITNESS_V1`, `DOMAIN_CLAIM_V1`, `DOMAIN_CONTRIBUTOR_ATTESTATION_V1`, `DOMAIN_CURATOR_LIST_V1`, `DOMAIN_DURESS_ACK_V1`, `DOMAIN_INVITE_V1`, `DOMAIN_KUDOS_V1`, `DOMAIN_POW_V1`, `DOMAIN_RESULT_V1`, `DOMAIN_TASK_V1`. Pas de `DOMAIN_PROVENANCE_V1` ni `DOMAIN_WARRANT_CANARY_V1` bien que définis `canonical.rs:104` et `canonical.rs:113`. Cosmétique API surface — carry S23. |
| C-6 Couche 3 RFC deferred S23-S27 | **PASS** | Grep `DelegationCert|DOMAIN_DELEGATION_CERT|DELEGATION_CERT_VERSION` sur `crates/` retourne 1 hit seulement : `attestations/mod.rs:23-24` commentaire `//! Couche 2. The Couche 3 DelegationCert is reserved (design-only)`. Aucun code implem. `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` ~250 LOC design-only S22. Implem S23-S27 planifiée. |

**Track C verdict** : **PASS** avec 1 P3 carry S23.

---

### Track D — Phase D (NVML baseline log-only foundation S24)

Commit `56211f2 feat(sprint22): Phase D`. Pré-revue
`sprint22_phase_D_review.md` verdict FAIL initial → PASS post-fix
iteration 2 (P1 profile.rs staging + 2 P2 body ref/LOC + 1 P3
doc comment).

| Check | Verdict | Evidence |
|---|---|---|
| D-1 Resolution iteration 2 P1 profile.rs staging | **PASS** | `git log --follow --oneline crates/nexus-worker-core/src/gpu/profile.rs` retourne `56211f2 feat(sprint22): Phase D`. `git ls-files` confirme `crates/nexus-worker-core/src/gpu/profile.rs` tracked. Aucune récurrence pattern staging manquant S22 ultérieur (Phase E `690fab3` clean). |
| D-2 P2 body commit THREAT_MODEL ref corrigée | **PASS** | `git log -1 --format=%B 56211f2` cite `HARDENING_ROADMAP.md §3 ligne 85 (table threats actifs C-ComputeTheft mitigation NVML-profile)` + `THREAT_MODEL.md §7 (mitigations table generale)` sans ligne — correctement amendé du draft initial (qui disait `THREAT_MODEL §7 ligne 85`). Pattern à éviter futur : ne pas citer numéro ligne docs référence externes qui drift. Carry possible README convention §6 amendement, mais non-bloquant. |
| D-3 P2 deviation LOC 643 vs ~250 | **PASS** (documenté) | Body commit D section « Code organization deviation Option A » chiffre `Deviation LOC : reel 643 lignes vs plan §7.2 estime ~250 (~2.5x). Origine [...]`. Breakdown implicite dans la section (tests + doc + helpers). Même pattern P2-E-2 Phase E (520 vs ~250) + P3-S22A-1 → **trois occurrences S22** du même pattern LOC estimation prospective → meta-carry S23 chore planning README §6.7. |
| D-4 P3 comment `last_seen_timestamp` corrigé | **PASS** | `crates/nexus-worker-core/src/gpu/profile.rs:113-119` docstring struct `NvmlComputeProcess` clarifie wall-clock (`current_unix_seconds()`) vs future-proof shape S24 (cf. pré-revue D P3 fix appliqué). |
| D-5 S24 foundation carry | **PASS** | `docs/security/HARDENING_ROADMAP.md:794` confirme séquence `S22 NVML baseline profile ────> S24 random re-run sampling ( C-ComputeTheft )  ( C-ComputeTheft detection )`. Dependency mapping à jour. |

**Track D verdict** : **PASS** (0 carry résiduel Phase D, P2 body ref
cosmétique accepté post-migration).

---

### Track E — Phase E (watermark canari-input primitive)

Commit `690fab3 feat(sprint22): Phase E`. Pré-revue
`sprint22_phase_E_review.md` verdict PASS LIGHT-AUDIT (2 P2 + 2 P3).

| Check | Verdict | Evidence |
|---|---|---|
| E-1 P2-E-1 `_reload_policy_locked` suffix trompeur | **CARRY S23** (P2) | `canary_input.py:508` appelle `self._reload_policy_locked()` depuis `__init__` (après `self._lock = threading.Lock()` ligne 501). Le helper ligne 619 `_reload_policy_locked` n'acquiert PAS le lock (attend d'être sous lock appelant). Single-threaded par construction dans `__init__` = safe, mais suffix `_locked` trompeur pour futur contributeur. Fix trivial : commentaire inline ligne 508 `# safe: single-threaded init, lock not yet needed` OU rename `_reload_policy_inner` + wrapper public. Non-bloquant. |
| E-2 P2-E-2 pattern LOC estimation prospective | **CARRY S23** (P2 → meta-carry) | Confirmé 3e occurrence S22 (Phase A P3-S22A-1 advisory + Phase D 643 LOC body deviation + Phase E 520 LOC body deviation). Carry S23 fort → chore planning amendement `docs/claude/README.md §6.7` avec clarification explicite « LOC = borne scoping admise plan si documentée comme telle, pas métrique succès ». |
| E-3 P3-E-1 `/api/canary/observed-divergence` expose `expected_answer` | **CARRY S23** (P3) | `canary_input.py:179-180` `DivergenceRecord.to_dict` retourne `expected_answer` + `observed_answer` en clair. Acceptable loopback bearer single-user. Carry S23 design alerting B1 Guardrails : surfacer filtre `?include_answers=false` default-off OU sanitize avant export monitoring externe. Docstring `DEFAULT_SEED_PROMPTS canary_input.py:695-702` avertit déjà. |
| E-4 P3-E-2 CLI terminologie `canary-rotate` vs `canary rotate` | **ACCEPT AS-IS** | Preflight E ligne 158 + 195 mentionne `canary-rotate` et `canary-status` (avec tiret). Code `cli/commands/canary.py:46` enregistre `@app.command("rotate")` + ligne 126 `@app.command("status")` sous `name="canary"` (cli/main.py). Invocation correcte `nexus-coordinator canary rotate`. Écart cosmétique preflight archivé, aucun impact fonctionnel. Non corrigé (migration post-archive, footnote possible mais coût > bénéfice). |

**Track E verdict** : **PASS** avec 2 P2 + 1 P3 carry S23.

---

### Track F — Phase F (wrap-up docs only — commit `f65914e`)

| Check | Verdict | Evidence |
|---|---|---|
| F-1 Process fix P2-S21-4 README §4.X | **PASS** | `docs/claude/README.md:544` contient section `### 4.4 Phase F wrap-up — parse phase reviews et route les P2/P3 au audit_plan`. Lignes 546-553 explicitent la règle : parse systématique `sprint{N}_phase_[A-F]_review.md` → injection audit_plan Track correspondant. Rationale ligne 550-553 cite le gap pré-S22. Ce plan S22 applique la règle (chaque Track A-E contient bien les P2/P3 des reviews respectifs). |
| F-2 Process fix P2-S21-5 GHA phase-review cross-check | **PASS** | `.github/workflows/phase-review-cross-check.yml` existe (4154 bytes). Workflow correct : `on: pull_request: branches: [master, main]` + regex `feat\(sprint[0-9]+\): Phase [A-F]` + vérification présence `review_active` (active/) OU `review_archive` (find .planning/archive/**). `.claude/.bypass_audit_trail.log` créé avec header schema + format + justification pattern. **Observation** : workflow ignore délibérément les commits `chore(sprintN): Phase X` — donc Phase F review manquante est compatible (Phase F = chore pas feat). Workflow dry-run sur S22 = green (5 feat A-E tous reviews présents). |
| F-3 Migration `active/` → `archive/v1.2/` | **PASS** | `ls .planning/active/` : vide (seul `.claude/` dossier de tooling). 17 fichiers `sprint22_*.md` dans `archive/v1.2/`. `sprint21_audit_findings.md` archivé aussi. Inventaire plan §F-3 complet : kickoff, plan, design_review, carry_summary, verification, audit_plan + 6 preflights A-F + 5 reviews A-E (Phase F review créée par ce commit). |
| F-4 Memory + CLAUDE.md + SPRINT_LOG + HARDENING update | **PASS** | `memory/nexus_grid_pivot.md` Tip `f65914e` synchro HEAD. `CLAUDE.md §État actuel` row Sprint 22 CLOSED 2026-04-20 + compteurs finaux + commits ligne par ligne + carry LT-2/LT-3/LT-4. `docs/claude/SPRINT_LOG.md v1.2` row S22 table détaillée avec commits + fichiers livrés. `docs/security/HARDENING_ROADMAP.md` `last_validated: 2026-04-20` + `audited_findings:` entry S22 CLOSED (résumé Phase F intégral). `MEMORY.md` row SBFB pivot résumé S22 cumul (descriptions mis à jour). |

**Track F verdict** : **PASS**. Wrap-up complet, 0 gap process.

---

## 2. Meta-tracks

### Meta-track — Radicle-v1.0 LT-2 (sortie cap G7)

**Audit check** : confirmer `[deferred] → LT-2 trigger v1.0 only`
(pas de re-carry G7 S23).

**Verdict** : **PASS**. `docs/release/ROADMAP_COMMITMENTS.md:106`
section `## LT-2 Meta-1 Radicle-v1.0 activation tracking` avec
trigger + runbook. Régularisation kickoff §4 D5 (rattrapage règle
§6.2.1 après 3 carry-overs consécutifs S19/S20/S21). Sorti du cap
G7 formel. Réouverture conditionnée tag `v1.0` go-live uniquement.
Aucune action S23 requise.

### Meta-track — G8 traceability (6/6 phases A-F)

**Audit check** : le zero DESIGN-CONFLICT S22 prouve-t-il la
qualité du G1 pre-gel, et non un sous-déclenchement G8 trop
permissif ?

**Verdict** : **PASS** calibration saine. Vérification indépendante
des 6 preflight :

| Phase | G8 verdict | Finding critique ? Scope-cut vs Design-conflict |
|---|---|---|
| A | EXECUTE plan-as-is | — (plan clean, wire-up mécanique) |
| B | SCOPE-CUT-CONSISTENT | S1-B-1 pseudocode 3-tensor vs canonical single-tensor = **adaptation upstream reality** (spec GLiNER context7 corrigeait l'hypothèse initiale). SCOPE-CUT-CONSISTENT correct — pas un design-conflict. |
| C | EXECUTE plan-as-is | — (design G1 robuste, P0-G1-1 + P0-G1-2 + P2-G1-3 acknowledgments pré-code) |
| D | SCOPE-CUT-CONSISTENT | S1-1 `memory_info` v1→v2 API + S2-1 module `gpu/` pré-existe → Option A integration = **adaptation structure module existante**. SCOPE-CUT-CONSISTENT correct. |
| E | EXECUTE plan-as-is | — (primitive simple, stack existant reused) |
| F | EXECUTE plan-as-is | — (doc-only trivial, S1-S4 tous clean) |

**Conclusion** : 0 finding post-mortem où G8 aurait dû DESIGN-
CONFLICT. Les 2 SCOPE-CUT-CONSISTENT (B et D) sont cohérents avec
leur définition (adaptation scope sans remise en cause Day 0).
Calibration G8 saine. Deuxième sprint consécutif G8 systématique
(S21 5/5 + S22 6/6) valide la procédure.

**Observation positive** : la qualité du G1 pre-gel S22 (§4.5
kickoff — P0-G1-1 bootstrap ceremony + P0-G1-2 in-toto PREDICATE
doc avant code + P2-G1-3 Matthew-effect caveat TODO + P2-G1-4
decomposition Couche + P2-G1-5 watermark prior-art search + P3-G1-6
NVML hardware matrix + P3-G1-7 LT-2 timing) a **fermé les angles
morts pré-gel**. Design Review Board indépendant agent G1 = levier
efficace pour 0 DESIGN-CONFLICT downstream.

### Meta-track — Hook coverage Phase D S21 closeout

**Audit check** :
1. Hook coverage a-t-il catché toutes les phases S22 B/C/D/E ?
2. Gap Phase D S21 fermé post-S22 process fix F-2 (GHA) ?
3. `.bypass_audit_trail.log` — 0 ou N usages `NEXUS_SKIP_PHASE_AUDITOR=1` ?

**Verdict** : **PASS avec 1 P2 structurel documenté**.

1. **Hook coverage S22** : 5/5 reviews Phase A-E présents dans
   `archive/v1.2/`. Phase F review absent = **pattern S18-21
   attendu** (créé par session fraîche S+1 Phase 0 — ce commit).
2. **Gap fermé post-S22** :
   - Hook `phase-auditor-gate.sh` amendements S22 `34dacdc`
     (critères conditional run + lightweight pre-commit checks) +
     `9c8805b` (regex extend `.rs|.py` crypto Python) + `8146db7`
     (C5 shell math fix + C9 rouge-ligne threat docs + prompt
     focus optimization).
   - GHA workflow `phase-review-cross-check.yml` catch côté PR
     (pas sur push master, cf. commentaire en-tête workflow).
   - **Limitation documentée** : le workflow ne catch que sur PR.
     En workflow master-direct-push (cas nexus-grid actuellement
     sans PR process), le GHA ne s'active pas. Hook local reste
     la garantie primaire. À reconsidérer S23+ si flow PR
     introduit.
3. **`.bypass_audit_trail.log`** : fichier créé header-only avec
   schema + entries vide. Vérification post-création Phase F — la
   Phase D iteration 2 du S22 avait utilisé `NEXUS_SKIP_PHASE_
   AUDITOR=1` (cf. review D lignes 31-40), mais **pas append dans
   le log** (log créé Phase F, bypass Phase D antérieur). **P2
   Meta-hook-coverage-1** : logiquement, Phase D aurait dû
   retroactively append une ligne dans le log post-Phase F. Carry
   S23 — soit retrospective append (cosmétique), soit clarifier
   README §6 que le log est forward-only depuis sa création Phase F
   (ce qui est le pattern GitHub workflow standard). Non-bloquant.

### Meta-track — agents_sudo hors-sprint absorption S22 Phase F

**Audit check** :
1. Aucun code Rust/Python introduit S22 sur ce item (pur doc).
2. HARDENING §3 S22 + S25 `audited_findings` entry placée ?
3. Mapping S23+ documenté research matrix ?

**Verdict** : **PASS**.

1. **Code check** : grep sur diff S22 `consent.json\|loopback_
   endpoints` dans fichiers code (`.rs|.py`) = 0 match hors docs
   security/ (les 3 docs `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` 189L
   + `CAPABILITY_TOGGLES.md` 280L + `GUARDRAILS_ARCHITECTURE.md`
   341L sont docs-only). Pur doc absorption. Conforme kickoff
   §D5 agents_sudo : "T1 CONFIRM_PROMPT = S25 co-landing D5 ;
   T2 BIOMETRIC_GATE = LT-4 post-v1.0".
2. **HARDENING audited_findings** : entry 2026-04-20 S22 CLOSED
   (Phase F wrap-up) présente dans frontmatter
   `docs/security/HARDENING_ROADMAP.md` (ligne 24). Mention
   hors-sprint `9676bd9 agents_sudo integration` intégrée au
   résumé S22 ligne 24 (pas ligne séparée pour ne pas sur-segmenter
   le log, cohérent pattern S17 research-only).
3. **Research matrix** : `.planning/research/S23_to_S29_agents
   _sudo_integration_matrix.md` (491 lignes) présente +
   mappings B1 S23 / A1 S24 / A3/B2/C2/C5/D5 S25 / A4/C1 S26 /
   C4/D2/D3 S28 / A2/B4 S29 documentés. Kickoff S23 devra arbitrer
   B1 timing (cf. §Pour Sprint 23 kickoff).

### Meta-track — LT-3 Contribution family Sybil matrix hors-sprint

**Audit check** :
1. `docs/release/ROADMAP_COMMITMENTS.md §LT-3` exists + trigger + runbook.
2. HARDENING §3 S23 amendement Couche 3 RFC implem preview + LT-3 decision boundary.
3. S31 stub réservé design-only.

**Verdict** : **PASS**.

1. **§LT-3** : `docs/release/ROADMAP_COMMITMENTS.md:148`
   section `## LT-3 Contribution family Sybil matrix` présente.
2. **HARDENING §3 S23** : amendé via `88eee23` chore hors-sprint
   (Couche 3 RFC implem preview + LT-3 reference).
3. **S31 stub** : design-only confirmé, pas ouvert. Research
   `.planning/research/S22_contribution_family_sybil_matrix.md`
   (archivé `dbc4ceb`) produit la matrice d'analyse.

---

## 3. Findings carry S23 (récapitulatif)

### P2 (non-blocking, Sprint 23 Phase A cleanup batch)

1. **P2-S22A-1** `dashmap` dep directe stale dans
   `crates/nexus-worker-core/Cargo.toml:163` post-refacto
   `RwLock<RateLimiterState>` Phase A. Grep confirmé 0 `use
   dashmap`. Action S23 : retirer la déclaration directe (governor
   tire dashmap transitivement) OU ajouter commentaire d'intention
   future. Trivial 1-line.
2. **P2-S22A-3** `docs/rust/PATTERNS.md §P33` header "Sprint 21
   Phase A" + struct snapshot `Arc<DashMap>` obsolète post-S22
   wire-up. Action S23 : update snippet struct → `RwLock<RateLimiter
   State>` + paragraphe « Post-S22 wire-up : worker-engine gate
   effectif + hot-reload Arc swap ».
3. **P2-B-1** End-to-end ONNX non exercé CI (jsdom stubs,
   modèle 45 MB non-loadable). Action S23 Track B : fixture
   model mini (< 10 MB) OU Playwright iframe end-to-end. Blocage
   infrastructure, non résoluble Phase F S22.
4. **P2-B-2** `web/src/sdk/pii/wrapper.ts:308-313` commentaire
   L309 « Scaffold path returns empty » obsolète post-Phase B
   (scaffold remplacé). Comportement (fallback sur empty) =
   defense-in-depth acceptable, mais sémantique explicite requise.
   Action S23 : update commentaire + décision `model_result_trusted`
   flag dans `PiiPolicy` OU suppression fallback sur empty quand
   `use_model === true && ready`.
5. **P2-E-1** `canary_input.py:508` `_reload_policy_locked`
   appelé depuis `__init__` sans lock (single-threaded safe,
   suffix `_locked` trompeur). Action S23 : commentaire inline
   `# safe: single-threaded init, lock not yet needed` OU rename
   `_reload_policy_inner` + wrapper public.
6. **P2-E-2 / meta-LOC-pattern** Pattern LOC estimation prospective
   observé **3 fois S22** (P3-S22A-1 advisory Phase A + Phase D
   643 vs ~250 plan §7.2 + Phase E 520 vs ~250 plan §8.2). Action
   S23 chore planning : amendement `docs/claude/README.md §6.7`
   clarifiant « LOC = borne scoping admise si documentée, pas
   métrique succès ». Carry fort.
7. **P2-Meta-hook-coverage-1** `.claude/.bypass_audit_trail.log`
   créé Phase F avec entries vide, mais Phase D iteration 2 a
   utilisé `NEXUS_SKIP_PHASE_AUDITOR=1` antérieurement sans
   append. Action S23 : soit retrospective append rétroactive
   cosmétique, soit clarifier README § Policy « forward-only
   depuis création Phase F S22 ». Non-bloquant.
8. **P2-D-1 (optionnel carry)** Convention body commit citation
   numéros ligne docs référence externes à éviter (ex: `THREAT_
   MODEL §7 ligne 85` fix D). Carry S23 possible README §6
   convention « cite section + sous-section, pas numéro ligne doc
   externe ».

### P3 (cosmétique, carry ou accept as-is)

1. **P3-S22A-1** LOC estimations plans S22 (absorbé par P2-E-2
   meta-LOC-pattern).
2. **P3-C-1 DOMAIN re-export** `DOMAIN_PROVENANCE_V1` +
   `DOMAIN_WARRANT_CANARY_V1` non re-exportés
   `crates/nexus-core-rs/src/lib.rs:63-66`. Action S23 : ajouter
   re-export OU justifier exclusion (usage interne crates uniquement).
3. **P3-E-1** `/api/canary/observed-divergence` expose
   `expected_answer` en clair. Carry S23 design alerting B1
   Guardrails.
4. **P3-E-2** CLI terminologie preflight `canary-rotate` vs code
   `canary rotate`. Accept as-is post-migration archive (footnote
   coût > bénéfice).

### Meta-carries (pas P-numbered)

1. Playwright PII end-to-end fixture model mini (carry S23
   Track B, absorbé P2-B-1).
2. Couche 3 RFC implem S23-S27 (multi-forge cross-validate +
   trust-web Amnesty S27, séquencée).
3. B1 Guardrails refactor pipeline déclaratif S23 (agents_sudo
   absorption — kickoff S23 doit arbitrer timing dédié/distribué/
   défer).

---

## 4. Recommandation

**Sprint 23 Phase A non-bloqué**. Aucun `fix(sprint22): ...`
requis (0 P0 + 0 P1).

**Carries G7 cap S22 → S23** :
- **Slot 1** : T-NN+2 iframe Rust-wasm Option G (PATTERNS §P34,
  hors cap formel).
- **Slot 2** : **LIBRE** ou consommé par audit findings batch
  Phase A cleanup S23 (P2-S22A-1 + P2-S22A-3 + P2-B-2 + P2-E-1
  triviaux bundled = 1 slot si batch PHASE-A dédié "cleanup
  S22 P2 batch", 0 slot si absorbés dans les phases métier S23).
- **LT-2 Radicle-v1.0** : sorti cap G7 (régularisé S22), trigger
  unique tag v1.0. Aucune action S23.

**Livrables pour Sprint 23 kickoff** (pattern §Pour Sprint 23
kickoff audit_plan) :
- Confirmer LT-2 `[deferred]` (aucun re-carry G7).
- Confirmer LT-3 design-only S31 (aucun re-carry).
- Confirmer LT-4 post-v1.0 (aucun re-carry).
- **Scope S23** : B1 Guardrails refactor pipeline déclaratif
  (agents_sudo absorption) + Couche 3 multi-forge cross-validate
  preview + ephemeral workers + escalating PoW + honeypot +
  redundancy voting 3-worker (carry S22 co-deferrer) + P2-B-1/P2-
  B-2 fixes ONNX end-to-end + carries P2 cleanup (dashmap + §P33
  + LOC estimations README §6.7 amend + `_reload_policy_locked`
  naming + bypass log policy).
- G1 Design Review Board agent Explore indépendant obligatoire
  (pattern §6.1.1) pour scoring D1..D5 S23.
- G2 trigger revalidate scan HARDENING_ROADMAP frontmatter +
  `openai-agents-python release > 0.7.0` avant gel D1..D5 S23.

**Commit stack du gate S22** :

```
<audit-gate-HEAD> chore(sprint22): audit gate S22 — findings (verdict PASS, no blocking fix) + Phase F review
f65914e  chore(sprint22): Phase F — wrap-up + verification + audit plan S23 + process fixes (P2-S21-4 + P2-S21-5) + migrate planning
```

---

**Verdict final** : **PASS**. Sprint 22 CLOSED audit gate levé.
Session S23 peut ouvrir kickoff.
