# Sprint 59 — Audit findings (Phase 0 gate S60)

**Date** : 2026-05-11
**Auditeur** : session fraiche (pas de contexte S59 d'implementation)
**Sprint audite** : Sprint 59
**Tip audite** : `8679dfc` (closing tip incluant 2 fixes post-Phase D)
**Audit plan** : `sprint60_audit_plan.md` (7 tracks A-G)

---

## Verdict : PASS

**0 P0, 0 P1, 1 P2, 5 P3.**

Le sprint S59 est proprement livre. Les 3 items fermes (LT-1,
STORAGE-JOIN-VALIDATE, STORAGE-ANTISPAM) sont correctement
implementes avec tests adequats. Le pre-launch protocol est
respecte (0 FORMAT_VERSION bumpe, 0 tolerant decoder, 0
serde(default) ajoute). Les D1-D4 figees sont fideles a
l'implementation. Les compteurs tests sont confirmes par
re-execution live : 1257 Rust / 258 Vitest.

---

## Tracks audites

| Track | Portee | Verdict | Findings |
|-------|--------|---------|----------|
| A — Kudos-v2 formula | credit(), effective_score(), verify_chain(), fairness, API | PASS | 1 P3 |
| B — Deploy E2E wiring | SBFB.json, tests deploy, Deploy.tsx, sync-bridge | PASS | 1 P3 |
| C — Launcher MessageBox | FFI, 5 error paths, cfg gate, windows_subsystem | PASS | 0 |
| D — Storage validation | is_replicated, StorageWriteLimiter, 429, tests | PASS | 1 P3 |
| E — Pre-launch protocol | FORMAT_VERSION, tolerant decoder, serde, wire, D1-D4 | PASS | 0 |
| F — Carries residuels | P2-A-1 rand, P2-AUDIT-2 iroh, 14/14 scope | PASS | 1 P3 advisory |
| G — Tests delta + meta | +17 Rust, +2 Vitest, PW 42+2f, 0 deleted, coverage | PASS | 1 P2 + 1 P3 |

---

## Findings

### P2

**G-1 : Release build exe lock non root-cause** —
`cargo build -p nexus-shell-daemon --release` a echoue 2 fois
avec "os error 5 — fichier exe verrouille" pendant la Phase D
review avant de reussir apres rename du binaire. Le processus
responsable n'a pas ete identifie (candidats : antivirus, IDE
indexer, daemon residuel). Carry S60 pour monitoring. Si
reproductible en CI, identifier le processus avec
`handle.exe` (Sysinternals).

### P3

**A-1 : Duplication formule EMA** —
`diagnostic_api.rs:57-60` reimplemente le calcul EMA inline
au lieu d'appeler `kudos_ledger::effective_score()`. Correct
fonctionnellement mais cree un risque maintenance si la formule
change. Non-bloquant : le diagnostic endpoint est un outil
d'observabilite interne, pas un chemin critique.

**B-1 : Pas de test integration happy-path deploy-from-repo** —
Aucun test E2E avec un vrai repo Git (clone → deploy → verify).
Par design : necessite acces reseau, flaky en CI. Le pipeline
est couvert par 11 tests unitaires dans `deploy.rs` + 4 tests
handler dans `http.rs`. Non-actionable.

**D-1 : Naming `author` dans check_write** —
`StorageWriteLimiter::check_write(author, app)` recoit
`state.node_id` (le daemon local) comme `author`. Correct
pour l'architecture single-daemon loopback actuelle. En cas
de multi-author futur, le keying devra etre adapte. Nit
cosmetic.

**F-1 : iroh 1.0.0-rc.0 disponible upstream** —
Le workspace pin iroh 0.98 est maintenant 2 minor versions
derriere (iroh-docs 0.99, iroh-gossip 0.99) et un major
derriere (iroh 1.0.0-rc.0, iroh-blobs 0.101). Pas un defaut
S59 — le pin 0.98 est Day 0 #3 depuis S32. A evaluer dans le
S60 kickoff risk register : si iroh 1.0 stable sort avant le
tag v1.0 SBFB, le projet serait sur une ligne pre-1.0
indefiniment sans sprint upgrade dedie.

**G-2 : Verification doc sub-delta cosmetic** —
La section 3 du verification.md redistribue les sous-deltas
differemment des commit bodies (ex: "A fixes +3" vs commit
body +1 pour `14775f2` + redistribution Phase B). Le total
cumulatif +17 est correct dans les deux representations.
Deja auto-flagge en Phase D review. Nit cosmetic.

---

## Checks security transversaux (hors tracks)

| Check | Resultat |
|-------|----------|
| `unwrap()` en code production | 0 (tous en code test) |
| `unsafe` nouveau | 2 blocs (launcher FFI MessageBoxW — verifie Track C) |
| Secrets dans le diff | 0 |
| `panic!` en production | 0 (1 en test assert) |
| `#[allow(dead_code)]` / `cfg(not(test))` | 0 |
| `#[serde(default)]` ajoute | 0 |
| `_FORMAT_VERSION` modifie | 0 |

---

## Compteurs tests confirmes (live run)

| Suite | Annonce S59 | Live run | Match |
|-------|-------------|----------|-------|
| Rust nextest | 1257 | 1257 pass, 0 fail | oui |
| Vitest | 258 | 258 pass, 0 fail | oui |
| Playwright | 42+2f | 42+2f (env pre-existant S27+) | oui |
| size-limit | 6/6 | 6/6 | oui |

---

## Recommendation

**Commit autorise.** Sprint 60 peut ouvrir. Les 2 carries
residuels (P2-A-1 rand, P2-AUDIT-2 iroh) restent avec
exemptions justifiees. Le P2 G-1 (exe lock) est un item
dev-env a monitorer S60, pas un defaut code. L'advisory F-1
(iroh 1.0.0-rc.0) merite une mention dans le risk register
S60 mais ne bloque pas le gate.
