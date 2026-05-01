# Sprint 48 — Design Review Board (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Mandat** : scorer la qualite des D-decisions Day 0, PAS
verifier si le code est deja implemente (c'est le role du sprint
lui-meme).

---

## Scoring

| Decision | Verdict | Raison |
|---|---|---|
| D1 | ✅ | Code source verifie (canary_input.rs:514,541 drop avant read confirme). Fix mutex-hold trivial, alternatives comparees. |
| D2 | ✅ | Issue confirmee (kudos_api.rs:76 count=page_size). Fix total_count avant skip/take correct, alternatives SQL comparees. |
| D3 | ⚠️ | Pattern feature gate idiomatique Rust, mais cfg(test) cross-crate est une limitation connue non documentee dans le kickoff. |
| D4 | ✅ | State-passing est le pattern Rust canonique vs process-global env var. Alternatives comparees (temp_env, unsafe). |

Rigor signal G4 satisfait (1 ⚠️ sur 4, 0 ❌).

---

## Detail par decision

### D1 — TOCTOU canary reload : mutex-hold-across-read ✅

**Verification factuelle** : le pattern TOCTOU est confirme dans
`canary_input.rs`. `reload_policy()` ligne 514 fait `drop(rs)`
avant `read_to_string()` ligne 515. Meme pattern dans
`reload_set()` lignes 541-542. Le fix propose (garder le lock
pendant le read) est le pattern standard pour ce type de race.

**Alternatives** : ArcSwap (dep externe) et read-then-compare
(inverse semantique) sont correctement rejetes. Le mutex hold
~1ms est negligeable pour un reload periodique.

**Verdict** : decision factuelle, code source recent, fix
structurellement correct.

### D2 — kudos SQL pagination : total_count avant skip/take ✅

**Verification factuelle** : `kudos_api.rs` lignes 59-76 fait
`skip().take()` puis `count = entries.len()` — c'est le count de
page, pas le total. Le frontend `KudosTab.tsx:41` affiche ce
count comme total, ce qui est trompeur avec pagination active.

**Alternatives** : SQL COUNT(*) correctement rejete (toutes les
lignes sont deja en memoire). LIMIT/OFFSET SQL correctement
differe (refactoring plus large). Le fix Rust-side est le plus
simple et correct.

**Verdict** : fix UX direct, pas de choix architectural lourd.

### D3 — execute_batch_raw : feature gate test-support ⚠️

**Verification factuelle** : `db.rs:348-352` est `pub` avec
`#[doc(hidden)]`. Le seul caller cross-crate est le test
`http.rs:4522`. Le `Cargo.toml` de nexus-coordinator-rs n'a
pas de section `[features]`.

**Angle mort** : le kickoff ne documente pas explicitement que
`#[cfg(test)]` en Rust est **per-crate** — un attribut
`#[cfg(test)]` sur une methode dans le crate A n'est pas visible
depuis le crate B meme en mode test. C'est pourquoi la feature
gate est necessaire. Ce detail devrait etre mentionne dans le
rationale "Rejete pub(crate)" pour eviter la confusion future
(un lecteur pourrait se demander pourquoi pas simplement
`#[cfg(test)] pub fn`).

**Verdict** : pattern feature gate idiomatique, detail technique
mineur non documente.

### D4 — set_var : refactor sbfb_home dans DaemonHttpState ✅

**Verification factuelle** : 7 appels `set_var("SBFB_HOME", ...)`
confirmes dans http.rs (lignes 4810, 4832, 4869, 4898, 4945,
4970, 5010). DaemonHttpState n'a pas de champ `sbfb_home`.
`consent.rs:15` et `files.rs:23` utilisent la fonction globale
`sbfb_home()`.

**Alternatives** : temp_env (wrapper cosmétique) et unsafe
set_var (accepte le risque) correctement rejetes. State-passing
est le pattern Rust standard pour eviter les mutations globales.

**Verdict** : refactoring propre, eliminera le risque UB Rust
1.81+.

---

## Note methodologique

Le reviewer Explore a initialement score D2/D3/D4 comme ❌
("implementation manquante"). C'est une confusion de mandat : le
G1 review evalue la **qualite des decisions** (sources, alternatives,
rationale), pas si le code est deja ecrit. Pour un sprint dette
sans choix de lib externe, les D-decisions sont des choix de
pattern de fix — la "source" est le code existant a corriger,
pas une release upstream.
