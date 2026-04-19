# Sprint 22 Phase A — nexus-phase-auditor review

HEAD pre-commit: `87b0891` (chore(planning): open Sprint 22)
Draft commit body: "feat(sprint22): Phase A — rate-limit engine wire-up + hot-reload + policy sample smoke"
Timebox: 45m

## Verdict : PASS

(0 P0 / 0 P1 / 3 P2 documentés — rigor signal G4 satisfait)

Les 3 P2 sont tous tracés ci-dessous. P2-S22A-1 et P2-S22A-2 étaient
pré-identifiés par l'exécuteur dans le draft commit body. P2-S22A-3
est un finding additionnel de l'auditeur (§P33 pattern drift). Aucun
P0/P1 détecté. Commit autorisé sous réserve d'entrée des 3 P2 dans
`sprint22_audit_plan.md` carry S23 (ou résolution in-phase F).

---

## Dimensions

### Security

- [x] **semgrep/grep scan** : 0 findings bloquants.
  - `grep -nE 'unwrap\(\)' runtime.rs` : tous les `unwrap()` dans le
    diff sont dans `#[cfg(test)]` (tests, build helpers, sign helpers)
    — aucun dans le code de production. Vérifié : `runtime.rs` L435-498
    (boot path) utilise `warn!` + fallback, jamais `unwrap`.
  - `grep -nE 'unsafe'` : 0 bloc `unsafe` ajouté.
  - `grep -nE '(AKIA|ghp_|pat_|sbfb_[a-z]+)' ` : 0 secret.
  - `grep -nE 'todo!|unimplemented!'` : 0 dans le diff.
- [x] **Fail-open gate** : quand `sbfb_home` non résolvable,
  `rate_limiter` boot sur `RateLimitPolicy::default()` (60 req/min,
  burst x2). Comportement documenté `runtime.rs:L486-498`. Cohérent
  avec le pattern consent S16 (`consent filter disabled` log).
  Acceptable : fallback sur une politique active (non silencing), le
  log `warn!` L487 informe l'opérateur.
- [x] **RateKey non-forgeable** : `hex::encode(task_entry.author_pubkey)`
  est construit APRÈS `task_entry.verify_signature()` (`runtime.rs:
  L875-878`). Un adversaire ne peut pas soumettre un `author_pubkey`
  arbitraire sans invalider la signature Ed25519.
- [x] **Loopback/wire/zip** : aucun nouveau listener loopback, aucun
  module wire format, aucun zip extract. N/A.
- [x] **JCS canonique** : `rate_limit.rs` ne sérialise rien sur le wire
  (état local uniquement, TOML sur disque). N/A.
- [x] **`InvalidQuota` arm** : `runtime.rs:L976-988` gère l'arm
  `Err(e)` (autre que Saturated) avec `warn!` + `continue` plutôt que
  crash. Correct : la politique par défaut a toujours des quotas
  valides, ce chemin ne se déclenche qu'en cas de bug de construction
  (impossible post-validation à boot).

### Patterns

- [x] **§P33 — struct layout** : le code `rate_limit.rs:L167-186`
  a migré vers `RwLock<RateLimiterState>` (état groupé = swap atomique)
  + `HashMap` interne pour les overrides. Rationale documenté en inline
  comment L161-178. Le pattern RwLock snapshot-and-release est correct
  pour hot-path read-heavy, writes rares (~1/h hot-reload).
- [x] **§P33 — callback hot-reload** : `spawn_with_on_reload` pattern
  (rate_limit_policy_loader.rs:L156-167) : callback appelé une fois
  au spawn (synchrone, L164) + à chaque reload. Cohérent avec §P28
  `PowPolicyWatcher` + §P18 `TokenRotator`.
- [x] **§P33 — parent-dir watch + debounce 50ms** : confirmé
  `rate_limit_policy_loader.rs:L188-231`. Identique au pattern existant.
- [x] **§P33 — malformed reload guard + deletion guard** :
  `rate_limit_policy_loader.rs:L213-219` (Remove → warn+keep) +
  L255-263 (parse error → warn+keep). Exact pattern.
- [x] **§P33 — RateKey tuple** : `(hex(author_pubkey), hex(worker_pubkey),
  task.model)` — `runtime.rs:L955-959`. Conforme plan §4.2 « tuple
  `(coordinator_that_signed, self, model)` ».
- [ ] **§P33 — struct shape stale** : voir P2-S22A-3 ci-dessous.
- [x] **§P34** : Phase A ne touche pas le canary. N/A.
- [x] **docs/shell/PATTERNS.md** : diff 100% Rust + main.rs propagation.
  N/A.

### Working tree audit (G5)

- [x] **PHASE** : 4 fichiers, tous listés dans `plan.md §4.2`
  (`runtime.rs`, `rate_limit.rs`, `rate_limit_policy_loader.rs`,
  `main.rs`). Comptage exact.
- [x] **CRAFT** : 0 fichier planning/docs dans le staging.
- [x] **DEBT** : 0 fichier scope-cut ou tech-debt hors phase.
- [x] **NOISE** : 0 fichier accidentel. `git status --short` =
  exactement les 4 fichiers attendus.
- [x] **Section "Working tree audit"** : draft commit body mentionne
  table fichiers + scope cuts. Conforme G5.

Note : `crates/nexus-worker-core/configs/rate_limit_policy.toml.sample`
est NOT dans le staging diff — il est déjà tracké depuis `63afe4e`
(S21 Phase A). Le plan §4.2 le liste comme « nouveau — fix P3-S21-4 »
mais ce fichier pré-existe. L'exécuteur a correctement identifié le
drift (P2-S22A-2).

### G8 traceability

- [x] Artefact G8 présent : `.planning/active/sprint22_phase_A_preflight.md`
  — verdict **EXECUTE plan-as-is** (2026-04-19, HEAD `87b0891`).
- [x] 4 scans S1-S4 documentés inline : S1 governor 0.10.2 context7 +
  WebSearch RUSTSEC clean / S2 décisions historiques traversées / S3
  threat model coverage / S4 wire format invariants.
- [x] Verdict EXECUTE : pas de pivot_proposal attendu. Pas de DESIGN-CONFLICT.
- [x] Exception Cas D hotfix : N/A (artefact présent).

### Scope-cuts

- [x] `redundancy voting` : 0 match dans le diff.
- [x] `traffic padding` : 0 match.
- [x] `sandbox tool-calling` / `tool.call` : 0 match.
- [x] `Radicle` / `radicle` : 0 match.
- [x] LT-2 / Meta-1 : 0 touch.
- [x] Cap G7 slots : aucun slot G7 consommé par Phase A (wire-up
  mécanique, pas nouvelle tech debt).
- [x] Items déférés S23 inchangés (confirmé par `git diff --cached
  | grep -iE "redundancy|traffic.pad|sandbox|radicle"` → vide).

### Tests-delta

- [x] **Rust nextest** : annoncé +7, réel **+7** (659 → 666). Vérifié
  `cargo nextest run --workspace --locked` : 666/666 passed, 0 skipped.
- [x] **Mapping noms vs plan §4.3** :
  1. `rate_limit_gate_rejects_saturated_tuple` ✓ (L1617)
  2. `rate_limit_gate_admits_fresh_tuple` ✓ (L1567)
  3. `rate_limit_gate_reloads_live_policy` ✓ (L1738)
  4. `rate_limit_gate_defer_preserves_task` ✓ (L1676)
  5. `rate_limit_policy_sample_loader_smoke` ✓ (L1805)
  6. `swap_preserves_unsaturated_tuples` ✓ (rate_limit_policy_loader L491)
  7. `swap_clears_saturated_state` ✓ (rate_limit_policy_loader L537)
- [x] **Python coord** : non modifié Phase A. N/A.
- [x] **Vitest** : non modifié Phase A. N/A.
- [x] **Playwright** : non modifié Phase A. N/A.
- [x] **Doctests** : `cargo test --workspace --locked --doc` : 0 passed,
  1 ignored (doctest `spawn_with_on_reload` L136 marqué `ignore` —
  code d'exemple incomplet avec `...` placeholder, convention Rust
  standard, rationale implicite dans le contexte). Pas de test skip
  sans reason= dans les tests unitaires/intégration.
- [x] **clippy + fmt** : 0 warning, 0 diff. Vérifié.

### Research-grounding

- [x] **Cargo.toml deps ajoutées/bumpées** : `git diff --cached --
  Cargo.toml Cargo.lock` → 0 nouvelle dep, 0 bump. `governor 0.10.2`
  déjà pré-existant workspace depuis S21 `5e67ce0`. `dashmap`,
  `notify`, `hex`, `tokio` : inchangés.
- [x] **Trace §Research consulté plan §3** : "Rate-limit wire-up Phase A"
  tracé (`governor 0.10.2` déjà intégré S21, `tower-governor 0.8`
  axum pattern, Arc swap hot-reload pattern). Clean.
- [x] **API crypto / spec standardisée** : aucune nouvelle API crypto.
  `RateLimiter::check` ne fait pas de crypto. GCRA = algo local.
- [x] **Aucune advisory governor 2026** : WebSearch RUSTSEC 2026 scanné
  dans preflight G8 S1. Clean.

### Horizon long-terme + documentation amont

- [x] **Design doc** : §P33 existait depuis S21 Phase A — trace écrite
  avant le code Phase A S22. Pas de nouveau module structurant.
  Phase A est une wire-up d'une primitive déjà documentée.
- [x] **Alternatives rejetées** : D2 scope γ kickoff §4 D1..D5 documentent
  les rejets (arc-swap vs RwLock justifié dans le draft body +
  rate_limit.rs:L173-178 inline). Alternative arc-swap explicitement
  documentée comme migration possible si profiling révèle contention.
- [x] **Solution la plus poussée** : `RwLock<RateLimiterState>` pour
  hot-swap atomique vs `DashMap` direct (S21 shape) = régression
  technique justifiée par la sémantique : le swap doit remplacer
  default + overrides **atomiquement** pour éviter l'état partiel.
  RwLock groupé est plus correct que 2 DashMaps swappés séparément.
- [ ] **Estimations LOC dans plan/kickoff** : plan §5-8 + kickoff
  contiennent des estimations LOC prospectives pour les phases
  futures B-F (~250 LOC, ~300 LOC, etc.) — voir P3-S22A-1.

---

## Findings

- **P2-S22A-1** : `dashmap = { workspace = true }` déclaré en dépendance
  directe dans `crates/nexus-worker-core/Cargo.toml` ligne 163, mais
  aucun `use dashmap` ni `dashmap::` dans le code source après refacto
  `RwLock<RateLimiterState>` avec `HashMap` interne. Dep stale.
  Rationale commentaire Cargo.toml L158-163 explique l'historique S21
  mais ne justifie plus la déclaration directe (governor tire dashmap
  transitivement). Fix : supprimer `dashmap` des deps directes de
  worker-core (ou conserver avec commentaire « governor backing store,
  re-declared to allow direct use in future » si une utilisation future
  est envisagée). Carry S23 ou Phase F.

- **P2-S22A-2** : `sprint21_verification.md` row 21 pointe
  `shell-daemon/configs/rate_limit_policy.toml.sample` (chemin
  incorrect) alors que le fichier existe en
  `crates/nexus-worker-core/configs/rate_limit_policy.toml.sample`
  depuis `63afe4e`. Le plan §4.2 S22 le liste à tort comme « nouveau
  — fix P3-S21-4 » alors qu'il était déjà livré. Trace de vérification
  S21 incohérente. Fix : corriger `sprint21_verification.md` dans un
  chore planning (ou Phase F chore). Carry S23.

- **P2-S22A-3** (finding additionnel auditeur) : `docs/rust/PATTERNS.md
  §P33` (`PATTERNS.md:L1837-1841`) décrit encore l'ancienne structure
  `RateLimiter` avec `Arc<DefaultKeyedRateLimiter>` + `Arc<DashMap<...>>`
  comme champs directs du struct. Le code après Phase A S22 a évolué
  vers `RwLock<RateLimiterState>` (groupant default + overrides sous
  un seul lock) + `HashMap<ConsumerId, ...>` pour les overrides
  (`rate_limit.rs:L167-186`). Pattern drift non mis à jour dans ce
  commit. Le plan §4.2 ne listait pas PATTERNS.md comme fichier
  modifié (scope hors-phase A), donc non-bloquant. Mais le doc pattern
  induit en erreur un futur lecteur sur la structure réelle. Fix :
  mettre à jour §P33 snippet struct + supprimer la référence DashMap
  dans Phase F chore ou commit séparé. Carry S23/Phase F.

- **P3-S22A-1** (advisory hors-phase) : `sprint22_plan.md §5-8` et
  `sprint22_kickoff.md` contiennent des estimations LOC prospectives
  pour les phases B-E (« ~250 LOC », « ~300 LOC », etc.) — contraire
  à `docs/claude/README.md §6.7` + `feedback_approach.md` qui proscrit
  les LOC estimées au plan. Phase A elle-même n'en a pas (§4 clean).
  Advisory pour les phases B-F : supprimer ou remplacer par une
  description fonctionnelle sans chiffre LOC. Non-bloquant pour Phase A.

- **P3-S22A-2** : `rate_limit_gate_reloads_live_policy` test
  (`runtime.rs:L1799-1802`) shutdown le node via `engine.node.shutdown()`
  directement sans passer par le channel `take_shutdown_sender()`.
  Intentionnel et documenté (L1799-1801 : « the test never injects a
  real task — the assertion is purely on the in-process rate_limiter
  state »). Pas de leak observable. Nit cosmétique pour uniformité avec
  les autres tests du module.

---

## Recommendation

**Commit autorisé.** 0 P0 / 0 P1.

Actions requises avant ou pendant Phase F :
1. Entrée P2-S22A-1 dans `sprint22_audit_plan.md` carry S23 (ou fix
   in-phase F : supprimer `dashmap` dep directe worker-core Cargo.toml).
2. Entrée P2-S22A-2 dans `sprint22_audit_plan.md` carry S23 (ou fix
   in-phase F : corriger `sprint21_verification.md` row 21).
3. Entrée P2-S22A-3 dans `sprint22_audit_plan.md` carry S23 (ou fix
   in-phase F : update §P33 PATTERNS.md struct snapshot).

P3-S22A-1 (LOC estimations plans B-F) : advisory pour les phases
suivantes, l'exécuteur peut supprimer opportunément lors des commits
plan des phases concernées.
