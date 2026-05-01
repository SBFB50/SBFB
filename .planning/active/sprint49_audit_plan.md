# Sprint 49 — Audit plan (Sprint 48 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S48).
**Tip d'entree** : `672c287` (S48 Phase B, dernier feat commit).
**Documents source** : `sprint48_kickoff.md` (D1..D4) +
`sprint48_plan.md` (§Phase A, §Phase B) +
`sprint48_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint48_plan.md,
(3) sprint48_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint48_audit_findings.md`.

## Track A — TOCTOU canary reload fix (Phase A)

- [ ] A-1 : verifier que `canary_input.rs` reload_policy() garde
  le verrou `reload` pendant `read_to_string()`. Le `drop(rs)`
  doit etre APRES le read, pas avant.
- [ ] A-2 : meme verification pour reload_set() — lock tenu
  pendant `load_canary_input_set()`.
- [ ] A-3 : verifier que `maybe_reload()` n'a pas change de
  structure (debounce + sequential reload_policy + reload_set).

## Track B — kudos total_count (Phase A)

- [ ] B-1 : verifier `kudos_api.rs` — `total_count` capture
  AVANT skip/take, `count` capture APRES.
- [ ] B-2 : verifier JSON repond `{entries, count, total_count}`.
- [ ] B-3 : verifier `KudosTab.tsx` affiche `total_count`.
- [ ] B-4 : verifier `KudosListSchema` Zod dans `coordinator.ts`
  inclut `total_count: z.number()`.
- [ ] B-5 : verifier test `kudos_entries_with_limit_offset`
  asserte `total_count=3`.

## Track C — execute_batch_raw feature gate (Phase B)

- [ ] C-1 : verifier `nexus-coordinator-rs/Cargo.toml` a
  `[features] test-support = []`.
- [ ] C-2 : verifier `db.rs` `execute_batch_raw` a
  `#[cfg(any(test, feature = "test-support"))]`.
- [ ] C-3 : verifier `nexus-shell-daemon/Cargo.toml`
  [dev-dependencies] inclut `nexus-coordinator-rs = { ...,
  features = ["test-support"] }`.
- [ ] C-4 : verifier que le test `diagnostic_fairness_returns_
  500_on_corrupted_db` compile et passe.

## Track D — invite format test (Phase B)

- [ ] D-1 : verifier test `invite_create_success` asserte
  `starts_with("inv-")`, 4 parts, parts[1].len() == 8.

## Track E — sbfb_home refactor (Phase B)

- [ ] E-1 : verifier `DaemonHttpState` a champ `sbfb_home:
  Option<PathBuf>`.
- [ ] E-2 : verifier `consent.rs` consent_path/load_consent/
  save_consent acceptent `override_home: Option<&Path>` et les
  4 handlers utilisent `state.sbfb_home.as_deref()`.
- [ ] E-3 : verifier `files.rs` files_dir/blob_path/manifest_path
  acceptent `override_home` et les 3 handlers utilisent
  `state.sbfb_home.as_deref()`.
- [ ] E-4 : verifier 0 appel `std::env::set_var("SBFB_HOME",...)`
  restant dans les tests consent/files (7 elimines).
- [ ] E-5 : verifier `mk_state_with_sbfb_home()` existe et les
  7 tests l'utilisent.
- [ ] E-6 : verifier `runtime.rs` a `sbfb_home: None`.

## Track F — Process / meta

- [ ] F-1 : G8 preflights 2/2 presents (A + B tous EXECUTE).
- [ ] F-2 : scope cuts 10/10 respectes (diff --stat).
- [ ] F-3 : 7 carries resolus verifies dans le diff.
- [ ] F-4 : Phase reviews 2/2 presents (A + B).
- [ ] F-5 : Delta tests cumule coherent : Rust +1.
- [ ] F-6 : Sprint pair — phase dette Phase A confirmee.

## Track G — Doc coherence

- [ ] G-1 : CLAUDE.md etat actuel — verifier S48 + compteurs.
- [ ] G-2 : SPRINT_LOG.md row S48 — verifier presente.
- [ ] G-3 : memory nexus_grid_pivot.md — verifier tip mis a jour.

---

## Carries S49

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | NEW |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | NEW |

**Note S49 impair** : S49 est impair → pas de phase dette
obligatoire (§6.2.1 Regle 1). 0 item a 2/3 ou 3/3.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S49 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint48): ... avant S49 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S48 (decision sprint, pas audit)
- auth.rs set_var (carry S49, scope Phase B documente)
