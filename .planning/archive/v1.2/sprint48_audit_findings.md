# Sprint 48 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S48).
**Tip d'entree** : `3591cf2` (HEAD). Tip audit plan : `672c287`.
**Documents consultes** : sprint49_audit_plan.md, sprint48_plan.md,
sprint48_kickoff.md, sprint48_verification.md,
sprint48_phase_A_review.md, sprint48_phase_B_review.md.

---

## Verdict : PASS (0 P0, 0 P1, 1 P2, 2 P3)

G4 rigor signal satisfait : 1 P2 documente (>=1 requis pour PASS).
0 P0 + 0 P1 → S49 Phase A demarre sans fix prealable.

---

## Track A — TOCTOU canary reload fix

- [x] A-1 : `canary_input.rs:509-518` — `reload_policy()` acquiert
  le verrou `reload` (l.509), check mtime (l.510), update mtime
  (l.513), `read_to_string()` (l.514), puis `drop(rs)` (l.518)
  APRES le read. Lock tenu pendant la lecture. ✅
- [x] A-2 : `canary_input.rs:536-545` — `reload_set()` acquiert
  le verrou (l.536), check mtime (l.537), update mtime (l.540),
  `load_canary_input_set()` (l.541 — lit ET parse), puis `drop(rs)`
  (l.545) APRES le load. Lock tenu pendant lecture + parse. ✅
- [x] A-3 : `canary_input.rs:487-498` — `maybe_reload()` structure
  inchangee : scope isole pour debounce (l.489-495, lock→check
  last_check→release), puis appels sequentiels `reload_policy()`
  (l.496) + `reload_set()` (l.497). Pas de deadlock (lock debounce
  libere avant reload). ✅

## Track B — kudos total_count

- [x] B-1 : `kudos_api.rs:60-61` — `total_count = all_entries.len()`
  capture AVANT `skip(offset).take(capped_limit)` (l.63-66). ✅
- [x] B-2 : `kudos_api.rs:80` — JSON repond
  `{"entries": ..., "count": count, "total_count": total_count}`. ✅
- [x] B-3 : `KudosTab.tsx:41` — affiche `query.data?.total_count`. ✅
- [x] B-4 : `coordinator.ts:217-220` — `KudosListSchema` inclut
  `total_count: z.number()`. ✅
- [x] B-5 : `http.rs:4482,4499` — test `kudos_entries_with_limit_
  offset` asserte `total_count == 3` pour les 2 requests. ✅

## Track C — execute_batch_raw feature gate

- [x] C-1 : `nexus-coordinator-rs/Cargo.toml:32` —
  `test-support = []`. ✅
- [x] C-2 : `db.rs:349` — `#[cfg(any(test, feature =
  "test-support"))]` sur `execute_batch_raw`. ✅
- [x] C-3 : `nexus-shell-daemon/Cargo.toml:132` — `[dev-dependencies]
  nexus-coordinator-rs = { ..., features = ["test-support"] }`.
  La dep normale (l.30) n'active PAS la feature. ✅
- [x] C-4 : `http.rs:4533` — test `diagnostic_fairness_returns_500_
  on_corrupted_db` existe. Verification confirms 1186 passed. ✅

## Track D — invite format test

- [x] D-1 : `http.rs:4190-4193` — test `invite_create_success`
  asserte `starts_with("inv-")`, `parts.len() == 4`,
  `parts[1].len() == 8`. ✅

## Track E — sbfb_home refactor

- [x] E-1 : `http.rs:146` — `DaemonHttpState` a `pub sbfb_home:
  Option<std::path::PathBuf>`. ✅
- [x] E-2 : `consent.rs:14-17` — `consent_path()` accepte
  `override_home: Option<&Path>`, fallback `sbfb_home()`.
  4 handlers (l.148, 166, 211, 230) utilisent
  `state.sbfb_home.as_deref()`. ✅
- [x] E-3 : `files.rs:22-34` — `files_dir()`, `blob_path()`,
  `manifest_path()` acceptent `override_home`. 4 handlers
  (l.89, 128, 150, 163) utilisent `state.sbfb_home.as_deref()`. ✅
- [x] E-4 : 0 appel `set_var("SBFB_HOME",...)` dans http.rs et
  consent.rs et files.rs. 4 restants dans `auth.rs:1073-1096`
  (carry S49 documente P2-REVIEW-B-1-S48). ✅
- [x] E-5 : `http.rs:1679` — `mk_state_with_sbfb_home()` existe.
  8 usages (7 tests migres depuis set_var + 1 nouveau test
  `files_dir_override_home` = le +1 Rust delta). ✅
- [x] E-6 : `runtime.rs:517` — `sbfb_home: None`. ✅

## Track F — Process / meta

- [x] F-1 : G8 preflights 2/2 presents — Phase A (`5939455`
  EXECUTE), Phase B (`c9bc0bf` EXECUTE). ✅
- [x] F-2 : Scope cuts 10/10 respectes — diff `3d14068..672c287`
  (31 files) ne touche aucun scope cut. ✅
- [x] F-3 : 7 carries resolus verifies dans verification §3 :
  2 CLOSED 2/3, 3 CLOSED 1/3, 1 RECLASSIFIE, 1 EXEMPTION. ✅
- [x] F-4 : Phase reviews 2/2 presents — Phase A PASS (1 P2,
  1 P3), Phase B PASS (1 P2, 1 P3). ✅
- [x] F-5 : Delta tests cumule Rust +1
  (`files_dir_override_home`). ✅
- [x] F-6 : Sprint pair — Phase A est phase dette obligatoire
  (§6.2.1 Regle 1). Kickoff §type confirme. ✅

## Track G — Doc coherence

- [x] G-1 : CLAUDE.md — mentionne S48 + compteurs corrects
  (~1937 total, 1186 Rust). ✅
- [x] G-2 : SPRINT_LOG.md — row S48 presente, detaillee, conforme
  au format des sprints precedents. ✅
- [x] G-3 : memory `nexus_grid_pivot.md` — tip `672c287`, content
  "S48 CLOSED", compteurs corrects. HEAD `3591cf2` = 2 commits
  chore/docs apres le tip (non-feat). Content accurate. ✅

---

## Findings

### P2 (1)

- **P2-AUDIT-A-1-S48** : documentation carry inaccuracy —
  Phase A review P2-REVIEW-A-1-S48 affirme que `reload_policy()`
  tient le mutex `reload` pendant `CanaryInputPolicy::from_toml()`.
  C'est faux : `canary_input.rs:518` fait `drop(rs)` AVANT le
  parse `from_toml()` a l.519. Le lock couvre mtime check + read,
  PAS le parse. En revanche, `reload_set()` tient le lock pendant
  `load_canary_input_set()` (read + parse, drop a l.545). La
  description du carry "canary reload size cap" est correcte pour
  reload_set mais surestime le scope pour reload_policy. S49
  devrait noter cette asymetrie quand l'item sera adresse —
  le size cap protege le read sous lock dans les deux fonctions,
  mais seul reload_set a le parse sous lock. Severite reduite
  pour reload_policy par rapport au carry documente. 1/3.

### P3 (2)

- **P3-AUDIT-A-1-S48** : memory tip SHA `672c287` est 2 commits
  derriere HEAD `3591cf2`. Les 2 commits sont chore/docs (Phase C
  wrap-up + migration planning). Le contenu de la memory est
  correct (S48 CLOSED, compteurs exacts). Mettre a jour le tip
  a la fin de cette session.

- **P3-AUDIT-A-2-S48** : asymetrie lock scope entre
  `reload_policy()` (read sous lock, parse hors lock) et
  `reload_set()` (read + parse sous lock). Fonctionnellement
  correct : un seul thread appelle `maybe_reload()` par cycle
  dispatch (D1 kickoff), le gate mtime empeche les re-lectures
  concurrentes. L'asymetrie est un choix de design implicite
  (parse TOML hors lock = contention reduite), pas un bug.

---

## Carries S49 (confirmes)

| Item | Compteur | Source | Note audit |
|---|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe | inchange |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 | inchange |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | NEW Phase A review | scope policy surévalué (P2-AUDIT-A-1-S48) |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | NEW Phase B review | 4 set_var auth.rs confirmes |
| P2-AUDIT-A-1-S48 carry doc accuracy | 1/3 | NEW this audit | asymetrie lock reload_policy documenter |

---

## S49 impair

S49 est impair → pas de phase dette obligatoire (§6.2.1 Regle 1).
0 item a 2/3. 0 item a 3/3.

---

## Tracabilite audit plan → findings

| Track | Checks | Result | Findings |
|---|---|---|---|
| A TOCTOU canary | 3/3 | ✅ | P2-AUDIT-A-1 (doc accuracy) + P3-AUDIT-A-2 (asymetrie) |
| B kudos total_count | 5/5 | ✅ | — |
| C execute_batch_raw | 4/4 | ✅ | — |
| D invite format | 1/1 | ✅ | — |
| E sbfb_home | 6/6 | ✅ | — |
| F process / meta | 6/6 | ✅ | — |
| G doc coherence | 3/3 | ✅ | P3-AUDIT-A-1 (tip SHA stale) |
