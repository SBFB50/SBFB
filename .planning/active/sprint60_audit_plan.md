# Sprint 60 — Audit plan (prepare par S59 Phase D)

**Ecrit** : 2026-05-11
**Sprint audite** : Sprint 59
**Tip de sortie S59** : `4d0f7b2` (pre-wrap-up)

---

## §1 Contexte

Sprint 59 = dernier sprint feature avant S60 (installer + tag v1.0).
Theme : launcher readiness + verified deploy E2E + LT-1 Kudos-v2
fairness reform + storage carries. 3 feat phases (A+B+C) + 4 fix
commits + 1 wrap-up (D). Compteurs : 1240→1257 Rust (+17), 256→258
Vitest (+2). 3 items CLOSED (LT-1, STORAGE-JOIN-VALIDATE,
STORAGE-ANTISPAM). 14/14 scope cuts respectes.

---

## §2 Tracks d'audit

### Track A — LT-1 Kudos-v2 formule correctness

Verifier que la formule log-utility + EMA est correctement
implementee et que le hash chain reste valide :

1. `credit()` dans `kudos_ledger.rs` : la formule
   `floor(1000 * log2(1 + tokens))` est-elle correcte ?
   Cas limites : tokens=0 → amount >= 1 ? tokens=u64::MAX ?
2. `effective_score()` : alpha=0.97, decay correct pour ages
   extremes (0j, 1j, 365j, 3650j) ? Pas d'overflow f64→u64 ?
3. `verify_chain()` : toujours valide apres log-transform ?
   Les entries existantes (pre-S59) sont-elles compatibles ?
4. `compute_gini()` / `compute_top_k_share()` dans `fairness.rs` :
   consomment bien effective scores (pas raw amounts) ?
5. `kudos_api.rs` handlers : passent bien `now_secs` a
   `get_project_kudos()` ?

### Track B — Verified deploy E2E wiring

Verifier que le flow deploy est teste de bout en bout :

1. `SBFB.json` dans les 2 apps exemples : structure correcte ?
   `node_id` = "PLACEHOLDER" (pas de vraie cle hardcodee) ?
2. Tests deploy dans `http.rs` : couvrent-ils les cas d'erreur
   (URL invalide, SHA invalide, SBFB.json absent) ?
3. `Deploy.tsx` : formulaire React fonctionne ? Route dans App.tsx ?
   Lien dans la navigation sidebar ?
4. `sync-bridge-sdk.sh` : copie correctement SBFB.json + bridge ?

### Track C — Launcher MessageBox UX

Verifier l'implementation FFI Windows :

1. `error_msgbox()` dans `launcher/main.rs` : FFI correct ?
   UTF-16 encoding correct (encode_utf16 + null terminator) ?
   HWND = null_mut ? MB_ICONERROR | MB_OK ?
2. Appels dans les 5 chemins d'erreur (spawn failure, port occupe,
   identity non initialisee, etc.) ?
3. `cfg(windows)` + `eprintln` fallback `cfg(not(windows))` ?
4. Pas de regression `#![windows_subsystem = "windows"]` en release ?

### Track D — Storage validation + rate-limit

Verifier les 2 carries resolus :

1. `storage_api.rs` : `is_replicated_app()` appele dans
   `storage_join` handler ? Rejet correct (400) pour app non
   repliquee ?
2. `storage_limiter.rs` : `StorageWriteLimiter` keyed
   (node_id, app_name) ? GCRA correct ? Quota 10 writes/min ?
3. `http.rs` : rate-limit applique sur `storage_set` et
   `storage_delete` ? Rejet 429 correct ?
4. Test handler HTTP pour 400 non-replicated et 429 rate-limited ?

### Track E — Pre-launch protocol compliance

Verifier que les changements S59 respectent le pre-launch protocol :

1. Aucun `*_FORMAT_VERSION` bumpe ?
2. Aucun tolerant decoder multi-version introduit ?
3. `#[serde(default)]` ajoutes sont documentes runtime tolerance ?
4. Wire formats inchanges (TaskEntry, ProjectAnnouncement,
   CuratorList, etc.) ?
5. Day 0 figees S59 (D1-D4) respectees dans l'implementation ?

### Track F — Carries residuels S60

1. P2-A-1 rand : toujours bloque upstream ? Verifier crates.io
   `rand` latest compatible iroh 0.98.
2. P2-AUDIT-2 iroh transitives : toujours herite pin 0.98 ?
   Verifier si iroh 0.99+ est sorti.
3. Pas de carry cache : tous les items planifies S59 ont ete
   livres ou scope-cut (14/14).

### Track G — Delta tests et regression

1. Delta Rust +17 (1240→1257) : coherent avec les commit bodies ?
2. Delta Vitest +2 (256→258) : coherent ?
3. Playwright 42+2f : les 2 env fail pre-existants sont-ils
   toujours les memes ?
4. Aucun test supprime ?
5. Coverage tests sur les nouvelles fonctions : credit() formula,
   effective_score(), storage validation, rate-limit ?

---

## §3 Rappel processus

L'audit gate Sprint 59 est joue en Phase 0 du Sprint 60 par une
session fraiche qui n'a pas participe a l'implementation S59.
Le verdict est PASS, CONDITIONAL PASS, ou FAIL.
- PASS : 0 P0, 0 P1, findings P2+ documentes
- CONDITIONAL PASS : 0 P0, <= 2 P1 avec fix immediats
- FAIL : >= 1 P0 ou >= 3 P1

Les findings P0/P1 sont resolus par commit `fix(sprint59): ...`
AVANT d'ouvrir le Sprint 60.
