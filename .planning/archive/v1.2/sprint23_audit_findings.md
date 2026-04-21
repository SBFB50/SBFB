# Sprint 23 — Audit Findings

**Auditeur** : session fraîche Claude Opus 4.6 (1M context),
2026-04-21. 3 agents parallèles (Tracks A+B, C+D, E+cross) +
vérification directe Track F process/meta.

**Tip audité** : `30b0308` (Phase F wrap-up S23).
**Range audité** : `2438c59..30b0308` (12 commits S23).
**D1-D5 gelées** : non rebattues.

---

## Verdict : CONDITIONAL PASS

**1 P1** fixable avant S24 Phase A. 3 P2 à logger tech debt.
7 P3 laissés sans action.

**Condition de levée** : commit `fix(sprint23): exclude
redundancy_factor from canonical bytes (R3 mitigation)` atterri
sur master AVANT le premier commit `feat(sprint24): Phase A`.

---

## P1 — Significatifs

### C-1 : `redundancy_factor` inclus dans canonical bytes (violation plan R3)

**Fichier** : `crates/nexus-core-rs/src/task.rs:155-156`
**Plan** : §13 R3 — "`redundancy_factor` exclu du canonical bytes
(champ dispatch-only, pas task identity)"
**Code actuel** :
```rust
#[serde(default = "default_redundancy_factor")]
pub redundancy_factor: u8,
```
Le champ est un `pub` field régulier sans `#[serde(skip)]` — il
participe à `canonical_bytes()` via `serde_jcs::to_vec`. Le test
`task_wire_redundancy_factor` (task.rs:684-689) confirme le
roundtrip canonical inclut le champ :
```rust
let bytes = canonical_bytes(&t, DOMAIN_TASK_V1).unwrap();
let body = &bytes[DOMAIN_TASK_V1.len() + 1..];
let restored: Task = serde_json::from_slice(body).unwrap();
assert_eq!(restored.redundancy_factor, 3);
```
**Impact** : deux coordinateurs signant la même tâche logique avec
des `redundancy_factor` différents produiraient des signatures
différentes. Le champ est une politique de dispatch, pas une identité
de tâche. Pré-launch, le coût de correction est nul (redéfinir v1).
Post-v1.0 ce serait un wire break.

**Fix recommandé** : créer un `TaskCanonical` wrapper (ou
`#[serde(skip)]` avec passage du factor par canal séparé dispatch-
only) pour exclure `redundancy_factor` du JCS sérialisé. Mettre
à jour le test pour vérifier l'exclusion. Le champ reste dans le
wire JSON normal (API, gossip) mais pas dans les canonical bytes
de signature.

---

## P2 — Mineurs

### B-1 : truncation `exponent as i32` dans `escalating_difficulty()`

**Fichier** : `crates/nexus-core-rs/src/pow.rs:484`
```rust
let factor = policy.multiplier.powi(exponent as i32);
```
`exponent` est `u64` (= `task_count / tranche_size`). Si `exponent
> i32::MAX` (~2.1 milliards), le cast wrappe en négatif, `powi(neg)`
→ ~0.0, et le floor `.max(base_difficulty)` ramène la difficulté
à la base au lieu de la capper au max. Pratiquement inexploitable
(daily reset + débit réel ~100K tasks/jour), mais l'invariant codé
("overflow saturates max") n'est pas respecté.

**Fix suggéré** : `let exponent = exponent.min(i32::MAX as u64);`
avant le cast, ou `powf(exponent as f64)`. Tech debt S24.

### C-2 : SHA-256 au lieu de BLAKE3 pour le hash de comparaison redundancy

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/redundancy.py:44-45`
```python
def hash_result_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()
```
Le kickoff §4 D3 spécifie "majority vote (bitwise hash comparison
BLAKE3 sur le résultat canonique)". L'implémentation utilise SHA-256
(stdlib, zero dep). Le module docstring (L9-12) documente ce choix
comme délibéré. Fonctionnellement équivalent pour de la comparaison
d'égalité (pas d'usage intégrité crypto — les Ed25519 sigs font ce
travail). Mais déviation du texte D3 gelé.

**Action** : documenter la déviation dans le kickoff S24 carry notes.
Si BLAKE3 requis pour cohérence, `from blake3 import blake3` est déjà
une dep workspace Rust (crate `blake3`), mais il faudrait l'ajouter
côté Python (`pip install blake3`).

### F-1 : PATTERNS.md manque P35 (ephemeral) et P36 (redundancy)

**Fichier** : `docs/rust/PATTERNS.md` (arrêt à P34)
Le plan §14 checkpoint dit explicitement : "PATTERNS.md mis à jour
(§P33 + nouveau §P35 ephemeral + §P36 redundancy)". P33 a été mis
à jour (Phase A), mais P35 et P36 ne sont jamais écrits. Aucun
pattern S23 ajouté non plus dans `docs/shell/PATTERNS.md` (0 diff
S23 sur ce fichier).

**Action** : absorber en S24 Phase A — écrire P35 ephemeral lifecycle
+ P36 redundancy voting + patterns coord-side fairness/honeypot dans
shell/PATTERNS.md.

---

## P3 — Observations

### A-1 : wrapping u32 arithmetic sur completed_count

`ephemeral.rs:95` — `self.completed_count += 1` sans `saturating_add`.
Si `max_tasks` était ignoré et le compteur atteignait `u32::MAX`, il
wrappe à 0. Théorique : le state machine transite `RestartPending`
bien avant. Aucune action requise.

### A-2 : start_task/complete_task naming trompeur

`runtime.rs:1121-1123` — `start_task()` et `complete_task()` appelés
dos-à-dos APRÈS le task réel. L'état "Running" dure quelques µs, pas
la durée de l'inférence. Naming confus mais comportement correct.
Aucune action requise.

### B-2 : test overflow ne couvre pas le cast i32

`pow.rs:816-825` — `test_escalating_difficulty_overflow_saturates_max`
utilise `task_count = 100` qui tient dans `i32`. N'exerce pas le bug
B-1. Aucune action requise (le fix B-1 corrigerait le chemin).

### B-3 : Python `assert` au lieu d'exception dans pow_counter.py

`pow_counter.py:71,105,121` — `assert self._db is not None` strippé
par `python -O`. FastAPI/uvicorn n'utilise pas `-O` en pratique.
Risque minimal.

### D-1 : pas de test à exactement 80% du seuil eclipse

`test_honeypot.py` teste 60% (below) et 90% (above) mais pas 80%
exact. Le code utilise `>=` (`honeypot.py:140`), donc 80% exact
déclenche. Test implicite correct, test explicite absent.

### E-1 : référence cassée `kudos/` dans design docs

`docs/fairness/CONTRIBUTION_FAMILIES_V1.md:186` et
`docs/fairness/KUDOS_V2_WIRE.md:167` référencent `kudos/` (répertoire)
au lieu de `kudos.py` (fichier). Docs design-only, 0 impact code.

### E-2 : import `time` lazy dans diagnostic.py

`packages/nexus-coordinator/src/nexus_coordinator/api/diagnostic.py:81`
— `import time` à l'intérieur du corps de fonction, contrairement
au reste des fichiers S23 qui importent au module-level. Style
inconsistant, pas de bug.

---

## Items connus (carry audit_plan S24)

Les items P2-D-1 / P2-D-2 / P2-E-1 / P2-E-2 / P2-E-3 / P2-F-1
documentés dans `sprint23_audit_plan.md §3` sont confirmés comme
carry — non rebattus dans cet audit. Ils n'apparaissent pas comme
findings car ils étaient des scope cuts explicites.

---

## Tracks audit

| Track | Sujet | Phase | Verdict | Findings |
|---|---|---|---|---|
| A | Ephemeral lifecycle | B | PASS | 2 P3 |
| B | PoW escalating | C | CONCERN | 1 P2 + 2 P3 |
| C | Redundancy voting | D | **CONCERN** | **1 P1** + 1 P2 |
| D | Honeypot eclipse | E | PASS | 1 P3 |
| E | DelegationCert | F | PASS | 2 P3 |
| F | Process / meta | all | CONCERN | 1 P2 |

---

## Working tree pré-audit

Un artefact stale détecté avant audit :
`.planning/active/sprint23_phase_F_preflight.md` restait dans active/
(copié en archive/ mais non supprimé lors du Phase F migration).
Corrigé par commit `0c415d3` avant le début de l'audit.

---

## Commits fix attendus

1. `fix(sprint23): exclude redundancy_factor from canonical bytes`
   — créer TaskCanonical ou serde skip, mettre à jour test
   `task_wire_redundancy_factor` pour vérifier l'exclusion

Après ce fix, verdict relevé à **PASS** et S24 Phase A peut
démarrer.

---

## Notes on audit completeness

- Suites de tests non rejouées intégralement (timebox audit gate).
  Les compteurs verification.md (~1561 total) acceptés sur la base
  des commit bodies détaillés et de la cohérence interne.
- 32 failures coord pre-existing (PyO3 wheel stale) confirmées
  comme environnement local, pas régression code. Tracké P2-F-1.
- Semgrep non installé sur cet environnement — fallback grep
  patterns critiques (unwrap, panic, secrets, path traversal).
  0 finding security.
