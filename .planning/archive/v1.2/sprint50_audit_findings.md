# Sprint 50 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S50).
**Tip d'entree** : `493fd73` (Phase C, HEAD master).
**Documents source** : `sprint51_audit_plan.md` (6 tracks) +
`sprint50_kickoff.md` (D1..D4) + `sprint50_plan.md` +
`sprint50_verification.md` (22/22).

---

## Verdict : PASS (0 P0, 0 P1, 1 P2, 3 P3)

Rigor signal G4 : 1 P2+ documente (>=1 requis pour PASS). Toutes
les 6 dimensions explorees avec evidence inline citee.

---

## Track A — Dispatch JoinHandle (Phase A)

- [x] A-1 : `DaemonRuntime` a le champ `dispatch_handle:
  Option<JoinHandle<()>>` — `runtime.rs:196`
- [x] A-2 : `start()` stocke le handle via
  `tokio::spawn(crate::dispatch_loop::run(...))` — `runtime.rs:521-528`,
  assigne `dispatch_handle: Some(dispatch_handle)` — `runtime.rs:670`
- [x] A-3 : `shutdown()` join le handle apres le HTTP serve join
  (L741-743 HTTP, L745-749 dispatch) — `runtime.rs:745`:
  `if let Some(mut handle) = self.dispatch_handle.take()`

## Track B — CLI handler integration tests (Phase A)

- [x] B-1 : 4 tests dans `handler_tests` module de `main.rs:718` :
  `init_creates_db` (L727), `invite_create_list_revoke_cycle` (L738),
  `quarantine_list_empty` (L760), `capability_enable_disable_cycle` (L770)
- [x] B-2 : tous utilisent `tempdir()` + `CoordinatorDb::open()` reelle
  (pas de mock) — `main.rs:728,739,761,771`
- [x] B-3 : 4/4 passent dans nextest — `4 passed, 230 skipped`

## Track C — Python deletion (Phase B)

- [x] C-1 : `git ls-files` retourne 0 fichier pour les 4 directories.
  Les dossiers existent physiquement (artefacts build non-trackes :
  `build/`, `dist/`, `src/`, `tests/`) mais 0 fichier git-tracked.
  Suppression git correcte.
- [x] C-2 : `Cargo.toml` n'a plus `nexus-core-py` dans members ni
  `pyo3` dans workspace.dependencies — grep 0 match.
- [x] C-3 : `pyproject.toml` workspace members = `["examples/hello-world-app"]`
  seulement. 0 reference `packages/*` ou `crates/nexus-core-py` dans
  la config fonctionnelle. Note : lignes 17-19 contiennent un commentaire
  stale mentionnant nexus-core-py (P3, cf. findings).
- [~] C-4 : verification.md confirme `cargo build --workspace --locked`
  OK. Non re-joue par l'auditeur (build implicite par nextest B-3).

## Track D — Frontend cleanup (Phase B)

- [x] D-1 : `useAppEvents.ts` ABSENT, `AppTabPage.tsx` ABSENT,
  `cross_lang.test.ts` ABSENT, `schema_v2_cross_lang.test.ts` ABSENT.
- [x] D-2 : App.tsx n'a plus la route `/app/:appName/tabs/:tabName` —
  grep 0 match.
- [x] D-3 : `.size-limit.json` n'a plus l'entry `TabViewRenderer` —
  grep 0 match.
- [x] D-4 : verification.md confirme tsc 0 error, Vitest 250, build OK,
  size 6/6.

## Track E — Process / meta

- [x] E-1 : G8 preflights 2/2 presents — `sprint50_phase_A_preflight.md`
  verdict EXECUTE, `sprint50_phase_B_preflight.md` verdict EXECUTE.
- [x] E-2 : scope cuts 8/8 respectes — diff grep 0 leak (SSE, MCP,
  app-gov, stake, LIMIT/OFFSET, mk_state).
- [x] E-3 : phase reviews 3/3 presents (A PASS, B PASS, C PASS).
- [x] E-4 : delta tests cumule coherent — Rust +4, Vitest -17,
  Python -528 (195+287+46) = verification.md §2 confirme.
- [x] E-5 : sprint pair — phase dette obligatoire Phase A (D1..D4,
  §6.2.1 Regle 1) — confirme.

## Track F — Doc coherence

- [x] F-1 : CLAUDE.md — 0 reference fonctionnelle Python stale.
  "Python/Pyodide" (L7, L36) = techno app supportee par la plateforme.
  "Rust+Frontend pur depuis S50" (L116) = etat actuel correct.
  "Option G hybride Rust+Python" (L160) = decision Day 0 historique
  gelee. `serde(default)` "client Python" (L193) = exemple valide.
  Compteurs : 1199 Rust / 250 Vitest / 42+2f PW / 6/6 size / ~1455
  total — coherent.
- [x] F-2 : SPRINT_LOG.md row S50 presente (L19) avec detail complet.
- [x] F-3 : memory `nexus_grid_pivot.md` tip `493fd73` = HEAD master.

---

## Findings

### P2-AUDIT-A-1-S50 — uv.lock stale (plan item non execute)

Le kickoff §5 Phase B prescrit : "Supprimer les references Python
dans `uv.lock` (regenerer ou supprimer)." Le fichier `uv.lock` contient
encore les 4 packages supprimes :
- `nexus-app-gov` (L8, L769-784)
- `nexus-coordinator` (L9, L791-835)
- `nexus-core-py` (L10, L855-868)
- `nexus-sdk` (L12, L871-873+)

Impact fonctionnel : **nul** — `uv` ignore les packages absents du
workspace. Le `pyproject.toml` workspace members est propre (seulement
`examples/hello-world-app`). Mais c'est un plan item explicite qui n'a
pas ete execute, ni scope-cut dans §7.

**Fix recommande** : `uv lock` pour regenerer le lockfile, ou supprimer
`uv.lock` si plus aucun workflow Python n'est prevu.

**Carry S51** : P2-AUDIT-A-1-S50 uv.lock stale 1/3.

### P3-AUDIT-A-2-S50 — pyproject.toml commentaire stale

Lignes 17-19 du `pyproject.toml` contiennent un commentaire historique :
```
# crates/nexus-core-py ships a Python wheel built via maturin (see
# crates/nexus-core-py/pyproject.toml), providing the `nexus_core`
# module that wraps iroh/docs/gossip/blobs from nexus-core-rs.
```
Ce crate n'existe plus. Commentaire stale sans impact fonctionnel.

### P3-AUDIT-A-3-S50 — directories untracked sur disque

Les 4 directories supprimees existent encore physiquement avec des
artefacts build (`build/`, `dist/`, `src/`, `tests/`) non git-trackes.
`git status` est propre (0 untracked, ces dirs sont ignorees par
`.gitignore` ou equivalents). Nettoyage cosmétique possible via
`git clean -fd packages/ crates/nexus-core-py/`.

### P3-AUDIT-A-4-S50 — uv.lock fonctionnellement inutile

Avec la suppression des 4 packages Python et le seul workspace member
restant (`examples/hello-world-app`, app SBFB HTML), la question se
pose de savoir si `uv.lock` + `pyproject.toml` racine restent
necessaires. Si `examples/hello-world-app` n'utilise pas `uv` pour
son build (c'est une archive HTML), ces fichiers deviennent vestigiaux.

---

## Carries S51

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 2/3 | S48 review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 2/3 | S48 review |
| P2-AUDIT-A-1-S48 carry doc accuracy | 2/3 | S48 audit |
| P2-REVIEW-A-1-S50 dispatch join order | 1/3 | S50 Phase A review |
| P2-REVIEW-B-1-S50 nexus/ legacy monolith | 1/3 | S50 Phase B review |
| P2-AUDIT-A-1-S50 uv.lock stale | 1/3 | NEW S50 audit |

**Total : 8 carries** (3 a 2/3 approchent seuil MANDATORY, 3 a 1/3,
2 exemptions). S51 impair — pas de phase dette obligatoire mais les
3 items a 2/3 deviennent 3/3 MANDATORY si non adresses S51.

---

## Verdict global

**PASS** — 0 P0, 0 P1, 1 P2 (uv.lock stale, non-bloquant), 3 P3.
Sprint 51 Phase A peut demarrer directement.
