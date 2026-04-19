---
sprint: 21
phase: A
date: 2026-04-19
head_pre_commit: b4bda81
skill: nexus-phase-review (couche 2 TOOLING.md §4.2)
---

# Phase Review — Sprint 21 Phase A (rate-limit GCRA multi-tier R1)

## Verdict : PASS

Rigor signal G4 : **2 findings P2 documentés** + 1 finding P3
cosmétique. ≥ 1 P2+ requis pour PASS rigoureux atteint. P2 findings
carry-over explicite vers `sprint22_audit_plan` à émettre Phase F.

## Working tree audit (Step 1bis)

8 fichiers trackés/untracked, tous **PHASE**. Aucun CRAFT / DEBT /
NOISE.

| Fichier | Catégorie | Rationale |
|---|---|---|
| `Cargo.toml` (M) | PHASE | Ajout `governor = "0.10.2"` + `nonzero_ext = "0.3"` workspace deps + update commentaire axum 0.8 (post bump `5e67ce0`) |
| `Cargo.lock` (M) | PHASE | Transitifs auto-résolus (`governor v0.10.4` + `nonzero_ext v0.3.0` + deps indirects) |
| `crates/nexus-worker-core/Cargo.toml` (M) | PHASE | Deps `governor` + `nonzero_ext` + `dashmap` (workspace inherits) |
| `crates/nexus-worker-core/src/lib.rs` (M) | PHASE | `pub mod rate_limit` + `pub mod rate_limit_policy_loader` exports |
| `docs/rust/PATTERNS.md` (M) | PHASE | Ajout §P33 rate-limit GCRA multi-tier worker-engine gate (R1 scope-cut record + policy schema + engine integration outline) |
| `crates/nexus-worker-core/configs/rate_limit_policy.toml.sample` (??) | PHASE | Template opérateur avec schema inline + threat model coverage HARDENING_ROADMAP §3 S21 |
| `crates/nexus-worker-core/src/rate_limit.rs` (??) | PHASE | Primitive `RateLimiter` + `RateKey` + `RateLimitPolicy` + 9 unit tests |
| `crates/nexus-worker-core/src/rate_limit_policy_loader.rs` (??) | PHASE | Watcher hot-reload pattern S20 `pow_policy_loader.rs` + 7 unit tests |

Aucun split `chore(planning|skill|debt)` requis — toutes les modifs
appartiennent à Phase A scope.

## Suites §7.4 (tous verts post-Phase A)

- **Rust workspace nextest** : 642 → **658 (+16) ✅** (baseline S20
  tip `5e67ce0`, delta = 9 tests `rate_limit.rs` + 7 tests
  `rate_limit_policy_loader.rs`)
- **Rust doctests** : 6 → 6 (+0) ✅ (Phase A modules n'ajoutent pas
  de `# Examples` doc runnables)
- **Cargo clippy** `--workspace --all-targets --locked -- -D
  warnings` : **0 warning ✅**
- **Cargo fmt** `--all --check` : **0 diff ✅** (rustfmt a ajusté
  2 blocs `spawn_with_initial` + imports après Edit initial)
- **Python suites** : non touchées Phase A, skip (cohérent §7.4
  conditional)
- **Frontend suites** : non touchées Phase A, skip

Delta total Phase A : **+16 tests Rust** (vs +10 plan révision R1 —
dépasse la cible, documenté body commit).

## Commit body validation

- **Format titre** : `feat(sprint21): Phase A — rate-limit sliding-
  window multi-tier per-(consumer, worker, model) via governor GCRA
  worker-engine gate R1` — matche regex `(feat|fix|docs|chore|test)\
  (sprint\d+\):` + "Phase A" ✅
- **Section Working tree audit** présente : ✅ (voir tableau
  ci-dessus replicé dans body)
- **Delta tests cohérent** : ✅ body annonce +16 Rust, suite confirme
  642 → 658
- **Scope cuts honoured** : ✅ HTTP middleware /task/submit déféré
  S22+ mentionné + scope-cut R1 tracé (cf. `b4bda81`) + scope cuts
  kickoff §8 tous respectés
- **Co-Authored-By** : ✅ présent

## Step 4bis — Research grounding via context7

Kickoff §D1 « Sources context7 + WebSearch consultées (pré-gel
D1..D5) » + `.planning/research/S21_research_*` (4 docs archivés
commit `f5ad2e1`) documentent :

| Dep / API | Source research | Date |
|---|---|---|
| `governor 0.10.2` | context7 `/boinkor-net/governor` | 2026-04-18 (kickoff) + 2026-04-19 (G8 S1 re-check) |
| `tower-governor 0.8` | context7 `/benwis/tower-governor` + WebFetch GitHub Cargo.toml | 2026-04-18 + 2026-04-19 (G8 S1) |
| `nonzero_ext 0.3` | kickoff §D1 (mention implicite via `nonzero!(..)` macro governor API) | 2026-04-18 |
| `dashmap 6` | déjà workspace (S7 Phase A curator registry) | pré-existant |
| axum 0.8 | WebSearch `tokio.rs/blog/2025-01-01-announcing-axum-0-8-0` + CHANGELOG | 2026-04-18 (G8 S1) + 2026-04-19 (chore bump `5e67ce0`) |
| RUSTSEC scan | WebSearch 2026-04-18 | aucun advisory actif sur `governor` |

Verdict Step 4bis : **PASS**. Chaque dep/API touché a une trace
research documentée via kickoff §D1 ou commits G8/bump précédents
(`60adceb` + `5e67ce0`).

## Step 4ter — Horizon long-terme + documentation amont

| Check | Verdict | Rationale |
|---|---|---|
| Design doc présent pour nouveaux modules | ✅ | Kickoff §D1 Implications code + plan §4 servent de design doc (pattern S19/S20 Phase A cohérent — pas de doc séparé `.planning/research/` systématique quand kickoff §Dx suffit). PATTERNS.md §P33 documente le pattern pour sprints futurs. |
| D1..D5 citent alternatives rejetées + rationale | ✅ | Kickoff §D1 Rejetés liste 6 alternatives avec rationale par item (`ratelimit_meter` archived, `leaky-bucket` non-keyed, `async-rate-limiter` non-keyed, `tokio-rate-limit 0.8` sans GCRA, custom lru+atomic reinvente, global daemon-wide défaite §4 fairness). |
| Solution la plus poussée (pas courte-vue) | ✅ | `governor 0.10` est la crate Rust 2026 mature canonical avec GCRA + keyed natif + battle-tested (FedHubs / tide-governor écosystème). Alternatives rejetées factuellement sans écart. |
| Aucune LOC estimée au plan | ✅ | `grep -En 'LOC estim\|~\s*[0-9]+\s*LOC\|estim.*LOC' .planning/active/sprint21_*.md` → seules mentions sont retrospectives ("30-50 LOC custom middleware" dans pivot_proposal §3 Option A, « ~10 LOC modifiées sur 5 sites » dans commit `5e67ce0` body — contextes post-hoc légitimes, pas estimation plan). |

Verdict Step 4ter : **PASS**.

## Step 5 — Scope cuts verification

Kickoff §8 scope cuts :

```
- Kudos-weighted gossip admission → S22
- Sandbox tool-calling allow-list strict → S22
- Redundancy voting `Task.redundancy_factor` → S22
- Ephemeral workers + VRAM wipe → S23
- Honeypot Eclipse detection → S23
- Re-run sampling + DNS fallback DHT → S24
- Arti Tor bridge integration → S25
- Domain fronting Snowflake-WebRTC → S25
- PQC migration ML-DSA + ML-KEM → S26+
- Hardware keystore TPM/SE/StrongBox → S22+
- HPKE envelope peer-restore → S22+
- Rust-wasm iframe PII SDK realignement → S22+
- LLM-based semantic output detection → S23+
- ProxyPrompt proactive defense → S23+
- spaCy NER wasm → DEPRECATED
- actions/checkout@v4 pin SHA sweep → sprint ops futur
```

Plus scope-cut R1 post-drift 2026-04-19 :
- HTTP middleware `/task/submit` FastAPI Python → S22+ sprint API
  sécurité dédié

Grep diff Phase A sur chaque item :
- Aucun fichier touché ne mentionne ou n'implémente de kudos-
  weighted, tool-calling allow-list, redundancy voting, etc.
- `git diff HEAD~3 HEAD` montre uniquement rate_limit primitive +
  loader + deps + PATTERNS.md + plan/kickoff updates.

Verdict Step 5 : **PASS**, 0 scope cut touché.

## Findings (rigor signal G4 — ≥ 1 P2+ requis)

### P2-A-1 : Primitive `RateLimiter` non câblée dans engine loop

**Description** : `crates/nexus-worker-core/src/rate_limit.rs` ship
la primitive `RateLimiter::check()` mais `engine/runtime.rs::scan_
and_execute_tasks` (L688) ne l'appelle pas. La primitive existe
mais n'est pas consommée — les menaces `HARDENING_ROADMAP §3 S21
C-ModelExtract` + `C-DosFlood` restent non-mitigées jusqu'à
wire-up.

**Localisation** : `crates/nexus-worker-core/src/engine/runtime.rs`
L688 (scan_and_execute_tasks) + `crates/nexus-worker-core/src/
engine/runtime.rs::EngineBoot::new` L107 (construction path où
`rate_limiter: Option<RateLimiter>` devrait être injecté).

**Rationale scope-plan** : Plan §4.1 liste primitive + loader +
sample + deps + tests comme deliverables Phase A — pas d'engine
wire-up explicitement. PATTERNS.md §P33 documente l'engine
integration outline en commentaire prospectif "Phase A ships the
primitive + loader + tests only ; no engine call-site yet". Scope-
plan tenu littéralement.

**Impact threat model** : Zero mitigation effective des menaces
S21 jusqu'à wire-up. Le primitive est "inert scaffolding" tant
qu'aucun call-site ne `check()` sur le path admission.

**Carry-over** : Créer Phase A.1 ou carry Phase B/C engine
wire-up : injecter `RateLimiter` via `EngineBoot::rate_limiter :
Option<RateLimiter>` + appel `rate_limiter.check(&key)?` dans
`scan_and_execute_tasks` avant `claim_task` sur le path admission
downstream. ~40-80 LOC touchées + 1-2 tests intégration (reject
→ defer). **À tracer dans `sprint21_audit_plan.md §Track-X`
Phase F + re-carry S22 audit_plan si non livré S21.**

### P2-A-2 : Hot-reload policy ne re-construit pas les governor limiters

**Description** : `RateLimiter` charge la policy initiale via
`from_policy` puis construit `DefaultKeyedRateLimiter` *une fois*.
Un `RateLimitPolicyWatcher::spawn` update l'`Arc<RwLock<RateLimit
Policy>>` sur write détecté, mais les governor limiters internes
restent immutables — changer `default.per_min` au runtime n'a
*aucun effet* sur le gate enforced. L'ajout d'une nouvelle
`overrides.consumer[].pubkey_hex` au runtime n'est pas non plus
appliqué automatiquement.

**Localisation** : `crates/nexus-worker-core/src/rate_limit.rs`
L170-175 (commentaire `_policy` field honest : "subsequent reloads
update the backing `Arc<RwLock<RateLimitPolicy>>` but do **not**
rebuild the governor state automatically") + L182-188 (`from_
policy` construit le snapshot initial puis délègue à `from_
snapshot`, pas de `refresh` subsequent).

**Impact** : Opérateur qui édite `~/.sbfb/rate_limit_policy.toml`
pendant que le worker tourne voit son edit parsé + stocké dans
l'Arc mais *non appliqué*. Effectif uniquement après restart
worker. Contredit l'intention "hot-reload" annoncée kickoff §D1
+ PATTERNS §P33.

**Rationale scope-plan** : Limitation connue documentée inline
code (doc comment `_policy`). Le tests passent parce qu'ils
vérifient `Watcher::current()` reflète la policy parsée, pas que
`RateLimiter::check()` utilise la nouvelle policy. Le gap est
entre "policy snapshot à jour" (OK) et "limiters rebuilt" (non
couvert Phase A).

**Carry-over** : Ajouter `RateLimiter::refresh_from_policy()` qui
reconstruit les `DefaultKeyedRateLimiter` à partir du snapshot
courant + appelé par le watcher sur reload event. ~20-40 LOC +
1 test intégration (reload policy → check avec nouveau budget
appliqué). **Carry-over S22 audit_plan OU Phase A.2 S21.**

### P3-A-1 : `burst_multiplier < 1.0` silent floor sans warn log

**Description** : `make_quota` floor le burst à `per_min` quand
`burst_multiplier < 1.0` pour éviter rejet `governor` (burst ≥
rate). L'opérateur qui écrit `burst_multiplier = 0.5` avec
`per_min = 10` pense avoir un burst de 5, reçoit effectivement
burst = 10.

**Localisation** : `crates/nexus-worker-core/src/rate_limit.rs`
L289-292 (`make_quota` function).

**Impact** : Comportement surprenant silencieux. Test
`burst_multiplier_below_one_floors_to_per_min` documente mais
n'alerte pas l'opérateur.

**Carry-over** : Ajouter `tracing::warn!` au path floor trigger
dans `make_quota`. Cosmétique mais améliore ops debuggability.
Optionnel Phase B ou ignoré si pas de report ops.

## Recommendation

**Ready to commit** : **OUI**.

**Carry-overs `sprint22_audit_plan.md` (à émettre Phase F)** :

- **Track-P2-A-1** (ex-`engine wire-up`) : RateLimiter non câblé
  engine loop. Dimension : "feature completeness vs threat model
  deliverable". Owner : Phase A.1 S21 (si capacité disponible)
  OU S22 rate-limit integration phase dédiée. Blocking factor :
  la mitigation `C-ModelExtract + C-DosFlood` annoncée
  HARDENING_ROADMAP §3 S21 n'est effective qu'après wire-up.
- **Track-P2-A-2** : Hot-reload policy incomplete (governor
  limiters immutables). Dimension : "honesty de l'interface vs
  implémentation". Owner : même sprint que A-1 (naturellement
  couplé — refresh_from_policy appelle from_snapshot).
- **Track-P3-A-1** : `burst_multiplier < 1.0` silent floor. Ops
  debuggability. Optionnel.

**Corrections needed** : **Aucune bloquante**. P2 findings sont
documentation-of-gap, pas des bugs à fixer Phase A.

## Timeline Phase A (traçabilité)

- `60adceb` 2026-04-18 : chore planning G8 pivot DESIGN-CONFLICT →
  Option C arbitré (axum bump workspace).
- `5e67ce0` 2026-04-18 : chore hors-sprint bump axum 0.7 → 0.8
  workspace-wide (5 sites, 642 tests verts post-bump).
- `b4bda81` 2026-04-19 : chore planning R1 scope-cut mid-phase drift
  detected (`/task/submit` Python, pas Rust).
- **Ce commit (à créer)** : feat sprint21 Phase A — primitive
  worker-engine gate + policy loader + 16 tests.
- Phase F S21 : `sprint21_audit_plan.md` track P2 findings pour
  S22 audit gate rigor signal.

---

**Fin review Phase A**. 0 P0 + 0 P1 + 2 P2 + 1 P3 — PASS
rigoureux. Commit autorisé.
